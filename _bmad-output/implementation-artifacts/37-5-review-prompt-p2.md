# Prompt de revue de code P2 — story 37-5

Deuxième passe complète (Sonnet, modèle différent de la P1 Opus) de `bmad-code-review` (lis `.claude/skills/bmad-code-review/SKILL.md` et suis-le : couches
Blind Hunter, Edge Case Hunter, Acceptance Auditor, triage), contexte frais, sur la story 37-5, worktree
`/home/beat/claude/kesh/.claude/worktrees/kesh-s37-5` (branche `story/37-5-reevaluation-fin-exercice`). N'écris
nulle part ailleurs, sauf journaux dans `/home/beat/claude/kesh-gate-logs/s37-5/review-p2/`.

## Périmètre

- Diff : `git diff epic/37-multi-devises...HEAD` (code, tests, manuels `.tex`, i18n, docs).
- Spec : la fiche `_bmad-output/implementation-artifacts/37-5-reevaluation-fin-exercice.md` (AC, FR/NFR, Dev Agent
  Record et écarts C-37-2-1 à 3), l'epic `_bmad-output/planning-artifacts/epic-37-multi-devises.md`.
- Écarts du dev : Dev Agent Record de la fiche ; la P1 les a jugés acceptables.

## Axes (déclare ceux exercés et non exercés)

Correction des AC ; comptabilité (aucune écriture fausse, gardes des comptes de différence) ; verrous, ordre, rejeu et
interblocages ; migration P1–P8 (non breaking, audit d'idempotence,
P6, P7, squash de test) ; sauvegarde, restauration 0.17.6, CSV, remise à zéro ; RBAC (Administrateur en session, jamais
clé d'API) ; audit ; MCP (tailles) ; i18n 4 locales ; **manuels** (`.tex` contre le code ; PDF non commité) ; tests
(muets, manquants, mutation) ; DRY ; sécurité ; sites non résolus.

## Phase 2

Applique les patches des findings (CRITICAL à LOW selon le skill), avec leurs tests ; gate ciblé (fmt, clippy workspace,
nextest des binaires touchés — **gate complet si un patch touche `kesh-db`** : migrations, repository, `post_restore`),
sous ``/home/beat/claude/kesh-gate-logs/heavy.sh scripts/mem-guard.sh <commande>` (au plus deux exécutions lourdes sur la station ; backend lancé avec `7>&- 8>&- 9>&-`), base `kesh_s37_5` (cf. `docs/agents-orchestration.md`) ; commit local
« fix(37-5) : revue de code P2 — … » (aucun mot-clé de fermeture, aucune mention d'IA). Change Log de la fiche :
« 2026-11-16 — revue de code P2 (<modèle>) : <décompte> ». N'affirme que ce qui a tourné.

## Interdits

Aucun `git push`, aucune branche autre que la story, aucun redémarrage du conteneur MariaDB, aucun
`scripts/prepare-release.sh` ni script mutant hors gates.

## Rapport (français)

Findings `37-5-CR-P2-N` (sévérité, fichier:ligne, constat, correctif appliqué) ; axes exercés / non exercés ; gates
exécutés (chiffres) ; décompte.

## Propre à la P2

- Relis d'abord les deux commits de la P1 (`7ab0b018`, `0c90cc3a`) et leurs entrées au Change Log : 6 MEDIUM (champ de
  cours inutile, bouton actif pendant une relecture, blocker `RATE_UNAVAILABLE` de l'aperçu, tests de l'AC-7, compteurs)
  et l'échappement commun des cellules texte des CSV de rapports (`csv_sanitize` déplacée dans `kesh_core::csv`,
  réexportée par `kesh-api`, appliquée par `kesh-report`). Cherche les **défauts introduits** (le compteur `rereading`
  peut-il rester bloqué après une erreur ? l'ordre des refus est-il le même aux deux routes ? l'échappement touche-t-il
  une cellule numérique, un en-tête, un CSV relu par un import ? les exports de `kesh-api` gardent-ils exactement leur
  comportement ? les tests rougissent-ils sans leur correctif ?), puis le reste de la story.
- **Corrige D1** (relevé par la P1, à ne pas laisser en dette : c'est un contournement de K9) : un compte de bilan en
  devise mouvementé qui devient non imputable (sous-compte créé, `postable=false`) sort du prédicat des candidats, donc
  de l'aperçu, de K9 et du refus de clôture. Un tel compte, s'il garde un écart à réévaluer, doit **bloquer** la clôture
  (K9 en attention, refus 409) avec un motif qui dit quoi faire, même si l'écriture de réévaluation ne peut pas se passer
  sur lui ; écris la règle retenue dans la fiche, avec ses tests (aperçu, K9, clôture refusée).
- Corrige aussi **D2** (un compte présélectionné hors bornes fait refuser tout l'aperçu : il doit sortir en blocker par
  compte) et écris le test **U1** (aperçu après restauration d'un `.keshbackup`), s'ils tiennent en quelques lignes.
- Le message de commit qui touche l'échappement des CSV reste neutre : ni « sécurité », ni « injection », ni « formule ».
- **Aucune mention d'IA dans les commits**, aucun `GRANT`. Base `kesh-mariadb-dev3` (3308) seulement.
- Gates : gate complet si `kesh-db` est touché (`heavy.sh scripts/test-fast.sh --ci`, sans `mem-guard.sh` en plus) ;
  Vitest complet seul et `npm run build` si le frontend est touché. Pas d'E2E complet ici (fait à la clôture).
