//! Repository CRUD pour `Account`.
//!
//! **Story 3.5** : toutes les fonctions CRUD (`create`, `update`, `archive`)
//! enregistrent une entrée d'audit dans la même transaction. La signature
//! accepte un `user_id` pour identifier l'auteur de l'action.
//!
//! **Exception** : `bulk_create_from_chart` et sa variante `_in_tx` n'écrivent
//! pas elles-mêmes d'entrée d'audit. Le seed de démonstration n'en écrit pas
//! pour le plan (sa synthèse `installation.demo_seeded` en porte le nombre de
//! comptes, Story 15-7b1) ; l'onboarding de production, appelant de la variante
//! `_in_tx`, écrit une entrée agrégée `account.chart_loaded` (Story 15-7a2).

use sqlx::mysql::MySqlPool;

use crate::entities::Company;
use crate::entities::account::{
    Account, AccountRole, AccountType, AccountUpdate, CurrencyChange, FxAccounts, FxAccountsOrigin,
    GroupKind, GroupMember, GroupProposal, GroupsOutcome, MAX_GROUP_MEMBERS, MAX_GROUP_PROPOSALS,
    NewAccount, OwnFxAccounts, OwnFxAccountsChange, ParentChange,
};
use crate::entities::audit_log::NewAuditLogEntry;
use crate::errors::{
    AccountGroupReason, CurrencyUseSite, DbError, FxAccountField, FxAccountReason, FxDesignation,
    GroupStepError, map_db_error,
};
use crate::repositories::{audit_log, company_invoice_settings};

/// Colonnes de `accounts` sélectionnées pour hydrater [`Account`], sous forme de
/// littéral — un `macro_rules!` plutôt qu'un `const` pour rester utilisable dans
/// `concat!`, qui n'accepte que des littéraux.
///
/// ⚠️ La colonne générée `singleton_role` est **volontairement absente** : elle
/// n'existe que pour porter la contrainte d'unicité partielle des rôles
/// singleton, elle n'a pas de représentation Rust.
///
/// `currency` se lit telle quelle (Story 37-1, choix C-37-1-1) : la colonne est
/// en `ascii_general_ci`, non en collation binaire — une collation `_bin` est
/// signalée `BINARY` par le serveur, et `sqlx` refuse alors de la décoder en
/// `String` (`sqlx-mysql`, `str::compatible`). La casse reste fermée par les
/// `CHECK … BINARY` de la migration.
macro_rules! columns {
    () => {
        "id, company_id, number, name, account_type, parent_id, active, role, postable, \
         currency, fx_gain_account_id, fx_loss_account_id, \
         version, created_at, updated_at"
    };
}

/// Source unique des colonnes, pour les requêtes construites dynamiquement.
const COLUMNS: &str = columns!();

/// Requête de relecture par ID, assemblée **à la compilation**.
///
/// Code review 14-3a : c'était un `fn -> String` appelé 7× par le module, soit
/// une allocation par requête pour concaténer deux littéraux. `concat!` sur la
/// macro [`columns!`] donne le même résultat à coût nul, sans réintroduire la
/// duplication que le `format!` cherchait à éviter.
const FIND_BY_ID_SQL: &str = concat!("SELECT ", columns!(), " FROM accounts WHERE id = ?");

/// [`FIND_BY_ID_SQL`] sous verrou exclusif — tête du retypage (Story 34-11a2,
/// C-e341-40).
const FIND_BY_ID_FOR_UPDATE_SQL: &str = concat!(
    "SELECT ",
    columns!(),
    " FROM accounts WHERE id = ? FOR UPDATE"
);

/// Snapshot JSON d'un compte pour l'audit log (Story 3.5).
///
/// Contient les champs essentiels pour reconstituer l'état du compte
/// au moment de l'action. Les dates ne sont pas incluses car non
/// pertinentes pour l'audit (l'entrée d'audit a son propre `created_at`).
fn account_snapshot_json(account: &Account) -> serde_json::Value {
    serde_json::json!({
        "id": account.id,
        "companyId": account.company_id,
        "number": account.number,
        "name": account.name,
        "accountType": account.account_type.as_str(),
        "parentId": account.parent_id,
        "active": account.active,
        // Story 14-3a : sans ces deux champs, l'audit log mentirait sur les
        // modifications de rôle et de postabilité (le diff before/after serait vide).
        "role": account.role.map(|r| r.as_str()),
        "postable": account.postable,
        // Story 37-1 : la devise et les comptes de différence propres, sans
        // quoi le diff before/after d'un changement de devise serait vide.
        "currency": account.currency,
        "fxGainAccountId": account.fx_gain_account_id,
        "fxLossAccountId": account.fx_loss_account_id,
        "version": account.version,
    })
}

/// Vérifie que `role` est compatible avec `account_type` (code review 14-3a, D1).
///
/// La règle vit dans `kesh-core` — source unique partagée avec la validation des
/// plans comptables JSON. On la joint par la représentation textuelle commune
/// des deux enums jumeaux plutôt que par un `match` à 10 bras, qui deviendrait
/// une quatrième source à synchroniser.
fn check_role_account_type(
    role: Option<AccountRole>,
    account_type: AccountType,
) -> Result<(), DbError> {
    let Some(role) = role else {
        return Ok(());
    };
    let core = kesh_core::chart_of_accounts::AccountRole::ALL
        .into_iter()
        .find(|c| c.as_str() == role.as_str())
        .ok_or_else(|| {
            DbError::Invariant(format!(
                "rôle {} absent de kesh_core::AccountRole::ALL — les deux enums ont divergé",
                role.as_str()
            ))
        })?;

    if core.accepts_account_type(account_type.as_str()) {
        Ok(())
    } else {
        Err(DbError::AccountRoleInvalidForType {
            role: role.as_str().to_string(),
            account_type: account_type.as_str().to_string(),
        })
    }
}

/// `true` si le compte a au moins un sous-compte **actif**.
async fn has_active_children(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    id: i64,
) -> Result<bool, DbError> {
    let row: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM accounts WHERE parent_id = ? AND active = TRUE")
            .bind(id)
            .fetch_one(&mut **tx)
            .await
            .map_err(map_db_error)?;
    Ok(row.0 > 0)
}

/// Postabilité effective d'un compte — code review 14-3a, décisions D1 et D2.
///
/// Deux normalisations, appliquées à **toutes** les écritures (`create` comme
/// `update`) plutôt qu'à un seul chemin :
///
/// 1. le compte de **résultat de l'exercice** n'est jamais postable — l'appli
///    calcule ce solde (Story 14-1) ; l'accepter postable ouvrirait un
///    double-comptage à la Story 14-3b ;
/// 2. un compte ayant des **sous-comptes actifs** est un compte de regroupement,
///    donc non postable — c'est la règle que le seed et le backfill de migration
///    appliquent déjà.
///
/// Normaliser des deux côtés est ce qui rend l'invariant **structurel** : le
/// contrat full-replace du `PUT` fait qu'un client dont la page date d'avant la
/// création d'un sous-compte renverrait `postable: true` et rétablirait l'état
/// incohérent si seul `create` corrigeait le parent.
fn effective_postable(role: Option<AccountRole>, requested: bool, has_children: bool) -> bool {
    if role == Some(AccountRole::CurrentYearResult) || has_children {
        return false;
    }
    requested
}

/// Crée un compte et retourne l'entité persistée, avec audit log atomique (Story 3.5).
///
/// # Gardes de rôle (code review 14-3a)
///
/// Symétriques de celles d'[`update`] : compatibilité rôle ↔ type, unicité des
/// rôles singleton (nommant le compte détenteur), normalisation de `postable`.
/// Sans elles, un `POST` heurtait directement `uq_accounts_company_singleton_role`
/// et remontait un 1062 générique que le formulaire traduisait en « ce numéro
/// existe déjà » — alors que le numéro était libre.
///
/// # Devise et comptes de différence propres (Story 37-1, AC-4, AC-4 bis)
///
/// `new.currency` est déjà normalisée par l'appelant (`CHF` → `None`, code
/// inconnu refusé en `CURRENCY_UNKNOWN` avant la transaction), et la paire de
/// comptes propres est entière (forme `missing` jugée avant). Ordre des refus
/// dans la transaction : `MULTI_CURRENCY_DISABLED` (devise donnée, société
/// mono-devise) → `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT` (le parent est un compte
/// de différence de change : il ne reçoit pas de sous-compte) →
/// `ACCOUNT_IN_FOREIGN_CURRENCY` (rôle posé avec une devise) →
/// `FX_ACCOUNT_INVALID` des comptes propres (`unexpected` sans devise, puis
/// gain, puis perte) → refus existants (rôle singleton).
///
/// **Verrous** : `companies` en partage — en `FOR UPDATE`, **premier** verrou,
/// quand le geste porte des comptes propres —, les réglages de facturation en
/// partage, puis le parent `FOR UPDATE` (il passera non imputable), puis les
/// désignations propres et les comptes propres en lecture verrouillante.
pub async fn create(pool: &MySqlPool, user_id: i64, new: NewAccount) -> Result<Account, DbError> {
    // Refus pur, avant de prendre une connexion (comme [`update`]) ;
    // [`create_in_tx`] le refait en tête, sans effet ici.
    check_role_account_type(new.role, new.account_type)?;
    let mut tx = pool.begin().await.map_err(map_db_error)?;
    let account = create_in_tx(&mut tx, user_id, new).await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(account)
}

/// [`create`] dans la transaction de l'appelant (Story 46-5a, AC-1) : **tout**
/// le corps de la création — gardes, verrous (dans le même ordre), `INSERT`,
/// passage du parent non imputable, copie des cours, audit —, sans `begin()` ni
/// `commit()`. Une erreur est rendue telle quelle : la transaction de
/// l'appelant s'annule à l'abandon, avec ce qu'elle a déjà écrit.
pub async fn create_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    new: NewAccount,
) -> Result<Account, DbError> {
    check_role_account_type(new.role, new.account_type)?;

    let company_lock = if new.own_fx_accounts.is_some() {
        CompanyLock::ForUpdate
    } else {
        CompanyLock::Share
    };
    let state = lock_company_currency_in_tx(tx, new.company_id, company_lock).await?;
    lock_invoice_settings_share_in_tx(tx, new.company_id).await?;

    if new.currency.is_some() && !state.multi_currency_enabled {
        return Err(DbError::MultiCurrencyDisabled {
            fiscal_year_id: None,
        });
    }

    // Le parent, verrouillé avant d'être passé non imputable plus bas : un
    // compte de différence de change ne reçoit pas de sous-compte.
    if let Some(parent_id) = new.parent_id {
        let parent: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM accounts WHERE id = ? AND company_id = ? FOR UPDATE",
        )
        .bind(parent_id)
        .bind(new.company_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?;
        if parent.is_some() {
            let hits = fx_designations_of_in_tx(tx, new.company_id, &state, parent_id).await?;
            refuse_designated(hits, |_| true)?;
            refuse_foreign_parent_with_entries_in_tx(tx, parent_id).await?;
        }
    }

    if let Some(currency) = &new.currency
        && new.role.is_some()
    {
        return Err(DbError::AccountInForeignCurrency {
            account_id: None,
            currency: currency.clone(),
        });
    }

    // Revue de code P2 (37-1-CR-P2-8) : le parent devient un compte de
    // regroupement (non imputable) dans cette transaction ; lu encore imputable,
    // il passerait le contrôle des comptes propres — refusé sur l'état résultant.
    if let (Some(parent_id), Some(own)) = (new.parent_id, new.own_fx_accounts) {
        for (field, id) in [
            (FxAccountField::Gain, own.gain),
            (FxAccountField::Loss, own.loss),
        ] {
            if id == parent_id {
                return Err(DbError::FxAccountInvalid {
                    field,
                    reason: FxAccountReason::NotPostable,
                    holder_account_id: None,
                });
            }
        }
    }

    check_own_fx_accounts_in_tx(
        tx,
        new.company_id,
        None,
        new.currency.as_deref(),
        None,
        new.own_fx_accounts,
    )
    .await?;

    // Un compte neuf n'a pas encore d'enfants : seule la règle « résultat de
    // l'exercice » peut s'appliquer ici.
    let postable = effective_postable(new.role, new.postable, false);

    // Le rôle singleton est-il déjà porté ? `exclude_id = 0` : aucune ligne ne
    // porte cet ID (AUTO_INCREMENT démarre à 1), donc rien n'est exclu.
    if let Some(role) = new.role
        && let Some(holder) = find_singleton_role_holder(tx, new.company_id, role, 0).await?
    {
        return Err(role_conflict(role, holder));
    }

    // Story 37-1 : la devise et les comptes propres s'écrivent tels que reçus
    // (déjà normalisés par l'appelant) ; le schéma refuse une graphie hors
    // `[A-Z]{3}`, `CHF`, et des comptes propres dépareillés ou sur un compte CHF.
    let result = sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, parent_id, role, postable, \
             currency, fx_gain_account_id, fx_loss_account_id) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(new.company_id)
    .bind(&new.number)
    .bind(&new.name)
    .bind(new.account_type)
    .bind(new.parent_id)
    .bind(new.role)
    .bind(postable)
    .bind(&new.currency)
    .bind(new.own_fx_accounts.map(|o| o.gain))
    .bind(new.own_fx_accounts.map(|o| o.loss))
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let last_id = result.last_insert_id();
    if last_id == 0 {
        return Err(DbError::Invariant(
            "last_insert_id == 0 après INSERT accounts".into(),
        ));
    }
    let id = i64::try_from(last_id)
        .map_err(|_| DbError::Invariant(format!("last_insert_id {last_id} dépasse i64::MAX")))?;

    // Code review 14-3a (D2) : le parent vient d'acquérir un sous-compte, il
    // devient donc un compte de regroupement. Les chemins qui créent une
    // relation de parenté sont celui-ci, le seed et, depuis le correctif
    // `fix-api-parent-compte`, le changement de parent d'[`update`], qui
    // applique la même règle ; la normalisation d'[`update`] couvre le reste.
    // Le scope `company_id` est défensif : sans lui, un `parentId` pointant sur
    // une autre société ferait écrire hors du tenant courant.
    // Pas de bump de `version` : `postable` est ici une conséquence structurelle,
    // pas une modification demandée par un utilisateur sur le parent.
    if let Some(parent_id) = new.parent_id {
        sqlx::query("UPDATE accounts SET postable = FALSE WHERE id = ? AND company_id = ?")
            .bind(parent_id)
            .bind(new.company_id)
            .execute(&mut **tx)
            .await
            .map_err(map_db_error)?;
    }

    let account = sqlx::query_as::<_, Account>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| DbError::Invariant(format!("account {id} introuvable après INSERT")))?;

    // Story 37-2 (AC-3 bis) : un compte créé avec une devise reçoit la copie
    // des cours déjà récupérés de cette devise, pour chaque exercice
    // multi-devises ouvert de la société, dans cette transaction (après les
    // verrous de la société). Le nombre copié va au détail de l'audit.
    let mut details = account_snapshot_json(&account);
    if let Some(currency) = &account.currency {
        let copied = crate::repositories::exchange_rates::copy_currency_to_open_years_in_tx(
            tx,
            account.company_id,
            currency,
        )
        .await?;
        details["copied"] = serde_json::json!(copied);
    }

    // Story 3.5 : audit log AVANT commit (snapshot direct, cohérent
    // avec la convention projet documentée en spec 3.5), dans la transaction
    // de l'appelant.
    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::user(
            user_id,
            "account.created".to_string(),
            "account".to_string(),
            account.id,
            Some(details),
        ),
    )
    .await?;
    Ok(account)
}

/// Les comptes de la société dont la devise est **verrouillée** (Story 37-1,
/// AC-4) : ceux qui portent au moins une ligne d'écriture, archivés compris —
/// en **une** requête bornée à la société, par l'index de la clé étrangère
/// `fk_jel_account`, pour `GET /accounts` (jamais une requête par compte).
/// Lecture ordinaire : c'est l'indication de l'écran, non la garde.
pub async fn currency_locked_account_ids(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
) -> Result<std::collections::BTreeSet<i64>, DbError> {
    let ids: Vec<i64> = sqlx::query_scalar(
        "SELECT a.id FROM accounts a WHERE a.company_id = ? \
         AND EXISTS (SELECT 1 FROM journal_entry_lines l WHERE l.account_id = a.id)",
    )
    .bind(company_id)
    .fetch_all(conn)
    .await
    .map_err(map_db_error)?;
    Ok(ids.into_iter().collect())
}

/// [`currency_locked_account_ids`] pour un seul compte (réponses à un compte :
/// création, modification, détail). `false` pour un compte d'une autre
/// société ou introuvable.
pub async fn is_currency_locked(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    account_id: i64,
) -> Result<bool, DbError> {
    let locked: Option<i64> = sqlx::query_scalar(
        "SELECT a.id FROM accounts a WHERE a.id = ? AND a.company_id = ? \
         AND EXISTS (SELECT 1 FROM journal_entry_lines l WHERE l.account_id = a.id)",
    )
    .bind(account_id)
    .bind(company_id)
    .fetch_optional(conn)
    .await
    .map_err(map_db_error)?;
    Ok(locked.is_some())
}

/// Retourne un compte par ID (ou None).
pub async fn find_by_id(pool: &MySqlPool, id: i64) -> Result<Option<Account>, DbError> {
    sqlx::query_as::<_, Account>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)
}

/// Retourne un compte par ID si et seulement s'il appartient à la company spécifiée (ou None).
/// Story 6.2: Multi-tenant scoping — utilisé pour les handlers PUT/DELETE qui doivent vérifier IDOR.
pub async fn find_by_id_in_company(
    pool: &MySqlPool,
    id: i64,
    company_id: i64,
) -> Result<Option<Account>, DbError> {
    sqlx::query_as::<_, Account>(&format!(
        "SELECT {COLUMNS} FROM accounts WHERE id = ? AND company_id = ?"
    ))
    .bind(id)
    .bind(company_id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

/// Retourne le nombre de comptes d'une company.
///
/// **Générique sur l'exécuteur** (Story 15-7a1, patron de
/// [`crate::repositories::journal_entries::count_by_company`]) : appelée avec
/// `&pool`, elle lit l'état commité ; appelée avec `&mut *tx`, elle voit en plus
/// les insertions **non commitées** de cette transaction. Ne commite jamais.
///
/// ⚠️ **Garde « aucun compte » sous verrou** : sous REPEATABLE READ, la première
/// lecture cohérente d'une transaction fige son instantané. Une garde
/// `count_by_company(&mut *tx, …) == 0` n'est exacte que si elle est lue
/// **après** le verrou d'état ([`crate::repositories::onboarding::lock_state_in_tx`]),
/// jamais avant.
pub async fn count_by_company<'e, E>(executor: E, company_id: i64) -> Result<i64, DbError>
where
    E: sqlx::Executor<'e, Database = sqlx::MySql>,
{
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM accounts WHERE company_id = ?")
        .bind(company_id)
        .fetch_one(executor)
        .await
        .map_err(map_db_error)?;
    Ok(row.0)
}

/// Retourne les paires `(id, account_type)` des comptes demandés, **scopées
/// company** (Story 14-4, P1-M1-ECH).
///
/// Sert la garde « comptes de bilan » de `POST /opening-balances` : le handler
/// rejette toute ligne dont le type retourné est `Revenue`/`Expense`. Un id
/// **absent** du résultat (compte inexistant ou d'une autre company) n'est PAS
/// une erreur ici — il retombe dans la garde `InactiveOrInvalidAccounts` de
/// `journal_entries::create_in_tx` (contrat AC-B, findings P3-AA-1/BH3-6).
///
/// `ids` vide → `Ok(vec![])` sans requête : un `IN ()` vide est une erreur de
/// syntaxe MariaDB qui remonterait en 500 (P3-ECH-LOW-1). Clause `IN`
/// construite par placeholders bindés manuellement (pattern
/// `reconciliation::find_contacts_by_ids`) — pas de N+1.
pub async fn find_types_by_ids_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    ids: &[i64],
) -> Result<Vec<(i64, String)>, DbError> {
    if ids.is_empty() {
        return Ok(vec![]);
    }
    let placeholders = std::iter::repeat_n("?", ids.len())
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT id, account_type FROM accounts WHERE company_id = ? AND id IN ({placeholders})"
    );
    let mut q = sqlx::query_as::<_, (i64, String)>(&sql).bind(company_id);
    for id in ids {
        q = q.bind(id);
    }
    q.fetch_all(&mut **tx).await.map_err(map_db_error)
}

/// Refuse un compte **actif et non imputable** de la société, désigné par le
/// client pour recevoir des écritures (Story 15-5b, #427) :
/// [`DbError::AccountsNotPostable`], HTTP 400 `ACCOUNT_NOT_POSTABLE`.
///
/// Contrôle partagé des dépôts qui gardent un compte **dans leur
/// transaction** — compte de contrepartie d'une règle de rapprochement
/// (`reconciliation_rules::create_in_tx`, `update_in_tx`) et compte comptable
/// d'un compte bancaire (`bank_accounts::update_for_company`,
/// `set_journal_account_id_for_company`).
///
/// Un compte inconnu, d'une autre société ou archivé — imputable ou non —
/// **n'est pas refusé ici** : le pré-vol des handlers l'a déjà rendu en 404
/// (anti-énumération KF-002), et la variante n'est émise que pour un compte
/// de la société, actif, comme l'exige la Story 15-5a. Lecture sans verrou
/// sur la ligne `accounts` : même dette de course LOW que le contrôle
/// `active`.
pub async fn ensure_postable_if_active_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    account_id: i64,
) -> Result<(), DbError> {
    let row: Option<(String, bool, bool)> = sqlx::query_as(
        "SELECT number, postable, active FROM accounts WHERE id = ? AND company_id = ?",
    )
    .bind(account_id)
    .bind(company_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;
    if let Some((number, postable, active)) = row
        && active
        && !postable
    {
        return Err(DbError::accounts_not_postable([
            crate::errors::NonPostableAccount {
                account_id,
                account_number: number,
            },
        ]));
    }
    Ok(())
}

/// Le **numéro** d'un compte de la société, lu dans la connexion de l'appelant
/// **sans verrou** — `None` si la ligne est absente (Story 15-6c ; partagé avec
/// le refus de la 15-6b, `invoice_settlements::claim_account_refusal`).
///
/// Sert à **nommer** un compte dans un refus déjà décidé : le refus ne dépend
/// plus de cette lecture, d'où l'absence de verrou. Une erreur SQL est rendue
/// telle quelle ; c'est alors elle que l'appelant propage.
pub async fn number_in_company(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    account_id: i64,
) -> Result<Option<String>, DbError> {
    sqlx::query_scalar("SELECT number FROM accounts WHERE id = ? AND company_id = ?")
        .bind(account_id)
        .bind(company_id)
        .fetch_optional(conn)
        .await
        .map_err(map_db_error)
}

/// Verrou d'une lecture de compte par rôle singleton
/// ([`find_singleton_role_in_tx`]) — Story 34-1 (#966).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleLookupLock {
    /// Lecture ordinaire, sans verrou de ligne : l'appelant tient déjà de quoi
    /// sérialiser (l'étape bancaire de l'assistant tient `onboarding_state` en
    /// `FOR UPDATE`, et la finalisation reverrouille ces lignes).
    None,
    /// `FOR UPDATE` : la ligne du compte (et l'entrée d'index du rôle, ou
    /// l'intervalle où elle serait) reste verrouillée jusqu'à la fin de la
    /// transaction — la finalisation de l'assistant
    /// (`company_invoice_settings::complete_default_accounts_in_tx`).
    ForUpdate,
}

/// L'identifiant du compte **actif** qui porte le rôle singleton `role`, lu dans
/// la transaction de l'appelant — `None` si aucun compte actif ne le porte
/// (Story 34-1, #966, T3).
///
/// Interroge la colonne générée `singleton_role` (le rôle si le compte est actif
/// et le rôle singleton, `NULL` sinon), donc l'index unique
/// `uq_accounts_company_singleton_role` : au plus une ligne, et le filtre
/// `active` est déjà encodé. ⛔ Lit le **rôle**, jamais le numéro (Story 14-3a) :
/// un plan renuméroté est suivi.
///
/// Source unique de cette lecture : la finalisation de l'assistant
/// (`company_invoice_settings::complete_default_accounts_in_tx`, trois rôles,
/// [`RoleLookupLock::ForUpdate`]) et l'étape bancaire de l'assistant (comptes
/// `Receivable` et `Payable` qu'un compte bancaire ne peut pas lier,
/// [`RoleLookupLock::None`]).
///
/// ⚠️ Ne lit pas `postable` : un compte de rôle sous lequel un sous-compte a été
/// créé reste rendu (la colonne générée ne lit que `active`). La finalisation
/// l'écarte ensuite, par une relecture verrouillante (Story 36-19, D6, V3-1) ;
/// l'étape bancaire, plus stricte, refuse le lien au compte qui porte le rôle,
/// imputable ou non.
pub async fn find_singleton_role_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    role: AccountRole,
    lock: RoleLookupLock,
) -> Result<Option<i64>, DbError> {
    let sql = match lock {
        RoleLookupLock::None => {
            "SELECT id FROM accounts WHERE company_id = ? AND singleton_role = ? ORDER BY id LIMIT 1"
        }
        RoleLookupLock::ForUpdate => {
            "SELECT id FROM accounts WHERE company_id = ? AND singleton_role = ? ORDER BY id LIMIT 1 FOR UPDATE"
        }
    };
    sqlx::query_scalar::<_, i64>(sql)
        .bind(company_id)
        .bind(role)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)
}

