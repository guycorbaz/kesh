# Prompt — validation P2 **ciblée** de la spec, Story 28-6 (un débit ne règle pas une facture client)

*Versionné le 2026-10-13. **Une seule lentille**, **Haiku**, contexte frais, lecture seule, braquée sur **le seul
commit de la remédiation P1** (`CLAUDE.md`, § *La passe ciblée* : quand une boucle converge, ce qu'il reste à relire
n'est plus la story, c'est la dernière remédiation). Trend : P1 = 0 CRITICAL, 0 HIGH, 1 MEDIUM, 9 LOW (8 distincts).*

Dépôt `/home/gcorbaz/devel/kesh-28-6`, branche `story/28-6`.
**Périmètre : `git show 362e9f06`** (remédiation P1), soit un **diff unique** — ne lis pas la séquence des commits
antérieurs. Fichiers touchés : la fiche `_bmad-output/implementation-artifacts/28-6-rapprochement-debit-facture.md` et
le registre `_bmad-output/implementation-artifacts/epic-28-choix-autonomes.md` (C-28-6-3, -8, -10 amendés ; C-28-6-11 à
-14 neufs). Rapports P1 que ce commit remédie : `/home/gcorbaz/devel/kesh-gate-logs/28-6-validate-p1-{R,F}.md`.

## La lentille — chasseur de régressions de la remédiation P1

Pour **chaque hunk** du commit, vérifie contre le code au sol :

