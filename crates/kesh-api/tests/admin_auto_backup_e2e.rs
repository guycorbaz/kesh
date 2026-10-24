//! Story 31-1c (#503) — la sauvegarde automatique faite par Kesh : écriture,
//! relecture, rétention, disque plein, verrou, routes, audits (AC24, AC28 à
//! AC32, AC35, AC36).
//!
//! `run_once` est appelé directement avec un environnement injecté
//! ([`TestEnv`] : espace libre et écrivain simulés) et un `now` choisi ; les
//! routes passent par un serveur monté sur **le même** `AppState` (même
//! verrou `backup_lock`). Données synthétiques seulement.

use std::fs::File;
use std::io::{self, Seek, SeekFrom, Write};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use chrono::{NaiveDateTime, TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::auth::password::hash_password;
use kesh_api::auto_backup::{
    BackupEnv, FailureStatus, RunOutcome, RunStatus, SuccessStatus, Trigger, WriteSeek,
    nom_de_sauvegarde, run_once,
};
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::{NewUser, Role};
use kesh_db::repositories::backup_schedule;
use kesh_db::repositories::users;
use reqwest::multipart;
use serde_json::{Value, json};
use sqlx::MySqlPool;

#[path = "support/pieces.rs"]
mod pieces;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";

// ============================================================================
// Montage
// ============================================================================

/// Environnement injecté : espace libre, et écrivain qui échoue ou tronque
/// après `limite` octets.
#[derive(Clone, Copy)]
enum Ecrivain {
    Normal,
    /// `StorageFull` dès que plus de `n` octets ont été écrits.
    PleinApres(u64),
    /// Écrit au plus `n` octets, prétend avoir écrit le reste (archive
    /// tronquée).
    TronqueApres(u64),
}

struct TestEnv {
    libre: u64,
    ecrivain: Ecrivain,
}

impl TestEnv {
    fn large() -> Arc<dyn BackupEnv> {
        Arc::new(Self {
            libre: u64::MAX,
            ecrivain: Ecrivain::Normal,
        })
    }
    fn avec(libre: u64, ecrivain: Ecrivain) -> Arc<dyn BackupEnv> {
        Arc::new(Self { libre, ecrivain })
    }
}

/// L'écrivain de test : possède le clone du descripteur.
struct EcrivainDeTest {
    fichier: File,
    ecrit: u64,
    mode: Ecrivain,
}

impl Write for EcrivainDeTest {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.mode {
            Ecrivain::Normal => self.fichier.write(buf),
            Ecrivain::PleinApres(n) => {
                if self.ecrit + buf.len() as u64 > n {
                    return Err(io::Error::from(io::ErrorKind::StorageFull));
                }
                let w = self.fichier.write(buf)?;
                self.ecrit += w as u64;
                Ok(w)
            }
            Ecrivain::TronqueApres(n) => {
                let pos = self.fichier.stream_position()?;
                if pos >= n {
                    self.fichier.seek(SeekFrom::Current(buf.len() as i64))?;
                    return Ok(buf.len());
                }
                let garde = usize::try_from(n - pos)
                    .unwrap_or(usize::MAX)
                    .min(buf.len());
                self.fichier.write_all(&buf[..garde])?;
                self.fichier
                    .seek(SeekFrom::Current((buf.len() - garde) as i64))?;
                Ok(buf.len())
            }
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        self.fichier.flush()
    }
}

impl Seek for EcrivainDeTest {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.fichier.seek(pos)
    }
}

impl BackupEnv for TestEnv {
    fn free_bytes(&self, _dir: &Path) -> io::Result<u64> {
        Ok(self.libre)
    }
    fn wrap_writer(&self, file: File) -> Box<dyn WriteSeek> {
        Box::new(EcrivainDeTest {
            fichier: file,
            ecrit: 0,
            mode: self.ecrivain,
        })
    }
}

fn test_config(documents: &Path, backup: &Path) -> Config {
    let mut config = Config::from_fields_for_test(
        "mysql://test:test@localhost:3306/test".to_string(),
        "admin".to_string(),
        "e2e-test-admin-password".to_string(),
        String::from_utf8(TEST_JWT_SECRET.to_vec()).unwrap(),
        TimeDelta::minutes(15),
        TimeDelta::days(30),
        TimeDelta::minutes(15),
        TimeDelta::minutes(15),
        100,
        TimeDelta::minutes(30),
        12,
    );
    config.documents_dir = documents.to_string_lossy().to_string();
    config.admin_backup_dir = backup.to_string_lossy().to_string();
    config
}

