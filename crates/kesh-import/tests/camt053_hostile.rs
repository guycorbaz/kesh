//! Entrées hostiles du parseur CAMT.053 (Story v017-1, #789, AC5 à AC7).
//!
//! Chaque document est **construit en mémoire** à partir d'un relevé v04
//! complet (une seule fonction de construction, [`releve`]) : sans réseau ni
//! fichier. Chaque parse tourne dans un fil, attendu par un canal avec un délai
//! de [`DELAI`] :
//! - `Timeout` fait échouer le test (« le parseur n'a pas rendu la main ») ;
//! - `Disconnected` le fait échouer sous un autre message (« le fil du parseur
//!   est mort — panique »), sans bloquer la suite.
//!
//! Rouges sous `quick-xml` 0.39.4, verts sous 0.42 (C-v017-1-4) :
//! - AC5 — RUSTSEC-2026-0194 : contrôle quadratique des attributs en double
//!   (dans Kesh : `attr_string` sur chaque `<Amt>`) ;
//! - AC6 — RUSTSEC-2026-0195 : déclarations d'espace de noms non bornées ;
//! - AC7 — imbrication au-delà de `u16::MAX` (quick-xml #977).
//!
//! ⚠️ À l'état rouge (sous 0.39), le fil d'AC5 continue de tourner après
//! l'échec et ralentit les tests suivants du binaire sous `--test-threads=1`.
//! Le délai dépend de la machine : un rouge de délai isolé se rejoue avant
//! d'être diagnostiqué.

use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

use kesh_import::{CamtError, ImportedStatement, parse_camt053};

/// Délai accordé à chaque parse.
const DELAI: Duration = Duration::from_secs(10);
/// AC5 : attributs distincts sur un `<Amt>` (≈ 2,4 Mo, sous le plafond d'upload de 10 MiB).
const ATTRIBUTS_HOSTILES: usize = 200_000;
/// AC6 (a)(b) : déclarations d'espace de noms sur un seul élément.
const DECLARATIONS_HOSTILES: usize = 10_000;
/// AC6 (c) : déclarations portées par `<Document>` dans un relevé ordinaire
/// (`xmlns`, `xmlns:xsi` et trois préfixes inutilisés) — très en deçà du plafond
/// par défaut de quick-xml 0.42 (`DEFAULT_MAX_NAMESPACE_BINDINGS` = 128).
const DECLARATIONS_ORDINAIRES: usize = 5;
/// AC7 : niveaux d'imbrication, au-delà de `u16::MAX` (65 535).
const NIVEAUX_HOSTILES: usize = 70_000;

/// Ce que l'on injecte dans le relevé v04 complet.
#[derive(Default)]
struct Injection {
    /// Attributs ajoutés à `<Document>` (après `xmlns`).
    document: String,
    /// Attributs de `<Stmt>`.
    stmt: String,
    /// Attributs du `Ntry/Amt` (balise fermante : un `<Amt/>` vide n'est pas lu) ;
    /// `None` : ` Ccy="CHF"`, la devise sans laquelle l'entrée n'est pas complète.
    ntry_amt: Option<String>,
    /// Contenu inséré dans `<BkToCstmrStmt>`, avant le `<Stmt>`.
    avant_stmt: String,
}

/// Le relevé v04 complet — `Id`, IBAN, `Ccy`, soldes, période, un `<Ntry>`
/// complet (`Amt` avec texte, `BookgDt`, `CdtDbtInd`) — augmenté de l'injection.
/// Seule fonction de construction de ce fichier : le témoin d'AC5 prouve qu'elle
/// traverse un `<Stmt>` valide jusqu'à l'`<Amt>`.
fn releve(inj: &Injection) -> Vec<u8> {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.04"{document}>
  <BkToCstmrStmt>
    <GrpHdr>
      <MsgId>HOSTILE-001</MsgId>
      <CreDtTm>2026-10-30T12:00:00</CreDtTm>
    </GrpHdr>{avant_stmt}
    <Stmt{stmt}>
      <Id>HOSTILE-STMT-001</Id>
      <FrToDt>
        <FrDtTm>2026-10-01T00:00:00</FrDtTm>
        <ToDtTm>2026-10-31T23:59:59</ToDtTm>
      </FrToDt>
      <Acct>
        <Id>
          <IBAN>CH9300762011623852957</IBAN>
        </Id>
        <Ccy>CHF</Ccy>
      </Acct>
      <Bal>
        <Tp><CdOrPrtry><Cd>OPBD</Cd></CdOrPrtry></Tp>
        <Amt Ccy="CHF">100.00</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <Dt><Dt>2026-10-01</Dt></Dt>
      </Bal>
      <Bal>
        <Tp><CdOrPrtry><Cd>CLBD</Cd></CdOrPrtry></Tp>
        <Amt Ccy="CHF">101.00</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <Dt><Dt>2026-10-31</Dt></Dt>
      </Bal>
      <Ntry>
        <Amt{ntry_amt}>1.00</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <Sts>BOOK</Sts>
        <BookgDt><Dt>2026-10-15</Dt></BookgDt>
        <AcctSvcrRef>HOSTILE-TX-1</AcctSvcrRef>
      </Ntry>
    </Stmt>
  </BkToCstmrStmt>
</Document>
"#,
        document = inj.document,
        stmt = inj.stmt,
        ntry_amt = inj.ntry_amt.as_deref().unwrap_or(r#" Ccy="CHF""#),
        avant_stmt = inj.avant_stmt,
    )
    .into_bytes()
}

