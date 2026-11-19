//! Les QR-factures de la recette visuelle (Stories 36-12a et 44-1a, #944) : des PDF de
//! fixtures **synthétiques**, en français, en allemand et en anglais, écrits dans le répertoire donné en
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
//!
//! Story 49-1b (#1242, AC12) : en plus, les pièces à **ligne de commentaire**, en français
//! (`49-1b-*.pdf`) — commentaire entre deux lignes, sur plusieurs paragraphes, long et
//! coupé, sur un avoir, titre de section (rendu comme un commentaire jusqu'à la 49-3) — et,
//! en témoin, une facture de lignes chiffrées seulement.

use std::path::PathBuf;

use chrono::NaiveDate;
use kesh_qrbill::validation::build_qrr;
use kesh_qrbill::{
    Address, AddressType, Currency, InvoiceLinePdf, InvoicePdfData, InvoiceVatLinePdf, LinePdfKind,
    QrBillData, QrBillI18n, Reference, generate_credit_note_pdf_with_date,
    generate_qr_bill_pdf_with_date,
};
use printpdf::OffsetDateTime;
use rust_decimal::Decimal;

/// Les trois langues de la capture, avec leur fichier de traductions (l'allemand depuis la
/// Story 44-1a, AC14 : ses libellés sont les plus larges).
const LANGUES: [(&str, &str); 3] = [
    (
        "fr",
        include_str!("../../kesh-i18n/locales/fr-CH/messages.ftl"),
    ),
    (
        "de",
        include_str!("../../kesh-i18n/locales/de-CH/messages.ftl"),
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

/// Une adresse de type S à l'étranger.
fn adresse_etrangere(
    nom: &str,
    rue: &str,
    numero: &str,
    npa: &str,
    localite: &str,
    pays: &str,
) -> Address {
    Address {
        country: pays.into(),
        ..adresse(nom, rue, numero, npa, localite)
    }
}

/// Les fixtures : ordinaire (référence QRR), rue longue, sans référence (IBAN simple) ; et,
/// depuis la Story 44-1a, une adresse réaliste de neuf lignes au récépissé (la rue y est
/// omise, palier R1) et le pire cas (paliers R4 et P3).
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
    // Le pire cas de la fiche 44-1a (AC5) : noms de 70 `W`, rue de 70 `W` et numéro de 16
    // chiffres, NPA étranger de 16 `W` et localité de 35 `W`, message de 139 caractères en
    // huit mots qui ne tiennent jamais à deux sur une ligne de 87 mm.
    let pire = adresse_etrangere(
        &"W".repeat(70),
        &"W".repeat(70),
        "1234567890123456",
        &"W".repeat(16),
        &"W".repeat(35),
        "DE",
    );
    let message_de_huit_lignes = [17, 16, 17, 16, 17, 16, 17, 16]
        .map(|n| "W".repeat(n))
        .join(" ");
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
                ..base.clone()
            },
        },
        Fixture {
            nom: "realiste-r1",
            donnees: QrBillData {
                creditor: adresse(
                    "Fondation suisse pour la protection du paysage",
                    "Chemin des Trois-Fontaines de la Vieille Tour",
                    "12",
                    "2501",
                    "Biel",
                ),
                ultimate_debtor: Some(adresse(
                    "Coopérative d'habitation des Jardins du Lac",
                    "Marktgasse",
                    "28",
                    "9400",
                    "Rorschach",
                )),
                ..base.clone()
            },
        },
        Fixture {
            nom: "pire-cas",
            donnees: QrBillData {
                creditor: pire.clone(),
                ultimate_debtor: Some(pire),
                unstructured_message: Some(message_de_huit_lignes),
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
            kind: LinePdfKind::Priced,
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

/// Une ligne de la capture 49-1b : `kind`, description, quantité, prix (centimes), taux (centièmes).
fn ligne_49(
    kind: LinePdfKind,
    description: &str,
    quantite: i64,
    prix: i64,
    taux: i64,
) -> InvoiceLinePdf {
    let (quantity, unit_price, vat_rate) = (
        Decimal::new(quantite, 0),
        Decimal::new(prix, 2),
        Decimal::new(taux, 2),
    );
    InvoiceLinePdf {
        kind,
        description: description.into(),
        quantity,
        unit_price,
        vat_rate,
        line_total: quantity * unit_price,
    }
}

/// Une facture de la capture 49-1b, ses sommes faites comme `kesh-api` les fait (toutes les
/// lignes ; une ligne non chiffrée porte zéro).
fn facture_49(numero: &str, lines: Vec<InvoiceLinePdf>) -> InvoicePdfData {
    let subtotal_ht: Decimal = lines.iter().map(|l| l.line_total).sum();
    let mut vat_lines: Vec<InvoiceVatLinePdf> = Vec::new();
    for l in lines.iter().filter(|l| l.vat_rate > Decimal::ZERO) {
        let tva = (l.line_total * l.vat_rate / Decimal::new(100, 0)).round_dp(2);
        match vat_lines.iter_mut().find(|v| v.rate_percent == l.vat_rate) {
            Some(v) => v.amount += tva,
            None => vat_lines.push(InvoiceVatLinePdf {
                rate_percent: l.vat_rate,
                amount: tva,
            }),
        }
    }
    let total = subtotal_ht + vat_lines.iter().map(|v| v.amount).sum::<Decimal>();
    InvoicePdfData {
        invoice_number: numero.into(),
        lines,
        subtotal_ht,
        vat_lines,
        total,
        ..facture()
    }
}

/// Les pièces de la recette visuelle de la Story 49-1b : `(nom, avoir ?, corps)`.
fn pieces_49_1b() -> Vec<(&'static str, bool, InvoicePdfData)> {
    use LinePdfKind::{Comment, Heading, Priced};
    let entre_deux = vec![
        ligne_49(Priced, "Loyer du mois de novembre", 1, 120_000, 0),
        ligne_49(
            Comment,
            "Appartement 3 pièces, 2e étage, rue du Lac 12",
            0,
            0,
            0,
        ),
        ligne_49(Priced, "Charges forfaitaires", 1, 18_000, 0),
    ];
    let paragraphes = vec![
        ligne_49(Priced, "Conseil stratégique", 4, 15_000, 810),
        ligne_49(
            Comment,
            "Prestations de novembre :\n- atelier du 4 novembre\n\n- suivi téléphonique",
            0,
            0,
            0,
        ),
    ];
    let long = vec![
        ligne_49(Priced, "Travaux de peinture", 1, 95_000, 810),
        ligne_49(
            Comment,
            "Selon devis du 2 octobre accepté le 9 octobre. Les travaux comprennent la \
             préparation des supports, deux couches de peinture acrylique mate sur les murs \
             et le plafond du séjour, la protection des sols et l'évacuation des déchets. \
             Garantie de deux ans sur l'adhérence, hors dégâts d'eau.",
            0,
            0,
            0,
        ),
    ];
    let titre = vec![
        ligne_49(Heading, "Travaux du lot B", 0, 0, 0),
        ligne_49(Priced, "Peinture du séjour", 1, 95_000, 810),
        ligne_49(Priced, "Ponçage des boiseries", 3, 4_500, 810),
    ];
    let temoin = vec![
        ligne_49(Priced, "Prestation 1", 1, 3_331, 810),
        ligne_49(Priced, "Prestation 2", 2, 3_331, 260),
        ligne_49(Priced, "Prestation 3", 3, 3_331, 810),
    ];
    let mut avoir = facture_49("AV-2026-0003", entre_deux.clone());
    avoir.origin_reference = Some("F-2026-0101".into());
    avoir.due_date = None;
    avoir.payment_terms = None;
    vec![
        (
            "49-1b-commentaire-entre-deux-lignes",
            false,
            facture_49("F-2026-0101", entre_deux),
        ),
        (
            "49-1b-commentaire-paragraphes",
            false,
            facture_49("F-2026-0102", paragraphes),
        ),
        (
            "49-1b-commentaire-long",
            false,
            facture_49("F-2026-0103", long),
        ),
        ("49-1b-avoir-commentaire", true, avoir),
        (
            "49-1b-titre-de-section",
            false,
            facture_49("F-2026-0104", titre),
        ),
        (
            "49-1b-temoin-chiffre",
            false,
            facture_49("F-2026-0105", temoin),
        ),
    ]
}

fn main() {
    let dossier: PathBuf = std::env::args_os()
        .nth(1)
        .expect("usage : capture_qr_facture <répertoire>")
        .into();
    std::fs::create_dir_all(&dossier).expect("répertoire de sortie");
    let date = OffsetDateTime::from_unix_timestamp(1_767_225_600).expect("date fixe");
    // Story 49-1b — les pièces à commentaire, en français.
    let fr = QrBillI18n::depuis_ftl(LANGUES[0].1);
    let mut fr_avoir = fr.clone();
    fr_avoir.entries.insert("invoice-pdf-title", "Avoir".into());
    fr_avoir
        .entries
        .insert("invoice-pdf-number", "N° d'avoir".into());
    let base = fixtures().remove(0).donnees;
    for (nom, est_avoir, corps) in pieces_49_1b() {
        let pdf = if est_avoir {
            generate_credit_note_pdf_with_date(&corps, &fr_avoir, date)
        } else {
            let donnees = QrBillData {
                amount: Some(corps.total.round_dp(2)),
                unstructured_message: Some(format!("Facture {}", corps.invoice_number)),
                ..base.clone()
            };
            generate_qr_bill_pdf_with_date(&donnees, &corps, &fr, date)
        }
        .expect("PDF 49-1b rendu");
        let chemin = dossier.join(format!("{nom}.pdf"));
        std::fs::write(&chemin, pdf).expect("PDF écrit");
        println!("{}", chemin.display());
    }
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