/// Le **type** et l'état actif d'un compte de la société, lus dans la
/// transaction de l'appelant **sous verrou partagé** (`LOCK IN SHARE MODE`) —
/// Story 34-11a2 (#972, C-e341-40). `None` si la ligne est absente.
///
/// Sert la création, le `PUT` et le `PATCH` d'un compte bancaire : ils relisent
/// le type du compte lié dans leur transaction, **après** les réglages et
/// **avant** la ligne `bank_accounts` (ordre global `companies` → réglages →
/// `accounts` → `bank_accounts`). Le retypage (`update`) prend la même ligne
/// `FOR UPDATE` en tête : les deux gestes se sérialisent sur elle, et aucun ne
/// laisse une carte liée à un compte qui n'est plus un passif.
///
/// ⚠️ `LOCK IN SHARE MODE`, pas `FOR SHARE` (erreur de syntaxe sur MariaDB 10.11).
///
/// Story 37-1 (AC-4) : rend aussi la **devise** du compte (troisième membre,
/// `None` pour le CHF) — un compte en devise ne se lie pas à un compte
/// bancaire (`ACCOUNT_IN_FOREIGN_CURRENCY`), jugé sous le même verrou ;
/// `accounts::update` prend la même ligne `FOR UPDATE` avant de poser une
/// devise et relit les liens bancaires en lecture verrouillante.
pub async fn type_in_share_mode(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    account_id: i64,
) -> Result<Option<(AccountType, bool, Option<String>)>, DbError> {
    // ⚠️ `currency` en tête : les sondes de course des routes bancaires
    // reconnaissent cette lecture au motif `account_type, active FROM accounts`.
    let row: Option<(Option<String>, AccountType, bool)> = sqlx::query_as(
        "SELECT currency, account_type, active FROM accounts WHERE id = ? AND company_id = ? LOCK IN SHARE MODE",
    )
    .bind(account_id)
    .bind(company_id)
    .fetch_optional(conn)
    .await
    .map_err(map_db_error)?;
    Ok(row.map(|(currency, account_type, active)| (account_type, active, currency)))
}

/// Story 38-11b (AC-2) — ce qu'il faut savoir d'un compte pour en faire le
/// **compte de frais** d'un compte PayPal : `(type, actif, imputable, devise)`,
/// lu dans la transaction de l'appelant **sous verrou partagé**, comme
/// [`type_in_share_mode`] (même ordre : après les réglages, avant
/// `bank_accounts`). `None` si la ligne est absente ou d'une autre société.
pub async fn fee_account_facts_in_share_mode(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    account_id: i64,
) -> Result<Option<(AccountType, bool, bool, Option<String>)>, DbError> {
    sqlx::query_as(
        "SELECT account_type, active, postable, currency FROM accounts \
         WHERE id = ? AND company_id = ? LOCK IN SHARE MODE",
    )
    .bind(account_id)
    .bind(company_id)
    .fetch_optional(conn)
    .await
    .map_err(map_db_error)
}

/// Le premier compte bancaire **non archivé** lié au compte `account_id` dont le
/// type n'admet pas `new_type`, par `id` — `(id, nom d'affichage)` — Story
/// 34-11a2 (#972, C-e341-31). Lecture `LOCK IN SHARE MODE` : elle voit la
/// dernière version commitée (un lien posé pendant que le retypage attendait la
/// ligne `accounts`). Un compte bancaire **archivé** ne compte pas : il ne se
/// modifie plus et ne sert plus. Un compte **clos** compte : il se rouvre.
async fn first_linked_bank_account_refusing(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    account_id: i64,
    new_type: AccountType,
) -> Result<Option<(i64, String)>, DbError> {
    use crate::entities::{BankAccountKind, bank_account::bank_account_display_name};
    let rows: Vec<(i64, BankAccountKind, String, Option<String>)> = sqlx::query_as(
        "SELECT id, account_kind, bank_name, description FROM bank_accounts \
         WHERE company_id = ? AND journal_account_id = ? AND archived = FALSE \
         ORDER BY id LOCK IN SHARE MODE",
    )
    .bind(company_id)
    .bind(account_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(map_db_error)?;
    Ok(rows
        .into_iter()
        .find(|(_, kind, _, _)| !kind.admits_ledger_type(new_type))
        .map(|(id, _, bank_name, description)| {
            (
                id,
                bank_account_display_name(&bank_name, description.as_deref()),
            )
        }))
}

/// `include_archived` : si `false`, seuls les comptes actifs sont retournés.
/// Pas de pagination — un plan comptable est borné à ~200-400 comptes.
pub async fn list_by_company(
    pool: &MySqlPool,
    company_id: i64,
    include_archived: bool,
) -> Result<Vec<Account>, DbError> {
    if include_archived {
        sqlx::query_as::<_, Account>(&format!(
            "SELECT {COLUMNS} FROM accounts WHERE company_id = ? ORDER BY number"
        ))
        .bind(company_id)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)
    } else {
        sqlx::query_as::<_, Account>(&format!(
            "SELECT {COLUMNS} FROM accounts WHERE company_id = ? AND active = TRUE ORDER BY number"
        ))
        .bind(company_id)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)
    }
}

/// Compare l'état persisté au payload — `true` si aucun champ métier ne diffère
/// (KF-004 : court-circuit no-op pour ne pas bumper version inutilement).
fn is_no_op_change(before: &Account, changes: &AccountUpdate) -> bool {
    before.name == changes.name
        && before.account_type == changes.account_type
        // Story 14-3a : sans ces deux comparaisons, un PUT qui ne change QUE le
        // rôle serait silencieusement ignoré et retournerait 200 avec l'ancienne
        // valeur — un bug utilisateur invisible.
        && before.role == changes.role
        && before.postable == changes.postable
        // Story 37-1 : renvoyer la devise en place, ou les comptes propres en
        // place, n'est pas un changement.
        && changes.currency.resolve(before.currency.as_deref()) == before.currency
        && changes.own_fx_accounts.resolve(before.own_fx_accounts()) == before.own_fx_accounts()
        // Correctif `fix-api-parent-compte` : renvoyer le parent en place n'est
        // pas un changement ; sans cette comparaison, un `PUT` qui ne change que
        // le parent serait ignoré en silence.
        && changes.parent.resolve(before.parent_id) == before.parent_id
}

/// La société d'un compte, lue **sur le pool, hors transaction** (Story 37-1,
/// AC-4) : `company_id` est immuable, et une lecture hors transaction ne fige
/// pas l'instantané REPEATABLE READ de la transaction qui suit. ⛔ Jamais
/// `LOCK IN SHARE MODE` sur la ligne du compte : le `FOR UPDATE` qui suit en
/// serait une promotion S → X, et deux gestes concurrents sur le même compte
/// s'interbloqueraient. Compte absent → `NotFound`.
async fn company_of_account(pool: &MySqlPool, id: i64) -> Result<i64, DbError> {
    sqlx::query_scalar::<_, i64>("SELECT company_id FROM accounts WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)?
        .ok_or(DbError::NotFound)
}

/// Les gardes de devise et de désignation d'[`update`] (Story 37-1, AC-4,
/// AC-4 bis), dans leur ordre, sur l'état résultant — après le no-op, avant
/// les refus existants. `changes.postable` est déjà normalisé.
async fn guard_currency_update(
    conn: &mut sqlx::MySqlConnection,
    state: &CompanyCurrencyState,
    before: &Account,
    changes: &AccountUpdate,
    currency: Option<&str>,
    own_fx: Option<OwnFxAccounts>,
) -> Result<(), DbError> {
    let company_id = before.company_id;
    let currency_changed = currency != before.currency.as_deref();
    let currency_posed = currency_changed && currency.is_some();

    // 1. MULTI_CURRENCY_DISABLED.
    if currency_posed && !state.multi_currency_enabled {
        return Err(DbError::MultiCurrencyDisabled {
            fiscal_year_id: None,
        });
    }

    // 2. ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT — un compte désigné (société ou
    //    comptes propres) ne reçoit pas de devise, ne sort pas de son type, ne
    //    passe pas non imputable, ne reçoit pas le rôle `DefaultRevenue`.
    let retyped = before.account_type != changes.account_type;
    let postable_off = before.postable && !changes.postable;
    let default_revenue_posed = changes.role == Some(AccountRole::DefaultRevenue)
        && before.role != Some(AccountRole::DefaultRevenue);
    if currency_posed || retyped || postable_off || default_revenue_posed {
        let hits = fx_designations_of_in_tx(conn, company_id, state, before.id).await?;
        refuse_designated(hits, |d| {
            currency_posed
                || postable_off
                || (retyped && !designation_admits_type(d, changes.account_type))
                || (default_revenue_posed && d != FxDesignation::Transit)
        })?;
    }

    // 3. ACCOUNT_CURRENCY_LOCKED — la devise d'un compte qui porte une ligne
    //    ne change plus (elle est celle de toutes ses lignes passées).
    if currency_changed && has_journal_line_locked(conn, before.id).await? {
        return Err(DbError::AccountCurrencyLocked);
    }

    // 3 bis. ACCOUNT_CURRENCY_PROVIDER_LINKED — Story 38-11b, revue P1 (M3) :
    //    la devise d'un compte lié au solde d'un prestataire est celle du
    //    solde ; elle ne change dans aucun sens (retirée, posée, remplacée), même
    //    sans ligne d'écriture (le solde porte déjà ses transactions importées).
    //    Avant la garde 4, dont le message (« employé en CHF ») ne dirait pas la
    //    raison.
    if currency_changed
        && let Some(bank_account_id) =
            provider_bank_account_linked_in_tx(conn, company_id, before.id).await?
    {
        return Err(DbError::AccountCurrencyProviderLinked { bank_account_id });
    }

    // 4. ACCOUNT_CURRENCY_IN_USE — un compte débité en CHF par un chemin
    //    automatique ne reçoit pas de devise : rôle gardé, puis les sites de
    //    configuration.
    if currency_posed {
        if let Some(role) = changes.role
            && before.role == Some(role)
        {
            return Err(DbError::AccountCurrencyInUse {
                site: CurrencyUseSite::Role(role.as_str()),
            });
        }
        if let Some(site) = currency_use_site_in_tx(conn, company_id, before.id).await? {
            return Err(DbError::AccountCurrencyInUse { site });
        }
    }

    // 5. ACCOUNT_IN_FOREIGN_CURRENCY — un rôle posé sur un compte en devise.
    if let Some(code) = currency
        && changes.role.is_some()
        && changes.role != before.role
    {
        return Err(DbError::AccountInForeignCurrency {
            account_id: Some(before.id),
            currency: code.to_string(),
        });
    }

    // 6. FX_ACCOUNT_INVALID des comptes propres. Société mono-devise : `unexpected`
    //    sans détenteur — le message dit alors le mode (37-1-CR-P3-1) ; celui
    //    du compte en devise ne vaut que dans une société multi-devises.
    if own_fx.is_some()
        && !matches!(changes.own_fx_accounts, OwnFxAccountsChange::Unchanged)
        && !state.multi_currency_enabled
    {
        return Err(DbError::FxAccountInvalid {
            field: FxAccountField::Gain,
            reason: FxAccountReason::Unexpected,
            holder_account_id: None,
        });
    }
    check_own_fx_accounts_in_tx(
        conn,
        company_id,
        Some(before.id),
        currency,
        before.own_fx_accounts(),
        own_fx,
    )
    .await
}

/// Recense ce qu'un changement d'`account_type` infligerait à l'historique d'un
/// compte : le nombre d'écritures **distinctes** qui le mouvementent, et les noms
/// des exercices **clos** parmi elles.
///
/// Story 25-2-a ([#382], [#274]) — c'est la matière du refus `ACCOUNT_HAS_ENTRIES`.
/// Un refus qui ne dit pas ce qu'il protège ne sert qu'à être contourné : le
/// comptable doit lire *combien* d'écritures et *quels exercices clos* basculent
/// d'un état à l'autre avant de confirmer.
///
/// ⚠️ **Des écritures distinctes, pas des lignes.** Une écriture qui mouvemente
/// deux fois le même compte reste **une** écriture ; l'annoncer deux fois
/// surestimerait l'ampleur à l'instant précis où l'utilisateur décide.
///
/// Générique sur l'exécuteur — comme `journal_entries::reversal_blocker` et
/// `journal_entries::count_by_company` — pour servir les tests sur un pool et
/// [`update`] dans sa propre transaction, sans deux variantes à tenir
/// synchronisées.
pub(crate) async fn retype_impact<'e, E>(
    executor: E,
    company_id: i64,
    account_id: i64,
) -> Result<(i64, Vec<String>), DbError>
where
    E: sqlx::Executor<'e, Database = sqlx::MySql>,
{
    // Une seule requête, groupée par exercice.
    //
    // ⚠️ `COUNT(DISTINCT je.id)` et non `COUNT(*)` : deux lignes d'une même
    // écriture sur ce compte ne font pas deux écritures.
    //
    // ⚠️ Le total s'obtient en **additionnant** les comptes par exercice, et
    // c'est exact parce qu'une écriture appartient à un seul exercice
    // (`journal_entries.fiscal_year_id` est unique et non nul). L'alternative —
    // un `GROUP_CONCAT` des noms d'exercices clos — exigerait un séparateur
    // qu'aucune contrainte n'interdit d'apparaître dans `fiscal_years.name`.
    let rows: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT fy.name, fy.status, COUNT(DISTINCT je.id) \
           FROM journal_entry_lines jel \
           JOIN journal_entries je ON je.id = jel.entry_id \
           JOIN fiscal_years fy ON fy.id = je.fiscal_year_id \
          WHERE jel.account_id = ? AND je.company_id = ? \
          GROUP BY fy.id, fy.name, fy.status \
          ORDER BY fy.name",
    )
    .bind(account_id)
    .bind(company_id)
    .fetch_all(executor)
    .await
    .map_err(map_db_error)?;

    let count: i64 = rows.iter().map(|(_, _, n)| *n).sum();
    let closed_fiscal_years: Vec<String> = rows
        .iter()
        .filter(|(_, status, _)| status == "Closed")
        .map(|(name, _, _)| name.clone())
        .collect();

    Ok((count, closed_fiscal_years))
}

/// Les contrôles d'un changement de parent d'[`update`] (correctif
/// `fix-api-parent-compte`) : ceux de [`create`], **plus** le refus d'un
/// cycle. Ordre des refus :
///
/// 1. le parent est le compte lui-même → [`DbError::AccountParentCycle`] ;
/// 2. absent de la société du compte → [`DbError::AccountParentNotFound`]
///    (la route l'a déjà refusé, même message que la création) ; archivé →
///    [`DbError::AccountParentArchived`] (même remarque) ;
/// 3. un sous-compte du compte, à quelque niveau → `AccountParentCycle` ;
/// 4. un compte de différence de change de la société, ou désigné compte
///    propre par un **autre** compte → `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT` ; un
///    compte d'actif ou de passif en devise mouvementé →
///    `ACCOUNT_FOREIGN_PARENT_HAS_ENTRIES` (comme [`create`]) ; tout autre compte
///    qui porte une ligne d'écriture → [`DbError::AccountParentHasEntries`]
///    (`ACCOUNT_PARENT_HAS_ENTRIES`, revue P1-2 — **plus strict que
///    [`create`]**, qui l'accepte) ;
/// 5. l'un des comptes propres **résultants** du compte → `FX_ACCOUNT_INVALID`
///    `notPostable` (il deviendrait non imputable — revue de code P2 de la
///    37-1, 37-1-CR-P2-8, appliquée à la création).
///
/// **Verrous** : le parent `FOR UPDATE` (il passera non imputable), puis
/// chacun de ses ancêtres en `LOCK IN SHARE MODE`, en remontant (puis les
/// désignations et les lignes d'écriture du parent, en lecture verrouillante) : lus
/// verrouillés, ils sont la dernière version commitée, si bien qu'un
/// déplacement concurrent qui fermerait le cycle par l'autre bout est vu, ou
/// s'interbloque avec celui-ci (InnoDB en annule un, que la route rejoue, et
/// le rejeu voit le cycle). La remontée est bornée par l'ensemble des comptes
/// déjà vus : une hiérarchie déjà cyclique en base (aucun chemin d'écriture ne
/// la produit) est signalée par `Invariant` plutôt que parcourue sans fin.
async fn guard_parent_change_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    state: &CompanyCurrencyState,
    company_id: i64,
    moved: Option<&Account>,
    new_parent_id: i64,
    own_fx: Option<OwnFxAccounts>,
) -> Result<(), DbError> {
    let parent: Option<(String, bool, Option<i64>)> = sqlx::query_as(
        "SELECT number, active, parent_id FROM accounts WHERE id = ? AND company_id = ? FOR UPDATE",
    )
    .bind(new_parent_id)
    .bind(company_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;
    let Some((parent_number, parent_active, mut ancestor)) = parent else {
        return Err(DbError::AccountParentNotFound);
    };
    if let Some(account) = moved
        && new_parent_id == account.id
    {
        return Err(DbError::AccountParentCycle { parent_number });
    }
    if !parent_active {
        return Err(DbError::AccountParentArchived { parent_number });
    }

    // Le cycle par les ancêtres (et leur verrouillage) ne vaut que pour un
    // compte existant déplacé : un groupe pas encore créé n'a pas de
    // sous-compte (Story 46-5a, AC-2).
    if let Some(account) = moved {
        let mut seen = std::collections::HashSet::from([new_parent_id]);
        while let Some(current) = ancestor {
            if current == account.id {
                return Err(DbError::AccountParentCycle { parent_number });
            }
            if !seen.insert(current) {
                return Err(DbError::Invariant(format!(
                    "hiérarchie de comptes cyclique en base au-dessus du compte {new_parent_id}"
                )));
            }
            ancestor = sqlx::query_scalar::<_, Option<i64>>(
                "SELECT parent_id FROM accounts WHERE id = ? LOCK IN SHARE MODE",
            )
            .bind(current)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_db_error)?
            .flatten();
        }
    }

    // Revue P1-4 : la désignation propre **du compte lui-même** se juge sur
    // l'état après le `PUT` (étape 5 ci-dessous, comptes propres résolus) ;
    // seules les désignations des **autres** comptes refusent ici. Sans compte
    // déplacé (groupe à créer), toutes refusent (`0` : aucun identifiant).
    let exclude_holder = moved.map_or(0, |a| a.id);
    let hits =
        fx_designations_excluding_in_tx(tx, company_id, state, new_parent_id, exclude_holder)
            .await?;
    refuse_designated(hits, |_| true)?;
    refuse_foreign_parent_with_entries_in_tx(tx, new_parent_id).await?;
    // Revue P1-2 (décision du responsable de l'epic) : un parent qui porte des
    // écritures, en quelque devise, passerait non imputable avec son solde, que
    // plus aucune saisie ne corrigerait — refusé. Après le refus propre à la
    // devise, qui garde son code. ⚠️ La création ne le refuse pas (écart
    // tracé) ; la route groupée, si (Story 46-5a).
    if has_journal_line_locked(tx, new_parent_id).await? {
        return Err(DbError::AccountParentHasEntries {
            parent_account_id: new_parent_id,
            parent_number,
        });
    }

    if let Some(own) = own_fx {
        for (field, id) in [
            (FxAccountField::Gain, own.gain),
            (FxAccountField::Loss, own.loss),
        ] {
            if id == new_parent_id {
                return Err(DbError::FxAccountInvalid {
                    field,
                    reason: FxAccountReason::NotPostable,
                    holder_account_id: moved.map(|a| a.id),
                });
            }
        }
    }
    Ok(())
}

/// Met à jour un compte actif (nom, type, rôle, postabilité, devise, comptes
/// propres et — depuis le correctif `fix-api-parent-compte` — parent).
/// Verrouillage optimiste + audit log (Story 3.5).
/// Retourne `IllegalStateTransition` si le compte est archivé.
///
/// # Parent (correctif `fix-api-parent-compte`)
///
/// `changes.parent` est à trois états (inchangé / racine / sous un compte),
/// résolu sous le `FOR UPDATE` du compte. Un parent qui change passe, après
/// les gardes de devise et avant les refus existants (rôle singleton, compte
/// bancaire lié, écritures), les contrôles de [`create`] — parent de la
/// société, actif, ni compte de différence de change, ni compte en devise
/// mouvementé, ni l'un des comptes propres du compte — **plus** le refus d'un
/// cycle ([`DbError::AccountParentCycle`]) et d'un parent qui porte des
/// écritures ([`DbError::AccountParentHasEntries`]) : voir
/// [`guard_parent_change_in_tx`].
/// Le nouveau parent devient non imputable (comme à la création) ; l'ancien
/// garde son état. Le compte déplacé garde sa postabilité, ses écritures et
/// son historique : seule la hiérarchie change. Verrous supplémentaires : le
/// nouveau parent `FOR UPDATE` **après** le compte, puis ses ancêtres en
/// lecture verrouillante — deux déplacements croisés qui fermeraient un cycle
/// se sérialisent (ou s'interbloquent, et la route les rejoue).
///
/// `confirm_retype` (Story 25-2-a, [#382], [#274]) : autorise le changement
/// d'`account_type` d'un compte **qui porte des écritures**. Sans lui, un tel
/// changement est refusé par [`DbError::AccountHasEntries`], qui en nomme
/// l'ampleur. ⚠️ Il ne dispense d'**aucune** autre garde — ni du verrou
/// optimiste, ni de l'interdit sur compte archivé.
///
/// Story 34-11a2 (#972, C-e341-31, C-e341-40) — la ligne `accounts` est prise
/// **`FOR UPDATE` en tête** (elle était lue sans verrou), puis un changement de
/// type est refusé par [`DbError::AccountLinkedToBankAccount`] si un compte
/// bancaire **non archivé** lié à ce compte n'admet plus le nouveau type (carte :
/// passif seulement ; compte bancaire : actif ou passif), avant la garde des
/// écritures. La création, le `PUT` et le `PATCH` d'un compte bancaire relisent
/// le type du compte lié `LOCK IN SHARE MODE` dans leur transaction
/// ([`type_in_share_mode`]) : les deux gestes se sérialisent sur cette ligne.
/// Un retypage qui passe le premier fait refuser le lien ; un lien qui passe le
/// premier est vu ici, sa transaction commitée avant que le `FOR UPDATE` ne
/// rende la main (la lecture des comptes bancaires est elle-même verrouillante,
/// donc la dernière version commitée). Ordre des verrous : `accounts` puis
/// `bank_accounts`, celui des routes bancaires.
///
/// # Devise et comptes de différence propres (Story 37-1, AC-4, AC-4 bis)
///
/// `changes.currency` et `changes.own_fx_accounts` sont à trois états
/// (inchangé / CHF ou par défaut / valeur), **résolus ici**, sous le
/// `FOR UPDATE` du compte. Ordre des refus, jugés sur l'**état résultant** :
/// compte introuvable ou archivé → version → no-op → `MULTI_CURRENCY_DISABLED`
/// → `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT` (devise posée, retypage hors du type de
/// la désignation, `postable = false`, rôle `DefaultRevenue`) →
/// `ACCOUNT_CURRENCY_LOCKED` (devise changée sur un compte qui porte une ligne)
/// → `ACCOUNT_CURRENCY_IN_USE` (devise posée sur un compte qui garde son rôle,
/// ou désigné par un site de configuration) → `ACCOUNT_IN_FOREIGN_CURRENCY`
/// (rôle posé sur un compte qui sera en devise) → `FX_ACCOUNT_INVALID` des
/// comptes propres → refus existants (rôle singleton, compte bancaire lié,
/// écritures).
///
/// **Verrous** : `company_id` (immuable) lu **sur le pool**, avant `begin()` —
/// jamais `LOCK IN SHARE MODE` sur la ligne du compte, qui se promouvrait en
/// exclusif ; puis `companies` en partage (en `FOR UPDATE`, **premier**
/// verrou, quand le geste porte des comptes propres), les réglages de
/// facturation en partage, le compte `FOR UPDATE` (son `company_id` relu est
/// comparé à la lecture hors transaction : écart → `NotFound`), puis, sous ce
/// verrou, les désignations, la ligne d'écriture et les liens en lecture
/// verrouillante.
pub async fn update(
    pool: &MySqlPool,
    id: i64,
    version: i32,
    user_id: i64,
    changes: AccountUpdate,
    confirm_retype: bool,
) -> Result<Account, DbError> {
    check_role_account_type(changes.role, changes.account_type)?;

    let company_id = company_of_account(pool, id).await?;

    let mut tx = pool.begin().await.map_err(map_db_error)?;
    let account = update_in_tx(
        &mut tx,
        company_id,
        id,
        version,
        user_id,
        changes,
        confirm_retype,
    )
    .await?;
    // Un no-op n'a rien écrit : le commit ne fait que rendre les verrous.
    tx.commit().await.map_err(map_db_error)?;
    Ok(account)
}

