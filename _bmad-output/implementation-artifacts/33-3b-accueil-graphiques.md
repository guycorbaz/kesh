# Story 33-3b : les deux graphiques de la page d'accueil, dessinés en SVG dans Kesh, avec le clic vers le détail

Status: ready-for-dev

*Fiche rédigée le 2026-11-06 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`, branche `story/33-0-kickoff`,
numéros de ligne de `4ef19901`. Epic 33, release **0.17.5**, **sans migration**. Registre : `e33-choix-autonomes-e332.md`, choix
`C-e332-k`. Issue **#918** : `closes` dans la PR, une fois les deux stories réunies.
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
8. **Le montage E2E.** `seedTestState('with-company')` sème un exercice **de 132 mois**, 2020-01-01 → 2030-12-31 (`crates/kesh-db/src/test_fixtures.rs:116-121`), ouvert : avec
   C-e332-14, 132 groupes et 132 lignes de tableau. Comme il couvre tout, un exercice de plus y chevaucherait et serait refusé (`repositories/fiscal_years.rs:346`). `with-company-no-fy` sème la
   société et les comptes, sans exercice (`test_endpoints.rs:88-91`, `:225` ; `test_fixtures.rs:265`, `:907`) : la spec part de là et **crée elle-même** un exercice civil de douze mois, par
   `POST /api/v1/fiscal-years` (`routes/fiscal_years.rs:79-83` : `name`, `startDate`, `endDate`). Aucun helper n'existe : `createFiscalYearViaApi` à écrire. `homepage-dashboard.spec.ts:22-23` reste sur `with-company` : son test axe (`:94-103`) voit donc
   le dessin à 132 mois, ce qui couvre le cas long sans autre montage. `createJournalEntryViaApi` passe au **Zurich**, débit `10xx`, crédit `3xxx`
   (`helpers/api-fixtures.ts:169-183`) : il faut une variante pour les charges (étape 0 : le seed a bien un compte `Expense`). Sélecteurs `data-testid` seulement (garde `e2e-selecteurs-traduits`, #326).
9. **Le manuel** : `user-manual.tex:232-233`, « Il présente trois tuiles de synthèse ». Faux dès cette story. Sa capture (`:235`, `dashboard.png`, `% TODO capture`) n'est pas au dépôt
   (`docs/manual/fr/screenshots/` ne porte que `_placeholder.png`) : il n'y a pas de capture à refaire, seulement une légende à récrire.
10. **axe et les descendants focalisables.** La règle `nested-interactive` s'applique à tout rôle à enfants présentationnels, dont `img` (`axe.js:14357-14362`, `:27383-27388`, axe-core 4.11.2 du dépôt principal), et son contrôle
    `no-focusable-content` rend `false` dès qu'un descendant **de rôle widget** est focalisable, `tabindex="-1"` compris (`axe.js:25720-25757`, `get_role_type_default(child) === 'widget'`, `:25752`). Le rôle implicite d'un `<a>` n'existe qu'avec `href` (`:16037-16039`) : un `<a>` sans `href` ou un `<g tabindex="-1">` n'y sont pas pris, et seul le test composant d'AC11 les attrape. Lu, non joué. Un `<svg role="img">` qui ne contient **aucun** élément focalisable n'y est pas exposé.

## Décisions

Décisions de Guy (#918, epic) : barres mensuelles revenus / dépenses sur douze mois, sans ligne de résultat ni cumul ; camembert à cinq
parts plus « Autres » ; clic vers le détail ; SVG sans bibliothèque ; exercice ouvert même vide. Au registre :

- **C-e332-16** : les deux `<svg>` sont des images (`role="img"`, titre et description) **sans aucun élément focalisable** ; **les données et les seuls liens vivent dans un vrai
  tableau HTML, voisin**. Le clic de souris sur une marque passe par un écouteur unique sur le `<svg>`, qui lit le `data-href` de la marque ; le clavier et le lecteur d'écran passent par le tableau.
- **C-e332-17** : couleur jamais seule : trames SVG (`<pattern>`) et position ; les parts du camembert se numérotent par leur rang.
- **C-e332-18** : « Autres » n'est pas cliquable.
- **C-e332-19** : barres négatives dessinées sous l'axe ; l'échelle se calcule sur la valeur absolue la plus grande des deux signes.
- **C-e332-20** : le lien d'un poste couvre l'exercice entier ; celui d'un mois, les bornes `from`/`to` que rend l'API.
- **C-e332-23** : un exercice long reste dessiné mois par mois, sans défilement ; ses étiquettes se clairsèment par une règle simple, tirée du mois rendu.
- **C-e332-24** : les barres décident sur `months`, le camembert sur `expensePosts`, chacun seul.

## Critères d'acceptation

### Barres mensuelles

- **AC1** — Pour chaque élément de `months`, deux barres : revenus, dépenses. Aucune ligne de résultat, aucun cumul, aucun solde.
- **AC2** — Un groupe par mois rendu, quel qu'en soit le nombre `n` (exercice de 12 mois : 12 groupes), à largeur égale, **sans défilement horizontal** : pas d'`overflow-x`, qui exposerait à
  `scrollable-region-focusable`. `viewBox` de 720 de large jusqu'à 12 mois ; au-delà, 1200 de large et carte sur toute la largeur (`lg:col-span-2`). **Étiquettes, règle simple, tirée du mois rendu et non de l'indice** (`axisLabels(months)`, pure, C-e332-23) :
  `n ≤ 12` : chaque mois, nom court ; `13 ≤ n ≤ 36` : janvier, avril, juillet et octobre, mois et année à deux chiffres ; `n > 36` : chaque janvier, l'année seule. Le tableau porte tous les mois.
- **AC2-bis** — Test pur, pour `n` = 1, 12, 13, 36, 37, 132, avec un départ en janvier et un départ en mars : `monthlyBars` rend `n` groupes, et `axisLabels` rend les mois attendus. Aucune mesure de largeur de texte : la lisibilité se regarde aux captures (T5), où marges et corps se règlent.
  **Limite assumée** : à partir d'une soixantaine de mois (estimation, non jouée), une barre ne fait plus que quelques pixels et sa trame ne se voit plus ; seule la position sépare alors revenus et dépenses (AC14), et le tableau porte les chiffres.
- **AC3** — Les hauteurs sont proportionnelles, calculées par `big.js` jusqu'au ratio, `Number` seulement sur le ratio géométrique. Une
  valeur négative part sous l'axe du zéro, qui est dessiné ; un jeu tout à zéro ne divise pas par zéro et ne dessine aucune barre.
- **AC4** — Mois nom court, locale de `i18nLocale()`, `timeZone: 'UTC'` pour ne pas glisser d'un mois. Graduations en format suisse
  (`formatSwissAmount`).
- **AC5** — Clic de souris sur un mois (toute la hauteur du groupe : un `<rect>` transparent, `pointer-events="all"`, dans le `<g data-href>` de la marque) → `/journal-entries?dateFrom=<from>&dateTo=<to>`, bornes rendues par
  l'API.

### Camembert

- **AC6** — Une part par `expensePosts`, dans l'ordre, sens horaire depuis midi, puis `otherExpenses` s'il existe. Angles depuis `amount /
  expensePostsBase`, pas depuis `share`, arrondi.
- **AC7** — **Une part à 100 %** se dessine (un arc de 360° ne se dessine pas : `<circle>` dans ce cas). Une part minuscule reste au
  tableau, même invisible au dessin.
- **AC8** — Clic de souris sur une part de compte (`<g data-href>`) → `ledgerHref(accountId, fiscalYear.startDate, fiscalYear.endDate)`. « Autres » : aucun lien ni `data-href`
  (C-e332-18), au dessin comme au tableau.
- **AC9** — Montant et pourcentage lisibles pour chaque part, dans le tableau. `share` s'affiche tel que rendu, suivi de `%`.
- **AC10** — `creditBalanceExpenseAccounts > 0` : une note sous le tableau, avec le nombre. **Deux clés, choisies par le composant** (`homepage-charts-credit-balance-note-one`, `-other`), le motif de `supplier-invoices-count-one` / `-other` (`supplier-invoices/+page.svelte:279-280`) : un sélecteur Fluent `{ $count -> … }` sortirait toujours sur sa variante par défaut, le backend pré-résolvant chaque message sans arguments (`kesh-i18n/src/loader.rs:158`).
  « 1 compte de charges au solde créditeur ne figure pas dans ce diagramme. » ; « { $count } comptes de charges au solde créditeur ne figurent pas dans ce diagramme. » Absente à zéro. Un compte à solde nul n'y entre pas (33-3a, AC14).

### Accessibilité

- **AC11** — Chaque `<svg>` : `role="img"`, `aria-labelledby` vers un `<title>` et un `<desc>` qui résument le contenu avec des chiffres
  (exercice, totaux, nombre de postes, poste le plus gros), i18n. **Aucun descendant focalisable** : ni `<a>`, ni `tabindex`, ni `<button>`, vérifié par le test du composant
  (`svg a, svg [tabindex], svg button` ne rend rien).
- **AC12** — Chaque graphique a son **tableau HTML** (`<caption>`, `<th scope>`) : mois, revenus, dépenses, avec un vrai lien sur le mois ;
  rang, compte, montant, part, avec un vrai lien sur le compte. Le tableau des mois est dans un `<details>` ouvrable au clavier ; celui des
  postes tient lieu de légende et reste visible.
- **AC13** — Tout se fait au clavier : `Tab` atteint le `<summary>`, qui s'ouvre par `Enter`, puis chaque lien du tableau des mois, et chaque lien du tableau des postes ; le SVG n'offre aucun arrêt
  de tabulation ; focus visible (`outline` de `app.css:71`). Test Playwright au clavier sur les **deux** chemins, pas seulement au clic.
- **AC14** — La couleur n'est jamais seule : revenus pleins, dépenses tramées ; six trames ou motifs distincts au camembert, numéro de rang
  dans la légende. Contraste de chaque remplissage ≥ 3:1 sur blanc, vérifié par un test pur sur la palette (ratio WCAG calculé ; pour un `<pattern>`, sur la couleur de son trait, sans égard à sa densité), « Autres »
  en gris. La palette et les motifs sont une constante pure : le même test vérifie six motifs distincts, revenus pleins et dépenses tramées.
- **AC15** — axe-core sans violation sur l'accueil **peuplé avec les graphiques**, et sur l'accueil sans exercice.

### États

- **AC16** — Chargement : un état `loading` qui dit qu'il charge (`aria-busy`), sans barre fantôme.
- **AC17** — Échec de l'API : le message `homepage-charts-unavailable`, **jamais** un graphique vide (#388).
- **AC18** — `state = "noOpenFiscalYear"` : phrase dédiée qui dit qu'aucun exercice n'est ouvert à la date du jour, à la place des deux
  graphiques, et aucun lien vers un autre exercice. **Trois** chemins y mènent (33-3a, AC2), une seule phrase.
- **AC19** — **Chaque graphique décide sur sa seule donnée** (C-e332-24). **Barres** : tous les montants de `months` à zéro : le cadre avec ses mois et le message « Aucun mouvement net de produits ou de charges dans cet exercice » (`homepage-charts-empty`), juste pour un exercice vide comme pour un reclassement. **Camembert** : `expensePosts` vide, quelle qu'en soit la raison (exercice vide, produits seuls, charges toutes créditrices ou qui se
  compensent) : le message « Aucun poste de charges à répartir dans cet exercice » (`homepage-charts-no-posts`), sans cercle plein trompeur, plus la note d'AC10 s'il y a lieu ; `expensePosts` non vide : le dessin et son tableau (AC6 à AC10), **même si `months` est tout à zéro**. Cas témoin : une seule écriture, débit 6000 de 100, crédit 6100 de 100, deux comptes `Expense` : `months` à zéro, `expensePosts = [6000 : 100, share "100.0"]`, `creditBalanceExpenseAccounts = 1` (33-3a, AC4, AC14) : message `homepage-charts-empty` aux barres, part à 100 %, tableau et note au camembert.

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

**Squelette sous axe, avant le reste du dessin** : sur l'accueil, un `<svg role="img">` à `<title>` et `<desc>` portant une barre et une part en `<g data-href>`, l'écouteur, et le tableau voisin dans son `<details>`. `AxeBuilder` joué, `<details>` fermé
puis ouvert : aucune violation. **Sonde** : le même squelette avec un `<a href="/x" tabindex="-1">` dans le SVG (le `href` est nécessaire, prémisse 10) ; le résultat attendu est `nested-interactive` (prémisse 10), et s'il n'y a aucune violation, c'est que l'analyse ne voit pas le squelette : à élucider avant de continuer. Sorties dans
`kesh-gate-logs/e333b/`, citées au Change Log. Si axe désigne autre chose sur le squelette, le corriger sans remettre d'élément focalisable dans le SVG : la conception ne se rouvre pas.

### T1 — Tests rouges

- `features/homepage/homepage-charts.test.ts` (purs) : `monthlyBars` (positifs, négatifs, tout à zéro) et `axisLabels` (AC2-bis) ; `pieSlices` (trois parts,
  une part à 100 %, part minuscule) ; `monthHref`, `monthLabel` (janvier en `timeZone` UTC) ; palette (AC14 : contraste, six motifs distincts, revenus pleins, dépenses tramées).
- `features/homepage/HomepageCharts.test.ts` : chaque AC d'état, de tableau, de lien, d'accessibilité statique (`role`, `aria-labelledby`, aucun descendant focalisable,
  `<caption>`), clic sur une marque (`goto` moqué, reçoit son `data-href`), « Autres » sans lien ni `data-href`, note d'AC10 **au compte 1 (singulier) et au compte 2 (pluriel)**, et les trois cas d'AC19 : exercice vide ; produits seuls (barres dessinées, `homepage-charts-no-posts`) ; reclassement entre deux comptes de charges (`reclassification_keeps_the_pie` : `homepage-charts-empty` aux barres, part, tableau et note au camembert, pas de `homepage-charts-no-posts`).
- `routes/(app)/homepage-page.test.ts` : la source neuve moquée ; son rejet mène à l'état d'échec, sans masquer les tuiles.
- Rouges parce que les modules n'existent pas ; **chaque rouge est vu avant d'écrire le code**.

### T2 — Le calcul pur (`features/homepage/homepage-charts.ts`)

Sur le modèle de `homepage.ts` : chaînes décimales, `Big`, retour de géométrie en nombres. `pieSlices` traite le cas 100 %. Aucune
dépendance ajoutée à `package.json` (AC : `git diff` de `package.json` et `package-lock.json` vide).

### T3 — L'API et les composants

- `features/homepage/homepage-summary.api.ts` et son type, miroir du contrat de la 33-3a. `state` est une union, `fiscalYear` nul seulement
  avec `noOpenFiscalYear`.
- `MonthlyBarsCard.svelte`, `ExpensePieCard.svelte`, mêmes classes que les tuiles (`rounded-lg border border-border bg-white p-6 shadow-sm`),
  `data-testid` : `homepage-card-monthly-bars`, `homepage-card-expense-pie`, `homepage-chart-month-link`, `homepage-chart-post-link` (liens du tableau),
  `homepage-chart-month-mark`, `homepage-chart-post-mark` (marques du dessin), `homepage-charts-no-open-year`, `homepage-charts-unavailable`, `homepage-charts-empty`, `homepage-charts-no-posts`.
- **Le clic de souris** : un seul écouteur sur le `<svg>`, `event.target.closest('[data-href]')` puis `goto`. Il déclenche des avertissements d'accessibilité de Svelte, levés par un `svelte-ignore` à codes nommés (à relever à l'étape 0), justifié en une ligne
  (« le clavier passe par le tableau »). `npm run check` reste à zéro avertissement. `data-href` n'est jamais construit depuis un texte saisi : identifiants et dates seulement.
- `+page.svelte` : `loadSummary`, nouvel état, section `lg:grid-cols-2` **sous** la grille des tuiles ; la carte des barres passe en `lg:col-span-2` au-delà de 12 mois (AC2).
- Clés i18n, quatre locales.

### T4 — E2E (`frontend/tests/e2e/homepage-charts.spec.ts`)

**Montage** (prémisse 8) : `seedTestState('with-company-no-fy')`, puis `createFiscalYearViaApi` : exercice civil de douze mois de l'année de la date du jour à Zurich, même règle que `createJournalEntryViaApi`, à côté duquel les deux aides s'écrivent. La spec vérifie
d'emblée 12 groupes : ainsi, elle ne teste pas un dessin à 132 mois sans le savoir. ⚠️ La route lit `Utc::now()` (33-3a, Dev Notes) : dans la première ou les deux premières heures du 1er janvier à Zurich, elle lit encore l'an passé et cette spec rougit (`noOpenFiscalYear`). Ce rouge relève de l'issue de la date UTC demandée par la 33-3a, et ne se corrige pas ici. Deux `test.describe` à leur propre `beforeAll` : les scénarios 7 et 8 (sans exercice) d'un côté, les autres de l'autre. Une charge et un produit passés par l'API avec des montants marqués (`uniq`), aide `createExpenseEntryViaApi`, même date locale :

1. les valeurs traversent : le tableau affiche le montant au bon mois et au bon poste, lu par `data-*` ;
2. clic de souris sur la marque d'un mois → URL à `dateFrom` / `dateTo`, l'écriture y figure ;
3. clic de souris sur la marque d'une part → grand livre du compte généré ; si le centre de la boîte ne tombe pas dans la part, `position` calculé dans la spec ;
4. **au clavier, les deux chemins** : (a) `Tab` jusqu'au lien du poste, `Enter`, même destination que 3 ; (b) `Tab` jusqu'au `<summary>`, `Enter` pour l'ouvrir, `Tab` jusqu'au lien du mois, `Enter`, même URL que 2 ;
5. contre-passation d'une écriture de charge dans l'exercice : le poste et le mois reviennent à leur valeur ;
6. rôle `Consultation` : mêmes graphiques ;
7. `seedTestState('with-company-no-fy')` : `homepage-charts-no-open-year` ;
8. axe-core dans les deux états (AC15), `<details>` ouvert à l'état peuplé.

### T5 — Captures

Base de démonstration jetable, thème clair, `/home/gcorbaz/devel/kesh-gate-logs/e333b/captures/` : peuplé (12 mois, six parts), exercices de 36 et de 132 mois, mois négatif
(un avoir seul, donc aussi `homepage-charts-no-posts`), une part à 100 % avec la note d'AC10, vide, aucun exercice, chargement, échec, tableau des mois ouvert, focus clavier ; et un en `de-CH`.
Jointes au Dev Agent Record et au prompt de revue. **Regardées**, pas seulement produites : un test dit qu'un élément existe, non qu'on le voit
(#640).

### T6 — Manuel et CHANGELOG

- `docs/manual/fr/user-manual.tex`, § *Tableau de bord* (`:229-`) : « trois tuiles » récrit (`:232-233`), légende de `:235` récrite ; le `% TODO capture` reste, aucune capture n'étant au dépôt ; `\subsection{Revenus, dépenses et principaux postes}`
  qui dit, en phrases de comptable : d'où viennent les chiffres (produits et charges du journal, non les flux bancaires) ; quel exercice, et
  « aucun exercice ouvert » ; **ce qui est écarté et ce qui ne l'est pas** : une écriture et sa contre-passation du même exercice disparaissent
  ensemble ; une contre-passation qui franchit l'exercice reste, comme un avoir, en montant négatif ; le total égale le compte de résultat ;
  Kesh ne passe pas d'écriture de clôture ; pourquoi le grand livre d'un poste ou les écritures d'un mois peuvent montrer des lignes que le graphique
  n'a pas retenues ; ce qu'est « Autres » et les comptes au solde créditeur ; le clavier, et le fait qu'un clic sur le dessin ne s'ouvre pas dans un autre onglet, le lien du tableau le permettant.
  **Deux limites écrites en toutes lettres** : C-e332-9 : la contre-passation à cheval sur deux exercices reste comptée de part et d'autre ; C-e332-14 : le graphique a autant de mois que
  l'exercice, et non douze, et les étiquettes d'un exercice long se clairsèment, le tableau les portant toutes. (C-e332-4, la troisième limite lue par Guy, se dit au manuel par la 33-2, T5.)
- `make fr` dans `docs/manual/` ; PDF aplati (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`), phrases cherchées citées au Change Log, dont celles de la paire à cheval et de l'exercice long.
- `README.md:240`, ligne « v0.5+ (prévu) » : retirer #165 et #166, fermées et remplacées par #918 ; #164 et les budgets restent. Dans le même commit que les graphiques. § *Fonctionnalités*, et `website/index.html` si le tableau de bord y figure : relire, une
  ligne au besoin.
