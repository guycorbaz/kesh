//! Story 46-5a — la route groupée du plan, côté dépôt :
//! `accounts::apply_groups` crée des comptes de groupe et y range des comptes
//! **en une seule transaction, tout ou rien**, avec les contrôles de la
//! création et du changement de parent ; `accounts::check_groups_shape` juge la
//! forme de la demande avant toute transaction.
//!
//! Chaque refus est vérifié avec son étape (`GroupStepError`) et, pour le tout
//! ou rien, sur l'état complet de la société : nombre de comptes, `parent_id`,
//! `postable`, `version` et nombre d'entrées d'audit, identiques avant et après.

use kesh_db::entities::account::{
    Account, AccountType, AccountUpdate, CurrencyChange, GroupKind, GroupMember, GroupProposal,
    MAX_GROUP_MEMBERS, MAX_GROUP_PROPOSALS, NewAccount, OwnFxAccountsChange, ParentChange,
};
use kesh_db::errors::{AccountGroupReason, DbError, GroupStepError};
use kesh_db::repositories::accounts;
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use sqlx::MySqlPool;

/// Un compte créé par le dépôt dans la société `company`.
async fn compte_dans(
    pool: &MySqlPool,
    user: i64,
    company: i64,
    numero: &str,
    type_: AccountType,
    parent: Option<i64>,
) -> i64 {
    accounts::create(
        pool,
        user,
        NewAccount::new(company, numero, format!("Compte {numero}"), type_, parent),
    )
    .await
    .unwrap()
    .id
}

/// Un compte d'actif de la société de test.
async fn compte(pool: &MySqlPool, s: &SeededCompany, numero: &str, parent: Option<i64>) -> i64 {
    compte_dans(
        pool,
        s.admin_user_id,
        s.company_id,
        numero,
        AccountType::Asset,
        parent,
    )
    .await
}

async fn lire(pool: &MySqlPool, id: i64) -> Account {
    accounts::find_by_id(pool, id).await.unwrap().unwrap()
}

/// Le membre `id` à sa `version` actuelle.
async fn membre(pool: &MySqlPool, id: i64) -> GroupMember {
    GroupMember {
        id,
        version: lire(pool, id).await.version,
    }
}

async fn membres(pool: &MySqlPool, ids: &[i64]) -> Vec<GroupMember> {
    let mut v = Vec::new();
    for id in ids {
        v.push(membre(pool, *id).await);
    }
    v
}

fn creer(numero: &str, parent: Option<i64>, members: Vec<GroupMember>) -> GroupProposal {
    GroupProposal {
        kind: GroupKind::Create {
            number: numero.into(),
            name: format!("Groupe {numero}"),
            parent_id: parent,
        },
        members,
    }
}

fn rattacher(account_id: i64, members: Vec<GroupMember>) -> GroupProposal {
    GroupProposal {
        kind: GroupKind::Attach { account_id },
        members,
    }
}

fn reutiliser(account_id: i64, members: Vec<GroupMember>) -> GroupProposal {
    GroupProposal {
        kind: GroupKind::Reuse { account_id },
        members,
    }
}

async fn appliquer(
    pool: &MySqlPool,
    s: &SeededCompany,
    groups: &[GroupProposal],
) -> Result<kesh_db::entities::account::GroupsOutcome, GroupStepError> {
    accounts::apply_groups(pool, s.company_id, s.admin_user_id, groups).await
}

