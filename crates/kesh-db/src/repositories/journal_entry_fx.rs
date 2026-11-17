//! Contrôle central des lignes en devise d'une écriture (Story 37-3a, AC-4).
//!
//! Appelé par les **deux seuls** chemins d'`INSERT` de lignes d'écriture —
//! `journal_entries::create_in_tx_inner` et `journal_entries::update_in_tx` —,
//! juste après la garde des comptes (rang 1, qui rend la devise de chaque
//! compte, lue `LOCK IN SHARE MODE`). Ordre des refus, une seule raison, la
//! plus bloquante d'abord :
//!
//! 2. ligne sur un compte en devise dans un exercice mono-devise → 409
//!    `MULTI_CURRENCY_DISABLED` (`details.fiscalYearId`) ;
//!
//!    Rang 2 bis : deux devises étrangères dans l'écriture →
//!    `FX_MULTIPLE_CURRENCIES` (exemptées : ouverture et complément,
//!    contre-passation ; au `PUT`, refus seulement si l'ensemble des devises
//!    s'élargit) ;
//! 3. accord de chaque ligne avec la devise de son compte, première ligne
//!    fautive : `ACCOUNT_REQUIRES_CURRENCY`, `FX_ACCOUNT_IN_CHF`,
//!    `FX_CURRENCY_MISMATCH` ;
//! 4. provenance du cours : une ligne `afc` porte le cours `afc` retenu par la
//!    société pour le mois de la date (`FX_RATE_NOT_AFC`, 404
//!    `RATE_UNAVAILABLE`) ; une ligne sans provenance est qualifiée `afc` ou
//!    `manual` ; une ligne au cours déduit prend le cours retenu s'il redonne
//!    son CHF (A32). Au `PUT`, une ligne identique à une ligne d'avant garde sa
//!    provenance (A17, CR-6). La contre-passation est exemptée de ce rang.
//!
//! Le cours de la société se lit **sans verrou**
//! (`exchange_rates::find_company_rate_in_tx`) : un ↑ ou un `PUT …/rates`
//! validé pendant la transaction n'est pas vu — limite assumée, sans effet sur
//! les autres lignes.

use std::collections::{BTreeSet, HashMap};

use chrono::NaiveDate;
use kesh_core::accounting::fx::{RateProvenance, fx_chf};
use kesh_core::exchange_rates::{RateKind, first_of_month};
use rust_decimal::Decimal;

use crate::entities::{JournalEntryLine, NewJournalEntryLine};
use crate::errors::DbError;
use crate::repositories::exchange_rates::{CompanyRate, find_company_rate_in_tx};

/// Portée de la règle d'une devise étrangère par écriture (R7, AC-4 rang 2 bis).
///
/// Paramètre **privé** du dépôt : jamais un champ public de `NewJournalEntry`
/// (un indicateur public permettrait à un appelant de se dispenser du
/// contrôle).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PorteeDevises {
    /// Toute écriture ordinaire : une seule devise étrangère.
    UneSeule,
    /// Écriture d'ouverture et son complément : plusieurs devises admises.
    Ouverture,
}

/// Ce qui précède l'écriture contrôlée.
pub(crate) struct Contexte<'a> {
    /// Société.
    pub company_id: i64,
    /// Exercice de l'écriture.
    pub fiscal_year_id: i64,
    /// Mode de l'exercice (`fiscal_years.multi_currency`, lu sous `FOR UPDATE`).
    pub exercice_multi_devises: bool,
    /// Date de l'écriture (mois du cours `afc`).
    pub entry_date: NaiveDate,
    /// Devise de chaque compte, lue au rang 1 (`LOCK IN SHARE MODE`).
    pub devises: &'a HashMap<i64, Option<String>>,
    /// Portée de la règle d'une devise.
    pub portee: PorteeDevises,
    /// Contre-passation (`reverses_entry_id.is_some()`) : exemptée des rangs
    /// 2 bis et 4.
    pub contre_passation: bool,
    /// Au `PUT` : les lignes d'avant (relues avec la devise de leur compte) et
    /// la date d'avant.
    pub avant: Option<(&'a [JournalEntryLine], NaiveDate)>,
}

