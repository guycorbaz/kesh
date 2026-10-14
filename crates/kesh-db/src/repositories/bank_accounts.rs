//! Repository CRUD pour `BankAccount`.
//!
//! ## Sémantique du flag `archived` (Story v014-1, FINDING-6 Pass 3 Opus)
//!
//! - [`find_primary`] : filtre `archived = FALSE` (garantit que les routes
//!   PDF QR Bill ne retournent jamais un primary archivé).
//! - [`find_by_id_for_company`] : **ne filtre PAS** `archived` (contrat
//!   intentionnel — c'est aux call sites de mutation post-archivage
//!   d'inspecter `bank_account.archived` après le find).
//! - [`list_by_company`] : filtre `archived = FALSE` quand
//!   `include_archived=false` (défaut), retourne tout quand `true`
//!   (export ZIP souveraineté + toggle UI « afficher archivés »).
//! - [`set_journal_account_id_for_company`], [`update_for_company`],
//!   [`archive_for_company`] : SELECT FOR UPDATE filtre `archived = FALSE`
//!   → `DbError::NotFound` (anti-énumération KF-002) si row archivée.
//! - [`count_transactions_for_bank_account`] : utilisé par DELETE pour
//!   refuser 412 BANK_ACCOUNT_HAS_TRANSACTIONS si historique présent.
//!
//! ## Sémantique de `closed_on` — le compte clos (Story 28-4a, #642)
//!
//! Un compte **clos** (`closed_on` non nul) est en **lecture seule** : il reste
//! lisible partout (détail, export, sauvegarde, historique des imports), il
//! n'est plus proposé là où l'on écrit ([`list_by_company`] l'écarte par
//! défaut, [`BankAccountListFilter`]), et tout chemin d'écriture qui le vise
//! refuse en [`DbError::BankAccountClosed`]. Gestes : [`close_in_tx`],
//! [`reopen_in_tx`]. Gardes des autres dépôts : [`ensure_open_in_tx`],
//! [`ensure_open_unarchived_in_tx`] — leur doc-comment porte la règle de la
//! garde, sa règle de placement, les trois invariants et la fenêtre de la
//! clôture, fermée par la revérification ([`recheck_open_locked_in_tx`],
//! [`recheck_open_unarchived_locked_in_tx`] ; Story 29-4, #651).
//! [`set_journal_account_id_for_company`], [`update_for_company`] et
//! [`archive_for_company`] refusent un compte clos dans leur `FOR UPDATE`
//! existant, après leur contrôle de version.

use sqlx::mysql::MySqlPool;
use sqlx::{MySql, Transaction};

use crate::entities::bank_account::{BankAccount, NewBankAccount, bank_account_display_name};
use crate::errors::{DbError, map_db_error};

/// Colonnes de `bank_accounts` sélectionnées pour hydrater [`BankAccount`], sous
/// forme de littéral — un `macro_rules!` plutôt qu'un `const` pour rester
/// utilisable dans `concat!`, qui n'accepte que des littéraux (précédent :
/// `repositories/accounts.rs`).
///
/// Story 26-2a (#622, C-26-2-20) — **source unique** : chaque `SELECT` qui
/// hydrate un [`BankAccount`] s'écrit `concat!("SELECT ", columns!(), " FROM
/// bank_accounts …")`. Une colonne ajoutée à l'entité s'ajoute ici une fois ; un
/// `SELECT` qui l'oublierait ferait échouer `FromRow` à l'exécution. Aucune
/// liste de colonnes ne sort de ce module : la route relit par
/// [`find_by_id_in_tx`].
macro_rules! columns {
    () => {
        "id, company_id, bank_name, iban, qr_iban, is_primary, journal_account_id, version, archived, description, closed_on, created_at, updated_at"
    };
}

const FIND_BY_ID_SQL: &str = concat!("SELECT ", columns!(), " FROM bank_accounts WHERE id = ?");

/// Crée un nouveau compte bancaire et retourne l'entité persistée.
pub async fn create(pool: &MySqlPool, new: NewBankAccount) -> Result<BankAccount, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let result = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, qr_iban, is_primary) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(new.company_id)
    .bind(&new.bank_name)
    .bind(&new.iban)
    .bind(&new.qr_iban)
    .bind(new.is_primary)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let last_id = result.last_insert_id();
    if last_id == 0 {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::Invariant(
            "last_insert_id == 0 après INSERT bank_accounts".into(),
        ));
    }
    let id = i64::try_from(last_id)
        .map_err(|_| DbError::Invariant(format!("last_insert_id {last_id} dépasse i64::MAX")))?;

    let account = sqlx::query_as::<_, BankAccount>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| DbError::Invariant(format!("bank_account {id} introuvable après INSERT")))?;

    tx.commit().await.map_err(map_db_error)?;
    Ok(account)
}

