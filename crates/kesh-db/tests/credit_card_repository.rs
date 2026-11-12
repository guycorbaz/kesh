//! Story 34-11a2 (#972) — la carte de crédit au dépôt : `PUT` d'une carte
//! (`update_for_company`, IBAN `NULL`, quatre chiffres écrits puis effacés),
//! retypage refusé d'un compte lié à un compte bancaire non archivé
//! (`accounts::update`, C-e341-31), et sérialisation du retypage contre la
//! création d'une carte (C-e341-40).
//!
//! Fixtures synthétiques seulement : aucun numéro de carte, les quatre chiffres
//! sont `1234` et `9876`.

use std::time::Duration;

use kesh_db::entities::account::{AccountType, AccountUpdate};
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{
    BankAccountKind, Language, NewBankAccount, NewCompany, NewUser, OrgType, Role,
};
use kesh_db::errors::DbError;
use kesh_db::repositories::bank_accounts::BankAccountPutFields;
use kesh_db::repositories::company_invoice_settings::ClaimAccounts;
use kesh_db::repositories::{accounts, bank_accounts, companies, users};
use sqlx::MySqlPool;

async fn company(pool: &MySqlPool) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: "Carte SA".into(),
            first_name: None,
            last_name: None,
            address_structured: StructuredAddress {
                street: "Rue Test".into(),
                building: "1".into(),
                postal_code: "1000".into(),
                city: "Lausanne".into(),
                country: "CH".into(),
            },
            ide_number: None,
            org_type: OrgType::Pme,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .unwrap()
    .id
}

