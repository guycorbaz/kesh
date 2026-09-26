# Prompt — passe 3 CIBLÉE de `bmad-code-review`, Story 25-5-a (revue a posteriori)

*Versionné le 2026-09-26. **Une seule lentille** (Opus), contexte frais, braquée sur **le seul
commit de la remédiation P2** — CLAUDE.md, § « La passe ciblée ». Cycle Sonnet → Haiku → Opus.*

Worktree `/home/gcorbaz/devel/kesh-wt-255a`, branche `story/25-5-a-revue-code`, tête `b0399adf`.
⚠️ Lis les fichiers **dans ce worktree**. Diff de la remédiation :
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/ca5ce2e2-67a3-4eeb-817f-c2de35620a1c/scratchpad/25-5-a-remediation-p2.diff`.
Fiche : `_bmad-output/implementation-artifacts/25-5-a-export-souverainete.md` (entrées review P1 et
P2 du Change Log, section Review Findings).

**Ce que la P2 a fait** : ajouté des colonnes à quatre sérialiseurs de `crates/kesh-api/src/exports/csv_tables.rs`
(`company`, `contacts`, `bank_accounts`, `company_invoice_settings`) ; posé une garde de colonnes
`chaque_colonne_du_schema_est_exportee_ou_ecartee`, qui lit `crates/kesh-db/test-schema/0001_schema_squash.sql`
par `include_str!`, avec des exemptions motivées (`COLONNES_HORS_EXPORT`) ; ouvert #466 ; CHANGELOG.
Reportés, **à ne pas re-signaler** : #465, #466.

## La lentille — chasseur de régressions du dernier patch

1. **Les colonnes ajoutées** : chaque en-tête est-il dans le **même ordre** que ses valeurs ? Les
   valeurs sont-elles formatées comme leurs voisines (booléens, énumérations, `Option`) ? Une
   cellule de texte libre échappe-t-elle à `txt` / `fmt_opt_str` ? L'ordre choisi casse-t-il un test
   ou un consommateur qui lit une colonne **par position** (grep des tests E2E et unitaires qui
   comparent un en-tête ou indexent une colonne de `contacts.csv`, `company.csv`,
   `bank_accounts.csv`, `company_invoice_settings.csv`) ?
2. **La garde** : peut-elle passer **à vide** ? — une table introuvable, un bloc mal découpé
   (colonne dont la ligne ne commence pas par une backquote, `PRIMARY KEY`, `CONSTRAINT`, clé
   générée sur plusieurs lignes), un en-tête lu depuis le mauvais octet (BOM), l'exemption
   `_json` trop large (une colonne `x` couverte par un en-tête `x_json` qui n'est pas elle). Les
   trente entrées correspondent-elles aux trente de `TABLES_EXPORTEES` (`exports/global.rs`) — et
   si une table y entre demain, la garde le sait-elle ? Le chemin `include_str!` est-il robuste ?
3. **Les motifs d'exemption** sont-ils **vrais** ? Vérifie chacun dans le code (qui écrit, qui lit
   la colonne). La P2 a déjà écrit un motif faux sur `country` et l'a corrigé : cherche le suivant.
4. **Le CHANGELOG** : l'entrée ajoutée dit-elle vrai, colonne par colonne ?

## Ce que tu rends

- Findings : sévérité, `fichier:ligne`, **preuve** (code lu, commande et résultat), correction.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier, n'exécute aucune commande qui écrit dans le dépôt, le worktree ou une base
— `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, tout `git commit`/`push`/`add`/`stash`/
`reset`/`rebase`/`checkout`/`switch`/`worktree`, `sqlx migrate`, `cargo test`/`cargo nextest`,
`npm run`, `npx playwright`. Lecture, `grep`, `git log`/`show`/`diff` et `cargo check` sont autorisés.
