# Prompt de revue de code P1 — story 37-4

Première passe complète de `bmad-code-review` (lis `.claude/skills/bmad-code-review/SKILL.md` et suis-le : couches
Blind Hunter, Edge Case Hunter, Acceptance Auditor, triage), contexte frais, sur la story 37-4, worktree
`<worktree>/kesh-s37-4` (branche `story/37-4-rapports-en-devise`). N'écris
nulle part ailleurs, sauf journaux dans `$GATE_LOGS/s37-4/review-p1/`.

## Périmètre

- Diff : `git diff story/37-3a-ecritures-en-devise-modele...HEAD` (la branche part de la 37-3a et a fusionné `epic/37-multi-devises` ; le périmètre propre à la 37-4 est ce diff ; code, tests, manuels `.tex`, i18n, docs).
- Spec : la fiche `_bmad-output/implementation-artifacts/37-4-rapports-en-devise.md` (AC, FR/NFR, Dev Agent
  Record et écarts de la fiche), l'epic `_bmad-output/planning-artifacts/epic-37-multi-devises.md`.
- Écarts du dev : Dev Agent Record de la fiche et registre `e37-choix-autonomes-37-4.md` (C-37-4-1 à 9) ; bornes MCP
  réduites (`kesh_grand_livre` 26/14, `kesh_balance` 42/23) ; pas de capture du compte de résultat ni du journal en
  devise. Tous sont à juger.

## Axes (déclare ceux exercés et non exercés)

Correction des AC ; comptabilité (aucune écriture fausse, gardes des comptes de différence) ; verrous, ordre, rejeu et
interblocages ; migration P1–P8 (non breaking, audit d'idempotence,
P6, P7, squash de test) ; sauvegarde, restauration 0.17.6, CSV, remise à zéro ; RBAC (Administrateur en session, jamais
clé d'API) ; audit ; MCP (tailles) ; i18n 4 locales ; **manuels** (`.tex` contre le code ; PDF non commité) ; tests
(muets, manquants, mutation) ; DRY ; sécurité ; sites non résolus.

## Phase 2

Applique les patches des findings (CRITICAL à LOW selon le skill), avec leurs tests ; gate ciblé (fmt, clippy workspace,
nextest des binaires touchés — **gate complet si un patch touche `kesh-db`** : migrations, repository, `post_restore`),
sous ``$GATE_LOGS/heavy.sh scripts/mem-guard.sh <commande>` (au plus deux exécutions lourdes sur la station ; backend lancé avec `7>&- 8>&- 9>&-`), base `kesh_s37_4` (cf. `docs/agents-orchestration.md`) ; commit local
« fix(37-4) : revue de code P1 — … » (aucun mot-clé de fermeture, aucune mention d'IA). Change Log de la fiche :
« 2026-11-16 — revue de code P1 (<modèle>) : <décompte> ». N'affirme que ce qui a tourné.

## Interdits

Aucun `git push`, aucune branche autre que la story, aucun redémarrage du conteneur MariaDB, aucun
`scripts/prepare-release.sh` ni script mutant hors gates.

## Rapport (français)

Findings `37-4-CR-P1-N` (sévérité, fichier:ligne, constat, correctif appliqué) ; axes exercés / non exercés ; gates
exécutés (chiffres) ; décompte.



## Propre à la 37-4 — points à examiner en priorité

1. **Les chiffres** : aucun rapport ne doit donner un total CHF ou en devise faux. Somme en devise unique
   (`kesh-db/src/repositories/account_balances.rs`), `balances_as_of` (bornes, tous exercices, filtre société, connexion
   fournie), solde progressif et remises à zéro, report « À reporter » / « Report » du PDF, compte à CHF 0 non nul en
   devise, totaux de la balance et du bilan, sens débit/crédit.
2. **Non-régression mono-devise** : les fixtures du commit `88955a09` doivent rester identiques ; vérifie qu'elles sont
   bien comparées (test muet ?).
3. **Inventaire AC-7** (tous les `SUM` de montants du dépôt) : aucun site oublié qui mélangerait CHF et devise.
4. MCP (bornes réduites, champs à `null`, `kesh_postes_ouverts` en CHF), CSV (colonnes conditionnelles), i18n, manuels
   `.tex`, écran (`AccountCurrencyBalance`).
