//! Tests du relevé saisi à la main (Story v016-2a, AC-4, AC-5).

use super::*;
use rust_decimal_macros::dec;

/// Une altération du corps reçu, pour les tables de cas.
type RawMutation = Box<dyn Fn(&mut ManualStatementRaw)>;
/// Une altération de l'entrée analysée, pour les tables de cas.
type InputMutation = Box<dyn Fn(&mut ManualStatementInput)>;

fn d(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
}

fn line(date: &str, amount: Decimal, direction: ManualDirection) -> ManualStatementLine {
    ManualStatementLine {
        booking_date: d(date),
        value_date: None,
        details: "Ligne".into(),
        counterparty_name: None,
        reference: None,
        amount,
        direction,
    }
}

/// Relevé équilibré : 1000 + 50 − 320.25 = 729.75.
fn input() -> ManualStatementInput {
    ManualStatementInput {
        bank_account_id: 3,
        period_from: d("2026-09-01"),
        period_to: d("2026-09-30"),
        statement_number: Some("9/2026".into()),
        opening_balance: dec!(1000.00),
        closing_balance: dec!(729.75),
        lines: vec![
            line("2026-09-03", dec!(320.25), ManualDirection::Debit),
            line("2026-09-15", dec!(50.00), ManualDirection::Credit),
        ],
    }
}

fn raw_line(date: &str, amount: &str, direction: &str) -> ManualLineRaw {
    ManualLineRaw {
        booking_date: Some(date.into()),
        value_date: None,
        details: Some("Ligne".into()),
        counterparty_name: None,
        reference: None,
        amount: Some(amount.into()),
        direction: Some(direction.into()),
    }
}

fn raw() -> ManualStatementRaw {
    ManualStatementRaw {
        bank_account_id: 3,
        period_from: Some("2026-09-01".into()),
        period_to: Some("2026-09-30".into()),
        statement_number: Some("9/2026".into()),
        opening_balance: Some("1000.00".into()),
        closing_balance: Some("729.75".into()),
        lines: Some(vec![
            raw_line("2026-09-03", "320.25", "debit"),
            raw_line("2026-09-15", "50.00", "credit"),
        ]),
        ..Default::default()
    }
}

fn invalid(
    line: Option<usize>,
    field: ManualStatementField,
    reason: ManualStatementReason,
) -> ManualStatementError {
    ManualStatementError::Invalid {
        line,
        field,
        reason,
    }
}

// ---------------------------------------------------------------------------
// Ensembles fermés (AC-4)
// ---------------------------------------------------------------------------

/// Épinglage sur les chaînes **littérales** de la fiche parente (§ *Ensembles
/// fermés*), dans les deux sens.
#[test]
fn reason_codes_are_pinned_to_the_closed_set() {
    let expected = [
        "required",
        "invalid_date",
        "invalid_number",
        "period_inverted",
        "too_many_lines",
        "out_of_period",
        "too_long",
        "not_positive",
        "too_many_decimals",
        "too_many_digits",
        "unknown_direction",
    ];
    let got: std::collections::BTreeSet<&str> = ManualStatementReason::ALL
        .iter()
        .map(|r| r.code())
        .collect();
    let want: std::collections::BTreeSet<&str> = expected.into_iter().collect();
    assert_eq!(got, want, "ensemble des `reason` ≠ celui de la fiche");
    assert_eq!(ManualStatementReason::ALL.len(), expected.len());
}

#[test]
fn field_codes_are_pinned_to_the_closed_set() {
    let expected = [
        "periodFrom",
        "periodTo",
        "statementNumber",
        "openingBalance",
        "closingBalance",
        "lines",
        "bookingDate",
        "valueDate",
        "details",
        "counterpartyName",
        "reference",
        "amount",
        "direction",
    ];
    let got: std::collections::BTreeSet<&str> =
        ManualStatementField::ALL.iter().map(|f| f.code()).collect();
    let want: std::collections::BTreeSet<&str> = expected.into_iter().collect();
    assert_eq!(got, want, "ensemble des `field` ≠ celui de la fiche");
    assert_eq!(ManualStatementField::ALL.len(), expected.len());
}

