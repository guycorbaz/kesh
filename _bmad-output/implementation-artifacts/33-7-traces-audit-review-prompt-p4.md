# Revue de code P4 ciblée — 33-7, la remédiation de la P3 (#435), une lentille (lecture seule)

Contexte frais, français soigné. Passe ciblée : relis **la remédiation de la P3**, non la story. Elle touche du code de production de `kesh-db` : `revoke_all_for_user` reprend la constante `REVOKE_REST_FOR_USER_SQL` (donc un `FORCE INDEX` neuf sur le chemin du changement de mot de passe et de la désactivation de compte) ; deux doc-comments ; et elle produit le premier E2E complet depuis `59fddee1`, qui touchait déjà du SQL de production.

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-e336 diff c9830962 138e59de
git -C /home/gcorbaz/devel/kesh-e336 log --oneline c9830962..story/33-7-traces-audit
```

Bornes en SHA : `c9830962` (fin de la P3), `138e59de` (dernier commit de code) ; la tête de `story/33-7-traces-audit` porte en plus le Change Log, le registre (C-e334-33) et ce prompt. Lis par `git show <sha>:<fichier>`. Rapport de la P3 : `/home/gcorbaz/devel/kesh-gate-logs/e337-review-p3.md` (0/0/1/3). Fiche, dernière entrée du Change Log (« remédiation de la revue P3 ») : `git show story/33-7-traces-audit:_bmad-output/implementation-artifacts/33-7-traces-audit.md`. Journaux : `/home/gcorbaz/devel/kesh-gate-logs/e337/p4/` — `plan-peuple-337.sql` et `plan-peuple.log` (mesure du plan sur table peuplée), `gates-head.log`, `gate-test-fast.log`, `gate-build.log`, `gate-front-build.log`, `e2e-r4/playwright.log`, `e2e-r4/backend.log`, `e2e-r4/ss-avant.log`, `e2e-r4/ss-apres.log`, et les scripts `gates-337-p4.sh`, `e2e-337-p4.sh`, `reset-db-337-p4.sh`.

## Axes obligatoires

1. **L2, `revoke_all_for_user`** : le texte SQL de l'ancienne fonction est-il exactement celui de `REVOKE_REST_FOR_USER_SQL` hors indice (mêmes liaisons, même ordre : motif, `user_id`) ; le contrat de retour (nombre de **lignes**) est-il inchangé ; ses six appelants (`git grep -n "revoke_all_for_user(" 138e59de -- crates`) sont-ils tous indifférents à l'indice ; le test `revoke_live_statements_go_through_the_user_id_index` couvre-t-il désormais ce chemin, et le dit-il honnêtement (il teste la constante, non l'appel) ?
2. **L1, la mesure** : `plan-peuple-337.sql` mesure-t-il ce qu'il prétend (5000 lignes, 50 utilisateurs, statistiques à jour par `ANALYZE … PERSISTENT`, plan sans indice et avec) ; le doc-comment de `REVOKE_LIVE_FOR_USER_SQL` et celui du test disent-ils exactement ce qui a été mesuré, ni plus ni moins (une seule distribution synthétique, MariaDB 10.11) ?
3. **M1, l'E2E** : le journal `e2e-r4/playwright.log` est-il celui de la tête de code `138e59de` (ligne « backend PID …, tête … ») ; la recette suit-elle `docs/testing.md` § *Prérequis Playwright local* (port 3186, `kesh_e2e_e337` recréée vide, dossiers `/tmp/kesh-e2e-e337-r4/…` passés au backend **et** au runner, `KESH_TEST_MODE` des deux côtés, `KESH_COOKIE_SECURE=false`, quatre `KESH_SMTP_*`, `smtpConfigured:true` au `/health`) ; backend lancé dans la prise de verrou, descripteurs `8>&- 9>&-` fermés, tué par PID, `ss` avant et après vides ; **chaque échec nommé contre `docs/testing.md` § « Les échecs attendus »**, fichier par fichier : un échec hors liste est-il une régression de la story (la révocation au changement de mot de passe, au vol de jeton, à la désactivation de compte) ?
4. **Les gates déclarés** : `gates-head.log` (tête, état propre, `test-fast rc`, résumé nextest, `build rc`, `front build rc`, `e2e rc`) contre ce que dit le Change Log ; ordre code d'abord, E2E ensuite, au même commit.
5. **Le compte rendu** : l'entrée « remédiation de la revue P3 » ne réécrit-elle aucune entrée passée ; ses décomptes se recomptent-ils depuis les journaux ; L3 « rien à faire » est-il dit ?

## Contraintes

⛔ **La lentille écrit son rapport à ce chemin, et à lui seul : `/home/gcorbaz/devel/kesh-gate-logs/e337-review-p4.md`.** Un rapport rendu par message ne compte pas. Lecture seule hors de ce fichier : ni redirection ni fichier temporaire (les tubes sont permis), aucune compilation ni exécution, aucune requête à une base ; `scripts/prepare-release.sh`, `git checkout`, `git stash`, `git switch`, `git reset`, `docker` et tout `gh` qui écrit sont interdits. `grep` est une fonction vers `ugrep` dans ce shell : `/usr/bin/grep` ou `git grep`.

Le rapport porte : **les axes réellement exercés et ceux qui ne l'ont pas été** ; chaque finding avec sévérité, fichier et ligne, extrait lu, scénario, remédiation ; toute présence ou absence adossée à une sortie `git grep` ou `/usr/bin/grep -nF` copiée ; un décompte recompté.
