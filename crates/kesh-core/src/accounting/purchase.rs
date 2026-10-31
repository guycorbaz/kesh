//! Le montant d'une ligne de facture fournisseur — Story v017-3b (#687).
//!
//! Une ligne de facture fournisseur vaut `quantité × prix unitaire`. Ce produit
//! est écrit tel quel au journal (une ligne de débit de charge **par ligne**,
//! `generate_purchase_journal_lines` de `kesh-db`) et il entre dans le TTC, que
//! le règlement écrit au compte de la banque. Une ligne dont le produit n'est
//! pas **au centime** écrirait donc au grand livre un montant qu'aucun relevé
//! bancaire ne peut porter, et la réconciliation ne proposerait jamais son
//! règlement.
//!
//! ⚠️ **La règle porte sur chaque ligne, non sur le seul total** : deux lignes
//! `1 × 0.005` donnent un TTC au centime (`0.01`), mais chacune écrit au journal
//! une ligne de charge à `0.005`.
//!
//! [`purchase_line_total`] est la **seule définition** de cette règle : la
//! saisie (`create_in_tx`) et la complétion d'une pièce importée l'appellent par
//! la même passe de forme (`purchase_lines_form`, `kesh-db`), et le formulaire
//! de l'écran en porte le miroir (`supplier-invoice-form.ts`).
//!
//! Toute l'arithmétique est en [`rust_decimal::Decimal`] — jamais de `f64`.

use rust_decimal::Decimal;

/// Refus d'une ligne dont `quantité × prix` n'est pas au centime.
///
/// `line_total` est le produit **normalisé** (`4.9995`, jamais `4.999500`) :
/// c'est la valeur que le message montre à l'utilisateur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("le montant de la ligne ({line_total}) n'est pas au centime")]
pub struct LineNotInCentimes {
    /// Le produit `quantité × prix`, normalisé.
    pub line_total: Decimal,
}

/// Le montant d'une ligne de facture fournisseur, s'il est au centime.
///
/// Rend `quantity × unit_price` si ce produit, **après `normalize()`**, a au
/// plus deux décimales ; sinon [`LineNotInCentimes`], qui porte le produit
/// normalisé. La normalisation compte : `4 × 2.5000` vaut `10.0000` (échelle 4)
/// et **est** au centime.
///
/// ⚠️ Ne contrôle **ni le signe ni l'échelle des facteurs** : la positivité se
/// juge avant (passe de forme de `kesh-db`), l'échelle des facteurs à la
/// conversion de la requête (quatre décimales, `MAX_DECIMAL_SCALE`). Ne fait pas
/// de multiplication contrôlée (`checked_mul`) : c'est #825.
///
/// # Exemples
///
/// ```
/// use rust_decimal_macros::dec;
/// use kesh_core::accounting::purchase_line_total;
///
/// assert_eq!(purchase_line_total(dec!(2), dec!(0.125)), Ok(dec!(0.25)));
/// assert!(purchase_line_total(dec!(3), dec!(33.3333)).is_err());
/// ```
pub fn purchase_line_total(
    quantity: Decimal,
    unit_price: Decimal,
) -> Result<Decimal, LineNotInCentimes> {
    let line_total = quantity * unit_price;
    let normalized = line_total.normalize();
    if normalized.scale() <= 2 {
        Ok(line_total)
    } else {
        Err(LineNotInCentimes {
            line_total: normalized,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn refus(q: Decimal, p: Decimal) -> Decimal {
        purchase_line_total(q, p)
            .expect_err("la ligne doit être refusée")
            .line_total
    }

    #[test]
    fn une_ligne_au_centime_est_admise() {
        assert_eq!(purchase_line_total(dec!(1), dec!(12.50)), Ok(dec!(12.50)));
    }

    #[test]
    fn un_produit_a_quatre_decimales_est_refuse() {
        assert_eq!(refus(dec!(3), dec!(33.3333)), dec!(99.9999));
    }

    #[test]
    fn un_prix_a_trois_decimales_dont_le_produit_est_au_centime_est_admis() {
        assert_eq!(purchase_line_total(dec!(2), dec!(0.125)), Ok(dec!(0.25)));
    }

    #[test]
    fn une_quantite_decimale_dont_le_produit_n_est_pas_au_centime_est_refusee() {
        assert_eq!(refus(dec!(1.5), dec!(3.333)), dec!(4.9995));
    }

    /// Tue M1 : sans `normalize()`, `10.0000` a l'échelle 4 et serait refusé.
    #[test]
    fn l_echelle_se_juge_apres_normalisation() {
        let total = purchase_line_total(dec!(4), dec!(2.5000)).expect("10.0000 est au centime");
        assert_eq!(total, dec!(10));
    }

    #[test]
    fn de_petits_facteurs_au_centime_sont_admis() {
        assert_eq!(purchase_line_total(dec!(0.1), dec!(0.1)), Ok(dec!(0.01)));
        assert_eq!(purchase_line_total(dec!(0.3), dec!(0.1)), Ok(dec!(0.03)));
    }

    /// Le produit refusé est rendu normalisé : `4.9995`, jamais `4.999500`.
    #[test]
    fn le_produit_refuse_est_normalise() {
        let total = refus(dec!(1.50), dec!(3.3330));
        assert_eq!(total.to_string(), "4.9995");
    }
}
