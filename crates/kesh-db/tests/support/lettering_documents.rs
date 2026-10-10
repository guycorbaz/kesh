//! Fichier d'appui **réservé aux tests** — Story 15-1a2-i (#518) : la fixture
//! partagée du lettrage des pièces clientes (`seed_lettering_documents`), le
//! prédicat d'AC5 (« lettrée `document` ») et les gestes de montage.
//!
//! ⛔ **Pourquoi ici, hors de `src/`** (validation P4, M-1 ; C-15-1a2-28) : les
//! états hérités se fabriquent en **SQL brut** — marques effacées, ligne d'avoir
//! déplacée sur un autre compte, compte de créance rattaché à un compte
//! bancaire. `kesh_db::test_fixtures` est compilé en permanence (endpoint
//! `_test/seed`) : c'est du code de production pour les deux détecteurs
//! lexicaux du lettrage, qui refuseraient ces littéraux. Ce fichier n'appartient
//! à aucune crate ; il est **inclus par `#[path]`** dans
//! `crates/kesh-db/tests/lettering_documents.rs`,
//! `crates/kesh-db/tests/letterings.rs` (AC9, `lettering_invariants`),
//! `crates/kesh-api/tests/rejeu_interblocage_e2e.rs` (AC15 c) et, à la
//! 15-1a2-ii, dans le binaire de son rattrapage
//! (`crates/kesh-db/tests/lettering_documents_backfill.rs`, son AC6). La
//! 15-1a2-ii y ajoute les **factures fournisseurs** (section en fin de
//! fichier).
//!
//! ⚠️ Chaque binaire n'en emploie qu'une partie — d'où l'`allow` ci-dessous :
//! un fichier d'appui partagé par plusieurs binaires, chacun sur un
//! sous-ensemble, n'a pas de « code mort » au sens du lint.
#![allow(dead_code)]

use chrono::NaiveDate;
use kesh_db::entities::{
    NewBankAccount, NewCreditNote, NewInvoice, NewInvoiceLine, NewPaymentBatch, NewSupplierInvoice,
    NewSupplierInvoiceLine, SettlementChoice, SettlementWriteOffNature,
};
use kesh_db::repositories::{
    bank_accounts, companies, credit_notes, invoice_settlements, invoice_settlements_write,
    invoices, letterings, payment_batches, supplier_invoices,
};
use kesh_db::test_fixtures::{SeededCompany, designate_rounding_account, seed_accounting_company};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

/// Aujourd'hui moins `n` jours — toutes les dates de montage sont PASSÉES :
/// `lock_books` et `unlock_books` refusent une borne `>= aujourd'hui`.
pub fn jours_avant(n: i64) -> NaiveDate {
    chrono::Utc::now().date_naive() - chrono::Duration::days(n)
}

/// Une société seedée, prête à régler, solder et créditer : comptes de nature
/// des soldes (4000), compte d'arrondi désigné (6940).
pub async fn societe(pool: &MySqlPool) -> SeededCompany {
    let seeded = seed_accounting_company(pool).await.expect("société");
    sqlx::query(
        "UPDATE company_invoice_settings SET default_discount_account_id = ?, \
         default_bank_fees_account_id = ?, default_bad_debt_account_id = ? WHERE company_id = ?",
    )
    .bind(seeded.accounts["4000"])
    .bind(seeded.accounts["4000"])
    .bind(seeded.accounts["4000"])
    .bind(seeded.company_id)
    .execute(pool)
    .await
    .expect("comptes de nature");
    designate_rounding_account(pool, seeded.company_id)
        .await
        .expect("compte d'arrondi");
    seeded
}