- `CHANGELOG.md`, `## [0.17.5] — Non publié`, *Ajouté* : les deux graphiques, `refs` #918, avec ce qui est écarté.

## Mutations (`kesh-gate-logs/e333b/vmut.sh`, sous le verrou)

| Id | Mutation | Tuée par |
|---|---|---|
| M1 | cas 100 % retiré de `pieSlices` | test pur, part à 100 % |
| M2 | angles depuis `share` | test pur, trois parts |
| M3 | ratio sans valeur absolue | test pur, négatifs |
| M4 | division par zéro non gardée | test pur, tout à zéro |
| M5 | `timeZone` retiré de `monthLabel` | test pur, 1er janvier |
| M6 | `dateTo` absent de `monthHref` | test pur, E2E 2 |
| M7 | un `<a href>` ajouté autour d'une marque du SVG | composant (AC11) ; axe, selon la sonde de T0 |
| M8 | état d'échec rendu comme le vide | composant |
| M9 | « Autres » rendu en lien, ou portant `data-href` | composant |
| M10 | note d'AC10 retirée | composant |
| M11 | `noOpenFiscalYear` non traité | composant, E2E 7 |
| M12 | une couleur remplacée par un gris à 2:1 | test de contraste |
| M13 | écouteur du `<svg>` retiré | composant, E2E 2 et 3 |
| M14 | `axisLabels` sans seuil au-delà de 12 mois | test pur, AC2-bis |
| M15 | camembert décidé sur `months` et non sur `expensePosts` | composant, `reclassification_keeps_the_pie` (AC19) |
| M16 | clé `-other` employée au compte 1 (AC10) | composant, compte 1 |
| M17 | `axisLabels` sur l'indice (`i % 3`, `i % 12`), sans lire le mois | test pur, départ en mars (AC2-bis) |
| M18 | deux parts de la palette partagent un motif | test pur, palette (AC14) |

