# Story 35-5a0 : l'horloge injectable, posée sans rien changer — `Horloge`, `HorlogeFixe`, `AppState.horloge`, `Lecture` à horloge, la date du jour passée à `kesh-db`, la garde de l'inventaire (#929, #1022, #907)

Status: review

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
  `:838` ; `:878` est un appel), `reconciliation_cancel::{cancel_blocker (:250), entry_cancel_outlook (:321)}` ;
- écritures et vues des factures fournisseur (`supplier_invoices.rs`, **définitions**) : `supplier_invoice_cancel_blocker`
  (`:2426`), `cancel_in_tx` (`:2510`), `supplier_settlement_cancel_blocker` (`:2706`, appelé `:2839` et `:3055`), le bloqueur
  de `:2724`, `cancel_settlement_in_tx` (`:2778`), `get_settlement_view` (`:3013`, l'enveloppe), `get_settlement_view_in_tx`
  (`:3040`, calcule les deux bloqueurs `:3055-3056`), `commit_with_view` (`:3095` ; `:3109` est son appel interne), et donc
  **toutes** les écritures qui se terminent par lui — `validate` (`:1058`), `unvalidate` (`:1074`), `pay` (`:1899`), `cancel`
  (`:2680`), `cancel_settlement` (`:2963`). Les **appels** internes `:2462`, `:2750` (bloqueur) et `:2600`, `:2871`
  (`reverse_owned_in_tx`) reçoivent la date de leur appelant ;
- règlements client : `invoice_settlements_write::{cancel_settlement (:1134), cancel_settlement_in_tx (:912)}` (`:1023` est son appel interne
  de `reverse_owned_in_tx(` ; validation P3, V3-4, `/usr/bin/grep -nE '^pub async fn'` lu) ;
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
vue (`supplier_invoice_response_e2e.rs:434-496`) et de `reconciliation_cancel::*`. Recompté à la validation P2 (V2-8),
`git grep -lE "$P" -- 'crates/*'`, où `$P` est l'alternative des noms de la fermeture ci-dessus suivis de `\s*\(` : **50
fichiers, dont 36 de tests** ; **68** avec ceux qui portent `Lecture::new(` ou un littéral `AppState {` (union triée). Ces
deux nombres sont les seuls que la fiche emploie. **Le compilateur donne la liste close** ; l'étape 0 la recompte par la même
commande et l'écrit au Change Log.

**Fichiers `support/` de `kesh-db/tests` qui appellent la fermeture** (validation P2, V2-7 ; lus) : `support/lettering_documents.rs`
— `:445` (`invoice_settlements_write::cancel_settlement(`), `:545` (`companies::lock_books(`), `:870`
(`supplier_invoices::pay(…, le, …)` dans `payer_par`, où **`le` est la date du paiement**, non le jour) —, inclus par quatre
binaires (`lettering_documents_backfill.rs:43`, `lettering_documents.rs:29`, `letterings.rs:31`, `open_items.rs:32`) ; et
`support/fournisseur_rapproche.rs:155` (`supplier_invoices::pay(`), inclus par quatre autres (`reconciliation_cancel.rs:31`,
`supplier_document_attachment.rs:21`, `supplier_invoices_draft.rs:23`, `supplier_invoices_internal_number.rs:11`).

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
- **D4 — Les appelants de tests** passent la date **du jour** qu'ils tiennent déjà (une variable `today` / `aujourd_hui` du
  montage, tirée de l'horloge), **jamais une date de pièce** (date d'écriture, de facture, de paiement) : passer une date de
  pièce comme `aujourd_hui` changerait le jour du bloqueur ou de la contre-passation, donc ce que le test exerce, sans
  qu'aucun `assert` ne bouge. Les pièges relevés (validation P2, V2-7 ; validation P3, V3-6 ; lus) : `payer_par`
  (`support/lettering_documents.rs:863`, appel de `pay` à `:870`) reçoit `le`, date du paiement, et `payer` (`:879`) la lui
  transmet (`:886-894`) — `payer_par` prend `aujourd_hui_test()`, `payer` ne change pas (validation P4, V4-4) ; `facture_payee`
  (`fournisseur_rapproche.rs:120-127`, appel de `pay` à `:155-162`) reçoit **`jour`**, date de la facture **et** du paiement —
  un nom qui invite à la passer comme `aujourd_hui`, et prend aussi `aujourd_hui_test()` (le module `jour` du helper et
  la variable locale `jour` vivent dans des espaces de noms distincts, à constater à la compilation). S'ils n'en tiennent pas, **`aujourd_hui_test()`**, helper partagé par crate de tests :
  - `crates/kesh-api/tests/common/jour.rs` : `horloge_de_test().aujourd_hui()` ;
  - `crates/kesh-db/tests/support/jour.rs` et **`crates/kesh-report/tests/support/jour.rs`** (neuf : V1-6 ; validation P4 de la
    35-5a, V4-2) :
    `HorlogeFixe(instant_de_test(std::env::var(NOM_VARIABLE_INSTANT).ok().as_deref()).unwrap_or_else(Utc::now)).aujourd_hui()`.
    `tests/support/` n'a pas de `mod.rs` : chaque binaire qui l'emploie l'inclut par `#[path = "support/jour.rs"] mod jour;` avec
    `#[allow(dead_code)]` (précédents : `kesh-db/tests/companies_repository.rs:13`, `credit_notes_repository.rs:24`) ; la liste
    des binaires concernés s'écrit au Change Log. Les deux fichiers `support/` qui appellent la fermeture
    (`lettering_documents.rs`, `fournisseur_rapproche.rs`, § *Ce qui se lit*) incluent eux-mêmes le helper par un
    `#[path = "jour.rs"] mod jour;` **imbriqué** (un `#[path]` placé dans un fichier chargé par `#[path]` se résout depuis le
    dossier de ce fichier, soit `tests/support/`), si bien que leurs huit binaires n'ont rien à déclarer ; un binaire qui
    inclut aussi `jour.rs` au premier niveau en a deux copies, sans conflit (deux modules distincts). Règle non vérifiée au
    compilateur à la rédaction : l'étape 0 la constate ; à défaut, ces fichiers reçoivent la date du jour en paramètre de
    leurs helpers, et leurs appelants la passent.
  Un `mod tests` de `src` écrit `HorlogeFixe(Utc::now()).aujourd_hui()` en ligne ou passe une date fixe : il ne lit pas la
  variable, sa nuit se prouve par une date fixe (35-5a). Le helper rend le jour de l'horloge du serveur : UTC ici, Zurich après
  la 35-5a, **sans retouche**. Aucun attendu **de comportement** ne change (AC1).