/// Une facture **validée par le chemin réel** (`validate_invoice` : numéro,
/// écriture de vente, arrondi selon les réglages), une ligne à 0 % de TVA.
pub async fn facture(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    ht: Decimal,
    date: NaiveDate,
) -> i64 {
    let contact_id: i64 = sqlx::query_scalar(
        "INSERT INTO contacts (company_id, contact_type, name, is_client) \
         VALUES (?, 'Entreprise', 'Client lettrage', TRUE) RETURNING id",
    )
    .bind(seeded.company_id)
    .fetch_one(pool)
    .await
    .expect("contact");
    let (inv, _) = invoices::create(
        pool,
        seeded.admin_user_id,
        NewInvoice {
            company_id: seeded.company_id,
            contact_id,
            date,
            due_date: Some(date),
            payment_terms: None,
            project_id: None,
            lines: vec![NewInvoiceLine {
                description: "Prestation".into(),
                quantity: dec!(1),
                unit_price: ht,
                vat_rate: dec!(0),
                revenue_account_id: Some(seeded.accounts["3000"]),
            }],
        },
    )
    .await
    .expect("facture créée");
    invoices::validate_invoice(pool, seeded.company_id, inv.id, seeded.admin_user_id)
        .await
        .expect("facture validée");
    inv.id
}

/// Règle `montant` en espèces (`internal_account` sur la caisse 1000) et rend
/// `(id de la ligne invoice_settlements, écriture de règlement)`.
pub async fn regler(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    inv: i64,
    montant: Decimal,
    le: NaiveDate,
) -> (i64, i64) {
    regler_par(
        pool,
        seeded,
        inv,
        SettlementChoice::InternalAccount {
            account_id: seeded.accounts["1000"],
        },
        montant,
        le,
    )
    .await
}

/// Règle `montant` par le mode `choix`.
pub async fn regler_par(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    inv: i64,
    choix: SettlementChoice,
    montant: Decimal,
    le: NaiveDate,
) -> (i64, i64) {
    let out = invoice_settlements_write::settle_invoice(
        pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv,
        choix,
        montant,
        le,
    )
    .await
    .expect("règlement");
    (
        ligne_de_reglement(pool, out.journal_entry_id).await,
        out.journal_entry_id,
    )
}

/// La ligne `invoice_settlements` d'une écriture de règlement.
pub async fn ligne_de_reglement(pool: &MySqlPool, entry_id: i64) -> i64 {
    sqlx::query_scalar("SELECT id FROM invoice_settlements WHERE journal_entry_id = ?")
        .bind(entry_id)
        .fetch_one(pool)
        .await
        .expect("ligne de règlement")
}

/// Solde le reste par la nature `nature` et rend `(ligne, écriture)`.
pub async fn solder(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    inv: i64,
    nature: SettlementWriteOffNature,
    le: NaiveDate,
) -> (i64, i64) {
    let version: i32 = sqlx::query_scalar("SELECT version FROM invoices WHERE id = ?")
        .bind(inv)
        .fetch_one(pool)
        .await
        .expect("version");
    let out = invoice_settlements_write::write_off_invoice(
        pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv,
        nature,
        le,
        version,
    )
    .await
    .expect("solde");
    (
        ligne_de_reglement(pool, out.journal_entry_id).await,
        out.journal_entry_id,
    )
}

/// Émet l'avoir total de la facture et rend l'écriture de l'avoir.
pub async fn crediter(pool: &MySqlPool, seeded: &SeededCompany, inv: i64, le: NaiveDate) -> i64 {
    credit_notes::create_credit_note(
        pool,
        NewCreditNote {
            company_id: seeded.company_id,
            invoice_id: inv,
            date: le,
        },
        seeded.admin_user_id,
    )
    .await
    .expect("avoir")
    .journal_entry
    .entry
    .id
}

/// L'écriture de vente d'une facture.
pub async fn vente(pool: &MySqlPool, inv: i64) -> i64 {
    sqlx::query_scalar("SELECT journal_entry_id FROM invoices WHERE id = ?")
        .bind(inv)
        .fetch_one(pool)
        .await
        .expect("écriture de vente")
}

/// Une ligne de `C(I)` : `(id, entry_id, clé, origine, exercice, date)`.
pub type LigneDePiece = (i64, i64, Option<i64>, Option<String>, i64, NaiveDate);

