//! Story 33-4 (T5, AC-9, refs #606) — la garde des dates reçues du client.
//!
//! **Propriété** : tout champ daté (`NaiveDate`, `NaiveDateTime`, `DateTime<…>`)
//! d'un item `Deserialize` de `src/routes`, `src/services` et `src/mcp` est
//! borné : il porte `dates_recues::` en `deserialize_with`, ou figure à la liste
//! [`EXCEPTIONS`] avec son motif. Un `Option` qui porte `dates_recues::` porte
//! aussi `default`, sans quoi le champ facultatif devient requis. Une exception
//! sans cible fait rougir, pour que la liste ne se périme pas en silence ; une
//! exception de classe B exige un `borner(…, "<champ>")` dans le même fichier ;
//! une classe A, le test qui prouve son refus.
//!
//! ⚠️ **Ce que cette garde ne voit pas** : une chaîne de date analysée à la main
//! (`asOf`, `dateFrom` de l'audit et des écritures : éprouvées par leurs tests),
//! un type daté caché derrière un alias, un `borner` posé après l'usage de la
//! date. La revue les lit.

use std::path::{Path, PathBuf};

use regex::Regex;

/// Un champ daté d'un item `Deserialize`.
#[derive(Debug, Clone, PartialEq)]
struct Champ {
    fichier: String,
    item: String,
    champ: String,
    borne: bool,
    defaut: bool,
    optionnel: bool,
}

/// Pourquoi un champ daté ne porte pas `dates_recues::`.
#[derive(Clone, Copy, Debug)]
enum Motif {
    /// Déjà refusé en 4xx partout, relevé par ce test (fichier, nom).
    ClasseA(&'static str, &'static str),
    /// Contrôle dans le handler, 400 JSON : `borner(…, "<champ>")`.
    ClasseB,
    /// Passé en chaîne à un service qui le borne ; le refus reste le sien.
    ParLeService,
    /// `DateTime<Utc>` borné par `expiration_stockable` dans le même fichier.
    Expiration,
    /// Un type de réponse qui dérive aussi `Deserialize` : rien n'est reçu.
    Reponse,
}

use Motif::{ClasseA, ClasseB, Expiration, ParLeService, Reponse};

const E2E: &str = "dates_bornes_e2e.rs";

