//! Entité `McpServerSettings` : réglages du serveur MCP d'une société
//! (Story 32-5a, refs #723).
//!
//! **Une ligne par société au plus**, absente tant que l'administrateur n'a
//! rien enregistré (C-32-5-8) : la lecture rend alors
//! [`McpServerSettings::defaults`], et rien n'est écrit sur un chemin de
//! lecture (la 32-6 lira ces réglages sur la `Lecture`, peut-être en session
//! `READ ONLY`). `version == 0` désigne « aucune ligne » : c'est la version
//! que rendent les défauts, et celle qu'un premier enregistrement présente.
//!
//! Les trois listes (plages admises, mandataires de confiance, origines) sont
//! des tableaux JSON de chaînes **canoniques** ; leur forme est validée à
//! l'écriture par la route (32-5c), leur sens est celui de la 32-6. La base ne
//! garantit que « tableau JSON valide » (`CHECK`).

use chrono::NaiveDateTime;

/// Défaut : serveur **désactivé** (C-32-5-9).
pub const DEFAULT_ENABLED: bool = false;

/// Défaut : filtre d'adresse **activé** (C-32-5-9).
pub const DEFAULT_ADDRESS_FILTER_ENABLED: bool = true;

/// Défaut des plages admises : boucle locale et réseaux privés, **sans** la
/// plage partagée `100.64.0.0/10` des VPN et du CGNAT (C-32-5-10).
pub const DEFAULT_ALLOWED_NETWORKS: [&str; 7] = [
    "127.0.0.0/8",
    "::1/128",
    "10.0.0.0/8",
    "172.16.0.0/12",
    "192.168.0.0/16",
    "fc00::/7",
    "fe80::/10",
];

/// Défaut des mandataires de confiance : aucun.
pub const DEFAULT_TRUSTED_PROXIES: [&str; 0] = [];

/// Défaut des origines admises : aucune.
pub const DEFAULT_ALLOWED_ORIGINS: [&str; 0] = [];

/// Réglages du serveur MCP d'une société, tels que lus (ligne ou défauts).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpServerSettings {
    pub company_id: i64,
    pub enabled: bool,
    pub address_filter_enabled: bool,
    pub allowed_networks: Vec<String>,
    pub trusted_proxies: Vec<String>,
    pub allowed_origins: Vec<String>,
    /// `0` : aucune ligne (défauts du code) ; `≥ 1` : version de la ligne.
    pub version: i32,
    /// `None` pour les défauts.
    pub updated_at: Option<NaiveDateTime>,
}

impl McpServerSettings {
    /// Les défauts du code pour une société sans ligne (`version == 0`).
    pub fn defaults(company_id: i64) -> Self {
        Self {
            company_id,
            enabled: DEFAULT_ENABLED,
            address_filter_enabled: DEFAULT_ADDRESS_FILTER_ENABLED,
            allowed_networks: DEFAULT_ALLOWED_NETWORKS.map(String::from).to_vec(),
            trusted_proxies: DEFAULT_TRUSTED_PROXIES.map(String::from).to_vec(),
            allowed_origins: DEFAULT_ALLOWED_ORIGINS.map(String::from).to_vec(),
            version: 0,
            updated_at: None,
        }
    }
}

/// Contenu d'un enregistrement (tous les champs : remplacement complet).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpServerSettingsUpdate {
    pub enabled: bool,
    pub address_filter_enabled: bool,
    pub allowed_networks: Vec<String>,
    pub trusted_proxies: Vec<String>,
    pub allowed_origins: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_disabled_filtered_and_private_networks_only() {
        let d = McpServerSettings::defaults(7);
        assert_eq!(d.company_id, 7);
        assert!(!d.enabled, "serveur désactivé par défaut");
        assert!(d.address_filter_enabled);
        assert_eq!(d.version, 0, "version 0 = aucune ligne");
        assert_eq!(d.updated_at, None);
        assert_eq!(d.allowed_networks.len(), 7);
        assert!(d.allowed_networks.iter().any(|n| n == "192.168.0.0/16"));
        assert!(
            !d.allowed_networks.iter().any(|n| n.starts_with("100.64.")),
            "la plage partagée des VPN n'est pas admise par défaut (C-32-5-10)"
        );
        assert!(d.trusted_proxies.is_empty());
        assert!(d.allowed_origins.is_empty());
    }
}
