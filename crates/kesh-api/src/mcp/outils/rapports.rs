//! `kesh_balance`, `kesh_bilan`, `kesh_compte_de_resultat` — les rapports à
//! période d'un exercice (Story 32-9a, AC-a10) ; chaque consultation s'inscrit
//! au journal d'audit (`ContexteMcp::afficher`). Portée `books:read`.
//!
//! La balance rend tout compte actif : l'outil la **tranche** (défaut 30,
//! maximum 55 lignes) et rend les totaux du rapport entier. Le bilan et le
//! compte de résultat ne sont pas paginés : au-delà d'environ 60 comptes à
//! solde non nul, la réponse est refusée pour sa taille (C-32-9-8).

use chrono::NaiveDate;
use serde::Deserialize;

use crate::mcp::outil::{ContexteMcp, FuturOutil, McpTool, arguments};
use crate::mcp::outils::commun::{decalage, limite, nombre, tranche};
use crate::mcp::sortie::{Date, Montant, Resume, TexteNonFiable, donnees, sortie};
use crate::mcp::textes::depuis_app_error;
use crate::services::acces::Portee;
use crate::services::rapports::{self, Affichage, DemandePeriode};

/// Lignes de balance par défaut.
pub const LIMITE_DEFAUT_BALANCE: i64 = 30;
/// Lignes de balance au plus (aucun plafond de service).
pub const LIMITE_MAX_MCP_BALANCE: i64 = 55;

