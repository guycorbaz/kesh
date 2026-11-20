//! Conversion entre deux devises par le compte de conversion (Story 37-3b,
//! AC-2 bis, R7) : **deux écritures dans une transaction**, tout ou rien.
//!
//! Une conversion EUR → USD ne tient pas dans une écriture (une seule devise
//! étrangère par écriture, `FX_MULTIPLE_CURRENCIES`, 37-3a) : elle passe par le
//! compte de conversion en CHF de la société (`companies.fx_transit_account_id`,
//! 37-1). L'écriture 1 crédite le compte EUR et débite le compte de conversion ;
//! l'écriture 2 débite le compte USD et crédite le compte de conversion du même
//! CHF, la différence de change l'équilibrant.
//!
//! Le cœur, [`create_conversion`], reçoit deux écritures déjà analysées par la
//! route (forme, `foreign` complété, contrôle FX par ligne) et rend, dans
//! l'ordre, une seule raison de refus :
//!
//! 1. sous les verrous `companies` (`LOCK IN SHARE MODE`) → compte de
//!    conversion (`LOCK IN SHARE MODE`, [`accounts::check_fx_accounts_in_tx`]) :
//!    `FX_ACCOUNT_INVALID` (`field: fxTransitAccountId`) ;
//! 2. `FX_CONVERSION_TRANSIT_UNBALANCED` : le solde net du compte de conversion
//!    est un débit dans l'une, le même CHF au crédit dans l'autre ;
//!    Puis (étape « 2 bis ») les deux écritures ne sont pas dans une même
//!    devise (CHF comprise ; les comptes de charge, frais, ne donnent pas sa
//!    devise à une écriture qui porte autre chose ; un compte non reconnu
//!    laisse le refus à la création) : 400 `Validation` (`SameCurrency`),
//!    sans `details.entry` ;
//! 3. l'écriture **à compléter** est celle où ce solde est un crédit ; l'autre
//!    est validée (`accounting::validate`, `ENTRY_UNBALANCED` …) ;
//! 4. forme et paire de l'écriture à compléter (A31, A33), ajout éventuel de la
//!    ligne de différence (`completeDifference`), puis sa validation ;
//! 5. les deux créations, dans l'ordre **chronologique** (exercices pris
//!    `FOR UPDATE` dans l'ordre de la clôture), chacune par
//!    [`journal_entries::create_in_tx`] — tous les contrôles de
//!    `POST /journal-entries`.
//!
//! **Cycles de verrous** (argument écrit, défendu par le rejeu de la route,
//! `Rejouee`) : le compte de conversion, les comptes de l'écriture à compléter
//! et ses comptes de différence sont lus en partage **avant** l'exercice, alors
//! que l'écriture ordinaire prend l'exercice puis les comptes en partage ;
//! seule l'activation du multi-devises (`companies::update_currency_settings`)
//! prend des exercices puis des comptes **en exclusif** (les exercices ouverts,
//! puis les comptes de change désignés), mais jamais un compte de l'écriture à
//! compléter avant d'avoir pris `companies` `FOR UPDATE`, que la conversion
//! tient en partage : elle l'attend, sans cycle. *(Corrigé à la revue
//! d'intégration de l'équipe de Guy, 2026-11-17 : cette phrase disait
//! qu'aucun chemin ne prenait un exercice puis un compte en exclusif depuis
//! le retrait de la bascule `force` de 37-1, CR-7. Ailleurs, l'activation peut
//! s'interbloquer avec un règlement, unitaire ou groupé, quand un même compte
//! y sert aux deux rôles ; sans conséquence : les deux côtés sont rejoués, et
//! l'activation n'a lieu qu'une fois par société.)* Mais la sentinelle
//! `companies` fait des cycles, tous défendus par le rejeu (revue de code P1,
//! 37-3b-CR-P1-16) :
//! - quand une ligne porte un projet, l'étape 0 de `create_in_tx` prend
//!   `companies` `FOR UPDATE` alors que cette transaction la tient en partage
//!   (S → X) : deux conversions concurrentes portant chacune un projet
//!   s'interbloquent, et une conversion à projet s'interbloque aussi avec une
//!   écriture à projet déjà en attente du X (InnoDB fait attendre derrière les
//!   demandes en file) ;
//! - une conversion, même sans projet, prend `companies` en partage **puis**
//!   l'exercice `FOR UPDATE`, quand un flux qui tient déjà l'exercice prend
//!   ensuite `companies` `FOR UPDATE` (rapprochement ventilé ou lot avec
//!   projet, `create_in_tx` étape 0) : cycle exercice ↔ `companies`.
//!
//! La reconnaissance des comptes des lignes
//! ([`accounts::conversion_line_accounts_in_tx`]) verrouille les comptes lus,
//! non les lignes de la sous-requête `EXISTS` qui dit si un compte est désigné
//! comme compte propre par un autre : une désignation posée en parallèle peut
//! n'être pas vue (une ligne classée ou non comme ligne de différence), la
//! paire étant de toute façon revalidée sous verrou à l'usage.

