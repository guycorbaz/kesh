# Prompt de revue de code P2 — story 37-3a

Deuxième passe complète (Sonnet, modèle différent de la P1 Opus) de `bmad-code-review` (lis `.claude/skills/bmad-code-review/SKILL.md` et suis-le : couches
Blind Hunter, Edge Case Hunter, Acceptance Auditor, triage), contexte frais, sur la story 37-3a, worktree
`/home/beat/claude/kesh/.claude/worktrees/kesh-s37-3a` (branche `story/37-3a-ecritures-en-devise-modele`). N'écris
nulle part ailleurs, sauf journaux dans `/home/beat/claude/kesh-gate-logs/s37-3a/review-p2/`.

## Périmètre

- Diff : `git diff epic/37-multi-devises...HEAD (la branche contient 37-1 et 37-2 terminées)` (code, tests, manuels `.tex`, i18n, docs).
- Spec : la fiche `_bmad-output/implementation-artifacts/37-3a-ecritures-en-devise-modele.md` (AC, FR/NFR, Dev Agent
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
sous ``/home/beat/claude/kesh-gate-logs/heavy.sh scripts/mem-guard.sh <commande>` (au plus deux exécutions lourdes sur la station ; backend lancé avec `7>&- 8>&- 9>&-`), base `kesh_s37_3a` (cf. `docs/agents-orchestration.md`) ; commit local
« fix(37-3a) : revue de code P2 — … » (aucun mot-clé de fermeture, aucune mention d'IA). Change Log de la fiche :
« 2026-11-16 — revue de code P2 (<modèle>) : <décompte> ». N'affirme que ce qui a tourné.

## Interdits

Aucun `git push`, aucune branche autre que la story, aucun redémarrage du conteneur MariaDB, aucun
`scripts/prepare-release.sh` ni script mutant hors gates.

## Rapport (français)

Findings `37-3a-CR-P2-N` (sévérité, fichier:ligne, constat, correctif appliqué) ; axes exercés / non exercés ; gates
exécutés (chiffres) ; décompte.

## Propre à la P2

- Relis d'abord le commit de la P1 (`c5ac0e46`) et son entrée au Change Log : 1 HIGH et 7 MEDIUM corrigés (tests par
  site d'`ACCOUNT_IN_FOREIGN_CURRENCY` et inventaire des 28 colonnes `*_account_id`, refus à la validation d'une facture
  fournisseur, CHF sans arrondi dans `FX_AMOUNT_MISMATCH`, message de `MULTI_CURRENCY_DISABLED`, test muet du cas (c),
  tests (e), (f), (h), (i) et HTTP). Cherche les **défauts introduits** (le refus ajouté sous les `FOR UPDATE` de
  `validate_in_tx` : ordre des verrous, interblocage, rejeu ; le nouveau message dans les routes de 37-1 et 37-2 ; les
  tests neufs rougissent-ils sans leur correctif ?), puis le reste de la story, en privilégiant ce que la P1 n'a pas
  exercé.
- Les sept LOW renvoyés par la P1 (points 14 à 20 de la liste) : corrige ceux qui touchent la production ou les tests de
  la story et tiennent en quelques lignes ; laisse le reste.
- **Aucune mention d'IA dans les commits** (pas de `Co-Authored-By: Claude`), consigne de Beat qui prime.
- Gates : gate complet si `kesh-db` est touché. Pas d'E2E ici (fait à la clôture).