/// `C(I)`, **recalculé ici sans le code de production** : l'ancre (première
/// ligne au débit de l'écriture de vente) puis les lignes sur son compte des
/// écritures des règlements en vigueur et de l'avoir émis — triées par `id`.
pub async fn lignes_de_piece(pool: &MySqlPool, inv: i64) -> Vec<LigneDePiece> {
    sqlx::query_as(
        "WITH ancre AS ( \
             SELECT jel.id, jel.account_id, jel.entry_id FROM journal_entry_lines jel \
             JOIN invoices i ON i.journal_entry_id = jel.entry_id \
             WHERE i.id = ? AND jel.debit > 0 ORDER BY jel.id LIMIT 1) \
         SELECT jel.id, jel.entry_id, jel.lettering_key, jel.lettering_origin, \
                je.fiscal_year_id, je.entry_date \
         FROM journal_entry_lines jel JOIN journal_entries je ON je.id = jel.entry_id \
         JOIN ancre a ON jel.account_id = a.account_id \
         WHERE jel.id = a.id \
            OR jel.entry_id IN (SELECT journal_entry_id FROM invoice_settlements \
                                WHERE invoice_id = ?) \
            OR jel.entry_id IN (SELECT journal_entry_id FROM credit_notes \
                                WHERE invoice_id = ? AND status = 'issued') \
         ORDER BY jel.id",
    )
    .bind(inv)
    .bind(inv)
    .bind(inv)
    .fetch_all(pool)
    .await
    .expect("lignes de la pièce")
}

/// Le prédicat d'AC5, **écrit dans le test** : « lettrée `document` » signifie
/// que les lignes de `C(I)` forment **exactement un groupe**, et qu'il est
/// d'origine `document` — toutes ses lignes sous une même clé, et aucune autre
/// ligne sous cette clé.
pub async fn lettree_document(pool: &MySqlPool, inv: i64) -> bool {
    let lignes = lignes_de_piece(pool, inv).await;
    let Some(Some(cle)) = lignes.first().map(|l| l.2) else {
        return false;
    };
    let toutes = lignes
        .iter()
        .all(|l| l.2 == Some(cle) && l.3.as_deref() == Some("document"));
    let dans_le_groupe: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM journal_entry_lines WHERE lettering_key = ?")
            .bind(cle)
            .fetch_one(pool)
            .await
            .expect("taille du groupe");
    toutes && dans_le_groupe == lignes.len() as i64
}

/// Le reste dû **dérivé** (`invoice_settlements::amount_due`).
pub async fn reste_du(pool: &MySqlPool, inv: i64) -> Decimal {
    invoice_settlements::amount_due(pool, inv)
        .await
        .expect("reste dû")
}

/// Une ligne de `C(I)` est-elle en période ouverte (règle de la 15-1a2-0) ?
pub async fn en_periode_ouverte(pool: &MySqlPool, company_id: i64, inv: i64) -> bool {
    let lignes: Vec<(i64, NaiveDate)> = lignes_de_piece(pool, inv)
        .await
        .into_iter()
        .map(|l| (l.4, l.5))
        .collect();
    let mut conn = pool.acquire().await.expect("connexion");
    letterings::lines_in_open_period(&mut conn, company_id, &lignes)
        .await
        .expect("règle des périodes")
}

/// Efface, en SQL brut, les marques du groupe `cle` — état hérité (aucune
/// entrée d'audit, comme une donnée réelle antérieure au lettrage). ⛔
/// Assertion de montage : au moins une ligne trouvée.
pub async fn effacer_marques(pool: &MySqlPool, cle: i64) {
    let n = sqlx::query(
        "UPDATE journal_entry_lines SET lettering_key = NULL, lettering_origin = NULL \
         WHERE lettering_key = ?",
    )
    .bind(cle)
    .execute(pool)
    .await
    .expect("effacer les marques")
    .rows_affected();
    assert!(n >= 2, "montage : le groupe {cle} n'avait que {n} ligne(s)");
}

/// La clé portée par l'ancre de la facture (`None` si elle est ouverte).
pub async fn cle_de(pool: &MySqlPool, inv: i64) -> Option<i64> {
    lignes_de_piece(pool, inv).await.first().and_then(|l| l.2)
}

