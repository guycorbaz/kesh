//! Story 20-3b1 — envoi d'une facture validée par e-mail (#224, epic-20).
//!
//! Deux endpoints Comptable+ :
//! - `GET /api/v1/invoices/{id}/email-preview` — rend le template
//!   `invoice_send` effectif (override company ou défaut) dans la langue du
//!   contact et renvoie destinataire + objet + corps pré-remplis. Endpoint
//!   nécessaire car les routes templates (20-1) sont Admin-only alors que
//!   l'expéditeur est Comptable+.
//! - `POST /api/v1/invoices/{id}/send-email` — envoie la facture (PDF
//!   QR-facture joint via `invoice_pdf_service::render`) au **destinataire
//!   verrouillé** `contacts.email` (décision #13 epic-20 — seuls objet et
//!   corps sont éditables), puis marque `invoices.emailed_at`/`emailed_to`
//!   et audite `invoice.emailed`. Le texte envoyé est celui du body de la
//!   requête (ce que l'utilisateur a vu/édité dans la modale) — le serveur
//!   ne re-rend PAS le template au send.

use std::collections::HashMap;

use axum::Extension;
use axum::extract::{Path, State};
use axum::{Json, http::StatusCode};
use serde::{Deserialize, Serialize};

use kesh_db::entities::contact::{Contact, ContactType, Salutation};
use kesh_db::entities::email_template::EmailTemplateType;
use kesh_db::entities::{Company, Language};
use kesh_db::errors::DbError;
use kesh_db::repositories::{
    contacts, dunning_levels, email_templates, invoice_reminders, invoices,
};
use kesh_i18n::{format_date, format_money};

use crate::AppState;
use crate::errors::AppError;
use crate::helpers::{exceeds_len, get_company_for, validate_text_len};
use crate::mail::{EmailAttachment, OutgoingEmail};
use crate::middleware::auth::CurrentUser;
use crate::routes::invoice_pdf_service;
use crate::routes::invoices::InvoiceResponse;

/// Réponse de `GET /api/v1/invoices/{id}/email-preview`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailPreviewResponse {
    /// Destinataire verrouillé (`contacts.email`). `null` = contact sans
    /// e-mail — le frontend désactive l'envoi (le POST refuserait en 400).
    pub to: Option<String>,
    /// Langue résolue (contact sinon instance) — `"FR"`/`"DE"`/`"IT"`/`"EN"`.
    pub language: Language,
    pub subject: String,
    pub body: String,
}

/// Payload de `POST /api/v1/invoices/{id}/send-email`. Pas de champ `to` :
/// destinataire verrouillé côté serveur (anti-exfiltration / relais de spam).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendInvoiceEmailRequest {
    pub subject: String,
    pub body: String,
}

/// Résout la langue de correspondance : celle du contact si renseignée,
/// sinon la langue d'instance de la société (décision #11 epic-20).
///
/// `pub(crate)` (Story 21-1) : réutilisé par `routes/contacts` (libellé des
/// conditions de paiement dans la langue du contact) et `routes/invoices`
/// (libellé auto à la création de facture).
pub(crate) fn resolve_language(contact: &Contact, company: &Company) -> Language {
    contact.language.unwrap_or(company.instance_language)
}

/// Destinataire verrouillé = `contacts.email` (décision #13 epic-20) —
/// vide/whitespace = absent. Partagé preview/send (review Pass 1 BH-5).
fn locked_recipient(contact: &Contact) -> Option<String> {
    contact
        .email
        .as_deref()
        .map(str::trim)
        .filter(|e| !e.is_empty())
        .map(String::from)
}

/// Charge le contact de la facture, scopé company (défense en profondeur,
/// review Pass 1 BH-2 — même si `invoice.contact_id` provient déjà d'une
/// facture scopée) et refuse les contacts archivés (`active = false`,
/// review Pass 1 ECH-2 : le carnet d'adresses les considère « à ne plus
/// utiliser », on ne leur envoie pas de facture).
async fn load_active_contact(
    pool: &sqlx::MySqlPool,
    contact_id: i64,
    company_id: i64,
) -> Result<Contact, AppError> {
    let contact = contacts::find_by_id_in_company(pool, contact_id, company_id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;
    if !contact.active {
        return Err(AppError::ContactArchived);
    }
    Ok(contact)
}

/// Formule d'appel `{salutation}` — matrice genre × langue × type de contact
/// (décision #12 epic-20, matrice figée dans la spec 20-3b1).
///
/// `Personne` avec civilité renseignée : formule genrée, + nom de famille
/// s'il est disponible. `Entreprise` ou civilité `Neutre` : formule neutre
/// (jamais de nom).
fn salutation_line(
    salutation: Salutation,
    contact_type: ContactType,
    last_name: Option<&str>,
    language: Language,
) -> String {
    use Language as L;
    use Salutation as S;

    let neutral = match language {
        L::Fr => "Madame, Monsieur",
        L::De => "Sehr geehrte Damen und Herren",
        L::It => "Gentili Signore e Signori",
        L::En => "Dear Sir or Madam",
    };

    if contact_type == ContactType::Entreprise || salutation == S::Neutre {
        return neutral.to_string();
    }

    let last_name = last_name.map(str::trim).filter(|n| !n.is_empty());
    match (salutation, language, last_name) {
        (S::Monsieur, L::Fr, Some(n)) => format!("Cher Monsieur {n}"),
        (S::Monsieur, L::Fr, None) => "Cher Monsieur".to_string(),
        (S::Monsieur, L::De, Some(n)) => format!("Sehr geehrter Herr {n}"),
        (S::Monsieur, L::De, None) => "Sehr geehrter Herr".to_string(),
        // IT : « Signor » (tronqué) devant un nom, « Signore » seul.
        (S::Monsieur, L::It, Some(n)) => format!("Egregio Signor {n}"),
        (S::Monsieur, L::It, None) => "Egregio Signore".to_string(),
        (S::Monsieur, L::En, Some(n)) => format!("Dear Mr {n}"),
        (S::Monsieur, L::En, None) => "Dear Sir".to_string(),
        (S::Madame, L::Fr, Some(n)) => format!("Chère Madame {n}"),
        (S::Madame, L::Fr, None) => "Chère Madame".to_string(),
        (S::Madame, L::De, Some(n)) => format!("Sehr geehrte Frau {n}"),
        (S::Madame, L::De, None) => "Sehr geehrte Frau".to_string(),
        (S::Madame, L::It, Some(n)) => format!("Gentile Signora {n}"),
        (S::Madame, L::It, None) => "Gentile Signora".to_string(),
        (S::Madame, L::En, Some(n)) => format!("Dear Ms {n}"),
        (S::Madame, L::En, None) => "Dear Madam".to_string(),
        // Neutre déjà court-circuité ci-dessus.
        (S::Neutre, ..) => neutral.to_string(),
    }
}

/// Construit la map des 6 variables déclarées par
/// `EmailTemplateType::InvoiceSend.allowed_variables()`, valeurs
/// **pré-formatées suisses** (`format_money` apostrophe U+2019,
/// `format_date` dd.mm.yyyy). `dueDate` absente → « — ».
///
/// `lines` (Story 21-2a, #246) : `{amount}` est le **TTC canonique** calculé
/// depuis les lignes — `invoice.total_amount` est le HT comptable et ne doit
/// jamais être annoncé comme montant dû au client.
fn build_invoice_vars(
    invoice: &kesh_db::entities::Invoice,
    lines: &[kesh_db::entities::InvoiceLine],
    contact: &Contact,
    company: &Company,
    language: Language,
) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    vars.insert(
        "salutation".to_string(),
        salutation_line(
            contact.salutation,
            contact.contact_type,
            contact.last_name.as_deref(),
            language,
        ),
    );
    vars.insert("contactName".to_string(), contact.name.clone());
    vars.insert("companyName".to_string(), company.name.clone());
    vars.insert(
        "invoiceNumber".to_string(),
        invoice
            .invoice_number
            .clone()
            .unwrap_or_else(|| format!("#{}", invoice.id)),
    );
    let total_ttc = kesh_core::accounting::vat::invoice_total_ttc(
        lines.iter().map(|l| (l.line_total, l.vat_rate)),
    );
    vars.insert("amount".to_string(), format_money(&total_ttc));
    vars.insert(
        "dueDate".to_string(),
        invoice
            .due_date
            .as_ref()
            .map(format_date)
            .unwrap_or_else(|| "—".to_string()),
    );
    vars
}

