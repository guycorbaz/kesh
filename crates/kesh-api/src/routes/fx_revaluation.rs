//! Réévaluation de fin d'exercice des comptes en devise (Story 37-5).
//!
//! - `POST /api/v1/fiscal-years/{id}/fx-revaluation/preview` — l'**aperçu** :
//!   par compte de bilan en devise, les soldes cumulés à la date de fin, le
//!   cours, le CHF réévalué, l'écart et l'écriture prévue ; les réévaluations
//!   déjà passées ; les cours par devise ; ce que la passation refuserait
//!   (`blockers[]`). N'écrit rien.
//! - `POST /api/v1/fiscal-years/{id}/fx-revaluation` — la **passation** : une
//!   écriture par compte réévalué, dans une transaction
//!   (`kesh_db::repositories::fx_revaluation::post`, rejouée sur interblocage).
//!
//! Les deux routes sont dans `comptable_routes` (Consultation : 403, aperçu
//! compris), ouvertes aux clés d'API (pas d'`ensure_not_pat`, comme la
//! clôture). Le serveur ne récupère jamais de cours : les cours par défaut se
//! lisent dans la table des cours retenus par la société, **avant** toute
//! transaction (`fx_revaluation::resolve_default_rates`). Aucun outil MCP ne
//! porte l'aperçu (un calcul à la demande, hors du périmètre en lecture des
//! livres du serveur MCP) ; les écritures passées se lisent par
//! `kesh_ecritures` et `kesh_ecriture`.
//!
//! # Ordre des refus avant la transaction (AC-4)
//!
//! Exercice inconnu ou d'une autre société (404) ; exercice mono-devise (409
//! `MULTI_CURRENCY_DISABLED`, lecture simple : le mode ne change plus) ;
//! exercice clos (400 `FISCAL_YEAR_CLOSED`) ; forme de la requête (400
//! `FX_RATE_INVALID`, `FX_REVALUATION_ACCOUNT_INVALID`) ; à la passation,
//! cours par défaut manquant (404 `RATE_UNAVAILABLE`). Le reste se juge sous
//! verrou, dans le dépôt.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::str::FromStr;

use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::{Extension, Json};
use chrono::NaiveDate;
use kesh_core::accounting::fx_revaluation::{
    RevaluationRateKind, RevaluationSide, chf_cents as cents, received_rate_kind, revaluation_side,
    revaluation_totals,
};
use kesh_core::exchange_rates::{RateKind, check_rate_bounds, first_of_month};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use kesh_db::entities::audit_log::Acteur;
use kesh_db::entities::{AccountType, FiscalYear, FiscalYearStatus, FxAccountsOrigin};
use kesh_db::errors::{DbError, FxAccountField, FxAccountReason, map_db_error};
use kesh_db::repositories::fx_revaluation::{
    self as depot, DefaultRate, LineOutcome, MAX_ACCOUNTS, MAX_CURRENCIES, PostRequest,
    RetainedRate, RevaluationAccountReason, RevaluationLine,
};
use kesh_db::repositories::{accounts, companies, fiscal_years};

use crate::AppState;
use crate::errors::AppError;
use crate::extractors::impl_validation_extractor;
use crate::middleware::auth::CurrentUser;

// ---------------------------------------------------------------------------
// Requête
// ---------------------------------------------------------------------------

/// Corps des deux routes : cours par devise (chaînes décimales, par unité) et
/// comptes sélectionnés.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevaluationBody {
    /// Cours surchargés, par devise. Absent : les cours par défaut.
    #[serde(default)]
    pub rates: Option<BTreeMap<String, String>>,
    /// Comptes sélectionnés. Absent à l'aperçu : la présélection ; à la
    /// passation : refusé (`reason: "empty"`).
    #[serde(default)]
    pub account_ids: Option<Vec<i64>>,
}

impl_validation_extractor!(
    RevaluationBodyExtractor,
    RevaluationBody,
    "`rates` objet { devise: \"cours\" } et `accountIds` tableau d'entiers, tous deux facultatifs"
);

