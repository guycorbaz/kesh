//! Story 49-1a (#1242) — le type d'une ligne de facture client et d'avoir.
//!
//! - T2 `constraints_by_type` : les contraintes de la migration
//!   `20261118000002_invoice_line_types`, sur **les deux tables** ; chaque refus
//!   nomme sa contrainte, pour que le retrait de l'une ne soit pas masqué par une
//!   autre.
//! - T4 `draft_roundtrip_and_no_op` : création et remplacement d'un brouillon
//!   portant un commentaire, positions, total, no-op, audit, refus d'une ligne
//!   non neutre (D10).
//!
//! Pré-requis : MariaDB démarré (`sqlx::test` crée une base éphémère par test).

use chrono::NaiveDate;
use kesh_db::entities::contact::{ContactType, NewContact};
use kesh_db::entities::{InvoiceUpdate, LineType, NewCreditNote, NewInvoice, NewInvoiceLine};
use kesh_db::errors::DbError;
use kesh_db::repositories::{contacts, credit_notes, invoices};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

async fn make_contact(pool: &MySqlPool, seeded: &SeededCompany) -> i64 {
    contacts::create(
        pool,
        seeded.admin_user_id,
        NewContact {
            company_id: seeded.company_id,
            contact_type: ContactType::Entreprise,
            name: "Client Types".into(),
            first_name: None,
            last_name: None,
            is_client: true,
            is_supplier: false,
            address: Some("Rue 1\n1000 Lausanne".into()),
            address_street: None,
            address_building: None,
            address_postal_code: None,
            address_city: None,
            address_country: None,
            email: None,
            phone: None,
            ide_number: None,
            client_number: None,
            default_payment_terms: Some("30".into()),
            default_payment_terms_days: None,
            language: None,
            salutation: kesh_db::entities::contact::Salutation::Neutre,
        },
    )
    .await
    .expect("create contact")
    .id
}

fn priced(desc: &str, qty: Decimal, price: Decimal, rate: Decimal) -> NewInvoiceLine {
    NewInvoiceLine {
        description: desc.into(),
        quantity: qty,
        unit_price: price,
        vat_rate: rate,
        revenue_account_id: None,
        line_type: LineType::Priced,
    }
}

fn unpriced(desc: &str, line_type: LineType) -> NewInvoiceLine {
    NewInvoiceLine {
        description: desc.into(),
        quantity: Decimal::ZERO,
        unit_price: Decimal::ZERO,
        vat_rate: Decimal::ZERO,
        revenue_account_id: None,
        line_type,
    }
}

fn new_invoice(seeded: &SeededCompany, contact_id: i64, lines: Vec<NewInvoiceLine>) -> NewInvoice {
    NewInvoice {
        company_id: seeded.company_id,
        contact_id,
        date: d(2026, 6, 15),
        due_date: None,
        payment_terms: None,
        project_id: None,
        lines,
    }
}

/// Une ligne brute, insérée en SQL pour éprouver les contraintes elles-mêmes.
struct Raw<'a> {
    line_type: Option<&'a str>,
    description: &'a str,
    quantity: Decimal,
    unit_price: Decimal,
    vat_rate: Decimal,
    line_total: Decimal,
    account: Option<i64>,
}

impl Raw<'_> {
    fn neutral(line_type: Option<&'static str>) -> Raw<'static> {
        Raw {
            line_type,
            description: "Note",
            quantity: Decimal::ZERO,
            unit_price: Decimal::ZERO,
            vat_rate: Decimal::ZERO,
            line_total: Decimal::ZERO,
            account: None,
        }
    }
    fn priced(quantity: Decimal) -> Raw<'static> {
        Raw {
            line_type: Some("priced"),
            description: "Ligne",
            quantity,
            unit_price: dec!(10),
            vat_rate: dec!(8.10),
            line_total: quantity * dec!(10),
            account: None,
        }
    }
}