/// Contrôle et complète les lignes (`source` résolue, cours déduit remplacé
/// par le cours retenu quand il le faut). En sortie, toute ligne en devise
/// porte `source: Some(_)`.
pub(crate) async fn controler_lignes(
    conn: &mut sqlx::MySqlConnection,
    ctx: &Contexte<'_>,
    lines: &mut [NewJournalEntryLine],
) -> Result<(), DbError> {
    let devise_de = |account_id: i64| -> Option<&String> {
        ctx.devises.get(&account_id).and_then(|d| d.as_ref())
    };

    // Rang 2 — compte en devise dans un exercice mono-devise.
    if !ctx.exercice_multi_devises && lines.iter().any(|l| devise_de(l.account_id).is_some()) {
        return Err(DbError::MultiCurrencyDisabled {
            fiscal_year_id: Some(ctx.fiscal_year_id),
        });
    }

    // Rang 2 bis — une seule devise étrangère (jugée sur les comptes).
    let devises_corps: BTreeSet<String> = lines
        .iter()
        .filter_map(|l| devise_de(l.account_id).cloned())
        .collect();
    if devises_corps.len() > 1 && ctx.portee == PorteeDevises::UneSeule && !ctx.contre_passation {
        let elargit = match ctx.avant {
            None => true,
            Some((avant, _)) => {
                let devises_avant: BTreeSet<&str> = avant
                    .iter()
                    .filter_map(|l| l.account_currency.as_deref())
                    .collect();
                devises_corps
                    .iter()
                    .any(|d| !devises_avant.contains(d.as_str()))
            }
        };
        if elargit {
            return Err(DbError::FxMultipleCurrencies {
                currencies: devises_corps.into_iter().collect(),
            });
        }
    }

    // Rang 3 — accord de chaque ligne avec la devise de son compte.
    for (idx, l) in lines.iter().enumerate() {
        let line = idx + 1;
        match (devise_de(l.account_id), &l.foreign) {
            (Some(_), None) => return Err(DbError::AccountRequiresCurrency { line }),
            (None, Some(_)) => return Err(DbError::FxAccountInChf { line }),
            (Some(devise), Some(f)) => {
                if let Some(annoncee) = &f.declared_currency
                    && annoncee != devise
                {
                    return Err(DbError::FxCurrencyMismatch {
                        line,
                        expected: devise.clone(),
                        provided: annoncee.clone(),
                    });
                }
            }
            (None, None) => {}
        }
    }

    // Rang 4 — provenance du cours. La contre-passation recopie l'origine.
    if ctx.contre_passation {
        return verifier_provenances(lines);
    }
    let retenues = apparier_avec_avant(ctx, lines);
    let mois = first_of_month(ctx.entry_date);
    let mut cache: HashMap<String, Option<CompanyRate>> = HashMap::new();
    for (idx, l) in lines.iter_mut().enumerate() {
        let line = idx + 1;
        let Some(devise) = ctx.devises.get(&l.account_id).and_then(|d| d.clone()) else {
            continue;
        };
        let chf = l.debit + l.credit;
        let Some(f) = l.foreign.as_mut() else {
            continue;
        };
        if let Some((source, cours)) = retenues.get(&idx) {
            // Ligne identique à une ligne d'avant : provenance (et, au cours
            // déduit, cours) stockées, sans lecture.
            f.source = Some(*source);
            if f.rate_derived {
                f.rate = *cours;
            }
            continue;
        }
        if matches!(
            f.source,
            Some(RateProvenance::Manual) | Some(RateProvenance::Revaluation)
        ) {
            continue;
        }
        if !cache.contains_key(&devise) {
            let r = find_company_rate_in_tx(conn, ctx.company_id, &devise, RateKind::Month, mois)
                .await?;
            cache.insert(devise.clone(), r);
        }
        let retenu = cache.get(&devise).and_then(|r| r.as_ref());
        let cours_afc = retenu.filter(|r| r.source.is_afc()).map(|r| r.rate);
        // Le cours retenu convient-il à cette ligne ?
        let convient = |cours: Decimal| -> bool {
            if f.rate_derived {
                matches!(fx_chf(&f.amount, &cours), Ok(c) if c == chf)
            } else {
                f.rate == cours
            }
        };
        match f.source {
            Some(RateProvenance::Afc) => match (retenu, cours_afc) {
                (None, _) => {
                    return Err(DbError::RateUnavailable {
                        currency: devise,
                        month: mois,
                    });
                }
                (Some(_), None) => {
                    return Err(DbError::FxRateNotAfc {
                        line,
                        expected_rate: None,
                    });
                }
                (Some(_), Some(cours)) => {
                    if !convient(cours) {
                        return Err(DbError::FxRateNotAfc {
                            line,
                            expected_rate: Some(cours),
                        });
                    }
                    f.rate = cours;
                }
            },
            None => match cours_afc {
                Some(cours) if convient(cours) => {
                    f.rate = cours;
                    f.source = Some(RateProvenance::Afc);
                }
                _ => f.source = Some(RateProvenance::Manual),
            },
            Some(RateProvenance::Manual) | Some(RateProvenance::Revaluation) => {}
        }
    }
    verifier_provenances(lines)
}