/// [`update`] dans la transaction de l'appelant (Story 46-5a, AC-1) : **tout**
/// le corps de la modification — verrous (dans le même ordre), gardes,
/// `UPDATE`, passage du nouveau parent non imputable, copie des cours, audit —,
/// sans `begin()` ni `commit()`. `company_id` est celui de l'appelant (lu hors
/// transaction par [`update`], ou la société de la route groupée) : le compte
/// relu `FOR UPDATE` dans une autre société rend `NotFound`.
///
/// Les refus sont rendus tels quels, dans le même ordre ; la transaction de
/// l'appelant s'annule à l'abandon. ⚠️ Le court-circuit **no-op** rend
/// `Ok(before)` **sans rien annuler** : la transaction de l'appelant peut
/// porter des écritures antérieures (groupes déjà créés et rangés par
/// [`apply_groups`]), qu'un `rollback` ici effacerait.
pub async fn update_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    id: i64,
    version: i32,
    user_id: i64,
    changes: AccountUpdate,
    confirm_retype: bool,
) -> Result<Account, DbError> {
    check_role_account_type(changes.role, changes.account_type)?;

    let company_lock = if matches!(changes.own_fx_accounts, OwnFxAccountsChange::Unchanged) {
        CompanyLock::Share
    } else {
        CompanyLock::ForUpdate
    };
    let state = lock_company_currency_in_tx(tx, company_id, company_lock).await?;
    lock_invoice_settings_share_in_tx(tx, company_id).await?;

    // Snapshot "before" AVANT l'UPDATE, dans la même transaction — sous
    // `FOR UPDATE` depuis la Story 34-11a2 (C-e341-40).
    let before_opt = sqlx::query_as::<_, Account>(FIND_BY_ID_FOR_UPDATE_SQL)
        .bind(id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?;

    let before = match before_opt {
        None => return Err(DbError::NotFound),
        Some(a) if a.company_id != company_id => return Err(DbError::NotFound),
        Some(a) if !a.active => {
            return Err(DbError::IllegalStateTransition(
                "impossible de modifier un compte archivé".into(),
            ));
        }
        Some(a) if a.version != version => return Err(DbError::OptimisticLockConflict),
        Some(a) => a,
    };

    // Code review 14-3a (D1/D2) : normaliser AVANT le court-circuit no-op, sinon
    // un payload demandant `postable: true` sur un compte de regroupement serait
    // vu comme un changement (puis normalisé) au lieu d'être un no-op.
    let changes = AccountUpdate {
        postable: effective_postable(
            changes.role,
            changes.postable,
            has_active_children(tx, id).await?,
        ),
        ..changes
    };

    // KF-004 : court-circuit no-op AVANT toute mutation. Depuis la Story
    // 34-11a2 (C-e341-40), `before` est lu sous `FOR UPDATE` : la course que
    // décrivait ici la note KF-004 (snapshot périmé d'un `SELECT` sans verrou,
    // cf. spec 7-3 §race-condition) n'existe plus — une transaction concurrente
    // commitée est vue, et sa version rend 409 plus haut.
    //
    // Story 46-5a (AC-1) : aucun `rollback` ici — la transaction est celle de
    // l'appelant, qui peut porter des écritures antérieures.
    if is_no_op_change(&before, &changes) {
        return Ok(before);
    }

    // Story 37-1 : devise et comptes propres résolus ICI, sous le `FOR UPDATE`
    // du compte (« absent = inchangé » ne se résout jamais par une lecture de
    // la route), puis les gardes de l'AC-4 dans leur ordre.
    let currency = changes.currency.resolve(before.currency.as_deref());
    let own_fx = changes.own_fx_accounts.resolve(before.own_fx_accounts());
    guard_currency_update(tx, &state, &before, &changes, currency.as_deref(), own_fx).await?;

    // Correctif `fix-api-parent-compte` : le parent résolu ici, sous le
    // `FOR UPDATE` du compte ; un parent qui change passe les contrôles de la
    // création, plus le refus d'un cycle.
    let parent_id = changes.parent.resolve(before.parent_id);
    if parent_id != before.parent_id
        && let Some(new_parent_id) = parent_id
    {
        guard_parent_change_in_tx(tx, &state, company_id, Some(&before), new_parent_id, own_fx)
            .await?;
    }

    // Story 14-3a : si le payload attribue un rôle singleton déjà porté par un
    // autre compte actif, échouer AVANT l'UPDATE pour pouvoir nommer ce compte.
    // La contrainte DB reste le filet (1062 sur course perdue).
    if let Some(role) = changes.role
        && before.role != Some(role)
        && let Some(holder) = find_singleton_role_holder(tx, before.company_id, role, id).await?
    {
        return Err(role_conflict(role, holder));
    }

    // Story 25-2-a ([#382], [#274]) — retyper un compte mouvementé reclasse tout
    // son historique, exercices CLOS compris : le résultat des années
    // antérieures change, donc le report à nouveau, donc le bilan d'ouverture de
    // l'exercice courant. Sans écriture pour l'expliquer.
    //
    // ⛔ **La place de cette garde est un critère de la story, pas un détail.**
    // Elle vient APRÈS le court-circuit no-op — renvoyer le même type n'est pas
    // un retypage — et APRÈS le contrôle de version — confirmer un retypage ne
    // doit jamais écraser la modification concurrente d'un tiers. Et AVANT
    // l'`UPDATE`, faute de quoi elle ne garderait rien.
    // ⚠️ L'ampleur est calculée **une fois**, et elle sert deux fois : à refuser
    // quand rien n'est confirmé, et à NOURRIR L'AUDIT quand le geste passe. Un
    // `account.retyped` qui ne dirait pas combien d'écritures ont basculé ne
    // vaudrait pas mieux qu'un `account.updated` — c'est l'ampleur qui fait la
    // différence pour qui relira le journal.
    let mut retype_trace: Option<(i64, Vec<String>)> = None;
    if before.account_type != changes.account_type {
        // Story 34-11a2 (C-e341-31) — un compte bancaire non archivé lié à ce
        // compte doit admettre le nouveau type ; sinon refus, qu'aucune
        // confirmation ne lève. Avant la garde des écritures.
        if let Some((bank_account_id, bank_account_name)) =
            first_linked_bank_account_refusing(tx, before.company_id, id, changes.account_type)
                .await?
        {
            return Err(DbError::AccountLinkedToBankAccount {
                bank_account_id,
                bank_account_name,
            });
        }
        let (entry_count, closed_fiscal_years) =
            retype_impact(&mut **tx, before.company_id, id).await?;
        if entry_count > 0 {
            if !confirm_retype {
                return Err(DbError::AccountHasEntries {
                    entry_count,
                    closed_fiscal_years,
                    from_type: before.account_type.as_str(),
                    to_type: changes.account_type.as_str(),
                });
            }
            retype_trace = Some((entry_count, closed_fiscal_years));
        }
    }

    let rows = sqlx::query(
        "UPDATE accounts SET name = ?, account_type = ?, role = ?, postable = ?, \
         currency = ?, fx_gain_account_id = ?, fx_loss_account_id = ?, parent_id = ?, \
         version = version + 1 WHERE id = ? AND version = ? AND active = TRUE",
    )
    .bind(&changes.name)
    .bind(changes.account_type)
    .bind(changes.role)
    .bind(changes.postable)
    .bind(&currency)
    .bind(own_fx.map(|o| o.gain))
    .bind(own_fx.map(|o| o.loss))
    .bind(parent_id)
    .bind(id)
    .bind(version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        // Défensif : ne devrait pas arriver puisqu'on a vérifié avant,
        // mais garde-fou contre une race theoretically possible entre
        // le SELECT et l'UPDATE (lecture repeatable InnoDB).
        return Err(DbError::OptimisticLockConflict);
    }

    // Correctif `fix-api-parent-compte` — comme à la création (code review
    // 14-3a, D2) : le nouveau parent acquiert un sous-compte, il devient un
    // compte de regroupement, non imputable, sans bump de version (conséquence
    // structurelle, non modification demandée sur le parent). L'ancien parent
    // garde son état, comme après l'archivage de son dernier sous-compte :
    // l'utilisateur le rend imputable par un `PUT` s'il le veut
    // ([`effective_postable`] l'admet dès qu'il n'a plus de sous-compte actif).
    if parent_id != before.parent_id
        && let Some(new_parent_id) = parent_id
    {
        sqlx::query("UPDATE accounts SET postable = FALSE WHERE id = ? AND company_id = ?")
            .bind(new_parent_id)
            .bind(before.company_id)
            .execute(&mut **tx)
            .await
            .map_err(map_db_error)?;
    }

    let after = sqlx::query_as::<_, Account>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_db_error)?;

    // Story 3.5 : audit log avec wrapper {before, after} pour update.
    // Même format {before, after} que `journal_entries::update` (rétablie par
    // la Story 15-8a, #532) et tous les `update` du dépôt.
    let mut audit_details = serde_json::json!({
        "before": account_snapshot_json(&before),
        "after": account_snapshot_json(&after),
    });

    // Story 37-2 (AC-3 bis) : un compte qui reçoit une devise (de CHF ou d'une
    // autre devise) reçoit la copie des cours déjà récupérés de cette devise,
    // pour chaque exercice multi-devises ouvert de la société, dans cette
    // transaction ; le nombre copié va au détail de l'audit (`copied`).
    if after.currency != before.currency
        && let Some(currency) = &after.currency
    {
        let copied = crate::repositories::exchange_rates::copy_currency_to_open_years_in_tx(
            tx,
            after.company_id,
            currency,
        )
        .await?;
        audit_details["copied"] = serde_json::json!(copied);
    }

    // Story 25-2-a ([#382], [#274]) — un retypage de compte MOUVEMENTÉ porte son
    // propre code d'action.
    //
    // ⛔ **Pourquoi un code distinct et non un `account.updated` de plus** : ce
    // geste ne modifie pas un compte, il **reclasse un historique**, exercices
    // clos compris. Noyé parmi les renommages et les changements de rôle, il
    // serait introuvable le jour où quelqu'un cherche pourquoi le résultat d'une
    // année close a bougé — et c'est précisément ce jour-là qu'on le cherche.
    //
    // ⚠️ Un compte **vierge** retypé reste un `account.updated` : sans écriture,
    // il n'y a pas d'historique à reclasser, donc rien à signaler.
    //
    // `account.retyped` est inscrit au registre `crates/kesh-api/src/audit_labels.rs`
    // et traduit dans les quatre locales — sans quoi le journal d'audit afficherait
    // le code brut, repli délibéré de la Story 25-1c-a. L'inscription a été faite
    // au merge de `main` dans la 25-2-a, le registre n'existant pas avant (#439).
    let action = match &retype_trace {
        Some((entry_count, closed_fiscal_years)) => {
            audit_details["retype"] = serde_json::json!({
                "fromType": before.account_type.as_str(),
                "toType": after.account_type.as_str(),
                "entryCount": entry_count,
                "closedFiscalYears": closed_fiscal_years,
            });
            "account.retyped"
        }
        None => "account.updated",
    };

    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::user(
            user_id,
            action.to_string(),
            "account".to_string(),
            id,
            Some(audit_details),
        ),
    )
    .await?;
    Ok(after)
}

/// Les contrôles de **forme** d'une demande groupée (Story 46-5a, AC-2),
/// avant toute transaction — `ACCOUNT_GROUP_INVALID` :
///
/// 1. plus de [`MAX_GROUP_PROPOSALS`] propositions (`groupIndex` = la première
///    en trop) ou plus de [`MAX_GROUP_MEMBERS`] membres (`groupIndex` = la
///    proposition qui franchit le plafond) → `tooMany` ;
/// 2. puis proposition par proposition, dans l'ordre reçu : sans membre →
///    `empty` ; membre par membre, dans l'ordre reçu, le compte même du
///    rattachement ou de la réutilisation → `memberIsOwnGroup`, un membre déjà
///    vu (dans cette proposition ou une précédente) → `accountInTwoGroups` ;
///    enfin, le compte du rattachement ou de la réutilisation membre d'une
///    **autre** proposition de la demande → `groupIsMember`, puis déjà visé par
///    un rattachement ou une réutilisation d'une proposition **précédente** →
///    `groupInTwoProposals` (deux `attach`, deux `reuse`, ou l'un et l'autre, du
///    même compte : revue P1, L2 — ses comptes se réunissent en une seule
///    proposition).
///
/// Fonction pure : la route l'appelle par [`apply_groups`].
pub fn check_groups_shape(groups: &[GroupProposal]) -> Result<(), DbError> {
    let invalid = |group_index, reason, account_id| DbError::AccountGroupInvalid {
        group_index,
        reason,
        account_id,
    };
    if groups.len() > MAX_GROUP_PROPOSALS {
        return Err(invalid(
            MAX_GROUP_PROPOSALS,
            AccountGroupReason::TooMany,
            None,
        ));
    }
    let mut total = 0usize;
    for (i, g) in groups.iter().enumerate() {
        total += g.members.len();
        if total > MAX_GROUP_MEMBERS {
            return Err(invalid(i, AccountGroupReason::TooMany, None));
        }
    }
    let all_members: std::collections::HashSet<i64> = groups
        .iter()
        .flat_map(|g| g.members.iter().map(|m| m.id))
        .collect();
    let mut seen = std::collections::HashSet::new();
    let mut targets = std::collections::HashSet::new();
    for (i, g) in groups.iter().enumerate() {
        if g.members.is_empty() {
            return Err(invalid(i, AccountGroupReason::Empty, None));
        }
        let target = g.kind.target();
        for m in &g.members {
            if target == Some(m.id) {
                return Err(invalid(i, AccountGroupReason::MemberIsOwnGroup, Some(m.id)));
            }
            if !seen.insert(m.id) {
                return Err(invalid(
                    i,
                    AccountGroupReason::AccountInTwoGroups,
                    Some(m.id),
                ));
            }
        }
        if let Some(t) = target
            && all_members.contains(&t)
        {
            return Err(invalid(i, AccountGroupReason::GroupIsMember, Some(t)));
        }
        if let Some(t) = target
            && !targets.insert(t)
        {
            return Err(invalid(i, AccountGroupReason::GroupInTwoProposals, Some(t)));
        }
    }
    Ok(())
}

/// Crée des comptes de groupe et y range des comptes, **en une seule
/// transaction, tout ou rien** (Story 46-5a, `POST /api/v1/accounts/groups`,
/// AC-2) : toute erreur annule la demande entière — aucun compte créé, déplacé
/// ni rendu non imputable, aucune entrée d'audit.
///
/// # Déroulé, et ordre des refus
///
/// 0. **Forme**, avant `begin()` : [`check_groups_shape`].
/// 1. **Verrous de tête** : `companies` en partage, les réglages de facturation
///    en partage (comme [`create`] et [`update`], qui les reprennent ensuite
///    sans effet), puis **tous** les membres et comptes de rattachement ou de
///    réutilisation de la demande `FOR UPDATE`, **en une passe**, par `id`
///    croissant sur toute la demande, par une lecture **filtrée par la
///    société** : un identifiant absent de la société → `NotFound`, le plus
///    petit d'abord, avant tout autre contrôle.
/// 2. **Membres**, par `id` croissant : archivé → `IllegalStateTransition` (le
///    refus d'[`update`]), puis `version` différente de celle de la demande →
///    `OptimisticLockConflict` — avant les contrôles d'état : un membre retypé
///    depuis l'aperçu rend `409`, non `mixedTypes`.
/// 3. **État**, **phase entière avant toute écriture**, proposition par
///    proposition dans l'ordre reçu, sur les lignes verrouillées : membres de
///    types différents → `mixedTypes` (le premier membre, dans l'ordre reçu,
///    d'un autre type que le premier) ; puis, pour une création, numéro déjà
///    porté (archivé compris, lu `FOR UPDATE` sur `(company_id, number)`) ou
///    déjà demandé par une création précédente (comparé sans casse) → `numberTaken` ; pour un
///    rattachement, compte inactif ou sans sous-compte actif → `notAGroup` ;
///    pour une réutilisation, compte inactif ou avec un sous-compte actif →
///    `notReusable` ; puis, pour ces deux, type différent de celui des membres
///    → `groupTypeMismatch`.
/// 4. **Écritures**, proposition par proposition dans l'ordre reçu : une
///    création passe d'abord le contrôle du parent du **changement de parent**
///    ([`guard_parent_change_in_tx`], sans compte déplacé : parent de la
///    société, actif, ni compte de différence de change, **ni mouvementé** —
///    `ACCOUNT_PARENT_HAS_ENTRIES`, plus strict que [`create`], écart A-1 de
///    `fix-api-parent-compte`), puis [`create_in_tx`] (type des membres, devise
///    principale, aucun rôle, non imputable) ; puis chaque membre, par `id`
///    croissant, passe par [`update_in_tx`], ligne verrouillée inchangée hors
///    du parent (`ParentChange::Under`, `confirm_retype = false`) — un compte
///    réutilisé qui porte des écritures est refusé là, sous verrou
///    (`ACCOUNT_PARENT_HAS_ENTRIES`), et un membre déjà sous son groupe est un
///    no-op (ni `version`, ni audit).
/// 5. **Relecture**, avant le commit : les comptes réutilisés et déplacés sont
///    relus après la dernière écriture — la réponse rend l'état que le commit
///    publie (un compte déplacé devenu parent d'un groupe créé ensuite y est
///    non imputable).
///
/// L'erreur porte l'étape ([`GroupStepError`]) : la proposition, et le compte
/// quand l'étape en vise un (membre, compte de rattachement ou de
/// réutilisation ; aucun pour la création du groupe).
///
/// # Verrous
///
/// `companies` → réglages de facturation → tous les comptes de la demande
/// (`FOR UPDATE`, `id` croissant, par la clé primaire) → numéros des groupes à
/// créer (`FOR UPDATE`, verrou d'intervalle pour un numéro absent) → parent
/// d'un groupe créé (`FOR UPDATE`, puis ses désignations et ses lignes en
/// lecture verrouillante) → les verrous d'[`update`] pour chaque membre (son
/// nouveau parent `FOR UPDATE`, ses ancêtres en partage). Deux demandes croisées
/// s'interbloquent moins par la passe de tête ; l'interblocage restant (verrous
/// d'intervalle de numéros voisins, notamment) est rejoué par la route, la
/// transaction entière à chaque tentative.
///
/// # Audit
///
/// Aucune action neuve : `account.created` par groupe créé, `account.updated`
/// par compte déplacé, écrites dans la transaction par [`create_in_tx`] et
/// [`update_in_tx`]. Un groupe réutilisé ou rattaché, qui passe non imputable
/// sans hausse de `version`, n'a pas d'entrée propre.
pub async fn apply_groups(
    pool: &MySqlPool,
    company_id: i64,
    user_id: i64,
    groups: &[GroupProposal],
) -> Result<GroupsOutcome, GroupStepError> {
    check_groups_shape(groups).map_err(GroupStepError::bare)?;
    if groups.is_empty() {
        return Ok(GroupsOutcome::default());
    }

    let bare = GroupStepError::bare;
    let mut tx = pool.begin().await.map_err(|e| bare(map_db_error(e)))?;
    let state = lock_company_currency_in_tx(&mut tx, company_id, CompanyLock::Share)
        .await
        .map_err(bare)?;
    lock_invoice_settings_share_in_tx(&mut tx, company_id)
        .await
        .map_err(bare)?;

    // La proposition de chaque compte de la demande (membre ou compte d'un
    // rattachement ou d'une réutilisation) — unique, la forme l'a garanti.
    let mut owner: std::collections::BTreeMap<i64, usize> = std::collections::BTreeMap::new();
    for (i, g) in groups.iter().enumerate() {
        for id in g.members.iter().map(|m| m.id).chain(g.kind.target()) {
            owner.entry(id).or_insert(i);
        }
    }

    // 1. La passe de verrous : une lecture, par la clé primaire, filtrée par la
    //    société, `id` croissant.
    let placeholders = vec!["?"; owner.len()].join(", ");
    let sql = format!(
        "SELECT {COLUMNS} FROM accounts FORCE INDEX (PRIMARY) \
         WHERE company_id = ? AND id IN ({placeholders}) ORDER BY id FOR UPDATE"
    );
    let mut query = sqlx::query_as::<_, Account>(&sql).bind(company_id);
    for id in owner.keys() {
        query = query.bind(*id);
    }
    let locked: std::collections::BTreeMap<i64, Account> = query
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| bare(map_db_error(e)))?
        .into_iter()
        .map(|a| (a.id, a))
        .collect();
    if let Some((&id, &i)) = owner.iter().find(|(id, _)| !locked.contains_key(id)) {
        return Err(GroupStepError::at(i, Some(id), DbError::NotFound));
    }

    // 2. Les membres, par `id` croissant : archivé, puis `version`.
    let mut requested: std::collections::BTreeMap<i64, (usize, i32)> =
        std::collections::BTreeMap::new();
    for (i, g) in groups.iter().enumerate() {
        for m in &g.members {
            requested.insert(m.id, (i, m.version));
        }
    }
    for (&id, &(i, version)) in &requested {
        let account = &locked[&id];
        if !account.active {
            return Err(GroupStepError::at(
                i,
                Some(id),
                DbError::IllegalStateTransition("impossible de modifier un compte archivé".into()),
            ));
        }
        if account.version != version {
            return Err(GroupStepError::at(
                i,
                Some(id),
                DbError::OptimisticLockConflict,
            ));
        }
    }

    // 3. L'état, proposition par proposition, avant toute écriture.
    let invalid = |i: usize, reason, account_id: Option<i64>| {
        GroupStepError::bare(DbError::AccountGroupInvalid {
            group_index: i,
            reason,
            account_id,
        })
    };
    let mut group_types = Vec::with_capacity(groups.len());
    // Les numéros des créations précédentes, comparés **sans casse** (revue P1,
    // L1) : `accounts.number` est sous une collation insensible à la casse, et
    // deux créations `10a` et `10A` se heurteraient à l'index unique (`409
    // UNIQUE_CONSTRAINT_VIOLATION`) au lieu du motif promis. Les autres
    // équivalences de la collation (accents) restent couvertes par l'index
    // unique, tout ou rien tenu.
    let mut numbers_to_create = std::collections::HashSet::new();
    for (i, g) in groups.iter().enumerate() {
        let first_type = locked[&g.members[0].id].account_type;
        if let Some(other) = g
            .members
            .iter()
            .find(|m| locked[&m.id].account_type != first_type)
        {
            return Err(invalid(i, AccountGroupReason::MixedTypes, Some(other.id)));
        }
        match &g.kind {
            GroupKind::Create { number, .. } => {
                let holder: Option<i64> = sqlx::query_scalar(
                    "SELECT id FROM accounts WHERE company_id = ? AND number = ? FOR UPDATE",
                )
                .bind(company_id)
                .bind(number)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| GroupStepError::at(i, None, map_db_error(e)))?;
                if holder.is_some() || !numbers_to_create.insert(number.to_lowercase()) {
                    return Err(invalid(i, AccountGroupReason::NumberTaken, holder));
                }
            }
            GroupKind::Attach { account_id } | GroupKind::Reuse { account_id } => {
                let target = &locked[account_id];
                let has_children = has_active_children(&mut tx, *account_id)
                    .await
                    .map_err(|e| GroupStepError::at(i, Some(*account_id), e))?;
                let refused = match g.kind {
                    GroupKind::Attach { .. } if !target.active || !has_children => {
                        Some(AccountGroupReason::NotAGroup)
                    }
                    GroupKind::Reuse { .. } if !target.active || has_children => {
                        Some(AccountGroupReason::NotReusable)
                    }
                    _ => None,
                };
                if let Some(reason) = refused {
                    return Err(invalid(i, reason, Some(*account_id)));
                }
                if target.account_type != first_type {
                    return Err(invalid(
                        i,
                        AccountGroupReason::GroupTypeMismatch,
                        Some(*account_id),
                    ));
                }
            }
        }
        group_types.push(first_type);
    }

    // 4. Les écritures.
    let mut outcome = GroupsOutcome::default();
    let mut reused_ids = std::collections::BTreeSet::new();
    let mut moved_ids = std::collections::BTreeSet::new();
    for (i, g) in groups.iter().enumerate() {
        let group_id = match &g.kind {
            GroupKind::Create {
                number,
                name,
                parent_id,
            } => {
                if let Some(parent_id) = *parent_id {
                    guard_parent_change_in_tx(&mut tx, &state, company_id, None, parent_id, None)
                        .await
                        .map_err(|e| GroupStepError::at(i, None, e))?;
                }
                let group = create_in_tx(
                    &mut tx,
                    user_id,
                    NewAccount {
                        company_id,
                        number: number.clone(),
                        name: name.clone(),
                        account_type: group_types[i],
                        parent_id: *parent_id,
                        role: None,
                        postable: false,
                        currency: None,
                        own_fx_accounts: None,
                    },
                )
                .await
                .map_err(|e| GroupStepError::at(i, None, e))?;
                let id = group.id;
                outcome.created.push(group);
                id
            }
            GroupKind::Attach { account_id } => *account_id,
            GroupKind::Reuse { account_id } => {
                reused_ids.insert(*account_id);
                *account_id
            }
        };

        let mut members: Vec<GroupMember> = g.members.clone();
        members.sort_by_key(|m| m.id);
        for m in members {
            let row = &locked[&m.id];
            let changes = AccountUpdate {
                name: row.name.clone(),
                account_type: row.account_type,
                role: row.role,
                postable: row.postable,
                currency: CurrencyChange::Unchanged,
                own_fx_accounts: OwnFxAccountsChange::Unchanged,
                parent: ParentChange::Under(group_id),
            };
            let after = update_in_tx(
                &mut tx, company_id, m.id, m.version, user_id, changes, false,
            )
            .await
            .map_err(|e| GroupStepError::at(i, Some(m.id), e))?;
            if after.version != m.version {
                moved_ids.insert(m.id);
            }
        }
    }

    // Les comptes réutilisés et les comptes déplacés relus **après la dernière
    // écriture**, avant le commit (revue P1, M1) : un compte déplacé par une
    // proposition peut devenir le parent d'un groupe créé par une proposition
    // suivante, qui le passe non imputable sans hausser sa `version` ; la ligne
    // rendue par `update_in_tx` serait alors périmée. Un compte réutilisé passe
    // non imputable au rangement de son premier membre.
    outcome.reused = reread_in_tx(&mut tx, &reused_ids).await.map_err(bare)?;
    outcome.moved = reread_in_tx(&mut tx, &moved_ids).await.map_err(bare)?;

    tx.commit().await.map_err(|e| bare(map_db_error(e)))?;
    outcome.created.sort_by_key(|a| a.id);
    Ok(outcome)
}

