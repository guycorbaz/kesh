//! Story 49-1b (#1242) — le PDF d'une facture, d'un rappel et d'un avoir faits de lignes
//! chiffrées se rend **à l'identique de la 0.19.0** (AC1, D4).
//!
//! Guy a déjà émis de vraies factures, remises sur papier ; un avoir n'est jamais figé et
//! se rend toujours avec le code en fonction. Les références de ce fichier ont été
//! capturées **avant** tout changement du rendu, dans un worktree propre et détaché au
//! commit de capture (fiche 49-1b, étape 0), par
//! `KESH_REFERENCES=ecrire cargo test -p kesh-qrbill --test rendu_avant_apres -- --ignored capturer`,
//! puis relues par `KESH_REFERENCES=comparer` (stabilité). Elles sont dans
//! `tests/fixtures/avant-0.20.0/`, avec le `sources.sha256` des fichiers de rendu compilés.
//!
//! La comparaison porte sur l'**empreinte** du rendu (`kesh_qrbill::empreinte`) : condensat
//! des opérations de contenu, polices, ressources, `MediaBox`, pages, et la liste lisible
//! des textes. Les octets du fichier ne peuvent pas servir (`/ID` et XMP aléatoires).
//!
//! ⛔ Les scénarios ne changent pas après la capture : un scénario modifié comparerait
//! autre chose que ce qui a été capturé. Les lignes se construisent par le seul [`ligne`].

use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use kesh_qrbill::empreinte::{
    Empreinte, ModeReferences, ecrire_ou_comparer, ecrire_sans_ecraser, empreinte, mode_references,
    sources_sha256,
};
use kesh_qrbill::{
    Address, AddressType, Currency, InvoiceLinePdf, InvoicePdfData, InvoiceVatLinePdf, QrBillData,
    QrBillI18n, Reference, ReminderPdf, generate_credit_note_pdf_with_date,
    generate_qr_bill_pdf_with_date, validation::build_qrr,
};
use printpdf::{BuiltinFont, Color, Line, Mm, OffsetDateTime, PdfDocument, Point, Rgb};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Les fichiers de rendu de `kesh-qrbill`, dont le `sources.sha256` garde l'empreinte
/// (AC1 (4)) ; chemins relatifs à la racine du dépôt.
const SOURCES_DU_RENDU: &[&str] = &[
    "crates/kesh-qrbill/src/pdf.rs",
    "crates/kesh-qrbill/src/types.rs",
    "crates/kesh-qrbill/src/largeurs.rs",
    "crates/kesh-qrbill/src/texte.rs",
    "crates/kesh-qrbill/src/generator.rs",
    "crates/kesh-qrbill/src/validation.rs",
    "crates/kesh-qrbill/src/parser.rs",
];

fn racine_du_depot() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("racine du dépôt")
}

fn dossier_des_references() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/avant-0.20.0")
}

/// Date de création figée : elle n'entre pas dans l'empreinte, mais deux rendus d'un même
/// scénario doivent rester comparables à l'octet près, hors `/ID` et XMP.
fn date_figee() -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(1_767_225_600).expect("timestamp valide")
}

/// **Le seul constructeur de ligne des scénarios** : une ligne chiffrée.
fn ligne(description: &str, quantite: Decimal, prix: Decimal, taux: Decimal) -> InvoiceLinePdf {
    InvoiceLinePdf {
        description: description.into(),
        quantity: quantite,
        unit_price: prix,
        vat_rate: taux,
        line_total: (quantite * prix).round_dp(2),
    }
}

/// Une adresse de type S, la seule que la production imprime.
fn adresse(nom: &str, rue: &str, numero: &str, npa: &str, ville: &str) -> Address {
    Address {
        address_type: AddressType::Structured,
        name: nom.into(),
        line1: rue.into(),
        line2: numero.into(),
        postal_code: npa.into(),
        town: ville.into(),
        country: "CH".into(),
    }
}

fn creancier() -> Address {
    adresse("Robert Schneider SA", "Rue du Lac", "1268", "2501", "Biel")
}

fn debiteur() -> Address {
    adresse("Pia Rutschmann", "Marktgasse", "28", "9400", "Rorschach")
}

