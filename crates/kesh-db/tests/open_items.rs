//! Tests de dépôt — Story 15-1b (#518) : la vue des postes ouverts d'un compte
//! à une date (`letterings::open_items`) et le chargement des propositions
//! (`letterings::lettering_proposals`).
//!
//! Couvre les tests de T6 qui se jugent au dépôt — 4 (pagination et tri du
//! Grand livre), 6 (`document` des cinq types), 7 (`manuallyLetterable`), 9
//! (`reason`), 10 (`documentState` et `amountDue`), 12 et 13 (chargement des
//! candidates, plafond), 19 (AC12 : ce qui est stable « au X » et ce qui ne
//! l'est pas), 20 (champs de la vue) —, et l'`EXPLAIN` d'AC8 (test ignoré,
//! lancé à la main, noté au Dev Agent Record). L'invariant contre la Balance
//! (tests 1 et 2) est dans `crates/kesh-report/tests/open_items_invariant.rs`,
//! la garde lexicale (test 3) au module, le moteur pur (11, 14) dans
//! `kesh-core`, la frontière HTTP (5, 15-18, 20) dans
//! `crates/kesh-api/tests/open_items_e2e.rs`.
//!
//! ⚠️ Les états hérités (marques effacées, données hors des gestes) se
//! fabriquent en SQL brut, comme dans `lettering_documents.rs` ; aucune marque
//! n'est jamais POSÉE autrement que par la primitive.

use chrono::NaiveDate;
use kesh_db::errors::{DbError, SettlementCancelBlocker};
use kesh_db::repositories::journal_entries::DocumentKind;
use kesh_db::repositories::letterings::{
    self, Actor, DocumentState, Mode, OpenItem, OpenItems, OpenReason, Origin,
};
use kesh_db::repositories::{companies, invoice_settlements, invoice_settlements_write};
use kesh_db::test_fixtures::SeededCompany;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

#[path = "support/lettering_documents.rs"]
mod lettering_support;
use lettering_support::*;

// ---------------------------------------------------------------------------
// Montage
// ---------------------------------------------------------------------------

fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn aujourdhui() -> NaiveDate {
    chrono::Utc::now().date_naive()
}

/// Une société prête à facturer (support de la 15-1a2-i) et deux exercices :
/// **2024** (`fy24`, ouvert — les tests le clôturent) et **2025-2030** (`fy`,
/// celui du seed ramené au 1er janvier 2025 ; toutes les dates `jours_avant(n)`
/// pour `n` < 280 y tombent).
struct Monde {
    s: SeededCompany,
    fy24: i64,
    fy: i64,
}

impl Monde {
    fn company(&self) -> i64 {
        self.s.company_id
    }
    /// La créance des tests (1100, lettrable, celle des factures).
    fn creance(&self) -> i64 {
        self.s.accounts["1100"]
    }
    fn actor(&self) -> Actor {
        Actor {
            user_id: self.s.admin_user_id,
            api_key_id: None,
        }
    }
}

async fn monde(pool: &MySqlPool) -> Monde {
    let s = societe(pool).await;
    sqlx::query(
        "UPDATE fiscal_years SET name = 'Exercice 2025-2030', start_date = '2025-01-01' \
         WHERE id = ?",
    )
    .bind(s.fiscal_year_id)
    .execute(pool)
    .await
    .unwrap();
    let fy24 = sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, 'Exercice 2024', '2024-01-01', '2024-12-31', 'Open')",
    )
    .bind(s.company_id)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    Monde {
        fy: s.fiscal_year_id,
        s,
        fy24,
    }
}

async fn statut(pool: &MySqlPool, fy: i64, status: &str) {
    sqlx::query("UPDATE fiscal_years SET status = ? WHERE id = ?")
        .bind(status)
        .bind(fy)
        .execute(pool)
        .await
        .unwrap();
}

/// Écriture posée en SQL direct ; `lignes` : `(compte, débit, crédit)`, posées
/// dans l'ordre avec `line_order` 1, 2, … ; rend `(écriture, ids des lignes)`.
async fn ecriture(
    pool: &MySqlPool,
    m: &Monde,
    fy: i64,
    date: NaiveDate,
    lignes: &[(i64, Decimal, Decimal)],
) -> (i64, Vec<i64>) {
    let numero: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(entry_number), 0) + 1 FROM journal_entries \
         WHERE company_id = ? AND fiscal_year_id = ?",
    )
    .bind(m.company())
    .bind(fy)
    .fetch_one(pool)
    .await
    .unwrap();
    let entry = sqlx::query(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, \
         journal, description) VALUES (?, ?, ?, ?, 'OD', 'postes ouverts')",
    )
    .bind(m.company())
    .bind(fy)
    .bind(numero)
    .bind(date)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let mut ids = Vec::new();
    for (n, (compte, debit, credit)) in lignes.iter().enumerate() {
        ids.push(
            sqlx::query(
                "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(entry)
            .bind(*compte)
            .bind(n as i32 + 1)
            .bind(*debit)
            .bind(*credit)
            .execute(pool)
            .await
            .unwrap()
            .last_insert_id() as i64,
        );
    }
    (entry, ids)
}

/// Une ligne sur la créance, au débit (`montant` > 0) ou au crédit (< 0), avec
/// sa contrepartie au 2000 ; rend `(écriture, ligne sur la créance)`.
async fn ligne(
    pool: &MySqlPool,
    m: &Monde,
    fy: i64,
    date: NaiveDate,
    montant: Decimal,
) -> (i64, i64) {
    let (d, c) = if montant > Decimal::ZERO {
        (montant, Decimal::ZERO)
    } else {
        (Decimal::ZERO, -montant)
    };
    let (e, ids) = ecriture(
        pool,
        m,
        fy,
        date,
        &[(m.creance(), d, c), (m.s.accounts["2000"], c, d)],
    )
    .await;
    (e, ids[0])
}

async fn lettrer(pool: &MySqlPool, m: &Monde, ids: &[i64]) -> i64 {
    let mut tx = pool.begin().await.unwrap();
    let g = letterings::create_group_in_tx(
        &mut tx,
        m.company(),
        ids,
        Origin::Manual,
        Mode::Manual,
        m.actor(),
    )
    .await
    .expect("lettrage manuel");
    tx.commit().await.unwrap();
    g.key
}

async fn delettrer(pool: &MySqlPool, m: &Monde, key: i64) {
    let mut tx = pool.begin().await.unwrap();
    letterings::dissolve_group_in_tx(&mut tx, m.company(), key, Mode::Manual, m.actor())
        .await
        .expect("délettrage manuel");
    tx.commit().await.unwrap();
}

async fn vue_page(
    pool: &MySqlPool,
    m: &Monde,
    compte: i64,
    x: NaiveDate,
    limit: i64,
    offset: i64,
) -> OpenItems {
    let mut c = pool.acquire().await.unwrap();
    letterings::open_items(&mut c, m.company(), compte, x, limit, offset)
        .await
        .expect("postes ouverts")
}

async fn vue(pool: &MySqlPool, m: &Monde, x: NaiveDate) -> OpenItems {
    vue_page(pool, m, m.creance(), x, 500, 0).await
}

/// La ligne `id` de la vue — panique si elle n'y est pas.
fn item(v: &OpenItems, id: i64) -> &OpenItem {
    v.items
        .iter()
        .find(|i| i.line_id == id)
        .unwrap_or_else(|| panic!("ligne {id} absente de la vue au {}", v.as_of))
}

fn absente(v: &OpenItems, id: i64) -> bool {
    v.items.iter().all(|i| i.line_id != id)
}

/// Ce que la vue liste, comparable avant et après un geste : lignes et montants.
fn contenu(v: &OpenItems) -> Vec<(i64, Decimal, Decimal)> {
    v.items
        .iter()
        .map(|i| (i.line_id, i.debit, i.credit))
        .collect()
}

