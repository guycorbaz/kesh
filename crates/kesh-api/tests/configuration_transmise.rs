//! Garde de la configuration transmise — Stories 15-11a et 15-11b (#550).
//!
//! # Le défaut qu'elle ferme
//!
//! Les compose distribués (`docker-compose.yml`, `docker-compose.prod.yml`)
//! n'ont pas d'`env_file` : le service `kesh-api` ne reçoit **que** les
//! variables listées une à une sous `environment:`. Docker Compose lit bien
//! `.env`, mais seulement pour **interpoler** les `${…}` du fichier compose.
//! Une variable posée dans `.env` sans ligne sous `environment:` n'atteint pas
//! Kesh, et rien ne le signale. Jusqu'à la 0.13.0, seize variables lues par le
//! code étaient dans ce cas (SMTP, sortie de la démonstration, langue…).
//!
//! # Ce qu'elle établit
//!
//! - **(L) Lectures** (15-11b) — le test **lit le code** de production
//!   (`crates/*/src`) : chaque occurrence d'un jeton par lequel toute lecture
//!   de l'environnement doit passer ([`JETONS_SURVEILLES`]) figure, à son
//!   emplacement et sous sa forme, dans la liste fermée
//!   [`EMPLACEMENTS_AUTORISES`]. L'**ensemble lu** (les noms que le code lit)
//!   en est **calculé** ; il remplace la liste écrite à la main de la 15-11a.
//! - **(T) Transmission** — chaque variable de l'ensemble lu est transmise par
//!   les deux compose, sauf les [`EXCEPTIONS`] (dont la raison est
//!   contrôlée) ; toute clé transmise est lue ; pas d'`env_file` ; structure
//!   gardée (`image:` de `docker-compose.yml`, montages des deux compose).
//! - **(V) Valeurs** — chaque valeur transmise a une forme admise ; listes
//!   fermées [`VIDE_SIGNIFIANT`], [`SANS_DEFAUT`] et [`AJOUTS`].
//! - **(E) `.env.example`** — chaque ligne d'affectation nomme une variable
//!   connue, chaque variable lue y a sa ligne (sauf `KESH_TEST_MODE`).
//! - **(F) Fantômes** — tout jeton `KESH_…` du gabarit, des compose, des
//!   catalogues i18n, du manuel français **et des littéraux de chaîne du code
//!   de production** (doc-comments, macros, attributs compris) nomme une
//!   variable connue.
//! - **(M) Service MariaDB** (15-13a, #551) — le service `mariadb` de
//!   `docker-compose.yml` ne publie aucun port (pas de clé `ports:`) et ses
//!   deux mots de passe sont de la forme `${NOM:?…}` (obligatoires, vide
//!   refusé) ; la `DATABASE_URL` composée de `kesh-api` exige
//!   `${MARIADB_PASSWORD:?` (famille (V), [`VALEURS_COMPOSEES`]) ; aucun
//!   fichier distribué ne porte `kesh_dev`, et les mots de passe MariaDB de
//!   `.env.example` sont des lignes commentées.
//! - **(S) Auto-test** des extracteurs sur des sources synthétiques.
//! - **(D) Documentation d'exploitation** (15-14b, #575, #554) — le manuel
//!   d'administration ne cite que des services et des volumes que les compose
//!   déclarent (G14, G15) ; ses sections Synology sauvegardent la base par le
//!   dump d'une tâche planifiée et la restaurent par un autre compte, sans
//!   `DROP` possible sur une étape ratée (G16) ; le compose de développement
//!   démarre sans `.env` (G17).
//!
//! # Ce qu'elle n'établit PAS (angles morts écrits)
//!
//! Les lectures internes aux dépendances (sans jeton dans le workspace :
//! `NO_COLOR` de `tracing-subscriber`, `TOKIO_WORKER_THREADS`, `TZ`…) et par
//! FFI ; les identifiants synthétisés par une macro procédurale ; les
//! littéraux d'octets ; `include!` et `#[path]` hors `crates/*/src` ; le
//! fichier d'un module hors ligne `#[cfg(test)] mod x;` (faux rouge possible).
//! Les fichiers hors `crates/*/src` — `build.rs`, `examples/`, `benches/` —
//! ne sont pas lus (aucun ne lit l'environnement au 2026-10-09 ; un futur
//! qui le ferait ne rougirait pas). Les identifiants bruts (`r#env`) sont
//! normalisés et relevés. (L) établit le **passage** par `env_nonempty`, non le
//! défaut qui s'ensuit : pour `KESH_STATIC_DIR` et `KESH_LOCALES_DIR` (lus
//! par `main` après la connexion à la base), le défaut appliqué à une valeur
//! vide n'a pas de test de comportement ; celui de `RUST_LOG` en a un
//! (`rust_log_vide_vaut_info`).
//! `TMPDIR` (`std::env::temp_dir()`) est inventorié mais non compté dans
//! l'ensemble lu. `docker-compose.dev.yml` (pile de développement, non
//! distribuée) n'est contraint ni par (T) ni par (M) : ses mots de passe de
//! développement et son port en loopback y restent. Depuis la 15-14b, G17 y
//! contrôle une seule chose — que les défauts de `KESH_ADMIN_PASSWORD` et de
//! `KESH_JWT_SECRET` passent la configuration du binaire (démarrage sans
//! `.env`, #554). (M) ne voit pas
//! un mot de passe **root** posé dans `.env` sur une valeur gabarit `<…>`
//! (issue #578).
//!
//! L'analyseur YAML (`yaml-rust2`) ne connaît pas les clés de fusion
//! (`<<: *ancre`) : une forme non reconnue fait **rougir**, jamais passer. La
//! validité au sens de Compose (clé inconnue, clé en double) est contrôlée par
//! l'étape `docker compose config -q` du job `docker-build` de la CI.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use regex::Regex;
use yaml_rust2::{Yaml, YamlLoader};

#[path = "common/binaire.rs"]
mod binaire;
#[path = "common/manuel.rs"]
mod manuel;

// ---------------------------------------------------------------------------
// Listes fermées
// ---------------------------------------------------------------------------

/// Pourquoi une variable lue n'est transmise par **aucun** compose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Exception {
    /// Fixée par l'image (`ENV` de l'étape finale du `Dockerfile`) : la
    /// transmettre écraserait le chemin de l'image par une valeur d'hôte.
    /// Contrôle : `ENV <nom>=` après le dernier `FROM`, et absente des compose.
    FixeeParImage,
    /// Expose `/api/v1/_test/*` : ne doit jamais être transmissible par un
    /// compose distribué. Contrôle : absente des deux compose.
    InterditeEnProduction,
}

/// Les **seules** variables lues que les compose ne transmettent pas (AC 3).
const EXCEPTIONS: &[(&str, Exception)] = &[
    ("KESH_LOCALES_DIR", Exception::FixeeParImage),
    ("KESH_STATIC_DIR", Exception::FixeeParImage),
    ("KESH_TEST_MODE", Exception::InterditeEnProduction),
];

/// Variables dont une ligne **vide** de `.env` a un sens distinct de
/// l'absence : elles portent la forme `${NOM-défaut}` (tiret **sans**
/// deux-points), qui n'applique le défaut qu'à une variable absente.
///
/// `KESH_LOG_FILE_PATH` : absente → journal fichier au chemin par défaut ;
/// ligne vide → journal fichier désactivé (`LogConfig::from_raw` filtre le
/// vide). Contrôle de la raison : le commentaire de `.env.example` qui précède
/// chaque ligne d'affectation le dit (« contrairement aux autres variables »)
/// et ne dit pas « ou absent ».
const VIDE_SIGNIFIANT: &[&str] = &["KESH_LOG_FILE_PATH"];

/// Variables sans défaut de déploiement : forme `${NOM:-}` et **aucune
/// autre** dans les deux compose.
///
/// `KESH_ADMIN_PASSWORD` est optionnelle : absente, l'administrateur se crée à
/// l'écran `/setup`. Un défaut (`changeme`) ferait refuser le démarrage ; la
/// forme `${NOM}` ferait avertir Compose à chaque `up` sur le flux recommandé
/// (choix C76, C79). Contrôle de la raison : la variable est lue, et chacune de
/// ses lignes d'affectation dans `.env.example` est commentée.
const SANS_DEFAUT: &[&str] = &["KESH_ADMIN_PASSWORD"];

/// Les deux compose distribués.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Compose {
    /// `docker-compose.yml` — installation autonome (MariaDB comprise).
    Y,
    /// `docker-compose.prod.yml` — installation Synology du manuel.
    P,
}

impl Compose {
    fn fichier(self) -> &'static str {
        match self {
            Compose::Y => "docker-compose.yml",
            Compose::P => "docker-compose.prod.yml",
        }
    }
}

/// Les **28** couples (variable, compose) ajoutés par la Story 15-11a (#550),
/// sous la forme `${NOM:-}` et aucune autre : le défaut reste celui du code
/// (DRY), jamais recopié dans le compose (choix C75).
///
/// Liste **historique**, non dérivée : une story future qui veut un défaut
/// de déploiement pour l'une de ces variables retire le couple d'ici, et
/// l'écrit.
const AJOUTS: &[(&str, Compose)] = &[
    // Transmises par aucun compose avant la 0.13.0 — dans les deux.
    ("KESH_SMTP_HOST", Compose::Y),
    ("KESH_SMTP_HOST", Compose::P),
    ("KESH_SMTP_PORT", Compose::Y),
    ("KESH_SMTP_PORT", Compose::P),
    ("KESH_SMTP_USER", Compose::Y),
    ("KESH_SMTP_USER", Compose::P),
    ("KESH_SMTP_PASSWORD", Compose::Y),
    ("KESH_SMTP_PASSWORD", Compose::P),
    ("KESH_SMTP_FROM", Compose::Y),
    ("KESH_SMTP_FROM", Compose::P),
    ("KESH_SMTP_TLS", Compose::Y),
    ("KESH_SMTP_TLS", Compose::P),
    ("KESH_PUBLIC_BASE_URL", Compose::Y),
    ("KESH_PUBLIC_BASE_URL", Compose::P),
    ("KESH_FEATURE_FORGOT_PASSWORD", Compose::Y),
    ("KESH_FEATURE_FORGOT_PASSWORD", Compose::P),
    ("KESH_PRODUCTION_RESET", Compose::Y),
    ("KESH_PRODUCTION_RESET", Compose::P),
    ("KESH_ADMIN_BACKUP_DIR", Compose::Y),
    ("KESH_ADMIN_BACKUP_DIR", Compose::P),
    ("KESH_ADMIN_EXPORT_INMEM_MB", Compose::Y),
    ("KESH_ADMIN_EXPORT_INMEM_MB", Compose::P),
    ("KESH_ADMIN_IMPORT_MAX_MB", Compose::Y),
    ("KESH_ADMIN_IMPORT_MAX_MB", Compose::P),
    // Transmises par un seul compose avant la 0.13.0.
    ("KESH_COOKIE_SECURE", Compose::P),
    ("KESH_LANG", Compose::Y),
    ("KESH_PASSWORD_MIN_LENGTH", Compose::Y),
    ("KESH_BANK_IMPORT_MAX_MB", Compose::Y),
];

/// Valeurs **composées** admises (exception nommée de (V)) : la valeur n'est
/// pas une interpolation de la clé elle-même. Contrôle de la raison : la
/// valeur contient l'interpolation annoncée.
const VALEURS_COMPOSEES: &[(&str, Compose, &str)] = &[
    // `docker-compose.yml` compose l'URL depuis les `MARIADB_*` de son
    // service MariaDB : la ligne `DATABASE_URL` de `.env` y est ignorée. Le
    // mot de passe y est OBLIGATOIRE et non vide (Story 15-13a, #551) : aucun
    // défaut publié ne doit pouvoir s'y glisser.
    ("DATABASE_URL", Compose::Y, "${MARIADB_PASSWORD:?"),
];

/// Variables de `.env.example` que le code ne lit pas, utilisées par les
/// **montages** de `docker-compose.yml` (seul compose qui les honore :
/// `docker-compose.prod.yml` a des montages fixes, issue #558, choix C83).
/// Contrôle : `${NOM` apparaît dans un `volumes:` de `kesh-api` de
/// `docker-compose.yml`.
const HOTE: &[&str] = &[
    "KESH_DOCUMENTS_HOST_DIR",
    "KESH_INBOX_HOST_DIR",
    "KESH_LOG_HOST_DIR",
];

/// Variables de `.env.example` que le code ne lit pas, utilisées par le
/// service MariaDB de `docker-compose.yml` (et sa `DATABASE_URL`). Contrôle :
/// `${NOM` apparaît dans `docker-compose.yml`.
const MARIADB: &[&str] = &[
    "MARIADB_DATABASE",
    "MARIADB_PASSWORD",
    "MARIADB_ROOT_PASSWORD",
    "MARIADB_USER",
];

/// Les montages de `kesh-api` : (cible dans le conteneur, source exigée dans
/// `docker-compose.yml` — préfixe si elle commence par `${`, exacte sinon —,
/// source exigée dans `docker-compose.prod.yml` — exacte). Voir
/// [`source_conforme`]. Story 15-13b (#552) : quatrième entrée, la sauvegarde
/// pré-import, montage fixe dans les deux compose (cible
/// `kesh_api::config::DEFAULT_ADMIN_BACKUP_DIR`, contrôlé par (T)).
const MONTAGES: &[(&str, &str, &str)] = &[
    ("/var/log/kesh", "${KESH_LOG_HOST_DIR:-", "./log"),
    ("/data/inbox", "${KESH_INBOX_HOST_DIR:-", "./inbox"),
    (
        "/data/documents",
        "${KESH_DOCUMENTS_HOST_DIR:-",
        "./documents",
    ),
    ("/data/backup", "./backup", "./backup"),
];

/// Story 15-13b (AC 8 e) — sources de montage admises dans `.gitignore` sous
/// une forme **non ancrée** : `log` seul (ligne `log/` historique, aucun
/// dossier `log` versionné dans le dépôt). Toute autre source exige `/x/`.
const TOLERANCES_NON_ANCREES: &[&str] = &["log"];

/// Le marqueur que le commentaire d'une variable de [`VIDE_SIGNIFIANT`] doit
/// porter dans `.env.example`.
const MARQUEUR_VIDE_SIGNIFIANT: &str = "contrairement aux autres variables";

// ---------------------------------------------------------------------------
// Lecture des fichiers du dépôt
// ---------------------------------------------------------------------------

fn racine() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn lire(relatif: &str) -> String {
    let chemin = racine().join(relatif);
    std::fs::read_to_string(&chemin)
        .unwrap_or_else(|e| panic!("lecture de {} impossible : {e}", chemin.display()))
}

/// L'ensemble lu, **calculé** depuis le code par (L).
fn lues() -> BTreeSet<&'static str> {
    analyse_depot().noms.iter().map(String::as_str).collect()
}

fn exception(nom: &str) -> Option<Exception> {
    EXCEPTIONS.iter().find(|(n, _)| *n == nom).map(|(_, e)| *e)
}

fn echouer_si(erreurs: Vec<String>, titre: &str) {
    if !erreurs.is_empty() {
        panic!(
            "{titre} — {} écart(s) :\n  - {}",
            erreurs.len(),
            erreurs.join("\n  - ")
        );
    }
}

// ---------------------------------------------------------------------------
// Extraction du service `kesh-api`
// ---------------------------------------------------------------------------

/// Valeur d'une entrée de `environment:`.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ValeurEnv {
    /// Clé sans valeur (`NOM:` en dictionnaire, `- NOM` en liste).
    SansValeur,
    /// Scalaire (chaîne, entier, booléen ou réel YAML), rendu en texte.
    Scalaire(String),
    /// Toute autre forme (liste, dictionnaire, alias…).
    NonReconnue(String),
}

/// Un service d'un compose (`kesh-api`, ou `mariadb` depuis la Story 15-13a),
/// tel que ce test le lit.
#[derive(Debug, Default)]
struct Service {
    /// Clé `ports:` présente (quelle qu'en soit la forme). Story 15-13a.
    ports: bool,
    /// Entrées de `environment:`, dans l'ordre.
    environnement: Vec<(String, ValeurEnv)>,
    /// `env_file` présent.
    env_file: bool,
    /// `image:`, si présente et scalaire.
    image: Option<String>,
    /// Entrées de `volumes:`, brutes.
    volumes: Vec<Yaml>,
}

impl Service {
    fn cles(&self) -> BTreeSet<&str> {
        self.environnement.iter().map(|(k, _)| k.as_str()).collect()
    }

    fn valeur(&self, cle: &str) -> Option<&ValeurEnv> {
        self.environnement
            .iter()
            .find(|(k, _)| k == cle)
            .map(|(_, v)| v)
    }
}

fn scalaire(y: &Yaml) -> Option<String> {
    match y {
        Yaml::String(s) | Yaml::Real(s) => Some(s.clone()),
        Yaml::Integer(i) => Some(i.to_string()),
        Yaml::Boolean(b) => Some(b.to_string()),
        _ => None,
    }
}