/// Les sommes d'un document, faites comme `kesh-api` les fait : sous-total de toutes les
/// lignes, TVA par taux arrondie au centime (taux nul écarté), total = HT + TVA + arrondi.
fn sommes(
    lignes: &[InvoiceLinePdf],
    arrondi: Decimal,
) -> (Decimal, Vec<InvoiceVatLinePdf>, Decimal) {
    let sous_total: Decimal = lignes.iter().map(|l| l.line_total).sum();
    let mut par_taux: Vec<InvoiceVatLinePdf> = Vec::new();
    for l in lignes.iter().filter(|l| l.vat_rate > Decimal::ZERO) {
        let tva = (l.line_total * l.vat_rate / dec!(100)).round_dp(2);
        match par_taux.iter_mut().find(|v| v.rate_percent == l.vat_rate) {
            Some(v) => v.amount += tva,
            None => par_taux.push(InvoiceVatLinePdf {
                rate_percent: l.vat_rate,
                amount: tva,
            }),
        }
    }
    let tva: Decimal = par_taux.iter().map(|v| v.amount).sum();
    (sous_total, par_taux, sous_total + tva + arrondi)
}

/// Une facture : numéro, lignes, arrondi ; le message de la QR est celui que produit l'API
/// (« Facture <numéro> »).
fn facture(
    numero: &str,
    lignes: Vec<InvoiceLinePdf>,
    arrondi: Decimal,
) -> (QrBillData, InvoicePdfData) {
    let (subtotal_ht, vat_lines, total) = sommes(&lignes, arrondi);
    let data = QrBillData {
        creditor_iban: "CH4431999123000889012".into(),
        creditor: creancier(),
        ultimate_debtor: Some(debiteur()),
        amount: Some(total.round_dp(2)),
        currency: Currency::Chf,
        reference: Reference::Qrr(build_qrr(1, 42).expect("QRR")),
        unstructured_message: Some(format!("Facture {numero}")),
        billing_information: None,
    };
    let invoice = InvoicePdfData {
        invoice_number: numero.into(),
        invoice_date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
        due_date: NaiveDate::from_ymd_opt(2026, 5, 14),
        payment_terms: Some("30 jours net".into()),
        creditor_name: "Robert Schneider SA".into(),
        creditor_address_lines: vec!["Rue du Lac 1268".into(), "2501 Biel".into()],
        creditor_ide: Some("CHE-123.456.789 TVA".into()),
        creditor_phone: None,
        creditor_email: None,
        creditor_website: None,
        debtor_name: "Pia Rutschmann".into(),
        debtor_address_lines: vec!["Marktgasse 28".into(), "9400 Rorschach".into()],
        debtor_client_number: None,
        lines: lignes,
        subtotal_ht,
        vat_lines,
        rounding: arrondi,
        total,
        currency: Currency::Chf,
        origin_reference: None,
        reminder: None,
    };
    (data, invoice)
}

/// Des lignes numérotées, aux taux donnés en alternance.
fn lignes(n: usize, prix: Decimal, taux: &[Decimal]) -> Vec<InvoiceLinePdf> {
    (0..n)
        .map(|i| {
            ligne(
                &format!("Prestation {}", i + 1),
                Decimal::from(i as i64 + 1),
                prix,
                taux[i % taux.len()],
            )
        })
        .collect()
}

/// Les libellés d'un avoir, comme `credit_notes.rs` les surcharge (titre et numéro).
fn i18n_avoir() -> QrBillI18n {
    let mut i18n = QrBillI18n::default();
    i18n.entries.insert("invoice-pdf-title", "Avoir".into());
    i18n.entries
        .insert("invoice-pdf-number", "N° d'avoir".into());
    i18n
}

