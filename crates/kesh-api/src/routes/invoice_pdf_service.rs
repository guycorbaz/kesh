//! Story 20.3a — service de génération du PDF QR-facture, factorisé depuis
//! le handler HTTP `get_invoice_pdf` (Story 5.3) pour être réutilisable par
//! l'envoi de facture par e-mail (Story 20-3b, PDF en pièce jointe).
//!
//! Charge la facture validée (scopée à la company), les lignes, le contact,
//! le compte bancaire primary, puis délègue à `kesh-qrbill` pour produire le
//! PDF. La `locale` est un paramètre : la langue des documents du débiteur,
//! celle du contact ou, à défaut, `instance_language`. Le PDF de la facture,
//! téléchargé comme envoyé, passe par `issued_invoice_pdf` (`client_language`),
//! qui le fige ; les rappels, par `invoice_email` (`resolve_language`).
//!
//! Héberge aussi les helpers PDF partagés avec `credit_notes.rs`
//! (`MAX_LINES_PER_PDF`, `build_i18n`, `map_qrbill_error`,
//! `sanitize_filename`, `split_lines`) — couplage route→service plutôt que
//! route→route.

use kesh_db::entities::{BankAccount, Company, Invoice, InvoiceLine, contact::Contact};
use kesh_db::errors::DbError;
use kesh_db::repositories::{
    bank_accounts, contacts, invoice_reminders, invoice_settlements, invoices,
};
use kesh_i18n::Locale;
use kesh_qrbill::{
    Address, AddressType, Currency, InvoiceLinePdf, InvoicePdfData, InvoiceVatLinePdf, QrBillData,
    QrBillError, QrBillI18n, Reference, ReminderPdf,
    validation::{build_qrr, normalize_iban},
};
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::errors::AppError;

/// #151 : construit le récapitulatif TVA par taux pour le PDF à partir des
/// paires `(line_total_ht, vat_rate)` d'un document (facture ou avoir).
/// Délègue l'agrégation (arrondi par ligne DC7) à `kesh_core` — source unique,
/// pas de logique comptable dupliquée ici ni dans `kesh-qrbill`.
pub(crate) fn vat_lines_pdf(
    pairs: impl IntoIterator<Item = (Decimal, Decimal)>,
) -> Vec<InvoiceVatLinePdf> {
    kesh_core::accounting::vat::vat_breakdown_by_rate(pairs)
        .into_iter()
        .map(|b| InvoiceVatLinePdf {
            rate_percent: b.rate_percent,
            amount: b.vat_amount,
        })
        .collect()
}

/// Borne v0.1 (grossière) du nombre de lignes d'un PDF A4 mono-page.
///
/// C'est un **pré-filtre** : la vérification géométrique **précise** (et
/// autoritaire) est la garde de `kesh-qrbill::pdf::draw_invoice_section`, qui
/// tient compte du header ET du bloc récap TVA (#151) :
/// - `ty` de la 1re ligne = `159` mm (167 après header, − 5 − 3 pour la ligne
///   de titre du tableau et son filet)
/// - pas par ligne = `5` mm ; la check a lieu **avant** le draw
/// - seuil de refus = `SEP_Y + 15 (=120) + réserve_récap`, où `réserve_récap`
///   = `0` sans TVA, sinon `sous-total (4.5) + n_taux×4.5 + espace (1)`
/// - sans récap : `159 − (N-1)*5 >= 120` ⇒ **8 lignes** tiennent ; avec un
///   récap multi-taux, moins (le récap descend le curseur sous le total).
///
/// Le rendu est **défensif** : au-delà de la capacité (lignes + récap), la garde
/// renvoie `QrBillError::PdfGeneration` plutôt que de chevaucher la zone QR ou
/// tronquer. Cette constante (9) reste un plafond supérieur simple ; un cas qui
/// la passe mais ne tient pas géométriquement échoue proprement dans `pdf.rs`.
pub const MAX_LINES_PER_PDF: usize = 9;

/// PDF de facture généré, prêt à être servi (handler HTTP) ou attaché
/// (e-mail Story 20-3b).
pub struct RenderedInvoicePdf {
    /// Contenu binaire du PDF.
    pub bytes: Vec<u8>,
    /// Nom de fichier déjà sanitizé, sans extension ni chemin — ex. "F-2026-0042"
    /// (le caller ajoute le préfixe/extension de son Content-Disposition ou
    /// de son attachment). Dérivé de `invoice.invoice_number` via
    /// `sanitize_filename`.
    pub filename_base: String,
    /// Story 25-6-b (#387) — la `version` de la facture **lue pour ce rendu**.
    /// Le gel la pose en garde : une facture qui a changé pendant le rendu
    /// n'est jamais figée sur des données périmées.
    pub invoice_version: i32,
}

/// Le document à produire (Story 25-4-b2, #416) : la facture, ou un **rappel**
/// pour le montant restant. Un paramètre qui dit ce qu'il porte, plutôt qu'un
/// booléen : `render_document(…, PdfDocument::Invoice)` se relit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PdfDocument {
    /// La facture : titre « Facture », QR au TTC — inchangée (téléchargement,
    /// envoi et renvoi de facture).
    Invoice,
    /// Un rappel : titre « Rappel », bloc réglé / reste / frais, QR au **reste
    /// dû**. Les montants sont **déjà calculés** par [`reminder_amounts`] — la
    /// construction des entrées QR reste testable sans base.
    Reminder(ReminderAmounts),
}

/// Les montants d'un rappel, calculés **une fois** et partagés par le texte
/// (`{totalDue}`), le PDF et la QR (Story 25-4-b2, AC 7).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReminderAmounts {
    /// Total des règlements enregistrés.
    pub amount_settled: Decimal,
    /// Reste dû **arrondi au centime** (`MidpointAwayFromZero`), strictement
    /// positif — le montant de la QR.
    pub amount_due: Decimal,
    /// Frais cumulés : ceux des niveaux déjà émis (dédupliqués, hors niveau
    /// courant) plus ceux du niveau envoyé. ⛔ Jamais dans la QR : ils ne sont pas
    /// comptabilisés (#401), un virement qui les inclurait serait refusé.
    pub fees: Decimal,
}

/// Arrondit un reste dû brut au centime et refuse ce qui ne se réclame pas
/// (Story 25-4-b2, AC 7 et 9). Le refus porte sur la valeur **arrondie** : un
/// reste de 0.004 est refusé ici, au lieu d'atteindre une QR invalide et de
/// ressortir en `INVOICE_NOT_PDF_READY`.
pub fn reminder_amount_due(raw: Decimal) -> Result<Decimal, AppError> {
    // Story 25-4-c3-b (AC 1) : la définition unique du reste dû au centime.
    let due = kesh_db::repositories::invoice_settlements::amount_due_to_centime(raw);
    if due <= Decimal::ZERO {
        return Err(AppError::ReminderNothingDue);
    }
    Ok(due)
}

/// Calcule les montants d'un rappel de niveau `level_number` (Story 25-4-b2).
///
/// Le reste dû vient d'`invoice_settlements::amount_due` (forme scalaire : une
/// facture à la fois) — jamais réécrit. ⚠️ `amount_due` et `amount_settled` ne
/// prennent pas de `company_id` : l'appelant DOIT avoir chargé la facture par
/// `find_by_id_with_lines(pool, company.id, …)`, qui porte le scoping.
///
/// ⛔ Les trois lectures se font dans UNE transaction (revue de code 25-4-b2, P1) :
/// sous REPEATABLE READ, elles voient le même instantané. Lues séparément, un
/// règlement inséré entre deux d'entre elles rendait « déjà réglé » et « reste à
/// payer » incohérents entre eux, et la QR pouvait réclamer un reste d'avant le
/// paiement.
pub async fn reminder_amounts(
    pool: &sqlx::MySqlPool,
    company_id: i64,
    invoice_id: i64,
    level_number: i16,
    level_fee: Decimal,
) -> Result<ReminderAmounts, AppError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;
    let raw_due = invoice_settlements::amount_due(&mut *tx, invoice_id).await?;
    let amount_settled = invoice_settlements::amount_settled(&mut *tx, invoice_id).await?;
    let other_fees = invoice_reminders::sum_fees_deduped_excluding(
        &mut *tx,
        company_id,
        invoice_id,
        level_number,
    )
    .await?;
    // Lecture seule : rien à valider, la transaction ne sert qu'à l'instantané.
    tx.rollback()
        .await
        .map_err(|e| AppError::Internal(format!("rollback tx: {e}")))?;
    let amount_due = reminder_amount_due(raw_due)?;
    Ok(ReminderAmounts {
        amount_settled,
        amount_due,
        fees: other_fees + level_fee,
    })
}

