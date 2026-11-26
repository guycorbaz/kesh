# Validation P2 — fiche 53-1 « Une écriture ne passe pas deux fois » (#1355, #1365, P1)

**Modèle : Opus. Passe complète, contexte frais.** (Alternance du `CLAUDE.md` § *Review Iteration Rule* : la P1 était Sonnet ; une passe complète, non ciblée, parce que la remédiation de la P1 a réécrit le mécanisme de sérialisation, D3.)

Tu es une lentille de validation adversariale de la fiche
`/home/gcorbaz/devel/kesh-53-1/_bmad-output/implementation-artifacts/53-1-ecriture-pas-deux-fois.md`
(worktree `/home/gcorbaz/devel/kesh-53-1`, branche `story/53-1-ecriture-unique`). Kesh est un logiciel de comptabilité suisse **en production** : une écriture passée deux fois est un défaut P1. Tu cherches ce qui ferait échouer le développement ou laisserait passer une seconde écriture — non à reformuler la fiche.

## À lire d'abord

- `/home/gcorbaz/devel/kesh-53-1/CLAUDE.md` (fait foi) : § *Règle de splitting préventif*, § *Pattern batch*, § *Qualité du code*, § *Migration breaking policy*, § *Recette visuelle*, § *Inventorier les sites NON RÉSOLUS*, § *Versions de l'API*.
- `docs/agents-orchestration.md` § 2 et § 3.
- Le fichier d'epic `_bmad-output/planning-artifacts/epic-53-ecritures-justes.md`.
- Les fiches de référence `48-12a-doublons-fournisseurs-serveur.md` et `48-12b-doublons-fournisseurs-ecran.md` (même dossier).
- **Le rapport de la P1** : `/home/gcorbaz/devel/kesh-gate-logs/e531/validation-p1.md`, et l'entrée « Validation P1 remédiée » du Change Log de la fiche. Les décisions du Project Lead du 2026-11-25 (Q1 à Q7) sont tranchées : ne les rediscute pas, vérifie qu'elles sont appliquées partout.

## Axes à exercer (tous)

1. **Chaque finding de la P1 réellement remédié.** Pour V1-1 à V1-14, dis : remédié, partiellement, ou non — en citant la ligne de la fiche **et** la ligne de code qui le prouvent. Une remédiation qui crée un défaut neuf est un finding.
2. **Le mécanisme de D3, au code.** Est-il vrai que, sur le chemin de la saisie après le déplacement prescrit, **aucune** lecture ordinaire ne précède le verrou de l'exercice (`create` → `create_in_tx_inner`, étape 0 `projects::validate_taggable_in_tx`, toute autre lecture) ? Et sur la conversion, après la lecture des exercices sur le pool et le pré-verrou (lis `create_conversion_in_tx` et `complete_entry`, et tout ce qu'ils appellent avant la boucle) ? Est-il vrai que **tout** geste qui fait naître, disparaître ou change une donnée comparée d'une écriture tient l'exercice jusqu'à son `COMMIT` (création, modification, suppression, contre-passation — origine et miroir —, et tout autre `UPDATE`/`DELETE` de `journal_entries` ou `journal_entry_lines` : cherche-les par le symptôme) ? Le déplacement de la lecture de la borne change-t-il un comportement observable d'un flux automatique ? Le pré-verrou de la conversion change-t-il la précédence publiée (`docs/api-external.md`, conversions) ou crée-t-il un cycle non défendu par le rejeu ?
3. **Prémisses au code.** Relis chaque citation `fichier:ligne` neuve ou modifiée par la remédiation ; une citation fausse est un finding.
4. **Inventaire clos.** Refais le relevé par le symptôme (y compris les `INSERT` à nom de table dynamique) et les appelants de `create_in_tx_inner` et de `create_conversion_in_tx` ; chaque site est-il résolu ou écrit en angle mort ?
5. **Contrat et compatibilité (D4, D5, D8).** Recompte les relevés de D4 (dépôt, HTTP, Playwright) avec leurs commandes ; vérifie `saisir_par` et les utilitaires cités ; `confirmNotDuplicate` dans `entries[i]`, au `PUT`, mal typé sur les deux extracteurs ; l'annonce ⚠️ et la condition sur `b`.
6. **Écran (D9, D10).** Tables J et K complètes (toute entrée, toute sortie, « Annuler », Ctrl+S, `fetchMissingMonth`, l'assistant TVA, le retour de la page de conversion) ; relevé C clos ; corps gardé ; clés i18n nommées et cohérentes entre D9, D10, AC-15 et F4.
7. **Tests et mutations.** Chaque AC a-t-il un test qui rougit sans le correctif ? Chaque mutation (M1 à M24, avec M2b, M12b, M13b) est-elle tuable par le test annoncé ? AC-7 et AC-8b sont-ils réalisables tels qu'écrits (attente prouvée sans `INNODB_TRX`) ? Un test est-il vrai par construction ?
8. **Découpage et ligne de coupe.** Décompte des modules ; la coupe 53-1a/53-1b tient-elle après la remédiation ?
9. **Migration.** « Aucune migration » : vérifie.
10. **Manuel, PDF compris.** Vérifie le manuel (`docs/manual/`) contre le code et contre la fiche : sections citées (`user-manual.tex:578`, `:760`, `:1333`, `admin-manual.tex:2680`) ; lis aussi les PDF, aplatis avant de greper : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (et `admin-manual.pdf`), **vers ta seule sortie standard** ; `docs/api-external.md`.
11. **Cohérence interne.** AC ↔ décisions ↔ tâches ↔ tests ↔ Note de version ↔ Change Log ; numéros, codes, clés identiques partout ; recompte tout décompte écrit dans la fiche.

## Interdits

- **N'écris aucun fichier, nulle part**, et n'exécute **aucune commande qui écrit** : pas d'`Edit`/`Write`, **aucune redirection `>` ni `>>`, pas de `tee`**, pas de `git commit`, `git checkout`, `git switch`, `git reset`, `git stash`, `git restore`, `git push`, pas de `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`, `scripts/mem-guard.sh` ni aucun script du dépôt, pas de `cargo` (ni `build`, ni `test`, ni `nextest`), pas de `npm`, pas de `python3`, pas de `docker` ni `docker compose`, pas de `sqlx`, pas de `mariadb`/`mysql`, pas de `curl`, pas de `make`.
- **Jamais `pkill`, `killall` ni `kill`** (ni par motif, ni par PID).
- Aucune commande `gh` qui écrit ; `gh` en lecture au plus, une minute au moins entre deux requêtes.
- Commandes permises : `git grep`, `git log`, `git show`, `git diff`, `grep` (y compris `grep -r` : sous ce dépôt, le pathspec `git grep … -- 'crates/*/tests'` ne désigne rien), `sed -n`, `awk` (lecture), `cat`, `head`, `tail`, `wc`, `ls`, `pdftotext … -` vers la sortie standard.

## Rapport attendu

En français. Pour chaque finding : identifiant `V2-n`, sévérité (CRITICAL, HIGH, MEDIUM, LOW), section de la fiche, constat **avec la ligne de code ou de fiche lue citée**, correctif proposé. Puis le tableau du sort de V1-1 à V1-14 (axe 1). Puis le décompte par sévérité, et **la liste des axes exercés et non exercés** (un axe non exercé dit pourquoi). Un « 0 finding » sans cette liste ne compte pas comme passe. L'orchestrateur range le rapport sous `/home/gcorbaz/devel/kesh-gate-logs/e531/validation-p2.md`.
