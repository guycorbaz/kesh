//! Entité `ReconciliationRule` — règle d'affectation automatique d'une
//! transaction bancaire vers un compte de contrepartie (Story 8-5b FR47).
//!
//! Une `ReconciliationRule` est consommée au flow `GET
//! /api/v1/reconciliation/proposals` pour proposer un compte de
//! contrepartie quand aucune `Invoice` ne match la transaction, puis au
//! flow `POST /api/v1/reconciliation/accept` body `{type: 'rule'}` pour
//! déclencher la création d'un `JournalEntry` via le helper
//! `kesh_reconciliation::manual::build_journal_entry_for_counterparty`
//! (réutilisé de 8-5a-base).
//!
//! ## Soft-delete (Q3 décision Guy 2026-05-07)
//!
//! Le DELETE physique est interdit v0.1 pour préserver l'audit
//! historique des `reconciliation_rule.applied`. La table utilise
//! `active=false` comme tombstone. La contrainte UNIQUE
//! `uq_reconciliation_rules_match_active` est partielle (effective
//! seulement sur les rules actives) via la colonne synthétique
//! `active_uniq` (cf. migration `20260513000001_reconciliation_rules.sql`).

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, MySql, Type, encode::IsNull, error::BoxDynError, mysql::MySqlTypeInfo};

/// Stratégie de matching d'une `ReconciliationRule` contre une
/// `BankTransaction`.
///
/// L'évaluation pratique se fait dans `kesh_reconciliation::rules::rule_matches`
/// (Story 8-5b T3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconciliationMatchType {
    /// Le nom de contrepartie (`tx.counterparty_name`) contient
    /// `match_value` (case-insensitive après `normalize()`).
    CounterpartyContains,
    /// Le nom de contrepartie est exactement égal à `match_value`
    /// (case-insensitive après `normalize()`).
    CounterpartyExact,
    /// La référence (`tx.reference`) contient `match_value`
    /// (case-insensitive après `normalize()`).
    ReferenceContains,
    /// L'IBAN de contrepartie (`tx.counterparty_iban`) est égal à
    /// `match_value` après normalisation canonique (uppercase + strip
    /// whitespace). Pass 1 P-H4 : la normalisation est imposée à la
    /// création par le handler API (cf. `kesh_qrbill::validation::normalize_iban`).
    IbanExact,
}

impl ReconciliationMatchType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CounterpartyContains => "counterparty_contains",
            Self::CounterpartyExact => "counterparty_exact",
            Self::ReferenceContains => "reference_contains",
            Self::IbanExact => "iban_exact",
        }
    }
}

impl std::fmt::Display for ReconciliationMatchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for ReconciliationMatchType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "counterparty_contains" => Ok(Self::CounterpartyContains),
            "counterparty_exact" => Ok(Self::CounterpartyExact),
            "reference_contains" => Ok(Self::ReferenceContains),
            "iban_exact" => Ok(Self::IbanExact),
            other => Err(format!("ReconciliationMatchType inconnu : {other}")),
        }
    }
}

impl Type<MySql> for ReconciliationMatchType {
    fn type_info() -> MySqlTypeInfo {
        <String as Type<MySql>>::type_info()
    }
    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <String as Type<MySql>>::compatible(ty) || <str as Type<MySql>>::compatible(ty)
    }
}

impl<'q> Encode<'q, MySql> for ReconciliationMatchType {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<MySql>>::encode_by_ref(&self.as_str(), buf)
    }
}

impl<'r> Decode<'r, MySql> for ReconciliationMatchType {
    fn decode(value: <MySql as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<MySql>>::decode(value)?;
        s.parse().map_err(Into::into)
    }
}

/// Règle de réconciliation persistée. La colonne synthétique
/// `active_uniq` (migration `20260513000001`) n'est pas exposée
/// Rust-side — elle existe uniquement pour réaliser le UNIQUE partiel.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationRule {
    pub id: i64,
    pub company_id: i64,
    pub label: String,
    pub match_type: ReconciliationMatchType,
    pub match_value: String,
    pub counterparty_account_id: i64,
    pub priority: i32,
    pub active: bool,
    /// Projet analytique par défaut (Story 19-5, Epic 19) : recopié sur
    /// l'écriture générée quand la règle est appliquée à l'accept
    /// (`accept_one_rule`). `None` = pas de tag analytique. Validé à la
    /// création/édition de la règle **et** re-validé à l'accept (le projet a
    /// pu être archivé entre-temps).
    pub default_project_id: Option<i64>,
    pub applied_count: i64,
    pub last_applied_at: Option<NaiveDateTime>,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Données de création d'une `ReconciliationRule` (POST /rules).
///
/// `match_value` doit déjà être normalisé si `match_type ==
/// IbanExact` — le repository ne re-normalise pas (responsabilité
/// handler API, cf. §rules-types-rust Pass 1 P-H4).
#[derive(Debug, Clone)]
pub struct NewReconciliationRule {
    pub label: String,
    pub match_type: ReconciliationMatchType,
    pub match_value: String,
    pub counterparty_account_id: i64,
    pub priority: i32,
    /// Projet analytique par défaut (Story 19-5). Validé par le repository
    /// (`create_in_tx`) via `projects::validate_taggable_in_tx` si `Some`.
    pub default_project_id: Option<i64>,
}

/// Patch partiel d'une `ReconciliationRule` (PATCH /rules/{id}).
///
/// `match_type` non patchable v0.1 (cf. §rules-types-rust : changer le
/// type briserait la cohérence des audit `reconciliation_rule.applied`
/// qui référencent un `match_type` figé).
///
/// `expected_version` est porté par les arguments du repository
/// `update_in_tx`, pas par cette struct (cf. T2.2).
#[derive(Debug, Clone, Default)]
pub struct UpdateReconciliationRule {
    pub label: Option<String>,
    pub match_value: Option<String>,
    pub counterparty_account_id: Option<i64>,
    pub priority: Option<i32>,
    pub active: Option<bool>,
    /// Projet analytique par défaut (Story 19-5) — sémantique à **deux
    /// niveaux** distincte des autres champs pour permettre l'effacement
    /// (DC4) : `None` = inchangé ; `Some(None)` = effacer le projet ;
    /// `Some(Some(id))` = affecter le projet `id` (validé si différent de
    /// la valeur stockée, grandfathering du tag inchangé).
    pub default_project_id: Option<Option<i64>>,
}
