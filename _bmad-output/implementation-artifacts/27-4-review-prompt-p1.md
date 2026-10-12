# Prompt — revue de code P1, Story 27-4 (un lot de réconciliation qui contient une transaction disparue est traité proposition par proposition)

*Versionné le 2026-10-13. Trois lentilles (Sonnet), contexte frais chacune, lecture seule.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-27-4`, branche `story/27-4-lot-reconciliation-transaction-disparue`.
**Le diff à revoir** : `git diff 215c847c 3a293f36 -- . ':(exclude)_bmad-output'` (`215c847c` = validation close de la
fiche ; `ff74bacc` = dernier commit de **code** ; `3a293f36` = manuel, PDF, contrat d'API et CHANGELOG ; les commits
suivants ne portent que la fiche, le registre et ce prompt). ⚠️ Pathspec `':(exclude)…'` et **non** `':!…'`, que le
shell peut interpréter. Fiche : `_bmad-output/implementation-artifacts/27-4-lot-reconciliation-transaction-disparue.md`
— AC1 à AC13, Tâches T0-T8, Dev Notes (inventaire des lots, **inventaire des sites non résolus**, pièges connus,
fichiers communs avec les 27-2 et 27-3), et le **Dev Agent Record** (choix C-27-4-1 à 39 au registre
`epic-27-choix-autonomes.md`, mutations M1 à M19, décomptes, gates). Règles : `CLAUDE.md` du dépôt — § *Pattern batch —
FailedProposal per-proposal*, § « Recompter ses propres comptes rendus », § « Propagation post-patch », § « Le prompt
d'une passe doit NOMMER le manuel », § « Une passe qui ne déclare pas ses axes exercés ne compte pas », § « Migration
breaking policy » (aucune migration attendue). Issue : #634 (la PR la fermera) ; #632 (Story 27-1, qui rend le cas
atteignable).

⛔ **DONNÉES PERSONNELLES — DÉPÔT PUBLIC.** N'ouvre aucun fichier hors du dépôt et de
`/home/gcorbaz/devel/kesh-gate-logs/` ; rien sous `~/kesh-test/`.

**Contexte** : story backend + écran + textes. `kesh-db` : `repositories/reconciliation.rs` —
`find_pending_by_ids` **remplacée** par `find_by_ids_for_account` (sans filtre de statut), `MAX_IN_CLAUSE_IDS` rendue
`pub`, `find_by_id_for_account_for_update` (lecture verrouillante) et la fonction privée
`bank_tx_by_id_for_account_sql` qu'elle partage avec `find_pending_by_id_for_account` ; doc-comments de
`bank_imports.rs` (`is_reconciled`, angle mort « lot préparé avant la suppression ») ; tests
`crates/kesh-db/tests/reconciliation_repository.rs` (deux neufs). `kesh-api` : `routes/reconciliation.rs` —
`classify_batch_transaction`, `check_batch_size`, `merge_failed_in_order`, pré-vols de `post_accept` et `post_reject`,
court-circuit sans verrou, `batch_size` explicite d'`accept_batch`, `reject_batch` (garde de l'`UPDATE`, relecture
verrouillante, reclassement, repli `race_during_update`), module `batch_classification_tests` ; tests
`crates/kesh-api/tests/reconciliation_e2e.rs` (trois réécrits, AC2 (a)(b), AC3, AC5, AC6-1 à AC6-8). `kesh-i18n` : deux
clés neuves et une récrite × quatre catalogues. Frontend : `failed-proposal-label.ts` (+ test, 29 codes),
`ReconciliationProposals.test.ts`, `bank-import-deletion.ts` (doc), `i18n-keys.test.ts` (`sitesTotal` 2041 → 2043), E2E
`frontend/tests/e2e/reconciliation-deleted-import.spec.ts`. Textes : `docs/manual/fr/user-manual.tex` **et PDF**,
`docs/api-external.md`, `CHANGELOG.md` (`## [0.13.2] — Non publié`).

⛔ **Axes communs aux trois lentilles** (chacune les déclare exercés ou non) :

