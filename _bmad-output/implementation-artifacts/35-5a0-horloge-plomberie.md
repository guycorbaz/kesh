# Story 35-5a0 : l'horloge injectable, posée sans rien changer — `Horloge`, `HorlogeFixe`, `AppState.horloge`, `Lecture` à horloge, la date du jour passée à `kesh-db`, la garde de l'inventaire (#929, #1022, #907)

Status: ready-for-dev

*Fiche rédigée le 2026-11-12 par l'agent de rédaction des fiches 35-5 (Opus 5.5), worktree `kesh-e355`, à la remédiation de
la validation P3 de la 35-5a (V3-4) : **découpage** décidé par l'orchestrateur (C-e350-28) — la dérogation C-e350-27 tombe,
sa prémisse (« le mécanisme ne se coupe pas ») étant fausse. La 35-5a0 est le *story-zero* de `CLAUDE.md` § *Règle de
splitting préventif* : elle pose le motif (plomberie de l'horloge, signatures, appelants, garde) **sans changer aucun
comportement** — la date passée partout est **le jour UTC d'aujourd'hui**, exactement celle d'avant. La **35-5a** part
d'elle fusionnée et bascule le jour sur Zurich. Release **0.17.7**. **Aucune migration**. Boucle de revue : `kesh-db`
touché, gate complet à chaque commit. Code lu sur **`release/0.17.6`** (`4e1c768a`, identique à `8424e36e` pour `crates/` et
`frontend/`). Registres : `e35-choix-autonomes-e350.md` (C-e350-16, -21, -24, -26, -28, -29) ;
`e35-choix-autonomes-e355a0.md` pour la story, à créer au premier choix.*

## Story

En tant que mainteneur, je veux que la date du jour du serveur vienne d'**une** horloge injectable — dans `AppState` et dans
la poignée `Lecture`, donc jusqu'au serveur MCP —, que `kesh-db` la reçoive en paramètre, et qu'une garde inventorie les
sites qui la prennent encore ailleurs, afin que la 35-5a n'ait plus qu'à changer le calcul du jour, et que les tests
puissent fixer l'heure.

## Ce qui se lit au code (`release/0.17.6`)

L'inventaire des 28 sites de production, des 8 commentaires et des tests est dans la fiche **35-5a** (§ *Ce qui se lit au
code*), qui fait foi pour les comptes ; la poignée `Lecture`, le câblage du MCP, `AppState` et ses 9 littéraux y sont décrits.

**La fermeture des appelants** (validation P3 de la 35-5a, V3-2 — partir du symptôme, non d'une liste) : les fonctions de
`kesh-db` qui lisent l'horloge pour une date (`journal_entries.rs:2935`, `settlement_cancellation.rs:172`, `companies.rs:1013`,
`:1100`) et **tout ce qui les atteint** :
- contre-passation : `reverse_in_tx`, `reverse_owned_in_tx`, `reverse` (`ReversalDating::Today`, `journal_entries.rs:2933-2938`) ;
- bloqueurs : `settlement_cancellation::settlement_entry_cancel_blocker` (définition `:87` ; c'est lui qui lit le jour à
  `:172`), `settlement_cancel_blocker`, `settlement_cancel_blocker_unlinking` (définitions `invoice_settlements_write.rs:826`,
  `:838`, `:912` ; `:878` est un appel), `reconciliation_cancel::{cancel_blocker (:250), entry_cancel_outlook (:321)}` ;
