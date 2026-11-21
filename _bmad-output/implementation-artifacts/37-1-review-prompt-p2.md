# Prompt de revue de code P2 — story 37-1

Deuxième passe complète (modèle différent de P1, Opus ; la P1 Sonnet a rendu 0 au-dessus de LOW sur un diff de 134 fichiers : un « 0 » se vérifie) de `bmad-code-review` (lis `.claude/skills/bmad-code-review/SKILL.md` et suis-le : couches
Blind Hunter, Edge Case Hunter, Acceptance Auditor, triage), contexte frais, sur la story 37-1, worktree
`<dépôt>/.claude/worktrees/kesh-s37-1` (branche `story/37-1-reglage-et-devise-des-comptes`). N'écris
nulle part ailleurs, sauf journaux dans `<station>/kesh-gate-logs/s37-1/review-p2/`.

## Périmètre

- Diff : `git diff epic/37-multi-devises...HEAD` (code, tests, manuels `.tex`, i18n, docs).
- Spec : la fiche `_bmad-output/implementation-artifacts/37-1-reglage-et-devise-des-comptes.md` (AC, FR/NFR, Dev Agent
  Record et écarts C-37-1-1 à 7), l'epic `_bmad-output/planning-artifacts/epic-37-multi-devises.md`.
- Décisions de l'orchestrateur sur les écarts : **collation `ascii_general_ci` acceptée** (les `CHECK` gardent
  `BINARY`) ; **numéros 6840/1090 non cités au manuel** (KISS) ; les autres écarts sont à juger.

## Axes (déclare ceux exercés et non exercés)

Correction des AC ; comptabilité (aucune écriture fausse, gardes des comptes de différence) ; verrous, ordre, rejeu et
interblocages (y compris `archive_account` signalé par le dev) ; migration P1–P8 (non breaking, audit d'idempotence,
P6, P7, squash de test) ; sauvegarde, restauration 0.17.6, CSV, remise à zéro ; RBAC (Administrateur en session, jamais
clé d'API) ; audit ; MCP (tailles) ; i18n 4 locales ; **manuels** (`.tex` contre le code ; PDF non commité) ; tests
(muets, manquants, mutation) ; DRY ; sécurité ; sites non résolus.

## Phase 2

Applique les patches des findings (CRITICAL à LOW selon le skill), avec leurs tests ; gate ciblé (fmt, clippy workspace,
nextest des binaires touchés — **gate complet si un patch touche `kesh-db`** : migrations, repository, `post_restore`),
sous `scripts/mem-guard.sh` et le verrou `flock`, base `kesh_s37_1` (cf. `docs/agents-orchestration.md`) ; commit local
« fix(37-1) : revue de code P2 — … » (aucun mot-clé de fermeture, aucune mention d'IA). Change Log de la fiche :
« 2026-11-16 — revue de code P2 (<modèle>) : <décompte> ». N'affirme que ce qui a tourné.

## Interdits

Aucun `git push`, aucune branche autre que la story, aucun redémarrage du conteneur MariaDB, aucun
`scripts/prepare-release.sh` ni script mutant hors gates.

## Rapport (français)

Findings `37-1-CR-P2-N` (sévérité, fichier:ligne, constat, correctif appliqué) ; axes exercés / non exercés ; gates
exécutés (chiffres) ; décompte.

## Propre à la P2

- Relis aussi le commit de la P1 (`8e9de8f4`, écho borné de `CURRENCY_UNKNOWN`) et l'entrée P1 du Change Log ;
  privilégie les axes que la P1 déclare **non exercés** (tests de course et victimes de rejeu, mutations ciblées sur les
  gardes des comptes de différence, captures `kesh-gate-logs/s37-1/captures/`).
- **Corrige 37-1-CR-P1-3** (LOW) : une finalisation déjà faite ignore le corps (AC-11) — lire l'étape avant de valider les
  dates ; test.
- 37-1-CR-P1-2 (`archive_account` non rejoué) est tracé en issue par l'orchestrateur : ne le corrige pas, sauf si tu
  trouves un cycle concret (alors MEDIUM, corrige).