/// `ALL.len() == COUNT` et `ALL[i].index() == i` : ordre, absence de doublon,
/// et `ALL` lié au `match` sans joker d'`index` (C-v016-2-20).
#[test]
fn all_is_indexed_and_counted() {
    assert_eq!(
        ManualStatementReason::ALL.len(),
        ManualStatementReason::COUNT
    );
    for (i, r) in ManualStatementReason::ALL.iter().enumerate() {
        assert_eq!(r.index(), i, "ManualStatementReason::ALL[{i}] = {r:?}");
    }
    assert_eq!(ManualStatementField::ALL.len(), ManualStatementField::COUNT);
    for (i, f) in ManualStatementField::ALL.iter().enumerate() {
        assert_eq!(f.index(), i, "ManualStatementField::ALL[{i}] = {f:?}");
    }
}

/// Compte les variantes déclarées dans le **texte source** de l'énumération :
/// une ligne du corps de `pub enum <name> {` qui commence par un identifiant en
/// capitale suivi d'une virgule. Ne dépend ni de `ALL` ni de `COUNT` : une
/// variante ajoutée avec son bras d'`index`, sans toucher `ALL`, rougit ici
/// (mutation M23-bis, C-v016-2-20).
///
/// ⚠️ **Limite** (revue de code P1, A6) : seules les variantes **unitaires**
/// écrites `Nom,` sur leur ligne sont comptées. Une variante à champs
/// (`Nom(u8),`, `Nom { … },`) ou deux variantes sur une ligne échapperaient au
/// décompte ; les deux énumérations sont unitaires et `Copy`, et le resteront
/// tant que `code()` rend une chaîne fixe par variante.
fn count_source_variants(name: &str) -> usize {
    let src = include_str!("manual.rs");
    let start = src
        .find(&format!("pub enum {name} {{"))
        .unwrap_or_else(|| panic!("`pub enum {name}` introuvable dans manual.rs"));
    let body = &src[start..];
    let end = body.find("\n}").expect("fin de l'énumération");
    body[..end]
        .lines()
        .skip(1)
        .map(str::trim)
        .filter(|l| {
            !l.starts_with("//")
                && l.ends_with(',')
                && l.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                && l[..l.len() - 1].chars().all(|c| c.is_ascii_alphanumeric())
        })
        .count()
}

#[test]
fn all_covers_every_declared_variant() {
    let n = count_source_variants("ManualStatementReason");
    assert_eq!(n, 11, "variantes de ManualStatementReason dans le source");
    assert_eq!(n, ManualStatementReason::COUNT);
    assert_eq!(n, ManualStatementReason::ALL.len());
    let n = count_source_variants("ManualStatementField");
    assert_eq!(n, 13, "variantes de ManualStatementField dans le source");
    assert_eq!(n, ManualStatementField::COUNT);
    assert_eq!(n, ManualStatementField::ALL.len());
}

// ---------------------------------------------------------------------------
// Analyse du corps
// ---------------------------------------------------------------------------

#[test]
fn parse_accepts_the_nominal_body() {
    let parsed = parse_manual_statement(&raw()).expect("corps nominal");
    assert_eq!(parsed, input());
}

#[test]
fn parse_treats_blank_optional_text_as_absent_and_trims() {
    let mut r = raw();
    r.statement_number = Some("   ".into());
    let l = &mut r.lines.as_mut().unwrap()[0];
    l.details = Some("  Assurance  ".into());
    l.reference = Some(" ".into());
    l.counterparty_name = Some("  Exemple SA ".into());
    l.value_date = Some("".into());
    let p = parse_manual_statement(&r).unwrap();
    assert_eq!(p.statement_number, None);
    assert_eq!(p.lines[0].details, "Assurance");
    assert_eq!(p.lines[0].reference, None);
    assert_eq!(p.lines[0].counterparty_name.as_deref(), Some("Exemple SA"));
    assert_eq!(p.lines[0].value_date, None);
}

