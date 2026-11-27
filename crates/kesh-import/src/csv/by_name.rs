//! Lecture d'un CSV par les **noms** de ses colonnes (Story 38-11a).
//!
//! Le CSV bancaire de la Story 8-2 se lit par indices, réglés dans un profil.
//! Les exports des prestataires de paiement (PayPal, puis Wise et Stripe, Epics
//! 39 et 40) ont un en-tête stable mais un ordre et un nombre de colonnes qui
//! varient d'un compte à l'autre : ils se lisent par noms. Ce module localise
//! les colonnes voulues dans la ligne d'en-tête et rend les cellules d'un
//! enregistrement par le nom de leur colonne.
//!
//! Règles communes :
//!
//! - les noms se comparent après `trim` et **sans casse** ;
//! - une colonne **requise** absente fait refuser le fichier, et le refus
//!   nomme **toutes** les requises absentes, dans l'ordre de leur déclaration ;
//! - une colonne **facultative** absente se lit comme une cellule vide ;
//! - deux colonnes d'en-tête portant le nom d'une colonne lue (requise ou
//!   facultative) font refuser le fichier : laquelle lire serait une
//!   supposition. Un doublon parmi les colonnes **ignorées** ne gêne pas.
//!
//! Aucun I/O, aucune dépendance interne.

/// Refus d'un en-tête lu par noms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderByNameError {
    /// Les colonnes requises absentes de l'en-tête, toutes, dans l'ordre de
    /// leur déclaration (graphie de la déclaration, non celle du fichier).
    Missing(Vec<String>),
    /// Une colonne lue apparaît deux fois dans l'en-tête (graphie de la
    /// déclaration).
    Duplicate(String),
}

/// Les enregistrements d'un CSV lu par noms, avant tout contrôle de colonnes.
#[derive(Debug, Default)]
pub(crate) struct RawRecords {
    /// L'en-tête : le premier enregistrement non vide.
    pub header: Option<csv::StringRecord>,
    /// Les enregistrements de données `(numéro de ligne du fichier, cellules)`,
    /// de même largeur que l'en-tête ; 1 = l'en-tête.
    pub records: Vec<(usize, csv::StringRecord)>,
    /// Les lignes de largeur différente de l'en-tête ou illisibles
    /// `(numéro de ligne, valeur à citer)`, dans l'ordre du fichier.
    pub malformed: Vec<(usize, String)>,
}

/// Découpe un texte CSV (virgule, guillemets doubles) en en-tête et
/// enregistrements : lignes entièrement vides sautées, numéro de ligne celui
/// du fichier (⚠️ un enregistrement qui suit une ligne vide au milieu du
/// fichier reçoit le numéro de cette ligne vide : le crate `csv` date un
/// enregistrement avant de sauter les lignes vides qui le précèdent).
/// `flexible(true)` : la largeur se contrôle ici, ce qui permet de sauter une
/// ligne vide (fin de fichier) avant de la compter fausse.
pub(crate) fn read_records(text: &str) -> RawRecords {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(text.as_bytes());
    let mut raw = RawRecords::default();
    for result in reader.records() {
        match result {
            Ok(record) => {
                if record.iter().all(|c| c.trim().is_empty()) {
                    continue;
                }
                let line = record.position().map_or(0, |p| p.line() as usize);
                match &raw.header {
                    None => raw.header = Some(record),
                    Some(h) if record.len() != h.len() => {
                        raw.malformed.push((line, record.len().to_string()));
                    }
                    Some(_) => raw.records.push((line, record)),
                }
            }
            Err(e) => {
                let line = e.position().map_or(0, |p| p.line() as usize);
                raw.malformed.push((line, e.to_string()));
            }
        }
    }
    raw
}

/// Forme comparable d'un nom de colonne : `trim`, puis minuscules.
pub(crate) fn normalize(name: &str) -> String {
    name.trim().to_lowercase()
}

/// Les colonnes d'un en-tête que l'appelant veut lire, localisées par nom.
///
/// Construite par [`ColumnsByName::locate`] ; les cellules se lisent par
/// [`ColumnsByName::cell`].
#[derive(Debug, Clone)]
pub struct ColumnsByName {
    /// `(nom normalisé, indice dans l'en-tête)` de chaque colonne déclarée ;
    /// indice `None` pour une facultative absente.
    columns: Vec<(String, Option<usize>)>,
}