/// Analyse un compose et rend son service `nom`, ou les raisons pour
/// lesquelles il ne se lit pas (forme non reconnue : rouge, jamais ignorée).
/// Généralisée de `service_kesh_api` à la Story 15-13a (service `mariadb`).
fn service(source: &str, nom: &str) -> Result<Service, Vec<String>> {
    let docs =
        YamlLoader::load_from_str(source).map_err(|e| vec![format!("YAML invalide : {e}")])?;
    let doc = docs
        .first()
        .ok_or_else(|| vec!["document YAML vide".to_string()])?;
    let svc = &doc["services"][nom];
    let Some(hash) = svc.as_hash() else {
        return Err(vec![format!("service `{nom}` introuvable")]);
    };
    let mut erreurs = Vec::new();
    let mut out = Service::default();
    for (k, v) in hash {
        match k.as_str() {
            Some("env_file") => out.env_file = true,
            Some("ports") => out.ports = true,
            Some("image") => out.image = scalaire(v),
            Some("volumes") => match v {
                Yaml::Array(a) => out.volumes = a.clone(),
                autre => erreurs.push(format!("`volumes:` de forme non reconnue : {autre:?}")),
            },
            Some("environment") => match v {
                Yaml::Hash(env) => {
                    for (ek, ev) in env {
                        let Some(nom) = scalaire(ek) else {
                            erreurs.push(format!("clé d'environnement non reconnue : {ek:?}"));
                            continue;
                        };
                        let valeur = match ev {
                            Yaml::Null => ValeurEnv::SansValeur,
                            autre => match scalaire(autre) {
                                Some(s) => ValeurEnv::Scalaire(s),
                                None => ValeurEnv::NonReconnue(format!("{autre:?}")),
                            },
                        };
                        out.environnement.push((nom, valeur));
                    }
                }
                Yaml::Array(liste) => {
                    for entree in liste {
                        let Some(s) = entree.as_str() else {
                            erreurs
                                .push(format!("entrée d'environnement non reconnue : {entree:?}"));
                            continue;
                        };
                        match s.split_once('=') {
                            Some((nom, val)) => out
                                .environnement
                                .push((nom.to_string(), ValeurEnv::Scalaire(val.to_string()))),
                            None => out
                                .environnement
                                .push((s.to_string(), ValeurEnv::SansValeur)),
                        }
                    }
                }
                autre => erreurs.push(format!("`environment:` de forme non reconnue : {autre:?}")),
            },
            _ => {}
        }
    }
    if erreurs.is_empty() {
        Ok(out)
    } else {
        Err(erreurs)
    }
}

/// Source d'un montage en syntaxe courte dont la cible est `cible` :
/// `<source>:<cible>` suivi de la fin de l'entrée ou de `:ro`/`:rw`. Jamais un
/// découpage sur le premier `:`, qui tomberait dans le `:-` de
/// `${KESH_LOG_HOST_DIR:-./log}`.
fn source_montage<'a>(entree: &'a str, cible: &str) -> Option<&'a str> {
    for suffixe in ["", ":ro", ":rw"] {
        let fin = format!(":{cible}{suffixe}");
        if let Some(source) = entree.strip_suffix(fin.as_str()) {
            return Some(source);
        }
    }
    None
}

/// Les sources des montages de [`MONTAGES`] dans un service, par cible.
/// Une entrée en forme longue, ou dont la cible n'est pas l'une d'elles,
/// rougit (« forme de montage non reconnue ») ; chaque cible doit être trouvée
/// exactement une fois.
fn sources_montages(svc: &Service) -> Result<BTreeMap<&'static str, String>, Vec<String>> {
    let mut erreurs = Vec::new();
    let mut trouvees: BTreeMap<&'static str, Vec<String>> = BTreeMap::new();
    for entree in &svc.volumes {
        let Some(s) = entree.as_str() else {
            erreurs.push(format!(
                "forme de montage non reconnue (forme longue ?) : {entree:?}"
            ));
            continue;
        };
        let mut reconnue = false;
        for (cible, _, _) in MONTAGES {
            if let Some(source) = source_montage(s, cible) {
                trouvees.entry(cible).or_default().push(source.to_string());
                reconnue = true;
                break;
            }
        }
        if !reconnue {
            erreurs.push(format!("forme de montage non reconnue : `{s}`"));
        }
    }
    let mut out = BTreeMap::new();
    for (cible, _, _) in MONTAGES {
        match trouvees.remove(cible).unwrap_or_default().as_slice() {
            [une] => {
                out.insert(*cible, une.clone());
            }
            autres => erreurs.push(format!(
                "montage vers `{cible}` trouvé {} fois (exactement une attendue)",
                autres.len()
            )),
        }
    }
    if erreurs.is_empty() {
        Ok(out)
    } else {
        Err(erreurs)
    }
}

// ---------------------------------------------------------------------------
// (T) Transmission
// ---------------------------------------------------------------------------

/// Vrai si l'étape finale du `Dockerfile` (après le dernier `FROM`) fixe
/// `nom` par `ENV nom=`.
fn dockerfile_fixe(dockerfile: &str, nom: &str) -> bool {
    let lignes: Vec<&str> = dockerfile.lines().collect();
    let debut = lignes
        .iter()
        .rposition(|l| l.trim_start().to_ascii_uppercase().starts_with("FROM "))
        .unwrap_or(0);
    let attendu = format!("ENV {nom}=");
    lignes[debut..]
        .iter()
        .any(|l| l.trim_start().starts_with(&attendu))
}

/// Story 15-13b (AC 10 c) — la source `source` d'un montage est-elle celle
/// qu'exige [`MONTAGES`] ? Pour `docker-compose.yml` : préfixe si la source
/// attendue commence par `${` (montage variable), **égalité** sinon (montage
/// fixe : `./backups` ne vaut pas `./backup`) ; pour
/// `docker-compose.prod.yml` : égalité.
fn source_conforme(compose: Compose, source: &str, attendue_y: &str, exacte_p: &str) -> bool {
    match compose {
        Compose::Y if attendue_y.starts_with("${") => source.starts_with(attendue_y),
        Compose::Y => source == attendue_y,
        Compose::P => source == exacte_p,
    }
}

/// Contrôle (T) sur les deux services : transmission, exceptions, clés
/// inconnues, `env_file`, structure.
fn controle_transmission(
    y: &Service,
    p: &Service,
    dockerfile: &str,
    lues: &BTreeSet<&str>,
) -> Vec<String> {
    let mut erreurs = Vec::new();
    for (compose, svc) in [(Compose::Y, y), (Compose::P, p)] {
        let f = compose.fichier();
        let cles = svc.cles();
        if svc.env_file {
            erreurs.push(format!(
                "{f} : `env_file` sur `kesh-api` — interdit (choix C71 : il transmettrait \
                 MARIADB_ROOT_PASSWORD et toute ligne décommentée par erreur, et viderait ce test de sens)"
            ));
        }
        for nom in lues {
            match exception(nom) {
                Some(_) => {
                    if cles.contains(nom) {
                        erreurs.push(format!(
                            "{f} : `{nom}` est une exception (AC 3) et ne doit pas être transmise"
                        ));
                    }
                }
                None => {
                    if !cles.contains(nom) {
                        erreurs.push(format!(
                            "{f} : `{nom}` est lue par Kesh mais non transmise — ajouter `{nom}: ${{{nom}:-}}` sous `environment:` de `kesh-api`"
                        ));
                    }
                }
            }
        }
        for cle in &cles {
            if !lues.contains(cle) {
                erreurs.push(format!(
                    "{f} : clé `{cle}` transmise mais lue nulle part (faute de frappe ?)"
                ));
            }
        }
    }
    for (nom, e) in EXCEPTIONS {
        if !lues.contains(nom) {
            erreurs.push(format!(
                "exception `{nom}` : la variable n'est pas dans l'ensemble lu (entrée inutilisée)"
            ));
        }
        if *e == Exception::FixeeParImage && !dockerfile_fixe(dockerfile, nom) {
            erreurs.push(format!(
                "exception `{nom}` (FixeeParImage) : le Dockerfile ne porte pas `ENV {nom}=` après le dernier FROM"
            ));
        }
    }
    // Structure (AC 4, AC 5).
    match &y.image {
        Some(i) if i.starts_with("gcorbaz/kesh:") => {}
        autre => erreurs.push(format!(
            "docker-compose.yml : `kesh-api` doit porter `image: gcorbaz/kesh:…` (AC 5 : installable seul), trouvé {autre:?}"
        )),
    }
    for (compose, svc) in [(Compose::Y, y), (Compose::P, p)] {
        let f = compose.fichier();
        match sources_montages(svc) {
            Err(es) => erreurs.extend(es.into_iter().map(|e| format!("{f} : {e}"))),
            Ok(sources) => {
                for (cible, prefixe_y, exacte_p) in MONTAGES {
                    let source = &sources[cible];
                    if source_conforme(compose, source, prefixe_y, exacte_p) {
                        continue;
                    }
                    match compose {
                        Compose::Y if prefixe_y.starts_with("${") => erreurs.push(format!(
                            "{f} : le montage de `{cible}` doit avoir pour source `{prefixe_y}…}}`, trouvé `{source}`"
                        )),
                        Compose::Y => erreurs.push(format!(
                            "{f} : le montage de `{cible}` doit être exactement `{prefixe_y}` (montage fixe), trouvé `{source}`"
                        )),
                        Compose::P => erreurs.push(format!(
                            "{f} : le montage de `{cible}` doit rester `{exacte_p}` (montage fixe), trouvé `{source}` — \
                             le rendre configurable exige la procédure de déplacement des données de l'issue #558 (choix C83, AC 4)"
                        )),
                    }
                }
            }
        }
    }
    // Story 15-13b (R4-1) — le défaut du code est la cible d'un montage : sans
    // ce lien, une constante changée renverrait la sauvegarde dans le système
    // de fichiers éphémère du conteneur sans qu'aucun contrôle ne rougisse.
    let defaut = kesh_api::config::DEFAULT_ADMIN_BACKUP_DIR;
    if !MONTAGES.iter().any(|(cible, _, _)| *cible == defaut) {
        erreurs.push(format!(
            "`DEFAULT_ADMIN_BACKUP_DIR` vaut `{defaut}`, qui n'est la cible d'aucun montage de `MONTAGES` : \
             la sauvegarde pré-import ne survivrait pas au conteneur (#552)"
        ));
    }
    erreurs
}

// ---------------------------------------------------------------------------
// (V) Valeurs
// ---------------------------------------------------------------------------

/// Une interpolation `${…}` qui occupe toute la valeur.
#[derive(Debug, PartialEq, Eq)]
enum Interpolation<'a> {
    /// `${NOM}`
    Simple(&'a str),
    /// `${NOM:-défaut}` (défaut éventuellement vide)
    DefautSiVide(&'a str, &'a str),
    /// `${NOM-défaut}`
    DefautSiAbsente(&'a str, &'a str),
    /// `${NOM:?message}` : refuse une variable absente **ou vide**.
    ObligatoireNonVide(&'a str),
    /// `${NOM?message}` : ne refuse qu'une variable **absente** — une ligne
    /// `NOM=` vide passe. Scindée de la précédente à la Story 15-13a.
    ObligatoireSiAbsente(&'a str),
}

fn interpolation(valeur: &str) -> Option<Interpolation<'_>> {
    let interieur = valeur.strip_prefix("${")?.strip_suffix('}')?;
    if interieur.contains("${") || interieur.contains('}') {
        return None;
    }
    let fin_nom = interieur
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(interieur.len());
    let (nom, reste) = interieur.split_at(fin_nom);
    if nom.is_empty() {
        return None;
    }
    if reste.is_empty() {
        Some(Interpolation::Simple(nom))
    } else if let Some(d) = reste.strip_prefix(":-") {
        Some(Interpolation::DefautSiVide(nom, d))
    } else if let Some(d) = reste.strip_prefix('-') {
        Some(Interpolation::DefautSiAbsente(nom, d))
    } else if reste.starts_with(":?") {
        Some(Interpolation::ObligatoireNonVide(nom))
    } else if reste.starts_with('?') {
        Some(Interpolation::ObligatoireSiAbsente(nom))
    } else {
        None
    }
}

/// Contrôle (V) d'une entrée `cle: valeur` du compose `compose`. Rend
/// l'écart, s'il y en a un.
fn controle_valeur(compose: Compose, cle: &str, valeur: &ValeurEnv) -> Option<String> {
    let f = compose.fichier();
    let forme_ajout = format!("${{{cle}:-}}");
    let v = match valeur {
        ValeurEnv::SansValeur => {
            return Some(format!(
                "{f} : `{cle}` est une clé sans valeur — interdite (son effet dépend de la version de Compose \
                 et elle ne protège pas du vide) : écrire la forme `{forme_ajout}`"
            ));
        }
        ValeurEnv::NonReconnue(d) => {
            return Some(format!("{f} : `{cle}` de forme non reconnue : {d}"));
        }
        ValeurEnv::Scalaire(s) => s.as_str(),
    };
    if v.is_empty() {
        return Some(format!(
            "{f} : `{cle}` vaut la chaîne vide littérale — écrire `{forme_ajout}`"
        ));
    }
    if VALEURS_COMPOSEES
        .iter()
        .any(|(n, c, _)| *n == cle && *c == compose)
    {
        let (_, _, motif) = VALEURS_COMPOSEES
            .iter()
            .find(|(n, c, _)| *n == cle && *c == compose)
            .unwrap();
        return (!v.contains(motif)).then(|| {
            format!("{f} : `{cle}` (valeur composée) doit contenir `{motif}`, trouvé `{v}`")
        });
    }
    let vide_signifiant = VIDE_SIGNIFIANT.contains(&cle);
    let sans_defaut = SANS_DEFAUT.contains(&cle);
    let ajout = AJOUTS.iter().any(|(n, c)| *n == cle && *c == compose);
    match interpolation(v) {
        None if v.contains('$') => {
            Some(format!("{f} : `{cle}` : interpolation non reconnue `{v}`"))
        }
        None => {
            // Littéral non vide.
            if vide_signifiant || sans_defaut || ajout {
                Some(format!(
                    "{f} : `{cle}` : littéral `{v}` interdit pour cette variable (liste fermée)"
                ))
            } else {
                None
            }
        }
        Some(i) => {
            let nom = match i {
                Interpolation::Simple(n)
                | Interpolation::DefautSiVide(n, _)
                | Interpolation::DefautSiAbsente(n, _)
                | Interpolation::ObligatoireNonVide(n)
                | Interpolation::ObligatoireSiAbsente(n) => n,
            };
            if nom != cle {
                return Some(format!(
                    "{f} : `{cle}` interpole une AUTRE variable : `{v}`"
                ));
            }
            if vide_signifiant {
                return match i {
                    Interpolation::DefautSiAbsente(_, d) if !d.is_empty() => None,
                    _ => Some(format!(
                        "{f} : `{cle}` (VIDE_SIGNIFIANT) doit porter la forme `${{{cle}-défaut}}` (tiret sans \
                         deux-points : une ligne vide de .env doit transmettre le vide), trouvé `{v}`"
                    )),
                };
            }
            if let Interpolation::DefautSiAbsente(..) = i {
                return Some(format!(
                    "{f} : `{cle}` porte `${{NOM-…}}` (tiret sans deux-points) sans être dans VIDE_SIGNIFIANT : `{v}`"
                ));
            }
            if sans_defaut || ajout {
                let liste = if sans_defaut {
                    "SANS_DEFAUT"
                } else {
                    "AJOUTS (#550)"
                };
                return match i {
                    Interpolation::DefautSiVide(_, "") => None,
                    _ => Some(format!(
                        "{f} : `{cle}` ({liste}) doit porter la forme `{forme_ajout}` et aucune autre, trouvé `{v}`"
                    )),
                };
            }
            None
        }
    }
}

// ---------------------------------------------------------------------------
// (M) Service MariaDB et mots de passe publiés — Story 15-13a (#551)
// ---------------------------------------------------------------------------

/// Mots de passe du service `mariadb` qui doivent être OBLIGATOIRES et non
/// vides (`${NOM:?…}`) : sans eux, Compose refuse de démarrer.
const MOTS_DE_PASSE_MARIADB: &[&str] = &["MARIADB_ROOT_PASSWORD", "MARIADB_PASSWORD"];

/// Sous-chaîne commune aux anciens mots de passe par défaut (`kesh_dev`,
/// `kesh_dev_root`) : aucun fichier distribué ne doit la porter.
const MOT_DE_PASSE_PUBLIE: &str = "kesh_dev";

/// Fichiers distribués où [`MOT_DE_PASSE_PUBLIE`] est interdit, commentaires
/// compris.
const FICHIERS_DISTRIBUES: &[&str] = &[
    "docker-compose.yml",
    "docker-compose.prod.yml",
    ".env.example",
];

/// (M) Le service `mariadb` de `source` : aucune clé `ports:` (un port publié
/// par Docker contourne le pare-feu de l'hôte), et chaque entrée de
/// [`MOTS_DE_PASSE_MARIADB`] de la forme `${NOM:?…}` sur elle-même. La forme
/// `${NOM?…}` est refusée : une ligne `NOM=` décommentée sans valeur ferait
/// démarrer MariaDB sans mot de passe. La valeur est lue après l'analyse YAML :
/// une valeur citée arrive sans ses guillemets.
fn controle_service_mariadb(source: &str) -> Vec<String> {
    let svc = match service(source, "mariadb") {
        Ok(s) => s,
        Err(e) => return e,
    };
    let mut erreurs = Vec::new();
    if svc.ports {
        erreurs.push(
            "service `mariadb` : clé `ports:` présente — le port de la base ne doit pas être publié \
             (Kesh passe par le réseau interne ; un port publié contourne UFW)"
                .to_string(),
        );
    }
    for cle in MOTS_DE_PASSE_MARIADB {
        let ok = match svc.valeur(cle) {
            Some(ValeurEnv::Scalaire(v)) => {
                matches!(interpolation(v), Some(Interpolation::ObligatoireNonVide(n)) if n == *cle)
            }
            _ => false,
        };
        if !ok {
            erreurs.push(format!(
                "service `mariadb` : `{cle}` doit être de la forme `${{{cle}:?message}}` (obligatoire, \
                 non vide, sans défaut), trouvé {:?}",
                svc.valeur(cle)
            ));
        }
    }
    erreurs
}

/// (M) Aucun fichier distribué ne porte [`MOT_DE_PASSE_PUBLIE`] (recherche de
/// sous-chaîne, commentaires compris) ; dans `.env.example`, toute ligne
/// d'affectation d'un mot de passe MariaDB est commentée. `fichiers` :
/// (nom, contenu).
fn controle_mots_de_passe_publies(fichiers: &[(&str, &str)]) -> Vec<String> {
    let mut erreurs = Vec::new();
    for (nom, contenu) in fichiers {
        for (i, ligne) in contenu.lines().enumerate() {
            if ligne.contains(MOT_DE_PASSE_PUBLIE) {
                erreurs.push(format!(
                    "{nom}:{} : mot de passe publié `{MOT_DE_PASSE_PUBLIE}`",
                    i + 1
                ));
            }
        }
        if *nom == ".env.example" {
            for (i, var, commentee) in lignes_affectation(contenu) {
                if MOTS_DE_PASSE_MARIADB.contains(&var.as_str()) && !commentee {
                    erreurs.push(format!(
                        ".env.example:{} : `{var}` doit rester commentée et sans valeur (un `.env` copié \
                         du gabarit fait alors refuser Compose)",
                        i + 1
                    ));
                }
            }
        }
    }
    erreurs
}

/// Lignes d'affectation de `.env.example` : `^#?\s*[A-Z][A-Z0-9_]*=`. Rend
/// (index de ligne, nom, commentée).
fn lignes_affectation(source: &str) -> Vec<(usize, String, bool)> {
    let mut out = Vec::new();
    for (i, ligne) in source.lines().enumerate() {
        let (commentee, reste) = match ligne.strip_prefix('#') {
            Some(r) => (true, r),
            None => (false, ligne),
        };
        let reste = reste.trim_start();
        let Some((nom, _)) = reste.split_once('=') else {
            continue;
        };
        let mut chars = nom.chars();
        let premier_ok = chars.next().is_some_and(|c| c.is_ascii_uppercase());
        if premier_ok && chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_') {
            out.push((i, nom.to_string(), commentee));
        }
    }
    out
}