/// Génère le PDF QR-facture d'une facture **validée** — la facture ou un
/// rappel (Story 25-4-b2) —, scopée à `company` (anti-IDOR).
///
/// Contrat d'autorisation : `company` DOIT provenir d'une source déjà
/// autorisée pour l'utilisateur courant (typiquement `get_company_for`) —
/// le service ne re-vérifie pas ce droit. Le scoping de la facture est
/// garanti par `find_by_id_with_lines(pool, company.id, …)`.
///
/// Séquence : chargement facture + lignes, validations (statut, nombre de
/// lignes, contact, compte bancaire primary), mapping `kesh-qrbill`,
/// génération. Le titre du rappel est résolu dans `locale` — celle du contact
/// pour un envoi —, pas dans celle de l'installation.
///
/// ⛔ **Story 25-6-b (#387) : la facture ne se rend plus directement.** Son
/// PDF passe par `issued_invoice_pdf::get_or_freeze`, qui le fige au premier
/// rendu et le relit ensuite ; appeler ici avec [`PdfDocument::Invoice`]
/// produirait un document qui n'est pas celui émis. L'ancienne fonction
/// `render` a été retirée pour cette raison. Les rappels, eux, restent
/// régénérés (#502).
pub async fn render_document(
    pool: &sqlx::MySqlPool,
    i18n: &kesh_i18n::I18nBundle,
    locale: Locale,
    company: &Company,
    invoice_id: i64,
    document: PdfDocument,
) -> Result<RenderedInvoicePdf, AppError> {
    // Chargement facture + lignes (scopé company).
    let (invoice, lines) = invoices::find_by_id_with_lines(pool, company.id, invoice_id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;

    if invoice.status != "validated" {
        return Err(AppError::InvoiceNotValidated);
    }

    if lines.len() > MAX_LINES_PER_PDF {
        return Err(AppError::InvoiceTooManyLinesForPdf(lines.len()));
    }

    // Contact (débiteur).
    let contact = contacts::find_by_id(pool, invoice.contact_id)
        .await?
        .ok_or_else(|| refus(CauseRefusPdf::ContactAbsent))?;

    // Primary bank account.
    let primary_bank = bank_accounts::find_primary(pool, company.id)
        .await?
        .ok_or_else(|| refus(CauseRefusPdf::CompteAbsent))?;

    // Construction des structures kesh-qrbill.
    // Pays ISO-3166-1 alpha-2 depuis companies.country / contacts.country
    // (ajoutés en v0.1 via migration 20260418000001, DEFAULT 'CH').
    let creditor_country = fetch_country(pool, "companies", company.id).await?;
    let debtor_country = fetch_country(pool, "contacts", contact.id).await?;

    // Story 31-5 (#504) — le préfixe de la communication de la QR, dans la
    // langue du document (`locale`, celle du reste du PDF). Clé sans variable :
    // aucune marque d'isolation Fluent n'entre dans la QR.
    let qr_message_prefix = i18n.format(&locale, "invoice-qr-message-prefix", None);
    let (qr_data, pdf_data) = build_qrbill_inputs(
        &invoice,
        &lines,
        &contact,
        company,
        &primary_bank,
        &creditor_country,
        &debtor_country,
        document,
        &qr_message_prefix,
    )?;
    let qr_i18n = build_i18n(i18n, locale);

    // Génération.
    let bytes = kesh_qrbill::generate_qr_bill_pdf(&qr_data, &pdf_data, &qr_i18n)
        .map_err(map_qrbill_error)?;

    let filename_base = sanitize_filename(invoice.invoice_number.as_deref().unwrap_or("facture"));

    Ok(RenderedInvoicePdf {
        bytes,
        filename_base,
        invoice_version: invoice.version,
    })
}

/// Convertit les entités DB en `QrBillData` + `InvoicePdfData`.
///
/// `qr_message_prefix` : le préfixe de la communication de la QR
/// (« `<préfixe> <numéro>` »), résolu par l'appelant dans la langue du document
/// (clé `invoice-qr-message-prefix`, Story 31-5, #504).
#[allow(clippy::too_many_arguments)]
fn build_qrbill_inputs(
    invoice: &Invoice,
    lines: &[InvoiceLine],
    contact: &Contact,
    company: &Company,
    primary_bank: &BankAccount,
    creditor_country: &str,
    debtor_country: &str,
    document: PdfDocument,
    qr_message_prefix: &str,
) -> Result<(QrBillData, InvoicePdfData), AppError> {
    // Adresse créancier — STRUCTURÉE type S (#213, conformité SIX 21.11.2025).
    // Story 35-2a (#964, D6) — « vide » se juge par la règle de la QR-facture
    // (`champ_vide_pour_qr`, celle de la saisie), que `trim` ne tient pas
    // (U+00AD, U+200B), et le refus nomme le premier champ vide.
    let ca = company.structured_address();
    adresse_complete("creditor", &ca.postal_code, &ca.city)?;
    let creditor = Address {
        address_type: AddressType::Structured,
        name: company.name.clone(),
        line1: ca.street.clone(),
        line2: ca.building.clone(),
        postal_code: ca.postal_code.clone(),
        town: ca.city.clone(),
        country: if ca.country.trim().is_empty() {
            creditor_country.to_string()
        } else {
            ca.country.clone()
        },
    };
    pays_iso(Partie::Entreprise, &creditor.country)?;

    // Adresse débiteur — STRUCTURÉE type S (#213). Requise et complète (NPA + localité).
    let Some(a) = contact.structured_address() else {
        return Err(refus(CauseRefusPdf::AdresseClientRequise));
    };
    adresse_complete("debtor", &a.postal_code, &a.city)?;
    let debtor = Address {
        address_type: AddressType::Structured,
        name: contact.name.clone(),
        line1: a.street.clone(),
        line2: a.building.clone(),
        postal_code: a.postal_code.clone(),
        town: a.city.clone(),
        country: if a.country.trim().is_empty() {
            debtor_country.to_string()
        } else {
            a.country.clone()
        },
    };
    pays_iso(Partie::Client, &debtor.country)?;

    // IBAN / QR-IBAN + référence.
    let (iban, reference) = match primary_bank.qr_iban.as_deref() {
        Some(qr) if !qr.trim().is_empty() => {
            // Story 35-2a (D5) — une QRR qui ne se construit pas est une panne
            // interne, que l'utilisateur ne corrige pas : le message le dit, le
            // détail va au journal.
            let qrr = build_qrr(company.id as u64, invoice.id as u64).map_err(|e| {
                tracing::warn!("build_qrr failed: {e}");
                refus(CauseRefusPdf::PanneInterne)
            })?;
            (normalize_iban(qr), Reference::Qrr(qrr))
        }
        _ => {
            // Story 34-11a1 (#972, AC-3) — un principal sans IBAN est exclu par
            // le schéma (`chk_bank_accounts_card_shape` : une carte n'est jamais
            // principale ; `chk_bank_accounts_bank_has_iban`). Si le schéma
            // venait à le permettre, erreur interne, jamais de panique.
            let iban = primary_bank.iban.as_deref().ok_or_else(|| {
                AppError::Internal(format!(
                    "compte bancaire principal {} sans IBAN : QR-facture impossible",
                    primary_bank.id
                ))
            })?;
            (normalize_iban(iban), Reference::None)
        }
    };

    // #246 (Story 21-2a) : le montant réclamé par le QR et affiché sous
    // « Total TTC » est le TTC canonique (helper kesh-core, même arithmétique
    // que le débit créance) — `total_amount` est le HT comptable et ne doit
    // JAMAIS être présenté comme montant dû.
    //
    // Story 25-4-c4-a (#494) : arrondi à 5 centimes figé à la validation compris.
    let total_ttc = kesh_core::accounting::vat::invoice_total_ttc_rounded(
        lines.iter().map(|l| (l.line_total, l.vat_rate)),
        invoice.rounding_amount,
    );

    // Story 25-4-b2 (#416, AC 7) : la QR d'un rappel porte le RESTE DÛ — la même
    // valeur, déjà arrondie, que le texte et le bloc imprimé. ⛔ La référence et
    // le message ci-dessous ne dépendent PAS du document : c'est ce que lit le
    // rapprochement. Le message dépend en revanche de la LANGUE (Story 31-5,
    // #504) : `qr_message_prefix` est résolu dans la langue du document, si bien
    // qu'une facture et son rappel rendus dans la même langue portent le même
    // message ; le rapprochement lit la contenance du numéro, non le préfixe.
    // Story 32-1a (#716, B1) — « rendus dans la même langue » n'est pas toujours
    // le cas : une facture dont le PDF a été FIGÉ (avant la 0.15.0, ou dans une
    // langue que le contact a quittée depuis) garde le préfixe de ce PDF, tandis
    // que son rappel, produit à chaque envoi, porte celui de la langue courante
    // du client. Les deux QR diffèrent alors par le préfixe ; le rapprochement
    // n'en est pas touché, puisqu'il lit la contenance du numéro.
    let (qr_amount, reminder) = match document {
        PdfDocument::Invoice => (total_ttc, None),
        PdfDocument::Reminder(a) => {
            // Défense : `reminder_amounts` ne produit jamais un reste ≤ 0.
            let due = reminder_amount_due(a.amount_due)?;
            (
                due,
                Some(ReminderPdf {
                    amount_settled: a.amount_settled,
                    amount_due: due,
                    fees: a.fees,
                }),
            )
        }
    };

    let qr_data = QrBillData {
        creditor_iban: iban,
        creditor: creditor.clone(),
        ultimate_debtor: Some(debtor.clone()),
        amount: Some(qr_amount),
        currency: Currency::Chf,
        reference,
        unstructured_message: invoice.invoice_number.as_ref().map(|n| {
            let msg = format!("{qr_message_prefix} {n}");
            // SIX 2.2: unstructured_message max 140 chars (USTRD_MAX).
            msg.chars().take(140).collect::<String>()
        }),
        billing_information: None,
    };

    let invoice_lines_pdf: Vec<InvoiceLinePdf> = lines
        .iter()
        .map(|l| InvoiceLinePdf {
            description: l.description.clone(),
            quantity: l.quantity,
            unit_price: l.unit_price,
            vat_rate: l.vat_rate,
            line_total: l.line_total,
        })
        .collect();

    // #151 : sous-total HT + ventilation TVA par taux (arrondi par ligne DC7).
    let subtotal_ht: Decimal = lines.iter().map(|l| l.line_total).sum();
    let vat_lines = vat_lines_pdf(lines.iter().map(|l| (l.line_total, l.vat_rate)));

    let pdf_data = InvoicePdfData {
        invoice_number: invoice
            .invoice_number
            .clone()
            .unwrap_or_else(|| format!("#{}", invoice.id)),
        invoice_date: invoice.date,
        due_date: invoice.due_date,
        payment_terms: invoice.payment_terms.clone(),
        creditor_name: company.name.clone(),
        creditor_address_lines: split_lines(&company.address),
        creditor_ide: company.ide_number.clone(),
        // Story 16-3a (#151) — coordonnées de contact, ici pour la facture.
        // ⚠️ `InvoicePdfData` est construit à DEUX endroits (facture et
        // avoir) : un site oublié rendrait un document sans coordonnées
        // alors que l'autre en porte, sans qu'aucun test de l'autre ne le voie.
        creditor_phone: company.phone.clone(),
        creditor_email: company.email.clone(),
        creditor_website: company.website.clone(),
        debtor_name: contact.name.clone(),
        debtor_address_lines: split_lines(contact.address.as_deref().unwrap_or_default()),
        // Story 16-3b (#151) — résolu depuis le CONTACT destinataire (D5), au
        // même titre que le nom et l'adresse : pas de copie dénormalisée sur
        // `invoices`, un changement doit se refléter sur les PDF régénérés.
        //
        // ⚠️ C'EST LE SITE QUE LA MUTATION D'AC7 DOIT TUER. `draw_invoice_section`
        // est partagée par la facture et l'avoir : un test posé dans `pdf.rs` ne
        // peut STRUCTURELLEMENT pas discriminer les deux et resterait vert si
        // l'un des deux sites oubliait de renseigner le champ.
        debtor_client_number: contact.client_number.clone(),
        lines: invoice_lines_pdf,
        subtotal_ht,
        vat_lines,
        // Story 25-4-c4-b : l'arrondi figé de la facture — un rappel montre aussi le sien.
        rounding: invoice.rounding_amount,
        total: total_ttc,
        currency: Currency::Chf,
        origin_reference: None,
        reminder,
    };

    Ok((qr_data, pdf_data))
}

/// Returns every non-empty line of a multi-line address (for display in the
/// invoice top section, derived `address` column).
pub(crate) fn split_lines(raw: &str) -> Vec<String> {
    raw.split('\n')
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}

/// Build a `QrBillI18n` by querying the shared Fluent bundle for every key.
pub(crate) fn build_i18n(bundle: &kesh_i18n::I18nBundle, locale: Locale) -> QrBillI18n {
    let mut entries: HashMap<&'static str, String> = HashMap::new();
    for key in kesh_qrbill::types::I18N_KEYS {
        let value = bundle.format(&locale, key, None);
        entries.insert(key, value);
    }
    QrBillI18n::new(entries)
}

/// Maps `QrBillError` to `AppError`. Business errors (invalid IBAN, field too
/// long, amount out of range) map to `InvoiceNotPdfReady` (400); PDF-rendering
/// errors map to `PdfGenerationFailed` (500, detail logged only).
///
/// Story 35-2a (#955, D5) — un refus de la QR-facture devient une **cause** de
/// l'ensemble clos [`CauseRefusPdf`], dont [`refus`] fait un message localisé qui
/// nomme le champ et le caractère ; le texte de `kesh-qrbill` (français en dur,
/// noms techniques) ne va plus qu'au journal.
pub(crate) fn map_qrbill_error(err: QrBillError) -> AppError {
    use CauseRefusPdf as C;
    let cause = match &err {
        QrBillError::InvalidIban(_) => C::IbanInvalide,
        QrBillError::InvalidQrIban(_) => C::QrIbanInvalide,
        // La QRR est calculée par Kesh, le payload SPC n'est lu qu'à l'import :
        // l'utilisateur ne corrige ni l'une ni l'autre.
        QrBillError::InvalidQrr(_) | QrBillError::InvalidPayload(_) => C::PanneInterne,
        QrBillError::FieldTooLong { field, max, got } => C::ChampTropLong {
            champ: field,
            max: *max,
            got: *got,
        },
        QrBillError::FieldEmpty(field) => C::ChampVide { champ: field },
        QrBillError::InvalidAmount(_) => C::MontantInvalide,
        QrBillError::InvalidCurrency(_) => C::DeviseInvalide,
        // Défensif : `build_qrbill_inputs` contrôle le pays de chaque partie avant
        // (D7). `validate_country` suffixe le code d'une glose technique, retirée.
        QrBillError::InvalidCountry(c) => C::PaysInvalide {
            code: c.split(" (").next().unwrap_or_default().to_string(),
        },
        QrBillError::InvalidCharset { field, codepoint } => C::CaractereRefuse {
            champ: field,
            codepoint: *codepoint,
        },
        QrBillError::PdfGeneration(msg) => return AppError::PdfGenerationFailed(msg.clone()),
        // #151 code-review : la garde géométrique (lignes + récap TVA débordant)
        // remonte un 400 actionnable, pas un 500 opaque.
        QrBillError::TooManyLines(n) => return AppError::InvoiceTooManyLinesForPdf(*n),
        // Story 16-3a (#151) — en-tête débordant sur le tableau. Actionnable par
        // l'utilisateur, donc 400 et non 500 — même raisonnement que TooManyLines.
        //
        // ⚠️ Variante DÉDIÉE et non `Validation` : l'écran de facture résout le
        // message par une liste blanche de codes, où `VALIDATION_ERROR` ne figure
        // pas — le message y serait remplacé par un générique. (Passe 3 de revue.)
        //
        // ⚠️ L'ordonnée atteinte est la SEULE donnée de diagnostic du refus, et
        // `AppError::InvoicePdfHeaderOverflow` ne porte aucune charge — à la
        // différence du jumeau `TooManyLines(n)`, qui conserve son compte. Sans
        // cette trace, un exploitant ne sait ni de combien le document dépassait,
        // ni si la cause est l'émetteur ou le destinataire, alors que le message
        // rendu énumère les deux hypothèses. On journalise donc `y` ici plutôt
        // que de la faire porter à l'`AppError` : la charge n'a aucun usage côté
        // client, le message affiché étant traduit et volontairement non
        // technique. *(Passe 6 de revue.)*
        QrBillError::HeaderOverflow(y) => {
            tracing::warn!(
                reached_y_mm = y,
                "en-tête de document débordant sur le tableau des lignes — PDF refusé en 400"
            );
            return AppError::InvoicePdfHeaderOverflow;
        }
    };
    tracing::warn!(detail = %err, "QR-facture refusée — PDF refusé en 400");
    refus(cause)
}

/// La partie d'une adresse de la QR-facture (D11 : « entreprise », le terme du
/// bloc `invoice-pdf-error-*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Partie {
    /// Le créancier : l'entreprise qui facture.
    Entreprise,
    /// Le débiteur : le client.
    Client,
}

