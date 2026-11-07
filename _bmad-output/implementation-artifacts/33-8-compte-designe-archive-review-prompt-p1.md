# Prompt — revue de code P1 de la Story 33-8, trois lentilles (B, E, A), Sonnet ou Opus

Tu relis la **Story 33-8** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5, MariaDB 10.11), version **0.17.5**, sans migration :
**un compte désigné dans les paramètres de facturation ne s'archive plus sans avertissement (#486)**, et sa seconde face : un compte archivé n'y verrouille plus les réglages. Kesh tient des livres réels.

Worktree `/home/gcorbaz/devel/kesh-e335`, branche `story/33-8-compte-designe-archive`. Relis par `git show`/`git diff`, jamais le fichier sur disque si la branche a changé. Diff **aplati** à relire :
`git -C /home/gcorbaz/devel/kesh-e335 diff bdc3faa4 story/33-8-compte-designe-archive`. Passe **complète**, contexte frais, langue du rapport : français.
Rapports : `/home/gcorbaz/devel/kesh-gate-logs/e338-review-p1-{B,E,A}.md`, un par lentille, et rien d'autre.

⚠️ **Les propriétés centrales.**
1. **Avertir, puis laisser faire** (C-e334-16) : le Comptable n'est jamais enfermé. Le refus n'écrit rien et ne consomme pas la version.
2. **L'ordre des refus** (C-e334-29) : sous-comptes actifs, désignation, version. Un écran périmé non confirmé puis confirmé.
3. **La seconde face** : `changed` commande inactif et imputable ; le type reste vérifié, toujours. Une valeur `null` envoyée pour un champ de la nature (`Option<Option<_>>`) ? Quelle différence entre les six champs historiques et les quatre d'écart ?
4. **Ajout hors fiche (C-e334-34)** : l'écran des réglages charge désormais les comptes **archivés compris** (`fetchAccounts(true)`) : qu'aucune liste de ce fichier ne propose un compte archivé qu'elle ne proposait pas, et que `withCurrentAccount`, `withoutAccountIds` et le libellé « (archivé) » tiennent.
5. **L'effet annoncé par le dialogue** : « une écriture qui l'emploie sera refusée, à la validation d'une facture ou à un règlement » : vrai pour chacun des dix réglages ? Le réglage `revenue`, `payable`, les comptes de TVA d'un non-assujetti ?

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant, seule) : la fiche `_bmad-output/implementation-artifacts/33-8-compte-designe-archive.md` en entier, dont le **Change Log** et le **Dev Agent Record** ; le registre `e33-choix-autonomes-e334.md` (C-e334-16 à 18, 29, 34) ;
`crates/kesh-db/src/repositories/accounts.rs`, `company_invoice_settings.rs`, `crates/kesh-db/src/errors.rs`, `crates/kesh-api/src/routes/accounts.rs`, `company_invoice_settings.rs`, `crates/kesh-api/tests/accounts_designated_archive_e2e.rs`, `frontend/src/routes/(app)/accounts/+page.svelte`, `frontend/src/routes/(app)/settings/invoicing/+page.svelte`. Ne prends pas le Dev Agent Record pour argent comptant :
recoupe-le avec les journaux hors dépôt, en lecture seule, sous `/home/gcorbaz/devel/kesh-gate-logs/e335/` : `rouges.log`, `mut/synthese-33-8.log`, `mut/33-8-M*.log` (`mut-33-8.py` est à **lire**, jamais à exécuter), `33-8/*.log`, `e2e-r33-8-2/`.
**Captures** (thème clair) : `/home/gcorbaz/devel/kesh-gate-logs/e335/captures/33-8-dialogue-normal.png`, `33-8-avertissement-admin.png`, `33-8-reglage-archive.png`.

⛔ Ne lis rien sous `~/kesh/`, `~/kesh-test/`, `/home/gcorbaz/travail/`, ni le NAS. Fixtures synthétiques seulement.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part ; **aucune
  redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni `pkill`, ni `killall`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`/`reset`/`rebase`/`push`/`tag`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ; ni
  **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` ; ni `make`,
  `latexmk`, `cargo`, `docker`, `mariadb`, `curl`, `claude`, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`** ; aucun accès réseau.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep` (dans ce shell, `grep` nu est une fonction),
  `sed -n`, `awk` (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Les axes obligatoires, pour chaque lentille

1. **Les manuels (`.tex` et PDF aplati) contre le code** : `docs/manual/fr/user-manual.tex` (§ `sec:plan-comptable-archivage`, le paragraphe *Archiver un compte désigné*) ; le **PDF** `docs/manual/fr/user-manual.pdf`, aplati dans
   `/home/gcorbaz/devel/kesh-gate-logs/e335/user-manual-33-8.txt` ; `docs/api-external.md` (`confirmDesignated`, ordre des refus) ; `CHANGELOG.md`. Le manuel promet-il une protection absente ? Annonce-t-il un trou absent ?
2. **Les chemins** : autres archivages d'un compte (import d'installation, MCP, `kesh-seed`, API par clé), et autres désignateurs (`products.revenue_account_id`, `bank_accounts.journal_account_id`, règles de réconciliation) : tracés, non traités ; le manuel ne doit pas laisser croire le contraire.
3. **Le dépôt** : `designations_of_account` (colonnes, `fetch_all`, société), `archive` et son `company_id` lu, `archive_guarded` et ses rollbacks ; le `DbError` et son code ; un `match` exhaustif oublié.
4. **L'écran** : le dialogue, `archiveDesignated` remis à zéro, le bouton pendant l'envoi, le double envoi, le lien pour l'Administrateur seul, les quatre locales, les captures.
5. **Les tests qui prouvent moins** : un test vert par construction, un montage qui n'exerce pas la branche voulue (les tests d'isolation, d'inventaire, de seconde face), les mutations déclarées (survivante ? équivalente abusive ?).
6. **Les gardes recomptées** : `i18n-keys.test.ts`, `i18n-libelle-en-dur.test.ts`.

## Lentilles

- **B (Blind Hunter)** : le diff seul d'abord, sans fiche ni contexte du projet ; puis le code autour. Que casse ce diff ?
- **E (Edge Case Hunter)** : chaque branche et chaque borne : compte désigné deux fois, compte déjà archivé archivé de nouveau, société sans ligne de réglages, confirmation sur un compte non désigné, `confirmDesignated: false` explicite, autre société, réactivation, course avec un `PUT` des réglages.
- **A (Acceptance Auditor)** : chaque AC de la fiche (1 à 9), T1 à T6, chaque mutation (M1 à M3, M5 à M8), chaque choix du registre : présent, testé, vrai.

## Le rapport

Pour chaque constat : sévérité (CRITICAL, HIGH, MEDIUM, LOW), lieu (`fichier:ligne`), preuve copiée, conséquence, remédiation proposée. Ensuite :
**la liste des axes réellement exercés et de ceux qui ne l'ont pas été** : un « 0 » sans cette liste ne compte pas comme passe. Le décompte par sévérité,
recompté, en tête.
