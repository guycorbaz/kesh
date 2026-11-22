//! Identifiants des API de prestataires, lus dans l'environnement — socle
//! commun S1 de la phase 2 des Epics 38, 39 et 40 (Story 38-12a, AC-1 à AC-3).
//!
//! Comme le mot de passe SMTP, les identifiants d'une application du
//! prestataire se posent par variables d'environnement, **une série par compte
//! du prestataire**, rattachée à son libellé (`provider_account` de 38-11b) :
//! rien en base, rien dans la sauvegarde. Pour un prestataire `<P>` (`PAYPAL`,
//! puis `WISE`, `STRIPE`) :
//!
//! - `KESH_<P>_<n>_ACCOUNT` — le libellé du compte, `n` de 1 à 9 ;
//! - `KESH_<P>_<n>_<CHAMP>` — les champs propres au prestataire
//!   ([`ProviderSpec::fields`] ; PayPal : `CLIENT_ID`, `SECRET`) ;
//! - `KESH_<P>_BASE_URL` — l'hôte de l'API ([`base_url_allowed`]) ;
//! - `KESH_<P>_PARALLEL` — requêtes simultanées, 1 à 6 (1 par défaut).
//!
//! Tout est lu **une fois**, au démarrage ([`crate::config::Config::from_env`]),
//! et validé sans condition ([`read`]) : une série partielle, un libellé en
//! double, un numéro hors `1..=9`, une valeur de gabarit `<…>`, une URL hors de
//! la règle ou un parallélisme illisible **refusent le démarrage**, d'un message
//! qui nomme la variable et jamais sa valeur. Une série entièrement vide est
//! absente — l'état de toute installation qui ne s'en sert pas.
//!
//! Les champs d'une série sont **privés** et lus par un accesseur unique
//! ([`ProviderSeries::field`]) ; `Debug` les masque.

use std::collections::BTreeMap;

use crate::config::ConfigError;

/// Premier numéro de série.
pub const MIN_SERIES: u8 = 1;
/// Dernier numéro de série : `KESH_<P>_10_…` est refusé.
pub const MAX_SERIES: u8 = 9;
/// Nombre de séries que portent les fichiers distribués (`docker-compose.yml`,
/// `docker-compose.prod.yml`, `.env.example`) ; au-delà, l'administrateur ajoute
/// les lignes.
pub const SERIES_IN_TEMPLATES: u8 = 3;
/// Parallélisme maximal d'un prestataire.
pub const MAX_PARALLEL: usize = 6;

/// Le champ du libellé, commun à tous les prestataires.
pub const ACCOUNT_FIELD: &str = "ACCOUNT";
/// Suffixe de la variable de l'hôte de l'API.
pub const BASE_URL_SUFFIX: &str = "BASE_URL";
/// Suffixe de la variable du parallélisme.
pub const PARALLEL_SUFFIX: &str = "PARALLEL";

/// Ce qui distingue un prestataire dans l'environnement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderSpec {
    /// Segment des variables (`PAYPAL` → `KESH_PAYPAL_…`).
    pub key: &'static str,
    /// Nom du prestataire dans l'API et les journaux (`paypal`, comme
    /// `bank_accounts.provider`).
    pub provider: &'static str,
    /// Champs propres au prestataire, outre [`ACCOUNT_FIELD`] (tous requis
    /// dans une série).
    pub fields: &'static [&'static str],
    /// Champs **facultatifs** d'une série (Wise : `PROFILE_ID`) : leur
    /// absence ne rend pas la série partielle ; seuls, sans les champs requis,
    /// ils la rendent partielle. Vide pour PayPal.
    pub optional_fields: &'static [&'static str],
    /// Hôte par défaut de l'API.
    pub default_base_url: &'static str,
}