- écritures et vues des factures fournisseur (`supplier_invoices.rs`, **définitions**) : `supplier_invoice_cancel_blocker`
  (`:2426`), `cancel_in_tx` (`:2510`), `supplier_settlement_cancel_blocker` (`:2706`, appelé `:2839` et `:3055`), le bloqueur
  de `:2724`, `cancel_settlement_in_tx` (`:2778`), `get_settlement_view` (`:3013`, l'enveloppe), `get_settlement_view_in_tx`
  (`:3040`, calcule les deux bloqueurs `:3055-3056`), `commit_with_view` (`:3095` ; `:3109` est son appel interne), et donc
  **toutes** les écritures qui se terminent par lui — `validate` (`:1058`), `unvalidate` (`:1074`), `pay` (`:1899`), `cancel`
  (`:2680`), `cancel_settlement` (`:2963`). Les **appels** internes `:2462`, `:2750` (bloqueur) et `:2600`, `:2871`
  (`reverse_owned_in_tx`) reçoivent la date de leur appelant ;
- règlements client : `invoice_settlements_write::{cancel_settlement (:1134), cancel_settlement_in_tx (:1023)}` ;
- rapprochements : `reconciliation_cancel::{cancel (:804), cancel_in_tx (:564), get_view}` ;
- verrou : `lock_books`, `unlock_books`.

**Appelants de production** : `routes/companies.rs:505`, `:518` ; `routes/journal_entries.rs:456` ; `routes/supplier_invoices.rs:829`,
`:871`, `:904`, `:931`, `:960`, `:990` ; `routes/invoices.rs:1361` ; `routes/imported_supplier_invoices.rs:230`, `:646`, `:723` ;
`routes/reconciliation.rs:5799`, `:5938` et **`:6587`** (`get_entry_candidates`, une **lecture** qui passe par le rang 5) ;
`services/factures.rs:552` (`reglements`) ; `services/factures_fournisseur.rs:206` (dont le motif est jeté par
`detail_sans_fichier`, `:227-237`). `routes/supplier_invoices.rs:272` est `from_view`, une conversion, non un appelant. Les
fonctions intermédiaires des routes (`_once`, l'assistant de `routes/supplier_invoices.rs:829`, celles de
`routes/imported_supplier_invoices.rs`) changent aussi de signature ; `routes/onboarding.rs:1043` (site 17) est dans
`finalize_inner(pool, current_user)` (`:914`), qui ne reçoit pas `state` : sa signature reçoit la date, son seul appel (`:890`)
la lit sur `state.horloge`. Homonyme non concerné : `invoice_reminders::cancel_in_tx`.

**Appelants de tests** : `git grep -cE 'supplier_invoices::(validate|unvalidate|pay|cancel|cancel_settlement)\s*\(' -- 'crates/*/tests/*'`
rend **158 appels dans 23 fichiers** ; s'y ajoutent les appelants de `cancel_settlement(` (dont
`kesh-db/tests/{credit_notes_repository,invoice_amount_due_parity}.rs`, `kesh-report/tests/{aged_receivables,vat_report_write_off}.rs`),
de `reverse*`, des bloqueurs, de `lock_books`/`unlock_books` (dont deux dans `admin_full_import_e2e.rs`, outre `:1809`), de la
vue (`supplier_invoice_response_e2e.rs:434-496`) et de `reconciliation_cancel::*` ; une première passe sur l'ensemble des
noms de la fermeture rend **56 fichiers** (`src` compris). **Le compilateur donne la liste close** ; l'étape 0 la recompte
par `git grep` sur les noms ci-dessus et l'écrit au Change Log.

## Décisions (C-e350-21, -24, -26, -28, -29)

