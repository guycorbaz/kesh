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
- bloqueurs : `settlement_entry_cancel_blocker`, `settlement_cancel_blocker`, `settlement_cancel_blocker_unlinking`
  (`invoice_settlements_write.rs:826-878`), `reconciliation_cancel::{cancel_blocker (:250), entry_cancel_outlook (:321)}` ;
- écritures et vues des factures fournisseur : `get_settlement_view`, `get_settlement_view_in_tx` (calcule les deux bloqueurs,
  `supplier_invoices.rs:3055-3056`), `commit_with_view` (`:3109`), et donc **toutes** les écritures qui se terminent par lui —
  `validate` (`:1058`), `unvalidate` (`:1074`), `pay` (`:1899`), `cancel` (`:2680`), `cancel_settlement` (`:2963`) —, plus
  `cancel_in_tx` (`:2462`, `:2600`, `:2750`, `:2871`) et `supplier_invoice_cancel_blocker` (`:3023-3068`) ;
- règlements client : `invoice_settlements_write::{cancel_settlement (:1134), cancel_settlement_in_tx (:1023)}` ;
- rapprochements : `reconciliation_cancel::{cancel (:804), cancel_in_tx (:564), get_view}` ;
- verrou : `lock_books`, `unlock_books`.

**Appelants de production** : `routes/companies.rs:505`, `:518` ; `routes/journal_entries.rs:456` ; `routes/supplier_invoices.rs:829`,
`:871`, `:904`, `:931`, `:960`, `:990` ; `routes/invoices.rs:1361` ; `routes/imported_supplier_invoices.rs:230`, `:646`, `:723` ;
`routes/reconciliation.rs:5799`, `:5938` et **`:6587`** (`get_entry_candidates`, une **lecture** qui passe par le rang 5) ;
`services/factures.rs:552` (`reglements`) ; `services/factures_fournisseur.rs:206` (dont le motif est jeté par
`detail_sans_fichier`, `:227-237`). `routes/supplier_invoices.rs:272` est `from_view`, une conversion, non un appelant.
Homonyme non concerné : `invoice_reminders::cancel_in_tx`.

**Appelants de tests** : `git grep -cE 'supplier_invoices::(validate|unvalidate|pay|cancel|cancel_settlement)\s*\(' -- 'crates/*/tests/*'`
rend **158 appels dans 23 fichiers** ; s'y ajoutent les appelants de `cancel_settlement(` (dont
`kesh-db/tests/{credit_notes_repository,invoice_amount_due_parity}.rs`, `kesh-report/tests/{aged_receivables,vat_report_write_off}.rs`),
de `reverse*`, des bloqueurs, de `lock_books`/`unlock_books` (dont deux dans `admin_full_import_e2e.rs`, outre `:1809`), de la
vue (`supplier_invoice_response_e2e.rs:434-496`) et de `reconciliation_cancel::*` ; une première passe sur l'ensemble des
noms de la fermeture rend **56 fichiers** (`src` compris). **Le compilateur donne la liste close** ; l'étape 0 la recompte
par `git grep` sur les noms ci-dessus et l'écrit au Change Log.

## Décisions (C-e350-21, -24, -26, -28, -29)

