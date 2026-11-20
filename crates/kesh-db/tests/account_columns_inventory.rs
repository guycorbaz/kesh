//! Story 37-3a (AC-5) — inventaire **clos** des colonnes qui désignent un
//! compte (`*_account_id`), lu au schéma de test
//! (`test-schema/0001_schema_squash.sql`) ; revue de code P1.
//!
//! Méthode « inventorier les sites non résolus » (`CLAUDE.md`) : plutôt que
//! d'énumérer les colonnes qu'on croit gardées, le test extrait **toutes** les
//! colonnes `<table>.<colonne>` dont le nom finit par `_account_id` et exige que
//! l'ensemble extrait égale, dans les deux sens, la liste écrite en dur
//! ci-dessous. Une colonne ajoutée au schéma sans y être classée fait rougir le
//! test en la nommant ; une colonne retirée aussi.
//!
//! Chaque entrée porte sa **preuve** :
//!
//! - les tests (`crate/tests/fichier.rs::fonction`) qui prouvent le refus d'un
//!   compte en devise sur cette colonne (Story 37-1 pour les configurations,
//!   Story 37-3a pour les pièces) — le test vérifie que chaque fonction existe
//!   dans le fichier nommé ;
//! - ou, pour une colonne qui ne désigne pas un compte du plan comptable,
//!   l'affirmation « clé étrangère vers `bank_accounts` », que le test vérifie
//!   au schéma (contrainte `FOREIGN KEY … REFERENCES bank_accounts`).
//!
//! Test pur, sans base.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Ce qui établit qu'une colonne ne peut pas désigner un compte en devise.
#[derive(Debug)]
enum Preuve {
    /// Tests de refus, `crate/tests/fichier.rs::fonction` (chemins relatifs à
    /// `crates/`).
    Tests(&'static [&'static str]),
    /// La colonne référence `bank_accounts.id`, non `accounts.id` : le compte
    /// comptable d'un compte bancaire est `bank_accounts.journal_account_id`,
    /// inventorié à part. Vérifié au schéma.
    VersBankAccounts,
}

/// Une colonne de la liste close, sa preuve et une note libre (ce que les
/// tests couvrent, ou ne couvrent pas, nommément).
struct Colonne {
    colonne: &'static str,
    preuve: Preuve,
    note: &'static str,
}

