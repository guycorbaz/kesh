# Prompt — revue de code P2 ciblée, Story 25-4-c4-b

*Versionné le 2026-10-01. **Une lentille** (Haiku), contexte frais, **passe ciblée** sur la seule
remédiation P1 : `git show 44925eea` (un seul commit, diff aplati).*

Dépôt `/home/gcorbaz/devel/kesh`. Fiche : `_bmad-output/implementation-artifacts/25-4-c4-b-arrondi-visible-et-reglage.md`
(Change Log, entrée « Revue de code P1 »).

## Axes — tous obligatoires

1. **La lecture pure du réglage** (`crates/kesh-db/src/repositories/company_invoice_settings.rs`,
   `round_to_5_centimes`) : le défaut `true` quand la ligne manque est-il le même que celui de la colonne
   (`grep -n "round_to_5_centimes" crates/kesh-db/migrations/20261001000001_invoice_rounding_5_centimes.sql`) ?
   Reste-t-il un appel à `get_or_create_default` dans un chemin de LECTURE de facture
   (`grep -rn "get_or_create_default" crates/kesh-api/src`) ?
2. **L'arrondi décidé au centime** — PDF (`crates/kesh-qrbill/src/pdf.rs`, `recap_lines`) et fiche
   (`frontend/src/routes/(app)/invoices/[id]/+page.svelte`, `roundingCents`) : les deux arrondissent-ils de la
   même façon (big.js `roundHalfUp` = loin de zéro ; Rust `MidpointAwayFromZero`) ? Le total estimé de la fiche
   (`displayedTotal`) utilise-t-il la valeur brute ou arrondie, et est-ce cohérent avec ce qui s'affiche ?
3. **Le manuel** (`docs/manual/fr/user-manual.tex`, paragraphe *Le total arrondi à 5 centimes*, et le PDF aplati
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers
   `/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/`) : chaque phrase
   est-elle vraie contre le code (`crates/kesh-qrbill/src/pdf.rs`, `draw_payment_part`) ?
4. **Les tests ajoutés** : prouvent-ils ce qu'ils disent — `reading_a_draft_writes_nothing`
   (`crates/kesh-api/tests/invoice_echeancier_e2e.rs`), `credit_note_pdf_carries_its_rounding`,
   `invoice_and_reminder_pdf_carry_the_frozen_rounding`, le cas infra-centime de `recap_lines_show_the_rounding_line`
   et le Vitest infra-centime ? Chacun rougirait-il si le correctif était retiré ?

## Ce que tu rends

Findings avec sévérité, `fichier:ligne`, **preuve** (sortie de `grep -nF` / `sed -n`, ou code cité relu),
correction. ⛔ La liste des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande mutante (`scripts/*` dont `scripts/prepare-release.sh`,
`make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`, `cargo test`/`nextest`, `npm run`, `npx`,
`gh issue create`/`comment`/`edit`). Autorisés : lecture, `grep`, `sed -n`, `git show`/`diff`/`log`,
`cargo check`, `pdftotext` vers le scratchpad.