/// Les neuf scénarios de l'AC1, rendus : `(nom, octets du PDF)`.
fn scenarios() -> Vec<(&'static str, Vec<u8>)> {
    let i18n = QrBillI18n::default();
    let rendre = |(data, invoice): (QrBillData, InvoicePdfData)| {
        generate_qr_bill_pdf_with_date(&data, &invoice, &i18n, date_figee()).expect("rendu")
    };
    let mut sortie = Vec::new();

    // S1 — facture, une ligne, QR-IBAN et référence QRR.
    sortie.push((
        "S1-facture-une-ligne",
        rendre(facture(
            "F-2026-0001",
            vec![ligne(
                "Conseil stratégique",
                dec!(10),
                dec!(120.00),
                dec!(8.10),
            )],
            Decimal::ZERO,
        )),
    ));

    // S2 — cinq lignes, deux taux, arrondi à 5 centimes non nul.
    {
        let (mut data, mut inv) = facture(
            "F-2026-0002",
            lignes(5, dec!(33.33), &[dec!(8.10), dec!(2.60)]),
            Decimal::ZERO,
        );
        let brut = inv.total;
        let arrondi = (brut * dec!(20)).round() / dec!(20) - brut;
        assert!(!arrondi.is_zero(), "S2 doit porter un arrondi");
        inv.rounding = arrondi;
        inv.total = brut + arrondi;
        data.amount = Some(inv.total.round_dp(2));
        sortie.push(("S2-facture-deux-taux-arrondi", rendre((data, inv))));
    }

    // S3 — huit lignes sans TVA : la limite d'une facture sans récapitulatif.
    sortie.push((
        "S3-facture-huit-lignes",
        rendre(facture(
            "F-2026-0003",
            lignes(8, dec!(50.00), &[Decimal::ZERO]),
            Decimal::ZERO,
        )),
    ));

    // S4 — avoir à douze lignes, multi-taux, pleine page.
    {
        let (_, mut inv) = facture(
            "AV-2026-0001",
            lignes(12, dec!(25.00), &[dec!(8.10), dec!(2.60), dec!(3.80)]),
            Decimal::ZERO,
        );
        inv.origin_reference = Some("F-2026-0004".into());
        inv.due_date = None;
        inv.payment_terms = None;
        let pdf = generate_credit_note_pdf_with_date(&inv, &i18n_avoir(), date_figee())
            .expect("rendu de l'avoir");
        sortie.push(("S4-avoir-douze-lignes", pdf));
    }

    // S5 — rappel : déjà réglé, reste dû, frais.
    {
        let (mut data, mut inv) = facture(
            "F-2026-0005",
            lignes(2, dec!(100.00), &[dec!(8.10)]),
            Decimal::ZERO,
        );
        let reste = dec!(116.20);
        inv.reminder = Some(ReminderPdf {
            amount_settled: inv.total - reste,
            amount_due: reste,
            fees: dec!(20.00),
        });
        data.amount = Some(reste);
        sortie.push(("S5-rappel", rendre((data, inv))));
    }

    // S6 — société non assujettie : ni IDE ni récapitulatif, coordonnées de contact.
    {
        let (data, mut inv) = facture(
            "F-2026-0006",
            lignes(3, dec!(80.00), &[Decimal::ZERO]),
            Decimal::ZERO,
        );
        inv.creditor_ide = None;
        inv.creditor_phone = Some("+41 32 000 00 00".into());
        inv.creditor_email = Some("factures@example.ch".into());
        inv.creditor_website = Some("www.example.ch".into());
        sortie.push(("S6-facture-non-assujettie", rendre((data, inv))));
    }

    // S7 — une description de plus de 45 caractères (réduite à 44 et « … », D8).
    sortie.push((
        "S7-facture-description-longue",
        rendre(facture(
            "F-2026-0007",
            vec![
                ligne(
                    "Entretien annuel de la chaudière et des radiateurs du bâtiment B",
                    dec!(1),
                    dec!(480.00),
                    dec!(8.10),
                ),
                ligne("Déplacement", dec!(1), dec!(60.00), dec!(8.10)),
            ],
            Decimal::ZERO,
        )),
    ));

    // S8 — sans référence (IBAN ordinaire) et adresses longues : la coupe d'adresse de la
    // section de paiement (`couper_ligne`, premier appelant).
    {
        let (mut data, inv) = facture(
            "F-2026-0008",
            vec![ligne(
                "Location de salle",
                dec!(2),
                dec!(150.00),
                dec!(8.10),
            )],
            Decimal::ZERO,
        );
        data.creditor_iban = "CH9300762011623852957".into();
        data.reference = Reference::None;
        data.creditor = adresse(
            "Fondation pour l'entretien des immeubles locatifs du quartier",
            "Avenue de la Gare des Marchandises et du Port",
            "125b",
            "1003",
            "Lausanne",
        );
        data.ultimate_debtor = Some(adresse(
            "Association des locataires et des propriétaires réunis",
            "Chemin des Vignes de la Côte Ensoleillée",
            "7",
            "1820",
            "Montreux",
        ));
        sortie.push((
            "S8-facture-sans-reference-adresses-longues",
            rendre((data, inv)),
        ));
    }

    // S9 — un numéro de facture long : le message de la QR (« Facture <numéro> ») se coupe
    // sur plusieurs lignes au palier P0 (`couper_ligne`, second appelant).
    sortie.push((
        "S9-facture-message-long",
        rendre(facture(
            "F-2026-RENOVATION-IMMEUBLE-RUE-DU-LAC-LOT-B-ETAGE-3-APPARTEMENT-12",
            vec![ligne(
                "Travaux de peinture",
                dec!(1),
                dec!(950.00),
                dec!(8.10),
            )],
            Decimal::ZERO,
        )),
    ));

    sortie
}