/// L'exercice de la réévaluation, avec les refus d'avant la transaction qui ne
/// dépendent pas du corps : 404, mono-devise (409), clos (400).
async fn exercice_reevaluable(
    state: &AppState,
    company_id: i64,
    id: i64,
) -> Result<FiscalYear, AppError> {
    let fy = fiscal_years::find_by_id_in_company(&state.pool, company_id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;
    if !fy.multi_currency {
        return Err(AppError::Database(DbError::MultiCurrencyDisabled {
            fiscal_year_id: Some(fy.id),
        }));
    }
    if fy.status == FiscalYearStatus::Closed {
        return Err(AppError::Database(DbError::FiscalYearClosed));
    }
    Ok(fy)
}

/// Les cours reçus, analysés et bornés (AC-4) : plus de 100 devises → 400
/// `FX_RATE_INVALID` `{reason: "tooMany"}` ; un cours illisible, nul ou
/// négatif, à plus de 10 décimales ou au-delà de 10⁶ → 400 `FX_RATE_INVALID`
/// `{currency}` (la fonction des bornes de 37-2, DRY).
fn cours_recus(
    rates: Option<&BTreeMap<String, String>>,
) -> Result<BTreeMap<String, Decimal>, AppError> {
    let Some(rates) = rates else {
        return Ok(BTreeMap::new());
    };
    if rates.len() > MAX_CURRENCIES {
        return Err(AppError::FxRateTooMany);
    }
    let mut out = BTreeMap::new();
    for (currency, texte) in rates {
        let invalide = || AppError::FxRateInvalid {
            currency: currency.clone(),
        };
        let lisible = !texte.is_empty()
            && texte
                .bytes()
                .all(|c| c.is_ascii_digit() || c == b'.' || c == b'-');
        let rate = if lisible {
            Decimal::from_str(texte).map_err(|_| invalide())?
        } else {
            return Err(invalide());
        };
        check_rate_bounds(&rate).map_err(|_| invalide())?;
        out.insert(currency.clone(), rate);
    }
    Ok(out)
}

/// Chaque clé de `rates` est la devise d'au moins un compte de `scope` ; sinon
/// 400 `FX_RATE_INVALID` `{currency}`.
fn cles_dans(rates: &BTreeMap<String, Decimal>, scope: &BTreeSet<String>) -> Result<(), AppError> {
    match rates.keys().find(|c| !scope.contains(*c)) {
        Some(c) => Err(AppError::FxRateInvalid {
            currency: c.clone(),
        }),
        None => Ok(()),
    }
}

/// Le cours retenu d'une devise : reçu, ou par défaut ; son genre par la règle
/// unique de l'AC-1 ; sa source pour l'audit (celle du défaut quand le genre
/// n'est pas `request`, `manual` sinon). `None` sans cours.
fn cours_retenu(received: Option<Decimal>, default: Option<&DefaultRate>) -> Option<RetainedRate> {
    let defaut = default.map(|d| (d.rate, d.kind));
    let (rate, kind) = match (received, default) {
        (Some(r), _) => (r, received_rate_kind(&r, defaut)),
        (None, Some(d)) => (d.rate, RevaluationRateKind::from(d.kind)),
        (None, None) => return None,
    };
    let source = match (kind, default) {
        (RevaluationRateKind::Request, _) | (_, None) => "manual".to_string(),
        (_, Some(d)) => d.source.as_str().to_string(),
    };
    Some(RetainedRate { rate, kind, source })
}

// ---------------------------------------------------------------------------
// Passation
// ---------------------------------------------------------------------------

/// Réponse 200 de la passation : les écritures passées, dans l'ordre des
/// numéros de compte (vide quand aucun écart n'est passé).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevaluationPostResponse {
    /// Identifiants des écritures.
    pub entry_ids: Vec<i64>,
}

