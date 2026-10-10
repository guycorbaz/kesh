//! Tests de dépôt — Story 15-1b-0 (#518) : la propriété des écritures, par lot
//! (`journal_entries::document_owners`), et `reversal_blockers` réécrit dessus.
//!
//! La parité se prouve contre un **oracle indépendant** (D4) :
//!
//! 1. l'ancien `reversal_blockers`, **gelé** dans ce binaire
//!    (`document_owners/reversal_blockers_frozen.rs`) — comparé écriture par
//!    écriture (`NotFound` compris) **et par lot** ;
//! 2. des **valeurs écrites à la main**, qui gardent l'oracle gelé contre un
//!    défaut qu'il partagerait avec le nouveau code.
//!
//! ⚠️ La fixture est posée en **SQL direct** (patron de `tests/letterings.rs`) :
//! aucun dépôt, pour pouvoir cumuler sur une écriture ce que les gestes
//! interdisent — le schéma, lui, le permet (modèle réel de la fiche).

use std::collections::BTreeMap;

use chrono::NaiveDate;
use kesh_db::errors::{DbError, ReversalBlocker};
use kesh_db::repositories::journal_entries::{
    self, DocumentKind, DocumentOwner, ReversalBlockerHit,
};
use kesh_db::test_fixtures::{seed_accounting_company, seed_contact_and_product};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::{MySqlConnection, MySqlPool, Row};

#[path = "document_owners/reversal_blockers_frozen.rs"]
mod frozen;

// ---------------------------------------------------------------------------
// Montage
// ---------------------------------------------------------------------------

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

/// Écriture posée en SQL direct (numéro suivant de l'exercice) ; rend son id.
async fn ecriture(
    pool: &MySqlPool,
    company_id: i64,
    fy: i64,
    lignes: &[(i64, Decimal, Decimal)],
) -> i64 {
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
         journal, description) VALUES (?, ?, ?, ?, 'OD', 'propriété')",
    )
    .bind(company_id)
    .bind(fy)
    .bind(numero)
    .bind(d(2026, 2, 1))
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    for (n, (account, debit, credit)) in lignes.iter().enumerate() {
        sqlx::query(
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
        .unwrap();
    }
    entry_id
}

async fn inserer(pool: &MySqlPool, sql: &str, binds: &[Option<i64>]) -> i64 {
    let mut q = sqlx::query(sql);
    for b in binds {
        q = q.bind(*b);
    }
    q.execute(pool).await.unwrap().last_insert_id() as i64
}

/// Une société et de quoi y poser des pièces.
struct Societe {
    company: i64,
    fy: i64,
    contact: i64,
    /// 1100 (débit) et 2000 (crédit) : les deux lignes d'une écriture saine.
    debit: i64,
    credit: i64,
    bank_account: i64,
    import: i64,
}

impl Societe {
    async fn ecriture(&self, pool: &MySqlPool) -> i64 {
        ecriture(
            pool,
            self.company,
            self.fy,
            &[
                (self.debit, dec!(100), dec!(0)),
                (self.credit, dec!(0), dec!(100)),
            ],
        )
        .await
    }

    async fn facture(&self, pool: &MySqlPool, entry: Option<i64>, numero: &str) -> i64 {
        sqlx::query(
            "INSERT INTO invoices (company_id, contact_id, date, journal_entry_id, invoice_number) \
             VALUES (?, ?, '2026-02-01', ?, ?)",
        )
        .bind(self.company)
        .bind(self.contact)
        .bind(entry)
        .bind(numero)
        .execute(pool)
        .await
        .unwrap()
        .last_insert_id() as i64
    }

    async fn facture_sans_numero(&self, pool: &MySqlPool, entry: Option<i64>) -> i64 {
        inserer(
            pool,
            "INSERT INTO invoices (company_id, contact_id, date, journal_entry_id) \
             VALUES (?, ?, '2026-02-01', ?)",
            &[Some(self.company), Some(self.contact), entry],
        )
        .await
    }

    /// Un avoir BROUILLON, sans numéro (`chk_credit_notes_issued_has_je` ne
    /// l'exige qu'à l'émission).
    async fn avoir_brouillon(&self, pool: &MySqlPool, entry: i64) -> i64 {
        let facture = self.facture_sans_numero(pool, None).await;
        inserer(
            pool,
            "INSERT INTO credit_notes (company_id, contact_id, invoice_id, status, date, \
             journal_entry_id) VALUES (?, ?, ?, 'draft', '2026-02-01', ?)",
            &[
                Some(self.company),
                Some(self.contact),
                Some(facture),
                Some(entry),
            ],
        )
        .await
    }

