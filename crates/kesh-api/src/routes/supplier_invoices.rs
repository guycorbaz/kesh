//! Story 12.2 — endpoints factures fournisseurs & règlement binaire (#191).
//!
//! - `GET    /api/v1/supplier-invoices`          liste paginée (tout rôle)
//! - `GET    /api/v1/supplier-invoices/{id}`     détail + lignes (tout rôle)
//! - `POST   /api/v1/supplier-invoices`          enregistrer (Comptable+, RBAC routing)
//! - `POST   /api/v1/supplier-invoices/{id}/pay` régler binaire (Comptable+)
//! - `POST   /api/v1/supplier-invoices/{id}/cancel` annuler, même payée (Comptable+)
//!
//! Les deux lectures (liste, détail) passent par `services::factures_fournisseur`
//! (Story 32-4b).
//!
//! Erreurs métier (config absente, déjà payée, compte étranger/inactif, etc.)
//! → `AppError` typée 4xx, jamais 500 (AC5/AC9).

use axum::Extension;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use chrono::{NaiveDate, NaiveDateTime};
use kesh_db::entities::{SettlementChoice, SupplierInvoice, SupplierInvoiceLine};
use kesh_db::errors::DbError;
use kesh_db::repositories::supplier_invoices;
use kesh_qrbill::{ScannedAddress, ScannedQrBill, ScannedReference, parse_spc_payload};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::errors::AppError;
use crate::helpers::get_company_for;
use crate::middleware::auth::CurrentUser;
use crate::routes::ListResponse;
use crate::routes::limits::{MAX_DECIMAL_SCALE, line_scale_error, scale_within};
use crate::services::acces::ContexteAcces;
use crate::services::factures_fournisseur::{self, DemandeListeFacturesFournisseur};

// ---------------------------------------------------------------------------
// Réponses
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierInvoiceLineResponse {
    pub position: i32,
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub vat_rate: Decimal,
    pub line_total: Decimal,
    pub expense_account_id: i64,
}

