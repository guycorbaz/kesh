//! Story 33-1a (#926) — le numéro interne d'une facture fournisseur, au niveau du
//! dépôt : attribution à la validation (AC8), conservation (AC9), refus qui ne
//! consomment rien (AC10), séquence par année de début (AC11), concurrence (AC12),
//! non-réémission (AC13), aucun autre écrivain (AC14), libellés (AC15).
//!
//! Décor synthétique : `seed_accounting_company`, dont l'exercice est ramené à
//! l'exercice A (1.1.2026 - 30.6.2026) ; B et les suivants sont posés en SQL.
//!
//! Pré-requis : MariaDB démarré.

#[path = "support/fournisseur_rapproche.rs"]
mod fournisseur_rapproche;

use chrono::NaiveDate;
use fournisseur_rapproche::{Fournisseur, IBAN_CREANCIER};
use kesh_db::entities::{NewSupplierInvoice, NewSupplierInvoiceLine, SettlementChoice};
use kesh_db::errors::DbError;
use kesh_db::repositories::supplier_invoices::{self, SupplierInvoiceWithLines};
use kesh_db::test_fixtures::{
    SeededCompany, attendre_une_requete_en_cours, seed_accounting_company,
};
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn jour(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// Une date dans l'exercice A, une dans l'exercice B.
fn dans_a() -> NaiveDate {
    jour(2026, 3, 1)
}
fn dans_b() -> NaiveDate {
    jour(2026, 8, 1)
}

struct Decor {
    seeded: SeededCompany,
    f: Fournisseur,
    /// L'exercice A, 1.1.2026 - 30.6.2026, « 2026 ».
    a: i64,
}

async fn decor(pool: &MySqlPool) -> Decor {
    let seeded = seed_accounting_company(pool).await.expect("seed");
    let f = fournisseur_rapproche::preparer(pool, &seeded).await;
    sqlx::query(
        "UPDATE fiscal_years SET name = '2026', start_date = '2026-01-01', \
         end_date = '2026-06-30' WHERE id = ?",
    )
    .bind(seeded.fiscal_year_id)
    .execute(pool)
    .await
    .unwrap();
    let a = seeded.fiscal_year_id;
    Decor { seeded, f, a }
}

impl Decor {
    fn company(&self) -> i64 {
        self.seeded.company_id
    }
    fn user(&self) -> i64 {
        self.seeded.admin_user_id
    }

    fn nouvelle(&self, date: NaiveDate, numero: Option<&str>) -> NewSupplierInvoice {
        NewSupplierInvoice {
            company_id: self.company(),
            contact_id: self.f.supplier_id,
            supplier_invoice_number: numero.map(str::to_string),
            invoice_date: date,
            due_date: None,
            creditor_iban: Some(IBAN_CREANCIER.into()),
            creditor_qr_iban: None,
            payment_reference: None,
            expected_payment_amount: None,
            project_id: None,
            lines: vec![NewSupplierInvoiceLine {
                description: "Ligne".into(),
                quantity: dec!(1),
                unit_price: dec!(100),
                vat_rate: dec!(0),
                expense_account_id: self.seeded.accounts["4000"],
            }],
        }
    }

    async fn brouillon(&self, pool: &MySqlPool, date: NaiveDate) -> SupplierInvoiceWithLines {
        supplier_invoices::create_draft(pool, self.nouvelle(date, None), self.user())
            .await
            .expect("brouillon")
    }

    async fn valider(
        &self,
        pool: &MySqlPool,
        d: &SupplierInvoiceWithLines,
    ) -> SupplierInvoiceWithLines {
        supplier_invoices::validate(
            pool,
            self.company(),
            d.invoice.id,
            d.invoice.version,
            self.user(),
        )
        .await
        .expect("validation")
    }

    /// Crée et valide ; rend le numéro attribué.
    async fn emettre(&self, pool: &MySqlPool, date: NaiveDate) -> SupplierInvoiceWithLines {
        let d = self.brouillon(pool, date).await;
        self.valider(pool, &d).await
    }

    async fn numero_emis(&self, pool: &MySqlPool, date: NaiveDate) -> String {
        self.emettre(pool, date)
            .await
            .invoice
            .internal_number
            .expect("une facture validée a un numéro")
    }

    /// La dernière facture émise repasse en brouillon et se supprime.
    async fn supprimer(&self, pool: &MySqlPool, v: &SupplierInvoiceWithLines) {
        let d = supplier_invoices::unvalidate(
            pool,
            self.company(),
            v.invoice.id,
            v.invoice.version,
            self.user(),
        )
        .await
        .expect("dévalidation");
        assert_eq!(d.invoice.status, "draft");
        supplier_invoices::delete_draft(pool, self.company(), d.invoice.id, self.user())
            .await
            .expect("suppression du brouillon");
    }

    async fn format(&self, pool: &MySqlPool, format: &str) {
        sqlx::query(
            "UPDATE company_invoice_settings SET supplier_internal_number_format = ? \
             WHERE company_id = ?",
        )
        .bind(format)
        .bind(self.company())
        .execute(pool)
        .await
        .unwrap();
    }

    async fn exercice(&self, pool: &MySqlPool, nom: &str, debut: &str, fin: &str) -> i64 {
        sqlx::query(
            "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
             VALUES (?, ?, ?, ?, 'Open')",
        )
        .bind(self.company())
        .bind(nom)
        .bind(debut)
        .bind(fin)
        .execute(pool)
        .await
        .unwrap()
        .last_insert_id() as i64
    }

    /// L'exercice B : 1.7.2026 - 30.6.2027, même année de début que A.
    async fn exercice_b(&self, pool: &MySqlPool) -> i64 {
        self.exercice(pool, "2026-2", "2026-07-01", "2027-06-30")
            .await
    }
}

async fn compteur(pool: &MySqlPool, fy: i64) -> i64 {
    sqlx::query_scalar("SELECT supplier_invoice_next_number FROM fiscal_years WHERE id = ?")
        .bind(fy)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn poser_compteur(pool: &MySqlPool, fy: i64, valeur: i64) {
    sqlx::query("UPDATE fiscal_years SET supplier_invoice_next_number = ? WHERE id = ?")
        .bind(valeur)
        .bind(fy)
        .execute(pool)
        .await
        .unwrap();
}

async fn numero_en_base(pool: &MySqlPool, id: i64) -> Option<String> {
    sqlx::query_scalar("SELECT internal_number FROM supplier_invoices WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn libelle_ecriture(pool: &MySqlPool, id: i64) -> String {
    sqlx::query_scalar("SELECT description FROM journal_entries WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

// ---------------------------------------------------------------------------
// AC8 — l'attribution
// ---------------------------------------------------------------------------

/// Un brouillon n'a pas de numéro ; la validation lui donne le premier, le compteur
/// de l'exercice avance, et l'instantané d'audit le porte.
#[sqlx::test(migrations = "./test-schema")]
async fn validation_gives_the_first_number(pool: MySqlPool) {
    let d = decor(&pool).await;
    let b = d.brouillon(&pool, dans_a()).await;
    assert_eq!(b.invoice.internal_number, None);
    assert_eq!(compteur(&pool, d.a).await, 1);

    let v = d.valider(&pool, &b).await;
    assert_eq!(v.invoice.internal_number.as_deref(), Some("FF-2026-001"));
    assert_eq!(compteur(&pool, d.a).await, 2);

    let audit: serde_json::Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'supplier_invoice.validated' \
         AND entity_id = ?",
    )
    .bind(v.invoice.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        audit["supplierInvoice"]["internalNumber"], "FF-2026-001",
        "l'instantané d'audit porte le numéro"
    );

    let suivant = d.numero_emis(&pool, dans_a()).await;
    assert_eq!(suivant, "FF-2026-002");
    assert_eq!(compteur(&pool, d.a).await, 3);
}

/// `{YEAR}` est l'année de début de l'exercice, non celle de la facture (C-e331-3).
#[sqlx::test(migrations = "./test-schema")]
async fn the_year_is_the_start_of_the_fiscal_year(pool: MySqlPool) {
    let d = decor(&pool).await;
    d.exercice_b(&pool).await;
    let numero = d.numero_emis(&pool, jour(2027, 2, 1)).await;
    assert_eq!(
        numero, "FF-2026-001",
        "facture de 2027, exercice commencé en 2026"
    );
}

/// Le format réglé s'applique aux numéros à venir ; ceux déjà attribués restent
/// (C-e331-7).
#[sqlx::test(migrations = "./test-schema")]
async fn a_new_format_applies_to_numbers_to_come(pool: MySqlPool) {
    let d = decor(&pool).await;
    let premiere = d.emettre(&pool, dans_a()).await;
    d.format(&pool, "KF-{YEAR}-{SEQ:04}").await;
    assert_eq!(d.numero_emis(&pool, dans_a()).await, "KF-2026-0002");
    assert_eq!(
        numero_en_base(&pool, premiere.invoice.id).await.as_deref(),
        Some("FF-2026-001")
    );
}

// ---------------------------------------------------------------------------
// AC9 — le numéro survit
// ---------------------------------------------------------------------------

/// Validée, puis brouillon : le numéro reste. Revalidée : le même, sans toucher au
/// compteur, même si le format a changé entre-temps.
#[sqlx::test(migrations = "./test-schema")]
async fn unvalidated_keeps_and_revalidated_takes_the_same_number(pool: MySqlPool) {
    let d = decor(&pool).await;
    let v = d.emettre(&pool, dans_a()).await;
    let autre = d.numero_emis(&pool, dans_a()).await;
    assert_eq!(autre, "FF-2026-002");
    let compteur_avant = compteur(&pool, d.a).await;

    let b = supplier_invoices::unvalidate(
        &pool,
        d.company(),
        v.invoice.id,
        v.invoice.version,
        d.user(),
    )
    .await
    .expect("dévalidation");
    assert_eq!(b.invoice.status, "draft");
    assert_eq!(b.invoice.internal_number.as_deref(), Some("FF-2026-001"));

    d.format(&pool, "KF-{YEAR}-{SEQ:04}").await;
    let r = d.valider(&pool, &b).await;
    assert_eq!(r.invoice.internal_number.as_deref(), Some("FF-2026-001"));
    assert_eq!(
        compteur(&pool, d.a).await,
        compteur_avant,
        "le compteur ne bouge pas"
    );
    let achat = r.invoice.purchase_journal_entry_id.unwrap();
    assert!(
        libelle_ecriture(&pool, achat).await.contains("FF-2026-001"),
        "la nouvelle écriture d'achat porte le numéro"
    );
}

/// Annulée, puis brouillon : le numéro reste aussi.
#[sqlx::test(migrations = "./test-schema")]
async fn cancelled_then_draft_keeps_the_number(pool: MySqlPool) {
    let d = decor(&pool).await;
    // La contre-passation se date du jour : A doit le couvrir, quel qu'il soit.
    sqlx::query("UPDATE fiscal_years SET end_date = '2099-12-31' WHERE id = ?")
        .bind(d.a)
        .execute(&pool)
        .await
        .unwrap();
    let v = d.emettre(&pool, dans_a()).await;
    let annulee = supplier_invoices::cancel(&pool, d.company(), v.invoice.id, d.user())
        .await
        .expect("annulation");
    assert_eq!(
        annulee.invoice.internal_number.as_deref(),
        Some("FF-2026-001")
    );
    let b = supplier_invoices::unvalidate(
        &pool,
        d.company(),
        annulee.invoice.id,
        annulee.invoice.version,
        d.user(),
    )
    .await
    .expect("dévalidation d'une annulée");
    assert_eq!(b.invoice.internal_number.as_deref(), Some("FF-2026-001"));
}

// ---------------------------------------------------------------------------
// AC10 — un refus ne consomme rien
// ---------------------------------------------------------------------------

async fn assert_refus_sans_numero(
    pool: &MySqlPool,
    d: &Decor,
    b: &SupplierInvoiceWithLines,
    version: i32,
    motif: &str,
) {
    let err = supplier_invoices::validate(pool, d.company(), b.invoice.id, version, d.user())
        .await
        .expect_err(motif);
    assert!(!matches!(err, DbError::Invariant(_)), "{motif} : {err:?}");
    assert_eq!(numero_en_base(pool, b.invoice.id).await, None, "{motif}");
    assert_eq!(compteur(pool, d.a).await, 1, "{motif} : compteur inchangé");
}

#[sqlx::test(migrations = "./test-schema")]
async fn a_refused_validation_consumes_nothing(pool: MySqlPool) {
    let d = decor(&pool).await;

    // Hors de tout exercice.
    let hors = d.brouillon(&pool, jour(2031, 1, 1)).await;
    assert_refus_sans_numero(&pool, &d, &hors, hors.invoice.version, "hors exercice").await;

    let b = d.brouillon(&pool, dans_a()).await;
    // Conflit de version.
    assert_refus_sans_numero(&pool, &d, &b, b.invoice.version + 1, "version").await;

    // Réglages incomplets.
    sqlx::query(
        "UPDATE company_invoice_settings SET default_payable_account_id = NULL WHERE company_id = ?",
    )
    .bind(d.company())
    .execute(&pool)
    .await
    .unwrap();
    assert_refus_sans_numero(&pool, &d, &b, b.invoice.version, "réglages").await;
    sqlx::query(
        "UPDATE company_invoice_settings SET default_payable_account_id = ? WHERE company_id = ?",
    )
    .bind(d.seeded.accounts["2000"])
    .bind(d.company())
    .execute(&pool)
    .await
    .unwrap();

    // Compte de charge non imputable, puis archivé.
    for (colonne, motif) in [("postable", "non imputable"), ("active", "archivé")] {
        sqlx::query(&format!(
            "UPDATE accounts SET {colonne} = FALSE WHERE id = ?"
        ))
        .bind(d.seeded.accounts["4000"])
        .execute(&pool)
        .await
        .unwrap();
        assert_refus_sans_numero(&pool, &d, &b, b.invoice.version, motif).await;
        sqlx::query(&format!(
            "UPDATE accounts SET {colonne} = TRUE WHERE id = ?"
        ))
        .bind(d.seeded.accounts["4000"])
        .execute(&pool)
        .await
        .unwrap();
    }

    // Exercice clos.
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(d.a)
        .execute(&pool)
        .await
        .unwrap();
    assert_refus_sans_numero(&pool, &d, &b, b.invoice.version, "exercice clos").await;
    sqlx::query("UPDATE fiscal_years SET status = 'Open' WHERE id = ?")
        .bind(d.a)
        .execute(&pool)
        .await
        .unwrap();

    // Le refus a laissé la voie libre : le premier numéro reste à prendre.
    assert_eq!(
        d.valider(&pool, &b)
            .await
            .invoice
            .internal_number
            .as_deref(),
        Some("FF-2026-001")
    );
}

/// Un échec **après** l'attribution (`create_in_tx`, ici une période verrouillée) :
/// le rollback emporte numéro et compteur.
#[sqlx::test(migrations = "./test-schema")]
async fn a_failure_after_the_attribution_rolls_it_back(pool: MySqlPool) {
    let d = decor(&pool).await;
    let b = d.brouillon(&pool, dans_a()).await;
    sqlx::query("UPDATE companies SET books_locked_through = '2026-12-31' WHERE id = ?")
        .bind(d.company())
        .execute(&pool)
        .await
        .unwrap();
    assert_refus_sans_numero(&pool, &d, &b, b.invoice.version, "période verrouillée").await;
}

// ---------------------------------------------------------------------------
// AC11 — la séquence appartient à l'année de début
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn each_start_year_has_its_own_sequence(pool: MySqlPool) {
    let d = decor(&pool).await;
    d.exercice(&pool, "2027", "2027-01-01", "2027-12-31").await;
    assert_eq!(d.numero_emis(&pool, dans_a()).await, "FF-2026-001");
    assert_eq!(d.numero_emis(&pool, jour(2027, 3, 1)).await, "FF-2027-001");
    assert_eq!(d.numero_emis(&pool, dans_a()).await, "FF-2026-002");
}

// ---------------------------------------------------------------------------
// AC12 — concurrence
// ---------------------------------------------------------------------------

/// Valide en rebouclant sur un interblocage (au plus dix fois) : le plafond de
/// trois de `retry_on_deadlock` n'est pas l'objet de ce banc.
async fn valider_avec_reprises(
    pool: MySqlPool,
    company: i64,
    id: i64,
    version: i32,
    user: i64,
) -> String {
    let mut reste = 10;
    loop {
        match supplier_invoices::validate(&pool, company, id, version, user).await {
            Ok(v) => return v.invoice.internal_number.expect("numéro"),
            Err(e) if reste > 0 && kesh_db::retry::is_deadlock_error(&e) => reste -= 1,
            Err(e) => panic!("validation : {e:?}"),
        }
    }
}

async fn valider_ensemble(
    pool: &MySqlPool,
    d: &Decor,
    brouillons: Vec<SupplierInvoiceWithLines>,
) -> Vec<String> {
    let mut taches = Vec::new();
    for b in brouillons {
        taches.push(tokio::spawn(valider_avec_reprises(
            pool.clone(),
            d.company(),
            b.invoice.id,
            b.invoice.version,
            d.user(),
        )));
    }
    let mut numeros = Vec::new();
    for t in taches {
        numeros.push(t.await.unwrap());
    }
    numeros.sort();
    numeros
}

/// Quatre validations simultanées du même exercice : numéros tous distincts, suite
/// sans trou, compteur à 5.
#[sqlx::test(migrations = "./test-schema")]
async fn concurrent_validations_give_distinct_numbers(pool: MySqlPool) {
    let d = decor(&pool).await;
    let mut brouillons = Vec::new();
    for _ in 0..4 {
        brouillons.push(d.brouillon(&pool, dans_a()).await);
    }
    let numeros = valider_ensemble(&pool, &d, brouillons).await;
    assert_eq!(
        numeros,
        ["FF-2026-001", "FF-2026-002", "FF-2026-003", "FF-2026-004"]
    );
    assert_eq!(compteur(&pool, d.a).await, 5);
}

/// De même sur deux exercices de la même année : numéros distincts, en suite.
#[sqlx::test(migrations = "./test-schema")]
async fn concurrent_validations_across_two_fiscal_years_of_a_year(pool: MySqlPool) {
    let d = decor(&pool).await;
    d.exercice_b(&pool).await;
    let mut brouillons = Vec::new();
    for date in [dans_a(), dans_b(), dans_a(), dans_b()] {
        brouillons.push(d.brouillon(&pool, date).await);
    }
    let numeros = valider_ensemble(&pool, &d, brouillons).await;
    assert_eq!(
        numeros,
        ["FF-2026-001", "FF-2026-002", "FF-2026-003", "FF-2026-004"]
    );
}

/// Une validation en cours et une création de brouillon : le contrôle de collision
/// pose un verrou d'intervalle sur l'`UNIQUE`, où tombe le `NULL` du brouillon
/// (première validation de la société) ; la création **attend** puis réussit, sans
/// interblocage, le validateur n'ayant besoin d'aucun verrou du brouillonneur.
#[sqlx::test(migrations = "./test-schema")]
async fn a_draft_creation_waits_for_a_validation_without_deadlock(pool: MySqlPool) {
    let d = decor(&pool).await;
    let b = d.brouillon(&pool, dans_a()).await;
    let mut validation = pool.begin().await.unwrap();
    supplier_invoices::validate_in_tx(
        &mut validation,
        d.company(),
        b.invoice.id,
        b.invoice.version,
        d.user(),
    )
    .await
    .expect("validation en cours");

    let (pool2, nouvelle, user) = (pool.clone(), d.nouvelle(dans_a(), None), d.user());
    let creation =
        tokio::spawn(async move { supplier_invoices::create_draft(&pool2, nouvelle, user).await });
    let vue = attendre_une_requete_en_cours(&pool, &["INSERT INTO supplier_invoices"], || {
        creation.is_finished()
    })
    .await;
    validation.commit().await.unwrap();
    let cree = creation.await.unwrap().expect("la création réussit");
    assert!(
        vue,
        "montage : la création devait attendre le verrou d'intervalle"
    );
    assert_eq!(cree.invoice.internal_number, None);
    assert_eq!(
        numero_en_base(&pool, b.invoice.id).await.as_deref(),
        Some("FF-2026-001")
    );
}

// ---------------------------------------------------------------------------
// AC13 — un numéro ne se réémet pas
// ---------------------------------------------------------------------------

/// (a) Brouillon numéroté supprimé, l'autre exercice de l'année valide.
#[sqlx::test(migrations = "./test-schema")]
async fn a_deleted_number_is_not_taken_by_the_other_fiscal_year(pool: MySqlPool) {
    let d = decor(&pool).await;
    let b = d.exercice_b(&pool).await;
    assert_eq!(d.numero_emis(&pool, dans_a()).await, "FF-2026-001");
    assert_eq!(d.numero_emis(&pool, dans_a()).await, "FF-2026-002");
    let troisieme = d.emettre(&pool, dans_a()).await;
    assert_eq!(
        troisieme.invoice.internal_number.as_deref(),
        Some("FF-2026-003")
    );
    d.supprimer(&pool, &troisieme).await;

    assert_eq!(d.numero_emis(&pool, dans_b()).await, "FF-2026-004");
    assert_eq!(compteur(&pool, d.a).await, 4);
    assert_eq!(compteur(&pool, b).await, 5);
    assert_eq!(d.numero_emis(&pool, dans_a()).await, "FF-2026-005");
}

/// (b) Base migrée : chaque exercice part du plus haut compteur de l'année.
#[sqlx::test(migrations = "./test-schema")]
async fn after_migration_the_highest_counter_of_the_year_rules(pool: MySqlPool) {
    let d = decor(&pool).await;
    let b = d.exercice_b(&pool).await;
    poser_compteur(&pool, d.a, 6).await;
    poser_compteur(&pool, b, 10).await;
    assert_eq!(d.numero_emis(&pool, dans_a()).await, "FF-2026-010");
    assert_eq!(compteur(&pool, d.a).await, 11);
    assert_eq!(d.numero_emis(&pool, dans_b()).await, "FF-2026-011");
}

/// (c) Exercice créé après coup : il part de son défaut, mais sa première facture
/// suit l'année.
#[sqlx::test(migrations = "./test-schema")]
async fn a_fiscal_year_created_later_follows_the_year(pool: MySqlPool) {
    let d = decor(&pool).await;
    for _ in 0..3 {
        d.numero_emis(&pool, dans_a()).await;
    }
    assert_eq!(compteur(&pool, d.a).await, 4);
    let b = d.exercice_b(&pool).await;
    assert_eq!(compteur(&pool, b).await, 1);
    assert_eq!(d.numero_emis(&pool, dans_b()).await, "FF-2026-004");
}

/// (d) Même chose, A étant clôturé : son compteur compte.
#[sqlx::test(migrations = "./test-schema")]
async fn a_closed_fiscal_year_counts(pool: MySqlPool) {
    let d = decor(&pool).await;
    for _ in 0..3 {
        d.numero_emis(&pool, dans_a()).await;
    }
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(d.a)
        .execute(&pool)
        .await
        .unwrap();
    d.exercice_b(&pool).await;
    assert_eq!(d.numero_emis(&pool, dans_b()).await, "FF-2026-004");
}

/// (e) Format changé puis repris, exercice renommé à accolade : la `-005` n'est
/// jamais reprise, et le nom ne gêne pas (M24).
#[sqlx::test(migrations = "./test-schema")]
async fn format_change_and_renaming_bring_back_nothing(pool: MySqlPool) {
    let d = decor(&pool).await;
    let b = d.exercice_b(&pool).await;
    for attendu in ["FF-2026-001", "FF-2026-002", "FF-2026-003"] {
        assert_eq!(d.numero_emis(&pool, dans_a()).await, attendu);
    }
    assert_eq!(compteur(&pool, d.a).await, 4);
    assert_eq!(d.numero_emis(&pool, dans_b()).await, "FF-2026-004");
    let cinquieme = d.emettre(&pool, dans_b()).await;
    assert_eq!(
        cinquieme.invoice.internal_number.as_deref(),
        Some("FF-2026-005")
    );
    assert_eq!(compteur(&pool, b).await, 6);
    d.supprimer(&pool, &cinquieme).await;

    d.format(&pool, "KF-{YEAR}-{SEQ:03}").await;
    assert_eq!(d.numero_emis(&pool, dans_a()).await, "KF-2026-006");
    assert_eq!(compteur(&pool, d.a).await, 7);

    sqlx::query("UPDATE fiscal_years SET name = '{2026}' WHERE id = ?")
        .bind(b)
        .execute(&pool)
        .await
        .unwrap();
    d.format(&pool, "FF-{YEAR}-{SEQ:03}").await;
    assert_eq!(d.numero_emis(&pool, dans_b()).await, "FF-2026-007");
}

/// (f) Le filet, vers le haut : un numéro posé en SQL au-dessus du compteur.
#[sqlx::test(migrations = "./test-schema")]
async fn a_number_posed_outside_the_application_is_skipped_upward(pool: MySqlPool) {
    let d = decor(&pool).await;
    poser_compteur(&pool, d.a, 5).await;
    let pose = d.brouillon(&pool, dans_a()).await;
    sqlx::query("UPDATE supplier_invoices SET internal_number = 'FF-2026-005' WHERE id = ?")
        .bind(pose.invoice.id)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(d.numero_emis(&pool, dans_a()).await, "FF-2026-006");
    assert_eq!(compteur(&pool, d.a).await, 7);
}

/// (f, suite) Le saut est borné : au-delà de 100 000 essais, un `Invariant`, et rien
/// n'est consommé.
#[sqlx::test(migrations = "./test-schema")]
async fn the_skip_is_bounded(pool: MySqlPool) {
    let d = decor(&pool).await;
    sqlx::query(
        "INSERT INTO supplier_invoices (company_id, contact_id, internal_number, status, \
         invoice_date, total_amount) \
         SELECT ?, ?, CONCAT('FF-2026-', LPAD(seq, GREATEST(3, CHAR_LENGTH(seq)), '0')), 'draft', \
         '2026-03-01', 0 FROM seq_1_to_100000",
    )
    .bind(d.company())
    .bind(d.f.supplier_id)
    .execute(&pool)
    .await
    .unwrap();
    let b = d.brouillon(&pool, dans_a()).await;
    let err = supplier_invoices::validate(
        &pool,
        d.company(),
        b.invoice.id,
        b.invoice.version,
        d.user(),
    )
    .await
    .expect_err("saut borné");
    assert!(matches!(err, DbError::Invariant(_)), "{err:?}");
    assert_eq!(numero_en_base(&pool, b.invoice.id).await, None);
    assert_eq!(compteur(&pool, d.a).await, 1);
}

/// (g) La lecture verrouillante, déterministe (M11, M22) : l'instantané de la
/// validation est pris (`fetch_lines`) avant son attente sur les réglages ; une
/// session écrit alors le compteur, ce que laissent des numéros émis puis supprimés,
/// et commite. Sur l'instantané, on lirait 2 et `-002`, libre, serait réémis.
async fn lecture_verrouillante(pool: MySqlPool, sur_b: bool) {
    let d = decor(&pool).await;
    let b = d.exercice_b(&pool).await;
    assert_eq!(d.numero_emis(&pool, dans_a()).await, "FF-2026-001");
    assert_eq!(compteur(&pool, d.a).await, 2);
    assert_eq!(compteur(&pool, b).await, 1);
    let brouillon = d.brouillon(&pool, dans_a()).await;

    let mut session = pool.begin().await.unwrap();
    sqlx::query("SELECT company_id FROM company_invoice_settings WHERE company_id = ? FOR UPDATE")
        .bind(d.company())
        .fetch_one(&mut *session)
        .await
        .unwrap();

    let (pool2, c, id, v, u) = (
        pool.clone(),
        d.company(),
        brouillon.invoice.id,
        brouillon.invoice.version,
        d.user(),
    );
    let validation =
        tokio::spawn(async move { supplier_invoices::validate(&pool2, c, id, v, u).await });
    let vue = attendre_une_requete_en_cours(
        &pool,
        &["INSERT IGNORE", "company_invoice_settings"],
        || validation.is_finished(),
    )
    .await;
    assert!(vue, "montage : la validation devait attendre les réglages");

    sqlx::query("UPDATE fiscal_years SET supplier_invoice_next_number = 5 WHERE id = ?")
        .bind(if sur_b { b } else { d.a })
        .execute(&mut *session)
        .await
        .unwrap();
    session.commit().await.unwrap();

    let validee = validation.await.unwrap().expect("validation");
    assert_eq!(
        validee.invoice.internal_number.as_deref(),
        Some("FF-2026-005")
    );
    assert_eq!(compteur(&pool, d.a).await, 6);
}

#[sqlx::test(migrations = "./test-schema")]
async fn the_counter_of_the_fiscal_year_is_read_under_lock(pool: MySqlPool) {
    lecture_verrouillante(pool, false).await;
}

#[sqlx::test(migrations = "./test-schema")]
async fn the_counter_of_the_other_fiscal_year_is_read_under_lock(pool: MySqlPool) {
    lecture_verrouillante(pool, true).await;
}

// ---------------------------------------------------------------------------
// AC14 — personne d'autre n'écrit le numéro
// ---------------------------------------------------------------------------

/// `update_draft` préserve le numéro d'un brouillon numéroté.
#[sqlx::test(migrations = "./test-schema")]
async fn update_draft_preserves_the_number(pool: MySqlPool) {
    let d = decor(&pool).await;
    let v = d.emettre(&pool, dans_a()).await;
    let b = supplier_invoices::unvalidate(
        &pool,
        d.company(),
        v.invoice.id,
        v.invoice.version,
        d.user(),
    )
    .await
    .unwrap();
    let modifie = supplier_invoices::update_draft(
        &pool,
        d.company(),
        b.invoice.id,
        b.invoice.version,
        d.nouvelle(dans_a(), Some("ABC-9")),
        d.user(),
    )
    .await
    .expect("modification");
    assert_eq!(
        modifie.invoice.internal_number.as_deref(),
        Some("FF-2026-001")
    );
    assert_eq!(
        modifie.invoice.supplier_invoice_number.as_deref(),
        Some("ABC-9")
    );
}

/// La suppression d'un brouillon numéroté réussit, le numéro figure à l'instantané
/// d'audit, le compteur ne redescend pas : la suivante prend le numéro d'après.
#[sqlx::test(migrations = "./test-schema")]
async fn deleting_a_numbered_draft_loses_the_number(pool: MySqlPool) {
    let d = decor(&pool).await;
    let v = d.emettre(&pool, dans_a()).await;
    d.supprimer(&pool, &v).await;
    assert_eq!(
        compteur(&pool, d.a).await,
        2,
        "le compteur ne redescend pas"
    );
    let audit: serde_json::Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'supplier_invoice.deleted'\
         AND entity_id = ?",
    )
    .bind(v.invoice.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(
        audit.to_string().contains("FF-2026-001"),
        "l'audit garde le numéro : {audit}"
    );
    assert_eq!(d.numero_emis(&pool, dans_a()).await, "FF-2026-002");
}

// ---------------------------------------------------------------------------
// AC15 — les libellés
// ---------------------------------------------------------------------------

/// L'écriture d'achat et le règlement portent le numéro interne, puis celui du
/// fournisseur entre parenthèses ; sans numéro du fournisseur, l'interne seul.
#[sqlx::test(migrations = "./test-schema")]
async fn purchase_and_settlement_labels_carry_the_internal_number(pool: MySqlPool) {
    let d = decor(&pool).await;
    let nom = "Fournisseur rapproché SA";
    let compte = SettlementChoice::InternalAccount {
        account_id: d.seeded.accounts["1000"],
    };

    for (numero, attendu) in [
        (Some("ABC-123"), "FF-2026-001 (ABC-123)"),
        (None, "FF-2026-002"),
        (Some("   "), "FF-2026-003"),
    ] {
        let b = supplier_invoices::create_draft(&pool, d.nouvelle(dans_a(), numero), d.user())
            .await
            .unwrap();
        let v = d.valider(&pool, &b).await;
        assert_eq!(
            libelle_ecriture(&pool, v.invoice.purchase_journal_entry_id.unwrap()).await,
            format!("Facture fournisseur {attendu} - {nom}")
        );
        let payee =
            supplier_invoices::pay(&pool, d.company(), v.invoice.id, compte, dans_a(), d.user())
                .await
                .unwrap();
        assert_eq!(
            libelle_ecriture(&pool, payee.invoice.settlement_journal_entry_id.unwrap()).await,
            format!("Règlement fournisseur {attendu} - {nom}")
        );
    }
}

/// Une facture sans numéro interne (posée hors de l'application) : repli sur le
/// numéro du fournisseur, puis sur `#id`, les deux sites alignés.
#[sqlx::test(migrations = "./test-schema")]
async fn settlement_label_falls_back_without_internal_number(pool: MySqlPool) {
    let d = decor(&pool).await;
    let nom = "Fournisseur rapproché SA";
    let compte = SettlementChoice::InternalAccount {
        account_id: d.seeded.accounts["1000"],
    };
    for (numero, attendu) in [(Some("ABC-123"), Some("ABC-123")), (None, None)] {
        let b = supplier_invoices::create_draft(&pool, d.nouvelle(dans_a(), numero), d.user())
            .await
            .unwrap();
        let v = d.valider(&pool, &b).await;
        sqlx::query("UPDATE supplier_invoices SET internal_number = NULL WHERE id = ?")
            .bind(v.invoice.id)
            .execute(&pool)
            .await
            .unwrap();
        let payee =
            supplier_invoices::pay(&pool, d.company(), v.invoice.id, compte, dans_a(), d.user())
                .await
                .unwrap();
        let etiquette = attendu
            .map(str::to_string)
            .unwrap_or_else(|| format!("#{}", v.invoice.id));
        assert_eq!(
            libelle_ecriture(&pool, payee.invoice.settlement_journal_entry_id.unwrap()).await,
            format!("Règlement fournisseur {etiquette} - {nom}")
        );
    }
}
