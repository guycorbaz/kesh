//! Analyse du XML des cours de l'AFC / OFDF (Story 37-2, AC-2) — pure, sans I/O.
//!
//! Deux documents, une seule analyse des `<devise>` :
//! - cours **mensuels** (`xmlavgmonth`) : racine `monatsmittelkurs`, date
//!   `<monat>AAAA-MM</monat>` ;
//! - cours **journaliers** (`xmldaily`) : racine `wechselkurse`, date
//!   `<datum>JJ.MM.AAAA</datum>`.
//!
//! Les éléments se lisent par **nom local** (la racine porte un espace de noms
//! par défaut, différent pour chacun) ; les éléments inconnus (`land_*`,
//! `zeit`, `gueltigkeit`) sont ignorés.
//!
//! **Tout ce qui vient de la source est hostile (R5).** Avant toute écriture :
//! corps borné à [`MAX_BODY_BYTES`] et UTF-8 valide ; racine attendue ;
//! profondeur ≤ 3 comptée par la boucle (aucune récursion) ; au plus
//! [`MAX_DEVISES`] `<devise>` ; une seule date ; `DOCTYPE` refusé ; toute
//! référence d'entité qui n'est ni une référence de caractère ni l'une des cinq
//! entités prédéfinies de XML refusée (quick-xml ne développe aucune entité
//! déclarée — le refus le rend vérifiable) ; `PI` et `CDATA` dans un élément lu
//! refusés ; textes lus et attribut `code` bornés à [`MAX_TEXT_BYTES`] octets ;
//! grammaires des nombres vérifiées **avant** toute conversion. Une devise
//! fautive est **écartée** ([`RejectedRate`]) sans bloquer les autres ; le
//! document entier n'est refusé que s'il est mal formé, sans date lisible, ou
//! sans aucune devise valide.

use std::str::FromStr;

use chrono::NaiveDate;
use quick_xml::Reader;
use quick_xml::XmlVersion;
use quick_xml::events::{BytesStart, Event};
use rust_decimal::{Decimal, RoundingStrategy};
use thiserror::Error;

use super::{RATE_MAX_SCALE, RateBoundsError, check_rate_bounds};
use crate::currency::CurrencyCode;

/// Taille maximale du corps d'une réponse de la source : 1 MiB (14 et 17 Ko
/// relevés). Le client HTTP la contrôle avant l'analyse ; l'analyse la
/// recontrôle.
pub const MAX_BODY_BYTES: usize = 1024 * 1024;

/// Nombre maximal d'éléments `<devise>` (73 et 74 relevés).
pub const MAX_DEVISES: usize = 500;

/// Taille maximale, en octets, du texte d'un élément lu (`monat`, `datum`,
/// `waehrung`, `kurs`) et de l'attribut `code`.
pub const MAX_TEXT_BYTES: usize = 64;

/// Profondeur maximale : racine → `devise` → enfant.
const MAX_DEPTH: usize = 3;

/// Plus grand cours publié admis (exclu) : 10⁹, borne de prudence plus stricte
/// que les 13 chiffres entiers de `DECIMAL(19,6)`.
const PUBLISHED_RATE_LIMIT: Decimal = Decimal::from_parts(1_000_000_000, 0, 0, false, 0);

/// Plus grande unité admise.
const MAX_UNIT: u32 = 1_000_000;

/// Un cours publié valide, pour une devise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedRate {
    /// Code ISO 4217 (majuscules, jamais `CHF`) : monnaie active ou retirée
    /// ([`crate::currency::WITHDRAWN_CURRENCIES`]).
    pub currency: CurrencyCode,
    /// Unité publiée en tête de `<waehrung>` (1, 100, 1000, 10000…).
    pub published_unit: i32,
    /// Cours publié pour l'unité (au plus 6 décimales).
    pub published_rate: Decimal,
    /// `published_rate / published_unit`, arrondi à 10 décimales
    /// (`MidpointAwayFromZero`) — la valeur stockée, celle que lisent les
    /// contrôles.
    pub rate_per_unit: Decimal,
}

