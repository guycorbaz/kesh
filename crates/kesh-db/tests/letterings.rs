//! Tests de dépôt — Story 15-1a-i (#518) : la marque du lettrage, posée et
//! retirée par la primitive unique (`letterings::create_group_in_tx`,
//! `dissolve_group_in_tx`).
//!
//! Couvre : la pose de la marque (AC3), un test par cause de refus **dans
//! l'ordre des rangs** — chaque requête cumulant la cause visée et la suivante,
//! pour qu'un rang permuté se voie —, la règle des périodes au lettrage (AC4)
//! et au délettrage (AC5), les quatre branches de la dissolution, la dispense
//! de lettrabilité à la dissolution (C104), le mode `System` (R7 point 3),
//! l'audit (AC10), l'anti-IDOR au dépôt (AC11), l'invariant des groupes (AC13)
//! et deux tests de concurrence : la sérialisation avec la clôture, et l'ordre
//! d'acquisition des verrous d'exercices (R7 point 2, mutation « trier par
//! `id` » tuée).
//!
//! ⚠️ Les écritures sont posées en SQL direct (`ecriture`), comme les états
//! hérités (« N ouvert, N+1 clos ») des tests de la Story 15-8a : le lettrage
//! doit se juger sur l'état de la base, quel qu'en soit le chemin. Aucune
//! marque n'est jamais écrite autrement que par la primitive.

use chrono::NaiveDate;
use kesh_db::errors::{DbError, ReversalBlocker};
use kesh_db::repositories::letterings::{self, Actor, LetteringGroup, Mode, Origin};
use kesh_db::test_fixtures::{
    SeededCompany, attendre_une_requete_en_cours, seed_accounting_company,
    seed_contact_and_product, sonde_verrou_nowait,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

// ---------------------------------------------------------------------------
// Montage
// ---------------------------------------------------------------------------

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

/// La société seedée et trois exercices : 2025 (celui du seed, ramené à
/// l'année), 2026 et 2027, tous ouverts.
struct Monde {
    s: SeededCompany,
    fy25: i64,
    fy26: i64,
    fy27: i64,
}

impl Monde {
    fn company(&self) -> i64 {
        self.s.company_id
    }
    /// Compte lettrable des tests (1100, `Asset`, sans compte bancaire).
    fn lettrable(&self) -> i64 {
        self.s.accounts["1100"]
    }
    fn actor(&self) -> Actor {
        Actor {
            user_id: self.s.admin_user_id,
            api_key_id: None,
        }
    }
}

async fn exercice(pool: &MySqlPool, company_id: i64, annee: i32, status: &str) -> i64 {
    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(format!("Exercice {annee}"))
    .bind(d(annee, 1, 1))
    .bind(d(annee, 12, 31))
    .bind(status)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

async fn monde(pool: &MySqlPool) -> Monde {
    let s = seed_accounting_company(pool).await.expect("seed");
    sqlx::query(
        "UPDATE fiscal_years SET name = 'Exercice 2025', start_date = '2025-01-01', \
         end_date = '2025-12-31' WHERE id = ?",
    )
    .bind(s.fiscal_year_id)
    .execute(pool)
    .await
    .unwrap();
    let fy26 = exercice(pool, s.company_id, 2026, "Open").await;
    let fy27 = exercice(pool, s.company_id, 2027, "Open").await;
    Monde {
        fy25: s.fiscal_year_id,
        s,
        fy26,
        fy27,
    }
}

async fn set_status(pool: &MySqlPool, fy: i64, status: &str) {
    sqlx::query("UPDATE fiscal_years SET status = ? WHERE id = ?")
        .bind(status)
        .bind(fy)
        .execute(pool)
        .await
        .unwrap();
}

async fn poser_borne(pool: &MySqlPool, company_id: i64, borne: NaiveDate) {
    sqlx::query("UPDATE companies SET books_locked_through = ? WHERE id = ?")
        .bind(borne)
        .bind(company_id)
        .execute(pool)
        .await
        .unwrap();
}

/// Écriture posée en SQL direct ; rend `(id de l'écriture, ids des lignes)`.
async fn ecriture(
    pool: &MySqlPool,
    company_id: i64,
    fy: i64,
    date: NaiveDate,
    lignes: &[(i64, Decimal, Decimal)],
) -> (i64, Vec<i64>) {
    let numero: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(entry_number), 0) + 1 FROM journal_entries \
         WHERE company_id = ? AND fiscal_year_id = ?",
    )
    .bind(company_id)
    .bind(fy)
    .fetch_one(pool)
    .await
    .unwrap();
    let entry_id = sqlx::query(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, \
         journal, description) VALUES (?, ?, ?, ?, 'OD', 'lettrage')",
    )
    .bind(company_id)
    .bind(fy)
    .bind(numero)
    .bind(date)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let mut ids = Vec::new();
    for (n, (account, debit, credit)) in lignes.iter().enumerate() {
        let id = sqlx::query(
            "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(entry_id)
        .bind(*account)
        .bind(n as i32 + 1)
        .bind(*debit)
        .bind(*credit)
        .execute(pool)
        .await
        .unwrap()
        .last_insert_id() as i64;
        ids.push(id);
    }
    (entry_id, ids)
}

/// Une ligne sur `compte`, au débit (`montant` > 0) ou au crédit (< 0), avec sa
/// contrepartie au compte 2000 ; rend `(écriture, ligne sur compte)`.
async fn ligne(
    pool: &MySqlPool,
    m: &Monde,
    compte: i64,
    fy: i64,
    date: NaiveDate,
    montant: Decimal,
) -> (i64, i64) {
    let contrepartie = m.s.accounts["2000"];
    let (debit, credit) = if montant > Decimal::ZERO {
        (montant, Decimal::ZERO)
    } else {
        (Decimal::ZERO, -montant)
    };
    let (entry, ids) = ecriture(
        pool,
        m.company(),
        fy,
        date,
        &[(compte, debit, credit), (contrepartie, credit, debit)],
    )
    .await;
    (entry, ids[0])
}

/// Deux lignes du compte lettrable qui se soldent : 100 au débit à `d1`, 100 au
/// crédit à `d2`.
async fn paire(
    pool: &MySqlPool,
    m: &Monde,
    (fy1, d1): (i64, NaiveDate),
    (fy2, d2): (i64, NaiveDate),
) -> (i64, i64) {
    let (_, a) = ligne(pool, m, m.lettrable(), fy1, d1, dec!(100)).await;
    let (_, b) = ligne(pool, m, m.lettrable(), fy2, d2, dec!(-100)).await;
    (a, b)
}

async fn lettrer_avec(
    pool: &MySqlPool,
    m: &Monde,
    ids: &[i64],
    origin: Origin,
    mode: Mode,
) -> Result<LetteringGroup, DbError> {
    let mut tx = pool.begin().await.unwrap();
    let r =
        letterings::create_group_in_tx(&mut tx, m.company(), ids, origin, mode, m.actor()).await;
    if r.is_ok() {
        tx.commit().await.unwrap();
    }
    r
}

