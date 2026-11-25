# Story 55-1a : l'état des Rapports dans l'URL — Retour, Suivant, lien partagé

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md` — D1 à D3, D5, Q2 (D4 : 55-1b). Issue : #À-VENIR (`bug`, P3, point b).
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

*Découpée à la validation P3 (signal de recyclage : la P3 a remonté un HIGH, dont une part née des remédiations de la
P2) : la 55-1a porte l'URL des rapports **sans** la présentation de l'Epic 46 ; la 55-1b y ajoute la présentation (D4).
Le modèle est en même temps **simplifié** : la page n'applique jamais l'URL qu'elle écrit elle-même.*

**L'URL décrit, pour l'onglet affiché, le dernier rapport demandé sur cet onglet.** Deux fonctions pures
(`reports-url.ts`) lisent l'URL en un état et écrivent l'état en URL.

**Règle fondatrice — deux sens, jamais mêlés** :

- **La page écrit l'URL** à ses propres gestes (tableau des gestes), **après** avoir changé son état comme aujourd'hui ;
  avant chaque `goto`, elle enregistre la clé canonique écrite comme « déjà appliquée ».
- **La page applique l'URL** (`applyUrl`) seulement quand `page.url` porte une clé **qu'elle n'a pas écrite** : chargement,
  Retour, Suivant, lien (de compte ou d'ailleurs). Une URL écrite par la page est reconnue par sa clé et **ignorée** par
  l'effet : aucun champ, aucune requête — le geste a déjà tout fait.

### Paramètres

| Paramètre | Onglets | Valeur valide |
|---|---|---|
| `tab` | tous | un des neuf identifiants de `tabs` |
| `fy` | bilan, compte de résultat, balance, journaux, TVA | identifiant d'un exercice de `data.fiscalYears` |
| `from`, `to` | mêmes onglets que `fy` (période facultative, partagée comme aujourd'hui) ; grand livre (requis) | `estDateStockable` (`frontend/src/lib/shared/utils/date-filtre.ts:19`, Story 33-4 : date réelle, années 1000 à 9999, comme l'API), aucune validation neuve |
| `accountId` | grand livre | entier `^[1-9][0-9]*$` ≤ `Number.MAX_SAFE_INTEGER` (le compte peut être archivé) |
| `includeZero` | grand livre | `1` |

- **Rapports par compte analytique** (dépenses, rendement) : **hors de la 55-1a** — l'URL n'en porte que `?tab=<id>`, comme
  aujourd'hui, et ne les génère pas (validation P4, V4-1 : restaurer projet et mode depuis l'URL demandait de désarmer la
  garde qui empêche un export d'un autre projet que celui de l'écran). Issue `enhancement` à ouvrir.
- **Liste des onglets** : `TabId` et la liste des neuf identifiants passent de la page à `reports-url.ts` (DRY), la page
  les importe.
- **Écriture** : tout ce qui décrit la requête s'écrit (`fy` toujours, sur les onglets qui le prennent) ; seuls
  `includeZero` faux et une période vide ne s'écrivent pas.
- **Ordre canonique** : `tab`, `fy`, `accountId`, `from`, `to`, `includeZero` ; forme de
  `URLSearchParams.toString()`. Pour le grand livre, il rend **exactement** la sortie d'aujourd'hui de `ledgerHref`
  (`tab`, `accountId`, `from`, `to`) : `ledgerHref` est réécrite sur la fonction d'écriture (DRY), sortie inchangée ;
  `build(parse(u)) === u` pour toute URL canonique.
- Un paramètre qui ne sert pas l'onglet est ignoré et retiré à la normalisation. Les paramètres de présentation de
  l'Epic 46 ne sont pas lus par la 55-1a (55-1b).
- **Complète** (le rapport peut être demandé) : classiques — `fy` ; grand livre — `from` et `to` ; balance âgée —
  toujours (un *Suivant* vers `?tab=aged-receivables` la génère donc, comme le lien de l'échéancier) ; rapports par compte
  analytique — jamais.
- **Sans `tab` valide** (`/reports`, `/reports?fy=3`, `?tab=xyz`) : onglet *Bilan*, aucun rapport demandé, aucun champ
  modifié ; l'écran montre le bilan en mémoire s'il y en a un, sinon l'invite d'aujourd'hui ; l'URL est normalisée en
  `/reports`. Le lien du menu *Rapports* cliqué depuis la page fait de même : il **revient à l'onglet *Bilan*** (aujourd'hui
  il ne changeait rien) — changement voulu, dit à la note de version.

### Gestes (D2) — la page écrit

| Geste | Ce que fait la page (comme aujourd'hui) | URL écrite | Appels de `goto` |
|---|---|---|---|
| Clic sur un onglet | `selectTab` : onglet changé, `errorMsg` effacé, règle de la 46-2 (`resyncActiveGroupedTab`) ; **aucune requête** | **empile** : l'onglet et la clé du **rapport affiché** de cet onglet, ou `?tab=<id>` seul s'il n'en a pas ; les champs du formulaire ne changent pas | 1 |
| Flèches, Origine, Fin sur les onglets | idem | **remplace** | 1 |
| *Générer* | `generate()` (une génération à la fois, comme aujourd'hui) | **remplace**, avec la requête partie — écrit par le **gestionnaire** du bouton, après l'appel de `generate()`, jamais par `generate()` elle-même (qui sert aussi `changeGrouping`, `resyncActiveGroupedTab` et `applyUrl`) | 1 |
| Exercice, date, compte, « comptes sans mouvement », compte analytique, mode modifiés | rien de neuf (P17 et l'effet du compte analytique vident comme aujourd'hui) | **aucune** : l'URL reste celle du dernier rapport demandé, jusqu'à *Générer* | 0 |
| Contrôles de présentation et accordéons (Epic 46) | la 46-2 redemande, comme aujourd'hui | **aucune** dans la 55-1a (la présentation entre dans l'URL à la 55-1b) | 0 |
| Clic sur un lien de compte (`<a href={ledgerHref(…)}>`) | — | **empile** (navigation SvelteKit, inchangée) : URL **non écrite par la page**, donc appliquée | 0 |

- **Où l'URL s'écrit** : seulement dans les gestionnaires des gestes (`selectTab`, clavier, bouton *Générer*), une seule
  fois par geste ; aucune réponse n'écrit l'URL dans la 55-1a (la 55-1b ajoute l'écho). `selectTab` appelle `goto` **en
  dernier**, après `resyncActiveGroupedTab` (qui n'écrit pas) : aucun second `goto` dans la même tâche n'abandonne le
  premier (`client.js:1809`, `:1873-1883`).
- **Clé** : une seule variable, `lastHandledKey`, posée à **chaque** `goto` de la page **et** à **chaque** application
  d'une URL, jamais consommée ; « déjà la même » compare la clé canonique de l'URL lue à `lastHandledKey`.
- `goto` est appelé avec `keepFocus: true, noScroll: true`, toujours `void goto(…)` (sans `await` ni `.then` : les mocks
  `goto: vi.fn()` des Vitest de la PR 46 rendent `undefined`) ; aucune écriture si l'URL canonique est déjà la même.
  « Empile » et « remplace » désignent les deux formes de `goto` (option `replaceState`), non l'API `pushState` de
  `$app/navigation` (C-55-1-3).
- **Conséquence voulue** : un clic sur un onglet dont le rapport est affiché écrit la clé de ce rapport, même si un champ a
  été modifié sans *Générer* — l'URL décrit le rapport, non le formulaire (C-55-1-1).

### Appliquer une URL (D3) — la page lit

`applyUrl(state)`, appelée par un `$effect` sur `page.url` (sous `untrack`), qui remplace l'`onMount` (`:546`) et l'effet
du grand livre (`:567`), **seulement si la clé lue diffère de `lastHandledKey`** ; elle pose `lastHandledKey` d'abord
(un *Retour* vers une entrée, puis *Suivant*, puis *Retour* sont chacun appliqués) :

1. **Onglet** : `activeTab` prend la valeur lue ; `errorMsg` est effacé (comme `selectTab`, correctif M10).
2. **Champs** — URL **complète** : chaque champ de l'onglet prend la valeur de l'URL, et **un champ absent prend son
   défaut** (période vide, compte nul, `includeZero` faux) ; URL **incomplète** : aucun champ n'est modifié. Les champs des
   autres onglets ne bougent pas ; les champs des rapports par compte analytique ne sont **jamais** touchés (leur garde
   reste armée). La garde P17 est **désarmée pour cette application** (`lastFyId` posé avant les champs) ; si l'exercice
   change, les rapports des autres onglets sont vidés comme par P17 (et leurs clés, point 4), pas la période restaurée.
3. **Rapport** — URL incomplète : le rapport affiché de l'onglet s'il y en a un, sinon l'invite ; aucune requête, aucune
   péremption. URL complète : si la **clé affichée** ou la **clé en vol** de l'onglet est celle-ci, rien (le rapport est
   là, ou il arrive) ; sinon `generate()` — **mais si une génération est en vol** (`loading`, d'un autre
   onglet), la demande est **différée à sa fin**, comme la 46-2 diffère la sienne (`afterGeneration`) : aucune génération
   n'est interrompue par une navigation (aujourd'hui *Générer* est désactivé pendant `loading`, `ReportSelector.svelte:38`,
   `+page.svelte` PR 46 `:907,982,1017` : rien n'interrompt un chargement, sinon un geste de présentation). Puis, si rien
   n'a été demandé, la règle de la 46-2 (`resyncActiveGroupedTab`).
4. **Clés** (invariant) : la **clé affichée** d'un onglet est celle de la réponse acceptée qu'il affiche ; la **clé en vol**,
   par onglet, est posée au départ de la requête et devient la clé affichée à l'acceptation ; elle est retirée à l'échec et
   à la péremption ; un vidage de rapport (P17, compte analytique) retire les deux. Une demande différée porte sa clé en
   vol dès qu'elle est enregistrée.
5. **Course avec un geste** : si un Retour supplante un `goto` de la page avant que l'effet n'ait lu la clé écrite, l'URL lue
   est celle du Retour, qui n'est pas la clé écrite : elle est appliquée (aucun drapeau consommé à tort).

### Valeurs invalides (D5)

Chaque paramètre est jugé seul ; une valeur invalide est traitée comme absente (point 2), et l'URL est **normalisée** par
un `goto` qui remplace (aucune entrée d'historique, aucun message), enregistré comme écrit par la page. Une URL rendue
incomplète par la normalisation n'est pas demandée (le `fy` invalide d'un bilan donne l'invite). Un exercice inconnu est
invalide. Période hors exercice, `from > to`, compte inconnu de la société : jugés **par l'API**, avec ses messages
d'aujourd'hui.

## Critères d'acceptation

1. **AC-1 — Retour après un lien de compte.** Bilan de l'exercice E, période P, affiché ; clic sur un numéro de compte →
   grand livre de ce compte sur P ; *Retour* → **le Bilan de E sur P**, sans clic sur *Générer*, **sans nouvelle requête**
   `GET /api/v1/reports/balance-sheet` ; *Suivant* → le grand livre, sans nouvelle requête. Même chose depuis la Balance.
2. **AC-2 — Retour entre onglets.** Bilan généré (E) ; clic sur l'onglet *Compte de résultat* (jamais généré) → une entrée
   d'historique de plus, URL `?tab=income-statement`, invite, **aucune requête** ; *Générer* → URL
   `?tab=income-statement&fy=E…` (remplacée) ; *Retour* (bouton du navigateur) → le Bilan, onglet *Bilan* `aria-selected`,
   sans requête ; *Suivant* → le compte de résultat, sans requête.
3. **AC-3 — *Générer* remplace.** Bilan de E affiché ; période changée, *Générer* → l'URL porte la nouvelle période,
   `history.length` est inchangé, **une** requête.
4. **AC-4 — Lien partagé et rechargement.** Une URL copiée — classique (onglet, exercice, période), grand livre (compte,
   dates, comptes sans mouvement) — ouverte dans un nouveau
   contexte du navigateur **qui reprend les cookies de la session** (`browser.newContext({ storageState })`, `storageState`
   réduit aux cookies, `localStorage` vide) ou rechargée affiche **le même rapport, contrôles remplis**, sans clic. Le lien
   rouvre la vue **pour un utilisateur connecté** : non connecté, la garde mène à la connexion, qui ne ramène pas à
   l'adresse (`(app)/+layout.ts:10-11`, `login/+page.svelte:74`) — hors périmètre, issue à ouvrir.
5. **AC-5 — Valeurs invalides.** `tab=xyz`, `fy=999999`, `fy=abc`, `from=2026-02-30`, `accountId=-3`, un paramètre étranger
   à l'onglet : chacun ignoré, URL normalisée sans entrée d'historique, aucune erreur affichée, aucune valeur invalide
   envoyée à l'API ; `?tab=balance-sheet&fy=abc` donne l'invite. Une période hors exercice garde le message de l'API.
6. **AC-6 — Compatibilité.** `ledgerHref(a, f, t)` rend la même chaîne qu'avant (trois cas) ; `homepage-charts.spec.ts` vert
   sans modification ; `/reports?tab=aged-receivables` (échéancier, rappels) affiche la balance âgée, **désormais sans
   clic** ; les Vitest `reports-page.test.ts` et `reports-page-groupes.test.ts` (PR 46) passent **sans modification**, et le
   scénario X1 de la 46-2 (niveau changé sur le Bilan, *Générer* sur le Compte de résultat, retour sur le Bilan : redemande)
   est **rejoué** dans le Vitest neuf avec un `page.url` qui suit `goto` (il ne doit plus passer à vide).
7. **AC-7 — Requêtes et `goto`.** Par geste, comptés (Vitest) : *Générer* — 1 requête, 1 `goto` ; clic sur un onglet —
   0 requête, 1 `goto` ; clic sur *Balance âgée* jamais générée — 0 requête, 1 `goto` ; arrivée sur `?tab=aged-receivables`
   — 1 requête, 0 `goto` ; Retour vers une URL dont le rapport est affiché ou en vol — 0 requête, 0 `goto` ; arrivée sur une
   URL invalide ou valide mais **non canonique** (p. ex. `?tab=general-ledger&from=…&to=…&accountId=…` de
   `open-items.spec.ts:357`) — 1 `goto` de normalisation qui remplace. Aucune boucle.
8. **AC-8 — Course.** (a) *Générer* sur le Compte de résultat, puis, requête en vol, **clic** sur l'onglet *Balance* jamais
   générée, puis **clic** sur le Compte de résultat : sa réponse s'affiche. (b) *Retour* vers une URL complète d'un rapport
   absent de la mémoire pendant la génération d'un autre onglet : cette génération **n'est pas interrompue** ; sa réponse
   s'affiche dans son onglet, puis le rapport de l'URL est demandé (une requête) et s'affiche.
9. **AC-9 — Restauration sans effacement.** Retour vers une URL d'un autre exercice : la période restaurée n'est pas
   effacée, et la génération lancée n'est pas périmée par P17. Retour pendant qu'un rapport par compte analytique est
   affiché : ses champs et son rapport ne bougent pas (sa garde reste armée).
10. **AC-10 — Erreur et accessibilité.** Échec sur le Compte de résultat, puis *Retour* au Bilan : aucun message d'erreur
    au-dessus du Bilan. Après Retour, l'onglet de l'URL est `aria-selected` et `tabindex=0` ; chaque `goto` de la page
    reçoit `noScroll: true, keepFocus: true` (Vitest) ; sur une page défilée (rapport long, `window.scrollY > 0` avant),
    `window.scrollY` est le même après un clic d'onglet (E2E) ;
    au `popstate`, focus et défilement sont ceux de SvelteKit (non testés : comportement du framework) ; test axe de
    `reports.spec.ts` à zéro violation.
11. **AC-11 — Manuel.** `docs/manual/fr/user-manual.tex` (lignes à `1e23edb55`, relevées sur la base PR 46 au T0) :
    **un** paragraphe « Naviguer dans les rapports », placé dans § *Visualisation et exports* (`:4368`), avec un renvoi depuis
    l'introduction de § *Rapports comptables* (`:4101`) : Retour et Suivant, chaque onglet une étape, adresse à copier (pour un utilisateur connecté), ce qu'elle
    retient (onglet, exercice, période, compte), ce qu'elle ne retient pas (un champ modifié sans *Générer*, les rapports
    par compte analytique), et qu'arriver par Retour ou par un lien pendant le chargement d'un autre rapport attend sa fin ; § *Grand livre*, l'item « Accès direct depuis le bilan et la balance » (`:4220`) : Retour ramène au
    rapport. Formulation à reprendre du précédent de la liste des factures (« la page et le filtre suivent l'adresse, si
    bien que le retour arrière du navigateur les retrouve »). `.tex` seul, PDF aplati contrôlé avant. Les manuels DE, EN et
    IT n'existent pas encore (`docs/manual/{de,en,it}/` ne porte qu'un `README.md`).
12. **AC-12 — Textes.** Aucun texte visible n'est ajouté (prévu) ; s'il en faut un, quatre locales dans le bloc
    `reports-nav-*`, `i18n-keys` (`sitesTotal` recompté par la garde) et `i18n-libelle-en-dur` verts.

## Tests à écrire, et la preuve du rouge

- **Vitest `frontend/src/lib/features/reports/reports-url.test.ts`** (neuf) : lecture et écriture de chaque paramètre ;
  aller-retour sur les URL canoniques ; chaque valeur invalide de l'AC-5 ; ordre canonique ; `ledgerHref` égal à
  l'ancienne sortie (trois cas).
- **Vitest de la page** (`frontend/src/routes/(app)/reports/reports-page-url.test.ts`, neuf), avec un helper neuf
  `reports/page-url.test-helper.svelte.ts` (patron de `journal-entries/[id]/page-state.test-helper.svelte.ts`, qui ne porte
  que `params`) : `page.url` en `$state`, **remplacé** à chaque navigation, et un `goto` simulé qui l'écrit ; un test avec
  un `goto` **différé** (promesse résolue à la main) pour un Retour qui supplante un geste (point 5). Cas : AC-2, AC-3,
  AC-6 (X1 rejoué), AC-7 (tous les compteurs, normalisation comprise), AC-8 (a, b), AC-9, AC-10 (erreur, options de
  `goto`) ; **clés** : lien de compte puis *Retour* (appliqué), *Retour* puis *Suivant* (appliqué) ; **règle de
  l'absent** : lien de compte après un grand livre « comptes sans mouvement » (requête sans `includeZero`), URL complète
  sans `from`/`to` après un autre onglet à période ; **clés** : échec puis Retour (redemande), P17 puis Retour.
- **E2E Playwright `frontend/tests/e2e/reports-navigation.spec.ts`** (neuf) : AC-1 (`page.goBack()`, `page.goForward()`,
  requêtes comptées par `page.on('request')`), AC-2, AC-3, AC-4 (`browser.newContext({ storageState })` réduit aux cookies,
  et `page.reload()`), AC-5 (deux valeurs invalides), AC-10 (`window.scrollY`).
- **Preuve du rouge** : l'E2E AC-1 est lancé **avant** le correctif (journal `kesh-gate-logs/s55-1a/e2e-rouge-avant.log`) et
  rougit sur « Bilan affiché après Retour » ; les Vitest neufs de la page rougissent sans le correctif (journal). Mutations,
  chacune tuée par un test nommé au Change Log : *Générer* qui empile ; clé écrite non enregistrée (l'effet réapplique →
  double requête) ; retrait du désarmement de P17 ; clé en vol
  ignorée au point 3 ; champ absent conservé au lieu du défaut ; `errorMsg` non effacé ; `applyUrl` qui ne pose pas
  `lastHandledKey` ; demande non différée pendant `loading` (interruption) ; écriture de l'URL dans `generate()` ; dans
  `reports-url.ts`, retrait de la validation de `fy`.

## Recette visuelle

Captures (thème clair, base de démonstration, `kesh-gate-logs/s55-1a/`) : Bilan avant le lien ; grand livre après le lien ;
Bilan après *Retour* (barre d'adresse visible) ; lien collé dans un nouveau contexte ; URL normalisée après une valeur
invalide. Jointes au Dev Agent Record et au prompt de la revue de code.

## Tâches

- [ ] **T0** — Reprendre la PR 46 dans `epic/55-navigation` (`git merge`, conflits résolus en gardant les deux côtés) ;
      relever les lignes de la page et du manuel à cette base ; vérifier qu'aucun filtre de devise n'existe.
- [ ] **T1** — E2E AC-1 rouge (journal).
- [ ] **T2** — `reports-url.ts` et ses Vitest ; `ledgerHref` réécrite dessus.
- [ ] **T3** — Page : écriture de l'URL aux gestes (clé enregistrée avant `goto`) ; `applyUrl` et son `$effect` (remplacent
      `onMount` `:546` et l'effet `:567`) ; clés affichée et en vol par onglet ; désarmement de P17 et de l'effet du compte
      analytique.
- [ ] **T4** — Suites existantes : Vitest `reports-page.test.ts`, `reports-page-groupes.test.ts` ; E2E `reports.spec.ts`,
      `reports-groupes*.spec.ts`, `reports-currency.spec.ts`, `reports-export-pdf.spec.ts`, `dunning-roundtrip.spec.ts`,
      `open-items.spec.ts`, `homepage-charts.spec.ts`, `invoice-write-off.spec.ts`, `opening-balances.spec.ts` ;
      adaptations nommées (AC-6).
- [ ] **T5** — Manuel, captures.
- [ ] **T6** — Gates (ci-dessous).

## Gates

`cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build` ; E2E : la spec neuve et
les specs de T4 ; suite complète selon le § *Gates* du fichier d'epic (Q6 : tant qu'elle n'est pas tranchée, E2E complet au
dernier commit de code de la story). Aucun gate backend.

## Note de version

*Rapports* : *Retour* et *Suivant* du navigateur ramènent au rapport précédent, avec son exercice, sa période et son
compte ; chaque changement d'onglet est une étape de l'historique ; l'adresse d'une vue se copie et rouvre la même vue pour
un utilisateur connecté, sans cliquer sur *Générer* (comme les liens de l'échéancier et des rappels vers la balance âgée,
qui l'affichent désormais à l'arrivée) ; le lien *Rapports* du menu ramène au Bilan. (refs #À-VENIR)

## Dev Notes

### Pièges

- **Effet P17** : voir *Appliquer une URL*, point 2 ; l'effet du compte analytique n'est jamais désarmé.
- **`load` ne doit pas lire `url`** : sinon SvelteKit le rejoue à chaque changement de paramètre.
- **Liste des comptes** : un `accountId` absent de la liste n'est pas invalide côté client — l'API juge.
- La PR 46 garde l'affichage pendant une redemande (`refreshing`) et périme par `invalidateInFlight()` : la page passe par
  elle, jamais par `genSeq++` direct.

### Choix et écarts

- **C-55-1-1** — URL = dernier rapport demandé, non l'état du formulaire.
- **C-55-1-2** — Un rapport en mémoire par onglet (sa réponse et sa clé) : *Retour* plus loin qu'une vue de l'onglet
  redemande à l'API (une trace d'audit de plus, assumée).
- **C-55-1-3** — `goto` plutôt que le routage superficiel de `$app/navigation`.
- **C-55-1-4** — Le clic sur un onglet ne génère pas (validation P1) ; il passe par `selectTab` d'aujourd'hui, non par
  `applyUrl` (validation P3, V3-1 : appliquer l'URL écrite par le clic recopiait dans les contrôles l'état de l'ancien
  rapport et faisait passer à vide la règle de la 46-2).
- **C-55-1-5** — Clavier en `replace` : une entrée par flèche rendrait *Retour* inutilisable.
- **C-55-1-6** — Arriver par une adresse pendant le chargement d'un autre rapport **attend sa fin** (validation P4, V4-3 :
  la prémisse « *Générer* interrompt » était fausse — le bouton est désactivé pendant `loading`) ; une génération à la fois,
  sans refaire la garde `genSeq` de la PR 46.
- **C-55-1-7** — Rapports par compte analytique hors de l'URL (validation P4, V4-1), issue à ouvrir.

### References

- `CLAUDE.md` § *Test Locally First* (E2E), § *Recette visuelle*, § *Travailler à plusieurs développeurs*.
- Fiche 46-2 (`pr/46-rapports-groupes:_bmad-output/implementation-artifacts/46-2-groupes-ecrans.md`), AC-3, AC-6, AC-10 ;
  fiche 55-1b.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### Issues à ouvrir

(Points de revue non corrigés : sévérité, site, texte proposé ; source de la story de solde 55-4.)

- (LOW, `enhancement`) Les rapports par compte analytique (dépenses, rendement) ne sont pas dans l'URL : *Retour* et un
  lien n'y ramènent que l'onglet (C-55-1-7) (Epic 55, story 55-1a).
- (LOW, `bug` P3, préexistant) L'export d'un rapport classique reprend la période des champs, et le nom de fichier celle
  du rapport affiché (`+page.svelte` PR 46 `:660-690`) : un champ modifié sans *Générer* exporte une autre période que
  l'écran (Epic 55, story 55-1a, relevé de validation).
- (LOW, `enhancement`) Après la connexion, Kesh mène à l'accueil et non à l'adresse demandée (`(app)/+layout.ts:10-11`,
  `login/+page.svelte:74`) : un lien de rapport partagé à un utilisateur non connecté ne rouvre pas la vue.

### File List

## Change Log

- 2026-11-25 — Fiche écrite au kickoff de l'Epic 55 (`drafted`).
- 2026-11-25 — Validation P1 (Opus) : 0/1/10/8 ; remédiée : le changement d'onglet ne génère plus (V1-1, C-55-1-4, D3 du
  fichier d'epic amendée, règle de la 46-2 gardée) ; période des rapports par projet dans l'URL (V1-2) ; application d'une
  URL en quatre points — champs, désarmement de P17 et de l'effet du compte analytique, clé en vol, invalidation (V1-3,
  V1-4) ; écriture des valeurs qui décrivent la requête (V1-5) ; regroupement tout ou rien, écho reporté (V1-6, V1-7) ;
  dernier rapport de l'onglet au clic (V1-8) ; `page` réactif dans les Vitest (V1-9) ; manuel de la PR 46 (V1-10) ;
  clavier en `replaceState` (V1-11) ; LOW V1-12 à V1-19.
- 2026-11-25 — Validation P2 (Sonnet) : 0/0/5/7 ; remédiée : drapeau du clic, qui ne génère jamais, balance âgée comprise
  (V2-1) ; aucune péremption d'un autre onglet, AC-8 réécrite en trois cas (V2-2) ; règle de l'absent (V2-3) ; invariant
  des clés affichée et en vol (V2-4) ; AC-4 sur les cookies de la session, issue du retour après connexion (V2-5) ; LOW
  V2-6 à V2-12 (resynchronisation après le point 4, menu *Rapports*, AC-10 constatable, helper de `page.url`, mutations,
  lecture et forme canonique, rédaction).
- 2026-11-25 — Validation P3 (Opus) : 0/1/6/6. **Signal de recyclage** (sévérité remontée de MEDIUM à HIGH, défauts nés des
  remédiations de la P2 : V3-1 du clic qui écrit une clé, V3-2 à V3-4 de `b4f3eb072`) : **découpage**, déclaré au
  mainteneur au fichier d'epic — la fiche devient la **55-1a** (URL sans la présentation de l'Epic 46), la présentation
  passe à la **55-1b**. Modèle **simplifié** : la page n'applique jamais l'URL qu'elle écrit (clé enregistrée avant `goto`),
  le clic passe par `selectTab` d'aujourd'hui (V3-1, V3-6, V3-10, V3-11) ; clé en vol au point 3 (V3-2) ; interruption
  d'une génération par une adresse, écrite au Change Log, au manuel et à l'AC-8 (V3-3, V3-4, C-55-1-6) ; écriture pour
  l'onglet actif seulement (V3-5) ; `errorMsg` effacé (V3-7) ; LOW V3-8 (55-1b), V3-9, V3-12 (55-1b), V3-13.
- 2026-11-25 — Validation P4 (Sonnet) : 0/1/4/8. **Signal de recyclage maintenu** (HIGH à P3 et P4, défauts distincts ;
  V4-1 né d'une remédiation de la P1), déclaré au mainteneur au fichier d'epic. Remédiation **par réduction du périmètre**
  plutôt que par une règle de plus : rapports par compte analytique **hors de l'URL** (V4-1, C-55-1-7, issue à ouvrir) ;
  `lastHandledKey` unique, posée à chaque `goto` et à chaque application (V4-2) ; une navigation **n'interrompt plus**, la
  demande attend la fin de la génération en vol (V4-3, prémisse fausse corrigée, C-55-1-6) ; URL écrite par les seuls
  gestionnaires, `goto` en dernier, AC-8(c) retiré (V4-4) ; `estDateStockable` (V4-5) ; LOW V4-6 à V4-12 (dont deux issues
  à ouvrir). La passe suivante est la P5 ; dès la P6, l'orchestrateur ne remédie plus (retrait ou renvoi).