/// Pourquoi une devise a été écartée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectReason {
    /// Attribut `code`, `<waehrung>` ou `<kurs>` absent.
    Missing,
    /// Texte ou attribut au-delà de [`MAX_TEXT_BYTES`] octets.
    TooLong,
    /// Code hors des trois lettres ASCII, absent des deux listes embarquées
    /// (monnaies actives et monnaies retirées,
    /// [`crate::currency::is_rate_currency`]), ou `CHF`.
    CodeInvalid,
    /// `<waehrung>` hors de la grammaire `^[1-9][0-9]{0,6} [A-Za-z]{3}$`.
    UnitInvalid,
    /// Le code de `<waehrung>` diffère de l'attribut `code`.
    CodeMismatch,
    /// Unité au-delà de 1 000 000.
    UnitTooLarge,
    /// `<kurs>` hors de la grammaire `^[0-9]{1,13}(\.[0-9]{1,6})?$`.
    RateInvalid,
    /// Cours publié nul.
    RateNotPositive,
    /// Cours publié ≥ 10⁹.
    RateTooLarge,
    /// Cours par unité hors de [`check_rate_bounds`] (arrondi à 0, ou > 10⁶).
    RatePerUnit(RateBoundsError),
    /// La devise apparaît plusieurs fois : toutes ses occurrences sont écartées.
    Duplicate,
}

impl RejectReason {
    /// Code court, pour le journal du serveur.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::TooLong => "too_long",
            Self::CodeInvalid => "code_invalid",
            Self::UnitInvalid => "unit_invalid",
            Self::CodeMismatch => "code_mismatch",
            Self::UnitTooLarge => "unit_too_large",
            Self::RateInvalid => "rate_invalid",
            Self::RateNotPositive => "rate_not_positive",
            Self::RateTooLarge => "rate_too_large",
            Self::RatePerUnit(_) => "rate_per_unit_out_of_bounds",
            Self::Duplicate => "duplicate",
        }
    }
}

/// Une devise écartée : son attribut `code` tel que lu (tronqué à
/// [`MAX_TEXT_BYTES`] octets, vide s'il manque) et le motif.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectedRate {
    /// L'attribut `code` lu, pour le journal.
    pub code: String,
    /// Le motif.
    pub reason: RejectReason,
}

/// Cours mensuels d'un mois.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonthlyRates {
    /// Le mois lu (`<monat>`), au 1er.
    pub month: NaiveDate,
    /// Les devises valides, dans l'ordre du document.
    pub rates: Vec<PublishedRate>,
    /// Les devises écartées.
    pub rejected: Vec<RejectedRate>,
}

/// Cours journaliers d'une publication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DailyRates {
    /// La date de publication lue (`<datum>`).
    pub date: NaiveDate,
    /// Les devises valides, dans l'ordre du document.
    pub rates: Vec<PublishedRate>,
    /// Les devises écartées.
    pub rejected: Vec<RejectedRate>,
}

/// Refus d'un document entier (ou contrôle de l'appelant). Chaque variante a
/// son code classé ([`RatesError::code`]) — jamais un texte libre en base.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RatesError {
    /// Racine sans enfant : pas encore publié.
    #[error("cours pas encore publiés")]
    NotPublished,
    /// Document refusé (XML mal formé, structure, entité, date illisible…).
    /// Le détail ne sert qu'au journal du serveur.
    #[error("réponse de la source mal formée : {0}")]
    Malformed(&'static str),
    /// Aucune devise valide.
    #[error("aucune devise valide dans la réponse de la source")]
    NoValidRate {
        /// Les devises écartées.
        rejected: Vec<RejectedRate>,
    },
    /// Le mois lu diffère du mois demandé ([`check_monthly_month`]).
    #[error("mois rendu {got} différent du mois demandé {expected}")]
    MonthMismatch {
        /// Mois demandé (1er).
        expected: NaiveDate,
        /// Mois lu (1er).
        got: NaiveDate,
    },
    /// La date de publication lue est postérieure à la date demandée, ou la
    /// précède de plus de 7 jours ([`check_daily_date`]).
    #[error("publication du {got} incompatible avec la date demandée {requested}")]
    DateMismatch {
        /// Date demandée.
        requested: NaiveDate,
        /// Date de publication lue.
        got: NaiveDate,
    },
}