/// L'invariant d'AC2 sur une vue, recalculé dans le test : `openTotal ==
/// balance == Σ(débit − crédit)` des lignes du compte datées ≤ X.
async fn invariant(pool: &MySqlPool, m: &Monde, v: &OpenItems) {
    let somme: Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(jel.debit - jel.credit), 0) FROM journal_entry_lines jel \
         JOIN journal_entries je ON je.id = jel.entry_id \
         WHERE jel.account_id = ? AND je.company_id = ? AND je.entry_date <= ?",
    )
    .bind(v.account_id)
    .bind(m.company())
    .bind(v.as_of)
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(
        v.open_total, v.balance,
        "openTotal == balance au {}",
        v.as_of
    );
    assert_eq!(v.balance, somme, "balance == Σ du test au {}", v.as_of);
}

/// La ligne de la créance d'une écriture (montage).
async fn ligne_creance(pool: &MySqlPool, m: &Monde, entry: i64) -> i64 {
    sqlx::query_scalar(
        "SELECT id FROM journal_entry_lines WHERE entry_id = ? AND account_id = ? ORDER BY id \
         LIMIT 1",
    )
    .bind(entry)
    .bind(m.creance())
    .fetch_one(pool)
    .await
    .expect("ligne de créance")
}

// ---------------------------------------------------------------------------
// Test 4 — pagination et tri du Grand livre
// ---------------------------------------------------------------------------

/// Test 4 (AC1) — `total`, `offset`, `limit` renvoyés ; `balance` et
/// `openTotal` identiques sur deux pages ; tri du Grand livre, dont deux lignes
/// d'une même écriture dont l'ordre d'`id` CONTREDIT l'ordre de `line_order`
/// (la ligne `line_order = 2` insérée avant la ligne `line_order = 1`) — un tri
/// par `lineId` seul échouerait.
#[sqlx::test(migrations = "./test-schema")]
async fn pagination_and_general_ledger_order(pool: MySqlPool) {
    let m = monde(&pool).await;
    let j = jours_avant(10);
    // Deux écritures du même jour, numéros 1 puis 2 de l'exercice 2025-2030.
    let (_, a) = ligne(&pool, &m, m.fy, j, dec!(10)).await;
    // Écriture à deux lignes sur la créance : line_order 2 posée d'abord.
    let (e, _) = ecriture(&pool, &m, m.fy, j, &[]).await;
    let inserer = |ordre: i32, debit: Decimal| {
        let pool = pool.clone();
        let compte = m.creance();
        async move {
            sqlx::query(
                "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
                 VALUES (?, ?, ?, ?, 0)",
            )
            .bind(e)
            .bind(compte)
            .bind(ordre)
            .bind(debit)
            .execute(&pool)
            .await
            .unwrap()
            .last_insert_id() as i64
        }
    };
    let deux = inserer(2, dec!(2)).await;
    let un = inserer(1, dec!(1)).await;
    assert!(deux < un, "montage : l'id contredit line_order");
    // Une ligne d'un jour antérieur, et une d'un exercice antérieur.
    let (_, ancienne) = ligne(&pool, &m, m.fy, jours_avant(20), dec!(5)).await;
    let (_, de_2024) = ligne(&pool, &m, m.fy24, ymd(2024, 6, 1), dec!(7)).await;

    let tout = vue(&pool, &m, aujourdhui()).await;
    let ordre: Vec<i64> = tout.items.iter().map(|i| i.line_id).collect();
    assert_eq!(ordre, vec![de_2024, ancienne, a, un, deux]);
    assert_eq!((tout.total, tout.offset, tout.limit), (5, 0, 500));
    invariant(&pool, &m, &tout).await;

    let p1 = vue_page(&pool, &m, m.creance(), aujourdhui(), 2, 0).await;
    let p2 = vue_page(&pool, &m, m.creance(), aujourdhui(), 2, 2).await;
    let p3 = vue_page(&pool, &m, m.creance(), aujourdhui(), 2, 4).await;
    assert_eq!((p1.total, p1.offset, p1.limit), (5, 0, 2));
    assert_eq!((p2.total, p2.offset, p2.limit), (5, 2, 2));
    assert_eq!((p1.balance, p1.open_total), (p2.balance, p2.open_total));
    assert_eq!((p1.balance, p1.open_total), (tout.balance, tout.open_total));
    let pages: Vec<i64> = [p1, p2, p3]
        .iter()
        .flat_map(|p| p.items.iter().map(|i| i.line_id))
        .collect();
    assert_eq!(pages, ordre, "les pages se suivent dans l'ordre complet");
}

// ---------------------------------------------------------------------------
// Tests 6 et 7 — `document` et `manuallyLetterable`
// ---------------------------------------------------------------------------

