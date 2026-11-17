//! Réévaluation de fin d'exercice des comptes de bilan en devise (Story 37-5).
//!
//! # Ce que fait ce module
//!
//! - [`resolve_default_rates`] — le **cours par défaut** de chaque devise, selon
//!   le genre de clôture de l'exercice (`day` : cours journalier retenu par la
//!   société pour la date de fin, à défaut le cours mensuel du mois de fin ;
//!   `month` : le cours mensuel seul), lu par
//!   [`exchange_rates::find_company_rate_in_tx`] — jamais la dernière valeur
//!   publiée non reprise (U1). **Une seule fonction**, appelée par l'aperçu et
//!   la passation (avant leur transaction), par le contrôle K9 (dans
//!   l'instantané) et par la clôture (avant `begin`).
//! - [`lines_for_fiscal_year`] — les **lignes de l'aperçu** : par compte de
//!   bilan en devise (`Asset`, `Liability`, imputable, archivés compris), les
//!   soldes cumulés depuis l'origine ([`balances_as_of`]) et l'écart au cours
//!   donné (calcul pur de `kesh-core`). **Une seule fonction** pour l'aperçu,
//!   K9 et la clôture ; la passation emploie le même calcul ([`build_lines`])
//!   sur les comptes qu'elle a lus sous verrou.
//! - [`post`] — la **passation** : une écriture `OD` de deux lignes par compte à
//!   écart non nul, toutes dans une transaction, plus l'audit
//!   `fx_revaluation.posted`.
//! - [`revaluation_label`] — le libellé, partagé par l'aperçu et la passation.
//! - [`previous_revaluations`] — les réévaluations déjà passées dans
//!   l'exercice (aperçu seulement).
//!
//! # Verrous et instantané de la passation
//!
//! Sous `REPEATABLE READ`, la vue cohérente naît à la **première lecture non
//! verrouillante** de la transaction : les soldes se lisent donc **après** tous
//! les verrous, et aucune lecture ordinaire ne les précède — sinon la seconde de
//! deux passations concurrentes relirait les soldes d'avant le `COMMIT` de la
//! première et passerait l'écart une seconde fois. La date de début et la liste
//! des antérieurs se lisent **avant** `begin`, sur le pool. Dans la
//! transaction :
//!
//! 0. la ligne `companies` `LOCK IN SHARE MODE` (comptes de différence de la
//!    société), **avant tout exercice** — l'ordre global `companies` → exercice ;
//! 1. chaque antérieur par clé primaire, puis l'exercice `FOR UPDATE`, puis
//!    2. la relecture verrouillante du plus ancien antérieur ouvert
//!    ([`fiscal_years::lock_with_earlier_in_tx`], partagée avec la clôture) ;
//! 3. les comptes sélectionnés `LOCK IN SHARE MODE`, puis, pour chacun, ses
//!    comptes de différence résolus ([`accounts::fx_accounts_for_columns`]) et
//!    revalidés ([`accounts::validate_fx_accounts_in_tx`], réglages, comptes et
//!    articles en partage) ;
//! 4. seulement ensuite, les soldes.
//!
//! Tout se lit d'abord, puis s'évalue dans l'ordre des refus de l'AC-4 ; un
//! cours ne se lit jamais sous ces verrous (résolus par l'appelant).
//!
//! # Ce que le module ne fait pas
//!
//! Il ne récupère aucun cours (aucun appel réseau) ; il ne passe aucune écriture
//! de report (le report à nouveau se fait par le cumul, `kesh-report`) ; il ne
//! modifie jamais une écriture : une nouvelle passation **ajoute** le résidu.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use chrono::NaiveDate;
use kesh_core::accounting::fx::{ForeignAmount, RateProvenance};
use kesh_core::accounting::fx_revaluation::{
    RevaluationDelta, RevaluationError, RevaluationRateKind, RevaluationSide, revaluation_delta,
    revaluation_side,
};
use kesh_core::exchange_rates::{RateKind, first_of_month};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::MySqlConnection;
use sqlx::mysql::MySqlPool;

use crate::entities::audit_log::{Acteur, NewAuditLogEntry};
use crate::entities::{
    AccountType, FiscalYear, FiscalYearStatus, FxAccounts, FxAccountsOrigin, Journal,
    NewJournalEntry, NewJournalEntryLine,
};
use crate::errors::{DbError, FxAccountField, FxAccountReason, map_db_error};
use crate::repositories::account_balances::balances_as_of;
use crate::repositories::exchange_rates::{
    RateOrigin, SOURCE_AFC, find_company_rate_in_tx, find_fetched_rate_in_tx,
};
use crate::repositories::{accounts, audit_log, closing_checks, fiscal_years, journal_entries};

/// Au plus tant de comptes par passation (taille de la transaction, de
/// l'aperçu et du détail d'audit) ; l'aperçu présélectionne au plus autant.
pub const MAX_ACCOUNTS: usize = 498;

/// Au plus tant de devises dans `rates`.
pub const MAX_CURRENCIES: usize = 100;

/// Au plus tant d'écritures déjà passées listées par compte (la somme porte sur
/// toutes) — comme `MAX_LISTED` des contrôles de clôture.
pub const MAX_PREVIOUS_LISTED: usize = 20;

/// Au plus tant de comptes listés dans les détails de K9 et du refus de la
/// clôture (le compte, lui, n'est pas borné).
pub const MAX_LISTED: usize = 20;

/// Longueur maximale d'un libellé d'écriture (colonne `description
/// VARCHAR(500)`, `MAX_DESCRIPTION_LEN` de la saisie manuelle).
pub const MAX_LABEL_CHARS: usize = 500;