/// Le texte des cinq lignes qui précèdent la ligne `index` : `#` et espaces de
/// tête retirés, lignes jointes par une espace, espaces multiples réduits,
/// en minuscules (le marqueur peut être coupé par un retour à la ligne).
fn commentaire_precedent(source: &str, index: usize) -> String {
    let lignes: Vec<&str> = source.lines().collect();
    let debut = index.saturating_sub(5);
    let joint = lignes[debut..index]
        .iter()
        .map(|l| l.trim_start().trim_start_matches('#').trim_start())
        .collect::<Vec<_>>()
        .join(" ");
    joint
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Contrôle des raisons de [`VIDE_SIGNIFIANT`] et [`SANS_DEFAUT`] dans
/// `.env.example`, et cohérence interne de [`AJOUTS`].
fn controle_raisons_valeurs(env_example: &str, lues: &BTreeSet<&str>) -> Vec<String> {
    let mut erreurs = Vec::new();
    let affectations = lignes_affectation(env_example);
    for nom in VIDE_SIGNIFIANT {
        if !lues.contains(nom) {
            erreurs.push(format!(
                "VIDE_SIGNIFIANT : `{nom}` n'est pas lue (entrée inutilisée)"
            ));
        }
        let lignes: Vec<_> = affectations.iter().filter(|(_, n, _)| n == nom).collect();
        if lignes.is_empty() {
            erreurs.push(format!(
                "VIDE_SIGNIFIANT : `{nom}` n'a aucune ligne d'affectation dans .env.example"
            ));
        }
        for (i, _, _) in lignes {
            let texte = commentaire_precedent(env_example, *i);
            if !texte.contains(MARQUEUR_VIDE_SIGNIFIANT) || texte.contains("ou absent") {
                erreurs.push(format!(
                    ".env.example:{} : le commentaire qui précède `{nom}=` doit dire « {MARQUEUR_VIDE_SIGNIFIANT} » \
                     et ne pas dire « ou absent » (sous Docker, absente ≠ vide) — lu : « {texte} »",
                    i + 1
                ));
            }
        }
    }
    for nom in SANS_DEFAUT {
        if !lues.contains(nom) {
            erreurs.push(format!(
                "SANS_DEFAUT : `{nom}` n'est pas lue (entrée inutilisée)"
            ));
        }
        for (i, n, commentee) in &affectations {
            if n == nom && !commentee {
                erreurs.push(format!(
                    ".env.example:{} : `{nom}` (SANS_DEFAUT) doit rester commentée — optionnelle, absente → onboarding `/setup`",
                    i + 1
                ));
            }
        }
    }
    let mut vus = BTreeSet::new();
    for (nom, compose) in AJOUTS {
        if !lues.contains(nom) {
            erreurs.push(format!("AJOUTS : `{nom}` n'est pas lue"));
        }
        if !vus.insert((*nom, *compose)) {
            erreurs.push(format!(
                "AJOUTS : couple en double ({nom}, {})",
                compose.fichier()
            ));
        }
    }
    erreurs
}

fn controle_valeurs(y: &Service, p: &Service) -> Vec<String> {
    let mut erreurs = Vec::new();
    for (compose, svc) in [(Compose::Y, y), (Compose::P, p)] {
        for (cle, valeur) in &svc.environnement {
            erreurs.extend(controle_valeur(compose, cle, valeur));
        }
    }
    erreurs
}

// ---------------------------------------------------------------------------
// (E) `.env.example`
// ---------------------------------------------------------------------------

fn controle_env_example(
    env_example: &str,
    y: &Service,
    y_source: &str,
    lues: &BTreeSet<&str>,
) -> Vec<String> {
    let mut erreurs = Vec::new();
    let affectations = lignes_affectation(env_example);
    let noms: BTreeSet<&str> = affectations.iter().map(|(_, n, _)| n.as_str()).collect();
    for (i, nom, _) in &affectations {
        if !(lues.contains(nom.as_str())
            || HOTE.contains(&nom.as_str())
            || MARIADB.contains(&nom.as_str()))
        {
            erreurs.push(format!(
                ".env.example:{} : `{nom}=` nomme une variable que ni le code, ni un montage, ni MariaDB n'utilise",
                i + 1
            ));
        }
    }
    for nom in lues {
        if *nom == "KESH_TEST_MODE" {
            continue;
        }
        if !noms.contains(nom) {
            erreurs.push(format!(
                "`{nom}` est lue par Kesh mais n'a aucune ligne d'affectation dans .env.example"
            ));
        }
    }
    // Exception unique, contrôlée dans les deux sens.
    if !env_example.contains("DO NOT SET KESH_TEST_MODE") {
        erreurs.push(".env.example doit contenir « DO NOT SET KESH_TEST_MODE »".to_string());
    }
    if noms.contains("KESH_TEST_MODE") {
        erreurs.push(
            ".env.example ne doit porter AUCUNE ligne d'affectation de KESH_TEST_MODE".to_string(),
        );
    }
    let volumes_y: Vec<&str> = y.volumes.iter().filter_map(|v| v.as_str()).collect();
    for nom in HOTE {
        let motif = format!("${{{nom}");
        if !volumes_y.iter().any(|v| v.contains(&motif)) {
            erreurs.push(format!(
                "HOTE : `{motif}` n'apparaît dans aucun montage de `kesh-api` de docker-compose.yml"
            ));
        }
        if !noms.contains(nom) {
            erreurs.push(format!("HOTE : `{nom}` n'a aucune ligne d'affectation dans .env.example (entrée inutilisée)"));
        }
    }
    for nom in MARIADB {
        let motif = format!("${{{nom}");
        if !y_source.contains(&motif) {
            erreurs.push(format!(
                "MARIADB : `{motif}` n'apparaît pas dans docker-compose.yml"
            ));
        }
        if !noms.contains(nom) {
            erreurs.push(format!("MARIADB : `{nom}` n'a aucune ligne d'affectation dans .env.example (entrée inutilisée)"));
        }
    }
    erreurs
}

// ---------------------------------------------------------------------------
// (F) Fantômes
// ---------------------------------------------------------------------------

/// Jetons `KESH_[A-Z0-9_]*[A-Z0-9]` d'un texte (après `\_` → `_`). Rend
/// (jeton, est un préfixe — immédiatement suivi de `_`).
fn jetons_kesh(texte: &str) -> Vec<(String, bool)> {
    let texte = texte.replace("\\_", "_");
    let octets = texte.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(pos) = texte[i..].find("KESH_") {
        let debut = i + pos;
        let mut fin = debut + "KESH_".len();
        while fin < octets.len()
            && (octets[fin].is_ascii_uppercase()
                || octets[fin].is_ascii_digit()
                || octets[fin] == b'_')
        {
            fin += 1;
        }
        // Le jeton finit sur un caractère alphanumérique.
        let mut fin_jeton = fin;
        while fin_jeton > debut + "KESH_".len() && octets[fin_jeton - 1] == b'_' {
            fin_jeton -= 1;
        }
        if fin_jeton > debut + "KESH_".len() {
            let prefixe = fin_jeton < octets.len() && octets[fin_jeton] == b'_';
            out.push((texte[debut..fin_jeton].to_string(), prefixe));
        }
        i = fin.max(debut + 1);
    }
    out
}

fn controle_fantomes(corpus: &[(String, String)], lues: &BTreeSet<&str>) -> Vec<String> {
    let connues: BTreeSet<&str> = lues.iter().copied().chain(HOTE.iter().copied()).collect();
    let mut erreurs = BTreeSet::new();
    for (fichier, texte) in corpus {
        for (jeton, prefixe) in jetons_kesh(texte) {
            let ok = if prefixe {
                let p = format!("{jeton}_");
                connues.iter().any(|c| c.starts_with(&p))
            } else {
                connues.contains(jeton.as_str())
            };
            if !ok {
                let quoi = if prefixe { "préfixe" } else { "variable" };
                erreurs.insert(format!(
                    "{fichier} : {quoi} fantôme `{jeton}` — lue nulle part"
                ));
            }
        }
    }
    erreurs.into_iter().collect()
}

fn corpus_texte() -> Vec<(String, String)> {
    let mut fichiers = vec![
        ".env.example".to_string(),
        Compose::Y.fichier().to_string(),
        Compose::P.fichier().to_string(),
    ];
    for locale in ["de-CH", "en-CH", "fr-CH", "it-CH"] {
        fichiers.push(format!("crates/kesh-i18n/locales/{locale}/messages.ftl"));
    }
    let mut tex: Vec<String> = std::fs::read_dir(racine().join("docs/manual/fr"))
        .expect("docs/manual/fr lisible")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".tex"))
        .map(|n| format!("docs/manual/fr/{n}"))
        .collect();
    tex.sort();
    assert!(
        !tex.is_empty(),
        "aucun .tex dans docs/manual/fr — test muet"
    );
    fichiers.extend(tex);
    fichiers
        .into_iter()
        .map(|f| {
            let t = lire(&f);
            (f, t)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// (L) Lectures — le test lit le code (Story 15-11b, C78)
// ---------------------------------------------------------------------------
//
// Le test ne reconnaît AUCUNE forme d'appel. Il relève chaque occurrence d'un
// jeton surveillé dans le flux de jetons du code de production et exige
// qu'elle figure, à son emplacement et sous sa forme, dans la liste fermée
// [`EMPLACEMENTS_AUTORISES`]. Faux rouge possible (une ligne à ajouter) ; pas
// de faux vert pour une lecture écrite dans `crates/*/src` par un jeton
// surveillé — et rien de plus n'est affirmé (revue P1, B4/E2) : `build.rs`,
// `examples/` et `benches/` ne sont pas lus, ni les lectures internes aux
// dépendances (voir « Ce qu'elle n'établit PAS » en tête du fichier).

/// Jetons surveillés (identifiants, comparés exactement). Toute lecture de
/// l'environnement écrite dans `crates/*/src` passe par l'un d'eux :
/// - `env` (sauf `env!(…)`, macro de compilation) — `std::env::var`, `var_os`,
///   `vars`, `temp_dir`, tout `use … env …` et tout renommage ;
/// - `dotenvy` — `dotenvy::var`, `dotenv_iter`, `use`, `extern crate … as` ;
/// - l'API de lecture de `tracing-subscriber` : `from_default_env`,
///   `try_from_default_env`, `EnvFilter` (tout appel, tout alias), `Builder`,
///   `with_env_var`, `from_env_lossy`, `try_from_env`, et `init` / `try_init`
///   (`fmt::init()` lit `RUST_LOG` sans autre jeton) ;
/// - les indirections de lecture de `kesh-api` : `env_nonempty` (la seule
///   fonction qui lit, C75), `parse_strict_bool`, `env_flag_enabled`,
///   `init_tracing`.
const JETONS_SURVEILLES: &[&str] = &[
    "env",
    "dotenvy",
    "from_default_env",
    "try_from_default_env",
    "EnvFilter",
    "Builder",
    "with_env_var",
    "from_env_lossy",
    "try_from_env",
    "init",
    "try_init",
    "env_nonempty",
    "parse_strict_bool",
    "env_flag_enabled",
    "init_tracing",
];

/// Jetons surveillés qui ne lisent aucun nom par eux-mêmes : leurs littéraux
/// d'argument ne sont pas des noms lus.
const JETONS_SANS_NOM: &[&str] = &["init_tracing", "init", "try_init"];

/// Forme d'une occurrence autorisée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Forme {
    /// Fenêtre exacte, jetons séparés par une espace.
    Exacte(&'static str),
    /// La fenêtre se termine par un groupe `( … )` dont le premier jeton est
    /// un littéral de chaîne, suivi de `,` ou de rien.
    Litteral,
}

/// Une entrée de la liste fermée : à cet emplacement, ce jeton apparaît sous
/// cette forme exactement `nombre` fois.
#[derive(Debug, Clone, Copy)]
struct Autorisation {
    fichier: &'static str,
    emplacement: &'static str,
    jeton: &'static str,
    forme: Forme,
    nombre: usize,
}

const fn autorise(
    fichier: &'static str,
    emplacement: &'static str,
    jeton: &'static str,
    forme: Forme,
    nombre: usize,
) -> Autorisation {
    Autorisation {
        fichier,
        emplacement,
        jeton,
        forme,
        nombre,
    }
}

const CONFIG: &str = "kesh-api/src/config.rs";
const MAIN: &str = "kesh-api/src/main.rs";
const LOGGING: &str = "kesh-api/src/logging.rs";
const ONBOARDING: &str = "kesh-api/src/routes/onboarding.rs";
const ADMIN: &str = "kesh-api/src/routes/admin.rs";

/// **Liste fermée** des occurrences de jetons surveillés dans le code de
/// production (AC3 de la 15-11b). Toute occurrence non couverte rougit ;
/// toute entrée dont le nombre diffère rougit (liste périmée).
///
/// Ajouter une lecture d'environnement, c'est appeler
/// `config::env_nonempty("NOM")` à un emplacement de cette liste — et ce test
/// exige alors que `NOM` soit transmis par les deux compose et documenté dans
/// `.env.example`.
const EMPLACEMENTS_AUTORISES: &[Autorisation] = &[
    // La seule fonction qui lit (C75).
    autorise(
        CONFIG,
        "env_nonempty",
        "env_nonempty",
        Forme::Exacte("env_nonempty ( name : & str )"),
        1,
    ),
    autorise(
        CONFIG,
        "env_nonempty",
        "env",
        Forme::Exacte("env :: var_os ( name )"),
        1,
    ),
    // Ses appels littéraux.
    autorise(
        CONFIG,
        "Config::from_env",
        "env_nonempty",
        Forme::Litteral,
        31,
    ),
    autorise(
        CONFIG,
        "Config::from_env",
        "parse_strict_bool",
        Forme::Litteral,
        2,
    ),
    autorise(
        CONFIG,
        "Config::from_env",
        "dotenvy",
        Forme::Exacte("dotenvy :: dotenv ( )"),
        1,
    ),
    autorise(
        CONFIG,
        "LogConfig::from_env",
        "env_nonempty",
        Forme::Litteral,
        4,
    ),
    autorise(
        MAIN,
        "main",
        "dotenvy",
        Forme::Exacte("dotenvy :: dotenv ( )"),
        1,
    ),
    autorise(MAIN, "main", "env_nonempty", Forme::Litteral, 2),
    autorise(
        MAIN,
        "main",
        "init_tracing",
        Forme::Exacte("init_tracing ( & log_config )"),
        1,
    ),
    // Indirections : définition et lecture.
    autorise(
        CONFIG,
        "parse_strict_bool",
        "parse_strict_bool",
        Forme::Exacte("parse_strict_bool ( var : & str , default : bool )"),
        1,
    ),
    autorise(
        CONFIG,
        "parse_strict_bool",
        "env_nonempty",
        Forme::Exacte("env_nonempty ( var )"),
        1,
    ),
    autorise(
        LOGGING,
        "init_tracing",
        "init_tracing",
        Forme::Exacte("init_tracing ( cfg : & LogConfig )"),
        1,
    ),
    autorise(
        LOGGING,
        "init_tracing",
        "env_nonempty",
        Forme::Exacte("env_nonempty ( EnvFilter :: DEFAULT_ENV )"),
        1,
    ),
    autorise(
        LOGGING,
        "init_tracing",
        "EnvFilter",
        Forme::Exacte("EnvFilter :: DEFAULT_ENV"),
        1,
    ),
    autorise(
        LOGGING,
        "init_tracing",
        "init",
        Forme::Exacte("init ( )"),
        1,
    ),
    autorise(
        ONBOARDING,
        "env_flag_enabled",
        "env_flag_enabled",
        Forme::Exacte("env_flag_enabled ( name : & str )"),
        1,
    ),
    autorise(
        ONBOARDING,
        "env_flag_enabled",
        "env_nonempty",
        Forme::Exacte("env_nonempty ( name )"),
        1,
    ),
    autorise(ONBOARDING, "reset", "env_flag_enabled", Forme::Litteral, 1),
    // `EnvFilter` sans lecture : import et construction depuis une chaîne.
    autorise(
        LOGGING,
        "<hors fonction>",
        "EnvFilter",
        Forme::Exacte("EnvFilter"),
        1,
    ),
    autorise(
        LOGGING,
        "build_log_filter",
        "EnvFilter",
        Forme::Exacte("EnvFilter"),
        1,
    ),
    autorise(
        LOGGING,
        "build_log_filter",
        "EnvFilter",
        Forme::Exacte("EnvFilter :: new ( raw )"),
        1,
    ),
    // `TMPDIR` : inventorié, nom non compté dans l'ensemble lu (angle mort).
    autorise(
        ADMIN,
        "stream_via_tempfile",
        "env",
        Forme::Exacte("env :: temp_dir ( )"),
        1,
    ),
];

/// Position (ligne 1-based, colonne 0-based), comparable.
type Pos = (usize, usize);

fn pos(lc: proc_macro2::LineColumn) -> Pos {
    (lc.line, lc.column)
}

/// Une occurrence relevée d'un jeton surveillé.
#[derive(Debug, Clone)]
struct Occurrence {
    fichier: String,
    emplacement: String,
    jeton: String,
    /// Fenêtre rendue, jetons séparés par une espace.
    fenetre: String,
    ligne: usize,
    /// La fenêtre a la forme [`Forme::Litteral`].
    litteral: bool,
    /// Nom lu par cette occurrence, s'il y en a un.
    nom: Option<String>,
}

/// Résultat de la lecture d'un ensemble de sources.
#[derive(Debug, Default)]
struct Analyse {
    occurrences: Vec<Occurrence>,
    /// (fichier, littéraux de chaîne du code de production, un par ligne).
    litteraux: Vec<(String, String)>,
    /// Fichiers lus.
    fichiers: Vec<String>,
    /// Nombre de plages exclues (`cfg(test)`) par fichier.
    exclusions: BTreeMap<String, usize>,
    /// Ensemble lu : les noms lus par toutes les occurrences, autorisées ou non.
    noms: BTreeSet<String>,
}

/// Un attribut qui retire l'élément du code de production : `#[test]`,
/// `#[cfg(test)]`, `#[cfg(all(test, …))]` (`test` au premier niveau de
/// `all`). `cfg(any(test, …))`, `cfg(not(test))` et `cfg_attr` n'excluent pas.
fn attribut_de_test(a: &syn::Attribute) -> bool {
    use syn::punctuated::Punctuated;
    use syn::{Meta, Token};
    if a.path().is_ident("test") {
        return true;
    }
    if !a.path().is_ident("cfg") {
        return false;
    }
    match a.parse_args::<Meta>() {
        Ok(Meta::Path(p)) => p.is_ident("test"),
        Ok(Meta::List(l)) if l.path.is_ident("all") => l
            .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .map(|v| {
                v.iter()
                    .any(|m| matches!(m, Meta::Path(p) if p.is_ident("test")))
            })
            .unwrap_or(false),
        _ => false,
    }
}

/// Les attributs externes en tête d'un nœud (élément, instruction…).
fn attributs_de_tete<T: quote::ToTokens>(n: &T) -> Vec<syn::Attribute> {
    use syn::parse::{ParseStream, Parser};
    let tokens = n.to_token_stream();
    let lire = |input: ParseStream| {
        let a = input.call(syn::Attribute::parse_outer)?;
        input.parse::<proc_macro2::TokenStream>()?;
        Ok(a)
    };
    lire.parse2(tokens).unwrap_or_default()
}

/// Plages relevées par le visiteur `syn` : fonctions (avec leur emplacement),
/// modules en ligne, éléments exclus (`cfg(test)`).
#[derive(Default)]
struct Plages {
    fonctions: Vec<(Pos, Pos, String)>,
    modules: Vec<(Pos, Pos, String)>,
    exclues: Vec<(Pos, Pos)>,
    chemin_modules: Vec<String>,
    impl_courant: Vec<String>,
    trait_courant: Vec<String>,
}

fn plage<T: syn::spanned::Spanned>(n: &T) -> (Pos, Pos) {
    let s = n.span();
    (pos(s.start()), pos(s.end()))
}

impl Plages {
    fn prefixe(&self) -> String {
        self.chemin_modules
            .iter()
            .map(|m| format!("{m}::"))
            .collect()
    }

    fn exclure_si<T: quote::ToTokens + syn::spanned::Spanned>(&mut self, n: &T) {
        if attributs_de_tete(n).iter().any(attribut_de_test) {
            let (debut, fin) = plage(n);
            // `visit_stmt` puis `visit_item` présentent le même nœud pour un
            // `Stmt::Item` : on ne compte une plage qu'une fois (revue P1, E3),
            // pour que `Analyse::exclusions` soit un nombre de plages.
            if !self.exclues.contains(&(debut, fin)) {
                self.exclues.push((debut, fin));
            }
        }
    }

    fn fonction<T: syn::spanned::Spanned>(&mut self, n: &T, nom: String) {
        let (debut, fin) = plage(n);
        self.fonctions
            .push((debut, fin, format!("{}{nom}", self.prefixe())));
    }

    /// L'emplacement d'une position : la fonction la plus intérieure qui la
    /// contient, sinon `<hors fonction>` (préfixé du module en ligne).
    fn emplacement(&self, p: Pos) -> String {
        let contient = |(d, f): (&Pos, &Pos)| *d <= p && p <= *f;
        if let Some((_, _, nom)) = self
            .fonctions
            .iter()
            .filter(|(d, f, _)| contient((d, f)))
            .max_by_key(|(d, _, _)| *d)
        {
            return nom.clone();
        }
        match self
            .modules
            .iter()
            .filter(|(d, f, _)| contient((d, f)))
            .max_by_key(|(d, _, _)| *d)
        {
            Some((_, _, m)) => format!("{m}::<hors fonction>"),
            None => "<hors fonction>".to_string(),
        }
    }

    fn est_exclue(&self, p: Pos) -> bool {
        self.exclues.iter().any(|(d, f)| *d <= p && p <= *f)
    }
}

fn dernier_segment(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(tp) => tp
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default(),
        syn::Type::Reference(r) => dernier_segment(&r.elem),
        _ => "<type>".to_string(),
    }
}

impl<'ast> syn::visit::Visit<'ast> for Plages {
    fn visit_item(&mut self, i: &'ast syn::Item) {
        self.exclure_si(i);
        syn::visit::visit_item(self, i);
    }
    fn visit_impl_item(&mut self, i: &'ast syn::ImplItem) {
        self.exclure_si(i);
        syn::visit::visit_impl_item(self, i);
    }
    fn visit_trait_item(&mut self, i: &'ast syn::TraitItem) {
        self.exclure_si(i);
        syn::visit::visit_trait_item(self, i);
    }
    fn visit_stmt(&mut self, s: &'ast syn::Stmt) {
        self.exclure_si(s);
        syn::visit::visit_stmt(self, s);
    }
    fn visit_item_mod(&mut self, m: &'ast syn::ItemMod) {
        if m.content.is_some() {
            let (debut, fin) = plage(m);
            let chemin = format!("{}{}", self.prefixe(), m.ident);
            self.modules.push((debut, fin, chemin));
            self.chemin_modules.push(m.ident.to_string());
            syn::visit::visit_item_mod(self, m);
            self.chemin_modules.pop();
        } else {
            syn::visit::visit_item_mod(self, m);
        }
    }
    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        self.fonction(f, f.sig.ident.to_string());
        syn::visit::visit_item_fn(self, f);
    }
    fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {
        self.impl_courant.push(dernier_segment(&i.self_ty));
        syn::visit::visit_item_impl(self, i);
        self.impl_courant.pop();
    }
    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
        let ty = self.impl_courant.last().cloned().unwrap_or_default();
        self.fonction(f, format!("{ty}::{}", f.sig.ident));
        syn::visit::visit_impl_item_fn(self, f);
    }
    fn visit_item_trait(&mut self, t: &'ast syn::ItemTrait) {
        self.trait_courant.push(t.ident.to_string());
        syn::visit::visit_item_trait(self, t);
        self.trait_courant.pop();
    }
    fn visit_trait_item_fn(&mut self, f: &'ast syn::TraitItemFn) {
        let tr = self.trait_courant.last().cloned().unwrap_or_default();
        self.fonction(f, format!("{tr}::{}", f.sig.ident));
        syn::visit::visit_trait_item_fn(self, f);
    }
}

