//! Story 37-1 (AC-3, AC-4 bis) — le réglage multi-devises au dépôt :
//! `companies::update_currency_settings` (ordre des refus, no-op, irréversibilité,
//! activation des exercices ouverts, audit), le contrôle partagé des comptes de
//! différence de change (`accounts::check_fx_accounts_in_tx`, liste close des
//! raisons), la résolution unique (`accounts::fx_accounts_for`) et la
//! revalidation à l'usage (`accounts::validate_fx_accounts_in_tx`).

use kesh_db::entities::{CurrencySettingsUpdate, FxAccountsOrigin};
use kesh_db::errors::{DbError, FxAccountField, FxAccountReason};
use kesh_db::repositories::accounts::{self, FxAccountCheck, FxAccountLock};
use kesh_db::repositories::companies;
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use sqlx::MySqlPool;

/// Un compte écrit en SQL direct : type, actif, imputable, rôle, devise.
#[allow(clippy::too_many_arguments)]
async fn compte(
    pool: &MySqlPool,
    company_id: i64,
    numero: &str,
    type_: &str,
    actif: bool,
    imputable: bool,
    role: Option<&str>,
    devise: Option<&str>,
) -> i64 {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, active, postable, role, currency) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(numero)
    .bind(format!("Compte {numero}"))
    .bind(type_)
    .bind(actif)
    .bind(imputable)
    .bind(role)
    .bind(devise)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

/// Charge de change (6950) et compte de conversion (1090), valides.
async fn comptes_valides(pool: &MySqlPool, s: &SeededCompany) -> (i64, i64) {
    let charge = compte(
        pool,
        s.company_id,
        "6950",
        "Expense",
        true,
        true,
        None,
        None,
    )
    .await;
    let transit = compte(pool, s.company_id, "1090", "Asset", true, true, None, None).await;
    (charge, transit)
}

fn activer(gain: i64, perte: i64, transit: i64) -> CurrencySettingsUpdate {
    CurrencySettingsUpdate {
        multi_currency_enabled: true,
        fx_gain_account_id: Some(gain),
        fx_loss_account_id: Some(perte),
        fx_transit_account_id: Some(transit),
    }
}

async fn version(pool: &MySqlPool, company_id: i64) -> i32 {
    companies::find_by_id(pool, company_id)
        .await
        .unwrap()
        .unwrap()
        .version
}

async fn audits(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE action = 'company.currency_settings_updated'",
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

fn refus(err: DbError) -> (FxAccountField, FxAccountReason, Option<i64>) {
    match err {
        DbError::FxAccountInvalid {
            field,
            reason,
            holder_account_id,
        } => (field, reason, holder_account_id),
        other => panic!("FxAccountInvalid attendu, reçu {other:?}"),
    }
}

/// Le décodage de `accounts.currency` (collation `ascii_general_ci`, choix
/// C-37-1-1) marche sans `CAST` : `find_by_id` relit la devise.
#[sqlx::test(migrations = "./test-schema")]
async fn la_devise_d_un_compte_se_relit_sans_cast(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let id = compte(
        &pool,
        s.company_id,
        "1025",
        "Asset",
        true,
        true,
        None,
        Some("USD"),
    )
    .await;
    let lu = accounts::find_by_id(&pool, id).await.unwrap().unwrap();
    assert_eq!(lu.currency.as_deref(), Some("USD"));
    // La casse reste refusée par le schéma (CHECK … BINARY).
    let minuscule = sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, currency) \
         VALUES (?, '1026', 'x', 'Asset', 'usd')",
    )
    .bind(s.company_id)
    .execute(&pool)
    .await;
    assert!(minuscule.is_err(), "« usd » doit être refusé par le schéma");
}

