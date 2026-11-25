# Prompt — revue de code P2 de la Story 48-12a (#975), passe complète, Sonnet

Tu relis la **Story 48-12a** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5, MariaDB), release **0.20.0**, Epic 48 :
**le refus serveur des factures fournisseur en double**. C'est la **deuxième passe** de la revue de code. La première (P1, trois
lentilles Opus) a rendu **0 CRITICAL, 0 HIGH, 2 MEDIUM, 16 LOW** ; tout a été remédié. Cette passe est **complète** : elle
relit la story entière, et pas seulement la remédiation, car celle-ci touche le serveur, les tests, la fiche et un catalogue.

Modèle : **Sonnet**, contexte frais, un seul agent qui exerce les trois regards (B : le diff seul ; E : chaque branche et
chaque borne ; A : la fiche contre le livré). Langue du rapport : français.

Worktree `/home/gcorbaz/devel/kesh-48-12`, branche `story/48-12-reponses-perdues-fournisseurs`, base `a8ffbb488`.

## À lire d'abord : le commit de remédiation

`git -C /home/gcorbaz/devel/kesh-48-12 show 49df1f985`. Il contient : la lecture des candidates **bornée à l'année civile**
(`guard_duplicates_in_tx`, `BETWEEN` du 1er janvier au 31 décembre) ; un doc-comment sur le cycle de deux repassages ;
plusieurs tests neufs ou durcis (bornes, numéro normalisé au `PUT`, plus petit identifiant de la règle 2, annulée rapprochée,
projet et non-fournisseur, complétion par clé, casse) ; et l'italien passé au tutoiement. Le commit de la fiche, qui suit,
porte le Dev Agent Record et le Change Log de la P1. Les rapports de la P1 sont à lire en lecture seule :
`/home/gcorbaz/devel/kesh-gate-logs/e4812/review-12a-p1-{B,E,A}.md`.