/// Retourne le compte bancaire principal d'une company (ou None).
///
/// Story v014-1 (FINDING-1 Pass 3 Opus) — filtre `archived = FALSE` pour ne
/// pas retourner un primary archivé. Cross-fichier : utilisé par
/// `routes/invoice_pdf.rs` pour générer le QR Bill — sinon le PDF utiliserait
/// un IBAN archivé alors que le compte est invisible côté UI.
pub async fn find_primary(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Option<BankAccount>, DbError> {
    sqlx::query_as::<_, BankAccount>(
        concat!(
            "SELECT ",
            columns!(),
            " FROM bank_accounts WHERE company_id = ? AND is_primary = TRUE AND archived = FALSE LIMIT 1",
        ),
    )
    .bind(company_id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

/// Cherche un compte bancaire par id **scopé multi-tenant**.
///
/// Story 8-1b T6.3 (review code Pass 1 H5) : utilisé par le handler
/// `POST /bank-imports/preview` pour valider que le `bankAccountId`
/// fourni par le client appartient bien à la company courante (sinon
/// un attaquant pourrait passer un id appartenant à une autre company
/// et persister ses transactions sous ce bank_account — IDOR).
///
/// Renvoie `None` si le compte n'existe pas OU appartient à une autre
/// company (pas de leak d'existence cross-tenant — pattern KF-002).
///
/// **Story v014-1 (FINDING-6 Pass 3 Opus)** — la fonction ne filtre PAS
/// `archived` (contrat intentionnel). Les call sites doivent inspecter
/// `bank_account.archived` post-find et rejeter avec `BankAccountNotFound`
/// (anti-énumération KF-002) pour les mutations post-archivage.
pub async fn find_by_id_for_company(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
) -> Result<Option<BankAccount>, DbError> {
    sqlx::query_as::<_, BankAccount>(concat!(
        "SELECT ",
        columns!(),
        " FROM bank_accounts WHERE company_id = ? AND id = ? LIMIT 1",
    ))
    .bind(company_id)
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

/// Relecture d'un compte bancaire **scopée par société**, dans la transaction
/// de l'appelant — Story 26-2a (#622, C-26-2-20).
///
/// Sert à la route de création à relire la ligne qu'elle vient d'insérer, sans
/// qu'aucune liste de colonnes ne sorte de ce module (macro [`columns!`]). Ne
/// filtre pas `archived` (comme [`find_by_id_for_company`]).
///
/// # Erreurs
///
/// [`DbError::NotFound`] si le compte n'existe pas ou appartient à une autre
/// société.
pub async fn find_by_id_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
) -> Result<BankAccount, DbError> {
    sqlx::query_as::<_, BankAccount>(concat!(
        "SELECT ",
        columns!(),
        " FROM bank_accounts WHERE company_id = ? AND id = ?"
    ))
    .bind(company_id)
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?
    .ok_or(DbError::NotFound)
}

/// Filtre de [`list_by_company`] — Story 28-4a (#642).
///
/// `Default` = les deux à `false` : comptes **ni archivés ni clos**, ceux où
/// l'on peut écrire (sélecteurs d'import, de réconciliation, de règlement, de
/// lot). `include_closed` ajoute les clos non archivés ; `include_archived`
/// ajoute les archivés, **clos ou non**.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BankAccountListFilter {
    /// Ajoute les comptes archivés (clos ou non).
    pub include_archived: bool,
    /// Ajoute les comptes clos non archivés.
    pub include_closed: bool,
}

impl BankAccountListFilter {
    /// Tous les comptes, archivés et clos compris (export global).
    pub const ALL: Self = Self {
        include_archived: true,
        include_closed: true,
    };
}

/// Liste les comptes bancaires d'une company.
///
/// Story v014-1, Story 28-4a — filtre [`BankAccountListFilter`] :
/// - défaut (handler GET, `/companies/current`) : ni archivés ni clos ;
/// - `include_closed` : + les clos non archivés (écran de gestion, réglages
///   de facturation) ;
/// - `include_archived` : + les archivés, clos ou non (toggle UI « afficher
///   archivés ») ; les deux : **tous** (export ZIP souveraineté).
pub async fn list_by_company(
    pool: &MySqlPool,
    company_id: i64,
    filter: BankAccountListFilter,
) -> Result<Vec<BankAccount>, DbError> {
    // (archivé ⇒ inclus si include_archived) ; (non archivé ⇒ ouvert, ou clos
    // si include_closed).
    let sql = match (filter.include_archived, filter.include_closed) {
        (true, true) => concat!(
            "SELECT ",
            columns!(),
            " FROM bank_accounts WHERE company_id = ? ORDER BY is_primary DESC, id",
        ),
        (true, false) => concat!(
            "SELECT ",
            columns!(),
            " FROM bank_accounts WHERE company_id = ? AND (archived = TRUE OR closed_on IS NULL) ORDER BY is_primary DESC, id",
        ),
        (false, true) => concat!(
            "SELECT ",
            columns!(),
            " FROM bank_accounts WHERE company_id = ? AND archived = FALSE ORDER BY is_primary DESC, id",
        ),
        (false, false) => concat!(
            "SELECT ",
            columns!(),
            " FROM bank_accounts WHERE company_id = ? AND archived = FALSE AND closed_on IS NULL ORDER BY is_primary DESC, id",
        ),
    };
    sqlx::query_as::<_, BankAccount>(sql)
        .bind(company_id)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)
}

/// Compare le compte existant au payload — `true` si aucun champ métier ne diffère
/// (KF-004 : court-circuit no-op pour ne pas bumper version inutilement).
/// Compare uniquement les champs effectivement écrits par l'UPDATE de
/// `upsert_primary` (`bank_name`, `iban`, `qr_iban`).
fn is_no_op_change(existing: &BankAccount, new: &NewBankAccount) -> bool {
    existing.bank_name == new.bank_name
        && existing.iban == new.iban
        && existing.qr_iban == new.qr_iban
}

/// Issue de [`upsert_primary_in_tx`] : ce que l'upsert a **réellement** fait
/// (Story 15-7a1). Permet à l'appelant de tracer la création, la modification
/// (avec l'état d'avant) ou rien du tout.
#[derive(Debug, Clone)]
pub enum UpsertPrimaryOutcome {
    /// Aucun compte principal actif : un compte a été inséré.
    Created(BankAccount),
    /// Le compte principal existant a été modifié ; `after.version = before.version + 1`.
    Updated {
        /// L'état lu sous verrou, avant l'`UPDATE`.
        before: BankAccount,
        /// L'état relu après l'`UPDATE`.
        after: BankAccount,
    },
    /// Le payload est identique au compte existant (court-circuit no-op
    /// KF-004) : rien n'a été écrit, `version` est inchangée.
    Unchanged(BankAccount),
}

impl UpsertPrimaryOutcome {
    /// Le compte tel qu'il est après l'upsert (`Created`, `after` ou `Unchanged`).
    pub fn into_account(self) -> BankAccount {
        match self {
            UpsertPrimaryOutcome::Created(a) | UpsertPrimaryOutcome::Unchanged(a) => a,
            UpsertPrimaryOutcome::Updated { after, .. } => after,
        }
    }
}

/// Upsert du compte bancaire principal (idempotent pour retries).
///
/// Enveloppe pool de [`upsert_primary_in_tx`] (Story 15-7a1), projetée sur
/// `BankAccount` (le compte de `Created`, `after` ou `Unchanged`) :
/// - `Created` / `Updated` → `commit` ;
/// - `Unchanged` → `rollback` (« rien n'a été modifié » ; le `SELECT … FOR
///   UPDATE` tient un X-lock que `rollback` libère comme `commit`) ;
/// - erreur (dont `OptimisticLockConflict`) → rollback *best-effort*, puis
///   l'erreur d'origine.
pub async fn upsert_primary(pool: &MySqlPool, new: NewBankAccount) -> Result<BankAccount, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    match upsert_primary_in_tx(&mut tx, new).await {
        Ok(UpsertPrimaryOutcome::Unchanged(account)) => {
            tx.rollback().await.map_err(map_db_error)?;
            Ok(account)
        }
        Ok(outcome) => {
            tx.commit().await.map_err(map_db_error)?;
            Ok(outcome.into_account())
        }
        Err(e) => {
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

/// Variante transactionnelle de [`upsert_primary`] (Story 15-7a1) : upsert du
/// compte bancaire principal **dans la transaction de l'appelant**.
///
/// `SELECT … FOR UPDATE` sur le compte principal actif, puis :
/// - absent → `INSERT`, rend [`UpsertPrimaryOutcome::Created`] ;
/// - présent et payload identique (KF-004) → rend
///   [`UpsertPrimaryOutcome::Unchanged`] **sans annuler** : « inchangé »
///   n'annule rien, l'appelant doit pouvoir poursuivre sa transaction ;
/// - présent et différent → `UPDATE` sous verrou optimiste, rend
///   [`UpsertPrimaryOutcome::Updated`] ; 0 ligne touchée →
///   `Err(OptimisticLockConflict)`, **sans annuler**.
///
/// **Ne commite ni n'annule jamais** : c'est l'appelant qui décide.
pub async fn upsert_primary_in_tx(
    tx: &mut Transaction<'_, MySql>,
    new: NewBankAccount,
) -> Result<UpsertPrimaryOutcome, DbError> {
    let existing = sqlx::query_as::<_, BankAccount>(
        concat!(
            "SELECT ",
            columns!(),
            " FROM bank_accounts WHERE company_id = ? AND is_primary = TRUE AND archived = FALSE LIMIT 1 FOR UPDATE",
        ),
    )
    .bind(new.company_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;

    match existing {
        Some(account) => {
            // KF-004 : court-circuit no-op AVANT toute mutation.
            if is_no_op_change(&account, &new) {
                return Ok(UpsertPrimaryOutcome::Unchanged(account));
            }

            let rows = sqlx::query(
                "UPDATE bank_accounts SET bank_name = ?, iban = ?, qr_iban = ?, version = version + 1 \
                 WHERE id = ? AND version = ?",
            )
            .bind(&new.bank_name)
            .bind(&new.iban)
            .bind(&new.qr_iban)
            .bind(account.id)
            .bind(account.version)
            .execute(&mut **tx)
            .await
            .map_err(map_db_error)?
            .rows_affected();

            if rows == 0 {
                return Err(DbError::OptimisticLockConflict);
            }

            let updated = sqlx::query_as::<_, BankAccount>(FIND_BY_ID_SQL)
                .bind(account.id)
                .fetch_one(&mut **tx)
                .await
                .map_err(map_db_error)?;

            Ok(UpsertPrimaryOutcome::Updated {
                before: account,
                after: updated,
            })
        }
        None => {
            let result = sqlx::query(
                "INSERT INTO bank_accounts (company_id, bank_name, iban, qr_iban, is_primary) \
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(new.company_id)
            .bind(&new.bank_name)
            .bind(&new.iban)
            .bind(&new.qr_iban)
            .bind(new.is_primary)
            .execute(&mut **tx)
            .await
            .map_err(map_db_error)?;

            let id = i64::try_from(result.last_insert_id())
                .map_err(|_| DbError::Invariant("last_insert_id overflow".into()))?;

            let account = sqlx::query_as::<_, BankAccount>(FIND_BY_ID_SQL)
                .bind(id)
                .fetch_one(&mut **tx)
                .await
                .map_err(map_db_error)?;

            Ok(UpsertPrimaryOutcome::Created(account))
        }
    }
}

/// Met à jour le `journal_account_id` d'un bank_account scopé multi-tenant
/// **dans une transaction fournie par le caller**.
///
/// Story 8-5a-zero — pose le pattern `bank_account.journal_account_id` qui sera
/// consommé par 8-5a-base (manual match) et 8-5a-bis (split) sans body field
/// `bankLedgerAccountId` (résolu serveur-side via cette colonne).
///
/// **Pass 3 Opus 4.7 — F1''' fix** : la fonction prend `&mut Transaction<MySql>`
/// au lieu d'ouvrir sa propre transaction. Cela permet au handler de partager
/// la tx avec `audit_log::insert_in_tx` et de garantir l'atomicité UPDATE +
/// audit (pattern Story 3-5 + 7-3 + 8-4 — audit_log écrit depuis le route
/// handler, jamais depuis le repo).
///
/// **Pass 1 code review Sonnet 4.6 — P-C1** : retourne `(updated, before)`
/// atomiquement. Le caller utilise `before` comme source `before` de
/// l'audit_log (pas un SELECT séparé hors-FOR UPDATE qui ouvrirait une
/// fenêtre TOCTOU avec un SELECT FOR UPDATE concurrent).
///
/// **Pass 1 code review Sonnet 4.6 — P-H2** : la version est validée AVANT
/// le court-circuit no-op. Un client avec version stale obtient
/// `OptimisticLockConflict` même si `journal_account_id` ne change pas
/// (pas de 200 OK silencieux sur version périmée).
///
/// **Story v014-1 (FINDING-2 Pass 3 Opus)** — le SELECT FOR UPDATE filtre
/// `archived = FALSE` ; une row archivée → `DbError::NotFound` (handler
/// retourne `AppError::BankAccountNotFound` anti-énumération). Un compte
/// archivé est immuable hors `un-archive` workflow (L1 v0.1 — pas de
/// restoration UI).
///
/// **Ordre des erreurs** (Story 15-5b, AC12 ; Story 15-6c, AC6) :
/// `DbError::NotFound` → `DbError::OptimisticLockConflict` → court-circuit
/// no-op (valeur inchangée, rien n'est contrôlé) → `DbError::AccountsNotPostable`
/// (nouveau compte lié actif et non imputable) →
/// `DbError::BankAccountLedgerIsClaimAccount` (nouveau compte lié = un compte
/// de créance de `claims`).
///
/// `claims` (Story 15-6c, AC4) : les comptes de créance lus par l'appelant
/// **sous verrou partagé, avant** le `FOR UPDATE` de la ligne
/// (`company_invoice_settings::claim_accounts_in_share_mode`) ;
/// [`ClaimAccounts::default`](super::company_invoice_settings::ClaimAccounts)
/// quand la cible est `NULL` (dé-liaison : ni lecture ni verrou).
///
/// ⚠️ **Cycles** (Story 29-4a, #651) : la demande exclusive de la ligne, en
/// attente derrière un détenteur partagé (un import ou un lot après son
/// `INSERT`), fait attendre derrière elle les demandes **neuves** de la ligne
/// — l'exclusif d'une route de gestion qui tient la sentinelle `companies`
/// que l'écrivain attend (cycle `I → M → PATCH → I`), et, à partir de la
/// 29-4b, la revérification d'une annulation ; la revérification de l'import
/// et du lot, redemande d'un partagé tenu, n'y attend jamais (T29a-6,
/// T29a-7). Le `PATCH`, qui n'a rien écrit, peut être la victime d'un tel
/// cycle par la file d'attente. La route est rejouée (`bank_accounts::patch`).
pub async fn set_journal_account_id_for_company(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
    journal_account_id: Option<i64>,
    expected_version: i32,
    claims: &super::company_invoice_settings::ClaimAccounts,
) -> Result<(BankAccount, BankAccount), DbError> {
    // SELECT FOR UPDATE scopé multi-tenant + verrou X sur la row.
    // Le `existing` retourné sert également de source `before` pour
    // l'audit_log côté handler (P-C1 : pas de SELECT séparé qui ouvrirait
    // une fenêtre TOCTOU).
    let existing = sqlx::query_as::<_, BankAccount>(concat!(
        "SELECT ",
        columns!(),
        " FROM bank_accounts WHERE company_id = ? AND id = ? AND archived = FALSE FOR UPDATE",
    ))
    .bind(company_id)
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let existing = match existing {
        Some(b) => b,
        None => return Err(DbError::NotFound),
    };

    // P-H2 : validation optimistic lock AVANT court-circuit no-op. Un client
    // avec version stale doit obtenir `OptimisticLockConflict` même si la
    // valeur cible (`journal_account_id`) coïncide avec l'état persisté.
    // Sinon il pourrait croire son écriture acceptée alors qu'une mutation
    // concurrente a entre-temps changé la row puis l'a remise à la même
    // valeur — état invisible côté client.
    if existing.version != expected_version {
        return Err(DbError::OptimisticLockConflict);
    }

    // Story 28-4a (#642, AC5) — un compte clos refuse ses propres
    // modifications, lu sur la ligne que ce `FOR UPDATE` verrouille déjà
    // (aucun verrou ajouté). AVANT le court-circuit no-op : un `PATCH`
    // identique sur un compte clos rend `409`, non `200`.
    if existing.is_closed() {
        return Err(DbError::BankAccountClosed {
            bank_account_id: id,
        });
    }

    // KF-004 court-circuit no-op : pas de bump version, pas d'audit_log côté
    // handler. Le caller (handler) doit checker `before.version ==
    // updated.version` pour détecter le no-op et skipper
    // `audit_log::insert_in_tx`.
    if existing.journal_account_id == journal_account_id {
        return Ok((existing.clone(), existing));
    }

    // Story 15-5b (AC12, choix C10) — postabilité du nouveau compte lié, sous
    // le verrou de la ligne et seulement si la valeur change (court-circuit
    // no-op ci-dessus) : un compte lié devenu non imputable après coup reste
    // accepté tel quel (D-A0).
    if let Some(account_id) = journal_account_id {
        super::accounts::ensure_postable_if_active_in_tx(tx, company_id, account_id).await?;
        // Story 15-6c (AC4) — puis le compte de créance, au même endroit et
        // sous la même exemption « inchangé ».
        refuse_if_ledger_is_claim_account(tx, company_id, account_id, claims).await?;
    }

    // M4 defense-in-depth : ajout `AND company_id = ?` au scope de l'UPDATE.
    let rows = sqlx::query(
        "UPDATE bank_accounts SET journal_account_id = ?, version = version + 1 \
         WHERE id = ? AND company_id = ? AND version = ?",
    )
    .bind(journal_account_id)
    .bind(id)
    .bind(company_id)
    .bind(expected_version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        return Err(DbError::OptimisticLockConflict);
    }

    let updated = sqlx::query_as::<_, BankAccount>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_db_error)?;

    // NOTE : pas de tx.commit() ici — c'est le caller (route handler) qui
    // commit après avoir écrit l'audit_log dans la même tx.
    Ok((updated, existing))
}

/// Les champs que le `PUT` écrit **hors** de [`NewBankAccount`] — Story 26-2a
/// (#622, C-26-2-27).
///
/// Regroupés pour que [`update_for_company`] garde sept paramètres
/// (`clippy::too_many_arguments`) sans élargir `NewBankAccount`, que
/// l'onboarding et la création par le dépôt partagent et qui ne connaissent
/// pas la description (C-26-2-7).
#[derive(Debug, Clone, Copy, Default)]
pub struct BankAccountPutFields<'a> {
    /// Compte du plan comptable lié (`None` = délier).
    pub journal_account_id: Option<i64>,
    /// Description **déjà normalisée** par la route (`None` = effacée :
    /// remplacement complet, comme `qr_iban`).
    pub description: Option<&'a str>,
}

/// Met à jour TOUS les champs métier d'un bank_account (PUT v014-1).
///
/// Différent de [`set_journal_account_id_for_company`] qui ne touche que
/// `journal_account_id` (PATCH legacy 8-5a-zero) : ce helper écrit
/// `bank_name`, `iban`, `qr_iban`, `is_primary`, et, depuis `fields`
/// ([`BankAccountPutFields`]), `journal_account_id` et `description` (Story
/// 26-2a). **Remplacement complet** : une description `None` efface celle en
/// base.
///
/// La transition `is_primary` (un compte devient primary alors qu'un autre
/// l'était) est **déléguée au caller** : ce repo flippe simplement
/// `is_primary` en accord avec le payload. Le caller doit appeler
/// [`flip_primary_off_for_company`] sur l'ancien primary AVANT (ou APRÈS,
/// indifférent dans la même tx) pour préserver l'invariant « au plus 1
/// primary par company ».
///
/// Filtre `archived = FALSE` au SELECT FOR UPDATE (un PUT sur compte archivé
/// → 404 anti-énumération).
///
/// Retourne `(updated, before)` cohérent avec `set_journal_account_id_for_company`.
///
/// **Ordre des erreurs** (Story 15-5b, AC12 ; Story 15-6c, AC6) :
/// `DbError::NotFound` (compte bancaire inconnu, d'une autre société ou
/// archivé) → `DbError::OptimisticLockConflict` (version périmée) →
/// `DbError::AccountsNotPostable` (nouveau compte lié actif et non imputable,
/// contrôlé seulement s'il change) → `DbError::BankAccountLedgerIsClaimAccount`
/// (nouveau compte lié = un compte de créance de `claims`, même exemption). Un
/// refus abandonne la transaction du caller : une démotion de l'ancien
/// principal faite dans la même transaction est annulée avec elle. Un compte
/// bancaire **déjà** lié à un compte de créance (donnée antérieure) reste
/// modifiable dans ses autres champs.
///
/// `claims` : comme pour [`set_journal_account_id_for_company`] — lus par la
/// route sous verrou partagé juste après la sentinelle, `default()` quand la
/// cible est `NULL`.
pub async fn update_for_company(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
    new: &NewBankAccount,
    fields: BankAccountPutFields<'_>,
    expected_version: i32,
    claims: &super::company_invoice_settings::ClaimAccounts,
) -> Result<(BankAccount, BankAccount), DbError> {
    let new_journal_account_id = fields.journal_account_id;
    let existing = sqlx::query_as::<_, BankAccount>(concat!(
        "SELECT ",
        columns!(),
        " FROM bank_accounts WHERE company_id = ? AND id = ? AND archived = FALSE FOR UPDATE",
    ))
    .bind(company_id)
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let existing = match existing {
        Some(b) => b,
        None => return Err(DbError::NotFound),
    };

    if existing.version != expected_version {
        return Err(DbError::OptimisticLockConflict);
    }

    // Story 28-4a (#642, AC5) — compte clos : refus lu sous le verrou de la
    // ligne cible, déjà pris (aucun verrou ajouté). Une démotion de l'ancien
    // principal faite par la route dans la même transaction est annulée avec
    // elle (le handler rend `Err` sans commit).
    if existing.is_closed() {
        return Err(DbError::BankAccountClosed {
            bank_account_id: id,
        });
    }

    // Story 15-5b (AC12, choix C10) — postabilité du compte lié, sous le
    // verrou, **seulement s'il change** et n'est pas `NULL` : un PUT qui
    // renvoie le compte en place, devenu non imputable après coup, passe.
    if new_journal_account_id != existing.journal_account_id
        && let Some(account_id) = new_journal_account_id
    {
        super::accounts::ensure_postable_if_active_in_tx(tx, company_id, account_id).await?;
        // Story 15-6c (AC4) — puis le compte de créance, même exemption.
        refuse_if_ledger_is_claim_account(tx, company_id, account_id, claims).await?;
    }

    let rows = sqlx::query(
        "UPDATE bank_accounts SET bank_name = ?, iban = ?, qr_iban = ?, is_primary = ?, journal_account_id = ?, description = ?, version = version + 1 \
         WHERE id = ? AND company_id = ? AND version = ?",
    )
    .bind(&new.bank_name)
    .bind(&new.iban)
    .bind(&new.qr_iban)
    .bind(new.is_primary)
    .bind(new_journal_account_id)
    .bind(fields.description)
    .bind(id)
    .bind(company_id)
    .bind(expected_version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        return Err(DbError::OptimisticLockConflict);
    }

    let updated = sqlx::query_as::<_, BankAccount>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_db_error)?;

    Ok((updated, existing))
}

/// Soft-delete (archive) un bank_account.
///
/// Story v014-1 — passe `archived = TRUE` + bump version. Préserve audit +
/// traçabilité (pas de DELETE physique). Filtre `archived = FALSE` au SELECT
/// FOR UPDATE : un DELETE sur compte déjà archivé → 404 anti-énumération
/// (idempotence non-supportée v0.1, L1 pas de restoration UI).
pub async fn archive_for_company(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
    expected_version: i32,
) -> Result<(BankAccount, BankAccount), DbError> {
    let existing = sqlx::query_as::<_, BankAccount>(concat!(
        "SELECT ",
        columns!(),
        " FROM bank_accounts WHERE company_id = ? AND id = ? AND archived = FALSE FOR UPDATE",
    ))
    .bind(company_id)
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let existing = match existing {
        Some(b) => b,
        None => return Err(DbError::NotFound),
    };

    if existing.version != expected_version {
        return Err(DbError::OptimisticLockConflict);
    }

    // Story 28-4a (#642, AC5) — un compte clos ne s'archive pas (il faut le
    // rouvrir d'abord). La route refuse déjà au pré-vol ; ce refus-ci, lu sur
    // la ligne que ce `FOR UPDATE` verrouillait déjà, est celui d'une clôture
    // commitée entre le pré-vol et ce verrou. Aucun verrou ajouté.
    if existing.is_closed() {
        return Err(DbError::BankAccountClosed {
            bank_account_id: id,
        });
    }

    let rows = sqlx::query(
        "UPDATE bank_accounts SET archived = TRUE, version = version + 1 \
         WHERE id = ? AND company_id = ? AND version = ?",
    )
    .bind(id)
    .bind(company_id)
    .bind(expected_version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        return Err(DbError::OptimisticLockConflict);
    }

    let updated = sqlx::query_as::<_, BankAccount>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_db_error)?;

    Ok((updated, existing))
}

// ─────────────────────────────────────────────────────────────────────────────
// Story 28-4a (#642) — le compte clos : gardes, clôture, réouverture
// ─────────────────────────────────────────────────────────────────────────────

/// SQL de [`ensure_open_in_tx`] — **lecture cohérente non verrouillante**, ni
/// `FOR UPDATE`, ni `LOCK IN SHARE MODE` (T-a45 le contrôle). Ne lit **que**
/// `closed_on` : jamais `archived` (voir la fonction).
const ENSURE_OPEN_SQL: &str = "SELECT closed_on FROM bank_accounts WHERE company_id = ? AND id = ?";

/// SQL de [`ensure_open_unarchived_in_tx`] — même contrat, plus `archived`.
const ENSURE_OPEN_UNARCHIVED_SQL: &str =
    "SELECT closed_on, archived FROM bank_accounts WHERE company_id = ? AND id = ?";

/// Refuse un compte bancaire **clos** — la garde des annulations (W9, W24,
/// W25, W28, W29, W30), Story 28-4a (#642).
///
/// Rend [`DbError::BankAccountClosed`] si `closed_on` est non nul,
/// [`DbError::NotFound`] si le compte n'existe pas dans la société (inatteignable
/// par les appelants : la clé étrangère de la ligne qu'ils annulent le
/// garantit), `Ok(())` sinon.
///
/// # La règle de la garde (AC7)
///
/// ⛔ **Aucune garde ne prend de verrou** — critère décidable de l'AC7, en deux
/// volets : (1) *la story n'ajoute aucun verrou de ligne*, explicite (`FOR
/// UPDATE`, `LOCK IN SHARE MODE`, `FOR SHARE`) ou implicite par clé étrangère,
/// *à aucun chemin qui existait avant elle, et ne change l'ordre des verrous
/// d'aucun* ; (2) *elle ne déplace la naissance de la vue cohérente d'aucun
/// chemin*. Là où le chemin verrouille **déjà** la ligne du compte au sol
/// (`settle_invoice`, `supplier_invoices::pay_in_tx`, et les trois
/// modifications du compte de ce module), `closed_on` se lit **dans cette
/// instruction**, élargie de colonnes ; là où le chemin lit déjà la ligne sans
/// verrou (`create_batch`, `confirm_batch`, pré-vols de route), `closed_on`
/// s'ajoute à cette lecture ; ailleurs, la garde est **cette** fonction (ou
/// [`ensure_open_unarchived_in_tx`] pour l'import), lecture cohérente non
/// verrouillante, dans la transaction de l'écrivain. Interdit d'y ajouter un
/// verrou : une garde exclusive ouvrait des cycles neufs avec les routes de
/// gestion du compte (validation P2 de la fiche, C-28-4-21).
///
/// # La règle de placement (AC7)
///
/// ⛔ La garde se pose **après le dernier verrou que le chemin prend avant sa
/// première lecture ordinaire — ou, si le chemin n'en a aucune hors l'audit,
/// après le dernier verrou qu'il prend avant sa première écriture, et avant
/// cette écriture**. Sous `REPEATABLE READ`, la vue cohérente naît à la
/// première lecture non verrouillante : posée **avant** un verrou, la garde
/// figerait l'instantané avant l'attente de ce verrou, et le chemin relirait
/// ensuite un exercice « ouvert » qu'une clôture d'exercice concurrente vient
/// de fermer — une erreur comptable. Chaque site renvoie, en une ligne, au
/// doc-comment du sol qui impose sa position. Les sous-`SELECT` sur `users` de
/// l'`INSERT` d'`audit_log` ne comptent pas comme « première lecture
/// ordinaire » (dernière instruction, aucune décision n'en dépend).
///
/// # Invariants qui tiennent les écrivains sans garde
///
/// - **I1** — un compte clos ne porte aucune transaction `pending` (rejetées
///   comprises) : établi par [`close_in_tx`]. Producteurs : l'import (gardé) et
///   l'annulation de rapprochement (gardée). Il tient hors d'atteinte les
///   acceptations de propositions, le rejet, le rapprochement manuel et ventilé.
/// - **I2** — un compte clos ne porte aucun lot `generated` : établi par
///   [`close_in_tx`]. Seul producteur : `payment_batches::create_batch` (gardé).
/// - **I3** — un compte clos n'est jamais principal : **structurel**, contrainte
///   `chk_bank_accounts_closed_not_primary`.
///
/// # La fenêtre de la clôture — fermée par la revérification (Story 29-4, [#651](https://github.com/guycorbaz/kesh/issues/651))
///
/// La garde est non verrouillante : elle lit l'instantané de sa transaction.
/// Seule, elle laissait passer une écriture dont la transaction avait pris sa
/// **vue cohérente** avant le commit d'une clôture : rien ne sérialisait
/// l'écrivain et la clôture, qui ne prend que sa propre ligne en exclusif
/// (l'angle mort L1 de la Story 28-4, tracé par #651 jusqu'à la v0.13.3). Sept
/// chemins y étaient exposés ; chacun est désormais **revérifié** sous un
/// verrou partagé de la ligne du compte, qui entre en conflit avec l'exclusif
/// de la clôture — ou bien la clôture est commitée et la revérification la
/// voit, ou bien la revérification passe d'abord et la clôture attend le
/// commit de l'écrivain, puis voit ce qu'il a écrit :
///
/// - l'**import** (CAMT et CSV) et la **création de lot** (Story 29-4a) —
///   [`recheck_open_unarchived_locked_in_tx`], juste après leur `INSERT` à clé
///   étrangère : le partagé est déjà tenu, la redemande ne crée rien ;
/// - les **annulations** — de rapprochement, de règlement client et
///   fournisseur, d'une facture fournisseur payée — et la **suppression
///   d'un import** (vide : un import qui porte des transactions les a
///   `pending`, et la clôture refuse) (Story 29-4b) —
///   [`recheck_open_locked_in_tx`], en **dernière acquisition de verrou**,
///   après leur dernière écriture, avant l'audit.
///
/// La garde reste en tête de chaque chemin : elle donne le refus précoce et
/// l'ordre des refus de la 28-4 ; la revérification ne répond que dans la
/// course. Sans fenêtre **par invariant**, sans revérification : l'annulation
/// de rejet (la transaction rejetée est `pending`, la clôture refuse un compte
/// qui en porte — I1 ; troisième inventaire de
/// `bank_account_closed_inventory.rs`) ; les acceptations de propositions, le
/// rejet, le rapprochement manuel et ventilé (même raison, I1). Sans fenêtre
/// **par verrou** : le règlement et le paiement fournisseur par virement, qui
/// lisent le compte sous `FOR UPDATE`. Les états qu'a pu laisser la fenêtre
/// jusqu'à la v0.13.3 (une transaction `pending` ou un lot `generated` sur un
/// compte clos) ne sont pas réparés d'office : rouvrir, traiter (rapprocher
/// ou supprimer l'import, confirmer ou annuler le lot — `cancel_batch` n'est
/// pas gardé), clore à nouveau.
///
/// ⛔ **Interdit** : faire lire `archived` à cette fonction, ou appeler
/// [`ensure_open_unarchived_in_tx`] depuis une annulation. Un compte archivé
/// peut porter des règlements et des lots, et **il ne se rouvre pas** :
/// refuser l'annulation d'un de ses règlements la rendrait définitivement
/// impossible.
///
/// # Angle mort assumé — un compte à la fois archivé et clos (revue P1, B-3)
///
/// Aucun geste ne produit cet état : la clôture refuse un archivé, l'archivage
/// refuse un clos, et les deux passent par le `FOR UPDATE` de la même ligne. Il
/// ne naît que d'une écriture **SQL directe**. Ses annulations rendent alors
/// `409 BANK_ACCOUNT_CLOSED`, et le compte ne se rouvre pas (la réouverture
/// filtre `archived = FALSE`) : impasse. Remède, à la main :
/// `UPDATE bank_accounts SET closed_on = NULL WHERE id = …`.
pub async fn ensure_open_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    bank_account_id: i64,
) -> Result<(), DbError> {
    let row: Option<(Option<chrono::NaiveDate>,)> = sqlx::query_as(ENSURE_OPEN_SQL)
        .bind(company_id)
        .bind(bank_account_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?;
    match row {
        None => Err(DbError::NotFound),
        Some((Some(_),)) => Err(DbError::BankAccountClosed { bank_account_id }),
        Some((None,)) => Ok(()),
    }
}

/// Refuse un compte bancaire **archivé** ([`DbError::NotFound`], l'erreur que
/// rend un compte inexistant) puis **clos** ([`DbError::BankAccountClosed`]) —
/// la garde de l'import (`bank_imports::create_with_transactions`), Story
/// 28-4a (#642, AC6-bis).
///
/// Même contrat que [`ensure_open_in_tx`] — lecture cohérente **non
/// verrouillante**, règle de la garde, règle de placement, fenêtre de la
/// clôture fermée par la revérification (#651) : voir sa documentation. L'appelant traduit `NotFound` en `404
/// BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` (`import_create_error` de la route
/// d'import). La course archivage / import (L5 de la route d'archivage), que
/// cette lecture non verrouillante ne ferme pas, est fermée par la
/// revérification de l'import, [`recheck_open_unarchived_locked_in_tx`]
/// (Story 29-4).
///
/// ⛔ **Interdit** : l'appeler depuis une annulation (un archivé ne se rouvre
/// pas), ou y ajouter un verrou.
pub async fn ensure_open_unarchived_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    bank_account_id: i64,
) -> Result<(), DbError> {
    let row: Option<(Option<chrono::NaiveDate>, bool)> = sqlx::query_as(ENSURE_OPEN_UNARCHIVED_SQL)
        .bind(company_id)
        .bind(bank_account_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?;
    match row {
        None | Some((_, true)) => Err(DbError::NotFound),
        Some((Some(_), false)) => Err(DbError::BankAccountClosed { bank_account_id }),
        Some((None, false)) => Ok(()),
    }
}

/// SQL de [`recheck_open_locked_in_tx`] — **lecture verrouillante partagée**
/// par égalité sur la clé primaire : verrou d'enregistrement partagé, sans
/// intervalle — celui-là même que pose un `INSERT` à clé étrangère vers
/// `bank_accounts(id)`. Ne lit **que** `closed_on`, jamais `archived`.
/// ⛔ `LOCK IN SHARE MODE`, jamais `FOR SHARE` (erreur de syntaxe sur MariaDB
/// 10.11) ni `FOR UPDATE` (T-a45 le contrôle ; `FOR SHARE` ferait aussi
/// échouer l'`EXPLAIN` de T29a-1, qui ne voit pas `FOR UPDATE`). Publique et cachée
/// pour que les tests de `tests/` la lisent (`EXPLAIN`).
#[doc(hidden)]
pub const RECHECK_OPEN_LOCKED_SQL: &str =
    "SELECT closed_on FROM bank_accounts WHERE id = ? AND company_id = ? LOCK IN SHARE MODE";

/// SQL de [`recheck_open_unarchived_locked_in_tx`] — même verrou, plus
/// `archived`.
#[doc(hidden)]
pub const RECHECK_OPEN_UNARCHIVED_LOCKED_SQL: &str = "SELECT closed_on, archived FROM bank_accounts WHERE id = ? AND company_id = ? LOCK IN SHARE MODE";

/// **Revérifie**, sous verrou partagé, qu'un compte bancaire n'est pas clos —
/// Story 29-4 (#651). Rend [`DbError::BankAccountClosed`] si `closed_on` est
/// non nul, [`DbError::NotFound`] si le compte n'existe pas dans la société,
/// `Ok(())` sinon. La revérification des **annulations** (W9, W24, W25, W28,
/// W29), qui l'appellent depuis la Story 29-4b, en dernière acquisition de
/// verrou ; l'import et la création de lot appellent la variante
/// [`recheck_open_unarchived_locked_in_tx`].
///
/// # Pourquoi une lecture verrouillante (C-29-4-1, C-29-4-2)
///
/// La garde de tête ([`ensure_open_in_tx`]) lit l'instantané de la
/// transaction : une clôture commitée après la naissance de cette vue ne s'y
/// voit pas, et la clôture, qui ne prend que sa propre ligne en exclusif, ne
/// rencontrait aucun verrou de l'écrivain — rien ne les sérialisait (L1 de la
/// Story 28-4, jusqu'à la v0.13.3). Une lecture **verrouillante** lit le dernier état commité, et
/// son verrou **partagé** sur la ligne du compte entre en conflit avec
/// l'exclusif de la clôture : ou bien la clôture est commitée, et la
/// revérification la voit ; ou bien la revérification passe d'abord, et la
/// clôture attend le commit de l'écrivain, puis voit ce qu'il a écrit. Partagé,
/// non exclusif : deux écrivains ne se gênent pas entre eux. La garde de tête
/// **reste** : elle donne le refus précoce et l'ordre des refus de la 28-4.
///
/// # La règle de position (critère de la story, volet 1)
///
/// ⛔ Le verrou que pose la revérification est **déjà tenu** par la transaction
/// au point où il est demandé — l'import et la création de lot l'appellent
/// juste après leur `INSERT` à clé étrangère vers le compte, qui a posé ce
/// partagé : InnoDB le trouve tenu, et la redemande passe **sans attente**,
/// même derrière un exclusif en file (T29a-6, T29a-7) — **ou bien** (les
/// annulations, à partir de la 29-4b) il est la **dernière acquisition de
/// verrou** de la transaction : seules le suivent l'insertion d'audit (`audit_log`, sans clé
/// étrangère) et des lectures non verrouillantes — à une borne près : les
/// deux sous-`SELECT` sur `users` de l'`INSERT` d'audit, dont le caractère
/// verrouillant n'est pas établi, inoffensifs puisque aucun chemin ne tient
/// `users` en exclusif en attendant `bank_accounts`. ⛔ **Interdit** de la poser
/// **avant** une écriture qui prend un autre verrou : elle y serait une demande
/// **neuve**, qui attend derrière un exclusif en file, et, pour une annulation,
/// le compte serait tenu avant `companies` (clé étrangère des écritures), dans
/// l'ordre inverse des routes de gestion du compte — le cycle que la Story
/// 28-4 a écarté. Pour l'import et le lot, l'argument d'ordre ne distingue
/// rien : leur `INSERT` prend déjà le compte avant `companies` (revue de code
/// P1, E-1) ; ce qui fixe leur position, c'est la redemande d'un verrou déjà
/// tenu. Posée après la première lecture non verrouillante du chemin, elle ne
/// déplace aucune naissance de vue (volet 3).
///
/// # Les cycles, et l'argument de poids (raisonné)
///
/// Une demande **neuve** attend derrière une demande exclusive **en attente**
/// sur la même ligne — non la redemande d'un partagé déjà tenu, qui passe
/// (T29a-6, T29a-7) : la revérification de l'import et du lot ne fait donc
/// jamais la queue. Le cycle atteignable en 29-4a est **préexistant** : un
/// écrivain `I` (import ou lot) tient la ligne du compte en partagé par le
/// premier contrôle de clé étrangère de son `INSERT` et demande `companies` en
/// partagé (le second ; ordre supposé, ci-dessous) ; une route de gestion `M` (`PUT`, archivage, `POST`
/// quand il démote le principal) tient la sentinelle `companies` en exclusif
/// et demande la ligne du compte en exclusif. À deux, `I ↔ M` ; à trois, quand la clôture, la réouverture ou
/// le `PATCH` est en file sur la ligne entre `I` et `M` : `I → M → E → I`
/// (T29a-4 et T29a-9 en jouent la forme, la transaction `t2` y tenant le rôle
/// de `M`). Depuis la 29-4b s'y ajoute la revérification d'une annulation,
/// demande partagée **neuve**, prise en dernier — après `companies` (clé
/// étrangère de la contre-passation), dans l'ordre des routes de gestion, donc
/// sans cycle avec elles ; et après l'exercice, d'où le cycle **neuf** compte
/// → exercice contre exercice → compte avec un détenteur de l'exclusif du
/// compte qui demande ensuite l'exercice (`settle_invoice`, `pay_in_tx`),
/// rejoué des deux côtés (`invoices::settle`, `supplier_invoices::pay`,
/// `payment_batches::confirm` ; les cinq annulations le sont toutes) —
/// **mesuré** une fois, pour l'annulation d'un règlement par virement, par
/// T29b-3 (`rejeu_interblocage_e2e.rs`) : `1213`, l'annulation victime, puis
/// rejouée. Tous les participants autres que l'écrivain sont
/// rejoués sur interblocage (`retry_app_on_deadlock`, routes
/// `bank_accounts::close`, `::reopen`, `::patch`, et — revue de code P1, E-1 —
/// `::create`, `::update`, `::archive`). ⚠️ *Mesuré et supposé* (Story 29-4b,
/// T6-bis, R-4) : **mesuré** au dépôt — la redemande sans attente (T29a-6,
/// T29a-7), la victime des cycles à trois par la file (T29a-4, T29a-9), le
/// cycle à deux `PUT` / archivage contre une ligne de compte tenue (T29a-13,
/// T29a-14). **Raisonné, non mesuré** : que l'`INSERT` d'`I` contrôle sa clé
/// étrangère vers le compte **avant** celle vers `companies` (l'ordre de
/// parcours des clés d'InnoDB ; le squash des tests et les migrations ne les
/// déclarent pas dans le même ordre) ; et l'exposition du `POST` — sa démotion
/// (`flip_primary_off_for_company`) balaie l'index `fk_bank_accounts_company`,
/// et son exclusif peut couvrir d'autres comptes de la société que l'ancien
/// principal (aucun `EXPLAIN` au dépôt ; T29a-15 joue un cycle synthétique,
/// non la démotion). InnoDB désigne pour victime
/// la transaction de moindre poids (écritures et verrous) : la clôture et la
/// réouverture, dont la demande est la première instruction, ne portent que
/// l'intention de table et leur demande en attente ; tout autre participant
/// hors route de gestion tient au moins un verrou d'enregistrement de plus.
/// ⚠️ Deux réserves : (1) le dépôt n'établit la désignation du plus léger que
/// pour des cycles **à deux** — qu'InnoDB compare tout le cycle est raisonné,
/// éprouvé par T29a-4 et T29a-9 ; (2) l'argument ne vaut pas contre une route
/// de gestion (`PUT`, `POST`, `DELETE /bank-accounts`) : en file sur la
/// sentinelle `companies`, elle ne tient aucun verrou d'enregistrement, à
/// égalité de poids avec la clôture et la réouverture, plus légère que le
/// `PATCH`, `settle_invoice` ou `pay_in_tx` — sans conséquence depuis qu'elle
/// est rejouée (revue de code P1, E-1) : quelle que soit la victime désignée
/// parmi elles, elle rejoue.
///
/// **Résidus** : (a) (29-4b) le cycle à trois où l'import (partagé sur le
/// compte depuis son `INSERT`, en attente de l'intervalle que tient une
/// suppression d'import qui revérifie) serait désigné victime au lieu du
/// demandeur d'exclusif — théorique : l'import et la suppression ont écrit, le
/// demandeur d'exclusif non (C-29-4-7) ; (b) **fermé** en revue de code P1
/// (E-1) — le cycle à quatre annulation → demandeur d'exclusif en file →
/// import (entre ses deux contrôles de clé étrangère) → route de gestion en
/// file sur la sentinelle → annulation, où la route de gestion, alors non
/// rejouée, pouvait être désignée et rendre `500` sans rien écrire : elle est
/// désormais rejouée, comme dans le cycle `I ↔ M` ci-dessus (C-29-4-14,
/// annoté).
///
/// # Angle mort — `innodb_snapshot_isolation`, non épinglé (#669)
///
/// « Lit le dernier état commité » vaut sous `innodb_snapshot_isolation =
/// OFF`, défaut de MariaDB 10.11 (CI, base de dev). À partir de MariaDB
/// **11.6**, le défaut passe à `ON` : une lecture verrouillante qui rencontre
/// une version de la ligne postérieure à la vue rend `ER_CHECKREAD` (1020),
/// que [`map_db_error`] ne traduit pas — non seulement dans la course que la
/// revérification traite (une clôture), mais après **tout** commit sur la
/// ligne du compte depuis la vue : `PATCH` du lien comptable, `PUT`, bascule du
/// principal, clôture puis réouverture. Un import ou un lot légitime,
/// concurrent d'un tel geste, rendrait alors `500`, rien d'écrit (raisonné ;
/// si le contrôle de clé étrangère de l'`INSERT` y est lui-même soumis, le
/// `1020` sort déjà au sol). Le réglage de session du pool réel, et ce rayon
/// élargi, sont portés par #669.
///
/// ⛔ **Interdit** : y faire lire `archived`, ou appeler
/// [`recheck_open_unarchived_locked_in_tx`] depuis une annulation. Un compte
/// archivé peut porter des règlements et des lots, et **il ne se rouvre
/// pas** : refuser l'annulation d'un de ses règlements la rendrait
/// définitivement impossible (même interdit qu'[`ensure_open_in_tx`]).
pub async fn recheck_open_locked_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    bank_account_id: i64,
) -> Result<(), DbError> {
    let row: Option<(Option<chrono::NaiveDate>,)> = sqlx::query_as(RECHECK_OPEN_LOCKED_SQL)
        .bind(bank_account_id)
        .bind(company_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?;
    match row {
        None => Err(DbError::NotFound),
        Some((Some(_),)) => Err(DbError::BankAccountClosed { bank_account_id }),
        Some((None,)) => Ok(()),
    }
}

/// **Revérifie**, sous verrou partagé, qu'un compte bancaire n'est ni
/// **archivé** ([`DbError::NotFound`], l'erreur d'un compte inexistant) ni
/// **clos** ([`DbError::BankAccountClosed`]) — dans l'ordre « archivé avant
/// clos » de la garde de tête [`ensure_open_unarchived_in_tx`] ; Story 29-4a
/// (#651, AC1). La revérification de l'**import**
/// (`bank_imports::create_with_transactions`, W22) et de la **création de lot**
/// (`payment_batches::create_batch`, W17), seuls chemins dont la garde de tête
/// lit `archived` : appelée juste après leur `INSERT` à clé étrangère, elle
/// redemande un partagé déjà tenu, sans verrou neuf. Elle ferme du même geste
/// la course archivage / import et archivage / lot : l'archivage prend
/// l'exclusif de la ligne (`archive_for_company`), que la revérification voit
/// comme elle voit une clôture.
///
/// Même contrat que [`recheck_open_locked_in_tx`] — pourquoi verrouillante,
/// règle de position, cycles et argument de poids, résidus, angle mort
/// `innodb_snapshot_isolation` (#669) : voir sa documentation. Lire `archived`
/// ne change ni le verrou ni les cycles : même égalité sur la clé primaire,
/// même verrou d'enregistrement.
///
/// ⛔ **Interdit** : l'appeler depuis une annulation (un archivé ne se rouvre
/// pas).
pub async fn recheck_open_unarchived_locked_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    bank_account_id: i64,
) -> Result<(), DbError> {
    let row: Option<(Option<chrono::NaiveDate>, bool)> =
        sqlx::query_as(RECHECK_OPEN_UNARCHIVED_LOCKED_SQL)
            .bind(bank_account_id)
            .bind(company_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_db_error)?;
    match row {
        None | Some((_, true)) => Err(DbError::NotFound),
        Some((Some(_), false)) => Err(DbError::BankAccountClosed { bank_account_id }),
        Some((None, false)) => Ok(()),
    }
}

