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
    périme la génération en vol (`genSeq`) ;
  - garde de course `genSeq` (`:149`) ; sur la PR 46, `invalidateInFlight()` est le seul endroit qui l'incrémente.
- `frontend/src/lib/features/reports/reports.api.ts:574-582` : `ledgerHref(accountId, from, to)` →
  `/reports?tab=general-ledger&accountId=…&from=…&to=…`, rendu en `<a href>` (SvelteKit empile l'entrée). Employé aussi par
  `homepage/ExpensePieCard.svelte:64` et `fiscal-years/RevaluationPanel.svelte:357` ; `tests/e2e/homepage-charts.spec.ts:129,
  142,233` vérifie l'URL par une expression régulière **dans cet ordre de paramètres**.
- `frontend/src/routes/(app)/reports/+page.ts` : `load` ne lit pas l'URL (exercices, société) ; il ne se rejoue donc pas
  quand seuls les paramètres de requête changent. **À garder ainsi** (sinon chaque navigation rechargerait les exercices).
- Sur la PR 46 : `reports-grouping.ts` (`GroupingView { shown, detailLevel, subtotalPosition, open, closed }`,
  `sanitizeGroupingView`, `isDetailLevel`, `MAX_DETAIL_LEVEL = 20`, `MAX_MANUAL_GROUPS = 500`, `readGroupingView`,
  `writeGroupingView`, `groupingParamsOf`) ; la page lit la vue mémorisée avant la première requête et la réécrit après
  chaque réponse (`acceptGrouped`).
- Liens entrants : `/reports?tab=aged-receivables` (`invoices/due-dates/+page.svelte:401`, `invoices/reminders/+page.svelte:277`),
  `/reports` (`settings/opening-balances/+page.svelte:691`). E2E qui ouvrent `/reports?…` : `dunning-roundtrip.spec.ts:115`,
  `open-items.spec.ts:357`, `reports.spec.ts:319,423,432,461`, `homepage-charts.spec.ts`.
- **Aucun filtre de devise** n'existe dans les Rapports (`grep -n "currency\|devise"` sur la page, à `main` et sur les trois
  branches de l'Epic 46) : rien à porter dans l'URL à ce titre. Si un filtre existe à la base de départ (T0), il entre dans
  le tableau des paramètres, sinon le Change Log le redit.

**Le défaut, par lecture** : sur le Bilan, un clic sur `1020` empile `/reports?tab=general-ledger&accountId=…` ; l'effet
génère le grand livre. *Retour* ramène l'URL à `?tab=balance-sheet` (ou `/reports`) sans remonter le composant ; l'effet
sort tôt (`tab ≠ general-ledger`), `activeTab` reste `general-ledger`. L'URL a changé, l'écran non ; un second *Retour*
quitte la page. À confirmer par l'E2E écrit rouge d'abord (T1).

## Le modèle (D1 à D5)

**L'URL décrit le dernier rapport demandé.** Une fonction pure lit l'URL en un état, une autre écrit l'état en URL ; la page
applique l'état lu **à chaque changement de `page.url`** (chargement, Retour, Suivant, lien), et n'écrit l'URL qu'aux gestes
ci-dessous.

### Paramètres

| Paramètre | Onglets | Valeur valide | Défaut (absent ou invalide) |
|---|---|---|---|
| `tab` | tous | un des neuf identifiants de `tabs` | `balance-sheet`, aucun rapport demandé |
| `fy` | bilan, compte de résultat, balance, journaux, TVA ; rapports par projet en mode `fiscal_year` | identifiant d'un exercice de `data.fiscalYears` | premier exercice (comportement d'aujourd'hui), rapport **non demandé** |
| `from`, `to` | rapports classiques (période, facultative) ; grand livre (requis) | date ISO `AAAA-MM-JJ` réelle (`2026-02-30` refusée) | vide |
| `accountId` | grand livre | entier > 0 (le compte peut être archivé) | tous les comptes mouvementés |
| `includeZero` | grand livre | `1` | absent (faux) |
| `projectId` | rapports par projet | entier > 0 | aucun |
| `mode` | rapports par projet | `fiscal_year`, `cumulative` | `fiscal_year` |
| `view` | bilan, compte de résultat, balance (Epic 46) | `grouped`, `flat` | vue mémorisée (46), sinon défaut 46 |
| `level` | idem | `1`…`20`, `all` (`isDetailLevel`) | idem |
| `subtotals` | idem | `header`, `footer` | idem |
| `open`, `closed` | idem | entiers > 0 séparés par des virgules, ≤ 500 chacun, sans identifiant commun aux deux | idem (les deux ignorés s'ils partagent un identifiant, comme `sanitizeGroupingView`) |

- Un paramètre qui ne sert pas l'onglet (p. ex. `accountId` sur le bilan) est **ignoré** et retiré à la normalisation.
- **Ordre canonique** : `tab`, `fy`, `accountId`, `from`, `to`, `includeZero`, `projectId`, `mode`, `view`, `level`,
  `subtotals`, `open`, `closed`. Pour le grand livre, il rend **exactement** la sortie d'aujourd'hui de `ledgerHref`
  (`tab`, `accountId`, `from`, `to`) : `ledgerHref` est réécrite sur la fonction d'écriture (DRY) et sa sortie ne change pas.
- Les valeurs par défaut ne s'écrivent pas (`includeZero` absent, `mode=fiscal_year` absent).
- **Requis pour qu'un rapport soit demandé** : classiques — `fy` ; grand livre — `from` et `to` ; projets — `projectId`, plus
  `fy` en mode `fiscal_year` ; balance âgée — rien ; regroupement — jamais requis.

### Gestes (D2)

| Geste | Écriture de l'URL | Génération |
|---|---|---|
| Changement d'onglet (clic, flèches du clavier) | **`pushState`** : l'onglet et l'état en mémoire de **cet onglet** (les rapports classiques partagent `fy`, `from`, `to`) | si l'URL écrite est complète (D3) |
| Clic sur un lien de compte (`<a href={ledgerHref(…)}>`) | **empile** (navigation SvelteKit, inchangée) | oui (complète par construction) |
| *Générer* | **`replaceState`** avec les valeurs du formulaire | oui (comme aujourd'hui) |
| Changement d'exercice, de date, de compte, de compte analytique, de mode, de la case « comptes sans mouvement » | **rien** tant que *Générer* n'est pas cliqué (l'URL reste celle du dernier rapport demandé) | non (comme aujourd'hui) |
| Contrôles de regroupement et accordéons (Epic 46) | **`replaceState`** | oui (l'Epic 46 redemande déjà) |
| Retour, Suivant, chargement, lien collé | — (lecture seule) | si complète : mémoire ou requête (D3) |

`goto` est appelé avec `keepFocus: true, noScroll: true` dans tous les cas ; aucune écriture de l'URL ne se fait si l'URL
canonique est déjà la même (pas d'entrée en double, pas de boucle).

### Afficher depuis l'URL (D3)

- À chaque changement de `page.url` **sur cette page**, l'état est relu ; si l'onglet lu est complet, le rapport s'affiche :
  **repris de la mémoire** si le rapport en mémoire de cet onglet a été produit avec les mêmes paramètres (clé = URL
  canonique de l'onglet), **redemandé** sinon (`generate()`, garde `genSeq` inchangée).
- Incomplet : l'onglet s'affiche avec son invite d'aujourd'hui, les champs remplis de ce que l'URL porte.
- `/reports` sans paramètre : écran d'aujourd'hui, **rien n'est écrit dans l'URL ni demandé**.
- Une écriture de la page (`goto`) relit l'état qu'elle vient d'écrire : l'application doit être **idempotente** (même
  clé → aucun effet), ce qui évite une double requête après *Générer*.

### Valeurs invalides (D5)

Chaque paramètre est jugé seul ; une valeur invalide prend le défaut de son paramètre, et l'URL est **normalisée** par un
`replaceState` (aucune entrée d'historique, aucun message). Un exercice inconnu (supprimé, autre société) est une valeur
invalide. La période hors de l'exercice, `from > to` ou un compte inconnu de la société restent jugés **par l'API**, qui
répond par ses messages d'aujourd'hui (`REPORT_PERIOD_OUT_OF_FISCAL_YEAR`, etc.) : le client ne double pas ses règles.

### Regroupement de l'Epic 46 (D4)

Pour le bilan, le compte de résultat et la balance : les paramètres de regroupement de l'URL **priment** sur la vue
mémorisée ; absents, la vue mémorisée (46, `readGroupingView`) s'applique, et l'URL est **complétée** par `replaceState` à
l'affichage. La validation réutilise `isDetailLevel` et `sanitizeGroupingView` (aucune copie des règles). La mémoire du
navigateur continue de retenir la dernière vue affichée (`acceptGrouped`), y compris celle venue d'un lien. L'export reprend
toujours les paramètres de la dernière réponse affichée (46-2, AC-6), inchangé.

## Critères d'acceptation

1. **AC-1 — Retour après un lien de compte.** Bilan de l'exercice E, période P, affiché ; clic sur un numéro de compte →
   grand livre de ce compte sur P ; *Retour* → **le Bilan de E sur P** s'affiche, sans clic sur *Générer*, avec la même vue
   de regroupement, et **sans nouvelle requête** `GET /api/v1/reports/balance-sheet` (repris de la mémoire) ; *Suivant* →
   le grand livre, sans nouvelle requête. Même chose depuis la Balance et depuis le Compte de résultat groupé.
2. **AC-2 — Retour entre onglets.** Bilan affiché (E) ; clic sur l'onglet *Compte de résultat* → l'URL gagne une entrée
   (`tab=income-statement&fy=E…`), le compte de résultat s'affiche (D3) ; *Retour* → le Bilan, onglet *Bilan* sélectionné
   (`aria-selected`), sans requête ; un second *Retour* quitte la page (l'arrivée `/reports` n'est pas doublée).
3. **AC-3 — *Générer* remplace.** Bilan de E affiché ; période changée, *Générer* → l'URL porte la nouvelle période, **sans
   entrée de plus** (`history.length` inchangé) ; *Retour* quitte la page ou revient à l'entrée d'avant le Bilan.
4. **AC-4 — Lien partagé et rechargement.** Une URL copiée (onglet, exercice, période, regroupement ; ou grand livre :
   compte, dates, comptes sans mouvement ; ou rapport par projet : compte analytique, mode, exercice) ouverte dans un
   nouvel onglet du navigateur ou rechargée affiche **le même rapport, avec les mêmes contrôles remplis**, sans clic.
   Le regroupement de l'URL prime sur la vue mémorisée (D4).
5. **AC-5 — Valeurs invalides.** `tab=xyz`, `fy=999999`, `fy=abc`, `from=2026-02-30`, `level=0`, `level=21`, `open=1,a`,
   `open=5&closed=5`, `accountId=-3`, un paramètre étranger à l'onglet : chacun prend son défaut, l'URL est normalisée sans
   entrée d'historique, aucune erreur n'est affichée, aucune valeur invalide n'est envoyée à l'API. Une période hors
   exercice garde le message de l'API d'aujourd'hui.
6. **AC-6 — Compatibilité.** `ledgerHref(a, f, t)` rend la même chaîne qu'avant (test d'égalité sur trois cas) ; les liens
   de l'accueil et de la réévaluation ouvrent le grand livre comme avant (`homepage-charts.spec.ts` vert sans modification) ;
   `/reports?tab=aged-receivables` (échéancier, rappels) affiche la balance âgée — désormais sans clic (D3) ; les E2E
   existants qui cliquent ensuite sur *Générer* restent verts (adaptés seulement s'ils attendaient l'écran vide, chaque
   adaptation nommée au Change Log, aucune assertion perdue).
7. **AC-7 — Pas de double requête, pas de boucle.** Un *Générer* produit **une** requête ; un changement d'onglet vers un
   onglet complet **une** ; une navigation vers une URL déjà affichée **zéro** ; aucune séquence ne fait d'appel à `goto`
   en boucle (Vitest : nombre d'appels de `goto` et des fonctions d'API compté).
8. **AC-8 — Course.** Retour pendant une génération en vol : la réponse périmée ne s'affiche pas (garde `genSeq`), l'écran
   montre le rapport de l'URL courante.
9. **AC-9 — Changement d'exercice restauré.** Retour vers une URL d'un autre exercice ne déclenche pas l'effet P17 de
   remise à zéro **après** la restauration (la période lue de l'URL n'est pas effacée) : test dédié.
10. **AC-10 — Accessibilité.** Onglets : `aria-selected` et focus suivent l'URL après Retour ; aucun saut de défilement
    (`noScroll`) ; test axe de `reports.spec.ts` à zéro violation.
11. **AC-11 — Manuel.** `docs/manual/fr/user-manual.tex`, § *Rapports comptables* (`\section` à `:4101` à `1e23edb55`) et
    § *Visualisation et exports* (`:4368`) : un paragraphe « Naviguer dans les rapports » — Retour et Suivant, lien à
    copier, ce qui y est retenu (onglet, exercice, période, compte, regroupement), ce qui ne l'est pas (un champ modifié sans
    *Générer*) ; et une phrase pour le changement d'onglet qui affiche le rapport. `.tex` seul, PDF non commité.
12. **AC-12 — Textes.** Aucun texte visible n'est ajouté (prévu) ; s'il en faut un, quatre locales dans un bloc
    `reports-nav-*`, `i18n-keys` (`sitesTotal` recompté par la garde) et `i18n-libelle-en-dur` verts.

## Tests à écrire, et la preuve du rouge

- **Vitest `frontend/src/lib/features/reports/reports-url.test.ts`** (neuf) : lecture et écriture de chaque paramètre ;
  aller-retour `build(parse(u)) === u` sur les URL canoniques ; chaque valeur invalide de l'AC-5 ; ordre canonique ;
  `ledgerHref` égal à l'ancienne sortie (trois cas, dont un compte archivé) ; paramètres de regroupement validés par
  `isDetailLevel`/`sanitizeGroupingView` (> 500 identifiants ignorés).
- **Vitest de la page** (`frontend/src/routes/(app)/reports/reports-page-url.test.ts`, neuf, mocks de `$app/state` et
  `$app/navigation` comme `reports-page.test.ts:22-27`, avec un `page.url` modifiable) : AC-2, AC-3 (option `replaceState`
  passée à `goto`), AC-7 (compteurs), AC-8, AC-9, D4 (URL prime sur la mémoire ; mémoire complète l'URL).
- **E2E Playwright `frontend/tests/e2e/reports-navigation.spec.ts`** (neuf), sur la base du seed CI : AC-1 (Bilan → lien →
  grand livre → `page.goBack()` → Bilan, `page.goForward()` → grand livre ; requêtes comptées par `page.on('request')`),
  AC-2, AC-3, AC-4 (nouvelle page du même contexte sur l'URL copiée, et `page.reload()`), AC-5 (deux valeurs invalides).
- **Preuve du rouge** : l'E2E AC-1 est écrit et lancé **avant** le correctif sur la base (journal
  `kesh-gate-logs/s55-1/e2e-rouge-avant.log`) : il doit rougir sur l'assertion « Bilan affiché après Retour ». Les Vitest
  neufs de la page rougissent sans le correctif (journal). Toute mutation de `reports-url.ts` (inverser `pushState` et
  `replaceState`, retirer la validation de `fy`) fait rougir un test nommé au Change Log.

## Recette visuelle

Captures (thème clair, base de démonstration, `kesh-gate-logs/s55-1/`) : Bilan avant le lien ; grand livre après le lien ;
Bilan après *Retour* (barre d'adresse visible) ; lien collé dans un nouvel onglet ; URL normalisée après une valeur
invalide. Jointes au Dev Agent Record et au prompt de la revue de code.

## Tâches

- [ ] **T0** — Reprendre la PR 46 dans `epic/55-navigation` (`git merge`, conflits résolus en gardant les deux côtés) ;
      relever les lignes de la page à cette base ; vérifier qu'aucun filtre de devise n'existe ; sinon arrêt et Change Log.
- [ ] **T1** — E2E AC-1 rouge (journal).
- [ ] **T2** — `reports-url.ts` (lecture, écriture, canonique) et ses Vitest ; `ledgerHref` réécrite dessus.
- [ ] **T3** — Page : un seul `$effect` sur `page.url` (sous `untrack` pour l'application), remplace `onMount` (`:546`) et
      l'effet du grand livre (`:567`) ; `selectTab` en `pushState` ; *Générer* et regroupement en `replaceState` ; mémoire
      par clé ; effet P17 neutralisé pendant une restauration (AC-9).
- [ ] **T4** — E2E existants : passage sur `reports.spec.ts`, `dunning-roundtrip.spec.ts`, `open-items.spec.ts`,
      `homepage-charts.spec.ts`, `reports-groupes*.spec.ts` (46) ; adaptations nommées.
- [ ] **T5** — Manuel, captures.
- [ ] **T6** — Gates (ci-dessous).

## Gates

`cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build` ; E2E : la spec neuve et
les specs des Rapports pendant la story, la suite complète à la fin de l'epic (règle des gates de l'epic). Aucun gate
backend (aucun fichier Rust) — `cargo fmt --check` seulement si la reprise de la PR 46 apporte du Rust non formaté.

## Note de version

*Rapports* : *Retour* et *Suivant* du navigateur ramènent au rapport précédent, avec son exercice, sa période, son compte
et sa présentation ; l'adresse d'une vue se copie et rouvre la même vue. Changer d'onglet affiche le rapport de l'onglet
pour l'exercice choisi, sans cliquer sur *Générer*. (refs #À-VENIR)

## Dev Notes

### Pièges

- **Effet P17** : restaurer `fy` depuis l'URL déclenche la remise à zéro de la période et des rapports ; poser
  `lastFyId` avec la valeur restaurée dans la même application, ou faire de la restauration le seul chemin qui change `fy`.
- **`goto` relit sa propre écriture** : l'application de l'état doit comparer la clé canonique avant toute action.
- **`load` ne doit pas lire `url`** : sinon SvelteKit le rejoue à chaque changement de paramètre (rechargement des
  exercices et de la société, perte de l'état des `$state` non).
- **Liste des comptes** : chargée une fois (`fetchAccounts(true)`) ; un `accountId` absent de la liste n'est pas invalide
  côté client (compte créé depuis) — c'est l'API qui juge.
- La PR 46 garde l'affichage pendant une redemande (`refreshing`) : une navigation vers un autre onglet pendant ce temps
  suit `invalidateInFlight()`.

### Choix et écarts

- **C-55-1-1** — URL = dernier rapport demandé, non l'état du formulaire : un champ modifié sans *Générer* n'entre pas
  dans l'URL. *Écarté* : écrire chaque saisie en `replaceState` — un lien copié ou un rechargement montrerait un rapport
  que l'écran n'affichait pas.
- **C-55-1-2** — Mémoire des rapports par onglet (une réponse et sa clé), non un cache de toutes les vues : *Retour* plus
  loin qu'une vue redemande à l'API (une trace d'audit de plus, assumée).
- **C-55-1-3** — `goto` plutôt que `pushState`/`replaceState` du routage superficiel de SvelteKit : `page.url` et la
  navigation par `<a>` restent la seule source, comme aujourd'hui.

### References

- `CLAUDE.md` § *Test Locally First* (E2E), § *Recette visuelle*, § *Travailler à plusieurs développeurs*.
- Fiche 46-2 (`pr/46-rapports-groupes:_bmad-output/implementation-artifacts/46-2-groupes-ecrans.md`), AC-5, AC-6, AC-10.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### Issues à ouvrir

(Points de revue non corrigés : sévérité, site, texte proposé ; source de la story de solde 55-4.)

### File List

## Change Log

- 2026-11-25 — Fiche écrite au kickoff de l'Epic 55 (`drafted`).
