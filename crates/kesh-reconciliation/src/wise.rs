//! Paramétrage **Wise** de la revue et de l'acceptation communes — Story
//! 39-11c (AC-2 à AC-6, amendement du 21.11).
//!
//! Fonctions pures, sans I/O :
//!
//! - le classement d'une transaction Wise ([`classify`]), sur l'identifiant
//!   posé par l'import (39-11b : `transaction_id` = l'identifiant Wise,
//!   `provider_type` = son préfixe), le signe, la contrepartie et le segment
//!   de conversion du libellé ([`exchange_of`]) ;
//! - le **paiement par carte converti** ([`card_groups`], amendement A-1) :
//!   les lignes `CARD-<id>` d'un même paiement (une par solde débité) et ses
//!   lignes `FEE-CARD-<id>`, regroupées en **une** écriture (la charge au
//!   montant du marchand, chaque solde crédité de sa part, la différence de
//!   change sur sa ligne) ;
//! - les écarts propres à Wise ([`fees_without_origin`]) ;
//! - le paramétrage [`SPEC`] que la route Wise passe au moteur commun
//!   ([`crate::provider_review`]) et le type de proposition [`WiseProposal`]
//!   (les genres communs, plus `card_group`).
//!
//! # Classement ([`classify`]), dans cet ordre
//!
//! 1. **Frais** — identifiant commençant par `FEE-`.
//! 2. **Versement Stripe** ([`KIND_STRIPE_PAYOUT`]) — montant > 0 et
//!    contrepartie contenant « stripe », sans casse. Jugé **avant** la
//!    conversion : un dépôt porte aussi le préfixe `BALANCE` (amendement A-5).
//! 3. **Conversion** — préfixe `BALANCE` ou `CONVERSION_ORDER` **et** segment de
//!    conversion au libellé.
//! 4. **Virement** ([`KIND_TRANSFER`]) — préfixe `TRANSFER` : l'autre côté
//!    d'un virement entre ses comptes peut déjà être passé (AC-4).
//! 5. **Autre** — achats par carte, dépôts, le reste : les règles.
//!
//! Le versement Stripe et le virement sont des genres **propres**
//! ([`ProviderKind::Specific`]) : la revue commune les laisse à la route Wise,
//! qui lit l'autre côté déjà passé, puis les règles pour un virement.

use std::collections::{BTreeMap, HashMap, HashSet};

use kesh_db::entities::BankTransaction;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::provider_review::{
    BASE_CURRENCY, EntryKind, Placement, Proposal, ProviderKind, ProviderSpec,
};

/// Valeur de `bank_accounts.provider` d'un compte Wise.
pub const PROVIDER: &str = "wise";

/// Préfixe de l'identifiant d'une ligne de **frais** Wise.
pub const FEE_PREFIX: &str = "FEE-";
/// Préfixe de l'identifiant d'un paiement par **carte**.
pub const CARD_PREFIX: &str = "CARD-";
/// Préfixes (`provider_type`) des deux moitiés d'une conversion.
pub const CONVERSION_PREFIXES: [&str; 2] = ["BALANCE", "CONVERSION_ORDER"];
/// Préfixe (`provider_type`) d'un virement ou d'un dépôt par virement.
pub const TRANSFER_PREFIX: &str = "TRANSFER";

/// Genre propre : versement Stripe reçu sur Wise.
pub const KIND_STRIPE_PAYOUT: &str = "stripe_payout";
/// Genre propre : virement (préfixe `TRANSFER`).
pub const KIND_TRANSFER: &str = "transfer";