/// Tests 6 et 7 (AC3) — `document` des cinq types (facture, avoir, règlement
/// avec sa facture, facture fournisseur achat ET règlement, transaction
/// bancaire), la précédence règlement + transaction → `settlement`, et
/// `manuallyLetterable` : faux pour chacun des quatre types de pièce, vrai pour
/// une ligne seulement rapprochée, faux pour une ligne lettrée.
#[sqlx::test(migrations = "./test-schema")]
async fn document_types_and_manual_letterability(pool: MySqlPool) {
    let m = monde(&pool).await;
    let d = jours_avant(60);
    let r = jours_avant(50);

    // Facture due, réglée en partie : ligne de vente et ligne de règlement.
    let partielle = facture(&pool, &m.s, dec!(80.00), d).await;
    let (sid, e_regl) = regler(&pool, &m.s, partielle, dec!(20.00), r).await;
    let l_vente = ligne_creance(&pool, &m, vente(&pool, partielle).await).await;
    let l_regl = ligne_creance(&pool, &m, e_regl).await;
    // Facture réglée puis le règlement rapproché, marques effacées : la ligne de
    // règlement porte règlement ET transaction.
    let rapprochee = facture(&pool, &m.s, dec!(40.00), d).await;
    let (_, e_rap) = regler(&pool, &m.s, rapprochee, dec!(40.00), r).await;
    rapprocher(&pool, &m.s, e_rap, dec!(40.00), r).await;
    effacer_marques(&pool, cle_de(&pool, rapprochee).await.expect("lettrée")).await;
    let l_rap = ligne_creance(&pool, &m, e_rap).await;
    // Avoir total, marques effacées (état hérité) : la ligne d'avoir s'ouvre.
    let creditee = facture(&pool, &m.s, dec!(60.00), d).await;
    let e_avoir = crediter(&pool, &m.s, creditee, r).await;
    effacer_marques(&pool, cle_de(&pool, creditee).await.expect("lettrée")).await;
    let l_avoir = ligne_creance(&pool, &m, e_avoir).await;
    // Ligne manuelle seulement rapprochée d'une transaction bancaire.
    let (e_banque, l_banque) = ligne(&pool, &m, m.fy, r, dec!(-15)).await;
    let tx_id = rapprocher(&pool, &m.s, e_banque, dec!(15), r).await;
    // Une paire manuelle lettrée après X.
    let (_, l_lettree) = ligne(&pool, &m, m.fy, r, dec!(9)).await;
    let (_, l_tard) = ligne(&pool, &m, m.fy, jours_avant(5), dec!(-9)).await;
    lettrer(&pool, &m, &[l_lettree, l_tard]).await;
    // Facture fournisseur payée, marques effacées : achat et règlement ouverts
    // sur la dette.
    let a = achats(&pool, &m.s).await;
    let s = facture_fournisseur(&pool, &m.s, &a, dec!(70.00), d, Some("FF-1")).await;
    let e_paiement = payer(&pool, &m.s, &a, s, r).await;
    let cle = lignes_de_piece_fournisseur(&pool, s).await[0]
        .2
        .expect("la facture fournisseur est lettrée");
    effacer_marques(&pool, cle).await;

    let x = jours_avant(30);
    let v = vue(&pool, &m, x).await;
    invariant(&pool, &m, &v).await;

    let doc = |id: i64| item(&v, id).document.clone().expect("document");
    let numero_partielle: String =
        sqlx::query_scalar("SELECT invoice_number FROM invoices WHERE id = ?")
            .bind(partielle)
            .fetch_one(&pool)
            .await
            .unwrap();
    // Facture.
    let f = doc(l_vente);
    assert_eq!((f.kind, f.id), (DocumentKind::Invoice, partielle));
    assert_eq!(f.number.as_deref(), Some(numero_partielle.as_str()));
    assert_eq!((f.invoice_id, f.invoice_number.clone()), (None, None));
    // Règlement : la facture réglée nommée.
    let g = doc(l_regl);
    assert_eq!((g.kind, g.id), (DocumentKind::Settlement, sid));
    assert_eq!(g.number, None);
    assert_eq!(g.invoice_id, Some(partielle));
    assert_eq!(g.invoice_number.as_deref(), Some(numero_partielle.as_str()));
    // Règlement rapproché : la précédence rend `settlement`.
    assert_eq!(doc(l_rap).kind, DocumentKind::Settlement);
    // Avoir.
    let c = doc(l_avoir);
    assert_eq!(c.kind, DocumentKind::CreditNote);
    assert!(c.number.is_some(), "un avoir émis porte son numéro");
    // Transaction bancaire seule.
    let b = doc(l_banque);
    assert_eq!(
        (b.kind, b.id, b.number.clone()),
        (DocumentKind::BankTransaction, tx_id, None)
    );

    // Test 7 — sur la créance.
    for l in [l_vente, l_regl, l_rap, l_avoir] {
        assert!(!item(&v, l).manually_letterable, "ligne de pièce {l}");
    }
    assert!(
        item(&v, l_banque).manually_letterable,
        "ligne seulement rapprochée"
    );
    let lettree = item(&v, l_lettree);
    assert_eq!(lettree.reason, OpenReason::LetteredAfterAsOf);
    assert!(!lettree.manually_letterable, "ligne lettrée");
    assert!(absente(&v, l_tard), "ligne datée après X");

    // Facture fournisseur : achat et règlement, sur la dette.
    let dette = vue_page(&pool, &m, a.payable, x, 500, 0).await;
    invariant(&pool, &m, &dette).await;
    let par_ecriture = |entry: i64| {
        dette
            .items
            .iter()
            .find(|i| i.entry_id == entry)
            .unwrap_or_else(|| panic!("écriture {entry} absente de la dette"))
    };
    for entry in [achat(&pool, s).await, e_paiement] {
        let i = par_ecriture(entry);
        let o = i.document.clone().expect("document fournisseur");
        assert_eq!((o.kind, o.id), (DocumentKind::SupplierInvoice, s));
        assert_eq!(o.number.as_deref(), Some("FF-1"));
        assert!(!i.manually_letterable, "ligne de facture fournisseur");
        assert_eq!(i.document_state, None, "pas une facture client");
        assert_eq!(i.amount_due, None);
    }
}

// ---------------------------------------------------------------------------
// Test 9 — `reason`
// ---------------------------------------------------------------------------

/// Test 9 (AC4) — `reason = letteredAfterAsOf` sur le groupe à cheval à la date
/// intermédiaire, `unlettered` ailleurs ; et le groupe à cheval absent après
/// sa dernière ligne. `letteredOn` = date de la ligne la plus tardive.
#[sqlx::test(migrations = "./test-schema")]
async fn reason_on_a_group_across_two_fiscal_years(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (_, a) = ligne(&pool, &m, m.fy24, ymd(2024, 6, 30), dec!(100)).await;
    let (_, b) = ligne(&pool, &m, m.fy, ymd(2025, 3, 1), dec!(-100)).await;
    let (_, libre) = ligne(&pool, &m, m.fy24, ymd(2024, 5, 1), dec!(3)).await;
    let cle = lettrer(&pool, &m, &[a, b]).await;

    let entre = vue(&pool, &m, ymd(2024, 12, 31)).await;
    invariant(&pool, &m, &entre).await;
    let i = item(&entre, a);
    assert_eq!(i.reason, OpenReason::LetteredAfterAsOf);
    assert_eq!(i.lettered_on, Some(ymd(2025, 3, 1)));
    assert_eq!(i.lettering_key, Some(cle));
    assert_eq!(
        i.lettering_code.as_deref(),
        Some(letterings::code_of(cle).as_str())
    );
    assert_eq!(i.lettering_origin, Some(Origin::Manual));
    assert_eq!(i.fiscal_year_name, "Exercice 2024");
    assert!(absente(&entre, b));
    let l = item(&entre, libre);
    assert_eq!(l.reason, OpenReason::Unlettered);
    assert_eq!(
        (l.lettering_key, l.lettered_on, l.lettering_origin),
        (None, None, None)
    );

    // X = la date de la dernière ligne : le lettrage est acquis (borne stricte).
    let le_jour = vue(&pool, &m, ymd(2025, 3, 1)).await;
    invariant(&pool, &m, &le_jour).await;
    assert!(
        absente(&le_jour, a) && absente(&le_jour, b),
        "acquis le jour même"
    );

    let apres = vue(&pool, &m, ymd(2025, 6, 1)).await;
    invariant(&pool, &m, &apres).await;
    assert!(
        absente(&apres, a) && absente(&apres, b),
        "groupe acquis au X"
    );
    assert_eq!(item(&apres, libre).reason, OpenReason::Unlettered);

    let avant = vue(&pool, &m, ymd(2024, 6, 1)).await;
    invariant(&pool, &m, &avant).await;
    assert!(absente(&avant, a), "ligne postérieure à X");
}

// ---------------------------------------------------------------------------
// Test 10 — `documentState` et `amountDue`
// ---------------------------------------------------------------------------

