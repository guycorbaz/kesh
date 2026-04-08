//! Erreurs du module i18n.

use thiserror::Error;

/// Erreurs liées au chargement et à la résolution des traductions.
#[derive(Debug, Error)]
pub enum I18nError {
    /// Erreur de parsing d'un fichier Fluent (.ftl).
    #[error("Erreur de parsing Fluent pour {locale}: {detail}")]
    FluentParse { locale: String, detail: String },

    /// Fichier de ressource manquant pour une locale.
    #[error("Fichier de ressource manquant: {0}")]
    MissingResource(String),

    /// Erreur d'entrée/sortie lors du chargement des fichiers.
    #[error("Erreur I/O: {0}")]
    Io(#[from] std::io::Error),
}