impl From<SupplierInvoiceLine> for SupplierInvoiceLineResponse {
    fn from(l: SupplierInvoiceLine) -> Self {
        Self {
            position: l.position,
            description: l.description,
            quantity: l.quantity,
            unit_price: l.unit_price,
            vat_rate: l.vat_rate,
            line_total: l.line_total,
            expense_account_id: l.expense_account_id,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierInvoiceResponse {
    pub id: i64,
    pub contact_id: i64,
    pub supplier_invoice_number: Option<String>,
    pub status: String,
    pub invoice_date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub total_amount: Decimal,
    pub creditor_iban: Option<String>,
    pub creditor_qr_iban: Option<String>,
    pub payment_reference: Option<String>,
    pub expected_payment_amount: Option<Decimal>,
    pub project_id: Option<i64>,
    pub purchase_journal_entry_id: i64,
    pub settlement_type: Option<String>,
    pub settlement_bank_account_id: Option<i64>,
    pub settlement_account_id: Option<i64>,
    pub settlement_journal_entry_id: Option<i64>,
    pub paid_at: Option<NaiveDateTime>,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub lines: Vec<SupplierInvoiceLineResponse>,
    /// Story 25-3-a-2 (#414) — le règlement peut-il être annulé ?
    ///
    /// ⚠️ **`None` veut dire « non calculé ici », jamais « non »** — même
    /// discipline que `InvoiceResponse::amount_settled` : seuls le GET, `pay` et
    /// l'annulation du règlement le calculent ([`Self::load_with_settlement_cancellation`]).
    /// Calculé par la fonction même qui refuse l'annulation : l'écran masque le
    /// bouton avant le clic. Ne tient pas compte du rôle.
    pub settlement_cancellable: Option<bool>,
    /// Code du motif qui empêche l'annulation, quand il y en a un.
    pub settlement_cancel_blocked_by: Option<&'static str>,
    /// Le numéro du compte archivé, quand c'est le motif.
    pub settlement_cancel_blocked_label: Option<String>,
    /// Le lot de paiement **confirmé le plus récent** qui contient la facture.
    ///
    /// ⛔ **Historique** : il ne dit PAS d'où vient le règlement courant
    /// (`payment_batch_items` ne relie aucune ligne à une écriture). Il sert à
    /// prévenir, avant d'annuler, que la banque a peut-être déjà exécuté un
    /// ordre pour cette facture — donc un double paiement.
    pub last_confirmed_batch: Option<LastConfirmedBatchResponse>,
    /// Story 25-3-c (#454) — la **facture** peut-elle être annulée ?
    ///
    /// ⚠️ Même discipline que `settlement_cancellable` : **`None` = non
    /// calculé ici**, jamais « non ». Calculé par la fonction même qui refuse
    /// (`supplier_invoices::supplier_invoice_cancel_blocker`), dans le même
    /// instantané que la facture. Ne tient pas compte du rôle.
    pub cancellable: Option<bool>,
    /// Code du motif qui empêche l'annulation de la facture.
    pub cancel_blocked_by: Option<&'static str>,
    /// Le numéro du compte archivé, quand c'est le motif.
    pub cancel_blocked_label: Option<String>,
    /// Story 30-2c (AC18) — le document de la facture, à **trois états** :
    /// - champ **absent** (`None`) : non calculé ici — réponses de
    ///   [`Self::from_parts`], dont `POST /supplier-invoices` ;
    /// - `null` (`Some(None)`) : la facture n'a pas de document ;
    /// - l'objet (`Some(Some(_))`) : son nom, son type et sa taille.
    ///
    /// Même discipline que `settlement_cancellable` (« non calculé n'est jamais
    /// non »), à ceci près qu'ici l'absence se distingue de `null`. Calculé par
    /// [`Self::load_with_settlement_cancellation`], dans l'instantané du GET.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_document: Option<Option<SourceDocumentInfo>>,
}

/// Ce que la fiche d'une facture fournisseur dit de son document (Story 30-2c,
/// AC18) — jamais son chemin de stockage ni son empreinte.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceDocumentInfo {
    pub original_filename: String,
    pub mime_type: String,
    pub byte_size: i64,
}

impl From<kesh_db::entities::imported_supplier_invoice::ImportedSupplierInvoice>
    for SourceDocumentInfo
{
    fn from(d: kesh_db::entities::imported_supplier_invoice::ImportedSupplierInvoice) -> Self {
        Self {
            original_filename: d.original_filename,
            mime_type: d.mime_type,
            byte_size: d.byte_size,
        }
    }
}

/// Le lot confirmé le plus récent qui contient une facture (Story 25-3-a-2).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LastConfirmedBatchResponse {
    pub id: i64,
    pub confirmed_at: Option<NaiveDateTime>,
}

impl SupplierInvoiceResponse {
    /// `pub` (Story 12-5c) : appelé par le module `imported_supplier_invoices`
    /// pour construire la réponse de complétion à partir du `SupplierInvoiceWithLines`
    /// retourné par `create_in_tx` (DRY — pas de redéfinition cross-module).
    pub fn from_parts(inv: SupplierInvoice, lines: Vec<SupplierInvoiceLine>) -> Self {
        Self {
            id: inv.id,
            contact_id: inv.contact_id,
            supplier_invoice_number: inv.supplier_invoice_number,
            status: inv.status,
            invoice_date: inv.invoice_date,
            due_date: inv.due_date,
            total_amount: inv.total_amount,
            creditor_iban: inv.creditor_iban,
            creditor_qr_iban: inv.creditor_qr_iban,
            payment_reference: inv.payment_reference,
            expected_payment_amount: inv.expected_payment_amount,
            project_id: inv.project_id,
            purchase_journal_entry_id: inv.purchase_journal_entry_id,
            settlement_type: inv.settlement_type,
            settlement_bank_account_id: inv.settlement_bank_account_id,
            settlement_account_id: inv.settlement_account_id,
            settlement_journal_entry_id: inv.settlement_journal_entry_id,
            paid_at: inv.paid_at,
            version: inv.version,
            created_at: inv.created_at,
            lines: lines.into_iter().map(Into::into).collect(),
            settlement_cancellable: None,
            settlement_cancel_blocked_by: None,
            settlement_cancel_blocked_label: None,
            last_confirmed_batch: None,
            cancellable: None,
            cancel_blocked_by: None,
            cancel_blocked_label: None,
            source_document: None,
        }
    }

    /// Story 25-3-a-2 (#414) — la réponse complète, champs d'annulation du
    /// règlement et dernier lot confirmé compris, **depuis un seul instantané**
    /// ([`supplier_invoices::get_settlement_view`]). ⚠️ Servie par le GET, `pay`
    /// (dont la réponse remplace l'état de l'écran), l'annulation de la facture
    /// et celle du règlement, la complétion d'un document (Story 30-2b) et, depuis
    /// la Story 30-2c, le rattachement et le détachement d'un document — le
    /// document lui-même (`source_document`) compris ; ailleurs, les champs
    /// restent `None` — « non calculé », jamais une valeur qui mentirait. Après une écriture, la facture est **relue** :
    /// la réponse décrit l'état courant, cohérent avec ses propres champs.
    pub async fn load_with_settlement_cancellation(
        pool: &sqlx::MySqlPool,
        company_id: i64,
        id: i64,
    ) -> Result<Self, AppError> {
        let view = supplier_invoices::get_settlement_view(pool, company_id, id)
            .await?
            .ok_or(AppError::Database(DbError::NotFound))?;
        Ok(Self::from_view(view))
    }