/// Écart : une moitié de conversion sans l'autre (pas encore importée, ou
/// déjà rapprochée).
pub const GAP_CONVERSION_HALF_MISSING: &str = "conversion_half_missing";
/// Écart : ligne de frais `FEE-x` sans transaction `x` sur le même compte
/// bancaire.
pub const GAP_FEE_WITHOUT_ORIGIN: &str = "fee_without_origin";
/// Écart : un relevé ne prolonge pas le précédent du compte (règle de 39-11b).
pub const GAP_BALANCE_GAP: &str = "balance_gap";
/// À examiner : plusieurs écritures sont l'autre côté possible.
pub const REVIEW_AMBIGUOUS_LINK: &str = "ambiguous_link";
/// À examiner : versement Stripe sans l'écriture de son versement.
pub const REVIEW_STRIPE_PAYOUT_UNMATCHED: &str = "stripe_payout_unmatched";
/// À examiner : paiement par carte converti dont les lignes n'ont pas la forme
/// attendue (amendement A-1).
pub const REVIEW_CARD_GROUP_UNSUPPORTED: &str = "card_group_unsupported";
/// À examiner : paiement par carte converti sans règle de contrepartie en CHF.
pub const REVIEW_CARD_GROUP_NO_RULE: &str = "card_group_no_rule";
/// À examiner : paiement par carte converti dont un solde du compte Wise,
/// sans ligne dans le groupe, peut en avoir une qui n'est pas encore importée
/// — le solde en CHF toujours, un solde en devise quand ses relevés encadrent
/// la date sans se suivre — : aucun relevé ne couvre la date du paiement, ni
/// ne l'atteste par la continuité des soldes (l'autre moitié d'une conversion
/// partielle peut manquer).
pub const REVIEW_CARD_GROUP_STATEMENTS_MISSING: &str = "card_group_statements_missing";
/// À examiner : paiement par carte converti dont une ligne (`CARD-<id>` ou
/// `FEE-CARD-<id>`, sur un solde du compte Wise) est **déjà rapprochée** —
/// une partielle dont une part a été passée seule : l'écrire donnerait une
/// seconde écriture du même paiement.
pub const REVIEW_CARD_GROUP_PARTLY_RECONCILED: &str = "card_group_partly_reconciled";

/// Écart maximal, en jours, entre la date de paiement d'une transaction et
/// l'écriture déjà passée de l'autre côté (AC-4).
pub const LINK_MAX_DAYS: i64 = 5;

/// Le segment de conversion du libellé d'une transaction Wise, écrit par
/// l'import (39-11b) : `DE→VERS [MONTANT] [au cours COURS]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exchange {
    /// Devise source.
    pub from: String,
    /// Devise cible.
    pub to: String,
    /// Montant reçu dans la devise cible (`Exchange To Amount`).
    pub to_amount: Option<Decimal>,
    /// Cours (`Exchange Rate`).
    pub rate: Option<Decimal>,
}

/// Un code de devise : trois lettres ASCII majuscules.
fn is_currency(s: &str) -> bool {
    s.len() == 3 && s.bytes().all(|b| b.is_ascii_uppercase())
}

/// Un décimal positif, tel qu'écrit par l'import.
fn positive_decimal(s: &str) -> Option<Decimal> {
    s.parse::<Decimal>().ok().filter(|d| *d > Decimal::ZERO)
}

/// Lit un segment ; `None` s'il n'en a pas **exactement** la forme.
fn parse_segment(segment: &str) -> Option<Exchange> {
    let (from, rest) = segment.split_once('→')?;
    let (to, rest) = match rest.split_once(' ') {
        Some((to, rest)) => (to, rest),
        None => (rest, ""),
    };
    if !is_currency(from) || !is_currency(to) {
        return None;
    }
    let (amount_part, rate_part) = match rest.split_once("au cours ") {
        Some((a, r)) => (a.trim_end(), Some(r)),
        None => (rest, None),
    };
    let to_amount = if amount_part.is_empty() {
        None
    } else {
        Some(positive_decimal(amount_part)?)
    };
    let rate = match rate_part {
        Some(r) => Some(positive_decimal(r)?),
        None => None,
    };
    Some(Exchange {
        from: from.to_string(),
        to: to.to_string(),
        to_amount,
        rate,
    })
}

/// Le segment de conversion d'un libellé (éléments séparés par ` · `), ou
/// `None`. Seul un élément qui en a **exactement** la forme compte : une
/// description qui contient une flèche n'en est pas un. L'import écrit la tête
/// (description, type, numéro), puis le segment, puis la note : c'est le
/// **dernier** élément conforme qui compte (une description conforme, placée
/// avant, ne l'emporte pas ; une note conforme, après lui, le ferait : cas
/// non traité).
pub fn exchange_of(details: &str) -> Option<Exchange> {
    details.rsplit(" · ").find_map(parse_segment)
}

/// L'identifiant Wise d'une transaction (vide sans identifiant).
fn wise_id(t: &BankTransaction) -> &str {
    t.transaction_id.as_deref().unwrap_or("")
}

/// Le préfixe posé par l'import (`provider_type`), sans blancs de bord.
fn prefix(t: &BankTransaction) -> &str {
    t.provider_type.as_deref().unwrap_or("").trim()
}

