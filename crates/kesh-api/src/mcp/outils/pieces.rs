//! `kesh_pieces` — la liste des pièces importées (Story 32-9b, refs #723,
//! AC-b2). Portée `documents:read`.
//!
//! **Les métadonnées seulement** (spike, E3 ; C-32-9-13) : le nom d'origine du
//! fichier et son type, enveloppés, le statut, la taille, l'instant de
//! l'import, la facture fournisseur créée de la pièce, et `qrLu`, qui dit si
//! un QR-code y a été lu — **jamais** ce qui y a été lu (IBAN, créancier,
//! référence, montant, message), ni le fichier : `pieces::lister` rend des
//! `PiecePublique`, qui ne portent ni l'un ni l'autre. Le contenu d'une pièce
//! viendra en 0.17.0.

use serde::Deserialize;

use crate::mcp::outil::{ContexteMcp, FuturOutil, McpTool, arguments};
use crate::mcp::outils::commun::{STATUTS_PIECE, StatutPiece, decalage, limite};
use crate::mcp::sortie::{Code, Horodatage, Resume, TexteNonFiable, donnees, sortie};
use crate::mcp::textes::depuis_app_error;
use crate::services::acces::Portee;
use crate::services::pieces::{self, DemandeListePieces, PiecePublique};

/// Taille de page par défaut (§ *Les tailles mesurées* de la fiche).
pub const LIMITE_DEFAUT: i64 = 35;
/// Taille de page maximale de l'outil (le service admet 65).
pub const LIMITE_MAX_MCP: i64 = 65;

/// La liste.
pub struct KeshPieces;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArgumentsListe {
    statut: StatutPiece,
    limit: Option<i64>,
    offset: Option<i64>,
}

sortie! {
    /// Une pièce, sans son fichier ni ce qui a été lu de son QR-code.
    struct PieceSortie {
        id: i64,
        statut: Code<StatutPiece>,
        facture_fournisseur_id: Option<i64>,
        nom_fichier: TexteNonFiable,
        type_mime: TexteNonFiable,
        taille_octets: i64,
        importee_le: Horodatage,
        qr_lu: bool,
    }
}

sortie! {
    /// La liste.
    struct Pieces { pieces: Vec<PieceSortie>, total: i64, limit: i64, offset: i64 }
}

/// La projection d'une pièce. ⚠️ Déstructuration **exhaustive**, sans `..`.
fn piece_sortie(p: PiecePublique) -> PieceSortie {
    let PiecePublique {
        id,
        status,
        supplier_invoice_id,
        original_filename,
        mime_type,
        byte_size,
        created_at,
        qr_lu,
    } = p;
    PieceSortie {
        id,
        statut: Code::parmi(status, STATUTS_PIECE),
        facture_fournisseur_id: supplier_invoice_id,
        nom_fichier: TexteNonFiable::de(original_filename),
        type_mime: TexteNonFiable::de(mime_type),
        taille_octets: byte_size,
        importee_le: Horodatage(created_at),
        qr_lu,
    }
}

impl McpTool for KeshPieces {
    fn nom(&self) -> &'static str {
        "kesh_pieces"
    }

    fn description(&self) -> &'static str {
        "Liste les pièces importées d'un statut (to_complete, completed, discarded), les plus \
         récentes d'abord : nom du fichier, type, taille, date d'import, facture fournisseur \
         créée, et si un QR-code y a été lu. Les métadonnées seulement : ni le contenu du \
         fichier, ni ce qui a été lu du QR-code. Paginées (limit, offset) ; si la réponse est \
         trop grosse, réduisez limit."
    }

    fn schema_entree(&self) -> &'static str {
        r#"{"type":"object","properties":{"statut":{"type":"string","enum":["to_complete","completed","discarded"]},"limit":{"type":"integer","minimum":1,"maximum":65,"default":35},"offset":{"type":"integer","minimum":0}},"required":["statut"],"additionalProperties":false}"#
    }

    fn portee(&self) -> Portee {
        Portee::DocumentsRead
    }

    fn executer<'a>(
        &'a self,
        ctx: &'a ContexteMcp<'a>,
        valeur: serde_json::Value,
    ) -> FuturOutil<'a> {
        Box::pin(async move {
            let a: ArgumentsListe = arguments(valeur)?;
            let limit = limite(a.limit, LIMITE_DEFAUT, LIMITE_MAX_MCP)?;
            let offset = decalage(a.offset)?;
            let resume = Resume::outil("kesh_pieces")
                .code("statut", Some(a.statut.code()))
                .entier("limit", Some(limit))
                .entier("offset", Some(offset));
            let demande = DemandeListePieces {
                statut: Some(a.statut.code().to_string()),
                limit: Some(limit),
                offset: Some(offset),
            };
            let page = pieces::lister(ctx.acces(), ctx.lecture(), demande)
                .await
                .map_err(depuis_app_error)?;
            let sortie = Pieces {
                pieces: page.items.into_iter().map(piece_sortie).collect(),
                total: page.total,
                limit: page.limit,
                offset: page.offset,
            };
            donnees(resume, &sortie)
        })
    }
}
