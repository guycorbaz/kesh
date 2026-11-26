# Prompt de revue de code P1 — story 37-3b

Première passe complète de `bmad-code-review` (lis `.claude/skills/bmad-code-review/SKILL.md` et suis-le : couches
Blind Hunter, Edge Case Hunter, Acceptance Auditor, triage), contexte frais, sur la story 37-3b, worktree
`<worktree>/kesh-s37-3b` (branche `story/37-3b-ecritures-en-devise-ecrans`). N'écris
nulle part ailleurs, sauf journaux dans `$GATE_LOGS/s37-3b/review-p1/`.

## Périmètre

- Diff : `git diff story/37-3a-ecritures-en-devise-modele...HEAD` (la branche part de la pointe de la 37-3a avant sa revue ; le périmètre propre à la 37-3b est ce diff ; code, tests, manuels `.tex`, i18n, docs).
- Spec : la fiche `_bmad-output/implementation-artifacts/37-3b-ecritures-en-devise-ecrans.md` (AC, FR/NFR, Dev Agent
  Record et écarts de la fiche), l'epic `_bmad-output/planning-artifacts/epic-37-multi-devises.md`.
- Écarts et limites du dev : Dev Agent Record de la fiche (manuel en français seulement, `Select` de bits-ui non
  testable sous jsdom, pas d'aller-retour `.keshbackup` d'une ouverture en devise, pas d'E2E de suppression d'une
  réévaluation, interblocage possible de deux conversions avec projets, rejoué, sans test de course). Tous sont à juger.

## Axes (déclare ceux exercés et non exercés)

Correction des AC ; comptabilité (aucune écriture fausse, gardes des comptes de différence) ; verrous, ordre, rejeu et
interblocages ; migration P1–P8 (non breaking, audit d'idempotence,
P6, P7, squash de test) ; sauvegarde, restauration 0.17.6, CSV, remise à zéro ; RBAC (Administrateur en session, jamais
clé d'API) ; audit ; MCP (tailles) ; i18n 4 locales ; **manuels** (`.tex` contre le code ; PDF non commité) ; tests
(muets, manquants, mutation) ; DRY ; sécurité ; sites non résolus.

## Phase 2

Applique les patches des findings (CRITICAL à LOW selon le skill), avec leurs tests ; gate ciblé (fmt, clippy workspace,
nextest des binaires touchés — **gate complet si un patch touche `kesh-db`** : migrations, repository, `post_restore`),
sous ``$GATE_LOGS/heavy.sh scripts/mem-guard.sh <commande>` (au plus deux exécutions lourdes sur la station ; backend lancé avec `7>&- 8>&- 9>&-`), base `kesh_s37_3b` (cf. `docs/agents-orchestration.md`) ; commit local
« fix(37-3b) : revue de code P1 — … » (aucun mot-clé de fermeture, aucune mention d'IA). Change Log de la fiche :
« 2026-11-16 — revue de code P1 (<modèle>) : <décompte> ». N'affirme que ce qui a tourné.

## Interdits

Aucun `git push`, aucune branche autre que la story, aucun redémarrage du conteneur MariaDB, aucun
`scripts/prepare-release.sh` ni script mutant hors gates.

## Rapport (français)

Findings `37-3b-CR-P1-N` (sévérité, fichier:ligne, constat, correctif appliqué) ; axes exercés / non exercés ; gates
exécutés (chiffres) ; décompte.




## Propre à la 37-3b — points à examiner en priorité

1. **Les montants** : le formulaire ne doit jamais envoyer un CHF ou un cours faux. Jumeau TypeScript du calcul
   (`fx.ts` et fixtures partagées avec kesh-core : mêmes résultats au centime ?), cours proposé, ligne d'écart
   (`completeDifference`, A31), cours déduit du CHF (A32), arrondis, saisie ambiguë (« 1.234 », règle de la 36-3).
2. **Soldes d'ouverture en devise et assistant Conversion** (route `create_conversion`) : écritures justes, compte de
   transit qui revient à 0 (A33 : frais sur un compte de frais de la même devise), verrous, rejeu, refus.
3. Libellés des codes de lot et motif `ENTRY_IS_FX_REVALUATION`, sélecteurs de comptes (devise filtrée),
   accessibilité, i18n des quatre locales, manuel `.tex` (et le PDF compilé hors dépôt), captures
   (`kesh-gate-logs/s37-3b/captures/`).
4. Un défaut du code de la 37-3a (contrôle central, API) ne se corrige pas ici : note-le dans ton rapport. La 37-3a
   finit sa clôture en parallèle.
5. Base de test : `kesh-mariadb-dev`, port 3306 (la 37-3a est sur `kesh-mariadb-dev2`, 3307 : n'y touche pas).
   **Aucune mention d'IA dans les commits**, aucun `GRANT`.