/// La liste **close**. Toute modification du schéma qui ajoute ou retire une
/// colonne `*_account_id` doit être reportée ici, avec sa preuve.
const INVENTAIRE: &[Colonne] = &[
    // --- Comptes propres d'un compte en devise (37-1 AC-4 bis) -------------
    Colonne {
        colonne: "accounts.fx_gain_account_id",
        preuve: Preuve::Tests(&[
            "kesh-db/tests/account_currency_repository.rs::comptes_propres_chaque_raison",
            "kesh-db/tests/account_currency_repository.rs::compte_de_difference_propre_garde_avec_son_detenteur",
        ]),
        note: "En CHF par construction (37-1) : désigner un compte en devise → \
               FX_ACCOUNT_INVALID/foreignCurrency (non ACCOUNT_IN_FOREIGN_CURRENCY) ; \
               passer le compte désigné en devise → refusé, détenteur nommé.",
    },
    Colonne {
        colonne: "accounts.fx_loss_account_id",
        preuve: Preuve::Tests(&[
            "kesh-db/tests/account_currency_repository.rs::comptes_propres_chaque_raison",
            "kesh-db/tests/account_currency_repository.rs::compte_de_difference_propre_garde_avec_son_detenteur",
        ]),
        note: "Même contrôle que fx_gain_account_id (check_fx_account, tous champs).",
    },
    // --- Compte bancaire -------------------------------------------------
    Colonne {
        colonne: "bank_accounts.journal_account_id",
        preuve: Preuve::Tests(&[
            "kesh-api/tests/account_currency_e2e.rs::un_compte_en_devise_n_est_designable_nulle_part",
            "kesh-db/tests/account_currency_repository.rs::devise_refusee_a_chaque_site_de_configuration",
        ]),
        note: "Lier un compte en devise → 400 ACCOUNT_IN_FOREIGN_CURRENCY (37-1) ; \
               passer un compte lié en devise → ACCOUNT_CURRENCY_IN_USE. Exception \
               voulue (38-11b, levée de C-37-9) : le compte du plan d'un compte PayPal \
               est admis en devise, c'est la devise du solde.",
    },
    // Story 38-11b — le compte de frais d'un compte PayPal : en CHF ou dans la
    // devise du compte lié (une écriture ne porte qu'une devise étrangère),
    // toute autre devise refusée.
    Colonne {
        colonne: "bank_accounts.fee_account_id",
        preuve: Preuve::Tests(&[
            "kesh-api/tests/paypal_import_e2e.rs::creer_un_solde_paypal_et_ses_refus",
        ]),
        note: "Compte de frais d'une autre devise que CHF et celle du solde → 400 \
               PROVIDER_FEE_ACCOUNT_INVALID (reason currency) ; dans la devise du \
               solde, admis par conception.",
    },
    Colonne {
        colonne: "bank_accounts.payout_bank_account_id",
        preuve: Preuve::VersBankAccounts,
        note: "Story 40-7b — désigne un compte bancaire (la destination des versements \
               Stripe), non un compte du plan comptable.",
    },
    Colonne {
        colonne: "bank_imports.bank_account_id",
        preuve: Preuve::VersBankAccounts,
        note: "Désigne un compte bancaire, non un compte du plan comptable.",
    },
    Colonne {
        colonne: "bank_transactions.bank_account_id",
        preuve: Preuve::VersBankAccounts,
        note: "Désigne un compte bancaire, non un compte du plan comptable.",
    },
    Colonne {
        colonne: "payment_batches.bank_account_id",
        preuve: Preuve::VersBankAccounts,
        note: "Désigne un compte bancaire, non un compte du plan comptable.",
    },
    Colonne {
        colonne: "invoice_settlements.settlement_bank_account_id",
        preuve: Preuve::VersBankAccounts,
        note: "Règlement par virement : la contrepartie est journal_account_id du compte bancaire.",
    },
    Colonne {
        colonne: "supplier_invoices.settlement_bank_account_id",
        preuve: Preuve::VersBankAccounts,
        note: "Règlement par virement : la contrepartie est journal_account_id du compte bancaire.",
    },
    // --- Comptes de différence de la société (37-1) -----------------------
    Colonne {
        colonne: "companies.fx_gain_account_id",
        preuve: Preuve::Tests(&[
            "kesh-db/tests/currency_settings_repository.rs::le_controle_partage_rend_chaque_raison",
            "kesh-api/tests/currency_settings_e2e.rs::chaque_raison_est_rendue_par_la_route",
            "kesh-db/tests/account_currency_repository.rs::compte_de_difference_de_la_societe_garde",
        ]),
        note: "En CHF par construction (37-1) : FX_ACCOUNT_INVALID/foreignCurrency ; \
               passer le compte désigné en devise → refusé.",
    },
    Colonne {
        colonne: "companies.fx_loss_account_id",
        preuve: Preuve::Tests(&[
            "kesh-db/tests/account_currency_repository.rs::compte_de_difference_de_la_societe_garde",
        ]),
        note: "Même contrôle que fx_gain_account_id (check_fx_account, tous champs) ; \
               la désignation directe d'un compte en devise n'est exercée nommément \
               que sur le champ des gains.",
    },
    Colonne {
        colonne: "companies.fx_transit_account_id",
        preuve: Preuve::Tests(&[
            "kesh-db/tests/account_currency_repository.rs::compte_de_difference_de_la_societe_garde",
        ]),
        note: "Passer le compte de conversion en devise → refusé ; la désignation \
               directe d'un compte en devise n'est exercée nommément que sur le champ \
               des gains (même contrôle check_fx_account).",
    },
    // --- Réglages de facturation (37-1) ----------------------------------
    Colonne {
        colonne: "company_invoice_settings.default_bad_debt_account_id",
        preuve: Preuve::Tests(&[
            "kesh-db/tests/account_currency_repository.rs::reglages_de_facturation_gardes_inverses",
        ]),
        note: NOTE_REGLAGES,
    },
    Colonne {
        colonne: "company_invoice_settings.default_bank_fees_account_id",
        preuve: Preuve::Tests(&[
            "kesh-db/tests/account_currency_repository.rs::reglages_de_facturation_gardes_inverses",
        ]),
        note: NOTE_REGLAGES,
    },
    Colonne {
        colonne: "company_invoice_settings.default_discount_account_id",
        preuve: Preuve::Tests(&[
            "kesh-db/tests/account_currency_repository.rs::reglages_de_facturation_gardes_inverses",
        ]),
        note: NOTE_REGLAGES,
    },
    Colonne {
        colonne: "company_invoice_settings.default_payable_account_id",
        preuve: Preuve::Tests(&[
            "kesh-db/tests/account_currency_repository.rs::reglages_de_facturation_gardes_inverses",
        ]),
        note: NOTE_REGLAGES,
    },
    Colonne {
        colonne: "company_invoice_settings.default_receivable_account_id",
        preuve: Preuve::Tests(&[
            "kesh-db/tests/account_currency_repository.rs::reglages_de_facturation_gardes_inverses",
        ]),
        note: NOTE_REGLAGES,
    },
    Colonne {
        colonne: "company_invoice_settings.default_revenue_account_id",
        preuve: Preuve::Tests(&[
            "kesh-db/tests/account_currency_repository.rs::reglages_de_facturation_gardes_inverses",
            "kesh-db/tests/account_currency_repository.rs::devise_refusee_a_chaque_site_de_configuration",
        ]),
        note: "Sens inverse exercé nommément (InvoiceSettings(\"revenue\")) ; sens direct : \
               même boucle designated_changes que les neuf autres.",
    },
    Colonne {
        colonne: "company_invoice_settings.default_rounding_account_id",
        preuve: Preuve::Tests(&[
            "kesh-db/tests/account_currency_repository.rs::reglages_de_facturation_gardes_inverses",
        ]),
        note: "Seule colonne des réglages dont le refus direct est exercé nommément.",
    },
    Colonne {
        colonne: "company_invoice_settings.default_vat_decompte_account_id",
        preuve: Preuve::Tests(&[
            "kesh-db/tests/account_currency_repository.rs::reglages_de_facturation_gardes_inverses",
        ]),
        note: NOTE_REGLAGES,
    },
    Colonne {
        colonne: "company_invoice_settings.default_vat_payable_account_id",
        preuve: Preuve::Tests(&[
            "kesh-db/tests/account_currency_repository.rs::reglages_de_facturation_gardes_inverses",
        ]),
        note: NOTE_REGLAGES,
    },
    Colonne {
        colonne: "company_invoice_settings.default_vat_recoverable_account_id",
        preuve: Preuve::Tests(&[
            "kesh-db/tests/account_currency_repository.rs::reglages_de_facturation_gardes_inverses",
        ]),
        note: NOTE_REGLAGES,
    },
    // --- Article, règle de réconciliation (37-1) -------------------------
    Colonne {
        colonne: "products.default_revenue_account_id",
        preuve: Preuve::Tests(&[
            "kesh-db/tests/account_currency_repository.rs::articles_gardes_inverses",
            "kesh-api/tests/account_currency_e2e.rs::un_compte_en_devise_n_est_designable_nulle_part",
        ]),
        note: "400 ACCOUNT_IN_FOREIGN_CURRENCY (37-1).",
    },
    Colonne {
        colonne: "reconciliation_rules.counterparty_account_id",
        preuve: Preuve::Tests(&[
            "kesh-db/tests/account_currency_repository.rs::regles_de_reconciliation_refusent_un_compte_en_devise",
            "kesh-api/tests/account_currency_e2e.rs::un_compte_en_devise_n_est_designable_nulle_part",
        ]),
        note: "400 ACCOUNT_IN_FOREIGN_CURRENCY (37-1).",
    },
    // --- Colonnes de pièce (37-3a, AC-5) ---------------------------------
    Colonne {
        colonne: "invoice_lines.revenue_account_id",
        preuve: Preuve::Tests(&[
            "kesh-db/tests/invoices_line_revenue_account.rs::create_rejects_foreign_currency_revenue_account",
            "kesh-db/tests/invoices_line_revenue_account.rs::validate_rejects_revenue_account_made_foreign_currency_after_draft",
        ]),
        note: "validate_line_revenue_accounts_in_tx : création et validation (le PUT \
               passe par la même fonction, non exercé nommément).",
    },
    Colonne {
        colonne: "credit_note_lines.revenue_account_id",
        preuve: Preuve::Tests(&[
            "kesh-db/tests/credit_notes_repository.rs::credit_note_refused_when_revenue_account_in_foreign_currency",
        ]),
        note: "create_credit_note.",
    },
    Colonne {
        colonne: "supplier_invoice_lines.expense_account_id",
        preuve: Preuve::Tests(&[
            "kesh-db/tests/supplier_invoices_draft.rs::a_draft_refuses_a_foreign_currency_expense_account",
            "kesh-db/tests/supplier_invoices_draft.rs::validation_refuses_an_expense_account_made_foreign_currency_since_the_draft",
        ]),
        note: "check_draft_input_in_tx (création, mise à jour) et validate_in_tx.",
    },
    Colonne {
        colonne: "supplier_invoices.settlement_account_id",
        preuve: Preuve::Tests(&[
            "kesh-db/tests/supplier_invoices_repository.rs::pay_with_foreign_currency_internal_account_is_rejected",
        ]),
        note: "pay_in_tx, SettlementChoice::InternalAccount.",
    },
    Colonne {
        colonne: "invoice_settlements.settlement_account_id",
        preuve: Preuve::Tests(&[
            "kesh-db/tests/invoice_settlement.rs::un_compte_interne_en_devise_est_refuse",
        ]),
        note: "settle_invoice, compte interne. Le solde du reste (write_off) écrit un \
               compte désigné des réglages (escompte, frais bancaires, pertes sur \
               créances : write_off_account_for_write), gardé par les réglages.",
    },
];