/// Le genre d'une transaction Wise — voir l'en-tête du module.
pub fn classify(t: &BankTransaction) -> ProviderKind {
    if wise_id(t).starts_with(FEE_PREFIX) {
        return ProviderKind::Fee;
    }
    let stripe = t
        .counterparty_name
        .as_deref()
        .is_some_and(|n| n.to_lowercase().contains("stripe"));
    if t.amount > Decimal::ZERO && stripe {
        return ProviderKind::Specific(KIND_STRIPE_PAYOUT);
    }
    let p = prefix(t);
    if CONVERSION_PREFIXES.contains(&p) && exchange_of(&t.details).is_some() {
        return ProviderKind::Conversion;
    }
    if p == TRANSFER_PREFIX {
        return ProviderKind::Specific(KIND_TRANSFER);
    }
    ProviderKind::Other
}

/// Clé de rattachement d'une transaction Wise : son identifiant.
fn match_key(t: &BankTransaction) -> Option<&str> {
    t.transaction_id.as_deref()
}

/// Aucune écriture de vente ne porte d'identifiant Wise (phase 1).
fn no_ids(_: &str) -> Vec<String> {
    Vec::new()
}

/// Les deux moitiés d'une conversion Wise portent le **même** identifiant
/// (une dans le relevé de chaque solde).
pub fn conversion_match(a: &BankTransaction, b: &BankTransaction) -> bool {
    let (ia, ib) = (wise_id(a).trim(), wise_id(b).trim());
    !ia.is_empty() && ia == ib
}

/// Libellé d'une écriture Wise : « Frais Wise <devise> · <identifiant> » pour
/// des frais, « Wise <compte> · <type> · <identifiant> » sinon.
pub fn entry_label(kind: EntryKind, account: &str, t: &BankTransaction) -> String {
    match kind {
        EntryKind::Fee => format!("Frais Wise {} · {}", t.currency, wise_id(t)),
        _ => {
            let mut label = format!("Wise {account} · {} · {}", prefix(t), wise_id(t));
            if let Some(name) = t.counterparty_name.as_deref().map(str::trim)
                && !name.is_empty()
            {
                label.push_str(" · ");
                label.push_str(name);
            }
            label
        }
    }
}

/// Genres admis au rattachement (en plus de l'encaissement) : le versement
/// Stripe et le virement (AC-4).
pub fn linkable(kind: ProviderKind) -> bool {
    matches!(
        kind,
        ProviderKind::Specific(KIND_STRIPE_PAYOUT | KIND_TRANSFER)
    )
}

/// Genres admis à la règle (en plus d'`Other`) : le virement (AC-5) ;
/// **jamais** le versement Stripe (FR-39-11c-4).
pub fn ruleable(kind: ProviderKind) -> bool {
    kind == ProviderKind::Specific(KIND_TRANSFER)
}

/// Le paramétrage Wise de la revue et de l'acceptation communes.
pub const SPEC: ProviderSpec = ProviderSpec {
    provider: PROVIDER,
    classify,
    match_key,
    extract_ids: no_ids,
    conversion_match,
    conversion_unpaired: (GAP_CONVERSION_HALF_MISSING, Placement::Gap),
    entry_label,
    linkable,
    ruleable,
};

// ---------------------------------------------------------------------------
// Paiement par carte converti (amendement A-1)
// ---------------------------------------------------------------------------

/// Proposition propre à Wise : un paiement par carte converti, écrit en
/// **une** écriture (amendement A-1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum CardGroupProposal {
    /// Les lignes du paiement (la principale en tête), la règle et sa
    /// contrepartie (compte de charge en CHF).
    CardGroup {
        bank_transaction_ids: Vec<i64>,
        rule_id: i64,
        counterparty_account_id: i64,
    },
}

/// Une proposition de la revue Wise, et corps de son acceptation : un genre
/// commun ([`Proposal`]) ou un paiement par carte groupé.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WiseProposal {
    /// Paiement par carte converti (`type: "card_group"`).
    CardGroup(CardGroupProposal),
    /// Les genres communs (`link_entry`, `fee`, `rule`, `conversion`…).
    Common(Proposal),
}

/// Le jugement d'un paiement par carte converti.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardGroupJudgement {
    /// Écrivable : le montant du marchand, en CHF.
    Valid {
        /// Somme des lignes en CHF et des montants CHF des segments de
        /// conversion, frais exclus.
        merchant_chf: Decimal,
    },
    /// Une ligne n'a pas la forme attendue (motif pour le détail).
    Unsupported(&'static str),
}

/// Un paiement par carte converti : toutes ses lignes en attente.
#[derive(Debug, Clone)]
pub struct CardGroup<'a> {
    /// L'identifiant du paiement (`CARD-<id>`).
    pub card_id: String,
    /// Ses lignes, principale en tête, puis par identifiant.
    pub lines: Vec<&'a BankTransaction>,
    /// Le jugement.
    pub judgement: CardGroupJudgement,
}