/// Test 10 (AC4) — un test par valeur de `documentState`, `amountDue` au
/// centime ; l'état est celui d'AUJOURD'HUI même pour un X antérieur au
/// règlement ; les deux sources de `nothingDue` sur une ligne ouverte
/// aujourd'hui (pièce historique close, avoir hérité sur un autre compte),
/// avec `manuallyLetterable = false` ; le reste brut de 0,004 ; `amountDue` =
/// TTC d'un héritage `paid_at` ; `documentState` nul pour une ligne d'avoir.
#[sqlx::test(migrations = "./test-schema")]
async fn document_state_each_value(pool: MySqlPool) {
    let m = monde(&pool).await;
    let d = jours_avant(200);
    let r = jours_avant(150);

    // unpaid.
    let due = facture(&pool, &m.s, dec!(80.00), d).await;
    // partiallySettled — réglée APRÈS le X de lecture.
    let partielle = facture(&pool, &m.s, dec!(90.00), d).await;
    regler(&pool, &m.s, partielle, dec!(30.00), r).await;
    // paidWithoutSettlementEntry.
    let heritee = facture(&pool, &m.s, dec!(30.00), d).await;
    sqlx::query("UPDATE invoices SET paid_at = ? WHERE id = ?")
        .bind(r.and_hms_opt(0, 0, 0).unwrap())
        .bind(heritee)
        .execute(&pool)
        .await
        .unwrap();
    // Reste brut de 0,004 — fixture nommée : réglée au centime (le grand livre
    // se nette, la synchronisation lettre `document`), puis la ligne de facture
    // portée à 10,0040 en SQL brut : le reste DÉRIVÉ vaut 0,004.
    let infime = facture(&pool, &m.s, dec!(10.00), d).await;
    regler(&pool, &m.s, infime, dec!(10.00), r).await;
    sqlx::query(
        "UPDATE invoice_lines SET line_total = 10.0040, unit_price = 10.0040 WHERE invoice_id = ?",
    )
    .bind(infime)
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(
        invoice_settlements::amount_due(&pool, infime)
            .await
            .unwrap(),
        dec!(0.0040),
        "montage : reste brut de 0,004"
    );
    assert!(
        lettree_document(&pool, infime).await,
        "montage : lettrée document"
    );
    // Ligne d'avoir ouverte (marques effacées) : documentState nul.
    let creditee = facture(&pool, &m.s, dec!(60.00), d).await;
    let e_avoir = crediter(&pool, &m.s, creditee, r).await;
    effacer_marques(&pool, cle_de(&pool, creditee).await.expect("lettrée")).await;

    let x = jours_avant(170); // après les factures, avant les règlements
    let v = vue(&pool, &m, x).await;
    invariant(&pool, &m, &v).await;
    let etat = |inv: i64| {
        let l = v
            .items
            .iter()
            .find(|i| {
                i.document
                    .as_ref()
                    .is_some_and(|o| o.kind == DocumentKind::Invoice && o.id == inv)
            })
            .unwrap_or_else(|| panic!("ligne de vente de {inv} absente"));
        (l.document_state, l.amount_due, l.reason)
    };
    assert_eq!(
        etat(due),
        (
            Some(DocumentState::Unpaid),
            Some(dec!(80.00)),
            OpenReason::Unlettered
        )
    );
    assert_eq!(
        etat(partielle),
        (
            Some(DocumentState::PartiallySettled),
            Some(dec!(60.00)),
            OpenReason::Unlettered
        ),
        "état d'aujourd'hui pour un X antérieur au règlement"
    );
    assert_eq!(
        etat(heritee),
        (
            Some(DocumentState::PaidWithoutSettlementEntry),
            Some(dec!(30.00)),
            OpenReason::Unlettered
        ),
        "amountDue = TTC entier"
    );
    let (etat_infime, du_infime, raison) = etat(infime);
    assert_eq!(
        etat_infime,
        Some(DocumentState::NothingDue),
        "seuil lu au centime (#490)"
    );
    assert_eq!(du_infime, Some(dec!(0.00)));
    assert_eq!(raison, OpenReason::LetteredAfterAsOf);
    assert_eq!(
        item(
            &v,
            ligne_creance(&pool, &m, vente(&pool, infime).await).await
        )
        .lettering_origin,
        Some(Origin::Document)
    );
    // Ligne d'avoir : documentState nul — l'avoir est daté de `r`, après X : on
    // le lit à aujourd'hui.
    let auj = vue(&pool, &m, aujourdhui()).await;
    let la = item(&auj, ligne_creance(&pool, &m, e_avoir).await);
    assert_eq!(
        la.document.as_ref().map(|o| o.kind),
        Some(DocumentKind::CreditNote)
    );
    assert_eq!((la.document_state, la.amount_due), (None, None));
    assert!(!la.manually_letterable);
}

/// Test 10 (AC4) — `nothingDue` sur une ligne OUVERTE aujourd'hui, par ses deux
/// sources : (1) pièce historique entièrement close, marques effacées, que la
/// synchronisation n'a pas lettrée (15-1a2-i P7 point 1) ; (2) facture créditée
/// par un avoir hérité crédité sur un autre compte (exception (a) d'AC5 de la
/// 15-1a2-i). Les deux : `reason = unlettered`, `manuallyLetterable = false`.
#[sqlx::test(migrations = "./test-schema")]
async fn nothing_due_on_a_line_open_today(pool: MySqlPool) {
    let m = monde(&pool).await;
    // (1) Pièce historique close.
    let le = jours_avant(90);
    let close = facture(&pool, &m.s, dec!(100.00), jours_avant(100)).await;
    regler(&pool, &m.s, close, dec!(100.00), le).await;
    effacer_marques(&pool, cle_de(&pool, close).await.expect("lettrée")).await;
    // (2) Avoir hérité crédité sur le 2000.
    let inv = facture(&pool, &m.s, dec!(25.00), jours_avant(80)).await;
    let avoir = crediter(&pool, &m.s, inv, jours_avant(70)).await;
    effacer_marques(&pool, cle_de(&pool, inv).await.expect("lettrée")).await;
    let n = sqlx::query(
        "UPDATE journal_entry_lines SET account_id = ? WHERE entry_id = ? AND account_id = ?",
    )
    .bind(m.s.accounts["2000"])
    .bind(avoir)
    .bind(m.creance())
    .execute(&pool)
    .await
    .unwrap()
    .rows_affected();
    assert_eq!(n, 1, "montage : la ligne de créance de l'avoir déplacée");
    companies::lock_books(&pool, m.s.admin_user_id, m.company(), le)
        .await
        .expect("verrou");

    let v = vue(&pool, &m, aujourdhui()).await;
    invariant(&pool, &m, &v).await;
    for inv in [close, inv] {
        let l = item(&v, ligne_creance(&pool, &m, vente(&pool, inv).await).await);
        assert_eq!(
            l.reason,
            OpenReason::Unlettered,
            "ouverte aujourd'hui ({inv})"
        );
        assert_eq!(l.document_state, Some(DocumentState::NothingDue), "{inv}");
        assert_eq!(l.amount_due, Some(dec!(0.00)), "{inv}");
        assert!(!l.manually_letterable, "{inv}");
    }
    // La ligne historique est sous la borne : hors période ouverte.
    let historique = item(
        &v,
        ligne_creance(&pool, &m, vente(&pool, close).await).await,
    );
    assert!(!historique.in_open_period);
}

// ---------------------------------------------------------------------------
// Test 20 — les champs de la vue
// ---------------------------------------------------------------------------

/// Test 20 (AC1) — `letteringOrigin` des trois origines, `letteredOn` (date de
/// la ligne la plus tardive du groupe), `fiscalYearName`, `inOpenPeriod` dans la
/// vue : vrai en période ouverte, faux sous la borne, faux pour un exercice
/// suivi d'un exercice clos.
#[sqlx::test(migrations = "./test-schema")]
async fn view_fields_origins_and_periods(pool: MySqlPool) {
    let m = monde(&pool).await;
    // document : facture réglée ; listée à un X entre facture et règlement.
    let inv = facture(&pool, &m.s, dec!(40.00), jours_avant(60)).await;
    regler(&pool, &m.s, inv, dec!(40.00), jours_avant(20)).await;
    let l_doc = ligne_creance(&pool, &m, vente(&pool, inv).await).await;
    // reversal : écriture manuelle contre-passée (contre-passation du jour).
    let (e_rev, l_rev) = ligne(&pool, &m, m.fy, jours_avant(40), dec!(12)).await;
    kesh_db::repositories::journal_entries::reverse(&pool, m.company(), e_rev, m.s.admin_user_id)
        .await
        .expect("contre-passation");
    // manual : groupe de trois lignes, la plus tardive à jours_avant(10).
    let (_, m1) = ligne(&pool, &m, m.fy, jours_avant(50), dec!(10)).await;
    let (_, m2) = ligne(&pool, &m, m.fy, jours_avant(15), dec!(-4)).await;
    let (_, m3) = ligne(&pool, &m, m.fy, jours_avant(10), dec!(-6)).await;
    lettrer(&pool, &m, &[m1, m2, m3]).await;

    let x = jours_avant(30);
    let v = vue(&pool, &m, x).await;
    invariant(&pool, &m, &v).await;
    let d = item(&v, l_doc);
    assert_eq!(d.lettering_origin, Some(Origin::Document));
    assert_eq!(d.lettered_on, Some(jours_avant(20)));
    let r = item(&v, l_rev);
    assert_eq!(r.lettering_origin, Some(Origin::Reversal));
    assert_eq!(r.lettered_on, Some(aujourdhui()));
    let mm = item(&v, m1);
    assert_eq!(mm.lettering_origin, Some(Origin::Manual));
    assert_eq!(
        mm.lettered_on,
        Some(jours_avant(10)),
        "la ligne la plus tardive"
    );
    assert_eq!(mm.fiscal_year_name, "Exercice 2025-2030");
    for i in &v.items {
        assert!(
            i.in_open_period,
            "aucune borne, exercice ouvert : {}",
            i.line_id
        );
    }

    // Sous la borne.
    companies::lock_books(&pool, m.s.admin_user_id, m.company(), jours_avant(45))
        .await
        .expect("verrou");
    let v = vue(&pool, &m, x).await;
    assert!(!item(&v, m1).in_open_period, "sous la borne");
    assert!(item(&v, l_rev).in_open_period, "au-dessus de la borne");

    // Exercice suivi d'un exercice clos (état hérité « N ouvert, N+1 clos »).
    let (_, l24) = ligne(&pool, &m, m.fy24, ymd(2024, 3, 1), dec!(2)).await;
    let v = vue(&pool, &m, x).await;
    assert!(!item(&v, l24).in_open_period, "sous la borne aussi");
    companies::unlock_books(&pool, m.s.admin_user_id, m.company(), None, "test".into())
        .await
        .expect("déverrou");
    let v = vue(&pool, &m, x).await;
    assert!(item(&v, l24).in_open_period, "2024 ouvert, sans borne");
    statut(&pool, m.fy, "Closed").await;
    let v = vue(&pool, &m, x).await;
    assert_eq!(item(&v, l24).fiscal_year_name, "Exercice 2024");
    assert!(
        !item(&v, l24).in_open_period,
        "exercice suivi d'un exercice clos"
    );
    assert!(!item(&v, m1).in_open_period, "exercice clos");
}

