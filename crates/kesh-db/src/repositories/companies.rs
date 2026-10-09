//! Repository CRUD pour `Company`.
//!
//! MySQL/MariaDB n'a pas de clause `RETURNING` (contrairement à Postgres),
//! d'où le pattern `create` en deux étapes : INSERT puis SELECT via `find_by_id`.
//! Pour garantir l'atomicité INSERT+SELECT (et éviter une race window avec un
//! éventuel DELETE concurrent), les opérations write utilisent une transaction.
//!
//! Utilise les variantes non-macro `sqlx::query_as::<_, T>("...")` pour
//! éviter la dépendance à une DB live au moment du build.

use chrono::{NaiveDate, NaiveDateTime, Utc};
use serde::Serialize;
use serde_json::json;
use sqlx::mysql::MySqlPool;
use sqlx::{MySql, Transaction};

use crate::entities::{Company, CompanyUpdate, Language, NewAuditLogEntry, NewCompany, OrgType};
use crate::errors::{DbError, map_db_error};
use crate::repositories::MAX_LIST_LIMIT;
use crate::repositories::{api_keys, audit_log};

const FIND_BY_ID_SQL: &str = "SELECT id, name, first_name, last_name, address, address_street, address_building, \
            address_postal_code, address_city, address_country, ide_number, org_type, \
            accounting_language, instance_language, email, phone, website, is_stub, books_locked_through, version, created_at, updated_at \
     FROM companies WHERE id = ?";

const LIST_SQL: &str = "SELECT id, name, first_name, last_name, address, address_street, address_building, \
            address_postal_code, address_city, address_country, ide_number, org_type, \
            accounting_language, instance_language, email, phone, website, is_stub, books_locked_through, version, created_at, updated_at \
     FROM companies ORDER BY id LIMIT ? OFFSET ?";

/// Crée une nouvelle company et retourne l'entité persistée.
///
/// INSERT puis SELECT dans une transaction atomique pour éviter une
/// race window avec un DELETE concurrent.
pub async fn create(pool: &MySqlPool, new: NewCompany) -> Result<Company, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    // Colonne `address` dérivée (#213) : recomposée depuis les champs structurés.
    let addr = &new.address_structured;
    let result = sqlx::query(
        "INSERT INTO companies (name, first_name, last_name, address, address_street, address_building, \
             address_postal_code, address_city, address_country, ide_number, org_type, \
             accounting_language, instance_language) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&new.name)
    .bind(&new.first_name)
    .bind(&new.last_name)
    .bind(addr.combined())
    .bind(&addr.street)
    .bind(&addr.building)
    .bind(&addr.postal_code)
    .bind(&addr.city)
    .bind(&addr.country)
    .bind(&new.ide_number)
    .bind(new.org_type)
    .bind(new.accounting_language)
    .bind(new.instance_language)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?;

    // Valider que l'AUTO_INCREMENT a bien produit un id exploitable
    let last_id = result.last_insert_id();
    if last_id == 0 {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::Invariant(
            "last_insert_id == 0 après INSERT (AUTO_INCREMENT manquant ?)".into(),
        ));
    }
    let id = match i64::try_from(last_id) {
        Ok(v) => v,
        Err(_) => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::Invariant(format!(
                "last_insert_id {last_id} dépasse i64::MAX"
            )));
        }
    };

    let company_opt = sqlx::query_as::<_, Company>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?;

    let company = match company_opt {
        Some(c) => c,
        None => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::Invariant(format!(
                "company {id} introuvable après INSERT"
            )));
        }
    };

    tx.commit().await.map_err(map_db_error)?;
    Ok(company)
}

/// Retrouve une company par son id. Retourne `None` si absente.
pub async fn find_by_id(pool: &MySqlPool, id: i64) -> Result<Option<Company>, DbError> {
    sqlx::query_as::<_, Company>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)
}

