//! Contre-passer au début de l'exercice suivant — les transitoires de fin
//! d'exercice (Story 29-1b, #232).
//!
//! L'origine est datée du dernier jour de l'exercice `N` (2026) ; la
//! contre-passation doit l'être du premier jour de l'exercice qui suit, dans
//! cet exercice. Les états que les transitions refusent (un exercice suivant
//! clos alors que `N` est ouvert, invariant I) se posent par **SQL direct**.
//! Chaque test nomme la mutation de la fiche qu'il tue.
//!
//! Pré-requis : MariaDB démarré.

use std::time::Duration;

use chrono::NaiveDate;
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{NewJournalEntry, NewJournalEntryLine};
use kesh_db::errors::{DbError, ReversalBlocker};
use kesh_db::repositories::{fiscal_years, journal_entries};
use kesh_db::test_fixtures::{
    SeededCompany, attendre_une_requete_en_cours, seed_accounting_company,
};
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

/// Société de test, son exercice redaté sur 2026 (`N`) : c'est son **premier**
/// exercice — aucun antérieur.
async fn societe(pool: &MySqlPool) -> SeededCompany {
    let s = seed_accounting_company(pool).await.expect("seed");
    sqlx::query(
        "UPDATE fiscal_years SET name = 'Exercice 2026', start_date = '2026-01-01', \
         end_date = '2026-12-31' WHERE id = ?",
    )
    .bind(s.fiscal_year_id)
    .execute(pool)
    .await
    .unwrap();
    s
}

/// Un exercice au statut voulu, posé par SQL (la création refuserait certains
/// états que ces tests posent exprès).
async fn exercice(
    pool: &MySqlPool,
    s: &SeededCompany,
    nom: &str,
    debut: NaiveDate,
    fin: NaiveDate,
    status: &str,
) -> i64 {
    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(s.company_id)
    .bind(nom)
    .bind(debut)
    .bind(fin)
    .bind(status)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

async fn poser_statut(pool: &MySqlPool, fy: i64, status: &str) {
    sqlx::query("UPDATE fiscal_years SET status = ? WHERE id = ?")
        .bind(status)
        .bind(fy)
        .execute(pool)
        .await
        .unwrap();
}

/// La régularisation de fin d'exercice : 250.00 au débit de 1000 (lettrable),
/// au crédit de 4000 (charge, non lettrable), datée du 31.12.2026.
async fn regularisation(pool: &MySqlPool, s: &SeededCompany) -> i64 {
    journal_entries::create(
        pool,
        s.fiscal_year_id,
        s.admin_user_id,
        NewJournalEntry {
            company_id: s.company_id,
            entry_date: d(2026, 12, 31),
            journal: Journal::OD,
            description: "Charge payée d'avance".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: s.accounts["1000"],
                    debit: dec!(250.00),
                    credit: dec!(0),
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: s.accounts["4000"],
                    debit: dec!(0),
                    credit: dec!(250.00),
                    project_id: None,
                },
            ],
        },
    )
    .await
    .expect("écriture d'origine")
    .entry
    .id
}

