# Story 1582-nom-comptabilite : Afficher le nom de la comptabilité ouverte sous le logo et dans le titre de l'onglet (#1582)

Status: ready-for-dev

<!-- Fiche écrite par bmad-create-story le 2026-11-25 (Sonnet 5.5). La validation P1 (Opus, contexte frais) reste à lancer : prompt `1582-nom-comptabilite-validation-prompt-p1.md`. -->

## Story

En tant que **personne qui tient deux comptabilités dans deux onglets**,
je veux **voir à tout instant le nom de la comptabilité ouverte, sous le logo et dans le titre de l'onglet**,
afin de **ne jamais saisir dans la mauvaise comptabilité**.

- Issue : [#1582](https://github.com/guycorbaz/kesh/issues/1582) (`enhancement`, P3), décisions du Project Lead du 2026-11-25.
- **Hors epic.** Visée : **v0.20.0** (coupure le jeudi 2026-11-26 au soir, release le vendredi 2026-11-27 au matin). **Si elle n'est pas close le jeudi soir, elle passe à la v0.21.0 sans retarder la release** (décision du Project Lead).
- **Frontend seul. Aucune migration, aucun changement d'API, aucun changement de serveur.** Vérifié au code : `GET /api/v1/companies/current` rend déjà `company.name`, `firstName`, `lastName`, `orgType` à tout rôle authentifié (`crates/kesh-api/src/lib.rs:1322`, `routes/companies.rs:650`, `CompanyJson` `:33-82`). Rien ne démentit ce point.

## Ce qui se lit au code (`1e23edb55`)

1. **Le logo n'est pas dans le menu latéral : il est dans l'en-tête** (`frontend/src/routes/(app)/+layout.svelte:311-314`, un `<a href="/">` portant `<span class="text-xl">Kesh</span>`, dans un `<header class="sticky top-0 … h-14 …">` de 56 px). Le menu latéral (`<aside class="w-56 shrink-0 …">`, `:394`) est **sous** l'en-tête.
2. **Il n'existe pas de menu latéral « replié »** : `grep -n -i "collapsed\|replié" (app)/+layout.svelte` ne trouve rien. Ce qui se replie, ce sont les **trois groupes** (`<details>` Quotidien, Mensuel, Administration, `:404-410`, état persisté en `localStorage`). La largeur de l'`<aside>` est fixe (`w-56`). **Conséquence : « menu replié compris » (décision 3) se lit comme « groupes repliés »**, et la protection est structurelle : le nom vit dans l'en-tête, hors de la `<nav>`, donc il ne bouge ni ne se déforme quand un groupe se replie. À confirmer par le Project Lead (question Q1) ; un vrai menu à icônes serait une autre story.
3. **La société courante n'est lue par aucun store.** Les écrans qui en ont besoin appellent chacun `GET /api/v1/companies/current` (`reconciliation/+page.svelte:24`, `bank-import/+page.svelte:38`, `bank-import/manual/+page.svelte:27`, `settings.api.ts:26` `fetchCompanyCurrent`). Le layout ne lit pas la société ; `onboardingState` (`features/onboarding/onboarding.svelte.ts`) ne porte pas son nom. Il faut donc **un petit store** (patron : `lib/app/stores/mode.svelte.ts`, `lib/shared/utils/app-version.svelte.ts`).
4. **Le type TypeScript `CompanyJson` (`features/settings/settings.types.ts:12-52`) n'a ni `firstName` ni `lastName`** alors que le serveur les envoie (miroir écrit à la main, il le dit lui-même). La fiche **ne les ajoute pas** (décision D1 : on n'en a pas besoin).
5. **`companies.name` est déjà « Prénom Nom » pour une personne physique.** `routes/onboarding.rs:542-545` : `(Some(f), Some(l)) => format!("{f} {l}")`, commentaire « `name` est alors recomposé "Prénom Nom" » (`entities/company.rs:140-146` : « `name` reste l'affichage canonique recomposé »). C'est ce `name` que le PDF imprime.
6. **Mais la correction du nom (Réglages, #1097) ne touche pas `first_name` / `last_name`** : `PUT /companies/current/name` les reporte à l'identique (`routes/companies.rs:539-565`, « limite D3, #1227 : pour une personne physique, le nom corrigé peut ne plus être "prénom nom" »). **Recomposer `firstName lastName` côté écran afficherait donc un nom périmé après une correction** — l'inverse de la décision 4 (« corrigé dans les paramètres, il suit »). Les Réglages n'affichent et n'éditent que `name` (`settings/+page.svelte`, pas de `firstName`).
7. **Titres de page** : **50 fichiers** portent chacun leur `<svelte:head><title>` (`grep -rl "<title>" frontend/src | wc -l` = 50, dont l'`app.html` « Kesh », `+error.svelte`, `setup`, `forgot-password`), toujours sous la forme `<page> - Kesh` ou `<page> — Kesh`, **sauf `reports/+page.svelte:619` : `Kesh — Rapports`**. Aucun test ne lit `document.title` hors `McpKeysSection.test.ts:379-423` (qui vérifie qu'il ne contient pas un secret) ; aucune spec E2E n'a de `toHaveTitle`. Il n'y a pas de titre global : **le titre est une affaire de page**, et le layout ne peut pas l'écraser (le `<title>` de la page, monté dans ses enfants, passe après).
8. **Mode démo** : `onboardingState.isDemo` n'affiche que `DemoBanner` (`+layout.svelte:373-378`). La société de démonstration est une société ordinaire (`kesh-seed/src/lib.rs:118-250`, « Démo SA » en fr-CH). Rien à changer au bandeau.
9. **L'en-tête a un plancher de largeur** (`min-w-[1280px]` sur la racine, `:305`) : Kesh n'est pas adapté au mobile, la fiche n'invente pas de cas étroit.
10. **Le module `(app)` est démonté puis remonté** quand l'utilisateur passe par `/onboarding` (hors du groupe `(app)`) : c'est le cas de « Réinitialiser la démonstration » (`DemoBanner.svelte` → `goto('/onboarding')`) puis du retour dans l'application. Un store de module garde alors le nom de la société **d'avant** : il faut le vider au démontage et le relire au montage (AC8).

## Décisions

- **D1 — La source est `company.name`, jamais une recomposition.** C'est le champ que les Réglages corrigent, que le PDF imprime, et qui est déjà « Prénom Nom » pour une personne physique (constats 5 et 6). `orgType`, `firstName` et `lastName` ne sont pas lus. *Écart à la lettre de la décision 1 de l'issue (« prénom et nom »)* : le résultat est le même pour tout compte créé par l'assistant, et il suit les corrections ; recomposer ferait diverger l'en-tête du PDF et des Réglages (limite #1227). À signaler au Project Lead (Q2).
- **D2 — Un nom vide (rogné) n'affiche rien** et laisse le titre de l'onglet inchangé. Le serveur interdit le nom vide ; c'est une défense, pas un cas métier.
- **D3 — Le nom sous le logo est dans le lien du logo** : `<a href="/">` contient « Kesh » puis le nom, l'un sous l'autre (« sous le logo », décision 1). Le nom accessible du lien devient « Kesh <nom> » : un lecteur d'écran lit le logo et la comptabilité ensemble (décision 6). Le texte complet est dans le DOM (la coupe est de la CSS : `truncate`), donc lu en entier ; le survol porte `title` = le nom complet.
- **D4 — Gabarit : `max-w-56` (14 rem, la largeur de l'`<aside>`), `truncate`, `text-xs`, couleur pleine `text-text` (pas `text-text-muted` : c'est un repère de sécurité, et le contraste se contrôle).** L'en-tête garde 56 px : « Kesh » en `text-xl` (28 px de ligne) + une ligne `text-xs` de 16 px = 44 px ≤ 56 px.
- **D5 — Titre de l'onglet : `Kesh — <nom> — <page>`**, la page étant le titre de la page débarrassé de son suffixe ou préfixe « Kesh » ; une page sans titre propre (`Kesh` seul, ou vide) donne `Kesh — <nom>`, **la forme de l'issue** (« Kesh — Dupont Sàrl »). **Le nom est placé avant la page** parce que les navigateurs coupent un onglet par la fin : avec « Écritures comptables - Kesh — Dupont Sàrl », deux onglets étroits seraient identiques, ce que la story veut éviter. Réversible : une seule fonction (`titleWithCompany`). À confirmer (Q3).
- **D6 — Le titre est décoré à un seul endroit, sans toucher les 50 pages (DRY).** Un `MutationObserver` sur `document.head` (module `tab-title.ts`, démarré par le layout `(app)` tant qu'un nom est connu) réécrit `document.title` à chaque changement venu d'une page, en mémorisant le dernier titre brut et le dernier titre écrit par lui (la réécriture qu'il fait lui-même est ignorée : pas de boucle). Au démontage ou au changement de nom, il **restaure le titre brut**. Écartées : (a) un composant `<PageTitle>` à substituer dans 50 fichiers (conflits avec les autres stories de la release, et le 51e `<title>` ajouté plus tard l'oublierait) ; (b) surcharger le setter `document.title` (global, fragile) ; (c) un `<title>` dans le layout (battu par celui de la page).
- **D7 — Les pages hors `(app)` (login, setup, onboarding, erreur) ne portent pas le nom** : pas de session, pas de société connue.
- **D8 — Les Réglages mettent le store à jour** : `adoptCompany` (`settings/+page.svelte:217`, point de passage de **toutes** les écritures de la société : nom, adresse, e-mail, contacts, devises) appelle `companyIdentity.set(company.name)`. Sans cela, le nom corrigé n'apparaîtrait qu'au rechargement, dans l'onglet même où on vient de le corriger. La décision 4 de l'issue (« au chargement suivant ») est la **borne basse** ; la mise à jour immédiate dans l'onglet qui corrige coûte une ligne. Les **autres** onglets suivent à leur prochain chargement.
- **D9 — Un échec de lecture de la société n'abîme rien** : pas de toast, pas de redirection ; le nom reste absent, le titre inchangé, `console.error` comme `onboarding guard` (`+layout.ts:21`). La lecture se rejoue au prochain montage du layout.
- **D10 — Aucune clé i18n nouvelle** : le nom est une donnée, « Kesh » est le nom du produit. `sitesTotal` de `i18n-keys.test.ts` ne bouge pas (à recompter depuis la source quand même).
- **D11 — Pas de recherche élargie** : aucun autre écran n'affiche la société ; le pied de page, la page d'accueil et l'en-tête des PDF restent inchangés.

## Critères d'acceptation

- **AC1 — Société.** Connecté à une société (« Dupont Sàrl »), l'en-tête montre sous « Kesh » le texte `Dupont Sàrl` (`data-testid="company-name"`), dans le lien du logo.
- **AC2 — Personne physique.** Pour une société créée comme personne physique (assistant : prénom « Marie », nom « Dupont »), l'en-tête montre `Marie Dupont` (le `name` rendu par le serveur, D1) ; `orgType`, `firstName`, `lastName` ne sont pas lus par le code de cette story.
- **AC3 — Onglet.** Sur chaque page de l'application, `document.title` vaut `Kesh — <nom> — <page>` (D5) ; sur une page sans titre propre, `Kesh — <nom>`. La navigation d'une page à l'autre met le titre à jour sans écrire deux fois le nom (`Kesh — Dupont Sàrl — Dupont Sàrl` est un défaut). Aucun des 50 fichiers de page n'est modifié.
- **AC4 — Nom long.** Un nom de 70 caractères (le maximum, `NAME_MAX`) est coupé par une ellipse à `max-w-56` ; le texte complet est dans le DOM et dans l'attribut `title` ; l'en-tête garde sa hauteur de 56 px, le champ de recherche et le menu profil ne se déplacent pas de plus que le logo n'a grandi, et la largeur de l'`<aside>` (14 rem) est inchangée.
- **AC5 — Groupes repliés.** Replier ou déplier les trois groupes du menu (Quotidien, Mensuel, Administration) ne change ni la présence, ni la position, ni le texte du nom.
- **AC6 — Suivi des Réglages.** Corrigé dans *Administration → Paramètres*, le nom change **dans l'en-tête et le titre de l'onglet sans rechargement**, et reste après un rechargement (D8). Un autre onglet le prend au chargement suivant.
- **AC7 — Mode démonstration.** `DemoBanner` est inchangé ; le nom de la société de démonstration (« Démo SA » ou sa traduction) s'affiche comme les autres, bandeau compris.
- **AC8 — Pas de nom d'une autre société.** Après « Réinitialiser la démonstration » puis un retour dans l'application, l'en-tête ne montre jamais le nom de la société d'avant (store vidé au démontage du layout, relu au montage). Pendant la lecture, ou après son échec, il n'affiche rien plutôt qu'un nom périmé.
- **AC9 — Accessibilité.** Le nom accessible du lien du logo est « Kesh <nom> » ; le nom complet est lu même quand l'affichage est coupé ; aucun texte caché ni `aria-label` en doublon (le texte visible suffit) ; le contraste du nom répond au seuil AA sur le fond de l'en-tête (`bg-surface`), contrôlé par le test d'accessibilité existant.
- **AC10 — Échec de lecture.** Quand `GET /companies/current` échoue (réseau, 5xx), l'application reste utilisable : ni toast, ni redirection, en-tête et titre comme avant la story (D9).
- **AC11 — Manuel et notes.** La phrase « Barre supérieure » du manuel d'utilisateur (français) dit le nom et le titre de l'onglet ; la Note de version est écrite.
- **AC12 — Hors périmètre intact.** Aucune migration, aucun fichier de `crates/`, aucune route, aucun PDF, aucun fichier d'i18n modifié ; aucune des 50 pages ne l'est.

## Tâches

1. **Étape 0** — Relire au code les constats 1 à 10 à `1e23edb55` (ou `main` du jour) ; en particulier `grep -rl "<title>" frontend/src | wc -l` (50), `(app)/+layout.svelte:311-314`, `settings/+page.svelte:217`, et qu'aucun `firstName` n'est lu par un écran de Réglages. Une prémisse démentie va au Change Log et à l'orchestrateur avant d'écrire.
2. **Store** `frontend/src/lib/app/stores/company-identity.svelte.ts` (patron `app-version.svelte.ts`) :
   - `companyIdentity.name: string | null` (lecture réactive) ;
   - `load()` : `fetchCompanyCurrent()` (existant, `settings.api.ts:26` — ne pas recréer l'appel), nom rogné, vide → `null` ; échec → `null` + `console.error`, **sans propager** (D9) ; un **numéro de séquence** fait que `set()` ou `clear()` survenus pendant le vol rendent la réponse tardive sans effet ;
   - `set(name)` (rogne, vide → `null`) ; `clear()`.
3. **Titre** `frontend/src/lib/shared/utils/tab-title.ts` : `titleWithCompany(raw, name)` (pure, idempotente : si `raw` commence déjà par `Kesh — <nom>`, rendue telle quelle) et `watchTabTitle(name, doc = document): () => void` (D6 ; l'appel de retour déconnecte l'observateur et restaure le titre brut). `SvelteKit` n'a pas de crochet de titre global : ne pas en chercher un.
4. **Composant** `frontend/src/lib/shared/components/AppBrand.svelte` (prop `name: string | null`) : le lien du logo et le nom, D3 et D4. Extrait du layout pour le tester seul ; JSDoc.
5. **Layout** `(app)/+layout.svelte` : remplacer le bloc `<!-- Logo -->` par `<AppBrand name={companyIdentity.name} />` ; un `$effect` qui, une fois `authState.isAuthenticated`, appelle `companyIdentity.load()` et rend `companyIdentity.clear` en nettoyage (AC8) ; un second `$effect` qui lit `companyIdentity.name` et, s'il est non nul, rend `watchTabTitle(name)` en nettoyage. Pas d'autre changement du layout (ne pas toucher aux groupes du menu : la story `v0200-menu-sous-groupes` s'y trouve, voir Fichiers communs).
6. **Réglages** `settings/+page.svelte` : `adoptCompany` appelle `companyIdentity.set(company.name)` **quand la société est adoptée** (après `if (!decision.adopt) return false;`), **pas** avant : une réponse périmée refusée ne doit pas écrire le store.
7. **Tests Vitest** (liste ci-dessous) puis **E2E** `frontend/tests/e2e/company-name-header.spec.ts`.
8. **Mutations** M1 à M9 (liste ci-dessous) : chacune jouée, rouge nommé, restaurée (`git checkout -- <f>` **puis `touch <f>`**), un journal par mutation dans `kesh-gate-logs/s1582/mut/`.
9. **Recette visuelle** (liste ci-dessous), captures dans `kesh-gate-logs/s1582/captures/`, jointes au Dev Agent Record et au prompt de revue.
10. **Manuel** `docs/manual/fr/user-manual.tex:150-152` (jamais le PDF : § *Fichiers communs*). **Notes de version** (ci-dessous). `README.md` : rien, aucun epic ni feuille de route ne change ; à vérifier au commit.
11. **Gates** (CLAUDE.md § *Test Locally First*) : frontend seul — `scripts/mem-guard.sh npm --prefix frontend run check`, `lint-i18n-ownership`, `test:unit`, `build` ; E2E complet au dernier commit de code, montage de `docs/testing.md` § *Prérequis Playwright local*, jugé contre § *Les échecs attendus* (sept KF-029). Pas de gate Rust : aucun fichier de `crates/` ne change (le déclarer, preuve : `git diff --stat main -- crates/` vide). Gate en trois voies de `docs/agents-orchestration.md` § 5.1 (voie frontend ; E2E en voie base et Rust).

## Tests à écrire

**Vitest** (écrits **avant** le code, rouges d'abord).

- `lib/shared/utils/tab-title.test.ts` — `titleWithCompany` : table `Accueil - Kesh` → `Kesh — N — Accueil` ; `Écritures comptables - Kesh` ; `Kesh — Rapports` (préfixe) → `Kesh — N — Rapports` ; `Kesh` et `` → `Kesh — N` ; un titre sans « Kesh » (`Erreur 404`) → `Kesh — N — Erreur 404` ; idempotence (`titleWithCompany(titleWithCompany(x))` = un seul nom) ; nom contenant « — » et « Kesh » (`Atelier Kesh SA`) ; nom jamais interprété comme motif (caractères `$`, `(`). `watchTabTitle` (jsdom, `vi.waitFor`) : un `document.title = 'Contacts — Kesh'` est décoré ; un second changement l'est aussi ; **le nombre de réécritures est borné** (pas de boucle : compteur sur un espion du setter ou sur le nombre de rappels de l'observateur) ; la fonction de retour restaure le titre brut et déconnecte (un changement ultérieur n'est plus décoré) ; un changement de nom (nettoyage puis nouvel appel) ne laisse pas deux noms.
- `lib/app/stores/company-identity.svelte.test.ts` — lecture réussie (rognée) ; nom vide ou d'espaces → `null` ; échec (rejet) → `null`, aucune exception, `console.error` appelé ; **`set()` pendant une lecture en vol l'emporte sur la réponse tardive** ; `clear()` pendant une lecture en vol aussi ; `load()` rejoué remplace le nom.
- `lib/shared/components/AppBrand.test.ts` — `name="Dupont Sàrl"` : `getByRole('link', { name: 'Kesh Dupont Sàrl' })` ; `data-testid="company-name"` porte `title="Dupont Sàrl"` ; `name=null` : lien nommé `Kesh` seul, aucun `company-name` (sonde positive d'abord : le lien existe) ; nom de 70 caractères : texte complet dans le DOM, classe `truncate` et `max-w-56` présentes sur l'élément, `title` complet ; nom avec `<b>x</b>` rendu en texte, non interprété.
- `settings/settings-page.test.ts` (existant) — **un test de plus** : après l'enregistrement d'un nom réussi, `companyIdentity.name` vaut le nom rendu par le serveur ; après une réponse périmée refusée par `adoptionDecision`, il reste inchangé (le mock de `updateCompanyName` rend une version plus ancienne). Étendre le mock existant, ne pas en dupliquer.
- Garde `i18n-keys.test.ts` : `sitesTotal` recompté depuis la source (attendu : inchangé).

**Playwright** `company-name-header.spec.ts` (sélecteurs `data-testid`, garde KF-043 ; `seedTestState('with-company')` pour la société de la CI, l'assistant par l'API pour la personne physique, motif de `onboarding-motif.spec.ts:25-47`) :

1. **Société** : après connexion, `company-name` visible et égal à `company.name` lu par `GET /api/v1/companies/current` (jamais une constante écrite deux fois) ; `await expect(page).toHaveTitle(...)` sur `/`, puis après navigation vers `/contacts` et `/settings` ; le nom n'apparaît qu'une fois dans le titre.
2. **Suivi des Réglages (AC6)** : corriger le nom dans *Paramètres* (motif de `company-name.spec.ts`, restauration du nom en `finally` — les tests partagent une société) ; l'en-tête et le titre changent sans rechargement ; après `page.reload()`, ils portent toujours le nouveau nom.
3. **Nom long (AC4)** : nom de 70 caractères posé par l'API ; `scrollWidth > clientWidth` sur `company-name` (coupé), attribut `title` = nom complet, hauteur de l'en-tête = 56 px, largeur de l'`<aside>` inchangée ; sonde positive avant (un nom court n'est pas coupé).
4. **Groupes repliés (AC5)** : replier puis déplier chacun des trois `<summary>` ; boîte du nom identique à chaque fois.
5. **Personne physique (AC2)** : `fresh` → assistant par l'API (org-type `Independant`, coordonnées avec `firstName` « Marie », `lastName` « Dupont »), reprise jusqu'à l'application ; en-tête `Marie Dupont`, titre `Kesh — Marie Dupont — …`. *Si l'assistant par l'API exige trop d'étapes pour arriver à l'application (finalisation, exercice), le dev l'écrit dans le Dev Agent Record et couvre ce cas en recette visuelle + Vitest : une raison écrite, pas un trou silencieux.*
6. **Mode démo (AC7)** : `fresh` puis `seed-demo` ; le bandeau et le nom de la société de démonstration sont tous deux visibles ; après « Réinitialiser la démonstration » et retour dans l'application, **l'en-tête ne montre pas « Démo SA »** (AC8).
7. **Accessibilité (AC9)** : le test d'accessibilité existant (axe, `tests/e2e/*a11y*`) vert sur une page de l'application avec le nom affiché (contraste) ; `getByRole('link', { name: /^Kesh .+/ })`.
8. **Échec de lecture (AC10)** : `page.route('**/api/v1/companies/current', r => r.fulfill({ status: 500 }))` avant le chargement ; la page d'accueil reste utilisable, `company-name` absent, aucun toast d'erreur, titre sans nom (sonde positive : le lien du logo est visible).

**Pourquoi un E2E** : Vitest simule l'API, et seul le navigateur lit un `<title>` réel, observe un `MutationObserver` réel et mesure la coupe.

## Mutations (chacune tuée par un test nommé ; motif vérifié unique avant écriture)

| N° | Mutation | Test qui la tue |
|---|---|---|
| M1 | `titleWithCompany` rend `raw` inchangé | `tab-title.test.ts` (table) + E2E 1 |
| M2 | `titleWithCompany` met la page avant le nom (`<page> — Kesh — <nom>`) | `tab-title.test.ts` (forme exacte) |
| M3 | `watchTabTitle` oublie d'ignorer sa propre réécriture | `tab-title.test.ts` (nombre de réécritures borné) |
| M4 | nettoyage de `watchTabTitle` sans restauration du titre brut | `tab-title.test.ts` (restauration) |
| M5 | `AppBrand` sans `truncate` | `AppBrand.test.ts` + E2E 3 (`scrollWidth`) |
| M6 | `AppBrand` sans attribut `title` | `AppBrand.test.ts` + E2E 3 |
| M7 | `adoptCompany` appelle `companyIdentity.set` **avant** `adoptionDecision` | `settings-page.test.ts` (réponse périmée) |
| M8 | `adoptCompany` n'appelle pas `companyIdentity.set` | `settings-page.test.ts` + E2E 2 |
| M9 | layout sans `clear()` au démontage | E2E 6 (retour après réinitialisation) |
| M10 | `load()` sans garde de séquence | `company-identity.svelte.test.ts` |

## Recette visuelle (CLAUDE.md § *Recette visuelle* — thème clair seulement, base de démonstration, `kesh-gate-logs/s1582/captures/`)

Une capture de l'en-tête (et de la fenêtre pour l'onglet) par état : (1) société ordinaire ; (2) personne physique ; (3) nom de 70 caractères coupé, **avec le survol** (infobulle visible) ; (4) trois groupes repliés, puis trois dépliés ; (5) titre de l'onglet — la barre d'onglets du navigateur, **sur deux onglets de deux sociétés** (le cas de l'issue) et sur une page à titre long en onglet étroit (le nom doit rester visible, D5) ; (6) mode démonstration, bandeau compris ; (7) lecture en échec (pas de nom, pas de toast). Chaque capture est regardée (le dev écrit ce qu'il y voit : alignement du nom sur le logo, absence de déformation du champ de recherche).

## Fichiers touchés et modules (règle de splitting)

`lib/app/stores/` (1 nouveau), `lib/shared/utils/` (1 nouveau), `lib/shared/components/` (1 nouveau), `routes/(app)/+layout.svelte`, `routes/(app)/settings/+page.svelte`, `docs/manual/fr/user-manual.tex`, tests. Au grain du domaine : **frontend « shared/app » et « settings » : deux modules** ≤ 5 ; pas de crate. Pas de splitting.

## Fichiers communs et coordination

- **`(app)/+layout.svelte`** : la story `v0200-menu-sous-groupes` (#1253) le modifie aussi (menu Administration en sous-groupes), à fusionner **en dernier** d'après `sprint-status.yaml`. Les deux ne touchent pas les mêmes lignes (en-tête `:311-314` ici, `navGroups` là-bas) : le second arrivé reprend `main` et résout en gardant les deux intentions. À vérifier avant la fusion (`git diff main -- '(app)/+layout.svelte'`).
- **`settings/+page.svelte`** : la fiche n'ajoute qu'une ligne dans `adoptCompany`.
- **`docs/manual/fr/user-manual.tex`** : seul le `.tex` change ; le PDF se régénère à la release (`make fr`). **DE/IT/EN** : `docs/manual/{de,en,it}/` ne contiennent que `README.md` — aucune phrase à mettre à jour (`ls docs/manual/{de,en,it}`, à recontrôler).
- **`CHANGELOG.md`** : non écrit dans la story ; la Note de version ci-dessous est assemblée à l'intégration.
- **`sprint-status.yaml`** : clé `1582-nom-comptabilite` au bloc « Stories hors epic de la 0.20.0 ».
- Aucun fichier d'i18n, aucune migration, aucun `crates/` : pas de conflit avec l'Epic 53.

## Manuel

- `docs/manual/fr/user-manual.tex:150-152`, puce « Barre supérieure » : « le logo, **sous lequel s'affiche le nom de la comptabilité ouverte** (la raison sociale, ou le prénom et le nom d'une personne physique ; le nom complet apparaît au survol quand il est coupé), un champ de recherche (à venir) et le menu utilisateur… ». Ajouter une phrase : **le titre de l'onglet du navigateur porte le même nom** (« Kesh — <nom> — <page> »), pour distinguer deux comptabilités ouvertes dans deux onglets ; **corrigé dans *Administration → Paramètres*, le nom change aussitôt dans cet onglet, et dans les autres à leur prochain chargement**.
- Le prompt de validation exige la lecture du **PDF** (`user-manual.pdf`), aplati avant de greper, pour ce que le manuel dit de l'en-tête et du titre de l'onglet.

## Note de version (v0.20.0, ou v0.21.0 si la story passe)

**Ajouté**

- **Le nom de la comptabilité ouverte s'affiche sous le logo et dans le titre de l'onglet** (refs [#1582](https://github.com/guycorbaz/kesh/issues/1582)) — la raison sociale de la société, ou le prénom et le nom d'une personne physique ; un nom long est coupé, le nom complet s'affiche au survol et un lecteur d'écran le lit avec le logo. Avec deux comptabilités ouvertes dans deux onglets, on voit sur lequel on travaille sans regarder l'adresse. Le nom corrigé dans *Administration → Paramètres* se met à jour aussitôt dans l'onglet où l'on corrige, et au chargement suivant dans les autres. Aucune migration, aucun changement de l'API.

## Dev Notes

- **Réutiliser, ne pas recréer** : `fetchCompanyCurrent()` (`settings.api.ts:26`) ; `app-version.svelte.ts` et `mode.svelte.ts` comme patrons de store ; `adoptionDecision` des Réglages ne change pas.
- **Svelte 5, runes** : le store lit `$state` ; l'`$effect` du titre dépend de `companyIdentity.name` seul et retourne sa fonction de nettoyage. `ssr = false` partout dans `(app)` : `document` existe, mais gardez `typeof document` dans `tab-title.ts` pour le test unitaire sous jsdom comme sous `vitest` Node si le motif du dépôt l'exige.
- **`MutationObserver`** : `{ childList: true, subtree: true, characterData: true }` sur `document.head` (Svelte met à jour `document.title`, ce qui modifie le nœud texte de l'élément `<title>`, ou en recrée un). `jsdom` le supporte ; les rappels sont asynchrones : les tests attendent (`vi.waitFor`).
- **Le titre d'une page peut être réécrit par elle-même** (changement de langue, données chargées) : chaque réécriture passe par l'observateur, qui redécore. Le titre qu'une page a *avant* le montage du layout est lu à l'appel de `watchTabTitle` (premier `apply()`).
- **Données réelles hors dépôt** (`docs/agents-orchestration.md` § 9) : les noms des fixtures et captures sont synthétiques (« Dupont Sàrl », « Marie Dupont »), jamais la société réelle de Guy ; la recette visuelle se prend sur une base de démonstration jetable, **jamais** sur la base du NAS ni sur `~/kesh/`.
- **Commits** : `refs #1582` seulement, aucun mot-clé de fermeture ; fermeture par la PR. Un commit par étape BMAD ; le dev pousse sa seule branche `story/1582-nom-comptabilite`.

## Dev Agent Record

*(à remplir au développement)*

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-25 — fiche écrite (bmad-create-story, Sonnet 5.5). Questions au Project Lead : Q1 (« menu replié » = groupes repliés), Q2 (D1 : `name` plutôt que recomposer prénom et nom), Q3 (D5 : le nom avant la page dans le titre de l'onglet). Validation P1 (Opus) à lancer.
