# Story 55-1 : l'état des Rapports dans l'URL — Retour, Suivant, lien partagé

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md` — D1 à D5, Q2. Issue : #À-VENIR (`bug`, P3, point b).
Release : la prochaine planifiée après la 0.20.0 (numéro fixé par le mainteneur). **Frontend seulement, aucune migration.**
**Base : la PR de l'Epic 46** (`pr/46-rapports-groupes`, `511e61d54` au kickoff), reprise dans `epic/55-navigation` avant T1.

## Story

En tant que comptable qui lit ses rapports,
je veux que *Retour* et *Suivant* du navigateur me ramènent au rapport que je regardais, avec sa période et ses filtres, et
qu'un lien copié rouvre la même vue,
afin de passer du bilan au grand livre d'un compte et d'en revenir sans perdre ma place ni tout régénérer.

## Modules touchés (règle de splitting préventif)

Quatre : `frontend/src/lib/features/reports` (`reports-url.ts` neuf, `reports.api.ts`, `ReportSelector.svelte` lu),
`frontend/src/routes/(app)/reports` (page), `frontend/tests/e2e`, `docs/manual/fr/user-manual.tex`. `kesh-i18n` seulement
si un texte apparaît (aucun n'est prévu). Aucun code Rust.

## Ce qui se lit au code

Relevé à `origin/main` `1e23edb55` et sur `pr/46-rapports-groupes` `511e61d54` (`git show`). Les numéros de ligne de la page
sont ceux de `origin/main` ; ceux de la PR 46 se relèvent au T0 (+287 lignes).

- `frontend/src/routes/(app)/reports/+page.svelte` (893 lignes à `main`) :
  - `activeTab` (`:85`) est un `$state` ; `selectTab` (`:608-615`) écrit `goto('?tab=' + next, { replaceState: true,
    keepFocus: true, noScroll: true })` — Story 21-7, D-7c : « pas d'empilement » ;
  - lecture de `?tab=` **une seule fois**, dans `onMount` (`:546-551`) ;
  - arrivée par un lien du grand livre : un `$effect` (`:567-586`) lit `tab`, `from`, `to`, `accountId`, **n'agit que si
    `tab=general-ledger`** et génère ; il est gardé par une clé (`lastLedgerDeepLink`) ;
  - exercice et période (`:82-84`), grand livre (`:109-112`), compte analytique et mode (`:91-92`) : `$state` jamais écrits
    dans l'URL ;
  - effet P17 (`:151-173`) : quand `selectedFiscalYearId` change, il vide la période et **tous** les rapports affichés et
    périme la génération en vol (`genSeq`) ; effet du compte analytique (`:182-195`) : quand projet ou mode change, il vide
    les deux rapports par projet et périme la génération en vol ;
  - la requête d'un rapport par projet en mode exercice lit **la période des rapports classiques** (`projectQuery`,
    `:222-232`, `periodStart`/`periodEnd`) ;
  - garde de course `genSeq` (`:149`) ; sur la PR 46, `invalidateInFlight()` est le seul endroit qui l'incrémente.
- `frontend/src/lib/features/reports/reports.api.ts:574-582` : `ledgerHref(accountId, from, to)` →
  `/reports?tab=general-ledger&accountId=…&from=…&to=…`, rendu en `<a href>` (SvelteKit empile l'entrée). Employé aussi par
  `homepage/ExpensePieCard.svelte:64` et `fiscal-years/RevaluationPanel.svelte:357` ; `tests/e2e/homepage-charts.spec.ts:129,
  142,233` vérifie l'URL par une expression régulière **dans cet ordre de paramètres**.
- `frontend/src/routes/(app)/reports/+page.ts` : `load` ne lit pas l'URL (exercices, société) ; il ne se rejoue donc pas
  quand seuls les paramètres de requête changent. **À garder ainsi** (sinon chaque navigation rechargerait les exercices).
- Sur la PR 46 : `reports-grouping.ts` (`GroupingView { shown, detailLevel, subtotalPosition, open, closed }`,
  `sanitizeGroupingView`, `isDetailLevel`, `MAX_DETAIL_LEVEL = 20`, `MAX_MANUAL_GROUPS = 500`, `readGroupingView`,
  `writeGroupingView`, `groupingParamsOf`, `withEcho`) ; la page lit la vue mémorisée avant la première requête et la
  réécrit après chaque réponse (`acceptGrouped`, qui reprend l'écho de l'API : les groupes disparus du plan sortent des
  listes). Elle remet un rapport groupé d'accord avec ses contrôles au retour sur l'onglet (`resyncActiveGroupedTab`,
  appelée par `selectTab`) ou, si une autre génération est en vol, **à la fin de celle-ci** (`afterGeneration`) — règle
  revue de la 46-2 (P2 M1), décrite au manuel de la PR 46 (§ *Bilan*, « si vous revenez sur un rapport pendant la
  génération d'un autre, dès la fin de celle-ci ») et éprouvée par `reports-page-groupes.test.ts` (PR 46, 681 lignes).
- SvelteKit 2 (relu à `/home/beat/claude/kesh/frontend/node_modules/@sveltejs/kit/src/runtime/client/client.js`) : un
  `goto` vers la même route met à jour `page.url` sans remonter le composant ni rejouer un `load` qui ne lit pas l'URL ;
  Retour et Suivant (`popstate`) de même ; au `popstate`, le focus est remis au corps de la page.
- Liens entrants : `/reports?tab=aged-receivables` (`invoices/due-dates/+page.svelte:401`, `invoices/reminders/+page.svelte:277`),
  `/reports` (`settings/opening-balances/+page.svelte:691`). E2E qui ouvrent `/reports?…` : `dunning-roundtrip.spec.ts:115`,
  `open-items.spec.ts:357`, `reports.spec.ts:319,423,432,461`, `homepage-charts.spec.ts`.
- **Aucun filtre de devise** n'existe dans les Rapports (`grep -n "currency\|devise"` sur la page ne rend, à `main` et sur
  les trois branches de l'Epic 46, qu'un commentaire, `// fx-filtre-comptes : exempté — rapports …`) : rien à porter dans
  l'URL à ce titre. Si un filtre existe à la base de départ (T0), il entre dans
  le tableau des paramètres, sinon le Change Log le redit.