- **D1 — L'horloge, dans `kesh-core`** (C-e350-29) : module `kesh-core/src/horloge.rs`, sans I/O :
  - `pub trait Horloge: Send + Sync { fn maintenant(&self) -> DateTime<Utc>; fn aujourd_hui(&self) -> NaiveDate {
    jour(self.maintenant()) } }`, `pub struct HorlogeFixe(pub DateTime<Utc>)` ;
  - `fn jour(instant: DateTime<Utc>) -> NaiveDate { instant.date_naive() }` (privée) : **le jour UTC, comme aujourd'hui** — le
    seul `.date_naive()` du fichier. La 35-5a la remplace par `jour_suisse`, si bien que le serveur **et** les tests qui
    passent par l'horloge basculent ensemble, et que le compte de la garde du fichier reste 1 à chaque étape (validation P4
    de la 35-5a, V4-4) ;
  - `pub const NOM_VARIABLE_INSTANT: &str = "KESH_TEST_INSTANT";` — **le seul littéral** du nom dans le dépôt (V1-1) ; le
    message de panique, les helpers et les tests passent par la constante ;
  - la fonction pure `pub fn instant_de_test(valeur: Option<&str>) -> Option<DateTime<Utc>>` : `None` **et la chaîne vide**
    (`KESH_TEST_INSTANT=` exporté vide, V1-9) rendent `None` ; une valeur non RFC 3339 fait `panic!` en citant
    `NOM_VARIABLE_INSTANT`.
  Les tests du module comparent à des `NaiveDate::from_ymd_opt` fixes, sans `.date_naive()` (V1-10).
- **D2 — `kesh-db` ne lit plus l'horloge pour une date** : toutes les fonctions de la fermeture reçoivent `aujourd_hui:
  NaiveDate`, ajouté **en dernier argument** (la garde lexicale `services/factures.rs:603`, `"settlement_cancel_blocker(&muttx,"`,
  tient). `ReversalDating::Today` porte la date ou la reçoit. Une lecture de l'horloge par requête, passée au bloqueur **et** à la
  contre-passation.
- **D3 — `kesh-api`**, module `kesh-api/src/horloge.rs` :
  - `HorlogeSysteme` (`maintenant()` = `Utc::now()`), **défini** dans ce module, **construit** dans `main.rs` et dans
    `horloge_depuis` seulement ;
  - `pub fn horloge_depuis(valeur: Option<&str>) -> Arc<dyn Horloge>` (pure, V1-2) : `HorlogeFixe` à `instant_de_test(valeur)`
    s'il y en a un, `HorlogeSysteme` sinon ; et `pub fn horloge_de_test() -> Arc<dyn Horloge>` (`#[doc(hidden)]`) =
    `horloge_depuis(std::env::var(NOM_VARIABLE_INSTANT).ok().as_deref())`, lue à chaque appel, **sans `static`, `OnceLock`,
    `LazyLock`**. Le test porte sur `horloge_depuis` : **aucun `set_var`** (`unsafe` en édition 2024, `Cargo.toml:17`, interdit
    hors tests par `lib.rs:9`, et qui changerait la variable sous les pieds des autres tests du binaire). « Lue à chaque appel »
    tient à la garde (aucune mémoire) et à la forme de `horloge_de_test` (une ligne) ;
  - champ `pub horloge: Arc<dyn Horloge>` d'`AppState` : `HorlogeSysteme` dans `main.rs:439` ; `horloge_de_test()` dans
    `new_for_tests` et dans les littéraux des **tests d'intégration** (`invoice_frozen_pdf_e2e.rs:101`,
    `invoice_send_email_e2e.rs:104`, `:1036`, `:1150`, `password_recovery_e2e.rs:126`, `setup_admin_e2e.rs:82`, et le `Self {` de
    `new_for_tests`) — ainsi la variable atteint tous les états de test d'intégration (validation P3 de la 35-5a, V3-1) ;
    `middleware/auth.rs:285` (`test_state()`, dans un `mod tests` **de `src`**, `:198`) prend `Arc::new(HorlogeFixe(Utc::now()))`,
    comme tout `mod tests` de `src` (D4), pour ne pas appeler `horloge_de_test` depuis `src` (V1-1 bis) ;
  - `Lecture { pool, horloge }`, `Lecture::new(pool, horloge)` **sans défaut**, `aujourd_hui()`, `horloge()` ;
    `AppState::lecture()` : `services::poignees::Lecture::new(self.pool.clone(), self.horloge.clone())` ; câblage du MCP en trois
    lignes courtes (`let horloge_outils = state.horloge.clone();`, `let lecture_pool = services::poignees::Lecture::new(pool_outils.clone(),
    horloge_outils);`, `let lecture_outils = mcp::LectureOutils::nouvelle(lecture_pool);`) ; gardes réécrites :
    `mcp_gardes_lexicales.rs:169`, `:575`, `services_motif.rs` `FABRIQUES_V2` `:2071`, `:2083`, et une garde de la ligne
    `horloge_outils` dans une **liste voisine, non `LIB_NOMME_MCP`** (qui ne relève que les lignes nommant `mcp::`,
    `mcp_gardes_lexicales.rs:207-211`) ;
  - les **9 sites** de production de la 35-5a lisent l'horloge (`state.horloge.aujourd_hui()`, `lecture.aujourd_hui()`) et
    passent la date ; `services::factures::reglements` la lit une fois. Les 19 sites de la 35-5b ne bougent pas. Le
    commentaire `kesh-db/src/repositories/companies.rs:986-989` (« « Aujourd'hui » est `Utc::now().date_naive()`, la même
    horloge que la contre-passation ») devient faux dès cette story et se réécrit ici : « la date reçue, lue une fois sur
    l'horloge de la requête et passée aussi à la contre-passation » (validation P4 de la 35-5a, V4-8).
  - Les 43 constructions `Lecture::new(` des tests passent `horloge_de_test()`.
