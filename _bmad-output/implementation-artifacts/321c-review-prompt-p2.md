# Prompt — revue de code P2 de la Story 32-1c — passe ciblée, une lentille (regression hunter), Haiku

*Versionné le 2026-10-24. Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille** — chasseur de
régressions —, en contexte frais, **Haiku**, braquée sur **le seul diff de la remédiation P1**. Lecture seule stricte.
Langue du rapport : français.*

Worktree `/home/gcorbaz/devel/kesh-32-1c`, branche `story/32-1c`. La 32-1c corrige le manuel d'administration, § *Répéter
une clôture sur une copie* (`sec:repetition-cloture`), **sans changer aucune commande ni ajouter aucune consigne
d'agir** (#714 ; ce qui changerait une commande relève de #726). Aucun code applicatif.

**Le diff à relire, aplati, en une seule commande** (n'en lis pas d'autre ; pas la suite des commits) :

```sh
git -C /home/gcorbaz/devel/kesh-32-1c diff f2cc0df8 b952cddf
```

Les numéros de ligne d'un hunk sont ceux du fichier **après** `b952cddf` ; pour lire un passage entier, lis le
fichier au commit (`git show b952cddf:<chemin>`, identique au fichier du worktree). Les rapports P1 sont hors
dépôt : `/home/gcorbaz/devel/kesh-gate-logs/321c-review-p1-B.md`, `-E.md`, `-A.md`. La fiche :
`_bmad-output/implementation-artifacts/32-1-defauts-courts.md` (partie C ; Dev Agent Record *32-1c*, § *Remédiation de
la revue de code P1* ; Change Log, « Revue de code P1 (32-1c) ») ; le registre
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-321.md` (**C-32-1-41, C-32-1-42**). Le Dev Agent Record
déclare des contrôles : **ne les prends pas pour argent comptant**, rejoue ceux qui se rejouent par pipe. Journaux, en
lecture seule : `/home/gcorbaz/devel/kesh-gate-logs/321c-make-fr-p1.log`, `321c-gate-cible-p1.log`. PDF d'avant (base
`c03d3a5f`) : `/home/gcorbaz/devel/kesh-gate-logs/321c-admin-avant.pdf`, `321c-brochure-avant.pdf`.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans `/tmp`, ni dans un scratchpad. **Aucune redirection** (`>`, `>>`, `tee`), pas même vers
  un fichier « jetable » : compare par des pipes ou par substitution de processus (`<(…)`), lis les PDF vers la sortie
  standard seulement. Un fichier créé par erreur se **déclare** au rapport ; il ne se supprime pas.
- **Aucun `git checkout`, `git switch`, `git restore`**, aucun `git` qui écrit (`commit`, `stash`, `reset`, `tag`,
  `push`, `rebase`, `worktree`, `add`).
- **N'exécute rien qui écrit, compile ou exécute du code** : ni `scripts/prepare-release.sh` (il bumpe les dix crates du
  workspace), ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni aucun script de `scripts/` ou de
  `kesh-gate-logs`, ni `make`, `latexmk`, `xelatex`, `pdftoppm`, ni `cargo` (aucune sous-commande), ni
  `npm`/`npx`/`node`, ni `docker`, `mysql`/`mariadb`, `sqlx`, `curl`, `python3`, `perl`, ni `sed -i`. Aucune commande
  `gh` qui écrit ; `gh issue view 714|726|731` est permis.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git merge-base`, `grep`, `sed -n` (et `sed` en filtre de
  pipe), `awk` (sans redirection), `tr`, `cmp`/`diff` sur des substitutions de processus, `cat`, `ls`, `wc`, `pdfinfo`,
  `pdftotext <f>.pdf -` (et `-layout`, `-f N -l N`).
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie copiée (`grep -nF`, `sed -n`, `git diff`,
  `pdftotext`).

### Lire un PDF : aplatir avant de greper