/// Ce qui s'oppose à une clôture ou à une réouverture, hors version et
/// existence (qui sont des [`DbError`]) — Story 28-4a (#642).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClosureRefusal {
    /// Réouverture d'un compte qui n'est pas clos (`409 BANK_ACCOUNT_NOT_CLOSED`).
    NotClosed,
    /// Clôture du compte principal (`412 BANK_ACCOUNT_CANNOT_CLOSE_PRIMARY`).
    Primary,
    /// Transactions `pending`, rejetées comprises
    /// (`412 BANK_ACCOUNT_HAS_PENDING_TRANSACTIONS`).
    PendingTransactions { pending_count: i64 },
    /// Lots au statut `generated` (`412 BANK_ACCOUNT_HAS_OPEN_PAYMENT_BATCHES`).
    OpenPaymentBatches { batch_count: i64 },
}

/// Issue de [`close_in_tx`] et de [`reopen_in_tx`].
#[derive(Debug, Clone)]
pub enum ClosureOutcome {
    /// Le geste a été fait : `(after, before)`, comme les autres mutations
    /// (en `Box` : `clippy::large_enum_variant`).
    Done {
        /// L'état relu après l'`UPDATE`.
        after: Box<BankAccount>,
        /// L'état lu sous verrou, avant l'`UPDATE`.
        before: Box<BankAccount>,
    },
    /// Refusé, rien n'a été écrit.
    Refused(ClosureRefusal),
}