async fn ecritures(pool: &MySqlPool, s: &SeededCompany) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE company_id = ?")
        .bind(s.company_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// La dernière entrée d'audit `journal_entry.reversed` : (cible, détails).
async fn dernier_audit(pool: &MySqlPool) -> (i64, Value) {
    let (cible, details): (i64, String) = sqlx::query_as(
        "SELECT entity_id, CAST(details_json AS CHAR) FROM audit_log \
         WHERE action = 'journal_entry.reversed' ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(pool)
    .await
    .expect("une entrée d'audit");
    (cible, serde_json::from_str(&details).unwrap())
}

async fn marque(pool: &MySqlPool, ligne: i64) -> Option<i64> {
    sqlx::query_scalar("SELECT lettering_key FROM journal_entry_lines WHERE id = ?")
        .bind(ligne)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// T-b1 — sur trois exercices ouverts N, N+1, N+2, la contre-passation est datée
/// du premier jour de **N+1** (non de N+2), dans N+1, liée à l'origine, lignes
/// inversées, lignes libres lettrées, audit qui porte la datation.
/// Tue M-b1 (daté du jour), M-b2 (`ORDER BY start_date DESC`), M-b4 (liaison
/// perdue).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t_b1_datee_du_premier_jour_de_l_exercice_suivant(pool: MySqlPool) {
    let s = societe(&pool).await;
    let n1 = exercice(
        &pool,
        &s,
        "Exercice 2027",
        d(2027, 1, 1),
        d(2027, 12, 31),
        "Open",
    )
    .await;
    let _n2 = exercice(
        &pool,
        &s,
        "Exercice 2028",
        d(2028, 1, 1),
        d(2028, 12, 31),
        "Open",
    )
    .await;
    let origine = regularisation(&pool, &s).await;
    let origine_lue = journal_entries::find_by_id(&pool, s.company_id, origine)
        .await
        .unwrap()
        .unwrap();

    let inverse = journal_entries::reverse_at_next_fiscal_year_start(
        &pool,
        s.company_id,
        origine,
        s.admin_user_id,
    )
    .await
    .expect("contre-passation au début de l'exercice suivant");

    assert_eq!(
        inverse.entry.entry_date,
        d(2027, 1, 1),
        "premier jour de N+1"
    );
    assert_eq!(inverse.entry.fiscal_year_id, n1, "dans N+1, non N+2");
    assert_eq!(
        inverse.entry.reverses_entry_id,
        Some(origine),
        "liée à l'origine"
    );
    assert_eq!(
        inverse.entry.description,
        format!(
            "Contre-passation écriture n° {} (Exercice 2026)",
            origine_lue.entry.entry_number
        )
    );
    // Lignes inversées, même position, mêmes comptes.
    assert_eq!(inverse.lines.len(), 2);
    for (o, m) in origine_lue.lines.iter().zip(inverse.lines.iter()) {
        assert_eq!(o.account_id, m.account_id);
        assert_eq!(o.debit, m.credit);
        assert_eq!(o.credit, m.debit);
    }
    // Lettrage : la ligne 1000 (lettrable, libre) avec son miroir ; 4000 non.
    let cle = marque(&pool, origine_lue.lines[0].id).await;
    assert!(cle.is_some(), "la ligne libre lettrable est lettrée");
    assert_eq!(
        marque(&pool, inverse.lines[0].id).await,
        cle,
        "avec son miroir"
    );
    assert_eq!(marque(&pool, origine_lue.lines[1].id).await, None);
    // Audit : action littérale, cible l'origine, la datation en plus.
    let (cible, details) = dernier_audit(&pool).await;
    assert_eq!(cible, origine);
    assert_eq!(
        details,
        json!({ "reversalJournalEntryId": inverse.entry.id, "dating": "nextFiscalYearStart" })
    );
}

/// T-b6 — un exercice suivant qui ne commence pas au lendemain (trou) : la date
/// est son `start_date`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t_b6_exercice_suivant_apres_un_trou(pool: MySqlPool) {
    let s = societe(&pool).await;
    let suivant = exercice(
        &pool,
        &s,
        "Exercice 2027-28",
        d(2027, 3, 1),
        d(2028, 2, 29),
        "Open",
    )
    .await;
    let origine = regularisation(&pool, &s).await;

    let inverse = journal_entries::reverse_at_next_fiscal_year_start(
        &pool,
        s.company_id,
        origine,
        s.admin_user_id,
    )
    .await
    .expect("contre-passation");
    assert_eq!(inverse.entry.entry_date, d(2027, 3, 1));
    assert_eq!(inverse.entry.fiscal_year_id, suivant);
}

/// T-b3 (a) — sans exercice suivant : `NextFiscalYearMissing`, qui nomme
/// l'exercice de l'origine ; aucune écriture créée. Tue M-b10 (repli sur la
/// date du jour).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t_b3_sans_exercice_suivant(pool: MySqlPool) {
    let s = societe(&pool).await;
    // Un exercice ANTÉRIEUR n'est pas un suivant.
    exercice(
        &pool,
        &s,
        "Exercice 2025",
        d(2025, 1, 1),
        d(2025, 12, 31),
        "Open",
    )
    .await;
    let origine = regularisation(&pool, &s).await;
    let avant = ecritures(&pool, &s).await;

    let r = journal_entries::reverse_at_next_fiscal_year_start(
        &pool,
        s.company_id,
        origine,
        s.admin_user_id,
    )
    .await;
    match r {
        Err(DbError::NextFiscalYearMissing { fiscal_year_name }) => {
            assert_eq!(fiscal_year_name, "Exercice 2026");
        }
        autre => panic!("NextFiscalYearMissing attendu, reçu {autre:?}"),
    }
    assert_eq!(ecritures(&pool, &s).await, avant, "aucune écriture créée");
}

