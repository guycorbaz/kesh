//! Story 55-3a (#1607) — les justificatifs d'une écriture : la définition
//! commune des cinq rattachements (`kesh_report::documents`), le booléen
//! `hasDocuments` du journal et du grand livre, et le contrat additif.
//!
//! Les données sont **posées en SQL direct** quand le cas est un négatif que
//! l'application n'écrit pas (avoir brouillon portant une écriture, document
//! écarté, ligne d'une société qui pointe l'écriture d'une autre) : le schéma
//! les admet (aucune contrainte ne les refuse, relevé au T0), et chaque filtre
//! n'est éprouvé que par une ligne qu'il est seul à écarter.
//!
//! Les tests appellent `entries_with_documents` et `documents_of_entry`
//! **directement** (AC-4), ici et non dans un `mod tests` de `src/` (AC-5 :
//! `src/` ne porte que la définition et les deux appels des rapports).
//!
//! Pré-requis : MariaDB démarré (`sqlx::test` crée une base éphémère par test).

use std::collections::BTreeMap;

use chrono::NaiveDate;
use kesh_db::entities::audit_log::Acteur;
use kesh_db::entities::journal_entry::{Journal, NewJournalEntry, NewJournalEntryLine};
use kesh_db::repositories::journal_entries;
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use kesh_report::documents::{
    EntryDocument, EntryDocumentKind, PdfStatus, documents_of_entry, entries_with_documents,
};
use kesh_report::general_ledger::{LedgerOptions, LedgerPeriod};
use kesh_report::period::ReportPeriod;
use kesh_report::{
    GeneralLedger, JournalReport, PdfContext, generate_general_ledger, generate_journal_report,
    render_general_ledger_csv, render_general_ledger_pdf, render_journal_report_csv,
    render_journal_report_pdf, texte_pdf,
};
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("date valide")
}

const DEBUT: (i32, u32, u32) = (2026, 1, 1);
const FIN: (i32, u32, u32) = (2026, 12, 31);

/// La société A (seedée), son contact, et la société B posée en SQL avec le
/// sien.
struct Ctx {
    a: SeededCompany,
    contact_a: i64,
    b: i64,
    contact_b: i64,
    /// Compteur des numéros et des empreintes (uniques par société).
    seq: std::cell::Cell<u32>,
}

impl Ctx {
    fn suivant(&self) -> u32 {
        let n = self.seq.get() + 1;
        self.seq.set(n);
        n
    }
    fn compte(&self, code: &str) -> i64 {
        self.a.accounts[code]
    }
}

async fn contact(pool: &MySqlPool, company_id: i64, nom: &str) -> i64 {
    sqlx::query("INSERT INTO contacts (company_id, contact_type, name) VALUES (?, 'Entreprise', ?)")
        .bind(company_id)
        .bind(nom)
        .execute(pool)
        .await
        .expect("contact")
        .last_insert_id() as i64
}

async fn poser(pool: &MySqlPool) -> Ctx {
    let a = seed_accounting_company(pool).await.expect("seed");
    let contact_a = contact(pool, a.company_id, "Client A").await;
    let b = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Société B', 'Rue B 1\n1000 Lausanne', 'Independant', 'FR', 'FR')",
    )
    .execute(pool)
    .await
    .expect("société B")
    .last_insert_id() as i64;
    let contact_b = contact(pool, b, "Client B").await;
    Ctx {
        a,
        contact_a,
        b,
        contact_b,
        seq: Default::default(),
    }
}

/// Une écriture équilibrée de la société A, au débit de 1100 et au crédit de
/// 3000 : deux sections du grand livre.
async fn ecriture(pool: &MySqlPool, ctx: &Ctx, date: NaiveDate, libelle: &str) -> i64 {
    journal_entries::create(
        pool,
        ctx.a.fiscal_year_id,
        Acteur::utilisateur(ctx.a.admin_user_id),
        NewJournalEntry {
            company_id: ctx.a.company_id,
            entry_date: date,
            journal: Journal::Ventes,
            description: libelle.into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: ctx.compte("1100"),
                    debit: dec!(10),
                    credit: dec!(0),
                    project_id: None,
                    foreign: None,
                },
                NewJournalEntryLine {
                    account_id: ctx.compte("3000"),
                    debit: dec!(0),
                    credit: dec!(10),
                    project_id: None,
                    foreign: None,
                },
            ],
        },
    )
    .await
    .expect("écriture")
    .entry
    .id
}

