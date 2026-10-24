# Prompt — revue de code P2 de la Story 32-4a, passe CIBLÉE, une lentille (regression hunter), Opus

Tu relis **la seule remédiation de la revue de code P1** de la Story 32-4a de Kesh (comptabilité suisse, Rust/Axum,
MariaDB) : *la couche de services — rollout des lectures : écritures comptables, postes ouverts, contacts*. Worktree
`/home/gcorbaz/devel/kesh-32-4a`, branche `story/32-4a`. Langue du rapport : français. Modèle : **Opus**. Contexte
frais : ne t'appuie sur aucune passe antérieure, ne prends aucun compte rendu pour argent comptant.

**Le périmètre, et rien d'autre** : le diff aplati de la remédiation,
`git -C /home/gcorbaz/devel/kesh-32-4a diff 8f3b4ffe..HEAD` (le dernier commit avant la remédiation est `8f3b4ffe` ;
la remédiation tient dans les commits qui le suivent, `git log --oneline 8f3b4ffe..HEAD`). Le reste de la story a été
relu en P1 ; **ne le relis pas**, sauf pour juger une ligne du diff.

À lire pour juger le diff :

- les trois rapports de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/324a-review-p1-{B,E,A}.md` (les findings que la
  remédiation prétend traiter : B1 = E1 = E2 avec A4, B2, A1, A2, A3 = E3, B4 ; B3, E4, E5 tracés hors code) ;
- dans la fiche `_bmad-output/implementation-artifacts/32-4a-services-ecritures-contacts.md` : l'AC6, l'AC14, et, au
  **Dev Agent Record**, la section « Revue de code P1 — remédiation » ; au **Change Log**, l'entrée « Revue de code P1 » ;
- au registre `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-324a.md`, les entrées ajoutées par la
  remédiation (C-32-4a-19 et suivantes) ;
- les journaux de la remédiation, en lecture seule : `/home/gcorbaz/devel/kesh-gate-logs/324a-p1r-*` (campagne
  `324a-p1r-mutations-campagne.log`, pilote `324a-p1r-mutations-pilote.py`, diffs et journaux
  `324a-p1r-mut-M*.{diff,log}`, gates `324a-p1r-*.log`).

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**,
  aucune redirection (`>`, `>>`, `tee`), même vers un fichier « jetable » : compare par des commandes en pipe. Un
  fichier créé par erreur se **déclare** au rapport ; il ne se supprime pas.
- **Aucun `git checkout`, `git switch`, `git restore`** sous aucune forme ; aucun `git` qui écrit (`commit`, `stash`,
  `reset`, `tag`, `push`, `rebase`, `worktree`, `add`).
- **N'exécute rien qui écrit, compile ou exécute du code** : ni **`scripts/prepare-release.sh`**, ni
  `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`, ni aucun script de
  `kesh-gate-logs` (le pilote `324a-p1r-mutations-pilote.py` compris : il écrit dans le dépôt), ni `make`, ni
  `latexmk`, ni `cargo` (aucune sous-commande, `cargo fmt` compris), ni `rustc`, ni `npm`/`npx`/`node`, ni `docker`,
  ni `mysql`/`mariadb`, ni `sqlx`, ni `curl`, ni `python3`, ni `sed -i`, ni `perl`. Aucune commande `gh`.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection),
  `cat`, `ls`, `find`, `wc`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée.
- **Données réelles** : aucune ; les fixtures sont synthétiques. Ne lis rien sous `~/kesh-test/`.

## La lentille — regression hunter, braquée sur la remédiation

La sévérité se déplace vers ce qu'on vient d'écrire : cherche d'abord ce que **la remédiation** a pu casser ou
mal établir.

1. **Les tests neufs prouvent-ils ce qu'ils disent ?** `filtres_des_ecritures_par_la_route`,
   `tri_par_defaut_et_filtres_du_carnet`, `ac11_b_la_liste_des_ecritures_ne_lit_pas_la_societe`, et les deux lignes
   ajoutées à `ac10_a`. Pour chaque ligne de leurs tables : l'ensemble ou l'ordre attendu se déduit-il bien de la
   fixture (dates, montants `SUM(debit)`, journaux, comptes, noms, `created_at`, `updated_at`, `active`) et de la
   requête du dépôt (`kesh-db/src/repositories/journal_entries.rs`, `push_where_clauses` ; `contacts.rs`,
   `push_where_clauses` et `ORDER BY`) ? Une ligne passerait-elle **à vide** — deux mutations rendant le même
   ensemble que l'attendu, une recherche FULLTEXT qui ne dépend pas du filtre, un tri à égalité dont l'ordre serait
   celui de l'insertion, une collation qui range « Archivé » ailleurs ? Le test du carnet au service passe-t-il par le
   même défaut que le MCP (`DemandeListeContacts::default()`) ? Le test de la société inexistante peut-il passer pour
   une autre raison qu'une absence de lecture de la société ?
2. **Les mutations M19 à M37** (`324a-p1r-mutations-campagne.log`, `324a-p1r-mut-M*.diff`) : chaque diff est-il
   celui que décrit le Dev Agent Record ? chaque « TUÉE » l'est-elle par l'assertion nommée, et non par un échec de
   compilation ou de montage ? une survivante éventuelle est-elle tuée ou déclarée équivalente **avec sa preuve** ?
   Une mutation plausible **non jouée** sur les recopies (les sept paramètres du handler des contacts, les onze du
   handler des écritures, les deux défauts du tri au service) survivrait-elle aux tests neufs ? Raisonne sur le code,
   n'exécute rien.
3. **Les textes modifiés** : le doc-comment de `services/mod.rs` (branches non atteintes, A1) dit-il vrai au code
   (`require_viewable_account`, libellé des conditions de paiement au handler) et est-il aligné sur le Dev Agent
   Record ? La limite de `sans_commentaires` (B4, `tests/services_motif.rs`) est-elle exacte (`//` **et** `/*` dans une
   chaîne) et écrite aux deux endroits annoncés ? Les doc-comments ajoutés aux services, s'il y en a, nomment-ils des
   tests qui existent (`grep -nF` du nom) ?