/// Le motif d'un refus `ACCOUNT_GROUP_INVALID` : `(groupIndex, motif, compte)`.
fn motif(e: &GroupStepError) -> (usize, AccountGroupReason, Option<i64>) {
    assert_eq!(
        e.group_index, None,
        "un ACCOUNT_GROUP_INVALID n'est pas enveloppé"
    );
    match &e.source {
        DbError::AccountGroupInvalid {
            group_index,
            reason,
            account_id,
        } => (*group_index, *reason, *account_id),
        other => panic!("ACCOUNT_GROUP_INVALID attendu, reçu {other:?}"),
    }
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

/// L'état complet du plan de la société et de son audit : ce que le tout ou
/// rien doit laisser intact.
#[derive(Debug, PartialEq)]
struct Etat {
    comptes: Vec<(i64, String, Option<i64>, bool, bool, i32)>,
    audit: i64,
}

async fn etat(pool: &MySqlPool, s: &SeededCompany) -> Etat {
    let comptes = sqlx::query_as(
        "SELECT id, number, parent_id, postable, active, version FROM accounts \
         WHERE company_id = ? ORDER BY id",
    )
    .bind(s.company_id)
    .fetch_all(pool)
    .await
    .unwrap();
    let audit: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_log")
        .fetch_one(pool)
        .await
        .unwrap();
    Etat { comptes, audit }
}

async fn audits(pool: &MySqlPool, action: &str, entity_id: i64) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE action = ? AND entity_id = ?")
        .bind(action)
        .bind(entity_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn autre_societe(pool: &MySqlPool) -> i64 {
    sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre SA', 'Rue 1\n1000 Lausanne', 'Independant', 'FR', 'FR')",
    )
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

/// Le `PUT` qui range `id` sous `parent`, le reste inchangé.
async fn ranger(
    pool: &MySqlPool,
    s: &SeededCompany,
    id: i64,
    parent: i64,
) -> Result<Account, DbError> {
    let a = lire(pool, id).await;
    accounts::update(
        pool,
        id,
        a.version,
        s.admin_user_id,
        AccountUpdate {
            name: a.name,
            account_type: a.account_type,
            role: a.role,
            postable: a.postable,
            currency: CurrencyChange::Unchanged,
            own_fx_accounts: OwnFxAccountsChange::Unchanged,
            parent: ParentChange::Under(parent),
        },
        false,
    )
    .await
}

/// Le nominal : deux groupes créés, leurs membres rangés dessous ; les
/// groupes sont non imputables, de type commun, sous le parent demandé ; une
/// entrée d'audit par groupe créé et par compte déplacé, aucune de plus.
#[sqlx::test(migrations = "./test-schema")]
async fn deux_groupes_crees_et_leurs_membres_ranges(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let racine = compte(&pool, &s, "10", None).await;
    let a1 = compte(&pool, &s, "1031", Some(racine)).await;
    let a2 = compte(&pool, &s, "1032", Some(racine)).await;
    let b1 = compte(&pool, &s, "1041", Some(racine)).await;
    let audit_avant = etat(&pool, &s).await.audit;

    let groups = vec![
        creer("103", Some(racine), membres(&pool, &[a1, a2]).await),
        creer("104", Some(racine), membres(&pool, &[b1]).await),
    ];
    let out = appliquer(&pool, &s, &groups).await.unwrap();

    assert_eq!(out.created.len(), 2);
    assert!(out.reused.is_empty());
    let moved: Vec<i64> = out.moved.iter().map(|a| a.id).collect();
    assert_eq!(moved, vec![a1, a2, b1]);
    for (g, membres) in [(&out.created[0], vec![a1, a2]), (&out.created[1], vec![b1])] {
        let g = lire(&pool, g.id).await;
        assert!(!g.postable, "un groupe créé est non imputable");
        assert_eq!(g.account_type, AccountType::Asset);
        assert_eq!(g.parent_id, Some(racine));
        assert_eq!(g.currency, None);
        assert_eq!(g.role, None);
        for m in membres {
            assert_eq!(lire(&pool, m).await.parent_id, Some(g.id));
        }
    }
    assert_eq!(lire(&pool, out.created[0].id).await.number, "103");
    assert_eq!(
        etat(&pool, &s).await.audit - audit_avant,
        2 + 3,
        "deux créations et trois déplacements"
    );
    assert_eq!(audits(&pool, "account.created", out.created[0].id).await, 1);
    assert_eq!(audits(&pool, "account.updated", a1).await, 1);
}

/// Le tout ou rien, sur une erreur qui ne naît **qu'après des écritures** : la
/// première proposition crée son groupe et y range ses membres ; la seconde
/// réutilise un compte qui porte une écriture, refusé au changement de parent
/// de son premier membre (`ACCOUNT_PARENT_HAS_ENTRIES`). Rien n'est écrit.
#[sqlx::test(migrations = "./test-schema")]
async fn tout_ou_rien_sur_un_refus_ne_apres_des_ecritures(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let a1 = compte(&pool, &s, "1031", None).await;
    let a2 = compte(&pool, &s, "1032", None).await;
    let r = compte(&pool, &s, "105", None).await;
    let c1 = compte(&pool, &s, "1051", None).await;
    ligne(&pool, &s, r).await;
    let avant = etat(&pool, &s).await;

    let groups = vec![
        creer("103", None, membres(&pool, &[a1, a2]).await),
        reutiliser(r, membres(&pool, &[c1]).await),
    ];
    let e = appliquer(&pool, &s, &groups).await.unwrap_err();
    assert_eq!((e.group_index, e.account_id), (Some(1), Some(c1)));
    assert!(
        matches!(e.source, DbError::AccountParentHasEntries { parent_account_id, .. } if parent_account_id == r),
        "{e:?}"
    );
    assert_eq!(etat(&pool, &s).await, avant, "rien n'est écrit");
}

/// Un conflit de `version` du dernier membre est refusé dans la passe de
/// verrous, avant toute écriture, avec son étape.
#[sqlx::test(migrations = "./test-schema")]
async fn conflit_de_version_du_dernier_membre_refuse_avant_toute_ecriture(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let a1 = compte(&pool, &s, "1031", None).await;
    let b1 = compte(&pool, &s, "1041", None).await;
    let avant = etat(&pool, &s).await;
    let mut perime = membre(&pool, b1).await;
    perime.version += 1;
    let groups = vec![
        creer("103", None, membres(&pool, &[a1]).await),
        creer("104", None, vec![perime]),
    ];
    let e = appliquer(&pool, &s, &groups).await.unwrap_err();
    assert_eq!((e.group_index, e.account_id), (Some(1), Some(b1)));
    assert!(matches!(e.source, DbError::OptimisticLockConflict), "{e:?}");
    assert_eq!(etat(&pool, &s).await, avant);
}

/// Un membre **retypé** après la lecture de sa `version` rend `409` avec son
/// étape, jamais `mixedTypes` : la `version` est jugée avant les contrôles
/// d'état.
#[sqlx::test(migrations = "./test-schema")]
async fn membre_retype_depuis_l_apercu_rend_409_et_non_mixed_types(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let a1 = compte(&pool, &s, "1031", None).await;
    let a2 = compte(&pool, &s, "1032", None).await;
    let lu = membres(&pool, &[a1, a2]).await;
    let a = lire(&pool, a2).await;
    accounts::update(
        &pool,
        a2,
        a.version,
        s.admin_user_id,
        AccountUpdate {
            name: a.name,
            account_type: AccountType::Liability,
            role: None,
            postable: true,
            currency: CurrencyChange::Unchanged,
            own_fx_accounts: OwnFxAccountsChange::Unchanged,
            parent: ParentChange::Unchanged,
        },
        false,
    )
    .await
    .unwrap();
    let e = appliquer(&pool, &s, &[creer("103", None, lu)])
        .await
        .unwrap_err();
    assert_eq!((e.group_index, e.account_id), (Some(0), Some(a2)));
    assert!(matches!(e.source, DbError::OptimisticLockConflict), "{e:?}");
}

/// Un membre archivé rend le refus d'`update` (`ILLEGAL_STATE_TRANSITION`),
/// un compte de rattachement archivé `notAGroup`, de réutilisation archivé
/// `notReusable`.
#[sqlx::test(migrations = "./test-schema")]
async fn comptes_archives_membre_rattachement_reutilisation(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let a1 = compte(&pool, &s, "1031", None).await;
    let archive = compte(&pool, &s, "1039", None).await;
    let v = lire(&pool, archive).await.version;
    accounts::archive(&pool, archive, v, s.admin_user_id)
        .await
        .unwrap();

    let m = membre(&pool, archive).await;
    let e = appliquer(&pool, &s, &[creer("103", None, vec![m])])
        .await
        .unwrap_err();
    assert_eq!((e.group_index, e.account_id), (Some(0), Some(archive)));
    assert!(
        matches!(e.source, DbError::IllegalStateTransition(_)),
        "{e:?}"
    );

    let m1 = membres(&pool, &[a1]).await;
    let e = appliquer(&pool, &s, &[rattacher(archive, m1.clone())])
        .await
        .unwrap_err();
    assert_eq!(motif(&e), (0, AccountGroupReason::NotAGroup, Some(archive)));
    let e = appliquer(&pool, &s, &[reutiliser(archive, m1)])
        .await
        .unwrap_err();
    assert_eq!(
        motif(&e),
        (0, AccountGroupReason::NotReusable, Some(archive))
    );
}

/// Un membre déjà sous son groupe **au milieu** d'une demande est un no-op
/// (ni `version`, ni audit) qui n'annule rien : le groupe créé avant lui et
/// les comptes rangés avant et après lui restent au commit.
#[sqlx::test(migrations = "./test-schema")]
async fn membre_deja_range_au_milieu_n_annule_rien(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let a1 = compte(&pool, &s, "1031", None).await;
    let g2 = compte(&pool, &s, "104", None).await;
    let x = compte(&pool, &s, "1041", Some(g2)).await;
    let y = compte(&pool, &s, "1042", None).await;
    let vx = lire(&pool, x).await.version;

    let groups = vec![
        creer("103", None, membres(&pool, &[a1]).await),
        rattacher(g2, membres(&pool, &[x, y]).await),
    ];
    let out = appliquer(&pool, &s, &groups).await.unwrap();
    let g1 = out.created[0].id;
    assert_eq!(lire(&pool, a1).await.parent_id, Some(g1));
    assert_eq!(lire(&pool, y).await.parent_id, Some(g2));
    let lx = lire(&pool, x).await;
    assert_eq!((lx.parent_id, lx.version), (Some(g2), vx));
    assert_eq!(
        audits(&pool, "account.updated", x).await,
        0,
        "no-op : aucun audit"
    );
    let moved: Vec<i64> = out.moved.iter().map(|a| a.id).collect();
    assert_eq!(
        moved,
        vec![a1, y],
        "le membre déjà rangé n'est pas « déplacé »"
    );
}

/// Réutilisation : le compte devient non imputable, garde son nom, son type et
/// sa `version`, et figure dans `reused` ; sans entrée d'audit propre.
#[sqlx::test(migrations = "./test-schema")]
async fn reutiliser_un_compte_le_rend_non_imputable(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let r = compte(&pool, &s, "105", None).await;
    let c1 = compte(&pool, &s, "1051", None).await;
    let avant = lire(&pool, r).await;
    assert!(avant.postable);
    let out = appliquer(&pool, &s, &[reutiliser(r, membres(&pool, &[c1]).await)])
        .await
        .unwrap();
    assert!(out.created.is_empty());
    assert_eq!(out.reused.len(), 1);
    let apres = lire(&pool, r).await;
    assert!(!apres.postable && !out.reused[0].postable);
    assert_eq!((apres.name, apres.version), (avant.name, avant.version));
    assert_eq!(lire(&pool, c1).await.parent_id, Some(r));
    assert_eq!(audits(&pool, "account.updated", r).await, 0);
}

/// Réutiliser un compte qui a reçu une écriture avant l'appel →
/// `ACCOUNT_PARENT_HAS_ENTRIES`, au changement de parent de son membre.
#[sqlx::test(migrations = "./test-schema")]
async fn reutiliser_un_compte_mouvemente_est_refuse(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let r = compte(&pool, &s, "105", None).await;
    let c1 = compte(&pool, &s, "1051", None).await;
    let lu = membres(&pool, &[c1]).await;
    ligne(&pool, &s, r).await;
    let e = appliquer(&pool, &s, &[reutiliser(r, lu)])
        .await
        .unwrap_err();
    assert_eq!((e.group_index, e.account_id), (Some(0), Some(c1)));
    assert!(
        matches!(e.source, DbError::AccountParentHasEntries { .. }),
        "{e:?}"
    );
    assert!(lire(&pool, r).await.postable, "rien n'est écrit");
}

/// Rattachement à un groupe existant ; refus sur un compte sans sous-compte
/// (`notAGroup`), sur un compte qui a un sous-compte (`notReusable` en
/// réutilisation), et sur un groupe mouvementé (`ACCOUNT_PARENT_HAS_ENTRIES`).
#[sqlx::test(migrations = "./test-schema")]
async fn rattacher_et_ses_refus(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let g = compte(&pool, &s, "104", None).await;
    let _x = compte(&pool, &s, "1041", Some(g)).await;
    let y = compte(&pool, &s, "1042", None).await;
    let seul = compte(&pool, &s, "106", None).await;

    let e = appliquer(&pool, &s, &[rattacher(seul, membres(&pool, &[y]).await)])
        .await
        .unwrap_err();
    assert_eq!(motif(&e), (0, AccountGroupReason::NotAGroup, Some(seul)));
    let e = appliquer(&pool, &s, &[reutiliser(g, membres(&pool, &[y]).await)])
        .await
        .unwrap_err();
    assert_eq!(motif(&e), (0, AccountGroupReason::NotReusable, Some(g)));

    let out = appliquer(&pool, &s, &[rattacher(g, membres(&pool, &[y]).await)])
        .await
        .unwrap();
    assert!(out.created.is_empty() && out.reused.is_empty());
    assert_eq!(out.moved.len(), 1);
    assert_eq!(lire(&pool, y).await.parent_id, Some(g));

    // Un groupe qui porte une écriture : refusé au rangement de son membre.
    let z = compte(&pool, &s, "1043", None).await;
    ligne(&pool, &s, g).await;
    let e = appliquer(&pool, &s, &[rattacher(g, membres(&pool, &[z]).await)])
        .await
        .unwrap_err();
    assert_eq!((e.group_index, e.account_id), (Some(0), Some(z)));
    assert!(
        matches!(e.source, DbError::AccountParentHasEntries { .. }),
        "{e:?}"
    );
}

/// `mixedTypes` (le premier membre d'un autre type) et `groupTypeMismatch`
/// (groupe rattaché d'un autre type que ses membres).
#[sqlx::test(migrations = "./test-schema")]
async fn types_meles_et_type_du_groupe(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let a1 = compte(&pool, &s, "1031", None).await;
    let p1 = compte_dans(
        &pool,
        s.admin_user_id,
        s.company_id,
        "2031",
        AccountType::Liability,
        None,
    )
    .await;
    let e = appliquer(
        &pool,
        &s,
        &[creer("103", None, membres(&pool, &[a1, p1]).await)],
    )
    .await
    .unwrap_err();
    assert_eq!(motif(&e), (0, AccountGroupReason::MixedTypes, Some(p1)));

    let gp = compte_dans(
        &pool,
        s.admin_user_id,
        s.company_id,
        "20",
        AccountType::Liability,
        None,
    )
    .await;
    let _enfant = compte_dans(
        &pool,
        s.admin_user_id,
        s.company_id,
        "201",
        AccountType::Liability,
        Some(gp),
    )
    .await;
    let e = appliquer(&pool, &s, &[rattacher(gp, membres(&pool, &[a1]).await)])
        .await
        .unwrap_err();
    assert_eq!(
        motif(&e),
        (0, AccountGroupReason::GroupTypeMismatch, Some(gp))
    );
}

/// `numberTaken` : numéro porté par un compte actif, par un compte archivé, ou
/// demandé deux fois dans la même demande.
#[sqlx::test(migrations = "./test-schema")]
async fn numero_deja_porte(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let a1 = compte(&pool, &s, "1031", None).await;
    let b1 = compte(&pool, &s, "1041", None).await;
    let pris = compte(&pool, &s, "103", None).await;
    let archive = compte(&pool, &s, "107", None).await;
    let v = lire(&pool, archive).await.version;
    accounts::archive(&pool, archive, v, s.admin_user_id)
        .await
        .unwrap();

    let e = appliquer(
        &pool,
        &s,
        &[creer("103", None, membres(&pool, &[a1]).await)],
    )
    .await
    .unwrap_err();
    assert_eq!(motif(&e), (0, AccountGroupReason::NumberTaken, Some(pris)));
    let e = appliquer(
        &pool,
        &s,
        &[creer("107", None, membres(&pool, &[a1]).await)],
    )
    .await
    .unwrap_err();
    assert_eq!(
        motif(&e),
        (0, AccountGroupReason::NumberTaken, Some(archive))
    );
    let e = appliquer(
        &pool,
        &s,
        &[
            creer("108", None, membres(&pool, &[a1]).await),
            creer("108", None, membres(&pool, &[b1]).await),
        ],
    )
    .await
    .unwrap_err();
    assert_eq!(motif(&e), (1, AccountGroupReason::NumberTaken, None));

    // Revue P1, L1 : la comparaison suit la collation de la colonne, sans
    // casse — `10a` puis `10A` rendent le motif, non l'index unique.
    let avant = etat(&pool, &s).await;
    let e = appliquer(
        &pool,
        &s,
        &[
            creer("10a", None, membres(&pool, &[a1]).await),
            creer("10A", None, membres(&pool, &[b1]).await),
        ],
    )
    .await
    .unwrap_err();
    assert_eq!(motif(&e), (1, AccountGroupReason::NumberTaken, None));
    assert_eq!(etat(&pool, &s).await, avant, "rien n'est écrit");
}

/// Revue P1, M1 : un compte déplacé par une proposition, puis parent d'un
/// groupe créé par la suivante, passe non imputable (sans hausse de
/// `version`) ; `moved` le rend tel que la base le porte au commit.
#[sqlx::test(migrations = "./test-schema")]
async fn moved_relu_apres_la_derniere_ecriture(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let a1 = compte(&pool, &s, "1031", None).await;
    let b1 = compte(&pool, &s, "2001", None).await;
    assert!(lire(&pool, a1).await.postable);

    let groups = vec![
        creer("103", None, membres(&pool, &[a1]).await),
        creer("200", Some(a1), membres(&pool, &[b1]).await),
    ];
    let out = appliquer(&pool, &s, &groups).await.unwrap();

    let base = lire(&pool, a1).await;
    assert!(!base.postable, "a1 est devenu parent du groupe 200");
    let rendu = out
        .moved
        .iter()
        .find(|a| a.id == a1)
        .expect("a1 est déplacé");
    assert_eq!(
        (rendu.postable, rendu.version, rendu.parent_id),
        (base.postable, base.version, base.parent_id),
        "moved rend l'état du commit"
    );
    let moved: Vec<i64> = out.moved.iter().map(|a| a.id).collect();
    assert_eq!(moved, vec![a1, b1]);
}

/// Le parent d'un groupe créé passe la règle du `PUT` : un parent mouvementé
/// est refusé (`ACCOUNT_PARENT_HAS_ENTRIES`, étape sans compte) — alors que
/// `POST /accounts` l'accepte et que `PUT /accounts/{id}` le refuse, comme
/// avant la généralisation de la garde.
#[sqlx::test(migrations = "./test-schema")]
async fn parent_mouvemente_d_un_groupe_cree_est_refuse(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let parent = compte(&pool, &s, "10", None).await;
    let a1 = compte(&pool, &s, "1031", None).await;
    ligne(&pool, &s, parent).await;
    let avant = etat(&pool, &s).await;
    let e = appliquer(
        &pool,
        &s,
        &[creer("103", Some(parent), membres(&pool, &[a1]).await)],
    )
    .await
    .unwrap_err();
    assert_eq!((e.group_index, e.account_id), (Some(0), None));
    assert!(
        matches!(&e.source, DbError::AccountParentHasEntries { parent_number, .. } if parent_number == "10"),
        "{e:?}"
    );
    assert_eq!(etat(&pool, &s).await, avant);

    // `POST /accounts` et `PUT /accounts/{id}` inchangés.
    let enfant = compte(&pool, &s, "109", Some(parent)).await;
    assert_eq!(lire(&pool, enfant).await.parent_id, Some(parent));
    let err = ranger(&pool, &s, a1, parent).await.unwrap_err();
    assert!(
        matches!(err, DbError::AccountParentHasEntries { .. }),
        "{err:?}"
    );
}

/// Le parent d'un groupe créé, membre de la même demande et rangé sous ce
/// groupe, ferme un cycle : `ACCOUNT_PARENT_CYCLE` au rangement de ce membre.
#[sqlx::test(migrations = "./test-schema")]
async fn parent_du_groupe_membre_de_la_demande_ferme_un_cycle(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let a1 = compte(&pool, &s, "1031", None).await;
    let a2 = compte(&pool, &s, "1032", None).await;
    let avant = etat(&pool, &s).await;
    let e = appliquer(
        &pool,
        &s,
        &[creer("103", Some(a1), membres(&pool, &[a1, a2]).await)],
    )
    .await
    .unwrap_err();
    assert_eq!((e.group_index, e.account_id), (Some(0), Some(a1)));
    assert!(
        matches!(e.source, DbError::AccountParentCycle { .. }),
        "{e:?}"
    );
    assert_eq!(etat(&pool, &s).await, avant);
}

/// Un membre, un compte de rattachement ou de réutilisation **d'une autre
/// société** rend `NotFound` (jamais `mixedTypes`, `notAGroup` ni
/// `notReusable`), rien n'est écrit. Les comptes étrangers sont choisis pour
/// que, sans le filtre de société de la passe de verrous, l'erreur soit l'un
/// de ces trois motifs.
#[sqlx::test(migrations = "./test-schema")]
async fn compte_d_une_autre_societe_rend_not_found(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let autre = autre_societe(&pool).await;
    let a1 = compte(&pool, &s, "1031", None).await;
    // Membre étranger de type différent : sans filtre, `mixedTypes`.
    let membre_etranger = compte_dans(
        &pool,
        s.admin_user_id,
        autre,
        "2031",
        AccountType::Liability,
        None,
    )
    .await;
    // Rattachement étranger sans sous-compte : sans filtre, `notAGroup`.
    let seul_etranger = compte_dans(
        &pool,
        s.admin_user_id,
        autre,
        "106",
        AccountType::Asset,
        None,
    )
    .await;
    // Réutilisation étrangère avec un sous-compte : sans filtre, `notReusable`.
    let groupe_etranger = compte_dans(
        &pool,
        s.admin_user_id,
        autre,
        "104",
        AccountType::Asset,
        None,
    )
    .await;
    let _ = compte_dans(
        &pool,
        s.admin_user_id,
        autre,
        "1041",
        AccountType::Asset,
        Some(groupe_etranger),
    )
    .await;
    let avant = etat(&pool, &s).await;

    let cas = [
        (
            creer("103", None, membres(&pool, &[a1, membre_etranger]).await),
            membre_etranger,
        ),
        (
            rattacher(seul_etranger, membres(&pool, &[a1]).await),
            seul_etranger,
        ),
        (
            reutiliser(groupe_etranger, membres(&pool, &[a1]).await),
            groupe_etranger,
        ),
    ];
    for (proposition, etranger) in cas {
        let e = appliquer(&pool, &s, &[proposition]).await.unwrap_err();
        assert_eq!((e.group_index, e.account_id), (Some(0), Some(etranger)));
        assert!(matches!(e.source, DbError::NotFound), "{e:?}");
    }
    assert_eq!(etat(&pool, &s).await, avant);
}

/// Les contrôles de forme, avant toute transaction : `empty`,
/// `memberIsOwnGroup`, `accountInTwoGroups` (dans une proposition et entre
/// deux), `groupIsMember`, `groupInTwoProposals`, `tooMany` (propositions et
/// membres).
#[test]
fn forme_de_la_demande() {
    let m = |id| GroupMember { id, version: 1 };
    let shape = |g: &[GroupProposal]| match accounts::check_groups_shape(g) {
        Err(DbError::AccountGroupInvalid {
            group_index,
            reason,
            account_id,
        }) => (group_index, reason, account_id),
        other => panic!("ACCOUNT_GROUP_INVALID attendu, reçu {other:?}"),
    };
    assert!(accounts::check_groups_shape(&[creer("1", None, vec![m(1)])]).is_ok());
    assert_eq!(
        shape(&[creer("1", None, vec![m(1)]), creer("2", None, vec![])]),
        (1, AccountGroupReason::Empty, None)
    );
    assert_eq!(
        shape(&[rattacher(5, vec![m(1), m(5)])]),
        (0, AccountGroupReason::MemberIsOwnGroup, Some(5))
    );
    assert_eq!(
        shape(&[creer("1", None, vec![m(1), m(1)])]),
        (0, AccountGroupReason::AccountInTwoGroups, Some(1))
    );
    assert_eq!(
        shape(&[
            creer("1", None, vec![m(1)]),
            creer("2", None, vec![m(2), m(1)])
        ]),
        (1, AccountGroupReason::AccountInTwoGroups, Some(1))
    );
    assert_eq!(
        shape(&[creer("1", None, vec![m(1)]), reutiliser(1, vec![m(2)])]),
        (1, AccountGroupReason::GroupIsMember, Some(1))
    );
    // Revue P1, L2 : un même compte visé par deux propositions — deux `reuse`,
    // deux `attach`, ou l'un et l'autre — est refusé à la forme, à la seconde.
    for (premiere, seconde) in [
        (reutiliser(9, vec![m(1)]), reutiliser(9, vec![m(2)])),
        (rattacher(9, vec![m(1)]), rattacher(9, vec![m(2)])),
        (rattacher(9, vec![m(1)]), reutiliser(9, vec![m(2)])),
    ] {
        assert_eq!(
            shape(&[creer("1", None, vec![m(3)]), premiere, seconde]),
            (2, AccountGroupReason::GroupInTwoProposals, Some(9))
        );
    }
    assert!(
        accounts::check_groups_shape(&[rattacher(9, vec![m(1)]), reutiliser(8, vec![m(2)])])
            .is_ok()
    );
    let beaucoup: Vec<GroupProposal> = (0..=MAX_GROUP_PROPOSALS as i64)
        .map(|i| creer(&i.to_string(), None, vec![m(i + 1)]))
        .collect();
    assert_eq!(
        shape(&beaucoup),
        (MAX_GROUP_PROPOSALS, AccountGroupReason::TooMany, None)
    );
    let gros = vec![
        creer("1", None, (1..=MAX_GROUP_MEMBERS as i64).map(m).collect()),
        creer("2", None, vec![m(1_000_000)]),
    ];
    assert_eq!(shape(&gros), (1, AccountGroupReason::TooMany, None));
}

// --- Epic 46, garde du mainteneur du 2026-11-26 : un compte désigné par un
// compte bancaire (son compte du plan, ou le compte de frais d'un prestataire)
// ne devient pas parent — ni groupe réutilisé, ni parent d'un groupe créé, ni
// parent au `PUT` ; rangé comme membre, il reste accepté et imputable.

/// Un compte bancaire ordinaire de la société, lié au compte du plan `compte`.
async fn banque_liee(pool: &MySqlPool, s: &SeededCompany, nom: &str, compte: i64) -> i64 {
    sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, journal_account_id) \
         VALUES (?, ?, 'CH9300762011623852957', ?)",
    )
    .bind(s.company_id)
    .bind(nom)
    .bind(compte)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

/// Un compte de prestataire (`paypal`, `wise`, `stripe`) dont `frais` est le
/// compte de frais, sans compte du plan lié.
async fn prestataire(pool: &MySqlPool, s: &SeededCompany, provider: &str, frais: i64) -> i64 {
    sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, provider, provider_account, fee_account_id) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(s.company_id)
    .bind(format!("Solde {provider}"))
    .bind(provider)
    .bind(format!("{provider} principal"))
    .bind(frais)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

/// Le refus attendu : `(parentAccountId, numéro, compte bancaire, nom affiché)`.
fn lie(e: &DbError) -> (i64, String, i64, String) {
    match e {
        DbError::AccountParentBankLinked {
            parent_account_id,
            parent_number,
            bank_account_id,
            bank_name,
            ..
        } => (
            *parent_account_id,
            parent_number.clone(),
            *bank_account_id,
            bank_name.clone(),
        ),
        other => panic!("ACCOUNT_PARENT_BANK_LINKED attendu, reçu {other:?}"),
    }
}

/// Le compte du plan d'un compte bancaire : refusé en groupe réutilisé, en
/// parent d'un groupe créé et en parent au `PUT`, sans rien écrire ; accepté
/// comme membre (il reste imputable) ; un compte ordinaire, réutilisé, accepté.
#[sqlx::test(migrations = "./test-schema")]
async fn compte_d_un_compte_bancaire_ne_devient_pas_groupe(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let lie_id = compte(&pool, &s, "102", None).await;
    let c1 = compte(&pool, &s, "1021", None).await;
    let banque = banque_liee(&pool, &s, "Banque Test", lie_id).await;
    let avant = etat(&pool, &s).await;

    // Groupe réutilisé : refusé au rangement de son premier membre.
    let e = appliquer(
        &pool,
        &s,
        &[reutiliser(lie_id, membres(&pool, &[c1]).await)],
    )
    .await
    .unwrap_err();
    assert_eq!((e.group_index, e.account_id), (Some(0), Some(c1)));
    assert_eq!(
        lie(&e.source),
        (lie_id, "102".into(), banque, "Banque Test".into())
    );
    assert_eq!(etat(&pool, &s).await, avant, "tout ou rien");

    // Parent d'un groupe créé : refusé, étape sans compte.
    let e = appliquer(
        &pool,
        &s,
        &[creer("103", Some(lie_id), membres(&pool, &[c1]).await)],
    )
    .await
    .unwrap_err();
    assert_eq!((e.group_index, e.account_id), (Some(0), None));
    assert_eq!(lie(&e.source).0, lie_id);
    assert_eq!(etat(&pool, &s).await, avant, "tout ou rien");

    // `PUT /accounts/{id}` avec `parentId` : même refus.
    let err = ranger(&pool, &s, c1, lie_id).await.unwrap_err();
    assert_eq!(lie(&err).2, banque);
    assert!(lire(&pool, lie_id).await.postable, "rien n'est écrit");

    // Membre : accepté, et il reste imputable.
    let out = appliquer(
        &pool,
        &s,
        &[creer("10", None, membres(&pool, &[lie_id]).await)],
    )
    .await
    .unwrap();
    let apres = lire(&pool, lie_id).await;
    assert_eq!(apres.parent_id, Some(out.created[0].id));
    assert!(apres.postable, "un membre lié reste imputable");

    // Un compte ordinaire réutilisé reste accepté.
    let ordinaire = compte(&pool, &s, "105", None).await;
    let c2 = compte(&pool, &s, "1051", None).await;
    appliquer(
        &pool,
        &s,
        &[reutiliser(ordinaire, membres(&pool, &[c2]).await)],
    )
    .await
    .unwrap();
    assert!(!lire(&pool, ordinaire).await.postable);
}

/// Le compte de frais partagé par deux prestataires : refusé, le refus nomme le
/// premier compte bancaire (par identifiant). Un compte bancaire clos, ou
/// archivé, le désigne encore (consigne du mainteneur : il peut être rouvert ou
/// désarchivé, ses écritures passées restent liées).
#[sqlx::test(migrations = "./test-schema")]
async fn compte_de_frais_partage_ne_devient_pas_groupe(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let frais = compte_dans(
        &pool,
        s.admin_user_id,
        s.company_id,
        "684",
        AccountType::Expense,
        None,
    )
    .await;
    let membre_frais = compte_dans(
        &pool,
        s.admin_user_id,
        s.company_id,
        "6841",
        AccountType::Expense,
        None,
    )
    .await;
    let paypal = prestataire(&pool, &s, "paypal", frais).await;
    let wise = prestataire(&pool, &s, "wise", frais).await;
    assert!(paypal < wise);

    let e = appliquer(
        &pool,
        &s,
        &[reutiliser(frais, membres(&pool, &[membre_frais]).await)],
    )
    .await
    .unwrap_err();
    assert_eq!(
        lie(&e.source),
        (frais, "684".into(), paypal, "Solde paypal".into())
    );

    // Le compte PayPal clos : le compte Wise le désigne toujours.
    sqlx::query("UPDATE bank_accounts SET closed_on = '2026-06-30' WHERE id = ?")
        .bind(paypal)
        .execute(&pool)
        .await
        .unwrap();
    let e = ranger(&pool, &s, membre_frais, frais).await.unwrap_err();
    assert_eq!(lie(&e).2, paypal, "un compte bancaire clos désigne encore");

    // Les deux archivés : ils le désignent encore, le rangement reste refusé.
    sqlx::query("UPDATE bank_accounts SET archived = TRUE WHERE id IN (?, ?)")
        .bind(paypal)
        .bind(wise)
        .execute(&pool)
        .await
        .unwrap();
    let e = ranger(&pool, &s, membre_frais, frais).await.unwrap_err();
    assert_eq!(
        lie(&e).2,
        paypal,
        "un compte bancaire archivé désigne encore"
    );
    assert!(lire(&pool, frais).await.postable, "rien n'est écrit");

    // Désigné par personne : le rangement passe.
    sqlx::query("UPDATE bank_accounts SET fee_account_id = NULL WHERE id IN (?, ?)")
        .bind(paypal)
        .bind(wise)
        .execute(&pool)
        .await
        .unwrap();
    ranger(&pool, &s, membre_frais, frais).await.unwrap();
    assert!(!lire(&pool, frais).await.postable);
}

/// Revue d'intégration de la PR de l'Epic 46 (H1) — la réactivation, chemin par
/// lequel un compte redevient parent sans changement de parent : un groupe dont
/// le seul sous-compte est archivé repasse imputable, puis est lié à un compte
/// bancaire (compte du plan ou compte de frais d'un prestataire, clos ou
/// archivé compris) ; réactiver le sous-compte est refusé, mêmes `details`, et
/// rien n'est écrit. Sans liaison, la réactivation passe.
#[sqlx::test(migrations = "./test-schema")]
async fn reactivation_sous_un_compte_lie_refusee(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();

    /// Le groupe `numero` et son seul sous-compte, archivé ; le groupe repassé
    /// imputable par `PUT postable: true`. Rend `(groupe, sous-compte)`.
    async fn groupe_vide(
        pool: &MySqlPool,
        s: &SeededCompany,
        numero: &str,
        t: AccountType,
    ) -> (i64, i64) {
        let g = compte_dans(pool, s.admin_user_id, s.company_id, numero, t, None).await;
        let c = compte_dans(
            pool,
            s.admin_user_id,
            s.company_id,
            &format!("{numero}1"),
            t,
            Some(g),
        )
        .await;
        assert!(!lire(pool, g).await.postable, "groupe non imputable");
        accounts::archive(pool, c, lire(pool, c).await.version, s.admin_user_id)
            .await
            .unwrap();
        let a = lire(pool, g).await;
        accounts::update(
            pool,
            g,
            a.version,
            s.admin_user_id,
            AccountUpdate {
                name: a.name,
                account_type: a.account_type,
                role: a.role,
                postable: true,
                currency: CurrencyChange::Unchanged,
                own_fx_accounts: OwnFxAccountsChange::Unchanged,
                parent: ParentChange::Unchanged,
            },
            false,
        )
        .await
        .unwrap();
        assert!(lire(pool, g).await.postable, "groupe repassé imputable");
        (g, c)
    }
    async fn reactiver(pool: &MySqlPool, s: &SeededCompany, c: i64) -> Result<Account, DbError> {
        accounts::reactivate(pool, c, lire(pool, c).await.version, s.admin_user_id, false).await
    }

    // Compte du plan d'un compte bancaire.
    let (g, c) = groupe_vide(&pool, &s, "102", AccountType::Asset).await;
    let banque = banque_liee(&pool, &s, "Banque Test", g).await;
    let err = reactiver(&pool, &s, c).await.unwrap_err();
    assert_eq!(lie(&err), (g, "102".into(), banque, "Banque Test".into()));
    match &err {
        DbError::AccountParentBankLinked {
            reactivated_number, ..
        } => assert_eq!(reactivated_number.as_deref(), Some("1021")),
        other => panic!("ACCOUNT_PARENT_BANK_LINKED attendu, reçu {other:?}"),
    }
    assert!(!lire(&pool, c).await.active, "le sous-compte reste archivé");
    assert!(lire(&pool, g).await.postable, "le groupe reste imputable");

    // Compte de frais d'un prestataire, compte bancaire archivé : refusé aussi.
    let (f, cf) = groupe_vide(&pool, &s, "684", AccountType::Expense).await;
    let paypal = prestataire(&pool, &s, "paypal", f).await;
    sqlx::query("UPDATE bank_accounts SET archived = TRUE WHERE id = ?")
        .bind(paypal)
        .execute(&pool)
        .await
        .unwrap();
    let err = reactiver(&pool, &s, cf).await.unwrap_err();
    assert_eq!(
        lie(&err).2,
        paypal,
        "un compte bancaire archivé désigne encore"
    );
    assert!(!lire(&pool, cf).await.active);

    // Liaison retirée : la réactivation passe, et le groupe repasse non
    // imputable, comme à la création d'un sous-compte (sans hausse de version).
    sqlx::query("UPDATE bank_accounts SET journal_account_id = NULL WHERE id = ?")
        .bind(banque)
        .execute(&pool)
        .await
        .unwrap();
    let v_groupe = lire(&pool, g).await.version;
    let apres = reactiver(&pool, &s, c).await.unwrap();
    assert!(apres.active);
    let groupe = lire(&pool, g).await;
    assert!(!groupe.postable, "le groupe redevient non imputable");
    assert_eq!(groupe.version, v_groupe);
    // Il ne se lie plus à un compte bancaire : le contrôle d'imputabilité de la
    // liaison lit la colonne (`ensure_postable_if_active_in_tx`).
    let mut tx = pool.begin().await.unwrap();
    let e = accounts::ensure_postable_if_active_in_tx(&mut tx, s.company_id, g)
        .await
        .unwrap_err();
    assert!(matches!(e, DbError::AccountsNotPostable(_)), "{e:?}");
}

/// `POST /accounts` (création d'un sous-compte) : un compte désigné par un compte
/// bancaire ne reçoit pas de sous-compte — même refus ; la règle des parents
/// mouvementés de la création, elle, ne change pas (#1431 : un parent
/// mouvementé non lié reste accepté à la création).
#[sqlx::test(migrations = "./test-schema")]
async fn creation_d_un_sous_compte_sous_un_compte_lie_refusee(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let lie_id = compte(&pool, &s, "102", None).await;
    let banque = banque_liee(&pool, &s, "Banque Test", lie_id).await;
    let avant = etat(&pool, &s).await;
    let err = accounts::create(
        &pool,
        s.admin_user_id,
        NewAccount::new(
            s.company_id,
            "1021",
            "Sous-compte",
            AccountType::Asset,
            Some(lie_id),
        ),
    )
    .await
    .unwrap_err();
    assert_eq!(
        lie(&err),
        (lie_id, "102".into(), banque, "Banque Test".into())
    );
    assert_eq!(etat(&pool, &s).await, avant, "rien n'est écrit");

    // Un parent mouvementé, non lié, reste accepté à la création (#1431 inchangée).
    let mouvemente = compte(&pool, &s, "10", None).await;
    ligne(&pool, &s, mouvemente).await;
    let enfant = compte(&pool, &s, "109", Some(mouvemente)).await;
    assert_eq!(lire(&pool, enfant).await.parent_id, Some(mouvemente));
}
