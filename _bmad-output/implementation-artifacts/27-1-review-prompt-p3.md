# Prompt — revue de code P3, Story 27-1 (supprimer un import bancaire dont aucune transaction n'est rapprochée)

*Versionné le 2026-10-12. Passe **complète**, trois lentilles (**Sonnet**), contexte frais chacune, lecture seule.
Rotation : P1 Sonnet → P2 Opus → P3 Sonnet (CLAUDE.md, décision D6). Copie adaptée de `27-1-review-prompt-p2.md` :
mêmes axes et mêmes interdits, l'axe 9 braqué sur la remédiation **P2**.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-27-1`, branche `story/27-1-supprimer-import`. **Le diff à revoir** :
`git diff fc45574c 1c9e4ea0 -- . ':(exclude)_bmad-output'` (`fc45574c` = validation close de la fiche ; `1c9e4ea0` =
dernier commit de code — remédiations P1 et P2 comprises ; les commits suivants ne portent que la fiche, le registre et
ce prompt). **La remédiation P2 seule** : `git diff 1cc5e8d7 1c9e4ea0 -- . ':(exclude)_bmad-output'` (trois commits,
`a5387405` tests, `7ab16821` écran et i18n, `1c9e4ea0` textes, Makefile et PDF : `git log --oneline 1cc5e8d7..1c9e4ea0`).
La remédiation P1, pour mémoire : `git diff 6e305767 5881450b`. ⚠️ Pathspec
`':(exclude)…'` et **non** `':!…'`, que le shell peut interpréter. Fiche :
`_bmad-output/implementation-artifacts/27-1-supprimer-import-non-rapproche.md` — AC1 à AC16 (dont AC4-bis), Tâches,
Dev Notes (inventaires (a) et (b), concurrence, angles morts), et le **Dev Agent Record** (choix C-27-1-1 à 31 au
registre `epic-27-choix-autonomes.md`, `EXPLAIN` de F10, mutations M1 à M39, décomptes, gates) et les sections
**« Revue de code P1 »** et **« Revue de code P2 »** du Change Log (findings, traitements, reclassements). Rapports :
`/home/gcorbaz/devel/kesh-gate-logs/27-1-review-p1-{B,E,A}.md` et `27-1-review-p2-{B,E,A}.md`. Règles : `CLAUDE.md`
du dépôt — § « Recompter ses propres comptes rendus », § « Propagation post-patch », § « Le prompt d'une passe doit
NOMMER le manuel », § « Une passe qui ne déclare pas ses axes exercés ne compte pas », § « Migration breaking policy »
(aucune migration attendue, AC15). Issues : #632 (la PR la fermera) ; #633, #634, #465 (référencées : angles morts
assumés, non corrigés ici).

⛔ **DONNÉES PERSONNELLES — DÉPÔT PUBLIC.** N'ouvre aucun fichier hors du dépôt ; la fixture E2E
`frontend/tests/e2e/fixtures/csv_e2e_delete_import.csv` est synthétique.

