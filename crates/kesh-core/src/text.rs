//! Prédicats et normalisations de texte partagés (Story 22-1, #294/#295).
//!
//! Ce module est l'**unique source** du prédicat [`is_invisible`] — écrit deux
//! fois avant cette story (`kesh-api/src/routes/contacts.rs`,
//! `kesh-qrbill/src/pdf.rs`), sur une justification de non-dépendance réfutée
//! en revue — et le siège de la forme canonique du numéro de client,
//! [`canonical_key`].

use unicode_normalization::UnicodeNormalization;

/// Longueur maximale du numéro de client, en caractères — la borne que la
/// colonne `contacts.client_number_canonical` (`VARCHAR(50)`, migration
/// `20260814000001`) matérialise côté base.
///
/// ⚠️ Elle borne la **canonique**, pas seulement la saisie : [`canonical_key`]
/// peut ALLONGER une chaîne (NFKC décompose `ﬁ` en `fi`, `to_lowercase` étend
/// `İ` en `i`+U+0307 — 50 caractères saisis peuvent en canoniser 100). Tout
/// écrivain de la colonne vérifie la canonique contre cette constante — la
/// route à la saisie, le backfill sur le parc. *(Relevé en passe 1 de revue,
/// prouvé par exécution.)*
pub const CLIENT_NUMBER_MAX_CHARS: usize = 50;

/// Vrai si le caractère ne **marque** rien à l'écran ni à l'impression.
///
/// `char::is_whitespace` suit la propriété Unicode `White_Space`, qui n'inclut
/// **pas** les caractères de largeur nulle : `U+200B` (ZWSP), `U+FEFF` (BOM) et
/// `U+2060` (word joiner) passent donc `trim()` et `is_empty()` sans être
/// retenus. Une valeur qui n'en contient que de ceux-là est vide *à l'écran*
/// comme *à l'impression* — c'est ce que ce prédicat permet de détecter.
///
/// `U+00AD` (trait d'union conditionnel) est inclus : il ne se rend pas hors
/// point de césure.
pub fn is_invisible(c: char) -> bool {
    c.is_whitespace()
        || c.is_control()
        || matches!(c,
            '\u{00AD}' | '\u{200B}'..='\u{200F}' | '\u{2060}'..='\u{2064}' | '\u{FEFF}')
}

/// Vrai si le caractère est invisible **sans occuper de largeur** : le
/// sous-ensemble de [`is_invisible`] qui exclut les blancs.
///
/// ⚠️ La distinction est ce qui empêche [`canonical_key`] de fusionner
/// `"CLI 1"` et `"CLI1"` : un espace **marque** — il sépare visiblement deux
/// caractères — alors qu'un ZWSP ou un trait d'union conditionnel ne marquent
/// rien. Le retrait de l'étape 1 de D2 porte donc sur ce prédicat-ci, et non
/// sur `is_invisible` entier ; les blancs sont traités par NFKC (repli des
/// espaces exotiques) puis `trim()` (bords).
fn is_zero_width(c: char) -> bool {
    is_invisible(c) && !c.is_whitespace()
}

