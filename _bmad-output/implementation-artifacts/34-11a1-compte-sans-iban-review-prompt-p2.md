# Prompt — revue de code P2 de la Story 34-11a1 (compte sans IBAN, #972), trois lentilles, passe complète, Sonnet

Tu relis la **Story 34-11a1** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.6**, Epic 34,
**après la remédiation de sa revue P1** (0 CRITICAL, 0 HIGH, 3 MEDIUM, 21 LOW). La story porte **la seule migration de la release, breaking**
(`20261110000001_bank_accounts_without_iban`), l'entité (`iban: Option<String>`, `BankAccountKind`, `card_last4`), les refus nommés
(`409 PAYMENT_BATCH_ACCOUNT_HAS_NO_IBAN`, `422 BANK_IMPORT_CAMT_NEEDS_IBAN`), l'empreinte d'aperçu à IBAN `null`, l'audit `iban_present` lu,
l'export `bank_accounts.csv`, la QR-facture sans panique, et la tolérance de l'écran. Aucune route ne crée encore de compte sans IBAN (34-11a2).

Worktree `/home/gcorbaz/devel/kesh-e34a1`, branche `story/34-11a1-compte-sans-iban`. Deux diffs, **aplatis** :
- la story entière : `git -C /home/gcorbaz/devel/kesh-e34a1 diff 07f714a2 75ab21ae` (`07f714a2` = base ; `75ab21ae` = dernier commit de code, la
  remédiation) ;
- **la remédiation seule**, à relire ligne à ligne : `git -C /home/gcorbaz/devel/kesh-e34a1 diff 6bf20de8 75ab21ae` (`6bf20de8` = compte rendu de la P1).

Langue du rapport : français. Modèle : **Sonnet**. Passe **complète**. Contexte frais.

⚠️ **Ce que la remédiation a changé, à éprouver d'abord** (la sévérité se déplace vers ce qu'on vient d'écrire) :
1. **E1** — la migration, **modifiée en place** (non publiée) : `CHECK (BINARY account_kind IN ('bank', 'credit_card'))`. Le squash régénéré porte
   `cast(\`account_kind\` as char charset binary) in (…)` : est-ce bien l'équivalent, et le `test_schema_guard` le compare-t-il ? Les deux autres `CHECK`
   (`account_kind <> 'bank'`, `<> 'credit_card'`), qui comparent sans `BINARY` sous `utf8mb4_unicode_ci`, sont-ils exacts **parce que** le domaine l'est ?
   `card_last4` sous la même collation : le raisonnement écrit (REGEXP exact, espace finale retirée par `CHAR(4)`) est-il juste ? Le registre
   `crates/kesh-db/migrations.sha384` porte-t-il le **nouveau** checksum ? P8 : la migration change après une première application sur les bases de dev :
   est-ce annoncé, et aucune base persistante ne garde l'ancien ?
2. **A1** — `\sloppy` … `\fussy` autour de « Passer à la 0.17.6 » (`docs/manual/fr/admin-manual.tex`) : le PDF aplati porte-t-il
   `KESH_SKIP_PRE_UPGRADE_BACKUP=0.17.6 dans` entier ? `\fussy` est-il rétabli **avant** la section suivante, et rien d'autre n'en est affecté ?
3. **A2/E4** — `full_import_round_trip_keeps_a_card_without_iban` et `full_import_of_a_0_17_5_backup_makes_every_account_a_bank_account`
   (`admin_full_import_e2e.rs`) : prouvent-ils l'aller-retour colonne par colonne, ou passent-ils à vide (par exemple : la modification entre
   l'export et l'import touche-t-elle une ligne ; l'assertion compare-t-elle les colonnes neuves) ?
