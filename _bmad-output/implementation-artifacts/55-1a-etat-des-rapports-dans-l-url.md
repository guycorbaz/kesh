# Story 55-1a : l'état des Rapports dans l'URL — Retour, Suivant, lien partagé

Status: ready-for-dev

Epic : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md` — D1 à D3 (réponse de l'utilisateur du 2026-11-25, AC-13), D5 (D4 : 55-1b). Issue : #À-VENIR (`bug`, P3, point b).
Release : la prochaine planifiée après la 0.20.0 (numéro fixé par le mainteneur). **Frontend seulement, aucune migration.**
**Base : la PR de l'Epic 46** (`pr/46-rapports-groupes`, `511e61d54` au kickoff), reprise dans `epic/55-navigation` avant T1.

## ⚠️ Points renvoyés au développement (validation P6)

La validation s'est arrêtée à la **sixième passe** sur deux MEDIUM (`kesh-gate-logs/s55-kickoff/55-1a-validation-p6.md`).
Par la règle de `CLAUDE.md` § *Review Iteration Rule* (« dès la sixième passe, l'orchestrateur ne remédie plus lui-même —
il retire, ou renvoie au développement »), ils sont **renvoyés au développement** : le T0 les tranche **avant toute
ligne de code**, avec le mainteneur si le choix change le comportement visible, et le Change Log cite la ligne lue et la
décision. Le correctif proposé par la passe est la résolution par défaut.

- **R-1 (V6-1)** — *Appliquer une URL*, point 3, n'a pas de branche pour « une génération **de cet onglet** en vol avec une
  autre clé, et la clé affichée différente de l'URL (ou rien d'affiché) » (p. ex. *Générer* sur le Bilan P1, puis arrivée
  sur le Bilan P2). Défaut proposé : `generate()` pour la clé de l'URL (qui périme la génération en vol du même onglet) ;
  un AC-8 (d) et une mutation.
- **R-2 (V6-2)** — Branche « génération d'un autre onglet en vol » : poser les champs de l'URL alors que l'onglet affiche un
  rapport d'une autre clé fait dire trois choses à l'écran, aux champs et à l'URL, et l'export suit les champs. Défaut
  proposé : dans cette branche, **ne poser ni champs ni exercice**, et remplacer l'URL par la clé affichée de l'onglet (ou
  `?tab=<id>` s'il n'a rien) ; AC-8 (b) vérifié avec un rapport affiché d'une autre période ; C-55-1-6 corrigé (« comme un
  clic »).
- **R-3 (P7, V7-1) — L'AC-13 prime partout.** Au T0, avant tout code, chaque site de l'ancienne règle « une adresse
  pendant la génération d'un autre onglet ne demande rien » est récrit dans le sens de l'AC-13 (l'adresse demande son
  rapport, l'autre génération est périmée) : point 3 et la phrase « Aucune navigation n'interrompt » ; **AC-8 (b)**
  (1 requête, l'autre génération périmée) ; la mutation « requête partie pendant la génération d'un autre onglet » (à
  inverser : « aucune requête » → test rouge) ; le texte du manuel de l'AC-11 ; C-55-1-6 ; le défaut de R-2 (remplacé) ; la
  liste des tests. Même alignement dans la 55-1b (son AC-12 : *Retour* et un lien interrompent aussi un chargement).
- **R-4 (P7, V7-3) — tranché le 2026-11-25** (réponse de l'utilisateur, partie A, point 2) : « pas de requête en
  double » était une formulation technique de la consigne, non une demande de l'utilisateur. La lecture par défaut est
  gardée comme **exigence technique seulement** : **jamais deux appels simultanés pour un même jeu de paramètres, ni
  d'appel pour le rapport déjà affiché** — le modèle garde un rapport par onglet (C-55-1-2) : *Retour* plus loin, un
  changement d'exercice ou une génération interrompue redemandent. Rien à confirmer au T0.
- **R-5 (P7, V7-4) — Portée du gras : remplacé le 2026-11-26** par D15 (AC-14 à AC-19) : un seul composant,
  `ReportSelector`, sur tous les onglets à période, menu natif entièrement en gras ; plus de gras partiel ni de
  `components/ui/select`. **R-7** (piloter `ui/select` sous jsdom, passe ciblée du 2026-11-26) est **retiré** avec lui.
- **R-8 (D15, 2026-11-26) — Le sélecteur d'exercice, versé au-delà du plafond de huit passes** (une passe ciblée seulement,
  dont les MEDIUM s'ajoutent ici en R-9 et suivants) : au T0, avant le code, relire contre D15 les sites que le sélecteur
  touche dans le modèle — P17 amendé (*Appliquer une URL*, point 2 ; AC-8 b' ; AC-9 ; mutation « retrait du désarmement de
  P17 »), la clé des champs comparée à la clé affichée (AC-16), `fy` déduit des dates (AC-5, AC-15) —, et confirmer avec le
  mainteneur la décision elle-même (question au fichier d'epic) et C-55-1-8.
- **R-9 à R-13 (passe ciblée D15, Sonnet, `kesh-gate-logs/s55-selecteur/selecteur-2026-11-26-validation.md`, 0/0/5/3, aucune remédiation)** — à trancher au T0
  avant le code, ligne lue citée au Change Log :
  - **R-9 (SE-1)** — Le modèle d'URL n'a pas suivi l'AC-15 : « période facultative » (tableau des paramètres), « une période
    vide ne s'écrit pas », « Complète : classiques — `fy` », AC-5 (« période hors exercice garde le message de l'API ») et D5
    du fichier d'epic ; fixer le `fy` écrit en « Plusieurs »/« Hors exercice » et *Générer* sur une date vidée ; un Vitest
    de `reports-url.test.ts` par cas.
  - **R-10 (SE-2)** — Le périmé (AC-16) repose sur une clé affichée que les rapports par compte analytique n'ont pas : dire
    leur sort au changement de date (vidage ou grisé, export), et ajouter un Vitest et une mutation pour eux et pour le
    Grand livre périmé.
  - **R-11 (SE-3)** — Le sort de `selectedFiscalYearId` : `$bindable` de `ReportSelector` (`:26`) alors que le Grand livre ne
    l'écrit jamais ; sa valeur en « Plusieurs »/« Hors exercice » ; l'effet P17 amendé déclenché par une date saisie ; le
    préremplissage de la 55-2 (AC-12) qui en dépend.
  - **R-12 (SE-5)** — Les choix des fiches non déclarés (exports désactivés quand périmé, compte du Grand livre dans le
    périmé, en-tête « (hors exercice) », bornes complètes d'un doublon, message `reports-nav-needs-one-fy`, majuscule de
    « Du », guillemets du nom saisi) : à lister en C-55-1-10 et à joindre à la question au mainteneur.
  - **R-13 (SE-6)** — Le décompte des modules (six avec `i18n-keys.test.ts`, au-dessus du seuil de cinq) et la taille : poser
    au mainteneur le vrai choix avant le T0 (story séparée avec dérogation à la borne de six, ou 55-1a en L).
  - LOW : SE-4 (restes de P17 : AC-9, mutation, T3, Pièges), SE-7 (`fiscalYears` des six vues, doublon dans l'en-tête),
    SE-8 (format jour-mois sans test de forme ; DE/IT/EN de six clés à écrire).
- **R-6 (décision Q6 du mainteneur, 2026-11-25, versée sans passe : plafond atteint)** — § *Gates* aligné sur la
  *Dérogation* du fichier d'epic (story purement écran : specs des Rapports et de la navigation à la story, E2E complet et
  `gate-ci.sh` en fin d'epic) ; le T0 relit la phrase contre le fichier d'epic et le cite au Change Log.
- **LOW de la P8** (`kesh-gate-logs/s55-maquettes/55-1a-validation-p8.md`, dernière passe, aucune remédiation) : V8-1
  (« aucune requête » au clic sur un onglet s'entend **du changement d'onglet lui-même** : la remise d'accord de la 46-2,
  `resyncActiveGroupedTab`, peut redemander un rapport groupé — à écrire ainsi dans l'AC-13, le tableau des gestes et D3) ;
  V8-3 (preuve du rouge de l'AC-13 : l'assertion décisive est « onglet actif et rapport affiché égaux à ceux de l'URL », non
  le seul décompte de requêtes ; « rechargement » et « lien partagé » sont un seul cas au Vitest) ; V8-2, V8-4, V8-5 portent
  sur `sprint-status.yaml`, la 55-3b et la 55-3a (repris là).
- **LOW à reprendre** : V7-5 à V7-10 (`ReportSelector` « modifié » à `:41` et au fichier d'epic ; tests et preuve du rouge de
  l'AC-13 et de l'AC-14, `ReportSelector.test.ts` neuf ; « par *Suivant* » quand on est parti par *Retour* ; le test « champ
  modifié sans clic » vérifie que le rapport reste affiché et n'emploie pas l'exercice ; « rapport absent de la mémoire » ;
  l'AC-13 tranche aussi R-1 ; « par lui » = par la page à l'arrivée d'une adresse) ; V6-3 (la phrase « aucune navigation n'interrompt » oublie la branche « clé affichée »), V6-4
  (*Générer* ne se désactive que pendant `loading`, non `refreshing`), V6-5 (fichier d'epic `:198`, « +287 » → « +231 nettes »),
  V6-6 (*Générer* : 0 `goto` si l'URL est déjà égale ; comparer sans le `?` de `page.url.search`), V6-7 (le test qui tue la
  mutation « clé écrite non enregistrée » vise un onglet qui affiche un rapport d'une autre période), V6-8 (la période d'un
  rapport par compte analytique suit celle des classiques au Retour : même classe que l'issue d'export, à y joindre).

## Story

En tant que comptable qui lit ses rapports,
je veux que *Retour* et *Suivant* du navigateur me ramènent au rapport que je regardais, avec sa période et ses filtres, et
qu'un lien copié rouvre la même vue,
afin de passer du bilan au grand livre d'un compte et d'en revenir sans perdre ma place ni tout régénérer.

## Modules touchés (règle de splitting préventif)

Cinq depuis le 2026-11-26 (D15), **à la borne** de la règle : `frontend/src/lib/features/reports` (`reports-url.ts`,
`fiscal-year-menu.ts`, `ReportPeriodHeading.svelte` neufs ; `reports.api.ts`, `ReportSelector.svelte` et les six vues
modifiés), `frontend/src/routes/(app)/reports` (page), `frontend/tests/e2e`, `docs/manual/fr/user-manual.tex`, `kesh-i18n`
(dix clés, AC-18). Aucun code Rust. ⚠️ La story grossit d'un sélecteur complet : taille déclarée au fichier d'epic
(*Stories*) ; si le T0 juge l'ensemble trop lourd, le découpage (sélecteur à part) se propose au mainteneur avant le code,
l'epic étant à six stories.

## Ce qui se lit au code

Relevé à `origin/main` `1e23edb55` et sur `pr/46-rapports-groupes` `511e61d54` (`git show`). Les numéros de ligne de la page
sont ceux de `origin/main` ; ceux de la PR 46 se relèvent au T0 (+231 lignes nettes).

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
  `open-items.spec.ts:357`, `reports.spec.ts:319,432,461` (`:423` est une assertion d'URL), `homepage-charts.spec.ts`.
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
| Exercice, date, compte, « comptes sans mouvement », compte analytique, mode modifiés | choix d'un exercice : dates aux bornes (AC-15) ; rapport affiché grisé (AC-16) ; l'effet du compte analytique vide comme aujourd'hui, P17 ne vide plus que les rapports par compte analytique | **aucune** : l'URL reste celle du dernier rapport demandé, jusqu'à *Générer* | 0 |
| Contrôles de présentation et accordéons (Epic 46) | la 46-2 redemande, comme aujourd'hui | **aucune** dans la 55-1a (la présentation entre dans l'URL à la 55-1b) | 0 |
| Clic sur un lien de compte (`<a href={ledgerHref(…)}>`) | — | **empile** (navigation SvelteKit, inchangée) : URL **non écrite par la page**, donc appliquée | 0 |

- **Où l'URL s'écrit** : seulement dans les gestionnaires des gestes (`selectTab`, clavier, bouton *Générer*), une seule
  fois par geste ; aucune réponse n'écrit l'URL dans la 55-1a (la 55-1b ajoute l'écho). `selectTab` appelle `goto` **en
  dernier**, après `resyncActiveGroupedTab` (qui n'écrit pas) : aucun second `goto` dans la même tâche n'abandonne le
  premier (`client.js:1809`, `:1873-1883`).
- **Clé** : une seule variable, `lastHandledKey`, posée à **chaque** `goto` de la page **et** à **chaque** application
  d'une URL, jamais consommée ; l'effet compare la clé canonique de l'URL lue à `lastHandledKey`.
- `goto` est appelé avec `keepFocus: true, noScroll: true`, toujours `void goto(…)` (sans `await` ni `.then` : les mocks
  `goto: vi.fn()` des Vitest de la PR 46 rendent `undefined`) ; aucune écriture si la recherche écrite égale `page.url.search`
  (comparaison au texte de l'adresse, non à `lastHandledKey`, sans quoi la normalisation n'aurait jamais lieu).
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
   défaut** (période vide — **depuis D15, les bornes de l'exercice** sur les onglets classiques, AC-15 —, compte nul,
   `includeZero` faux) ; URL **incomplète** : aucun champ n'est modifié. Les champs des
   autres onglets ne bougent pas ; les champs des rapports par compte analytique ne sont **jamais** touchés (leur garde
   reste armée). Les `from`/`to` valides d'une URL incomplète restent dans l'URL. **Si l'exercice change** : ce que fait P17
   aujourd'hui — tous les rapports vidés (rapports par compte analytique compris, qui lisent le même exercice ; leur compte
   et leur mode restent) et toute génération en vol périmée (`invalidateInFlight`), clés retirées —, puis la période de
   l'URL est posée (`lastFyId` posé avant les champs, pour que P17 ne l'efface pas après). *Amendé par D15 (AC-15,
   AC-16)* : P17 ne vide plus ni les dates ni les rapports classiques — ceux des autres onglets restent affichés, périmés
   si leur clé n'est plus celle des champs ; seuls les rapports par compte analytique sont vidés ; la péremption de la
   génération en vol reste.
3. **Rapport** — URL incomplète : le rapport affiché de l'onglet s'il y en a un, sinon l'invite ; aucune requête, aucune
   péremption. URL complète :
   - la **clé en vol** de l'onglet est celle-ci : rien (le rapport arrive) ;
   - la **clé affichée** est celle-ci : rien ; si une génération **de cet onglet** est en vol avec une autre clé,
     `invalidateInFlight()` — l'écran garde le rapport de l'URL, aucune requête (même règle que la 55-1b) ;
   - sinon, **aucune génération en vol** (ni `loading` ni `refreshing`) : `generate()` ;
   - sinon (une génération d'un autre onglet est en vol, au même exercice) : **aucune requête**, l'invite de l'onglet avec
     ses champs remplis ; l'utilisateur clique *Générer* quand le bouton se réactive (aujourd'hui *Générer* est désactivé
     pendant `loading`, `ReportSelector.svelte:38`, `+page.svelte` PR 46 `:907,982,1017`). *Écarté* (validation P5) : une
     demande différée à la fin de l'autre génération — son cycle de vie (onglet, annulation, clé) ouvrait plus de cas qu'il
     n'en fermait.
   Aucune navigation n'interrompt donc une génération, sauf au changement d'exercice (comme P17). Puis, si rien n'a été
   demandé, la règle de la 46-2 (`resyncActiveGroupedTab`).
4. **Clés** (invariant) : la **clé affichée** d'un onglet est celle de la réponse acceptée qu'il affiche ; la **clé en vol**,
   par onglet, est posée au départ de la requête et devient la clé affichée à l'acceptation ; elle est retirée à l'échec et
   à la péremption ; un vidage de rapport (P17, compte analytique) retire les deux.
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
   absent de la mémoire, **même exercice**, pendant la génération d'un autre onglet : aucune requête, l'invite de l'onglet
   avec ses champs remplis ; l'autre génération n'est pas interrompue et sa réponse s'affiche dans son onglet. (b')
   *Retour* vers une URL d'un **autre exercice** pendant une génération : celle-ci est périmée (comme P17), le rapport de
   l'URL est demandé. (c) *Retour* vers la clé affichée pendant une génération de cet onglet avec une autre période : 0
   requête, la génération périmée, l'écran garde le rapport de l'URL.
9. **AC-9 — Restauration sans effacement.** Retour vers une URL d'un autre exercice : la période restaurée n'est pas
   effacée, la génération lancée n'est pas périmée par P17 ; un rapport par compte analytique affiché est vidé (même
   exercice partagé), son compte et son mode restent. Retour au **même** exercice : le rapport par compte analytique, ses
   champs et sa garde ne bougent pas. Lien du menu *Rapports* cliqué depuis la page : onglet *Bilan*, aucune requête.
10. **AC-10 — Erreur et accessibilité.** Échec sur le Compte de résultat, puis *Retour* au Bilan : aucun message d'erreur
    au-dessus du Bilan. Après Retour, l'onglet de l'URL est `aria-selected` et `tabindex=0` ; chaque `goto` de la page
    reçoit `noScroll: true, keepFocus: true` (Vitest) ; sur une page défilée (rapport long, `window.scrollY > 0` avant),
    `window.scrollY` est le même après un clic d'onglet (E2E) ;
    au `popstate`, focus et défilement sont ceux de SvelteKit (non testés : comportement du framework) ; test axe de
    `reports.spec.ts` à zéro violation.
11. **AC-11 — Manuel.** `docs/manual/fr/user-manual.tex` (lignes à `1e23edb55`, relevées sur la base PR 46 au T0) :
    **un** paragraphe « Naviguer dans les rapports », placé dans § *Visualisation et exports* (`:4368`) (§ *Rapports
    comptables*, `:4101`, n'a pas d'introduction : il s'ouvre sur § *Bilan*) : Retour et Suivant, chaque clic sur un onglet
    une étape, adresse à copier (pour un utilisateur connecté), ce qu'elle
    retient (onglet, exercice, période, compte), ce qu'elle ne retient pas (un champ modifié sans *Générer*, les rapports
    par compte analytique), et qu'arriver par Retour ou par un lien pendant le chargement d'un autre rapport laisse l'onglet à son invite (*Générer*
    ensuite) ; § *Grand livre*, l'item « Accès direct depuis le bilan et la balance » (`:4220`) : Retour ramène au
    rapport. Formulation à reprendre du précédent de la liste des factures (« la page et le filtre suivent l'adresse, si
    bien que le retour arrière du navigateur les retrouve »). `.tex` seul, PDF aplati contrôlé avant. Les manuels DE, EN et
    IT n'existent pas encore (`docs/manual/{de,en,it}/` ne porte qu'un `README.md`).
12. **AC-12 — Textes.** *Remplacé le 2026-11-26 par l'AC-18* (le sélecteur d'exercice ajoute des textes).

### Exigences de l'utilisateur (2026-11-25), critères explicites

13. **AC-13 — Une adresse affiche son rapport sans « Générer » (D3).** *Récrit le 2026-11-25 sur la réponse de
    l'utilisateur (partie A, point 1)* : revenir par un **clic** sur un onglet déjà généré montre **déjà** son rapport
    aujourd'hui (`selectTab`, aucune requête) ; cette story ne le change pas et ne régénère rien au changement d'onglet.
    Ce qui reste à faire :
    - Une adresse de rapport **collée**, **rechargée**, ou atteinte par **Retour** ou **Suivant** affiche son rapport
      **sans clic sur *Générer*** : le rapport gardé par la page s'il a ces paramètres (aucune requête), une requête
      sinon (une trace d'audit, C-55-1-2).
    - Le clic sur un onglet reste **comme aujourd'hui** : le rapport que l'onglet a déjà, ou son invite ; **aucune
      requête**, jamais.
    - Un onglet **jamais généré**, atteint par un clic, reste à son invite jusqu'au clic sur *Générer*.
    - Une modification des paramètres **sans clic** ne régénère rien (C-55-1-1).
    - **Exigence technique** (R-4, tranché) : jamais deux appels simultanés pour un même jeu de paramètres, ni d'appel pour
      le rapport déjà affiché (clés affichée et en vol, point 4 ; comptés par les Vitest de l'AC-7). Ce n'est pas une
      demande de l'utilisateur : la note de version et le manuel ne la mentionnent pas.
    - **Ce critère tranche R-2** (point 3, génération d'un autre onglet en vol) : une adresse complète qui arrive pendant la
      génération d'un autre onglet **demande son rapport** (comme un clic sur *Générer*, une génération à la fois) ; la
      génération de l'autre onglet est périmée, et son onglet se réaffichera seul par son adresse (*Retour*), ou restera à
      son invite s'il n'avait encore rien affiché. C-55-1-6 et la branche « aucune requête, l'invite » du point 3 sont
      **remplacés** par cette règle, qui prime (R-3) ; le manuel (AC-11) dit « arriver par *Retour* ou par un lien pendant le
      chargement d'un autre rapport l'interrompt ».
    - Tests : Vitest de la page — *Retour* et *Suivant* (0 requête si gardé, 1 sinon), rechargement et lien partagé (1),
      adresse complète pendant la génération d'un autre onglet (1, l'autre périmée) ; **non-régression** du clic sur un
      onglet : onglet généré (0 requête, rapport affiché), onglet jamais généré (0, invite), champ modifié sans clic puis
      clic sur un autre onglet et retour (0, rapport inchangé) ; E2E de l'AC-1, de l'AC-2 et de l'AC-4. *Preuve du rouge* :
      les cas « Retour », « Suivant », « rechargement » et « lien partagé » rougissent sans le correctif ; les cas du clic
      sur un onglet sont **verts avant et après** (non-régression, déclarée telle au Change Log, non comptée comme preuve).
### Le sélecteur d'exercice (D15 — décision de l'utilisateur et avis UX du 2026-11-26, à confirmer par le mainteneur)

*Remplace l'ancien AC-14 (le nom seul en gras par le composant `components/ui/select`), avec R-5 et R-7 : le menu reste un
`<select>` natif, entièrement en gras, et ne montre plus les dates. Le composant `ui/select` n'est plus employé par les
Rapports.* Règles pures dans `frontend/src/lib/features/reports/fiscal-year-menu.ts` (neuf), sans état ; le composant et la
page ne font que les appeler (DRY : un seul endroit pour le texte d'un exercice, l'état du menu et l'en-tête).

14. **AC-14 — Le libellé et le menu.**
    - **Un seul libellé « Exercice »**, hors du menu, par un vrai `<label for="report-fiscal-year">` associé au
      `<select id="report-fiscal-year">` ; l'`aria-label` en double de `ReportSelector.svelte:51` est **retiré**. Constat :
      Vitest — `getByRole('combobox', { name: 'Exercice' })` trouve le menu, et le menu n'a **pas** d'attribut `aria-label`
      (mutation : `aria-label` remis → test rouge) ; E2E — `page.getByLabel('Exercice')` ; axe à zéro violation sur le Bilan
      (`reports.spec.ts`) et dans `reports-navigation.spec.ts`. Lu par un lecteur d'écran « Exercice, liste déroulante, 2025 »
      (constat manuel, déclaré tel au Dev Agent Record, non compté comme test).
    - **Le menu est en gras** : `select { font-weight: 600 }`, `option { font-weight: 400 }` (classes du thème
      `font-semibold` sur le `<select>`, `font-normal` sur les `<option>`, aucune couleur neuve). Le menu ne montrant plus
      les dates, le texte fermé est l'exercice seul, donc tout en gras. Constat : Vitest (classes présentes sur le `<select>`
      et sur chaque `<option>`) ; capture du menu fermé et du menu ouvert.
    - **Le texte d'un exercice** (`libellesExercices(fiscalYears): Map<id, string>`) :
      - les **années** — « 2025 », ou « 2025/2026 » pour un exercice décalé — quand le nom est celui que Kesh aurait donné,
        `name === nomParDefaut(startDate, endDate)`, réplique TypeScript de `nom_par_defaut`
        (`crates/kesh-core/src/accounting/exercice.rs:33-38` : « Exercice AAAA » si début et fin tombent la même année,
        « Exercice AAAA/AAAA » sinon ; commentaire qui renvoie à la fonction Rust) ;
      - sinon le **nom saisi**, tel quel (espaces compris, aucune troncature) ;
      - **doublon de texte** : les seules entrées concernées reçoivent leurs bornes jour-mois, « 2025 (01.03–30.06) » ;
        si les bornes jour-mois ne les séparent pas encore, les bornes complètes, « Compta (01.01.2024–31.12.2024) ». Les
        autres entrées ne changent pas.
    - Société sans exercice : menu vide et désactivé, message d'aujourd'hui (`reports-error-no-fiscal-year-available`),
      inchangé.
15. **AC-15 — Le menu et les dates (« Du », « Au »).**
    - Vocabulaire unique **« Du » / « Au »** (clés existantes `reports-ledger-from`, `reports-ledger-to`, quatre locales
      déjà traduites) : les libellés « Période — début » et « Période — fin », à moitié écrits en dur
      (`ReportSelector.svelte:63,75`), disparaissent.
    - **Les dates sont toujours remplies** : à l'ouverture de la page, avec les bornes de l'exercice choisi par défaut
      (`data.fiscalYears[0]`, comme `:82`) ; **choisir un exercice** remplit « Du » et « Au » avec ses bornes. *Amende P17*
      (`+page.svelte:151-173`) : le remplissage se fait dans le **gestionnaire du choix** (`onchange` du menu), jamais dans
      l'effet ; l'effet ne vide plus ni les dates ni les rapports classiques affichés (ils deviennent périmés, AC-16) ; il
      garde la péremption de la génération en vol (`invalidateInFlight`) et le vidage des rapports par compte analytique
      (leur garde d'export, `:182-195`, est inchangée). Une date changée à la main n'appelle jamais ce remplissage.
    - **L'état du menu se déduit des dates** (`etatDuMenu(du, au, fiscalYears, precedent)`) ; un exercice contient une date
      si `startDate ≤ date ≤ endDate` :
      - les deux dates dans **le même exercice** E : le menu montre E, **même pour une période partielle** ; si la période
        n'est pas exactement les bornes de E, un lien **« Exercice entier »** (`reports-nav-fy-whole`, bouton de style lien,
        `data-testid="report-fy-whole"`) apparaît à côté des dates et remet les bornes de E ;
      - les deux dates dans des **exercices différents** : une option **désactivée** « Plusieurs » (`reports-nav-fy-multiple` :
        FR « Plusieurs », DE « Mehrere », IT « Diversi », EN « Multiple ») est ajoutée et sélectionnée ;
      - **une date hors de tout exercice** : option désactivée « Hors exercice » (`reports-nav-fy-outside` : FR « Hors
        exercice », DE « Ausserhalb », IT « Fuori esercizio », EN « Outside fiscal years ») ;
      - **début après fin**, ou une date vide ou incomplète pendant la saisie : le menu **ne change pas** (`precedent`) —
        c'est une erreur de saisie, jugée par l'API comme aujourd'hui (D5) ;
      - dès qu'un exercice est choisi, l'option désactivée **disparaît** du menu.
    - **Onglets à un seul exercice** (Bilan, Compte de résultat, Balance, Journaux, TVA, rapports par compte analytique en
      mode *Exercice*) : l'API exige un exercice qui contienne la période (`crates/kesh-api/src/routes/reports.rs:65`,
      `fiscal_year_id: i64`, et `REPORT_PERIOD_OUT_OF_FISCAL_YEAR`) ; la requête prend l'exercice **déduit des dates**
      (`selectedFiscalYearId` suit le menu) ; avec « Plusieurs » ou « Hors exercice », *Générer* est **désactivé** et un
      message dit pourquoi (`reports-nav-needs-one-fy`, « Ce rapport porte sur un seul exercice : choisissez deux dates du
      même exercice. », `role="status"`). Le Grand livre, qui franchit la borne, génère dans tous les cas (55-2).
      *Choix de la fiche, à confirmer avec la décision* : la décision ne dit pas ce que fait *Générer* sur ces onglets ;
      désactiver est la seule issue sans changement d'API.
    - **Le même composant sur tous les onglets à période** : `ReportSelector` porte le libellé, le menu, « Du », « Au »,
      « Exercice entier », *Générer* et les exports ; les contrôles propres d'un onglet (compte et « comptes sans
      mouvement » du Grand livre, compte analytique et mode des rapports par compte analytique) lui sont passés par un
      *snippet* ; la page n'a plus qu'un bandeau à période (DRY). Le Grand livre l'emploie avec ses propres dates
      (`ledgerFrom`, `ledgerTo`) et `unSeulExercice={false}` (55-2, AC-11) ; le menu des rapports par compte analytique
      (`+page.svelte:747-760`, `data-testid="project-report-fiscal-year"`) est remplacé par celui de `ReportSelector`, avec
      « Du » et « Au » (la requête lit déjà la période des rapports classiques, `:228-230`) ; en mode *Cumulé*, menu et dates
      sont masqués. La balance âgée (arrêtée à ce jour) n'a pas de période : inchangée.
    - **URL (D1, D5)** : aucun paramètre neuf. Les dates étant toujours remplies, `from` et `to` s'écrivent toujours sur les
      onglets classiques ; ce sont eux qui déterminent l'exercice : `fy` reste écrit (l'exercice déduit) et ne se lit seul
      que si `from` et `to` manquent (les bornes de `fy` sont alors le défaut de la période, au lieu d'une période vide,
      *Appliquer une URL*, point 2) ; un `fy` qui contredit les dates est corrigé à la normalisation. Une URL classique dont
      les dates tombent dans plusieurs exercices n'est pas complète : l'invite, « Plusieurs », aucune requête.
16. **AC-16 — Rapport périmé.** Si l'exercice ou les dates — ou, au Grand livre, le compte et « comptes sans mouvement » —
    changent après *Générer*, le rapport affiché reste à l'écran, **grisé** (`opacity-60`, `aria-disabled` non posé : ses
    liens restent utilisables), et porte au-dessus de lui « Paramètres modifiés — cliquez sur Générer »
    (`reports-nav-stale`, `role="status"`, `data-testid="report-stale"`). Le critère est **la clé des champs différente de
    la clé affichée** de l'onglet (*Appliquer une URL*, point 4) : remettre les paramètres du rapport (p. ex. « Exercice
    entier ») retire le grisé sans requête. Tant que le rapport est périmé, **Export PDF et Export CSV sont désactivés** :
    l'export suit les champs (`+page.svelte` PR 46 `:660-690`) et exporterait autre chose que l'écran (l'issue « export
    d'une autre période » de la section *Issues à ouvrir* se réduit d'autant, à reformuler au T0). *Générer* retire le
    grisé à l'acceptation de la réponse.
17. **AC-17 — L'en-tête du rapport dit la période réelle.** Les lignes « Période : … — … » des vues
    (`BalanceSheetView.svelte:117-118`, `IncomeStatementView.svelte:37`, `TrialBalanceView.svelte:39`,
    `JournalReportView.svelte:39`, `VatReportView.svelte:23`, `GeneralLedgerView.svelte:207-208` à `ef993ae62`, relevées de
    nouveau sur la base PR 46 au T0) sont remplacées
    par un composant `ReportPeriodHeading.svelte` (neuf) qui lit la période **de la réponse** (non des champs) et
    `enTetePeriode(du, au, fiscalYears)` :
    - exercice entier : « Exercice 2025 — du 01.01.2025 au 31.12.2025 » (`reports-nav-heading-fy`, `{ $years }`), ou, nom
      saisi, « Compta TVA — du … au … » (`reports-nav-heading-named`) ;
    - partie d'un exercice : « Du 01.01.2026 au 31.03.2026 (partie de l'exercice 2025/2026) » (`reports-nav-heading-part`,
      `{ $fiscalYear }` = les années, ou le nom saisi entre guillemets « … ») ;
    - plusieurs exercices (Grand livre) : « Du … au … (exercices 2024/2025 et 2025/2026) » (`reports-nav-heading-several`,
      liste formée par `Intl.ListFormat` de la locale, type `conjunction`) ;
    - hors exercice (Grand livre) : « Du … au … (hors exercice) » (`reports-nav-heading-outside`).
    Dates au format suisse (`formatSwissDate`). Exports PDF et CSV inchangés (leur en-tête est celui du serveur).
18. **AC-18 — Textes.** *Remplace l'ancien AC-12 (« aucun texte visible n'est ajouté »).* Dix clés neuves dans le bloc
    `reports-nav-*` des quatre locales : `reports-nav-fy-multiple`, `-fy-outside`, `-fy-whole`, `-stale`, `-needs-one-fy`,
    `-heading-fy`, `-heading-named`, `-heading-part`, `-heading-several`, `-heading-outside` ; textes FR ci-dessus, DE, IT et
    EN écrits au T1 (les quatre libellés du menu sont imposés par la décision) ; aucune phrase assemblée par concaténation
    (la liste des exercices passe par `Intl.ListFormat`) ; `i18n-keys` (`sitesTotal` recompté par la garde) et
    `i18n-libelle-en-dur` verts ; `parity_between_locales` à la clôture (§ *Gates*).
19. **AC-19 — Manuel.** `docs/manual/fr/user-manual.tex`, § *Rapports comptables* : un paragraphe « Choisir la période »
    avant § *Bilan* (le menu montre l'exercice, ses années ou son nom ; « Du » et « Au » ; « Exercice entier » ; « Plusieurs »
    et « Hors exercice », et pourquoi le Bilan, le Compte de résultat, la Balance, les Journaux et la TVA ne portent que sur
    un exercice ; le rapport grisé « Paramètres modifiés » ; l'en-tête). Les phrases qui disent encore « période — début »
    ou que changer d'exercice vide les dates se corrigent (grep `période\|exercice` du § au T0). `.tex` seul, PDF aplati
    contrôlé avant.

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
- **Vitest `frontend/src/lib/features/reports/fiscal-year-menu.test.ts`** (neuf, D15) : `nomParDefaut` (mêmes cas que les
  tests de `exercice.rs` : même année, décalé, 29 février) ; `libellesExercices` — années pour un nom par défaut, nom saisi
  tel quel, nom saisi égal aux années d'un autre (« 2025 » saisi), doublon séparé par jour-mois sur les **seules** entrées
  concernées, doublon que jour-mois ne sépare pas (bornes complètes) ; `etatDuMenu` — bornes exactes (exercice, pas de
  lien), partie (exercice, lien), deux exercices (« Plusieurs »), une date hors (« Hors exercice »), les deux hors, début
  après fin (précédent), date vide (précédent), exercices non contigus ; `enTetePeriode` — les quatre cas, nom saisi,
  trois exercices (liste). Mutations, chacune tuée par un test nommé au Change Log : comparaison au nom par défaut
  retirée (toujours les années) ; doublon ajouté à toutes les entrées ; `<` au lieu de `≤` aux bornes ; début après fin
  qui rend « Hors exercice ».
- **Vitest `frontend/src/lib/features/reports/ReportSelector.test.ts`** (neuf, D15) : `<label for>` et nom accessible
  « Exercice », aucun `aria-label` ; classes de graisse ; choix d'un exercice → « Du » et « Au » aux bornes ; date changée à
  la main → menu sur l'exercice, lien « Exercice entier » présent, clic → bornes, lien absent ; deux exercices → option
  « Plusieurs » sélectionnée et `disabled`, puis choix d'un exercice → option retirée ; « Hors exercice » de même ;
  `unSeulExercice` vrai → *Générer* désactivé et message, faux → *Générer* actif.
- **Vitest de la page** (`reports-page-url.test.ts`, D15) : ouverture → dates aux bornes du premier exercice ; date changée
  après *Générer* → rapport grisé, message, exports désactivés, 0 requête ; « Exercice entier » qui rend la clé affichée →
  grisé retiré, 0 requête ; *Générer* → grisé retiré ; P17 amendé — changement d'exercice par le menu : rapport classique
  gardé et grisé (mutation : vidage remis → test rouge), génération en vol périmée, rapport par compte analytique vidé ;
  en-tête de la réponse affichée (et non des champs).
- **E2E Playwright `frontend/tests/e2e/reports-navigation.spec.ts`** (neuf) : AC-1 (`page.goBack()`, `page.goForward()`,
  requêtes comptées par `page.on('request')`), AC-2, AC-3, AC-4 (`browser.newContext({ storageState })` réduit aux cookies,
  et `page.reload()`), AC-5 (deux valeurs invalides), AC-10 (`window.scrollY`) ; **D15** (deux exercices créés par l'API,
  modèle `api-fixtures.ts:445`) : `getByLabel('Exercice')`, choix → dates, « Du » changé → « Exercice entier », dates à
  cheval → « Plusieurs » et *Générer* désactivé sur le Bilan, rapport grisé après un changement de date, en-tête ; axe à
  zéro violation sur ces états.
- **Preuve du rouge** : l'E2E AC-1 est lancé **avant** le correctif (journal `kesh-gate-logs/s55-1a/e2e-rouge-avant.log`) et
  rougit sur « Bilan affiché après Retour » ; les Vitest neufs de la page rougissent sans le correctif (journal). Mutations,
  chacune tuée par un test nommé au Change Log : *Générer* qui empile ; clé écrite non enregistrée (l'effet réapplique l'URL du clic : un champ modifié sans *Générer* est remis à la
  valeur de la clé — test « champ gardé après un clic d'onglet ») ; retrait du désarmement de P17 ; clé en vol
  ignorée au point 3 ; champ absent conservé au lieu du défaut ; `errorMsg` non effacé ; `applyUrl` qui ne pose pas
  `lastHandledKey` ; requête partie pendant la génération d'un autre onglet (AC-8 b) ; péremption omise au changement d'exercice (AC-8 b') ;
  péremption omise vers la clé affichée (AC-8 c) ; écriture de l'URL dans `generate()` ; dans
  `reports-url.ts`, retrait de la validation de `fy`.

## Recette visuelle

Captures (thème clair, base de démonstration, `kesh-gate-logs/s55-1a/`) : Bilan avant le lien ; grand livre après le lien ;
Bilan après *Retour* (barre d'adresse visible) ; lien collé dans un nouveau contexte ; URL normalisée après une valeur
invalide ; **D15** : menu fermé (année civile et exercice décalé), menu ouvert, période partielle avec « Exercice entier »,
« Plusieurs » (*Générer* désactivé et message), « Hors exercice », rapport grisé « Paramètres modifiés », doublon et nom
saisi, chaque en-tête. Référence de rendu : la démonstration hors du dépôt,
`kesh-gate-logs/s55-selecteur/selecteur-exercice-demo.html`. Jointes au Dev Agent Record et au prompt de la revue de code.

## Tâches

- [ ] **T0** — Reprendre la PR 46 dans `epic/55-navigation` (`git merge`, conflits résolus en gardant les deux côtés) ;
      relever les lignes de la page et du manuel à cette base ; vérifier qu'aucun filtre de devise n'existe.
- [ ] **T1** — E2E AC-1 rouge (journal).
- [ ] **T2** — `reports-url.ts` et ses Vitest ; `ledgerHref` réécrite dessus.
- [ ] **T3** — Page : écriture de l'URL par les gestionnaires (`lastHandledKey` posée à chaque `goto`) ; `applyUrl` et son
      `$effect` (remplacent `onMount` `:546` et l'effet `:567`) ; clés affichée et en vol par onglet ; P17 désarmée pour la
      seule période restaurée ; l'effet du compte analytique **jamais** désarmé.
- [ ] **T3ter** — Sélecteur d'exercice (D15, AC-14 à AC-19) : `fiscal-year-menu.ts` et ses Vitest d'abord (rouges) ;
      `ReportSelector` (libellé, menu, « Du »/« Au », « Exercice entier », *snippet* des contrôles propres) ; P17 amendé ;
      grisé et exports ; `ReportPeriodHeading` dans les six vues ; dix clés i18n ; le bandeau des rapports par compte
      analytique passe à `ReportSelector`. Le Grand livre est branché par la 55-2 (AC-11).
- [ ] **T4** — Suites existantes : Vitest `reports-page.test.ts`, `reports-page-groupes.test.ts` ; E2E `reports.spec.ts`,
      `reports-groupes*.spec.ts`, `reports-currency.spec.ts`, `reports-export-pdf.spec.ts`, `dunning-roundtrip.spec.ts`,
      `open-items.spec.ts`, `homepage-charts.spec.ts`, `invoice-write-off.spec.ts`, `opening-balances.spec.ts` ;
      adaptations nommées (AC-6).
- [ ] **T5** — Manuel, captures.
- [ ] **T6** — Gates (ci-dessous).

## Gates

`cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build` ; E2E : la spec neuve et
les specs de T4 (les specs des Rapports et de la navigation) ; suite complète et `scripts/gate-ci.sh` en fin d'epic, story **purement écran** (fichier d'epic, § *Gates*,
*Dérogation*, Q6 tranchée par le mainteneur le 2026-11-25). Aucun gate
backend, sauf : **À la clôture de la story**, comme elle modifie le manuel (et `messages.ftl` si un texte apparaît), aussi `cargo nextest run -p kesh-i18n` et
`cargo nextest run -p kesh-api -E 'binary(textes_coherents)'` (resserrement de la dérogation, décision du mainteneur du
2026-11-26, fichier d'epic § *Dérogation*).

## Note de version

*Rapports* : *Retour* et *Suivant* du navigateur ramènent au rapport précédent, avec son exercice, sa période et son
compte ; chaque clic sur un onglet est une étape de l'historique ; l'adresse d'une vue se copie et rouvre la même vue pour
un utilisateur connecté, sans cliquer sur *Générer* (comme les liens de l'échéancier et des rappels vers la balance âgée,
qui l'affichent désormais à l'arrivée) ; le lien *Rapports* du menu ramène au Bilan. Le sélecteur d'exercice est le même
sur tous les onglets : il montre l'exercice seul, en gras (ses années, ou son nom s'il a été renommé) ; « Du » et « Au » sont
toujours remplis, « Exercice entier » remet les bornes, « Plusieurs » ou « Hors exercice » s'affichent quand les dates
sortent d'un exercice ; un rapport dont les paramètres ont changé est grisé jusqu'à *Générer*, et son en-tête dit la
période réelle. (refs #À-VENIR)

## Dev Notes

### Pièges

- **Effet P17** : voir *Appliquer une URL*, point 2 ; l'effet du compte analytique n'est jamais désarmé.
- **Normalisation au montage** : un `goto` de normalisation à l'arrivée interrompt la navigation d'entrée de SvelteKit
  (`client.js:1727`, `:1988`) — défilement et focus non remis ; sans effet aujourd'hui (les liens internes sont canoniques),
  à garder en tête.
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
- **C-55-1-6** — Arriver par une adresse pendant le chargement d'un autre rapport ne demande rien (validation P5 ; la P4
  avait corrigé la prémisse fausse « *Générer* interrompt » — le bouton est désactivé pendant `loading` — par une demande
  différée, retirée en P5) : l'onglet montre son invite, comme si l'on y arrivait par un clic.
- **C-55-1-7** — Rapports par compte analytique hors de l'URL (validation P4, V4-1), issue à ouvrir.
- **C-55-1-8** (D15) — Sur un onglet à un seul exercice, « Plusieurs » et « Hors exercice » désactivent *Générer* : l'API
  exige un exercice qui contienne la période ; la décision ne le dit pas, c'est la seule issue sans changement d'API.
- **C-55-1-9** (D15) — Un menu natif entièrement en gras plutôt que `components/ui/select` : le menu ne montre plus les
  dates, le gras partiel n'a plus d'objet ; le natif garde le clavier, le lecteur d'écran et les E2E d'aujourd'hui.

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
- (LOW, `bug` P3, préexistant ; **réduit par D15** : exports désactivés tant que le rapport est grisé, AC-16 — à
  reformuler ou retirer au T0) L'export d'un rapport classique reprend la période des champs, et le nom de fichier celle
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
- 2026-11-25 — Validation P5 (Opus) : 0/0/5/9 (le HIGH a disparu ; deux MEDIUM nés de la demande différée de la P4).
  Remédiation, la dernière permise à l'orchestrateur avant la règle de la sixième passe, **par retrait** de la demande
  différée : une adresse qui arrive pendant la génération d'un autre onglet ne demande rien (V5-1, C-55-1-6) ; changement
  d'exercice par l'URL : vidage et péremption comme P17 (V5-2, AC-8 b') ; AC-9 juste pour les rapports par compte analytique
  (V5-3) ; T3 sans le désarmement de l'effet du compte analytique (V5-4) ; fichier d'epic amendé (D1, *Taille*) (V5-5) ;
  cas « clé affichée, génération de l'onglet en vol » écrit au point 3 (repris de la P3 de la 55-1b, AC-8 c) ; LOW V5-6 à
  V5-14. **Dès la P6, plus aucune remédiation de l'orchestrateur** : retrait ou renvoi au développement.
- 2026-11-25 — Validation P6 ciblée (Sonnet) : 0/0/2/6. **Arrêt de la boucle par renvoi au développement** (règle de la
  sixième passe) : V6-1 et V6-2 en R-1 et R-2, avec leur résolution par défaut ; LOW listés. Tendance : P1 Opus 0/1/10/8 →
  P2 Sonnet 0/0/5/7 → P3 Opus 0/1/6/6 (découpage) → P4 Sonnet 0/1/4/8 (réduction) → P5 Opus 0/0/5/9 (retrait) → P6 ciblée
  Sonnet 0/0/2/6. `ready-for-dev` **sous réserve** du T0 (R-1, R-2).
- 2026-11-25 — **Ajout** sur demande de l'utilisateur : AC-13 (pas de second « Générer », un seul appel par jeu de
  paramètres ; tranche R-2 et remplace C-55-1-6) et AC-14 (exercice en gras dans le sélecteur du haut). Statut repassé à
  `drafted`, validation reprise (P7, ciblée sur l'ajout).
- 2026-11-25 — Validation P7 ciblée sur l'ajout (Opus) : 0/0/4/6. **Arrêt par renvoi au développement** (règle de la sixième
  passe ; la P8, dernière permise, est gardée en réserve) : R-3 (l'AC-13 prime, sites à récrire), R-4 (portée de « pas de
  requête en double », à confirmer avec l'utilisateur), R-5 (portée du gras) ; LOW listés. `ready-for-dev` **sous réserve**
  du T0 (R-1 à R-5).
- 2026-11-25 — **Réponses de l'utilisateur versées** (partie A, points 1 et 2) : l'AC-13 est récrit — revenir à un onglet
  déjà généré montre déjà son rapport aujourd'hui ; ce qui reste à faire est qu'une adresse collée, rechargée ou atteinte
  par *Retour* ou *Suivant* affiche son rapport sans *Générer* (D3) ; aucune régénération au changement d'onglet ; les
  tests du clic sur un onglet deviennent des tests de non-régression. **R-4 tranché** : « pas de requête en double » reste
  une exigence technique seulement. Validation P8 ciblée sur ce changement (dernière passe permise).
- 2026-11-25 — Validation P8 ciblée sur les réponses de l'utilisateur (Sonnet) : 0/0/0/5 — **validation terminée** (dernière
  passe permise, aucune remédiation : les LOW V8-1 et V8-3 se reprennent au T0, ligne lue citée au Change Log). Tendance :
  P1 Opus 0/1/10/8 → P2 Sonnet 0/0/5/7 → P3 Opus 0/1/6/6 → P4 Sonnet 0/1/4/8 → P5 Opus 0/0/5/9 → P6 ciblée Sonnet 0/0/2/6 →
  P7 ciblée Opus 0/0/4/6 → P8 ciblée Sonnet 0/0/0/5. `ready-for-dev` sous réserve du T0 (R-1 à R-3, R-5 ; R-4 tranché).
- 2026-11-25 — **Décision du mainteneur versée** (Q6, dérogation partielle) : § *Gates* aligné sur la *Dérogation* du
  fichier d'epic. **Sans passe** : le plafond de huit passes est atteint (P8) ; alignement mécanique sur une décision, à
  recontrôler au **T0** (R-6 : la phrase du § *Gates* contre le § *Dérogation* du fichier d'epic, ligne lue citée au Change
  Log).
- 2026-11-26 — **Décision du mainteneur du 2026-11-26 versée** (resserrement de la dérogation Q6, fichier d'epic
  § *Dérogation*) : § *Gates* — à la clôture, `nextest -p kesh-i18n` et `binary(textes_coherents)`, la story modifiant le
  manuel. Aucune passe (plafond de huit atteint) : la phrase se recontrôle au **T0** contre le § *Dérogation*, ligne lue
  citée (avec R-6).
- 2026-11-26 — **Précision d'interface de l'utilisateur versée** (transmise par le coordinateur) : AC-14 récrit — le
  **nom** de l'exercice seul en gras, les dates en graisse normale ; un `<select>` natif ne le permet pas, d'où le
  composant `components/ui/select` ; constat Vitest à mutation et capture ; R-5 précisé. Maquette `m2` corrigée hors du
  dépôt (`kesh-gate-logs/s55-maquettes/`, nom seul en gras ; ancienne version gardée en `mocks-src.avant-2026-11-26.html`).
  La fiche est au plafond de huit passes : la passe ciblée du 2026-11-26 (précisions d'interface) la lit, ses points vont
  au **T0**.
- 2026-11-26 — Passe ciblée sur les précisions d'interface (Opus, `kesh-gate-logs/s55-decisions-2/interface-2026-11-26-validation.md`), hors décompte (fiche au plafond) :
  0/0/1/4. Renvoi au **T0**, ligne lue citée : **R-7 (UI-9, MEDIUM)** — aucun Vitest de Kesh ne pilote aujourd'hui
  `components/ui/select` (pas de polyfill dans `vitest-setup.ts`, `fireEvent.change` ne suffit plus) : écrire d'abord, au
  T0, un test qui pilote le sélecteur sous jsdom (Vitest de la page, `ReportSelector.test.ts` neuf) ; s'il ne tient pas,
  arbitrer **avec l'utilisateur** avant le code (p. ex. `<select>` natif gardé et nom en gras affiché à côté) ; LOW : UI-10
  (`Number('')` vaut `0` : conversion explicite), UI-11 (« seuls `paypal-*` » est faux, 139 lignes de `selectOption`, aucune
  sur `ReportSelector` : la conclusion tient), UI-12 (AC-14 absent des § *Recette visuelle* et *Tests* ; « options en
  graisse normale » sans constat), UI-13 (R-5 parle encore de « classe commune » et de « valeur affichée du `<select>` »).
- 2026-11-26 — **Décision de l'utilisateur et avis UX versés, à confirmer par le mainteneur** (D15, le sélecteur d'exercice
  des Rapports et du Grand livre) : AC-14 récrit et AC-15 à AC-19 ajoutés — libellé « Exercice » unique par `<label for>`,
  `aria-label` retiré ; menu natif entièrement en gras, sans dates ; années si le nom est celui de `nom_par_defaut`, sinon le
  nom saisi, bornes ajoutées aux seuls doublons ; dates toujours remplies, choix d'un exercice → bornes ; menu déduit des
  dates (exercice, même partiel, et « Exercice entier » ; « Plusieurs » et « Hors exercice » désactivées) ; même
  `ReportSelector` sur tous les onglets à période, « Du » / « Au » ; rapport périmé grisé, exports désactivés ; en-tête de la
  période réelle ; dix clés i18n (AC-18 remplace l'AC-12) ; manuel (AC-19) ; P17 amendé ; C-55-1-8, C-55-1-9 ; modules
  portés à cinq. **Retirés** : l'ancien AC-14 (nom seul en gras par `components/ui/select`), R-5 (portée du gras), R-7
  (piloter `ui/select` sous jsdom), l'AC-12 « aucun texte ». Passe de validation ciblée unique (Sonnet, prompt
  `55-1a-55-2-epic-validation-prompt-2026-11-26-selecteur.md`), hors décompte (fiche au plafond) ; ses MEDIUM vont au T0
  (R-8 et suivants).
- 2026-11-26 — Passe ciblée D15 (Sonnet, `kesh-gate-logs/s55-selecteur/selecteur-2026-11-26-validation.md`), hors décompte : 0/0/5/3 (SE-1, SE-2, SE-3, SE-5, SE-6 MEDIUM ; SE-4,
  SE-7, SE-8 LOW). **Aucune remédiation** (fiche au plafond) : renvoi au **T0** en R-9 à R-13, LOW listés avec eux.