impl RatesError {
    /// Code classé (`last_error` de `_kesh_exchange_rate_fetch`) :
    /// `not_published`, `malformed`, `no_valid_rate`, `month_mismatch`,
    /// `date_mismatch`.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotPublished => "not_published",
            Self::Malformed(_) => "malformed",
            Self::NoValidRate { .. } => "no_valid_rate",
            Self::MonthMismatch { .. } => "month_mismatch",
            Self::DateMismatch { .. } => "date_mismatch",
        }
    }
}

/// Analyse une réponse de `xmlavgmonth` : le mois lu, les devises valides et
/// les devises écartées. Le contrôle « mois demandé » est
/// [`check_monthly_month`], à la charge de l'appelant.
pub fn parse_monthly_rates(xml: &[u8]) -> Result<MonthlyRates, RatesError> {
    let doc = analyser(xml, "monatsmittelkurs", "monat")?;
    let month = lire_mois(&doc.date)?;
    let (rates, rejected) = trier_devises(doc.devises)?;
    Ok(MonthlyRates {
        month,
        rates,
        rejected,
    })
}

/// Analyse une réponse de `xmldaily` : la date de publication lue, les devises
/// valides et les devises écartées. Le contrôle « date demandée » est
/// [`check_daily_date`], à la charge de l'appelant.
pub fn parse_daily_rates(xml: &[u8]) -> Result<DailyRates, RatesError> {
    let doc = analyser(xml, "wechselkurse", "datum")?;
    let date = lire_datum(&doc.date)?;
    let (rates, rejected) = trier_devises(doc.devises)?;
    Ok(DailyRates {
        date,
        rates,
        rejected,
    })
}

/// Le mois lu doit égaler le mois demandé (`expected`, ramené au 1er) ; sinon
/// [`RatesError::MonthMismatch`] (code `month_mismatch`, rien n'est écrit).
pub fn check_monthly_month(rates: &MonthlyRates, expected: NaiveDate) -> Result<(), RatesError> {
    let expected = super::first_of_month(expected);
    if rates.month == expected {
        Ok(())
    } else {
        Err(RatesError::MonthMismatch {
            expected,
            got: rates.month,
        })
    }
}

/// La publication lue doit être datée au plus tard du jour demandé, et au plus
/// 7 jours avant (un jour sans publication rend la dernière publication
/// antérieure) ; sinon [`RatesError::DateMismatch`] (code `date_mismatch`).
pub fn check_daily_date(rates: &DailyRates, requested: NaiveDate) -> Result<(), RatesError> {
    let ecart = (requested - rates.date).num_days();
    if (0..=7).contains(&ecart) {
        Ok(())
    } else {
        Err(RatesError::DateMismatch {
            requested,
            got: rates.date,
        })
    }
}

// ---------------------------------------------------------------------------
// Analyse commune
// ---------------------------------------------------------------------------

/// Un texte lu, borné : au-delà de [`MAX_TEXT_BYTES`], il n'est plus allongé
/// et se marque trop long.
#[derive(Debug, Default)]
struct TexteBorne {
    valeur: String,
    trop_long: bool,
}

impl TexteBorne {
    fn pousser(&mut self, s: &str) {
        if self.trop_long {
            return;
        }
        if self.valeur.len() + s.len() > MAX_TEXT_BYTES {
            self.trop_long = true;
            self.valeur.clear();
        } else {
            self.valeur.push_str(s);
        }
    }
}

/// Une `<devise>` telle que lue, avant validation.
#[derive(Debug, Default)]
struct DeviseBrute {
    code: Option<TexteBorne>,
    waehrung: Option<TexteBorne>,
    kurs: Option<TexteBorne>,
    /// Un enfant lu (`waehrung`, `kurs`) répété.
    enfant_double: bool,
}

/// Ce que l'élément courant capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Capture {
    Aucune,
    Date,
    Waehrung,
    Kurs,
}