// ---------------------------------------------------------------------------
// Test 19 — AC12 : « au X » stable en période close, mobile au-dessus
// ---------------------------------------------------------------------------

/// Test 19 (AC12 a) — `X` dans un exercice CLOS : des gestes qui RÉUSSISSENT
/// touchent des lignes ≤ X — lettrage manuel d'une ligne ≤ X avec une ligne
/// > X, règlement en N+1 d'une facture de l'exercice clos dont la créance est
/// ≤ X — et la liste « au X » est identique ; seul le `reason` des lignes
/// lettrées passe à `letteredAfterAsOf`.
#[sqlx::test(migrations = "./test-schema")]
async fn as_of_in_a_closed_fiscal_year_is_stable(pool: MySqlPool) {
    let m = monde(&pool).await;
    let inv = facture(&pool, &m.s, dec!(50.00), ymd(2024, 3, 1)).await;
    let l_vente = ligne_creance(&pool, &m, vente(&pool, inv).await).await;
    let (_, a) = ligne(&pool, &m, m.fy24, ymd(2024, 5, 1), dec!(100)).await;
    let (_, b) = ligne(&pool, &m, m.fy, jours_avant(30), dec!(-100)).await;
    statut(&pool, m.fy24, "Closed").await;
    let x = ymd(2024, 12, 31);

    let avant = vue(&pool, &m, x).await;
    invariant(&pool, &m, &avant).await;
    assert_eq!(item(&avant, a).reason, OpenReason::Unlettered);
    assert_eq!(item(&avant, l_vente).reason, OpenReason::Unlettered);

    lettrer(&pool, &m, &[a, b]).await;
    regler(&pool, &m.s, inv, dec!(50.00), jours_avant(20)).await;
    assert!(
        lettree_document(&pool, inv).await,
        "montage : le règlement a lettré"
    );

    let apres = vue(&pool, &m, x).await;
    invariant(&pool, &m, &apres).await;
    assert_eq!(contenu(&apres), contenu(&avant), "liste identique");
    assert_eq!(
        (apres.open_total, apres.total),
        (avant.open_total, avant.total)
    );
    assert_eq!(item(&apres, a).reason, OpenReason::LetteredAfterAsOf);
    assert_eq!(item(&apres, l_vente).reason, OpenReason::LetteredAfterAsOf);
}

/// Test 19 (AC12 b) — `X` ≤ `books_locked_through`, exercice ouvert :
/// l'annulation d'un règlement dont tout le groupe `document` est ≤ borne est
/// REFUSÉE et la liste est identique ; un administrateur déverrouille, la même
/// annulation réussit, et la créance et la ligne de règlement réapparaissent
/// « au X ».
#[sqlx::test(migrations = "./test-schema")]
async fn as_of_under_the_lock_is_stable_until_an_admin_unlocks(pool: MySqlPool) {
    let m = monde(&pool).await;
    let inv = facture(&pool, &m.s, dec!(70.00), jours_avant(100)).await;
    let (sid, e_regl) = regler(&pool, &m.s, inv, dec!(70.00), jours_avant(90)).await;
    let l_vente = ligne_creance(&pool, &m, vente(&pool, inv).await).await;
    let l_regl = ligne_creance(&pool, &m, e_regl).await;
    let borne = jours_avant(80);
    companies::lock_books(&pool, m.s.admin_user_id, m.company(), borne)
        .await
        .expect("verrou");
    let x = jours_avant(85);

    let avant = vue(&pool, &m, x).await;
    invariant(&pool, &m, &avant).await;
    assert!(
        absente(&avant, l_vente) && absente(&avant, l_regl),
        "groupe acquis au X"
    );

    let refus = invoice_settlements_write::cancel_settlement(
        &pool,
        m.s.admin_user_id,
        m.company(),
        inv,
        sid,
    )
    .await
    .expect_err("annulation refusée sous la borne");
    assert!(
        matches!(
            refus,
            DbError::SettlementNotCancellable {
                blocker: SettlementCancelBlocker::DocumentLetteringInClosedPeriods
            }
        ),
        "{refus:?}"
    );
    let refusee = vue(&pool, &m, x).await;
    assert_eq!(
        contenu(&refusee),
        contenu(&avant),
        "liste identique après le refus"
    );
    assert_eq!(refusee.open_total, avant.open_total);

    companies::unlock_books(
        &pool,
        m.s.admin_user_id,
        m.company(),
        None,
        "correction".into(),
    )
    .await
    .expect("déverrouillage");
    invoice_settlements_write::cancel_settlement(&pool, m.s.admin_user_id, m.company(), inv, sid)
        .await
        .expect("annulation après déverrouillage");
    let apres = vue(&pool, &m, x).await;
    invariant(&pool, &m, &apres).await;
    assert_eq!(item(&apres, l_vente).reason, OpenReason::Unlettered);
    assert_eq!(
        item(&apres, l_regl).reason,
        OpenReason::LetteredAfterAsOf,
        "lettrée `reversal` avec sa contre-passation du jour"
    );
    assert_eq!(apres.total, avant.total + 2);
}

/// Test 19 (AC12 c) — `X` au-dessus de la borne, exercice ouvert : le
/// délettrage manuel d'un groupe entièrement ≤ X fait réapparaître ses lignes
/// « au X ».
#[sqlx::test(migrations = "./test-schema")]
async fn as_of_above_the_lock_moves(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (_, a) = ligne(&pool, &m, m.fy, jours_avant(40), dec!(30)).await;
    let (_, b) = ligne(&pool, &m, m.fy, jours_avant(35), dec!(-30)).await;
    let cle = lettrer(&pool, &m, &[a, b]).await;
    companies::lock_books(&pool, m.s.admin_user_id, m.company(), jours_avant(60))
        .await
        .expect("verrou");
    let x = jours_avant(30);
    let avant = vue(&pool, &m, x).await;
    invariant(&pool, &m, &avant).await;
    assert!(absente(&avant, a) && absente(&avant, b));
    delettrer(&pool, &m, cle).await;
    let apres = vue(&pool, &m, x).await;
    invariant(&pool, &m, &apres).await;
    assert_eq!(item(&apres, a).reason, OpenReason::Unlettered);
    assert_eq!(item(&apres, b).reason, OpenReason::Unlettered);
    assert_eq!(apres.total, avant.total + 2);
}