#[test]
fn parse_refuses_missing_and_malformed_strings_with_their_field() {
    use ManualStatementField as F;
    use ManualStatementReason as R;
    let cases: Vec<(RawMutation, ManualStatementError)> = vec![
        (
            Box::new(|r| r.period_from = None),
            invalid(None, F::PeriodFrom, R::Required),
        ),
        (
            Box::new(|r| r.period_to = Some("2026-9-30".into())),
            invalid(None, F::PeriodTo, R::InvalidDate),
        ),
        (
            Box::new(|r| r.period_to = Some("2026-02-30".into())),
            invalid(None, F::PeriodTo, R::InvalidDate),
        ),
        (
            Box::new(|r| r.opening_balance = Some(" ".into())),
            invalid(None, F::OpeningBalance, R::Required),
        ),
        (
            Box::new(|r| r.closing_balance = Some("12,50".into())),
            invalid(None, F::ClosingBalance, R::InvalidNumber),
        ),
        (
            Box::new(|r| r.closing_balance = Some("1e3".into())),
            invalid(None, F::ClosingBalance, R::InvalidNumber),
        ),
        (
            Box::new(|r| r.lines = None),
            invalid(None, F::Lines, R::Required),
        ),
        (
            Box::new(|r| r.lines.as_mut().unwrap()[1].booking_date = None),
            invalid(Some(2), F::BookingDate, R::Required),
        ),
        (
            Box::new(|r| r.lines.as_mut().unwrap()[0].value_date = Some("15.09.2026".into())),
            invalid(Some(1), F::ValueDate, R::InvalidDate),
        ),
        (
            Box::new(|r| r.lines.as_mut().unwrap()[0].details = Some("  ".into())),
            invalid(Some(1), F::Details, R::Required),
        ),
        (
            Box::new(|r| r.lines.as_mut().unwrap()[1].amount = Some("abc".into())),
            invalid(Some(2), F::Amount, R::InvalidNumber),
        ),
        (
            Box::new(|r| r.lines.as_mut().unwrap()[1].amount = Some("12345678901234".into())),
            invalid(Some(2), F::Amount, R::TooManyDigits),
        ),
        (
            Box::new(|r| r.lines.as_mut().unwrap()[0].direction = Some("out".into())),
            invalid(Some(1), F::Direction, R::UnknownDirection),
        ),
        (
            Box::new(|r| r.lines.as_mut().unwrap()[0].direction = Some("".into())),
            invalid(Some(1), F::Direction, R::Required),
        ),
        (
            Box::new(|r| r.lines = Some(vec![raw_line("2026-09-03", "1", "credit"); 501])),
            invalid(None, F::Lines, R::TooManyLines),
        ),
    ];
    for (i, (mutate, expected)) in cases.into_iter().enumerate() {
        let mut r = raw();
        mutate(&mut r);
        assert_eq!(
            parse_manual_statement(&r).unwrap_err(),
            expected,
            "cas n° {i}"
        );
    }
}

#[test]
fn parse_accepts_500_lines() {
    let mut r = raw();
    r.opening_balance = Some("0".into());
    r.closing_balance = Some("500".into());
    r.lines = Some(vec![raw_line("2026-09-03", "1", "credit"); 500]);
    let p = parse_manual_statement(&r).unwrap();
    assert_eq!(
        validate_manual_statement(&p).unwrap().transaction_count,
        500
    );
}

// ---------------------------------------------------------------------------
// Règles de validation (AC-4)
// ---------------------------------------------------------------------------

#[test]
fn nominal_statement_is_valid_and_summed() {
    let s = validate_manual_statement(&input()).unwrap();
    assert_eq!(s.total_credits, dec!(50.00));
    assert_eq!(s.total_debits, dec!(320.25));
    assert_eq!(s.computed_closing_balance, dec!(729.75));
    assert_eq!(s.transaction_count, 2);
}