**Le défaut, par lecture** : sur le Bilan, un clic sur `1020` empile `/reports?tab=general-ledger&accountId=…` ; l'effet
génère le grand livre. *Retour* ramène l'URL à `?tab=balance-sheet` (ou `/reports`) sans remonter le composant ; l'effet
sort tôt (`tab ≠ general-ledger`), `activeTab` reste `general-ledger`. L'URL a changé, l'écran non ; un second *Retour*
quitte la page. À confirmer par l'E2E écrit rouge d'abord (T1).

## Le modèle (D1 à D5)

**L'URL décrit, pour l'onglet affiché, le dernier rapport demandé sur cet onglet.** Deux fonctions pures
(`reports-url.ts`) lisent l'URL en un état et écrivent l'état en URL ; la page applique l'état lu **à chaque changement de
`page.url`** (chargement, Retour, Suivant, lien, `goto` de la page elle-même), et n'écrit l'URL qu'aux gestes ci-dessous.
**Le changement d'onglet ne génère rien de neuf** (comme aujourd'hui) : il montre le rapport que l'onglet a déjà, ou son
invite.

### Paramètres

| Paramètre | Onglets | Valeur valide |
|---|---|---|
| `tab` | tous | un des neuf identifiants de `tabs` |
| `fy` | bilan, compte de résultat, balance, journaux, TVA ; rapports par projet en mode `fiscal_year` | identifiant d'un exercice de `data.fiscalYears` |
| `from`, `to` | mêmes onglets que `fy` (période facultative, partagée comme aujourd'hui) ; grand livre (requis) | date ISO `AAAA-MM-JJ` réelle (`2026-02-30` refusée) |
| `accountId` | grand livre | entier > 0 (le compte peut être archivé) |
| `includeZero` | grand livre | `1` |
| `projectId` | rapports par projet | entier > 0 |
| `mode` | rapports par projet | `cumulative` (absent : `fiscal_year`) |
| `view` | bilan, compte de résultat, balance (Epic 46) | `grouped`, `flat` |
| `level` | idem | `1`…`20`, `all` (`isDetailLevel`) |
| `subtotals` | idem | `header`, `footer` |
| `open`, `closed` | idem | entiers > 0 séparés par des virgules, ≤ 500 chacun, sans identifiant commun aux deux |

- **Écriture** : tout ce qui décrit la requête **s'écrit**, valeur par défaut comprise — `fy` toujours (sur les onglets qui
  le prennent), la vue de regroupement toujours (`view`, `level`, `subtotals` ; `open`, `closed` s'ils sont non vides) ;
  seuls `includeZero` faux, `mode=fiscal_year` et une période vide ne s'écrivent pas.
- **Ordre canonique** : `tab`, `fy`, `accountId`, `from`, `to`, `includeZero`, `projectId`, `mode`, `view`, `level`,
  `subtotals`, `open`, `closed`. Pour le grand livre, il rend **exactement** la sortie d'aujourd'hui de `ledgerHref`
  (`tab`, `accountId`, `from`, `to`) : `ledgerHref` est réécrite sur la fonction d'écriture (DRY), sortie inchangée.
- Un paramètre qui ne sert pas l'onglet est ignoré et retiré à la normalisation.
- **Complète** (le rapport peut être demandé) : classiques — `fy` ; grand livre — `from` et `to` ; projets — `projectId`,
  plus `fy` en mode `fiscal_year` ; balance âgée — toujours ; le regroupement n'est jamais requis (absent : D4).
- **Sans `tab` valide** (`/reports`, `/reports?fy=3`, `?tab=xyz`) : l'écran d'arrivée d'aujourd'hui — onglet *Bilan*,
  invite « Sélectionnez un exercice et cliquez sur Générer », aucun rapport demandé ; les autres paramètres sont ignorés et
  l'URL normalisée en `/reports`. Les rapports déjà produits restent en mémoire (un clic sur un onglet les montre). C'est
  aussi ce que fait le lien du menu *Rapports* cliqué depuis la page.

### Gestes (D2)

| Geste | URL | Génération |
|---|---|---|
| Clic sur un onglet | **`pushState`** : l'onglet et **le dernier rapport demandé sur cet onglet** (sa clé), ou `?tab=<id>` seul s'il n'en a pas | **aucune** : le rapport en mémoire s'affiche, sinon l'invite (comportement d'aujourd'hui) ; la règle de la 46-2 (`resyncActiveGroupedTab`) reste appelée telle quelle |
| Flèches, Origine, Fin du clavier sur les onglets | **`replaceState`** (aucune entrée par flèche) | aucune |
| Clic sur un lien de compte (`<a href={ledgerHref(…)}>`) | **empile** (navigation SvelteKit, inchangée) | oui (URL complète) |
| *Générer* | **`replaceState`** avec la requête partie | oui (comme aujourd'hui) |
| Exercice, date, compte, compte analytique, mode, « comptes sans mouvement » modifiés | **rien** : l'URL reste celle du dernier rapport demandé, jusqu'à *Générer* | non (comme aujourd'hui) |
| Contrôles de regroupement et accordéons (Epic 46) | **`replaceState`** | oui (la 46-2 redemande déjà) |
| Retour, Suivant, chargement, lien collé | — (lecture) | si l'URL est complète : mémoire ou requête |

`goto` est appelé avec `keepFocus: true, noScroll: true` ; aucune écriture si l'URL canonique est déjà la même.

### Appliquer une URL (D3)

Une seule fonction de la page, `applyUrl(state)`, appelée par un `$effect` sur `page.url` (sous `untrack`), qui remplace
l'`onMount` (`:546`) et l'effet du grand livre (`:567`) :

1. **Idempotence** : si la clé canonique lue est celle de la dernière application, rien.
2. **Onglet** : `activeTab` prend la valeur lue, par le même chemin interne que `selectTab` **sans écrire l'URL** (la
   règle de la 46-2 `resyncActiveGroupedTab` s'applique donc à l'identique).
3. **Champs** : seuls les champs de l'onglet lu prennent les valeurs de l'URL ; ceux des autres onglets ne bougent pas.
   Les gardes P17 et du compte analytique sont **désarmées pour cette application** : `lastFyId` et `lastProjectSel`
   reçoivent les valeurs restaurées **avant** les champs, si bien que ni la période lue ni la génération lancée ne sont
   effacées. Si l'exercice change, les rapports des autres onglets (produits pour l'autre exercice) sont vidés comme par
   P17, mais pas la période restaurée.
4. **Rapport** : URL incomplète → toute génération en vol est périmée (`invalidateInFlight`), l'invite s'affiche, champs
   remplis de ce que l'URL porte. URL complète → si le rapport en mémoire de l'onglet porte **cette clé**, il s'affiche
   sans requête ; sinon `generate()`. La clé d'un onglet est posée **au départ** de la requête (clé « en vol ») et
   confirmée à la réponse : la relecture de l'URL écrite par *Générer* ne relance rien. Une réponse périmée (`genSeq`) ne
   s'affiche jamais.

### Valeurs invalides (D5)

Chaque paramètre est jugé seul ; une valeur invalide est ignorée (le champ garde sa valeur d'aujourd'hui : premier
exercice, période vide, aucun compte…), et l'URL est **normalisée** en `replaceState` (aucune entrée d'historique, aucun
message). Une URL rendue incomplète par la normalisation n'est pas demandée (le `fy` invalide d'un bilan donne l'invite,
non le bilan du premier exercice). Un exercice inconnu (supprimé, autre société) est invalide. Période hors exercice,
`from > to`, compte inconnu de la société : jugés **par l'API**, avec ses messages d'aujourd'hui.