use std::collections::{BTreeSet, HashMap};

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::MySqlPool;

use kesh_core::accounting::{self, JournalEntryDraft, JournalEntryLineDraft};
use kesh_core::errors::CoreError;
use kesh_core::types::Money;

use crate::entities::account::AccountType;
use crate::entities::audit_log::Acteur;
use crate::entities::{
    FiscalYearStatus, JournalEntryWithLines, NewJournalEntry, NewJournalEntryLine,
};
use crate::errors::{DbError, FxAccountField, FxAccountReason, map_db_error};
use crate::repositories::accounts::{
    self, CompanyLock, ConversionLineAccount, FxAccountCheck, FxAccountLock,
};
use crate::repositories::{fiscal_years, journal_entries};

/// Refus de forme propre à la conversion (400 `Validation`), avec la ligne
/// visée (rang 1-based dans le corps de l'écriture).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConversionValidation {
    /// `completeDifference: true` alors que l'écriture à compléter porte déjà
    /// une ligne de différence (A31) : le client choisit, jamais les deux.
    DifferenceLineAndComplete,
    /// Une ligne au **crédit** d'un compte de charge en devise (A33) : un frais
    /// de conversion se passe au débit.
    FeeOnCredit {
        /// Devise du compte de charge.
        currency: String,
    },
    /// Une ligne de différence portée sans **un seul** compte en devise hors
    /// comptes de charge : la paire est indéterminée.
    DifferenceLineWithoutPair,
    /// Une ligne de différence portée hors de la paire résolue (compte de
    /// gains au crédit, ou de pertes au débit, du compte en devise).
    DifferenceLineOffPair {
        /// Numéro du compte en devise dont la paire s'applique.
        account_number: String,
    },
    /// L'écriture compte déjà le nombre maximal de lignes : la ligne calculée
    /// ne peut s'y ajouter.
    TooManyLines,
    /// Les deux écritures sont dans la même devise (CHF comprise) : ce n'est
    /// pas une conversion, mais deux écritures ordinaires (revue de code P2,
    /// 37-3b-CR-P2-1, ex CR-P1-20).
    SameCurrency {
        /// Devise commune (`CHF` si aucune ligne n'est en devise).
        currency: String,
    },
}

/// Ce qui refuse une conversion.
#[derive(Debug)]
pub enum ConversionErrorKind {
    /// Refus du dépôt (création, compte de conversion, devises, interblocage…).
    Db(DbError),
    /// Refus de `accounting::validate` (équilibre, lignes, libellé…).
    Core(CoreError),
    /// Le compte de conversion ne revient pas à zéro : soldes nets (débit −
    /// crédit) dans chaque écriture.
    TransitUnbalanced {
        /// Solde net du compte de conversion dans l'écriture 1 du corps.
        entry1: Decimal,
        /// Solde net dans l'écriture 2 du corps.
        entry2: Decimal,
    },
    /// Refus de forme propre à la conversion.
    Validation {
        /// La raison.
        reason: ConversionValidation,
        /// Rang 1-based de la ligne visée dans le corps de l'écriture.
        line: Option<usize>,
    },
    /// Aucun exercice ne couvre la date de l'écriture.
    NoFiscalYear(NaiveDate),
    /// L'exercice qui couvre la date est clos.
    FiscalYearClosed(NaiveDate),
}

