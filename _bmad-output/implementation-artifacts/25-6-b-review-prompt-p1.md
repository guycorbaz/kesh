# Prompt — revue de code P1, Story 25-6-b (le PDF d'une facture est figé)

*Versionné le 2026-10-05. Trois lentilles (Sonnet), contexte frais chacune.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-6-b-pdf-facture-archive`. **Diff à revoir : `git diff main 9d842984`**
(migration `51d1696f`, service et tests `bd753fdc`, écran/export/manuels `9d842984`). Fiche :
`_bmad-output/implementation-artifacts/25-6-b-pdf-facture-archive.md` (arbitrages 1–8, AC 1 à 8, Dev Notes, Dev Agent
Record). Issues : `gh issue view 387`, `502`, `503`. Règles : `CLAUDE.md`.

## Lentilles

- **A — Blind hunter** : le diff **seul**, sans la fiche.
  - `crates/kesh-api/src/routes/issued_invoice_pdf.rs` : chaque branche de `get_or_freeze`, `pose`, `refreeze`,
    `with_one_retry` — erreurs propagées ou avalées, `spawn_blocking` (panique, annulation), ordre écriture du
    fichier → pose → audit, ce qui est servi dans chaque cas.
  - `crates/kesh-db/src/repositories/invoices.rs` : `freeze_pdf`, `refreeze_pdf` (liaisons dans l'ordre — compte-les
    contre les `?`), transaction et rollback, `unvalidate` (l'`UPDATE` unique), la macro `invoice_columns!` et
    `INVOICE_COLUMNS` (ordre identique aux champs de `Invoice` ? `FromRow` lit par nom ou par position ?).
  - `errors.rs` : les six variantes, statuts HTTP, messages et arguments Fluent.
  - Svelte 5 : `+page.svelte` de la fiche facture — snippet, `$derived`, état `pdfGone` (quand se remet-il à faux ?).
- **B — Edge-case hunter** : le diff et le code environnant.
  - **Courses et gardes**, refaites à la main : deux premiers téléchargements simultanés ; un envoi e-mail pendant un
    téléchargement ; une dévalidation, un avoir, un règlement, une pause des rappels entre rendu et pose ; deux
    refigeages ; un refigeage pendant un avoir. Pour chacun : qu'est-ce qui est figé, servi, tracé ?
  - **Tous les chemins qui rendent un PDF de facture** — pars du symptôme :
    `grep -rn "render_document\|PdfDocument::Invoice\|generate_qr_bill_pdf" crates/`. Un chemin contourne-t-il le
    gel (lot d'envois, aperçu, rappel, export, test endpoint) ?
  - **Tous les chemins qui changent `status` ou les lignes d'une facture** :
    `grep -rn "UPDATE invoices" crates/kesh-db/src` — chacun est-il compatible avec un PDF figé (détache-t-il quand
    il le faut, bumpe-t-il `version` quand les données rendues changent) ?
  - Les fichiers : `KESH_DOCUMENTS_DIR` relatif, lien symbolique, fichier vide, `storage_path` corrompu en base,
    `HEAD` sur la route `GET`. La restauration `.keshbackup` (`post_restore.rs`, `check_schema_compat`) et l'export
    CSV : une sauvegarde antérieure s'importe-t-elle ? la colonne écartée l'est-elle à bon droit ?
  - Les tests : déterministes ? chaque mutation déclarée au Dev Agent Record est-elle tuée par l'assertion écrite, et
    non par un effet de bord ? le E2E Playwright crée-t-il sa propre facture ?
- **C — Acceptance auditor** : chaque AC (1, 1-bis, 2, 3, 3-bis, 4, 5, 5-bis, 6, 7, 8) tenu, point par point ?
  - Recompte depuis la source : bloc admin (`ADMIN_COUPLES`, `lib.rs` « exactement 28 »), registre d'audit
    (110 / 92 / 113), `sitesTotal` 1767 (`frontend/src/lib/shared/i18n-keys.test.ts`), parité des clés neuves dans les
    4 `.ftl`, compteurs de `docs/migrations-idempotence-audit.md` et de `migrations_upgrade_path.rs`, décomptes de
    tests du Dev Agent Record (`main` → `9d842984`).
  - **Le manuel** : `docs/manual/fr/user-manual.tex` et `admin-manual.tex`, **et leurs PDF aplatis**
    (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '` vers `target/gate-logs/`). Chaque site nommé par l'AC 8 est-il
    traité ? Le manuel promet-il quelque chose que le code ne fait pas, ou omet-il un flux que le code ouvre (un
    écran, une route, une clé d'API) ? Pars du symptôme :
    `grep -nE "PDF|figé|refig" docs/manual/fr/*.tex`.
  - Le CHANGELOG `[0.12.1]`, le README, `docs/testing.md`.
  - Les écarts déclarés au Dev Agent Record (bloc admin 27 → 28 au lieu de 25 → 26 ; garde d'export non anticipée ;
    E2E complet non relancé après la correction du montage de la spec) sont-ils exacts et acceptables ?

## Ce que tu rends

Findings avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code cité relu),
scénario d'échec, correction proposée. Pour tout finding affirmant qu'un code est absent ou présent : la sortie d'un
`grep -nF`. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une base :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`, `sqlx`,
`cargo test`/`nextest`/`build`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`,
`sed -n`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers `target/gate-logs/`.
