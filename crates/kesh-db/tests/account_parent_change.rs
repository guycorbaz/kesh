//! Correctif `fix-api-parent-compte` — le parent d'un compte se change par
//! `accounts::update` (`AccountUpdate::parent`, trois états) : les contrôles
//! de la création, le refus d'un cycle, la postabilité du nouveau parent,
//! l'audit, et la course qui fermerait un cycle par un ancêtre.

use std::time::Duration;

use kesh_db::entities::account::{
    Account, AccountType, AccountUpdate, CurrencyChange, NewAccount, OwnFxAccounts,
    OwnFxAccountsChange, ParentChange,
};
use kesh_db::errors::{DbError, FxAccountField, FxAccountReason};
use kesh_db::repositories::accounts;
use kesh_db::test_fixtures::{
    SeededCompany, attendre_une_requete_en_cours, seed_accounting_company,
};
use sqlx::MySqlPool;

const ATTENTE: Duration = Duration::from_secs(20);

/// Un compte créé par le dépôt, racine ou sous `parent`.
async fn compte(pool: &MySqlPool, s: &SeededCompany, numero: &str, parent: Option<i64>) -> i64 {
    accounts::create(
        pool,
        s.admin_user_id,
        NewAccount::new(
            s.company_id,
            numero,
            format!("Compte {numero}"),
            AccountType::Asset,
            parent,
        ),
    )
    .await
    .unwrap()
    .id
}

async fn lire(pool: &MySqlPool, id: i64) -> Account {
    accounts::find_by_id(pool, id).await.unwrap().unwrap()
}

/// Le `PUT` qui renvoie le compte tel quel, avec le changement de parent donné.
async fn ranger(
    pool: &MySqlPool,
    s: &SeededCompany,
    id: i64,
    parent: ParentChange,
) -> Result<Account, DbError> {
    let a = lire(pool, id).await;
    accounts::update(
        pool,
        id,
        a.version,
        s.admin_user_id,
        AccountUpdate {
            name: a.name.clone(),
            account_type: a.account_type,
            role: a.role,
            postable: a.postable,
            currency: CurrencyChange::Unchanged,
            own_fx_accounts: OwnFxAccountsChange::Unchanged,
            parent,
        },
        false,
    )
    .await
}

/// Une écriture CHF sur `compte` contre la caisse, en SQL direct.
async fn ligne(pool: &MySqlPool, s: &SeededCompany, compte: i64) {
    let numero: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(entry_number), 0) + 1 FROM journal_entries WHERE company_id = ?",
    )
    .bind(s.company_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let je: i64 = sqlx::query_scalar(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, journal, description) \
         VALUES (?, ?, ?, '2026-06-01', 'Banque', 'test') RETURNING id",
    )
    .bind(s.company_id)
    .bind(s.fiscal_year_id)
    .bind(numero)
    .fetch_one(pool)
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
        .execute(pool)
        .await
        .unwrap();
    }
}