Une survivante se traite ou se justifie au Change Log.

## Gates

Commun : `e33-commun.md`. Entre passes : `npm run check`, `npm run lint-i18n-ownership`, Vitest ciblé. **Au dernier commit de code** : `npm run
test:unit`, `npm run build`, `scripts/test-fast.sh --ci` (le test `textes_coherents.rs` lit manuels et clés), **E2E complet**, rouges jugés
fichier par fichier contre `docs/testing.md`. Interdit : « rejouée au push ».

## Dev Notes

- **Une fiche n'a pas dessiné le graphique** : les marges, les étiquettes et les trames se règlent aux captures. Ce qui ne se règle pas : AC3, AC6,
  AC7, AC11 à AC14.
- **SVG écrit à la main, sans `{@html}`** : les noms de compte passent par l'échappement de Svelte.
- **Aucun `<a>` dans le SVG, sans repli** (C-e332-16) : la conception ne dépend pas de la façon dont axe traite `tabindex="-1"`, puisqu'il n'y a rien de focalisable. `aria-hidden` n'est pas retenu : l'image a un texte de remplacement. Ce que cela coûte : pas de
  clic du milieu ni de `Ctrl`+clic sur une marque ; les vrais liens du tableau les permettent.
- **Un exercice long** (C-e332-23) : 132 mois se lisent en barres fines et en repères d'année. Le lecteur a le tableau, fermé par défaut pour ne pas allonger l'accueil. Aucune agrégation par trimestre : elle détacherait les barres des liens de mois.
- **Les mois à venir** sont vides parce que l'API rend zéro ; une écriture datée d'un mois futur y apparaît, comme au compte de résultat (33-3a,
  invariant). Pas de masquage côté écran.

