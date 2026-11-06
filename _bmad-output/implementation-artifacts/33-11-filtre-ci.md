# Story 33-11 : la CI saute ses jobs lourds quand une PR ne touche que `_bmad-output/`

Status: ready-for-dev

*Epic 33 (v0.17.5). Demande de Guy du 2026-11-06 : « oui, ajoute un filtre ci pour éviter de perdre du temps inutile ». Clé de choix `C-e3311-k`, registre `e33-choix-autonomes-e3311.md`. Aucune migration, aucun code de production. Issue : #928 (`refs` dans les commits).*

## Story

As a Project Lead,
I want qu'une PR qui ne porte que des artefacts de travail n'attende pas la suite complète,
so that un cycle de CI ne coûte plus 37 minutes pour de la documentation interne (PR #927, job backend, 2026-11-06).

## Faits établis

- F1. `.github/workflows/ci.yml:3-7` : déclencheurs `push` sur `main` et `pull_request` vers `main`, sans filtre. Quatre jobs : `backend` (`:21`), `frontend` (`:208`), `docker-build` (`:252`), `e2e-smoke` (`:334`, `needs: docker-build`, `:337`).
- F2. Protection de `main` : ces quatre noms de checks sont requis (`gh api repos/guycorbaz/kesh/branches/main/protection/required_status_checks`). **Un `paths-ignore` au niveau du workflow bloquerait donc toute PR filtrée**, ses checks restant en attente.
- F3. GitHub compte comme réussi un job sauté par sa condition `if`, et le nom du check est conservé. **À constater sur une PR réelle** (AC-6) : la fiche ne repose pas sur la documentation seule.
- F4. `git grep -n '_bmad-output' -- crates frontend/src frontend/tests scripts .github` rend trois mentions, toutes des commentaires (`kesh_version.sql:4`, `version.rs:4`, `i18n-dette-connue.ts:23`) : aucun test ne lit ce dossier. À refaire au développement.
- F5. Les tests lisent d'autres textes : `crates/kesh-api/tests/textes_coherents.rs`, `mcp_matrice_manuel.rs`, `configuration_transmise.rs` lisent manuels, CHANGELOG, `.env.example`. **Manuels, `docs/`, `README.md`, `CHANGELOG.md`, `CLAUDE.md`, `website/` restent donc hors du filtre.** La PR #423, citée au `CLAUDE.md`, n'aurait pas été entièrement filtrée ; c'est voulu.
- F6. `CLAUDE.md` § *Quand sauter* et `docs/ci.md:12` affirment l'absence de filtre : ils deviennent faux.

## Critères d'acceptation

1. **AC-1.** Un job `changes`, sans dépendance et sans condition, calcule la sortie `code` : `false` **seulement si** l'événement est `pull_request` et que **tous** les fichiers du diff de la PR sont sous `_bmad-output/`.
2. **AC-2. Le filtre échoue vers la CI complète.** Tout autre cas rend `code=true` : événement `push`, liste vide, erreur de lecture du diff, fichier renommé depuis ou vers un autre dossier, fichier au nom contenant espace ou accent, PR de plus de fichiers que l'API n'en rend.
3. **AC-3.** `backend`, `frontend`, `docker-build` ont `needs: changes` et `if: needs.changes.outputs.code == 'true'` ; `e2e-smoke` a `needs: [changes, docker-build]` et la même condition. Leurs `name:` ne changent pas (F2), ni les réglages de protection.
4. **AC-4.** La décision vit dans un script du dépôt, `scripts/ci-seuls-artefacts.sh`, qui lit une liste de fichiers sur l'entrée standard et rend `code=true` ou `code=false` ; aucune action tierce nouvelle.
5. **AC-5. Tests du script**, lancés par un job léger ou par `scripts/test-ci-filtre.sh`, en CI à chaque exécution du job `changes` : liste vide → `true` ; `_bmad-output/a.md` seul → `false` ; `_bmad-output/a.md` + `crates/x.rs` → `true` ; `_bmad-output-bis/a.md` → `true` ; `docs/manual/fr/x.tex` → `true` ; `CHANGELOG.md` → `true` ; `.github/workflows/ci.yml` → `true` ; chemin `_bmad-output/../crates/x.rs` → `true` ; nom à espace sous `_bmad-output/` → `false`.
6. **AC-6. Preuve sur de vraies PR, consignée au Change Log, liens cités.** La PR d'intégration, qui touche du code, fait tourner les quatre jobs. Une PR portant un seul fichier sous `_bmad-output/` rend les quatre checks sautés et **reste fusionnable** (`mergeStateStatus` relu) ; si elle ne l'est pas, le filtre est retiré et Guy prévenu. Cette seconde preuve vient **après** la fusion de la release, sur la PR de la rétrospective de l'Epic 33, et l'orchestrateur la consigne.
7. **AC-7.** `push` sur `main` : les quatre jobs tournent toujours, y compris après un commit d'artefacts seul.
8. **AC-8. Textes.** `CLAUDE.md` § *Quand sauter* et `docs/ci.md` disent le filtre, son périmètre étroit, et ce qui en reste exclu ; le passage historique reste, annoté et daté. `git grep` du symptôme (« aucun filtre », « Aucun filtre », `paths:`, « quatre jobs ») sur tout le dépôt, sites traités.

## Tâches

- [ ] T1. Relire F1 à F5 au code, lignes citées au Change Log.
- [ ] T2. Tests du script d'abord, rouges ; script ; verts (AC-4, AC-5).
- [ ] T3. `ci.yml` : job `changes`, conditions (AC-1 à AC-3). Liste des fichiers par l'API de la PR, paginée, ou par `git diff` sur la base et la tête avec l'historique nécessaire : le développeur choisit et consigne, pourvu qu'AC-2 tienne.
- [ ] T4. Textes (AC-8). CHANGELOG `[0.17.5]`, *Modifié*.
- [ ] T5. `actionlint` si disponible, sinon lecture ligne à ligne, déclarée.

## Mutations

M1 `-v` retiré ou condition inversée dans le script ; M2 préfixe sans barre finale (`_bmad-output-bis/` passerait) ; M3 liste vide rendue `false` ; M4 chemin à `..` accepté. Chacune tuée par un cas d'AC-5, consignée dans `kesh-gate-logs/e3311/mut/`.

## Gates

Story sans code de production : `scripts/test-ci-filtre.sh`, `shellcheck` si présent, `cargo test -p kesh-api --test textes_coherents` sous verrou si un texte qu'il lit change. Le gate complet reste à l'intégration, et la PR d'intégration exerce le workflow pour de bon.

## Dev Notes

- Une erreur dans ce filtre peut faire sauter des tests à tort : le risque est **muet**. D'où un périmètre d'un seul dossier, le défaut `true`, et `main` jamais filtré, qui reste le filet.
- Élargir le périmètre plus tard demande de vérifier d'abord que rien ne lit les fichiers visés (F5).

## Change Log

- 2026-11-06 — Fiche écrite par l'orchestrateur (Opus 5.5) sur demande de Guy. Prochaine : validation P1.
