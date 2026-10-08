//! Routes `GET`/`PUT /api/v1/company/invoice-settings` (Story 5.2 — FR35).
//!
//! - `GET` : tout rôle authentifié (lecture config). Crée la row avec les
//!   DEFAULT si absente (pattern upsert read, cf. repository).
//! - `PUT` : Admin uniquement (paramétrage société).
//!
//! La validation métier (format, comptes Asset/Revenue actifs,
//! journal whitelist) vit ici. Le repository ne fait que persister +
//! auditer.
//!
//! **Postabilité à la désignation** (Story 15-5b, #429, choix C3) : tous les
//! comptes désignés ici — les six champs historiques comme les comptes
//! d'arrondi et d'écart soldé — sont refusés **non imputables** (compte de
//! regroupement, de résultat ou de clôture) quand leur valeur **change**. Une
//! valeur inchangée n'est pas re-contrôlée sur ce point (exemption « inchangé »,
//! patron de [`resolve_designated_account`]) : un compte devenu non imputable
//! après coup ne bloque pas l'enregistrement des réglages. Ce n'est **pas** une
//! garde à l'usage — celle des comptes de réglage est portée par la Story 15-5d.

use axum::extract::State;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};

use kesh_core::invoice_format;
use kesh_db::entities::{
    CompanyInvoiceSettings, CompanyInvoiceSettingsUpdate, Journal, account::AccountType,
};
use kesh_db::repositories::{accounts, company_invoice_settings};

use crate::AppState;
use crate::errors::AppError;
use crate::helpers::get_company_for;
use crate::middleware::auth::CurrentUser;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceSettingsResponse {
    pub company_id: i64,
    pub invoice_number_format: String,
    pub default_receivable_account_id: Option<i64>,
    pub default_revenue_account_id: Option<i64>,
    pub default_vat_payable_account_id: Option<i64>,
    pub default_vat_recoverable_account_id: Option<i64>,
    pub default_vat_decompte_account_id: Option<i64>,
    pub default_sales_journal: String,
    pub journal_entry_description_template: String,
    pub credit_note_number_format: String,
    pub default_payable_account_id: Option<i64>,
    /// Story 25-4-c3-a1 (#476) — compte de différences d'arrondi.
    pub default_rounding_account_id: Option<i64>,
    /// Story 25-4-c4-b (#494) — arrondir à 5 centimes le total des pièces émises.
    pub round_to_5_centimes: bool,
    /// Story 25-4-e (#495) — montant minimum d'une facture ; `null` = aucun seuil.
    pub minimum_invoice_amount: Option<rust_decimal::Decimal>,
    /// Story 25-4-d1 (#384) — comptes des natures d'écart soldé.
    pub default_discount_account_id: Option<i64>,
    pub default_bank_fees_account_id: Option<i64>,
    pub default_bad_debt_account_id: Option<i64>,
    pub version: i32,
}