async fn lettrer(pool: &MySqlPool, m: &Monde, ids: &[i64]) -> Result<LetteringGroup, DbError> {
    lettrer_avec(pool, m, ids, Origin::Manual, Mode::Manual).await
}

async fn delettrer_avec(
    pool: &MySqlPool,
    m: &Monde,
    key: i64,
    mode: Mode,
) -> Result<LetteringGroup, DbError> {
    let mut tx = pool.begin().await.unwrap();
    let r = letterings::dissolve_group_in_tx(&mut tx, m.company(), key, mode, m.actor()).await;
    if r.is_ok() {
        tx.commit().await.unwrap();
    }
    r
}

async fn delettrer(pool: &MySqlPool, m: &Monde, key: i64) -> Result<LetteringGroup, DbError> {
    delettrer_avec(pool, m, key, Mode::Manual).await
}

/// La marque d'une ligne, telle qu'en base.
async fn marque(pool: &MySqlPool, line: i64) -> (Option<i64>, Option<String>) {
    sqlx::query_as("SELECT lettering_key, lettering_origin FROM journal_entry_lines WHERE id = ?")
        .bind(line)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Rattache l'écriture `entry` à une facture client (pièce : `OwnedByInvoice`).
async fn faire_facture(pool: &MySqlPool, m: &Monde, entry: i64, numero: &str) -> i64 {
    let (contact, _) = seed_contact_and_product(pool, m.company())
        .await
        .expect("contact");
    sqlx::query(
        "INSERT INTO invoices (company_id, contact_id, date, journal_entry_id, invoice_number) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(m.company())
    .bind(contact)
    .bind(d(2026, 2, 1))
    .bind(entry)
    .bind(numero)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

/// Une seconde société, avec son exercice 2026 et ses comptes ; rend l'id
/// d'une de ses lignes sur un compte d'actif, et celui d'une seconde.
async fn autre_societe(pool: &MySqlPool, m: &Monde) -> (i64, i64) {
    let other: i64 = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         SELECT CONCAT(name, ' bis'), address, org_type, accounting_language, instance_language \
         FROM companies WHERE id = ?",
    )
    .bind(m.company())
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let fy = exercice(pool, other, 2026, "Open").await;
    let compte = |numero: &'static str, ty: &'static str| {
        let pool = pool.clone();
        async move {
            sqlx::query(
                "INSERT INTO accounts (company_id, number, name, account_type) VALUES (?, ?, ?, ?)",
            )
            .bind(other)
            .bind(numero)
            .bind(numero)
            .bind(ty)
            .execute(&pool)
            .await
            .unwrap()
            .last_insert_id() as i64
        }
    };
    let actif = compte("1100", "Asset").await;
    let passif = compte("2000", "Liability").await;
    let (_, a) = ecriture(
        pool,
        other,
        fy,
        d(2026, 3, 1),
        &[(actif, dec!(100), dec!(0)), (passif, dec!(0), dec!(100))],
    )
    .await;
    let (_, b) = ecriture(
        pool,
        other,
        fy,
        d(2026, 3, 2),
        &[(actif, dec!(0), dec!(100)), (passif, dec!(100), dec!(0))],
    )
    .await;
    (a[0], b[0])
}

// ---------------------------------------------------------------------------
// AC3 — la pose de la marque
// ---------------------------------------------------------------------------

/// AC3 — la marque est la clé `MIN(id)` et l'origine, sur **toutes** les lignes ;
/// la réponse porte le code, le compte, et l'exercice par ligne (C127).
#[sqlx::test(migrations = "./test-schema")]
async fn lettering_marks_every_line_with_the_smallest_id(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (_, facture) = ligne(&pool, &m, m.lettrable(), m.fy26, d(2026, 2, 1), dec!(100)).await;
    let (_, acompte) = ligne(&pool, &m, m.lettrable(), m.fy26, d(2026, 2, 5), dec!(-60)).await;
    let (_, solde) = ligne(&pool, &m, m.lettrable(), m.fy27, d(2027, 1, 10), dec!(-40)).await;
    let (_, ouverte) = ligne(&pool, &m, m.lettrable(), m.fy26, d(2026, 2, 6), dec!(-5)).await;

    // Ordre de la requête indifférent : la clé est le plus petit `id`.
    let group = lettrer(&pool, &m, &[solde, facture, acompte])
        .await
        .expect("lettrage");
    assert_eq!(group.key, facture);
    assert_eq!(
        group.code,
        kesh_core::lettering::code_from_key(facture as u64)
    );
    assert_eq!(group.origin, Origin::Manual);
    assert_eq!(group.account_id, m.lettrable());
    assert_eq!(group.account_number, "1100");
    let ids: Vec<i64> = group.lines.iter().map(|l| l.id).collect();
    assert_eq!(ids, vec![facture, acompte, solde], "lignes triées par id");
    let noms: Vec<&str> = group
        .lines
        .iter()
        .map(|l| l.fiscal_year_name.as_str())
        .collect();
    assert_eq!(
        noms,
        vec!["Exercice 2026", "Exercice 2026", "Exercice 2027"]
    );
    assert_eq!(group.lines[2].fiscal_year_id, m.fy27);

    for l in [facture, acompte, solde] {
        assert_eq!(
            marque(&pool, l).await,
            (Some(facture), Some("manual".into()))
        );
    }
    assert_eq!(marque(&pool, ouverte).await, (None, None), "intacte");
}

// ---------------------------------------------------------------------------
// AC3 — un test par cause, chaque requête cumulant la cause visée et la suivante
// ---------------------------------------------------------------------------

/// Rang 1 avant rang 2 : un identifiant seul, ou répété, est refusé en forme —
/// même inexistant.
#[sqlx::test(migrations = "./test-schema")]
async fn rank_1_too_few_lines_comes_first(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (a, _) = paire(&pool, &m, (m.fy26, d(2026, 2, 1)), (m.fy26, d(2026, 2, 2))).await;
    for ids in [
        vec![],
        vec![a],
        vec![a, a],
        vec![999_999],
        vec![999_999, 999_999],
    ] {
        assert!(
            matches!(
                lettrer(&pool, &m, &ids).await,
                Err(DbError::LetteringTooFewLines)
            ),
            "{ids:?}"
        );
    }
}

/// Rang 2 avant rang 3 — et anti-IDOR (AC11) : une ligne introuvable ou d'une
/// autre société rend le même `NotFound`, même quand les comptes diffèrent.
#[sqlx::test(migrations = "./test-schema")]
async fn rank_2_unknown_or_foreign_line_is_not_found(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (a, b) = paire(&pool, &m, (m.fy26, d(2026, 2, 1)), (m.fy26, d(2026, 2, 2))).await;
    let (_, autre_compte) = ligne(
        &pool,
        &m,
        m.s.accounts["1000"],
        m.fy26,
        d(2026, 2, 3),
        dec!(5),
    )
    .await;
    let (x, y) = autre_societe(&pool, &m).await;
    for ids in [
        vec![a, 999_999],
        vec![autre_compte, 999_999],
        vec![x, y],
        vec![a, x],
        vec![b, autre_compte, y],
    ] {
        assert!(
            matches!(lettrer(&pool, &m, &ids).await, Err(DbError::NotFound)),
            "{ids:?}"
        );
    }
    // La société étrangère n'a rien reçu.
    assert_eq!(marque(&pool, x).await, (None, None));
}

/// Rang 3 avant rang 4 : deux comptes, dont un non lettrable.
#[sqlx::test(migrations = "./test-schema")]
async fn rank_3_accounts_differ(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (_, a) = ligne(&pool, &m, m.lettrable(), m.fy26, d(2026, 2, 1), dec!(100)).await;
    let (_, b) = ligne(
        &pool,
        &m,
        m.s.accounts["3000"],
        m.fy26,
        d(2026, 2, 2),
        dec!(-100),
    )
    .await;
    assert!(matches!(
        lettrer(&pool, &m, &[a, b]).await,
        Err(DbError::LetteringAccountsDiffer)
    ));
}

/// Rang 4 avant rang 4 bis : compte de produit, dans un exercice clos.
#[sqlx::test(migrations = "./test-schema")]
async fn rank_4_account_not_letterable(pool: MySqlPool) {
    let m = monde(&pool).await;
    let produit = m.s.accounts["3000"];
    let (_, a) = ligne(&pool, &m, produit, m.fy25, d(2025, 2, 1), dec!(100)).await;
    let (_, b) = ligne(&pool, &m, produit, m.fy25, d(2025, 2, 2), dec!(-100)).await;
    set_status(&pool, m.fy25, "Closed").await;
    assert!(matches!(
        lettrer(&pool, &m, &[a, b]).await,
        Err(DbError::LetteringAccountNotLetterable)
    ));
    // Charge : pas lettrable non plus.
    let charge = m.s.accounts["4000"];
    let (_, c) = ligne(&pool, &m, charge, m.fy26, d(2026, 2, 1), dec!(100)).await;
    let (_, e) = ligne(&pool, &m, charge, m.fy26, d(2026, 2, 2), dec!(-100)).await;
    assert!(matches!(
        lettrer(&pool, &m, &[c, e]).await,
        Err(DbError::LetteringAccountNotLetterable)
    ));
    // Passif : lettrable.
    let passif = m.s.accounts["2000"];
    let (_, f) = ligne(&pool, &m, passif, m.fy26, d(2026, 2, 1), dec!(100)).await;
    let (_, g) = ligne(&pool, &m, passif, m.fy26, d(2026, 2, 2), dec!(-100)).await;
    lettrer(&pool, &m, &[f, g])
        .await
        .expect("un passif se lettre");
}

/// R4, C127 — un compte désigné par un compte bancaire **archivé** reste non
/// lettrable : il relève de la réconciliation.
#[sqlx::test(migrations = "./test-schema")]
async fn archived_bank_account_keeps_its_account_unletterable(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (a, b) = paire(&pool, &m, (m.fy26, d(2026, 2, 1)), (m.fy26, d(2026, 2, 2))).await;
    sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, journal_account_id, archived) \
         VALUES (?, 'Banque', 'CH9300762011623852957', ?, TRUE)",
    )
    .bind(m.company())
    .bind(m.lettrable())
    .execute(&pool)
    .await
    .unwrap();
    assert!(matches!(
        lettrer(&pool, &m, &[a, b]).await,
        Err(DbError::LetteringAccountNotLetterable)
    ));
    let mut conn = pool.acquire().await.unwrap();
    assert!(
        !letterings::is_letterable_account(&mut conn, m.company(), m.lettrable())
            .await
            .unwrap()
    );
}

