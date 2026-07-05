//! Parseur CAMT.053 ISO 20022 (versions `.04` et `.08`).
//!
//! Le point d'entrée [`parse`] détecte la version via le namespace racine
//! `<Document>` et délègue à [`v04::parse`] ou [`v08::parse`]. Les deux
//! versions partagent la même logique de parsing (les sous-modules sont
//! des wrappers minces autour de [`parse_with_version`]) — le delta v04 → v08
//! au niveau des tags effectivement extraits par le parseur Kesh est
//! limité au wrapping `<Pty>` autour de `<Cdtr>` / `<Dbtr>` dans
//! `<RltdPties>`, pris en charge par le matching de chemin.
//!
//! La stratégie est pull-based : `quick_xml::NsReader` produit un flux
//! d'événements `Start` / `Text` / `End` que le parseur consomme en
//! maintenant une pile de chemin (`path: Vec<String>`) et des builders
//! pour les éléments composites (`<Stmt>`, `<Bal>`, `<Ntry>`, `<TxDtls>`).
//! Les builders sont initialisés sur l'`Event::Start` de leur tag
//! d'ancrage et finalisés sur l'`Event::End` correspondant ; les
//! événements `Text` sont dispatchés au builder actif selon le contexte
//! (bal, ntry, tx-dtls ou stmt).
//!
//! Les types autonomes [`crate::types::ImportedStatement`] et
//! [`crate::types::ImportedTransaction`] ne portent **pas** les clés
//! étrangères (`bank_account_id`, `import_id`, `company_id`) — celles-ci
//! sont injectées par `kesh-core::bank_imports::from_imported`. Le
//! parseur reste ainsi publiable indépendamment (décision archi #7).

pub mod v04;
pub mod v08;

use std::str::FromStr;

use chrono::NaiveDate;
use quick_xml::NsReader;
use quick_xml::events::{BytesStart, Event};
use quick_xml::name::{Namespace, ResolveResult};
use rust_decimal::Decimal;

use crate::error::CamtError;
use crate::types::{ImportedStatement, ImportedTransaction, SourceFormat};

/// Namespace XML de la version `camt.053.001.04`.
pub const NS_V04: &str = "urn:iso:std:iso:20022:tech:xsd:camt.053.001.04";

/// Namespace XML de la version `camt.053.001.08`.
pub const NS_V08: &str = "urn:iso:std:iso:20022:tech:xsd:camt.053.001.08";

/// Parse un document CAMT.053 et retourne tous les `<Stmt>` qu'il contient.
///
/// La version (v04 ou v08) est détectée à partir du namespace racine du
/// tag `<Document>`, indépendamment du préfixe utilisé (forme par défaut
/// `<Document xmlns="...">` ou préfixée `<ns:Document xmlns:ns="...">`).
///
/// # Erreurs
///
/// - [`CamtError::MalformedXml`] : XML invalide ou tronqué.
/// - [`CamtError::UnsupportedVersion`] : namespace `<Document>` autre que
///   `camt.053.001.04` ou `camt.053.001.08`.
/// - [`CamtError::MissingRequiredField`] : un champ requis pour construire
///   un `ImportedStatement` ou une `ImportedTransaction` est absent.
/// - [`CamtError::InvalidAmount`] / [`CamtError::InvalidDate`] : valeur
///   non parseable dans un `<Amt>` ou un élément date.
pub fn parse(xml: &[u8]) -> Result<Vec<ImportedStatement>, CamtError> {
    let mut reader = NsReader::from_reader(xml);
    {
        let cfg = reader.config_mut();
        cfg.trim_text(true);
        // Defense-in-depth : explicite, indépendant du défaut quick-xml.
        // Sans ça, un futur bump du crate qui flippe le défaut désynchroniserait
        // silencieusement la pile de chemin et corromprait le parsing.
        cfg.check_end_names = true;
    }

    let mut buf = Vec::new();

    loop {
        match reader.read_resolved_event_into(&mut buf) {
            Err(e) => return Err(CamtError::MalformedXml(e.to_string())),
            Ok((_, Event::Eof)) => {
                return Err(CamtError::MalformedXml(
                    "élément racine <Document> introuvable".into(),
                ));
            }
            Ok((ns, Event::Start(start))) if start.local_name().as_ref() == b"Document" => {
                let uri = match ns {
                    ResolveResult::Bound(Namespace(b)) => std::str::from_utf8(b)
                        .map_err(|e| CamtError::MalformedXml(e.to_string()))?
                        .to_owned(),
                    ResolveResult::Unbound => String::new(),
                    ResolveResult::Unknown(prefix) => {
                        return Err(CamtError::MalformedXml(format!(
                            "préfixe namespace non lié : {}",
                            String::from_utf8_lossy(&prefix)
                        )));
                    }
                };
                buf.clear();
                return match uri.as_str() {
                    NS_V04 => v04::parse(&mut reader),
                    NS_V08 => v08::parse(&mut reader),
                    other => Err(CamtError::UnsupportedVersion(other.to_string())),
                };
            }
            _ => {}
        }
        buf.clear();
    }
}

