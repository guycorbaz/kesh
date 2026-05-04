//! Variante v08 du parseur CAMT.053 (`camt.053.001.08`).
//!
//! Delta v04 → v08 effectivement mobilisé par le parseur Kesh : le
//! wrapping `<Pty>` autour de `<Cdtr>` / `<Dbtr>` dans `<RltdPties>`
//! (Party11Choice → Party40Choice / PartyIdentification135). Ce delta
//! est absorbé par le matching de chemin dans
//! [`super::parse_with_version`], qui accepte les deux formes (parent
//! direct `Cdtr|Dbtr > Nm` ou parent via `Pty > Nm`).
//!
//! Les autres deltas v04 → v08 (enrichissements `<Othr><SchmeNm>`,
//! nouveaux types de remittance information structurée) ne sont pas
//! mobilisés par les champs extraits — la même logique de parsing
//! s'applique donc.

use quick_xml::NsReader;

use crate::error::CamtError;
use crate::types::ImportedStatement;

/// Continue le parsing d'un document v08 à partir d'un reader déjà
/// positionné après le tag `<Document>` ouvrant.
pub fn parse<R: std::io::BufRead>(
    reader: &mut NsReader<R>,
) -> Result<Vec<ImportedStatement>, CamtError> {
    super::parse_with_version(reader, "001.08")
}