/// Refus d'une conversion : la raison, et l'écriture visée (`1` ou `2`, rang
/// dans le corps) — `None` pour un refus qui vise la conversion entière
/// (`FX_ACCOUNT_INVALID` du compte de conversion, transit déséquilibré).
#[derive(Debug)]
pub struct ConversionError {
    /// Rang de l'écriture visée dans le corps, `None` pour la conversion.
    pub entry: Option<usize>,
    /// La raison.
    pub kind: ConversionErrorKind,
}

impl ConversionError {
    fn conversion(kind: ConversionErrorKind) -> Self {
        Self { entry: None, kind }
    }

    fn entry(index: usize, kind: ConversionErrorKind) -> Self {
        Self {
            entry: Some(index + 1),
            kind,
        }
    }
}

impl From<DbError> for ConversionErrorKind {
    fn from(e: DbError) -> Self {
        Self::Db(e)
    }
}

/// Le solde net (débit − crédit) des lignes de `entry` sur `account_id`.
fn net_on(entry: &NewJournalEntry, account_id: i64) -> Decimal {
    entry
        .lines
        .iter()
        .filter(|l| l.account_id == account_id)
        .map(|l| l.debit - l.credit)
        .sum()
}

/// `accounting::validate` d'une écriture analysée (le brouillon de
/// `kesh-core` recomposé depuis ses lignes).
fn validate(entry: &NewJournalEntry) -> Result<(), CoreError> {
    let draft = JournalEntryDraft {
        date: entry.entry_date,
        journal: entry.journal.into(),
        description: entry.description.clone(),
        lines: entry
            .lines
            .iter()
            .map(|l| JournalEntryLineDraft {
                account_id: l.account_id,
                debit: Money::new(l.debit),
                credit: Money::new(l.credit),
                project_id: l.project_id,
                foreign: l.foreign.clone(),
            })
            .collect(),
    };
    accounting::validate(draft).map(|_| ())
}

