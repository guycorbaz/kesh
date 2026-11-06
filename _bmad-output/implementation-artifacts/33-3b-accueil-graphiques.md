# Story 33-3b : les deux graphiques de la page d'accueil, dessinés en SVG dans Kesh, avec le clic vers le détail

Status: ready-for-dev

*Fiche rédigée le 2026-11-06 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`, branche `story/33-0-kickoff`,
numéros de ligne de `4ef19901`. Epic 33, release **0.17.5**, **sans migration**. Registre : `e33-choix-autonomes-e332.md`, choix
`C-e332-k`. Issue **#918** : `closes` dans la PR, une fois les deux stories réunies ; **#165** et **#166** en `refs`, **non fermées**.
**Suit la 33-3a**, dont le contrat fait foi. Modules : `frontend`, `kesh-i18n`, manuel.*

## Story

En tant qu'utilisateur de Kesh, je veux voir sur la page d'accueil les revenus et les dépenses de chaque mois de l'exercice ouvert et
la part des plus gros postes de charges, et passer d'un clic ou au clavier au détail d'un mois ou d'un compte, y compris avec un lecteur
d'écran.

## Ce que le code dit (4ef19901)

1. **La page.** `frontend/src/routes/(app)/+page.svelte` charge chaque source par sa fonction, avec un état `'loading' | 'ready' | 'error'`
   (`:20`, `:37-76`), et confie le rendu aux tuiles de `features/homepage/` (`:15-18`), dans une grille `lg:grid-cols-3` (`:92`). Un
   échec **se dit** : « prétendre qu'il n'y a rien est le défaut même de #388 » (`RecentEntriesCard.svelte:2-4`, `:28-31`).
   `homepage-page.test.ts` moque toutes les sources (`:7-27`) : une source de plus y entre.
2. **Le clic vers un mois.** `journal-entries/+page.svelte:143` lit `parseQueryFromUrl`, qui accepte `dateFrom` et `dateTo`
   (`features/journal-entries/query-helpers.ts:87-91`). Pas de filtre de compte requis. Tous rôles : la tuile lit déjà cette route.
3. **Le clic vers un poste.** `ledgerHref(accountId, from, to)` (`features/reports/reports.api.ts:473-481`) rend
   `/reports?tab=general-ledger&accountId=…&from=…&to=…`, et la page génère d'office à l'arrivée (`reports/+page.svelte:565-578`).
4. **Le lien profond des rapports ne connaît que le grand livre** (`:569`, `tab !== 'general-ledger'` → sortie) : « Autres », qui n'a pas de
   compte unique, n'a pas de destination existante (C-e332-18).
5. **Formats.** `formatSwissAmount(Big, decimals = 2)` (`features/journal-entries/balance.ts:129-139`) et `formatSwissDate`
   (`reports.api.ts:238`). « Tout montant reste une chaîne décimale jusqu'ici et passe par `big.js` : jamais de `Number` sur un montant »
   (`homepage.ts:3-5`). `i18nLocale()` donne la locale de l'installation, ⚠️ jamais `navigator.language`
   (`shared/utils/i18n.svelte.ts`) : elle sert au nom court des mois par `Intl.DateTimeFormat`.
6. **Pas de bibliothèque de graphiques** : décision de Guy, et aucune n'est en dépendance à ajouter. Pas de thème sombre (`CLAUDE.md`,
   *Recette visuelle*). Jetons : `--color-primary #1e40af`, `--color-warning #d97706`, `--color-text-muted #64748b`, `--color-border
   #e2e8f0` (`frontend/src/app.css:12-33`).
