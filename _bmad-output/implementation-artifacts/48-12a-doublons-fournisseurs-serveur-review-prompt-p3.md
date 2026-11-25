# Prompt — revue de code P3 de la Story 48-12a (#975), passe complète, Opus

Tu relis la **Story 48-12a** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5, MariaDB), release **0.20.0**, Epic 48 :
**le refus serveur des factures fournisseur en double**. C'est la **troisième passe** de la revue de code.

- **P1** (trois lentilles Opus) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 16 LOW ; remédiée par `49df1f985`.
- **P2** (Sonnet) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 4 LOW (`/home/gcorbaz/devel/kesh-gate-logs/e4812/review-12a-p2.md`) ; remédiée
  (mutations N05 à N18 jouées et journalisées, gate complet à `49df1f985`, marqueurs et décomptes de la fiche, Change Log, Pattern 5).
  Les deux MEDIUM portaient sur la **preuve** (mutations sans journal, gate absent), non sur le comportement.

Cette passe est **complète** (un MEDIUM a été rendu à la P2 : règle de `CLAUDE.md`, § *Review Iteration Rule*). Modèle : **Opus**,
contexte frais, **un seul agent qui exerce B, E et A sur toute la story et sur la remédiation P2** (B : le diff seul, sans la
fiche ; E : chaque branche, chaque borne, chaque verrou ; A : la fiche contre le livré). Langue du rapport : français.

Worktree `/home/gcorbaz/devel/kesh-48-12`, branche `story/48-12-reponses-perdues-fournisseurs`, base `a8ffbb488`.

## À lire d'abord

1. La remédiation P2 : `git -C /home/gcorbaz/devel/kesh-48-12 log --oneline 145dda094..HEAD` puis `git show` de chacun de ces commits.
   Elle ne touche pas le code de production de `kesh-db` ; elle ajoute le tableau des mutations N01 à N18, les journaux de gate,
   des corrections de la fiche et une phrase du Pattern 5.
2. Le diff entier de la story, **aplati** :
   `git -C /home/gcorbaz/devel/kesh-48-12 diff a8ffbb488 HEAD -- crates docs README.md frontend/src/lib/shared/utils`
   (les commits de la 48-12b ne te concernent que pour les textes partagés et `f35c56925`, le décor E2E).
3. La fiche `_bmad-output/implementation-artifacts/48-12a-doublons-fournisseurs-serveur.md` en entier, puis les rapports en lecture
   seule : `/home/gcorbaz/devel/kesh-gate-logs/e4812/review-12a-p1-{B,E,A}.md` et `review-12a-p2.md`.

Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec les journaux hors dépôt (lecture seule), sous
`/home/gcorbaz/devel/kesh-gate-logs/e4812/dev/` : `mut-12a/*.log` (dont `N01` à `N18`, `resume-p2-n05-n18.log`), les scripts
`mut-12a/muter.py` et `muter-p1.py` (lis-les, ne les lance pas), les journaux de gate nommés au Dev Agent Record
(section *Gates*), `e2e-complet*.log`.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris aucun fichier hors ton rapport ; **aucun fichier temporaire** ; **aucune redirection** (`>`, `>>`,
  `<`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**, il ne se supprime pas.
- **Aucun `rm`**, ni `pkill`, `killall`, `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`/`commit`/`reset`**, aucun `git` qui écrit.
- **Rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ; ni
  **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`, `scripts/mem-guard.sh`,
  ni aucun script du dépôt ou de `kesh-gate-logs` (`muter.py`, `muter-p1.py`, `reset-bases.sh`, `e2e-e4812.sh` compris) ;
  ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, `claude`, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**, aucun accès réseau.
- Permis : `git diff|show|log|grep|ls-files|cat-file -e` ; `/usr/bin/grep`, `sed -n`, `awk` (sans redirection ni `system()`) ;
  `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, `tr` ; pour un PDF :
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'` ; pour le `.tex` aplati :
  `cat <f>.tex | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport ; tout chemin
  cité se vérifie (`git cat-file -e <ref>:<chemin>` ou `ls`). Tout finding CRITICAL ou HIGH qui affirme l'absence d'un code attendu
  se vérifie par lecture directe avant d'être posé.

## Les axes obligatoires (à déclarer exercés ou non)

1. **AC et décisions de Guy** : D1 à D7, D6bis, D3ter ; règle 1 par année civile (jamais l'exercice), règle 2 par date égale,
   règle 3, annulées exclues, numéro facultatif, données existantes intactes ; `find_duplicate` seule décision ;
   `DbError::Invariant` inatteignable ; la lecture des candidates bornée à l'année ne change aucune décision.
2. **Relevés A à D par le symptôme** : A (tables enfants de `contacts`, cycles, y compris les deux repassages d'annulées par un
   exercice postérieur, désormais écrit au Pattern 5) ; B (commande gawk de la fiche rejouée à la main : le décompte de paragraphes
   est celui de HEAD, expliqué) ; lectures ordinaires avant le verrou du fournisseur sur les quatre chemins.
3. **Rejeu de `POST` et `PUT /invoices`** : `retry_on_deadlock`, témoins `exiger_un_rejeu`, prose du registre ; aucune facture
   client, aucun avoir, aucun PDF au diff.
4. **Textes partagés** : trois clés `error-supplier-invoice-*` aux quatre catalogues, égales aux replis Rust ; italien au
   tutoiement, y compris les clés de la 48-12b.
5. **Mutations** : le tableau N01 à N18 de la fiche contre chaque journal (verdict, assertion qui tue, compilation : `Starting`
   présent, pas d'`error[E`) ; les mutants équivalents (borne d'année trop large) sont-ils justifiés ? Une mutation plausible
   reste-t-elle non jouée ? Les décomptes (« 38 », « N01 à N18 », totaux) se recomptent depuis les journaux.
6. **Gates** : la section *Gates* de la fiche n'affirme que ce qui a tourné ; chaque chiffre cité se retrouve au journal nommé, avec
   le commit ; le « dernier commit de code » est le bon (`git log --format=%h -- crates`).
7. **Manuels** : `docs/manual/fr/user-manual.tex`, `admin-manual.tex`, `docs/api-external.md`, `README.md`, chaque phrase contre
   le code ; contrôle négatif et positif sur le `.tex` aplati **et** sur le PDF aplati (`pdftotext`). Le PDF versionné n'est pas
   régénéré hors release : point d'intégration, non défaut. Dis-le, avec les chiffres.
8. **Factures réelles existantes** : aucune n'est refusée à la relecture ; aucune migration.
9. **Captures** : aucune n'est due pour la 48-12a.

Et la fiche contre le livré (regard A) : les AC ; le tableau « Tests à écrire » (chaque ligne a son test, nommé) ; les écarts
déclarés ; le décompte des tests ; le statut en tête, aux Dev Notes (`sprint-status.yaml` est posé par l'orchestrateur) ; les
entrées P1 et P2 du Change Log (tendance, modèles, sort de chaque finding) ; les issues #1415 et #1416.

## Rapport

Écris `/home/gcorbaz/devel/kesh-gate-logs/e4812/review-12a-p3.md` : les findings numérotés (`P3-1`…) avec sévérité, lieu
(`fichier:ligne`), ligne lue copiée, preuve et remède ; **la liste des axes, exercés ET non exercés**, un par un ; le décompte final,
recompté depuis la liste. Un « 0 finding » sans cette liste ne compte pas comme passe.
