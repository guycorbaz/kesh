//! Plan d'une **conversion de prestataire** — Story 38-11c2 (AC-8), règle
//! commune aux prestataires (décision de l'orchestrateur du 2026-11-20),
//! appelée telle quelle par PayPal, Wise (39-11c AC-3) et Stripe.
//!
//! Fonction pure ([`plan_conversion`]) : de la moitié **débitée** (la source,
//! d'où sort l'argent) et de la moitié **créditée** (la cible), elle rend ce
//! qu'il faut écrire, que le dépôt passe par
//! `journal_entry_conversions::create_provider_conversion_in_tx` :
//!
//! - **un côté en CHF** → [`ConversionPlan::Single`], **une** écriture : la
//!   ligne CHF au montant réel du côté CHF, la ligne en devise au montant de
//!   l'autre côté, cours `rate_from_chf(montant en devise, CHF)`, provenance
//!   `manual` — le CHF réel fait foi, aucun cours de Kesh ;
//! - **deux devises étrangères** → [`ConversionPlan::Double`], **deux**
//!   écritures par le compte de conversion : CHF `X` = montant source au cours
//!   mensuel retenu (`source_rate`), cible au même `X` par
//!   `rate_from_chf(montant cible, X)`, provenance `manual`.

use kesh_core::accounting::fx::{ForeignAmount, RateProvenance, fx_chf, rate_from_chf};
use kesh_db::repositories::journal_entry_conversions::{ConversionLeg, ConversionPlan};
use rust_decimal::Decimal;

use crate::provider_review::BASE_CURRENCY;

/// Une moitié d'une conversion : compte du plan, devise, montant (> 0, en
/// valeur absolue).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversionSide {
    /// Le compte du plan du compte bancaire de la moitié.
    pub ledger_account_id: i64,
    /// Sa devise (`CHF` pour la monnaie de base).
    pub currency: String,
    /// Le montant, en valeur absolue.
    pub amount: Decimal,
}

/// Le cours mensuel retenu du côté source, et sa provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceRate {
    /// Cours par unité de devise.
    pub rate: Decimal,
    /// `Afc` ou `Manual`, selon la source du cours retenu.
    pub source: RateProvenance,
}

/// Refus du plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanError {
    /// Un montant nul (ou négatif).
    ZeroAmount,
    /// Les deux moitiés sont de la même devise : ce n'est pas une conversion.
    SameCurrency,
    /// Deux devises étrangères sans cours mensuel du côté source.
    MissingRate,
    /// Le cours déduit du CHF (ou le CHF au cours) n'a pu se calculer
    /// (`FX_RATE_INVALID`) : `currency` est la devise du côté dont le cours
    /// est en cause (le côté en devise d'une conversion avec le CHF, la source
    /// ou la cible d'une conversion entre deux devises étrangères).
    RateInvalid {
        /// La devise du côté en cause.
        currency: String,
        /// Le refus du calcul.
        message: String,
    },
}

/// Le plan — voir l'en-tête du module.
pub fn plan_conversion(
    source: &ConversionSide,
    target: &ConversionSide,
    source_rate: Option<SourceRate>,
) -> Result<ConversionPlan, PlanError> {
    if source.amount <= Decimal::ZERO || target.amount <= Decimal::ZERO {
        return Err(PlanError::ZeroAmount);
    }
    if source.currency == target.currency {
        return Err(PlanError::SameCurrency);
    }
    let derived = |side: &ConversionSide, chf: Decimal| -> Result<ConversionLeg, PlanError> {
        let rate = rate_from_chf(&side.amount, &chf).map_err(|e| PlanError::RateInvalid {
            currency: side.currency.clone(),
            message: format!("{e:?}"),
        })?;
        Ok(ConversionLeg {
            ledger_account_id: side.ledger_account_id,
            chf,
            foreign: Some(foreign(side.amount, rate, RateProvenance::Manual)),
        })
    };
    let chf_leg = |side: &ConversionSide| ConversionLeg {
        ledger_account_id: side.ledger_account_id,
        chf: side.amount,
        foreign: None,
    };
    if source.currency == BASE_CURRENCY {
        return Ok(ConversionPlan::Single {
            debit: derived(target, source.amount)?,
            credit: chf_leg(source),
        });
    }
    if target.currency == BASE_CURRENCY {
        return Ok(ConversionPlan::Single {
            debit: chf_leg(target),
            credit: derived(source, target.amount)?,
        });
    }
    let SourceRate {
        rate,
        source: provenance,
    } = source_rate.ok_or(PlanError::MissingRate)?;
    let chf = fx_chf(&source.amount, &rate).map_err(|e| PlanError::RateInvalid {
        currency: source.currency.clone(),
        message: format!("{e:?}"),
    })?;
    Ok(ConversionPlan::Double {
        source: ConversionLeg {
            ledger_account_id: source.ledger_account_id,
            chf,
            foreign: Some(foreign(source.amount, rate, provenance)),
        },
        target: derived(target, chf)?,
    })
}

