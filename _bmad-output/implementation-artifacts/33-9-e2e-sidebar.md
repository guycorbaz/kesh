# Story 33-9 : le test E2E `sidebar-navigation` cesse de lancer de fausses alertes (#424, #287)

Status: ready-for-dev

*Fiche rédigée le 2026-11-06 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`, branche `story/33-0-kickoff`,
numéros de ligne de `4ef19901`. Release **0.17.5**, Epic 33. **Aucune migration, a priori aucun code de production.** Registre :
`e33-choix-autonomes-e334.md`, C-e334-19, 20. Commits en `refs` ; à la PR : `closes #424`. ⚠️ **#287 est déjà fermée**, reportée dans #424 à
l'audit du 2026-11-06 : aucun mot-clé sur elle ; ses mesures, elles, valent.*

## Story

En tant que développeur qui lit un rouge d'E2E, je veux que `persistence localStorage : Administration ouvert puis reload reste ouvert` ne rougisse que si la persistance
est cassée, pour qu'on n'ait plus à rejouer ce test pour savoir si c'est la pollution, la KF-052, ou un défaut.

## Ce que le code dit (prémisses établies)

- **Le test est à `frontend/tests/e2e/sidebar-navigation.spec.ts:81`** (non `:75`, ni `:71` : `docs/testing.md:402,421,448` et `CLAUDE.md` citent `:75`).
- ⚠️ **Prémisse à réfuter à l'exécution — « la route d'atterrissage ».** `docs/testing.md:402,416` et #424 attribuent la KF-052 à une route d'atterrissage dans le groupe. Mais `loginAsAdmin` **exige `toHaveURL('/')`** (`:36`), et `/` appartient à *Quotidien* (`nav-link-home`, `:132`), non à Administration. Le groupe ne peut donc pas s'auto-ouvrir à cet endroit… **si** cette assertion existait déjà à l'époque (`git log -L` à regarder) et si `/` est bien la route rendue. Le raisonnement ne suffit pas : **une hypothèse éliminée par raisonnement n'est pas une hypothèse testée** (`CLAUDE.md`).
- **L'hypothèse restante, celle de #287** (commentaire du 2026-08-24) : `<details ontoggle=…>` (`(app)/+layout.svelte:391-393`), `handleToggle` écrit le stockage (`:268-272`). L'attribut `open` se pose à l'activation ; **`toggle` est mis en file**, et l'écriture suit. Le test constate `open`, puis lit `localStorage` **une seule fois** (`spec:91-93`). Une course, dont l'issue dépend de la charge : le profil « déterministe ici, intermittent ailleurs » que #287 décrit. L'application est un SPA (`frontend/src/routes/+layout.ts:5`, `(app)/+layout.ts:7`, `ssr = false`) : pas de course d'hydratation à ajouter.
- **Le dernier relevé** : `rel0174/pr-corps.md:43` : rouge dans la suite, vert rejoué seul. `e2e-rejeu-sidebar.log` est au dossier. **Ce profil-là n'est pas celui de « déterministe et échoue seul »** de `docs/testing.md:423` : `docs/testing.md:421-425` en consigne deux, et dix tirages « spec seule » peuvent n'en montrer qu'un (d'où les deux protocoles de l'étape 1, et le profil écrit avec le verdict).
- **Le bon sujet, peut-être, n'est pas le test seul** : `expandedState` s'initialise à la création du composant (`:242-246`), `$effect` déplie selon le chemin (`:258-266`). Le toggle `false` initial d'un `<details>` **re-rendu** pourrait aussi écrire : à observer, ne pas supposer.
- **Le contexte navigateur est neuf à chaque test** (fixture `page`) : la pollution ne vient pas de `localStorage` mais de la **base** (`seedTestState` en `beforeAll`, `:23`, `workers: 1`) ou de la charge. Le test `:34`, lui aussi, a déjà rougi (#287).
- ⚠️ **`docs/testing.md:421-429`** bâtit le compte attendu sur la KF-052 : `7 + (1 si KF-052) + 0 à 2` = **7 à 10**. `CLAUDE.md:191-195` le répète. Le compte devient **7 à 9** si elle disparaît, et `sidebar-navigation` sort des listes de pollution **actuelles**, mais pas des relevés datés (qui restent : **on ne réécrit pas un relevé passé**).

## Étape 1 — reproduire d'abord, aucune cause supposée (C-e334-19)

Sous le double `flock`, base `kesh_e2e_e339` neuve, montage de `docs/testing.md`, sans toucher au test :

1. **Spec seule, dix tirages** ; puis **suite complète, un tirage** si le premier protocole ne rougit jamais ; sorties gardées dans `kesh-gate-logs/e339/`.
2. **Copie instrumentée, hors dépôt ou jetée** (`sidebar-probe.spec.ts`, non commitée) qui journalise : l'URL après connexion ; `open` **avant** le clic ; `localStorage` juste après `open`, puis à +50 ms, +500 ms, +5 s (`expect.poll`) ; tout événement `toggle` (`page.addInitScript`, écouteur capturant) avec son état ; et au T0, `page.url()` et `open` au moment du clic.
3. **Trois verdicts possibles, à écrire au Change Log avec la sortie** :
   - **(a) écriture tardive** (`null` puis `'true'`) : le test lit trop tôt. Correctif au test.
   - **(b) jamais écrite** ou **écrite puis défaite** : **défaut de production** : un groupe qu'on replie se rouvre. On corrige `+layout.svelte`, et on signale la portée à l'orchestrateur, ce qui change la nature de la story (visible, manuel à relire).
   - **(c) groupe déjà ouvert avant le clic.** ⚠️ **Écarté d'avance comme explication du symptôme relevé, gardé comme sonde.** Le symptôme de #424 et de #287 est : `toHaveAttribute('open', '')` **passe**, puis `Expected "true", Received null`. Or un clic sur un groupe ouvert le **ferme** (action par défaut de `<summary>`) : l'assertion `open` échouerait alors, avant toute lecture du stockage, et le `toggle` écrirait `'false'`, non `null`. La « mécanique probable » de #424 (« l'attribut `open` reste vrai ») ne tient donc pas au code lu ; **ceci reste un raisonnement, et la sonde le juge** : si elle voit le groupe ouvert avant le clic, c'est un autre défaut que le symptôme connu, avec son propre message, et on identifie la route et l'état de base qui y mènent. Il reste, comme suspects de ce symptôme, (a) et (b).
4. Une fois la cause nommée : **si elle n'est pas reproductible** en dix tirages et un tirage de suite, le dire, garder le durcissement ci-dessous, et **ne pas écrire « cause établie »** : la mutation M1 reste la preuve de pouvoir, le verdict reste « course compatible, non observée ».

## Conception du test (sous réserve du verdict)

Le test **pose son état de départ et mesure une transition** ; il garde tout ce qu'il vérifiait, et y ajoute (C-e334-20) :

- **Départ établi** : après la connexion, `localStorage.removeItem` puis `page.reload()` ; assertion `not open` **avant** le clic, qui échoue net (et nomme le cas (c)) si la route rend le groupe ouvert.
- **Clic, puis `expect.poll`** sur la valeur stockée, attendue `'true'`, avec un délai court et borné. ⚠️ **Le poll n'est honnête que si la persistance est réelle** : M1 le prouve.
- **Reload → reste ouvert** (existant).
- **Sens inverse, nouveau** : clic pour replier → poll `'false'` → reload → **reste fermé**. C'est ce que `handleToggle` promet (`:256-257`, « on respecte son choix »), et ce que le test actuel ne regarde pas.
- Les jumeaux : `:44` et `:58` (clic sur le groupe) reposent sur la même hypothèse de départ : même garde `not open` à `:58`. `:101` (auto-expand) reste tel quel. `resetGroups` (`:234`) passe déjà par un retrait et un `reload` : réemployer un helper.

## Acceptance Criteria

1. **AC-1** : le verdict (a), (b), (c) ou « non reproduit » est écrit au Change Log, sortie citée, **avant** toute modification du test.
2. **AC-2** : la spec passe **vingt fois de suite**, seule, et **deux fois** dans la suite complète, sur base neuve (chiffres déclarés, et ce que cela n'établit pas : `CLAUDE.md`, « zéro flake sur trois runs ne prouve pas… »).
3. **AC-3 — pas d'affaiblissement** : M1 à M4 rougissent toutes. Aucun `test.skip`, `retry`, `.fixme` ; aucun délai fixe `waitForTimeout`.
4. **AC-4** : sens inverse couvert (`'false'`, reload, fermé).
5. **AC-5** : `docs/testing.md` : lignes `:372`, `:402`, `:416-429` ; la formule et « 7 à 10 » deviennent **7 à 9** s'il n'y a plus de KF-052, avec une note datée qui garde l'histoire ; `CLAUDE.md:192-195` suit ; le numéro `:75` passe au numéro réel. **Les relevés datés (`:448-451`) ne se réécrivent pas.** Contrôle à déclarer, restreint pour ne pas noyer le résultat : `/usr/bin/grep -nE '7 à 10|sidebar-navigation[^ ]*:75' docs/testing.md CLAUDE.md` et `git grep -n 'sidebar-navigation' -- frontend/ ':!frontend/tests/e2e/sidebar-navigation.spec.ts'`.
6. **AC-6** : si le verdict est (b) : `+layout.svelte` corrigé, Vitest ou E2E qui rougit avant, manuel relu.

## Tâches

- [ ] **T1** : étape 1, à l'identique. **T2** : réécriture du test, helper. **T3** : mutations. **T4** : tirages de l'AC-2. **T5** : `docs/testing.md`, `CLAUDE.md`, CHANGELOG (seulement si le correctif touche la production ; sinon rien à annoncer à l'utilisateur). **T6** : **dire à l'orchestrateur** que la mémoire `e2e-echecs-attendus.md` est périmée (8 à 11, KF-045), hors dépôt.

### Mutations nommées

- **M1** : `persistGroupState` vidé (`+layout.svelte:218-222`) → rouge (preuve que le poll n'attend pas en vain).
- **M2** : `initialOpen` ignorant le stockage (`:224-227`) → rouge au reload.
- **M3** : `persistGroupState(key, true)` en dur → rouge au sens inverse.
- **M4** : valeur de défaut `administration: true` (`:245`) → rouge à la garde `not open`.

Rebuild du frontend à chaque mutation (`npm run build`, sous verrous), restauration `git checkout --` du seul fichier muté, `git status` vide vérifié ; ces mutations touchent la production **le temps du test**, jamais commitées.

## Gates

E2E complet au dernier commit de code, chaque rouge jugé contre `docs/testing.md`. Si seul le test change : Vitest et backend inchangés, pas de `test-fast.sh` requis, à déclarer. Le code de production touché (verdict b) ramène le gate complet.

## Manuels

Aucun si le verdict est (a) ou (c) : pas de comportement visible. Verdict (b) : `user-manual` à relire sur le menu, PDF régénéré.

## Recoupements

`frontend/tests/e2e/sidebar-navigation.spec.ts` et `docs/testing.md` : aucune autre fiche de l'epic ne les écrit, à la connaissance de cette fiche ; 33-10 touche `docs/testing.md`, **autre section** (union). `CLAUDE.md` : 33-10 aussi, autre paragraphe.

## Change Log

- 2026-11-06 : fiche créée (Opus 5.5).
- 2026-11-06 : validation P1 (Sonnet 5.5), remédiation (Opus 5.5). Aucun finding au-dessus de LOW. **L-R9-1** : verdict (c) récrit, écarté comme explication du symptôme, gardé comme sonde (lus : `spec:81-93`, `+layout.svelte:268-272,391-393`, `+layout.ts:5`, le corps de #424 et de #287) ; le rapport le jugeait au code, sans exécution, et cette fiche non plus n'a rien exécuté : **le protocole reproduit d'abord**. Deux profils dits. **L-F9-1** : grep restreint, `:372` ajouté, `CLAUDE.md:192-195` relu (la fiche disait `:191`). Aucun choix ajouté.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