impl CardGroup<'_> {
    /// La ligne principale (en tête de [`CardGroup::lines`]).
    pub fn principal(&self) -> &BankTransaction {
        self.lines[0]
    }

    /// Les identifiants des lignes, principale en tête.
    pub fn ids(&self) -> Vec<i64> {
        self.lines.iter().map(|t| t.id).collect()
    }
}

/// L'identifiant du paiement par carte d'une ligne (`CARD-<id>`, frais
/// compris), ou `None`.
pub fn card_id_of(t: &BankTransaction) -> Option<&str> {
    let id = wise_id(t);
    let id = id.strip_prefix(FEE_PREFIX).unwrap_or(id);
    id.starts_with(CARD_PREFIX).then_some(id)
}

/// La ligne est-elle un **frais** de carte (`FEE-CARD-<id>`) ?
fn is_card_fee(t: &BankTransaction) -> bool {
    wise_id(t).starts_with(FEE_PREFIX)
}

/// Le paiement est-il **converti** vers le CHF : une de ses lignes de carte
/// (hors frais) porte un segment vers le CHF ?
fn converted_to_chf(lines: &[&BankTransaction]) -> bool {
    lines
        .iter()
        .any(|t| !is_card_fee(t) && exchange_of(&t.details).is_some_and(|e| e.to == BASE_CURRENCY))
}

/// Juge les lignes d'un paiement converti (voir [`CardGroupJudgement`]) :
/// toutes au débit ; chaque ligne de carte en CHF sans segment, ou dans la
/// devise de son solde avec un segment vers le CHF et son montant CHF.
pub fn judge_card_group(lines: &[&BankTransaction]) -> CardGroupJudgement {
    let mut merchant = Decimal::ZERO;
    let mut cards = 0;
    for t in lines {
        if t.amount >= Decimal::ZERO {
            return CardGroupJudgement::Unsupported("not_a_debit");
        }
        if is_card_fee(t) {
            continue;
        }
        cards += 1;
        match exchange_of(&t.details) {
            None if t.currency == BASE_CURRENCY => {
                if t.amount.normalize().scale() > 2 {
                    return CardGroupJudgement::Unsupported("merchant_scale");
                }
                merchant += t.amount.abs();
            }
            None => return CardGroupJudgement::Unsupported("exchange_missing"),
            Some(e) if e.to != BASE_CURRENCY => {
                return CardGroupJudgement::Unsupported("merchant_not_chf");
            }
            Some(e) if e.from != t.currency => {
                return CardGroupJudgement::Unsupported("exchange_currency_mismatch");
            }
            Some(Exchange {
                to_amount: Some(chf),
                ..
            }) if chf.normalize().scale() > 2 => {
                return CardGroupJudgement::Unsupported("merchant_scale");
            }
            Some(Exchange {
                to_amount: Some(chf),
                ..
            }) => merchant += chf,
            Some(_) => return CardGroupJudgement::Unsupported("exchange_amount_missing"),
        }
    }
    if cards == 0 {
        return CardGroupJudgement::Unsupported("card_line_missing");
    }
    CardGroupJudgement::Valid {
        merchant_chf: merchant,
    }
}

/// Ordonne les lignes : la principale en tête — la ligne de carte en CHF, à
/// défaut la ligne de carte de plus petit identifiant —, puis par identifiant.
fn order_lines(lines: &mut [&BankTransaction]) {
    lines.sort_by_key(|t| t.id);
    let principal = lines
        .iter()
        .position(|t| !is_card_fee(t) && t.currency == BASE_CURRENCY)
        .or_else(|| lines.iter().position(|t| !is_card_fee(t)));
    if let Some(i) = principal {
        lines[..=i].rotate_right(1);
    }
}

/// Les paiements par carte **convertis** parmi les transactions en attente
/// (amendement A-1) : par identifiant `CARD-<id>`, toutes les lignes `CARD-` et
/// `FEE-CARD-` du paiement, sur tous les soldes ; un paiement est retenu dès
/// qu'une de ses lignes de carte porte un segment vers le CHF. Triés par
/// identifiant du paiement.
pub fn card_groups(pending: &[BankTransaction]) -> Vec<CardGroup<'_>> {
    let mut by_card: BTreeMap<&str, Vec<&BankTransaction>> = BTreeMap::new();
    for t in pending {
        if let Some(card) = card_id_of(t) {
            by_card.entry(card).or_default().push(t);
        }
    }
    by_card
        .into_iter()
        .filter(|(_, lines)| converted_to_chf(lines))
        .map(|(card, mut lines)| {
            order_lines(&mut lines);
            let judgement = judge_card_group(&lines);
            CardGroup {
                card_id: card.to_string(),
                lines,
                judgement,
            }
        })
        .collect()
}