/// Champ PayPal : l'identifiant de l'application (`KESH_PAYPAL_<n>_CLIENT_ID`).
/// Le client le lit par ce nom, la spécification le déclare par ce nom : une
/// seule écriture (revue P1, F6).
pub const PAYPAL_CLIENT_ID: &str = "CLIENT_ID";
/// Champ PayPal : le secret de l'application (`KESH_PAYPAL_<n>_SECRET`).
pub const PAYPAL_SECRET: &str = "SECRET";

/// PayPal (Story 38-12a) : une application REST par compte PayPal, avec la
/// seule fonction « Transaction Search ».
pub const PAYPAL: ProviderSpec = ProviderSpec {
    key: "PAYPAL",
    provider: "paypal",
    fields: &[PAYPAL_CLIENT_ID, PAYPAL_SECRET],
    optional_fields: &[],
    default_base_url: "https://api-m.paypal.com",
};

/// Les prestataires lus au démarrage (Wise et Stripe s'y ajoutent).
pub const ALL_PROVIDERS: &[ProviderSpec] = &[PAYPAL];

impl ProviderSpec {
    /// `KESH_<P>_<n>_<champ>`.
    pub fn series_var(&self, n: u8, field: &str) -> String {
        format!("KESH_{}_{n}_{field}", self.key)
    }

    /// `KESH_<P>_<suffixe>` (variables sans numéro).
    pub fn global_var(&self, suffix: &str) -> String {
        format!("KESH_{}_{suffix}", self.key)
    }

    /// Préfixe des variables du prestataire, `KESH_<P>_`.
    fn prefix(&self) -> String {
        format!("KESH_{}_", self.key)
    }

    /// Les champs requis d'une série, le libellé en tête.
    fn required_fields(&self) -> impl Iterator<Item = &'static str> + '_ {
        std::iter::once(ACCOUNT_FIELD).chain(self.fields.iter().copied())
    }

    /// Tous les champs d'une série : requis, puis facultatifs.
    pub fn all_fields(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.required_fields()
            .chain(self.optional_fields.iter().copied())
    }

    /// Lit le champ **requis** `name` d'une série, pour un client de
    /// prestataire — le point de lecture commun (revue P1, F6). Un nom que la
    /// spécification ne déclare pas parmi ses champs requis est une faute de
    /// Kesh, non de configuration : `AppError::Internal`, journalisée (jamais
    /// la valeur) ; un champ déclaré mais absent de la série rend
    /// `PROVIDER_API_NOT_CONFIGURED` (impossible après la validation du
    /// démarrage, gardé par défense).
    pub fn require_field(
        &self,
        series: &ProviderSeries,
        name: &str,
    ) -> Result<String, crate::errors::AppError> {
        if !self.fields.contains(&name) {
            tracing::error!(
                provider = self.provider,
                champ = name,
                "champ de série non déclaré par le prestataire"
            );
            return Err(crate::errors::AppError::Internal(
                "champ de série non déclaré".into(),
            ));
        }
        series.field(name).map(str::to_string).ok_or_else(|| {
            crate::errors::AppError::ProviderApiNotConfigured {
                provider: self.provider,
                account: series.account.clone(),
            }
        })
    }
}

/// Une série complète : le libellé et les champs secrets.
#[derive(Clone)]
pub struct ProviderSeries {
    /// Numéro de la série (1 à 9).
    pub n: u8,
    /// Libellé du compte du prestataire (`provider_account`).
    pub account: String,
    fields: BTreeMap<&'static str, String>,
}

impl ProviderSeries {
    /// Valeur d'un champ propre au prestataire (`CLIENT_ID`, `SECRET`…), point
    /// d'accès unique. Ne jamais la journaliser ni la renvoyer.
    pub fn field(&self, name: &str) -> Option<&str> {
        self.fields.get(name).map(String::as_str)
    }

