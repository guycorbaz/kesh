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
/// **Stratégie : remplacement par une ESPACE (pas escape, pas suppression)**. Le backslash-escaping
/// (`\+`, `\-`, etc.) en `BOOLEAN MODE` n'est pas garanti déterministe
/// selon la version MariaDB exacte. Le strip total donne un comportement
/// prévisible sur toutes versions 11.x : l'utilisateur tape du texte,
/// le helper retire les 10 caractères opérateurs, puis le repository
/// caller append un `*` en suffixe pour le prefix wildcard (préservation
/// de l'UX prefix-search).
///
/// **Caractères remplacés par une espace** : `+ - > < ( ) ~ * " \` (10).
/// ⚠️ Ils étaient SUPPRIMÉS jusqu'à l'issue #314, ce qui collait les tokens
/// voisins et rendait `Coop-Vaud` introuvable par son propre nom.
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
/// - Whitespace en entrée : `input.trim()` est appliqué (whitespace
///   leading/trailing). Les espaces internes multiples (`"foo  bar"`)
///   sont **conservés tels quels** — le tokenizer InnoDB FULLTEXT
///   les traite comme des séparateurs équivalents à un espace simple.
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
    // Issue #314 — ⚠️ REMPLACER par une espace, et non supprimer.
    //
    // La suppression collait les tokens de part et d'autre de l'opérateur :
    // `Coop-Vaud` devenait `CoopVaud`, puis la requête `CoopVaud*` — qui ne
    // matche NI `Coop` NI `Vaud`, les deux seuls tokens que l'index FULLTEXT
    // ait réellement produits. **Retaper un nom au caractère près ne trouvait
    // rien**, ce qui est le comble pour une recherche.
    //
    // Le cas est massif en Suisse dès que le nom est composé — `Müller-Weber`,
    // `Perrin-Jaquet` : le terme y dégénérait en recherche sur le seul prénom.
    //
    // `split_whitespace()` fait ici trois choses d'un coup, et c'est pourquoi
    // il est préféré à un `replace` suivi d'un `trim` : il replie les espaces
    // multiples (`a--b` → `a  b` → `a b`, sans token vide), retire les espaces
    // de bord nés d'un opérateur en tête ou en queue (`-abc` → ` abc` → `abc`),
    // et rend la chaîne vide quand l'entrée n'était QUE des opérateurs — cas
    // que les appelants gardent déjà (cf. `products.rs:127`).
    trimmed
        .chars()
        .map(|c| {
            if BOOLEAN_FT_OPERATORS.contains(&c) {
                ' '
            } else {
                c
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
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
        // ⚠️ Ce test assertait `"foobar"` — c'est-à-dire EXACTEMENT le défaut de
        // l'issue #314 : il verrouillait la suppression qui collait les tokens.
        // Un test peut figer un bug aussi solidement qu'il protège une règle.
        //
        // Pour chacun des 10 opérateurs, il est désormais remplacé par une
        // espace : les deux tokens restent séparés, donc trouvables.
        for op in BOOLEAN_FT_OPERATORS {
            let input = format!("foo{op}bar");
            assert_eq!(
                escape_boolean_ft(&input),
                "foo bar",
                "operator {op:?} non remplacé par une espace"
            );
        }
    }

    #[test]
    fn test_escape_strip_combined() {
        // Combinaison de plusieurs opérateurs (issue #314).
        assert_eq!(escape_boolean_ft("foo+*bar\"baz"), "foo bar baz");
        assert_eq!(escape_boolean_ft("(foo) -bar ~baz"), "foo bar baz");
        // ⚠️ Opérateurs ADJACENTS : deux espaces naîtraient du remplacement,
        // et un token vide en découlerait sans le repli. C'est le piège que
        // l'issue signale nommément.
        assert_eq!(escape_boolean_ft("a--b"), "a b");
        assert_eq!(escape_boolean_ft("a+-*b"), "a b");
        // Opérateur en tête ou en queue : l'espace née au bord disparaît.
        assert_eq!(escape_boolean_ft("-abc"), "abc");
        assert_eq!(escape_boolean_ft("abc-"), "abc");
    }

    #[test]
    fn test_escape_nom_compose_reste_trouvable() {
        // Issue #314 — le cas fondateur, et il est massif en Suisse dès que le
        // nom de famille est composé. Avant : `CoopVaud`, qui ne matche NI le
        // token `Coop` NI le token `Vaud` produits par l'index FULLTEXT ;
        // retaper le nom AU CARACTÈRE PRÈS ne trouvait rien.
        assert_eq!(escape_boolean_ft("Coop-Vaud"), "Coop Vaud");
        assert_eq!(escape_boolean_ft("Müller-Weber"), "Müller Weber");
        assert_eq!(escape_boolean_ft("Perrin-Jaquet"), "Perrin Jaquet");
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

    #[test]
    fn test_escape_internal_multiple_spaces_normalises() {
        // ⚠️ Changement assumé à l'issue #314 : les espaces internes multiples
        // sont désormais REPLIÉES, là où ce test les disait préservées.
        //
        // **Sans effet fonctionnel**, et ce test le disait déjà lui-même : le
        // tokenizer MariaDB « les traite comme un séparateur équivalent ». Le
        // repli est le prix du remplacement des opérateurs par une espace — on
        // ne peut pas distinguer une espace tapée par l'utilisateur d'une
        // espace née d'un `-`, et il faut bien replier les secondes.
        assert_eq!(escape_boolean_ft("foo  bar"), "foo bar");
        assert_eq!(escape_boolean_ft("a  b  c"), "a b c");
    }

    #[test]
    fn test_escape_trailing_leading_spaces_trimmed() {
        // Les espaces de bord disparaissent au `trim` initial, et les internes
        // sont repliées depuis #314 (cf. test ci-dessus).
        assert_eq!(escape_boolean_ft(" foo bar "), "foo bar");
        assert_eq!(escape_boolean_ft("\t Mar \n"), "Mar");
    }
}
