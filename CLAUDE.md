# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Kesh** is a Swiss personal and small business accounting software. It is currently in the design/planning phase, developed using the BMAD (Breakthrough Method of Agile AI-driven Development) framework. No application code has been written yet — the repository contains BMAD framework assets and design artifacts.

**Target stack**: Rust backend (Axum), Svelte frontend, MariaDB, web app only (no Tauri). Configuration via environment variables. Deployment via docker-compose.

**PRD**: `_bmad-output/planning-artifacts/prd.md` — Swiss accounting focus: QR Bill 2.2, pain.001.001.03, CAMT.053.001.04, multilingual (FR/DE/IT/EN).

## Communication

The user (Guy) works in **French**. All conversation and document output should be in French unless otherwise specified.

## Repository Structure

```
_bmad/                  # BMAD framework (agents, workflows, skills, config)
  bmm/                  # Build Method Methodology — discovery → implementation phases
  wds/                  # Workflow Design System — UX and design workflows
  cis/                  # Creative Innovation Strategy — strategic methodologies
  tea/                  # Test Architecture — QA and testing workflows
  bmb/                  # Builder — meta-skills for creating agents/workflows
  core/                 # Universal skills (brainstorming, reviews, editing)
  _config/              # CSV manifests (agents, skills, workflows, files)
  _memory/              # Persistent memory for sidecar agents
_bmad-output/           # Generated artifacts (planning, implementation, test)
design-artifacts/       # Project deliverables by phase (A through G)
  A-Product-Brief/      # Product positioning
  B-Trigger-Map/        # Business goals → user psychology
  C-UX-Scenarios/       # User interaction scenarios
  D-Design-System/      # UI components and tokens
  E-PRD/                # Requirements + design deliveries
  F-Testing/            # Test plans
  G-Product-Development/ # Implementation artifacts
docs/                   # Project documentation
.claude/skills/         # 118 installed BMAD skills for Claude Code
```

## BMAD Architecture

**Agents** are named personas (PM, Developer, Architect, QA, etc.) defined in `.md` files with menus that invoke skills or workflows. **Skills** are self-contained capabilities (52 total). **Workflows** are multi-step stateful processes (51 total) using step-file architecture — each step in a separate file, loaded just-in-time. Progress is tracked in document frontmatter.

Key manifests in `_bmad/_config/`: `agent-manifest.csv`, `skill-manifest.csv`, `workflow-manifest.csv`.

BMAD module config: `_bmad/bmm/config.yaml` — defines project name, user name, language preferences, and output paths.

## Key Patterns

- Workflows execute steps sequentially — never skip steps
- State is stored in document frontmatter (`stepsCompleted` array)
- Design artifacts follow the A→G phase progression
- Skills are invoked via `/skill-name` slash commands in Claude Code
- All generated output goes to `_bmad-output/` (not mixed with framework files)

## Code Quality Rules

- **DRY (Don't Repeat Yourself)** — No duplicated code. Extract shared logic into reusable functions/modules.
- **Documentation** — Source code must be documented following best practices: public APIs, complex logic, module-level docs. Rust: use `///` doc comments. Svelte: use JSDoc where appropriate.
- **Testing** — Test everything that can be tested. Unit tests for all business logic (especially the accounting engine, VAT calculations, and financial computations). Integration tests for parsers (CAMT.053, QR Bill, pain.001).
- **E2E Testing** — Use Playwright for all end-to-end tests. Each user journey from the PRD maps to a Playwright test scenario.

## Review Iteration Rule

**Règle de remédiation des revues (code review et spec validate)** :

Tant qu'une passe de revue remonte **au moins un finding de sévérité supérieure à LOW** (c'est-à-dire `CRITICAL`, `HIGH`, ou `MEDIUM`), on **relance une nouvelle passe de revue** après application des patches. Le critère d'arrêt est :

- **Uniquement des findings de sévérité `LOW`** (nits cosmétiques, améliorations de lisibilité, documentation mineure), OU
- **Maximum 8 passes atteint** (limite de budget LLM)

Pour chaque nouvelle passe de revue sur la même story :
- **Utiliser un LLM différent** de la passe précédente si possible (cycle Opus → Sonnet → Haiku → Opus), afin de contourner le biais d'auteur sur les patches qu'on vient d'appliquer. Les régressions introduites par la remédiation ne sont souvent détectables que par un modèle orthogonal.
- **Fenêtre de contexte fraîche** — ne pas réutiliser le contexte de la passe précédente.
- **Patches appliqués avant passe N+1** : chaque finding trouvé en passe N est remédié avant relancer la passe N+1.
- **Documenter dans le Change Log final** (pas une entrée par passe) : résumé du trend numérique (passe 1: X findings → passe 2: Y → ... → passe N: 0 > LOW), modèles LLM utilisés, décisions de reclassement.

**Boucle automatique** :
- `bmad-create-story validate` : relancer automatiquement en boucle après chaque passe (LLM différent, patches appliqués, contexte frais) jusqu'à atteindre 0 CRITICAL/HIGH/MEDIUM OU 8 passes.
- `bmad-code-review` : appliqué après implémentation (`dev-story` complétée), même boucle.

Cette règle s'applique à :
- `bmad-create-story validate` (revue de spec multi-passes)
- `bmad-code-review` (revue de code adversariale)
- Toute revue adversariale similaire où le budget LLM le permet

**Exception** : si un finding `MEDIUM+` est explicitement reclassé en **dette technique documentée** (dans une section `Security debt` / `Performance debt` / équivalente du story file ou des Dev Notes) avec un propriétaire et une story de remédiation planifiée, il compte comme « résolu » pour cette itération.

