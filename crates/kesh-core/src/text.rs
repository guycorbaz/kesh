//! Prédicats et normalisations de texte partagés (Story 22-1, #294/#295).
//!
//! Ce module est l'**unique source** du prédicat [`is_invisible`] — écrit deux
//! fois avant cette story (`kesh-api/src/routes/contacts.rs`,
//! `kesh-qrbill/src/pdf.rs`), sur une justification de non-dépendance réfutée
//! en revue — et le siège de la forme canonique du numéro de client,
//! [`canonical_key`].
//!
//! Il porte aussi, depuis la Story 32-8b0, le prédicat partagé des
//! **caractères cachés** — [`is_hidden_format`] — et ses deux emplois :
//! [`is_hidden_in_text`], pour un texte sur plusieurs lignes (un envoi du
//! serveur MCP, le texte d'un prompt), et [`is_hidden_in_line`], pour une
//! valeur d'une ligne (la description d'un compte bancaire, celle d'un prompt).

use unicode_normalization::UnicodeNormalization;
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};

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

/// Vrai si le caractère est **caché** : il peut modifier ce qu'un texte dit ou
/// la façon dont il s'affiche sans se voir lui-même — Story 32-8b0, selon
/// l'AC-ii1 de la fiche 32-10a.
///
/// Deux voies, et seulement deux :
///
/// 1. la **catégorie générale** Unicode du caractère est `Format` (Cf : marques
///    bidirectionnelles, espaces de largeur nulle, étiquettes, marques de
///    format de certaines écritures), `PrivateUse` (Co : sans signification
///    convenue, tel le logo Apple U+F8FF) ou `Unassigned` (Cn : non attribué —
///    un point absent de la table de la crate rend `Unassigned`, si bien qu'un
///    caractère attribué par une version d'Unicode postérieure à celle de la
///    crate est refusé, le cas strict) ;
/// 2. le caractère appartient à l'une des **deux listes écrites** ci-dessous,
///    d'invisibles rangés par Unicode dans une autre catégorie : les
///    remplissages `Lo` (hangûl), les marques `Mn` sans rendu (gluon de
///    graphèmes, voyelles inhérentes khmères, sélecteurs de variante), et les
///    séparateurs de ligne et de paragraphe (`Zl`, `Zp`).
///
/// ⚠️ **Les sélecteurs de variante U+FE0E et U+FE0F et le liant U+200D sont
/// refusés**, si bien qu'un emoji **composé** (`⚠️` = U+26A0 U+FE0F,
/// `👨‍💻` = U+1F468 U+200D U+1F4BB) l'est aussi : admis, ces trois caractères
/// formeraient un alphabet invisible à glisser après n'importe quelle lettre.
/// Un emoji **simple** (U+26A0, U+1F4BB : `OtherSymbol`) passe.
///
/// Les contrôles `Cc` (`\n`, `\t`, U+0000…) n'en relèvent **pas** : c'est à
/// l'emploi de dire lesquels il admet — [`is_hidden_in_text`] admet `\n` et
/// `\t`, [`is_hidden_in_line`] n'en admet aucun.
///
/// **Angle mort, qualifié** : la classification est celle d'Unicode 17.0.0,
/// la version de `unicode-properties` 0.1.4 ; reste admis un invisible d'une
/// catégorie **autre** que Cf, Co, Cn que les listes ne nomment pas (un `Mn`,
/// un `Lo` ou un `Zs` sans rendu), et les homoglyphes, qui sont visibles.
pub fn is_hidden_format(c: char) -> bool {
    matches!(
        c.general_category(),
        GeneralCategory::Format | GeneralCategory::PrivateUse | GeneralCategory::Unassigned
    ) || matches!(
        c,
        // Remplissages `Lo` : choseong et jungseong de remplissage, remplissage
        // hangûl, remplissage hangûl demi-chasse.
        '\u{115F}' | '\u{1160}' | '\u{3164}' | '\u{FFA0}'
        // Marques `Mn` sans rendu : gluon de graphèmes ; voyelles inhérentes
        // khmères ; sélecteurs de variante mongols ; sélecteurs de variante et
        // leur supplément.
        | '\u{034F}'
        | '\u{17B4}' | '\u{17B5}'
        | '\u{180B}'..='\u{180D}' | '\u{180F}'
        | '\u{FE00}'..='\u{FE0F}'
        | '\u{E0100}'..='\u{E01EF}'
        // Séparateurs de ligne (`Zl`) et de paragraphe (`Zp`).
        | '\u{2028}' | '\u{2029}'
    )
}

/// Vrai si le caractère est refusé dans un **texte sur plusieurs lignes** :
/// tout contrôle `Cc` sauf le saut de ligne `\n` et la tabulation `\t`, et
/// tout caractère de [`is_hidden_format`] — Story 32-8b0.
///
/// C'est le prédicat d'un envoi du serveur MCP (32-8b) et du texte d'un prompt
/// (32-10a-ii, qui le nommait `is_forbidden_in_prompt`) : une seule définition,
/// sous un nom qui ne dit ni l'un ni l'autre. Le retour chariot `\r` est
/// refusé : un texte qui en porte se normalise avant le contrôle (`\r\n` →
/// `\n`), il ne se corrige pas ici.
pub fn is_hidden_in_text(c: char) -> bool {
    (c.is_control() && c != '\n' && c != '\t') || is_hidden_format(c)
}