/// Aucune ligne en devise ne part vers l'`INSERT` sans provenance (un `None`
/// est un invariant violé, jamais une ligne écrite).
pub(crate) fn verifier_provenances(lines: &[NewJournalEntryLine]) -> Result<(), DbError> {
    if lines
        .iter()
        .any(|l| l.foreign.as_ref().is_some_and(|f| f.source.is_none()))
    {
        return Err(DbError::Invariant(
            "ligne en devise sans provenance à l'INSERT — devait être qualifiée".into(),
        ));
    }
    Ok(())
}

/// Au `PUT`, l'appariement **unique** des lignes du corps aux lignes d'avant
/// (AC-4, rang 4 : A17, CR6-P3-1, CR6-P3-2, CR6-P4-1, CR6-P4-2), sans égard à
/// l'ordre. Rend, par indice de ligne du corps, la provenance et le cours
/// stockés à reprendre.
///
/// Ordre : lignes à provenance explicite (`afc`, `manual`) d'abord, puis sans
/// provenance ; dans chaque groupe, celles qui portent `rate` avant celles au
/// cours déduit. Une ligne `afc` ou sans provenance ne s'apparie que si la date
/// n'a pas changé ; une ligne `manual` s'apparie toujours (elle n'est jamais
/// contrôlée). Identité : compte, débit, crédit, montant en devise (en valeur)
/// et provenance (sauf ligne sans provenance), plus le cours — sauf au cours
/// déduit, jugé sans le cours. Le projet n'entre pas dans la comparaison.
fn apparier_avec_avant(
    ctx: &Contexte<'_>,
    lines: &[NewJournalEntryLine],
) -> HashMap<usize, (RateProvenance, Decimal)> {
    let mut retenues = HashMap::new();
    let Some((avant, date_avant)) = ctx.avant else {
        return retenues;
    };
    let meme_date = date_avant == ctx.entry_date;
    let mut prises = vec![false; avant.len()];
    let rang = |l: &NewJournalEntryLine| -> Option<u8> {
        let f = l.foreign.as_ref()?;
        let groupe = match f.source {
            Some(RateProvenance::Afc) | Some(RateProvenance::Manual) => 0,
            None => 1,
            Some(RateProvenance::Revaluation) => return None,
        };
        Some(groupe * 2 + u8::from(f.rate_derived))
    };
    let mut ordre: Vec<(u8, usize)> = lines
        .iter()
        .enumerate()
        .filter_map(|(i, l)| rang(l).map(|r| (r, i)))
        .collect();
    ordre.sort();
    for (_, i) in ordre {
        let l = &lines[i];
        let Some(f) = l.foreign.as_ref() else {
            continue;
        };
        if f.source != Some(RateProvenance::Manual) && !meme_date {
            continue;
        }
        let trouve = avant.iter().enumerate().find(|(j, b)| {
            if prises[*j] {
                return false;
            }
            let (Some(b_montant), Some(b_cours), Some(b_source)) = (
                b.amount_currency,
                b.exchange_rate,
                b.rate_source
                    .as_deref()
                    .and_then(|s| s.parse::<RateProvenance>().ok()),
            ) else {
                return false;
            };
            if b_source == RateProvenance::Revaluation {
                return false;
            }
            if let Some(s) = f.source
                && s != b_source
            {
                return false;
            }
            b.account_id == l.account_id
                && b.debit == l.debit
                && b.credit == l.credit
                && b_montant == f.amount
                && (f.rate_derived || b_cours == f.rate)
        });
        if let Some((j, b)) = trouve {
            prises[j] = true;
            let source = b
                .rate_source
                .as_deref()
                .and_then(|s| s.parse().ok())
                .unwrap_or(RateProvenance::Manual);
            retenues.insert(i, (source, b.exchange_rate.unwrap_or(f.rate)));
        }
    }
    retenues
}