/// Rapproche l'écriture `entry_id` d'une transaction bancaire (montage SQL : le
/// rapprochement réel vit dans `kesh-api`) et rend l'id de la transaction.
/// ⚠️ La facture a été réglée par le GESTE (`settle_invoice`, qui lettre) : une
/// ligne `invoice_settlements` posée en SQL brut ne lettrerait rien.
pub async fn rapprocher(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    entry_id: i64,
    montant: Decimal,
    le: NaiveDate,
) -> i64 {
    let compte_bancaire = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban) \
         VALUES (?, 'Banque lettrage', 'CH9300762011623852957')",
    )
    .bind(seeded.company_id)
    .execute(pool)
    .await
    .expect("compte bancaire")
    .last_insert_id() as i64;
    let import_id = sqlx::query(
        "INSERT INTO bank_imports (company_id, bank_account_id, filename, file_hash, \
         source_format, period_from, period_to, imported_by_user_id) \
         VALUES (?, ?, 'l.xml', REPEAT('b', 64), 'camt053', ?, ?, ?)",
    )
    .bind(seeded.company_id)
    .bind(compte_bancaire)
    .bind(le)
    .bind(le)
    .bind(seeded.admin_user_id)
    .execute(pool)
    .await
    .expect("import")
    .last_insert_id() as i64;
    sqlx::query(
        "INSERT INTO bank_transactions (company_id, import_id, bank_account_id, booking_date, \
         amount, currency, details, matched_entry_id, status) \
         VALUES (?, ?, ?, ?, ?, 'CHF', 'lettrage', ?, 'reconciled')",
    )
    .bind(seeded.company_id)
    .bind(import_id)
    .bind(compte_bancaire)
    .bind(le)
    .bind(montant)
    .bind(entry_id)
    .execute(pool)
    .await
    .expect("transaction rapprochée")
    .last_insert_id() as i64
}

/// Pourquoi une facture de la fixture est une **exception nommée** d'AC5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exception {
    /// (a) avoir hérité crédité sur un autre compte que la créance (d'avant 15-6a).
    AvoirSurUnAutreCompte,
    /// (b) compte de créance non lettrable (AC13).
    CreanceNonLettrable,
}