- **D5 — La garde** (C-e350-24) : `crates/kesh-api/tests/date_du_jour_garde.rs`, telle que la fiche 35-5a la décrit (D4 de
  sa fiche : sources et tests, motifs bornés, commentaires écartés, formes coupées, égalité stricte par fichier, fichier de
  la garde exclu), **posée ici**, ses comptes **écrits depuis son exécution** : l'unité est **l'occurrence** après réduction
  des blancs (validation P3 de la 35-5a, V3-7 : `onboarding_audit_e2e.rs:1916` porte deux formes sur une ligne,
  `kesh-db invoices.rs:4379` deux `CURDATE(`). Les 9 sites de la 35-5a sortent de la liste (leur forme a disparu) ; entrées
  permanentes : `kesh-core/src/horloge.rs` (1 en production, le `.date_naive()` de `jour` ; ses tests sans forme relevée),
  `admin.rs:722` (1, horodatage). S'y ajoutent les **gardes de durcissement** (validation P3 de la 35-5a, V3-5), leurs
  comptes écrits **depuis l'exécution** (V1-1). Toutes sont des **listes closes en égalité stricte par fichier** sur
  `crates/*/src` et `crates/*/tests` — un fichier absent de la liste compte 0 —, le fichier de la garde exclu nommément, et
  **les lignes dont le début, blancs ôtés, est `//` (donc `///` et `//!`) écartées, comme pour la garde de date** (validation
  P2, V2-5 : sans cela, les comptes dépendraient de la rédaction des doc-comments de `new_for_tests`, `horloge_de_test`,
  `horloge_depuis`). **Le retrait des instructions `use` ne vaut que pour le compte de `HorlogeSysteme` construit**
  (validation P3, V3-2, restreint à la validation P4, V4-1) : rustfmt écrit un import groupé une entrée par ligne
  (`main.rs:41-49`, `use kesh_api::{` … `};`, lu), et la ligne `horloge::HorlogeSysteme,` d'un tel groupe compterait sinon
  comme une construction. Le retrait est **borné** (V4-2) : une instruction qui commence, blancs ôtés, par `use `, `pub use `
  ou `pub(…) use `, jusqu'au premier `;` hors commentaire, les lignes `//` étant écartées avant. **Toutes les autres gardes
  comptent les `use` compris**, et leurs noms **comme mots** (`\bnom\b`, non `nom(`) : un import, même renommé par `as`, nomme
  le symbole et compte — sans quoi `use … ::horloge_de_test as h;` puis `h()` passerait. Leurs comptes incluent donc la
  définition, les appels et les imports :
  - le littéral `"KESH_TEST_INSTANT"` : 2, dans `kesh-core/src/horloge.rs` (la constante, et l'attribut `should_panic` de son
    test, qui exige un littéral) ; 0 ailleurs ;
  - les occurrences de **`NOM_VARIABLE_INSTANT`** (validation P2, V2-4) : `kesh-core/src/horloge.rs` (définition, message de
    panique, tests : compte relevé), `kesh-api/src/horloge.rs` (dans `horloge_de_test`, son import
    compris : compte relevé), `kesh-db/tests/support/jour.rs`, `kesh-report/tests/support/jour.rs` (comptes relevés) ; **0 ailleurs, `main.rs` et
    `lib.rs` compris** ;
  - les occurrences de **`instant_de_test`** : `kesh-core/src/horloge.rs` (définition et tests), `kesh-api/src/horloge.rs`
    (dans `horloge_depuis`, import compris), les deux `support/jour.rs` : comptes relevés ; 0 ailleurs ;
  - les occurrences de **`horloge_depuis`** dans `crates/*/src` : `kesh-api/src/horloge.rs` (définition, appel par
    `horloge_de_test`, tests du module : compte relevé) ; 0 ailleurs ;
  - **`new_for_tests`** (mot entier, validation P5, V5-1 : `let f = AppState::new_for_tests;` ne contient pas
    `new_for_tests(`) et **`horloge_de_test`** dans `crates/*/src` : `lib.rs` (définition de
    `new_for_tests`, appel et import éventuel de `horloge_de_test` : compte relevé), `kesh-api/src/horloge.rs` (définition de
    `horloge_de_test` : 1) ; 0 ailleurs — aucun appel ni import de production, ni de `mod tests` de `src` ;
  - `HorlogeSysteme` **construit** : instructions `use` retirées (ci-dessus, pour ce seul compte), puis la ligne `pub struct HorlogeSysteme` et la
    ligne d'en-tête `impl … for HorlogeSysteme` ; ce qui reste nomme une construction : `main.rs` 1, `kesh-api/src/horloge.rs`
    1 (dans `horloge_depuis` ; ses tests ne le nomment pas, sinon compte relevé), 0 ailleurs ;
  - **`HorlogeFixe`** dans `crates/*/src` (V2-5) : `kesh-core/src/horloge.rs` (définition `pub struct HorlogeFixe(` et tests :
    compte relevé), `kesh-api/src/horloge.rs` (dans `horloge_depuis`, import compris : compte relevé),
    `kesh-api/src/middleware/auth.rs` (1, `test_state()` du `mod tests`), et les fichiers de `src` dont le `mod tests` date par `HorlogeFixe(Utc::now())` (D4 ; `journal_entries.rs`, qui appelle
    `reverse_in_tx` à `:3984` et `:5824`, en est un si son test n'a pas de date fixe) : comptes relevés, en égalité stricte —
    une construction ajoutée en production dans l'un d'eux fait `+ 1` et rougit ; **0 dans `main.rs`, `lib.rs` et tout autre
    fichier de `src`** — une `HorlogeFixe(Utc::now())` de production figerait le jour au démarrage ;
  - aucun `static`, `OnceLock`, `LazyLock`, `lazy_static!`, `thread_local!` dans les **fichiers** `kesh-core/src/horloge.rs`,
    `kesh-api/src/horloge.rs`, `tests/common/jour.rs` et les deux `tests/support/jour.rs`.

  Ce que ces listes ferment, ensemble : `main.rs` ne peut **ni importer ni nommer** — alias `as` compris, puisque l'import
  compte — le littéral, la constante, `instant_de_test`, `horloge_depuis`, `horloge_de_test` ou `HorlogeFixe`, ni appeler
  `new_for_tests` ; il construit `HorlogeSysteme` une fois (seul compte où les `use` sont retirés, et un alias de
  `HorlogeSysteme` n'ouvre rien : c'est l'horloge de production). **Angle mort assumé** : un autre nom de variable d'environnement, lu en production et converti
  à la main en `DateTime` passé à une horloge écrite pour l'occasion — ce n'est plus `KESH_TEST_INSTANT`, et ce code
  n'aurait plus rien de la plomberie de test ; la revue fichier par fichier de `main.rs` et de `lib.rs` le voit.
- **D6 — Docs** : une ligne dans `docs/testing.md` — `KESH_TEST_INSTANT` est une variable **de tests**, lue par
  `horloge_de_test()` et les helpers, sans effet sur le binaire servi (`main.rs` construit `HorlogeSysteme`) ; vide, elle vaut
  absente ; invalide, elle arrête le test. Aucun texte d'interface, aucun manuel, aucun CHANGELOG (aucun changement visible).

**« Comportement inchangé », précisément** (V1-8) : même jour (UTC), mêmes refus, mêmes écritures — à une nuance près, voulue :
la date est lue **une fois** à l'entrée de la requête (avant l'attente d'un verrou et les reprises `retry_on_deadlock`) et
partagée par le bloqueur et la contre-passation, où elle était lue deux fois à l'intérieur. L'écart n'existe que dans une
fenêtre de quelques secondes autour de minuit UTC, et va dans le sens voulu.

## Critères d'acceptation

1. **AC1 — Rien ne change** : le gate complet (`test-fast.sh --ci`), `scripts/gate-ci.sh`, Vitest et l'E2E complet sont verts
   **sans qu'aucun attendu de comportement ne change** — date, montant, statut, code d'erreur, refus. Seuls changent,
   **nommément**, les attendus **lexicaux** des gardes que D3 réécrit : `mcp_gardes_lexicales.rs:169` et `:575` (la ligne
   `mcp::LectureOutils::nouvelle(…)` du câblage, qui passe à trois lignes), `services_motif.rs:2071` et `:2083`
   (`FABRIQUES_V2`, `Lecture::new(self.pool.clone())` qui prend l'horloge) ; et une entrée neuve (la ligne `horloge_outils`,
   liste voisine de `LIB_NOMME_MCP`). **`<base>`**, ici et dans la *Dérogation*, est **le parent du premier commit de la
   story** (`<commit (1)>^`), pour que le contrôle ne dépende pas de la branche (validation P3, V3-7). Contrôle en deux temps
   (validation P2, V2-3), au Change Log :
   - **fichiers modifiés**, lignes retirées seulement : `git diff -U0 --diff-filter=M <base> -- 'crates/*/tests/*' 'crates/*/src/*'
     | /usr/bin/grep -E '^-' | /usr/bin/grep -vE '^---' | /usr/bin/grep -E 'assert|from_ymd_opt|expect_err|StatusCode::'` —
     chaque ligne rendue est **listée** au Change Log avec sa ligne `+` correspondante et sa raison, parmi trois : un argument
     ajouté (date, horloge) ; un reflux de rustfmt (même expression recoupée, sens inchangé) ; un attendu de garde lexicale
     de la liste ci-dessus. Toute autre raison est un défaut de cette story. **Limite écrite** : un attendu porté par une
     ligne qui ne contient aucun de ces jetons (une chaîne `"2025-06-15"` seule, la queue d'un `assert!(` sur plusieurs
     lignes) échappe à ce filtre ; le gate reste le juge du comportement ;
   - **fichiers neufs** (`--diff-filter=A` : `date_du_jour_garde.rs`, les helpers `jour.rs`, `kesh-core/src/horloge.rs`,
     `kesh-api/src/horloge.rs`, tests d'AC2 et d'AC3) : listés à part ; leurs `assert` sont neufs par construction et ne
     relèvent pas de ce contrôle.
2. **AC2 — L'horloge fixe atteint les sites** : montage `seed_accounting_company` (exercice unique **2020-2030 ouvert**,
   `kesh-db/src/test_fixtures.rs:71`, `:119`, lu), qui couvre le `2025-12-31`, le `2026-01-01` et le `2026-06-15`, si bien que
   le test que la 35-5a retourne reste une écriture et ne devient pas un refus d'exercice (validation P5 de la 35-5a,
   V5-3) ; l'écriture contre-passée est datée d'un jour antérieur (par exemple `2025-06-01`). Un `AppState` à
   `HorlogeFixe(2026-06-15T10:00:00Z)` date une contre-passation du
   `2026-06-15` et refuse un verrou au `2026-06-15` ; à `HorlogeFixe(2025-12-31T23:30:00Z)`, la contre-passation est datée du
   `2025-12-31` (**le jour UTC** : comportement d'avant, voulu ici) — ce test est celui que la 35-5a retourne. AC2 n'exerce
   que **2 des 9 sites** ; les sept autres (soldes d'ouverture ×4, rang 5, `unlock_books`, assistant) ne sont tenus ici que par
   la garde de forme (D5), et par les critères de la 35-5a (V1-8).
3. **AC3 — Le MCP reçoit la même horloge** : sur un état de `new_for_tests` dont on remplace l'horloge par
   `state.horloge = Arc::new(HorlogeFixe(2025-06-15T10:00:00Z))` — un jour distinct du jour réel, pour que l'égalité ne tienne
   pas par deux horloges système (validation P2, V2-6) —, `state.lecture().aujourd_hui() == 2025-06-15` ; les gardes
   lexicales du câblage sont vertes ; la mutation « câblage du MCP sur `HorlogeSysteme` » est tuée par la garde de
   construction de `HorlogeSysteme` (D5). Aucun outil du MCP n'expose un site de jour (`factures_fournisseur.rs:206` jette le
   motif) : la garde est **le seul filet** du câblage dans cette story.
4. **AC4 — `KESH_TEST_INSTANT`** : `instant_de_test(Some("2025-12-31T23:30:00Z"))` rend l'instant ; `None` et `Some("")`
   rendent `None` ; une valeur invalide `panic!` en citant la constante (`#[should_panic(expected = "KESH_TEST_INSTANT")]`,
   l'attribut exigeant un littéral : le second et dernier littéral admis, dans le `mod tests` de `horloge.rs`) ; `horloge_depuis(Some("2025-12-31T23:30:00Z")).maintenant()` rend l'instant,
   et `horloge_depuis(None)` n'est pas figée : deux lectures de `maintenant()` séparées par un
   `std::thread::sleep(Duration::from_millis(10))` sont **différentes** (`assert_ne!`) — une horloge figée rend deux valeurs
   égales ; un réglage NTP peut faire reculer la seconde, non la rendre égale (validation P4, V4-3 ; l'ordre, lui, n'est pas
   asserté, V3-7). La garde de construction de `HorlogeSysteme` (D5) reste le filet de la forme : une branche `None` qui
   rendrait une `HorlogeFixe` ferait, dans `kesh-api/src/horloge.rs`, `HorlogeSysteme` 0 ou `HorlogeFixe` un de plus.
5. **AC5 — La garde** : verte avec les comptes de son exécution ; sondes au Change Log (forme ajoutée, forme coupée, compte
   + 1, `KESH_TEST_INSTANT` dans un fichier de plus, `HorlogeSysteme` construit ailleurs, `OnceLock` dans un helper de jour,
   appel de `new_for_tests(` dans `src`, **`NOM_VARIABLE_INSTANT` nommé dans `main.rs`**, `instant_de_test(` ou
   `horloge_depuis(` appelé dans `main.rs`, `HorlogeFixe(` construit dans `main.rs`, et une ligne `// NOM_VARIABLE_INSTANT`
   ajoutée qui, elle, **ne** rougit **pas** — commentaires écartés ; et, **verte** aussi, un import groupé sur plusieurs
   lignes ajouté à `main.rs`, `use kesh_api::{` / `horloge::HorlogeSysteme,` / `};`, qui ne compte pas comme construction —
   instructions `use` retirées pour ce seul compte, validation P3, V3-2 ; et, **rouge**, un import renommé dans `main.rs`,
   `use kesh_api::horloge::horloge_de_test as h;` — l'import compte, validation P4, V4-1).
6. **AC6 — Aucune migration** ; `crates/kesh-db/migrations/` intact.

## Tests à écrire, et la preuve du rouge

AC2 à AC5 rouges tant que la plomberie n'est pas posée (ils ne compilent pas, puis échouent par attendu) ; AC1 est l'absence
de rouge. Journal `kesh-gate-logs/e355a0/rouges.log`.

**Mutations** : M1 un site de la 35-5a qui garde `Utc::now()` (tuée par la **garde** pour les neuf sites, le fichier repassant à
un compte non nul ; par AC2 aussi pour la contre-passation et le verrou) ; M2 le câblage du MCP sur `HorlogeSysteme` (garde de
construction, AC3) ; M3 `horloge_de_test` mémorisée dans un `OnceLock` (garde D5) ; M4 le bloqueur et la contre-passation qui
lisent deux fois l'horloge (un test du dépôt donne deux dates distinctes au chemin qui n'en devrait recevoir qu'une : il ne
compile pas, ou la garde le voit) ; M5 `instant_de_test(Some(""))` qui panique (AC4) ; M6 (validation P2, V2-4) `main.rs`
construit son horloge par `match instant_de_test(std::env::var(NOM_VARIABLE_INSTANT).ok().as_deref()) { Some(i) =>
Arc::new(HorlogeFixe(i)), None => Arc::new(HorlogeSysteme) }` — `HorlogeSysteme` y reste construit une fois et le littéral
absent : tuée par les gardes de `NOM_VARIABLE_INSTANT`, d'`instant_de_test` et d'`HorlogeFixe`, en mots entiers (trois rouges
attendus).

## Tâches

- [x] Étape 0 : recompter la fermeture des appelants (production, tests) par `git grep` sur les noms de D2 ; les 43
      constructions de `Lecture`, les 9 littéraux ; Change Log.
- [x] Commits internes, **dans cet ordre** (validation P2, V2-1 : l'ordre précédent mettait `kesh-db` avant `kesh-api`, qui
      appelle la fermeture — `routes/companies.rs:505`, `journal_entries.rs:456`, `services/factures.rs:552`,
      `supplier_invoices.rs:829`, lus — et ne compilait pas). Chacun compile et passe le gate complet ; le raisonnement est
      écrit ici, et l'étape le constate (`cargo build --workspace --all-targets` puis gate complet, au Change Log) :
      1. **`kesh-core`** : `horloge.rs` (D1) et `pub mod horloge;`. Rien ne l'appelle encore ; `chrono` est déjà une
         dépendance de `kesh-core`. Compile seul.
      2. **Plomberie `kesh-api`, sans toucher `kesh-db`** (D3, hors les neuf sites) : `kesh-api/src/horloge.rs`
         (`HorlogeSysteme`, `horloge_depuis`, `horloge_de_test`) ; le champ `AppState.horloge` et **ses 9 littéraux dans le même
         commit** (un littéral de structure sans le champ ne compile pas : `main.rs:439`, le `Self {` de `new_for_tests`,
         `middleware/auth.rs:285`, et les six des tests d'intégration) ; `Lecture { pool, horloge }` et **toutes** ses
         constructions (`AppState::lecture()`, câblage du MCP, les 43 des tests) ; les attendus lexicaux des gardes que ces
         lignes changent (`mcp_gardes_lexicales.rs:169`, `:575`, `services_motif.rs:2071`, `:2083`, ligne `horloge_outils`),
         sans quoi le gate rougit ; les tests d'AC3 et d'AC4. `Lecture::aujourd_hui()` / `horloge()` sont `pub`, donc sans
         avertissement `dead_code` ; les neuf sites lisent encore l'horloge dans `kesh-db`, inchangé. Compile, car aucune
         signature de `kesh-db` ne bouge.
      3. **Le noyau D2, avec tous ses appelants** : les signatures de la fermeture (`aujourd_hui: NaiveDate` en dernier
         argument) ; ses appelants internes à `kesh-db` ; les appelants de production de `kesh-api` (les neuf sites lisent
         `state.horloge` / `lecture`, `finalize_inner` reçoit la date, `services::factures::reglements` la lit une fois) ; les
         appelants de **tests des trois crates** (`kesh-db`, `kesh-api`, `kesh-report`) et des `mod tests` de `src` ; les
         helpers `aujourd_hui_test()` (`common/jour.rs` et `pub mod jour;` dans `common/mod.rs`, les deux `support/jour.rs`,
         les inclusions `#[path]`), posés **avec leurs premiers appelants** — un fichier qu'aucun binaire n'inclut n'est pas
         compilé, et l'y mettre plus tôt ne prouverait rien ; le commentaire `companies.rs:986-989` ; le test d'AC2.
         Compile une fois tous les appelants à jour (le compilateur donne la liste close). Une signature changée oblige
         tous ses appelants ; seule la chaîne contre-passation → bloqueurs → vues est insécable, **par la règle D2** (une lecture
         par requête, passée au bloqueur et à la contre-passation). `lock_books` /
         `unlock_books`, qui ne sont appelées par aucune fonction de cette chaîne, pourraient faire un commit (3a) avant
         elle ; elles restent dans (3) par économie (Dérogation).
      4. **Garde (D5) et `docs/testing.md` (D6)** : fichier de test neuf qui lit les sources, sondes de l'AC5, mutations ;
         ses comptes supposent les neuf sites déjà convertis, d'où sa place après (3).
- [x] Gates : `test-fast.sh --ci` complet à chaque commit, `scripts/gate-ci.sh`, Vitest, E2E complet au dernier commit de code.
      *(Fait au commit (2) et au commit (5), dernier commit de code ; commits (3) et (4) jugés par le gate du (5) — écart
      déclaré au Change Log, C-e355a0-5.)* *(Annoté le 2026-11-13, remédiation P1, A1 : au commit (2), `test-fast` seul —
      `gate-ci.sh` n'y a pas tourné ; gate complet au (5) seulement ; détail commit par commit dans l'annotation de
      C-e355a0-5 et au Change Log. Le gate du commit de la remédiation y est noté.)*
- [ ] Revue : **fichier par fichier** pour les appelants (rollout mécanique, `CLAUDE.md` § *Règle de splitting préventif*),
      passes adversariales sur `kesh-core/src/horloge.rs`, `kesh-api/src/horloge.rs`, D2 et la garde.

## Dev Notes

### Dérogation règle de splitting

*(Récrite à la validation P2, V2-2 : la version précédente disait la story entière indivisible, alors que son propre plan de
commits la montrait divisible.)*

**Ce qui est indivisible** (récrit à la validation P3, V3-5 : la version P2 disait « et seulement cela » du noyau entier, et
« aucun commit plus fin n'est possible », ce qui est faux) : la **chaîne contre-passation → bloqueurs → vues** du noyau D2 —
`journal_entries`, `settlement_cancellation`, `invoice_settlements_write`, `reconciliation_cancel`, `supplier_invoices` et
leurs routes —, avec **tous** ses appelants. Une signature changée oblige chacun de ses appelants, production et tests, dans
le même commit. La chaîne, elle, se couperait encore à la compilation (bloqueurs dans un commit, contre-passation dans un
autre) : c'est **la règle D2** qui la rend insécable — une coupe laisserait un état qui ne compile pas, **ou** qui lit
l'horloge deux fois par requête, contraire à D2 (validation P4, V4-5). Elle dépasse seule cinq modules. Le noyau (3)
pèse la quasi-totalité des **50 fichiers** de la fermeture (dont **36 de tests**, § *Ce qui se lit*).

**Ce qui serait séparable** : (1) `kesh-core` et (2) la plomberie de `kesh-api` (les **18** fichiers de `Lecture::new(` et des
littéraux d'`AppState {` hors fermeture, 68 − 50) compilent et se testent sans le noyau ; **`lock_books` / `unlock_books`**
(`companies.rs:1013`, `:1100`), qui lisent l'horloge elles-mêmes et ne sont appelées par aucune fonction de la chaîne
(production : `routes/companies.rs:505`, `:518`, lus), feraient un commit (3a) compilable avant elle ; la garde (4) aussi se
sépare. La coupe (2)/(3) n'est donc pas la seule possible.

**Choix : la story reste une, en quatre commits** — **décision de l'orchestrateur** (validation P3, V3-5), annotée à C-e350-28.
Les commits (1), (2) et (4), et `lock_books` / `unlock_books` dans (3), restent dans la story **pour un motif de coût**, que
l'exception de `CLAUDE.md` (cycles Cargo, fusions intermédiaires impossibles à tester) ne nomme pas : ce motif est **assumé
par l'orchestrateur**, non tiré de l'exception. Le coût évité : chaque story de plus coûte une boucle de validation, un cycle
de CI complet et un E2E, pour un diff du noyau qui ne diminue pas ; (1) et (2) seuls livrent une horloge que personne ne lit
(leur seule preuve serait AC3 et AC4) ; la garde (4) n'a de comptes qu'après (3). Les quatre commits se relisent séparément
(revue fichier par fichier, commit par commit). Réversible : la story se coupe entre (2) et (3), ou avant (3a), sans
réécriture.

**Risque accepté** (exigé par `CLAUDE.md`) : un commit (3) d'environ cinquante fichiers, où l'erreur la plus probable est
**muette** — un appel de test qui passe comme `aujourd_hui` une date de pièce au lieu du jour (V2-7) compile, reste vert, et
change seulement ce que le test exerce ; le contrôle de l'AC1 ne le voit pas (il ne lit que les lignes retirées). Ce risque
est accepté, avec trois filets : la règle de D4 (jamais une date de pièce) ; **deux relevés** au Change Log, après (3), sur
`git diff -U0 --diff-filter=M <base> -- 'crates/*/tests/*' 'crates/*/src/*'` (validation P3, V3-1 : avec `-U0`, un appel
écrit sur plusieurs lignes ne change que par **une ligne d'argument ajoutée**, la ligne qui nomme la fonction restant du
contexte, absente du diff — un filtre sur les noms serait faussement vide sur la forme dominante) :
- **appels sur une ligne** : `… | /usr/bin/grep -E '^\+'` filtré sur les noms de la fermeture, dernier argument lu ;
- **lignes d'argument seules, dédoublonnées** : `… | /usr/bin/grep -E '^\+\s+[^/[:space:]].*,\s*$' | /usr/bin/grep -vE
  '^\+\s*(fn|let|use|pub|impl)\b' | sed -E 's/^\+\s+//' | sort | uniq -c` — une liste courte de jetons distincts
  (`aujourd_hui_test(),`, `today,`, …). Simulé à la rédaction sur `support/fournisseur_rapproche.rs:155-165` (copie hors
  dépôt, une ligne `aujourd_hui_test(),` insérée dans l'appel de `pay(`, `git diff --no-index -U0`) : le relevé rend
  `1 aujourd_hui_test(),`, que le filtre sur les noms ne rendait pas.

Dans les deux, chaque jeton qui n'est ni `aujourd_hui_test()`, ni une variable du jour du montage, ni une date fixe de test
justifiée est listé et motivé (un `jour,` ou un `le,` ajouté est le piège de D4) ; puis la revue fichier par fichier. Une régression de comportement (date, refus) reste visible au gate et aux gardes. Le signal de
découpage est déclaré au Project Lead (Change Log, validation P2).

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
les fait passer avant la 35-5a0, qui se rebase sur elles. Avec la **35-2b** : `invoice_send_email_e2e.rs` (la 35-2b y ajoute
un test, AC1 de sa fiche, `35-2b-lot-rappels-refus-pdf.md:41` ; ici les trois littéraux `AppState {` `:104`, `:1036`, `:1150`) ; avec la **35-2a** :
`invoice_send_email_e2e.rs` et `invoice_frozen_pdf_e2e.rs` (`35-2a-refus-pdf-contrat-details.md:224-225` ; ici le littéral `:101`) — hunks distincts, couverts par la règle
générale ci-après (validation P3, V3-7). **Tests neufs des stories sœurs** : tout appel neuf à une fonction de
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

Opus 5.5 (agent de développement, worktree `kesh-e355a0`, clé `e355a0`), 2026-11-13.

### Debug Log References

Journaux sous `/home/gcorbaz/devel/kesh-gate-logs/e355a0/` : `etape0-*.log` (étape 0), `c2-*.log` (gate du commit (2)),
`c3-erreurs.json` / `c3-conversion.log` (diagnostics et conversion du commit (3)), `garde-comptes-bruts.log` (comptes de
la garde relevés à son exécution, listes vides), `c5-*.log` (gate complet du dernier commit de code), `mut/` (mutations),
`rouges.log`. Gates de commit sur le clone `kesh-gate-logs/e355a0/gwt` (C-e355a0-1).

### Completion Notes List

- Fiche rédigée au découpage de la 35-5a (C-e350-28) ; validation terminée en P5 ciblée (Change Log).
- Développement en **cinq** commits — les quatre de la fiche, plus un correctif (5) né du gate du commit (2)
  (C-e355a0-7) : `48b9e627` (1) `kesh-core`, `05666dbf` (2) plomberie `kesh-api`, `5426cb8b` (3) noyau D2 et appelants,
  `79ec67b4` (4) garde et `docs/testing.md`, `0deb9abc` (5) `configuration_transmise`. **Aucune migration**
  (`git diff --stat 2b1c0ec8 HEAD -- crates/kesh-db/migrations` vide). Aucun texte d'interface, manuel ni CHANGELOG (D6).
- Choix consignés au registre `e35-choix-autonomes-e355a0.md` : C-e355a0-1 à -7 (C-e355a0-3 et -5 annotés à la
  remédiation de la revue P1).

#### Transmis à la 35-5a (remédiation de la revue P1, A3)

- **`configuration_transmise.rs`** nomme `NOM_VARIABLE_INSTANT` **deux fois** (la lecture admise `env_nonempty (
  NOM_VARIABLE_INSTANT )` de `horloge_de_test`, en forme exacte, et la liste fermée `VARIABLES_DE_TEST`) : la garde de la
  date du jour l'inscrit à 2 (C-e355a0-7). Toute retouche de ces deux listes se répercute là.
- **`horloge_de_test` lit par `config::env_nonempty`**, non par `std::env::var` (écart à D3) : vide ou blanche, la
  variable vaut absente ; `instant_de_test` rogne aussi la valeur depuis la remédiation P1, si bien que les helpers de
  `kesh-db` et `kesh-report` disent la même chose.
- **E3, écart voulu** : la date est lue une fois à l'entrée de la requête ; une contre-passation entrée à 23 h 59 59 UTC le
  31 décembre, qui attend le verrou d'une clôture de l'exercice en cours, est refusée (`FiscalYearInvalid`) là où l'ancien
  code, relisant l'heure après l'attente, la datait du 1er janvier. Fenêtre de quelques secondes, dans le sens voulu (« lue
  une fois », fiche) ; aucun test. À la 35-5a, qui fait passer ce jour à Zurich, de dire si le cas mérite le manuel.
- **Montages et horloge** : D4 est tenue aux sites dont la fonction englobante tenait son jour (remédiation P1) ; les
  montages qui alimentent un helper (`figer`, `supplier_invoices_repository.rs`) restent à l'heure réelle — la 35-5c.
  `homepage_summary_e2e.rs` passe son `today()` (heure réelle) à ses contre-passations : il change avec le site 21, 35-5b.
  *(Annoté le 2026-11-13, remédiation de la revue P2, C1 : la phrase sur `figer` est fausse — l'inventaire parti du
  symptôme a converti ces montages (`monter`, `monter_achat`, `:2979`), et avec eux `jours_avant` du support, `aujourdhui`
  d'`open_items.rs` et les autres sites de l'entrée P2 du Change Log. **Reste à la 35-5c** : les tests qui lisent l'heure
  réelle **sans** porter de site converti (liste close de la garde) ; **à la 35-5b** : `homepage_summary_e2e.rs`, dont
  `today()` est relu à chaque appel, montage compris (`year_start()`, `setup`) — revue P2, C2.)*
- **La `Lecture` des outils du MCP** n'est tenue que lexicalement (gardes `horloge_outils` et `FABRIQUES_V2`) : aucun outil
  n'expose un site de jour dans cette story.
- **B2 → #1105** : « lue une fois, avant le rejeu » n'est tenu par aucun filet (tracé, non traité ici).

### File List

**Neufs** : `crates/kesh-core/src/horloge.rs`, `crates/kesh-api/src/horloge.rs`, `crates/kesh-api/tests/{common/jour,
date_du_jour_garde,horloge_etat,horloge_routes_e2e}.rs`, `crates/kesh-db/tests/support/jour.rs`,
`crates/kesh-report/tests/support/jour.rs`, `_bmad-output/implementation-artifacts/e35-choix-autonomes-e355a0.md`.

**Modifiés** (production) : `crates/kesh-core/src/lib.rs` ; `crates/kesh-api/src/{lib,main}.rs`, `middleware/auth.rs`,
`services/{poignees,factures,factures_fournisseur}.rs`,
`routes/{bank_accounts,companies,imported_supplier_invoices,invoices,journal_entries,onboarding,opening_balances,reconciliation,supplier_invoices}.rs` ;
`crates/kesh-db/src/repositories/{companies,invoice_settlements_write,journal_entries,reconciliation_cancel,settlement_cancellation,supplier_invoices}.rs`.
(`kesh-core/Cargo.toml` inchangé : `chrono` y était.)

**Modifiés** (tests) : `crates/kesh-api/tests/` — les six littéraux d'`AppState` (`invoice_frozen_pdf_e2e`,
`invoice_send_email_e2e` ×3, `password_recovery_e2e`, `setup_admin_e2e`), les 43 `Lecture::new(` (`services_rollout`,
`services_motif`, `cles_mcp_e2e`, `mcp_prompts_e2e`, `onboarding_audit_e2e`, `admin_full_import_e2e`,
`common/lecture_seule`, `mcp_admin_routes_e2e`, `mcp_envois_e2e`, `mcp_journal_e2e`), les gardes `mcp_gardes_lexicales`,
`services_motif`, `configuration_transmise`, `common/mod.rs`, et les appelants de la fermeture
(`accounts_designated_archive_e2e`, `admin_full_import_e2e`, `bank_accounts_closed_e2e`, `homepage_summary_e2e`,
`invoice_echeancier_e2e`, `letterings_e2e`, `mcp_outils_e2e`, `reconciliation_e2e`, `supplier_invoice_draft_e2e`,
`supplier_settlement_cancel_e2e`) ; `crates/kesh-db/tests/` — `credit_notes_repository`, `filet_bilan_clos`,
`invoice_amount_due_parity`, `invoice_settlement`, `invoice_write_off`, `journal_entries_modification`,
`lettering_documents`, `lettering_documents_backfill`, `letterings`, `migrations_upgrade_path`, `open_items`,
`opening_complement_repository`, `payment_batches_repository`, `reconciliation_cancel`, `supplier_document_attachment`,
`supplier_invoices_draft`, `supplier_invoices_internal_number`, `supplier_invoices_repository`,
`support/{fournisseur_rapproche,lettering_documents}` ; `crates/kesh-report/tests/{aged_receivables,open_items_invariant,
vat_report_write_off}.rs`.

**Docs** : `docs/testing.md` (D6).

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
- 2026-11-12 — **Validation P2** (Opus, `kesh-gate-logs/e35-5a0-validation-p2.md` : 0 CRITICAL, 0 HIGH, 4 MEDIUM, 4 LOW ;
  série 3 → 4 MEDIUM) remédiée par l'agent de rédaction (worktree `kesh-e355`, relève de l'agent précédent). Prémisses lues
  au code de `release/0.17.6` (`92ccb6ef`).
  - **V2-1** : ordre des commits refait — (1) `kesh-core`, (2) plomberie `kesh-api` sans toucher `kesh-db` (9 littéraux,
    43 + 2 constructions de `Lecture`, attendus des gardes lexicales, AC3, AC4), (3) noyau D2 avec **tous** ses appelants
    (production, tests des trois crates, `mod tests` de `src`, helpers posés avec leurs premiers appelants), (4) garde et
    docs ; la compilation de chaque étape est raisonnée sur les appels (*Tâches*). Prémisse lue : `routes/companies.rs:505`,
    `journal_entries.rs:456`, `services/factures.rs:552`, `supplier_invoices.rs:829` appellent la fermeture.
  - **V2-2** : *Dérogation* récrite — seul le noyau D2 est indivisible (≈ 50 fichiers, dont 36 de tests) ; (1), (2) et (4)
    seraient séparables ; choix motivé de garder la story unique en quatre commits ; **risque accepté** écrit (date de pièce
    passée comme jour, muette), avec un relevé des derniers arguments au Change Log de dev.
  - **V2-3** : AC1 — « aucun attendu **de comportement** » ; les attendus lexicaux des gardes de D3 nommés un à un ; contrôle
    en deux temps (`--diff-filter=M`, lignes `-` seulement, chaque ligne listée avec sa raison ; fichiers neufs à part).
  - **V2-4** : gardes de D5 en listes closes à égalité stricte, `0 ailleurs` : `NOM_VARIABLE_INSTANT`, `instant_de_test(`,
    `horloge_depuis(`, `HorlogeFixe(` (0 dans `main.rs` et `lib.rs`) ; sondes de l'AC5 dans `main.rs` ; **M6** (`main.rs` lit
    la variable par la constante, `HorlogeSysteme` en repli : trois gardes rougissent).
  - **LOW** : V2-5 (lignes `//` écartées pour toutes les gardes de D5 ; `HorlogeFixe` de production tenu par sa garde) ;
    V2-6 (AC3 sur `HorlogeFixe(2025-06-15T10:00:00Z)`, AC4 strictement croissant autour d'un `sleep` de 5 ms) ; V2-7 (fichiers
    `support/` qui appellent la fermeture nommés, avec leurs huit binaires ; inclusion `#[path]` imbriquée, à constater à
    l'étape 0, repli par paramètre ; D4 : « la date **du jour**, jamais une date de pièce » — `lettering_documents.rs:870`,
    où `le` est la date du paiement, prend `aujourd_hui_test()`) ; V2-8 (`Cargo.toml:16`, lu ; un seul décompte de fichiers,
    recompté : 50 dont 36 de tests, 68 avec `Lecture::new(` / `AppState {` — la passe en donnait 67, écart d'un fichier
    laissé au recompte de l'étape 0, commande écrite).
  - Depuis la validation P5 de la 35-5a, **V5-3** : AC2 nomme son montage, `seed_accounting_company`, exercice 2020-2030
    ouvert (`test_fixtures.rs:71`, `:119`, lus) : le test que la 35-5a retourne reste une écriture au `2026-01-01`.
  - Aucun finding réfuté.
  - **Signal de découpage** : levé — sévérité égale ou supérieure (3 → 4 MEDIUM) **et recyclage** : les quatre MEDIUM naissent
    tous de la remédiation de P1 (V2-1, V2-2, V2-3 de celle de V1-3 — ordre des commits, dérogation, commande de l'AC1 ; V2-4 de
    celle de V1-1 — garde de la constante), aucun ne vient de la conception D1-D3. **Non suivi** au-delà du découpage en
    commits, motivé à la *Dérogation* (C-e350-28, annoté) : les défauts recyclés sont dans la prose de contrôle de la fiche,
    qu'un découpage en stories ne supprimerait pas. **Déclaré au Project Lead** par le rapport à l'orchestrateur, qui peut
    couper entre les commits (2) et (3). La passe suivante, sur cette remédiation, est à lancer. Aucune migration. Statut
    `ready-for-dev`.
- 2026-11-12 — **Validation P3** (Sonnet, `kesh-gate-logs/e35-5a0-validation-p3.md` : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 5 LOW ;
  série 3 → 4 → 2 MEDIUM) remédiée par l'agent de rédaction (worktree `kesh-e355`). Prémisses lues au code de
  `release/0.17.6` (`92ccb6ef`).
  - **V3-1** : le filet du risque accepté devient deux relevés — appels sur une ligne (filtre sur les noms) et **lignes
    d'argument seules, dédoublonnées** (`sort | uniq -c`). Simulé hors dépôt sur une copie de
    `support/fournisseur_rapproche.rs:155-165` (`git diff --no-index -U0`) : le relevé rend la ligne d'argument ajoutée, que le
    filtre sur les noms ne voyait pas.
  - **V3-2** : gardes de durcissement **par instruction** — instructions `use … ;` multi-lignes retirées avant le compte
    (`main.rs:41-49`, import groupé, lu) ; définitions comptées dans les motifs en `nom(`, comptes écrits ; sonde de l'AC5
    « import groupé de `HorlogeSysteme` dans `main.rs` = vert ».
  - **LOW** : V3-3 (`Cargo.toml:17`, relu : `cat -n Cargo.toml` rend `17 edition = "2024"` ; la remédiation P2 avait remplacé
    à tort la citation juste de P1 — l'entrée P2 ci-dessus, qui dit `:16`, est fausse **à sa date** et reste telle quelle) ;
    V3-4 (`cancel_settlement_in_tx` défini à `:912`, `:1023` est son appel interne ; `:912` retiré des bloqueurs ;
    `/usr/bin/grep -nE '^pub async fn' invoice_settlements_write.rs` lu) ; V3-6 (D4 nomme aussi `payer` `:880` et
    `facture_payee` / `pay` de `fournisseur_rapproche.rs:120-127`, `:155-162`, dont la variable `jour` est la date de la
    pièce) ; V3-7 (`<base>` = parent du premier commit de la story ; limite du filtre de l'AC1 écrite ; AC4 sans comparaison
    d'horloge murale, la branche `None` tenue par les gardes de `kesh-api/src/horloge.rs` ; 35-2a et 35-2b nommées aux
    fichiers communs, `35-2b-…:41`, `35-2a-…:224-225`).
  - **V3-5** (décision de l'orchestrateur) : la story reste une, en quatre commits. La phrase « aucun commit plus fin n'est
    possible » est corrigée — `lock_books` / `unlock_books` sont séparables (commit (3a) possible) ; seule la chaîne
    contre-passation → bloqueurs → vues est insécable. Les commits (1), (2), (4) restent dans la story pour un **motif de coût,
    assumé par l'orchestrateur** (annotation de C-e350-28), non tiré de l'exception de `CLAUDE.md`.
  - Aucun finding réfuté. Signal de découpage : sévérité en baisse (4 → 2 MEDIUM), mais les deux MEDIUM sont encore **nés de
    la remédiation P2** (le filet du risque accepté, les gardes à listes closes) : recyclage, déclaré ; découpage tranché par
    l'orchestrateur (V3-5). Aucune migration. Statut `ready-for-dev` ; la passe suivante est à lancer.
- 2026-11-12 — **Validation P4, passe ciblée** (Opus, `kesh-gate-logs/e35-5a0-validation-p4-ciblee.md` : 0 CRITICAL, 0 HIGH,
  1 MEDIUM, 4 LOW ; série 3 → 4 → 2 → 1 MEDIUM) remédiée au plus court par l'agent de rédaction (worktree `kesh-e355`).
  - **V4-1** (né de la remédiation P3) : le retrait des `use` ne vaut que pour le compte de `HorlogeSysteme` construit ; les
    autres gardes comptent les `use` compris, noms comme mots ; sonde de l'AC5 « import renommé `as` dans `main.rs` = rouge » ;
    « Ce que ces listes ferment » récrit.
  - **LOW** : V4-2 (retrait borné : début de ligne, `use` / `pub use` / `pub(…) use`, jusqu'au premier `;` hors commentaire,
    lignes `//` écartées avant) ; V4-3 (AC4 : deux lectures séparées par un `sleep` de 10 ms, `assert_ne!`) ; V4-4 (`payer_par`
    `support/lettering_documents.rs:863`, appel `:870` ; `payer` `:879` transmet `le` et ne change pas — lus) ; V4-5 (la chaîne
    est insécable par la règle D2, non par la compilation).
  - Aucun finding réfuté. Recyclage déclaré (V4-1 né de V3-2). Aucune migration. Statut `ready-for-dev` ; passe ciblée P5 à
    lancer.
- 2026-11-12 — **Validation P5, passe ciblée** (Sonnet, sur la remédiation `eb52ac22` ;
  `kesh-gate-logs/e35-5a0-validation-p5-ciblee.md` : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 2 LOW), LOW appliqués par l'agent de
  rédaction : **V5-1** (`new_for_tests` compté en mot entier, `\bnew_for_tests\b`) ; **V5-2** (`payer` transmet `le` à
  `payer_par` par l'appel `:886-894`, relu ; M6 décrite aux gardes en mots entiers). Les citations de l'historique (entrées
  P3, P4) restent telles quelles.
  - **Synthèse de la validation** (`CLAUDE.md` § *Review Iteration Rule*) : cinq passes, **3 → 4 → 2 → 1 → 0 MEDIUM**
    au-dessus de LOW (aucun CRITICAL ni HIGH). Modèles : P1 Sonnet, P2 Opus, P3 Sonnet (complètes), P4 Opus (ciblée), P5 Sonnet
    (ciblée, sur une remédiation qui ne touche que la fiche). Aucun finding réfuté sur la série, aucun reclassement.
  - **Dérogation et découpage** : la 35-5a0 est le *story-zero* né du découpage de la 35-5a (**C-e350-28**). Elle reste **une
    story en quatre commits** — `kesh-core`, plomberie `kesh-api`, noyau D2 avec tous ses appelants, garde et docs —, chacun
    compilable ; la chaîne contre-passation → bloqueurs → vues est insécable par la règle D2 ; `lock_books` / `unlock_books` et
    les commits (1), (2), (4) seraient séparables et restent pour un **motif de coût, assumé par l'orchestrateur** (annotations
    de C-e350-28, validations P2 et P3). Risque accepté écrit à la *Dérogation*.
  - **Recyclage déclaré** au Project Lead à chaque passe : les MEDIUM de P2 (V2-1 à V2-4), de P3 (V3-1, V3-2) et de P4 (V4-1)
    sont tous nés de la remédiation précédente, aucun de la conception D1-D3 ; la sévérité a décru de P2 à P5.
  - **Validation terminée** sur une passe qui ne rend aucun finding au-dessus de LOW. Aucune migration. Statut
    `ready-for-dev`.
- 2026-11-13 — **Développement** (Opus 5.5, worktree `kesh-e355a0`, branche `story/35-5a0-horloge-plomberie`, partie de
  `epic-35/base3` et des huit commits de la 35-3 rejoués par cherry-pick, `2b1c0ec8`). Choix C-e355a0-1 à -7. Statut
  `review`.
  - **Étape 0 — la branche telle quelle** (`2b1c0ec8`, avant tout changement, sur le clone de gate) : `cargo build
    --workspace --all-targets` vert ; les tests de la 35-3 — `nextest --profile ci` sur `designated_account_type_e2e`,
    `textes_coherents`, `invoices_validate_vat`, `supplier_invoices_repository` et les `lib` de `kesh-db` et `kesh-api` —
    **1293 passés, 5 ignorés, 0 échec** (`kesh-gate-logs/e355a0/etape0-nextest.log`). **Recompte** au même commit : la
    commande de la fiche `git grep -cE 'supplier_invoices::(validate|unvalidate|pay|cancel|cancel_settlement)\s*\(' --
    'crates/*/tests/*'` rend **158 appels dans 23 fichiers** (égal) ; `Lecture::new(` : **43 constructions dans 10
    fichiers** de tests, plus six chaînes des gardes (`mcp_gardes_lexicales.rs` 2, `services_motif.rs` 4) — égal ;
    littéraux d'`AppState` (champ `rate_limiter_send_email:`) : **9** — égal. Le « 50 fichiers dont 36 de tests, 68 » de
    la fiche ne se recompte pas à l'identique : l'alternative `$P` n'y est pas écrite, et la mienne (noms nus compris :
    `cancel_blocker(`, `cancel_settlement(`, `cancel_in_tx(`, qui prennent des homonymes) rend 53, 37 et 71 ; le
    compilateur a donné la liste close (ci-dessous).
  - **Commits** : (1) `48b9e627` `kesh-core/src/horloge.rs` (D1) ; (2) `05666dbf` plomberie `kesh-api` (D3, AC3, AC4),
    `kesh-db` intact ; (3) `5426cb8b` noyau D2 et tous ses appelants, commentaire `companies.rs:986-989` réécrit, helpers
    `aujourd_hui_test()`, AC2 ; (4) `79ec67b4` garde (D5) et `docs/testing.md` (D6) ; (5) `0deb9abc` correctif de
    `configuration_transmise` (C-e355a0-7). Chacun compile (`cargo build --workspace --all-targets`, `clippy -D
    warnings`, `fmt --check` verts à chaque commit).
  - **La fermeture, par le compilateur** : production — les **18** appels de `kesh-api` que rend `cargo build -p
    kesh-api` après le changement des signatures (exactement la liste de la fiche : `routes/companies.rs:505`, `:518`,
    `imported_supplier_invoices.rs:230`, `:646`, `:723`, `invoices.rs:1361`, `journal_entries.rs:456`,
    `reconciliation.rs:5799`, `:5938`, `:6587`, `supplier_invoices.rs:829`, `:871`, `:904`, `:931`, `:960`, `:990`,
    `services/factures.rs:552`, `services/factures_fournisseur.rs:206`), plus les cinq sites de jour de `kesh-api`
    (`bank_accounts.rs:1007`, `:1252`, `opening_balances.rs:244`, `:604`, `onboarding.rs:1043` par `finalize_inner`),
    édités à l'outil Edit ; la date est lue **une fois**, avant le rejeu `retry_on_deadlock` et l'attente du verrou ;
    tests — **336 E0061**, **311 sites** distincts dans **33 fichiers**, convertis par script depuis les diagnostics
    (C-e355a0-3 : écart déclaré à la consigne « Edit seulement », chiffré), puis `cargo fmt` (C-e355a0-2). Aucun appel ne
    reçoit une date de pièce : le script n'insère que `aujourd_hui_test()` ; les deux appels internes des `mod tests` de
    `journal_entries.rs` reçoivent la variable de jour de leur montage (`jour`, `today`, tirées de `Utc::now()`, non une
    date de pièce).
  - **Helpers** : `crates/kesh-api/tests/common/jour.rs` (`pub mod jour;` dans `common/mod.rs`) ; `crates/kesh-db/tests/
    support/jour.rs`, `crates/kesh-report/tests/support/jour.rs` (neuf). Inclusion `#[path = "support/jour.rs"] mod jour;`
    dans **18** binaires `kesh-db` et **3** `kesh-report` (`aged_receivables`, `open_items_invariant`,
    `vat_report_write_off`) ; `mod common { pub mod jour; }` dans les trois binaires `kesh-api` sans `mod common;`
    (`homepage_summary_e2e`, `supplier_invoice_draft_e2e`, `supplier_settlement_cancel_e2e`) ; inclusion **imbriquée**
    `#[path = "jour.rs"]` dans `support/lettering_documents.rs` et `support/fournisseur_rapproche.rs` — **constatée au
    compilateur** (résolue depuis `support/`, y compris pour les binaires de `kesh-report` et `kesh-api` qui incluent ces
    fichiers par `../../kesh-db/tests/support/`) ; `clippy::duplicate_mod` rougit sur le doublon, d'où C-e355a0-4.
  - **AC1, contrôle en deux temps** (`<base>` = `2b1c0ec8`, parent du premier commit). (a) Fichiers modifiés, lignes `-`
    filtrées sur `assert|from_ymd_opt|expect_err|StatusCode::` : **34 lignes, toutes `.expect_err("…")`**, chacune avec
    sa ligne `+` identique (reflux de rustfmt : l'appel, allongé d'un argument, passe sur plusieurs lignes) — appariement
    vérifié ligne à ligne (`moins.txt` / `plus.txt`, 34 retirées, 34 retrouvées ; côté `+`, 3 lignes de plus, les
    assertions neuves de la garde `horloge_outils` de `mcp_gardes_lexicales.rs`) *(annoté le 2026-11-13, revue P1, A4 :
    faux à sa date — il y en a **5** : les trois de `mcp_gardes_lexicales.rs` et deux de `configuration_transmise.rs`,
    `assert!(` et `assert!(VARIABLES_DE_TEST…)`, assertions neuves du commit (5))*. Aucun attendu de comportement ne change.
    Les attendus lexicaux changés sont ceux que D3 nomme : `mcp_gardes_lexicales.rs` (`LIB_NOMME_MCP` et le site de
    `LectureOutils::nouvelle`, qui devient `let lecture_outils = mcp::LectureOutils::nouvelle(lecture_pool);`),
    `services_motif.rs` (`FABRIQUES_V2` : `Lecture::new(self.pool.clone(), self.horloge.clone())` et la ligne
    `lecture_pool`), et la garde neuve de `horloge_outils` (liste voisine). (b) Fichiers neufs : les huit de la *File
    List*, `assert` neufs par construction.
  - **Risque accepté, les deux relevés** (sur `git diff -U0 --diff-filter=M 2b1c0ec8 HEAD`) : appels sur une ligne,
    dernier argument — `aujourd_hui` ×23 (production), `crate::jour::aujourd_hui_test()` ×36, `aujourd_hui()` ×1
    (`lecture.aujourd_hui()`), `jour` ×1 et `today` ×1 (les deux montages de `journal_entries.rs`, ci-dessus) ; lignes
    d'argument seules, dédoublonnées — en tête `crate::jour::aujourd_hui_test(),` ×235,
    `crate::common::jour::aujourd_hui_test(),` ×33, `aujourd_hui,` ×29, `jour::aujourd_hui_test(),` ×4 ; les autres
    jetons (`&pool,`, `seeded.company_id,`, `le,` ×5…) sont des arguments **préexistants** que rustfmt a remis chacun sur
    sa ligne — `le,` est l'argument `payment_date` de `pay` dans `payer_par`, inchangé, suivi de
    `jour::aujourd_hui_test(),`. Aucun jeton de date de pièce passé comme jour.
  - **D5, inventaire** : `git grep -nE 'HorlogeFixe\(\s*(chrono::)?Utc::now' -- 'crates/*/src/*' 'crates/*/tests/*'` →
    `middleware/auth.rs:302` (`test_state()`) seul en code, plus deux textes de la garde (commentaire, sonde).
  - **La garde** (`date_du_jour_garde.rs`) : comptes **écrits depuis son exécution** à listes vides
    (`garde-comptes-bruts.log`) — inventaire : **52 fichiers** à la liste close (production : `kesh-core/src/horloge.rs`
    1, permanent ; `admin.rs` 2, site 9 et l'horodatage `:722` ; les 19 sites de la 35-5b ; `journal_entries.rs` 50 et
    `invoices.rs` 10 de `kesh-db`, modules de tests compris ; les 38 fichiers de `crates/*/tests` du verdict de la 35-5a,
    à leurs comptes) ; les neuf sites de la 35-5a en sont sortis. Durcissement : littéral 2 (`kesh-core/src/horloge.rs`) ;
    `NOM_VARIABLE_INSTANT` 3 / 2 / 2 / 2 (`kesh-core`, `kesh-api/src/horloge.rs`, les deux `support/jour.rs`) et 2 dans
    `configuration_transmise.rs` (C-e355a0-7) ; `instant_de_test` 6 / 2 / 2 / 2 ; `horloge_depuis` 4 ; `new_for_tests` 1
    (`lib.rs`, sa définition) ; `horloge_de_test` 1 + 1 ; `HorlogeSysteme` construit 1 (`main.rs`) + 1
    (`horloge_depuis`) ; `HorlogeFixe` 5 / 2 / 1 (`auth.rs`) ; aucune mémoire dans les cinq fichiers. **Sondes (AC5)**,
    tests du fichier, toutes conformes : forme ajoutée et forme coupée sur deux lignes → rouges ; `CURDATE()` dans un
    fichier de tests → rouge ; compte + 1 d'un fichier exempté → rouge ; exemption portée à + 1 → rouge ; ligne `//` →
    verte ; littéral dans un fichier de plus, `HorlogeSysteme` construit ailleurs, `OnceLock` dans un helper de jour,
    `new_for_tests(` et `new_for_tests` nu dans `src`, `NOM_VARIABLE_INSTANT` / `instant_de_test(` / `horloge_depuis(`
    / `HorlogeFixe(` / `use … horloge_de_test as h;` dans `main.rs` → rouges ; `// NOM_VARIABLE_INSTANT` et l'import
    groupé `use kesh_api::{ horloge::HorlogeSysteme, };` dans `main.rs` → verts ; et la garde lit un autre fichier du
    même dossier (`horloge_etat.rs`) et exclut le sien. **Angles morts** écrits dans l'en-tête (jour tiré d'une variable
    d'instant ; autre nom de variable lu à la main en production).
  - **Tests neufs** (de `2b1c0ec8` à `HEAD`) : **19** — `kesh-core/src/horloge.rs` 6 (AC4), `kesh-api/src/horloge.rs` 2
    (AC4 : `horloge_depuis`, figée et non figée par `assert_ne!` autour de 10 ms), `horloge_etat.rs` 1 (AC3),
    `horloge_routes_e2e.rs` 2 (AC2), `date_du_jour_garde.rs` 7 (AC5), `configuration_transmise.rs` 1.
  - **Gates réellement exécutés** :
    - commit (2), clone de gate : `test-fast.sh --ci` **5333 exécutés, 5331 passés, 2 échecs, 13 ignorés** — les deux
      échecs, `configuration_transmise::{fantomes, lectures}`, sont ceux que corrige le commit (5) ; `gate-ci.sh` non
      lancé à ce commit (retiré de la file au profit du gate du commit (5)) ;
    - commits (3) et (4) : gate ciblé (`fmt`, `clippy --workspace --all-targets -D warnings`, garde exécutée,
      `cargo build --all-targets`) ; **pas de gate complet propre à ces commits** — écart à l'exception `kesh-db`,
      déclaré : leur contenu est jugé par le gate complet du commit (5), qui ne diffère du (4) que par
      `horloge_de_test` et deux fichiers de tests ;
    - **commit (5), dernier commit de code — gate complet** (base remise à neuf avant chaque étape) :
      `test-fast.sh --ci` **5343 exécutés, 5343 passés, 13 ignorés** (`c5-test-fast.log`) ; `scripts/gate-ci.sh`
      **5350 passés, 0 échoué, 17 ignorés** (`c5-gate-ci.log`, journal `c5-gate-ci.journal`) ; frontend : `check` 0
      erreur (29 avertissements, préexistants), `lint-i18n-ownership` PASS, Vitest **3188 passés**, `build` vert ;
      **E2E complet** (recette entière, `KESH_INBOX_DIR` et `KESH_DOCUMENTS_DIR` des deux côtés, port 3408, base
      `kesh_e2e_e355a0`) : **403 passés, 7 échecs, 18 ignorés** — les sept de la KF-029 (#97), nommés un par un contre
      `docs/testing.md` : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`,
      `onboarding.spec.ts:57`, `:77`, `:150` ; aucun autre.
  - **Mutations** (`kesh-gate-logs/e355a0/mut/mutations-e355a0.py`, sur le worktree au commit (5), une prise de la
    voie base, motif vérifié unique, restauration `git checkout --` + `touch` ; `mutations.log`, `M*.log`) : **8 / 8
    tuées**, chacune par le motif attendu — **M1a** (`bank_accounts.rs:1007` garde `Utc::now()`) : garde,
    `bank_accounts.rs : 2 occurrence(s), 1 écrite(s)` ; **M1b** (la contre-passation relit `Utc::now()` dans `kesh-db`) :
    garde (`journal_entries.rs : 51, 50`) **et** AC2 (les deux tests de `horloge_routes_e2e`) ; **M1c** (`lock_books`
    relit `Utc::now()`) : garde (`companies.rs : 1, 0`) et AC2 ; **M2** (câblage du MCP sur `HorlogeSysteme`) : garde
    `[HorlogeSysteme construit] lib.rs : 1, 0` ; **M3** (`OnceLock` dans `horloge_de_test`) : `[mémoire]` ; **M4** (le
    bloqueur relit l'horloge, deux lectures par requête) : garde (`settlement_cancellation.rs : 1, 0`) ; **M5**
    (`Some("")` panique) : `instant_de_test_absent_ou_vide_rend_none` ; **M6** (`main.rs` lit la variable par la
    constante, `HorlogeSysteme` en repli) : trois gardes, `[NOM_VARIABLE_INSTANT]`, `[instant_de_test]`,
    `[HorlogeFixe]` sur `main.rs` — le compte de `HorlogeSysteme` construit reste 1 dans `main.rs`, comme la fiche
    l'annonce. Preuve du rouge : `kesh-gate-logs/e355a0/rouges.log`.
  - **Recontrôles** : `git log 2b1c0ec8..HEAD --format=%B | /usr/bin/grep -niE
    '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` vide ; aucune migration ; aucun écrit sous `~/kesh/`.
  - **Écarts déclarés** : la conversion des tests par script (C-e355a0-3, 311 sites) et les déclarations du helper
    (26 fichiers) — consigne « Edit seulement » ; un cinquième commit (C-e355a0-7) ; `horloge_de_test` lit par
    `env_nonempty` et non par `std::env::var` (D3) ; `allow(clippy::duplicate_mod)` (C-e355a0-4) ; gates des commits
    (3) et (4) non complets (ci-dessus).
- 2026-11-13 — **Revue de code P1** (Opus, trois lentilles B, E, A ; `kesh-gate-logs/e355a0-review-p1-{B,E,A}.md` :
  **3 MEDIUM, 14 LOW**, aucun CRITICAL ni HIGH ; findings vérifiés au code par l'orchestrateur) **remédiée** par l'agent de
  développement (Opus 5.5, worktree `kesh-e355a0`), commit de code `c8cda87e`. Prémisses relues au code.
  - *(Annoté le 2026-11-13, revue P2, C1 : l'inventaire ci-dessous **n'était pas clos** — il ne relevait que les jours
    portés par une variable nommée `today` / `aujourd_hui` / `jour`, et ratait les jours tirés en ligne ou par une fonction
    (`fn aujourdhui()`, `jours_avant`, `facture_lettree`…). L'exclusion « `:2976` … dans le helper `figer` » était fausse
    pour `supplier_cancels_are_refused_under_a_frozen_lettering`, qui porte des sites convertis. Inventaire repris depuis
    le symptôme : entrée P2 ci-après.)*
  - **E1 / A2 (MEDIUM) — D4 aux montages qui tiennent leur jour.** Inventaire **clos** d'abord
    (`kesh-gate-logs/e355a0/inventaire-d4-e355a0.py`, lecture seule ; `r1-inventaire-d4.txt`) : pour les **311** sites
    convertis de `c3-erreurs.json`, la fonction englobante relue à `05666dbf` déclare-t-elle `let today | aujourd_hui |
    jour` ? **13 sites** ; complété à la main par les fonctions de jour des fichiers convertis (`fn today()`,
    `fn jour()`) et les paramètres `today: NaiveDate`. Verdicts :
    - `kesh-db/tests/invoice_settlement.rs` — `monter` (`today` `:1205` → `lock_books` `:1254`, lignes à `05666dbf`) et
      `une_cloture_d_exercice_attend_l_annulation_d_un_virement` (`:2340` → `cancel_settlement` `:2392`) : **convertis**,
      `let today = crate::jour::aujourd_hui_test();`, l'appel reçoit `today` ;
    - `kesh-db/tests/supplier_invoices_repository.rs` — `monter_par_virement` (`:2973` → `pay` `:3017`) : **converti** ;
    - `kesh-db/tests/opening_complement_repository.rs` — `entrelacement_5_contre_passation_en_vol` (`:880` → `reverse`
      `:924`) : **converti** ; l'année de l'exercice suit `today` (`let y = today.year()`), si bien que l'exercice couvre
      toujours le jour, variable posée ou non ;
    - `kesh-db/tests/lettering_documents_backfill.rs` — `rattrapage`, 9 sites (`let jour = jours_avant;`) : **hors
      champ**, `jour` y est une fonction (alias de `jours_avant`), non un jour ;
    - `kesh-api/tests/homepage_summary_e2e.rs` — 8 sites dans quatre tests dont le montage tient `fn today()` (heure
      réelle) et l'asserte (`assert_eq!(r1.entry.entry_date, today(), …)`) : **convertis à `today()`** ; le montage reste
      à l'heure réelle, comme l'accueil qu'il éprouve (site 21, 35-5b) ; la déclaration `mod common { pub mod jour; }`,
      devenue inutile, est retirée ;
    - `kesh-db/tests/supplier_document_attachment.rs` (`fn jour()` = `2026-06-15`, date de pièce fixe) et les fichiers aux
      `fn jour(y, m, d)` (constructeurs de dates) : **hors champ**, ce ne sont pas des jours du montage ;
    - `supplier_invoices_repository.rs` `:1646` (`monter`), `:1927` (`monter_achat`), `:2976` : leurs fonctions n'ont
      **aucun** site converti ; le site est dans le helper `figer`, qui ne tient pas de jour — **hors règle**, laissés à la
      35-5c (transmis).
    **La garde change de compte** (recompté à son exécution) : `invoice_settlement.rs` 4 → **2**,
    `supplier_invoices_repository.rs` 4 → **3**, `opening_complement_repository.rs` 2 → **0** (entrée retirée).
  - **A1 (MEDIUM) et A6 (LOW)** : C-e355a0-5 **annoté** (daté, l'entrée gardée) — ce qui a tourné commit par commit ; au
    (2), `test-fast` seul et `gate-ci.sh` jamais lancé (`c2-gate-ci.log` vide) ; aux (3) et (4), aucun gate complet ;
    l'exception `kesh-db` n'y a pas été suivie, et le contenu `kesh-db` du (3) au (5) est identique (`git diff --stat
    5426cb8b 0deb9abc -- crates/kesh-db` vide). *Tâches* annotées de même. C-e355a0-3 annoté pour E1/A2.
  - **LOW traités** : **A3** — section *Transmis à la 35-5a* au Dev Agent Record (le compte 2 de `NOM_VARIABLE_INSTANT`
    dans `configuration_transmise.rs`, la lecture par `env_nonempty`, E3, les montages laissés, la `Lecture` du MCP tenue
    lexicalement) ; **A4** — l'entrée du développement annotée : côté `+`, **5** lignes de plus, non 3 (recompté à
    `0deb9abc` et à `c8cda87e` : 34 lignes `-`, toutes `.expect_err`, et 39 lignes `+` dont 34 `.expect_err` identiques) ;
    **A5** — M2 rejouée sur `mcp_gardes_lexicales.rs` (ci-dessous) ; **B1** — `kesh-core/src/horloge.rs`, `lib.rs`,
    `journal_entries.rs` : les commentaires disent « des sites convertis » et renvoient à l'inventaire de la garde ;
    **B3** — les formes non relevées (`Utc::now().date()`, `NaiveDate::from(… naive_utc())`, `CAST(NOW() AS DATE)`,
    `SYSDATE()`, `DATE_FORMAT(NOW(), …)`, `… < NOW()`) écrites aux angles morts de l'en-tête de la garde ; **B4** — AC2
    asserte aussi le refus par sa garde (`INVALID_INPUT` et le message de `booksLockBoundNotPast`, « antérieure à
    aujourd'hui » — le code `booksLockBoundNotPast` n'est pas exposé tel quel au corps HTTP, qui porte `INVALID_INPUT`) ;
    **B5 / E2** — `instant_de_test` rogne la valeur : blanche, elle vaut absente, côté `kesh-api` comme dans les helpers
    de `kesh-db` et `kesh-report` (docs des helpers alignées), test `Some("   ")` ; **E4** — doc de `common/jour.rs` : vrai
    tant que le test ne remplace pas `state.horloge` ; **E5** — test `instant_de_test(Some("2026-01-01T00:30:00+01:00"))`
    (ramené au `2025-12-31T23:30Z`, jour du 31) et valeur entourée de blancs ; **E6** — la ligne de `docs/testing.md` dit
    les sites convertis, « vide ou blanche », et que la suite ne suit pas encore toute la variable (35-5c) ; **E3** —
    déclaré comme **écart voulu** et transmis à la 35-5a (section ci-dessus). Garde recomptée aussi pour `kesh-core` :
    `instant_de_test` 6 → **9**, `HorlogeFixe` 5 → **6** (tests neufs).
  - **Tracé, non traité** : **B2 → #1105** (« lue une fois, avant le rejeu » sans filet).
  - **Fichiers de production touchés** : `crates/kesh-core/src/horloge.rs` (`instant_de_test` rogne ; doc),
    `crates/kesh-api/src/lib.rs` (doc), `crates/kesh-db/src/repositories/journal_entries.rs` (doc). Édition à l'outil
    Edit, sans script (l'inventaire est un script **en lecture seule**, déclaré).
  - **Gates de `c8cda87e`, dernier commit de code** (dans le worktree, sans clone, aucune édition de code pendant le run ;
    base remise à neuf ; `kesh-gate-logs/e355a0/suite3.log`, `r1-*.log`) : `fmt --check` et `clippy --workspace
    --all-targets -D warnings` verts ; `test-fast.sh --ci` **5344 exécutés, 5344 passés, 13 ignorés** (+1 : le test neuf
    de `kesh-core`) ; `scripts/gate-ci.sh kesh_ci_e355a0` **5351 passés, 0 échoué, 17 ignorés** ; frontend : **`npm run
    build` seul** (vert, il sert l'E2E) — `check`, `lint-i18n-ownership` et Vitest **non rejoués**, la remédiation ne
    touchant aucun fichier de `frontend/` ; **E2E complet** (production touchée : `instant_de_test` rogne) : **403 passés,
    7 échecs, 18 ignorés**, les sept de la KF-029 nommés contre `docs/testing.md` (`mode-expert.spec.ts:26`, `:41`,
    `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`), aucun autre. **Mutations rejouées**
    (`mut/r1-mutations.log`) : **M2** TUÉE [101, 101] — garde de la date **et** `mcp_gardes_lexicales.rs`
    (`outils_et_politique_en_leur_lieu`, l'assertion `horloge_outils`, `:592`) ; **M5** TUÉE [101]. `rouges.log` annoté
    (A5). Contrôle des mots-clés de fermeture (`2b1c0ec8..HEAD`) : vide. Statut : `review`.
  - **Trend de la revue** : P1 : **3 MEDIUM, 14 LOW** (B 5 LOW ; E 1 MEDIUM, 5 LOW ; A 2 MEDIUM, 4 LOW) ; tous remédiés
    ou déclarés, sauf B2 tracé en **#1105**. Passe suivante à lancer sur cette remédiation.
- 2026-11-13 — **Revue de code P2, passe ciblée** (Sonnet, sur `c8cda87e` ; `kesh-gate-logs/e355a0-review-p2-ciblee.md` :
  **1 MEDIUM, 1 LOW**) **remédiée** par l'agent de développement, commit de code `1a9af93b`. **Signal de recyclage**
  (déclaré au Project Lead par l'orchestrateur) : C1 est le défaut de P1 E1/A2 revenu sous une autre forme — un inventaire
  parti d'une liste de noms, non du symptôme.
  - **C1 (MEDIUM) — inventaire D4, cette fois parti du symptôme.** Détecteur
    `kesh-gate-logs/e355a0/inventaire-d4-symptome-e355a0.py` (lecture seule) : dans le code de test (`crates/*/tests/**`,
    et dans `crates/*/src/**` la partie qui suit `#[cfg(test)]`), toute fonction dont le corps contient un appel
    `aujourd_hui_test(` **et** une lecture de l'heure réelle — `Utc::now`, `Local::now`, `SystemTime::now`, ou l'appel d'un
    helper qui en dérive, transitivement (point fixe sur les fonctions du fichier et des dossiers partagés `support/`,
    `common/` ; appels reconnus à la syntaxe `nom(` / `chemin::nom(` / `= nom;`, jamais `.nom(`). Commentaires et chaînes
    neutralisés avant le découpage des corps. **Sonde positive** : la première exécution
    (`r2-inventaire-symptome.txt`) rend `open_items.rs:733 fn view_fields_origins_and_periods jour_test=[746, …]
    reel=[…, '764:aujourdhui', …]` et `supplier_invoices_repository.rs:2976 fn
    supplier_cancels_are_refused_under_a_frozen_lettering … reel=['2979:horloge']` — le détecteur voit les deux cas de la
    revue. **47 fonctions relevées.**
  - **Verdicts** — sur les 47 : les **41** de `kesh-db` et `kesh-report` sont **convertis** ; les **6** de `kesh-api`
    gardent une lecture d'un **instant** (hors champ, ci-dessous), et trois d'entre eux avaient aussi un jour, converti
    (`admin_full_import_e2e.rs:2008`, `reconciliation_e2e.rs:4285`, `jours_avant` pour `letterings_e2e.rs`). Conversion de
    la source du jour — un seul jour, tiré de l'horloge de test, que reçoivent l'appel, les assertions et les helpers
    relatifs du montage :
    - `kesh-db/tests/support/lettering_documents.rs:48` `jours_avant` → `jour::aujourd_hui_test() - n` : **27** fonctions
      de `kesh-db`/`kesh-report` (`lettering_documents.rs` 17, `seed_lettering_documents`,
      `lettering_documents_backfill.rs` 2, `letterings.rs` `group_detail_of_reversal_pairs`, `open_items.rs` 5 (avec
      `aujourdhui`), `kesh-report/tests/open_items_invariant.rs`), et `letterings_e2e.rs`
      `forecast_equals_delete_over_http` ;
    - `open_items.rs:49` `fn aujourdhui()` → `crate::jour::aujourd_hui_test()` (assertion `:764` comprise) ;
    - `filet_bilan_clos.rs` `societe_du_jour` (`:71`, l'année de l'exercice) et `corps(&s, …)` `:567`, `:683` : 2 tests ;
    - `invoice_settlement.rs` `facture_lettree` (`:1965`) et `later_closed_year_group_is_refused` (`:2234`) : 4 tests ;
    - `supplier_invoices_repository.rs` `monter` (`:1646`), `monter_achat` (`:1927`) et
      `supplier_cancels_are_refused_under_a_frozen_lettering` (`:2979`) : 6 tests ;
    - `letterings.rs:1463` (`lettering_invariants`, année de l'exercice) et `:2392` (borne posée au jour) ;
    - `reconciliation_cancel.rs:469` : l'assertion de la date de la contre-passation juge le jour reçu ;
    - `kesh-report/tests/open_items_invariant.rs:235` ;
    - `kesh-api/tests/admin_full_import_e2e.rs:2008` (assertion « borne passée ») ;
    - `kesh-api/tests/reconciliation_e2e.rs:4285` `ensure_fiscal_year_today` (l'exercice couvre le jour du serveur de
      test) : `unreconcile_of_a_closed_lettering_is_refused_in_its_family`.
    **6 hors champ**, ligne lue citée — l'heure lue y est un **instant**, non un jour, ou un nom en collision :
    `admin_full_import_e2e.rs` ×3 (`seed_business` → `seed_admin` → `seed_role` → `forge_jwt`, `:123` `let now =
    Utc::now().timestamp();`, jeton) ; `letterings_e2e.rs` `forecast_equals_delete_over_http` pour `monde_pieces`
    (`:790` `let now = Utc::now().timestamp();`, jeton) et `ecriture_manuelle` (collision de nom : `support/
    lettering_documents.rs:1047` `let ligne = |i: usize| je.lines[i].id;`, une fermeture locale, non la `fn ligne` de
    `letterings_e2e.rs:185`) ; `reconciliation_e2e.rs` ×2 (`setup_company` / `setup_301b` → `forge_jwt`, `:131` `let now =
    Utc::now().timestamp();` ; `reconciled_invoice` → `unique_hash`, `:550` `timestamp_nanos_opt`, une empreinte). Après
    conversion, le détecteur ne rend **que** ces 6 (`r2-inventaire-symptome-apres.txt`).
    Les tests qui lisent l'heure réelle **sans** porter de site converti restent à la 35-5c (liste close de la garde ; ex.
    `admin_full_import_e2e.rs:2177`, `:2235`, `:2283`).
  - **La garde change de comptes** (recompté à son exécution) : `admin_full_import_e2e.rs` 4 → **3** ; **neuf entrées
    retirées** (compte 0) — `reconciliation_e2e.rs`, `filet_bilan_clos.rs`, `invoice_settlement.rs`, `letterings.rs`,
    `open_items.rs`, `reconciliation_cancel.rs`, `supplier_invoices_repository.rs`, `support/lettering_documents.rs`,
    `kesh-report/tests/open_items_invariant.rs`. L'entrée P1 qui disait l'inventaire clos est **annotée** (datée), non
    réécrite ; section « Transmis à la 35-5a » annotée.
  - **C2 (LOW) — non traité, pas court** : dans `homepage_summary_e2e.rs`, `today()` est relu non seulement par les tests
    mais par le montage (`year_start()`, `setup`), si bien qu'une lecture unique par test ne suffit pas : il faudrait passer
    le jour au montage. **Issue à ouvrir** par l'orchestrateur (avec la 35-5b, site 21).
  - **Écart déclaré** : en arrêtant la première version, trop lente, du détecteur, un `pkill -f … -n` a été lancé — contraire
    aux interdits ; il n'a tué que le shell de la commande elle-même (le plus récent processus au motif), et le script a été
    arrêté ensuite par son PID (978982). Aucun autre processus touché.
  - **Fichiers touchés** : tests seulement (`crates/kesh-api/tests/{admin_full_import_e2e,reconciliation_e2e,
    date_du_jour_garde}.rs`, `crates/kesh-db/tests/{filet_bilan_clos,invoice_settlement,letterings,open_items,
    reconciliation_cancel,supplier_invoices_repository,support/lettering_documents}.rs`,
    `crates/kesh-report/tests/open_items_invariant.rs`) ; **aucun fichier de production ni du frontend** — E2E et frontend
    non rejoués.
  - **Gates de `1a9af93b`, dernier commit de code** (worktree, base remise à neuf ; `kesh-gate-logs/e355a0/suite4.log`,
    `r2-*.log`) : `clippy -D warnings` vert ; `test-fast.sh --ci` (fmt et clippy du workspace compris) **5344 exécutés,
    5344 passés, 13 ignorés** ; `scripts/gate-ci.sh kesh_ci_e355a0` **5351 passés, 0 échoué, 17 ignorés** ; garde
    `date_du_jour_garde` verte (7/7, sondes comprises). **E2E et frontend non rejoués** : la remédiation ne touche que des
    fichiers de tests, aucun de production ni du frontend. Contrôle des mots-clés de fermeture : vide. Statut : `review`.
  - **Trend de la revue** : P1 : 3 MEDIUM, 14 LOW ; P2 ciblée : **1 MEDIUM, 1 LOW** (C1 remédié, C2 non traité, issue
    demandée) ; #1105 (B2) tracée. Passe suivante à lancer sur cette remédiation.