/// Rend une suite de jetons : groupes aplatis avec leurs délimiteurs,
/// ponctuations jointes fusionnées (`::`, `->`), un texte par jeton.
fn rendre(seq: &[proc_macro2::TokenTree], out: &mut Vec<String>) {
    use proc_macro2::{Delimiter, Spacing, TokenTree};
    let mut i = 0;
    while i < seq.len() {
        match &seq[i] {
            TokenTree::Group(g) => {
                let (o, f) = match g.delimiter() {
                    Delimiter::Parenthesis => ("(", ")"),
                    Delimiter::Brace => ("{", "}"),
                    Delimiter::Bracket => ("[", "]"),
                    Delimiter::None => ("", ""),
                };
                if !o.is_empty() {
                    out.push(o.to_string());
                }
                let inner: Vec<TokenTree> = g.stream().into_iter().collect();
                rendre(&inner, out);
                if !f.is_empty() {
                    out.push(f.to_string());
                }
            }
            TokenTree::Punct(p) => {
                let mut texte = p.as_char().to_string();
                let mut joint = p.spacing() == Spacing::Joint;
                while joint {
                    match seq.get(i + 1) {
                        Some(TokenTree::Punct(q)) => {
                            texte.push(q.as_char());
                            joint = q.spacing() == Spacing::Joint;
                            i += 1;
                        }
                        _ => break,
                    }
                }
                out.push(texte);
            }
            autre => out.push(autre.to_string()),
        }
        i += 1;
    }
}

/// Le littéral de chaîne qui ouvre un groupe d'arguments, suivi de `,` ou de
/// rien (forme [`Forme::Litteral`]).
fn premier_litteral(args: &[proc_macro2::TokenTree]) -> Option<String> {
    use proc_macro2::TokenTree;
    let TokenTree::Literal(l) = args.first()? else {
        return None;
    };
    let syn::Lit::Str(s) = syn::Lit::new(l.clone()) else {
        return None;
    };
    match args.get(1) {
        None => Some(s.value()),
        Some(TokenTree::Punct(p)) if p.as_char() == ',' => Some(s.value()),
        _ => None,
    }
}

/// Fenêtre d'une occurrence à l'indice `i` de `seq` : l'occurrence, puis les
/// jetons qui la suivent dans le même groupe, jusqu'au premier groupe `( … )`
/// inclus, ou jusqu'au premier `;`, `,`, `{ … }`, `[ … ]` exclu, ou la fin du
/// groupe. Rend (texte, arguments si la fenêtre finit par un groupe `( … )`).
fn fenetre(
    seq: &[proc_macro2::TokenTree],
    i: usize,
) -> (String, Option<Vec<proc_macro2::TokenTree>>) {
    use proc_macro2::{Delimiter, TokenTree};
    let mut fin = i + 1;
    let mut args = None;
    while fin < seq.len() {
        match &seq[fin] {
            TokenTree::Group(g) if g.delimiter() == Delimiter::Parenthesis => {
                args = Some(g.stream().into_iter().collect());
                fin += 1;
                break;
            }
            TokenTree::Group(g) if g.delimiter() != Delimiter::None => break,
            TokenTree::Punct(p) if p.as_char() == ';' || p.as_char() == ',' => break,
            _ => fin += 1,
        }
    }
    let mut rendu = Vec::new();
    rendre(&seq[i..fin], &mut rendu);
    (rendu.join(" "), args)
}

/// Parcourt le flux de jetons d'un fichier (tous les groupes : arguments,
/// blocs, corps de macros, attributs), relève les occurrences surveillées et
/// les littéraux de chaîne, hors plages exclues.
fn parcourir(
    flux: proc_macro2::TokenStream,
    fichier: &str,
    plages: &Plages,
    analyse: &mut Analyse,
    litteraux: &mut Vec<String>,
) {
    use proc_macro2::TokenTree;
    let seq: Vec<TokenTree> = flux.into_iter().collect();
    for (i, t) in seq.iter().enumerate() {
        match t {
            TokenTree::Group(g) => parcourir(g.stream(), fichier, plages, analyse, litteraux),
            TokenTree::Literal(l) => {
                if plages.est_exclue(pos(l.span().start())) {
                    continue;
                }
                if let syn::Lit::Str(s) = syn::Lit::new(l.clone()) {
                    litteraux.push(s.value());
                }
            }
            TokenTree::Ident(id) => {
                // Identifiant brut (`r#env`) : son `Display` garde le préfixe
                // `r#`, qu'on retire pour le comparer aux jetons surveillés
                // (revue P1, E1).
                let brut = id.to_string();
                let jeton = brut.strip_prefix("r#").unwrap_or(&brut).to_string();
                if !JETONS_SURVEILLES.contains(&jeton.as_str()) {
                    continue;
                }
                // `env!(…)` : macro de compilation, non une lecture.
                if jeton == "env"
                    && matches!(seq.get(i + 1), Some(TokenTree::Punct(p)) if p.as_char() == '!')
                {
                    continue;
                }
                let p = pos(id.span().start());
                if plages.est_exclue(p) {
                    continue;
                }
                let (texte, args) = fenetre(&seq, i);
                let litteral = args.as_deref().and_then(premier_litteral);
                let nom = if JETONS_SANS_NOM.contains(&jeton.as_str()) {
                    None
                } else if let Some(n) = &litteral {
                    Some(n.clone())
                } else {
                    args.as_deref().and_then(|a| {
                        let mut r = Vec::new();
                        rendre(a, &mut r);
                        r.ends_with(&["EnvFilter".into(), "::".into(), "DEFAULT_ENV".into()])
                            .then(|| tracing_subscriber::EnvFilter::DEFAULT_ENV.to_string())
                    })
                };
                if let Some(n) = &nom {
                    analyse.noms.insert(n.clone());
                }
                analyse.occurrences.push(Occurrence {
                    fichier: fichier.to_string(),
                    emplacement: plages.emplacement(p),
                    jeton,
                    fenetre: texte,
                    ligne: p.0,
                    litteral: litteral.is_some(),
                    nom,
                });
            }
            TokenTree::Punct(_) => {}
        }
    }
}

/// Lit des sources `(fichier relatif à crates/, contenu)`.
fn analyser(sources: &[(String, String)]) -> Analyse {
    use syn::visit::Visit;
    let mut analyse = Analyse::default();
    for (fichier, contenu) in sources {
        analyse.fichiers.push(fichier.clone());
        let arbre =
            syn::parse_file(contenu).unwrap_or_else(|e| panic!("{fichier} : analyse syn : {e}"));
        let mut plages = Plages::default();
        if arbre.attrs.iter().any(attribut_de_test) {
            // `#![cfg(test)]` : le fichier entier est du code de test.
            plages.exclues.push(((0, 0), (usize::MAX, usize::MAX)));
        }
        plages.visit_file(&arbre);
        analyse
            .exclusions
            .insert(fichier.clone(), plages.exclues.len());
        let flux: proc_macro2::TokenStream = contenu
            .parse()
            .unwrap_or_else(|e| panic!("{fichier} : flux de jetons : {e:?}"));
        let mut litteraux = Vec::new();
        parcourir(flux, fichier, &plages, &mut analyse, &mut litteraux);
        analyse
            .litteraux
            .push((format!("crates/{fichier}"), litteraux.join("\n")));
    }
    analyse
}

