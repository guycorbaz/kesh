# Prompt — validation P2 de la fiche `50-5-page-sauvegarde-export` (#1250, #1248), passe complète

Tu valides, **en lecture seule**, la fiche de story `50-5-page-sauvegarde-export` de Kesh (comptabilité suisse ; Rust/Axum,
MariaDB, SvelteKit 5 runes), Epic 50, release **0.20.0** (coupure le vendredi 2026-11-27 au matin ; la story sort de la release
si elle n'est pas close à temps). Kesh tient les livres réels du Project Lead (Guy) : la story est une réorganisation d'écran
**sans changement de fonctionnement** ; aucune migration, aucun code Rust de production.

- **Modèle : Opus. Contexte frais. Passe complète** (la P1, Sonnet, a rendu 0/0/3/12, remédiés ; tu ne lis **pas** son
  rapport avant d'avoir fini le tien : ta lecture ne doit pas s'ancrer sur la sienne ; tu peux le lire ensuite pour vérifier
  que chaque remède est réel, axe 14). Langue du rapport : français.
- Worktree : `/home/gcorbaz/devel/kesh-50-5`, branche `story/50-5-page-sauvegarde-export`, partie de
  `origin/epic/50-corrections` = `e5654fc00`. Fiche :
  `/home/gcorbaz/devel/kesh-50-5/_bmad-output/implementation-artifacts/50-5-page-sauvegarde-export.md`.
- Lis d'abord : le `CLAUDE.md` du worktree (Qualité du code — DRY ; Recette visuelle ; E2E (Playwright) ; Review Iteration
  Rule, dont *Le prompt d'une passe doit NOMMER le manuel* et *Inventorier les sites NON RÉSOLUS* ; Règle de splitting
  préventif ; Issue Tracking Rule ; Documentation à synchroniser ; Fusions et fichiers communs) ;
  `/home/gcorbaz/devel/kesh-gate-logs/prompts/e0200-commun.md` ; le fichier d'epic
  `_bmad-output/planning-artifacts/epic-50-corrections.md` ; les issues, **une commande `gh` par minute au plus**, lecture
  seule (`gh issue view <N> --json number,title,labels,milestone,body`) : #1250, #1248, #1253, puis, pour la cohérence de la
  série, #1249, #1251, #1252.

## Interdits

Tu n'écris **aucun fichier** hors de ton rapport. En particulier, tu ne lances **ni** `cargo` (build, test, nextest, clippy,
fmt), **ni** `npm` / `npx` (check, test, build, lint, playwright), **ni** `make`, **ni** aucun `scripts/*` (dont
`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`,
`scripts/recette-montee.sh`, `scripts/install-hooks.sh`), **ni** `docker` / `docker compose`, **ni** `mariadb` / `sqlx`, **ni**
`git commit`, `git checkout`, `git switch`, `git stash`, `git restore`, `git reset`, `git push`, `git merge`, `git worktree`,
**ni** `gh` en écriture (`issue create|edit|comment|close`, `pr …`, `project …`), **ni** `sed -i`, `tee`, `>` ou `>>` vers un
fichier du dépôt. Rien sous `~/kesh/` (production). Permis : `git log`, `git show`, `git diff`, `git grep`, `grep`, `sed -n`,
`cat`, `ls`, `pdftotext … -` (vers la sortie standard), `gh issue view|list`, et un `python3 -I` qui ne fait que **lire**.

Rapport : **`/home/gcorbaz/devel/kesh-gate-logs/e505/validation-p2.md`** (le seul fichier que tu écris).

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

1. **Prémisses relues au code.** Chaque affirmation de § *Ce qui se lit au code* (`chemin:ligne`) est relue à `e5654fc00` ;
   une ligne fausse, une prémisse démentie ou une cause manquée est un finding. En particulier : les gardes de
   `routes/(app)/admin/backup/+page.ts` et `admin/restore/+page.ts` ; `routes/(app)/export/+page.svelte` ;
   `routes/(app)/+layout.svelte` (`:137`, `:162-176`, `findGroupKeyForPath`, `navTestid`) ; les `<svelte:head>` et `<h1>` de
   `AdminBackupPanel.svelte` et `AdminRestorePanel.svelte` ; l'`onDestroy` d'`AdminRestorePanel.svelte` (`:99` et l'en-tête) ;
   `AutoBackupSection.svelte` ; `BackupScheduleBanner.svelte:66-67` ; `crates/kesh-api/src/routes/exports.rs:34-37` (droits
   de l'export) ; `crates/kesh-i18n/src/loader.rs:1174-1190` ; `i18n-keys.test.ts` (`FAMILLES_RESOLUES`, `sitesTotal`,
   `PREFIXES_A_COUVERTURE_CLOSE`) ; `scripts/lint-i18n-ownership.js`. Les prémisses P1 à P4 des *Dev Notes* : lis ce que
   tu peux au code de SvelteKit (`frontend/node_modules/@sveltejs/kit/src/`, lecture seule) pour P1 (route à `+page.ts` seul)
   et P2 (`redirect` dans un `load` client).
2. **Droits.** Qui voit quoi : la fiche dit Admin trois onglets, Comptable et Consultation l'onglet d'export seul, droits du
   serveur inchangés. Vérifie au code (routeur `crates/kesh-api/src/lib.rs` vers `:1510`, `require_admin_role` des routes de
   sauvegarde et d'import) qu'aucun rôle ne gagne ni ne perd un accès ; que la conséquence de D2 (un comptable qui ouvre
   `/admin/backup` arrive sur l'export au lieu de l'accueil) n'expose rien ; que D5 (pas de réécriture de l'adresse) ne laisse
   aucun panneau admin se monter pour un non-Admin, y compris pendant le chargement de l'utilisateur (`(app)/+layout.ts`,
   `auth.svelte.ts`).
