//! Story 32-8a (refs #723) — la table `mcp_sends` : ses contraintes (AC-A2),
//! l'état dérivé (AC-A3) et son dépôt (AC-A5 à AC-A9).
//!
//! Les tests de contrainte écrivent en SQL brut, sans passer par le dépôt :
//! c'est MariaDB qui doit refuser, et chaque test l'attend de lui. Les
//! échéances passées sont posées par le pool du test (`NOW(3) - INTERVAL …`),
//! à l'horloge de la base — celle que lit [`ETAT_DERIVE_SQL`].
//!
//! Fixtures synthétiques seulement.

use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::mcp_send::{
    Decision, EnvoiPourCle, ErreurDecision, ErreurDepot, EtatEnvoi, NouvelEnvoi,
};
use kesh_db::entities::{Language, NewCompany, NewUser, OrgType, Role};
use kesh_db::repositories::mcp_sends::{self, LISTE_MAX};
use kesh_db::repositories::{companies, users};
use sqlx::MySqlPool;

const JOUR: u32 = 86_400;

// ---------------------------------------------------------------------------
// Montage
// ---------------------------------------------------------------------------

async fn company(pool: &MySqlPool, name: &str) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: name.into(),
            first_name: None,
            last_name: None,
            address_structured: StructuredAddress {
                street: "Rue Test".into(),
                building: "1".into(),
                postal_code: "1000".into(),
                city: "Lausanne".into(),
                country: "CH".into(),
            },
            ide_number: None,
            org_type: OrgType::Pme,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .expect("société")
    .id
}

