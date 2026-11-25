# Prompt — validation P1 de la fiche `1582-nom-comptabilite` (#1582), passe complète

Tu valides, **en lecture seule**, la fiche de story `1582-nom-comptabilite` de Kesh (comptabilité suisse ; Rust/Axum, MariaDB,
SvelteKit 5 runes), hors epic, visée **v0.20.0** (coupure le jeudi 2026-11-26 au soir ; si elle n'est pas close, elle passe à la
v0.21.0 sans retarder la release). Kesh tient les livres réels du Project Lead (Guy). La story est **frontend seul : aucune
migration, aucun changement d'API ni de serveur** ; elle affiche le nom de la comptabilité ouverte sous le logo et dans le titre de
l'onglet (décisions du Project Lead du 2026-11-25, dans l'issue #1582).

- **Modèle : Opus. Contexte frais. Passe complète** (première passe : jamais ciblée, jamais Haiku). Langue du rapport : français.
- Worktree : `/home/gcorbaz/devel/kesh-1582`, branche `story/1582-nom-comptabilite`, partie de `main` = `1e23edb55`. Fiche :
  `/home/gcorbaz/devel/kesh-1582/_bmad-output/implementation-artifacts/1582-nom-comptabilite.md`.
- Lis d'abord : le `CLAUDE.md` du worktree (Qualité du code — DRY, Recette visuelle, Test Locally First, Review Iteration Rule,
  Inventorier les sites NON RÉSOLUS, Propagation post-patch, Priorités des défauts, Issue Tracking Rule, Règle de splitting préventif,
  Documentation à synchroniser) ; `docs/agents-orchestration.md` ; l'issue (`gh issue view 1582 --json number,title,body,comments`,
  lecture seule).

## Interdits

Tu n'écris **aucun fichier** hors de ton rapport. En particulier, tu ne lances **ni** `cargo` (build, test, nextest, clippy, fmt),
**ni** `npm` / `npx` (check, test, build, lint), **ni** `make`, **ni** aucun `scripts/*` (dont `scripts/prepare-release.sh`,
`scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`, `scripts/recette-montee.sh`, `scripts/install-hooks.sh`),
**ni** `docker` / `docker compose`, **ni** `mariadb` / `sqlx`, **ni** `curl` / `wget`, **ni** `git commit`, `git checkout`, `git stash`,
`git restore`, `git reset`, `git push`, `git worktree`, **ni** `gh` en écriture (`issue create|edit|comment|close`, `pr …`), **ni**
`sed -i`, `perl -pi`, `tee`, `>`, `>>` ou toute autre redirection vers un fichier (même « jetable », même dans le scratchpad), **ni**
`pkill`, `killall` ni `kill` par motif, **ni** `python3` ou tout autre interpréteur (lire avec `grep`, `sed -n`, `cat`). Rien sous
`~/kesh/`. Permis : `git log`, `git show`, `git diff`, `git grep`, `grep`, `sed -n`, `cat`, `ls`, `wc`, `pdftotext … -` (vers la sortie
standard, **en tube**, jamais vers un fichier), `gh … view|list`.

Rapport : **`/home/gcorbaz/devel/kesh-gate-logs/s1582/validation-p1.md`** (le seul fichier que tu écris, par l'outil d'écriture).

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

1. **Prémisses au code.** Chaque ligne citée par la fiche (constats 1 à 10) est relue à `1e23edb55` ; une ligne fausse, une prémisse
   démentie, une cause plus profonde manquée est un finding. En particulier : le logo est-il bien dans l'en-tête et non dans le menu
   (`frontend/src/routes/(app)/+layout.svelte:311-314`) ; **existe-t-il un menu latéral replié** que la fiche aurait manqué
   (`grep -rn "collaps\|replié\|sidebar" frontend/src`) ; `GET /api/v1/companies/current` est-il lisible par **tout** rôle (y compris
   Consultation) et rend-il le nom ; le nombre de pages à `<title>` (50) ; leur forme.
2. **Source du nom (D1).** `companies.name` est-il vraiment « Prénom Nom » pour une personne physique
   (`crates/kesh-api/src/routes/onboarding.rs:542-545`), et la correction du nom laisse-t-elle bien `first_name` / `last_name` périmés
   (`routes/companies.rs:539-565`) ? Y a-t-il **un autre chemin** qui écrit `name` ou `first_name` / `last_name` (import
   d'installation, restauration `.keshbackup`, seed, MCP, `PUT /companies/current/*`) qui rendrait `name` faux pour une personne
   physique ? La décision de ne pas recomposer est-elle sûre, ou préférable-t-il `firstName lastName` quand `orgType` vaut
   `Independant` ? Écart à la lettre de l'issue correctement signalé ?
3. **Mécanisme du titre (D6).** Un `MutationObserver` sur `document.head` tient-il avec **Svelte 5** (comment `<svelte:head><title>`
   écrit-il : `document.title`, texte du nœud, élément recréé ?) — lis la source de la version installée dans `frontend/node_modules`
   (`svelte/src/internal/client/dom/blocks/svelte-head.js`, `…/elements/…`, selon la version de `frontend/package.json`) **sans rien
   exécuter** ; boucle de réécriture possible ; titre posé par la page **avant** le montage du layout ; page qui réécrit le même titre ;
   navigation SvelteKit entre pages (un titre non réécrit reste-t-il décoré, ou perd-il sa décoration ?) ; retour à une page hors `(app)`
   (login, onboarding) : le titre est-il bien restauré, aucun nom d'une société qui n'est plus la bonne ? Une alternative plus simple
   et sûre existe-t-elle, qui ne touche pas les 50 pages ?
4. **Format du titre (D5).** `Kesh — <nom> — <page>` : la fonction `titleWithCompany` décrite couvre-t-elle toutes les formes réelles
   (`grep -rn "<title>" frontend/src`, dont `Kesh — Rapports`, `Erreur {page.status} - Kesh`, les titres calculés `{entry ? … : …}`) ?
   Pièges : nom contenant « Kesh », « — » ou des caractères spéciaux ; titre vide.
5. **Cycle de vie du store (D9, AC8, AC10).** Le store de module survit au démontage du layout : le scénario « Réinitialiser la
   démonstration » puis retour est-il bien couvert ? Autres passages hors `(app)` : déconnexion, `/onboarding`, `/setup`, expiration de
   session, **changement d'utilisateur dans le même onglet** (nom d'une société précédente) ? Une lecture en vol, un échec, un 401
   (renouvellement du jeton) : que fait `apiClient` et le store ? `authState.isAuthenticated` : l'`$effect` se déclenche-t-il au bon
   moment (avant la fin du `load` du layout ? `+layout.ts` redirige les étapes < 3/6) ?
