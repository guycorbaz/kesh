# Story 55-4 : solde de la dette de l'epic

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md`. Release : celle de l'epic, **avant le tag**.
Règle : `CLAUDE.md` § *Priorités des défauts*, « Tarir la source, puis solder ».

## Story

En tant que mainteneur,
je veux que la dette laissée par les revues des stories 55-1 à 55-3 soit soldée avant la release,
afin que l'epic ne fasse pas monter le compte d'issues ouvertes par sa propre dette.

## Périmètre

- **Seulement** les issues ouvertes pendant l'Epic 55 **par les revues de ses stories** (LOW tracés, constats
  d'intégration), portant la référence de l'epic dans leur corps. Aucune issue antérieure, aucune autre epic.
- **Plafond : 2 issues** (fixé au kickoff, fichier d'epic). Au-delà, les suivantes restent ouvertes, nommées à la
  rétrospective avec leur raison.
- Frontend seulement, **sauf** une issue née de la 55-3 côté `kesh-report` (le seul code serveur de l'epic).
- Aucune migration, aucun changement de contrat de l'API.

## Critères d'acceptation

1. **AC-1 — Liste au départ.** Le Change Log nomme, au T0, chaque issue candidate (numéro, titre, story d'origine) et les
   deux retenues, avec le motif du choix (priorité, puis effet visible) ; la liste se lit depuis GitHub par le mainteneur
   ou son agent (`gh issue list … --search "Epic 55"`), l'agent de l'epic n'ayant pas `gh`.
2. **AC-2 — Chaque issue retenue** est corrigée **avec son test** (Vitest, Rust ou E2E selon le site), rouge avant le
   correctif (journal `kesh-gate-logs/s55-4/`), et son symptôme grepé sur le dépôt (`CLAUDE.md` § *Propagation post-patch*).
3. **AC-3 — Fermeture par la PR.** Le mot-clé de fermeture ne s'écrit que dans le corps de la PR de l'epic, jamais dans
   un commit (`CLAUDE.md` § *Issue Tracking Rule*) ; commits en `refs #N`.
4. **AC-4 — Rien hors plafond.** Aucune issue fermée pour le chiffre ; un défaut réel non traité reste ouvert.
5. **AC-5 — Manuel et textes** si une correction touche un texte visible : quatre locales, `.tex` seul.
6. **AC-6 — Captures** si un contrôle visible change (`CLAUDE.md` § *Recette visuelle*).

## Tâches

- [ ] **T0** — Liste des candidates (AC-1).
- [ ] **T1** — Corrections, tests rouges puis verts.
- [ ] **T2** — Gates de fin d'epic : `scripts/gate-ci.sh` et E2E complet (règle des gates de l'epic), journaux.

## Note de version

Rien en propre : chaque correction prend sa ligne dans la note de la story qu'elle complète, ou une ligne *Corrigé* de
l'epic. (refs les issues retenues)

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-25 — Fiche écrite au kickoff de l'Epic 55 (`drafted`).
