# Prompt — revue de code P2 **ciblée** de la Story 36-1 (#1095, #1165) : une lentille, la seule remédiation de la P1

Tu relis **la seule remédiation de la revue P1** de la Story 36-1 de Kesh (comptabilité suisse, SvelteKit 5 runes), release
0.17.8. La P1 (trois lentilles, rapports `/home/gcorbaz/devel/kesh-gate-logs/e36-1/review-p1-{B,E,A}.md`) n'a rien rendu
au-dessus de LOW ; l'orchestrateur a fait corriger ses LOW **dans la story** (règle « Tarir la source ») : ce qui reste à
relire n'est plus la story, c'est ce correctif. Langue du rapport : français. Contexte frais. Une seule lentille, celle du
**chasseur de régressions** : *ce correctif introduit-il un défaut, ou laisse-t-il un résidu du symptôme qu'il corrige ?*

Worktree `/home/gcorbaz/devel/kesh-36-1`. **Le diff à relire** :
`git -C /home/gcorbaz/devel/kesh-36-1 diff 258a04559 de7a79b17` (`258a04559` = dernier commit de code relu en P1 ;
`de7a79b17` = le commit de la remédiation, seul commit de code depuis). Le commit qui le suit ne porte que la fiche, le
registre et ce prompt : lis-en le Change Log (entrée « Remédiation de la revue P1 ») et le registre
`e36-choix-autonomes-36-1.md` (C-36-1-10, C-36-1-11), sans les prendre pour argent comptant.

Ce que la remédiation dit avoir fait, par finding de la P1 :

- **E1** — un **brouillon** qui reprend le début du libellé affiché (`3200 — Honoraires`, `3200 — Hon`) désigne son compte :
  filtre `(draft !== null && labelOf(a).toLowerCase().startsWith(q))` (C-36-1-10). Sans brouillon, un champ non touché garde
  sa liste vide.
- **E2** — variantes « espaces seules » d'AC1 n et o (`n-bis`, `o-bis`) ; mutations `trim()` aux deux sites (M29, M30).
- **B3/E3** — garde IME `e.isComposing || e.keyCode === 229` (`j-bis`).
- **B1** — `showNoMatch` porte `!loadError` (`z`).
- **B2** — E4 (message) seulement si `active.length > 0` ; sinon **E4-bis** : `commitDraft()`, liste fermée, index `-1`,
  aucun message (C-36-1-11 ; `m`, `m-bis`).
- **B4** — `CLASSE_LISTE` exportée de `account-autocomplete-classes.ts`, posée par le composant sur sa liste et lue par la
  garde AC8 ; un cas lit le source du composant.