## Change Log

- 2026-11-06 — Fiche rédigée en autonomie (Opus 5.5).
- 2026-11-06 — Remédiation de la validation P1 (rapport `kesh-gate-logs/e33-validate-p1-e332.md`, Opus 5.5, aucune compilation ni aucun test). Constats de cette fiche : R-1, R-2 (MEDIUM), R-8, F-1, F-3 (LOW), plus la suite de R-7 ; tous remédiés. **F-3 réfuté en
  partie** : le rapport craint une capture montrant trois tuiles ; il n'y en a aucune au dépôt (`docs/manual/fr/screenshots/` ne porte que `_placeholder.png`, `user-manual.tex:235` en `% TODO capture`) : seule la légende change. Prémisses relues : `axe.js:14357-14362`, `:25720-25742`, `:27383-27388`, `:31237-31244` (R-1, **établi par lecture**, non joué : d'où le squelette et la sonde de T0) ; `test_fixtures.rs:116-121`, `:265`, `repositories/fiscal_years.rs:346`, `routes/fiscal_years.rs:79-83` (R-2) ; `gh issue view 165` et `166` : `CLOSED` (F-1) ; `README.md:240`, `user-manual.tex:232-233` (F-3).
  **Conception changée** : plus aucun `<a>` dans le SVG, écouteur unique et `data-href`, tableau voisin pour le clavier ; le repli du Dev Note est retiré. **Choix ajouté** : C-e332-23. **Défaut voisin corrigé** : AC19 faisait dire « aucun produit ni aucune charge » à un exercice qui a des produits. #165 et #166 sorties de la fiche ; ce qu'elles demandaient au-delà de #918 n'y est plus
  décrit, Guy les ayant fermées. Décomptes de cette fiche, recomptés : 23 AC, 15 mutations.
