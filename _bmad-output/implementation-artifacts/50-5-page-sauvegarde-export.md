# Story 50-5 : une page « Sauvegarde et export » à onglets, au lieu de trois entrées du menu (#1250)

Status: drafted

*Fiche rédigée le 2026-11-24 par l'agent de rédaction de la 50-5 (Opus 5.5), par le workflow `bmad-create-story`, en
autonomie ; validation P1 à lancer (prompt versionné à côté : `50-5-page-sauvegarde-export-validation-prompt-p1.md`).*

Issue : **#1250** — `enhancement`, **P4**, milestone v0.20.0. Ergonomie, **sans changement de fonctionnement**. Release
**0.20.0** (vendredi 2026-11-27, coupure le jeudi 2026-11-26 au soir), **Epic 50**, cinquième story (sous la borne de six).
**Sous condition** (décision du Project Lead du 2026-11-24) : si elle n'est pas close à la coupure, elle sort de la release
sans la retarder. Voie : **frontend** (plus les catalogues de `kesh-i18n` et les manuels). **Aucune migration, aucun code
Rust de production.** Base : `origin/epic/50-corrections` = `e5654fc00` (porte `main` avec l'Epic 38 PayPal).

Liées : **#1248** (composant d'onglets, milestone « Plus tard », prérequis de #1250 : **cette story le livre**, D3, D7) ;
#1249 (MCP), #1251 (Facturation), #1252 (Paramètres) — la série qui réutilisera le composant ; **#1253** (sous-groupes du menu
Administration, release ultérieure) — à ne pas gêner (D6).

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
   /api/v1/exports/global.zip` — « Tous rôles authentifiés (AC #11 Consultation 200) ». La sauvegarde et l'import restent
   `require_admin_role` (commentaires de `admin/backup/+page.ts:1-6` et `admin/restore/+page.ts:1-5`).
3. **Le menu.** `src/routes/(app)/+layout.svelte` :
   - `:137` (groupe Administration, `items`, visible de **tous** les rôles) :
     `{ i18nKey: 'nav-export-global', fallback: 'Export global', href: '/export' },` ;
   - `:162-176` (`adminOnly`) : `nav-admin-backup` → `/admin/backup`, `nav-admin-restore` → `/admin/restore` ;
   - `:238-245` `findGroupKeyForPath` compare `it.href === pathname` (la requête n'y entre pas) ;
   - `:297-300` `navTestid(href)` : `nav-link-` + chemin, `/` → `-` (donc `/backup-export` → `nav-link-backup-export`).
4. **Les en-têtes des panneaux.** `AdminBackupPanel.svelte:54-58` (`<svelte:head><title>`) et `:61` (`<h1 …
   data-testid="admin-backup-title">`) ; `AdminRestorePanel.svelte:304-308` et `:312` (idem, `admin-restore-title`). Sur une
   page à trois panneaux, chacun écrirait son `<title>` et son `<h1>`.
5. **`AdminRestorePanel` sert aussi l'assistant.** `src/routes/onboarding/restore/+page.svelte:11` l'importe et le rend
   (`:29`) sous un lien de retour, sans titre propre : son `<h1>` et son `<title>` y sont **le** titre de la page.
6. **`AdminRestorePanel` agit à son démontage.** `:99` `onDestroy(() => …)` : lève le maintien de session et **termine la
   session si le compte rendu d'import était affiché** (en-tête du fichier, `:18-26`). Démonter le panneau à chaque changement
   d'onglet changerait donc le fonctionnement (D4).
7. **Une saisie non enregistrée vit dans l'onglet de sauvegarde.** `AutoBackupSection.svelte:58-60` (`enabled`,
   `intervalHours`, `retentionCount`), bouton « Enregistrer » `:370-375` (`save()` `:171`). Démonter le panneau la perdrait (D4).
8. **Le bandeau de la sauvegarde automatique** renvoie à l'ancienne adresse : `BackupScheduleBanner.svelte:66` `<a
   href="/admin/backup" …>` avec le libellé `admin-backup-banner-link` (`:67`) = « Administration → Sauvegarde complète »
   (`crates/kesh-i18n/locales/fr-CH/messages.ftl:2410`, et `:2320` des trois autres catalogues). Son test épingle l'adresse :
   `BackupScheduleBanner.test.ts:73-75`.
9. **Kesh n'a pas de composant d'onglets** (#1248 : « Kesh n'a aucun composant d'onglets »). Trois pages écrivent leur
   `tablist` à la main : `routes/(app)/reports/+page.svelte:820-838` (clavier `:589-601`, adresse `?tab=` en `replaceState`
   `:608-614`, lecture au montage `:546-551`), `settings/email-templates/+page.svelte:307-330`,
   `invoices/due-dates/+page.svelte:491-495`. `src/lib/components/ui/` ne contient ni `tabs/` ni équivalent
   (`button`, `dialog`, `dropdown-menu`, `input`, `select`, `separator`, `sonner`, `table`, `tooltip`). `bits-ui` est une
   dépendance (`package.json:46`), sans composant `Tabs` enveloppé dans le dépôt.
10. **Le commentaire de `reports/+page.svelte:553-566` met en garde** : un lien vers la **même** route ne remonte pas le
    composant ; un `onMount` qui lit l'adresse reste muet. Le bandeau (point 8) est un tel lien quand on est déjà sur la page.
11. **L'utilisateur est connu avant la page.** `src/routes/(app)/+layout.ts:10-12` redirige vers `/login` si
    `!authState.isAuthenticated`, qui vaut `_currentUser !== null` (`src/lib/app/stores/auth.svelte.ts:131-132`).
12. **Gardes i18n qui nomment ces clés.**
    - `crates/kesh-i18n/src/loader.rs:1174-1190` (`les_chemins_cites_aux_catalogues_existent`, G3-bis) :
      `ENTREES_ADMINISTRATION: [&str; 14]` contient `nav-export-global`, `nav-admin-backup`, `nav-admin-restore` ; tout texte
      « Administration → X » d'un catalogue doit commencer par le libellé d'une de ces entrées.
    - `src/lib/shared/i18n-keys.test.ts:1357-1364` (`FAMILLES_RESOLUES['nav-']`) contient `export-global`, `admin-backup`,
      `admin-restore` ; `:1203` `sitesTotal: 3695` ; `:1441` `PREFIXES_A_COUVERTURE_CLOSE`.
    - `scripts/lint-i18n-ownership.js:102-105`, `:128-137` : dans `src/lib/features/<f>/`, une clé doit appartenir à `<f>`
      (`export` : préfixe `export-`) ; les fichiers de `src/routes/` ne sont pas contrôlés.
13. **E2E qui passent par ces adresses** (`git grep -nE "admin/backup|admin/restore|'/export'" -- frontend/tests/e2e`) :
    `backup-schedule.spec.ts:149`, `:214-216` (comptable : `nav a[href="/admin/backup"]` absent, puis `toHaveURL('/')`),
    `:222`, `:275` ; `backup-restore-new-install.spec.ts:104`, `:282`, `:294` (`toHaveURL(/\/admin\/restore$/)`), `:315`,
    `:328`, `:389` ; `export-global.spec.ts:70` ; `sidebar-navigation.spec.ts:219`, `:226-227` (`ADMINISTRATION`).
14. **Manuels et documents qui décrivent ces chemins** — inventaire par le symptôme, ensemble clos à `e5654fc00` :

    ```
    $ git grep -nE 'Sauvegarde complète|Restaurer / Importer|Export global|/admin/backup|/admin/restore|keshpath\{/export\}' \
        -- docs/manual/fr/*.tex docs/user-guide README.md website | cut -c1-80
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

    `website/` : aucun site. Les manuels DE, IT et EN n'existent qu'en `README.md` (`docs/manual/{de,en,it}/`) : rien à
    traduire. Aucune capture des manuels ne montre ces pages (`docs/manual/fr/screenshots/` : aucun nom qui contienne backup,
    restor, export, sauveg). `CHANGELOG.md:1483` et `docs/testing.md:476` sont historiques : on ne les réécrit pas.

## Décisions

- **D1 — La page : `/backup-export`, titre « Sauvegarde et export ».** Une route neuve
  `src/routes/(app)/backup-export/+page.svelte` (+ `+page.ts`, `ssr = false`, **sans garde de rôle** : un comptable y a son
  onglet). *Écartées* : `/admin/backup` gardée comme page (un comptable n'a rien à faire sous `/admin/`, dont toutes les pages
  sont réservées à l'administrateur) ; `/export` gardée (le titre dirait « export » d'une page dont le premier onglet est la
  sauvegarde).
- **D2 — Les anciennes adresses redirigent vers l'onglet correspondant**, pour ne casser ni un favori ni le lien d'un ancien
  PDF du manuel : `/admin/backup` → `/backup-export?onglet=sauvegarder`, `/admin/restore` → `?onglet=restaurer`, `/export` →
  `?onglet=exporter`. Chaque ancien `+page.ts` devient `export const ssr = false;` plus un `load()` qui lève
  `redirect(301, …)`, **sans condition de rôle** (la page cible filtre ses onglets, D5) ; les anciens `+page.svelte` sont
  retirés, ou réduits à un fichier vide si SvelteKit l'exige (prémisse P1). Conséquence assumée : un comptable qui tape
  `/admin/backup` n'est plus renvoyé à l'accueil mais arrive sur l'onglet d'export — il ne voit toujours ni la sauvegarde ni
  la restauration. *Écartée* : redirection côté serveur (Axum ne connaît pas les routes du frontend ; l'adaptateur statique
  sert le même `index.html`).
- **D3 — Un composant d'onglets réutilisable, livré ici** : `src/lib/components/ui/tabs/tabs.svelte` + `index.ts` (forme des
  composants voisins, `button/`, `dialog/`). Il répond aux exigences de #1248 que la série demande :
  - accessibilité : `role="tablist"` (avec `aria-label`), `role="tab"` (`aria-selected`, `aria-controls`, `id`, tabindex
    itinérant 0/-1), `role="tabpanel"` (`aria-labelledby`) ; clavier : Flèche droite / gauche (circulaires), Origine, Fin ;
    la sélection suit le focus (comme `reports`) ;
  - **onglet porté par l'adresse** : paramètre `onglet` (nom fixé par #1248 ; propriété `param`, défaut `'onglet'`).
    L'onglet actif est **dérivé de `page.url`** (`$app/state`), jamais lu une seule fois au montage (point 10 : le bandeau
    est un lien vers la même route) ; un clic ou une touche écrit l'adresse par `goto(…, { replaceState: true, keepFocus:
    true, noScroll: true })`, en gardant les autres paramètres ;
  - **onglets filtrables par rôle** : chaque onglet porte `visible?: boolean` (défaut vrai) ; seuls les visibles se
    rendent ; un `onglet` inconnu ou invisible, ou absent, donne le **premier onglet visible** (sans réécrire l'adresse,
    D5) ; aucun onglet visible : rien ne se rend ;
  - **montage paresseux, puis conservé** : un panneau se monte la première fois qu'on l'ouvre et reste monté, caché
    (`hidden`), quand on change d'onglet (D4) ;
  - API Svelte 5 : `tabs: { id: string; label: string; visible?: boolean }[]`, `label` (nom accessible de la liste),
    `idPrefix` (préfixe des `id` et des `data-testid` : `<idPrefix>-tab-<id>`, `<idPrefix>-panel-<id>`), `param?`, et un
    snippet `panel(id)`. Le composant n'a **aucun texte propre** (pas de clé i18n) ; jetons de couleur du thème existant
    seulement (`border-primary`, `text-primary`, `text-text-muted`, `hover:text-text`, `border-border`) — la garde
    `components/ui/jetons-non-definis.test.ts` le contrôle.
  *Écartées* : `Tabs` de `bits-ui` (sa sémantique de montage des contenus et la synchronisation à l'adresse seraient à
  envelopper de toute façon, et rien dans le dépôt ne l'emploie encore) ; recopier le `tablist` de `reports` (quatrième
  copie, contraire au DRY). La migration de `reports`, `email-templates` et `due-dates` vers le composant **n'est pas** dans
  le périmètre (elle changerait `?tab=` en `?onglet=` sur `reports`, dont des liens dépendent) : dette à tracer en issue
  (question à l'orchestrateur, *Dev Notes*).
- **D4 — Les panneaux restent montés une fois ouverts.** Démonter `AdminRestorePanel` en changeant d'onglet terminerait la
  session après un import réussi (point 6) et abandonnerait un import en cours ; démonter `AdminBackupPanel` perdrait le
  réglage non enregistré (point 7). Le montage paresseux évite en outre que l'onglet d'export d'un administrateur lance les
  lectures de la sauvegarde automatique avant qu'on ouvre l'onglet. *Écartée* : tout monter dès l'arrivée (trois chargements
  pour un seul onglet regardé, et `AutoBackupSection` interroge le serveur à son montage).
- **D5 — Onglets, ordre et droits.** Ordre de #1250 : **Sauvegarder** (`sauvegarder`, `AdminBackupPanel`), **Restaurer ou
  importer** (`restaurer`, `AdminRestorePanel`), **Exporter en CSV** (`exporter`, panneau d'export, D8). Les deux premiers :
  `visible: role === 'Admin'` ; le troisième : visible de tout utilisateur connecté (Admin, Comptable, Consultation), comme
  la route serveur (point 2) et l'entrée de menu d'aujourd'hui (point 3). Un administrateur arrive sur « Sauvegarder », un
  autre rôle sur « Exporter en CSV ». La liste d'onglets s'affiche **même quand un seul onglet est visible** (le comptable
  voit où il est ; *écartée* : la cacher, une variante de plus dans le composant). Pas de réécriture de l'adresse quand
  l'onglet demandé n'est pas permis (*écartée* : un `$effect` qui réécrit l'adresse, une écriture que personne n'a demandée,
  sans gain pour l'utilisateur). Les droits du serveur ne changent pas ; la garde d'écran des anciennes routes est remplacée
  par le filtre des onglets (même garantie : l'écran de sauvegarde ou de restauration ne se rend pas pour un non-Admin).
- **D6 — Le menu : une entrée au lieu de trois.** `nav-backup-export` → `/backup-export`, libellé « Sauvegarde et export »,
  dans les `items` du groupe Administration (visible de tous les rôles), **à la place** de « Export global » (`:137`) ; les
  deux entrées `adminOnly` (`:162-176`) sont retirées. Pour **#1253** : rien de neuf dans la structure du menu (pas de
  sous-groupe, pas de nouvelle variante de `NavItem`) ; l'inventaire du point 14 lui sert de liste des sites à repasser
  (« Administration → Système → Sauvegarde et export ») ; la garde G3-bis (`loader.rs`) contrôlera ses chemins comme les
  nôtres.
- **D7 — Pas d'avertissement « modifications non enregistrées »** au changement d'onglet : avec D4, rien ne se perd en
  changeant d'onglet, et cette page n'a qu'un formulaire (la sauvegarde automatique), à bouton propre. #1248 le demande pour
  la série (Paramètres, Facturation) : le composant reste ouvert à cet ajout (un rappel `onBeforeChange` facultatif), qui
  n'est **pas** écrit ici (aucun appelant, donc aucun test honnête). **À arbitrer par le Project Lead** : #1248 se ferme-t-il
  avec cette story (l'avertissement rendu inutile par D4), ou se scinde-t-il avant la PR (l'avertissement dans une issue
  neuve, portée par #1251/#1252) ? La règle « une issue, une release » interdit de le résoudre en deux fois sans scission.
  Dans le doute, les commits citent `refs #1248`, jamais un mot-clé de fermeture.
- **D8 — L'export quitte la route pour un composant de feature** : `src/lib/features/export/GlobalExportPanel.svelte`
  (la logique de `export/+page.svelte:1-53` et son balisage, **mot pour mot**, sauf le `<svelte:head>` retiré et le `<h1>`
  devenu `<h2>`), testable et soumis au lint d'ownership (`export-global-*` appartient bien à `export`). La route `/export`
  ne garde que sa redirection (D2).
- **D9 — Un seul `<h1>` et un seul `<title>` par page.** La page porte `<h1>` « Sauvegarde et export », la phrase de #1250
  et `<title>` « Sauvegarde et export – Kesh ». Dans la page, les titres des panneaux deviennent des `<h2>` (texte et
  `data-testid` inchangés) et leurs `<svelte:head>` disparaissent. `AdminBackupPanel` et `GlobalExportPanel` ne servent
  qu'ici : changement direct. `AdminRestorePanel` sert aussi l'assistant (point 5) : il reçoit une propriété
  `embedded = false` ; `embedded` vrai rend un `<h2>` et aucun `<title>` ; faux (l'assistant) garde exactement le rendu
  d'aujourd'hui. *Écartée* : donner un titre propre à la page de l'assistant (change l'assistant, hors périmètre).
- **D10 — Textes (quatre locales).** Clés neuves, préfixe `backup-export-` (page, hors `features/` : pas de propriétaire au
  lint) et une clé de menu :

  | Clé | fr-CH | de-CH | it-CH | en-CH |
  |---|---|---|---|---|
  | `nav-backup-export` | Sauvegarde et export | Sicherung und Export | Backup ed esportazione | Backup and export |
  | `backup-export-title` | Sauvegarde et export | Sicherung und Export | Backup ed esportazione | Backup and export |
  | `backup-export-intro` | La sauvegarde (.keshbackup) couvre toute l'installation et se restaure dans Kesh ; l'export produit des fichiers CSV lisibles d'une société, pour une fiduciaire ou un autre logiciel. | Die Sicherung (.keshbackup) umfasst die ganze Installation und lässt sich in Kesh wiederherstellen; der Export erzeugt lesbare CSV-Dateien einer Firma, für eine Treuhand oder eine andere Software. | Il backup (.keshbackup) comprende l'intera installazione e si ripristina in Kesh; l'esportazione produce file CSV leggibili di una società, per una fiduciaria o un altro software. | The backup (.keshbackup) covers the whole installation and can be restored in Kesh; the export produces readable CSV files for one company, for a fiduciary or another software. |
  | `backup-export-tab-backup` | Sauvegarder | Sichern | Backup | Back up |
  | `backup-export-tab-restore` | Restaurer ou importer | Wiederherstellen oder importieren | Ripristina o importa | Restore or import |
  | `backup-export-tab-export` | Exporter en CSV | Als CSV exportieren | Esporta in CSV | Export as CSV |

  Vocabulaire selon `docs/i18n-glossaire.md:158` (sauvegarde / Sicherung / backup / backup). `admin-backup-banner-link`
  change de **valeur** (pas de clé) : « Administration → Sauvegarde et export » et ses trois traductions, exigé par G3-bis.
  Les clés `nav-export-global`, `nav-admin-backup`, `nav-admin-restore` sont **retirées** des quatre catalogues (plus aucun
  appelant ; la parité reste égale). `backup-export-` entre dans `PREFIXES_A_COUVERTURE_CLOSE` (ses cinq clés sont toutes
  demandées) ; les trois `backup-export-tab-*` vivent dans une table (`labelKey`) : famille `backup-export-` ajoutée à
  `FAMILLES_RESOLUES`. Les traductions sont des propositions de la fiche : la revue les relit contre le glossaire.

## Critères d'acceptation

1. **AC1 — Menu.** Pour un Admin, le groupe Administration porte **une** entrée « Sauvegarde et export »
   (`data-testid="nav-link-backup-export"`, `href="/backup-export"`) et **aucune** des trois anciennes (`nav-link-export`,
   `nav-link-admin-backup`, `nav-link-admin-restore` : 0 élément). Pour un Comptable, la même entrée est présente (comme
   « Export global » aujourd'hui). Ouvrir `/backup-export` déplie le groupe Administration (`findGroupKeyForPath`).
2. **AC2 — Page, Admin.** `/backup-export` rend un seul `<h1>` « Sauvegarde et export », la phrase `backup-export-intro`,
   puis une `tablist` de **trois** onglets dans l'ordre Sauvegarder, Restaurer ou importer, Exporter en CSV ; sans paramètre,
   « Sauvegarder » est sélectionné (`aria-selected="true"`) et son panneau montre `admin-backup-panel`. `document.title`
   vaut « Sauvegarde et export – Kesh ».
3. **AC3 — Page, autres rôles.** Pour un Comptable et pour un utilisateur Consultation, la `tablist` ne porte **qu'un**
   onglet, « Exporter en CSV », sélectionné ; `admin-backup-panel` et `admin-restore-panel` ne sont **pas** dans le DOM, même
   avec `?onglet=sauvegarder` ou `?onglet=restaurer`. Le bouton `export-global-start` est présent.
4. **AC4 — Adresse.** Cliquer un onglet écrit `?onglet=<id>` sans empiler d'entrée d'historique ; recharger la page garde
   l'onglet ; ouvrir `/backup-export?onglet=restaurer` sélectionne « Restaurer ou importer » ; un `onglet` inconnu donne le
   premier onglet visible ; un lien vers la même route avec un autre `onglet` (le bandeau, depuis l'onglet d'export) change
   l'onglet **sans recharger** la page.
5. **AC5 — Clavier et ARIA.** Sur un onglet focalisé : Flèche droite / gauche passe au suivant / précédent (circulaire),
   Origine au premier, Fin au dernier, et le focus suit ; un seul onglet a `tabindex="0"` ; chaque `tab` a `aria-controls`
   égal à l'`id` de son `tabpanel`, chaque `tabpanel` a `aria-labelledby` égal à l'`id` de son `tab`.
6. **AC6 — Panneaux conservés.** Un panneau non encore ouvert n'est pas monté ; une fois ouvert, il reste dans le DOM,
   caché (`hidden`), quand on change d'onglet. Constat : modifier « nombre conservé » dans la sauvegarde automatique sans
   enregistrer, passer à « Exporter en CSV », revenir : la valeur saisie est toujours là, et aucun `PUT` n'est parti.
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
    plus nulle part (`git grep -nE 'nav-(export-global|admin-backup|admin-restore)\b'` vide hors historique) ;
    `lint-i18n-ownership`, `parity_between_locales`, `i18n-keys.test.ts` (dont `sitesTotal` recompté par la garde et la
    couverture close de `backup-export-`) passent.
12. **AC12 — Composant réutilisable.** `src/lib/components/ui/tabs/` exporte le composant ; son test unitaire couvre AC4,
    AC5, AC6 et le filtre de visibilité **sans** dépendre de la page de sauvegarde (montage d'un harnais à trois onglets
    factices).
13. **AC13 — Manuels.** Les seize sites du point 14 (hors `:2678`, historique exclu) disent « Administration → Sauvegarde et
    export », avec l'onglet quand le texte nomme une action (« onglet *Sauvegarder* », « onglet *Restaurer ou importer* »,
    « onglet *Exporter en CSV* ») et `\keshpath{/backup-export}` là où une adresse était écrite ; le manuel d'administration
    gagne une phrase sur la page (trois onglets, l'onglet d'export seul pour un comptable, anciennes adresses redirigées).
    Après la story, `git grep -nE 'Sauvegarde complète|Restaurer / Importer|→ (\\emph\{)?Export global|/admin/backup|/admin/restore'
    -- docs/manual/fr/*.tex docs/user-guide README.md` ne rend **plus** de chemin de menu (les seuls restes permis : le nom
    de la fonction « export global » en prose, et `\paragraph{Export global.}` `:2678`, listés au Dev Agent Record).
14. **AC14 — Recette visuelle.** Captures (thème clair, base de démonstration E2E) de chaque état touché, dans
    `/home/gcorbaz/devel/kesh-gate-logs/e505/dev/captures/` : menu Administration déplié (Admin, puis Comptable) ; page Admin
    sur chacun des trois onglets ; focus clavier visible sur un onglet ; page Comptable (un onglet) ; page de l'assistant
    `/onboarding/restore` (inchangée). Jointes au Dev Agent Record et au prompt de la revue de code.

## Tests à écrire, et la preuve du rouge

Rouge d'abord : chaque test neuf est écrit et lancé **avant** le code qu'il exige, son échec copié dans
`/home/gcorbaz/devel/kesh-gate-logs/e505/dev/rouge/`. Puis les mutations, chacune tuée par un test nommé (journal
`e505/dev/mut/`, restauration par `git restore` puis `touch` du fichier, mémoire *Mutation restaurée, binaire périmé*).

| Test (fichier) | Ce qu'il prouve | AC |
|---|---|---|
| `src/lib/components/ui/tabs/tabs.test.ts` (neuf) — harnais `TabsHarness.svelte` à trois onglets factices, `$app/state` et `$app/navigation` simulés (motif : `routes/(app)/reports/reports-page.test.ts`) | ARIA (rôles, `aria-controls`/`aria-labelledby`, tabindex itinérant) ; clavier (→, ←, circulaire, Origine, Fin, focus) ; clic → `goto` avec `?onglet=…`, `replaceState: true`, autres paramètres gardés ; onglet dérivé de `page.url` **après** montage (on change l'URL simulée, l'onglet suit) ; `onglet` inconnu ou invisible → premier visible ; aucun visible → rien ; montage paresseux et conservation (`hidden`) ; nom de paramètre `param` | AC4, AC5, AC6, AC12 |
| `src/routes/(app)/backup-export/backup-export-page.test.ts` (neuf) | Admin : trois onglets dans l'ordre, un seul `h1`, phrase d'en-tête, « Sauvegarder » par défaut ; Comptable et Consultation : un onglet, panneaux admin absents même avec `?onglet=sauvegarder` ; `ssr === false` | AC2, AC3 |
| `src/routes/(app)/admin/backup/admin-backup-redirect.test.ts`, `admin/restore/admin-restore-redirect.test.ts`, `export/export-redirect.test.ts` (neufs ; motif `admin/mcp/admin-mcp-page.test.ts`, `browser` simulé) | `load()` lève 301 vers la bonne adresse, pour Admin, Comptable et Consultation | AC7 |
| `src/lib/features/export/GlobalExportPanel.test.ts` (neuf) | bouton, garde de ré-entrance, état « Génération… », succès, erreur `ApiError` et repli i18n ; `<h2>`, pas de `<title>` | AC8, D8, D9 |
| `AdminRestorePanel.test.ts` (complété) | `embedded` : `<h2>`, `document.title` non écrit ; sans `embedded` : `<h1>` et `<title>` comme avant | AC9, D9 |
| `AdminBackupPanel.test.ts` (complété) | titre en `<h2>`, `admin-backup-title` gardé | D9 |
| `BackupScheduleBanner.test.ts:73-75` (modifié) | `href` = `/backup-export?onglet=sauvegarder` | AC10 |
| `crates/kesh-i18n/src/loader.rs` (`les_chemins_cites_aux_catalogues_existent`, modifié) | `ENTREES_ADMINISTRATION` : `nav-backup-export` remplace les trois ; le texte du bandeau passe | AC10 |
| `i18n-keys.test.ts` (modifié) | famille `backup-export-` résolue, couverture close, `sitesTotal` recompté par la garde | AC11 |
| E2E `frontend/tests/e2e/backup-export.spec.ts` (neuf) | AC1 (Admin et Comptable, comptable créé par l'API comme `backup-schedule.spec.ts:205-210`) ; AC2 ; AC3 (Comptable) ; AC4 (clic, rechargement, lien profond, bandeau depuis l'onglet d'export) ; AC5 (clavier réel) ; AC6 (saisie gardée, aucun `PUT` : `page.on('request')`) ; AC7 (trois anciennes adresses) ; captures d'AC14 (`KESH_E2E_CAPTURES_DIR`, motif `sidebar-navigation.spec.ts` `capture851`) | AC1-AC7, AC14 |
| E2E adaptés : `backup-schedule.spec.ts` (`:149`, `:222`, `:275` → `/backup-export?onglet=sauvegarder` ; `:214-216` : le comptable voit `nav a[href="/backup-export"]`, `/admin/backup` mène à `/backup-export?onglet=sauvegarder`, `auto-backup-section` absent, `export-global-start` visible, bandeau absent) ; `backup-restore-new-install.spec.ts` (`:104`, `:315` ; `:282`, `:328`, `:389` → `?onglet=restaurer` ; `:294` → `toHaveURL(/\/backup-export\?onglet=restaurer$/)`) ; `export-global.spec.ts:70` (→ `?onglet=exporter` ; l'assertion `heading level 1` reste vraie) ; `sidebar-navigation.spec.ts:219`, `:226-227` (`ADMINISTRATION`) | AC1, AC8 |

**Mutations** (une à la fois, chacune tuée) :

- **M1** — retirer le filtre `visible` du composant → `tabs.test.ts` (invisible rendu) et `backup-export-page.test.ts`
  (Comptable : trois onglets).
- **M2** — lire `onglet` une seule fois au montage (`onMount`) au lieu de le dériver de `page.url` → `tabs.test.ts`
  (« l'onglet suit l'URL après montage »).
- **M3** — démonter le panneau inactif (`{#if active}` au lieu de `hidden`) → `tabs.test.ts` (conservation) et E2E AC6.
- **M4** — rediriger `/admin/restore` vers `?onglet=sauvegarder` → `admin-restore-redirect.test.ts`.
- **M5** — `End` vers l'avant-dernier onglet → `tabs.test.ts`.
- **M6** — laisser l'entrée de menu dans `adminOnly` → E2E AC1 (Comptable).
- **M7** — ignorer `embedded` dans `AdminRestorePanel` → `AdminRestorePanel.test.ts` et `backup-export-page.test.ts`
  (un seul `h1`).
- **M8** — `goto` sans `replaceState` → `tabs.test.ts`.

## Tâches

1. **Étape 0** — relire au code les prémisses P1 à P4 (*Dev Notes*) et les lignes citées ; écrire le constat au Dev Agent
   Record, ligne lue citée ; un démenti s'arrête et se dit à l'orchestrateur.
2. **Composant d'onglets** (D3, D4) : `components/ui/tabs/` + harnais + `tabs.test.ts` ; rouge, puis vert. Commit.
3. **Panneau d'export** (D8, D9) : `features/export/GlobalExportPanel.svelte` + test ; `AdminBackupPanel` en `<h2>` sans
   `<svelte:head>` ; `AdminRestorePanel` reçoit `embedded`. Commit.
4. **Page** `/backup-export` (D1, D5, D9) + test ; **redirections** des trois anciennes routes (D2) + tests. Commit.
5. **Menu** (D6), **bandeau** (point 8), **catalogues** (D10, quatre locales), `loader.rs`, `i18n-keys.test.ts`. Commit.
6. **E2E** : spec neuve, quatre specs adaptées ; captures d'AC14. Commit.
7. **Manuels** (AC13) : `admin-manual.tex`, `user-manual.tex`, `docs/user-guide/fr/getting-started.md`, `README.md:52`.
   `make fr` dans `docs/manual/` pour contrôler la compilation ; **les PDF ne se commitent pas** (règle des fichiers
   communs). Commit.
8. **Gates** (*Gates* ci-dessous), mutations M1-M8, Dev Agent Record, File List, Change Log.

## Gates

- Bases propres à la story : `kesh_e505`, `kesh_ci_e505`, `kesh_e2e_e505` (`docs/agents-orchestration.md`) ; journaux
  sous `/home/gcorbaz/devel/kesh-gate-logs/e505/`. Tout sous `scripts/mem-guard.sh` et `flock` du verrou de la voie.
- **Entre deux étapes** : `npm run check`, `npm run lint-i18n-ownership`, `vitest` ciblé ; pour `loader.rs` :
  `cargo nextest run -p kesh-i18n`.
- **Au dernier commit de code** : gate complet (`scripts/gate-ci.sh`, la story touche `kesh-i18n` : catalogues lus par les
  gardes de `kesh-api`, dont `textes_coherents.rs` qui lit les manuels) ; frontend complet (`check`,
  `lint-i18n-ownership`, `test:unit`, `build`) ; **E2E complet** (« vert » = seulement les échecs attendus de
  `docs/testing.md`, jugés fichier par fichier).

## Note de version

**Modifié** — La sauvegarde complète, la restauration et l'export CSV sont réunis sur une page *Administration →
Sauvegarde et export*, à trois onglets (*Sauvegarder*, *Restaurer ou importer*, *Exporter en CSV*) ; un comptable n'y voit
que l'onglet d'export. Les anciennes adresses (`/admin/backup`, `/admin/restore`, `/export`) mènent à l'onglet
correspondant. Rien ne change dans ce que font ces écrans. (refs #1250)

## Dev Notes

### Prémisses à confirmer à l'étape 0

- **P1** — SvelteKit 2 accepte un dossier de route qui n'a qu'un `+page.ts` (sans `+page.svelte`) et exécute son `load()`
  côté client sous l'adaptateur statique. À vérifier par un essai (`npm run build`, puis ouverture de `/admin/backup`) ;
  sinon, garder un `+page.svelte` vide, commenté. Dans les deux cas, AC7 se constate en E2E.
- **P2** — `redirect(301, …)` levé dans un `load()` client avec `ssr = false` mène bien à l'adresse avec sa requête
  (`?onglet=…`) ; à défaut, `307`/`302` (le code n'a pas d'effet côté client ; le choix de 301 dit l'intention).
- **P3** — `goto` vers la même route avec une autre requête ne remonte pas la page (comportement sur lequel repose D4 et
  le commentaire de `reports/+page.svelte:553-566`).
- **P4** — Aucun test ni script n'énumère les fichiers `routes/(app)/export/+page.svelte`,
  `admin/backup/+page.svelte`, `admin/restore/+page.svelte` (relevé à `e5654fc00` : seuls des commentaires les nomment,
  `admin/mcp/admin-mcp-page.test.ts:4`, `onboarding/restore/+page.ts:2`, `+page.svelte:5`, `onboarding-restore-page.test.ts:2`,
  à mettre à jour s'ils deviennent faux).

### Modules touchés (règle de splitting)

Au grain du `CLAUDE.md` (crate, ou domaine `frontend/src/lib/features/<domaine>` dont le code de production change) :
**`kesh-i18n`** (catalogues ; `loader.rs` en test seulement), **`features/admin-backup`** (`AdminBackupPanel`,
`BackupScheduleBanner`), **`features/admin-restore`** (`AdminRestorePanel`), **`features/export`**
(`GlobalExportPanel`, neuf) : **quatre**. S'y ajoute le frontend partagé (`routes/(app)/` : page, redirections, menu ;
`components/ui/tabs/`), qui n'est pas un domaine `features/` : **cinq** si on le compte. Dans la borne, sans marge : rien ne
s'ajoute en cours de route ; une découverte qui toucherait un sixième module s'arrête et se dit.

### Fichiers

- Neufs : `frontend/src/lib/components/ui/tabs/{tabs.svelte,index.ts,tabs.test.ts,TabsHarness.svelte}` (le harnais à côté
  du test, ou sous `__tests__/` selon l'usage du dépôt) ; `frontend/src/routes/(app)/backup-export/{+page.svelte,+page.ts,backup-export-page.test.ts}` ;
  `frontend/src/lib/features/export/{GlobalExportPanel.svelte,GlobalExportPanel.test.ts}` ; trois tests de redirection ;
  `frontend/tests/e2e/backup-export.spec.ts`.
- Modifiés : `frontend/src/routes/(app)/+layout.svelte` ; `routes/(app)/admin/backup/+page.ts`,
  `admin/restore/+page.ts`, `export/+page.ts` ; retirés (P1) : leurs `+page.svelte` ; `features/admin-backup/{AdminBackupPanel.svelte,AdminBackupPanel.test.ts,BackupScheduleBanner.svelte,BackupScheduleBanner.test.ts}` ;
  `features/admin-restore/{AdminRestorePanel.svelte,AdminRestorePanel.test.ts}` ;
  `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` ; `crates/kesh-i18n/src/loader.rs` (test) ;
  `frontend/src/lib/shared/i18n-keys.test.ts` ; quatre specs E2E ; `docs/manual/fr/admin-manual.tex`,
  `docs/manual/fr/user-manual.tex`, `docs/user-guide/fr/getting-started.md`, `README.md`.
- **Fichiers communs avec les autres stories de la release** : les quatre catalogues (chaque story dans son bloc ; conflit
  résolu en gardant les deux côtés), `i18n-keys.test.ts` (`sitesTotal` recompté après fusion), `+layout.svelte` (le menu :
  l'Epic 38 PayPal y a ajouté `comptableOnly` au groupe Mensuel, déjà dans la base), `admin-manual.tex`. Aucun fichier de
  48-11, 48-12a/b n'est touché par ailleurs (ils sont sur les factures fournisseurs et l'arrondi).

### Ce qui doit rester vrai

- Les droits : le serveur reste l'autorité (`require_admin_role` pour sauvegarde et import ; tous rôles pour l'export) ;
  l'écran ne rend ni la sauvegarde ni la restauration pour un non-Admin.
- `AdminRestorePanel` : maintien de session, `onDestroy`, « Se reconnecter », rendu de l'assistant — inchangés (D4, D9).
- `AutoBackupSection` : ses lectures au montage, son 409, ses avis — inchangés.
- Les `data-testid` existants des panneaux (`admin-backup-*`, `admin-restore-*`, `auto-backup-*`, `export-global-*`) :
  inchangés (garde KF-043, sélecteurs des E2E).
- Aucune facture, aucun avoir, aucun PDF, aucune écriture n'est touché : la story ne lit ni n'écrit rien de comptable.

### Hors périmètre

- **#1253** (sous-groupes du menu) : release ultérieure ; D6 ne lui ajoute aucune contrainte.
- **#1249, #1251, #1252** : pages MCP, Facturation, Paramètres ; elles prendront le composant tel quel (et l'avertissement
  de D7 selon l'arbitrage).
- Migration de `reports`, `email-templates`, `due-dates` vers le composant (D3) : **issue à ouvrir** par l'orchestrateur
  (`technical-debt`, P4) si le Project Lead le veut — question au rapport de la fiche.
- Textes des panneaux (descriptions « À distinguer de l'export global d'une seule société », etc.) : inchangés, même s'ils
  disent désormais ce que la phrase d'en-tête dit aussi.

### Références

- `CLAUDE.md` : § *Qualité du code* (DRY), § *Recette visuelle*, § *E2E (Playwright)*, § *Règle de splitting préventif*,
  § *Review Iteration Rule* (le prompt de passe nomme le manuel, PDF compris), § *Documentation à synchroniser* (PDF non
  commités dans `story/…`).
- Issues : #1250, #1248, #1249, #1251, #1252, #1253.
- Motifs : `routes/(app)/reports/+page.svelte:589-614` (clavier, `goto` en `replaceState`) ;
  `routes/(app)/admin/mcp/admin-mcp-page.test.ts` (test d'un `load()` de garde) ; `sidebar-navigation.spec.ts`
  (captures, liste `ADMINISTRATION`).

### Charge estimée

Petite à moyenne : un composant d'environ 120 lignes et son test, une page de 50 lignes, un panneau extrait sans
réécriture, trois redirections, une dizaine de lignes de menu et de catalogues, une spec E2E neuve et quatre adaptées,
seize phrases de manuel. **Environ une journée de développement**, plus la boucle de revue et un E2E complet.

## Dev Agent Record

*(vide)*

## Change Log

| Date | Auteur | Entrée |
|---|---|---|
| 2026-11-24 | agent de rédaction (Opus 5.5) | Création de la fiche par `bmad-create-story`, en autonomie ; statut `drafted` ; validation P1 à lancer (Sonnet, prompt `50-5-page-sauvegarde-export-validation-prompt-p1.md`). Points à arbitrer : D7 (#1248 fermé ou scindé), issue de dette pour la migration des trois `tablist` existants. |