/// Une installation de test : société, facture validée, un document
/// fournisseur, dossiers propres ; serveur monté sur l'état partagé.
struct Install {
    state: AppState,
    base_url: String,
    client: reqwest::Client,
    jwt: String,
    company_id: i64,
    documents: tempfile::TempDir,
    backup: tempfile::TempDir,
    document_path: String,
}

impl Install {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
    fn auto(&self) -> PathBuf {
        self.backup.path().join("auto")
    }
    async fn get(&self, path: &str) -> reqwest::Response {
        self.client
            .get(self.url(path))
            .header("Authorization", format!("Bearer {}", self.jwt))
            .send()
            .await
            .unwrap()
    }
    async fn put_schedule(&self, body: Value) -> reqwest::Response {
        self.client
            .put(self.url("/api/v1/admin/backup-schedule"))
            .header("Authorization", format!("Bearer {}", self.jwt))
            .json(&body)
            .send()
            .await
            .unwrap()
    }
    async fn schedule(&self) -> Value {
        let r = self.get("/api/v1/admin/backup-schedule").await;
        assert_eq!(r.status(), 200);
        r.json().await.unwrap()
    }
    /// Noms présents dans `auto/`, triés.
    fn noms_auto(&self) -> Vec<String> {
        let mut noms: Vec<String> = std::fs::read_dir(self.auto())
            .map(|it| {
                it.map(|e| e.unwrap().file_name().to_string_lossy().to_string())
                    .collect()
            })
            .unwrap_or_default();
        noms.sort();
        noms
    }
    /// Dépose `n` sauvegardes automatiques anciennes (janvier 2026).
    fn deposer(&self, n: u32) -> Vec<String> {
        std::fs::create_dir_all(self.auto()).unwrap();
        (1..=n)
            .map(|i| {
                let nom = format!("kesh-auto-202601{i:02}T000000000Z.keshbackup");
                std::fs::write(self.auto().join(&nom), b"ancienne").unwrap();
                nom
            })
            .collect()
    }
    async fn retention(&self, n: i32) {
        sqlx::query("UPDATE _kesh_backup_schedule SET retention_count = ? WHERE id = 1")
            .bind(n)
            .execute(&self.state.pool)
            .await
            .unwrap();
    }
}

fn forge_jwt(user_id: i64, role: &str, company_id: i64) -> String {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        company_id,
        iat: now,
        exp: now + 3600,
    };
    jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(TEST_JWT_SECRET),
    )
    .unwrap()
}

async fn monter_avec(pool: MySqlPool, ajuster: impl FnOnce(&mut Config)) -> Install {
    let documents = pieces::dossier();
    let backup = pieces::dossier();
    let (seeded, _invoice) = pieces::seed_company_with_invoice(&pool).await;
    let (_id, document_path) = pieces::seed_document(
        &pool,
        seeded.company_id,
        documents.path(),
        b"%PDF-1.4 piece fictive de la story 31-1c",
        "pdf",
    )
    .await;
    let mut config = test_config(documents.path(), backup.path());
    ajuster(&mut config);
    let rate_limiter = kesh_api::middleware::rate_limit::RateLimiter::new(&config);
    let i18n = Arc::new(
        kesh_i18n::I18nBundle::load(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("kesh-i18n/locales")
                .as_path(),
        )
        .expect("i18n"),
    );
    let state = AppState::new_for_tests(pool, Arc::new(config), Arc::new(rate_limiter), i18n);
    let app = build_router(state.clone(), "nonexistent-static-dir".to_string());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    let jwt = forge_jwt(seeded.admin_user_id, "Admin", seeded.company_id);
    Install {
        state,
        base_url: format!("http://{addr}"),
        client: reqwest::Client::new(),
        jwt,
        company_id: seeded.company_id,
        documents,
        backup,
        document_path,
    }
}

async fn monter(pool: MySqlPool) -> Install {
    monter_avec(pool, |_| {}).await
}

/// L'heure présente (à la milliseconde, comme la colonne `DATETIME(3)`),
/// décalée de `ms` millisecondes.
fn instant(ms: i64) -> NaiveDateTime {
    use chrono::SubsecRound;
    Utc::now().naive_utc().trunc_subsecs(3) + TimeDelta::milliseconds(ms)
}

/// Une date rendue par l'API.
fn date(v: &Value) -> NaiveDateTime {
    chrono::DateTime::parse_from_rfc3339(v.as_str().expect("date"))
        .expect("RFC 3339")
        .naive_utc()
}

async fn reglage(pool: &MySqlPool) -> backup_schedule::BackupSchedule {
    backup_schedule::get(pool).await.unwrap()
}

