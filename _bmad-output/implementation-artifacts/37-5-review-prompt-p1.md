# Prompt de revue de code P1 — story 37-5

Première passe complète de `bmad-code-review` (lis `.claude/skills/bmad-code-review/SKILL.md` et suis-le : couches
Blind Hunter, Edge Case Hunter, Acceptance Auditor, triage), contexte frais, sur la story 37-5, worktree
`/home/beat/claude/kesh/.claude/worktrees/kesh-s37-5` (branche `story/37-5-reevaluation-fin-exercice`). N'écris
nulle part ailleurs, sauf journaux dans `/home/beat/claude/kesh-gate-logs/s37-5/review-p1/`.

## Périmètre

- Diff : `git diff epic/37-multi-devises...HEAD` (la branche contient tout l'Epic 37 terminé, fusionné en `d818fad6` ; code, tests, manuels `.tex`, i18n, docs).
- Spec : la fiche `_bmad-output/implementation-artifacts/37-5-reevaluation-fin-exercice.md` (AC, FR/NFR, Dev Agent
  Record et écarts de la fiche), l'epic `_bmad-output/planning-artifacts/epic-37-multi-devises.md`.
- Écarts du dev : Dev Agent Record de la fiche (C-37-5-1 à 6 : champ `defaultKind` ajouté à l'aperçu, `blockers[]`
  un par compte et passation au premier, atomicité prouvée par déclencheur SQL dans la base de test, E2E de K9 sur le
  même exercice). Tous sont à juger.

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
« fix(37-5) : revue de code P1 — … » (aucun mot-clé de fermeture, aucune mention d'IA). Change Log de la fiche :
« 2026-11-16 — revue de code P1 (<modèle>) : <décompte> ». N'affirme que ce qui a tourné.

## Interdits

Aucun `git push`, aucune branche autre que la story, aucun redémarrage du conteneur MariaDB, aucun
`scripts/prepare-release.sh` ni script mutant hors gates.

## Rapport (français)

Findings `37-5-CR-P1-N` (sévérité, fichier:ligne, constat, correctif appliqué) ; axes exercés / non exercés ; gates
exécutés (chiffres) ; décompte.




## Propre à la 37-5 — points à examiner en priorité

1. **Comptabilité d'abord** : la réévaluation ne doit jamais passer une écriture fausse. Écart en débit net, produit en
   `i128`, arrondi, choix du cours (`resolve_default_rates` : journalier, repli mensuel, cours saisi, genre de clôture),
   comptes de différence (société ou propres au compte), filtre des comptes de **bilan** seulement
   (`balances_as_of(None)` rend aussi les comptes de résultat et les archivés), cumul depuis l'origine (A39 : pas
   d'écriture de report), une écriture `OD` par compte, réévaluations précédentes, contre-passation refusée.
2. **Clôture et K9** : `fiscal_years::close` réorganisé (lectures avant `begin`, `lock_with_earlier_in_tx` partagé) —
   aucun changement de comportement des contrôles existants ; ordre des verrous `companies` → exercices antérieurs →
   exercice → comptes contre tous les autres gestes (cycles, rejeu) ; 409 `FOREIGN_ACCOUNTS_NOT_REVALUED` sans
   contournement possible (API, MCP, clé d'API) ; K9 `ok` quand rien n'est à réévaluer.
3. **« huit » → « neuf »** partout (grep des deux graphies, des chiffres, des quatre locales et des manuels).
4. **La fusion de `fx_accounts_for_settings`** avec la 37-3b (une seule règle dans `fx_accounts_for_columns`) : les
   deux usages gardent-ils leur sens ?
5. Écran (`RevaluationPanel.svelte`, aperçu, « Clôturer » désactivé), captures, manuel `.tex`, brochure, README, guide
   d'API.
6. Base : `kesh-mariadb-dev3`, port 3308. **Aucune mention d'IA dans les commits**, aucun `GRANT`.