#[test]
fn zero_lines_statement_is_valid_when_balances_are_equal() {
    let mut i = input();
    i.lines.clear();
    i.closing_balance = i.opening_balance;
    assert_eq!(validate_manual_statement(&i).unwrap().transaction_count, 0);
}

#[test]
fn rule_1_period_inverted() {
    let mut i = input();
    i.period_from = d("2026-10-01");
    i.lines.clear();
    i.closing_balance = i.opening_balance;
    assert_eq!(
        validate_manual_statement(&i).unwrap_err(),
        invalid(
            None,
            ManualStatementField::PeriodTo,
            ManualStatementReason::PeriodInverted
        )
    );
    // Une période d'un seul jour est admise.
    i.period_from = d("2026-09-30");
    assert!(validate_manual_statement(&i).is_ok());
}

#[test]
fn rule_2_too_many_lines() {
    let mut i = input();
    i.opening_balance = Decimal::ZERO;
    i.lines = vec![line("2026-09-10", dec!(1), ManualDirection::Credit); 501];
    i.closing_balance = dec!(501);
    assert_eq!(
        validate_manual_statement(&i).unwrap_err(),
        invalid(
            None,
            ManualStatementField::Lines,
            ManualStatementReason::TooManyLines
        )
    );
}

#[test]
fn rule_3_booking_date_inside_period_value_date_free() {
    let mut i = input();
    i.lines[1].booking_date = d("2026-10-01");
    assert_eq!(
        validate_manual_statement(&i).unwrap_err(),
        invalid(
            Some(2),
            ManualStatementField::BookingDate,
            ManualStatementReason::OutOfPeriod
        )
    );
    let mut i = input();
    i.lines[0].booking_date = d("2026-08-31");
    assert_eq!(
        validate_manual_statement(&i).unwrap_err(),
        invalid(
            Some(1),
            ManualStatementField::BookingDate,
            ManualStatementReason::OutOfPeriod
        )
    );
    // Bornes incluses ; date de valeur hors période admise.
    let mut i = input();
    i.lines[0].booking_date = d("2026-09-01");
    i.lines[1].booking_date = d("2026-09-30");
    i.lines[1].value_date = Some(d("2026-10-02"));
    assert!(validate_manual_statement(&i).is_ok());
}

#[test]
fn rule_4_lengths_in_characters_after_trim() {
    use ManualStatementField as F;
    use ManualStatementReason::TooLong;
    // 500 caractères accentués (1000 octets) : admis ; 501 : refusé.
    let mut i = input();
    i.lines[0].details = "é".repeat(500);
    assert!(validate_manual_statement(&i).is_ok());
    i.lines[0].details = format!("  {}  ", "é".repeat(500));
    assert!(validate_manual_statement(&i).is_ok(), "trim avant compte");
    i.lines[0].details = "é".repeat(501);
    assert_eq!(
        validate_manual_statement(&i).unwrap_err(),
        invalid(Some(1), F::Details, TooLong)
    );

    let mut i = input();
    i.lines[1].counterparty_name = Some("ü".repeat(255));
    i.lines[1].reference = Some("ß".repeat(255));
    i.statement_number = Some("à".repeat(255));
    assert!(validate_manual_statement(&i).is_ok());
    i.lines[1].counterparty_name = Some("ü".repeat(256));
    assert_eq!(
        validate_manual_statement(&i).unwrap_err(),
        invalid(Some(2), F::CounterpartyName, TooLong)
    );
    i.lines[1].counterparty_name = None;
    i.lines[1].reference = Some("ß".repeat(256));
    assert_eq!(
        validate_manual_statement(&i).unwrap_err(),
        invalid(Some(2), F::Reference, TooLong)
    );
    i.lines[1].reference = None;
    i.statement_number = Some("à".repeat(256));
    assert_eq!(
        validate_manual_statement(&i).unwrap_err(),
        invalid(None, F::StatementNumber, TooLong)
    );
}

#[test]
fn rule_5_amount_positive() {
    use ManualStatementReason::NotPositive;
    for bad in [dec!(0), dec!(0.00), dec!(-5)] {
        let mut i = input();
        i.lines[1].amount = bad;
        assert_eq!(
            validate_manual_statement(&i).unwrap_err(),
            invalid(Some(2), ManualStatementField::Amount, NotPositive),
            "{bad}"
        );
    }
}