/// Confronte les occurrences à la liste fermée.
fn controle_lectures(analyse: &Analyse, autorisations: &[Autorisation]) -> Vec<String> {
    let mut comptes = vec![0usize; autorisations.len()];
    let mut erreurs = Vec::new();
    for o in &analyse.occurrences {
        let couverte = autorisations.iter().position(|e| {
            e.fichier == o.fichier
                && e.emplacement == o.emplacement
                && e.jeton == o.jeton
                && match e.forme {
                    Forme::Exacte(f) => f == o.fenetre,
                    Forme::Litteral => o.litteral,
                }
        });
        match couverte {
            Some(k) => comptes[k] += 1,
            None => erreurs.push(format!(
                "{} :{} [{}] : occurrence de `{}` hors liste — `{}`{}. Toute lecture de \
                 l'environnement passe par `config::env_nonempty` (C75) ; si elle est voulue, \
                 ajouter à EMPLACEMENTS_AUTORISES : autorise({:?}, {:?}, {:?}, Forme::Exacte({:?}), 1)",
                o.fichier,
                o.ligne,
                o.emplacement,
                o.jeton,
                o.fenetre,
                o.nom
                    .as_ref()
                    .map(|n| format!(" (lit `{n}`)"))
                    .unwrap_or_default(),
                o.fichier,
                o.emplacement,
                o.jeton,
                o.fenetre
            )),
        }
    }
    for (e, n) in autorisations.iter().zip(&comptes) {
        if *n != e.nombre {
            erreurs.push(format!(
                "entrée périmée ({}, {}, `{}`, {:?}) : {} occurrence(s) attendue(s), {} trouvée(s)",
                e.fichier, e.emplacement, e.jeton, e.forme, e.nombre, n
            ));
        }
    }
    erreurs
}

/// Les sources de production du workspace : `crates/*/src/**/*.rs`, lues à
/// l'exécution, triées.
fn sources_du_depot() -> Vec<(String, String)> {
    fn parcourir_dossier(dossier: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entrees) = std::fs::read_dir(dossier) else {
            return;
        };
        for e in entrees.flatten() {
            let chemin = e.path();
            if chemin.is_dir() {
                parcourir_dossier(&chemin, out);
            } else if chemin.extension().is_some_and(|x| x == "rs") {
                out.push(chemin);
            }
        }
    }
    let crates = racine().join("crates");
    let mut chemins = Vec::new();
    for c in std::fs::read_dir(&crates)
        .expect("crates/ lisible")
        .flatten()
    {
        parcourir_dossier(&c.path().join("src"), &mut chemins);
    }
    let mut sources: Vec<(String, String)> = chemins
        .into_iter()
        .map(|p| {
            let relatif = p
                .strip_prefix(&crates)
                .expect("sous crates/")
                .to_string_lossy()
                .replace('\\', "/");
            let contenu = std::fs::read_to_string(&p)
                .unwrap_or_else(|e| panic!("lecture de {} impossible : {e}", p.display()));
            (relatif, contenu)
        })
        .collect();
    sources.sort();
    sources
}

/// L'analyse du dépôt, calculée une fois.
fn analyse_depot() -> &'static Analyse {
    static ANALYSE: std::sync::OnceLock<Analyse> = std::sync::OnceLock::new();
    ANALYSE.get_or_init(|| analyser(&sources_du_depot()))
}

/// Les littéraux de chaîne du code de production, corpus du (F) étendu.
fn corpus_code() -> Vec<(String, String)> {
    analyse_depot().litteraux.clone()
}

// ---------------------------------------------------------------------------
// Tests sur le dépôt
// ---------------------------------------------------------------------------

fn services() -> (Service, Service, String) {
    let y_source = lire(Compose::Y.fichier());
    let p_source = lire(Compose::P.fichier());
    let y = service(&y_source, "kesh-api").unwrap_or_else(|e| panic!("docker-compose.yml : {e:?}"));
    let p = service(&p_source, "kesh-api")
        .unwrap_or_else(|e| panic!("docker-compose.prod.yml : {e:?}"));
    (y, p, y_source)
}

/// Garde contre le test muet : une lecture du code qui ne verrait rien
/// rendrait tous les contrôles verts sans rien vérifier.
#[test]
fn garde_lecture_du_code() {
    let a = analyse_depot();
    for f in [CONFIG, MAIN, LOGGING, ONBOARDING] {
        assert!(a.fichiers.iter().any(|x| x == f), "{f} doit être lu");
    }
    assert!(
        a.exclusions.get(CONFIG).copied().unwrap_or(0) >= 1,
        "au moins une plage `cfg(test)` attendue dans {CONFIG} (son `mod tests`)"
    );
    assert_eq!(
        tracing_subscriber::EnvFilter::DEFAULT_ENV,
        "RUST_LOG",
        "`EnvFilter::DEFAULT_ENV` de tracing-subscriber a changé"
    );
    for nom in [
        "DATABASE_URL",
        "KESH_JWT_SECRET",
        "KESH_SMTP_HOST",
        "KESH_PRODUCTION_RESET",
        "RUST_LOG",
    ] {
        assert!(a.noms.contains(nom), "l'ensemble lu doit contenir `{nom}`");
    }
    assert!(
        a.noms.len() >= 41,
        "ensemble lu : {} noms, au moins 41 attendus",
        a.noms.len()
    );
    assert_eq!(
        AJOUTS.len(),
        28,
        "AJOUTS : 28 couples attendus (15 dans docker-compose.yml, 13 dans docker-compose.prod.yml)"
    );
}

#[test]
fn transmission() {
    let (y, p, _) = services();
    let dockerfile = lire("Dockerfile");
    echouer_si(
        controle_transmission(&y, &p, &dockerfile, &lues()),
        "(T) transmission",
    );
}

/// Story 15-13b (AC 8 e, test 19) — les sources des montages par défaut ne se
/// versionnent pas et n'entrent pas dans le contexte de build : pour chaque
/// entrée de [`MONTAGES`] (liste **dérivée**, un montage ajouté demain est
/// contrôlé sans retouche), `.gitignore` porte la ligne ancrée `/x/` (seule
/// tolérance : [`TOLERANCES_NON_ANCREES`]) et `.dockerignore` la ligne `x/`.
/// Lignes comparées **entières** (après `trim`), jamais en sous-chaîne.
#[test]
fn montages_hors_du_depot() {
    assert_eq!(
        TOLERANCES_NON_ANCREES,
        &["log"],
        "seule tolérance admise : log"
    );
    assert!(
        MONTAGES.len() >= 4,
        "assertion de montage : au moins quatre montages attendus, trouvé {}",
        MONTAGES.len()
    );
    assert!(
        MONTAGES
            .iter()
            .any(|(cible, _, _)| *cible == "/data/backup"),
        "assertion de montage : le montage de la sauvegarde pré-import manque"
    );
    let gitignore = lire(".gitignore");
    let dockerignore = lire(".dockerignore");
    let lignes =
        |texte: &str| -> BTreeSet<String> { texte.lines().map(|l| l.trim().to_string()).collect() };
    let (g, d) = (lignes(&gitignore), lignes(&dockerignore));
    let mut erreurs = Vec::new();
    for (_, _, exacte_p) in MONTAGES {
        let dossier = exacte_p
            .strip_prefix("./")
            .unwrap_or_else(|| panic!("source de montage inattendue : {exacte_p}"));
        let ancree = format!("/{dossier}/");
        let tolere =
            TOLERANCES_NON_ANCREES.contains(&dossier) && g.contains(&format!("{dossier}/"));
        if !g.contains(&ancree) && !tolere {
            erreurs.push(format!(
                ".gitignore : ligne `{ancree}` absente (motif ancré exigé)"
            ));
        }
        if !d.contains(&format!("{dossier}/")) {
            erreurs.push(format!(".dockerignore : ligne `{dossier}/` absente"));
        }
    }
    echouer_si(erreurs, "montages hors du dépôt");
}

#[test]
fn valeurs() {
    let (y, p, _) = services();
    let mut erreurs = controle_valeurs(&y, &p);
    erreurs.extend(controle_raisons_valeurs(&lire(".env.example"), &lues()));
    echouer_si(erreurs, "(V) valeurs");
}

#[test]
fn env_example() {
    let (y, _, y_source) = services();
    echouer_si(
        controle_env_example(&lire(".env.example"), &y, &y_source, &lues()),
        "(E) .env.example",
    );
}

/// Test 1 de la fiche 15-13a — (M) le service `mariadb` de
/// `docker-compose.yml`.
#[test]
fn mariadb() {
    echouer_si(
        controle_service_mariadb(&lire(Compose::Y.fichier())),
        "(M) service mariadb",
    );
}

/// Test 2 de la fiche 15-13a — (M) aucun mot de passe publié dans les
/// fichiers distribués ; mots de passe MariaDB commentés dans le gabarit.
#[test]
fn mots_de_passe_publies() {
    let contenus: Vec<(&str, String)> = FICHIERS_DISTRIBUES.iter().map(|f| (*f, lire(f))).collect();
    let fichiers: Vec<(&str, &str)> = contenus.iter().map(|(n, c)| (*n, c.as_str())).collect();
    // Assertion de montage : le gabarit porte bien les deux lignes commentées.
    let gabarit = &contenus
        .iter()
        .find(|(n, _)| *n == ".env.example")
        .expect("montage : .env.example doit figurer dans FICHIERS_DISTRIBUES")
        .1;
    for cle in MOTS_DE_PASSE_MARIADB {
        assert!(
            lignes_affectation(gabarit)
                .iter()
                .any(|(_, v, c)| v == cle && *c),
            "montage : `#{cle}=` attendue dans .env.example"
        );
    }
    echouer_si(
        controle_mots_de_passe_publies(&fichiers),
        "(M) mots de passe publiés",
    );
}

#[test]
fn fantomes() {
    let mut corpus = corpus_texte();
    corpus.extend(corpus_code());
    echouer_si(controle_fantomes(&corpus, &lues()), "(F) fantômes");
}

#[test]
fn lectures() {
    echouer_si(
        controle_lectures(analyse_depot(), EMPLACEMENTS_AUTORISES),
        "(L) lectures",
    );
}

// ---------------------------------------------------------------------------
// (S) Auto-test des extracteurs sur sources synthétiques
// ---------------------------------------------------------------------------

fn svc(env_yaml: &str) -> Service {
    let source = format!("services:\n  kesh-api:\n    environment:\n{env_yaml}");
    service(&source, "kesh-api").expect("YAML synthétique valide")
}

fn valeur_de(env_yaml: &str, cle: &str, compose: Compose) -> Option<String> {
    let s = svc(env_yaml);
    controle_valeur(compose, cle, s.valeur(cle).expect("clé présente"))
}

#[test]
fn s_liste_et_dictionnaire_rendent_les_memes_cles() {
    let d = svc("      KESH_PORT: ${KESH_PORT:-80}\n      KESH_HOST: 0.0.0.0\n");
    let l = svc("      - KESH_PORT=${KESH_PORT:-80}\n      - KESH_HOST=0.0.0.0\n");
    assert_eq!(d.cles(), l.cles());
    assert_eq!(d.valeur("KESH_PORT"), l.valeur("KESH_PORT"));
}

/// Test 4 de la fiche 15-13a — (S) le contrôle (M) du service `mariadb` sur
/// des sources synthétiques.
#[test]
fn s_service_mariadb() {
    let source = |root: &str, mdp: &str, ports: &str| {
        format!(
            "services:\n  mariadb:\n    environment:\n      MARIADB_ROOT_PASSWORD: {root}\n      \
             MARIADB_PASSWORD: {mdp}\n{ports}"
        )
    };
    let ok_root = "${MARIADB_ROOT_PASSWORD:?m}";
    let ok_mdp = "${MARIADB_PASSWORD:?m}";
    // Acceptées : forme `:?` nue, et citée (les guillemets tombent à l'analyse).
    assert_eq!(
        controle_service_mariadb(&source(ok_root, ok_mdp, "")),
        Vec::<String>::new()
    );
    assert_eq!(
        controle_service_mariadb(&source(&format!("\"{ok_root}\""), ok_mdp, "")),
        Vec::<String>::new()
    );
    // Refusées.
    for (cas, src) in [
        (
            "ports",
            source(
                ok_root,
                ok_mdp,
                "    ports:\n      - \"127.0.0.1:3306:3306\"\n",
            ),
        ),
        ("défaut", source(ok_root, "${MARIADB_PASSWORD:-x}", "")),
        ("simple", source(ok_root, "${MARIADB_PASSWORD}", "")),
        ("forme ?", source(ok_root, "${MARIADB_PASSWORD?m}", "")),
        (
            "autre variable",
            source(ok_root, "${MARIADB_ROOT_PASSWORD:?m}", ""),
        ),
        ("littéral", source(ok_root, "kesh_dev", "")),
    ] {
        assert_eq!(
            controle_service_mariadb(&src).len(),
            1,
            "{cas} doit rougir une fois : {src}"
        );
    }
    // Mots de passe publiés : commentaire compris, ligne active refusée.
    let rouge =
        controle_mots_de_passe_publies(&[("docker-compose.yml", "# ancien : kesh_dev_root\n")]);
    assert_eq!(rouge.len(), 1, "{rouge:?}");
    let rouge = controle_mots_de_passe_publies(&[(
        ".env.example",
        "MARIADB_ROOT_PASSWORD=x\n#MARIADB_PASSWORD=\n",
    )]);
    assert_eq!(rouge.len(), 1, "{rouge:?}");
}

#[test]
fn s_cle_sans_valeur_rougit_en_dictionnaire_et_en_liste() {
    assert!(valeur_de("      KESH_SMTP_PORT:\n", "KESH_SMTP_PORT", Compose::P).is_some());
    assert!(valeur_de("      - KESH_SMTP_PORT\n", "KESH_SMTP_PORT", Compose::P).is_some());
}

#[test]
fn s_formes_de_valeur() {
    // `${A:-}` sous A → vert.
    assert_eq!(
        valeur_de(
            "      KESH_SMTP_HOST: ${KESH_SMTP_HOST:-}\n",
            "KESH_SMTP_HOST",
            Compose::Y
        ),
        None
    );
    // `${B}` sous la clé A → rouge.
    assert!(
        valeur_de(
            "      KESH_SMTP_HOST: ${KESH_SMTP_HSOT:-}\n",
            "KESH_SMTP_HOST",
            Compose::Y
        )
        .is_some()
    );
    // `${A-x}` pour A hors VIDE_SIGNIFIANT → rouge.
    assert!(
        valeur_de(
            "      KESH_PORT: ${KESH_PORT-80}\n",
            "KESH_PORT",
            Compose::Y
        )
        .is_some()
    );
    // VIDE_SIGNIFIANT : `${A-x}` vert, `${A:-x}` rouge.
    assert_eq!(
        valeur_de(
            "      KESH_LOG_FILE_PATH: ${KESH_LOG_FILE_PATH-/x}\n",
            "KESH_LOG_FILE_PATH",
            Compose::P
        ),
        None
    );
    assert!(
        valeur_de(
            "      KESH_LOG_FILE_PATH: ${KESH_LOG_FILE_PATH:-/x}\n",
            "KESH_LOG_FILE_PATH",
            Compose::P
        )
        .is_some()
    );
    // SANS_DEFAUT : `${A:-x}` et `${A}` rouges, `${A:-}` vert.
    assert!(
        valeur_de(
            "      KESH_ADMIN_PASSWORD: ${KESH_ADMIN_PASSWORD:-changeme}\n",
            "KESH_ADMIN_PASSWORD",
            Compose::Y
        )
        .is_some()
    );
    assert!(
        valeur_de(
            "      KESH_ADMIN_PASSWORD: ${KESH_ADMIN_PASSWORD}\n",
            "KESH_ADMIN_PASSWORD",
            Compose::P
        )
        .is_some()
    );
    assert_eq!(
        valeur_de(
            "      KESH_ADMIN_PASSWORD: ${KESH_ADMIN_PASSWORD:-}\n",
            "KESH_ADMIN_PASSWORD",
            Compose::P
        ),
        None
    );
    // AJOUTS : `${A:-x}` rouge (défaut du code recopié).
    assert!(
        valeur_de(
            "      KESH_SMTP_PORT: ${KESH_SMTP_PORT:-587}\n",
            "KESH_SMTP_PORT",
            Compose::P
        )
        .is_some()
    );
    // Hors listes : défaut écrit, obligatoire, littéraux typés → verts.
    assert_eq!(
        valeur_de(
            "      KESH_JWT_EXPIRY_MINUTES: ${KESH_JWT_EXPIRY_MINUTES:-15}\n",
            "KESH_JWT_EXPIRY_MINUTES",
            Compose::P
        ),
        None
    );
    assert_eq!(
        valeur_de(
            "      KESH_JWT_SECRET: ${KESH_JWT_SECRET}\n",
            "KESH_JWT_SECRET",
            Compose::P
        ),
        None
    );
    assert_eq!(
        valeur_de("      KESH_PORT: 80\n", "KESH_PORT", Compose::P),
        None
    );
    assert_eq!(
        valeur_de(
            "      KESH_COOKIE_SECURE: true\n",
            "KESH_COOKIE_SECURE",
            Compose::Y
        ),
        None
    );
    // Chaîne vide littérale → rouge.
    assert!(valeur_de("      KESH_PORT: ''\n", "KESH_PORT", Compose::P).is_some());
}