/// Le nominal : un compte mouvementé se range sous un groupe (qui devient non
/// imputable), le `PUT` est audité avec le parent avant et après, puis le
/// compte redevient racine ; l'ancien parent garde son état ; renvoyer le
/// parent en place est un no-op.
#[sqlx::test(migrations = "./test-schema")]
async fn ranger_puis_detacher_un_compte_mouvemente(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let banque = s.accounts["1100"];
    ligne(&pool, &s, banque).await;
    let groupe = compte(&pool, &s, "10", None).await;
    assert!(lire(&pool, groupe).await.postable);
    let avant = lire(&pool, banque).await;

    let apres = ranger(&pool, &s, banque, ParentChange::Under(groupe))
        .await
        .unwrap();
    assert_eq!(apres.parent_id, Some(groupe));
    assert_eq!(apres.version, avant.version + 1);
    assert_eq!(
        apres.postable, avant.postable,
        "le compte déplacé garde sa postabilité"
    );
    let g = lire(&pool, groupe).await;
    assert!(
        !g.postable,
        "le nouveau parent devient un compte de regroupement"
    );
    assert_eq!(g.version, 1, "sans bump de version, comme à la création");

    let details: String = sqlx::query_scalar(
        "SELECT CAST(details_json AS CHAR) FROM audit_log \
         WHERE action = 'account.updated' AND entity_id = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(banque)
    .fetch_one(&pool)
    .await
    .unwrap();
    let details: serde_json::Value = serde_json::from_str(&details).unwrap();
    assert_eq!(details["before"]["parentId"], serde_json::Value::Null);
    assert_eq!(details["after"]["parentId"], groupe);

    // Le parent en place : no-op, pas de version, pas d'audit.
    let meme = ranger(&pool, &s, banque, ParentChange::Under(groupe))
        .await
        .unwrap();
    assert_eq!(meme.version, apres.version);
    let inchange = ranger(&pool, &s, banque, ParentChange::Unchanged)
        .await
        .unwrap();
    assert_eq!(inchange.parent_id, Some(groupe));
    assert_eq!(inchange.version, apres.version);

    // Racine ; l'ancien parent reste non imputable (comme après l'archivage
    // de son dernier sous-compte).
    let racine = ranger(&pool, &s, banque, ParentChange::Root).await.unwrap();
    assert_eq!(racine.parent_id, None);
    assert!(!lire(&pool, groupe).await.postable);

    // Les écritures du compte n'ont pas bougé.
    let lignes: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM journal_entry_lines WHERE account_id = ?")
            .bind(banque)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(lignes, 1);
}

/// Le cycle : le compte lui-même, son enfant, son petit-enfant ; rien n'est
/// écrit.
#[sqlx::test(migrations = "./test-schema")]
async fn un_compte_ne_se_range_pas_sous_lui_meme(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let a = compte(&pool, &s, "10", None).await;
    let b = compte(&pool, &s, "100", Some(a)).await;
    let c = compte(&pool, &s, "1001", Some(b)).await;
    for (cible, numero) in [(a, "10"), (b, "100"), (c, "1001")] {
        match ranger(&pool, &s, a, ParentChange::Under(cible)).await {
            Err(DbError::AccountParentCycle { parent_number }) => {
                assert_eq!(parent_number, numero)
            }
            autre => panic!("cycle attendu sous {numero}, obtenu {autre:?}"),
        }
    }
    assert_eq!(lire(&pool, a).await.parent_id, None);
    assert_eq!(lire(&pool, a).await.version, 1);
    // Un déplacement voisin, sans cycle, passe : le petit-enfant remonte sous
    // la racine.
    let c2 = ranger(&pool, &s, c, ParentChange::Under(a)).await.unwrap();
    assert_eq!(c2.parent_id, Some(a));
}

/// Les contrôles de la création : parent d'une autre société, archivé.
#[sqlx::test(migrations = "./test-schema")]
async fn parent_etranger_ou_archive_refuse(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let a = compte(&pool, &s, "10", None).await;
    let autre: i64 = sqlx::query_scalar(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre SA', 'Rue 1\n1000 Lausanne', 'Independant', 'FR', 'FR') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let etranger: i64 = sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type) \
         VALUES (?, '20', 'Étranger', 'Asset') RETURNING id",
    )
    .bind(autre)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(matches!(
        ranger(&pool, &s, a, ParentChange::Under(etranger)).await,
        Err(DbError::AccountParentNotFound)
    ));

    let archive = compte(&pool, &s, "19", None).await;
    accounts::archive(&pool, archive, 1, s.admin_user_id)
        .await
        .unwrap();
    match ranger(&pool, &s, a, ParentChange::Under(archive)).await {
        Err(DbError::AccountParentArchived { parent_number }) => assert_eq!(parent_number, "19"),
        autre => panic!("parent archivé attendu, obtenu {autre:?}"),
    }
    assert_eq!(lire(&pool, a).await.parent_id, None);
}

/// Les contrôles de devise de la création : un compte de différence de change
/// de la société ne reçoit pas de sous-compte ; un compte en devise mouvementé
/// non plus ; et un compte ne se range pas sous l'un de ses comptes propres.
#[sqlx::test(migrations = "./test-schema")]
async fn parent_de_change_ou_en_devise_mouvemente_refuse(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let gain = compte(&pool, &s, "6950", None).await;
    let perte = compte(&pool, &s, "6960", None).await;
    let transit = compte(&pool, &s, "1090", None).await;
    sqlx::query("UPDATE accounts SET account_type = 'Revenue' WHERE id = ?")
        .bind(gain)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE accounts SET account_type = 'Expense' WHERE id = ?")
        .bind(perte)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "UPDATE companies SET multi_currency_enabled = TRUE, fx_gain_account_id = ?, \
         fx_loss_account_id = ?, fx_transit_account_id = ? WHERE id = ?",
    )
    .bind(gain)
    .bind(perte)
    .bind(transit)
    .bind(s.company_id)
    .execute(&pool)
    .await
    .unwrap();
    let a = compte(&pool, &s, "1050", None).await;

    for designe in [gain, perte, transit] {
        assert!(
            matches!(
                ranger(&pool, &s, a, ParentChange::Under(designe)).await,
                Err(DbError::AccountIsFxDifferenceAccount { .. })
            ),
            "compte de différence {designe}"
        );
    }

    // Un compte d'actif en devise qui porte une ligne.
    let usd = compte(&pool, &s, "1021", None).await;
    ligne(&pool, &s, usd).await;
    sqlx::query("UPDATE accounts SET currency = 'USD' WHERE id = ?")
        .bind(usd)
        .execute(&pool)
        .await
        .unwrap();
    assert!(matches!(
        ranger(&pool, &s, a, ParentChange::Under(usd)).await,
        Err(DbError::AccountForeignParentHasEntries { .. })
    ));

    // Un compte en devise dont les comptes propres sont `g` et `p` ne se range
    // pas sous `g` : `g` deviendrait non imputable.
    let g = compte(&pool, &s, "6951", None).await;
    let p = compte(&pool, &s, "6961", None).await;
    sqlx::query("UPDATE accounts SET account_type = 'Revenue' WHERE id = ?")
        .bind(g)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE accounts SET account_type = 'Expense' WHERE id = ?")
        .bind(p)
        .execute(&pool)
        .await
        .unwrap();
    let eur = lire(&pool, compte(&pool, &s, "1022", None).await).await;
    let eur = accounts::update(
        &pool,
        eur.id,
        eur.version,
        s.admin_user_id,
        AccountUpdate {
            name: eur.name.clone(),
            account_type: eur.account_type,
            role: None,
            postable: true,
            currency: CurrencyChange::Foreign("EUR".into()),
            own_fx_accounts: OwnFxAccountsChange::Own(OwnFxAccounts { gain: g, loss: p }),
            parent: ParentChange::Unchanged,
        },
        false,
    )
    .await
    .unwrap();
    // Sous l'un de ses comptes propres qu'il garde : `g` deviendrait non
    // imputable — jugé sur l'état après le `PUT` (revue P1-4).
    match ranger(&pool, &s, eur.id, ParentChange::Under(g)).await {
        Err(DbError::FxAccountInvalid {
            field: FxAccountField::Gain,
            reason: FxAccountReason::NotPostable,
            holder_account_id,
        }) => assert_eq!(holder_account_id, Some(eur.id)),
        autre => panic!("FX_ACCOUNT_INVALID notPostable attendu, obtenu {autre:?}"),
    }
    // Sous l'un des comptes propres que le même `PUT` lui donne : `g2`, encore
    // désigné par personne, deviendrait non imputable.
    let g2 = compte(&pool, &s, "6952", None).await;
    sqlx::query("UPDATE accounts SET account_type = 'Revenue' WHERE id = ?")
        .bind(g2)
        .execute(&pool)
        .await
        .unwrap();
    let res = accounts::update(
        &pool,
        eur.id,
        eur.version,
        s.admin_user_id,
        AccountUpdate {
            name: eur.name.clone(),
            account_type: eur.account_type,
            role: None,
            postable: true,
            currency: CurrencyChange::Unchanged,
            own_fx_accounts: OwnFxAccountsChange::Own(OwnFxAccounts { gain: g2, loss: p }),
            parent: ParentChange::Under(g2),
        },
        false,
    )
    .await;
    match res {
        Err(DbError::FxAccountInvalid {
            field: FxAccountField::Gain,
            reason: FxAccountReason::NotPostable,
            holder_account_id,
        }) => assert_eq!(holder_account_id, Some(eur.id)),
        autre => panic!("FX_ACCOUNT_INVALID notPostable attendu, obtenu {autre:?}"),
    }
    assert_eq!(lire(&pool, a).await.parent_id, None);
    assert_eq!(lire(&pool, eur.id).await.parent_id, None);
}