/// Liste les companies avec pagination offset/limit.
///
/// `limit` est clampé dans `[0, MAX_LIST_LIMIT]` et `offset` à `>= 0`
/// pour éviter les valeurs invalides et les OOM.
pub async fn list(pool: &MySqlPool, limit: i64, offset: i64) -> Result<Vec<Company>, DbError> {
    let limit = limit.clamp(0, MAX_LIST_LIMIT);
    let offset = offset.max(0);
    sqlx::query_as::<_, Company>(LIST_SQL)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)
}

/// Compare l'état persisté au payload — `true` si aucun champ métier ne diffère
/// (KF-004 : court-circuit no-op pour ne pas bumper version inutilement).
fn is_no_op_change(before: &Company, changes: &CompanyUpdate) -> bool {
    before.name == changes.name
        && before.first_name == changes.first_name
        && before.last_name == changes.last_name
        && before.structured_address() == changes.address_structured
        && before.ide_number == changes.ide_number
        && before.org_type == changes.org_type
        && before.accounting_language == changes.accounting_language
        && before.instance_language == changes.instance_language
        && before.email == changes.email
        // Story 16-3a (#151) — sans ces deux comparaisons, modifier le seul
        // téléphone serait vu comme un no-op : la valeur ne partirait pas en
        // base et `version` ne bougerait pas, en rendant 200.
        && before.phone == changes.phone
        && before.website == changes.website
}

/// Raison sociale d'une **société provisoire** (`is_stub = TRUE`).
///
/// Partagée par les trois sites qui posent le stub — le premier démarrage sur
/// une base sans société (`kesh-api`, `auth::bootstrap`), la branche « aucune
/// société » du choix de la langue (`routes::onboarding`), et la remise à zéro
/// (`kesh_seed::reset_demo`, Story 15-7b2), qui le recrée **en place** ou
/// l'insère sur une base sans société — pour qu'ils ne divergent pas (DRY).
/// Descendue de `auth/bootstrap.rs` dans `kesh-db` par la Story 15-7b2 :
/// `kesh-seed` ne dépend pas de `kesh-api`. Le wizard lève le drapeau quand
/// l'utilisateur renseigne ses vraies coordonnées ([`clear_stub_in_tx`]).
pub const STUB_COMPANY_NAME: &str = "(en cours de configuration)";

/// Adresse combinée d'une société provisoire — cf. [`STUB_COMPANY_NAME`].
pub const STUB_COMPANY_ADDRESS: &str = "-";

/// Insère une **société provisoire** et rend son `id` — Story 15-7b2 (AC 4).
///
/// **Seul site d'insertion du stub** : `org_type = Independant`,
/// `accounting_language = FR`, `instance_language` passée par l'appelant,
/// `is_stub = TRUE` ; toute autre colonne prend son défaut du schéma. Les
/// valeurs que [`reset_to_stub_in_tx`] écrit en place sont celles-ci, de sorte
/// qu'une société recréée et une société remise à zéro soient identiques
/// (`FR` pour la remise à zéro et le premier démarrage, la langue demandée
/// pour la branche « aucune société » du choix de la langue).
///
/// Générique sur l'exécuteur : le premier démarrage l'appelle sur le pool, la
/// remise à zéro et le choix de la langue dans leur transaction. **Ne commite
/// jamais.**
pub async fn insert_stub<'e, E>(executor: E, instance_language: Language) -> Result<i64, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    let result = sqlx::query(
        "INSERT INTO companies \
         (name, address, org_type, accounting_language, instance_language, is_stub) \
         VALUES (?, ?, ?, ?, ?, TRUE)",
    )
    .bind(STUB_COMPANY_NAME)
    .bind(STUB_COMPANY_ADDRESS)
    .bind(OrgType::Independant)
    .bind(Language::Fr)
    .bind(instance_language)
    .execute(executor)
    .await
    .map_err(map_db_error)?;
    i64::try_from(result.last_insert_id())
        .ok()
        .filter(|id| *id > 0)
        .ok_or_else(|| DbError::Invariant("last_insert_id invalide après INSERT companies".into()))
}

