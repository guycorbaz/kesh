//! Complément des soldes de départ (Story 25-7, #445).
//!
//! Une fois l'écriture d'ouverture générée, l'écran des soldes de départ se
//! verrouille : la génération exige une société **sans aucune écriture**. Un
//! compte de bilan oublié à la saisie se rattrape ici, par une **écriture
//! d'ajustement** — journal `OD`, libellé « Complément des soldes de départ » —
//! dont la contrepartie est portée au compte de rôle `RetainedEarnings`
//! (report à-nouveau), calculée par le serveur.
//!
//! # Ce qui est complétable
//!
//! Un compte de bilan (`Asset` / `Liability`), actif, imputable, **jamais
//! mouvementé** (aucune ligne d'écriture, dans aucun exercice), et qui n'est
//! pas le compte de report lui-même. Une écriture contre-passée garde ses
//! lignes : le compte reste mouvementé.
//!
//! # La date (arbitrage 2)
//!
//! - **(a)** le premier jour du premier exercice, s'il est ouvert et que ce jour
//!   est **après** `books_locked_through` — le compte oublié faisait partie de
//!   l'ouverture ;
//! - **(b)** sinon **aujourd'hui** (date UTC, passée en paramètre), dans
//!   l'exercice ouvert qui la couvre — une régularisation ;
//! - **(c)** sinon refus nommé.
//!
//! # Verrous — la règle, et pourquoi elle n'est pas un ordre « sans cycle »
//!
//! Le dépôt n'a **pas** d'ordre de verrouillage unique (une écriture sans
//! projet prend l'exercice puis ses comptes ; le règlement prend un compte puis
//! l'exercice ; la contre-passation l'écriture puis les exercices). Aucun ordre
//! ne rend le complément exempt de cycle. [`create_opening_complement`] suit donc
//! une **règle** : tous les verrous d'abord, **une ligne par requête** ; les
//! lectures ordinaires ensuite (l'instantané REPEATABLE READ s'ouvre là, après
//! les verrous) ; les refus en dernier, dans l'ordre de priorité du status. Les
//! cycles résiduels sont décrits dans la fiche de la story (section
//! « Cycles ») ; l'appelant enveloppe l'appel dans l'enveloppe `DbError`
//! [`crate::retry::retry_on_deadlock`].
//!
//! ⛔ `FOR SHARE` est une **erreur de syntaxe** sur MariaDB 10.11 : le verrou
//! partagé s'écrit `LOCK IN SHARE MODE`.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{MySql, MySqlPool, QueryBuilder, Transaction};

use kesh_core::accounting::{
    self, Journal as CoreJournal, JournalEntryDraft, JournalEntryLineDraft,
};
use kesh_core::types::Money;

use crate::entities::{Journal, JournalEntryWithLines, NewJournalEntry, NewJournalEntryLine};
use crate::errors::{DbError, map_db_error};
use crate::repositories::journal_entries;

/// Valeur de `accounts.role` / `accounts.singleton_role` du compte de report.
const RETAINED_EARNINGS_ROLE: &str = "RetainedEarnings";

/// Pourquoi un complément est refusé (POST) ou indisponible (status).
///
/// L'ordre des variantes **est** l'ordre de priorité d'évaluation : une seule
/// raison est rendue, la première qui s'applique (AC 3 / AC 4 étape 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OpeningComplementRefusal {
    /// La société n'a aucune écriture (ou aucun exercice) : il n'y a rien à
    /// compléter, la génération de l'ouverture s'applique.
    NoEntries,
    /// Branche (b) : aucun exercice **ouvert** ne couvre la date du jour.
    NoOpenFiscalYear,
    /// Branche (b) : la date du jour tombe dans la période verrouillée.
    /// Défensif — `lock_books` refuse une borne `>= aujourd'hui`.
    DateLocked,
    /// Aucun compte **actif** ne porte le rôle `RetainedEarnings`.
    NoRetainedEarnings,
    /// Le compte de report existe mais n'est pas imputable.
    RetainedEarningsNotPostable,
    /// Compte inconnu, d'une autre société, archivé ou non imputable.
    AccountInvalid,
    /// Le compte de report est saisi comme ligne : la contrepartie est
    /// calculée par le serveur.
    RetainedEarningsLine,
    /// Compte de résultat (`Revenue` / `Expense`).
    NotBalanceAccount,
    /// Le compte porte déjà au moins une ligne d'écriture.
    AccountMoved,
}

