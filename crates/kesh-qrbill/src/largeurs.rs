//! La largeur imprimée d'un texte en Helvetica et Helvetica-Bold, les deux polices
//! intégrées du PDF (Story 36-12a, #944, D4 ; C-36-12-4, C-36-12-10, C-36-12a-2).
//!
//! Le PDF s'écrit en WinAnsi : printpdf pose `WinAnsiEncoding` sur toute police intégrée
//! (`printpdf-0.7.0/src/font.rs:76`), et `lopdf` passe chaque caractère à son octet par
//! `WIN_ANSI_ENCODING` (`lopdf-0.31.0/src/encodings/mappings.rs:521`), en retirant en silence
//! ceux qui n'en ont pas. La largeur se lit donc en deux temps : le caractère donne son
//! octet et le **nom de glyphe** que WinAnsi y place ([`WINANSI`]), le nom donne la chasse
//! `WX` de l'AFM, en millièmes d'em ([`CHASSES`]).
//!
//! **Provenance.** Les deux tableaux sont engendrés, non écrits à la main, par le script
//! versionné `scripts/engendrer-largeurs-helvetica.py`, depuis :
//! - `WIN_ANSI_ENCODING` et `glyphnames.rs` de `lopdf` 0.31.0 ;
//! - les métriques d'Adobe de 1990 distribuées par TeX Live (paquet
//!   `texlive-fonts-recommended`), `fonts/afm/adobe/helvetic/phvr8a.afm` (Helvetica) et
//!   `phvb8a.afm` (Helvetica-Bold), dont le script porte les empreintes, lues **par nom**
//!   (`N <nom>`) : leur colonne `C` est l'*AdobeStandardEncoding*, non WinAnsi
//!   (`C 39 ; … N quoteright`, alors que l'octet 0x27 de WinAnsi est `quotesingle`).
//!
//! Pour vérifier que ce fichier porte bien la sortie du script, depuis la racine du dépôt :
//! `python3 -I scripts/engendrer-largeurs-helvetica.py --verifier crates/kesh-qrbill/src/largeurs.rs`
//! (code 0 si les tableaux sont identiques). Sans argument, le script écrit les tableaux sur
//! la sortie standard, à recopier ici entre les deux `#[rustfmt::skip]`.
//!
//! Seul ajout à la main : **`Euro`**, absent des AFM de 1990, reçoit **556** en maigre et en
//! gras, sa chasse dans les métriques Adobe actuelles d'Helvetica. Pour un caractère porté
//! par plusieurs octets (U+2022 aux octets 0x7F, 0x81…), le premier octet est retenu, comme
//! `lopdf::encodings::string_to_bytes`.
//!
//! ⚠️ **La table vaut pour ces deux polices seulement.** Le jour où le PDF embarque une autre
//! police (#943), elle est à engendrer de nouveau, depuis les métriques de celle-ci.

/// Millimètres par point typographique (1 pt = 1/72 pouce).
const MM_PAR_POINT: f32 = 25.4 / 72.0;

