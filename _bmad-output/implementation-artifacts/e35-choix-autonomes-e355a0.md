# Story 35-5a0 — choix autonomes de l'agent de développement (clé `e355a0`)

*Registre ouvert le 2026-11-13 par l'agent de développement (Opus 5.5), worktree `kesh-e355a0`. Mise à jour par union
(C-e350-8).*

### C-e355a0-1 — Les gates de chaque commit tournent sur un clone, non dans le worktree

- **Contexte** : la fiche exige le gate complet à chaque commit de code (exception `kesh-db`). La voie base est partagée
  et la file d'attente du verrou peut durer ; un gate lancé dans le worktree lit les fichiers au moment où il compile, si
  bien que tout travail du commit suivant pendant l'attente fausserait le gate du commit précédent.
- **Option retenue** : un clone local `kesh-gate-logs/e355a0/gwt` (`git clone --no-hardlinks` du worktree, aucune écriture
  dans le dépôt principal), positionné par `git fetch` + `git checkout <commit>` sur le commit à juger ; sa cible cargo lui
  est propre. Les journaux disent le commit jugé (première ligne).
- **Autres options** : travailler en série (gate puis commit suivant), coûteux en attente ; `git worktree add`, qui écrit
  dans le `.git` du dépôt principal.
- **Réversibilité** : totale ; le clone est hors dépôt, à retirer avec le reste de `kesh-gate-logs/e355a0/`.

### C-e355a0-2 — Les reformatages passent par `cargo fmt`

- **Contexte** : consigne « Edit seulement » ; les lignes allongées par l'argument ajouté sortent du gabarit de rustfmt.
- **Option retenue** : écrire par l'outil Edit, puis laisser `cargo fmt` recouper les lignes (outil de formatage du
  dépôt, que la CI contrôle) ; déclaré au Change Log.
- **Réversibilité** : sans objet (sens inchangé, `cargo fmt --check` le juge).

### C-e355a0-3 — Les 311 appels de tests de la fermeture sont convertis par un script, depuis le compilateur

- **Contexte** : le commit (3) change la signature de la fermeture ; `cargo build --all-targets` rend **336 E0061** (311
  sites distincts, les fichiers `support/` étant inclus par plusieurs binaires), dans **33** fichiers de tests. La consigne
  est « Edit seulement » ; le prompt de l'orchestrateur permet de s'en écarter pour une conversion mécanique, à déclarer.
- **Option retenue** : `kesh-gate-logs/e355a0/conv-e355a0.py` lit les diagnostics JSON E0061 (la suggestion `/* NaiveDate */`
  donne l'étendue exacte des parenthèses de l'appel) et ajoute en **dernier argument** le seul helper de jour de la crate
  (`crate::jour::aujourd_hui_test()`, `crate::common::jour::aujourd_hui_test()`, `jour::aujourd_hui_test()` dans
  `support/`) — jamais une date de pièce (D4), par construction : le script n'insère rien d'autre. Il refuse tout site hors
  `crates/*/tests`, toute erreur d'un autre code, tout appel dont la dernière ligne d'argument porte un commentaire (aucun
  refus rendu). `decl-e355a0.py` déclare le helper (26 fichiers) avant la première ligne `use`. Puis `cargo fmt`. La
  production — les 18 appels de `kesh-api` que rend le compilateur, les appels internes de `kesh-db`, les neuf sites — et
  les deux appels des `mod tests` de `journal_entries.rs` sont édités à l'outil Edit.
- **Autres options** : 311 éditions à l'outil Edit (plus d'une centaine de fichiers lus et relus, risque d'oubli ou de
  coquille plus grand que celui d'un script borné par le compilateur) ; la suggestion de rustc appliquée telle quelle (elle
  réécrit la liste d'arguments et perdrait les commentaires).
