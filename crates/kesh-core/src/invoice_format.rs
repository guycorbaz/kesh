//! Format de numérotation des factures (Story 5.2 — FR35).
//!
//! Helper pur (zéro I/O) pour rendre et valider les templates de numéro
//! de facture. Utilisé par [`kesh_db::repositories::invoices::validate_invoice`]
//! (rendu) et par les handlers `PUT /company/invoice-settings` (validation).
//!
//! # Placeholders supportés
//!
//! - `{YEAR}` : année du fiscal_year (i32).
//! - `{FY}` : nom littéral du fiscal_year (ex. « 2026 » ou « 2025/2026 »).
//! - `{SEQ}` : numéro séquentiel sans padding.
//! - `{SEQ:NN}` : numéro avec padding zéros. NN ∈ [1, 10].
//!
//! # Règles de validation (`validate_template`)
//!
//! - Caractères autorisés : `[A-Za-z0-9\-_/.#\s{}:]`.
//! - Longueur du template ≤ 64.
//! - Au moins 1 placeholder reconnu présent.
//! - Pour chaque `{SEQ:NN}` : 1 ≤ NN ≤ 10.
//! - Longueur du rendu maximal ≤ 64 (cas worst-case `seq = 10^NN - 1`).

use std::fmt;

/// Longueur max du template et du rendu final.
pub const MAX_TEMPLATE_LEN: usize = 64;
/// Longueur max du rendu final (identique à la VARCHAR(64) DB).
pub const MAX_RENDERED_LEN: usize = 64;
/// Borne haute du padding `{SEQ:NN}`.
pub const MAX_SEQ_PADDING: u32 = 10;

/// Erreur de format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatError {
    Empty,
    TemplateTooLong(usize),
    IllegalCharacter(char),
    UnknownPlaceholder(String),
    InvalidSeqPadding(u32),
    /// Numéro séquentiel invalide (< 1). Review Q4 (pass 2) : variant
    /// dédié pour distinguer d'une erreur de padding.
    InvalidSeq(i64),
    NoPlaceholder,
    RenderedTooLong(usize),
    UnmatchedBrace,
    /// Le format n'a pas la forme d'un format de numérotation
    /// `<préfixe>{YEAR}<séparateur>{SEQ}` — cf. [`validate_numbering_template`].
    NotNumberingShape,
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "Le format de numérotation est vide"),
            Self::TemplateTooLong(n) => write!(
                f,
                "Le format de numérotation dépasse {MAX_TEMPLATE_LEN} caractères (actuel : {n})"
            ),
            Self::IllegalCharacter(c) => {
                write!(f, "Caractère non autorisé dans le format : '{c}'")
            }
            Self::UnknownPlaceholder(p) => {
                write!(f, "Placeholder inconnu : {{{p}}}")
            }
            Self::InvalidSeqPadding(n) => write!(
                f,
                "Padding {{SEQ:{n}}} invalide — doit être entre 1 et {MAX_SEQ_PADDING}"
            ),
            Self::InvalidSeq(n) => write!(f, "Numéro séquentiel invalide : {n} (doit être ≥ 1)"),
            Self::NoPlaceholder => write!(
                f,
                "Le format doit contenir au moins un placeholder reconnu ({{YEAR}}, {{FY}}, {{SEQ}}, {{SEQ:NN}})"
            ),
            Self::RenderedTooLong(n) => write!(
                f,
                "Le format générerait un numéro de {n} caractères (max {MAX_RENDERED_LEN})"
            ),
            Self::UnmatchedBrace => write!(f, "Accolade non appariée dans le format"),
            Self::NotNumberingShape => write!(
                f,
                "Le format doit s'écrire : un préfixe libre, {{YEAR}}, un séparateur sans chiffre, \
                 puis {{SEQ}} ou {{SEQ:NN}}, rien après (exemple : FF-{{YEAR}}-{{SEQ:03}})"
            ),
        }
    }
}

impl std::error::Error for FormatError {}

fn is_allowed_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || matches!(
            c,
            '-' | '_' | '/' | '.' | '#' | '{' | '}' | ':' | ' ' | '\t'
        )
}

/// Token intermédiaire parsé.
enum Token<'a> {
    Literal(&'a str),
    Year,
    Fy,
    Seq(Option<u32>),
}