/// Ligne du compte, verrouillée en **première instruction** de la transaction
/// — partagé par [`close_in_tx`] et [`reopen_in_tx`]. Filtre `archived =
/// FALSE` : un archivé ne se clôt ni ne se rouvre (`NotFound`).
async fn lock_for_closure(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
    expected_version: i32,
) -> Result<BankAccount, DbError> {
    let existing = sqlx::query_as::<_, BankAccount>(concat!(
        "SELECT ",
        columns!(),
        " FROM bank_accounts WHERE company_id = ? AND id = ? AND archived = FALSE FOR UPDATE",
    ))
    .bind(company_id)
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?
    .ok_or(DbError::NotFound)?;
    if existing.version != expected_version {
        return Err(DbError::OptimisticLockConflict);
    }
    Ok(existing)
}

/// Écrit `closed_on` (une date, ou `NULL`) sous verrou optimiste et relit.
async fn write_closed_on(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
    expected_version: i32,
    closed_on: Option<chrono::NaiveDate>,
) -> Result<BankAccount, DbError> {
    let rows = sqlx::query(
        "UPDATE bank_accounts SET closed_on = ?, version = version + 1 \
         WHERE id = ? AND company_id = ? AND version = ?",
    )
    .bind(closed_on)
    .bind(id)
    .bind(company_id)
    .bind(expected_version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();
    if rows == 0 {
        return Err(DbError::OptimisticLockConflict);
    }
    sqlx::query_as::<_, BankAccount>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_db_error)
}