/// Le document analysé : le texte de la date et les devises brutes.
struct Document {
    date: TexteBorne,
    devises: Vec<DeviseBrute>,
}

/// Analyse commune aux deux documents : racine `racine`, date `element_date`.
/// Itérative : la profondeur est un compteur, jamais une pile d'appels.
fn analyser(xml: &[u8], racine: &str, element_date: &str) -> Result<Document, RatesError> {
    if xml.len() > MAX_BODY_BYTES {
        return Err(RatesError::Malformed("corps trop volumineux"));
    }
    let texte = std::str::from_utf8(xml).map_err(|_| RatesError::Malformed("UTF-8 invalide"))?;
    let mut reader = Reader::from_str(texte);
    let cfg = reader.config_mut();
    cfg.trim_text(false);
    cfg.check_end_names = true;
    cfg.expand_empty_elements = false;

    let mut profondeur: usize = 0;
    let mut racine_vue = false;
    let mut racine_fermee = false;
    let mut racine_a_un_enfant = false;
    let mut date: Option<TexteBorne> = None;
    let mut devises: Vec<DeviseBrute> = Vec::new();
    let mut devise: Option<DeviseBrute> = None;
    let mut capture = Capture::Aucune;
    let mut texte_capture = TexteBorne::default();

    loop {
        let event = reader
            .read_event()
            .map_err(|_| RatesError::Malformed("XML mal formé"))?;
        // `Start` et `Empty` ouvrent un élément ; `Empty` le referme aussitôt.
        let (ouverture, vide) = match &event {
            Event::Start(e) => (Some(e), false),
            Event::Empty(e) => (Some(e), true),
            _ => (None, false),
        };
        if let Some(e) = ouverture {
            if capture != Capture::Aucune {
                return Err(RatesError::Malformed("élément dans un texte lu"));
            }
            profondeur += 1;
            if profondeur > MAX_DEPTH {
                return Err(RatesError::Malformed("imbrication trop profonde"));
            }
            let nom = e.local_name();
            let nom = nom.as_ref();
            match profondeur {
                1 => {
                    if racine_vue || racine_fermee {
                        return Err(RatesError::Malformed("plusieurs racines"));
                    }
                    if nom != racine {
                        return Err(RatesError::Malformed("racine inattendue"));
                    }
                    racine_vue = true;
                }
                2 => {
                    racine_a_un_enfant = true;
                    if nom == element_date {
                        if date.is_some() {
                            return Err(RatesError::Malformed("date répétée"));
                        }
                        date = Some(TexteBorne::default());
                        capture = Capture::Date;
                    } else if nom == "devise" {
                        if devises.len() >= MAX_DEVISES {
                            return Err(RatesError::Malformed("trop de devises"));
                        }
                        devise = Some(DeviseBrute {
                            code: lire_attribut_code(e)?,
                            ..DeviseBrute::default()
                        });
                    }
                }
                _ => {
                    if let Some(d) = devise.as_mut() {
                        let deja = match nom {
                            "waehrung" => {
                                capture = Capture::Waehrung;
                                d.waehrung.is_some()
                            }
                            "kurs" => {
                                capture = Capture::Kurs;
                                d.kurs.is_some()
                            }
                            _ => false,
                        };
                        d.enfant_double |= deja;
                    }
                }
            }
            texte_capture = TexteBorne::default();
            if vide {
                fermer(
                    &mut profondeur,
                    &mut capture,
                    &mut texte_capture,
                    &mut date,
                    &mut devise,
                    &mut devises,
                    &mut racine_fermee,
                );
            }
            continue;
        }
        match event {
            Event::End(_) => fermer(
                &mut profondeur,
                &mut capture,
                &mut texte_capture,
                &mut date,
                &mut devise,
                &mut devises,
                &mut racine_fermee,
            ),
            Event::Text(t) => {
                if capture != Capture::Aucune {
                    let brut = t.into_inner();
                    let clair = quick_xml::escape::unescape(&brut)
                        .map_err(|_| RatesError::Malformed("texte mal échappé"))?;
                    texte_capture.pousser(&clair);
                }
            }
            Event::GeneralRef(r) => {
                // Seules les références de caractère et les cinq entités
                // prédéfinies sont admises, où qu'elles soient.
                let resolue = match r
                    .resolve_char_ref()
                    .map_err(|_| RatesError::Malformed("référence de caractère invalide"))?
                {
                    Some(c) => c.to_string(),
                    None => quick_xml::escape::resolve_xml_entity(&r)
                        .ok_or(RatesError::Malformed("référence d'entité inconnue"))?
                        .to_string(),
                };
                if capture != Capture::Aucune {
                    texte_capture.pousser(&resolue);
                }
            }
            Event::DocType(_) => return Err(RatesError::Malformed("DOCTYPE refusé")),
            Event::CData(_) if capture != Capture::Aucune => {
                return Err(RatesError::Malformed("CDATA dans un élément lu"));
            }
            Event::PI(_) if capture != Capture::Aucune => {
                return Err(RatesError::Malformed(
                    "instruction de traitement dans un élément lu",
                ));
            }
            Event::Eof => {
                if profondeur != 0 {
                    return Err(RatesError::Malformed("document tronqué"));
                }
                if !racine_vue {
                    return Err(RatesError::Malformed("racine absente"));
                }
                break;
            }
            _ => {}
        }
    }

    if !racine_a_un_enfant {
        return Err(RatesError::NotPublished);
    }
    let date = date.ok_or(RatesError::Malformed("date absente"))?;
    Ok(Document { date, devises })
}