/// Le prédicat des comptes **candidats** (alias `a`) : de bilan, en devise —
/// archivés et non imputables compris. Un compte archivé, ou devenu non
/// imputable (sous-compte créé, `postable = false`), **à solde non nul** est
/// listé, non sélectionnable, et K9 le compte : il ne se réévalue pas, mais il
/// ne contourne pas la clôture (37-5-CR-P1-D1). Sans solde, il n'est pas rendu
/// ([`lines_for_fiscal_year`]). Partagé, à l'octet près, avec la requête
/// `EXISTS` de K9 (`closing_checks::FX_CANDIDATES_EXIST_SQL`, gardé par un
/// test) : « même prédicat que l'aperçu ».
pub const CANDIDATE_PREDICATE: &str =
    "a.account_type IN ('Asset', 'Liability') AND a.currency IS NOT NULL";

// ---------------------------------------------------------------------------
// Cours par défaut
// ---------------------------------------------------------------------------

/// Le cours par défaut d'une devise pour la réévaluation : le cours **retenu
/// par la société** (une ligne `exchange_rates`), son genre, sa date et sa
/// source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefaultRate {
    /// Le cours, par unité.
    pub rate: Decimal,
    /// `day` (date de fin) ou `month` (dernier mois, choisi ou repli).
    pub kind: RateKind,
    /// La date du cours : la date de fin (`day`) ou le 1er de son mois
    /// (`month`).
    pub rate_date: NaiveDate,
    /// La source de la ligne, valeur brute (`afc`, `manual` ou autre, A30).
    pub source: RateOrigin,
    /// Date de publication d'un cours journalier `afc`, lue sur la valeur
    /// publiée (affichage seulement) ; `None` hors `day`, hors `afc`, ou si la
    /// valeur publiée n'est pas connue.
    pub source_date: Option<NaiveDate>,
}

/// Le cours par défaut de chaque devise de `currencies`, selon le genre de
/// clôture de `fy` (AC-1) — `None` pour une devise sans cours retenu
/// (`RATE_UNAVAILABLE` chez l'appelant) :
///
/// - genre `day` : le cours journalier de la date de fin ; à défaut, le cours
///   mensuel du mois de cette date (repli, que l'écran signale) ;
/// - genre `month` : le cours mensuel du mois de la date de fin, seul.
///
/// Lectures **non verrouillantes** sur `conn`, sans appel sortant : l'aperçu et
/// la passation l'appellent sur le pool avant leur transaction, K9 dans
/// l'instantané, la clôture avant `begin`. Un exercice de durée quelconque n'a
/// rien de particulier : « date de fin » et « dernier mois » sont ceux de
/// `end_date` (S4).
pub async fn resolve_default_rates(
    conn: &mut MySqlConnection,
    company_id: i64,
    fy: &FiscalYear,
    currencies: &BTreeSet<String>,
) -> Result<BTreeMap<String, Option<DefaultRate>>, DbError> {
    let end = fy.end_date;
    let month = first_of_month(end);
    let mut out = BTreeMap::new();
    for currency in currencies {
        let mut found = None;
        if fy.closing_rate_kind == RateKind::Day {
            found = find_company_rate_in_tx(&mut *conn, company_id, currency, RateKind::Day, end)
                .await?;
        }
        if found.is_none() {
            found =
                find_company_rate_in_tx(&mut *conn, company_id, currency, RateKind::Month, month)
                    .await?;
        }
        let resolved = match found {
            None => None,
            Some(row) => {
                let source_date = if row.kind == RateKind::Day && row.source.is_afc() {
                    find_fetched_rate_in_tx(&mut *conn, SOURCE_AFC, currency, RateKind::Day, end)
                        .await?
                        .and_then(|p| p.source_date)
                } else {
                    None
                };
                Some(DefaultRate {
                    rate: row.rate,
                    kind: row.kind,
                    rate_date: row.rate_date,
                    source: row.source,
                    source_date,
                })
            }
        };
        out.insert(currency.clone(), resolved);
    }
    Ok(out)
}

/// Les devises des comptes **candidats** de la société, lecture ordinaire.
pub async fn candidate_currencies(
    conn: &mut MySqlConnection,
    company_id: i64,
) -> Result<BTreeSet<String>, DbError> {
    let sql = format!(
        "SELECT DISTINCT a.currency FROM accounts a WHERE a.company_id = ? AND {CANDIDATE_PREDICATE}"
    );
    let rows: Vec<String> = sqlx::query_scalar(&sql)
        .bind(company_id)
        .fetch_all(&mut *conn)
        .await
        .map_err(map_db_error)?;
    Ok(rows.into_iter().collect())
}

/// Les cours par défaut de K9 et de la clôture (NFR-37-5-7) : la requête
/// `EXISTS` des comptes candidats ; aucun compte → aucune autre lecture ;
/// sinon les devises de ces comptes et leurs cours par défaut, la valeur de
/// chaque cours trouvé (une devise sans cours est absente de la table rendue).
pub async fn default_rates_for_candidates(
    conn: &mut MySqlConnection,
    company_id: i64,
    fy: &FiscalYear,
) -> Result<BTreeMap<String, Decimal>, DbError> {
    if !closing_checks::fx_candidates_exist(&mut *conn, company_id).await? {
        return Ok(BTreeMap::new());
    }
    let currencies = candidate_currencies(&mut *conn, company_id).await?;
    let resolved = resolve_default_rates(&mut *conn, company_id, fy, &currencies).await?;
    Ok(resolved
        .into_iter()
        .filter_map(|(c, r)| r.map(|r| (c, r.rate)))
        .collect())
}

// ---------------------------------------------------------------------------
// Lignes de l'aperçu
// ---------------------------------------------------------------------------

