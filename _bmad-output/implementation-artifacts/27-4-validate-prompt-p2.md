# Prompt — validation P2 de la spec, Story 27-4 (lot de réconciliation contenant une transaction disparue)

*Versionné le 2026-10-12. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : Sonnet. P2 : **Opus**
pour les deux lentilles — passe **complète**, non ciblée.*

Dépôt `/home/gcorbaz/devel/kesh-27-4`, branche `story/27-4-lot-reconciliation-transaction-disparue` (base
`f9725c29`, = merge de la Story 27-1, #635).
Fiche : `_bmad-output/implementation-artifacts/27-4-lot-reconciliation-transaction-disparue.md` — création
`70e1523d`, **remédiation de la P1 `7a610943`** (`git show 7a610943` ; `git diff 70e1523d 7a610943`).
Rapports de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/27-4-validate-p1-F.md` et `…-p1-R.md`.
Epic : `_bmad-output/planning-artifacts/epic-27-recette-v0-13-1.md` (section « Amendement du 2026-10-12 »).
Issue #634 (`gh api repos/guycorbaz/kesh/issues/634` et `…/634/comments`) ; contexte : #632 et la fiche
`27-1-supprimer-import-non-rapproche.md` (AC3 (b), AC8, choix C-27-1-17). Registre :
`_bmad-output/implementation-artifacts/epic-27-choix-autonomes.md`, **C-27-4-1 à C-27-4-25** (C-27-4-15 à 25 : la
remédiation de la P1 ; C-27-4-19 remplace la clause « `match` exhaustif » de C-27-4-1).
Règles : `CLAUDE.md` — § *Pattern batch — FailedProposal per-proposal* (au cœur de la story), § *Review Iteration
Rule* et ses sous-sections (le manuel nommé, les axes déclarés, les sites NON RÉSOLUS, la propagation post-patch, le
recompte des comptes rendus), § *Pendant une boucle de revue* (exception `kesh-db`), § *Issue Tracking Rule*.

## Axe premier (commun aux deux lentilles) — les régressions de la remédiation P1

*La sévérité se déplace vers ce qu'on vient d'écrire* (`CLAUDE.md`, § *La passe ciblée*). Relis d'abord
`git diff 70e1523d 7a610943`, phrase par phrase, et cherche ce que la remédiation a **cassé ou laissé à moitié** :

1. **Les montages neufs sont-ils justes et déterministes ?**
   - **AC6-8** (tue M11) : le `DELETE` non validé tient-il bien un verrou X sur la seule ligne `tA` (clé primaire,
     pas de verrou d'intervalle qui bloquerait l'`UPDATE` de `tB`) ? La lecture `SELECT auto_match_rejected_at` de
     `tB` ouvre-t-elle **réellement** la vue cohérente de la transaction du lot (avant elle, rien que des lectures
     courantes — vérifie `with_account_lock` et ce qu'il exécute après `BEGIN`) ? Le pré-vol, lecture hors
     transaction, voit-il `tA` (suppression non validée) ? L'observateur `attendre_une_requete_en_cours` avec
     `["UPDATE bank_transactions", "auto_match_rejected_at"]` ne peut-il pas confondre l'`UPDATE` de `tB` (déjà
     terminé ?) avec celui de `tA` ? `innodb_lock_wait_timeout` et `LOCK_TIMEOUT_SECS` laissent-ils la fenêtre
     nécessaire ? Sous M11, le repli rend-il bien `race_during_update` — et sous le code correct, la relecture
     `FOR UPDATE` voit-elle la ligne absente **après** la validation ?
   - **AC6-7** (tue M9) : `tHaut` rejetée avant le lot, `tBas` supprimée pendant — le sous-lot éligible est
     `[tBas]`, le verrou est donc pris : le montage de l'AC6-2 s'applique-t-il sans changement ? `failed` attendu
     `[tBas, tHaut]` : est-ce bien le contrat de l'AC4 et ce qu'une fusion fautive inverserait ?
   - **AC6-5** (tue M10) : la sentinelle `'2026-01-01 00:00:00.000'` tient-elle dans `DATETIME(3)` ; la `version`
     relue est-elle celle d'après l'`UPDATE` direct du test ; les assertions (a)-(c) rougissent-elles **toutes**
     sous M10, et aucune sous le code correct ?
2. **La réutilisation d'`is_reconciled` (C-27-4-19)** : la classification qui en découle est-elle juste au regard de
   **tous** les écrivains de `bank_transactions` ? L'abandon du `match` exhaustif laisse-t-il un cas qui échoue
   **ouvert** ? Le refus au pré-vol de `pending` + `matched_entry_id` change-t-il un comportement **atteignable**
   (un test existant, une route, l'écran `bank-import-deletion.ts`) ? Le doc-comment « Écrite deux fois, et
   seulement deux » reste-t-il vrai tel que l'AC12 le prescrit ?
3. **La fonction de dépôt `find_by_id_for_account_for_update` (C-27-4-21)** et le changement de signature de
   `reject_batch` : appelants, tests, `Project Structure Notes`, File List — tout est-il cohérent ?
4. **Les mutations déclarées équivalentes (M17, M18)** : la raison tient-elle au code ? Rejoue la commande de M17
   (`grep -rnE "SET[^;]*bank_account_id *=" crates --include=*.rs --include=*.sql`) et **élargis-la** (un `INSERT
   … ON DUPLICATE KEY UPDATE`, un `REPLACE`, un import de sauvegarde peuvent-ils changer le compte d'une ligne
   existante ?). Une équivalence fausse est au moins **MEDIUM** : elle désactive une preuve en silence.
5. **Le contrat réécrit (AC10, C-27-4-18)** et le manuel (AC9, C-27-4-23) : chaque code annoncé est-il émis là où le
   texte le dit, et aucun code émis n'est-il tu ? La décision de ne **pas** toucher la l. 1920 est-elle tenable au
   regard de la règle « le manuel dit vrai » ?
6. **Le triage des angles morts (C-27-4-25)** : chaque site classé catégorie C l'est-il vraiment, ou l'un d'eux
   est-il un défaut (écriture, solde, piste d'audit faux) qui exigerait une issue ? Un site manquant au tableau ?
7. **Propagation** : chaque symptôme corrigé (code `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND`, M1-M15 → M1-M18, « six »
   → « huit » scénarios, `MAX_BATCH_IDS`, « à confirmer », « une seule trace », « reste vrai », « Quatre codes »)
   a-t-il un résidu, dans la fiche, le registre ou le prompt ? Greppe la **valeur**, pas la formulation.

## Axes prioritaires (communs aux deux lentilles, repris de la P1)

1. **Inventaire des lots et des sites non résolus** (Dev Notes, deux tableaux et le triage) : refais les commandes
   copiées et **élargis-les** — un détecteur part du symptôme, pas d'une liste. Tout chemin qui lit
   `bank_transactions` pour un lot, tout site qui rend un refus lié à l'état d'une transaction
   (`BANK_TRANSACTION_NOT_FOUND`, `RECONCILIATION_ALREADY_RECONCILED`, `RECONCILIATION_ALREADY_REJECTED`,
   `RECONCILIATION_TRANSACTION_NOT_PENDING`, `race_during_update`, `OptimisticLockConflict`,
   `AppError::Validation` après une lecture), dans `routes/reconciliation.rs`, mais aussi `reconciliation_rules.rs`,
   `bank_imports.rs`, `payment_batches.rs`, `kesh-reconciliation`, le frontend. Un lot ou un site manquant, ou un
   site déclaré « résolu » que la fiche ne résout pas, est au moins **MEDIUM**.
2. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` — § *Supprimer un import*
   (l. 1722-1760, dont la puce l. 1756-1759), § *Accepter ou rejeter les propositions* (l. 1906-1925, dont la puce
   *Rejeter* l. 1920 et le paragraphe *Un refus n'empêche pas les autres*, l. 1925). Ce que le manuel dit
   **aujourd'hui** est-il vrai au code ? Ce que l'AC9 prescrit d'écrire sera-t-il vrai au code que la fiche
   prescrit — y compris pour les sites qu'elle laisse non résolus ? Contrôle aussi le **PDF**, aplati **et
   normalisé** (ligatures, apostrophe typographique) :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | sed "s/’/'/g; s/ﬀ/ff/g; s/ﬁ/fi/g; s/ﬂ/fl/g"`.
   Une promesse que le code ne tiendrait pas est au moins MEDIUM.
3. **Recomptes** : chaque nombre de la fiche se recompte depuis la source — 26 → 27 littéraux
   (`grep -ohE 'error_code: "[A-Z_]+"' crates/kesh-api/src/routes/reconciliation.rs | sort -u | wc -l`), 28 → 29
   codes, `sitesTotal` 2041 → 2043 (`frontend/src/lib/shared/i18n-keys.test.ts:580`, et la méthode de comptage du
   fichier), 13 AC, AC6-1 à AC6-8, T0-T8, mutations M1-M18 (19 lignes, M7a/M7b), 25 choix, « refusé en entier »
   2 → 1 dans le PDF, chaque numéro de ligne cité (dont ceux ajoutés par la remédiation : `errors.rs:2250`,
   `reconciliation_e2e.rs:2210`, l. 1661 / 2197 / 2573, `bank_imports.rs:346-349`, `repositories/reconciliation.rs:214`,
   `reconciliation.rs:1464`, `:3182-3189`, `failed-proposal-label.ts:25`).

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de l'issue #634 couverte par un AC testable, chaque AC par une tâche, chaque AC par un test **qui
  rougirait** ; chaque fait, chemin et numéro de ligne cité **revérifié** (`sed -n`, `grep -nF`) ; les textes
  prescrits (libellés et traductions de l'AC7 et des Dev Notes, manuel, `api-external.md`, CHANGELOG) confrontés à ce
  que le code prescrit fera réellement et à `docs/i18n-glossaire.md` ; les tests existants qui changeront de sens
  (`reconciliation_e2e.rs`, `reconciliation_rules_e2e.rs`, `reconciliation_split_e2e.rs`, `rejeu_interblocage_e2e.rs`,
  `reconciliation_repository.rs`, `bank_imports_e2e.rs` et `bank_import_delete.rs` — `is_reconciled` désormais
  appelée ailleurs —, `failed-proposal-label.test.ts`, `ReconciliationProposals.test.ts`, `i18n-keys.test.ts`,
  `textes_coherents.rs`, `lint-i18n-ownership`) — la fiche les nomme-t-elle tous ? La table des mutations : chaque
  mutation est-elle rattachée à un test qui la tue réellement, ou en existe-t-il une **équivalente** non déclarée —
  ou une déclarée équivalente qui ne l'est pas ? Les fichiers communs avec les branches
  `story/27-2-devise-import-csv` et `story/27-3-profil-apres-apercu` (`git diff --name-only origin/main...<branche>`,
  lecture seule) : la table corrigée est-elle complète et exacte ?
- **F — Faisabilité, concurrence et contrat (adversaire de la conception)** : la classification de l'AC1 et sa
  priorité, désormais fondée sur `is_reconciled`, sont-elles justes au regard de **tous** les écrivains de
  `bank_transactions` (inventaire (b) de la fiche 27-1) ? L'AC4 est-elle correcte **sous InnoDB REPEATABLE READ** :
  `UPDATE` gardé à 0 ligne puis `find_by_id_for_account_for_update` (lecture courante vs vue cohérente, verrou
  d'intervalle sur une ligne absente, ordre des verrous contre `bank_imports::delete_unreconciled_in_tx` et contre
  un lot d'acceptation concurrent) ? Les montages des AC6-2, AC6-5, AC6-7 et **AC6-8** sont-ils déterministes — un
  `DELETE` non validé tenu par le test peut-il bloquer autre chose que l'`UPDATE` visé (pré-vol, relecture de `tB`,
  trace d'audit, clés étrangères) ? Le court-circuit de l'AC3 et le pré-vol hors de `retry_with` laissent-ils une
  course ou un double traitement au rejeu ? La fusion de `failed[]` (ordre reçu pour `accept`, trié en entier pour
  `reject`) est-elle tenable sans ambiguïté ? L'anti-énumération (AC1, AC6-3) tient-elle aussi au **temps de
  réponse** et aux traces ? Le contrat publié réécrit (`docs/api-external.md:536-548`) : un client existant peut-il
  encore s'y tromper ? Le plafond de l'AC5 : 500 identifiants passent-ils la limite de corps d'Axum et le plafond du
  dépôt, et `MAX_IN_CLAUSE_IDS` rendue `pub` change-t-elle quoi que ce soit pour `find_contacts_by_ids` ?

## Ce que tu rends

Findings numérotés (`R2-…` ou `F2-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée ; pour chaque finding, dis s'il est **né de la
remédiation P1** (`7a610943`) ou préexistant à la création de la fiche. Toute affirmation de présence ou d'absence =
sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte
pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/27-4-validate-p2-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/27-4-validate-p2-F.md` (lentille F) ; dernier message : le chemin du rapport,
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