/// La balance.
pub struct KeshBalance;
/// Le bilan.
pub struct KeshBilan;
/// Le compte de résultat.
pub struct KeshCompteDeResultat;

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ArgumentsPeriode {
    fiscal_year_id: i64,
    period_start: Option<NaiveDate>,
    period_end: Option<NaiveDate>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ArgumentsBalance {
    fiscal_year_id: i64,
    period_start: Option<NaiveDate>,
    period_end: Option<NaiveDate>,
    limit: Option<i64>,
    offset: Option<i64>,
}

const SCHEMA_PERIODE: &str = r#"{"type":"object","properties":{"fiscalYearId":{"type":"integer","minimum":1},"periodStart":{"type":"string","format":"date"},"periodEnd":{"type":"string","format":"date"}},"required":["fiscalYearId"],"additionalProperties":false}"#;

sortie! {
    /// La période résolue.
    struct Periode { exercice_id: i64, debut: Date, fin: Date }
}

sortie! {
    /// Un compte et son solde (bilan, compte de résultat).
    struct LigneDeSolde {
        compte_id: i64,
        numero: TexteNonFiable,
        nom: TexteNonFiable,
        type_de_compte: kesh_db::entities::AccountType,
        actif: bool,
        solde: Montant,
        role: Option<kesh_db::entities::AccountRole>,
    }
}

sortie! {
    /// Le bilan.
    struct Bilan {
        periode: Periode,
        actifs: Vec<LigneDeSolde>,
        passifs: Vec<LigneDeSolde>,
        fonds_propres: Vec<LigneDeSolde>,
        total_actifs: Montant,
        total_passifs: Montant,
        total_fonds_propres: Montant,
        report: Montant,
        resultat: Montant,
        equation_tenue: bool,
    }
}

sortie! {
    /// Le compte de résultat.
    struct CompteDeResultat {
        periode: Periode,
        produits: Vec<LigneDeSolde>,
        charges: Vec<LigneDeSolde>,
        total_produits: Montant,
        total_charges: Montant,
        resultat: Montant,
    }
}

sortie! {
    /// Une ligne de balance.
    struct LigneBalance {
        compte_id: i64,
        numero: TexteNonFiable,
        nom: TexteNonFiable,
        type_de_compte: kesh_db::entities::AccountType,
        actif: bool,
        ouverture: Montant,
        debit: Montant,
        credit: Montant,
        solde: Montant,
        cloture: Montant,
    }
}

sortie! {
    /// La balance : une tranche, et les totaux du rapport entier.
    struct Balance {
        periode: Periode,
        lignes: Vec<LigneBalance>,
        total: i64,
        limit: i64,
        offset: i64,
        total_debit: Montant,
        total_credit: Montant,
        equilibree: bool,
        report_benefice: Montant,
        ouverture_equilibree: bool,
    }
}

fn periode(p: kesh_report::ReportPeriod) -> Periode {
    Periode {
        exercice_id: p.fiscal_year_id,
        debut: Date(p.start_date),
        fin: Date(p.end_date),
    }
}

fn ligne(a: kesh_report::AccountBalance) -> LigneDeSolde {
    LigneDeSolde {
        compte_id: a.account_id,
        numero: TexteNonFiable::de(a.account_number),
        nom: TexteNonFiable::de(a.account_name),
        type_de_compte: a.account_type,
        actif: a.active,
        solde: Montant(a.balance),
        role: a.role,
    }
}

fn lignes(v: Vec<kesh_report::AccountBalance>) -> Vec<LigneDeSolde> {
    v.into_iter().map(ligne).collect()
}

fn resume_periode(nom: &'static str, a: &ArgumentsPeriode) -> Resume {
    Resume::outil(nom)
        .entier("fiscalYearId", Some(a.fiscal_year_id))
        .date("periodStart", a.period_start)
        .date("periodEnd", a.period_end)
}

fn demande(a: &ArgumentsPeriode) -> DemandePeriode {
    DemandePeriode {
        fiscal_year_id: a.fiscal_year_id,
        period_start: a.period_start,
        period_end: a.period_end,
    }
}

impl McpTool for KeshBalance {
    fn nom(&self) -> &'static str {
        "kesh_balance"
    }

    fn description(&self) -> &'static str {
        "Lit la balance des comptes d'un exercice (fiscalYearId, requis ; kesh_exercices), \
         sur une période facultative (periodStart, periodEnd) : soldes d'ouverture, \
         mouvements et soldes par compte, par tranches (limit, offset), avec les totaux du \
         rapport entier. Si la réponse est trop grosse, réduisez limit."
    }

    fn schema_entree(&self) -> &'static str {
        r#"{"type":"object","properties":{"fiscalYearId":{"type":"integer","minimum":1},"periodStart":{"type":"string","format":"date"},"periodEnd":{"type":"string","format":"date"},"limit":{"type":"integer","minimum":1,"maximum":55,"default":30},"offset":{"type":"integer","minimum":0}},"required":["fiscalYearId"],"additionalProperties":false}"#
    }

    fn portee(&self) -> Portee {
        Portee::BooksRead
    }

    fn executer<'a>(
        &'a self,
        ctx: &'a ContexteMcp<'a>,
        valeur: serde_json::Value,
    ) -> FuturOutil<'a> {
        Box::pin(async move {
            let b: ArgumentsBalance = arguments(valeur)?;
            let limit = limite(b.limit, LIMITE_DEFAUT_BALANCE, LIMITE_MAX_MCP_BALANCE)?;
            let offset = decalage(b.offset)?;
            let a = ArgumentsPeriode {
                fiscal_year_id: b.fiscal_year_id,
                period_start: b.period_start,
                period_end: b.period_end,
            };
            let resume = resume_periode("kesh_balance", &a)
                .entier("limit", Some(limit))
                .entier("offset", Some(offset));
            let rapport = rapports::balance(ctx.acces(), ctx.lecture(), demande(&a), Affichage)
                .await
                .map_err(depuis_app_error)?;
            let r = ctx.afficher(rapport).await;
            let total = nombre(r.rows.len());
            let sortie = Balance {
                periode: periode(r.period),
                lignes: tranche(r.rows, offset, limit)
                    .into_iter()
                    .map(|l| LigneBalance {
                        compte_id: l.account_id,
                        numero: TexteNonFiable::de(l.account_number),
                        nom: TexteNonFiable::de(l.account_name),
                        type_de_compte: l.account_type,
                        actif: l.active,
                        ouverture: Montant(l.opening_balance),
                        debit: Montant(l.total_debit),
                        credit: Montant(l.total_credit),
                        solde: Montant(l.balance),
                        cloture: Montant(l.closing_balance),
                    })
                    .collect(),
                total,
                limit,
                offset,
                total_debit: Montant(r.total_debit),
                total_credit: Montant(r.total_credit),
                equilibree: r.balanced,
                report_benefice: Montant(r.retained_earnings),
                ouverture_equilibree: r.opening_balanced,
            };
            donnees(resume, &sortie)
        })
    }
}

