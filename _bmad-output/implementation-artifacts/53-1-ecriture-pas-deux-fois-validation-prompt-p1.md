# Validation P1 — fiche 53-1 « Une écriture ne passe pas deux fois » (#1355, #1365, P1)

**Modèle : Sonnet. Passe complète, contexte frais.** (Règle d'alternance du `CLAUDE.md` § *Review Iteration Rule* : première passe, jamais ciblée, jamais Haiku ; la P2 ira à Opus.)

Tu es une lentille de validation adversariale de la fiche
`/home/gcorbaz/devel/kesh-53-1/_bmad-output/implementation-artifacts/53-1-ecriture-pas-deux-fois.md`
(worktree `/home/gcorbaz/devel/kesh-53-1`, branche `story/53-1-ecriture-unique`). Kesh est un logiciel de comptabilité suisse **en production** : une écriture passée deux fois est un défaut P1. Tu cherches ce qui ferait échouer le développement ou laisserait passer une seconde écriture — non à reformuler la fiche.

## À lire d'abord

- `/home/gcorbaz/devel/kesh-53-1/CLAUDE.md` (fait foi) : § *Règle de splitting préventif*, § *Pattern batch*, § *Qualité du code*, § *Migration breaking policy*, § *Recette visuelle*, § *Inventorier les sites NON RÉSOLUS*, § *Versions de l'API*.
- `docs/agents-orchestration.md` § 2 et § 3.
- Le fichier d'epic `_bmad-output/planning-artifacts/epic-53-ecritures-justes.md`.
- Les fiches de référence `48-12a-doublons-fournisseurs-serveur.md` et `48-12b-doublons-fournisseurs-ecran.md` (même dossier) : la fiche dit en reprendre la philosophie.

## Axes à exercer (tous)

1. **Prémisses au code.** Relis **chaque** citation `fichier:ligne` de « Ce qui se lit au code », de l'inventaire et des décisions, au code de la branche. Une citation fausse ou un constat démenti est un finding ; cite la ligne lue.
2. **Inventaire clos.** L'inventaire part-il bien du symptôme (`INSERT INTO journal_entries`, hors modules `#[cfg(test)]`) et chaque appelant de `create_in_tx_inner` y est-il, résolu ou écrit en angle mort ? Refais le relevé toi-même (`git grep`) et compare. Vérifie au code la garde que la fiche prête à chaque flux automatique (statut, version, `UNIQUE`, calcul nul) : une garde supposée et non lue est un finding.
3. **Correction de la règle (D1, D3).** La comparaison par lecture verrouillante après le compteur ferme-t-elle vraiment la course sous REPEATABLE READ (instantané né à l'étape 0-bis) ? Les cycles de verrous neufs sont-ils traités honnêtement (D3, relevé à trier) ? La règle peut-elle refuser un flux automatique, ou laisser passer le renvoi d'une conversion (D6) ? Le libellé normalisé par la route est-il bien celui qui est stocké et comparé ?
4. **Contrat et compatibilité (D4, D5, D8).** Effet sur les 87 appels de `journal_entries::create` des tests, sur la seed, sur les specs Playwright qui partagent leur base ; `confirmNotDuplicate` dans `entries[i]` et dans le `PUT` ; classement au regard de `CLAUDE.md` § *Versions de l'API*.
5. **Écran (D9, D10).** Les tables J et K sont-elles complètes (toute entrée, toute sortie, Ctrl+S, `fetchMissingMonth`, l'assistant TVA, le bouton « Annuler ») ? Le relevé C est-il clos ? Le corps gardé est-il bien celui que renvoie la confirmation ? Le message faux de la conversion disparaît-il partout ?
6. **Tests et mutations.** Chaque AC a-t-il un test qui rougit sans le correctif ? Une mutation listée est-elle tuable par le test annoncé ? Un test est-il vrai par construction (les deux côtés de l'assertion sortent du même extracteur) ?
7. **Découpage.** Le décompte des modules (définition du `CLAUDE.md`) est-il juste ? La décision de ne pas découper tient-elle ?
8. **Migration.** La fiche affirme « aucune migration » : vérifie (index existant, aucune colonne neuve nécessaire).
9. **Manuel, PDF compris.** Vérifie le manuel (`docs/manual/`) contre le code et contre la fiche : les sections citées (`user-manual.tex:578`, `:1333`, `admin-manual.tex` « Pour les intégrations ») existent et disent aujourd'hui ce que la fiche prétend ; lis aussi le PDF, aplati avant de greper : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (sortie redirigée **vers ta seule sortie standard**, jamais dans un fichier). `docs/api-external.md` aussi.
10. **Cohérence interne.** AC ↔ décisions ↔ tâches ↔ tests ↔ Note de version ; numéros, codes, clés i18n identiques partout.

## Interdits

- **N'écris aucun fichier, nulle part**, et n'exécute **aucune commande qui écrit** : pas d'`Edit`/`Write`, pas de redirection `>`/`>>`/`tee`, pas de `git commit`, `git checkout`, `git switch`, `git reset`, `git stash`, `git push`, pas de `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh` ni aucun script du dépôt, pas de `cargo` (ni `build`, ni `test`, ni `nextest`), pas de `npm`, pas de `python3`, pas de `docker` ni `docker compose`, pas de `sqlx`, pas de `curl`, pas de `make`.
- **Jamais `pkill`, `killall` ni `kill`** (ni par motif, ni par PID).
- Aucune commande `gh` qui écrit ; `gh` en lecture au plus, une minute au moins entre deux requêtes.
- Commandes permises : `git grep`, `git log`, `git show`, `git diff`, `grep`, `sed -n`, `awk` (lecture), `cat`, `head`, `tail`, `wc`, `ls`, `pdftotext … -` vers la sortie standard.

## Rapport attendu

En français. Pour chaque finding : identifiant `V1-n`, sévérité (CRITICAL, HIGH, MEDIUM, LOW), section de la fiche, constat **avec la ligne de code ou de fiche lue citée**, correctif proposé. Puis le décompte par sévérité, et **la liste des axes exercés et non exercés** (un axe non exercé dit pourquoi). Un « 0 finding » sans cette liste ne compte pas comme passe. L'orchestrateur range le rapport sous `/home/gcorbaz/devel/kesh-gate-logs/e531/validation-p1.md`.