/// Jours de retard d'une facture (clampé ≥ 0), `0` si `due_date` absente. `today` en
/// UTC (`chrono::Utc::now().naive_utc().date()`), cohérent avec `is_invoice_overdue`.
pub(crate) fn days_overdue(due_date: Option<chrono::NaiveDate>, today: chrono::NaiveDate) -> i64 {
    due_date.map(|d| (today - d).num_days().max(0)).unwrap_or(0)
}

/// Variables de substitution pour un rappel débiteur (Story 21-5b) : les 6 variables
/// de base de [`build_invoice_vars`] (`amount` = TTC facture) + les 4 spécifiques rappel.
/// `total_due`/`reminder_fee`/`days_overdue`/`level_number` sont **pré-calculés** par
/// l'appelant (le builder reste pur, sans accès DB).
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_reminder_vars(
    invoice: &kesh_db::entities::Invoice,
    lines: &[kesh_db::entities::InvoiceLine],
    contact: &Contact,
    company: &Company,
    language: Language,
    level_number: i16,
    reminder_fee: &rust_decimal::Decimal,
    total_due: &rust_decimal::Decimal,
    days_overdue: i64,
) -> HashMap<String, String> {
    let mut vars = build_invoice_vars(invoice, lines, contact, company, language);
    vars.insert("reminderLevel".to_string(), level_number.to_string());
    vars.insert("reminderFee".to_string(), format_money(reminder_fee));
    vars.insert("totalDue".to_string(), format_money(total_due));
    vars.insert("daysOverdue".to_string(), days_overdue.to_string());
    vars
}

/// `GET /api/v1/invoices/{id}/email-preview` — Comptable+.
pub async fn preview_invoice_email(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<EmailPreviewResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;

    // Facture scopée company (anti-IDOR) + contact actif scopé company.
    // #246 : les lignes servent au TTC de {amount} — ne plus les jeter.
    let (invoice, lines) = invoices::find_by_id_with_lines(&state.pool, company.id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;
    let contact = load_active_contact(&state.pool, invoice.contact_id, company.id).await?;

    let language = resolve_language(&contact, &company);
    let template = email_templates::get_effective(
        &state.pool,
        company.id,
        EmailTemplateType::InvoiceSend,
        language,
        0,
    )
    .await?;

    let vars = build_invoice_vars(&invoice, &lines, &contact, &company, language);
    let subject = kesh_core::email_template_engine::render(&template.subject, &vars);
    let body = kesh_core::email_template_engine::render(&template.body, &vars);

    Ok(Json(EmailPreviewResponse {
        to: locked_recipient(&contact),
        language,
        subject,
        body,
    }))
}

/// Paramètre de niveau requis pour la preview/envoi de rappel (Story 21-5b).
/// `#[serde(default)]` volontairement ABSENT (M1) : absence → 400.
#[derive(Debug, serde::Deserialize)]
pub struct ReminderLevelQuery {
    pub level: i16,
}

/// Réponse de preview d'un rappel (Story 21-5b) — calquée `EmailPreviewResponse` + `level`.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderPreviewResponse {
    pub to: Option<String>,
    pub language: Language,
    pub level: i16,
    pub subject: String,
    pub body: String,
}

/// Corps de l'envoi unitaire d'un rappel (Story 21-5b). **PAS de champ `to`**
/// (destinataire verrouillé = `contacts.email`). subject/body édités depuis la preview.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendReminderRequest {
    pub level_number: i16,
    pub subject: String,
    pub body: String,
}

/// Prochain niveau de rappel à envoyer = plus petit `level_number > current_level`
/// dans la config, `None` si le dernier niveau est atteint (état terminal).
fn next_reminder_level(
    levels: &[kesh_db::entities::DunningLevel],
    current_level: i16,
) -> Option<i16> {
    levels
        .iter()
        .map(|l| l.level_number)
        .filter(|ln| *ln > current_level)
        .min()
}

/// Calcule `{totalDue}`, `{reminderFee}`, `{daysOverdue}` puis rend subject+body d'un
/// rappel de niveau `level` pour la facture (helper partagé preview/preview-lot).
/// `total_due` = TTC + Σ frais non-annulés dédupliqués (hors niveau courant) + frais du niveau.
#[allow(clippy::too_many_arguments)]
async fn render_reminder(
    state: &AppState,
    company: &Company,
    invoice: &kesh_db::entities::Invoice,
    lines: &[kesh_db::entities::InvoiceLine],
    contact: &Contact,
    language: Language,
    level_number: i16,
    level_fee: &rust_decimal::Decimal,
) -> Result<(String, String), AppError> {
    let ttc = kesh_core::accounting::vat::invoice_total_ttc(
        lines.iter().map(|l| (l.line_total, l.vat_rate)),
    );
    let other_fees = invoice_reminders::sum_fees_deduped_excluding(
        &state.pool,
        company.id,
        invoice.id,
        level_number,
    )
    .await?;
    let total_due = ttc + other_fees + *level_fee;
    let today = chrono::Utc::now().naive_utc().date();
    let days = days_overdue(invoice.due_date, today);

    let template = email_templates::get_effective(
        &state.pool,
        company.id,
        EmailTemplateType::InvoiceReminder,
        language,
        level_number,
    )
    .await?;
    let vars = build_reminder_vars(
        invoice,
        lines,
        contact,
        company,
        language,
        level_number,
        level_fee,
        &total_due,
        days,
    );
    let subject = kesh_core::email_template_engine::render(&template.subject, &vars);
    let body = kesh_core::email_template_engine::render(&template.body, &vars);
    Ok((subject, body))
}

/// `GET /api/v1/invoices/{id}/reminder-preview?level=N` — Comptable+ (Story 21-5b).
pub async fn preview_reminder_email(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    axum::extract::Query(q): axum::extract::Query<ReminderLevelQuery>,
) -> Result<Json<ReminderPreviewResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    if q.level < 1 {
        return Err(AppError::Validation(
            "niveau de rappel invalide".to_string(),
        ));
    }
    let level = dunning_levels::find_by_level_number(&state.pool, company.id, q.level)
        .await?
        .ok_or(AppError::DunningLevelNotFound)?;

    let (invoice, lines) = invoices::find_by_id_with_lines(&state.pool, company.id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;
    let contact = load_active_contact(&state.pool, invoice.contact_id, company.id).await?;
    let language = resolve_language(&contact, &company);

    let (subject, body) = render_reminder(
        &state,
        &company,
        &invoice,
        &lines,
        &contact,
        language,
        q.level,
        &level.fee_amount,
    )
    .await?;

    Ok(Json(ReminderPreviewResponse {
        to: locked_recipient(&contact),
        language,
        level: q.level,
        subject,
        body,
    }))
}