/// `"50.00"` et `"50.000"` admis, `"50.0100"` ≡ `"50.01"` admis, `"50.001"`
/// refusé — `"50.000"` distingue la règle de sa mutation M4 (`normalize()`
/// retiré).
#[test]
fn rule_5_decimals_after_normalize() {
    for (s, ok) in [
        ("50.00", true),
        ("50.000", true),
        ("50.0100", true),
        ("50.001", false),
    ] {
        let mut r = raw();
        r.lines.as_mut().unwrap()[1].amount = Some(s.into());
        // Le solde d'arrivée suit le montant admis, pour isoler la règle 5 ; il
        // reste à deux décimales pour le montant refusé (les soldes passent
        // la règle 5 avant les lignes).
        if ok {
            let amount: Decimal = s.parse().unwrap();
            r.closing_balance = Some((dec!(1000.00) - dec!(320.25) + amount).to_string());
        }
        // Depuis la revue de code P1 (B1), un excès de décimales est refusé dès
        // l'analyse, avec le même rang, le même champ et le même motif.
        let res = parse_manual_statement(&r).and_then(|p| validate_manual_statement(&p));
        if ok {
            assert!(res.is_ok(), "{s} doit être admis : {res:?}");
        } else {
            assert_eq!(
                res.unwrap_err(),
                invalid(
                    Some(2),
                    ManualStatementField::Amount,
                    ManualStatementReason::TooManyDecimals
                ),
                "{s}"
            );
        }
    }
}

/// La règle 5 porte aussi sur une entrée construite sans passer par l'analyse :
/// un montant d'échelle 3 sans décimale significative (`50.000`) est admis —
/// c'est ce qui distingue la règle de sa mutation M4 (`normalize()` retiré),
/// l'analyse ôtant désormais elle-même les zéros finals.
#[test]
fn rule_5_scale_is_checked_after_normalize_on_the_domain_input() {
    let mut i = input();
    i.lines[1].amount = Decimal::new(50_000, 3);
    assert_eq!(i.lines[1].amount.scale(), 3);
    assert!(validate_manual_statement(&i).is_ok());
    i.opening_balance = Decimal::new(1_000_000, 3);
    assert!(validate_manual_statement(&i).is_ok());
}