/// Story 35-2a (#955, #964, D5) — la cause d'un refus `INVOICE_NOT_PDF_READY`,
/// sur un **ensemble clos** : les refus de `kesh-qrbill` et ceux que Kesh pose
/// avant lui. Un champ se nomme par son nom technique `kesh-qrbill`
/// (`debtor.town`) ; [`refus_pdf`] en fait des mots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CauseRefusPdf {
    /// Un caractère hors du jeu de la directive SIX (§ 4.1.1).
    CaractereRefuse { champ: &'static str, codepoint: u32 },
    /// Un champ obligatoire vide (par la règle `champ_vide_pour_qr`).
    ChampVide { champ: &'static str },
    /// Un champ plus long que la QR-facture ne l'admet.
    ChampTropLong {
        champ: &'static str,
        max: usize,
        got: usize,
    },
    /// L'IBAN du compte principal.
    IbanInvalide,
    /// Le QR-IBAN du compte principal.
    QrIbanInvalide,
    /// QRR, payload : une panne interne, que l'utilisateur ne corrige pas.
    PanneInterne,
    /// Le montant, hors des bornes de la QR-facture.
    MontantInvalide,
    /// La devise.
    DeviseInvalide,
    /// Un code pays refusé par `kesh-qrbill`, sans partie (bras défensif).
    PaysInvalide { code: String },
    /// Le code pays d'une partie, hors de la liste ISO (D7), tel que stocké.
    PaysDeLaPartie { partie: Partie, code: String },
    /// Le contact de la facture introuvable.
    ContactAbsent,
    /// Le contact de l'avoir introuvable (D8).
    ContactAbsentAvoir,
    /// Un client sans aucune adresse.
    AdresseClientRequise,
    /// Aucun compte bancaire principal.
    CompteAbsent,
}

/// Ce que [`refus_pdf`] rend : la clé du catalogue, ses arguments (déjà formatés
/// en chaînes : Fluent formaterait un nombre selon la locale) et le repli
/// fr-CH, égal au catalogue (garde G9 de `textes_coherents.rs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RefusPdf {
    pub cle: &'static str,
    pub args: Vec<(&'static str, String)>,
    pub repli: String,
}

/// Story 35-2a (D3) — la fonction **pure** du refus : la clé, les arguments et le
/// repli d'une cause. Elle ne résout rien (aucun état global) ; c'est le seul
/// endroit où le caractère est échappé (D4 : `escape_debug`, puis `U+XXXX`).
///
/// Une clé par champ et par raison, sans sélecteur Fluent : G9 compare un
/// littéral par clé. Un nom de champ inconnu (`address.*`,
/// `billing_information`, un nom futur) prend le repli de sa raison.
pub(crate) fn refus_pdf(cause: &CauseRefusPdf) -> RefusPdf {
    use CauseRefusPdf as C;
    let sans_argument = |cle: &'static str, repli: &str| RefusPdf {
        cle,
        args: Vec::new(),
        repli: repli.to_string(),
    };
    match cause {
        C::CaractereRefuse { champ, codepoint } => {
            let car = char::from_u32(*codepoint)
                .unwrap_or(char::REPLACEMENT_CHARACTER)
                .escape_debug()
                .to_string();
            let point = format!("U+{codepoint:04X}");
            let (cle, repli) = match *champ {
                "creditor.name" => (
                    "invoice-pdf-error-charset-company-name",
                    format!(
                        "Le nom de l'entreprise contient un caractère refusé par la QR-facture : « {car} » ({point}). Corrigez-le dans Paramètres → Organisation."
                    ),
                ),
                "creditor.line1" => (
                    "invoice-pdf-error-charset-company-street",
                    format!(
                        "La rue de l'entreprise contient un caractère refusé par la QR-facture : « {car} » ({point}). Corrigez-la dans Paramètres → Organisation."
                    ),
                ),
                "creditor.line2" => (
                    "invoice-pdf-error-charset-company-building",
                    format!(
                        "Le numéro de bâtiment de l'entreprise contient un caractère refusé par la QR-facture : « {car} » ({point}). Corrigez-le dans Paramètres → Organisation."
                    ),
                ),
                "creditor.postal_code" => (
                    "invoice-pdf-error-charset-company-postal-code",
                    format!(
                        "Le NPA de l'entreprise contient un caractère refusé par la QR-facture : « {car} » ({point}). Corrigez-le dans Paramètres → Organisation."
                    ),
                ),
                "creditor.town" => (
                    "invoice-pdf-error-charset-company-city",
                    format!(
                        "La localité de l'entreprise contient un caractère refusé par la QR-facture : « {car} » ({point}). Corrigez-la dans Paramètres → Organisation."
                    ),
                ),
                "debtor.name" => (
                    "invoice-pdf-error-charset-client-name",
                    format!(
                        "Le nom du client contient un caractère refusé par la QR-facture : « {car} » ({point}). Corrigez la fiche du client dans le carnet d'adresses."
                    ),
                ),
                "debtor.line1" => (
                    "invoice-pdf-error-charset-client-street",
                    format!(
                        "La rue du client contient un caractère refusé par la QR-facture : « {car} » ({point}). Corrigez la fiche du client dans le carnet d'adresses."
                    ),
                ),
                "debtor.line2" => (
                    "invoice-pdf-error-charset-client-building",
                    format!(
                        "Le numéro de bâtiment du client contient un caractère refusé par la QR-facture : « {car} » ({point}). Corrigez la fiche du client dans le carnet d'adresses."
                    ),
                ),
                "debtor.postal_code" => (
                    "invoice-pdf-error-charset-client-postal-code",
                    format!(
                        "Le NPA du client contient un caractère refusé par la QR-facture : « {car} » ({point}). Corrigez la fiche du client dans le carnet d'adresses."
                    ),
                ),
                "debtor.town" => (
                    "invoice-pdf-error-charset-client-city",
                    format!(
                        "La localité du client contient un caractère refusé par la QR-facture : « {car} » ({point}). Corrigez la fiche du client dans le carnet d'adresses."
                    ),
                ),
                // D9 — la communication porte le numéro de facture.
                "unstructured_message" => (
                    "invoice-pdf-error-charset-invoice-number",
                    format!(
                        "Le numéro de facture contient un caractère refusé par la QR-facture : « {car} » ({point}). Une facture validée ne se corrige pas : vérifiez le format de numérotation dans Administration → Facturation."
                    ),
                ),
                _ => (
                    "invoice-pdf-error-charset-other",
                    format!(
                        "Un champ de la QR-facture contient un caractère refusé : « {car} » ({point}). Vérifiez les fiches du client et de l'entreprise."
                    ),
                ),
            };
            RefusPdf {
                cle,
                args: vec![("car", car), ("point", point)],
                repli,
            }
        }
        C::ChampVide { champ } => match *champ {
            "creditor.name" => sans_argument(
                "invoice-pdf-error-empty-company-name",
                "Le nom de l'entreprise est vide. Complétez-le dans Paramètres → Organisation.",
            ),
            "creditor.postal_code" => sans_argument(
                "invoice-pdf-error-empty-company-postal-code",
                "Adresse de l'entreprise incomplète : le NPA est vide. Complétez-la dans Paramètres → Organisation (NPA et localité sont exigés par la QR-facture).",
            ),
            "creditor.town" => sans_argument(
                "invoice-pdf-error-empty-company-city",
                "Adresse de l'entreprise incomplète : la localité est vide. Complétez-la dans Paramètres → Organisation (NPA et localité sont exigés par la QR-facture).",
            ),
            "debtor.name" => sans_argument(
                "invoice-pdf-error-empty-client-name",
                "Le nom du client est vide. Complétez la fiche du client dans le carnet d'adresses.",
            ),
            "debtor.postal_code" => sans_argument(
                "invoice-pdf-error-empty-client-postal-code",
                "Adresse du client incomplète : le NPA est vide. Complétez la fiche du client dans le carnet d'adresses (NPA et localité sont exigés par la QR-facture).",
            ),
            "debtor.town" => sans_argument(
                "invoice-pdf-error-empty-client-city",
                "Adresse du client incomplète : la localité est vide. Complétez la fiche du client dans le carnet d'adresses (NPA et localité sont exigés par la QR-facture).",
            ),
            _ => sans_argument(
                "invoice-pdf-error-empty-other",
                "Un champ obligatoire de la QR-facture est vide. Vérifiez les fiches du client et de l'entreprise.",
            ),
        },
        C::ChampTropLong { champ, max, got } => {
            let (max, got) = (max.to_string(), got.to_string());
            let (cle, repli) = match *champ {
                "creditor.name" => (
                    "invoice-pdf-error-too-long-company-name",
                    format!(
                        "Le nom de l'entreprise dépasse {max} caractères ({got}). Raccourcissez-le dans Paramètres → Organisation."
                    ),
                ),
                "creditor.line1" => (
                    "invoice-pdf-error-too-long-company-street",
                    format!(
                        "La rue de l'entreprise dépasse {max} caractères ({got}). Raccourcissez-la dans Paramètres → Organisation."
                    ),
                ),
                "creditor.line2" => (
                    "invoice-pdf-error-too-long-company-building",
                    format!(
                        "Le numéro de bâtiment de l'entreprise dépasse {max} caractères ({got}). Raccourcissez-le dans Paramètres → Organisation."
                    ),
                ),
                "creditor.postal_code" => (
                    "invoice-pdf-error-too-long-company-postal-code",
                    format!(
                        "Le NPA de l'entreprise dépasse {max} caractères ({got}). Raccourcissez-le dans Paramètres → Organisation."
                    ),
                ),
                "creditor.town" => (
                    "invoice-pdf-error-too-long-company-city",
                    format!(
                        "La localité de l'entreprise dépasse {max} caractères ({got}). Raccourcissez-la dans Paramètres → Organisation."
                    ),
                ),
                "debtor.name" => (
                    "invoice-pdf-error-too-long-client-name",
                    format!(
                        "Le nom du client dépasse {max} caractères ({got}). Raccourcissez-le dans la fiche du client, au carnet d'adresses."
                    ),
                ),
                "debtor.line1" => (
                    "invoice-pdf-error-too-long-client-street",
                    format!(
                        "La rue du client dépasse {max} caractères ({got}). Raccourcissez-la dans la fiche du client, au carnet d'adresses."
                    ),
                ),
                "debtor.line2" => (
                    "invoice-pdf-error-too-long-client-building",
                    format!(
                        "Le numéro de bâtiment du client dépasse {max} caractères ({got}). Raccourcissez-le dans la fiche du client, au carnet d'adresses."
                    ),
                ),
                "debtor.postal_code" => (
                    "invoice-pdf-error-too-long-client-postal-code",
                    format!(
                        "Le NPA du client dépasse {max} caractères ({got}). Raccourcissez-le dans la fiche du client, au carnet d'adresses."
                    ),
                ),
                "debtor.town" => (
                    "invoice-pdf-error-too-long-client-city",
                    format!(
                        "La localité du client dépasse {max} caractères ({got}). Raccourcissez-la dans la fiche du client, au carnet d'adresses."
                    ),
                ),
                _ => (
                    "invoice-pdf-error-too-long-other",
                    format!(
                        "Un champ de la QR-facture dépasse {max} caractères ({got}). Vérifiez les fiches du client et de l'entreprise."
                    ),
                ),
            };
            RefusPdf {
                cle,
                args: vec![("max", max), ("got", got)],
                repli,
            }
        }
        C::IbanInvalide => sans_argument(
            "invoice-pdf-error-iban-invalid",
            "L'IBAN du compte bancaire principal n'est pas valable pour la QR-facture. Corrigez-le dans Administration → Comptes bancaires.",
        ),
        C::QrIbanInvalide => sans_argument(
            "invoice-pdf-error-qr-iban-invalid",
            "Le QR-IBAN du compte bancaire principal n'est pas valable pour la QR-facture. Corrigez-le dans Administration → Comptes bancaires.",
        ),
        C::PanneInterne => sans_argument(
            "invoice-pdf-error-internal",
            "La QR-facture n'a pas pu être construite (erreur interne, non corrigeable depuis l'écran). Contactez l'administrateur.",
        ),
        C::MontantInvalide => sans_argument(
            "invoice-pdf-error-amount-invalid",
            "Le montant à payer est hors des bornes admises par la QR-facture (de 0.01 à 999 999 999.99).",
        ),
        C::DeviseInvalide => sans_argument(
            "invoice-pdf-error-currency-invalid",
            "La devise n'est pas admise par la QR-facture (CHF ou EUR).",
        ),
        C::PaysInvalide { code } => RefusPdf {
            cle: "invoice-pdf-error-country-invalid",
            args: vec![("code", code.clone())],
            repli: format!("Le code pays « {code} » n'est pas un code ISO valide."),
        },
        C::PaysDeLaPartie {
            partie: Partie::Entreprise,
            code,
        } => RefusPdf {
            cle: "invoice-pdf-error-country-invalid-company",
            args: vec![("code", code.clone())],
            repli: format!(
                "Le pays de l'entreprise (« {code} ») n'est pas un code ISO valide. Corrigez-le dans Paramètres → Organisation."
            ),
        },
        C::PaysDeLaPartie {
            partie: Partie::Client,
            code,
        } => RefusPdf {
            cle: "invoice-pdf-error-country-invalid-client",
            args: vec![("code", code.clone())],
            repli: format!(
                "Le pays du client (« {code} ») n'est pas un code ISO valide. Corrigez la fiche du client dans le carnet d'adresses."
            ),
        },
        C::ContactAbsent => sans_argument(
            "invoice-pdf-error-contact-missing",
            "Le contact lié à la facture est introuvable.",
        ),
        C::ContactAbsentAvoir => sans_argument(
            "credit-note-pdf-error-contact-missing",
            "Le contact lié à l'avoir est introuvable.",
        ),
        C::AdresseClientRequise => sans_argument(
            "invoice-pdf-error-client-address-required",
            "L'adresse du client est obligatoire pour la génération du PDF.",
        ),
        C::CompteAbsent => sans_argument(
            "invoice-pdf-error-no-primary-bank",
            "Aucun compte bancaire principal n'est configuré pour cette entreprise.",
        ),
    }
}

