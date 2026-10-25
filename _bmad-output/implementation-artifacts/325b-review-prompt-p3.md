# Prompt — revue de code P3 de la Story 32-5b : passe ciblée, regression hunter, Sonnet

Tu relis **une seule chose** : la remédiation de la revue de code P2 de la partie b de la Story 32-5 de Kesh
(comptabilité suisse, Rust/Axum, MariaDB), qui porte sur l'authentification par clé MCP. Worktree
`/home/gcorbaz/devel/kesh-32-5b`, branche `story/32-5b`. Le commit de code de cette remédiation est **`e64ef129`** ;
le diff à relire, **aplati** :

```
git -C /home/gcorbaz/devel/kesh-32-5b diff e2557b47 e64ef129 -- crates
```

(`631bb6a1` et `709f2f0d`, entre les deux, ne portent que des artefacts de `_bmad-output/`.) Langue du rapport :
français. Modèle : **Sonnet**. Contexte frais : ne t'appuie sur aucune passe antérieure, sauf pour savoir ce que la
remédiation prétendait corriger.

**Le motif que tu chasses** : *la sévérité se déplace vers ce qu'on vient d'écrire*. La question n'est pas « la story
est-elle juste ? » mais « **cette remédiation a-t-elle introduit un défaut, affaibli une garde, fait passer un test à
vide, ou laissé un texte dire autre chose que le code ?** ». La passe P2 a précisément trouvé un test rendu muet par la
remédiation P1 : regarde d'abord de ce côté.

Ce que la remédiation prétend faire (rapport `/home/gcorbaz/devel/kesh-gate-logs/325b-review-p2.md`, lecture seule) :

- **R1 (MEDIUM)** — `societe_absente_refusee` (`crates/kesh-api/tests/cles_mcp_e2e.rs`) était refusé par la condition
  `u.company_id = k.company_id` (C-32-5-43) et non par la société absente : le créateur y est désormais un
  administrateur actif orphelin de la **même** société absente `999999`, inséré sur la même connexion sous
  `FOREIGN_KEY_CHECKS = 0` ; la mutation mx5 (`JOIN companies` retiré) est rejouée sur `cles_mcp_e2e` **et**
  `mcp_keys_repository` (`/home/gcorbaz/devel/kesh-gate-logs/325b-p2r-mutations.log`) ;
- **R2 (LOW)** — les textes qui énumèrent les refus nomment le créateur d'une autre société : `cles_mcp.rs` (doc du
  module, point 2 ; doc d'`authentifier` ; libellé du journal), `mcp_keys.rs` (doc du module), AC-b3 de la fiche
  annoté, C-32-5-41 du registre annoté ;
- **R3 (LOW)** — le doc-comment de `jetons_hostiles_avec_le_prefixe`.

Gates déclarés : `325b-p2r-cible.log`, `325b-p2r-gate-complet.log`, l'E2E `325b-p2r-e2e.log`.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire** ;
  **aucune redirection** (`>`, `>>`, `tee`). Un fichier créé par erreur se **déclare** au rapport ; il ne se supprime pas.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git restore`), aucun `git` qui écrit (`commit`,
  `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`).
- **N'exécute rien qui écrit, compile ou exécute du code** : ni **`scripts/prepare-release.sh`**, ni
  `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/regen-test-schema.sh`, ni aucun script de
  `kesh-gate-logs` ou d'un scratchpad, ni `make`, ni `cargo` (aucune sous-commande), ni `rustc`, ni `npm`/`npx`/`node`,
  ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni `curl`, ni `python3`, ni `sed -i`, ni `perl`.
- **Aucune commande `gh`**, pas même en lecture.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `grep`, `sed -n`, `awk` (sans redirection), `cat`, `ls`,
  `find`, `wc`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée.
- **Données réelles** : aucune. Ne lis rien sous `~/kesh-test/`.

## Les axes (une seule lentille : regression hunter)

1. **`societe_absente_refusee` réécrit** : l'`INSERT INTO users`, l'`INSERT INTO mcp_keys` et les deux
   `SET FOREIGN_KEY_CHECKS` passent-ils par la **même** connexion (`&mut *conn`) ? `last_insert_id()` est-il lu sur la
   requête `users` ? Le créateur passe-t-il toutes les autres conditions (actif, `Admin`, même société que la clé), si
   bien que **seule** la jointure à `companies` refuse ? Le `username` `admin-orphelin` peut-il entrer en collision avec
   un autre utilisateur du même test (contrainte `uq_users_username`) ? Le test peut-il encore passer à vide pour une
   autre raison (le refus venant d'une erreur d'insertion avalée, de la portée, du préfixe) ? La mutation mx5 rougit-elle
   **ce** test, et lui seul pour la bonne raison (`325b-p2r-mutations.log`) ?
2. **Les autres montages « orphelins » ou « société absente »** : `git grep -nE "999_?999|orphel|absente" e64ef129 --
   crates/kesh-api/tests crates/kesh-db/tests` ; chacun prouve-t-il encore ce que dit son nom après C-32-5-43 (les tests
   de clés étrangères de `mcp_keys_repository.rs`, `INEXISTANT` des tests de services, le montage #528 d'
   `admin_full_import_e2e.rs` qui passe par `api_keys`) ?
3. **Les textes R2** : chaque énumération des refus dit-elle maintenant la même liste, dans le code (`cles_mcp.rs`,
   `mcp_keys.rs`), la fiche (AC-b3, annotation datée) et le registre (C-32-5-41, annotation) ? Une énumération reste-t-elle
   ailleurs sans le cas neuf (`git grep -n "orpheline\|orphan" e64ef129 -- crates/kesh-api/src crates/kesh-db/src`) ? Le
   libellé du journal (`refus(...)`) reste-t-il sans condensat ni jeton ?
4. **R3** : le doc-comment de `jetons_hostiles_avec_le_prefixe` dit-il ce que le test prouve, ni plus ni moins ?
5. **Ce qui n'a pas bougé et aurait dû** : l'entrée « Revue de code P2 de la 32-5b » du Change Log de la fiche
   (`32-5-cles-et-reglages-mcp.md`) — décomptes, gates, mutations, « valeurs grepées » — est-elle vraie à `e64ef129` et
   contre les journaux ? Le binaire de l'E2E a-t-il été bâti après `e64ef129` ?

## Rapport

Écris ton rapport dans **`/home/gcorbaz/devel/kesh-gate-logs/325b-review-p3.md`** :

- bilan en tête : CRITICAL / HIGH / MEDIUM / LOW ;
- findings numérotés `S1…`, chacun avec sévérité, fichier:ligne, **preuve copiée** (sortie `grep -nF`, `sed -n` ou
  `git diff`), et, s'il vient de la remédiation, le hunk qui l'introduit ; correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** — un « 0 finding » sans cette liste ne
  compte pas comme passe ;
- rien d'autre n'est écrit nulle part.