    /// Construit une série — **tests seulement**.
    #[doc(hidden)]
    pub fn for_tests(n: u8, account: &str, fields: &[(&'static str, &str)]) -> Self {
        Self {
            n,
            account: account.to_string(),
            fields: fields.iter().map(|(k, v)| (*k, v.to_string())).collect(),
        }
    }
}

impl std::fmt::Debug for ProviderSeries {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let masked: BTreeMap<&str, &str> = self.fields.keys().map(|k| (*k, "***")).collect();
        f.debug_struct("ProviderSeries")
            .field("n", &self.n)
            .field("account", &self.account)
            .field("fields", &masked)
            .finish()
    }
}

/// La configuration d'un prestataire : hôte, parallélisme, séries.
#[derive(Clone, Debug)]
pub struct ProviderApiConfig {
    /// Le prestataire.
    pub spec: ProviderSpec,
    /// Hôte de l'API, sans `/` final.
    pub base_url: String,
    /// Requêtes simultanées (1 à [`MAX_PARALLEL`]).
    pub parallel: usize,
    series: Vec<ProviderSeries>,
}

impl ProviderApiConfig {
    /// La série d'un libellé (sans casse), `None` s'il n'en a pas : le compte
    /// reste sans téléchargement, le CSV continue.
    pub fn series_for(&self, account: &str) -> Option<&ProviderSeries> {
        let wanted = account.trim().to_lowercase();
        self.series
            .iter()
            .find(|s| s.account.to_lowercase() == wanted)
    }

    /// Nombre de séries configurées.
    pub fn series_count(&self) -> usize {
        self.series.len()
    }

    /// Configuration sans série (installation qui n'utilise pas l'API).
    pub fn empty(spec: ProviderSpec) -> Self {
        Self {
            spec,
            base_url: spec.default_base_url.to_string(),
            parallel: 1,
            series: Vec::new(),
        }
    }

    /// Configuration construite — **tests seulement** (serveur simulé).
    #[doc(hidden)]
    pub fn for_tests(
        spec: ProviderSpec,
        base_url: &str,
        parallel: usize,
        series: Vec<ProviderSeries>,
    ) -> Self {
        Self {
            spec,
            base_url: base_url.trim_end_matches('/').to_string(),
            parallel,
            series,
        }
    }
}

/// Mode d'un prestataire vu par l'écran — Story 38-12b (AC-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderApiMode {
    /// L'hôte par défaut du prestataire (données réelles).
    Live,
    /// Un hôte d'essai (un segment du nom d'hôte vaut `sandbox`).
    Sandbox,
}

/// Ce que la liste des comptes bancaires dit du téléchargement d'un compte de
/// prestataire — Story 38-12b (AC-1, socle S7) : `{ configured, mode? }`,
/// **jamais un secret** ni l'hôte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct ProviderApiStatus {
    /// Une série d'identifiants porte le libellé du compte.
    pub configured: bool,
    /// Le mode de l'hôte ([`ProviderApiConfig::mode`]) ; absent hors d'une
    /// série configurée et pour un hôte ni réel ni d'essai (serveur simulé).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<ProviderApiMode>,
}

impl ProviderApiConfig {
    /// Le mode de l'hôte : `live` pour l'hôte par défaut du prestataire,
    /// `sandbox` quand un segment du nom d'hôte est `sandbox`
    /// (`api-m.sandbox.paypal.com`), `None` sinon (serveur simulé du bouclage).
    /// Générique : aucun nom de prestataire.
    pub fn mode(&self) -> Option<ProviderApiMode> {
        if self.base_url == self.spec.default_base_url.trim_end_matches('/') {
            return Some(ProviderApiMode::Live);
        }
        let authority = self
            .base_url
            .split_once("://")
            .map_or(self.base_url.as_str(), |(_, rest)| rest);
        let host = authority
            .split(['/', ':'])
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        host.split('.')
            .any(|label| label == "sandbox")
            .then_some(ProviderApiMode::Sandbox)
    }