/// Un compte candidat ou sélectionné, tel que la réévaluation le lit.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct RevaluationAccount {
    /// Identifiant.
    pub id: i64,
    /// Numéro.
    pub number: String,
    /// Nom.
    pub name: String,
    /// Type.
    pub account_type: AccountType,
    /// Devise (`None` = CHF).
    pub currency: Option<String>,
    /// Actif (non archivé).
    pub active: bool,
    /// Imputable.
    pub postable: bool,
    /// Compte de gains propre.
    pub fx_gain_account_id: Option<i64>,
    /// Compte de pertes propre.
    pub fx_loss_account_id: Option<i64>,
}

impl RevaluationAccount {
    /// Admissible à la passation : de bilan, en devise, imputable, actif.
    pub fn is_eligible(&self) -> bool {
        self.is_candidate() && self.active && self.postable
    }

    /// Candidat (prédicat [`CANDIDATE_PREDICATE`]) : de bilan, en devise,
    /// imputable ou non, archivé ou non.
    pub fn is_candidate(&self) -> bool {
        matches!(
            self.account_type,
            AccountType::Asset | AccountType::Liability
        ) && self.currency.is_some()
    }
}

/// Colonnes de [`RevaluationAccount`] (alias `a`).
const ACCOUNT_COLUMNS: &str = "a.id, a.number, a.name, a.account_type, a.currency, a.active, \
     a.postable, a.fx_gain_account_id, a.fx_loss_account_id";

/// Ce que le calcul rend pour une ligne.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineOutcome {
    /// L'écart au cours donné.
    Delta(RevaluationDelta),
    /// La devise n'a pas de cours (`RATE_UNAVAILABLE`).
    RateUnavailable,
    /// Le calcul sort des bornes (`FX_VALUE_OUT_OF_RANGE`).
    OutOfRange,
}

/// Une ligne de la réévaluation : un compte, ses soldes cumulés, l'écart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevaluationLine {
    /// Le compte.
    pub account: RevaluationAccount,
    /// Débit net en devise, depuis l'origine jusqu'à la date de fin.
    pub devise_net: Decimal,
    /// Débit net en CHF, sur les mêmes lignes.
    pub chf_net: Decimal,
    /// Le cours employé, `None` sans cours.
    pub rate: Option<Decimal>,
    /// Le résultat du calcul.
    pub outcome: LineOutcome,
    /// Comptes de différence résolus (`fx_accounts_for`) ; `None` si ni le
    /// compte ni la société n'en désignent.
    pub fx_accounts: Option<FxAccounts>,
}

impl RevaluationLine {
    /// L'écart `D`, `None` sans cours ou hors bornes.
    pub fn delta(&self) -> Option<Decimal> {
        match self.outcome {
            LineOutcome::Delta(d) => Some(d.delta),
            _ => None,
        }
    }

    /// Le CHF réévalué, `None` sans cours ou hors bornes.
    pub fn target_chf(&self) -> Option<Decimal> {
        match self.outcome {
            LineOutcome::Delta(d) => Some(d.target_chf),
            _ => None,
        }
    }

    /// Le compte porte un solde (en devise ou en CHF).
    pub fn has_balance(&self) -> bool {
        !self.devise_net.is_zero() || !self.chf_net.is_zero()
    }

    /// Le refus `FX_VALUE_OUT_OF_RANGE` de la ligne, si son calcul sort des
    /// bornes (aperçu et clôture le rendent tel quel).
    pub fn out_of_range(&self) -> Option<DbError> {
        (self.outcome == LineOutcome::OutOfRange).then(|| DbError::FxRevaluationValueOutOfRange {
            currency: self.account.currency.clone().unwrap_or_default(),
            account_id: self.account.id,
        })
    }
}

/// Les comptes de différence d'un compte, d'après les réglages de la société
/// (`company_gain`, `company_loss`) — [`accounts::fx_accounts_for_columns`].
fn fx_for(
    company_fx: (Option<i64>, Option<i64>),
    account: &RevaluationAccount,
) -> Option<FxAccounts> {
    accounts::fx_accounts_for_columns(
        company_fx.0,
        company_fx.1,
        account.id,
        account.currency.as_deref(),
        account.fx_gain_account_id,
        account.fx_loss_account_id,
    )
}

/// Le calcul des lignes, **sans I/O** : pour chaque compte (dans l'ordre reçu),
/// son solde (`balances`, ceux de [`balances_as_of`]) et l'écart au cours de sa
/// devise dans `rates`. Partagé par [`lines_for_fiscal_year`] (aperçu, K9,
/// clôture) et [`post`].
pub fn build_lines(
    accounts: &[RevaluationAccount],
    balances: &[crate::repositories::account_balances::SoldeALaDate],
    rates: &BTreeMap<String, Decimal>,
    company_fx: (Option<i64>, Option<i64>),
) -> Vec<RevaluationLine> {
    let by_id: HashMap<i64, (Decimal, Option<Decimal>)> = balances
        .iter()
        .map(|(id, chf, devise)| (*id, (*chf, *devise)))
        .collect();
    accounts
        .iter()
        .map(|account| {
            let (chf_net, devise_net) = by_id.get(&account.id).copied().unwrap_or_default();
            let devise_net = devise_net.unwrap_or_default();
            let rate = account
                .currency
                .as_ref()
                .and_then(|c| rates.get(c))
                .copied();
            let outcome = match rate {
                None => LineOutcome::RateUnavailable,
                Some(r) => match revaluation_delta(&devise_net, &chf_net, &r) {
                    Ok(d) => LineOutcome::Delta(d),
                    Err(RevaluationError::OutOfRange) => LineOutcome::OutOfRange,
                },
            };
            RevaluationLine {
                account: account.clone(),
                devise_net,
                chf_net,
                rate,
                outcome,
                fx_accounts: fx_for(company_fx, account),
            }
        })
        .collect()
}

