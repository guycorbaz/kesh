# Story 33-11 : la CI saute ses jobs lourds quand une PR ne touche que `_bmad-output/`

Status: done

*Epic 33 (v0.17.5). Demande de Guy du 2026-11-06 : « oui, ajoute un filtre ci pour éviter de perdre du temps inutile ». Clé de choix `C-e3311-k`, registre `e33-choix-autonomes-e3311.md`. Aucune migration, aucun code de production. Commits en `refs #928` ; PR d'intégration : `refs #928` ; **`closes #928` à la PR de la rétrospective**, qui porte la dernière preuve (AC-6 c).*

## Story

As a Project Lead,
I want qu'une PR qui ne porte que des artefacts de travail n'attende pas la suite complète,
so that un cycle de CI ne coûte plus 37 minutes pour de la documentation interne (PR #927, job backend, 2026-11-06).

## Faits établis

- F1. `.github/workflows/ci.yml:3-7` : déclencheurs `push` sur `main` et `pull_request` vers `main`, sans filtre. Quatre jobs : `backend` (`:21`), `frontend` (`:208`), `docker-build` (`:252`), `e2e-smoke` (`:334`, `needs: docker-build`, `:337`).
- F2. Protection de `main` : ces quatre noms de checks sont requis (`gh api repos/guycorbaz/kesh/branches/main/protection/required_status_checks`). **Un `paths-ignore` au niveau du workflow bloquerait donc toute PR filtrée**, ses checks restant en attente. **`changes` ne devient pas requis.**
- F3. **Documentation GitHub, lue le 2026-11-06, non exécutée** (d'où l'AC-6). *Troubleshooting required status checks* : « Successful check statuses are `success`, `skipped`, and `neutral` » ; un job sauté par sa condition « reports 'Success' » ; un job dont la dépendance a échoué : « The dependent job is skipped and may not block merging », remède : « Use `always()` with `needs` for required checks that depend on other jobs ». *Expressions* : « A default status check of `success()` is applied unless you include one of these functions » ; `always` s'exécute « even when canceled ». *Contexts* : « If you attempt to dereference a nonexistent property, it will evaluate to an empty string » : une sortie jamais écrite vaut donc `''`. *Workflow cancellation* : à l'annulation, « the server re-evaluates `if` conditions for all currently running jobs » ; « If the condition evaluates to `true`, the job will not get canceled », exemple `always()` ; rien sur un job non démarré, d'où l'AC-6 (d).
- F4. `git grep -n '_bmad-output' -- crates frontend/src frontend/tests scripts .github` rend trois mentions, toutes des commentaires (`kesh_version.sql:4`, `version.rs:4`, `i18n-dette-connue.ts:23`) : aucun test ne lit ce dossier. À refaire au développement.
- F5. Les tests lisent d'autres textes : `crates/kesh-api/tests/textes_coherents.rs`, `mcp_matrice_manuel.rs`, `configuration_transmise.rs` lisent manuels, CHANGELOG, `.env.example`. **Manuels, `docs/`, `README.md`, `CHANGELOG.md`, `CLAUDE.md`, `website/` restent donc hors du filtre.** La PR #423, citée au `CLAUDE.md`, n'aurait pas été entièrement filtrée ; c'est voulu.
- F6. `CLAUDE.md:293` et `docs/ci.md:12` affirment l'absence de filtre : ils deviennent faux.
- F7. **Sonde sur un dépôt jetable, 2026-11-06.** `git diff --name-only --no-renames -z A...B` rend les deux noms d'un renommage, et les noms bruts (espace, accent), sans guillemets ; sans `--no-renames`, le nom d'arrivée seul. Un SHA inconnu : code 128, rien sur la sortie. Git refuse d'indexer un chemin à `..` (`Invalid path`).

## Conception

Toute la décision vit dans le script, le YAML n'est que de la plomberie.

- **`scripts/ci-seuls-artefacts.sh <événement> <base> <tête>`**, `LC_ALL=C`, écrit **une seule ligne** sur sa sortie, en dernier : `code=true` ou `code=false`. Il lit lui-même `git diff --name-only --no-renames -z <base>...<tête>`, sortie en fichier temporaire, code de retour contrôlé avant toute décision. Motif sur la sortie d'erreur.
- **Pas d'API, donc ni pagination, ni plafond, ni droit `pull-requests: read`.** Les trois points comparent à la base de fusion : ce que la PR apporte, même après une fusion de `main` dans sa branche.
- **Un nom réel, entièrement sous `_bmad-output/`, rend `false`, quels que soient ses caractères.** `-z` supprime les noms cités ; un nom lu autrement, ou ailleurs, rend `true`.
- **Job `changes`**, `timeout-minutes: 10`, trois étapes, dans cet ordre : `actions/checkout` en `fetch-depth: 0` ; `scripts/tests/ci-seuls-artefacts.test.sh` ; le script, `>> "$GITHUB_OUTPUT"`, arguments passés par `env:`, soit `github.event_name`, `github.event.pull_request.base.sha`, `github.event.pull_request.head.sha`. Un test rouge arrête le job avant toute sortie.
- **Conditions par exclusion** : seule une sortie `false` d'un `changes` **réussi** saute un job ; la sûreté ne repose donc pas sur la sortie d'un job en échec, que la documentation lue ne décrit pas.

## Critères d'acceptation

1. **AC-1.** `changes`, sans dépendance et sans condition, rend `code=false` **seulement si** l'événement est `pull_request` et que le diff, non vide, ne porte que des fichiers sous `_bmad-output/`.
2. **AC-2. Le filtre échoue vers la CI complète.** Tout autre cas rend `code=true`, ou n'écrit rien : événement `push`, diff vide, arguments manquants, SHA inconnu, historiques sans base commune, `git` en échec, fichier renommé depuis ou vers un autre dossier.
3. **AC-3.** `backend`, `frontend`, `docker-build` : `needs: changes`, `if: ${{ !cancelled() && !(needs.changes.result == 'success' && needs.changes.outputs.code == 'false') }}`. `e2e-smoke` : `needs: [changes, docker-build]`, la même condition, `&& needs.docker-build.result == 'success'`. Leurs `name:` ne changent pas (F2), ni les réglages de protection.
4. **AC-4.** Le script est celui de la *Conception* ; aucune action tierce nouvelle.
5. **AC-5. Tests, `scripts/tests/ci-seuls-artefacts.test.sh`**, sur un mini-dépôt git réel sous `mktemp -d` (identité passée par `git -c`), chaque cas par de vrais commits :
   - **`false`** : `_bmad-output/a.md` seul ; nom à espace et nom accentué sous `_bmad-output/` ;
   - **`true`** : `push` avec le même diff ; `_bmad-output/a.md` + `crates/x.rs` ; `_bmad-output-bis/a.md` ; `_bmad-output` comme fichier ; `docs/manual/fr/x.tex` ; `CHANGELOG.md` ; `.github/workflows/ci.yml` ; nom accentué hors du dossier ; renommage de `crates/x.rs` vers `_bmad-output/x.rs`, et l'inverse ; diff vide ; arguments manquants ; SHA inconnu ; branche orpheline ; `git` factice (`PATH`) qui rend `_bmad-output/a.md` puis sort en 1 ;
   - **garde de `ci.yml`** : les quatre jobs portent les conditions de l'AC-3 (ou celles de l'AC-6 (d), si le relevé les impose), aucun `== 'true'`, `changes` porte `timeout-minutes: 10` et les trois expressions de la *Conception*, et l'étape des tests précède celle qui écrit `GITHUB_OUTPUT`.
