//! Lettrage — la part **pure** de la marque (Story 15-1a-i, #518).
//!
//! Un **groupe de lettrage** est un ensemble d'au moins deux lignes d'un même
//! compte lettrable dont la somme `Σ(débit − crédit)` est exactement nulle ; la
//! marque est portée par chaque ligne (`journal_entry_lines.lettering_key`,
//! `lettering_origin`), la clé du groupe étant le **plus petit `id`** de ses
//! lignes (R1).
//!
//! Ce module rassemble ce qui se décide **sans base** :
//!
//! - le **code** affiché du groupe — sa clé écrite en lettres, en base 26
//!   bijective (`1 → A`, `27 → AA`), et son inverse (R2) ;
//! - **une fonction par cause de refus** que la primitive
//!   `kesh_db::repositories::letterings::create_group_in_tx` appelle **à son
//!   rang** (R3) : [`check_line_ids`] (rang 1), [`check_same_account`]
//!   (rang 3), [`check_none_lettered`] (rang 6), [`check_balanced`] (rang 7).
//!   ⛔ Pas de `validate_group` unique : appelé d'un bloc, il rendrait les
//!   rangs 6 et 7 avant les contrôles en base des rangs 4, 4 bis et 5 ;
//! - [`check_rows_affected`], la comparaison du nombre de lignes **trouvées**
//!   par l'`UPDATE` final de chacune des deux primitives (R7 point 4) ;
//! - le **moteur des propositions** ([`proposals`], Story 15-1b) : des paires
//!   débit/crédit de montants égaux parmi les lignes candidates d'un compte.

use rust_decimal::Decimal;
use std::collections::BTreeSet;

pub mod proposals;

/// Plafond de lignes d'un groupe posé par la route manuelle (AC6) : il borne
/// le corps de la requête et l'`IN (…)` de la lecture verrouillante.
pub const MAX_LINES_PER_GROUP: usize = 200;

/// Pourquoi un groupe ne se forme pas, ou pourquoi l'écriture de sa marque a
/// trouvé un autre nombre de lignes que prévu — les causes qui se décident
/// sans base. La couche de persistance les convertit en ses propres erreurs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LetteringRefusal {
    /// Rang 1 : moins de deux identifiants, ou un identifiant répété.
    TooFewLines,
    /// Rang 3 : les lignes ne portent pas toutes sur le même compte.
    AccountsDiffer,
    /// Rang 6 : une ligne est déjà lettrée — `key` est la clé de son groupe.
    LineAlreadyLettered { key: i64 },
    /// Rang 7 : la somme `Σ(débit − crédit)` n'est pas nulle.
    Unbalanced { difference: Decimal },
    /// R7 point 4 : l'`UPDATE` final n'a pas trouvé le nombre de lignes
    /// attendu — le groupe a changé entre-temps.
    ConcurrentChange,
}

/// Code affiché d'un groupe : sa clé écrite en **base 26 bijective**
/// (`1 → A`, `26 → Z`, `27 → AA`, `52 → AZ`, `53 → BA`, `702 → ZZ`,
/// `703 → AAA`).
///
/// La clé d'un groupe est un identifiant de ligne, donc ≥ 1 ; `0` n'a pas de
/// code et rend la chaîne vide (aucune lettre).
pub fn code_from_key(key: u64) -> String {
    let mut n = key;
    let mut lettres = Vec::new();
    while n > 0 {
        n -= 1;
        // `n % 26` < 26 : la conversion est exacte.
        lettres.push(b'A' + (n % 26) as u8);
        n /= 26;
    }
    lettres.reverse();
    // Que des octets ASCII `A`–`Z`.
    String::from_utf8(lettres).unwrap_or_default()
}

/// Inverse de [`code_from_key`] : `AA → Some(27)`.
///
/// L'entrée est mise en majuscules par [`str::to_ascii_uppercase`], **puis**
/// validée octet par octet sur `A`–`Z` : tout autre caractère rend `None`.
/// ⛔ Jamais `str::to_uppercase` avant la validation : la casse Unicode ferait
/// de `ß` un `SS`, du s long `ſ` un `S` et du i sans point `ı` un `I`, si bien
/// qu'un code étranger désignerait un groupe réel.
///
/// Arithmétique **vérifiée** : un code dont la valeur dépasse `i64::MAX` (la
/// colonne est un `BIGINT` signé) rend `None` — sans quoi un code assez long
/// déborderait `u64` et bouclerait en silence en release, jusqu'à désigner une
/// clé réelle.
pub fn key_from_code(code: &str) -> Option<u64> {
    if code.is_empty() {
        return None;
    }
    let majuscules = code.to_ascii_uppercase();
    let mut valeur: u64 = 0;
    for octet in majuscules.bytes() {
        if !octet.is_ascii_uppercase() {
            return None;
        }
        let chiffre = u64::from(octet - b'A') + 1;
        valeur = valeur.checked_mul(26)?.checked_add(chiffre)?;
        if valeur > i64::MAX as u64 {
            return None;
        }
    }
    Some(valeur)
}