/// Le caractère porté par chaque octet WinAnsi de 0x20 à 0xFF, avec le nom de son glyphe ;
/// un caractère n'y figure qu'une fois, à son premier octet. Engendré (voir le module).
#[rustfmt::skip]
pub(crate) const WINANSI: [(u8, char, &str); 216] = [
    (0x20, ' ', "space"),
    (0x21, '!', "exclam"),
    (0x22, '"', "quotedbl"),
    (0x23, '#', "numbersign"),
    (0x24, '$', "dollar"),
    (0x25, '%', "percent"),
    (0x26, '&', "ampersand"),
    (0x27, '\'', "quotesingle"),
    (0x28, '(', "parenleft"),
    (0x29, ')', "parenright"),
    (0x2A, '*', "asterisk"),
    (0x2B, '+', "plus"),
    (0x2C, ',', "comma"),
    (0x2D, '-', "hyphen"),
    (0x2E, '.', "period"),
    (0x2F, '/', "slash"),
    (0x30, '0', "zero"),
    (0x31, '1', "one"),
    (0x32, '2', "two"),
    (0x33, '3', "three"),
    (0x34, '4', "four"),
    (0x35, '5', "five"),
    (0x36, '6', "six"),
    (0x37, '7', "seven"),
    (0x38, '8', "eight"),
    (0x39, '9', "nine"),
    (0x3A, ':', "colon"),
    (0x3B, ';', "semicolon"),
    (0x3C, '<', "less"),
    (0x3D, '=', "equal"),
    (0x3E, '>', "greater"),
    (0x3F, '?', "question"),
    (0x40, '@', "at"),
    (0x41, 'A', "A"),
    (0x42, 'B', "B"),
    (0x43, 'C', "C"),
    (0x44, 'D', "D"),
    (0x45, 'E', "E"),
    (0x46, 'F', "F"),
    (0x47, 'G', "G"),
    (0x48, 'H', "H"),
    (0x49, 'I', "I"),
    (0x4A, 'J', "J"),
    (0x4B, 'K', "K"),
    (0x4C, 'L', "L"),
    (0x4D, 'M', "M"),
    (0x4E, 'N', "N"),
    (0x4F, 'O', "O"),
    (0x50, 'P', "P"),
    (0x51, 'Q', "Q"),
    (0x52, 'R', "R"),
    (0x53, 'S', "S"),
    (0x54, 'T', "T"),
    (0x55, 'U', "U"),
    (0x56, 'V', "V"),
    (0x57, 'W', "W"),
    (0x58, 'X', "X"),
    (0x59, 'Y', "Y"),
    (0x5A, 'Z', "Z"),
    (0x5B, '[', "bracketleft"),
    (0x5C, '\\', "backslash"),
    (0x5D, ']', "bracketright"),
    (0x5E, '^', "asciicircum"),
    (0x5F, '_', "underscore"),
    (0x60, '`', "grave"),
    (0x61, 'a', "a"),
    (0x62, 'b', "b"),
    (0x63, 'c', "c"),
    (0x64, 'd', "d"),
    (0x65, 'e', "e"),
    (0x66, 'f', "f"),
    (0x67, 'g', "g"),
    (0x68, 'h', "h"),
    (0x69, 'i', "i"),
    (0x6A, 'j', "j"),
    (0x6B, 'k', "k"),
    (0x6C, 'l', "l"),
    (0x6D, 'm', "m"),
    (0x6E, 'n', "n"),
    (0x6F, 'o', "o"),
    (0x70, 'p', "p"),
    (0x71, 'q', "q"),
    (0x72, 'r', "r"),
    (0x73, 's', "s"),
    (0x74, 't', "t"),
    (0x75, 'u', "u"),
    (0x76, 'v', "v"),
    (0x77, 'w', "w"),
    (0x78, 'x', "x"),
    (0x79, 'y', "y"),
    (0x7A, 'z', "z"),
    (0x7B, '{', "braceleft"),
    (0x7C, '|', "bar"),
    (0x7D, '}', "braceright"),
    (0x7E, '~', "asciitilde"),
    (0x7F, '\u{2022}', "bullet"),
    (0x80, '\u{20AC}', "Euro"),
    (0x82, '\u{201A}', "quotesinglbase"),
    (0x83, '\u{0192}', "florin"),
    (0x84, '\u{201E}', "quotedblbase"),
    (0x85, '\u{2026}', "ellipsis"),
    (0x86, '\u{2020}', "dagger"),
    (0x87, '\u{2021}', "daggerdbl"),
    (0x88, '\u{02C6}', "circumflex"),
    (0x89, '\u{2030}', "perthousand"),
    (0x8A, '\u{0160}', "Scaron"),
    (0x8B, '\u{2039}', "guilsinglleft"),
    (0x8C, '\u{0152}', "OE"),
    (0x8E, '\u{017D}', "Zcaron"),
    (0x91, '\u{2018}', "quoteleft"),
    (0x92, '\u{2019}', "quoteright"),
    (0x93, '\u{201C}', "quotedblleft"),
    (0x94, '\u{201D}', "quotedblright"),
    (0x96, '\u{2013}', "endash"),
    (0x97, '\u{2014}', "emdash"),
    (0x98, '\u{02DC}', "tilde"),
    (0x99, '\u{2122}', "trademark"),
    (0x9A, '\u{0161}', "scaron"),
    (0x9B, '\u{203A}', "guilsinglright"),
    (0x9C, '\u{0153}', "oe"),
    (0x9E, '\u{017E}', "zcaron"),
    (0x9F, '\u{0178}', "Ydieresis"),
    (0xA1, '\u{00A1}', "exclamdown"),
    (0xA2, '\u{00A2}', "cent"),
    (0xA3, '\u{00A3}', "sterling"),
    (0xA4, '\u{00A4}', "currency"),
    (0xA5, '\u{00A5}', "yen"),
    (0xA6, '\u{00A6}', "brokenbar"),
    (0xA7, '\u{00A7}', "section"),
    (0xA8, '\u{00A8}', "dieresis"),
    (0xA9, '\u{00A9}', "copyright"),
    (0xAA, '\u{00AA}', "ordfeminine"),
    (0xAB, '\u{00AB}', "guillemotleft"),
    (0xAC, '\u{00AC}', "logicalnot"),
    (0xAE, '\u{00AE}', "registered"),
    (0xAF, '\u{00AF}', "macron"),
    (0xB0, '\u{00B0}', "degree"),
    (0xB1, '\u{00B1}', "plusminus"),
    (0xB2, '\u{00B2}', "twosuperior"),
    (0xB3, '\u{00B3}', "threesuperior"),
    (0xB4, '\u{00B4}', "acute"),
    (0xB5, '\u{00B5}', "mu"),
    (0xB6, '\u{00B6}', "paragraph"),
    (0xB7, '\u{00B7}', "periodcentered"),
    (0xB8, '\u{00B8}', "cedilla"),
    (0xB9, '\u{00B9}', "onesuperior"),
    (0xBA, '\u{00BA}', "ordmasculine"),
    (0xBB, '\u{00BB}', "guillemotright"),
    (0xBC, '\u{00BC}', "onequarter"),
    (0xBD, '\u{00BD}', "onehalf"),
    (0xBE, '\u{00BE}', "threequarters"),
    (0xBF, '\u{00BF}', "questiondown"),
    (0xC0, '\u{00C0}', "Agrave"),
    (0xC1, '\u{00C1}', "Aacute"),
    (0xC2, '\u{00C2}', "Acircumflex"),
    (0xC3, '\u{00C3}', "Atilde"),
    (0xC4, '\u{00C4}', "Adieresis"),
    (0xC5, '\u{00C5}', "Aring"),
    (0xC6, '\u{00C6}', "AE"),
    (0xC7, '\u{00C7}', "Ccedilla"),
    (0xC8, '\u{00C8}', "Egrave"),
    (0xC9, '\u{00C9}', "Eacute"),
    (0xCA, '\u{00CA}', "Ecircumflex"),
    (0xCB, '\u{00CB}', "Edieresis"),
    (0xCC, '\u{00CC}', "Igrave"),
    (0xCD, '\u{00CD}', "Iacute"),
    (0xCE, '\u{00CE}', "Icircumflex"),
    (0xCF, '\u{00CF}', "Idieresis"),
    (0xD0, '\u{00D0}', "Eth"),
    (0xD1, '\u{00D1}', "Ntilde"),
    (0xD2, '\u{00D2}', "Ograve"),
    (0xD3, '\u{00D3}', "Oacute"),
    (0xD4, '\u{00D4}', "Ocircumflex"),
    (0xD5, '\u{00D5}', "Otilde"),
    (0xD6, '\u{00D6}', "Odieresis"),
    (0xD7, '\u{00D7}', "multiply"),
    (0xD8, '\u{00D8}', "Oslash"),
    (0xD9, '\u{00D9}', "Ugrave"),
    (0xDA, '\u{00DA}', "Uacute"),
    (0xDB, '\u{00DB}', "Ucircumflex"),
    (0xDC, '\u{00DC}', "Udieresis"),
    (0xDD, '\u{00DD}', "Yacute"),
    (0xDE, '\u{00DE}', "Thorn"),
    (0xDF, '\u{00DF}', "germandbls"),
    (0xE0, '\u{00E0}', "agrave"),
    (0xE1, '\u{00E1}', "aacute"),
    (0xE2, '\u{00E2}', "acircumflex"),
    (0xE3, '\u{00E3}', "atilde"),
    (0xE4, '\u{00E4}', "adieresis"),
    (0xE5, '\u{00E5}', "aring"),
    (0xE6, '\u{00E6}', "ae"),
    (0xE7, '\u{00E7}', "ccedilla"),
    (0xE8, '\u{00E8}', "egrave"),
    (0xE9, '\u{00E9}', "eacute"),
    (0xEA, '\u{00EA}', "ecircumflex"),
    (0xEB, '\u{00EB}', "edieresis"),
    (0xEC, '\u{00EC}', "igrave"),
    (0xED, '\u{00ED}', "iacute"),
    (0xEE, '\u{00EE}', "icircumflex"),
    (0xEF, '\u{00EF}', "idieresis"),
    (0xF0, '\u{00F0}', "eth"),
    (0xF1, '\u{00F1}', "ntilde"),
    (0xF2, '\u{00F2}', "ograve"),
    (0xF3, '\u{00F3}', "oacute"),
    (0xF4, '\u{00F4}', "ocircumflex"),
    (0xF5, '\u{00F5}', "otilde"),
    (0xF6, '\u{00F6}', "odieresis"),
    (0xF7, '\u{00F7}', "divide"),
    (0xF8, '\u{00F8}', "oslash"),
    (0xF9, '\u{00F9}', "ugrave"),
    (0xFA, '\u{00FA}', "uacute"),
    (0xFB, '\u{00FB}', "ucircumflex"),
    (0xFC, '\u{00FC}', "udieresis"),
    (0xFD, '\u{00FD}', "yacute"),
    (0xFE, '\u{00FE}', "thorn"),
    (0xFF, '\u{00FF}', "ydieresis"),
];