async fn audits(pool: &MySqlPool, action: &str) -> Vec<Value> {
    let lignes: Vec<Option<Value>> =
        sqlx::query_scalar("SELECT details_json FROM audit_log WHERE action = ? ORDER BY id")
            .bind(action)
            .fetch_all(pool)
            .await
            .unwrap();
    lignes
        .into_iter()
        .map(|v| v.unwrap_or(Value::Null))
        .collect()
}

const OK: RunOutcome = RunOutcome::Done(RunStatus::Success(SuccessStatus::Ok));

// ============================================================================
// Succès, importabilité (AC28, AC36)
// ============================================================================

/// Succès : fichier `0600`, dossier `0700`, état `ok`, dernière complète ; et
/// l'archive s'importe par `POST /api/v1/admin/full-import`, qui restaure la
/// pièce retirée entre-temps du dossier des documents.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn succes_puis_archive_importable(pool: MySqlPool) {
    let i = monter(pool.clone()).await;
    let now = instant(0);
    let issue = run_once(&i.state, TestEnv::large(), now, Trigger::Schedule)
        .await
        .unwrap();
    assert_eq!(issue, OK);

    let nom = nom_de_sauvegarde(now);
    assert_eq!(i.noms_auto(), vec![nom.clone()]);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = |p: &Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&i.auto()), 0o700, "dossier auto/ en 0700");
        assert_eq!(mode(&i.auto().join(&nom)), 0o600, "sauvegarde en 0600");
    }
    let s = reglage(&pool).await;
    assert_eq!(s.last_status.as_deref(), Some("ok"));
    assert_eq!(s.last_success_file.as_deref(), Some(nom.as_str()));
    assert_eq!(s.last_complete_file.as_deref(), Some(nom.as_str()));
    assert_eq!(s.last_files_not_included, Some(0));
    assert_eq!(s.last_attempt_at, Some(now));
    assert!(s.last_success_bytes.unwrap() > 0);

    // La pièce disparaît du dossier des documents ; l'import la rend.
    let piece = i.documents.path().join(&i.document_path);
    std::fs::remove_file(&piece).unwrap();
    let octets = std::fs::read(i.auto().join(&nom)).unwrap();
    let form = multipart::Form::new().part(
        "file",
        multipart::Part::bytes(octets)
            .file_name("auto.keshbackup")
            .mime_str("application/octet-stream")
            .unwrap(),
    );
    let r = i
        .client
        .post(i.url("/api/v1/admin/full-import"))
        .header("Authorization", format!("Bearer {}", i.jwt))
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "{:?}", r.text().await);
    let corps: Value = r.json().await.unwrap();
    assert!(corps["filesRestored"].as_u64().unwrap() >= 1, "{corps}");
    assert_eq!(corps["filesMissing"], 0, "{corps}");
    assert!(piece.exists(), "la pièce est restaurée par l'import");
}

/// AC24 — un import ne touche pas la table système : réglage et état de la
/// destination conservés.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn import_conserve_le_reglage_de_la_destination(pool: MySqlPool) {
    let i = monter(pool.clone()).await;
    run_once(&i.state, TestEnv::large(), instant(0), Trigger::Manual)
        .await
        .unwrap();
    let nom = reglage(&pool).await.last_success_file.unwrap();
    sqlx::query(
        "UPDATE _kesh_backup_schedule SET enabled = FALSE, interval_hours = 168, \
         retention_count = 42, last_status = 'disk_full' WHERE id = 1",
    )
    .execute(&pool)
    .await
    .unwrap();
    let avant = reglage(&pool).await;
    let octets = std::fs::read(i.auto().join(&nom)).unwrap();
    let form = multipart::Form::new().part(
        "file",
        multipart::Part::bytes(octets).file_name("a.keshbackup"),
    );
    let r = i
        .client
        .post(i.url("/api/v1/admin/full-import"))
        .header("Authorization", format!("Bearer {}", i.jwt))
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    assert_eq!(
        reglage(&pool).await,
        avant,
        "l'import ne modifie pas la table"
    );
}

/// Deux sauvegardes à une milliseconde d'écart : deux fichiers, deux noms.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn deux_sauvegardes_a_une_milliseconde(pool: MySqlPool) {
    let i = monter(pool.clone()).await;
    let now = instant(0);
    for ms in [0, 1] {
        assert_eq!(
            run_once(
                &i.state,
                TestEnv::large(),
                now + TimeDelta::milliseconds(ms),
                Trigger::Manual
            )
            .await
            .unwrap(),
            OK
        );
    }
    assert_eq!(
        i.noms_auto(),
        vec![
            nom_de_sauvegarde(now),
            nom_de_sauvegarde(now + TimeDelta::milliseconds(1))
        ]
    );
}