    async fn facture_fournisseur_sans_numero(&self, pool: &MySqlPool, achat: i64) -> i64 {
        inserer(
            pool,
            "INSERT INTO supplier_invoices (company_id, contact_id, invoice_date, \
             purchase_journal_entry_id) VALUES (?, ?, '2026-02-01', ?)",
            &[Some(self.company), Some(self.contact), Some(achat)],
        )
        .await
    }

    /// Un avoir (sur une facture brouillon neuve, `uq_credit_notes_invoice`).
    async fn avoir(&self, pool: &MySqlPool, entry: i64, numero: &str) -> i64 {
        let facture = self.facture(pool, None, &format!("{numero}-src")).await;
        sqlx::query(
            "INSERT INTO credit_notes (company_id, contact_id, invoice_id, status, date, \
             credit_note_number, journal_entry_id) VALUES (?, ?, ?, 'issued', '2026-02-01', ?, ?)",
        )
        .bind(self.company)
        .bind(self.contact)
        .bind(facture)
        .bind(numero)
        .bind(entry)
        .execute(pool)
        .await
        .unwrap()
        .last_insert_id() as i64
    }

    async fn facture_fournisseur(
        &self,
        pool: &MySqlPool,
        achat: i64,
        reglement: Option<i64>,
        numero: &str,
    ) -> i64 {
        sqlx::query(
            "INSERT INTO supplier_invoices (company_id, contact_id, invoice_date, \
             supplier_invoice_number, purchase_journal_entry_id, settlement_journal_entry_id) \
             VALUES (?, ?, '2026-02-01', ?, ?, ?)",
        )
        .bind(self.company)
        .bind(self.contact)
        .bind(numero)
        .bind(achat)
        .bind(reglement)
        .execute(pool)
        .await
        .unwrap()
        .last_insert_id() as i64
    }

    /// Une ligne `invoice_settlements` (type `internal_account`) de `facture`.
    async fn reglement(&self, pool: &MySqlPool, facture: i64, entry: i64) -> i64 {
        inserer(
            pool,
            "INSERT INTO invoice_settlements (company_id, invoice_id, journal_entry_id, amount, \
             settled_on, settlement_type, settlement_account_id) \
             VALUES (?, ?, ?, 100, '2026-02-01', 'internal_account', ?)",
            &[
                Some(self.company),
                Some(facture),
                Some(entry),
                Some(self.debit),
            ],
        )
        .await
    }

    async fn transaction(&self, pool: &MySqlPool, entry: i64) -> i64 {
        inserer(
            pool,
            "INSERT INTO bank_transactions (company_id, import_id, bank_account_id, booking_date, \
             amount, currency, details, status, matched_entry_id) \
             VALUES (?, ?, ?, '2026-02-01', 100, 'CHF', 'virement', 'reconciled', ?)",
            &[
                Some(self.company),
                Some(self.import),
                Some(self.bank_account),
                Some(entry),
            ],
        )
        .await
    }
}

async fn compte(pool: &MySqlPool, company: i64, numero: &str, ty: &str, actif: bool) -> i64 {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, active) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(company)
    .bind(numero)
    .bind(numero)
    .bind(ty)
    .bind(actif)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

async fn banque(pool: &MySqlPool, company: i64, user: i64, iban: &str) -> (i64, i64) {
    let bank: i64 =
        sqlx::query("INSERT INTO bank_accounts (company_id, bank_name, iban) VALUES (?, 'B', ?)")
            .bind(company)
            .bind(iban)
            .execute(pool)
            .await
            .unwrap()
            .last_insert_id() as i64;
    let import: i64 = sqlx::query(
        "INSERT INTO bank_imports (company_id, bank_account_id, filename, file_hash, \
         source_format, period_from, period_to, imported_by_user_id) \
         VALUES (?, ?, 'x.xml', REPEAT('a', 64), 'camt053', '2026-02-01', '2026-02-28', ?)",
    )
    .bind(company)
    .bind(bank)
    .bind(user)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    (bank, import)
}

async fn societe(pool: &MySqlPool) -> Societe {
    let s = seed_accounting_company(pool).await.expect("seed");
    let (contact, _) = seed_contact_and_product(pool, s.company_id)
        .await
        .expect("contact");
    let (bank_account, import) =
        banque(pool, s.company_id, s.admin_user_id, "CH9300762011623852957").await;
    Societe {
        company: s.company_id,
        fy: s.fiscal_year_id,
        contact,
        debit: s.accounts["1100"],
        credit: s.accounts["2000"],
        bank_account,
        import,
    }
}