/// Revue de code P1 (B1 = A1 = E3) — la lecture est **exacte** : au-delà de deux
/// décimales significatives, le refus est `too_many_decimals`, jamais un arrondi
/// silencieux (`str::parse::<Decimal>` arrondissait au-delà de 28 décimales).
#[test]
fn parse_reads_amounts_exactly_and_refuses_long_fractions() {
    use ManualStatementReason as R;
    let amount_of = |s: &str| {
        let mut r = raw();
        r.lines.as_mut().unwrap()[1].amount = Some(s.into());
        parse_manual_statement(&r).map(|p| p.lines[1].amount)
    };
    let reason_of = |s: &str| match amount_of(s) {
        Err(ManualStatementError::Invalid { reason, .. }) => Some(reason),
        _ => None,
    };
    // 29 et 30 décimales, queue non nulle : refusées, jamais arrondies.
    assert_eq!(
        reason_of("50.00000000000000000000000000001"),
        Some(R::TooManyDecimals)
    );
    assert_eq!(
        reason_of("0.99999999999999999999999999999"),
        Some(R::TooManyDecimals)
    );
    assert_eq!(
        reason_of("0.00000000000000000000000000004"),
        Some(R::TooManyDecimals)
    );
    assert_eq!(reason_of("50.001"), Some(R::TooManyDecimals));
    // Zéros finals, en nombre quelconque : la valeur exacte.
    assert_eq!(
        amount_of("50.000000000000000000000000000000").unwrap(),
        dec!(50)
    );
    assert_eq!(amount_of("50.0100").unwrap(), dec!(50.01));
    // Formes (E6).
    assert_eq!(amount_of("0005.00").unwrap(), dec!(5));
    for bad in ["+5", "5.", ".5", "-", "5.5.5", "1e3", "5 0"] {
        assert_eq!(reason_of(bad), Some(R::InvalidNumber), "{bad}");
    }
    // `-0` : un montant de ligne nul est refusé à la règle 5 ; un solde `-0`
    // vaut 0.
    let mut r = raw();
    r.lines.as_mut().unwrap()[1].amount = Some("-0".into());
    let p = parse_manual_statement(&r).unwrap();
    assert_eq!(
        validate_manual_statement(&p).unwrap_err(),
        invalid(
            Some(2),
            ManualStatementField::Amount,
            ManualStatementReason::NotPositive
        )
    );
    let mut r = raw();
    r.lines = Some(vec![]);
    r.opening_balance = Some("-0".into());
    r.closing_balance = Some("0.00".into());
    let p = parse_manual_statement(&r).unwrap();
    assert!(p.opening_balance.is_zero());
    assert!(validate_manual_statement(&p).is_ok());
    assert_eq!(
        manual_statement_canonical_bytes(&p),
        {
            let mut q = p.clone();
            q.opening_balance = Decimal::ZERO;
            manual_statement_canonical_bytes(&q)
        },
        "-0 et 0 ont la même empreinte"
    );
    // Un montant négatif reste lu exactement.
    let mut r = raw();
    r.opening_balance = Some("-1250.40".into());
    assert_eq!(
        parse_manual_statement(&r).unwrap().opening_balance,
        dec!(-1250.40)
    );
}

