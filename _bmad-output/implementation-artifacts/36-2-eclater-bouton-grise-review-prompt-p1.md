# Prompt — revue de code P1 de la Story 36-2 (#1150, #1151), trois lentilles, première passe complète

Tu relis la **Story 36-2** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.8**,
Epic 36, phase 1 : **dans la fenêtre *Éclater* de la réconciliation, le bouton s'active quand les lignes sont complètes et
équilibrées, sinon le message dit ce qui manque, et l'éclatement passe réellement depuis l'écran**. Le montant d'une ligne
devient un champ texte lu par la grammaire du journal (`readJournalAmount` de la 36-3, importée, non modifiée) avec une
**garde du centime** ; il part en **chaîne décimale à deux décimales** (il partait en nombre JSON, refusé `422` par
l'extracteur `serde-str`) ; somme et comparaison en `Big` ; l'indicateur et le message disent l'attendu et l'écart **depuis le
catalogue** (clés neuves ; `reconciliation-split-error-imbalance`, message de l'API, intacte) ; le message porte
`split-client-error` ; la fenêtre passe à `sm:max-w-4xl` avec des largeurs de colonnes (#1151). **Le serveur ne change pas** ;
sous `crates/`, seuls les quatre `messages.ftl`. **Aucune migration.**

Worktree `/home/gcorbaz/devel/kesh-36-2`, branche `story/36-2-eclater-grise`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-36-2 diff c3f722ca0 ee9f15714` (`c3f722ca0` = base : 36-1 et 36-3 fusionnées par
l'orchestrateur ; `4d55adbbf` = dernier commit du code de production ; `7efe74db4` ajoute le `.tex` du manuel ;
`95259d3db` et `ee9f15714`, deux commits de tests — capture de l'état « succès », compteurs de la garde des libellés en
dur ; `ee9f15714` = **dernier commit de code** ; les commits suivants ne portent que la fiche, le registre, ce prompt et le
registre de sprint). Langue du rapport : français. Modèle :
**Sonnet ou Opus** (première passe : jamais Haiku). Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/36-2-eclater-bouton-grise.md` en entier (H1 à H5, D1 à D7, tables A et B de *Si R0
rend…*, AC1 à AC10, mutations, **Dev Agent Record**, **Change Log**) ; les registres `e36-choix-autonomes-e360.md`
(C-e360-5, C-e360-11) et `e36-choix-autonomes-36-2.md` (C-36-2-1 à C-36-2-19, dont **C-36-2-18 et C-36-2-19**, écrits au
développement) ; le rapport `/home/gcorbaz/devel/kesh-gate-logs/e36-2/validation-p5-ciblee.md`. Ne prends pas le Dev Agent
Record pour argent comptant : recoupe-le avec les journaux (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e36-2/` :
`r0-tete.log`, `r0-tete-h.log`, `r0-0176.log`, `rouges.log`, `rouges-vitest.log`, `r0-apres*.log`, `inventaires.log`,
`grep-consommateurs-avant.log`, `mut/campagne-*.log` et `mut/M*.log` (script `mut/mut-36-2.py`), et les journaux de gate
nommés au Dev Agent Record. **Captures** (thème clair, AC10) sous `/home/gcorbaz/devel/kesh-gate-logs/e36-2/captures/` :
`r0-tete-*.png` (avant), `r0-0176-*.png` (image 0.17.6), `r0-apres-*.png` (après) — regarde au moins `r0-tete-g-largeur.png`,
`r0-apres-g-largeur.png`, `r0-apres-h-six-lignes-liste.png`, `r0-apres-f-somme-fausse.png`, `r0-apres-e-*.png`, si ton outil lit
les images.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis tes trois rapports. **Aucun fichier temporaire**,
  nulle part ; **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`,
  `bash -c`/`sh -c`, `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`,
  `scripts/recette-montee.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` (dont
  `mut/mut-36-2.py`, `mut/run-mut-36-2.sh`, `e2e-36-2.sh`, `r0b-36-2.sh`) ; ni `make`, `latexmk`, `cargo`, `docker`,
  `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`,
  `awk` (sans redirection ni `system()`), `cat`, `cat -A`, `od -c`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`,
  `uniq`, `cmp`, et, pour un PDF, la seule forme `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport. Pour
  une lecture de saisie, **déroule `readJournalAmount` et `lireMontant` à la main** contre le code lu ligne à ligne.
- **Tout chemin cité** se vérifie (`ls` ou `git cat-file -e <ref>:<chemin>`), la ligne lue citée.

## Les axes obligatoires, pour chaque lentille

1. **La lecture d'un montant (D5, C-36-2-13)** — `lireMontant` dans `TransactionSplitModal.svelte` : ordre des refus
   (`empty`/`nonpositive` → « montant > 0 requis », `unreadable`, `ambiguous`, `cents`) ; la garde
   `value.eq(value.round(2))` ; une saisie que le serveur refuserait peut-elle activer le bouton (échelle > 2, plus de quinze
   chiffres, signe, `0.00`) ? `readJournalAmount` n'est-il **pas** modifié (`git diff c3f722ca0 ee9f15714 --
   frontend/src/lib/features/journal-entries/` vide) ?
2. **L'envoi (D2)** — `handleConfirm` : la charge est-elle toujours `value.toFixed(2)` d'une lecture `ok` ? Le transtypage
   `(lectures[i] as { kind: 'ok'; value: Big })` peut-il voir une lecture non `ok` (course entre `clientError` et le clic,
   `submitting`) ? `SplitProposalLine.amount: string` dit-il désormais vrai ?
3. **La somme et l'équilibre** — `sumAmounts` en `Big`, `balanceExact` par `eq`, `balanceDiff` signé, `expectedBig` (repli
   `null` si le serveur rendait un montant illisible), `expectedLabel` (C-36-2-19). État `$derived` qui resterait périmé
   (`lectures` dérivé de `splits`, réaffectation de `splits` à l'ouverture) ?
4. **Les textes (D3, D5)** — six clés × quatre locales, à côté du bloc `reconciliation-split-*` (jamais en fin de fichier) ;
   replis littéraux égaux au fr-CH ; arguments passés ; **`reconciliation-split-error-imbalance` inchangée** dans les quatre
   `.ftl` et ses trois autres sites (`crates/kesh-api/src/errors.rs`, `failed-proposal-label.ts`, son test) ; traductions
   qui disent la même règle (de, it, en) ; `sitesTotal` 3044 (`i18n-keys.test.ts`) ; `i18n-entrees-a-variables.test.ts`.
   Grep des consommateurs de chaque clé touchée : `git grep -n "reconciliation-split-" -- crates frontend/src`.
5. **La largeur (D6, C-36-2-18)** — `sm:max-w-4xl` et `w-[30%]`/`w-[17%]`/`w-[33%]`/`w-[15%]` : sans colonne *Projet*, le
   rendu reste-t-il sain ? Aucun `max-h` ni `overflow` ajouté (la liste du champ de compte, en `absolute`, serait rognée) ;
   `dialog-content.svelte` non touché. Le seuil de #1181 (9 lignes) est-il bien mesuré et écrit ?
6. **Le manuel contre le code (AC9)** — `docs/manual/fr/user-manual.tex`, § *Éclatement d'une transaction (split)* : chaque
   phrase est-elle vraie au code (forme d'un montant, « sans signe », bouton grisé, message, indicateur) ? **Le PDF n'est pas
   régénéré dans la branche** (à l'intégration, fichier d'epic) : vérifie-le (`git diff --stat c3f722ca0 ee9f15714 --
   docs/`) et donne la sonde à jouer à l'intégration :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'Le bouton reste grisé tant'`
   (attendu ≥ 1 après régénération ; 0 aujourd'hui).
7. **Les témoins et les mutations** — chaque AC a-t-il un test qui peut rougir ? Mutations M1 à M9 (M1-bis, M1-ter, M2-bis,
   M2-ter, M3-bis, M7-bis, M8-bis comprises) jouées et tuées (`mut/campagne-*.log`) ; celles que seul l'E2E tue (M2-ter,
   M7, M7-bis, M9) le sont-elles par l'assertion annoncée ? Les assertions de R0 qui sont `expect.soft` rougissent-elles le
   test ? Un test qui passe **à vide** (ex. `relever` qui ne trouve aucune ligne, `messageClient` absent) ? Aucun attendu de
   test existant retouché sans motif (`git diff c3f722ca0 ee9f15714 -- '*.test.ts' '*.spec.ts'`, lignes `-`) ? Propose des
   mutations plausibles non jouées.
8. **R0 et ses verdicts (AC1)** — les tables A et B appliquées (A3, B2) se lisent-elles dans les journaux R0-a et R0-b, au
   critère mécanique de la fiche (V4-3) ? Le verdict H2 « réfutée sous Chromium » est-il étayé (`illisible=false`,
   `70,00` → `70.00`) ? `closes #1150` est-il justifié ?
9. **Le périmètre** — `git diff --stat c3f722ca0 ee9f15714` ne nomme ni `AccountAutocomplete.svelte` (AC7), ni
   `dialog-content.svelte`, ni `crates/` hors `kesh-i18n/locales/`, ni `CHANGELOG.md`.

## Lentille B — Blind (le diff seul, sans la fiche)

Relis ligne à ligne `TransactionSplitModal.svelte`, les quatre catalogues, les tests et la spec E2E. Cherche : une valeur
envoyée différente de la valeur lue ; un refus qui ne se dit pas ; un `$derived` périmé ; une largeur qui casse un autre
état ; un test qui passe à vide.

## Lentille E — Edge (chaque branche, chaque borne)

Déroule au code : `''`, `' 70'`, `'70 '`, `0`, `0.00`, `0,001`, `0.005`, `0.01`, `00.10`, `-0`, `+70`, `-70.00`, `7O`,
`70,,0`, `1.234`, `1,234`, `12'34`, `1'200.50`, `1’200.50` (U+2019), `1'200'000.00`, `999999999999999.99`,
`1000000000000000`, `60.0000`, `30.0050`, `1.2`, `1.`, `.5`, `１２３` (pleine chasse) ; une transaction de crédit (montant
positif), une transaction à `-388.2`, à `0`, illisible. Et : 2 et 50 lignes ; une ligne supprimée après saisie (les lignes
sont rendues par indice). Chaque cas : issue au code, issue attendue (fiche), test qui le couvre (ou absence, prouvée par
`grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC1 à AC10 : livré, testé, au lieu prescrit ? Les rouges de départ (`rouges.log`, `r0-tete.log`) ; les verts
au départ dits tels ((b) et (h)) ; les mutations ; les gates au dernier commit de code (chiffres du Dev Agent Record, chacun
retrouvé à son journal) ; les écarts déclarés (C-36-2-18, C-36-2-19, V-a à `'60.00'`). Le Dev Agent Record n'affirme-t-il
que ce qui a tourné (**E2E complet et `gate-ci` de clôture non lancés** : ils tournent à la clôture ; `gate-ci` de passage en
`review` : voir le Record) ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e36-2/review-p1-B.md`, `…/review-p1-E.md`,
`…/review-p1-A.md`. Chaque rapport : findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM /
LOW), lieu (`fichier:ligne`), ligne lue copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non
exercés** ; le décompte final **recompté depuis la liste**.