/// Une seconde société (copie de la première), avec son exercice, ses comptes,
/// son contact et sa banque.
async fn autre_societe(pool: &MySqlPool, s: &Societe) -> Societe {
    let company: i64 = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         SELECT CONCAT(name, ' bis'), address, org_type, accounting_language, instance_language \
         FROM companies WHERE id = ?",
    )
    .bind(s.company)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let fy: i64 = sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, 'Exercice 2026', '2026-01-01', '2026-12-31', 'Open')",
    )
    .bind(company)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let (contact, _) = seed_contact_and_product(pool, company)
        .await
        .expect("contact");
    let user: i64 = sqlx::query_scalar("SELECT MIN(id) FROM users")
        .fetch_one(pool)
        .await
        .unwrap();
    let (bank_account, import) = banque(pool, company, user, "CH5604835012345678009").await;
    Societe {
        company,
        fy,
        contact,
        debit: compte(pool, company, "1100", "Asset", true).await,
        credit: compte(pool, company, "2000", "Liability", true).await,
        bank_account,
        import,
    }
}

async fn contre_passer(pool: &MySqlPool, reversal: i64, origine: i64) {
    sqlx::query("UPDATE journal_entries SET reverses_entry_id = ? WHERE id = ?")
        .bind(origine)
        .bind(reversal)
        .execute(pool)
        .await
        .unwrap();
}

/// La fixture des huit rangs, seuls et cumulés.
struct Fixture {
    s: Societe,
    autre: Societe,
    // Pièces seules.
    e_facture: i64,
    facture: i64,
    e_avoir: i64,
    avoir: i64,
    /// Achat et règlement d'UNE même facture fournisseur — tous deux du lot.
    e_achat: i64,
    e_paiement: i64,
    fournisseur: i64,
    /// Règlement encaissé par rapprochement : `OwnedBySettlement` ET
    /// `MatchedBankTransaction`.
    e_reglement: i64,
    reglement: i64,
    facture_reglee: i64,
    transaction_reglement: i64,
    e_banque: i64,
    transaction: i64,
    // Rangs 1, 2, 8 seuls.
    e_origine: i64,
    e_inverse: i64,
    e_archivee: i64,
    // Les huit rangs sur UNE écriture : les cinq types, contre-passation et
    // contre-passée, compte archivé.
    e_tout: i64,
    tout_facture: i64,
    tout_avoir: i64,
    tout_fournisseur: i64,
    tout_reglement: i64,
    tout_facture_reglee: i64,
    tout_transaction: i64,
    /// Sans propriétaire.
    e_libre: i64,
    /// Pièces SANS numéro (facture, avoir brouillon, facture fournisseur,
    /// règlement d'une facture sans numéro) : étiquettes `None`.
    e_sans_numero: i64,
    sans_numero_facture: i64,
    sans_numero_avoir: i64,
    sans_numero_fournisseur: i64,
    sans_numero_reglement: i64,
    sans_numero_facture_reglee: i64,
    sans_numero_transaction: i64,
    /// Écriture de l'autre société, possédée par les CINQ types de pièces de
    /// l'autre société — chaque bloc de l'`UNION ALL` a sa portée éprouvée.
    e_autre: i64,
    autre_proprietaires: Vec<DocumentOwner>,
    /// Identifiant qu'aucune écriture ne porte.
    inexistante: i64,
}

impl Fixture {
    /// Les écritures de la société, dans l'ordre de leur pose.
    fn ecritures(&self) -> Vec<i64> {
        vec![
            self.e_facture,
            self.e_avoir,
            self.e_achat,
            self.e_paiement,
            self.e_reglement,
            self.e_banque,
            self.e_origine,
            self.e_inverse,
            self.e_archivee,
            self.e_tout,
            self.e_sans_numero,
            self.e_libre,
        ]
    }
}