/// (fichier, item, champ, motif).
const EXCEPTIONS: &[(&str, &str, &str, Motif)] = &[
    // Classe A : la période sort de l'exercice, `REPORT_PERIOD_OUT_OF_FISCAL_YEAR`.
    (
        "routes/reports.rs",
        "ReportQuery",
        "period_start",
        ClasseA(E2E, "q_bilan_debut"),
    ),
    (
        "routes/reports.rs",
        "ReportQuery",
        "period_end",
        ClasseA(E2E, "q_bilan_fin"),
    ),
    (
        "routes/reports.rs",
        "JournalReportQuery",
        "period_start",
        ClasseA(E2E, "q_journaux_debut"),
    ),
    (
        "routes/reports.rs",
        "JournalReportQuery",
        "period_end",
        ClasseA(E2E, "q_journaux_fin"),
    ),
    (
        "routes/reports.rs",
        "ExportQuery",
        "period_start",
        ClasseA(E2E, "q_export_debut"),
    ),
    (
        "routes/reports.rs",
        "ExportQuery",
        "period_end",
        ClasseA(E2E, "q_export_fin"),
    ),
    // Classe A : `DATE_OUTSIDE_FISCAL_YEAR` sur l'exercice de l'écriture.
    (
        "routes/journal_entries.rs",
        "UpdateJournalEntryRequest",
        "entry_date",
        ClasseA(E2E, "c_ecriture_maj_date"),
    ),
    // Classe A d'office : contrats antérieurs à la story, qui ne bougent pas.
    (
        "routes/bank_accounts.rs",
        "CloseBankAccountBody",
        "closed_on",
        ClasseA("bank_accounts_closed_e2e.rs", "+10000"),
    ),
    (
        "routes/mcp_keys.rs",
        "CreateMcpKeyRequest",
        "expires_at",
        ClasseA("mcp_admin_routes_e2e.rs", "c3_an_expiry_beyond_year_9999"),
    ),
    // Classe B : un refus JSON existait, une autre valeur passait (relevé T1).
    (
        "routes/companies.rs",
        "LockBooksRequest",
        "through",
        ClasseB,
    ),
    (
        "routes/companies.rs",
        "UnlockBooksRequest",
        "through",
        ClasseB,
    ),
    (
        "routes/credit_notes.rs",
        "CreateCreditNoteRequest",
        "date",
        ClasseB,
    ),
    (
        "routes/dunning_reminders.rs",
        "ManualReminderBody",
        "sent_at",
        ClasseB,
    ),
    (
        "routes/fiscal_years.rs",
        "CreateFiscalYearRequest",
        "start_date",
        ClasseB,
    ),
    (
        "routes/fiscal_years.rs",
        "CreateFiscalYearRequest",
        "end_date",
        ClasseB,
    ),
    (
        "routes/invoices.rs",
        "CreateInvoiceRequest",
        "date",
        ClasseB,
    ),
    (
        "routes/invoices.rs",
        "CreateInvoiceRequest",
        "due_date",
        ClasseB,
    ),
    (
        "routes/invoices.rs",
        "UpdateInvoiceRequest",
        "date",
        ClasseB,
    ),
    (
        "routes/invoices.rs",
        "UpdateInvoiceRequest",
        "due_date",
        ClasseB,
    ),
    (
        "routes/invoices.rs",
        "SettleInvoiceRequest",
        "settled_on",
        ClasseB,
    ),
    (
        "routes/invoices.rs",
        "WriteOffInvoiceRequest",
        "settled_on",
        ClasseB,
    ),
    (
        "routes/journal_entries.rs",
        "CreateJournalEntryRequest",
        "entry_date",
        ClasseB,
    ),
    (
        "routes/payment_batches.rs",
        "ConfirmPaymentBatchRequest",
        "payment_date",
        ClasseB,
    ),
    (
        "routes/reconciliation.rs",
        "ManualMatchBody",
        "value_date",
        ClasseB,
    ),
    (
        "routes/reconciliation.rs",
        "SplitBody",
        "value_date",
        ClasseB,
    ),
    ("routes/reports.rs", "GeneralLedgerQuery", "from", ClasseB),
    ("routes/reports.rs", "GeneralLedgerQuery", "to", ClasseB),
    (
        "routes/reports.rs",
        "GeneralLedgerExportQuery",
        "from",
        ClasseB,
    ),
    (
        "routes/reports.rs",
        "GeneralLedgerExportQuery",
        "to",
        ClasseB,
    ),
    (
        "routes/supplier_invoices.rs",
        "CreateSupplierInvoiceRequest",
        "invoice_date",
        ClasseB,
    ),
    (
        "routes/supplier_invoices.rs",
        "CreateSupplierInvoiceRequest",
        "due_date",
        ClasseB,
    ),
    (
        "routes/supplier_invoices.rs",
        "PaySupplierInvoiceRequest",
        "payment_date",
        ClasseB,
    ),
    ("routes/vat.rs", "CreateVatRateBody", "valid_from", ClasseB),
    ("routes/vat.rs", "CreateVatRateBody", "valid_to", ClasseB),
    ("routes/vat.rs", "UpdateVatRateBody", "valid_to", ClasseB),
    // MCP, par le service : `analyser_date`, `parse_as_of`.
    (
        "mcp/outils/ecritures.rs",
        "ArgumentsListe",
        "date_from",
        ParLeService,
    ),
    (
        "mcp/outils/ecritures.rs",
        "ArgumentsListe",
        "date_to",
        ParLeService,
    ),
    (
        "mcp/outils/postes_ouverts.rs",
        "Arguments",
        "as_of",
        ParLeService,
    ),
    // Expiration d'une clé d'API (#755).
    (
        "routes/api_keys.rs",
        "CreateApiKeyRequest",
        "expires_at",
        Expiration,
    ),
    // Faux positifs : des réponses qui dérivent aussi `Deserialize`.
    (
        "routes/bank_profiles.rs",
        "BankProfileResponse",
        "created_at",
        Reponse,
    ),
    (
        "routes/bank_profiles.rs",
        "BankProfileResponse",
        "updated_at",
        Reponse,
    ),
    (
        "routes/reconciliation_rules.rs",
        "ReconciliationRuleResponse",
        "last_applied_at",
        Reponse,
    ),
    (
        "routes/reconciliation_rules.rs",
        "ReconciliationRuleResponse",
        "created_at",
        Reponse,
    ),
    (
        "routes/reconciliation_rules.rs",
        "ReconciliationRuleResponse",
        "updated_at",
        Reponse,
    ),
];

