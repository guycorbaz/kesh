# Prompt de revue de code P2 — story 37-2

Deuxième passe complète (Sonnet, modèle différent de la P1 Opus) de `bmad-code-review` (lis `.claude/skills/bmad-code-review/SKILL.md` et suis-le : couches
Blind Hunter, Edge Case Hunter, Acceptance Auditor, triage), contexte frais, sur la story 37-2, worktree
`/home/beat/claude/kesh/.claude/worktrees/kesh-s37-2` (branche `story/37-2-cours-mensuels-afc`). N'écris
nulle part ailleurs, sauf journaux dans `/home/beat/claude/kesh-gate-logs/s37-2/review-p2/`.

## Périmètre

- Diff : `git diff epic/37-multi-devises...HEAD` (code, tests, manuels `.tex`, i18n, docs).
- Spec : la fiche `_bmad-output/implementation-artifacts/37-2-cours-mensuels-afc.md` (AC, FR/NFR, Dev Agent
  Record et écarts C-37-2-1 à 3), l'epic `_bmad-output/planning-artifacts/epic-37-multi-devises.md`.
- Écarts du dev (Dev Agent Record) : C-37-2-1 collation `ascii_general_ci` (acceptée, comme en 37-1) ; C-37-2-2
  `RateKind` déplacé dans kesh-core ; C-37-2-3 copie par `INSERT` ligne à ligne au lieu de `INSERT … SELECT` ; pas de
  `chrono-tz` (horloge de l'Epic 35 réutilisée). C-37-2-2 et C-37-2-3 sont à juger.

## Axes (déclare ceux exercés et non exercés)

Correction des AC ; comptabilité (aucune écriture fausse, gardes des comptes de différence) ; verrous, ordre, rejeu et
interblocages ; migration P1–P8 (non breaking, audit d'idempotence,
P6, P7, squash de test) ; sauvegarde, restauration 0.17.6, CSV, remise à zéro ; RBAC (Administrateur en session, jamais
clé d'API) ; audit ; MCP (tailles) ; i18n 4 locales ; **manuels** (`.tex` contre le code ; PDF non commité) ; tests
(muets, manquants, mutation) ; DRY ; sécurité ; sites non résolus.

## Phase 2

Applique les patches des findings (CRITICAL à LOW selon le skill), avec leurs tests ; gate ciblé (fmt, clippy workspace,
nextest des binaires touchés — **gate complet si un patch touche `kesh-db`** : migrations, repository, `post_restore`),
sous ``/home/beat/claude/kesh-gate-logs/heavy.sh scripts/mem-guard.sh <commande>` (au plus deux exécutions lourdes sur la station ; backend lancé avec `7>&- 8>&- 9>&-`), base `kesh_s37_2` (cf. `docs/agents-orchestration.md`) ; commit local
« fix(37-2) : revue de code P2 — … » (aucun mot-clé de fermeture, aucune mention d'IA). Change Log de la fiche :
« 2026-11-16 — revue de code P2 (<modèle>) : <décompte> ». N'affirme que ce qui a tourné.

## Interdits

Aucun `git push`, aucune branche autre que la story, aucun redémarrage du conteneur MariaDB, aucun
`scripts/prepare-release.sh` ni script mutant hors gates.

## Rapport (français)

Findings `37-2-CR-P2-N` (sévérité, fichier:ligne, constat, correctif appliqué) ; axes exercés / non exercés ; gates
exécutés (chiffres) ; décompte.

## Propre à la P2

- Relis le commit de la P1 (`c691bd4b`) et l'entrée P1 du Change Log : les huit MEDIUM corrigés (↑ sans valeur publiée,
  écriture de cours pendant une restauration — `garde_hors_restauration`, copie verrouillée `LOCK IN SHARE MODE`, plafond
  qui garde le mois courant, trois défauts de l'écran, manuels). Cherche d'abord les **défauts introduits** par ces
  correctifs (ordre des verrous `backup_lock` / `rate_fetch_lock` / tables, cycles, tests qui ne rougiraient pas sans le
  correctif), puis le reste de la story, en privilégiant les axes non exercés par la P1 (mutation des tests ajoutés,
  CSV, remise à zéro de démonstration).
- **Tarir la source** (CLAUDE.md § *Priorités des défauts*) : la P1 a laissé **33 LOW** à ouvrir en issues (liste au
  Change Log). **Corrige ceux qui touchent la production ou les tests de la story et tiennent en quelques lignes**
  (p. ex. textes bruts, `legend`, no-op, `copied` par société, horloge injectée, attente de 300 ms, montages dupliqués,
  test de la remise à zéro, manuel de l'acquittement). Ne laisse en issue que ce qui sort du périmètre ou demande une
  conception (p. ex. coupe-circuit, limite de fréquence) ; écris la liste finale au Change Log.
- RBAC : l'intention de la fiche (NFR-37-2-7 : Administrateur et Comptable, clé d'API admise et auditée) fait foi.
- Gates : gate complet si `kesh-db` est touché ; **Vitest complet et `npm run build`** si le frontend est touché. L'E2E
  complet se fera à la clôture, pas ici.