// ============================================================================
// Rétention (AC29)
// ============================================================================

/// 8 déposées + 1 neuve, `retention_count = 3` → restent les 3 plus récentes ;
/// un `kesh-pre-import-x` et un `autre.keshbackup` déposés dans `auto/` restent
/// intacts (tue M-c3).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn retention_garde_les_plus_recentes(pool: MySqlPool) {
    let i = monter(pool.clone()).await;
    let deposees = i.deposer(8);
    std::fs::write(i.auto().join("kesh-pre-import-x.keshbackup"), b"x").unwrap();
    std::fs::write(i.auto().join("autre.keshbackup"), b"x").unwrap();
    i.retention(3).await;
    let now = instant(0);
    assert_eq!(
        run_once(&i.state, TestEnv::large(), now, Trigger::Schedule)
            .await
            .unwrap(),
        OK
    );
    let mut attendu = vec![
        "autre.keshbackup".to_string(),
        deposees[6].clone(),
        deposees[7].clone(),
        nom_de_sauvegarde(now),
        "kesh-pre-import-x.keshbackup".to_string(),
    ];
    attendu.sort();
    assert_eq!(i.noms_auto(), attendu);
}

/// Sauvegarde incomplète (pièce retirée) → `partial`, dernière complète
/// inchangée ; une suite de `retention_count + 2` sauvegardes `partial` ne la
/// supprime pas ; une sauvegarde complète la remplace et la rétention retrouve
/// son compte (tue M-c11, M-c12).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn partial_protege_la_derniere_complete(pool: MySqlPool) {
    let i = monter(pool.clone()).await;
    i.retention(2).await;
    let t0 = instant(0);
    assert_eq!(
        run_once(&i.state, TestEnv::large(), t0, Trigger::Manual)
            .await
            .unwrap(),
        OK
    );
    let complete = nom_de_sauvegarde(t0);

    let piece = i.documents.path().join(&i.document_path);
    let contenu = std::fs::read(&piece).unwrap();
    std::fs::remove_file(&piece).unwrap();
    for k in 1..=4 {
        let issue = run_once(
            &i.state,
            TestEnv::large(),
            t0 + TimeDelta::seconds(k),
            Trigger::Manual,
        )
        .await
        .unwrap();
        assert_eq!(
            issue,
            RunOutcome::Done(RunStatus::Success(SuccessStatus::Partial))
        );
    }
    let s = reglage(&pool).await;
    assert_eq!(s.last_status.as_deref(), Some("partial"));
    assert_eq!(s.last_files_not_included, Some(1));
    assert_eq!(s.last_complete_file.as_deref(), Some(complete.as_str()));
    let noms = i.noms_auto();
    assert!(
        noms.contains(&complete),
        "la dernière complète survit : {noms:?}"
    );
    assert_eq!(
        noms.len(),
        3,
        "2 conservées + la complète protégée : {noms:?}"
    );

    std::fs::write(&piece, contenu).unwrap();
    let t5 = t0 + TimeDelta::seconds(5);
    assert_eq!(
        run_once(&i.state, TestEnv::large(), t5, Trigger::Manual)
            .await
            .unwrap(),
        OK
    );
    let s = reglage(&pool).await;
    assert_eq!(
        s.last_complete_file.as_deref(),
        Some(nom_de_sauvegarde(t5).as_str())
    );
    let noms = i.noms_auto();
    assert_eq!(noms.len(), 2, "la rétention retrouve son compte : {noms:?}");
    assert!(!noms.contains(&complete));
}

// ============================================================================
// Espace, disque plein, échecs (AC30, AC28)
// ============================================================================

/// Espace insuffisant (seuil injecté) → `insufficient_space`, aucun fichier
/// neuf, les 8 déposées restent 8 sous `retention_count = 3` (tue M-c4, M-c5).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn espace_insuffisant(pool: MySqlPool) {
    let i = monter(pool.clone()).await;
    let deposees = i.deposer(8);
    i.retention(3).await;
    let issue = run_once(
        &i.state,
        TestEnv::avec(1024, Ecrivain::Normal),
        instant(0),
        Trigger::Schedule,
    )
    .await
    .unwrap();
    assert_eq!(
        issue,
        RunOutcome::Done(RunStatus::Failure(FailureStatus::InsufficientSpace))
    );
    assert_eq!(i.noms_auto(), deposees);
    let s = reglage(&pool).await;
    assert_eq!(s.last_status.as_deref(), Some("insufficient_space"));
    assert_eq!(s.last_success_at, None);
}