fn parse(template: &str) -> Result<Vec<Token<'_>>, FormatError> {
    let bytes = template.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0;
    let mut literal_start = 0;

    while i < bytes.len() {
        let c = template[i..].chars().next().unwrap();
        if !is_allowed_char(c) {
            return Err(FormatError::IllegalCharacter(c));
        }
        if c == '{' {
            if literal_start < i {
                tokens.push(Token::Literal(&template[literal_start..i]));
            }
            // Chercher la '}' fermante.
            let close = template[i + 1..]
                .find('}')
                .ok_or(FormatError::UnmatchedBrace)?;
            let inner = &template[i + 1..i + 1 + close];
            let token = match inner {
                "YEAR" => Token::Year,
                "FY" => Token::Fy,
                "SEQ" => Token::Seq(None),
                s if s.starts_with("SEQ:") => {
                    let n_str = &s[4..];
                    let n: u32 = n_str
                        .parse()
                        .map_err(|_| FormatError::InvalidSeqPadding(0))?;
                    if !(1..=MAX_SEQ_PADDING).contains(&n) {
                        return Err(FormatError::InvalidSeqPadding(n));
                    }
                    Token::Seq(Some(n))
                }
                other => return Err(FormatError::UnknownPlaceholder(other.to_string())),
            };
            tokens.push(token);
            i += 1 + close + 1;
            literal_start = i;
        } else if c == '}' {
            return Err(FormatError::UnmatchedBrace);
        } else {
            i += c.len_utf8();
        }
    }

    if literal_start < bytes.len() {
        tokens.push(Token::Literal(&template[literal_start..]));
    }

    Ok(tokens)
}

/// Valide un template sans rendre.
pub fn validate_template(template: &str) -> Result<(), FormatError> {
    if template.trim().is_empty() {
        return Err(FormatError::Empty);
    }
    if template.len() > MAX_TEMPLATE_LEN {
        return Err(FormatError::TemplateTooLong(template.len()));
    }
    let tokens = parse(template)?;

    let mut has_placeholder = false;
    let mut worst_case_len = 0usize;
    for tok in &tokens {
        match tok {
            Token::Literal(s) => worst_case_len += s.len(),
            Token::Year => {
                has_placeholder = true;
                worst_case_len += 4; // année 4 chiffres max raisonnable
            }
            Token::Fy => {
                has_placeholder = true;
                // Nom fy peut être « 2025/2026 » → 9 chars ; on borne à 16 pour la vérif worst-case.
                worst_case_len += 16;
            }
            Token::Seq(None) => {
                has_placeholder = true;
                // seq est `i64` (BIGINT en DB, CHECK next_number ≥ 1). Worst case
                // i64::MAX = 19 chiffres. Review P2 : la borne précédente
                // (MAX_SEQ_PADDING=10) sous-estimait la taille maximale rendue.
                worst_case_len += 19;
            }
            Token::Seq(Some(n)) => {
                has_placeholder = true;
                // Le padding garantit au moins `n` chiffres, mais le numéro peut
                // dépasser `10^n - 1` → prendre max(n, 19) pour couvrir le worst
                // case i64 (review P2).
                worst_case_len += std::cmp::max(*n as usize, 19);
            }
        }
    }

    if !has_placeholder {
        return Err(FormatError::NoPlaceholder);
    }
    if worst_case_len > MAX_RENDERED_LEN {
        return Err(FormatError::RenderedTooLong(worst_case_len));
    }
    Ok(())
}

/// Valide un **format de numérotation interne** (Story 33-1a, #926 ; C-e331-16) :
/// [`validate_template`], puis la forme `<préfixe>{YEAR}<séparateur>{SEQ}` ou
/// `{SEQ:NN}` — un seul `{YEAR}` suivi d'un seul compteur, préfixe libre (vide
/// admis), séparateur non vide et sans chiffre, rien après le compteur, aucun
/// `{FY}`, ni blanc en tête ni tabulation : invisibles sur le papier, ils rendraient
/// un numéro impossible à recopier.
///
/// Cette forme est ce qui rend un numéro **injectif**, pour une année écrite sur
/// quatre chiffres : le rang est la suite de chiffres finale, le séparateur ce qui
/// la précède sans chiffre, l'année les quatre chiffres d'avant. Deux formats ne
/// rendent donc la même chaîne que pour la même année et le même rang, si bien
/// qu'un changement de format ne fait revenir aucun numéro. Les formats client et
/// avoir gardent [`validate_template`].
pub fn validate_numbering_template(template: &str) -> Result<(), FormatError> {
    validate_template(template)?;
    if template.starts_with(char::is_whitespace) || template.contains('\t') {
        return Err(FormatError::NotNumberingShape);
    }
    let tokens = parse(template)?;
    let sans_prefixe = match tokens.as_slice() {
        [Token::Literal(_), reste @ ..] => reste,
        tous => tous,
    };
    match sans_prefixe {
        [Token::Year, Token::Literal(separateur), Token::Seq(_)]
            if !separateur.chars().any(|c| c.is_ascii_digit()) =>
        {
            Ok(())
        }
        _ => Err(FormatError::NotNumberingShape),
    }
}

