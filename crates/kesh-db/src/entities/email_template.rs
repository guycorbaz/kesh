//! Entité `EmailTemplate` — socle des templates d'e-mail company-scoped
//! (Epic 20 #224, Story 20-1).
//!
//! Une ligne = un override explicite du template par défaut pour
//! `(company_id, template_type, language)`. L'absence de ligne est le cas
//! normal (zéro-config) : le texte par défaut (cf. [`crate::entities::email_template_defaults`])
//! s'applique. [`EffectiveEmailTemplate`] est le résultat déjà résolu
//! (override ou défaut) renvoyé par le repository — jamais de 404, jamais
//! d'état "introuvable" pour une combinaison type×langue valide.
//!
//! Réutilise [`crate::entities::Language`] tel quel (FR/DE/IT/EN) — pas de
//! nouvel enum langue.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, MySql, Type, encode::IsNull, error::BoxDynError, mysql::MySqlTypeInfo};

use crate::entities::Language;

/// Type de template d'e-mail. v1 : `InvoiceSend` seul (facture validée
/// envoyée par e-mail). Futurs : rappel de facture, facture récurrente,
/// devis — chacun déclarera son propre jeu de variables autorisées.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmailTemplateType {
    /// Envoi d'une facture validée par e-mail (PDF QR-facture joint).
    InvoiceSend,
}

impl EmailTemplateType {
    pub const ALL: [EmailTemplateType; 1] = [EmailTemplateType::InvoiceSend];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InvoiceSend => "invoice_send",
        }
    }

    /// Variables `{var}` autorisées dans le subject/body de ce type de
    /// template. Toute variable hors de cette liste est rejetée à la
    /// validation du save (cf. `kesh_core::email_template_engine::validate_tokens`).
    pub fn allowed_variables(&self) -> &'static [&'static str] {
        match self {
            Self::InvoiceSend => &[
                "salutation",
                "contactName",
                "invoiceNumber",
                "amount",
                "dueDate",
                "companyName",
            ],
        }
    }

    /// `allowed_variables()` converti en `Vec<String>` — évite de dupliquer
    /// `iter().map(|s| s.to_string()).collect()` à chaque site d'appel
    /// (repository + DTOs API).
    pub fn allowed_variables_owned(&self) -> Vec<String> {
        self.allowed_variables()
            .iter()
            .map(|s| s.to_string())
            .collect()
    }
}

impl std::str::FromStr for EmailTemplateType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "invoice_send" => Ok(Self::InvoiceSend),
            other => Err(format!("Type de template d'e-mail inconnu : '{other}'")),
        }
    }
}

impl Type<MySql> for EmailTemplateType {
    fn type_info() -> MySqlTypeInfo {
        <String as Type<MySql>>::type_info()
    }
    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <String as Type<MySql>>::compatible(ty) || <str as Type<MySql>>::compatible(ty)
    }
}

impl<'q> Encode<'q, MySql> for EmailTemplateType {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<MySql>>::encode_by_ref(&self.as_str(), buf)
    }
}

impl<'r> Decode<'r, MySql> for EmailTemplateType {
    fn decode(value: <MySql as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<MySql>>::decode(value)?;
        s.parse().map_err(Into::into)
    }
}

/// Override de template persisté en base (`email_templates`).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EmailTemplate {
    pub id: i64,
    pub company_id: i64,
    pub template_type: EmailTemplateType,
    pub language: Language,
    pub subject: String,
    pub body: String,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Résultat résolu (override si présent, sinon défaut) — jamais d'échec
/// pour une combinaison type×langue valide.
#[derive(Debug, Clone)]
pub struct EffectiveEmailTemplate {
    pub template_type: EmailTemplateType,
    pub language: Language,
    pub subject: String,
    pub body: String,
    /// `None` quand `is_default = true` (rien à verrouiller : pas de ligne).
    pub version: Option<i32>,
    pub is_default: bool,
    pub allowed_variables: Vec<String>,
}