impl OpeningComplementRefusal {
    /// Code machine stable, rendu tel quel au client.
    pub fn code(self) -> &'static str {
        match self {
            Self::NoEntries => "OPENING_COMPLEMENT_NO_ENTRIES",
            Self::NoOpenFiscalYear => "OPENING_COMPLEMENT_NO_OPEN_FISCAL_YEAR",
            Self::DateLocked => "OPENING_COMPLEMENT_DATE_LOCKED",
            Self::NoRetainedEarnings => "OPENING_COMPLEMENT_NO_RETAINED_EARNINGS",
            Self::RetainedEarningsNotPostable => {
                "OPENING_COMPLEMENT_RETAINED_EARNINGS_NOT_POSTABLE"
            }
            Self::AccountInvalid => "OPENING_COMPLEMENT_ACCOUNT_INVALID",
            Self::RetainedEarningsLine => "OPENING_COMPLEMENT_RETAINED_EARNINGS_LINE",
            Self::NotBalanceAccount => "OPENING_COMPLEMENT_NOT_BALANCE_ACCOUNT",
            Self::AccountMoved => "OPENING_COMPLEMENT_ACCOUNT_MOVED",
        }
    }

    /// Valeur de `completeReason` dans le status (`READY` exclu).
    pub fn status_reason(self) -> &'static str {
        match self {
            Self::NoEntries => "NO_ENTRIES",
            Self::NoOpenFiscalYear => "NO_OPEN_FISCAL_YEAR",
            Self::DateLocked => "DATE_LOCKED",
            Self::NoRetainedEarnings => "NO_RETAINED_EARNINGS",
            Self::RetainedEarningsNotPostable => "RETAINED_EARNINGS_NOT_POSTABLE",
            // Refus par compte : le status ne les rend jamais (il ne propose
            // que des comptes complétables) ; libellés pour l'exhaustivité.
            Self::AccountInvalid => "ACCOUNT_INVALID",
            Self::RetainedEarningsLine => "RETAINED_EARNINGS_LINE",
            Self::NotBalanceAccount => "NOT_BALANCE_ACCOUNT",
            Self::AccountMoved => "ACCOUNT_MOVED",
        }
    }
}

/// Une ligne saisie : montants déjà validés par le handler (strictement
/// positifs, une seule colonne non nulle, au plus quatre décimales).
#[derive(Debug, Clone)]
pub struct ComplementLine {
    pub account_id: i64,
    pub debit: Decimal,
    pub credit: Decimal,
}

/// Branche de la date retenue (arbitrage 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplementDateBranch {
    /// (a) premier jour du premier exercice.
    OpeningDay,
    /// (b) date du jour, régularisation.
    Today,
}

impl ComplementDateBranch {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OpeningDay => "OPENING_DAY",
            Self::Today => "TODAY",
        }
    }
}

/// Date et exercice retenus pour le complément.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComplementDate {
    pub fiscal_year_id: i64,
    pub fiscal_year_name: String,
    pub date: NaiveDate,
    pub branch: ComplementDateBranch,
}

/// Exercice tel que le lisent les deux requêtes à une ligne.
#[derive(Debug, Clone, sqlx::FromRow)]
struct FyRow {
    id: i64,
    name: String,
    start_date: NaiveDate,
    end_date: NaiveDate,
    status: String,
}

/// Compte de report de la société.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RetainedEarningsAccount {
    pub id: i64,
    pub number: String,
    pub name: String,
    pub postable: bool,
}

/// Compte proposé au complément.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CompletableAccount {
    pub id: i64,
    pub number: String,
    pub name: String,
    pub account_type: String,
}