#[sqlx::test(migrations = "./test-schema")]
async fn fx_accounts_for_resout_les_comptes_propres_puis_ceux_de_la_societe(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let (charge, transit) = comptes_valides(&pool, &s).await;
    let produit = compte(
        &pool,
        s.company_id,
        "6951",
        "Revenue",
        true,
        true,
        None,
        None,
    )
    .await;
    let usd = compte(
        &pool,
        s.company_id,
        "1025",
        "Asset",
        true,
        true,
        None,
        Some("USD"),
    )
    .await;
    let eur_propre = compte(
        &pool,
        s.company_id,
        "1026",
        "Asset",
        true,
        true,
        None,
        Some("EUR"),
    )
    .await;
    sqlx::query("UPDATE accounts SET fx_gain_account_id = ?, fx_loss_account_id = ? WHERE id = ?")
        .bind(produit)
        .bind(produit)
        .bind(eur_propre)
        .execute(&pool)
        .await
        .unwrap();
    let chf = s.accounts["1100"];
    let lire = |id| accounts::find_by_id(&pool, id);

    // Société sans comptes de différence (état hérité) : rien pour un compte par défaut.
    let societe = companies::find_by_id(&pool, s.company_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        accounts::fx_accounts_for(&societe, &lire(usd).await.unwrap().unwrap()),
        None
    );

    let societe = companies::update_currency_settings(
        &pool,
        s.admin_user_id,
        s.company_id,
        societe.version,
        activer(charge, charge, transit),
    )
    .await
    .unwrap();
    let defaut = accounts::fx_accounts_for(&societe, &lire(usd).await.unwrap().unwrap()).unwrap();
    assert_eq!(
        (defaut.gain, defaut.loss, defaut.origin),
        (charge, charge, FxAccountsOrigin::Company)
    );
    assert_eq!(defaut.holder_account_id, usd);

    let propres =
        accounts::fx_accounts_for(&societe, &lire(eur_propre).await.unwrap().unwrap()).unwrap();
    assert_eq!(
        (propres.gain, propres.loss, propres.origin),
        (produit, produit, FxAccountsOrigin::Own)
    );

    // Compte CHF : jamais de comptes de différence.
    assert_eq!(
        accounts::fx_accounts_for(&societe, &lire(chf).await.unwrap().unwrap()),
        None
    );

    // Le défaut suit le réglage quand il change.
    let societe = companies::update_currency_settings(
        &pool,
        s.admin_user_id,
        s.company_id,
        societe.version,
        activer(produit, charge, transit),
    )
    .await
    .unwrap();
    let suivi = accounts::fx_accounts_for(&societe, &lire(usd).await.unwrap().unwrap()).unwrap();
    assert_eq!((suivi.gain, suivi.loss), (produit, charge));
}

