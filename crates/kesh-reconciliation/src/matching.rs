//! Algorithme de matching transaction bancaire ↔ facture (Story 8-4 FR44).
//!
//! Helper pure (sans I/O) calculant des propositions de réconciliation
//! avec un score de confiance pondéré :
//!
//! - **Montant** (0.50) : `1.0` si exact ([`Decimal::normalize`]),
//!   sinon `0.0` (pas de gradient — décision conservatrice v0.1). Le montant
//!   comparé est le **montant à régler** de la facture, son **reste dû**
//!   (`TTC − avoir émis − Σ règlements`, #420, Story 25-4-c) : un virement qui
//!   solde une facture réglée en partie porte ce reste, pas le TTC — et
//!   encore moins le HT `invoice.total_amount` (#246). Le crate restant pur
//!   (pas d'accès DB), le caller fournit ce montant dans le tuple candidat.
//! - **Référence** (0.40) : `1.0` si containment bidirectionnel post
//!   normalisation `coalesce(reference, end_to_end_id, transaction_id)`
//!   vs `invoice.invoice_number`, `0.5` si common prefix ≥ 4 chars
//!   bidirectionnel, sinon `0.0`.
//! - **Contact** (0.10) : `1.0` si substring bidirectionnel sur
//!   `tx.counterparty_name` ↔ `contact.name` post normalisation,
//!   sinon `0.0`.
//!
//! Score final = pondération sommée ∈ `[0.0..=1.0]`. Les paires
//! avec `score.total > 0.0` sont retournées triées par
//! `score.total DESC`, puis, **à score égal**, par l'ordre d'imputation de
//! la loi (art. 87 CO, Story 29-5, #668 —
//! [`kesh_db::repositories::reconciliation::imputation_order`]) : la facture
//! échue la première passe en tête ; une facture sans échéance est tenue pour
//! échue dès sa date. Jusqu'à la v0.13.3, le plus petit `invoice_id`
//! l'emportait. Le score n'en est pas touché : une facture désignée par la
//! référence et au montant exact garde la tête devant toute facture que la
//! référence ne désigne pas aussi. Le caller (`kesh-api`) charge les `Contact`
//! correspondants en parallèle des `Invoice` candidates pour le
//! scoring contact.

use kesh_db::entities::bank_transaction::BankTransaction;
use kesh_db::entities::contact::Contact;
use kesh_db::entities::invoice::Invoice;
use kesh_db::repositories::reconciliation::imputation_order;
use rust_decimal::Decimal;

/// Sub-scores et score total pour une proposition de matching.
///
/// **MP3-4 Pass 3** : `rename_all = "camelCase"` explicite pour
/// produire `{ total, amountScore, referenceScore, contactScore }`
/// côté JSON (cohérent AC #44 + audit log shape).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchScore {
    pub total: f64,
    pub amount_score: f64,
    pub reference_score: f64,
    pub contact_score: f64,
}

/// Proposition de matching d'une transaction bancaire vers une facture.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchProposal {
    pub bank_transaction_id: i64,
    pub invoice_id: i64,
    pub score: MatchScore,
}