6. **AC-6. Preuve sur de vraies PR, par l'orchestrateur, liens et `gh run view --json jobs` cités au Change Log.**
   - **(a)** La PR d'intégration, qui touche du code, fait tourner les quatre jobs.
   - **(b) Le chemin de l'échec, avant la fusion** : sur cette PR, un commit temporaire ajoute `exit 1` en première étape de `changes`. Attendu : `changes` en échec, et les quatre jobs **exécutés**, aucun `skipped`, `e2e-smoke` compris. Le commit se retire par `git revert`, sans réécrire la branche, et la CI repasse. **Si un job est sauté, la PR ne se fusionne pas** avec le filtre, et Guy est prévenu.
   - **(c)** Après la fusion, seul moment où `ci.yml` n'est plus dans le diff : la PR de la rétrospective, qui ne porte que `_bmad-output/`, rend les quatre checks sautés et **reste fusionnable** (`mergeStateStatus` relu). Sinon, le filtre est retiré et Guy prévenu.
   - **(d) L'annulation, avant la fusion** : un commit temporaire ajoute `sleep 180` en première étape de `changes` ; `gh run cancel` pendant ce sommeil ; `gh run view --json jobs` relève l'état des quatre jobs ; `git revert`. Aucun `skipped` : la condition reste. **Si un job est `skipped`** (donc vert pour la protection, F3), **le filtre passe à `always()`** : `if: ${{ always() && !(needs.changes.result == 'success' && needs.changes.outputs.code == 'false') }}`, `e2e-smoke` gardant son `&& needs.docker-build.result == 'success'` ; garde de l'AC-5 alignée, (a), (b), (d) rejoués. Coût à relever au Change Log et à dire à Guy : sous `always()`, un job lancé ne s'annule plus (F3), et un run remplacé par `cancel-in-progress` va à son terme.
   - **(e) Le délai dépassé, avant la fusion** *(ajouté le 2026-11-07, revue P1, E-1)* : un commit temporaire porte `timeout-minutes: 1` sur `changes` et `sleep 180` en première étape ; relevé de l'état des quatre jobs et du rollup ; `git revert`. Même sanction qu'en (d).
   - La story passe `done` sur (a), (b), (d) et (e) ; (c) garde l'issue ouverte, non la story. Conduite détaillée, observables et commandes : `33-11-filtre-ci-preuves-ac6.md`.