/// Chaque raison de la liste close (hors `missing` et `unexpected`, de forme),
/// par le contrôle partagé, pour les gains et pour la conversion.
#[sqlx::test(migrations = "./test-schema")]
async fn le_controle_partage_rend_chaque_raison(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let c = s.company_id;
    let autre_societe: i64 = sqlx::query(
        "INSERT INTO companies (name, address, address_street, address_building, \
             address_postal_code, address_city, address_country, org_type, \
             accounting_language, instance_language) \
         VALUES ('Autre SA', 'Rue 1\n1000 Lausanne', 'Rue', '1', '1000', 'Lausanne', 'CH', \
             'Independant', 'FR', 'FR')",
    )
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let charge_autre = compte(
        &pool,
        autre_societe,
        "6950",
        "Expense",
        true,
        true,
        None,
        None,
    )
    .await;
    let (charge, transit) = comptes_valides(&pool, &s).await;
    let archive = compte(&pool, c, "6952", "Expense", false, true, None, None).await;
    let titre = compte(&pool, c, "6953", "Expense", true, false, None, None).await;
    let actif = compte(&pool, c, "1027", "Asset", true, true, None, None).await;
    let en_devise = compte(&pool, c, "6954", "Expense", true, true, None, Some("USD")).await;
    let role_ca = compte(
        &pool,
        c,
        "3200",
        "Revenue",
        true,
        true,
        Some("DefaultRevenue"),
        None,
    )
    .await;
    let produit_article = compte(&pool, c, "3300", "Revenue", true, true, None, None).await;
    sqlx::query(
        "INSERT INTO products (company_id, name, unit_price, vat_rate, default_revenue_account_id) \
         VALUES (?, 'Article', 10, 8.1, ?)",
    )
    .bind(c)
    .bind(produit_article)
    .execute(&pool)
    .await
    .unwrap();
    // 3000 est désigné compte de produit par défaut des réglages de facturation.
    let produit_reglages = s.accounts["3000"];
    // 2000 est désigné TVA due : une autre désignation reste admise pour la conversion.
    let tva_due = s.accounts["2000"];

    let cas: Vec<(FxAccountField, i64, Option<FxAccountReason>)> = vec![
        (FxAccountField::Gain, charge, None),
        (
            FxAccountField::Gain,
            i64::MAX,
            Some(FxAccountReason::Unknown),
        ),
        (
            FxAccountField::Gain,
            charge_autre,
            Some(FxAccountReason::Unknown),
        ),
        (
            FxAccountField::Gain,
            archive,
            Some(FxAccountReason::Archived),
        ),
        (
            FxAccountField::Gain,
            titre,
            Some(FxAccountReason::NotPostable),
        ),
        (
            FxAccountField::Gain,
            actif,
            Some(FxAccountReason::AccountType),
        ),
        (
            FxAccountField::Gain,
            en_devise,
            Some(FxAccountReason::ForeignCurrency),
        ),
        (
            FxAccountField::Gain,
            role_ca,
            Some(FxAccountReason::RevenueAccount),
        ),
        (
            FxAccountField::Loss,
            produit_reglages,
            Some(FxAccountReason::RevenueAccount),
        ),
        (
            FxAccountField::Loss,
            produit_article,
            Some(FxAccountReason::RevenueAccount),
        ),
        (FxAccountField::Transit, transit, None),
        (FxAccountField::Transit, tva_due, None),
        (
            FxAccountField::Transit,
            charge,
            Some(FxAccountReason::AccountType),
        ),
        (
            FxAccountField::Transit,
            produit_reglages,
            Some(FxAccountReason::AccountType),
        ),
    ];
    for (field, id, attendu) in cas {
        let mut conn = pool.acquire().await.unwrap();
        let r = accounts::check_fx_accounts_in_tx(
            &mut conn,
            c,
            &[FxAccountCheck {
                field,
                account_id: id,
                holder_account_id: None,
            }],
            FxAccountLock::ForUpdate,
        )
        .await;
        match attendu {
            None => assert!(r.is_ok(), "{field:?} {id} devait passer : {r:?}"),
            Some(reason) => {
                assert_eq!(
                    refus(r.unwrap_err()),
                    (field, reason, None),
                    "{field:?} {id}"
                )
            }
        }
    }
}

/// Revalidation à l'usage : `holderAccountId` quand les comptes sont propres,
/// aucun quand ils viennent de la société ; deux comptes valides passent.
#[sqlx::test(migrations = "./test-schema")]
async fn la_revalidation_nomme_le_compte_porteur_des_comptes_propres(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let (charge, transit) = comptes_valides(&pool, &s).await;
    let usd = compte(
        &pool,
        s.company_id,
        "1025",
        "Asset",
        true,
        true,
        None,
        Some("USD"),
    )
    .await;
    let societe = companies::find_by_id(&pool, s.company_id)
        .await
        .unwrap()
        .unwrap();
    let societe = companies::update_currency_settings(
        &pool,
        s.admin_user_id,
        s.company_id,
        societe.version,
        activer(charge, charge, transit),
    )
    .await
    .unwrap();
    let compte_usd = accounts::find_by_id(&pool, usd).await.unwrap().unwrap();
    let mut fx = accounts::fx_accounts_for(&societe, &compte_usd).unwrap();
    let mut tx = pool.begin().await.unwrap();
    accounts::validate_fx_accounts_in_tx(&mut tx, s.company_id, &fx)
        .await
        .unwrap();
    tx.rollback().await.unwrap();

    // Comptes de la société devenus invalides (archivés) : pas de porteur.
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(charge)
        .execute(&pool)
        .await
        .unwrap();
    let mut tx = pool.begin().await.unwrap();
    let e = accounts::validate_fx_accounts_in_tx(&mut tx, s.company_id, &fx)
        .await
        .unwrap_err();
    assert_eq!(
        refus(e),
        (FxAccountField::Gain, FxAccountReason::Archived, None)
    );

    // Comptes propres : la perte est un compte d'actif.
    fx.origin = FxAccountsOrigin::Own;
    fx.gain = transit;
    fx.loss = transit;
    let mut tx = pool.begin().await.unwrap();
    let e = accounts::validate_fx_accounts_in_tx(&mut tx, s.company_id, &fx)
        .await
        .unwrap_err();
    assert_eq!(
        refus(e),
        (
            FxAccountField::Gain,
            FxAccountReason::AccountType,
            Some(usd)
        )
    );
}