/// Ramène une société existante à l'état d'une **société provisoire**,
/// **en place** (même `id`), dans la transaction de l'appelant — Story 15-7b2
/// (AC 4, choix C-15-7-9).
///
/// Écrit les valeurs de [`insert_stub`] (`Language::Fr` pour les deux
/// langues), `is_stub = TRUE`, `version = version + 1`, et ramène **chaque
/// autre colonne** à la valeur qu'elle prend à l'insertion du stub, c'est-à-dire
/// à son défaut du schéma : `NULL` pour les sept colonnes nullables, `''` pour
/// les quatre `address_*` textuelles (`NOT NULL DEFAULT ''`), `'CH'` pour
/// `address_country` et `country`. Cette énumération est **ouverte** par
/// nature : ce qui la ferme est le test 5 de la story
/// (`reset_restores_the_stub_columns_in_place`, `onboarding_audit_e2e.rs`), qui
/// compare **toutes** les colonnes lues dans `information_schema.COLUMNS` à un
/// stub fraîchement inséré — une colonne neuve le fait rougir.
///
/// `DbError::NotFound` si la société n'existe pas. **Ne commite jamais.**
pub async fn reset_to_stub_in_tx(tx: &mut Transaction<'_, MySql>, id: i64) -> Result<(), DbError> {
    let rows = sqlx::query(
        "UPDATE companies SET \
            name = ?, address = ?, org_type = ?, accounting_language = ?, \
            instance_language = ?, is_stub = TRUE, \
            first_name = NULL, last_name = NULL, ide_number = NULL, email = NULL, \
            phone = NULL, website = NULL, books_locked_through = NULL, \
            address_street = '', address_building = '', address_postal_code = '', \
            address_city = '', address_country = 'CH', country = 'CH', \
            version = version + 1 \
         WHERE id = ?",
    )
    .bind(STUB_COMPANY_NAME)
    .bind(STUB_COMPANY_ADDRESS)
    .bind(OrgType::Independant)
    .bind(Language::Fr)
    .bind(Language::Fr)
    .bind(id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();
    if rows != 1 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

/// Une clé d'API **active** et **orpheline** révoquée par
/// [`reattach_orphan_principals_in_tx`] — ce que l'entrée d'audit en dit.
///
/// **Aucune empreinte** : `api_keys` ne porte que `key_hash`, jamais rendu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RevokedApiKey {
    pub id: i64,
    pub name: String,
    pub created_by_user_id: i64,
    pub created_at: NaiveDateTime,
    pub last_used_at: Option<NaiveDateTime>,
}

/// Ce que [`reattach_orphan_principals_in_tx`] a fait.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OrphanPrincipals {
    /// Les utilisateurs orphelins **rattachés** à la cible (vide sans cible).
    pub user_ids: Vec<i64>,
    /// Les clés actives orphelines **révoquées**, cible ou non.
    pub api_keys_revoked: Vec<RevokedApiKey>,
    /// Le nombre de clés orphelines — révoquées ici ou avant — **repointées**
    /// vers la cible (0 sans cible).
    pub api_keys_repointed: u64,
}

impl OrphanPrincipals {
    /// `true` si rien n'a été rattaché, révoqué ni repointé.
    pub fn is_empty(&self) -> bool {
        self.user_ids.is_empty() && self.api_keys_revoked.is_empty() && self.api_keys_repointed == 0
    }
}

/// Prédicat d'**orphelin** : le `company_id` de la ligne `t` ne désigne aucune
/// société. Partagé par les sélections et les `UPDATE` ci-dessous.
const ORPHAN: &str = "NOT EXISTS (SELECT 1 FROM companies c WHERE c.id = t.company_id)";

