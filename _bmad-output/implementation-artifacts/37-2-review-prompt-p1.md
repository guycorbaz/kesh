# Prompt de revue de code P1 — story 37-2

Première passe complète de `bmad-code-review` (lis `.claude/skills/bmad-code-review/SKILL.md` et suis-le : couches
Blind Hunter, Edge Case Hunter, Acceptance Auditor, triage), contexte frais, sur la story 37-2, worktree
`<worktree>/kesh-s37-2` (branche `story/37-2-cours-mensuels-afc`). N'écris
nulle part ailleurs, sauf journaux dans `$GATE_LOGS/s37-2/review-p1/`.

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
sous ``$GATE_LOGS/heavy.sh scripts/mem-guard.sh <commande>` (au plus deux exécutions lourdes sur la station ; backend lancé avec `7>&- 8>&- 9>&-`), base `kesh_s37_2` (cf. `docs/agents-orchestration.md`) ; commit local
« fix(37-2) : revue de code P1 — … » (aucun mot-clé de fermeture, aucune mention d'IA). Change Log de la fiche :
« 2026-11-16 — revue de code P1 (<modèle>) : <décompte> ». N'affirme que ce qui a tourné.

## Interdits

Aucun `git push`, aucune branche autre que la story, aucun redémarrage du conteneur MariaDB, aucun
`scripts/prepare-release.sh` ni script mutant hors gates.

## Rapport (français)

Findings `37-2-CR-P1-N` (sévérité, fichier:ligne, constat, correctif appliqué) ; axes exercés / non exercés ; gates
exécutés (chiffres) ; décompte.

## Propre à la 37-2 — points à examiner en priorité

1. **Interblocage 1213 à l'import d'une sauvegarde** (issue n° 11 de la liste) : vu pendant l'E2E
   `backup-restore-new-install` sur 37-1 et 37-2, **jamais sur `19bf12fb` (avant l'Epic 37)**. Il vient donc
   probablement de l'Epic 37 : verrous ajoutés par 37-1 (`companies` en partage, comptes, réglages), copie des cours de
   37-2, tâche automatique ou déclenchement à la connexion qui tournerait pendant l'import. Diagnostique-le (journaux
   `$GATE_LOGS/s37-2/final/e2e-backend.log` et `s37-1/final3/`) ; si la cause est dans le code de
   l'epic, c'est au moins un MEDIUM : corrige-le, avec un test.
2. **C-37-8, option « toutes les sociétés » du ↑**, spécifiée et non livrée (issue n° 8) : juge si la fiche l'exige
   (AC manquant = finding) ou si c'est hors périmètre.
3. **Source hostile** : client HTTP borné (sans `Content-Length`, la lecture par morceaux est la seule borne), filtrage
   XML (entités, profondeur, taille), aucune redirection, délai. Route `_test/…` inaccessible hors mode test.
4. **Tâche automatique** : concurrence avec un import ou une remise à zéro, rejeu, plusieurs sociétés, horloge de
   Zurich.
5. La définition « moyenne du 25 au 24, publiée le 25 » (R4) attend la confirmation de Beat : ne la tranche pas, note
   seulement si le code en dépend.
