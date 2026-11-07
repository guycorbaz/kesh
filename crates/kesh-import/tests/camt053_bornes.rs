//! Story 33-4 (AC-7, AC-11, refs #606) — les années d'un relevé, et la frontière
//! avec la lecture héritée (C-e334-26).
//!
//! `parse` refuse une année que la base ne sait pas tenir ; `parse_legacy_0173`,
//! qui recalcule la clé de doublon d'une ligne déjà importée, reste la lecture de
//! la 0.17.3 et **ne voit pas la borne**. Les relevés sont les fixtures minimales,
//! dont une date est remplacée en mémoire.

use chrono::NaiveDate;
use kesh_import::camt053::parse_legacy_0173;
use kesh_import::{CamtError, parse_camt053};

/// Une fixture minimale, sa date de comptabilisation remplacée.
fn releve(fixture: &str, ancienne: &str, nouvelle: &str) -> Vec<u8> {
    let chemin = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/camt053")
        .join(fixture);
    let texte = std::fs::read_to_string(chemin).expect("fixture");
    assert!(texte.contains(ancienne), "{ancienne} absent de {fixture}");
    texte.replacen(ancienne, nouvelle, 1).into_bytes()
}

const V04: &str = "v04_minimal.xml";
const V08: &str = "v08_minimal.xml";
const COMPTABILISATION_V04: &str = "<BookgDt>\n          <Dt>2026-05-15</Dt>";

fn dt(d: &str) -> String {
    format!("<BookgDt>\n          <Dt>{d}</Dt>")
}

/// La date de comptabilisation de la fixture v08, lue et non supposée.
fn comptabilisation_v08() -> (String, String) {
    let chemin = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/camt053")
        .join(V08);
    let texte = std::fs::read_to_string(chemin).unwrap();
    let debut = texte.find("<BookgDt>").expect("BookgDt");
    let fin = texte[debut..].find("</Dt>").unwrap() + debut + "</Dt>".len();
    let bloc = texte[debut..fin].to_string();
    let date_debut = bloc.find("<Dt>").unwrap() + "<Dt>".len();
    let prefixe = bloc[..date_debut].to_string();
    (bloc, prefixe)
}

fn releves_a(date: &str) -> Vec<Vec<u8>> {
    let (bloc_v08, prefixe_v08) = comptabilisation_v08();
    vec![
        releve(V04, COMPTABILISATION_V04, &dt(date)),
        releve(V08, &bloc_v08, &format!("{prefixe_v08}{date}</Dt>")),
    ]
}

/// AC-7 : `v04::parse` et `v08::parse`, chacun.
#[test]
fn une_annee_hors_plage_est_une_date_invalide() {
    for hors in ["0999-12-31", "+10000-01-01", "-0001-01-01", "+70000-01-01"] {
        for (i, xml) in releves_a(hors).into_iter().enumerate() {
            match parse_camt053(&xml) {
                Err(CamtError::InvalidDate(d)) => {
                    assert!(d.contains(hors.trim_start_matches('+')), "{d}");
                }
                autre => panic!("{hors}, relevé {i} : {autre:?}"),
            }
        }
    }
}

/// La date de valeur et la période passent par le même contrôle.
#[test]
fn la_valeur_et_la_periode_aussi() {
    let chemin = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/camt053")
        .join(V04);
    let texte = std::fs::read_to_string(chemin).unwrap();
    let valeur = texte.find("<ValDt>").expect("ValDt");
    let dt_ouvre = texte[valeur..].find("<Dt>").unwrap() + valeur + "<Dt>".len();
    let dt_ferme = texte[dt_ouvre..].find("</Dt>").unwrap() + dt_ouvre;
    let mut xml = texte.clone();
    xml.replace_range(dt_ouvre..dt_ferme, "0999-12-31");
    assert!(matches!(
        parse_camt053(xml.as_bytes()),
        Err(CamtError::InvalidDate(_))
    ));
    let xml = texte.replacen(
        "<FrDtTm>2026-05-01T00:00:00",
        "<FrDtTm>0999-05-01T00:00:00",
        1,
    );
    assert!(matches!(
        parse_camt053(xml.as_bytes()),
        Err(CamtError::InvalidDate(_))
    ));
    let xml = texte.replacen(
        "<ToDtTm>2026-05-31T23:59:59",
        "<ToDtTm>+10000-05-31T23:59:59",
        1,
    );
    assert!(matches!(
        parse_camt053(xml.as_bytes()),
        Err(CamtError::InvalidDate(_))
    ));
}

/// AC-4 : les deux bornes passent, dans les deux versions.
#[test]
fn les_deux_bornes_passent() {
    for bon in ["1000-01-01", "9999-12-31"] {
        for xml in releves_a(bon) {
            let stmts = parse_camt053(&xml).unwrap_or_else(|e| panic!("{bon} : {e:?}"));
            assert_eq!(stmts[0].transactions[0].booking_date.to_string(), bon);
        }
    }
}

/// AC-11 (a) : la lecture héritée lit ce que la corrigée refuse.
#[test]
fn la_lecture_heritee_ne_voit_pas_la_borne() {
    for xml in releves_a("0999-12-31") {
        assert!(matches!(
            parse_camt053(&xml),
            Err(CamtError::InvalidDate(_))
        ));
        let stmts = parse_legacy_0173(&xml).expect("lecture héritée");
        assert_eq!(
            stmts[0].transactions[0].booking_date,
            NaiveDate::from_ymd_opt(999, 12, 31).unwrap()
        );
    }
}

/// AC-11 (b) : une référence de caractère dans la date. La lecture corrigée lit
/// le texte entier ; la lecture héritée ignore la référence et lit l'an 26, comme
/// la 0.17.3 : ce qui ne doit pas se refuser, sans quoi la clé de doublon d'une
/// ligne déjà importée changerait.
#[test]
fn la_lecture_heritee_garde_l_an_26() {
    let xml = releve(V04, COMPTABILISATION_V04, &dt("&#x32;026-10-10"));
    let stmts = parse_camt053(&xml).expect("lecture corrigée");
    assert_eq!(
        stmts[0].transactions[0].booking_date,
        NaiveDate::from_ymd_opt(2026, 10, 10).unwrap()
    );
    let stmts = parse_legacy_0173(&xml).expect("lecture héritée");
    assert_eq!(
        stmts[0].transactions[0].booking_date,
        NaiveDate::from_ymd_opt(26, 10, 10).unwrap()
    );
}