/// Passe une conversion : les deux écritures de `entries` (dans l'ordre du
/// corps), dans **une** transaction, ou rien (voir l'en-tête du module pour
/// l'ordre des refus et les verrous).
///
/// `entries` : écritures déjà analysées par la route (libellé rogné, montants
/// lus, `foreign` complété et contrôlé ligne par ligne) ; `company_id` est
/// celui de l'appelant, posé par la route sur chacune. `complete_difference`
/// : le corps demande la ligne de différence calculée (A31). `max_lines` : la
/// limite de lignes d'une écriture de la route (`MAX_LINES_PER_ENTRY`), qui
/// s'applique au corps **complété**.
///
/// Réponse : les deux écritures créées, dans l'ordre du corps.
pub async fn create_conversion(
    pool: &MySqlPool,
    company_id: i64,
    acteur: Acteur,
    entries: [NewJournalEntry; 2],
    complete_difference: bool,
    max_lines: usize,
) -> Result<[JournalEntryWithLines; 2], ConversionError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ConversionError::conversion(map_db_error(e).into()))?;
    let result = create_conversion_in_tx(
        &mut tx,
        company_id,
        acteur,
        entries,
        complete_difference,
        max_lines,
    )
    .await;
    match result {
        Ok(created) => {
            tx.commit()
                .await
                .map_err(|e| ConversionError::conversion(map_db_error(e).into()))?;
            Ok(created)
        }
        Err(e) => {
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

async fn create_conversion_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    acteur: Acteur,
    mut entries: [NewJournalEntry; 2],
    complete_difference: bool,
    max_lines: usize,
) -> Result<[JournalEntryWithLines; 2], ConversionError> {
    let conv = |e: DbError| ConversionError::conversion(e.into());

    // --- 1. `companies` en partage, puis le compte de conversion. ---------
    let state = accounts::lock_company_currency_in_tx(tx, company_id, CompanyLock::Share)
        .await
        .map_err(conv)?;
    if !state.multi_currency_enabled {
        return Err(conv(DbError::MultiCurrencyDisabled {
            fiscal_year_id: None,
        }));
    }
    let Some(transit) = state.fx_transit_account_id else {
        return Err(conv(DbError::FxAccountInvalid {
            field: FxAccountField::Transit,
            reason: FxAccountReason::Missing,
            holder_account_id: None,
        }));
    };
    accounts::check_fx_accounts_in_tx(
        tx,
        company_id,
        &[FxAccountCheck {
            field: FxAccountField::Transit,
            account_id: transit,
            holder_account_id: None,
        }],
        FxAccountLock::Share,
    )
    .await
    .map_err(conv)?;

    // --- 2. Le compte de conversion revient à zéro. -----------------------
    let nets = [net_on(&entries[0], transit), net_on(&entries[1], transit)];
    let balanced = !nets[0].is_zero() && nets[0] == -nets[1];
    if !balanced {
        return Err(ConversionError::conversion(
            ConversionErrorKind::TransitUnbalanced {
                entry1: nets[0],
                entry2: nets[1],
            },
        ));
    }

    // --- 2 bis. Deux devises différentes (CHF comprise). ------------------
    let ids: Vec<i64> = entries
        .iter()
        .flat_map(|e| e.lines.iter().map(|l| l.account_id))
        .filter(|id| *id != transit)
        .collect();
    let read = accounts::conversion_line_accounts_in_tx(tx, company_id, &ids)
        .await
        .map_err(conv)?;
    // `None` dès qu'un compte n'est pas reconnu (inexistant, d'une autre
    // société) : sa devise est inconnue, le refus revient à la création
    // (revue de code P3, 37-3b-CR-P3-2 — il était lu comme un compte CHF).
    let currency_of = |entry: &NewJournalEntry| -> Option<BTreeSet<String>> {
        // (compte de charge ?, devise) de chaque ligne hors compte de conversion.
        let lines: Vec<(bool, String)> = entry
            .lines
            .iter()
            .filter(|l| l.account_id != transit)
            .map(|l| {
                let account = read.iter().find(|a| a.account.id == l.account_id)?;
                Some((
                    account.account.account_type == AccountType::Expense,
                    account
                        .account
                        .currency
                        .clone()
                        .unwrap_or_else(|| "CHF".to_string()),
                ))
            })
            .collect::<Option<_>>()?;
        // Les frais (comptes de charge) ne donnent pas sa devise à l'écriture,
        // sauf s'il n'y a rien d'autre.
        let main: BTreeSet<String> = lines
            .iter()
            .filter(|(expense, _)| !expense)
            .map(|(_, c)| c.clone())
            .collect();
        Some(if main.is_empty() {
            lines.into_iter().map(|(_, c)| c).collect()
        } else {
            main
        })
    };
    // Un compte inconnu ou une écriture à deux devises : refusés plus loin
    // (création, `FX_MULTIPLE_CURRENCIES`) ; ici seul le cas net d'une même
    // devise unique de part et d'autre.
    if let (Some(cur0), Some(cur1)) = (currency_of(&entries[0]), currency_of(&entries[1]))
        && cur0.len() == 1
        && cur0 == cur1
    {
        return Err(ConversionError::conversion(
            ConversionErrorKind::Validation {
                reason: ConversionValidation::SameCurrency {
                    currency: cur0.into_iter().next().unwrap_or_default(),
                },
                line: None,
            },
        ));
    }

    // --- 3. L'écriture à compléter (compte de conversion au crédit net), et
    // l'équilibre de l'autre. -----------------------------------------------
    let target = if nets[0] < Decimal::ZERO { 0 } else { 1 };
    let other = 1 - target;
    validate(&entries[other])
        .map_err(|e| ConversionError::entry(other, ConversionErrorKind::Core(e)))?;

    // --- 4. Forme et paire de l'écriture à compléter. ---------------------
    complete_entry(
        tx,
        company_id,
        &state,
        &mut entries[target],
        complete_difference,
        max_lines,
    )
    .await
    .map_err(|kind| ConversionError::entry(target, kind))?;
    validate(&entries[target])
        .map_err(|e| ConversionError::entry(target, ConversionErrorKind::Core(e)))?;

    // --- 5. Les deux créations, dans l'ordre chronologique. ---------------
    let order: [usize; 2] = if entries[1].entry_date < entries[0].entry_date {
        [1, 0]
    } else {
        [0, 1]
    };
    let mut created: [Option<JournalEntryWithLines>; 2] = [None, None];
    for idx in order {
        let date = entries[idx].entry_date;
        let fy = fiscal_years::find_covering_date_in_tx(tx, company_id, date)
            .await
            .map_err(|e| ConversionError::entry(idx, e.into()))?;
        let fy = match fy {
            None => {
                return Err(ConversionError::entry(
                    idx,
                    ConversionErrorKind::NoFiscalYear(date),
                ));
            }
            Some(fy) if fy.status == FiscalYearStatus::Closed => {
                return Err(ConversionError::entry(
                    idx,
                    ConversionErrorKind::FiscalYearClosed(date),
                ));
            }
            Some(fy) => fy,
        };
        let new = entries[idx].clone();
        let result = journal_entries::create_in_tx(tx, fy.id, acteur, new, true)
            .await
            .map_err(|e| {
                let kind = match e {
                    DbError::FiscalYearClosed => ConversionErrorKind::FiscalYearClosed(date),
                    other => ConversionErrorKind::Db(other),
                };
                ConversionError::entry(idx, kind)
            })?;
        created[idx] = Some(result);
    }
    let [a, b] = created;
    match (a, b) {
        (Some(a), Some(b)) => Ok([a, b]),
        _ => Err(conv(DbError::Invariant(
            "journal-entry-conversion:missing-entry".into(),
        ))),
    }
}

/// Un côté d'une **conversion de prestataire** (Story 38-11c2, AC-8) : le
/// compte du plan, le CHF de la ligne, et sa partie en devise (`None` pour un
/// compte en CHF). Construit par le plan pur `kesh_reconciliation::conversion::
/// plan_conversion`.
#[derive(Debug, Clone, PartialEq)]
pub struct ConversionLeg {
    /// Le compte du plan.
    pub ledger_account_id: i64,
    /// Le CHF de la ligne.
    pub chf: Decimal,
    /// La partie en devise, provenance posée.
    pub foreign: Option<crate::entities::ForeignAmount>,
}

/// Ce qu'écrit une conversion de prestataire (Story 38-11c2, AC-8 ; règle
/// commune aux prestataires, servie telle quelle à Wise et Stripe).
#[derive(Debug, Clone, PartialEq)]
pub enum ConversionPlan {
    /// Un côté en CHF : **une** écriture, débit `debit` (la cible) / crédit
    /// `credit` (la source), au même CHF.
    Single {
        /// Le côté crédité par le prestataire, débité au journal.
        debit: ConversionLeg,
        /// Le côté débité par le prestataire, crédité au journal.
        credit: ConversionLeg,
    },
    /// Deux devises étrangères : **deux** écritures par le compte de
    /// conversion, au même CHF.
    Double {
        /// Le côté source (crédité dans l'écriture 1).
        source: ConversionLeg,
        /// Le côté cible (débité dans l'écriture 2).
        target: ConversionLeg,
    },
}

/// Passe une conversion de prestataire **dans la transaction de l'appelant**
/// (Story 38-11c2, AC-8), sans `BEGIN` ni `COMMIT`, et rend les écritures
/// créées — une (côté CHF) ou deux (`[écriture de la source, écriture de la
/// cible]`).
///
/// - [`ConversionPlan::Single`] : une écriture au journal Banque, datée
///   `source_date`, par [`journal_entries::create_in_tx`] (garde de
///   postabilité désactivée, comme les autres flux de rapprochement) ; aucun
///   exercice ouvert → [`ConversionErrorKind::NoFiscalYear`] ou
///   [`ConversionErrorKind::FiscalYearClosed`].
/// - [`ConversionPlan::Double`] : écriture 1 (débit conversion / crédit source,
///   datée `source_date`) et écriture 2 (débit cible / crédit conversion,
///   datée `target_date`), par [`create_conversion_in_tx`]
///   (`complete_difference = false`), qui contrôle le compte de conversion
///   (`FX_ACCOUNT_INVALID`) et le mode multi-devises. Le compte de conversion
///   est lu ici d'abord, sous le même verrou partagé de `companies`, pour
///   construire les lignes.
///
/// Aucun nom de prestataire : PayPal, Wise et Stripe l'appellent tel quel.
pub async fn create_provider_conversion_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    acteur: Acteur,
    plan: ConversionPlan,
    source_date: NaiveDate,
    target_date: NaiveDate,
    description: String,
) -> Result<Vec<i64>, ConversionError> {
    let conv = |e: DbError| ConversionError::conversion(e.into());
    let line = |leg: &ConversionLeg, debit: bool| NewJournalEntryLine {
        account_id: leg.ledger_account_id,
        debit: if debit { leg.chf } else { Decimal::ZERO },
        credit: if debit { Decimal::ZERO } else { leg.chf },
        project_id: None,
        foreign: leg.foreign.clone(),
    };
    let entry = |date: NaiveDate, lines: Vec<NewJournalEntryLine>| NewJournalEntry {
        company_id,
        entry_date: date,
        journal: crate::entities::Journal::Banque,
        description: description.clone(),
        project_id: None,
        lines,
    };
    match plan {
        ConversionPlan::Single { debit, credit } => {
            let new = entry(source_date, vec![line(&debit, true), line(&credit, false)]);
            let fy = fiscal_years::find_covering_date_in_tx(tx, company_id, source_date)
                .await
                .map_err(conv)?;
            let fy = match fy {
                None => {
                    return Err(ConversionError::entry(
                        0,
                        ConversionErrorKind::NoFiscalYear(source_date),
                    ));
                }
                Some(fy) if fy.status == FiscalYearStatus::Closed => {
                    return Err(ConversionError::entry(
                        0,
                        ConversionErrorKind::FiscalYearClosed(source_date),
                    ));
                }
                Some(fy) => fy,
            };
            let created = journal_entries::create_in_tx(tx, fy.id, acteur, new, false)
                .await
                .map_err(|e| {
                    let kind = match e {
                        DbError::FiscalYearClosed => {
                            ConversionErrorKind::FiscalYearClosed(source_date)
                        }
                        other => ConversionErrorKind::Db(other),
                    };
                    ConversionError::entry(0, kind)
                })?;
            Ok(vec![created.entry.id])
        }
        ConversionPlan::Double { source, target } => {
            let state = accounts::lock_company_currency_in_tx(tx, company_id, CompanyLock::Share)
                .await
                .map_err(conv)?;
            let Some(transit) = state.fx_transit_account_id else {
                return Err(conv(DbError::FxAccountInvalid {
                    field: FxAccountField::Transit,
                    reason: FxAccountReason::Missing,
                    holder_account_id: None,
                }));
            };
            let transit_leg = |chf: Decimal| ConversionLeg {
                ledger_account_id: transit,
                chf,
                foreign: None,
            };
            let first = entry(
                source_date,
                vec![line(&transit_leg(source.chf), true), line(&source, false)],
            );
            let second = entry(
                target_date,
                vec![line(&target, true), line(&transit_leg(target.chf), false)],
            );
            let [a, b] = create_conversion_in_tx(
                tx,
                company_id,
                acteur,
                [first, second],
                false,
                PROVIDER_CONVERSION_MAX_LINES,
            )
            .await?;
            Ok(vec![a.entry.id, b.entry.id])
        }
    }
}