**Contexte** : story **pleine pile**. `kesh-db` : `bank_imports::{is_reconciled, DeleteBankImportOutcome,
DeletedBankImport, delete_unreconciled_in_tx}` (verrous entête puis transactions, `FORCE INDEX
(idx_bank_transactions_import)`, cascade), doc-comment de module corrigé, doc-comment de
`count_transactions_for_bank_account` ; tests `crates/kesh-db/tests/bank_import_delete.rs` (D1-D6, AC3 (a)(b)(c)).
`kesh-api` : `AppError::{ApiKeyOperationForbidden, BankImportHasReconciledTransactions}` et leurs réponses, message du
`412 BANK_ACCOUNT_HAS_TRANSACTIONS` ; `routes/bank_imports::{delete, delete_once}` (refus des clés, rejeu, rollback
explicite, trace `bank_import.deleted`) ; montage `lib.rs` (`comptable_routes`) ; `audit_labels.rs` ; registre
`audit_route_registry.rs` (115/109/118/91, point (iv) et (vi)) ; tri de `post_reject` (`reconciliation.rs`, AC4-bis) ;
tests H1-H5, H8 (`bank_imports_e2e.rs`), H6, H9 (`bank_accounts_e2e.rs`), H7 (`reconciliation_e2e.rs`). `kesh-i18n` :
onze clés neuves et une modifiée × quatre catalogues ; test de message (`loader.rs`). Frontend :
`bank-import-deletion.ts`, `deleteBankImport`, `DeleteBankImportAction.svelte`, page `bank-import/[id]` (et son Vitest
de page `bank-import-detail-page.test.ts`), `data-testid="bank-import-list-link"` de `BankImportList.svelte`, Vitest,
`i18n-keys.test.ts` (`sitesTotal` 2032 → 2041), E2E `bank-import-delete.spec.ts`. Textes : manuels utilisateur et
d'administration FR (`.tex` **et PDF**), `docs/api-external.md` (§ 4, § 7, note ⁸, § 10, rejet trié), `CHANGELOG.md`
(`## [0.13.2] — Non publié`), `README.md` ; `docs/manual/Makefile` (passes jusqu'à stabilisation, C-27-1-30).

⛔ **Axes communs aux trois lentilles** (chacune les déclare exercés ou non — l'axe 9 compris) :
1. **La garde et ses verrous.** Aucune lecture ordinaire avant le premier `FOR UPDATE` (ni dans le dépôt, ni dans
   `delete_once`, ni dans le handler) ; comptage des rapprochées sur le résultat de la lecture verrouillante ;
   `rows_affected() != 1` ; aucune écriture sur `NotFound` / `HasReconciled` ; `rollback` explicite avant chaque
   erreur ; erreurs sqlx par `map_db_error` (1213 reconnu par `retry_app_on_deadlock`). Le plan de l'étape 2 et le
   `FORCE INDEX` (C-27-1-21) : justifié ? un effet de bord (motif de la sonde de l'AC3 (a)) ?
2. **« Rapprochée », deux fois et seulement deux** : `is_reconciled` (Rust) et `isReconciled` (TS) disent-ils la même
   chose ? Un troisième site qui décide de la même question ailleurs (`grep -rn "matched_entry_id\|matchedEntryId"
   crates frontend/src`) ? L'inventaire (b) des écrivains de `bank_transactions` est-il toujours clos au `HEAD` ?
3. **Refus et ordre** : rôle (`403`), clé `read` (`403 API_KEY_READ_ONLY`), clé `read-write`
   (`403 API_KEY_OPERATION_FORBIDDEN`, avant l'enveloppe), `id <= 0` et autre société (`404 NOT_FOUND`), rapprochée
   (`409`, `details.reconciledCount` / `transactionCount`). Rien d'écrit sur chacun. Doctrine D2 respectée (code neuf,
   message distinct) ; `require_not_pat` non employé.
4. **Trace d'audit** : action littérale à l'index 2 de `for_actor`, huit clés de `details_json` (snake_case, dates
   `AAAA-MM-JJ`), écrite après le `DELETE` et avant le `commit` ; `ACTIONS` trié ; libellé dans les quatre catalogues.
5. **AC4-bis** : `post_reject` trié avant le verrou, effets écrits (`rejected[]`, `failed[]`, `entity_id`,
   `details_json`), commentaires corrigés, test existant et H7. Un consommateur qui dépendait de l'ordre reçu ?
6. **i18n** : onze clés neuves × 4, une modifiée × 4 (chemin de menu = libellés réels de `nav-quotidien` /
   `nav-bank-import` dans chaque langue), replis Svelte et Rust = texte fr-CH, arguments Fluent présents partout,
   terminologie du glossaire (`docs/i18n-glossaire.md`), `sitesTotal` recompté aux deux bornes, `lint-i18n-ownership`.
7. ⛔ **LES MANUELS — `docs/manual/fr/user-manual.tex`, `admin-manual.tex` ET leurs PDF.** La section « Supprimer un
   import », le paragraphe *Rejeter*, la tuile *Relevé du …* (deux sites), le renvoi du § *Comptes bancaires*, la puce
   *Suppression d'un import bancaire* et la puce *Portée* du § *Clés API* du manuel d'administration disent-ils
   **vrai** sur le code (relire `DeleteBankImportAction.svelte`, `+page.svelte`, `routes/bank_imports.rs`,
   `routes/bank_accounts.rs` (archivage), `repositories/bank_accounts.rs` (tuile), `reconciliation.rs`) ? Chaque
   affirmation : chemin de menu, libellés, rôles, rejetées comprises, trace, réimport sans avertissement, tuile,
   archivage « s'il ne reste aucune transaction », motifs d'annulation et leurs remèdes, lot refusé en entier.
   **Cherche une phrase des manuels, de `api-external.md`, du README ou du site, hors des sections touchées, que la
   story rend fausse** (ex. : une phrase qui énumère les fermetures aux clés ; « aucun geste » ; une mention de
   l'irréversibilité d'un import ; l'archivage d'un compte bancaire). Contrôle les **PDF aplatis**
   (`pdftotext docs/manual/fr/<manuel>.pdf - | tr '\n' ' ' | tr -s ' '` vers `/home/gcorbaz/devel/kesh-gate-logs/`) —
   l'apostrophe y est typographique, écrire `.` à sa place dans un motif.
8. **CHANGELOG, `api-external.md`, README** : une seule section `## [0.13.2] — Non publié` ; rien qui promette
   au-delà du code ; « depuis la v0.13.2 » et jamais « Unreleased » ni « à venir » ; paragraphe *Interblocages*
   inchangé (AC4) ; références `#N` du README en liens.

9. ⛔ **La remédiation P2 — régressions du patch** (`git diff 1cc5e8d7 1c9e4ea0`). Le motif mesuré du dépôt : *la
   sévérité se déplace vers ce qu'on vient d'écrire*. Relis chaque commit de la remédiation comme du code neuf :
   - `add_lines` (E-P2-1, E-P2-2) : les lignes copiées par `INSERT … SELECT` sont-elles réalistes (même société, même
     compte, même import ; colonnes `NOT NULL` toutes fournies) ? Ne faussent-elles rien d'autre dans H1 (aperçu sans
     `duplicateFile`, réimport `n2 == n` — l'entête dit toujours `n`) ou H2 (« import intact » à `n + 1`) ? Les
     assertions distinguent-elles enfin lignes, entête, rapprochées, rejetées (M28 à M33) — ou une autre inversion
     plausible reste-t-elle verte ?
   - H8 sur `id + 10_000` (E-P2-4) : la clé `read` aussi — le middleware l'arrête-t-il avant le `404` ? M34 rougit-elle
     pour la bonne raison ?
   - G9 étendue aux trois messages `ubs_*` (E-P2-3) : paramètres `{ $count }` → `{total_errors}` et `{ $line }` →
     `{line}` justes ? nombre de sites (1) exact ? le doc-comment de H2 dit-il vrai ?
   - `DeleteBankImportAction.svelte` (B2-1) : `bind:open` et la garde de `Dialog.Content` — une fermeture reste-t-elle
     possible pendant l'envoi (bouton *Annuler*, navigation, `onOpenChange` disparu) ? Après un succès, `open = false`
     puis `onDeleted` ; après une erreur, le dialogue reste ouvert et l'erreur visible ? Les deux Vitest neufs
     prouvent-ils ce qu'ils nomment sous jsdom (Échap atteint-il bits-ui ? l'attente de 50 ms est-elle une
     synchronisation par minutage déguisée ?) ; `interactOutsideBehavior` n'est tenu par aucun test (C-27-1-28) :
     l'angle mort est-il acceptable ?
   - Message du 409 de course (B2-4) : les quatre locales et le repli Svelte disent-ils la même chose, avec l'argument
     `count` ; terminologie du glossaire ; le Vitest du `409` lit-il le texte neuf ?
   - Vitest de page (E-P2-5) : mocks hoistés justes, détail réellement chargé avant l'assertion d'absence, M39 rouge.
   - Textes (B2-2, B2-3, B2-5) : « le relevé dont la période se termine le plus tard » est-il vrai contre
     `repositories/bank_accounts.rs` (`ORDER BY period_to DESC, imported_at DESC, id DESC`, relevé **portant un solde
     de clôture**) aux trois sites du manuel, aux doc-comments et commentaires TS/Svelte ? **Grep de la valeur** :
     reste-t-il « plus grande période » ou une variante (« plus longue », « dernier relevé ») hors artefacts BMAD ?
     « s'il n'en reste aucun autre import » vrai contre `find_by_company_and_hash` ? `POST /bank-imports/preview` dans
     la bonne colonne du § 7 ?
   - Makefile (C-27-1-30) : la boucle termine-t-elle toujours (échec bruyant à `MAX_PASSES`) ; le motif du journal
     ne prend-il ni trop (bannière `rerunfilecheck`) ni trop peu ; `$$` et continuations justes sous `make` ; une
     erreur de `xelatex` fait-elle bien échouer la cible ? PDF : TOC contre pieds de page (six titres orphelins
     préexistants déclarés au registre, à ne pas compter comme régression s'ils sont identiques à `5881450b`).
   - Campagne M28 à M39 (`kesh-gate-logs/27-1-mutations.log`, `27-1-mut/M28-0.out` à `M39-0.out`) : chaque mutation
     a-t-elle rougi pour la bonne raison ? Une mutation plausible du code **remédié** reste-t-elle verte ?

## Lentilles

- **B — Blind Hunter : bugs et états**, sans présupposer que la fiche a raison. Le dépôt (chaque étape, import vide,
  `transaction_count` de l'entête ≠ lignes réelles, conversion `usize → i64`), le handler (enveloppe rejouée :
  l'erreur `ApiKeyOperationForbidden` hors de l'enveloppe ; `delete_once` rejoué après un commit partiel ?), la
  réponse `409` (`t_args`, repli `format!`), le composant (état du dialogue après erreur, fermeture pendant l'envoi,
  `details` mal formé, `count` du corps = `detail.transactions.length`), la page (notification, `goto`), le client API
  sur `204`, les types TS contre les DTO Rust.
- **E — Edge Case Hunter : bords, et les tests prouvent-ils ce qu'ils nomment.** Rejoue mentalement les mutations
  M1 à M39 du Dev Agent Record et du Change Log (dont M14 et M15, déclarées équivalentes, et M3, rouge par la sonde) et cherche une
  **mutation plausible qui resterait verte** (ex. : rapprochées comptées par une seconde requête non verrouillante ;
  trace écrite avec un mauvais `entity_id` ; `rejected_count` compté sur les rapprochées ; `FORCE INDEX` retiré ; tri
  de `post_reject` appliqué après le verrou ; le composant comptant `detail.transactionCount` au lieu des lignes).
  Tests qui passent à vide ? Les courses : synchronisation par la sonde seulement, jamais par `sleep` ; motifs
  suffisamment étroits. L'E2E : indépendance, rejouabilité (deuxième passage sur la même base : profil en `409`,
  import déjà présent ?), `data-testid` seuls.
- **A — Acceptance Auditor : conformité, décomptes, documentation.** Chaque AC (1 à 16, et 4-bis) contre le code :
  satisfait, partiel, absent. Les tests nommés (D1-D6, AC3 (a)(b)(c), H1-H9, test de message, Vitest, E2E) existent-ils
  et prouvent-ils ce qu'ils nomment ? AC15 (`git diff --stat` : ni `migrations/`, ni `test-schema/`, ni
  `post_restore.rs`, ni `Cargo.toml`). AC16 (chaque mutation exécutée, résultat rapporté, journal
  `kesh-gate-logs/27-1-mutations.log`). **Décomptes recomptés depuis la source** (clés i18n, `sitesTotal`, registre
  des routes, tests ajoutés, nextest / Vitest / E2E déclarés) et périmètre de chaque décompte. Les choix C-27-1-28 à
  31 : justifiés, ou défaut ? Les findings P1 laissés (B-2, E6) et le traitement de chaque finding P2 (MEDIUM et
  LOW) : l'argument tient-il ? La File List et les *Project Structure Notes* couvrent-elles tout le diff
  (`git diff --name-only fc45574c 1c9e4ea0 -- . ':(exclude)_bmad-output' | wc -l`) ?

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/27-1-review-p3-<B|E|A>.md` (ta lettre). Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction proposée. Pour
tout finding affirmant qu'un code ou un texte est absent ou présent : la sortie d'un `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** (les neuf axes communs et ceux de ta lentille) — un « 0 finding » sans
elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` (et le texte aplati des PDF au
même endroit) ; aucune commande qui écrit dans le dépôt ou dans une base, ni qui compile ou exécute : `scripts/*` (dont
`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`, `xelatex`,
`git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/`rebase`, `sqlx`, `cargo` (aucune sous-commande), `npm`, `npx`,
`node`, `docker`, `gh` en écriture, aucune requête SQL ni HTTP. Aucun fichier hors du dépôt. Autorisés : lecture,
`grep`, `sed -n`, `wc`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers `/home/gcorbaz/devel/kesh-gate-logs/`.