async fn fixture(pool: &MySqlPool) -> Fixture {
    let s = societe(pool).await;
    let autre = autre_societe(pool, &s).await;
    let archive = compte(pool, s.company, "6999", "Expense", false).await;

    let e_facture = s.ecriture(pool).await;
    let facture = s.facture(pool, Some(e_facture), "F-2026-001").await;
    let e_avoir = s.ecriture(pool).await;
    let avoir = s.avoir(pool, e_avoir, "AV-2026-001").await;
    let e_achat = s.ecriture(pool).await;
    let e_paiement = s.ecriture(pool).await;
    let fournisseur = s
        .facture_fournisseur(pool, e_achat, Some(e_paiement), "FF-77")
        .await;
    let e_reglement = s.ecriture(pool).await;
    let facture_reglee = s.facture(pool, None, "F-2026-002").await;
    let reglement = s.reglement(pool, facture_reglee, e_reglement).await;
    let transaction_reglement = s.transaction(pool, e_reglement).await;
    let e_banque = s.ecriture(pool).await;
    let transaction = s.transaction(pool, e_banque).await;
    let e_origine = s.ecriture(pool).await;
    let e_inverse = s.ecriture(pool).await;
    contre_passer(pool, e_inverse, e_origine).await;
    let e_archivee = ecriture(
        pool,
        s.company,
        s.fy,
        &[
            (archive, dec!(100), dec!(0)),
            (s.credit, dec!(0), dec!(100)),
        ],
    )
    .await;

    // L'écriture aux huit rangs : elle contre-passe `cible`, `inverse_tout` la
    // contre-passe, une ligne porte le compte archivé, et les cinq types la
    // visent.
    let cible = s.ecriture(pool).await;
    let e_tout = ecriture(
        pool,
        s.company,
        s.fy,
        &[
            (archive, dec!(100), dec!(0)),
            (s.credit, dec!(0), dec!(100)),
        ],
    )
    .await;
    contre_passer(pool, e_tout, cible).await;
    let inverse_tout = s.ecriture(pool).await;
    contre_passer(pool, inverse_tout, e_tout).await;
    let tout_facture = s.facture(pool, Some(e_tout), "F-2026-005").await;
    let tout_avoir = s.avoir(pool, e_tout, "AV-2026-005").await;
    let tout_fournisseur = s.facture_fournisseur(pool, e_tout, None, "FF-5").await;
    let tout_facture_reglee = s.facture(pool, None, "F-2026-006").await;
    let tout_reglement = s.reglement(pool, tout_facture_reglee, e_tout).await;
    let tout_transaction = s.transaction(pool, e_tout).await;

    let e_libre = s.ecriture(pool).await;

    // Une pièce de chaque type, sans numéro (le schéma l'admet pour les quatre
    // types numérotés) ; le règlement l'est d'une facture sans numéro.
    let e_sans_numero = s.ecriture(pool).await;
    let sans_numero_facture = s.facture_sans_numero(pool, Some(e_sans_numero)).await;
    let sans_numero_avoir = s.avoir_brouillon(pool, e_sans_numero).await;
    let sans_numero_fournisseur = s.facture_fournisseur_sans_numero(pool, e_sans_numero).await;
    let sans_numero_facture_reglee = s.facture_sans_numero(pool, None).await;
    let sans_numero_reglement = s
        .reglement(pool, sans_numero_facture_reglee, e_sans_numero)
        .await;
    let sans_numero_transaction = s.transaction(pool, e_sans_numero).await;

    let e_autre = autre.ecriture(pool).await;
    let autre_facture = autre.facture(pool, Some(e_autre), "F-AUTRE-1").await;
    let autre_avoir = autre.avoir(pool, e_autre, "AV-AUTRE-1").await;
    let autre_fournisseur = autre
        .facture_fournisseur(pool, e_autre, None, "FF-AUTRE-1")
        .await;
    let autre_facture_reglee = autre.facture(pool, None, "F-AUTRE-2").await;
    let autre_reglement = autre.reglement(pool, autre_facture_reglee, e_autre).await;
    let autre_transaction = autre.transaction(pool, e_autre).await;
    use DocumentKind::*;
    let autre_proprietaires = vec![
        possede(Invoice, autre_facture, Some("F-AUTRE-1"), None),
        possede(CreditNote, autre_avoir, Some("AV-AUTRE-1"), None),
        possede(SupplierInvoice, autre_fournisseur, Some("FF-AUTRE-1"), None),
        possede(
            Settlement,
            autre_reglement,
            None,
            Some((autre_facture_reglee, Some("F-AUTRE-2"))),
        ),
        possede(BankTransaction, autre_transaction, None, None),
    ];

    let inexistante: i64 = sqlx::query_scalar("SELECT MAX(id) + 1000 FROM journal_entries")
        .fetch_one(pool)
        .await
        .unwrap();

    Fixture {
        s,
        autre,
        e_facture,
        facture,
        e_avoir,
        avoir,
        e_achat,
        e_paiement,
        fournisseur,
        e_reglement,
        reglement,
        facture_reglee,
        transaction_reglement,
        e_banque,
        transaction,
        e_origine,
        e_inverse,
        e_archivee,
        e_tout,
        tout_facture,
        tout_avoir,
        tout_fournisseur,
        tout_reglement,
        tout_facture_reglee,
        tout_transaction,
        e_libre,
        e_sans_numero,
        sans_numero_facture,
        sans_numero_avoir,
        sans_numero_fournisseur,
        sans_numero_reglement,
        sans_numero_facture_reglee,
        sans_numero_transaction,
        e_autre,
        autre_proprietaires,
        inexistante,
    }
}

/// Les rangs 3 à 7 d'une liste de motifs : ceux de la propriété.
fn rangs_de_propriete(motifs: &[ReversalBlockerHit]) -> Vec<ReversalBlockerHit> {
    motifs
        .iter()
        .filter(|(b, _, _)| {
            !matches!(
                b,
                ReversalBlocker::IsAReversal
                    | ReversalBlocker::AlreadyReversed
                    | ReversalBlocker::AccountArchived
            )
        })
        .cloned()
        .collect()
}