/// Les lignes de frais `FEE-x` dont aucune transaction (en attente ou
/// rapprochée) du **même** compte bancaire ne porte l'identifiant `x` (AC-6) :
/// `pending` les lignes revues, `all` toutes les transactions des comptes.
pub fn fees_without_origin<'a>(
    pending: &'a [BankTransaction],
    all: &[BankTransaction],
) -> Vec<&'a BankTransaction> {
    let known: HashSet<(i64, &str)> = all
        .iter()
        .chain(pending.iter())
        .filter_map(|t| {
            t.transaction_id
                .as_deref()
                .map(|id| (t.bank_account_id, id))
        })
        .collect();
    pending
        .iter()
        .filter(|t| {
            wise_id(t)
                .strip_prefix(FEE_PREFIX)
                .is_some_and(|origin| !known.contains(&(t.bank_account_id, origin)))
        })
        .collect()
}

/// La part de chaque solde d'un paiement groupé : par compte bancaire, la
/// somme des montants **absolus** de ses lignes (carte et frais), dans sa
/// devise.
pub fn shares_by_account(lines: &[&BankTransaction]) -> BTreeMap<i64, Decimal> {
    let mut shares: BTreeMap<i64, Decimal> = BTreeMap::new();
    for t in lines {
        *shares.entry(t.bank_account_id).or_default() += t.amount.abs();
    }
    shares
}