/// `POST /api/v1/fiscal-years/{id}/fx-revaluation` — passe la réévaluation
/// (AC-2) : le serveur recalcule, sans se fier à l'aperçu ; cours par défaut
/// résolus avant la transaction ; la passation dans
/// `retry_on_deadlock("fx_revaluation::post", …)`.
pub async fn post_revaluation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    RevaluationBodyExtractor(body): RevaluationBodyExtractor,
) -> Result<Json<RevaluationPostResponse>, AppError> {
    let company_id = current_user.company_id;
    let fy = exercice_reevaluable(&state, company_id, id).await?;
    let recus = cours_recus(body.rates.as_ref())?;
    let account_ids = body.account_ids.unwrap_or_default();
    depot::check_account_ids_shape(&account_ids)?;

    // Lectures ordinaires, avant la transaction : devises des comptes
    // sélectionnés, puis cours par défaut.
    let mut conn = state.pool.acquire().await.map_err(map_db_error)?;
    let selected = depot::find_accounts(&mut conn, company_id, &account_ids).await?;
    let currencies: BTreeSet<String> = selected.iter().filter_map(|a| a.currency.clone()).collect();
    cles_dans(&recus, &currencies)?;
    let defaults = depot::resolve_default_rates(&mut conn, company_id, &fy, &currencies).await?;
    drop(conn);

    let mut rates = BTreeMap::new();
    for currency in &currencies {
        let default = defaults.get(currency).and_then(Option::as_ref);
        let Some(retained) = cours_retenu(recus.get(currency).copied(), default) else {
            return Err(AppError::Database(DbError::RateUnavailable {
                currency: currency.clone(),
                month: first_of_month(fy.end_date),
            }));
        };
        rates.insert(currency.clone(), retained);
    }

    let request = PostRequest {
        fiscal_year_id: fy.id,
        closing_rate_kind: fy.closing_rate_kind,
        rates,
        account_ids,
    };
    let acteur = Acteur {
        user_id: current_user.user_id,
        api_key_id: current_user.api_key_id,
    };
    let posted = kesh_db::retry::retry_on_deadlock("fx_revaluation::post", || {
        depot::post(&state.pool, acteur, company_id, &request)
    })
    .await?;
    Ok(Json(RevaluationPostResponse {
        entry_ids: posted.into_iter().map(|p| p.entry_id).collect(),
    }))
}

// ---------------------------------------------------------------------------
// Aperçu
// ---------------------------------------------------------------------------

/// Le cours d'une devise dans l'aperçu (AC-1, U1).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewRate {
    /// Code ISO 4217.
    pub currency: String,
    /// Le cours employé (reçu ou par défaut) ; `null` sans cours.
    pub rate: Option<Decimal>,
    /// `day`, `month` ou `request` ; `null` sans cours.
    pub rate_kind: Option<RevaluationRateKind>,
    /// Date du cours par défaut (jour, ou 1er du mois) ; `null` sans défaut.
    pub rate_date: Option<NaiveDate>,
    /// Le cours retenu par la société de la résolution par défaut.
    pub default_rate: Option<Decimal>,
    /// Sa source, valeur brute (`afc`, `manual` ou autre).
    pub default_source: Option<String>,
    /// Vrai si et seulement si le genre est `request`.
    pub overridden: bool,
    /// Date de publication d'un cours journalier `afc` ; `null` sinon.
    pub source_date: Option<NaiveDate>,
}