- 2026-11-06 — Remédiation de la validation P2 (rapport `kesh-gate-logs/e33-validate-p2-e332.md`, Opus 5.5, aucune compilation ni aucun test, aucun commit). Huit constats de cette fiche, tous remédiés, aucun réfuté : R-1, R-2 (MEDIUM), R-6, R-7, R-8, R-9, R-10, R-13 (LOW). Prémisses relues : `kesh-i18n/src/loader.rs:158`, `i18n.svelte.ts:19-28`, `supplier-invoices/+page.svelte:279-280`, `messages.ftl:2685-2686` (R-1) ; `axe.js:25743-25757`, `:16037-16039` (R-6) ; `+layout.svelte:295`, `:381` (R-7) ; `api-fixtures.ts:183-184` (R-13). R-2 raisonné sur le contrat de la 33-3a, non joué.
  **Conception changée** : AC19 : chaque graphique décide sur sa seule donnée (C-e332-24), le camembert n'ayant plus qu'un message pour `expensePosts` vide, et la phrase des barres devenue juste pour un reclassement : **une condition en moins**. **Exigence retirée** (R-7, R-8, C-e332-23) : plus de distance minimale testée, qui ne prouvait pas la lisibilité, et trois tranches au lieu de quatre, tirées du mois rendu ; la pleine largeur commence à 13 mois. Estimation, non jouée, à 1200 unités en pleine largeur (environ 0,8 px par unité à 1280 px) : au moins 100 unités entre deux étiquettes jusqu'à 36 mois. La limite de la trame est écrite (AC2-bis).
  Décomptes de cette fiche, recomptés après remédiation : 23 AC, 18 mutations (15 + M16, M17, M18 ; M15 récrite).

## Dev Agent Record

*À remplir au développement.*