4. **Les témoins neufs** : `compte_sans_iban_le_refus_camt_precede_l_empreinte` (le `422` vient-il bien du refus **précoce** de `create`, et non de celui
   de `create_camt_under_lock` ?), `compte_sans_iban_archive_ou_clos_garde_son_refus_d_abord`, `create_batch_refuses_a_closed_or_archived_card_by_its_state_first`,
   `create_accepts_an_account_without_iban` (`bank_statements_manual_e2e.rs`), `compte_sans_iban_le_put_ignore_le_type`, les Vitest neufs de
   `BankImportUpload.test.ts` et de `format.test.ts` (l'espace insécable U+00A0 de `cardLast4Label`).

Lis d'abord (lentilles E et A ; la lentille B lit les diffs avant) : la fiche `_bmad-output/implementation-artifacts/34-11a1-compte-sans-iban.md`
en entier (inventaire, AC-1 à AC-13, mutations, **Change Log** — dont l'entrée de la remédiation — et **Dev Agent Record**) ; les registres
`e34-choix-autonomes-e341.md` et `e34-choix-autonomes-e34a1.md` (C-e34a1-1 à 11) ; les rapports P1
`/home/gcorbaz/devel/kesh-gate-logs/e34a1-review-p1-{B,E,A}.md` (chaque finding est-il traité, ou renvoyé par écrit — E5/B7, B5, E6 à la fiche
`34-11a2-carte-credit-creation.md`, § *Points hérités*) ; la fiche 34-11a2 pour ce paragraphe seulement. Recoupe le Dev Agent Record avec les
journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e34a1/` : `cible-r1.log`, `vitest-cible-r1.log`, `gate-test-fast-r1.log`,
`build-r1.log`, `gate-ci-r1.log`, `gate-ci-journal-r1.log`, `front-check-r1.log`, `front-lint-i18n-r1.log`, `front-unit-r1.log`, `front-build-r1.log`,
`e2e-r2/playwright.log`, `mut/campagne-r1-rust.log`, `mut/campagne-r1-front.log` et `mut/M*.log` (les scripts `mut-e34a1.py` et
`mut-campagne-e34a1.sh` sont à **lire**, jamais à exécuter), `recette-montee-r1.log` (crochet en échec : QR-IBAN d'exemple, voir le Dev Agent Record)
et `recette-montee-r2.log`, `recette-montee-journal-r2/` (dont `apres.log`), le crochet `apres-sans-iban.sh`, `recette-complement-e34a1.sh` et
`recette-c/resume.txt` (`recette-complement-r1.log`), `manuels-4.log`, `flat-admin-manual.txt`. **Captures** (thème clair) sous
`/home/gcorbaz/devel/kesh-gate-logs/e34a1/captures/` : `34-11a1-01-tableau-comptes.png` à `34-11a1-05-lot-sans-la-carte.png`.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part ; **aucune
  redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ; ni
  **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`, `scripts/mem-guard.sh`,
  `scripts/regen-test-schema.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`,
  `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep`, `sed -n`, `awk` (sans redirection ni `system()`),
  `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, `sha384sum` (lecture d'un fichier du dépôt),
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Les axes obligatoires, pour chaque lentille

1. **La migration et P1 à P8 du `CLAUDE.md`** (axe obligatoire) : P1/P3 bump en **dernière instruction** ; P2-bis (`git show --stat 194faa78`) ;
   P4 ; P5 — compteurs **recomptés** (`ls crates/kesh-db/migrations/*.sql | wc -l`, `/usr/bin/grep -c '^| \`20' docs/migrations-idempotence-audit.md`,
   partitions recomptées au tableau) et ligne d'audit à jour de `BINARY` ; P6 — `git grep -n "migrations.len()\|apply_migrations_up_to" crates/` ;
   P7 — l'exemption et sa justification, prouvée par le test E4 ; P8 — `sha384sum` de la migration contre `migrations.sha384`.
2. **Le grep du symptôme, la valeur** : `git grep -nE "account_kind\s+(IN|in)\b" -- crates/ docs/` (un `IN` sans `BINARY` resté ailleurs ?) ;
   `git grep -nF "avant toute lecture" -- crates/kesh-api/src/routes/bank_imports.rs crates/kesh-api/src/errors.rs docs/api-external.md` ;
   `git grep -nF "•••• " -- frontend/src` (une forme à espace sécable restée hors de `cardLast4Label`) ; `git grep -nE "\b0\.17\.6\b" -- crates/ docs/ CHANGELOG.md`.
3. **L'inventaire clos des sites (S1-S18)** : la remédiation a-t-elle ajouté un lecteur de l'IBAN d'un compte, ou un `unwrap` ?
4. **Les manuels et la documentation contre le code** : `admin-manual.tex` et le **PDF** aplati (§ *Passer à la 0.17.6*, *Compatibilité*) ;
   `docs/api-external.md` (note ¹¹ du § 7 — l'ordre des refus du CAMT désormais écrit : archivé `404`, clos `409`, format, puis IBAN — ; § 10, la ligne
   `422` rangée) ; `CHANGELOG.md` § `[0.17.6] — Non publié`.
5. **Les témoins et les mutations** : M1 à M13 (M9 à M13 neuves) — chacune tuée par le test attendu (`mut/M*.log`) ? Mutations plausibles non jouées à
   proposer (par exemple : `CHECK` à `BINARY` sur `chk_bank_accounts_card_shape` seulement ; `CARD_LAST4_SEPARATOR` remis en espace ordinaire ;
   l'aller-retour de sauvegarde qui ne compare plus `card_last4`).

## Lentille B — Blind (les diffs seuls, sans la fiche)

Relis ligne à ligne **la remédiation** (`6bf20de8..75ab21ae`), puis le reste de la story. Cherche : un test qui passe **à vide** ; une assertion qui ne
porte pas sur ce que son nom promet ; un `\sloppy` qui fuit ; une fixture qui ne contient pas le cas ; une modification d'un fichier de migration qui
change autre chose que le `CHECK`.

## Lentille E — Edge (chaque branche, chaque borne)

`account_kind` à `'bank'` suivi d'une tabulation, à `'bank\0'`, à la chaîne vide ; `card_last4` `'0000'`, `'12 4'`, Unicode ; une carte archivée **et**
close ; une carte à `journal_account_id` nul au relevé saisi ; une sauvegarde 0.17.6 réimportée dans une base qui porte déjà une carte de même `id` ; un
`.keshbackup` retouché qui porte `account_kind = 'BANK'` (l'import le refuse-t-il **proprement**, sans laisser la base à moitié restaurée ?). Chaque cas :
comportement au code, test qui le couvre (ou absence, prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche et la P1 contre le livré)

Chaque finding P1 (E1 à E7, B1 à B7, A1 à A10) : traité, au lieu dit, avec un témoin, ou renvoyé par écrit ? Chaque AC-1 à AC-13 : livré, testé ? Les
gates du Dev Agent Record ont-ils tourné **au dernier commit de code** (`75ab21ae`), E2E compris (A3 de la P1) ? La recette de montée rejouée (la
migration a changé), son crochet (lot et QR-facture après la montée, A4) et son complément (conteneurs restants : A5) ? Le Dev Agent Record n'affirme-t-il
que ce qui a tourné, et ses décomptes sont-ils recomptés (motif nommé, A6) ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e34a1-review-p2-B.md`, `/home/gcorbaz/devel/kesh-gate-logs/e34a1-review-p2-E.md`,
`/home/gcorbaz/devel/kesh-gate-logs/e34a1-review-p2-A.md`. Chaque rapport : findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH /
MEDIUM / LOW), lieu (`fichier:ligne`), ligne lue copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non exercés** ; le décompte
final **recompté depuis la liste**.
