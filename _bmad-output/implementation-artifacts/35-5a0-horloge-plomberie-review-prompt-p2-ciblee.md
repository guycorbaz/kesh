# Prompt — revue de code P2 ciblée de la Story 35-5a0 (refs #929, #1022, #907), une lentille, braquée sur la remédiation de la P1

Passe **ciblée**, une seule lentille, contexte frais, **Sonnet**, en lecture seule. Worktree `/home/gcorbaz/devel/kesh-e355a0`, branche
`story/35-5a0-horloge-plomberie`. La P1 (Opus, trois lentilles, prompt `35-5a0-horloge-plomberie-review-prompt-p1.md`) a rendu
3 MEDIUM et 14 LOW (rapports `/home/gcorbaz/devel/kesh-gate-logs/e355a0-review-p1-{B,E,A}.md`) ; B2 est tracé en #1105 et n'est pas à
rapporter de nouveau. Choix de la passe ciblée : la remédiation ne touche en production qu'une fonction (`instant_de_test`,
`crates/kesh-core/src/horloge.rs`) et deux commentaires ; le reste est du montage de test, de la garde et de la documentation.

## Périmètre : le seul commit `c8cda87e5`

`git -C /home/gcorbaz/devel/kesh-e355a0 show c8cda87e5` — treize fichiers. Le compte rendu est au Change Log et au Dev Agent Record de la
fiche `_bmad-output/implementation-artifacts/35-5a0-horloge-plomberie.md` (commit `6855b1a1f`, section « Transmis à la 35-5a »
comprise) et au registre `e35-choix-autonomes-e355a0.md` (annotation datée de C-e355a0-5 et C-e355a0-3). Inventaire de D4 :
`/home/gcorbaz/devel/kesh-gate-logs/e355a0/inventaire-d4-e355a0.py` (à **lire**, jamais à exécuter) et sa sortie `r1-inventaire-d4.txt`.

Il remédie :

- **E1/A2 (MEDIUM)** — des montages qui tenaient leur propre `today` (`Utc::now().date_naive()`) appelaient la fonction avec
  `aujourd_hui_test()` : deux dates du jour dans un même test. D4 de la fiche exige un seul jour, tiré de l'horloge de test.
- **A1 (MEDIUM)** — C-e355a0-5 affirmait des gates complets aux commits (2) à (4) qui n'ont pas tourné.

## Ce que tu vérifies

1. **`instant_de_test` (production)** : lis la fonction avant/après. Rogner la valeur de la variable d'environnement change-t-il un
   comportement hors des tests (la variable est-elle lue en production, par quel chemin, et une installation réelle peut-elle la
   porter) ? Une valeur faite de blancs est-elle désormais « absente » **partout** où la variable est lue (`git grep -n
   'instant_de_test\|NOM_VARIABLE_INSTANT\|env_nonempty' crates/`) — ou un lecteur la juge-t-il encore présente ? Le test ajouté
   rougirait-il sans le rognage ? Le décalage `+01:00` : le jour rendu est-il celui attendu (jour UTC, comportement inchangé de la
   35-5a0) ?
