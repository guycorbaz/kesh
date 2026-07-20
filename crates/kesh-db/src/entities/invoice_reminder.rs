//! Rappel débiteur (dunning) — historique append-only d'une facture.
//!
//! Calqué sur `dunning_level` : `FromRow` sans `Serialize` (anti-fuite `company_id`
//! au client ; l'exposition REST passe par un `…Response` dédié côté routes). Chaque
//! ligne snapshote `level_number` + `fee_amount` + `subject` + `body` (preuve de ce
//! qui a été réclamé — acte pré-contentieux, item 11 epic-21). `cancelled_at` = annulation
//! SOFT (Admin) qui exclut la ligne du MAX(level_number) déterminant le niveau courant,
//! sans casser l'append-only.

use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use std::str::FromStr;

/// Canal d'un rappel : e-mail envoyé par Kesh (21-5b) ou rappel papier enregistré
/// manuellement (21-5a). Sérialisé/persisté en **minuscule** (`'email'`/`'manual'`),
/// cohérent avec le CHECK DB `chk_invoice_reminders_channel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReminderChannel {
    Email,
    Manual,
}

impl ReminderChannel {
    /// Représentation persistée / JSON (minuscule).
    pub fn as_str(&self) -> &'static str {
        match self {
            ReminderChannel::Email => "email",
            ReminderChannel::Manual => "manual",
        }
    }
}

impl FromStr for ReminderChannel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "email" => Ok(ReminderChannel::Email),
            "manual" => Ok(ReminderChannel::Manual),
            other => Err(format!("canal de rappel inconnu : {other}")),
        }
    }
}

/// Rappel persisté (`invoice_reminders`). `channel` reste `String` en `FromRow`
/// (parse via [`ReminderChannel::from_str`] au besoin) — la contrainte de validité
/// est garantie par le CHECK DB.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct InvoiceReminder {
    pub id: i64,
    pub company_id: i64,
    pub invoice_id: i64,
    /// Niveau du rappel (≥ 1, CHECK DB).
    pub level_number: i16,
    /// Snapshot du frais du niveau au moment de l'envoi (CHF, borné 0..10'000).
    pub fee_amount: Decimal,
    pub sent_at: NaiveDateTime,
    /// `'email'` ou `'manual'` (CHECK DB).
    pub channel: String,
    /// Destinataire e-mail réel (snapshot) ; `NULL` si `channel = 'manual'`.
    pub sent_to: Option<String>,
    /// Snapshots du texte réclamé (preuve pré-contentieuse).
    pub subject: String,
    pub body: String,
    /// Note libre d'un rappel manuel.
    pub note: Option<String>,
    /// Acteur (pointeur logique, pas de FK — l'audit_log porte la trace authentifiée).
    pub actor_user_id: Option<i64>,
    /// Annulation soft (Admin) — exclut la ligne du MAX(level_number).
    pub cancelled_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

/// Payload d'insertion d'un rappel (manuel en 21-5a ; e-mail en 21-5b réutilisera
/// le même chemin). `id`/`created_at` posés par la DB ; `cancelled_at` toujours NULL
/// à l'insertion.
#[derive(Debug, Clone)]
pub struct NewInvoiceReminder {
    pub company_id: i64,
    pub invoice_id: i64,
    pub level_number: i16,
    pub fee_amount: Decimal,
    pub sent_at: NaiveDateTime,
    pub channel: ReminderChannel,
    pub sent_to: Option<String>,
    pub subject: String,
    pub body: String,
    pub note: Option<String>,
    pub actor_user_id: Option<i64>,
}
