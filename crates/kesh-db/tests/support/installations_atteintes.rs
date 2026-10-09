//! Aides de montage **réservées aux tests** — Stories 15-7b2 et 15-7b3
//! (revue de code P1 de la 15-7b3, B-4 = E3 = A6 ; choix C-15-7b3-4).
//!
//! Elles contournent les garde-fous du moteur — `FOREIGN_KEY_CHECKS = 0`,
//! `CREATE TRIGGER` composé par `format!` — et n'ont donc **rien à faire dans le
//! binaire de production** : elles vivaient d'abord dans
//! `kesh_db::test_fixtures`, compilé en permanence. Ce fichier n'appartient à
//! aucune crate ; il est **inclus par `#[path]`** là où on l'emploie :
//!
//! - `crates/kesh-db/tests/companies_repository.rs` ;
//! - `crates/kesh-api/tests/admin_full_import_e2e.rs`,
//!   `crates/kesh-api/tests/onboarding_audit_e2e.rs` ;
//! - `crates/kesh-api/src/auth/bootstrap.rs`, sous `#[cfg(test)]` : le module
//!   n'existe que dans le binaire de tests unitaires.
//!
//! ⛔ Ne pas le réexposer par une route de `test_endpoints` : rien n'y est
//! échappé (les arguments de [`poser_declencheur_en_echec`] sont des littéraux
//! de test).
#![allow(dead_code)]

use sqlx::MySqlPool;

/// **État orphelin de #528** (Story 15-7b3, montage commun des tests) : porte
/// `users.company_id` et **toutes** les lignes d'`api_keys` — y compris les
/// clés déjà révoquées — à `dead_company_id`, un id qu'aucune société ne porte.
///
/// Écrit sur une connexion **détachée du pool** (`detach()` : elle n'y revient
/// jamais), `FOREIGN_KEY_CHECKS = 0` de session, fermée en fin de montage —
/// l'état qu'a laissé la remise à zéro v0.12.x, qui effaçait `companies` sans
/// rattacher personne.
pub async fn rendre_principaux_orphelins(pool: &MySqlPool, dead_company_id: i64) {
    use sqlx::Connection;
    let mut conn = pool.acquire().await.expect("connexion de montage").detach();
    sqlx::query("SET FOREIGN_KEY_CHECKS = 0")
        .execute(&mut conn)
        .await
        .expect("montage de l'état orphelin");
    for sql in [
        "UPDATE users SET company_id = ?",
        "UPDATE api_keys SET company_id = ?",
    ] {
        sqlx::query(sql)
            .bind(dead_company_id)
            .execute(&mut conn)
            .await
            .expect("montage de l'état orphelin");
    }
    conn.close()
        .await
        .expect("fermeture de la connexion de montage");
}

/// **Déclencheur qui échoue** (Stories 15-7b2 et 15-7b3) : crée `nom`,
/// `quand` (ex. `"BEFORE INSERT ON audit_log"`), dont le corps lève
/// `SIGNAL SQLSTATE '45000'` — toujours, ou si `condition` (ex.
/// `"OLD.id = 7"`) est vraie.
///
/// Pré-requis **asserté**, jamais contourné : journal binaire inactif
/// (`@@log_bin = 0`) — sinon `CREATE TRIGGER` exige `SUPER` ou
/// `log_bin_trust_function_creators`. Pas de sortie anticipée (un test muet),
/// pas de `SET GLOBAL`. Le message d'échec nomme le pré-requis : un échec de
/// montage ne se lit pas comme un échec du test.
pub async fn poser_declencheur_en_echec(
    pool: &MySqlPool,
    nom: &str,
    quand: &str,
    condition: Option<&str>,
) {
    let log_bin: i64 = sqlx::query_scalar("SELECT CAST(@@log_bin AS SIGNED)")
        .fetch_one(pool)
        .await
        .expect("lecture de @@log_bin");
    assert_eq!(
        log_bin, 0,
        "pré-requis du montage : journal binaire inactif (sinon CREATE TRIGGER exige \
         SUPER ou log_bin_trust_function_creators)"
    );
    let signal = format!("SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = '{nom}'");
    let corps = match condition {
        Some(c) => format!("IF {c} THEN {signal}; END IF"),
        None => signal,
    };
    sqlx::raw_sql(&format!(
        "CREATE TRIGGER {nom} {quand} FOR EACH ROW {corps}"
    ))
    .execute(pool)
    .await
    .expect("création du déclencheur");
}