/// Revue P1-4 : le même `PUT` qui retire à `eur` son compte propre `g` peut le
/// ranger sous `g` — après le geste, `g` n'est plus désigné par personne. Mais
/// un compte que d'**autres** comptes désignent reste refusé.
#[sqlx::test(migrations = "./test-schema")]
async fn sous_son_ancien_compte_propre_retire_dans_le_meme_put(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    sqlx::query("UPDATE companies SET multi_currency_enabled = TRUE WHERE id = ?")
        .bind(s.company_id)
        .execute(&pool)
        .await
        .unwrap();
    let mut ids = Vec::new();
    for (numero, type_) in [
        ("6951", "Revenue"),
        ("6961", "Expense"),
        ("6952", "Revenue"),
    ] {
        let id = compte(&pool, &s, numero, None).await;
        sqlx::query("UPDATE accounts SET account_type = ? WHERE id = ?")
            .bind(type_)
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
        ids.push(id);
    }
    let (g, p, g2) = (ids[0], ids[1], ids[2]);

    // Le `PUT` d'un compte : devise, comptes propres `(gain, p)`, parent.
    async fn poser(
        pool: &MySqlPool,
        s: &SeededCompany,
        id: i64,
        currency: CurrencyChange,
        gain: i64,
        loss: i64,
        parent: ParentChange,
    ) -> Result<Account, DbError> {
        let a = lire(pool, id).await;
        accounts::update(
            pool,
            id,
            a.version,
            s.admin_user_id,
            AccountUpdate {
                name: a.name.clone(),
                account_type: a.account_type,
                role: None,
                postable: true,
                currency,
                own_fx_accounts: OwnFxAccountsChange::Own(OwnFxAccounts { gain, loss }),
                parent,
            },
            false,
        )
        .await
    }
    let eur_ = CurrencyChange::Foreign("EUR".into());
    let mut en_devise = Vec::new();
    for (numero, gain) in [("1022", g), ("1023", g2), ("1024", g2)] {
        let id = compte(&pool, &s, numero, None).await;
        poser(
            &pool,
            &s,
            id,
            eur_.clone(),
            gain,
            p,
            ParentChange::Unchanged,
        )
        .await
        .unwrap();
        en_devise.push(id);
    }
    let (eur, usd, eur2) = (en_devise[0], en_devise[1], en_devise[2]);
    let _ = usd;

    // Seul `eur` désigne `g`, et ce `PUT` le lui retire : accepté.
    let apres = poser(
        &pool,
        &s,
        eur,
        CurrencyChange::Unchanged,
        g2,
        p,
        ParentChange::Under(g),
    )
    .await
    .unwrap();
    assert_eq!(apres.parent_id, Some(g));
    assert_eq!(apres.fx_gain_account_id, Some(g2));
    assert!(!lire(&pool, g).await.postable);

    // `g2` est désigné par `usd` et `eur` : `eur2` ne se range pas dessous.
    match poser(
        &pool,
        &s,
        eur2,
        CurrencyChange::Unchanged,
        g2,
        p,
        ParentChange::Under(g2),
    )
    .await
    {
        Err(DbError::AccountIsFxDifferenceAccount { .. }) => {}
        autre => panic!("ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT attendu, obtenu {autre:?}"),
    }
    assert_eq!(lire(&pool, eur2).await.parent_id, None);
}

