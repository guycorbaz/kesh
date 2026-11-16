# Prompt — revue de code P1 de la Story 36-1 (#1095, #1165), trois lentilles, première passe complète

Tu relis la **Story 36-1** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.8**,
Epic 36 (phase 1, sans migration) : **la touche *Entrée* dans le champ « Compte de produit » n'envoie plus un compte que le
champ ne montre pas (#1095), ni un compte que personne n'a désigné (#1165)**. Le correctif vit dans le composant partagé
`AccountAutocomplete.svelte` (sept consommateurs) : table d'états E0 à E5 de la touche *Entrée* ; invariant « l'index de
surbrillance ≥ 0 si et seulement si un compte a été désigné », tenu par un `$derived` réassignable ; message « aucun compte ne
correspond » dans une région `role="status"` (clé neuve `common-account-no-match`, quatre locales) ; désignation rendue visible
par des jetons du thème (`bg-primary text-primary-foreground`), distincte du survol (`hover:bg-border`). **Frontend et `.ftl`
seulement ; aucune migration ; aucun changement serveur ni d'API.**

Worktree `/home/gcorbaz/devel/kesh-36-1`, branche `story/36-1-entree-compte-produit`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-36-1 diff 6fcd654b6 258a04559` (`6fcd654b6` = fiche validée, code identique à
`9d5a142df` ; `258a04559` = dernier commit de code ; le commit suivant ne porte que la fiche, le registre, ce prompt et le
`sprint-status.yaml`). Langue du rapport : français.
Modèle : **Opus ou Sonnet** (première passe : jamais Haiku). Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/36-1-entree-compte-produit.md` en entier (D1 à D4, table E0 à E5, AC1 a à x, AC2 à
AC8, mutations M1 à M25 et M15-bis, Dev Notes, **Change Log** et **Dev Agent Record**) ; les registres
`e36-choix-autonomes-e360.md` (§ *Étape 0* — 36-1 ; C-e360-1 à -4, -11) et `e36-choix-autonomes-36-1.md` (C-36-1-1 à 9).
Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec les journaux hors dépôt (lecture seule) sous
`/home/gcorbaz/devel/kesh-gate-logs/e36-1/` : `rouges.log`, `rouges-vitest-brut.log`, `rouges-vitest-ac3-ac4.log`,
`rouges-e2e.log`, `verts-*.log`, `mut/` (`synthese.log`, un journal par mutation, le banc `mutations-36-1.py`), les journaux de
gates nommés au Dev Agent Record, `e2e-36-1.sh`, `captures-36-1.mjs`. **Captures** (thème clair, base jetable) sous
`/home/gcorbaz/devel/kesh-gate-logs/e36-1/captures/` (`01-…` à `08b-…`, et `notes-*.txt` : options désignées au focus, couleurs
calculées, hauteurs de la ligne d'écriture manuelle avant et après) — regarde-les si ton outil lit les images.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis tes trois rapports. **Aucun fichier temporaire**, nulle
  part ; **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par erreur se
  **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`,
  `bash -c`/`sh -c`, `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`,
  `scripts/recette-montee.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` (le banc de mutations
  et les scripts d'E2E et de captures se **lisent**, ne se lancent pas) ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`,
  `sqlx`, `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk`
  (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, et, pour un PDF,
  la seule forme `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport ; tout
  chemin cité se vérifie (`git cat-file -e <ref>:<chemin>` ou `ls`).

## Les axes obligatoires, pour chaque lentille

1. **La table d'*Entrée*** (`handleEnter`, `AccountAutocomplete.svelte`) : E0 (`loadError`, `isComposing`), E0-bis
   (`e.repeat`), E1 (`open && filtered[highlightIndex]`), E2 (`draft === null`), puis E3/E4/E5. Est-elle **close** : un état où
   un brouillon est affiché et où l'événement n'est pas empêché existe-t-il ? E5 appelle-t-il `commitDraft()` avec exactement la
   règle du `blur` (strict : jamais `onSelect`, #271 ; `allowClear` : `onSelect(null)` seulement si `value !== null && selected
   !== undefined`) ? E4 pendant le chargement (`accounts.length === 0`) : aucun message, rien ne part.
2. **L'invariant de désignation** : `highlightIndex` en `$derived` réassignable (`indexParDefaut(draft, filteredIds,
   valeurLue)`). Toutes les transitions de l'index (focus, Échap, choix, `blur`, flèches, frappe, × , valeur posée par le
   parent, comptes arrivés, liste rétrécie, même liste dans un nouveau tableau) respectent-elles « ≥ 0 ⇔ désigné » ? Le
   `$derived` de `value` (`valeurLue`, C-36-1-8) est-il juste, et ne masque-t-il rien ? Une réassignation suivie d'un recalcul
   peut-elle laisser un index hors bornes (`filtered[highlightIndex]` sur une liste plus courte) ?
3. **Les sept consommateurs** (`git grep -n "<AccountAutocomplete" -- frontend/src`) : les deux dans un `<form>` (facture,
   article) — *Entrée* sur un champ non touché envoie-t-elle ce que le champ montre ? Les cinq stricts
   (`JournalEntryForm`, `VatPurchaseAssistant` ×2, `ManualMatchModal`, `TransactionSplitModal`) : un geste clavier d'aujourd'hui
   est-il cassé (frappe puis *Entrée*, flèches, clic, *Entrée* dans le champ **crédit** de `JournalEntryForm:297`, `Ctrl+S`) ?
   La région `role="status"` change-t-elle une hauteur (capture `08a`/`08b`, `notes-*.txt`) ?
4. **Le message et l'accessibilité** : région toujours présente, contenu seul variable ; `aria-describedby` ne la vise que
   quand le message est affiché ; effacement à la frappe, au choix, au ×, au `blur`, au retapé, au rechargement des comptes.
   Les quatre `messages.ftl` (clé sous `common-account-invalid`, parité), `sitesTotal` 3037 (`i18n-keys.test.ts`).
5. **Le rendu** : `account-autocomplete-classes.ts`, la forme `class={[…]}` ; le survol n'est jamais posé sur l'option
   désignée ; aucune classe de couleur écrite en dur dans le composant pour les options ; la garde AC8
   (`account-autocomplete-classes.test.ts`) lit-elle vraiment les jetons dans les constantes (V5-1) ? Le module partagé
   `tests/e2e/helpers/theme.ts` (extrait de `button-destructive.test.ts`, C-36-1-7) : la garde de la 28-1 garde-t-elle
   exactement ses assertions ? (`git diff 6fcd654b6 258a04559 -- frontend/src/lib/components/ui/button/`)
6. **Les tests et les mutations** : chaque AC1 a…x a-t-il son cas, qui peut rougir (`rouges.log`) ? Le montage qui réinjecte
   la valeur (`monter`) masque-t-il un défaut (un `rerender` dans `onSelect`) ? AC3/AC4 : la soumission implicite simulée par
   `requestSubmit` est-elle fidèle au navigateur (bouton par défaut désactivé, validation du formulaire) ? AC5 : les trois E2E
   prouvent-ils l'absence d'envoi par la **relecture** (`GET`) et non par le seul compteur ? Les mutations M1 à M26 et M15-bis
   (`mut/synthese.log` ; M24 rejouée sur le motif du commit `258a04559`, M26 hors fiche) : chacune tuée par le cas que la
   fiche annonce ? Les équivalences déclarées (M14 seule) sont-elles vraies ? Propose des mutations plausibles non jouées.
   Aucun attendu de test existant retouché sans motif (`git diff 6fcd654b6 258a04559 -- '*.test.ts'`, lignes `-`). Les
   trois constats du développement (C-36-1-8 ; × sur le message ; sélecteur E2E traduit, garde KF-043) : leurs correctifs
   sont-ils justes et sans effet de bord ?
7. **Le manuel contre le code** : D4 dit que le manuel ne change pas (`docs/manual/fr/user-manual.tex`, aucun geste clavier du
   champ de compte décrit ; `:1129`, `:1159`). Le contester : une phrase du manuel est-elle devenue fausse (`git grep -n -i -E
   "entrée|clavier|surbrillance|flèche" -- docs/manual/fr/user-manual.tex`) ? Le PDF (`docs/manual/fr/user-manual.pdf`) n'a
   pas à changer : le contrôler par la forme permise si une phrase est en cause. La **note de version** de la fiche (aucun
   CHANGELOG sur la branche, C-e360-11) : chaque phrase est-elle vraie au code (« la première flèche vers le bas désigne
   désormais le premier compte », « la flèche vers le haut ouvre la liste sur le dernier », « fond bleu, texte blanc ») ?

## Lentille B — Blind (le diff seul, sans la fiche)

Relis ligne à ligne `handleKeydown`, `handleEnter`, `handleBlur`, `commitDraft`, les `$derived` (`filteredIds`, `valeurLue`,
`highlightIndex`, `showNoMatch`), le gabarit (région de statut, classes des options), le module de classes, les tests et les E2E.
Cherche : un ordre d'événements (`keydown` / `input` / `blur` / `mousedown`) qui laisse partir un formulaire avec un brouillon
affiché, ou qui choisit un compte non désigné ; une réassignation de `$derived` écrasée au mauvais moment ; une comparaison
`null`/chaîne fragile ; un test qui passe **à vide**.

## Lentille E — Edge (chaque branche, chaque borne)

Brouillon d'espaces seuls ; brouillon égal au libellé affiché (« 3000 — Ventes » retapé) ; un seul compte dans la liste ;
vingt comptes et plus (`slice(0, 20)`) ; *flèche haut* puis *flèche bas* aux bornes ; *Échap* puis flèche ; × sur une valeur
déjà nulle avec un brouillon ; valeur du parent qui change pendant que le message est affiché ; comptes qui arrivent pendant le
message ; mode dégradé (`loadError`) puis liste revenue (#1085) ; IME (`isComposing`) puis *Entrée* simple ; *Entrée* maintenue
sur un champ non touché d'un formulaire ; deux champs de compte sur la même facture. Chaque cas : comportement au code, test qui
le couvre (ou absence, prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC1 à AC8 : livré, testé, au lieu prescrit ? L'étape 0 (V5-1 à V5-3, lignes lues citées au Change Log) ; les rouges
de départ (Vitest 32 rouges sur 121 ; AC3/AC4 rejoués l'envoi en tête : `3200` et `3000` partis ; E2E 3 rouges sur 8) ; les
verts annoncés « verts au départ » (AC1 c, g, i, j, t, v ; AC3 b) ; les mutations ; les gates **au dernier commit de code**
(chiffres au Dev Agent Record, chaque chiffre au journal nommé, horodaté après le commit) ; l'E2E complet **non lancé** au
passage en `review` (règle de l'Epic 35, `e35-dev-commun.md`) — il tourne à la clôture ; les captures d'AC7 (huit états) ; les
écarts déclarés (C-36-1-7 à 9 ; module `theme.ts` hors liste de la fiche) sont-ils exacts et sans effet sur le livré ? Le Dev
Agent Record n'affirme-t-il que ce qui a tourné ? Les issues demandées (I1, I2, I3, I5, I6) le sont-elles, une par problème ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e36-1/review-p1-B.md`, `…/review-p1-E.md`,
`…/review-p1-A.md`. Chaque rapport : findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW),
lieu (`fichier:ligne`), ligne lue copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non exercés** ; le
décompte final **recompté depuis la liste**.