/// Revue de code P1 (E1) — au-delà de 501 éléments, `lines` n'est plus
/// matérialisé : la désérialisation en retient 501, et le relevé est refusé
/// `too_many_lines` comme avant.
#[test]
fn lines_are_capped_at_deserialization() {
    let body = |n: usize| {
        format!(
            r#"{{"bankAccountId":3,"periodFrom":"2026-09-01","periodTo":"2026-09-30",
                 "openingBalance":"0","closingBalance":"0","lines":[{}]}}"#,
            vec!["{}"; n].join(",")
        )
    };
    let r: ManualStatementRaw = serde_json::from_str(&body(5000)).unwrap();
    assert_eq!(
        r.lines.as_ref().unwrap().len(),
        MANUAL_STATEMENT_MAX_LINES + 1
    );
    assert_eq!(
        parse_manual_statement(&r).unwrap_err(),
        invalid(
            None,
            ManualStatementField::Lines,
            ManualStatementReason::TooManyLines
        )
    );
    let r: ManualStatementRaw = serde_json::from_str(&body(500)).unwrap();
    assert_eq!(r.lines.as_ref().unwrap().len(), 500);
    let r: ManualStatementRaw = serde_json::from_str(&body(0)).unwrap();
    assert_eq!(r.lines.as_ref().unwrap().len(), 0);
    // `null` et absence : `None` ; une autre forme : rejet du corps.
    let r: ManualStatementRaw =
        serde_json::from_str(r#"{"bankAccountId":3,"lines":null}"#).unwrap();
    assert!(r.lines.is_none());
    let r: ManualStatementRaw = serde_json::from_str(r#"{"bankAccountId":3}"#).unwrap();
    assert!(r.lines.is_none());
    assert!(
        serde_json::from_str::<ManualStatementRaw>(r#"{"bankAccountId":3,"lines":"non"}"#).is_err()
    );
    // Un élément retenu d'un autre type reste un rejet du corps.
    assert!(
        serde_json::from_str::<ManualStatementRaw>(
            r#"{"bankAccountId":3,"lines":[{"amount":50}]}"#
        )
        .is_err()
    );
}

#[test]
fn rule_5_integer_digits_and_balances() {
    let mut i = input();
    i.lines.clear();
    i.opening_balance = dec!(-9999999999999.99); // treize chiffres, signe libre
    i.closing_balance = i.opening_balance;
    assert!(validate_manual_statement(&i).is_ok());
    i.opening_balance = dec!(10000000000000); // quatorze chiffres
    i.closing_balance = i.opening_balance;
    assert_eq!(
        validate_manual_statement(&i).unwrap_err(),
        invalid(
            None,
            ManualStatementField::OpeningBalance,
            ManualStatementReason::TooManyDigits
        )
    );
    let mut i = input();
    i.closing_balance = dec!(729.755);
    assert_eq!(
        validate_manual_statement(&i).unwrap_err(),
        invalid(
            None,
            ManualStatementField::ClosingBalance,
            ManualStatementReason::TooManyDecimals
        )
    );
}

/// Règle 6 — égalité exacte : un centime d'écart est refusé (mutation M1).
#[test]
fn rule_6_balance_is_exact() {
    let mut i = input();
    i.closing_balance = dec!(729.76);
    assert_eq!(
        validate_manual_statement(&i).unwrap_err(),
        ManualStatementError::Unbalanced {
            opening: dec!(1000.00),
            closing: dec!(729.76),
            computed_closing: dec!(729.75),
            diff: dec!(0.01),
        }
    );
    i.closing_balance = dec!(729.74);
    assert!(matches!(
        validate_manual_statement(&i),
        Err(ManualStatementError::Unbalanced { .. })
    ));
}

// ---------------------------------------------------------------------------
// Empreinte canonique (AC-5)
// ---------------------------------------------------------------------------

fn bytes(i: &ManualStatementInput) -> Vec<u8> {
    manual_statement_canonical_bytes(i)
}

#[test]
fn canonical_bytes_are_stable() {
    assert_eq!(bytes(&input()), bytes(&input()));
    // Ordre des champs JSON indifférent : deux corps qui ne diffèrent que par
    // l'ordre des clés s'analysent vers la même entrée.
    let a: ManualStatementRaw = serde_json::from_str(
        r#"{"bankAccountId":3,"periodFrom":"2026-09-01","periodTo":"2026-09-30",
            "openingBalance":"0","closingBalance":"50",
            "lines":[{"bookingDate":"2026-09-03","details":"x","amount":"50","direction":"credit"}]}"#,
    )
    .unwrap();
    let b: ManualStatementRaw = serde_json::from_str(
        r#"{"lines":[{"direction":"credit","amount":"50.00","details":" x ","bookingDate":"2026-09-03"}],
            "closingBalance":"50.00","openingBalance":"0.00",
            "periodTo":"2026-09-30","periodFrom":"2026-09-01","bankAccountId":3}"#,
    )
    .unwrap();
    assert_eq!(
        bytes(&parse_manual_statement(&a).unwrap()),
        bytes(&parse_manual_statement(&b).unwrap()),
        "\"50\" ≡ \"50.00\", ordre des clés et blancs indifférents"
    );
    assert!(bytes(&input()).starts_with(&[0, 0, 0, 24]));
    assert_eq!(
        &bytes(&input())[4..28],
        MANUAL_STATEMENT_HASH_DOMAIN.as_bytes()
    );
}

#[test]
fn canonical_bytes_are_sensitive_to_every_field() {
    let base = bytes(&input());
    let mutations: Vec<(&str, InputMutation)> = vec![
        ("compte", Box::new(|i| i.bank_account_id = 4)),
        ("du", Box::new(|i| i.period_from = d("2026-09-02"))),
        ("au", Box::new(|i| i.period_to = d("2026-09-29"))),
        (
            "numéro",
            Box::new(|i| i.statement_number = Some("10/2026".into())),
        ),
        ("numéro absent", Box::new(|i| i.statement_number = None)),
        ("départ", Box::new(|i| i.opening_balance = dec!(1000.01))),
        ("arrivée", Box::new(|i| i.closing_balance = dec!(729.76))),
        (
            "date",
            Box::new(|i| i.lines[0].booking_date = d("2026-09-04")),
        ),
        (
            "valeur",
            Box::new(|i| i.lines[0].value_date = Some(d("2026-09-04"))),
        ),
        ("libellé", Box::new(|i| i.lines[0].details = "Autre".into())),
        (
            "contrepartie",
            Box::new(|i| i.lines[0].counterparty_name = Some("X".into())),
        ),
        (
            "référence",
            Box::new(|i| i.lines[0].reference = Some("R".into())),
        ),
        ("montant", Box::new(|i| i.lines[0].amount = dec!(320.26))),
        (
            "sens",
            Box::new(|i| i.lines[0].direction = ManualDirection::Credit),
        ),
        ("ordre des lignes", Box::new(|i| i.lines.swap(0, 1))),
        (
            "une ligne en moins",
            Box::new(|i| {
                i.lines.pop();
            }),
        ),
    ];
    for (name, m) in mutations {
        let mut i = input();
        m(&mut i);
        assert_ne!(bytes(&i), base, "l'empreinte doit voir : {name}");
    }
}