/// Rend une ligne neutre non neutre (le second argument : un compte valide).
type Muter = fn(&mut Raw<'_>, i64);

/// Les deux tables de lignes et leur colonne parente.
#[derive(Clone, Copy)]
enum Table {
    Invoice,
    CreditNote,
}

impl Table {
    fn name(self) -> &'static str {
        match self {
            Table::Invoice => "invoice_lines",
            Table::CreditNote => "credit_note_lines",
        }
    }
    fn parent(self) -> &'static str {
        match self {
            Table::Invoice => "invoice_id",
            Table::CreditNote => "credit_note_id",
        }
    }
}

/// Insère `raw` à la position `pos`. Sans `line_type` (forme 0.19.0, celle de
/// `invoices.rs` et de `credit_notes.rs` avant la story), la colonne est omise.
async fn insert_raw(
    pool: &MySqlPool,
    t: Table,
    parent_id: i64,
    pos: i32,
    raw: &Raw<'_>,
) -> Result<(), sqlx::Error> {
    let (cols, marks) = match raw.line_type {
        Some(_) => (", line_type", ", ?"),
        None => ("", ""),
    };
    let sql = format!(
        "INSERT INTO {} ({}, position, description, quantity, unit_price, vat_rate, \
         line_total, revenue_account_id{cols}) VALUES (?, ?, ?, ?, ?, ?, ?, ?{marks})",
        t.name(),
        t.parent()
    );
    let mut q = sqlx::query(&sql)
        .bind(parent_id)
        .bind(pos)
        .bind(raw.description)
        .bind(raw.quantity)
        .bind(raw.unit_price)
        .bind(raw.vat_rate)
        .bind(raw.line_total)
        .bind(raw.account);
    if let Some(lt) = raw.line_type {
        q = q.bind(lt);
    }
    q.execute(pool).await.map(|_| ())
}

/// L'erreur doit être le refus de la contrainte `chk_<table>_<suffix>`.
fn assert_refused_by(r: Result<(), sqlx::Error>, t: Table, suffix: &str) {
    let constraint = format!("chk_{}_{suffix}", t.name());
    let err = r.expect_err(&format!(
        "{constraint} : l'insertion aurait dû être refusée"
    ));
    let msg = err.to_string();
    assert!(
        msg.contains(&constraint),
        "refus attendu par {constraint}, reçu : {msg}"
    );
}