#[test]
fn s_vide_signifiant_controle_chaque_ligne_d_affectation() {
    let lues = lues();
    // Deux lignes d'affectation : une prose sans marqueur, une réelle avec → rouge.
    let deux = "# KESH_LOG_FILE_PATH=/x vaut le défaut\n#\n#\n#\n#\n#\n\
                # ligne présente et vide, contrairement aux autres variables.\n\
                # KESH_LOG_FILE_PATH=/var/log/kesh/kesh.log\n";
    assert!(!controle_raisons_valeurs(deux, &lues).is_empty());
    // « VIDE ou absent » → rouge.
    let ou_absent = "# Contrairement aux autres variables. VIDE ou absent → désactivés\n\
                     # KESH_LOG_FILE_PATH=/var/log/kesh/kesh.log\n";
    assert!(!controle_raisons_valeurs(ou_absent, &lues).is_empty());
    // Marqueur coupé sur deux lignes → trouvé.
    let coupe = "# vide → désactivé, contrairement aux\n#   autres variables.\n\
                 # KESH_LOG_FILE_PATH=/var/log/kesh/kesh.log\n";
    assert!(
        controle_raisons_valeurs(coupe, &lues).is_empty(),
        "{:?}",
        controle_raisons_valeurs(coupe, &lues)
    );
}

#[test]
fn s_lignes_d_affectation() {
    let src = "# is_demo=true. NE PAS définir\n# blabla KESH_X à la fin\n#KESH_LANG=fr\n  # KESH_PORT=80\nRUST_LOG=info\n";
    let noms: Vec<String> = lignes_affectation(src)
        .into_iter()
        .map(|(_, n, _)| n)
        .collect();
    assert_eq!(noms, vec!["KESH_LANG".to_string(), "RUST_LOG".to_string()]);
    // `# blabla KESH_X à la fin` n'est pas une affectation, mais le jeton est vu par (F).
    assert!(jetons_kesh(src).iter().any(|(j, _)| j == "KESH_X"));
}

#[test]
fn s_fantomes() {
    let lues = lues();
    let corpus = vec![(
        "synthetique.tex".to_string(),
        "voir \\texttt{KESH\\_FANTOME} et KESH\\_SMTP\\_*".to_string(),
    )];
    let erreurs = controle_fantomes(&corpus, &lues);
    assert_eq!(erreurs.len(), 1, "{erreurs:?}");
    assert!(erreurs[0].contains("KESH_FANTOME"));
    // Préfixe inconnu → rouge ; préfixe connu → vert.
    assert_eq!(
        jetons_kesh("KESH_SMTP_*"),
        vec![("KESH_SMTP".to_string(), true)]
    );
    let inconnu = vec![("x".to_string(), "KESH_ZZZ_*".to_string())];
    assert_eq!(controle_fantomes(&inconnu, &lues).len(), 1);
}

#[test]
fn s_sources_de_montage() {
    assert_eq!(
        source_montage("${X:-./a}:/data/inbox", "/data/inbox"),
        Some("${X:-./a}")
    );
    assert_eq!(
        source_montage("./a:/data/inbox:ro", "/data/inbox"),
        Some("./a")
    );
    assert_eq!(source_montage("./a:/data/inboxe", "/data/inbox"), None);
    let longue = "services:\n  kesh-api:\n    volumes:\n      - type: bind\n        source: ./a\n        target: /data/inbox\n";
    let s = service(longue, "kesh-api").expect("YAML valide");
    assert!(
        sources_montages(&s).is_err(),
        "une forme longue doit rougir"
    );
    // Story 15-13b (AC 10 d) — `source_conforme` : égalité exacte pour un
    // montage fixe, préfixe pour un montage variable (M14).
    assert!(source_conforme(
        Compose::Y,
        "./backup",
        "./backup",
        "./backup"
    ));
    assert!(!source_conforme(
        Compose::Y,
        "./backups",
        "./backup",
        "./backup"
    ));
    assert!(!source_conforme(
        Compose::Y,
        "${X:-./backup}",
        "./backup",
        "./backup"
    ));
    assert!(source_conforme(
        Compose::Y,
        "${KESH_INBOX_HOST_DIR:-./inbox}",
        "${KESH_INBOX_HOST_DIR:-",
        "./inbox"
    ));
    assert!(source_conforme(
        Compose::P,
        "./backup",
        "./backup",
        "./backup"
    ));
    assert!(!source_conforme(
        Compose::P,
        "./backups",
        "./backup",
        "./backup"
    ));
}

// ---------------------------------------------------------------------------
// (S) Auto-test de (L) et du (F) du code (Story 15-11b)
// ---------------------------------------------------------------------------

const SYNTH: &str = "kesh-api/src/x.rs";

fn analyser_un(src: &str) -> Analyse {
    analyser(&[(SYNTH.to_string(), src.to_string())])
}

/// (écarts de (L), noms lus) pour une source synthétique.
fn lire_src(src: &str, autorisations: &[Autorisation]) -> (Vec<String>, BTreeSet<String>) {
    let a = analyser_un(src);
    (controle_lectures(&a, autorisations), a.noms.clone())
}

fn rouge(src: &str) {
    let (e, _) = lire_src(src, &[]);
    assert!(!e.is_empty(), "rouge attendu pour {src:?}");
}

fn fenetres(src: &str) -> Vec<(String, String)> {
    analyser_un(src)
        .occurrences
        .into_iter()
        .map(|o| (o.emplacement, o.fenetre))
        .collect()
}