### Regroupement de l'Epic 46 (D4)

- **Tout ou rien** : si l'URL porte au moins un paramètre de regroupement, la vue est `sanitizeGroupingView` de ces seuls
  paramètres (les absents prennent la vue **par défaut** de la 46-2, non la mémoire) ; si elle n'en porte aucun, la vue
  mémorisée (`readGroupingView`) s'applique. `view=flat` garde les autres paramètres s'ils sont donnés (la 46-2 garde
  niveau et ouvertures pendant que la hiérarchie est masquée).
- À l'affichage, l'URL est complétée ou corrigée en `replaceState` avec la vue **affichée**, écho de l'API compris
  (`withEcho` : les groupes disparus sortent de `open`/`closed`), et la clé de l'onglet suit : Retour vers cette entrée ne
  redemande rien, et un lien copié ne porte que des groupes existants.
- La mémoire du navigateur retient la dernière vue affichée, celle d'un lien comprise (`acceptGrouped`, inchangé). L'export
  reprend les paramètres de la dernière réponse affichée (46-2, AC-6), inchangé.

## Critères d'acceptation

1. **AC-1 — Retour après un lien de compte.** Bilan de l'exercice E, période P, affiché ; clic sur un numéro de compte →
   grand livre de ce compte sur P ; *Retour* → **le Bilan de E sur P**, sans clic sur *Générer*, même vue de regroupement,
   **sans nouvelle requête** `GET /api/v1/reports/balance-sheet` ; *Suivant* → le grand livre, sans nouvelle requête. Même
   chose depuis la Balance et depuis le Compte de résultat groupé.
