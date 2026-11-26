# Story 55-1a1 : l'état des Rapports dans l'URL — Retour, Suivant, lien partagé

Status: review

> **Découpée de la 55-1a le 2026-11-27 sur demande du mainteneur, après coup : le code est livré, le découpage sert la
> revue.** Fiche mère : `55-1a-etat-des-rapports-dans-l-url.md` (`split`), qui reste la référence complète — modèle,
> critères, T0, Dev Agent Record, Change Log. Cette fiche ne réécrit rien : elle dit **ce qu'une revue de code de la
> 55-1a1 vérifie**, par renvoi aux sections et aux numéros de la mère. Sœur : `55-1a2-selecteur-et-chargement-sans-generer.md`
> (le sélecteur d'exercice et le chargement sans *Générer*). Règle appliquée : `CLAUDE.md` § *Règle de splitting préventif*
> (« dégager une story qui pose le pattern, puis le rollout ») — la 55-1a1 pose le modèle de l'URL, la 55-1a2 y branche le
> sélecteur et le chargement.

Epic : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md` — D1 à D3, D5. Issue : #1604 (`bug`, P3, point b).
**Frontend seulement, aucune migration, aucun code Rust.**

## Story

En tant que comptable qui lit ses rapports,
je veux que *Retour* et *Suivant* du navigateur me ramènent au rapport que je regardais, avec sa période et ses filtres, et
qu'un lien copié rouvre la même vue,
afin de passer du bilan au grand livre d'un compte et d'en revenir sans perdre ma place ni tout régénérer.

## Ce que la fille couvre dans la fiche mère

- **Décisions** : D1 (l'URL source de l'état), D2 (empiler ou remplacer selon le geste), D3 (une adresse venue de
  l'extérieur affiche son rapport), D5 (valeurs invalides ignorées, URL normalisée) — fichier d'epic, § *Décisions*, avec
  leurs amendements par D16 en tant qu'ils touchent l'écriture et la lecture de l'URL.
- **Le modèle** (mère, § *Le modèle (D1 à D5)*) : *Paramètres*, *Gestes (D2) — la page écrit*, *Appliquer une URL (D3) —
  la page lit* (points 1 à 5), *Valeurs invalides (D5)*, avec les lectures du T0 qui les amendent (Change Log de la mère,
  2026-11-27, puces « R-1, R-2, R-3, R-17 » et « R-14, R-22 »).
- **Critères** : AC-1 à AC-11 et AC-13, **tels qu'amendés** (annotations D16 dans la mère : AC-2, AC-3, AC-7, AC-13 ;
  lecture D16 de AC-8 a/b, AC-9, AC-10, AC-11 au T0). L'AC-12 est remplacé par l'AC-18 (55-1a2).

### Ce que le relecteur vérifie, critère par critère

| AC | À vérifier (résumé ; le texte fait foi dans la mère) |
|---|---|
| AC-1 | Bilan → lien de compte → grand livre ; *Retour* : le Bilan, sans requête ; *Suivant* : le grand livre, sans requête. Même depuis la Balance. |
| AC-2 | Clic sur un onglet = une entrée d'historique ; l'onglet jamais affiché **se charge** (D16), l'URL complète est empilée ; *Retour*/*Suivant* sans requête. |
| AC-3 | Changer la période **remplace** l'URL (`history.length` inchangé), une requête (le geste est la sortie des dates depuis D16 ; son déclenchement est de la 55-1a2, l'écriture de l'URL ici). |
| AC-4 | Lien collé dans un nouveau contexte (cookies seuls) et rechargement : même rapport, contrôles remplis, sans clic. |
| AC-5 | `tab=xyz`, `fy=999999`, `fy=abc`, `from=2026-02-30`, `accountId=-3`, paramètre étranger : ignorés, URL normalisée en `replace`, aucune valeur invalide envoyée. `from > to` et « hors exercice » ne sont plus jugés par l'API (T0, R-9 ; leur affichage est de la 55-1a2). |
| AC-6 | `ledgerHref` rend la même chaîne (trois cas) ; `homepage-charts.spec.ts` et `open-items.spec.ts` sans modification ; X1 de la 46-2 rejoué avec un `page.url` qui suit `goto`. |
| AC-7 | Compteurs de requêtes et de `goto` par geste (amendés par D16 : clic sur un onglet non affiché aux champs = 1 requête) ; normalisation = 1 `goto` qui remplace ; aucune boucle. |
| AC-8 | Courses : (a) clic d'onglets pendant une requête ; (b) *Retour* vers un rapport absent pendant le chargement d'un autre : **demandé, l'autre périmé** (AC-13, R-3) ; (b') autre exercice ; (c) *Retour* vers la clé affichée pendant une demande du même onglet : 0 requête, demande périmée. |
| AC-9 | Adresse d'un autre exercice : la demande qu'elle lance n'est pas périmée par le changement d'exercice ; rapport par compte analytique vidé, compte et mode gardés ; menu *Rapports* : Bilan. (P17 retiré au T0 : la clause « période restaurée non effacée » tombe, SE-4.) |
| AC-10 | Pas de message d'erreur d'un autre onglet après *Retour* ; `aria-selected`/`tabindex` ; `keepFocus`/`noScroll` sur chaque `goto` ; défilement gardé au clic d'onglet ; axe à zéro. |
| AC-11 | Manuel : paragraphe « Naviguer dans les rapports » (§ *Visualisation et exports*) et item « Accès direct depuis le bilan et la balance » (§ *Grand livre*). |
| AC-13 | Une adresse collée, rechargée, ou atteinte par *Retour*/*Suivant* affiche son rapport sans *Générer* (0 requête s'il est gardé, 1 sinon) ; exigence technique R-4 (jamais deux appels pour une même clé, ni d'appel pour la clé affichée). Les puces du clic sur un onglet sont remplacées par l'AC-22 (55-1a2). |

### Points renvoyés au T0 qui reviennent à la fille

R-1, R-2, R-3 (l'AC-13 prime), R-4 (tranché), R-6 (§ *Gates*), R-14 **pour la partie URL** (sites du modèle qui
supposaient *Générer* : point 3 d'*Appliquer une URL*, AC-8 a/b, AC-9, AC-10, AC-11, AC-13), R-17 (retour aux dates du
rapport affiché pendant une demande en vol : péremption, même branche que R-1 dans `loadActiveIfNeeded`), R-22 (`/reports`,
URL incomplète, lien du menu, *Où l'URL s'écrit*). LOW repris au T0 : V8-1, V8-3, V6-3 à V6-8, V7-5 à V7-10 (absorbés ou
sans objet sous D16). Décisions : mère, Change Log du **2026-11-27 (T0)**, puces « Base », « R-1, R-2, R-3, R-17 (DE-2) »,
« R-14, R-22 (DE-7) », « R-4 … R-6 », et les LOW V8-1, V8-3, V6-6.

Choix et écarts de la mère qui la concernent : C-55-1-1, C-55-1-2, C-55-1-3, C-55-1-5, C-55-1-6 (retiré au T0), C-55-1-7.

## Fichiers

Relevés par `git diff --stat 617508645 5645d6890` et le contenu. Un fichier partagé avec la 55-1a2 est nommé dans les
deux, avec la part de chacune.

| Fichier | Part de la 55-1a1 |
|---|---|
| `frontend/src/lib/features/reports/reports-url.ts` (neuf) | tout, **sauf** `resolveClassicPeriod` et `ClassicPeriod` (R-9, 55-1a2) : `REPORT_TABS`, `TabId`, `ClassicTab`, `isTabId`, `isClassicTab`, `ExerciceBornes`, `exerciceDe`, `ReportUrlParams`, `entier`, `parseReportUrl`, `buildReportSearch`, `reportHref` |
| `frontend/src/lib/features/reports/reports-url.test.ts` (neuf) | `describe` « lecture (D5) » et « écriture canonique » (dont `ledgerHref`, trois cas, et les neuf onglets) |
| `frontend/src/lib/features/reports/reports.api.ts` | `ledgerHref` réécrite sur `reportHref`, sortie inchangée |
| `frontend/src/routes/(app)/reports/+page.svelte` | l'URL et les clés : `inFlightTab`, `displayedKeys`, `invalidateInFlight` (péremption, hors le choix « la dernière demande l'emporte »), `classicKey`, `ledgerKey`, `fieldsKey`, `urlForTab`, `isDisplayed`, `loadActiveIfNeeded` (le chemin unique et ses branches clé en vol / clé affichée, R-1, R-17), `writeUrl`, `applyUrl`, le `$effect` sur `page.url` (remplace l'`onMount` de la 21-7 et l'effet du lien du grand livre de la 24-1), `handleTabKeydown`, `selectTab` (empile / remplace, `goto` en dernier, `errorMsg` effacé, `resyncActiveGroupedTab`) |
| `frontend/src/routes/(app)/reports/page-url.test-helper.svelte.ts` (neuf) | tout (`page.url` en `$state`, `goto` simulé, `goto` différé) |
| `frontend/src/routes/(app)/reports/reports-page-url.test.ts` (neuf) | `describe` « arrivée, rechargement, lien partagé (AC-4, AC-5, D16) », « Retour et Suivant (AC-1, AC-2, AC-13) », « X1 de la 46-2 rejoué (AC-6) », et le cas « AC-9 — une adresse d'un autre exercice » |
| `frontend/tests/e2e/reports-navigation.spec.ts` (neuf) | tests AC-1, AC-2/AC-22 (pour l'historique), AC-4, AC-5, AC-10 |
| `frontend/tests/e2e/homepage-charts.spec.ts`, `open-items.spec.ts` | aucune modification (AC-6) : leur passage vert est à constater |
| `docs/manual/fr/user-manual.tex` | « Naviguer dans les rapports » (§ *Visualisation et exports*) ; item « Accès direct » (§ *Grand livre*) |

Aucune clé i18n n'est ajoutée par la 55-1a1 (l'URL ne montre aucun texte).

## Commits

Les quatre commits de la 55-1a sont **communs** aux deux filles ; aucun commit neuf.

- `5d98bc9ab` — T0 (fiche mère seule) : les puces citées plus haut.
- `e33426630` — code et tests : à la 55-1a1, `reports-url.ts` et son test (hors `resolveClassicPeriod`), `reports.api.ts`,
  `page-url.test-helper.svelte.ts`, la part URL de `+page.svelte` (tableau ci-dessus), la part URL de
  `reports-page-url.test.ts` et de `reports-navigation.spec.ts`.
- `946347e16` — textes et manuel : à la 55-1a1, les deux hunks du manuel « Accès direct » (hunk `@@ +4416`) et
  « Naviguer dans les rapports » (hunk `@@ +4566`) ; aucune ligne de `messages.ftl`.
- `5645d6890` — fiche et `reports-navigation.spec.ts` (bornes de l'exercice du preset) : la correction de la spec touche
  aussi les tests de la 55-1a1 (bornes lues à l'API).

Pour information, commits postérieurs qui retouchent ces fichiers (hors périmètre de cette revue) : `c3f28d2f9` (55-1b,
la présentation dans l'URL), `8e801d1aa` (55-2), `647b5b039` (55-4 : test de la clé écrite par la page), et pour la spec
`215fad3c8`, `07ff47e62`, `37a25b439`, `0a8f04d06`, `8db734ef8`.

## Tests

Depuis le Dev Agent Record de la mère (*Tests neufs*, *Preuve du rouge*, *Mutations*) :

- **Vitest** : `reports-url.test.ts` (lecture, écriture, `ledgerHref`) ; `reports-page-url.test.ts`, cas : `/reports`
  normalisé ; adresse complète collée ou rechargée ; valeurs invalides (`it.each`) ; balance âgée par son adresse (1
  requête, 0 `goto`) ; AC-1 ; AC-2/AC-22 (historique) ; clic sur un onglet déjà affiché (non-régression) ; options de
  `goto` (AC-10) ; échec puis *Retour* (AC-10) ; AC-8 b/AC-13 ; point 5 (`goto` différé supplanté) ; AC-9 (adresse d'un
  autre exercice) ; X1 rejoué. Le cas « un `fy` qui contredit les dates est réécrit » est de la 55-1a2 (R-9).
- **E2E** `reports-navigation.spec.ts` : AC-1, AC-2/AC-22, AC-4, AC-5, AC-10.
- **Preuve du rouge** : E2E AC-1 rouge sur le code d'avant (`e2e-rouge-avant.log`) ; Vitest de la page rouges sur la page
  d'avant (33 sur 34, commun aux deux filles).
- **Mutations tuées** qui reviennent à la fille : clic qui remplace ; chargement qui empile ; `errorMsg` non effacé ;
  validation de `fy` retirée ; URL écrite dans `generate()`. Non faite : « clé écrite non enregistrée » (test ajouté
  depuis par la 55-4, `647b5b039`).

## Gates

Ceux de la mère, communs (mère, § *Gates* et Dev Agent Record, *Gates*, au dernier commit de code `946347e16`) : front
(check, lint-i18n-ownership, Vitest 5439, build), E2E partiel 12 specs, `kesh-i18n` 42/42, `textes_coherents` 34/34 ; E2E
complet et `gate-ci.sh` en fin d'epic (dérogation du fichier d'epic). Aucun gate neuf pour la fille.

## Issues à ouvrir

Reprises de la mère (*Issues à ouvrir*), part de la 55-1a1 :

- (LOW, `enhancement`) Les rapports par compte analytique ne sont pas dans l'URL : *Retour* et un lien n'y ramènent que
  l'onglet (C-55-1-7).
- (LOW, `enhancement`) Après la connexion, Kesh mène à l'accueil et non à l'adresse demandée (`(app)/+layout.ts:10-11`,
  `login/+page.svelte:74`) : un lien de rapport partagé à un non-connecté ne rouvre pas la vue.
- (LOW, `technical-debt`) Mutation « clé écrite non enregistrée » sans test qui la tue — **reprise par la 55-4** (L2,
  `647b5b039`) : à ne pas ouvrir si la revue de la 55-4 la confirme.

## Note de version

Part de la note de la mère : *Rapports* : *Retour* et *Suivant* du navigateur ramènent au rapport précédent, avec son
exercice, sa période et son compte ; chaque clic sur un onglet est une étape de l'historique ; l'adresse d'une vue se copie
et rouvre la même vue pour un utilisateur connecté, sans cliquer sur *Générer* (comme les liens de l'échéancier et des
rappels vers la balance âgée, qui l'affichent désormais à l'arrivée) ; le lien *Rapports* du menu ramène au Bilan.
(refs #1604)

## Périmètre de revue

**À lire** :

- `reports-url.ts` en entier sauf `resolveClassicPeriod` ; `reports-url.test.ts` (deux premiers `describe`) ;
  `reports.api.ts` (`ledgerHref`, contre l'ancienne construction).
- `+page.svelte` : les fonctions du tableau *Fichiers* — en priorité `applyUrl`, le `$effect` sur `page.url`, `writeUrl`
  et `lastHandledKey`, `selectTab`, `handleTabKeydown`, `loadActiveIfNeeded`, les clés `displayedKeys`/`inFlightTab`,
  `invalidateInFlight`. Questions : une URL écrite par la page n'est-elle jamais réappliquée ? une seule écriture par
  geste ? `goto` toujours avec `keepFocus`/`noScroll`, jamais attendu ? normalisation sans boucle ? règle de la 46-2
  (`resyncActiveGroupedTab`) appelée après tout affichage sans requête ?
- Helper et Vitest de la page (part ci-dessus), E2E AC-1, AC-2, AC-4, AC-5, AC-10 ; manuel : les deux passages (PDF
  aplati, `CLAUDE.md` § *Le prompt d'une passe doit NOMMER le manuel*).

**Laissé à la 55-1a2** : `resolveClassicPeriod` (dates qui décident l'exercice) ; `generate()` et « la dernière demande
l'emporte » ; les déclencheurs du chargement (menu, sortie des dates, Entrée, `pointerdown` des onglets) ; les exports ;
les états (« Chargement… », grisé, `aria-busy`, rapport retiré) ; `ReportSelector`, `fiscal-year-menu.ts`,
`ReportPeriodHeading`, les six vues ; `messages.ftl` ; « Choisir la période » et la réécriture des phrases de l'Epic 46 au
manuel ; les specs E2E adaptées à D16. Quand une fonction de la 55-1a1 est appelée par un geste de la 55-1a2
(`loadActiveIfNeeded` depuis le menu ou les dates, `writeUrl` depuis le chargement), la 55-1a1 juge la fonction, la
55-1a2 l'appel.