/// Première sauvegarde sans historique : le seuil compte le dossier des
/// documents (tue M-c19) — 256 Mio + 50 octets libres, des documents de 100
/// octets → refus.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn premiere_sauvegarde_compte_les_documents(pool: MySqlPool) {
    let i = monter(pool.clone()).await;
    let total: u64 = std::fs::read_dir(i.documents.path())
        .unwrap()
        .map(|e| e.unwrap().metadata().unwrap().len())
        .sum();
    if total < 100 {
        std::fs::write(i.documents.path().join("bourrage.bin"), vec![0u8; 100]).unwrap();
    }
    assert_eq!(
        reglage(&pool).await.last_success_bytes,
        None,
        "montage : sans historique"
    );
    let libre = 256 * 1024 * 1024 + 50;
    let issue = run_once(
        &i.state,
        TestEnv::avec(libre, Ecrivain::Normal),
        instant(0),
        Trigger::Schedule,
    )
    .await
    .unwrap();
    assert_eq!(
        issue,
        RunOutcome::Done(RunStatus::Failure(FailureStatus::InsufficientSpace))
    );
    assert!(i.noms_auto().is_empty());
    // Témoin : sans documents, le même espace suffit.
    for e in std::fs::read_dir(i.documents.path()).unwrap() {
        std::fs::remove_file(e.unwrap().path()).unwrap();
    }
    let issue = run_once(
        &i.state,
        TestEnv::avec(libre, Ecrivain::Normal),
        instant(1),
        Trigger::Schedule,
    )
    .await
    .unwrap();
    assert!(
        matches!(issue, RunOutcome::Done(RunStatus::Success(_))),
        "{issue:?}"
    );
}

/// Disque saturé pendant l'écriture → `disk_full`, `.partial` supprimé, les 8
/// déposées restent 8 (tue M-c13, M-c14).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn disque_sature_pendant_l_ecriture(pool: MySqlPool) {
    let i = monter(pool.clone()).await;
    let deposees = i.deposer(8);
    i.retention(3).await;
    let issue = run_once(
        &i.state,
        TestEnv::avec(u64::MAX, Ecrivain::PleinApres(2048)),
        instant(0),
        Trigger::Schedule,
    )
    .await
    .unwrap();
    assert_eq!(
        issue,
        RunOutcome::Done(RunStatus::Failure(FailureStatus::DiskFull))
    );
    assert_eq!(
        i.noms_auto(),
        deposees,
        "ni fichier neuf, ni .partial, ni suppression"
    );
    assert_eq!(
        reglage(&pool).await.last_status.as_deref(),
        Some("disk_full")
    );
}

/// Archive tronquée (relecture impossible) → `failed`, fichier supprimé, les 8
/// déposées restent 8.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn archive_illisible_supprimee(pool: MySqlPool) {
    let i = monter(pool.clone()).await;
    let deposees = i.deposer(8);
    i.retention(3).await;
    let issue = run_once(
        &i.state,
        TestEnv::avec(u64::MAX, Ecrivain::TronqueApres(512)),
        instant(0),
        Trigger::Schedule,
    )
    .await
    .unwrap();
    assert_eq!(
        issue,
        RunOutcome::Done(RunStatus::Failure(FailureStatus::Failed))
    );
    assert_eq!(i.noms_auto(), deposees);
    assert_eq!(reglage(&pool).await.last_status.as_deref(), Some("failed"));
}

/// Écriture impossible (`auto` remplacé par un fichier) → `failed`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn dossier_auto_inutilisable(pool: MySqlPool) {
    let i = monter(pool.clone()).await;
    std::fs::write(i.auto(), b"un fichier, pas un dossier").unwrap();
    let issue = run_once(&i.state, TestEnv::large(), instant(0), Trigger::Schedule)
        .await
        .unwrap();
    assert_eq!(
        issue,
        RunOutcome::Done(RunStatus::Failure(FailureStatus::Failed))
    );
    assert_eq!(reglage(&pool).await.last_status.as_deref(), Some("failed"));
}

/// Recule la date de modification d'un fichier d'une heure : un `.partial`
/// abandonné (C-31-1-82).
fn vieillir(chemin: &Path) {
    std::fs::File::options()
        .write(true)
        .open(chemin)
        .unwrap()
        .set_modified(std::time::SystemTime::now() - Duration::from_secs(3600))
        .unwrap();
}