7. **L'accessibilité se contrôle déjà.** `homepage-dashboard.spec.ts:94-103` passe axe-core sur l'accueil peuplé : à étendre.
8. **Le montage E2E.** `seedTestState('with-company')` sème une société, un exercice et cinq comptes (`crates/kesh-api/src/routes/test_endpoints.rs:78-82`,
   `:200` ; à l'étape 0 : vérifier que l'exercice couvre la date du jour et qu'un des cinq comptes est une charge, sinon en créer un par l'API dans la spec) ;
   `with-company-no-fy` sans exercice (`:88-91`, `:225`) ; `createJournalEntryViaApi` passe au **Zurich**, débit `10xx`, crédit
   `3xxx` (`helpers/api-fixtures.ts:169-183`) : il faut une variante pour les charges. Sélecteurs `data-testid` seulement (garde
   `e2e-selecteurs-traduits`, #326).
9. **Le manuel** : `user-manual.tex:231-232`, « Il présente trois tuiles de synthèse ». Faux dès cette story.
10. **#165 et #166** : voir § *Ce qui reste de #165 et #166*.

## Décisions

Décisions de Guy (#918, epic) : barres mensuelles revenus / dépenses sur douze mois, sans ligne de résultat ni cumul ; camembert à cinq
parts plus « Autres » ; clic vers le détail ; SVG sans bibliothèque ; exercice ouvert même vide. Au registre :

- **C-e332-16** : les deux `<svg>` sont des images (`role="img"`, titre et description) ; **les données et les liens vivent dans un vrai
  tableau HTML**. Les `<a>` du SVG sont réservés à la souris (`tabindex="-1"`), le clavier et le lecteur d'écran passent par le tableau.
- **C-e332-17** : couleur jamais seule : trames SVG (`<pattern>`) et position ; les parts du camembert se numérotent par leur rang.
- **C-e332-18** : « Autres » n'est pas cliquable.
- **C-e332-19** : barres négatives dessinées sous l'axe ; l'échelle se calcule sur la valeur absolue la plus grande des deux signes.
- **C-e332-20** : le lien d'un poste couvre l'exercice entier ; celui d'un mois, les bornes `from`/`to` que rend l'API.

## Critères d'acceptation

### Barres mensuelles

- **AC1** — Pour chaque élément de `months`, deux barres : revenus, dépenses. Aucune ligne de résultat, aucun cumul, aucun solde.
- **AC2** — Exercice de 12 mois : 12 groupes ; le dessin tient aussi pour 13 et 18 mois (`viewBox`, largeur calculée), sans chevauchement
  d'étiquettes (une étiquette sur deux au-delà de 12 si la largeur l'exige, le tableau les portant toutes).
- **AC3** — Les hauteurs sont proportionnelles, calculées par `big.js` jusqu'au ratio, `Number` seulement sur le ratio géométrique. Une
  valeur négative part sous l'axe du zéro, qui est dessiné ; un jeu tout à zéro ne divise pas par zéro et ne dessine aucune barre.
- **AC4** — Mois nom court, locale de `i18nLocale()`, `timeZone: 'UTC'` pour ne pas glisser d'un mois. Graduations en format suisse
  (`formatSwissAmount`).
- **AC5** — Clic sur un mois (toute la hauteur du groupe) → `/journal-entries?dateFrom=<from>&dateTo=<to>`, bornes rendues par l'API.

### Camembert

- **AC6** — Une part par `expensePosts`, dans l'ordre, sens horaire depuis midi, puis `otherExpenses` s'il existe. Angles depuis `amount /
  expensePostsBase`, pas depuis `share`, arrondi.
- **AC7** — **Une part à 100 %** se dessine (un arc de 360° ne se dessine pas : `<circle>` dans ce cas). Une part minuscule reste au
  tableau, même invisible au dessin.
- **AC8** — Clic sur une part de compte → `ledgerHref(accountId, fiscalYear.startDate, fiscalYear.endDate)`. « Autres » : aucun lien
  (C-e332-18), au dessin comme au tableau.
- **AC9** — Montant et pourcentage lisibles pour chaque part, dans le tableau. `share` s'affiche tel que rendu, suivi de `%`.
- **AC10** — `nonPositiveExpenseAccounts > 0` : une note sous le tableau, avec le nombre : « N comptes de charges au solde créditeur ou nul
  ne figurent pas dans ce diagramme. » Absente à zéro.

### Accessibilité

- **AC11** — Chaque `<svg>` : `role="img"`, `aria-labelledby` vers un `<title>` et un `<desc>` qui résument le contenu avec des chiffres
  (exercice, totaux, nombre de postes, poste le plus gros), i18n.
- **AC12** — Chaque graphique a son **tableau HTML** (`<caption>`, `<th scope>`) : mois, revenus, dépenses, avec un vrai lien sur le mois ;
  rang, compte, montant, part, avec un vrai lien sur le compte. Le tableau des mois est dans un `<details>` ouvrable au clavier ; celui des
  postes tient lieu de légende et reste visible.
- **AC13** — Tout se fait au clavier : `Tab` atteint le `<summary>` et chaque lien du tableau, jamais un `<a>` du SVG ; focus visible
  (`outline` de `app.css:71`). Test Playwright au clavier, pas seulement au clic.
- **AC14** — La couleur n'est jamais seule : revenus pleins, dépenses tramées ; six trames ou motifs distincts au camembert, numéro de rang
  dans la légende. Contraste de chaque remplissage ≥ 3:1 sur blanc, vérifié par un test pur sur la palette (ratio WCAG calculé), « Autres »
  en gris.
- **AC15** — axe-core sans violation sur l'accueil **peuplé avec les graphiques**, et sur l'accueil sans exercice.

### États

- **AC16** — Chargement : un état `loading` qui dit qu'il charge (`aria-busy`), sans barre fantôme.
- **AC17** — Échec de l'API : le message `homepage-charts-unavailable`, **jamais** un graphique vide (#388).
- **AC18** — `state = "noOpenFiscalYear"` : phrase dédiée qui dit qu'aucun exercice n'est ouvert à la date du jour, à la place des deux
  graphiques, et aucun lien vers un autre exercice. **Trois** chemins y mènent (33-3a, AC2), une seule phrase.
- **AC19** — Exercice ouvert et vide : le cadre des barres avec ses mois et le message « Aucun produit ni aucune charge dans cet exercice »
  ; à la place du camembert, le même état, sans cercle plein trompeur. Des charges toutes créditrices : la note d'AC10, pas de camembert.

### Formats et langues

- **AC20** — Tout montant par `formatSwissAmount` (apostrophe typographique, point décimal), toute date par `formatSwissDate`. Graduations
  sans décimales seulement si la valeur est ronde.
- **AC21** — Toute chaîne visible par une clé `homepage-charts-*`, dans les quatre `messages.ftl` ; `npm run lint-i18n-ownership` vert.
  Aucun libellé traduit dans un sélecteur E2E.

### Rôles

- **AC22** — `Consultation` voit les mêmes graphiques et les mêmes liens (les trois tuiles le font, `user-manual.tex`, *Un rôle Consultation
  voit les mêmes chiffres*).

## Tâches

### T0 — Étape 0

Lire le contrat réel de la 33-3a sur le dernier `HEAD` et les prémisses ci-dessus, lignes citées au Change Log. Charger la skill `dataviz`
pour la palette et les marques avant la première ligne de dessin, sous réserve d'AC14 et des jetons de Kesh, qui priment.

### T1 — Tests rouges

- `features/homepage/homepage-charts.test.ts` (purs) : `monthlyBars` (positifs, négatifs, tout à zéro, 13 mois) ; `pieSlices` (trois parts,
  une part à 100 %, part minuscule) ; `monthHref`, `monthLabel` (janvier en `timeZone` UTC) ; contraste de la palette.
- `features/homepage/HomepageCharts.test.ts` : chaque AC d'état, de tableau, de lien, d'accessibilité statique (`role`, `aria-labelledby`,
  `tabindex="-1"`, `<caption>`), « Autres » sans lien, note d'AC10.
- `routes/(app)/homepage-page.test.ts` : la source neuve moquée ; son rejet mène à l'état d'échec, sans masquer les tuiles.
- Rouges parce que les modules n'existent pas ; **chaque rouge est vu avant d'écrire le code**.

### T2 — Le calcul pur (`features/homepage/homepage-charts.ts`)

Sur le modèle de `homepage.ts` : chaînes décimales, `Big`, retour de géométrie en nombres. `pieSlices` traite le cas 100 %. Aucune
dépendance ajoutée à `package.json` (AC : `git diff` de `package.json` et `package-lock.json` vide).

### T3 — L'API et les composants

- `features/homepage/homepage-summary.api.ts` et son type, miroir du contrat de la 33-3a. `state` est une union, `fiscalYear` nul seulement
  avec `noOpenFiscalYear`.
- `MonthlyBarsCard.svelte`, `ExpensePieCard.svelte`, mêmes classes que les tuiles (`rounded-lg border border-border bg-white p-6 shadow-sm`),
  `data-testid` : `homepage-card-monthly-bars`, `homepage-card-expense-pie`, `homepage-chart-month-link`, `homepage-chart-post-link`,
  `homepage-charts-no-open-year`, `homepage-charts-unavailable`, `homepage-charts-empty`.
- `+page.svelte` : `loadSummary`, nouvel état, section `lg:grid-cols-2` **sous** la grille des tuiles.
- Clés i18n, quatre locales.

### T4 — E2E (`frontend/tests/e2e/homepage-charts.spec.ts`)

Une charge et un produit passés par l'API avec des montants marqués (`uniq`), aide `createExpenseEntryViaApi` à côté de
`createJournalEntryViaApi`, même date locale :

1. les valeurs traversent : le tableau affiche le montant au bon mois et au bon poste, lu par `data-*` ;
2. clic sur un mois → URL à `dateFrom` / `dateTo`, l'écriture y figure ;
3. clic sur une part → grand livre du compte généré ;
4. **au clavier** : `Tab` jusqu'au lien du poste, `Enter`, même destination ;
5. contre-passation d'une écriture de charge dans l'exercice : le poste et le mois reviennent à leur valeur ;
6. rôle `Consultation` : mêmes graphiques ;
7. `seedTestState('with-company-no-fy')` : `homepage-charts-no-open-year` ;
8. axe-core dans les deux états (AC15).

### T5 — Captures

Base de démonstration jetable, thème clair, `/home/gcorbaz/devel/kesh-gate-logs/e333b/captures/` : peuplé (12 mois, six parts), mois négatif
(un avoir seul), une part à 100 %, vide, aucun exercice, chargement, échec, tableau des mois ouvert, focus clavier ; et un en `de-CH`.
Jointes au Dev Agent Record et au prompt de revue. **Regardées**, pas seulement produites : un test dit qu'un élément existe, non qu'on le voit
(#640).

### T6 — Manuel et CHANGELOG

- `docs/manual/fr/user-manual.tex`, § *Tableau de bord* (`:229-`) : « trois tuiles » récrit ; `\subsection{Revenus, dépenses et principaux postes}`
  qui dit, en phrases de comptable : d'où viennent les chiffres (produits et charges du journal, non les flux bancaires) ; quel exercice, et
  « aucun exercice ouvert » ; **ce qui est écarté et ce qui ne l'est pas** : une écriture et sa contre-passation du même exercice disparaissent
  ensemble ; une contre-passation qui franchit l'exercice reste, comme un avoir, en montant négatif ; le total égale le compte de résultat ;
  Kesh ne passe pas d'écriture de clôture ; pourquoi le grand livre d'un poste ou les écritures d'un mois peuvent montrer des lignes que le graphique
  n'a pas retenues ; ce qu'est « Autres » et les comptes au solde créditeur ; le clavier.
- `make fr` dans `docs/manual/` ; PDF aplati (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`), phrases cherchées citées au Change Log.
- `README.md`, § *Fonctionnalités*, et `website/index.html` si le tableau de bord y figure : relire, une ligne au besoin.
- `CHANGELOG.md`, `## [0.17.5] — Non publié`, *Ajouté* : les deux graphiques, `refs` #918, #165, #166, avec ce qui est écarté.

## Ce qui reste de #165 et #166

Les deux restent ouvertes (P4, `v0.4-milestone`) ; l'orchestrateur le commente sur chacune :

- **#165 (donut)** : livré ici : répartition des charges par compte, regroupement « Autres », montant et part. **Reste** : le choix des comptes par
  l'utilisateur et sa persistance ; la descente par position, compte et sous-compte le long de `accounts.parent_id` (précision du 2026-06-29) ;
  le choix de la période ; le total au centre de l'anneau ; la mutualisation avec #164.
- **#166 (barres)** : livré ici : barres mensuelles groupées revenus / charges sur l'exercice ouvert. **Reste** : le choix des comptes et sa
  persistance ; la période glissante réglable ; le solde net visible, que Guy a écarté pour #918 mais que #166 demandait.

## Mutations (`kesh-gate-logs/e333b/vmut.sh`, sous le verrou)

| Id | Mutation | Tuée par |
|---|---|---|
| M1 | cas 100 % retiré de `pieSlices` | test pur, part à 100 % |
| M2 | angles depuis `share` | test pur, trois parts |
| M3 | ratio sans valeur absolue | test pur, négatifs |
| M4 | division par zéro non gardée | test pur, tout à zéro |
| M5 | `timeZone` retiré de `monthLabel` | test pur, 1er janvier |
| M6 | `dateTo` absent de `monthHref` | test pur, E2E 2 |
| M7 | `tabindex="-1"` retiré | composant, axe |
| M8 | état d'échec rendu comme le vide | composant |
| M9 | « Autres » rendu en lien | composant |
| M10 | note d'AC10 retirée | composant |
| M11 | `noOpenFiscalYear` non traité | composant, E2E 7 |
| M12 | une couleur remplacée par un gris à 2:1 | test de contraste |

Une survivante se traite ou se justifie au Change Log.

## Gates

Commun : `e33-commun.md`. Entre passes : `npm run check`, `npm run lint-i18n-ownership`, Vitest ciblé. **Au dernier commit de code** : `npm run
test:unit`, `npm run build`, `scripts/test-fast.sh --ci` (le test `textes_coherents.rs` lit manuels et clés), **E2E complet**, rouges jugés
fichier par fichier contre `docs/testing.md`. Interdit : « rejouée au push ».

## Dev Notes

- **Une fiche n'a pas dessiné le graphique** : les marges, les étiquettes et les trames se règlent aux captures. Ce qui ne se règle pas : AC3, AC6,
  AC7, AC11 à AC14.
- **SVG écrit à la main, sans `{@html}`** : les noms de compte passent par l'échappement de Svelte.
- **`aria-hidden` n'est pas retenu** : l'image a un texte de remplacement. axe tranche ; si `role="img"` et `<a tabindex="-1">` coexistent mal,
  sortir les zones cliquables du `role="img"` en le disant au Change Log, sans retirer le texte de remplacement.
- **Les mois à venir** sont vides parce que l'API rend zéro ; une écriture datée d'un mois futur y apparaît, comme au compte de résultat (33-3a,
  invariant). Pas de masquage côté écran.

## Change Log

- 2026-11-06 — Fiche rédigée en autonomie (Opus 5.5).

## Dev Agent Record

*À remplir au développement.*
