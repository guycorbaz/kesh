# Prompt de revue de code P1 — story 37-3a

Première passe complète de `bmad-code-review` (lis `.claude/skills/bmad-code-review/SKILL.md` et suis-le : couches
Blind Hunter, Edge Case Hunter, Acceptance Auditor, triage), contexte frais, sur la story 37-3a, worktree
`<worktree>/kesh-s37-3a` (branche `story/37-3a-ecritures-en-devise-modele`). N'écris
nulle part ailleurs, sauf journaux dans `$GATE_LOGS/s37-3a/review-p1/`.

## Périmètre

- Diff : `git diff epic/37-multi-devises...HEAD (la branche contient 37-1 et 37-2 terminées, fusionnées en `e89291e6`)` (code, tests, manuels `.tex`, i18n, docs).
- Spec : la fiche `_bmad-output/implementation-artifacts/37-3a-ecritures-en-devise-modele.md` (AC, FR/NFR, Dev Agent
  Record et écarts de la fiche), l'epic `_bmad-output/planning-artifacts/epic-37-multi-devises.md`.
- Écarts du dev : voir le Dev Agent Record de la fiche (fixture sans pas d'ajustement, page MCP 18/10,
  `refuse_foreign_currency_accounts_in_tx` sans verrou, ordre des refus au `PUT`, message de `MULTI_CURRENCY_DISABLED`,
  tests de la fiche non écrits, concurrence argumentée sans test). Tous sont à juger.

## Axes (déclare ceux exercés et non exercés)

Correction des AC ; comptabilité (aucune écriture fausse, gardes des comptes de différence) ; verrous, ordre, rejeu et
interblocages ; migration P1–P8 (non breaking, audit d'idempotence,
P6, P7, squash de test) ; sauvegarde, restauration 0.17.6, CSV, remise à zéro ; RBAC (Administrateur en session, jamais
clé d'API) ; audit ; MCP (tailles) ; i18n 4 locales ; **manuels** (`.tex` contre le code ; PDF non commité) ; tests
(muets, manquants, mutation) ; DRY ; sécurité ; sites non résolus.

## Phase 2

Applique les patches des findings (CRITICAL à LOW selon le skill), avec leurs tests ; gate ciblé (fmt, clippy workspace,
nextest des binaires touchés — **gate complet si un patch touche `kesh-db`** : migrations, repository, `post_restore`),
sous ``$GATE_LOGS/heavy.sh scripts/mem-guard.sh <commande>` (au plus deux exécutions lourdes sur la station ; backend lancé avec `7>&- 8>&- 9>&-`), base `kesh_s37_3a` (cf. `docs/agents-orchestration.md`) ; commit local
« fix(37-3a) : revue de code P1 — … » (aucun mot-clé de fermeture, aucune mention d'IA). Change Log de la fiche :
« 2026-11-16 — revue de code P1 (<modèle>) : <décompte> ». N'affirme que ce qui a tourné.

## Interdits

Aucun `git push`, aucune branche autre que la story, aucun redémarrage du conteneur MariaDB, aucun
`scripts/prepare-release.sh` ni script mutant hors gates.

## Rapport (français)

Findings `37-3a-CR-P1-N` (sévérité, fichier:ligne, constat, correctif appliqué) ; axes exercés / non exercés ; gates
exécutés (chiffres) ; décompte.


## Propre à la 37-3a — points à examiner en priorité

1. **Comptabilité d'abord** : une écriture en devise ne doit jamais produire un CHF faux. Contrôle central
   (`crates/kesh-db/src/repositories/journal_entry_fx.rs`), produit exact, `rate_from_chf` et sa boucle d'ajustement,
   A17, A31, A32 (cours déduit qui retombe sur le cours AFC), CR-6, provenance du cours, contre-passation qui recopie les
   colonnes, refus `ENTRY_IS_FX_REVALUATION`.
2. **Tous les chemins d'écriture automatiques** (factures, règlements, soldes, rapprochement, lots, import, restauration,
   seed) : aucun ne doit écrire une ligne en devise incohérente ni contourner le contrôle. Inventorie les sites non
   résolus (CLAUDE.md).
3. **Tests que la fiche exige et qui manquent** (cas e, h, CR-6/U2/(f) à l'API, variante d'ouverture) : un test exigé
   et absent est un finding ; écris-les. Concurrence sans test de course : juge si un test est nécessaire.
4. **`refuse_foreign_currency_accounts_in_tx` sans verrou** : la fenêtre de course est-elle réelle (devise posée sur
   un compte entre le contrôle et l'écriture) ? Le contrôle central verrouille-t-il bien dans tous les cas ?
5. Migration P1–P8 (non breaking, squash, audit, absences admises, rejeu des sauvegardes 0.17.6), exports CSV
   (4 et 10 décimales), MCP (bornes 18/10), i18n des codes neufs.
