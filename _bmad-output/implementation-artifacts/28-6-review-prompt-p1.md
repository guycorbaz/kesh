# Prompt — revue de code P1, Story 28-6 (le rapprochement refuse de régler une facture client par un débit)

*Versionné le 2026-10-13. Trois lentilles (**Sonnet**), contexte frais chacune, lecture seule.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-28-6`, branche `story/28-6`.
**Le diff à revoir** : `git diff 3d9753fc 289a7fed -- . ':(exclude)_bmad-output'` (`3d9753fc` = étape 0 du
développement, fiche close et rebasée sur `eab77cf1` ; `88ff9d62` = tests rouges (T1) ; `3c3d0a86` = **dernier commit
de code** (garde, écran, catalogue) ; `289a7fed` = `docs/api-external.md` et `CHANGELOG.md` ; les commits suivants ne
portent que la fiche, le registre des choix, le registre de sprint et ce prompt). ⚠️ Pathspec `':(exclude)…'` et
**non** `':!…'`, que le shell peut interpréter. Fiche : `_bmad-output/implementation-artifacts/28-6-rapprochement-debit-facture.md`
— AC1 à AC9 ; tâches T0-T7 ; § *Mutations* (M1-M8) ; Dev Notes (inventaire clos des chemins qui lient une
transaction à une facture, inventaire des sites non résolus, « ce qui doit être préservé », fichiers communs avec la
28-5 et la 28-7, pièges, exclusions) ; et le **Dev Agent Record** (rouge d'abord et code observé, campagne de
mutations, gates finaux, décomptes). Choix C-28-6-1 à C-28-6-15 au registre `epic-28-choix-autonomes.md`. Règles :
`CLAUDE.md` du dépôt — § *Pattern batch — FailedProposal per-proposal*, § « Recompter ses propres comptes rendus »,
§ « Propagation post-patch », § « Inventorier les sites NON RÉSOLUS », § « Le prompt d'une passe doit NOMMER le
manuel », § « Une passe qui ne déclare pas ses axes exercés ne compte pas », § « Migration breaking policy » (aucune
migration attendue). Issue : #482 — la PR la fermera (`closes #482` au titre ou au corps de la **PR** seulement).

⛔ **DONNÉES PERSONNELLES — DÉPÔT PUBLIC.** N'ouvre aucun fichier hors du dépôt et de
`/home/gcorbaz/devel/kesh-gate-logs/` ; rien sous `~/kesh-test/`.

**Contexte** : `kesh-api` : `routes/reconciliation.rs` — fonction pure `refuse_non_credit_settlement` au-dessus
d'`accept_one_invoice`, son appel à l'étape 4bis (entre le statut `pending` et la lecture de la facture), une ligne au
doc-comment d'`accept_one_invoice`, un commentaire au filtre de `get_proposals`, `mod non_credit_settlement_tests` en
fin de fichier (AC3) ; `tests/reconciliation_e2e.rs` — `post_accept_filters_signed_amount` réécrit (égalité stricte,
AC2-3) et une section neuve en fin de fichier (AC2-1, -2, -4, -5, auxiliaires `*_286`). `kesh-i18n` : une clé × quatre
catalogues. `frontend` : `failed-proposal-label.ts` (un `case`, doc-comment 29/27 → 30/28), son test
(`EXPECTED_KEYS`, 29 → 30), `i18n-keys.test.ts` (`sitesTotal` 2079 → 2080). `docs/api-external.md`, `CHANGELOG.md`.
Aucun fichier de `kesh-db`. La 28-5b travaille en parallèle sur l'écran du détail d'import ; la 28-4 ajoutera une
garde « compte clos » : leurs régions ne sont pas dans ce diff.

⛔ **Axes communs aux trois lentilles** (chacune les déclare exercés ou non) :

1. **Place et ordre des refus.** La garde lit-elle la transaction **rechargée sous le verrou du compte**
   (`find_pending_by_id_for_account`), jamais l'instantané du pré-vol ? Est-elle **après** l'étape 4 (statut) et
   **avant** l'étape 5 (facture) ? L'ordre complet — pré-vol (`BANK_TRANSACTION_NOT_FOUND`,
   `RECONCILIATION_ALREADY_RECONCILED`, `RECONCILIATION_ALREADY_REJECTED`), puis sous verrou recharge, statut,
   **sens**, facture, éligibilité, score — est-il celui de l'AC1 ? Un débit déjà rejeté rend-il bien
   `RECONCILIATION_ALREADY_REJECTED` ?
2. **La règle et sa forme.** `amount <= Decimal::ZERO` (comparaison, non test de signe) ; `debit_transaction` pour
   `< 0`, `zero_amount_transaction` pour `= 0` ; `-0.00` classé nul ; `transactionAmount` de la même forme que celui
   de `RECONCILIATION_OVERPAYMENT` ; code écrit au site sous la forme `error_code: "…".to_string()` (28 littéraux par
   `grep -ohE 'error_code: "[A-Z_]+"' crates/kesh-api/src/routes/reconciliation.rs | sort -u | wc -l`).