/// **Clôt** un compte bancaire — Story 28-4a (#642, AC3) : `closed_on =
/// <date>`, `version + 1`, **rien d'autre** (ni `archived`, ni
/// `journal_account_id`, ni le compte comptable lié). L'appelant trace
/// `bank_account.closed` dans la même transaction et commite.
///
/// Ordre des refus : [`DbError::NotFound`] (inexistant, autre société,
/// archivé) → [`DbError::OptimisticLockConflict`] → [`DbError::BankAccountClosed`]
/// (déjà clos) → [`ClosureRefusal::Primary`] → [`ClosureRefusal::PendingTransactions`]
/// → [`ClosureRefusal::OpenPaymentBatches`].
///
/// # Séquence — et pourquoi elle établit I1 et I2
///
/// `SELECT … FOR UPDATE` de la ligne du compte, **première instruction** de la
/// transaction (l'appelant ne lit rien avant), sans sentinelle `companies` ;
/// puis les deux décomptes en **lecture non verrouillante** ; puis l'`UPDATE`.
/// Sous `REPEATABLE READ`, la vue naît au premier décompte, **après** le verrou :
/// un écrivain qui a inséré une ligne rattachée au compte (import, lot,
/// règlement) tient, par la clé étrangère, un verrou partagé sur la ligne du
/// compte jusqu'à son commit — la clôture l'attend, puis ses décomptes voient
/// ce qu'il a commité (T-a31). Chemin **neuf** : son seul verrou est celui de
/// sa propre ligne, sans autre verrou tenu ni demandé ensuite (l'audit n'a
/// aucune clé étrangère) — **comme détenteur**, il ne peut fermer aucun cycle.
/// ⚠️ **En attente**, si (Story 29-4a, #651) : sa demande exclusive fait
/// attendre derrière elle toute demande **neuve** de la ligne — l'exclusif
/// d'une route de gestion (`PUT`, archivage) qui tient la sentinelle
/// `companies`, ou le contrôle de clé étrangère de l'`INSERT` d'un second
/// écrivain, et, à partir de la 29-4b, la revérification d'une annulation.
/// La revérification de l'import et du lot
/// ([`recheck_open_unarchived_locked_in_tx`]), redemande d'un partagé déjà
/// tenu, n'y attend jamais (T29a-6, T29a-7). Cycle atteignable en 29-4a : un
/// import ou un lot `I` tient la ligne en partagé par son `INSERT` et attend
/// `companies`, que tient une route de gestion `M` en file derrière la
/// clôture — `I → M → clôture → I`, dont la clôture, la plus légère, est la
/// victime ; d'où le rejeu de la route (`bank_accounts::close`).
///
/// ⚠️ La date est **informative** : ni bornée, ni contrôlée contre les dates
/// des transactions (C-28-4-3). Angle mort : la cohérence d'une sauvegarde
/// prise pendant la clôture relève de l'instantané commun de la sauvegarde
/// (Story 28-3, #633), qui la tient depuis — la sauvegarde voit l'état d'avant
/// ou d'après la clôture, jamais un état mêlé.
pub async fn close_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
    expected_version: i32,
    closed_on: chrono::NaiveDate,
) -> Result<ClosureOutcome, DbError> {
    let existing = lock_for_closure(tx, company_id, id, expected_version).await?;
    if existing.is_closed() {
        return Err(DbError::BankAccountClosed {
            bank_account_id: id,
        });
    }
    // I3 : la contrainte de la migration le rend structurel ; le refus est
    // nommé ici pour un message utile, avant les décomptes.
    if existing.is_primary {
        return Ok(ClosureOutcome::Refused(ClosureRefusal::Primary));
    }
    // I1 — lecture non verrouillante, APRÈS le verrou de la ligne (la vue
    // naît ici). Rejetées comprises : une rejetée est `pending`.
    let (pending_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM bank_transactions \
         WHERE company_id = ? AND bank_account_id = ? AND status = 'pending'",
    )
    .bind(company_id)
    .bind(id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;
    if pending_count > 0 {
        return Ok(ClosureOutcome::Refused(
            ClosureRefusal::PendingTransactions { pending_count },
        ));
    }
    // I2 — même vue.
    let (batch_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM payment_batches \
         WHERE company_id = ? AND bank_account_id = ? AND status = 'generated'",
    )
    .bind(company_id)
    .bind(id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;
    if batch_count > 0 {
        return Ok(ClosureOutcome::Refused(
            ClosureRefusal::OpenPaymentBatches { batch_count },
        ));
    }
    let after = write_closed_on(tx, company_id, id, expected_version, Some(closed_on)).await?;
    Ok(ClosureOutcome::Done {
        after: Box::new(after),
        before: Box::new(existing),
    })
}

/// **Rouvre** un compte bancaire clos — Story 28-4a (#642, AC4) : `closed_on
/// = NULL`, `version + 1`. Le compte **ne redevient pas principal** (il ne
/// l'était pas, I3). L'appelant trace `bank_account.reopened`.
///
/// Ordre des refus : [`DbError::NotFound`] (inexistant, autre société,
/// archivé) → [`DbError::OptimisticLockConflict`] →
/// [`ClosureRefusal::NotClosed`]. Même verrou unique que [`close_in_tx`] —
/// et, comme lui, victime possible d'un cycle par la file d'attente quand sa
/// demande exclusive attend derrière un détenteur partagé (Story 29-4a, #651) :
/// la route est rejouée (`bank_accounts::reopen`).
pub async fn reopen_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
    expected_version: i32,
) -> Result<ClosureOutcome, DbError> {
    let existing = lock_for_closure(tx, company_id, id, expected_version).await?;
    if !existing.is_closed() {
        return Ok(ClosureOutcome::Refused(ClosureRefusal::NotClosed));
    }
    let after = write_closed_on(tx, company_id, id, expected_version, None).await?;
    Ok(ClosureOutcome::Done {
        after: Box::new(after),
        before: Box::new(existing),
    })
}

/// Helper transition primary : flip un éventuel autre primary à `is_primary=FALSE`
/// (sans bump audit log — c'est le caller qui décide).
///
/// Story v014-1 (FINDING-3 Pass 3 Opus) — politique uniforme POST + PUT :
/// quand un client crée/édite un compte avec `is_primary=true` alors qu'un
/// autre primary existe, on flip silencieusement l'ancien (transition).
///
/// Filtre `archived = FALSE` + exclut `excluded_id` (le compte qu'on est en
/// train d'updater — ne pas se flip soi-même).
///
/// **F15 Pass 1 code review** : retourne `Option<(updated, existing)>` cohérent
/// avec `update_for_company` et `archive_for_company`. Le `existing` est le
/// snapshot pré-flip (capturé par le SELECT FOR UPDATE) — le caller l'utilise
/// directement comme source `before` de l'audit log `primary_transition` sans
/// arithmétique fragile sur `version`.
pub async fn flip_primary_off_for_company(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    excluded_id: i64,
) -> Result<Option<(BankAccount, BankAccount)>, DbError> {
    let existing = sqlx::query_as::<_, BankAccount>(
        concat!(
            "SELECT ",
            columns!(),
            " FROM bank_accounts WHERE company_id = ? AND is_primary = TRUE AND archived = FALSE AND id != ? FOR UPDATE",
        ),
    )
    .bind(company_id)
    .bind(excluded_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let Some(old_primary) = existing else {
        return Ok(None);
    };

    let rows = sqlx::query(
        "UPDATE bank_accounts SET is_primary = FALSE, version = version + 1 \
         WHERE id = ? AND company_id = ? AND version = ?",
    )
    .bind(old_primary.id)
    .bind(company_id)
    .bind(old_primary.version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        // Race : un autre tx a flippé le primary entre notre SELECT FOR UPDATE
        // (qui prend le lock) et notre UPDATE. Impossible en pratique car
        // FOR UPDATE bloque. Mais défense en profondeur.
        return Err(DbError::OptimisticLockConflict);
    }

    let updated = sqlx::query_as::<_, BankAccount>(FIND_BY_ID_SQL)
        .bind(old_primary.id)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_db_error)?;

    Ok(Some((updated, old_primary)))
}

/// Compte le nombre de `bank_transactions` associées à un bank_account
/// **dans une transaction fournie par le caller** (F2 Pass 1 code review —
/// élimine la fenêtre TOCTOU entre count hors-tx et archive dans tx).
///
/// Story v014-1 (AC#8) — utilisé par DELETE pour refuser 412
/// BANK_ACCOUNT_HAS_TRANSACTIONS si une transaction (pending ou reconciled)
/// existe. `bank_transactions` n'a pas de colonne `archived` — comptage
/// inconditionnel (auditabilité CO Art. 958f).
///
/// ⚠️ La garde compte les lignes **présentes** : depuis la Story 27-1 (#632),
/// la suppression d'un import non rapproché
/// (`bank_imports::delete_unreconciled_in_tx`) en retire — tracée
/// `bank_import.deleted` (fichier, empreinte, période, nombres). Un compte dont
/// tous les imports ont été supprimés peut donc s'archiver ; l'auditabilité
/// passe alors par la trace, non par la conservation des lignes (C-27-1-12).
pub async fn count_transactions_for_bank_account(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    bank_account_id: i64,
) -> Result<i64, DbError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM bank_transactions WHERE bank_account_id = ? AND company_id = ?",
    )
    .bind(bank_account_id)
    .bind(company_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;
    Ok(row.0)
}