    /// Story 32-4b (C-32-4b-5) — la mise en forme **pure** d'une vue de
    /// règlement : la seule, employée par
    /// [`Self::load_with_settlement_cancellation`] (les mutants, et les trois
    /// sites de `imported_supplier_invoices.rs`) et par le handler de la fiche,
    /// qui lit la vue par `services::factures_fournisseur::detail`.
    pub fn from_view(view: supplier_invoices::SupplierInvoiceSettlementView) -> Self {
        let hit = view.settlement_cancel_blocker;
        let invoice_hit = view.invoice_cancel_blocker;
        let mut resp = Self::from_parts(view.invoice, view.lines);
        resp.settlement_cancellable = Some(hit.is_none());
        resp.settlement_cancel_blocked_by = hit.as_ref().map(|h| h.0.code());
        resp.settlement_cancel_blocked_label = hit.and_then(|h| h.2);
        resp.cancellable = Some(invoice_hit.is_none());
        resp.cancel_blocked_by = invoice_hit.as_ref().map(|h| h.0.code());
        resp.cancel_blocked_label = invoice_hit.and_then(|h| h.2);
        resp.last_confirmed_batch = view
            .last_confirmed_batch
            .map(|(id, confirmed_at)| LastConfirmedBatchResponse { id, confirmed_at });
        resp.source_document = Some(view.source_document.map(SourceDocumentInfo::from));
        resp
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierInvoiceListItemResponse {
    pub id: i64,
    pub contact_id: i64,
    pub contact_name: String,
    pub supplier_invoice_number: Option<String>,
    pub status: String,
    pub invoice_date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub total_amount: Decimal,
}

impl From<kesh_db::repositories::supplier_invoices::SupplierInvoiceListItem>
    for SupplierInvoiceListItemResponse
{
    fn from(it: kesh_db::repositories::supplier_invoices::SupplierInvoiceListItem) -> Self {
        Self {
            id: it.id,
            contact_id: it.contact_id,
            contact_name: it.contact_name,
            supplier_invoice_number: it.supplier_invoice_number,
            status: it.status,
            invoice_date: it.invoice_date,
            due_date: it.due_date,
            total_amount: it.total_amount,
        }
    }
}

// ---------------------------------------------------------------------------
// Requêtes
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSupplierInvoiceLineRequest {
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub vat_rate: Decimal,
    pub expense_account_id: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSupplierInvoiceRequest {
    pub contact_id: i64,
    pub supplier_invoice_number: Option<String>,
    pub invoice_date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub creditor_iban: Option<String>,
    pub creditor_qr_iban: Option<String>,
    pub payment_reference: Option<String>,
    pub expected_payment_amount: Option<Decimal>,
    /// Projet analytique (Epic 19, Story 19-3) — document-level, optionnel.
    pub project_id: Option<i64>,
    pub lines: Vec<CreateSupplierInvoiceLineRequest>,
}

// ---------------------------------------------------------------------------
// Story 30-2b (#685) — un seul corps, une seule conversion, une seule validation
// ---------------------------------------------------------------------------

/// Longueur maximale d'une référence de paiement libre — colonne
/// `supplier_invoices.payment_reference VARCHAR(64)`.
const PAYMENT_REFERENCE_MAX_CHARS: usize = 64;

/// Retire d'une coordonnée de paiement l'ensemble de blancs **écrit** de
/// l'AC12-bis : les caractères `White_Space` d'Unicode (`char::is_whitespace` —
/// espace, tabulation, U+00A0, U+202F, U+0085… compris) plus U+200B (espace de
/// largeur nulle) et U+FEFF (BOM).
///
/// ⚠️ La même liste est tenue côté écran par `normalizeIban`
/// (`frontend/src/lib/shared/utils/iban.ts`), et un test de chaque côté porte
/// les mêmes sept caractères : `\s` en JavaScript et `is_whitespace` en Rust ne
/// coïncident pas d'eux-mêmes. `Iban::new` n'ôte que l'espace ASCII et refuse
/// tout non-ASCII : sans cette normalisation, un IBAN collé depuis un PDF (avec
/// des insécables) serait refusé.
pub(crate) fn normalize_payment_field(raw: &str) -> String {
    raw.chars().filter(|c| !is_payment_blank(*c)).collect()
}

/// Un caractère de l'ensemble de blancs de l'AC12-bis — **seule** définition côté
/// serveur, partagée par [`normalize_payment_field`] (qui les retire tous) et par le
/// rognage de la référence libre (qui ne les retire qu'aux extrémités : revue P1 de
/// la 30-2b, B3 — un U+200B final collé depuis un PDF n'est plus stocké ni compté).
fn is_payment_blank(c: char) -> bool {
    c.is_whitespace() || c == '\u{200B}' || c == '\u{FEFF}'
}

/// Une coordonnée de paiement **présente** : sa forme normalisée n'est pas vide.
/// Une chaîne vide ou faite de blancs vaut absente (AC11, AC12-bis).
fn present(raw: Option<&str>) -> Option<&str> {
    raw.filter(|s| !normalize_payment_field(s).is_empty())
}

/// Les coordonnées de paiement validées et normalisées d'une facture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PaymentDetails {
    pub creditor_iban: Option<String>,
    pub creditor_qr_iban: Option<String>,
    pub payment_reference: Option<String>,
}

fn invalid(field: &'static str, reason: &'static str) -> AppError {
    AppError::InvalidPaymentDetails { field, reason }
}

/// Valide les coordonnées de paiement saisies (AC12-bis, C-30-2-20) — **seule**
/// définition côté serveur, appelée par la conversion commune des deux routes.
///
/// - `creditorIban` : `kesh_core::types::Iban` (format, ASCII, pays de la table
///   intégrée et longueur propre à ce pays, MOD-97), et **pas** un QR-IBAN ;
/// - `creditorQrIban` : `kesh_core::types::QrIban` (IBAN CH/LI, QR-IID
///   30000-31999) — une valeur qui n'est pas un IBAN rend `INVALID_IBAN`, un IBAN
///   hors plage QR `NOT_A_QR_IBAN` ;
/// - pas les deux ensemble ;
/// - référence : avec un QR-IBAN, **obligatoire** et QRR valide
///   (`validate_qrr`, blancs retirés) ; sinon texte libre rogné de 64
///   caractères au plus, 27 chiffres compris (ancienne référence BVR/ESR) —
///   jamais de refus « forme QRR ».
///
/// Valeurs rendues normalisées (`Iban::as_str`, QRR sans blancs, texte rogné).
///
/// ⚠️ Angle mort écrit : le **montant attendu** (`expectedPaymentAmount`) n'est
/// pas validé ici — ni en signe, ni en échelle, ni contre le total des lignes
/// (comportement antérieur à la 30-2b, gardé ; écrit au manuel et à
/// `docs/api-external.md`).
pub(crate) fn validate_payment_details(
    creditor_iban: Option<&str>,
    creditor_qr_iban: Option<&str>,
    payment_reference: Option<&str>,
) -> Result<PaymentDetails, AppError> {
    use kesh_core::errors::CoreError;
    use kesh_core::types::{Iban, QrIban};

    let iban = present(creditor_iban).map(normalize_payment_field);
    let qr_iban = present(creditor_qr_iban).map(normalize_payment_field);
    let reference = present(payment_reference);

    if iban.is_some() && qr_iban.is_some() {
        return Err(invalid("creditorIban", "BOTH_IBANS"));
    }

    let creditor_iban = match iban {
        None => None,
        Some(raw) => {
            let parsed = Iban::new(&raw).map_err(|_| invalid("creditorIban", "INVALID_IBAN"))?;
            if QrIban::new(parsed.as_str()).is_ok() {
                return Err(invalid("creditorIban", "QR_IBAN_IN_IBAN_FIELD"));
            }
            Some(parsed.as_str().to_string())
        }
    };

    let creditor_qr_iban = match qr_iban {
        None => None,
        Some(raw) => match QrIban::new(&raw) {
            Ok(q) => Some(q.as_str().to_string()),
            Err(CoreError::InvalidIban(_)) => {
                return Err(invalid("creditorQrIban", "INVALID_IBAN"));
            }
            Err(_) => return Err(invalid("creditorQrIban", "NOT_A_QR_IBAN")),
        },
    };

    let payment_reference = if creditor_qr_iban.is_some() {
        let qrr = reference
            .map(normalize_payment_field)
            .ok_or_else(|| invalid("paymentReference", "QRR_REQUIRED"))?;
        kesh_qrbill::validation::validate_qrr(&qrr)
            .map_err(|_| invalid("paymentReference", "INVALID_QRR"))?;
        Some(qrr)
    } else {
        match reference {
            None => None,
            Some(r) => {
                // Rognée du même ensemble de blancs que `present()`, longueur comptée
                // APRÈS ce rognage et en caractères (la colonne est `VARCHAR(64)`).
                let trimmed = r.trim_matches(is_payment_blank);
                if trimmed.chars().count() > PAYMENT_REFERENCE_MAX_CHARS {
                    return Err(invalid("paymentReference", "REFERENCE_TOO_LONG"));
                }
                Some(trimmed.to_string())
            }
        }
    };

    Ok(PaymentDetails {
        creditor_iban,
        creditor_qr_iban,
        payment_reference,
    })
}

/// D'où viennent les coordonnées de paiement de la facture créée.
pub(crate) enum PaymentSource {
    /// Du corps, validées par [`validate_payment_details`] : `POST
    /// /supplier-invoices` et la complétion d'un document **sans** QR.
    Body,
    /// Du QR-code du document (complétion d'un document **avec** QR) : lues sur
    /// la ligne de staging **verrouillée**, déjà validées au décodage. Le corps
    /// ne doit en porter aucune (AC11, `QR_FIELDS_FROM_DOCUMENT`).
    Document {
        payment: PaymentDetails,
        expected_payment_amount: Option<Decimal>,
    },
}

impl CreateSupplierInvoiceRequest {
    /// Vrai si le corps porte l'une des quatre coordonnées que fournit un QR
    /// (une chaîne vide ou blanche vaut absente).
    fn carries_payment_fields(&self) -> bool {
        present(self.creditor_iban.as_deref()).is_some()
            || present(self.creditor_qr_iban.as_deref()).is_some()
            || present(self.payment_reference.as_deref()).is_some()
            || self.expected_payment_amount.is_some()
    }

    /// La conversion **commune** des deux routes (AC13, C-30-2-10) : `POST
    /// /supplier-invoices` et `POST /imported-supplier-invoices/{id}/complete`
    /// construisent leur `NewSupplierInvoice` ici, projet compris, et nulle part
    /// ailleurs. Porte la validation de l'AC12-bis (source [`PaymentSource::Body`])
    /// et le refus `QR_FIELDS_FROM_DOCUMENT` (source [`PaymentSource::Document`]).
    ///
    /// Story v017-3b (#687, AC-2) — **avant** le `match` sur la source de
    /// paiement : la quantité et le prix de chaque ligne portent au plus
    /// [`MAX_DECIMAL_SCALE`] décimales **après `normalize()`** (`400
    /// VALIDATION_ERROR` qui nomme la ligne et le champ), ligne par ligne, la
    /// quantité avant le prix. Un défaut de saisie de ligne se dit avant un
    /// défaut de coordonnées. ⚠️ Écart voulu avec la facture client, qui juge
    /// l'échelle **sans** normaliser (`routes/invoices.rs`) : `2.50000` y est
    /// refusé, accepté ici — la valeur est la même, et MariaDB la stocke sans
    /// perte. Sans cette borne, une quantité ou un prix à cinq décimales était
    /// arrondi en silence par la colonne `DECIMAL(19,4)` alors que `line_total`
    /// était calculé sur la valeur reçue.
    pub(crate) fn to_new_supplier_invoice(
        &self,
        company_id: i64,
        source: PaymentSource,
    ) -> Result<kesh_db::entities::NewSupplierInvoice, AppError> {
        for (index, line) in self.lines.iter().enumerate() {
            if !scale_within(&line.quantity.normalize(), MAX_DECIMAL_SCALE) {
                return Err(line_scale_error(index, "la quantité"));
            }
            if !scale_within(&line.unit_price.normalize(), MAX_DECIMAL_SCALE) {
                return Err(line_scale_error(index, "le prix unitaire"));
            }
        }
        let (payment, expected_payment_amount) = match source {
            PaymentSource::Body => (
                validate_payment_details(
                    self.creditor_iban.as_deref(),
                    self.creditor_qr_iban.as_deref(),
                    self.payment_reference.as_deref(),
                )?,
                self.expected_payment_amount,
            ),
            PaymentSource::Document {
                payment,
                expected_payment_amount,
            } => {
                if self.carries_payment_fields() {
                    return Err(AppError::QrFieldsFromDocument);
                }
                (payment, expected_payment_amount)
            }
        };
        Ok(kesh_db::entities::NewSupplierInvoice {
            company_id,
            contact_id: self.contact_id,
            supplier_invoice_number: self.supplier_invoice_number.clone(),
            invoice_date: self.invoice_date,
            due_date: self.due_date,
            creditor_iban: payment.creditor_iban,
            creditor_qr_iban: payment.creditor_qr_iban,
            payment_reference: payment.payment_reference,
            expected_payment_amount,
            project_id: self.project_id,
            lines: self
                .lines
                .iter()
                .map(|l| kesh_db::entities::NewSupplierInvoiceLine {
                    description: l.description.clone(),
                    quantity: l.quantity,
                    unit_price: l.unit_price,
                    vat_rate: l.vat_rate,
                    expense_account_id: l.expense_account_id,
                })
                .collect(),
        })
    }
}

/// Requête de règlement binaire. `settlementType` = `bank_transfer` (→ `bankAccountId`)
/// ou `internal_account` (→ `accountId`).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaySupplierInvoiceRequest {
    pub settlement_type: String,
    pub bank_account_id: Option<i64>,
    pub account_id: Option<i64>,
    pub payment_date: NaiveDate,
}

impl PaySupplierInvoiceRequest {
    fn into_choice(self) -> Result<(SettlementChoice, NaiveDate), AppError> {
        let choice = match self.settlement_type.as_str() {
            "bank_transfer" => {
                let bank_account_id = self.bank_account_id.ok_or_else(|| {
                    AppError::Validation("bankAccountId requis pour un virement".into())
                })?;
                SettlementChoice::BankTransfer { bank_account_id }
            }
            "internal_account" => {
                let account_id = self.account_id.ok_or_else(|| {
                    AppError::Validation("accountId requis pour un compte interne".into())
                })?;
                SettlementChoice::InternalAccount { account_id }
            }
            other => {
                return Err(AppError::Validation(format!(
                    "settlementType inconnu : '{other}' (attendu bank_transfer | internal_account)"
                )));
            }
        };
        Ok((choice, self.payment_date))
    }
}

#[derive(Deserialize)]
pub struct ListSupplierInvoicesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `GET /api/v1/supplier-invoices` — liste paginée (les plus récentes d'abord).
pub async fn list_supplier_invoices(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<ListSupplierInvoicesQuery>,
) -> Result<Json<ListResponse<SupplierInvoiceListItemResponse>>, AppError> {
    // Story 32-4b — la société, les bornes (écrêtage) et la page passent par le
    // service (`services::factures_fournisseur::lister`).
    let page = factures_fournisseur::lister(
        &ContexteAcces::depuis_api(&current_user),
        &state.lecture(),
        DemandeListeFacturesFournisseur {
            limit: params.limit,
            offset: params.offset,
        },
    )
    .await?;
    Ok(Json(ListResponse {
        items: page.items.into_iter().map(Into::into).collect(),
        total: page.total,
        offset: page.offset,
        limit: page.limit,
    }))
}

/// `GET /api/v1/supplier-invoices/{id}` — détail + lignes.
pub async fn get_supplier_invoice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<SupplierInvoiceResponse>, AppError> {
    // Story 32-4b — la société et la vue passent par le service
    // (`services::factures_fournisseur::detail`) ; la mise en forme est celle de
    // `load_with_settlement_cancellation` (`from_view`).
    let vue = factures_fournisseur::detail(
        &ContexteAcces::depuis_api(&current_user),
        &state.lecture(),
        id,
    )
    .await?;
    Ok(Json(SupplierInvoiceResponse::from_view(vue)))
}

/// `POST /api/v1/supplier-invoices` — enregistre une facture fournisseur (Comptable+).
///
/// ⚠️ **Rejouée sur interblocage** (Story 15-5e1, choix C66) par l'enveloppe
/// `DbError` [`kesh_db::retry::retry_on_deadlock`] ; le `NewSupplierInvoice` est
/// cloné à chaque tentative.
pub async fn create_supplier_invoice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateSupplierInvoiceRequest>,
) -> Result<(StatusCode, Json<SupplierInvoiceResponse>), AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    // Story 30-2b (AC12-bis, AC13) : conversion commune, coordonnées validées.
    let new = req.to_new_supplier_invoice(company.id, PaymentSource::Body)?;
    let created = kesh_db::retry::retry_on_deadlock("supplier_invoices::create", || {
        supplier_invoices::create(&state.pool, new.clone(), current_user.user_id)
    })
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(SupplierInvoiceResponse::from_parts(
            created.invoice,
            created.lines,
        )),
    ))
}