/// La chasse `WX` de chaque glyphe de [`WINANSI`], en millièmes d'em : Helvetica, puis
/// Helvetica-Bold. Triée par nom (ordre des octets), pour la dichotomie de [`chasse`].
/// Engendrée (voir le module), `Euro` excepté.
#[rustfmt::skip]
pub(crate) const CHASSES: [(&str, u16, u16); 216] = [
    ("A", 667, 722),
    ("AE", 1000, 1000),
    ("Aacute", 667, 722),
    ("Acircumflex", 667, 722),
    ("Adieresis", 667, 722),
    ("Agrave", 667, 722),
    ("Aring", 667, 722),
    ("Atilde", 667, 722),
    ("B", 667, 722),
    ("C", 722, 722),
    ("Ccedilla", 722, 722),
    ("D", 722, 722),
    ("E", 667, 667),
    ("Eacute", 667, 667),
    ("Ecircumflex", 667, 667),
    ("Edieresis", 667, 667),
    ("Egrave", 667, 667),
    ("Eth", 722, 722),
    ("Euro", 556, 556),
    ("F", 611, 611),
    ("G", 778, 778),
    ("H", 722, 722),
    ("I", 278, 278),
    ("Iacute", 278, 278),
    ("Icircumflex", 278, 278),
    ("Idieresis", 278, 278),
    ("Igrave", 278, 278),
    ("J", 500, 556),
    ("K", 667, 722),
    ("L", 556, 611),
    ("M", 833, 833),
    ("N", 722, 722),
    ("Ntilde", 722, 722),
    ("O", 778, 778),
    ("OE", 1000, 1000),
    ("Oacute", 778, 778),
    ("Ocircumflex", 778, 778),
    ("Odieresis", 778, 778),
    ("Ograve", 778, 778),
    ("Oslash", 778, 778),
    ("Otilde", 778, 778),
    ("P", 667, 667),
    ("Q", 778, 778),
    ("R", 722, 722),
    ("S", 667, 667),
    ("Scaron", 667, 667),
    ("T", 611, 611),
    ("Thorn", 667, 667),
    ("U", 722, 722),
    ("Uacute", 722, 722),
    ("Ucircumflex", 722, 722),
    ("Udieresis", 722, 722),
    ("Ugrave", 722, 722),
    ("V", 667, 667),
    ("W", 944, 944),
    ("X", 667, 667),
    ("Y", 667, 667),
    ("Yacute", 667, 667),
    ("Ydieresis", 667, 667),
    ("Z", 611, 611),
    ("Zcaron", 611, 611),
    ("a", 556, 556),
    ("aacute", 556, 556),
    ("acircumflex", 556, 556),
    ("acute", 333, 333),
    ("adieresis", 556, 556),
    ("ae", 889, 889),
    ("agrave", 556, 556),
    ("ampersand", 667, 722),
    ("aring", 556, 556),
    ("asciicircum", 469, 584),
    ("asciitilde", 584, 584),
    ("asterisk", 389, 389),
    ("at", 1015, 975),
    ("atilde", 556, 556),
    ("b", 556, 611),
    ("backslash", 278, 278),
    ("bar", 260, 280),
    ("braceleft", 334, 389),
    ("braceright", 334, 389),
    ("bracketleft", 278, 333),
    ("bracketright", 278, 333),
    ("brokenbar", 260, 280),
    ("bullet", 350, 350),
    ("c", 500, 556),
    ("ccedilla", 500, 556),
    ("cedilla", 333, 333),
    ("cent", 556, 556),
    ("circumflex", 333, 333),
    ("colon", 278, 333),
    ("comma", 278, 278),
    ("copyright", 737, 737),
    ("currency", 556, 556),
    ("d", 556, 611),
    ("dagger", 556, 556),
    ("daggerdbl", 556, 556),
    ("degree", 400, 400),
    ("dieresis", 333, 333),
    ("divide", 584, 584),
    ("dollar", 556, 556),
    ("e", 556, 556),
    ("eacute", 556, 556),
    ("ecircumflex", 556, 556),
    ("edieresis", 556, 556),
    ("egrave", 556, 556),
    ("eight", 556, 556),
    ("ellipsis", 1000, 1000),
    ("emdash", 1000, 1000),
    ("endash", 556, 556),
    ("equal", 584, 584),
    ("eth", 556, 611),
    ("exclam", 278, 333),
    ("exclamdown", 333, 333),
    ("f", 278, 333),
    ("five", 556, 556),
    ("florin", 556, 556),
    ("four", 556, 556),
    ("g", 556, 611),
    ("germandbls", 611, 611),
    ("grave", 333, 333),
    ("greater", 584, 584),
    ("guillemotleft", 556, 556),
    ("guillemotright", 556, 556),
    ("guilsinglleft", 333, 333),
    ("guilsinglright", 333, 333),
    ("h", 556, 611),
    ("hyphen", 333, 333),
    ("i", 222, 278),
    ("iacute", 278, 278),
    ("icircumflex", 278, 278),
    ("idieresis", 278, 278),
    ("igrave", 278, 278),
    ("j", 222, 278),
    ("k", 500, 556),
    ("l", 222, 278),
    ("less", 584, 584),
    ("logicalnot", 584, 584),
    ("m", 833, 889),
    ("macron", 333, 333),
    ("mu", 556, 611),
    ("multiply", 584, 584),
    ("n", 556, 611),
    ("nine", 556, 556),
    ("ntilde", 556, 611),
    ("numbersign", 556, 556),
    ("o", 556, 611),
    ("oacute", 556, 611),
    ("ocircumflex", 556, 611),
    ("odieresis", 556, 611),
    ("oe", 944, 944),
    ("ograve", 556, 611),
    ("one", 556, 556),
    ("onehalf", 834, 834),
    ("onequarter", 834, 834),
    ("onesuperior", 333, 333),
    ("ordfeminine", 370, 370),
    ("ordmasculine", 365, 365),
    ("oslash", 611, 611),
    ("otilde", 556, 611),
    ("p", 556, 611),
    ("paragraph", 537, 556),
    ("parenleft", 333, 333),
    ("parenright", 333, 333),
    ("percent", 889, 889),
    ("period", 278, 278),
    ("periodcentered", 278, 278),
    ("perthousand", 1000, 1000),
    ("plus", 584, 584),
    ("plusminus", 584, 584),
    ("q", 556, 611),
    ("question", 556, 611),
    ("questiondown", 611, 611),
    ("quotedbl", 355, 474),
    ("quotedblbase", 333, 500),
    ("quotedblleft", 333, 500),
    ("quotedblright", 333, 500),
    ("quoteleft", 222, 278),
    ("quoteright", 222, 278),
    ("quotesinglbase", 222, 278),
    ("quotesingle", 191, 238),
    ("r", 333, 389),
    ("registered", 737, 737),
    ("s", 500, 556),
    ("scaron", 500, 556),
    ("section", 556, 556),
    ("semicolon", 278, 333),
    ("seven", 556, 556),
    ("six", 556, 556),
    ("slash", 278, 278),
    ("space", 278, 278),
    ("sterling", 556, 556),
    ("t", 278, 333),
    ("thorn", 556, 611),
    ("three", 556, 556),
    ("threequarters", 834, 834),
    ("threesuperior", 333, 333),
    ("tilde", 333, 333),
    ("trademark", 1000, 1000),
    ("two", 556, 556),
    ("twosuperior", 333, 333),
    ("u", 556, 611),
    ("uacute", 556, 611),
    ("ucircumflex", 556, 611),
    ("udieresis", 556, 611),
    ("ugrave", 556, 611),
    ("underscore", 556, 556),
    ("v", 500, 556),
    ("w", 722, 778),
    ("x", 500, 556),
    ("y", 500, 556),
    ("yacute", 500, 556),
    ("ydieresis", 500, 556),
    ("yen", 556, 556),
    ("z", 500, 500),
    ("zcaron", 500, 500),
    ("zero", 556, 556),
];