// ---------------------------------------------------------------------------
// La lecture
// ---------------------------------------------------------------------------

fn est_date(type_: &str) -> bool {
    Regex::new(r"\b(NaiveDate|NaiveDateTime|DateTime<)")
        .unwrap()
        .is_match(type_)
}

/// Les champs datés des items `Deserialize` d'un source. Hors des tests : le
/// texte s'arrête au `#[cfg(test)]` de fin de fichier.
fn champs_dates(fichier: &str, source: &str) -> Vec<Champ> {
    let source = source.split("#[cfg(test)]\nmod ").next().unwrap();
    let debut_item = Regex::new(r"^\s*(?:pub(?:\([a-z]+\))?\s+)?(?:struct|enum)\s+(\w+)").unwrap();
    let champ = Regex::new(r"^\s*(?:pub(?:\([a-z]+\))?\s+)?(\w+):\s*(.+?),?\s*$").unwrap();
    let mut sortie = Vec::new();
    let mut derive = String::new();
    let mut dans_derive = false;
    let mut deserialize = false;
    let mut item: Option<String> = None;
    let mut profondeur = 0i32;
    let mut attributs = String::new();
    for ligne in source.lines() {
        let t = ligne.trim();
        if t.starts_with("//") {
            continue;
        }
        if item.is_none() {
            if t.starts_with("#[derive(") || dans_derive {
                derive.push_str(t);
                dans_derive = !t.contains(")]");
                if !dans_derive {
                    deserialize = derive.contains("Deserialize");
                    derive.clear();
                }
                continue;
            }
            if let Some(c) = debut_item.captures(ligne) {
                let ouvre = t.matches('{').count() as i32 - t.matches('}').count() as i32;
                if deserialize && ouvre > 0 {
                    item = Some(c[1].to_string());
                    profondeur = ouvre;
                    attributs.clear();
                }
                deserialize = false;
            }
            continue;
        }
        if t.starts_with("#[") || !attributs.is_empty() && !attributs.ends_with(")]") {
            attributs.push_str(t);
            continue;
        }
        if let Some(c) = champ.captures(ligne)
            && est_date(&c[2])
        {
            sortie.push(Champ {
                fichier: fichier.to_string(),
                item: item.clone().unwrap(),
                champ: c[1].to_string(),
                borne: attributs.contains("dates_recues::"),
                defaut: attributs.contains("default"),
                optionnel: c[2].starts_with("Option<"),
            });
        }
        attributs.clear();
        profondeur += t.matches('{').count() as i32 - t.matches('}').count() as i32;
        if profondeur <= 0 {
            item = None;
        }
    }
    sortie
}

fn chamelle(champ: &str) -> String {
    let mut s = String::new();
    let mut haut = false;
    for c in champ.chars() {
        if c == '_' {
            haut = true;
        } else if haut {
            s.extend(c.to_uppercase());
            haut = false;
        } else {
            s.push(c);
        }
    }
    s
}