/// T-b3 (b) — l'exercice suivant est clos : `NextFiscalYearClosed`, qui le
/// nomme — rendu par la confirmation verrouillante. Deux états : N ouvert et
/// N+1 clos (hérité, posé par SQL : l'invariant I l'interdit aux transitions),
/// puis le cas légal, N clos et N+1 clos. Tue M-b3 (sans confirmation,
/// `create_in_tx_inner` rendrait `FiscalYearClosed`) et M-b6.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t_b3_exercice_suivant_clos(pool: MySqlPool) {
    let s = societe(&pool).await;
    let n1 = exercice(
        &pool,
        &s,
        "Exercice 2027",
        d(2027, 1, 1),
        d(2027, 12, 31),
        "Open",
    )
    .await;
    // L'origine d'abord : sous un postérieur clos, sa création serait refusée.
    let origine = regularisation(&pool, &s).await;
    poser_statut(&pool, n1, "Closed").await;
    let avant = ecritures(&pool, &s).await;

    for etat in ["N ouvert, N+1 clos (hérité)", "N clos, N+1 clos"] {
        if etat.starts_with("N clos") {
            poser_statut(&pool, s.fiscal_year_id, "Closed").await;
        }
        let r = journal_entries::reverse_at_next_fiscal_year_start(
            &pool,
            s.company_id,
            origine,
            s.admin_user_id,
        )
        .await;
        match r {
            Err(DbError::NextFiscalYearClosed {
                fiscal_year_id,
                fiscal_year_name,
            }) => {
                assert_eq!(fiscal_year_id, n1, "{etat}");
                assert_eq!(fiscal_year_name, "Exercice 2027", "{etat}");
            }
            autre => panic!("{etat} : NextFiscalYearClosed attendu, reçu {autre:?}"),
        }
    }
    assert_eq!(ecritures(&pool, &s).await, avant, "aucune écriture créée");
}

/// T-b3 (c) — le premier jour de N+1 est dans la période verrouillée :
/// `PeriodLocked`, le refus de `create_in_tx_inner`, inchangé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t_b3_premier_jour_verrouille(pool: MySqlPool) {
    let s = societe(&pool).await;
    exercice(
        &pool,
        &s,
        "Exercice 2027",
        d(2027, 1, 1),
        d(2027, 12, 31),
        "Open",
    )
    .await;
    let origine = regularisation(&pool, &s).await;
    // Posée par SQL : `lock_books` refuse une borne au jour ou au futur.
    sqlx::query("UPDATE companies SET books_locked_through = '2027-01-01' WHERE id = ?")
        .bind(s.company_id)
        .execute(&pool)
        .await
        .unwrap();

    let r = journal_entries::reverse_at_next_fiscal_year_start(
        &pool,
        s.company_id,
        origine,
        s.admin_user_id,
    )
    .await;
    match r {
        Err(DbError::PeriodLocked {
            locked_through,
            attempted,
        }) => {
            assert_eq!(locked_through, d(2027, 1, 1));
            assert_eq!(attempted, d(2027, 1, 1));
        }
        autre => panic!("PeriodLocked attendu, reçu {autre:?}"),
    }
}

