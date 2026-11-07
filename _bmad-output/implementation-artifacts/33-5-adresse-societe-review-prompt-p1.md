# Prompt — revue de code P1 de la Story 33-5, trois lentilles (B, E, A), Sonnet ou Opus

Tu relis la **Story 33-5** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5, MariaDB 10.11), version **0.17.5**, sans migration :
**corriger l'adresse de la société après l'assistant (#909)**. Une société enregistrée avec un pays inconnu n'avait plus aucun moyen de
produire ses QR-factures. Kesh tient des livres réels.

Worktree `/home/gcorbaz/devel/kesh-e335`, **où plusieurs stories se succèdent** : relis par `git show`/`git diff` sur la branche
`story/33-5-adresse-societe`, jamais le fichier sur disque. Diff **aplati** à relire :
`git -C /home/gcorbaz/devel/kesh-e335 diff bdc3faa4 story/33-5-adresse-societe`. Passe **complète**, contexte frais, langue du rapport : français.
Rapports : `/home/gcorbaz/devel/kesh-gate-logs/e335-review-p1-{B,E,A}.md`, un par lentille, et rien d'autre.

⚠️ **Les propriétés centrales.**
1. **Un pays inconnu est refusé même inchangé** (`validate_required`, C-e334-5) : un succès qui laisserait la QR-facture en panne serait le pire résultat.
2. **`update_in_tx` est un full-replace** : aucun champ non visé n'est perdu, ni par cette route ni par ses deux jumelles. `is_stub` ne bouge pas.
3. **Le `CompanyJson` est un miroir écrit à la main** : `addressCountryKnown` et `addressStructured` arrivent-ils jusqu'à l'écran, y compris par les autres
   routes qui rendent un `CompanyJson` (e-mail, coordonnées, verrou de période) ?
4. **Une société aux seules colonnes de texte libre** : pays vide, préremplissage, PDF, `address` remplacée.
5. **Droits** : routeur Admin, `require_not_pat`, jamais une autre société.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant, seule) : la fiche `_bmad-output/implementation-artifacts/33-5-adresse-societe.md` en entier,
dont le **Change Log** et le **Dev Agent Record** ; le registre `e33-choix-autonomes-e334.md` (C-e334-5 à 7, C-e334-32 s'il existe) ;
`crates/kesh-api/src/routes/companies.rs`, `crates/kesh-api/src/address_input.rs`, `crates/kesh-api/tests/company_address_e2e.rs`,
`frontend/src/routes/(app)/settings/+page.svelte`, `frontend/tests/e2e/company-address.spec.ts`. Ne prends pas le Dev Agent Record pour argent comptant :
recoupe-le avec les journaux hors dépôt, en lecture seule, sous `/home/gcorbaz/devel/kesh-gate-logs/e335/` : `rouges.log`, `mut/synthese-33-5.log`, `mut/M*.log`
(`mut-e335.py` est à **lire**, jamais à exécuter), `33-5/*.log`, `e2e-r33-5/`. **Captures** (thème clair) sous
`/home/gcorbaz/devel/kesh-gate-logs/e335/captures/33-5-*.png` : lecture, édition, refus de pays, avis ; regarde-les si ton outil lit les images.

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

1. **Les manuels (`.tex` et PDF aplati) contre le code** : `docs/manual/fr/user-manual.tex` (§ `sec:coordonnees-emetteur`, l'étape *Coordonnées légales*) et
   `docs/manual/fr/admin-manual.tex` (§ `sec:conformite-audit`, le décompte `137 + 5 + 3 = 145`) ; les **PDF** `docs/manual/fr/user-manual.pdf` et `admin-manual.pdf`,
   lus par `git show` n'étant pas possibles sans redirection : relis le PDF de la branche dans `/home/gcorbaz/devel/kesh-gate-logs/e335/user-manual-33-5.txt` et `admin-manual-33-5.txt` (aplatis du PDF
   commité), et recoupe avec les `.tex`. Le manuel promet-il quelque chose d'absent ? Dit-il qu'on corrige un PDF figé ?
2. **Les chemins d'écriture de l'adresse** : assistant, route neuve, import d'installation ; un autre lecteur du pays (`fetch_country`, la colonne `companies.country`) ?
3. **Les refus** : ordre validation/version, message affiché, 400/409, un corps sans `address` ni `country` (`null`, absent, chaîne vide, minuscules, blancs).
4. **L'écran** : préremplissage, avis, Annuler, 409 suivi d'un second essai, le rôle Comptable, l'état `addressEditing` après un rechargement, les quatre locales, les captures.
5. **Les tests qui prouvent moins** : un test vert par construction, un montage qui n'exerce pas la branche voulue, les mutations déclarées (survivante ? équivalente abusive ?).
6. **Le registre** `audit_route_registry.rs` et `admin_pat_denied_e2e.rs` : chaque compteur recompté.

## Lentilles

- **B (Blind Hunter)** : le diff seul d'abord, sans fiche ni contexte du projet ; puis le code autour. Que casse ce diff ?
- **E (Edge Case Hunter)** : chaque branche et chaque borne : longueurs exactes (70/71, 16/17, 35/36), caractères multioctets, pays `ch`, ` ch `, `C`, `CHE`, `` ; société à pays vide ; adresse identique ; version périmée et adresse invalide ensemble.
- **A (Acceptance Auditor)** : chaque AC de la fiche (1 à 10), T1 à T6, chaque mutation (M1 à M5), chaque choix du registre : présent, testé, vrai.

## Le rapport

Pour chaque constat : sévérité (CRITICAL, HIGH, MEDIUM, LOW), lieu (`fichier:ligne`), preuve copiée, conséquence, remédiation proposée. Ensuite :
**la liste des axes réellement exercés et de ceux qui ne l'ont pas été** : un « 0 » sans cette liste ne compte pas comme passe. Le décompte par sévérité,
recompté, en tête.
