# Story 53-6a — prompt de la revue de code P3 (Haiku, passe ciblée, contexte frais)

Tu conduis la **passe P3, ciblée, de la story 53-6a de Kesh** : une seule lentille, sur le **seul commit du lot des LOW de tests de la revue P2**. Contexte frais, modèle Haiku.

## Contexte

Kesh est un logiciel de comptabilité suisse ; Guy y tient ses livres réels. La story corrige un P1 (#1674) : le dé-rapprochement contre-passe l'écriture à **sa propre date**, dans son exercice (`CancellationDating::OriginDate`), non plus à la date du jour. Ce lot n'ajoute que des **tests** (aucun code de production) :

- **P2-L1** : l'exercice « suivant » du test Wise `s536a_ac7_un_paiement_wise_groupe_se_contre_passe_dans_son_exercice` est créé avec `multi_currency = TRUE`, pour que ce test rougisse sur la date sous la mutation M-R1 et non sur `409 MULTI_CURRENCY_DISABLED`.
- **P2-L5** : deux tests « AC-1, même exercice » neufs : `wise_reconciliation_e2e::s536a_ac1_un_paiement_wise_groupe_du_meme_exercice_se_contre_passe_a_sa_date` (R6, M-R1) et `jour_suisse_nuit_e2e::s536a_ac1_un_reglement_client_du_meme_exercice_se_contre_passe_a_sa_date` (R2, M-R2).

## Où travailler

- Worktree `/home/gcorbaz/devel/kesh-53-6`, branche `story/53-6-annulation-rapprochement-exercice`. Le commit du lot est celui qui suit `9d87fd2e8` ; trouve-le par `git log -3 --format='%h %s'`, puis lis le **diff unique** `git diff 9d87fd2e8 HEAD -- crates/` (aplati ; ne reconstitue pas un diff multi-commits).
- Fiche : `_bmad-output/implementation-artifacts/53-6a-annulation-rapprochement-date-origine.md` (tableau des mutations de la remédiation, Change Log, entrée du lot).
- Preuves : `/home/gcorbaz/devel/kesh-gate-logs/e53/s536a/remediation-p2/` (`lot.sh`, `codes.txt`, `M-R1.log`, `M-R2.log`, `*.diffstat`, `nextest-*.log`).
- Rapport de la P2 : `/home/gcorbaz/devel/kesh-gate-logs/e53/s536a/53-6a-revue-p2.md` (P2-L1, P2-L5).

## La lentille unique

**Ces tests prouvent-ils ce qu'on leur attribue ?** Pour chacun des trois tests touchés :

1. Au code sain, son assertion de date est-elle atteinte et vérifie-t-elle bien la date de l'**écriture d'origine** (10.03.2020 pour Wise ; 20.12.2025 pour R2), dans l'exercice d'origine ? La date du jour de l'instant simulé (22.12.2025 pour R2) est-elle bien **distincte** de celle de l'écriture ?
2. Sous la mutation (`CancellationDating::OriginDate` → `CancellationDating::Today(chrono::Utc::now().date_naive())` à `reconciliation_cancel.rs:978` pour R6, `:942` pour R2), le journal montre-t-il que le test rougit **sur l'assertion de date** (`left`/`right`) et non sur un statut, un `409` ou un autre accident de décor ? Lis `M-R1.log` et `M-R2.log`.
3. Le décor neuf est-il honnête : l'exercice multidevise du test Wise ne change-t-il rien d'autre à ce que le test exerce (phases « clos » et « rouvert » intactes) ? Le test « même exercice » Wise ne crée-t-il pas un exercice suivant (il doit rester dans l'exercice 2020-2030) ?
4. La restauration : `git diff --quiet` sur `reconciliation_cancel.rs` après chaque mutation (`codes.txt`), et le script n'utilise pas `sed -i` sur le source.

## Axes — déclare chacun exercé ou non exercé

1. Les trois tests contre le code et le décor (lentille 1 et 3).
2. Les journaux de mutation (lentille 2) : cite la ligne `left`/`right` lue.
3. La restauration et les gates (`codes.txt` : fmt, clippy, nextest ; recompte les tests passés dans `nextest-*.log`).
4. Le périmètre : `git diff --name-only 9d87fd2e8 HEAD` ne doit nommer que les deux fichiers de test, la fiche et ce prompt ; aucune migration, aucun code de production.
5. La fiche : les noms de tests cités existent-ils ? (`grep -nF "fn <nom>"`.)
6. Manuel et i18n : **non concernés** (aucun texte visible touché) ; déclare-les non exercés, avec cette raison. Si tu les exerces, aplatis le PDF : `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`.

Un « 0 finding » sans la liste des axes **exercés et non exercés** ne compte pas comme passe.

## Garde-fou

Tout finding (surtout CRITICAL ou HIGH) qui **affirme l'absence** d'un code, d'un test ou d'une assertion se vérifie par `grep -nF "<chaîne>" <fichier>` (`-F` obligatoire ; une ligne discriminante, ou `grep -nFA <N>` pour plusieurs lignes) avant d'être rendu ; cite la sortie. Un finding réfuté est écarté et consigné comme faux positif.

## Interdits

- Rien hors du worktree, de `/tmp` et de `/home/gcorbaz/devel/kesh-gate-logs/e53/`.
- Aucun correctif : tu n'appliques rien. Pas de `gh` en écriture, pas de `scripts/prepare-release.sh`, pas de `docker`, pas de `sed -i`.
- Si tu rejoues un test : sous `flock /tmp/station-heavy.lock flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock`, base `kesh_e536`. **Jamais de `GRANT` ni de `REVOKE`, jamais de redémarrage du conteneur.**
- Jamais `pkill`, `killall` ni `kill` par motif.

## Ce que tu produis

Un rapport `/home/gcorbaz/devel/kesh-gate-logs/e53/s536a/53-6a-revue-p3.md` : décompte (CRITICAL/HIGH/MEDIUM/LOW), findings avec leur preuve, axes exercés et non exercés. Réponse finale en français.