`pdftotext` coupe les lignes et écrit l'apostrophe typographique `’` là où le `.tex` a `'`. Commande :

```sh
pdftotext /home/gcorbaz/devel/kesh-32-1c/docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | sed "s/’/'/g" | tr -s ' ' | grep -oF "<phrase>"
```

**Témoin positif d'abord** (« trop grande pour être restaurée par l'écran » doit sortir). Une boîte `\keshcommand`
laisse une espace avant la ponctuation (« COPIE , contraire ») : une phrase qui ne sort pas se cherche par un fragment
plus court avant de conclure. Les **trois** PDF sont `docs/manual/fr/admin-manual.pdf`, `user-manual.pdf`,
`marketing-brochure.pdf` ; le manuel source est `docs/manual/fr/admin-manual.tex`.

## Ce que la remédiation P1 a fait (à vérifier, non à croire)

- **B2 = A4 = E4** (MEDIUM) : puce *Au contrôle~3* de *En cas d'arrêt* — le refus nommé est la **comparaison** du
  chemin que rend `realpath -e` à `COPIE` ; la barre finale (`realpath -e` réussit) et `/var/tmp` lien comme causes ;
  ligne du `CHANGELOG.md` (bloc `[0.16.0]`) reformulée.
- **B3 = E3** (l. ~1625, *Ce qu'elle ne fait pas*) : les sessions rendues aux connexions. **B4** (contrôle 4) : « La
  dernière commande relève ». **B1** réfuté, renvoi précisé « en tête de ce contrôle » (C-32-1-41).
- **B5, A3 = E2, E1** : `\Needspace` (exact) avant les items 1, 4, 8, avant les paragraphes des amorces « Sans sudo : »
  et du troisième listing, avant l'étape 6 Synology et deux listings InnoDB ; les `\needspace` (approchés) d'entre amorce
  et listing retirés (C-32-1-42). Commentaire du `\usepackage{needspace}` changé.
- **A5** : ligne du CHANGELOG bornée (sous-section, PDF de cette version, #731 ; U+FFFD dans les trois PDF). **A2** :
  en-tête du registre. **A1 = E5** : `make -B fr` rejoué, journal `321c-make-fr-p1.log`.

## La lentille — chasseur de régressions

**Le motif mesuré du dépôt : la sévérité se déplace vers ce qu'on vient d'écrire.** Pour chaque ligne **ajoutée** du
diff, deux questions, et une preuve pour chacune.

**A. Ce que la ligne affirme est-il vrai ?**

1. *Au contrôle~3* : au contrôle 3, `[ "$(realpath -e -- "$COPIE")" = "$COPIE" ] && echo 'chemin réel'` — la phrase
   « `chemin réel` ne s'affiche pas » désigne-t-elle bien cette ligne ? La garde (les trois listings de *Détruire la
   copie*) fait-elle **la même** comparaison (`grep -nF 'realpath -e --' docs/manual/fr/admin-manual.tex`) ? Avec un
   `/var/tmp` qui serait un lien, `realpath -e` rend-il « le chemin de la cible » ? La phrase est-elle grammaticale, sans
   contresens, sans **consigne d'agir neuve** (le geste « laissez le dossier vide » doit être inchangé) ?
2. *Ce qu'elle ne fait pas* : la phrase réécrite garde-t-elle **tous** les faits d'avant (deux lignes d'audit, une par
   export ; lignes de réglages créées par l'export de souveraineté ; facturation = première ouverture de la page ;
   relances : la page pose en plus les trois niveaux) ? Les incises `---` sont-elles appariées ?
3. Contrôle 4 : « La dernière commande relève… » — la phrase suivante (« Une ligne `posée` : … refaites cette
   commande ») garde-t-elle son sens ?
4. Contrôle final : la commande `ls -d /var/tmp/kesh-repetition-*` est-elle **effectivement** écrite en tête du
   contrôle final, au-dessus du renvoi (`grep -n 'ls -d /var/tmp' docs/manual/fr/admin-manual.tex`) ?
5. `CHANGELOG.md` : la ligne reformulée dit-elle vrai et rien de plus (un listing coupé **dans la sous-section** au PDF
   actuel ? un U+FFFD dans l'un des trois PDF : `pdftotext <pdf> - | grep -o $'\xef\xbf\xbd' | wc -l`) ? Le bloc garde-t-il
   la forme `## [0.16.0] — Non publié` (que `scripts/prepare-release.sh` cherche — **lis**-le, ne l'exécute pas) ?