/// Les comptes admis comme contrepartie d'une règle **en CHF** seulement : la
/// charge d'un paiement groupé est au montant du marchand, en CHF.
pub fn chf_rule_accounts(all: &HashMap<i64, Option<String>>) -> HashMap<i64, Option<String>> {
    all.iter()
        .filter(|(_, c)| c.as_deref().is_none_or(|c| c == BASE_CURRENCY))
        .map(|(id, c)| (*id, c.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use kesh_db::entities::bank_transaction::BankTransactionStatus;
    use std::str::FromStr;

    fn tx(id: i64, account: i64, wise: &str, amount: &str, currency: &str) -> BankTransaction {
        let now = chrono::Utc::now().naive_utc();
        let prefix: String = wise
            .split(|c: char| c.is_ascii_digit())
            .next()
            .unwrap_or("")
            .trim_end_matches('-')
            .to_string();
        BankTransaction {
            id,
            company_id: 1,
            import_id: 1,
            bank_account_id: account,
            booking_date: NaiveDate::from_ymd_opt(2026, 11, 7).unwrap(),
            value_date: None,
            amount: Decimal::from_str(amount).unwrap(),
            currency: currency.into(),
            reference: None,
            details: String::new(),
            end_to_end_id: None,
            transaction_id: Some(wise.into()),
            counterparty_iban: None,
            counterparty_name: None,
            provider_type: Some(prefix),
            counterparty_email: None,
            status: BankTransactionStatus::Pending,
            matched_entry_id: None,
            auto_match_rejected_at: None,
            version: 1,
            created_at: now,
            updated_at: now,
        }
    }

    fn with_details(mut t: BankTransaction, details: &str) -> BankTransaction {
        t.details = details.into();
        t
    }

    fn named(mut t: BankTransaction, name: &str) -> BankTransaction {
        t.counterparty_name = Some(name.into());
        t
    }

    #[test]
    fn segment_de_conversion_lu_seulement_dans_sa_forme() {
        assert_eq!(
            exchange_of("Carte Boutique · CARD · USD→CHF 128.89 au cours 0.82060"),
            Some(Exchange {
                from: "USD".into(),
                to: "CHF".into(),
                to_amount: Some(Decimal::from_str("128.89").unwrap()),
                rate: Some(Decimal::from_str("0.82060").unwrap()),
            })
        );
        assert_eq!(
            exchange_of("CHF→EUR"),
            Some(Exchange {
                from: "CHF".into(),
                to: "EUR".into(),
                to_amount: None,
                rate: None,
            })
        );
        assert_eq!(
            exchange_of("EUR→USD au cours 1.09").unwrap().rate,
            Some(Decimal::from_str("1.09").unwrap())
        );
        // Une flèche dans une description n'est pas un segment.
        assert_eq!(exchange_of("Aller → retour"), None);
        assert_eq!(exchange_of("usd→CHF 1"), None);
        assert_eq!(exchange_of("USD→CHF abc"), None);
        assert_eq!(exchange_of("USD→CHF -1"), None);
        assert_eq!(exchange_of(""), None);
    }

    #[test]
    fn chaque_genre_et_sa_preseance() {
        // Frais, quel que soit le reste.
        let fee = with_details(
            tx(1, 1, "FEE-BALANCE-3", "-0.35", "CHF"),
            "Frais · CHF→EUR 1",
        );
        assert_eq!(classify(&fee), ProviderKind::Fee);
        assert_eq!(
            classify(&named(tx(1, 1, "FEE-TRANSFER-4", "0.40", "CHF"), "Stripe")),
            ProviderKind::Fee
        );
        // Versement Stripe : avant la conversion (préfixe BALANCE d'un dépôt).
        let depot = named(tx(2, 1, "BALANCE-8", "1500", "CHF"), "Stripe Fictif SA");
        assert_eq!(classify(&depot), ProviderKind::Specific(KIND_STRIPE_PAYOUT));
        let virement_stripe = named(tx(2, 1, "TRANSFER-9", "10", "EUR"), "STRIPE");
        assert_eq!(
            classify(&virement_stripe),
            ProviderKind::Specific(KIND_STRIPE_PAYOUT)
        );
        // Un débit vers Stripe n'est pas un versement.
        assert_eq!(
            classify(&named(tx(2, 1, "TRANSFER-9", "-10", "EUR"), "Stripe")),
            ProviderKind::Specific(KIND_TRANSFER)
        );
        // Conversion : préfixe et segment.
        let conv = with_details(
            tx(3, 1, "BALANCE-3", "-500", "CHF"),
            "Converti · CONVERSION · CHF→EUR 462.30 au cours 0.9246",
        );
        assert_eq!(classify(&conv), ProviderKind::Conversion);
        let ordre = with_details(
            tx(3, 1, "CONVERSION_ORDER-11", "109", "USD"),
            "EUR→USD 109.00",
        );
        assert_eq!(classify(&ordre), ProviderKind::Conversion);
        // Dépôt BALANCE sans segment, ni Stripe : autre.
        assert_eq!(
            classify(&named(tx(4, 1, "BALANCE-12", "50", "CHF"), "Client")),
            ProviderKind::Other
        );
        // Virement, carte.
        assert_eq!(
            classify(&tx(5, 1, "TRANSFER-6", "-80", "CHF")),
            ProviderKind::Specific(KIND_TRANSFER)
        );
        let carte = with_details(tx(6, 1, "CARD-7", "-19", "CHF"), "USD→CHF 1");
        assert_eq!(classify(&carte), ProviderKind::Other);
    }

    #[test]
    fn genres_admis_au_rattachement_et_a_la_regle() {
        assert!(linkable(ProviderKind::Specific(KIND_STRIPE_PAYOUT)));
        assert!(linkable(ProviderKind::Specific(KIND_TRANSFER)));
        assert!(!linkable(ProviderKind::Other));
        assert!(!linkable(ProviderKind::Fee));
        assert!(ruleable(ProviderKind::Specific(KIND_TRANSFER)));
        // Un versement Stripe n'est jamais affecté par une règle.
        assert!(!ruleable(ProviderKind::Specific(KIND_STRIPE_PAYOUT)));
        assert!(!ruleable(ProviderKind::Conversion));
    }

    #[test]
    fn conversion_appariee_par_le_meme_identifiant() {
        let a = tx(1, 1, "BALANCE-3", "-500", "CHF");
        let b = tx(2, 2, "BALANCE-3", "462.30", "EUR");
        let c = tx(3, 2, "BALANCE-4", "462.30", "EUR");
        assert!(conversion_match(&a, &b));
        assert!(!conversion_match(&a, &c));
        let mut vide = a.clone();
        vide.transaction_id = Some("  ".into());
        let mut vide2 = b.clone();
        vide2.transaction_id = Some("  ".into());
        assert!(!conversion_match(&vide, &vide2));
    }

    #[test]
    fn libelles_des_ecritures() {
        let fee = tx(1, 1, "FEE-CARD-7", "-0.44", "USD");
        assert_eq!(
            entry_label(EntryKind::Fee, "Wise Pro", &fee),
            "Frais Wise USD · FEE-CARD-7"
        );
        let carte = named(tx(2, 1, "CARD-7", "-41.11", "CHF"), "Boutique Exemple");
        assert_eq!(
            entry_label(EntryKind::Rule, "Wise Pro", &carte),
            "Wise Wise Pro · CARD · CARD-7 · Boutique Exemple"
        );
    }

    /// La partielle « USD, CHF » du relevé (montants des fixtures de 39-11c) :
    /// CHF −86.60 (solde vidé), USD −13.89 converti pour 11.35 CHF, frais USD.
    fn partielle() -> Vec<BankTransaction> {
        vec![
            named(tx(10, 1, "CARD-500", "-86.60", "CHF"), "Boutique"),
            named(
                with_details(
                    tx(11, 3, "CARD-500", "-13.85", "USD"),
                    "Carte · CARD · USD→CHF 11.35 au cours 0.81950",
                ),
                "Boutique",
            ),
            tx(12, 3, "FEE-CARD-500", "-0.04", "USD"),
            // Un autre paiement, sans conversion : jamais un groupe.
            named(tx(13, 1, "CARD-501", "-5.00", "CHF"), "Café"),
        ]
    }

    #[test]
    fn partielle_regroupee_au_montant_du_marchand() {
        let pending = partielle();
        let groups = card_groups(&pending);
        assert_eq!(groups.len(), 1);
        let g = &groups[0];
        assert_eq!(g.card_id, "CARD-500");
        // La ligne en CHF en tête, puis par identifiant.
        assert_eq!(g.ids(), [10, 11, 12]);
        assert_eq!(
            g.judgement,
            CardGroupJudgement::Valid {
                merchant_chf: Decimal::from_str("97.95").unwrap()
            }
        );
        let shares = shares_by_account(&g.lines);
        assert_eq!(shares[&1], Decimal::from_str("86.60").unwrap());
        assert_eq!(shares[&3], Decimal::from_str("13.89").unwrap());
    }

    #[test]
    fn totale_depuis_un_seul_solde_regroupee_aussi() {
        let pending = vec![
            tx(21, 2, "FEE-CARD-600", "-0.20", "EUR"),
            with_details(
                tx(20, 2, "CARD-600", "-44.66", "EUR"),
                "EUR→CHF 41.85 au cours 0.93710",
            ),
        ];
        let groups = card_groups(&pending);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].ids(), [20, 21]);
        assert_eq!(
            groups[0].judgement,
            CardGroupJudgement::Valid {
                merchant_chf: Decimal::from_str("41.85").unwrap()
            }
        );
    }

    #[test]
    fn marchand_en_devise_ou_sans_conversion_n_est_pas_un_groupe() {
        let pending = vec![
            with_details(tx(30, 2, "CARD-700", "-10.78", "EUR"), "EUR→USD 12.45"),
            tx(31, 2, "FEE-CARD-700", "-0.05", "EUR"),
            tx(32, 1, "CARD-701", "-36.85", "CHF"),
        ];
        assert!(card_groups(&pending).is_empty());
    }

    #[test]
    fn formes_refusees() {
        // Ligne positive dans le groupe.
        let mut p = partielle();
        p[0].amount = Decimal::from_str("86.60").unwrap();
        assert_eq!(
            card_groups(&p)[0].judgement,
            CardGroupJudgement::Unsupported("not_a_debit")
        );
        // Ligne en devise sans segment.
        let mut p = partielle();
        p.push(tx(14, 2, "CARD-500", "-1.00", "EUR"));
        assert_eq!(
            card_groups(&p)[0].judgement,
            CardGroupJudgement::Unsupported("exchange_missing")
        );
        // Segment d'une autre devise que celle du solde.
        let mut p = partielle();
        p[1].details = "EUR→CHF 11.35".into();
        assert_eq!(
            card_groups(&p)[0].judgement,
            CardGroupJudgement::Unsupported("exchange_currency_mismatch")
        );
        // Segment sans montant CHF.
        let mut p = partielle();
        p[1].details = "USD→CHF au cours 0.8".into();
        assert_eq!(
            card_groups(&p)[0].judgement,
            CardGroupJudgement::Unsupported("exchange_amount_missing")
        );
        // Un segment vers le CHF et un autre vers l'EUR.
        let mut p = partielle();
        p.push(with_details(
            tx(15, 2, "CARD-500", "-1.00", "EUR"),
            "EUR→USD 1.10",
        ));
        assert_eq!(
            card_groups(&p)[0].judgement,
            CardGroupJudgement::Unsupported("merchant_not_chf")
        );
    }

    /// Revue P2, L-b : un montant CHF au centime fractionnaire est refusé
    /// (`merchant_scale`), sur la ligne en CHF comme dans le segment ; des
    /// zéros de queue (une colonne relue à quatre décimales) ne le sont pas.
    #[test]
    fn montant_du_marchand_au_centime() {
        let admis = |p: &[BankTransaction]| card_groups(p)[0].judgement.clone();
        let mut p = partielle();
        p[0].amount = Decimal::from_str("-86.6000").unwrap();
        p[1].details = "Carte · CARD · USD→CHF 11.350 au cours 0.81950".into();
        assert_eq!(
            admis(&p),
            CardGroupJudgement::Valid {
                merchant_chf: Decimal::from_str("97.95").unwrap()
            }
        );
        let mut p = partielle();
        p[0].amount = Decimal::from_str("-86.605").unwrap();
        assert_eq!(admis(&p), CardGroupJudgement::Unsupported("merchant_scale"));
        let mut p = partielle();
        p[1].details = "Carte · CARD · USD→CHF 11.355 au cours 0.81950".into();
        assert_eq!(admis(&p), CardGroupJudgement::Unsupported("merchant_scale"));
    }

    /// Revue P2, L-c : c'est le **dernier** élément conforme qui compte — une
    /// description conforme, placée avant le segment de l'import, ne l'emporte
    /// pas ; une note conforme, placée après, l'emporte (limite écrite à
    /// `exchange_of`).
    #[test]
    fn segment_le_dernier_element_conforme() {
        let lu = exchange_of("EUR→CHF 1 · CARD · USD→CHF 11.35 au cours 0.81950").unwrap();
        assert_eq!(
            (lu.from.as_str(), lu.to_amount),
            ("USD", Some(Decimal::from_str("11.35").unwrap()))
        );
        let note = exchange_of("Carte · CARD · USD→CHF 11.35 · EUR→CHF 2").unwrap();
        assert_eq!(
            (note.from.as_str(), note.to_amount),
            ("EUR", Some(Decimal::from_str("2").unwrap()))
        );
        // Un élément non conforme après le segment ne le masque pas.
        let apres = exchange_of("Carte · USD→CHF 11.35 · Aller → retour").unwrap();
        assert_eq!(apres.from, "USD");
    }

    #[test]
    fn principale_sans_ligne_en_chf() {
        let pending = vec![
            with_details(tx(41, 3, "CARD-800", "-5", "USD"), "USD→CHF 4"),
            with_details(tx(40, 2, "CARD-800", "-5", "EUR"), "EUR→CHF 4.6"),
            tx(39, 2, "FEE-CARD-800", "-0.02", "EUR"),
        ];
        let g = &card_groups(&pending)[0];
        assert_eq!(g.ids(), [40, 39, 41]);
        assert_eq!(g.principal().id, 40);
    }

    #[test]
    fn frais_sans_origine_sur_le_meme_compte() {
        let pending = vec![
            tx(1, 1, "FEE-TRANSFER-4", "-1.20", "CHF"),
            tx(2, 1, "FEE-BALANCE-3", "-0.35", "CHF"),
            tx(3, 2, "FEE-CARD-9", "-0.10", "EUR"),
        ];
        let all = vec![
            tx(4, 1, "TRANSFER-4", "-300", "CHF"),
            // L'origine de FEE-CARD-9 est sur un autre compte : sans origine.
            tx(5, 1, "CARD-9", "-1", "CHF"),
        ];
        let sans: Vec<i64> = fees_without_origin(&pending, &all)
            .iter()
            .map(|t| t.id)
            .collect();
        assert_eq!(sans, [2, 3]);
    }

    #[test]
    fn contreparties_en_chf_seulement() {
        let all: HashMap<i64, Option<String>> = [
            (1, None),
            (2, Some("CHF".to_string())),
            (3, Some("EUR".to_string())),
        ]
        .into();
        let chf = chf_rule_accounts(&all);
        let mut ids: Vec<i64> = chf.keys().copied().collect();
        ids.sort();
        assert_eq!(ids, [1, 2]);
    }

    #[test]
    fn proposition_de_groupe_et_commune_lues_et_ecrites() {
        let groupe: WiseProposal = serde_json::from_value(serde_json::json!({
            "type": "card_group", "bankTransactionIds": [10, 11], "ruleId": 3,
            "counterpartyAccountId": 4
        }))
        .unwrap();
        assert_eq!(
            groupe,
            WiseProposal::CardGroup(CardGroupProposal::CardGroup {
                bank_transaction_ids: vec![10, 11],
                rule_id: 3,
                counterparty_account_id: 4,
            })
        );
        let frais: WiseProposal = serde_json::from_value(serde_json::json!({
            "type": "fee", "bankTransactionId": 1, "feeAccountId": 2
        }))
        .unwrap();
        assert_eq!(
            frais,
            WiseProposal::Common(Proposal::Fee {
                bank_transaction_id: 1,
                fee_account_id: 2
            })
        );
        assert_eq!(serde_json::to_value(&groupe).unwrap()["type"], "card_group");
        assert!(
            serde_json::from_value::<WiseProposal>(serde_json::json!({ "type": "inconnu" }))
                .is_err()
        );
    }
}
