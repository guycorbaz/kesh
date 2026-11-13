# Prompt — revue de code P4 ciblée de la Story 35-5a0 (refs #929, #1022, #907), une lentille, braquée sur la remédiation de la P3

Passe **ciblée**, une seule lentille, contexte frais, **Sonnet** (rotation : P1 Opus, P2 Sonnet, P3 Opus), en lecture seule. Worktree
`/home/gcorbaz/devel/kesh-e355a0`, branche `story/35-5a0-horloge-plomberie`. Historique : P1 3 MEDIUM → P2 ciblée 1 MEDIUM → P3 ciblée
(`35-5a0-horloge-plomberie-review-prompt-p3-ciblee.md`, rapport `/home/gcorbaz/devel/kesh-gate-logs/e355a0-review-p3-ciblee.md`)
1 MEDIUM (D1 : des tests mêlaient encore le jour réel et le jour de test, que les inventaires par lecture ne voyaient pas) et 1 LOW (D2).
La P3 a changé de **méthode** : le détecteur est désormais **l'exécution** de la suite sous `KESH_TEST_INSTANT` décalé de trois jours
(un test qui mêle deux jours rougit, quelle que soit sa forme). Ta passe juge si cette méthode est saine et si sa remédiation est juste.

## Périmètre : le seul commit `59f005a1f`

`git -C /home/gcorbaz/devel/kesh-e355a0 show 59f005a1f` — cinq fichiers de tests (`bank_accounts_closed_e2e.rs`,
`date_du_jour_garde.rs`, `journal_entry_reversal_e2e.rs`, `period_lock_e2e.rs`, `supplier_invoices_draft.rs`) ; aucun fichier de
production (vérifie par `git show --stat`). Compte rendu : entrée P3 du Change Log de
`_bmad-output/implementation-artifacts/35-5a0-horloge-plomberie.md` (commit `3040801df`). Journaux sous
`/home/gcorbaz/devel/kesh-gate-logs/e355a0/` : `r3-sonde-avant.log` (sonde, commit `98f1a910`), `r3-instant-decale.log` (10 rouges
avant correction), `r3-instant-decale-2.log` et `r3-test-fast.log` (5344/5344 au commit `59f005a1`).

## Ce que tu vérifies

1. **La méthode** : la variable atteint-elle vraiment chaque côté ? Lis le chemin de lecture de `KESH_TEST_INSTANT` dans
   `crates/kesh-db/tests/support/jour.rs`, `crates/kesh-api/tests/common/jour.rs`, `crates/kesh-report/tests/support/jour.rs`, le
   serveur lancé par les tests `kesh-api` (`git grep -n 'NOM_VARIABLE_INSTANT\|instant_de_test' crates/`), et `scripts/test-fast.sh` /
   `scripts/mem-guard.sh` (la variable traverse-t-elle `systemd-run` ? lis comment l'environnement est transmis). `kesh-report` n'a
   aucun rouge qui prouve qu'il la lit : est-ce établi par le code ? Le journal décalé porte-t-il bien la variable (en-tête, commande) ?
2. **La sonde** : `r3-sonde-avant.log` rend-il des rouges **nommés** de `supplier_invoices_draft` dont le message montre deux dates
   différentes (le 10 et le 13 novembre) ? Le commit indiqué est-il antérieur à la correction ?
3. **Les `replace_all`** dans `journal_entry_reversal_e2e.rs` et `period_lock_e2e.rs` : chaque `Utc::now().date_naive()` remplacé
   désignait-il bien « le jour », et non un instant, une borne d'expiration, ou une date qui devait rester réelle (comparée à une valeur
   que le serveur lit sur l'heure réelle : jeton, `created_at`, journal d'audit) ? `annee_courante()` et `fn hier()` : cohérents avec le
   verrou de période que le serveur juge au jour de test ? Lis chaque ligne changée.
4. **`supplier_invoices_draft.rs`** et **`bank_accounts_closed_e2e.rs`** : un seul jour par test désormais ? Les deux `closedOn` restant
   dans `bank_accounts_closed_e2e.rs` (renvoyés à la 35-5b) : la lecture du code les classe-t-elle bien comme un comportement de
   production non encore converti, et non comme un mélange dans le test ?
5. **La limite déclarée** : un décalage de trois jours ne voit que les comparaisons à l'égalité ou à moins de trois jours de marge. Cite
   un exemple concret au code qui échapperait (marge de 30 jours, bascule d'exercice) s'il en existe, ou dis que tu n'en trouves pas
   (commande copiée). La limite est-elle transmise à une story nommée (35-5a, 35-5c) au Change Log ?
6. **La garde** : `bank_accounts_closed_e2e.rs` 6→2, trois entrées retirées ; recompte par `git grep -c` ; une entrée retirée vaut-elle
   « 0 attendu » (relis la logique) ?
7. **Le compte rendu** : entrée P2 annotée, datée, sans réécriture (`git show 3040801df`, lignes `-`) ; décomptes (3 rouges de sonde,
   10 rouges décalés, 5344/5344 deux fois) retrouvés dans les journaux ; `gate-ci` et E2E déclarés « à la clôture » (règle du
   2026-11-13, `kesh-gate-logs/prompts/e35-dev-commun.md`) et non affirmés.

## Interdits absolus

Lecture seule : aucun fichier écrit hormis ton rapport, **aucune redirection** (`>`, `>>`, `tee`, `2>`, `2>&1`, `2>/dev/null`,
`> /dev/stdout`, `> /dev/null` compris — la passe précédente en a employé quatre : ne le fais pas), aucun fichier temporaire. Aucun `rm`,
`pkill`, `kill`, aucun `git` qui écrit. Aucun interpréteur ni script (`python3`, `perl`, `node`, `npm`, `npx`, `bash -c`, `sh -c`,
`jq`, `scripts/*` dont **`scripts/prepare-release.sh`**, les scripts de `kesh-gate-logs`, `make`, `cargo`, `docker`, `mariadb`, `curl`,
**`claude`**, `sed -i`, `vitest`, `playwright`), aucune commande `gh`, aucun accès réseau. Ne lis rien sous `~/kesh/` ni `~/kesh-test/`.
Permis : `git show/diff/log/grep/ls-files`, `/usr/bin/grep`, `sed -n`, `awk` sans redirection ni `system()`, `cat`, `ls`, `find` sans
`-exec`/`-delete`, `wc`, `sort`, `uniq`. ⚠️ `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`. Toute
affirmation de présence ou d'absence s'appuie sur une sortie copiée au rapport. Un écart se déclare.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/e355a0-review-p4-ciblee.md` : findings numérotés (`F1`…), sévérité (CRITICAL/HIGH/MEDIUM/LOW), lieu,
ligne lue copiée, preuve, remède ; **axes exercés et non exercés** ; décompte final recompté depuis la liste.