/// Les comptes de différence de la société, lecture ordinaire.
async fn company_fx_accounts(
    conn: &mut MySqlConnection,
    company_id: i64,
) -> Result<(Option<i64>, Option<i64>), DbError> {
    sqlx::query_as::<_, (Option<i64>, Option<i64>)>(
        "SELECT fx_gain_account_id, fx_loss_account_id FROM companies WHERE id = ?",
    )
    .bind(company_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(map_db_error)?
    .ok_or(DbError::NotFound)
}

/// Les **lignes de l'aperçu** de l'exercice `fy` (AC-1, AC-9 *Une seule
/// fonction*) : tous les comptes candidats ([`CANDIDATE_PREDICATE`], archivés
/// compris) par numéro, leurs soldes cumulés à la date de fin, et l'écart au
/// cours de `rates` (devise absente : [`LineOutcome::RateUnavailable`]). Un
/// compte **archivé ou non imputable** sans solde (ni en devise ni en CHF)
/// n'est pas rendu.
///
/// Lectures **non verrouillantes** sur `conn` : l'aperçu les fait sur le pool,
/// K9 dans l'instantané, la clôture après ses verrous (la vue naît alors ici).
pub async fn lines_for_fiscal_year(
    conn: &mut MySqlConnection,
    company_id: i64,
    fy: &FiscalYear,
    rates: &BTreeMap<String, Decimal>,
) -> Result<Vec<RevaluationLine>, DbError> {
    let sql = format!(
        "SELECT {ACCOUNT_COLUMNS} FROM accounts a WHERE a.company_id = ? AND {CANDIDATE_PREDICATE} \
         ORDER BY a.number ASC, a.id ASC"
    );
    let candidates: Vec<RevaluationAccount> = sqlx::query_as(&sql)
        .bind(company_id)
        .fetch_all(&mut *conn)
        .await
        .map_err(map_db_error)?;
    if candidates.is_empty() {
        return Ok(Vec::new());
    }
    let company_fx = company_fx_accounts(&mut *conn, company_id).await?;
    let ids: Vec<i64> = candidates.iter().map(|a| a.id).collect();
    let balances = balances_as_of(&mut *conn, company_id, fy.end_date, Some(&ids)).await?;
    Ok(build_lines(&candidates, &balances, rates, company_fx)
        .into_iter()
        .filter(|l| l.account.is_eligible() || l.has_balance())
        .collect())
}

// ---------------------------------------------------------------------------
// K9 — comptes non réévalués
// ---------------------------------------------------------------------------

/// Motif d'un compte relevé par K9.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NotRevaluedReason {
    /// Écart non nul au cours par défaut (ou calcul hors bornes, `delta` nul).
    NotRevalued,
    /// Devise sans cours de clôture, compte à solde non nul.
    RateUnavailable,
    /// Compte actif, à solde, devenu non imputable : il ne se réévalue pas
    /// tant qu'il ne l'est plus (37-5-CR-P1-D1) ; l'utilisateur doit le
    /// rendre imputable. Prime sur les deux autres motifs.
    NotPostable,
}

/// Un compte relevé par K9.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotRevaluedAccount {
    /// Identifiant du compte.
    pub account_id: i64,
    /// Numéro.
    pub number: String,
    /// Nom.
    pub name: String,
    /// Devise.
    pub currency: String,
    /// L'écart `D` (chaîne décimale) ; `None` sans cours ou hors bornes.
    pub delta: Option<Decimal>,
    /// Motif.
    pub reason: NotRevaluedReason,
}

/// Les comptes relevés par K9 : `count` (jamais borné) et au plus
/// [`MAX_LISTED`] comptes, par numéro — `details` de K9 et du refus de la
/// clôture (même forme, même liste).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotRevaluedAccounts {
    /// Nombre de comptes concernés.
    #[serde(skip)]
    pub count: i64,
    /// Au plus [`MAX_LISTED`] comptes.
    pub accounts: Vec<NotRevaluedAccount>,
}

/// K9 sur les lignes de [`lines_for_fiscal_year`] aux cours par défaut :
/// toute ligne (archivés compris, sans plafond) à écart non nul, hors bornes,
/// ou sans cours avec un solde non nul.
pub fn not_revalued(lines: &[RevaluationLine]) -> NotRevaluedAccounts {
    let mut count = 0;
    let mut accounts = Vec::new();
    for line in lines {
        let (delta, reason) = match line.outcome {
            LineOutcome::Delta(d) if !d.delta.is_zero() => {
                (Some(d.delta), NotRevaluedReason::NotRevalued)
            }
            LineOutcome::Delta(_) => continue,
            LineOutcome::OutOfRange => (None, NotRevaluedReason::NotRevalued),
            LineOutcome::RateUnavailable if line.has_balance() => {
                (None, NotRevaluedReason::RateUnavailable)
            }
            LineOutcome::RateUnavailable => continue,
        };
        let reason = if line.account.active && !line.account.postable {
            NotRevaluedReason::NotPostable
        } else {
            reason
        };
        count += 1;
        if accounts.len() < MAX_LISTED {
            accounts.push(NotRevaluedAccount {
                account_id: line.account.id,
                number: line.account.number.clone(),
                name: line.account.name.clone(),
                currency: line.account.currency.clone().unwrap_or_default(),
                delta,
                reason,
            });
        }
    }
    NotRevaluedAccounts { count, accounts }
}

// ---------------------------------------------------------------------------
// Réévaluations déjà passées (A39) et libellé
// ---------------------------------------------------------------------------

/// Les réévaluations déjà passées sur un compte dans l'exercice.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PreviousRevaluations {
    /// Σ débit − Σ crédit, en CHF, des lignes `revaluation` du compte datées
    /// dans l'exercice.
    pub total: Decimal,
    /// Leurs écritures, par date puis `id`, au plus [`MAX_PREVIOUS_LISTED`].
    pub entry_ids: Vec<i64>,
}