3. **Pattern batch.** Le refus est un `Err(FailedProposal)` ordinaire : le point de sauvegarde de la proposition
   est-il relâché par le chemin ordinaire d'`accept_batch`, les autres propositions traitées, le `200` conservé ?
4. **Ce qui ne change pas (AC4).** `accept_one_split`, `accept_one_rule`, `post_manual`, `post_split`,
   `get_proposals` (hors son commentaire), `classify_batch_transaction`, `classify_payment`,
   `settlement_journal_lines`, le bras `_` → `DATABASE_ERROR` de `project_error_to_failed_proposal` : aucune ligne
   de code changée ? Les quatre tests nommés par l'AC4 (`accept_with_explicit_split_type_runs_split_flow`,
   `accept_with_rule_creates_journal_entry_and_increments_count`,
   `post_manual_creates_journal_entry_and_marks_transaction_reconciled`,
   `split_creates_journal_entry_with_n_plus_1_lines`) sont-ils **intouchés** (`git diff 3d9753fc 289a7fed --
   crates/kesh-api/tests/`) ?
5. **Inventaire des sites non résolus** (Dev Notes, D4-ter) : est-il encore **clos** au `289a7fed` ? Méthode de la
   fiche (`grep -rn "matched_entry_id\s*=" crates/*/src --include=*.rs`, `grep -rn
   "invoice_settlements::create_in_tx\|INSERT INTO invoice_settlements" crates/*/src --include=*.rs`, …), à
   **élargir** si une graphie manque. Un chemin qui écrit un règlement de facture à partir d'une transaction bancaire
   sans passer par la garde existe-t-il ?
6. **Décomptes.** **Recompte depuis la source** les décomptes du Dev Agent Record : tests neufs
   (`git show <ref>:crates/kesh-api/tests/reconciliation_e2e.rs | grep -c '#\[sqlx::test'` et
   `git show <ref>:crates/kesh-api/src/routes/reconciliation.rs | grep -c '#\[test\]'` aux **deux** bornes
   `3d9753fc` et `289a7fed`), fichiers du `git diff --stat`, 30 codes / 28 littéraux du frontend, `sitesTotal` 2080.
   Le **périmètre** de chaque nombre est-il déclaré ? Les chiffres des gates correspondent-ils aux journaux
   (`kesh-gate-logs/28-6-gates.txt`, `28-6-gate-testfast.log`, `28-6-gate-front-*.log`, `28-6-gate-e2e.log`,
   `28-6-e2e-backend.log.build`, `28-6-t1-rouge*.log`, `28-6-t2-vert.log`, `28-6-mutations.txt`,
   `28-6-mutations/`) ?
7. **Catalogue et libellé.** La clé dans les **quatre** catalogues, au bloc `reconciliation-failed-*`, après
   `score-too-low` et avant `transaction-not-pending` ; textes de l'AC6 mot pour mot ; repli français de
   `failed-proposal-label.ts` **égal** à la ligne fr-CH (`grep -nF "n’est pas une entrée d’argent"
   frontend/src/lib/features/reconciliation/failed-proposal-label.ts crates/kesh-i18n/locales/fr-CH/messages.ftl`) ;
   apostrophe typographique ; terminologie du glossaire (`docs/i18n-glossaire.md`).
