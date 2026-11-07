# Story 33-9 : le test E2E `sidebar-navigation` cesse de lancer de fausses alertes (#424, #287)

Status: done

*Fiche rédigée le 2026-11-06 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`, branche `story/33-0-kickoff`,
numéros de ligne de `4ef19901`. Release **0.17.5**, Epic 33. **Aucune migration, a priori aucun code de production.** Registre :
`e33-choix-autonomes-e334.md`, C-e334-19, 20. Commits en `refs` ; à la PR : `closes #424`. ⚠️ **#287 est déjà fermée**, reportée dans #424 à
l'audit du 2026-11-06 : aucun mot-clé sur elle ; ses mesures, elles, valent.*

## Story

En tant que développeur qui lit un rouge d'E2E, je veux que `persistence localStorage : Administration ouvert puis reload reste ouvert` ne rougisse que si la persistance
est cassée, pour qu'on n'ait plus à rejouer ce test pour savoir si c'est la pollution, la KF-052, ou un défaut.

## Ce que le code dit (prémisses établies)

- **Le test est à `frontend/tests/e2e/sidebar-navigation.spec.ts:81`** (non `:75`, ni `:71` : `docs/testing.md:402,421,448` citent `:75`).
- ⚠️ **Prémisse à réfuter à l'exécution — « la route d'atterrissage ».** `docs/testing.md:402,416` et #424 attribuent la KF-052 à une route d'atterrissage dans le groupe. Mais `loginAsAdmin` **exige `toHaveURL('/')`** (`:36`), et `/` appartient à *Quotidien* (`nav-link-home`, `:132`), non à Administration. L'assertion existait à l'époque du symptôme (présente à la création du fichier, `e7f2a5c2`, 2026-06-01 ; #424 date du 2026-08-30). Le groupe ne peut donc pas s'auto-ouvrir à cet endroit… **si** `/` est bien la route rendue au moment du clic. Le raisonnement ne suffit pas : **une hypothèse éliminée par raisonnement n'est pas une hypothèse testée** (`CLAUDE.md`).
- **L'hypothèse restante, celle de #287** (commentaire du 2026-08-24) : `<details ontoggle=…>` (`(app)/+layout.svelte:391-393`), `handleToggle` écrit le stockage (`:268-272`). L'attribut `open` se pose à l'activation ; **`toggle` est mis en file**, et l'écriture suit. Le test constate `open`, puis lit `localStorage` **une seule fois** (`spec:91-93`). Une course, dont l'issue dépend de la charge : le profil « déterministe ici, intermittent ailleurs » que #287 décrit. L'application est un SPA (`frontend/src/routes/+layout.ts:5`, `(app)/+layout.ts:7`, `ssr = false`) : pas de course d'hydratation à ajouter.
- **Le dernier relevé** : `rel0174/pr-corps.md:43` : rouge dans la suite, vert rejoué seul. `e2e-rejeu-sidebar.log` est au dossier. **Ce profil-là n'est pas celui de « déterministe et échoue seul »** de `docs/testing.md:423` : `docs/testing.md:421-425` en consigne deux, et dix tirages « spec seule » peuvent n'en montrer qu'un (d'où les deux protocoles de l'étape 1, et le profil écrit avec le verdict).
- **Le bon sujet, peut-être, n'est pas le test seul** : `expandedState` s'initialise à la création du composant (`:242-246`), `$effect` déplie selon le chemin (`:258-266`). Le toggle `false` initial d'un `<details>` **re-rendu** pourrait aussi écrire : à observer, ne pas supposer.
- **Le contexte navigateur est neuf à chaque test** (fixture `page`) : la pollution ne vient pas de `localStorage` mais de la **base** (`seedTestState` en `beforeAll`, `:23`, `workers: 1`) ou de la charge. Le test `:34`, lui aussi, a déjà rougi (#287).
- ⚠️ **`docs/testing.md:421-429`** bâtit le compte attendu sur la KF-052 : `7 + (1 si KF-052) + 0 à 2` = **7 à 10**. `CLAUDE.md:192-195` le répète. Le compte devient **7 à 9** si elle disparaît, et `sidebar-navigation` sort des listes de pollution **actuelles**, mais pas des relevés datés (qui restent : **on ne réécrit pas un relevé passé**).

## Étape 1 — reproduire d'abord, aucune cause supposée (C-e334-19)

Sous le double `flock`, base `kesh_e2e_e339` neuve, montage de `docs/testing.md`, sans toucher au test :

1. **Spec seule, dix tirages** ; puis **suite complète, un tirage** si le premier protocole ne rougit jamais ; sorties gardées dans `kesh-gate-logs/e339/`.
2. **Copie instrumentée, hors dépôt ou jetée** (`sidebar-probe.spec.ts`, non commitée) qui journalise : l'URL après connexion ; `open` **avant** le clic ; `localStorage` juste après `open`, puis à +50 ms, +500 ms, +5 s (`expect.poll`) ; tout événement `toggle` (`page.addInitScript`, écouteur capturant) avec son état ; et au T0, `page.url()` et `open` au moment du clic. Puis, à part : `'false'` posé au stockage, `goto('/accounts')` (groupe Administration), lecture du stockage après l'auto-dépliage.
3. **Trois verdicts possibles, à écrire au Change Log avec la sortie** :
   - **(a) écriture tardive** (`null` puis `'true'`) : le test lit trop tôt. Correctif au test.
   - **(b) jamais écrite** ou **écrite puis défaite** : **défaut de production** : un groupe qu'on replie se rouvre. On corrige `+layout.svelte`, et on signale la portée à l'orchestrateur, ce qui change la nature de la story (visible, manuel à relire).
   - **(c) groupe déjà ouvert avant le clic.** ⚠️ **Écarté d'avance comme explication du symptôme relevé, gardé comme sonde.** Le symptôme de #424 et de #287 est : `toHaveAttribute('open', '')` **passe**, puis `Expected "true", Received null`. Or un clic sur un groupe ouvert le **ferme** (action par défaut de `<summary>`) : l'assertion `open` échouerait alors, avant toute lecture du stockage, et le `toggle` écrirait `'false'`, non `null`. La « mécanique probable » de #424 (« l'attribut `open` reste vrai ») ne tient donc pas au code lu ; **ceci reste un raisonnement, et la sonde le juge** : si elle voit le groupe ouvert avant le clic, c'est un autre défaut que le symptôme connu, avec son propre message, et on identifie la route et l'état de base qui y mènent. **Mais un groupe ouvert suivi d'un clic sans effet** (re-rendu entre l'appui et le relâchement, redirection après `toHaveURL('/')`) donnerait ce même symptôme : `open` passe, aucun `toggle`, `null`. (c) se juge donc sur **trois observations** : `open` avant le clic, `toggle` reçus, stockage ; on identifie alors la route et l'état de base qui y mènent. La garde `not open` de la conception vaut dans tous les cas. Restent aussi suspects de ce symptôme (a) et (b).
4. **À part du verdict — l'auto-dépliage écrit-il ?** `handleToggle` persiste tout `toggle` (`+layout.svelte:268-272`), et le HTML en émet aussi pour un `open` posé par programme : l'auto-dépliage (`:258-266`) écrirait `'true'`, contre son commentaire (`:229-231`), et écraserait un `'false'` choisi. Hypothèse lue, non exécutée. Si la sonde le confirme : défaut de production, court, corrigé ici par l'AC-6 ; sinon, le dire.
5. Une fois la cause nommée : **si elle n'est pas reproductible** en dix tirages et un tirage de suite, le dire, garder le durcissement ci-dessous, et **ne pas écrire « cause établie »** : la mutation M1 reste la preuve de pouvoir, le verdict reste « course compatible, non observée ».

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
3. **AC-3 — pas d'affaiblissement** : M1 à M4 rougissent toutes. Aucun `test.skip`, `retry`, `.fixme` ; aucun délai fixe `waitForTimeout` dans les tests touchés (`sidebar-navigation.spec.ts:281`, dans le helper `reachable` de la #851, est antérieur et hors du périmètre ; précisé à la remédiation de la P1).
4. **AC-4** : sens inverse couvert (`'false'`, reload, fermé).
5. **AC-5** : `docs/testing.md` : lignes `:372`, `:402`, `:416-429` ; la formule et « 7 à 10 » deviennent **7 à 9** s'il n'y a plus de KF-052, avec une note datée qui garde l'histoire ; `CLAUDE.md:192-195` suit ; le numéro `:75` passe au numéro réel. **Les relevés datés (`:448-451`) ne se réécrivent pas.** Contrôle à déclarer, restreint pour ne pas noyer le résultat : `/usr/bin/grep -nE '7 à 10|sidebar-navigation[^ ]*:75' docs/testing.md CLAUDE.md` et `git grep -n 'sidebar-navigation' -- frontend/ ':!frontend/tests/e2e/sidebar-navigation.spec.ts'`.
6. **AC-6** : si le verdict est (b), ou si l'auto-dépliage écrit au stockage : `+layout.svelte` corrigé, Vitest ou E2E qui rougit avant, CHANGELOG, manuel relu.

## Tâches

- [x] **T1** : étape 1, à l'identique. **T2** : réécriture du test, helper ; Vitest sur `frontend/src/lib/shared/e2e-selecteurs-traduits.test.ts`, qui lit cette spec : si le sélecteur « Administration » disparaît, retirer sa ligne de `DETTE_CONNUE` (`:176`), le test rougissant sur une ligne morte (`:204-214`), et le dire au Change Log. **T3** : mutations. **T4** : tirages de l'AC-2. **T5** : `docs/testing.md`, `CLAUDE.md`, CHANGELOG (seulement si le correctif touche la production ; sinon rien à annoncer à l'utilisateur). **T6** : **dire à l'orchestrateur** que la mémoire `e2e-echecs-attendus.md` est périmée (8 à 11, KF-045), hors dépôt.

### Mutations nommées

- **M1** : `persistGroupState` vidé (`+layout.svelte:218-222`) → rouge (preuve que le poll n'attend pas en vain).
- **M2** : `initialOpen` ignorant le stockage (`:224-227`) → rouge au reload.
- **M3** : `persistGroupState(key, true)` en dur → rouge au sens inverse.
- **M4** : valeur de défaut `administration: true` (`:245`) → rouge à la garde `not open`.

Rebuild du frontend à chaque mutation (`npm run build`, sous verrous), restauration `git checkout --` du seul fichier muté, `git status` vide vérifié ; ces mutations touchent la production **le temps du test**, jamais commitées.

## Gates

E2E complet au dernier commit de code, chaque rouge jugé contre `docs/testing.md`. Si seul le test change : Vitest ciblé sur la garde de T2, backend inchangé, pas de `test-fast.sh` requis, à déclarer. Le code de production touché (AC-6) ramène le gate complet.

## Manuels

Aucun sans correctif de production : pas de comportement visible. Avec l'AC-6 : `user-manual` à relire sur le menu, PDF régénéré.

## Recoupements

`frontend/tests/e2e/sidebar-navigation.spec.ts` et `docs/testing.md` : aucune autre fiche de l'epic ne les écrit, à la connaissance de cette fiche ; 33-10 touche `docs/testing.md`, **autre section** (union). `CLAUDE.md` : 33-10 aussi, autre paragraphe.

## Change Log

- 2026-11-06 : fiche créée (Opus 5.5).
- 2026-11-06 : validation P1 (Sonnet 5.5), remédiation (Opus 5.5). Aucun finding au-dessus de LOW. **L-R9-1** : verdict (c) récrit, écarté comme explication du symptôme, gardé comme sonde (lus : `spec:81-93`, `+layout.svelte:268-272,391-393`, `+layout.ts:5`, le corps de #424 et de #287) ; le rapport le jugeait au code, sans exécution, et cette fiche non plus n'a rien exécuté : **le protocole reproduit d'abord**. Deux profils dits. **L-F9-1** : grep restreint, `:372` ajouté, `CLAUDE.md:192-195` relu (la fiche disait `:191`). Aucun choix ajouté.
- 2026-11-06 : validation P2 (Opus 5.5), remédiation (Opus 5.5). Trois LOW, aucun réfuté. **L-R9-1** : (c) jugé sur trois observations, « autre défaut » retiré. **L-R9-2** : sonde de l'auto-dépliage, AC-6 élargi (lus : `+layout.svelte:229-231,258-272`, `:125` pour `/accounts`). **L-F9-3** : mention de `CLAUDE.md` retirée (`/usr/bin/grep -n sidebar CLAUDE.md` vide). La question `git log -L` est levée (lu : `git show e7f2a5c2:…spec.ts`, ligne 30). `CLAUDE.md:191` corrigé en `:192`, résidu de la P1 (`/usr/bin/grep -n '7 à 10' CLAUDE.md` : `:192,195`). Aucune exécution : le protocole reproduit d'abord.
- 2026-11-06 : validation P3 (Sonnet 5.5, lentilles R et F), remédiation (Opus 5.5). Un LOW, non réfuté. **L-R9-1** : T2 et *Gates* (lus : `e2e-selecteurs-traduits.test.ts:176,204-214`). Mutations : 4, inchangées.
- 2026-11-07 : **étape 1, avant toute modification du test** (Opus 5.5 ; AC-1). Étape 0 : L-R9-1 de `e33-validate-p3-e334.md` déjà porté par T2 et *Gates* ; `e33-validate-p4.md` confirme `e2e-selecteurs-traduits.test.ts:176,204-214`. **Tirages** : spec seule, `--repeat-each=10`, base neuve : 100 passés, aucun rouge (`e339/e2e-rspec/playwright.log`) ; **suite complète, deux tirages sur le même frontend** (`e337/e2e-r1`, `e337/e2e-r2`) : vert puis **rouge** *(« vert » : sans autre échec que les sept attendus ; précisé le 2026-11-07, revue P2, L-P2-1)*, symptôme de #424 à l'identique, `Expected: "true"`, `Received: null` (`e339/suite-e337-r2-extrait.log`). **Sonde 1** (`e339/e2e-rsonde/playwright.log`, cinq tirages) : URL `/` après connexion ; `open` faux et stockage `null` **avant** le clic ; un seul `toggle`, `open: true`, vu en capture avec le stockage encore `null`. **Sonde 2** (`e339/e2e-rsonde2/playwright.log`) : dans la page, `["synchrone-apres-clic",true,null]`, `["microtache",true,null]`, `["tache-1",true,"true"]` ; **et le chemin même du test, sans ralentir : `premier=null final="true" ms=6`, puis `ms=4`, deux tirages sur cinq.** **Verdict : (a), écriture tardive, reproduite.** `open` se pose au clic, `toggle` est une tâche, l'écriture la suit de quelques millisecondes ; le test lit une seule fois entre les deux. La persistance fonctionne. (b) et (c) écartés sur observation : l'écriture a lieu, et le groupe est fermé avant le clic. La « route d'atterrissage » de #424 et de `docs/testing.md` est **réfutée à l'exécution** : la route est `/` à chaque tirage. ⚠️ Ce qui n'est pas expliqué : ralentir le processeur par CDP (×6, ×20, ×50) n'a produit **aucun** `null` sur quinze tirages : c'est l'ordonnancement entre le pilote et la page qui compte, non la seule lenteur de la page, et ces tirages ne montrent pas pourquoi la spec seule, cent fois, ne rougit pas. **Étape 4, l'auto-dépliage : il écrit** (`auto-stockage-apres "true"`, `auto-toggles [{"open":true,"stored":"false"}]`, `auto-retour-open true`) : `'false'` posé, `goto('/accounts')`, le stockage passe à `'true'` et le groupe reste ouvert au retour sur `/`, contre le commentaire de `+layout.svelte:229-231`. **Défaut de production, signalé à l'orchestrateur : la story change de nature (AC-6).**
- 2026-11-07 : remédiation de la revue P1 (Opus 5.5), sans code. Lentilles, B : 0/0/1/3, E : 0/0/0/4, A : 0/0/1/3. **M-A1 = M-B1, E-LOW-1 confirmés** (lus : `+layout.svelte:242,258-266`, rien ne remet l'état au choix enregistré ; le montage écrivait `'true'`) : CHANGELOG reformulé, **comportement inchangé**, sur décision de l'orchestrateur : au prochain chargement ; un état déjà enregistré se corrige d'un clic. **L-A1 = L-B1** : `testing.md:415`, `:90`. **L-A2 = L-B2, L-A3** : « l'un des deux tirages relevés », réserve recopiée à `testing.md`. **E-LOW-2** : AC-3 précisé pour `:281`, lu antérieur (`bdc3faa4:…:219`). **Écrits ici, sans correctif** : E-LOW-3, couverture : le repli à la main sur une page du groupe suivi d'une navigation, et le non-écrit au montage pour *Quotidien* et *Mensuel*, ne sont pas joués en E2E. L-B3 = E-LOW-4, course théorique de quelques millisecondes entre un clic et l'auto-dépliage du même groupe : un clic perdu, sans effet durable. **Issues demandées** : la remise du groupe au choix enregistré en navigation interne (comportement, à l'arbitrage de Guy), et ces deux couvertures. Aucun finding réfuté. Aucune ligne de code ni de test : la revue P2 ne porte que sur les textes ; frontend et spec restent ceux de `d96707e2`, gates non rejoués. Prompt : `33-9-e2e-sidebar-review-prompt-p2.md`.
- 2026-11-07 : **revue P2 ciblée** (Sonnet 5.5, rapport `kesh-gate-logs/e339-review-p2.md`) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 2 LOW, aucun réfuté. **L-P2-1** : « deux suites vertes » devient « sans autre échec que les sept attendus » à `docs/testing.md:415`, et l'entrée de l'étape 1 de ce Change Log est annotée, non réécrite. **L-P2-2** : `docs/testing.md:405`, « `:75` à l'époque, `:90` aujourd'hui ». Le comportement en navigation interne et les deux trous de couverture sont réunis en **#959**. Documentation seule, gate non rejoué.
- 2026-11-07 : **revue de code close, story `done`.** Tendance, recomptée sur les rapports, en CRITICAL/HIGH/MEDIUM/LOW : P1, lentille B 0/0/1/3, E 0/0/0/4, A 0/0/1/3 (deux MEDIUM, un seul défaut : M-A1 = M-B1) ; P2 ciblée 0/0/0/2. Modèles : P1, lentilles B, E et A en Sonnet 5.5 ; P2 en Sonnet 5.5 ; développement et remédiations en Opus 5.5. Aucun reclassement en dette. La boucle se clôt sur une passe sans finding au-dessus de LOW, dont la remédiation ne touche aucune ligne de code. Code au dernier commit de code, `f5c8daec` (spec et `+layout.svelte`), gates et E2E complet à `d96707e2` (Dev Agent Record). Issue de suite : #959.

## Dev Agent Record

### Agent Model Used

Opus 5.5 (`claude-opus-5-5`).

### Completion Notes List

- **Verdict** : (a), écriture tardive, reproduite (Change Log) ; l'auto-dépliage écrivait au stockage : **défaut de production**, corrigé (AC-6). La story touche donc la production, ce que la fiche prévoyait : gate complet, CHANGELOG, manuel relu.
- **Correctif** : `handleToggle` n'écrit plus quand l'état vaut déjà `open`, c'est-à-dire quand Kesh a posé `open` lui-même, à l'auto-dépliage et au montage. Effet de bord voulu : le montage n'écrit plus `'true'` pour *Quotidien* et *Mensuel*.
- **Rouges** (`e339/e2e-rrouge/playwright.log`, `e339/rouges.log`), spec réécrite, production d'origine : 10 passés, 1 échec, le test neuf, `Expected "false"`, `Received "true"`.
- **Mutations** (`e339/mut/resume.txt`), témoin M0 vert, M1 à M4 de la fiche et **M5** (garde de `handleToggle` retirée) toutes tuées, `git status` vide après chacune, frontend reconstruit à chacune et après la dernière.
- **AC-2, `d96707e2`** : spec seule, `--repeat-each=20` : **220 passés, aucun rouge** (11 tests × 20) ; suite complète, deux tirages sur base neuve : **353 passés, 18 ignorés, 7 échecs**, les sept de la KF-029, deux fois, aucun `sidebar-navigation`. ⚠️ **Ce que cela n'établit pas** : avant le correctif, la spec seule ne rougissait pas non plus (cent passages) et la suite une fois sur deux : deux suites vertes sont compatibles avec un défaut de même fréquence, et leur probabilité par le seul hasard n'est pas négligeable. La preuve qui compte est de forme : le test n'a plus de lecture unique, et M1 montre que son attente n'est pas vide.
- **Gates, `d96707e2`** : `test-fast.sh --ci` vert, **nextest 4813 exécutés, 4813 passés, 12 ignorés** ; `cargo build --workspace --all-targets` vert ; frontend : `check`, `lint-i18n-ownership`, `test:unit` **2696 tests, 218 fichiers**, `build` verts (dont `e2e-selecteurs-traduits.test.ts`, inchangée : le site « Administration » de `DETTE_CONNUE` existe toujours, la spec gardant son commentaire `has-text("Administration")`). Journal : `e339/gates-head.log`.
- **AC-5, contrôles** : `/usr/bin/grep -nE '7 à 10|sidebar-navigation[^ ]*:75' docs/testing.md CLAUDE.md` ne rend que les annotations historiques voulues et le relevé daté (`testing.md:405,422,429,448`, `CLAUDE.md:195`) ; `git grep -n 'sidebar-navigation' -- frontend/ ':!frontend/tests/e2e/sidebar-navigation.spec.ts'` : `e2e-selecteurs-traduits.test.ts:176`, seul.
- **Manuel** : `user-manual.tex:132-137` décrit les groupes dépliables, et rien de leur mémorisation ; `/usr/bin/grep -niE 'replier|replié|déplie'` n'en dit rien d'autre : relu, rien à changer, PDF non régénéré.
- **Captures** (thème clair, `e339/captures/`) : `/accounts`, groupe déplié pour la page ; `/`, groupe replié comme choisi. Aucun contrôle neuf.
- ⚠️ **#424 disait aussi « déterministe, échoue seul »** (2026-08-30) : ce profil n'a pas été retrouvé en 100 + 220 passages, et sa cause d'alors reste non établie. La mémoire `e2e-echecs-attendus.md`, hors dépôt, est périmée (T6) : à dire à l'orchestrateur.

### File List

`frontend/src/routes/(app)/+layout.svelte`, `frontend/tests/e2e/sidebar-navigation.spec.ts`, `docs/testing.md`, `CLAUDE.md`, `CHANGELOG.md`, `_bmad-output/implementation-artifacts/{33-9-e2e-sidebar.md,33-9-e2e-sidebar-review-prompt-p1.md,sprint-status.yaml}`.