    /// L'état du téléchargement pour le libellé `account` (même règle que la
    /// route : [`Self::series_for`]).
    pub fn status_for(&self, account: &str) -> ProviderApiStatus {
        let configured = self.series_for(account).is_some();
        ProviderApiStatus {
            configured,
            mode: if configured { self.mode() } else { None },
        }
    }
}

/// Les configurations de tous les prestataires de [`ALL_PROVIDERS`].
#[derive(Clone, Debug, Default)]
pub struct ProvidersConfig {
    configs: Vec<ProviderApiConfig>,
}

impl ProvidersConfig {
    /// La configuration d'un prestataire ; vide s'il n'est pas lu.
    pub fn get(&self, spec: ProviderSpec) -> ProviderApiConfig {
        self.configs
            .iter()
            .find(|c| c.spec.key == spec.key)
            .cloned()
            .unwrap_or_else(|| ProviderApiConfig::empty(spec))
    }

    /// Remplace (ou ajoute) la configuration d'un prestataire — tests.
    #[doc(hidden)]
    pub fn with(mut self, config: ProviderApiConfig) -> Self {
        self.configs.retain(|c| c.spec.key != config.spec.key);
        self.configs.push(config);
        self
    }

    /// L'état du téléchargement d'un compte du prestataire `provider` (valeur
    /// de `bank_accounts.provider`) au libellé `account` ; un prestataire que
    /// cette version ne télécharge pas rend `configured: false` — Story 38-12b
    /// (AC-1).
    pub fn status_for(&self, provider: &str, account: &str) -> ProviderApiStatus {
        ALL_PROVIDERS
            .iter()
            .find(|spec| spec.provider == provider)
            .map_or(
                ProviderApiStatus {
                    configured: false,
                    mode: None,
                },
                |spec| self.get(*spec).status_for(account),
            )
    }

    /// Lit tous les prestataires dans l'environnement du processus.
    pub fn from_env() -> Result<Self, ConfigError> {
        let names: Vec<String> = std::env::vars_os()
            .filter_map(|(k, _)| k.into_string().ok())
            .collect();
        let mut configs = Vec::new();
        for spec in ALL_PROVIDERS {
            configs.push(read(*spec, crate::config::env_nonempty, &names)?);
        }
        Ok(Self { configs })
    }
}

/// Règle de l'hôte d'une API (socle S5) : `https://…`, ou `http://` vers le
/// seul bouclage (`127.0.0.1`, `localhost`, port admis) pour le serveur simulé
/// des tests. Ni identifiants dans l'URL, ni chemin autre que `/`.
pub fn base_url_allowed(url: &str) -> bool {
    let (scheme, rest) = match url.split_once("://") {
        Some(p) => p,
        None => return false,
    };
    let authority = rest.trim_end_matches('/');
    if authority.is_empty()
        || authority.contains(['/', '@', '?', '#', ' '])
        || !authority
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-.:[]".contains(&b))
    {
        return false;
    }
    match scheme {
        "https" => true,
        "http" => {
            let host = authority
                .rsplit_once(':')
                .filter(|(_, port)| port.bytes().all(|b| b.is_ascii_digit()))
                .map_or(authority, |(h, _)| h);
            host == "127.0.0.1" || host == "localhost"
        }
        _ => false,
    }
}