/// Les fautes d'un inventaire : `source_de` rend le texte d'un fichier.
fn fautes(
    champs: &[Champ],
    exceptions: &[(&str, &str, &str, Motif)],
    source_de: &dyn Fn(&str) -> String,
    test_de: &dyn Fn(&str) -> String,
) -> Vec<String> {
    let mut f = Vec::new();
    for c in champs {
        let lieu = format!("{} {}::{}", c.fichier, c.item, c.champ);
        let exception = exceptions
            .iter()
            .find(|e| e.0 == c.fichier && e.1 == c.item && e.2 == c.champ);
        match (c.borne, exception) {
            (true, Some(_)) => f.push(format!("{lieu} : borné ET en exception")),
            (true, None) if c.optionnel && !c.defaut => {
                f.push(format!("{lieu} : `date_opt` sans `default`"));
            }
            (false, None) => f.push(format!("{lieu} : date reçue non bornée")),
            _ => {}
        }
    }
    for e in exceptions {
        let lieu = format!("{} {}::{}", e.0, e.1, e.2);
        if !champs
            .iter()
            .any(|c| c.fichier == e.0 && c.item == e.1 && c.champ == e.2)
        {
            f.push(format!("{lieu} : exception sans cible"));
            continue;
        }
        match e.3 {
            ClasseB => {
                let motif = format!(r#"borner(?:_opt)?\([^;]*?,\s*"{}"\s*\)"#, chamelle(e.2));
                if !Regex::new(&motif).unwrap().is_match(&source_de(e.0)) {
                    f.push(format!(
                        "{lieu} : classe B sans `borner(…, \"{}\")`",
                        chamelle(e.2)
                    ));
                }
            }
            ClasseA(fichier_test, preuve) => {
                if !test_de(fichier_test).contains(preuve) {
                    f.push(format!(
                        "{lieu} : classe A, preuve `{preuve}` absente de {fichier_test}"
                    ));
                }
            }
            Expiration => {
                if !source_de(e.0).contains("expiration_stockable(") {
                    f.push(format!("{lieu} : sans `expiration_stockable`"));
                }
            }
            ParLeService | Reponse => {}
        }
    }
    f
}

fn racine() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fichiers(dir: &Path, v: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).expect("répertoire lisible") {
        let p = e.unwrap().path();
        if p.is_dir() {
            fichiers(&p, v);
        } else if p.extension().is_some_and(|x| x == "rs") {
            v.push(p);
        }
    }
}

fn inventaire() -> Vec<Champ> {
    let src = racine().join("src");
    let mut v = Vec::new();
    for d in ["routes", "services", "mcp"] {
        fichiers(&src.join(d), &mut v);
    }
    v.sort();
    let mut champs = Vec::new();
    for p in v {
        let relatif = p.strip_prefix(&src).unwrap().to_string_lossy().to_string();
        champs.extend(champs_dates(
            &relatif,
            &std::fs::read_to_string(&p).unwrap(),
        ));
    }
    champs
}

fn source_reelle(f: &str) -> String {
    std::fs::read_to_string(racine().join("src").join(f)).unwrap()
}

fn test_reel(f: &str) -> String {
    std::fs::read_to_string(racine().join("tests").join(f)).unwrap()
}

// ---------------------------------------------------------------------------
// La garde
// ---------------------------------------------------------------------------

#[test]
fn aucune_date_recue_n_est_sans_borne() {
    let champs = inventaire();
    let f = fautes(&champs, EXCEPTIONS, &source_reelle, &test_reel);
    assert!(f.is_empty(), "{}", f.join("\n"));
}

/// Le décompte, à recompter si un champ s'ajoute : un inventaire qui se viderait
/// passerait la garde à vide. 63 champs typés reçus (17 requêtes, 29 + 1 corps,
/// 2 expirations, 14 MCP), plus cinq faux positifs ; les chaînes de date ne sont
/// pas ici (`asOf`, `dateFrom`, `dateTo` : voir l'en-tête).
#[test]
fn l_inventaire_n_est_pas_vide() {
    let champs = inventaire();
    assert_eq!(champs.len(), COMPTE_CHAMPS, "{champs:#?}");
    let bornes = champs.iter().filter(|c| c.borne).count();
    assert_eq!(bornes + EXCEPTIONS.len(), champs.len());
}

const COMPTE_CHAMPS: usize = 68;

// ---------------------------------------------------------------------------
// Les sondes : un contrôle vide se prouve par une sonde
// ---------------------------------------------------------------------------

