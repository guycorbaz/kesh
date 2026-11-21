# Prompt de revue de code P2 — story 37-4

Deuxième passe complète (Sonnet, modèle différent de la P1 Opus) de `bmad-code-review` (lis `.claude/skills/bmad-code-review/SKILL.md` et suis-le : couches
Blind Hunter, Edge Case Hunter, Acceptance Auditor, triage), contexte frais, sur la story 37-4, worktree
`<dépôt>/.claude/worktrees/kesh-s37-4` (branche `story/37-4-rapports-en-devise`). N'écris
nulle part ailleurs, sauf journaux dans `<station>/kesh-gate-logs/s37-4/review-p2/`.

## Périmètre

- Diff : `git diff story/37-3a-ecritures-en-devise-modele...HEAD` (code, tests, manuels `.tex`, i18n, docs).
- Spec : la fiche `_bmad-output/implementation-artifacts/37-4-rapports-en-devise.md` (AC, FR/NFR, Dev Agent
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
sous ``<station>/kesh-gate-logs/heavy.sh scripts/mem-guard.sh <commande>` (au plus deux exécutions lourdes sur la station ; backend lancé avec `7>&- 8>&- 9>&-`), base `kesh_s37_4` (cf. `docs/agents-orchestration.md`) ; commit local
« fix(37-4) : revue de code P2 — … » (aucun mot-clé de fermeture, aucune mention d'IA). Change Log de la fiche :
« 2026-11-16 — revue de code P2 (<modèle>) : <décompte> ». N'affirme que ce qui a tourné.

## Interdits

Aucun `git push`, aucune branche autre que la story, aucun redémarrage du conteneur MariaDB, aucun
`scripts/prepare-release.sh` ni script mutant hors gates.

## Rapport (français)

Findings `37-4-CR-P2-N` (sévérité, fichier:ligne, constat, correctif appliqué) ; axes exercés / non exercés ; gates
exécutés (chiffres) ; décompte.

## Propre à la P2

- Relis d'abord le commit de la P1 (`36cea077`) et son entrée au Change Log : 4 MEDIUM (ordre des `bind` de la balance
  sur une période en cours d'exercice, captures du résultat et du journal, `docs/api-external.md`, identifiants
  dédoublonnés dans `balances_as_of`) et 9 LOW corrigés (« -0.00 », troncature des noms au PDF, place réservée du grand
  livre, tests T3 et MCP). Cherche les **défauts introduits** par ces correctifs (le dédoublonnage change-t-il le
  contrat attendu par la 37-5 ? la troncature à 38 caractères casse-t-elle une fixture mono-devise ? les tests neufs
  rougissent-ils sans leur correctif ?), puis le reste de la story, en privilégiant ce que la P1 n'a pas exercé (rendu
  visuel des PDF : régénère et regarde au moins le grand livre et la balance d'un compte en devise).
- Les deux LOW non traités par la P1 (tests « CHF seulement » faibles ; « CHF » en dur au lieu de « devise
  principale ») : corrige-les s'ils tiennent en quelques lignes.
- **Aucune mention d'IA dans les commits** (pas de `Co-Authored-By: Claude`), consigne du responsable de l'epic qui prime. Aucun `GRANT`
  à la main.
- Gates : gate complet si `kesh-db` est touché ; Vitest complet et `npm run build` si le frontend est touché. Pas d'E2E
  complet ici (fait à la clôture).