2. **AC-2 — Retour entre onglets.** Bilan généré (E) ; clic sur l'onglet *Compte de résultat* (jamais généré) → une entrée
   d'historique de plus, URL `?tab=income-statement`, invite affichée, **aucune requête** ; *Générer* → compte de résultat,
   URL `?tab=income-statement&fy=E…` (remplacée) ; *Retour* → le Bilan, onglet *Bilan* `aria-selected`, sans requête ;
   *Suivant* → le compte de résultat, sans requête.
3. **AC-3 — *Générer* remplace.** Bilan de E affiché ; période changée, *Générer* → l'URL porte la nouvelle période et
   `history.length` est inchangé.
4. **AC-4 — Lien partagé et rechargement.** Une URL copiée — classique (onglet, exercice, période, regroupement), grand
   livre (compte, dates, comptes sans mouvement), rapport par projet (compte analytique, mode, exercice, période) — ouverte
   dans un **nouveau contexte de navigateur** (stockage vide) ou rechargée affiche **le même rapport, contrôles remplis**,
   sans clic. Le regroupement de l'URL prime sur une vue mémorisée différente (test dédié qui pose d'abord une autre vue en
   mémoire).
5. **AC-5 — Valeurs invalides.** `tab=xyz`, `fy=999999`, `fy=abc`, `from=2026-02-30`, `level=0`, `level=21`, `open=1,a`,
   `open=5&closed=5`, `accountId=-3`, un paramètre étranger à l'onglet : chacun ignoré, URL normalisée sans entrée
   d'historique, aucune erreur affichée, aucune valeur invalide envoyée à l'API ; `?tab=balance-sheet&fy=abc` donne
   l'invite. Une période hors exercice garde le message de l'API d'aujourd'hui.