/// La projection d'un vecteur de propriétaires sur les motifs de
/// contre-passation — par `DocumentKind::reversal_blocker`, l'identifiant, et
/// le numéro de la pièce comme étiquette.
fn projection(owners: Option<&Vec<DocumentOwner>>) -> Vec<ReversalBlockerHit> {
    owners
        .map(|v| {
            v.iter()
                .map(|o| (o.kind.reversal_blocker(), Some(o.id), o.number.clone()))
                .collect()
        })
        .unwrap_or_default()
}

fn possede(
    kind: DocumentKind,
    id: i64,
    number: Option<&str>,
    invoice: Option<(i64, Option<&str>)>,
) -> DocumentOwner {
    DocumentOwner {
        kind,
        id,
        number: number.map(str::to_string),
        invoice_id: invoice.map(|(i, _)| i),
        invoice_number: invoice.and_then(|(_, n)| n.map(str::to_string)),
    }
}

// ---------------------------------------------------------------------------
// D4 (1) — l'oracle gelé
// ---------------------------------------------------------------------------

/// AC2 — `reversal_blockers` (nouveau) == l'ancien, gelé, pour chaque écriture
/// de la fixture (motif, identifiant, étiquette, ordre ; `NotFound` compris) ;
/// et UN appel par lot de `document_owners` sur toute la fixture égale, pour
/// chaque écriture, les rangs 3 à 7 de l'oracle — l'achat et le règlement d'une
/// même facture fournisseur dans le même lot.
#[sqlx::test(migrations = "./test-schema")]
async fn owners_match_the_frozen_reversal_blockers(pool: MySqlPool) {
    let f = fixture(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let c = f.s.company;

    // Assertion de montage : les huit rangs sont exercés par la fixture.
    let mut vus = std::collections::BTreeSet::new();
    for e in f.ecritures() {
        for (b, _, _) in frozen::reversal_blockers_frozen(&mut *conn, c, e)
            .await
            .unwrap()
        {
            vus.insert(b.code());
        }
    }
    assert_eq!(vus.len(), 8, "les huit rangs doivent être montés : {vus:?}");

    // Par écriture.
    for e in f.ecritures() {
        let ancien = frozen::reversal_blockers_frozen(&mut *conn, c, e)
            .await
            .unwrap();
        let nouveau = journal_entries::reversal_blockers(&mut conn, c, e)
            .await
            .unwrap();
        assert_eq!(nouveau, ancien, "écriture {e}");
        assert_eq!(
            journal_entries::reversal_blocker(&mut conn, c, e)
                .await
                .unwrap(),
            ancien.first().cloned(),
            "premier motif, écriture {e}"
        );
    }
    // `NotFound` : écriture d'une autre société, écriture inexistante.
    for e in [f.e_autre, f.inexistante] {
        assert!(matches!(
            frozen::reversal_blockers_frozen(&mut *conn, c, e).await,
            Err(DbError::NotFound)
        ));
        assert!(
            matches!(
                journal_entries::reversal_blockers(&mut conn, c, e).await,
                Err(DbError::NotFound)
            ),
            "écriture {e}"
        );
        assert!(matches!(
            journal_entries::reversal_blocker(&mut conn, c, e).await,
            Err(DbError::NotFound)
        ));
    }

    // Par lot : un seul appel sur toute la fixture (et les deux absentes).
    let mut lot = f.ecritures();
    lot.extend([f.e_autre, f.inexistante]);
    let owners = journal_entries::document_owners(&mut conn, c, &lot)
        .await
        .unwrap();
    assert!(
        owners.contains_key(&f.e_achat) && owners.contains_key(&f.e_paiement),
        "achat ET règlement d'une même facture fournisseur, dans le même lot"
    );
    for e in f.ecritures() {
        let ancien = frozen::reversal_blockers_frozen(&mut *conn, c, e)
            .await
            .unwrap();
        assert_eq!(
            projection(owners.get(&e)),
            rangs_de_propriete(&ancien),
            "lot, écriture {e}"
        );
        assert_eq!(
            owners.contains_key(&e),
            !rangs_de_propriete(&ancien).is_empty(),
            "absente de la table = aucun motif de propriété, écriture {e}"
        );
    }
    assert!(!owners.contains_key(&f.e_autre));
    assert!(!owners.contains_key(&f.inexistante));
}

// ---------------------------------------------------------------------------
// D4 (2) — les valeurs écrites à la main
// ---------------------------------------------------------------------------

/// D4 (2) — pour une écriture de chaque type et pour l'écriture aux cinq
/// types, le `Vec<DocumentOwner>` attendu, dans l'ordre ; les écritures sans
/// propriétaire sont ABSENTES de la table ; deux pièces d'un même type sur une
/// écriture (factures, avoirs, factures fournisseurs, transactions) rendent la
/// plus petite, numéro de la même ligne.
#[sqlx::test(migrations = "./test-schema")]
async fn owners_match_handwritten_expectations(pool: MySqlPool) {
    let f = fixture(&pool).await;
    let s = &f.s;
    // Deux factures et deux transactions sur une écriture (hors de l'oracle :
    // son `LIMIT 1` sans `ORDER BY` n'y serait pas déterministe).
    let e_double = s.ecriture(&pool).await;
    let double_1 = s.facture(&pool, Some(e_double), "F-DOUBLE-1").await;
    let double_2 = s.facture(&pool, Some(e_double), "F-DOUBLE-2").await;
    let banque_1 = s.transaction(&pool, e_double).await;
    let banque_2 = s.transaction(&pool, e_double).await;
    // Et deux avoirs, deux factures fournisseurs : une jointure de retour sur
    // l'ÉCRITURE (au lieu de `id`) y rendrait deux lignes (mutation (e)).
    let avoir_1 = s.avoir(&pool, e_double, "AV-DOUBLE-1").await;
    let avoir_2 = s.avoir(&pool, e_double, "AV-DOUBLE-2").await;
    let fournisseur_1 = s
        .facture_fournisseur(&pool, e_double, None, "FF-DOUBLE-1")
        .await;
    let fournisseur_2 = s
        .facture_fournisseur(&pool, e_double, None, "FF-DOUBLE-2")
        .await;
    assert!(
        double_1 < double_2
            && banque_1 < banque_2
            && avoir_1 < avoir_2
            && fournisseur_1 < fournisseur_2,
        "montage"
    );

    // Écriture croisée : achat de la facture fournisseur S1 ET règlement de S2
    // (données héritées) — une seule ligne fournisseur, la plus petite.
    let e_croise = s.ecriture(&pool).await;
    let autre_achat = s.ecriture(&pool).await;
    let s1 = s.facture_fournisseur(&pool, e_croise, None, "FF-S1").await;
    let s2 = s
        .facture_fournisseur(&pool, autre_achat, Some(e_croise), "FF-S2")
        .await;
    assert!(s1 < s2, "montage");

    let mut conn = pool.acquire().await.unwrap();
    let mut lot = f.ecritures();
    lot.extend([e_double, e_croise, autre_achat]);
    let owners = journal_entries::document_owners(&mut conn, s.company, &lot)
        .await
        .unwrap();

    use DocumentKind::*;
    let attendu: BTreeMap<i64, Vec<DocumentOwner>> = [
        (
            f.e_facture,
            vec![possede(Invoice, f.facture, Some("F-2026-001"), None)],
        ),
        (
            f.e_avoir,
            vec![possede(CreditNote, f.avoir, Some("AV-2026-001"), None)],
        ),
        (
            f.e_achat,
            vec![possede(SupplierInvoice, f.fournisseur, Some("FF-77"), None)],
        ),
        (
            f.e_paiement,
            vec![possede(SupplierInvoice, f.fournisseur, Some("FF-77"), None)],
        ),
        (
            f.e_reglement,
            vec![
                possede(
                    Settlement,
                    f.reglement,
                    None,
                    Some((f.facture_reglee, Some("F-2026-002"))),
                ),
                possede(BankTransaction, f.transaction_reglement, None, None),
            ],
        ),
        (
            f.e_banque,
            vec![possede(BankTransaction, f.transaction, None, None)],
        ),
        (
            f.e_tout,
            vec![
                possede(Invoice, f.tout_facture, Some("F-2026-005"), None),
                possede(CreditNote, f.tout_avoir, Some("AV-2026-005"), None),
                possede(SupplierInvoice, f.tout_fournisseur, Some("FF-5"), None),
                possede(
                    Settlement,
                    f.tout_reglement,
                    None,
                    Some((f.tout_facture_reglee, Some("F-2026-006"))),
                ),
                possede(BankTransaction, f.tout_transaction, None, None),
            ],
        ),
        (
            f.e_sans_numero,
            vec![
                possede(Invoice, f.sans_numero_facture, None, None),
                possede(CreditNote, f.sans_numero_avoir, None, None),
                possede(SupplierInvoice, f.sans_numero_fournisseur, None, None),
                possede(
                    Settlement,
                    f.sans_numero_reglement,
                    None,
                    Some((f.sans_numero_facture_reglee, None)),
                ),
                possede(BankTransaction, f.sans_numero_transaction, None, None),
            ],
        ),
        (
            e_croise,
            vec![possede(SupplierInvoice, s1, Some("FF-S1"), None)],
        ),
        (
            autre_achat,
            vec![possede(SupplierInvoice, s2, Some("FF-S2"), None)],
        ),
        (
            e_double,
            vec![
                possede(Invoice, double_1, Some("F-DOUBLE-1"), None),
                possede(CreditNote, avoir_1, Some("AV-DOUBLE-1"), None),
                possede(SupplierInvoice, fournisseur_1, Some("FF-DOUBLE-1"), None),
                possede(BankTransaction, banque_1, None, None),
            ],
        ),
    ]
    .into_iter()
    .collect();
    assert_eq!(owners, attendu);
    // Absentes — jamais présentes avec un vecteur vide.
    for e in [f.e_origine, f.e_inverse, f.e_archivee, f.e_libre] {
        assert!(!owners.contains_key(&e), "écriture {e} sans propriétaire");
    }
    assert!(owners.values().all(|v| !v.is_empty()));
}

/// D1, D4 (2) — la table des trois méthodes de `DocumentKind`, cinq variants,
/// valeurs écrites à la main. Les chaînes de `as_str` sont celles que publient
/// les tests d'audit de la 15-1a2-i et de la 15-1a2-ii (`documentType`).
#[test]
fn document_kind_table_is_closed() {
    use DocumentKind::*;
    let table = [
        (Invoice, ReversalBlocker::OwnedByInvoice, true, "invoice"),
        (
            CreditNote,
            ReversalBlocker::OwnedByCreditNote,
            true,
            "creditNote",
        ),
        (
            SupplierInvoice,
            ReversalBlocker::OwnedBySupplierInvoice,
            true,
            "supplierInvoice",
        ),
        (
            Settlement,
            ReversalBlocker::OwnedBySettlement,
            true,
            "settlement",
        ),
        (
            BankTransaction,
            ReversalBlocker::MatchedBankTransaction,
            false,
            "bankTransaction",
        ),
    ];
    for (kind, motif, bloque, chaine) in table {
        assert_eq!(kind.reversal_blocker(), motif, "{kind:?}");
        assert_eq!(kind.blocks_manual_lettering(), bloque, "{kind:?}");
        assert_eq!(kind.as_str(), chaine, "{kind:?}");
    }
    // L'ordre de précédence (rangs 3 à 7) est l'ordre des variants.
    let mut tri = [
        BankTransaction,
        Settlement,
        Invoice,
        SupplierInvoice,
        CreditNote,
    ];
    tri.sort();
    assert_eq!(
        tri,
        [
            Invoice,
            CreditNote,
            SupplierInvoice,
            Settlement,
            BankTransaction
        ]
    );
    // Table fermée : un variant neuf rend ce `match` non exhaustif.
    for (kind, _, _, _) in table {
        match kind {
            Invoice | CreditNote | SupplierInvoice | Settlement | BankTransaction => {}
        }
    }
}

// ---------------------------------------------------------------------------
// AC1 — le lot, mesuré
// ---------------------------------------------------------------------------

/// `Com_stmt_execute` de la session, lu en protocole TEXTE (`raw_sql`) : une
/// lecture par `sqlx::query` passerait en protocole préparé et compterait
/// elle-même (validation P3 ciblée, P3C-1).
async fn executions(conn: &mut MySqlConnection) -> i64 {
    let row = sqlx::raw_sql("SHOW SESSION STATUS LIKE 'Com_stmt_execute'")
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    row.get::<String, _>(1).parse().unwrap()
}

/// AC1 — 1 200 écritures (possédées aux rangs 1, 501 et 1001 de l'ordre trié)
/// se lisent en TROIS instructions, chacune recevant son propriétaire ; une
/// liste vide ne coûte rien ; des doublons ne changent ni le résultat ni le
/// coût. Étalonnage du compteur en tête (deux mesures, Dev Agent Record).
#[sqlx::test(migrations = "./test-schema")]
async fn owners_are_read_by_batches_of_500(pool: MySqlPool) {
    let s = societe(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    // Étalonnage (i) : la lecture ne se compte pas elle-même.
    let a = executions(&mut conn).await;
    let b = executions(&mut conn).await;
    let surcout = b - a;
    assert_eq!(surcout, 0, "étalonnage (i) : deux lectures successives");
    // Étalonnage (ii) : une exécution préparée compte UNE unité.
    let a = executions(&mut conn).await;
    sqlx::query("SELECT ?")
        .bind(1_i64)
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    let b = executions(&mut conn).await;
    assert_eq!(b - a, 1, "étalonnage (ii) : un `SELECT ?` lié");

    // 1 200 écritures, par un INSERT multi-valeurs.
    let valeurs = (1..=1200)
        .map(|n| format!("({}, {}, {n}, '2026-02-01', 'OD', 'lot')", s.company, s.fy))
        .collect::<Vec<_>>()
        .join(",");
    sqlx::raw_sql(&format!(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, \
         journal, description) VALUES {valeurs}"
    ))
    .execute(&mut *conn)
    .await
    .unwrap();
    let ids: Vec<i64> = sqlx::query_scalar(
        "SELECT id FROM journal_entries WHERE company_id = ? AND description = 'lot' ORDER BY id",
    )
    .bind(s.company)
    .fetch_all(&mut *conn)
    .await
    .unwrap();
    assert_eq!(ids.len(), 1200, "montage");
    // Possédées aux rangs 1, 501, 1001 : une par tranche.
    let mut factures = BTreeMap::new();
    for rang in [0_usize, 500, 1000] {
        let facture = s
            .facture(&pool, Some(ids[rang]), &format!("F-LOT-{rang}"))
            .await;
        factures.insert(ids[rang], (facture, format!("F-LOT-{rang}")));
    }

    // Liste vide : aucune instruction.
    let avant = executions(&mut conn).await;
    let vide = journal_entries::document_owners(&mut conn, s.company, &[])
        .await
        .unwrap();
    assert!(vide.is_empty());
    assert_eq!(executions(&mut conn).await - avant, 0, "liste vide");

    // Les bornes de la tranche (revue P1, A-2 = E4) : 500 → une instruction,
    // 501 → deux, 1 000 → deux, 1 001 → trois.
    for (n, attendu) in [(500_usize, 1_i64), (501, 2), (1000, 2), (1001, 3)] {
        let avant = executions(&mut conn).await;
        journal_entries::document_owners(&mut conn, s.company, &ids[..n])
            .await
            .unwrap();
        assert_eq!(
            executions(&mut conn).await - avant,
            attendu + surcout,
            "{n} écritures"
        );
    }

    // 1 200 écritures, dans le désordre : trois tranches, trois instructions.
    let mut desordre = ids.clone();
    desordre.reverse();
    let avant = executions(&mut conn).await;
    let owners = journal_entries::document_owners(&mut conn, s.company, &desordre)
        .await
        .unwrap();
    let delta = executions(&mut conn).await - avant;
    assert_eq!(delta, 3 + surcout, "trois tranches de 500 au plus");
    let attendu: BTreeMap<i64, Vec<DocumentOwner>> = factures
        .iter()
        .map(|(e, (id, numero))| {
            (
                *e,
                vec![possede(DocumentKind::Invoice, *id, Some(numero), None)],
            )
        })
        .collect();
    assert_eq!(
        owners, attendu,
        "chaque tranche lue, chaque propriétaire le sien"
    );

    // Doublons : même résultat, même coût.
    let mut doubles = desordre.clone();
    doubles.extend(ids.iter().take(300));
    doubles.extend(ids.iter().take(10));
    let avant = executions(&mut conn).await;
    let avec_doublons = journal_entries::document_owners(&mut conn, s.company, &doubles)
        .await
        .unwrap();
    assert_eq!(
        executions(&mut conn).await - avant,
        delta,
        "doublons : même coût"
    );
    assert_eq!(avec_doublons, owners, "doublons : même résultat");
}

/// AC1, D4 (4) — la portée par société : l'écriture d'une autre société, que
/// visent les cinq types de pièces de cette société, n'a aucun propriétaire,
/// seule ou dans un lot ; lue sous sa propre société, elle a les cinq.
#[sqlx::test(migrations = "./test-schema")]
async fn owners_are_scoped_by_company(pool: MySqlPool) {
    let f = fixture(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let seule = journal_entries::document_owners(&mut conn, f.s.company, &[f.e_autre])
        .await
        .unwrap();
    assert!(seule.is_empty(), "écriture d'une autre société : {seule:?}");
    let lot = journal_entries::document_owners(&mut conn, f.s.company, &[f.e_facture, f.e_autre])
        .await
        .unwrap();
    assert_eq!(lot.keys().copied().collect::<Vec<_>>(), vec![f.e_facture]);
    // Assertion de montage : chez elle, l'écriture porte les CINQ types — la
    // table vide ci-dessus n'est donc pas une fixture creuse, et chaque bloc de
    // l'`UNION ALL` voit sa portée éprouvée (revue P1, A-1 = B-1 = E1).
    let chez_elle =
        journal_entries::document_owners(&mut conn, f.autre.company, &[f.e_autre, f.e_facture])
            .await
            .unwrap();
    assert_eq!(f.autre_proprietaires.len(), 5, "montage : cinq types");
    assert_eq!(
        chez_elle,
        [(f.e_autre, f.autre_proprietaires.clone())]
            .into_iter()
            .collect()
    );
}