/// Referme l'élément courant : range le texte capturé, et une `<devise>`
/// complète à la fin de son élément.
#[allow(clippy::too_many_arguments)]
fn fermer(
    profondeur: &mut usize,
    capture: &mut Capture,
    texte: &mut TexteBorne,
    date: &mut Option<TexteBorne>,
    devise: &mut Option<DeviseBrute>,
    devises: &mut Vec<DeviseBrute>,
    racine_fermee: &mut bool,
) {
    let lu = std::mem::take(texte);
    match *capture {
        Capture::Date => *date = Some(lu),
        Capture::Waehrung => {
            if let Some(d) = devise.as_mut() {
                d.waehrung = Some(lu);
            }
        }
        Capture::Kurs => {
            if let Some(d) = devise.as_mut() {
                d.kurs = Some(lu);
            }
        }
        Capture::Aucune => {
            if *profondeur == 2
                && let Some(d) = devise.take()
            {
                devises.push(d);
            }
        }
    }
    *capture = Capture::Aucune;
    if *profondeur == 1 {
        *racine_fermee = true;
    }
    *profondeur = profondeur.saturating_sub(1);
}

/// L'attribut `code` d'une `<devise>`, borné. Une entité inconnue dans sa valeur
/// refuse le document, comme dans un texte.
fn lire_attribut_code(e: &BytesStart<'_>) -> Result<Option<TexteBorne>, RatesError> {
    for attr in e.attributes() {
        let attr = attr.map_err(|_| RatesError::Malformed("attribut mal formé"))?;
        if attr.key.local_name().as_ref() == "code" {
            let valeur = attr
                .normalized_value(XmlVersion::Implicit1_0)
                .map_err(|_| RatesError::Malformed("attribut mal échappé"))?;
            let mut t = TexteBorne::default();
            t.pousser(&valeur);
            return Ok(Some(t));
        }
    }
    Ok(None)
}