/// La règle **unique** des principaux orphelins — Story 15-7b2 (AC 4, choix
/// C-15-7-23, C-15-7-45 ; #528).
///
/// Un principal est **orphelin** quand son `company_id` ne désigne aucune
/// société (une version antérieure de la remise à zéro effaçait `companies`
/// sous `FOREIGN_KEY_CHECKS=0`, sans cascade ni repointage — #528). La règle :
///
/// 1. les **utilisateurs** orphelins sont verrouillés (`FOR UPDATE`, par `id`) ;
/// 2. les **clés d'API actives** orphelines sont verrouillées, puis
///    **révoquées** (`api_keys::revoke_in_tx`, `version + 1`) — **quelle que
///    soit la cible** : une clé a pu naître pendant une démonstration, et la
///    repointer active la **réveillerait** sur la société vivante ;
/// 3. si `target = Some(c)` : les utilisateurs orphelins sont rattachés à `c`
///    (`rows_affected` égal au nombre verrouillé, sinon `DbError::Invariant`),
///    et **toutes** les clés orphelines — révoquées à l'instant ou avant — sont
///    repointées vers `c` sans toucher `version` ni `revoked_at` : elles
///    paraissent, révoquées, sur la page des clés de la société ;
///    `target = None` : ni rattachement ni repointage, les clés restent
///    révoquées sur leur `company_id` mort, repointées plus tard par le même
///    appel.
///
/// Ainsi **aucune clé orpheline ne redevient active par aucun chemin**.
///
/// Appelants : `kesh_seed::reset_demo` (`Some`), la branche « aucune société »
/// du choix de la langue (`routes::onboarding`, `Some` de la société créée), et
/// — Story 15-7b3 — la réparation au démarrage et à la restauration.
///
/// **Pré-condition** : l'appelant tient `companies` (`FOR UPDATE`, Pattern 5)
/// et `target`, s'il est donné, existe. Ordre des verrous : `users` puis
/// `api_keys`. **Ne commite jamais.**
pub async fn reattach_orphan_principals_in_tx(
    tx: &mut Transaction<'_, MySql>,
    target: Option<i64>,
) -> Result<OrphanPrincipals, DbError> {
    let user_ids: Vec<i64> = sqlx::query_scalar(&format!(
        "SELECT t.id FROM users t WHERE {ORPHAN} ORDER BY t.id FOR UPDATE"
    ))
    .fetch_all(&mut **tx)
    .await
    .map_err(map_db_error)?;

    #[allow(clippy::type_complexity)]
    let keys: Vec<(
        i64,
        i64,
        i32,
        String,
        i64,
        NaiveDateTime,
        Option<NaiveDateTime>,
    )> = sqlx::query_as(&format!(
        "SELECT t.id, t.company_id, t.version, t.name, t.created_by_user_id, \
                    t.created_at, t.last_used_at \
             FROM api_keys t WHERE t.revoked_at IS NULL AND {ORPHAN} ORDER BY t.id FOR UPDATE"
    ))
    .fetch_all(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let mut api_keys_revoked = Vec::with_capacity(keys.len());
    for (id, company_id, version, name, created_by_user_id, created_at, last_used_at) in keys {
        api_keys::revoke_in_tx(tx, company_id, id, version).await?;
        api_keys_revoked.push(RevokedApiKey {
            id,
            name,
            created_by_user_id,
            created_at,
            last_used_at,
        });
    }

    let Some(target) = target else {
        return Ok(OrphanPrincipals {
            user_ids: Vec::new(),
            api_keys_revoked,
            api_keys_repointed: 0,
        });
    };

    let attached = sqlx::query(&format!(
        "UPDATE users t SET t.company_id = ? WHERE {ORPHAN}"
    ))
    .bind(target)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();
    if attached != user_ids.len() as u64 {
        return Err(DbError::Invariant(format!(
            "rattachement des utilisateurs orphelins : {attached} ligne(s) écrite(s), {} verrouillée(s)",
            user_ids.len()
        )));
    }
    let api_keys_repointed = sqlx::query(&format!(
        "UPDATE api_keys t SET t.company_id = ? WHERE {ORPHAN}"
    ))
    .bind(target)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    Ok(OrphanPrincipals {
        user_ids,
        api_keys_revoked,
        api_keys_repointed,
    })
}

/// Lève le drapeau « société provisoire » (`is_stub`) d'une société, **dans la
/// transaction de l'appelant** (Story 15-7a1, choix C-15-7-21).
///
/// `UPDATE companies SET is_stub = FALSE, version = version + 1 WHERE id = ?
/// AND is_stub = TRUE` : rend `true` si le drapeau était levé (et `version` a
/// pris +1), `false` sinon (société non provisoire ou absente — rien n'est
/// écrit, `version` est inchangée). Bornée à `id = ?` et bumpant `version`,
/// comme la mise à jour des coordonnées de l'onboarding
/// (`update_company_coordinates_in_tx`, son appelant depuis la Story 15-7a2).
/// **Ne commite jamais.**
pub async fn clear_stub_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
) -> Result<bool, DbError> {
    let rows = sqlx::query(
        "UPDATE companies SET is_stub = FALSE, version = version + 1 \
         WHERE id = ? AND is_stub = TRUE",
    )
    .bind(company_id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();
    Ok(rows == 1)
}

/// Met à jour une company avec verrouillage optimiste.
///
/// SELECT before → version check applicatif → court-circuit no-op (KF-004) →
/// UPDATE puis SELECT after, le tout dans une transaction atomique. Retourne
/// `DbError::OptimisticLockConflict` si la version en base ne correspond pas
/// à `version`, ou `DbError::NotFound` si l'entité n'existe pas.
pub async fn update(
    pool: &MySqlPool,
    id: i64,
    version: i32,
    changes: CompanyUpdate,
) -> Result<Company, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;
    let company = update_in_tx(&mut tx, id, version, changes).await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(company)
}