/// `POST /api/v1/supplier-invoices/{id}/pay` — règlement binaire (Comptable+).
///
/// ⚠️ **Rejouée sur interblocage** (Story 15-5e2) par l'enveloppe `DbError`
/// [`kesh_db::retry::retry_on_deadlock`].
pub async fn pay_supplier_invoice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<PaySupplierInvoiceRequest>,
) -> Result<Json<SupplierInvoiceResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let (choice, payment_date) = req.into_choice()?;
    kesh_db::retry::retry_on_deadlock("supplier_invoices::pay", || {
        supplier_invoices::pay(
            &state.pool,
            company.id,
            id,
            choice,
            payment_date,
            current_user.user_id,
        )
    })
    .await?;
    Ok(Json(
        SupplierInvoiceResponse::load_with_settlement_cancellation(&state.pool, company.id, id)
            .await?,
    ))
}

/// `POST /api/v1/supplier-invoices/{id}/cancel` — annule une facture
/// fournisseur, **ouverte ou payée** (Comptable+, Story 25-3-c, #454) :
/// contre-passe l'écriture d'achat par le socle ; payée, son règlement reste au
/// grand livre, détaché. La réponse est relue avec ses champs de lecture, dans
/// un seul instantané.
///
/// ⚠️ **Rejouée sur interblocage** (Story 15-5e2) par l'enveloppe `DbError`
/// [`kesh_db::retry::retry_on_deadlock`].
pub async fn cancel_supplier_invoice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<SupplierInvoiceResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    kesh_db::retry::retry_on_deadlock("supplier_invoices::cancel", || {
        supplier_invoices::cancel(&state.pool, company.id, id, current_user.user_id)
    })
    .await?;
    Ok(Json(
        SupplierInvoiceResponse::load_with_settlement_cancellation(&state.pool, company.id, id)
            .await?,
    ))
}