#[test]
fn s_lectures_autorisees_et_noms_lus() {
    let lit = [autorise(SYNTH, "f", "env_nonempty", Forme::Litteral, 1)];
    let (e, n) = lire_src(r#"fn f() { let _ = env_nonempty("X"); }"#, &lit);
    assert!(e.is_empty(), "{e:?}");
    assert!(n.contains("X"));
    // Appel qualifié, sans `use` (R-1 = F-2) : la fenêtre commence à `env_nonempty`.
    let (e, n) = lire_src(
        r#"fn f() { let _ = crate::config::env_nonempty("X"); }"#,
        &lit,
    );
    assert!(e.is_empty(), "{e:?}");
    assert!(n.contains("X"));
    let dot = [autorise(
        SYNTH,
        "main",
        "dotenvy",
        Forme::Exacte("dotenvy :: dotenv ( )"),
        1,
    )];
    let (e, _) = lire_src("fn main() { dotenvy::dotenv().ok(); }", &dot);
    assert!(e.is_empty(), "{e:?}");
    // `RUST_LOG` lu par `EnvFilter::DEFAULT_ENV`.
    let (_, n) = lire_src("fn f() { env_nonempty(EnvFilter::DEFAULT_ENV); }", &[]);
    assert!(n.contains("RUST_LOG"), "{n:?}");
}

#[test]
fn s_lectures_hors_liste_rougissent_et_lisent() {
    for (src, nom) in [
        (r#"fn f() { let _ = std::env::var("X"); }"#, Some("X")),
        (r#"fn f() { let _ = dotenvy::var("X"); }"#, Some("X")),
        // Identifiant brut (revue P1, E1).
        (r#"fn f() { let _ = std::r#env::var("X"); }"#, Some("X")),
        (
            r#"fn f() { tracing::info!("{:?}", std::env::var_os("X")); }"#,
            Some("X"),
        ),
        (r#"fn f() { let _ = EnvFilter::from_env("X"); }"#, Some("X")),
        (
            r#"fn f() { let _ = EnvFilter::try_from_env("X"); }"#,
            Some("X"),
        ),
        (
            r#"fn f() { let _ = EnvFilter::builder().with_env_var("X"); }"#,
            Some("X"),
        ),
        ("fn f() { let _ = EnvFilter::from_default_env(); }", None),
        (
            "fn f() { let _ = tracing_subscriber::filter::Builder::default().from_env_lossy(); }",
            None,
        ),
        ("fn f() { tracing_subscriber::fmt::init(); }", None),
        (
            r#"macro_rules! m { () => { std::env::var("X") }; }"#,
            Some("X"),
        ),
    ] {
        let (e, n) = lire_src(src, &[]);
        assert!(!e.is_empty(), "rouge attendu pour {src:?}");
        if let Some(nom) = nom {
            assert!(n.contains(nom), "{nom} lu attendu pour {src:?}, got {n:?}");
        }
    }
}

#[test]
fn s_imports_renommages_et_references_rougissent() {
    for src in [
        "use std::env;",
        "use std::env as e;",
        "use std::env::{self, var_os};",
        "use std::{env, fs};",
        "extern crate dotenvy as d;",
        "fn f() { let _: Vec<_> = v.into_iter().map(std::env::var).collect(); }",
        "fn f() { let _: Vec<_> = v.into_iter().map(env_nonempty).collect(); }",
        "use crate::config::env_nonempty as lire;",
        "use crate::config::env_nonempty;",
        "use tracing_subscriber::EnvFilter as F;",
    ] {
        rouge(src);
    }
}

#[test]
fn s_indirections_et_init_tracing() {
    // Un appel non littéral dans une fonction qui a d'autres entrées (R-1).
    let lit = [autorise(SYNTH, "f", "env_nonempty", Forme::Litteral, 1)];
    let (e, _) = lire_src(
        r#"fn f(n: &str) { env_nonempty("A"); env_nonempty(n); }"#,
        &lit,
    );
    assert_eq!(e.len(), 1, "{e:?}");
    // `init_tracing(&cfg)` : vert à son emplacement, rouge ailleurs (F1).
    let it = [autorise(
        SYNTH,
        "main",
        "init_tracing",
        Forme::Exacte("init_tracing ( & cfg )"),
        1,
    )];
    let (e, n) = lire_src("fn main() { let _g = init_tracing(&cfg); }", &it);
    assert!(e.is_empty(), "{e:?}");
    assert!(n.is_empty(), "init_tracing ne lit aucun nom : {n:?}");
    let (e, _) = lire_src(
        "fn main() { let _g = init_tracing(&cfg); } fn g() { init_tracing(&cfg); }",
        &it,
    );
    assert_eq!(e.len(), 1, "{e:?}");
}

#[test]
fn s_fenetres_et_emplacements() {
    assert_eq!(
        fenetres("fn f() -> EnvFilter { todo!() }"),
        vec![("f".to_string(), "EnvFilter".to_string())]
    );
    assert_eq!(
        fenetres("use tracing_subscriber::{EnvFilter, Layer, fmt};"),
        vec![("<hors fonction>".to_string(), "EnvFilter".to_string())]
    );
    assert_eq!(
        fenetres("fn f() { let _ = std::env::var_os(name); }"),
        vec![("f".to_string(), "env :: var_os ( name )".to_string())]
    );
    // Deux `from_env` dans deux `impl` : deux emplacements.
    let src = "impl A { fn from_env() { env_nonempty(\"X\"); } }\n\
               impl B { fn from_env() { env_nonempty(\"Y\"); } }";
    let emp: Vec<String> = fenetres(src).into_iter().map(|(e, _)| e).collect();
    assert_eq!(emp, vec!["A::from_env", "B::from_env"]);
    // Dernière ligne d'une fonction de plusieurs lignes (R-5 = F-3).
    let src = "fn f() {\n    let a = 1;\n    let _ = a;\n    std::env::var(\"X\")\n}\n";
    assert_eq!(fenetres(src)[0].0, "f");
    let src =
        "#[cfg(test)]\nfn f() {\n    let a = 1;\n    let _ = a;\n    std::env::var(\"X\")\n}\n";
    assert!(fenetres(src).is_empty(), "exclue attendue");
    // Pas des occurrences.
    assert!(fenetres(r#"fn f() { let _ = env!("CARGO_PKG_VERSION"); }"#).is_empty());
    assert!(fenetres("fn parse_strict_bool_multipart() {}").is_empty());
    // Nom en commentaire `//` : ni occurrence, ni nom.
    let a = analyser_un("fn f() {\n    // std::env::var(\"X\")\n}\n");
    assert!(a.occurrences.is_empty() && a.noms.is_empty());
}

#[test]
fn s_liste_fermee_a_nombre_exact() {
    let src = r#"fn f() { env_nonempty("A"); env_nonempty("B"); }"#;
    let plus = [autorise(SYNTH, "f", "env_nonempty", Forme::Litteral, 1)];
    let moins = [autorise(SYNTH, "f", "env_nonempty", Forme::Litteral, 3)];
    let juste = [autorise(SYNTH, "f", "env_nonempty", Forme::Litteral, 2)];
    assert!(!lire_src(src, &plus).0.is_empty());
    assert!(!lire_src(src, &moins).0.is_empty());
    assert!(lire_src(src, &juste).0.is_empty());
    // Une entrée sans occurrence.
    let vide = [autorise(
        SYNTH,
        "g",
        "env",
        Forme::Exacte("env :: var ( \"X\" )"),
        1,
    )];
    assert_eq!(lire_src("fn g() {}", &vide).0.len(), 1);
}

#[test]
fn s_regle_cfg_test() {
    let v = r#"std::env::var("X")"#;
    for exclue in [
        format!("#[cfg(test)] mod t {{ fn f() {{ let _ = {v}; }} }}"),
        format!("#[cfg(all(test, unix))] fn f() {{ let _ = {v}; }}"),
        format!("fn f() {{ #[cfg(test)] let _ = {v}; }}"),
        format!("trait T {{ #[cfg(test)] fn f() {{ let _ = {v}; }} }}"),
        format!("#![cfg(test)]\nfn f() {{ let _ = {v}; }}"),
        format!("#[test] fn f() {{ let _ = {v}; }}"),
    ] {
        assert!(fenetres(&exclue).is_empty(), "exclue attendue : {exclue}");
    }
    for gardee in [
        format!("#[cfg(any(test, feature = \"x\"))] fn f() {{ let _ = {v}; }}"),
        format!("fn f() {{ #[cfg(not(test))] let _ = {v}; }}"),
    ] {
        assert_eq!(fenetres(&gardee).len(), 1, "non exclue attendue : {gardee}");
    }
}

#[test]
fn s_fantomes_du_code() {
    let lues: BTreeSet<&str> = ["KESH_LANG"].into_iter().collect();
    for src in [
        r#"fn f() { tracing::info!("voir KESH_FANTOME"); }"#,
        r#"fn f(o: &mut String) { let _ = write!(o, "voir KESH_FANTOME"); }"#,
        "/// voir KESH_FANTOME\nfn f() {}",
        r#"enum E { #[error("voir KESH_FANTOME")] A }"#,
    ] {
        let a = analyser_un(src);
        let e = controle_fantomes(&a.litteraux, &lues);
        assert_eq!(e.len(), 1, "fantôme attendu pour {src:?} : {e:?}");
    }
    // Le même jeton dans du code de test : non vu (F-2).
    let a = analyser_un(r#"#[cfg(test)] mod t { fn f() { let _ = "KESH_FANTOME"; } }"#);
    assert!(controle_fantomes(&a.litteraux, &lues).is_empty());
    // Commentaire `//` : invisible.
    let a = analyser_un("// KESH_FANTOME\nfn f() {}");
    assert!(controle_fantomes(&a.litteraux, &lues).is_empty());
}

// ---------------------------------------------------------------------------
// Comportement du binaire (revue P1, B3)
// ---------------------------------------------------------------------------

/// Lance le binaire `kesh-api` dans un environnement vidé, dans un répertoire
/// temporaire (aucun `.env` à charger), avec `RUST_LOG` posé à `rust_log` et
/// les deux variables obligatoires **vides** : `Config::from_env` refuse, et le
/// binaire sort en erreur avant toute connexion. `KESH_LOG_FILE_ROTATION`
/// invalide fait rejouer un avertissement (`warn`) par `main` après
/// l'installation de l'abonné. Rend la sortie complète (stdout + stderr).
fn sortie_du_binaire_sans_configuration(rust_log: &str) -> String {
    let sortie = binaire::lancer_binaire(&[
        ("RUST_LOG", rust_log),
        ("DATABASE_URL", ""),
        ("KESH_JWT_SECRET", ""),
        ("KESH_LOG_FILE_ROTATION", "inconnue"),
    ]);
    assert!(
        !sortie.status.success(),
        "le binaire doit refuser de démarrer sans configuration"
    );
    binaire::texte(&sortie)
}

/// `RUST_LOG=""` vaut une absence : le niveau `info` s'applique, et
/// l'avertissement de `KESH_LOG_FILE_ROTATION` est émis. Lu brut,
/// `EnvFilter::new("")` ne laissait passer que les erreurs (CHANGELOG 0.13.0,
/// « Modifié ») — mutation rouge au Change Log de la fiche 15-11b.
#[test]
fn rust_log_vide_vaut_info() {
    for rust_log in ["", "   "] {
        let sortie = sortie_du_binaire_sans_configuration(rust_log);
        // Assertion de montage : la sortie n'est pas muette, et le refus
        // vient bien de la configuration.
        assert!(
            sortie.contains("Erreur de configuration"),
            "RUST_LOG={rust_log:?} : refus de configuration attendu, sortie : {sortie}"
        );
        assert!(
            sortie.contains("KESH_LOG_FILE_ROTATION='inconnue' invalide"),
            "RUST_LOG={rust_log:?} : le niveau `info` doit laisser passer l'avertissement, sortie : {sortie}"
        );
    }
    // Témoin : `RUST_LOG=error` masque l'avertissement — l'assertion
    // ci-dessus discrimine donc bien le niveau appliqué.
    let sortie = sortie_du_binaire_sans_configuration("error");
    assert!(
        sortie.contains("Erreur de configuration"),
        "sortie : {sortie}"
    );
    assert!(
        !sortie.contains("KESH_LOG_FILE_ROTATION='inconnue' invalide"),
        "RUST_LOG=error ne doit pas laisser passer un avertissement, sortie : {sortie}"
    );
}

// ---------------------------------------------------------------------------
// (D) Documentation d'exploitation — Story 15-14b (#575, #554)
// ---------------------------------------------------------------------------

/// Le manuel que lisent G14, G15 et G16.
const MANUEL_ADMIN: &str = "docs/manual/fr/admin-manual.tex";

/// Noms des services (`services:`) et des volumes nommés (`volumes:` de tête)
/// d'un compose. Panique sur un YAML illisible : une garde qui lirait un
/// ensemble vide passerait à vide.
fn services_et_volumes(source: &str) -> (BTreeSet<String>, BTreeSet<String>) {
    let docs = YamlLoader::load_from_str(source).expect("compose : YAML lisible");
    let doc = docs.first().expect("compose : un document YAML");
    let cles = |y: &Yaml| -> BTreeSet<String> {
        match y {
            Yaml::Hash(h) => h.keys().filter_map(scalaire).collect(),
            _ => BTreeSet::new(),
        }
    };
    (cles(&doc["services"]), cles(&doc["volumes"]))
}

/// Numéro de ligne (1-based) d'une position d'octet.
fn ligne_de(source: &str, pos: usize) -> usize {
    source[..pos].matches('\n').count() + 1
}

/// Nombre de `docker compose exec` attendus dans les sections Synology :
/// **zéro, par construction** — `docker-compose.prod.yml` n'a pas de service de
/// base ; le dump et le rechargement passent par un conteneur jetable
/// (`docker run … mariadb:10.11`), non par `exec` (C-15-14-10).
const EXEC_SYNOLOGY_ATTENDUS: usize = 0;

/// **G14** (#575) — chaque service nommé par une commande `docker compose
/// <exec|stop|start|restart|logs|pull|rm> [options] <service>` du manuel
/// d'administration existe : dans les sections Synology, un service de
/// `docker-compose.prod.yml` ; ailleurs, un service de `docker-compose.yml`. Au
/// moins une commande dans chaque groupe (anti-test-muet) ; et, à part, aucun
/// `exec` dans les sections Synology ([`EXEC_SYNOLOGY_ATTENDUS`]) — la base n'y
/// est pas un service du compose. Le défaut fermé : le pré-script Synology
/// faisait `exec -T mariadb` sur un compose sans `mariadb`.
///
/// **Formes non lues** (angle mort écrit) : `docker-compose` (v1),
/// `docker compose -p <projet> …`, `docker exec <conteneur>` — aucune n'est dans
/// le manuel au 2026-10-09. Le réseau des conteneurs jetables des deux scripts
/// Synology est contrôlé par G16 (réglages communs).
#[test]
fn les_services_cites_existent() {
    let source = manuel::desechapper(&lire(MANUEL_ADMIN));
    let synology = manuel::sections_synology(&source);
    let (prod, _) = services_et_volumes(&lire("docker-compose.prod.yml"));
    let (standard, _) = services_et_volumes(&lire("docker-compose.yml"));
    // Assertion de montage : les deux compose ont été lus.
    assert!(prod.contains("kesh-api"), "prod lu à vide : {prod:?}");
    assert!(
        standard.contains("mariadb"),
        "standard lu à vide : {standard:?}"
    );

    // `[ \t]` et non `\s` : une commande sans service (`docker compose ps`) ne
    // doit pas prendre le premier mot de la ligne suivante pour un service.
    let commande = Regex::new(
        r"docker compose(?:[ \t]+-f[ \t]+\S+)?[ \t]+(exec|stop|start|restart|logs|pull|rm)((?:[ \t]+-[A-Za-z]+)*)[ \t]+([A-Za-z0-9_][A-Za-z0-9_.-]*)",
    )
    .unwrap();
    let mut erreurs = Vec::new();
    let (mut dans, mut hors, mut exec_dans) = (0usize, 0usize, 0usize);
    for c in commande.captures_iter(&source) {
        let pos = c.get(0).unwrap().start();
        let (verbe, nom) = (&c[1], &c[3]);
        let ligne = ligne_de(&source, pos);
        if synology.iter().any(|(d, f)| pos >= *d && pos < *f) {
            dans += 1;
            if verbe == "exec" {
                exec_dans += 1;
            }
            if !prod.contains(nom) {
                erreurs.push(format!(
                    ":{ligne} (section Synology) : `{verbe} {nom}`, service absent de docker-compose.prod.yml {prod:?}"
                ));
            }
        } else {
            hors += 1;
            if !standard.contains(nom) {
                erreurs.push(format!(
                    ":{ligne} : `{verbe} {nom}`, service absent de docker-compose.yml {standard:?}"
                ));
            }
        }
    }
    if exec_dans != EXEC_SYNOLOGY_ATTENDUS {
        erreurs.push(format!(
            "sections Synology : {exec_dans} `docker compose exec`, {EXEC_SYNOLOGY_ATTENDUS} attendu(s) — \
             le dump et le rechargement passent par un conteneur jetable, pas par `exec`"
        ));
    }
    if dans == 0 || hors == 0 {
        erreurs.push(format!(
            "commandes trouvées : {dans} dans les sections Synology, {hors} ailleurs — extracteur muet ?"
        ));
    }
    echouer_si(
        erreurs,
        "G14 — services cités par le manuel d'administration",
    );
}

/// **G15** (#575) — tout volume Docker nommé par le manuel d'administration
/// (`/var/lib/docker/volumes/<x>`, « volume \texttt{<x>} ») est un volume de
/// `docker-compose.yml`, nu ou préfixé du nom de projet (`<projet>_`,
/// `kesh_`… : ce que `docker volume ls` affiche) ; et aucun chemin
/// `@docker/volumes/` n'apparaît **dans les sections Synology**, qui décrivent
/// `docker-compose.prod.yml`, lequel n'a aucun volume nommé (ailleurs, un
/// compose standard sur DSM y loge légitimement les siens).
#[test]
fn les_volumes_cites_existent() {
    let source = manuel::desechapper(&lire(MANUEL_ADMIN));
    let synology = manuel::sections_synology(&source);
    let (_, volumes) = services_et_volumes(&lire("docker-compose.yml"));
    assert!(!volumes.is_empty(), "docker-compose.yml : aucun volume lu");
    let mut erreurs = Vec::new();
    for (d, f) in &synology {
        if let Some(p) = source[*d..*f].find("@docker/volumes/") {
            erreurs.push(format!(
                ":{} (section Synology) : chemin `@docker/volumes/` — docker-compose.prod.yml n'a aucun volume nommé",
                ligne_de(&source, d + p)
            ));
        }
    }
    let cite =
        Regex::new(r"/var/lib/docker/volumes/([^\s}/]+)|volume\s+\\texttt\{([^}]+)\}").unwrap();
    let mut trouves = 0;
    for c in cite.captures_iter(&source) {
        trouves += 1;
        let brut = c.get(1).or_else(|| c.get(2)).unwrap().as_str();
        let connu = volumes
            .iter()
            .any(|v| brut == v || brut.ends_with(&format!("_{v}")));
        if !connu {
            erreurs.push(format!(
                ":{} : volume `{brut}` absent de docker-compose.yml {volumes:?}",
                ligne_de(&source, c.get(0).unwrap().start())
            ));
        }
    }
    if trouves == 0 {
        erreurs.push("aucun volume cité trouvé : extracteur muet ?".into());
    }
    echouer_si(
        erreurs,
        "G15 — volumes cités par le manuel d'administration",
    );
}

/// Les « Hyper Backup » permis **hors** des sections Synology (G16 (e)) :
/// liste fermée, un fragment normalisé par site, chacun trouvé **une et une
/// seule** fois (une exemption morte ou ambiguë rougit). Un fragment de site à
/// renvoi va de l'occurrence au `\ref{sec:backup-dsm}` qui la suit : le renvoi
/// fait partie du fragment, si bien que sa présence est prouvée par le décompte
/// lui-même — retirer le renvoi du manuel rend le fragment introuvable.
const HYPER_BACKUP_HORS_SYNOLOGY: &[&str] = &[
    // Avertissement sur les droits des logs (root) : juste, sans rapport avec
    // la base (inventaire de l'AC 1, classe (iii)) — exempté, sans renvoi.
    "Hyper Backup (qui tourne en root) les sauvegarde sans problème",
    // Tableau des méthodes : périmètre réel, renvoi conservé.
    "Hyper Backup DSM & Dossier du compose (documents, logs, .env, backup/) + dump de la base par une tâche planifiée (cf. \\ref{sec:backup-dsm}",
    // `keshtip` : le filet quotidien n'est juste qu'avec la tâche de dump.
    "Hyper Backup avec sa tâche de dump, \\S\\ref{sec:backup-dsm}",
    // Annexe *Opérations en ligne de commande* : une seule occurrence.
    "Hyper Backup (\\S\\ref{sec:backup-dsm}",
];

/// Position du premier motif `re` dans `texte`, ou `None`.
fn premier(texte: &str, re: &str) -> Option<usize> {
    Regex::new(re).unwrap().find(texte).map(|m| m.start())
}

/// `set -e…o pipefail` : le script s'arrête sur toute erreur, pipelines compris.
/// **Couplage écrit** : `set -e` puis `set -o pipefail` sur deux lignes, ou
/// `set -eu -o pipefail`, rougiraient — les scripts sont du dépôt, on les écrit
/// sous cette forme.
const SET_E_PIPEFAIL: &str = r"(?m)^set\s+-[a-zA-Z]*e[a-zA-Z]*o\s+pipefail\b";

/// Les scripts Synology, fichiers du dépôt que le manuel cite (C-15-14-68).
const SCRIPT_DUMP: &str = "scripts/synology/kesh-dump.sh";
const SCRIPT_RECHARGEMENT: &str = "scripts/synology/kesh-restore.sh";
/// Dossier du compose sur Synology, défaut des scripts et chemin du manuel.
const DOSSIER_SYNOLOGY: &str = "/volume1/docker/kesh";

/// Le code d'un script shell, privé de ses lignes de commentaire (`#` en tête
/// de ligne, shebang compris) : sans cela, l'en-tête qui **décrit** le script
/// (« --single-transaction », « sha256sum -c ») satisferait la garde même après
/// le retrait de la commande (mutation de la revue de code P2).
fn code_shell(texte: &str) -> String {
    texte
        .lines()
        .map(|l| {
            if l.trim_start().starts_with('#') {
                ""
            } else {
                l
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Chaque commande `rm` d'un script shell (non `--rm`), avec ses arguments :
/// après un guillemet (`trap 'rm …'`, `sh -c "rm …"`), jusqu'au même guillemet ;
/// sinon jusqu'à la fin de la commande (`;`, `&`, `|`, `)`, accent grave, fin
/// de ligne). Les arguments sont rendus sans leurs guillemets, options écartées.
fn commandes_rm(texte: &str) -> Vec<(usize, Vec<String>)> {
    let rm = Regex::new(r#"(?m)(^|[\s;&|({`'"/])rm[ \t]"#).unwrap();
    rm.captures_iter(texte)
        .map(|c| {
            let debut = c.get(0).unwrap().end();
            let guillemet = c[1].chars().next().filter(|ch| *ch == '\'' || *ch == '"');
            let reste = &texte[debut..];
            let fin = match guillemet {
                Some(q) => reste.find(q).unwrap_or(reste.len()),
                None => reste
                    .find([';', '&', '|', ')', '`', '\n'])
                    .unwrap_or(reste.len()),
            };
            let args = reste[..fin]
                .split_whitespace()
                .filter(|a| !a.starts_with('-'))
                .map(|a| a.trim_matches(|ch| ch == '\'' || ch == '"').to_string())
                .collect();
            (c.get(0).unwrap().start(), args)
        })
        .collect()
}

/// **G16** (#575) — la sauvegarde Synology passe par un dump planifié et se
/// restaure par un autre compte, sans pouvoir détruire la base sur une étape
/// ratée. Depuis la revue de code P2 (C-15-14-68), les deux scripts sont des
/// **fichiers du dépôt** (`scripts/synology/`) que le manuel **cite** ; la garde
/// lit les fichiers — **code seul**, commentaires retirés ([`code_shell`]) — pour
/// leurs invariants et le manuel pour ses renvois, et la
/// recette `scripts/synology/recette.sh` les **exécute** (hors gate,
/// `docs/testing.md`). Clauses :
///
/// - **(a)** aucun `MARIADB_ROOT_PASSWORD` dans les sections Synology ;
/// - **(b)** `kesh-dump.sh` : `set -euo pipefail` et `umask 077` avant le
///   premier `docker run` ; `--single-transaction` ; dump écrit dans
///   `<cible>.tmp`, vérifié par `gzip -t`, renommé par `mv` en `<cible>` —
///   jamais `> <cible>` ; empreinte calculée sur le `.tmp` avant le renommage ;
///   un `--defaults-extra-file`, monté par `-v` ; tout `--network` vaut la
///   variable de réseau, dont le défaut est un réseau de
///   `docker-compose.prod.yml` ; défaut du dossier = [`DOSSIER_SYNOLOGY`] ;
/// - **(c)** `kesh-restore.sh` : `set -euo pipefail` et `umask 077` ; le
///   dossier donné est résolu **en absolu** (`SOURCE=$(cd "$1" && pwd -P)`)
///   avant tout autre usage, et aucun `cd "$SOURCE"` hors d'un sous-shell —
///   sans quoi l'empreinte vérifiée et le dump rechargé peuvent être deux
///   fichiers ; dans cet ordre `sha256sum -c`, `gzip -t`, `docker compose stop
///   kesh-api`, dump de sécurité (`mariadb-dump`) **dans un `if`** — non
///   bloquant, une base absente ne doit pas empêcher le rechargement —, puis
///   `gunzip -c "$SOURCE/…" | docker run … -i` ; chaque
///   `--defaults-extra-file` et chaque fichier d'hôte monté diffèrent de ceux
///   du dump ; réseau comme en (b) ;
/// - **(d)** aucun `rm` ne vise `<cible>` : dans les scripts, toute commande
///   `rm` n'a que des arguments en `.tmp` ([`commandes_rm`], `trap` et
///   guillemets compris) ; dans la prose des sections, même motif sur le nom ;
///   « Ne supprimez pas le dump » présent ;
/// - **(e)** hors des sections, chaque « Hyper Backup » tombe dans un fragment
///   de [`HYPER_BACKUP_HORS_SYNOLOGY`] ;
/// - **(f)** le manuel **cite** les scripts : leur URL de téléchargement dans
///   le dépôt, la commande de la tâche (`bash <dossier>/kesh-dump.sh`), celle du
///   rechargement et celle du secours (`kesh-restore.sh` sur un dossier
///   `avant-restauration/`), le Planificateur de tâches ; il n'en recopie aucun (pas de `lstlisting` qui
///   porte `mariadb-dump` ou `gunzip -c` dans les sections) ; ni « post-script »
///   ni chemin de menu `→ Pré-script` (Hyper Backup n'en a pas, constaté sur DSM
///   le 2026-10-09).
///
/// **Angles morts écrits** : en (d), jokers (`rm kesh_pre*`), `truncate`,
/// `: > <cible>`, `mv <cible> …`, `unlink`, `find -delete` ; en (c), un `-i`
/// placé après une option à valeur (`--network x -i`) ou écrit `--interactive`
/// rougirait (couplage), de même que `docker stop` pour `docker compose stop
/// kesh-api`.
#[test]
fn synology_sauvegarde_la_base_par_le_dump() {
    let brut = lire(MANUEL_ADMIN);
    let bornes = manuel::sections_synology(&brut);
    let parts: Vec<&str> = bornes.iter().map(|(d, f)| &brut[*d..*f]).collect();
    // Assertion de montage : les deux sections sont bien celles attendues.
    assert!(
        parts[0].contains("Container Manager"),
        "sec:synology mal bornée"
    );
    assert!(
        parts[1].starts_with("\\subsection{Backup natif sur Synology DSM (Hyper Backup")
            && parts[1].contains("Snapshot Replication"),
        "sec:backup-dsm mal bornée (titre compris)"
    );
    let normees: Vec<String> = parts
        .iter()
        .map(|p| manuel::normaliser(&manuel::sans_commentaires(p)))
        .collect();
    let (dump_brut, restore_brut) = (lire(SCRIPT_DUMP), lire(SCRIPT_RECHARGEMENT));
    assert!(
        dump_brut.starts_with("#!/bin/bash") && restore_brut.starts_with("#!/bin/bash"),
        "scripts Synology lus à vide"
    );
    let (dump, restore) = (code_shell(&dump_brut), code_shell(&restore_brut));
    assert!(
        dump.contains("docker run") && restore.contains("docker run"),
        "scripts Synology : code vide une fois les commentaires retirés"
    );
    let (_, _, reseaux_prod) = compose_reseaux(&lire("docker-compose.prod.yml"));
    let mut erreurs = Vec::new();

    // (a)
    for (n, label) in normees.iter().zip(manuel::LABELS_SYNOLOGY) {
        if n.contains("MARIADB_ROOT_PASSWORD") {
            erreurs.push(format!("(a) {label} : MARIADB_ROOT_PASSWORD cité"));
        }
    }
    assert!(
        normees[1].contains("DATABASE_URL"),
        "(a) sec:backup-dsm normalisée ne nomme pas DATABASE_URL : lecture à vide ?"
    );

    let reseau_explicite = Regex::new(r"--network\s+(\S+)").unwrap();
    // Réglages communs (b)(c) : dossier et réseau par défaut.
    for (nom, texte) in [("kesh-dump.sh", &dump), ("kesh-restore.sh", &restore)] {
        let defaut = |var: &str| -> Option<String> {
            Regex::new(&format!(r"(?m)^{var}=\$\{{{var}:-([^}}]*)\}}"))
                .unwrap()
                .captures(texte)
                .map(|c| c[1].to_string())
        };
        match defaut("SAUVEGARDE_DOSSIER") {
            Some(d) if d == DOSSIER_SYNOLOGY => {}
            autre => erreurs.push(format!(
                "{nom} : défaut de SAUVEGARDE_DOSSIER {autre:?}, {DOSSIER_SYNOLOGY} attendu"
            )),
        }
        match defaut("SAUVEGARDE_RESEAU") {
            Some(r) if reseaux_prod.contains(&r) => {}
            autre => erreurs.push(format!(
                "{nom} : réseau par défaut {autre:?} absent de docker-compose.prod.yml {reseaux_prod:?}"
            )),
        }
        for c in reseau_explicite.captures_iter(texte) {
            if &c[1] != "\"$SAUVEGARDE_RESEAU\"" {
                erreurs.push(format!(
                    "{nom} : `--network {}` ne passe pas par SAUVEGARDE_RESEAU",
                    &c[1]
                ));
            }
        }
        let premier_run = texte.find("docker run").unwrap_or(usize::MAX);
        match premier(texte, SET_E_PIPEFAIL) {
            Some(p) if p < premier_run => {}
            _ => erreurs.push(format!(
                "{nom} : pas de `set -euo pipefail` avant le premier `docker run`"
            )),
        }
        match premier(texte, r"(?m)^umask\s+0?077\b") {
            Some(p) if p < premier_run => {}
            _ => erreurs.push(format!(
                "{nom} : pas de `umask 077` avant le premier `docker run` (dumps lisibles de tous)"
            )),
        }
    }

    // (b) — kesh-dump.sh
    if !dump.contains("--single-transaction") {
        erreurs.push("(b) kesh-dump.sh sans `--single-transaction` : dump incohérent".into());
    }
    let mv = Regex::new(r"(?m)^mv\s+(\S+)\.tmp\s+(\S+)\s*$").unwrap();
    let renommages: Vec<(String, String, usize)> = mv
        .captures_iter(&dump)
        .map(|c| {
            (
                c[1].to_string(),
                c[2].to_string(),
                c.get(0).unwrap().start(),
            )
        })
        .collect();
    let (_, cible, pos_mv) = renommages
        .iter()
        .find(|(_, cible, _)| cible.ends_with(".gz"))
        .cloned()
        .unwrap_or_else(|| panic!("(b) kesh-dump.sh sans `mv <cible>.tmp <cible>.gz`"));
    for (source, but, _) in &renommages {
        if source != but {
            erreurs.push(format!(
                "(b) `mv {source}.tmp {but}` : la source n'est pas `<but>.tmp`"
            ));
        }
    }
    let e = regex::escape(&cible);
    match premier(&dump, &format!(r">\s*{e}\.tmp\b")) {
        Some(p) if p < pos_mv => {}
        _ => erreurs.push(format!(
            "(b) le dump n'est pas écrit dans `{cible}.tmp` avant le `mv`"
        )),
    }
    match premier(&dump, &format!(r"gzip\s+-t\s+{e}\.tmp\b")) {
        Some(p) if p < pos_mv => {}
        _ => erreurs.push(format!(
            "(b) `{cible}.tmp` n'est pas vérifié par `gzip -t` avant le `mv`"
        )),
    }
    if premier(&dump, &format!(r">\s*{e}(?:[^A-Za-z0-9_.-]|$)")).is_some() {
        erreurs.push(format!(
            "(b) `> {cible}` : écriture directe sur la cible, un dump raté remplacerait celui de la veille"
        ));
    }
    match premier(&dump, &format!(r"sha256sum\s+{e}\.tmp\b")) {
        Some(p) if p < pos_mv => {}
        _ => erreurs.push(format!(
            "(b) l'empreinte n'est pas calculée sur `{cible}.tmp` avant le renommage"
        )),
    }
    let option = Regex::new(r"--defaults-extra-file=(\S+)").unwrap();
    let montages = |texte: &str| -> Vec<(String, String)> {
        Regex::new(r#"-v\s+"?([^\s":]+):([^\s":]+)(?::ro)?"?(?:\s|$)"#)
            .unwrap()
            .captures_iter(texte)
            .map(|c| (c[1].to_string(), c[2].to_string()))
            .collect()
    };
    let f_dump: Vec<String> = option
        .captures_iter(&dump)
        .map(|c| c[1].to_string())
        .collect();
    assert_eq!(
        f_dump.len(),
        1,
        "(b) kesh-dump.sh : un `--defaults-extra-file` attendu"
    );
    let f_dump = &f_dump[0];
    let hote_dump = montages(&dump)
        .into_iter()
        .find(|(_, c)| c == f_dump)
        .map(|(h, _)| h)
        .unwrap_or_else(|| panic!("(b) kesh-dump.sh : `{f_dump}` n'est pas monté par `-v`"));

    // (c) — kesh-restore.sh
    let resolution = premier(&restore, r#"(?m)^SOURCE=\$\(cd "\$1" && pwd -P\)\s*$"#);
    match resolution {
        None => erreurs.push(
            "(c) `SOURCE=$(cd \"$1\" && pwd -P)` absent : le dossier donné n'est pas résolu en absolu".into(),
        ),
        Some(p) => {
            let autres_1 = Regex::new(r#"\$\{?1\b"#).unwrap().find_iter(&restore).count();
            if premier(&restore, r"(?m)^SOURCE=").is_some_and(|q| q < p) || autres_1 != 1 {
                erreurs.push(format!(
                    "(c) `$1` ou `SOURCE` employé hors de la résolution absolue ({autres_1} usage(s) de `$1`, 1 attendu : la résolution)"
                ));
            }
        }
    }
    if premier(&restore, r#"(?m)^\s*cd\s+"?\$SOURCE"?\s*$"#).is_some() {
        erreurs.push("(c) `cd \"$SOURCE\"` hors d'un sous-shell : la suite résoudrait les chemins depuis le dossier du dump".into());
    }
    if !restore.contains("gunzip -c \"$SOURCE/") {
        erreurs.push("(c) le rechargement ne lit pas `\"$SOURCE/…\"`".into());
    }
    let etapes = [
        ("`sha256sum -c`", r"sha256sum\s+-c\b"),
        ("`gzip -t`", r#"gzip\s+-t\s+"\$SOURCE/"#),
        (
            "`docker compose stop kesh-api`",
            r"docker compose stop kesh-api\b",
        ),
        ("dump de sécurité (`mariadb-dump`)", r"mariadb-dump\b"),
        ("`gunzip -c`", r"gunzip -c\b"),
    ];
    let mut precedente: Option<(&str, usize)> = None;
    for (nom, re) in etapes {
        match premier(&restore, re) {
            None => erreurs.push(format!("(c) kesh-restore.sh sans {nom}")),
            Some(p) => {
                if let Some((avant, q)) = precedente
                    && p < q
                {
                    erreurs.push(format!("(c) {nom} vient avant {avant}"));
                }
                precedente = Some((nom, p));
            }
        }
    }
    if let Some(p) = premier(&restore, r"mariadb-dump\b") {
        let avant = &restore[..p];
        let si = avant.rfind("\nif ").map(|i| i + 1);
        let bloquant = match si {
            None => true,
            Some(i) => {
                avant[i..].contains("\nthen")
                    || avant[i..].contains("; then")
                    || avant[i..].contains("\nfi")
            }
        };
        if bloquant {
            erreurs.push(
                "(c) le dump de sécurité n'est pas dans la condition d'un `if` : une base absente arrêterait le rechargement".into(),
            );
        }
    }
    let ligne_gunzip = restore
        .lines()
        .find(|l| l.contains("gunzip -c"))
        .unwrap_or("");
    let interactif = Regex::new(r"^-[a-zA-Z]*i[a-zA-Z]*$").unwrap();
    if !ligne_gunzip
        .split("docker run")
        .nth(1)
        .unwrap_or("")
        .split_whitespace()
        .take_while(|t| t.starts_with('-'))
        .any(|t| interactif.is_match(t))
    {
        erreurs.push(
            "(c) `gunzip -c … | docker run` sans `-i` : Docker ferme l'entrée, rien n'est rechargé"
                .into(),
        );
    }
    let f_rec: Vec<String> = option
        .captures_iter(&restore)
        .map(|c| c[1].to_string())
        .collect();
    if f_rec.is_empty() {
        erreurs.push("(c) kesh-restore.sh n'a aucun `--defaults-extra-file`".into());
    }
    let montes = montages(&restore);
    for f in &f_rec {
        if f == f_dump {
            erreurs.push(format!(
                "(c) kesh-restore.sh lit `{f}`, le fichier du dump, dont le compte ne peut que lire"
            ));
        }
        if !montes.iter().any(|(_, c)| c == f) {
            erreurs.push(format!(
                "(c) kesh-restore.sh : `{f}` n'est pas monté par `-v`"
            ));
        }
    }
    for (hote, conteneur) in &montes {
        if *hote == hote_dump {
            erreurs.push(format!(
                "(c) kesh-restore.sh monte `{hote}` (sur `{conteneur}`), le fichier d'identifiants du dump"
            ));
        }
    }

    // (d)
    for (nom, texte) in [("kesh-dump.sh", &dump), ("kesh-restore.sh", &restore)] {
        for (pos, args) in commandes_rm(texte) {
            for a in args.iter().filter(|a| !a.ends_with(".tmp")) {
                erreurs.push(format!(
                    "(d) {nom}:{} : `rm` de `{a}` — seuls des `.tmp` peuvent être supprimés",
                    ligne_de(texte, pos)
                ));
            }
        }
    }
    let base = cible.rsplit('/').next().unwrap();
    let rm_prose = Regex::new(&format!(
        r#"(?m)(?:^|[\s;&|({{`/'"]){}\s+[^;&|\n]*?{}(?:[^A-Za-z0-9_.-]|$)"#,
        "rm",
        regex::escape(base)
    ))
    .unwrap();
    for (p, label) in parts.iter().zip(manuel::LABELS_SYNOLOGY) {
        let p = manuel::desechapper(p);
        if let Some(m) = rm_prose.find(&p) {
            erreurs.push(format!(
                "(d) {label} : `{}` supprime le dump",
                m.as_str().trim()
            ));
        }
    }
    if normees.iter().any(|n| n.contains("supprimer le dump")) {
        erreurs.push("(d) « supprimer le dump » : le post-script d'avant".into());
    }
    if !normees[1].contains("Ne supprimez pas le dump") {
        erreurs.push("(d) phrase « Ne supprimez pas le dump » absente".into());
    }

    // (e)
    let hors = manuel::normaliser(&manuel::sans_commentaires(&manuel::hors_sections(
        &brut, &bornes,
    )));
    let mut couverts: Vec<(usize, usize)> = Vec::new();
    for fragment in HYPER_BACKUP_HORS_SYNOLOGY {
        let n = hors.matches(fragment).count();
        if n != 1 {
            erreurs.push(format!(
                "(e) fragment trouvé {n} fois (1 attendue — exemption morte ou ambiguë) : « {fragment} »"
            ));
        }
        couverts.extend(hors.match_indices(fragment).map(|(i, f)| (i, i + f.len())));
    }
    for m in Regex::new(r"(?i)hyper ?backup").unwrap().find_iter(&hors) {
        if !couverts
            .iter()
            .any(|(d, f)| m.start() >= *d && m.end() <= *f)
        {
            let d = hors[..m.start()]
                .char_indices()
                .rev()
                .nth(60)
                .map_or(0, |(i, _)| i);
            let f = (m.end() + 60).min(hors.len());
            let f = (f..=hors.len())
                .find(|i| hors.is_char_boundary(*i))
                .unwrap();
            erreurs.push(format!(
                "(e) « Hyper Backup » hors des sections Synology et hors liste : …{}…",
                &hors[d..f]
            ));
        }
    }

    // (f)
    for exige in [
        format!("raw.githubusercontent.com/guycorbaz/kesh/main/{SCRIPT_DUMP}"),
        format!("raw.githubusercontent.com/guycorbaz/kesh/main/{SCRIPT_RECHARGEMENT}"),
        format!("bash {DOSSIER_SYNOLOGY}/kesh-dump.sh"),
        // Rechargement depuis le dossier restauré, et secours depuis le dump de sécurité.
        format!("bash {DOSSIER_SYNOLOGY}/kesh-restore.sh <dossier dump restauré>"),
        format!("bash {DOSSIER_SYNOLOGY}/kesh-restore.sh {DOSSIER_SYNOLOGY}/avant-restauration/"),
        "Planificateur de tâches".to_string(),
    ] {
        if !normees[1].contains(&exige) {
            erreurs.push(format!("(f) sec:backup-dsm ne cite pas « {exige} »"));
        }
    }
    for listing in parts.iter().flat_map(|p| manuel::listings(p)) {
        if listing.contains("mariadb-dump") || listing.contains("gunzip -c") {
            erreurs.push(
                "(f) un `lstlisting` des sections recopie un script (mariadb-dump / gunzip -c) : le manuel doit citer scripts/synology/".into(),
            );
        }
    }
    let post = Regex::new(r"(?i)post-?script|→\s*pré-?script").unwrap();
    for (n, label) in normees.iter().zip(manuel::LABELS_SYNOLOGY) {
        if let Some(m) = post.find(n) {
            erreurs.push(format!(
                "(f) {label} : « {} » — Hyper Backup n'a pas de pré-script ni de post-script",
                m.as_str()
            ));
        }
    }
    echouer_si(erreurs, "G16 — la sauvegarde Synology passe par le dump");
}

/// Services, volumes et réseaux (`networks:` de tête) d'un compose.
fn compose_reseaux(source: &str) -> (BTreeSet<String>, BTreeSet<String>, BTreeSet<String>) {
    let (services, volumes) = services_et_volumes(source);
    let docs = YamlLoader::load_from_str(source).expect("compose : YAML lisible");
    let reseaux = match &docs[0]["networks"] {
        Yaml::Hash(h) => h.keys().filter_map(scalaire).collect(),
        _ => BTreeSet::new(),
    };
    assert!(!reseaux.is_empty(), "compose : aucun réseau de tête lu");
    (services, volumes, reseaux)
}

/// **G17** (#554) — le compose de développement démarre **sans `.env`** : les
/// défauts de `KESH_ADMIN_USERNAME`, `KESH_ADMIN_PASSWORD` et `KESH_JWT_SECRET`
/// du service `kesh` (`${NOM:-défaut}`) passent la configuration du binaire.
///
/// La règle n'est **pas recopiée** : `is_template_placeholder` est privée et
/// `Config` n'expose que `from_env`, si bien que le test lance le **vrai**
/// binaire (`common/binaire.rs`) avec ces valeurs et une base injoignable. Une
/// configuration acceptée va jusqu'à la connexion (« Base de données
/// indisponible ») ; un défaut refusé (`:-admin`, 5 caractères :
/// `WeakAdminPassword`) s'arrête sur « Erreur de configuration ». Aucune
/// longueur de secret n'est figée ici.
#[test]
fn le_compose_de_dev_demarre_sans_env() {
    let svc = service(&lire("docker-compose.dev.yml"), "kesh")
        .unwrap_or_else(|e| panic!("docker-compose.dev.yml, service `kesh` : {e:?}"));
    // Valeur effective sans `.env` ni variable d'environnement.
    let sans_env = |cle: &str| -> String {
        let brute = match svc.valeur(cle) {
            Some(ValeurEnv::Scalaire(v)) => v.clone(),
            autre => panic!("docker-compose.dev.yml : `{cle}` = {autre:?}"),
        };
        match interpolation(&brute) {
            Some(Interpolation::DefautSiVide(_, d) | Interpolation::DefautSiAbsente(_, d)) => {
                d.to_string()
            }
            Some(Interpolation::Simple(_)) => String::new(),
            Some(Interpolation::ObligatoireNonVide(_) | Interpolation::ObligatoireSiAbsente(_)) => {
                panic!("docker-compose.dev.yml : `{cle}` refusé par Compose sans `.env`")
            }
            None => brute,
        }
    };
    let env = [
        ("KESH_ADMIN_USERNAME", sans_env("KESH_ADMIN_USERNAME")),
        ("KESH_ADMIN_PASSWORD", sans_env("KESH_ADMIN_PASSWORD")),
        ("KESH_JWT_SECRET", sans_env("KESH_JWT_SECRET")),
    ];
    let mut args: Vec<(&str, &str)> = env.iter().map(|(k, v)| (*k, v.as_str())).collect();
    args.push(("DATABASE_URL", "mysql://kesh:x@kesh-15-14b.invalid/kesh"));
    args.push(("KESH_HOST", "127.0.0.1"));
    let sortie = binaire::lancer_binaire(&args);
    let texte = binaire::texte(&sortie);
    assert!(
        !texte.contains("Erreur de configuration"),
        "défauts du compose de dev {env:?} refusés par la configuration : {texte}"
    );
    // Assertion de montage : la configuration acceptée, le binaire a tenté la base.
    assert!(
        texte.contains("Base de données indisponible"),
        "le binaire doit être allé jusqu'à la connexion : {texte}"
    );
}