/// Variante transaction-aware de [`update`] — Story 25-1b.
///
/// **Pourquoi** : deux routes (`/companies/current/email` et
/// `/contact-details`) doivent écrire leur trace d'audit dans la transaction de
/// la mutation. L'enveloppe ci-dessus reste en place pour le seed et les neuf
/// tests qui l'appellent — *un refactor qui touche onze appelants pour en
/// servir deux paie un risque au prix d'une commodité*.
///
/// ⚠️ **Le court-circuit no-op de KF-004 vit ICI** : si rien ne change, la
/// fonction retourne le snapshot `before` SANS incrémenter `version`.
/// L'appelant compare les versions et n'écrit alors aucune trace.
///
/// Ne commite jamais.
pub async fn update_in_tx(
    tx: &mut Transaction<'_, MySql>,
    id: i64,
    version: i32,
    changes: CompanyUpdate,
) -> Result<Company, DbError> {
    // Snapshot "before" pour permettre la détection no-op (KF-004).
    let before_opt = sqlx::query_as::<_, Company>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?;

    let before = match before_opt {
        None => {
            return Err(DbError::NotFound);
        }
        Some(c) if c.version != version => {
            return Err(DbError::OptimisticLockConflict);
        }
        Some(c) => c,
    };

    // KF-004 : court-circuit no-op AVANT toute mutation.
    // NOTE concurrence (KF-004): sous REPEATABLE READ + plain SELECT, si une tx
    // parallèle commit entre notre BEGIN et ce check, on retourne notre snapshot
    // stale au lieu d'un 409. Race acceptée v0.1 (cf. spec 7-3 §race-condition).
    // Mitigation future: SELECT FOR UPDATE partout (non v0.1).
    if is_no_op_change(&before, &changes) {
        return Ok(before);
    }

    let addr = &changes.address_structured;

    // Story 16-3a, passe 6 de revue — NE JAMAIS écraser `address` par une
    // chaîne vide.
    //
    // `combined()` rend `""` quand les quatre composants structurés sont vides,
    // ce qui est l'état de **toute société créée avant le 2026-07-05** : la
    // migration `structured_addresses` (#213, v0.5.0) a ajouté ces colonnes en
    // `NOT NULL DEFAULT ''` **sans backfill** — vérifié, aucune migration du
    // dépôt ne fait `UPDATE companies`. Sur ces lignes, l'adresse ne vit que
    // dans la colonne `address` en texte libre.
    //
    // Sans cette garde, toute route qui reconstruit `CompanyUpdate` depuis
    // l'entité en full-replace — `update_company_email` (20-3b1) comme
    // `update_company_contact_details` (16-3a) — écrit `address = ''`, que
    // `chk_companies_address_nonempty` rejette : l'utilisateur reçoit un **500**
    // en voulant simplement renseigner son téléphone.
    //
    // On préserve alors la valeur existante plutôt que d'échouer : elle est
    // garantie non vide par la contrainte elle-même, et la conserver ne dégrade
    // rien — elle reste exactement ce qu'elle était.
    let combined = addr.combined();
    let address_to_write = if combined.trim().is_empty() {
        before.address.clone()
    } else {
        combined
    };

    let rows_affected = sqlx::query(
        "UPDATE companies
         SET name = ?, first_name = ?, last_name = ?, address = ?, address_street = ?, address_building = ?,
             address_postal_code = ?, address_city = ?, address_country = ?,
             ide_number = ?, org_type = ?,
             accounting_language = ?, instance_language = ?,
             email = ?, phone = ?, website = ?,
             version = version + 1
         WHERE id = ? AND version = ?",
    )
    .bind(&changes.name)
    .bind(&changes.first_name)
    .bind(&changes.last_name)
    .bind(&address_to_write)
    .bind(&addr.street)
    .bind(&addr.building)
    .bind(&addr.postal_code)
    .bind(&addr.city)
    .bind(&addr.country)
    .bind(&changes.ide_number)
    .bind(changes.org_type)
    .bind(changes.accounting_language)
    .bind(changes.instance_language)
    .bind(&changes.email)
    .bind(&changes.phone)
    .bind(&changes.website)
    .bind(id)
    .bind(version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows_affected == 0 {
        // Défensif : ne devrait pas arriver puisque la version-check applicative
        // a déjà validé la version. Race théorique entre le SELECT et l'UPDATE.
        return Err(DbError::OptimisticLockConflict);
    }

    let company_opt = sqlx::query_as::<_, Company>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?;

    // Défensif : sous REPEATABLE READ InnoDB dans la même transaction, le SELECT
    // après un UPDATE `rows_affected > 0` retourne toujours la ligne mise à jour.
    // Cette branche est techniquement unreachable mais préservée comme garde-fou.
    let company = match company_opt {
        Some(c) => c,
        None => {
            return Err(DbError::Invariant(format!(
                "company {id} introuvable après UPDATE réussi"
            )));
        }
    };

    Ok(company)
}

// ---------------------------------------------------------------------------
// Story 24-4c (#380) — le verrou de période
// ---------------------------------------------------------------------------

/// Code de refus — la borne proposée n'est pas strictement passée.
///
/// ⛔ **C'est un CODE, pas un message.** `DbError::InvalidInput` est confronté
/// par `AppError` à une **liste blanche stricte** ; tout code inconnu retombe
/// sur « Entrée invalide », sans date ni raison. Le code doit donc être ajouté
/// au dispatch de `kesh-api/src/errors.rs` **et** aux quatre catalogues.
pub const BOOKS_LOCK_BOUND_NOT_PAST: &str = "booksLockBoundNotPast";

/// Code de refus — le déverrouillage exige un motif non blanc.
pub const BOOKS_UNLOCK_MOTIF_REQUIRED: &str = "booksUnlockMotifRequired";

/// Pose ou **avance** la borne du verrou de période.
///
/// Autorisé aux rôles **Admin et Comptable** : verrouiller est un geste
/// d'hygiène, qu'on doit pouvoir faire souvent et sans cérémonie.
///
/// # Deux gardes de VALEUR, et elles ne sont pas décoratives
///
/// ⛔ **`through` doit être STRICTEMENT antérieure à aujourd'hui.** Une borne
/// posée à la date du jour refuserait toute contre-passation faite le même jour
/// — celle-ci étant datée du jour et le seuil de la garde étant inclusif —,
/// c'est-à-dire rendrait les livres incorrigibles le jour même. ⚠️ « Aujourd'hui »
/// est `Utc::now().date_naive()`, la **même horloge** que la contre-passation :
/// mélanger les deux réintroduirait l'écart d'un jour sous une autre forme.
///
/// ⛔ **`through` doit être STRICTEMENT postérieure à la borne courante non
/// nulle.** Sans cette garde, la séparation par rôle serait contournable *par le
/// verbe* : un Comptable appellerait ce point d'entrée avec une date antérieure,
/// la borne reculerait sans motif ni rôle Admin, et le journal d'audit écrirait
/// `books.locked` — un retrait **maquillé en pose**. Avancer veut dire avancer.
///
/// ⚠️ Le « non nulle » compte : à la première pose la borne vaut `NULL`, et
/// c'est le seul cas où cette garde doit se taire.
pub async fn lock_books(
    pool: &MySqlPool,
    user_id: i64,
    company_id: i64,
    through: NaiveDate,
) -> Result<Company, DbError> {
    // ⚠️ `InvalidInput` transporte un **CODE**, jamais une phrase : `AppError`
    // le confronte à une liste blanche stricte (`errors.rs`, whitelist B13) et
    // retombe sur « Entrée invalide » pour tout code inconnu. Une phrase
    // française y arriverait donc **sans date, sans raison et sans quoi faire**
    // — sur le geste même que cette garde existe pour rattraper.
    if through >= Utc::now().date_naive() {
        return Err(DbError::InvalidInput(BOOKS_LOCK_BOUND_NOT_PAST.into()));
    }

    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let before: Option<Option<NaiveDate>> =
        sqlx::query_scalar("SELECT books_locked_through FROM companies WHERE id = ? FOR UPDATE")
            .bind(company_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_db_error)?;

    let before = match before {
        None => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::NotFound);
        }
        Some(v) => v,
    };

    if let Some(current) = before
        && through <= current
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::IllegalStateTransition(format!(
            "les livres sont déjà verrouillés jusqu'au {current} — avancer la borne exige une date postérieure ;              la reculer relève du déverrouillage (Admin, motif obligatoire)"
        )));
    }

    sqlx::query("UPDATE companies SET books_locked_through = ? WHERE id = ?")
        .bind(through)
        .bind(company_id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;

    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            user_id,
            "books.locked".to_string(),
            "company".to_string(),
            company_id,
            Some(json!({ "before": before, "after": through })),
        ),
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;
    find_by_id(pool, company_id).await?.ok_or(DbError::NotFound)
}