/// Les réévaluations déjà passées dans l'exercice `fy`, par compte de
/// `account_ids` (A39, CR-8) : les lignes `rate_source = 'revaluation'` du
/// compte, datées de `start_date` à `end_date`. Lecture ordinaire, par
/// l'aperçu seulement. Un compte sans réévaluation est absent de la table.
pub async fn previous_revaluations(
    conn: &mut MySqlConnection,
    company_id: i64,
    fy: &FiscalYear,
    account_ids: &[i64],
) -> Result<HashMap<i64, PreviousRevaluations>, DbError> {
    if account_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let placeholders = std::iter::repeat_n("?", account_ids.len())
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT jel.account_id, je.id, jel.debit, jel.credit \
         FROM journal_entry_lines jel \
         INNER JOIN journal_entries je ON je.id = jel.entry_id \
         WHERE je.company_id = ? AND jel.rate_source = 'revaluation' \
           AND je.entry_date BETWEEN ? AND ? AND jel.account_id IN ({placeholders}) \
         ORDER BY je.entry_date ASC, je.id ASC"
    );
    let mut q = sqlx::query_as::<_, (i64, i64, Decimal, Decimal)>(&sql)
        .bind(company_id)
        .bind(fy.start_date)
        .bind(fy.end_date);
    for id in account_ids {
        q = q.bind(id);
    }
    let rows = q.fetch_all(&mut *conn).await.map_err(map_db_error)?;
    let mut out: HashMap<i64, PreviousRevaluations> = HashMap::new();
    for (account_id, entry_id, debit, credit) in rows {
        let p = out.entry(account_id).or_default();
        p.total += debit - credit;
        if p.entry_ids.len() < MAX_PREVIOUS_LISTED && !p.entry_ids.contains(&entry_id) {
            p.entry_ids.push(entry_id);
        }
    }
    Ok(out)
}

/// Le libellé d'une écriture de réévaluation, en français, partagé par
/// l'aperçu et la passation (DRY) : « Gain de change <compte> <DEVISE> <cours>
/// au JJ.MM.AAAA (non réalisé) » ou « Perte de change … ». Cours par unité,
/// sans zéros terminaux ; le nom du compte est tronqué pour que le libellé
/// tienne en [`MAX_LABEL_CHARS`] caractères.
pub fn revaluation_label(
    side: RevaluationSide,
    account_name: &str,
    currency: &str,
    rate: &Decimal,
    date: NaiveDate,
) -> String {
    let head = match side {
        RevaluationSide::Gain => "Gain de change",
        RevaluationSide::Loss => "Perte de change",
    };
    let tail = format!(
        " {currency} {} au {} (non réalisé)",
        rate.normalize(),
        date.format("%d.%m.%Y")
    );
    let fixed = head.chars().count() + 1 + tail.chars().count();
    let room = MAX_LABEL_CHARS.saturating_sub(fixed);
    let name: String = account_name.trim().chars().take(room).collect();
    format!("{head} {name}{tail}")
}

// ---------------------------------------------------------------------------
// Comptes de différence employés
// ---------------------------------------------------------------------------

/// Les verdicts des comptes de différence **employés** par des comptes (S2,
/// AC-4) : pour chaque `(compte, comptes résolus)`, `None` si sa paire est
/// valide, sinon le refus `FxAccountInvalid` — `reason: missing` (champ des
/// gains, `holderAccountId` = le compte) quand rien n'est résolu. Les paires
/// sont revalidées par [`accounts::validate_fx_accounts_in_tx`] (réglages,
/// comptes, articles `LOCK IN SHARE MODE`) ; la paire **de la société**, la
/// même pour tous les comptes qui la suivent, ne l'est qu'une fois. Une erreur
/// de base se propage aussitôt. Partagé par l'aperçu (`blockers[]`) et la
/// passation (dans l'ordre des comptes reçus, verdicts gardés sans être
/// propagés).
pub async fn fx_account_verdicts(
    conn: &mut MySqlConnection,
    company_id: i64,
    accounts: &[(i64, Option<FxAccounts>)],
) -> Result<Vec<(i64, Option<DbError>)>, DbError> {
    let mut company_pair: Option<Option<(FxAccountField, FxAccountReason)>> = None;
    let mut out = Vec::with_capacity(accounts.len());
    for (account_id, fx) in accounts {
        let verdict = match fx {
            None => Some(DbError::FxAccountInvalid {
                field: FxAccountField::Gain,
                reason: FxAccountReason::Missing,
                holder_account_id: Some(*account_id),
            }),
            Some(fx) if fx.origin == FxAccountsOrigin::Company => {
                if company_pair.is_none() {
                    company_pair = Some(
                        match accounts::validate_fx_accounts_in_tx(&mut *conn, company_id, fx).await
                        {
                            Ok(()) => None,
                            Err(DbError::FxAccountInvalid { field, reason, .. }) => {
                                Some((field, reason))
                            }
                            Err(e) => return Err(e),
                        },
                    );
                }
                company_pair
                    .flatten()
                    .map(|(field, reason)| DbError::FxAccountInvalid {
                        field,
                        reason,
                        holder_account_id: None,
                    })
            }
            Some(fx) => {
                match accounts::validate_fx_accounts_in_tx(&mut *conn, company_id, fx).await {
                    Ok(()) => None,
                    Err(e @ DbError::FxAccountInvalid { .. }) => Some(e),
                    Err(e) => return Err(e),
                }
            }
        };
        out.push((*account_id, verdict));
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Passation
// ---------------------------------------------------------------------------

/// Le cours **retenu** pour une devise de la passation, résolu par l'appelant
/// avant la transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetainedRate {
    /// Le cours, par unité.
    pub rate: Decimal,
    /// Son genre (règle unique de l'AC-1).
    pub kind: RevaluationRateKind,
    /// Sa source pour l'audit : celle du cours par défaut quand le cours retenu
    /// l'égale, `manual` sinon (valeur brute).
    pub source: String,
}

/// La demande de passation, préparée par le handler avant la transaction.
#[derive(Debug, Clone)]
pub struct PostRequest {
    /// L'exercice.
    pub fiscal_year_id: i64,
    /// Le genre de clôture lu avec l'exercice, avant la transaction : relu sous
    /// verrou, changé → `OPTIMISTIC_LOCK_CONFLICT`.
    pub closing_rate_kind: RateKind,
    /// Les cours retenus, par devise.
    pub rates: BTreeMap<String, RetainedRate>,
    /// Les comptes sélectionnés (distincts, non vides, au plus
    /// [`MAX_ACCOUNTS`] : contrôlés par l'appelant).
    pub account_ids: Vec<i64>,
}

/// Une écriture passée.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostedEntry {
    /// L'écriture.
    pub entry_id: i64,
    /// Le compte réévalué.
    pub account_id: i64,
    /// Sa devise.
    pub currency: String,
    /// Le cours retenu.
    pub rate: Decimal,
    /// L'écart `D`.
    pub delta: Decimal,
    /// Le compte de contrepartie (gains si `D > 0`, pertes sinon).
    pub counterpart_account_id: i64,
    /// Comptes propres ou réglage de la société.
    pub fx_accounts_origin: FxAccountsOrigin,
}

