# Story 50-5 : une page « Sauvegarde et export » à onglets, au lieu de trois entrées du menu (#1250, #1248)

Status: drafted

*Fiche rédigée le 2026-11-24 par l'agent de rédaction de la 50-5 (Opus 5.5), par le workflow `bmad-create-story`, en
autonomie. Validation P1 (Sonnet) remédiée le 2026-11-24 ; validation P2 à lancer (Opus, prompt versionné à côté :
`50-5-page-sauvegarde-export-validation-prompt-p2.md`).*

Issues : **#1250** — `enhancement`, **P4**, milestone v0.20.0 — et **#1248** (composant d'onglets réutilisable,
`technical-debt`, **P4**), prérequis de #1250, **livré par cette story et fermé avec elle** (décision du Project Lead du
2026-11-24 ; le mot-clé de fermeture ne s'écrit que dans la PR). Ergonomie, **sans changement de fonctionnement**. Release
**0.20.0** (vendredi 2026-11-27 ; **coupure le vendredi 2026-11-27 au matin**, décision du Project Lead du 2026-11-24),
**Epic 50**, cinquième story (sous la borne de six). **Sous condition** (même décision) : si elle n'est pas close à la
coupure, elle sort de la release sans la retarder. Voie : **frontend** (plus les catalogues de `kesh-i18n` et les manuels).
**Aucune migration, aucun code Rust de production.** Base : `origin/epic/50-corrections` = `e5654fc00` (porte `main` avec
l'Epic 38 PayPal).

Le fichier d'epic (`epic-50-corrections.md`), `sprint-status.yaml` et la feuille de route de `README.md` (ligne
« v0.20.0 (prévu) ») sont tenus par l'orchestrateur : la story n'y touche pas.

Liées : #1249 (MCP), #1251 (Facturation), #1252 (Paramètres) — la série qui réutilisera le composant ; **#1253**
(sous-groupes du menu Administration, release ultérieure) — à ne pas gêner (D6).

## Story

En tant qu'administrateur de Kesh,
je veux trouver la sauvegarde de l'installation, sa restauration et l'export CSV d'une société **sur une seule page à
onglets**, avec en tête une phrase qui dit ce qui les distingue,
afin de ne plus chercher trois entrées du menu Administration ni confondre la sauvegarde restaurable avec l'export pour la
fiduciaire.

En tant que comptable (ou utilisateur en consultation),
je veux que cette page ne me montre que l'onglet d'export, comme aujourd'hui je ne vois que « Export global »,
afin que rien ne change de ce que j'ai le droit de faire.

## Ce qui se lit au code

Tout est lu à `e5654fc00`, dans le worktree `/home/gcorbaz/devel/kesh-50-5`. Chemins relatifs à `frontend/` sauf mention.

1. **Trois routes, trois écrans.**
   - `src/routes/(app)/admin/backup/+page.svelte:8-12` rend `AdminBackupPanel` ; garde `+page.ts:13-17` :
     `if (browser && authState.currentUser?.role !== 'Admin') { throw redirect(302, '/'); }`.
   - `src/routes/(app)/admin/restore/+page.svelte:6-10` rend `AdminRestorePanel` ; garde `+page.ts:13-17`, identique.
   - `src/routes/(app)/export/+page.svelte` porte **toute la logique dans la route** (`let exporting = $state(false)` `:15`,
     `startExport` `:36-52` qui appelle `downloadGlobalExport()` `:45`), son `<svelte:head>` `:55-57` et son `<h1>` `:61` ;
     `+page.ts:10` : `export const ssr = false;`, sans garde de rôle.