/// Rend chaque scénario retenu par `filtre`, l'écrit ou le compare selon `mode`, et rend
/// les écarts **par scénario** (vide : tout égale la référence).
fn verifier(mode: ModeReferences, filtre: impl Fn(&str) -> bool) -> Vec<String> {
    let mut ecarts = Vec::new();
    for (nom, pdf) in scenarios().into_iter().filter(|(nom, _)| filtre(nom)) {
        let obtenue = empreinte(&pdf);
        if mode == ModeReferences::Ecrire {
            // Stabilité (D4) : un second rendu du même scénario donne la même empreinte.
            let second = scenarios()
                .into_iter()
                .find(|(n, _)| *n == nom)
                .map(|(_, pdf)| empreinte(&pdf))
                .expect("second rendu");
            assert_eq!(obtenue, second, "{nom} : deux rendus, deux empreintes");
        }
        let chemin = dossier_des_references().join(format!("{nom}.empreinte"));
        for e in ecrire_ou_comparer(mode, &chemin, &obtenue) {
            ecarts.push(format!("{nom} : {e}"));
        }
    }
    ecarts
}

/// **T0** — capture (`KESH_REFERENCES=ecrire`) ou relecture (`comparer`, par défaut) des
/// références de l'AC1. Ne se lance qu'à la main, dans le worktree de capture.
#[test]
#[ignore = "capture des références de la 0.19.0 (étape 0 de la 49-1b) : KESH_REFERENCES=ecrire|comparer"]
fn capturer_references_avant_0_20_0() {
    let mode = mode_references();
    let ecarts = verifier(mode, |_| true);
    if mode == ModeReferences::Ecrire {
        ecrire_sans_ecraser(
            &dossier_des_references().join("sources.sha256"),
            &sources_sha256(&racine_du_depot(), SOURCES_DU_RENDU),
        );
    }
    assert!(
        ecarts.is_empty(),
        "écarts à la référence :\n{}",
        ecarts.join("\n")
    );
}

// ----- T1-bis : l'empreinte distingue un rendu d'un autre -----

/// Ce qui distingue un PDF fabriqué du témoin.
#[derive(Clone, Copy, Debug)]
enum Variante {
    Temoin,
    /// Le texte déplacé de 0,01 mm.
    Decalage,
    /// Un caractère du texte changé.
    Texte,
    /// L'épaisseur du trait changée d'un cheveu (une opération de contenu).
    Operation,
    /// Une police intégrée de plus, jamais employée.
    Police,
    /// Une page plus large d'un millimètre.
    MediaBox,
    /// Une page de plus.
    Page,
}

/// Un PDF fabriqué par `printpdf`, comme le fait `kesh-qrbill` : un texte et un trait.
fn pdf_fabrique(v: Variante) -> Vec<u8> {
    let largeur = if matches!(v, Variante::MediaBox) {
        211.0
    } else {
        210.0
    };
    let (doc, page, calque) = PdfDocument::new("T1-bis", Mm(largeur), Mm(297.0), "Layer 1");
    let doc = doc
        .with_creation_date(date_figee())
        .with_mod_date(date_figee())
        .with_metadata_date(date_figee())
        .with_document_id("kesh-t1-bis".into());
    let helv = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
    if matches!(v, Variante::Police) {
        doc.add_builtin_font(BuiltinFont::HelveticaOblique).unwrap();
    }
    let calque = doc.get_page(page).get_layer(calque);
    let x = if matches!(v, Variante::Decalage) {
        20.01
    } else {
        20.0
    };
    let texte = if matches!(v, Variante::Texte) {
        "Conseik"
    } else {
        "Conseil"
    };
    calque.use_text(texte, 9.0, Mm(x), Mm(159.0), &helv);
    calque.set_outline_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
    let epaisseur = if matches!(v, Variante::Operation) {
        0.51
    } else {
        0.5
    };
    calque.set_outline_thickness(epaisseur);
    calque.add_line(Line {
        points: vec![
            (Point::new(Mm(20.0), Mm(150.0)), false),
            (Point::new(Mm(190.0), Mm(150.0)), false),
        ],
        is_closed: false,
    });
    if matches!(v, Variante::Page) {
        doc.add_page(Mm(largeur), Mm(297.0), "Layer 1");
    }
    doc.save_to_bytes().unwrap()
}

/// **T1-bis** — l'empreinte distingue les six paires fabriquées, et une paire identique
/// donne la même empreinte. Le test n'affirme pas quelle composante diffère.
#[test]
fn l_empreinte_change_des_qu_un_rendu_change() {
    let temoin: Empreinte = empreinte(&pdf_fabrique(Variante::Temoin));
    assert_eq!(
        temoin,
        empreinte(&pdf_fabrique(Variante::Temoin)),
        "une paire identique donne la même empreinte"
    );
    let mut indistinctes = Vec::new();
    for v in [
        Variante::Decalage,
        Variante::Texte,
        Variante::Operation,
        Variante::Police,
        Variante::MediaBox,
        Variante::Page,
    ] {
        if empreinte(&pdf_fabrique(v)) == temoin {
            indistinctes.push(v);
        }
    }
    assert!(
        indistinctes.is_empty(),
        "l'empreinte ne distingue pas du témoin : {indistinctes:?}"
    );
}