/// Rang 4 bis avant rang 5 : toutes les lignes en exercice clos, dont une
/// d'une pièce.
#[sqlx::test(migrations = "./test-schema")]
async fn rank_4_bis_all_lines_in_closed_periods(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (entry, a) = ligne(&pool, &m, m.lettrable(), m.fy25, d(2025, 2, 1), dec!(100)).await;
    let (_, b) = ligne(&pool, &m, m.lettrable(), m.fy25, d(2025, 2, 2), dec!(-100)).await;
    faire_facture(&pool, &m, entry, "F-2025-001").await;
    set_status(&pool, m.fy25, "Closed").await;
    assert!(matches!(
        lettrer(&pool, &m, &[a, b]).await,
        Err(DbError::LetteringAllLinesInClosedPeriods)
    ));
}

/// Rang 5 avant rang 6 (R5) : une ligne d'une pièce, et une ligne déjà lettrée.
/// Le refus nomme la pièce.
#[sqlx::test(migrations = "./test-schema")]
async fn rank_5_line_owned_by_a_document(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (entry, a) = ligne(&pool, &m, m.lettrable(), m.fy26, d(2026, 2, 1), dec!(100)).await;
    let invoice = faire_facture(&pool, &m, entry, "F-2026-014").await;
    let (x, y) = paire(&pool, &m, (m.fy26, d(2026, 3, 1)), (m.fy26, d(2026, 3, 2))).await;
    lettrer(&pool, &m, &[x, y]).await.expect("groupe préalable");
    match lettrer(&pool, &m, &[a, y]).await {
        Err(DbError::LetteringLineOwnedByDocument {
            blocker,
            document_id,
            document_label,
        }) => {
            assert_eq!(blocker, ReversalBlocker::OwnedByInvoice);
            assert_eq!(document_id, Some(invoice));
            assert_eq!(document_label.as_deref(), Some("F-2026-014"));
        }
        other => panic!("attendu LetteringLineOwnedByDocument, obtenu {other:?}"),
    }
}

/// R5 — une écriture **rapprochée** d'une transaction bancaire n'est pas une
/// pièce : elle se lettre à la main.
#[sqlx::test(migrations = "./test-schema")]
async fn a_bank_matched_entry_is_not_a_document(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (entry, a) = ligne(&pool, &m, m.lettrable(), m.fy26, d(2026, 2, 1), dec!(100)).await;
    let (_, b) = ligne(&pool, &m, m.lettrable(), m.fy26, d(2026, 2, 2), dec!(-100)).await;
    let bank: i64 = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban) VALUES (?, 'B', 'CH9300762011623852957')",
    )
    .bind(m.company())
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let import: i64 = sqlx::query(
        "INSERT INTO bank_imports (company_id, bank_account_id, filename, file_hash, \
         source_format, period_from, period_to, imported_by_user_id) \
         VALUES (?, ?, 'x.xml', REPEAT('a', 64), 'camt053', '2026-02-01', '2026-02-28', ?)",
    )
    .bind(m.company())
    .bind(bank)
    .bind(m.s.admin_user_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    sqlx::query(
        "INSERT INTO bank_transactions (company_id, import_id, bank_account_id, booking_date, \
         amount, currency, details, status, matched_entry_id) \
         VALUES (?, ?, ?, '2026-02-01', 100, 'CHF', 'virement', 'reconciled', ?)",
    )
    .bind(m.company())
    .bind(import)
    .bind(bank)
    .bind(entry)
    .execute(&pool)
    .await
    .unwrap();
    // Le montage a bien produit le motif `MatchedBankTransaction`.
    let mut conn = pool.acquire().await.unwrap();
    let motifs =
        kesh_db::repositories::journal_entries::reversal_blockers(&mut *conn, m.company(), entry)
            .await
            .unwrap();
    assert!(
        motifs
            .iter()
            .any(|(b, _, _)| *b == ReversalBlocker::MatchedBankTransaction)
    );
    drop(conn);
    lettrer(&pool, &m, &[a, b])
        .await
        .expect("une écriture rapprochée se lettre à la main");
}