/// Passe la réévaluation de l'exercice (AC-2) : **une** écriture `OD` de deux
/// lignes par compte sélectionné à écart non nul, dans l'ordre des numéros,
/// toutes dans **une** transaction, puis l'audit `fx_revaluation.posted` —
/// aucune ligne d'audit quand rien n'est passé.
///
/// Verrous et ordre des lectures : doc du module. Ordre des refus, sous verrou
/// (AC-4) : exercice clos (`FiscalYearClosed`) ; antérieur ouvert
/// (`EarlierFiscalYearOpen`) ; comptes de différence employés par les comptes
/// admissibles (`FxAccountInvalid`, premier fautif par numéro, gain avant
/// perte ; `reason: missing` sans comptes résolus) ; comptes hors de la liste
/// admissible (`FxRevaluationAccountInvalid`, `notEligible`) ; genre de
/// clôture changé (`OptimisticLockConflict`) ; devise d'un compte admissible
/// sans cours retenu (`RateUnavailable`) ; calcul hors bornes
/// (`FxRevaluationValueOutOfRange`) ; puis les refus de `create_in_tx`
/// (`LaterFiscalYearClosed`, `DateOutsideFiscalYear`, `PeriodLocked`).
///
/// L'appelant l'enveloppe dans `retry::retry_on_deadlock`.
pub async fn post(
    pool: &MySqlPool,
    acteur: Acteur,
    company_id: i64,
    request: &PostRequest,
) -> Result<Vec<PostedEntry>, DbError> {
    // Avant `begin`, sur le pool : la date de début et les antérieurs (une
    // lecture ordinaire dans la transaction fixerait l'instantané trop tôt).
    let Some(before) =
        fiscal_years::find_by_id_in_company(pool, company_id, request.fiscal_year_id).await?
    else {
        return Err(DbError::NotFound);
    };
    let earlier = fiscal_years::list_earlier_ids(pool, company_id, before.start_date).await?;

    let mut tx = pool.begin().await.map_err(map_db_error)?;
    match post_in_tx(&mut tx, acteur, company_id, request, &earlier).await {
        Ok(entries) => {
            tx.commit().await.map_err(map_db_error)?;
            Ok(entries)
        }
        Err(e) => {
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

/// Le corps transactionnel de [`post`] ; ne touche pas à la transaction.
async fn post_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    acteur: Acteur,
    company_id: i64,
    request: &PostRequest,
    earlier: &[i64],
) -> Result<Vec<PostedEntry>, DbError> {
    // (0) La ligne `companies`, en partage, avant tout exercice.
    let company =
        accounts::lock_company_currency_in_tx(&mut *tx, company_id, accounts::CompanyLock::Share)
            .await?;
    let company_fx = (company.fx_gain_account_id, company.fx_loss_account_id);

    // (1)-(2) Antérieurs, exercice, plus ancien antérieur ouvert.
    let locked = fiscal_years::lock_with_earlier_in_tx(
        &mut *tx,
        company_id,
        request.fiscal_year_id,
        earlier,
    )
    .await?
    .ok_or(DbError::NotFound)?;

    // (3) Les comptes sélectionnés, en partage, par numéro.
    let placeholders = std::iter::repeat_n("?", request.account_ids.len())
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT {ACCOUNT_COLUMNS} FROM accounts a WHERE a.company_id = ? AND a.id IN ({placeholders}) \
         ORDER BY a.number ASC, a.id ASC LOCK IN SHARE MODE"
    );
    let mut q = sqlx::query_as::<_, RevaluationAccount>(&sql).bind(company_id);
    for id in &request.account_ids {
        q = q.bind(id);
    }
    let selected: Vec<RevaluationAccount> = q.fetch_all(&mut **tx).await.map_err(map_db_error)?;

    // Comptes de différence de chaque compte sélectionné : résolus, puis
    // revalidés (verrous) ; le verdict gardé par compte, sans être propagé.
    let pairs: Vec<(i64, Option<FxAccounts>)> = selected
        .iter()
        .map(|a| (a.id, fx_for(company_fx, a)))
        .collect();
    let mut fx_verdicts: HashMap<i64, Option<DbError>> =
        fx_account_verdicts(&mut *tx, company_id, &pairs)
            .await?
            .into_iter()
            .collect();

    // (4) Les soldes, après tous les verrous (la vue naît ici).
    let ids: Vec<i64> = selected.iter().map(|a| a.id).collect();
    let balances = if ids.is_empty() {
        Vec::new()
    } else {
        balances_as_of(&mut *tx, company_id, locked.target.end_date, Some(&ids)).await?
    };

    // Évaluation, dans l'ordre de l'AC-4.
    let target = &locked.target;
    if target.status == FiscalYearStatus::Closed {
        return Err(DbError::FiscalYearClosed);
    }
    if let Some(open) = &locked.earliest_open {
        return Err(DbError::EarlierFiscalYearOpen {
            fiscal_year_id: open.id,
            fiscal_year_name: open.name.clone(),
        });
    }
    let eligible: Vec<&RevaluationAccount> = selected.iter().filter(|a| a.is_eligible()).collect();
    for account in &eligible {
        if let Some(Some(e)) = fx_verdicts.remove(&account.id) {
            return Err(e);
        }
    }
    let eligible_ids: BTreeSet<i64> = eligible.iter().map(|a| a.id).collect();
    let not_eligible: Vec<i64> = request
        .account_ids
        .iter()
        .copied()
        .filter(|id| !eligible_ids.contains(id))
        .collect();
    if !not_eligible.is_empty() {
        return Err(DbError::FxRevaluationAccountInvalid {
            reason: RevaluationAccountReason::NotEligible,
            account_ids: not_eligible,
        });
    }
    if target.closing_rate_kind != request.closing_rate_kind {
        return Err(DbError::OptimisticLockConflict);
    }
    for account in &eligible {
        let currency = account.currency.as_deref().unwrap_or_default();
        if !request.rates.contains_key(currency) {
            return Err(DbError::RateUnavailable {
                currency: currency.to_string(),
                month: first_of_month(target.end_date),
            });
        }
    }
    let rate_values: BTreeMap<String, Decimal> = request
        .rates
        .iter()
        .map(|(c, r)| (c.clone(), r.rate))
        .collect();
    let eligible_accounts: Vec<RevaluationAccount> =
        eligible.iter().map(|a| (*a).clone()).collect();
    let lines = build_lines(&eligible_accounts, &balances, &rate_values, company_fx);
    if let Some(e) = lines.iter().find_map(RevaluationLine::out_of_range) {
        return Err(e);
    }

    // Une écriture par compte à écart non nul, dans l'ordre des numéros.
    let mut posted = Vec::new();
    for line in &lines {
        let (Some(delta), Some(rate), Some(fx)) = (line.delta(), line.rate, line.fx_accounts)
        else {
            continue;
        };
        let Some(side) = revaluation_side(&delta) else {
            continue;
        };
        let currency = line.account.currency.clone().unwrap_or_default();
        let amount = delta.abs();
        let (counterpart, revalued_debit, revalued_credit) = match side {
            RevaluationSide::Gain => (fx.gain, amount, Decimal::ZERO),
            RevaluationSide::Loss => (fx.loss, Decimal::ZERO, amount),
        };
        let new = NewJournalEntry {
            company_id,
            entry_date: target.end_date,
            journal: Journal::OD,
            description: revaluation_label(
                side,
                &line.account.name,
                &currency,
                &rate,
                target.end_date,
            ),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: line.account.id,
                    debit: revalued_debit,
                    credit: revalued_credit,
                    project_id: None,
                    foreign: Some(ForeignAmount {
                        declared_currency: None,
                        amount: Decimal::ZERO,
                        rate,
                        source: Some(RateProvenance::Revaluation),
                        rate_derived: false,
                    }),
                },
                NewJournalEntryLine {
                    account_id: counterpart,
                    debit: revalued_credit,
                    credit: revalued_debit,
                    project_id: None,
                    foreign: None,
                },
            ],
        };
        let created = journal_entries::create_in_tx(tx, target.id, acteur, new, true).await?;
        posted.push(PostedEntry {
            entry_id: created.entry.id,
            account_id: line.account.id,
            currency,
            rate,
            delta,
            counterpart_account_id: counterpart,
            fx_accounts_origin: fx.origin,
        });
    }

    if !posted.is_empty() {
        let mut rates = serde_json::Map::new();
        let mut sources = serde_json::Map::new();
        let mut kinds = serde_json::Map::new();
        for (currency, retained) in &request.rates {
            rates.insert(
                currency.clone(),
                serde_json::json!(retained.rate.normalize().to_string()),
            );
            sources.insert(currency.clone(), serde_json::json!(retained.source));
            kinds.insert(currency.clone(), serde_json::json!(retained.kind.as_str()));
        }
        let details = serde_json::json!({
            "entries": posted,
            "rates": rates,
            "sources": sources,
            "rateKinds": kinds,
            "closingRateKind": request.closing_rate_kind.as_str(),
            "accountIds": request.account_ids,
        });
        audit_log::insert_in_tx(
            &mut *tx,
            NewAuditLogEntry::par(
                acteur,
                "fx_revaluation.posted",
                "fiscal_year",
                target.id,
                Some(details),
            ),
        )
        .await?;
    }
    Ok(posted)
}