/// Facture client posée en SQL : `status`, écriture, figée ou non.
async fn facture(
    pool: &MySqlPool,
    ctx: &Ctx,
    company_id: i64,
    contact_id: i64,
    status: &str,
    je: Option<i64>,
    figee: bool,
) -> i64 {
    let n = ctx.suivant();
    let (chemin, empreinte, langue) = if figee {
        (
            Some(format!("invoices/f-{n}.pdf")),
            Some(format!("{n:064}")),
            Some("FR"),
        )
    } else {
        (None, None, None)
    };
    sqlx::query(
        "INSERT INTO invoices (company_id, contact_id, invoice_number, status, date, \
             journal_entry_id, pdf_storage_path, pdf_sha256, pdf_frozen_at, pdf_language) \
         VALUES (?, ?, ?, ?, '2026-03-01', ?, ?, ?, IF(? IS NULL, NULL, NOW(3)), ?)",
    )
    .bind(company_id)
    .bind(contact_id)
    .bind(format!("F-{n}"))
    .bind(status)
    .bind(je)
    .bind(&chemin)
    .bind(&empreinte)
    .bind(&chemin)
    .bind(langue)
    .execute(pool)
    .await
    .expect("facture")
    .last_insert_id() as i64
}

/// Règlement d'une facture (`internal_account` ou `write_off`).
async fn reglement(
    pool: &MySqlPool,
    ctx: &Ctx,
    company_id: i64,
    invoice_id: i64,
    je: i64,
    genre: &str,
) -> i64 {
    let (nature, tva) = if genre == "write_off" {
        (Some("discount"), Some("[]"))
    } else {
        (None, None)
    };
    // `bank_transfer` vise un compte bancaire, les deux autres un compte du
    // plan (`chk_invoice_settlements_counterparty`).
    let (banque, compte) = if genre == "bank_transfer" {
        let banque = sqlx::query(
            "INSERT INTO bank_accounts (company_id, bank_name, iban) \
             VALUES (?, 'Banque', 'CH9300762011623852957')",
        )
        .bind(company_id)
        .execute(pool)
        .await
        .expect("compte bancaire")
        .last_insert_id() as i64;
        (Some(banque), None)
    } else {
        (None, Some(ctx.compte("1000")))
    };
    sqlx::query(
        "INSERT INTO invoice_settlements (company_id, invoice_id, journal_entry_id, amount, \
             settled_on, settlement_type, settlement_bank_account_id, settlement_account_id, \
             write_off_nature, write_off_vat) \
         VALUES (?, ?, ?, 1, '2026-03-10', ?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(invoice_id)
    .bind(je)
    .bind(genre)
    .bind(banque)
    .bind(compte)
    .bind(nature)
    .bind(tva)
    .execute(pool)
    .await
    .expect("règlement")
    .last_insert_id() as i64
}

/// Avoir posé en SQL (sur une facture qui lui est propre : un avoir par
/// facture).
async fn avoir(
    pool: &MySqlPool,
    ctx: &Ctx,
    company_id: i64,
    contact_id: i64,
    status: &str,
    je: i64,
) -> i64 {
    let facture_id = facture(pool, ctx, company_id, contact_id, "draft", None, false).await;
    let n = ctx.suivant();
    sqlx::query(
        "INSERT INTO credit_notes (company_id, contact_id, invoice_id, credit_note_number, \
             status, date, journal_entry_id) VALUES (?, ?, ?, ?, ?, '2026-03-05', ?)",
    )
    .bind(company_id)
    .bind(contact_id)
    .bind(facture_id)
    .bind(format!("AV-{n}"))
    .bind(status)
    .bind(je)
    .execute(pool)
    .await
    .expect("avoir")
    .last_insert_id() as i64
}

/// Facture fournisseur posée en SQL : achat `purchase`, règlement facultatif
/// (statut `paid`), sinon `status`.
async fn facture_fournisseur(
    pool: &MySqlPool,
    ctx: &Ctx,
    company_id: i64,
    contact_id: i64,
    status: &str,
    achat: i64,
    paiement: Option<i64>,
) -> i64 {
    let n = ctx.suivant();
    sqlx::query(
        "INSERT INTO supplier_invoices (company_id, contact_id, supplier_invoice_number, \
             internal_number, status, invoice_date, purchase_journal_entry_id, \
             settlement_type, settlement_account_id, settlement_journal_entry_id, paid_at) \
         VALUES (?, ?, ?, ?, ?, '2026-02-01', ?, IF(? IS NULL, NULL, 'internal_account'), \
                 IF(? IS NULL, NULL, ?), ?, IF(? IS NULL, NULL, NOW(3)))",
    )
    .bind(company_id)
    .bind(contact_id)
    .bind(format!("EXT-{n}"))
    .bind(format!("FF-{n}"))
    .bind(status)
    .bind(achat)
    .bind(paiement)
    .bind(paiement)
    .bind(ctx.compte("1000"))
    .bind(paiement)
    .bind(paiement)
    .execute(pool)
    .await
    .expect("facture fournisseur")
    .last_insert_id() as i64
}

/// Document importé, rattaché à la facture fournisseur `si`.
async fn document(pool: &MySqlPool, ctx: &Ctx, company_id: i64, si: i64, status: &str) -> String {
    let n = ctx.suivant();
    let nom = format!("doc-{n}.pdf");
    sqlx::query(
        "INSERT INTO imported_supplier_invoices (company_id, status, supplier_invoice_id, \
             file_hash, storage_path, original_filename, mime_type, byte_size) \
         VALUES (?, ?, ?, ?, ?, ?, 'application/pdf', 1)",
    )
    .bind(company_id)
    .bind(status)
    .bind(si)
    .bind(format!("{n:064}"))
    .bind(format!("inbox/{nom}"))
    .bind(&nom)
    .execute(pool)
    .await
    .expect("document");
    nom
}

/// Les justificatifs d'une écriture, réduits à `(genre, id, état du PDF)`.
async fn liste(pool: &MySqlPool, company_id: i64, je: i64) -> Vec<EntryDocument> {
    documents_of_entry(pool, company_id, je)
        .await
        .expect("documents_of_entry")
}

/// Un justificatif réduit à ce que les cas comparent.
type Resume = (EntryDocumentKind, i64, Option<PdfStatus>);

fn genres(docs: &[EntryDocument]) -> Vec<Resume> {
    docs.iter().map(|d| (d.kind, d.id, d.pdf_status)).collect()
}

fn periode() -> (NaiveDate, NaiveDate) {
    (ymd(DEBUT.0, DEBUT.1, DEBUT.2), ymd(FIN.0, FIN.1, FIN.2))
}

async fn avec_justificatifs(pool: &MySqlPool, company_id: i64) -> std::collections::HashSet<i64> {
    let (from, to) = periode();
    entries_with_documents(pool, company_id, from, to)
        .await
        .expect("entries_with_documents")
}

/// Les cas de l'AC-3 et de l'AC-10 : une écriture chacun, la liste attendue,
/// puis la comparaison du booléen du journal et du grand livre à la liste,
/// sur tous les cas (même constructeur).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac3_chaque_rattachement_rend_sa_liste_et_le_booleen_la_suit(pool: MySqlPool) {
    let ctx = poser(&pool).await;
    let a = ctx.a.company_id;
    let c = ctx.contact_a;
    let jour = ymd(2026, 3, 1);
    let mut attendu: BTreeMap<&str, (i64, Vec<Resume>)> = BTreeMap::new();

    // Facture validée non figée : un justificatif, `notFrozen`.
    let e = ecriture(&pool, &ctx, jour, "validée non figée").await;
    let f = facture(&pool, &ctx, a, c, "validated", Some(e), false).await;
    attendu.insert(
        "validée non figée",
        (
            e,
            vec![(EntryDocumentKind::Invoice, f, Some(PdfStatus::NotFrozen))],
        ),
    );
    let f_reglee = f;

    // Facture validée figée : `frozen`.
    let e = ecriture(&pool, &ctx, jour, "validée figée").await;
    let f = facture(&pool, &ctx, a, c, "validated", Some(e), true).await;
    attendu.insert(
        "validée figée",
        (
            e,
            vec![(EntryDocumentKind::Invoice, f, Some(PdfStatus::Frozen))],
        ),
    );
    let f_figee = f;

    // Validée non figée, envoyée (`emailed_at`) : `notFrozen` aussi (D12).
    let e = ecriture(&pool, &ctx, jour, "envoyée non figée").await;
    let f = facture(&pool, &ctx, a, c, "validated", Some(e), false).await;
    sqlx::query("UPDATE invoices SET emailed_at = NOW(6) WHERE id = ?")
        .bind(f)
        .execute(&pool)
        .await
        .unwrap();
    attendu.insert(
        "envoyée non figée",
        (
            e,
            vec![(EntryDocumentKind::Invoice, f, Some(PdfStatus::NotFrozen))],
        ),
    );

    // Facture annulée figée : un justificatif.
    let e = ecriture(&pool, &ctx, jour, "annulée figée").await;
    let f = facture(&pool, &ctx, a, c, "cancelled", Some(e), true).await;
    attendu.insert(
        "annulée figée",
        (
            e,
            vec![(EntryDocumentKind::Invoice, f, Some(PdfStatus::Frozen))],
        ),
    );

    // Facture annulée jamais figée : aucun.
    let e = ecriture(&pool, &ctx, jour, "annulée non figée").await;
    facture(&pool, &ctx, a, c, "cancelled", Some(e), false).await;
    attendu.insert("annulée non figée", (e, vec![]));

    // Règlement simple (virement) : la facture réglée.
    let e = ecriture(&pool, &ctx, jour, "règlement simple").await;
    reglement(&pool, &ctx, a, f_reglee, e, "bank_transfer").await;
    attendu.insert(
        "règlement simple",
        (
            e,
            vec![(
                EntryDocumentKind::Invoice,
                f_reglee,
                Some(PdfStatus::NotFrozen),
            )],
        ),
    );

    // Règlement par compte interne : la facture réglée.
    let e = ecriture(&pool, &ctx, jour, "règlement interne").await;
    reglement(&pool, &ctx, a, f_reglee, e, "internal_account").await;
    attendu.insert(
        "règlement interne",
        (
            e,
            vec![(
                EntryDocumentKind::Invoice,
                f_reglee,
                Some(PdfStatus::NotFrozen),
            )],
        ),
    );

    // Solde du reste : la facture.
    let e = ecriture(&pool, &ctx, jour, "solde du reste").await;
    reglement(&pool, &ctx, a, f_figee, e, "write_off").await;
    attendu.insert(
        "solde du reste",
        (
            e,
            vec![(EntryDocumentKind::Invoice, f_figee, Some(PdfStatus::Frozen))],
        ),
    );

    // Règlement client groupé de deux factures : deux, par identifiant.
    let e_ventes1 = ecriture(&pool, &ctx, jour, "vente groupée 1").await;
    let g1 = facture(&pool, &ctx, a, c, "validated", Some(e_ventes1), false).await;
    let e_ventes2 = ecriture(&pool, &ctx, jour, "vente groupée 2").await;
    let g2 = facture(&pool, &ctx, a, c, "validated", Some(e_ventes2), true).await;
    attendu.insert(
        "vente groupée 1",
        (
            e_ventes1,
            vec![(EntryDocumentKind::Invoice, g1, Some(PdfStatus::NotFrozen))],
        ),
    );
    attendu.insert(
        "vente groupée 2",
        (
            e_ventes2,
            vec![(EntryDocumentKind::Invoice, g2, Some(PdfStatus::Frozen))],
        ),
    );
    let e = ecriture(&pool, &ctx, jour, "règlement groupé").await;
    // Inséré dans l'ordre inverse : la liste trie par identifiant.
    reglement(&pool, &ctx, a, g2, e, "internal_account").await;
    reglement(&pool, &ctx, a, g1, e, "internal_account").await;
    attendu.insert(
        "règlement groupé",
        (
            e,
            vec![
                (EntryDocumentKind::Invoice, g1, Some(PdfStatus::NotFrozen)),
                (EntryDocumentKind::Invoice, g2, Some(PdfStatus::Frozen)),
            ],
        ),
    );

    // Avoir émis : un.
    let e = ecriture(&pool, &ctx, jour, "avoir émis").await;
    let av = avoir(&pool, &ctx, a, c, "issued", e).await;
    attendu.insert(
        "avoir émis",
        (e, vec![(EntryDocumentKind::CreditNote, av, None)]),
    );

    // Facture fournisseur à document : l'achat et le règlement.
    let achat = ecriture(&pool, &ctx, jour, "achat à document").await;
    let paiement = ecriture(&pool, &ctx, jour, "paiement à document").await;
    let si = facture_fournisseur(&pool, &ctx, a, c, "paid", achat, Some(paiement)).await;
    document(&pool, &ctx, a, si, "completed").await;
    attendu.insert(
        "achat à document",
        (achat, vec![(EntryDocumentKind::SupplierDocument, si, None)]),
    );
    attendu.insert(
        "paiement à document",
        (
            paiement,
            vec![(EntryDocumentKind::SupplierDocument, si, None)],
        ),
    );

    // Règlement fournisseur groupé de deux factures à document : deux.
    let paiement_groupe = ecriture(&pool, &ctx, jour, "paiement groupé").await;
    let mut groupe = Vec::new();
    for i in 0..2 {
        let achat = ecriture(&pool, &ctx, jour, &format!("achat groupé {i}")).await;
        let si = facture_fournisseur(&pool, &ctx, a, c, "paid", achat, Some(paiement_groupe)).await;
        document(&pool, &ctx, a, si, "completed").await;
        groupe.push(si);
        attendu.insert(
            if i == 0 {
                "achat groupé 0"
            } else {
                "achat groupé 1"
            },
            (achat, vec![(EntryDocumentKind::SupplierDocument, si, None)]),
        );
    }
    attendu.insert(
        "paiement groupé",
        (
            paiement_groupe,
            groupe
                .iter()
                .map(|si| (EntryDocumentKind::SupplierDocument, *si, None))
                .collect(),
        ),
    );

    // Facture fournisseur sans document : aucun.
    let e = ecriture(&pool, &ctx, jour, "achat sans document").await;
    facture_fournisseur(&pool, &ctx, a, c, "open", e, None).await;
    attendu.insert("achat sans document", (e, vec![]));

    // Facture fournisseur annulée, écriture d'achat : le document.
    let e = ecriture(&pool, &ctx, jour, "achat annulé").await;
    let si = facture_fournisseur(&pool, &ctx, a, c, "cancelled", e, None).await;
    document(&pool, &ctx, a, si, "completed").await;
    attendu.insert(
        "achat annulé",
        (e, vec![(EntryDocumentKind::SupplierDocument, si, None)]),
    );

    // Une écriture sans rien.
    let e = ecriture(&pool, &ctx, jour, "sans rien").await;
    attendu.insert("sans rien", (e, vec![]));

    for (cas, (e, docs)) in &attendu {
        assert_eq!(&genres(&liste(&pool, a, *e).await), docs, "cas « {cas} »");
    }

    // Le document fournisseur porte son nom de fichier et son numéro interne ;
    // la facture et l'avoir leur numéro, sans nom de fichier.
    let docs = liste(&pool, a, attendu["achat à document"].0).await;
    assert!(
        docs[0]
            .file_name
            .as_deref()
            .is_some_and(|n| n.starts_with("doc-"))
    );
    assert!(
        docs[0]
            .number
            .as_deref()
            .is_some_and(|n| n.starts_with("FF-"))
    );
    let docs = liste(&pool, a, attendu["avoir émis"].0).await;
    assert!(
        docs[0]
            .number
            .as_deref()
            .is_some_and(|n| n.starts_with("AV-"))
    );
    assert_eq!(docs[0].file_name, None);

    // Le booléen du journal et du grand livre est vrai SI ET SEULEMENT SI la
    // liste est non vide, sur tous les cas.
    let ensemble = avec_justificatifs(&pool, a).await;
    let (from, to) = periode();
    let journal = generate_journal_report(
        &pool,
        a,
        &ReportPeriod {
            fiscal_year_id: ctx.a.fiscal_year_id,
            start_date: from,
            end_date: to,
        },
        None,
    )
    .await
    .expect("journal");
    let ledger = generate_general_ledger(
        &pool,
        a,
        &LedgerPeriod::new(from, to).unwrap(),
        &LedgerOptions {
            account_ids: None,
            include_zero: false,
            offset: 0,
            limit: None,
        },
    )
    .await
    .expect("grand livre");
    let mut vus_journal = 0;
    for row in journal.journals.iter().flat_map(|j| &j.entries) {
        let attendu = !liste(&pool, a, row.entry_id).await.is_empty();
        assert_eq!(
            row.has_documents, attendu,
            "journal, écriture {}",
            row.entry_id
        );
        assert_eq!(ensemble.contains(&row.entry_id), attendu);
        vus_journal += 1;
    }
    assert_eq!(
        vus_journal,
        attendu.len(),
        "toutes les écritures au journal"
    );
    // Chaque écriture a une ligne au débit de 1100 et une au crédit de 3000 :
    // elle se lit dans deux sections, et chacune porte le booléen.
    let mut sections_par_ecriture: BTreeMap<i64, usize> = BTreeMap::new();
    for line in ledger.sections.iter().flat_map(|s| &s.lines) {
        let attendu = !liste(&pool, a, line.entry_id).await.is_empty();
        assert_eq!(
            line.has_documents, attendu,
            "grand livre, écriture {}",
            line.entry_id
        );
        *sections_par_ecriture.entry(line.entry_id).or_default() += 1;
    }
    assert_eq!(sections_par_ecriture.len(), attendu.len());
    assert!(sections_par_ecriture.values().all(|n| *n == 2));
    assert!(
        journal
            .journals
            .iter()
            .flat_map(|j| &j.entries)
            .any(|r| r.has_documents)
            && journal
                .journals
                .iter()
                .flat_map(|j| &j.entries)
                .any(|r| !r.has_documents),
        "les deux valeurs sont exercées"
    );
}

