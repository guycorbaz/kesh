//! kesh-i18n — Internationalisation et formatage suisse pour Kesh.
//!
//! Cette crate fournit :
//! - Chargement de fichiers Fluent (.ftl) pour FR/DE/IT/EN
//! - Résolution de messages avec fallback vers FR-CH
//! - Formatage suisse : montants (apostrophe U+2019) et dates (dd.mm.yyyy)

pub mod error;
pub mod formatting;
pub mod loader;

pub use error::I18nError;
pub use fluent_bundle::FluentArgs;
pub use formatting::{format_date, format_datetime, format_money};
pub use loader::I18nBundle;

use std::fmt;

/// Locales suisses supportées.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Locale {
    FrCh,
    DeCh,
    ItCh,
    EnCh,
}

impl Locale {
    /// Toutes les locales supportées.
    pub const ALL: [Locale; 4] = [Locale::FrCh, Locale::DeCh, Locale::ItCh, Locale::EnCh];

    /// Nom du répertoire correspondant dans `locales/`.
    pub fn dir_name(&self) -> &'static str {
        match self {
            Locale::FrCh => "fr-CH",
            Locale::DeCh => "de-CH",
            Locale::ItCh => "it-CH",
            Locale::EnCh => "en-CH",
        }
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.dir_name())
    }
}

impl From<&str> for Locale {
    /// Parsing permissif : "fr", "fr-CH", "FR", "fr-ch" → FrCh.
    /// Valeur inconnue → FrCh (fallback avec warning).
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "fr" | "fr-ch" => Locale::FrCh,
            "de" | "de-ch" => Locale::DeCh,
            "it" | "it-ch" => Locale::ItCh,
            "en" | "en-ch" => Locale::EnCh,
            other => {
                tracing::warn!("Locale '{}' non reconnue, fallback vers fr-CH", other);
                Locale::FrCh
            }
        }
    }
}