3. **Décisions.** D1 à D10 : chacune justifiée, option écartée nommée, cohérente avec #1250 (onglets, ordre, phrase d'en-tête,
   anciennes adresses) et #1248 (accessibilité, `?onglet=`, filtre par rôle, bouton Enregistrer par onglet, avertissement).
   En particulier : D4 (panneaux conservés) ferme-t-il vraiment les deux risques nommés (fin de session à l'`onDestroy`,
   saisie perdue) sans en ouvrir un autre (un import en cours dans un onglet caché ; deux panneaux qui écrivent l'état du
   bandeau) ? D7 (pas d'avertissement ; #1248 fermé avec la story, décision du Project Lead
   du 2026-11-24) : la fiche livre-t-elle tout ce que #1248 exige, sauf ce que la décision écarte, et `beforeNavigate` est-il le
   bon point d'accroche pour un futur garde-fou ? D3 : voir l'axe 15.
4. **DRY et réutilisation.** Le composant de D3 sert-il tel quel #1249, #1251, #1252 (API, nom du paramètre, filtre par rôle,
   liens profonds depuis un message d'erreur) ? Ne duplique-t-il rien de ce qui existe (`reports/+page.svelte:589-614`) sans
   le dire ? Le panneau d'export de D8 est-il un déplacement sans réécriture ?
5. **#1253 non gêné.** La fiche ajoute-t-elle au menu une structure ou une contrainte que la story des sous-groupes devrait
   défaire ? L'inventaire des chemins du manuel lui est-il réutilisable ?
6. **Inventaire du symptôme, ensemble clos.** Refais toi-même les relevés : (a) tout site qui écrit une des trois anciennes
   adresses ou des trois anciens libellés du menu — code, tests unitaires, E2E, catalogues des quatre locales, messages Rust,
   `docs/`, manuels, `README.md`, `website/` ; (b) tout site qui compte sur le `<h1>` ou le `<title>` d'un des trois
   panneaux ; (c) tout test ou script qui énumère les fichiers de route touchés. Un site manqué, ou un site rangé « hors
   périmètre » sans raison, est un finding.
