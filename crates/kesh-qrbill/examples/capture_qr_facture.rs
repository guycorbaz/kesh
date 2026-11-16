//! Les QR-factures de la recette visuelle (Stories 36-12a et 36-12b, #944) : des PDF de
//! fixtures **synthétiques**, en français et en anglais, écrits dans le répertoire donné en
//! argument, pour être rendus en PNG et comparés aux échantillons de SIX.
//!
//! ```sh
//! scripts/mem-guard.sh cargo run -p kesh-qrbill --example capture_qr_facture -- <répertoire>
//! mutool draw -r 150 -o <png> <pdf>
//! ```
//!
//! Les libellés sont ceux des `messages.ftl` de `kesh-i18n`, lus à la compilation : la
//! capture montre ce que la production imprime, langue par langue. Aucune donnée réelle :
//! les adresses et l'IBAN sont ceux des exemples de la directive.

use std::path::PathBuf;

use chrono::NaiveDate;
use kesh_qrbill::validation::build_qrr;
use kesh_qrbill::{
    Address, AddressType, Currency, InvoiceLinePdf, InvoicePdfData, InvoiceVatLinePdf, QrBillData,
    QrBillI18n, Reference, generate_qr_bill_pdf_with_date,
};
use printpdf::OffsetDateTime;
use rust_decimal::Decimal;

/// Les deux langues de la capture, avec leur fichier de traductions.
const LANGUES: [(&str, &str); 2] = [
    (
        "fr",
        include_str!("../../kesh-i18n/locales/fr-CH/messages.ftl"),
    ),
    (
        "en",
        include_str!("../../kesh-i18n/locales/en-CH/messages.ftl"),
    ),
];

/// Une adresse de type S en Suisse, la seule que la production imprime.
fn adresse(nom: &str, rue: &str, numero: &str, npa: &str, localite: &str) -> Address {
    Address {
        address_type: AddressType::Structured,
        name: nom.into(),
        line1: rue.into(),
        line2: numero.into(),
        postal_code: npa.into(),
        town: localite.into(),
        country: "CH".into(),
    }
}

/// Une fixture de la capture : son nom de fichier, et ses données.
struct Fixture {
    nom: &'static str,
    donnees: QrBillData,
}

/// Les trois fixtures : ordinaire (référence QRR), rue longue, sans référence (IBAN simple).
fn fixtures() -> Vec<Fixture> {
    let rue_longue = "Chemin des Trois-Fontaines de la Vieille Tour du Haut-Plateau";
    let base = QrBillData {
        creditor_iban: "CH4431999123000889012".into(),
        creditor: adresse("Robert Schneider SA", "Rue du Lac", "1268", "2501", "Biel"),
        ultimate_debtor: Some(adresse(
            "Pia Rutschmann",
            "Marktgasse",
            "28",
            "9400",
            "Rorschach",
        )),
        amount: Some(Decimal::new(129_240, 2)),
        currency: Currency::Chf,
        reference: Reference::Qrr(build_qrr(42, 100).expect("référence QRR")),
        unstructured_message: Some("Facture F-2026-0042".into()),
        billing_information: None,
    };
    vec![
        Fixture {
            nom: "ordinaire",
            donnees: base.clone(),
        },
        Fixture {
            nom: "rue-longue",
            donnees: QrBillData {
                creditor: adresse("Robert Schneider SA", rue_longue, "12", "2501", "Biel"),
                ultimate_debtor: Some(adresse(
                    "Pia Rutschmann",
                    rue_longue,
                    "12",
                    "9400",
                    "Rorschach",
                )),
                ..base.clone()
            },
        },
        Fixture {
            nom: "sans-reference",
            donnees: QrBillData {
                creditor_iban: "CH9300762011623852957".into(),
                reference: Reference::None,
                ..base
            },
        },
    ]
}

/// Le corps de la facture, le même pour toutes les fixtures.
fn facture() -> InvoicePdfData {
    InvoicePdfData {
        invoice_number: "F-2026-0042".into(),
        invoice_date: NaiveDate::from_ymd_opt(2026, 4, 14).expect("date"),
        due_date: NaiveDate::from_ymd_opt(2026, 5, 14),
        payment_terms: Some("30 jours net".into()),
        creditor_name: "Robert Schneider SA".into(),
        creditor_address_lines: vec!["Rue du Lac 1268".into(), "2501 Biel".into()],
        creditor_ide: Some("CHE-123.456.789".into()),
        creditor_phone: None,
        creditor_email: None,
        creditor_website: None,
        debtor_name: "Pia Rutschmann".into(),
        debtor_address_lines: vec!["Marktgasse 28".into(), "9400 Rorschach".into()],
        debtor_client_number: None,
        lines: vec![InvoiceLinePdf {
            description: "Conseil".into(),
            quantity: Decimal::new(10, 0),
            unit_price: Decimal::new(12_000, 2),
            vat_rate: Decimal::new(770, 2),
            line_total: Decimal::new(120_000, 2),
        }],
        subtotal_ht: Decimal::new(120_000, 2),
        vat_lines: vec![InvoiceVatLinePdf {
            rate_percent: Decimal::new(770, 2),
            amount: Decimal::new(9_240, 2),
        }],
        rounding: Decimal::ZERO,
        total: Decimal::new(129_240, 2),
        currency: Currency::Chf,
        origin_reference: None,
        reminder: None,
    }
}

fn main() {
    let dossier: PathBuf = std::env::args_os()
        .nth(1)
        .expect("usage : capture_qr_facture <répertoire>")
        .into();
    std::fs::create_dir_all(&dossier).expect("répertoire de sortie");
    let date = OffsetDateTime::from_unix_timestamp(1_767_225_600).expect("date fixe");
    for (langue, ftl) in LANGUES {
        let i18n = QrBillI18n::depuis_ftl(ftl);
        for fixture in fixtures() {
            let pdf = generate_qr_bill_pdf_with_date(&fixture.donnees, &facture(), &i18n, date)
                .expect("PDF rendu");
            let chemin = dossier.join(format!("{}-{langue}.pdf", fixture.nom));
            std::fs::write(&chemin, pdf).expect("PDF écrit");
            println!("{}", chemin.display());
        }
    }
}
