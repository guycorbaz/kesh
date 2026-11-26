# Story 55-1b : la présentation des rapports groupés dans l'URL

Status: review

Epic : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md` — D4. Issue : #À-VENIR (`bug`, P3, point b, comme
la 55-1a). Release : la prochaine planifiée après la 0.20.0. **Frontend seulement, aucune migration.** **Suit la 55-1a**
(le modèle d'URL, ses deux sens, ses clés) ; **base : la PR de l'Epic 46**, qui crée la présentation.

*Découpée de la 55-1 à sa validation P3 (signal de recyclage) : la 55-1a porte l'URL des rapports, la 55-1b y ajoute la
présentation de l'Epic 46 (hiérarchie, niveau, place des sous-totaux, groupes ouverts ou fermés à la main).*

## ⚠️ D16 (2026-11-26, soir) — le rapport se charge seul

Décision de l'utilisateur et avis UX (fichier d'epic, D16 ; 55-1a, AC-20 à AC-26) : plus de bouton *Générer* sur les
onglets groupés (Bilan, Compte de résultat, Balance). Partout où cette fiche dit « *Générer* sur un onglet groupé » (le
gestionnaire qui écrit les paramètres et la vue, le tableau des compteurs, AC-5, AC-12), lire « le chargement de la
55-1a, AC-20 » — même écriture de l'URL (remplace), mêmes compteurs ; le scénario X1 de la 46-2 se rejoue avec un
changement d'exercice ou une sortie des dates au lieu du clic. Les contrôles de présentation redemandent comme
aujourd'hui (46-2), et cette redemande est une demande comme une autre : la dernière l'emporte (55-1a, AC-21). **R-4
(au T0, fiche au plafond)** : récrire ces sites, ligne lue citée au Change Log.

## ⚠️ Points renvoyés au développement (validation P6)

La validation s'est arrêtée à la **sixième passe** sur deux MEDIUM (`kesh-gate-logs/s55-kickoff/55-1b-validation-p6.md`),
**renvoyés au développement** par la règle de la sixième passe (`CLAUDE.md` § *Review Iteration Rule*) : le T0 les tranche
avant toute ligne de code, ligne lue et décision citées au Change Log ; le correctif proposé est la résolution par défaut.

- **R-1 (V6-1)** — La garde de l'écriture à la réponse (« aucune navigation en cours », « l'entrée courante porte la clé de
  la requête ») supprime, lue à la lettre, l'écriture de toute redemande par contrôle. Défaut proposé : relever
  `page.url.search` **au départ** de la requête et n'écrire à la réponse que si elle n'a pas changé ; retirer « navigation
  en cours » ; un test (*Retour* vers une URL incomplète ou un autre onglet pendant une redemande) et la mutation « garde
  omise ».
- **R-2 (V6-3)** — `keepDisplay: true` seulement si les paramètres de rapport de l'URL égalent ceux de la clé affichée,
  sinon `generate()` (chargement complet) ; un AC et une mutation. Même famille que R-1 de la 55-1a : les deux se tranchent
  ensemble.
- **R-3 (de la P7 de la 55-1a)** — L'AC-13 de la 55-1a (une adresse demande son rapport, même pendant la génération d'un
  autre onglet) prime : l'AC-12 de cette fiche est récrit au T0 — *Retour* et un lien interrompent aussi un chargement.
- **LOW à reprendre** : V6-2 (AC, ligne du tableau et mutation pour la branche « génération de l'onglet en vol »), V6-4
  (renvoyer à la règle complète du point 3 de la 55-1a), V6-5 (justification : la garde de la 46-2 ne sort pas pendant une
  redemande), V6-6 (ordre de C-55-1b-3 et -4 ; `?tab=balance-sheet&level=3` à l'AC-8). Et ceux de la P5 déjà renvoyés :
  V5-5, V5-9.

## Points de la P7 (décisions du mainteneur, au T0)

- **LOW** (`kesh-gate-logs/s55-decisions/55-1b-validation-p7.md`) : V7-1 (**tranché le 2026-11-26** par le
  mainteneur, § *Gates*) (le manuel modifié par l'AC-12 est gardé par
  `textes_coherents`, qui ne tournerait qu'en fin d'epic : la précision de la *Dérogation* proposée au mainteneur, fichier
  d'epic, question *Gardes Rust des textes*) ; V7-2 (§ *Gates* : nommer les specs de la *Dérogation*, dont les specs
  groupées de l'Epic 46 `reports-*groupes*.spec.ts`).

## Story

En tant que comptable qui lit son bilan par groupes,
je veux que l'adresse d'un bilan, d'un compte de résultat ou d'une balance porte aussi sa présentation,
afin qu'un lien copié, un rechargement ou *Retour* me rendent le rapport tel que je le regardais.

## Modules touchés (règle de splitting préventif)

Trois : `frontend/src/lib/features/reports` (`reports-url.ts`, lecture de `reports-grouping.ts`),
`frontend/src/routes/(app)/reports` (page), `docs/manual/fr/user-manual.tex` ; plus `frontend/tests/e2e`.

## Ce qui se lit au code (`pr/46-rapports-groupes` `511e61d54`, par `git show`)

- `frontend/src/lib/features/reports/reports-grouping.ts` : `GroupingView { shown, detailLevel, subtotalPosition, open,
  closed }`, `defaultGroupingView`, `isDetailLevel` (entier 1 à `MAX_DETAIL_LEVEL = 20`, ou `all`), `readIdList` (entiers
  > 0, au plus `MAX_MANUAL_GROUPS = 500`, dédoublonnés), `sanitizeGroupingView` (objet **typé** ; deux listes qui partagent
  un identifiant sont ignorées toutes deux), `readGroupingView`, `writeGroupingView` (mémoire par société et rapport),
  `groupingParamsOf`, `sameGroupingParams`, `withEcho`.
- `frontend/src/routes/(app)/reports/+page.svelte` : `groupingViews` (les contrôles, par rapport), `displayedGrouping`
  (paramètres de la dernière réponse affichée, repris par l'export), `setGroupingView` (contrôles et mémoire),
  `changeGrouping` (redemande si un rapport est affiché ou en chargement, **rien sinon**), `displayedMatchesView`,
  `resyncActiveGroupedTab`, `afterGeneration`, `acceptGrouped` (reprend l'écho de l'API ; mémorise la vue **seulement**
  quand la requête portait des paramètres de regroupement — la vue `flat` n'en porte pas) ; `GroupingControls` rendu avant
  toute génération (le contrôle existe sans rapport affiché).
- Manuel de la PR 46, paragraphe de la présentation (`user-manual.tex:4240-4250` sur la PR 46) : « Le navigateur
  **retient votre présentation** … »

## Le modèle (D4), sur celui de la 55-1a

*Récrit à la validation P3 (deux passes de MEDIUM nés des remédiations) sur une règle plus simple : **la présentation
n'entre dans l'URL que par la réponse affichée**. Les gestes de présentation n'écrivent jamais l'URL.*

### Paramètres (bilan, compte de résultat, balance)

| Paramètre | Valeur valide |
|---|---|
| `view` | `grouped`, `flat` |
| `level` | `all`, ou un entier écrit `^[1-9][0-9]*$` de 1 à 20 (`03`, `3.0`, `+3`, `ALL` refusés) |
| `subtotals` | `header`, `footer` |
| `open`, `closed` | entiers `^[1-9][0-9]*$` séparés par des virgules, au plus 500 chacun, sans identifiant commun aux deux ; `open=` vide = absent ; `1,,2` refusé |

- **Lecture** : une fonction **exportée de `reports-grouping.ts`** (neuve, `groupingFromParams(params): { view: GroupingView;
  anyValid: boolean }`) juge chaque paramètre avec les règles du module (`isDetailLevel`, `readIdList`, la règle conjointe
  des deux listes) ; `readIdList` gagne la borne `Number.isSafeInteger` (elle sert aussi la mémoire du navigateur). Aucune
  copie des règles dans `reports-url.ts`, qui ne fait que lui passer les chaînes.
- **Écriture** : vue groupée — `view=grouped`, `level`, `subtotals`, et `open`, `closed` s'ils sont non vides, en ordre
  croissant et sans doublon ; vue à plat — `view=flat` **seul** (`groupingParamsOf` rend `undefined` à plat,
  `reports-grouping.ts:144-145`). À la suite des paramètres de la 55-1a, dans l'ordre `view`, `level`, `subtotals`,
  `open`, `closed`. Forme canonique de `URLSearchParams.toString()` (la virgule s'écrit `%2C`) ; la lecture accepte la
  virgule brute.

### Ce que la page écrit, et quand

- **Seule la réponse acceptée pour l'onglet actif écrit la présentation** : l'URL est remplacée par la **clé affichée** —
  paramètres de la requête partie et présentation de la réponse (écho compris : les groupes disparus sortent des listes) —,
  clé posée dans `lastHandledKey` (55-1a). Pour un autre onglet, rien.
- **Les gestes de présentation** (contrôles, accordéons) **n'écrivent pas l'URL** : la 46-2 redemande comme aujourd'hui, et
  c'est sa réponse qui écrit. Sans rapport affiché ni chargement de ce rapport, ils ne font rien d'autre que mémoriser la
  vue (46-2) ; l'URL reste celle du dernier rapport demandé.
- **Clic sur un onglet groupé** : la clé affichée de l'onglet (55-1a), présentation comprise ; s'il n'a pas de rapport,
  `?tab=<id>` seul.
- **Bouton *Générer* sur un onglet groupé** : le gestionnaire (55-1a) écrit les paramètres du rapport **et la vue des
  contrôles** envoyée avec la requête — l'URL égale alors la clé en vol ; la réponse ne réécrit que si l'écho change la vue.
- **Règle d'écriture** (55-1a) : aucun `goto` si la recherche écrite égale `page.url.search` ; `lastHandledKey` est la clé
  canonique **avec** la présentation.

### Ce que la page lit (`applyUrl` de la 55-1a, pour un onglet groupé)

1. **Vue** — URL complète : `groupingFromParams`. `view=flat` **seul** : `{ ...groupingViews[tab], shown: false }` (les
   autres choix sont gardés, manuel de la PR 46 `:4207-4211`) ; `view=flat` accompagné d'autres paramètres : ceux-ci sont
   ignorés et retirés à la normalisation. Sinon, si `anyValid` (**au moins un champ effectivement retenu** par
   `sanitizeGroupingView` — `open=5&closed=5` seul n'en retient aucun), la vue de l'URL (absents : vue par défaut de la
   46-2) ; sinon, **la vue des contrôles** `groupingViews[tab]` (qui porte déjà la mémoire du navigateur, relue avant la
   première requête et tenue à jour par `setGroupingView`) — jamais une relecture du stockage, qui rendrait la vue par
   défaut si le stockage est refusé. La vue retenue est posée par `setGroupingView` (contrôles et mémoire). URL
   incomplète ou `/reports` : la vue ne change pas.
2. **Rapport** — on compare la clé de l'URL (paramètres et vue retenue) :
   - à la **clé en vol** de l'onglet : égale, rien (la réponse arrive et écrira l'URL) ;
   - à la **clé affichée** : égale, et une génération de l'onglet est en vol avec une **autre** clé (paramètres ou vue) :
     `invalidateInFlight()` — l'écran garde le rapport affiché, qui est celui de l'URL, **aucune requête** ; égale, rien en
     vol : rien ;
   - sinon, une génération **de cet onglet** est en vol avec une autre clé (chargement ou redemande) : `generate({
     keepDisplay: true })` s'il y a un rapport affiché, `generate()` sinon — qui la périme ; la garde de la 46-2
     (`resyncActiveGroupedTab`, qui sort dès que `loading`) ne s'en charge pas ;
   - sinon, mêmes paramètres **que la clé affichée**, autre vue → `resyncActiveGroupedTab()` décide (redemande en gardant
     l'affichage, ou à la fin d'une génération d'un autre onglet, règle de la 46-2) ; autres paramètres, ou rien d'affiché →
     `generate()` (55-1a).
   - La **normalisation** écrit la clé complète, présentation comprise (invariant de `lastHandledKey`, 55-1a).
   - L'écriture à la réponse ne se fait que si **aucune navigation n'est en cours** et si l'entrée courante porte encore la
     clé de la requête partie (sinon un *Retour* entre-temps serait réécrit).
3. **Complétion** : URL complète sans présentation (ou présentation invalide), rapport affiché et égal en paramètres :
   l'URL est remplacée par la clé affichée (1 `goto`) ; une requête : la réponse écrit l'URL (1 `goto`). URL à normaliser
   **et** à compléter : un seul `goto`.

### Compteurs (amendent l'AC-7 de la 55-1a pour les onglets groupés)

| Cas | Requêtes | `goto` |
|---|---|---|
| Contrôle de présentation, rapport affiché | 1 (la 46-2) | 1 (à la réponse) |
| Contrôle de présentation, sans rapport ni chargement | 0 | 0 |
| *Générer* sur un onglet groupé | 1 | 1 (au geste ; 0 de plus à la réponse, sauf écho) |
| Retour vers la clé affichée, rien en vol | 0 | 0 |
| Retour vers la clé affichée, redemande de l'onglet en vol | 0 (péremption) | 0 |
| Retour vers la clé en vol | 0 | 0 (la réponse égale l'URL ; 1 si l'écho la change) |
| Retour, mêmes paramètres, autre vue | 1 | 0 (idem) |
| URL sans présentation, rapport gardé égal | 0 | 1 |
| URL à normaliser, sans requête | 0 | 1 |
| URL à normaliser **et** une requête | 1 | 1 (normalisation) + 1 si la réponse diffère |
| Entrée ancienne dont un groupe a disparu | 1 si redemandée | 1 (écho) |

## Critères d'acceptation

1. **AC-1 — Lien et rechargement.** Une URL d'un bilan groupé (niveau 2, un groupe ouvert à la main, sous-totaux en pied)
   ouverte dans un nouveau contexte qui reprend les cookies de la session et un `localStorage` **vide** affiche le bilan dans
   cette présentation, contrôles remplis, sans clic ; de même au rechargement dans un contexte dont la mémoire porte une
   **autre** vue.
2. **AC-2 — L'URL prime sur la mémoire.** Avec une autre vue en mémoire pour ce rapport, l'URL l'emporte ; la mémoire prend
   ensuite la vue de l'URL — ouvrir un lien remplace la présentation retenue du destinataire.
3. **AC-3 — La mémoire du navigateur complète l'URL.** `?tab=balance-sheet&fy=E` sans présentation, bilan non gardé par la
   page : la vue des contrôles (qui porte celle du `localStorage`) s'applique, 1 requête, 1 `goto` à la réponse ; bilan gardé par la page avec cette vue :
   0 requête, 1 `goto`.
4. **AC-4 — Retour vers une autre présentation.** Bilan au niveau 2, clic sur l'onglet *Balance*, clic sur le Bilan, niveau 3,
   puis *Retour* deux fois : le Bilan **au niveau 2**, contrôles au niveau 2 ; sans la 55-1b, les contrôles restent au
   niveau 3 — le test rougit.
5. **AC-5 — Règle de la 46-2 intacte.** Scénario X1 (niveau changé sur le Bilan, *Générer* sur le Compte de résultat, **clic**
   sur le Bilan) : le Bilan est redemandé dans la vue des contrôles **à la fin** de l'autre génération, qui n'est pas
   interrompue ; l'URL porte ensuite la clé affichée. Le même par ***Retour*** vers l'entrée du Bilan : les contrôles
   prennent la vue de l'entrée (la clé affichée), **0 requête**, l'autre génération n'est pas interrompue — *Retour* écarte
   un choix de présentation qui n'était pas encore affiché.
6. **AC-6 — Les gestes n'écrivent pas l'URL.** Contrôle changé avec rapport affiché : aucun `goto` avant la réponse, un à la
   réponse, une requête, deux écritures en mémoire — une au geste (`changeGrouping`), une à la réponse (`acceptGrouped`) —,
   **aucune** du fait de l'URL (espion sur `localStorage.setItem`) ; sans rapport : 0 `goto`, 0
   requête.
7. **AC-7 — Écho.** Un groupe ouvert à la main puis disparu du plan : la réponse le retire des listes ; l'URL et la mémoire
   le retirent ; *Retour* puis *Suivant* vers cette entrée : 0 requête.
8. **AC-8 — Valeurs invalides.** `level=0`, `21`, `03`, `3.0`, `+3`, `ALL` ; `open=1,a`, `open=1,,2`,
   `open=99999999999999999999`, 501 identifiants, `closed=x` ; `view=xyz` ; `view=flat&level=3` (les autres retirés) :
   chacun ignoré, URL normalisée, aucune valeur invalide envoyée à l'API ; un paramètre invalide seul, et `open=5&closed=5`
   seul, laissent la vue des contrôles ; `open=` vide est absent.
9. **AC-9 — Vue à plat.** Lien `view=flat` sur un bilan dont les contrôles portent le niveau 2 : à plat, puis « hiérarchie
   montrée » rend le niveau 2 ; une réponse à plat écrit `view=flat` seul.
10. **AC-10 — Retour pendant une redemande de l'onglet.** Bilan au niveau 2, clic sur *Balance*, clic sur le Bilan, niveau 3
    (redemande en vol), *Retour* deux fois vers l'entrée du niveau 2 : 0 requête, la redemande en vol périmée, l'écran au
    niveau 2 sous des contrôles au niveau 2 ; de même avec une **autre période** générée en vol (période changée,
    *Générer*, *Retour*) ; *Retour* vers une URL égale à la clé en vol : 0 requête.
11. **AC-11 — Export.** Pendant une redemande ou un chargement lancé par *Retour*, PDF et CSV sont désactivés (`canExport`,
    PR 46 `+page.svelte:575-577`) ; à la réponse, l'export porte la présentation de l'entrée (`displayedGrouping`) — Vitest
    des deux états.
12. **AC-12 — Manuel.** Paragraphe de la présentation de la PR 46 (`:4240-4250` sur la PR 46) : l'adresse porte la
    présentation affichée, qui prime sur celle que retient le navigateur, et ouvrir un lien ou revenir par *Retour*
    remplace la présentation retenue ; la phrase « changer la présentation pendant le chargement d'un autre rapport
    interrompt ce chargement » est précisée : un geste de présentation, non *Retour* ni un lien ; PDF de la base à contrôler
    au T0 ; paragraphe « Naviguer dans les rapports » de la 55-1a : la présentation s'ajoute à « ce qu'elle retient ».
    `.tex` seul.

## Tests à écrire, et la preuve du rouge

- **Vitest** `reports-grouping.test.ts` (PR 46) : `groupingFromParams`, chaque valeur de l'AC-8, `anyValid` ; borne de
  `readIdList`.
- **Vitest** `reports-url.test.ts` (55-1a) : écriture groupée et à plat, ordre et unicité des listes, virgule brute et
  encodée, aller-retour canonique.
- **Vitest de la page** (`reports-page-url.test.ts`, helper de la 55-1a) : AC-2 à AC-11, chaque ligne du tableau des
  compteurs, flèches du clavier comme le clic ; stockage refusé puis *Retour* vers une entrée sans présentation (contrôles
  inchangés) ; `open=5&closed=5` seul (`anyValid` faux).
- **E2E** `reports-navigation.spec.ts`, bloc 55-1b : AC-1 et AC-4, sur un plan groupé (helper
  `tests/e2e/helpers/reports-groupes.ts` de la PR 46).
- **Preuve du rouge** (journal `kesh-gate-logs/s55-1b/`) et **mutations**, chacune tuée par un test nommé : mémoire
  appliquée malgré l'URL (AC-2) ; `view=flat` qui remet les autres champs aux défauts (AC-9) ; écho non reporté dans l'URL
  (AC-7) ; un geste de présentation qui écrit l'URL (AC-6, compteur de `goto` avant la réponse) ; redemande immédiate au
  lieu de `resyncActiveGroupedTab` (AC-5) ; péremption omise quand l'URL égale la clé affichée (AC-10, l'écran passe au
  niveau 3) ; clé en vol sans présentation (AC-10).

## Recette visuelle

Captures (`kesh-gate-logs/s55-1b/`) : bilan groupé avant le lien ; après *Retour* ; lien collé dans un nouveau contexte.

## Tâches

- [x] **T0** — Base `epic/55-navigation` avec la 55-1a.
- [x] **T1** — Tests rouges.
- [x] **T2** — `groupingFromParams` et borne de `readIdList` (`reports-grouping.ts`) ; présentation dans `reports-url.ts` ;
      écriture à la réponse ; lecture.
- [x] **T3** — Manuel, E2E, captures ; gates.

## Gates

`cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build` ; E2E des Rapports et
de la navigation au dernier commit de code ; suite complète et `scripts/gate-ci.sh` en fin d'epic, story **purement écran** (fichier d'epic, § *Gates*,
*Dérogation*, Q6 tranchée par le mainteneur le 2026-11-25). **À la clôture de la story**, comme elle modifie le manuel, aussi `cargo nextest run -p kesh-i18n` et
`cargo nextest run -p kesh-api -E 'binary(textes_coherents)'` (resserrement de la dérogation, décision du mainteneur du
2026-11-26, fichier d'epic § *Dérogation*).

## Note de version

*Rapports* : l'adresse d'un bilan, d'un compte de résultat ou d'une balance porte aussi sa présentation (hiérarchie,
niveau, sous-totaux, groupes ouverts ou fermés à la main) ; elle prime sur celle que retient le navigateur, et ouvrir un
lien ou revenir par *Retour* remplace la présentation retenue. (refs #À-VENIR)

## Dev Notes

### Choix et écarts

- **C-55-1b-1** — Tout ou rien entre l'URL et la mémoire (mêler un niveau de l'URL et des listes de la mémoire donnerait
  une vue que personne n'a affichée), **sauf `view=flat` seul**, qui garde les autres choix comme le fait la 46-2.
- **C-55-1b-2** — La présentation n'entre dans l'URL que par la réponse affichée (validation P3) : l'URL ne décrit jamais
  une vue que l'écran n'affiche pas, et un geste ne peut pas être réappliqué par l'effet. *Écarté* : écrire la vue des
  contrôles au geste (P1, P2) — elle divergeait de la requête partie, qui lit le formulaire.
- **C-55-1b-4** — Une URL **incomplète** qui porte une présentation (`?tab=balance-sheet&level=3`) : la présentation est
  retirée à la normalisation (rien n'est affiché, rien ne s'écrit). Un lien d'une autre société porte un `fy` inconnu :
  l'URL devient incomplète, rien n'est demandé.
- **C-55-1b-3** — Longueur : au plus 500 + 500 identifiants, soit environ 9 Ko d'URL avec `%2C` — au-delà des 8 Ko de
  certains mandataires inverses ; cas théorique (500 groupes ouverts à la main), assumé. Un lien ouvert dans une autre société porte
  des identifiants inconnus d'elle : l'API les ignore et l'écho les retire (46-1), sans erreur.

### References

- Fiche 55-1a ; fiche 46-2, AC-3, AC-5, AC-6, AC-10.

## Dev Agent Record

### Agent Model Used

Opus (agent de développement de la story), 2026-11-27.

### Completion Notes List

- **Modèle livré** (sur celui de la 55-1a, étendu sans réécriture) : la clé d'un onglet groupé — affichée, en vol, des
  champs — porte la présentation (`classicKey(tab, q, presentation)`, `buildReportSearch` gagne `presentation`) ; la
  clé affichée prend celle de la réponse (`displayedGrouping`, écho compris). Lecture : `groupingFromParams`
  (`reports-grouping.ts`, neuve, règles du module : `isDetailLevel`, `readIdList`, `sanitizeGroupingView`, aucune
  copie) appelée par `parseReportUrl` (`presentation: GroupingParams | 'flat' | null`). Écriture : `view`, `level`,
  `subtotals`, `open`, `closed` (croissants, sans doublon, `%2C`) ou `view=flat` seul ; `reportPart` retire la
  présentation pour comparer deux clés « même rapport ». `readIdList` borné par `Number.isSafeInteger`.
- **Page** : `applyPresentation` (adresse complète d'un onglet groupé : `view=flat` garde les autres choix, présentation
  valide prise tout entière, sinon la vue des contrôles ; `setGroupingView`, donc la mémoire) ; `writeUrlAtResponse`
  (réponse acceptée d'un onglet groupé actif, garde R-1 par `reportPart(lastHandledKey)`) ; `urlForTab` rend la clé
  affichée tant que la présentation n'est pas celle de la demande en vol des champs (C-55-1b-2) ; `loadActiveIfNeeded({
  byClick })` : au clic, présentation seule différente → règle de la 46-2 (AC-5) ; *Retour* et lien demandent (R-3).
  Corrections relevées au développement : `changeGrouping` pendant un **rechargement** de l'onglet garde la période en
  vol (T0) ; `resyncActiveGroupedTab` ne redemande plus pendant une demande de l'onglet (`inFlightTab !== activeTab`) —
  sans quoi une adresse égale à la clé en vol (AC-10) relançait la même requête (V6-5) ; `canExport` compare les clés
  sans la présentation (l'export reprend la présentation affichée, 46-2 AC-6, S3 du Vitest de la 46-2).
- **Tests existants adaptés, et pourquoi** : `reports-page-url.test.ts` (55-1a) — les adresses attendues des onglets
  groupés portent la présentation (`PRES`) ; « adresse complète collée » : une normalisation (présentation ajoutée)
  au lieu de 0 `goto` ; X1 : l'adresse finale porte `level=1` (écrite par la réponse, AC-5). `reports-url.test.ts` :
  `presentation: null` dans les deux `toEqual` de lecture. E2E `reports-navigation.spec.ts` AC-5 (55-1a) : l'adresse
  normalisée porte la présentation (relevé par le premier passage E2E, seul échec, aucun code de production touché).
- **Preuve du rouge** : le bloc 55-1b de `reports-page-url.test.ts` joué sur `+page.svelte`, `reports-url.ts` et
  `reports-grouping.ts` de la 55-1a (`5645d6890`) : **35 rouges sur 38** (`kesh-gate-logs/s55-dev/55-1b/rouge-vitest/avant.log`) ;
  les trois verts sont attendus (sans rapport, rien ne se passait déjà ; R-1 : la 55-1a n'écrivait rien à la réponse).
  **Mutations** : **14 sur 14 tuées**, chacune par une assertion nommée, copie non mutée verte d'abord
  (`…/55-1b/mutations/run.py`, `resultat.log`) — mémoire appliquée malgré l'URL, `view=flat` aux défauts, écho non
  reporté, geste qui écrit l'URL, redemande immédiate au clic, péremption omise (clé affichée), clé en vol sans
  présentation, garde R-1 omise, période affichée redemandée pendant un rechargement, borne `isSafeInteger` retirée,
  troisième clé non demandée (V6-2), export bloqué par la présentation, `resyncActiveGroupedTab` pendant une demande de
  l'onglet, `anyValid` vrai pour des listes ignorées.
- **Tests neufs** (exécutés, de `5645d6890` à la tête) : `reports-grouping.test.ts` 20, `reports-url.test.ts` 9,
  `reports-page-url.test.ts` 38 — **67 Vitest** (5439 → 5506 au gate front, recompté) ; `reports-navigation.spec.ts` 1
  E2E (AC-1, AC-2 au rechargement, AC-4).
- **Gates** (au dernier commit de code `c3f28d2f9`, specs E2E et manuel retouchés ensuite, aucun code de production) :
  `clippy.sh 55-1b/final` fmt rc=0, clippy rc=0 ; `nextest -p kesh-i18n` **42/42** ; `nextest -p kesh-api -E
  'binary(textes_coherents)'` **34/34** (`55-1b/textes`, rejoué après la dernière retouche du manuel : `55-1b/textes-2`) ;
  front (`55-1b/front`) : lint-i18n-ownership vert, Vitest **5506 passés (314 fichiers)**, build vert, check **3 erreurs**
  de type dans la spec E2E neuve (`selectOption` rend `string[]`) corrigées (`07ff47e62`), front rejoué
  (`55-1b/front-2`) : check **0 erreur**, lint-i18n-ownership vert, Vitest **5506 passés (314 fichiers)**, build vert. **E2E partiel (10 specs)**, `e2e.sh 55-1b/e2e` : `reports.spec.ts`,
  `reports-navigation.spec.ts`, `reports-groupes.spec.ts`, `reports-groupes-export.spec.ts`,
  `reports-balance-groupes.spec.ts`, `reports-currency.spec.ts`, `reports-export-pdf.spec.ts` (les specs des Rapports,
  dont les specs groupées de l'Epic 46, V7-2), `homepage-charts.spec.ts`, `open-items.spec.ts`,
  `dunning-roundtrip.spec.ts` : **48 passés, 1 rouge** (`reports-navigation.spec.ts` AC-5 de la 55-1a : l'attendu de la
  spec ignorait la présentation, comportement voulu) ; spec corrigée, `e2e.sh 55-1b/e2e-nav` : **9 sur 9**. **E2E complet
  et gate-ci en fin d'epic** (dérogation du fichier d'epic). Aucun `messages.ftl` touché : `sitesTotal` inchangé.
- **Manuel** : `user-manual.tex` seul — paragraphe de la présentation (l'adresse porte la présentation affichée, prime
  sur celle du navigateur, ouvrir un lien ou *Retour* la remplace, l'adresse change à la réponse ; *Retour* et un lien
  interrompent aussi un autre chargement, R-3) ; « Naviguer dans les rapports » (la présentation s'ajoute). PDF compilé
  hors du dépôt (`/tmp`, `\og … \fg{}` de la ligne 3562 remplacés dans la copie seulement) et aplati : les trois
  passages lus.
- **Captures** (thème clair, `kesh-gate-logs/s55-dev/55-1b/captures/55-1b-*.png`, montage E2E) : bilan groupé avant le
  lien, après *Retour* deux fois, lien collé dans un nouveau contexte. Regardées : les trois montrent le Bilan au
  « Niveau 1 », sous-totaux « en pied », le groupe 19 ouvert à la main (190 visible, fermé par le niveau), la ligne
  « Total Actifs du test » en pied ; la barre d'adresse n'y figure pas (l'adresse est vérifiée par l'E2E).
- **Défauts retenus, en attente du mainteneur** : aucun choix neuf hors de ceux de la 55-1a ; R-1 (garde par
  `lastHandledKey` au lieu de `page.url` relevée au départ) et R-2 (rechargement grisé, AC-24 de la 55-1a) sont tranchés
  dans le sens de la 55-1a, à confirmer avec elle.

### Issues à ouvrir

(Points de revue non corrigés : sévérité, site, texte proposé ; source de la story de solde 55-4.)

- *(Retirée au T0 : la 55-1a a corrigé `changeGrouping`, qui redemande `displayedClassic`.)* (LOW, `bug` P3,
  préexistant) Une redemande de présentation (46-2, `changeGrouping`) lit le formulaire.
- (LOW, `bug` P3, préexistant) `resyncActiveGroupedTab` ne sort que sur `loading` : un clic sur un onglet groupé dont
  la présentation diffère de l'affichée, pendant le **rechargement** grisé d'un autre onglet (`refreshing`), redemande
  tout de suite et périme ce rechargement, au lieu d'attendre sa fin comme le dit la règle de la 46-2
  (`+page.svelte`, `resyncActiveGroupedTab`) (Epic 55, story 55-1b).
- (LOW, `technical-debt`) L'écriture de l'URL à la réponse est asynchrone : un `goto` de la réponse émis pendant une
  navigation SvelteKit encore en cours (lien de compte cliqué dans la même microtâche) la périmerait (V5-8 b) ; fenêtre
  de microtâches, aucun `load` ne lisant l'URL ; non testé (Epic 55, story 55-1b).

### File List

- `frontend/src/lib/features/reports/reports-grouping.ts`, `reports-grouping.test.ts`
- `frontend/src/lib/features/reports/reports-url.ts`, `reports-url.test.ts`
- `frontend/src/routes/(app)/reports/+page.svelte`, `reports-page-url.test.ts`
- `frontend/tests/e2e/reports-navigation.spec.ts`
- `docs/manual/fr/user-manual.tex`
- `_bmad-output/implementation-artifacts/55-1b-presentation-dans-l-url.md`, `sprint-status.yaml`

## Change Log

- 2026-11-25 — Fiche écrite au découpage de la 55-1 (validation P3), reprend D4 et les findings V1-6, V1-7, V2-11 et V3-1,
  V3-6, V3-8, V3-12 de la 55-1 (`drafted`).
- 2026-11-25 — Validation P1 (Opus) : 0/0/7/8 ; remédiée : redemande par `resyncActiveGroupedTab` (V1-1) ; clés affichée et en
  vol avec la présentation, URL de la réponse (V1-2) ; clic sans rapport, URL incomplète et `/reports` sans complétion,
  compteurs écrits (V1-3) ; `groupingFromParams` exportée de `reports-grouping.ts` (V1-4) ; virgule encodée, lecture des deux
  formes (V1-5) ; AC-4 qui rougit sans la story (V1-6) ; manuel (V1-7) ; LOW V1-8 à V1-15.
- 2026-11-25 — Validation P2 (Sonnet) : 0/0/3/6 ; remédiée : vue à plat (`view=flat` seul, autres champs gardés, AC-9bis)
  (V2-1) ; contrôle sans rapport n'écrit pas l'URL, AC-6 constatable (V2-2) ; Retour pendant une redemande de l'onglet
  (AC-9ter) (V2-3 ; le cas voisin de la 55-1a lui est signalé) ; LOW V2-4 à V2-9 (dont une issue).
- 2026-11-25 — Validation P3 ciblée (Opus) : 0/0/3/5. Modèle **récrit plus simple** plutôt que complété : la présentation
  n'entre dans l'URL que par la réponse affichée, les gestes de présentation n'écrivent plus (V3-3, V3-4, V3-6, C-55-1b-2) ;
  exception `view=flat` reportée à la lecture et à C-55-1b-1, `view=flat` accompagné tranché (V3-1) ; Retour vers la clé
  affichée pendant une génération de l'onglet avec une autre clé : péremption sans requête, généralisée aux paramètres
  (V3-2, V3-5 ; le cas des rapports non groupés est écrit à la 55-1a) ; borne dans `readIdList`, AC-8 complété, tableau des
  compteurs (V3-7) ; AC renumérotés, citation `:4207-4211` (V3-8). Prochaine passe : P4 ; dès la P6, retrait seulement.
- 2026-11-25 — Validation P4 (Sonnet) : 0/0/4/5 ; remédiée : AC-5 restreinte au clic, attendu de *Retour* écrit (V4-1) ;
  tableau des compteurs corrigé (règle « écriture identique : pas de `goto` », *Générer*, normalisation et requête) (V4-2) ;
  deux écritures en mémoire (V4-3) ; vue des contrôles au lieu d'une relecture du stockage (V4-4) ; LOW V4-5 à V4-9 (fichier
  d'epic : D2, D4, fichiers communs). Prochaine passe : P5, dernière remédiation possible de l'orchestrateur.
- 2026-11-25 — Validation P5 (Opus) : 0/0/2/7 ; remédiée (dernière remédiation permise à l'orchestrateur) : génération de
  l'onglet en vol avec une autre clé, péremption par `generate` (V5-1 ; même trou que R-1 de la 55-1a, renvoyé au
  développement là-bas) ; AC-11 sur des états atteignables (V5-2) ; LOW V5-3, V5-4, V5-6 à V5-8 ; V5-5 (deux lignes du tableau)
  et V5-9 (AC-10, AC-7, AC-1 à préciser) **renvoyés au T0**. Dès la P6, retrait ou renvoi seulement.
- 2026-11-25 — Validation P6 ciblée (Sonnet) : 0/0/2/4. **Arrêt de la boucle par renvoi au développement** (R-1, R-2).
  Tendance : P1 Opus 0/0/7/8 → P2 Sonnet 0/0/3/6 → P3 ciblée Opus 0/0/3/5 (modèle récrit) → P4 Sonnet 0/0/4/5 → P5 Opus
  0/0/2/7 → P6 ciblée Sonnet 0/0/2/4. `ready-for-dev` **sous réserve** du T0 (R-1, R-2).
- 2026-11-25 — **Décision du mainteneur versée** (Q6, dérogation partielle) : § *Gates* aligné sur la *Dérogation* du
  fichier d'epic. Passe P7 ciblée sur ce changement ; au-delà de la sixième passe, ses points vont au T0 (R-n).
- 2026-11-25 — Validation P7 ciblée (Opus) sur la décision Q6 : 0/0/0/2 — **validation terminée** sur ce changement ; les
  deux LOW vont au T0 (section *Points de la P7*). `ready-for-dev` sous réserve du T0 (R-1, R-2, et ces LOW).
- 2026-11-26 — **Décision du mainteneur du 2026-11-26 versée** (resserrement de la dérogation Q6) : § *Gates* — à la
  clôture, `nextest -p kesh-i18n` et `binary(textes_coherents)` ; V7-1 tranché. Passe P8 ciblée (la dernière permise ;
  au-delà de la sixième, ses points vont au T0).
- 2026-11-26 — Validation P8 ciblée (Sonnet, `kesh-gate-logs/s55-decisions-2/55-2-55-1b-validation-p8.md`, commune aux 55-2 et 55-1b) sur les
  décisions du 2026-11-26 : 0/0/0/1 — P8-3 (V7-1 au passé, après « tranché »). **Validation terminée**, dernière passe permise ; les LOW se reprennent au **T0**, ligne
  lue citée au Change Log.
- 2026-11-26 (soir) — **D16 versée** (le rapport se charge seul) : note d'en-tête et R-4 au T0, sans passe de
  remédiation (fiche au plafond) ; relue par la passe ciblée hors décompte du même jour.
- 2026-11-27 — **T0 du développement** (base `epic/55-navigation` `5645d6890`, 55-1a développée, statut `review` ; lignes
  relevées sur cette base). Chaque point tranché avant le code, ligne lue citée :
  - **Base** : `+page.svelte` 1405 lignes ; `changeGrouping` `:270-287` (`if (loadingThis) { void generate({ keepDisplay:
    false, classicQuery: inFlightClassic ?? undefined }); } else if (displayed) { void generate({ keepDisplay: true,
    classicQuery: displayedClassic[report] }); }`) ; `resyncActiveGroupedTab` `:308-314` (`if (displayed && !loading &&
    !displayedMatchesView(activeTab))`) ; `acceptGrouped` `:336-356` ; `generate` `:521-687`, `keepDisplay` `:614`
    (`options.keepDisplay ?? isDisplayed(myTab)`) ; `loadActiveIfNeeded` `:719-730` ; `writeUrl` `:964-968`
    (`lastHandledKey = search; if (page.url.search.replace(/^\?/, '') === search) return;`) ; `applyUrl` `:976-1010` ;
    `selectTab` `:1044-1054`. `reports-grouping.ts:75-79` `readIdList` (`Number.isInteger(e) && e > 0`, sans borne
    `Number.isSafeInteger`). `reports-url.ts` `buildReportSearch` `:115-125`. Manuel : présentation `user-manual.tex:4300-4313`
    (« Changer la présentation d'un rapport pendant le chargement d'un autre rapport interrompt ce chargement » `:4307`,
    « retient votre présentation » `:4309`) ; « Naviguer dans les rapports » `:4569-4577`.
  - **R-4 (D16)** — Partout où la fiche dit « *Générer* sur un onglet groupé », lire le chargement de la 55-1a (AC-20) :
    `commitPeriod` (`:1059-1064`, `if (loadActiveIfNeeded()) writeUrl(urlForTab(activeTab), true)`), le clic
    (`selectTab`) et l'adresse appliquée (`applyUrl`). Ces gestes écrivent la clé des champs **avec la présentation des
    contrôles envoyée** : l'URL égale la clé en vol, la réponse ne réécrit que si l'écho la change (§ *Ce que la page écrit*,
    ligne *Générer*). X1 de la 46-2 se rejoue par un changement d'exercice ou une sortie des dates.
  - **R-3** — Tranché dans le sens de la 55-1a (Change Log du T0 de la 55-1a, R-1 à R-3 : « sinon `generate()`, qui périme
    toute demande en vol, de quelque onglet qu'elle soit (AC-13, R-3 : une adresse pendant le chargement d'un autre onglet
    demande son rapport ») : *Retour* et un lien vers une **autre** présentation, mêmes paramètres, **demandent** le
    rapport (`generate()`) et périment le chargement d'un autre onglet. La règle de la 46-2 (attendre la fin de l'autre
    génération, `resyncActiveGroupedTab`) ne reste que pour le **clic** sur un onglet (AC-5, premier cas ; AC-22 de la 55-1a
    « elle ne s'applique qu'à un rapport repris de la mémoire ») : au clic, une clé affichée qui ne diffère des champs que
    par la présentation est reprise de la mémoire, sans requête, et `resyncActiveGroupedTab` décide. AC-12 récrit : la
    phrase `:4307` reste vraie (un geste de présentation interrompt), et « Naviguer dans les rapports » dit déjà que
    *Retour* et un lien abandonnent l'autre chargement (`:4575-4576`) ; la précision « un geste, non *Retour* » de l'AC-12
    tombe.
  - **R-2 (V6-3)** — Tranché dans le sens de la 55-1a (AC-24 : « **rechargement** : l'ancien rapport reste affiché,
    grisé », `:614` `keepDisplay = options.keepDisplay ?? isDisplayed(myTab)`) : toute demande d'un onglet qui affiche déjà
    un rapport le garde, grisé (`aria-busy`), quels que soient les paramètres — c'est le défaut de `generate()`, qui n'est
    pas changé. Le « chargement complet » proposé par R-2 contredirait l'AC-24 de la 55-1a. Constat : Vitest *Retour* vers
    d'autres paramètres pendant une redemande (rapport gardé, `aria-busy`, puis la réponse des paramètres de l'URL).
  - **R-1 (V6-1)** — La garde « aucune navigation en cours » est retirée (défaut). Le relevé de `page.url.search` **au
    départ** de la requête ne tient pas : le clic, le menu et la sortie des dates lancent `generate()` **avant** leur
    `writeUrl` (`selectTab` `:1048-1053`, `commitPeriod` `:1063`), si bien que l'adresse relevée au départ est celle d'avant
    le geste. **Garde retenue** : à la réponse acceptée d'un onglet groupé, l'URL n'est remplacée que si l'onglet est
    l'onglet actif **et** si la dernière adresse écrite ou appliquée par la page (`lastHandledKey`, posée de façon synchrone
    par `writeUrl` et par l'effet) désigne **le même rapport** que la requête (onglet, exercice, période ; la présentation
    exceptée). Une navigation vers un autre onglet, vers des dates « Plusieurs » / « Hors exercice », ou vers d'autres
    paramètres change cette adresse : rien n'est écrit. Une navigation vers le même rapport, autre présentation, périme la
    requête (clé affichée égale) ou en lance une autre (autre clé) : la réponse périmée n'écrit rien. Tests : *Retour* vers
    un autre onglet et vers une URL « Plusieurs » pendant une redemande (URL intacte) ; mutation « garde omise ».
  - **Interaction relevée au T0 (corrigée dans la story)** — `changeGrouping` pendant un **rechargement** de l'onglet
    (`refreshing`, autres paramètres en vol : sortie des dates puis contrôle changé) prend la branche `displayed` et
    redemande `displayedClassic` (`:285`) : la période en vol est perdue. Correctif : toute demande **de cet onglet** en vol
    (chargement ou rechargement) est redemandée avec sa requête (`inFlightClassic`) et la vue neuve. Vitest et mutation.
  - **LOW** : V6-2 (troisième clé pendant une génération de l'onglet) → AC-10 complété d'une clause, test et mutation
    « génération en vol non périmée » (une troisième clé demande et périme : la réponse ancienne ne s'affiche pas) ; V6-4
    → sans objet sous D16 (la 55-1a ne laisse plus « aucune requête » quand un autre onglet génère) ; V6-5 → juste
    (`:311` ne sort que sur `loading`) ; la péremption passe par `loadActiveIfNeeded` (`:724-726`), non par
    `resyncActiveGroupedTab` ; V6-6 → C-55-1b-4 sous D16 : une URL **incomplète** d'un onglet groupé charge le rapport des
    champs (55-1a, R-14) et la normalisation écrit la clé du rapport chargé, présentation des contrôles comprise ; la
    présentation de l'adresse incomplète n'est pas lue (Vitest `?tab=balance-sheet&level=3`) ; V5-5 (a) clic, contrôles
    différents, rien en vol : 1 requête (`resyncActiveGroupedTab`), 2 `goto` (le clic empile la clé affichée, la réponse
    la remplace) ; (b) contrôle pendant le chargement de ce rapport : 1 requête de plus, 1 `goto` à la réponse ; V5-9 :
    AC-10 « *Retour* deux fois » pour les deux cas, AC-7 entrée voisine sur la *Balance*, AC-1 autre vue posée par
    `page.evaluate` sur `localStorage` avant `page.reload()` ; V5-7, V5-8 absorbés (R-1, C-55-1b-4) ; V7-1 tranché
    (gardes Rust à la clôture), V7-2 : les specs `reports*.spec.ts` couvrent `reports-groupes.spec.ts`,
    `reports-groupes-export.spec.ts`, `reports-balance-groupes.spec.ts`.
  - **Issue préexistante de la fiche** (« une redemande de présentation lit le formulaire ») : **retirée**, la 55-1a l'a
    corrigée (`:285`, `classicQuery: displayedClassic[report]`).
  - **Modèle de clé** : la clé d'un onglet groupé (affichée, en vol, des champs) est la recherche canonique **avec** la
    présentation : affichée = paramètres de la réponse acceptée et écho (`displayedGrouping`) ; en vol = présentation
    envoyée ; des champs = présentation des contrôles. Lecture `groupingFromParams` (`reports-grouping.ts`), écriture
    `buildReportSearch` (`reports-url.ts`), ordre `view`, `level`, `subtotals`, `open`, `closed`.
- 2026-11-27 — **Développement** (T1 à T3) : `groupingFromParams` et borne de `readIdList` ; présentation dans
  `reports-url.ts` (lecture, écriture, `reportPart`) ; page : clés avec présentation, application d'une adresse, écriture
  à la réponse (garde R-1), clic et règle de la 46-2 ; trois corrections d'interaction (`changeGrouping` pendant un
  rechargement, `resyncActiveGroupedTab` pendant une demande de l'onglet, `canExport` sans la présentation) ; manuel.
  67 Vitest neufs, 1 E2E neuf, 14 mutations tuées, 35 rouges sur 38 avant. Gates : fmt, clippy, `kesh-i18n` 42/42,
  `textes_coherents` 34/34, front vert, E2E partiel 10 specs (48 + 9 sur la spec corrigée). Statut `review` (aucune revue
  de code : accord du mainteneur attendu).