/// Raison du refus d'`accountIds` (`FX_REVALUATION_ACCOUNT_INVALID`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevaluationAccountReason {
    /// Liste vide.
    Empty,
    /// Identifiant en double.
    Duplicate,
    /// Plus de [`MAX_ACCOUNTS`] identifiants.
    TooMany,
    /// Compte d'une autre société, inexistant, ou hors de la liste admissible
    /// (de groupe, archivé, de résultat, sans devise) — sans fuite d'existence.
    NotEligible,
}

impl RevaluationAccountReason {
    /// La valeur de `details.reason`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Duplicate => "duplicate",
            Self::TooMany => "tooMany",
            Self::NotEligible => "notEligible",
        }
    }
}

/// Le contrôle de forme d'`accountIds` (avant la transaction) : vide,
/// doublons, plus de [`MAX_ACCOUNTS`] — dans cet ordre ; `Ok` sinon. Les
/// identifiants fautifs sont rendus **tels que reçus** (rien de lu en base).
pub fn check_account_ids_shape(ids: &[i64]) -> Result<(), DbError> {
    if ids.is_empty() {
        return Err(DbError::FxRevaluationAccountInvalid {
            reason: RevaluationAccountReason::Empty,
            account_ids: Vec::new(),
        });
    }
    let mut seen = BTreeSet::new();
    let duplicates: Vec<i64> = ids.iter().copied().filter(|id| !seen.insert(*id)).collect();
    if !duplicates.is_empty() {
        return Err(DbError::FxRevaluationAccountInvalid {
            reason: RevaluationAccountReason::Duplicate,
            account_ids: duplicates,
        });
    }
    if ids.len() > MAX_ACCOUNTS {
        return Err(DbError::FxRevaluationAccountInvalid {
            reason: RevaluationAccountReason::TooMany,
            account_ids: Vec::new(),
        });
    }
    Ok(())
}