/// Rang 6 avant rang 7 : une ligne déjà lettrée, et une somme non nulle. Le
/// refus porte le code du groupe existant.
#[sqlx::test(migrations = "./test-schema")]
async fn rank_6_line_already_lettered(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (x, y) = paire(&pool, &m, (m.fy26, d(2026, 2, 1)), (m.fy26, d(2026, 2, 2))).await;
    let g = lettrer(&pool, &m, &[x, y]).await.unwrap();
    let (_, z) = ligne(&pool, &m, m.lettrable(), m.fy26, d(2026, 2, 3), dec!(-7)).await;
    match lettrer(&pool, &m, &[y, z]).await {
        Err(DbError::LetteringLineAlreadyLettered { code }) => assert_eq!(code, g.code),
        other => panic!("attendu LetteringLineAlreadyLettered, obtenu {other:?}"),
    }
    // AC10 d'août : la ligne ne « ré-apparaît » pas dans un second groupe.
    assert_eq!(marque(&pool, y).await.0, Some(g.key));
    assert_eq!(marque(&pool, z).await, (None, None));
}

/// Rang 7 : la somme exacte, et l'écart signé.
#[sqlx::test(migrations = "./test-schema")]
async fn rank_7_unbalanced(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (_, a) = ligne(&pool, &m, m.lettrable(), m.fy26, d(2026, 2, 1), dec!(100)).await;
    let (_, b) = ligne(
        &pool,
        &m,
        m.lettrable(),
        m.fy26,
        d(2026, 2, 2),
        dec!(-99.99),
    )
    .await;
    match lettrer(&pool, &m, &[a, b]).await {
        Err(DbError::LetteringUnbalanced { difference }) => assert_eq!(difference, dec!(0.01)),
        other => panic!("attendu LetteringUnbalanced, obtenu {other:?}"),
    }
    assert_eq!(marque(&pool, a).await, (None, None));
}

// ---------------------------------------------------------------------------
// AC4 — la règle des périodes au lettrage
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn lettering_across_closed_and_open_is_accepted(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (a, b) = paire(
        &pool,
        &m,
        (m.fy25, d(2025, 12, 1)),
        (m.fy26, d(2026, 1, 15)),
    )
    .await;
    set_status(&pool, m.fy25, "Closed").await;
    lettrer(&pool, &m, &[a, b])
        .await
        .expect("permis à cheval : une ligne en période ouverte suffit");
}

#[sqlx::test(migrations = "./test-schema")]
async fn lettering_all_in_closed_years_is_refused(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (a, b) = paire(&pool, &m, (m.fy25, d(2025, 2, 1)), (m.fy25, d(2025, 3, 1))).await;
    set_status(&pool, m.fy25, "Closed").await;
    assert!(matches!(
        lettrer(&pool, &m, &[a, b]).await,
        Err(DbError::LetteringAllLinesInClosedPeriods)
    ));
}

#[sqlx::test(migrations = "./test-schema")]
async fn lettering_all_in_locked_period_is_refused(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (a, b) = paire(
        &pool,
        &m,
        (m.fy26, d(2026, 3, 15)),
        (m.fy26, d(2026, 3, 20)),
    )
    .await;
    poser_borne(&pool, m.company(), d(2026, 3, 31)).await;
    assert!(matches!(
        lettrer(&pool, &m, &[a, b]).await,
        Err(DbError::LetteringAllLinesInClosedPeriods)
    ));
}

/// Seuil inclusif : une ligne datée **exactement** de la borne est close, celle
/// du lendemain est ouverte.
#[sqlx::test(migrations = "./test-schema")]
async fn lettering_at_the_lock_boundary(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (a, b) = paire(
        &pool,
        &m,
        (m.fy26, d(2026, 3, 31)),
        (m.fy26, d(2026, 3, 31)),
    )
    .await;
    let (c, e) = paire(&pool, &m, (m.fy26, d(2026, 3, 31)), (m.fy26, d(2026, 4, 1))).await;
    poser_borne(&pool, m.company(), d(2026, 3, 31)).await;
    assert!(matches!(
        lettrer(&pool, &m, &[a, b]).await,
        Err(DbError::LetteringAllLinesInClosedPeriods)
    ));
    lettrer(&pool, &m, &[c, e])
        .await
        .expect("le lendemain de la borne est ouvert");
}

#[sqlx::test(migrations = "./test-schema")]
async fn lettering_across_locked_and_unlocked_is_accepted(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (a, b) = paire(&pool, &m, (m.fy26, d(2026, 3, 15)), (m.fy26, d(2026, 5, 1))).await;
    poser_borne(&pool, m.company(), d(2026, 3, 31)).await;
    lettrer(&pool, &m, &[a, b])
        .await
        .expect("à cheval sur la borne");
}

/// État hérité « N ouvert, N+1 clos » posé par SQL direct : un groupe tout
/// entier dans N est refusé — lu par la lecture **non verrouillante** du (ii).
#[sqlx::test(migrations = "./test-schema")]
async fn lettering_under_a_later_closed_year_is_refused(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (a, b) = paire(&pool, &m, (m.fy25, d(2025, 2, 1)), (m.fy25, d(2025, 3, 1))).await;
    set_status(&pool, m.fy26, "Closed").await;
    assert!(matches!(
        lettrer(&pool, &m, &[a, b]).await,
        Err(DbError::LetteringAllLinesInClosedPeriods)
    ));
    // Une ligne de 2027 (ouvert, aucun postérieur clos) rend le groupe permis.
    let (_, c) = ligne(&pool, &m, m.lettrable(), m.fy27, d(2027, 1, 5), dec!(-100)).await;
    let (_, e) = ligne(&pool, &m, m.lettrable(), m.fy25, d(2025, 4, 1), dec!(100)).await;
    lettrer(&pool, &m, &[e, c])
        .await
        .expect("une ligne en 2027 suffit");
}

