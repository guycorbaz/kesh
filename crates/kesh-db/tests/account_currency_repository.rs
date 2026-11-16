//! Story 37-1 (AC-4, AC-4 bis) — la devise d'un compte et ses comptes de
//! différence de change propres, au dépôt : `accounts::create`, `update` et
//! `archive_guarded` (ordre des refus, verrous), les gardes inverses des sites
//! de configuration (réglages de facturation, articles, règles de
//! réconciliation), les `CHECK` du schéma et les courses de l'AC-4.
//!
//! ⚠️ Toute ligne d'écriture se passe sur un compte **encore CHF** : la devise
//! se tente ensuite.

use std::time::Duration;

use kesh_db::entities::account::{
    Account, AccountRole, AccountType, AccountUpdate, CurrencyChange, NewAccount, OwnFxAccounts,
    OwnFxAccountsChange,
};
use kesh_db::entities::{
    CompanyInvoiceSettingsUpdate, NewProduct, NewReconciliationRule, ProductUpdate,
    ReconciliationMatchType, UpdateReconciliationRule,
};
use kesh_db::errors::{CurrencyUseSite, DbError, FxAccountField, FxAccountReason, FxDesignation};
use kesh_db::repositories::{accounts, company_invoice_settings, products, reconciliation_rules};
use kesh_db::test_fixtures::{
    SeededCompany, attendre_une_requete_en_cours, seed_accounting_company,
};
use rust_decimal::Decimal;
use sqlx::MySqlPool;

// ---------------------------------------------------------------------------
// Montage
// ---------------------------------------------------------------------------

/// Un compte écrit en SQL direct (type, devise, rôle), actif et imputable.
async fn compte(
    pool: &MySqlPool,
    company_id: i64,
    numero: &str,
    type_: &str,
    devise: Option<&str>,
    role: Option<&str>,
) -> i64 {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, role, currency) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(numero)
    .bind(format!("Compte {numero}"))
    .bind(type_)
    .bind(role)
    .bind(devise)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

/// Les comptes de différence de la société, posés par SQL : `(gain, perte,
/// conversion)`.
struct Fx {
    gain: i64,
    perte: i64,
    transit: i64,
}

/// Active le multi-devises de la société par SQL, avec trois comptes valides.
async fn multi(pool: &MySqlPool, s: &SeededCompany) -> Fx {
    let gain = compte(pool, s.company_id, "6950", "Revenue", None, None).await;
    let perte = compte(pool, s.company_id, "6960", "Expense", None, None).await;
    let transit = compte(pool, s.company_id, "1090", "Asset", None, None).await;
    sqlx::query(
        "UPDATE companies SET multi_currency_enabled = TRUE, fx_gain_account_id = ?, \
         fx_loss_account_id = ?, fx_transit_account_id = ? WHERE id = ?",
    )
    .bind(gain)
    .bind(perte)
    .bind(transit)
    .bind(s.company_id)
    .execute(pool)
    .await
    .unwrap();
    Fx {
        gain,
        perte,
        transit,
    }
}

async fn lire(pool: &MySqlPool, id: i64) -> Account {
    accounts::find_by_id(pool, id).await.unwrap().unwrap()
}

/// Le `PUT` qui renvoie le compte tel quel (nom, type, rôle, postabilité),
/// devise et comptes propres inchangés — à modifier par le test.
fn tel_quel(a: &Account) -> AccountUpdate {
    AccountUpdate {
        name: a.name.clone(),
        account_type: a.account_type,
        role: a.role,
        postable: a.postable,
        currency: CurrencyChange::Unchanged,
        own_fx_accounts: OwnFxAccountsChange::Unchanged,
    }
}

async fn modifier(
    pool: &MySqlPool,
    s: &SeededCompany,
    id: i64,
    f: impl FnOnce(&mut AccountUpdate),
) -> Result<Account, DbError> {
    let a = lire(pool, id).await;
    let mut changes = tel_quel(&a);
    f(&mut changes);
    accounts::update(pool, id, a.version, s.admin_user_id, changes, false).await
}

fn usd() -> CurrencyChange {
    CurrencyChange::Foreign("USD".into())
}

async fn archiver(pool: &MySqlPool, s: &SeededCompany, id: i64) -> Result<Account, DbError> {
    let a = lire(pool, id).await;
    accounts::archive_guarded(pool, id, a.version, s.admin_user_id, true).await
}

/// Une écriture CHF à une ligne de débit sur `compte` et une de crédit sur
/// la caisse, en SQL direct.
async fn ligne(pool: &MySqlPool, s: &SeededCompany, compte: i64) {
    let mut tx = pool.begin().await.unwrap();
    ligne_dans(&mut tx, s, compte).await;
    tx.commit().await.unwrap();
}

async fn ligne_dans(
    tx: &mut sqlx::Transaction<'static, sqlx::MySql>,
    s: &SeededCompany,
    compte: i64,
) {
    let numero: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(entry_number), 0) + 1 FROM journal_entries WHERE company_id = ?",
    )
    .bind(s.company_id)
    .fetch_one(&mut **tx)
    .await
    .unwrap();
    let je: i64 = sqlx::query_scalar(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, journal, description) \
         VALUES (?, ?, ?, '2026-06-01', 'Banque', 'test') RETURNING id",
    )
    .bind(s.company_id)
    .bind(s.fiscal_year_id)
    .bind(numero)
    .fetch_one(&mut **tx)
    .await
    .unwrap();
    for (ordre, (id, debit, credit)) in [(compte, 10, 0), (s.accounts["1000"], 0, 10)]
        .into_iter()
        .enumerate()
    {
        sqlx::query(
            "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(je)
        .bind(id)
        .bind(ordre as i32 + 1)
        .bind(debit)
        .bind(credit)
        .execute(&mut **tx)
        .await
        .unwrap();
    }
}

fn designation(e: DbError) -> (FxDesignation, Option<i64>, Option<String>) {
    match e {
        DbError::AccountIsFxDifferenceAccount {
            designation,
            holder_account_id,
            holder_number,
        } => (designation, holder_account_id, holder_number),
        other => panic!("AccountIsFxDifferenceAccount attendu, reçu {other:?}"),
    }
}

fn fx_refus(e: DbError) -> (FxAccountField, FxAccountReason, Option<i64>) {
    match e {
        DbError::FxAccountInvalid {
            field,
            reason,
            holder_account_id,
        } => (field, reason, holder_account_id),
        other => panic!("FxAccountInvalid attendu, reçu {other:?}"),
    }
}

fn site(e: DbError) -> CurrencyUseSite {
    match e {
        DbError::AccountCurrencyInUse { site } => site,
        other => panic!("AccountCurrencyInUse attendu, reçu {other:?}"),
    }
}

/// Un compte en devise qui désigne `gain`/`perte` comme comptes propres (SQL).
async fn detenteur(
    pool: &MySqlPool,
    s: &SeededCompany,
    numero: &str,
    gain: i64,
    perte: i64,
) -> i64 {
    let id = compte(pool, s.company_id, numero, "Asset", Some("EUR"), None).await;
    sqlx::query("UPDATE accounts SET fx_gain_account_id = ?, fx_loss_account_id = ? WHERE id = ?")
        .bind(gain)
        .bind(perte)
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
    id
}

