//! Reconnaissance d'un **numéro de carte de paiement complet** dans un texte
//! libre — Story 34-11a2 (#972, choix C-e341-15).
//!
//! Kesh ne garde que les quatre derniers chiffres d'une carte de crédit ; le
//! numéro complet n'est jamais enregistré. [`looks_like_card_number`] est la
//! garde qui refuse un nom ou une description de compte bancaire qui en
//! porterait un, saisi par accident.
//!
//! ## Ce que la garde reconnaît (C-e341-15, (c) à (g))
//!
//! 1. **Suite candidate** : une plus longue séquence de chiffres ASCII dont deux
//!    groupes consécutifs sont séparés par **exactement un** séparateur — espace
//!    U+0020, espaces insécables U+00A0 et U+202F, tiret `-`. Un double
//!    séparateur ou un séparateur de queue termine la suite.
//! 2. **Suite entière** : le caractère immédiatement avant son premier chiffre
//!    et celui immédiatement après son dernier sont absents (bord du texte) ou ne
//!    sont ni chiffre ni lettre. `4111111111111111perso` n'est pas entière ;
//!    `CH95 0076 …` non plus (`H` colle au premier chiffre).
//! 3. **Groupement d'un réseau** : contigu ; 4-4-4-4 ; 4-4-4-4-3 ; 4-6-5
//!    (American Express) ; 4-6-4 (Diners) — séparateur unique et constant.
//! 4. **Longueur** de 13 à 19 chiffres.
//! 5. **Préfixe d'un réseau** (IIN) cohérent avec la longueur ([`network_admits`]).
//! 6. **Somme de Luhn** juste.
//!
//! **Avant tout, les IBAN valides sont écartés** (revue P1, B7 = E4) : une suite
//! qui commence par deux lettres (bordées à gauche par ni chiffre ni lettre),
//! suivie de chiffres et de lettres sous des séparateurs uniques, et dont un
//! préfixe compacté est un IBAN valide ([`crate::types::Iban::new`] : pays connu,
//! longueur du pays, MOD-97), est masquée. Un IBAN suisse écrit `CH 48…` d'un
//! bloc, ou `CH 4826 7511 …` (4-4-4-4-3), dont les dix-neuf chiffres passeraient
//! pour un Visa, n'est donc pas refusé.
//!
//! ## Limites assumées
//!
//! La garde protège d'une **saisie accidentelle** ; ce n'est pas un filtre de
//! sécurité. Passent, entre autres :
//! - un numéro suivi ou précédé d'autres chiffres sous un séparateur unique, même
//!   différent (`4111 1111 1111 1111 12/28` forme 4-4-4-4-2 ;
//!   `12-4111 1111 1111 1111`, `Facture 12 4111 1111 1111 1111` forment 2-4-4-4-4) ;
//! - un séparateur autre que ceux du point 1 (`4111.1111.1111.1111`, tabulation,
//!   espaces U+2007, U+2009, U+200B, tiret U+2013) ;
//! - des chiffres non ASCII (pleine chasse `４１１１…`) ;
//! - une lettre collée à un bord (`VISA4111111111111111`, `4111111111111111perso`) ;
//! - un groupement que la liste n'a pas (American Express par quatre, 4-4-4-3 ;
//!   Visa 13 groupé) ; Diners en seize chiffres ;
//! - un numéro coupé entre le nom et la description : chaque champ est contrôlé seul.

/// Vrai si `c` sépare deux groupes de chiffres d'une suite candidate.
fn is_separator(c: char) -> bool {
    matches!(c, ' ' | '\u{00A0}' | '\u{202F}' | '-')
}

/// Les groupements admis (longueurs des groupes, dans l'ordre), hors suite
/// contiguë, qui est admise à part.
const GROUPINGS: &[&[usize]] = &[&[4, 4, 4, 4], &[4, 4, 4, 4, 3], &[4, 6, 5], &[4, 6, 4]];