async fn user(pool: &MySqlPool, company_id: i64, username: &str) -> i64 {
    users::create(
        pool,
        NewUser {
            username: username.into(),
            password_hash: "$argon2id$v=19$m=19456,t=2,p=1$dGVzdA$dGVzdA".into(),
            role: Role::Admin,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .expect("utilisateur")
    .id
}

/// Une clé MCP posée en SQL brut ; `nom` sert aussi à son condensat.
async fn key(pool: &MySqlPool, company_id: i64, user_id: i64, nom: &str) -> i64 {
    sqlx::query(
        "INSERT INTO mcp_keys (company_id, created_by_user_id, name, key_hash, scopes) \
         VALUES (?, ?, ?, SHA2(?, 256), '[\"books:read\"]')",
    )
    .bind(company_id)
    .bind(user_id)
    .bind(nom)
    .bind(nom)
    .execute(pool)
    .await
    .expect("clé MCP")
    .last_insert_id() as i64
}

/// Société, administrateur et clé : le montage commun.
struct Montage {
    c: i64,
    u: i64,
    k: i64,
}

async fn montage(pool: &MySqlPool, nom: &str) -> Montage {
    let c = company(pool, nom).await;
    let u = user(pool, c, &format!("admin-{nom}")).await;
    let k = key(pool, c, u, &format!("cle-{nom}")).await;
    Montage { c, u, k }
}

/// Une ligne brute. `decide_par` pose `decided_at = NOW(3)` et l'utilisateur
/// ensemble ; `echeance_s` est relatif à `NOW(3)` (négatif : échu). Rend
/// l'identifiant, ou le message de MariaDB si l'insertion est refusée.
#[allow(clippy::too_many_arguments)]
async fn brut(
    pool: &MySqlPool,
    company_id: i64,
    mcp_key_id: i64,
    tool_name: &str,
    state: &str,
    summary: Option<&str>,
    content: Option<&str>,
    modified: bool,
    echeance_s: i64,
    decide_par: Option<i64>,
) -> Result<i64, String> {
    let decided_at = if decide_par.is_some() {
        "NOW(3)"
    } else {
        "NULL"
    };
    let sql = format!(
        "INSERT INTO mcp_sends (company_id, mcp_key_id, tool_name, state, summary, content, \
         modified, expires_at, decided_at, decided_by_user_id) \
         VALUES (?, ?, ?, ?, ?, ?, ?, NOW(3) + INTERVAL ? SECOND, {decided_at}, ?)"
    );
    match sqlx::query(&sql)
        .bind(company_id)
        .bind(mcp_key_id)
        .bind(tool_name)
        .bind(state)
        .bind(summary)
        .bind(content)
        .bind(modified)
        .bind(echeance_s)
        .bind(decide_par)
        .execute(pool)
        .await
    {
        Ok(r) => Ok(r.last_insert_id() as i64),
        Err(sqlx::Error::Database(e)) => Err(e.message().to_string()),
        Err(e) => panic!("erreur inattendue : {e}"),
    }
}

async fn en_attente(pool: &MySqlPool, m: &Montage, content: &str, echeance_s: i64) -> i64 {
    brut(
        pool,
        m.c,
        m.k,
        "outil",
        "pending",
        Some("résumé"),
        Some(content),
        false,
        echeance_s,
        None,
    )
    .await
    .expect("envoi en attente")
}

async fn valide(pool: &MySqlPool, m: &Montage, content: &str, echeance_s: i64) -> i64 {
    brut(
        pool,
        m.c,
        m.k,
        "outil",
        "approved",
        Some("résumé"),
        Some(content),
        false,
        echeance_s,
        Some(m.u),
    )
    .await
    .expect("envoi validé")
}

async fn refuse(pool: &MySqlPool, m: &Montage) -> i64 {
    brut(
        pool,
        m.c,
        m.k,
        "outil",
        "refused",
        None,
        None,
        false,
        3600,
        Some(m.u),
    )
    .await
    .expect("envoi refusé")
}

/// Échéance posée dans le passé par le pool du test.
async fn echoir(pool: &MySqlPool, id: i64) {
    sqlx::query("UPDATE mcp_sends SET expires_at = NOW(3) - INTERVAL 1 SECOND WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
}

fn envoi(m: &Montage, content: &str) -> NouvelEnvoi {
    NouvelEnvoi {
        company_id: m.c,
        mcp_key_id: m.k,
        tool_name: "outil".into(),
        summary: Some("résumé".into()),
        content: content.into(),
    }
}

async fn deposer(pool: &MySqlPool, e: &NouvelEnvoi, plafond: u32) -> Result<i64, ErreurDepot> {
    let mut tx = pool.begin().await.unwrap();
    let r = mcp_sends::deposer_in_tx(&mut tx, e, JOUR, plafond).await;
    tx.commit().await.unwrap();
    r
}

async fn decider(
    pool: &MySqlPool,
    m: &Montage,
    id: i64,
    version: i32,
    d: Decision,
) -> Result<kesh_db::entities::EnvoiDecide, ErreurDecision> {
    let mut tx = pool.begin().await.unwrap();
    let r = mcp_sends::decider_in_tx(&mut tx, m.c, id, version, d, m.u).await;
    tx.commit().await.unwrap();
    r
}

/// `(state, content, summary, modified, version)` lus en base.
async fn ligne(pool: &MySqlPool, id: i64) -> (String, Option<String>, Option<String>, bool, i32) {
    sqlx::query_as("SELECT state, content, summary, modified, version FROM mcp_sends WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn compte(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM mcp_sends")
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Un contenu de 65 536 octets exactement, qui porte des lettres accentuées,
/// des tabulations, une suite LITTÉRALE `\u{202E}` (le rendu visible d'un
/// caractère caché), `⚠` (U+26A0), un idéogramme et une ligne vide finale.
fn contenu_piege() -> String {
    let tete = "Écriture n° 12\tÉlan\tçà\n\\u{202E} ⚠ 帳\n";
    let fin = "\n\n";
    let remplissage = 65_536 - tete.len() - fin.len();
    let s = format!("{tete}{}{fin}", "x".repeat(remplissage));
    assert_eq!(s.len(), 65_536, "montage : 65 536 octets");
    s
}

// ---------------------------------------------------------------------------
// AC-A2 — contraintes de la table
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn une_ligne_valide_s_insere_et_porte_ses_defauts(pool: MySqlPool) {
    let m = montage(&pool, "A").await;
    let id = en_attente(&pool, &m, "contenu", 60).await;
    let (state, content, summary, modified, version) = ligne(&pool, id).await;
    assert_eq!(state, "pending");
    assert_eq!(content.as_deref(), Some("contenu"));
    assert_eq!(summary.as_deref(), Some("résumé"));
    assert!(!modified);
    assert_eq!(version, 1, "défaut de version");
    let created: Option<chrono::NaiveDateTime> =
        sqlx::query_scalar("SELECT created_at FROM mcp_sends WHERE id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(created.is_some(), "défaut de created_at");
}

#[sqlx::test(migrations = "./test-schema")]
async fn state_hors_des_trois_valeurs_exactes_refuse(pool: MySqlPool) {
    let m = montage(&pool, "A").await;
    for faux in ["Pending", "pending ", "PENDING", "expired", ""] {
        let err = brut(
            &pool,
            m.c,
            m.k,
            "outil",
            faux,
            None,
            Some("c"),
            false,
            60,
            None,
        )
        .await
        .expect_err(&format!("{faux:?} devait être refusé"));
        assert!(err.contains("chk_mcp_sends_state"), "{faux:?} : {err}");
    }
    // Témoin : les trois valeurs exactes passent.
    en_attente(&pool, &m, "c", 60).await;
    valide(&pool, &m, "c", 60).await;
    refuse(&pool, &m).await;
}

#[sqlx::test(migrations = "./test-schema")]
async fn decision_partielle_ou_incoherente_refusee(pool: MySqlPool) {
    let m = montage(&pool, "A").await;
    // decided_at posé sans décideur, puis l'inverse.
    for sql in [
        "INSERT INTO mcp_sends (company_id, mcp_key_id, tool_name, state, content, modified, \
         expires_at, decided_at, decided_by_user_id) \
         VALUES (?, ?, 'outil', 'approved', 'c', FALSE, NOW(3), NOW(3), NULL)",
        "INSERT INTO mcp_sends (company_id, mcp_key_id, tool_name, state, content, modified, \
         expires_at, decided_at, decided_by_user_id) \
         VALUES (?, ?, 'outil', 'approved', 'c', FALSE, NOW(3), NULL, ?)",
    ] {
        let q = sqlx::query(sql).bind(m.c).bind(m.k);
        let q = if sql.ends_with("NULL, ?)") {
            q.bind(m.u)
        } else {
            q
        };
        let err = q.execute(&pool).await.expect_err(sql);
        assert!(err.to_string().contains("chk_mcp_sends_decision"), "{err}");
    }
    // En attente avec décision ; validé ou refusé sans décision.
    let err = brut(
        &pool,
        m.c,
        m.k,
        "outil",
        "pending",
        None,
        Some("c"),
        false,
        60,
        Some(m.u),
    )
    .await
    .unwrap_err();
    assert!(err.contains("chk_mcp_sends_decision"), "{err}");
    for state in ["approved", "refused"] {
        let err = brut(&pool, m.c, m.k, "outil", state, None, None, false, 60, None)
            .await
            .unwrap_err();
        assert!(err.contains("chk_mcp_sends_decision"), "{state} : {err}");
    }
}

#[sqlx::test(migrations = "./test-schema")]
async fn refuse_avec_contenu_ou_resume_refuse(pool: MySqlPool) {
    let m = montage(&pool, "A").await;
    for (summary, content) in [(None, Some("c")), (Some("r"), None), (Some("r"), Some("c"))] {
        let err = brut(
            &pool,
            m.c,
            m.k,
            "outil",
            "refused",
            summary,
            content,
            false,
            60,
            Some(m.u),
        )
        .await
        .unwrap_err();
        assert!(
            err.contains("chk_mcp_sends_refused_empty"),
            "{summary:?}/{content:?} : {err}"
        );
    }
}

#[sqlx::test(migrations = "./test-schema")]
async fn modified_vrai_hors_validation_refuse(pool: MySqlPool) {
    let m = montage(&pool, "A").await;
    let err = brut(
        &pool,
        m.c,
        m.k,
        "outil",
        "pending",
        None,
        Some("c"),
        true,
        60,
        None,
    )
    .await
    .unwrap_err();
    assert!(err.contains("chk_mcp_sends_modified"), "{err}");
    let err = brut(
        &pool,
        m.c,
        m.k,
        "outil",
        "refused",
        None,
        None,
        true,
        60,
        Some(m.u),
    )
    .await
    .unwrap_err();
    assert!(err.contains("chk_mcp_sends_modified"), "{err}");
    // Témoin : un validé modifié passe.
    brut(
        &pool,
        m.c,
        m.k,
        "outil",
        "approved",
        None,
        Some("c"),
        true,
        60,
        Some(m.u),
    )
    .await
    .expect("validé modifié");
}

#[sqlx::test(migrations = "./test-schema")]
async fn nom_d_outil_vide_refuse(pool: MySqlPool) {
    let m = montage(&pool, "A").await;
    let err = brut(
        &pool,
        m.c,
        m.k,
        "",
        "pending",
        None,
        Some("c"),
        false,
        60,
        None,
    )
    .await
    .unwrap_err();
    assert!(err.contains("chk_mcp_sends_tool"), "{err}");
}

#[sqlx::test(migrations = "./test-schema")]
async fn cle_inexistante_refusee_et_cle_referencee_non_supprimable(pool: MySqlPool) {
    let m = montage(&pool, "A").await;
    let err = brut(
        &pool,
        m.c,
        m.k + 1000,
        "outil",
        "pending",
        None,
        Some("c"),
        false,
        60,
        None,
    )
    .await
    .unwrap_err();
    assert!(err.contains("fk_mcp_sends_key"), "{err}");
    en_attente(&pool, &m, "c", 60).await;
    let err = sqlx::query("DELETE FROM mcp_keys WHERE id = ?")
        .bind(m.k)
        .execute(&pool)
        .await
        .expect_err("RESTRICT");
    assert!(err.to_string().contains("fk_mcp_sends_key"), "{err}");
}

// ---------------------------------------------------------------------------
// AC-A3 — l'état dérivé
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn etat_derive_par_l_horloge_de_la_base(pool: MySqlPool) {
    let m = montage(&pool, "A").await;
    let attente = en_attente(&pool, &m, "a", 3600).await;
    let valide_ = valide(&pool, &m, "v", 3600).await;
    let refuse_ = refuse(&pool, &m).await;
    let lire = |id| mcp_sends::lire_pour_cle(&pool, m.c, m.k, id);
    assert_eq!(
        lire(attente).await.unwrap().unwrap().etat,
        EtatEnvoi::EnAttente
    );
    assert_eq!(
        lire(valide_).await.unwrap().unwrap().etat,
        EtatEnvoi::Valide
    );
    assert_eq!(
        lire(refuse_).await.unwrap().unwrap().etat,
        EtatEnvoi::Refuse
    );

    for id in [attente, valide_, refuse_] {
        echoir(&pool, id).await;
    }
    assert_eq!(lire(attente).await.unwrap().unwrap().etat, EtatEnvoi::Echu);
    assert_eq!(lire(valide_).await.unwrap().unwrap().etat, EtatEnvoi::Echu);
    assert_eq!(
        lire(refuse_).await.unwrap().unwrap().etat,
        EtatEnvoi::Refuse,
        "un refusé reste refusé"
    );
    // Rien n'est écrit : l'état stocké est inchangé.
    assert_eq!(ligne(&pool, attente).await.0, "pending");
    assert_eq!(ligne(&pool, valide_).await.0, "approved");
}

// ---------------------------------------------------------------------------
// AC-A5 — le dépôt
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn deposer_refuse_le_vingt_et_unieme_envoi_vivant(pool: MySqlPool) {
    let m = montage(&pool, "A").await;
    for i in 0..20 {
        deposer(&pool, &envoi(&m, &format!("c{i}")), 20)
            .await
            .unwrap_or_else(|e| panic!("dépôt {i} : {e}"));
    }
    assert!(matches!(
        deposer(&pool, &envoi(&m, "c20"), 20).await,
        Err(ErreurDepot::TropNombreux)
    ));
    assert_eq!(compte(&pool).await, 20, "rien n'est écrit");
    // Une autre clé n'est pas comptée avec celle-ci.
    let k2 = key(&pool, m.c, m.u, "cle-2").await;
    let mut e = envoi(&m, "autre clé");
    e.mcp_key_id = k2;
    deposer(&pool, &e, 20).await.expect("autre clé");
}

#[sqlx::test(migrations = "./test-schema")]
async fn un_echu_un_valide_ou_un_refuse_ne_compte_pas(pool: MySqlPool) {
    let m = montage(&pool, "A").await;
    let echu = en_attente(&pool, &m, "échu", 3600).await;
    echoir(&pool, echu).await;
    valide(&pool, &m, "v", 3600).await;
    refuse(&pool, &m).await;
    let id = deposer(&pool, &envoi(&m, "c"), 1)
        .await
        .expect("aucun envoi en attente vivant : le plafond 1 admet");
    assert!(matches!(
        deposer(&pool, &envoi(&m, "c"), 1).await,
        Err(ErreurDepot::TropNombreux)
    ));
    let (state, content, summary, modified, version) = ligne(&pool, id).await;
    assert_eq!(state, "pending");
    assert_eq!(content.as_deref(), Some("c"));
    assert_eq!(summary.as_deref(), Some("résumé"));
    assert!(!modified);
    assert_eq!(version, 1);
}

#[sqlx::test(migrations = "./test-schema")]
async fn echeance_vaut_creation_plus_duree_lue_en_base(pool: MySqlPool) {
    let m = montage(&pool, "A").await;
    let id = deposer(&pool, &envoi(&m, "c"), 20).await.unwrap();
    let ecart_ms: i64 = sqlx::query_scalar(
        "SELECT TIMESTAMPDIFF(MICROSECOND, created_at, expires_at) DIV 1000 \
         FROM mcp_sends WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(ecart_ms, i64::from(JOUR) * 1000, "24 h à la milliseconde");
    let vivant: i64 = sqlx::query_scalar(
        "SELECT CAST(expires_at > NOW(3) + INTERVAL 86390 SECOND AS SIGNED) \
         FROM mcp_sends WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(vivant, 1, "horloge de la base");
}

// ---------------------------------------------------------------------------
// AC-A6 — la décision
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn decider_refuse_introuvable_y_compris_d_une_autre_societe(pool: MySqlPool) {
    let a = montage(&pool, "A").await;
    let b = montage(&pool, "B").await;
    let id_b = en_attente(&pool, &b, "c", 3600).await;
    for id in [id_b, id_b + 1000] {
        assert!(matches!(
            decider(&pool, &a, id, 1, Decision::Refuser).await,
            Err(ErreurDecision::Introuvable)
        ));
    }
    assert_eq!(ligne(&pool, id_b).await.0, "pending", "rien n'est écrit");
}

#[sqlx::test(migrations = "./test-schema")]
async fn decider_refuse_un_envoi_deja_decide(pool: MySqlPool) {
    let m = montage(&pool, "A").await;
    let v = valide(&pool, &m, "c", 3600).await;
    let r = refuse(&pool, &m).await;
    for id in [v, r] {
        assert!(matches!(
            decider(
                &pool,
                &m,
                id,
                1,
                Decision::Valider {
                    contenu: Some("autre".into())
                }
            )
            .await,
            Err(ErreurDecision::DejaDecidee)
        ));
    }
    assert_eq!(ligne(&pool, v).await.1.as_deref(), Some("c"), "inchangé");
}

#[sqlx::test(migrations = "./test-schema")]
async fn decider_refuse_un_envoi_echu(pool: MySqlPool) {
    let m = montage(&pool, "A").await;
    let id = en_attente(&pool, &m, "c", 3600).await;
    echoir(&pool, id).await;
    assert!(matches!(
        decider(&pool, &m, id, 1, Decision::Valider { contenu: None }).await,
        Err(ErreurDecision::Echue)
    ));
    assert_eq!(ligne(&pool, id).await.0, "pending", "rien n'est écrit");
}

#[sqlx::test(migrations = "./test-schema")]
async fn decider_refuse_une_version_perimee(pool: MySqlPool) {
    let m = montage(&pool, "A").await;
    let id = en_attente(&pool, &m, "c", 3600).await;
    for v in [0, 2] {
        assert!(matches!(
            decider(&pool, &m, id, v, Decision::Refuser).await,
            Err(ErreurDecision::VersionPerimee)
        ));
    }
    assert_eq!(ligne(&pool, id).await.0, "pending", "rien n'est écrit");
}

#[sqlx::test(migrations = "./test-schema")]
async fn valider_tel_quel_garde_le_contenu(pool: MySqlPool) {
    let m = montage(&pool, "A").await;
    let id = en_attente(&pool, &m, "C1", 3600).await;
    let d = decider(&pool, &m, id, 1, Decision::Valider { contenu: None })
        .await
        .unwrap();
    assert_eq!(d.id, id);
    assert_eq!(d.mcp_key_id, m.k);
    assert_eq!(d.tool_name, "outil");
    assert_eq!(d.etat, EtatEnvoi::Valide);
    assert!(!d.modified);
    assert_eq!(d.version, 2);
    let (state, content, summary, modified, version) = ligne(&pool, id).await;
    assert_eq!(
        (
            state.as_str(),
            content.as_deref(),
            summary.as_deref(),
            modified,
            version
        ),
        ("approved", Some("C1"), Some("résumé"), false, 2)
    );
    let (decided_by, decided): (Option<i64>, i64) = sqlx::query_as(
        "SELECT decided_by_user_id, CAST(decided_at IS NOT NULL AS SIGNED) \
         FROM mcp_sends WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((decided_by, decided), (Some(m.u), 1));
}

#[sqlx::test(migrations = "./test-schema")]
async fn une_valeur_identique_soumise_n_est_pas_modifiee(pool: MySqlPool) {
    let m = montage(&pool, "A").await;
    let id = en_attente(&pool, &m, "Élan\t ", 3600).await;
    let d = decider(
        &pool,
        &m,
        id,
        1,
        Decision::Valider {
            contenu: Some("Élan\t ".into()),
        },
    )
    .await
    .unwrap();
    assert!(!d.modified);
    assert!(!ligne(&pool, id).await.3);
}

#[sqlx::test(migrations = "./test-schema")]
async fn casse_et_blanc_final_comptent_comme_modification(pool: MySqlPool) {
    let m = montage(&pool, "A").await;
    for (stocke, recu) in [("Élan", "élan"), ("abc", "abc "), ("abc", "abc\t")] {
        let id = en_attente(&pool, &m, stocke, 3600).await;
        let d = decider(
            &pool,
            &m,
            id,
            1,
            Decision::Valider {
                contenu: Some(recu.into()),
            },
        )
        .await
        .unwrap();
        assert!(d.modified, "{stocke:?} → {recu:?}");
        let (_, content, _, modified, _) = ligne(&pool, id).await;
        assert!(modified);
        assert_eq!(
            content.as_deref().map(str::as_bytes),
            Some(recu.as_bytes()),
            "stocké tel quel"
        );
    }
}

#[sqlx::test(migrations = "./test-schema")]
async fn refuser_efface_contenu_et_resume(pool: MySqlPool) {
    let m = montage(&pool, "A").await;
    let id = en_attente(&pool, &m, "C1", 3600).await;
    let d = decider(&pool, &m, id, 1, Decision::Refuser).await.unwrap();
    assert_eq!(d.etat, EtatEnvoi::Refuse);
    assert!(!d.modified);
    assert_eq!(d.version, 2);
    let (state, content, summary, modified, version) = ligne(&pool, id).await;
    assert_eq!(
        (state.as_str(), content, summary, modified, version),
        ("refused", None, None, false, 2)
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn decision_modifiee_ecrit_le_texte_recu(pool: MySqlPool) {
    let m = montage(&pool, "A").await;
    let c1 = "C1 — texte déposé";
    let c2 = contenu_piege();
    let id = en_attente(&pool, &m, c1, 3600).await;
    let d = decider(
        &pool,
        &m,
        id,
        1,
        Decision::Valider {
            contenu: Some(c2.clone()),
        },
    )
    .await
    .unwrap();
    assert!(d.modified);
    let detail = mcp_sends::trouver(&pool, m.c, id).await.unwrap().unwrap();
    assert_eq!(
        detail.content.as_deref().map(str::as_bytes),
        Some(c2.as_bytes())
    );
    assert_eq!(detail.content_bytes, Some(65_536));
    let lu = mcp_sends::lire_pour_cle(&pool, m.c, m.k, id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        lu.contenu.as_deref().map(str::as_bytes),
        Some(c2.as_bytes())
    );
    assert!(lu.modified);
}

// ---------------------------------------------------------------------------
// AC-A7 — la relecture pour une clé
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn lire_pour_cle_rend_le_meme_none_inexistant_autre_cle_autre_societe(pool: MySqlPool) {
    let a = montage(&pool, "A").await;
    let b = montage(&pool, "B").await;
    let k_a2 = key(&pool, a.c, a.u, "cle-A2").await;
    let id = valide(&pool, &a, "c", 3600).await;
    let id_b = valide(&pool, &b, "c", 3600).await;
    // Témoin : la bonne clé de la bonne société lit.
    assert!(
        mcp_sends::lire_pour_cle(&pool, a.c, a.k, id)
            .await
            .unwrap()
            .is_some()
    );
    assert_eq!(
        mcp_sends::lire_pour_cle(&pool, a.c, a.k, id + 1000)
            .await
            .unwrap(),
        None,
        "inexistant"
    );
    assert_eq!(
        mcp_sends::lire_pour_cle(&pool, a.c, k_a2, id)
            .await
            .unwrap(),
        None,
        "autre clé de la même société"
    );
    assert_eq!(
        mcp_sends::lire_pour_cle(&pool, a.c, a.k, id_b)
            .await
            .unwrap(),
        None,
        "envoi d'une autre société"
    );
}

/// Une ligne **incohérente** — la clé de l'appel, la société d'autrui —,
/// insérée par le pool du test : lue avec la société de la clé, `None`. Sans ce
/// cas, lever le filtre de société serait une mutation équivalente (MA12).
#[sqlx::test(migrations = "./test-schema")]
async fn lire_pour_cle_ecarte_une_ligne_incoherente(pool: MySqlPool) {
    let a = montage(&pool, "A").await;
    let b = montage(&pool, "B").await;
    let id = brut(
        &pool,
        b.c,
        a.k,
        "outil",
        "approved",
        None,
        Some("c"),
        false,
        3600,
        Some(a.u),
    )
    .await
    .expect("montage : ligne incohérente");
    assert_eq!(
        mcp_sends::lire_pour_cle(&pool, a.c, a.k, id).await.unwrap(),
        None
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn lire_pour_cle_ne_rend_le_contenu_que_d_un_valide_vivant(pool: MySqlPool) {
    let m = montage(&pool, "A").await;
    let attente = en_attente(&pool, &m, "a", 3600).await;
    let refuse_ = refuse(&pool, &m).await;
    let echu = valide(&pool, &m, "e", 3600).await;
    echoir(&pool, echu).await;
    for (id, etat) in [
        (attente, EtatEnvoi::EnAttente),
        (refuse_, EtatEnvoi::Refuse),
        (echu, EtatEnvoi::Echu),
    ] {
        assert_eq!(
            mcp_sends::lire_pour_cle(&pool, m.c, m.k, id).await.unwrap(),
            Some(EnvoiPourCle {
                id,
                etat,
                modified: false,
                contenu: None
            })
        );
    }
    let c = contenu_piege();
    let id = valide(&pool, &m, &c, 3600).await;
    let lu = mcp_sends::lire_pour_cle(&pool, m.c, m.k, id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(lu.etat, EtatEnvoi::Valide);
    assert_eq!(lu.contenu.as_deref().map(str::as_bytes), Some(c.as_bytes()));
}

// ---------------------------------------------------------------------------
// AC-A8 — la liste et le détail
// ---------------------------------------------------------------------------

/// Pose `created_at` d'une ligne, en secondes relatives à `NOW(3)`.
async fn cree_il_y_a(pool: &MySqlPool, id: i64, secondes: i64) {
    sqlx::query("UPDATE mcp_sends SET created_at = NOW(3) - INTERVAL ? SECOND WHERE id = ?")
        .bind(secondes)
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
}

#[sqlx::test(migrations = "./test-schema")]
async fn lister_ordonne_les_attentes_vivantes_puis_le_reste(pool: MySqlPool) {
    let m = montage(&pool, "A").await;
    let b = montage(&pool, "B").await;
    let tard = en_attente(&pool, &m, "a", 7200).await;
    let tot = en_attente(&pool, &m, "a", 600).await;
    let valide_ = valide(&pool, &m, "v", 3600).await;
    let refuse_ = refuse(&pool, &m).await;
    let echu = en_attente(&pool, &m, "e", 3600).await;
    echoir(&pool, echu).await;
    en_attente(&pool, &b, "autre société", 60).await;
    // Créations : l'échu le plus récent, puis le refusé, puis le validé ; les
    // attentes, plus anciennes, passent quand même devant.
    for (id, age) in [
        (tard, 500),
        (tot, 400),
        (valide_, 300),
        (refuse_, 200),
        (echu, 100),
    ] {
        cree_il_y_a(&pool, id, age).await;
    }
    let liste = mcp_sends::lister(&pool, m.c).await.unwrap();
    let ids: Vec<i64> = liste.iter().map(|e| e.id).collect();
    assert_eq!(ids, vec![tot, tard, echu, refuse_, valide_]);
    let etats: Vec<EtatEnvoi> = liste.iter().map(|e| e.etat).collect();
    assert_eq!(
        etats,
        vec![
            EtatEnvoi::EnAttente,
            EtatEnvoi::EnAttente,
            EtatEnvoi::Echu,
            EtatEnvoi::Refuse,
            EtatEnvoi::Valide
        ]
    );
    let v = &liste[4];
    assert_eq!(v.mcp_key_id, m.k);
    assert_eq!(v.mcp_key_name, "cle-A");
    assert!(!v.mcp_key_revoked);
    assert_eq!(v.tool_name, "outil");
    assert_eq!(v.decided_by.as_deref(), Some("admin-A"));
    assert!(v.decided_at.is_some());
    assert_eq!(liste[0].decided_by, None);
    // La clé révoquée se dit telle.
    sqlx::query("UPDATE mcp_keys SET revoked_at = NOW(3) WHERE id = ?")
        .bind(m.k)
        .execute(&pool)
        .await
        .unwrap();
    assert!(
        mcp_sends::lister(&pool, m.c)
            .await
            .unwrap()
            .iter()
            .all(|e| e.mcp_key_revoked)
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn lister_rend_au_plus_deux_cents_lignes(pool: MySqlPool) {
    let m = montage(&pool, "A").await;
    sqlx::query(
        "INSERT INTO mcp_sends (company_id, mcp_key_id, tool_name, state, content, modified, \
         expires_at) \
         SELECT ?, ?, 'outil', 'pending', 'c', FALSE, NOW(3) + INTERVAL seq SECOND \
         FROM seq_1_to_201",
    )
    .bind(m.c)
    .bind(m.k)
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(compte(&pool).await, 201, "montage");
    let liste = mcp_sends::lister(&pool, m.c).await.unwrap();
    assert_eq!(liste.len(), LISTE_MAX as usize);
    assert_eq!(LISTE_MAX, 200);
    // Les 200 plus proches échéances, croissantes.
    assert!(liste.windows(2).all(|w| w[0].expires_at <= w[1].expires_at));
}

#[sqlx::test(migrations = "./test-schema")]
async fn trouver_porte_resume_contenu_taille_et_version(pool: MySqlPool) {
    let m = montage(&pool, "A").await;
    let b = montage(&pool, "B").await;
    let id = en_attente(&pool, &m, "Élan ⚠", 3600).await;
    let d = mcp_sends::trouver(&pool, m.c, id).await.unwrap().unwrap();
    assert_eq!(d.resume.id, id);
    assert_eq!(d.resume.etat, EtatEnvoi::EnAttente);
    assert_eq!(d.summary.as_deref(), Some("résumé"));
    assert_eq!(d.content.as_deref(), Some("Élan ⚠"));
    assert_eq!(d.content_bytes, Some("Élan ⚠".len() as u64));
    assert_eq!(d.content_bytes, Some(9), "É et ⚠ comptent 2 et 3 octets");
    assert_eq!(d.version, 1);

    // Une autre société : ni au détail ni à la liste.
    assert_eq!(mcp_sends::trouver(&pool, b.c, id).await.unwrap(), None);
    assert!(mcp_sends::lister(&pool, b.c).await.unwrap().is_empty());
    assert_eq!(
        mcp_sends::trouver(&pool, m.c, id + 1000).await.unwrap(),
        None
    );

    // Refusé, puis échu non encore purgé : ni résumé ni contenu.
    let r = refuse(&pool, &m).await;
    let e = valide(&pool, &m, "encore stocké", 3600).await;
    echoir(&pool, e).await;
    for (id, etat) in [(r, EtatEnvoi::Refuse), (e, EtatEnvoi::Echu)] {
        let d = mcp_sends::trouver(&pool, m.c, id).await.unwrap().unwrap();
        assert_eq!(d.resume.etat, etat);
        assert_eq!((d.summary, d.content, d.content_bytes), (None, None, None));
    }
    assert_eq!(
        ligne(&pool, e).await.1.as_deref(),
        Some("encore stocké"),
        "montage : le contenu échu est encore en base"
    );
}

// ---------------------------------------------------------------------------
// AC-A9 — la purge
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn purger_echus_ne_touche_que_les_echus_de_la_societe(pool: MySqlPool) {
    let a = montage(&pool, "A").await;
    let b = montage(&pool, "B").await;
    let vivant = en_attente(&pool, &a, "vivant", 3600).await;
    let echu_valide = valide(&pool, &a, "échu validé", 3600).await;
    let echu_attente = en_attente(&pool, &a, "échu en attente", 3600).await;
    let echu_b = en_attente(&pool, &b, "échu d'autrui", 3600).await;
    for id in [echu_valide, echu_attente, echu_b] {
        echoir(&pool, id).await;
    }
    assert_eq!(mcp_sends::purger_echus(&pool, a.c).await.unwrap(), 2);
    for id in [echu_valide, echu_attente] {
        let (_, content, summary, _, _) = ligne(&pool, id).await;
        assert_eq!((content, summary), (None, None), "purgé");
    }
    let (_, content, summary, _, _) = ligne(&pool, vivant).await;
    assert_eq!(
        (content.as_deref(), summary.as_deref()),
        (Some("vivant"), Some("résumé"))
    );
    let (_, content, _, _, _) = ligne(&pool, echu_b).await;
    assert_eq!(content.as_deref(), Some("échu d'autrui"), "autre société");
    // Rejouée, elle ne touche plus rien.
    assert_eq!(mcp_sends::purger_echus(&pool, a.c).await.unwrap(), 0);
}