/// Les comptes `ids`, relus dans la transaction, par `id` croissant (l'ordre
/// du `BTreeSet`) : l'état que le commit publiera.
async fn reread_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    ids: &std::collections::BTreeSet<i64>,
) -> Result<Vec<Account>, DbError> {
    let mut accounts = Vec::with_capacity(ids.len());
    for id in ids {
        let account = sqlx::query_as::<_, Account>(FIND_BY_ID_SQL)
            .bind(id)
            .fetch_one(&mut **tx)
            .await
            .map_err(map_db_error)?;
        accounts.push(account);
    }
    Ok(accounts)
}

/// Archive un compte (active = false). Verrouillage optimiste + audit log (Story 3.5).
/// Retourne `IllegalStateTransition` si le compte a des sous-comptes actifs.
///
/// Sans l'avertissement de [`archive_guarded`], que seule la route emploie : c'est
/// la forme des montages de tests (`confirm_designated = true`).
pub async fn archive(
    pool: &MySqlPool,
    id: i64,
    version: i32,
    user_id: i64,
) -> Result<Account, DbError> {
    archive_guarded(pool, id, version, user_id, true).await
}

/// Archive un compte, et refuse sans confirmation un compte désigné dans les
/// paramètres de facturation (Story 33-8, #486) : `AccountDesignatedInSettings`,
/// qui nomme les réglages et ne consomme pas la version. Confirmé, le réglage reste
/// tel quel, et l'audit nomme ce qui le désignait.
///
/// **Ordre des refus (C-e334-29)** : sous-comptes actifs, désignation, version. La
/// désignation se lit sur l'état courant, juste avant l'`UPDATE` qui contrôle la
/// version : un écran périmé non confirmé reçoit donc d'abord le refus de désignation,
/// puis, s'il confirme, le conflit de version.
///
/// La société est celle du compte, lue ici : aucun appelant ne peut en faire lire
/// une autre (revue P1, A L-5).
///
/// ⚠️ **Limite assumée (C-e334-45)** : un `PUT` concurrent des paramètres, qui a
/// validé le compte encore actif, peut le désigner juste après cette lecture. La
/// verrouiller ne suffirait pas : il faudrait aussi que le `PUT` verrouille le compte
/// qu'il valide. Le compte reste alors archivé et désigné, et l'écriture qui l'emploie
/// est refusée : l'écart se voit à l'usage, il n'écrit rien de faux.
///
/// # Compte de différence de change (Story 37-1, AC-4, AC-4 bis)
///
/// Un compte désigné gains, pertes ou conversion — par la société, ou comme
/// compte propre d'un compte en devise, archivé compris — ne s'archive pas :
/// `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`, **sans recours** (`confirm_designated`
/// ne le lève pas). Ordre des refus : sous-comptes actifs → gain / perte /
/// conversion → désignation de facturation → version.
///
/// **Verrous** : `company_id` lu sur le pool avant `begin()` (immuable ; voir
/// [`update`]), puis `companies` en partage, les réglages de facturation en
/// partage, le compte `FOR UPDATE` (`company_id` relu comparé), puis les
/// désignations propres en lecture verrouillante par index : deux archivages
/// concurrents de comptes différents ne se verrouillent pas mutuellement.
pub async fn archive_guarded(
    pool: &MySqlPool,
    id: i64,
    version: i32,
    user_id: i64,
    confirm_designated: bool,
) -> Result<Account, DbError> {
    let company_id = company_of_account(pool, id).await?;

    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let state = lock_company_currency_in_tx(&mut tx, company_id, CompanyLock::Share).await?;
    lock_invoice_settings_share_in_tx(&mut tx, company_id).await?;

    let locked_company: Option<i64> =
        sqlx::query_scalar("SELECT company_id FROM accounts WHERE id = ? FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_db_error)?;
    if locked_company != Some(company_id) {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::NotFound);
    }

    // Vérifier que le compte n'a pas d'enfants actifs
    let children: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM accounts WHERE parent_id = ? AND active = TRUE")
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_db_error)?;
    if children.0 > 0 {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::IllegalStateTransition(
            "impossible d'archiver un compte avec des sous-comptes actifs".into(),
        ));
    }

    // Story 37-1 — un compte de différence de change ne s'archive pas, et
    // aucune confirmation n'y change rien.
    let hits = fx_designations_of_in_tx(&mut tx, company_id, &state, id).await?;
    refuse_designated(hits, |_| true)?;

    let designated =
        company_invoice_settings::designations_of_account(&mut *tx, company_id, id).await?;
    if !designated.is_empty() && !confirm_designated {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::AccountDesignatedInSettings {
            settings: designated,
        });
    }

    let rows = sqlx::query(
        "UPDATE accounts SET active = FALSE, version = version + 1 \
         WHERE id = ? AND version = ?",
    )
    .bind(id)
    .bind(version)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        tx.rollback().await.map_err(map_db_error)?;
        let exists = sqlx::query_as::<_, Account>(FIND_BY_ID_SQL)
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(map_db_error)?;
        return if exists.is_some() {
            Err(DbError::OptimisticLockConflict)
        } else {
            Err(DbError::NotFound)
        };
    }

    let account = sqlx::query_as::<_, Account>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_error)?;

    // Story 3.5 : audit log (snapshot direct, cohérent avec create/delete).
    // Rollback explicite en cas d'erreur pour cohérence stylistique avec
    // les autres branches de la fonction (le Drop de tx rollback déjà
    // implicitement, mais être explicite évite tout ambiguïté).
    let mut details = account_snapshot_json(&account);
    if !designated.is_empty() {
        details["designatedSettings"] = serde_json::json!(designated);
    }
    if let Err(e) = audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            user_id,
            "account.archived".to_string(),
            "account".to_string(),
            id,
            Some(details),
        ),
    )
    .await
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(e);
    }

    tx.commit().await.map_err(map_db_error)?;
    Ok(account)
}

/// Cherche le compte **actif** qui porte déjà `role` dans la société, en
/// excluant `exclude_id` (le compte qu'on est en train d'écrire).
///
/// Retourne `None` si le rôle est libre, ou si `role` n'est pas singleton (dans
/// ce cas plusieurs comptes peuvent légitimement le porter).
///
/// Sert **uniquement** à produire un message d'erreur qui nomme le compte en
/// conflit : la contrainte `uq_accounts_company_singleton_role` reste la source
/// de vérité et rattrape les courses perdues (1062).
async fn find_singleton_role_holder(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    role: AccountRole,
    exclude_id: i64,
) -> Result<Option<Account>, DbError> {
    if !role.is_singleton() {
        return Ok(None);
    }
    sqlx::query_as::<_, Account>(&format!(
        "SELECT {COLUMNS} FROM accounts \
         WHERE company_id = ? AND role = ? AND active = TRUE AND id <> ? LIMIT 1"
    ))
    .bind(company_id)
    .bind(role)
    .bind(exclude_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)
}

/// Construit l'erreur « rôle déjà attribué » à partir du compte détenteur.
fn role_conflict(role: AccountRole, holder: Account) -> DbError {
    DbError::AccountRoleAlreadyAssigned {
        role: role.as_str().to_string(),
        account_id: holder.id,
        account_number: holder.number,
        account_name: holder.name,
    }
}

/// Réactive un compte archivé (`active = false` → `true`). Story 14-3a, issue #269.
///
/// Symétrique de [`archive`] : verrouillage optimiste + audit `account.reactivated`
/// dans la même transaction.
///
/// # Gardes
///
/// - **Parent archivé** → `IllegalStateTransition`. Miroir du garde-fou de
///   [`archive`] (qui refuse d'archiver un compte ayant des enfants actifs) :
///   sans lui on obtiendrait un compte actif sous un parent inactif.
/// - **Rôle singleton repris entre-temps** → [`DbError::AccountRoleAlreadyAssigned`].
///   La réactivation fait repasser la colonne générée `singleton_role` de `NULL`
///   à sa valeur et heurterait le UNIQUE (1062) ; on le détecte en amont pour
///   pouvoir nommer le compte qui bloque.
/// - **Compte déjà actif** → no-op idempotent : entité retournée telle quelle,
///   sans bump de version ni audit (cohérent avec le court-circuit KF-004 d'[`update`]).
/// - **Parent désigné compte de gains, de pertes ou de conversion** (Story 37-1,
///   revue de code P2, 37-1-CR-P2-7) → [`DbError::AccountIsFxDifferenceAccount`] :
///   un compte désigné ne reçoit pas de sous-compte (AC-4), et réactiver un
///   enfant archivé lui en rendrait un — l'enfant archivé laisse le parent
///   repasser imputable, puis être désigné. La société est lue sur le pool
///   (`company_id` immuable), puis `companies` en partage **en premier verrou**
///   (ordre global ; une désignation tient `companies` en exclusif), le parent en
///   partage, et ses désignations par les recherches verrouillantes de
///   [`fx_designations_of_in_tx`].
///
/// # `clear_role` — code review 14-3a, décision D4
///
/// Un compte archivé conserve son rôle, mais [`update`] refuse de modifier un
/// compte inactif : si le rôle a été repris entre-temps, l'utilisateur était
/// coincé dans une séquence en quatre étapes (retirer le rôle du détenteur,
/// réactiver, retirer le rôle de l'ancien, le rendre au détenteur) que rien
/// n'indiquait. Avec `clear_role = true`, la réactivation **retire** le rôle du
/// compte au lieu d'échouer — le garde ci-dessus devient sans objet.
pub async fn reactivate(
    pool: &MySqlPool,
    id: i64,
    version: i32,
    user_id: i64,
    clear_role: bool,
) -> Result<Account, DbError> {
    // Story 37-1 (37-1-CR-P2-7) — la société sur le pool, avant `begin()` : une
    // lecture simple en tête de transaction figerait l'instantané avant
    // l'attente du verrou de `companies`.
    let company_id = company_of_account(pool, id).await?;
    let mut tx = pool.begin().await.map_err(map_db_error)?;
    let state = lock_company_currency_in_tx(&mut tx, company_id, CompanyLock::Share).await?;

    let before = match sqlx::query_as::<_, Account>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?
    {
        None => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::NotFound);
        }
        Some(a) if a.company_id == company_id => a,
        Some(_) => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::NotFound);
        }
    };

    // Code review 14-3a : le contrôle de version passe AVANT le court-circuit
    // no-op — dans l'autre ordre, réactiver avec une version périmée un compte
    // qu'un tiers a déjà réactivé puis modifié retournait 200 avec une entité
    // que le client ne connaissait pas, au lieu du 409 que toutes les autres
    // mutations de la ressource renvoient. Ordre aligné sur [`update`].
    if before.version != version {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::OptimisticLockConflict);
    }

    // No-op idempotent : déjà actif → on ne touche à rien.
    if before.active {
        tx.rollback().await.map_err(map_db_error)?;
        return Ok(before);
    }

    // Garde 1 : le parent doit être actif (sinon arborescence incohérente).
    if let Some(parent_id) = before.parent_id {
        let parent: Option<(bool, String)> =
            sqlx::query_as("SELECT active, number FROM accounts WHERE id = ? LOCK IN SHARE MODE")
                .bind(parent_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(map_db_error)?;
        let (parent_active, parent_number) = parent.unwrap_or((false, parent_id.to_string()));
        if !parent_active {
            tx.rollback().await.map_err(map_db_error)?;
            // Variante dédiée, et non `IllegalStateTransition` : ce dernier est
            // mappé sur un message générique côté API, si bien que l'explication
            // écrite ici n'atteignait jamais l'utilisateur (code review 14-3a).
            return Err(DbError::AccountParentArchived { parent_number });
        }
        // Story 37-1 (37-1-CR-P2-7) : un parent désigné ne retrouve pas de
        // sous-compte actif.
        let hits = fx_designations_of_in_tx(&mut tx, company_id, &state, parent_id).await?;
        if let Err(e) = refuse_designated(hits, |_| true) {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(e);
        }
    }

    // Garde 2 : le rôle singleton du compte a-t-il été repris pendant l'archivage ?
    // Sans objet si l'appelant a demandé à réactiver sans le rôle (D4).
    if !clear_role
        && let Some(role) = before.role
        && let Some(holder) =
            find_singleton_role_holder(&mut tx, before.company_id, role, id).await?
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(role_conflict(role, holder));
    }

    let rows = sqlx::query(if clear_role {
        "UPDATE accounts SET active = TRUE, role = NULL, version = version + 1 \
         WHERE id = ? AND version = ? AND active = FALSE"
    } else {
        "UPDATE accounts SET active = TRUE, version = version + 1 \
         WHERE id = ? AND version = ? AND active = FALSE"
    })
    .bind(id)
    .bind(version)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        // Défensif : une tx concurrente a modifié la ligne entre le SELECT et l'UPDATE.
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::OptimisticLockConflict);
    }

    let after = sqlx::query_as::<_, Account>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_error)?;

    if let Err(e) = audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            user_id,
            "account.reactivated".to_string(),
            "account".to_string(),
            id,
            Some(account_snapshot_json(&after)),
        ),
    )
    .await
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(e);
    }

    tx.commit().await.map_err(map_db_error)?;
    Ok(after)
}

/// Crée plusieurs comptes dans une transaction unique.
///
/// Les `NewAccount` doivent avoir `parent_id` déjà résolu (ID réel ou None).
/// Pour le chargement depuis les fichiers JSON (résolution `parent_number` → `parent_id`),
/// utiliser `bulk_create_from_chart` qui gère le tri topologique et la résolution.
///
/// Soit tous les comptes sont créés, soit aucun (rollback complet).
pub async fn bulk_create(
    pool: &MySqlPool,
    accounts: Vec<NewAccount>,
) -> Result<Vec<Account>, DbError> {
    if accounts.is_empty() {
        return Ok(vec![]);
    }

    let mut tx = pool.begin().await.map_err(map_db_error)?;
    let mut created_ids: Vec<i64> = Vec::with_capacity(accounts.len());

    for new in &accounts {
        // Story 37-1 : la devise et les comptes propres sont liés, non omis —
        // un `INSERT` qui les ignorerait les perdrait en silence (les appelants
        // passent `None` : comptes neufs en CHF).
        let result = sqlx::query(
            "INSERT INTO accounts (company_id, number, name, account_type, parent_id, role, postable, \
                 currency, fx_gain_account_id, fx_loss_account_id) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(new.company_id)
        .bind(&new.number)
        .bind(&new.name)
        .bind(new.account_type)
        .bind(new.parent_id)
        .bind(new.role)
        .bind(new.postable)
        .bind(&new.currency)
        .bind(new.own_fx_accounts.map(|o| o.gain))
        .bind(new.own_fx_accounts.map(|o| o.loss))
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;

        let last_id = result.last_insert_id();
        if last_id == 0 {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::Invariant(
                "last_insert_id == 0 après INSERT accounts (bulk)".into(),
            ));
        }
        let id = i64::try_from(last_id).map_err(|_| {
            DbError::Invariant(format!("last_insert_id {last_id} dépasse i64::MAX"))
        })?;

        created_ids.push(id);
    }

    // Récupérer tous les comptes créés
    let placeholders = created_ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(",");
    let sql =
        format!("SELECT {COLUMNS} FROM accounts WHERE id IN ({placeholders}) ORDER BY number");
    let mut query = sqlx::query_as::<_, Account>(&sql);
    for id in &created_ids {
        query = query.bind(id);
    }
    let result = query.fetch_all(&mut *tx).await.map_err(map_db_error)?;

    tx.commit().await.map_err(map_db_error)?;
    Ok(result)
}

/// Crée les comptes d'un plan comptable dans une transaction unique.
///
/// Enveloppe pool de [`bulk_create_from_chart_in_tx`] : `begin`, variante,
/// `commit`. Sur erreur, rollback *best-effort* puis l'erreur d'origine
/// (Story 15-7a1). Une liste vide court-circuite **avant** `begin`.
///
/// **Cette fonction n'écrit pas elle-même d'entrée d'audit** : elle
/// n'emprunte pas le chemin `create` audité (FR88, Story 3.5). C'est à
/// l'appelant de tracer — l'onboarding de production écrit
/// `account.chart_loaded` autour de la variante `_in_tx` (Story 15-7a2) ; le
/// seed de démonstration ne trace pas le plan par fait de domaine : sa
/// synthèse `installation.demo_seeded` en porte le nombre de comptes (Story
/// 15-7b1).
pub async fn bulk_create_from_chart(
    pool: &MySqlPool,
    company_id: i64,
    entries: &[kesh_core::chart_of_accounts::ChartEntry],
    lang: &str,
) -> Result<Vec<Account>, DbError> {
    if entries.is_empty() {
        return Ok(vec![]);
    }

    let mut tx = pool.begin().await.map_err(map_db_error)?;
    match bulk_create_from_chart_in_tx(&mut tx, company_id, entries, lang).await {
        Ok(accounts) => {
            tx.commit().await.map_err(map_db_error)?;
            Ok(accounts)
        }
        Err(e) => {
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

/// Variante transactionnelle de [`bulk_create_from_chart`] (Story 15-7a1) :
/// crée les comptes du plan **dans la transaction de l'appelant**.
///
/// Prend les ChartEntry bruts et résout la hiérarchie parent_number → parent_id
/// en insérant en ordre topologique (tri par longueur de numéro, puis numéro).
///
/// `lang` : code langue lowercase (ex: "fr") pour extraire le nom du compte.
///
/// Rend les comptes **réellement insérés**, triés par numéro ; une liste vide
/// rend `Ok(vec![])` sans requête. **Ne commite ni n'annule jamais** : sur
/// erreur, la variante rend l'erreur et c'est l'appelant qui annule.
///
/// **Cette fonction n'écrit pas elle-même d'entrée d'audit** : l'appelant
/// onboarding (`accounting-language`) écrit `account.chart_loaded` dans la
/// même transaction, à partir du `Vec` rendu (Story 15-7a2).
pub async fn bulk_create_from_chart_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    entries: &[kesh_core::chart_of_accounts::ChartEntry],
    lang: &str,
) -> Result<Vec<Account>, DbError> {
    if entries.is_empty() {
        return Ok(vec![]);
    }

    // Trier par longueur de numéro puis par numéro pour ordre topologique
    let mut sorted: Vec<_> = entries.iter().collect();
    sorted.sort_by(|a, b| {
        a.number
            .len()
            .cmp(&b.number.len())
            .then(a.number.cmp(&b.number))
    });

    // Numéros qui sont parents d'au moins une entrée → comptes titres, non-postables.
    let parent_numbers = kesh_core::chart_of_accounts::parent_numbers(entries);

    let mut number_to_id: std::collections::HashMap<&str, i64> = std::collections::HashMap::new();
    let mut created_ids: Vec<i64> = Vec::with_capacity(entries.len());

    for entry in &sorted {
        let name = kesh_core::chart_of_accounts::resolve_name(entry, lang);
        let parent_id = entry
            .parent_number
            .as_deref()
            .and_then(|pn| number_to_id.get(pn).copied());

        // Story 14-3a : le rôle vient du plan JSON, la postabilité est calculée
        // par la MÊME fonction pure que celle testée contre le backfill SQL de
        // la migration (invariant « seed ≡ backfill »).
        let role = entry.role.map(|r| r.as_str());
        let postable = kesh_core::chart_of_accounts::is_postable(entry, &parent_numbers);

        // Story 37-1 : un compte d'un plan livré est en CHF, sans comptes de
        // différence propres — `NULL` écrit explicitement.
        let result = sqlx::query(
            "INSERT INTO accounts (company_id, number, name, account_type, parent_id, role, postable, \
                 currency, fx_gain_account_id, fx_loss_account_id) \
             VALUES (?, ?, ?, ?, ?, ?, ?, NULL, NULL, NULL)",
        )
        .bind(company_id)
        .bind(&entry.number)
        .bind(&name)
        .bind(entry.account_type.as_str())
        .bind(parent_id)
        .bind(role)
        .bind(postable)
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?;

        let last_id = result.last_insert_id();
        if last_id == 0 {
            return Err(DbError::Invariant(
                "last_insert_id == 0 après INSERT accounts (bulk_chart)".into(),
            ));
        }
        let id = i64::try_from(last_id).map_err(|_| {
            DbError::Invariant(format!("last_insert_id {last_id} dépasse i64::MAX"))
        })?;

        number_to_id.insert(&entry.number, id);
        created_ids.push(id);
    }

    let placeholders = created_ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(",");
    let sql =
        format!("SELECT {COLUMNS} FROM accounts WHERE id IN ({placeholders}) ORDER BY number");
    let mut query = sqlx::query_as::<_, Account>(&sql);
    for id in &created_ids {
        query = query.bind(id);
    }
    query.fetch_all(&mut **tx).await.map_err(map_db_error)
}

// ---------------------------------------------------------------------------
// Story 37-1 (AC-3, AC-4 bis) — comptes de différence de change
// ---------------------------------------------------------------------------

/// Les comptes de différence de change d'un compte en devise (Story 37-1,
/// AC-4 bis, « résolution unique ») : ses comptes **propres** s'il en porte,
/// sinon ceux de la **société** (`companies.fx_gain_account_id` et
/// `fx_loss_account_id`). `None` pour un compte en CHF (sans devise), ou si la
/// société n'en désigne pas (état hérité).
///
/// Fonction **pure**, sans I/O : **seule** fonction qui choisit ces comptes côté
/// serveur. La réévaluation (37-5) et la conversion (37-3b) l'appellent, **puis**
/// [`validate_fx_accounts_in_tx`] sous leurs verrous — la résolution ne juge pas
/// la validité des comptes, la revalidation le fait.
pub fn fx_accounts_for(company: &Company, account: &Account) -> Option<FxAccounts> {
    fx_accounts_for_settings(
        company.fx_gain_account_id,
        company.fx_loss_account_id,
        account,
    )
}

/// [`fx_accounts_for`] sur les seuls comptes de la société, quand l'appelant
/// tient le réglage lu **sous verrou** ([`lock_company_currency_in_tx`], route
/// de conversion de la Story 37-3b) et non une [`Company`] entière. Même règle —
/// `fx_accounts_for` n'en est que l'enveloppe, aucun choix n'est écrit à côté.
pub fn fx_accounts_for_settings(
    company_fx_gain: Option<i64>,
    company_fx_loss: Option<i64>,
    account: &Account,
) -> Option<FxAccounts> {
    fx_accounts_for_columns(
        company_fx_gain,
        company_fx_loss,
        account.id,
        account.currency.as_deref(),
        account.fx_gain_account_id,
        account.fx_loss_account_id,
    )
}

/// [`fx_accounts_for_settings`] sur les seules colonnes qu'elle lit — la
/// **même** règle, pour un appelant qui tient une projection du compte (Story
/// 37-5, réévaluation) plutôt qu'un `Account` complet. Seul lieu de la règle :
/// les deux enveloppes ci-dessus y délèguent.
pub fn fx_accounts_for_columns(
    company_gain: Option<i64>,
    company_loss: Option<i64>,
    account_id: i64,
    account_currency: Option<&str>,
    own_gain: Option<i64>,
    own_loss: Option<i64>,
) -> Option<FxAccounts> {
    account_currency?;
    if let (Some(gain), Some(loss)) = (own_gain, own_loss) {
        return Some(FxAccounts {
            gain,
            loss,
            origin: FxAccountsOrigin::Own,
            holder_account_id: account_id,
        });
    }
    match (company_gain, company_loss) {
        (Some(gain), Some(loss)) => Some(FxAccounts {
            gain,
            loss,
            origin: FxAccountsOrigin::Company,
            holder_account_id: account_id,
        }),
        _ => None,
    }
}

/// Verrou de ligne des lectures de [`check_fx_accounts_in_tx`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FxAccountLock {
    /// `FOR UPDATE` : le compte est **désigné** par le geste en cours (réglage
    /// de la société, comptes propres d'un compte) — il ne change pas avant le
    /// `COMMIT`, et un geste concurrent sur lui attend.
    ForUpdate,
    /// `LOCK IN SHARE MODE` : le compte est **employé** (revalidation à l'usage,
    /// [`validate_fx_accounts_in_tx`]) — il ne change pas avant le `COMMIT`.
    Share,
}

/// Un compte de différence de change à contrôler (Story 37-1, AC-3, AC-4 bis).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FxAccountCheck {
    /// Le champ visé : gains, pertes ou conversion.
    pub field: FxAccountField,
    /// Le compte désigné.
    pub account_id: i64,
    /// Le compte en devise qui porte ce compte comme compte **propre** ;
    /// `None` pour un compte de la société. Reporté tel quel dans le refus
    /// (`holderAccountId`).
    pub holder_account_id: Option<i64>,
}