/// `POST /api/v1/invoices/{id}/reminders/send` — Comptable+ (Story 21-5b).
///
/// **Ordre corrigé C1** : toutes les vérifications qui peuvent rejeter la requête
/// (rate-limit, SMTP, éligibilité + niveau sous verrou bref) sont AVANT le SMTP ;
/// après le SMTP l'e-mail est parti → enregistrement best-effort (jamais de rejet
/// métier post-envoi ; échec d'enregistrement → trace best-effort + 409).
pub async fn send_reminder(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<SendReminderRequest>,
) -> Result<
    (
        StatusCode,
        Json<crate::routes::dunning_reminders::ReminderResponse>,
    ),
    AppError,
> {
    let company = get_company_for(&current_user, &state.pool).await?;

    // Rate-limit (consomme un slot).
    if let Err(reject) = state
        .rate_limiter_send_email
        .check_and_record((company.id, current_user.user_id))
    {
        return Err(AppError::RateLimited {
            retry_after: reject.retry_after_secs,
        });
    }
    if !state.smtp_ready {
        return Err(AppError::SmtpNotConfigured);
    }
    if req.level_number < 1 {
        return Err(AppError::Validation(
            "niveau de rappel invalide".to_string(),
        ));
    }

    // Frais du niveau visé (lecture hors verrou).
    let level = dunning_levels::find_by_level_number(&state.pool, company.id, req.level_number)
        .await?
        .ok_or(AppError::DunningLevelNotFound)?;
    let levels = dunning_levels::list_all_by_company(&state.pool, company.id).await?;

    // (a) VERROU BREF pré-SMTP : re-check éligibilité + niveau, puis relâche.
    {
        let mut tx = state
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;
        let invoice = invoices::find_scoped_for_update_in_tx(&mut tx, company.id, id)
            .await?
            .ok_or(AppError::Database(DbError::NotFound))?;
        if invoice.status != "validated" {
            return Err(AppError::InvoiceNotValidated);
        }
        if invoice.paid_at.is_some() {
            return Err(AppError::InvoiceAlreadyPaid);
        }
        if invoice.dunning_paused_at.is_some() {
            return Err(AppError::DunningPaused);
        }
        let current = invoice_reminders::current_level_in_tx(&mut tx, company.id, id).await?;
        let next = next_reminder_level(&levels, current).unwrap_or(current + 1);
        // Unitaire : niveau ≤ prochain (ré-émission autorisée D18). Saut > prochain interdit.
        if req.level_number > next {
            return Err(AppError::LevelAlreadySent);
        }
        tx.commit()
            .await
            .map_err(|e| AppError::Internal(format!("commit tx: {e}")))?;
    }

    // Contact + destinataire verrouillé + contenu + PDF (hors verrou).
    let (invoice, _lines) = invoices::find_by_id_with_lines(&state.pool, company.id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;
    let contact = load_active_contact(&state.pool, invoice.contact_id, company.id).await?;
    let to = locked_recipient(&contact).ok_or(AppError::ContactEmailMissing)?;
    let subject = req.subject.trim().to_string();
    let body = req.body.trim().to_string();
    if subject.is_empty() || body.is_empty() {
        return Err(AppError::InvoiceEmailEmptyContent);
    }
    // Borne AVANT le SMTP (C1) : `invoice_reminders.subject`/`body` sont des colonnes
    // TEXT. Sans cette garde, un contenu trop long partait chez le débiteur puis
    // échouait à l'INSERT (MariaDB 1406) — envoi réel, aucune trace, et un re-essai
    // rejouait l'e-mail à l'identique indéfiniment (review Pass 3).
    validate_text_len(&subject, REMINDER_SUBJECT_MAX, "objet")?;
    validate_text_len(&body, REMINDER_BODY_MAX, "corps")?;
    let language = resolve_language(&contact, &company);
    let locale = kesh_i18n::Locale::from(language.as_str());
    let rendered =
        invoice_pdf_service::render(&state.pool, &state.i18n, locale, &company, id).await?;

    let email = OutgoingEmail {
        to: to.clone(),
        subject: subject.clone(),
        body: body.clone(),
        from_display_name: Some(company.name.clone()),
        reply_to: company.email.clone(),
        attachment: Some(EmailAttachment {
            filename: format!("facture-{}.pdf", rendered.filename_base),
            content_type: "application/pdf".to_string(),
            bytes: rendered.bytes,
        }),
    };

    // (b) SMTP — échec → 500, rien enregistré (e-mail pas parti).
    state.mailer.send_email(&email).await?;

    // (c) Enregistrement BEST-EFFORT post-SMTP (l'e-mail est parti).
    match record_reminder_email(
        &state,
        &current_user,
        company.id,
        id,
        req.level_number,
        &level.fee_amount,
        &to,
        &subject,
        &body,
    )
    .await
    {
        Ok(created) => {
            tracing::info!(
                company_id = company.id,
                invoice_id = id,
                level_number = req.level_number,
                channel = "email",
                to = %to,
                "rappel envoyé par e-mail"
            );
            Ok((StatusCode::CREATED, Json(created)))
        }
        Err(e) => {
            // L'e-mail EST parti : ne jamais le présenter comme un échec d'envoi.
            tracing::error!(invoice_id = id, error = %e, "rappel e-mail parti mais enregistrement échoué");
            let _ = best_effort_reminder_audit(&state, &current_user, id, req.level_number).await;
            // Ne pas diagnostiquer une facture disparue sur un hoquet DB, ni l'inverse.
            //
            // `insert_in_tx` fait un INSERT nu : si la facture a disparu entre le verrou
            // et l'enregistrement (#219), c'est la **FK** `invoice_reminders.invoice_id`
            // qui saute → MySQL 1452 → `ForeignKeyViolation`, jamais `NotFound` (review
            // Pass 4 — tester `NotFound` seul rendait `ReminderSentButInvoiceGone`
            // inatteignable et annonçait un hoquet sur une vraie suppression).
            match e {
                AppError::Database(DbError::NotFound | DbError::ForeignKeyViolation(_)) => {
                    Err(AppError::ReminderSentButInvoiceGone)
                }
                _ => Err(AppError::ReminderSentButNotRecorded),
            }
        }
    }
}

/// Enregistre un rappel `channel='email'` + audit `invoice.reminder_sent` dans une tx.
#[allow(clippy::too_many_arguments)]
async fn record_reminder_email(
    state: &AppState,
    current_user: &CurrentUser,
    company_id: i64,
    invoice_id: i64,
    level_number: i16,
    fee_amount: &rust_decimal::Decimal,
    to: &str,
    subject: &str,
    body: &str,
) -> Result<crate::routes::dunning_reminders::ReminderResponse, AppError> {
    use crate::audit::AuditActor;
    use kesh_db::entities::audit_log::NewAuditLogEntry;
    use kesh_db::entities::{NewInvoiceReminder, ReminderChannel};

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("tx: {e}")))?;
    let created = invoice_reminders::insert_in_tx(
        &mut tx,
        &NewInvoiceReminder {
            company_id,
            invoice_id,
            level_number,
            fee_amount: *fee_amount,
            sent_at: chrono::Utc::now().naive_utc(),
            channel: ReminderChannel::Email,
            sent_to: Some(to.to_string()),
            subject: subject.to_string(),
            body: body.to_string(),
            note: None,
            actor_user_id: Some(current_user.user_id),
        },
    )
    .await?;
    kesh_db::repositories::audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::from_current_user(
            current_user,
            "invoice.reminder_sent",
            "invoice",
            invoice_id,
            Some(serde_json::json!({
                "reminderId": created.id,
                "levelNumber": created.level_number,
                "channel": "email",
            })),
        ),
    )
    .await?;
    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("tx: {e}")))?;
    Ok(created.into())
}

