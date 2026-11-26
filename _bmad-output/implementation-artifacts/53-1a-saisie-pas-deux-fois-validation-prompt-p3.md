# Validation P3 — fiche 53-1a « La saisie ne passe pas deux fois » (#1355, P1)

**Modèle : Sonnet. Passe complète, contexte frais.** (Alternance du `CLAUDE.md` § *Review Iteration Rule* : la P2 était Opus ; passe complète, non ciblée, parce que la fiche a été découpée et réécrite.)

Tu es une lentille de validation adversariale de la fiche
`/home/gcorbaz/devel/kesh-53-1/_bmad-output/implementation-artifacts/53-1a-saisie-pas-deux-fois.md`
(worktree `/home/gcorbaz/devel/kesh-53-1`, branche `story/53-1-ecriture-unique`). Kesh est un logiciel de comptabilité suisse **en production** : une écriture passée deux fois est un défaut P1. La 53-1 a été **découpée par le Project Lead le 2026-11-25** (signal de recyclage V2-1) : la 53-1a porte la **saisie** (#1355) ; la conversion (#1365) est dans `53-1b-conversion-pas-deux-fois.md`, **hors de ta passe** sauf pour vérifier la frontière. Tu cherches ce qui ferait échouer le développement ou laisserait passer une seconde écriture — non à reformuler la fiche.

## À lire d'abord

- `/home/gcorbaz/devel/kesh-53-1/CLAUDE.md` (fait foi) : § *Règle de splitting préventif*, § *Qualité du code*, § *Migration breaking policy*, § *Recette visuelle*, § *Inventorier les sites NON RÉSOLUS*, § *Versions de l'API*.
- `docs/agents-orchestration.md` § 2 et § 3 ; le fichier d'epic `_bmad-output/planning-artifacts/epic-53-ecritures-justes.md`.
- Les fiches de référence `48-12a-doublons-fournisseurs-serveur.md` et `48-12b-doublons-fournisseurs-ecran.md`.
- **Les rapports des P1 et P2** : `/home/gcorbaz/devel/kesh-gate-logs/e531/validation-p1.md`, `/home/gcorbaz/devel/kesh-gate-logs/e531/validation-p2.md`, et le Change Log de la fiche. Les décisions du Project Lead du 2026-11-25 (Q1 à Q6 ; Q7 pour la 53-1b) sont tranchées : ne les rediscute pas, vérifie qu'elles sont appliquées.

## Axes à exercer (tous)

1. **Chaque finding des P1 et P2 qui concerne la saisie, réellement remédié.** Pour V1-1 à V1-14 et V2-2, V2-3, V2-6, V2-7, V2-9, V2-10 (part saisie), dis : remédié, partiellement, non, ou « parti en 53-1b » — en citant la ligne de la fiche **et** la ligne de code qui le prouvent. Une remédiation qui crée un défaut neuf est un finding.
2. **Plus aucune trace de la conversion hors du module commun.** Toute mention de la conversion dans la 53-1a doit être : un renvoi à la 53-1b, ou une pièce commune marquée « (prévu pour la 53-1b) » (module de la politique, entrée `create_in_tx_with_policy`, erreur et message, module d'états), ou une garantie de non-régression (AC-4, AC-6). Toute autre trace (AC, tâche, mutation, test, clé, table d'écran) est un finding. Vérifie aussi l'inverse : rien dont la 53-1b a besoin n'a disparu des deux fiches.
3. **Le mécanisme de D3, au code, pour la saisie.** Après le déplacement prescrit de la lecture de la borne, **aucune** lecture ordinaire ne précède-t-elle le verrou de l'exercice dans la transaction de `create` / `create_with_policy` (étape 0, sentinelle, projets, tout ce qu'ils appellent ; sous-requêtes comprises — c'est le défaut V2-1 de la conversion) ? L'inventaire clos des `UPDATE`/`DELETE` est-il complet (refais-le par le symptôme) et chaque site tient-il bien l'exercice ou est-il sans effet sur une donnée comparée ? Le déplacement change-t-il un comportement observable d'un flux automatique ?
4. **Prémisses au code.** Relis chaque citation `fichier:ligne` ; une citation fausse est un finding.
5. **Inventaire clos des créations.** Refais le relevé (y compris les `INSERT` à nom dynamique) et les appelants de `create_in_tx_inner`.
6. **Contrat et compatibilité (D4, D5, D7).** Recompte les relevés de D4 (dépôt, HTTP, Playwright) avec leurs commandes (le pathspec `crates/*/tests` est muet sous `git grep` : `grep -r`) ; `saisir_par` ; le corps propre `#[serde(flatten)]` (comportement de serde avec un champ mal typé, effet sur `entries[i]` de la conversion, sur le `PUT`) ; l'annonce ⚠️ et la condition sur `b`.
7. **Écran (D8).** Table J complète (entrées, sorties, « Annuler », Ctrl+S, `fetchMissingMonth`, l'assistant TVA) ; relevé C clos ; corps gardé ; clés de AC-13 cohérentes avec D8 ; module d'états sans dépendance au formulaire.
8. **Tests et mutations.** Chaque AC a-t-il un test qui rougit sans le correctif ? Chaque mutation (M1 à M23, avec M12b, M12c, M13c) est-elle tuable par le test annoncé ? AC-7 réalisable sans `INNODB_TRX` ? Un test vrai par construction ?
9. **Découpage.** La 53-1a reste-t-elle à quatre modules ? La dépendance 53-1a → 53-1b est-elle écrite des deux côtés ?
10. **Migration.** « Aucune migration » : vérifie.
11. **Manuel, PDF compris.** Vérifie le manuel (`docs/manual/`) contre le code et la fiche : `user-manual.tex:578`, `:760`, `:1764`, `:2463`, `admin-manual.tex:2680` ; lis les PDF, aplatis avant de greper : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (et `admin-manual.pdf`), **vers ta seule sortie standard** ; `docs/api-external.md`.
12. **Cohérence interne.** AC ↔ décisions ↔ tâches ↔ tests ↔ Note de version ↔ Change Log ; recompte tout décompte écrit dans la fiche.

## Interdits

- **N'écris aucun fichier, nulle part**, et n'exécute **aucune commande qui écrit** : pas d'`Edit`/`Write`, **aucune redirection `>` ni `>>`, pas de `tee`**, pas de `git commit`, `git checkout`, `git switch`, `git reset`, `git stash`, `git restore`, `git mv`, `git push`, pas de `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`, `scripts/mem-guard.sh` ni aucun script du dépôt, pas de `cargo`, pas de `npm`, pas de `python3`, pas de `docker` ni `docker compose`, pas de `sqlx`, pas de `mariadb`/`mysql`, pas de `curl`, pas de `make`.
- **Jamais `pkill`, `killall` ni `kill`** (ni par motif, ni par PID).
- Aucune commande `gh` qui écrit ; `gh` en lecture au plus, une minute au moins entre deux requêtes.
- Commandes permises : `git grep`, `git log`, `git show`, `git diff`, `grep`, `sed -n`, `awk` (lecture), `cat`, `head`, `tail`, `wc`, `ls`, `pdftotext … -` vers la sortie standard.

## Rapport attendu

En français. Pour chaque finding : identifiant `V3-n`, sévérité (CRITICAL, HIGH, MEDIUM, LOW), section de la fiche, constat **avec la ligne de code ou de fiche lue citée**, correctif proposé. Puis le tableau du sort des findings des P1 et P2 (axe 1). Puis le décompte par sévérité, et **la liste des axes exercés et non exercés** (un axe non exercé dit pourquoi). Un « 0 finding » sans cette liste ne compte pas comme passe. L'orchestrateur range le rapport sous `/home/gcorbaz/devel/kesh-gate-logs/e531/validation-p3.md`.
