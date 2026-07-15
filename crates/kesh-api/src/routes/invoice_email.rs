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
use kesh_db::repositories::{contacts, email_templates, invoices};
use kesh_i18n::{format_date, format_money};

use crate::AppState;
use crate::errors::AppError;
use crate::helpers::get_company_for;
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

    Ok((
        StatusCode::OK,
        Json(InvoiceResponse::from_parts(updated, lines)),
    ))
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
}
