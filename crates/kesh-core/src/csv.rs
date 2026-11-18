//! Échappement commun des cellules **texte** des exports CSV.
//!
//! Une seule règle pour tous les CSV du produit : les exports de `kesh-api`
//! (factures, journal d'audit, tables de la sauvegarde) et les rapports
//! comptables de `kesh-report` (journal, grand livre, balance, bilan, compte
//! de résultat et autres). Logique pure, sans I/O — d'où sa place dans
//! `kesh-core`, dont dépendent les deux crates (`kesh-report` ne peut pas
//! dépendre de `kesh-api`).
//!
//! ⚠️ Elle ne s'applique qu'aux cellules **texte** : un montant négatif
//! (`-12.50`) commence par `-` et doit rester un nombre pour le tableur.

/// Échappe une cellule CSV texte.
///
/// Un tableur interprète une cellule commençant par `=`, `+`, `-` ou `@` comme
/// une expression à évaluer : on la préfixe d'une apostrophe simple pour forcer
/// le texte.
///
/// Remplace aussi TAB, CR et LF par une espace — Excel traite la tabulation en
/// déclencheur dans certains contextes, et un champ remplacé par une espace
/// reste lisible sans casser l'alignement pour un parseur tiers naïf. Le
/// **blanc de tête** est pris en compte : sans cela, `" =cmd"` contournerait le
/// contrôle.
///
/// Extraite de `routes::invoices` par la Story 25-1c-a, puis déplacée de
/// `kesh-api` (`util`) vers `kesh-core` par la Story 37-5 pour servir aussi
/// `kesh-report` — **comportement identique**.
///
/// ```
/// use kesh_core::csv::csv_sanitize;
/// assert_eq!(csv_sanitize("=1+1".to_string()), "'=1+1");
/// assert_eq!(csv_sanitize("Müller SA".to_string()), "Müller SA");
/// ```
pub fn csv_sanitize(raw: String) -> String {
    let raw: String = raw
        .chars()
        .map(|c| {
            if c == '\r' || c == '\n' || c == '\t' {
                ' '
            } else {
                c
            }
        })
        .collect();
    if let Some(first) = raw.trim_start().chars().next()
        && matches!(first, '=' | '+' | '-' | '@')
    {
        let mut out = String::with_capacity(raw.len() + 1);
        out.push('\'');
        out.push_str(&raw);
        return out;
    }
    raw
}

#[cfg(test)]
mod tests {
    use super::*;

    // ----- csv_sanitize (Story 25-1c-a, AC 13 ; déplacés de kesh-api, 37-5) -----

    #[test]
    fn csv_sanitize_prefixes_each_formula_trigger() {
        for declencheur in ['=', '+', '-', '@'] {
            let cellule = format!("{declencheur}HYPERLINK(\"x\")");
            assert_eq!(
                csv_sanitize(cellule.clone()),
                format!("'{cellule}"),
                "une cellule commençant par `{declencheur}` doit être préfixée"
            );
        }
    }

    #[test]
    fn csv_sanitize_sees_through_leading_whitespace() {
        // Sans `trim_start`, `" =cmd"` contournerait le contrôle.
        assert_eq!(csv_sanitize(" =cmd".to_string()), "' =cmd");
        assert_eq!(csv_sanitize("\t=cmd".to_string()), "' =cmd");
    }

    #[test]
    fn csv_sanitize_replaces_cr_lf_and_tab_with_spaces() {
        assert_eq!(
            csv_sanitize("a\rb\nc\td".to_string()),
            "a b c d",
            "CR, LF et TAB deviennent des espaces"
        );
        // En tête de cellule aussi : la tabulation et le retour chariot tombent.
        assert_eq!(csv_sanitize("\tabc".to_string()), " abc");
        assert_eq!(csv_sanitize("\rabc".to_string()), " abc");
    }

    #[test]
    fn csv_sanitize_leaves_an_ordinary_string_untouched() {
        assert_eq!(csv_sanitize("Müller SA".to_string()), "Müller SA");
        assert_eq!(csv_sanitize(String::new()), "");
    }
}