/// Logique de parsing partagée par v04 et v08, paramétrée par le suffixe
/// de version inscrit dans [`SourceFormat::Camt053`] (`"001.04"` ou
/// `"001.08"`).
///
/// Le reader doit avoir consommé le tag `<Document>` ouvrant ; le parseur
/// continue à partir de `<BkToCstmrStmt>` jusqu'à `</Document>`.
pub(crate) fn parse_with_version<R: std::io::BufRead>(
    reader: &mut NsReader<R>,
    version: &str,
) -> Result<Vec<ImportedStatement>, CamtError> {
    let mut buf = Vec::new();
    let mut path: Vec<String> = Vec::new();
    let mut stmts: Vec<ImportedStatement> = Vec::new();
    let mut sb: Option<StmtBuilder> = None;
    // Compteurs de localisation pour enrichir les erreurs
    // `MissingRequiredField` avec un dot-path indexé style
    // `stmt[2].ntry[5].amount` — voir doc de [`CamtError::MissingRequiredField`].
    let mut stmt_index: usize = 0;
    let mut ntry_index: usize = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => return Err(CamtError::MalformedXml(e.to_string())),
            Ok(Event::Eof) => {
                // Tags non fermés à EOF : XML tronqué. quick-xml peut
                // renvoyer Eof sans erreur quand la fin du fichier est
                // atteinte au milieu d'un élément, donc on contrôle ici
                // que la pile de chemin est bien vide et que tous les
                // builders sont fermés.
                if !path.is_empty() || sb.is_some() {
                    return Err(CamtError::MalformedXml(format!(
                        "fin de fichier inattendue, éléments non fermés : {}",
                        path.join(" > ")
                    )));
                }
                break;
            }
            Ok(Event::Start(start)) => {
                let local = local_name(start.local_name().as_ref())?;

                match local.as_str() {
                    "Stmt" => {
                        sb = Some(StmtBuilder::default());
                        ntry_index = 0;
                    }
                    "Bal" => {
                        if let Some(s) = sb.as_mut() {
                            s.bal = Some(BalBuilder::default());
                        }
                    }
                    "Ntry" => {
                        if let Some(s) = sb.as_mut() {
                            s.ntry = Some(NtryBuilder::default());
                        }
                    }
                    "TxDtls" => {
                        if let Some(n) = sb.as_mut().and_then(|s| s.ntry.as_mut()) {
                            n.current_txdtls = Some(TxDtlsBuilder::default());
                        }
                    }
                    "Amt" => {
                        if let Some(ccy) = attr_string(&start, b"Ccy")?
                            && let Some(s) = sb.as_mut()
                            && let Some(n) = s.ntry.as_mut()
                        {
                            if let Some(t) = n.current_txdtls.as_mut() {
                                t.currency = Some(ccy);
                            } else {
                                n.currency = Some(ccy);
                            }
                        }
                    }
                    _ => {}
                }

                path.push(local);
            }
            Ok(Event::End(end)) => {
                let local = local_name(end.local_name().as_ref())?;

                match local.as_str() {
                    "Bal" => {
                        if let Some(s) = sb.as_mut()
                            && let Some(b) = s.bal.take()
                        {
                            apply_balance(s, b, stmt_index)?;
                        }
                    }
                    "TxDtls" => {
                        if let Some(n) = sb.as_mut().and_then(|s| s.ntry.as_mut())
                            && let Some(t) = n.current_txdtls.take()
                        {
                            n.txdtls.push(t);
                        }
                    }
                    "Ntry" => {
                        if let Some(s) = sb.as_mut()
                            && let Some(n) = s.ntry.take()
                        {
                            let txs = emit_transactions(n, stmt_index, ntry_index)?;
                            s.transactions.extend(txs);
                            ntry_index += 1;
                        }
                    }
                    "Stmt" => {
                        if let Some(b) = sb.take() {
                            stmts.push(finalize_statement(b, version, stmt_index)?);
                            stmt_index += 1;
                        }
                    }
                    _ => {}
                }

                path.pop();
            }
            Ok(Event::Text(t)) => {
                let decoded = t
                    .decode()
                    .map_err(|e| CamtError::MalformedXml(e.to_string()))?;
                let unescaped = quick_xml::escape::unescape(&decoded)
                    .map_err(|e| CamtError::MalformedXml(e.to_string()))?;
                let txt = unescaped.trim();
                if !txt.is_empty()
                    && let Some(s) = sb.as_mut()
                {
                    handle_text(s, &path, txt)?;
                }
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(stmts)
}

#[derive(Default)]
struct StmtBuilder {
    statement_id: Option<String>,
    account_iban: Option<String>,
    currency: Option<String>,
    period_from: Option<NaiveDate>,
    period_to: Option<NaiveDate>,
    opening_balance: Option<Decimal>,
    closing_balance: Option<Decimal>,
    transactions: Vec<ImportedTransaction>,

    bal: Option<BalBuilder>,
    ntry: Option<NtryBuilder>,
}

#[derive(Default)]
struct BalBuilder {
    code: Option<String>,
    amount: Option<Decimal>,
    sign: Option<String>,
}

#[derive(Default)]
struct NtryBuilder {
    booking_date: Option<NaiveDate>,
    value_date: Option<NaiveDate>,
    amount: Option<Decimal>,
    sign: Option<String>,
    currency: Option<String>,
    reference: Option<String>,
    transaction_id: Option<String>,
    addtl_ntry_inf: Option<String>,

    txdtls: Vec<TxDtlsBuilder>,
    current_txdtls: Option<TxDtlsBuilder>,
}

#[derive(Default)]
struct TxDtlsBuilder {
    end_to_end_id: Option<String>,
    transaction_id: Option<String>,
    amount: Option<Decimal>,
    sign: Option<String>,
    currency: Option<String>,
    cdtr_ref: Option<String>,
    ustrd_parts: Vec<String>,
    counterparty_name: Option<String>,
    counterparty_iban: Option<String>,
}

fn handle_text(stmt: &mut StmtBuilder, path: &[String], txt: &str) -> Result<(), CamtError> {
    if let Some(n) = stmt.ntry.as_mut() {
        if let Some(t) = n.current_txdtls.as_mut() {
            return handle_txdtls_text(t, path, txt);
        }
        return handle_ntry_text(n, path, txt);
    }
    if let Some(b) = stmt.bal.as_mut() {
        return handle_bal_text(b, path, txt);
    }
    handle_stmt_text(stmt, path, txt)
}

fn handle_stmt_text(stmt: &mut StmtBuilder, path: &[String], txt: &str) -> Result<(), CamtError> {
    // Tous les matchers anchorent le suffix complet du chemin via
    // `ends_with(...)` pour rester robustes aux extensions vendor qui
    // pourraient introduire un autre `<Id>` / `<FrDtTm>` à un niveau
    // imbriqué (review code Pass 1, finding F7 — alignement avec les
    // matchers Acct/Id/IBAN, Acct/Ccy).
    let leaf = path.last().map(String::as_str).unwrap_or("");
    match leaf {
        "Id" => {
            if ends_with(path, &["Stmt", "Id"]) {
                stmt.statement_id = Some(txt.to_string());
            }
        }
        "IBAN" => {
            if ends_with(path, &["Acct", "Id", "IBAN"]) {
                stmt.account_iban = Some(txt.to_string());
            }
        }
        "Ccy" => {
            if ends_with(path, &["Acct", "Ccy"]) {
                stmt.currency = Some(txt.to_string());
            }
        }
        "FrDtTm" => {
            if ends_with(path, &["FrToDt", "FrDtTm"]) {
                stmt.period_from = Some(parse_date(txt)?);
            }
        }
        "ToDtTm" if ends_with(path, &["FrToDt", "ToDtTm"]) => {
            stmt.period_to = Some(parse_date(txt)?);
        }
        _ => {}
    }
    Ok(())
}

fn handle_bal_text(bal: &mut BalBuilder, path: &[String], txt: &str) -> Result<(), CamtError> {
    let leaf = path.last().map(String::as_str).unwrap_or("");
    match leaf {
        "Cd" => {
            // <Bal><Tp><CdOrPrtry><Cd>OPBD|CLBD</Cd>...</CdOrPrtry></Tp>
            if ends_with(path, &["Tp", "CdOrPrtry", "Cd"]) {
                bal.code = Some(txt.to_string());
            }
        }
        "Amt" => {
            if ends_with(path, &["Bal", "Amt"]) {
                bal.amount = Some(
                    Decimal::from_str(txt)
                        .map_err(|_| CamtError::InvalidAmount(txt.to_string()))?,
                );
            }
        }
        "CdtDbtInd" if ends_with(path, &["Bal", "CdtDbtInd"]) => {
            bal.sign = Some(txt.to_string());
        }
        _ => {}
    }
    Ok(())
}

fn handle_ntry_text(ntry: &mut NtryBuilder, path: &[String], txt: &str) -> Result<(), CamtError> {
    let leaf = path.last().map(String::as_str).unwrap_or("");
    match leaf {
        "Amt" => {
            if ends_with(path, &["Ntry", "Amt"]) {
                ntry.amount = Some(
                    Decimal::from_str(txt)
                        .map_err(|_| CamtError::InvalidAmount(txt.to_string()))?,
                );
            }
        }
        "CdtDbtInd" => {
            if ends_with(path, &["Ntry", "CdtDbtInd"]) {
                ntry.sign = Some(txt.to_string());
            }
        }
        "Dt" | "DtTm" => {
            // Matchers anchored sur le tail complet (review code Pass 2
            // finding F14, alignement avec F7) : un futur `<XxxDt><Dt>`
            // imbriqué ailleurs dans le subtree Ntry n'écrasera pas
            // silencieusement booking_date / value_date.
            if ends_with(path, &["BookgDt", "Dt"]) || ends_with(path, &["BookgDt", "DtTm"]) {
                ntry.booking_date = Some(parse_date(txt)?);
            } else if ends_with(path, &["ValDt", "Dt"]) || ends_with(path, &["ValDt", "DtTm"]) {
                ntry.value_date = Some(parse_date(txt)?);
            }
        }
        "NtryRef" => {
            if ends_with(path, &["Ntry", "NtryRef"]) {
                ntry.reference = Some(txt.to_string());
            }
        }
        "AcctSvcrRef" => {
            if ends_with(path, &["Ntry", "AcctSvcrRef"]) {
                ntry.transaction_id = Some(txt.to_string());
            }
        }
        "AddtlNtryInf" if ends_with(path, &["Ntry", "AddtlNtryInf"]) => {
            ntry.addtl_ntry_inf = Some(txt.to_string());
        }
        _ => {}
    }
    Ok(())
}

fn handle_txdtls_text(t: &mut TxDtlsBuilder, path: &[String], txt: &str) -> Result<(), CamtError> {
    let leaf = path.last().map(String::as_str).unwrap_or("");
    match leaf {
        "EndToEndId" => {
            if ends_with(path, &["TxDtls", "Refs", "EndToEndId"]) {
                t.end_to_end_id = Some(txt.to_string());
            }
        }
        "AcctSvcrRef" => {
            if ends_with(path, &["TxDtls", "Refs", "AcctSvcrRef"]) {
                t.transaction_id = Some(txt.to_string());
            }
        }
        "Amt" => {
            if ends_with(path, &["TxDtls", "Amt"]) {
                t.amount = Some(
                    Decimal::from_str(txt)
                        .map_err(|_| CamtError::InvalidAmount(txt.to_string()))?,
                );
            }
        }
        "CdtDbtInd" => {
            if ends_with(path, &["TxDtls", "CdtDbtInd"]) {
                t.sign = Some(txt.to_string());
            }
        }
        "Ustrd" => {
            if ends_with(path, &["TxDtls", "RmtInf", "Ustrd"]) {
                t.ustrd_parts.push(txt.to_string());
            }
        }
        "Ref" => {
            if ends_with(path, &["TxDtls", "RmtInf", "Strd", "CdtrRefInf", "Ref"]) {
                t.cdtr_ref = Some(txt.to_string());
            }
        }
        "Nm" => {
            // v04 : RltdPties > (Cdtr|Dbtr) > Nm — parent in {Cdtr, Dbtr}.
            // v08 : RltdPties > (Cdtr|Dbtr) > Pty > Nm — parent = Pty,
            // grand-parent in {Cdtr, Dbtr}.
            //
            // Le check `in_rltd_pties` (review code Pass 2 finding F21)
            // garantit qu'on ne capture pas un `<Nm>` Cdtr/Dbtr d'un
            // autre subtree (ex. `<RltdAgts>` ou extensions vendor).
            let parent = path.iter().rev().nth(1).map(String::as_str);
            let grand = path.iter().rev().nth(2).map(String::as_str);
            let is_v04 = matches!(parent, Some("Cdtr") | Some("Dbtr"));
            let is_v08 = parent == Some("Pty") && matches!(grand, Some("Cdtr") | Some("Dbtr"));
            let in_rltd_pties = path.iter().any(|s| s == "RltdPties");
            if (is_v04 || is_v08) && in_rltd_pties {
                t.counterparty_name = Some(txt.to_string());
            }
        }
        "IBAN"
            if (ends_with(path, &["CdtrAcct", "Id", "IBAN"])
                || ends_with(path, &["DbtrAcct", "Id", "IBAN"])) =>
        {
            t.counterparty_iban = Some(txt.to_string());
        }
        _ => {}
    }
    Ok(())
}

/// Indicateur Crédit/Débit (`<CdtDbtInd>`) — interne au parseur.
///
/// CAMT.053 n'autorise que `CRDT` ou `DBIT`. Toute autre valeur est
/// strictement rejetée par `parse_sign` plutôt que silencieusement
/// traitée comme positif (qui inverserait silencieusement un débit en
/// crédit sur un fichier malformé — corruption comptable directe).
#[derive(Clone, Copy)]
enum CdtDbtSign {
    Credit,
    Debit,
}

impl CdtDbtSign {
    fn apply(self, amount: Decimal) -> Decimal {
        match self {
            Self::Credit => amount,
            Self::Debit => -amount,
        }
    }
}

fn parse_sign(raw: &str, location: &str) -> Result<CdtDbtSign, CamtError> {
    match raw {
        "CRDT" => Ok(CdtDbtSign::Credit),
        "DBIT" => Ok(CdtDbtSign::Debit),
        other => Err(CamtError::MalformedXml(format!(
            "{location}.CdtDbtInd inattendu : {other:?} (attendu CRDT ou DBIT)"
        ))),
    }
}

/// Applique un bloc `<Bal>` au `StmtBuilder`.
///
/// Sémantique stricte (issue de la review code Pass 1, finding F1+F2) :
///
/// - Si `code` (Tp/CdOrPrtry/Cd) absent → skip silencieux : on ne sait
///   pas comment classifier le solde (peut être une extension vendor ou
///   un code non-OPBD/CLBD comme PRCD, ITBD).
/// - Si `code` présent mais `amount` absent → erreur (CAMT.053 schéma
///   exige `<Amt>`).
/// - Si `code` présent mais `sign` (`CdtDbtInd`) absent → erreur (idem).
/// - Si `sign` présent mais valeur ≠ `CRDT`/`DBIT` → erreur explicite
///   (refus de traiter un sign inconnu comme positif silencieusement).
/// - Codes `OPBD` / `CLBD` → assigne `opening_balance` / `closing_balance`.
/// - Autres codes (PRCD, ITBD, OPAV, CLAV, etc.) → silencieusement
///   ignorés (informationnels, hors périmètre v0.1 + CR-010 #62).
fn apply_balance(
    stmt: &mut StmtBuilder,
    b: BalBuilder,
    stmt_index: usize,
) -> Result<(), CamtError> {
    let Some(code) = b.code.as_deref() else {
        return Ok(());
    };
    let amount = b.amount.ok_or_else(|| {
        CamtError::MissingRequiredField(format!("stmt[{stmt_index}].bal[{code}].amount"))
    })?;
    let sign_raw = b.sign.as_deref().ok_or_else(|| {
        CamtError::MissingRequiredField(format!("stmt[{stmt_index}].bal[{code}].cdt_dbt_ind"))
    })?;
    let sign = parse_sign(sign_raw, &format!("stmt[{stmt_index}].bal[{code}]"))?;
    let signed = sign.apply(amount);
    match code {
        "OPBD" => stmt.opening_balance = Some(signed),
        "CLBD" => stmt.closing_balance = Some(signed),
        _ => {}
    }
    Ok(())
}

fn emit_transactions(
    n: NtryBuilder,
    stmt_index: usize,
    ntry_index: usize,
) -> Result<Vec<ImportedTransaction>, CamtError> {
    let location = |suffix: &str| format!("stmt[{stmt_index}].ntry[{ntry_index}].{suffix}");

    let booking_date = n
        .booking_date
        .ok_or_else(|| CamtError::MissingRequiredField(location("booking_date")))?;
    let ntry_amount = n
        .amount
        .ok_or_else(|| CamtError::MissingRequiredField(location("amount")))?;
    // Le sign Ntry est optionnel ici : certains exporters bancaires
    // omettent `<CdtDbtInd>` au niveau `<Ntry>` quand chaque
    // `<TxDtls>` porte le sien. On ne lève l'erreur que si le sign
    // n'est ni présent au niveau Ntry ni au niveau TxDtls
    // correspondant — voir review code Pass 2 finding F13.
    let ntry_sign: Option<CdtDbtSign> = match n.sign.as_deref() {
        Some(raw) => Some(parse_sign(
            raw,
            &format!("stmt[{stmt_index}].ntry[{ntry_index}]"),
        )?),
        None => None,
    };
    let ntry_currency = n
        .currency
        .clone()
        .ok_or_else(|| CamtError::MissingRequiredField(location("currency")))?;

    let value_date = n.value_date;
    let ntry_reference = n.reference.clone();
    let ntry_transaction_id = n.transaction_id.clone();
    let ntry_details = n.addtl_ntry_inf.clone();

    if n.txdtls.is_empty() {
        // Pas de TxDtls : un sign au niveau Ntry est obligatoire
        // (sinon on ne sait pas comment signer le montant agrégé).
        let sign =
            ntry_sign.ok_or_else(|| CamtError::MissingRequiredField(location("cdt_dbt_ind")))?;
        return Ok(vec![ImportedTransaction {
            booking_date,
            value_date,
            amount: sign.apply(ntry_amount),
            currency: ntry_currency,
            reference: ntry_reference,
            details: ntry_details.unwrap_or_default(),
            end_to_end_id: None,
            transaction_id: ntry_transaction_id,
            counterparty_iban: None,
            counterparty_name: None,
        }]);
    }

    let mut out = Vec::with_capacity(n.txdtls.len());
    for (i, t) in n.txdtls.into_iter().enumerate() {
        let amount_raw = t.amount.unwrap_or(ntry_amount);
        let txdtls_loc = format!("stmt[{stmt_index}].ntry[{ntry_index}].txdtls[{i}]");
        // Pour chaque TxDtls : sign propre si présent, sinon
        // fallback Ntry, sinon erreur (ni TxDtls ni Ntry n'a de
        // signe — Ntry et au moins un TxDtls défaillants).
        let sign = if let Some(s) = t.sign.as_deref() {
            parse_sign(s, &txdtls_loc)?
        } else {
            ntry_sign.ok_or_else(|| {
                CamtError::MissingRequiredField(format!(
                    "{txdtls_loc}.cdt_dbt_ind (ni TxDtls ni Ntry n'a de signe)"
                ))
            })?
        };
        let signed = sign.apply(amount_raw);
        let currency = t.currency.unwrap_or_else(|| ntry_currency.clone());
        let reference = t.cdtr_ref.or_else(|| ntry_reference.clone());
        let transaction_id = t.transaction_id.or_else(|| ntry_transaction_id.clone());
        let details = if t.ustrd_parts.is_empty() {
            ntry_details.clone().unwrap_or_default()
        } else {
            t.ustrd_parts.join(" ")
        };
        out.push(ImportedTransaction {
            booking_date,
            value_date,
            amount: signed,
            currency,
            reference,
            details,
            end_to_end_id: t.end_to_end_id,
            transaction_id,
            counterparty_iban: t.counterparty_iban,
            counterparty_name: t.counterparty_name,
        });
    }
    Ok(out)
}

fn finalize_statement(
    b: StmtBuilder,
    version: &str,
    stmt_index: usize,
) -> Result<ImportedStatement, CamtError> {
    let location = |suffix: &str| format!("stmt[{stmt_index}].{suffix}");
    let account_iban = b
        .account_iban
        .ok_or_else(|| CamtError::MissingRequiredField(location("account_iban")))?;
    let currency = b
        .currency
        .ok_or_else(|| CamtError::MissingRequiredField(location("currency")))?;
    let period_from = b
        .period_from
        .ok_or_else(|| CamtError::MissingRequiredField(location("period_from")))?;
    let period_to = b
        .period_to
        .ok_or_else(|| CamtError::MissingRequiredField(location("period_to")))?;

    Ok(ImportedStatement {
        statement_id: b.statement_id,
        account_iban,
        currency,
        opening_balance: b.opening_balance,
        closing_balance: b.closing_balance,
        period_from,
        period_to,
        transactions: b.transactions,
        source_format: SourceFormat::Camt053 {
            version: version.to_string(),
        },
    })
}

fn local_name(bytes: &[u8]) -> Result<String, CamtError> {
    std::str::from_utf8(bytes)
        .map(String::from)
        .map_err(|e| CamtError::MalformedXml(format!("nom de tag non UTF-8 : {e}")))
}

fn attr_string(start: &BytesStart, name: &[u8]) -> Result<Option<String>, CamtError> {
    for attr in start.attributes() {
        match attr {
            Ok(attr) => {
                if attr.key.local_name().as_ref() == name {
                    let value = attr
                        .unescape_value()
                        .map_err(|e| CamtError::MalformedXml(e.to_string()))?
                        .into_owned();
                    return Ok(Some(value));
                }
            }
            Err(e) => return Err(CamtError::MalformedXml(e.to_string())),
        }
    }
    Ok(None)
}

fn ends_with(path: &[String], suffix: &[&str]) -> bool {
    if path.len() < suffix.len() {
        return false;
    }
    let start = path.len() - suffix.len();
    path[start..]
        .iter()
        .zip(suffix.iter())
        .all(|(a, b)| a.as_str() == *b)
}

fn parse_date(s: &str) -> Result<NaiveDate, CamtError> {
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Ok(d);
    }
    if let Some((date_part, _)) = s.split_once('T')
        && let Ok(d) = NaiveDate::parse_from_str(date_part, "%Y-%m-%d")
    {
        return Ok(d);
    }
    Err(CamtError::InvalidDate(s.to_string()))
}