/// Vrai si `text` contient une suite qui a la forme d'un numéro de carte
/// complet au sens de C-e341-15 (voir la doc du module), Luhn compris.
pub fn looks_like_card_number(text: &str) -> bool {
    let chars = mask_ibans(&text.chars().collect::<Vec<char>>());
    let mut i = 0;
    while i < chars.len() {
        if !chars[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        // Début d'une suite : on lit ses groupes.
        let start = i;
        let mut groups: Vec<usize> = Vec::new();
        let mut digits = String::new();
        let mut separators: Vec<char> = Vec::new();
        let mut j = i;
        loop {
            let group_start = j;
            while j < chars.len() && chars[j].is_ascii_digit() {
                digits.push(chars[j]);
                j += 1;
            }
            groups.push(j - group_start);
            // Un seul séparateur, suivi d'un chiffre : le groupe suivant.
            if j + 1 < chars.len() && is_separator(chars[j]) && chars[j + 1].is_ascii_digit() {
                separators.push(chars[j]);
                j += 1;
                continue;
            }
            break;
        }
        let end = j; // indice qui suit le dernier chiffre
        if is_whole(&chars, start, end) && is_card_shaped(&groups, &separators, &digits) {
            return true;
        }
        i = end;
    }
    false
}

/// Longueur maximale d'un IBAN (norme ISO 13616), et minimale d'un pays connu.
const IBAN_MAX: usize = 34;
const IBAN_MIN: usize = 15;

/// Masque (remplace par `X`) les IBAN valides de `chars` — voir la doc du module.
/// Une suite candidate part de deux lettres ASCII bordées à gauche par ni chiffre
/// ni lettre, et lit les caractères alphanumériques ASCII qui suivent, sous des
/// séparateurs uniques ; le plus long préfixe compacté qui est un IBAN valide est
/// masqué, séparateurs internes laissés tels quels.
fn mask_ibans(chars: &[char]) -> Vec<char> {
    let mut out = chars.to_vec();
    let mut i = 0;
    while i + 1 < chars.len() {
        let starts = chars[i].is_ascii_alphabetic()
            && chars[i + 1].is_ascii_alphabetic()
            && (i == 0 || !chars[i - 1].is_alphanumeric());
        if !starts {
            i += 1;
            continue;
        }
        // Indices des caractères alphanumériques de la suite.
        let mut idx = vec![i, i + 1];
        let mut j = i + 2;
        while j < chars.len() && idx.len() < IBAN_MAX {
            if chars[j].is_ascii_alphanumeric() {
                idx.push(j);
                j += 1;
            } else if is_separator(chars[j])
                && j + 1 < chars.len()
                && chars[j + 1].is_ascii_alphanumeric()
            {
                j += 1;
            } else {
                break;
            }
        }
        let compact: String = idx.iter().map(|&k| chars[k]).collect();
        let found = (IBAN_MIN..=compact.len())
            .rev()
            .find(|&len| crate::types::Iban::new(&compact[..len]).is_ok());
        match found {
            Some(len) => {
                for &k in &idx[..len] {
                    out[k] = 'X';
                }
                i = idx[len - 1] + 1;
            }
            None => i += 1,
        }
    }
    out
}

/// La suite `chars[start..end]` est entière : ni chiffre ni lettre à ses bords.
fn is_whole(chars: &[char], start: usize, end: usize) -> bool {
    let before_ok = start == 0 || !chars[start - 1].is_alphanumeric();
    let after_ok = end >= chars.len() || !chars[end].is_alphanumeric();
    before_ok && after_ok
}

/// Groupement, séparateur constant, longueur, préfixe, puis Luhn.
fn is_card_shaped(groups: &[usize], separators: &[char], digits: &str) -> bool {
    if let Some(first) = separators.first()
        && separators.iter().any(|s| s != first)
    {
        return false;
    }
    let grouping_ok = groups.len() == 1 || GROUPINGS.contains(&groups);
    let len = digits.len();
    grouping_ok && (13..=19).contains(&len) && network_admits(digits) && luhn_valid(digits)
}

/// Préfixe d'un réseau (IIN) cohérent avec la longueur (C-e341-15 (f)) : Visa
/// `4` (13, 16, 19) ; Mastercard `51`-`55`, `2221`-`2720` (16) ; American
/// Express `34`, `37` (15) ; Diners `300`-`305`, `36`, `38` (14) ; Discover
/// `6011`, `644`-`649`, `65` (16, 19) ; JCB `3528`-`3589` (16 à 19) ; Maestro
/// `50`, `56`-`69` (13 à 19). Aucun préfixe ne commence par `0`.
fn network_admits(digits: &str) -> bool {
    let len = digits.len();
    let prefix = |n: usize| -> u32 { digits.get(..n).and_then(|p| p.parse().ok()).unwrap_or(0) };
    let p1 = prefix(1);
    let p2 = prefix(2);
    let p3 = prefix(3);
    let p4 = prefix(4);
    let visa = p1 == 4 && matches!(len, 13 | 16 | 19);
    let mastercard = ((51..=55).contains(&p2) || (2221..=2720).contains(&p4)) && len == 16;
    let amex = matches!(p2, 34 | 37) && len == 15;
    let diners = ((300..=305).contains(&p3) || matches!(p2, 36 | 38)) && len == 14;
    let discover = (p4 == 6011 || (644..=649).contains(&p3) || p2 == 65) && matches!(len, 16 | 19);
    let jcb = (3528..=3589).contains(&p4) && (16..=19).contains(&len);
    let maestro = (p2 == 50 || (56..=69).contains(&p2)) && (13..=19).contains(&len);
    visa || mastercard || amex || diners || discover || jcb || maestro
}

/// Somme de Luhn (modulo 10) d'une suite de chiffres ASCII.
fn luhn_valid(digits: &str) -> bool {
    let mut sum = 0u32;
    for (i, c) in digits.chars().rev().enumerate() {
        let Some(mut d) = c.to_digit(10) else {
            return false;
        };
        if i % 2 == 1 {
            d *= 2;
            if d > 9 {
                d -= 9;
            }
        }
        sum += d;
    }
    sum.is_multiple_of(10)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Numéros de test publics des réseaux, sous leurs formes usuelles :
    /// refusés, seuls et entourés de texte (AC-3).
    #[test]
    fn numeros_de_test_publics_refuses() {
        let refuses = [
            "4111111111111111",                         // Visa 16 contigu
            "4111 1111 1111 1111",                      // Visa 4-4-4-4 espacé
            "4111\u{00A0}1111\u{00A0}1111\u{00A0}1111", // espace insécable
            "4111\u{202F}1111\u{202F}1111\u{202F}1111", // espace fine insécable
            "4111 1111 1111 1111 110",                  // Visa 19, 4-4-4-4-3
            "5555-5555-5555-4444",                      // Mastercard 4-4-4-4 tiret
            "3782 822463 10005",                        // American Express 4-6-5
            "378282246310005",                          // American Express contigu
            "3056 930902 5904",                         // Diners 4-6-4
            "Visa 4111 1111 1111 1111 perso",           // entouré de texte
            "Carte: 5555-5555-5555-4444.",              // ponctuation aux bords
            "(3782 822463 10005)",
        ];
        for t in refuses {
            assert!(looks_like_card_number(t), "{t:?} doit être reconnu");
        }
    }

    /// Les acceptés de la famille (i) sont **construits pour passer Luhn** :
    /// la table l'asserte, sans quoi le test ne prouverait rien (AC-3).
    #[test]
    fn acceptes_qui_passent_luhn() {
        let cas = [
            // IBAN suisse synthétique : ses 19 chiffres passent Luhn.
            ("CH9500762008063608377", "9500762008063608377"),
            ("CH95 0076 2008 0636 0837 7", "9500762008063608377"),
            // Référence QR synthétique, 27 chiffres, contiguë puis par cinq.
            ("210000000003139471430009313", "210000000003139471430009313"),
            (
                "21 00000 00003 13947 14300 09313",
                "210000000003139471430009313",
            ),
            // Visa 16, Luhn vrai, groupement 2-4-4-4-2 non admis (M2b).
            ("41 1111 1111 1111 11", "4111111111111111"),
            // 13 chiffres, Luhn vrai, préfixe `0` d'aucun réseau (M2c).
            ("0041791234564", "0041791234564"),
            // Lettre collée au dernier chiffre : suite non entière (M2).
            ("4111111111111111perso", "4111111111111111"),
            // 16 chiffres, Luhn vrai, sans préfixe de réseau (M2c).
            ("0111111111111119", "0111111111111119"),
            // 16 chiffres, Luhn vrai, préfixe American Express (15 seulement) :
            // la longueur liée au préfixe est contrôlée.
            ("3782246310005001", "3782246310005001"),
        ];
        for (texte, chiffres) in cas {
            assert!(luhn_valid(chiffres), "{chiffres} doit passer Luhn");
            assert!(
                !looks_like_card_number(texte),
                "{texte:?} doit être accepté"
            );
        }
    }

    /// Famille (ii) : acceptés par la longueur ou le groupement, sans exigence
    /// de Luhn (AC-3).
    #[test]
    fn acceptes_par_longueur_ou_groupement() {
        for t in [
            "+41 79 123 45 67",
            "2026-11-10",
            "20261110-0042",
            "1'234.50",
            "Visa — Personnel",
            "",
            "Compte 1234",
            "4111 1111 1111 1111 12/28",      // limite (1) : 4-4-4-4-2
            "4111.1111.1111.1111",            // limite (2) : séparateur non reconnu
            "Facture 12 4111 1111 1111 1111", // limite (4)
            "4111  1111  1111  1111",         // double séparateur : quatre suites
            "4111 1111-1111 1111",            // séparateur non constant
        ] {
            assert!(!looks_like_card_number(t), "{t:?} doit être accepté");
        }
    }

    /// Revue P1 (B7 = E4) — un IBAN suisse **valide** dont les dix-neuf chiffres
    /// passent pour un Visa 19 (Luhn vrai), écrit avec une espace après le code
    /// pays, d'un bloc ou groupé : accepté. Les mêmes chiffres sans le code pays
    /// sont refusés — sans quoi le test ne prouverait rien (M16).
    #[test]
    fn un_iban_valide_n_est_jamais_un_numero_de_carte() {
        // IBAN synthétique, MOD-97 juste (calculé), chiffres Luhn vrais.
        let chiffres = "4826751125328704655";
        assert!(luhn_valid(chiffres));
        assert!(crate::types::Iban::new("CH4826751125328704655").is_ok());
        assert!(
            looks_like_card_number(chiffres),
            "témoin : sans le code pays"
        );
        for t in [
            "CH 4826751125328704655",
            "CH-4826751125328704655",
            "CH 4826 7511 2532 8704 655",
            "Compte ch 4826 7511 2532 8704 655 perso",
            "CH4826751125328704655",
        ] {
            assert!(!looks_like_card_number(t), "{t:?} est un IBAN");
        }
        // Deux lettres qui ne forment pas un IBAN valide (pays inconnu) : rien
        // n'est masqué, la garde reprend la main.
        assert!(looks_like_card_number("XY 4826751125328704655"));
        // Après un IBAN valide, un numéro de carte reste vu.
        assert!(looks_like_card_number(
            "CH 4826751125328704655 et 4111 1111 1111 1111"
        ));
    }

    /// Une suite Luhn-fausse à la forme d'un Visa n'est pas un numéro de carte.
    #[test]
    fn luhn_faux_accepte() {
        assert!(!looks_like_card_number("4111 1111 1111 1112"));
    }

    #[test]
    fn luhn_de_reference() {
        assert!(luhn_valid("79927398713"));
        assert!(!luhn_valid("79927398710"));
    }
}