const NOTE_REGLAGES: &str = "Même boucle designated_changes (company_invoice_settings::update) \
     pour les dix colonnes ; le refus direct n'est exercé nommément que sur \
     default_rounding_account_id.";

/// Chemin d'un fichier relatif à `crates/`.
fn sous_crates(relatif: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(relatif)
}

fn lire(chemin: &Path) -> String {
    std::fs::read_to_string(chemin).unwrap_or_else(|e| panic!("lecture de {chemin:?} : {e}"))
}

/// Les colonnes `<table>.<colonne>` finissant par `_account_id` des blocs
/// `CREATE TABLE`, avec, pour chacune, la table que sa clé étrangère
/// référence (`None` sans clé étrangère).
fn colonnes_de_compte(sql: &str) -> BTreeMap<String, Option<String>> {
    let mut out = BTreeMap::new();
    let mut table: Option<String> = None;
    let mut colonnes: Vec<String> = Vec::new();
    let mut fks: BTreeMap<String, String> = BTreeMap::new();
    let entre_backquotes = |s: &str| -> Vec<String> {
        s.split('`')
            .enumerate()
            .filter(|(i, _)| i % 2 == 1)
            .map(|(_, x)| x.to_string())
            .collect()
    };
    for ligne in sql.lines() {
        let l = ligne.trim();
        if l.starts_with("CREATE TABLE") {
            table = entre_backquotes(l).into_iter().next();
            colonnes.clear();
            fks.clear();
            continue;
        }
        let Some(t) = table.as_ref() else { continue };
        if l.starts_with(')') {
            for c in colonnes.drain(..) {
                let fk = fks.get(&c).cloned();
                out.insert(format!("{t}.{c}"), fk);
            }
            table = None;
            continue;
        }
        if l.starts_with('`') {
            if let Some(c) = entre_backquotes(l).into_iter().next()
                && c.ends_with("_account_id")
            {
                colonnes.push(c);
            }
        } else if l.contains("FOREIGN KEY") {
            // CONSTRAINT `fk` FOREIGN KEY (`col`) REFERENCES `table` (`id`)
            let noms = entre_backquotes(l);
            if noms.len() >= 3 {
                fks.insert(noms[1].clone(), noms[2].clone());
            }
        }
    }
    out
}