2. **Droits, côté serveur (inchangés par la story).** `crates/kesh-api/src/routes/exports.rs:34-37` : `GET
   /api/v1/exports/global.zip` — « Tous rôles authentifiés (AC #11 Consultation 200) ». La sauvegarde, l'import, le réglage
   et la liste des sauvegardes restent sous `require_admin_role` (`crates/kesh-api/src/lib.rs:376-406`, `:575`, relevé de la
   validation P1).
3. **Le menu.** `src/routes/(app)/+layout.svelte` :
   - `:137` (groupe Administration, `items`, visible de **tous** les rôles) :
     `{ i18nKey: 'nav-export-global', fallback: 'Export global', href: '/export' },` ;
   - `:162-176` (`adminOnly`) : `nav-admin-backup` → `/admin/backup`, `nav-admin-restore` → `/admin/restore` ;
   - `:238-245` `findGroupKeyForPath` compare `it.href === pathname` (la requête n'y entre pas) ;
   - `:297-300` `navTestid(href)` : `nav-link-` + chemin, `/` → `-` (donc `/backup-export` → `nav-link-backup-export`).
4. **Les en-têtes des panneaux.** `AdminBackupPanel.svelte:55-59` (`<svelte:head><title>`) et `:63` (`<h1 …
   data-testid="admin-backup-title">`) ; `AdminRestorePanel.svelte:304-308` et `:312` (idem, `admin-restore-title`). Sur une
   page à trois panneaux, chacun écrirait son `<title>` et son `<h1>`. `AutoBackupSection.svelte:318` et `:509` portent déjà
   des `<h2>` (« Sauvegarde automatique », sauvegardes du serveur).
5. **`AdminRestorePanel` sert aussi l'assistant.** `src/routes/onboarding/restore/+page.svelte:11` l'importe et le rend
   (`:29`) sous un lien de retour, sans titre propre : son `<h1>` et son `<title>` y sont **le** titre de la page.
6. **`AdminRestorePanel` agit à son démontage.** `:99` `onDestroy(() => …)` : lève le maintien de session et **termine la
   session si le compte rendu d'import était affiché** (en-tête du fichier, `:18-26`). Démonter le panneau à chaque changement
   d'onglet changerait donc le fonctionnement (D4).
7. **Une saisie non enregistrée vit dans l'onglet de sauvegarde.** `AutoBackupSection.svelte:58-60` (`enabled`,
   `intervalHours`, `retentionCount`), bouton « Enregistrer » `:370-375` (`save()` `:171`). Démonter le panneau la perdrait
   (D4). Seul `AutoBackupSection` écrit l'état du bandeau (`setBackupScheduleState`, `:110`, `:126`).
8. **Le bandeau de la sauvegarde automatique** renvoie à l'ancienne adresse : `BackupScheduleBanner.svelte:66` `<a
   href="/admin/backup" …>` avec le libellé `admin-backup-banner-link` (`:67`) = « Administration → Sauvegarde complète »
   (`crates/kesh-i18n/locales/fr-CH/messages.ftl:2410`, et `:2320` des trois autres catalogues). Son test épingle l'adresse :
   `BackupScheduleBanner.test.ts:73-75`.
9. **Kesh n'a pas de composant d'onglets** (#1248 : « Kesh n'a aucun composant d'onglets »). Trois pages écrivent leur
   `tablist` à la main : `routes/(app)/reports/+page.svelte:820-838` (clavier `:589-601`, adresse `?tab=` en `replaceState`
   `:608-614`, lecture au montage `:546-551`), `settings/email-templates/+page.svelte:307-330`,
   `invoices/due-dates/+page.svelte:491-495`. `src/lib/components/ui/` ne contient pas de `tabs/`.
10. **`bits-ui` fournit les onglets, et le dépôt enveloppe déjà `bits-ui`.** `package.json:46` : `"bits-ui": "^2.16.5"`
    (installé : 2.16.5). `ui/separator/separator.svelte:2` (`import { Separator as SeparatorPrimitive } from "bits-ui";`) et
    `ui/dialog/` (un fichier par partie, `index.ts` qui les réexporte) sont des enveloppes shadcn-svelte. Lu à la source
    (`frontend/node_modules/bits-ui/dist/bits/tabs/`, dépôt principal) :
    - `components/tabs.svelte` : `value = $bindable("")`, `onValueChange`, `loop = true`, `activationMode = "automatic"` ;
    - `tabs.svelte.js:75` `role: "tablist"` ; `:146-150` `aria-selected`, `aria-controls`, `tabindex` (0 sur l'actif, -1
      sinon, `:100-108`) ; `:119-121` en mode automatique, **le focus active l'onglet** (`onfocus` → `setValue`) ;
    - `:137` les touches passent à `RovingFocusGroup.handleKeydown` (`internal/roving-focus-group.js:32-70`) : suivant,
      précédent, `Home`, `End`, circulaire si `loop` ; **l'index part du nœud focalisé** (`items.indexOf(node)`, `:39`) ;
    - `:174-186` `TabsContent` : `role: "tabpanel"`, **`hidden` sur l'inactif** (`:178`), `aria-labelledby` ;
      `components/tabs-content.svelte` rend **toujours** son `<div>` et ses `children` : tous les contenus sont montés, les
      inactifs cachés.
11. **Un lien vers la même route ne remonte pas le composant** : le commentaire de `reports/+page.svelte:553-566` le dit,
    et qu'un `onMount` qui lit l'adresse reste alors muet. Le bandeau (point 8) est un tel lien quand on est déjà sur la page.
12. **L'utilisateur est connu avant la page.** `src/hooks.client.ts:20-22` (`await authState.hydrate()`) attend `authState.hydrate()` avant la première
    navigation (relevé P1) ; `src/routes/(app)/+layout.ts:10-12` redirige vers `/login` si `!authState.isAuthenticated`, qui
    vaut `_currentUser !== null` (`src/lib/app/stores/auth.svelte.ts:132`).
13. **Les simulations de `$app/state` du dépôt sont statiques** : `routes/(app)/reports/reports-page.test.ts:22-26`
    (`vi.mock('$app/state', () => ({ page: { url: new URL(…) } }))`). Aucun test ne simule une adresse qui change après le
    montage (`git grep -n "vi.mock('\$app/state'" -- frontend/src` : objets littéraux seulement).
14. **Gardes i18n qui nomment ces clés.**
    - `crates/kesh-i18n/src/loader.rs:1174-1190` (`les_chemins_cites_aux_catalogues_existent`, G3-bis) :
      `ENTREES_ADMINISTRATION: [&str; 14]` contient `nav-export-global`, `nav-admin-backup`, `nav-admin-restore` ; tout texte
      « Administration → X » d'un catalogue doit commencer par le libellé d'une de ces entrées.
    - `src/lib/shared/i18n-keys.test.ts:1356-1364` (`FAMILLES_RESOLUES['nav-']`) contient `export-global`, `admin-backup`,
      `admin-restore` ; `:1203` `sitesTotal: 3695` ; `:1441` `PREFIXES_A_COUVERTURE_CLOSE`.
    - `scripts/lint-i18n-ownership.js:102-105`, `:128-137` : dans `src/lib/features/<f>/`, une clé doit appartenir à `<f>`
      (`export` : préfixe `export-`) ; les fichiers de `src/routes/` et de `src/lib/components/` ne sont pas contrôlés.
15. **E2E qui passent par ces adresses** (`git grep -nE "admin/backup|admin/restore|'/export'" -- frontend/tests/e2e`) :
    `backup-schedule.spec.ts:149`, `:214-216` (comptable : `nav a[href="/admin/backup"]` absent, puis `toHaveURL('/')`),
    `:222`, `:275` ; `backup-restore-new-install.spec.ts:104`, `:282`, `:294` (`toHaveURL(/\/admin\/restore$/)`), `:315`,
    `:328`, `:389` ; `export-global.spec.ts:70` ; `sidebar-navigation.spec.ts:219`, `:226-227` (`ADMINISTRATION`).
    `BackupEnvNotice.svelte:16` (`data-testid="backup-env-notice"`) est rendu par les **deux** panneaux admin.
16. **Textes qui décrivent ces chemins — inventaire par le symptôme, sur tout le dépôt**, ensemble clos à `e5654fc00` :

    ```
    $ git grep -nE 'Sauvegarde complète|Restaurer / Importer|Export global|/admin/backup|/admin/restore|keshpath\{/export\}' \
        -- . ':!CHANGELOG.md' ':!_bmad-output' ':!docs/testing.md' ':!frontend/tests' ':!crates/kesh-i18n/locales'
    ```

    (résultats ramenés aux sites de **texte lu par une personne** ; les catalogues sont au point 8 et en D10, les E2E au
    point 15, le code au point 3 ; les commentaires de code au point 17) :

    ```
    .env.example:198                     (réglées dans Administration → Sauvegarde complète)
    README.md:52                         (Administration → Sauvegarde complète / Restaurer / Importer)
    docs/manual/fr/admin-manual.tex:1772 (\emph{Administration → Sauvegarde complète})
    docs/manual/fr/admin-manual.tex:1828 (idem)
    docs/manual/fr/admin-manual.tex:2325 (menu \textbf{Administration → Export global})
    docs/manual/fr/admin-manual.tex:2330 (Administration → Sauvegarde complète, \keshpath{/admin/backup})
    docs/manual/fr/admin-manual.tex:2370 (Administration → Export global)
    docs/manual/fr/admin-manual.tex:2400 (Administration → Restaurer / Importer, \keshpath{/admin/restore})
    docs/manual/fr/admin-manual.tex:2450 (Administration → Sauvegarde complète)
    docs/manual/fr/admin-manual.tex:2499 (idem)
    docs/manual/fr/admin-manual.tex:2509 (le bandeau, lien vers Administration → Sauvegarde complète)
    docs/manual/fr/admin-manual.tex:2552 (Administration → Restaurer / Importer)
    docs/manual/fr/admin-manual.tex:2678 (\paragraph{Export global.} — nom de la fonction, pas un chemin : inchangé)
    docs/manual/fr/admin-manual.tex:2948 (Administration → Sauvegarde complète)
    docs/manual/fr/admin-manual.tex:3821 (idem)
    docs/manual/fr/admin-manual.tex:4250-4251 (Administration → Sauvegarde complète / Restaurer, sur deux lignes)
    docs/manual/fr/user-manual.tex:4577 (\emph{Administration} → \emph{Export global})
    docs/user-guide/fr/getting-started.md:96 (Administration → Export global)
    ```

    Dix-sept sites à reprendre (`:2678` exclu). `website/` : aucun. Les manuels DE, IT et EN n'existent qu'en `README.md`
    (`docs/manual/{de,en,it}/`) : rien à traduire. Aucune capture des manuels ne montre ces pages. `CHANGELOG.md:1483` et
    `docs/testing.md:476` sont historiques : on ne les réécrit pas. Les **PDF** publiés (`admin-manual.pdf`,
    `user-manual.pdf`, régénérés pour la 0.19.1) disent l'ancien chemin, ce qui est exact pour la version qu'ils décrivent ;
    ils se régénèrent à la release (`CLAUDE.md`, point 4-bis) et la redirection de D2 couvre leurs liens d'ici là.
17. **Commentaires de code qui deviendront faux** (`git grep -nE "admin/(backup|restore)|/export\b|Restaurer /" --
    frontend/src`, relevé P1) : `routes/(app)/admin/mcp/+page.ts:2` (« copie de `admin/backup/+page.ts` »),
    `admin/mcp/admin-mcp-page.test.ts:4`, `AdminBackupPanel.svelte:4`, `:9`, `AdminRestorePanel.svelte:4`,
    `onboarding/restore/+page.ts:2`, `:9` (« Administration → Restaurer / Importer »), `onboarding/restore/+page.svelte:5`,
    `onboarding-restore-page.test.ts:2`, `(app)/+layout.svelte:58`, `:164`. Aucun test ni script n'énumère les trois
    `+page.svelte` retirés (P4, confirmée en P1) ; `crates/kesh-api/tests/textes_coherents.rs:1649`, `:1654` lit
    `AdminBackupPanel.svelte` par fragments de texte que la story ne change pas.

## Décisions

- **D1 — La page : `/backup-export`, titre « Sauvegarde et export ».** Une route neuve
  `src/routes/(app)/backup-export/+page.svelte` (+ `+page.ts`, `ssr = false`, **sans garde de rôle** : un comptable y a son
  onglet). *Écartées* : `/admin/backup` gardée comme page (un comptable n'a rien à faire sous `/admin/`, dont toutes les pages
  sont réservées à l'administrateur) ; `/export` gardée (le titre dirait « export » d'une page dont le premier onglet est la
  sauvegarde).
- **D2 — Les anciennes adresses redirigent vers l'onglet correspondant**, pour ne casser ni un favori ni le lien d'un PDF du
  manuel : `/admin/backup` → `/backup-export?onglet=sauvegarder`, `/admin/restore` → `?onglet=restaurer`, `/export` →
  `?onglet=exporter`. Chaque ancien `+page.ts` devient `export const ssr = false;` plus un `load()` qui lève
  `redirect(301, …)`, **sans condition de rôle** (la page cible filtre ses onglets, D5) ; les anciens `+page.svelte` sont
  retirés, ou réduits à un fichier vide si SvelteKit l'exige (prémisse P1). Conséquence assumée : un comptable qui tape
  `/admin/backup` n'est plus renvoyé à l'accueil mais arrive sur l'onglet d'export (l'adresse garde `?onglet=sauvegarder`,
  D5) — il ne voit toujours ni la sauvegarde ni la restauration. *Écartée* : redirection côté serveur (Axum sert le même
  `index.html` à toute adresse inconnue, `lib.rs:279-280`).
- **D3 — Un composant d'onglets réutilisable qui enveloppe `bits-ui` `Tabs`** (point 10), dans `src/lib/components/ui/tabs/`,
  à la forme de `ui/dialog/` et `ui/separator/` :
  - **les parties stylées** : `tabs.svelte` (`Root`), `tabs-list.svelte`, `tabs-trigger.svelte`, `tabs-content.svelte`,
    réexportées par `index.ts` ; chacune passe `restProps` et `class` (`cn`) au primitif ; jetons du thème seulement
    (`border-border`, `border-primary`, `text-primary`, `text-text-muted`, `hover:text-text`), contrôlés par
    `components/ui/jetons-non-definis.test.ts`. **`bits-ui` fournit** les rôles ARIA, `aria-selected`, `aria-controls`,
    `aria-labelledby`, le tabindex itinérant, les flèches, Origine, Fin, la boucle, la sélection qui suit le focus et
    `hidden` sur le panneau inactif : la story **n'écrit ni ne reteste** ces comportements (au plus un test de fumée qui
    constate que l'enveloppe les transmet) ;
  - **`url-tabs.svelte`** (exporté `UrlTabs`), le seul code neuf à prouver, qui ajoute ce que #1248 demande en plus :
    - **onglet porté par l'adresse** : paramètre `onglet` (nom fixé par #1248 ; propriété `param`, défaut `'onglet'`). Le
      `value` passé à `Tabs.Root` est **dérivé de `page.url`** (`$app/state`), jamais lu une seule fois au montage (point 11) ;
      `onValueChange(v)` écrit l'adresse par `goto(…, { replaceState: true, keepFocus: true, noScroll: true })`, en gardant
      les autres paramètres. Entre le geste et la fin du `goto`, `bits-ui` affiche déjà la sélection (son `value` local),
      puis la dérivation la confirme ; comme `bits-ui` calcule la touche suivante depuis l'onglet **focalisé** (point 10),
      deux flèches rapides ne se perdent pas. Pas de `$effect` qui relise `page.url` pour écrire (la boucle que
      `reports/+page.svelte:543-545` écarte) ;
    - **onglets filtrables par rôle** : `tabs: { id: string; label: string; visible?: boolean }[]` ; un onglet invisible
      ne rend **ni** déclencheur **ni** panneau ; un `onglet` inconnu, invisible ou absent donne le **premier onglet
      visible** (sans réécrire l'adresse, D5) ; aucun onglet visible : rien ne se rend ;
    - **montage paresseux** du contenu : l'élément `tabpanel` (`Tabs.Content`) est **toujours rendu** pour chaque onglet
      visible (donc `aria-controls` vise toujours un élément existant) ; son **contenu** ne se monte qu'à la première
      ouverture (`{#if vus.has(id)}`, `SvelteSet`), puis reste monté, caché par le `hidden` de `bits-ui` (D4) ;
    - autres propriétés : `label` (nom accessible de la liste), `idPrefix` (`data-testid` : `<idPrefix>-tab-<id>`,
      `<idPrefix>-panel-<id>`), snippet `panel(id)`. Aucun texte propre (pas de clé i18n).
  *Écartées* : un composant écrit à la main (motif de la fiche initiale : il recodait l'ARIA, le clavier et `hidden` que
  `bits-ui` fournit déjà, avec des tests et des mutations de comportement de bibliothèque — V1-2) ; recopier le `tablist` de
  `reports` (quatrième copie). La migration de `reports`, `email-templates` et `due-dates` vers le composant **n'est pas**
  dans le périmètre (elle changerait `?tab=` en `?onglet=` sur `reports`, dont des liens dépendent) : **issue de dette
  décidée** par le Project Lead (`technical-debt`, P4), ouverte par l'orchestrateur le 2026-11-25.
- **D4 — Les panneaux restent montés une fois ouverts.** Démonter `AdminRestorePanel` en changeant d'onglet terminerait la
  session après un import réussi (point 6) et abandonnerait un import en cours ; démonter `AdminBackupPanel` perdrait le
  réglage non enregistré (point 7). Un import en cours dans un onglet caché continue comme aujourd'hui quand on regarde une
  autre partie de la page ; un seul panneau écrit l'état du bandeau (point 7). Le montage paresseux évite que l'onglet
  d'export d'un administrateur lance les lectures de la sauvegarde automatique avant qu'on ouvre l'onglet. *Écartées* : tout
  monter dès l'arrivée (comportement par défaut de `bits-ui` : trois chargements pour un onglet regardé) ; démonter
  l'inactif. Contrepartie d'outillage : `backup-env-notice` peut se trouver deux fois dans le DOM (point 15) — *Dev Notes*.
- **D5 — Onglets, ordre et droits.** Ordre de #1250 : **Sauvegarder** (`sauvegarder`, `AdminBackupPanel`), **Restaurer ou
  importer** (`restaurer`, `AdminRestorePanel`), **Exporter en CSV** (`exporter`, panneau d'export, D8). Les deux premiers :
  `visible: role === 'Admin'` ; le troisième : visible de tout utilisateur connecté (Admin, Comptable, Consultation), comme
  la route serveur (point 2) et l'entrée de menu d'aujourd'hui (point 3). Un administrateur arrive sur « Sauvegarder », un
  autre rôle sur « Exporter en CSV ». La liste d'onglets s'affiche **même quand un seul onglet est visible** (le comptable
  voit où il est ; *écartée* : la cacher, une variante de plus). Pas de réécriture de l'adresse quand l'onglet demandé n'est
  pas permis (*écartée* : un `$effect` qui écrit l'adresse sans geste de l'utilisateur ; l'adresse trompeuse qui en résulte
  n'expose rien, le serveur restant l'autorité, point 2). L'utilisateur étant connu avant toute page (point 12), aucun
  instant ne monte un panneau admin pour un non-Admin. La garde d'écran des anciennes routes est remplacée par le filtre des
  onglets (même garantie).
- **D6 — Le menu : une entrée au lieu de trois.** `nav-backup-export` → `/backup-export`, libellé « Sauvegarde et export »,
  dans les `items` du groupe Administration (visible de tous les rôles), **à la place** de « Export global » (`:137`) ; les
  deux entrées `adminOnly` (`:162-176`) sont retirées. Pour **#1253** : rien de neuf dans la structure du menu (pas de
  sous-groupe, pas de nouvelle variante de `NavItem`) ; les inventaires des points 16 et 17 lui servent de liste des sites à
  repasser (« Administration → Système → Sauvegarde et export ») ; G3-bis (`loader.rs`) contrôlera ses chemins comme les
  nôtres. **Note pour #1253** : cette entrée est visible de **tous** les rôles, alors que les autres entrées prévues pour
  « Système » sont réservées à l'administrateur ; le sous-groupe sera mixte, ce que #1253 devra traiter.
- **D7 — Pas d'avertissement « modifications non enregistrées »** au changement d'onglet, et **#1248 se ferme avec cette
  story** (décision du Project Lead du 2026-11-24) : avec D4, rien ne se perd en changeant d'onglet, et cette page n'a qu'un
  formulaire, à bouton propre. Si une page de la série (#1251, #1252, à formulaires) en a un jour besoin, le point
  d'accroche est **`beforeNavigate`** (`$app/navigation`), qui voit tout `goto` — clic, lien du bandeau, retour arrière — et
  peut l'annuler ; un rappel au clic ne verrait pas les deux derniers. La story n'écrit rien de cela (aucun appelant).
  **Ordre de la série** : #1248 désignait la page MCP (#1249) comme pilote ; c'est cette story qui le devient, parce que
  #1250 est la seule de la série retenue pour la 0.20.0 (#1249, #1251, #1252 sont au milestone « Plus tard ») et que sa page
  n'a pas de formulaire multiple (le composant s'y éprouve sans la question de l'avertissement).
- **D8 — L'export quitte la route pour un composant de feature** : `src/lib/features/export/GlobalExportPanel.svelte`
  (la logique de `export/+page.svelte:1-53` et son balisage, **mot pour mot**, sauf le `<svelte:head>` retiré et le `<h1>`
  devenu `<h2>`), testable et soumis au lint d'ownership (`export-global-*` appartient bien à `export`). La route `/export`
  ne garde que sa redirection (D2).
- **D9 — Un seul `<h1>` et un seul `<title>` par page, et une hiérarchie qui descend.** La page porte `<h1>` « Sauvegarde
  et export », la phrase de #1250 et `<title>` « Sauvegarde et export – Kesh ». Dans la page, les titres des panneaux
  deviennent des `<h2>` (texte et `data-testid` inchangés), leurs `<svelte:head>` disparaissent, et les deux `<h2>`
  d'`AutoBackupSection` (`:318`, `:509`) passent `<h3>` (`data-testid` inchangés). `AdminBackupPanel` et
  `GlobalExportPanel` ne servent qu'ici : changement direct. `AdminRestorePanel` sert aussi l'assistant (point 5) : il
  reçoit une propriété `embedded = false` ; `embedded` vrai rend un `<h2>` et aucun `<title>` ; faux (l'assistant) garde
  exactement le rendu d'aujourd'hui. *Écartée* : donner un titre propre à la page de l'assistant (hors périmètre).
- **D10 — Textes (quatre locales).** Clés neuves, préfixe `backup-export-` (page, hors `features/` : pas de propriétaire au
  lint) et une clé de menu :

  | Clé | fr-CH | de-CH | it-CH | en-CH |
  |---|---|---|---|---|
  | `nav-backup-export` | Sauvegarde et export | Sicherung und Export | Backup ed esportazione | Backup and export |
  | `backup-export-title` | Sauvegarde et export | Sicherung und Export | Backup ed esportazione | Backup and export |
  | `backup-export-intro` | La sauvegarde (.keshbackup) couvre toute l'installation et se restaure dans Kesh ; l'export produit des fichiers CSV lisibles d'une société, pour une fiduciaire ou un autre logiciel. | Die Sicherung (.keshbackup) umfasst die ganze Installation und lässt sich in Kesh wiederherstellen; der Export erzeugt lesbare CSV-Dateien einer Firma, für eine Treuhand oder eine andere Software. | Il backup (.keshbackup) comprende l'intera installazione e si ripristina in Kesh; l'esportazione produce file CSV leggibili di una società, per una fiduciaria o un altro software. | The backup (.keshbackup) covers the whole installation and can be restored in Kesh; the export produces readable CSV files for one company, for a fiduciary or another software. |
  | `backup-export-tab-backup` | Sauvegarder | Sichern | Esegui il backup | Back up |
  | `backup-export-tab-restore` | Restaurer ou importer | Wiederherstellen oder importieren | Ripristina o importa | Restore or import |
  | `backup-export-tab-export` | Exporter en CSV | Als CSV exportieren | Esporta in CSV | Export as CSV |

  Vocabulaire selon `docs/i18n-glossaire.md:158` ; les quatre libellés d'onglet sont des verbes dans chaque locale.
  `admin-backup-banner-link` change de **valeur** (pas de clé) : « Administration → Sauvegarde et export » et ses trois
  traductions, exigé par G3-bis. Les clés `nav-export-global`, `nav-admin-backup`, `nav-admin-restore` sont **retirées** des
  quatre catalogues (plus aucun appelant ; la parité reste égale ; `dette-parite-connue.txt` ne les cite pas).
  `backup-export-` entre dans `PREFIXES_A_COUVERTURE_CLOSE` (ses cinq clés sont toutes demandées) ; les trois
  `backup-export-tab-*` vivent dans une table (`labelKey`) : famille `backup-export-` ajoutée à `FAMILLES_RESOLUES`.

## Critères d'acceptation

1. **AC1 — Menu.** Pour un Admin, le groupe Administration porte **une** entrée « Sauvegarde et export »
   (`data-testid="nav-link-backup-export"`, `href="/backup-export"`) et **aucune** des trois anciennes (`nav-link-export`,
   `nav-link-admin-backup`, `nav-link-admin-restore` : 0 élément, après la sonde positive que l'entrée neuve est là). Pour un
   Comptable, la même entrée est présente. Ouvrir `/backup-export` déplie le groupe Administration.
2. **AC2 — Page, Admin.** `/backup-export` rend un seul `<h1>` « Sauvegarde et export », la phrase `backup-export-intro`,
   puis une `tablist` de **trois** onglets dans l'ordre Sauvegarder, Restaurer ou importer, Exporter en CSV ; sans paramètre,
   « Sauvegarder » est sélectionné (`aria-selected="true"`) et son panneau montre `admin-backup-panel`. `document.title`
   vaut « Sauvegarde et export – Kesh ». Sous le `<h1>`, les titres de panneau sont des `<h2>` et ceux
   d'`AutoBackupSection` des `<h3>`.
3. **AC3 — Page, autres rôles.** Pour un Comptable et pour un utilisateur Consultation, la `tablist` ne porte **qu'un**
   onglet, « Exporter en CSV », sélectionné ; `admin-backup-panel` et `admin-restore-panel` ne sont **pas** dans le DOM, même
   avec `?onglet=sauvegarder` ou `?onglet=restaurer` (sonde positive : `export-global-start` présent).
4. **AC4 — Adresse.** Cliquer un onglet écrit `?onglet=<id>` sans empiler d'entrée d'historique ; recharger la page garde
   l'onglet ; ouvrir `/backup-export?onglet=restaurer` sélectionne « Restaurer ou importer » ; un `onglet` inconnu donne le
   premier onglet visible ; **un changement de l'adresse après le montage change l'onglet sans remonter la page** — en
   particulier le lien du bandeau, cliqué depuis l'onglet d'export, ouvre « Sauvegarder » sans recharger le document.
5. **AC5 — L'enveloppe transmet l'accessibilité de `bits-ui`.** Dans la page : rôles `tablist`/`tab`/`tabpanel`, un seul
   onglet à `tabindex="0"`, `aria-controls` de chaque `tab` égal à l'`id` d'un `tabpanel` **présent dans le DOM** (y compris
   pour un onglet jamais ouvert), Flèche droite depuis l'onglet focalisé sélectionne le suivant et écrit l'adresse. (Le
   détail du clavier — Origine, Fin, boucle — est celui de `bits-ui`, non retesté.)
6. **AC6 — Panneaux paresseux et conservés.** Le contenu d'un onglet jamais ouvert n'est pas monté (son `tabpanel` vide
   l'est) ; une fois ouvert, il reste dans le DOM, caché (`hidden`), quand on change d'onglet. Constat : modifier « nombre
   conservé » dans la sauvegarde automatique sans enregistrer, passer à « Exporter en CSV », revenir : la valeur saisie est
   toujours là, et aucun `PUT` n'est parti.
7. **AC7 — Anciennes adresses.** `/admin/backup`, `/admin/restore` et `/export` mènent respectivement à
   `/backup-export?onglet=sauvegarder`, `?onglet=restaurer`, `?onglet=exporter`, pour tout rôle connecté (un Comptable qui
   ouvre `/admin/backup` arrive sur l'onglet d'export, AC3).
8. **AC8 — Fonctionnement inchangé.** Sauvegarde complète (bouton, avis, sauvegarde automatique, liste des sauvegardes du
   serveur), restauration (confirmation forte, compte rendu, « Se reconnecter », maintien de session) et export CSV (bouton,
   état « Génération de l'export… », téléchargement `kesh-export-…-AAAA-MM-JJ.zip`) se comportent comme avant : les tests
   unitaires de `AdminBackupPanel`, `AutoBackupSection`, `AdminRestorePanel`, de `onboarding/restore` et les E2E
   `backup-schedule`, `backup-restore-new-install`, `export-global` passent, **seules leurs adresses de départ et les
   attentes d'AC7 changées**. Aucun appel d'API n'est ajouté, retiré ni modifié (`git diff` sans changement sous
   `crates/kesh-api/` ni dans les `*.api.ts`).
9. **AC9 — L'assistant inchangé.** `/onboarding/restore` rend toujours le titre « Restauration / import d'installation » en
   `<h1>` et en `<title>` (`AdminRestorePanel` sans `embedded`).
10. **AC10 — Bandeau.** Le lien du bandeau de la sauvegarde automatique pointe vers `/backup-export?onglet=sauvegarder` et
    dit « Administration → Sauvegarde et export » (et ses traductions) ; G3-bis (`loader.rs`) passe avec la liste
    `ENTREES_ADMINISTRATION` mise à jour.
11. **AC11 — i18n.** Les six clés de D10 existent dans les quatre catalogues ; les trois clés `nav-*` retirées n'existent
    plus nulle part (`git grep -nE 'nav-(export-global|admin-backup|admin-restore)\b' -- . ':!_bmad-output' ':!CHANGELOG.md'`
    vide) ; `lint-i18n-ownership`, `parity_between_locales`, `i18n-keys.test.ts` (dont `sitesTotal` recompté par la garde et
    la couverture close de `backup-export-`) passent.
12. **AC12 — Composant réutilisable.** `src/lib/components/ui/tabs/` réexporte les parties de `bits-ui` stylées et
    `UrlTabs` ; le test unitaire d'`UrlTabs` couvre AC4 (dont l'adresse qui change après le montage), AC6 et le filtre de
    visibilité **sans** dépendre de la page de sauvegarde (harnais à trois onglets factices).
13. **AC13 — Manuels et textes.** Les dix-sept sites du point 16 disent « Administration → Sauvegarde et export », avec
    l'onglet quand le texte nomme une action (« onglet *Sauvegarder* », « onglet *Restaurer ou importer* », « onglet
    *Exporter en CSV* ») et `\keshpath{/backup-export}` là où une adresse était écrite ; le manuel d'administration gagne une
    phrase sur la page (trois onglets, l'onglet d'export seul pour un comptable, anciennes adresses redirigées). Après la
    story, sur **tout le dépôt** hors historiques :
    `git grep -nE 'Sauvegarde complète|Restaurer / Importer|→ (\\emph\{)?Export global|/admin/backup|/admin/restore' -- . ':!CHANGELOG.md' ':!_bmad-output' ':!docs/testing.md'`
    ne rend **plus** de chemin de menu (`.env.example`, `docs/`, `README.md`, `frontend/`, `crates/` compris). Restes permis,
    listés au Dev Agent Record : le nom de la fonction « export global » en prose, `\paragraph{Export global.}` (`:2678`),
    les libellés du journal d'audit (`audit-log-action-admin-full-export` = « Sauvegarde complète exportée »), le titre
    `admin-backup-page-title` (« Sauvegarde complète de l'installation », titre de panneau, non un chemin), et les
    redirections elles-mêmes (`admin/backup/+page.ts`, `admin/restore/+page.ts` et leurs tests).
14. **AC14 — Recette visuelle.** Captures (thème clair, base de démonstration E2E) de chaque état touché, dans
    `/home/gcorbaz/devel/kesh-gate-logs/e505/dev/captures/` :
    - menu Administration déplié (Admin, puis Comptable) ;
    - page Admin sur chacun des trois onglets ; focus clavier visible sur un onglet ;
    - onglet « Restaurer ou importer » avec un fichier choisi, puis le dialogue de confirmation, puis le compte rendu d'un
      import (titre passé `<h2>`, D9) ;
    - le bandeau de la sauvegarde automatique (statut `failed` stubé) affiché sur la page, depuis l'onglet d'export, puis
      après le clic sur son lien ;
    - page Comptable (un onglet) ; la page Consultation est déclarée identique (même filtre, constaté par le test unitaire
      d'AC3) ;
    - page de l'assistant `/onboarding/restore` (inchangée) ;
    - les états d'`AutoBackupSection` sont repris par les captures existantes de `backup-schedule.spec.ts` (« états de
      l'écran et du bandeau »), rejouées à la nouvelle adresse.
    Jointes au Dev Agent Record et au prompt de la revue de code.

## Tests à écrire, et la preuve du rouge

Rouge d'abord : chaque test neuf est écrit et lancé **avant** le code qu'il exige, son échec copié dans
`/home/gcorbaz/devel/kesh-gate-logs/e505/dev/rouge/`. Puis les mutations, chacune tuée par un test nommé (journal
`e505/dev/mut/`, restauration par `git restore` puis `touch` du fichier, mémoire *Mutation restaurée, binaire périmé*).

| Test (fichier) | Ce qu'il prouve | AC |
|---|---|---|
| `src/lib/components/ui/tabs/url-tabs.test.ts` (neuf) — harnais `UrlTabsHarness.svelte` à trois onglets factices ; **`$app/state` simulé par un module réactif** `src/lib/components/ui/tabs/page-mock.svelte.ts` (`export const page = $state({ url: new URL('http://localhost/x') })`, branché par `vi.mock('$app/state', () => import('./page-mock.svelte'))`), `$app/navigation` simulé (`goto` espion) | (a) « **l'onglet suit l'adresse après montage** » : monter sur `?onglet=a`, puis `page.url = new URL('http://localhost/x?onglet=c')`, `await tick()`, attendre `aria-selected="true"` sur l'onglet `c` ; (b) clic → `goto` avec `?onglet=…`, `replaceState: true`, autres paramètres gardés ; (c) `onglet` inconnu ou invisible → premier visible ; aucun visible → rien ; invisible : ni déclencheur ni `tabpanel` ; (d) montage paresseux (contenu absent avant ouverture, `tabpanel` présent) et conservation (`hidden` après changement) ; (e) `param` personnalisé ; (f) fumée : `aria-controls` → `tabpanel` existant, un seul `tabindex="0"` | AC4, AC5, AC6, AC12 |
| `src/routes/(app)/backup-export/backup-export-page.test.ts` (neuf) | Admin : trois onglets dans l'ordre, un seul `h1`, `h2`/`h3` sous lui, phrase d'en-tête, « Sauvegarder » par défaut ; Comptable et Consultation : un onglet, panneaux admin absents même avec `?onglet=sauvegarder` ; `ssr === false` | AC2, AC3 |
| `src/routes/(app)/admin/backup/admin-backup-redirect.test.ts`, `admin/restore/admin-restore-redirect.test.ts`, `export/export-redirect.test.ts` (neufs ; motif `admin/mcp/admin-mcp-page.test.ts`, `browser` simulé) | `load()` lève 301 vers la bonne adresse, pour Admin, Comptable et Consultation | AC7 |
| `src/lib/features/export/GlobalExportPanel.test.ts` (neuf) | bouton, garde de ré-entrance, état « Génération… », succès, erreur `ApiError` et repli i18n ; `<h2>`, pas de `<title>` | AC8, D8, D9 |
| `AdminRestorePanel.test.ts` (complété) | `embedded` : `<h2>`, `document.title` non écrit ; sans `embedded` : `<h1>` et `<title>` comme avant | AC9, D9 |
| `AdminBackupPanel.test.ts`, `AutoBackupSection.test.ts` (complétés) | titre du panneau en `<h2>`, sections en `<h3>`, `data-testid` gardés | D9 |
| `BackupScheduleBanner.test.ts:73-75` (modifié) | `href` = `/backup-export?onglet=sauvegarder` | AC10 |
| `crates/kesh-i18n/src/loader.rs` (`les_chemins_cites_aux_catalogues_existent`, modifié) | `ENTREES_ADMINISTRATION` : `nav-backup-export` remplace les trois ; le texte du bandeau passe | AC10 |
| `i18n-keys.test.ts` (modifié) | famille `backup-export-` résolue, couverture close, `sitesTotal` recompté par la garde | AC11 |
| E2E `frontend/tests/e2e/backup-export.spec.ts` (neuf) | AC1 (Admin et Comptable, comptable créé par l'API comme `backup-schedule.spec.ts:203-210`) ; AC2 ; AC3 (Comptable) ; AC4 (clic, rechargement, lien profond ; **bandeau** : `GET /api/v1/admin/backup-schedule` stubé en statut `failed`, activé, par le motif `poser()` de `backup-schedule.spec.ts:137-143` ; marqueur `await page.evaluate(() => { (window as any).__keshSansRecharge = 1; })` posé sur `?onglet=exporter` ; clic sur `backup-schedule-banner-link` ; attendre `backup-export-tab-sauvegarder` à `aria-selected="true"`, l'adresse `?onglet=sauvegarder`, et le marqueur **toujours à 1**) ; AC5 (Flèche droite réelle) ; AC6 (saisie gardée, aucun `PUT` : `page.on('request')`) ; AC7 (trois anciennes adresses) ; captures d'AC14 (`KESH_E2E_CAPTURES_DIR`, motif `sidebar-navigation.spec.ts` `capture851`) | AC1-AC7, AC14 |
| E2E adaptés : `backup-schedule.spec.ts` (`:149`, `:222`, `:275` → `/backup-export?onglet=sauvegarder` ; `:214-216` : le comptable voit `nav a[href="/backup-export"]`, `/admin/backup` mène à `/backup-export?onglet=sauvegarder`, `auto-backup-section` absent, `export-global-start` visible, bandeau absent) ; `backup-restore-new-install.spec.ts` (`:104`, `:315` ; `:282`, `:328`, `:389` → `?onglet=restaurer` ; `:294` → `toHaveURL(/\/backup-export\?onglet=restaurer$/)`) ; `export-global.spec.ts:70` (→ `?onglet=exporter` ; l'assertion `heading level 1` reste vraie) ; `sidebar-navigation.spec.ts:219`, `:226-227` (`ADMINISTRATION`) | AC1, AC8 |

**L'échec attendu du test (a) sous M2**, tel que vitest l'imprime (forme à confirmer au rouge, et copiée au journal) :

```
FAIL  src/lib/components/ui/tabs/url-tabs.test.ts > UrlTabs > l'onglet suit l'adresse après montage
AssertionError: expected 'false' to be 'true' // Object.is equality
- Expected
+ Received
- true
+ false
```

(l'assertion `expect(screen.getByTestId('h-tab-c').getAttribute('aria-selected')).toBe('true')` dans un `waitFor`).
**Sous M2, l'E2E du bandeau** échoue ainsi (forme Playwright) :

```
Error: Timed out 5000ms waiting for expect(locator).toHaveAttribute(expected)
Locator: getByTestId('backup-export-tab-sauvegarder')
Expected string: "true"
Received string: "false"
```

Une sonde positive précède : sans mutation, le même test passe, et le marqueur `__keshSansRecharge` vaut 1 après le clic
(un rechargement complet le remettrait à `undefined` — c'est ce qui distingue « l'onglet a suivi l'adresse » de « la page a
été rechargée »).

**Mutations** (une à la fois, chacune tuée) :

- **M1** — retirer le filtre `visible` d'`UrlTabs` → `url-tabs.test.ts` (c) et `backup-export-page.test.ts` (Comptable :
  trois onglets).
- **M2** — lire `onglet` une seule fois au montage (`onMount` qui fixe un `$state`) au lieu de le dériver de `page.url` →
  `url-tabs.test.ts` (a) **et** E2E du bandeau (AC4).
- **M3** — démonter le contenu inactif (`{#if actif === id}` au lieu de `{#if vus.has(id)}`) → `url-tabs.test.ts` (d) et E2E
  AC6.
- **M4** — rediriger `/admin/restore` vers `?onglet=sauvegarder` → `admin-restore-redirect.test.ts`.
- **M5** — `onValueChange` qui n'appelle pas `goto` (l'adresse n'est plus écrite) → `url-tabs.test.ts` (b) et E2E AC4
  (rechargement).
- **M6** — laisser l'entrée de menu dans `adminOnly` → E2E AC1 (Comptable).
- **M7** — ignorer `embedded` dans `AdminRestorePanel` → `AdminRestorePanel.test.ts` et `backup-export-page.test.ts`
  (un seul `h1`).
- **M8** — `goto` sans `replaceState` → `url-tabs.test.ts` (b).
- **M9** — monter tous les contenus dès l'arrivée (sans `vus`) → `url-tabs.test.ts` (d, « contenu absent avant ouverture »).

## Tâches

1. **Étape 0** — relire au code les prémisses P1 à P5 (*Dev Notes*) et les lignes citées ; écrire le constat au Dev Agent
   Record, ligne lue citée ; un démenti s'arrête et se dit à l'orchestrateur.
2. **Composant d'onglets** (D3, D4) : parties stylées de `bits-ui`, `UrlTabs`, harnais, `page-mock.svelte.ts`,
   `url-tabs.test.ts` ; rouge (dont l'échec attendu du test (a) sous la forme M2), puis vert. Commit.
3. **Panneau d'export** (D8, D9) : `features/export/GlobalExportPanel.svelte` + test ; `AdminBackupPanel` en `<h2>` sans
   `<svelte:head>`, `AutoBackupSection` en `<h3>` ; `AdminRestorePanel` reçoit `embedded`. Commit.
4. **Page** `/backup-export` (D1, D5, D9) + test ; **redirections** des trois anciennes routes (D2) + tests. Commit.
5. **Menu** (D6), **bandeau** (point 8), **catalogues** (D10, quatre locales), `loader.rs`, `i18n-keys.test.ts`. Commit.
6. **Commentaires** devenus faux (point 17) : chacun réécrit ou retiré, dans le même patch que le code qui les rend faux.
7. **E2E** : spec neuve, quatre specs adaptées ; captures d'AC14. Commit.
8. **Manuels et textes** (AC13) : `admin-manual.tex`, `user-manual.tex`, `docs/user-guide/fr/getting-started.md`,
   `README.md:52`, `.env.example:198`. `make fr` dans `docs/manual/` pour contrôler la compilation ; **les PDF ne se
   commitent pas** (ils se régénèrent à la release). Commit.
9. **Gates** (*Gates* ci-dessous), mutations M1-M9, Dev Agent Record, File List, Change Log.

Ordre de repli si la coupure approche : tâches 2 à 6 d'abord (le fonctionnement), puis 7 et 8 ; une story dont le manuel ou
les captures manquent n'est pas close et sort de la release (elle ne part pas à moitié).

## Gates

- Bases propres à la story : `kesh_e505`, `kesh_ci_e505`, `kesh_e2e_e505` (`docs/agents-orchestration.md`) ; journaux
  sous `/home/gcorbaz/devel/kesh-gate-logs/e505/`. Tout sous `scripts/mem-guard.sh` et `flock` du verrou de la voie.
- **Entre deux étapes** : `npm run check`, `npm run lint-i18n-ownership`, `vitest` ciblé ; pour `loader.rs` :
  `cargo nextest run -p kesh-i18n`.
- **Au dernier commit de code** : gate complet (`scripts/gate-ci.sh` : la story touche `kesh-i18n`, et les gardes de
  `kesh-api` lisent catalogues et manuels, dont `textes_coherents.rs`) ; frontend complet (`check`,
  `lint-i18n-ownership`, `test:unit`, `build`) ; **E2E complet** (« vert » = seulement les échecs attendus de
  `docs/testing.md`, jugés fichier par fichier).

## Note de version

**Modifié** — La sauvegarde complète, la restauration et l'export CSV sont réunis sur une page *Administration →
Sauvegarde et export*, à trois onglets (*Sauvegarder*, *Restaurer ou importer*, *Exporter en CSV*) ; un comptable n'y voit
que l'onglet d'export. Les anciennes adresses (`/admin/backup`, `/admin/restore`, `/export`) mènent à l'onglet
correspondant. Rien ne change dans ce que font ces écrans. (refs #1250, #1248)

## Dev Notes

### Prémisses à confirmer à l'étape 0

- **P1** — SvelteKit 2 accepte un dossier de route qui n'a qu'un `+page.ts` et exécute son `load()` côté client (lu en P1 à
  la source de SvelteKit 2.55.0 : `create_manifest_data/index.js:332-342`, `write_client_manifest.js:32`,
  `runtime/client/client.js:1695-1706` ; vraisemblable, non prouvé). À vérifier par un essai (`npm run build`, puis ouverture
  de `/admin/backup`) ; sinon, garder un `+page.svelte` vide, commenté. Dans les deux cas, AC7 se constate en E2E.
- **P2** — `redirect(301, …)` levé dans un `load()` client mène à `new URL(location, url)` (`client.js:1699`) : la requête
  `?onglet=…` est conservée.
- **P3** — `goto` vers la même route avec une autre requête ne remonte pas la page (le marqueur de l'E2E du bandeau le
  constate).
- **P4** — Aucun test ni script n'énumère les trois `+page.svelte` retirés (confirmée en P1, point 17).
- **P5** — vitest compile un module `.svelte.ts` importé par la fabrique d'un `vi.mock` (runes actives), et la mise à jour
  de `page.url` y est vue par le `$derived` d'`UrlTabs`. Aucun précédent au dépôt (point 13) : si la prémisse tombe, repli
  sur un harnais qui reçoit `url` en propriété réactive (`UrlTabs` prend alors une propriété `url` facultative, `page.url`
  par défaut) — à dire au Dev Agent Record ; l'E2E du bandeau reste le second tueur de M2.
- **P6** — `bits-ui` 2.16.5 se comporte comme lu au point 10 avec un `value` contrôlé non lié (`value={actif}` sans
  `bind:`) et `onValueChange` : la sélection locale puis la confirmation par l'adresse ne bouclent pas (à constater par le
  test (b) et l'E2E AC5).

### Modules touchés (règle de splitting)

Au grain du `CLAUDE.md` — un module est un crate, ou un domaine `frontend/src/lib/features/<domaine>` dont le **code de
production** change (décision du Project Lead du 2026-11-21) : **`kesh-i18n`** (catalogues ; `loader.rs` en test seulement),
**`features/admin-backup`** (`AdminBackupPanel`, `AutoBackupSection`, `BackupScheduleBanner`), **`features/admin-restore`**
(`AdminRestorePanel`), **`features/export`** (`GlobalExportPanel`, neuf) : **quatre modules**. `components/ui/tabs/` et
`routes/(app)/` ne sont ni des crates ni des domaines `features/` : la définition ne les compte pas. Si on les comptait
chacun, on arriverait à six — c'est écrit ici pour que le décompte ne laisse pas de doute ; la règle en vigueur donne
quatre, sous la borne. Une découverte qui toucherait un cinquième domaine `features/` ou un autre crate se dit à
l'orchestrateur.

### Fichiers

- Neufs : `frontend/src/lib/components/ui/tabs/{index.ts,tabs.svelte,tabs-list.svelte,tabs-trigger.svelte,tabs-content.svelte,url-tabs.svelte,url-tabs.test.ts,UrlTabsHarness.svelte,page-mock.svelte.ts}` ;
  `frontend/src/routes/(app)/backup-export/{+page.svelte,+page.ts,backup-export-page.test.ts}` ;
  `frontend/src/lib/features/export/{GlobalExportPanel.svelte,GlobalExportPanel.test.ts}` ; trois tests de redirection ;
  `frontend/tests/e2e/backup-export.spec.ts`.
- Modifiés : `frontend/src/routes/(app)/+layout.svelte` ; `routes/(app)/admin/backup/+page.ts`,
  `admin/restore/+page.ts`, `export/+page.ts` ; retirés (P1) : leurs `+page.svelte` ;
  `features/admin-backup/{AdminBackupPanel.svelte,AdminBackupPanel.test.ts,AutoBackupSection.svelte,AutoBackupSection.test.ts,BackupScheduleBanner.svelte,BackupScheduleBanner.test.ts}` ;
  `features/admin-restore/{AdminRestorePanel.svelte,AdminRestorePanel.test.ts}` ; commentaires du point 17
  (`admin/mcp/+page.ts`, `admin-mcp-page.test.ts`, `onboarding/restore/{+page.ts,+page.svelte,onboarding-restore-page.test.ts}`) ;
  `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` ; `crates/kesh-i18n/src/loader.rs` (test) ;
  `frontend/src/lib/shared/i18n-keys.test.ts` ; quatre specs E2E ; `docs/manual/fr/admin-manual.tex`,
  `docs/manual/fr/user-manual.tex`, `docs/user-guide/fr/getting-started.md`, `README.md` (ligne 52 seulement ; la feuille
  de route est à l'orchestrateur), `.env.example`.
- **Fichiers communs avec les autres stories de la release** : les quatre catalogues (chaque story dans son bloc ; conflit
  résolu en gardant les deux côtés), `i18n-keys.test.ts` (`sitesTotal` recompté après fusion), `+layout.svelte` (le menu),
  `admin-manual.tex`, `README.md`. Aucun fichier de 48-11, 48-12a/b n'est touché par ailleurs.

### Ce qui doit rester vrai

- Les droits : le serveur reste l'autorité (`require_admin_role` pour sauvegarde et import ; tous rôles pour l'export) ;
  l'écran ne rend ni la sauvegarde ni la restauration pour un non-Admin.
- `AdminRestorePanel` : maintien de session, `onDestroy`, « Se reconnecter », rendu de l'assistant — inchangés (D4, D9).
- `AutoBackupSection` : ses lectures au montage, son 409, ses avis — inchangés.
- Les `data-testid` existants des panneaux (`admin-backup-*`, `admin-restore-*`, `auto-backup-*`, `export-global-*`,
  `backup-env-notice`) : inchangés (garde KF-043, sélecteurs des E2E).
- Aucune facture, aucun avoir, aucun PDF, aucune écriture n'est touché : la story ne lit ni n'écrit rien de comptable.

### Pièges connus

- **`backup-env-notice` en double** (V1-9) : un Admin qui a ouvert les deux premiers onglets a deux `BackupEnvNotice` dans le
  DOM (l'un `hidden`) ; `getByTestId` de Playwright est strict. Dans la spec neuve, borner le sélecteur au panneau :
  `page.getByTestId('backup-export-panel-restaurer').getByTestId('backup-env-notice')`. Les E2E existants, qui ouvrent une
  seule fois la page, ne sont pas touchés.
- **Simulation statique de `$app/state`** (point 13) : elle ne peut pas tuer M2 ; c'est la raison de `page-mock.svelte.ts`.
- **Pas de `$effect` qui écrit l'adresse** (D3, D5).

### Hors périmètre

- **#1253** (sous-groupes du menu) : release ultérieure ; D6 ne lui ajoute aucune contrainte, et lui laisse une note.
- **#1249, #1251, #1252** : pages MCP, Facturation, Paramètres ; elles prendront le composant tel quel (avertissement : D7).
- Migration de `reports`, `email-templates`, `due-dates` vers le composant (D3) : **issue de dette décidée** (`technical-debt`,
  P4), ouverte par l'orchestrateur le 2026-11-25.
- Textes des panneaux (descriptions « À distinguer de l'export global d'une seule société », etc.) : inchangés.

### Références

- `CLAUDE.md` : § *Qualité du code* (DRY), § *Recette visuelle*, § *E2E (Playwright)*, § *Règle de splitting préventif*,
  § *Review Iteration Rule* (le prompt de passe nomme le manuel, PDF compris), § *Documentation à synchroniser* (PDF non
  commités dans `story/…`).
- Issues : #1250, #1248, #1249, #1251, #1252, #1253.
- Source : `frontend/node_modules/bits-ui/dist/bits/tabs/` (2.16.5) ; motifs d'enveloppe : `ui/separator/`, `ui/dialog/`.
- Motifs : `routes/(app)/admin/mcp/admin-mcp-page.test.ts` (test d'un `load()` de garde) ; `backup-schedule.spec.ts`
  (`poser()`, création d'un comptable) ; `sidebar-navigation.spec.ts` (captures, liste `ADMINISTRATION`).
- Rapport de la validation P1 : `/home/gcorbaz/devel/kesh-gate-logs/e505/validation-p1.md`.

### Charge estimée

Développement : **un jour environ** — l'enveloppe de `bits-ui` (quatre parties d'une dizaine de lignes) et `UrlTabs`
(environ 60 lignes) avec son test, une page de 50 lignes, un panneau déplacé, trois redirections, menu et catalogues, une
spec E2E neuve et quatre adaptées, dix-sept phrases de textes. Revue : **une demi-journée à une journée** — validation P2
(Opus), au moins deux passes de revue de code en modèles alternés, mutations M1-M9, `gate-ci` complet et E2E complet au
dernier commit de code. Avec une coupure au **vendredi 2026-11-27 au matin**, la story tient si le développement part le
mercredi 2026-11-25 ; un recyclage de la validation ou de la revue la sort de la release.

## Dev Agent Record

*(vide)*

## Change Log

| Date | Auteur | Entrée |
|---|---|---|
| 2026-11-24 | agent de rédaction (Opus 5.5) | Création de la fiche par `bmad-create-story`, en autonomie ; statut `drafted` ; validation P1 à lancer (Sonnet). |
| 2026-11-24 | agent de rédaction (Opus 5.5) | **Validation P1 remédiée** (P1 Sonnet, contexte frais : 0 CRITICAL / 0 HIGH / 3 MEDIUM / 12 LOW ; rapport `kesh-gate-logs/e505/validation-p1.md`). Prémisses relues au code avant chaque remède. MEDIUM : **V1-2** — prémisse confirmée à la source (`bits-ui` 2.16.5, `tabs.svelte.js:75`, `:100-108`, `:119-121`, `:137`, `:146-150`, `:174-186` ; `roving-focus-group.js:32-70` ; `tabs-content.svelte`) : D3 réécrit, le composant **enveloppe `bits-ui` `Tabs`** et n'ajoute que `UrlTabs` (adresse, filtre, montage paresseux) ; D4, AC5, AC12, mutations (M5 « `End` » retirée, M5 « `goto` absent » et M9 « tout monté » ajoutées) et tâches suivent. **V1-1** — `.env.example:198` relu et ajouté (point 16), grep de clôture d'AC13 élargi à tout le dépôt hors historiques. **V1-3** — prémisse confirmée (`reports-page.test.ts:22-26`, simulation statique) : module réactif `page-mock.svelte.ts` (P5, repli écrit) et E2E du bandeau (stub `GET backup-schedule`, marqueur de non-rechargement) ; échecs attendus écrits. LOW : V1-4 coupure au vendredi 2026-11-27 au matin (décision du Project Lead) et charge de revue écrite, modules recomptés (quatre au grain du `CLAUDE.md`, `ui/tabs` et `routes` dits) ; V1-5 fichier d'epic, `sprint-status` et feuille de route laissés à l'orchestrateur, dit en tête ; V1-6 commentaires devenus faux (point 17, tâche 6) ; V1-7 `tabpanel` toujours rendu, contenu paresseux ; V1-8 clavier depuis l'onglet focalisé (fourni par `bits-ui`, `roving-focus-group.js:39`) ; V1-9 `backup-env-notice` en double (*Pièges connus*) ; V1-10 `AutoBackupSection` en `<h3>` ; V1-11 états ajoutés à AC14 ; V1-12 D7 : `beforeNavigate`, ordre de la série dit et justifié ; V1-13 issue de dette des trois `tablist` décidée (ouverte par l'orchestrateur le 2026-11-25) ; V1-14 note pour #1253 (sous-groupe mixte) ; V1-15 italien « Esegui il backup » ; V1-16 PDF régénérés à la release, dit au point 16. **#1248 se ferme avec la story** (décision du Project Lead du 2026-11-24), dit en tête et en D7. Validation P2 à lancer : Opus, passe complète, prompt `50-5-page-sauvegarde-export-validation-prompt-p2.md`. |
