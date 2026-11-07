//! Le texte d'une QR-facture à ses deux niveaux : le code, puis l'imprimé qui en
//! dérive (Story 33-12a, #941, C-e3312-1).
//!
//! La directive (`docs/six-references/ig-qr-bill-v2.4-en.pdf`) fixe le jeu de
//! caractères au § 4.1.1 et exige au § 3.6.2 que les valeurs imprimées soient
//! « exactly matching those in the Swiss QR Code ». Helvetica intégrée, en
//! WinAnsi, ne porte pourtant pas 125 lettres du § 4.1.1, et `lopdf` les retire
//! **en silence** à l'encodage (`Čolić` s'imprimait `oli`).
//!
//! Le sens est donc unique, du code vers l'imprimé :
//! - [`texte_code`] normalise la typographie et les invisibles, sans toucher à
//!   aucune lettre ;
//! - [`texte_imprime`] part de ce résultat et replie les seules lettres que
//!   WinAnsi ne porte pas sur leur base.
//!
//! ⚠️ L'écart qui reste à § 3.6.2 est écrit et assumé : ces 125 lettres s'impriment
//! par leur base alors que le code les garde. Une police embarquée le lèvera (#943).

/// Le texte d'un champ tel qu'il entre dans le code QR, à partir d'un texte déjà
/// validé (`validation::validate`).
///
/// U+2019 devient `'`, U+2014 devient `-`, U+00A0 devient une espace, U+00AD est
/// retiré ; puis les blancs de bordure sont rognés. Le rognage ne retire que
/// U+0020, seul blanc restant à ce stade (le § 4.1.1 n'en admet pas d'autre) : une
/// rue d'invisibles sort vide, un NPA ne commence pas par un blanc. Aucune lettre
/// n'est touchée, et la sortie n'est jamais plus longue que l'entrée.
pub(crate) fn texte_code(s: &str) -> String {
    let normalise: String = s
        .chars()
        .filter_map(|c| match c {
            '\u{2019}' => Some('\''),
            '\u{2014}' => Some('-'),
            '\u{00A0}' => Some(' '),
            '\u{00AD}' => None,
            autre => Some(autre),
        })
        .collect();
    normalise.trim_matches(' ').to_string()
}

/// Le texte d'un champ tel qu'il s'imprime sur la section de paiement et, au corps, pour
/// le nom et l'adresse : [`texte_code`], puis le repli des lettres hors WinAnsi sur leur base.
///
/// Les lettres que WinAnsi porte (é, Š, Œ, €…) restent telles quelles. `Ĳ` rend
/// `IJ` : l'imprimé peut donc être plus long que le code, ce qui ne touche ni les
/// bornes du code ni sa validation.
pub(crate) fn texte_imprime(s: &str) -> String {
    let mut sortie = String::with_capacity(s.len());
    for c in texte_code(s).chars() {
        match repli(c) {
            Some(base) => sortie.push_str(base),
            None => sortie.push(c),
        }
    }
    sortie
}

/// La base d'une lettre du § 4.1.1 que WinAnsi ne porte pas : table close de
/// 125 lettres, 121 de Latin Extended-A et les quatre de U+0218 à U+021B.
/// `None` pour tout le reste.
fn repli(c: char) -> Option<&'static str> {
    Some(match c {
        'Ā' | 'Ă' | 'Ą' => "A",
        'ā' | 'ă' | 'ą' => "a",
        'Ć' | 'Ĉ' | 'Ċ' | 'Č' => "C",
        'ć' | 'ĉ' | 'ċ' | 'č' => "c",
        'Ď' | 'Đ' => "D",
        'ď' | 'đ' => "d",
        'Ē' | 'Ĕ' | 'Ė' | 'Ę' | 'Ě' => "E",
        'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' => "e",
        'Ĝ' | 'Ğ' | 'Ġ' | 'Ģ' => "G",
        'ĝ' | 'ğ' | 'ġ' | 'ģ' => "g",
        'Ĥ' | 'Ħ' => "H",
        'ĥ' | 'ħ' => "h",
        'Ĩ' | 'Ī' | 'Ĭ' | 'Į' | 'İ' => "I",
        'ĩ' | 'ī' | 'ĭ' | 'į' | 'ı' => "i",
        'Ĳ' => "IJ",
        'ĳ' => "ij",
        'Ĵ' => "J",
        'ĵ' => "j",
        'Ķ' => "K",
        'ķ' | 'ĸ' => "k",
        'Ĺ' | 'Ļ' | 'Ľ' | 'Ŀ' | 'Ł' => "L",
        'ĺ' | 'ļ' | 'ľ' | 'ŀ' | 'ł' => "l",
        'Ń' | 'Ņ' | 'Ň' | 'Ŋ' => "N",
        'ń' | 'ņ' | 'ň' | 'ŉ' | 'ŋ' => "n",
        'Ō' | 'Ŏ' | 'Ő' => "O",
        'ō' | 'ŏ' | 'ő' => "o",
        'Ŕ' | 'Ŗ' | 'Ř' => "R",
        'ŕ' | 'ŗ' | 'ř' => "r",
        'Ś' | 'Ŝ' | 'Ş' | 'Ș' => "S",
        'ś' | 'ŝ' | 'ş' | 'ș' | 'ſ' => "s",
        'Ţ' | 'Ť' | 'Ŧ' | 'Ț' => "T",
        'ţ' | 'ť' | 'ŧ' | 'ț' => "t",
        'Ũ' | 'Ū' | 'Ŭ' | 'Ů' | 'Ű' | 'Ų' => "U",
        'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' => "u",
        'Ŵ' => "W",
        'ŵ' => "w",
        'Ŷ' => "Y",
        'ŷ' => "y",
        'Ź' | 'Ż' => "Z",
        'ź' | 'ż' => "z",
        _ => return None,
    })
}