/// Le drapeau `is_primary` du compte `id`, lu **sous `FOR UPDATE`** dans la
/// transaction de l'appelant (Story 29-4b, T6-bis, R-2 de la passe ciblée P2
/// de la 29-4a, #651) — `None` si le compte n'existe pas, appartient à une
/// autre société ou est archivé (même filtre qu'[`archive_for_company`]).
///
/// Sert la garde « principal + autres comptes actifs » de l'archivage : elle
/// décidait sur la valeur du pré-vol, lue sur le pool hors transaction, et que
/// le rejeu de la route réutilisait à chaque tentative ; un transfert de
/// principal commité entre-temps la faisait décider sur un état périmé. Lue
/// ici, sous verrou, elle voit le dernier état commité.
///
/// **Verrou** : l'exclusif de la ligne du compte — celui
/// qu'[`archive_for_company`] prend ensuite (sa redemande est sans effet),
/// **avancé** devant la garde. L'appelant tient déjà la sentinelle `companies`
/// en exclusif : l'ordre « `companies` X, puis compte X » de l'archivage est
/// inchangé, et entre les deux positions l'archivage ne fait que des lectures
/// non verrouillantes (le décompte des autres comptes actifs) — aucun verrou
/// neuf, aucun cycle neuf (raisonné, au code ; non mesuré).
pub async fn primary_flag_locked_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
) -> Result<Option<bool>, DbError> {
    sqlx::query_scalar::<_, bool>(
        "SELECT is_primary FROM bank_accounts \
         WHERE company_id = ? AND id = ? AND archived = FALSE FOR UPDATE",
    )
    .bind(company_id)
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)
}

/// Compte le nombre d'autres bank_accounts ni archivés ni clos du même company
/// (exclut le compte courant) **dans une transaction fournie par le caller**.
/// Utilisé par DELETE pour décider si on autorise l'archivage d'un primary
/// (autorisé seulement si c'est le dernier compte ni archivé ni clos — AC#10).
///
/// Story 28-4a (#642, AC2) — **exclut les comptes clos** : un compte clos ne
/// peut pas devenir principal (contrainte `chk_bank_accounts_closed_not_primary`),
/// il ne doit donc plus empêcher l'archivage du principal. ⚠️ Conséquence
/// assumée : quand tous les autres comptes sont clos, le principal s'archive
/// et la société reste **sans principal** (QR-facture sans IBAN) ; le remède
/// est de rouvrir un compte clos, puis de le désigner principal.
pub async fn count_other_active_for_company(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    excluded_id: i64,
) -> Result<i64, DbError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM bank_accounts \
         WHERE company_id = ? AND archived = FALSE AND closed_on IS NULL AND id != ?",
    )
    .bind(company_id)
    .bind(excluded_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;
    Ok(row.0)
}