/// Revue P1-2 (décision de Beat) : un parent qui porte des écritures, même en
/// CHF, est refusé ; le même compte, sans écriture, est accepté.
#[sqlx::test(migrations = "./test-schema")]
async fn parent_mouvemente_refuse(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let a = compte(&pool, &s, "1050", None).await;
    let vierge = compte(&pool, &s, "10", None).await;
    let mouvemente = compte(&pool, &s, "11", None).await;
    ligne(&pool, &s, mouvemente).await;
    match ranger(&pool, &s, a, ParentChange::Under(mouvemente)).await {
        Err(DbError::AccountParentHasEntries {
            parent_account_id,
            parent_number,
        }) => {
            assert_eq!(parent_account_id, mouvemente);
            assert_eq!(parent_number, "11");
        }
        autre => panic!("ACCOUNT_PARENT_HAS_ENTRIES attendu, obtenu {autre:?}"),
    }
    let m = lire(&pool, mouvemente).await;
    assert!(m.postable, "le parent refusé reste imputable");
    assert_eq!(lire(&pool, a).await.parent_id, None);
    // La sonde : le parent sans écriture passe.
    let ok = ranger(&pool, &s, a, ParentChange::Under(vierge))
        .await
        .unwrap();
    assert_eq!(ok.parent_id, Some(vierge));
}

