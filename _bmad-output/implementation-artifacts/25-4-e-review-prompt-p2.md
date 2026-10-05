# Prompt — revue de code P2 ciblée, Story 25-4-e

*Versionné le 2026-10-01. **Une lentille** (Haiku), contexte frais, **passe ciblée** sur la seule
remédiation P1 : `git show 2b95bc7f` (un seul commit, diff aplati).*

Dépôt `/home/gcorbaz/devel/kesh`. Fiche : `_bmad-output/implementation-artifacts/25-4-e-montant-minimum-facture.md`.

## Axes — tous obligatoires

1. **La comparaison au centime** (`crates/kesh-db/src/repositories/invoices.rs`, bloc « 2 bis'' ») : utilise-t-elle
   la même stratégie d'arrondi que le reste du dépôt (`Money::round_to_centimes`) ? Le message (`crates/kesh-api/src/errors.rs`,
   `InvoiceBelowMinimum`) affiche-t-il maintenant exactement la valeur comparée ? Un cas peut-il encore afficher
   deux montants égaux ?
2. **Le plafond** (`crates/kesh-api/src/routes/company_invoice_settings.rs`) : `MAX_UNIT_PRICE` est-il la bonne
   référence (`crates/kesh-api/src/routes/limits.rs`) ? Le message d'erreur le dit-il ?
3. **L'export** (`crates/kesh-api/src/exports/csv_tables.rs`) : `grep -nF "fmt_opt_decimal(cis.minimum_invoice_amount)"`.
4. **Les tests ajoutés** : `the_minimum_compares_at_the_centime` et le cas `1000000000.01` — prouvent-ils ce qu'ils
   disent ? Rougiraient-ils si le correctif était retiré ?

## Ce que tu rends

Findings avec sévérité, `fichier:ligne`, **preuve** (sortie de `grep -nF` / `sed -n`), correction. ⛔ La liste des
axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande mutante (`scripts/*` dont `scripts/prepare-release.sh`,
`make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`, `cargo test`/`nextest`, `npm run`, `npx`,
`gh issue create`/`comment`/`edit`). Autorisés : lecture, `grep`, `sed -n`, `git show`/`diff`/`log`, `cargo check`.
