# Prompt — revue de code P1 de la Story 32-1c (la procédure de répétition), trois lentilles, Sonnet

Tu relis le développement de la clé **32-1c** de la Story 32-1 de Kesh (comptabilité suisse) : les constats de
**#714** sur le manuel d'administration, § *Répéter une clôture sur une copie* (`sec:repetition-cloture`), corrigés
**sans changer aucune commande ni ajouter aucune consigne d'agir**, et les signes sans glyphe des trois PDF FR
(⚠️ des deux manuels, coches et flèche de la brochure, signes des deux scripts d'exemple). Aucun code applicatif.
Worktree `/home/gcorbaz/devel/kesh-32-1c`, branche `story/32-1c`, base `story/32-1` = `01b9a58b` (fiche validée). Le
diff à relire, **aplati** (un seul diff, pas la suite des commits) :
`git -C /home/gcorbaz/devel/kesh-32-1c diff story/32-1..HEAD`. Le développement est le commit `43587681` ; le commit
suivant ne porte que ce prompt. Langue du rapport : français. Modèle : **Sonnet**.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/32-1-defauts-courts.md` — § *Constats établis au code*, tableau **#714** ;
**partie C** (AC-c1 à AC-c10, tâches) ; Dev Notes § *Contrôle aplati d'un PDF* et § *32-1c — remarques* ; *Issues
ouvertes*, texte de [#726] ; Dev Agent Record, sous-section **32-1c** — et le registre
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-321.md` (C-32-1-15, 16, 18, 26, 31 ; **C-32-1-39 et
C-32-1-40**, écrits au développement). Le Dev Agent Record déclare des comptes, des contrôles et un gate : **ne les
prends pas pour argent comptant**, recoupe-les avec les journaux `/home/gcorbaz/devel/kesh-gate-logs/321c-make-fr.log`,
`321c-gate-cible.log`, `321c-gate-vat.log` (lecture seule), et rejoue en lecture les contrôles qui se rejouent par
pipe. Les captures sont jointes dans `/home/gcorbaz/devel/kesh-gate-logs/321c-captures/` (pages du PDF rendues en PNG,
thème clair) : `admin-p63-ou-avec-sudo-063.png`, `admin-p08-signe-attention-008.png`, `brochure-p03-replis-03.png` —
ouvre-les. Le PDF d'avant de la brochure est dans `/home/gcorbaz/devel/kesh-gate-logs/321c-brochure-avant.pdf`, celui
du manuel d'administration dans `321c-admin-avant.pdf` (tous deux extraits de `c03d3a5f`, base `origin/main`).

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans `/tmp`, ni dans un scratchpad. **Aucune redirection** (`>`, `>>`, `tee`), pas même vers
  un fichier « jetable » : compare par des commandes enchaînées (pipe) ou par substitution de processus (`<(…)`), lis
  les PDF vers la sortie standard seulement. Un fichier créé par erreur se **déclare** au rapport ; il ne se supprime
  pas (supprimer, c'est écrire).
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`).
- **N'exécute rien qui écrit, compile ou exécute du code** : ni `scripts/prepare-release.sh` (il bumpe les dix crates du
  workspace), ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni aucun script de `scripts/` ou de
  `kesh-gate-logs`, ni `make`, ni `latexmk`, ni `xelatex`, ni `pdftoppm`, ni `cargo` (aucune sous-commande), ni
  `npm`/`npx`/`node`, ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni `curl`, ni `python3`, ni `sed -i`, ni `perl`.
  Aucune commande `gh` qui écrit ; `gh issue view 714|726` est permis.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git merge-base`, `grep`, `sed -n` (et `sed`
  en filtre de pipe, sans `-i`), `awk` (sans redirection), `tr`, `cmp`/`diff` sur des substitutions de processus,
  `cat`, `ls`, `find`, `wc`, `pdfinfo`, `pdftotext <f>.pdf -` (et `-layout`, `-f N -l N`), la lecture des PNG.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie copiée (`grep -nF`, `sed -n`, `git diff`).

### Lire un PDF : aplatir avant de greper

`pdftotext` coupe les lignes et écrit l'apostrophe typographique `’` là où le `.tex` a `'` ; un `grep` naïf rend un faux
négatif. Commande :

```sh
pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | sed "s/’/'/g" | tr -s ' ' | grep -oF "<phrase>"
```

**Témoin positif d'abord** (« trop grande pour être restaurée par l'écran » doit sortir). Une boîte `\keshcommand`
laisse une espace avant la ponctuation (« COPIE , contraire ») : une phrase qui ne sort pas se cherche par un fragment
plus court avant de conclure. Compte des caractères sans glyphe :
`pdftotext docs/manual/fr/<pdf> - | grep -o $'\xef\xbf\xbd' | wc -l`.

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention.

1. `docs/manual/shared/kesh-style.sty` : la macro `\keshAttention` (définition, `\newcommand` vs `\providecommand`,
   l'espace avalée), les deux replis, le commentaire qui remplace le `literate` — dit-il vrai ? Un repli `→` en `\textbf`
   dans la police du texte : rendu ?
2. `docs/manual/fr/admin-manual.tex`, sous-section `sec:repetition-cloture` : chaque phrase neuve ou modifiée — est-elle
   **vraie** au code et au shell (`realpath` et la barre finale ; `curl -fsSL` silencieux en succès ; `grep` sur un
   fichier absent ; les filtres d'étiquette du contrôle final) ? Est-elle claire, sans contresens ni redite, sans
   **consigne d'agir neuve** ? Les trois `\needspace` ajoutés et le déplacé : rien d'autre ne bouge ?
3. Les 26 `\keshAttention\ ` des deux `.tex` : forme uniforme ? rendu correct en `\emph`, en `\item`, en début de ligne ?
4. Les deux scripts d'exemple (hors sous-section) : signes passés en ASCII ; le script reste-t-il un script juste ?
5. `_bmad-output/implementation-artifacts/31-4-repetition-a-blanc.md` : deux annotations — datées, exactes, texte
   d'origine gardé ?
6. `CHANGELOG.md` : bloc `## [0.16.0] — Non publié` (forme que `scripts/prepare-release.sh` attend — **lis** le script,
   ne l'exécute pas), rubrique, ligne juste et sans sur-promesse.

## Lentille E — Edge cases

1. **Coupure de page** : rejoue au `pdftotext -layout -f N -l N` le haut et le bas de chaque page de la sous-section
   (cherche ses bornes : « Répéter une clôture sur une copie » et « Ce que cette procédure n'a pas éprouvé ») — un
   listing coupé ? une amorce (« Sans sudo : », « Ou, avec sudo : », l'amorce du troisième listing) séparée de son
   listing ? Compare au PDF d'avant (`321c-admin-avant.pdf`) : les deux coupures déclarées préexistantes l'étaient-elles ?
2. **Signes** : un `⚠`, un U+FE0F orphelin, un `✓ ✗ ═ ⇄` reste-t-il dans un `.tex` FR (`grep -n`) ? Un autre caractère
   sans glyphe (U+FFFD) dans l'un des trois PDF ? Un `??` dans un texte aplati ? Un `\keshAttention` suivi d'autre chose
   qu'une espace échappée ?
3. **Brochure** : rejoue la comparaison de l'AC-c9 par substitution de processus (`cmp <(…) <(…)`, fonction `aplatir`
   de l'AC), **témoin positif d'abord**. Le retrait de `+` et de `→` pourrait-il masquer une différence réelle (un `+`
   ou un `→` de texte disparu ou ajouté) ? Cherche-les dans les deux textes avant retrait.
4. **AC-c1** : la phrase dit-elle exactement ce que fait le code ? Lis `crates/kesh-api/src/exports/global.rs` (l. ~300-
   320), `routes/company_dunning_settings.rs` (l. ~56), `routes/company_invoice_settings.rs` (l. ~263), le chemin du
   `.keshbackup` (`routes/admin.rs`, `emit_full_export_audit`) : deux lignes d'audit, vraiment une par export ? L'export
   **complet** crée-t-il aussi des lignes de réglages (la phrase ne l'attribue qu'à l'export de souveraineté) ?
5. **AC-c3, AC-c5** : un cas où la phrase neuve induit en erreur (un `curl` qui écrit sur la sortie d'erreur en
   succès ? un dossier d'autre répétition **avec** objets Docker ?).
6. Les gardes `textes_coherents` (G19, G14) et `configuration_transmise` (G14 à G16) lisent-elles un texte que le diff
   a changé de façon à les faire passer à vide ? Raisonne sur le code des gardes, n'exécute rien.

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC-c1 à AC-c10, rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve :

- **AC-c1 à AC-c5** : phrase au `.tex` **et au PDF aplati** `docs/manual/fr/admin-manual.pdf` ; « aucune commande
  touchée » — rejoue le `diff` des blocs `lstlisting` de la sous-section entre `story/32-1` et `HEAD` (par `awk` en
  pipe, substitution de processus) : vide ? Aucune consigne d'agir ajoutée (compare au partage C-32-1-15 et au texte de
  [#726]) ?
- **AC-c6** : `\needspace` avant l'amorce ; page de l'amorce et de la première ligne du listing ; capture p. 63.
- **AC-c7** : 26 remplacements (17 + 9), `grep -c "⚠"` et `grep -c $'\xef\xb8\x8f'` → 0 ; U+FFFD 26/9/16 → 0/0/0 aux
  deux bornes (borne « avant » : `git show c03d3a5f:docs/manual/fr/<pdf>` en pipe vers `pdftotext - -`) ; échantillon
  de phrases qui commencent par le signe ; témoins des replis. **Écart C-32-1-39** (repli ASCII au lieu du `literate`) :
  est-il celui que l'AC prescrit, et dit au Dev Agent Record ? Le `literate` est-il bien absent du `.sty` ?
- **AC-c8** : deux annotations, au bon endroit, datées, prémisses justes (G15 : lis `les_volumes_cites_existent` dans
  `crates/kesh-api/tests/configuration_transmise.rs`).
- **AC-c9** : `make fr` (journal), brochure, gate ciblé — les chiffres du Dev Agent Record (48 = 30 + 18 ; 1 test
  `kesh-db`) correspondent-ils **aux journaux** ? Le PDF commité est-il celui du `.tex` commité (le dernier `make fr` du
  journal est-il postérieur aux dernières modifications du `.tex` ; un `.tex` modifié sans PDF régénéré se verrait au
  texte aplati) ?
- **AC-c10** : chaque ligne du tableau #714 de la fiche a-t-elle un sort (corrigée, [#726], écartée) — et `gh issue view
  726` porte-t-il bien les points sortis ?
- **Écart C-32-1-40** : trois `\needspace` hors de ce que l'AC-c6 nomme — justifié, borné, sans effet de bord ?
- **Registre et `sprint-status.yaml`** : C-32-1-39 et 40 numérotés à la suite ; clé `32-1c-defauts-procedure-repetition`
  à `review`, commentaire daté ; aucune autre clé touchée.
- **Messages de commit** : `git log story/32-1..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
  doit ne rien rendre ; chaque commit porte `refs #714` et le `Co-Authored-By`.
- **Données réelles** : aucune donnée réelle (IBAN, solde, relevé) dans le diff.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/321c-review-p1-<B|E|A>.md` (selon ta lentille), par l'outil
d'écriture de fichier — jamais par une redirection :

- bilan en tête : CRITICAL / HIGH / MEDIUM / LOW ;
- findings numérotés `B1…`/`E1…`/`A1…`, chacun avec sévérité, fichier:ligne, **preuve copiée** (sortie `grep -nF`,
  `sed -n`, `pdftotext` aplati ou `git diff`), correctif proposé — et, s'il touche le manuel, s'il **change une commande
  ou ajoute une consigne d'agir** (alors il relève de [#726], non de la 32-1c) ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** — un « 0 finding » sans cette liste ne compte
  pas comme passe ;
- tout fichier créé par erreur, déclaré ;
- rien d'autre n'est écrit nulle part.
