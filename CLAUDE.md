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

- **Zéro finding**, OU
- **Uniquement des findings de sévérité `LOW`** (nits cosmétiques, améliorations de lisibilité, documentation mineure)

Pour chaque nouvelle passe de revue sur la même story :
- **Utiliser un LLM différent** de la passe précédente si possible (Opus ↔ Sonnet ↔ Haiku), afin de contourner le biais d'auteur sur les patches qu'on vient d'appliquer. Les régressions introduites par la remédiation ne sont souvent détectables que par un modèle orthogonal.
- **Fenêtre de contexte fraîche** — ne pas réutiliser le contexte de la passe précédente.
- **Documenter dans le Change Log** les findings trouvés, les patches appliqués, et le modèle utilisé.

Cette règle s'applique à :
- `bmad-create-story validate` (revue de spec multi-passes)
- `bmad-code-review` (revue de code adversariale)
- Toute revue adversariale similaire où le budget LLM le permet

**Exception** : si un finding `MEDIUM+` est explicitement reclassé en **dette technique documentée** (dans une section `Security debt` / `Performance debt` / équivalente du story file ou des Dev Notes) avec un propriétaire et une story de remédiation planifiée, il compte comme « résolu » pour cette itération.
