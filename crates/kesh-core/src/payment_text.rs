//! L'ensemble des **blancs d'une saisie de facture fournisseur** — coordonnées de
//! paiement (Story 30-2b, AC12-bis) et numéro de facture du fournisseur (Story
//! 48-12a, #975) — et ses normalisations : **une seule définition** côté serveur.
//!
//! L'ensemble est : les caractères `White_Space` d'Unicode (`char::is_whitespace`
//! — espace, tabulation, U+00A0, U+202F, U+0085… compris), plus U+200B (espace de
//! largeur nulle) et U+FEFF (BOM). `kesh-api` l'importe (validation des
//! coordonnées de paiement), `kesh-db` s'en sert (comparaison des numéros de
//! facture du refus de doublon) ; ni l'un ni l'autre n'en porte de copie.
//!
//! ⚠️ **Ce n'est pas [`crate::text::is_invisible`].** Ce dernier est plus large
//! (caractères de contrôle, U+00AD, U+200C à U+200F, U+2060 à U+2064) et sert à
//! un autre usage : dire qu'une valeur ne *marque* rien à l'écran (nom de
//! contact, champ obligatoire vide). Ici, l'ensemble est **écrit** par l'AC12-bis
//! et tenu à l'identique côté écran (`normalizeIban`,
//! `frontend/src/lib/shared/utils/iban.ts`) : un test de chaque côté porte les
//! mêmes sept caractères (U+0020, U+0009, U+00A0, U+202F, U+0085, U+200B,
//! U+FEFF). Aucun troisième sens de « blanc » ne doit naître : un besoin neuf
//! choisit l'un des deux, en le nommant.

/// Un caractère de l'ensemble des blancs de l'AC12-bis : `White_Space` d'Unicode,
/// plus U+200B et U+FEFF.
pub fn is_payment_blank(c: char) -> bool {
    c.is_whitespace() || c == '\u{200B}' || c == '\u{FEFF}'
}

/// Retire d'une coordonnée de paiement **tous** les blancs de l'ensemble
/// ([`is_payment_blank`]), où qu'ils soient.
///
/// `Iban::new` n'ôte que l'espace ASCII et refuse tout non-ASCII : sans cette
/// normalisation, un IBAN collé depuis un PDF (avec des insécables) serait
/// refusé.
pub fn normalize_payment_field(raw: &str) -> String {
    raw.chars().filter(|c| !is_payment_blank(*c)).collect()
}

/// La forme **comparable** d'un numéro de facture du fournisseur (Story 48-12a,
/// D2) : tous les blancs de l'ensemble retirés, puis `to_lowercase()`. Rend
/// `None` quand rien ne reste — un numéro vide ou fait de blancs vaut
/// **absent**.
///
/// `"FA 2026-1"`, `"fa2026-1"`, `"FA\u{A0}2026-1"` et `" FA2026-1\u{200B}"` ont
/// la même forme comparable, `"fa2026-1"`.
pub fn normalize_supplier_invoice_number(raw: Option<&str>) -> Option<String> {
    let normalized = normalize_payment_field(raw?).to_lowercase();
    (!normalized.is_empty()).then_some(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les sept blancs de l'ensemble écrit, par points de code (un caractère
    /// invisible littéral pourrait disparaître sous un formateur) — la même liste
    /// que `iban.test.ts` côté écran.
    const SEPT_BLANCS: [char; 7] = [
        '\u{0020}', '\u{0009}', '\u{00A0}', '\u{202F}', '\u{0085}', '\u{200B}', '\u{FEFF}',
    ];

    #[test]
    fn les_sept_blancs_de_l_ensemble_ecrit() {
        for b in SEPT_BLANCS {
            assert!(is_payment_blank(b), "U+{:04X} doit être un blanc", b as u32);
            assert_eq!(
                normalize_payment_field(&format!("{b}CH93{b}0076{b}")),
                "CH930076",
                "U+{:04X} doit être retiré",
                b as u32
            );
        }
    }

    #[test]
    fn hors_de_l_ensemble_rien_n_est_retire() {
        // U+00AD et U+2060 sont « invisibles » pour `text::is_invisible`, pas des
        // blancs ici : l'écart entre les deux notions est voulu.
        for c in ['a', '-', '\u{00AD}', '\u{2060}', '\u{200C}'] {
            assert!(!is_payment_blank(c), "U+{:04X}", c as u32);
        }
    }

    #[test]
    fn numeros_egaux_apres_normalisation() {
        let attendu = Some("fa2026-1".to_string());
        for raw in [
            "FA 2026-1",
            "fa2026-1",
            "FA\u{A0}2026-1",
            " FA2026-1\u{200B}",
            "\u{FEFF}Fa\u{202F}2026-1\u{0085}",
        ] {
            assert_eq!(
                normalize_supplier_invoice_number(Some(raw)),
                attendu,
                "{raw:?}"
            );
        }
        for b in SEPT_BLANCS {
            assert_eq!(
                normalize_supplier_invoice_number(Some(&format!("FA{b}2026-1"))),
                attendu,
                "U+{:04X}",
                b as u32
            );
        }
    }

    #[test]
    fn numero_vide_ou_blanc_vaut_absent() {
        assert_eq!(normalize_supplier_invoice_number(None), None);
        assert_eq!(normalize_supplier_invoice_number(Some("")), None);
        let que_des_blancs: String = SEPT_BLANCS.iter().collect();
        assert_eq!(
            normalize_supplier_invoice_number(Some(&que_des_blancs)),
            None
        );
    }

    #[test]
    fn la_casse_est_ignoree_mais_pas_les_autres_signes() {
        // La casse (revue P1, A10 : sans majuscule, le test ne la tenait pas).
        assert_eq!(
            normalize_supplier_invoice_number(Some("Fa-1")),
            normalize_supplier_invoice_number(Some("fA-1"))
        );
        assert_eq!(
            normalize_supplier_invoice_number(Some("15")),
            normalize_supplier_invoice_number(Some(" 15 "))
        );
        assert_ne!(
            normalize_supplier_invoice_number(Some("15")),
            normalize_supplier_invoice_number(Some("15-b"))
        );
        // Les autres signes restent : « 15-b » n'est pas « 15b ».
        assert_ne!(
            normalize_supplier_invoice_number(Some("15-b")),
            normalize_supplier_invoice_number(Some("15b"))
        );
    }
}
