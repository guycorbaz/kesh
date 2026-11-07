//! Variante v04 du parseur CAMT.053 (`camt.053.001.04`).
//!
//! Le tag `<Document>` doit déjà avoir été consommé par
//! [`super::parse`]. Ce module continue le flux d'événements à partir de
//! `<BkToCstmrStmt>` jusqu'à la fermeture de `<Document>`, en
//! produisant un [`ImportedStatement`](crate::types::ImportedStatement)
//! par `<Stmt>` rencontré.

use quick_xml::NsReader;

use crate::error::CamtError;
use crate::types::ImportedStatement;

/// Continue le parsing d'un document v04 à partir d'un reader déjà
/// positionné après le tag `<Document>` ouvrant.
///
/// Le texte d'un élément se lit en entier, entités et CDATA compris (#819).
/// Le lecteur fourni ne doit **pas** rogner le texte (`trim_text(false)`, le
/// défaut de quick-xml) : rogné morceau par morceau, « Dupont &amp; Fils » se
/// lirait « Dupont&Fils ».
pub fn parse<R: std::io::BufRead>(
    reader: &mut NsReader<R>,
) -> Result<Vec<ImportedStatement>, CamtError> {
    super::parse_corrige(reader, "001.04")
}