/// T2 (AC1, D4) — les contraintes par type, sur les deux tables.
#[sqlx::test(migrations = "./test-schema")]
async fn constraints_by_type(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, &seeded).await;

    // Une facture validée (parent des lignes de facture) et son avoir (parent
    // des lignes d'avoir).
    let (inv, _) = invoices::create(
        &pool,
        seeded.admin_user_id,
        new_invoice(
            &seeded,
            contact,
            vec![priced("A", dec!(1), dec!(100), dec!(8.10))],
        ),
    )
    .await
    .unwrap();
    invoices::validate_invoice(&pool, seeded.company_id, inv.id, seeded.admin_user_id)
        .await
        .unwrap();
    let issued = credit_notes::create_credit_note(
        &pool,
        NewCreditNote {
            company_id: seeded.company_id,
            invoice_id: inv.id,
            date: d(2026, 7, 1),
        },
        seeded.admin_user_id,
    )
    .await
    .unwrap();
    let account = seeded.accounts["3000"];

    for (t, parent) in [
        (Table::Invoice, inv.id),
        (Table::CreditNote, issued.credit_note.id),
    ] {
        let mut pos = 100;
        let mut next = || {
            pos += 1;
            pos
        };

        // `priced` : la règle d'avant, quantité > 0.
        assert_refused_by(
            insert_raw(&pool, t, parent, next(), &Raw::priced(dec!(0))).await,
            t,
            "priced_quantity",
        );
        insert_raw(&pool, t, parent, next(), &Raw::priced(dec!(2)))
            .await
            .expect("priced, quantité 2");

        // `comment` : neutre admis, tout montant ou compte refusé.
        insert_raw(&pool, t, parent, next(), &Raw::neutral(Some("comment")))
            .await
            .expect("commentaire neutre");
        let non_neutres: [(&str, Muter); 5] = [
            ("quantité", |r, _| r.quantity = dec!(1)),
            ("prix", |r, _| r.unit_price = dec!(1)),
            ("taux", |r, _| r.vat_rate = dec!(7.7)),
            ("total", |r, _| r.line_total = dec!(1)),
            ("compte", |r, a| r.account = Some(a)),
        ];
        for (nom, muter) in non_neutres {
            let mut r = Raw::neutral(Some("comment"));
            muter(&mut r, account);
            let res = insert_raw(&pool, t, parent, next(), &r).await;
            assert!(res.is_err(), "commentaire avec {nom} admis");
            assert_refused_by(res, t, "unpriced_neutral");
        }
        // La description reste obligatoire pour un commentaire.
        let mut vide = Raw::neutral(Some("comment"));
        vide.description = "   ";
        assert_refused_by(
            insert_raw(&pool, t, parent, next(), &vide).await,
            t,
            "description_not_empty",
        );

        // `heading` : neutre admis, montant refusé.
        insert_raw(&pool, t, parent, next(), &Raw::neutral(Some("heading")))
            .await
            .expect("titre neutre");
        let mut titre = Raw::neutral(Some("heading"));
        titre.quantity = dec!(1);
        assert_refused_by(
            insert_raw(&pool, t, parent, next(), &titre).await,
            t,
            "unpriced_neutral",
        );

        // Un type inconnu, neutre — seul `chk_*_line_type` peut le refuser.
        assert_refused_by(
            insert_raw(&pool, t, parent, next(), &Raw::neutral(Some("subtotal"))).await,
            t,
            "line_type",
        );

        // Un INSERT de forme 0.19.0, sans `line_type`, passe et donne `priced`.
        let p = next();
        insert_raw(
            &pool,
            t,
            parent,
            p,
            &Raw {
                line_type: None,
                ..Raw::priced(dec!(1))
            },
        )
        .await
        .expect("INSERT de forme 0.19.0");
        let lt: String = sqlx::query_scalar(&format!(
            "SELECT line_type FROM {} WHERE {} = ? AND position = ?",
            t.name(),
            t.parent()
        ))
        .bind(parent)
        .bind(p)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(lt, "priced", "{} : DEFAULT de la colonne", t.name());
    }
}