// ---------------------------------------------------------------------------
// create
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn creation_d_un_compte_en_devise(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let mut new = NewAccount::new(s.company_id, "1020", "Banque USD", AccountType::Asset, None);
    new.currency = Some("USD".into());

    // Mono-devise : refusé.
    let err = accounts::create(&pool, s.admin_user_id, new.clone())
        .await
        .unwrap_err();
    assert!(
        matches!(
            err,
            DbError::MultiCurrencyDisabled {
                fiscal_year_id: None
            }
        ),
        "{err:?}"
    );

    multi(&pool, &s).await;
    let a = accounts::create(&pool, s.admin_user_id, new.clone())
        .await
        .unwrap();
    assert_eq!(a.currency.as_deref(), Some("USD"));
    assert_eq!(a.own_fx_accounts(), None);

    // Rôle posé avec la devise → ACCOUNT_IN_FOREIGN_CURRENCY (compte neuf).
    let mut avec_role = new.clone();
    avec_role.number = "1021".into();
    avec_role.role = Some(AccountRole::Receivable);
    let err = accounts::create(&pool, s.admin_user_id, avec_role)
        .await
        .unwrap_err();
    assert!(
        matches!(
            err,
            DbError::AccountInForeignCurrency { account_id: None, ref currency } if currency == "USD"
        ),
        "{err:?}"
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn creation_sous_un_compte_de_difference_refusee(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let fx = multi(&pool, &s).await;
    for (parent, attendu, numero) in [
        (fx.gain, FxDesignation::Gain, "69501"),
        (fx.perte, FxDesignation::Loss, "69601"),
        (fx.transit, FxDesignation::Transit, "10901"),
    ] {
        let a = lire(&pool, parent).await;
        let new = NewAccount::new(
            s.company_id,
            numero,
            "Sous-compte",
            a.account_type,
            Some(parent),
        );
        let err = accounts::create(&pool, s.admin_user_id, new)
            .await
            .unwrap_err();
        assert_eq!(designation(err), (attendu, None, None));
        assert!(
            lire(&pool, parent).await.postable,
            "le parent reste imputable"
        );
    }

    // Désignation propre : le refus nomme le compte en devise qui désigne.
    let propre = compte(&pool, s.company_id, "6970", "Expense", None, None).await;
    let h = detenteur(&pool, &s, "1030", propre, propre).await;
    let new = NewAccount::new(
        s.company_id,
        "69701",
        "Sous",
        AccountType::Expense,
        Some(propre),
    );
    let err = accounts::create(&pool, s.admin_user_id, new)
        .await
        .unwrap_err();
    assert_eq!(
        designation(err),
        (FxDesignation::Gain, Some(h), Some("1030".into()))
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn creation_avec_comptes_propres(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    multi(&pool, &s).await;
    let propre = compte(&pool, s.company_id, "6970", "Expense", None, None).await;

    // Même compte pour les gains et les pertes : admis.
    let mut new = NewAccount::new(s.company_id, "1020", "PayPal USD", AccountType::Asset, None);
    new.currency = Some("USD".into());
    new.own_fx_accounts = Some(OwnFxAccounts {
        gain: propre,
        loss: propre,
    });
    let a = accounts::create(&pool, s.admin_user_id, new.clone())
        .await
        .unwrap();
    assert_eq!(
        a.own_fx_accounts(),
        Some(OwnFxAccounts {
            gain: propre,
            loss: propre
        })
    );

    // Sans devise : `unexpected`.
    let mut chf = new.clone();
    chf.number = "1021".into();
    chf.currency = None;
    let err = accounts::create(&pool, s.admin_user_id, chf)
        .await
        .unwrap_err();
    assert_eq!(
        fx_refus(err),
        (FxAccountField::Gain, FxAccountReason::Unexpected, None)
    );

    // Un compte de bilan en compte de pertes : `accountType`, sur la perte.
    let mut mauvais = new.clone();
    mauvais.number = "1022".into();
    mauvais.own_fx_accounts = Some(OwnFxAccounts {
        gain: propre,
        loss: s.accounts["1100"],
    });
    let err = accounts::create(&pool, s.admin_user_id, mauvais)
        .await
        .unwrap_err();
    assert_eq!(
        fx_refus(err),
        (FxAccountField::Loss, FxAccountReason::AccountType, None)
    );
}

/// Revue de code P2 (37-1-CR-P2-8) — un compte en devise créé sous un parent
/// qu'il nomme compte propre : le parent deviendrait un compte de regroupement
/// désigné, refusé en `notPostable` (état résultant), rien n'est créé et le
/// parent reste imputable.
#[sqlx::test(migrations = "./test-schema")]
async fn creation_sous_son_propre_compte_de_difference_refusee(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    multi(&pool, &s).await;
    let parent = compte(&pool, s.company_id, "6970", "Expense", None, None).await;
    let ok = compte(&pool, s.company_id, "6971", "Expense", None, None).await;
    for (paire, champ, numero) in [
        (
            OwnFxAccounts {
                gain: parent,
                loss: ok,
            },
            FxAccountField::Gain,
            "69701",
        ),
        (
            OwnFxAccounts {
                gain: ok,
                loss: parent,
            },
            FxAccountField::Loss,
            "69702",
        ),
    ] {
        let mut new = NewAccount::new(
            s.company_id,
            numero,
            "Frais de change USD",
            AccountType::Expense,
            Some(parent),
        );
        new.currency = Some("USD".into());
        new.own_fx_accounts = Some(paire);
        let err = accounts::create(&pool, s.admin_user_id, new)
            .await
            .unwrap_err();
        assert_eq!(fx_refus(err), (champ, FxAccountReason::NotPostable, None));
        assert!(
            lire(&pool, parent).await.postable,
            "le parent reste imputable"
        );
    }
    let enfants: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM accounts WHERE parent_id = ?")
        .bind(parent)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(enfants, 0);
}

/// Revue de code P2 (37-1-CR-P2-7) — réactiver un sous-compte archivé d'un
/// compte désigné (le parent, sans enfant actif, était repassé imputable puis
/// désigné) lui rendrait un sous-compte : refusé, désignation nommée — de la
/// société comme propre.
#[sqlx::test(migrations = "./test-schema")]
async fn reactivation_sous_un_compte_de_difference_refusee(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let fx = multi(&pool, &s).await;
    let propre = compte(&pool, s.company_id, "6970", "Expense", None, None).await;
    let h = detenteur(&pool, &s, "1030", propre, propre).await;
    for (parent, attendu, numero) in [
        (fx.perte, (FxDesignation::Loss, None, None), "69601"),
        (
            propre,
            (FxDesignation::Gain, Some(h), Some("1030".into())),
            "69701",
        ),
    ] {
        // L'état atteignable par l'archivage de l'enfant puis la désignation
        // du parent, posé en SQL direct : enfant archivé, parent imputable.
        let enfant = compte(&pool, s.company_id, numero, "Expense", None, None).await;
        sqlx::query("UPDATE accounts SET parent_id = ?, active = FALSE WHERE id = ?")
            .bind(parent)
            .bind(enfant)
            .execute(&pool)
            .await
            .unwrap();
        let e = lire(&pool, enfant).await;
        let err = accounts::reactivate(&pool, enfant, e.version, s.admin_user_id, false)
            .await
            .unwrap_err();
        assert_eq!(designation(err), attendu, "parent {parent}");
        assert!(!lire(&pool, enfant).await.active);
    }

    // Un parent non désigné : la réactivation passe.
    let libre = compte(&pool, s.company_id, "6980", "Expense", None, None).await;
    let enfant = compte(&pool, s.company_id, "69801", "Expense", None, None).await;
    sqlx::query("UPDATE accounts SET parent_id = ?, active = FALSE WHERE id = ?")
        .bind(libre)
        .bind(enfant)
        .execute(&pool)
        .await
        .unwrap();
    let e = lire(&pool, enfant).await;
    assert!(
        accounts::reactivate(&pool, enfant, e.version, s.admin_user_id, false)
            .await
            .unwrap()
            .active
    );
}

// ---------------------------------------------------------------------------
// update — devise
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn devise_posee_puis_verrouillee(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let id = compte(&pool, s.company_id, "1020", "Asset", None, None).await;

    // Mono-devise : refusé.
    let err = modifier(&pool, &s, id, |c| c.currency = usd())
        .await
        .unwrap_err();
    assert!(
        matches!(err, DbError::MultiCurrencyDisabled { .. }),
        "{err:?}"
    );
    // CHF renvoyé en mono-devise : no-op (jamais MULTI_CURRENCY_DISABLED).
    let a = modifier(&pool, &s, id, |c| c.currency = CurrencyChange::Base)
        .await
        .unwrap();
    assert_eq!(a.version, 1, "no-op : pas de bump");

    multi(&pool, &s).await;
    let a = modifier(&pool, &s, id, |c| c.currency = usd())
        .await
        .unwrap();
    assert_eq!(a.currency.as_deref(), Some("USD"));
    // Un compte sans ligne change de devise librement, et redevient CHF.
    let a = modifier(&pool, &s, id, |c| {
        c.currency = CurrencyChange::Foreign("EUR".into())
    })
    .await
    .unwrap();
    assert_eq!(a.currency.as_deref(), Some("EUR"));
    let a = modifier(&pool, &s, id, |c| c.currency = CurrencyChange::Base)
        .await
        .unwrap();
    assert_eq!(a.currency, None);

    // La ligne se passe sur le compte ENCORE CHF ; la devise est ensuite refusée.
    ligne(&pool, &s, id).await;
    let err = modifier(&pool, &s, id, |c| c.currency = usd())
        .await
        .unwrap_err();
    assert!(matches!(err, DbError::AccountCurrencyLocked), "{err:?}");
    // Renvoyer la devise en place n'est pas un changement : pas de verrou.
    let avant = lire(&pool, id).await;
    let a = modifier(&pool, &s, id, |c| c.currency = CurrencyChange::Base)
        .await
        .unwrap();
    assert_eq!(a.version, avant.version);
    // Un renommage (devise absente) reste permis.
    let a = modifier(&pool, &s, id, |c| c.name = "Renommé".into())
        .await
        .unwrap();
    assert_eq!(a.name, "Renommé");
}

#[sqlx::test(migrations = "./test-schema")]
async fn devise_refusee_a_chaque_site_de_configuration(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    multi(&pool, &s).await;
    let c = s.company_id;

    // Rôle gardé.
    let a_role = compte(&pool, c, "1105", "Asset", None, Some("Receivable")).await;
    let err = modifier(&pool, &s, a_role, |u| u.currency = usd())
        .await
        .unwrap_err();
    assert_eq!(site(err), CurrencyUseSite::Role("Receivable"));
    // Rôle retiré dans le même PUT : admis.
    let a = modifier(&pool, &s, a_role, |u| {
        u.currency = usd();
        u.role = None;
    })
    .await
    .unwrap();
    assert_eq!(a.currency.as_deref(), Some("USD"));

    // Réglages de facturation (3000 désigné compte de produit par défaut).
    let err = modifier(&pool, &s, s.accounts["3000"], |u| u.currency = usd())
        .await
        .unwrap_err();
    assert_eq!(site(err), CurrencyUseSite::InvoiceSettings("revenue"));

    // Compte bancaire ARCHIVÉ : le lien compte quand même.
    let lie = compte(&pool, c, "1021", "Asset", None, None).await;
    let banque: i64 = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, journal_account_id, archived) \
         VALUES (?, 'B', 'CH9300762011623852957', ?, TRUE)",
    )
    .bind(c)
    .bind(lie)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let err = modifier(&pool, &s, lie, |u| u.currency = usd())
        .await
        .unwrap_err();
    assert_eq!(site(err), CurrencyUseSite::BankAccount(banque));

    // Article ARCHIVÉ.
    let produit = compte(&pool, c, "3200", "Revenue", None, None).await;
    let article: i64 = sqlx::query(
        "INSERT INTO products (company_id, name, unit_price, vat_rate, default_revenue_account_id, active) \
         VALUES (?, 'Article', 10, 8.1, ?, FALSE)",
    )
    .bind(c)
    .bind(produit)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let err = modifier(&pool, &s, produit, |u| u.currency = usd())
        .await
        .unwrap_err();
    assert_eq!(site(err), CurrencyUseSite::Product(article));

    // Règle INACTIVE.
    let contrepartie = compte(&pool, c, "4100", "Expense", None, None).await;
    let regle: i64 = sqlx::query(
        "INSERT INTO reconciliation_rules (company_id, label, match_type, match_value, \
         counterparty_account_id, priority, active) VALUES (?, 'R', 'counterparty_contains', 'x', ?, 1, FALSE)",
    )
    .bind(c)
    .bind(contrepartie)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let err = modifier(&pool, &s, contrepartie, |u| u.currency = usd())
        .await
        .unwrap_err();
    assert_eq!(site(err), CurrencyUseSite::ReconciliationRule(regle));
}

#[sqlx::test(migrations = "./test-schema")]
async fn role_pose_sur_un_compte_en_devise_refuse(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    multi(&pool, &s).await;
    let id = compte(&pool, s.company_id, "1020", "Asset", Some("USD"), None).await;
    let err = modifier(&pool, &s, id, |u| u.role = Some(AccountRole::Receivable))
        .await
        .unwrap_err();
    assert!(
        matches!(err, DbError::AccountInForeignCurrency { account_id: Some(a), .. } if a == id),
        "{err:?}"
    );
    // Devise ET rôle posés dans le même PUT : ACCOUNT_IN_FOREIGN_CURRENCY.
    let chf = compte(&pool, s.company_id, "1021", "Asset", None, None).await;
    let err = modifier(&pool, &s, chf, |u| {
        u.currency = usd();
        u.role = Some(AccountRole::Receivable);
    })
    .await
    .unwrap_err();
    assert!(
        matches!(err, DbError::AccountInForeignCurrency { .. }),
        "{err:?}"
    );
}

// ---------------------------------------------------------------------------
// update — comptes de différence de change désignés (gardes inverses)
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn compte_de_difference_de_la_societe_garde(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let fx = multi(&pool, &s).await;

    // Gains : devise, retypage hors charges/produits, non imputable, rôle produit.
    let err = modifier(&pool, &s, fx.gain, |u| u.currency = usd())
        .await
        .unwrap_err();
    assert_eq!(designation(err), (FxDesignation::Gain, None, None));
    let err = modifier(&pool, &s, fx.gain, |u| u.account_type = AccountType::Asset)
        .await
        .unwrap_err();
    assert_eq!(designation(err).0, FxDesignation::Gain);
    let err = modifier(&pool, &s, fx.perte, |u| u.postable = false)
        .await
        .unwrap_err();
    assert_eq!(designation(err).0, FxDesignation::Loss);
    let err = modifier(&pool, &s, fx.gain, |u| {
        u.role = Some(AccountRole::DefaultRevenue)
    })
    .await
    .unwrap_err();
    assert_eq!(designation(err).0, FxDesignation::Gain);
    // Entre charges et produits : admis.
    let a = modifier(&pool, &s, fx.perte, |u| {
        u.account_type = AccountType::Revenue
    })
    .await
    .unwrap();
    assert_eq!(a.account_type, AccountType::Revenue);

    // Conversion : hors actif/passif refusé, vers le passif admis.
    let err = modifier(&pool, &s, fx.transit, |u| {
        u.account_type = AccountType::Expense
    })
    .await
    .unwrap_err();
    assert_eq!(designation(err), (FxDesignation::Transit, None, None));
    let err = modifier(&pool, &s, fx.transit, |u| u.currency = usd())
        .await
        .unwrap_err();
    assert_eq!(designation(err).0, FxDesignation::Transit);
    let a = modifier(&pool, &s, fx.transit, |u| {
        u.account_type = AccountType::Liability
    })
    .await
    .unwrap();
    assert_eq!(a.account_type, AccountType::Liability);
}

#[sqlx::test(migrations = "./test-schema")]
async fn compte_de_difference_propre_garde_avec_son_detenteur(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    multi(&pool, &s).await;
    let gain = compte(&pool, s.company_id, "6971", "Revenue", None, None).await;
    let perte = compte(&pool, s.company_id, "6972", "Expense", None, None).await;
    let h1 = detenteur(&pool, &s, "1031", gain, perte).await;
    let h2 = detenteur(&pool, &s, "1032", perte, gain).await;
    // Le détenteur archivé compte aussi.
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id IN (?, ?)")
        .bind(h1)
        .bind(h2)
        .execute(&pool)
        .await
        .unwrap();

    // `gain` : gain de h1, perte de h2 → le plus petit identifiant (h1), gains.
    let err = modifier(&pool, &s, gain, |u| u.currency = usd())
        .await
        .unwrap_err();
    assert_eq!(
        designation(err),
        (FxDesignation::Gain, Some(h1), Some("1031".into()))
    );
    // `perte` : perte de h1 (plus petit), gain de h2 → pertes.
    let err = modifier(&pool, &s, perte, |u| u.postable = false)
        .await
        .unwrap_err();
    assert_eq!(
        designation(err),
        (FxDesignation::Loss, Some(h1), Some("1031".into()))
    );
    let err = archiver(&pool, &s, perte).await.unwrap_err();
    assert_eq!(designation(err).1, Some(h1));
    let err = modifier(&pool, &s, gain, |u| u.account_type = AccountType::Liability)
        .await
        .unwrap_err();
    assert_eq!(designation(err).1, Some(h1));
    let err = modifier(&pool, &s, gain, |u| {
        u.role = Some(AccountRole::DefaultRevenue)
    })
    .await
    .unwrap_err();
    assert_eq!(designation(err).1, Some(h1));
}

#[sqlx::test(migrations = "./test-schema")]
async fn ordre_des_refus_de_la_modification(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let fx = multi(&pool, &s).await;

    // Version d'abord.
    let a = lire(&pool, fx.gain).await;
    let mut c = tel_quel(&a);
    c.currency = usd();
    let err = accounts::update(&pool, fx.gain, a.version + 1, s.admin_user_id, c, false)
        .await
        .unwrap_err();
    assert!(matches!(err, DbError::OptimisticLockConflict), "{err:?}");

    // ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT avant ACCOUNT_CURRENCY_LOCKED : le compte
    // de pertes porte une ligne (passée tant qu'il est CHF).
    ligne(&pool, &s, fx.perte).await;
    let err = modifier(&pool, &s, fx.perte, |u| u.currency = usd())
        .await
        .unwrap_err();
    assert_eq!(designation(err).0, FxDesignation::Loss);

    // ACCOUNT_CURRENCY_LOCKED avant ACCOUNT_CURRENCY_IN_USE : 3000 porte une
    // ligne ET est désigné dans les réglages.
    ligne(&pool, &s, s.accounts["3000"]).await;
    let err = modifier(&pool, &s, s.accounts["3000"], |u| u.currency = usd())
        .await
        .unwrap_err();
    assert!(matches!(err, DbError::AccountCurrencyLocked), "{err:?}");

    // ACCOUNT_CURRENCY_IN_USE avant ACCOUNT_IN_FOREIGN_CURRENCY : rôle gardé
    // (IN_USE) même si un autre site désigne aussi.
    let role = compte(
        &pool,
        s.company_id,
        "1106",
        "Asset",
        None,
        Some("Receivable"),
    )
    .await;
    let err = modifier(&pool, &s, role, |u| u.currency = usd())
        .await
        .unwrap_err();
    assert!(
        matches!(err, DbError::AccountCurrencyInUse { .. }),
        "{err:?}"
    );

    // ACCOUNT_IN_FOREIGN_CURRENCY avant FX_ACCOUNT_INVALID : rôle posé et
    // comptes propres invalides dans le même PUT.
    let chf = compte(&pool, s.company_id, "1022", "Asset", None, None).await;
    let err = modifier(&pool, &s, chf, |u| {
        u.currency = usd();
        u.role = Some(AccountRole::Receivable);
        u.own_fx_accounts = OwnFxAccountsChange::Own(OwnFxAccounts {
            gain: s.accounts["1100"],
            loss: s.accounts["1100"],
        });
    })
    .await
    .unwrap_err();
    assert!(
        matches!(err, DbError::AccountInForeignCurrency { .. }),
        "{err:?}"
    );

    // Compte archivé : refus d'état, avant tout le reste.
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(chf)
        .execute(&pool)
        .await
        .unwrap();
    let err = modifier(&pool, &s, chf, |u| u.currency = usd())
        .await
        .unwrap_err();
    assert!(matches!(err, DbError::IllegalStateTransition(_)), "{err:?}");
}

// ---------------------------------------------------------------------------
// update — comptes propres
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn comptes_propres_poses_modifies_et_rendus_au_defaut(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    multi(&pool, &s).await;
    let c = s.company_id;
    let g = compte(&pool, c, "6971", "Revenue", None, None).await;
    let p = compte(&pool, c, "6972", "Expense", None, None).await;
    let id = compte(&pool, c, "1020", "Asset", Some("USD"), None).await;

    let a = modifier(&pool, &s, id, |u| {
        u.own_fx_accounts = OwnFxAccountsChange::Own(OwnFxAccounts { gain: g, loss: p })
    })
    .await
    .unwrap();
    assert_eq!(
        a.own_fx_accounts(),
        Some(OwnFxAccounts { gain: g, loss: p })
    );
    let audit: String = sqlx::query_scalar(
        "SELECT CAST(details_json AS CHAR) FROM audit_log WHERE action = 'account.updated' AND entity_id = ? \
         ORDER BY id DESC LIMIT 1",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let audit: serde_json::Value = serde_json::from_str(&audit).unwrap();
    assert_eq!(audit["after"]["fxGainAccountId"], g);
    assert_eq!(audit["before"]["fxGainAccountId"], serde_json::Value::Null);

    // Absents = inchangés.
    let a = modifier(&pool, &s, id, |u| u.name = "PayPal".into())
        .await
        .unwrap();
    assert_eq!(
        a.own_fx_accounts(),
        Some(OwnFxAccounts { gain: g, loss: p })
    );
    // Un compte propre devenu invalide n'empêche pas un renommage (motif C4).
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(g)
        .execute(&pool)
        .await
        .unwrap();
    modifier(&pool, &s, id, |u| u.name = "PayPal 2".into())
        .await
        .unwrap();
    // … mais le reposer, si : `archived`, avec le détenteur.
    let err = modifier(&pool, &s, id, |u| {
        u.own_fx_accounts = OwnFxAccountsChange::Own(OwnFxAccounts { gain: p, loss: g })
    })
    .await
    .unwrap_err();
    assert_eq!(
        fx_refus(err),
        (FxAccountField::Loss, FxAccountReason::Archived, Some(id))
    );

    // Retour au défaut.
    let a = modifier(&pool, &s, id, |u| {
        u.own_fx_accounts = OwnFxAccountsChange::Company
    })
    .await
    .unwrap();
    assert_eq!(a.own_fx_accounts(), None);
}

#[sqlx::test(migrations = "./test-schema")]
async fn comptes_propres_chaque_raison(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let fx = multi(&pool, &s).await;
    let c = s.company_id;
    let id = compte(&pool, c, "1020", "Asset", Some("USD"), None).await;
    let ok = compte(&pool, c, "6971", "Expense", None, None).await;
    let archive = compte(&pool, c, "6973", "Expense", None, None).await;
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(archive)
        .execute(&pool)
        .await
        .unwrap();
    let groupe = compte(&pool, c, "6974", "Expense", None, None).await;
    sqlx::query("UPDATE accounts SET postable = FALSE WHERE id = ?")
        .bind(groupe)
        .execute(&pool)
        .await
        .unwrap();
    let en_devise = compte(&pool, c, "6975", "Expense", Some("EUR"), None).await;
    let cas = [
        (999_999_999, FxAccountReason::Unknown),
        (archive, FxAccountReason::Archived),
        (groupe, FxAccountReason::NotPostable),
        (s.accounts["1100"], FxAccountReason::AccountType),
        (fx.transit, FxAccountReason::AccountType),
        (en_devise, FxAccountReason::ForeignCurrency),
        (s.accounts["3000"], FxAccountReason::RevenueAccount),
    ];
    for (compte_refuse, raison) in cas {
        let err = modifier(&pool, &s, id, |u| {
            u.own_fx_accounts = OwnFxAccountsChange::Own(OwnFxAccounts {
                gain: compte_refuse,
                loss: ok,
            })
        })
        .await
        .unwrap_err();
        assert_eq!(
            fx_refus(err),
            (FxAccountField::Gain, raison, Some(id)),
            "compte {compte_refuse}"
        );
    }

    // `unexpected` : comptes propres sur un compte CHF, et sur un compte rendu
    // CHF dans le même PUT (sans `null` pour ses comptes propres).
    let chf = compte(&pool, c, "1021", "Asset", None, None).await;
    let err = modifier(&pool, &s, chf, |u| {
        u.own_fx_accounts = OwnFxAccountsChange::Own(OwnFxAccounts { gain: ok, loss: ok })
    })
    .await
    .unwrap_err();
    assert_eq!(
        fx_refus(err),
        (FxAccountField::Gain, FxAccountReason::Unexpected, Some(chf))
    );
    modifier(&pool, &s, id, |u| {
        u.own_fx_accounts = OwnFxAccountsChange::Own(OwnFxAccounts { gain: ok, loss: ok })
    })
    .await
    .unwrap();
    let err = modifier(&pool, &s, id, |u| u.currency = CurrencyChange::Base)
        .await
        .unwrap_err();
    assert_eq!(fx_refus(err).1, FxAccountReason::Unexpected);
    // Avec `null` pour les comptes propres : admis.
    let a = modifier(&pool, &s, id, |u| {
        u.currency = CurrencyChange::Base;
        u.own_fx_accounts = OwnFxAccountsChange::Company;
    })
    .await
    .unwrap();
    assert_eq!((a.currency.clone(), a.own_fx_accounts()), (None, None));
}

/// Revue de code P2 (37-1-CR-P2-1) — un compte CHF qui reçoit une devise dans
/// le même `PUT` ne se nomme pas lui-même gain ou perte : lu avant l'`UPDATE`,
/// il est encore CHF, et seul le contrôle de l'état résultant le refuse
/// (`foreignCurrency`, champ visé). Rien n'est écrit.
#[sqlx::test(migrations = "./test-schema")]
async fn comptes_propres_un_compte_ne_se_designe_pas_lui_meme(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    multi(&pool, &s).await;
    let c = s.company_id;
    let ok = compte(&pool, c, "6971", "Expense", None, None).await;
    // Un compte de charges CHF, imputable, sans rôle : il passerait toutes les
    // gardes de l'AC-3 s'il n'était pas le compte qui reçoit la devise.
    let id = compte(&pool, c, "6972", "Expense", None, None).await;
    for (paire, champ) in [
        (OwnFxAccounts { gain: id, loss: ok }, FxAccountField::Gain),
        (OwnFxAccounts { gain: ok, loss: id }, FxAccountField::Loss),
    ] {
        let err = modifier(&pool, &s, id, |u| {
            u.currency = usd();
            u.own_fx_accounts = OwnFxAccountsChange::Own(paire);
        })
        .await
        .unwrap_err();
        assert_eq!(
            fx_refus(err),
            (champ, FxAccountReason::ForeignCurrency, Some(id))
        );
        let a = lire(&pool, id).await;
        assert_eq!((a.currency.clone(), a.own_fx_accounts()), (None, None));
    }
}

// ---------------------------------------------------------------------------
// archive_guarded
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn archivage_d_un_compte_de_difference_sans_recours(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let fx = multi(&pool, &s).await;
    for (id, attendu) in [
        (fx.gain, FxDesignation::Gain),
        (fx.perte, FxDesignation::Loss),
        (fx.transit, FxDesignation::Transit),
    ] {
        // `confirm_designated = true` ne lève rien.
        let err = archiver(&pool, &s, id).await.unwrap_err();
        assert_eq!(designation(err), (attendu, None, None));
        assert!(lire(&pool, id).await.active);
    }

    // Ordre : sous-comptes actifs d'abord.
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, parent_id) VALUES (?, '69601', 'x', 'Expense', ?)",
    )
    .bind(s.company_id)
    .bind(fx.perte)
    .execute(&pool)
    .await
    .unwrap();
    let err = archiver(&pool, &s, fx.perte).await.unwrap_err();
    assert!(matches!(err, DbError::IllegalStateTransition(_)), "{err:?}");

    // Puis la désignation de change, avant celle des réglages de facturation
    // (le compte de gains désigné aussi compte d'arrondi), avant la version.
    sqlx::query(
        "UPDATE company_invoice_settings SET default_rounding_account_id = ? WHERE company_id = ?",
    )
    .bind(fx.gain)
    .bind(s.company_id)
    .execute(&pool)
    .await
    .unwrap();
    let a = lire(&pool, fx.gain).await;
    let err = accounts::archive_guarded(&pool, fx.gain, a.version + 5, s.admin_user_id, false)
        .await
        .unwrap_err();
    assert_eq!(designation(err).0, FxDesignation::Gain);

    // Le compte en devise qui désigne, lui, s'archive.
    let propre = compte(&pool, s.company_id, "6971", "Expense", None, None).await;
    let h = detenteur(&pool, &s, "1031", propre, propre).await;
    archiver(&pool, &s, h).await.unwrap();
    // … et sa désignation compte toujours.
    let err = archiver(&pool, &s, propre).await.unwrap_err();
    assert_eq!(
        designation(err),
        (FxDesignation::Gain, Some(h), Some("1031".into()))
    );
}

// ---------------------------------------------------------------------------
// Sites de configuration (gardes inverses)
// ---------------------------------------------------------------------------

fn reglages_tels_quels(
    r: &kesh_db::entities::CompanyInvoiceSettings,
) -> CompanyInvoiceSettingsUpdate {
    CompanyInvoiceSettingsUpdate {
        invoice_number_format: r.invoice_number_format.clone(),
        default_receivable_account_id: r.default_receivable_account_id,
        default_revenue_account_id: r.default_revenue_account_id,
        default_vat_payable_account_id: r.default_vat_payable_account_id,
        default_vat_recoverable_account_id: r.default_vat_recoverable_account_id,
        default_vat_decompte_account_id: r.default_vat_decompte_account_id,
        default_sales_journal: r.default_sales_journal,
        journal_entry_description_template: r.journal_entry_description_template.clone(),
        credit_note_number_format: r.credit_note_number_format.clone(),
        supplier_internal_number_format: r.supplier_internal_number_format.clone(),
        default_payable_account_id: r.default_payable_account_id,
        default_rounding_account_id: r.default_rounding_account_id,
        round_to_5_centimes: r.round_to_5_centimes,
        minimum_invoice_amount: r.minimum_invoice_amount,
        default_discount_account_id: r.default_discount_account_id,
        default_bank_fees_account_id: r.default_bank_fees_account_id,
        default_bad_debt_account_id: r.default_bad_debt_account_id,
    }
}

async fn reglages(
    pool: &MySqlPool,
    s: &SeededCompany,
    f: impl FnOnce(&mut CompanyInvoiceSettingsUpdate),
) -> Result<kesh_db::entities::CompanyInvoiceSettings, DbError> {
    let r = company_invoice_settings::get_or_create_default(pool, s.company_id)
        .await
        .unwrap();
    let mut u = reglages_tels_quels(&r);
    f(&mut u);
    company_invoice_settings::update(pool, s.company_id, r.version, s.admin_user_id, u).await
}

#[sqlx::test(migrations = "./test-schema")]
async fn reglages_de_facturation_gardes_inverses(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let fx = multi(&pool, &s).await;
    let c = s.company_id;
    let usd = compte(&pool, c, "6990", "Expense", Some("USD"), None).await;

    // Un compte en devise, à n'importe quel réglage changé.
    let err = reglages(&pool, &s, |u| u.default_rounding_account_id = Some(usd))
        .await
        .unwrap_err();
    assert!(
        matches!(err, DbError::AccountInForeignCurrency { account_id: Some(a), .. } if a == usd),
        "{err:?}"
    );

    // Le compte de gains en compte de produit par défaut : refusé.
    let err = reglages(&pool, &s, |u| u.default_revenue_account_id = Some(fx.gain))
        .await
        .unwrap_err();
    assert_eq!(designation(err), (FxDesignation::Gain, None, None));
    // Désignation propre : le détenteur est nommé.
    let propre = compte(&pool, c, "3900", "Revenue", None, None).await;
    let h = detenteur(&pool, &s, "1031", propre, propre).await;
    let err = reglages(&pool, &s, |u| u.default_revenue_account_id = Some(propre))
        .await
        .unwrap_err();
    assert_eq!(
        designation(err),
        (FxDesignation::Gain, Some(h), Some("1031".into()))
    );
    // Les autres désignations restent admises pour un compte de différence.
    reglages(&pool, &s, |u| {
        u.default_rounding_account_id = Some(fx.perte)
    })
    .await
    .unwrap();

    // Inchangé (motif C4) : rien n'est rejugé. Le compte d'arrondi passe en
    // devise par SQL ; un autre réglage s'enregistre.
    sqlx::query(
        "UPDATE company_invoice_settings SET default_bank_fees_account_id = ? WHERE company_id = ?",
    )
    .bind(usd)
    .bind(c)
    .execute(&pool)
    .await
    .unwrap();
    reglages(&pool, &s, |u| {
        u.journal_entry_description_template = "Autre".into()
    })
    .await
    .unwrap();
}

#[sqlx::test(migrations = "./test-schema")]
async fn articles_gardes_inverses(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let fx = multi(&pool, &s).await;
    let c = s.company_id;
    let usd = compte(&pool, c, "3990", "Revenue", Some("USD"), None).await;
    let nouveau = |compte: Option<i64>| NewProduct {
        company_id: c,
        name: "Article".into(),
        description: None,
        unit_price: Decimal::new(10, 0),
        vat_rate: Decimal::new(81, 1),
        default_revenue_account_id: compte,
    };
    let err = products::create(&pool, s.admin_user_id, nouveau(Some(usd)))
        .await
        .unwrap_err();
    assert!(
        matches!(err, DbError::AccountInForeignCurrency { .. }),
        "{err:?}"
    );
    let err = products::create(&pool, s.admin_user_id, nouveau(Some(fx.gain)))
        .await
        .unwrap_err();
    assert_eq!(designation(err).0, FxDesignation::Gain);

    let p = products::create(&pool, s.admin_user_id, nouveau(Some(s.accounts["3000"])))
        .await
        .unwrap();
    let maj = |compte: Option<i64>, nom: &str| ProductUpdate {
        name: nom.into(),
        description: None,
        unit_price: Decimal::new(10, 0),
        vat_rate: Decimal::new(81, 1),
        default_revenue_account_id: compte,
    };
    let err = products::update(
        &pool,
        c,
        p.id,
        p.version,
        s.admin_user_id,
        maj(Some(fx.perte), "A"),
    )
    .await
    .unwrap_err();
    assert_eq!(designation(err).0, FxDesignation::Loss);
    let err = products::update(
        &pool,
        c,
        p.id,
        p.version,
        s.admin_user_id,
        maj(Some(usd), "A"),
    )
    .await
    .unwrap_err();
    assert!(
        matches!(err, DbError::AccountInForeignCurrency { .. }),
        "{err:?}"
    );
    // Inchangé : pas de refus, même si le compte en place est devenu en devise.
    sqlx::query("UPDATE products SET default_revenue_account_id = ? WHERE id = ?")
        .bind(usd)
        .bind(p.id)
        .execute(&pool)
        .await
        .unwrap();
    products::update(
        &pool,
        c,
        p.id,
        p.version,
        s.admin_user_id,
        maj(Some(usd), "Renommé"),
    )
    .await
    .unwrap();
}

#[sqlx::test(migrations = "./test-schema")]
async fn regles_de_reconciliation_refusent_un_compte_en_devise(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    multi(&pool, &s).await;
    let c = s.company_id;
    let usd = compte(&pool, c, "4990", "Expense", Some("USD"), None).await;
    let regle = |compte: i64, valeur: &str| NewReconciliationRule {
        label: "R".into(),
        match_type: ReconciliationMatchType::CounterpartyContains,
        match_value: valeur.into(),
        counterparty_account_id: compte,
        priority: 1,
        default_project_id: None,
    };
    let mut tx = pool.begin().await.unwrap();
    let err = reconciliation_rules::create_in_tx(&mut tx, c, s.admin_user_id, &regle(usd, "a"))
        .await
        .unwrap_err();
    assert!(
        matches!(err, DbError::AccountInForeignCurrency { .. }),
        "{err:?}"
    );
    tx.rollback().await.unwrap();

    let mut tx = pool.begin().await.unwrap();
    let r = reconciliation_rules::create_in_tx(
        &mut tx,
        c,
        s.admin_user_id,
        &regle(s.accounts["4000"], "b"),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    let err = reconciliation_rules::update_in_tx(
        &mut tx,
        c,
        s.admin_user_id,
        r.id,
        r.version,
        &UpdateReconciliationRule {
            counterparty_account_id: Some(usd),
            ..Default::default()
        },
    )
    .await
    .unwrap_err();
    assert!(
        matches!(err, DbError::AccountInForeignCurrency { .. }),
        "{err:?}"
    );
}

// ---------------------------------------------------------------------------
// Schéma
// ---------------------------------------------------------------------------

/// Les `CHECK` de la migration, en SQL brut : paire de comptes propres, comptes
/// propres sur un compte CHF, `CHF` écrit en toutes lettres.
#[sqlx::test(migrations = "./test-schema")]
async fn les_check_du_schema_refusent_en_sql_brut(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let c = s.company_id;
    let g = s.accounts["4000"];
    for (sql, contrainte) in [
        (
            "INSERT INTO accounts (company_id, number, name, account_type, currency, fx_gain_account_id) \
             VALUES (?, '1050', 'x', 'Asset', 'USD', ?)",
            "chk_accounts_fx_own_pair",
        ),
        (
            "INSERT INTO accounts (company_id, number, name, account_type, fx_gain_account_id, fx_loss_account_id) \
             VALUES (?, '1051', 'x', 'Asset', ?, ?)",
            "chk_accounts_fx_own_foreign",
        ),
        (
            "INSERT INTO accounts (company_id, number, name, account_type, currency) \
             VALUES (?, '1052', 'x', 'Asset', 'CHF')",
            "chk_accounts_currency_not_base",
        ),
    ] {
        let err = sqlx::query(sql)
            .bind(c)
            .bind(g)
            .bind(g)
            .execute(&pool)
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains(contrainte),
            "{contrainte} attendue : {err}"
        );
    }
}

// ---------------------------------------------------------------------------
// Courses (AC-4, AC-4 bis)
// ---------------------------------------------------------------------------

const ATTENTE: Duration = Duration::from_secs(20);

/// Activation (non commitée, `companies` tenu `FOR UPDATE`) contre création
/// d'un compte en devise : la création attend `companies` en partage, puis
/// voit l'activation commitée.
#[sqlx::test(migrations = "./test-schema")]
async fn course_activation_contre_creation_en_devise(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM companies WHERE id = ? FOR UPDATE")
        .bind(s.company_id)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    sqlx::query("UPDATE companies SET multi_currency_enabled = TRUE WHERE id = ?")
        .bind(s.company_id)
        .execute(&mut *tx)
        .await
        .unwrap();

    let (pool_b, c, u) = (pool.clone(), s.company_id, s.admin_user_id);
    let creation = tokio::spawn(async move {
        let mut new = NewAccount::new(c, "1020", "USD", AccountType::Asset, None);
        new.currency = Some("USD".into());
        accounts::create(&pool_b, u, new).await
    });
    assert!(
        attendre_une_requete_en_cours(&pool, &["FROM companies", "LOCK IN SHARE MODE"], || {
            creation.is_finished()
        })
        .await,
        "la création doit attendre la ligne companies"
    );
    tx.commit().await.unwrap();
    let a = tokio::time::timeout(ATTENTE, creation)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(a.currency.as_deref(), Some("USD"));
}

/// Désignation de la société (non commitée) contre archivage et retypage du
/// compte désigné : chacun attend, puis refuse.
#[sqlx::test(migrations = "./test-schema")]
async fn course_designation_contre_archivage_et_retypage(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    multi(&pool, &s).await;
    let cible = compte(&pool, s.company_id, "6980", "Expense", None, None).await;
    for retypage in [false, true] {
        let mut tx = pool.begin().await.unwrap();
        sqlx::query("SELECT id FROM companies WHERE id = ? FOR UPDATE")
            .bind(s.company_id)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
        sqlx::query("UPDATE companies SET fx_loss_account_id = ? WHERE id = ?")
            .bind(cible)
            .bind(s.company_id)
            .execute(&mut *tx)
            .await
            .unwrap();
        let (pool_b, s_b) = (pool.clone(), s.clone());
        let geste = tokio::spawn(async move {
            if retypage {
                modifier(&pool_b, &s_b, cible, |u| {
                    u.account_type = AccountType::Asset
                })
                .await
            } else {
                archiver(&pool_b, &s_b, cible).await
            }
        });
        assert!(
            attendre_une_requete_en_cours(&pool, &["FROM companies", "LOCK IN SHARE MODE"], || {
                geste.is_finished()
            })
            .await,
            "le geste doit attendre la ligne companies"
        );
        tx.commit().await.unwrap();
        let err = tokio::time::timeout(ATTENTE, geste)
            .await
            .unwrap()
            .unwrap()
            .unwrap_err();
        assert_eq!(
            designation(err).0,
            FxDesignation::Loss,
            "retypage = {retypage}"
        );
    }
}

/// Première ligne (non commitée) contre changement de devise : le `PUT`
/// attend le compte, puis voit la ligne commitée → `ACCOUNT_CURRENCY_LOCKED`.
#[sqlx::test(migrations = "./test-schema")]
async fn course_changement_de_devise_contre_premiere_ligne(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    multi(&pool, &s).await;
    let id = compte(&pool, s.company_id, "1020", "Asset", None, None).await;
    let mut tx = pool.begin().await.unwrap();
    ligne_dans(&mut tx, &s, id).await;

    let (pool_b, s_b) = (pool.clone(), s.clone());
    let geste =
        tokio::spawn(async move { modifier(&pool_b, &s_b, id, |u| u.currency = usd()).await });
    assert!(
        attendre_une_requete_en_cours(&pool, &["FROM accounts WHERE id = ? FOR UPDATE"], || {
            geste.is_finished()
        })
        .await,
        "le PUT doit attendre la ligne du compte"
    );
    tx.commit().await.unwrap();
    let err = tokio::time::timeout(ATTENTE, geste)
        .await
        .unwrap()
        .unwrap()
        .unwrap_err();
    assert!(matches!(err, DbError::AccountCurrencyLocked), "{err:?}");
}

/// Désignation dans les réglages de facturation (non commitée, réglages tenus
/// `FOR UPDATE`) contre changement de devise : le `PUT` attend les réglages en
/// partage, puis voit la désignation → `ACCOUNT_CURRENCY_IN_USE`.
#[sqlx::test(migrations = "./test-schema")]
async fn course_changement_de_devise_contre_designation_de_facturation(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    multi(&pool, &s).await;
    let id = compte(&pool, s.company_id, "6900", "Expense", None, None).await;
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT company_id FROM company_invoice_settings WHERE company_id = ? FOR UPDATE")
        .bind(s.company_id)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    sqlx::query(
        "UPDATE company_invoice_settings SET default_bank_fees_account_id = ? WHERE company_id = ?",
    )
    .bind(id)
    .bind(s.company_id)
    .execute(&mut *tx)
    .await
    .unwrap();

    let (pool_b, s_b) = (pool.clone(), s.clone());
    let geste =
        tokio::spawn(async move { modifier(&pool_b, &s_b, id, |u| u.currency = usd()).await });
    assert!(
        attendre_une_requete_en_cours(
            &pool,
            &["FROM company_invoice_settings", "LOCK IN SHARE MODE"],
            || geste.is_finished()
        )
        .await,
        "le PUT doit attendre les réglages"
    );
    tx.commit().await.unwrap();
    let err = tokio::time::timeout(ATTENTE, geste)
        .await
        .unwrap()
        .unwrap()
        .unwrap_err();
    assert_eq!(site(err), CurrencyUseSite::InvoiceSettings("bankFees"));
}

/// Lien bancaire (non commité : type relu en partage, puis `INSERT`) contre
/// changement de devise : le `PUT` attend le compte, puis voit le lien.
#[sqlx::test(migrations = "./test-schema")]
async fn course_changement_de_devise_contre_lien_bancaire(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    multi(&pool, &s).await;
    let id = compte(&pool, s.company_id, "1020", "Asset", None, None).await;
    let mut tx = pool.begin().await.unwrap();
    let lu = accounts::type_in_share_mode(&mut tx, s.company_id, id)
        .await
        .unwrap();
    assert_eq!(lu, Some((AccountType::Asset, true, None)));
    let banque = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, journal_account_id) \
         VALUES (?, 'B', 'CH9300762011623852957', ?)",
    )
    .bind(s.company_id)
    .bind(id)
    .execute(&mut *tx)
    .await
    .unwrap()
    .last_insert_id() as i64;

    let (pool_b, s_b) = (pool.clone(), s.clone());
    let geste =
        tokio::spawn(async move { modifier(&pool_b, &s_b, id, |u| u.currency = usd()).await });
    assert!(
        attendre_une_requete_en_cours(&pool, &["FROM accounts WHERE id = ? FOR UPDATE"], || {
            geste.is_finished()
        })
        .await,
        "le PUT doit attendre la ligne du compte"
    );
    tx.commit().await.unwrap();
    let err = tokio::time::timeout(ATTENTE, geste)
        .await
        .unwrap()
        .unwrap()
        .unwrap_err();
    assert_eq!(site(err), CurrencyUseSite::BankAccount(banque));
}

