//! Advisory lock per bank account (Story 8-4 §mutex-account).
//!
//! Helper [`with_account_lock`] acquiert un advisory lock MariaDB
//! `GET_LOCK('reconcile:{base}:{company_id}:{bank_account_id}', timeout_secs)`
//! pour serializer les flows accept/reject sur le même compte bancaire.
//!
//! ⚠️ **Le nom de la base fait partie de la clé, et il n'y est pas par
//! esthétique — c'est la cause racine de la KF-038 (#228).**
//!
//! `GET_LOCK` est **global au serveur MariaDB**, jamais borné à la base
//! courante : un verrou pris depuis une base est vu, et bloque, depuis toute
//! autre base du même serveur. Vérifié au sol le 2026-08-27, deux bases
//! distinctes, une session tenant le verrou — la seconde ne peut pas
//! l'acquérir et attend le timeout entier.
//!
//! En production la conséquence est nulle : une seule base, donc
//! `{company_id}:{bank_account_id}` suffisait déjà à identifier un compte. Mais
//! **chaque test `#[sqlx::test]` crée une base éphémère neuve, où les
//! `AUTO_INCREMENT` repartent à 1** : tous les tests de réconciliation se
//! disputaient donc le même `reconcile:1:1`, sur le même serveur, en parallèle.
//! D'où des `409 AccountLocked` sporadiques, d'autant plus fréquents que le
//! plafond de threads est haut et que le verrou est tenu longtemps.
//!
//! Le préfixe rend la clé unique par base **sans changer quoi que ce soit en
//! production**, où `DATABASE()` est constant.
//!
//! **Choix vs alternatives** :
//! - ❌ `SELECT ... FOR UPDATE` row-level : sémantique floue, plus
//!   long que nécessaire.
//! - ❌ Tokio Mutex en mémoire : pas multi-instance.
//! - ✅ Advisory lock MariaDB : portable multi-instance, nommé, libéré à la
//!   fin de session si l'application s'arrête.
//!
//! ⚠️ **Le verrou est de session, et le pool garde la session** : il survit au
//! `ROLLBACK` d'une transaction abandonnée. Depuis la 0.17.4 (#842), c'est le
//! crochet du pool (`kesh_db::pool::avec_remise_a_zero_de_session`, posé par
//! `main.rs` et `create_pool`) qui le rend **au retour de la connexion au
//! pool** — requête interrompue (future abandonnée avant `RELEASE_LOCK`) ou
//! `RELEASE_LOCK` raté. Avant, la connexion revenait au pool en le tenant, et le
//! compte restait bloqué jusqu'à la fermeture de cette connexion.
//!
//! **HP3-1 + HP4-1 + HP5-1 — caller pattern** : sur
//! `Err(LockReleaseFailed)`, le handler **DOIT** :
//!
//! ```ignore
//! match with_account_lock(&mut tx_outer, ...).await {
//!     Ok(result) => result,
//!     Err(ReconciliationError::LockReleaseFailed { bank_account_id, .. }) => {
//!         drop(tx_outer); // Drop impl rollback + retour pool
//!         return Err(AppError::ReconciliationLockReleaseFailed { bank_account_id });
//!     }
//!     Err(e) => return Err(map_reconciliation_error(e)),
//! }
//! ```
//!
//! **Anti-patterns** : `pool.close().await` (ferme TOUT le pool, outage),
//! `connection.detach()` (méthode sur PoolConnection inaccessible via
//! `&mut Transaction`), `tx_outer.rollback().await` (redondant avec Drop).

use crate::errors::ReconciliationError;