/// AC-4 — chaque filtre, éprouvé par une ligne qu'il est seul à écarter ;
/// appels directs, société A puis B.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac4_les_filtres_ecartent_ce_que_le_schema_admet(pool: MySqlPool) {
    let ctx = poser(&pool).await;
    let (a, b) = (ctx.a.company_id, ctx.b);
    let (ca, cb) = (ctx.contact_a, ctx.contact_b);
    let jour = ymd(2026, 4, 1);
    let mut negatifs: Vec<(&str, i64)> = Vec::new();

    // Statuts : avoir brouillon portant une écriture ; facture brouillon
    // portant une écriture ; document écarté d'une facture fournisseur qui
    // porte l'achat.
    let e = ecriture(&pool, &ctx, jour, "avoir brouillon").await;
    avoir(&pool, &ctx, a, ca, "draft", e).await;
    negatifs.push(("cn.status", e));
    let e = ecriture(&pool, &ctx, jour, "facture brouillon").await;
    facture(&pool, &ctx, a, ca, "draft", Some(e), false).await;
    negatifs.push(("i.status (vente)", e));
    let e = ecriture(&pool, &ctx, jour, "règlement d'une facture brouillon").await;
    let brouillon = facture(&pool, &ctx, a, ca, "draft", None, false).await;
    reglement(&pool, &ctx, a, brouillon, e, "internal_account").await;
    negatifs.push(("i.status (règlement)", e));
    let e = ecriture(&pool, &ctx, jour, "document écarté").await;
    let si = facture_fournisseur(&pool, &ctx, a, ca, "open", e, None).await;
    document(&pool, &ctx, a, si, "discarded").await;
    negatifs.push(("isi.status (achat)", e));
    let achat = ecriture(&pool, &ctx, jour, "achat d'un document écarté").await;
    let e = ecriture(&pool, &ctx, jour, "paiement d'un document écarté").await;
    let si = facture_fournisseur(&pool, &ctx, a, ca, "paid", achat, Some(e)).await;
    document(&pool, &ctx, a, si, "to_complete").await;
    negatifs.push(("isi.status (règlement)", e));
    negatifs.push(("isi.status (achat, à compléter)", achat));

    // Sociétés, par branche : une ligne de B qui pointe une écriture de A.
    let e = ecriture(&pool, &ctx, jour, "facture de B").await;
    facture(&pool, &ctx, b, cb, "validated", Some(e), true).await;
    negatifs.push(("i.company_id (vente)", e));
    let e = ecriture(&pool, &ctx, jour, "règlement de B d'une facture de A").await;
    // `validated` exige une écriture (`chk_invoices_validated_has_je`) : la
    // facture de A porte la sienne.
    let e_fa = ecriture(&pool, &ctx, jour, "vente de A").await;
    let fa = facture(&pool, &ctx, a, ca, "validated", Some(e_fa), false).await;
    reglement(&pool, &ctx, b, fa, e, "internal_account").await;
    negatifs.push(("s.company_id", e));
    let e = ecriture(&pool, &ctx, jour, "règlement de A d'une facture de B").await;
    let e_fb = ecriture(&pool, &ctx, jour, "écriture de la facture de B").await;
    let fb = facture(&pool, &ctx, b, cb, "validated", Some(e_fb), true).await;
    reglement(&pool, &ctx, a, fb, e, "internal_account").await;
    negatifs.push(("i.company_id (règlement)", e));
    negatifs.push(("i.company_id (vente, bis)", e_fb));
    let e = ecriture(&pool, &ctx, jour, "avoir de B").await;
    avoir(&pool, &ctx, b, cb, "issued", e).await;
    negatifs.push(("cn.company_id", e));
    let e = ecriture(&pool, &ctx, jour, "facture fournisseur de B, document de A").await;
    let si_b = facture_fournisseur(&pool, &ctx, b, cb, "open", e, None).await;
    document(&pool, &ctx, a, si_b, "completed").await;
    negatifs.push(("si.company_id (achat)", e));
    let achat = ecriture(&pool, &ctx, jour, "achat de A").await;
    let e = ecriture(&pool, &ctx, jour, "paiement de B").await;
    let si_b = facture_fournisseur(&pool, &ctx, b, cb, "paid", achat, Some(e)).await;
    document(&pool, &ctx, a, si_b, "completed").await;
    negatifs.push(("si.company_id (achat, bis)", achat));
    negatifs.push(("si.company_id (règlement)", e));
    let e = ecriture(&pool, &ctx, jour, "document de B, facture fournisseur de A").await;
    let si_a = facture_fournisseur(&pool, &ctx, a, ca, "open", e, None).await;
    document(&pool, &ctx, b, si_a, "completed").await;
    negatifs.push(("isi.company_id", e));

    let ensemble = avec_justificatifs(&pool, a).await;
    for (filtre, e) in &negatifs {
        assert!(
            liste(&pool, a, *e).await.is_empty(),
            "filtre {filtre} : l'écriture {e} ne doit rien rendre"
        );
        assert!(
            !ensemble.contains(e),
            "filtre {filtre} : écriture {e} au journal"
        );
    }
    // Contrôle positif : la vente de A est bien vue (le montage n'est pas vide).
    assert!(ensemble.contains(&e_fa));
    assert_eq!(liste(&pool, a, e_fa).await.len(), 1);

    // `je.company_id` : la société B ne voit aucune écriture de A, même celles
    // qui portent une ligne de B.
    assert!(avec_justificatifs(&pool, b).await.is_empty());
    assert!(liste(&pool, b, e_fa).await.is_empty());
}