// ---------------------------------------------------------------------------
// Tests 12 et 13 — le chargement des propositions
// ---------------------------------------------------------------------------

async fn propositions(
    pool: &MySqlPool,
    m: &Monde,
    limit: usize,
) -> Result<letterings::LetteringProposals, DbError> {
    let mut c = pool.acquire().await.unwrap();
    letterings::lettering_proposals(&mut c, m.company(), m.creance(), limit).await
}

fn paires(p: &letterings::LetteringProposals) -> Vec<(i64, i64)> {
    p.items
        .iter()
        .map(|i| (i.debit.line_id, i.credit.line_id))
        .collect()
}

/// Test 12 (AC5) — exclusions : ligne de pièce, ligne lettrée, paire tout
/// entière en exercice clos ; inclusion d'une paire dont une seule ligne est en
/// période ouverte ; et trois lignes de même montant — `A` débit et `B` crédit
/// en période close (même jour, exercice 2024 clos), `C` crédit en période
/// ouverte — : la réponse porte `A–C` et elle seule, alors que `A–B` (écart
/// nul) la précède au classement (C-15-1b-15 : « même date » n'est pas
/// montable pour deux périodes distinctes ; la preuve à dates égales est au
/// test 11, dans le moteur pur).
#[sqlx::test(migrations = "./test-schema")]
async fn proposals_load_filters_candidates(pool: MySqlPool) {
    let m = monde(&pool).await;
    // Ligne de pièce : une facture due de 100 (aucune paire avec D).
    let inv = facture(&pool, &m.s, dec!(100.00), jours_avant(20)).await;
    let l_piece = ligne_creance(&pool, &m, vente(&pool, inv).await).await;
    // Ligne lettrée : 77 / −77 lettrées, et un −77 libre.
    let (_, l1) = ligne(&pool, &m, m.fy, jours_avant(20), dec!(77)).await;
    let (_, l2) = ligne(&pool, &m, m.fy, jours_avant(19), dec!(-77)).await;
    lettrer(&pool, &m, &[l1, l2]).await;
    let (_, l77) = ligne(&pool, &m, m.fy, jours_avant(18), dec!(-77)).await;
    // Paire tout entière en exercice clos : 55 / −55 en 2024.
    let (_, c1) = ligne(&pool, &m, m.fy24, ymd(2024, 2, 1), dec!(55)).await;
    let (_, c2) = ligne(&pool, &m, m.fy24, ymd(2024, 2, 2), dec!(-55)).await;
    // Une seule ligne ouverte : 44 en 2024, −44 en période ouverte.
    let (_, o1) = ligne(&pool, &m, m.fy24, ymd(2024, 4, 1), dec!(44)).await;
    let (_, o2) = ligne(&pool, &m, m.fy, jours_avant(5), dec!(-44)).await;
    // A, B (2024, même jour) et C (ouverte), 100 : A débit, B et C crédit.
    let (_, a) = ligne(&pool, &m, m.fy24, ymd(2024, 9, 1), dec!(100)).await;
    let (_, b) = ligne(&pool, &m, m.fy24, ymd(2024, 9, 1), dec!(-100)).await;
    let (_, c) = ligne(&pool, &m, m.fy, jours_avant(3), dec!(-100)).await;
    statut(&pool, m.fy24, "Closed").await;

    let p = propositions(&pool, &m, 100).await.expect("propositions");
    assert_eq!(paires(&p), vec![(a, c), (o1, o2)]);
    assert_eq!(p.total, 2);
    // Candidates : toutes les lignes non lettrées hors pièce (l77, c1, c2, o1,
    // o2, a, b, c) — la pièce et les lettrées exclues.
    assert_eq!(p.candidate_count, 8);
    let toutes: Vec<i64> = p
        .items
        .iter()
        .flat_map(|i| [i.debit.line_id, i.credit.line_id])
        .collect();
    for exclue in [l_piece, l1, l2, c1, c2, b, l77] {
        assert!(!toutes.contains(&exclue), "ligne {exclue} non proposée");
    }
    let ac = &p.items[0];
    assert!(!ac.debit.in_open_period && ac.credit.in_open_period);
    assert_eq!(ac.amount, dec!(100));
    assert_eq!(ac.debit.fiscal_year_name, "Exercice 2024");
    assert!(!ac.reversal_pair);
    // limit : la première paire seule ; `total` reste 2.
    let p = propositions(&pool, &m, 1).await.unwrap();
    assert_eq!((p.items.len(), p.total, p.limit), (1, 2, 1));
}

/// AC5 — une ligne seulement rapprochée d'une transaction bancaire reste
/// candidate, porte son `document`, et la paire contre-passation/origine
/// délettrée passe en tête.
#[sqlx::test(migrations = "./test-schema")]
async fn proposals_keep_bank_lines_and_rank_reversal_pairs_first(pool: MySqlPool) {
    let m = monde(&pool).await;
    let (e_banque, l_banque) = ligne(&pool, &m, m.fy, jours_avant(10), dec!(-20)).await;
    rapprocher(&pool, &m.s, e_banque, dec!(20), jours_avant(10)).await;
    let (_, l20) = ligne(&pool, &m, m.fy, jours_avant(10), dec!(20)).await;
    // Contre-passation d'une ligne de 20 : le groupe `reversal` délettré.
    let (e_rev, l_origine) = ligne(&pool, &m, m.fy, jours_avant(40), dec!(20)).await;
    let renv = kesh_db::repositories::journal_entries::reverse(
        &pool,
        m.company(),
        e_rev,
        m.s.admin_user_id,
    )
    .await
    .expect("contre-passation");
    let l_miroir = ligne_creance(&pool, &m, renv.entry.id).await;
    let cle: i64 = sqlx::query_scalar("SELECT lettering_key FROM journal_entry_lines WHERE id = ?")
        .bind(l_origine)
        .fetch_one(&pool)
        .await
        .unwrap();
    delettrer(&pool, &m, cle).await;

    let p = propositions(&pool, &m, 100).await.unwrap();
    assert_eq!(paires(&p), vec![(l_origine, l_miroir), (l20, l_banque)]);
    assert!(p.items[0].reversal_pair);
    assert_eq!(
        p.items[1].credit.document.as_ref().map(|o| o.kind),
        Some(DocumentKind::BankTransaction)
    );
}