/// Réponse de l'annulation d'un règlement fournisseur : la facture relue, et
/// l'écriture inverse produite (Story 25-3-a-2).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelSupplierSettlementResponse {
    pub invoice: SupplierInvoiceResponse,
    pub reversal_journal_entry_id: i64,
}

/// `POST /api/v1/supplier-invoices/{id}/settlement/cancel` — annule le
/// règlement d'une facture `paid` par contre-passation datée du jour et la
/// ramène à `open` (Comptable+, Story 25-3-a-2, #414). Le lot de paiement
/// confirmé qui l'a éventuellement réglée n'est pas modifié.
///
/// ⚠️ **Rejouée sur interblocage** (Story 15-5e2) par l'enveloppe `DbError`
/// [`kesh_db::retry::retry_on_deadlock`].
pub async fn cancel_supplier_invoice_settlement(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<CancelSupplierSettlementResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let done = kesh_db::retry::retry_on_deadlock("supplier_invoices::cancel_settlement", || {
        supplier_invoices::cancel_settlement(&state.pool, company.id, id, current_user.user_id)
    })
    .await?;
    Ok(Json(CancelSupplierSettlementResponse {
        invoice: SupplierInvoiceResponse::load_with_settlement_cancellation(
            &state.pool,
            company.id,
            id,
        )
        .await?,
        reversal_journal_entry_id: done.reversal_journal_entry_id,
    }))
}