// ---------------------------------------------------------------------------
// AC5 — la dissolution
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn dissolution_of_a_manual_group_clears_every_mark(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (a, b) = paire(&pool, &m, (m.fy26, d(2026, 2, 1)), (m.fy26, d(2026, 2, 2))).await;
    let g = lettrer(&pool, &m, &[a, b]).await.unwrap();
    let dissous = delettrer(&pool, &m, g.key).await.expect("délettrage");
    assert_eq!(dissous.key, g.key);
    assert_eq!(dissous.origin, Origin::Manual);
    assert_eq!(marque(&pool, a).await, (None, None));
    assert_eq!(marque(&pool, b).await, (None, None));
    // Une seconde fois : le groupe n'existe plus.
    assert!(matches!(
        delettrer(&pool, &m, g.key).await,
        Err(DbError::NotFound)
    ));
    // Et les lignes se relettrent — sous le MÊME code (réémission, R2).
    let g2 = lettrer(&pool, &m, &[a, b]).await.unwrap();
    assert_eq!(g2.code, g.code);
}

/// Refus 1 — le lettrage d'une pièce suit la pièce.
#[sqlx::test(migrations = "./test-schema")]
async fn dissolution_of_a_document_group_is_refused(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (a, b) = paire(&pool, &m, (m.fy26, d(2026, 2, 1)), (m.fy26, d(2026, 2, 2))).await;
    let g = lettrer_avec(
        &pool,
        &m,
        &[a, b],
        Origin::Document,
        Mode::System {
            held_open_fiscal_year_id: m.fy26,
        },
    )
    .await
    .unwrap();
    assert!(matches!(
        delettrer(&pool, &m, g.key).await,
        Err(DbError::LetteringIsDocument)
    ));
    assert_eq!(marque(&pool, a).await.0, Some(g.key), "intact");
}

/// Refus 2 — une paire `reversal` dont une ligne est celle d'une pièce ; et
/// branche 4 — une paire `reversal` sans pièce se dissout.
#[sqlx::test(migrations = "./test-schema")]
async fn dissolution_of_a_reversal_pair_with_a_document_line_is_refused(pool: MySqlPool) {
    let m = monde(&pool).await;
    let systeme = Mode::System {
        held_open_fiscal_year_id: m.fy26,
    };
    let (entry, a) = ligne(&pool, &m, m.lettrable(), m.fy26, d(2026, 2, 1), dec!(100)).await;
    let (_, b) = ligne(&pool, &m, m.lettrable(), m.fy26, d(2026, 2, 2), dec!(-100)).await;
    let invoice = faire_facture(&pool, &m, entry, "F-2026-020").await;
    let g = lettrer_avec(&pool, &m, &[a, b], Origin::Reversal, systeme)
        .await
        .expect("en mode System, la cause 5 ne s'applique pas");
    match delettrer(&pool, &m, g.key).await {
        Err(DbError::LetteringLineOwnedByDocument { document_id, .. }) => {
            assert_eq!(document_id, Some(invoice))
        }
        other => panic!("attendu LetteringLineOwnedByDocument, obtenu {other:?}"),
    }

    let (c, e) = paire(&pool, &m, (m.fy26, d(2026, 3, 1)), (m.fy26, d(2026, 3, 2))).await;
    let g2 = lettrer_avec(&pool, &m, &[c, e], Origin::Reversal, systeme)
        .await
        .unwrap();
    let dissous = delettrer(&pool, &m, g2.key).await.expect("branche 4");
    assert_eq!(dissous.origin, Origin::Reversal);
}

#[sqlx::test(migrations = "./test-schema")]
async fn dissolution_all_in_closed_years_is_refused(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (a, b) = paire(&pool, &m, (m.fy25, d(2025, 2, 1)), (m.fy25, d(2025, 3, 1))).await;
    let g = lettrer(&pool, &m, &[a, b]).await.unwrap();
    set_status(&pool, m.fy25, "Closed").await;
    assert!(matches!(
        delettrer(&pool, &m, g.key).await,
        Err(DbError::LetteringAllLinesInClosedPeriods)
    ));
}

#[sqlx::test(migrations = "./test-schema")]
async fn dissolution_all_in_locked_period_is_refused(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (a, b) = paire(&pool, &m, (m.fy26, d(2026, 3, 1)), (m.fy26, d(2026, 3, 31))).await;
    let g = lettrer(&pool, &m, &[a, b]).await.unwrap();
    poser_borne(&pool, m.company(), d(2026, 3, 31)).await;
    assert!(matches!(
        delettrer(&pool, &m, g.key).await,
        Err(DbError::LetteringAllLinesInClosedPeriods)
    ));
}

#[sqlx::test(migrations = "./test-schema")]
async fn dissolution_under_a_later_closed_year_is_refused(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (a, b) = paire(&pool, &m, (m.fy25, d(2025, 2, 1)), (m.fy25, d(2025, 3, 1))).await;
    let g = lettrer(&pool, &m, &[a, b]).await.unwrap();
    set_status(&pool, m.fy26, "Closed").await;
    assert!(matches!(
        delettrer(&pool, &m, g.key).await,
        Err(DbError::LetteringAllLinesInClosedPeriods)
    ));
}

/// C104 — la dissolution n'exige jamais la lettrabilité : retypé, ou rattaché
/// à un compte bancaire depuis, le groupe se dissout.
#[sqlx::test(migrations = "./test-schema")]
async fn dissolution_does_not_require_a_letterable_account(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (a, b) = paire(&pool, &m, (m.fy26, d(2026, 2, 1)), (m.fy26, d(2026, 2, 2))).await;
    let (c, e) = paire(&pool, &m, (m.fy26, d(2026, 2, 3)), (m.fy26, d(2026, 2, 4))).await;
    let g1 = lettrer(&pool, &m, &[a, b]).await.unwrap();
    let g2 = lettrer(&pool, &m, &[c, e]).await.unwrap();

    sqlx::query("UPDATE accounts SET account_type = 'Expense' WHERE id = ?")
        .bind(m.lettrable())
        .execute(&pool)
        .await
        .unwrap();
    delettrer(&pool, &m, g1.key).await.expect("compte retypé");

    sqlx::query("UPDATE accounts SET account_type = 'Asset' WHERE id = ?")
        .bind(m.lettrable())
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, journal_account_id) \
         VALUES (?, 'Banque', 'CH9300762011623852957', ?)",
    )
    .bind(m.company())
    .bind(m.lettrable())
    .execute(&pool)
    .await
    .unwrap();
    delettrer(&pool, &m, g2.key)
        .await
        .expect("compte rattaché à un compte bancaire");
}

/// AC11 — la clé d'une autre société est introuvable au délettrage comme à la
/// lecture.
#[sqlx::test(migrations = "./test-schema")]
async fn a_foreign_group_is_not_found(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (a, b) = paire(&pool, &m, (m.fy26, d(2026, 2, 1)), (m.fy26, d(2026, 2, 2))).await;
    let g = lettrer(&pool, &m, &[a, b]).await.unwrap();
    let (x, _) = autre_societe(&pool, &m).await;
    let other: i64 = sqlx::query_scalar(
        "SELECT je.company_id FROM journal_entry_lines jel JOIN journal_entries je \
         ON je.id = jel.entry_id WHERE jel.id = ?",
    )
    .bind(x)
    .fetch_one(&pool)
    .await
    .unwrap();
    let mut tx = pool.begin().await.unwrap();
    let r = letterings::dissolve_group_in_tx(&mut tx, other, g.key, Mode::Manual, m.actor()).await;
    assert!(matches!(r, Err(DbError::NotFound)));
    drop(tx);
    let mut conn = pool.acquire().await.unwrap();
    assert!(
        letterings::find_group(&mut conn, other, g.key)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        letterings::find_group(&mut conn, m.company(), g.key)
            .await
            .unwrap()
            .is_some()
    );
}

