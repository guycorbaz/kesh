# Revue de code P3 ciblée — 33-7, la remédiation de la P2 (#435), une lentille (lecture seule)

Contexte frais, français soigné. Passe ciblée : relis **la remédiation de la P2**, non la story. Elle touche du code de production : `kesh-db` (deux instructions SQL devenues constantes, `FORCE INDEX`), le message d'un refus (code et quatre locales), un doc-comment, des tests, `api-external.md`, le manuel utilisateur et le CHANGELOG.

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-e337 diff 05335438 59fddee1
git -C /home/gcorbaz/devel/kesh-e337 log --oneline 05335438..story/33-7-traces-audit
```

Bornes en SHA : `05335438` (fin de la P2), `59fddee1` (dernier commit de code) ; la tête de `story/33-7-traces-audit` porte en plus les PDF et le Change Log. Le worktree peut porter une autre branche : lis par `git show 59fddee1:<fichier>`. Rapport de la P2 : `/home/gcorbaz/devel/kesh-gate-logs/e337-review-p2.md`. Fiche, dernière entrée du Change Log : `git show story/33-7-traces-audit:_bmad-output/implementation-artifacts/33-7-traces-audit.md`. Journaux : `/home/gcorbaz/devel/kesh-gate-logs/e337/p3-explain.log`, `/home/gcorbaz/devel/kesh-gate-logs/e337/p3/` (`rouge-L5.log`, `vert-repo.log`, `mut-P3M1.log`, `mut-P3M2.log`, `mut-L2.log`, `gates3-head.log`).

## Axes obligatoires

1. **`FORCE INDEX`** : la syntaxe d'un `UPDATE` mono-table en MariaDB 10.11 ; l'index nommé existe-t-il sous ce nom dans la migration (`20260405000001_auth_refresh_tokens.sql`) **et** dans le squash ; ce qui se passe si l'index est un jour renommé (erreur bruyante, ou silence ?) ; le test `revoke_live_statements_go_through_the_user_id_index` : `EXPLAIN` d'une instruction préparée, colonne `key`, prouve-t-il ce qu'il nomme, et le prouverait-il sur une table peuplée ?
2. **Le contrat de `revoke_all_for_user_counting_live`** : le test de dépôt couvre-t-il ce que dit le doc-comment ; les deux mutations au journal sont-elles tuées par l'assertion qu'elles visent ?
3. **Le message du 403** : la traduction des trois locales est-elle fidèle au français ; le message reste-t-il **distinct** de ses deux voisins (`loader.rs:699-725`) ; le repli Rust est-il égal au fr-CH (`textes_coherents.rs:775`) ; reste-t-il un site qui dit « Utilisez l'interface web » pour une opération sans écran (`git grep -nE "interface web|web interface|Weboberfläche|interfaccia web" 59fddee1 -- crates docs frontend/src`) ? Un client ou un test qui comparerait l'ancien texte ?
4. **L2** : la mutation au journal est-elle la bonne, et le test la tue-t-il pour la bonne raison ?
5. **Les textes** : `api-external.md` § 4 et § 10, le CHANGELOG, et **le manuel** : `docs/manual/fr/user-manual.tex` et son PDF, aplati (`git show <tête>:docs/manual/fr/user-manual.pdf | pdftotext - - | tr '\n' ' ' | tr -s ' '`), la conduite « Mot de passe oublié » sous sa condition.
6. **Les gates déclarés** : `gates3-head.log` contre ce que dit le Change Log ; l'E2E n'a pas été rejoué, motif écrit au Change Log : le juger.

## Contraintes

⛔ **La lentille écrit son rapport à ce chemin, et à lui seul : `/home/gcorbaz/devel/kesh-gate-logs/e337-review-p3.md`.** Un rapport rendu par message ne compte pas. Lecture seule hors de ce fichier : ni redirection ni fichier temporaire (les tubes sont permis), aucune compilation ni exécution, aucune requête à une base ; `scripts/prepare-release.sh`, `git checkout`, `git stash`, `git switch`, `docker` et tout `gh` qui écrit sont interdits. `grep` est une fonction vers `ugrep` dans ce shell : `/usr/bin/grep` ou `git grep`.

Le rapport porte : **les axes réellement exercés et ceux qui ne l'ont pas été** ; chaque finding avec sévérité, fichier et ligne, extrait lu, scénario, remédiation ; toute présence ou absence adossée à une sortie `git grep` ou `/usr/bin/grep -nF` copiée ; un décompte recompté.