/// **Recule ou retire** la borne du verrou de période.
///
/// ⛔ Réservé à **Admin**, et le **motif est obligatoire** — c'est l'asymétrie
/// qui fait toute la mesure. Verrouiller est un geste d'hygiène ; déverrouiller
/// **défait une garantie**, et doit donc coûter, se justifier et se retrouver
/// dans le journal d'audit, exactement comme la réouverture d'un exercice clos.
///
/// `through = None` retire le verrou entièrement.
///
/// ⚠️ L'action `books.unlocked` a **un seul producteur** : cette fonction. La
/// restauration d'une sauvegarde, qui peut elle aussi faire reculer la borne,
/// écrit `books.restored` — confondre les deux rendrait le filtre d'audit
/// inutilisable pour le réviseur qui cherche **qui** a déverrouillé.
pub async fn unlock_books(
    pool: &MySqlPool,
    user_id: i64,
    company_id: i64,
    through: Option<NaiveDate>,
    motif: String,
) -> Result<Company, DbError> {
    if motif.trim().is_empty() {
        return Err(DbError::InvalidInput(BOOKS_UNLOCK_MOTIF_REQUIRED.into()));
    }

    // ⛔ La MÊME garde de date que `lock_books`, et son absence ici était un
    // trou béant : ce point d'entrée POSE aussi une borne (il la recule), donc
    // un administrateur pouvait y placer une date future — d'un clic
    // malencontreux dans le formulaire — et refuser du même coup TOUTE création
    // d'écriture datée d'aujourd'hui, contre-passation comprise.
    //
    // ⚠️ C'est-à-dire casser l'invariant I2, « le verrou n'enferme pas », que
    // toute la vague 24-4a → 24-4c existe pour tenir. Le déni est récupérable
    // (un second appel avec une date passée), mais total pendant la fenêtre.
    if let Some(d) = through
        && d >= Utc::now().date_naive()
    {
        return Err(DbError::InvalidInput(BOOKS_LOCK_BOUND_NOT_PAST.into()));
    }

    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let before: Option<Option<NaiveDate>> =
        sqlx::query_scalar("SELECT books_locked_through FROM companies WHERE id = ? FOR UPDATE")
            .bind(company_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_db_error)?;

    let before = match before {
        None => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::NotFound);
        }
        Some(v) => v,
    };

    // ⛔ Ce point d'entrée RECULE ou RETIRE — il n'avance pas. Sans cette garde,
    // un Admin pouvait y poster une date POSTÉRIEURE à la borne courante :
    // le verrou avançait, et le journal d'audit écrivait `books.unlocked`.
    //
    // ⚠️ Le doc-comment de cette fonction affirme deux écrans plus haut que
    // `books.unlocked` a **un seul producteur**, le déverrouillage délibéré. Le
    // laisser produire une AVANCÉE ferait mentir le verbe, et le réviseur qui
    // filtre « qui a déverrouillé » lirait une pose. *Une garde de valeur
    // manquante ne crée pas ici de faille de droits — elle corrompt la trace.*
    if let (Some(avant), Some(vise)) = (before, through)
        && vise > avant
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::IllegalStateTransition(format!(
            "les livres sont verrouillés jusqu'au {avant} — avancer la borne relève de la pose \
             (`lock_books`), pas du déverrouillage"
        )));
    }

    // ⛔ Et le sous-cas symétrique : **poser un PREMIER verrou par la levée**.
    // Sans borne courante, la garde ci-dessus se tait — comme celle de
    // `lock_books`, où ce silence est voulu. Mais ici l'effet diffère : la
    // société n'a jamais été verrouillée, et l'opération réussirait en écrivant
    // `books.unlocked` sur un verrou qui n'a jamais existé.
    //
    // ⚠️ Aucune faille de droits — la route est Admin seule, et l'écran ne rend
    // le formulaire que si une borne existe. Ce qui se corrompt est la **trace** :
    // le réviseur qui filtre « qui a déverrouillé » lirait une pose, sur une
    // société qui n'a jamais rien verrouillé. *C'est la même exigence que R4,
    // étendue au seul sous-cas qu'elle avait laissé ouvert.*
    // ⚠️ La condition est `before.is_none()` SEULE, et le conjoint qu'une
    // première rédaction y avait ajouté (`&& through.is_some()`) la rétrécissait
    // sans raison : il laissait passer `through = None` — retirer une borne
    // inexistante —, une opération qui ne change rien et écrit pourtant
    // `books.unlocked`, verbe qui affirme qu'une borne a été retirée.
    //
    // ⛔ Sans borne courante, il n'y a **rien à reculer ni à retirer**, quelle
    // que soit la cible. C'est le troisième et dernier angle de l'exigence
    // « `books.unlocked` a un seul producteur » : les passes 2, 3 et 4 en ont
    // fermé un chacune. *Une garde partielle est une garde qui rouvre.*
    if before.is_none() {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::IllegalStateTransition(
            "les livres ne sont pas verrouillés — il n'y a rien à reculer ni à retirer ; \
             poser une borne relève de `lock_books`"
                .into(),
        ));
    }

    sqlx::query("UPDATE companies SET books_locked_through = ? WHERE id = ?")
        .bind(through)
        .bind(company_id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;

    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            user_id,
            "books.unlocked".to_string(),
            "company".to_string(),
            company_id,
            Some(json!({ "before": before, "after": through, "motif": motif })),
        ),
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;
    find_by_id(pool, company_id).await?.ok_or(DbError::NotFound)
}