- **D1 — L'horloge, dans `kesh-core`** (C-e350-29) : `pub trait Horloge: Send + Sync { fn maintenant(&self) -> DateTime<Utc>;
  fn aujourd_hui(&self) -> NaiveDate { … } }`, `pub struct HorlogeFixe(pub DateTime<Utc>)`, et la fonction pure `pub fn
  instant_de_test(valeur: Option<&str>) -> Option<DateTime<Utc>>` (RFC 3339 ; une valeur invalide fait `panic!` en nommant
  `KESH_TEST_INSTANT`), dans un module `kesh-core/src/horloge.rs`, sans I/O. **Dans cette story, `aujourd_hui()` rend
  `self.maintenant().date_naive()` : le jour UTC, comme aujourd'hui.** La 35-5a change ce seul corps en `jour_suisse(…)`, si
  bien que le serveur **et** les tests qui passent par l'horloge basculent ensemble.
- **D2 — `kesh-db` ne lit plus l'horloge pour une date** : toutes les fonctions de la fermeture reçoivent `aujourd_hui:
  NaiveDate`, ajouté **en dernier argument** (la garde lexicale `services/factures.rs:603`, `"settlement_cancel_blocker(&muttx,"`,
  tient). `ReversalDating::Today` porte la date ou la reçoit. Une lecture de l'horloge par requête, passée au bloqueur **et** à la
  contre-passation.
- **D3 — `kesh-api`** :
  - `HorlogeSysteme` (`maintenant()` = `Utc::now()`), construit dans **`main.rs`** et dans **`horloge_de_test()`** seulement ;
  - `pub fn horloge_de_test() -> Arc<dyn Horloge>` (`#[doc(hidden)]`, `lib.rs`) : `HorlogeFixe` à l'instant de
    `KESH_TEST_INSTANT` si la variable est posée, `HorlogeSysteme` sinon ; lue à chaque appel, **sans `static`, `OnceLock`,
    `LazyLock`** ;
  - champ `pub horloge: Arc<dyn Horloge>` d'`AppState` : `HorlogeSysteme` dans `main.rs:439` ; `horloge_de_test()` dans
    `new_for_tests` **et** dans les 8 littéraux de test (`middleware/auth.rs:285`, `invoice_frozen_pdf_e2e.rs:101`,
    `invoice_send_email_e2e.rs:104`, `:1036`, `:1150`, `password_recovery_e2e.rs:126`, `setup_admin_e2e.rs:82`, et le `Self {`
    de `new_for_tests`) — ainsi la variable atteint tous les états de test (validation P3 de la 35-5a, V3-1) ;
  - `Lecture { pool, horloge }`, `Lecture::new(pool, horloge)` **sans défaut**, `aujourd_hui()`, `horloge()` ;
    `AppState::lecture()` : `services::poignees::Lecture::new(self.pool.clone(), self.horloge.clone())` ; câblage du MCP en trois
    lignes courtes (`let horloge_outils = state.horloge.clone();`, `let lecture_pool = services::poignees::Lecture::new(pool_outils.clone(),
    horloge_outils);`, `let lecture_outils = mcp::LectureOutils::nouvelle(lecture_pool);`) ; gardes réécrites :
    `mcp_gardes_lexicales.rs:169`, `:575`, `services_motif.rs` `FABRIQUES_V2` `:2071`, `:2083`, et une garde de la ligne
    `horloge_outils` dans une **liste voisine, non `LIB_NOMME_MCP`** (qui ne relève que les lignes nommant `mcp::`,
    `mcp_gardes_lexicales.rs:207-211`) ;
  - les **9 sites** de production de la 35-5a lisent l'horloge (`state.horloge.aujourd_hui()`, `lecture.aujourd_hui()`) et
    passent la date ; `services::factures::reglements` la lit une fois. Les 19 sites de la 35-5b ne bougent pas.
  - Les 43 constructions `Lecture::new(` des tests passent `horloge_de_test()`.
- **D4 — Les appelants de tests** passent la date qu'ils tiennent déjà ; s'ils n'en tiennent pas, **`aujourd_hui_test()`**,
  helper partagé par crate de tests : `crates/kesh-api/tests/common/jour.rs` (`horloge_de_test().aujourd_hui()`) et
  `crates/kesh-db/tests/support/jour.rs` (`HorlogeFixe(instant_de_test(env).unwrap_or_else(Utc::now)).aujourd_hui()`) ; un
  `mod tests` de `src` écrit `HorlogeFixe(Utc::now()).aujourd_hui()` en ligne ou passe une date fixe. Le helper rend le jour
  de l'horloge du serveur : UTC ici, Zurich après la 35-5a, **sans retouche**. Aucun attendu de test ne change.
