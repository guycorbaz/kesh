# Passe de validation ciblée — Epic 55, décisions D16 et D17 (2026-11-26, soir)

Passe **ciblée**, une lentille, contexte frais, modèle **Opus**. Hors décompte : les fiches 55-1a, 55-1b, 55-2 et 55-3b
sont au plafond de huit passes, le fichier d'epic au-delà de la sixième. **Aucune remédiation ne suivra** : chaque point
au-dessus de LOW ira au T0 de la fiche (`CLAUDE.md` § *Review Iteration Rule*).

## Ce qui est relu

Le seul changement des commits `2af5bd39a` et `15cc3c3f8` de la branche `epic/55-navigation` (base `ffd809529`) :
`git -C <worktree> diff ffd809529 15cc3c3f8`. Fichiers :

- `_bmad-output/planning-artifacts/epic-55-navigation-interface.md` (D2, D3, D15 annotées ; D16, D17 neuves ; *Objectif*,
  *Stories*, *Fichiers communs*, section *Décisions de l'utilisateur et de l'UX (2026-11-26, soir)*, *Questions*) ;
- `_bmad-output/implementation-artifacts/55-1a-etat-des-rapports-dans-l-url.md` (R-14, R-15 ; AC-20 à AC-26 ; annotations
  d'AC-2, AC-3, AC-7, AC-13, AC-15, AC-16, du tableau des gestes, de C-55-1-4, C-55-1-8 ; C-55-1-10 ; note de version) ;
- `55-2-lien-de-compte-partout.md` (R-6, AC-11 et AC-12 amendés, AC-16 neuf, T2bis, note de version) ;
- `55-1b-presentation-dans-l-url.md` (note D16, R-4) ; `55-3b-justificatifs-ecran.md` (note à l'AC-5, R-11) ;
- `sprint-status.yaml` (commentaires du bloc 55).

## Les décisions à verser (texte de référence, transmis par le coordinateur)

1. **Plus de bouton *Générer*** pour les rapports qui ont une période (bilan, compte de résultat, balance, journaux, TVA,
   Grand livre filtré sur un compte). Le rapport se charge : à l'ouverture de la page (un exercice est toujours choisi) ;
   au changement d'exercice dans le menu ; quand le focus quitte **l'ensemble** des deux champs de date, regroupés
   (`<fieldset>` ou `role="group"` nommé « Période »), `focusout` avec `relatedTarget` hors du groupe (passer de « début »
   à « fin » ne déclenche rien) ; à Entrée dans un champ de date. Seulement si les dates ont changé depuis la dernière
   requête, et si la période est valide (début avant fin). La dernière demande l'emporte, les précédentes sont annulées.
   Des dates tapées puis quittées par un clic sur un autre onglet passent dans l'URL de cet onglet. **Exception** : le
   Grand livre **non filtré** garde un bouton. **États** : « Chargement… » en `role="status"` ; pendant un rechargement,
   l'ancien rapport reste affiché, grisé, `aria-busy` ; sans ligne, « Aucune écriture sur cette période. ». Le texte
   « Sélectionnez un exercice et cliquez sur Générer » disparaît ; seul reste `reports-error-no-fiscal-year-available`.
   Le **rapport périmé** (AC-16 de la 55-1a) devient le simple grisé de rechargement. « Plusieurs » ou « Hors exercice »
   sur un onglet limité à un exercice : rien n'est chargé, un message dit pourquoi.
2. **Style des liens**, partout, tableaux compris : la couleur actuelle **et** un soulignement léger, `underline
   decoration-indigo-700/40 underline-offset-2 hover:decoration-current` (WCAG 1.4.1). `AccountLink` le porte.

## Axes à exercer (tous, et déclarer lesquels l'ont été)

1. **Fidélité** : chaque point des décisions est-il versé, sans ajout présenté comme décidé ? Les choix des fiches que la
   décision ne tranche pas sont-ils marqués comme tels et posés au mainteneur (R-15, *Questions*) ?
2. **Contradictions laissées** : greper sur les quatre fiches et le fichier d'epic `Générer`, `invite`, `périmé`,
   `reports-nav-stale`, `aucune requête`, `génération en vol` ; chaque site qui contredit D16 est-il soit récrit, soit
   annoté, soit nommé par R-14 (55-1a), R-4 (55-1b), R-6 (55-2), R-11 (55-3b) ? Un site oublié est un finding.
3. **Au code** (lecture seule, `<worktree>` ; lignes à `fbfa64a27` ou à la base de la branche) : les clés citées existent
   ou non comme dit (`crates/kesh-i18n/locales/fr-CH/messages.ftl` : `reports-loading`, `reports-filter-period`,
   `reports-error-no-entries-in-period`, `reports-instruction-select-and-generate`, `reports-aged-instruction-generate`,
   `reports-ledger-instruction`) ; les sites de l'inventaire du balayage (AC-16 de la 55-2) sont-ils justes et complets
   selon sa propre règle (`<a>` sans `underline` au repos, hors boutons et navigation) ; la variante `link` de
   `components/ui/button/button.svelte` ; Tailwind 4 et `decoration-current/40` ; `focusout`/`relatedTarget` (nul quand
   le focus quitte la fenêtre ou va sur un élément non focalisable) ; un `AbortController` passable aux fonctions de
   `reports.api.ts`.
4. **Modèle de la 55-1a** : l'AC-21 (« la dernière demande l'emporte », toutes sortes d'onglets) et l'AC-22 (le clic sur
   un onglet charge) sont-ils compatibles avec *Appliquer une URL* (points 3 et 4 : clés affichée et en vol), avec la
   règle de la 46-2 (`resyncActiveGroupedTab`) et avec D3 (une URL venue de l'extérieur) ? Le `focusout` vers un onglet
   (AC-22) est-il décidable par `relatedTarget` ? Le cas d'une date vide, d'Entrée sur une date incomplète, d'un exercice
   choisi pendant une saisie.
5. **Accessibilité** : `<fieldset>`/`<legend>` masquée, `role="status"` et `aria-busy`, focus après un clic d'onglet ;
   contraste du soulignement à 40 % (1.4.1, 1.4.11).
6. **Manuel** : les phrases du `.tex` (§ *Rapports comptables*) que D16 rend fausses sont-elles nommées (AC-11, AC-19
   amendés) ? Contrôler le PDF aplati : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o
   "cliquez sur Générer[^.]*"` (lecture seule).
7. **Décomptes** : nombre de clés neuves (AC-18 amendé : dix), modules déclarés (55-1a : cinq, D16 n'en ajoute aucun ;
   55-2 : six modules de balayage), numérotation des AC et des R sans doublon dans chaque fiche.
8. **Interaction avec l'Epic 56** (`epic/56-liens-de-compte`, lecture par `git -C <worktree-56> show HEAD:<fichier>`) :
   `LINK_CLASS` (56-1 la plaçait dans `highlight.ts`), Grand livre filtré auto-chargé et le paramètre `entryId` de
   l'Epic 56.

## Interdits

Aucune écriture dans les dépôts ni dans les worktrees ; aucune commande qui écrit (`git commit`, `git checkout`, `git
stash`, `scripts/prepare-release.sh`, `npm`, `cargo`, `latexmk`, `make`) ; aucun `gh`, aucun réseau. Lecture : `git show`,
`git diff`, `grep`, `sed -n`, `pdftotext` vers la sortie standard.

## Rapport

Un seul fichier, `$GATE_LOGS/s55-56-decisions/epic-55-d16-d17-validation.md` : findings numérotés `DE-n`, chacun avec
sévérité (CRITICAL, HIGH, MEDIUM, LOW), fiche et site (`fichier:ligne`), constat, preuve (ligne lue citée) et correctif
proposé ; puis la **liste des axes exercés et non exercés** ; puis le décompte par fiche (C/H/M/L). Un « 0 » sans cette
liste ne compte pas.
