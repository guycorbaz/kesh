# Prompt — revue de code P2, Story 25-6-b (le PDF d'une facture est figé)

*Versionné le 2026-10-05. Deux lentilles (Opus), contexte frais chacune. Rotation : P1 Sonnet ×3 → P2 Opus.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-6-b-pdf-facture-archive`. Fiche :
`_bmad-output/implementation-artifacts/25-6-b-pdf-facture-archive.md` — lire le **Change Log** (entrée « Revue de code
P1 ») pour savoir ce qui a été corrigé. Règles : `CLAUDE.md`. Issues : `gh issue view 387`.

La remédiation de P1 touche trois modules (`routes/invoice_email.rs`, `inbox_import.rs`, la fiche facture Svelte) :
la passe reprend donc le périmètre complet, avec une lentille braquée sur la remédiation.

## Lentilles

- **R — Regression hunter** : **`git show 8ac0a4de` seul**. Chaque correctif de P1 a-t-il introduit un défaut ?
  - La garde de statut ajoutée à `send_invoice_email` : l'ordre des refus a-t-il changé pour un appelant qui
    l'observait (tests `invoice_send_email_e2e`, frontend `SendEmailDialog`, lot d'envois) ? un statut légitime
    est-il désormais refusé ?
  - Le nettoyage de l'inbox retiré : que devient un fichier orphelin ? un ré-import du même fichier ? le test ajouté
    prouve-t-il ce qu'il dit (le nom qu'il écrit est-il exactement celui que l'import aurait supprimé) ?
  - La fiche : la nouvelle branche du `catch`, `pdfGone` remis à faux, la condition `validated` du bouton, la clé
    `invoice-pdf-refreeze-error` — le mock Vitest qui honore un catalogue prouve-t-il le défaut M1 tel qu'il se
    produirait avec le vrai `i18nMsg` (`frontend/src/lib/shared/utils/i18n.svelte.ts`) ?
  - **Propagation** : le symptôme M1 (message à argument Fluent tiré du catalogue côté frontend) existe-t-il
    ailleurs ? Pars du symptôme : `grep -rn "{ \$" crates/kesh-i18n/locales/fr-CH/messages.ftl | grep "^.*error-"`
    puis cherche si ces codes d'erreur sont résolus par `i18nMsg(clé, err.message)` dans `frontend/src`.
- **F — Full-scope adversary** : `git diff main 8ac0a4de`, avec la fiche. Ce que P1 n'a **pas** exercé, d'après ses
  rapports :
  - la restauration `.keshbackup` (`crates/kesh-api/src/routes/admin/` ou `backup*`, `post_restore.rs`,
    `check_schema_compat`) : une sauvegarde **antérieure** à la migration s'importe-t-elle ? une sauvegarde
    **postérieure** restaure-t-elle les quatre colonnes ? une facture restaurée figée sans son fichier répond-elle
    bien 410 ?
  - `HEAD /api/v1/invoices/{id}/pdf` : fige-t-il ? est-ce acceptable et écrit ?
  - `KESH_DOCUMENTS_DIR` relatif ou lien symbolique (l'inbox canonicalise le sien — et le service de gel ?) ;
  - les clés d'API : le gel par une clé `read` est-il tracé au nom de la clé (`actor_api_key_id`) ? testé ?
  - les trois `.ftl` DE/IT/EN : clés, variables `{ $sha256 }` présentes partout où le FR les a ;
  - les corps des tests Rust `crates/kesh-api/tests/invoice_frozen_pdf_e2e.rs` et
    `crates/kesh-db/tests/invoice_frozen_pdf.rs` : un test qui passerait **à vide** (assertion vraie par
    construction, montage qui ne produit pas l'état annoncé) ?
  - le **manuel** (`docs/manual/fr/*.tex` **et PDF aplatis**, `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '` vers
    `target/gate-logs/`) contre le code : chaque promesse est-elle tenue ? chaque flux ouvert est-il dit ?

## Ce que tu rends

Findings avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code cité relu),
scénario d'échec, correction proposée. Pour tout finding affirmant qu'un code est absent ou présent : la sortie d'un
`grep -nF`. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une base :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`, `sqlx`,
`cargo test`/`nextest`/`build`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`,
`sed -n`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers `target/gate-logs/`.