/// Story 35-2a (D3) — **la** construction du refus `INVOICE_NOT_PDF_READY` : la
/// cause passe par [`refus_pdf`], puis est résolue dans la langue de
/// l'installation (`t_args`, qui retire les marques d'isolation ; `t` pour une clé
/// sans argument). Toute construction de production passe par ici (garde AC14 de
/// `textes_coherents.rs`).
pub(crate) fn refus(cause: CauseRefusPdf) -> AppError {
    let RefusPdf { cle, args, repli } = refus_pdf(&cause);
    let message = if args.is_empty() {
        crate::errors::t(cle, &repli)
    } else {
        let mut fluent = kesh_i18n::FluentArgs::new();
        for (nom, valeur) in args {
            fluent.set(nom, valeur);
        }
        crate::errors::t_args(cle, &repli, &fluent)
    };
    AppError::InvoiceNotPdfReady(message)
}

/// D6 (#964) — NPA et localité d'une partie, jugés par la règle de la
/// QR-facture ; le premier vide est nommé, par la même phrase que si
/// `kesh-qrbill` l'avait refusé.
fn adresse_complete(partie: &'static str, npa: &str, localite: &str) -> Result<(), AppError> {
    let (champ_npa, champ_localite) = match partie {
        "creditor" => ("creditor.postal_code", "creditor.town"),
        _ => ("debtor.postal_code", "debtor.town"),
    };
    if kesh_qrbill::champ_vide_pour_qr(npa) {
        return Err(refus(CauseRefusPdf::ChampVide { champ: champ_npa }));
    }
    if kesh_qrbill::champ_vide_pour_qr(localite) {
        return Err(refus(CauseRefusPdf::ChampVide {
            champ: champ_localite,
        }));
    }
    Ok(())
}