Puis le diff entier de la story, **aplati** : `git -C /home/gcorbaz/devel/kesh-48-12 diff a8ffbb488 49df1f985 -- crates docs README.md frontend/src/lib/shared/utils`
(les commits de la 48-12b, `16fc5742b`, `3a10dba62`, `f7280c030`, `f35c56925`, `cf38f506f`, ne te concernent que pour les textes
partagés et pour `f35c56925`, le décor E2E qu'a cassé la règle 2). Lis aussi la fiche
`_bmad-output/implementation-artifacts/48-12a-doublons-fournisseurs-serveur.md` en entier.

Ne prends pas le Dev Agent Record pour argent comptant. Recoupe-le avec les journaux hors dépôt (lecture seule), sous
`/home/gcorbaz/devel/kesh-gate-logs/e4812/dev/` :
- `mut-12a/*.log`, `mut-12a/resume*.log` ;
- les scripts `mut-12a/muter.py` et `mut-12a/muter-p1.py` (lis-les, ne les lance pas) ;
- les journaux de gate nommés au Dev Agent Record ;
- `e2e-complet.log` et `e2e-complet-2.log`.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part ;
  **aucune redirection** (`>`, `>>`, `<`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par erreur se
  **déclare**, il ne se supprime pas.
- **Aucun `rm`**, sous aucune forme ; ni `pkill`, ni `killall`, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`/`commit`/`reset`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code.** Sont interdits :
  - `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ;
  - **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`,
    `scripts/mem-guard.sh`, et tout autre script du dépôt ou de `kesh-gate-logs` (`muter.py`, `muter-p1.py`, `reset-bases.sh`,
    `e2e-e4812.sh` compris) ;
  - `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis :
  - `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e` ;
  - `/usr/bin/grep`, `sed -n`, `awk` (sans redirection ni `system()`) ;
  - `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, `tr` ;
  - pour le contrôle de PDF, seule forme admise : `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'` ;
  - pour le `.tex` aplati, sans redirection : `cat <f>.tex | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport. Tout chemin
  cité se vérifie (`git cat-file -e <ref>:<chemin>` ou `ls`).

## Les huit axes obligatoires

Ce sont les huit axes du prompt de la P1 (`48-12a-doublons-fournisseurs-serveur-review-prompt-p1.md`, à côté de la fiche ; lis-le
pour leur détail). Chacun s'exerce à `49df1f985` :

1. **AC et décisions de Guy** : D1 à D7, D6bis, D3ter. La borne d'année change-t-elle une décision ? La règle 1 compare-t-elle
   encore « même année civile, jamais l'exercice » ? La règle 2, « même date » ? Les candidates du 1er janvier et du 31 décembre
   comptent-elles ? `find_duplicate` reste-t-elle la seule décision ? `DbError::Invariant` est-il atteignable ?
2. **Relevés A à D par le symptôme** :
   - A : le cycle de deux repassages, écrit à la fiche (D3, relevé A ligne 4) et au doc-comment « Ordre des verrous », est-il
     exact ? Un autre cycle neuf existe-t-il ?
   - B : rejoue la commande gawk de la fiche.
   - Les lectures avant le verrou : la borne d'année ne doit en ajouter aucune.
3. **Rejeu de `POST` et `PUT /invoices`** : inchangé depuis la P1 ? Aucune facture client, aucun avoir ni PDF au diff.
4. **Textes partagés** : les trois clés `error-supplier-invoice-*` aux quatre catalogues, égales aux replis Rust. L'italien est
   désormais au tutoiement (glossaire `docs/i18n-glossaire.md`). La propagation aux clés italiennes de la 48-12b est-elle
   complète et juste ? Reste-t-il, dans les clés neuves des deux fiches, un vouvoiement italien ?
5. **Mutations** : chaque mutation N01 à N15 de `muter-p1.py` est-elle tuée par le test qu'elle vise, et compilée (`Starting`
   présent, pas d'`error[E`) ? Les cinq tests qui n'avaient aucun rouge prouvé à la P1 en ont-ils un maintenant ? Une mutation
   plausible reste-t-elle non jouée sur le code neuf (la borne d'année, par exemple : année de l'échéance, `year - 1`) ?
6. **Manuels** : `user-manual.tex`, `admin-manual.tex`, `docs/api-external.md`, chaque phrase contre le code. Contrôle négatif et
   positif, sur le `.tex` aplati **et** sur le PDF aplati (commande de la fiche rejouée). Le PDF versionné n'est pas régénéré : c'est
   un point d'intégration, non un défaut. La borne d'année est-elle dite là où le manuel parle du coût ou de l'historique ?
7. **Factures réelles existantes** : aucune n'est refusée à la relecture. La borne d'année ne doit rien changer à un doublon
   préexistant modifié sans changer les quatre éléments. Aucune migration.
8. **Captures** : aucune n'est due pour la 48-12a.

Et la fiche contre le livré (regard A) :
- les AC ;
- le tableau « Tests à écrire » ;
- les écarts déclarés, dont A1 (relevé des fixtures sans E2E, lignes citées de `e2e-complet.log`) ;
- le décompte des tests (30, annoté) ;
- le statut `review` en tête, aux Dev Notes et à `sprint-status.yaml` ;
- l'entrée P1 du Change Log : tendance, modèles, reclassements ;
- les issues #1415 et #1416 citées aux Dev Notes.

## Rapport

Écris `/home/gcorbaz/devel/kesh-gate-logs/e4812/review-12a-p2.md`. Il contient :
- les findings numérotés (`P2-1`…), chacun avec sa sévérité (CRITICAL / HIGH / MEDIUM / LOW), son lieu (`fichier:ligne`), la
  ligne lue copiée, la preuve et le remède proposé ;
- **la liste des huit axes, exercés ET non exercés**, un par un ;
- le décompte final, **recompté depuis la liste**.

Un « 0 finding » non adossé à cette liste ne compte pas comme passe.