/// Les écarts entre l'ensemble extrait et la liste : `(absentes de la liste,
/// en trop dans la liste)`.
fn ecarts(extrait: &BTreeSet<String>, liste: &BTreeSet<String>) -> (Vec<String>, Vec<String>) {
    (
        extrait.difference(liste).cloned().collect(),
        liste.difference(extrait).cloned().collect(),
    )
}

/// `true` si `source` définit la fonction `nom` (`fn nom(`, `async fn`
/// compris).
fn fonction_definie(source: &str, nom: &str) -> bool {
    source.contains(&format!("fn {nom}("))
}

#[test]
fn chaque_colonne_de_compte_du_schema_a_sa_preuve() {
    let sql = lire(&sous_crates("kesh-db/test-schema/0001_schema_squash.sql"));
    let schema = colonnes_de_compte(&sql);
    assert!(
        schema.len() >= 20,
        "extraction suspecte, {} colonnes : {schema:?}",
        schema.len()
    );

    // (i) L'ensemble extrait égale la liste, dans les deux sens.
    let extrait: BTreeSet<String> = schema.keys().cloned().collect();
    let liste: BTreeSet<String> = INVENTAIRE.iter().map(|c| c.colonne.to_string()).collect();
    assert_eq!(liste.len(), INVENTAIRE.len(), "doublon dans INVENTAIRE");
    let (absentes, en_trop) = ecarts(&extrait, &liste);
    assert!(
        absentes.is_empty() && en_trop.is_empty(),
        "colonnes du schéma absentes de l'inventaire : {absentes:?} ; \
         entrées de l'inventaire absentes du schéma : {en_trop:?}"
    );

    // (ii) Chaque preuve se vérifie.
    let mut sources: BTreeMap<&str, String> = BTreeMap::new();
    for c in INVENTAIRE {
        assert!(!c.note.trim().is_empty(), "{} : note vide", c.colonne);
        match c.preuve {
            Preuve::Tests(tests) => {
                assert!(!tests.is_empty(), "{} : aucun test", c.colonne);
                for t in tests {
                    let (fichier, fonction) = t
                        .split_once("::")
                        .unwrap_or_else(|| panic!("{} : preuve mal formée {t}", c.colonne));
                    let source = sources
                        .entry(fichier)
                        .or_insert_with(|| lire(&sous_crates(fichier)));
                    assert!(
                        fonction_definie(source, fonction),
                        "{} : la fonction {fonction} n'existe pas dans {fichier}",
                        c.colonne
                    );
                }
            }
            Preuve::VersBankAccounts => {
                assert_eq!(
                    schema[c.colonne].as_deref(),
                    Some("bank_accounts"),
                    "{} : clé étrangère vers bank_accounts attendue",
                    c.colonne
                );
            }
        }
    }
}