impl ColumnsByName {
    /// Localise dans `header` les colonnes `required` puis `optional`.
    ///
    /// # Erreurs
    ///
    /// - [`HeaderByNameError::Duplicate`] si une colonne déclarée apparaît
    ///   plus d'une fois dans l'en-tête (premier doublon rencontré dans
    ///   l'ordre des déclarations) ;
    /// - [`HeaderByNameError::Missing`] si une ou plusieurs requises
    ///   manquent (toutes listées).
    ///
    /// Le doublon se contrôle avant les absentes : un en-tête qui a les deux
    /// défauts est refusé pour son doublon.
    pub fn locate<'h>(
        header: impl IntoIterator<Item = &'h str>,
        required: &[&str],
        optional: &[&str],
    ) -> Result<Self, HeaderByNameError> {
        let header: Vec<String> = header.into_iter().map(normalize).collect();
        let mut columns = Vec::with_capacity(required.len() + optional.len());
        let mut missing = Vec::new();
        for (name, is_required) in required
            .iter()
            .map(|n| (n, true))
            .chain(optional.iter().map(|n| (n, false)))
        {
            let wanted = normalize(name);
            let mut found = header
                .iter()
                .enumerate()
                .filter(|(_, h)| **h == wanted)
                .map(|(i, _)| i);
            let index = found.next();
            if found.next().is_some() {
                return Err(HeaderByNameError::Duplicate((*name).to_string()));
            }
            if index.is_none() && is_required {
                missing.push((*name).to_string());
            }
            columns.push((wanted, index));
        }
        if !missing.is_empty() {
            return Err(HeaderByNameError::Missing(missing));
        }
        Ok(Self { columns })
    }

    /// La cellule de la colonne `name` dans `record`, après `trim` ;
    /// `None` si elle est vide, si la colonne (facultative) est absente de
    /// l'en-tête, ou si l'enregistrement est plus court que l'en-tête.
    ///
    /// `name` doit être l'un des noms passés à [`ColumnsByName::locate`] :
    /// un nom non déclaré est une faute de programmation (assertion en
    /// débogage, `None` en production).
    pub fn cell<'r>(&self, record: &'r csv::StringRecord, name: &str) -> Option<&'r str> {
        let wanted = normalize(name);
        let entry = self.columns.iter().find(|(n, _)| *n == wanted);
        debug_assert!(entry.is_some(), "colonne non déclarée : {name}");
        let index = entry.and_then(|(_, i)| *i)?;
        let value = record.get(index)?.trim();
        (!value.is_empty()).then_some(value)
    }

    /// `true` si la colonne `name` (déclarée) est présente dans l'en-tête.
    pub fn has(&self, name: &str) -> bool {
        let wanted = normalize(name);
        self.columns
            .iter()
            .any(|(n, i)| *n == wanted && i.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_records_numbers_lines_skips_blanks_and_flags_bad_widths() {
        let raw = read_records("a,b\n1,2\n3\n4,5\n,\n\n");
        assert_eq!(raw.header.unwrap().iter().collect::<Vec<_>>(), ["a", "b"]);
        let lines: Vec<usize> = raw.records.iter().map(|(l, _)| *l).collect();
        assert_eq!(lines, [2, 4]);
        assert_eq!(raw.malformed, [(3, "1".to_string())]);
    }

    #[test]
    fn read_records_keeps_a_multiline_quoted_cell_in_one_record() {
        let raw = read_records("a,b\n\"x\ny\",2\nz,3\n");
        assert_eq!(raw.records.len(), 2);
        assert_eq!(raw.records[0].1.get(0), Some("x\ny"));
        assert_eq!(raw.records[1].0, 4);
        assert!(raw.malformed.is_empty());
        assert!(read_records("").header.is_none());
    }

    fn rec(cells: &[&str]) -> csv::StringRecord {
        csv::StringRecord::from(cells.to_vec())
    }

    #[test]
    fn locates_by_name_in_any_order_trimmed_without_case() {
        let cols = ColumnsByName::locate([" net ", "GROSS", "Autre"], &["Gross", "Net"], &["Fee"])
            .unwrap();
        let r = rec(&[" 3.00 ", "4.00", "x"]);
        assert_eq!(cols.cell(&r, "Gross"), Some("4.00"));
        assert_eq!(cols.cell(&r, "net"), Some("3.00"));
        assert_eq!(cols.cell(&r, "Fee"), None, "facultative absente = vide");
        assert!(!cols.has("Fee"));
        assert!(cols.has("Gross"));
    }

    #[test]
    fn empty_cell_and_short_record_are_none() {
        let cols = ColumnsByName::locate(["A", "B"], &["A", "B"], &[]).unwrap();
        assert_eq!(cols.cell(&rec(&["  ", "x"]), "A"), None);
        assert_eq!(cols.cell(&rec(&["x"]), "B"), None);
    }

    #[test]
    fn all_missing_required_are_listed_in_declaration_order() {
        let err = ColumnsByName::locate(["B"], &["A", "B", "C"], &["D"]).unwrap_err();
        assert_eq!(
            err,
            HeaderByNameError::Missing(vec!["A".to_string(), "C".to_string()])
        );
    }

    #[test]
    fn duplicate_of_a_read_column_is_refused() {
        let err = ColumnsByName::locate(["A", " a "], &["A"], &[]).unwrap_err();
        assert_eq!(err, HeaderByNameError::Duplicate("A".to_string()));
        let err = ColumnsByName::locate(["A", "O", "o"], &["A"], &["O"]).unwrap_err();
        assert_eq!(err, HeaderByNameError::Duplicate("O".to_string()));
    }

    #[test]
    fn duplicate_of_an_ignored_column_is_accepted() {
        let cols = ColumnsByName::locate(["A", "", "", "X", "X"], &["A"], &[]).unwrap();
        assert_eq!(cols.cell(&rec(&["1", "", "", "", ""]), "A"), Some("1"));
    }
}
