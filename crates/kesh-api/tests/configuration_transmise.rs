//! Garde de la configuration transmise — Story 15-11a (#550, AC 8).
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
//! - **(T) Transmission** — chaque variable de [`LUES`] est transmise par les
//!   deux compose, sauf les [`EXCEPTIONS`] (dont la raison est contrôlée) ;
//!   toute clé transmise est dans [`LUES`] ; pas d'`env_file` ; structure
//!   gardée (`image:` de `docker-compose.yml`, montages des deux compose).
//! - **(V) Valeurs** — chaque valeur transmise a une forme admise ; listes
//!   fermées [`VIDE_SIGNIFIANT`], [`SANS_DEFAUT`] et [`AJOUTS`].
//! - **(E) `.env.example`** — chaque ligne d'affectation nomme une variable
//!   connue, chaque variable lue y a sa ligne (sauf `KESH_TEST_MODE`).
//! - **(F) Fantômes** — tout jeton `KESH_…` du gabarit, des compose, des
//!   catalogues i18n et du manuel français nomme une variable connue.
//! - **(S) Auto-test** des extracteurs sur des sources synthétiques.
//!
//! # Ce qu'elle n'établit PAS (angles morts écrits)
//!
//! ⚠️ **Elle ne lit pas le code Rust.** [`LUES`] est écrite à la main : une
//! variable ajoutée au code **sans** être ajoutée à [`LUES`] n'est pas vue.
//! Propriétaire : Story 15-11b, qui remplace [`LUES`] par la lecture du code.
//! Jusque-là, la revue de toute story qui ajoute une lecture d'environnement
//! vérifie [`LUES`] à la main. Les lectures faites par des dépendances
//! (`sqlx`, `lettre`…) et `TMPDIR` (`std::env::temp_dir()`) sont hors
//! inventaire. `docker-compose.dev.yml` (pile de développement, non
//! distribuée) n'est pas contraint.
//!
//! L'analyseur YAML (`yaml-rust2`) ne connaît pas les clés de fusion
//! (`<<: *ancre`) : une forme non reconnue fait **rougir**, jamais passer. La
//! validité au sens de Compose (clé inconnue, clé en double) est contrôlée par
//! l'étape `docker compose config -q` du job `docker-build` de la CI.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use yaml_rust2::{Yaml, YamlLoader};

// ---------------------------------------------------------------------------
// Listes fermées
// ---------------------------------------------------------------------------

/// Les **41 variables lues** par le code de production du workspace, triées
/// dans l'ordre des octets (`FILES_` avant `FILE_` : `S` < `_`).
///
/// Reproduite par (40 noms, plus `RUST_LOG`, lu par `logging.rs` via
/// `EnvFilter::DEFAULT_ENV` de `tracing-subscriber`) :
///
/// ```sh
/// grep -rhoE '(env::var|opt_trimmed_env|parse_strict_bool|env_flag_enabled)\("[A-Z][A-Z0-9_]*"' crates/*/src \
///   | grep -oE '"[A-Z][A-Z0-9_]*"' | tr -d '"' | LC_ALL=C sort -u
/// ```
///
/// ⚠️ Liste écrite à la main jusqu'à la Story 15-11b, qui la **remplace** par
/// la lecture du code. Toute story qui ajoute une lecture d'environnement doit
/// l'ajouter ici — et alors ce test exige qu'elle soit transmise par les deux
/// compose et documentée dans `.env.example`.
const LUES: &[&str] = &[
    "DATABASE_URL",
    "KESH_ADMIN_BACKUP_DIR",
    "KESH_ADMIN_EXPORT_INMEM_MB",
    "KESH_ADMIN_IMPORT_MAX_MB",
    "KESH_ADMIN_PASSWORD",
    "KESH_ADMIN_USERNAME",
    "KESH_BANK_IMPORT_MAX_MB",
    "KESH_COOKIE_SECURE",
    "KESH_DOCUMENTS_DIR",
    "KESH_FEATURE_FORGOT_PASSWORD",
    "KESH_HOST",
    "KESH_INBOX_DIR",
    "KESH_INBOX_MAX_FILES_PER_RUN",
    "KESH_INBOX_MAX_FILE_BYTES",
    "KESH_INBOX_MAX_PDF_PAGES",
    "KESH_JWT_EXPIRY_MINUTES",
    "KESH_JWT_SECRET",
    "KESH_LANG",
    "KESH_LOCALES_DIR",
    "KESH_LOG_FILE_FORMAT",
    "KESH_LOG_FILE_MAX_FILES",
    "KESH_LOG_FILE_PATH",
    "KESH_LOG_FILE_ROTATION",
    "KESH_PASSWORD_MIN_LENGTH",
    "KESH_PORT",
    "KESH_PRODUCTION_RESET",
    "KESH_PUBLIC_BASE_URL",
    "KESH_RATE_LIMIT_BLOCK_MINUTES",
    "KESH_RATE_LIMIT_MAX_ATTEMPTS",
    "KESH_RATE_LIMIT_WINDOW_MINUTES",
    "KESH_REFRESH_INACTIVITY_MINUTES",
    "KESH_REFRESH_TOKEN_MAX_LIFETIME_DAYS",
    "KESH_SMTP_FROM",
    "KESH_SMTP_HOST",
    "KESH_SMTP_PASSWORD",
    "KESH_SMTP_PORT",
    "KESH_SMTP_TLS",
    "KESH_SMTP_USER",
    "KESH_STATIC_DIR",
    "KESH_TEST_MODE",
    "RUST_LOG",
];

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
    // service MariaDB : la ligne `DATABASE_URL` de `.env` y est ignorée.
    ("DATABASE_URL", Compose::Y, "${MARIADB_"),
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

