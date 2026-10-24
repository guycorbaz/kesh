# Prompt — revue de code P3 de la Story 32-4a, passe CIBLÉE, une lentille (regression hunter), Sonnet

Tu relis **la seule remédiation de la revue de code P2** de la Story 32-4a de Kesh (comptabilité suisse, Rust/Axum,
MariaDB) : *la couche de services — rollout des lectures : écritures comptables, postes ouverts, contacts*. Worktree
`/home/gcorbaz/devel/kesh-32-4a`, branche `story/32-4a`. Langue du rapport : français. Modèle : **Sonnet**. Contexte
frais : ne t'appuie sur aucune passe antérieure, ne prends aucun compte rendu pour argent comptant.

**Le périmètre, et rien d'autre** : le diff aplati de la remédiation,
`git -C /home/gcorbaz/devel/kesh-32-4a diff 2e46e0d2..HEAD` (le dernier commit avant la remédiation est `2e46e0d2` ;
la remédiation tient dans les commits qui le suivent, `git log --oneline 2e46e0d2..HEAD`). Le reste de la story a été
relu en P1 et en P2 ; **ne le relis pas**, sauf pour juger une ligne du diff.

À lire pour juger le diff :

- le rapport de la P2 : `/home/gcorbaz/devel/kesh-gate-logs/324a-review-p2.md` (P2-1 MEDIUM : le tri et le décalage
  des écritures, le décalage du carnet, sans témoin, et une phrase fausse du compte rendu ; P2-2 et P2-3 LOW :
  doc-comments) ;
- dans la fiche `_bmad-output/implementation-artifacts/32-4a-services-ecritures-contacts.md` : au **Dev Agent
  Record**, la section « Revue de code P2 — remédiation » et l'annotation datée du 2026-10-25 sous le tableau des
  mutations M19 à M37 ; au **Change Log**, l'entrée « Revue de code P2 » ;
- au registre `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-324a.md`, l'annotation datée de
  C-32-4a-20 et l'entrée C-32-4a-24 ;
- les journaux de la remédiation, en lecture seule : `/home/gcorbaz/devel/kesh-gate-logs/324a-p2r-*` (campagne
  `324a-p2r-mutations-campagne.log`, pilote `324a-p2r-mutations-pilote.py`, diffs et journaux
  `324a-p2r-mut-M*.{diff,log}`, gates `324a-p2r-*.log`).

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**,
  aucune redirection (`>`, `>>`, `tee`), même vers un fichier « jetable » : compare par des commandes en pipe. Un
  fichier créé par erreur se **déclare** au rapport ; il ne se supprime pas.
- **Aucun `git checkout`, `git switch`, `git restore`** sous aucune forme ; aucun `git` qui écrit (`commit`, `stash`,
  `reset`, `tag`, `push`, `rebase`, `worktree`, `add`).
- **N'exécute rien qui écrit, compile ou exécute du code** : ni **`scripts/prepare-release.sh`**, ni
  `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`, ni aucun script de
  `kesh-gate-logs` (les pilotes `324a-*mutations-pilote.py` compris : ils écrivent dans le dépôt), ni `make`, ni
  `latexmk`, ni `cargo` (aucune sous-commande, `cargo fmt` compris), ni `rustc`, ni `npm`/`npx`/`node`, ni `docker`,
  ni `mysql`/`mariadb`, ni `sqlx`, ni `curl`, ni `python3`, ni `sed -i`, ni `perl`. Aucune commande `gh`.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection),
  `cat`, `ls`, `find`, `wc`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée.
- **Données réelles** : aucune ; les fixtures sont synthétiques. Ne lis rien sous `~/kesh-test/`.

## La lentille — regression hunter, braquée sur la remédiation de la P2

