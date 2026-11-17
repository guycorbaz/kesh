# Prompt de revue de code P2 — story 37-3b

Deuxième passe complète (Sonnet, modèle différent de la P1 Opus) de `bmad-code-review` (lis `.claude/skills/bmad-code-review/SKILL.md` et suis-le : couches
Blind Hunter, Edge Case Hunter, Acceptance Auditor, triage), contexte frais, sur la story 37-3b, worktree
`/home/beat/claude/kesh/.claude/worktrees/kesh-s37-3b` (branche `story/37-3b-ecritures-en-devise-ecrans`). N'écris
nulle part ailleurs, sauf journaux dans `/home/beat/claude/kesh-gate-logs/s37-3b/review-p2/`.

## Périmètre

- Diff : `git diff story/37-3a-ecritures-en-devise-modele...HEAD` (code, tests, manuels `.tex`, i18n, docs).
- Spec : la fiche `_bmad-output/implementation-artifacts/37-3b-ecritures-en-devise-ecrans.md` (AC, FR/NFR, Dev Agent
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
sous ``/home/beat/claude/kesh-gate-logs/heavy.sh scripts/mem-guard.sh <commande>` (au plus deux exécutions lourdes sur la station ; backend lancé avec `7>&- 8>&- 9>&-`), base `kesh_s37_3b` (cf. `docs/agents-orchestration.md`) ; commit local
« fix(37-3b) : revue de code P2 — … » (aucun mot-clé de fermeture, aucune mention d'IA). Change Log de la fiche :
« 2026-11-16 — revue de code P2 (<modèle>) : <décompte> ». N'affirme que ce qui a tourné.

## Interdits

Aucun `git push`, aucune branche autre que la story, aucun redémarrage du conteneur MariaDB, aucun
`scripts/prepare-release.sh` ni script mutant hors gates.

## Rapport (français)

Findings `37-3b-CR-P2-N` (sévérité, fichier:ligne, constat, correctif appliqué) ; axes exercés / non exercés ; gates
exécutés (chiffres) ; décompte.

## Propre à la P2

- Relis d'abord le commit de la P1 (`9be0b53c`) et son entrée au Change Log : 3 HIGH (CHF envoyé à l'ancien cours,
  champ CHF réécrit à chaque frappe, montant normalisé jugé ambigu à la relecture) et 13 MEDIUM corrigés (frais d'une
  autre devise, `debounce.flush()` avant l'envoi, lookups attendus par l'assistant, échecs passagers non mémoïsés,
  refus par ligne décalés, frais au centime, lookup relancé au changement de compte, cours des lignes relues gardé,
  état d'erreur de la page Conversion, refus des grilles indexés par compte, trois tests renforcés). Cherche les
  **défauts introduits** par ces correctifs (le `flush` peut-il envoyer deux fois ou bloquer l'envoi ? `commitChfEntry`
  sur `onblur` : que se passe-t-il si on envoie sans quitter le champ, au clavier avec Entrée ? les refus indexés par
  compte avec deux lignes sur le même compte ? les tests neufs rougissent-ils sans leur correctif ?), puis le reste.
- **Corrige aussi**, parmi les LOW non traités de la P1 (`/home/beat/claude/kesh-gate-logs/s37-3b/review-p1/issues-a-ouvrir-p1.md`) :
  la ligne de différence au-delà de `DECIMAL(19,4)` qui rend 500 au lieu de 400 ; la conversion de même devise admise
  par le serveur (refus 400) ; le sélecteur qui offre des comptes en devise en mono-devise. Et ceux qui tiennent en
  quelques lignes. Mets à jour la liste de ce fichier (ce qui reste à ouvrir).
- **Aucune mention d'IA dans les commits**, aucun `GRANT`. Base `kesh-mariadb-dev` (3306) seulement.
- Gates : gate complet si `kesh-db` est touché ; Vitest complet (seul, pas en même temps que le gate backend) et
  `npm run build`. Pas d'E2E complet ici (fait à la clôture).