/// Le numéro sans ses blancs de bord ; vide ou blanc, il vaut absent (règle de K8,
/// `closing_checks`, dont le SQL est le miroir de ces libellés).
fn non_blank(numero: Option<&str>) -> Option<&str> {
    numero.map(str::trim).filter(|n| !n.is_empty())
}

/// La forme **courte** du libellé d'une facture fournisseur, celle qui la nomme
/// dans une phrase : le numéro interne, à défaut celui du fournisseur. Un numéro
/// vide ou blanc vaut absent ; `None` si aucun des deux n'existe, à l'appelant de
/// dire `#<id>` (Story 33-1a, #926 ; C-e331-20).
pub fn supplier_invoice_short_label(
    interne: Option<&str>,
    fournisseur: Option<&str>,
) -> Option<String> {
    non_blank(interne)
        .or(non_blank(fournisseur))
        .map(str::to_string)
}

/// La forme **longue** du libellé d'une facture fournisseur, celle qui sert à
/// reconnaître la pièce : `FF-2026-001 (ABC-123)`, le numéro interne seul si le
/// fournisseur n'en a pas donné, celui du fournisseur seul si la facture n'a pas de
/// numéro interne. Un numéro vide ou blanc vaut absent ; `None` si aucun des deux
/// n'existe, à l'appelant de dire `#<id>` (Story 33-1a, #926 ; C-e331-10).
pub fn supplier_invoice_long_label(
    interne: Option<&str>,
    fournisseur: Option<&str>,
) -> Option<String> {
    match (non_blank(interne), non_blank(fournisseur)) {
        (Some(i), Some(f)) => Some(format!("{i} ({f})")),
        (Some(n), None) | (None, Some(n)) => Some(n.to_string()),
        (None, None) => None,
    }
}

/// Rend un template avec les valeurs données.
///
/// Appelle `validate_template` en amont — si votre caller a déjà validé
/// le template à l'écriture, cette fonction ne peut échouer qu'en cas
/// de `seq` négatif ou de `fy_name` contenant des caractères interdits
/// dans la sortie (aucune vérif ici — responsabilité du caller).
pub fn render(template: &str, year: i32, fy_name: &str, seq: i64) -> Result<String, FormatError> {
    // Review P14 : DB CHECK `next_number >= 1` devrait garantir seq ≥ 1,
    // mais défense en profondeur si un caller contourne le repository.
    if seq < 1 {
        return Err(FormatError::InvalidSeq(seq));
    }
    // Review Q2 (pass 2) : fy.name est admin-controlled à la création de
    // l'exercice. Un nom contenant '{' ou '}' pourrait produire un numéro
    // visuellement trompeur. Refuser côté render (défense en profondeur).
    if fy_name.contains('{') || fy_name.contains('}') {
        return Err(FormatError::IllegalCharacter(
            fy_name.chars().find(|c| *c == '{' || *c == '}').unwrap(),
        ));
    }
    let tokens = parse(template)?;
    let seq_str = seq.to_string();

    let mut out = String::with_capacity(template.len() + 16);
    for tok in tokens {
        match tok {
            Token::Literal(s) => out.push_str(s),
            Token::Year => out.push_str(&year.to_string()),
            Token::Fy => out.push_str(fy_name),
            Token::Seq(None) => out.push_str(&seq_str),
            Token::Seq(Some(n)) => {
                let width = n as usize;
                if seq_str.len() < width {
                    out.push_str(&"0".repeat(width - seq_str.len()));
                }
                out.push_str(&seq_str);
            }
        }
    }

    if out.len() > MAX_RENDERED_LEN {
        return Err(FormatError::RenderedTooLong(out.len()));
    }
    Ok(out)
}