/// Les comptes `ids` de la société, lecture ordinaire (avant la transaction de
/// la passation, pour choisir les cours à résoudre et contrôler les clés de
/// `rates` ; dans l'aperçu, pour juger la sélection). Un identifiant inconnu
/// ou d'une autre société est absent du résultat (KF-002).
pub async fn find_accounts(
    conn: &mut MySqlConnection,
    company_id: i64,
    ids: &[i64],
) -> Result<Vec<RevaluationAccount>, DbError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = std::iter::repeat_n("?", ids.len())
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT {ACCOUNT_COLUMNS} FROM accounts a WHERE a.company_id = ? AND a.id IN ({placeholders}) \
         ORDER BY a.number ASC, a.id ASC"
    );
    let mut q = sqlx::query_as::<_, RevaluationAccount>(&sql).bind(company_id);
    for id in ids {
        q = q.bind(id);
    }
    q.fetch_all(&mut *conn).await.map_err(map_db_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn libelle_gain_et_perte() {
        let d = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        assert_eq!(
            revaluation_label(
                RevaluationSide::Gain,
                "PAYPAL ONLINE",
                "USD",
                &dec!(0.911930),
                d
            ),
            "Gain de change PAYPAL ONLINE USD 0.91193 au 31.12.2024 (non réalisé)"
        );
        assert_eq!(
            revaluation_label(RevaluationSide::Loss, "Wise EUR", "EUR", &dec!(0.94947), d),
            "Perte de change Wise EUR EUR 0.94947 au 31.12.2024 (non réalisé)"
        );
    }

    #[test]
    fn libelle_tronque_sous_cinq_cents_caracteres() {
        let d = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let nom = "é".repeat(600);
        let l = revaluation_label(RevaluationSide::Loss, &nom, "USD", &dec!(0.9123456789), d);
        assert!(
            l.chars().count() <= MAX_LABEL_CHARS,
            "{}",
            l.chars().count()
        );
        assert!(l.ends_with("USD 0.9123456789 au 31.12.2024 (non réalisé)"));
    }

    #[test]
    fn forme_des_comptes_recus() {
        assert!(matches!(
            check_account_ids_shape(&[]),
            Err(DbError::FxRevaluationAccountInvalid {
                reason: RevaluationAccountReason::Empty,
                ..
            })
        ));
        match check_account_ids_shape(&[3, 4, 3]) {
            Err(DbError::FxRevaluationAccountInvalid {
                reason: RevaluationAccountReason::Duplicate,
                account_ids,
            }) => assert_eq!(account_ids, vec![3]),
            other => panic!("{other:?}"),
        }
        let trop: Vec<i64> = (1..=499).collect();
        assert!(matches!(
            check_account_ids_shape(&trop),
            Err(DbError::FxRevaluationAccountInvalid {
                reason: RevaluationAccountReason::TooMany,
                ..
            })
        ));
        let assez: Vec<i64> = (1..=498).collect();
        assert!(check_account_ids_shape(&assez).is_ok());
    }

    fn compte(id: i64, currency: Option<&str>, active: bool) -> RevaluationAccount {
        RevaluationAccount {
            id,
            number: format!("10{id}"),
            name: format!("Compte {id}"),
            account_type: AccountType::Asset,
            currency: currency.map(str::to_string),
            active,
            postable: true,
            fx_gain_account_id: None,
            fx_loss_account_id: None,
        }
    }

    #[test]
    fn k9_releve_ecart_hors_bornes_et_devise_sans_cours_a_solde() {
        let comptes = [
            compte(1, Some("USD"), true),  // écart non nul
            compte(2, Some("USD"), true),  // écart nul
            compte(3, Some("EUR"), true),  // sans cours, solde
            compte(4, Some("EUR"), true),  // sans cours, sans solde
            compte(5, Some("USD"), false), // archivé, écart non nul
        ];
        let soldes = vec![
            (1, dec!(880), Some(dec!(1000))),
            (2, dec!(900), Some(dec!(1000))),
            (3, dec!(10), Some(dec!(0))),
            (4, dec!(0), Some(dec!(0))),
            (5, dec!(1), Some(dec!(0))),
        ];
        let cours = BTreeMap::from([("USD".to_string(), dec!(0.9))]);
        let lignes = build_lines(&comptes, &soldes, &cours, (Some(10), Some(11)));
        let k9 = not_revalued(&lignes);
        assert_eq!(k9.count, 3);
        let ids: Vec<i64> = k9.accounts.iter().map(|a| a.account_id).collect();
        assert_eq!(ids, vec![1, 3, 5]);
        assert_eq!(k9.accounts[0].delta, Some(dec!(20.00)));
        assert_eq!(k9.accounts[1].reason, NotRevaluedReason::RateUnavailable);
        assert_eq!(k9.accounts[1].delta, None);
        // Comptes de la société pour un compte sans comptes propres.
        assert_eq!(
            lignes[0].fx_accounts.unwrap().origin,
            FxAccountsOrigin::Company
        );
    }
}