/// Le **premier compte bancaire non archivé** (par `id`) lié au compte du grand
/// livre `account_id`, lu **sous verrou partagé** — `(id, nom d'affichage)`, ou
/// `None` (Story 15-6c, AC2 ; choix C-15-6-13, C-15-6-21, C-15-6-27). Story 26-2b
/// (#622, C-26-2-9) : le nom est celui de [`bank_account_display_name`]
/// (« UBS — Personnel »), pour que le refus de désignation nomme le compte.
///
/// ⛔ **Ordre des verrous** : appelé par `company_invoice_settings::update`
/// **après** le `FOR UPDATE` de la ligne des réglages — ordre global
/// `companies` → réglages → `bank_accounts`, celui de
/// `company_invoice_settings::claim_accounts_in_share_mode`. L'ordre inverse
/// ouvrirait un interblocage entre une désignation et un lien concurrents.
///
/// ⚠️ **Portée réelle du verrou** : sous REPEATABLE READ, ce balayage
/// verrouillant de l'index secondaire non unique `idx_bank_accounts_journal_account`
/// pose des verrous **next-key / d'intervalle**, y compris quand il ne trouve
/// aucune ligne. Un `INSERT INTO bank_accounts` dont le `journal_account_id`
/// tombe dans l'intervalle voisin peut donc **attendre** la fin du PUT des
/// réglages — y compris un INSERT **à `journal_account_id` NULL** (création
/// sans lien : l'`INSERT` de `upsert_primary_in_tx`, à l'onboarding, ne nomme
/// pas la colonne, qui prend donc NULL). Les NULL précèdent les valeurs dans
/// l'index (`idx_bank_accounts_journal_account`, sur la seule colonne
/// `journal_account_id`), et l'INSERT se pose au bord de l'intervalle qui
/// précède la première valeur non nulle : il n'attend donc **que lorsque
/// aucune valeur non nulle de l'index n'est inférieure à `account_id`** —
/// c'est-à-dire quand `account_id` est inférieur ou égal à la plus petite
/// valeur non nulle, seul cas où l'intervalle verrouillé par le balayage
/// touche la zone des NULL. Attente transitoire, sans cycle
/// (aucun flux ne tient `bank_accounts` puis ne réclame les réglages), **non
/// mesurée** (revue de code P1, F1).
///
/// Story 28-4a (#642, C-28-4-7) — **garde les comptes clos** : le compte
/// comptable d'un compte clos reste « lié à une banque » (le compte se
/// rouvre), et ne peut donc pas être désigné compte de créance.
///
/// ⚠️ `LOCK IN SHARE MODE`, pas `FOR SHARE` (erreur de syntaxe sur MariaDB
/// 10.11). Le test de sérialisation côté réglages attend sur le texte
/// `FROM bank_accounts` + `LOCK IN SHARE MODE`.
pub async fn first_active_bank_account_linked_to(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    account_id: i64,
) -> Result<Option<(i64, String)>, DbError> {
    let row: Option<(i64, String, Option<String>)> = sqlx::query_as(
        "SELECT id, bank_name, description FROM bank_accounts WHERE company_id = ? AND journal_account_id = ? AND archived = FALSE ORDER BY id LIMIT 1 LOCK IN SHARE MODE",
    )
    .bind(company_id)
    .bind(account_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;
    Ok(row.map(|(id, bank_name, description)| {
        (
            id,
            bank_account_display_name(&bank_name, description.as_deref()),
        )
    }))
}

/// Refuse de lier un compte bancaire au compte `account_id` s'il est l'un des
/// deux comptes de créance désignés (`claims`, lus par la route sous verrou
/// partagé) — [`DbError::BankAccountLedgerIsClaimAccount`] (Story 15-6c, AC3,
/// AC4). Le numéro n'est lu qu'une fois le refus décidé, sans verrou.
///
/// Partagé par la création (route) et par [`update_for_company`] /
/// [`set_journal_account_id_for_company`] (dépôt) : la comparaison n'existe
/// qu'une fois. Aux appelants de n'appeler que si la valeur **change**.
pub async fn refuse_if_ledger_is_claim_account(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    account_id: i64,
    claims: &super::company_invoice_settings::ClaimAccounts,
) -> Result<(), DbError> {
    let Some(claim) = claims.side_of(account_id) else {
        return Ok(());
    };
    let account_number = super::accounts::number_in_company(tx, company_id, account_id).await?;
    Err(DbError::BankAccountLedgerIsClaimAccount {
        account_id,
        account_number,
        claim,
    })
}

/// Acquiert un advisory lock sentinel sur la row `companies.id` pour
/// serializer les mutations CRUD bank_accounts d'un même tenant (mitigation
/// L5 race condition primary applicatif-only, FINDING-9 Pass 3 Opus).
///
/// À appeler au début des handlers POST/PUT avant tout autre SELECT FOR
/// UPDATE sur `bank_accounts`. Le lock est libéré au commit/rollback de la
/// tx. SELECT … FOR UPDATE sur companies n'impacte pas les autres handlers
/// qui ne mutent pas `companies` (lecture seule via `read uncommitted` ou
/// SELECT sans FOR UPDATE).
pub async fn acquire_company_sentinel_lock(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
) -> Result<(), DbError> {
    let _: Option<(i64,)> = sqlx::query_as("SELECT id FROM companies WHERE id = ? FOR UPDATE")
        .bind(company_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?;
    Ok(())
}

/// Soldes d'un compte bancaire, tels que la liste les rend (Story v014-1,
/// Story 25-6-a #389).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BankAccountBalances {
    /// **Solde comptable** du compte de grand livre lié — `SUM(debit) −
    /// SUM(credit)`, toutes dates. `None` sans compte lié ; `0` si le compte lié
    /// n'a aucune ligne.
    pub current_balance: Option<rust_decimal::Decimal>,
    /// `MAX(entry_date)` des lignes du compte lié.
    pub last_transaction_date: Option<chrono::NaiveDate>,
    /// Solde de clôture du **relevé dont la période se termine le plus tard**
    /// parmi ceux qui en portent un (CAMT ; un import CSV n'en a jamais) —
    /// `period_to` le plus récent, départagé par `imported_at` puis `id`.
    /// Ce n'est pas forcément le dernier importé.
    pub statement_closing_balance: Option<rust_decimal::Decimal>,
    /// `period_to` de ce relevé.
    pub statement_date: Option<chrono::NaiveDate>,
    /// Solde comptable du compte lié **à `statement_date` inclus**, corrigé des
    /// dates de valeur (voir [`list_by_company_with_balances`]). `None` sans
    /// relevé, sans compte lié, ou quand **plusieurs** comptes bancaires
    /// partagent le compte de grand livre (son solde ne s'attribue à aucun).
    pub ledger_balance_at_statement: Option<rust_decimal::Decimal>,
}

/// Agrégats d'un compte bancaire, lus en une requête ([`BALANCES_SQL`], par
/// [`balances_by_bank_account`] ; voir [`list_by_company_with_balances`]).
///
/// `pub(crate)` parce que [`balances_by_bank_account`] la rend au contrôle K3
/// de la préparation de clôture (`closing_checks`, Story 29-1a) : une
/// `pub(crate) fn` qui rend un type privé déclencherait `private_interfaces`.
/// Ses champs restent privés : on les lit par [`balances_from_row`] et
/// [`is_shared`].
#[derive(Debug, sqlx::FromRow)]
pub(crate) struct BalancesRow {
    bank_account_id: i64,
    balance: Option<rust_decimal::Decimal>,
    last_entry_date: Option<chrono::NaiveDate>,
    closing_balance: Option<rust_decimal::Decimal>,
    period_to: Option<chrono::NaiveDate>,
    ledger_at: Option<rust_decimal::Decimal>,
    booked_before_entered_after: Option<rust_decimal::Decimal>,
    booked_after_entered_before: Option<rust_decimal::Decimal>,
    sharing: Option<i64>,
}

/// Liste les comptes bancaires avec leurs soldes calculés + date dernière
/// transaction depuis `journal_entry_lines`, et le relevé dont la période se
/// termine le plus tard.
///
/// Story v014-1 T5 (FINDING-10 Pass 3 Opus — périmètre du calcul) :
/// - **Pas de filtre de status** : `journal_entries` n'a pas de colonne
///   `status` ; toute écriture insérée est par construction validée. Donc
///   toutes les `journal_entry_lines` participent au solde.
/// - Pas de filtre `fiscal_year_id` : « solde depuis création ».
/// - Pas de rollup hiérarchique (uniquement sur `journal_account_id` exact).
/// - `last_transaction_date` = `MAX(je.entry_date)` sur le compte lié.
///
/// # Le relevé de référence et l'écart (Story 25-6-a, #389)
///
/// Le solde comptable n'est **pas** le solde bancaire. Pour les confronter, la
/// liste rend le solde de clôture du relevé portant un solde dont la période
/// se termine le plus tard (non le dernier importé), et le
/// solde comptable **à sa date**. ⚠️ Le relevé reflète les mouvements par date
/// de **comptabilisation bancaire** (`booking_date`), alors que les écritures de
/// rapprochement sont datées à la date de **valeur** : le solde à la date du
/// relevé est donc **corrigé** des transactions rapprochées dont les deux dates
/// tombent de part et d'autre de `period_to` — `+` le mouvement que leur
/// écriture porte sur le compte lié quand la banque l'a comptabilisé avant (ou
/// le jour même) et l'écriture après, `−` dans le cas inverse. On somme la
/// **ligne de l'écriture sur le compte lié actuel**, pas `amount` : une écriture
/// posée sur un ancien compte lié sort d'elle-même de la correction. Une
/// transaction **non rapprochée** n'y entre pas — son absence du grand livre est
/// l'écart à montrer.
///
/// ⚠️ **Invariant supposé : une transaction, une écriture.** Les cinq chemins de
/// rapprochement (`kesh-api/src/routes/reconciliation.rs`) lient chacun une
/// écriture qu'ils viennent de créer ; aucune écriture n'est rapprochée de
/// plusieurs transactions. Si cela changeait, la correction ajouterait la ligne
/// entière de l'écriture là où seule la part d'une transaction franchit la date
/// du relevé (revue P1, lentille B).
///
/// # Forme
///
/// Une requête agrégée **par compte bancaire** ; chaque somme dans sa propre
/// table dérivée ou sous-requête corrélée. ⛔ Ne jamais joindre
/// `bank_transactions` dans le même `FROM` que les lignes du grand livre : le
/// produit multiplierait ces lignes et fausserait les sommes.
pub async fn list_by_company_with_balances(
    pool: &MySqlPool,
    company_id: i64,
    filter: BankAccountListFilter,
) -> Result<Vec<(BankAccount, BankAccountBalances)>, DbError> {
    let accounts = list_by_company(pool, company_id, filter).await?;

    if accounts.is_empty() {
        return Ok(Vec::new());
    }

    let by_bank_account = balances_by_bank_account(pool, company_id, None).await?;

    Ok(accounts
        .into_iter()
        .map(|b| {
            let balances = balances_from_row(b.journal_account_id, by_bank_account.get(&b.id));
            (b, balances)
        })
        .collect())
}

/// Le texte **unique** des agrégats par compte bancaire (voir
/// [`list_by_company_with_balances`], § *Forme*) — extrait à la Story 29-1a
/// (#232, AC3) pour que le contrôle K3 de la préparation de clôture lise
/// **la même** requête, bornée.
///
/// **La borne du relevé de référence** : `(? IS NULL OR period_to <= ?)`, posée
/// **dans** la table dérivée `st`, **avant** le `ROW_NUMBER()` — sinon `rn = 1`
/// désignerait le relevé postérieur à la borne, puis serait filtré, et le
/// compte n'aurait plus de relevé du tout. Sans borne (`NULL`), le résultat est
/// celui d'avant la story. La correction des dates de valeur est relative à
/// `st.period_to` : elle reste juste pour un relevé antérieur.
const BALANCES_SQL: &str = "SELECT ba.id AS bank_account_id, \
                agg.balance, agg.last_entry_date, \
                st.closing_balance, st.period_to, \
                CASE WHEN st.period_to IS NULL OR ba.journal_account_id IS NULL THEN NULL ELSE \
                  (SELECT COALESCE(SUM(jel.debit) - SUM(jel.credit), 0) \
                   FROM journal_entry_lines jel \
                   INNER JOIN journal_entries je ON jel.entry_id = je.id \
                   WHERE je.company_id = ba.company_id \
                     AND jel.account_id = ba.journal_account_id \
                     AND je.entry_date <= st.period_to) END AS ledger_at, \
                CASE WHEN st.period_to IS NULL OR ba.journal_account_id IS NULL THEN NULL ELSE \
                  (SELECT COALESCE(SUM(jel.debit) - SUM(jel.credit), 0) \
                   FROM journal_entry_lines jel \
                   INNER JOIN journal_entries je ON jel.entry_id = je.id \
                   WHERE je.company_id = ba.company_id \
                     AND jel.account_id = ba.journal_account_id \
                     AND je.entry_date > st.period_to \
                     AND je.id IN (SELECT bt.matched_entry_id FROM bank_transactions bt \
                                   WHERE bt.bank_account_id = ba.id AND bt.company_id = ba.company_id \
                                     AND bt.status = 'reconciled' AND bt.matched_entry_id IS NOT NULL \
                                     AND bt.booking_date <= st.period_to)) END \
                  AS booked_before_entered_after, \
                CASE WHEN st.period_to IS NULL OR ba.journal_account_id IS NULL THEN NULL ELSE \
                  (SELECT COALESCE(SUM(jel.debit) - SUM(jel.credit), 0) \
                   FROM journal_entry_lines jel \
                   INNER JOIN journal_entries je ON jel.entry_id = je.id \
                   WHERE je.company_id = ba.company_id \
                     AND jel.account_id = ba.journal_account_id \
                     AND je.entry_date <= st.period_to \
                     AND je.id IN (SELECT bt.matched_entry_id FROM bank_transactions bt \
                                   WHERE bt.bank_account_id = ba.id AND bt.company_id = ba.company_id \
                                     AND bt.status = 'reconciled' AND bt.matched_entry_id IS NOT NULL \
                                     AND bt.booking_date > st.period_to)) END \
                  AS booked_after_entered_before, \
                CAST(shared.n AS SIGNED) AS sharing \
         FROM bank_accounts ba \
         LEFT JOIN ( \
             SELECT jel.account_id, \
                    COALESCE(SUM(jel.debit) - SUM(jel.credit), 0) AS balance, \
                    MAX(je.entry_date) AS last_entry_date \
             FROM journal_entry_lines jel \
             INNER JOIN journal_entries je ON jel.entry_id = je.id \
             WHERE je.company_id = ? \
               AND jel.account_id IN (SELECT journal_account_id FROM bank_accounts \
                                      WHERE company_id = ? AND journal_account_id IS NOT NULL) \
             GROUP BY jel.account_id \
         ) agg ON agg.account_id = ba.journal_account_id \
         LEFT JOIN ( \
             SELECT bank_account_id, closing_balance, period_to, \
                    ROW_NUMBER() OVER (PARTITION BY bank_account_id \
                                       ORDER BY period_to DESC, imported_at DESC, id DESC) AS rn \
             FROM bank_imports \
             WHERE company_id = ? AND closing_balance IS NOT NULL \
               AND (? IS NULL OR period_to <= ?) \
         ) st ON st.bank_account_id = ba.id AND st.rn = 1 \
         LEFT JOIN ( \
             SELECT journal_account_id, COUNT(*) AS n \
             FROM bank_accounts \
             WHERE company_id = ? AND journal_account_id IS NOT NULL \
             GROUP BY journal_account_id \
         ) shared ON shared.journal_account_id = ba.journal_account_id \
         WHERE ba.company_id = ?";

/// Les agrégats de [`BALANCES_SQL`] pour **tous** les comptes bancaires de la
/// société (archivés et clos compris), indexés par `bank_account_id`, sur
/// l'exécuteur reçu — une connexion du pool, ou celle d'un instantané
/// (`closing_checks`). **Sans verrou.**
///
/// `statement_bound` : `None` = le relevé à solde dont la période se termine le
/// plus tard (tableau de bord) ; `Some(d)` = le plus tardif de ceux dont
/// `period_to <= d` (contrôle K3, borne = dernier jour de l'exercice).
pub(crate) async fn balances_by_bank_account<'e, E>(
    executor: E,
    company_id: i64,
    statement_bound: Option<chrono::NaiveDate>,
) -> Result<std::collections::HashMap<i64, BalancesRow>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    let rows: Vec<BalancesRow> = sqlx::query_as(BALANCES_SQL)
        .bind(company_id) // agg : je.company_id
        .bind(company_id) // agg : bornée aux comptes liés de la société
        .bind(company_id) // st : company_id
        .bind(statement_bound) // st : `? IS NULL`
        .bind(statement_bound) // st : `period_to <= ?`
        .bind(company_id) // shared
        .bind(company_id) // ba.company_id
        .fetch_all(executor)
        .await
        .map_err(map_db_error)?;
    Ok(rows.into_iter().map(|r| (r.bank_account_id, r)).collect())
}