/// Sonde positive : une liste erronée (une colonne oubliée, une colonne
/// inventée) est détectée, et l'écart nommé.
#[test]
fn sonde_une_liste_erronee_est_detectee() {
    let extrait: BTreeSet<String> = ["a.x_account_id", "b.y_account_id"]
        .into_iter()
        .map(String::from)
        .collect();
    let fausse: BTreeSet<String> = ["a.x_account_id", "c.z_account_id"]
        .into_iter()
        .map(String::from)
        .collect();
    assert_eq!(
        ecarts(&extrait, &fausse),
        (
            vec!["b.y_account_id".to_string()],
            vec!["c.z_account_id".to_string()]
        )
    );
    assert_eq!(ecarts(&extrait, &extrait), (vec![], vec![]));

    // Une fonction absente est détectée ; un préfixe ne suffit pas.
    let source = "async fn un_test_reel(pool: MySqlPool) {}\nfn autre() {}";
    assert!(fonction_definie(source, "un_test_reel"));
    assert!(fonction_definie(source, "autre"));
    assert!(!fonction_definie(source, "un_test"));
    assert!(!fonction_definie(source, "inexistant"));
}

/// Sonde de l'extracteur sur un schéma minimal : colonnes `*_account_id` de
/// chaque bloc, clé étrangère lue, colonne `account_id` nue et colonnes hors
/// bloc ignorées.
#[test]
fn sonde_l_extracteur_lit_les_blocs_et_les_cles() {
    let sql = "\
CREATE TABLE `t1` (
  `id` bigint(20) NOT NULL,
  `account_id` bigint(20) NOT NULL,
  `foo_account_id` bigint(20) DEFAULT NULL,
  `bank_account_id` bigint(20) NOT NULL,
  KEY `k` (`foo_account_id`),
  CONSTRAINT `fk_t1_bank` FOREIGN KEY (`bank_account_id`) REFERENCES `bank_accounts` (`id`)
) ENGINE=InnoDB;
  `hors_bloc_account_id` bigint(20),
CREATE TABLE `t2` (
  `bar_account_id` bigint(20) NOT NULL,
  CONSTRAINT `fk_t2` FOREIGN KEY (`bar_account_id`) REFERENCES `accounts` (`id`)
) ENGINE=InnoDB;
";
    let attendu: BTreeMap<String, Option<String>> = [
        ("t1.bank_account_id", Some("bank_accounts")),
        ("t1.foo_account_id", None),
        ("t2.bar_account_id", Some("accounts")),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.map(String::from)))
    .collect();
    assert_eq!(colonnes_de_compte(sql), attendu);
}