/// Vrai si le caractère est refusé dans une **valeur d'une ligne** : tout
/// contrôle `Cc`, `\n` et `\t` compris, et tout caractère de
/// [`is_hidden_format`] — Story 32-8b0.
///
/// C'est le prédicat de la description d'un compte bancaire
/// (`kesh-api/src/routes/bank_accounts.rs`) et de la description d'un prompt
/// (32-10a-ii) : une seule définition, publique, au lieu d'une fonction privée
/// d'une route que l'autre devrait recopier.
pub fn is_hidden_in_line(c: char) -> bool {
    c.is_control() || is_hidden_format(c)
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

    // ── Story 32-8b0 : le prédicat des caractères cachés (AC-01, AC-02, AC-04) ──

    /// `char` d'un point de code écrit en littéral — panique s'il n'en est pas un.
    fn ch(cp: u32) -> char {
        char::from_u32(cp).unwrap_or_else(|| panic!("U+{cp:04X} n'est pas un char"))
    }

    /// Vrai si une chaîne contient au moins un caractère caché d'un texte.
    fn texte_cache(s: &str) -> bool {
        s.chars().any(is_hidden_in_text)
    }

    /// L'angle mort de [`is_hidden_format`] est daté par la version d'Unicode
    /// de la crate : une mise à jour de `unicode-properties` fait rougir ce
    /// test, et c'est le moment de relire les deux listes écrites.
    #[test]
    fn version_d_unicode_epinglee() {
        assert_eq!(unicode_properties::UNICODE_VERSION, (17, 0, 0));
    }

    /// Voie 1, `Format` (Cf) : un point de chaque table de contrôles de format
    /// attribués que l'ancienne liste de `bank_accounts.rs` omettait (M01).
    #[test]
    fn famille_cf_refusee() {
        for cp in [
            0x0600, 0x0605, 0x06DD, 0x070F, 0x0890, 0x0891, 0x08E2, 0x110BD, 0x110CD, 0x13430,
            0x1343F, 0x1BCA0, 0x1BCA3, 0x1D173, 0x1D17A,
        ] {
            let c = ch(cp);
            assert_eq!(c.general_category(), GeneralCategory::Format, "U+{cp:04X}");
            assert!(is_hidden_format(c), "U+{cp:04X} (Cf) doit être caché");
        }
    }

    /// Voie 1, `PrivateUse` (Co) : les trois plages, dont le logo Apple (M03).
    #[test]
    fn famille_co_refusee() {
        for cp in [0xE000, 0xF8FF, 0xF0000, 0xFFFFD, 0x100000, 0x10FFFD] {
            let c = ch(cp);
            assert_eq!(
                c.general_category(),
                GeneralCategory::PrivateUse,
                "U+{cp:04X}"
            );
            assert!(is_hidden_format(c), "U+{cp:04X} (Co) doit être caché");
        }
    }

    /// Voie 1, `Unassigned` (Cn) : un point absent de la table rend
    /// `Unassigned` ; U+2065 et les étiquettes non attribuées en sont (M02).
    #[test]
    fn famille_cn_refusee() {
        for cp in [0x0378, 0x2065, 0xE0000, 0xE0002, 0xE001F] {
            let c = ch(cp);
            assert_eq!(
                c.general_category(),
                GeneralCategory::Unassigned,
                "U+{cp:04X}"
            );
            assert!(is_hidden_format(c), "U+{cp:04X} (Cn) doit être caché");
        }
    }

    /// Voie 2 : chaque point ou borne des deux listes écrites, avec la
    /// catégorie qui les met hors de la voie 1 (M04 pour U+E0100–U+E01EF).
    #[test]
    fn listes_ecrites_refusees() {
        let cas: &[(u32, GeneralCategory)] = &[
            (0x115F, GeneralCategory::OtherLetter),
            (0x1160, GeneralCategory::OtherLetter),
            (0x3164, GeneralCategory::OtherLetter),
            (0xFFA0, GeneralCategory::OtherLetter),
            (0x034F, GeneralCategory::NonspacingMark),
            (0x17B4, GeneralCategory::NonspacingMark),
            (0x17B5, GeneralCategory::NonspacingMark),
            (0x180B, GeneralCategory::NonspacingMark),
            (0x180C, GeneralCategory::NonspacingMark),
            (0x180D, GeneralCategory::NonspacingMark),
            (0x180F, GeneralCategory::NonspacingMark),
            (0xFE00, GeneralCategory::NonspacingMark),
            (0xFE0E, GeneralCategory::NonspacingMark),
            (0xFE0F, GeneralCategory::NonspacingMark),
            (0xE0100, GeneralCategory::NonspacingMark),
            (0xE01EF, GeneralCategory::NonspacingMark),
            (0x2028, GeneralCategory::LineSeparator),
            (0x2029, GeneralCategory::ParagraphSeparator),
        ];
        for &(cp, categorie) in cas {
            let c = ch(cp);
            assert_eq!(c.general_category(), categorie, "U+{cp:04X}");
            assert!(is_hidden_format(c), "U+{cp:04X} (liste) doit être caché");
        }
    }

    /// Les contrôles bidirectionnels et les étiquettes, refusés.
    #[test]
    fn bidi_et_etiquettes_refuses() {
        for cp in [0x202E, 0x2066, 0xE0041] {
            assert!(is_hidden_format(ch(cp)), "U+{cp:04X} doit être caché");
            assert!(is_hidden_in_text(ch(cp)), "U+{cp:04X} refusé dans un texte");
        }
    }

    /// Les voisins **visibles** des listes, de part et d'autre, sont admis :
    /// la liste nomme des points, non des plages qui déborderaient.
    #[test]
    fn voisins_visibles_admis() {
        for cp in [0x1161, 0x17B6, 0x180A, 0xFE10] {
            let c = ch(cp);
            assert!(!is_hidden_format(c), "U+{cp:04X} doit être admis");
            assert!(!is_hidden_in_text(c), "U+{cp:04X} admis dans un texte");
            assert!(!is_hidden_in_line(c), "U+{cp:04X} admis dans une ligne");
        }
    }

    /// U+FE0E, U+FE0F et U+200D refusés isolés ; un emoji simple admis, un
    /// emoji composé refusé (M05 : U+FE0F ; M06 : U+200D).
    #[test]
    fn emojis_simples_admis_composes_refuses() {
        for cp in [0xFE0E, 0xFE0F, 0x200D] {
            assert!(is_hidden_format(ch(cp)), "U+{cp:04X} isolé doit être caché");
        }
        for simple in ["\u{26A0}", "\u{1F4BB}"] {
            assert!(!texte_cache(simple), "{simple:?} (emoji simple) admis");
            assert!(
                !simple.chars().any(is_hidden_in_line),
                "{simple:?} admis dans une ligne"
            );
        }
        // ⚠️ = U+26A0 U+FE0F ; 👨‍💻 = U+1F468 U+200D U+1F4BB.
        for compose in ["\u{26A0}\u{FE0F}", "\u{1F468}\u{200D}\u{1F4BB}"] {
            assert!(texte_cache(compose), "{compose:?} (emoji composé) refusé");
            assert!(
                compose.chars().any(is_hidden_in_line),
                "{compose:?} refusé dans une ligne"
            );
        }
    }

    /// `is_hidden_in_text` : `\n` et `\t` admis, tout autre contrôle et tout
    /// caché refusé ; l'espace insécable admis (M07 : `\r`).
    #[test]
    fn texte_sur_plusieurs_lignes() {
        assert!('\n'.is_control(), "\\n est bien un contrôle Cc");
        for c in ['\n', '\t', '\u{00A0}', 'a', 'É', ' '] {
            assert!(!is_hidden_in_text(c), "{c:?} admis dans un texte");
        }
        for c in [
            '\r',
            '\u{0000}',
            '\u{202E}',
            '\u{2066}',
            '\u{200B}',
            '\u{FEFF}',
            '\u{E0100}',
        ] {
            assert!(is_hidden_in_text(c), "{c:?} refusé dans un texte");
        }
    }

    /// `is_hidden_in_line` : tout contrôle refusé, `\n` et `\t` compris.
    #[test]
    fn valeur_d_une_ligne() {
        for c in [
            '\n', '\t', '\r', '\u{0000}', '\u{0085}', '\u{2028}', '\u{F8FF}',
        ] {
            assert!(is_hidden_in_line(c), "{c:?} refusé dans une ligne");
        }
        for c in ['a', 'É', ' ', '\u{00A0}', '-', '’'] {
            assert!(!is_hidden_in_line(c), "{c:?} admis dans une ligne");
        }
    }

    /// Non-régression, déménagé de `kesh-api/src/routes/bank_accounts.rs`
    /// (`ensemble_refuse_borne_par_borne`, Story 26-2a) : **chaque point** de
    /// l'ancienne liste fermée de la description d'un compte reste caché. Le
    /// contrôle des voisins est retiré : la catégorie le rend sans objet.
    #[test]
    fn ancienne_liste_reste_refusee() {
        let plages: &[(u32, u32)] = &[
            (0x00AD, 0x00AD),
            (0x034F, 0x034F),
            (0x061C, 0x061C),
            (0x180E, 0x180E),
            (0x2028, 0x2029),
            (0x200B, 0x200F),
            (0x202A, 0x202E),
            (0x2060, 0x2064),
            (0x2066, 0x2069),
            (0x206A, 0x206F),
            (0x3164, 0x3164),
            (0xFE00, 0xFE0F),
            (0xFEFF, 0xFEFF),
            (0xFFF9, 0xFFFB),
            (0xE0000, 0xE007F),
        ];
        for &(debut, fin) in plages {
            for cp in debut..=fin {
                assert!(is_hidden_format(ch(cp)), "U+{cp:04X} doit rester caché");
            }
        }
    }
}