6. **Accessibilité (AC9, décision 6).** Nom accessible du lien « Kesh <nom> » : calcul réel (éléments de bloc, espaces) ; `title`
   en doublon ; texte coupé par `truncate` lu en entier ; contraste de `text-text` sur `bg-surface` (lis `frontend/src/app.css` ou le
   thème ; la fiche impose le test d'accessibilité existant, existe-t-il et couvre-t-il l'en-tête ?) ; `data-testid`.
7. **Mise en page (AC4, AC5).** 56 px de l'en-tête, `max-w-56`, `truncate` dans un conteneur flex (`min-w-0` nécessaire ?), déplacement
   du champ de recherche selon le nom : la fiche l'a-t-elle mesuré ou le suppose-t-elle ? Largeur de l'`<aside>` ; groupes repliés ;
   **question Q1** : « menu replié » = « groupes repliés » est-il la lecture raisonnable de la décision 3 de l'issue ?
8. **Réglages (D8).** `adoptCompany` (`settings/+page.svelte:217`) est-il le **seul** point de passage de toutes les écritures de la
   société ? Une écriture qui ne passe pas par lui (conflit de version et relecture `reloadAfterConflict`, `full` = relecture) laisse-t-elle
   le store périmé ? Le placement après `adoptionDecision` est-il juste ? Autres écrans qui changent le nom (assistant, restauration) ?
9. **Critères et tests.** Chaque AC est testable et rougit pour la bonne raison ; mutations M1 à M10 uniques, chacune tuée par un test
   nommé (vérifie que le test nommé **peut** la tuer) ; sondes positives avant les assertions d'absence ; aucun test vrai par
   construction (les deux côtés du même extracteur) ; E2E pour chaque comportement visible, la raison donnée pour le cas de la personne
   physique (assistant par l'API) tient-elle ? Étendre `settings-page.test.ts` : le mock existant le permet-il ? Gardes à ne pas casser
   (`i18n-keys.test.ts`, `lint-i18n-ownership`, KF-043 sélecteurs).
10. **DRY et réutilisation.** `fetchCompanyCurrent` réutilisé ; un store déjà existant qui porterait le nom (grep de `companies/current`,
    `company.name` dans les stores) ; un composant de logo ou de titre déjà existant ; la duplication entre `AppBrand` et le layout.
11. **Manuel, PDF compris.** `docs/manual/fr/user-manual.tex:150-152` **et** le PDF publié, aplati avant de greper :
    `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o '.\{0,200\}Barre sup.\{0,300\}'` (motifs :
    « Barre supérieure », « logo », « titre de l'onglet »). Le manuel dit-il aujourd'hui autre chose que le code ? Les autres phrases
    qui décrivent l'en-tête ou l'onglet (`grep -n "onglet\|logo\|en-tête" docs/manual/fr/*.tex`) ? Les manuels DE/IT/EN existent-ils
    (`ls docs/manual/de docs/manual/en docs/manual/it`) ? L'AC11 suffit-elle ? Le guide de l'API, le site (`website/`) et le `README.md`
    décrivent-ils l'en-tête ?
12. **Règle de splitting et périmètre.** Recompte les modules au grain du domaine depuis la liste des fichiers touchés ; au-delà de
    cinq, finding. La fiche élargit-elle l'issue (D8, D9, AC10) au-delà de ce que le Project Lead a décidé ? Rien en dehors de l'écran.
13. **Notes de version et documentation.** Une puce, sans nommer d'epic, numéro de release seulement ; aucun mot-clé de fermeture
    (`closes`, `fixes`, `resolves` et variantes) dans la fiche ni dans les messages de commit (`git log origin/main..HEAD --format=%B`) ;
    cohérence de `sprint-status.yaml` (clé `1582-nom-comptabilite`, bloc, statut).
14. **Fichiers communs et coordination.** Conflits possibles avec `v0200-menu-sous-groupes` (même `+layout.svelte`), l'Epic 53 et les PR
    en cours (`gh pr list --state open`).

## Rapport

Pour chaque finding : identifiant `V1-n`, sévérité (CRITICAL, HIGH, MEDIUM, LOW), axe, constat **avec la ligne lue citée**
(`chemin:ligne` et l'extrait), conséquence, remède proposé. Pour tout CRITICAL ou HIGH qui affirme l'absence d'un code ou la présence
d'un anti-pattern, la commande `grep -nF` qui le prouve et sa sortie.
En fin de rapport : décompte **CRITICAL / HIGH / MEDIUM / LOW**, recompté depuis tes findings ; la liste des **axes exercés** et des
**axes non exercés** (avec la raison) ; les commandes lancées ; et ton avis sur les trois questions du Project Lead (Q1, Q2, Q3 du
Change Log de la fiche). Un « 0 finding » sans cette liste ne compte pas comme passe.