/// Acquiert un advisory lock MariaDB sur `(base, company_id, bank_account_id)`,
/// exécute la closure `f` dans la transaction sous le lock, puis
/// libère explicitement le lock.
///
/// Retourne :
/// - `Ok(T)` si la closure réussit ET le lock est libéré sans erreur.
/// - `Err(AccountLocked)` si l'acquisition timeout.
/// - `Err(LockReleaseFailed)` si la closure réussit mais
///   `RELEASE_LOCK` échoue (cas pathologique connexion poisoned —
///   le caller doit drop tx_outer pour rollback + retour pool ; le
///   lock advisory est rendu au retour de la connexion au pool, par le
///   crochet `kesh_db::pool::avec_remise_a_zero_de_session`, #842).
/// - `Err(...)` propagé tel quel depuis la closure.
pub async fn with_account_lock<F, T>(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    bank_account_id: i64,
    timeout_secs: u32,
    f: F,
) -> Result<T, ReconciliationError>
where
    F: AsyncFnOnce(&mut sqlx::Transaction<'_, sqlx::MySql>) -> Result<T, ReconciliationError>,
{
    let lock_name = account_lock_name(tx, company_id, bank_account_id).await?;
    acquire_account_lock(tx, &lock_name, bank_account_id, timeout_secs).await?;

    let result = f(tx).await;

    // HP3-1 + HP6-1 + C4 Pass 1 — RELEASE_LOCK explicite avant retour.
    // M3 Pass 1 : utiliser query_scalar pour récupérer la valeur de
    // retour (Some(1) = released, Some(0) = was not held, None =
    // lock didn't exist). C4 Pass 1 : préserver l'erreur business
    // (`result`) si elle existe, prioritaire sur une erreur RELEASE.
    let release_outcome: Result<Option<i32>, sqlx::Error> =
        sqlx::query_scalar("SELECT RELEASE_LOCK(?)")
            .bind(&lock_name)
            .fetch_one(&mut **tx)
            .await;

    match (result, release_outcome) {
        // Cas nominal : business OK + release OK Some(1).
        (Ok(value), Ok(Some(1))) => Ok(value),
        // Business OK mais release problématique (Some(0)/None ou Err) :
        // remonter LockReleaseFailed en log + Err. La valeur business
        // est sacrifiée — le caller voit l'erreur et drop tx_outer.
        (Ok(_value), release) => match release {
            Ok(release_status) => {
                tracing::error!(
                    ?release_status,
                    ?bank_account_id,
                    lock_name = %lock_name,
                    "RELEASE_LOCK returned non-1 after successful business op; \
                     lock is released when the connection returns to a pool fitted with \
                     the session reset hook (#842)"
                );
                Err(ReconciliationError::LockReleaseFailed {
                    bank_account_id,
                    source: sqlx::Error::Protocol(format!(
                        "RELEASE_LOCK returned {release_status:?}"
                    )),
                })
            }
            Err(e) => {
                tracing::error!(
                    ?e,
                    ?bank_account_id,
                    lock_name = %lock_name,
                    "RELEASE_LOCK failed after successful business op; \
                     lock is released when the connection returns to a pool fitted with \
                     the session reset hook (#842)"
                );
                Err(ReconciliationError::LockReleaseFailed {
                    bank_account_id,
                    source: e,
                })
            }
        },
        // Erreur business prime ; warning si release a aussi échoué.
        (Err(business_err), release) => {
            if let Err(release_err) = release {
                tracing::warn!(
                    ?release_err,
                    ?bank_account_id,
                    lock_name = %lock_name,
                    "RELEASE_LOCK also failed during error path (business error preserved)"
                );
            } else if let Ok(release_status) = release
                && release_status != Some(1)
            {
                tracing::warn!(
                    ?release_status,
                    ?bank_account_id,
                    lock_name = %lock_name,
                    "RELEASE_LOCK returned non-1 during error path \
                     (business error preserved)"
                );
            }
            Err(business_err)
        }
    }
}

/// Nom du verrou nommé d'un compte : `reconcile:{base}:{société}:{compte}`.
async fn account_lock_name(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    bank_account_id: i64,
) -> Result<String, ReconciliationError> {
    // ⚠️ `DATABASE()` et non un identifiant applicatif : c'est la seule chose
    // qui distingue deux bases éphémères sur le même serveur (cf. l'en-tête du
    // module, KF-038 #228). Le `COALESCE` couvre le cas — théorique ici, la
    // transaction venant d'un pool configuré sur une base — d'une session sans
    // base courante : on retombe alors sur l'ancienne clé plutôt que de
    // fabriquer un nom contenant `NULL`.
    let db_name: String = sqlx::query_scalar("SELECT COALESCE(DATABASE(), '')")
        .fetch_one(&mut **tx)
        .await
        .map_err(ReconciliationError::Database)?;
    Ok(format!(
        "reconcile:{db_name}:{company_id}:{bank_account_id}"
    ))
}

/// `GET_LOCK(lock_name, timeout_secs)` : `Ok(())` si acquis,
/// [`ReconciliationError::AccountLocked`] au délai dépassé.
async fn acquire_account_lock(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    lock_name: &str,
    bank_account_id: i64,
    timeout_secs: u32,
) -> Result<(), ReconciliationError> {
    // H5 Pass 1 — i32 binding explicite (MariaDB attend signed int) +
    // distinguer NULL vs 0 vs valeur inattendue.
    let acquired: Option<i32> = sqlx::query_scalar("SELECT GET_LOCK(?, ?)")
        .bind(lock_name)
        .bind(timeout_secs as i32)
        .fetch_one(&mut **tx)
        .await
        .map_err(ReconciliationError::Database)?;
    match acquired {
        Some(1) => Ok(()),
        Some(0) => Err(ReconciliationError::AccountLocked {
            bank_account_id,
            timeout_secs,
        }),
        Some(other) => {
            tracing::error!(
                ?other,
                ?bank_account_id,
                "GET_LOCK returned unexpected non-0/1 value"
            );
            Err(ReconciliationError::Database(sqlx::Error::Protocol(
                format!("GET_LOCK unexpected return: {other}"),
            )))
        }
        None => {
            tracing::error!(
                ?bank_account_id,
                "GET_LOCK returned NULL — DB internal error"
            );
            Err(ReconciliationError::Database(sqlx::Error::Protocol(
                "GET_LOCK returned NULL".to_string(),
            )))
        }
    }
}

/// Story 38-11b (AC-5) — les verrous nommés de **plusieurs** comptes d'une
/// société, pris par **identifiant croissant** (ids triés et dédoublonnés),
/// `GET_LOCK` successifs sur la même connexion, puis `f`, puis libérés en ordre
/// inverse. Sert l'import PayPal, qui écrit sur un compte bancaire par devise,
/// et l'acceptation de 38-11c2 (partagé avec les Epics 39 et 40).
///
/// L'ordre croissant exclut l'interblocage entre deux appelants qui visent des
/// ensembles qui se recoupent : chacun attend au plus le verrou suivant de sa
/// liste, jamais un verrou plus petit que celui qu'il tient.
///
/// Retourne :
/// - [`ReconciliationError::AccountLocked`] au **premier** délai dépassé —
///   les verrous déjà pris sont rendus (ordre inverse) avant le retour, `f`
///   n'est pas appelée ;
/// - [`ReconciliationError::LockReleaseFailed`] si `f` réussit et qu'un
///   `RELEASE_LOCK` échoue (le premier compte en échec est nommé ; le verrou
///   est rendu au retour de la connexion au pool, #842) ;
/// - l'erreur de `f` telle quelle (prioritaire sur un échec de libération).
pub async fn with_account_locks<F, T>(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    bank_account_ids: &[i64],
    timeout_secs: u32,
    f: F,
) -> Result<T, ReconciliationError>
where
    F: AsyncFnOnce(&mut sqlx::Transaction<'_, sqlx::MySql>) -> Result<T, ReconciliationError>,
{
    let mut ids = bank_account_ids.to_vec();
    ids.sort_unstable();
    ids.dedup();

    let mut held: Vec<(i64, String)> = Vec::with_capacity(ids.len());
    for id in ids {
        let name = match account_lock_name(tx, company_id, id).await {
            Ok(name) => name,
            Err(e) => {
                release_all(tx, &held).await;
                return Err(e);
            }
        };
        if let Err(e) = acquire_account_lock(tx, &name, id, timeout_secs).await {
            release_all(tx, &held).await;
            return Err(e);
        }
        held.push((id, name));
    }

    let result = f(tx).await;
    let failed = release_all(tx, &held).await;
    match (result, failed) {
        (Ok(value), None) => Ok(value),
        (Ok(_), Some((bank_account_id, source))) => Err(ReconciliationError::LockReleaseFailed {
            bank_account_id,
            source,
        }),
        (Err(e), _) => Err(e),
    }
}

/// Rend les verrous `held` en ordre inverse ; rend le premier échec (compte et
/// cause), après avoir tenté de rendre tous les autres.
async fn release_all(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    held: &[(i64, String)],
) -> Option<(i64, sqlx::Error)> {
    let mut first_failure = None;
    for (id, name) in held.iter().rev() {
        let outcome: Result<Option<i32>, sqlx::Error> =
            sqlx::query_scalar("SELECT RELEASE_LOCK(?)")
                .bind(name)
                .fetch_one(&mut **tx)
                .await;
        let failure = match outcome {
            Ok(Some(1)) => None,
            Ok(other) => Some(sqlx::Error::Protocol(format!(
                "RELEASE_LOCK returned {other:?}"
            ))),
            Err(e) => Some(e),
        };
        if let Some(e) = failure {
            tracing::error!(error = ?e, bank_account_id = id, lock_name = %name, "RELEASE_LOCK en échec (verrous de plusieurs comptes)");
            if first_failure.is_none() {
                first_failure = Some((*id, e));
            }
        }
    }
    first_failure
}
