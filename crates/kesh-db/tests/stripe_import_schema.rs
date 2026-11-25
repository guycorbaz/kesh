//! Story 40-7b (AC-1) — les contraintes de la migration
//! `20261120000003_stripe_import`, éprouvées sur le schéma de test (le squash,
//! tenu égal au schéma des migrations par `test_schema_guard.rs`) :
//!
//! - `payout_bank_account_id` ne se renseigne que sur un compte `stripe`
//!   (`chk_bank_accounts_payout_stripe`) : refusé sans prestataire (la CHECK
//!   ne doit pas valoir NULL, donc être satisfaite), sur un compte PayPal et
//!   sur un `Stripe` écrit en capitale (comparaison à l'octet) ;
//! - la clé étrangère `RESTRICT` : un compte bancaire désigné comme destination
//!   d'un versement ne se supprime pas.
//!
//! Fixtures fictives seulement.

use kesh_db::test_fixtures::seed_accounting_company;
use sqlx::MySqlPool;

/// `INSERT` d'un compte bancaire de prestataire (sans IBAN) ; rend l'id ou
/// l'erreur MariaDB, en texte.
async fn compte(
    pool: &MySqlPool,
    company_id: i64,
    provider: Option<&str>,
    payout: Option<i64>,
) -> Result<i64, String> {
    sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, provider, provider_account, \
         payout_bank_account_id) VALUES (?, 'Compte', ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(
        provider
            .is_none()
            .then_some("CH9300762011623852957".to_string()),
    )
    .bind(provider)
    .bind(provider.map(|_| "Libellé"))
    .bind(payout)
    .execute(pool)
    .await
    .map(|r| r.last_insert_id() as i64)
    .map_err(|e| e.to_string())
}

fn refuse(r: Result<i64, String>, contrainte: &str) {
    match r {
        Err(e) => assert!(e.contains(contrainte), "attendu {contrainte}, reçu : {e}"),
        Ok(_) => panic!("attendu le refus de {contrainte}"),
    }
}

#[sqlx::test(migrations = "./test-schema")]
async fn la_destination_de_versement_ne_va_qu_a_un_compte_stripe(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let c = s.company_id;
    let wise = compte(&pool, c, Some("wise"), None)
        .await
        .expect("compte Wise");

    // Sans prestataire : la CHECK ne vaut pas NULL, elle refuse.
    refuse(
        compte(&pool, c, None, Some(wise)).await,
        "chk_bank_accounts_payout_stripe",
    );
    // Compte PayPal.
    refuse(
        compte(&pool, c, Some("paypal"), Some(wise)).await,
        "chk_bank_accounts_payout_stripe",
    );
    // À l'octet : `Stripe` est refusé par le domaine fermé de `provider` déjà ;
    // `chk_bank_accounts_provider` ou la nôtre, l'une des deux refuse.
    assert!(
        compte(&pool, c, Some("Stripe"), Some(wise)).await.is_err(),
        "`Stripe` en capitale refusé"
    );
    // Compte Stripe : admis, avec et sans destination.
    compte(&pool, c, Some("stripe"), Some(wise))
        .await
        .expect("compte Stripe avec destination");
    compte(&pool, c, Some("stripe"), None)
        .await
        .expect("compte Stripe sans destination");
}

#[sqlx::test(migrations = "./test-schema")]
async fn un_compte_de_versement_reference_ne_se_supprime_pas(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let c = s.company_id;
    let wise = compte(&pool, c, Some("wise"), None).await.unwrap();
    compte(&pool, c, Some("stripe"), Some(wise)).await.unwrap();
    let err = sqlx::query("DELETE FROM bank_accounts WHERE id = ?")
        .bind(wise)
        .execute(&pool)
        .await
        .expect_err("suppression refusée par la clé étrangère");
    assert!(
        err.to_string()
            .contains("fk_bank_accounts_payout_bank_account"),
        "reçu : {err}"
    );
}
