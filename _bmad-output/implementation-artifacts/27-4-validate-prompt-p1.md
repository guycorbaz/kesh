# Prompt — validation P1 de la spec, Story 27-4 (lot de réconciliation contenant une transaction disparue)

*Versionné le 2026-10-12. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : **Sonnet** pour les deux
lentilles (la fiche a été écrite par Opus).*

Dépôt `/home/gcorbaz/devel/kesh-27-4`, branche `story/27-4-lot-reconciliation-transaction-disparue` (base
`f9725c29`, = merge de la Story 27-1, #635).
Fiche : `_bmad-output/implementation-artifacts/27-4-lot-reconciliation-transaction-disparue.md` (commit `70e1523d`).
Epic : `_bmad-output/planning-artifacts/epic-27-recette-v0-13-1.md` (section « Amendement du 2026-10-12 »).
Issue #634 (`gh api repos/guycorbaz/kesh/issues/634` et `…/634/comments`) ; contexte : #632 et la fiche
`27-1-supprimer-import-non-rapproche.md` (AC3 (b), AC8, choix C-27-1-17). Registre :
`_bmad-output/implementation-artifacts/epic-27-choix-autonomes.md`, **C-27-4-1 à C-27-4-14**.
Règles : `CLAUDE.md` — § *Pattern batch — FailedProposal per-proposal* (au cœur de la story), § *Review Iteration
Rule* et ses sous-sections (le manuel nommé, les axes déclarés, les sites NON RÉSOLUS, la propagation post-patch, le
recompte des comptes rendus), § *Pendant une boucle de revue* (exception `kesh-db`), § *Issue Tracking Rule*.

## Axes prioritaires (communs aux deux lentilles)

1. **Inventaire des lots et des sites non résolus** (Dev Notes, deux tableaux) : refais les commandes copiées et
   **élargis-les** — un détecteur part du symptôme, pas d'une liste. Tout chemin qui lit `bank_transactions` pour un
   lot, tout site qui rend un refus lié à l'état d'une transaction (`BANK_TRANSACTION_NOT_FOUND`,
   `RECONCILIATION_ALREADY_RECONCILED`, `RECONCILIATION_TRANSACTION_NOT_PENDING`, `race_during_update`,
   `OptimisticLockConflict`, `AppError::Validation` après une lecture), dans `routes/reconciliation.rs`, mais aussi
   `reconciliation_rules.rs`, `bank_imports.rs`, `payment_batches.rs`, `kesh-reconciliation`, le frontend. Un lot ou
   un site manquant, ou un site déclaré « résolu » que la fiche ne résout pas, est au moins **MEDIUM**.
2. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` — § *Supprimer un import*
   (l. 1722-1760, dont la puce l. 1756-1759), § *Accepter ou rejeter les propositions* (l. 1906-1925, dont la puce
   *Rejeter* l. 1920 et le paragraphe *Un refus n'empêche pas les autres*, l. 1925). Ce que le manuel dit
   **aujourd'hui** est-il vrai au code ? Ce que l'AC9 prescrit d'écrire sera-t-il vrai au code que la fiche
   prescrit — y compris pour les sites qu'elle laisse non résolus ? Contrôle aussi le **PDF**, aplati :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`. Une promesse que le code ne tiendrait pas
   est au moins MEDIUM.
3. **Recomptes** : chaque nombre de la fiche se recompte depuis la source — 26 → 27 littéraux
   (`grep -ohE 'error_code: "[A-Z_]+"' crates/kesh-api/src/routes/reconciliation.rs | sort -u | wc -l`), 28 → 29
   codes, `sitesTotal` 2041 → 2043 (`frontend/src/lib/shared/i18n-keys.test.ts:580`, et la méthode de comptage du
   fichier), 13 AC, T0-T8, M1-M15, 14 choix, chaque numéro de ligne cité.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de l'issue #634 (dont la question « un autre compte : `FailedProposal` ou refus global ? ») couverte
  par un AC testable, chaque AC par une tâche, chaque AC par un test **qui rougirait** ; chaque fait, chemin et numéro
  de ligne cité **revérifié** (`sed -n`, `grep -nF`) ; les textes prescrits (libellés et traductions de l'AC7 et des
  Dev Notes, manuel, `api-external.md`, CHANGELOG) confrontés à ce que le code prescrit fera réellement et à
  `docs/i18n-glossaire.md` ; les tests existants qui changeront de sens (`reconciliation_e2e.rs`,
  `reconciliation_rules_e2e.rs`, `reconciliation_split_e2e.rs`, `rejeu_interblocage_e2e.rs`,
  `reconciliation_repository.rs`, `failed-proposal-label.test.ts`, `ReconciliationProposals.test.ts`,
  `i18n-keys.test.ts`, `textes_coherents.rs`, `lint-i18n-ownership`) — la fiche les nomme-t-elle tous ? La table des
  mutations : chaque mutation est-elle rattachée à un test qui la tue réellement, ou en existe-t-il une
  **équivalente** non déclarée ? Les fichiers communs avec les branches `story/27-2-devise-import-csv` et
  `story/27-3-profil-apres-apercu` (`git diff --stat origin/main...<branche>`, lecture seule) : la liste de la fiche
  est-elle complète ?
- **F — Faisabilité, concurrence et contrat (adversaire de la conception)** : la classification de l'AC1 et sa
  priorité sont-elles justes au regard de **tous** les écrivains de `bank_transactions` (inventaire (b) de la fiche
  27-1) — un état `reconciled` sans `matched_entry_id`, `pending` avec lien, rejetée puis rapprochée ? L'AC4 est-elle
  correcte **sous InnoDB REPEATABLE READ** : `UPDATE` à 0 ligne puis `SELECT … FOR UPDATE` (lecture courante vs vue
  cohérente, verrou d'intervalle sur une ligne absente, ordre des verrous contre
  `bank_imports::delete_unreconciled_in_tx` et contre un lot d'acceptation concurrent) ? Le court-circuit de l'AC3 et
  le pré-vol hors de `retry_with` laissent-ils une course ou un double traitement au rejeu ? La fusion de `failed[]`
  dans l'ordre reçu (`accept`) et triée (`reject`) est-elle tenable sans ambiguïté (identifiants dupliqués déjà
  refusés en `400`) ? Le montage des tests de l'AC6-2 et de l'AC6-5 (verrou nommé tenu, attente sur `GET_LOCK`
  observée par `attendre_une_requete_en_cours`) est-il **déterministe** — la suppression d'import, qui ne prend pas le
  verrou du compte, peut-elle bloquer derrière le lot ou l'inverse ? L'anti-énumération (AC1, AC6-3) tient-elle aussi
  au **temps de réponse** et aux traces ? Le contrat publié (`docs/api-external.md:536-548`, § 4 et § 7) : le passage
  `400` → `200` + `failed[]` est-il écrit comme un changement, et un client existant peut-il s'y tromper ? Le
  plafond de l'AC5 : 500 identifiants passent-ils réellement la limite de corps d'Axum et le plafond du dépôt ?

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence =
sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte
pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/27-4-validate-p1-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/27-4-validate-p1-F.md` (lentille F) ; dernier message : le chemin du rapport,
le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et les
scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/
`reset`/`rebase`/`tag`/`push`/`worktree`, `cargo`, `npm`, `npx`, `docker`, `bash` ou `sh` sur un script du dépôt,
`gh` en écriture, toute requête SQL, et **tout interpréteur** — `python3`, `python`, `node`, `deno`, `perl`,
`ruby`, `awk` en programme écrit dans un fichier. Rien de `~/kesh-test/` (relevé réel de Guy) ne se lit ni ne se
cite. Autorisés : lecture, `grep`, `sed -n`, `wc`, `sort`, `uniq`, `git log`/`show`/`diff`/`blame`, `gh api` en
lecture, `pdftotext` vers la sortie standard.
