# Story 33-11 : la CI saute ses jobs lourds quand une PR ne touche que `_bmad-output/`

Status: ready-for-dev

*Epic 33 (v0.17.5). Demande de Guy du 2026-11-06 : « oui, ajoute un filtre ci pour éviter de perdre du temps inutile ». Clé de choix `C-e3311-k`, registre `e33-choix-autonomes-e3311.md`. Aucune migration, aucun code de production. Commits en `refs #928` ; PR d'intégration : `refs #928` ; **`closes #928` à la PR de la rétrospective**, qui porte la dernière preuve (AC-6 c).*

## Story

As a Project Lead,
I want qu'une PR qui ne porte que des artefacts de travail n'attende pas la suite complète,
so that un cycle de CI ne coûte plus 37 minutes pour de la documentation interne (PR #927, job backend, 2026-11-06).

## Faits établis

- F1. `.github/workflows/ci.yml:3-7` : déclencheurs `push` sur `main` et `pull_request` vers `main`, sans filtre. Quatre jobs : `backend` (`:21`), `frontend` (`:208`), `docker-build` (`:252`), `e2e-smoke` (`:334`, `needs: docker-build`, `:337`).
- F2. Protection de `main` : ces quatre noms de checks sont requis (`gh api repos/guycorbaz/kesh/branches/main/protection/required_status_checks`). **Un `paths-ignore` au niveau du workflow bloquerait donc toute PR filtrée**, ses checks restant en attente. **`changes` ne devient pas requis.**
- F3. **Documentation GitHub, lue le 2026-11-06, non exécutée** (d'où l'AC-6). *Troubleshooting required status checks* : « Successful check statuses are `success`, `skipped`, and `neutral` » ; un job sauté par sa condition « reports 'Success' » ; un job dont la dépendance a échoué : « The dependent job is skipped and may not block merging », remède : « Use `always()` with `needs` for required checks that depend on other jobs ». *Expressions* : « A default status check of `success()` is applied unless you include one of these functions » ; `always` s'exécute « even when canceled ». *Contexts* : « If you attempt to dereference a nonexistent property, it will evaluate to an empty string » : une sortie jamais écrite vaut donc `''`, et `'' != 'false'` est vrai.
- F4. `git grep -n '_bmad-output' -- crates frontend/src frontend/tests scripts .github` rend trois mentions, toutes des commentaires (`kesh_version.sql:4`, `version.rs:4`, `i18n-dette-connue.ts:23`) : aucun test ne lit ce dossier. À refaire au développement.
- F5. Les tests lisent d'autres textes : `crates/kesh-api/tests/textes_coherents.rs`, `mcp_matrice_manuel.rs`, `configuration_transmise.rs` lisent manuels, CHANGELOG, `.env.example`. **Manuels, `docs/`, `README.md`, `CHANGELOG.md`, `CLAUDE.md`, `website/` restent donc hors du filtre.** La PR #423, citée au `CLAUDE.md`, n'aurait pas été entièrement filtrée ; c'est voulu.
- F6. `CLAUDE.md:293` et `docs/ci.md:12` affirment l'absence de filtre : ils deviennent faux.
- F7. **Sonde sur un dépôt jetable, 2026-11-06.** `git diff --name-only --no-renames -z A...B` rend les deux noms d'un renommage, et les noms bruts (espace, accent), sans guillemets ; sans `--no-renames`, le nom d'arrivée seul. Un SHA inconnu : code 128, rien sur la sortie. Git refuse d'indexer un chemin à `..` (`Invalid path`).

## Conception

Toute la décision vit dans le script, le YAML n'est que de la plomberie.

- **`scripts/ci-seuls-artefacts.sh <événement> <base> <tête>`**, `LC_ALL=C`, écrit **une seule ligne** sur sa sortie, en dernier : `code=true` ou `code=false`. Il lit lui-même `git diff --name-only --no-renames -z <base>...<tête>`, sortie en fichier temporaire, code de retour contrôlé avant toute décision. Motif sur la sortie d'erreur.
- **Pas d'API, donc ni pagination, ni plafond, ni droit `pull-requests: read`.** Les trois points comparent à la base de fusion : ce que la PR apporte, même après une fusion de `main` dans sa branche.
- **Un nom réel, entièrement sous `_bmad-output/`, rend `false`, quels que soient ses caractères.** `-z` supprime les noms cités ; un nom lu autrement, ou ailleurs, rend `true`.
- **Job `changes`**, trois étapes, dans cet ordre : `actions/checkout` en `fetch-depth: 0` ; `scripts/tests/ci-seuls-artefacts.test.sh` ; le script, `>> "$GITHUB_OUTPUT"`, arguments passés par `env:`. Un test rouge arrête le job avant toute sortie.
- **Conditions par exclusion** : seule une sortie écrite `false` saute un job.

## Critères d'acceptation

1. **AC-1.** `changes`, sans dépendance et sans condition, rend `code=false` **seulement si** l'événement est `pull_request` et que le diff, non vide, ne porte que des fichiers sous `_bmad-output/`.
2. **AC-2. Le filtre échoue vers la CI complète.** Tout autre cas rend `code=true`, ou n'écrit rien : événement `push`, diff vide, arguments manquants, SHA inconnu, historiques sans base commune, `git` en échec, fichier renommé depuis ou vers un autre dossier.
3. **AC-3.** `backend`, `frontend`, `docker-build` : `needs: changes`, `if: ${{ !cancelled() && needs.changes.outputs.code != 'false' }}`. `e2e-smoke` : `needs: [changes, docker-build]`, `if: ${{ !cancelled() && needs.changes.outputs.code != 'false' && needs.docker-build.result == 'success' }}`. Leurs `name:` ne changent pas (F2), ni les réglages de protection.
4. **AC-4.** Le script est celui de la *Conception* ; aucune action tierce nouvelle.
5. **AC-5. Tests, `scripts/tests/ci-seuls-artefacts.test.sh`**, sur un mini-dépôt git réel sous `mktemp -d` (identité passée par `git -c`), chaque cas par de vrais commits :
   - **`false`** : `_bmad-output/a.md` seul ; nom à espace et nom accentué sous `_bmad-output/` ;
   - **`true`** : `push` avec le même diff ; `_bmad-output/a.md` + `crates/x.rs` ; `_bmad-output-bis/a.md` ; `_bmad-output` comme fichier ; `docs/manual/fr/x.tex` ; `CHANGELOG.md` ; `.github/workflows/ci.yml` ; nom accentué hors du dossier ; renommage de `crates/x.rs` vers `_bmad-output/x.rs`, et l'inverse ; diff vide ; arguments manquants ; SHA inconnu ; branche orpheline ; `git` factice (`PATH`) qui rend `_bmad-output/a.md` puis sort en 1 ;
   - **garde de `ci.yml`** : les quatre jobs portent les conditions de l'AC-3, aucun `== 'true'`, et l'étape des tests précède celle qui écrit `GITHUB_OUTPUT`.
6. **AC-6. Preuve sur de vraies PR, par l'orchestrateur, liens et `gh run view --json jobs` cités au Change Log.**
   - **(a)** La PR d'intégration, qui touche du code, fait tourner les quatre jobs.
   - **(b) Le chemin de l'échec, avant la fusion** : sur cette PR, un commit temporaire ajoute `exit 1` en première étape de `changes`. Attendu : `changes` en échec, et les quatre jobs **exécutés**, aucun `skipped`, `e2e-smoke` compris. Le commit se retire par `git revert`, sans réécrire la branche, et la CI repasse. **Si un job est sauté, la PR ne se fusionne pas** avec le filtre, et Guy est prévenu.
   - **(c)** Après la fusion, seul moment où `ci.yml` n'est plus dans le diff : la PR de la rétrospective, qui ne porte que `_bmad-output/`, rend les quatre checks sautés et **reste fusionnable** (`mergeStateStatus` relu). Sinon, le filtre est retiré et Guy prévenu.
   - La story passe `done` sur (a) et (b) ; (c) garde l'issue ouverte, non la story.
7. **AC-7.** `push` sur `main` : les quatre jobs tournent toujours, y compris après un commit d'artefacts seul.
8. **AC-8. Textes.** `CLAUDE.md` § *Quand sauter* et `docs/ci.md` disent le filtre, son périmètre étroit, et ce qui en reste exclu ; le passage historique reste, annoté et daté. `git grep` du symptôme (« aucun filtre », « Aucun filtre », `paths:`, « quatre jobs ») sur tout le dépôt, sites traités.

## Tâches

- [ ] T1. Relire F1 à F7 au code, lignes citées au Change Log.
- [ ] T2. Tests d'abord, rouges ; script ; verts (AC-1, AC-2, AC-4, AC-5).
- [ ] T3. `ci.yml` : job `changes`, conditions (AC-3).
- [ ] T4. Textes (AC-8). CHANGELOG `[0.17.5]`, *Ajouté*.
- [ ] T5. `actionlint` si disponible, sinon lecture ligne à ligne, déclarée.

## Mutations

Chacune tuée par un cas de l'AC-5, consignée dans `kesh-gate-logs/e3311/mut/` : **M1** condition inversée ; **M2** préfixe sans barre finale ; **M3** liste vide rendue `false` ; **M4** `--no-renames` retiré ; **M5** `-z` retiré ; **M6** événement non contrôlé ; **M7** code de retour de `git` ignoré ; **M8** `== 'true'` rétabli dans `ci.yml`.

## Gates

Story sans code de production : `scripts/tests/ci-seuls-artefacts.test.sh`, `shellcheck` si présent, `cargo test -p kesh-api --test textes_coherents` sous verrou si un texte qu'il lit change. Le gate complet reste à l'intégration, et la PR d'intégration exerce le workflow pour de bon.

## Dev Notes

- Une erreur dans ce filtre peut faire sauter des tests à tort : le risque est **muet**. D'où un périmètre d'un seul dossier, des conditions par exclusion, et `main` jamais filtré, qui reste le filet.
- **Limites assumées.** (1) Annuler le run à la main pendant `changes`, quelques secondes, saute les jobs par `!cancelled()`, ce qu'un `always()` éviterait, mais il lancerait aussi les jobs d'un run remplacé (`cancel-in-progress`, `ci.yml:11`). Ne pas fusionner sur un run annulé. (2) Le script et ses tests viennent de la PR elle-même ; mais une PR qui les modifie sort du périmètre, et un script faux doit encore passer ses tests.
- Élargir le périmètre plus tard demande de vérifier d'abord que rien ne lit les fichiers visés (F5).

## Recoupements

`docs/ci.md`, `CLAUDE.md`, `CHANGELOG.md` avec **33-9** et **33-10**, autres paragraphes, union à l'intégration. **33-10** lit `ci.yml` (ligne `cargo test`, boucle de migrations) : ce filtre n'y touche pas.

## Change Log

- 2026-11-06 — Fiche écrite par l'orchestrateur (Opus 5.5) sur demande de Guy. Prochaine : validation P1.
- 2026-11-06 — Première validation (Opus 5.5, rapport `e33-validate-p2-e334.md` : 1 HIGH, 2 MEDIUM, 2 LOW), remédiation (Opus 5.5), aucun réfuté, aucune exécution de Kesh ni de la CI. **H-R11-1** : conditions par exclusion, décision de Guy (`!cancelled()`, `changes` non requis) ; tests avant la sortie ; AC-6 (b) ; documentation relue (F3). **M-R11-2** : tranché, AC-2 et AC-5 alignés. **M-R11-3** : l'extraction passe dans le script, testée sur un dépôt réel ; **l'API est écartée**, sonde F7. **L-R11-4** : sans objet, sans API. **L-F11-5** : `closes #928` à la PR de la rétrospective, *Ajouté*, *Recoupements*. **Retirés** : le choix ouvert de T3, la lecture sur l'entrée standard, le cas et la mutation `..` (git refuse ce chemin, F7), et la « PR de plus de fichiers que l'API n'en rend ». Le script de test se renomme selon le patron `scripts/tests/*.test.sh`. Mutations : 8. Passe suivante requise.