/// AC-5 — la période borne la lecture : une écriture hors période n'est pas
/// lue ; le règlement annulé (supprimé) ne rattache plus rien
/// (non-régression).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac5_hors_periode_et_reglement_supprime(pool: MySqlPool) {
    let ctx = poser(&pool).await;
    let a = ctx.a.company_id;
    let dans = ecriture(&pool, &ctx, ymd(2026, 12, 31), "dernier jour").await;
    facture(
        &pool,
        &ctx,
        a,
        ctx.contact_a,
        "validated",
        Some(dans),
        false,
    )
    .await;
    let hors = ecriture(&pool, &ctx, ymd(2027, 1, 1), "lendemain").await;
    facture(
        &pool,
        &ctx,
        a,
        ctx.contact_a,
        "validated",
        Some(hors),
        false,
    )
    .await;
    let avant = ecriture(&pool, &ctx, ymd(2025, 12, 31), "veille").await;
    facture(
        &pool,
        &ctx,
        a,
        ctx.contact_a,
        "validated",
        Some(avant),
        false,
    )
    .await;

    let ensemble = avec_justificatifs(&pool, a).await;
    assert!(ensemble.contains(&dans));
    assert!(!ensemble.contains(&hors));
    assert!(!ensemble.contains(&avant));
    // La liste d'une écriture ne dépend d'aucune période.
    assert_eq!(liste(&pool, a, hors).await.len(), 1);

    let reg = ecriture(&pool, &ctx, ymd(2026, 6, 1), "règlement annulé").await;
    let f = facture(&pool, &ctx, a, ctx.contact_a, "validated", Some(dans), true).await;
    let s = reglement(&pool, &ctx, a, f, reg, "internal_account").await;
    assert_eq!(liste(&pool, a, reg).await.len(), 1);
    sqlx::query("DELETE FROM invoice_settlements WHERE id = ?")
        .bind(s)
        .execute(&pool)
        .await
        .unwrap();
    assert!(liste(&pool, a, reg).await.is_empty());
    assert!(!avec_justificatifs(&pool, a).await.contains(&reg));
}