/// La raison pour laquelle `account` ne peut pas tenir le rôle `field` (Story
/// 37-1, AC-3), ou `None` s'il le peut — fonction pure, partagée par toutes les
/// gardes d'un compte de différence de change.
///
/// Gardes, dans l'ordre de la liste close : de la société (`unknown`), actif
/// (`archived`), imputable (`notPostable`), du bon type (`accountType` :
/// `Expense`/`Revenue` pour les gains et pertes, `Asset`/`Liability` pour la
/// conversion), sans devise (`foreignCurrency`), et — gains et pertes seulement
/// — ni au rôle `DefaultRevenue`, ni désigné compte de produit par défaut
/// (`revenueAccount`, CO 959b ; `revenue_designated` dit si les réglages de
/// facturation ou un article le désignent). Les autres désignations (arrondi,
/// frais bancaires…) restent admises.
pub fn fx_account_rejection(
    field: FxAccountField,
    company_id: i64,
    account: Option<&Account>,
    revenue_designated: bool,
) -> Option<FxAccountReason> {
    let Some(account) = account.filter(|a| a.company_id == company_id) else {
        return Some(FxAccountReason::Unknown);
    };
    if !account.active {
        return Some(FxAccountReason::Archived);
    }
    if !account.postable {
        return Some(FxAccountReason::NotPostable);
    }
    let type_ok = match field {
        FxAccountField::Gain | FxAccountField::Loss => {
            matches!(
                account.account_type,
                AccountType::Expense | AccountType::Revenue
            )
        }
        FxAccountField::Transit => {
            matches!(
                account.account_type,
                AccountType::Asset | AccountType::Liability
            )
        }
    };
    if !type_ok {
        return Some(FxAccountReason::AccountType);
    }
    if account.currency.is_some() {
        return Some(FxAccountReason::ForeignCurrency);
    }
    if field != FxAccountField::Transit
        && (account.role == Some(AccountRole::DefaultRevenue) || revenue_designated)
    {
        return Some(FxAccountReason::RevenueAccount);
    }
    None
}

/// Contrôle partagé des comptes de différence de change **changés** ou
/// **employés** (Story 37-1, AC-3, AC-4 bis), dans la transaction de
/// l'appelant : `companies::update_currency_settings` (comptes de la société,
/// [`FxAccountLock::ForUpdate`]), les comptes propres d'un compte en devise
/// (`accounts::create`/`update`, [`FxAccountLock::Share`] : la ligne
/// `companies`, tenue `FOR UPDATE` par le geste, les sérialise) et la
/// revalidation à l'usage ([`validate_fx_accounts_in_tx`],
/// [`FxAccountLock::Share`]).
///
/// Le premier compte de `checks` (dans l'ordre reçu) qui échoue rend
/// [`DbError::FxAccountInvalid`] ; `checks` vide ne lit rien.
///
/// **Verrous, dans l'ordre global du dépôt** (`companies` → réglages →
/// `accounts`) : les réglages de facturation `LOCK IN SHARE MODE` (s'il y a un
/// compte de gains ou de pertes à juger), puis les comptes, en une requête
/// triée par `id`, sous `lock`, puis les articles qui les désignent compte de
/// produit (`LOCK IN SHARE MODE`). L'appelant qui verrouille `companies` ou des
/// exercices le fait **avant**. Aucune lecture simple ne précède ces lectures
/// verrouillantes dans la fonction (REPEATABLE READ : une lecture simple
/// figerait l'instantané avant l'attente des verrous).
///
/// Motif C4 : l'appelant ne passe que les comptes **changés** (réglage) ou
/// **employés** (revalidation) — un compte inchangé devenu invalide n'empêche
/// pas d'enregistrer l'autre.
pub async fn check_fx_accounts_in_tx(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    checks: &[FxAccountCheck],
    lock: FxAccountLock,
) -> Result<(), DbError> {
    if checks.is_empty() {
        return Ok(());
    }
    let revenue_ids: Vec<i64> = checks
        .iter()
        .filter(|c| c.field != FxAccountField::Transit)
        .map(|c| c.account_id)
        .collect();

    // 1. Réglages de facturation, en partage (avant les comptes).
    let settings_revenue: Option<i64> = if revenue_ids.is_empty() {
        None
    } else {
        sqlx::query_scalar::<_, Option<i64>>(
            "SELECT default_revenue_account_id FROM company_invoice_settings \
             WHERE company_id = ? LOCK IN SHARE MODE",
        )
        .bind(company_id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(map_db_error)?
        .flatten()
    };

    // 2. Les comptes, en une requête triée par `id` (ordre de verrouillage stable).
    let mut ids: Vec<i64> = checks.iter().map(|c| c.account_id).collect();
    ids.sort_unstable();
    ids.dedup();
    let placeholders = std::iter::repeat_n("?", ids.len())
        .collect::<Vec<_>>()
        .join(",");
    let lock_clause = match lock {
        FxAccountLock::ForUpdate => "FOR UPDATE",
        FxAccountLock::Share => "LOCK IN SHARE MODE",
    };
    let sql = format!(
        "SELECT {COLUMNS} FROM accounts WHERE id IN ({placeholders}) ORDER BY id {lock_clause}"
    );
    let mut query = sqlx::query_as::<_, Account>(&sql);
    for id in &ids {
        query = query.bind(id);
    }
    let accounts: Vec<Account> = query.fetch_all(&mut *conn).await.map_err(map_db_error)?;

    // 3. Les articles qui désignent l'un d'eux compte de produit, en partage.
    let mut product_revenue: Vec<i64> = Vec::new();
    if !revenue_ids.is_empty() {
        let placeholders = std::iter::repeat_n("?", revenue_ids.len())
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT DISTINCT default_revenue_account_id FROM products \
             WHERE company_id = ? AND default_revenue_account_id IN ({placeholders}) \
             LOCK IN SHARE MODE"
        );
        let mut query = sqlx::query_scalar::<_, i64>(&sql).bind(company_id);
        for id in &revenue_ids {
            query = query.bind(id);
        }
        product_revenue = query.fetch_all(&mut *conn).await.map_err(map_db_error)?;
    }

    for check in checks {
        let account = accounts.iter().find(|a| a.id == check.account_id);
        let revenue_designated = settings_revenue == Some(check.account_id)
            || product_revenue.contains(&check.account_id);
        if let Some(reason) =
            fx_account_rejection(check.field, company_id, account, revenue_designated)
        {
            return Err(DbError::FxAccountInvalid {
                field: check.field,
                reason,
                holder_account_id: check.holder_account_id,
            });
        }
    }
    Ok(())
}

/// **Revalidation à l'usage** des comptes de différence de change résolus par
/// [`fx_accounts_for`] (Story 37-1, AC-4 bis, C-37-2 (3)) : relit les deux
/// comptes `LOCK IN SHARE MODE` et leur applique les gardes de l'AC-3
/// ([`fx_account_rejection`]) — sinon [`DbError::FxAccountInvalid`] (`field`
/// `fxGainAccountId` | `fxLossAccountId`, `holderAccountId` quand
/// `origin = Own`).
///
/// Appelée sous les verrous de l'appelant : la réévaluation (37-5) et la route
/// de conversion (37-3b). Un compte de la société invalide mais **non employé**
/// n'arrête rien : seuls les comptes résolus pour les comptes touchés se
/// passent ici.
pub async fn validate_fx_accounts_in_tx(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    fx: &FxAccounts,
) -> Result<(), DbError> {
    let holder_account_id = (fx.origin == FxAccountsOrigin::Own).then_some(fx.holder_account_id);
    let checks = [
        FxAccountCheck {
            field: FxAccountField::Gain,
            account_id: fx.gain,
            holder_account_id,
        },
        FxAccountCheck {
            field: FxAccountField::Loss,
            account_id: fx.loss,
            holder_account_id,
        },
    ];
    check_fx_accounts_in_tx(conn, company_id, &checks, FxAccountLock::Share).await
}

/// Un compte d'une ligne de l'écriture à compléter d'une conversion (Story
/// 37-3b, AC-2 bis), lu par [`conversion_line_accounts_in_tx`].
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ConversionLineAccount {
    /// Le compte (type, devise, comptes de différence propres).
    #[sqlx(flatten)]
    pub account: Account,
    /// Non nul si un compte de la société désigne celui-ci comme compte
    /// **propre** de gains ou de pertes de change (`accounts.fx_*`, 37-1 AC-4
    /// bis). La désignation par la société (`companies.fx_*`) se lit sur le
    /// réglage verrouillé de l'appelant.
    pub designated_by_account: i64,
}

/// Reconnaissance des comptes des lignes de l'écriture à compléter d'une
/// conversion (Story 37-3b, AC-2 bis) : pour chaque compte de `ids`, le compte
/// lui-même (type et devise, lus par la règle de forme, A33) et s'il est le
/// compte de différence **propre** d'un compte de la société — une requête
/// **bornée** à ces comptes (`EXISTS` par identifiant, jamais un balayage de
/// tous les comptes), `LOCK IN SHARE MODE`, triée par `id`. Le verrou porte
/// sur les comptes lus, **non** sur les lignes `o` de la sous-requête
/// `EXISTS` : une désignation de compte propre posée en parallèle peut n'être
/// pas vue (revue de code P1, 37-3b-CR-P1-16 ; la paire est revalidée sous
/// verrou à l'usage).
///
/// Appelée **après** le verrou de `companies` (ordre `companies` → `accounts`
/// de 37-1 AC-3). Un identifiant inconnu ou d'une autre société est absent du
/// résultat ; la création le refusera (`INACTIVE_OR_INVALID_ACCOUNTS`).
pub async fn conversion_line_accounts_in_tx(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    ids: &[i64],
) -> Result<Vec<ConversionLineAccount>, DbError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut ids: Vec<i64> = ids.to_vec();
    ids.sort_unstable();
    ids.dedup();
    let placeholders = std::iter::repeat_n("?", ids.len())
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT {COLUMNS}, \
         EXISTS (SELECT 1 FROM accounts o WHERE o.company_id = accounts.company_id \
                 AND (o.fx_gain_account_id = accounts.id OR o.fx_loss_account_id = accounts.id)) \
           AS designated_by_account \
         FROM accounts WHERE company_id = ? AND id IN ({placeholders}) \
         ORDER BY id LOCK IN SHARE MODE"
    );
    let mut query = sqlx::query_as::<_, ConversionLineAccount>(&sql).bind(company_id);
    for id in &ids {
        query = query.bind(id);
    }
    query.fetch_all(&mut *conn).await.map_err(map_db_error)
}

// ---------------------------------------------------------------------------
// Story 37-1 (AC-4, AC-4 bis) — verrous et gardes de la devise d'un compte
// ---------------------------------------------------------------------------

/// Le réglage multi-devises d'une société, lu **sous verrou** dans la
/// transaction de l'appelant ([`lock_company_currency_in_tx`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::FromRow)]
pub struct CompanyCurrencyState {
    /// Le mode multi-devises est actif (irréversible, CR-7).
    pub multi_currency_enabled: bool,
    /// Compte des gains de change de la société.
    pub fx_gain_account_id: Option<i64>,
    /// Compte des pertes de change de la société.
    pub fx_loss_account_id: Option<i64>,
    /// Compte de conversion de la société.
    pub fx_transit_account_id: Option<i64>,
}

/// Verrou de la ligne `companies` d'un geste sur un compte (Story 37-1, AC-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompanyLock {
    /// `LOCK IN SHARE MODE` : toute garde de devise ou de désignation (création,
    /// modification et archivage d'un compte, sites de configuration).
    Share,
    /// `FOR UPDATE`, **premier** verrou du geste : un `POST` ou un `PUT` de
    /// compte qui porte des comptes de différence propres (AC-4 bis) — il se
    /// sérialise ainsi contre toutes les gardes inverses, qui prennent
    /// `companies` en partage avant le compte visé. Jamais un partage suivi
    /// d'un exclusif (promotion S → X).
    ForUpdate,
}

/// Lit le réglage multi-devises de la société **sous verrou** — premier verrou
/// d'un geste sur un compte, dans l'ordre global du dépôt (`companies` →
/// réglages → `accounts` → `bank_accounts`). Société absente → `NotFound`.
pub async fn lock_company_currency_in_tx(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    lock: CompanyLock,
) -> Result<CompanyCurrencyState, DbError> {
    let sql = match lock {
        CompanyLock::Share => {
            "SELECT multi_currency_enabled, fx_gain_account_id, fx_loss_account_id, \
             fx_transit_account_id FROM companies WHERE id = ? LOCK IN SHARE MODE"
        }
        CompanyLock::ForUpdate => {
            "SELECT multi_currency_enabled, fx_gain_account_id, fx_loss_account_id, \
             fx_transit_account_id FROM companies WHERE id = ? FOR UPDATE"
        }
    };
    sqlx::query_as::<_, CompanyCurrencyState>(sql)
        .bind(company_id)
        .fetch_optional(conn)
        .await
        .map_err(map_db_error)?
        .ok_or(DbError::NotFound)
}

/// La ligne des réglages de facturation de la société, prise **en partage**
/// (Story 37-1, AC-4) — après `companies`, **avant** le `FOR UPDATE` du compte :
/// la lire après formerait un cycle avec `company_invoice_settings::update`,
/// qui tient les réglages en `FOR UPDATE` puis lit le compte. Une société sans
/// ligne de réglages verrouille l'intervalle où elle serait.
async fn lock_invoice_settings_share_in_tx(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
) -> Result<(), DbError> {
    let _: Option<i64> = sqlx::query_scalar(
        "SELECT company_id FROM company_invoice_settings WHERE company_id = ? LOCK IN SHARE MODE",
    )
    .bind(company_id)
    .fetch_optional(conn)
    .await
    .map_err(map_db_error)?;
    Ok(())
}

/// Une désignation de compte de différence de change trouvée par
/// [`fx_designations_of_in_tx`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FxDesignationHit {
    /// Gains, pertes ou conversion.
    pub designation: FxDesignation,
    /// `(id, numéro)` du compte en devise qui le désigne (désignation propre) ;
    /// `None` pour une désignation de la société.
    pub holder: Option<(i64, String)>,
}

impl FxDesignationHit {
    /// Le refus `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT` qui nomme cette désignation.
    pub fn into_error(self) -> DbError {
        let (holder_account_id, holder_number) = match self.holder {
            Some((id, number)) => (Some(id), Some(number)),
            None => (None, None),
        };
        DbError::AccountIsFxDifferenceAccount {
            designation: self.designation,
            holder_account_id,
            holder_number,
        }
    }
}

/// `true` si un compte qui porte la désignation `designation` peut prendre le
/// type `account_type` : charges ou produits pour les gains et pertes, actif ou
/// passif pour la conversion (Story 37-1, AC-3, AC-4).
pub fn designation_admits_type(designation: FxDesignation, account_type: AccountType) -> bool {
    match designation {
        FxDesignation::Gain | FxDesignation::Loss => {
            matches!(account_type, AccountType::Expense | AccountType::Revenue)
        }
        FxDesignation::Transit => {
            matches!(account_type, AccountType::Asset | AccountType::Liability)
        }
    }
}

/// Les désignations de `account_id` comme compte de différence de change
/// (Story 37-1, AC-4, AC-4 bis) : celles de la **société** d'abord (gains,
/// pertes, conversion, lues sur `state`, déjà sous verrou), puis **une**
/// désignation propre — le compte en devise de plus petit identifiant qui le
/// désigne, archivé compris.
///
/// La désignation propre se cherche par **deux** lectures verrouillantes, chacune
/// par son index nommé (`FORCE INDEX (idx_accounts_fx_gain)`, puis
/// `idx_accounts_fx_loss` ; jamais un `OR` sur les deux colonnes, qui
/// balaierait — et verrouillerait en partage — toutes les lignes de la
/// société) ; le plus petit identifiant des deux est gardé, les gains à
/// égalité. ⚠️ À appeler **après** le `FOR UPDATE` du compte visé : deux
/// archivages concurrents de comptes différents ne verrouillent ainsi que les
/// entrées d'index de leur propre compte.
pub async fn fx_designations_of_in_tx(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    state: &CompanyCurrencyState,
    account_id: i64,
) -> Result<Vec<FxDesignationHit>, DbError> {
    // `0` : aucune ligne ne porte cet identifiant (AUTO_INCREMENT démarre à 1).
    fx_designations_excluding_in_tx(conn, company_id, state, account_id, 0).await
}

/// [`fx_designations_of_in_tx`], sans les désignations propres portées par le
/// compte `exclude_holder` (correctif `fix-api-parent-compte`, revue P1-4) :
/// le changement de parent juge celles du compte déplacé sur l'état après son
/// `PUT`. Mêmes lectures verrouillantes, par les mêmes index.
async fn fx_designations_excluding_in_tx(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    state: &CompanyCurrencyState,
    account_id: i64,
    exclude_holder: i64,
) -> Result<Vec<FxDesignationHit>, DbError> {
    let mut hits: Vec<FxDesignationHit> = [
        (state.fx_gain_account_id, FxDesignation::Gain),
        (state.fx_loss_account_id, FxDesignation::Loss),
        (state.fx_transit_account_id, FxDesignation::Transit),
    ]
    .into_iter()
    .filter(|(designated, _)| *designated == Some(account_id))
    .map(|(_, designation)| FxDesignationHit {
        designation,
        holder: None,
    })
    .collect();

    let gain: Option<(i64, String)> = sqlx::query_as(
        "SELECT id, number FROM accounts FORCE INDEX (idx_accounts_fx_gain) \
         WHERE company_id = ? AND fx_gain_account_id = ? AND id <> ? \
         ORDER BY id LIMIT 1 LOCK IN SHARE MODE",
    )
    .bind(company_id)
    .bind(account_id)
    .bind(exclude_holder)
    .fetch_optional(&mut *conn)
    .await
    .map_err(map_db_error)?;
    let loss: Option<(i64, String)> = sqlx::query_as(
        "SELECT id, number FROM accounts FORCE INDEX (idx_accounts_fx_loss) \
         WHERE company_id = ? AND fx_loss_account_id = ? AND id <> ? \
         ORDER BY id LIMIT 1 LOCK IN SHARE MODE",
    )
    .bind(company_id)
    .bind(account_id)
    .bind(exclude_holder)
    .fetch_optional(&mut *conn)
    .await
    .map_err(map_db_error)?;
    let own = match (gain, loss) {
        (Some(g), Some(l)) if l.0 < g.0 => Some((FxDesignation::Loss, l)),
        (Some(g), _) => Some((FxDesignation::Gain, g)),
        (None, Some(l)) => Some((FxDesignation::Loss, l)),
        (None, None) => None,
    };
    if let Some((designation, holder)) = own {
        hits.push(FxDesignationHit {
            designation,
            holder: Some(holder),
        });
    }
    Ok(hits)
}

/// Le premier refus parmi `hits` dont la désignation interdit le geste
/// (`forbidden`), ou `Ok(())`.
fn refuse_designated(
    hits: Vec<FxDesignationHit>,
    forbidden: impl Fn(FxDesignation) -> bool,
) -> Result<(), DbError> {
    match hits.into_iter().find(|h| forbidden(h.designation)) {
        Some(hit) => Err(hit.into_error()),
        None => Ok(()),
    }
}

/// Pourquoi un compte ne peut pas être désigné à un site de configuration qui
/// débite ou crédite en CHF — rendu par [`designation_refusal_in_tx`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesignationRefusal {
    /// Le compte est tenu en devise (`ACCOUNT_IN_FOREIGN_CURRENCY`).
    ForeignCurrency {
        /// Code ISO 4217 de la devise du compte.
        currency: String,
    },
    /// Le compte de produit par défaut est un compte de gains ou de pertes de
    /// change, de la société ou propre à un compte en devise
    /// (`ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`).
    FxDifference(FxDesignationHit),
}

impl DesignationRefusal {
    /// L'erreur que rend un geste de l'utilisateur refusé pour cette cause
    /// (`PUT` des réglages de facturation, article).
    pub fn into_error(self, account_id: i64) -> DbError {
        match self {
            Self::ForeignCurrency { currency } => DbError::AccountInForeignCurrency {
                account_id: Some(account_id),
                currency,
            },
            Self::FxDifference(hit) => hit.into_error(),
        }
    }
}

/// **La** règle de désignation d'un compte à un réglage de facturation ou au
/// compte de produit d'un article (Story 37-1, AC-4, FR-37-1-5 et FR-37-1-12) —
/// partagée par le `PUT` des réglages, l'article et la finalisation de
/// l'assistant (revue P1 de l'intégration de la v0.18.0, M1 : la finalisation en
/// tenait une copie plus large) :
///
/// 1. un compte **en devise** n'est désigné à aucun réglage
///    ([`DesignationRefusal::ForeignCurrency`]) ;
/// 2. un compte de **gains ou de pertes de change** — de la société, ou propre à
///    un compte en devise, archivé compris — n'est pas désigné **compte de produit
///    par défaut** (`default_revenue`) : les gains de change ne vont pas au chiffre
///    d'affaires ([`DesignationRefusal::FxDifference`]). Aux autres réglages
///    (débiteurs, créanciers, arrondi, escompte, frais bancaires, pertes), il se
///    désigne ; le **compte de conversion** n'est visé par aucun refus de
///    désignation (son type, actif ou passif, lui ferme déjà le rôle
///    `DefaultRevenue`).
///
/// `None` si le compte peut être désigné. La devise se relit `LOCK IN SHARE MODE`
/// ([`currency_in_share_mode`]) ; `state` est lu sous le verrou de `companies` de
/// l'appelant ; les recherches de désignation propre sont verrouillantes
/// ([`fx_designations_of_in_tx`]). Un compte absent ou d'une autre société n'est
/// pas refusé ici.
pub async fn designation_refusal_in_tx(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    state: &CompanyCurrencyState,
    account_id: i64,
    default_revenue: bool,
) -> Result<Option<DesignationRefusal>, DbError> {
    if let Some(Some(currency)) = currency_in_share_mode(&mut *conn, company_id, account_id).await?
    {
        return Ok(Some(DesignationRefusal::ForeignCurrency { currency }));
    }
    if !default_revenue {
        return Ok(None);
    }
    Ok(
        fx_designations_of_in_tx(conn, company_id, state, account_id)
            .await?
            .into_iter()
            .find(|h| h.designation != FxDesignation::Transit)
            .map(DesignationRefusal::FxDifference),
    )
}