/// Trace best-effort qu'un e-mail de rappel est parti (utilisé quand l'enregistrement
/// principal échoue — la facture a disparu #219 ou hoquet DB). Calqué `EmailSentInvoiceGone`.
async fn best_effort_reminder_audit(
    state: &AppState,
    current_user: &CurrentUser,
    invoice_id: i64,
    level_number: i16,
) -> Result<(), AppError> {
    use crate::audit::AuditActor;
    use kesh_db::entities::audit_log::NewAuditLogEntry;
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("tx: {e}")))?;
    kesh_db::repositories::audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::from_current_user(
            current_user,
            "invoice.reminder_sent",
            "invoice",
            invoice_id,
            Some(serde_json::json!({
                "levelNumber": level_number,
                "channel": "email",
                "recordFailed": true,
            })),
        ),
    )
    .await?;
    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("tx: {e}")))?;
    Ok(())
}

/// `POST /api/v1/invoices/{id}/send-email` — Comptable+.
///
/// Séquence de gardes (AC #15 Story 20-3b1, dans cet ordre) :
/// auth/tenant → rate-limit 429 → SMTP prêt 412 → facture scopée 404 →
/// e-mail contact 400 → contenu vide 422 → rendu PDF (erreurs héritées
/// 20-3a) → envoi SMTP (échec 500, facture NON marquée) → marquage
/// `emailed_at` + audit → 200.
pub async fn send_invoice_email(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<SendInvoiceEmailRequest>,
) -> Result<(StatusCode, Json<InvoiceResponse>), AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;

    // Rate-limit par (company, user) — chaque tentative consomme un slot
    // (sémantique check_and_record, comme le recovery).
    if let Err(reject) = state
        .rate_limiter_send_email
        .check_and_record((company.id, current_user.user_id))
    {
        tracing::warn!(
            company_id = company.id,
            user_id = current_user.user_id,
            retry_after = reject.retry_after_secs,
            "send-email rate limit triggered"
        );
        return Err(AppError::RateLimited {
            retry_after: reject.retry_after_secs,
        });
    }

    // Garde 412 : sans SMTP prêt, le NoopMailer retournerait Ok et la
    // facture serait marquée « envoyée » à tort (envoi fantôme).
    if !state.smtp_ready {
        return Err(AppError::SmtpNotConfigured);
    }

    // Facture scopée company (anti-IDOR) + contact actif scopé company.
    // `lines` réutilisées pour la réponse finale : une facture validée est
    // immuable côté lignes (seul un brouillon s'édite) — pas de re-fetch
    // post-marquage (review Pass 2 BH2-1).
    let (invoice, lines) = invoices::find_by_id_with_lines(&state.pool, company.id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;
    let contact = load_active_contact(&state.pool, invoice.contact_id, company.id).await?;

    // Destinataire VERROUILLÉ = contacts.email (décision #13 epic-20).
    let to = locked_recipient(&contact).ok_or(AppError::ContactEmailMissing)?;

    let subject = req.subject.trim().to_string();
    let body = req.body.trim().to_string();
    if subject.is_empty() || body.is_empty() {
        return Err(AppError::InvoiceEmailEmptyContent);
    }

    // Rendu PDF dans la langue du contact (le corps ET la pièce jointe
    // partagent la même locale — décision #11 epic-20). Erreurs héritées
    // 20-3a inchangées (validated, ≤ 9 lignes, adresse, banque primary).
    let language = resolve_language(&contact, &company);
    let locale = kesh_i18n::Locale::from(language.as_str());
    let rendered =
        invoice_pdf_service::render(&state.pool, &state.i18n, locale, &company, id).await?;

    let email = OutgoingEmail {
        to: to.clone(),
        subject: subject.clone(),
        body,
        // From = KESH_SMTP_FROM avec display-name société ; Reply-To =
        // e-mail société si renseigné (décision #2 epic-20 + L20-1).
        from_display_name: Some(company.name.clone()),
        reply_to: company.email.clone(),
        attachment: Some(EmailAttachment {
            filename: format!("facture-{}.pdf", rendered.filename_base),
            content_type: "application/pdf".to_string(),
            bytes: rendered.bytes,
        }),
    };

    // Échec SMTP → 500 SMTP_SEND_FAILED, facture NON marquée (décision #16).
    state.mailer.send_email(&email).await?;

    // Marquage + audit invoice.emailed (même tx, repository). À partir d'ici
    // l'e-mail est PARTI : un échec de marquage ne doit plus se présenter
    // comme un échec d'envoi (review Pass 1 ECH-1/BH-3 — facture supprimée
    // #219 pendant l'envoi → 404 trompeur → renvoi en double par l'appelant).
    let updated = match invoices::mark_emailed(
        &state.pool,
        company.id,
        id,
        &to,
        &subject,
        current_user.user_id,
        current_user.api_key_id,
    )
    .await
    {
        Ok(updated) => updated,
        Err(DbError::NotFound) => {
            tracing::error!(
                company_id = company.id,
                invoice_id = id,
                to = %to,
                "e-mail de facture envoyé mais facture disparue avant le marquage — trace audit best-effort"
            );
            // Trace comptable best-effort : l'e-mail avec le PDF est parti,
            // il DOIT en rester une trace même sans row invoices (audit_log
            // n'a pas de FK sur entity_id). Un échec de CETTE écriture est
            // loggué explicitement (review Pass 2 BH2-2) — sans faire
            // échouer davantage la requête (le 409 part quoi qu'il arrive).
            let audit_result = async {
                let mut tx = state
                    .pool
                    .begin()
                    .await
                    .map_err(kesh_db::errors::DbError::Sqlx)?;
                let entry = kesh_db::entities::NewAuditLogEntry::for_actor(
                    current_user.user_id,
                    current_user.api_key_id,
                    "invoice.emailed".to_string(),
                    "invoice".to_string(),
                    id,
                    Some(serde_json::json!({
                        "to": to,
                        "subject": subject,
                        "invoiceGone": true,
                    })),
                );
                kesh_db::repositories::audit_log::insert_in_tx(&mut tx, entry).await?;
                tx.commit().await.map_err(kesh_db::errors::DbError::Sqlx)?;
                Ok::<(), kesh_db::errors::DbError>(())
            }
            .await;
            if let Err(audit_err) = audit_result {
                tracing::error!(
                    company_id = company.id,
                    invoice_id = id,
                    error = %audit_err,
                    "écriture de la trace d'audit best-effort échouée — envoi non tracé"
                );
            }
            return Err(AppError::EmailSentInvoiceGone);
        }
        Err(e) => return Err(e.into()),
    };

    // AC 9 (21-5b) — rétrofit : l'envoi de facture d'Epic 20 ne laissait aucune trace
    // dans le log fichier sur le chemin de succès (seulement warn/error).
    tracing::info!(
        company_id = company.id,
        invoice_id = id,
        channel = "email",
        to = %to,
        "facture envoyée par e-mail"
    );

    Ok((
        StatusCode::OK,
        Json(InvoiceResponse::from_parts(updated, lines)),
    ))
}