/// La chasse d'un caractère en millièmes d'em, ou `None` s'il n'a pas d'octet WinAnsi —
/// `lopdf` ne l'imprimerait pas.
fn chasse(c: char, gras: bool) -> Option<u16> {
    let (_, _, nom) = WINANSI.iter().find(|(_, car, _)| *car == c)?;
    let i = CHASSES.binary_search_by(|(n, _, _)| n.cmp(nom)).ok()?;
    let (_, maigre, grasse) = CHASSES[i];
    Some(if gras { grasse } else { maigre })
}

/// La largeur de `texte` imprimé en Helvetica (ou Helvetica-Bold si `gras`) à `taille`
/// points, en millimètres : la somme des chasses, sans crénage (printpdf n'en applique pas).
///
/// Un caractère sans entrée compte **zéro** : [`WINANSI`] porte tous les octets imprimables
/// (`table_triee_et_complete`), si bien qu'un caractère sans entrée n'a pas d'octet, et
/// `lopdf` le retire à l'écriture (`lopdf-0.31.0/src/encodings/mod.rs:14-18`,
/// `string_to_bytes`) : il ne s'imprime pas. Une espace insécable (U+00A0, que WinAnsi place à
/// 0xA0 sous le nom `space`, déjà pris par 0x20) en est un exemple. La largeur rendue est donc
/// celle du texte réellement imprimé (revue P1, B-2).
pub(crate) fn largeur_mm(texte: &str, taille: f32, gras: bool) -> f32 {
    let milliemes: u32 = texte
        .chars()
        .map(|c| u32::from(chasse(c, gras).unwrap_or(0)))
        .sum();
    milliemes as f32 / 1000.0 * taille * MM_PAR_POINT
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::texte::texte_imprime;
    use crate::validation::est_caractere_qr;

    /// **AC8** — des témoins recopiés de l'AFM à la main, pour qu'une erreur de la table
    /// (ou du script qui l'engendre) se voie : maigre `phvr8a.afm`, gras `phvb8a.afm`, lus
    /// par nom de glyphe ; `€` est l'ajout écrit au doc-comment du module.
    #[test]
    fn table_des_largeurs_temoins() {
        for (c, maigre, gras) in [
            (' ', 278, 278),
            ('A', 667, 722),
            ('W', 944, 944),
            ('@', 1015, 975),
            ('é', 556, 556),
            // WinAnsi 0x27 est `quotesingle` (191 / 238), non `quoteright` (222), que
            // l'AdobeStandardEncoding de l'AFM place au code 39.
            ('\'', 191, 238),
            // WinAnsi 0x60 est `grave` (333), non `quoteleft`.
            ('`', 333, 333),
            ('€', 556, 556),
        ] {
            assert_eq!(chasse(c, false), Some(maigre), "maigre {c:?}");
            assert_eq!(chasse(c, true), Some(gras), "gras {c:?}");
        }
    }

    /// La dichotomie de [`chasse`] exige [`CHASSES`] triée et sans doublon, et chaque nom de
    /// [`WINANSI`] doit y avoir sa ligne : sans quoi un caractère imprimable compterait pour
    /// zéro, en silence.
    #[test]
    fn table_triee_et_complete() {
        assert!(CHASSES.windows(2).all(|p| p[0].0 < p[1].0), "CHASSES triée");
        for (octet, c, nom) in WINANSI {
            assert!(
                CHASSES.binary_search_by(|(n, _, _)| n.cmp(&nom)).is_ok(),
                "{nom} (octet {octet:#04X}, {c:?}) sans chasse"
            );
        }
        assert!(
            WINANSI.windows(2).all(|p| p[0].0 < p[1].0),
            "octets croissants"
        );
    }

    /// **AC8** — tout ce que [`texte_imprime`] peut sortir d'un caractère admis par la
    /// directive (§ 4.1.1, [`est_caractere_qr`]) a son entrée : aucun caractère imprimé ne
    /// compte pour zéro. `Ĳ` sort `IJ`, deux entrées. Le balayage couvre tout Unicode (revue
    /// P1, E-3) : une addition future au jeu admis, où qu'elle tombe, est contrôlée.
    #[test]
    fn table_couvre_les_sorties_imprimees() {
        let admis: Vec<char> = (0..=0x10FFFFu32)
            .filter_map(char::from_u32)
            .filter(|c| est_caractere_qr(*c))
            .collect();
        assert!(
            admis.contains(&'€') && admis.contains(&'Ĳ'),
            "montage : jeu admis"
        );
        let mut sans_entree = Vec::new();
        for c in admis {
            for sortie in texte_imprime(&c.to_string()).chars() {
                if chasse(sortie, false).is_none() || chasse(sortie, true).is_none() {
                    sans_entree.push((c, sortie));
                }
            }
        }
        assert!(sans_entree.is_empty(), "sans entrée : {sans_entree:?}");
        assert_eq!(texte_imprime("Ĳ"), "IJ");
    }

    /// La largeur d'un texte, en mm : « Acceptance point » en gras à 6 pt, 8335 millièmes
    /// d'em (somme des `WX` de `phvb8a.afm`), soit 17,642 mm ; un caractère sans octet
    /// WinAnsi (U+4E00, U+00A0), que `lopdf` n'imprime pas, compte zéro.
    #[test]
    fn largeur_en_millimetres() {
        let attendu = 8335.0 / 1000.0 * 6.0 * 25.4 / 72.0;
        assert!((largeur_mm("Acceptance point", 6.0, true) - attendu).abs() < 0.001);
        assert_eq!(largeur_mm("\u{4E00}", 10.0, false), 0.0);
        let sans_espace = (8335.0 - 278.0) / 1000.0 * 6.0 * 25.4 / 72.0;
        assert!((largeur_mm("Acceptance\u{A0}point", 6.0, true) - sans_espace).abs() < 0.001);
        assert_eq!(largeur_mm("", 8.0, false), 0.0);
    }
}