impl McpTool for KeshBilan {
    fn nom(&self) -> &'static str {
        "kesh_bilan"
    }

    fn description(&self) -> &'static str {
        "Lit le bilan d'un exercice (fiscalYearId, requis ; kesh_exercices) à la fin d'une \
         période facultative : actifs, passifs, fonds propres et leurs totaux, comptes à \
         solde non nul. Non paginé : au-delà d'environ 60 comptes à solde non nul, la \
         réponse est refusée."
    }

    fn schema_entree(&self) -> &'static str {
        SCHEMA_PERIODE
    }

    fn portee(&self) -> Portee {
        Portee::BooksRead
    }

    fn executer<'a>(
        &'a self,
        ctx: &'a ContexteMcp<'a>,
        valeur: serde_json::Value,
    ) -> FuturOutil<'a> {
        Box::pin(async move {
            let a: ArgumentsPeriode = arguments(valeur)?;
            let resume = resume_periode("kesh_bilan", &a);
            let rapport = rapports::bilan(ctx.acces(), ctx.lecture(), demande(&a), Affichage)
                .await
                .map_err(depuis_app_error)?;
            let r = ctx.afficher(rapport).await;
            let sortie = Bilan {
                periode: periode(r.period),
                actifs: lignes(r.assets),
                passifs: lignes(r.liabilities),
                fonds_propres: lignes(r.equity),
                total_actifs: Montant(r.total_assets),
                total_passifs: Montant(r.total_liabilities),
                total_fonds_propres: Montant(r.total_equity),
                report: Montant(r.retained_earnings),
                resultat: Montant(r.equity_result),
                equation_tenue: r.equation_holds,
            };
            donnees(resume, &sortie)
        })
    }
}

impl McpTool for KeshCompteDeResultat {
    fn nom(&self) -> &'static str {
        "kesh_compte_de_resultat"
    }

    fn description(&self) -> &'static str {
        "Lit le compte de résultat d'un exercice (fiscalYearId, requis ; kesh_exercices) sur \
         une période facultative : produits, charges, leurs totaux et le résultat. Non \
         paginé : au-delà d'environ 60 comptes à solde non nul, la réponse est refusée ; \
         choisissez une période plus courte."
    }

    fn schema_entree(&self) -> &'static str {
        SCHEMA_PERIODE
    }

    fn portee(&self) -> Portee {
        Portee::BooksRead
    }

    fn executer<'a>(
        &'a self,
        ctx: &'a ContexteMcp<'a>,
        valeur: serde_json::Value,
    ) -> FuturOutil<'a> {
        Box::pin(async move {
            let a: ArgumentsPeriode = arguments(valeur)?;
            let resume = resume_periode("kesh_compte_de_resultat", &a);
            let rapport =
                rapports::compte_de_resultat(ctx.acces(), ctx.lecture(), demande(&a), Affichage)
                    .await
                    .map_err(depuis_app_error)?;
            let r = ctx.afficher(rapport).await;
            let sortie = CompteDeResultat {
                periode: periode(r.period),
                produits: lignes(r.revenues),
                charges: lignes(r.expenses),
                total_produits: Montant(r.total_revenues),
                total_charges: Montant(r.total_expenses),
                resultat: Montant(r.net_result),
            };
            donnees(resume, &sortie)
        })
    }
}
