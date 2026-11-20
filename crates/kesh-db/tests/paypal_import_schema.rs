//! Story 38-11b (AC-1) — les contraintes de la migration
//! `20261120000001_paypal_import`, éprouvées sur le schéma de test (le squash,
//! tenu égal au schéma des migrations par `test_schema_guard.rs`) :
//!
//! - un compte `bank` sans IBAN est refusé sans prestataire, admis avec
//!   (`chk_bank_accounts_bank_has_iban`, reposée élargie) ;
//! - le domaine du prestataire est fermé à l'octet (`PayPal` refusé), `wise`
//!   et `stripe` admis par le schéma (réservés) ; un compte d'un prestataire
//!   n'est jamais principal ni une carte, et porte un libellé non vide ; un
//!   compte ordinaire ne porte ni libellé ni compte de frais
//!   (`chk_bank_accounts_provider`) ;
//! - les filtres d'une règle : vide refusé, sens fermé à l'octet
//!   (`chk_reconciliation_rules_filters`).
//!
//! Fixtures fictives seulement.

use kesh_db::test_fixtures::seed_accounting_company;
use sqlx::MySqlPool;

/// `INSERT` d'un compte bancaire aux colonnes données ; rend l'erreur MariaDB
/// éventuelle, en texte.
#[allow(clippy::too_many_arguments)] // une colonne par argument : la table qu'on éprouve
async fn compte(
    pool: &MySqlPool,
    company_id: i64,
    iban: Option<&str>,
    kind: &str,
    primary: bool,
    provider: Option<&str>,
    provider_account: Option<&str>,
    fee: Option<i64>,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, account_kind, is_primary, \
         provider, provider_account, fee_account_id) VALUES (?, 'Banque', ?, ?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(iban)
    .bind(kind)
    .bind(primary)
    .bind(provider)
    .bind(provider_account)
    .bind(fee)
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(|e| e.to_string())
}

fn refuse(r: Result<(), String>, contrainte: &str) {
    match r {
        Err(e) => assert!(e.contains(contrainte), "attendu {contrainte}, reçu : {e}"),
        Ok(()) => panic!("attendu le refus de {contrainte}"),
    }
}

#[sqlx::test(migrations = "./test-schema")]
async fn les_contraintes_du_compte_d_un_prestataire(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let c = s.company_id;
    let iban = Some("CH9300762011623852957");
    let frais = s.accounts["4000"];

    refuse(
        compte(&pool, c, None, "bank", false, None, None, None).await,
        "chk_bank_accounts_bank_has_iban",
    );
    compte(
        &pool,
        c,
        None,
        "bank",
        false,
        Some("paypal"),
        Some("PayPal www"),
        Some(frais),
    )
    .await
    .expect("compte PayPal sans IBAN");
    compte(
        &pool,
        c,
        iban,
        "bank",
        false,
        Some("paypal"),
        Some("PayPal www"),
        None,
    )
    .await
    .expect("compte PayPal avec IBAN");
    compte(
        &pool,
        c,
        None,
        "bank",
        false,
        Some("wise"),
        Some("Wise"),
        None,
    )
    .await
    .expect("wise réservé au schéma");
    compte(
        &pool,
        c,
        None,
        "bank",
        false,
        Some("stripe"),
        Some("Stripe"),
        None,
    )
    .await
    .expect("stripe réservé au schéma");
    compte(&pool, c, iban, "bank", false, None, None, None)
        .await
        .expect("compte ordinaire inchangé");

    for (provider, account, kind, primary, iban_) in [
        (Some("PayPal"), Some("PayPal www"), "bank", false, None),
        (Some("paypal "), Some("PayPal www"), "bank", false, None),
        (Some("revolut"), Some("Revolut"), "bank", false, None),
        (Some("paypal"), Some("PayPal www"), "bank", true, iban),
        (Some("paypal"), Some("  "), "bank", false, None),
        (Some("paypal"), None, "bank", false, None),
        (
            Some("paypal"),
            Some("PayPal www"),
            "credit_card",
            false,
            None,
        ),
        (None, Some("PayPal www"), "bank", false, iban),
    ] {
        refuse(
            compte(&pool, c, iban_, kind, primary, provider, account, None).await,
            "chk_bank_accounts_provider",
        );
    }
    refuse(
        compte(&pool, c, iban, "bank", false, None, None, Some(frais)).await,
        "chk_bank_accounts_provider",
    );
    // Clé étrangère du compte de frais.
    refuse(
        compte(
            &pool,
            c,
            None,
            "bank",
            false,
            Some("paypal"),
            Some("X"),
            Some(999_999),
        )
        .await,
        "fk_bank_accounts_fee_account",
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn les_filtres_d_une_regle(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let regle = |provider_type: Option<&'static str>,
                 email: Option<&'static str>,
                 direction: Option<&'static str>,
                 valeur: &'static str| {
        let pool = pool.clone();
        let (c, compte) = (s.company_id, s.accounts["4000"]);
        async move {
            sqlx::query(
                "INSERT INTO reconciliation_rules (company_id, label, match_type, match_value, \
                 counterparty_account_id, priority, provider_type, counterparty_email, direction) \
                 VALUES (?, 'Règle', 'counterparty_contains', ?, ?, 1, ?, ?, ?)",
            )
            .bind(c)
            .bind(valeur)
            .bind(compte)
            .bind(provider_type)
            .bind(email)
            .bind(direction)
            .execute(&pool)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
        }
    };
    regle(None, None, None, "a").await.expect("sans filtre");
    regle(
        Some("Website Payment"),
        Some("a@example.invalid"),
        Some("credit"),
        "b",
    )
    .await
    .expect("trois filtres");
    regle(None, None, Some("debit"), "c").await.expect("débit");
    for (p, e, d, v) in [
        (Some("  "), None, None, "d"),
        (None, Some(""), None, "e"),
        (None, None, Some("Credit"), "f"),
        (None, None, Some("both"), "g"),
    ] {
        refuse(regle(p, e, d, v).await, "chk_reconciliation_rules_filters");
    }
}