8. ⛔ **LA DOCUMENTATION — `docs/api-external.md`, LE CHANGELOG, ET LES MANUELS.** La phrase ajoutée au § `POST
   /api/v1/reconciliation/accept` dit-elle **vrai au code** (code, `reason`, `transactionAmount`, place avant la
   facture et après le pré-vol, `split`/`rule` inchangés, divergence du montant nul) ? Le code « auparavant » qu'elle
   cite est-il bien celui **observé** au rouge (`kesh-gate-logs/28-6-t1-rouge.log`, `28-6-t1-rouge-2.log`), sans le
   présenter comme le seul possible ? « *depuis la v0.13.3* », jamais « Unreleased » ni « à venir ». La puce du
   CHANGELOG est-elle dans la section `## [0.13.3] — Non publié` existante, sans l'écraser ? **Manuels** : le manuel
   utilisateur n'est **pas** modifié (AC9) — cherche dans `docs/manual/fr/user-manual.tex` **et son PDF** une phrase
   que la story rendrait fausse (ce que dit le manuel du rapprochement d'une transaction de débit, d'un refus « erreur
   de la base de données », des factures cherchées pour une transaction « créditrice, en CHF ») ; le manuel
   d'administration n'a pas de sous-section « Passer à la 0.13.3 » au rebase, et sa ligne revient à la préparation de
   release (C-28-6-11) : vérifie cette absence, et qu'**aucune** phrase de `admin-manual.tex` ou de son PDF (le
   paragraphe « Intégrations par l'API : les lots de réconciliation » de la 0.13.2) ne devient fausse ; idem README et
   `website/`. Contrôle les **PDF aplatis et normalisés** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | sed "s/’/'/g; s/ﬀ/ff/g; s/ﬁ/fi/g; s/ﬂ/fl/g; s/ﬃ/ffi/g; s/ﬄ/ffl/g"`
   (et de même `admin-manual.pdf`) vers `/home/gcorbaz/devel/kesh-gate-logs/28-6-review-p1-<lettre>-<manuel>-flat.txt`.

## Lentilles

- **B — Blind Hunter : bugs et états**, sans présupposer que la fiche a raison. Le type et l'échelle de
  `bank_transaction.amount` à la relecture (`DECIMAL(18,2)` → `Decimal`) ; une transaction non CHF qui serait aussi
  un débit (quel refus gagne, et est-ce un défaut ?) ; l'effet de la garde sur une proposition `invoice` dont la
  transaction a changé de signe entre la proposition et l'acceptation ; la sérialisation de `transactionAmount`
  (chaîne ou nombre, `-100.00` ou `-100`) et son égalité avec celle de `RECONCILIATION_OVERPAYMENT` ; l'interaction
  avec le rejeu sur interblocage (`retry_with`) ; un appelant de `refuse_non_credit_settlement` autre
  qu'`accept_one_invoice` ; le libellé frontend pour les deux `reason`.
- **E — Edge Case Hunter : bords, et les tests prouvent-ils ce qu'ils nomment.** Rejoue mentalement M1-M8 (journal
  `kesh-gate-logs/28-6-mutations.txt`, sorties dans `kesh-gate-logs/28-6-mutations/`) et cherche une **mutation
  plausible qui resterait verte** (ex. : la garde appliquée à l'instantané du pré-vol ; `reason` dérivé de
  `is_zero()` d'abord ; `transactionAmount` remplacé par `amount.abs()` ; `bank_transaction_id` remplacé par
  `invoice_id` ; garde placée **avant** l'étape 4 — un débit déjà rapproché rendrait-il alors le code neuf, et un test
  le verrait-il ?). Le constat « rien d'écrit » (`Etat286`) prouve-t-il ce qu'il nomme : le compteur
  `journal_entry_number_sequences` est-il **exercé** (une ligne existe-t-elle avant, ou la comparaison passe-t-elle
  à vide de `[]` à `[]`, et le `ROLLBACK TO SAVEPOINT` serait-il vu s'il manquait) ? Les traces sont-elles comptées
  par société ou globalement, et cela compte-t-il dans une base éphémère ? AC2-4 : l'état du débit est-il comparé
  champ par champ, et l'encaissement prouvé rapproché ? AC2-5 : `inv_id + 100_000` est-il garanti inexistant ? Le zéro
  négatif de l'AC3 (`set_sign_negative`, C-28-6-15) : la précondition rend-elle le cas non vide ? Indépendance et
  rejouabilité des tests (noms de société, IBAN, hash).
- **A — Acceptance Auditor : conformité, décomptes, documentation.** Chaque AC (AC1-AC9) contre le code : satisfait,
  partiel, absent. Les tests nommés par la fiche (AC2-1 à -5, AC3) existent-ils et prouvent-ils ce qu'ils nomment ?
  Les régions touchées sont-elles celles du tableau *Fichiers communs* (recalé au `eab77cf1` à l'étape 0) — et
  aucune région de la 28-5a (section `unreject`, `conclude_locked_attempt`) ni de la 28-7 (étape 6 d'`accept_one_invoice`)
  n'est-elle touchée ? Aucun fichier de `migrations/`, de `test-schema/`, de `kesh-db`, ni `post_restore.rs`, ni
  `Cargo.toml`, ni `docs/manual/` touché (`git diff --stat`). Les choix C-28-6-1 à C-28-6-15 : justifiés, ou défaut ?
  Les deux LOW de la validation P2 (P2-1, P2-2) sont-ils appliqués (`grep -nF "jamais d'un débit"` sur la fiche,
  `grep -nF "Amendé en validation P1 (F1, R6)"` sur le registre) ?

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/28-6-review-p1-<B|E|A>.md` (ta lettre). Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction proposée. Pour
tout finding affirmant qu'un code ou un texte est absent ou présent : la sortie d'un `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** (les huit axes communs et ceux de ta lentille) — un « 0 finding » sans
elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` (et les textes aplatis des PDF
au même endroit) ; aucune commande qui écrit dans le dépôt ou dans une base, ni qui compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`,
`xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/`rebase`/`restore`/`push`/`tag`, `sqlx`, `cargo`
(aucune sous-commande), `rustc`, `npm`, `npx`, `node`, `deno`, `bun`, `python`/`python3`, `perl`, `ruby`, `php`,
`sed -i`, `awk` en écriture, tout autre interpréteur, `mariadb`, `mysql`, `docker`, `curl` et `wget` (aucune requête
HTTP), `gh` en écriture, `pkill`/`kill`, aucune requête SQL. Aucun fichier hors du dépôt. Autorisés : lecture, `grep`,
`sed -n`, `wc`, `sort`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers
`/home/gcorbaz/devel/kesh-gate-logs/`.