/// Une ligne de l'aperçu : un compte et l'écriture prévue (R9, A39).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewLine {
    /// Le compte réévalué.
    pub account_id: i64,
    /// Son numéro.
    pub account_number: String,
    /// Son nom.
    pub account_name: String,
    /// `Asset` ou `Liability` — pour le signe naturel affiché.
    pub account_type: AccountType,
    /// Devise du compte (T3 : la ligne n'en porte pas).
    pub currency: String,
    /// Compte archivé (non sélectionnable).
    pub archived: bool,
    /// Le compte peut être coché.
    pub selectable: bool,
    /// Le compte est dans la sélection de cette réponse.
    pub selected: bool,
    /// `ACCOUNT_ARCHIVED`, `RATE_UNAVAILABLE` ou `null`.
    pub reason: Option<&'static str>,
    /// Débit net en devise, depuis l'origine.
    pub devise_net: Decimal,
    /// Débit net en CHF (« CHF actuel »).
    pub chf_net: Decimal,
    /// Le cours de la devise ; `null` sans cours.
    pub rate: Option<Decimal>,
    /// CHF réévalué (« CHF cible ») ; `null` sans cours.
    pub target_chf: Option<Decimal>,
    /// Écart `D` en débit net ; `null` sans cours.
    pub delta: Option<Decimal>,
    /// Compte de contrepartie de l'écriture prévue (gains si `D > 0`, pertes
    /// si `D < 0`, gains à `D = 0`) ; `null` sans comptes de différence.
    pub counterpart_account_id: Option<i64>,
    /// Son numéro.
    pub counterpart_account_number: Option<String>,
    /// Son nom.
    pub counterpart_account_name: Option<String>,
    /// `own` (comptes propres du compte) ou `company` ; `null` sans comptes.
    pub fx_accounts_origin: Option<FxAccountsOrigin>,
    /// Libellé de l'écriture prévue ; `null` à écart nul ou sans cours.
    pub label: Option<String>,
    /// Σ des réévaluations déjà passées sur le compte dans l'exercice, en CHF.
    pub previous_revaluation_total: Decimal,
    /// Leurs écritures, par date puis `id` (au plus 20).
    pub previous_entry_ids: Vec<i64>,
    /// Valeur cumulée après passage : `previousRevaluationTotal + D`.
    pub revaluation_total_after: Option<Decimal>,
}

/// Ce que la passation refuserait, sans refus à l'aperçu.
#[derive(Debug, Serialize)]
pub struct PreviewBlocker {
    /// Le code d'erreur de la passation.
    pub code: String,
    /// Le message du serveur, traduit.
    pub message: String,
    /// Les détails du refus.
    pub details: serde_json::Value,
}

/// Totaux affichés.
#[derive(Debug, Serialize)]
pub struct PreviewTotals {
    /// Σ des écarts positifs des comptes sélectionnés.
    pub gains: Decimal,
    /// Σ des |écarts négatifs| des comptes sélectionnés.
    pub losses: Decimal,
}

/// Réponse 200 de l'aperçu.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevaluationPreviewResponse {
    /// L'exercice.
    pub fiscal_year_id: i64,
    /// Son premier jour.
    pub start_date: NaiveDate,
    /// Son dernier jour (date des écritures).
    pub end_date: NaiveDate,
    /// Genre de clôture de l'exercice.
    pub closing_rate_kind: RateKind,
    /// Les cours, par devise.
    pub rates: Vec<PreviewRate>,
    /// Les comptes, par numéro.
    pub lines: Vec<PreviewLine>,
    /// Totaux des comptes sélectionnés.
    pub totals: PreviewTotals,
    /// Sans `accountIds`, plus de 498 comptes admissibles : seuls les 498
    /// premiers par numéro sont présélectionnés.
    pub preselection_truncated: bool,
    /// Ce que la passation refuserait.
    pub blockers: Vec<PreviewBlocker>,
}

/// Le corps d'erreur `{code, message, details}` qu'une erreur rendrait — pour
/// les `blockers[]`, avec le message traduit du refus (DRY).
async fn blocker_de(err: AppError) -> PreviewBlocker {
    let response = err.into_response();
    let bytes = axum::body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .unwrap_or_default();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_default();
    let e = &value["error"];
    PreviewBlocker {
        code: e["code"].as_str().unwrap_or_default().to_string(),
        message: e["message"].as_str().unwrap_or_default().to_string(),
        details: e["details"].clone(),
    }
}