/// Lit et valide la configuration d'un prestataire. `lookup` rend une variable
/// non vide (trim, vide = absente, comme `env_nonempty`) ; `names` est la liste
/// des noms présents dans l'environnement, parcourue pour repérer un numéro de
/// série hors `1..=9`. Fonction pure, testée sans toucher à l'environnement.
pub fn read(
    spec: ProviderSpec,
    lookup: impl Fn(&str) -> Option<String>,
    names: &[String],
) -> Result<ProviderApiConfig, ConfigError> {
    let invalid =
        |var: String, reason: &'static str| ConfigError::InvalidProviderVar { var, reason };

    // Numéros hors `1..=9`, repérés sur les noms présents (valeur non vide).
    let prefix = spec.prefix();
    for name in names {
        let Some(rest) = name.strip_prefix(&prefix) else {
            continue;
        };
        let Some((num, field)) = rest.split_once('_') else {
            continue;
        };
        if num.is_empty() || !num.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let known_field = spec.all_fields().any(|f| f == field);
        let in_range = num
            .parse::<u8>()
            .is_ok_and(|n| (MIN_SERIES..=MAX_SERIES).contains(&n))
            && !num.starts_with('0');
        if (!in_range || !known_field) && lookup(name).is_some() {
            return Err(invalid(
                name.clone(),
                if in_range {
                    "champ inconnu"
                } else {
                    "numéro de série hors de 1 à 9"
                },
            ));
        }
    }

    let mut series: Vec<ProviderSeries> = Vec::new();
    for n in MIN_SERIES..=MAX_SERIES {
        let values: Vec<(&'static str, Option<String>)> = spec
            .all_fields()
            .map(|f| (f, lookup(&spec.series_var(n, f))))
            .collect();
        let present = values.iter().filter(|(_, v)| v.is_some()).count();
        if present == 0 {
            continue;
        }
        let required: Vec<&'static str> = spec.required_fields().collect();
        if let Some((missing, _)) = values
            .iter()
            .find(|(f, v)| v.is_none() && required.contains(f))
        {
            return Err(invalid(
                spec.series_var(n, missing),
                "série incomplète : variable absente ou vide",
            ));
        }
        for (field, value) in &values {
            if value
                .as_deref()
                .is_some_and(crate::config::is_template_placeholder)
            {
                return Err(invalid(spec.series_var(n, field), "valeur de gabarit <…>"));
            }
        }
        let mut fields = BTreeMap::new();
        let mut account = String::new();
        for (field, value) in values {
            // Un champ facultatif absent n'entre pas ; les requis sont présents.
            let Some(value) = value else { continue };
            if field == ACCOUNT_FIELD {
                account = value;
            } else {
                fields.insert(field, value);
            }
        }
        if series
            .iter()
            .any(|s| s.account.to_lowercase() == account.to_lowercase())
        {
            return Err(invalid(
                spec.series_var(n, ACCOUNT_FIELD),
                "libellé déjà porté par une autre série",
            ));
        }
        series.push(ProviderSeries { n, account, fields });
    }

    let base_var = spec.global_var(BASE_URL_SUFFIX);
    let base_url = match lookup(&base_var) {
        None => spec.default_base_url.to_string(),
        Some(v) if crate::config::is_template_placeholder(&v) => {
            return Err(invalid(base_var, "valeur de gabarit <…>"));
        }
        Some(v) if base_url_allowed(&v) => v.trim_end_matches('/').to_string(),
        Some(_) => {
            return Err(invalid(
                base_var,
                "URL refusée : https, ou http vers 127.0.0.1 ou localhost seulement",
            ));
        }
    };

    let parallel_var = spec.global_var(PARALLEL_SUFFIX);
    let parallel = match lookup(&parallel_var) {
        None => 1,
        Some(v) => match v.parse::<usize>() {
            Ok(p) if (1..=MAX_PARALLEL).contains(&p) && v.bytes().all(|b| b.is_ascii_digit()) => p,
            _ => return Err(invalid(parallel_var, "entier de 1 à 6 attendu")),
        },
    };

    Ok(ProviderApiConfig {
        spec,
        base_url,
        parallel,
        series,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env(pairs: &[(&str, &str)]) -> (HashMap<String, String>, Vec<String>) {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let names = map.keys().cloned().collect();
        (map, names)
    }

    fn lire(pairs: &[(&str, &str)]) -> Result<ProviderApiConfig, ConfigError> {
        let (map, names) = env(pairs);
        read(
            PAYPAL,
            |k| {
                map.get(k)
                    .map(|v| v.trim().to_string())
                    .filter(|v| !v.is_empty())
            },
            &names,
        )
    }

    fn var_refusee(pairs: &[(&str, &str)]) -> String {
        match lire(pairs) {
            Err(ConfigError::InvalidProviderVar { var, .. }) => var,
            other => panic!("refus attendu, obtenu {other:?}"),
        }
    }

    const SERIE_1: [(&str, &str); 3] = [
        ("KESH_PAYPAL_1_ACCOUNT", "Boutique"),
        ("KESH_PAYPAL_1_CLIENT_ID", "id-un"),
        ("KESH_PAYPAL_1_SECRET", "secret-un"),
    ];

    #[test]
    fn rien_de_pose_vaut_aucune_serie_et_les_defauts() {
        let c = lire(&[]).unwrap();
        assert_eq!(c.series_count(), 0);
        assert_eq!(c.base_url, "https://api-m.paypal.com");
        assert_eq!(c.parallel, 1);
        // Une série entièrement vide (lignes des compose en `${NOM:-}`) est absente.
        let c = lire(&[
            ("KESH_PAYPAL_1_ACCOUNT", ""),
            ("KESH_PAYPAL_1_CLIENT_ID", "  "),
            ("KESH_PAYPAL_1_SECRET", ""),
            ("KESH_PAYPAL_BASE_URL", ""),
            ("KESH_PAYPAL_PARALLEL", ""),
        ])
        .unwrap();
        assert_eq!(c.series_count(), 0);
        assert_eq!(c.parallel, 1);
    }

    #[test]
    fn une_serie_complete_se_lit_et_se_retrouve_par_son_libelle() {
        let mut pairs = SERIE_1.to_vec();
        pairs.extend([
            ("KESH_PAYPAL_3_ACCOUNT", "Hébergement"),
            ("KESH_PAYPAL_3_CLIENT_ID", "id-trois"),
            ("KESH_PAYPAL_3_SECRET", "secret-trois"),
        ]);
        let c = lire(&pairs).unwrap();
        assert_eq!(c.series_count(), 2);
        let s = c.series_for(" boutique ").expect("sans casse");
        assert_eq!((s.n, s.field("CLIENT_ID")), (1, Some("id-un")));
        assert_eq!(c.series_for("HÉBERGEMENT").unwrap().n, 3);
        assert!(c.series_for("Autre").is_none());
    }

    #[test]
    fn une_serie_partielle_refuse_et_nomme_la_variable_manquante() {
        let v = var_refusee(&[
            ("KESH_PAYPAL_2_ACCOUNT", "Boutique"),
            ("KESH_PAYPAL_2_SECRET", "s"),
        ]);
        assert_eq!(v, "KESH_PAYPAL_2_CLIENT_ID");
    }

    #[test]
    fn libelle_en_double_numero_hors_plage_et_champ_inconnu() {
        let mut pairs = SERIE_1.to_vec();
        pairs.extend([
            ("KESH_PAYPAL_2_ACCOUNT", "BOUTIQUE"),
            ("KESH_PAYPAL_2_CLIENT_ID", "x"),
            ("KESH_PAYPAL_2_SECRET", "y"),
        ]);
        assert_eq!(var_refusee(&pairs), "KESH_PAYPAL_2_ACCOUNT");
        assert_eq!(
            var_refusee(&[("KESH_PAYPAL_10_ACCOUNT", "B")]),
            "KESH_PAYPAL_10_ACCOUNT"
        );
        assert_eq!(
            var_refusee(&[("KESH_PAYPAL_0_SECRET", "B")]),
            "KESH_PAYPAL_0_SECRET"
        );
        assert_eq!(
            var_refusee(&[("KESH_PAYPAL_01_SECRET", "B")]),
            "KESH_PAYPAL_01_SECRET"
        );
        assert_eq!(
            var_refusee(&[("KESH_PAYPAL_1_TOKEN", "B")]),
            "KESH_PAYPAL_1_TOKEN"
        );
        // Une variable hors plage mais vide ne compte pas.
        assert!(lire(&[("KESH_PAYPAL_10_ACCOUNT", " ")]).is_ok());
    }

    #[test]
    fn gabarit_url_et_parallelisme() {
        let mut pairs = SERIE_1.to_vec();
        pairs[2] = ("KESH_PAYPAL_1_SECRET", "<secret>");
        assert_eq!(var_refusee(&pairs), "KESH_PAYPAL_1_SECRET");

        for bad in [
            "http://api-m.paypal.com",
            "ftp://x",
            "https://u:p@api-m.paypal.com",
            "https://api-m.paypal.com/v1",
            "<url>",
            "http://127.0.0.2:8080",
        ] {
            assert_eq!(
                var_refusee(&[("KESH_PAYPAL_BASE_URL", bad)]),
                "KESH_PAYPAL_BASE_URL",
                "{bad}"
            );
        }
        for ok in [
            "https://api-m.sandbox.paypal.com",
            "https://api-m.sandbox.paypal.com/",
            "http://127.0.0.1:43123",
            "http://localhost:8080",
        ] {
            let c = lire(&[("KESH_PAYPAL_BASE_URL", ok)]).unwrap();
            assert_eq!(c.base_url, ok.trim_end_matches('/'));
        }

        for bad in ["0", "7", "deux", "-1", "+2", "1.5"] {
            assert_eq!(
                var_refusee(&[("KESH_PAYPAL_PARALLEL", bad)]),
                "KESH_PAYPAL_PARALLEL",
                "{bad}"
            );
        }
        assert_eq!(lire(&[("KESH_PAYPAL_PARALLEL", "6")]).unwrap().parallel, 6);
    }

    /// Coordinateur (validation 39-12-v2 P1) : un champ facultatif absent ne
    /// rend pas la série partielle ; seul, il la rend partielle.
    #[test]
    fn champ_facultatif() {
        const WISE_FICTIF: ProviderSpec = ProviderSpec {
            key: "WISEFICTIF",
            provider: "wisefictif",
            fields: &["TOKEN"],
            optional_fields: &["PROFILE_ID"],
            default_base_url: "https://api.example.invalid",
        };
        let lire_w = |pairs: &[(&str, &str)]| {
            let (map, names) = env(pairs);
            read(WISE_FICTIF, |k| map.get(k).cloned(), &names)
        };
        let c = lire_w(&[
            ("KESH_WISEFICTIF_1_ACCOUNT", "A"),
            ("KESH_WISEFICTIF_1_TOKEN", "t"),
        ])
        .unwrap();
        assert_eq!(c.series_for("A").unwrap().field("PROFILE_ID"), None);
        let c = lire_w(&[
            ("KESH_WISEFICTIF_1_ACCOUNT", "A"),
            ("KESH_WISEFICTIF_1_TOKEN", "t"),
            ("KESH_WISEFICTIF_1_PROFILE_ID", "42"),
        ])
        .unwrap();
        assert_eq!(c.series_for("A").unwrap().field("PROFILE_ID"), Some("42"));
        match lire_w(&[("KESH_WISEFICTIF_1_PROFILE_ID", "42")]) {
            Err(ConfigError::InvalidProviderVar { var, .. }) => {
                assert_eq!(var, "KESH_WISEFICTIF_1_ACCOUNT")
            }
            other => panic!("refus attendu : {other:?}"),
        }
    }

    #[test]
    fn debug_et_message_ne_disent_aucun_secret() {
        let c = lire(&SERIE_1).unwrap();
        let dbg = format!("{c:?}");
        assert!(dbg.contains("Boutique"));
        assert!(
            !dbg.contains("secret-un") && !dbg.contains("id-un"),
            "{dbg}"
        );
        let mut pairs = SERIE_1.to_vec();
        pairs[2] = ("KESH_PAYPAL_1_SECRET", "<mon-secret-tres-long>");
        let msg = lire(&pairs).unwrap_err().to_string();
        assert!(msg.contains("KESH_PAYPAL_1_SECRET"));
        assert!(!msg.contains("mon-secret"), "{msg}");
    }

    /// Revue P1, F6 — le client lit ses champs par le point commun : un champ
    /// déclaré se lit, un nom non déclaré est une faute de Kesh (`Internal`),
    /// jamais un « non configuré » trompeur.
    #[test]
    fn un_champ_se_lit_s_il_est_declare() {
        let s = ProviderSeries::for_tests(
            1,
            "Boutique",
            &[(PAYPAL_CLIENT_ID, "id-un"), (PAYPAL_SECRET, "secret-un")],
        );
        assert_eq!(PAYPAL.require_field(&s, PAYPAL_CLIENT_ID).unwrap(), "id-un");
        assert_eq!(
            PAYPAL.require_field(&s, PAYPAL_SECRET).unwrap(),
            "secret-un"
        );
        assert!(matches!(
            PAYPAL.require_field(&s, "CLIENT_SECRET"),
            Err(crate::errors::AppError::Internal(_))
        ));
        let partielle = ProviderSeries::for_tests(1, "Boutique", &[(PAYPAL_CLIENT_ID, "id-un")]);
        assert!(matches!(
            PAYPAL.require_field(&partielle, PAYPAL_SECRET),
            Err(crate::errors::AppError::ProviderApiNotConfigured { .. })
        ));
    }

    /// Story 38-12b (AC-1) — l'état que la liste des comptes bancaires rend :
    /// configuré selon le libellé (sans casse), mode selon l'hôte, rien pour un
    /// prestataire non téléchargé ; jamais un secret ni l'hôte dans la forme.
    #[test]
    fn etat_du_telechargement_d_un_compte() {
        let serie = || {
            vec![ProviderSeries::for_tests(
                1,
                "Boutique",
                &[(PAYPAL_CLIENT_ID, "id-un"), (PAYPAL_SECRET, "secret-un")],
            )]
        };
        let avec = |url: &str| {
            ProvidersConfig::default().with(ProviderApiConfig::for_tests(PAYPAL, url, 1, serie()))
        };
        let reel = avec("https://api-m.paypal.com/");
        assert_eq!(
            reel.status_for("paypal", "  boutique "),
            ProviderApiStatus {
                configured: true,
                mode: Some(ProviderApiMode::Live)
            }
        );
        assert_eq!(
            reel.status_for("paypal", "Autre"),
            ProviderApiStatus {
                configured: false,
                mode: None
            }
        );
        assert_eq!(
            avec("https://api-m.sandbox.paypal.com")
                .status_for("paypal", "Boutique")
                .mode,
            Some(ProviderApiMode::Sandbox)
        );
        // Un mot qui contient « sandbox » n'est pas un segment d'hôte d'essai.
        assert_eq!(
            avec("https://notsandbox.example.com")
                .status_for("paypal", "Boutique")
                .mode,
            None
        );
        // Serveur simulé : configuré, sans mode.
        let simule = avec("http://127.0.0.1:4010");
        assert_eq!(
            simule.status_for("paypal", "Boutique"),
            ProviderApiStatus {
                configured: true,
                mode: None
            }
        );
        // Prestataire que cette version ne télécharge pas.
        assert!(!simule.status_for("stripe", "Boutique").configured);
        // Forme JSON : `mode` absent quand il n'y en a pas, aucun secret.
        let json = serde_json::to_string(&simule.status_for("paypal", "Boutique")).unwrap();
        assert_eq!(json, r#"{"configured":true}"#);
        let json = serde_json::to_string(&reel.status_for("paypal", "Boutique")).unwrap();
        assert_eq!(json, r#"{"configured":true,"mode":"live"}"#);
        assert!(!json.contains("secret") && !json.contains("paypal.com"));
    }
}
