//! `kesh_exercices` — les exercices de la société (Story 32-9a, AC-a10,
//! C-32-9-11) : sans lui, aucun rapport ne s'appelle, faute d'identifiant
//! d'exercice. Portée `books:read`.

use serde::Deserialize;

use crate::mcp::outil::{ContexteMcp, FuturOutil, McpTool, arguments};
use crate::mcp::sortie::{Date, Resume, TexteNonFiable, donnees, sortie};
use crate::mcp::textes::depuis_app_error;
use crate::services::acces::Portee;
use crate::services::exercices;

/// L'outil.
pub struct KeshExercices;

/// Aucun argument.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SansArgument {}

sortie! {
    /// Un exercice.
    struct Exercice {
        id: i64,
        nom: TexteNonFiable,
        debut: Date,
        fin: Date,
        statut: kesh_db::entities::FiscalYearStatus,
        // Story 37-1 (AC-2, V10) : le mode et les genres de cours, présents en
        // mono-devise aussi (ce ne sont pas des champs en devise).
        multi_devises: bool,
        genre_ouverture: kesh_db::entities::RateKind,
        genre_cloture: kesh_db::entities::RateKind,
    }
}

sortie! {
    /// La sortie.
    struct Exercices { exercices: Vec<Exercice> }
}

impl McpTool for KeshExercices {
    fn nom(&self) -> &'static str {
        "kesh_exercices"
    }

    fn description(&self) -> &'static str {
        "Liste les exercices comptables de la société (identifiant, nom, dates, statut \
         ouvert ou clos, mode multi-devises, genre du cours d'ouverture et de clôture : \
         day pour le cours journalier, month pour le cours mensuel), du plus récent au \
         plus ancien. L'identifiant sert aux rapports (kesh_balance, kesh_bilan, \
         kesh_compte_de_resultat)."
    }

    fn schema_entree(&self) -> &'static str {
        r#"{"type":"object","properties":{},"additionalProperties":false}"#
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
            let SansArgument {} = arguments(valeur)?;
            let liste = exercices::lister(ctx.acces(), ctx.lecture())
                .await
                .map_err(depuis_app_error)?;
            let sortie = Exercices {
                exercices: liste
                    .into_iter()
                    .map(|fy| Exercice {
                        id: fy.id,
                        nom: TexteNonFiable::de(fy.name),
                        debut: Date(fy.start_date),
                        fin: Date(fy.end_date),
                        statut: fy.status,
                        multi_devises: fy.multi_currency,
                        genre_ouverture: fy.opening_rate_kind,
                        genre_cloture: fy.closing_rate_kind,
                    })
                    .collect(),
            };
            donnees(Resume::outil("kesh_exercices"), &sortie)
        })
    }
}