// ─────────────────────────────────────────────────────────────────────────────
// Story 12.4 — scan / import du QR-facture (pré-remplissage)
// ─────────────────────────────────────────────────────────────────────────────

/// Corps de `POST /api/v1/supplier-invoices/scan-qr` : le texte SPC brut décodé
/// **côté navigateur** (jsQR, DC1 — pas de caméra ni de décodage image serveur).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanQrRequest {
    pub spc_text: String,
}

/// Coordonnées extraites d'un QR-facture, destinées à pré-remplir le formulaire
/// de facture fournisseur (12-2). **Lecture seule** — rien n'est persisté.
///
/// Exactement l'un de `creditor_iban` / `creditor_qr_iban` est renseigné, selon
/// que l'IBAN est un QR-IBAN (plage IID 30000–31999) ou un IBAN classique
/// (AC4 / mapping `supplier_invoices`).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanQrResponse {
    pub creditor_iban: Option<String>,
    pub creditor_qr_iban: Option<String>,
    pub payment_reference: Option<String>,
    pub expected_payment_amount: Option<Decimal>,
    pub currency: String,
    pub creditor_name: String,
    pub creditor_address: Option<String>,
    pub unstructured_message: Option<String>,
}

/// Recompose une adresse lisible (indice pour retrouver/créer le fournisseur) à
/// partir du bloc `ScannedAddress` — combiné (`K`, deux lignes libres) ou
/// structuré (`S`, rue/NPA/localité). `None` si aucun élément d'adresse.
fn format_scanned_address(a: &ScannedAddress) -> Option<String> {
    let parts: Vec<&str> = [
        a.street_or_line1.as_str(),
        a.building_or_line2.as_str(),
        a.postal_code.as_deref().unwrap_or(""),
        a.town.as_deref().unwrap_or(""),
        a.country.as_str(),
    ]
    .into_iter()
    .map(str::trim)
    .filter(|s| !s.is_empty())
    .collect();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(", "))
    }
}

