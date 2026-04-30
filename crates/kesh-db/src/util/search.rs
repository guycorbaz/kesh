//! Helpers de recherche partagés (Story 7-4 / KF-005).
//!
//! Ce module centralise deux helpers historiquement dupliqués dans 4
//! repositories (`contacts`, `products`, `journal_entries`, `invoices`) :
//!
//! - [`escape_like`] — échappement des caractères réservés `LIKE` (`%`, `_`,
//!   `\`) pour usage avec la clause SQL `LIKE ? ESCAPE '\\'`.
//! - [`escape_boolean_ft`] — sanitization des chaînes utilisateur avant
//!   injection dans `MATCH(...) AGAINST(? IN BOOLEAN MODE)`.
//!
//! La mutualisation a été décidée Story 7-4 quand la 4e duplication
//! (`invoices.rs`) a déclenché la condition d'extraction notée dans les
//! commentaires inline pré-existants.

/// Échappe les caractères spéciaux pour l'opérateur SQL `LIKE`.
///
/// Pattern utilisé : `LIKE ? ESCAPE '\\'`. Le backslash est le caractère
/// d'échappement. **Ordre critique** : le backslash doit être doublé
/// AVANT `%` et `_`, sinon le backslash injecté par la première passe
/// réinitialise les passes suivantes.
///
/// Hérité du pattern Story 3.4 (`journal_entries.rs`) — implémentation
/// extraite Story 7-4 / KF-005 / AC #18.
pub fn escape_like(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

/// Caractères opérateurs `BOOLEAN MODE` MariaDB 11.x à supprimer (10).
///
/// Liste vérifiée doc MariaDB 2026-04-29 : `+` `-` (must/must-not),
/// `>` `<` (relevance modifier), `(` `)` (grouping), `~` (penalty),
/// `*` (prefix wildcard, le seul `*` ajouté en suffixe par le repo
/// après strip est légitime), `"` (phrase), `\` (escape interne).
///
/// **Note importante** : `@` n'est PAS un opérateur `BOOLEAN MODE` —
/// il appartient à la grammaire SQL générale. Il passe donc tel quel
/// (utile pour rechercher des fragments d'email).
const BOOLEAN_FT_OPERATORS: &[char] = &['+', '-', '>', '<', '(', ')', '~', '*', '"', '\\'];

/// Sanitize une chaîne utilisateur pour usage dans `MATCH(...) AGAINST(?
/// IN BOOLEAN MODE)` MariaDB.
///
/// **Stratégie : strip TOTAL (pas escape)**. Le backslash-escaping
/// (`\+`, `\-`, etc.) en `BOOLEAN MODE` n'est pas garanti déterministe
/// selon la version MariaDB exacte. Le strip total donne un comportement
/// prévisible sur toutes versions 11.x : l'utilisateur tape du texte,
/// le helper retire les 10 caractères opérateurs, puis le repository
/// caller append un `*` en suffixe pour le prefix wildcard (préservation
/// de l'UX prefix-search).
///
/// **Caractères supprimés** : `+ - > < ( ) ~ * " \` (10 caractères).
///
/// **Caractères PRÉSERVÉS** :
/// - `@` — non-opérateur `BOOLEAN MODE` (utile pour fragments d'email).
/// - `%` `_` — non-opérateurs `BOOLEAN MODE`. Ils sont silencieusement
///   ignorés par le tokenizer InnoDB FULLTEXT (traités comme caractères
///   non-token).
/// - Caractères regex (`$ ^ [ ] | .`) — non-opérateurs `BOOLEAN MODE`.
///   FULLTEXT InnoDB ne supporte pas la regex donc ils passent tels
///   quels et seront généralement ignorés par la tokenization.
/// - Accents UTF-8 (`é`, `ç`, etc.) — préservés (`utf8mb4_unicode_ci`
///   tokenize correctement).
/// - Caractères Unicode étendus (chinois, cyrillique, etc.) — préservés.
///
/// **Comportement aux bornes** :
/// - Whitespace en entrée : `input.trim()` est appliqué.
/// - Payload vide après trim/strip : retourne `""`. Le caller doit
///   tester `if escaped.is_empty() { skip search clause }`.
/// - Payload tokens courts (`"de"`, `"le"`) : préservé tel quel — la
///   limite ≥ 3 caractères est appliquée par MariaDB
///   (`innodb_ft_min_token_size`), pas par ce helper.
///
/// **Sémantique multi-mots** : `"foo bar"` est conservé tel quel
/// (l'espace n'est pas dans la strip-list). Quand le repo append `*`
/// global donnant `"foo bar*"`, MariaDB interprète : « les mots sans
/// préfixe `+`/`-` sont **optionnels avec ranking de pertinence** » (cf.
/// MySQL docs § 14.9.2 — « A word that has no leading +/- operator is
/// optional, but the rows that contain it are rated higher »).
/// Fonctionnellement équivalent à OR inclusif. Si un AND strict
/// multi-mots devient nécessaire en v0.2+, le repo splitterait par
/// whitespace et appendrait `+` + `*` à chaque token (`"+foo* +bar*"`).
///
/// **⚠️ Wildcard sur dernier token uniquement** (Pass 1 F5) : le `*`
/// appendé par le repo se colle au DERNIER token seulement, pas à chaque
/// token. Pour un input `"Jea Pie"`, la query devient `"Jea Pie*"` →
/// `Jea` est traité comme **mot exact** (≥3 chars indexé), `Pie*` comme
/// préfixe. Conséquences observables :
/// - `"Jean Pie"` cherchant `"Jeanette Pierre"` → matche via `Pie*`
///   (préfixe sur `Pierre`), PAS via `Jean` (qui ne matche pas
///   `Jeanette` — pas de prefix wildcard). Le row est retourné mais via
///   le second token, ce qui peut surprendre.
/// - `"Jea Pierre"` cherchant `"Jeanette Pierre"` → matche via `Pierre`
///   exact (le `*` final donne `Pierre*` qui matche `Pierre`).
/// - `"foo bar"` où aucun document ne contient `bar` (exact OU prefix)
///   ni `foo` (exact) → 0 résultats malgré tokens présents en partial.
///
/// Acceptable v0.1 : la recherche multi-mots reste utile (≥1 token
/// match → row retourné). Évolution v0.2+ possible : split + prefix par
/// token pour rendre toute la recherche prefix-friendly.
///
/// # Exemple
///
/// ```ignore
/// use kesh_db::util::search::escape_boolean_ft;
///
/// let user_term = "  Mar  ";
/// let escaped = escape_boolean_ft(user_term);          // "Mar"
/// let bool_query = format!("{}*", escaped);            // "Mar*"
/// // qb.push(" AND MATCH(name) AGAINST(")
/// //   .push_bind(bool_query)
/// //   .push(" IN BOOLEAN MODE)");
/// ```
pub fn escape_boolean_ft(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    trimmed
        .chars()
        .filter(|c| !BOOLEAN_FT_OPERATORS.contains(c))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- escape_like (extrait des 4 repos pré-existants) ----------

    #[test]
    fn escape_like_doubles_backslash_first() {
        // Si l'ordre est inversé, le backslash injecté re-corrompt les
        // passes suivantes (`%` → `\%` puis `\` → `\\` produit `\\%`).
        assert_eq!(escape_like("a%b_c\\d"), "a\\%b\\_c\\\\d");
    }

    #[test]
    fn escape_like_passthrough_normal_chars() {
        assert_eq!(escape_like("Marie Curie"), "Marie Curie");
    }

    #[test]
    fn escape_like_empty() {
        assert_eq!(escape_like(""), "");
    }

    #[test]
    fn escape_like_preserves_accents() {
        assert_eq!(escape_like("Crémant"), "Crémant");
    }

    // ---------- escape_boolean_ft (Story 7-4 / KF-005 / T1.3) ----------

    #[test]
    fn test_escape_strip_all_operators() {
        // Pour chacun des 10 opérateurs, vérifier qu'il est strippé entre
        // deux tokens texte.
        for op in BOOLEAN_FT_OPERATORS {
            let input = format!("foo{op}bar");
            assert_eq!(
                escape_boolean_ft(&input),
                "foobar",
                "operator {op:?} non strippé"
            );
        }
    }

    #[test]
    fn test_escape_strip_combined() {
        // Combinaison de plusieurs opérateurs.
        assert_eq!(escape_boolean_ft("foo+*bar\"baz"), "foobarbaz");
        assert_eq!(escape_boolean_ft("(foo) -bar ~baz"), "foo bar baz");
    }

    #[test]
    fn test_escape_at_passes_through() {
        // `@` n'est PAS un opérateur BOOLEAN MODE.
        assert_eq!(escape_boolean_ft("@gmail.com"), "@gmail.com");
        assert_eq!(escape_boolean_ft("user@example.org"), "user@example.org");
    }

    #[test]
    fn test_escape_accents_preserved() {
        assert_eq!(escape_boolean_ft("Crémant"), "Crémant");
        assert_eq!(escape_boolean_ft("Société"), "Société");
        assert_eq!(escape_boolean_ft("Genève"), "Genève");
    }

    #[test]
    fn test_escape_empty_input() {
        assert_eq!(escape_boolean_ft(""), "");
    }

    #[test]
    fn test_escape_whitespace_only() {
        assert_eq!(escape_boolean_ft("   "), "");
        assert_eq!(escape_boolean_ft("\t\n  "), "");
    }

    #[test]
    fn test_escape_only_operators() {
        // Strip total → vide.
        assert_eq!(escape_boolean_ft("+-*\"~"), "");
        assert_eq!(escape_boolean_ft("(>)<"), "");
    }

    #[test]
    fn test_escape_short_token() {
        // Le helper ne juge pas la longueur — c'est le rôle de
        // `innodb_ft_min_token_size` (défaut 3) côté MariaDB.
        assert_eq!(escape_boolean_ft("de"), "de");
        assert_eq!(escape_boolean_ft("le"), "le");
        assert_eq!(escape_boolean_ft("a"), "a");
    }

    #[test]
    fn test_escape_unicode_general() {
        // Caractères chinois / cyrilliques / arabes : préservés.
        assert_eq!(escape_boolean_ft("北京"), "北京");
        assert_eq!(escape_boolean_ft("Москва"), "Москва");
        assert_eq!(escape_boolean_ft("القاهرة"), "القاهرة");
    }

    #[test]
    fn test_escape_percent_passes_through() {
        // `%` n'est PAS dans la strip-list (10 chars uniquement).
        // Le tokenizer InnoDB FULLTEXT l'ignore silencieusement.
        assert_eq!(escape_boolean_ft("100%"), "100%");
        assert_eq!(escape_boolean_ft("50% remise"), "50% remise");
    }

    #[test]
    fn test_escape_regex_chars_pass_through() {
        // `$ ^ [ ] | .` ne sont PAS des opérateurs BOOLEAN MODE.
        assert_eq!(escape_boolean_ft("foo$bar"), "foo$bar");
        assert_eq!(escape_boolean_ft("foo^bar"), "foo^bar");
        assert_eq!(escape_boolean_ft("foo|bar"), "foo|bar");
        assert_eq!(escape_boolean_ft("foo.bar"), "foo.bar");
    }

    #[test]
    fn test_escape_trim_then_strip() {
        // Le trim s'applique en premier, puis le strip.
        assert_eq!(escape_boolean_ft("  Mar  "), "Mar");
        assert_eq!(escape_boolean_ft("\t+foo+\n"), "foo");
    }

    #[test]
    fn test_escape_multi_words_preserved() {
        // L'espace n'est pas dans la strip-list — séparateur de tokens
        // côté MariaDB (sémantique optionnelle avec ranking).
        assert_eq!(escape_boolean_ft("foo bar baz"), "foo bar baz");
    }
}