/// Un `.partial` abandonné de `auto/` est supprimé au tour suivant, un
/// `.partial` récent (écriture peut-être en cours, autre processus) reste ;
/// un `.partial` pré-import abandonné est supprimé par un import (AC28, F9,
/// C-31-1-82).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn partiels_residuels_supprimes(pool: MySqlPool) {
    let i = monter(pool.clone()).await;
    std::fs::create_dir_all(i.auto()).unwrap();
    let residuel = "kesh-auto-20260101T000000000Z.keshbackup.partial";
    std::fs::write(i.auto().join(residuel), b"interrompu").unwrap();
    vieillir(&i.auto().join(residuel));
    let recent = "kesh-auto-20260102T000000000Z.keshbackup.partial";
    std::fs::write(i.auto().join(recent), b"en cours ailleurs").unwrap();
    let now = instant(0);
    assert_eq!(
        run_once(&i.state, TestEnv::large(), now, Trigger::Manual)
            .await
            .unwrap(),
        OK
    );
    assert_eq!(
        i.noms_auto(),
        vec![recent.to_string(), nom_de_sauvegarde(now)]
    );

    let pre = i
        .backup
        .path()
        .join("kesh-pre-import-20260101T000000000-1-0.keshbackup.partial");
    std::fs::write(&pre, b"interrompu").unwrap();
    vieillir(&pre);
    let octets = std::fs::read(i.auto().join(nom_de_sauvegarde(now))).unwrap();
    let r = i
        .client
        .post(i.url("/api/v1/admin/full-import"))
        .header("Authorization", format!("Bearer {}", i.jwt))
        .multipart(
            multipart::Form::new().part("file", multipart::Part::bytes(octets).file_name("a")),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    assert!(
        !pre.exists(),
        "le .partial pré-import résiduel est supprimé par l'import"
    );
}

// ============================================================================
// Verrou (AC27, AC35)
// ============================================================================

/// Verrou tenu (comme par un import) : `run_once` saute sans attendre ni
/// écrire (tue M-c6) ; `POST …/run` rend 409 `BACKUP_IN_PROGRESS` (tue
/// M-c17) ; après la libération, `run_once` écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn verrou_tenu_la_sauvegarde_saute(pool: MySqlPool) {
    let i = monter(pool.clone()).await;
    let avant = reglage(&pool).await;
    let garde = i.state.backup_lock.clone().lock_owned().await;

    let issue = tokio::time::timeout(
        Duration::from_secs(5),
        run_once(&i.state, TestEnv::large(), instant(0), Trigger::Schedule),
    )
    .await;
    assert!(issue.is_ok(), "la tâche doit sauter, pas attendre");
    assert_eq!(issue.unwrap().unwrap(), RunOutcome::Skipped);
    assert_eq!(reglage(&pool).await, avant, "rien d'écrit en base");
    assert!(i.noms_auto().is_empty(), "rien d'écrit sur disque");

    let r = tokio::time::timeout(
        Duration::from_secs(5),
        i.client
            .post(i.url("/api/v1/admin/backup-schedule/run"))
            .header("Authorization", format!("Bearer {}", i.jwt))
            .send(),
    )
    .await
    .expect("POST …/run ne doit pas attendre le verrou")
    .unwrap();
    assert_eq!(r.status(), 409);
    let corps: Value = r.json().await.unwrap();
    assert_eq!(corps["error"]["code"], "BACKUP_IN_PROGRESS");

    drop(garde);
    assert_eq!(
        run_once(&i.state, TestEnv::large(), instant(1), Trigger::Schedule)
            .await
            .unwrap(),
        OK
    );
}

// ============================================================================
// Routes (AC32)
// ============================================================================

/// `POST …/run` : 200, état rendu, sauvegarde écrite, audit `admin.backup_run`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn sauvegarder_maintenant(pool: MySqlPool) {
    let i = monter(pool.clone()).await;
    let r = i
        .client
        .post(i.url("/api/v1/admin/backup-schedule/run"))
        .header("Authorization", format!("Bearer {}", i.jwt))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let corps: Value = r.json().await.unwrap();
    assert_eq!(corps["lastStatus"], "ok");
    let nom = corps["lastSuccessFile"].as_str().unwrap().to_string();
    assert_eq!(i.noms_auto(), vec![nom.clone()]);
    let a = audits(&pool, "admin.backup_run").await;
    assert_eq!(a.len(), 1);
    assert_eq!(a[0]["status"], "ok");
    assert_eq!(a[0]["file"], nom.as_str());
}