// ---------------------------------------------------------------------------
// R7 point 3 — le mode System
// ---------------------------------------------------------------------------

/// L'exercice tenu doit couvrir une ligne du groupe — sinon `Invariant`, défaut
/// de l'appelant ; et la réponse porte le nom des exercices (C128).
#[sqlx::test(migrations = "./test-schema")]
async fn system_mode_requires_the_held_year_and_reports_names(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (a, b) = paire(&pool, &m, (m.fy25, d(2025, 6, 1)), (m.fy26, d(2026, 1, 5))).await;
    let r = lettrer_avec(
        &pool,
        &m,
        &[a, b],
        Origin::Reversal,
        Mode::System {
            held_open_fiscal_year_id: m.fy27,
        },
    )
    .await;
    assert!(matches!(r, Err(DbError::Invariant(_))), "{r:?}");

    // En mode System, ni la règle des périodes (2025 clos) ni la cause 5.
    set_status(&pool, m.fy25, "Closed").await;
    let g = lettrer_avec(
        &pool,
        &m,
        &[a, b],
        Origin::Reversal,
        Mode::System {
            held_open_fiscal_year_id: m.fy26,
        },
    )
    .await
    .expect("mode System");
    let noms: Vec<&str> = g
        .lines
        .iter()
        .map(|l| l.fiscal_year_name.as_str())
        .collect();
    assert_eq!(noms, vec!["Exercice 2025", "Exercice 2026"]);
    let details: serde_json::Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'lettering.created' AND entity_id = ?",
    )
    .bind(g.key)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(details["lines"][0]["fiscalYearName"], "Exercice 2025");
    assert_eq!(details["origin"], "reversal");

    // Dissolution System : l'exercice tenu doit aussi couvrir une ligne.
    let mut tx = pool.begin().await.unwrap();
    let r = letterings::dissolve_group_in_tx(
        &mut tx,
        m.company(),
        g.key,
        Mode::System {
            held_open_fiscal_year_id: m.fy27,
        },
        m.actor(),
    )
    .await;
    assert!(matches!(r, Err(DbError::Invariant(_))), "{r:?}");
}

// ---------------------------------------------------------------------------
// AC10 — l'audit
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn lettering_created_is_audited_with_its_lines(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (a, b) = paire(
        &pool,
        &m,
        (m.fy25, d(2025, 12, 30)),
        (m.fy26, d(2026, 1, 5)),
    )
    .await;
    let g = lettrer(&pool, &m, &[a, b]).await.unwrap();
    let (user_id, entity_type, actor_type, details): (i64, String, String, serde_json::Value) =
        sqlx::query_as(
            "SELECT user_id, entity_type, actor_type, details_json FROM audit_log \
             WHERE action = 'lettering.created' AND entity_id = ?",
        )
        .bind(g.key)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(user_id, m.s.admin_user_id);
    assert_eq!(entity_type, "lettering");
    assert_eq!(actor_type, "user");
    assert_eq!(details["code"], g.code);
    assert_eq!(details["origin"], "manual");
    assert_eq!(details["accountId"], m.lettrable());
    assert_eq!(details["accountNumber"], "1100");
    let lignes = details["lines"].as_array().unwrap();
    assert_eq!(lignes.len(), 2);
    assert_eq!(lignes[0]["id"], a);
    assert_eq!(lignes[0]["fiscalYearId"], m.fy25);
    assert_eq!(lignes[0]["fiscalYearName"], "Exercice 2025");
    assert_eq!(lignes[1]["fiscalYearName"], "Exercice 2026");
    assert_eq!(lignes[0]["entryNumber"], 1);
    assert_eq!(lignes[0]["debit"], "100.0000");
    assert_eq!(lignes[1]["credit"], "100.0000");
    for champ in ["entryId", "entryNumber", "debit", "credit"] {
        assert!(lignes[1].get(champ).is_some(), "{champ}");
    }
}

#[sqlx::test(migrations = "./test-schema")]
async fn lettering_removed_is_audited(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (a, b) = paire(&pool, &m, (m.fy26, d(2026, 2, 1)), (m.fy26, d(2026, 2, 2))).await;
    let g = lettrer(&pool, &m, &[a, b]).await.unwrap();
    delettrer(&pool, &m, g.key).await.unwrap();
    let (entity_type, details): (String, serde_json::Value) = sqlx::query_as(
        "SELECT entity_type, details_json FROM audit_log \
         WHERE action = 'lettering.removed' AND entity_id = ?",
    )
    .bind(g.key)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(entity_type, "lettering");
    assert_eq!(details["code"], g.code);
    assert_eq!(details["lines"].as_array().unwrap().len(), 2);
    // Un refus n'écrit rien.
    let n: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE action LIKE 'lettering.%'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(n, 2);
}

// ---------------------------------------------------------------------------
// AC13 — l'invariant des groupes
// ---------------------------------------------------------------------------

/// Les clés des groupes qui violent l'invariant d'AC13 (toutes sociétés).
async fn violations_des_groupes(pool: &MySqlPool) -> Vec<i64> {
    sqlx::query_scalar(
        "SELECT jel.lettering_key FROM journal_entry_lines jel \
         JOIN journal_entries je ON je.id = jel.entry_id \
         WHERE jel.lettering_key IS NOT NULL \
         GROUP BY jel.lettering_key \
         HAVING COUNT(*) < 2 \
             OR COUNT(DISTINCT jel.account_id) <> 1 \
             OR COUNT(DISTINCT jel.lettering_origin) <> 1 \
             OR COUNT(DISTINCT je.company_id) <> 1 \
             OR SUM(jel.debit - jel.credit) <> 0 \
             OR MIN(jel.id) <> jel.lettering_key",
    )
    .fetch_all(pool)
    .await
    .unwrap()
}