/// Test 13 (AC5) — le plafond porte sur les candidates APRÈS les filtres :
/// 2 000 candidates → rendu ; 2 001 → `LetteringProposalsTooManyLines { max:
/// 2000 }` ; 3 000 lignes de pièces (une écriture par facture) et 10
/// candidates → rendu, `candidateCount = 10`. Temps du dernier cas noté
/// (`--nocapture`).
#[sqlx::test(migrations = "./test-schema")]
async fn proposals_cap_counts_candidates_after_filters(pool: MySqlPool) {
    let m = monde(&pool).await;
    // 3 000 écritures, chacune possédée par une facture, une ligne sur la
    // créance — par le moteur `Sequence` de MariaDB.
    let modele = facture(&pool, &m.s, dec!(1.00), jours_avant(30)).await;
    sqlx::query(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, \
         journal, description) SELECT ?, ?, 100000 + seq, ?, 'Ventes', 'volume' \
         FROM seq_1_to_3000",
    )
    .bind(m.company())
    .bind(m.fy)
    .bind(jours_avant(30))
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
         SELECT id, ?, 1, 1, 0 FROM journal_entries WHERE company_id = ? AND description = 'volume'",
    )
    .bind(m.creance())
    .bind(m.company())
    .execute(&pool)
    .await
    .unwrap();
    let colonnes: Vec<String> = sqlx::query_scalar(
        "SELECT COLUMN_NAME FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = DATABASE() \
         AND TABLE_NAME = 'invoices' AND COLUMN_NAME NOT IN ('id', 'journal_entry_id', \
         'invoice_number') ORDER BY ORDINAL_POSITION",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    let liste = colonnes.join(", ");
    let copie = colonnes
        .iter()
        .map(|c| format!("i.{c}"))
        .collect::<Vec<_>>()
        .join(", ");
    let n = sqlx::query(&format!(
        "INSERT INTO invoices ({liste}, journal_entry_id, invoice_number) \
         SELECT {copie}, je.id, CONCAT('VOL-', je.entry_number) FROM invoices i \
         JOIN journal_entries je ON je.company_id = i.company_id AND je.description = 'volume' \
         WHERE i.id = ?"
    ))
    .bind(modele)
    .execute(&pool)
    .await
    .unwrap()
    .rows_affected();
    assert_eq!(n, 3000, "montage : 3 000 factures");
    // 10 candidates.
    for k in 0..10 {
        ligne(&pool, &m, m.fy, jours_avant(10), Decimal::from(k + 1)).await;
    }
    let debut = std::time::Instant::now();
    let p = propositions(&pool, &m, 100)
        .await
        .expect("200 malgré 3 000 lignes de pièces");
    eprintln!(
        "propositions, 3 000 lignes de pièces + 10 candidates : {:?}",
        debut.elapsed()
    );
    assert_eq!(p.candidate_count, 10);

    // 1 990 candidates de plus : 2 000 tout juste.
    let (e, _) = ecriture(&pool, &m, m.fy, jours_avant(9), &[]).await;
    let ajouter = |k: usize, depuis: i64| {
        let pool = pool.clone();
        let compte = m.creance();
        async move {
            sqlx::query(&format!(
                "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
                 SELECT ?, ?, ? + seq, 3, 0 FROM seq_1_to_{k}"
            ))
            .bind(e)
            .bind(compte)
            .bind(depuis)
            .execute(&pool)
            .await
            .unwrap();
        }
    };
    ajouter(1990, 0).await;
    let p = propositions(&pool, &m, 100)
        .await
        .expect("2 000 candidates : rendu");
    assert_eq!(p.candidate_count, 2000);
    ajouter(1, 1990).await;
    let refus = propositions(&pool, &m, 100)
        .await
        .expect_err("2 001 candidates");
    assert!(
        matches!(refus, DbError::LetteringProposalsTooManyLines { max: 2000 }),
        "{refus:?}"
    );
}

/// AC7 au dépôt — compte d'une autre société et compte inexistant →
/// `NotFound` ; compte non lettrable (charge, compte bancaire) →
/// `LetteringAccountNotLetterable`, sur les deux fonctions.
#[sqlx::test(migrations = "./test-schema")]
async fn account_refusals(pool: MySqlPool) {
    let m = monde(&pool).await;
    // Un compte d'actif d'une autre société (lettrable chez elle).
    let autre_societe = kesh_db::test_fixtures::seed_stub_company_only(&pool)
        .await
        .expect("autre société");
    let etranger = compte(&pool, autre_societe, "1100", "Asset").await;
    let mut c = pool.acquire().await.unwrap();
    for compte in [etranger, i64::MAX] {
        let e = letterings::open_items(&mut c, m.company(), compte, aujourdhui(), 50, 0)
            .await
            .expect_err("404");
        assert!(matches!(e, DbError::NotFound), "{e:?}");
        let e = letterings::lettering_proposals(&mut c, m.company(), compte, 100)
            .await
            .expect_err("404");
        assert!(matches!(e, DbError::NotFound), "{e:?}");
    }
    let a = achats(&pool, &m.s).await;
    for compte in [m.s.accounts["4000"], a.bank_ledger] {
        let e = letterings::open_items(&mut c, m.company(), compte, aujourdhui(), 50, 0)
            .await
            .expect_err("409");
        assert!(matches!(e, DbError::LetteringAccountNotLetterable), "{e:?}");
        let e = letterings::lettering_proposals(&mut c, m.company(), compte, 100)
            .await
            .expect_err("409");
        assert!(matches!(e, DbError::LetteringAccountNotLetterable), "{e:?}");
    }
}

// ---------------------------------------------------------------------------
// AC8 — EXPLAIN (ignoré : volume, lancé à la main, noté au Dev Agent Record)
// ---------------------------------------------------------------------------