/// Désignation propre contre archivage du compte désigné, dans les deux sens.
#[sqlx::test(migrations = "./test-schema")]
async fn course_designation_propre_contre_archivage_du_designe(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    multi(&pool, &s).await;
    let c = s.company_id;
    let designe = compte(&pool, c, "6981", "Expense", None, None).await;
    let h = compte(&pool, c, "1020", "Asset", Some("USD"), None).await;

    // (a) La désignation commitée d'abord : l'archivage attend `companies`,
    //     puis refuse.
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM companies WHERE id = ? FOR UPDATE")
        .bind(c)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    sqlx::query("UPDATE accounts SET fx_gain_account_id = ?, fx_loss_account_id = ? WHERE id = ?")
        .bind(designe)
        .bind(designe)
        .bind(h)
        .execute(&mut *tx)
        .await
        .unwrap();
    let (pool_b, s_b) = (pool.clone(), s.clone());
    let archivage = tokio::spawn(async move { archiver(&pool_b, &s_b, designe).await });
    assert!(
        attendre_une_requete_en_cours(&pool, &["FROM companies", "LOCK IN SHARE MODE"], || {
            archivage.is_finished()
        })
        .await
    );
    tx.commit().await.unwrap();
    let err = tokio::time::timeout(ATTENTE, archivage)
        .await
        .unwrap()
        .unwrap()
        .unwrap_err();
    assert_eq!(
        designation(err),
        (FxDesignation::Gain, Some(h), Some("1020".into()))
    );

    // (b) L'archivage commité d'abord : la désignation attend `companies` en
    //     exclusif, puis refuse en `archived`.
    let autre = compte(&pool, c, "6982", "Expense", None, None).await;
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM companies WHERE id = ? LOCK IN SHARE MODE")
        .bind(c)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(autre)
        .execute(&mut *tx)
        .await
        .unwrap();
    let (pool_b, s_b) = (pool.clone(), s.clone());
    let designation_propre = tokio::spawn(async move {
        modifier(&pool_b, &s_b, h, |u| {
            u.own_fx_accounts = OwnFxAccountsChange::Own(OwnFxAccounts {
                gain: autre,
                loss: autre,
            })
        })
        .await
    });
    assert!(
        attendre_une_requete_en_cours(&pool, &["FROM companies", "FOR UPDATE"], || {
            designation_propre.is_finished()
        })
        .await
    );
    tx.commit().await.unwrap();
    let err = tokio::time::timeout(ATTENTE, designation_propre)
        .await
        .unwrap()
        .unwrap()
        .unwrap_err();
    assert_eq!(
        fx_refus(err),
        (FxAccountField::Gain, FxAccountReason::Archived, Some(h))
    );
}