/// Tout groupe a ≥ 2 lignes, un seul compte, une seule origine, une seule
/// société, une somme nulle, et `lettering_key = MIN(id)`. ⛔ La lettrabilité
/// n'est PAS contrôlée, par décision (C104). La Story 15-1a-ii y ajoute des
/// groupes `reversal` posés par la contre-passation (AC13 part ii) — dont une
/// écriture à une ligne déjà lettrée (R6, seconde branche : son groupe reste,
/// son miroir reste ouvert) ; un groupe `reversal`
/// posé en mode `System` et un groupe d'une seconde société y figurent aussi,
/// et deux contrôles négatifs prouvent que les clauses « une origine » et
/// « une société » rougissent (revue P1, E-6).
#[sqlx::test(migrations = "./test-schema")]
async fn lettering_invariants(pool: MySqlPool) {
    let m = monde(&pool).await;
    // Un scénario mêlé : groupes à deux et trois lignes, à cheval, un délettré
    // puis relettré, un groupe sur un passif, des lignes restées ouvertes.
    let (a, b) = paire(&pool, &m, (m.fy25, d(2025, 11, 1)), (m.fy26, d(2026, 1, 3))).await;
    lettrer(&pool, &m, &[a, b]).await.unwrap();
    let (_, c) = ligne(&pool, &m, m.lettrable(), m.fy26, d(2026, 2, 1), dec!(300)).await;
    let (_, e) = ligne(
        &pool,
        &m,
        m.lettrable(),
        m.fy26,
        d(2026, 2, 2),
        dec!(-120.5),
    )
    .await;
    let (_, f) = ligne(
        &pool,
        &m,
        m.lettrable(),
        m.fy26,
        d(2026, 2, 3),
        dec!(-179.5),
    )
    .await;
    let g = lettrer(&pool, &m, &[f, e, c]).await.unwrap();
    delettrer(&pool, &m, g.key).await.unwrap();
    lettrer(&pool, &m, &[c, e, f]).await.unwrap();
    let passif = m.s.accounts["2000"];
    let (_, h) = ligne(&pool, &m, passif, m.fy26, d(2026, 3, 1), dec!(-50)).await;
    let (_, i) = ligne(&pool, &m, passif, m.fy26, d(2026, 3, 2), dec!(50)).await;
    lettrer(&pool, &m, &[h, i]).await.unwrap();
    paire(&pool, &m, (m.fy26, d(2026, 4, 1)), (m.fy26, d(2026, 4, 2))).await;
    // Un groupe d'une autre origine (mode System, comme la 15-1a-ii le posera)
    // et un groupe dans une seconde société : sans eux, les clauses « une
    // seule origine » et « une seule société » portaient sur une seule valeur
    // possible (revue P1, E-6 / A-L6).
    let (_, r1) = ligne(&pool, &m, passif, m.fy26, d(2026, 5, 1), dec!(70)).await;
    let (_, r2) = ligne(&pool, &m, passif, m.fy26, d(2026, 5, 2), dec!(-70)).await;
    lettrer_avec(
        &pool,
        &m,
        &[r1, r2],
        Origin::Reversal,
        Mode::System {
            held_open_fiscal_year_id: m.fy26,
        },
    )
    .await
    .unwrap();
    let (x, y) = autre_societe(&pool, &m).await;
    let (other, other_fy): (i64, i64) = sqlx::query_as(
        "SELECT je.company_id, je.fiscal_year_id FROM journal_entry_lines jel \
         JOIN journal_entries je ON je.id = jel.entry_id WHERE jel.id = ?",
    )
    .bind(x)
    .fetch_one(&pool)
    .await
    .unwrap();
    let mut tx = pool.begin().await.unwrap();
    letterings::create_group_in_tx(
        &mut tx,
        other,
        &[x, y],
        Origin::Manual,
        Mode::Manual,
        m.actor(),
    )
    .await
    .expect("groupe de la seconde société");
    tx.commit().await.unwrap();

    // AC13 part (ii) — la contre-passation lettre (R6). Une écriture à deux
    // lignes lettrables (1100, 2000) → deux groupes `reversal` ; une seconde,
    // dont la ligne 1100 est déjà lettrée `manual`, → un seul groupe `reversal`
    // (2000), le groupe manuel intact et le miroir 1100 ouvert.
    let annee = chrono::Datelike::year(&chrono::Utc::now().date_naive());
    if !(2025..=2027).contains(&annee) {
        exercice(&pool, m.company(), annee, "Open").await;
    }
    let (libre, _) = ecriture(
        &pool,
        m.company(),
        m.fy26,
        d(2026, 6, 1),
        &[
            (m.lettrable(), dec!(40), Decimal::ZERO),
            (passif, Decimal::ZERO, dec!(40)),
        ],
    )
    .await;
    let (en_partie, lignes_ep) = ecriture(
        &pool,
        m.company(),
        m.fy26,
        d(2026, 6, 2),
        &[
            (m.lettrable(), dec!(60), Decimal::ZERO),
            (passif, Decimal::ZERO, dec!(60)),
        ],
    )
    .await;
    let (_, lignes_contre) = ecriture(
        &pool,
        m.company(),
        m.fy26,
        d(2026, 6, 3),
        &[
            (passif, dec!(60), Decimal::ZERO),
            (m.lettrable(), Decimal::ZERO, dec!(60)),
        ],
    )
    .await;
    let manuel = lettrer(&pool, &m, &[lignes_ep[0], lignes_contre[1]])
        .await
        .unwrap();
    for origine in [libre, en_partie] {
        kesh_db::repositories::journal_entries::reverse(
            &pool,
            m.company(),
            origine,
            m.s.admin_user_id,
        )
        .await
        .expect("contre-passation");
    }
    let cle_1100: Option<i64> =
        sqlx::query_scalar("SELECT lettering_key FROM journal_entry_lines WHERE id = ?")
            .bind(lignes_ep[0])
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(cle_1100, Some(manuel.key), "le groupe manuel reste");

    let groupes: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT lettering_key) FROM journal_entry_lines WHERE lettering_key IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(groupes, 9, "5 du scénario, 2 + 1 reversal, 1 manuel (R6)");
    let (origines, societes): (i64, i64) = sqlx::query_as(
        "SELECT COUNT(DISTINCT jel.lettering_origin), COUNT(DISTINCT je.company_id) \
         FROM journal_entry_lines jel JOIN journal_entries je ON je.id = jel.entry_id \
         WHERE jel.lettering_key IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((origines, societes), (2, 2), "deux origines, deux sociétés");

    assert!(
        violations_des_groupes(&pool).await.is_empty(),
        "groupes invalides : {:?}",
        violations_des_groupes(&pool).await
    );

    // Contrôles négatifs : chaque clause qu'un montage ordinaire ne peut
    // violer rougit sur une violation isolée, posée en SQL direct puis
    // défaite — sans quoi la requête pourrait être vraie par construction.
    // (1) Une seule origine : une ligne d'un groupe manuel passe `reversal`.
    sqlx::query("UPDATE journal_entry_lines SET lettering_origin = 'reversal' WHERE id = ?")
        .bind(c)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        violations_des_groupes(&pool).await.len(),
        1,
        "clause des origines"
    );
    sqlx::query("UPDATE journal_entry_lines SET lettering_origin = 'manual' WHERE id = ?")
        .bind(c)
        .execute(&pool)
        .await
        .unwrap();
    // (2) Une seule société : une écriture de la seconde société, sur le compte
    // lettrable de la première (donnée corrompue que les FK admettent), à
    // somme nulle, rattachée au groupe {a, b} — même compte, même origine,
    // somme nulle, `MIN(id)` inchangé : seule la clause des sociétés la voit.
    let (intrus_ecriture, intrus) = ecriture(
        &pool,
        other,
        other_fy,
        d(2026, 6, 1),
        &[
            (m.lettrable(), dec!(50), dec!(0)),
            (m.lettrable(), dec!(0), dec!(50)),
        ],
    )
    .await;
    let cle_ab = marque(&pool, a).await.0.expect("groupe {a, b}");
    for id in &intrus {
        sqlx::query(
            "UPDATE journal_entry_lines SET lettering_key = ?, lettering_origin = 'manual' \
             WHERE id = ?",
        )
        .bind(cle_ab)
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
    }
    assert_eq!(
        violations_des_groupes(&pool).await,
        vec![cle_ab],
        "clause des sociétés"
    );
    // Les deux `Invariant` de la remédiation P1 (E-2), atteints sur cette même
    // donnée corrompue (revue de code P2, A2-3). Lu par la seconde société, le
    // groupe se réduit aux intrus : (a) leur compte est celui de la première
    // société → `group_account_number` refuse ; (b) leur écriture passée sur
    // un exercice de la première société → `fiscal_year_names` refuse, avant
    // le compte.
    let mut conn = pool.acquire().await.unwrap();
    let r = letterings::find_group(&mut conn, other, cle_ab).await;
    assert!(
        matches!(&r, Err(DbError::Invariant(msg)) if msg.contains("compte")),
        "branche du compte : {r:?}"
    );
    sqlx::query("UPDATE journal_entries SET fiscal_year_id = ? WHERE id = ?")
        .bind(m.fy26)
        .bind(intrus_ecriture)
        .execute(&pool)
        .await
        .unwrap();
    let r = letterings::find_group(&mut conn, other, cle_ab).await;
    assert!(
        matches!(&r, Err(DbError::Invariant(msg)) if msg.contains("exercice")),
        "branche de l'exercice : {r:?}"
    );
    sqlx::query("UPDATE journal_entries SET fiscal_year_id = ? WHERE id = ?")
        .bind(other_fy)
        .bind(intrus_ecriture)
        .execute(&pool)
        .await
        .unwrap();
    drop(conn);
    sqlx::query(
        "UPDATE journal_entry_lines SET lettering_key = NULL, lettering_origin = NULL \
         WHERE lettering_key = ? AND id IN (?, ?)",
    )
    .bind(cle_ab)
    .bind(intrus[0])
    .bind(intrus[1])
    .execute(&pool)
    .await
    .unwrap();
    assert!(violations_des_groupes(&pool).await.is_empty());
    let demi: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM journal_entry_lines \
         WHERE (lettering_key IS NULL) <> (lettering_origin IS NULL)",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(demi, 0);
}