/// Les trois montages de `kesh-api` : (cible dans le conteneur, source exigée
/// dans `docker-compose.yml` — préfixe —, source exigée dans
/// `docker-compose.prod.yml` — exacte).
const MONTAGES: &[(&str, &str, &str)] = &[
    ("/var/log/kesh", "${KESH_LOG_HOST_DIR:-", "./log"),
    ("/data/inbox", "${KESH_INBOX_HOST_DIR:-", "./inbox"),
    (
        "/data/documents",
        "${KESH_DOCUMENTS_HOST_DIR:-",
        "./documents",
    ),
];

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

fn lues() -> BTreeSet<&'static str> {
    LUES.iter().copied().collect()
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

/// Le service `kesh-api` d'un compose, tel que ce test le lit.
#[derive(Debug, Default)]
struct Service {
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

/// Analyse un compose et rend son service `kesh-api`, ou les raisons pour
/// lesquelles il ne se lit pas (forme non reconnue : rouge, jamais ignorée).
fn service_kesh_api(source: &str) -> Result<Service, Vec<String>> {
    let docs =
        YamlLoader::load_from_str(source).map_err(|e| vec![format!("YAML invalide : {e}")])?;
    let doc = docs
        .first()
        .ok_or_else(|| vec!["document YAML vide".to_string()])?;
    let svc = &doc["services"]["kesh-api"];
    let Some(hash) = svc.as_hash() else {
        return Err(vec!["service `kesh-api` introuvable".to_string()]);
    };
    let mut erreurs = Vec::new();
    let mut out = Service::default();
    for (k, v) in hash {
        match k.as_str() {
            Some("env_file") => out.env_file = true,
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

/// Les sources des trois montages de [`MONTAGES`] dans un service, par cible.
/// Une entrée en forme longue, ou dont la cible n'est pas l'une des trois,
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
                "exception `{nom}` : la variable n'est pas dans LUES (entrée inutilisée)"
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
                    match compose {
                        Compose::Y if !source.starts_with(prefixe_y) => erreurs.push(format!(
                            "{f} : le montage de `{cible}` doit avoir pour source `{prefixe_y}…}}`, trouvé `{source}`"
                        )),
                        Compose::P if source != exacte_p => erreurs.push(format!(
                            "{f} : le montage de `{cible}` doit rester `{exacte_p}` (montage fixe), trouvé `{source}` — \
                             le rendre configurable exige la procédure de déplacement des données de l'issue #558 (choix C83, AC 4)"
                        )),
                        _ => {}
                    }
                }
            }
        }
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
    /// `${NOM:?message}` ou `${NOM?message}`
    Obligatoire(&'a str),
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
    } else if reste.starts_with(":?") || reste.starts_with('?') {
        Some(Interpolation::Obligatoire(nom))
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
                | Interpolation::Obligatoire(n) => n,
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
// Tests sur le dépôt
// ---------------------------------------------------------------------------

fn services() -> (Service, Service, String) {
    let y_source = lire(Compose::Y.fichier());
    let p_source = lire(Compose::P.fichier());
    let y = service_kesh_api(&y_source).unwrap_or_else(|e| panic!("docker-compose.yml : {e:?}"));
    let p =
        service_kesh_api(&p_source).unwrap_or_else(|e| panic!("docker-compose.prod.yml : {e:?}"));
    (y, p, y_source)
}

/// Garde contre le test muet : une `LUES` vidée ou tronquée rendrait tous les
/// contrôles verts sans rien vérifier.
#[test]
fn garde_liste_lues() {
    assert!(
        LUES.len() >= 41,
        "LUES compte {} noms, au moins 41 attendus",
        LUES.len()
    );
    for nom in [
        "DATABASE_URL",
        "KESH_JWT_SECRET",
        "KESH_SMTP_HOST",
        "KESH_PRODUCTION_RESET",
        "RUST_LOG",
    ] {
        assert!(LUES.contains(&nom), "LUES doit contenir `{nom}`");
    }
    let mut triee = LUES.to_vec();
    triee.sort_unstable();
    triee.dedup();
    assert_eq!(triee, LUES, "LUES doit être triée et sans doublon");
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

#[test]
fn fantomes() {
    echouer_si(controle_fantomes(&corpus_texte(), &lues()), "(F) fantômes");
}

// ---------------------------------------------------------------------------
// (S) Auto-test des extracteurs sur sources synthétiques
// ---------------------------------------------------------------------------

fn svc(env_yaml: &str) -> Service {
    let source = format!("services:\n  kesh-api:\n    environment:\n{env_yaml}");
    service_kesh_api(&source).expect("YAML synthétique valide")
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
    let s = service_kesh_api(longue).expect("YAML valide");
    assert!(
        sources_montages(&s).is_err(),
        "une forme longue doit rougir"
    );
}