/// Affiche une ligne d'`EXPLAIN` (colonnes de types variés).
fn ligne_explain(row: &sqlx::mysql::MySqlRow) -> String {
    use sqlx::{Column, Row};
    row.columns()
        .iter()
        .map(|col| {
            let i = col.ordinal();
            let v = row
                .try_get::<Option<String>, _>(i)
                .ok()
                .flatten()
                .or_else(|| {
                    row.try_get::<Option<i64>, _>(i)
                        .ok()
                        .flatten()
                        .map(|n| n.to_string())
                })
                .or_else(|| {
                    row.try_get::<Option<u64>, _>(i)
                        .ok()
                        .flatten()
                        .map(|n| n.to_string())
                })
                .or_else(|| {
                    row.try_get::<Option<f64>, _>(i)
                        .ok()
                        .flatten()
                        .map(|n| n.to_string())
                })
                .unwrap_or_else(|| "NULL".into());
            format!("{}={v}", col.name())
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

/// Copie `n` fois la ligne `id` de `table`, pour un montage de volume : les
/// colonnes de `remplacer` reçoivent l'expression donnée (sur l'alias `x` de la
/// ligne modèle et `s.seq`, de 1 à `n`) ; les autres sont recopiées.
async fn copier(pool: &MySqlPool, table: &str, id: i64, n: u32, remplacer: &[(&str, String)]) {
    let colonnes: Vec<String> = sqlx::query_scalar(
        "SELECT COLUMN_NAME FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = DATABASE() \
         AND TABLE_NAME = ? AND COLUMN_NAME <> 'id' ORDER BY ORDINAL_POSITION",
    )
    .bind(table)
    .fetch_all(pool)
    .await
    .unwrap();
    let exprs: Vec<String> = colonnes
        .iter()
        .map(|c| {
            remplacer
                .iter()
                .find(|(nom, _)| nom == c)
                .map(|(_, e)| e.clone())
                .unwrap_or_else(|| format!("x.{c}"))
        })
        .collect();
    sqlx::query(&format!(
        "INSERT INTO {table} ({}) SELECT {} FROM {table} x JOIN seq_1_to_{n} s WHERE x.id = ?",
        colonnes.join(", "),
        exprs.join(", ")
    ))
    .bind(id)
    .execute(pool)
    .await
    .unwrap_or_else(|e| panic!("copie de {table} : {e}"));
}

/// AC8 — `EXPLAIN` des requêtes A et B sur une base de volume : 20 000
/// factures — chacune avec sa ligne, un règlement et un avoir (copies, sans
/// cohérence comptable : seuls les plans sont jugés) — sur 20 000 écritures de
/// 40 000 lignes hors du compte vu ; 5 000 écritures portant chacune une ligne
/// sur le compte vu, dont 2 500 lettrées par paires. `cargo nextest run -p
/// kesh-db -E 'binary(open_items)' --run-ignored only --no-capture`.
#[ignore = "volume — lancé à la main pour le Dev Agent Record (AC8)"]
#[sqlx::test(migrations = "./test-schema")]
async fn explain_plans(pool: MySqlPool) {
    let m = monde(&pool).await;
    let company = m.company();
    let compte = m.creance();
    let j = jours_avant(30);
    // Modèles : une facture réglée en partie, une facture créditée.
    let modele = facture(&pool, &m.s, dec!(5.00), j).await;
    let (sid, _) = regler(&pool, &m.s, modele, dec!(1.00), j).await;
    let creditee = facture(&pool, &m.s, dec!(5.00), j).await;
    crediter(&pool, &m.s, creditee, j).await;
    let avoir: i64 = sqlx::query_scalar("SELECT id FROM credit_notes WHERE invoice_id = ?")
        .bind(creditee)
        .fetch_one(&pool)
        .await
        .unwrap();
    let ligne_facture: i64 =
        sqlx::query_scalar("SELECT id FROM invoice_lines WHERE invoice_id = ?")
            .bind(modele)
            .fetch_one(&pool)
            .await
            .unwrap();
    let ligne_avoir: i64 =
        sqlx::query_scalar("SELECT id FROM credit_note_lines WHERE credit_note_id = ?")
            .bind(avoir)
            .fetch_one(&pool)
            .await
            .unwrap();
    // 20 000 écritures hors du compte vu, deux lignes chacune.
    sqlx::query(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, \
         journal, description) SELECT ?, ?, 100000 + seq, ?, 'Ventes', 'volume' \
         FROM seq_1_to_20000",
    )
    .bind(company)
    .bind(m.fy)
    .bind(j)
    .execute(&pool)
    .await
    .unwrap();
    for (compte_autre, ordre) in [(m.s.accounts["1000"], 1), (m.s.accounts["3000"], 2)] {
        sqlx::query(
            "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
             SELECT id, ?, ?, 5, 5 - 5 FROM journal_entries WHERE description = 'volume'",
        )
        .bind(compte_autre)
        .bind(ordre)
        .execute(&pool)
        .await
        .unwrap();
    }
    let premiere: i64 =
        sqlx::query_scalar("SELECT MIN(id) FROM journal_entries WHERE description = 'volume'")
            .fetch_one(&pool)
            .await
            .unwrap();
    // 20 000 factures, lignes, règlements, avoirs et lignes d'avoir.
    copier(
        &pool,
        "invoices",
        modele,
        20000,
        &[
            ("invoice_number", "CONCAT('VOLF-', s.seq)".into()),
            ("journal_entry_id", format!("{premiere} + s.seq - 1")),
        ],
    )
    .await;
    let premiere_facture: i64 =
        sqlx::query_scalar("SELECT MIN(id) FROM invoices WHERE invoice_number LIKE 'VOLF-%'")
            .fetch_one(&pool)
            .await
            .unwrap();
    copier(
        &pool,
        "invoice_lines",
        ligne_facture,
        20000,
        &[("invoice_id", format!("{premiere_facture} + s.seq - 1"))],
    )
    .await;
    copier(
        &pool,
        "invoice_settlements",
        sid,
        20000,
        &[
            ("invoice_id", format!("{premiere_facture} + s.seq - 1")),
            ("journal_entry_id", format!("{premiere} + s.seq - 1")),
        ],
    )
    .await;
    copier(
        &pool,
        "credit_notes",
        avoir,
        20000,
        &[
            ("invoice_id", format!("{premiere_facture} + s.seq - 1")),
            ("credit_note_number", "CONCAT('VOLAV-', s.seq)".into()),
            ("journal_entry_id", format!("{premiere} + s.seq - 1")),
        ],
    )
    .await;
    let premier_avoir: i64 = sqlx::query_scalar(
        "SELECT MIN(id) FROM credit_notes WHERE credit_note_number LIKE 'VOLAV-%'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    copier(
        &pool,
        "credit_note_lines",
        ligne_avoir,
        20000,
        &[("credit_note_id", format!("{premier_avoir} + s.seq - 1"))],
    )
    .await;
    // Le compte vu : 5 000 écritures d'une ligne, la moitié lettrées par paires
    // (SQL brut : montage de volume, aucune règle n'est jugée ici).
    sqlx::query(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, \
         journal, description) SELECT ?, ?, 200000 + seq, ?, 'OD', 'vu' FROM seq_1_to_5000",
    )
    .bind(company)
    .bind(m.fy)
    .bind(jours_avant(20))
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
         SELECT id, ?, 1, IF(entry_number % 2, 1, 0), IF(entry_number % 2, 0, 1) \
         FROM journal_entries WHERE description = 'vu' ORDER BY id",
    )
    .bind(compte)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "UPDATE journal_entry_lines jel JOIN journal_entries je ON je.id = jel.entry_id \
         SET jel.lettering_key = jel.id - (je.entry_number + 1) % 2, \
             jel.lettering_origin = 'manual' \
         WHERE je.description = 'vu' AND je.entry_number <= 202500",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "ANALYZE TABLE journal_entry_lines, journal_entries, invoices, invoice_lines, \
         invoice_settlements, credit_notes, credit_note_lines",
    )
    .execute(&pool)
    .await
    .unwrap();

    let x = aujourdhui();
    let afficher = |titre: &str, rows: Vec<sqlx::mysql::MySqlRow>| {
        eprintln!("=== EXPLAIN {titre}");
        for r in &rows {
            eprintln!("{}", ligne_explain(r));
        }
    };
    let sql = format!("EXPLAIN {}", letterings::OPEN_ITEMS_TOTALS_SQL);
    let rows = sqlx::query(&sql)
        .bind(compte)
        .bind(company)
        .bind(compte)
        .bind(company)
        .bind(x)
        .bind(x)
        .fetch_all(&pool)
        .await
        .unwrap();
    afficher("A totaux", rows);
    let sql = format!("EXPLAIN {}", letterings::OPEN_ITEMS_PAGE_SQL);
    let rows = sqlx::query(&sql)
        .bind(compte)
        .bind(company)
        .bind(compte)
        .bind(company)
        .bind(x)
        .bind(x)
        .bind(50i64)
        .bind(0i64)
        .fetch_all(&pool)
        .await
        .unwrap();
    afficher("A page", rows);
    let sql = format!("EXPLAIN {}", letterings::OPEN_ITEMS_BALANCE_SQL);
    let rows = sqlx::query(&sql)
        .bind(compte)
        .bind(company)
        .bind(x)
        .fetch_all(&pool)
        .await
        .unwrap();
    afficher("A solde", rows);
    // B : 50 factures du volume (une page).
    let ids: Vec<i64> = (0..50).map(|k| premiere_facture + k * 400).collect();
    let sql = format!(
        "EXPLAIN {}",
        letterings::open_items_invoice_states_sql(ids.len())
    );
    let mut q = sqlx::query(&sql);
    for id in &ids {
        q = q.bind(*id);
    }
    let rows = q.bind(company).fetch_all(&pool).await.unwrap();
    afficher("B états des factures (50)", rows);

    // Temps, sur la même base.
    let mut c = pool.acquire().await.unwrap();
    let debut = std::time::Instant::now();
    let v = letterings::open_items(&mut c, company, compte, x, 50, 0)
        .await
        .unwrap();
    eprintln!("open_items, page de 50 : {:?}", debut.elapsed());
    assert_eq!(
        v.total,
        2500 + 2,
        "2 500 lignes ouvertes du volume et les 2 ventes modèles"
    );
    let debut = std::time::Instant::now();
    let sql = letterings::open_items_invoice_states_sql(ids.len());
    let mut q = sqlx::query(&sql);
    for id in &ids {
        q = q.bind(*id);
    }
    let lus = q.bind(company).fetch_all(&pool).await.unwrap();
    eprintln!(
        "requête B, 50 factures parmi 20 002 : {:?} ({} lignes)",
        debut.elapsed(),
        lus.len()
    );
}