7. **AC-7.** `push` sur `main` : les quatre jobs tournent toujours, y compris après un commit d'artefacts seul.
8. **AC-8. Textes.** `CLAUDE.md` § *Quand sauter* et `docs/ci.md` disent le filtre, son périmètre étroit, et ce qui en reste exclu ; le passage historique reste, annoté et daté. `docs/ci.md` : schéma d'exécution et colonne *Dépendances* comprennent `changes`. `git grep` du symptôme (« aucun filtre », « Aucun filtre », `paths:`, « quatre jobs », « 4 jobs ») sur tout le dépôt, sites traités.

## Tâches

- [x] T1. Relire F1 à F7 au code, lignes citées au Change Log.
- [x] T2. Tests d'abord, rouges ; script ; verts (AC-1, AC-2, AC-4, AC-5).
- [x] T3. `ci.yml` : job `changes`, conditions (AC-3).
- [x] T4. Textes (AC-8). CHANGELOG `[0.17.5]`, *Ajouté*.
- [x] T5. `actionlint` si disponible, sinon lecture ligne à ligne, déclarée.
- [ ] T6. *(Ajoutée le 2026-11-07, revue P2, F5 ; à l'orchestrateur.)* Une fois l'AC-6 (a), (b), (d), (e) relevé, repasser du conditionnel au relevé `docs/ci.md`, `CHANGELOG.md` et `CLAUDE.md:297`, PR citée (mode opératoire).

## Mutations

Chacune tuée par un cas de l'AC-5, consignée dans `kesh-gate-logs/e3311/mut/` : **M1** condition inversée ; **M2** préfixe sans barre finale ; **M3** liste vide rendue `false` ; **M4** `--no-renames` retiré ; **M5** `-z` retiré ; **M6** événement non contrôlé ; **M7** code de retour de `git` ignoré ; **M8** `== 'true'` rétabli dans `ci.yml` ; **M9** `needs.changes.result` retiré des conditions.

## Gates

Story sans code de production : `scripts/tests/ci-seuls-artefacts.test.sh`, `shellcheck` si présent, `cargo test -p kesh-api --test textes_coherents` sous verrou si un texte qu'il lit change. Le gate complet reste à l'intégration, et la PR d'intégration exerce le workflow pour de bon.

## Dev Notes

- Une erreur dans ce filtre peut faire sauter des tests à tort : le risque est **muet**. D'où un périmètre d'un seul dossier, des conditions par exclusion, et `main` jamais filtré, qui reste le filet.
- **Limites assumées.** (1) L'annulation d'un run pendant `changes` se juge sur pièce (AC-6 (d)), la documentation étant muette sur les jobs non démarrés. (2) Le script et ses tests viennent de la PR elle-même ; une PR qui les modifie sort du périmètre **tant qu'elle ne change pas le motif** : une PR qui élargirait le `case` à `scripts/*` et ajusterait les tests sauterait sa propre CI, et seule la relecture humaine le verrait (E-9 de la revue P1). Le filtre n'est pas une frontière de confiance : une PR pouvait déjà modifier `ci.yml`, et la fusion lance toujours la CI complète sur `main`. (3) Le coût de `changes` en série avant les jobs lourds (`fetch-depth: 0`, tests du filtre) n'est pas mesuré ; il se lira sur la PR d'intégration.
- Élargir le périmètre plus tard demande de vérifier d'abord que rien ne lit les fichiers visés (F5).

## Recoupements

`docs/ci.md`, `CLAUDE.md`, `CHANGELOG.md` avec **33-9** et **33-10**, autres paragraphes, union à l'intégration. **33-10** lit `ci.yml` (ligne `cargo test`, boucle de migrations) : ce filtre n'y touche pas.

## Change Log

- 2026-11-06 — Fiche écrite par l'orchestrateur (Opus 5.5) sur demande de Guy. Prochaine : validation P1.
- 2026-11-06 — Première validation (Opus 5.5, rapport `e33-validate-p2-e334.md` : 1 HIGH, 2 MEDIUM, 2 LOW), remédiation (Opus 5.5), aucun réfuté, aucune exécution de Kesh ni de la CI. **H-R11-1** : conditions par exclusion, arbitrage de l'orchestrateur (`!cancelled()`, `changes` non requis ; attribution à Guy corrigée le 2026-11-06, il n'a décidé que le filtre lui-même) ; tests avant la sortie ; AC-6 (b) ; documentation relue (F3). **M-R11-2** : tranché, AC-2 et AC-5 alignés. **M-R11-3** : l'extraction passe dans le script, testée sur un dépôt réel ; **l'API est écartée**, sonde F7. **L-R11-4** : sans objet, sans API. **L-F11-5** : `closes #928` à la PR de la rétrospective, *Ajouté*, *Recoupements*. **Retirés** : le choix ouvert de T3, la lecture sur l'entrée standard, le cas et la mutation `..` (git refuse ce chemin, F7), et la « PR de plus de fichiers que l'API n'en rend ». Le script de test se renomme selon le patron `scripts/tests/*.test.sh`. Mutations : 8. Passe suivante requise.
- 2026-11-06 — Validation P3 (Sonnet 5.5, rapport `e33-validate-p3-e334.md` : 1 MEDIUM, 4 LOW), remédiation (Opus 5.5), aucun réfuté, aucune exécution de la CI. **M-R11-1** : relevé AC-6 (d), conduite fixée d'avance (arbitrage de l'orchestrateur) ; la limite (1) des Dev Notes ne décide plus rien. **L-R11-2** : garde `needs.changes.result == 'success'`, M9. **L-R11-3**, **L-R11-4** : *Conception*, garde de l'AC-5 (lu : `ci.yml:11,45,211,255,338`). **L-F11-5** : AC-8 (lus : `docs/ci.md:9-12,21-28`). Documentation lue ce jour (*Workflow cancellation*, *Contexts* pour la phrase de F3, que la P3 n'avait pas lue) : **un job sous `always()` ne s'annule plus**, coût que le rapport n'avait pas mesuré, écrit à l'AC-6 (d). Mutations : 9.
- 2026-11-06 — **Validation terminée** en quatre passes (P1 de cette fiche en P2 de l'epic). P4 ciblée Haiku (`e33-validate-p4.md`) : MEDIUM **réfuté** par l'orchestrateur, ligne lue : « les trois expressions de la *Conception* » désigne `github.event_name`, `base.sha`, `head.sha`, bien présentes au point « Job `changes` » ; le rapport y cherchait `needs.changes`. LOW (« documentation muette », Dev Notes limite (1)) renvoyé à l'étape 0. Ces remédiations de Change Log ne touchent aucune règle.
- 2026-11-07 — Développement (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e337`. **Étape 0** : LOW de la P4 (`e33-validate-p4.md`, Dev Notes limite (1), « documentation muette ») : corrigé, « muette sur les jobs non démarrés » (lus : `33-11-filtre-ci.md:31,83`) ; son MEDIUM reste réfuté, comme le dit l'entrée précédente. LOW de la P3 déjà portés. **T1, F1 à F7 relus** : déclencheurs et jobs à `ci.yml:3-7,21,208,252,334`, `needs: docker-build` à `:337` ; F2 non rejoué (`gh` en lecture, relevé de la P3 repris) ; F4 refait, `git grep -n '_bmad-output' -- crates frontend/src frontend/tests scripts .github` : trois mentions, trois commentaires ; F5 : `textes_coherents` lu et joué ; F6 : `CLAUDE.md:293`, `docs/ci.md:9-12` ; F7 : éprouvé par les cas du test. **Le symptôme** (`git grep -nE 'aucun filtre|Aucun filtre|quatre jobs|4 jobs|paths:'` sur `CLAUDE.md docs README.md website .github`) : `docs/ci.md:9,10,12`, `CLAUDE.md:293`, traités ; `CLAUDE.md:248` (« ses quatre jobs », la protection de `main`) reste vrai ; `deploy-pages.yml:9` (`paths:`) est un autre workflow. **Écarts à la fiche** : aucun. Le passage historique de `CLAUDE.md` reste, annoté et daté. **AC-6 : rien n'a tourné, cela revient à l'orchestrateur** (`33-11-filtre-ci-preuves-ac6.md`) : le filtre n'est donc **pas prouvé** sur GitHub.
- 2026-11-07 — **Remédiation de la revue P1** (Opus 5.5 ; rapports `e3311-review-p1-{A,B,E}.md` : A 2 MEDIUM, 5 LOW ; B 5 LOW ; E 3 MEDIUM, 6 LOW). Code par l'outil Edit seul. Prémisses lues au code avant chaque correctif.
  - **E-1** (MEDIUM, le délai dépassé de `changes`) : relevé **(e)** ajouté à l'AC-6 et au mode opératoire, `timeout-minutes: 1` et `sleep 180`, même sanction qu'en (d). Rien n'a tourné.
  - **MEDIUM-1 = E-7** (`code=` non observable) : prémisse confirmée, `ci.yml:45` envoie la sortie standard dans `$GITHUB_OUTPUT`. L'observable devient le motif au journal (`gh run view <run> --log --job <id>`, filtré sur `ci-seuls-artefacts :`) et l'état des quatre jobs (`gh run view --json jobs`), commandes écrites.
  - **MEDIUM-2 = E-2** (`CLEAN` sous `strict`) : protection relue sans écrire (`gh api …/branches/main/protection` : `strict: true`, les quatre contextes requis). (c) juge les quatre contextes en `SKIPPED` ; `UNKNOWN` réinterrogé ; `BEHIND` traité par *Update branch*, motif relevé de nouveau après la mise à jour ; seul un blocage dû aux checks retire le filtre.
  - **E-3 = A-L-1** (trois points non distingués des deux) : cas « main avancée après la branche », où `base..tête` nomme le fichier de `main` et `base...tête` non. **M10** (`...` → `..`) jouée par Edit : tuée (`e3311/p2/mut/M10-deux-points.log`), restaurée.
  - **E-4** (la boucle retombe sur `false` si la lecture échoue) : `exec 3<` sous contrôle, noms comptés, comparés aux NUL du fichier ; `false` exige au moins un nom et autant de NUL que de noms. **Rouge** avant le correctif, 1 assertion (`e3311/p2/rouges.log`, « sortie de git sans NUL final ») ; **M11** (contrôle des NUL retiré) tuée (`e3311/p2/mut/M11-nul.log`).
  - **E-5** : cas ajoutés, fichier supprimé seul, nom à retour à la ligne, dernier nom sans NUL. **E-6 = B-L1** : garde ancrée sur la ligne entière (`contient_ligne`, `grep -qxF`), `id: decision` et `steps.decision.outputs.code` exacts, `continue-on-error` refusé ; un cinquième job n'est pas contrôlé (sens sûr, voir issues). **B-L2** : l'ouverture se lit sur la ligne du job `backend` et s'exige aux quatre. **B-L5** : l'insertion dans le passage historique de `CLAUDE.md:293` retirée, l'annotation datée suffit.
  - **A-L-2** : `changes` au tableau des timeouts de `docs/ci.md`. **A-L-3 = E-8** : `docs/ci.md:12` et `CLAUDE.md:297` au conditionnel jusqu'aux relevés, « conditions des quatre jobs » ; « sur chaque PR » (`docs/ci.md:60,82`) précisé. **A-L-5** : (b) et (d) calés sur l'état des jobs ; « ne pas fusionner tant que le revert n'est pas poussé et vert ». **A-L-4 = B-L4 = E-9** : limite (2) précisée. **B-L3** : coût de `changes` non mesuré, limite (3).
  - Réfuté : aucun. Gates : voir *Completion Notes*. Prochaine : passe P2 ciblée (`33-11-filtre-ci-review-prompt-p2.md`).
- 2026-11-07 — **Revue P2 ciblée** (Sonnet 5.5, rapport `kesh-gate-logs/e3311-review-p2.md`) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 6 LOW (F1 à F6), aucun réfuté. Soldés par l'outil Edit, sans toucher le script ni `ci.yml` ; seul le test change.
  - **F1** : la garde lit en ligne entière `EVENEMENT:`, `BASE:`, `TETE:` et les deux lignes `run:` (test, puis script vers `$GITHUB_OUTPUT`) ; l'ordre se lit sur ces deux lignes exactes. Mutations sur des copies de `ci.yml` (`CI_YML_POUR_TEST`, `e3311/p3/mut/`) : `BASE`/`TETE` permutés, `run: echo code=false`, test commenté : **trois tuées**, deux assertions chacune. Le test d'avant n'était pas rejoué contre elles : qu'il les laissait passer se lit à ses sous-chaînes (rapport, F1).
  - **F2** : `grep -m1 -nxF … || l_… =` ; une ligne absente rend un échec nommé (« ligne ? », vu aux mutations ci-dessus), et non plus un arrêt muet. `contient`, devenu sans usage, est retiré.
  - **F3** : « fichier supprimé » porte un artefact, pour que seule la suppression le rende hors du périmètre ; le nom à retour à la ligne est **sous** le dossier, `false` attendu. Mutations du script, jouées par Edit puis `git checkout --` et `touch` : **M12** (`|| [ -n "$nom" ]` retiré) tuée par « sans NUL final » et « dernier nom sans NUL » ; **M13** (`--diff-filter=d`) tuée par « supprimé, avec un artefact » et le renommage ; **M14** (lecture par lignes) tuée par sept cas, dont le nouveau. Journaux `e3311/p3/mut/M12…M14`.
  - **F4** : `docs/ci.md:9` et la ligne `changes` du tableau au conditionnel, « cas à relever » ; `CHANGELOG.md`, réserve « à confirmer avant la release » ; `CLAUDE.md:297`, le script est lancé par `changes`, dont les conditions lisent la sortie. **F5** : étape « textes au relevé » au mode opératoire, T6 à la fiche. **F6** : (c) se relève après la fin de `Filtre des artefacts`.
  - Issue de suite : **#961**. Test shell vert, 54 assertions (`e3311/p3/test-shell.log`). **Boucle de revue close** sur une passe sans finding au-dessus de LOW ; tendance P1 (A 0/0/2/5, B 0/0/0/5, E 0/0/3/6) → P2 0/0/0/6. **La fiche reste en `review`** : elle passe `done` sur les relevés de l'AC-6 (a), (b), (d), (e), par l'orchestrateur.
- 2026-11-07 : **relevés de l'AC-6 (a), (b), (d), (e)**, par l'orchestrateur, sur la PR de release [#965](https://github.com/guycorbaz/kesh/pull/965) (mode opératoire `33-11-filtre-ci-preuves-ac6.md`) — tous conformes, aucun job requis `skipped`, la condition reste (pas de passage à `always()`) :
  - (a) run 42398838476 (PR #965, tête 9a78413a), 2026-11-07 19:35 : Filtre des artefacts success, motif « hors du périmètre : .github/workflows/ci.yml » ; Backend, Frontend, Docker build, E2E smoke : success ; mergeStateStatus CLEAN.
  - (b) run 42402136024 (commit temporaire fb61b0ba, exit 1 en tête du job changes) : à 19:38, E2E smoke sorti de queued — Filtre failure, Frontend/Backend/E2E in_progress, Docker success ; fin 20:30 — Filtre failure, les quatre requis success, aucun skipped ; rollup des quatre : COMPLETED SUCCESS ; mergeStateStatus UNSTABLE.
  - (d) run 42406580735 (commit temporaire 39a54840, sleep 180 en tête du job changes) : Filtre in_progress à 20:31:03, gh run cancel à 20:31:04 ; fin : run cancelled, Filtre et les quatre requis cancelled, aucun skipped ; rollup des quatre : COMPLETED CANCELLED ; mergeStateStatus BLOCKED. La condition reste.
  - (e) run 42406684036 (commit temporaire ce81d08b, timeout-minutes 1 et sleep 180 sur le job changes) : fin 21:10 — run cancelled ; Filtre cancelled (délai dépassé) ; les quatre requis exécutés, success, aucun skipped ; rollup des quatre : COMPLETED SUCCESS ; mergeStateStatus UNSTABLE. La condition reste.
  Commits temporaires `fb61b0ba`, `39a54840`, `ce81d08b` défaits par `git revert --no-edit` (`2c261c9d`, `b83a5fa1`, `c2d5bcf7`) ; arbre égal à `9a78413a` après chacun (`git diff --stat` vide). Textes passés au relevé (`docs/ci.md`, CHANGELOG, `CLAUDE.md`), le cas nominal restant à relever en (c), sur la PR de la rétrospective (#928 reste ouverte jusque-là). Story **done**, sur (a), (b), (d), (e), comme le prévoit le mode opératoire.
- 2026-11-10 : **relevé de l'AC-6 (c)**, par l'orchestrateur, sur la PR de la rétrospective de l'Epic 33 [#971](https://github.com/guycorbaz/kesh/pull/971) (diff : `_bmad-output/implementation-artifacts/epic-33-retro-2026-11-10.md` et `sprint-status.yaml` seuls, tête `42542450`) — **conforme** : run 42805055989, Filtre des artefacts `completed | success`, motif au journal « ci-seuls-artefacts : seulement des artefacts de travail » (11:09:05Z) ; Backend (Rust), Frontend (Svelte), Docker build (sanity), E2E smoke (Playwright) : `completed | skipped` ; rollup des quatre : COMPLETED SKIPPED ; mergeStateStatus **CLEAN**. Le cas nominal est prouvé ; #928 se ferme à la fusion de #971 (`closes #928` au corps de la PR). Ce commit, lui aussi limité à `_bmad-output/`, relance le filtre sur la PR.

## Dev Agent Record

### Agent Model Used

Opus 5.5 (`claude-opus-5-5`).

### Completion Notes List

- **Rouges** : `e3311/rouges.log`, la garde de `ci.yml` sur le workflow d'origine, quinze assertions rouges, chacune nommée. ⚠️ Le script a été écrit avant la fin de ses cas, en brouillon : ses cas ne se sont pas vus rouges d'abord, et ce sont les mutations qui les prouvent.
- **Mutations** (`e3311/mut/resume.txt`) : M1 à M9 toutes tuées par le test shell. M8 jouée sur le job `frontend` ; M9 sur les quatre.
- **Gates, conformes à la fiche et au prompt** : `bash scripts/tests/ci-seuls-artefacts.test.sh` vert ; `cargo test -p kesh-api --test textes_coherents`, sous verrou, 23 passés (`e3311/textes.log`) ; **`actionlint` et `shellcheck` absents de la station** : lecture ligne à ligne du YAML, et `yaml.safe_load` : cinq jobs, `needs` et `if` relus. Aucun gate complet, ni E2E : aucun code de production ; le gate complet revient à l'intégration, et la PR d'intégration exerce le workflow pour de bon.
- **Ce qui reste à prouver** : AC-6 (a), (b), (c), (d) et AC-7, sur de vraies PR. La story ne passe `done` qu'après (a), (b), (d).
- ⚠️ **Écart de méthode, à déclarer** : script et test rédigés hors worktree, corrigés en brouillon par un script `python3`, puis copiés. Ils n'étaient pas encore au dépôt ; `ci.yml` a été édité par l'outil Edit.
- **Remédiation P1 (2026-11-07)** : rouge E-4, 1 assertion (`e3311/p2/rouges.log`) ; M10 et M11 tuées (`e3311/p2/mut/`) ; test shell vert, 53 assertions (`e3311/p2/test-shell.log`) ; `textes_coherents` sous verrou (`e3311/p2/textes.log`). Ce qui reste à prouver : AC-6 (a), (b), (c), (d), **(e)** et AC-7.
- Choix : C-e3311-1 à 3. Aucune migration, aucun code de production ; CHANGELOG *Ajouté*, `refs #928`.

### File List

`.github/workflows/ci.yml`, `scripts/ci-seuls-artefacts.sh`, `scripts/tests/ci-seuls-artefacts.test.sh`, `docs/ci.md`, `CLAUDE.md`, `CHANGELOG.md`, `_bmad-output/implementation-artifacts/{33-11-filtre-ci.md,33-11-filtre-ci-preuves-ac6.md,33-11-filtre-ci-review-prompt-p1.md,33-11-filtre-ci-review-prompt-p2.md,e33-choix-autonomes-e3311.md,sprint-status.yaml}`.