/// Activation : les exercices **ouverts** passent en multi-devises, les clos non ;
/// leurs identifiants vont au détail d'audit, avec l'avant et l'après.
#[sqlx::test(migrations = "./test-schema")]
async fn l_activation_passe_les_exercices_ouverts_en_multi_devises(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let (charge, transit) = comptes_valides(&pool, &s).await;
    let clos: i64 = sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, 'Exercice 2019', '2019-01-01', '2019-12-31', 'Closed')",
    )
    .bind(s.company_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let v = version(&pool, s.company_id).await;

    let apres = companies::update_currency_settings(
        &pool,
        s.admin_user_id,
        s.company_id,
        v,
        activer(charge, charge, transit),
    )
    .await
    .unwrap();
    assert!(apres.multi_currency_enabled);
    assert_eq!(apres.version, v + 1);
    assert_eq!(apres.fx_transit_account_id, Some(transit));

    let modes: Vec<(i64, bool)> = sqlx::query_as(
        "SELECT id, multi_currency FROM fiscal_years WHERE company_id = ? ORDER BY id",
    )
    .bind(s.company_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(modes, vec![(s.fiscal_year_id, true), (clos, false)]);

    let details: Vec<u8> = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'company.currency_settings_updated'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let details: serde_json::Value = serde_json::from_slice(&details).unwrap();
    assert_eq!(
        details["fiscalYearIds"],
        serde_json::json!([s.fiscal_year_id])
    );
    assert_eq!(details["before"]["multiCurrencyEnabled"], false);
    assert_eq!(details["after"]["multiCurrencyEnabled"], true);
    assert_eq!(details["after"]["fxTransitAccountId"], transit);
}

/// Ordre des refus : forme (avant tout), version, no-op, irréversible, comptes.
#[sqlx::test(migrations = "./test-schema")]
async fn l_ordre_des_refus_et_le_no_op(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let c = s.company_id;
    let (charge, transit) = comptes_valides(&pool, &s).await;
    let v = version(&pool, c).await;

    // Forme : activer sans conversion → missing ; désactivé avec un compte → unexpected.
    let mut sans_transit = activer(charge, charge, transit);
    sans_transit.fx_transit_account_id = None;
    let e = companies::update_currency_settings(&pool, s.admin_user_id, c, v, sans_transit)
        .await
        .unwrap_err();
    assert_eq!(
        refus(e),
        (FxAccountField::Transit, FxAccountReason::Missing, None)
    );
    let inattendu = CurrencySettingsUpdate {
        multi_currency_enabled: false,
        fx_gain_account_id: None,
        fx_loss_account_id: Some(charge),
        fx_transit_account_id: None,
    };
    // La forme passe avant la version (périmée ici).
    let e = companies::update_currency_settings(&pool, s.admin_user_id, c, v + 7, inattendu)
        .await
        .unwrap_err();
    assert_eq!(
        refus(e),
        (FxAccountField::Loss, FxAccountReason::Unexpected, None)
    );

    // Version périmée, avant le contrôle des comptes (compte inconnu ici).
    let e = companies::update_currency_settings(
        &pool,
        s.admin_user_id,
        c,
        v + 1,
        activer(i64::MAX, charge, transit),
    )
    .await
    .unwrap_err();
    assert!(matches!(e, DbError::OptimisticLockConflict), "{e:?}");

    // No-op en mono-devise : rien d'écrit.
    let mono = CurrencySettingsUpdate {
        multi_currency_enabled: false,
        fx_gain_account_id: None,
        fx_loss_account_id: None,
        fx_transit_account_id: None,
    };
    let inchangee = companies::update_currency_settings(&pool, s.admin_user_id, c, v, mono)
        .await
        .unwrap();
    assert_eq!(inchangee.version, v);
    assert_eq!(audits(&pool).await, 0);

    // Compte refusé : gain d'abord.
    let e = companies::update_currency_settings(
        &pool,
        s.admin_user_id,
        c,
        v,
        activer(transit, transit, charge),
    )
    .await
    .unwrap_err();
    assert_eq!(
        refus(e),
        (FxAccountField::Gain, FxAccountReason::AccountType, None)
    );
    assert_eq!(version(&pool, c).await, v, "un refus n'écrit rien");

    // Activation, puis le même corps : no-op, sans bump ni audit.
    let active = companies::update_currency_settings(
        &pool,
        s.admin_user_id,
        c,
        v,
        activer(charge, charge, transit),
    )
    .await
    .unwrap();
    assert_eq!(audits(&pool).await, 1);
    let meme = companies::update_currency_settings(
        &pool,
        s.admin_user_id,
        c,
        active.version,
        activer(charge, charge, transit),
    )
    .await
    .unwrap();
    assert_eq!(meme.version, active.version);
    assert_eq!(audits(&pool).await, 1);

    // Désactiver : irréversible (le no-op passe avant, la désactivation ensuite).
    let e = companies::update_currency_settings(&pool, s.admin_user_id, c, active.version, mono)
        .await
        .unwrap_err();
    assert!(matches!(e, DbError::MultiCurrencyIrreversible), "{e:?}");
    let relue = companies::find_by_id(&pool, c).await.unwrap().unwrap();
    assert!(relue.multi_currency_enabled);
    assert_eq!(relue.version, active.version);
}

/// Motif C4 : un compte inchangé devenu invalide n'empêche pas d'enregistrer
/// l'autre ; changé, il est refusé.
#[sqlx::test(migrations = "./test-schema")]
async fn un_compte_inchange_devenu_invalide_n_empeche_pas_d_enregistrer_l_autre(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let c = s.company_id;
    let (charge, transit) = comptes_valides(&pool, &s).await;
    let autre_charge = compte(&pool, c, "6955", "Expense", true, true, None, None).await;
    let active = companies::update_currency_settings(
        &pool,
        s.admin_user_id,
        c,
        version(&pool, c).await,
        activer(charge, charge, transit),
    )
    .await
    .unwrap();
    // La conversion est archivée après coup.
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(transit)
        .execute(&pool)
        .await
        .unwrap();
    let apres = companies::update_currency_settings(
        &pool,
        s.admin_user_id,
        c,
        active.version,
        activer(charge, autre_charge, transit),
    )
    .await
    .unwrap();
    assert_eq!(apres.fx_loss_account_id, Some(autre_charge));

    // Désigner un autre compte archivé comme conversion : refusé.
    let archive = compte(&pool, c, "1091", "Asset", false, true, None, None).await;
    let e = companies::update_currency_settings(
        &pool,
        s.admin_user_id,
        c,
        apres.version,
        activer(charge, autre_charge, archive),
    )
    .await
    .unwrap_err();
    assert_eq!(
        refus(e),
        (FxAccountField::Transit, FxAccountReason::Archived, None)
    );
}

/// La liste close des raisons et leurs codes d'API ne bougent pas sans que ce
/// test rougisse (reprise par la 37-5).
#[test]
fn la_liste_close_des_raisons() {
    let codes: Vec<&str> = FxAccountReason::ALL.iter().map(|r| r.as_str()).collect();
    assert_eq!(
        codes,
        [
            "missing",
            "unexpected",
            "unknown",
            "archived",
            "notPostable",
            "accountType",
            "foreignCurrency",
            "revenueAccount"
        ]
    );
    assert_eq!(FxAccountField::Gain.as_api_field(), "fxGainAccountId");
    assert_eq!(FxAccountField::Loss.as_api_field(), "fxLossAccountId");
    assert_eq!(FxAccountField::Transit.as_api_field(), "fxTransitAccountId");
}
