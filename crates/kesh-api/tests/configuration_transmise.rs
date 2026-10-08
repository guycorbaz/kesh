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
//! - **(S) Auto-test** des extracteurs sur des sources synthétiques.
//!
//! # Ce qu'elle n'établit PAS (angles morts écrits)
//!
//! Les lectures internes aux dépendances (sans jeton dans le workspace :
//! `NO_COLOR` de `tracing-subscriber`, `TOKIO_WORKER_THREADS`, `TZ`…) et par
//! FFI ; les identifiants synthétisés par une macro procédurale ; les
//! littéraux d'octets ; `include!` et `#[path]` hors `crates/*/src` ; le
//! fichier d'un module hors ligne `#[cfg(test)] mod x;` (faux rouge possible).
//! `TMPDIR` (`std::env::temp_dir()`) est inventorié mais non compté dans
//! l'ensemble lu. `docker-compose.dev.yml` (pile de développement, non
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
// (L) Lectures — le test lit le code (Story 15-11b, C78)
// ---------------------------------------------------------------------------
//
// Le test ne reconnaît AUCUNE forme d'appel. Il relève chaque occurrence d'un
// jeton surveillé dans le flux de jetons du code de production et exige
// qu'elle figure, à son emplacement et sous sa forme, dans la liste fermée
// [`EMPLACEMENTS_AUTORISES`]. Faux rouge possible (une ligne à ajouter) ; faux
// vert impossible pour le code du workspace.

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
            self.exclues.push((debut, fin));
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
                let jeton = id.to_string();
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
    let y = service_kesh_api(&y_source).unwrap_or_else(|e| panic!("docker-compose.yml : {e:?}"));
    let p =
        service_kesh_api(&p_source).unwrap_or_else(|e| panic!("docker-compose.prod.yml : {e:?}"));
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
        EMPLACEMENTS_AUTORISES.len(),
        22,
        "EMPLACEMENTS_AUTORISES : 22 entrées attendues (AC3 de la 15-11b)"
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