/// Valide chaque devise brute, puis écarte toutes les occurrences d'une devise
/// répétée. Aucune devise valide → [`RatesError::NoValidRate`].
fn trier_devises(
    brutes: Vec<DeviseBrute>,
) -> Result<(Vec<PublishedRate>, Vec<RejectedRate>), RatesError> {
    let lues: Vec<(
        String,
        Option<CurrencyCode>,
        Result<PublishedRate, RejectReason>,
    )> = brutes
        .into_iter()
        .map(|b| {
            let code_lu = b
                .code
                .as_ref()
                .map(|t| t.valeur.clone())
                .unwrap_or_default();
            let code = b.code.as_ref().and_then(lire_code);
            let res = valider_devise(&b, code.as_ref());
            (code_lu, code, res)
        })
        .collect();
    let mut rates = Vec::new();
    let mut rejected = Vec::new();
    for (code_lu, code, res) in &lues {
        let doublon = code.as_ref().is_some_and(|c| {
            lues.iter()
                .filter(|(_, autre, _)| autre.as_ref() == Some(c))
                .count()
                > 1
        });
        match (doublon, res) {
            (true, _) => rejected.push(RejectedRate {
                code: code_lu.clone(),
                reason: RejectReason::Duplicate,
            }),
            (false, Ok(r)) => rates.push(r.clone()),
            (false, Err(reason)) => rejected.push(RejectedRate {
                code: code_lu.clone(),
                reason: *reason,
            }),
        }
    }
    if rates.is_empty() {
        return Err(RatesError::NoValidRate { rejected });
    }
    Ok((rates, rejected))
}

/// Le code d'une devise : trois lettres ASCII (blancs ASCII de bord retirés),
/// monnaie active **ou retirée** ([`CurrencyCode::parse_rate_currency`] :
/// l'AFC publie le lev bulgare, la kuna… pour les périodes où ils avaient
/// cours), différent de `CHF`.
fn lire_code(t: &TexteBorne) -> Option<CurrencyCode> {
    if t.trop_long {
        return None;
    }
    let brut = t.valeur.trim_matches(|c: char| c.is_ascii_whitespace());
    if brut.len() != 3 || !brut.bytes().all(|b| b.is_ascii_alphabetic()) {
        return None;
    }
    CurrencyCode::parse_rate_currency(brut)
        .ok()
        .filter(|c| !c.is_base())
}

/// Le texte d'un enfant lu, présent et borné, blancs ASCII de bord retirés.
fn texte_lu(t: &Option<TexteBorne>) -> Result<&str, RejectReason> {
    let t = t.as_ref().ok_or(RejectReason::Missing)?;
    if t.trop_long {
        return Err(RejectReason::TooLong);
    }
    Ok(t.valeur.trim_matches(|c: char| c.is_ascii_whitespace()))
}

/// Valide une devise : code, unité (`<waehrung>`), cours (`<kurs>`), cours par
/// unité.
fn valider_devise(
    b: &DeviseBrute,
    code: Option<&CurrencyCode>,
) -> Result<PublishedRate, RejectReason> {
    let attr = b.code.as_ref().ok_or(RejectReason::Missing)?;
    if attr.trop_long {
        return Err(RejectReason::TooLong);
    }
    let code = code.ok_or(RejectReason::CodeInvalid)?;
    if b.enfant_double {
        return Err(RejectReason::Duplicate);
    }
    let (unite, code_unite) = lire_waehrung(texte_lu(&b.waehrung)?)?;
    if !code_unite.eq_ignore_ascii_case(code.as_str()) {
        return Err(RejectReason::CodeMismatch);
    }
    if unite > MAX_UNIT {
        return Err(RejectReason::UnitTooLarge);
    }
    let cours = lire_kurs(texte_lu(&b.kurs)?)?;
    if cours <= Decimal::ZERO {
        return Err(RejectReason::RateNotPositive);
    }
    if cours >= PUBLISHED_RATE_LIMIT {
        return Err(RejectReason::RateTooLarge);
    }
    let par_unite = (cours / Decimal::from(unite))
        .round_dp_with_strategy(RATE_MAX_SCALE, RoundingStrategy::MidpointAwayFromZero);
    check_rate_bounds(&par_unite).map_err(RejectReason::RatePerUnit)?;
    Ok(PublishedRate {
        currency: code.clone(),
        published_unit: i32::try_from(unite).map_err(|_| RejectReason::UnitTooLarge)?,
        published_rate: cours,
        rate_per_unit: par_unite,
    })
}