- **Réversibilité** : totale (`git revert` du commit (3)) ; les deux scripts restent dans `kesh-gate-logs/e355a0/`.
- *(Annoté le 2026-11-13, remédiation de la revue P1, E1 et A2 : le script ne lisait pas la portée, si bien que D4 —
  « la date du jour que le montage tient déjà » — n'a pas été suivie là où la fonction englobante tenait son jour. Inventaire
  clos (`inventaire-d4-e355a0.py`, lecture seule, sur les 311 sites de `c3-erreurs.json`, fonction englobante relue à
  `05666dbf`) et verdicts au Change Log de la fiche ; sites corrigés à l'outil Edit.)*

### C-e355a0-4 — Les deux copies de `support/jour.rs` d'un même binaire : `allow(clippy::duplicate_mod)`

- **Contexte** : la fiche (D4) prévoit qu'un binaire qui inclut `jour.rs` au premier niveau **et** un fichier `support/`
  qui l'inclut lui-même en a deux copies « sans conflit ». Le compilateur l'accepte ; `clippy -D warnings` non
  (`duplicate_mod`, constaté sur `lettering_documents_backfill`).
- **Option retenue** : `#[allow(dead_code, clippy::duplicate_mod)]` sur la déclaration imbriquée des deux fichiers
  `support/` (`lettering_documents.rs`, `fournisseur_rapproche.rs`), avec sa raison en commentaire.
- **Autres options** : passer la date en paramètre aux helpers `support/` (repli de la fiche), qui ferait changer la
  signature de `payer_par`, `facture_payee` et de leurs appelants ; ou rendre le module imbriqué `pub(crate)` et y renvoyer
  les binaires, ce qui les couple au fichier `support/` qu'ils incluent.
- **Réversibilité** : totale.

### C-e355a0-5 — Le gate de chaque commit : backend complet ; front et E2E au dernier commit de code

- **Contexte** : la fiche veut « chacun compile et passe le gate complet » ; le prompt, le gate complet à chaque commit de
  code (exception `kesh-db`) et le frontend et l'E2E au dernier commit de code.
- **Option retenue** : `test-fast.sh --ci` **et** `scripts/gate-ci.sh` aux commits (2), (3) et (4) ; frontend complet et
  E2E complet au commit (4), dernier commit de code (le commit de la fiche ne touche que `_bmad-output/`). Le commit (1)
  (`kesh-core` seul, un module que rien n'appelle) est couvert par le gate du commit (2), qui le contient, et par
  `cargo test -p kesh-core` à son commit. Aucun commit ne touche le frontend.
- **Réversibilité** : sans objet.
- *(Annoté le 2026-11-13, remédiation de la revue P1, A1 et A6 : l'option ci-dessus, écrite avant les gates, **n'a pas
  été tenue** ; elle reste telle quelle comme relevé de l'intention. **Ce qui a tourné, commit par commit** : (1)
  `48b9e627` — `cargo test -p kesh-core horloge` (6 passés), `clippy -p kesh-core` ; (2) `05666dbf` — `test-fast.sh --ci`
  seul (5333 exécutés, 5331 passés, 2 échecs `configuration_transmise`, 13 ignorés) ; `gate-ci.sh` **pas lancé** (la
  prise de verrou a été retirée de la file ; `c2-gate-ci.log` vide, 0 octet) ; (3) `5426cb8b` et (4) `79ec67b4` —
  **aucun gate complet** : `fmt --check`, `clippy --workspace --all-targets -D warnings`, compilation complète, garde
  `date_du_jour_garde` exécutée ; les suites `test-fast` et `gate-ci` préparées pour eux (`suite-e355a0.sh`) ont été
  arrêtées avant d'obtenir le verrou ; (5) `0deb9abc` — **gate complet** : `test-fast.sh --ci` 5343/5343, `gate-ci.sh` 5350
  passés, frontend complet, E2E complet (403 passés, les 7 KF-029), mutations 8/8. **Exception `kesh-db` (A6)** : elle
  n'a pas été suivie aux commits (3) et (4), qui touchent `kesh-db` (le (3) ses dépôts et ses tests) ; leur contenu
  `kesh-db` est **identique** à celui du (5) (`git diff --stat 5426cb8b 0deb9abc -- crates/kesh-db` ne rend rien), si bien
  que le gate complet du (5) le juge, mais l'historique porte trois commits ((2) à (4)) aux deux rouges de
  `configuration_transmise`, et deux ((3), (4)) sans gate complet propre : à garder en tête à la fusion (squash
  recommandé, ou bisect à éviter sur cet intervalle). Le gate de la remédiation P1 est noté au Change Log de la fiche.)*

### C-e355a0-7 — `horloge_de_test` lit par `config::env_nonempty` ; `KESH_TEST_INSTANT` connue de `configuration_transmise`

- **Contexte** : le gate du commit (2) a rendu deux rouges de `kesh-api/tests/configuration_transmise.rs`, que la fiche
  n'avait pas vus : (L) toute lecture de l'environnement de `crates/*/src` passe par `config::env_nonempty` ou figure à la
  liste fermée `EMPLACEMENTS_AUTORISES` — or D3 écrit `std::env::var(NOM_VARIABLE_INSTANT)` ; (F) tout jeton `KESH_…`
  d'un littéral du code de production nomme une variable connue — or `NOM_VARIABLE_INSTANT` est un littéral de
  `kesh-core/src`, que rien ne lit au sens de la garde.
- **Option retenue** : `horloge_de_test()` lit par `crate::config::env_nonempty(NOM_VARIABLE_INSTANT)` (la seule fonction
  qui lise, C75 ; vide ou blanche → absente, ce que voulait D1), inscrit à `EMPLACEMENTS_AUTORISES` sous sa forme exacte ;
  liste fermée neuve `VARIABLES_DE_TEST` (lue sur la constante, non recopiée), ajoutée aux variables connues de (F), avec
  son contrôle de la raison (`variables_de_test_hors_distribution` : absente de `.env.example` et des deux compose). La
  garde de la date du jour compte `NOM_VARIABLE_INSTANT` 2 dans `configuration_transmise.rs` (forme exacte et liste),
  écart assumé au « 0 ailleurs » de D5, qui visait `main.rs` et `lib.rs`. Commit (5), à part : les commits (2) à (4)
  portent ces deux rouges, que le gate du commit (5) juge clos.
- **Autres options** : `KESH_TEST_INSTANT` dans l'ensemble lu (il faudrait alors le transmettre par les compose ou
  l'exempter de (T) et de (E), et le documenter dans `.env.example` : contraire à « variable de tests ») ; réécrire les
  commits (2) à (4) (`git rebase`, interdit).
- **Réversibilité** : totale.

### C-e355a0-6 — La base E2E est migrée, non seedée par `seed-dev-db.sql`

- **Contexte** : le prompt dit « migrations du worktree, puis `scripts/seed-dev-db.sql` » pour les trois bases.
- **Option retenue** : `kesh_e355a0` migrée **et** seedée ; `kesh_e2e_e355a0` migrée seulement — le `globalSetup` de
  Playwright appelle `seedTestState` (`POST /api/v1/_test/seed`), qui vide et remplit la base lui-même ; recette reprise
  du script de gate de la 35-3 (`kesh-gate-logs/e353/reset-db-e353.sh`), qui a rendu les sept échecs attendus.
  `kesh_ci_e355a0` est recréée par `gate-ci.sh` lui-même (migrations, seed).
- **Réversibilité** : totale.