1. **Les lignes neuves prouvent-elles ce qu'elles disent ?** Dans `filtres_des_ecritures_par_la_route`, la table
   ordonnée (défaut, `sortDir=Asc`, `sortBy=Description&sortDir=Asc`, `limit=1&offset=1`) et le tri par défaut au
   service ; dans `tri_par_defaut_et_filtres_du_carnet`, `limit=1&offset=1`. Chaque ordre attendu se déduit-il de la
   fixture (dates, libellés, noms) et de la requête du dépôt (`kesh-db/src/repositories/journal_entries.rs` :
   `ORDER BY {sort_col} {sort_dir}, entry_number DESC` ; `contacts.rs` : `ORDER BY`, `LIMIT`, `OFFSET`) et des défauts
   (`kesh-core/src/listing/mod.rs` : `SortBy::default`, `SortDirection::default` ; `routes/journal_entries.rs` :
   `default_sort_by`, `default_sort_dir`, `default_offset`) ? Une ligne passe-t-elle à vide — deux mutations
   distinctes qui rendraient l'ordre attendu, un ordre par libellé qui dépendrait d'une collation, un `offset` dont la
   ligne rendue serait la même avec `offset 0` ? Le test du service passe-t-il par `DemandeListeEcritures::default()`,
   ce que reçoit le MCP ?
2. **Les mutations M38 à M43** (`324a-p2r-mutations-campagne.log`, `324a-p2r-mut-M*.diff`) : chaque diff est-il celui
   que décrit le Dev Agent Record ? chaque « TUÉE » l'est-elle par la ligne visée, et non par une compilation ou un
   montage ? Une recopie de tri ou de décalage (service ou handler, écritures ou carnet) reste-t-elle sans témoin ?
   Raisonne sur le code, n'exécute rien.
3. **Les doc-comments corrigés** (P2-2 : `ac11_b_la_liste_des_ecritures_ne_lit_pas_la_societe` ; P2-3 :
   `sans_commentaires` et `ac12_iii_inventaire_des_sites_non_resolus` dans `tests/services_motif.rs`) disent-ils vrai au
   code (`ac9_c`, `ac9_carnet_detail_lit_le_contact_avant_la_societe`, la regex `(?s)/\*.*?\*/` sur le corps entier,
   les quatre fichiers lus par (ii) et (iii)) ? Les noms cités existent-ils (`grep -nF`) ?
4. **Production** : la remédiation touche-t-elle une ligne de `crates/*/src` (`git diff 2e46e0d2..HEAD --stat --
   'crates/*/src'`) ? Si oui, laquelle, et qu'est-ce qui la tient ?
5. **Comptes rendus** : les annotations datées (fiche, sous le tableau M19-M37 ; registre, C-32-4a-20) corrigent-elles
   exactement ce que P2-1 dit faux, sans réécrire l'entrée passée (`git diff 2e46e0d2..HEAD` de la fiche et du
   registre : des ajouts seuls) ? Les décomptes de l'entrée « Revue de code P2 » du Change Log et du Dev Agent Record
   (tests, lignes de table, mutations, gates) se recomptent-ils depuis la source, avec leur périmètre ? Les gates
   déclarés au **dernier commit de code** le sont-ils (révision en tête des journaux contre `git log --oneline
   2e46e0d2..HEAD`) ? Les échecs E2E déclarés sont-ils les sept KF-029 de `docs/testing.md` § *Les échecs attendus* ?
6. **Messages de commit** : `git log 2e46e0d2..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
   ne doit rien rendre ; chaque commit porte `refs #723`.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/324a-review-p3.md` :

- bilan en tête : CRITICAL / HIGH / MEDIUM / LOW ;
- findings numérotés `P3-1…`, chacun avec sévérité, fichier:ligne, **preuve copiée** (sortie `grep -nF`, `sed -n` ou
  `git diff`), correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** (points 1 à 6 ci-dessus, un par un) — un
  « 0 finding » sans cette liste ne compte pas comme passe ;
- rien d'autre n'est écrit nulle part.