- **D4 — Les appelants de tests** passent la date qu'ils tiennent déjà ; s'ils n'en tiennent pas, **`aujourd_hui_test()`**,
  helper partagé par crate de tests :
  - `crates/kesh-api/tests/common/jour.rs` : `horloge_de_test().aujourd_hui()` ;
  - `crates/kesh-db/tests/support/jour.rs` et **`crates/kesh-report/tests/support/jour.rs`** (neuf : V1-6 ; validation P4 de la
    35-5a, V4-2) :
    `HorlogeFixe(instant_de_test(std::env::var(NOM_VARIABLE_INSTANT).ok().as_deref()).unwrap_or_else(Utc::now)).aujourd_hui()`.
    `tests/support/` n'a pas de `mod.rs` : chaque binaire qui l'emploie l'inclut par `#[path = "support/jour.rs"] mod jour;` avec
    `#[allow(dead_code)]` (précédents : `kesh-db/tests/companies_repository.rs:13`, `credit_notes_repository.rs:24`) ; la liste
    des binaires concernés s'écrit au Change Log.
  Un `mod tests` de `src` écrit `HorlogeFixe(Utc::now()).aujourd_hui()` en ligne ou passe une date fixe : il ne lit pas la
  variable, sa nuit se prouve par une date fixe (35-5a). Le helper rend le jour de l'horloge du serveur : UTC ici, Zurich après
  la 35-5a, **sans retouche**. Aucun attendu de test ne change.
- **D5 — La garde** (C-e350-24) : `crates/kesh-api/tests/date_du_jour_garde.rs`, telle que la fiche 35-5a la décrit (D4 de
  sa fiche : sources et tests, motifs bornés, commentaires écartés, formes coupées, égalité stricte par fichier, fichier de
  la garde exclu), **posée ici**, ses comptes **écrits depuis son exécution** : l'unité est **l'occurrence** après réduction
  des blancs (validation P3 de la 35-5a, V3-7 : `onboarding_audit_e2e.rs:1916` porte deux formes sur une ligne,
  `kesh-db invoices.rs:4379` deux `CURDATE(`). Les 9 sites de la 35-5a sortent de la liste (leur forme a disparu) ; entrées
  permanentes : `kesh-core/src/horloge.rs` (1 en production, le `.date_naive()` de `jour` ; ses tests sans forme relevée),
  `admin.rs:722` (1, horodatage). S'y ajoutent les **gardes de durcissement** (validation P3 de la 35-5a, V3-5), leurs
  comptes écrits **depuis l'exécution** (V1-1) :
  - le littéral `"KESH_TEST_INSTANT"` : 2, dans `kesh-core/src/horloge.rs` (la constante, et l'attribut `should_panic` de son
    test, qui exige un littéral) ; 0 ailleurs (fichier de la garde exclu) ; les lecteurs de `NOM_VARIABLE_INSTANT` (`kesh-api/src/horloge.rs`, les deux `support/jour.rs`) à leurs comptes
    relevés ;
  - appels `new_for_tests(` et `horloge_de_test(` dans `crates/*/src` : uniquement leurs définitions et l'appel de
    `horloge_de_test()` par `new_for_tests` (`lib.rs`) — aucun appel de production, ni de `mod tests` de `src` ;
  - `HorlogeSysteme` **construit** (lignes qui le nomment hors `pub struct`, `impl`, `use`) : `main.rs` 1,
    `kesh-api/src/horloge.rs` 1 (dans `horloge_depuis`), 0 ailleurs ;
  - aucun `static`, `OnceLock`, `LazyLock`, `lazy_static!`, `thread_local!` dans les **fichiers** `kesh-core/src/horloge.rs`,
    `kesh-api/src/horloge.rs`, `tests/common/jour.rs` et les deux `tests/support/jour.rs`.