/// État du mode « compléter », pour le status (lecture **sans verrou** : le
/// POST fait autorité).
#[derive(Debug, Clone)]
pub struct ComplementStatus {
    /// `None` = `READY`.
    pub refusal: Option<OpeningComplementRefusal>,
    /// `true` quand aucune autre raison ne s'applique mais que la liste est
    /// vide (`NO_COMPLETABLE_ACCOUNT`).
    pub no_completable_account: bool,
    pub completable_accounts: Vec<CompletableAccount>,
    pub date: Option<ComplementDate>,
    pub retained_earnings: Option<RetainedEarningsAccount>,
}

impl ComplementStatus {
    /// Valeur de `completeReason`.
    pub fn complete_reason(&self) -> &'static str {
        match self.refusal {
            Some(r) => r.status_reason(),
            None if self.no_completable_account => "NO_COMPLETABLE_ACCOUNT",
            None => "READY",
        }
    }

    /// `canComplete` : vrai seulement si `completeReason = READY`.
    pub fn can_complete(&self) -> bool {
        self.refusal.is_none() && !self.no_completable_account
    }
}

// ---------------------------------------------------------------------------
// Décision pure : la date (arbitrage 2)
// ---------------------------------------------------------------------------

/// Choisit la date et l'exercice du complément.
///
/// `first` est le premier exercice de la société ; `candidate` celui dont le
/// début est le plus récent parmi ceux qui ont commencé au plus tard `today`
/// (les exercices ne se chevauchant pas, c'est le seul qui puisse couvrir
/// `today` — sa fin et son statut sont contrôlés ici).
fn decide_date(
    first: &FyRow,
    candidate: Option<&FyRow>,
    locked_through: Option<NaiveDate>,
    today: NaiveDate,
) -> Result<ComplementDate, OpeningComplementRefusal> {
    // (a) premier exercice ouvert, son premier jour hors période verrouillée.
    let opening_day_unlocked = locked_through.is_none_or(|l| first.start_date > l);
    if first.status == "Open" && opening_day_unlocked {
        return Ok(ComplementDate {
            fiscal_year_id: first.id,
            fiscal_year_name: first.name.clone(),
            date: first.start_date,
            branch: ComplementDateBranch::OpeningDay,
        });
    }

    // (b) aujourd'hui, dans l'exercice ouvert qui le couvre.
    let covering = candidate.filter(|fy| fy.end_date >= today && fy.status == "Open");
    let Some(fy) = covering else {
        return Err(OpeningComplementRefusal::NoOpenFiscalYear);
    };
    // (c) défensif : `lock_books` refuse une borne `>= aujourd'hui`.
    if locked_through.is_some_and(|l| today <= l) {
        return Err(OpeningComplementRefusal::DateLocked);
    }
    Ok(ComplementDate {
        fiscal_year_id: fy.id,
        fiscal_year_name: fy.name.clone(),
        date: today,
        branch: ComplementDateBranch::Today,
    })
}

/// Contrôle du compte de report : présent et imputable.
fn check_retained_earnings(
    retained: Option<&RetainedEarningsAccount>,
) -> Result<&RetainedEarningsAccount, OpeningComplementRefusal> {
    match retained {
        None => Err(OpeningComplementRefusal::NoRetainedEarnings),
        Some(r) if !r.postable => Err(OpeningComplementRefusal::RetainedEarningsNotPostable),
        Some(r) => Ok(r),
    }
}

// ---------------------------------------------------------------------------
// Requêtes
// ---------------------------------------------------------------------------

/// Premier exercice : `ORDER BY start_date` **sans** départage `, id`.
///
/// ⛔ Mesuré sur MariaDB 10.11 (validation P6 de la story) : avec `, id`, le
/// plan passe par `Using filesort`, qui lit — donc, en `FOR UPDATE`, verrouille
/// — **tous** les exercices de la société. L'unicité `(company_id, start_date)`
/// rend le départage inutile.
const FIRST_FY_SQL: &str = "SELECT id, name, start_date, end_date, status FROM fiscal_years \
     WHERE company_id = ? ORDER BY start_date LIMIT 1";