1. **Le § *Pattern batch*.** Plus aucune erreur **par proposition** n'escalade en `AppError` global dans `post_accept`
   et `post_reject` ; restent globaux, et seulement eux : corps vide ou mal formé, identifiant non positif, doublon,
   plus de 500 identifiants (`400`), compte bancaire inconnu, d'une autre société ou archivé
   (`404 BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND`), verrou non obtenu (`409`), `500` d'infrastructure. Le message « …
   n'appartiennent pas au bankAccountId » a-t-il disparu de partout (`grep -rnF "appartiennent pas au bankAccountId"
   crates frontend/src docs`) ? Le `?` de `find_by_id_for_account_for_update` dans `reject_batch` peut-il, hors panne,
   faire sortir le lot en `500` (cf. la réserve `innodb_snapshot_isolation` des Pièges connus) ?
2. **Une seule classification, une seule définition de « rapprochée ».** `classify_batch_transaction` appelle-t-elle
   `is_reconciled` sans second test de statut ? La priorité (rapprochée avant rejetée) ? L'anti-énumération : les trois
   causes d'absence par le **même** chemin, même code, `details` nul — rien d'autre (trace, temps, message) ne les
   distingue ? Les codes écrits au site (`error_code: "…".to_string()`) : recompter `grep -ohE 'error_code: "[A-Z_]+"'
   crates/kesh-api/src/routes/reconciliation.rs | sort -u | wc -l` (27 attendus).
3. **Ordre et fusion de `failed[]`.** `accept` : ordre des propositions reçues ; `reject` : trié en entier, refus du
   pré-vol et refus sous verrou mêlés ; l'ordre de verrouillage de la 27-1 (identifiant croissant) préservé ; le
   sous-lot éligible d'`accept` cloné dans la fermeture de `retry_with`, le pré-vol calculé une fois, hors du rejeu.
   `batch_size` = lot reçu.
4. **La reprise sous verrou du rejet.** Gardes de l'`UPDATE` (`bank_account_id`, `status`, `matched_entry_id IS NULL`,
   `auto_match_rejected_at IS NULL`) ; relecture `FOR UPDATE` (lecture courante) et reclassement ; repli « relue
   éligible » ; nouvel ordre de verrous ou cycle avec `bank_imports::delete_unreconciled_in_tx` ? La relecture
   `SELECT auto_match_rejected_at` qui suit un `UPDATE` réussi est-elle toujours juste ? La trace
   `reconciliation.rejected` (identifiants rejetés seulement, `entity_id` = le plus petit) ?
5. **Inventaire des sites non résolus** (Dev Notes) : est-il encore **clos** au `HEAD` ? Méthode : `grep -nE
   'BANK_TRANSACTION_NOT_FOUND|RECONCILIATION_ALREADY_RECONCILED|RECONCILIATION_ALREADY_REJECTED|RECONCILIATION_TRANSACTION_NOT_PENDING|race_during_update|find_by_ids_for_account|find_pending_by_id_for_account|find_strictly_pending|find_by_id_for_account_for_update'
   crates/kesh-api/src/routes/reconciliation.rs` — chaque site est-il résolu ou écrit comme angle mort ?
6. **i18n** : deux clés neuves et une récrite × 4 (`crates/kesh-i18n/locales/*/messages.ftl`), replis du `.ts`
   identiques au fr-CH, terminologie du glossaire (`docs/i18n-glossaire.md`), « **Cinq codes** » lisent leur
   `details` (doc-comment du module et de `failedProposalLabel`), « 29 codes » partout (`grep -nwE "2[6-9]"
   frontend/src/lib/features/reconciliation/failed-proposal-label*.ts`), `sitesTotal` recompté.
7. ⛔ **LE MANUEL — `docs/manual/fr/user-manual.tex` ET son PDF.** La puce « Une page *Réconciliation* ouverte avant
   la suppression » (§ *Supprimer un import*) et la liste des motifs de « Un refus n'empêche pas les autres »
   disent-elles **vrai** sur le code (`ReconciliationProposals.svelte` : relevé avant le lot, rechargement après, bilan
   hors des branches de la liste ; `reconciliation.rs`) ? La phrase « l'acceptation d'une proposition la refuse »
   (paragraphe *Rejeter*) reste-t-elle vraie ? **Cherche une phrase des manuels (utilisateur ET administration), de
   `api-external.md`, du README ou de `website/`, hors des sections touchées, que la story rend fausse** (ex. : un
   lot refusé en entier, « rechargez la page », un `400` de lot de réconciliation, une transaction rejetée
   « acceptée »). Contrôle le **PDF aplati et normalisé** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | sed "s/’/'/g; s/ﬀ/ff/g; s/ﬁ/fi/g; s/ﬂ/fl/g; s/ﬃ/ffi/g; s/ﬄ/ffl/g"`
   vers `/home/gcorbaz/devel/kesh-gate-logs/27-4-review-p1-<lettre>-user-manual-flat.txt` ; attendus de l'AC9
   (« refusé en entier » 1, « les autres propositions du lot sont traitées » 1, « rapprochée ou rejetée » 1).
8. ⛔ **LE CONTRAT D'API — `docs/api-external.md`.** Le changement `400` → `200` + `failed[]` est-il écrit **comme un
   changement de contrat** ? Les codes du pré-vol et ceux d'une transaction changée pendant le traitement
   (`RECONCILIATION_TRANSACTION_NOT_PENDING` des variantes `split` et `rule`, `details.reason = "race_during_update"`
   sur `RECONCILIATION_ALREADY_RECONCILED`) sont-ils exacts **au code** ? Le plafond de 500 et la limite de corps
   d'Axum ? « *depuis la v0.13.2* », jamais « Unreleased » ni « à venir ». Le `CHANGELOG.md` : une seule section
   `## [0.13.2] — Non publié`, **un seul** `### Corrigé`, lignes de la 27-1 intactes, rien qui promette au-delà du
   code.

## Lentilles

- **B — Blind Hunter : bugs et états**, sans présupposer que la fiche a raison. Le pré-vol (une transaction présente
  dans la `HashMap` mais d'un statut inconnu — `BankTransactionStatus` ne se décode pas : erreur `500` ?), la fusion
  (`HashMap::remove` sur un identifiant absent des deux sources ?), le court-circuit (un lot entièrement refusé qui
  rendait autrefois `409` sous verrou tenu), la reprise (une relecture qui rend une ligne d'un **autre** état que
  celui qui a fait échouer l'`UPDATE` ; `map_err(ReconciliationError::Db)` et le bras `Db` de `post_reject`), le
  libellé côté écran (`failureReason` sur un `details` mal formé), le spec E2E (nettoyage en `finally` quand le test
  échoue avant la création de la règle).
- **E — Edge Case Hunter : bords, et les tests prouvent-ils ce qu'ils nomment.** Rejoue mentalement les mutations
  M1 à M19 du Dev Agent Record (dont M18 et M19, déclarées équivalentes, et M11, dont l'angle mort est écrit au
  doc-comment de l'AC6-8) et cherche une **mutation plausible qui resterait verte** (ex. : `company_id` retiré de
  `find_by_ids_for_account` ; le pré-vol de `reject` lu sur le lot non trié ; `merge_failed_in_order` qui garde le
  doublon d'une source ; le repli `race_during_update` d'`accept_one_*` changé ; le court-circuit qui rend
  `failed: []` ; le texte SQL partagé qui perd ` FOR UPDATE` côté appelant). Tests qui passent à vide ? Les courses :
  synchronisation par `attendre_une_requete_en_cours` seulement (motif `GET_LOCK`, motifs de l'AC6-8), un seul
  `sleep` assumé (500 ms de l'AC6-8, C-27-4-26) ; motifs suffisamment étroits. Les montages qui construisent bien
  l'état qu'ils croient (identifiants croissants de l'AC6-7, `low_absent < mid` de l'AC2 (a)). L'E2E : indépendance,
  rejouabilité (deuxième passage sur la même base), `data-testid` seuls, règle à valeur unique.
- **A — Acceptance Auditor : conformité, décomptes, documentation.** Chaque AC (1 à 13) contre le code : satisfait,
  partiel, absent. Les tests nommés par la fiche existent-ils et prouvent-ils ce qu'ils nomment ? Aucun fichier de
  `migrations/`, de `test-schema/`, ni `post_restore.rs`, ni `Cargo.toml` touché (`git diff --stat`). AC13 : chaque
  mutation exécutée et son résultat rapporté au journal `kesh-gate-logs/27-4-mutations.log` (sorties dans
  `kesh-gate-logs/27-4-mut/`). **Décomptes recomptés depuis la source** (littéraux de codes, codes du test Vitest,
  clés i18n, `sitesTotal`, tests ajoutés, chiffres de gates déclarés) et le **périmètre** de chaque décompte. Les choix
  C-27-4-33 à 39 : justifiés, ou défaut ? Les conflits annoncés avec la 27-2 (PR #636) et la 27-3 (§ *Fichiers
  communs*) : la table est-elle juste au `HEAD` de chaque branche (`git diff --stat origin/main...<branche>`) ?

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/27-4-review-p1-<B|E|A>.md` (ta lettre). Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction proposée. Pour
tout finding affirmant qu'un code ou un texte est absent ou présent : la sortie d'un `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** (les huit axes communs et ceux de ta lentille) — un « 0 finding » sans
elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` (et le texte aplati du PDF au
même endroit) ; aucune commande qui écrit dans le dépôt ou dans une base, ni qui compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`,
`xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/`rebase`/`restore`, `sqlx`, `cargo` (aucune
sous-commande), `rustc`, `npm`, `npx`, `node`, `deno`, `bun`, `python`/`python3`, `perl`, `ruby`, `php`, `awk` en
écriture, tout autre interpréteur, `mariadb`, `mysql`, `docker`, `curl` et `wget` (aucune requête HTTP), `gh` en
écriture, aucune requête SQL. Aucun fichier hors du dépôt. Autorisés : lecture, `grep`, `sed -n`, `wc`, `sort`,
`git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers `/home/gcorbaz/devel/kesh-gate-logs/`.