/// Forme canonique du numéro de client (décision **D2** de la Story 22-1).
///
/// Quatre étapes, **dans cet ordre** :
///
/// 1. retrait de tout caractère invisible de largeur nulle ([`is_zero_width`]) ;
/// 2. normalisation **NFKC** — replie les décompositions (`E`+U+0301 → `É`),
///    les formes de compatibilité (chiffres pleine chasse, ligatures) et les
///    espaces exotiques vers l'espace simple ;
/// 3. `trim()` ;
/// 4. repli de casse par `to_lowercase()`.
///
/// ⚠️ Le `trim()` vient en **troisième** : en tête, il s'arrêtait sur un
/// invisible de bord (`"CLI-1 ‹U+200B›"`) et laissait l'espace masqué dans la
/// canonique — le chemin d'attaque #294 réintroduit par l'algorithme, réfuté
/// par exécution en passe 1 de validate. Placé après le retrait des invisibles
/// **et** après NFKC, il nettoie aussi les blancs que NFKC vient de produire.
///
/// Une canonique **vide** signifie « valeur absente » : l'appelant stocke
/// `NULL` pour les deux colonnes (prolongement de la garde de vacuité 16-3b).
pub fn canonical_key(s: &str) -> String {
    s.chars()
        .filter(|c| !is_zero_width(*c))
        .nfkc()
        .collect::<String>()
        .trim()
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests de table d'AC1 — chaque ligne est `(gauche, droite, attendu)` où
    /// `attendu` dit si les deux canoniques doivent être ÉGALES.
    #[test]
    fn canonical_key_table() {
        let equal: &[(&str, &str)] = &[
            // casse
            ("CLI-1", "cli-1"),
            // accents composés vs décomposés (É NFC vs E+U+0301)
            ("CL\u{00C9}-1", "CLE\u{0301}-1"),
            // formes de compatibilité : chiffres pleine chasse
            ("CLI-\u{FF11}", "CLI-1"),
            // invisibles encastrés
            ("CLI-1", "CLI\u{200B}-1"),
            ("CLI-1", "CLI\u{FEFF}-1"),
            ("CLI-1", "CLI\u{2060}-1"),
            ("CLI-1", "CLI\u{00AD}-1"),
            // espaces de tête et de queue
            ("CLI-1", "  CLI-1  "),
            // ⚠️ le cas de BORD qui a réfuté l'ordre initial : un invisible en
            // bord masquant un espace — canonique "cli-1", PAS "cli-1 ".
            ("CLI-1", "CLI-1 \u{200B}"),
            // espace exotique replié par NFKC vers l'espace simple
            ("CLI 1", "CLI\u{00A0}1"),
        ];
        for (a, b) in equal {
            assert_eq!(
                canonical_key(a),
                canonical_key(b),
                "canoniques attendues ÉGALES pour {a:?} et {b:?}"
            );
        }

        let distinct: &[(&str, &str)] = &[
            // un accent DISTINGUE : deux clients légitimement différents (#295
            // ferme la fusion accidentelle, pas la distinction).
            ("CLI-\u{00C9}1", "CLI-E1"),
            // un espace INTERNE marque — "CLI 1" et "CLI1" sont visuellement
            // distincts, le retrait ne porte que sur la largeur nulle.
            ("CLI 1", "CLI1"),
        ];
        for (a, b) in distinct {
            assert_ne!(
                canonical_key(a),
                canonical_key(b),
                "canoniques attendues DISTINCTES pour {a:?} et {b:?}"
            );
        }
    }

    #[test]
    fn canonical_key_empty_means_absent() {
        // chaîne vide, valeur intégralement invisible, blancs seuls : la
        // canonique est vide — l'appelant traite la valeur comme absente.
        for v in [
            "",
            "\u{200B}\u{FEFF}",
            "   ",
            " \u{200B} ",
            "\u{00AD}\u{2060}",
        ] {
            assert!(
                canonical_key(v).is_empty(),
                "canonique attendue VIDE pour {v:?}"
            );
        }
        // mixte visible+invisible : la part visible survit.
        assert_eq!(canonical_key(" \u{200B}CLI\u{00AD}-1 "), "cli-1");
    }

    /// Revue passe 1 (CRITICAL) : la canonique peut être PLUS LONGUE que la
    /// saisie — propriété épinglée ici pour que la borne de stockage
    /// ([`CLIENT_NUMBER_MAX_CHARS`]) ne soit jamais raisonnée depuis la seule
    /// longueur d'entrée. Prouvé sur les deux chemins d'expansion : NFKC
    /// (ligature) et repli de casse (İ turc).
    #[test]
    fn canonical_key_can_be_longer_than_its_input() {
        assert_eq!(canonical_key("\u{FB01}"), "fi"); // ﬁ → fi (NFKC)
        assert_eq!(canonical_key("\u{0130}").chars().count(), 2); // İ → i + U+0307
        let fifty = "\u{FB01}".repeat(50);
        assert_eq!(fifty.chars().count(), 50);
        assert_eq!(canonical_key(&fifty).chars().count(), 100);
    }

    /// La mutation d'ordre d'AC1, épinglée par un test et non par un
    /// commentaire : rejouer l'ANCIEN ordre (trim d'abord) sur le cas de bord
    /// rend une canonique différente — si quelqu'un « simplifie » l'ordre de
    /// [`canonical_key`], `canonical_key_table` rougit sur ce même cas.
    #[test]
    fn the_old_order_is_really_wrong() {
        let v = "CLI-1 \u{200B}";
        let old_order: String = v
            .trim()
            .chars()
            .filter(|c| !is_zero_width(*c))
            .nfkc()
            .collect::<String>()
            .to_lowercase();
        assert_eq!(old_order, "cli-1 ", "l'ancien ordre laissait l'espace");
        assert_eq!(canonical_key(v), "cli-1", "l'ordre D2 le retire");
    }

    #[test]
    fn the_invisible_predicate_keeps_its_16_3b_semantics() {
        // Le prédicat déménagé garde exactement sa sémantique d'origine : les
        // blancs SONT invisibles (vacuité), la lettre ne l'est pas.
        for c in [
            ' ', '\t', '\u{00A0}', '\u{200B}', '\u{FEFF}', '\u{00AD}', '\u{7F}',
        ] {
            assert!(is_invisible(c), "{c:?} doit être invisible");
        }
        for c in ['a', 'É', '1', '-'] {
            assert!(!is_invisible(c), "{c:?} ne doit pas être invisible");
        }
    }
}
