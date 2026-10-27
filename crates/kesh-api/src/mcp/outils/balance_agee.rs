//! `kesh_balance_agee` — la balance âgée des débiteurs, arrêtée à aujourd'hui
//! (Story 32-9a, AC-a10) ; la consultation s'inscrit au journal d'audit. Le
//! rapport rend un débiteur par client à solde ouvert : l'outil le **tranche**
//! (défaut 40, maximum 75) et rend les totaux du rapport entier (validation P3,
//! F2). Portée `invoices:read`.

use serde::Deserialize;

use crate::mcp::outil::{ContexteMcp, FuturOutil, McpTool, arguments};
use crate::mcp::outils::commun::{decalage, limite, nombre, tranche};
use crate::mcp::sortie::{Date, Montant, Resume, TexteNonFiable, donnees, sortie};
use crate::mcp::textes::depuis_app_error;
use crate::services::acces::Portee;
use crate::services::rapports::{self, Affichage};

/// Débiteurs par défaut.
pub const LIMITE_DEFAUT: i64 = 40;
/// Débiteurs au plus (aucun plafond de service).
pub const LIMITE_MAX_MCP: i64 = 75;

/// L'outil.
pub struct KeshBalanceAgee;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Arguments {
    limit: Option<i64>,
    offset: Option<i64>,
}

sortie! {
    /// Les tranches d'âge.
    struct Tranches {
        non_echu: Montant,
        de_1_a_30_jours: Montant,
        de_31_a_60_jours: Montant,
        de_61_a_90_jours: Montant,
        plus_de_90_jours: Montant,
        total: Montant,
    }
}

sortie! {
    /// Un débiteur.
    struct Debiteur { client_id: i64, client: TexteNonFiable, tranches: Tranches }
}

sortie! {
    /// La sortie.
    struct BalanceAgee {
        au: Date,
        lignes: Vec<Debiteur>,
        total: i64,
        limit: i64,
        offset: i64,
        totaux: Tranches,
    }
}

fn tranches(b: kesh_report::aged_receivables::AgedBucket) -> Tranches {
    Tranches {
        non_echu: Montant(b.not_due),
        de_1_a_30_jours: Montant(b.days_1_to_30),
        de_31_a_60_jours: Montant(b.days_31_to_60),
        de_61_a_90_jours: Montant(b.days_61_to_90),
        plus_de_90_jours: Montant(b.days_over_90),
        total: Montant(b.total),
    }
}

impl McpTool for KeshBalanceAgee {
    fn nom(&self) -> &'static str {
        "kesh_balance_agee"
    }

    fn description(&self) -> &'static str {
        "La balance âgée des débiteurs, arrêtée à aujourd'hui : le reste dû de chaque client \
         par tranche d'âge (non échu, 1-30, 31-60, 61-90, plus de 90 jours), par tranches \
         (limit, offset), avec les totaux du rapport entier. Si la réponse est trop grosse, \
         réduisez limit."
    }

    fn schema_entree(&self) -> &'static str {
        r#"{"type":"object","properties":{"limit":{"type":"integer","minimum":1,"maximum":75,"default":40},"offset":{"type":"integer","minimum":0}},"additionalProperties":false}"#
    }

    fn portee(&self) -> Portee {
        Portee::InvoicesRead
    }

    fn executer<'a>(
        &'a self,
        ctx: &'a ContexteMcp<'a>,
        valeur: serde_json::Value,
    ) -> FuturOutil<'a> {
        Box::pin(async move {
            let a: Arguments = arguments(valeur)?;
            let limit = limite(a.limit, LIMITE_DEFAUT, LIMITE_MAX_MCP)?;
            let offset = decalage(a.offset)?;
            let resume = Resume::outil("kesh_balance_agee")
                .entier("limit", Some(limit))
                .entier("offset", Some(offset));
            let rapport = rapports::balance_agee(ctx.acces(), ctx.lecture(), Affichage)
                .await
                .map_err(depuis_app_error)?;
            let r = ctx.afficher(rapport).await;
            let total = nombre(r.rows.len());
            let sortie = BalanceAgee {
                au: Date(r.as_of),
                lignes: tranche(r.rows, offset, limit)
                    .into_iter()
                    .map(|l| Debiteur {
                        client_id: l.contact_id,
                        client: TexteNonFiable::de(l.contact_name),
                        tranches: tranches(l.buckets),
                    })
                    .collect(),
                total,
                limit,
                offset,
                totaux: tranches(r.totals),
            };
            donnees(resume, &sortie)
        })
    }
}