- **D5 — La garde** (C-e350-24) : `crates/kesh-api/tests/date_du_jour_garde.rs`, telle que la fiche 35-5a la décrit (D4 de
  sa fiche : sources et tests, motifs bornés, commentaires écartés, formes coupées, égalité stricte par fichier, fichier de
  la garde exclu), **posée ici**, ses comptes **écrits depuis son exécution** : l'unité est **l'occurrence** après réduction
  des blancs (validation P3 de la 35-5a, V3-7 : `onboarding_audit_e2e.rs:1916` porte deux formes sur une ligne,
  `kesh-db invoices.rs:4379` deux `CURDATE(`). Les 9 sites de la 35-5a sortent de la liste (leur forme a disparu) ; entrée
  permanente : `kesh-core/src/horloge.rs` (1, le `.date_naive()` du jour), `admin.rs:722` (1, horodatage). S'y ajoutent les
  **gardes de durcissement** (validation P3, V3-5) :
  - `KESH_TEST_INSTANT` : `kesh-api/src/lib.rs` 1, `kesh-db/tests/support/jour.rs` 1, 0 ailleurs (fichier de la garde exclu) ;
  - appels `new_for_tests(` et `horloge_de_test(` dans `crates/*/src` : uniquement leurs définitions et l'appel de
    `horloge_de_test()` par `new_for_tests` (`lib.rs`) — aucun appel de production ;
  - `HorlogeSysteme` construit dans `main.rs` (1) et `lib.rs` (1, `horloge_de_test`), 0 ailleurs ;
  - aucun `static`, `OnceLock`, `LazyLock`, `lazy_static!`, `thread_local!` dans `kesh-core/src/horloge.rs`,
    `tests/common/jour.rs`, `tests/support/jour.rs` et le corps de `horloge_de_test`.
- **D6 — Docs** : une ligne dans `docs/testing.md` — `KESH_TEST_INSTANT` est une variable **de tests**, lue par
  `horloge_de_test()` et les helpers, sans effet sur le binaire servi (`main.rs` construit `HorlogeSysteme`) ; une valeur
  invalide arrête le test. Aucun texte d'interface, aucun manuel, aucun CHANGELOG (aucun changement visible).

## Critères d'acceptation

1. **AC1 — Rien ne change** : le gate complet (`test-fast.sh --ci`), `scripts/gate-ci.sh`, Vitest et l'E2E complet sont verts
   **sans qu'aucun attendu de test ne change** ; `git diff` ne touche aucune assertion de date (relevé au Change Log).