6. **AC-6 — Compatibilité.** `ledgerHref(a, f, t)` rend la même chaîne qu'avant (trois cas) ;
   `homepage-charts.spec.ts` vert sans modification ; `/reports?tab=aged-receivables` (échéancier, rappels) affiche la
   balance âgée, **désormais sans clic** (URL complète) ; les Vitest de la PR 46 (`reports-page-groupes.test.ts`, dont X1 et
   le `describe.each`) et `reports-page.test.ts` passent **sans modification** — toute adaptation indispensable est nommée
   au Change Log, sans perte d'assertion, et soumise à la revue.
7. **AC-7 — Pas de requête de trop, pas de boucle.** *Générer* : **une** requête ; clic sur un onglet : **zéro** ;
   navigation vers une URL dont le rapport est en mémoire : **zéro** ; arrivée sur une URL complète : **une** ; aucun appel
   de `goto` en boucle (Vitest : appels de `goto` et des fonctions d'API comptés).
8. **AC-8 — Course.** Retour pendant une génération en vol : la réponse périmée ne s'affiche pas, l'écran montre le
   rapport de l'URL courante (mémoire ou nouvelle requête) ; Retour vers un rapport groupé pendant la génération d'un autre
   onglet suit la règle de la 46-2 (redemande à la fin de l'autre génération si la vue diffère).
9. **AC-9 — Restauration sans effacement.** Retour vers une URL d'un autre exercice, et vers une URL d'un autre compte
   analytique : la période et le compte analytique restaurés ne sont pas effacés par P17 ni par l'effet du compte
   analytique, et la génération lancée n'est pas périmée par eux (deux tests).
10. **AC-10 — Accessibilité.** Après Retour, l'onglet de l'URL est `aria-selected` et `tabindex=0` ; aucun saut de
    défilement provoqué par la page (`noScroll` sur ses `goto`) ; le focus suit SvelteKit (corps de la page au `popstate`) ;
    test axe de `reports.spec.ts` à zéro violation.
11. **AC-11 — Manuel.** `docs/manual/fr/user-manual.tex` (lignes à `1e23edb55`, à relever sur la base PR 46 au T0) :
    § *Rapports comptables* (`:4101`) et § *Visualisation et exports* (`:4368`) — un paragraphe « Naviguer dans les
    rapports » : Retour et Suivant, adresse à copier, ce qu'elle retient (onglet, exercice, période, compte, compte
    analytique, présentation), ce qu'elle ne retient pas (un champ modifié sans *Générer*) ; § *Grand livre*, l'item « Accès
    direct depuis le bilan et la balance » (`:4220`) : Retour ramène au rapport ; sur la PR 46, le paragraphe de la
    présentation (« Le navigateur retient votre présentation… », manuel de la PR 46 `:4240-4250`) : une adresse copiée porte
    la présentation, qui prime sur celle que retient le navigateur. `.tex` seul, PDF aplati contrôlé avant.
12. **AC-12 — Textes.** Aucun texte visible n'est ajouté (prévu) ; s'il en faut un, quatre locales dans le bloc
    `reports-nav-*`, `i18n-keys` (`sitesTotal` recompté par la garde) et `i18n-libelle-en-dur` verts.

## Tests à écrire, et la preuve du rouge

- **Vitest `frontend/src/lib/features/reports/reports-url.test.ts`** (neuf) : lecture et écriture de chaque paramètre ;
  aller-retour `build(parse(u)) === u` sur les URL canoniques ; chaque valeur invalide de l'AC-5 ; ordre canonique ;
  `ledgerHref` égal à l'ancienne sortie (trois cas) ; regroupement tout ou rien, > 500 identifiants ignorés.
- **Vitest de la page** (`frontend/src/routes/(app)/reports/reports-page-url.test.ts`, neuf) avec un **`page` réactif**
  (patron `frontend/src/routes/(app)/journal-entries/[id]/page-state.test-helper.svelte.ts`) et un `goto` simulé **qui écrit
  `page.url`** (sinon la relecture par l'effet n'est jamais exercée) : AC-2, AC-3 (option `replaceState`), AC-7
  (compteurs), AC-8, AC-9, D4 (URL prime ; mémoire complète l'URL ; écho reporté dans l'URL et la clé).
- **E2E Playwright `frontend/tests/e2e/reports-navigation.spec.ts`** (neuf) : AC-1 (`page.goBack()`, `page.goForward()`,
  requêtes comptées par `page.on('request')`), AC-2, AC-3, AC-4 (**`browser.newContext()`**, donc `localStorage` vide, et
  `page.reload()`), AC-5 (deux valeurs invalides).
- **Preuve du rouge** : l'E2E AC-1 est lancé **avant** le correctif (journal `kesh-gate-logs/s55-1/e2e-rouge-avant.log`) et
  rougit sur « Bilan affiché après Retour » ; les Vitest neufs de la page rougissent sans le correctif (journal). Mutations
  nommées au Change Log, chacune tuée par un test : dans la page, `pushState` au lieu de `replaceState` pour *Générer* ;
  retrait du désarmement de P17 ; dans `reports-url.ts`, retrait de la validation de `fy`.

## Recette visuelle

Captures (thème clair, base de démonstration, `kesh-gate-logs/s55-1/`) : Bilan avant le lien ; grand livre après le lien ;
Bilan après *Retour* (barre d'adresse visible) ; lien collé dans un nouveau contexte ; URL normalisée après une valeur
invalide. Jointes au Dev Agent Record et au prompt de la revue de code.

## Tâches

- [ ] **T0** — Reprendre la PR 46 dans `epic/55-navigation` (`git merge`, conflits résolus en gardant les deux côtés) ;
      relever les lignes de la page et du manuel à cette base ; vérifier qu'aucun filtre de devise n'existe ; sinon arrêt
      et Change Log.
- [ ] **T1** — E2E AC-1 rouge (journal).
- [ ] **T2** — `reports-url.ts` (lecture, écriture, canonique) et ses Vitest ; `ledgerHref` réécrite dessus.
- [ ] **T3** — Page : `applyUrl` et son `$effect` (remplacent `onMount` `:546` et l'effet `:567`) ; `selectTab` en
      `pushState` au clic, `replaceState` au clavier ; *Générer* et regroupement en `replaceState` ; clés par onglet
      (en vol, confirmée) ; désarmement de P17 et de l'effet du compte analytique ; écho reporté dans l'URL.
- [ ] **T4** — Suites existantes : Vitest `reports-page.test.ts`, `reports-page-groupes.test.ts` ; E2E `reports.spec.ts`,
      `reports-groupes*.spec.ts`, `reports-currency.spec.ts`, `reports-export-pdf.spec.ts`, `dunning-roundtrip.spec.ts`,
      `open-items.spec.ts`, `homepage-charts.spec.ts`, `invoice-write-off.spec.ts`, `opening-balances.spec.ts` ;
      adaptations nommées (AC-6).
- [ ] **T5** — Manuel, captures.
- [ ] **T6** — Gates (ci-dessous).

## Gates

Règle de l'epic (fichier d'epic, § *Gates*) : `cd frontend && npm run check && npm run lint-i18n-ownership && npm run
test:unit && npm run build` ; E2E : la spec neuve et les specs de T4 ; suite complète en fin d'epic. Aucun gate backend
(aucun fichier Rust) — `cargo fmt --check` seulement si la reprise de la PR 46 apporte du Rust non formaté.

## Note de version

*Rapports* : *Retour* et *Suivant* du navigateur ramènent au rapport précédent, avec son exercice, sa période, son compte
et sa présentation ; l'adresse d'une vue se copie et rouvre la même vue, sans cliquer sur *Générer*. (refs #À-VENIR)

## Dev Notes

### Pièges

- **Effets P17 et du compte analytique** : voir *Appliquer une URL*, point 3.
- **`goto` relit sa propre écriture** : l'idempotence (point 1) et la clé en vol (point 4) évitent la double requête.
- **`load` ne doit pas lire `url`** : sinon SvelteKit le rejoue à chaque changement de paramètre.
- **Liste des comptes** : un `accountId` absent de la liste n'est pas invalide côté client (compte créé depuis) — l'API juge.
- La PR 46 garde l'affichage pendant une redemande (`refreshing`) et périme par `invalidateInFlight()` : `applyUrl` passe
  par elle, jamais par `genSeq++` direct.

### Choix et écarts

- **C-55-1-1** — URL = dernier rapport demandé, non l'état du formulaire. *Écarté* : écrire chaque saisie en
  `replaceState` — un lien copié ou un rechargement montrerait un rapport que l'écran n'affichait pas.
- **C-55-1-2** — Un rapport en mémoire par onglet (sa réponse et sa clé) : *Retour* plus loin qu'une vue de l'onglet
  redemande à l'API (une trace d'audit de plus, assumée).
- **C-55-1-3** — `goto` plutôt que le routage superficiel (`pushState`/`replaceState` de `$app/navigation`) : `page.url` et
  la navigation par `<a>` restent la seule source.
- **C-55-1-4** — Le changement d'onglet ne génère pas (validation P1, V1-1) : générer à chaque onglet aurait contredit la
  règle revue de la 46-2 (redemande à la fin de la génération en vol) et ajouté une requête et une trace par clic ;
  l'onglet montre le rapport qu'il a, comme aujourd'hui. Seules une URL arrivée de l'extérieur (lien, rechargement,
  Retour/Suivant vers un rapport absent de la mémoire) déclenchent une requête.
- **C-55-1-5** — Clavier en `replaceState` : une entrée par flèche rendrait *Retour* inutilisable.

### References

- `CLAUDE.md` § *Test Locally First* (E2E), § *Recette visuelle*, § *Travailler à plusieurs développeurs*.
- Fiche 46-2 (`pr/46-rapports-groupes:_bmad-output/implementation-artifacts/46-2-groupes-ecrans.md`), AC-3, AC-5, AC-6,
  AC-10.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### Issues à ouvrir

(Points de revue non corrigés : sévérité, site, texte proposé ; source de la story de solde 55-4.)

### File List

## Change Log

- 2026-11-25 — Fiche écrite au kickoff de l'Epic 55 (`drafted`).
- 2026-11-25 — Validation P1 (Opus) : 0/1/10/8 ; remédiée : le changement d'onglet ne génère plus (V1-1, C-55-1-4, D3 du
  fichier d'epic amendée, règle de la 46-2 gardée) ; période des rapports par projet dans l'URL (V1-2) ; application d'une
  URL en quatre points — champs, désarmement de P17 et de l'effet du compte analytique, clé en vol, invalidation (V1-3,
  V1-4) ; écriture des valeurs qui décrivent la requête (V1-5) ; regroupement tout ou rien, écho reporté (V1-6, V1-7) ;
  dernier rapport de l'onglet au clic (V1-8) ; `page` réactif dans les Vitest (V1-9) ; manuel de la PR 46 (V1-10) ;
  clavier en `replaceState` (V1-11) ; LOW V1-12 à V1-19.
