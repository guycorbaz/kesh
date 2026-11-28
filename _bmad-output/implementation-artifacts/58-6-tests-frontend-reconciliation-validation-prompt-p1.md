# Validation P1 de la fiche 58-6 (Epic 65) — prompt

Tu fais la **passe P1 de validation** de la fiche `_bmad-output/implementation-artifacts/58-6-tests-frontend-reconciliation.md`, dans le worktree
`/home/gcorbaz/devel/kesh-58-6` (workflow `bmad-create-story`, mode *validate* : `.claude/skills/bmad-create-story/`).
Contexte frais : tu n'as pas écrit cette fiche. Lis `CLAUDE.md`, `docs/agents-orchestration.md` § 6 (lentilles),
`/home/gcorbaz/devel/kesh-gate-logs/e65/e65-commun.md` et `_bmad-output/planning-artifacts/epic-65-nettoyage-suite.md`.

## Interdits

Lecture seule : tu n'écris **que** ton rapport. Aucune modification de fichier du dépôt, aucun commit, aucun push,
aucune commande `gh` qui écrit, aucun `scripts/prepare-release.sh` ni script mutant, aucun build, test, `npm`,
`cargo`, `docker`. Lecture : fichiers, `git log`/`git show`/`git grep`, `gh issue view`.

## Axes (tous à exercer, et à déclarer exercés ou non)

1. **Prémisses** : chaque issue retenue existe-t-elle encore au code ? Chaque issue écartée l'est-elle à bon droit
   (preuve relue) ? Sites exacts (`fichier:ligne`) vérifiés par `grep -nF`.
2. **Correctif** : juste, complet, sans effet de bord non inventorié (inventaire des **sites non résolus** : tout
   autre usage du jeton, de la classe, du composant ou du test touché, résolu ou écrit comme angle mort).
3. **AC** : vérifiables, et chacun prouvé par un test qui **rougirait** sur une mutation nommée (un test vrai par
   construction ne prouve rien).
4. **Périmètre** : epic de nettoyage — aucune règle métier, migration ni changement d'API ; un vrai défaut sort.
5. **Fichiers communs** avec les deux autres stories de l'Epic 65 (catalogues i18n, `ReconciliationProposals.test.ts`,
   `AccountAutocomplete.test.ts`) : conflits prévus et traités ?
6. **Manuel** : vérifie `docs/manual/` contre la fiche et le code, **PDF compris**, aplati avant de greper
   (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`). Captures du manuel touchées ?
7. **Recette visuelle et gates** : conformes au CLAUDE.md et au fichier d'epic.
8. **Décomptes** de la fiche recomptés depuis la source.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/e65/58-6/58-6-validation-p1.md` : findings numérotés avec sévérité
(CRITICAL, HIGH, MEDIUM, LOW), chacun avec sa preuve (`grep -nF` copié ou extrait cité) et le correctif proposé ;
puis le décompte par sévérité ; puis la liste des **axes exercés et non exercés**. Un « 0 finding » sans cette
liste ne compte pas. Réponds par le décompte et les findings au-dessus de LOW en une ligne chacun.