- **E4** — variante « au choix d'un compte » d'AC1 h retirée (inatteignable pendant que le message s'affiche).
- **A6** — `brouillonEntree(page, plage)` : plages 38000-38499 et 38500-38999, une par test E2E.
- Compte rendu (A1 à A5) : note de version, en-tête, base des mutations, provenance de 7,1:1 et de `sitesTotal`, I7 = #1184.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire** ; **aucune
  redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`** ; ni `pkill`, ni `killall`, ni `kill`. **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui
  écrit.
- **Aucun interpréteur, rien qui compile ou exécute** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ;
  ni **`scripts/prepare-release.sh`**, ni `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`, ni aucun
  script du dépôt ou de `kesh-gate-logs` (le banc `mut/mutations-36-1.py` et `gates-remed-36-1.sh` se **lisent**) ; ni `make`,
  `cargo`, `docker`, `mariadb`, `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`. **Aucune commande `gh`**, aucun réseau.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk` (sans redirection
  ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`. ⚠️ `grep` est une fonction vers `ugrep` : emploie
  `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée ; tout chemin cité se
  vérifie (`git cat-file -e <ref>:<chemin>` ou `ls`).

## Axes obligatoires

1. **Le filtre au libellé (E1)** : la condition `draft !== null` suffit-elle à garder le champ non touché (focus d'un champ
   rempli, sept consommateurs) ? Un brouillon partiel peut-il désigner un **autre** compte que celui que l'utilisateur vise
   (préfixe commun de deux libellés, `slice(0, 20)`) ? L'ordre des comptes dans `filtered` change-t-il ce qu'*Entrée* choisit
   pour une saisie numérique (régression d'AC1 c, d, t) ? `filteredIds` et l'invariant de désignation restent-ils justes ?
2. **E4-bis (B2)** : la table d'*Entrée* reste-t-elle **close** (un brouillon affiché qui laisse partir l'événement) ?
   `commitDraft()` en `allowClear` avec un texte libre non blanc : jamais `onSelect` (sinon un effacement non voulu) ?
   Pendant le chargement réel (facture : `accounts` vide puis rempli), la saisie jetée est-elle acceptable et dite ? Le
   message E4 ne peut-il plus jamais s'afficher sans liste proposable ? `active` vs `accounts` : un compte archivé ou non
   imputable seul dans la liste ?
3. **IME et mode dégradé (B3, B1)** : `keyCode` (déprécié) a-t-il un effet de bord sur une *Entrée* normale (jsdom,
   navigateurs : `keyCode` 13) ? `!loadError` dans `showNoMatch` : le retour d'un `loadError` à faux fait-il reparaître un
   message périmé (`noMatchDraft` jamais remis à `null` ?) ?
4. **`CLASSE_LISTE` (B4)** : le `<ul>` du composant rend-il exactement les mêmes classes qu'avant (`git diff 258a04559
   de7a79b17 -- '*AccountAutocomplete.svelte'`) ? Le cas qui lit le source du composant est-il robuste (chemin relatif au
   `cwd` de Vitest, faux vert possible) ?
5. **Tests et mutations** : chaque cas neuf (`j-bis`, `m-bis` ×2, `n-bis`, `o-bis`, `y` ×3, `z`, AC8 source) rougit-il sous sa
   mutation (`mut/synthese.log`, `mut/M27.log` à `M35.log`) ? Un cas qui passe à vide (`keyCode` réellement transmis par
   `fireEvent` ? `rerender({ loadError: true })` effectif ?) ? La variante retirée d'AC1 h laisse-t-elle un déclencheur sans
   preuve ? Les motifs revus du banc (M5, M6, M8, M9, M12, M14) mutent-ils encore ce qu'ils disent muter ?
6. **E2E (A6)** : les deux plages sont-elles réellement disjointes, entre elles et du test existant (39000-39900), et
   `mkRevenueAccount` (39101+) de l'autre spec ?
7. **Le symptôme ailleurs** : « aucun compte ne correspond », `accounts.length > 0`, `isComposing`, `bg-popover`, `7,3` —
   greper la **valeur** sur le dépôt et la fiche (code, tests, `.ftl`, note de version, Dev Notes, prompts) : un résidu de
   l'ancienne règle reste-t-il écrit comme vivant ? La note de version (A1) est-elle vraie au code, phrase par phrase,
   E4-bis compris ? Le manuel (`docs/manual/fr/user-manual.tex`) reste-t-il juste (D4 : aucun geste clavier du champ décrit) ?
8. **Les gates déclarés** (Dev Agent Record) : chaque chiffre au journal nommé (`front-complet-r3.log`,
   `verts-e2e-cible-r3.log`, `test-fast-ci-r3.log`), le commit `de7a79b17` en première ligne ; `gate-ci` et E2E complet
   annoncés pour la clôture, non lancés.

## Rapport

Écris **un fichier** : `/home/gcorbaz/devel/kesh-gate-logs/e36-1/review-p2-ciblee.md`. Findings numérotés (`R1`, `R2`…)
avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), lieu (`fichier:ligne`), ligne lue copiée, preuve, remède proposé ; **la liste
des axes exercés ET non exercés** ; le décompte final **recompté depuis la liste**.