6. Registre C-32-1-41 et C-32-1-42, Dev Agent Record, Change Log : chaque fait et chaque **nombre** est-il vrai
   (recompte depuis la source : pages relevées, frontières, `\Needspace` ajoutés — `git diff … | grep -c '^+\\Needspace'`) ?

**B. La ligne a-t-elle cassé quelque chose ailleurs ?**

1. **Aucune commande touchée** : rejoue le `diff` des 22 blocs `lstlisting` de la sous-section (de
   `\label{sec:repetition-cloture}` au `\endgroup`) entre `f2cc0df8` et `b952cddf`, par `awk` en pipe et
   substitution de processus : vide ? `grep -c 'begin{lstlisting}'` sur l'extrait : 22 ?
2. **Coupures de page — le risque propre de cette remédiation** : un `\Needspace` déplace les sauts de page de tout le
   reste du manuel. Rejoue, au `pdftotext -layout -f N -l N`, le bas de chaque page et le haut de la suivante sur **tout**
   `admin-manual.pdf` (129 pages) — au minimum chaque frontière où la dernière et la première ligne sont indentées de 3
   espaces ou plus (un listing l'est toujours) —, et compare à `321c-admin-avant.pdf` : une coupure **neuve** (absente du
   PDF d'avant) ? Une page presque vide créée par un `\Needspace` trop grand ? Les items 1, 4, 8 sont-ils avec leur
   listing ; « Sans sudo : », « Ou, avec sudo : » et l'amorce du troisième listing avec le leur ? Le manuel utilisateur et
   la brochure ont-ils bougé (leur `.tex` n'est pas touché ; brochure : commande de l'AC-c9 de la fiche, témoin positif
   d'abord) ?
3. **PDF commité = `.tex` commité** : le journal `321c-make-fr-p1.log` montre-t-il les **trois** PDF ? Les phrases
   neuves sont-elles au PDF aplati (témoin d'abord) ? Un `.tex` modifié sans PDF régénéré se verrait au texte aplati.
4. **Gardes** : le diff change-t-il un texte que lisent `crates/kesh-api/tests/textes_coherents.rs` ou
   `configuration_transmise.rs` (G14 à G16, G19) de façon à les faire passer à vide ? Raisonne sur leur code ; recoupe
   avec `321c-gate-cible-p1.log` (n'exécute rien).
5. **Propagation** : la formule fausse « `realpath -e` en échec » / « qui a échoué » reste-t-elle dans un texte
   **publié** (manuels, CHANGELOG, README, `website/`) ? (`git grep -n 'realpath -e' -- docs/manual CHANGELOG.md README.md website`.)
   Les fiches et rétrospectives passées sont des relevés datés : ne les compte pas.
6. **Messages de commit** : `git log f2cc0df8..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
   ne rend rien ; chaque commit porte `refs #714` et `Co-Authored-By`. Aucune donnée réelle (IBAN, solde, relevé).

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/321c-review-p2.md`, par l'outil d'écriture de fichier —
jamais par une redirection :

- bilan en tête : CRITICAL / HIGH / MEDIUM / LOW, recompté sur ta liste ;
- findings numérotés `R1…`, chacun avec sévérité, fichier:ligne, **preuve copiée**, correctif proposé — et, s'il touche
  le manuel, s'il **change une commande ou ajoute une consigne d'agir** (alors il relève de #726, non de la 32-1c) ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** — un « 0 finding » sans cette liste ne compte
  pas comme passe ;
- tout fichier créé par erreur, déclaré ;
- rien d'autre n'est écrit nulle part.
