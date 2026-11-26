# Story 55-1a2 : le sélecteur d'exercice et le chargement sans *Générer*

Status: done

> **Découpée de la 55-1a le 2026-11-27 sur demande du mainteneur, après coup : le code est livré, le découpage sert la
> revue.** Fiche mère : `55-1a-etat-des-rapports-dans-l-url.md` (`split`), qui reste la référence complète — modèle,
> critères, T0, Dev Agent Record, Change Log. Cette fiche ne réécrit rien : elle dit **ce qu'une revue de code de la
> 55-1a2 vérifie**, par renvoi aux sections et aux numéros de la mère. Sœur : `55-1a1-etat-dans-l-url-et-retour.md`
> (l'état des Rapports dans l'URL, qui pose le modèle sur lequel la 55-1a2 branche le sélecteur et le chargement).

Epic : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md` — D15 (le sélecteur d'exercice), D16 (le
chargement sans *Générer*). Issue : #1604 (`bug`, P3, point b), par la mère. **Frontend seulement, aucune migration, aucun
code Rust** (dix clés `messages.ftl`).

## Story

En tant que comptable qui lit ses rapports,
je veux un seul sélecteur d'exercice, qui montre l'exercice en gras et garde « Du » et « Au » remplis, et un rapport qui se
charge seul quand je change d'exercice ou de dates,
afin de lire la période réelle de chaque rapport sans cliquer sur *Générer*.

## Ce que la fille couvre dans la fiche mère

- **Décisions** : D15 et D16 (fichier d'epic, § *Décisions*, et § *Décisions de l'utilisateur et de l'UX (2026-11-26,
  soir)*), confirmées par le mainteneur le 2026-11-27 (fichier d'epic, § *Décisions du mainteneur (2026-11-27)*).
- **Sections de la mère** : § *Le sélecteur d'exercice (D15 …)* (AC-14 à AC-19) et § *Le chargement sans Générer (D16 …)*
  (AC-20 à AC-26, avec les paragraphes *Textes (AC-18 amendé)*, *E2E*, *Manuel (AC-11 et AC-19 amendés)*, *Recette
  visuelle*). L'AC-21 est **celui récrit au T0** (péremption par numéro de demande, aucune annulation réseau, R-16).

### Ce que le relecteur vérifie, critère par critère

| AC | À vérifier (résumé ; le texte fait foi dans la mère) |
|---|---|
| AC-14 | Un seul `<label for="report-fiscal-year">`, aucun `aria-label` ; menu `font-semibold`, options `font-normal` ; texte d'un exercice : années si `name === nomParDefaut`, sinon le nom saisi ; doublons séparés par jour-mois, puis bornes complètes ; société sans exercice inchangée. |
| AC-15 | « Du »/« Au » (clés existantes) ; dates toujours remplies, choix d'un exercice → ses bornes dans le **gestionnaire** (jamais dans un effet) ; `etatDuMenu` : exercice (même partiel, lien « Exercice entier »), « Plusieurs », « Hors exercice », précédent sur une saisie invalide ; même `ReportSelector` sur tous les onglets à période, *snippets* des contrôles propres ; URL : les dates décident l'exercice (R-9). |
| AC-16 | **Remplacé par D16** (AC-24, AC-25, AC-26) : rien à vérifier en propre ; la clé `reports-nav-stale` n'existe pas. |
| AC-17 | `ReportPeriodHeading` dans les six vues, période **de la réponse** ; quatre formes d'en-tête ; `Intl.ListFormat` ; `formatSwissDate`. |
| AC-18 | Dix clés `reports-nav-*` ×4 locales ; `reports-loading`, `reports-error-no-entries-in-period` récrites ; `reports-instruction-select-and-generate`, `reports-aged-instruction-generate` retirées ; `reports-ledger-instruction` inchangée (55-2) ; gardes `i18n-keys` et `i18n-libelle-en-dur` recomptées. |
| AC-19 | Manuel : « Choisir la période » avant § *Bilan* ; phrases de l'Epic 46 récrites (R-23). |
| AC-20 | Chargement à l'ouverture, à l'arrivée sur l'onglet, au choix d'un exercice, à la sortie du `<fieldset>` « Période », à Entrée ; seulement si la clé des champs diffère des clés affichée et en vol et que la période est valide ; `from > to` : message, aucune requête. |
| AC-21 (récrit au T0) | La dernière demande l'emporte : `invalidateInFlight()` et numéro de demande ; la réponse périmée ignorée, sans message ; `api-client.ts` inchangé. |
| AC-22 | Clic (ou flèches) : le rapport des champs, repris de la mémoire ou chargé ; dates tapées puis clic sur un autre onglet : l'onglet quitté ne charge pas (drapeau `pointerdown`, R-19), le visé charge avec ces dates. |
| AC-23 | Grand livre non filtré et rapports par compte analytique gardent *Générer* ; balance âgée chargée à l'arrivée, son bouton retiré. |
| AC-24 | « Chargement… » en `role="status"` ; rechargement : ancien rapport grisé, `aria-busy="true"` ; « Aucune écriture sur cette période. » |
| AC-25 | Exports sur la **clé affichée** (classiques, grand livre, comptes analytiques) ; désactivés pendant une demande de l'onglet et tant que les champs diffèrent du rapport affiché (R-20). |
| AC-26 | « Plusieurs »/« Hors exercice » sur un onglet à un exercice : rien chargé, rapport **masqué** (gardé en mémoire), message `reports-nav-needs-one-fy` ; URL inchangée ; un seul exercice : chargé. |

### Points renvoyés au T0 qui reviennent à la fille

R-5 et R-7 (retirés), R-8, R-9, R-10, R-11, R-12, R-13 (taille et modules), R-14 **pour la partie chargement** (AC-15,
AC-16, AC-19, C-55-1-4, C-55-1-8), R-15, R-16 (HIGH, tranché : pas d'annulation réseau), R-18, R-19, R-20, R-21, R-23,
R-24, R-25, R-26 (la liste unique des choix (a) à (k)). LOW repris au T0 : SE-4, SE-7, SE-8, DE-15 à DE-19. Décisions :
mère, Change Log du **2026-11-27 (T0)**, puces « R-16 », « R-8, R-11, SE-4 », « R-9 », « R-10 », « R-12, R-15, R-26 »,
« R-13 », « R-18 » à « R-25 », « C-55-1-9 », les LOW SE-7, SE-8, DE-15 à DE-19, et « AC-21 récrit ».

Choix et écarts de la mère qui la concernent : C-55-1-4 (renversé par D16), C-55-1-8, C-55-1-9, C-55-1-10.

## Fichiers

Relevés par `git diff --stat 617508645 5645d6890` et le contenu. Un fichier partagé avec la 55-1a1 est nommé dans les
deux, avec la part de chacune.

| Fichier | Part de la 55-1a2 |
|---|---|
| `frontend/src/lib/features/reports/fiscal-year-menu.ts` (neuf) | tout : `nomParDefaut`, `anneesDe`, `formatJourMois`, `libellesExercices`, `EtatMenu`, `etatDuMenu`, `EnTete`, `enTetePeriode` |
| `frontend/src/lib/features/reports/fiscal-year-menu.test.ts` (neuf) | tout |
| `frontend/src/lib/features/reports/ReportSelector.svelte` | tout le composant tel que récrit (libellé, menu natif, `<fieldset>` « Période », « Du »/« Au », « Exercice entier », options désactivées, message d'un seul exercice, période invalide, *snippets* `controls` et `actions`, `focusout`/Entrée, plus de `$bindable` ni de désactivation pendant `loading`) |
| `frontend/src/lib/features/reports/ReportSelector.test.ts` (neuf) | tout |
| `frontend/src/lib/features/reports/ReportPeriodHeading.svelte` (neuf) | tout |
| `frontend/src/lib/features/reports/{BalanceSheet,IncomeStatement,TrialBalance,JournalReport,VatReport,GeneralLedger}View.svelte` | l'en-tête de période remplacé par `ReportPeriodHeading` (prop `fiscalYears` facultative), replis de `reports-loading` et de `reports-error-no-entries-in-period` |
| `frontend/src/lib/features/reports/reports-url.ts` | `ClassicPeriod`, `resolveClassicPeriod` seulement (R-9 : les dates décident l'exercice) |
| `frontend/src/lib/features/reports/reports-url.test.ts` | `describe` « période d'un onglet classique (AC-15, R-9) » |
| `frontend/src/routes/(app)/reports/+page.svelte` | le sélecteur et le chargement : état `selectedFiscalYearId`/`periodStart`/`periodEnd`/`menu`/`periodInvalid`, `fiscalYearChanged`, `syncMenu`, `setPeriod`, `periodValid`, `projectQuery`/`ledgerQuery`/`classicQueryOfFields`, `generate()` (la dernière demande l'emporte, numéro de demande ; *Générer* retiré des onglets classiques et de la balance âgée), `activeReportPeriod`, `activeBusy`, `reloadingActive`, `hiddenByMenu`, `exportReport` et les exports (clé affichée, désactivation), `commitPeriod`, `selectFiscalYear`, `wholeFiscalYear`, `onDatesInput`, `tablistEl` et le drapeau `pointerdown`, `onPeriodCommit`, `generateLedger` ; le bandeau unique `ReportSelector` (rapports par compte analytique compris) ; retrait de P17 |
| `frontend/src/routes/(app)/reports/reports-page-url.test.ts` | `describe` « le chargement sans Générer (D16, AC-20 à AC-26) », le cas « changement d'exercice : rapport par compte analytique vidé ; export du rapport affiché » (R-10), le cas « un `fy` qui contredit les dates est réécrit » (R-9) |
| `frontend/src/routes/(app)/reports/reports-page-groupes.test.ts` | adaptations à D16 (attente du chargement au lieu du clic, « Chargement… », S4, X1, deux cas récrits ; mère, Dev Agent Record, *Vitest existants adaptés*) |
| `frontend/src/lib/shared/i18n-keys.test.ts`, `i18n-libelle-en-dur.test.ts` | compteurs recomptés par les gardes (`sitesTotal` 3789 → 3788 ; `CANDIDATES_ATTENDUES` 151 → 152, `conforme` 130 → 131) |
| `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` | tout le diff de la 55-1a |
| `frontend/tests/e2e/reports-navigation.spec.ts` | tests AC-3/AC-20, AC-24, D15 |
| `frontend/tests/e2e/reports.spec.ts`, `reports-groupes.spec.ts`, `reports-groupes-export.spec.ts`, `reports-balance-groupes.spec.ts`, `reports-currency.spec.ts`, `reports-export-pdf.spec.ts`, `opening-balances.spec.ts`, `invoice-write-off.spec.ts`, `dunning-roundtrip.spec.ts` | adaptations à D16 (clics sur *Générer* retirés ou remplacés ; menu désactivé sans exercice) |
| `docs/manual/fr/user-manual.tex` | « Choisir la période » (§ *Rapports comptables*) ; phrases de l'Epic 46 récrites (reprise d'une erreur, interruption, audit) |

## Commits

Les quatre commits de la 55-1a sont **communs** aux deux filles ; aucun commit neuf.

- `5d98bc9ab` — T0 (fiche mère seule) : les puces citées plus haut.
- `e33426630` — code et tests : à la 55-1a2, `fiscal-year-menu.ts` et son test, `ReportSelector.svelte` et son test,
  `ReportPeriodHeading.svelte`, les six vues, `resolveClassicPeriod` et son `describe`, la part sélecteur et chargement de
  `+page.svelte` (tableau ci-dessus), la part D16 de `reports-page-url.test.ts`, les adaptations de
  `reports-page-groupes.test.ts`, les deux compteurs i18n, la part D15/D16 de `reports-navigation.spec.ts` et les neuf
  specs E2E adaptées.
- `946347e16` — textes et manuel : à la 55-1a2, les quatre `messages.ftl` et les hunks du manuel `@@ +4170` (« Choisir la
  période ») et `@@ +4303` (phrases de l'Epic 46).
- `5645d6890` — fiche et `reports-navigation.spec.ts` : le test D15 déplacé en dernier, sur `with-company-no-fy` et deux
  exercices créés par l'API.

Pour information, commits postérieurs qui retouchent ces fichiers (hors périmètre de cette revue) : `8e801d1aa` (55-2 :
Grand livre branché sur `ReportSelector`, chargé seul une fois filtré ; liens), `c3f28d2f9` (55-1b), `647b5b039` (55-4),
et pour le manuel `215fad3c8`, `37a25b439`, `0a8f04d06`, `9952b2d30`, `939bfa438`.

## Tests

Depuis le Dev Agent Record de la mère (*Tests neufs*, *Preuve du rouge*, *Mutations*) :

- **Vitest** : `fiscal-year-menu.test.ts` (`nomParDefaut`, `libellesExercices`, `etatDuMenu`, `enTetePeriode`,
  `formatJourMois`) ; `ReportSelector.test.ts` (15 cas : libellé, graisse, choix, « Exercice entier », « Plusieurs »,
  « Hors exercice », `unSeulExercice`, `<fieldset>`, Du → Au, saisie, R-18, aucun exercice, période invalide) ;
  `reports-url.test.ts` (R-9) ; `reports-page-url.test.ts`, cas : AC-20 (Tab, sortie, Entrée, dates inchangées,
  `from > to`, choix d'exercice), AC-21, R-17 (péremption au retour aux dates affichées — déclenchée par les dates, jugée
  avec la 55-1a1), AC-22/R-19 (Safari et `relatedTarget` dans la liste), AC-24, R-18, AC-25/R-20, AC-26, « Exercice
  entier » repris de la mémoire, changement d'exercice et export analytique (R-10), `fy` contredit (R-9).
- **E2E** `reports-navigation.spec.ts` : AC-3/AC-20, AC-24, D15 ; neuf specs adaptées (liste ci-dessus).
- **Preuve du rouge** : Vitest de la page sur la page et le sélecteur d'avant, 33 rouges sur 34 (commun aux deux filles).
- **Mutations tuées** qui reviennent à la fille : numéro de demande ignoré ; `aria-label` remis ; `relatedTarget` non
  consulté ; clé affichée non comparée ; `aria-busy` jamais posé ; nom par défaut non comparé ; doublon partout ; `<` au
  lieu de `≤` ; début après fin « Hors exercice » ; péremption R-17 omise ; drapeau R-19 ignoré ; rapport non retiré
  (AC-26) ; export analytique sur les champs ; P17 remis ; exports actifs quand les champs diffèrent.
- **Constat manuel non fait** : lecture par un lecteur d'écran (mère, Dev Agent Record).

## Gates

Ceux de la mère, communs (mère, § *Gates* et Dev Agent Record, *Gates*, au dernier commit de code `946347e16`) : front
(check, lint-i18n-ownership, Vitest 5439, build), E2E partiel 12 specs, `kesh-i18n` 42/42 et `textes_coherents` 34/34 — ces
deux-là sont dus à la 55-1a2 (`messages.ftl`, manuel) ; E2E complet et `gate-ci.sh` en fin d'epic (dérogation du fichier
d'epic). Aucun gate neuf pour la fille. Captures : mère, Dev Agent Record, *Captures* (sélecteur, période partielle,
« Plusieurs », « Hors exercice », rechargement grisé).

## Issues à ouvrir

Reprises de la mère (*Issues à ouvrir*), part de la 55-1a2 :

- (LOW, `enhancement`) R-25 : aucun bouton « Réessayer » après un chargement en erreur — **reprise par la 55-4** (L1) : à
  ne pas ouvrir si la revue de la 55-4 la tranche.
- (MEDIUM, `bug` P3, préexistant, hors story) Le manuel utilisateur ne se compilait plus (`\og … \fg{}`,
  `user-manual.tex:3562`, venu de `1bd88a987`) : relevé au contrôle du PDF de la 55-1a2 ; corrigé depuis dans la branche
  (`9952b2d30`, repris de `fix/manuel-guillemets`) — à ne pas ouvrir si la correction est confirmée.
- L'issue « export d'une autre période » est retirée (AC-25 la ferme).

## Note de version

Part de la note de la mère : Le sélecteur d'exercice est le même sur tous les onglets : il montre l'exercice seul, en
gras (ses années, ou son nom s'il a été renommé) ; « Du » et « Au » sont toujours remplis, « Exercice entier » remet les
bornes, « Plusieurs » ou « Hors exercice » s'affichent quand les dates sortent d'un exercice, et l'en-tête du rapport dit
la période réelle. Le bouton *Générer* disparaît : le rapport se charge à l'ouverture, au choix d'un exercice, en quittant
les dates ou par Entrée, et reste affiché, grisé, pendant qu'il se recharge ; seul le Grand livre de tous les comptes
garde un bouton. Les exports portent sur le rapport affiché. (refs #1604)

## Périmètre de revue

**À lire** :

- `fiscal-year-menu.ts` et son test (réplique de `nom_par_defaut`, `crates/kesh-core/src/accounting/exercice.rs:33-38`) ;
  `ReportSelector.svelte` et son test ; `ReportPeriodHeading.svelte` et les six vues ; `resolveClassicPeriod`.
- `+page.svelte` : les fonctions du tableau *Fichiers* — en priorité `generate()` (numéro de demande, `mySeq !== genSeq`),
  `onPeriodCommit` et le drapeau `pointerdown` (R-19), `selectFiscalYear`, `fiscalYearChanged` (rapports par compte
  analytique vidés, aucun rapport classique vidé), `hiddenByMenu` (AC-26), `exportReport` (clé affichée, désactivation),
  et chaque **appel** de `loadActiveIfNeeded` et de `writeUrl` depuis un geste du sélecteur. Questions : un chargement par
  geste, jamais deux pour une clé ? aucune réponse périmée affichée ? un export ne sort-il jamais une autre période que
  l'écran ? le focus reste-t-il pendant un chargement ?
- `messages.ftl` (dix clés, deux récrites, deux retirées, aucune concaténation) ; Vitest et E2E de la part ci-dessus ; les
  neuf specs adaptées (chaque clic retiré remplacé par une attente qui constate la même chose) ; manuel : « Choisir la
  période » et les phrases de l'Epic 46 (PDF aplati, `CLAUDE.md` § *Le prompt d'une passe doit NOMMER le manuel*).

**Laissé à la 55-1a1** : la lecture et l'écriture de l'URL (`parseReportUrl`, `buildReportSearch`, `reportHref`,
`ledgerHref`), `applyUrl` et son `$effect`, `writeUrl` et `lastHandledKey`, `selectTab` et le clavier (empiler ou
remplacer), le corps de `loadActiveIfNeeded` et l'invariant des clés affichée et en vol, la normalisation, *Retour* et
*Suivant* ; « Naviguer dans les rapports » et l'item « Accès direct » du manuel. Quand un geste de la 55-1a2 appelle une
fonction de la 55-1a1, la 55-1a2 juge l'appel, la 55-1a1 la fonction.

## Change Log
- **2026-11-26 — revue de code, close (trois passes).** Tendance : P1 complète Sonnet **0/1/2/3** → P2 complète Opus
  **0/0/2/3** → P3 ciblée Sonnet **0/0/0/2**. P1 : P1-1 (HIGH) dates puis Tab vers l'onglet actif ne chargeait rien,
  P1-2 (MEDIUM) clic sur l'onglet actif idem — `leavesToTablist` ne retient plus qu'un AUTRE onglet (`pointerTab`,
  `tabOfFocus`) ; P1-3 (MEDIUM) AC-26 étendu aux rapports par compte analytique en mode Exercice (`hiddenByMenu`),
  comme le dit le manuel ; P1-5 (LOW, production, corrigé dans la story) période invalide signalée à l'arrivée sur un
  onglet (`updatePeriodInvalid`) ; commit `a50712ce2`, quatre tests Vitest rouges sous M2 à M4. P2 : P2-1 (MEDIUM,
  régression de la remédiation) exports d'un rapport par compte analytique retiré de l'écran restés actifs, P2-2
  (MEDIUM, manuel) phrase des exports fausse pour le grand livre et les comptes analytiques — commit `5a3242d80`
  (`|| hiddenByMenu`, manuel § des exports, note de la fiche mère), test rouge sous M5 (`mutations-p2/`). La P3 ciblée
  relit ce seul correctif : rien au-dessus de LOW. Le correctif de `leavesToTablist` sert aussi le grand livre (55-2),
  relu par la P2 complète. Aucun reclassement.
  Prompts versionnés à côté de la fiche (`<clé>-code-review-prompt-pN.md`, consignes communes `55-code-review-brief-commun.md`) ; rapports hors dépôt, `kesh-gate-logs/s55-revue/<clé>/revue-pN-rapport.md`. Base : `epic/55-navigation` après la fusion de la PR de l'Epic 46 (`c8fe4d0ac`, revue d'intégration P1 Opus 0/0/0/2, LOW corrigés `f16463f91`). LOW non corrigés : issues à ouvrir, `kesh-gate-logs/s55-revue/issues-a-ouvrir.md` (R1, R5, R8 à R12, R27, R28). Gates de fin d'epic (gate-ci et E2E complet) au dernier commit de code de l'epic : voir le fichier d'epic.