/// Le partage du compte de grand livre lié : **plusieurs** comptes bancaires de
/// la société (archivés compris) le désignent. Même critère que
/// [`balances_from_row`] ; le contrôle K3 en tire le motif `NOT_COMPARABLE`,
/// que [`BankAccountBalances`] ne distingue pas d'un compte sans relevé.
pub(crate) fn is_shared(row: Option<&BalancesRow>) -> bool {
    row.is_some_and(|r| r.sharing.unwrap_or(1) > 1)
}

/// Passage d'une ligne de [`BALANCES_SQL`] aux soldes d'un compte bancaire —
/// le seul endroit qui sache combiner `ledger_at`, `booked_before_entered_after`
/// et `booked_after_entered_before` (correction des dates de valeur, voir
/// [`list_by_company_with_balances`]).
pub(crate) fn balances_from_row(
    journal_account_id: Option<i64>,
    row: Option<&BalancesRow>,
) -> BankAccountBalances {
    match (journal_account_id, row) {
        (Some(_), Some(r)) => {
            // Le partage se compte sur TOUS les comptes de la société,
            // archivés compris : la réponse ne dépend pas d'`include_archived`.
            let shared = is_shared(Some(r));
            let ledger_balance_at_statement = match (
                r.ledger_at,
                r.booked_before_entered_after,
                r.booked_after_entered_before,
            ) {
                (Some(at), Some(plus), Some(minus)) if !shared => Some(at + plus - minus),
                _ => None,
            };
            BankAccountBalances {
                // Compte lié sans aucune ligne → 0 (comportement v014-1).
                current_balance: Some(r.balance.unwrap_or(rust_decimal::Decimal::ZERO)),
                last_transaction_date: r.last_entry_date,
                statement_closing_balance: r.closing_balance,
                statement_date: r.period_to,
                ledger_balance_at_statement,
            }
        }
        // Sans compte lié : ni solde comptable ni écart ; le relevé reste.
        (None, Some(r)) => BankAccountBalances {
            statement_closing_balance: r.closing_balance,
            statement_date: r.period_to,
            ..Default::default()
        },
        (_, None) => BankAccountBalances::default(),
    }
}

/// **L'écart** entre le relevé et les livres, une seule définition côté
/// serveur (Story 29-1a, #232, C-29-1-17 et C-29-1-25) : servi au tableau de
/// bord (`statementGap` de `GET /api/v1/bank-accounts`) et lu par le contrôle
/// K3 de la préparation de clôture.
///
/// `None` si l'un des deux termes manque (`ledger_balance_at_statement` ou
/// `statement_closing_balance`) ; sinon `livres − relevé`, chacun **arrondi au
/// centime** d'abord (`Money::round_to_centimes`, demi loin de zéro) — les
/// lignes du grand livre sont en `DECIMAL(19,4)`, le relevé en
/// `DECIMAL(18,2)` : sans cet arrondi, un reste de `0.0030` passerait pour un
/// écart. `None` aussi quand l'écart est nul.
pub fn statement_gap(b: &BankAccountBalances) -> Option<rust_decimal::Decimal> {
    use kesh_core::types::Money;
    let ledger = Money::new(b.ledger_balance_at_statement?).round_to_centimes();
    let statement = Money::new(b.statement_closing_balance?).round_to_centimes();
    let gap = ledger.amount() - statement.amount();
    (!gap.is_zero()).then_some(gap)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// T-a45 (Story 28-4a, AC7) — les deux gardes sont des lectures
    /// **non verrouillantes** : leur SQL ne porte ni `FOR UPDATE`, ni
    /// `LOCK IN SHARE MODE`, ni `FOR SHARE`. ⛔ Ne prouve pas le critère sur
    /// les instructions élargies des autres chemins : celles-là se vérifient
    /// au diff, par la table *Position par chemin* de la fiche.
    #[test]
    fn les_gardes_du_compte_clos_ne_verrouillent_rien() {
        for sql in [ENSURE_OPEN_SQL, ENSURE_OPEN_UNARCHIVED_SQL] {
            let haut = sql.to_uppercase();
            for verrou in ["FOR UPDATE", "LOCK IN SHARE MODE", "FOR SHARE"] {
                assert!(
                    !haut.contains(verrou),
                    "une garde du compte clos ne doit prendre aucun verrou ({verrou}) : {sql}"
                );
            }
        }
        // La garde des annulations ne lit jamais `archived` (un archivé ne se
        // rouvre pas : refuser l'annulation la rendrait impossible).
        assert!(!ENSURE_OPEN_SQL.contains("archived"));
        assert!(ENSURE_OPEN_UNARCHIVED_SQL.contains("archived"));
    }

    fn soldes(livres: Option<&str>, releve: Option<&str>) -> BankAccountBalances {
        BankAccountBalances {
            ledger_balance_at_statement: livres.map(|s| s.parse().unwrap()),
            statement_closing_balance: releve.map(|s| s.parse().unwrap()),
            ..Default::default()
        }
    }

    /// T-a14 (Story 29-1a, AC3) — `statement_gap`, l'écart servi au tableau de
    /// bord et lu par K3. Les quatre premiers cas sont ceux de `homepage.test.ts`
    /// l. 57-80 à `16eeed21`, repris à l'identique : ils y étaient le test de
    /// la règle TypeScript, qui disparaît. Mutation tuée : M-a16 (écart brut,
    /// sans arrondi — le cas `100.0049` / `100.00` rendrait `0.0049`).
    #[test]
    fn l_ecart_du_releve_est_arrondi_au_centime() {
        use rust_decimal_macros::dec;
        // 800.0000 aux livres contre 1000.00 au relevé : écart de -200.
        assert_eq!(
            statement_gap(&soldes(Some("800.0000"), Some("1000.00"))),
            Some(dec!(-200))
        );
        // Un reste inférieur au demi-centime n'est pas un écart.
        assert_eq!(
            statement_gap(&soldes(Some("100.0049"), Some("100.00"))),
            None
        );
        // Le demi-centime s'arrondit loin de zéro : 100.005 → 100.01.
        assert_eq!(
            statement_gap(&soldes(Some("100.005"), Some("100.01"))),
            None
        );
        assert_eq!(
            statement_gap(&soldes(Some("-100.005"), Some("-100.01"))),
            None
        );
        // Un terme absent : pas d'écart calculable.
        assert_eq!(statement_gap(&soldes(None, Some("100.00"))), None);
        assert_eq!(statement_gap(&soldes(Some("100.00"), None)), None);
        assert_eq!(statement_gap(&soldes(None, None)), None);
        // Chaque solde est arrondi avant la soustraction (29-1a, P2 F1) : une
        // différence brute de 0.008 peut être nulle, une de 0.001 valoir 0.01.
        assert_eq!(
            statement_gap(&soldes(Some("100.004"), Some("99.996"))),
            None
        );
        assert_eq!(
            statement_gap(&soldes(Some("100.005"), Some("100.004"))),
            Some(dec!(0.01))
        );
        // L'écart de 0.005 aux livres en DECIMAL(19,4) vaut un centime.
        assert_eq!(
            statement_gap(&soldes(Some("100.005"), Some("100.00"))),
            Some(dec!(0.01))
        );
    }

    /// T-a45 étendu (Story 29-4a, #651, AC1) — les deux **revérifications**
    /// verrouillent, et en **partagé** seulement : `LOCK IN SHARE MODE`, ni
    /// `FOR UPDATE` (une demande exclusive d'un détenteur partagé, derrière une
    /// clôture en file, fermerait un cycle avec elle) ni `FOR SHARE` (refusé
    /// par MariaDB 10.11). Celle des annulations ne lit pas `archived`, celle
    /// de l'import et du lot le lit. Tue M29-3, M29-19 et M29-22 dans leur
    /// forme SQL.
    #[test]
    fn les_reverifications_verrouillent_en_partage() {
        for sql in [RECHECK_OPEN_LOCKED_SQL, RECHECK_OPEN_UNARCHIVED_LOCKED_SQL] {
            let haut = sql.to_uppercase();
            assert!(
                haut.contains("LOCK IN SHARE MODE"),
                "verrou partagé : {sql}"
            );
            for interdit in ["FOR UPDATE", "FOR SHARE"] {
                assert!(!haut.contains(interdit), "{interdit} interdit : {sql}");
            }
        }
        assert!(!RECHECK_OPEN_LOCKED_SQL.contains("archived"));
        assert!(RECHECK_OPEN_UNARCHIVED_LOCKED_SQL.contains("archived"));
    }
}