/// Le § 4.1.1, plages recopiées du PDF de la directive. Écrites ici à la main, et
/// non dérivées du code : c'est ce qui rend les balayages non circulaires.
#[cfg(test)]
pub(crate) const PLAGES_4_1_1: &[(u32, u32)] = &[
    (0x0020, 0x007E),
    (0x00A0, 0x00FF),
    (0x0100, 0x017F),
    (0x0218, 0x021B),
    (0x20AC, 0x20AC),
];

/// Chaque caractère du § 4.1.1, dans l'ordre.
#[cfg(test)]
pub(crate) fn caracteres_4_1_1() -> Vec<char> {
    PLAGES_4_1_1
        .iter()
        .flat_map(|&(bas, haut)| bas..=haut)
        .map(|cp| char::from_u32(cp).expect("point de code valide"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use printpdf::lopdf::Document;

    /// Les 125 lettres, regroupées par base : liste écrite à part de [`repli`].
    const LETTRES_REPLIEES: &[(&str, &str)] = &[
        ("ĀĂĄ", "A"),
        ("āăą", "a"),
        ("ĆĈĊČ", "C"),
        ("ćĉċč", "c"),
        ("ĎĐ", "D"),
        ("ďđ", "d"),
        ("ĒĔĖĘĚ", "E"),
        ("ēĕėęě", "e"),
        ("ĜĞĠĢ", "G"),
        ("ĝğġģ", "g"),
        ("ĤĦ", "H"),
        ("ĥħ", "h"),
        ("ĨĪĬĮİ", "I"),
        ("ĩīĭįı", "i"),
        ("Ĳ", "IJ"),
        ("ĳ", "ij"),
        ("Ĵ", "J"),
        ("ĵ", "j"),
        ("Ķ", "K"),
        ("ķĸ", "k"),
        ("ĹĻĽĿŁ", "L"),
        ("ĺļľŀł", "l"),
        ("ŃŅŇŊ", "N"),
        ("ńņňŉŋ", "n"),
        ("ŌŎŐ", "O"),
        ("ōŏő", "o"),
        ("ŔŖŘ", "R"),
        ("ŕŗř", "r"),
        ("ŚŜŞȘ", "S"),
        ("śŝşșſ", "s"),
        ("ŢŤŦȚ", "T"),
        ("ţťŧț", "t"),
        ("ŨŪŬŮŰŲ", "U"),
        ("ũūŭůűų", "u"),
        ("Ŵ", "W"),
        ("ŵ", "w"),
        ("Ŷ", "Y"),
        ("ŷ", "y"),
        ("ŹŻ", "Z"),
        ("źż", "z"),
    ];

    /// **AC3** — paires littérales de la normalisation et du rognage.
    #[test]
    fn texte_code_paires_litterales() {
        for (entree, attendu) in [
            ("Grand\u{2019}Rue", "Grand'Rue"),
            ("Paul\u{2014}Henri", "Paul-Henri"),
            ("a\u{A0}b", "a b"),
            ("a\u{AD}b", "ab"),
            (" ", ""),
            ("\u{A0}", ""),
            ("\u{A0}x", "x"),
            ("x ", "x"),
            ("\u{AD}", ""),
            (" \u{AD} ", ""),
            // L'ordre : le retrait d'abord, sans quoi une espace de bordure resterait.
            ("\u{AD} x", "x"),
            ("x \u{AD}", "x"),
            ("a b", "a b"),
            ("Čolić & Söhne", "Čolić & Söhne"),
        ] {
            assert_eq!(texte_code(entree), attendu, "entrée {entree:?}");
        }
    }

    /// **AC3** — chaque caractère admis, pris seul : inchangé, sauf les quatre des
    /// paires et U+0020 ; la sortie reste au § 4.1.1 et jamais plus longue.
    #[test]
    fn texte_code_balayage_du_jeu() {
        for c in caracteres_4_1_1() {
            let entree = c.to_string();
            let sortie = texte_code(&entree);
            let attendu = match c {
                ' ' | '\u{A0}' | '\u{AD}' => String::new(),
                autre => autre.to_string(),
            };
            assert_eq!(sortie, attendu, "U+{:04X}", c as u32);
            assert!(sortie.chars().count() <= 1, "U+{:04X}", c as u32);
            assert!(
                sortie.chars().all(|s| caracteres_4_1_1().contains(&s)),
                "U+{:04X}",
                c as u32
            );
        }
    }

    /// **AC4** — la table de repli, complète : 121 + 4 lettres.
    #[test]
    fn texte_imprime_table_de_paires_complete() {
        let total: usize = LETTRES_REPLIEES
            .iter()
            .map(|(lettres, _)| lettres.chars().count())
            .sum();
        assert_eq!(total, 125, "montage : 121 + 4 lettres");
        for (lettres, base) in LETTRES_REPLIEES {
            for c in lettres.chars() {
                assert_eq!(texte_imprime(&c.to_string()), *base, "{c}");
            }
        }
        let etendu_a = LETTRES_REPLIEES
            .iter()
            .flat_map(|(lettres, _)| lettres.chars())
            .filter(|c| (0x0100..=0x017F).contains(&(*c as u32)))
            .count();
        assert_eq!(etendu_a, 121, "montage : Latin Extended-A");
    }

    /// **AC4** — chaque caractère admis, pris seul, s'imprime : encodable en WinAnsi,
    /// relu identique, non vide sauf U+0020, U+00A0, U+00AD, sans `?` sauf pour `?`.
    #[test]
    fn texte_imprime_balayage_encodable_en_winansi() {
        for c in caracteres_4_1_1() {
            let cp = c as u32;
            let sortie = texte_imprime(&c.to_string());
            if matches!(c, ' ' | '\u{A0}' | '\u{AD}') {
                assert_eq!(sortie, "", "U+{cp:04X}");
                continue;
            }
            assert!(!sortie.is_empty(), "U+{cp:04X} vidé");
            if c != '?' {
                assert!(!sortie.contains('?'), "U+{cp:04X} rend un `?`");
            }
            let octets = Document::encode_text(Some("WinAnsiEncoding"), &sortie);
            assert_eq!(
                Document::decode_text(Some("WinAnsiEncoding"), &octets),
                sortie,
                "U+{cp:04X} perdu à l'encodage"
            );
        }
    }

    /// **AC4, revue P1** — tout caractère du § 4.1.1 hors des 125 lettres sort de
    /// `texte_imprime` comme de `texte_code` : le repli ne touche rien d'autre.
    #[test]
    fn texte_imprime_identite_hors_des_125_lettres() {
        let repliees: Vec<char> = LETTRES_REPLIEES
            .iter()
            .flat_map(|(lettres, _)| lettres.chars())
            .collect();
        let mut compte = 0;
        for c in caracteres_4_1_1()
            .into_iter()
            .filter(|c| !repliees.contains(c))
        {
            let entree = c.to_string();
            assert_eq!(
                texte_imprime(&entree),
                texte_code(&entree),
                "U+{:04X}",
                c as u32
            );
            compte += 1;
        }
        assert_eq!(compte, 324 - 125, "montage : 199 caractères");
    }

    /// **AC4** — les lettres que WinAnsi porte restent ; le repli part de `texte_code`.
    #[test]
    fn texte_imprime_garde_ce_que_winansi_porte() {
        for (entree, attendu) in [
            ("Łódź", "Lódz"),
            ("Čolić", "Colic"),
            ("Šéf Œuvre Ÿ Ž €", "Šéf Œuvre Ÿ Ž €"),
            ("Ștefan\u{A0}Țară\u{AD}", "Stefan Tara"),
            ("Ĳssel", "IJssel"),
            ("Grand\u{2019}Rue", "Grand'Rue"),
        ] {
            assert_eq!(texte_imprime(entree), attendu, "entrée {entree:?}");
        }
    }
}