2. **D4, l'inventaire clos** : les quatre montages convertis (`invoice_settlement.rs` `monter` et `une_cloture_…`,
   `supplier_invoices_repository.rs` `monter_par_virement`, `opening_complement_repository.rs` `entrelacement_5`) tirent-ils
   **un seul** jour, passé à l'appel et aux assertions ? L'exercice d'`entrelacement_5` couvre-t-il la date pour tout `today`
   (bascule d'année) ? Reste-t-il un site où un montage tient son propre jour et l'appel en lit un autre : cherche **toi-même**, sans
   partir de la liste de l'agent — `git grep -nE 'Utc::now\(\)\.date_naive\(\)|Local::now\(\)' crates/*/tests/* crates/*/src/*` croisé
   avec `git grep -n 'aujourd_hui_test()'` dans les mêmes fonctions. Les exclusions déclarées (`lettering_documents_backfill.rs`,
   les `fn jour()` de pièce, `supplier_invoices_repository.rs:1646`, `:1927`, `:2976` dans `figer`) sont-elles justes au code ?
3. **`homepage_summary_e2e.rs`** : les 8 sites passent le `today()` du montage, resté à l'heure réelle — le test peut-il encore lire
   deux jours différents (autour de minuit) ? Le module `common::jour` retiré n'a-t-il plus aucun usage ni déclaration (`mod jour`)
   orpheline ?
4. **La garde** (`date_du_jour_garde.rs`) : les nouveaux comptes (invoice_settlement 4→2, supplier_invoices_repository 4→3,
   opening_complement_repository retiré ; `instant_de_test` 6→9, `HorlogeFixe` 5→6) se **recomptent** au code par `git grep -c` ;
   une garde qui compte encore juste par construction (les deux côtés du même extracteur) ? Les angles morts ajoutés (B3) disent-ils
   vrai ?
5. **A1, l'annotation** : C-e355a0-5 et C-e355a0-3 annotés sans réécrire l'entrée d'origine (`git show 6855b1a1f --
   _bmad-output/implementation-artifacts/e35-choix-autonomes-e355a0.md`, lignes `-`) ? L'affirmation « contenu `kesh-db` identique
   entre (4) et (5) » : rejoue `git diff --stat 5426cb8b 0deb9abc -- crates/kesh-db`.
6. **Les LOW traités** (A3, A4, A5, B1, B3, B4, B5/E2, E4, E5, E6) : chacun l'est-il au code, sans introduire de défaut ? B4 : le test
   juge-t-il le refus sur un texte stable, ou sur une phrase localisée susceptible de changer avec la locale de l'installation ?
7. **Les gates** au commit `c8cda87e5` : test-fast 5344, gate-ci 5351, `npm run build`, E2E 403/7 KF-029 (`mode-expert:26`, `:41`,
   `onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`, `:150`) — chaque chiffre au journal sous `kesh-gate-logs/e355a0/`,
   **postérieur** au commit ? Dispense de `check`/lint/Vitest : aucun fichier sous `frontend/` dans `c8cda87e5` (`git show --stat`) ?
   Le Change Log n'affirme-t-il que ce qui a tourné ?

## Interdits absolus

Lecture seule : aucun fichier écrit hormis ton rapport, aucune redirection (`>`, `>>`, `tee`, `2>`, `2>&1`, `2>/dev/null`, `> /dev/null`
compris), aucun fichier temporaire. Aucun `rm`, `pkill`, `kill`, aucun `git` qui écrit. Aucun interpréteur ni script (`python3`, `perl`,
`node`, `npm`, `npx`, `bash -c`, `sh -c`, `jq`, `scripts/*` dont **`scripts/prepare-release.sh`**, les scripts de `kesh-gate-logs`,
`make`, `cargo`, `docker`, `mariadb`, `curl`, **`claude`**, `sed -i`, `vitest`, `playwright`), aucune commande `gh`, aucun accès réseau.
Ne lis rien sous `~/kesh/` ni `~/kesh-test/`. Permis : `git show/diff/log/grep/ls-files`, `/usr/bin/grep`, `sed -n`, `awk` sans
redirection ni `system()`, `cat`, `ls`, `find` sans `-exec`/`-delete`, `wc`, `sort`, `uniq`. ⚠️ `grep` est une fonction vers `ugrep` :
emploie `/usr/bin/grep` ou `git grep`. Toute affirmation de présence ou d'absence s'appuie sur une sortie copiée au rapport. Un fichier
créé par erreur se déclare.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/e355a0-review-p2-ciblee.md` : findings numérotés (`C1`…), sévérité (CRITICAL/HIGH/MEDIUM/LOW), lieu,
ligne lue copiée, preuve, remède ; **axes exercés et non exercés** ; décompte final recompté depuis la liste.