/// Lit la référence d'un groupe telle que la route la reçoit (`{key}`) : la
/// **clé numérique** (`27`) ou le **code** (`AA`).
///
/// Rend `None` — que la route traduit en 404 — pour une clé ≤ 0, une clé au-delà
/// de `i64::MAX`, un code invalide ou qui déborde ([`key_from_code`]), et tout
/// mélange de chiffres et de lettres.
pub fn parse_group_reference(reference: &str) -> Option<i64> {
    if !reference.is_empty() && reference.bytes().all(|b| b.is_ascii_digit()) {
        return reference.parse::<i64>().ok().filter(|k| *k >= 1);
    }
    key_from_code(reference).and_then(|k| i64::try_from(k).ok())
}

/// Rang 1 : au moins **deux** identifiants, **sans doublon**.
pub fn check_line_ids(line_ids: &[i64]) -> Result<(), LetteringRefusal> {
    let distincts: BTreeSet<i64> = line_ids.iter().copied().collect();
    if line_ids.len() < 2 || distincts.len() != line_ids.len() {
        return Err(LetteringRefusal::TooFewLines);
    }
    Ok(())
}

/// Rang 3 : toutes les lignes portent sur le **même compte**.
pub fn check_same_account(account_ids: &[i64]) -> Result<(), LetteringRefusal> {
    match account_ids.split_first() {
        Some((premier, reste)) if reste.iter().any(|a| a != premier) => {
            Err(LetteringRefusal::AccountsDiffer)
        }
        _ => Ok(()),
    }
}

/// Rang 6 : aucune ligne n'est déjà lettrée. Rend la clé du **premier** groupe
/// rencontré, dans l'ordre reçu (celui des `id`, la lecture étant triée).
pub fn check_none_lettered(lettering_keys: &[Option<i64>]) -> Result<(), LetteringRefusal> {
    match lettering_keys.iter().flatten().next() {
        Some(key) => Err(LetteringRefusal::LineAlreadyLettered { key: *key }),
        None => Ok(()),
    }
}

/// Rang 7 : `Σ(débit − crédit) = 0` **exactement** (`DECIMAL(19,4)` est exact).
/// L'écart rendu est cette somme, signée.
pub fn check_balanced(amounts: &[(Decimal, Decimal)]) -> Result<(), LetteringRefusal> {
    let difference: Decimal = amounts.iter().map(|(d, c)| *d - *c).sum();
    if difference.is_zero() {
        Ok(())
    } else {
        Err(LetteringRefusal::Unbalanced { difference })
    }
}

