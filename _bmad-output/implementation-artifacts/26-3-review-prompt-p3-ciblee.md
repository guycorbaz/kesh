# Prompt — passe ciblée P3 de la revue de code de la Story 26-3 (Haiku, une lentille)

Une seule lentille, **chasseur de régression de la remédiation P2**, braquée sur **le seul commit**
`dfa821af` : `git -C /home/gcorbaz/devel/kesh-26-3 show dfa821af`. Contexte : rapports P2
`/home/gcorbaz/devel/kesh-gate-logs/26-3-review-p2-{B,E,A}.md` ; fiche
`_bmad-output/implementation-artifacts/26-3-manuel-et-restauration-synology.md` (Change Log, entrée « Revue de
code P2 remédiée »). Langue : français. ⚠️ Lis les fichiers **à HEAD** (`sed -n`, `grep -nF`) pour toute
affirmation de présence ou d'absence — ne te fie pas aux numéros de ligne d'un hunk.

## Interdits absolus

Lecture seule. N'exécute AUCUN script du dépôt (`scripts/prepare-release.sh`, `scripts/test-fast.sh`,
`scripts/mem-guard.sh`, `scripts/synology/*.sh`), ni `cargo`, `npm`, `npx`, `make`, `latexmk`, `docker`.
Aucun `git` qui écrit, aucune commande `gh` qui écrit. Permis : `git show`, `git diff`, `git grep`, `grep`,
`sed -n`, `cat`, `pdftotext … -` (sortie standard), `bash -n`. N'écris rien hors de ton rapport.

## Axes

1. **G16, clause de la branche « service »** (`crates/kesh-api/tests/configuration_transmise.rs`, cherche
   `ne refuse pas un état actif`) : la regex correspond-elle **exactement** à la ligne 107-109 de
   `scripts/synology/kesh-restore.sh` à HEAD ? Une écriture fautive de la branche service (refus retiré,
   état actif admis, comparaison au littéral `kesh-api`, compteur absent) passerait-elle ? Déborde-t-elle sur
   le filet ?
2. **G16, sortie sur zéro déplacée dans les étapes ordonnées** : l'ordre est-il juste (après `docker ps`,
   avant le verrou) ; la clause a-t-elle bien disparu de la liste non ordonnée (pas de double rouge) ?
3. **G14, `(?:=\S+)?`** : la regex lit-elle maintenant `logs --tail=100 kesh-api` (`admin-manual.tex`) ;
   crée-t-elle un faux positif ou un faux négatif ailleurs dans le manuel (`grep -nE "docker compose" `) ? La
   doc-comment dit-elle vrai ?
4. **Recette** (`scripts/synology/recette.sh`) : le contrôle S17 (« Cannot connect to the Docker daemon »)
   correspond-il au message du faux `docker` ; les motifs de la section 4 réelle (`(service « kesh-api ») est
   « … »`) correspondent-ils au libellé de `kesh-restore.sh` ; prérequis du mode 0 aux trois sites
   (script, `docs/testing.md`, fiche) identiques ?
5. **Manuel utilisateur** (`docs/manual/fr/user-manual.tex`, phrase de la réconciliation) et son **PDF
   aplati** (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`) : vrai contre
   `crates/kesh-i18n/locales/fr-CH/messages.ftl` et `frontend/src/lib/features/reconciliation/` ?
6. **CHANGELOG** : une seule section `## [0.13.1] — Non publié` ; la retouche est-elle juste ?

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/26-3-review-p3-ciblee.md` : findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), fichier:ligne, **sortie `grep -nF`/`sed -n` copiée** comme preuve ;
**liste des axes exercés ET non exercés** (un « 0 » sans cette liste ne compte pas) ; bilan chiffré.