/// Calcule les propositions pour UNE transaction bancaire contre N
/// candidates. Retourne uniquement les paires `score.total > 0.0`,
/// triées par `score.total DESC`, puis, à score égal, par
/// [`imputation_order`] des factures (date d'exigibilité, date, `id`
/// croissants — art. 87 CO, Story 29-5). Pure (zéro I/O).
///
/// Chaque candidat est un triplet `(facture, contact, montant à régler)` — le
/// **reste dû** de la facture (#420, Story 25-4-c), fourni par le caller (le
/// crate reste pur) et comparé au montant de la transaction bancaire.
pub fn propose_matches(
    tx: &BankTransaction,
    candidates: &[(Invoice, Option<Contact>, Decimal)],
) -> Vec<MatchProposal> {
    // La facture reste à côté de sa proposition le temps du tri (référence,
    // sans clone) : le départage la lit, `MatchProposal` n'en porte que l'id.
    let mut out: Vec<(&Invoice, MatchProposal)> = candidates
        .iter()
        .filter_map(|(invoice, contact, amount_to_settle)| {
            let amount_score = amount_score(tx.amount, *amount_to_settle);
            let reference_score = reference_score(
                tx.reference.as_deref(),
                tx.end_to_end_id.as_deref(),
                tx.transaction_id.as_deref(),
                invoice.invoice_number.as_deref(),
            );
            let contact_score = contact_score(
                tx.counterparty_name.as_deref(),
                contact.as_ref().map(|c| c.name.as_str()),
            );
            let total = 0.50 * amount_score + 0.40 * reference_score + 0.10 * contact_score;
            if total > 0.0 {
                Some((
                    invoice,
                    MatchProposal {
                        bank_transaction_id: tx.id,
                        invoice_id: invoice.id,
                        score: MatchScore {
                            total,
                            amount_score,
                            reference_score,
                            contact_score,
                        },
                    },
                ))
            } else {
                None
            }
        })
        .collect();
    // Tri par score décroissant ; à score égal, par l'ordre d'imputation
    // (art. 87 CO, Story 29-5, #668) : la facture échue la première d'abord,
    // puis la plus ancienne, puis le plus petit identifiant. L'identifiant
    // restant la dernière composante, l'ordre est total et déterministe — la
    // proposition affichée, et acceptée d'un clic, ne dépend pas de l'ordre
    // de chargement des candidates.
    out.sort_by(|(inv_a, a), (inv_b, b)| {
        b.score
            .total
            .partial_cmp(&a.score.total)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| imputation_order(inv_a, inv_b))
    });
    out.into_iter().map(|(_, proposal)| proposal).collect()
}

fn amount_score(tx_amount: rust_decimal::Decimal, amount_to_settle: rust_decimal::Decimal) -> f64 {
    // M4 (équivalent Story 8-3) : `Decimal::normalize()` strip trailing
    // zeros pour matcher 1.50 == 1.5 (DB peut renvoyer scale différent
    // du parser). Décision v0.1 : binaire 0/1 (pas de gradient sur
    // écart de centimes) — cf. L17.
    if tx_amount.normalize() == amount_to_settle.normalize() {
        1.0
    } else {
        0.0
    }
}

fn reference_score(
    tx_ref: Option<&str>,
    tx_eid: Option<&str>,
    tx_tid: Option<&str>,
    invoice_number: Option<&str>,
) -> f64 {
    let tx_norm = normalize(coalesce(tx_ref, tx_eid, tx_tid));
    let inv_norm = normalize(invoice_number.unwrap_or(""));
    if tx_norm.is_empty() || inv_norm.is_empty() {
        return 0.0;
    }
    if tx_norm.contains(&inv_norm) || inv_norm.contains(&tx_norm) {
        return 1.0;
    }
    // A2-1 Pass 2 : `.chars()` au lieu de `.bytes()` pour boundary
    // UTF-8 Unicode-aware (noms suisses Müller, École, René, Sàrl).
    let common_prefix = tx_norm
        .chars()
        .zip(inv_norm.chars())
        .take_while(|(a, b)| a == b)
        .count();
    if common_prefix >= 4 {
        return 0.5;
    }
    0.0
}

fn contact_score(tx_counterparty: Option<&str>, contact_name: Option<&str>) -> f64 {
    let tx_norm = normalize(tx_counterparty.unwrap_or(""));
    let contact_norm = normalize(contact_name.unwrap_or(""));
    if tx_norm.is_empty() || contact_norm.is_empty() {
        return 0.0;
    }
    // H4 Pass 1 : bidirectionnel — CAMT.053 contient souvent un nom
    // plus long (ville/forme juridique) que `contact.name`, mais
    // l'inverse est aussi possible. Trade-off documenté L28
    // (false positives entre formes juridiques courtes).
    if tx_norm.contains(&contact_norm) || contact_norm.contains(&tx_norm) {
        1.0
    } else {
        0.0
    }
}

/// `trim().to_lowercase()` — normalisation case+espace pour matching.
fn normalize(s: &str) -> String {
    s.trim().to_lowercase()
}