/// La course : `D` est sous `B`, `A` sous `C`. Une transaction non commitée
/// range `C` sous `D` ; pendant ce temps, on range `B` sous `A`. Le cycle
/// `B → A → C → D → B` ne se voit qu'en relisant l'ancêtre `C` **sous
/// verrou** : le `PUT` attend la transaction, puis voit sa modification et
/// refuse. Une lecture simple de l'ancêtre (instantané pris avant le commit)
/// laisserait passer le cycle.
#[sqlx::test(migrations = "./test-schema")]
async fn course_un_cycle_ferme_par_un_ancetre_concurrent_est_vu(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let b = compte(&pool, &s, "10", None).await;
    let d = compte(&pool, &s, "100", Some(b)).await;
    let c = compte(&pool, &s, "20", None).await;
    let a = compte(&pool, &s, "200", Some(c)).await;

    let mut tx = pool.begin().await.unwrap();
    sqlx::query("UPDATE accounts SET parent_id = ? WHERE id = ?")
        .bind(d)
        .bind(c)
        .execute(&mut *tx)
        .await
        .unwrap();

    let (pool_b, s_b) = (pool.clone(), s.clone());
    let deplacement =
        tokio::spawn(async move { ranger(&pool_b, &s_b, b, ParentChange::Under(a)).await });
    assert!(
        attendre_une_requete_en_cours(
            &pool,
            &["SELECT parent_id FROM accounts", "LOCK IN SHARE MODE"],
            || { deplacement.is_finished() }
        )
        .await,
        "le déplacement doit attendre la relecture verrouillée de l'ancêtre"
    );
    tx.commit().await.unwrap();

    match tokio::time::timeout(ATTENTE, deplacement)
        .await
        .unwrap()
        .unwrap()
    {
        Err(DbError::AccountParentCycle { parent_number }) => assert_eq!(parent_number, "200"),
        autre => panic!("cycle attendu, obtenu {autre:?}"),
    }
    assert_eq!(lire(&pool, b).await.parent_id, None);
}

/// Revue ciblée des correctifs de la revue d'intégration de la PR de l'Epic 46
/// (M-A) : la réactivation passe le parent non imputable ; un parent de bilan en
/// devise qui porte des écritures est refusé, comme à la création —
/// `AccountForeignParentHasEntries` —, le sous-compte reste archivé et le parent
/// imputable. Un parent en CHF mouvementé reste accepté (écart #1431, celui de
/// la création).
#[sqlx::test(migrations = "./test-schema")]
async fn reactivation_sous_un_parent_en_devise_mouvemente_refusee(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    sqlx::query("UPDATE companies SET multi_currency_enabled = TRUE WHERE id = ?")
        .bind(s.company_id)
        .execute(&pool)
        .await
        .unwrap();
    // L'état atteignable (enfant archivé, parent repassé imputable puis
    // mouvementé), posé en SQL direct.
    let usd = compte(&pool, &s, "1021", None).await;
    let enfant = compte(&pool, &s, "10211", None).await;
    sqlx::query("UPDATE accounts SET parent_id = ?, active = FALSE WHERE id = ?")
        .bind(usd)
        .bind(enfant)
        .execute(&pool)
        .await
        .unwrap();
    ligne(&pool, &s, usd).await;
    sqlx::query("UPDATE accounts SET currency = 'USD' WHERE id = ?")
        .bind(usd)
        .execute(&pool)
        .await
        .unwrap();
    let v = lire(&pool, enfant).await.version;
    let err = accounts::reactivate(&pool, enfant, v, s.admin_user_id, false)
        .await
        .unwrap_err();
    assert!(
        matches!(
            err,
            DbError::AccountForeignParentHasEntries { parent_account_id, .. } if parent_account_id == usd
        ),
        "{err:?}"
    );
    assert!(
        !lire(&pool, enfant).await.active,
        "le sous-compte reste archivé"
    );
    assert!(lire(&pool, usd).await.postable, "rien n'est écrit");

    // En CHF, mouvementé : accepté, le parent passe non imputable.
    sqlx::query("UPDATE accounts SET currency = NULL WHERE id = ?")
        .bind(usd)
        .execute(&pool)
        .await
        .unwrap();
    assert!(
        accounts::reactivate(&pool, enfant, v, s.admin_user_id, false)
            .await
            .unwrap()
            .active
    );
    assert!(!lire(&pool, usd).await.postable);
}