## Issue Tracking Rule

**Règle de traçage des CR, KF et bug reports** :

**GitHub Issues est l'unique source de vérité.** Toute nouvelle découverte d'un **CR (Change Request)**, d'une **KF (Known Failure)** ou d'un **bug report** DOIT être créée comme GitHub Issue sur [guycorbaz/kesh/issues](https://github.com/guycorbaz/kesh/issues) en utilisant le template approprié dans `.github/ISSUE_TEMPLATE/`.

**Pas de tracking local en parallèle** — aucun fichier dans le repo (Markdown, YAML, tableau de story) ne doit maintenir sa propre liste de KF/CR/bugs. Pas de double-tracking, pas de sync bidirectionnelle, pas de dérive de source de vérité.

| Type | Template | Labels appliqués par le template |
|------|----------|----------------------------------|
| Bug report | `bug_report.yml` | `bug`, `triage` |
| KF | `known_failure.yml` | `known-failure`, `triage` (+ `technical-debt` à ajouter manuellement si dette persistante) |
| CR / feature request | `feature_request.yml` | `enhancement`, `triage` |

Titre homogène pour les KF : `[KF-NNN] description` — facilite la recherche visuelle dans la liste d'issues.

### Quand créer une issue

- **Bug report** : dès qu'un comportement incorrect est reproduit, **hors du flux normal de dev d'une story en cours**. Si le bug est découvert pendant l'implémentation d'une story liée, le corriger directement dans la story et le documenter dans le Change Log de la story.
- **KF** : dès qu'un test cassé ou un comportement défaillant est détecté **hors scope du travail courant**.
- **CR** : **avant** tout changement de scope qui modifie le PRD ou les AC d'une story déjà validée (`done` ou en `review`). Ne pas faire de modification silencieuse du scope.

### Commits qui adressent une issue

Chaque commit qui adresse partiellement ou totalement une issue doit mentionner son numéro :
- **Fermer l'issue** : `fix(api): close IDOR on contacts (#2)` ou `... (closes #2)` ou `... (fixes #2)` — GitHub ferme automatiquement l'issue au merge sur `main`.
- **Référencer sans fermer** : `fix: round invoice totals (refs #42)` — lie le commit à l'issue sans la fermer.

### Legacy

Deux fichiers dans `docs/` sont **archivés et ne doivent plus être mis à jour** — ils ne servent que de trace historique :

- `docs/change_request.md` — archivé depuis 2026-04-16, 8 CR migrés sur GitHub.
- `docs/known-failures.md` — archivé depuis 2026-04-18, 7 KF migrées sur GitHub (KF-001 à KF-007).

Toute nouvelle KF/CR/bug → GitHub uniquement. Ne **pas** rouvrir ces fichiers pour y ajouter des entrées.

## Règle de commit et push

**Commit systématique après chaque étape BMAD** :

On commit localement après chaque étape structurante du workflow BMAD, sans attendre :

- **Après `bmad-create-story`** (ou `bmad-quick-spec`) — la spec est un artefact versionné, pas un brouillon.
- **Après chaque passe de `bmad-create-story validate`** — chaque passe produit un Change Log entry + patches éventuels, qu'il faut tracer séparément.
- **Après `bmad-dev-story`** (ou `bmad-quick-dev`) — l'implémentation est prête à être revue.
- **Après chaque passe de `bmad-code-review`** — idem validate, chaque passe a ses findings + patches.

Un commit par étape, pas un commit géant en fin de story. Ça permet de revenir en arrière proprement et de voir le fil du processus dans `git log`.

**Push à la demande ou en fin d'epic** :

On **ne push pas automatiquement** après chaque commit. Deux déclencheurs :

1. **Sur demande explicite** de Guy (ex. « pousse », « fais une PR », « ouvre la PR »).
2. **À la fin d'un epic**, après la rétrospective (`bmad-retrospective`). Le push de fin d'epic est le moment où l'on regroupe plusieurs stories dans un PR (ou plusieurs PRs) et où l'on matérialise la clôture de l'epic.

**Exception** : si une règle de workflow BMAD spécifique impose un push (ex. CI check obligatoire avant passe suivante), on push. À justifier dans le message de commit ou dans la conversation.

**Synchroniser le planning du README à chaque commit** :

Avant de créer un commit, vérifier que la **section « Feuille de route »** de `README.md` reflète encore l'état du projet. Le but est d'éviter qu'elle dérive au fil des epics et donne une fausse image (epic marqué "En cours" alors que toutes ses stories sont done, feature listée *(à venir)* alors qu'elle est livrée, etc.).

Vérifier en particulier après :

- **Clôture d'un epic** (rétro done) → mettre à jour le statut dans le tableau des versions (✅ Done / 🚧 En cours / 📋 Backlog).
- **Story qui livre une feature listée dans la section « Fonctionnalités »** → retirer le marqueur *(à venir)*.
- **Renumérotation d'epics** (cf. décision rétro Epic 5 qui a renuméroté 6→13 en 7→14) → propager le nouveau découpage.
- **Changement de scope d'une version** (feature déplacée v0.1 → v0.2 ou inverse, validé via CR) → refléter dans le tableau.

Si une mise à jour est nécessaire, l'inclure **dans le même commit** que le changement qui l'a déclenchée (typiquement le merge de la dernière story de l'epic, ou la rétro). Pas de commit séparé « sync README » a posteriori — sinon le `git log` ne raconte plus l'histoire.

Si le commit ne change rien à la planification (refactor interne, fix de bug, code review patches) le README reste tel quel.