/// Premier `Some(s)` non-vide de la chaîne `a → b → c`, sinon `""`.
fn coalesce<'a>(a: Option<&'a str>, b: Option<&'a str>, c: Option<&'a str>) -> &'a str {
    a.filter(|s| !s.trim().is_empty())
        .or_else(|| b.filter(|s| !s.trim().is_empty()))
        .or_else(|| c.filter(|s| !s.trim().is_empty()))
        .unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveDateTime};
    use kesh_db::entities::bank_transaction::BankTransactionStatus;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    fn make_tx(
        amount: Decimal,
        reference: Option<&str>,
        counterparty: Option<&str>,
    ) -> BankTransaction {
        BankTransaction {
            id: 1,
            company_id: 100,
            import_id: 1,
            bank_account_id: 17,
            booking_date: NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
            value_date: None,
            amount,
            currency: "CHF".to_string(),
            reference: reference.map(String::from),
            details: String::new(),
            end_to_end_id: None,
            transaction_id: None,
            counterparty_iban: None,
            counterparty_name: counterparty.map(String::from),
            status: BankTransactionStatus::Pending,
            matched_entry_id: None,
            auto_match_rejected_at: None,
            version: 1,
            created_at: NaiveDateTime::default(),
            updated_at: NaiveDateTime::default(),
        }
    }

    fn make_invoice(
        id: i64,
        total: Decimal,
        invoice_number: Option<&str>,
        contact_id: i64,
    ) -> Invoice {
        Invoice {
            id,
            company_id: 100,
            contact_id,
            invoice_number: invoice_number.map(String::from),
            status: "validated".to_string(),
            date: NaiveDate::from_ymd_opt(2026, 5, 10).unwrap(),
            due_date: None,
            payment_terms: None,
            total_amount: total,
            rounding_amount: Decimal::ZERO,
            journal_entry_id: Some(999),
            paid_at: None,
            emailed_at: None,
            emailed_to: None,
            project_id: None,
            dunning_paused_at: None,
            dunning_paused_note: None,
            pdf_storage_path: None,
            pdf_sha256: None,
            pdf_frozen_at: None,
            pdf_language: None,
            version: 1,
            created_at: NaiveDateTime::default(),
            updated_at: NaiveDateTime::default(),
        }
    }

    fn make_contact(id: i64, name: &str) -> Contact {
        Contact {
            id,
            company_id: 100,
            contact_type: kesh_db::entities::contact::ContactType::Entreprise,
            name: name.to_string(),
            first_name: None,
            last_name: None,
            is_client: true,
            is_supplier: false,
            address: None,
            address_street: None,
            address_building: None,
            address_postal_code: None,
            address_city: None,
            address_country: None,
            email: None,
            phone: None,
            ide_number: None,
            client_number: None,
            default_payment_terms: None,
            default_payment_terms_days: None,
            language: None,
            salutation: kesh_db::entities::contact::Salutation::Neutre,
            active: true,
            version: 1,
            created_at: NaiveDateTime::default(),
            updated_at: NaiveDateTime::default(),
        }
    }

    /// Construit un triplet candidat `(facture, contact, montant à régler)`
    /// pour les tests (#246, #420). Ces fixtures n'ont ni lignes TVA ni
    /// règlement → le reste dû vaut le `total_amount` posé directement, ce qui
    /// préserve exactement les assertions de montant existantes.
    fn cand(invoice: Invoice, contact: Option<Contact>) -> (Invoice, Option<Contact>, Decimal) {
        let amount_to_settle = invoice.total_amount;
        (invoice, contact, amount_to_settle)
    }

    /// AC #30 — full match → score 1.0 (0.50 + 0.40 + 0.10).
    #[test]
    fn score_full_match_returns_1_0() {
        let tx = make_tx(dec!(1234.56), Some("INV-2026-001"), Some("ACME GMBH"));
        let invoice = make_invoice(101, dec!(1234.56), Some("INV-2026-001"), 50);
        let contact = make_contact(50, "ACME GMBH");
        let proposals = propose_matches(&tx, &[cand(invoice, Some(contact))]);
        assert_eq!(proposals.len(), 1);
        assert!((proposals[0].score.total - 1.0).abs() < 1e-10);
        assert!((proposals[0].score.amount_score - 1.0).abs() < 1e-10);
        assert!((proposals[0].score.reference_score - 1.0).abs() < 1e-10);
        assert!((proposals[0].score.contact_score - 1.0).abs() < 1e-10);
    }

    /// AC #31 — amount only → score 0.50.
    #[test]
    fn score_amount_only_returns_0_50() {
        let tx = make_tx(dec!(200.00), Some("RANDOM-REF"), Some("UNKNOWN COMPANY"));
        let invoice = make_invoice(101, dec!(200.00), Some("OTHER-NUM"), 50);
        let contact = make_contact(50, "Different Co");
        let proposals = propose_matches(&tx, &[cand(invoice, Some(contact))]);
        assert_eq!(proposals.len(), 1);
        assert!((proposals[0].score.total - 0.50).abs() < 1e-10);
    }

    /// AC #32 — reference common prefix ≥ 4 chars → score 0.5 → 0.20.
    #[test]
    fn score_reference_prefix_match_returns_0_5() {
        let tx = make_tx(dec!(99.00), Some("INV-2026-XYZ"), None);
        let invoice = make_invoice(101, dec!(100.00), Some("INV-2026-001"), 50);
        let proposals = propose_matches(&tx, &[cand(invoice, None)]);
        assert_eq!(proposals.len(), 1);
        // amount mismatch (0.0) + reference 0.5 (preffix 8 chars "inv-2026") + contact 0 = 0.20.
        assert!((proposals[0].score.reference_score - 0.5).abs() < 1e-10);
        assert!((proposals[0].score.total - 0.20).abs() < 1e-10);
    }

    /// AC #33 — fallback `coalesce(reference, end_to_end_id, transaction_id)`.
    #[test]
    fn score_reference_falls_back_to_end_to_end_id() {
        let mut tx = make_tx(dec!(150.00), None, None);
        tx.end_to_end_id = Some("INV-2026-042".to_string());
        let invoice = make_invoice(101, dec!(150.00), Some("INV-2026-042"), 50);
        let proposals = propose_matches(&tx, &[cand(invoice, None)]);
        assert_eq!(proposals.len(), 1);
        assert!((proposals[0].score.reference_score - 1.0).abs() < 1e-10);
    }

    /// AC #34 + L28 — contact substring bidirectionnel.
    #[test]
    fn score_contact_substring_match() {
        // tx counterparty plus long que contact.name (CAMT.053 typique).
        let tx = make_tx(dec!(100.00), None, Some("ACME GMBH BERLIN"));
        let invoice = make_invoice(101, dec!(100.00), None, 50);
        let contact = make_contact(50, "ACME GMBH");
        let proposals = propose_matches(&tx, &[cand(invoice, Some(contact))]);
        // amount 0.50 + ref 0.0 + contact 1.0 (bidirectional: "acme gmbh" ⊂ "acme gmbh berlin").
        assert_eq!(proposals.len(), 1);
        assert!((proposals[0].score.contact_score - 1.0).abs() < 1e-10);
        assert!((proposals[0].score.total - 0.60).abs() < 1e-10);
    }

    /// AC #35 — score 0.0 → filtré out.
    #[test]
    fn score_zero_filters_out_proposal() {
        let tx = make_tx(dec!(100.00), Some("ABC"), Some("Random"));
        let invoice = make_invoice(101, dec!(200.00), Some("XYZ"), 50);
        let contact = make_contact(50, "Different");
        let proposals = propose_matches(&tx, &[cand(invoice, Some(contact))]);
        assert!(proposals.is_empty());
    }

    /// AC #36 — amount mismatch within repo window but score=0 amount,
    /// reference match → partial score.
    #[test]
    fn score_amount_mismatch_within_repo_window_returns_partial() {
        let tx = make_tx(dec!(100.00), Some("INV-A"), None);
        let invoice = make_invoice(101, dec!(100.03), Some("INV-A"), 50);
        let proposals = propose_matches(&tx, &[cand(invoice, None)]);
        assert_eq!(proposals.len(), 1);
        assert!((proposals[0].score.amount_score - 0.0).abs() < 1e-10);
        assert!((proposals[0].score.reference_score - 1.0).abs() < 1e-10);
        // 0.0 + 0.40 + 0.0 = 0.40
        assert!((proposals[0].score.total - 0.40).abs() < 1e-10);
    }

    /// Tri par score DESC, puis, à score égal, l'ordre d'imputation (Story
    /// 29-5) — qui se réduit ici à l'identifiant : `make_invoice` donne même
    /// date et aucune échéance à 101 et 103.
    #[test]
    fn propose_matches_returns_sorted_desc() {
        let tx = make_tx(dec!(100.00), Some("INV-2026-001"), Some("ACME"));
        let inv1 = make_invoice(101, dec!(100.00), Some("OTHER"), 50);
        let inv2 = make_invoice(102, dec!(100.00), Some("INV-2026-001"), 51);
        let inv3 = make_invoice(103, dec!(100.00), None, 52);
        let proposals =
            propose_matches(&tx, &[cand(inv1, None), cand(inv2, None), cand(inv3, None)]);
        assert_eq!(proposals.len(), 3);
        // inv2 amount + ref = 0.90, inv1 amount only = 0.50, inv3 amount only = 0.50.
        // Tri: 102 (0.90), 101 (0.50), 103 (0.50) — à score égal, ordre
        // d'imputation, ici l'id (mêmes dates, aucune échéance).
        assert_eq!(proposals[0].invoice_id, 102);
        assert_eq!(proposals[1].invoice_id, 101);
        assert_eq!(proposals[2].invoice_id, 103);
    }

    /// Empty candidates → empty proposals.
    #[test]
    fn propose_matches_empty_candidates_returns_empty() {
        let tx = make_tx(dec!(100.00), Some("REF"), Some("ACME"));
        let proposals = propose_matches(&tx, &[]);
        assert!(proposals.is_empty());
    }

    /// E11 (Story 8-3 pattern reused) — Decimal::ZERO scale invariant.
    #[test]
    fn score_handles_zero_amount_scale_invariant() {
        let tx = make_tx(Decimal::new(0, 0), Some("REF"), None);
        let invoice = make_invoice(101, Decimal::new(0, 4), Some("REF"), 50);
        let proposals = propose_matches(&tx, &[cand(invoice, None)]);
        assert_eq!(proposals.len(), 1);
        // amount 0.0 vs 0.0 (post-normalize) → 1.0 ; ref 1.0 → total 0.90.
        assert!((proposals[0].score.amount_score - 1.0).abs() < 1e-10);
    }

    /// AC #63 perf smoke — 1000 tx × 500 candidates < 200ms (HP4-3 patch).
    /// Note : on appelle `propose_matches` 1× avec 500 candidates per tx,
    /// 1000 itérations → 500 000 score computations.
    #[test]
    fn propose_matches_handles_1000_x_500_under_200ms() {
        let tx = make_tx(dec!(100.00), Some("REF-X"), Some("ACME"));
        let candidates: Vec<(Invoice, Option<Contact>, Decimal)> = (0..500)
            .map(|i| {
                cand(
                    make_invoice(i, dec!(100.00), Some(&format!("INV-{i}")), i),
                    None,
                )
            })
            .collect();

        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = propose_matches(&tx, &candidates);
        }
        let elapsed = start.elapsed();
        eprintln!("propose_matches 1000×500 = {elapsed:?}");
        // Smoke non-bloquant : warning si > 200ms (cf. AC #63 + A3-8 patch).
        if elapsed.as_millis() > 200 {
            eprintln!(
                "[L10-style perf warning] propose_matches_handles_1000_x_500: {elapsed:?} > 200ms"
            );
        }
        // M8 Pass 1 — upper bound large pour catch les régressions
        // catastrophiques (e.g. introduction d'I/O dans le helper, boucle
        // accidentelle O(N²) sur les chars). 5s = ~25× le seuil cible
        // 200ms, garde la marge pour l'overhead CI variable.
        assert!(
            elapsed < std::time::Duration::from_secs(5),
            "matching perf catastrophic regression: {elapsed:?}"
        );
    }

    /// L28 / H4 — fallback chain transaction_id si reference + eid null.
    #[test]
    fn score_reference_falls_back_to_transaction_id() {
        let mut tx = make_tx(dec!(150.00), None, None);
        tx.transaction_id = Some("TID-2026-99".to_string());
        let invoice = make_invoice(101, dec!(150.00), Some("TID-2026-99"), 50);
        let proposals = propose_matches(&tx, &[cand(invoice, None)]);
        assert_eq!(proposals.len(), 1);
        assert!((proposals[0].score.reference_score - 1.0).abs() < 1e-10);
    }

    /// L31 — contact = None (archivé/supprimé) → contact_score = 0.0.
    #[test]
    fn score_contact_none_returns_zero() {
        let tx = make_tx(dec!(100.00), Some("INV-A"), Some("ACME"));
        let invoice = make_invoice(101, dec!(100.00), Some("INV-A"), 50);
        let proposals = propose_matches(&tx, &[cand(invoice, None)]);
        assert_eq!(proposals.len(), 1);
        assert!((proposals[0].score.contact_score - 0.0).abs() < 1e-10);
        // amount 0.50 + ref 0.40 = 0.90.
        assert!((proposals[0].score.total - 0.90).abs() < 1e-10);
    }

    // -----------------------------------------------------------------------
    // Story 29-5 (#668) — à score égal, l'ordre d'imputation (art. 87 CO)
    // -----------------------------------------------------------------------

    /// Une date de 2026.
    fn d(m: u32, j: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, m, j).unwrap()
    }

    /// Une facture de 100.00 du client 50, datée `f`, d'échéance `e`, sans
    /// numéro (aucune référence ne la désigne, aucun préfixe commun).
    fn dated(id: i64, f: NaiveDate, e: Option<NaiveDate>) -> Invoice {
        let mut invoice = make_invoice(id, dec!(100.00), None, 50);
        invoice.date = f;
        invoice.due_date = e;
        invoice
    }

    /// Les identifiants proposés, dans l'ordre, pour une transaction de 100.00
    /// de « ACME » sans référence : chaque facture vaut 0,60 (montant exact et
    /// contact), le départage seul décide.
    fn tie_order(invoices: Vec<Invoice>) -> Vec<i64> {
        let tx = make_tx(dec!(100.00), None, Some("ACME"));
        let candidates: Vec<_> = invoices
            .into_iter()
            .map(|i| cand(i, Some(make_contact(50, "ACME"))))
            .collect();
        let proposals = propose_matches(&tx, &candidates);
        for p in &proposals {
            assert!((p.score.total - 0.60).abs() < 1e-10, "score égal : 0,60");
        }
        proposals.iter().map(|p| p.invoice_id).collect()
    }

    /// AC5 cas 1 — le cas de l'issue (ligne 1 du tableau) : A non échue, créée
    /// d'abord ; B échue le jour du paiement. B en tête.
    #[test]
    fn ties_put_the_due_invoice_before_a_not_yet_due_older_one() {
        let a = dated(101, d(5, 26), Some(d(7, 25)));
        let b = dated(102, d(6, 10), Some(d(6, 15)));
        assert_eq!(tie_order(vec![a, b]), vec![102, 101]);
    }

    /// AC5 cas 2 — ligne 2 : B, plus récente, est échue la première.
    #[test]
    fn ties_put_the_first_fallen_due_before_an_older_invoice() {
        let a = dated(101, d(5, 20), Some(d(6, 14)));
        let b = dated(102, d(5, 25), Some(d(6, 5)));
        assert_eq!(tie_order(vec![a, b]), vec![102, 101]);
    }

    /// AC5 cas 3 — ligne 3 : A sans échéance est exigible dès sa date, avant
    /// B à échéance plus tardive. B, à écarter, porte le plus petit `id`.
    #[test]
    fn ties_treat_a_missing_due_date_as_due_on_the_invoice_date() {
        let b = dated(101, d(5, 25), Some(d(6, 20)));
        let a = dated(102, d(5, 20), None);
        assert_eq!(tie_order(vec![b, a]), vec![102, 101]);
    }

    /// AC5 cas 4 — ligne 4 : même échéance, la plus ancienne (al. 2,
    /// C-29-5-5). B, la plus récente, porte le plus petit `id` (mutation M6).
    #[test]
    fn ties_on_the_same_due_date_keep_the_oldest_invoice() {
        let b = dated(101, d(5, 25), Some(d(6, 10)));
        let a = dated(102, d(5, 20), Some(d(6, 10)));
        assert_eq!(tie_order(vec![b, a]), vec![102, 101]);
    }

    /// AC5 cas 5 — ligne 5 : une échéance antérieure à la date ne compte pas
    /// (`X = F`, mutation M4) ; B sans échéance, plus ancienne, en tête.
    #[test]
    fn ties_ignore_a_due_date_before_the_invoice_date() {
        let a = dated(101, d(5, 20), Some(d(5, 10)));
        let b = dated(102, d(5, 15), None);
        assert_eq!(tie_order(vec![a, b]), vec![102, 101]);
    }

    /// AC5 cas 6 — ligne 6 : A sans échéance a ici la **plus grande** `X` ;
    /// une comparaison d'`Option` (`None` en tête) la mettrait devant
    /// (mutation M3).
    #[test]
    fn ties_put_an_earlier_due_date_before_a_later_invoice_without_due_date() {
        let a = dated(101, d(6, 1), None);
        let b = dated(102, d(5, 20), Some(d(5, 25)));
        assert_eq!(tie_order(vec![a, b]), vec![102, 101]);
    }

    /// AC5 cas 7 — mêmes dates : l'identifiant départage. Entrée 102 puis
    /// 101 : `sort_by` étant stable, une clé sans `id` garderait 102 en tête
    /// (mutation M13).
    #[test]
    fn ties_on_same_dates_fall_back_to_invoice_id() {
        let x = dated(102, d(5, 20), Some(d(6, 19)));
        let y = dated(101, d(5, 20), Some(d(6, 19)));
        assert_eq!(tie_order(vec![x, y]), vec![101, 102]);
    }

    /// AC5 cas 8 — le score reste la première clé (mutation M7). Toutes les
    /// factures sont du client « ACME », contrepartie de la transaction
    /// (contact 0,10 chacune) ; les factures non désignées n'ont pas de
    /// numéro, donc aucune demi-référence.
    #[test]
    fn score_still_beats_exigibility() {
        let acme = || Some(make_contact(50, "ACME"));
        let score = |p: &MatchProposal| p.score.total;

        // (a) Désignée au montant exact, non échue (1,0) devant une échue non
        // désignée (0,60).
        let tx = make_tx(dec!(100.00), Some("INV-2026-001"), Some("ACME"));
        let due = dated(101, d(5, 1), Some(d(5, 20)));
        let mut designated = dated(102, d(5, 10), Some(d(7, 30)));
        designated.invoice_number = Some("INV-2026-001".into());
        let p = propose_matches(&tx, &[cand(due, acme()), cand(designated, acme())]);
        assert_eq!(
            p.iter().map(|p| p.invoice_id).collect::<Vec<_>>(),
            [102, 101]
        );
        assert!((score(&p[0]) - 1.0).abs() < 1e-10);
        assert!((score(&p[1]) - 0.60).abs() < 1e-10);

        // (b) Sans référence : au montant exact, non échue (0,60) devant une
        // échue à 3 centimes près (0,10, contact seul).
        let tx = make_tx(dec!(100.00), None, Some("ACME"));
        let mut off = dated(101, d(5, 1), Some(d(5, 20)));
        off.total_amount = dec!(100.03);
        let exact = dated(102, d(5, 10), Some(d(7, 30)));
        let p = propose_matches(&tx, &[cand(off, acme()), cand(exact, acme())]);
        assert_eq!(
            p.iter().map(|p| p.invoice_id).collect::<Vec<_>>(),
            [102, 101]
        );
        assert!((score(&p[0]) - 0.60).abs() < 1e-10);
        assert!((score(&p[1]) - 0.10).abs() < 1e-10);

        // (c) La condition de montant de la désignation (AC7) : désignée par
        // le numéro mais à 3 centimes près, échue (0,50), derrière une facture
        // non désignée au montant exact, non échue (0,60).
        let tx = make_tx(dec!(100.00), Some("INV-2026-001"), Some("ACME"));
        let mut designated_off = dated(101, d(5, 1), Some(d(5, 20)));
        designated_off.invoice_number = Some("INV-2026-001".into());
        designated_off.total_amount = dec!(100.03);
        let exact = dated(102, d(5, 10), Some(d(7, 30)));
        let p = propose_matches(&tx, &[cand(designated_off, acme()), cand(exact, acme())]);
        assert_eq!(
            p.iter().map(|p| p.invoice_id).collect::<Vec<_>>(),
            [102, 101]
        );
        assert!((score(&p[0]) - 0.60).abs() < 1e-10);
        assert!((score(&p[1]) - 0.50).abs() < 1e-10);
    }
}