// ---------------------------------------------------------------------------
// Concurrence (R7 point 2)
// ---------------------------------------------------------------------------

/// Sérialisation avec la clôture : la dissolution, dans une transaction tenue
/// ouverte, a passé R7 point 2 ; la clôture de l'exercice **attend** (étape
/// (c), verrou de Y par clé primaire — sans antérieur, l'étape (b') ne prend
/// rien), la dissolution valide, la clôture aboutit. Ce test ne prétend rien
/// de l'ordre des verrous.
#[sqlx::test(migrations = "./test-schema")]
async fn dissolution_serializes_with_the_closing_of_its_year(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (a, b) = paire(&pool, &m, (m.fy25, d(2025, 2, 1)), (m.fy25, d(2025, 3, 1))).await;
    let g = lettrer(&pool, &m, &[a, b]).await.unwrap();

    let mut tx = pool.begin().await.unwrap();
    letterings::dissolve_group_in_tx(&mut tx, m.company(), g.key, Mode::Manual, m.actor())
        .await
        .expect("dissolution");

    let cloture = {
        let pool = pool.clone();
        let (user, company, fy) = (m.s.admin_user_id, m.company(), m.fy25);
        tokio::spawn(async move {
            kesh_db::repositories::fiscal_years::close(&pool, user, company, fy).await
        })
    };
    let vue = attendre_une_requete_en_cours(
        &pool,
        &["SELECT id, company_id", "WHERE id = ", "FOR UPDATE"],
        || cloture.is_finished(),
    )
    .await;
    assert!(
        vue,
        "la clôture devait attendre l'exercice tenu par la dissolution"
    );
    tx.commit().await.unwrap();
    let fy = cloture
        .await
        .unwrap()
        .expect("la clôture aboutit après la dissolution");
    assert_eq!(fy.status.as_str(), "Closed");
    assert_eq!(marque(&pool, a).await, (None, None));
}

/// Ordre d'acquisition : A (antérieur, `id` HAUT, créé après coup) et B
/// (postérieur, `id` bas). T tient A ; la dissolution — appel direct, hors
/// enveloppe de rejeu — bute sur A, vue en attente deux fois à 100 ms d'écart ;
/// une sonde `NOWAIT` sur B réussit : la dissolution n'a rien pris au-delà de A.
///
/// **Mutation tuée** : « trier par `id` au lieu de `start_date` » — la
/// dissolution prend B puis bute sur A, et la sonde rend `false` (1205).
#[sqlx::test(migrations = "./test-schema")]
async fn lettering_locks_fiscal_years_in_date_order_not_id_order(pool: MySqlPool) {
    let m = monde(&pool).await;
    let fy24 = exercice(&pool, m.company(), 2024, "Open").await;
    let (fy_a, fy_b) = (fy24, m.fy25);
    assert!(fy_a > fy_b, "A doit avoir l'id le plus haut");
    let (a, b) = paire(&pool, &m, (fy_a, d(2024, 6, 1)), (fy_b, d(2025, 6, 1))).await;
    let g = lettrer(&pool, &m, &[a, b]).await.unwrap();

    let mut t = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE")
        .bind(fy_a)
        .fetch_one(&mut *t)
        .await
        .unwrap();

    let dissolution = {
        let pool = pool.clone();
        let (company, key, actor) = (m.company(), g.key, m.actor());
        tokio::spawn(async move {
            let mut tx = pool.begin().await.unwrap();
            let r =
                letterings::dissolve_group_in_tx(&mut tx, company, key, Mode::Manual, actor).await;
            if r.is_ok() {
                tx.commit().await.unwrap();
            }
            r.map(|g| g.key)
        })
    };
    let motifs = [
        "SELECT id, start_date, status FROM fiscal_years WHERE id = ",
        "FOR UPDATE",
    ];
    assert!(attendre_une_requete_en_cours(&pool, &motifs, || dissolution.is_finished()).await);
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    assert!(
        attendre_une_requete_en_cours(&pool, &motifs, || dissolution.is_finished()).await,
        "la requête encore en cours 100 ms plus tard est celle qui attend A"
    );
    assert!(
        sonde_verrou_nowait(
            &pool,
            "SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE NOWAIT",
            fy_b
        )
        .await,
        "la dissolution ne doit rien tenir au-delà de A : elle verrouille dans l'ordre des dates"
    );
    t.rollback().await.unwrap();
    assert_eq!(dissolution.await.unwrap().expect("dissolution"), g.key);
}