- **D6 — Docs** : une ligne dans `docs/testing.md` — `KESH_TEST_INSTANT` est une variable **de tests**, lue par
  `horloge_de_test()` et les helpers, sans effet sur le binaire servi (`main.rs` construit `HorlogeSysteme`) ; vide, elle vaut
  absente ; invalide, elle arrête le test. Aucun texte d'interface, aucun manuel, aucun CHANGELOG (aucun changement visible).

**« Comportement inchangé », précisément** (V1-8) : même jour (UTC), mêmes refus, mêmes écritures — à une nuance près, voulue :
la date est lue **une fois** à l'entrée de la requête (avant l'attente d'un verrou et les reprises `retry_on_deadlock`) et
partagée par le bloqueur et la contre-passation, où elle était lue deux fois à l'intérieur. L'écart n'existe que dans une
fenêtre de quelques secondes autour de minuit UTC, et va dans le sens voulu.

## Critères d'acceptation

1. **AC1 — Rien ne change** : le gate complet (`test-fast.sh --ci`), `scripts/gate-ci.sh`, Vitest et l'E2E complet sont verts
   **sans qu'aucun attendu de test ne change**. Contrôle (V1-3), au Change Log : `git diff -U0 <base> -- 'crates/*/tests/*'
   'crates/*/src/*' | /usr/bin/grep -E '^[-+]' | /usr/bin/grep -vE '^(\+\+\+|---)'` trié à la main — les seules lignes de
   test modifiées sont des **arguments ajoutés** (date, horloge) et des constructions (`Lecture::new`, champ `horloge`) ;
   aucune ligne `assert…` ne change (`… | /usr/bin/grep -E 'assert'` : vide, ou chaque ligne commentée).
2. **AC2 — L'horloge fixe atteint les sites** : un `AppState` à `HorlogeFixe(2026-06-15T10:00:00Z)` date une contre-passation du
   `2026-06-15` et refuse un verrou au `2026-06-15` ; à `HorlogeFixe(2025-12-31T23:30:00Z)`, la contre-passation est datée du
   `2025-12-31` (**le jour UTC** : comportement d'avant, voulu ici) — ce test est celui que la 35-5a retourne. AC2 n'exerce
   que **2 des 9 sites** ; les sept autres (soldes d'ouverture ×4, rang 5, `unlock_books`, assistant) ne sont tenus ici que par
   la garde de forme (D5), et par les critères de la 35-5a (V1-8).
3. **AC3 — Le MCP reçoit la même horloge** : `state.lecture().aujourd_hui() == state.horloge.aujourd_hui()` ; les gardes
   lexicales du câblage sont vertes ; la mutation « câblage du MCP sur `HorlogeSysteme` » est tuée par la garde de
   construction de `HorlogeSysteme` (D5). Aucun outil du MCP n'expose un site de jour (`factures_fournisseur.rs:206` jette le
   motif) : la garde est **le seul filet** du câblage dans cette story.
4. **AC4 — `KESH_TEST_INSTANT`** : `instant_de_test(Some("2025-12-31T23:30:00Z"))` rend l'instant ; `None` et `Some("")`
   rendent `None` ; une valeur invalide `panic!` en citant la constante (`#[should_panic(expected = "KESH_TEST_INSTANT")]`,
   l'attribut exigeant un littéral : le second et dernier littéral admis, dans le `mod tests` de `horloge.rs`) ; `horloge_depuis(Some("2025-12-31T23:30:00Z")).maintenant()` rend l'instant,
   `horloge_depuis(None)` une horloge système (deux lectures croissantes).
5. **AC5 — La garde** : verte avec les comptes de son exécution ; sondes au Change Log (forme ajoutée, forme coupée, compte
   + 1, `KESH_TEST_INSTANT` dans un fichier de plus, `HorlogeSysteme` construit ailleurs, `OnceLock` dans un helper de jour,
   appel de `new_for_tests(` dans `src`).
6. **AC6 — Aucune migration** ; `crates/kesh-db/migrations/` intact.

## Tests à écrire, et la preuve du rouge

AC2 à AC5 rouges tant que la plomberie n'est pas posée (ils ne compilent pas, puis échouent par attendu) ; AC1 est l'absence
de rouge. Journal `kesh-gate-logs/e355a0/rouges.log`.

**Mutations** : M1 un site de la 35-5a qui garde `Utc::now()` (tuée par la **garde** pour les neuf sites, le fichier repassant à
un compte non nul ; par AC2 aussi pour la contre-passation et le verrou) ; M2 le câblage du MCP sur `HorlogeSysteme` (garde de
construction, AC3) ; M3 `horloge_de_test` mémorisée dans un `OnceLock` (garde D5) ; M4 le bloqueur et la contre-passation qui
lisent deux fois l'horloge (un test du dépôt donne deux dates distinctes au chemin qui n'en devrait recevoir qu'une : il ne
compile pas, ou la garde le voit) ; M5 `instant_de_test(Some(""))` qui panique (AC4).

## Tâches

- [ ] Étape 0 : recompter la fermeture des appelants (production, tests) par `git grep` sur les noms de D2 ; les 43
      constructions de `Lecture`, les 9 littéraux ; Change Log.
- [ ] Commits internes, **dans cet ordre** (chacun compile et passe le gate complet) : (1) `kesh-core` (`horloge.rs`) ;
      (2) `kesh-db` (D2) **avec** ses appelants de tests de `kesh-db` et de `kesh-report`, qui doivent compiler ensemble ;
      (3) `kesh-api` (D3) avec ses appelants et ses tests, helpers (D4) ; (4) garde (D5) et `docs/testing.md` (D6).
- [ ] Gates : `test-fast.sh --ci` complet à chaque commit, `scripts/gate-ci.sh`, Vitest, E2E complet au dernier commit de code.
- [ ] Revue : **fichier par fichier** pour les appelants (rollout mécanique, `CLAUDE.md` § *Règle de splitting préventif*),
      passes adversariales sur `kesh-core/src/horloge.rs`, `kesh-api/src/horloge.rs`, D2 et la garde.

## Dev Notes

### Dérogation règle de splitting

La 35-5a0 touche plus de cinq modules (`kesh-core`, une dizaine de dépôts de `kesh-db`, `kesh-api` — routes, services,
middleware, `lib.rs`, `main.rs` —, les tests de `kesh-db`, `kesh-api` et `kesh-report`, des gardes lexicales) et un diff
mécanique de l'ordre de 70 fichiers (V1-3). **Elle n'est pas redécoupée**, au titre de l'exception de `CLAUDE.md` : un
changement de signature (D2, `Lecture::new`) oblige **tous** ses appelants, production et tests, dans le même commit ; un
découpage plus fin laisserait un état intermédiaire qui ne compile pas, donc qu'on ne peut ni tester ni fusionner. Elle est
déjà le *story-zero* de la 35-5a (C-e350-28) : elle ne porte aucune règle métier et ne change aucun comportement. Atténuations :
l'ordre des commits internes ci-dessus (chacun compile), la revue fichier par fichier des appelants, le contrôle de l'AC1 (aucune
assertion modifiée), le gate complet à chaque commit.

### Ordre de fusion avec la 35-3 (validation P3 de la 35-5a, V3-2 ; validation P1, V1-7)

La 35-5a0 change la signature des cinq écritures fournisseur et des bloqueurs. **À ce jour, la 35-3 n'en ajoute aucun appel** :
ses tests neufs de `supplier_invoices_repository.rs` appellent `create_validated` (hors fermeture ;
`git diff 8424e36e HEAD | /usr/bin/grep -E '^\+.*supplier_invoices::(validate|unvalidate|pay|cancel|cancel_settlement)\b'` vide
sur `kesh-e353`, `51d303ea`) ; le module qu'elle touche (`:2250-2257`) porte déjà des `pay(` (`:2550`, `:2600`), comptés dans les
158. Le conflit de compilation reste **possible** si la 35-3 ajoute un tel appel d'ici sa fusion : **à constater à
l'intégration**. Règle : **la seconde des deux fusionnée ajoute la date aux appels de l'autre** (`aujourd_hui_test()`) et relance
le gate complet. Hunks de la 35-3 dans `supplier_invoices.rs` (`:1523`, `:1725`, commentaires et `validate_in_tx`) et
`invoice_settlements_write.rs` (`:679-680`) : distincts. Ordre préféré : **35-3 puis 35-5a0** (la 35-3 est en tête d'epic) ;
l'autre ordre est permis sous la même règle.

### Fichiers communs — règle d'union C-e350-8

Ceux de la 35-5a (sa fiche, § *Fichiers communs*), pour la part plomberie : `lib.rs`, `services/poignees.rs`,
`mcp_gardes_lexicales.rs`, `services_motif.rs`, `tests/common/mcp.rs`, les fichiers de la fermeture des appelants, et
`crates/kesh-api/tests/date_du_jour_garde.rs` (créé ici ; sa liste d'exemptions est ensuite éditée par la 35-5a, la 35-5b,
la 35-5c, dans cet ordre). Avec la **35-6** : `opening_balances.rs` (35-6 `:355-366`, `:515-530`, dans son `mod tests` ; ici
`:244`, `:604`) ; avec la **35-4** : `reconciliation.rs` (35-4 `:1540` ; ici `:5799`, `:5938`, `:6587`) — hunks distincts ; l'epic
les fait passer avant la 35-5a0, qui se rebase sur elles. **Tests neufs des stories sœurs** : tout appel neuf à une fonction de
la fermeture reçoit `aujourd_hui_test()` ou une date fixe ; toute construction neuve `Lecture::new(` ou tout littéral
`AppState {` prend l'horloge (`horloge_de_test()`) ; à l'union, l'intégration relance la garde.

### Ce qui doit rester vrai

- Aucun comportement ne change : même jour (UTC), mêmes refus, mêmes écritures. Aucune migration, aucune donnée réécrite.
- La lecture seule du MCP (Q11) : la `Lecture` des outils garde le pool en lecture seule ; l'horloge n'y ajoute aucun accès.
- `KESH_TEST_INSTANT` ne peut pas changer la date des livres : `main.rs` ne la lit pas, et aucun code de production
  n'appelle `new_for_tests` ni `horloge_de_test` (gardes).

### Références

- Issues #929, #1022, #907 ; fiche 35-5a (inventaire, garde) ; `CLAUDE.md` § *Règle de splitting préventif* (story-zero),
  § *Exception `kesh-db`*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

- Fiche rédigée au découpage de la 35-5a (C-e350-28) ; validation à lancer.

### File List

*Attendue (à confirmer)* : `crates/kesh-core/{Cargo.toml,src/lib.rs,src/horloge.rs}` ; les dépôts de `kesh-db` de la
fermeture, dont le commentaire de `companies.rs:986-989` ; `crates/kesh-api/src/{lib,main,horloge}.rs`, `middleware/auth.rs`,
`services/{poignees,factures,factures_fournisseur}.rs`,
`routes/{bank_accounts,opening_balances,companies,journal_entries,onboarding,invoices,supplier_invoices,imported_supplier_invoices,reconciliation}.rs` ;
`crates/kesh-api/tests/{date_du_jour_garde,common/jour,mcp_gardes_lexicales,services_motif}.rs` et les fichiers de tests de la
fermeture ; `crates/kesh-db/tests/support/jour.rs` ; `crates/kesh-report/tests/support/jour.rs` et les tests `kesh-report`
concernés ; `docs/testing.md`.

## Change Log

- 2026-11-12 — Fiche rédigée (découpage de la 35-5a sur décision de l'orchestrateur, validation P3 de la 35-5a, V3-4 ;
  C-e350-28). Validation non lancée.
- 2026-11-12 — **Validation P1** (Sonnet, `kesh-gate-logs/e35-5a0-validation-p1.md` : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 7 LOW)
  remédiée par l'agent de rédaction (worktree `kesh-e355`), avec ce qui, dans la validation P4 de la 35-5a, revient à la 35-5a0
  (V4-2 en partie, V4-4, V4-6, V4-8). Prémisses lues au code de `release/0.17.6`.
  - **V1-1** (et V1-1 bis) : nom de la variable porté par une constante unique `NOM_VARIABLE_INSTANT` ; littéral compté 2 dans
    `horloge.rs` (constante, attribut `should_panic`) ; `middleware/auth.rs:285` (`mod tests` de `src`, `:198`) prend
    `HorlogeFixe(Utc::now())` ; comptes écrits depuis l'exécution.
  - **V1-2** : fonction pure `horloge_depuis(Option<&str>)`, testée sans `set_var` (`unsafe` en édition 2024, `Cargo.toml:17` ;
    `lib.rs:9`) ; « lue à chaque appel » tenu par la garde et la forme de `horloge_de_test`.
  - **V1-3** : section *Dérogation règle de splitting* (un changement de signature oblige tous les appelants dans le même
    commit) ; ordre des commits internes ; revue fichier par fichier ; commande de contrôle de l'AC1.
  - **LOW** : V1-4 (définitions citées : `:2426`, `:2510`, `:2706`, `:2778`, `:3013`, `:3040`, `:3095` ; `settlement_cancellation:87` ;
    `finalize_inner` `:914`, lus) ; V1-5 (`HorlogeSysteme` défini dans `kesh-api/src/horloge.rs`, construction comptée hors
    `struct`/`impl`/`use`) ; V1-6 (`kesh-report/tests/support/jour.rs`, inclusion `#[path]`) ; V1-7 (la 35-3 appelle
    `create_validated`, conflit « à constater à l'intégration » ; 35-6 et 35-4 aux fichiers communs ; `Lecture::new` et littéraux
    d'`AppState` des sœurs) ; V1-8 (AC2 : 2 sites sur 9, le reste par la garde ; AC3 : la garde seul filet ; nuance « lue une
    fois ») ; V1-9 (chaîne vide = absente) ; V1-10 (tests de `horloge.rs` sans forme relevée).
  - Depuis la 35-5a P4 : V4-4 (fonction privée `jour`, que la 35-5a remplace : compte 1 à chaque étape) ; V4-8 (commentaire
    `companies.rs:986-989` réécrit ici) ; V4-2 (helper `kesh-report`).
  - Aucun finding réfuté. Signal de découpage : levé (plus de cinq modules) et **non suivi**, au titre de l'exception de
    `CLAUDE.md` (dérogation écrite). Aucune migration. Statut `ready-for-dev`.