/// AC-3 — dédoublonnage : une facture rattachée deux fois à la même écriture
/// (par la vente et par un règlement, ce que rien au schéma n'interdit) ne
/// sort qu'une fois.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac3_une_facture_ne_sort_qu_une_fois(pool: MySqlPool) {
    let ctx = poser(&pool).await;
    let a = ctx.a.company_id;
    let e = ecriture(&pool, &ctx, ymd(2026, 5, 1), "vente et règlement").await;
    let f = facture(&pool, &ctx, a, ctx.contact_a, "validated", Some(e), false).await;
    reglement(&pool, &ctx, a, f, e, "internal_account").await;
    assert_eq!(
        genres(&liste(&pool, a, e).await),
        vec![(EntryDocumentKind::Invoice, f, Some(PdfStatus::NotFrozen))]
    );
}

// ============================================================
// AC-6 — contrat additif : JSON et exports
// ============================================================

fn cles(v: &serde_json::Value) -> Vec<String> {
    let mut c: Vec<String> = v.as_object().expect("objet").keys().cloned().collect();
    c.sort();
    c
}

/// AC-6 — une écriture sans justificatif sérialise exactement les clés d'avant
/// la 55-3a ; avec, `hasDocuments: true` s'y ajoute et rien d'autre. Les
/// exports CSV (chaînes égales) et PDF (texte extrait égal, qui contient le
/// numéro de l'écriture) du journal et du grand livre ne changent pas selon
/// `has_documents`. Garde de non-régression : le champ n'est lu par aucun
/// rendu, elle ne rougit qu'à une régression.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac6_le_champ_est_additif_et_les_exports_l_ignorent(pool: MySqlPool) {
    let ctx = poser(&pool).await;
    let a = ctx.a.company_id;
    let avec = ecriture(&pool, &ctx, ymd(2026, 3, 1), "Vente JUSTIF-55").await;
    facture(&pool, &ctx, a, ctx.contact_a, "validated", Some(avec), true).await;
    let sans = ecriture(&pool, &ctx, ymd(2026, 3, 2), "Vente SANS-55").await;
    let (from, to) = periode();

    let journal = generate_journal_report(
        &pool,
        a,
        &ReportPeriod {
            fiscal_year_id: ctx.a.fiscal_year_id,
            start_date: from,
            end_date: to,
        },
        None,
    )
    .await
    .unwrap();
    let ledger = generate_general_ledger(
        &pool,
        a,
        &LedgerPeriod::new(from, to).unwrap(),
        &LedgerOptions {
            account_ids: None,
            include_zero: false,
            offset: 0,
            limit: None,
        },
    )
    .await
    .unwrap();

    // JSON, journal.
    let json = serde_json::to_value(&journal).unwrap();
    let rows: Vec<&serde_json::Value> = json["journals"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|j| j["entries"].as_array().unwrap())
        .collect();
    let row = |id: i64| *rows.iter().find(|r| r["entryId"] == id).unwrap();
    let avant = [
        "description",
        "entryDate",
        "entryId",
        "entryNumber",
        "lines",
    ];
    assert_eq!(
        cles(row(sans)),
        avant,
        "écriture sans justificatif : JSON d'avant"
    );
    let mut avec_cle: Vec<&str> = avant.to_vec();
    avec_cle.push("hasDocuments");
    avec_cle.sort();
    assert_eq!(cles(row(avec)), avec_cle);
    assert_eq!(row(avec)["hasDocuments"], true);

    // JSON, grand livre.
    let json = serde_json::to_value(&ledger).unwrap();
    let lignes: Vec<&serde_json::Value> = json["sections"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|s| s["lines"].as_array().unwrap())
        .collect();
    let avant = [
        "counterpart",
        "credit",
        "debit",
        "description",
        "entryDate",
        "entryId",
        "entryNumber",
        "fiscalYearId",
        "fiscalYearName",
        "journal",
        "letteringCode",
        "lineId",
        "runningBalance",
    ];
    for l in &lignes {
        if l["entryId"] == sans {
            assert_eq!(cles(l), avant, "ligne sans justificatif : JSON d'avant");
        } else {
            let mut attendu: Vec<&str> = avant.to_vec();
            attendu.push("hasDocuments");
            attendu.sort();
            assert_eq!(cles(l), attendu);
        }
    }

    // Exports : le même rapport, champ basculé partout.
    let bascule_journal = |valeur: bool| {
        let mut j = journal.clone();
        for row in j.journals.iter_mut().flat_map(|s| s.entries.iter_mut()) {
            row.has_documents = valeur;
        }
        j
    };
    let bascule_ledger = |valeur: bool| {
        let mut g = ledger.clone();
        for line in g.sections.iter_mut().flat_map(|s| s.lines.iter_mut()) {
            line.has_documents = valeur;
        }
        g
    };
    let numero = journal
        .journals
        .iter()
        .flat_map(|s| &s.entries)
        .next()
        .unwrap()
        .entry_number;
    let ctx_pdf = PdfContext::fr_ch_default("CI Test Company");

    let csv_journal = |j: &JournalReport| {
        let mut out = Vec::new();
        render_journal_report_csv(j, &mut out).unwrap();
        String::from_utf8(out).unwrap()
    };
    let csv_ledger = |g: &GeneralLedger| {
        let mut out = Vec::new();
        render_general_ledger_csv(g, &mut out).unwrap();
        String::from_utf8(out).unwrap()
    };
    assert_eq!(
        csv_journal(&bascule_journal(true)),
        csv_journal(&bascule_journal(false))
    );
    assert_eq!(
        csv_ledger(&bascule_ledger(true)),
        csv_ledger(&bascule_ledger(false))
    );

    let texte = |pdf: Vec<u8>| texte_pdf::lignes(&pdf);
    let pj_vrai = texte(render_journal_report_pdf(&bascule_journal(true), &ctx_pdf).unwrap());
    let pj_faux = texte(render_journal_report_pdf(&bascule_journal(false), &ctx_pdf).unwrap());
    assert!(
        pj_vrai.iter().any(|l| l.contains("JUSTIF-55")),
        "le texte extrait contient une écriture connue (n° {numero}) : {pj_vrai:?}"
    );
    assert_eq!(pj_vrai, pj_faux);
    let pl_vrai = texte(render_general_ledger_pdf(&bascule_ledger(true), &ctx_pdf).unwrap());
    let pl_faux = texte(render_general_ledger_pdf(&bascule_ledger(false), &ctx_pdf).unwrap());
    assert!(pl_vrai.iter().any(|l| l.contains("JUSTIF-55")));
    assert_eq!(pl_vrai, pl_faux);
}