impl ScanQrResponse {
    fn from_scanned(s: ScannedQrBill) -> Self {
        // QR-IBAN → creditor_qr_iban ; IBAN classique → creditor_iban (AC4).
        let (creditor_iban, creditor_qr_iban) = if s.is_qr_iban {
            (None, Some(s.creditor_iban))
        } else {
            (Some(s.creditor_iban), None)
        };
        // QRR comme SCOR fournissent une référence de paiement ; NON → aucune.
        let payment_reference = match s.reference {
            ScannedReference::Qrr(r) | ScannedReference::Scor(r) => Some(r),
            ScannedReference::None => None,
        };
        Self {
            creditor_iban,
            creditor_qr_iban,
            payment_reference,
            expected_payment_amount: s.amount,
            currency: s.currency,
            creditor_address: format_scanned_address(&s.creditor),
            creditor_name: s.creditor.name,
            unstructured_message: s.unstructured_message,
        }
    }
}

/// `POST /api/v1/supplier-invoices/scan-qr` — pré-remplissage par scan QR (Comptable+).
///
/// Prend le texte SPC décodé côté navigateur et retourne les coordonnées de
/// paiement (IBAN/QR-IBAN, référence, montant attendu, créancier). **Ne crée
/// rien** : pure transformation en lecture seule. Un payload SPC invalide (en-tête,
/// IBAN, référence…) → `400 VALIDATION_ERROR`, **jamais 500** (AC3).
pub async fn scan_qr_supplier_invoice(
    Json(req): Json<ScanQrRequest>,
) -> Result<Json<ScanQrResponse>, AppError> {
    let scanned = parse_spc_payload(&req.spc_text)
        .map_err(|e| AppError::Validation(format!("QR-facture illisible : {e}")))?;
    Ok(Json(ScanQrResponse::from_scanned(scanned)))
}