/// Injectivité : déplacer la frontière entre deux champs voisins change les
/// octets (encodage par longueur, non par séparateur).
#[test]
fn canonical_bytes_are_injective_across_field_boundaries() {
    let mut a = input();
    a.lines[0].details = "a|b".into();
    a.lines[0].reference = Some("c".into());
    let mut b = input();
    b.lines[0].details = "a".into();
    b.lines[0].reference = Some("b|c".into());
    assert_ne!(bytes(&a), bytes(&b));

    let mut a = input();
    a.lines[0].details = "ab".into();
    a.lines[0].counterparty_name = Some("c".into());
    let mut b = input();
    b.lines[0].details = "a".into();
    b.lines[0].counterparty_name = Some("bc".into());
    assert_ne!(bytes(&a), bytes(&b));

    let mut a = input();
    a.lines[0].counterparty_name = Some("x".into());
    a.lines[0].reference = None;
    let mut b = input();
    b.lines[0].counterparty_name = None;
    b.lines[0].reference = Some("x".into());
    assert_ne!(bytes(&a), bytes(&b));
}

// ---------------------------------------------------------------------------
// Brouillons
// ---------------------------------------------------------------------------

#[test]
fn drafts_carry_manual_format_balances_and_signed_amounts() {
    let at = d("2026-10-01").and_hms_opt(8, 0, 0).unwrap();
    let mut i = input();
    i.lines[1].reference = Some("F-2026-0042".into());
    i.lines[1].counterparty_name = Some("Exemple SA".into());
    i.lines[1].value_date = Some(d("2026-09-16"));
    let (h, txs) = manual_statement_to_drafts(
        &i,
        3,
        7,
        "ab".repeat(32),
        manual_statement_filename(i.period_from, i.period_to),
        at,
        11,
    );
    assert_eq!(h.source_format, SourceFormatTag::Manual);
    assert_eq!(h.source_format.as_db_str(), "MANUAL");
    assert_eq!(h.filename, "saisie-manuelle-2026-09-01-2026-09-30");
    assert_eq!(h.statement_id.as_deref(), Some("9/2026"));
    assert_eq!(h.opening_balance, Some(Money::new(dec!(1000.00))));
    assert_eq!(h.closing_balance, Some(Money::new(dec!(729.75))));
    assert_eq!(h.transaction_count, 2);
    assert_eq!(
        (h.company_id, h.bank_account_id, h.imported_by_user_id),
        (7, 3, 11)
    );
    assert_eq!(txs[0].amount, Money::new(dec!(-320.25)), "débit négatif");
    assert_eq!(txs[1].amount, Money::new(dec!(50.00)), "crédit positif");
    assert!(txs.iter().all(|t| t.currency == "CHF"));
    assert_eq!(txs[1].reference.as_deref(), Some("F-2026-0042"));
    assert_eq!(txs[1].counterparty_name.as_deref(), Some("Exemple SA"));
    assert_eq!(txs[1].value_date, Some(d("2026-09-16")));
    assert!(txs.iter().all(|t| t.end_to_end_id.is_none()
        && t.transaction_id.is_none()
        && t.counterparty_iban.is_none()));
}