async fn user(pool: &MySqlPool, company_id: i64) -> i64 {
    users::create(
        pool,
        NewUser {
            username: "carte_admin".into(),
            password_hash: "$argon2id$v=19$m=65536,t=3,p=4$abcdefgh$ijklmnop".into(),
            role: Role::Admin,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .unwrap()
    .id
}

async fn account(pool: &MySqlPool, company_id: i64, number: &str, t: AccountType) -> i64 {
    sqlx::query("INSERT INTO accounts (company_id, number, name, account_type) VALUES (?, ?, ?, ?)")
        .bind(company_id)
        .bind(number)
        .bind(format!("Compte {number}"))
        .bind(t)
        .execute(pool)
        .await
        .unwrap()
        .last_insert_id() as i64
}

const INSERT_CARD: &str = "INSERT INTO bank_accounts (company_id, bank_name, iban, account_kind, card_last4, is_primary, journal_account_id) \
     VALUES (?, 'Visa', NULL, 'credit_card', '1234', FALSE, ?)";

/// Une carte liée à `journal_account_id`, écrite directement (le dépôt n'a pas
/// de création de carte : c'est la route qui l'écrit).
async fn card(pool: &MySqlPool, company_id: i64, journal_account_id: Option<i64>) -> i64 {
    sqlx::query(INSERT_CARD)
        .bind(company_id)
        .bind(journal_account_id)
        .execute(pool)
        .await
        .unwrap()
        .last_insert_id() as i64
}

async fn bank(pool: &MySqlPool, company_id: i64, journal_account_id: i64) -> i64 {
    let id = bank_accounts::create(
        pool,
        NewBankAccount {
            company_id,
            bank_name: "UBS".into(),
            iban: "CH9300762011623852957".into(),
            qr_iban: None,
            is_primary: false,
        },
    )
    .await
    .unwrap()
    .id;
    sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
        .bind(journal_account_id)
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
    id
}

async fn retype(
    pool: &MySqlPool,
    account_id: i64,
    user_id: i64,
    to: AccountType,
) -> Result<kesh_db::entities::Account, DbError> {
    let current = accounts::find_by_id(pool, account_id)
        .await
        .unwrap()
        .unwrap();
    accounts::update(
        pool,
        account_id,
        current.version,
        user_id,
        AccountUpdate {
            name: current.name.clone(),
            account_type: to,
            role: None,
            postable: true,
        },
        true,
    )
    .await
}

/// Attend (dix secondes au plus) qu'une autre connexion de la base de test
/// exécute une requête dont le texte répond à `motif` (`LIKE`) — c'est-à-dire,
/// pour une requête qui devrait finir en microsecondes, qu'elle est **bloquée**
/// sur un verrou. Sonde `PROCESSLIST` (le privilège `PROCESS`, qu'exige
/// `INNODB_TRX`, manque à l'utilisateur de test) ; le texte montre les `?`, non
/// les valeurs.
async fn attendre_requete_en_cours(pool: &MySqlPool, motif: &str) {
    for _ in 0..200 {
        let n: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM information_schema.PROCESSLIST \
             WHERE DB = DATABASE() AND ID <> CONNECTION_ID() AND INFO LIKE ?",
        )
        .bind(motif)
        .fetch_one(pool)
        .await
        .unwrap();
        if n > 0 {
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("aucune requête en cours ne répond à {motif:?}");
}

async fn type_of(pool: &MySqlPool, account_id: i64) -> String {
    sqlx::query_scalar("SELECT account_type FROM accounts WHERE id = ?")
        .bind(account_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// T4 (M9, M9b) — le `PUT` d'une carte écrit `iban` `NULL` et les quatre
/// chiffres ; `card_last4: None` les efface (remplacement complet).
#[sqlx::test(migrations = "./test-schema")]
async fn put_d_une_carte_ecrit_puis_efface_les_quatre_chiffres(pool: MySqlPool) {
    let company_id = company(&pool).await;
    let passif = account(&pool, company_id, "2050", AccountType::Liability).await;
    let id = card(&pool, company_id, Some(passif)).await;

    // `NewBankAccount::iban` n'est plus lu au `PUT` : une chaîne vide ici
    // ferait rougir le `CHECK` si elle l'était (M9b).
    let new = NewBankAccount {
        company_id,
        bank_name: "Visa".into(),
        iban: String::new(),
        qr_iban: None,
        is_primary: false,
    };
    let mut tx = pool.begin().await.unwrap();
    let (after, _) = bank_accounts::update_for_company(
        &mut tx,
        company_id,
        id,
        &new,
        BankAccountPutFields {
            journal_account_id: Some(passif),
            description: Some("Personnel"),
            iban: None,
            card_last4: Some("9876"),
        },
        1,
        &ClaimAccounts::default(),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(after.iban, None);
    assert_eq!(after.card_last4.as_deref(), Some("9876"));
    assert_eq!(after.account_kind, BankAccountKind::CreditCard);

    let mut tx = pool.begin().await.unwrap();
    let (after, _) = bank_accounts::update_for_company(
        &mut tx,
        company_id,
        id,
        &new,
        BankAccountPutFields {
            journal_account_id: Some(passif),
            description: None,
            iban: None,
            card_last4: None,
        },
        after.version,
        &ClaimAccounts::default(),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(after.card_last4, None, "absents : effacés");
    let en_base: Option<String> =
        sqlx::query_scalar("SELECT card_last4 FROM bank_accounts WHERE id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(en_base, None);
}

/// AC-5 (M5) — retyper `Expense` le passif lié à une carte : `409`, rien
/// d'écrit ; même avec la confirmation du retypage.
#[sqlx::test(migrations = "./test-schema")]
async fn retypage_du_passif_d_une_carte_refuse(pool: MySqlPool) {
    let company_id = company(&pool).await;
    let user_id = user(&pool, company_id).await;
    let passif = account(&pool, company_id, "2050", AccountType::Liability).await;
    let carte = card(&pool, company_id, Some(passif)).await;

    let res = retype(&pool, passif, user_id, AccountType::Expense).await;
    match res {
        Err(DbError::AccountLinkedToBankAccount {
            bank_account_id, ..
        }) => assert_eq!(bank_account_id, carte),
        other => panic!("attendu AccountLinkedToBankAccount, obtenu {other:?}"),
    }
    assert_eq!(type_of(&pool, passif).await, "Liability", "rien d'écrit");
    // Un actif non plus : la carte n'admet qu'un passif.
    assert!(matches!(
        retype(&pool, passif, user_id, AccountType::Asset).await,
        Err(DbError::AccountLinkedToBankAccount { .. })
    ));
}

/// AC-5 — retyper `Revenue` l'actif lié à un compte bancaire : même refus ;
/// le retyper `Liability` reste permis (un compte bancaire admet un passif).
#[sqlx::test(migrations = "./test-schema")]
async fn retypage_de_l_actif_d_un_compte_bancaire(pool: MySqlPool) {
    let company_id = company(&pool).await;
    let user_id = user(&pool, company_id).await;
    let actif = account(&pool, company_id, "1020", AccountType::Asset).await;
    bank(&pool, company_id, actif).await;

    assert!(matches!(
        retype(&pool, actif, user_id, AccountType::Revenue).await,
        Err(DbError::AccountLinkedToBankAccount { .. })
    ));
    assert_eq!(type_of(&pool, actif).await, "Asset");
    retype(&pool, actif, user_id, AccountType::Liability)
        .await
        .expect("actif ou passif : admis");
}

/// AC-5 (M5b) — un compte lié à une carte **archivée** se retype ; un compte
/// non lié se retype comme avant.
#[sqlx::test(migrations = "./test-schema")]
async fn retypage_permis_si_le_lien_est_archive_ou_absent(pool: MySqlPool) {
    let company_id = company(&pool).await;
    let user_id = user(&pool, company_id).await;
    let passif = account(&pool, company_id, "2050", AccountType::Liability).await;
    let carte = card(&pool, company_id, Some(passif)).await;
    sqlx::query("UPDATE bank_accounts SET archived = TRUE WHERE id = ?")
        .bind(carte)
        .execute(&pool)
        .await
        .unwrap();
    retype(&pool, passif, user_id, AccountType::Expense)
        .await
        .expect("lien archivé : retypage permis");

    let libre = account(&pool, company_id, "2060", AccountType::Liability).await;
    retype(&pool, libre, user_id, AccountType::Expense)
        .await
        .expect("compte non lié : retypage permis");
}

/// Revue P1 (E7, M15) — une carte **close** compte (elle se rouvre) ; une carte
/// et un compte bancaire sur le même passif, retypé en actif : le compte
/// bancaire l'admet, la carte non — le refus nomme la carte.
#[sqlx::test(migrations = "./test-schema")]
async fn retypage_carte_close_et_lien_partage(pool: MySqlPool) {
    let company_id = company(&pool).await;
    let user_id = user(&pool, company_id).await;
    let passif = account(&pool, company_id, "2050", AccountType::Liability).await;
    let carte = card(&pool, company_id, Some(passif)).await;
    sqlx::query("UPDATE bank_accounts SET closed_on = '2026-06-30' WHERE id = ?")
        .bind(carte)
        .execute(&pool)
        .await
        .unwrap();
    assert!(matches!(
        retype(&pool, passif, user_id, AccountType::Expense).await,
        Err(DbError::AccountLinkedToBankAccount { bank_account_id, .. }) if bank_account_id == carte
    ));

    let passif2 = account(&pool, company_id, "2060", AccountType::Liability).await;
    bank(&pool, company_id, passif2).await;
    let carte2 = card(&pool, company_id, Some(passif2)).await;
    match retype(&pool, passif2, user_id, AccountType::Asset).await {
        Err(DbError::AccountLinkedToBankAccount {
            bank_account_id,
            bank_account_name,
        }) => {
            assert_eq!(bank_account_id, carte2, "la carte, non le compte bancaire");
            assert_eq!(bank_account_name, "Visa");
        }
        other => panic!("attendu AccountLinkedToBankAccount, obtenu {other:?}"),
    }
    assert_eq!(type_of(&pool, passif2).await, "Liability");
}

/// AC-5, concurrence (C-e341-40, M8) — une transaction « création de carte »
/// relit le type du compte `LOCK IN SHARE MODE` ; un retypage lancé pendant
/// qu'elle le tient **attend** la ligne `accounts` (`FOR UPDATE` en tête), puis
/// voit la carte commitée et refuse. Sans le `FOR UPDATE` de tête (M8), le
/// retypage lirait les comptes bancaires avant l'`INSERT` de la carte et
/// aboutirait — ou l'interblocage ferait échouer la carte : dans les deux cas,
/// ce test rougit.
#[sqlx::test(migrations = "./test-schema")]
async fn retypage_croise_avec_la_creation_d_une_carte(pool: MySqlPool) {
    let company_id = company(&pool).await;
    let user_id = user(&pool, company_id).await;
    let passif = account(&pool, company_id, "2050", AccountType::Liability).await;

    let mut tx = pool.begin().await.unwrap();
    let lu = accounts::type_in_share_mode(&mut tx, company_id, passif)
        .await
        .unwrap();
    assert_eq!(lu, Some((AccountType::Liability, true)));

    let pool_b = pool.clone();
    let retypage =
        tokio::spawn(async move { retype(&pool_b, passif, user_id, AccountType::Expense).await });
    // Revue P1 (E8) — l'attente se constate, elle ne se suppose pas : la
    // lecture `FOR UPDATE` de tête du retypage est vue en cours (bloquée) dans
    // `PROCESSLIST`. Sans elle (M8), la sonde ne la voit jamais et le test rougit.
    attendre_requete_en_cours(&pool, "%FROM accounts WHERE id = ? FOR UPDATE%").await;
    assert!(
        !retypage.is_finished(),
        "le retypage doit attendre la ligne"
    );

    sqlx::query(INSERT_CARD)
        .bind(company_id)
        .bind(passif)
        .execute(&mut *tx)
        .await
        .expect("la carte s'écrit");
    tx.commit().await.expect("la carte se commite");

    let res = retypage.await.unwrap();
    assert!(
        matches!(res, Err(DbError::AccountLinkedToBankAccount { .. })),
        "le retypage doit voir la carte commitée : {res:?}"
    );
    assert_eq!(type_of(&pool, passif).await, "Liability");
}