const EXTRAIT: &str = r#"
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sonde {
    pub nom: String,
    pub sans_borne: NaiveDate,
    #[serde(deserialize_with = "crate::dates_recues::date")]
    pub bornee: NaiveDate,
    #[serde(deserialize_with = "crate::dates_recues::date_opt")]
    pub sans_defaut: Option<NaiveDate>,
    #[serde(default, deserialize_with = "crate::dates_recues::date_opt")]
    pub complete: Option<chrono::NaiveDate>,
    pub instant: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct Reponse {
    pub ignoree: NaiveDate,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Variante {
    #[serde(rename = "a")]
    A {
        id: i64,
        #[serde(default)]
        jour: Option<NaiveDate>,
    },
    B { id: i64 },
}
"#;

fn extrait() -> Vec<Champ> {
    champs_dates("sonde.rs", EXTRAIT)
}

fn vide(_: &str) -> String {
    String::new()
}

#[test]
fn sonde_les_champs_sont_tous_lus() {
    let noms: Vec<_> = extrait().into_iter().map(|c| c.champ).collect();
    assert_eq!(
        noms,
        [
            "sans_borne",
            "bornee",
            "sans_defaut",
            "complete",
            "instant",
            "jour"
        ],
        "`Serialize` seul est ignoré ; la variante d'enum est lue"
    );
}

#[test]
fn sonde_un_champ_sans_borne_est_releve() {
    let f = fautes(&extrait(), &[], &vide, &vide);
    for nom in ["sans_borne", "instant", "jour"] {
        assert!(
            f.iter()
                .any(|x| x.contains(nom) && x.contains("non bornée")),
            "{nom} : {f:?}"
        );
    }
    assert!(
        !f.iter()
            .any(|x| x.contains("::bornee") || x.contains("::complete")),
        "{f:?}"
    );
}

#[test]
fn sonde_un_date_opt_sans_default_est_releve() {
    let f = fautes(&extrait(), &[], &vide, &vide);
    assert!(
        f.iter()
            .any(|x| x.contains("sans_defaut") && x.contains("sans `default`")),
        "{f:?}"
    );
}

#[test]
fn sonde_une_exception_sans_cible_rougit() {
    let exceptions = [("sonde.rs", "Sonde", "disparu", ClasseB)];
    let f = fautes(&extrait(), &exceptions, &vide, &vide);
    assert!(
        f.iter()
            .any(|x| x.contains("disparu") && x.contains("sans cible")),
        "{f:?}"
    );
}

#[test]
fn sonde_une_classe_b_exige_son_borner() {
    let exceptions = [("sonde.rs", "Sonde", "sans_borne", ClasseB)];
    let sans = fautes(&extrait(), &exceptions, &vide, &vide);
    assert!(sans.iter().any(|x| x.contains("classe B sans")), "{sans:?}");
    let source =
        |_: &str| "borner(\n    req.sans_borne,\n    \"sansBorne\",\n)".replace(",\n)", "\n)");
    let avec = fautes(&extrait(), &exceptions, &source, &vide);
    assert!(
        !avec.iter().any(|x| x.contains("classe B sans")),
        "{avec:?}"
    );
    // Un autre champ du même fichier ne vaut pas preuve.
    let autre = |_: &str| "borner(req.autre, \"autre\")".to_string();
    let f = fautes(&extrait(), &exceptions, &autre, &vide);
    assert!(f.iter().any(|x| x.contains("classe B sans")), "{f:?}");
}

#[test]
fn sonde_une_classe_a_exige_sa_preuve() {
    let exceptions = [(
        "sonde.rs",
        "Sonde",
        "sans_borne",
        ClasseA("t.rs", "le_test"),
    )];
    let f = fautes(&extrait(), &exceptions, &vide, &vide);
    assert!(f.iter().any(|x| x.contains("preuve")), "{f:?}");
    let t = |_: &str| "async fn le_test()".to_string();
    let f = fautes(&extrait(), &exceptions, &vide, &t);
    assert!(!f.iter().any(|x| x.contains("preuve")), "{f:?}");
}