/// `record_*` ne touche pas la version : un `PUT` avec la version lue avant un
/// tour rend 200 (tue M-c15).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn version_inchangee_par_un_tour(pool: MySqlPool) {
    let i = monter(pool.clone()).await;
    let version = i.schedule().await["version"].clone();
    run_once(&i.state, TestEnv::large(), instant(0), Trigger::Schedule)
        .await
        .unwrap();
    let r = i
        .put_schedule(json!({
            "enabled": true, "intervalHours": 12, "retentionCount": 5, "version": version
        }))
        .await;
    assert_eq!(r.status(), 200);
    let corps: Value = r.json().await.unwrap();
    assert_eq!(corps["intervalHours"], 12);
    assert_eq!(corps["version"], version.as_i64().unwrap() + 1);
}

/// `PUT` : 400 hors domaine (sans audit), 409 version périmée, audit exact
/// `before` / `after`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reglage_validation_conflit_et_audit(pool: MySqlPool) {
    let i = monter(pool.clone()).await;
    for corps in [
        json!({"enabled": true, "intervalHours": 5, "retentionCount": 7, "version": 1}),
        json!({"enabled": true, "intervalHours": 24, "retentionCount": 0, "version": 1}),
        json!({"enabled": true, "intervalHours": 24, "retentionCount": 61, "version": 1}),
    ] {
        let r = i.put_schedule(corps.clone()).await;
        assert_eq!(r.status(), 400, "{corps}");
        let e: Value = r.json().await.unwrap();
        assert_eq!(e["error"]["code"], "VALIDATION_ERROR");
    }
    assert!(
        audits(&pool, "admin.backup_schedule_updated")
            .await
            .is_empty()
    );

    let r = i
        .put_schedule(
            json!({"enabled": false, "intervalHours": 168, "retentionCount": 3, "version": 1}),
        )
        .await;
    assert_eq!(r.status(), 200);
    let a = audits(&pool, "admin.backup_schedule_updated").await;
    assert_eq!(a.len(), 1);
    assert_eq!(
        a[0],
        json!({
            "before": {"enabled": true, "interval_hours": 24, "retention_count": 7},
            "after": {"enabled": false, "interval_hours": 168, "retention_count": 3},
        })
    );

    let r = i
        .put_schedule(
            json!({"enabled": true, "intervalHours": 24, "retentionCount": 7, "version": 1}),
        )
        .await;
    assert_eq!(r.status(), 409);
    let e: Value = r.json().await.unwrap();
    assert_eq!(e["error"]["code"], "OPTIMISTIC_LOCK_CONFLICT");
}

/// Un Comptable est refusé (403) sur les cinq routes.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn comptable_refuse(pool: MySqlPool) {
    let i = monter(pool.clone()).await;
    let user = users::create(
        &pool,
        NewUser {
            username: "comptable_311c".into(),
            password_hash: hash_password("password123").unwrap(),
            role: Role::Comptable,
            active: true,
            company_id: i.company_id,
            email: None,
        },
    )
    .await
    .unwrap();
    let jwt = forge_jwt(user.id, "Comptable", i.company_id);
    let send = |m: reqwest::Method, p: &str| {
        i.client
            .request(m, i.url(p))
            .header("Authorization", format!("Bearer {jwt}"))
            .json(&json!({"enabled": true, "intervalHours": 24, "retentionCount": 7, "version": 1}))
            .send()
    };
    for (m, p) in [
        (reqwest::Method::GET, "/api/v1/admin/backup-schedule"),
        (reqwest::Method::PUT, "/api/v1/admin/backup-schedule"),
        (reqwest::Method::POST, "/api/v1/admin/backup-schedule/run"),
        (reqwest::Method::GET, "/api/v1/admin/backups"),
        (
            reqwest::Method::GET,
            "/api/v1/admin/backups/auto/kesh-auto-20260101T000000000Z.keshbackup",
        ),
    ] {
        let r = send(m.clone(), p).await.unwrap();
        assert_eq!(r.status(), 403, "{m} {p}");
    }
    assert!(i.noms_auto().is_empty(), "rien d'écrit par un Comptable");
}