2. **AC2 — L'horloge fixe atteint les sites** : un `AppState` à `HorlogeFixe(2026-06-15T10:00:00Z)` date une contre-passation du
   `2026-06-15` et refuse un verrou au `2026-06-15` ; à `HorlogeFixe(2025-12-31T23:30:00Z)`, la contre-passation est datée du
   `2025-12-31` (**le jour UTC** : comportement d'avant, voulu ici) — ce test est celui que la 35-5a retourne.
3. **AC3 — Le MCP reçoit la même horloge** : `state.lecture().aujourd_hui() == state.horloge.aujourd_hui()` ; les gardes
   lexicales du câblage sont vertes ; la mutation « câblage du MCP sur `HorlogeSysteme` » est tuée par la garde de
   construction de `HorlogeSysteme` (D5).
4. **AC4 — `KESH_TEST_INSTANT`** : `instant_de_test(Some("2025-12-31T23:30:00Z"))` rend l'instant ; `None` rend `None` ; une
   valeur invalide `panic!` en nommant la variable (`#[should_panic(expected = "KESH_TEST_INSTANT")]`) ; `horloge_de_test()`
   lit la variable à chaque appel (deux appels, valeur changée entre les deux : deux instants).
5. **AC5 — La garde** : verte avec les comptes de son exécution ; sondes au Change Log (forme ajoutée, forme coupée, compte
   + 1, `KESH_TEST_INSTANT` dans un fichier de plus, `HorlogeSysteme` construit ailleurs, `OnceLock` dans un helper de jour,
   appel de `new_for_tests(` dans `src`).
6. **AC6 — Aucune migration** ; `crates/kesh-db/migrations/` intact.

## Tests à écrire, et la preuve du rouge

AC2 à AC5 rouges tant que la plomberie n'est pas posée (ils ne compilent pas, puis échouent par attendu) ; AC1 est l'absence
de rouge. Journal `kesh-gate-logs/e355a0/rouges.log`.

**Mutations** : M1 un site de la 35-5a qui garde `Utc::now()` (AC2 : l'horloge fixe ne l'atteint pas) ; M2 le câblage du MCP
sur `HorlogeSysteme` (AC3, garde) ; M3 `horloge_de_test` mémorisée dans un `OnceLock` (garde D5, et AC4) ; M4 le bloqueur et la
contre-passation qui lisent deux fois l'horloge (un test du dépôt donne deux dates distinctes au chemin qui n'en devrait
recevoir qu'une : il ne compile pas, ou la garde le voit).

## Tâches

- [ ] Étape 0 : recompter la fermeture des appelants (production, tests) par `git grep` sur les noms de D2 ; les 43
      constructions de `Lecture`, les 9 littéraux ; Change Log.
- [ ] `kesh-core` (`horloge.rs`) ; `kesh-db` (D2) ; `kesh-api` (D3) ; helpers (D4) ; garde (D5) ; `docs/testing.md` (D6).
- [ ] Gates : `test-fast.sh --ci` complet à chaque commit, `scripts/gate-ci.sh`, Vitest, E2E complet au dernier commit de code.

## Dev Notes

### Ordre de fusion avec la 35-3 (validation P3 de la 35-5a, V3-2)

La 35-3 ajoute dans `crates/kesh-db/tests/supplier_invoices_repository.rs` (`:2250-2257`) des tests qui appellent
`supplier_invoices::validate` / `pay` ; la 35-5a0 change leur signature. C'est un **conflit de compilation**, non de texte :
**la seconde des deux fusionnée ajoute la date aux appels de l'autre** (`aujourd_hui_test()`), et relance le gate complet.
De même pour `supplier_invoices.rs` (35-3 `:1521-1530`, `:1698-1727`) et `invoice_settlements_write.rs` (35-3 `:679-680`) : hunks
distincts, recompilation obligatoire. L'ordre préféré est **35-3 puis 35-5a0** (la 35-3 est en tête d'epic) ; l'autre ordre
est permis sous la même règle.

### Fichiers communs — règle d'union C-e350-8

Ceux de la 35-5a (sa fiche, § *Fichiers communs*), pour la part plomberie : `lib.rs`, `services/poignees.rs`,
`mcp_gardes_lexicales.rs`, `services_motif.rs`, `tests/common/mcp.rs`, les fichiers de la fermeture des appelants, et
`crates/kesh-api/tests/date_du_jour_garde.rs` (créé ici ; sa liste d'exemptions est ensuite éditée par la 35-5a, la 35-5b,
la 35-5c, dans cet ordre). **Tests neufs des stories sœurs** : tout appel neuf à une fonction de la fermeture reçoit
`aujourd_hui_test()` ou une date fixe ; à l'union, l'intégration relance la garde.

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
fermeture ; `crates/kesh-api/src/{lib,main}.rs`, `middleware/auth.rs`, `services/{poignees,factures,factures_fournisseur}.rs`,
`routes/{bank_accounts,opening_balances,companies,journal_entries,onboarding,invoices,supplier_invoices,imported_supplier_invoices,reconciliation}.rs` ;
`crates/kesh-api/tests/{date_du_jour_garde,common/jour,mcp_gardes_lexicales,services_motif}.rs` et les fichiers de tests de la
fermeture ; `crates/kesh-db/tests/support/jour.rs` ; `crates/kesh-report/tests/*` concernés ; `docs/testing.md`.

## Change Log

- 2026-11-12 — Fiche rédigée (découpage de la 35-5a sur décision de l'orchestrateur, validation P3 de la 35-5a, V3-4 ;
  C-e350-28). Validation non lancée.