// ===========================================================================
// Envoi par lot (Story 21-5b) — pattern batch { accepted, failed }
// ===========================================================================

/// Cap dur du nombre de factures par lot (item 19 — borne la durée HTTP).
const REMINDER_BATCH_MAX: usize = 20;

/// Bornes de `invoice_reminders.subject`/`body`, colonnes `TEXT` (65 535 **octets**).
///
/// Exprimées en caractères : le pire cas UTF-8 (4 octets/caractère) reste sous la
/// borne DB (500×4 = 2 000 ; 10 000×4 = 40 000), donc l'INSERT est garanti quel que
/// soit le contenu. Largement au-dessus d'un rappel réaliste.
const REMINDER_SUBJECT_MAX: usize = 500;
const REMINDER_BODY_MAX: usize = 10_000;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendReminderBatchRequest {
    pub invoice_ids: Vec<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptedReminder {
    pub invoice_id: i64,
    pub reminder_id: i64,
    pub level_number: i16,
}

/// Échec per-facture (pattern `FailedProposal` — id business = `invoice_id`).
///
/// Signature canonique Epic 8 (cf. `reconciliation::FailedProposal`) : identifiant
/// business + `error_code` constant + `details` optionnel pour le contexte dynamique
/// (jamais d'interpolation dans `error_code`).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedReminder {
    pub invoice_id: i64,
    pub error_code: String,
    pub details: Option<serde_json::Value>,
}

/// Échec d'UNE facture d'un lot → `FailedProposal`.
///
/// **Aucune erreur rencontrée dans la boucle per-facture n'escalade en `AppError`
/// global** (décision review Pass 2). Les exceptions 500 de CLAUDE.md §"Pattern
/// batch" visent les erreurs qui invalident la requête « en amont du traitement
/// per-proposal » — et elles sont bien attrapées en amont, par les `?` qui
/// précèdent la boucle (`get_company_for`, `list_all_by_company`) : un pool déjà
/// mort y échoue avant qu'aucun e-mail ne parte.
///
/// Escalader depuis l'intérieur de la boucle jetterait `accepted` alors que ces
/// e-mails sont **réellement partis** — le client ne saurait plus qui a été
/// relancé. Une panne survenant en cours de lot reste donc per-facture, mais est
/// journalisée en `error!` (cf. [`BatchItemError::infra`]) pour rester visible à
/// l'exploitant.
struct BatchItemError {
    error_code: String,
    details: Option<serde_json::Value>,
}

impl BatchItemError {
    /// Échec métier per-facture sans contexte additionnel.
    fn failed(error_code: &str) -> Self {
        Self {
            error_code: error_code.to_string(),
            details: None,
        }
    }

    /// Échec métier per-facture avec contexte JSON (`details`).
    fn failed_with(error_code: &str, details: serde_json::Value) -> Self {
        Self {
            error_code: error_code.to_string(),
            details: Some(details),
        }
    }

    /// Le code d'erreur porté, pour les tests.
    #[cfg(test)]
    fn code(&self) -> &str {
        &self.error_code
    }

    /// Erreur d'**infrastructure** survenue en cours de lot (pool mort, IO).
    ///
    /// Reste per-facture pour préserver `accepted` (cf. doc du type), mais loggée
    /// en `error!` : sans ça, une panne se déguiserait en simple échec métier dans
    /// un HTTP 200 et resterait invisible.
    fn infra(context: &str, invoice_id: i64, e: impl std::fmt::Display) -> Self {
        tracing::error!(
            invoice_id,
            context,
            error = %e,
            "erreur d'infrastructure en cours de lot — facture écartée, lot poursuivi"
        );
        Self::failed("DATABASE_ERROR")
    }
}