/// `GET backups` : `importMaxMib` de la `Config`, les sauvegardes des deux
/// types, aucun `.partial`, aucun autre fichier, aucun lien symbolique (tue
/// M-c8 avec le test suivant).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn liste_des_sauvegardes(pool: MySqlPool) {
    let i = monter_avec(pool.clone(), |c| c.admin_import_max_mib = 7).await;
    run_once(&i.state, TestEnv::large(), instant(0), Trigger::Manual)
        .await
        .unwrap();
    let auto = reglage(&pool).await.last_success_file.unwrap();
    let pre = "kesh-pre-import-20260101T000000000-1-0.keshbackup";
    std::fs::write(i.backup.path().join(pre), b"pre").unwrap();
    std::fs::write(i.backup.path().join("autre.keshbackup"), b"x").unwrap();
    std::fs::write(
        i.auto()
            .join("kesh-auto-20260101T000000000Z.keshbackup.partial"),
        b"x",
    )
    .unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(
        i.auto().join(&auto),
        i.auto().join("kesh-auto-20260102T000000000Z.keshbackup"),
    )
    .unwrap();

    let r = i.get("/api/v1/admin/backups").await;
    assert_eq!(r.status(), 200);
    let corps: Value = r.json().await.unwrap();
    assert_eq!(corps["importMaxMib"], 7);
    let mut noms: Vec<(String, String)> = corps["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| {
            (
                x["kind"].as_str().unwrap().into(),
                x["name"].as_str().unwrap().into(),
            )
        })
        .collect();
    noms.sort();
    assert_eq!(
        noms,
        vec![
            ("auto".into(), auto.clone()),
            ("pre-import".into(), pre.to_string())
        ]
    );
}

/// Téléchargement : le fichier servi est celui du disque, audit
/// `admin.backup_downloaded` ; 404 sur `../x`, un nom hors expression, un nom
/// absent, un type inconnu, et un lien symbolique au nom conforme.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn telechargement(pool: MySqlPool) {
    let i = monter(pool.clone()).await;
    run_once(&i.state, TestEnv::large(), instant(0), Trigger::Manual)
        .await
        .unwrap();
    let nom = reglage(&pool).await.last_success_file.unwrap();
    let attendu = std::fs::read(i.auto().join(&nom)).unwrap();
    let r = i.get(&format!("/api/v1/admin/backups/auto/{nom}")).await;
    assert_eq!(r.status(), 200);
    assert_eq!(r.bytes().await.unwrap().to_vec(), attendu);
    let a = audits(&pool, "admin.backup_downloaded").await;
    assert_eq!(
        a,
        vec![json!({"name": nom, "kind": "auto", "size": attendu.len()})]
    );

    let lien = "kesh-auto-20260102T000000000Z.keshbackup";
    #[cfg(unix)]
    std::os::unix::fs::symlink(i.auto().join(&nom), i.auto().join(lien)).unwrap();
    for p in [
        "/api/v1/admin/backups/auto/..%2Fx".to_string(),
        "/api/v1/admin/backups/auto/autre.keshbackup".to_string(),
        "/api/v1/admin/backups/auto/kesh-auto-20250101T000000000Z.keshbackup".to_string(),
        format!("/api/v1/admin/backups/autre/{nom}"),
        format!("/api/v1/admin/backups/pre-import/{nom}"),
        format!("/api/v1/admin/backups/auto/{lien}"),
    ] {
        let r = i.get(&p).await;
        assert_eq!(r.status(), 404, "{p}");
    }
}

/// `nextDueAt` = dernière réussite + intervalle après un succès ; = tentative
/// + 1 h après l'échec d'un tour planifié (parti une fois l'intervalle
/// atteint) ; `overdue` vrai pour une réussite datée du futur.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn prochaine_et_retard(pool: MySqlPool) {
    let i = monter(pool.clone()).await;
    let t0 = instant(0);
    run_once(&i.state, TestEnv::large(), t0, Trigger::Schedule)
        .await
        .unwrap();
    let s = i.schedule().await;
    assert_eq!(date(&s["nextDueAt"]), t0 + TimeDelta::hours(24));
    assert_eq!(s["overdue"], false);

    // Un tour planifié qui échoue, la dernière réussite datant d'un jour.
    sqlx::query("UPDATE _kesh_backup_schedule SET last_success_at = ? WHERE id = 1")
        .bind(t0 - TimeDelta::hours(25))
        .execute(&pool)
        .await
        .unwrap();
    let t1 = instant(5);
    let issue = run_once(
        &i.state,
        TestEnv::avec(1, Ecrivain::Normal),
        t1,
        Trigger::Schedule,
    )
    .await
    .unwrap();
    assert_eq!(
        issue,
        RunOutcome::Done(RunStatus::Failure(FailureStatus::InsufficientSpace))
    );
    let s = i.schedule().await;
    assert_eq!(date(&s["nextDueAt"]), t1 + TimeDelta::hours(1));
    assert_eq!(s["lastStatus"], "insufficient_space");

    sqlx::query("UPDATE _kesh_backup_schedule SET last_success_at = ? WHERE id = 1")
        .bind(Utc::now().naive_utc() + TimeDelta::days(3))
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(i.schedule().await["overdue"], true);
}