/// [`designation_refusal_in_tx`], rendu en erreur : le refus d'un geste de
/// l'utilisateur (`PUT` des réglages de facturation, article).
pub async fn refuse_designation_in_tx(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    state: &CompanyCurrencyState,
    account_id: i64,
    default_revenue: bool,
) -> Result<(), DbError> {
    match designation_refusal_in_tx(conn, company_id, state, account_id, default_revenue).await? {
        Some(refusal) => Err(refusal.into_error(account_id)),
        None => Ok(()),
    }
}

/// La devise d'un compte de la société, lue **`LOCK IN SHARE MODE`** —
/// `None` si la ligne est absente, `Some(None)` pour un compte en CHF.
pub async fn currency_in_share_mode(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    account_id: i64,
) -> Result<Option<Option<String>>, DbError> {
    sqlx::query_scalar::<_, Option<String>>(
        "SELECT currency FROM accounts WHERE id = ? AND company_id = ? LOCK IN SHARE MODE",
    )
    .bind(account_id)
    .bind(company_id)
    .fetch_optional(conn)
    .await
    .map_err(map_db_error)
}

/// Refuse, parmi les comptes d'une **pièce** (lignes de facture, d'avoir, de
/// facture fournisseur ; compte de règlement), le premier compte **en devise**,
/// par identifiant croissant (Story 37-3a, AC-5) → 400
/// `ACCOUNT_IN_FOREIGN_CURRENCY`. Une pièce se comptabilise en CHF : la
/// refuser à l'enregistrement dit pourquoi plus tôt que le contrôle central de
/// l'écriture, qui reste le filet. Lecture **sans verrou**, délibérément :
/// l'enregistrement d'un brouillon ne prend aucun verrou de compte
/// (C-v017-4-28) — un verrou partagé ici ferait attendre la création d'un
/// brouillon derrière toute validation qui tient les comptes ; la devise se
/// rejuge sous verrou au contrôle central de l'écriture (`LOCK IN SHARE MODE`).
/// Un compte absent ou d'une autre société n'est pas refusé ici (les contrôles
/// d'appartenance de l'appelant le font).
pub async fn refuse_foreign_currency_accounts_in_tx(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    account_ids: &[i64],
) -> Result<(), DbError> {
    if account_ids.is_empty() {
        return Ok(());
    }
    let mut ids = account_ids.to_vec();
    ids.sort_unstable();
    ids.dedup();
    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT id, currency FROM accounts \
         WHERE company_id = ? AND id IN ({placeholders}) AND currency IS NOT NULL \
         ORDER BY id LIMIT 1"
    );
    let mut q = sqlx::query_as::<_, (i64, String)>(&sql).bind(company_id);
    for id in &ids {
        q = q.bind(id);
    }
    match q.fetch_optional(conn).await.map_err(map_db_error)? {
        Some((account_id, currency)) => Err(DbError::AccountInForeignCurrency {
            account_id: Some(account_id),
            currency,
        }),
        None => Ok(()),
    }
}