/// `POST /api/v1/fiscal-years/{id}/fx-revaluation/preview` — l'aperçu (AC-1) :
/// lectures ordinaires sur le pool, aucune écriture.
pub async fn preview_revaluation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    RevaluationBodyExtractor(body): RevaluationBodyExtractor,
) -> Result<Json<RevaluationPreviewResponse>, AppError> {
    let company_id = current_user.company_id;
    let fy = exercice_reevaluable(&state, company_id, id).await?;
    let recus = cours_recus(body.rates.as_ref())?;
    if let Some(ids) = &body.account_ids {
        depot::check_account_ids_shape(ids)?;
    }

    let mut conn = state.pool.acquire().await.map_err(map_db_error)?;
    let currencies = depot::candidate_currencies(&mut conn, company_id).await?;
    let defaults = depot::resolve_default_rates(&mut conn, company_id, &fy, &currencies).await?;
    let mut effective: BTreeMap<String, Decimal> = defaults
        .iter()
        .filter_map(|(c, d)| d.as_ref().map(|d| (c.clone(), d.rate)))
        .collect();
    for (c, r) in &recus {
        effective.insert(c.clone(), *r);
    }
    let lines = depot::lines_for_fiscal_year(&mut conn, company_id, &fy, &effective).await?;

    // La sélection : `accountIds`, chacun admissible ; sinon la présélection.
    let eligible: Vec<&RevaluationLine> = lines.iter().filter(|l| l.account.active).collect();
    let (selected, truncated): (BTreeSet<i64>, bool) = match &body.account_ids {
        Some(ids) => {
            let admissibles: BTreeSet<i64> = eligible.iter().map(|l| l.account.id).collect();
            let refuses: Vec<i64> = ids
                .iter()
                .copied()
                .filter(|i| !admissibles.contains(i))
                .collect();
            if !refuses.is_empty() {
                return Err(AppError::Database(DbError::FxRevaluationAccountInvalid {
                    reason: RevaluationAccountReason::NotEligible,
                    account_ids: refuses,
                }));
            }
            (ids.iter().copied().collect(), false)
        }
        None => {
            let avec_cours: Vec<i64> = eligible
                .iter()
                .filter(|l| l.outcome != LineOutcome::RateUnavailable)
                .map(|l| l.account.id)
                .collect();
            (
                avec_cours.iter().take(MAX_ACCOUNTS).copied().collect(),
                avec_cours.len() > MAX_ACCOUNTS,
            )
        }
    };
    let scope: BTreeSet<String> = match &body.account_ids {
        Some(_) => lines
            .iter()
            .filter(|l| selected.contains(&l.account.id))
            .filter_map(|l| l.account.currency.clone())
            .collect(),
        None => eligible
            .iter()
            .filter_map(|l| l.account.currency.clone())
            .collect(),
    };
    cles_dans(&recus, &scope)?;
    if let Some(e) = lines
        .iter()
        .filter(|l| selected.contains(&l.account.id))
        .find_map(RevaluationLine::out_of_range)
    {
        return Err(AppError::Database(e));
    }

    // Les cours, par devise des comptes listés.
    let listed: BTreeSet<String> = lines
        .iter()
        .filter_map(|l| l.account.currency.clone())
        .collect();
    let rates: Vec<PreviewRate> = listed
        .iter()
        .map(|currency| {
            let default = defaults.get(currency).and_then(Option::as_ref);
            let retained = cours_retenu(recus.get(currency).copied(), default);
            PreviewRate {
                currency: currency.clone(),
                rate: retained.as_ref().map(|r| r.rate),
                rate_kind: retained.as_ref().map(|r| r.kind),
                rate_date: default.map(|d| d.rate_date),
                default_rate: default.map(|d| d.rate),
                default_source: default.map(|d| d.source.as_str().to_string()),
                overridden: retained
                    .as_ref()
                    .is_some_and(|r| r.kind == RevaluationRateKind::Request),
                source_date: default.and_then(|d| d.source_date),
            }
        })
        .collect();

    // Réévaluations déjà passées et comptes de contrepartie.
    let ids: Vec<i64> = lines.iter().map(|l| l.account.id).collect();
    let previous = depot::previous_revaluations(&mut conn, company_id, &fy, &ids).await?;
    let contreparties: Vec<i64> = lines
        .iter()
        .filter_map(|l| l.fx_accounts)
        .flat_map(|fx| [fx.gain, fx.loss])
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let comptes_cp: HashMap<i64, (String, String)> =
        depot::find_accounts(&mut conn, company_id, &contreparties)
            .await?
            .into_iter()
            .map(|a| (a.id, (a.number, a.name)))
            .collect();

    let mut out_lines = Vec::with_capacity(lines.len());
    for line in &lines {
        let delta = line.delta();
        let counterpart =
            line.fx_accounts
                .map(|fx| match delta.as_ref().and_then(revaluation_side) {
                    Some(RevaluationSide::Loss) => fx.loss,
                    _ => fx.gain,
                });
        let label = match (delta.as_ref().and_then(revaluation_side), line.rate) {
            (Some(side), Some(rate)) => Some(depot::revaluation_label(
                side,
                &line.account.name,
                line.account.currency.as_deref().unwrap_or_default(),
                &rate,
                fy.end_date,
            )),
            _ => None,
        };
        let prev = previous.get(&line.account.id).cloned().unwrap_or_default();
        let reason = if !line.account.active {
            Some("ACCOUNT_ARCHIVED")
        } else if line.outcome == LineOutcome::RateUnavailable {
            Some("RATE_UNAVAILABLE")
        } else {
            None
        };
        out_lines.push(PreviewLine {
            account_id: line.account.id,
            account_number: line.account.number.clone(),
            account_name: line.account.name.clone(),
            account_type: line.account.account_type,
            currency: line.account.currency.clone().unwrap_or_default(),
            archived: !line.account.active,
            selectable: reason.is_none(),
            selected: selected.contains(&line.account.id),
            reason,
            devise_net: line.devise_net,
            chf_net: line.chf_net,
            rate: line.rate,
            target_chf: line.target_chf(),
            delta,
            counterpart_account_id: counterpart,
            counterpart_account_number: counterpart
                .and_then(|c| comptes_cp.get(&c))
                .map(|c| c.0.clone()),
            counterpart_account_name: counterpart
                .and_then(|c| comptes_cp.get(&c))
                .map(|c| c.1.clone()),
            fx_accounts_origin: line.fx_accounts.map(|fx| fx.origin),
            label,
            previous_revaluation_total: cents(prev.total),
            previous_entry_ids: prev.entry_ids,
            revaluation_total_after: delta.map(|d| cents(prev.total + d)),
        });
    }
    let selected_deltas: Vec<Decimal> = out_lines
        .iter()
        .filter(|l| l.selected)
        .filter_map(|l| l.delta)
        .collect();
    let (gains, losses) = revaluation_totals(&selected_deltas);

    // Ce que la passation refuserait (sans refus ici).
    let mut blockers = Vec::new();
    if let Some(open) =
        fiscal_years::find_earliest_earlier_open(&mut *conn, company_id, fy.start_date).await?
    {
        blockers.push(
            blocker_de(AppError::Database(DbError::EarlierFiscalYearOpen {
                fiscal_year_id: open.id,
                fiscal_year_name: open.name,
            }))
            .await,
        );
    }
    if let Some(later) =
        fiscal_years::find_later_closed(&mut conn, company_id, fy.start_date).await?
    {
        blockers.push(
            blocker_de(AppError::Database(DbError::LaterFiscalYearClosed {
                fiscal_year_id: later.id,
                fiscal_year_name: later.name,
            }))
            .await,
        );
    }
    let company = companies::find_by_id(&state.pool, company_id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;
    if let Some(locked_through) = company.books_locked_through
        && fy.end_date <= locked_through
    {
        blockers.push(
            blocker_de(AppError::Database(DbError::PeriodLocked {
                locked_through,
                attempted: fy.end_date,
            }))
            .await,
        );
    }
    for line in lines.iter().filter(|l| selected.contains(&l.account.id)) {
        let verdict = match line.fx_accounts {
            None => Err(DbError::FxAccountInvalid {
                field: FxAccountField::Gain,
                reason: FxAccountReason::Missing,
                holder_account_id: Some(line.account.id),
            }),
            Some(fx) => {
                match accounts::validate_fx_accounts_in_tx(&mut conn, company_id, &fx).await {
                    Err(e @ DbError::FxAccountInvalid { .. }) => Err(e),
                    Err(e) => return Err(AppError::Database(e)),
                    Ok(()) => Ok(()),
                }
            }
        };
        if let Err(e) = verdict {
            blockers.push(blocker_de(AppError::Database(e)).await);
        }
    }

    Ok(Json(RevaluationPreviewResponse {
        fiscal_year_id: fy.id,
        start_date: fy.start_date,
        end_date: fy.end_date,
        closing_rate_kind: fy.closing_rate_kind,
        rates,
        lines: out_lines,
        totals: PreviewTotals { gains, losses },
        preselection_truncated: truncated,
        blockers,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use kesh_db::repositories::exchange_rates::RateOrigin;
    use rust_decimal_macros::dec;

    fn defaut(rate: Decimal, kind: RateKind, source: RateOrigin) -> DefaultRate {
        DefaultRate {
            rate,
            kind,
            rate_date: NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            source,
            source_date: None,
        }
    }

    #[test]
    fn source_et_genre_du_cours_retenu() {
        let afc = defaut(dec!(0.91193), RateKind::Day, RateOrigin::Afc);
        // Prérempli renvoyé tel quel : genre du défaut, source du défaut.
        let r = cours_retenu(Some(dec!(0.911930)), Some(&afc)).unwrap();
        assert_eq!(
            (r.kind, r.source.as_str()),
            (RevaluationRateKind::Day, "afc")
        );
        // Modifié : request, manual.
        let r = cours_retenu(Some(dec!(0.9)), Some(&afc)).unwrap();
        assert_eq!(
            (r.kind, r.source.as_str()),
            (RevaluationRateKind::Request, "manual")
        );
        // Absent : le défaut.
        let r = cours_retenu(None, Some(&afc)).unwrap();
        assert_eq!((r.rate, r.kind), (dec!(0.91193), RevaluationRateKind::Day));
        // Source brute d'une autre valeur (A30), gardée telle quelle.
        let ecb = defaut(dec!(0.95), RateKind::Month, RateOrigin::Other("ecb".into()));
        let r = cours_retenu(None, Some(&ecb)).unwrap();
        assert_eq!(
            (r.kind, r.source.as_str()),
            (RevaluationRateKind::Month, "ecb")
        );
        // Sans défaut : request, manual ; sans rien : aucun cours.
        let r = cours_retenu(Some(dec!(1)), None).unwrap();
        assert_eq!(
            (r.kind, r.source.as_str()),
            (RevaluationRateKind::Request, "manual")
        );
        assert!(cours_retenu(None, None).is_none());
    }

    #[test]
    fn cours_recus_bornes_et_trop_nombreux() {
        let un = |c: &str, v: &str| BTreeMap::from([(c.to_string(), v.to_string())]);
        for v in ["0", "-1", "0.12345678901", "1000000.5", "abc", "", "1e3"] {
            assert!(
                matches!(
                    cours_recus(Some(&un("USD", v))),
                    Err(AppError::FxRateInvalid { .. })
                ),
                "{v}"
            );
        }
        assert_eq!(
            cours_recus(Some(&un("USD", "0.9119300"))).unwrap()["USD"],
            dec!(0.91193)
        );
        let trop: BTreeMap<String, String> = (0..101)
            .map(|i| (format!("C{i:02}"), "1".to_string()))
            .collect();
        assert!(matches!(
            cours_recus(Some(&trop)),
            Err(AppError::FxRateTooMany)
        ));
    }
}
