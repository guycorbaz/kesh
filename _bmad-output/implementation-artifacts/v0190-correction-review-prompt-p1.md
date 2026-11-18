# Prompt — revue de code P1 de la story `v0190-correction` (#1162), trois lentilles, première passe complète

Tu relis la story **`v0190-correction`** de Kesh (comptabilité suisse ; SvelteKit 5 runes), release **0.19.0** :
**le champ « client » d'une facture (`ContactPicker`) ne montre plus un client que la facture n'enregistrera pas**. Le texte tapé
est un brouillon (`draft`), jeté au départ du champ ; *Entrée* suit trois règles (option désignée et recherche terminée → choisie ;
brouillon affiché → jamais d'envoi ; champ non touché → le formulaire part) ; une surbrillance ne vaut choix que **désignée**
(par la saisie, à l'arrivée de la réponse, liste ouverte, ou par une flèche) ; recherche serveur avec anti-rebond, invalidée par
toute frappe (`searchSeq`, `pending`). **Frontend seul : aucune migration, aucun changement serveur, aucune clé i18n.** La règle
est reprise d'`AccountAutocomplete.svelte` (Story 36-1), avec quatre différences dites dans la fiche.

Worktree `/home/gcorbaz/devel/kesh-v0190c`, branche `story/v0190-correction`. Diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-v0190c diff 39441131d d5d3f8b17` (`39441131d` = fiche validée, code d'origine ;
`d5d3f8b17` = dernier commit de code ; le commit suivant ne porte que la fiche, le registre, ce prompt et `sprint-status.yaml`). Langue du rapport : français. Modèle : **Opus ou Sonnet** (première passe : jamais Haiku).
Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/v0190-correction.md` en entier (D1 à D3, AC1 a à h, AC2 à AC6, Dev Notes, **Change Log**
— dont l'entrée « étape 0 du développement », qui revient sur la clause *Échap* de la validation P7 —, **Dev Agent Record**) ;
le registre `_bmad-output/implementation-artifacts/e0190-choix-autonomes-v0190c.md` (C-v0190c-1 à 3, 11 à 13, 16, 17, 19 à 21) ;
le dernier rapport de validation `/home/gcorbaz/devel/kesh-gate-logs/v0190c/validation-p8-ciblee.md`.
Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec les journaux hors dépôt (lecture seule) sous
`/home/gcorbaz/devel/kesh-gate-logs/v0190c/` : `rouges-unit-avant.log`, `unit-apres-1.log`, `mut/` (`resume.log`, `resume2.log`,
un journal par mutation, `campagne.sh`, `campagne2.sh`, `muter.py`), `e2e-rouge-origine-3.log`, `e2e-vert.log`,
`e2e-un-test.sh`, `front-*.log`, `test-fast-review.log`, `gate-ci-review.log`. **Captures** (thème clair, base jetable) :
`/home/gcorbaz/devel/kesh-gate-logs/v0190c/captures/` (`1-texte-tape-non-choisi.png`, `2-champ-quitte-client-reaffiche.png`,
`3-liste-designation-par-fleche.png`) et leur script `captures.mjs` — regarde-les si ton outil lit les images.

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

1. **Le client montré est celui qui part.** Existe-t-il une séquence (frappe, flèches, *Entrée*, *Échap*, clic, `blur`,
   changement de `selected` par le parent, réponse tardive ou en échec de `listContacts`) au terme de laquelle le formulaire part
   (soumission implicite ou bouton) pendant que le champ montre un autre nom que `selected` ? Ou au terme de laquelle `onSelect`
   reçoit un contact que l'écran n'a pas montré désigné ?
2. **La table d'*Entrée*** (`onEnter`) : IME (`isComposing`, `keyCode === 229`), `repeat`, règle 1 (`open && !searching &&
   highlighted >= 0`), règle 2 (brouillon : blanc → jeté ; sinon ouverture et désignation si `!searching`), règle 3. Est-elle
   close ? Les gardes `searching` des règles 1 et 2, individuellement équivalentes en mutation (M4, M11), le sont-elles vraiment,
   et la paire est-elle tenue (M4-M11 combinées, cas d) ?
3. **La recherche et ses abandons** : `searchSeq` (frappe, `abandonSearch` au `blur`, au choix, au brouillon blanc, au changement
   de client) ; `pending` et `loading` levés par la seule réponse de séquence courante ; **« Chargement… » ne reste-t-il jamais
   affiché** ? *Échap* n'abandonne rien (étape 0, P8-1/P8-2) : une réouverture par *Entrée* ou par une flèche peut-elle désigner
   une liste qui ne répond pas à la saisie affichée ? Le test de `open` à l'arrivée (seule garde de e-bis-bis) suffit-il ?
   La direction de relance (`fleche`) peut-elle survivre à un abandon ?
4. **Le `$effect` du changement de client** (`selectedId`, `lastSelectedId`, `untrack`) : premier passage, choix suivi du
   `rerender` du parent (double purge), passage à `null`, même identifiant dans un nouvel objet, course du chargement d'une
   facture (`InvoiceForm.svelte:376-386`, issue #1233 hors périmètre).
5. **Les trois consommateurs** (`git grep -n "<ContactPicker" -- frontend/src`) : la facture (dans un `<form>`), les filtres des
   pages de liste et des échéances (hors formulaire). Un geste d'aujourd'hui est-il cassé (choix à la souris, filtre posé par
   l'URL, *Réinitialiser*, E2E existants qui choisissent un contact par `fill` puis clic sur l'option) ? `mousedown` empêché sur
   la liste et les options : le focus reste-t-il au champ, et le clic choisit-il toujours ?
6. **Le rendu** : `CLASSE_DESIGNEE` / `CLASSE_SURVOL` importées, sous-ligne IDE (`text-primary-foreground` désignée, sans
   `group-hover:` ; `text-text-muted group-hover:text-text` sinon), `group` sur l'option ; jetons définis dans `app.css` ;
   `aria-activedescendant` et `aria-selected` cohérents avec ce qu'*Entrée* choisirait. Captures.
7. **Les tests et les mutations** : chaque cas d'AC1 (a à h, dont d-bis à d-sexies, e-bis, e-bis-bis et son second volet, e-ter,
   e-quater, e-quater-bis, e-quinquies, e-sexies, f-bis à f-quater, g, g-bis) a-t-il un test qui peut rougir (`rouges-unit-avant.log`,
   `mut/`) ? Le montage (temps simulé, `el.focus()`/`el.blur()` réels, `rerender` dans `onSelect`) masque-t-il un défaut ?
   g-bis par une flèche au lieu d'un focus (C-v0190c-21) : écart justifié ? AC2 (`InvoiceForm.test.ts`) : l'affichage prouve-t-il
   le rouge ? AC3 (`invoices-list-page.test.ts`) ; AC4 (E2E) : rouge constaté sur le code d'origine à la ligne de la liste
   après *Entrée* (la vue détail de la facture créée par l'ancien code), vert sur le correctif ; le compteur `creations` et la
   relecture par la liste des factures suffisent-ils ?
8. **Le manuel** (`docs/manual/fr/user-manual.tex`, § *Création d'une facture*, l'item **Client**) : dit-il ce que fait le code
   (clic, *Entrée* sur l'option mise en évidence, texte non choisi abandonné au départ, *Entrée* qui ne fait pas partir la facture
   tant qu'un texte est affiché, création dans *Contacts*) ? « (ou création à la volée) » a-t-il disparu
   (`git grep -n "à la volée" -- docs/manual/fr/user-manual.tex` ne doit rendre que les lignes des rapports) ? Le **PDF n'est
   pas régénéré dans la branche** (fiche, AC6) : contrôle à la release, forme
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'ou création à la volée'` — dans la
   branche il rend encore `1`, c'est attendu ; le dire, ne pas le compter comme défaut.
9. **La fiche et le registre** : l'entrée « étape 0 » du Change Log cite-t-elle des lignes réellement lues ? D1 à D3 réécrits
   (P8-1 à P8-4) sont-ils cohérents entre eux et avec le code ? La Note de version dit-elle vrai ? Les décomptes du Dev Agent
   Record se recomptent-ils (`grep -c` des `it(` du fichier de test, journaux) ?

## Lentilles

- **B — Blind Hunter** : le diff seul d'abord (sans la fiche), défauts de code, de course et d'état ; puis les axes.
- **E — Edge Case Hunter** : toutes les branches et séquences (axes 1 à 5 surtout), états limites (liste vide, échec, réponse
  hors ordre, focus perdu pendant la recherche, parent qui refuse le choix).
- **A — Acceptance Auditor** : chaque AC et chaque décision de la fiche contre le code, les tests et les journaux (axes 6 à 9).

## Rapport

Un rapport par lentille : `/home/gcorbaz/devel/kesh-gate-logs/v0190c/review-p1-B.md`, `…-E.md`, `…-A.md`. Pour chaque finding :
identifiant (`B1`, `E1`, `A1`…), sévérité (CRITICAL, HIGH, MEDIUM, LOW), fichier et ligne, extrait cité, scénario, correctif
proposé. Verdict chiffré en tête. **Liste des axes exercés ET non exercés** (un « 0 finding » sans elle ne compte pas). Ce qui n'a
pas tourné reste écrit : la revue n'exécute rien ; `test-fast --ci` et `gate-ci` ont tourné au passage en revue, l'E2E
complet (et le `gate-ci` de clôture) non.
