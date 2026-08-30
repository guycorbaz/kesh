//! Repository CRUD pour `Company`.
//!
//! MySQL/MariaDB n'a pas de clause `RETURNING` (contrairement à Postgres),
//! d'où le pattern `create` en deux étapes : INSERT puis SELECT via `find_by_id`.
//! Pour garantir l'atomicité INSERT+SELECT (et éviter une race window avec un
//! éventuel DELETE concurrent), les opérations write utilisent une transaction.
//!
//! Utilise les variantes non-macro `sqlx::query_as::<_, T>("...")` pour
//! éviter la dépendance à une DB live au moment du build.

use chrono::{NaiveDate, Utc};
use serde_json::json;
use sqlx::mysql::MySqlPool;

use crate::entities::{Company, CompanyUpdate, NewAuditLogEntry, NewCompany};
use crate::errors::{DbError, map_db_error};
use crate::repositories::MAX_LIST_LIMIT;
use crate::repositories::audit_log;

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

    // Snapshot "before" pour permettre la détection no-op (KF-004).
    let before_opt = sqlx::query_as::<_, Company>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?;

    let before = match before_opt {
        None => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::NotFound);
        }
        Some(c) if c.version != version => {
            tx.rollback().await.map_err(map_db_error)?;
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
        tx.rollback().await.map_err(map_db_error)?;
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
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows_affected == 0 {
        // Défensif : ne devrait pas arriver puisque la version-check applicative
        // a déjà validé la version. Race théorique entre le SELECT et l'UPDATE.
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::OptimisticLockConflict);
    }

    let company_opt = sqlx::query_as::<_, Company>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?;

    // Défensif : sous REPEATABLE READ InnoDB dans la même transaction, le SELECT
    // après un UPDATE `rows_affected > 0` retourne toujours la ligne mise à jour.
    // Cette branche est techniquement unreachable mais préservée comme garde-fou.
    let company = match company_opt {
        Some(c) => c,
        None => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::Invariant(format!(
                "company {id} introuvable après UPDATE réussi"
            )));
        }
    };

    tx.commit().await.map_err(map_db_error)?;
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