/// La fixture partagée d'AC5.
pub struct Fixture {
    pub seeded: SeededCompany,
    /// Chaque facture de la fixture, avec son libellé (nommé par le test).
    pub factures: Vec<(&'static str, i64)>,
    /// Les deux exceptions d'AC5 — présentes, pour que le filtre soit exercé.
    pub exceptions: Vec<(i64, Exception)>,
}

/// **La fixture partagée d'AC5** (Story 15-1a2-i, T5) : une société, et une
/// facture par cas — paiement total, partiels, solde, avoir, annulation,
/// rapprochement, `paid_at` hérité sans règlement, créditée et réglée héritée,
/// pièce soldée puis passée sous la borne, et les deux exceptions nommées.
///
/// Calendrier : la pièce passée sous la borne est datée `aujourd'hui − 300` ; le
/// verrou est posé à cette date **en dernier** ; toutes les autres pièces sont
/// datées entre `aujourd'hui − 200` et `aujourd'hui − 100`, en période ouverte.
pub async fn seed_lettering_documents(pool: &MySqlPool) -> Fixture {
    let seeded = societe(pool).await;
    let d = jours_avant(200);
    let apres = jours_avant(150);
    let mut factures = Vec::new();
    let mut exceptions = Vec::new();

    // Paiement total.
    let inv = facture(pool, &seeded, dec!(100.00), d).await;
    regler(pool, &seeded, inv, dec!(100.00), apres).await;
    factures.push(("paiement total", inv));

    // Trois partiels, dont un `internal_account`.
    let inv = facture(pool, &seeded, dec!(90.00), d).await;
    for montant in [dec!(30.00), dec!(30.00), dec!(30.00)] {
        regler(pool, &seeded, inv, montant, apres).await;
    }
    factures.push(("trois partiels", inv));

    // Un partiel seul : due, non lettrée.
    let inv = facture(pool, &seeded, dec!(80.00), d).await;
    regler(pool, &seeded, inv, dec!(20.00), apres).await;
    factures.push(("partiel seul", inv));

    // Solde du reste.
    let inv = facture(pool, &seeded, dec!(70.00), d).await;
    regler(pool, &seeded, inv, dec!(60.00), apres).await;
    solder(
        pool,
        &seeded,
        inv,
        SettlementWriteOffNature::Discount,
        apres,
    )
    .await;
    factures.push(("solde", inv));

    // Avoir total.
    let inv = facture(pool, &seeded, dec!(60.00), d).await;
    crediter(pool, &seeded, inv, apres).await;
    factures.push(("avoir", inv));

    // Règlement complet puis annulé.
    let inv = facture(pool, &seeded, dec!(50.00), d).await;
    let (sid, _) = regler(pool, &seeded, inv, dec!(50.00), apres).await;
    invoice_settlements_write::cancel_settlement(
        pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv,
        sid,
    )
    .await
    .expect("annulation");
    factures.push(("annulation", inv));

    // Encaissement rapproché (montage F4-4 : le geste, puis le lien en SQL).
    let inv = facture(pool, &seeded, dec!(40.00), d).await;
    let (_, entry) = regler(pool, &seeded, inv, dec!(40.00), apres).await;
    rapprocher(pool, &seeded, entry, dec!(40.00), apres).await;
    factures.push(("rapprochement", inv));

    // Héritage : `paid_at` sans aucun règlement (d'avant la 24-2).
    let inv = facture(pool, &seeded, dec!(30.00), d).await;
    sqlx::query("UPDATE invoices SET paid_at = ? WHERE id = ?")
        .bind(apres.and_hms_opt(0, 0, 0).expect("minuit"))
        .bind(inv)
        .execute(pool)
        .await
        .expect("paid_at hérité");
    factures.push(("paid_at hérité", inv));

    // Héritage : créditée ET réglée — « détacher, créditer, rattacher »
    // (`invoice_settlement.rs`), l'avoir lettre pendant le détachement : ses
    // marques sont effacées avant de rattacher.
    let inv = facture(pool, &seeded, dec!(100.00), d).await;
    let (sid, _) = regler(pool, &seeded, inv, dec!(40.00), apres).await;
    let parking = facture(pool, &seeded, dec!(10.00), d).await;
    sqlx::query("UPDATE invoice_settlements SET invoice_id = ? WHERE id = ?")
        .bind(parking)
        .bind(sid)
        .execute(pool)
        .await
        .expect("détacher");
    crediter(pool, &seeded, inv, apres).await;
    let cle = cle_de(pool, inv)
        .await
        .expect("l'avoir a lettré la facture");
    effacer_marques(pool, cle).await;
    sqlx::query("UPDATE invoice_settlements SET invoice_id = ? WHERE id = ?")
        .bind(inv)
        .bind(sid)
        .execute(pool)
        .await
        .expect("rattacher");
    factures.push(("créditée et réglée", inv));
    factures.push(("facture de détachement", parking));

    // Exception (a) : avoir hérité crédité sur un autre compte que la créance.
    let inv = facture(pool, &seeded, dec!(25.00), d).await;
    let avoir = crediter(pool, &seeded, inv, apres).await;
    let cle = cle_de(pool, inv)
        .await
        .expect("l'avoir a lettré la facture");
    effacer_marques(pool, cle).await;
    let deplacees = sqlx::query(
        "UPDATE journal_entry_lines SET account_id = ? WHERE entry_id = ? AND account_id = ?",
    )
    .bind(seeded.accounts["2000"])
    .bind(avoir)
    .bind(seeded.accounts["1100"])
    .execute(pool)
    .await
    .expect("avoir sur un autre compte")
    .rows_affected();
    assert_eq!(deplacees, 1, "montage : la ligne de créance de l'avoir");
    factures.push(("avoir hérité sur un autre compte", inv));
    exceptions.push((inv, Exception::AvoirSurUnAutreCompte));

    // Exception (b) : créance non lettrable. Un second compte de créance
    // (1105), désigné le temps de valider la facture, puis rattaché à un compte
    // bancaire en SQL brut (`refuse_if_ledger_is_claim_account` refuse le lien
    // tant qu'il est désigné) ; réglée ensuite en entier.
    let creance = sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type) \
         VALUES (?, '1105', 'Débiteurs bis', 'Asset')",
    )
    .bind(seeded.company_id)
    .execute(pool)
    .await
    .expect("second compte de créance")
    .last_insert_id() as i64;
    designer_creance(pool, seeded.company_id, creance).await;
    let inv = facture(pool, &seeded, dec!(20.00), d).await;
    designer_creance(pool, seeded.company_id, seeded.accounts["1100"]).await;
    rattacher_a_un_compte_bancaire(pool, seeded.company_id, creance).await;
    regler(pool, &seeded, inv, dec!(20.00), apres).await;
    factures.push(("créance non lettrable", inv));
    exceptions.push((inv, Exception::CreanceNonLettrable));

    // Pièce soldée puis passée sous la borne : hors du critère d'AC5.
    let ancienne = jours_avant(300);
    let inv = facture(pool, &seeded, dec!(15.00), ancienne).await;
    regler(pool, &seeded, inv, dec!(15.00), ancienne).await;
    factures.push(("soldée puis sous la borne", inv));
    companies::lock_books(pool, seeded.admin_user_id, seeded.company_id, ancienne)
        .await
        .expect("verrou de période");

    Fixture {
        seeded,
        factures,
        exceptions,
    }
}