4. **Production** : la remédiation touche-t-elle une ligne de code de production autre que des doc-comments
   (`git diff 8f3b4ffe..HEAD --stat -- crates/*/src`, puis le diff lui-même) ? Si oui, laquelle, et qu'est-ce qui la
   tient ?
5. **Comptes rendus** : les décomptes de l'entrée « Revue de code P1 » du Change Log et du Dev Agent Record (tests
   ajoutés, mutations, gates) se recomptent-ils depuis la source (`git show 8f3b4ffe:<f>` contre `HEAD`,
   `grep -cE '#\[(sqlx::|tokio::)?test'` ; lignes `TUÉE` / `SURVIVANTE` du journal ; `Summary` des journaux de gate),
   avec leur périmètre ? Les gates déclarés au **dernier commit de code** le sont-ils bien (révision en tête des
   journaux contre `git log --oneline 8f3b4ffe..HEAD`) ? Les échecs E2E déclarés sont-ils les sept KF-029 de
   `docs/testing.md` § *Les échecs attendus* ? Une entrée de passe passée a-t-elle été **réécrite** (`git diff
   8f3b4ffe..HEAD -- <fiche>` ne doit montrer que des ajouts dans le Change Log et le Dev Agent Record, hors entrée
   vivante) ? Le registre n'a-t-il que des ajouts ?
6. **Messages de commit** : `git log 8f3b4ffe..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
   ne doit rien rendre ; chaque commit porte `refs #723`.
7. **Ce qui a été tracé au lieu d'être corrigé** (B3, E5 → #738 ; E4 → #742) : est-ce dit au Change Log comme une
   demande à l'orchestrateur, et rien dans le code ne prétend-il l'avoir réglé ?

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/324a-review-p2.md` :

- bilan en tête : CRITICAL / HIGH / MEDIUM / LOW ;
- findings numérotés `P2-1…`, chacun avec sévérité, fichier:ligne, **preuve copiée** (sortie `grep -nF`, `sed -n` ou
  `git diff`), correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** (points 1 à 7 ci-dessus, un par un) — un
  « 0 finding » sans cette liste ne compte pas comme passe ;
- rien d'autre n'est écrit nulle part.