/// T-b3 (d) (e) (f) — les refus de la contre-passation, inchangés : déjà
/// contre-passée, contre-passation d'une contre-passation, écriture d'une pièce.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t_b3_les_refus_de_la_contre_passation_restent(pool: MySqlPool) {
    let s = societe(&pool).await;
    exercice(
        &pool,
        &s,
        "Exercice 2027",
        d(2027, 1, 1),
        d(2027, 12, 31),
        "Open",
    )
    .await;
    let origine = regularisation(&pool, &s).await;
    let inverse = journal_entries::reverse_at_next_fiscal_year_start(
        &pool,
        s.company_id,
        origine,
        s.admin_user_id,
    )
    .await
    .unwrap();

    fn refus<T>(r: Result<T, DbError>) -> ReversalBlocker {
        match r {
            Err(DbError::EntryNotReversable { blocker, .. }) => blocker,
            Err(e) => panic!("EntryNotReversable attendu, reçu {e:?}"),
            Ok(_) => panic!("EntryNotReversable attendu, la contre-passation a réussi"),
        }
    }
    let r = journal_entries::reverse_at_next_fiscal_year_start(
        &pool,
        s.company_id,
        origine,
        s.admin_user_id,
    )
    .await;
    assert_eq!(refus(r), ReversalBlocker::AlreadyReversed);
    let r = journal_entries::reverse_at_next_fiscal_year_start(
        &pool,
        s.company_id,
        inverse.entry.id,
        s.admin_user_id,
    )
    .await;
    assert_eq!(refus(r), ReversalBlocker::IsAReversal);

    // Écriture d'une facture client (posée par SQL).
    let piece = regularisation(&pool, &s).await;
    let contact = sqlx::query(
        "INSERT INTO contacts (company_id, contact_type, name) VALUES (?, 'Entreprise', 'Client test')",
    )
    .bind(s.company_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    sqlx::query(
        "INSERT INTO invoices (company_id, contact_id, date, journal_entry_id) VALUES (?, ?, ?, ?)",
    )
    .bind(s.company_id)
    .bind(contact)
    .bind(d(2026, 12, 31))
    .bind(piece)
    .execute(&pool)
    .await
    .unwrap();
    let r = journal_entries::reverse_at_next_fiscal_year_start(
        &pool,
        s.company_id,
        piece,
        s.admin_user_id,
    )
    .await;
    assert_eq!(refus(r), ReversalBlocker::OwnedByInvoice);
}

/// Une écriture d'une autre société est introuvable (anti-énumération).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_ecriture_d_une_autre_societe_est_introuvable(pool: MySqlPool) {
    let s = societe(&pool).await;
    exercice(
        &pool,
        &s,
        "Exercice 2027",
        d(2027, 1, 1),
        d(2027, 12, 31),
        "Open",
    )
    .await;
    let origine = regularisation(&pool, &s).await;
    let r = journal_entries::reverse_at_next_fiscal_year_start(
        &pool,
        s.company_id + 1000,
        origine,
        s.admin_user_id,
    )
    .await;
    assert!(
        matches!(r, Err(DbError::NotFound)),
        "reçu {:?}",
        r.map(|_| ())
    );
}

/// T-b5 (i) — **la contre-passation d'abord** : A la mène jusqu'au bout dans une
/// transaction qu'il tient ouverte ; `close(N+1)` (B) attend le verrou de N à
/// son étape (b'), puis aboutit après le `COMMIT` de A. N est le **premier**
/// exercice de la société (sinon c'est le cycle du test 13 d, non une attente).
///
/// ⚠️ Test de **comportement** (les deux gestes se sérialisent, sans
/// interblocage), il ne tue aucune mutation de la fiche, et on le déclare.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t_b5_i_la_contre_passation_d_abord_puis_la_cloture(pool: MySqlPool) {
    let s = societe(&pool).await;
    let n1 = exercice(
        &pool,
        &s,
        "Exercice 2027",
        d(2027, 1, 1),
        d(2027, 12, 31),
        "Open",
    )
    .await;
    let origine = regularisation(&pool, &s).await;
    poser_statut(&pool, s.fiscal_year_id, "Closed").await;

    let mut a = pool.begin().await.unwrap();
    let inverse = journal_entries::reverse_at_next_fiscal_year_start_in_tx(
        &mut a,
        s.company_id,
        origine,
        s.admin_user_id,
    )
    .await
    .expect("contre-passation de A");

    let (p, company, user) = (pool.clone(), s.company_id, s.admin_user_id);
    let b = tokio::spawn(async move {
        kesh_db::retry::retry_on_deadlock("test::close", || {
            fiscal_years::close(&p, user, company, n1)
        })
        .await
    });
    // B attend N en (b') : motif de `LOCK_EARLIER_BY_ID_SQL` (liste de colonnes
    // réduite à `id`), absent de l'étape (c) et de `create_in_tx_inner`.
    let vu = attendre_une_requete_en_cours(
        &pool,
        &["SELECT id FROM fiscal_years WHERE id = ", "FOR UPDATE"],
        || b.is_finished(),
    )
    .await;
    assert!(vu, "B aurait dû attendre le verrou de N tenu par A");
    a.commit().await.unwrap();

    let clos = tokio::time::timeout(Duration::from_secs(10), b)
        .await
        .expect("B aboutit")
        .unwrap()
        .expect("clôture de N+1");
    assert_eq!(clos.id, n1);
    let fy: i64 = sqlx::query_scalar("SELECT fiscal_year_id FROM journal_entries WHERE id = ?")
        .bind(inverse.entry.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(fy, n1, "la contre-passation est dans N+1, désormais clos");
}