/// T4 (AC6, AC7, D10, D11) — brouillon à commentaire, no-op, audit.
#[sqlx::test(migrations = "./test-schema")]
async fn draft_roundtrip_and_no_op(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, &seeded).await;

    let lignes = vec![
        priced("A", dec!(2), dec!(100), dec!(8.10)),
        unpriced("Livraison du 3 juin", LineType::Comment),
        priced("B", dec!(1), dec!(50), dec!(2.60)),
    ];
    let (inv, lines) = invoices::create(
        &pool,
        seeded.admin_user_id,
        new_invoice(&seeded, contact, lignes.clone()),
    )
    .await
    .expect("création d'un brouillon à commentaire");

    // Positions 0..n tous types confondus ; total = lignes chiffrées seules.
    assert_eq!(
        lines.iter().map(|l| l.position).collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    assert_eq!(
        lines.iter().map(|l| l.line_type).collect::<Vec<_>>(),
        vec![LineType::Priced, LineType::Comment, LineType::Priced]
    );
    assert_eq!(lines[1].line_total, Decimal::ZERO);
    assert_eq!(lines[1].revenue_account_id, None);
    assert_eq!(inv.total_amount, dec!(250));

    // Relecture : le type survit.
    let (_, relues) = invoices::find_by_id_with_lines(&pool, seeded.company_id, inv.id)
        .await
        .unwrap()
        .expect("facture relue");
    assert_eq!(relues[1].line_type, LineType::Comment);

    let update = |lines: Vec<NewInvoiceLine>| InvoiceUpdate {
        contact_id: contact,
        date: inv.date,
        due_date: inv.due_date,
        payment_terms: inv.payment_terms.clone(),
        project_id: None,
        lines,
    };

    // Même contenu : no-op, la version ne bouge pas.
    let (same, _) = invoices::update(
        &pool,
        seeded.company_id,
        inv.id,
        inv.version,
        seeded.admin_user_id,
        update(lignes.clone()),
    )
    .await
    .unwrap();
    assert_eq!(same.version, inv.version, "contenu identique : no-op");

    // Seul le type change : c'est une modification (AC7).
    let mut retypees = lignes.clone();
    retypees[1].line_type = LineType::Heading;
    let (after, after_lines) = invoices::update(
        &pool,
        seeded.company_id,
        inv.id,
        inv.version,
        seeded.admin_user_id,
        update(retypees),
    )
    .await
    .unwrap();
    assert_eq!(
        after.version,
        inv.version + 1,
        "type seul changé : version +1"
    );
    assert_eq!(after_lines[1].line_type, LineType::Heading);
    assert_eq!(after.total_amount, dec!(250), "total inchangé");

    // L'audit porte `lineType` au « before » et à l'« after ».
    let details: serde_json::Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'invoice.updated' \
         AND entity_id = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(inv.id)
    .fetch_one(&pool)
    .await
    .expect("audit invoice.updated");
    assert_eq!(details["before"]["lines"][1]["lineType"], "comment");
    assert_eq!(details["after"]["lines"][1]["lineType"], "heading");
    assert_eq!(details["after"]["lines"][0]["lineType"], "priced");

    // Une ligne non chiffrée qui porte un montant ou un compte : `Invariant`,
    // rien n'est écrit (D10).
    for muter in [
        (|l: &mut NewInvoiceLine| l.quantity = dec!(1)) as fn(&mut NewInvoiceLine),
        |l| l.unit_price = dec!(1),
        |l| l.vat_rate = dec!(8.10),
        |l| l.revenue_account_id = Some(1),
    ] {
        let mut mauvaises = lignes.clone();
        muter(&mut mauvaises[1]);
        let err = invoices::create(
            &pool,
            seeded.admin_user_id,
            new_invoice(&seeded, contact, mauvaises.clone()),
        )
        .await
        .expect_err("ligne non neutre à la création");
        assert!(
            matches!(&err, DbError::Invariant(m) if m.starts_with("ligne 2 :")),
            "{err:?}"
        );
        let err = invoices::update(
            &pool,
            seeded.company_id,
            inv.id,
            after.version,
            seeded.admin_user_id,
            update(mauvaises),
        )
        .await
        .expect_err("ligne non neutre à la modification");
        assert!(matches!(err, DbError::Invariant(_)), "{err:?}");
    }
    let (_, inchangees) = invoices::find_by_id_with_lines(&pool, seeded.company_id, inv.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(inchangees[1].line_type, LineType::Heading, "rien d'écrit");

    // Une pièce de commentaires seuls se crée (brouillon) et ne se valide pas.
    let (seule, _) = invoices::create(
        &pool,
        seeded.admin_user_id,
        new_invoice(
            &seeded,
            contact,
            vec![
                unpriced("Titre", LineType::Heading),
                unpriced("Note", LineType::Comment),
            ],
        ),
    )
    .await
    .expect("brouillon de commentaires seuls");
    assert_eq!(seule.total_amount, Decimal::ZERO);
    let err = invoices::validate_invoice(&pool, seeded.company_id, seule.id, seeded.admin_user_id)
        .await
        .expect_err("commentaires seuls : pas de validation");
    assert!(
        matches!(&err, DbError::InvalidInput(m) if m == "invoiceTotalZero"),
        "{err:?}"
    );
}