/// D7 — le pays d'une partie, sur la liste ISO que la QR-facture exige ; le code
/// est montré tel que stocké (une valeur en minuscules posée en SQL reste
/// refusée, comme par `kesh-qrbill`).
fn pays_iso(partie: Partie, code: &str) -> Result<(), AppError> {
    if kesh_qrbill::is_iso_3166_alpha2(code) {
        return Ok(());
    }
    Err(refus(CauseRefusPdf::PaysDeLaPartie {
        partie,
        code: code.to_string(),
    }))
}

pub(crate) fn sanitize_filename(raw: &str) -> String {
    // B20 (review pass 2 G2 B) : cap à 64 caractères pour borner la taille
    // du header `Content-Disposition` (un `invoice_number` arbitrairement
    // long polluerait la réponse HTTP).
    raw.chars()
        .take(64)
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Lit la colonne `country` (CHAR(2)) de `companies` ou `contacts`.
/// `table` doit être un littéral validé en call-site pour éviter toute
/// injection SQL — seuls "companies" et "contacts" sont acceptés.
pub(crate) async fn fetch_country(
    pool: &sqlx::MySqlPool,
    table: &'static str,
    id: i64,
) -> Result<String, AppError> {
    let sql = match table {
        "companies" => "SELECT country FROM companies WHERE id = ?",
        "contacts" => "SELECT country FROM contacts WHERE id = ?",
        _ => {
            return Err(AppError::Internal(format!(
                "fetch_country: table `{table}` non autorisée"
            )));
        }
    };
    let row: Option<(String,)> = sqlx::query_as(sql)
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(format!("fetch_country({table}): {e}")))?;
    row.map(|(c,)| c)
        .ok_or_else(|| AppError::Database(DbError::NotFound))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn sanitize_filename_replaces_non_alphanumeric() {
        assert_eq!(sanitize_filename("F-2026-0042"), "F-2026-0042");
        assert_eq!(sanitize_filename("../../etc/passwd"), ".._.._etc_passwd");
        assert_eq!(sanitize_filename("F 2026 #42"), "F_2026__42");
    }

    fn company_with_contact_details() -> kesh_db::entities::Company {
        kesh_db::entities::Company {
            id: 1,
            name: "Démo SA".into(),
            first_name: None,
            last_name: None,
            address: "Rue du Lac 1\n1000 Lausanne".into(),
            address_street: "Rue du Lac".into(),
            address_building: "1".into(),
            address_postal_code: "1000".into(),
            address_city: "Lausanne".into(),
            address_country: "CH".into(),
            ide_number: Some("CHE-123.456.789".into()),
            org_type: kesh_db::entities::OrgType::Pme,
            accounting_language: kesh_db::entities::Language::Fr,
            instance_language: kesh_db::entities::Language::Fr,
            email: Some("contact@demo.ch".into()),
            phone: Some("+41 21 123 45 67".into()),
            website: Some("https://demo.ch".into()),
            is_stub: false,
            books_locked_through: None,
            version: 1,
            created_at: chrono::NaiveDateTime::default(),
            updated_at: chrono::NaiveDateTime::default(),
        }
    }

    /// Contact avec adresse **structurée** — `build_qrbill_inputs` refuse le
    /// débiteur dont le NPA ou la localité manquent (conformité SIX type S).
    fn contact_with_structured_address() -> kesh_db::entities::Contact {
        kesh_db::entities::Contact {
            id: 1,
            company_id: 1,
            contact_type: kesh_db::entities::ContactType::Entreprise,
            name: "Client SA".into(),
            first_name: None,
            last_name: None,
            is_client: true,
            is_supplier: false,
            address: Some("Marktgasse 28\n9400 Rorschach".into()),
            address_street: Some("Marktgasse".into()),
            address_building: Some("28".into()),
            address_postal_code: Some("9400".into()),
            address_city: Some("Rorschach".into()),
            address_country: Some("CH".into()),
            email: None,
            phone: None,
            ide_number: None,
            client_number: None,
            default_payment_terms: None,
            default_payment_terms_days: None,
            language: None,
            salutation: Default::default(),
            active: true,
            version: 1,
            created_at: chrono::NaiveDateTime::default(),
            updated_at: chrono::NaiveDateTime::default(),
        }
    }

    fn invoice() -> kesh_db::entities::Invoice {
        kesh_db::entities::Invoice {
            id: 1,
            company_id: 1,
            contact_id: 1,
            invoice_number: Some("F-2026-0001".into()),
            status: "validated".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 8, 8).unwrap(),
            due_date: None,
            payment_terms: None,
            total_amount: dec!(100.00),
            rounding_amount: dec!(0),
            journal_entry_id: None,
            paid_at: None,
            emailed_at: None,
            emailed_to: None,
            project_id: None,
            dunning_paused_at: None,
            dunning_paused_note: None,
            pdf_storage_path: None,
            pdf_sha256: None,
            pdf_frozen_at: None,
            pdf_language: None,
            version: 1,
            created_at: chrono::NaiveDateTime::default(),
            updated_at: chrono::NaiveDateTime::default(),
        }
    }

    /// `qr_iban: None` → branche IBAN simple, sans génération de QRR.
    fn primary_bank() -> kesh_db::entities::BankAccount {
        kesh_db::entities::BankAccount {
            id: 1,
            company_id: 1,
            bank_name: "Banque Démo".into(),
            description: None,
            iban: Some("CH9300762011623852957".into()),
            qr_iban: None,
            account_kind: kesh_db::entities::BankAccountKind::Bank,
            card_last4: None,
            is_primary: true,
            journal_account_id: None,
            version: 1,
            archived: false,
            closed_on: None,
            created_at: chrono::NaiveDateTime::default(),
            updated_at: chrono::NaiveDateTime::default(),
        }
    }

    /// Story 16-3a (#151) — **le pendant FACTURE** de
    /// `credit_note_pdf_carries_the_issuer_contact_details`.
    ///
    /// ⚠️ Ce test comble le trou relevé en passe 6 de `bmad-code-review` :
    /// `InvoicePdfData` est construit à **deux** endroits, et seul celui de
    /// l'avoir était vérifié. Écrire `creditor_phone: None` au site facture ne
    /// faisait **rougir aucun test** — le struct literal restant complet, le
    /// compilateur se tait ; les tests de `kesh-qrbill` posent leurs propres
    /// valeurs sans jamais voir `Company` ; et `invoice_pdf_e2e.rs` ne contrôle
    /// que le statut, le type MIME, l'en-tête `%PDF-1.` et la taille. Toutes les
    /// factures seraient sorties sans coordonnées pendant que les avoirs en
    /// portaient — la dissymétrie exacte que la story nomme « le piège qui
    /// coûterait le plus cher », et le document principal était le côté nu.
    /// Story 25-4-c4-b — la facture ET le rappel portent l'arrondi figé de la
    /// facture (revue de code P1, lentille C : l'AC 6 le promettait sans test).
    #[test]
    fn invoice_and_reminder_pdf_carry_the_frozen_rounding() {
        let inv = kesh_db::entities::Invoice {
            rounding_amount: dec!(0.01),
            ..invoice()
        };
        for doc in [
            PdfDocument::Invoice,
            PdfDocument::Reminder(ReminderAmounts {
                amount_settled: dec!(0),
                amount_due: dec!(100.00),
                fees: dec!(0),
            }),
        ] {
            let (_qr, data) = build_qrbill_inputs(
                &inv,
                &[],
                &contact_with_structured_address(),
                &company_with_contact_details(),
                &primary_bank(),
                "CH",
                "CH",
                doc,
                "Facture",
            )
            .expect("montage exploitable");
            assert_eq!(data.rounding, dec!(0.01));
        }
    }

    #[test]
    fn invoice_pdf_carries_the_issuer_contact_details() {
        let company = company_with_contact_details();
        let (_qr, data) = build_qrbill_inputs(
            &invoice(),
            &[],
            &contact_with_structured_address(),
            &company,
            &primary_bank(),
            "CH",
            "CH",
            PdfDocument::Invoice,
            "Facture",
        )
        .expect("le montage doit produire un PDF exploitable");

        assert_eq!(
            data.creditor_phone.as_deref(),
            Some("+41 21 123 45 67"),
            "la facture doit porter le téléphone de l'émetteur"
        );
        assert_eq!(
            data.creditor_email.as_deref(),
            Some("contact@demo.ch"),
            "la facture doit porter l'e-mail de l'émetteur"
        );
        assert_eq!(
            data.creditor_website.as_deref(),
            Some("https://demo.ch"),
            "la facture doit porter le site web de l'émetteur"
        );
    }

    /// Le pendant négatif : une société sans coordonnées n'en invente aucune.
    /// Sans lui, remplacer les trois champs par une constante non vide
    /// passerait le test précédent.
    #[test]
    fn invoice_pdf_omits_contact_details_the_company_does_not_have() {
        let company = kesh_db::entities::Company {
            email: None,
            phone: None,
            website: None,
            ..company_with_contact_details()
        };
        let (_qr, data) = build_qrbill_inputs(
            &invoice(),
            &[],
            &contact_with_structured_address(),
            &company,
            &primary_bank(),
            "CH",
            "CH",
            PdfDocument::Invoice,
            "Facture",
        )
        .expect("le montage doit produire un PDF exploitable");

        assert!(data.creditor_phone.is_none(), "aucun téléphone à inventer");
        assert!(data.creditor_email.is_none(), "aucun e-mail à inventer");
        assert!(data.creditor_website.is_none(), "aucun site web à inventer");
    }

    /// **AC6 / D5** (Story 16-3b, #151) — la FACTURE porte le numéro de client
    /// du destinataire, résolu depuis le contact.
    ///
    /// Pendant exact du test d'AC7 posé dans `credit_notes.rs` : les deux sites
    /// de construction d'`InvoicePdfData` doivent être couverts séparément, un
    /// test de rendu ne pouvant pas les discriminer.
    #[test]
    fn invoice_pdf_carries_the_debtor_client_number() {
        let mut contact = contact_with_structured_address();
        contact.client_number = Some("CLI-2026-00042".into());

        let (_qr, data) = build_qrbill_inputs(
            &invoice(),
            &[],
            &contact,
            &company_with_contact_details(),
            &primary_bank(),
            "CH",
            "CH",
            PdfDocument::Invoice,
            "Facture",
        )
        .expect("le montage doit produire un PDF exploitable");

        assert_eq!(
            data.debtor_client_number.as_deref(),
            Some("CLI-2026-00042"),
            "la facture doit porter le numéro de client du destinataire"
        );
    }

    /// Le pendant négatif : un contact sans numéro n'en fait pas apparaître un.
    #[test]
    fn invoice_pdf_omits_the_client_number_the_contact_does_not_have() {
        let (_qr, data) = build_qrbill_inputs(
            &invoice(),
            &[],
            &contact_with_structured_address(),
            &company_with_contact_details(),
            &primary_bank(),
            "CH",
            "CH",
            PdfDocument::Invoice,
            "Facture",
        )
        .expect("le montage doit produire un PDF exploitable");

        assert!(data.debtor_client_number.is_none());
    }

    // ─── Story 25-4-b2 (#416) — le PDF de rappel ──────────────────────────

    /// Une ligne à TVA non nulle : 1 000.— HT à 8,1 % = 1 081.— TTC.
    fn lines_1081() -> Vec<kesh_db::entities::InvoiceLine> {
        vec![kesh_db::entities::InvoiceLine {
            id: 1,
            invoice_id: 1,
            position: 1,
            description: "Prestation".into(),
            quantity: dec!(1),
            unit_price: dec!(1000.00),
            vat_rate: dec!(8.10),
            line_total: dec!(1000.00),
            revenue_account_id: None,
            created_at: chrono::NaiveDateTime::default(),
        }]
    }

    fn qr_bank() -> kesh_db::entities::BankAccount {
        kesh_db::entities::BankAccount {
            qr_iban: Some("CH4431999123000889012".into()),
            ..primary_bank()
        }
    }

    fn inputs(
        bank: &kesh_db::entities::BankAccount,
        document: PdfDocument,
    ) -> (QrBillData, InvoicePdfData) {
        build_qrbill_inputs(
            &invoice(),
            &lines_1081(),
            &contact_with_structured_address(),
            &company_with_contact_details(),
            bank,
            "CH",
            "CH",
            document,
            "Facture",
        )
        .expect("montage exploitable")
    }

    /// Story 34-11a1 (#972, AC-3) — un principal sans IBAN (inatteignable : le
    /// schéma l'exclut) rend une erreur interne, jamais une panique.
    #[test]
    fn principal_sans_iban_rend_une_erreur_interne() {
        let bank = kesh_db::entities::BankAccount {
            iban: None,
            ..primary_bank()
        };
        let res = build_qrbill_inputs(
            &invoice(),
            &lines_1081(),
            &contact_with_structured_address(),
            &company_with_contact_details(),
            &bank,
            "CH",
            "CH",
            PdfDocument::Invoice,
            "Facture",
        );
        let err = res.err();
        assert!(matches!(err, Some(AppError::Internal(_))), "{err:?}");
    }

    /// Story 31-5 (#504, AC9 a) — la communication de la QR porte le préfixe
    /// reçu, résolu par l'appelant dans la langue du document : `Rechnung` pour
    /// un document allemand, jamais `Facture` écrit en dur.
    #[test]
    fn qr_message_uses_the_document_language_prefix() {
        let (qr, _data) = build_qrbill_inputs(
            &invoice(),
            &lines_1081(),
            &contact_with_structured_address(),
            &company_with_contact_details(),
            &primary_bank(),
            "CH",
            "CH",
            PdfDocument::Invoice,
            "Rechnung",
        )
        .expect("montage exploitable");
        assert_eq!(
            qr.unstructured_message.as_deref(),
            Some("Rechnung F-2026-0001")
        );
    }

    /// AC 7 — ⛔ la QR du rappel d'une facture réglée en partie porte le RESTE
    /// DÛ ; la référence (QRR ou aucune) et le message sont ceux de la facture,
    /// à l'identique — c'est ce que lit le rapprochement. Les frais n'y sont pas.
    #[test]
    fn reminder_qr_carries_the_amount_due_and_keeps_the_reference() {
        let amounts = ReminderAmounts {
            amount_settled: dec!(900.00),
            amount_due: dec!(181.00),
            fees: dec!(20.00),
        };
        for bank in [primary_bank(), qr_bank()] {
            let (facture_qr, facture_pdf) = inputs(&bank, PdfDocument::Invoice);
            let (rappel_qr, rappel_pdf) = inputs(&bank, PdfDocument::Reminder(amounts));

            assert_eq!(
                facture_qr.amount,
                Some(dec!(1081.00)),
                "la facture reste au TTC"
            );
            assert_eq!(
                rappel_qr.amount,
                Some(dec!(181.00)),
                "le rappel au reste, sans frais"
            );
            assert_eq!(
                rappel_qr.reference, facture_qr.reference,
                "référence identique"
            );
            assert_eq!(
                rappel_qr.unstructured_message, facture_qr.unstructured_message,
                "message identique"
            );
            assert!(
                facture_pdf.reminder.is_none(),
                "la facture n'est pas un rappel"
            );
            let r = rappel_pdf.reminder.expect("le rappel porte son bloc");
            assert_eq!(r.amount_due, dec!(181.00), "même valeur que la QR");
            assert_eq!(r.amount_settled, dec!(900.00));
            assert_eq!(r.fees, dec!(20.00));
            assert_eq!(
                rappel_pdf.total,
                dec!(1081.00),
                "le total imprimé reste le TTC"
            );
        }
        // Anti-vacuité : la branche QR-IBAN porte bien une QRR.
        assert!(matches!(
            inputs(&qr_bank(), PdfDocument::Invoice).0.reference,
            Reference::Qrr(_)
        ));
    }

    /// AC 7 et 9 — un seul arrondi, au centime, loin de zéro ; le refus porte sur
    /// la valeur ARRONDIE.
    #[test]
    fn reminder_amount_due_rounds_once_and_refuses_nothing_due() {
        assert_eq!(reminder_amount_due(dec!(181.0000)).unwrap(), dec!(181.00));
        assert_eq!(reminder_amount_due(dec!(10.0050)).unwrap(), dec!(10.01));
        for rien in [dec!(0), dec!(0.0040), dec!(-40.00)] {
            assert!(
                matches!(reminder_amount_due(rien), Err(AppError::ReminderNothingDue)),
                "{rien} ne se réclame pas"
            );
        }
        // Défense : des montants de rappel à reste nul ne passent pas non plus
        // au montage des entrées QR.
        let nul = ReminderAmounts {
            amount_settled: dec!(1081.00),
            amount_due: dec!(0.00),
            fees: dec!(0),
        };
        assert!(matches!(
            build_qrbill_inputs(
                &invoice(),
                &lines_1081(),
                &contact_with_structured_address(),
                &company_with_contact_details(),
                &primary_bank(),
                "CH",
                "CH",
                PdfDocument::Reminder(nul),
                "Facture",
            ),
            Err(AppError::ReminderNothingDue)
        ));
    }

    /// AC 8 — ⚠️ la mention des frais court sur toute la largeur utile ; les
    /// gardes du PDF ne surveillent que l'ordonnée. Sa traduction, dans les 4
    /// locales réelles, tient dans `REMINDER_NOTE_MAX_CHARS` — l'allemand est le
    /// plus long.
    #[test]
    fn reminder_fees_note_fits_its_width_in_all_four_locales() {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../kesh-i18n/locales");
        let bundle = kesh_i18n::I18nBundle::load(&dir).unwrap();
        for locale in [Locale::FrCh, Locale::DeCh, Locale::ItCh, Locale::EnCh] {
            let note = build_i18n(&bundle, locale)
                .get("invoice-pdf-reminder-fees-note")
                .to_string();
            assert!(
                note.chars().count() <= kesh_qrbill::REMINDER_NOTE_MAX_CHARS,
                "{locale:?} : « {note} » dépasse {} caractères",
                kesh_qrbill::REMINDER_NOTE_MAX_CHARS
            );
            // Revue de code 25-4-b2, P1 — les libellés COURTS, dans leur colonne de
            // 50 mm : la troncature au dessin est une défense, pas une mise en page.
            for key in [
                "invoice-pdf-settled",
                "invoice-pdf-amount-due",
                "invoice-pdf-reminder-fees",
            ] {
                let label = build_i18n(&bundle, locale).get(key).to_string();
                assert!(
                    label.chars().count() <= kesh_qrbill::REMINDER_LABEL_MAX_CHARS,
                    "{locale:?} : « {label} » dépasse {} caractères",
                    kesh_qrbill::REMINDER_LABEL_MAX_CHARS
                );
            }
        }
    }

    // ─── Story 35-2a (#955, #964) — le refus du PDF nomme le champ ─────────

    /// Table écrite à part de la fonction pure (V5-2) : nom de champ `kesh-qrbill`
    /// → (suffixe de la clé, mot fr-CH de la partie, mot fr-CH du champ).
    const CHAMPS: [(&str, &str, &str, &str); 11] = [
        ("creditor.name", "company-name", "entreprise", "nom"),
        ("creditor.line1", "company-street", "entreprise", "rue"),
        ("creditor.line2", "company-building", "entreprise", "numéro"),
        (
            "creditor.postal_code",
            "company-postal-code",
            "entreprise",
            "NPA",
        ),
        ("creditor.town", "company-city", "entreprise", "localité"),
        ("debtor.name", "client-name", "client", "nom"),
        ("debtor.line1", "client-street", "client", "rue"),
        ("debtor.line2", "client-building", "client", "numéro"),
        ("debtor.postal_code", "client-postal-code", "client", "NPA"),
        ("debtor.town", "client-city", "client", "localité"),
        (
            "unstructured_message",
            "invoice-number",
            "facture",
            "numéro de facture",
        ),
    ];

    /// Le mot de la partie hors fr-CH (AC2 c).
    fn partie_hors_fr(locale: Locale, partie: &str) -> &'static str {
        match (locale, partie) {
            (Locale::DeCh, "entreprise") => "Unternehmen",
            (Locale::DeCh, "client") => "Kunde",
            (Locale::DeCh, _) => "Rechnung",
            (Locale::ItCh, "entreprise") => "azienda",
            (Locale::ItCh, "client") => "cliente",
            (Locale::ItCh, _) => "fattura",
            (Locale::EnCh, "entreprise") => "company",
            (Locale::EnCh, "client") => "client",
            (Locale::EnCh, _) => "invoice",
            (Locale::FrCh, _) => unreachable!("fr-CH se contrôle par la table"),
        }
    }

    /// Les cas de l'ensemble clos (D5) : la cause, la clé attendue, les noms des
    /// arguments attendus et, pour une clé par champ, (partie, mot) en fr-CH.
    #[allow(clippy::type_complexity)]
    fn cas_du_refus() -> Vec<(
        CauseRefusPdf,
        String,
        Vec<&'static str>,
        Option<(&'static str, &'static str)>,
    )> {
        use CauseRefusPdf as C;
        let mut cas = Vec::new();
        for (champ, suffixe, partie, mot) in CHAMPS {
            cas.push((
                C::CaractereRefuse {
                    champ,
                    codepoint: 0x2013,
                },
                format!("invoice-pdf-error-charset-{suffixe}"),
                vec!["car", "point"],
                Some((partie, mot)),
            ));
            if matches!(mot, "nom" | "NPA" | "localité") {
                cas.push((
                    C::ChampVide { champ },
                    format!("invoice-pdf-error-empty-{suffixe}"),
                    vec![],
                    Some((partie, mot)),
                ));
            }
            if champ != "unstructured_message" {
                cas.push((
                    C::ChampTropLong {
                        champ,
                        max: 70,
                        got: 71,
                    },
                    format!("invoice-pdf-error-too-long-{suffixe}"),
                    vec!["max", "got"],
                    Some((partie, mot)),
                ));
            }
        }
        // Les replis par raison : `billing_information` (toujours `None` dans
        // Kesh), un nom `address.*` des bras par défaut de `kesh-qrbill`.
        for champ in ["billing_information", "address.name"] {
            cas.push((
                C::CaractereRefuse {
                    champ,
                    codepoint: 0x2013,
                },
                "invoice-pdf-error-charset-other".into(),
                vec!["car", "point"],
                None,
            ));
            cas.push((
                C::ChampVide { champ },
                "invoice-pdf-error-empty-other".into(),
                vec![],
                None,
            ));
            cas.push((
                C::ChampTropLong {
                    champ,
                    max: 140,
                    got: 141,
                },
                "invoice-pdf-error-too-long-other".into(),
                vec!["max", "got"],
                None,
            ));
        }
        let pays = || "SU".to_string();
        for (cause, cle, args) in [
            (C::IbanInvalide, "invoice-pdf-error-iban-invalid", vec![]),
            (
                C::QrIbanInvalide,
                "invoice-pdf-error-qr-iban-invalid",
                vec![],
            ),
            (C::PanneInterne, "invoice-pdf-error-internal", vec![]),
            (
                C::MontantInvalide,
                "invoice-pdf-error-amount-invalid",
                vec![],
            ),
            (
                C::DeviseInvalide,
                "invoice-pdf-error-currency-invalid",
                vec![],
            ),
            (
                C::PaysInvalide { code: pays() },
                "invoice-pdf-error-country-invalid",
                vec!["code"],
            ),
            (
                C::PaysDeLaPartie {
                    partie: Partie::Entreprise,
                    code: pays(),
                },
                "invoice-pdf-error-country-invalid-company",
                vec!["code"],
            ),
            (
                C::PaysDeLaPartie {
                    partie: Partie::Client,
                    code: pays(),
                },
                "invoice-pdf-error-country-invalid-client",
                vec!["code"],
            ),
            (
                C::ContactAbsent,
                "invoice-pdf-error-contact-missing",
                vec![],
            ),
            (
                C::ContactAbsentAvoir,
                "credit-note-pdf-error-contact-missing",
                vec![],
            ),
            (
                C::AdresseClientRequise,
                "invoice-pdf-error-client-address-required",
                vec![],
            ),
            (C::CompteAbsent, "invoice-pdf-error-no-primary-bank", vec![]),
        ] {
            cas.push((cause, cle.to_string(), args, None));
        }
        cas
    }

    /// **AC2** (D3, D4, D5, AC8) — la table complète du refus, par la fonction pure
    /// et un catalogue chargé **localement** (jamais l'état global d'`errors`) :
    /// (a) la clé, les arguments et le repli ; (b) le rendu des quatre locales, qui
    /// n'est ni la clé, ni (hors fr-CH) la valeur fr-CH, et ne porte ni variable, ni
    /// nom technique ; (c) le rendu fr-CH de chaque clé par champ nomme la partie et
    /// le champ, et les autres langues au moins la partie. Le repli se compare au
    /// rendu fr-CH, jamais à un littéral (G9 compte les littéraux du fichier).
    #[test]
    fn le_refus_du_pdf_nomme_le_champ_dans_les_quatre_langues() {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../kesh-i18n/locales");
        let bundle = kesh_i18n::I18nBundle::load(&dir).unwrap();
        let sans_marques = |s: String| s.replace(['\u{2068}', '\u{2069}'], "");
        let cas = cas_du_refus();
        let mut cles = std::collections::BTreeSet::new();
        let mut par_champ = 0;
        for (cause, cle_attendue, args_attendus, mots) in &cas {
            let r = refus_pdf(cause);
            assert_eq!(r.cle, cle_attendue, "{cause:?}");
            let noms: Vec<&str> = r.args.iter().map(|(n, _)| *n).collect();
            assert_eq!(&noms, args_attendus, "{cause:?}");
            cles.insert(r.cle);
            let mut fluent = kesh_i18n::FluentArgs::new();
            for (nom, valeur) in &r.args {
                fluent.set(*nom, valeur.clone());
            }
            let fluent = (!r.args.is_empty()).then_some(fluent);
            let rendu = |l: Locale| bundle.format(&l, r.cle, fluent.as_ref());
            let fr = rendu(Locale::FrCh);
            assert_eq!(r.repli, sans_marques(fr.clone()), "repli de {}", r.cle);
            for locale in [Locale::FrCh, Locale::DeCh, Locale::ItCh, Locale::EnCh] {
                let texte = rendu(locale);
                assert_ne!(texte, r.cle, "{locale:?} : {} absente", r.cle);
                if locale != Locale::FrCh {
                    assert_ne!(texte, fr, "{locale:?} : {} retombe sur le français", r.cle);
                }
                for interdit in [
                    "{",
                    "$",
                    "Champ",
                    "creditor.",
                    "debtor.",
                    "address.",
                    "unstructured_message",
                    "billing_information",
                ] {
                    assert!(
                        !texte.contains(interdit),
                        "{locale:?} : « {interdit} » dans {} : {texte}",
                        r.cle
                    );
                }
                if let Some((partie, mot)) = mots {
                    if locale == Locale::FrCh {
                        assert!(
                            texte.contains(partie) && texte.contains(mot),
                            "{}: {texte}",
                            r.cle
                        );
                    } else {
                        // En minuscules : la phrase peut commencer par la partie
                        // (« Company address incomplete »).
                        let p = partie_hors_fr(locale, partie).to_lowercase();
                        assert!(
                            texte.to_lowercase().contains(&p),
                            "{locale:?} : {} sans « {p} » : {texte}",
                            r.cle
                        );
                    }
                }
            }
            if mots.is_some() {
                par_champ += 1;
            }
        }
        // AC8 : 27 clés par champ, 3 replis par raison, 9 causes sans champ, 3 reprises.
        assert_eq!(par_champ, 27);
        assert_eq!(cles.len(), 42, "{cles:?}");
    }

    /// D4 — le caractère s'écrit par `escape_debug` (un saut de ligne en `\n`, deux
    /// caractères) suivi de son point de code, en chaîne déjà formatée.
    #[test]
    fn le_caractere_refuse_est_echappe_et_suivi_de_son_point_de_code() {
        let r = refus_pdf(&CauseRefusPdf::CaractereRefuse {
            champ: "debtor.line2",
            codepoint: 0x0A,
        });
        assert_eq!(
            r.args,
            vec![("car", "\\n".to_string()), ("point", "U+000A".to_string())]
        );
        let r = refus_pdf(&CauseRefusPdf::CaractereRefuse {
            champ: "debtor.town",
            codepoint: 0x2013,
        });
        assert_eq!(r.args[0], ("car", "\u{2013}".to_string()));
        assert_eq!(r.args[1], ("point", "U+2013".to_string()));
    }

    /// D5 — chaque variante de `QrBillError` qui refuse en 400 passe par la cause
    /// attendue ; les trois autres gardent leur variante propre.
    #[test]
    fn map_qrbill_error_rend_la_cause_de_chaque_variante() {
        use CauseRefusPdf as C;
        let message = |e: QrBillError| match map_qrbill_error(e) {
            AppError::InvoiceNotPdfReady(m) => m,
            autre => panic!("{autre:?}"),
        };
        // Sans catalogue chargé, `refus` rend le repli : la cause se lit à son repli.
        let repli = |c: C| refus_pdf(&c).repli;
        for (erreur, cause) in [
            (QrBillError::InvalidIban("x".into()), C::IbanInvalide),
            (QrBillError::InvalidQrIban("x".into()), C::QrIbanInvalide),
            (QrBillError::InvalidQrr("x".into()), C::PanneInterne),
            (QrBillError::InvalidPayload("x".into()), C::PanneInterne),
            (QrBillError::InvalidAmount("x".into()), C::MontantInvalide),
            (QrBillError::InvalidCurrency("x".into()), C::DeviseInvalide),
            (
                QrBillError::InvalidCountry("SU (non ISO-3166-1 alpha-2)".into()),
                C::PaysInvalide { code: "SU".into() },
            ),
            (
                QrBillError::FieldEmpty("debtor.town"),
                C::ChampVide {
                    champ: "debtor.town",
                },
            ),
            (
                QrBillError::FieldTooLong {
                    field: "debtor.name",
                    max: 70,
                    got: 71,
                },
                C::ChampTropLong {
                    champ: "debtor.name",
                    max: 70,
                    got: 71,
                },
            ),
            (
                QrBillError::InvalidCharset {
                    field: "creditor.line1",
                    codepoint: 0x2013,
                },
                C::CaractereRefuse {
                    champ: "creditor.line1",
                    codepoint: 0x2013,
                },
            ),
        ] {
            assert_eq!(message(erreur), repli(cause));
        }
        assert!(matches!(
            map_qrbill_error(QrBillError::TooManyLines(12)),
            AppError::InvoiceTooManyLinesForPdf(12)
        ));
        assert!(matches!(
            map_qrbill_error(QrBillError::HeaderOverflow(1.0)),
            AppError::InvoicePdfHeaderOverflow
        ));
        assert!(matches!(
            map_qrbill_error(QrBillError::PdfGeneration("x".into())),
            AppError::PdfGenerationFailed(_)
        ));
    }

    /// Story 32-1a (#716, B3 = E3) — la clé du préfixe de la communication de la
    /// QR est exigée des quatre catalogues réels. Le chargeur rend la clé brute si
    /// elle manque partout, et se replie sur le français si elle manque à une seule
    /// langue : les deux cas laisseraient une QR lisible mais fausse. Aucun repli
    /// Rust n'existe (C-32-1-3) : c'est ce test qui tient la clé.
    #[test]
    fn qr_message_prefix_is_translated_in_all_four_locales() {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../kesh-i18n/locales");
        let bundle = kesh_i18n::I18nBundle::load(&dir).unwrap();
        let cle = "invoice-qr-message-prefix";
        let fr = bundle.format(&Locale::FrCh, cle, None);
        for locale in [Locale::FrCh, Locale::DeCh, Locale::ItCh, Locale::EnCh] {
            let prefixe = bundle.format(&locale, cle, None);
            assert!(
                !prefixe.trim().is_empty() && prefixe != cle,
                "{locale:?} : le préfixe de la QR est absent du catalogue (rendu « {prefixe} »)"
            );
            if locale != Locale::FrCh {
                assert_ne!(
                    prefixe, fr,
                    "{locale:?} : le préfixe de la QR retombe sur le français (« {fr} »)"
                );
            }
        }
    }
}