impl From<CompanyInvoiceSettings> for InvoiceSettingsResponse {
    fn from(s: CompanyInvoiceSettings) -> Self {
        Self {
            company_id: s.company_id,
            invoice_number_format: s.invoice_number_format,
            default_receivable_account_id: s.default_receivable_account_id,
            default_revenue_account_id: s.default_revenue_account_id,
            default_vat_payable_account_id: s.default_vat_payable_account_id,
            default_vat_recoverable_account_id: s.default_vat_recoverable_account_id,
            default_vat_decompte_account_id: s.default_vat_decompte_account_id,
            default_sales_journal: s.default_sales_journal.as_str().to_string(),
            journal_entry_description_template: s.journal_entry_description_template,
            credit_note_number_format: s.credit_note_number_format,
            default_payable_account_id: s.default_payable_account_id,
            default_rounding_account_id: s.default_rounding_account_id,
            round_to_5_centimes: s.round_to_5_centimes,
            minimum_invoice_amount: s.minimum_invoice_amount,
            default_discount_account_id: s.default_discount_account_id,
            default_bank_fees_account_id: s.default_bank_fees_account_id,
            default_bad_debt_account_id: s.default_bad_debt_account_id,
            version: s.version,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInvoiceSettingsRequest {
    pub invoice_number_format: String,
    pub default_receivable_account_id: Option<i64>,
    pub default_revenue_account_id: Option<i64>,
    pub default_vat_payable_account_id: Option<i64>,
    pub default_vat_recoverable_account_id: Option<i64>,
    pub default_vat_decompte_account_id: Option<i64>,
    pub default_sales_journal: String,
    pub journal_entry_description_template: String,
    /// #216 — optionnel + **préservé** si absent : le formulaire frontend ne le
    /// gère pas encore ; sans ça, tout enregistrement des réglages échouait en 422.
    #[serde(default)]
    pub credit_note_number_format: Option<String>,
    /// Story 12.2 — compte créanciers. **Absent du corps : préservé** (Story
    /// 15-5b, #521, choix C25 — patron du compte d'arrondi) : l'écran
    /// *Paramètres → Facturation* ne l'envoie pas, et chaque enregistrement
    /// l'effaçait, si bien que la saisie d'une facture fournisseur échouait
    /// ensuite en `ConfigurationRequired`. **Présent à `null` : effacé.**
    #[serde(default, deserialize_with = "crate::helpers::double_option")]
    pub default_payable_account_id: Option<Option<i64>>,
    /// Story 25-4-c3-a1 (#476) — compte de différences d'arrondi. **Absent du
    /// corps : préservé** (comme `credit_note_number_format`, #216), pour qu'un
    /// client qui ignore encore le champ — clé d'API, onglet ouvert avant la mise
    /// à jour — ne l'efface pas en silence. **Présent à `null` : effacé.**
    #[serde(default, deserialize_with = "crate::helpers::double_option")]
    pub default_rounding_account_id: Option<Option<i64>>,
    /// Story 25-4-c4-b (#494) — arrondir à 5 centimes. **Absent du corps :
    /// préservé**, pour la même raison que le compte d'arrondi.
    #[serde(default)]
    pub round_to_5_centimes: Option<bool>,
    /// Story 25-4-e (#495) — montant minimum d'une facture. **Absent : préservé ;
    /// présent à `null` : effacé** (aucun seuil), patron du compte d'arrondi.
    #[serde(default, deserialize_with = "crate::helpers::double_option")]
    pub minimum_invoice_amount: Option<Option<rust_decimal::Decimal>>,
    /// Story 25-4-d1 (#384) — comptes des natures d'écart soldé (escompte, frais
    /// bancaires, perte sur débiteur). **Absent : préservé ; `null` : effacé**,
    /// patron du compte d'arrondi.
    #[serde(default, deserialize_with = "crate::helpers::double_option")]
    pub default_discount_account_id: Option<Option<i64>>,
    #[serde(default, deserialize_with = "crate::helpers::double_option")]
    pub default_bank_fees_account_id: Option<Option<i64>>,
    #[serde(default, deserialize_with = "crate::helpers::double_option")]
    pub default_bad_debt_account_id: Option<Option<i64>>,
    pub version: i32,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_journal(raw: &str) -> Result<Journal, AppError> {
    raw.parse::<Journal>()
        .map_err(|_| AppError::Validation(format!("Journal inconnu : '{raw}'")))
}

/// Valide un des six comptes **historiques** des réglages (créance, produit,
/// TVA due, TVA récupérable, décompte TVA, créanciers) : existence, société,
/// `active` et type, toujours ; postabilité **seulement si la valeur change**
/// par rapport à `current` (Story 15-5b, AC10, AC11 — exemption « inchangé »).
async fn validate_account(
    state: &AppState,
    company_id: i64,
    account_id: Option<i64>,
    current: Option<i64>,
    expected: AccountType,
    field_label: &str,
) -> Result<(), AppError> {
    validate_account_of(
        state,
        company_id,
        account_id,
        &[expected],
        account_id != current,
        field_label,
    )
    .await
}

/// Forme générale de [`validate_account`] (Story 25-4-c3-a1) : plusieurs types
/// acceptés, et la postabilité exigée ou non.
///
/// ⚠️ `require_postable` est exigé pour un compte que des **écritures
/// automatiques** utiliseront : elles passent `enforce_postable = false`
/// (`journal_entries::create_in_tx`), si bien que la garde doit tenir ici, au
/// moment où le compte est désigné. Tous les champs l'exigent désormais quand
/// la valeur **change** — les six champs historiques depuis la Story 15-5b
/// (#429) ; une valeur inchangée en est exemptée, pour qu'un compte devenu non
/// imputable après coup ne bloque pas tout enregistrement.
async fn validate_account_of(
    state: &AppState,
    company_id: i64,
    account_id: Option<i64>,
    accepted: &[AccountType],
    require_postable: bool,
    field_label: &str,
) -> Result<(), AppError> {
    let Some(id) = account_id else {
        return Ok(());
    };
    let account = accounts::find_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::Validation(format!("{field_label} : compte introuvable")))?;
    if account.company_id != company_id {
        return Err(AppError::Validation(format!(
            "{field_label} : compte introuvable"
        )));
    }
    if !account.active {
        return Err(AppError::Validation(format!(
            "{field_label} : compte archivé"
        )));
    }
    if !accepted.contains(&account.account_type) {
        let attendus: Vec<&str> = accepted.iter().map(|t| t.as_str()).collect();
        return Err(AppError::Validation(format!(
            "{field_label} : type de compte incompatible (attendu {})",
            attendus.join(" ou ")
        )));
    }
    if require_postable && !account.postable {
        return Err(AppError::Validation(format!(
            "{field_label} : compte non imputable (compte de regroupement, de résultat ou de clôture)"
        )));
    }
    Ok(())
}

/// Résout un compte **désigné** des réglages — compte d'arrondi (25-4-c3-a1) ou
/// d'une nature d'écart soldé (25-4-d1) : absent du corps, la valeur en place est
/// préservée ; présent, il est validé (charge ou produit, actif, imputable, de la
/// société).
///
/// ⚠️ Validé **seulement s'il change**. Le compte désigné peut devenir archivé
/// ou non imputable par une autre route (l'archivage n'est pas gardé, #486) ;
/// le revalider à chaque enregistrement bloquerait tout changement SANS
/// rapport — un format de numérotation — sur un champ que l'utilisateur n'a
/// pas touché. L'écriture qui lit le réglage refusera d'écrire sur un compte
/// devenu invalide : c'est là que la garde doit tenir, pas ici.
async fn resolve_designated_account(
    state: &AppState,
    company_id: i64,
    requested: Option<Option<i64>>,
    current: Option<i64>,
    field_label: &str,
) -> Result<Option<i64>, AppError> {
    let resolved = requested.unwrap_or(current);
    if resolved != current {
        validate_account_of(
            state,
            company_id,
            resolved,
            &[AccountType::Expense, AccountType::Revenue],
            true,
            field_label,
        )
        .await?;
    }
    Ok(resolved)
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `GET /api/v1/company/invoice-settings` — lecture config (tout rôle auth).
pub async fn get_invoice_settings(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<InvoiceSettingsResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let settings = company_invoice_settings::get_or_create_default(&state.pool, company.id).await?;
    Ok(Json(settings.into()))
}

/// `PUT /api/v1/company/invoice-settings` — mise à jour config (Admin).
///
/// ⚠️ **Rejouée sur interblocage** (Story 15-5e1, choix C70) par l'enveloppe
/// `DbError` [`kesh_db::retry::retry_on_deadlock`] ; la `version` du corps est
/// rejugée à chaque tentative par le verrou optimiste de `update`.
pub async fn update_invoice_settings(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<UpdateInvoiceSettingsRequest>,
) -> Result<Json<InvoiceSettingsResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;

    // #216 — `credit_note_number_format` optionnel : si le client ne l'envoie pas
    // (cas du formulaire frontend actuel), on **préserve** la valeur existante au
    // lieu de rejeter la requête en 422.
    let current = company_invoice_settings::get_or_create_default(&state.pool, company.id).await?;
    let credit_note_number_format = req
        .credit_note_number_format
        .clone()
        .unwrap_or(current.credit_note_number_format);

    // 1. Valider le format (facture + avoir — même grammaire {YEAR}/{FY}/{SEQ:NN}).
    invoice_format::validate_template(&req.invoice_number_format)
        .map_err(|e| AppError::Validation(e.to_string()))?;
    invoice_format::validate_template(&credit_note_number_format)
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // 2. Valider le template de description.
    invoice_format::validate_description_template(&req.journal_entry_description_template)
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // 3. Valider le journal (whitelist via FromStr).
    let journal = parse_journal(&req.default_sales_journal)?;

    // 4. Valider les comptes (existence, scope company, type, actif ; imputable
    //    si la valeur change — Story 15-5b, AC10).
    validate_account(
        &state,
        company.id,
        req.default_receivable_account_id,
        current.default_receivable_account_id,
        AccountType::Asset,
        "Compte créance",
    )
    .await?;
    validate_account(
        &state,
        company.id,
        req.default_revenue_account_id,
        current.default_revenue_account_id,
        AccountType::Revenue,
        "Compte produit",
    )
    .await?;
    // Comptes TVA (Story 18-1a) : TVA due + décompte = Liability, récupérable = Asset.
    validate_account(
        &state,
        company.id,
        req.default_vat_payable_account_id,
        current.default_vat_payable_account_id,
        AccountType::Liability,
        "Compte TVA due",
    )
    .await?;
    validate_account(
        &state,
        company.id,
        req.default_vat_recoverable_account_id,
        current.default_vat_recoverable_account_id,
        AccountType::Asset,
        "Compte TVA récupérable",
    )
    .await?;
    validate_account(
        &state,
        company.id,
        req.default_vat_decompte_account_id,
        current.default_vat_decompte_account_id,
        AccountType::Liability,
        "Compte décompte TVA",
    )
    .await?;
    // Compte créanciers (Story 12.2) : contrepartie achat fournisseur = Liability.
    // Story 15-5b (AC19, #521) : absent du corps → valeur en place préservée,
    // **sans contrôle** (on ne refuse pas un enregistrement pour un champ que
    // le client n'a pas envoyé) ; `null` → effacé ; valeur → validée, et sur la
    // postabilité si elle change (AC10, AC11).
    let default_payable_account_id = match req.default_payable_account_id {
        None => current.default_payable_account_id,
        Some(requested) => {
            validate_account(
                &state,
                company.id,
                requested,
                current.default_payable_account_id,
                AccountType::Liability,
                "Compte créanciers",
            )
            .await?;
            requested
        }
    };

    // Compte de différences d'arrondi (Story 25-4-c3-a1) : un écart d'arrondi est
    // un résultat, dans un sens ou dans l'autre — charge ou produit, imputable.
    let default_rounding_account_id = resolve_designated_account(
        &state,
        company.id,
        req.default_rounding_account_id,
        current.default_rounding_account_id,
        "Compte de différences d'arrondi",
    )
    .await?;
    // Story 25-4-d1 (#384) — les comptes des natures d'écart soldé.
    let default_discount_account_id = resolve_designated_account(
        &state,
        company.id,
        req.default_discount_account_id,
        current.default_discount_account_id,
        "Compte d'escompte",
    )
    .await?;
    let default_bank_fees_account_id = resolve_designated_account(
        &state,
        company.id,
        req.default_bank_fees_account_id,
        current.default_bank_fees_account_id,
        "Compte de frais bancaires",
    )
    .await?;
    let default_bad_debt_account_id = resolve_designated_account(
        &state,
        company.id,
        req.default_bad_debt_account_id,
        current.default_bad_debt_account_id,
        "Compte de pertes sur créances",
    )
    .await?;

    // Story 25-4-e (#495) — le montant minimum : strictement positif, au centime.
    let minimum_invoice_amount = req
        .minimum_invoice_amount
        .unwrap_or(current.minimum_invoice_amount);
    // Plafond : celui d'un prix unitaire (patron des frais de rappel, revue de code P1).
    if let Some(min) = minimum_invoice_amount
        && (min <= rust_decimal::Decimal::ZERO
            || min > *crate::routes::limits::MAX_UNIT_PRICE
            || !crate::routes::limits::scale_within(&min.normalize(), 2))
    {
        return Err(AppError::Validation(
            "Le montant minimum d'une facture doit être positif, au centime.".into(),
        ));
    }

    // 5. Persister.
    let update = CompanyInvoiceSettingsUpdate {
        invoice_number_format: req.invoice_number_format,
        default_receivable_account_id: req.default_receivable_account_id,
        default_revenue_account_id: req.default_revenue_account_id,
        default_vat_payable_account_id: req.default_vat_payable_account_id,
        default_vat_recoverable_account_id: req.default_vat_recoverable_account_id,
        default_vat_decompte_account_id: req.default_vat_decompte_account_id,
        default_sales_journal: journal,
        journal_entry_description_template: req.journal_entry_description_template,
        credit_note_number_format,
        default_payable_account_id,
        default_rounding_account_id,
        // Changer le réglage ne touche aucune pièce émise : leur arrondi est figé.
        round_to_5_centimes: req
            .round_to_5_centimes
            .unwrap_or(current.round_to_5_centimes),
        minimum_invoice_amount,
        default_discount_account_id,
        default_bank_fees_account_id,
        default_bad_debt_account_id,
    };
    let settings = kesh_db::retry::retry_on_deadlock("company_invoice_settings::update", || {
        company_invoice_settings::update(
            &state.pool,
            company.id,
            req.version,
            current_user.user_id,
            update.clone(),
        )
    })
    .await?;

    Ok(Json(settings.into()))
}

// ---------------------------------------------------------------------------
// Tests unitaires (validation)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_journal_ok() {
        assert_eq!(parse_journal("Ventes").unwrap(), Journal::Ventes);
        assert_eq!(parse_journal("OD").unwrap(), Journal::OD);
    }

    #[test]
    fn parse_journal_unknown_rejected() {
        assert!(parse_journal("Sales").is_err());
        assert!(parse_journal("ventes").is_err()); // casse matters (BINARY)
    }
}
