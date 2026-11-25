# Story 55-1b : la présentation des rapports groupés dans l'URL

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md` — D4. Issue : #À-VENIR (`bug`, P3, point b, comme
la 55-1a). Release : la prochaine planifiée après la 0.20.0. **Frontend seulement, aucune migration.** **Suit la 55-1a**
(le modèle d'URL, ses deux sens, ses clés) ; **base : la PR de l'Epic 46**, qui crée la présentation.

*Découpée de la 55-1 à sa validation P3 (signal de recyclage) : la 55-1a porte l'URL des rapports, la 55-1b y ajoute la
présentation de l'Epic 46 (hiérarchie, niveau, place des sous-totaux, groupes ouverts ou fermés à la main).*

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

### Paramètres (bilan, compte de résultat, balance)

| Paramètre | Valeur valide |
|---|---|
| `view` | `grouped`, `flat` |
| `level` | `all`, ou un entier écrit `^[1-9][0-9]*$` de 1 à 20 (`03`, `3.0`, `+3` refusés) |
| `subtotals` | `header`, `footer` |
| `open`, `closed` | entiers `^[1-9][0-9]*$` ≤ `Number.MAX_SAFE_INTEGER` séparés par des virgules, au plus 500 chacun, sans identifiant commun aux deux ; `open=` vide = absent ; `1,,2` refusé |

- **Lecture** : une fonction **exportée de `reports-grouping.ts`** (neuve, `groupingFromParams(params): { view: GroupingView;
  anyValid: boolean }`) juge chaque paramètre avec les règles du module (`isDetailLevel`, `readIdList`, la règle conjointe
  des deux listes) : aucune copie des règles dans `reports-url.ts`, qui ne fait que lui passer les chaînes. `open=5&closed=5`
  seul : les deux listes ignorées, aucun paramètre valide (`anyValid` faux).
- **Vue à plat** : à l'écriture, `view=flat` **seul** (les gestes comme la réponse : `groupingParamsOf` rend `undefined` à
  plat, la clé affichée n'a pas d'autre champ) ; à la lecture, `view=flat` masque la hiérarchie **et garde les autres
  champs** de la vue en cours (contrôles, sinon mémoire du navigateur) — « vos autres choix sont gardés pendant que la
  hiérarchie est masquée » (manuel de la PR 46 `:4207-4210`) ; exception assumée au « tout ou rien ».
- **Écriture** (vue groupée) : `view`, `level`, `subtotals`, et `open`, `closed` s'ils sont non vides, **en ordre croissant et sans
  doublon**, à la suite des paramètres de la 55-1a, dans l'ordre `view`, `level`, `subtotals`, `open`, `closed`. Forme
  canonique : celle de `URLSearchParams.toString()` de la 55-1a, qui encode la virgule (`open=1%2C2`) ; la lecture accepte
  `open=1,2` comme `open=1%2C2`, et une URL collée avec des virgules brutes est normalisée (un `goto` qui remplace).

### Clés d'un onglet groupé (extension du point 4 de la 55-1a)

- La **clé affichée** = paramètres du rapport **et** présentation de la réponse affichée (`displayedGrouping`, écho
  compris). La **clé en vol** = paramètres du rapport **et** présentation **envoyée** avec la requête.
- Après une réponse acceptée pour l'**onglet actif**, l'URL est remplacée par la clé affichée (écho : les groupes disparus
  sortent des listes), clé enregistrée comme écrite par la page ; pour un autre onglet, rien.

### La page écrit

| Geste | URL | Requêtes, `goto` |
|---|---|---|
| Contrôles de présentation, accordéons, **rapport affiché** | **remplace** par les paramètres du rapport affiché et la vue **des contrôles**, clé enregistrée (la 55-1a ignore l'URL écrite par la page) ; la 46-2 redemande comme aujourd'hui | 0 requête de l'URL, 1 `goto` |
| Contrôles de présentation **sans** rapport affiché | **aucune** (l'URL ne décrit aucun rapport) ; la 46-2 ne demande rien, sauf chargement **de ce même rapport** | 0, 0 |
| Clic ou flèche sur un onglet groupé **avec** un rapport affiché | la clé de la 55-1a, présentation = vue **des contrôles** ; la règle de la 46-2 (`resyncActiveGroupedTab`) redemande si la vue affichée diffère | 0 requête de l'URL, 1 `goto` |
| Clic ou flèche sur un onglet groupé **sans** rapport | `?tab=<id>` seul, sans présentation (55-1a) | 0, 1 |
| Réponse acceptée, onglet actif | **remplace** par la clé affichée | 1 `goto` (aucun si l'URL est déjà celle-ci) |

### La page lit (`applyUrl` de la 55-1a, pour un onglet groupé)

- **Vue** — URL **complète** : `groupingFromParams` ; si `anyValid`, la vue de l'URL (les absents prennent la vue par défaut
  de la 46-2, non la mémoire) est posée par `setGroupingView` (contrôles **et** mémoire, vue `flat` comprise) ; sinon, la vue
  mémorisée (`readGroupingView`) est posée aux contrôles. URL **incomplète** ou `/reports` : la vue des contrôles ne change
  pas, aucune complétion.
- **Rapport** (point 3 de la 55-1a, clés de ce document) : si la **clé en vol** de l'onglet égale l'URL, rien (la réponse
  arrive) ; si la **clé affichée** égale l'URL **et qu'aucune génération de cet onglet n'est en vol** avec une autre
  présentation, rien ; si la clé affichée égale l'URL mais qu'une redemande de cet onglet est en vol avec une autre vue
  (niveau changé, puis *Retour*), la vue est posée et le rapport redemandé (`generate({ keepDisplay: true })`), qui périme
  la redemande en vol ; **mêmes paramètres de rapport, autre vue** : la vue est posée, puis
  `resyncActiveGroupedTab()` décide — redemande en gardant l'affichage si aucune génération n'est en vol, **à la fin de
  l'autre génération** sinon (règle de la 46-2, manuel de la PR 46 `:4240-4242`) ; autres paramètres : `generate()` (55-1a).
- **Complétion** : URL complète sans présentation valide, rapport affiché pris de la mémoire : l'URL est remplacée par la
  clé (1 `goto`) ; si une requête part, la complétion se fait à sa réponse (ligne « Réponse acceptée », 1 `goto`). Ces
  `goto` **amendent** l'AC-7 de la 55-1a pour les onglets groupés : normalisation 1, complétion 1, Retour vers une URL
  canonique et complète 0.
- **Paramètres de la redemande** : une redemande de présentation (46-2, `changeGrouping` → `generate`) lit le formulaire,
  non la clé affichée ; un champ modifié sans *Générer* passe donc dans la redemande (comportement de la PR 46). L'URL
  écrite à la réponse est la clé de la requête partie, donc vraie ; l'écart avec C-55-1-1 est tracé en issue.

## Critères d'acceptation

1. **AC-1 — Lien et rechargement.** Une URL d'un bilan groupé (niveau 2, un groupe ouvert à la main, sous-totaux en pied)
   ouverte dans un nouveau contexte qui reprend les cookies de la session et un `localStorage` **vide** affiche le bilan dans
   cette présentation, contrôles remplis, sans clic ; de même au rechargement dans un contexte dont la mémoire porte une
   **autre** vue.
2. **AC-2 — L'URL prime sur la mémoire.** Avec une autre vue en mémoire pour ce rapport, l'URL l'emporte ; la mémoire prend
   ensuite la vue de l'URL (vue `flat` comprise) — ouvrir un lien remplace la présentation retenue du destinataire.
3. **AC-3 — La mémoire du navigateur complète l'URL.** `?tab=balance-sheet&fy=E` sans présentation, bilan **non gardé par la
   page** : la vue du `localStorage` s'applique, **1 requête** part avec elle, et l'URL est complétée à la réponse (**1
   `goto`** qui remplace) ; bilan gardé par la page avec cette vue : 0 requête, 1 `goto`.
4. **AC-4 — Retour vers une autre présentation.** Bilan au niveau 2 (URL remplacée), clic sur l'onglet *Balance*, clic sur
   le Bilan, niveau 3 (URL remplacée), puis *Retour* deux fois : le Bilan **au niveau 2**, contrôles au niveau 2 (une
   redemande, affichage gardé) ; sans la 55-1b, les contrôles restent au niveau 3 — le test rougit.
5. **AC-5 — Règle de la 46-2 intacte.** Scénario X1 (niveau changé sur le Bilan, *Générer* sur le Compte de résultat, clic
   sur le Bilan) et **le même scénario par *Retour*** au lieu du clic : le Bilan est redemandé dans la vue des contrôles **à
   la fin** de l'autre génération, qui n'est pas interrompue ; l'URL porte ensuite la clé affichée. Rejoués dans le Vitest
   de la page avec un `page.url` qui suit `goto`.
6. **AC-6 — Aucune requête d'une écriture.** Contrôle changé **avec** rapport affiché : URL remplacée (1 `goto`), **une**
   requête (celle de la 46-2), aucune seconde requête née de la relecture de l'URL ; **sans** rapport affiché : 0 `goto`,
   0 requête.
7. **AC-7 — Écho.** Un groupe ouvert à la main puis disparu du plan : la réponse le retire des listes ; l'URL de l'entrée
   courante (remplacée) et la mémoire le retirent aussi ; *Retour* puis *Suivant* vers cette entrée : 0 requête (Vitest).
8. **AC-8 — Valeurs invalides.** `level=0`, `level=21`, `level=03`, `level=3.0`, `level=+3`, `open=1,a`, `open=1,,2`,
   `open=99999999999999999999`, une liste de 501 identifiants, `closed` mal formé, `view=xyz` : chacun ignoré, URL
   normalisée, aucune valeur invalide envoyée à l'API ; un paramètre invalide seul, et `open=5&closed=5` seul, laissent la
   vue mémorisée (`anyValid` faux).
9. **AC-9bis — Vue à plat.** Lien `view=flat` sur un bilan dont la mémoire porte le niveau 2 : à plat, puis « hiérarchie
   montrée » rend le niveau 2 ; une réponse à plat n'écrit que `view=flat` (0 `goto` de plus si l'URL l'a déjà).
9. **AC-9ter — Retour pendant une redemande.** Bilan au niveau 2, niveau 3 (redemande en vol), *Retour* vers l'entrée du
   niveau 2 : le Bilan est redemandé au niveau 2, contrôles au niveau 2, l'écran ne montre jamais le niveau 3 sous des
   contrôles au niveau 2 ; *Retour* vers une URL égale à la clé en vol : 0 requête.
9. **AC-9 — Export.** Les boutons PDF et CSV reprennent toujours la dernière réponse affichée (46-2, AC-6), inchangé.
10. **AC-10 — Manuel.** Paragraphe de la présentation de la PR 46 (`:4240-4250` sur la PR 46) : l'adresse porte la
    présentation, qui prime sur celle que retient le navigateur, et ouvrir un lien remplace la présentation retenue ;
    paragraphe « Naviguer dans les rapports » de la 55-1a : la présentation s'ajoute à « ce qu'elle retient ». `.tex` seul.

## Tests à écrire, et la preuve du rouge

- **Vitest** `reports-grouping.test.ts` (PR 46) : `groupingFromParams`, chaque valeur de l'AC-8, `anyValid`.
- **Vitest** `reports-url.test.ts` (55-1a) : écriture des cinq paramètres, ordre et unicité des listes, virgule brute et
  encodée lues de même, aller-retour canonique.
- **Vitest de la page** (`reports-page-url.test.ts`, helper de la 55-1a) : AC-2 à AC-8, avec les compteurs de requêtes et
  de `goto` de chaque cas ; flèches du clavier comme le clic.
- **E2E** `reports-navigation.spec.ts`, bloc 55-1b : AC-1 (nouveau contexte à `localStorage` vide, et rechargement avec une
  autre vue mémorisée) et AC-4, sur un plan groupé (helper `tests/e2e/helpers/reports-groupes.ts` de la PR 46).
- **Preuve du rouge** (journal `kesh-gate-logs/s55-1b/`) et **mutations**, chacune tuée par un test nommé : mémoire
  appliquée malgré l'URL (AC-2) ; vue `flat` d'un lien non mémorisée (AC-2) ; écho non reporté dans l'URL (AC-7) ; URL
  écrite par un contrôle réappliquée (AC-6) ; redemande immédiate au lieu de `resyncActiveGroupedTab` (AC-5 par *Retour*) ;
  clé en vol sans présentation (AC-9ter) ; `view=flat` qui remet les autres champs aux défauts (AC-9bis) ; contrôle sans
  rapport qui écrit l'URL (AC-6).

## Recette visuelle

Captures (`kesh-gate-logs/s55-1b/`) : bilan groupé avant le lien ; après *Retour* ; lien collé dans un nouveau contexte.

## Tâches

- [ ] **T0** — Base `epic/55-navigation` avec la 55-1a.
- [ ] **T1** — Tests rouges.
- [ ] **T2** — `groupingFromParams` (`reports-grouping.ts`) ; paramètres de présentation dans `reports-url.ts` ; clés ;
      écriture aux gestes et à la réponse ; lecture.
- [ ] **T3** — Manuel, E2E, captures ; gates.

## Gates

`cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build` ; E2E des Rapports ;
suite complète selon le § *Gates* du fichier d'epic (Q6 non tranchée ou refusée : E2E complet au dernier commit de code de
la story).

## Note de version

*Rapports* : l'adresse d'un bilan, d'un compte de résultat ou d'une balance porte aussi sa présentation (hiérarchie,
niveau, sous-totaux, groupes ouverts ou fermés à la main) ; elle prime sur celle que retient le navigateur, et ouvrir un
lien remplace la présentation retenue. (refs #À-VENIR)

## Dev Notes

### Choix et écarts

- **C-55-1b-1** — Tout ou rien entre l'URL et la mémoire : mêler un niveau de l'URL et des listes de la mémoire donnerait
  une vue que personne n'a affichée (les listes dépendent du niveau).
- **C-55-1b-2** — Les gestes écrivent la vue **des contrôles**, la réponse la vue **affichée** : c'est ce que la 46-2 fera
  afficher (redemande à la fin d'une génération en vol), et l'écho corrige l'URL à la réponse.
- **C-55-1b-3** — Longueur : au plus 500 + 500 identifiants (≈ 6 Ko d'URL), sous les limites des navigateurs et du serveur.
  Un lien ouvert dans une autre société porte des identifiants inconnus d'elle : l'API les ignore et l'écho les retire
  (46-1), sans erreur.

### References

- Fiche 55-1a ; fiche 46-2, AC-3, AC-5, AC-6, AC-10.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### Issues à ouvrir

(Points de revue non corrigés : sévérité, site, texte proposé ; source de la story de solde 55-4.)

- (LOW, `bug` P3, préexistant) Une redemande de présentation (46-2, `changeGrouping`) lit le formulaire : un champ
  modifié sans *Générer* passe dans la redemande (Epic 55, story 55-1b, relevé de validation).

### File List

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
