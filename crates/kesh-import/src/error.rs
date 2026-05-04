//! Erreurs émises par les parseurs `kesh-import`.
//!
//! `CamtError` représente uniquement les défaillances à la frontière du
//! parseur CAMT.053 (XML mal formé, version non supportée, champ requis
//! manquant). Les violations métier (devise non supportée par la v0.1,
//! balance check échoué) appartiennent à `kesh-core::CoreError` et ne
//! transitent pas par cette enum — la séparation reflète la décision
//! architecture #7 : `kesh-import` est publiable indépendamment et ne
//! connaît rien du domaine Kesh.

use thiserror::Error;

/// Erreurs détectées lors du parsing d'un fichier CAMT.053.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum CamtError {
    /// Le contenu XML est mal formé (balise non fermée, encodage invalide,
    /// caractère illégal, etc.). Le message contient le détail remonté par
    /// `quick-xml` pour faciliter le diagnostic côté logs serveur.
    #[error("XML mal formé : {0}")]
    MalformedXml(String),

    /// Le namespace racine de `<Document>` ne correspond à aucune version
    /// CAMT.053 supportée par le parseur (`camt.053.001.04` ou `.08`).
    #[error("Version CAMT.053 non supportée : {0}")]
    UnsupportedVersion(String),

    /// Un champ requis pour construire un `ImportedStatement` ou une
    /// `ImportedTransaction` est absent du document. Le paramètre porte
    /// le chemin du champ manquant en notation dot-path indexée pour
    /// faciliter le diagnostic sur fichiers multi-`<Stmt>` / multi-`<Ntry>`
    /// (ex. `"stmt[2].ntry[5].amount"`, `"stmt[0].account_iban"`).
    #[error("Champ requis manquant : {0}")]
    MissingRequiredField(String),

    /// Un montant `<Amt>` n'a pas pu être parsé comme `Decimal`. Le message
    /// contient la valeur brute incriminée.
    #[error("Montant invalide : {0}")]
    InvalidAmount(String),

    /// Une date (`<Dt>`, `<DtTm>`, `<FrDtTm>`, `<ToDtTm>`) n'a pas pu être
    /// parsée. Le message contient la valeur brute incriminée.
    #[error("Date invalide : {0}")]
    InvalidDate(String),
}