/// Deux archivages concurrents de comptes **différents** aboutissent tous deux,
/// sans 1213 (37-1-CR3-P1-1) : la transaction de test tient exactement ce que
/// tient l'archivage de A à sa fin (`companies` et réglages en partage, A
/// `FOR UPDATE`, les recherches propres de A par index, A archivé) ; l'archivage
/// de B passe pendant ce temps, sans attendre.
#[sqlx::test(migrations = "./test-schema")]
async fn deux_archivages_concurrents_de_comptes_differents(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    multi(&pool, &s).await;
    let c = s.company_id;
    let a = compte(&pool, c, "6983", "Expense", None, None).await;
    let b = compte(&pool, c, "6984", "Expense", None, None).await;
    // Des comptes en devise désignent d'autres comptes : les index ne sont pas vides.
    let autre = compte(&pool, c, "6985", "Expense", None, None).await;
    detenteur(&pool, &s, "1031", autre, autre).await;

    let mut tx = pool.begin().await.unwrap();
    for sql in [
        "SELECT id FROM companies WHERE id = ? LOCK IN SHARE MODE",
        "SELECT company_id FROM company_invoice_settings WHERE company_id = ? LOCK IN SHARE MODE",
    ] {
        sqlx::query(sql).bind(c).fetch_all(&mut *tx).await.unwrap();
    }
    sqlx::query("SELECT company_id FROM accounts WHERE id = ? FOR UPDATE")
        .bind(a)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    for index in ["fx_gain", "fx_loss"] {
        sqlx::query(&format!(
            "SELECT id, number FROM accounts FORCE INDEX (idx_accounts_{index}) \
             WHERE company_id = ? AND {index}_account_id = ? ORDER BY id LIMIT 1 LOCK IN SHARE MODE"
        ))
        .bind(c)
        .bind(a)
        .fetch_all(&mut *tx)
        .await
        .unwrap();
    }
    sqlx::query("UPDATE accounts SET active = FALSE, version = version + 1 WHERE id = ?")
        .bind(a)
        .execute(&mut *tx)
        .await
        .unwrap();

    let archivage_b = tokio::time::timeout(Duration::from_secs(10), archiver(&pool, &s, b)).await;
    let archivage_b = archivage_b.expect("l'archivage de B ne doit pas attendre celui de A");
    assert!(!archivage_b.unwrap().active);
    tx.commit().await.unwrap();
    assert!(!lire(&pool, a).await.active);
}