/// La partie en devise d'une ligne interne (jamais de devise annoncée, jamais
/// de cours déduit au sens de l'API).
fn foreign(amount: Decimal, rate: Decimal, source: RateProvenance) -> ForeignAmount {
    ForeignAmount {
        declared_currency: None,
        amount,
        rate,
        source: Some(source),
        rate_derived: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn side(ledger: i64, currency: &str, amount: Decimal) -> ConversionSide {
        ConversionSide {
            ledger_account_id: ledger,
            currency: currency.into(),
            amount,
        }
    }

    #[test]
    fn chf_vers_eur_une_ecriture_au_chf_reel() {
        let plan = plan_conversion(
            &side(1020, "CHF", dec!(93.00)),
            &side(1022, "EUR", dec!(100.00)),
            None,
        )
        .unwrap();
        let ConversionPlan::Single { debit, credit } = plan else {
            panic!("une écriture attendue");
        };
        assert_eq!(credit.ledger_account_id, 1020);
        assert_eq!(credit.chf, dec!(93.00));
        assert!(credit.foreign.is_none());
        assert_eq!(debit.ledger_account_id, 1022);
        assert_eq!(debit.chf, dec!(93.00));
        let f = debit.foreign.unwrap();
        assert_eq!(f.amount, dec!(100.00));
        assert_eq!(f.rate, dec!(0.93));
        assert_eq!(f.source, Some(RateProvenance::Manual));
    }

    #[test]
    fn eur_vers_chf_une_ecriture() {
        let plan = plan_conversion(
            &side(1022, "EUR", dec!(100.00)),
            &side(1020, "CHF", dec!(92.37)),
            None,
        )
        .unwrap();
        let ConversionPlan::Single { debit, credit } = plan else {
            panic!("une écriture attendue");
        };
        assert_eq!(debit.ledger_account_id, 1020);
        assert_eq!(debit.chf, dec!(92.37));
        assert_eq!(credit.chf, dec!(92.37));
        let f = credit.foreign.unwrap();
        assert_eq!(fx_chf(&f.amount, &f.rate).unwrap(), dec!(92.37));
    }

    #[test]
    fn eur_vers_usd_deux_ecritures_au_meme_chf() {
        let rate = SourceRate {
            rate: dec!(0.95),
            source: RateProvenance::Afc,
        };
        let plan = plan_conversion(
            &side(1022, "EUR", dec!(100.00)),
            &side(1021, "USD", dec!(108.10)),
            Some(rate),
        )
        .unwrap();
        let ConversionPlan::Double { source, target } = plan else {
            panic!("deux écritures attendues");
        };
        assert_eq!(source.chf, dec!(95.00));
        assert_eq!(target.chf, dec!(95.00));
        assert_eq!(
            source.foreign.as_ref().unwrap().source,
            Some(RateProvenance::Afc)
        );
        let t = target.foreign.unwrap();
        assert_eq!(t.amount, dec!(108.10));
        assert_eq!(t.source, Some(RateProvenance::Manual));
        assert_eq!(fx_chf(&t.amount, &t.rate).unwrap(), dec!(95.00));
    }

    #[test]
    fn refus_du_plan() {
        assert_eq!(
            plan_conversion(&side(1, "EUR", dec!(10)), &side(2, "CHF", dec!(0)), None),
            Err(PlanError::ZeroAmount)
        );
        assert_eq!(
            plan_conversion(&side(1, "EUR", dec!(10)), &side(2, "EUR", dec!(10)), None),
            Err(PlanError::SameCurrency)
        );
        assert_eq!(
            plan_conversion(&side(1, "EUR", dec!(10)), &side(2, "USD", dec!(11)), None),
            Err(PlanError::MissingRate)
        );
        // CHF au cours qui s'arrondit à zéro : refusé, jamais une écriture nulle
        // — au cours de la cible, déduit d'un CHF nul (la devise nommée est
        // celle de la cible).
        let tiny = SourceRate {
            rate: dec!(0.0001),
            source: RateProvenance::Manual,
        };
        assert!(matches!(
            plan_conversion(
                &side(1, "EUR", dec!(1)),
                &side(2, "USD", dec!(1)),
                Some(tiny)
            ),
            Err(PlanError::RateInvalid { currency, .. }) if currency == "USD"
        ));
        // L4 (revue P1) : le refus nomme la devise du côté dont le cours est
        // déduit — la source EUR d'une conversion EUR→CHF, jamais le CHF.
        assert!(matches!(
            plan_conversion(
                &side(1, "EUR", dec!(999999999)),
                &side(2, "CHF", dec!(0.01)),
                None
            ),
            Err(PlanError::RateInvalid { currency, .. }) if currency == "EUR"
        ));
        assert!(matches!(
            plan_conversion(
                &side(1, "CHF", dec!(0.01)),
                &side(2, "USD", dec!(999999999)),
                None
            ),
            Err(PlanError::RateInvalid { currency, .. }) if currency == "USD"
        ));
    }
}