/// Classe une erreur de rendu PDF en échec **métier** per-facture ou en panne
/// d'**infrastructure**, pour la boucle du lot de rappels.
///
/// Extraite du handler en passe 7 de revue de la Story 16-3a : tant qu'elle était
/// écrite en ligne, **aucun test ne pouvait l'atteindre**, et c'est précisément là
/// qu'un variant d'erreur ajouté par cette story avait été oublié.
///
/// La règle qui gouverne cette fonction, et qu'il faut tenir à chaque ajout de
/// variant : les causes **métier** — celles qui dépendent des seules données de la
/// facture et que l'utilisateur peut corriger — sont énumérées **explicitement** ;
/// le bras final est réservé aux **pannes** (pool mort, IO), qu'il journalise en
/// `error!`. Un refus déterministe tombé dans le bras final ressort en
/// `DATABASE_ERROR` : le client ne sait pas quoi corriger, et l'exploitant reçoit
/// une fausse alerte d'infrastructure. C'est aussi ce qu'impose la
/// § *Pattern batch — FailedProposal per-proposal* du `CLAUDE.md`.
fn classify_render_error(e: AppError, invoice_id: i64) -> BatchItemError {
    match e {
        AppError::InvoiceNotValidated => BatchItemError::failed("INVOICE_NOT_VALIDATED"),
        AppError::Database(DbError::NotFound) => BatchItemError::failed("INVOICE_NOT_FOUND"),
        // Données de facturation incomplètes (pas de compte bancaire, adresse QR
        // invalide, trop de lignes, en-tête débordant) — le PDF ne peut pas être
        // produit, mais rien n'est cassé.
        AppError::InvoiceNotPdfReady(_)
        | AppError::InvoiceTooManyLinesForPdf(_)
        | AppError::InvoicePdfHeaderOverflow => BatchItemError::failed("INVOICE_NOT_PDF_READY"),
        other => BatchItemError::infra("render pdf", invoice_id, other),
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendReminderBatchResponse {
    pub accepted: Vec<AcceptedReminder>,
    pub failed: Vec<FailedReminder>,
}

/// `POST /api/v1/dunning/reminders/send-batch` — Comptable+ (Story 21-5b).
/// Envoie le **prochain niveau** à une liste de factures. Succès partiel = HTTP 200.
pub async fn send_reminder_batch(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<SendReminderBatchRequest>,
) -> Result<(StatusCode, Json<SendReminderBatchResponse>), AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;

    // Dédup des ids en préservant l'ordre.
    let mut seen = std::collections::HashSet::new();
    let ids: Vec<i64> = req
        .invoice_ids
        .iter()
        .copied()
        .filter(|id| seen.insert(*id))
        .collect();

    // Cap dur (après dédup) — 422 global AVANT tout SMTP.
    if ids.len() > REMINDER_BATCH_MAX {
        return Err(AppError::BatchTooLarge);
    }
    if !state.smtp_ready {
        return Err(AppError::SmtpNotConfigured);
    }
    // Un lot plus grand que le quota d'envoi ne pourra JAMAIS passer : la fenêtre ne
    // libère au mieux que `max_attempts` slots. Renvoyer 429 ici ferait réessayer un
    // client obéissant indéfiniment pour rien — c'est 422 (review Pass 3).
    let quota = state.rate_limiter_send_email.max_attempts();
    if ids.len() as u32 > quota {
        return Err(AppError::BatchExceedsSendQuota { max: quota });
    }

    // Pré-check de capacité rate-limit — 429 global AVANT le 1er SMTP (pas de blocage mi-course).
    let key = (company.id, current_user.user_id);
    if (ids.len() as u32) > state.rate_limiter_send_email.remaining_slots(key) {
        // Délai réel (blocage restant, ou sortie de fenêtre glissante) — un `1` en dur
        // ferait marteler l'endpoint chaque seconde pendant tout le blocage.
        return Err(AppError::RateLimited {
            retry_after: state
                .rate_limiter_send_email
                .retry_after_secs(key, ids.len() as u32)
                .max(1),
        });
    }

    let levels = dunning_levels::list_all_by_company(&state.pool, company.id).await?;

    let mut accepted = Vec::new();
    let mut failed = Vec::new();
    for invoice_id in ids {
        match send_one_batch_reminder(&state, &current_user, &company, invoice_id, &levels).await {
            Ok(acc) => accepted.push(acc),
            Err(item) => failed.push(FailedReminder {
                invoice_id,
                error_code: item.error_code,
                details: item.details,
            }),
        }
    }

    Ok((
        StatusCode::OK,
        Json(SendReminderBatchResponse { accepted, failed }),
    ))
}

/// Envoie le prochain niveau à UNE facture d'un lot.
///
/// `Err(_)` = `FailedProposal` → `failed[]` + HTTP 200, jamais d'escalade globale
/// (cf. doc de [`BatchItemError`]). Gardes pré-SMTP sous verrou → SMTP → enregistrement.
async fn send_one_batch_reminder(
    state: &AppState,
    current_user: &CurrentUser,
    company: &Company,
    invoice_id: i64,
    levels: &[kesh_db::entities::DunningLevel],
) -> Result<AcceptedReminder, BatchItemError> {
    // (a) Gardes pré-SMTP sous verrou bref.
    let next_level = {
        let mut tx = state
            .pool
            .begin()
            .await
            .map_err(|e| BatchItemError::infra("begin tx", invoice_id, e))?;
        let invoice = match invoices::find_scoped_for_update_in_tx(&mut tx, company.id, invoice_id)
            .await
            .map_err(|e| BatchItemError::infra("find invoice for update", invoice_id, e))?
        {
            Some(inv) => inv,
            // Absente ou cross-tenant — même code, pas de fuite d'existence.
            None => return Err(BatchItemError::failed("INVOICE_NOT_FOUND")),
        };
        if invoice.status != "validated" {
            return Err(BatchItemError::failed("INVOICE_NOT_VALIDATED"));
        }
        if invoice.paid_at.is_some() {
            return Err(BatchItemError::failed("INVOICE_ALREADY_PAID"));
        }
        if invoice.dunning_paused_at.is_some() {
            return Err(BatchItemError::failed("DUNNING_PAUSED"));
        }
        let current = invoice_reminders::current_level_in_tx(&mut tx, company.id, invoice_id)
            .await
            .map_err(|e| BatchItemError::infra("current level", invoice_id, e))?;
        let next = match next_reminder_level(levels, current) {
            Some(n) => n,
            // Dernier niveau atteint / dunning désactivé.
            None => {
                return Err(BatchItemError::failed_with(
                    "NO_NEXT_LEVEL",
                    serde_json::json!({ "currentLevel": current }),
                ));
            }
        };
        tx.commit()
            .await
            .map_err(|e| BatchItemError::infra("commit tx", invoice_id, e))?;
        next
    };

    // `next_reminder_level` sélectionne `next_level` DANS `levels` — ce `find` ne peut
    // donc pas échouer. Le bras d'erreur ne sert qu'à couvrir un refactor qui romprait
    // cet invariant : bug structurel, donc loggué (review Pass 3 — l'ancien bras
    // renvoyait un `NO_NEXT_LEVEL` mort, au `details` incompatible avec le vrai).
    let level_fee = levels
        .iter()
        .find(|l| l.level_number == next_level)
        .map(|l| l.fee_amount)
        .ok_or_else(|| {
            BatchItemError::infra(
                "niveau calculé absent de la config (invariant rompu)",
                invoice_id,
                format!("level {next_level}"),
            )
        })?;

    // Contact + destinataire + rendu serveur + PDF.
    let (invoice, lines) = invoices::find_by_id_with_lines(&state.pool, company.id, invoice_id)
        .await
        .map_err(|e| BatchItemError::infra("find invoice with lines", invoice_id, e))?
        .ok_or_else(|| BatchItemError::failed("INVOICE_NOT_FOUND"))?;
    // `|_| CONTACT_EMAIL_MISSING` mentirait : un contact archivé a le plus souvent
    // un e-mail — le problème est ailleurs (review Pass 2). Le contact ne peut pas
    // avoir été supprimé (FK `invoices.contact_id` ON DELETE RESTRICT), donc un
    // `NotFound` ici signale une incohérence de données, pas un cas métier.
    let contact = load_active_contact(&state.pool, invoice.contact_id, company.id)
        .await
        .map_err(|e| match e {
            AppError::ContactArchived => BatchItemError::failed("CONTACT_ARCHIVED"),
            other => BatchItemError::infra("load contact", invoice_id, other),
        })?;
    let to = locked_recipient(&contact)
        .ok_or_else(|| BatchItemError::failed("CONTACT_EMAIL_MISSING"))?;
    let language = resolve_language(&contact, company);

    let (subject, body) = render_reminder(
        state, company, &invoice, &lines, &contact, language, next_level, &level_fee,
    )
    .await
    .map_err(|e| BatchItemError::infra("render reminder", invoice_id, e))?;
    if subject.trim().is_empty() || body.trim().is_empty() {
        return Err(BatchItemError::failed("REMINDER_CONTENT_EMPTY"));
    }
    // Même garde pré-SMTP que l'unitaire : un template surdimensionné ferait partir
    // l'e-mail puis échouer l'INSERT (colonnes TEXT). Moins atteignable ici (contenu
    // rendu serveur, pas fourni par le client) mais le chemin existe (review Pass 3).
    if exceeds_len(&subject, REMINDER_SUBJECT_MAX) || exceeds_len(&body, REMINDER_BODY_MAX) {
        return Err(BatchItemError::failed_with(
            "REMINDER_CONTENT_TOO_LONG",
            serde_json::json!({
                "subjectMax": REMINDER_SUBJECT_MAX,
                "bodyMax": REMINDER_BODY_MAX,
            }),
        ));
    }

    let locale = kesh_i18n::Locale::from(language.as_str());
    // Le rendu PDF échoue pour des causes distinctes — les préserver plutôt que de
    // toutes les écraser sur `INVOICE_NOT_PDF_READY` (AC 12 énumère les codes
    // séparément). Le bras final ne fourre PAS les pannes d'infra dans un code
    // métier (review Pass 2) : les causes métier sont énumérées explicitement.
    let rendered =
        invoice_pdf_service::render(&state.pool, &state.i18n, locale, company, invoice_id)
            .await
            .map_err(|e| classify_render_error(e, invoice_id))?;

    // Rate-limit : consomme 1 slot par e-mail (le pré-check a garanti la capacité, mais un
    // envoi concurrent peut avoir consommé entre-temps → RATE_LIMITED per-facture).
    if let Err(reject) = state
        .rate_limiter_send_email
        .check_and_record((company.id, current_user.user_id))
    {
        return Err(BatchItemError::failed_with(
            "RATE_LIMITED",
            serde_json::json!({ "retryAfterSecs": reject.retry_after_secs }),
        ));
    }

    let email = OutgoingEmail {
        to: to.clone(),
        subject: subject.clone(),
        body: body.clone(),
        from_display_name: Some(company.name.clone()),
        reply_to: company.email.clone(),
        attachment: Some(EmailAttachment {
            filename: format!("facture-{}.pdf", rendered.filename_base),
            content_type: "application/pdf".to_string(),
            bytes: rendered.bytes,
        }),
    };

    // (b) SMTP. L'erreur est journalisée ici : contrairement à l'unitaire, elle ne
    // remonte pas jusqu'à `IntoResponse` (qui loggue `SmtpSendFailed`), donc sans ça
    // une panne SMTP produisait 20 échecs dans un HTTP 200 et zéro ligne de log
    // (review Pass 3). Le code reste per-facture — AC 12 l'impose.
    if let Err(e) = state.mailer.send_email(&email).await {
        tracing::error!(invoice_id, error = %e, "échec SMTP en cours de lot");
        return Err(BatchItemError::failed("SMTP_SEND_FAILED"));
    }

    // (c) Enregistrement best-effort post-SMTP.
    match record_reminder_email(
        state,
        current_user,
        company.id,
        invoice_id,
        next_level,
        &level_fee,
        &to,
        &subject,
        &body,
    )
    .await
    {
        Ok(created) => {
            tracing::info!(
                company_id = company.id,
                invoice_id,
                level_number = next_level,
                channel = "email",
                to = %to,
                "rappel (lot) envoyé"
            );
            Ok(AcceptedReminder {
                invoice_id,
                reminder_id: created.id,
                level_number: next_level,
            })
        }
        Err(e) => {
            // L'e-mail EST parti : jamais d'escalade globale ici, sinon on perdrait
            // les factures déjà acceptées du lot. Reste un échec per-facture.
            tracing::error!(invoice_id, error = %e, "rappel lot parti mais enregistrement échoué");
            let _ = best_effort_reminder_audit(state, current_user, invoice_id, next_level).await;
            // Même distinction que l'unitaire (review Pass 5) : « facture disparue »
            // et « hoquet DB » n'appellent pas la même action au dossier de
            // recouvrement. La FK saute quand la facture a réellement disparu.
            match e {
                AppError::Database(DbError::NotFound | DbError::ForeignKeyViolation(_)) => {
                    Err(BatchItemError::failed("REMINDER_SENT_BUT_INVOICE_GONE"))
                }
                _ => Err(BatchItemError::failed("RECORD_FAILED_EMAIL_SENT")),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;

    // ------------------------------------------------------------------
    // salutation_line — matrice complète (AC #18)
    // ------------------------------------------------------------------

    #[test]
    fn salutation_personne_monsieur_avec_nom() {
        let cases = [
            (Language::Fr, "Cher Monsieur Dupont"),
            (Language::De, "Sehr geehrter Herr Dupont"),
            (Language::It, "Egregio Signor Dupont"),
            (Language::En, "Dear Mr Dupont"),
        ];
        for (lang, expected) in cases {
            assert_eq!(
                salutation_line(
                    Salutation::Monsieur,
                    ContactType::Personne,
                    Some("Dupont"),
                    lang
                ),
                expected
            );
        }
    }

    #[test]
    fn salutation_personne_madame_avec_nom() {
        let cases = [
            (Language::Fr, "Chère Madame Dupont"),
            (Language::De, "Sehr geehrte Frau Dupont"),
            (Language::It, "Gentile Signora Dupont"),
            (Language::En, "Dear Ms Dupont"),
        ];
        for (lang, expected) in cases {
            assert_eq!(
                salutation_line(
                    Salutation::Madame,
                    ContactType::Personne,
                    Some("Dupont"),
                    lang
                ),
                expected
            );
        }
    }

    #[test]
    fn salutation_personne_sans_nom() {
        assert_eq!(
            salutation_line(
                Salutation::Monsieur,
                ContactType::Personne,
                None,
                Language::Fr
            ),
            "Cher Monsieur"
        );
        assert_eq!(
            salutation_line(
                Salutation::Monsieur,
                ContactType::Personne,
                Some("   "),
                Language::It
            ),
            "Egregio Signore",
            "nom whitespace-only = absent"
        );
        assert_eq!(
            salutation_line(
                Salutation::Madame,
                ContactType::Personne,
                None,
                Language::En
            ),
            "Dear Madam"
        );
    }

    #[test]
    fn salutation_neutre_et_entreprise() {
        // Neutre → formule neutre, même avec un nom.
        assert_eq!(
            salutation_line(
                Salutation::Neutre,
                ContactType::Personne,
                Some("Dupont"),
                Language::Fr
            ),
            "Madame, Monsieur"
        );
        // Entreprise → formule neutre, même avec civilité genrée.
        let cases = [
            (Language::Fr, "Madame, Monsieur"),
            (Language::De, "Sehr geehrte Damen und Herren"),
            (Language::It, "Gentili Signore e Signori"),
            (Language::En, "Dear Sir or Madam"),
        ];
        for (lang, expected) in cases {
            assert_eq!(
                salutation_line(
                    Salutation::Monsieur,
                    ContactType::Entreprise,
                    Some("Dupont"),
                    lang
                ),
                expected
            );
        }
    }

    // ------------------------------------------------------------------
    // build_invoice_vars — fallbacks et formatage suisse (AC #18)
    // ------------------------------------------------------------------

    fn sample_invoice(number: Option<&str>, due: Option<NaiveDate>) -> kesh_db::entities::Invoice {
        kesh_db::entities::Invoice {
            id: 7,
            company_id: 1,
            contact_id: 2,
            invoice_number: number.map(String::from),
            status: "validated".to_string(),
            date: NaiveDate::from_ymd_opt(2026, 7, 1).expect("date"),
            due_date: due,
            payment_terms: None,
            total_amount: Decimal::new(123_456, 2), // 1234.56
            journal_entry_id: Some(1),
            paid_at: None,
            emailed_at: None,
            emailed_to: None,
            project_id: None,
            dunning_paused_at: None,
            dunning_paused_note: None,
            version: 1,
            created_at: chrono::NaiveDateTime::default(),
            updated_at: chrono::NaiveDateTime::default(),
        }
    }

    fn sample_contact() -> Contact {
        Contact {
            id: 2,
            company_id: 1,
            contact_type: ContactType::Personne,
            name: "Jean Dupont".to_string(),
            first_name: Some("Jean".to_string()),
            last_name: Some("Dupont".to_string()),
            is_client: true,
            is_supplier: false,
            address: None,
            address_street: None,
            address_building: None,
            address_postal_code: None,
            address_city: None,
            address_country: None,
            email: Some("jean@example.ch".to_string()),
            phone: None,
            ide_number: None,
            default_payment_terms: None,
            default_payment_terms_days: None,
            language: Some(Language::De),
            salutation: Salutation::Monsieur,
            active: true,
            version: 1,
            created_at: chrono::NaiveDateTime::default(),
            updated_at: chrono::NaiveDateTime::default(),
        }
    }

    fn sample_company() -> Company {
        Company {
            id: 1,
            name: "Ma PME SA".to_string(),
            first_name: None,
            last_name: None,
            address: "Rue Test 1\n1000 Lausanne".to_string(),
            address_street: "Rue Test".to_string(),
            address_building: "1".to_string(),
            address_postal_code: "1000".to_string(),
            address_city: "Lausanne".to_string(),
            address_country: "CH".to_string(),
            ide_number: None,
            org_type: kesh_db::entities::company::OrgType::Pme,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
            email: Some("info@mapme.ch".to_string()),
            phone: None,
            website: None,
            is_stub: false,
            version: 1,
            created_at: chrono::NaiveDateTime::default(),
            updated_at: chrono::NaiveDateTime::default(),
        }
    }

    /// Lignes de la facture d'exemple (#246 : `{amount}` = TTC dérivé des
    /// lignes) : 1 ligne HT 1234.56 @ 8.1 % — TVA = 99.99936 → 100.00
    /// (arrondi centime), TTC = 1334.56.
    fn sample_lines() -> Vec<kesh_db::entities::InvoiceLine> {
        vec![kesh_db::entities::InvoiceLine {
            revenue_account_id: None,
            id: 1,
            invoice_id: 7,
            position: 1,
            description: "Ligne".to_string(),
            quantity: Decimal::ONE,
            unit_price: Decimal::new(123_456, 2),
            vat_rate: Decimal::new(810, 2),       // 8.10 %
            line_total: Decimal::new(123_456, 2), // 1234.56
            created_at: chrono::NaiveDateTime::default(),
        }]
    }

    #[test]
    fn vars_complete_et_formatage_suisse() {
        let invoice = sample_invoice(Some("F-2026-0042"), NaiveDate::from_ymd_opt(2026, 7, 31));
        let contact = sample_contact();
        let company = sample_company();
        let vars = build_invoice_vars(&invoice, &sample_lines(), &contact, &company, Language::De);

        // Les 6 variables déclarées par InvoiceSend, exactement.
        let mut keys: Vec<&str> = vars.keys().map(String::as_str).collect();
        keys.sort_unstable();
        let mut expected = EmailTemplateType::InvoiceSend.allowed_variables().to_vec();
        expected.sort_unstable();
        assert_eq!(keys, expected);

        assert_eq!(vars["invoiceNumber"], "F-2026-0042");
        // #246 : {amount} = TTC (1234.56 + TVA 8.1 % = 1234.56 + 100.00 = 1334.56),
        // plus le HT — line_vat_amount(1234.56, 8.1) = 99.99936 → 100.00.
        assert_eq!(vars["amount"], "1\u{2019}334.56", "TTC, apostrophe U+2019");
        assert_eq!(vars["dueDate"], "31.07.2026", "format dd.mm.yyyy");
        assert_eq!(vars["companyName"], "Ma PME SA");
        assert_eq!(vars["contactName"], "Jean Dupont");
        assert_eq!(vars["salutation"], "Sehr geehrter Herr Dupont");
    }

    #[test]
    fn vars_fallbacks_number_et_due_date() {
        let invoice = sample_invoice(None, None);
        let vars = build_invoice_vars(
            &invoice,
            &sample_lines(),
            &sample_contact(),
            &sample_company(),
            Language::Fr,
        );
        assert_eq!(vars["invoiceNumber"], "#7", "fallback #id comme le PDF");
        assert_eq!(vars["dueDate"], "—", "échéance absente → tiret");
    }

    #[test]
    fn resolve_language_contact_prime_sinon_instance() {
        let company = sample_company(); // instance FR
        let mut contact = sample_contact(); // language DE
        assert_eq!(resolve_language(&contact, &company), Language::De);
        contact.language = None;
        assert_eq!(resolve_language(&contact, &company), Language::Fr);
    }

    #[test]
    fn reminder_vars_ajoute_les_4_variables_rappel() {
        let invoice = sample_invoice(Some("F-2026-0042"), NaiveDate::from_ymd_opt(2026, 6, 30));
        let vars = build_reminder_vars(
            &invoice,
            &sample_lines(),
            &sample_contact(),
            &sample_company(),
            Language::Fr,
            2,
            &Decimal::new(2000, 2),   // 20.00 (frais niveau 2)
            &Decimal::new(135456, 2), // 1354.56 (total dû)
            45,
        );
        // 10 variables = les 6 base + 4 rappel (allowed_variables InvoiceReminder).
        let mut keys: Vec<&str> = vars.keys().map(String::as_str).collect();
        keys.sort_unstable();
        let mut expected = EmailTemplateType::InvoiceReminder
            .allowed_variables()
            .to_vec();
        expected.sort_unstable();
        assert_eq!(keys, expected);
        assert_eq!(vars["reminderLevel"], "2");
        assert_eq!(vars["reminderFee"], "20.00");
        assert_eq!(vars["totalDue"], "1\u{2019}354.56", "apostrophe U+2019");
        assert_eq!(vars["daysOverdue"], "45", "entier brut, pas format_money");
        assert_eq!(vars["amount"], "1\u{2019}334.56", "TTC facture inchangé");
    }

    #[test]
    fn days_overdue_clampe_et_gere_none() {
        let today = NaiveDate::from_ymd_opt(2026, 7, 16).unwrap();
        assert_eq!(
            days_overdue(NaiveDate::from_ymd_opt(2026, 6, 16), today),
            30
        );
        assert_eq!(
            days_overdue(NaiveDate::from_ymd_opt(2026, 8, 1), today),
            0,
            "échéance future → 0 (clampé)"
        );
        assert_eq!(days_overdue(None, today), 0, "pas d'échéance → 0");
    }

    // ------------------------------------------------------------------
    // classify_render_error — un refus de rendu n'est pas une panne
    // (Story 16-3a #151, passe 7 de revue)
    // ------------------------------------------------------------------

    /// Verrouille la frontière métier / infrastructure du lot de rappels.
    ///
    /// Le défaut qui a motivé ce test : `InvoicePdfHeaderOverflow`, variante
    /// ajoutée par la Story 16-3a, n'avait pas été déclarée dans le `match` et
    /// tombait dans le bras final. Elle ressortait donc en `DATABASE_ERROR`
    /// **avec un `tracing::error!` « erreur d'infrastructure »**, pour un refus
    /// entièrement déterministe que l'utilisateur peut corriger lui-même. Le
    /// client n'avait aucun moyen de savoir quoi corriger, et l'exploitant
    /// recevait une fausse alerte de panne.
    ///
    /// Le test couvre les DEUX sens : les causes métier rendent leur code
    /// propre, et une vraie panne continue de rendre `DATABASE_ERROR`. Sans ce
    /// second volet, déclarer tous les variants métier ferait passer le test
    /// tout en supprimant la détection des pannes.
    #[test]
    fn classify_render_error_ne_deguise_pas_un_refus_en_panne() {
        let metier = [
            (AppError::InvoiceNotValidated, "INVOICE_NOT_VALIDATED"),
            (AppError::Database(DbError::NotFound), "INVOICE_NOT_FOUND"),
            (
                AppError::InvoiceNotPdfReady("adresse incomplète".into()),
                "INVOICE_NOT_PDF_READY",
            ),
            (
                AppError::InvoiceTooManyLinesForPdf(42),
                "INVOICE_NOT_PDF_READY",
            ),
            (AppError::InvoicePdfHeaderOverflow, "INVOICE_NOT_PDF_READY"),
        ];
        for (err, attendu) in metier {
            let libelle = format!("{err:?}");
            assert_eq!(
                classify_render_error(err, 1).code(),
                attendu,
                "{libelle} est une cause MÉTIER — elle ne doit pas ressortir en DATABASE_ERROR"
            );
        }

        // Une panne réelle doit continuer d'être signalée comme telle.
        assert_eq!(
            classify_render_error(AppError::Internal("pool fermé".into()), 1).code(),
            "DATABASE_ERROR",
            "une panne d'infrastructure doit rester détectée comme telle"
        );
    }
}