/// Candidat du jour : le dernier exercice commencé au plus tard `today`.
///
/// ⛔ Ne pas réutiliser `fiscal_years::find_open_covering_date` sous verrou :
/// `end_date` et `status` étant hors index, il verrouille tous les exercices
/// qu'il parcourt, depuis le premier (mesuré, validation P6).
const CANDIDATE_FY_SQL: &str = "SELECT id, name, start_date, end_date, status FROM fiscal_years \
     WHERE company_id = ? AND start_date <= ? ORDER BY start_date DESC LIMIT 1";

const RETAINED_SQL: &str = "SELECT id, number, name, postable FROM accounts \
     WHERE company_id = ? AND singleton_role = ?";

const COMPLETABLE_SQL: &str = "SELECT a.id, a.number, a.name, a.account_type FROM accounts a \
     WHERE a.company_id = ? AND a.active = TRUE AND a.postable = TRUE \
       AND a.account_type IN ('Asset', 'Liability') \
       AND (a.role IS NULL OR a.role <> ?) \
       AND NOT EXISTS (SELECT 1 FROM journal_entry_lines l WHERE l.account_id = a.id) \
     ORDER BY a.number";

async fn has_entries<'e, E>(executor: E, company_id: i64) -> Result<bool, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM journal_entries WHERE company_id = ?)",
    )
    .bind(company_id)
    .fetch_one(executor)
    .await
    .map_err(map_db_error)
}

/// État du mode « compléter » (status). Lectures **sans verrou**.
pub async fn complement_status(
    pool: &MySqlPool,
    company_id: i64,
    today: NaiveDate,
) -> Result<ComplementStatus, DbError> {
    let locked_through: Option<NaiveDate> =
        sqlx::query_scalar("SELECT books_locked_through FROM companies WHERE id = ?")
            .bind(company_id)
            .fetch_optional(pool)
            .await
            .map_err(map_db_error)?
            .flatten();
    let first: Option<FyRow> = sqlx::query_as(FIRST_FY_SQL)
        .bind(company_id)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)?;
    let candidate: Option<FyRow> = sqlx::query_as(CANDIDATE_FY_SQL)
        .bind(company_id)
        .bind(today)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)?;
    let retained: Option<RetainedEarningsAccount> = sqlx::query_as(RETAINED_SQL)
        .bind(company_id)
        .bind(RETAINED_EARNINGS_ROLE)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)?;
    let completable: Vec<CompletableAccount> = sqlx::query_as(COMPLETABLE_SQL)
        .bind(company_id)
        .bind(RETAINED_EARNINGS_ROLE)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)?;
    let entries = has_entries(pool, company_id).await?;

    let mut status = ComplementStatus {
        refusal: None,
        no_completable_account: false,
        completable_accounts: completable,
        date: None,
        retained_earnings: retained,
    };

    // Priorité : NO_ENTRIES (dont « aucun exercice ») > date > report > liste.
    let Some(first) = first.filter(|_| entries) else {
        status.refusal = Some(OpeningComplementRefusal::NoEntries);
        return Ok(status);
    };
    match decide_date(&first, candidate.as_ref(), locked_through, today) {
        Ok(date) => status.date = Some(date),
        Err(r) => {
            status.refusal = Some(r);
            return Ok(status);
        }
    }
    if let Err(r) = check_retained_earnings(status.retained_earnings.as_ref()) {
        status.refusal = Some(r);
        return Ok(status);
    }
    status.no_completable_account = status.completable_accounts.is_empty();
    Ok(status)
}

/// Compte saisi, lu sous verrou.
#[derive(Debug, sqlx::FromRow)]
struct LockedAccount {
    id: i64,
    number: String,
    account_type: String,
    active: bool,
    postable: bool,
    role: Option<String>,
}

fn refused(reason: OpeningComplementRefusal, account: Option<(i64, Option<String>)>) -> DbError {
    let (account_id, account_number) = match account {
        Some((id, number)) => (Some(id), number),
        None => (None, None),
    };
    DbError::OpeningComplementRefused {
        reason,
        account_id,
        account_number,
    }
}