/// Refuse un sous-compte sous un compte **candidat à la réévaluation**
/// ([`super::fx_revaluation::CANDIDATE_PREDICATE`] : de bilan, en devise) qui
/// porte au moins une ligne d'écriture — `AccountForeignParentHasEntries`
/// (37-5-CR-P3-1). Le parent passerait non imputable avec son solde : la
/// réévaluation ne l'atteindrait plus, K9 et la clôture le compteraient, et il
/// ne redeviendrait pas imputable tant qu'il a des sous-comptes actifs
/// ([`effective_postable`]) — une clôture sans recours. Le parent est déjà
/// verrouillé `FOR UPDATE` par l'appelant ; la lecture des lignes est
/// verrouillante (comme [`has_journal_line_locked`]), si bien qu'une écriture
/// concurrente sur le parent est vue, ou refusée ensuite comme non imputable.
async fn refuse_foreign_parent_with_entries_in_tx(
    conn: &mut sqlx::MySqlConnection,
    parent_id: i64,
) -> Result<(), DbError> {
    let sql = format!(
        "SELECT a.number, a.currency FROM accounts a WHERE a.id = ? AND {}",
        super::fx_revaluation::CANDIDATE_PREDICATE
    );
    let parent: Option<(String, String)> = sqlx::query_as(&sql)
        .bind(parent_id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(map_db_error)?;
    if let Some((parent_number, currency)) = parent
        && has_journal_line_locked(&mut *conn, parent_id).await?
    {
        return Err(DbError::AccountForeignParentHasEntries {
            parent_account_id: parent_id,
            parent_number,
            currency,
        });
    }
    Ok(())
}

/// `true` si le compte porte au moins une ligne d'écriture — lecture
/// **verrouillante** (`LOCK IN SHARE MODE`, par l'index `idx_jel_account`),
/// après le `FOR UPDATE` du compte (Story 37-1, AC-4) : une ligne commitée
/// pendant l'attente des verrous est vue, et une ligne en cours d'insertion
/// attend la fin du geste.
async fn has_journal_line_locked(
    conn: &mut sqlx::MySqlConnection,
    account_id: i64,
) -> Result<bool, DbError> {
    let line: Option<i32> = sqlx::query_scalar(
        "SELECT 1 FROM journal_entry_lines WHERE account_id = ? LIMIT 1 LOCK IN SHARE MODE",
    )
    .bind(account_id)
    .fetch_optional(conn)
    .await
    .map_err(map_db_error)?;
    Ok(line.is_some())
}

/// Le premier compte bancaire **de prestataire** (`provider` non nul, archivé
/// ou non) lié au compte du plan `account_id`, par `id` — Story 38-11b, revue
/// P1 (M3). Lecture `LOCK IN SHARE MODE`, après le `FOR UPDATE` du compte :
/// elle voit un lien posé pendant que la modification attendait la ligne.
async fn provider_bank_account_linked_in_tx(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    account_id: i64,
) -> Result<Option<i64>, DbError> {
    sqlx::query_scalar(
        "SELECT id FROM bank_accounts WHERE journal_account_id = ? AND company_id = ? \
         AND provider IS NOT NULL ORDER BY id LIMIT 1 LOCK IN SHARE MODE",
    )
    .bind(account_id)
    .bind(company_id)
    .fetch_optional(conn)
    .await
    .map_err(map_db_error)
}

/// Le premier **site de configuration** (hors rôle, que l'appelant juge sur
/// l'état résultant) qui débite le compte en CHF (Story 37-1, AC-4) : réglages
/// de facturation, compte bancaire, article, règle de réconciliation — chacun
/// compté **quel que soit l'état** de l'objet qui désigne. Lectures
/// verrouillantes, après le `FOR UPDATE` du compte (les réglages sont déjà
/// tenus en partage par l'appelant).
async fn currency_use_site_in_tx(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    account_id: i64,
) -> Result<Option<CurrencyUseSite>, DbError> {
    if let Some(code) = company_invoice_settings::designations_of_account_in_share_mode(
        &mut *conn, company_id, account_id,
    )
    .await?
    .into_iter()
    .next()
    {
        return Ok(Some(CurrencyUseSite::InvoiceSettings(code)));
    }
    let bank: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM bank_accounts WHERE journal_account_id = ? AND company_id = ? \
         ORDER BY id LIMIT 1 LOCK IN SHARE MODE",
    )
    .bind(account_id)
    .bind(company_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(map_db_error)?;
    if let Some(id) = bank {
        return Ok(Some(CurrencyUseSite::BankAccount(id)));
    }
    let product: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM products WHERE default_revenue_account_id = ? AND company_id = ? \
         ORDER BY id LIMIT 1 LOCK IN SHARE MODE",
    )
    .bind(account_id)
    .bind(company_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(map_db_error)?;
    if let Some(id) = product {
        return Ok(Some(CurrencyUseSite::Product(id)));
    }
    let rule: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM reconciliation_rules WHERE counterparty_account_id = ? AND company_id = ? \
         ORDER BY id LIMIT 1 LOCK IN SHARE MODE",
    )
    .bind(account_id)
    .bind(company_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(map_db_error)?;
    Ok(rule.map(CurrencyUseSite::ReconciliationRule))
}

/// Les comptes de différence propres à contrôler pour un geste qui les pose
/// (Story 37-1, AC-4 bis) : `unexpected` si le compte reste ou devient CHF,
/// puis gain puis perte, comptes **changés** seulement (motif C4), lus
/// `LOCK IN SHARE MODE` après le `FOR UPDATE` du compte en devise.
///
/// Revue de code P2 (37-1-CR-P2-1) : un compte ne se désigne pas lui-même.
/// Lu avant l'`UPDATE`, il est encore CHF quand le même `PUT` lui donne une
/// devise et le nomme gain ou perte, de sorte que le refus `foreignCurrency`
/// de [`check_fx_accounts_in_tx`] ne le verrait pas : il est jugé ici sur
/// l'état résultant (le compte porte une devise), même raison.
async fn check_own_fx_accounts_in_tx(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    holder_account_id: Option<i64>,
    resulting_currency: Option<&str>,
    before: Option<OwnFxAccounts>,
    after: Option<OwnFxAccounts>,
) -> Result<(), DbError> {
    let Some(after) = after else {
        return Ok(());
    };
    if resulting_currency.is_none() {
        return Err(DbError::FxAccountInvalid {
            field: FxAccountField::Gain,
            reason: FxAccountReason::Unexpected,
            holder_account_id,
        });
    }
    if let Some(holder) = holder_account_id {
        for (field, id) in [
            (FxAccountField::Gain, after.gain),
            (FxAccountField::Loss, after.loss),
        ] {
            if id == holder {
                return Err(DbError::FxAccountInvalid {
                    field,
                    reason: FxAccountReason::ForeignCurrency,
                    holder_account_id,
                });
            }
        }
    }
    let mut checks = Vec::with_capacity(2);
    if before.map(|b| b.gain) != Some(after.gain) {
        checks.push(FxAccountCheck {
            field: FxAccountField::Gain,
            account_id: after.gain,
            holder_account_id,
        });
    }
    if before.map(|b| b.loss) != Some(after.loss) {
        checks.push(FxAccountCheck {
            field: FxAccountField::Loss,
            account_id: after.loss,
            holder_account_id,
        });
    }
    check_fx_accounts_in_tx(conn, company_id, &checks, FxAccountLock::Share).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::account::AccountType;

    /// Helper : obtient le pool de test via DATABASE_URL depuis .env.
    /// Les tests d'intégration nécessitent une MariaDB réelle.
    async fn test_pool() -> MySqlPool {
        dotenvy::dotenv().ok();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL required for DB tests");
        MySqlPool::connect(&url).await.expect("DB connect failed")
    }

    /// Helper : obtient le company_id de la première company (créée par le seed/onboarding).
    async fn get_company_id(pool: &MySqlPool) -> i64 {
        let row: (i64,) = sqlx::query_as("SELECT id FROM companies LIMIT 1")
            .fetch_one(pool)
            .await
            .expect("need at least one company in DB for tests");
        row.0
    }

    /// Helper : obtient un user_id admin pour les appels write qui exigent un acteur audité.
    async fn get_admin_user_id(pool: &MySqlPool) -> i64 {
        let row: (i64,) = sqlx::query_as("SELECT id FROM users WHERE role = 'Admin' LIMIT 1")
            .fetch_one(pool)
            .await
            .expect("need at least one Admin user in DB for tests");
        row.0
    }

    /// Helper : société **jetable**, dédiée aux tests qui posent un rôle singleton.
    ///
    /// Code review 14-3a : ces tests s'exécutaient sur la société partagée
    /// (`get_company_id`). Si celle-ci possède les comptes 1100 / 2000 / 3000 —
    /// ce qui est le cas d'une base de développement seedée **avant** la
    /// migration 14-3a, dont le backfill leur attribue alors `Receivable`,
    /// `Payable` et `DefaultRevenue` — l'INSERT du test heurte
    /// `uq_accounts_company_singleton_role` et panique. Le résultat dépendait du
    /// contenu de la base, pas du code. Une société propre par test supprime le
    /// couplage.
    async fn mk_role_company(pool: &MySqlPool, tag: &str) -> i64 {
        let name = format!("Test roles {tag}");
        // Résidu d'un run précédent interrompu avant le nettoyage.
        drop_role_company_by_name(pool, &name).await;

        let res = sqlx::query(
            "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
             VALUES (?, 'Test Address\n1000 Lausanne', 'Independant', 'FR', 'FR')",
        )
        .bind(&name)
        .execute(pool)
        .await
        .expect("création de la société de test");
        res.last_insert_id() as i64
    }

    /// Supprime une société de test et ses comptes.
    async fn drop_role_company(pool: &MySqlPool, company_id: i64) {
        sqlx::query("UPDATE accounts SET parent_id = NULL WHERE company_id = ?")
            .bind(company_id)
            .execute(pool)
            .await
            .ok();
        sqlx::query("DELETE FROM accounts WHERE company_id = ?")
            .bind(company_id)
            .execute(pool)
            .await
            .ok();
        sqlx::query("DELETE FROM companies WHERE id = ?")
            .bind(company_id)
            .execute(pool)
            .await
            .ok();
    }

    async fn drop_role_company_by_name(pool: &MySqlPool, name: &str) {
        if let Ok(Some(row)) =
            sqlx::query_as::<_, (i64,)>("SELECT id FROM companies WHERE name = ?")
                .bind(name)
                .fetch_optional(pool)
                .await
        {
            drop_role_company(pool, row.0).await;
        }
    }

    /// Helper : nettoie les comptes de test (numéros commençant par "T").
    async fn cleanup_test_accounts(pool: &MySqlPool, company_id: i64) {
        // Détacher les parents d'abord
        sqlx::query(
            "UPDATE accounts SET parent_id = NULL WHERE company_id = ? AND number LIKE 'T%'",
        )
        .bind(company_id)
        .execute(pool)
        .await
        .ok();
        sqlx::query("DELETE FROM accounts WHERE company_id = ? AND number LIKE 'T%'")
            .bind(company_id)
            .execute(pool)
            .await
            .ok();
    }

    // -----------------------------------------------------------------------
    // Story 25-2-a ([#382], [#274]) — recensement de l'ampleur d'un retypage.
    // -----------------------------------------------------------------------

    /// Nettoyage **profond** d'une société jetable.
    ///
    /// ⚠️ `drop_role_company` ne suffit pas ici, et son insuffisance est muette.
    /// Il enchaîne `DELETE FROM accounts` puis `DELETE FROM companies` ; or
    /// `fk_jel_account` est en `ON DELETE RESTRICT`. Dès qu'une écriture
    /// mouvemente un compte de la société, les deux suppressions échouent — et
    /// l'échec est **avalé par `.ok()`** —, laissant en base société, exercice,
    /// comptes et écritures. C'est le mode d'échec de la **KF-039** : un résidu
    /// qui fait rougir le gate SUIVANT, sur un module que la branche ne touche
    /// pas.
    ///
    /// D'où deux différences délibérées : l'ordre imposé par les clés étrangères,
    /// et un **`panic!` au lieu d'un `.ok()`** — un nettoyage qui échoue doit le
    /// dire, sans quoi il est pire que pas de nettoyage du tout.
    ///
    /// `audit_log` n'est **pas** touchée : `company_id` y est un pointeur logique
    /// sans clé étrangère (Story 25-1c-zero), et la piste d'audit se conserve.
    async fn drop_company_deep(pool: &MySqlPool, company_id: i64) {
        for sql in [
            "DELETE FROM journal_entries WHERE company_id = ?",
            "UPDATE accounts SET parent_id = NULL WHERE company_id = ?",
            "DELETE FROM accounts WHERE company_id = ?",
            // Story 25-2-c : le compteur d'écritures porte une clé étrangère
            // `RESTRICT` vers l'exercice, et toute écriture créée en pose une
            // ligne. Il part donc avant l'exercice.
            "DELETE FROM journal_entry_number_sequences WHERE company_id = ?",
            "DELETE FROM fiscal_years WHERE company_id = ?",
            "DELETE FROM companies WHERE id = ?",
        ] {
            sqlx::query(sql)
                .bind(company_id)
                .execute(pool)
                .await
                .unwrap_or_else(|e| panic!("nettoyage « {sql} » : {e}"));
        }
    }

    /// Société jetable **avec son exercice** — `mk_role_company` n'en crée aucun,
    /// et une écriture en exige un. L'exercice naît toujours `Open` : une
    /// écriture ne s'insère pas dans un exercice clos, si bien que le cas « clos »
    /// se construit comme dans la vraie vie — écrire d'abord, fermer ensuite.
    async fn mk_company_with_fy(pool: &MySqlPool, tag: &str) -> (i64, i64) {
        let name = format!("Test roles {tag}");
        if let Ok(Some(row)) =
            sqlx::query_as::<_, (i64,)>("SELECT id FROM companies WHERE name = ?")
                .bind(&name)
                .fetch_optional(pool)
                .await
        {
            drop_company_deep(pool, row.0).await;
        }

        let company_id = mk_role_company(pool, tag).await;
        let res = sqlx::query(
            "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
             VALUES (?, ?, '2026-01-01', '2026-12-31', 'Open')",
        )
        .bind(company_id)
        .bind(format!("Exercice {tag}"))
        .execute(pool)
        .await
        .expect("création de l'exercice de test");

        (company_id, res.last_insert_id() as i64)
    }

    async fn mk_account(
        pool: &MySqlPool,
        company_id: i64,
        admin_user_id: i64,
        number: &str,
        account_type: AccountType,
    ) -> i64 {
        create(
            pool,
            admin_user_id,
            NewAccount {
                company_id,
                number: number.into(),
                name: format!("Compte {number}"),
                account_type,
                parent_id: None,
                role: None,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
            },
        )
        .await
        .expect("création du compte de test")
        .id
    }

    /// Écriture équilibrée à trois lignes dont **deux sur le compte débité** :
    /// c'est la forme qui distingue « écritures distinctes » de « lignes ».
    async fn mk_entry_two_lines_on_debit(
        pool: &MySqlPool,
        company_id: i64,
        fiscal_year_id: i64,
        admin_user_id: i64,
        debit_account: i64,
        credit_account: i64,
    ) {
        use crate::entities::journal_entry::{Journal, NewJournalEntry, NewJournalEntryLine};
        use crate::repositories::journal_entries;
        use rust_decimal::Decimal;

        journal_entries::create(
            pool,
            fiscal_year_id,
            crate::entities::audit_log::Acteur::utilisateur(admin_user_id),
            NewJournalEntry {
                company_id,
                entry_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
                journal: Journal::OD,
                description: "Écriture de test 25-2-a".into(),
                project_id: None,
                lines: vec![
                    NewJournalEntryLine {
                        account_id: debit_account,
                        debit: Decimal::from(100),
                        credit: Decimal::ZERO,
                        project_id: None,
                        foreign: None,
                    },
                    NewJournalEntryLine {
                        account_id: debit_account,
                        debit: Decimal::from(50),
                        credit: Decimal::ZERO,
                        project_id: None,
                        foreign: None,
                    },
                    NewJournalEntryLine {
                        account_id: credit_account,
                        debit: Decimal::ZERO,
                        credit: Decimal::from(150),
                        project_id: None,
                        foreign: None,
                    },
                ],
            },
        )
        .await
        .expect("création de l'écriture de test");
    }

    /// Écriture simple à deux lignes.
    async fn mk_entry(
        pool: &MySqlPool,
        company_id: i64,
        fiscal_year_id: i64,
        admin_user_id: i64,
        debit_account: i64,
        credit_account: i64,
    ) {
        use crate::entities::journal_entry::{Journal, NewJournalEntry, NewJournalEntryLine};
        use crate::repositories::journal_entries;
        use rust_decimal::Decimal;

        journal_entries::create(
            pool,
            fiscal_year_id,
            crate::entities::audit_log::Acteur::utilisateur(admin_user_id),
            NewJournalEntry {
                company_id,
                entry_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
                journal: Journal::OD,
                description: "Écriture de test 25-2-a".into(),
                project_id: None,
                lines: vec![
                    NewJournalEntryLine {
                        account_id: debit_account,
                        debit: Decimal::from(100),
                        credit: Decimal::ZERO,
                        project_id: None,
                        foreign: None,
                    },
                    NewJournalEntryLine {
                        account_id: credit_account,
                        debit: Decimal::ZERO,
                        credit: Decimal::from(100),
                        project_id: None,
                        foreign: None,
                    },
                ],
            },
        )
        .await
        .expect("création de l'écriture de test");
    }

    /// AC 1 — un compte qu'aucune écriture ne touche ne recense rien, et le
    /// retypage doit donc rester libre.
    ///
    /// ⚠️ **La société porte DÉLIBÉRÉMENT une écriture sur un AUTRE compte.**
    /// Sans elle, ce test vérifiait « zéro » dans un univers où tout rend zéro :
    /// il restait vert sous n'importe quelle mutation de la requête — filtre
    /// `account_id` relâché, inversé ou supprimé — et ne gardait donc rien. Le
    /// voisin mouvementé est ce qui rend l'assertion discriminante : si le filtre
    /// par compte cède, ce test voit l'écriture de T908 et rougit.
    #[tokio::test]
    async fn retype_impact_d_un_compte_vierge_ne_recense_rien() {
        let pool = test_pool().await;
        let admin = get_admin_user_id(&pool).await;
        let (company_id, fy) = mk_company_with_fy(&pool, "impact-vierge").await;

        let vierge = mk_account(&pool, company_id, admin, "T900", AccountType::Expense).await;

        // Le voisin mouvementé, dont ce compte-ci ne doit RIEN voir.
        let voisin = mk_account(&pool, company_id, admin, "T908", AccountType::Expense).await;
        let caisse = mk_account(&pool, company_id, admin, "T909", AccountType::Asset).await;
        mk_entry(&pool, company_id, fy, admin, voisin, caisse).await;

        let (count, closed) = retype_impact(&pool, company_id, vierge).await.unwrap();

        assert_eq!(
            count, 0,
            "un compte vierge ne porte aucune écriture — même quand sa société en porte"
        );
        assert!(closed.is_empty(), "aucun exercice clos ne peut être touché");

        drop_company_deep(&pool, company_id).await;
    }

    /// AC 2 — l'ampleur annoncée est le nombre d'écritures, dans un exercice
    /// ouvert la liste des exercices clos reste vide.
    #[tokio::test]
    async fn retype_impact_compte_les_ecritures_d_un_exercice_ouvert() {
        let pool = test_pool().await;
        let admin = get_admin_user_id(&pool).await;
        let (company_id, fy) = mk_company_with_fy(&pool, "impact-ouvert").await;

        let charge = mk_account(&pool, company_id, admin, "T901", AccountType::Expense).await;
        let caisse = mk_account(&pool, company_id, admin, "T902", AccountType::Asset).await;

        mk_entry(&pool, company_id, fy, admin, charge, caisse).await;
        mk_entry(&pool, company_id, fy, admin, charge, caisse).await;

        let (count, closed) = retype_impact(&pool, company_id, charge).await.unwrap();

        assert_eq!(count, 2, "deux écritures mouvementent ce compte");
        assert!(
            closed.is_empty(),
            "l'exercice est ouvert : rien à signaler comme clos, or on a reçu {closed:?}"
        );

        drop_company_deep(&pool, company_id).await;
    }

    /// AC 2 — **le cas qui justifie la story** : le refus doit nommer les
    /// exercices clos, puisque ce sont eux dont le résultat changerait sans
    /// qu'aucune écriture ne l'explique.
    #[tokio::test]
    async fn retype_impact_nomme_les_exercices_clos_touches() {
        let pool = test_pool().await;
        let admin = get_admin_user_id(&pool).await;
        let (company_id, fy) = mk_company_with_fy(&pool, "impact-clos").await;

        let charge = mk_account(&pool, company_id, admin, "T903", AccountType::Expense).await;
        let caisse = mk_account(&pool, company_id, admin, "T904", AccountType::Asset).await;
        mk_entry(&pool, company_id, fy, admin, charge, caisse).await;

        // Fermer APRÈS avoir écrit : une écriture ne s'insère pas dans un
        // exercice clos, et c'est bien l'ordre de la vraie vie.
        sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
            .bind(fy)
            .execute(&pool)
            .await
            .expect("fermeture de l'exercice de test");

        let (count, closed) = retype_impact(&pool, company_id, charge).await.unwrap();

        assert_eq!(count, 1);
        assert_eq!(
            closed,
            vec!["Exercice impact-clos".to_string()],
            "le refus doit NOMMER l'exercice clos touché"
        );

        drop_company_deep(&pool, company_id).await;
    }

    /// AC 2 — des écritures **distinctes**, jamais des lignes : une écriture qui
    /// mouvemente deux fois le même compte reste une écriture.
    #[tokio::test]
    async fn retype_impact_compte_les_ecritures_et_non_les_lignes() {
        let pool = test_pool().await;
        let admin = get_admin_user_id(&pool).await;
        let (company_id, fy) = mk_company_with_fy(&pool, "impact-lignes").await;

        let charge = mk_account(&pool, company_id, admin, "T905", AccountType::Expense).await;
        let caisse = mk_account(&pool, company_id, admin, "T906", AccountType::Asset).await;

        mk_entry_two_lines_on_debit(&pool, company_id, fy, admin, charge, caisse).await;

        let (count, _closed) = retype_impact(&pool, company_id, charge).await.unwrap();

        assert_eq!(
            count, 1,
            "deux lignes d'une même écriture ne font pas deux écritures"
        );

        drop_company_deep(&pool, company_id).await;
    }

    // -----------------------------------------------------------------------
    // Story 25-2-a — la garde elle-même : avertissement bloquant, pas refus sec.
    // -----------------------------------------------------------------------

    /// AC 1 — sans confirmation, retyper un compte mouvementé est refusé, et le
    /// refus porte l'ampleur.
    #[tokio::test]
    async fn retyper_un_compte_mouvemente_sans_confirmation_est_refuse() {
        let pool = test_pool().await;
        let admin = get_admin_user_id(&pool).await;
        let (company_id, fy) = mk_company_with_fy(&pool, "garde-refus").await;

        let charge = mk_account(&pool, company_id, admin, "T910", AccountType::Expense).await;
        let caisse = mk_account(&pool, company_id, admin, "T911", AccountType::Asset).await;
        mk_entry(&pool, company_id, fy, admin, charge, caisse).await;

        let before = find_by_id(&pool, charge).await.unwrap().unwrap();
        let err = update(
            &pool,
            charge,
            before.version,
            admin,
            AccountUpdate {
                name: before.name.clone(),
                account_type: AccountType::Asset,
                role: None,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
                parent: Default::default(),
            },
            false,
        )
        .await
        .unwrap_err();

        match err {
            DbError::AccountHasEntries {
                entry_count,
                closed_fiscal_years,
                from_type,
                to_type,
            } => {
                assert_eq!(entry_count, 1);
                assert!(closed_fiscal_years.is_empty());
                assert_eq!(from_type, "Expense");
                assert_eq!(to_type, "Asset");
            }
            other => panic!("attendu AccountHasEntries, reçu {other:?}"),
        }

        // Et le type n'a PAS bougé : un refus qui laisserait passer l'écriture
        // serait pire qu'aucun refus.
        let after = find_by_id(&pool, charge).await.unwrap().unwrap();
        assert_eq!(after.account_type, AccountType::Expense);
        assert_eq!(after.version, before.version, "aucune version consommée");

        drop_company_deep(&pool, company_id).await;
    }

    /// AC 2 — le refus nomme les exercices clos : ce sont eux dont le résultat
    /// changerait sans qu'aucune écriture ne l'explique.
    #[tokio::test]
    async fn le_refus_de_retypage_nomme_les_exercices_clos() {
        let pool = test_pool().await;
        let admin = get_admin_user_id(&pool).await;
        let (company_id, fy) = mk_company_with_fy(&pool, "garde-clos").await;

        let charge = mk_account(&pool, company_id, admin, "T912", AccountType::Expense).await;
        let caisse = mk_account(&pool, company_id, admin, "T913", AccountType::Asset).await;
        mk_entry(&pool, company_id, fy, admin, charge, caisse).await;
        sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
            .bind(fy)
            .execute(&pool)
            .await
            .expect("fermeture de l'exercice de test");

        let before = find_by_id(&pool, charge).await.unwrap().unwrap();
        let err = update(
            &pool,
            charge,
            before.version,
            admin,
            AccountUpdate {
                name: before.name.clone(),
                account_type: AccountType::Asset,
                role: None,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
                parent: Default::default(),
            },
            false,
        )
        .await
        .unwrap_err();

        match err {
            DbError::AccountHasEntries {
                closed_fiscal_years,
                ..
            } => assert_eq!(closed_fiscal_years, vec!["Exercice garde-clos".to_string()]),
            other => panic!("attendu AccountHasEntries, reçu {other:?}"),
        }

        drop_company_deep(&pool, company_id).await;
    }

    /// AC 1 — **avertissement bloquant, pas refus sec** : confirmé, le geste
    /// passe. Sans ce test, une garde qui refuserait TOUJOURS satisferait les
    /// précédents.
    #[tokio::test]
    async fn retyper_un_compte_mouvemente_passe_une_fois_confirme() {
        let pool = test_pool().await;
        let admin = get_admin_user_id(&pool).await;
        let (company_id, fy) = mk_company_with_fy(&pool, "garde-confirme").await;

        let charge = mk_account(&pool, company_id, admin, "T914", AccountType::Expense).await;
        let caisse = mk_account(&pool, company_id, admin, "T915", AccountType::Asset).await;
        mk_entry(&pool, company_id, fy, admin, charge, caisse).await;

        let before = find_by_id(&pool, charge).await.unwrap().unwrap();
        let updated = update(
            &pool,
            charge,
            before.version,
            admin,
            AccountUpdate {
                name: before.name.clone(),
                account_type: AccountType::Asset,
                role: None,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
                parent: Default::default(),
            },
            true,
        )
        .await
        .expect("un retypage confirmé doit aboutir");

        assert_eq!(updated.account_type, AccountType::Asset);

        drop_company_deep(&pool, company_id).await;
    }

    /// AC 8 — un compte vierge n'est pas gêné : le drapeau ne change rien.
    #[tokio::test]
    async fn retyper_un_compte_vierge_ne_demande_aucune_confirmation() {
        let pool = test_pool().await;
        let admin = get_admin_user_id(&pool).await;
        let (company_id, _fy) = mk_company_with_fy(&pool, "garde-vierge").await;

        let compte = mk_account(&pool, company_id, admin, "T916", AccountType::Expense).await;
        let before = find_by_id(&pool, compte).await.unwrap().unwrap();

        let updated = update(
            &pool,
            compte,
            before.version,
            admin,
            AccountUpdate {
                name: before.name.clone(),
                account_type: AccountType::Asset,
                role: None,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
                parent: Default::default(),
            },
            false,
        )
        .await
        .expect("un compte sans écriture se retype librement");

        assert_eq!(updated.account_type, AccountType::Asset);

        drop_company_deep(&pool, company_id).await;
    }

    /// AC 3 — **la garde ne mord que sur le type.** Renommer un compte
    /// mouvementé doit rester libre ; une garde qui déborderait sur le nom
    /// serait une régression silencieuse de la 14-3a.
    #[tokio::test]
    async fn renommer_un_compte_mouvemente_reste_libre() {
        let pool = test_pool().await;
        let admin = get_admin_user_id(&pool).await;
        let (company_id, fy) = mk_company_with_fy(&pool, "garde-renomme").await;

        let charge = mk_account(&pool, company_id, admin, "T917", AccountType::Expense).await;
        let caisse = mk_account(&pool, company_id, admin, "T918", AccountType::Asset).await;
        mk_entry(&pool, company_id, fy, admin, charge, caisse).await;

        let before = find_by_id(&pool, charge).await.unwrap().unwrap();
        let updated = update(
            &pool,
            charge,
            before.version,
            admin,
            AccountUpdate {
                name: "Nom modifié sans retypage".into(),
                account_type: before.account_type,
                role: None,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
                parent: Default::default(),
            },
            false,
        )
        .await
        .expect("renommer ne retype pas");

        assert_eq!(updated.name, "Nom modifié sans retypage");
        assert_eq!(updated.account_type, AccountType::Expense);

        drop_company_deep(&pool, company_id).await;
    }

    /// AC 9 — **l'ordre des gardes est un critère.** Le conflit de version
    /// précède la garde de retypage : sinon, confirmer un retypage écraserait la
    /// modification concurrente d'un tiers. Le compte est mouvementé ET la
    /// version est fausse ; c'est `OptimisticLockConflict` qui doit sortir, pas
    /// `AccountHasEntries`.
    #[tokio::test]
    async fn le_conflit_de_version_precede_la_garde_de_retypage() {
        let pool = test_pool().await;
        let admin = get_admin_user_id(&pool).await;
        let (company_id, fy) = mk_company_with_fy(&pool, "garde-version").await;

        let charge = mk_account(&pool, company_id, admin, "T919", AccountType::Expense).await;
        let caisse = mk_account(&pool, company_id, admin, "T920", AccountType::Asset).await;
        mk_entry(&pool, company_id, fy, admin, charge, caisse).await;

        let before = find_by_id(&pool, charge).await.unwrap().unwrap();
        let err = update(
            &pool,
            charge,
            before.version + 7, // version qui n'a jamais existé
            admin,
            AccountUpdate {
                name: before.name.clone(),
                account_type: AccountType::Asset,
                role: None,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
                parent: Default::default(),
            },
            false,
        )
        .await
        .unwrap_err();

        assert!(
            matches!(err, DbError::OptimisticLockConflict),
            "le verrou optimiste doit primer, reçu {err:?}"
        );

        drop_company_deep(&pool, company_id).await;
    }

    /// AC 5 — un retypage confirmé porte **son propre code d'action**, et il
    /// porte l'ampleur.
    ///
    /// ⛔ Noyé parmi les `account.updated`, ce geste serait introuvable le jour
    /// où quelqu'un cherche pourquoi le résultat d'une année close a bougé — et
    /// c'est précisément ce jour-là qu'on le cherche.
    #[tokio::test]
    async fn un_retypage_confirme_journalise_son_propre_code_d_action() {
        let pool = test_pool().await;
        let admin = get_admin_user_id(&pool).await;
        let (company_id, fy) = mk_company_with_fy(&pool, "audit-retype").await;

        let charge = mk_account(&pool, company_id, admin, "T921", AccountType::Expense).await;
        let caisse = mk_account(&pool, company_id, admin, "T922", AccountType::Asset).await;
        mk_entry(&pool, company_id, fy, admin, charge, caisse).await;
        sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
            .bind(fy)
            .execute(&pool)
            .await
            .expect("fermeture de l'exercice de test");

        let before = find_by_id(&pool, charge).await.unwrap().unwrap();
        update(
            &pool,
            charge,
            before.version,
            admin,
            AccountUpdate {
                name: before.name.clone(),
                account_type: AccountType::Asset,
                role: None,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
                parent: Default::default(),
            },
            true,
        )
        .await
        .expect("retypage confirmé");

        let entries = audit_log::find_by_entity(&pool, "account", charge, 10)
            .await
            .unwrap();

        let retyped = entries
            .iter()
            .find(|e| e.action == "account.retyped")
            .expect("une entrée account.retyped doit exister");
        let details = retyped
            .details_json
            .as_ref()
            .expect("details_json must be present");
        let trace = details.get("retype").expect("le bloc retype");
        assert_eq!(
            trace.get("fromType").and_then(|v| v.as_str()),
            Some("Expense")
        );
        assert_eq!(trace.get("toType").and_then(|v| v.as_str()), Some("Asset"));
        assert_eq!(trace.get("entryCount").and_then(|v| v.as_i64()), Some(1));
        assert_eq!(
            trace
                .get("closedFiscalYears")
                .and_then(|v| v.as_array())
                .and_then(|a| a.first())
                .and_then(|v| v.as_str()),
            Some("Exercice audit-retype"),
            "le journal doit NOMMER l'exercice clos reclassé"
        );

        // Et le geste ne se dédouble pas en un `account.updated` ordinaire, qui
        // le rendrait indistinguable d'un renommage.
        assert!(
            !entries.iter().any(|e| e.action == "account.updated"),
            "un retypage ne journalise PAS aussi account.updated"
        );

        drop_company_deep(&pool, company_id).await;
    }

    /// AC 5 — le code ordinaire reste celui des autres modifications : une garde
    /// qui déborderait sur le renommage rendrait le journal illisible.
    #[tokio::test]
    async fn un_renommage_garde_le_code_d_action_ordinaire() {
        let pool = test_pool().await;
        let admin = get_admin_user_id(&pool).await;
        let (company_id, fy) = mk_company_with_fy(&pool, "audit-renomme").await;

        let charge = mk_account(&pool, company_id, admin, "T923", AccountType::Expense).await;
        let caisse = mk_account(&pool, company_id, admin, "T924", AccountType::Asset).await;
        mk_entry(&pool, company_id, fy, admin, charge, caisse).await;

        let before = find_by_id(&pool, charge).await.unwrap().unwrap();
        update(
            &pool,
            charge,
            before.version,
            admin,
            AccountUpdate {
                name: "Libellé corrigé".into(),
                account_type: before.account_type,
                role: None,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
                parent: Default::default(),
            },
            false,
        )
        .await
        .expect("renommer ne retype pas");

        let entries = audit_log::find_by_entity(&pool, "account", charge, 10)
            .await
            .unwrap();
        assert!(entries.iter().any(|e| e.action == "account.updated"));
        assert!(!entries.iter().any(|e| e.action == "account.retyped"));

        drop_company_deep(&pool, company_id).await;
    }

    /// AC 5 — **un compte vierge retypé reste un `account.updated`.** Sans
    /// écriture, il n'y a pas d'historique à reclasser, donc rien à signaler à
    /// un contrôleur. Sans ce test, l'affirmation du commentaire de `update` ne
    /// serait éprouvée par rien.
    #[tokio::test]
    async fn retyper_un_compte_vierge_reste_une_modification_ordinaire() {
        let pool = test_pool().await;
        let admin = get_admin_user_id(&pool).await;
        let (company_id, _fy) = mk_company_with_fy(&pool, "audit-vierge").await;

        let compte = mk_account(&pool, company_id, admin, "T925", AccountType::Expense).await;
        let before = find_by_id(&pool, compte).await.unwrap().unwrap();

        update(
            &pool,
            compte,
            before.version,
            admin,
            AccountUpdate {
                name: before.name.clone(),
                account_type: AccountType::Asset,
                role: None,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
                parent: Default::default(),
            },
            false,
        )
        .await
        .expect("un compte vierge se retype librement");

        let entries = audit_log::find_by_entity(&pool, "account", compte, 10)
            .await
            .unwrap();
        assert!(entries.iter().any(|e| e.action == "account.updated"));
        assert!(
            !entries.iter().any(|e| e.action == "account.retyped"),
            "sans écriture, rien n'est reclassé : pas de code distinct"
        );

        drop_company_deep(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_create_and_find() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        let new = NewAccount {
            company_id,
            number: "T100".into(),
            name: "Test Create".into(),
            account_type: AccountType::Asset,
            parent_id: None,
            role: None,
            postable: true,
            currency: Default::default(),
            own_fx_accounts: Default::default(),
        };
        let account = create(&pool, admin_user_id, new).await.unwrap();
        assert_eq!(account.number, "T100");
        assert_eq!(account.name, "Test Create");
        assert_eq!(account.account_type, AccountType::Asset);
        assert!(account.active);
        assert_eq!(account.version, 1);

        let found = find_by_id(&pool, account.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().number, "T100");

        cleanup_test_accounts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_list_by_company_filters_archived() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        // Créer un compte actif et un archivé
        let active = create(
            &pool,
            admin_user_id,
            NewAccount {
                company_id,
                number: "T200".into(),
                name: "Active".into(),
                account_type: AccountType::Revenue,
                parent_id: None,
                role: None,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
            },
        )
        .await
        .unwrap();

        let archived = create(
            &pool,
            admin_user_id,
            NewAccount {
                company_id,
                number: "T201".into(),
                name: "To Archive".into(),
                account_type: AccountType::Expense,
                parent_id: None,
                role: None,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
            },
        )
        .await
        .unwrap();
        archive(&pool, archived.id, archived.version, admin_user_id)
            .await
            .unwrap();

        // Sans archivés
        let without = list_by_company(&pool, company_id, false).await.unwrap();
        assert!(without.iter().any(|a| a.id == active.id));
        assert!(!without.iter().any(|a| a.id == archived.id));

        // Avec archivés
        let with = list_by_company(&pool, company_id, true).await.unwrap();
        assert!(with.iter().any(|a| a.id == active.id));
        assert!(with.iter().any(|a| a.id == archived.id));

        cleanup_test_accounts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_update_optimistic_locking() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        let account = create(
            &pool,
            admin_user_id,
            NewAccount {
                company_id,
                number: "T300".into(),
                name: "Original".into(),
                account_type: AccountType::Asset,
                parent_id: None,
                role: None,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
            },
        )
        .await
        .unwrap();

        // Update réussit avec la bonne version
        let updated = update(
            &pool,
            account.id,
            account.version,
            admin_user_id,
            AccountUpdate {
                name: "Updated".into(),
                account_type: AccountType::Liability,
                role: None,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
                parent: Default::default(),
            },
            false,
        )
        .await
        .unwrap();
        assert_eq!(updated.name, "Updated");
        assert_eq!(updated.account_type, AccountType::Liability);
        assert_eq!(updated.version, 2);

        // Update échoue avec l'ancienne version
        let err = update(
            &pool,
            account.id,
            account.version, // version 1, mais en DB c'est 2
            admin_user_id,
            AccountUpdate {
                name: "Should Fail".into(),
                account_type: AccountType::Asset,
                role: None,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
                parent: Default::default(),
            },
            false,
        )
        .await
        .unwrap_err();
        assert!(matches!(err, DbError::OptimisticLockConflict));

        cleanup_test_accounts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_archive_sets_inactive() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        let account = create(
            &pool,
            admin_user_id,
            NewAccount {
                company_id,
                number: "T400".into(),
                name: "To Archive".into(),
                account_type: AccountType::Asset,
                parent_id: None,
                role: None,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
            },
        )
        .await
        .unwrap();
        assert!(account.active);

        let archived = archive(&pool, account.id, account.version, admin_user_id)
            .await
            .unwrap();
        assert!(!archived.active);
        assert_eq!(archived.version, 2);

        cleanup_test_accounts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_unique_constraint_on_company_number() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        create(
            &pool,
            admin_user_id,
            NewAccount {
                company_id,
                number: "T500".into(),
                name: "First".into(),
                account_type: AccountType::Asset,
                parent_id: None,
                role: None,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
            },
        )
        .await
        .unwrap();

        // Duplicate number → UniqueConstraintViolation
        let err = create(
            &pool,
            admin_user_id,
            NewAccount {
                company_id,
                number: "T500".into(),
                name: "Duplicate".into(),
                account_type: AccountType::Asset,
                parent_id: None,
                role: None,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
            },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, DbError::UniqueConstraintViolation(_)));

        cleanup_test_accounts(&pool, company_id).await;
    }

    /// Story 3.5 — vérifie que `create` insère une entrée `audit_log` avec
    /// `action = "account.created"` et un snapshot direct (pas de wrapper).
    #[tokio::test]
    async fn test_create_account_writes_audit_log() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        let account = create(
            &pool,
            admin_user_id,
            NewAccount {
                company_id,
                number: "T600".into(),
                name: "Audit Create".into(),
                account_type: AccountType::Asset,
                parent_id: None,
                role: None,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
            },
        )
        .await
        .unwrap();

        let entries = audit_log::find_by_entity(&pool, "account", account.id, 10)
            .await
            .unwrap();
        let created_audit = entries
            .iter()
            .find(|e| e.action == "account.created")
            .expect("audit entry with action account.created must exist");

        assert_eq!(created_audit.user_id, admin_user_id);
        assert_eq!(created_audit.entity_type, "account");
        assert_eq!(created_audit.entity_id, account.id);

        let details = created_audit
            .details_json
            .as_ref()
            .expect("details_json must be present");
        // Convention projet : snapshot direct pour create (pas de wrapper).
        assert!(details.get("before").is_none());
        assert!(details.get("after").is_none());
        assert_eq!(details.get("number").and_then(|v| v.as_str()), Some("T600"));
        assert_eq!(
            details.get("name").and_then(|v| v.as_str()),
            Some("Audit Create")
        );

        cleanup_test_accounts(&pool, company_id).await;
    }

    /// Story 3.5 — vérifie que `update` insère une entrée `audit_log` avec
    /// `action = "account.updated"` et un wrapper `{before, after}`.
    #[tokio::test]
    async fn test_update_account_writes_audit_log() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        let account = create(
            &pool,
            admin_user_id,
            NewAccount {
                company_id,
                number: "T601".into(),
                name: "Before Name".into(),
                account_type: AccountType::Asset,
                parent_id: None,
                role: None,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
            },
        )
        .await
        .unwrap();

        let updated = update(
            &pool,
            account.id,
            account.version,
            admin_user_id,
            AccountUpdate {
                name: "After Name".into(),
                account_type: AccountType::Liability,
                role: None,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
                parent: Default::default(),
            },
            false,
        )
        .await
        .unwrap();

        let entries = audit_log::find_by_entity(&pool, "account", updated.id, 10)
            .await
            .unwrap();
        let update_audit = entries
            .iter()
            .find(|e| e.action == "account.updated")
            .expect("audit entry with action account.updated must exist");

        let details = update_audit
            .details_json
            .as_ref()
            .expect("details_json must be present");

        // Convention projet : update utilise un wrapper {before, after}.
        let before = details
            .get("before")
            .expect("update audit must wrap snapshot in {{before, after}}");
        let after = details
            .get("after")
            .expect("update audit must wrap snapshot in {{before, after}}");

        assert_eq!(
            before.get("name").and_then(|v| v.as_str()),
            Some("Before Name")
        );
        assert_eq!(
            after.get("name").and_then(|v| v.as_str()),
            Some("After Name")
        );
        assert_eq!(
            before.get("accountType").and_then(|v| v.as_str()),
            Some("Asset")
        );
        assert_eq!(
            after.get("accountType").and_then(|v| v.as_str()),
            Some("Liability")
        );

        cleanup_test_accounts(&pool, company_id).await;
    }

    /// Story 3.5 — vérifie que `archive` insère une entrée `audit_log` avec
    /// `action = "account.archived"` et un snapshot direct.
    #[tokio::test]
    async fn test_archive_account_writes_audit_log() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        let account = create(
            &pool,
            admin_user_id,
            NewAccount {
                company_id,
                number: "T602".into(),
                name: "To Archive Audit".into(),
                account_type: AccountType::Asset,
                parent_id: None,
                role: None,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
            },
        )
        .await
        .unwrap();

        let archived = archive(&pool, account.id, account.version, admin_user_id)
            .await
            .unwrap();

        let entries = audit_log::find_by_entity(&pool, "account", archived.id, 10)
            .await
            .unwrap();
        let archive_audit = entries
            .iter()
            .find(|e| e.action == "account.archived")
            .expect("audit entry with action account.archived must exist");

        assert_eq!(archive_audit.user_id, admin_user_id);

        let details = archive_audit
            .details_json
            .as_ref()
            .expect("details_json must be present");
        // Snapshot direct (pas de wrapper).
        assert!(details.get("before").is_none());
        assert!(details.get("after").is_none());
        assert_eq!(details.get("active").and_then(|v| v.as_bool()), Some(false));

        cleanup_test_accounts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_bulk_create_from_chart() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        // Charger un petit sous-ensemble du plan PME
        let entries = vec![
            kesh_core::chart_of_accounts::ChartEntry {
                number: "T1".into(),
                name: std::collections::HashMap::from([
                    ("fr".into(), "Test Actifs".into()),
                    ("de".into(), "Test Aktiven".into()),
                    ("it".into(), "Test Attivi".into()),
                    ("en".into(), "Test Assets".into()),
                ]),
                account_type: kesh_core::chart_of_accounts::AccountType::Asset,
                parent_number: None,
                role: None,
                postable: None,
                rounding_difference: false,
                write_off_nature: None,
            },
            kesh_core::chart_of_accounts::ChartEntry {
                number: "T10".into(),
                name: std::collections::HashMap::from([
                    ("fr".into(), "Test Circulants".into()),
                    ("de".into(), "Test Umlauf".into()),
                    ("it".into(), "Test Circolante".into()),
                    ("en".into(), "Test Current".into()),
                ]),
                account_type: kesh_core::chart_of_accounts::AccountType::Asset,
                parent_number: Some("T1".into()),
                role: None,
                postable: None,
                rounding_difference: false,
                write_off_nature: None,
            },
        ];

        let created = bulk_create_from_chart(&pool, company_id, &entries, "fr")
            .await
            .unwrap();
        assert_eq!(created.len(), 2);

        let root = created.iter().find(|a| a.number == "T1").unwrap();
        assert_eq!(root.name, "Test Actifs");
        assert!(root.parent_id.is_none());

        let child = created.iter().find(|a| a.number == "T10").unwrap();
        assert_eq!(child.name, "Test Circulants");
        assert_eq!(child.parent_id, Some(root.id));

        cleanup_test_accounts(&pool, company_id).await;
    }

    /// KF-004 : payload identique → pas de bump version, pas d'audit_log.
    #[tokio::test]
    async fn update_no_op_returns_unchanged_entity_no_audit() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        let account = create(
            &pool,
            admin_user_id,
            NewAccount {
                company_id,
                number: "T800".into(),
                name: "Test NoOp".into(),
                account_type: AccountType::Revenue,
                parent_id: None,
                role: None,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
            },
        )
        .await
        .unwrap();
        let version_initial = account.version;
        let updated_at_initial = account.updated_at;

        let result = update(
            &pool,
            account.id,
            version_initial,
            admin_user_id,
            AccountUpdate {
                name: account.name.clone(),
                account_type: account.account_type,
                role: None,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
                parent: Default::default(),
            },
            false,
        )
        .await
        .unwrap();

        assert_eq!(result.version, version_initial);
        assert_eq!(result.updated_at, updated_at_initial);

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM audit_log WHERE entity_type = 'account' AND entity_id = ? AND action = 'account.updated'",
        )
        .bind(account.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count.0, 0);

        cleanup_test_accounts(&pool, company_id).await;
    }

    /// KF-004 régression : modifier `name` → bump version + audit log présent.
    #[tokio::test]
    async fn update_partial_change_bumps_version() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        let account = create(
            &pool,
            admin_user_id,
            NewAccount {
                company_id,
                number: "T801".into(),
                name: "Test Rename".into(),
                account_type: AccountType::Asset,
                parent_id: None,
                role: None,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
            },
        )
        .await
        .unwrap();
        let version_initial = account.version;

        let result = update(
            &pool,
            account.id,
            version_initial,
            admin_user_id,
            AccountUpdate {
                name: "Test Rename Updated".into(),
                account_type: account.account_type,
                role: None,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
                parent: Default::default(),
            },
            false,
        )
        .await
        .unwrap();
        assert_eq!(result.version, version_initial + 1);
        assert_eq!(result.name, "Test Rename Updated");

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM audit_log WHERE entity_type = 'account' AND entity_id = ? AND action = 'account.updated'",
        )
        .bind(account.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count.0, 1);

        cleanup_test_accounts(&pool, company_id).await;
    }

    // =======================================================================
    // Story 14-3a — rôles de comptes & postabilité
    // =======================================================================

    /// Helper : crée un compte de test avec rôle et postabilité explicites.
    async fn mk(
        pool: &MySqlPool,
        company_id: i64,
        user_id: i64,
        number: &str,
        role: Option<AccountRole>,
        postable: bool,
    ) -> Account {
        create(
            pool,
            user_id,
            NewAccount::new(
                company_id,
                number,
                format!("Test {number}"),
                AccountType::Asset,
                None,
            )
            .with_role(role, postable),
        )
        .await
        .unwrap_or_else(|e| panic!("create {number} failed: {e:?}"))
    }

    #[tokio::test]
    async fn role_and_postable_round_trip() {
        let pool = test_pool().await;
        let company_id = mk_role_company(&pool, "round-trip").await;
        let user_id = get_admin_user_id(&pool).await;

        let a = mk(
            &pool,
            company_id,
            user_id,
            "T900",
            Some(AccountRole::VatPayable),
            false,
        )
        .await;
        assert_eq!(a.role, Some(AccountRole::VatPayable));
        assert!(!a.postable);

        // Relecture par un autre chemin (find_by_id) → Decode correct.
        let reread = find_by_id(&pool, a.id).await.unwrap().unwrap();
        assert_eq!(reread.role, Some(AccountRole::VatPayable));
        assert!(!reread.postable);

        // Et par find_by_id_in_company, qui a sa propre liste de colonnes.
        let scoped = find_by_id_in_company(&pool, a.id, company_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(scoped.role, Some(AccountRole::VatPayable));

        drop_role_company(&pool, company_id).await;
    }

    #[tokio::test]
    async fn update_role_only_bumps_version_and_audits() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        let a = mk(&pool, company_id, user_id, "T901", None, true).await;
        let updated = update(
            &pool,
            a.id,
            a.version,
            user_id,
            AccountUpdate {
                name: a.name.clone(),
                account_type: a.account_type,
                role: Some(AccountRole::EquityCapital),
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
                parent: Default::default(),
            },
            false,
        )
        .await
        .unwrap();

        assert_eq!(updated.role, Some(AccountRole::EquityCapital));
        assert_eq!(
            updated.version,
            a.version + 1,
            "changer le rôle DOIT bumper la version"
        );

        // L'audit doit porter le diff de rôle (sinon il mentirait).
        let details: (serde_json::Value,) = sqlx::query_as(
            "SELECT details_json FROM audit_log WHERE entity_type = 'account' AND entity_id = ? \
             AND action = 'account.updated' ORDER BY id DESC LIMIT 1",
        )
        .bind(a.id)
        .fetch_one(&pool)
        .await
        .expect("audit account.updated attendu");
        assert_eq!(details.0["before"]["role"], serde_json::Value::Null);
        assert_eq!(details.0["after"]["role"], "EquityCapital");

        cleanup_test_accounts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn update_no_op_includes_role_and_postable() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        let a = mk(
            &pool,
            company_id,
            user_id,
            "T902",
            Some(AccountRole::EquityOther),
            false,
        )
        .await;
        let same = update(
            &pool,
            a.id,
            a.version,
            user_id,
            AccountUpdate {
                name: a.name.clone(),
                account_type: a.account_type,
                role: a.role,
                postable: a.postable,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
                parent: Default::default(),
            },
            false,
        )
        .await
        .unwrap();
        assert_eq!(same.version, a.version, "no-op strict → pas de bump");

        // Changer UNIQUEMENT postable n'est pas un no-op.
        let toggled = update(
            &pool,
            a.id,
            a.version,
            user_id,
            AccountUpdate {
                name: a.name.clone(),
                account_type: a.account_type,
                role: a.role,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
                parent: Default::default(),
            },
            false,
        )
        .await
        .unwrap();
        assert_eq!(toggled.version, a.version + 1);
        assert!(toggled.postable);

        cleanup_test_accounts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn singleton_role_rejected_twice_in_same_company() {
        let pool = test_pool().await;
        let company_id = mk_role_company(&pool, "dup-singleton").await;
        let user_id = get_admin_user_id(&pool).await;

        let holder = mk(
            &pool,
            company_id,
            user_id,
            "T910",
            Some(AccountRole::Receivable),
            true,
        )
        .await;
        let other = mk(&pool, company_id, user_id, "T911", None, true).await;

        let err = update(
            &pool,
            other.id,
            other.version,
            user_id,
            AccountUpdate {
                name: other.name.clone(),
                account_type: other.account_type,
                role: Some(AccountRole::Receivable),
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
                parent: Default::default(),
            },
            false,
        )
        .await
        .expect_err("le rôle Receivable est déjà pris");

        match err {
            DbError::AccountRoleAlreadyAssigned {
                ref role,
                account_id,
                ref account_number,
                ..
            } => {
                assert_eq!(role, "Receivable");
                assert_eq!(
                    account_id, holder.id,
                    "l'erreur doit NOMMER le compte en conflit"
                );
                assert_eq!(account_number, "T910");
            }
            other => panic!("attendu AccountRoleAlreadyAssigned, obtenu {other:?}"),
        }

        drop_role_company(&pool, company_id).await;
    }

    #[tokio::test]
    async fn multi_valued_role_allowed_twice() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        // EquityOther n'est PAS singleton : 2850 + 2860 coexistent dans les
        // plans association et indépendant.
        mk(
            &pool,
            company_id,
            user_id,
            "T920",
            Some(AccountRole::EquityOther),
            true,
        )
        .await;
        mk(
            &pool,
            company_id,
            user_id,
            "T921",
            Some(AccountRole::EquityOther),
            true,
        )
        .await;
        mk(
            &pool,
            company_id,
            user_id,
            "T922",
            Some(AccountRole::EquityCapital),
            true,
        )
        .await;

        cleanup_test_accounts(&pool, company_id).await;
    }

    /// Le test le plus important de la story : l'archivage LIBÈRE le rôle
    /// singleton, et la réactivation est refusée si le rôle a été repris.
    #[tokio::test]
    async fn singleton_role_is_released_on_archive_and_blocks_reactivation() {
        let pool = test_pool().await;
        let company_id = mk_role_company(&pool, "release-on-archive").await;
        let user_id = get_admin_user_id(&pool).await;

        // A porte le rôle.
        let a = mk(
            &pool,
            company_id,
            user_id,
            "T930",
            Some(AccountRole::Payable),
            true,
        )
        .await;

        // Archiver A → le rôle est libéré (singleton_role repasse à NULL).
        let a = archive(&pool, a.id, a.version, user_id).await.unwrap();
        assert!(!a.active);
        assert_eq!(
            a.role,
            Some(AccountRole::Payable),
            "archiver ne doit PAS effacer le rôle"
        );

        // B, actif, peut désormais prendre le rôle.
        let b = mk(
            &pool,
            company_id,
            user_id,
            "T931",
            Some(AccountRole::Payable),
            true,
        )
        .await;

        // Réactiver A est refusé, en nommant B.
        let err = reactivate(&pool, a.id, a.version, user_id, false)
            .await
            .expect_err("le rôle Payable a été repris par B");
        match err {
            DbError::AccountRoleAlreadyAssigned { account_id, .. } => {
                assert_eq!(account_id, b.id);
            }
            other => panic!("attendu AccountRoleAlreadyAssigned, obtenu {other:?}"),
        }

        // Libérer le rôle de B → A redevient réactivable.
        update(
            &pool,
            b.id,
            b.version,
            user_id,
            AccountUpdate {
                name: b.name.clone(),
                account_type: b.account_type,
                role: None,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
                parent: Default::default(),
            },
            false,
        )
        .await
        .unwrap();

        let a = reactivate(&pool, a.id, a.version, user_id, false)
            .await
            .unwrap();
        assert!(a.active);
        assert_eq!(a.role, Some(AccountRole::Payable));

        drop_role_company(&pool, company_id).await;
    }

    /// Code review 14-3a (D4) : plutôt que la séquence en quatre étapes imposée
    /// à l'utilisateur, `clear_role` réactive le compte en lui retirant le rôle.
    #[tokio::test]
    async fn reactivate_with_clear_role_drops_the_conflicting_role() {
        let pool = test_pool().await;
        let company_id = mk_role_company(&pool, "clear-role").await;
        let user_id = get_admin_user_id(&pool).await;

        let a = mk(
            &pool,
            company_id,
            user_id,
            "T950",
            Some(AccountRole::Receivable),
            true,
        )
        .await;
        let a = archive(&pool, a.id, a.version, user_id).await.unwrap();

        // B reprend le rôle pendant l'archivage.
        mk(
            &pool,
            company_id,
            user_id,
            "T951",
            Some(AccountRole::Receivable),
            true,
        )
        .await;

        // Sans le drapeau : refus (comportement nominal conservé).
        let err = reactivate(&pool, a.id, a.version, user_id, false)
            .await
            .expect_err("le rôle a été repris");
        assert!(
            matches!(err, DbError::AccountRoleAlreadyAssigned { .. }),
            "obtenu {err:?}"
        );

        // Avec le drapeau : réactivé, sans le rôle.
        let back = reactivate(&pool, a.id, a.version, user_id, true)
            .await
            .expect("réactivation sans le rôle");
        assert!(back.active);
        assert_eq!(back.role, None, "le rôle doit avoir été retiré");
        assert_eq!(back.version, a.version + 1);

        drop_role_company(&pool, company_id).await;
    }

    /// AC-G : le même rôle singleton dans DEUX sociétés est légitime — c'est
    /// tout l'objet du `company_id` dans `uq_accounts_company_singleton_role`.
    /// Sans ce test, une migration retirant `company_id` de la contrainte
    /// passerait toute la suite au vert.
    #[tokio::test]
    async fn same_singleton_role_allowed_in_two_companies() {
        let pool = test_pool().await;
        let a_id = mk_role_company(&pool, "tenant-a").await;
        let b_id = mk_role_company(&pool, "tenant-b").await;
        let user_id = get_admin_user_id(&pool).await;

        mk(
            &pool,
            a_id,
            user_id,
            "T960",
            Some(AccountRole::Receivable),
            true,
        )
        .await;
        mk(
            &pool,
            b_id,
            user_id,
            "T960",
            Some(AccountRole::Receivable),
            true,
        )
        .await;

        drop_role_company(&pool, a_id).await;
        drop_role_company(&pool, b_id).await;
    }

    /// Code review 14-3a (HIGH) : le conflit de rôle doit être détecté au
    /// `create`, pas seulement à l'`update`. Sinon le 1062 remonte en
    /// `UniqueConstraintViolation` générique et le formulaire affiche
    /// « ce numéro existe déjà » alors que le numéro est libre.
    #[tokio::test]
    async fn create_rejects_taken_singleton_role_and_names_the_holder() {
        let pool = test_pool().await;
        let company_id = mk_role_company(&pool, "create-conflict").await;
        let user_id = get_admin_user_id(&pool).await;

        let holder = mk(
            &pool,
            company_id,
            user_id,
            "T970",
            Some(AccountRole::Receivable),
            true,
        )
        .await;

        let err = create(
            &pool,
            user_id,
            NewAccount::new(company_id, "T971", "Autre", AccountType::Asset, None)
                .with_role(Some(AccountRole::Receivable), true),
        )
        .await
        .expect_err("le rôle Receivable est déjà porté par T970");

        match err {
            DbError::AccountRoleAlreadyAssigned {
                account_id,
                ref account_number,
                ..
            } => {
                assert_eq!(account_id, holder.id);
                assert_eq!(account_number, "T970");
            }
            other => panic!("attendu AccountRoleAlreadyAssigned, obtenu {other:?}"),
        }

        // Un rôle multi-valué reste acceptable deux fois au `create`.
        create(
            &pool,
            user_id,
            NewAccount::new(company_id, "T972", "Fonds", AccountType::Liability, None)
                .with_role(Some(AccountRole::EquityOther), true),
        )
        .await
        .unwrap();
        create(
            &pool,
            user_id,
            NewAccount::new(company_id, "T973", "Fonds 2", AccountType::Liability, None)
                .with_role(Some(AccountRole::EquityOther), true),
        )
        .await
        .unwrap();

        drop_role_company(&pool, company_id).await;
    }

    /// Code review 14-3a (D1) : la frontière bilan / résultat est la seule
    /// contrainte de type, et elle vaut pour `create` comme pour `update`.
    #[tokio::test]
    async fn role_must_match_the_side_of_the_chart() {
        let pool = test_pool().await;
        let company_id = mk_role_company(&pool, "role-type").await;
        let user_id = get_admin_user_id(&pool).await;

        // `Payable` sur une charge → refus au create.
        let err = create(
            &pool,
            user_id,
            NewAccount::new(company_id, "T980", "Charges", AccountType::Expense, None)
                .with_role(Some(AccountRole::Payable), true),
        )
        .await
        .expect_err("Payable sur une charge");
        match err {
            DbError::AccountRoleInvalidForType {
                ref role,
                ref account_type,
            } => {
                assert_eq!(role, "Payable");
                assert_eq!(account_type, "Expense");
            }
            other => panic!("attendu AccountRoleInvalidForType, obtenu {other:?}"),
        }

        // …et au update.
        let a = mk(&pool, company_id, user_id, "T981", None, true).await;
        let err = update(
            &pool,
            a.id,
            a.version,
            user_id,
            AccountUpdate {
                name: a.name.clone(),
                account_type: AccountType::Expense,
                role: Some(AccountRole::Receivable),
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
                parent: Default::default(),
            },
            false,
        )
        .await
        .expect_err("Receivable sur une charge");
        assert!(
            matches!(err, DbError::AccountRoleInvalidForType { .. }),
            "obtenu {err:?}"
        );

        // `DefaultRevenue` exige un compte de produit — et l'accepte.
        create(
            &pool,
            user_id,
            NewAccount::new(company_id, "T982", "Ventes", AccountType::Revenue, None)
                .with_role(Some(AccountRole::DefaultRevenue), true),
        )
        .await
        .expect("DefaultRevenue sur un produit est valide");

        drop_role_company(&pool, company_id).await;
    }

    /// Code review 14-3a (D1/D2) : `postable` est normalisé à chaque écriture,
    /// pas seulement au seed.
    #[tokio::test]
    async fn postable_is_normalised_on_every_write() {
        let pool = test_pool().await;
        let company_id = mk_role_company(&pool, "postable-norm").await;
        let user_id = get_admin_user_id(&pool).await;

        // 1. Le compte de résultat de l'exercice n'est jamais postable, même si
        //    l'appelant demande explicitement `postable: true`.
        let result_account = create(
            &pool,
            user_id,
            NewAccount::new(company_id, "T990", "Résultat", AccountType::Liability, None)
                .with_role(Some(AccountRole::CurrentYearResult), true),
        )
        .await
        .unwrap();
        assert!(
            !result_account.postable,
            "CurrentYearResult doit être forcé non-postable"
        );

        // 2. Créer un enfant rend le parent non-postable.
        let parent = mk(&pool, company_id, user_id, "T991", None, true).await;
        assert!(parent.postable);
        create(
            &pool,
            user_id,
            NewAccount::new(
                company_id,
                "T992",
                "Enfant",
                AccountType::Asset,
                Some(parent.id),
            ),
        )
        .await
        .unwrap();
        let parent = find_by_id(&pool, parent.id).await.unwrap().unwrap();
        assert!(
            !parent.postable,
            "un compte qui acquiert un sous-compte devient un compte de regroupement"
        );

        // 3. Un client dont la page date d'avant ne peut pas le rétablir : le
        //    contrat full-replace du PUT est normalisé à l'écriture. Sans cela,
        //    l'invariant tiendrait au create et se dégraderait au premier update.
        let same = update(
            &pool,
            parent.id,
            parent.version,
            user_id,
            AccountUpdate {
                name: parent.name.clone(),
                account_type: parent.account_type,
                role: parent.role,
                postable: true,
                currency: Default::default(),
                own_fx_accounts: Default::default(),
                parent: Default::default(),
            },
            false,
        )
        .await
        .unwrap();
        assert!(
            !same.postable,
            "la normalisation doit gagner sur le payload"
        );
        assert_eq!(
            same.version, parent.version,
            "après normalisation il n'y a plus de changement → no-op, pas de bump"
        );

        drop_role_company(&pool, company_id).await;
    }

    #[tokio::test]
    async fn reactivate_nominal_idempotent_and_guards() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        let parent = mk(&pool, company_id, user_id, "T940", None, true).await;
        let child = create(
            &pool,
            user_id,
            NewAccount::new(
                company_id,
                "T941",
                "Test enfant",
                AccountType::Asset,
                Some(parent.id),
            ),
        )
        .await
        .unwrap();

        // --- nominal : archiver puis réactiver
        let archived = archive(&pool, child.id, child.version, user_id)
            .await
            .unwrap();
        assert!(!archived.active);
        let back = reactivate(&pool, archived.id, archived.version, user_id, false)
            .await
            .unwrap();
        assert!(back.active);
        assert_eq!(back.version, archived.version + 1);

        // audit account.reactivated écrit
        let n: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM audit_log WHERE entity_type = 'account' AND entity_id = ? \
             AND action = 'account.reactivated'",
        )
        .bind(child.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(n.0, 1);

        // --- déjà actif : no-op, pas de bump, pas d'audit supplémentaire
        let noop = reactivate(&pool, back.id, back.version, user_id, false)
            .await
            .unwrap();
        assert_eq!(
            noop.version, back.version,
            "réactiver un compte actif = no-op"
        );
        let n2: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM audit_log WHERE entity_type = 'account' AND entity_id = ? \
             AND action = 'account.reactivated'",
        )
        .bind(child.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(n2.0, 1, "no-op ne doit PAS écrire d'audit");

        // --- déjà actif MAIS version périmée → conflit, pas no-op silencieux.
        //     Code review 14-3a : le no-op était évalué avant le contrôle de
        //     version, si bien qu'un client travaillant sur une page obsolète
        //     recevait 200 avec une entité qu'il n'avait jamais vue.
        let err = reactivate(&pool, back.id, back.version + 99, user_id, false)
            .await
            .expect_err("version périmée sur un compte déjà actif");
        assert!(
            matches!(err, DbError::OptimisticLockConflict),
            "attendu OptimisticLockConflict, obtenu {err:?}"
        );

        // --- mauvaise version → conflit optimiste
        let archived = archive(&pool, back.id, back.version, user_id)
            .await
            .unwrap();
        let err = reactivate(&pool, archived.id, archived.version + 99, user_id, false)
            .await
            .expect_err("version incorrecte");
        assert!(
            matches!(err, DbError::OptimisticLockConflict),
            "obtenu {err:?}"
        );

        // --- parent archivé → refus, avec une erreur qui NOMME le parent
        //     (le générique IllegalStateTransition était mappé côté API sur
        //     « Transition d'état interdite », sans dire quoi corriger).
        archive(&pool, parent.id, parent.version, user_id)
            .await
            .unwrap();
        let err = reactivate(&pool, archived.id, archived.version, user_id, false)
            .await
            .expect_err("parent archivé");
        match err {
            DbError::AccountParentArchived { ref parent_number } => {
                assert_eq!(parent_number, "T940");
            }
            other => panic!("attendu AccountParentArchived, obtenu {other:?}"),
        }

        cleanup_test_accounts(&pool, company_id).await;
    }

    /// Garde-fou central : la liste des rôles singleton vit à TROIS endroits
    /// (SQL de la migration, `kesh-db`, `kesh-core`). Ce test compare la source
    /// Rust à l'expression SQL **réellement en base** — sans lui, les deux enums
    /// Rust pourraient dériver de concert avec le SQL sans que rien n'échoue.
    #[tokio::test]
    async fn singleton_list_matches_sql_generation_expression() {
        let pool = test_pool().await;

        let expr: (String,) = sqlx::query_as(
            "SELECT GENERATION_EXPRESSION FROM information_schema.COLUMNS \
             WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'accounts' \
               AND COLUMN_NAME = 'singleton_role'",
        )
        .fetch_one(&pool)
        .await
        .expect("colonne générée singleton_role introuvable");

        // Code review 14-3a : le `active AND` n'est PAS cosmétique — c'est lui
        // qui libère le rôle à l'archivage et le fait reprendre à la
        // réactivation. En n'extrayant que les littéraux entre quotes, ce test
        // restait vert si un refactor le supprimait, alors qu'en production un
        // compte archivé aurait squatté son rôle à vie.
        let normalised = expr.0.replace('`', "").to_lowercase();
        assert!(
            normalised.contains("active"),
            "l'expression générée doit rester conditionnée par `active` : {}",
            expr.0
        );

        // Extrait les littéraux entre quotes simples de l'expression normalisée
        // par MariaDB, p.ex. : case when `active` <> 0 and `role` in ('A','B') …
        let mut from_sql: Vec<String> = expr
            .0
            .split('\'')
            .skip(1)
            .step_by(2)
            .map(|s| s.to_string())
            .collect();
        from_sql.sort();
        from_sql.dedup();

        let from_rust: Vec<String> = AccountRole::singletons()
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        assert_eq!(
            from_sql, from_rust,
            "la liste des rôles singleton du SQL ({from_sql:?}) diverge de celle de Rust ({from_rust:?}) \
             — synchroniser migration + kesh-db + kesh-core"
        );

        // Et les deux enums Rust doivent être identiques entre eux.
        let core: Vec<&str> = kesh_core::chart_of_accounts::AccountRole::ALL
            .iter()
            .map(|r| r.as_str())
            .collect();
        let db: Vec<&str> = AccountRole::ALL.iter().map(|r| r.as_str()).collect();
        assert_eq!(core, db, "les deux enums AccountRole ont divergé");

        for (c, d) in kesh_core::chart_of_accounts::AccountRole::ALL
            .iter()
            .zip(AccountRole::ALL.iter())
        {
            assert_eq!(
                c.is_singleton(),
                d.is_singleton(),
                "is_singleton() diverge pour {}",
                c.as_str()
            );
        }

        // Et la liste FERMÉE des 10 rôles du CHECK doit couvrir l'enum : un 11ᵉ
        // rôle ajouté côté Rust sans migration passerait la comparaison
        // ci-dessus (elle ne porte que sur les 8 singletons) puis échouerait au
        // premier POST en 4025 « CONSTRAINT chk_accounts_role failed ».
        let check: (String,) = sqlx::query_as(
            "SELECT CHECK_CLAUSE FROM information_schema.CHECK_CONSTRAINTS \
             WHERE CONSTRAINT_SCHEMA = DATABASE() AND CONSTRAINT_NAME = 'chk_accounts_role'",
        )
        .fetch_one(&pool)
        .await
        .expect("contrainte chk_accounts_role introuvable");

        let mut from_check: Vec<String> = check
            .0
            .split('\'')
            .skip(1)
            .step_by(2)
            .map(|s| s.to_string())
            .collect();
        from_check.sort();
        from_check.dedup();

        let mut all_rust: Vec<String> = AccountRole::ALL
            .iter()
            .map(|r| r.as_str().to_string())
            .collect();
        all_rust.sort();

        assert_eq!(
            from_check, all_rust,
            "la liste des rôles du CHECK SQL ({from_check:?}) diverge de AccountRole::ALL ({all_rust:?}) \
             — ajouter une migration ALTER … DROP CHECK / ADD CHECK"
        );
    }
}