/// R7 point 4 : l'`UPDATE` final d'une primitive vise des lignes **tenues**
/// depuis sa lecture verrouillante ; son compte de lignes trouvées
/// (`CLIENT_FOUND_ROWS`, posé par sqlx) doit égaler l'attendu. Un écart est un
/// refus **métier** ([`LetteringRefusal::ConcurrentChange`]), jamais un succès
/// partiel ni un invariant violé.
pub fn check_rows_affected(expected: u64, actual: u64) -> Result<(), LetteringRefusal> {
    if expected == actual {
        Ok(())
    } else {
        Err(LetteringRefusal::ConcurrentChange)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn code_from_key_named_limits() {
        for (key, code) in [
            (1, "A"),
            (26, "Z"),
            (27, "AA"),
            (52, "AZ"),
            (53, "BA"),
            (702, "ZZ"),
            (703, "AAA"),
        ] {
            assert_eq!(code_from_key(key), code, "clé {key}");
            assert_eq!(key_from_code(code), Some(key), "code {code}");
        }
    }

    #[test]
    fn code_round_trip_up_to_100_000() {
        for key in 1..=100_000u64 {
            assert_eq!(key_from_code(&code_from_key(key)), Some(key), "clé {key}");
        }
    }

    #[test]
    fn key_from_code_refuses_invalid_input() {
        for code in ["", "1", "A1", "Ä", "A B", " ", "ß", "ſ", "ı", "-A"] {
            assert_eq!(key_from_code(code), None, "code {code:?}");
        }
    }

    /// F4-2 : `to_ascii_uppercase` laisse `ß`, `ſ` et `ı` tels quels, et la
    /// validation les refuse — `str::to_uppercase` en aurait fait `SS`, `S`, `I`.
    #[test]
    fn unicode_case_folding_does_not_reach_an_ascii_code() {
        assert_eq!(key_from_code("ß"), None);
        assert_ne!(key_from_code("SS"), None);
        assert_eq!(key_from_code("ſ"), None);
        assert_eq!(key_from_code("ı"), None);
    }

    #[test]
    fn key_from_code_accepts_ascii_lowercase() {
        assert_eq!(key_from_code("aa"), Some(27));
        assert_eq!(key_from_code("Az"), Some(52));
    }

    #[test]
    fn key_from_code_overflow_is_none() {
        assert_eq!(key_from_code(&"Z".repeat(20)), None);
        assert_eq!(key_from_code("CRPXNLSKVLJFHG"), Some(i64::MAX as u64));
        assert_eq!(code_from_key(i64::MAX as u64), "CRPXNLSKVLJFHG");
        assert_eq!(key_from_code("CRPXNLSKVLJFHH"), None);
    }

    #[test]
    fn parse_group_reference_reads_key_or_code() {
        assert_eq!(parse_group_reference("27"), Some(27));
        assert_eq!(parse_group_reference("AA"), Some(27));
        assert_eq!(parse_group_reference("aa"), Some(27));
        for invalide in [
            "",
            "0",
            "-1",
            "A1",
            "1A",
            "Ä",
            "ß",
            "99999999999999999999",
            "9223372036854775808",
        ] {
            assert_eq!(parse_group_reference(invalide), None, "{invalide:?}");
        }
        assert_eq!(parse_group_reference(&"Z".repeat(20)), None);
        assert_eq!(parse_group_reference("9223372036854775807"), Some(i64::MAX));
    }

    #[test]
    fn check_line_ids_rank_1() {
        assert_eq!(check_line_ids(&[]), Err(LetteringRefusal::TooFewLines));
        assert_eq!(check_line_ids(&[4]), Err(LetteringRefusal::TooFewLines));
        assert_eq!(check_line_ids(&[4, 4]), Err(LetteringRefusal::TooFewLines));
        assert_eq!(
            check_line_ids(&[4, 5, 4]),
            Err(LetteringRefusal::TooFewLines)
        );
        assert_eq!(check_line_ids(&[4, 5]), Ok(()));
    }

    #[test]
    fn check_same_account_rank_3() {
        assert_eq!(check_same_account(&[7, 7, 7]), Ok(()));
        assert_eq!(
            check_same_account(&[7, 8]),
            Err(LetteringRefusal::AccountsDiffer)
        );
    }

    #[test]
    fn check_none_lettered_rank_6_names_the_first_group() {
        assert_eq!(check_none_lettered(&[None, None]), Ok(()));
        assert_eq!(
            check_none_lettered(&[None, Some(12), Some(3)]),
            Err(LetteringRefusal::LineAlreadyLettered { key: 12 })
        );
    }

    #[test]
    fn check_balanced_rank_7_is_exact() {
        assert_eq!(
            check_balanced(&[
                (dec!(100), dec!(0)),
                (dec!(0), dec!(60)),
                (dec!(0), dec!(40))
            ]),
            Ok(())
        );
        assert_eq!(
            check_balanced(&[(dec!(100), dec!(0)), (dec!(0), dec!(99.9999))]),
            Err(LetteringRefusal::Unbalanced {
                difference: dec!(0.0001)
            })
        );
        assert_eq!(
            check_balanced(&[(dec!(0), dec!(10)), (dec!(5), dec!(0))]),
            Err(LetteringRefusal::Unbalanced {
                difference: dec!(-5)
            })
        );
    }

    #[test]
    fn check_rows_affected_compares_found_rows() {
        assert_eq!(check_rows_affected(3, 3), Ok(()));
        assert_eq!(
            check_rows_affected(3, 2),
            Err(LetteringRefusal::ConcurrentChange)
        );
        assert_eq!(
            check_rows_affected(3, 4),
            Err(LetteringRefusal::ConcurrentChange)
        );
    }
}
