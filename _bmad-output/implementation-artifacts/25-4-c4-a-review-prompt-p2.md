# Prompt — revue de code P2 ciblée, Story 25-4-c4-a

*Versionné le 2026-10-01. **Une lentille** (Haiku), contexte frais, **passe ciblée** sur la seule
remédiation P1 : `git show 34dd5dbf` (un seul commit, diff aplati).*

Dépôt `/home/gcorbaz/devel/kesh`. Fiche : `_bmad-output/implementation-artifacts/25-4-c4-a-arrondi-fige-a-la-validation.md`
(Change Log, entrée « Revue de code P1 »).

## Axes — tous obligatoires

1. **La forme jointe du montant crédité** (`crates/kesh-db/src/repositories/invoice_settlements.rs`,
   `INVOICE_CREDITED_DERIVED_JOIN_SQL`) : le `LEFT JOIN` + `COALESCE` rend-il exactement la forme scalaire
   (`INVOICE_CREDITED_SUBQUERY_SQL`) dans tous les cas — avoir avec lignes, sans ligne, `draft` vs `issued`,
   plusieurs avoirs ? La somme par facture peut-elle compter un arrondi deux fois ?
2. **Le paragraphe du manuel** (`docs/manual/fr/admin-manual.tex`, *Compte de différences d'arrondi*, et le
   PDF aplati : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers
   `/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/`) : chaque
   affirmation est-elle vraie contre le code (`crates/kesh-db/src/repositories/invoices.rs`, `validate_invoice` ;
   `credit_notes.rs`, `create_credit_note`) ? Le paragraphe suivant (paiement au centime) le contredit-il ?
3. **Les deux tests ajoutés** (`crates/kesh-db/tests/invoices_validate_vat.rs`,
   `a_credit_note_is_refused_when_the_rounding_account_was_archived` ; `crates/kesh-api/tests/admin_full_import_e2e.rs`,
   `seed_rounding_state` et ses deux appelants) : prouvent-ils ce qu'ils disent ? L'avoir brut inséré
   (`status = 'draft'`) respecte-t-il les contraintes de la table (`grep -n "credit_notes" -A30
   crates/kesh-db/test-schema/0001_schema_squash.sql`) ? L'assertion « numéro non consommé » est-elle juste
   (où le numéro est-il tiré, dans la même transaction ?) ?
4. **Le commentaire de l'ordre des verrous** (`validate_invoice`) : exact contre le code ?

## Ce que tu rends

Findings avec sévérité, `fichier:ligne`, **preuve** (sortie de `grep -nF` / `sed -n`, ou code cité relu),
correction. ⛔ La liste des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande mutante (`scripts/*` dont `scripts/prepare-release.sh`,
`make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`, `cargo test`/`nextest`, `npm run`, `npx`,
`gh issue create`/`comment`/`edit`). Autorisés : lecture, `grep`, `sed -n`, `git show`/`diff`/`log`,
`cargo check`, `pdftotext` vers le scratchpad.
