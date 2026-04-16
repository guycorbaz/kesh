# Known Failures (KF)

Registre des tests/comportements **constatés cassés mais hors scope** du travail en cours, à ne pas oublier de corriger.

## Convention

- **Une entrée par défaillance** identifiée par un ID `KF-NNN`.
- Format : symptôme reproductible, root cause hypothétique, story d'origine, story de remédiation prévue, status (`open` / `closed`).
- Une KF résolue passe à `status: closed` + lien vers le commit fix.
- Pattern process : tout commit qui constate un échec hors scope doit mentionner la KF dans son message.
- Migration prévue vers GitHub Issues (cf. discussion 2026-04-16) — ce fichier reste la source de vérité offline / dans Git.

---

## KF-001 — `invoice_pdf_e2e` : violation `chk_companies_ide_format` + silent fail `validate_invoice`

- **Découvert** : 2026-04-16 (session code review story 5-4 Groupe B)
- **Symptôme** : 10/11 tests de `cargo test -p kesh-api --test invoice_pdf_e2e` paniquent avec :
  ```
  CheckConstraintViolation("CONSTRAINT `chk_companies_ide_format` failed for `_sqlx_test_*`.`companies`")
  ```
- **Root cause effective** (2 problèmes imbriqués) :
  1. La fixture `seed_base` utilisait `ide_number: Some("CHE-123.456.789".into())` — format d'affichage avec séparateurs — alors que la CHECK DB `chk_companies_ide_format` impose la forme canonique `^CHE[0-9]{9}$` (pas de séparateurs, normalisation côté route `contacts.rs`). De surcroît `123456789` ne satisfait pas le checksum mod-11 du `CheNumber`.
  2. Une fois le CHECK corrigé, 8 tests échouaient toujours avec `INVOICE_NOT_VALIDATED` — `seed_validated_invoice` appelait `invoices::validate_invoice` mais swallowait le `Result`. `validate_invoice` exige un `fiscal_year` ouvert + `company_invoice_settings` avec `default_receivable_account_id` et `default_revenue_account_id` non-NULL, aucun desquels n'était seedé.
- **Scope d'origine** : Story 5-3 (Génération PDF QR Bill). Pré-existant — pas une régression introduite par la review story 5-4.
- **Correctif** (commit à suivre) :
  1. `ide_number` → `CHE109322551` (forme canonique, mod-11 valide, aligné avec `kesh-seed`).
  2. `seed_validated_invoice` refondu pour utiliser le pattern SQL bypass (`fiscal_year` + `journal_entry` stub + `UPDATE status='validated'`) aligné avec `invoice_echeancier_e2e::create_validated_invoice_via_sql`. Évite la dépendance à `validate_invoice` qui exige une config comptable complète non pertinente pour tester la route PDF.
- **Validation** : `cargo test -p kesh-api --test invoice_pdf_e2e` → 11/11 ✅
- **Status** : closed 2026-04-16