/// `<waehrung>` : `^[1-9][0-9]{0,6} [A-Za-z]{3}$` — l'unité et le code.
fn lire_waehrung(s: &str) -> Result<(u32, &str), RejectReason> {
    let (unite, code) = s.split_once(' ').ok_or(RejectReason::UnitInvalid)?;
    let b = unite.as_bytes();
    let unite_ok = !b.is_empty()
        && b.len() <= 7
        && (b'1'..=b'9').contains(&b[0])
        && b.iter().all(u8::is_ascii_digit);
    let code_ok = code.len() == 3 && code.bytes().all(|c| c.is_ascii_alphabetic());
    if !unite_ok || !code_ok {
        return Err(RejectReason::UnitInvalid);
    }
    let unite = unite
        .parse::<u32>()
        .map_err(|_| RejectReason::UnitInvalid)?;
    Ok((unite, code))
}

/// `<kurs>` : `^[0-9]{1,13}(\.[0-9]{1,6})?$`, contrôlé avant la conversion —
/// ni signe, ni exposant, ni séparateur, ni chiffre non ASCII.
fn lire_kurs(s: &str) -> Result<Decimal, RejectReason> {
    let (entier, decimales) = match s.split_once('.') {
        Some((e, d)) => (e, Some(d)),
        None => (s, None),
    };
    let chiffres = |p: &str, max: usize| {
        !p.is_empty() && p.len() <= max && p.bytes().all(|c| c.is_ascii_digit())
    };
    if !chiffres(entier, 13) || !decimales.is_none_or(|d| chiffres(d, 6)) {
        return Err(RejectReason::RateInvalid);
    }
    Decimal::from_str(s).map_err(|_| RejectReason::RateInvalid)
}

/// `<monat>` : `AAAA-MM`, mois 01–12, année 2000–2099 ; sinon le document est
/// refusé.
fn lire_mois(t: &TexteBorne) -> Result<NaiveDate, RatesError> {
    const ILLISIBLE: RatesError = RatesError::Malformed("mois illisible");
    if t.trop_long {
        return Err(ILLISIBLE);
    }
    let s = t.valeur.trim_matches(|c: char| c.is_ascii_whitespace());
    let b = s.as_bytes();
    // ASCII d'abord : les tranches ci-dessous se font à l'octet.
    if !s.is_ascii() || b.len() != 7 || b[4] != b'-' {
        return Err(ILLISIBLE);
    }
    let annee = nombre(&s[0..4]).ok_or(ILLISIBLE)?;
    let mois = nombre(&s[5..7]).ok_or(ILLISIBLE)?;
    if !(2000..=2099).contains(&annee) {
        return Err(ILLISIBLE);
    }
    NaiveDate::from_ymd_opt(annee as i32, mois, 1).ok_or(ILLISIBLE)
}

/// `<datum>` : `JJ.MM.AAAA`, date calendaire valide, année 1999–2099 (la source
/// rend le 31.12.1999 pour le 01.01.2000) ; sinon le document est refusé.
fn lire_datum(t: &TexteBorne) -> Result<NaiveDate, RatesError> {
    const ILLISIBLE: RatesError = RatesError::Malformed("date illisible");
    if t.trop_long {
        return Err(ILLISIBLE);
    }
    let s = t.valeur.trim_matches(|c: char| c.is_ascii_whitespace());
    let b = s.as_bytes();
    if !s.is_ascii() || b.len() != 10 || b[2] != b'.' || b[5] != b'.' {
        return Err(ILLISIBLE);
    }
    let jour = nombre(&s[0..2]).ok_or(ILLISIBLE)?;
    let mois = nombre(&s[3..5]).ok_or(ILLISIBLE)?;
    let annee = nombre(&s[6..10]).ok_or(ILLISIBLE)?;
    if !(1999..=2099).contains(&annee) {
        return Err(ILLISIBLE);
    }
    NaiveDate::from_ymd_opt(annee as i32, mois, jour).ok_or(ILLISIBLE)
}

/// Un nombre de chiffres ASCII seulement (aucun signe).
fn nombre(s: &str) -> Option<u32> {
    if s.is_empty() || !s.bytes().all(|c| c.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

#[cfg(test)]
mod tests;