/// Rend le template de libellé d'écriture comptable.
///
/// Placeholders : `{YEAR}`, `{INVOICE_NUMBER}`, `{CONTACT_NAME}`.
/// Longueur max 128 (aligne `company_invoice_settings.journal_entry_description_template`).
pub fn render_journal_entry_description(
    template: &str,
    year: i32,
    invoice_number: &str,
    contact_name: &str,
) -> String {
    template
        .replace("{YEAR}", &year.to_string())
        .replace("{INVOICE_NUMBER}", invoice_number)
        .replace("{CONTACT_NAME}", contact_name)
}

/// Valide un template de libellé (longueur + présence d'au moins 1
/// placeholder reconnu + pas de `{...}` inconnu).
pub fn validate_description_template(template: &str) -> Result<(), FormatError> {
    const MAX: usize = 128;
    if template.trim().is_empty() {
        return Err(FormatError::Empty);
    }
    // Review P17 : longueur en caractères (pas bytes) pour gérer l'UTF-8.
    let char_count = template.chars().count();
    if char_count > MAX {
        return Err(FormatError::TemplateTooLong(char_count));
    }
    // Review P11 : rejeter les caractères de contrôle (< 0x20 hors tab).
    for c in template.chars() {
        let cp = c as u32;
        if cp < 0x20 && cp != 0x09 {
            return Err(FormatError::IllegalCharacter(c));
        }
    }

    // Vérifier que toutes les accolades forment des placeholders connus.
    let mut i = 0;
    let bytes = template.as_bytes();
    let mut has_placeholder = false;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            let close = template[i + 1..]
                .find('}')
                .ok_or(FormatError::UnmatchedBrace)?;
            let inner = &template[i + 1..i + 1 + close];
            match inner {
                "YEAR" | "INVOICE_NUMBER" | "CONTACT_NAME" => has_placeholder = true,
                other => return Err(FormatError::UnknownPlaceholder(other.to_string())),
            }
            i += 1 + close + 1;
        } else if bytes[i] == b'}' {
            return Err(FormatError::UnmatchedBrace);
        } else {
            i += 1;
        }
    }

    if !has_placeholder {
        return Err(FormatError::NoPlaceholder);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_f_year_seq04() {
        assert_eq!(
            render("F-{YEAR}-{SEQ:04}", 2026, "2026", 42).unwrap(),
            "F-2026-0042"
        );
    }

    #[test]
    fn renders_fact_seq() {
        assert_eq!(render("FACT{SEQ}", 2026, "2026", 1).unwrap(), "FACT1");
    }

    #[test]
    fn renders_fy_seq06() {
        assert_eq!(
            render("{FY}/{SEQ:06}", 2026, "2025/2026", 42).unwrap(),
            "2025/2026/000042"
        );
    }

    #[test]
    fn renders_seq_over_padding_keeps_full_number() {
        // seq > 10^NN - 1 : le numéro n'est pas tronqué (comportement attendu,
        // mais validate_template aura refusé si worst-case > 64).
        assert_eq!(render("{SEQ:03}", 2026, "2026", 12345).unwrap(), "12345");
    }

    #[test]
    fn validates_template_ok_simple() {
        assert!(validate_template("F-{YEAR}-{SEQ:04}").is_ok());
    }

    #[test]
    fn validates_template_empty_rejected() {
        assert_eq!(validate_template("   "), Err(FormatError::Empty));
    }

    #[test]
    fn validates_template_no_placeholder_rejected() {
        assert_eq!(validate_template("FACT"), Err(FormatError::NoPlaceholder));
    }

    #[test]
    fn validates_template_unknown_placeholder_rejected() {
        assert!(matches!(
            validate_template("{INVALID}"),
            Err(FormatError::UnknownPlaceholder(_))
        ));
    }

    #[test]
    fn validates_template_illegal_char_rejected() {
        assert!(matches!(
            validate_template("F-{YEAR}-{SEQ:04}!"),
            Err(FormatError::IllegalCharacter('!'))
        ));
    }

    #[test]
    fn test_padding_nn_zero_rejected() {
        assert_eq!(
            validate_template("{SEQ:0}"),
            Err(FormatError::InvalidSeqPadding(0))
        );
    }

    #[test]
    fn test_padding_nn_above_10_rejected() {
        assert_eq!(
            validate_template("{SEQ:11}"),
            Err(FormatError::InvalidSeqPadding(11))
        );
    }

    #[test]
    fn test_max_padding_within_varchar64() {
        // {SEQ:10} → worst case 10 chars, + année 4 chars + "F-" = 16 chars, OK.
        assert!(validate_template("F-{YEAR}-{SEQ:10}").is_ok());
    }

    #[test]
    fn validates_template_too_long_rejected() {
        let t = "a".repeat(65);
        assert!(matches!(
            validate_template(&t),
            Err(FormatError::TemplateTooLong(_))
        ));
    }

    #[test]
    fn description_template_render_all_placeholders() {
        assert_eq!(
            render_journal_entry_description(
                "{YEAR}-{INVOICE_NUMBER} - {CONTACT_NAME}",
                2026,
                "F-2026-0042",
                "Acme SA"
            ),
            "2026-F-2026-0042 - Acme SA"
        );
    }

    #[test]
    fn description_template_default_rendered() {
        assert_eq!(
            render_journal_entry_description("{YEAR}-{INVOICE_NUMBER}", 2026, "F-2026-0001", "X"),
            "2026-F-2026-0001"
        );
    }

    #[test]
    fn description_template_validation_ok() {
        assert!(validate_description_template("{YEAR}-{INVOICE_NUMBER}").is_ok());
    }

    #[test]
    fn description_template_validation_unknown_rejected() {
        assert!(matches!(
            validate_description_template("{FOO}"),
            Err(FormatError::UnknownPlaceholder(_))
        ));
    }

    // Story 33-1a (#926) — le format de numérotation interne (AC16).
    #[test]
    fn numbering_template_accepts_the_numbering_shape() {
        for ok in [
            "FF-{YEAR}-{SEQ:03}",
            "KF-{YEAR}-{SEQ:04}",
            "{YEAR}/{SEQ}",
            "F1-{YEAR}-{SEQ}",
        ] {
            assert_eq!(validate_numbering_template(ok), Ok(()), "{ok}");
        }
    }

    #[test]
    fn numbering_template_refuses_every_other_shape() {
        for ko in [
            "FF-{YEAR}",
            "FF-{SEQ:03}",
            "FF-{FY}-{SEQ}",
            "FF-{FY}-{YEAR}-{SEQ}",
            "{YEAR}-{YEAR}-{SEQ}",
            "{SEQ}-{YEAR}",
            "FF-{YEAR}{SEQ}",
            "FF-{YEAR}-1{SEQ}",
            "FF-{YEAR}-{SEQ}-X",
            " FF-{YEAR}-{SEQ}",
            "\tFF-{YEAR}-{SEQ}",
            "FF\t-{YEAR}-{SEQ}",
            "FF-{YEAR}\t{SEQ}",
        ] {
            assert_eq!(
                validate_numbering_template(ko),
                Err(FormatError::NotNumberingShape),
                "{ko}"
            );
        }
        assert_eq!(validate_numbering_template(""), Err(FormatError::Empty));
        assert_eq!(validate_numbering_template("   "), Err(FormatError::Empty));
        assert_eq!(
            validate_numbering_template("FF-{YEAR}-{SEQ:11}"),
            Err(FormatError::InvalidSeqPadding(11))
        );
    }

    // Story 33-1a (#926) — les libellés d'une facture fournisseur (AC15).
    #[test]
    fn supplier_invoice_labels_prefer_the_internal_number() {
        type Numero<'a> = Option<&'a str>;
        let cas: [(Numero, Numero, Numero, Numero); 8] = [
            (
                Some("FF-2026-001"),
                Some("ABC-123"),
                Some("FF-2026-001"),
                Some("FF-2026-001 (ABC-123)"),
            ),
            (
                Some("FF-2026-001"),
                None,
                Some("FF-2026-001"),
                Some("FF-2026-001"),
            ),
            (
                Some("FF-2026-001"),
                Some("   "),
                Some("FF-2026-001"),
                Some("FF-2026-001"),
            ),
            (None, Some("ABC-123"), Some("ABC-123"), Some("ABC-123")),
            (
                Some("  "),
                Some("ABC-123"),
                Some("ABC-123"),
                Some("ABC-123"),
            ),
            (None, None, None, None),
            (Some(""), Some(" "), None, None),
            (
                Some(" FF-2026-001 "),
                Some(" ABC "),
                Some("FF-2026-001"),
                Some("FF-2026-001 (ABC)"),
            ),
        ];
        for (interne, fournisseur, courte, longue) in cas {
            assert_eq!(
                supplier_invoice_short_label(interne, fournisseur).as_deref(),
                courte,
                "courte : {interne:?}, {fournisseur:?}"
            );
            assert_eq!(
                supplier_invoice_long_label(interne, fournisseur).as_deref(),
                longue,
                "longue : {interne:?}, {fournisseur:?}"
            );
        }
    }
}
