# Prompt — revue de code P1 de la story 43-6 (#823, #1167), trois lentilles, première passe complète

Tu relis la story **43-6** de Kesh (comptabilité suisse ; backend Rust Axum + SQLx/MariaDB, frontend SvelteKit 5), release
**0.19.0** : **« Changer le compte analytique » (geste de la 42-1) est refusé sur une facture client réglée avec d'autres par
un même virement** (règlement groupé de la 43-1a). Code `409 PROJECT_REASSIGNMENT_GROUPED_SETTLEMENT`,
`details { invoiceId, journalEntryId }`, levé dans `reassign_invoice` à l'étape (3) — la lecture verrouillante des
règlements —, après le contrôle de version et avant le « rien à faire », le compte visé et la période close. Message traduit
côté serveur (quatre locales, repli Rust), aucun code neuf à l'écran, aucune migration, aucune route neuve.

Worktree `/home/gcorbaz/devel/kesh-43-6`, branche `story/43-6-raccord-reimputation`, partie de `release/0.19.0`
(`753baf088`). Diff à relire, **aplati** : `git -C /home/gcorbaz/devel/kesh-43-6 diff ff79da71c 8a75833ec`
(`ff79da71c` = fiche validée, code d'origine ; `8a75833ec` = dernier commit de code ; le commit suivant ne porte que la
fiche, le registre, ce prompt et `sprint-status.yaml`). Langue du rapport : français. Modèle : **Opus ou Sonnet**
(première passe : jamais Haiku). Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/43-6-raccord-reimputation.md` en entier (Décision, AC1 à AC7, Tests et mutations,
Dev Notes, **Dev Agent Record**, **Change Log**) ; le registre `_bmad-output/implementation-artifacts/e0190-choix-autonomes-e436.md`
(C-e436-1 à C-e436-6) ; l'épique `_bmad-output/planning-artifacts/epic-43-transaction-plusieurs-factures.md`, décision C-43-11 ;
le dernier rapport de validation `/home/gcorbaz/devel/kesh-gate-logs/e436/validation-p2.md`.
Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec les journaux hors dépôt (lecture seule) sous
`/home/gcorbaz/devel/kesh-gate-logs/e436/` : `rouges.log`, `mut/` (un journal par mutation, `muter-e436.py`,
`run-rust-e436.sh`), `cible-1.log`, `test-fast-reprise.log`, `gate-ci-reprise.log`, `front-*-reprise.log`,
`e2e-reprise.log`, `reprise-prompt.md` (la station a planté pendant le développement). **Capture** (thème clair, base
jetable de l'E2E) : `/home/gcorbaz/devel/kesh-gate-logs/e436/captures/43-6-refus-groupe.png` — regarde-la si ton outil lit
les images.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni `pkill`, ni `killall`, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`,
  `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`,
  `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` (ils se **lisent**) ; ni `make`, `latexmk`, `cargo`,
  `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk`
  (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, et, pour un PDF, la
  seule forme `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
- ⚠️ `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport ; tout chemin
  cité se vérifie (`git cat-file -e <ref>:<chemin>` ou `ls`).

## Les axes obligatoires, pour chaque lentille

1. **Le critère.** `journal_entry_line_id IS NOT NULL` sur un règlement de la facture désigne-t-il exactement le règlement
   groupé ? Inventaire de l'ensemble clos des sites qui écrivent `invoice_settlements` (`git grep -n "invoice_settlements::create_in_tx\|INSERT INTO invoice_settlements" -- crates`) :
   lequel écrit une ligne non nulle, lesquels `None` ? Un règlement groupé peut-il exister sans ligne (import d'installation,
   `post_restore.rs`, migration `20261118000001`) ? Une facture fournisseur peut-elle atteindre ce geste ?
2. **La position.** Le refus tombe-t-il après la `version` (étape 2) et avant toute lecture des avoirs, `entry_ids_in_tx`,
   `lock_entries_and_lines_in_tx` ? Aucune ligne des autres factures n'est-elle verrouillée ? `.min()` sur un itérateur filtré :
   `details.journalEntryId` est-il l'écriture du groupe ? `base_ids` reçoit-il toujours tous les règlements quand il n'y a pas
   de refus (aucune régression de la 42-1) ?
3. **Les verrous et la concurrence** : la lecture `FOR UPDATE` gagne une colonne ; un dé-rapprochement concurrent
   (`grouped_settlements::cancel_in_tx`) ou une acceptation de groupe concurrente change-t-il ce que le geste voit ? Ordre des
   verrous du module (doc :17-44) inchangé ?
4. **Le mappage d'erreur** (`crates/kesh-db/src/errors.rs`, `crates/kesh-api/src/errors.rs`) : statut, code, `details`, message
   par `t(…)` avec repli égal à la valeur fr-CH ; garde G9 (`les_replis_rust_suivent_le_catalogue`) ; G25 (vocabulaire
   « compte analytique » : ni « projet », « Projekt », « progett », « project » dans les quatre valeurs).
5. **Les catalogues** : la clé dans les quatre `messages.ftl` ; aucune autre ligne touchée (comparer à `ff79da71c`) ; traductions
   fidèles au français.
6. **L'écran** : `ProjectReassignDialog.svelte` inchangé ; affiche-t-il bien `err.message` pour ce code, sans avertissement ni
   *Confirmer* ? Le test Vitest neuf et l'E2E neuf de `project-reassignment.spec.ts` (valeur réelle du serveur) ; l'extraction
   de `createManualCreditStatementViaApi` dans `api-fixtures.ts` laisse-t-elle `reconciliation-group.spec.ts` inchangé dans
   son comportement ? `sitesTotal` (`frontend/src/lib/shared/i18n-keys.test.ts`) inchangé ?
7. **Les tests et les mutations** : chaque AC (AC1, AC3, AC4 en ses trois volets, AC5) a-t-il un test Rust qui peut rougir
   (`rouges.log`, `mut/M1` à `M5-bis`, `M2c` ajoutée à la reprise) ? Les mutations déclarées par la fiche (« Tests et
   mutations ») sont-elles toutes jouées ? AC2 : les tests de la 42-1 sont-ils inchangés (`git diff ff79da71c 8a75833ec --
   crates/kesh-api/tests/project_reassignment_e2e.rs` ne doit qu'ajouter) ? Le montage du groupe (`grouped_pair`) est-il
   celui de la production ?
8. **Le manuel** (`docs/manual/fr/user-manual.tex`, § `sec:reimputation-analytique`, puce de la facture réglée en groupe ; §
   règlement groupé, paragraphe *Limites*) et `docs/api-external.md` (tableau de la route, ordre des refus, phrase du `200` ;
   tableau général des codes) : disent-ils ce que fait le code ? Les deux sites du manuel sont-ils cohérents entre eux ? Le
   **PDF n'est pas régénéré dans la branche** (règle des fichiers communs) : contrôle à la release, forme
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'réglée avec d'` — dans la branche
   le PDF ne porte pas encore la phrase, c'est attendu ; le dire, ne pas le compter comme défaut.
9. **La fiche et le registre** : Note de version, Dev Agent Record (décomptes recomptés, gates cités à leurs journaux), File
   List complète (`git diff --stat ff79da71c 8a75833ec`), choix C-e436-5 et C-e436-6.

## Lentilles

- **B — Blind Hunter** : le diff seul d'abord (sans la fiche), défauts de code, de course et d'état ; puis les axes.
- **E — Edge Case Hunter** : toutes les branches (axes 1 à 4 surtout) : facture sans règlement, plusieurs règlements dont un
  seul groupé, brouillon, facture annulée, avoir, groupe dé-rapproché puis re-groupé, clé d'API.
- **A — Acceptance Auditor** : chaque AC et chaque décision de la fiche contre le code, les tests et les journaux (axes 5 à 9).

## Rapport

Un rapport par lentille : `/home/gcorbaz/devel/kesh-gate-logs/e436/review-p1-B.md`, `…-E.md`, `…-A.md`. Pour chaque finding :
identifiant (`B1`, `E1`, `A1`…), sévérité (CRITICAL, HIGH, MEDIUM, LOW), fichier et ligne, extrait cité, scénario, correctif
proposé. Verdict chiffré en tête. **Liste des axes exercés ET non exercés** (un « 0 finding » sans elle ne compte pas). Ce qui
n'a pas tourné reste écrit : la revue n'exécute rien ; voir le Dev Agent Record pour les gates qui ont tourné au passage en
revue.