/// Crée l'écriture de complément (AC 4). Une seule transaction ; l'appelant
/// l'enveloppe dans [`crate::retry::retry_on_deadlock`] (cycles résiduels, cf.
/// doc du module).
///
/// Ordre (règle « verrous d'abord, une ligne par requête ») :
/// 1. `companies` en **partagé** (`LOCK IN SHARE MODE`) : fige
///    `books_locked_through` contre `lock_books` ;
/// 2. comptes saisis `FOR UPDATE` **par clé primaire** (arrête la première
///    écriture concurrente sur ces comptes, dont les lignes demandent un
///    partagé de clé étrangère), puis compte de report en partagé ;
/// 3. premier exercice puis candidat du jour, `FOR UPDATE`, une ligne chacun —
///    **avant** de choisir la branche de date ;
/// 4. lectures ordinaires (l'instantané s'ouvre ici) : « jamais mouvementé »,
///    puis « la société a des écritures » ;
/// 5. refus, dans l'ordre de [`OpeningComplementRefusal`] ;
/// 6. contrepartie sur le compte de report ;
/// 7. `accounting::validate`, puis `create_in_tx` (qui reverrouille l'exercice
///    déjà tenu, numérote, audite `journal_entry.created`).
pub async fn create_opening_complement(
    pool: &MySqlPool,
    company_id: i64,
    user_id: i64,
    lines: &[ComplementLine],
    description: String,
    today: NaiveDate,
) -> Result<JournalEntryWithLines, DbError> {
    // Les comptes de la société parmi ceux saisis, lus **hors** de la
    // transaction (autocommit) : sous REPEATABLE READ, une lecture ordinaire
    // dans la transaction ouvrirait l'instantané avant les verrous. Un compte ne
    // change jamais de société, donc ce tri ne peut pas devenir faux ensuite.
    //
    // ⛔ Sans lui, le `FOR UPDATE` par clé primaire verrouillait le compte d'une
    // AUTRE société avant que le filtre `company_id` ne l'écarte — mesuré à deux
    // sessions (revue de code P1, B-F1). Un identifiant étranger n'est donc
    // jamais verrouillé ; il est refusé `ACCOUNT_INVALID` à l'étape des refus.
    // L'appelant garantit au moins une ligne (la route refuse `NO_LINES`) ; un
    // appel direct sans ligne est un défaut de l'appelant, pas une requête SQL
    // `IN ()` invalide (revue de code P2, R-L4a).
    if lines.is_empty() {
        return Err(DbError::InvalidInput("opening-complement:no-lines".into()));
    }
    let owned = owned_account_ids(pool, company_id, lines).await?;
    let mut tx = pool.begin().await.map_err(map_db_error)?;
    match create_in_open_tx(
        &mut tx,
        company_id,
        user_id,
        lines,
        &owned,
        description,
        today,
    )
    .await
    {
        Ok(result) => {
            tx.commit().await.map_err(map_db_error)?;
            Ok(result)
        }
        Err(e) => {
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

/// Identifiants des comptes saisis qui appartiennent à la société, triés.
async fn owned_account_ids(
    pool: &MySqlPool,
    company_id: i64,
    lines: &[ComplementLine],
) -> Result<Vec<i64>, DbError> {
    let mut qb: QueryBuilder<MySql> =
        QueryBuilder::new("SELECT id FROM accounts WHERE company_id = ");
    qb.push_bind(company_id).push(" AND id IN (");
    {
        let mut sep = qb.separated(", ");
        for line in lines {
            sep.push_bind(line.account_id);
        }
    }
    qb.push(") ORDER BY id");
    qb.build_query_scalar()
        .fetch_all(pool)
        .await
        .map_err(map_db_error)
}

#[allow(clippy::too_many_arguments)]
async fn create_in_open_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    user_id: i64,
    lines: &[ComplementLine],
    owned: &[i64],
    description: String,
    today: NaiveDate,
) -> Result<JournalEntryWithLines, DbError> {
    // --- 1. La société, en partagé. -------------------------------------
    let company: Option<(Option<NaiveDate>,)> = sqlx::query_as(
        "SELECT books_locked_through FROM companies WHERE id = ? LOCK IN SHARE MODE",
    )
    .bind(company_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;
    let Some((locked_through,)) = company else {
        return Err(DbError::NotFound);
    };

    // --- 2. Les comptes. -------------------------------------------------
    let mut ids: Vec<i64> = lines.iter().map(|l| l.account_id).collect();
    ids.sort_unstable();
    ids.dedup();
    // Verrou par clé primaire sur les SEULS comptes de la société (cf.
    // `owned_account_ids`) ; le filtre `company_id` reste en défense.
    let locked: Vec<LockedAccount> = if owned.is_empty() {
        Vec::new()
    } else {
        let mut qb: QueryBuilder<MySql> = QueryBuilder::new(
            "SELECT id, number, account_type, active, postable, role FROM accounts WHERE id IN (",
        );
        {
            let mut sep = qb.separated(", ");
            for id in owned {
                sep.push_bind(*id);
            }
        }
        qb.push(") AND company_id = ")
            .push_bind(company_id)
            .push(" ORDER BY id FOR UPDATE");
        qb.build_query_as()
            .fetch_all(&mut **tx)
            .await
            .map_err(map_db_error)?
    };

    let retained: Option<RetainedEarningsAccount> =
        sqlx::query_as(&format!("{RETAINED_SQL} LOCK IN SHARE MODE"))
            .bind(company_id)
            .bind(RETAINED_EARNINGS_ROLE)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_db_error)?;

    // --- 3. Les exercices, une ligne chacun. ------------------------------
    let first: Option<FyRow> = sqlx::query_as(&format!("{FIRST_FY_SQL} FOR UPDATE"))
        .bind(company_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?;
    let candidate: Option<FyRow> = sqlx::query_as(&format!("{CANDIDATE_FY_SQL} FOR UPDATE"))
        .bind(company_id)
        .bind(today)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?;

    // --- 4. Les lectures ordinaires — l'instantané s'ouvre ici. ----------
    let mut moved_qb: QueryBuilder<MySql> = QueryBuilder::new(
        "SELECT DISTINCT account_id FROM journal_entry_lines WHERE account_id IN (",
    );
    {
        let mut sep = moved_qb.separated(", ");
        for id in &ids {
            sep.push_bind(*id);
        }
    }
    moved_qb.push(")");
    let moved: Vec<i64> = moved_qb
        .build_query_scalar()
        .fetch_all(&mut **tx)
        .await
        .map_err(map_db_error)?;
    let entries = has_entries(&mut **tx, company_id).await?;

    // --- 5. Les refus, dans l'ordre de priorité. --------------------------
    let Some(first) = first.filter(|_| entries) else {
        return Err(refused(OpeningComplementRefusal::NoEntries, None));
    };
    let date = decide_date(&first, candidate.as_ref(), locked_through, today)
        .map_err(|r| refused(r, None))?;
    let retained = check_retained_earnings(retained.as_ref()).map_err(|r| refused(r, None))?;
    check_lines(&ids, &locked, &moved)?;

    // --- 6. La contrepartie. ---------------------------------------------
    let gap: Decimal = lines.iter().map(|l| l.debit - l.credit).sum();
    let mut entry_lines: Vec<JournalEntryLineDraft> = lines
        .iter()
        .map(|l| JournalEntryLineDraft {
            account_id: l.account_id,
            debit: Money::new(l.debit),
            credit: Money::new(l.credit),
            project_id: None,
        })
        .collect();
    if gap != Decimal::ZERO {
        let (debit, credit) = if gap > Decimal::ZERO {
            (Decimal::ZERO, gap)
        } else {
            (-gap, Decimal::ZERO)
        };
        entry_lines.push(JournalEntryLineDraft {
            account_id: retained.id,
            debit: Money::new(debit),
            credit: Money::new(credit),
            project_id: None,
        });
    }

    // --- 7. Validation métier, puis création. ----------------------------
    let draft = JournalEntryDraft {
        date: date.date,
        journal: CoreJournal::OD,
        description,
        lines: entry_lines,
    };
    // Équilibré par construction : un échec ici est un défaut du calcul
    // ci-dessus, pas une erreur de l'utilisateur.
    let validated = accounting::validate(draft)
        .map_err(|e| DbError::Invariant(format!("opening-complement:unbalanced:{e}")))?
        .into_draft();
    let new = NewJournalEntry {
        company_id,
        entry_date: validated.date,
        journal: Journal::from(validated.journal),
        description: validated.description,
        project_id: None,
        lines: validated
            .lines
            .into_iter()
            .map(|l| NewJournalEntryLine {
                account_id: l.account_id,
                debit: l.debit.amount(),
                credit: l.credit.amount(),
                project_id: None,
            })
            .collect(),
    };
    journal_entries::create_in_tx(tx, date.fiscal_year_id, user_id, new, true).await
}

/// Refus par compte, **par catégorie** dans l'ordre de priorité, puis le plus
/// petit `id` dans une catégorie (`ids` est trié).
fn check_lines(ids: &[i64], locked: &[LockedAccount], moved: &[i64]) -> Result<(), DbError> {
    let find = |id: i64| locked.iter().find(|a| a.id == id);
    // Le compte fautif est NOMMÉ par son numéro quand on le connaît (un
    // identifiant inconnu ou d'une autre société n'en a pas : on ne révèle rien).
    let named = |id: i64| Some((id, find(id).map(|a| a.number.clone())));

    let invalid = ids.iter().find(|id| match find(**id) {
        None => true,
        Some(a) => !a.active || !a.postable,
    });
    if let Some(id) = invalid {
        return Err(refused(
            OpeningComplementRefusal::AccountInvalid,
            named(*id),
        ));
    }
    let retained_line = ids
        .iter()
        .find(|id| find(**id).is_some_and(|a| a.role.as_deref() == Some(RETAINED_EARNINGS_ROLE)));
    if let Some(id) = retained_line {
        return Err(refused(
            OpeningComplementRefusal::RetainedEarningsLine,
            named(*id),
        ));
    }
    let not_balance = ids.iter().find(|id| {
        find(**id).is_some_and(|a| a.account_type != "Asset" && a.account_type != "Liability")
    });
    if let Some(id) = not_balance {
        return Err(refused(
            OpeningComplementRefusal::NotBalanceAccount,
            named(*id),
        ));
    }
    let moved_line = ids.iter().find(|id| moved.contains(id));
    if let Some(id) = moved_line {
        return Err(refused(OpeningComplementRefusal::AccountMoved, named(*id)));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fy(id: i64, start: (i32, u32, u32), end: (i32, u32, u32), status: &str) -> FyRow {
        FyRow {
            id,
            name: format!("FY{id}"),
            start_date: NaiveDate::from_ymd_opt(start.0, start.1, start.2).unwrap(),
            end_date: NaiveDate::from_ymd_opt(end.0, end.1, end.2).unwrap(),
            status: status.into(),
        }
    }

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn branche_a_premier_exercice_ouvert_non_verrouille() {
        let first = fy(1, (2026, 1, 1), (2026, 12, 31), "Open");
        let got = decide_date(&first, Some(&first), None, d(2026, 6, 1)).unwrap();
        assert_eq!(got.date, d(2026, 1, 1));
        assert_eq!(got.branch, ComplementDateBranch::OpeningDay);
    }

    /// Un premier exercice ouvert qui commence APRÈS `today` : la branche (a)
    /// date le complément de son premier jour — comme l'écriture d'ouverture,
    /// elle-même datée de ce jour (revue de code P1, B-F2 : comportement fixé).
    #[test]
    fn branche_a_premier_exercice_futur() {
        let first = fy(1, (2027, 1, 1), (2027, 12, 31), "Open");
        let got = decide_date(&first, None, None, d(2026, 6, 1)).unwrap();
        assert_eq!(
            (got.date, got.branch),
            (d(2027, 1, 1), ComplementDateBranch::OpeningDay)
        );
    }

    #[test]
    fn branche_b_premier_exercice_clos() {
        let first = fy(1, (2025, 1, 1), (2025, 12, 31), "Closed");
        let cur = fy(2, (2026, 1, 1), (2026, 12, 31), "Open");
        let got = decide_date(&first, Some(&cur), None, d(2026, 6, 1)).unwrap();
        assert_eq!((got.fiscal_year_id, got.date), (2, d(2026, 6, 1)));
        assert_eq!(got.branch, ComplementDateBranch::Today);
    }

    #[test]
    fn branche_b_premier_jour_verrouille() {
        let first = fy(1, (2026, 1, 1), (2026, 12, 31), "Open");
        let got = decide_date(&first, Some(&first), Some(d(2026, 1, 1)), d(2026, 6, 1)).unwrap();
        assert_eq!(
            (got.date, got.branch),
            (d(2026, 6, 1), ComplementDateBranch::Today)
        );
    }

    #[test]
    fn branche_c_aucun_exercice_ouvert_couvrant() {
        let first = fy(1, (2025, 1, 1), (2025, 12, 31), "Closed");
        // Le candidat a commencé mais est clos, puis : il est terminé.
        assert_eq!(
            decide_date(&first, Some(&first), None, d(2026, 6, 1)),
            Err(OpeningComplementRefusal::NoOpenFiscalYear)
        );
        assert_eq!(
            decide_date(&first, None, None, d(2026, 6, 1)),
            Err(OpeningComplementRefusal::NoOpenFiscalYear)
        );
    }

    #[test]
    fn branche_c_aujourd_hui_verrouille() {
        let first = fy(1, (2026, 1, 1), (2026, 12, 31), "Open");
        assert_eq!(
            decide_date(&first, Some(&first), Some(d(2026, 6, 1)), d(2026, 6, 1)),
            Err(OpeningComplementRefusal::DateLocked)
        );
    }

    #[test]
    fn l_ordre_des_variantes_est_la_priorite() {
        use OpeningComplementRefusal::*;
        let ordre = [
            NoEntries,
            NoOpenFiscalYear,
            DateLocked,
            NoRetainedEarnings,
            RetainedEarningsNotPostable,
            AccountInvalid,
            RetainedEarningsLine,
            NotBalanceAccount,
            AccountMoved,
        ];
        assert!(ordre.windows(2).all(|w| w[0] < w[1]));
    }

    fn acc(id: i64, ty: &str, role: Option<&str>) -> LockedAccount {
        LockedAccount {
            id,
            number: id.to_string(),
            account_type: ty.into(),
            active: true,
            postable: true,
            role: role.map(Into::into),
        }
    }

    fn reason(r: Result<(), DbError>) -> (OpeningComplementRefusal, Option<i64>) {
        match r {
            Err(DbError::OpeningComplementRefused {
                reason, account_id, ..
            }) => (reason, account_id),
            other => panic!("attendu un refus, obtenu {other:?}"),
        }
    }

    #[test]
    fn refus_par_compte_par_categorie_puis_plus_petit_id() {
        // 3 : mouvementé ; 5 : résultat ; 7 : inconnu. La catégorie
        // « invalide » prime, quel que soit l'id.
        let locked = [acc(3, "Asset", None), acc(5, "Revenue", None)];
        assert_eq!(
            reason(check_lines(&[3, 5, 7], &locked, &[3])),
            (OpeningComplementRefusal::AccountInvalid, Some(7))
        );
        assert_eq!(
            reason(check_lines(&[3, 5], &locked, &[3])),
            (OpeningComplementRefusal::NotBalanceAccount, Some(5))
        );
        assert_eq!(
            reason(check_lines(&[3], &locked, &[3])),
            (OpeningComplementRefusal::AccountMoved, Some(3))
        );
        let with_re = [
            acc(2, "Liability", Some("RetainedEarnings")),
            acc(5, "Revenue", None),
        ];
        assert_eq!(
            reason(check_lines(&[2, 5], &with_re, &[])),
            (OpeningComplementRefusal::RetainedEarningsLine, Some(2))
        );
        assert!(check_lines(&[3], &locked, &[]).is_ok());
    }
}