1. **AC9 / C-28-6-8 / C-28-6-11, manuel d'administration** : la description du paragraphe de la 0.13.2
   (`docs/manual/fr/admin-manual.tex:1992-1994`, trois codes nommés, renvoi à `docs/api-external.md`) est-elle exacte
   mot pour mot ? L'affirmation « aucune sous-section *Passer à la 0.13.3* n'existe au `5aad8b35` » est-elle vraie
   (`grep -nF "0.13.3" docs/manual/fr/admin-manual.tex`) ? La généalogie (`d415e66c` a écrit le paragraphe
   d'intégrations, `e04122dc` a ouvert la sous-section 0.13.2) est-elle exacte
   (`git log --format='%h %s' -S'Intégrations par l' -- docs/manual/fr/admin-manual.tex`, idem avec
   `-S'Passer à la 0.13.2'`) ? La prescription conditionnelle (ligne + `make fr` si la sous-section existe ; sinon
   préparation de release) est-elle sans contradiction avec le reste de la fiche — Exclusions, T5, T7,
   § *Fichiers communs*, § *Découpage préventif* ?
2. **Manuel contre code (obligatoire), `.tex` ET PDF aplati** : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`
   et `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (aplatir avant de greper) — le PDF
   dit-il ce que la fiche lui attribue (`créditrice, en CHF` ; les trois codes de la 0.13.2) ?
3. **AC1, ordre des refus et verrou** (R6, F1) : à `crates/kesh-api/src/routes/reconciliation.rs`, le pré-vol
   `classify_batch_transaction` (`:872-891`) rend-il bien les trois codes nommés, dans cet ordre ? La recharge
   `find_pending_by_id_for_account` (`:1563`) est-elle sans `FOR UPDATE`
   (`crates/kesh-db/src/repositories/reconciliation.rs`, autour de `:374`) ? L'étape 4 (`:1588-1594`) rend-elle
   `RECONCILIATION_ALREADY_RECONCILED` ? Reste-t-il dans la fiche ou le registre un « sous verrou » non précisé qui
   contredirait « sous le verrou du compte » (hors Change Log historique) ?
4. **AC4, quatre tests nommés** (R3, C-28-6-14) : chacun existe-t-il au fichier cité, accepte-t-il un **débit**
   (montant négatif dans son montage), et assert-il un succès qui rougirait si une garde `amount <= 0` était posée
   sur la variante ? `accept_with_explicit_split_type_runs_split_flow` (`crates/kesh-api/tests/reconciliation_e2e.rs`),
   `accept_with_rule_creates_journal_entry_and_increments_count` (`reconciliation_rules_e2e.rs`),
   `post_manual_creates_journal_entry_and_marks_transaction_reconciled` (`reconciliation_manual_e2e.rs`),
   `split_creates_journal_entry_with_n_plus_1_lines` (`reconciliation_split_e2e.rs`, montage `setup_split_ctx`).
   Les lignes `:2447`, `:2870` (par proposition, `VALIDATION_ERROR` + `details.reason`) et `:3659`, `:4109`
   (`400 VALIDATION_ERROR`) disent-elles ce que l'AC4 et l'AC7 leur font dire ?
5. **T1 / AC2, « rien d'écrit »** (R4, F2) : `journal_entry_number_sequences.next_number`, `next_number_for`, l'étape 3
   de `create_in_tx_inner` (`crates/kesh-db/src/repositories/journal_entries.rs`) existent-ils sous ces noms ? Le
   compteur est-il bien écrit dans la transaction du lot (donc annulé par `ROLLBACK TO SAVEPOINT`), et non par une
   connexion à part ? Le montage nommé (« facture validée de 100.00 sans règlement, reste dû > 0 ») est-il celui de
   `seed_validated_invoice` ?
6. **`-0.00` et M8** (F3 = R5, C-28-6-12) : `bank_transactions.amount` est-il `decimal(18,2)`
   (`crates/kesh-db/test-schema/0001_schema_squash.sql`) ? La mutation M8 (`is_sign_negative()` → débit, sinon
   `is_zero()` → nul) est-elle tuée par le cas `dec!(-0.00)` de l'AC3, et seulement si la précondition prescrite est
   tenue ? Le texte prétend-il encore quelque part que `-0.00` atteint la garde par la base ?
7. **AC5, doc-comment frontend** (R2) : les sites cités (`frontend/src/lib/features/reconciliation/failed-proposal-label.ts:9-10`,
   `:25-28` ; `failed-proposal-label.test.ts:5-9`, `:68-69`) portent-ils les textes que la fiche cite ? Un autre
   site du dépôt énonce-t-il « 29 codes » ou « 27 littéraux » hors de ces deux fichiers et de `_bmad-output/`
   (`grep -rnE "29 codes|27 littéraux|les 27" --exclude-dir=node_modules --exclude-dir=target --exclude-dir=_bmad-output .`) ?
8. **Recomptes** écrits par le commit : 9 LOW dont 8 distincts en P1 (relis les deux rapports), 8 mutations (M1-M8),
   14 choix C-28-6-1 à C-28-6-14, 9 AC — recompte depuis les fichiers (`grep -c`), ne relis pas les nombres.
   Vérifie aussi qu'**aucune entrée historique** du Change Log n'a été réécrite (la ligne « Choix C-28-6-1 à
   C-28-6-10 » de l'entrée de création doit être intacte).

⚠️ **Erreur de catégorie** (consigne 5 des agents) : un livrable **prescrit par la fiche mais absent du code** n'est
**pas** un défaut — la fiche est une spec, le code n'est pas encore écrit (« `RECONCILIATION_TRANSACTION_NOT_CREDIT`
n'existe pas dans `reconciliation.rs` », « la ligne n'est pas dans `admin-manual.tex` », « M8 n'a pas été jouée »,
« le test du `-0.00` n'existe pas » sont **attendus**). Un finding ne porte que sur ce que **le texte du commit affirme
du code existant**, ou sur ce qu'il **prescrit de faux, de contradictoire ou d'infaisable**.
⚠️ **Grep ground-truth obligatoire** : toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée ; un
CRITICAL ou HIGH qui affirme qu'un patch « n'a pas été appliqué » se vérifie au `grep -nF` sur le fichier **final**
(pas sur les numéros de ligne d'un hunk) avant d'être écrit.

## Ce que tu rends

Findings numérotés `P2-1…` avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve** (commande et
sortie copiée), correction proposée. ⛔ **La liste des axes exercés ET non exercés** (les huit ci-dessus, chacun
marqué « exercé » avec la commande lancée, ou « non exercé » avec la raison) — un « 0 finding » sans elle ne compte
pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/28-6-validate-p2-ciblee.md` ; dernier message : le
chemin du rapport, le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`,
`latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`,
`cargo`, `npm`, `npx`, `node`, `docker`, **tout interpréteur** (`python`, `python3`, `perl`, `ruby`, `node`, `bash`
ou `sh` sur un script, `awk` qui écrit), `gh` en écriture, toute requête SQL. Rien de `~/kesh-test/` (relevé réel de
Guy). Autorisés : lecture, `grep`, `sed -n`, `wc`, `git log`/`show`/`diff`/`blame`, `gh api` en lecture,
`pdftotext` vers la sortie standard.