/// `n` déclarations `xmlns:pK="urn:x:K"`.
fn declarations(n: usize) -> String {
    (0..n)
        .map(|i| format!(r#" xmlns:p{i}="urn:x:{i}""#))
        .collect()
}

/// Parse dans un fil, borné par [`DELAI`]. `#[track_caller]` : un échec est
/// rapporté à la ligne de l'appel (ce qui distingue, dans AC5, le témoin du
/// document hostile).
#[track_caller]
fn parse_borne(xml: Vec<u8>) -> Result<Vec<ImportedStatement>, CamtError> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(parse_camt053(&xml));
    });
    match rx.recv_timeout(DELAI) {
        Ok(r) => r,
        Err(RecvTimeoutError::Timeout) => {
            panic!("le parseur n'a pas rendu la main en {} s", DELAI.as_secs())
        }
        Err(RecvTimeoutError::Disconnected) => {
            panic!("le fil du parseur est mort — panique")
        }
    }
}

/// AC5 — RUSTSEC-2026-0194 : 200 000 attributs distincts, sans `Ccy`, sur un
/// `Ntry/Amt` à balise fermante. Seul le délai est asserté (le document n'a pas
/// à être un relevé valide). Témoin : la même construction, trois attributs et
/// `Ccy="CHF"`, rend `Ok` avec la devise lue — sans lui, une retouche du
/// constructeur qui ferait échouer le parse avant `<Amt>` rendrait ce test vert
/// à vide.
#[test]
fn ac5_many_attributes_on_amt_return_in_time() {
    let temoin = parse_borne(releve(&Injection {
        ntry_amt: Some(r#" a0="" a1="" a2="" Ccy="CHF""#.into()),
        ..Default::default()
    }))
    .expect("le témoin se lit");
    assert_eq!(temoin[0].transactions[0].currency, "CHF");

    let attributs: String = (0..ATTRIBUTS_HOSTILES)
        .map(|i| format!(r#" a{i}="""#))
        .collect();
    let _ = parse_borne(releve(&Injection {
        ntry_amt: Some(attributs),
        ..Default::default()
    }));
}

/// AC6 (a) — 10 000 déclarations d'espace de noms sur `<Stmt>` : refus.
#[test]
fn ac6a_many_namespace_declarations_on_stmt_are_refused() {
    let r = parse_borne(releve(&Injection {
        stmt: declarations(DECLARATIONS_HOSTILES),
        ..Default::default()
    }));
    assert!(
        matches!(r, Err(CamtError::MalformedXml(_))),
        "Err(MalformedXml) attendu, obtenu {}",
        resume(&r)
    );
}

/// AC6 (b) — les mêmes 10 000 déclarations sur `<Document>` : refus.
#[test]
fn ac6b_many_namespace_declarations_on_document_are_refused() {
    let r = parse_borne(releve(&Injection {
        document: declarations(DECLARATIONS_HOSTILES),
        ..Default::default()
    }));
    assert!(
        matches!(r, Err(CamtError::MalformedXml(_))),
        "Err(MalformedXml) attendu, obtenu {}",
        resume(&r)
    );
}

/// AC6 (c) — pas de faux positif : un relevé dont `<Document>` porte cinq
/// déclarations (`xmlns`, `xmlns:xsi`, trois préfixes inutilisés) se lit à
/// l'identique du même relevé sans les préfixes inutilisés.
#[test]
fn ac6c_ordinary_namespace_declarations_are_accepted() {
    let xsi = r#" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance""#;
    let inutilises = r#" xmlns:a="urn:x:a" xmlns:b="urn:x:b" xmlns:c="urn:x:c""#;
    // `xmlns` (dans le gabarit) + `xmlns:xsi` + trois préfixes.
    assert_eq!(
        1 + xsi.matches("xmlns:").count() + inutilises.matches("xmlns:").count(),
        DECLARATIONS_ORDINAIRES
    );
    let avec = parse_borne(releve(&Injection {
        document: format!("{xsi}{inutilises}"),
        ..Default::default()
    }))
    .expect("cinq déclarations se lisent");
    let sans = parse_borne(releve(&Injection {
        document: xsi.into(),
        ..Default::default()
    }))
    .expect("le relevé de référence se lit");
    assert_eq!(avec, sans);
    assert_eq!(avec[0].transactions.len(), 1);
}

/// AC7 — 70 000 éléments `<X>` imbriqués dans `<BkToCstmrStmt>` : refus, sans
/// panique (le canal ne rend pas `Disconnected`).
#[test]
fn ac7_deep_nesting_is_refused_without_panic() {
    let mut profond = String::with_capacity(NIVEAUX_HOSTILES * 7);
    profond.push('\n');
    for _ in 0..NIVEAUX_HOSTILES {
        profond.push_str("<X>");
    }
    for _ in 0..NIVEAUX_HOSTILES {
        profond.push_str("</X>");
    }
    let r = parse_borne(releve(&Injection {
        avant_stmt: profond,
        ..Default::default()
    }));
    assert!(
        matches!(r, Err(CamtError::MalformedXml(_))),
        "Err(MalformedXml) attendu, obtenu {}",
        resume(&r)
    );
}

/// Résumé court d'un résultat (un `Ok` hostile n'est pas imprimé en entier).
fn resume(r: &Result<Vec<ImportedStatement>, CamtError>) -> String {
    match r {
        Ok(s) => format!("Ok({} relevé(s))", s.len()),
        Err(e) => format!("Err({e:?})"),
    }
}
