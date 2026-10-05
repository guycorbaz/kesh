# Prompt — validation P1, Story 25-6-b (le PDF d'une facture est figé)

*Versionné le 2026-10-03. Trois lentilles (Sonnet), contexte frais chacune.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-6-b-pdf-facture-archive`. **Fiche :**
`_bmad-output/implementation-artifacts/25-6-b-pdf-facture-archive.md` (tête `9048d8e8`). Issue : `gh issue view 387` ; CR
`gh issue view 502`, `503`. Règles : `CLAUDE.md` (en particulier la § *Migration breaking policy*, P1 à P8). La fiche
décrit du travail **à faire**, et ses six **arbitrages** sont rendus par le Project Lead : un finding qui les conteste
doit dire pourquoi ils seraient faux, pas seulement qu'un autre choix existe.

## Lentilles

- **A — Justesse et intégrité.** Le mécanisme garantit-il vraiment « un seul document par facture » ? L'ordre « écrire le
  fichier, puis `UPDATE … WHERE pdf_storage_path IS NULL` » : que se passe-t-il si l'`UPDATE` échoue après l'écriture, si
  deux rendus concurrents écrivent deux fichiers (noms différents, non reproductibles), si le serveur meurt entre les deux ?
  Le perdant d'une course relit-il le bon document ? La vérification d'empreinte à la lecture et le 410 sont-ils
  cohérents avec un fichier partagé ? La langue figée (`pdf_language`) : que devient-elle si le client change de langue
  après le gel ? L'envoi par e-mail d'une facture **déjà figée** dans une autre langue que celle du client actuel ? La
  dévalidation qui détache : un PDF déjà **téléchargé** (et transmis hors Kesh) puis détaché — la fiche le dit-elle ?
  RBAC : qui peut déclencher le gel (un Consultation qui télécharge le premier fige le document — est-ce un problème) ?
- **B — Faisabilité dans le code.** Chaque référence `fichier:ligne` existe-t-elle et dit-elle ce que la fiche affirme
  (`grep -nF`, `sed -n`) ? `document_storage::store_document` est-il appelable depuis le service (synchrone, chemins,
  `DocumentMeta`) ? `invoice_pdf_service::render` exige `validated` : comment le service rend-il une facture `cancelled`
  figée sans le modifier ? Le DTO de la facture (`routes/invoices.rs`) et les `SELECT` de `kesh-db` qui lisent les
  colonnes de `invoices` (inventaire : `grep -rn "FROM invoices" crates/kesh-db/src`) — un `SELECT *` ou un mapping
  `FromRow` casserait-il avec quatre colonnes de plus ? La sauvegarde (`crates/kesh-db/src/backup.rs`, `post_restore.rs`
  `check_schema_compat`) et l'export de souveraineté (`crates/kesh-api/src/exports/global.rs`) traitent-ils les colonnes
  neuves sans changement, ou faut-il les inscrire quelque part ? La migration datée `20261003000001` est-elle compatible
  avec les gardes de `crates/kesh-db/tests/migrations_upgrade_path.rs` ? Le périmètre (quatre modules) est-il exact ?
- **C — Complétude et tests.** Chaque AC est-il testable, et le test prévu aurait-il échoué avant le patch ? Cas
  manquants : facture validée puis contact **supprimé ou archivé** ; société qui change de langue d'instance ; envoi
  e-mail qui **échoue** après le gel (le document est-il figé quand même ?) ; `KESH_DOCUMENTS_DIR` non inscriptible au
  moment du gel ; facture à 0 ligne ou limites de `render`. Les E2E prévus sont-ils réalisables (comparer les octets de
  deux téléchargements dans Playwright) ? Le manuel : relis `docs/manual/fr/user-manual.tex` (section des factures, PDF,
  envoi par e-mail, dévalidation) et `admin-manual.tex` (`KESH_DOCUMENTS_DIR`, sauvegarde) **et les PDF aplatis**
  (`pdftotext … | tr '\n' ' ' | tr -s ' '` vers `target/gate-logs/`) : la fiche nomme-t-elle tous les sites que le patch
  rendra faux ? Pars du symptôme : `grep -rn "régénér\|PDF de la facture\|langue de l'installation\|KESH_DOCUMENTS_DIR" docs`.

## Ce que tu rends

Findings avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), endroit, **preuve** (commande ET sortie, `grep -nF` pour toute
présence ou absence), correction proposée. ⛔ La liste des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une base :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`,
`cargo test`/`nextest`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`, `sed`,
`git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers `target/gate-logs/`.
