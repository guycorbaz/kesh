//! Éligibilité des factures aux rappels débiteurs (Story 21-5a, #231).
//!
//! Détermine, pour une company, la liste des factures « à rappeler » avec leur niveau
//! courant, prochain niveau, état terminal et date du dernier rappel.
//!
//! **Approche** : le SQL récupère les candidates brutes (factures `validated`/impayées/
//! non-suspendues, avec `due_date`, jointes au contact et à l'agrégat des rappels
//! non-annulés) ; la logique de déclenchement par date est faite en Rust (`today` en
//! **UTC** via `chrono::Utc`, cohérent avec `UTC_DATE()`/`due_dates_summary`). Cela évite
//! l'arithmétique de dates + sous-requêtes NULL fragiles en SQL.
//!
//! **Garde « dunning désactivé » (C1/D7)** : si la company a 0 niveau configuré (table
//! `dunning_levels` vidée volontairement, `seeded_at` non-NULL → seed lazy no-op), la
//! liste est **vide** — aucune facture candidate ni terminale (pas de résurrection).

use crate::errors::{DbError, map_db_error};
use crate::repositories::{company_dunning_settings, dunning_levels};
use chrono::{Duration, NaiveDate, NaiveDateTime, Utc};
use sqlx::MySqlPool;

/// Une facture candidate au rappel (ou en état terminal), avec le contexte de contact.
#[derive(Debug, Clone)]
pub struct ReminderCandidate {
    pub invoice_id: i64,
    pub invoice_number: Option<String>,
    pub due_date: NaiveDate,
    pub contact_id: i64,
    pub contact_name: String,
    /// `true` si le contact a une adresse e-mail non vide (cochable pour l'envoi lot, 21-6).
    pub has_email: bool,
    /// Niveau courant = MAX(level_number) des rappels non-annulés (0 si aucun).
    pub current_level: i16,
    /// Prochain niveau à envoyer (plus petit `level_number > current_level`), `None` si terminal.
    pub next_level: Option<i16>,
    /// `true` si le dernier niveau configuré est atteint (fin de cycle visible, item 9).
    pub terminal: bool,
    /// Date/heure du dernier rappel non-annulé (`None` si aucun).
    pub last_reminder_at: Option<NaiveDateTime>,
}

#[derive(sqlx::FromRow)]
struct CandidateRow {
    invoice_id: i64,
    invoice_number: Option<String>,
    due_date: NaiveDate,
    contact_id: i64,
    contact_name: String,
    contact_email: Option<String>,
    current_level: i64,
    last_reminder_at: Option<NaiveDateTime>,
}

/// Retourne les factures à rappeler pour une company (déclenche le seed lazy des
/// niveaux par défaut si la company n'a jamais été seedée). Vide si dunning désactivé.
pub async fn list_reminder_candidates(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Vec<ReminderCandidate>, DbError> {
    // Seed lazy « au premier accès / 1re évaluation » (no-op si déjà seedé OU vidé
    // volontairement — `seeded_at` non-NULL). Transaction courte dédiée.
    let mut tx = pool.begin().await.map_err(map_db_error)?;
    let settings = company_dunning_settings::ensure_seeded_in_tx(&mut tx, company_id).await?;
    tx.commit().await.map_err(map_db_error)?;

    // Config des niveaux (level_number → delay_days). Garde C1 : 0 niveau → liste vide.
    let levels = dunning_levels::list_all_by_company(pool, company_id).await?;
    if levels.is_empty() {
        return Ok(Vec::new());
    }
    let grace = settings.grace_period_days;

    let rows = sqlx::query_as::<_, CandidateRow>(
        "SELECT i.id AS invoice_id, i.invoice_number AS invoice_number, i.due_date AS due_date, \
                c.id AS contact_id, c.name AS contact_name, c.email AS contact_email, \
                CAST(COALESCE(ra.current_level, 0) AS SIGNED) AS current_level, \
                ra.last_reminder_at AS last_reminder_at \
         FROM invoices i \
         JOIN contacts c ON c.id = i.contact_id \
         LEFT JOIN ( \
             SELECT invoice_id, MAX(level_number) AS current_level, MAX(sent_at) AS last_reminder_at \
             FROM invoice_reminders \
             WHERE company_id = ? AND cancelled_at IS NULL \
             GROUP BY invoice_id \
         ) ra ON ra.invoice_id = i.id \
         WHERE i.company_id = ? \
           AND i.status = 'validated' \
           AND i.paid_at IS NULL \
           AND i.dunning_paused_at IS NULL \
           AND i.due_date IS NOT NULL \
         ORDER BY c.name, i.due_date, i.id",
    )
    .bind(company_id)
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)?;

    let today = Utc::now().date_naive();
    let delay_of = |ln: i16| -> i32 {
        levels
            .iter()
            .find(|l| l.level_number == ln)
            .map(|l| l.delay_days)
            .unwrap_or(0)
    };

    let mut candidates = Vec::new();
    for row in rows {
        let current_level = row.current_level as i16;
        let next_level = levels
            .iter()
            .map(|l| l.level_number)
            .filter(|ln| *ln > current_level)
            .min();
        // Terminal : au moins un rappel envoyé et aucun niveau supérieur configuré.
        let terminal = current_level >= 1 && next_level.is_none();

        // Date de déclenchement du prochain rappel.
        let due = if current_level == 0 {
            // Niveau 1 (jamais rappelée) : échéance + grâce + délai(niveau 1).
            // `level_number` contigu 1-based (D5) → le niveau 1 existe dès que la config
            // est non vide.
            let trigger = row.due_date + Duration::days((grace + delay_of(1)) as i64);
            today >= trigger
        } else if let Some(nl) = next_level {
            // Niveau N>1 : dernier rappel + délai(niveau suivant).
            match row.last_reminder_at {
                Some(lr) => today >= lr.date() + Duration::days(delay_of(nl) as i64),
                None => false,
            }
        } else {
            false
        };

        if due || terminal {
            let has_email = row
                .contact_email
                .as_deref()
                .map(str::trim)
                .is_some_and(|e| !e.is_empty());
            candidates.push(ReminderCandidate {
                invoice_id: row.invoice_id,
                invoice_number: row.invoice_number,
                due_date: row.due_date,
                contact_id: row.contact_id,
                contact_name: row.contact_name,
                has_email,
                current_level,
                next_level,
                terminal,
                last_reminder_at: row.last_reminder_at,
            });
        }
    }

    Ok(candidates)
}