#[cfg(test)]
mod scan_qr_tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn addr(name: &str) -> ScannedAddress {
        ScannedAddress {
            address_type: 'K',
            name: name.to_string(),
            street_or_line1: "Rue du Test 1".to_string(),
            building_or_line2: "1000 Lausanne".to_string(),
            postal_code: None,
            town: None,
            country: "CH".to_string(),
        }
    }

    #[test]
    fn qr_iban_maps_to_creditor_qr_iban_with_qrr() {
        let scanned = ScannedQrBill {
            creditor_iban: "CH4431999123000889012".to_string(),
            is_qr_iban: true,
            creditor: addr("Fournisseur QR SA"),
            amount: Some(dec!(199.95)),
            currency: "CHF".to_string(),
            reference: ScannedReference::Qrr("210000000003139471430009017".to_string()),
            unstructured_message: Some("Facture 42".to_string()),
            billing_information: None,
        };
        let r = ScanQrResponse::from_scanned(scanned);
        assert_eq!(r.creditor_qr_iban.as_deref(), Some("CH4431999123000889012"));
        assert!(r.creditor_iban.is_none());
        assert_eq!(
            r.payment_reference.as_deref(),
            Some("210000000003139471430009017")
        );
        assert_eq!(r.expected_payment_amount, Some(dec!(199.95)));
        assert_eq!(r.currency, "CHF");
        assert_eq!(r.creditor_name, "Fournisseur QR SA");
        assert_eq!(r.unstructured_message.as_deref(), Some("Facture 42"));
        assert_eq!(
            r.creditor_address.as_deref(),
            Some("Rue du Test 1, 1000 Lausanne, CH")
        );
    }

    #[test]
    fn classic_iban_maps_to_creditor_iban_and_open_amount() {
        let scanned = ScannedQrBill {
            creditor_iban: "CH9300762011623852957".to_string(),
            is_qr_iban: false,
            creditor: addr("Fournisseur IBAN SA"),
            amount: None, // montant ouvert
            currency: "CHF".to_string(),
            reference: ScannedReference::None,
            unstructured_message: None,
            billing_information: None,
        };
        let r = ScanQrResponse::from_scanned(scanned);
        assert_eq!(r.creditor_iban.as_deref(), Some("CH9300762011623852957"));
        assert!(r.creditor_qr_iban.is_none());
        assert!(r.payment_reference.is_none());
        assert!(r.expected_payment_amount.is_none());
    }

    #[test]
    fn scor_reference_is_exposed_as_payment_reference() {
        let scanned = ScannedQrBill {
            creditor_iban: "CH9300762011623852957".to_string(),
            is_qr_iban: false,
            creditor: addr("Fournisseur SCOR"),
            amount: Some(dec!(50.00)),
            currency: "CHF".to_string(),
            reference: ScannedReference::Scor("RF18539007547034".to_string()),
            unstructured_message: None,
            billing_information: None,
        };
        let r = ScanQrResponse::from_scanned(scanned);
        assert_eq!(r.payment_reference.as_deref(), Some("RF18539007547034"));
    }

    #[test]
    fn empty_address_lines_yield_none() {
        let a = ScannedAddress {
            address_type: 'S',
            name: "X".to_string(),
            street_or_line1: "  ".to_string(),
            building_or_line2: String::new(),
            postal_code: None,
            town: None,
            country: String::new(),
        };
        assert!(format_scanned_address(&a).is_none());
    }
}