/// Texte de `LOCK_EARLIER_BY_ID_SQL` (`fiscal_years.rs`), **recopié** : la
/// constante est privée, et le côté B de T-b5 (ii) refait les effets de
/// `close`, qui n'a pas de variante en transaction. ⚠️ Couplage déclaré : si la
/// constante change, ce texte doit suivre (comme les motifs du test 15-12a).
const LOCK_EARLIER_BY_ID_SQL: &str =
    "SELECT id FROM fiscal_years WHERE id = ? AND company_id = ? FOR UPDATE";
/// Texte de `LOCK_IN_COMPANY_SQL` (`fiscal_years.rs`), recopié — même couplage.
const LOCK_IN_COMPANY_SQL: &str = "SELECT id, company_id, name, start_date, end_date, status, created_at, updated_at \
     FROM fiscal_years WHERE id = ? AND company_id = ? FOR UPDATE";

/// T-b5 (ii) — **la clôture d'abord** : B tient N puis N+1 et passe N+1 à
/// `Closed`, sans valider ; A (la vraie fonction, sur le pool) attend à
/// l'étape (1), sur le `JOIN fiscal_years fy … FOR UPDATE` ; B valide ; A rend
/// **`NextFiscalYearClosed`** — le refus est le sien, non `FiscalYearClosed`.
/// Tue M-b11 (étape (1) sans le `JOIN` verrouillant : A n'attend plus là).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t_b5_ii_la_cloture_d_abord_puis_la_contre_passation(pool: MySqlPool) {
    let s = societe(&pool).await;
    let n1 = exercice(
        &pool,
        &s,
        "Exercice 2027",
        d(2027, 1, 1),
        d(2027, 12, 31),
        "Open",
    )
    .await;
    let origine = regularisation(&pool, &s).await;
    poser_statut(&pool, s.fiscal_year_id, "Closed").await;
    let avant = ecritures(&pool, &s).await;

    let mut b = pool.begin().await.unwrap();
    let _: Option<i64> = sqlx::query_scalar(LOCK_EARLIER_BY_ID_SQL)
        .bind(s.fiscal_year_id)
        .bind(s.company_id)
        .fetch_optional(&mut *b)
        .await
        .unwrap();
    let tenu = sqlx::query(LOCK_IN_COMPANY_SQL)
        .bind(n1)
        .bind(s.company_id)
        .fetch_optional(&mut *b)
        .await
        .unwrap();
    assert!(tenu.is_some(), "B tient N+1");
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ? AND company_id = ?")
        .bind(n1)
        .bind(s.company_id)
        .execute(&mut *b)
        .await
        .unwrap();

    let (p, company, user) = (pool.clone(), s.company_id, s.admin_user_id);
    let a = tokio::spawn(async move {
        journal_entries::reverse_at_next_fiscal_year_start(&p, company, origine, user).await
    });
    let vu = attendre_une_requete_en_cours(&pool, &["JOIN fiscal_years fy", "FOR UPDATE"], || {
        a.is_finished()
    })
    .await;
    assert!(
        vu,
        "A aurait dû attendre à l'étape (1), sur le verrou de N tenu par B"
    );
    b.commit().await.unwrap();

    let r = tokio::time::timeout(Duration::from_secs(10), a)
        .await
        .expect("A aboutit")
        .unwrap();
    match r {
        Err(DbError::NextFiscalYearClosed { fiscal_year_id, .. }) => assert_eq!(fiscal_year_id, n1),
        autre => panic!("NextFiscalYearClosed attendu, reçu {:?}", autre.map(|_| ())),
    }
    assert_eq!(
        ecritures(&pool, &s).await,
        avant,
        "rien n'est écrit dans N+1 clos"
    );
}