/// Limite de lignes passée à [`create_conversion_in_tx`] pour une conversion
/// de prestataire : sans effet (deux lignes par écriture, aucune ligne de
/// différence demandée).
const PROVIDER_CONVERSION_MAX_LINES: usize = 50;

/// Forme, paire et ligne de différence de l'écriture à compléter (AC-2 bis,
/// V3, A31, A33), dans l'ordre des refus de la fiche ; ajoute la ligne
/// calculée quand le corps la demande. L'équilibre final est jugé par
/// l'appelant.
async fn complete_entry(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    state: &accounts::CompanyCurrencyState,
    entry: &mut NewJournalEntry,
    complete_difference: bool,
    max_lines: usize,
) -> Result<(), ConversionErrorKind> {
    // Reconnaissance des comptes des lignes (sous le verrou de `companies`).
    let ids: Vec<i64> = entry.lines.iter().map(|l| l.account_id).collect();
    let read = accounts::conversion_line_accounts_in_tx(tx, company_id, &ids).await?;
    let by_id: HashMap<i64, &ConversionLineAccount> =
        read.iter().map(|a| (a.account.id, a)).collect();
    let designated = |id: i64| {
        state.fx_gain_account_id == Some(id)
            || state.fx_loss_account_id == Some(id)
            || by_id.get(&id).is_some_and(|a| a.designated_by_account != 0)
    };
    let currency_of = |id: i64| by_id.get(&id).and_then(|a| a.account.currency.clone());
    let is_expense = |id: i64| {
        by_id
            .get(&id)
            .is_some_and(|a| a.account.account_type == AccountType::Expense)
    };

    // Lignes de différence portées (rang 1-based).
    let difference_lines: Vec<usize> = entry
        .lines
        .iter()
        .enumerate()
        .filter(|(_, l)| designated(l.account_id))
        .map(|(i, _)| i + 1)
        .collect();
    let total_debit: Decimal = entry.lines.iter().map(|l| l.debit).sum();
    let total_credit: Decimal = entry.lines.iter().map(|l| l.credit).sum();
    let unbalanced = total_debit != total_credit;

    // A31 : la ligne calculée et une ligne portée, jamais les deux.
    if complete_difference && let Some(&line) = difference_lines.first() {
        return Err(ConversionErrorKind::Validation {
            reason: ConversionValidation::DifferenceLineAndComplete,
            line: Some(line),
        });
    }
    let to_add = complete_difference && unbalanced;
    if difference_lines.is_empty() && !to_add {
        // Équilibrée sans ligne de différence, ou sans la demande : la forme
        // n'est pas examinée, aucune paire n'est lue.
        return Ok(());
    }

    // Comptes en devise de deux devises (charges comprises) → le code du dépôt,
    // anticipé pour ne pas rendre un `ENTRY_UNBALANCED` trompeur (A33).
    let currencies: BTreeSet<String> = entry
        .lines
        .iter()
        .filter_map(|l| currency_of(l.account_id))
        .collect();
    if currencies.len() > 1 {
        return Err(DbError::FxMultipleCurrencies {
            currencies: currencies.into_iter().collect(),
        }
        .into());
    }
    // Un frais au crédit d'un compte de charge en devise (A33).
    if let Some((i, l)) = entry.lines.iter().enumerate().find(|(_, l)| {
        is_expense(l.account_id) && currency_of(l.account_id).is_some() && l.credit > Decimal::ZERO
    }) {
        return Err(ConversionErrorKind::Validation {
            reason: ConversionValidation::FeeOnCredit {
                currency: currency_of(l.account_id).unwrap_or_default(),
            },
            line: Some(i + 1),
        });
    }

    // Le compte en devise de l'écriture : un seul, hors comptes de charge.
    let foreign_accounts: BTreeSet<i64> = entry
        .lines
        .iter()
        .map(|l| l.account_id)
        .filter(|id| currency_of(*id).is_some() && !is_expense(*id))
        .collect();
    let holder = if foreign_accounts.len() == 1 {
        foreign_accounts.iter().next().copied()
    } else {
        None
    };
    let Some(holder) = holder else {
        if let Some(&line) = difference_lines.first() {
            return Err(ConversionErrorKind::Validation {
                reason: ConversionValidation::DifferenceLineWithoutPair,
                line: Some(line),
            });
        }
        // Sans la forme exigée, l'écriture n'est jamais complétée : son
        // déséquilibre rend `ENTRY_UNBALANCED` à la validation.
        return Ok(());
    };
    // Complétée seulement si le compte en devise est au débit net.
    let holder_at_debit = net_on(entry, holder) > Decimal::ZERO;
    if difference_lines.is_empty() && !holder_at_debit {
        return Ok(());
    }

    // La paire, résolue puis revalidée sous verrou, à l'usage seulement.
    let holder_account = &by_id
        .get(&holder)
        .ok_or_else(|| DbError::Invariant("journal-entry-conversion:holder".into()))?
        .account;
    let fx = accounts::fx_accounts_for_settings(
        state.fx_gain_account_id,
        state.fx_loss_account_id,
        holder_account,
    )
    .ok_or(DbError::FxAccountInvalid {
        // Le champ réellement manquant de la société (le compte en devise n'a
        // pas de paire propre) : gains d'abord, pertes sinon (revue de code P1,
        // 37-3b-CR-P1-15).
        field: if state.fx_gain_account_id.is_none() {
            FxAccountField::Gain
        } else {
            FxAccountField::Loss
        },
        reason: FxAccountReason::Missing,
        holder_account_id: None,
    })?;
    accounts::validate_fx_accounts_in_tx(tx, company_id, &fx).await?;

    // Une ligne portée emploie la paire : gains au crédit, pertes au débit.
    for &line in &difference_lines {
        let l = &entry.lines[line - 1];
        let on_pair = (l.account_id == fx.gain && l.credit > Decimal::ZERO)
            || (l.account_id == fx.loss && l.debit > Decimal::ZERO);
        if !on_pair {
            return Err(ConversionErrorKind::Validation {
                reason: ConversionValidation::DifferenceLineOffPair {
                    account_number: holder_account.number.clone(),
                },
                line: Some(line),
            });
        }
    }

    if !(to_add && holder_at_debit) {
        return Ok(());
    }
    if entry.lines.len() >= max_lines {
        return Err(ConversionErrorKind::Validation {
            reason: ConversionValidation::TooManyLines,
            line: None,
        });
    }
    // La ligne calculée, en dernière ligne : |Σ débit − Σ crédit| (frais
    // compris), au crédit du compte de gains si les débits l'emportent, au
    // débit du compte de pertes sinon.
    let gap = total_debit - total_credit;
    // Au-delà de DECIMAL(19,4) l'INSERT échouerait en 500 : refusé ici en 400
    // (revue de code P2, 37-3b-CR-P2-2, ex CR-P1-21).
    // Même borne que le CHF d'une ligne en devise (`> FX_CHF_MAX`, 37-3a).
    if gap.abs() > accounting::fx::FX_CHF_MAX {
        return Err(ConversionErrorKind::Core(CoreError::FxValueOutOfRange {
            line: entry.lines.len() + 1,
        }));
    }
    let (account_id, debit, credit) = if gap > Decimal::ZERO {
        (fx.gain, Decimal::ZERO, gap)
    } else {
        (fx.loss, -gap, Decimal::ZERO)
    };
    entry.lines.push(NewJournalEntryLine {
        account_id,
        debit,
        credit,
        project_id: None,
        // Ligne de différence de change : toujours en CHF.
        foreign: None,
    });
    Ok(())
}