7. **Critères et tests.** Chaque AC est constatable et rougit pour la bonne raison ; mutations M1 à M8 distinctes, chacune
   tuée par un test nommé ; aucune assertion vraie par construction ; sondes positives avant une assertion d'absence (un
   `toHaveCount(0)` sur un sélecteur qui ne trouverait rien de toute façon) ; E2E pour tout comportement visible ; sélecteurs
   `data-testid` (garde KF-043) ; les E2E adaptés gardent leur force (rien d'affaibli pour passer).
8. **i18n.** Les six clés aux quatre locales, traductions contre `docs/i18n-glossaire.md` ; retrait des trois `nav-*` sans
   appelant restant ; `admin-backup-banner-link` contre G3-bis ; `lint-i18n-ownership` (la page hors `features/`, le panneau
   d'export dans `features/export`) ; `parity_between_locales` et `dette-parite-connue.txt` ; famille et couverture close de
   `i18n-keys.test.ts`.
9. **Manuel, PDF compris.** Vérifie `docs/manual/fr/admin-manual.tex` et `docs/manual/fr/user-manual.tex` contre le code et
   contre la fiche, **et les PDF publiés**, aplatis avant de greper :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o '.\{0,160\}<motif>.\{0,160\}'` (motifs :
   « Sauvegarde complète », « Restaurer / Importer », « Export global », « /admin/backup », « /admin/restore », « Exporter
   toute l'installation », « Lancer l'export »), et de même pour `user-manual.pdf`. Ce que les manuels disent aujourd'hui de
   ces écrans est-il exact au code avant la story, et l'AC13 suffit-elle à le rendre exact après ? Les PDF, qui ne se
   régénèrent qu'à la release, diront-ils l'ancien chemin d'ici là (la redirection de D2 le couvre-t-elle) ? Les manuels
   DE/IT/EN décrivent-ils ces écrans ?
10. **Recette visuelle.** L'AC14 couvre-t-elle **chaque** état touché (menu des deux rôles, trois onglets, focus clavier,
    page Comptable, assistant inchangé) ? Manque-t-il un état (onglet de restauration pendant un import, bandeau) ?
11. **Règle de splitting.** Recompte les modules au grain du `CLAUDE.md` depuis la liste des fichiers de la fiche ; plus de
    cinq est un finding. La story est-elle assez **petite** pour se clore avant la coupure (charge estimée crédible) ?
12. **Note de version et commits.** Une note « Modifié », numéro de release seulement, sans epic ; aucun mot-clé de fermeture
    dans la fiche ni dans les messages de commit :
    `git log origin/epic/50-corrections..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
    doit être vide.
13. **Fichiers communs et coordination.** Conflits possibles avec les autres stories de l'Epic 50 (48-11, 48-12a, 48-12b,
    solde) et de la release (Epic 49) : catalogues, `i18n-keys.test.ts`, `+layout.svelte`, `admin-manual.tex`.

14. **Remèdes de la P1 réels.** Après ton propre rapport, lis `/home/gcorbaz/devel/kesh-gate-logs/e505/validation-p1.md` et le
    Change Log de la fiche : chaque finding V1-1 à V1-16 est-il traité au fond (et non par une phrase) ? Un remède qui en ouvre
    un autre est un finding.
15. **L'enveloppe de `bits-ui`, à la source.** Lis `frontend/node_modules/bits-ui/dist/bits/tabs/` du **dépôt principal**
    (`/home/gcorbaz/devel/kesh/frontend/node_modules/bits-ui/`, version dans `package.json`, lecture seule) et
    `dist/internal/roving-focus-group.js`, et vérifie **ce que `bits-ui` fournit réellement** contre ce que D3 et AC5 lui
    prêtent : rôles et attributs ARIA, tabindex itinérant, clavier (flèches, `Home`, `End`, boucle, index depuis le nœud
    focalisé), sélection qui suit le focus en mode automatique, `hidden` sur le contenu inactif, montage des contenus ;
    comportement d'un `value` **contrôlé non lié** (`value={x}` sans `bind:`) avec `onValueChange` : la valeur locale
    écrite par le setter du `boxWith` de `components/tabs.svelte` se resynchronise-t-elle quand la propriété change, et
    sans boucle quand `onValueChange` appelle `goto` (prémisse P6) ? Un onglet retiré (invisible) se désinscrit-il proprement
    (`registerTrigger` / `registerContent`) ? Compare aux enveloppes existantes (`frontend/src/lib/components/ui/separator/`,
    `ui/dialog/`) : la forme proposée est-elle la même ? Une affirmation de la fiche que la source dément est un finding.
16. **Le tueur de M2 (V1-3).** Le test (a) d'`url-tabs.test.ts` peut-il réellement rougir sous M2 et verdir sans elle ?
    Examine la prémisse P5 : un `vi.mock('$app/state', () => import('./page-mock.svelte'))` vers un module `.svelte.ts` qui
    exporte un `$state` est-il compilé avec les runes par la configuration du dépôt (`frontend/vite.config.ts`,
    `vitest` ; lis, ne lance rien), et la réassignation `page.url = new URL(…)` est-elle vue par le `$derived` du composant
    (objet `$state` profond, propriété réassignée) ? Le repli écrit en P5 tient-il ? L'E2E du bandeau : le stub de
    `GET /api/v1/admin/backup-schedule` affiche-t-il bien le bandeau (lis `BackupScheduleBanner.svelte` et sa condition
    d'affichage), le marqueur `window` distingue-t-il un rechargement complet d'une navigation client, et les deux échecs
    attendus écrits sont-ils ceux que vitest et Playwright imprimeraient ?

## Rapport

Pour chaque finding : identifiant `V2-n`, sévérité (CRITICAL, HIGH, MEDIUM, LOW), axe, constat **avec la ligne lue citée**
(`chemin:ligne` et l'extrait), conséquence, remède proposé. Pour tout CRITICAL ou HIGH qui affirme l'absence d'un code ou la
présence d'un anti-pattern, la commande `grep -nF` qui le prouve et sa sortie.

En fin de rapport : décompte **CRITICAL / HIGH / MEDIUM / LOW**, recompté depuis tes findings ; verdict (la fiche peut-elle
passer en développement après remédiation) ; la liste des **axes exercés** et des **axes non exercés** (avec la raison) ; les
commandes lancées. Un « 0 finding » sans cette liste ne compte pas comme passe.