/// Désigne `account_id` comme compte de créance par défaut (montage).
pub async fn designer_creance(pool: &MySqlPool, company_id: i64, account_id: i64) {
    sqlx::query(
        "UPDATE company_invoice_settings SET default_receivable_account_id = ? \
         WHERE company_id = ?",
    )
    .bind(account_id)
    .bind(company_id)
    .execute(pool)
    .await
    .expect("désigner la créance");
}

/// Rattache `account_id` à un compte bancaire, en SQL brut : le compte cesse
/// d'être lettrable (R4).
pub async fn rattacher_a_un_compte_bancaire(pool: &MySqlPool, company_id: i64, account_id: i64) {
    sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, journal_account_id) \
         VALUES (?, 'Banque créance', 'CH5604835012345678009', ?)",
    )
    .bind(company_id)
    .bind(account_id)
    .execute(pool)
    .await
    .expect("rattacher la créance à un compte bancaire");
}

/// Les divergences d'AC5 sur la société, **en nommant la facture** : pour
/// chaque facture validée ou créditée dont une ligne de `C(I)` est en période
/// ouverte, *lettrée `document`* ⇔ *reste dû nul*, hors les `exceptions`.
pub async fn divergences_ac5(
    pool: &MySqlPool,
    company_id: i64,
    exceptions: &[(i64, Exception)],
) -> Vec<String> {
    let factures: Vec<(i64, Option<String>)> = sqlx::query_as(
        "SELECT id, invoice_number FROM invoices WHERE company_id = ? \
         AND status IN ('validated', 'cancelled') ORDER BY id",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .expect("factures");
    let mut fautes = Vec::new();
    for (inv, numero) in factures {
        if exceptions.iter().any(|(e, _)| *e == inv) {
            continue;
        }
        if !en_periode_ouverte(pool, company_id, inv).await {
            continue;
        }
        let lettree = lettree_document(pool, inv).await;
        let soldee = reste_du(pool, inv).await == Decimal::ZERO;
        if lettree != soldee {
            fautes.push(format!(
                "facture {inv} ({numero:?}) : lettrée `document` = {lettree}, reste dû nul = \
                 {soldee}"
            ));
        }
    }
    fautes
}

// ---------------------------------------------------------------------------
// Story 15-1a2-ii — les factures fournisseurs
// ---------------------------------------------------------------------------

/// Ce qu'il faut pour acheter et payer : un fournisseur, une dette `B` dédiée
/// (`2010`, pour ne croiser aucune ligne des factures clientes), et un compte
/// bancaire source dont le compte du grand livre (`1020`) est donc **non
/// lettrable** (R4).
pub struct Achats {
    pub supplier_id: i64,
    /// La dette `B` désignée (`2010`).
    pub payable: i64,
    /// Le compte bancaire source des virements et des lots.
    pub bank_account_id: i64,
    /// Son compte du grand livre (`1020`), non lettrable.
    pub bank_ledger: i64,
}

/// Insère un compte (montage) et rend son id.
pub async fn compte(pool: &MySqlPool, company_id: i64, numero: &str, type_: &str) -> i64 {
    sqlx::query("INSERT INTO accounts (company_id, number, name, account_type) VALUES (?, ?, ?, ?)")
        .bind(company_id)
        .bind(numero)
        .bind(format!("Compte {numero}"))
        .bind(type_)
        .execute(pool)
        .await
        .expect("compte")
        .last_insert_id() as i64
}

/// Désigne `account_id` comme dette fournisseurs par défaut (montage) : les
/// factures fournisseurs créées ensuite la créditent.
pub async fn designer_dette(pool: &MySqlPool, company_id: i64, account_id: i64) {
    sqlx::query(
        "UPDATE company_invoice_settings SET default_payable_account_id = ? WHERE company_id = ?",
    )
    .bind(account_id)
    .bind(company_id)
    .execute(pool)
    .await
    .expect("désigner la dette");
}

/// Prépare les achats de la société seedée (cf. [`Achats`]).
pub async fn achats(pool: &MySqlPool, seeded: &SeededCompany) -> Achats {
    let payable = compte(pool, seeded.company_id, "2010", "Liability").await;
    let recuperable = compte(pool, seeded.company_id, "1170", "Asset").await;
    sqlx::query(
        "UPDATE company_invoice_settings SET default_payable_account_id = ?, \
         default_vat_recoverable_account_id = ? WHERE company_id = ?",
    )
    .bind(payable)
    .bind(recuperable)
    .bind(seeded.company_id)
    .execute(pool)
    .await
    .expect("réglages d'achat");
    let supplier_id: i64 = sqlx::query_scalar(
        "INSERT INTO contacts (company_id, contact_type, name, is_supplier) \
         VALUES (?, 'Entreprise', 'Fournisseur lettrage', TRUE) RETURNING id",
    )
    .bind(seeded.company_id)
    .fetch_one(pool)
    .await
    .expect("fournisseur");
    let bank_ledger = compte(pool, seeded.company_id, "1020", "Asset").await;
    let bank = bank_accounts::create(
        pool,
        NewBankAccount {
            company_id: seeded.company_id,
            bank_name: "Banque des achats".into(),
            iban: "CH9300762011623852957".into(),
            qr_iban: None,
            is_primary: false,
        },
    )
    .await
    .expect("compte bancaire source");
    sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
        .bind(bank_ledger)
        .bind(bank.id)
        .execute(pool)
        .await
        .expect("compte du grand livre de la banque");
    Achats {
        supplier_id,
        payable,
        bank_account_id: bank.id,
        bank_ledger,
    }
}

/// Une facture fournisseur `open` de `ttc` (une ligne à 0 % sur la charge
/// `4000`), avec coordonnées de paiement (pour les lots) et le numéro `numero`.
pub async fn facture_fournisseur(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    achats: &Achats,
    ttc: Decimal,
    date: NaiveDate,
    numero: Option<&str>,
) -> i64 {
    supplier_invoices::create(
        pool,
        NewSupplierInvoice {
            company_id: seeded.company_id,
            contact_id: achats.supplier_id,
            supplier_invoice_number: numero.map(String::from),
            invoice_date: date,
            due_date: None,
            creditor_iban: Some("CH5604835012345678009".into()),
            creditor_qr_iban: None,
            payment_reference: Some("Lettrage".into()),
            expected_payment_amount: None,
            project_id: None,
            lines: vec![NewSupplierInvoiceLine {
                description: "Achat".into(),
                quantity: dec!(1),
                unit_price: ttc,
                vat_rate: dec!(0),
                expense_account_id: seeded.accounts["4000"],
            }],
        },
        seeded.admin_user_id,
    )
    .await
    .expect("facture fournisseur")
    .invoice
    .id
}

/// Paie la facture fournisseur par le mode `choix` et rend l'écriture de
/// règlement.
pub async fn payer_par(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    id: i64,
    choix: SettlementChoice,
    le: NaiveDate,
) -> i64 {
    supplier_invoices::pay(pool, seeded.company_id, id, choix, le, seeded.admin_user_id)
        .await
        .expect("paiement fournisseur")
        .invoice
        .settlement_journal_entry_id
        .expect("écriture de règlement")
}

/// Paie par virement depuis le compte bancaire des [`Achats`].
pub async fn payer(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    achats: &Achats,
    id: i64,
    le: NaiveDate,
) -> i64 {
    payer_par(
        pool,
        seeded,
        id,
        SettlementChoice::BankTransfer {
            bank_account_id: achats.bank_account_id,
        },
        le,
    )
    .await
}

/// Paie les factures par un lot pain.001 **confirmé** (`confirm_batch`).
pub async fn payer_par_lot(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    achats: &Achats,
    ids: Vec<i64>,
    le: NaiveDate,
) {
    let lot = payment_batches::create_batch(
        pool,
        NewPaymentBatch {
            company_id: seeded.company_id,
            bank_account_id: achats.bank_account_id,
            requested_execution_date: le,
            supplier_invoice_ids: ids,
        },
        seeded.admin_user_id,
    )
    .await
    .expect("lot créé");
    assert!(lot.failed.is_empty(), "montage : lot sans refus");
    let lot_id = lot.batch.expect("lot").batch.id;
    payment_batches::confirm_batch(pool, seeded.company_id, lot_id, le, seeded.admin_user_id)
        .await
        .expect("lot confirmé");
}

/// L'écriture d'achat d'une facture fournisseur.
pub async fn achat(pool: &MySqlPool, id: i64) -> i64 {
    sqlx::query_scalar("SELECT purchase_journal_entry_id FROM supplier_invoices WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .expect("écriture d'achat")
}

/// `C(S)`, **recalculé ici sans le code de production** : l'ancre (première
/// ligne au crédit de l'écriture d'achat) puis la ligne sur son compte de
/// l'écriture de règlement **en vigueur** — triées par `id`. Même forme que
/// [`LigneDePiece`].
pub async fn lignes_de_piece_fournisseur(pool: &MySqlPool, id: i64) -> Vec<LigneDePiece> {
    sqlx::query_as(
        "WITH ancre AS ( \
             SELECT jel.id, jel.account_id FROM journal_entry_lines jel \
             JOIN supplier_invoices s ON s.purchase_journal_entry_id = jel.entry_id \
             WHERE s.id = ? AND jel.credit > 0 ORDER BY jel.id LIMIT 1) \
         SELECT jel.id, jel.entry_id, jel.lettering_key, jel.lettering_origin, \
                je.fiscal_year_id, je.entry_date \
         FROM journal_entry_lines jel JOIN journal_entries je ON je.id = jel.entry_id \
         JOIN ancre a ON jel.account_id = a.account_id \
         WHERE jel.id = a.id \
            OR jel.entry_id = (SELECT settlement_journal_entry_id FROM supplier_invoices \
                               WHERE id = ?) \
         ORDER BY jel.id",
    )
    .bind(id)
    .bind(id)
    .fetch_all(pool)
    .await
    .expect("lignes de la pièce fournisseur")
}

/// Le prédicat « lettrée `document` » d'une facture fournisseur, **écrit dans
/// le test** : `C(S)` est exactement un groupe d'origine `document`.
pub async fn fournisseur_lettree_document(pool: &MySqlPool, id: i64) -> bool {
    let lignes = lignes_de_piece_fournisseur(pool, id).await;
    if lignes.len() < 2 {
        return false;
    }
    let Some(cle) = lignes[0].2 else {
        return false;
    };
    let toutes = lignes
        .iter()
        .all(|l| l.2 == Some(cle) && l.3.as_deref() == Some("document"));
    let dans_le_groupe: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM journal_entry_lines WHERE lettering_key = ?")
            .bind(cle)
            .fetch_one(pool)
            .await
            .expect("taille du groupe");
    toutes && dans_le_groupe == lignes.len() as i64
}

/// Écriture manuelle `D debit_account / C credit_account` de `montant` (montage
/// des contre-passations libres) ; rend `(écriture, ligne au débit, ligne au
/// crédit)`.
pub async fn ecriture_manuelle(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    debit_account: i64,
    credit_account: i64,
    montant: Decimal,
    le: NaiveDate,
) -> (i64, i64, i64) {
    use kesh_db::entities::{Journal, NewJournalEntry, NewJournalEntryLine};
    let je = kesh_db::repositories::journal_entries::create(
        pool,
        seeded.fiscal_year_id,
        seeded.admin_user_id,
        NewJournalEntry {
            company_id: seeded.company_id,
            entry_date: le,
            journal: Journal::OD,
            description: "Écriture manuelle lettrage".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: debit_account,
                    debit: montant,
                    credit: Decimal::ZERO,
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: credit_account,
                    debit: Decimal::ZERO,
                    credit: montant,
                    project_id: None,
                },
            ],
        },
    )
    .await
    .expect("écriture manuelle");
    let ligne = |i: usize| je.lines[i].id;
    (je.entry.id, ligne(0), ligne(1))
}
