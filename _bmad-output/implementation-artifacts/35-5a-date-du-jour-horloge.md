# Story 35-5a : la date du jour est celle de Zurich pour ce qui écrit dans les livres ou les borne — solde d'ouverture, contre-passation, annulation de règlement, verrou de période, premier exercice de l'assistant (#1022, #907, #929)

Status: done

*Fiche rédigée le 2026-11-12 par l'agent de kickoff de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, par le workflow
`bmad-create-story` ; remédiée après les validations P1 à P3 le même jour (worktree `kesh-e355`, Change Log). Née du
**découpage préventif** de la 35-5 (C-e350-14), **découpée à nouveau** à la validation P3 (C-e350-28, décision de
l'orchestrateur) : la **35-5a0** pose la plomberie — `Horloge`, `HorlogeFixe`, `AppState.horloge`, `Lecture` à horloge
(C-e350-21), la date passée à `kesh-db` et tous ses appelants, la garde de l'inventaire, `KESH_TEST_INSTANT` — **sans rien
changer** (le jour reste UTC) ; la **35-5a** part de la 35-5a0 fusionnée et **bascule le jour sur Zurich** : `jour_suisse`,
les critères de nuit des neuf sites qui écrivent dans les livres ou les bornent, l'écran du verrou, les tests qui atteignent
ces sites, les preuves de nuit. La **35-5b** convertit les sites restants de la production, la **35-5c** les tests restants
(C-e350-22). Release **0.17.7**. **Aucune migration** (C-e350-16). Boucle complète : règle comptable, `kesh-db` touché (gate
complet même en cours de boucle). Code lu sur **`release/0.17.6`** (`4e1c768a` ; vérifié identique à `8424e36e` pour
`crates/` et `frontend/`). Registres : `e35-choix-autonomes-e350.md` (C-e350-8, -14, -16, -21 à -26, -28, -29) ;
`e35-choix-autonomes-e355a.md` pour la story, à créer au premier choix.*

**Ce qui est à la 35-5a0 et non ici** : D2, D3, la pose de la garde (D4) et des helpers (D5, `aujourd_hui_test()`), les
critères « `kesh-db` sans horloge » et « l'horloge atteint les services et le MCP » (anciens AC6, AC7), la fermeture des
appelants (§ *Ce qui se lit* ci-dessous, second tableau). Ces passages restent écrits ici pour l'inventaire ; **la fiche
35-5a0 fait foi** pour ce qu'elle porte.

## Story

En tant que titulaire qui travaille parfois tard, je veux qu'un solde d'ouverture, une contre-passation, une annulation
ou la création du premier exercice faits entre minuit et 1 h (2 h en été) soient datés **du jour suisse**, et que
« verrouiller jusqu'à hier » soit accepté à cette heure-là, afin qu'aucune écriture ne tombe la veille, voire dans
l'exercice précédent la nuit du 1er janvier.

## Ce qui se lit au code (`release/0.17.6`)

**L'inventaire complet** — `git grep -nE 'Utc::now\(\)\.(date_naive|naive_utc\(\)\.date)\(\)|UTC_DATE|CURDATE|Local::now' -- 'crates/*/src/*'`
(⚠️ le pathspec `'crates/*/src'` ne désigne rien et rend un inventaire vide sans le dire : `'crates/*/src/*'`), recompté
fichier par fichier (module `#[cfg(test)]` séparé). Il rend **92 lignes** : 28 sites de production, 56 sites des modules
de tests, **8 commentaires** (ci-dessous). Formes larges (`.date_naive()` et `naive_utc().date()` sur tout récepteur,
`naive_local`, `Utc::today`, `Local::`, `CURRENT_DATE`, `DATE(NOW`, et ces formes coupées sur plusieurs lignes) : rien
de plus dans `src`. **28 sites de production** :

| # | Site | Ce qu'il date ou borne | Story |
|---|---|---|---|
| 1 | `kesh-api/src/routes/bank_accounts.rs:1007` | solde d'ouverture à la création d'un compte (`OpeningRequest.today`) | **35-5a** |
| 2 | `kesh-api/src/routes/bank_accounts.rs:1252` | solde d'ouverture d'un compte existant | **35-5a** |
| 3 | `kesh-api/src/routes/opening_balances.rs:244` | statut du complément des *Soldes de départ* (`complement_status`) | **35-5a** |
| 4 | `kesh-api/src/routes/opening_balances.rs:604` | date du complément des *Soldes de départ* | **35-5a** |
| 5 | `kesh-db/src/repositories/journal_entries.rs:2935` | contre-passation datée du jour (`ReversalDating::Today`) | **35-5a** |
| 6 | `kesh-db/src/repositories/settlement_cancellation.rs:172` | exercice ouvert au jour de l'annulation d'un règlement (rang 5) | **35-5a** |
| 7 | `kesh-db/src/repositories/companies.rs:1013` | `lock_books` : borne refusée si ≥ aujourd'hui (#907) | **35-5a** |
| 8 | `kesh-db/src/repositories/companies.rs:1100` | `unlock_books` : même borne | **35-5a** |
| 9 | `kesh-api/src/routes/admin.rs:99` | nom du fichier de sauvegarde | 35-5b |
| 10 | `kesh-api/src/routes/audit_log.rs:420` | nom du CSV du journal d'audit | 35-5b |
| 11 | `kesh-api/src/routes/bank_accounts.rs:1802` | `closedOn` par défaut à la clôture d'un compte bancaire | 35-5b |
| 12 | `kesh-api/src/routes/exports.rs:58` | nom de l'export de souveraineté | 35-5b |
| 13 | `kesh-api/src/routes/invoice_email.rs:381` | jours de retard d'un rappel | 35-5b |
| 14–16 | `kesh-api/src/routes/invoices.rs:333`, `:425`, `:1459` | facture échue (fiche, liste) ; CSV de l'échéancier | 35-5b |
| 17 | `kesh-api/src/routes/onboarding.rs:1043` | année du premier exercice à l'assistant (`fiscal_years::create_if_absent_in_tx`, ligne de `fiscal_years` tracée) | **35-5a** (C-e350-23) |
| 18 | `kesh-api/src/services/postes_ouverts.rs:74` | `asOf` par défaut des postes ouverts | 35-5b |
| 19–21 | `kesh-api/src/services/rapports.rs:732`, `:780`, `:870` | balance âgée ; fin d'un rapport de projet ; accueil (#929) | 35-5b |
| 22 | `kesh-db/src/repositories/dunning_eligibility.rs:98` | éligibilité aux rappels | 35-5b |
| 23–25 | `kesh-db/src/repositories/invoices.rs:404`, `:970`, `:971` | `UTC_DATE()` en SQL : filtre « échues », synthèse de l'échéancier | 35-5b |
| 26 | `kesh-seed/src/lib.rs:279` | année de la démonstration | 35-5b |
| 27–28 | `kesh-seed/src/pieges.rs:378`, `:461` | données des pièges de démonstration | 35-5b |

Partage : **9 sites en 35-5a** (1–8 et 17), **19 en 35-5b**.

**Les 8 commentaires** que rend la même commande : `kesh-api/src/routes/invoice_email.rs:206`,
`kesh-api/src/services/postes_ouverts.rs:55`, `kesh-db/src/repositories/companies.rs:988`, `dunning_eligibility.rs:9`,
`invoices.rs:378`, `:396`, `:5040` (module de tests), `kesh-report/src/aged_receivables.rs:27`. La garde les masque (D4) ;
ceux qui deviennent faux se réécrivent : `companies.rs:986-989` à la **35-5a0** (faux dès qu'elle passe la date, V4-8), les
autres à la 35-5b. `aged_receivables.rs:27`
(« `as_of` bindé (pas `UTC_DATE()` en dur) ») reste vrai et ne change pas.

Hors production : **56** sites dans les modules de tests de `src` (`journal_entries.rs` 50, à partir de `mod tests` `:3178`,
dont les tests de contre-passation ; `invoices.rs` 6 : `fn today()` `:3498`, son `Utc::now()` à `:3499`, et cinq `CURDATE()`
de montage), **107** sur une ligne dans `crates/*/tests` (38 fichiers ; verdict par fichier ci-dessous), **plus 5** coupés sur
plusieurs lignes (`accounts_e2e.rs:243-245`, `filet_bilan_clos_e2e.rs:179-181`, `journal_entry_reversal_e2e.rs:1397-1399`,
`letterings_e2e.rs:122-123`, `fiscal_years_e2e.rs:1068-1070`), soit **112** ; specs E2E : **34** `toISOString().slice(0, 10)`
dans 26 fichiers, plus **1** coupé sur trois lignes (`invoices.spec.ts:527-528`), soit **35** (le « 36 » d'avant comptait tous
les `toISOString` de `frontend/tests/e2e/`, dans 27 fichiers).

**Les appelants des fonctions de `kesh-db` qui recevront la date** (35-5a), recomptés par
`git grep -nE 'reverse_in_tx\(|reverse_owned_in_tx\(|settlement_entry_cancel_blocker\('` : `kesh-api/src/routes/companies.rs:505`,
`:518`, `kesh-api/src/routes/journal_entries.rs:456`, `kesh-db/src/repositories/invoice_settlements_write.rs` (2),
`journal_entries.rs` (3, dont 2 dans les tests), `reconciliation_cancel.rs` (2), `supplier_invoices.rs` (4) ; plus
`lock_books`, `unlock_books`, `settlement_cancel_blocker` et ses lecteurs (`services/factures.rs:552`,
`supplier_invoices` : `fetch_lines`, `supplier_settlement_cancel_blocker`, lus par `routes/supplier_invoices.rs:272`).
Les signatures de `cancel_in_tx` (`supplier_invoices.rs:2510`), `cancel_settlement_in_tx` (`:2778`) et des bloqueurs (`:2426`,
`:2706`, `:2724` ; `invoice_settlements_write.rs:826`, `:838`, `:912`) changent — `:2462`, `:2750` (bloqueur), `:2600`, `:2871`
(`reverse_owned_in_tx`) et `invoice_settlements_write.rs:878` sont des **appels** —, ainsi que les fonctions de **vue** qui
calculent un bloqueur : `supplier_invoices::{get_settlement_view (:3013), get_settlement_view_in_tx (:3040), commit_with_view
(:3095)}` et `reconciliation_cancel::get_view` (définitions citées par la fiche 35-5a0, qui fait foi ; V4-6). La chaîne de
production remonte à `routes/invoices.rs`, `routes/supplier_invoices.rs`, `routes/imported_supplier_invoices.rs` (`:230`,
`:646`, `:723`), `routes/reconciliation.rs` (`:5799`), `services/factures.rs:552` et `services/factures_fournisseur.rs:206`
(dont le motif du rang 5 est calculé puis jeté par `detail_sans_fichier`, `:227-237` : aucun outil du MCP n'expose un site de
la 35-5a). Le compilateur donne la liste close ; homonymes non concernés : `invoice_reminders::cancel_in_tx`
(`routes/dunning_reminders.rs:360`, `kesh-db/tests/invoice_reminders_repository.rs`).
`reverse_at_next_fiscal_year_start*` ne lit pas le jour.

**Appelants de tests sans site de jour** (V2-4 de la validation P2), relevés par
`git grep -lE 'journal_entries::reverse\(|reverse_in_tx\(|reverse_owned_in_tx\(|settlement_entry_cancel_blocker\(|settlement_cancel_blocker\(|lock_books\(|unlock_books\(|cancel_settlement_in_tx\(|cancel_in_tx\(|get_settlement_view|commit_with_view|supplier_invoice_cancel_blocker|reconciliation_cancel::get_view' -- 'crates/*/tests/*' 'crates/*/tests/**'`,
**hors des 38 fichiers** du verdict : `kesh-api/tests/invoice_echeancier_e2e.rs:2014` (`lock_books`),
`supplier_settlement_cancel_e2e.rs:749` (`lock_books`), `supplier_invoice_response_e2e.rs:434-496` (`commit_with_view`,
`get_settlement_view_in_tx`), `kesh-db/tests/invoice_write_off.rs:649-1558` (`settlement_cancel_blocker`, `lock_books`,
`get_view`), `journal_entries_modification.rs:365`, `:473` (`reverse_in_tx`), `lettering_documents.rs:755-1040` (`lock_books`,
`unlock_books`), `lettering_documents_backfill.rs:289-772` (`unlock_books`, `reverse` ×6), `migrations_upgrade_path.rs:1508`,
`:1521`, `:1640` (`supplier_invoices::cancel_in_tx`, `get_settlement_view`) ; **dans** les 38, `admin_full_import_e2e.rs:1809`
(`journal_entries::reverse`). Gardes lexicales sur ces noms, indifférentes à un argument de plus : `bank_account_closed_inventory.rs:862-870`,
`kesh-db/tests/letterings_lexical.rs:445-547` (à constater). Ces fichiers changent **dans la 35-5a0** (la compilation l'impose) :
chaque appel reçoit la date qu'il tient, ou `aujourd_hui_test()`. **La validation P3 (V3-2) a montré que cette liste n'est
pas close** : la vue des factures fournisseur termine toutes leurs écritures (`validate`, `unvalidate`, `pay`, `cancel`,
`cancel_settlement` par `commit_with_view`), et `reconciliation_cancel::*`, `invoice_settlements_write::cancel_settlement`
s'y ajoutent — 158 appels dans 23 fichiers pour les seules écritures fournisseur. **La fermeture complète est dans la fiche
35-5a0**, qui fait foi. Le verdict de nuit de ces fichiers se prouve ici par l'exécution d'AC9 (tous les binaires).

**La poignée `Lecture`** (`crates/kesh-api/src/services/poignees.rs:33-48`) : `pub struct Lecture(MySqlPool)`, `Lecture::new(pool)`.
Construite par `AppState::lecture()` (`lib.rs:159-161`) et, pour les outils du MCP, au câblage
(`lib.rs:1403` : `mcp::LectureOutils::nouvelle(services::poignees::Lecture::new(pool_outils.clone()));`) ; l'état du MCP
(`EtatMcp`, `mcp/mod.rs:105-130`) ne voit pas l'`AppState`, et les outils n'atteignent les services que par `ctx.acces()` et
`ctx.lecture()` (`mcp/outils/balance_agee.rs:107`). Gardes lexicales qui figent ces lignes :
`tests/mcp_gardes_lexicales.rs:169` et `:575`, `tests/services_motif.rs:2071` et `:2083` (`FABRIQUES_V2`, AC10 (v-2) : `Lecture::new(`
aux seules lignes du câblage). **43** constructions `Lecture::new(` dans **10** fichiers de tests (`services_rollout.rs` 25,
`services_motif.rs` 7, `cles_mcp_e2e.rs`, `mcp_prompts_e2e.rs`, `onboarding_audit_e2e.rs` 2 chacun, `admin_full_import_e2e.rs`,
`common/lecture_seule.rs`, `mcp_admin_routes_e2e.rs`, `mcp_envois_e2e.rs`, `mcp_journal_e2e.rs` 1 chacun ; recompter).
Garde lexicale de `services/factures.rs:603` : `"settlement_cancel_blocker(&muttx,"` (espaces retirés) — elle tient si la
date est **ajoutée en dernier argument**.

**Le précédent de l'écran** : `frontend/src/lib/shared/utils/local-date.ts` (`todayLocal`, #889, 0.17.4), gardé par
`local-date-guard.test.ts`, qui tient l'inventaire des sites clos. La 35-5a en est le pendant serveur. L'écran du verrou
(`frontend/src/routes/(app)/settings/+page.svelte:563-567`, `maxLockDate`) prend encore la **veille UTC**, entrée d'allowlist
`local-date-guard.test.ts:55-58` motivée par « borne du verrou jugée par le serveur à sa date UTC (C-v0174c-14, #907) » :
fausse dès cette story.

**`AppState`** (`crates/kesh-api/src/lib.rs:61-107`) ; constructeur de test `new_for_tests` (`:109-…`), qui défaute les champs
dans son corps ; mais `AppState` se construit aussi **par littéral** : **9 littéraux dans 7 fichiers** (compte par le champ
`rate_limiter_send_email:`, 9 occurrences ; `git grep -n 'AppState {'` rend 16 lignes, dont la définition, l'`impl`, un
commentaire, deux signatures et trois chaînes de garde) — le `Self {` de `new_for_tests` (`lib.rs`), `main.rs:439`,
`middleware/auth.rs:285`, `invoice_frozen_pdf_e2e.rs:101`, `invoice_send_email_e2e.rs:104`, `:1036`, `:1150`,
`password_recovery_e2e.rs:126`, `setup_admin_e2e.rs:82`. `spawn_app` est défini **par fichier** de test ; `tests/common/` n'en a
pas.

**Les docs qui disent UTC ou « du jour »** : `docs/api-external.md:272` (« sinon du jour **UTC** »), 35-5a ; `:252`
(`closedOn`), `:292` (accueil), `:443` (postes ouverts), 35-5b. **Le manuel utilisateur dit déjà « en temps universel » deux
fois** (validation P4, V4-1 ; `/usr/bin/grep -n 'temps universel' docs/manual/fr/user-manual.tex` → `:995`, `:2458`, lus) : `:995`
(complément des soldes de départ : « \textbf{Le jour est celui du serveur, en temps universel}~: entre minuit et une ou deux
heures du matin, heure suisse, c'est encore la veille --- le 1\textsuperscript{er} janvier, une régularisation peut tomber
dans l'exercice précédent… ») et `:2458` (solde d'ouverture d'un compte bancaire : « … Le jour est celui du serveur, en temps
universel… (#1022, comme pour le mode compléter) ») ; le PDF publié les porte aussi. Ces deux phrases décrivent les sites 1 à 4
et deviennent fausses à la fusion. Partout ailleurs (`:259-261`, `:355`, `:709`, `:722`, `:994`, `:1449-1482`, `:1900`, `:2011`,
`:2085`), il dit « la date du jour » sans fuseau ; il ne dit nulle part « date du poste »
(`/usr/bin/grep -n 'du poste' docs/manual/fr/user-manual.tex` : vide).

## Décisions (C-e350-16, C-e350-21 à -26, -28, -29)

- **D1 — Europe/Zurich, constante unique, et la bascule** : `pub fn jour_suisse(instant: DateTime<Utc>) -> NaiveDate`, pure,
  dans `kesh-core/src/horloge.rs` (le module de la 35-5a0), écrite `instant.with_timezone(&chrono_tz::Europe::Zurich).date_naive()`
  — jamais ambiguë dans ce sens (un instant UTC a une seule heure locale), contrairement à `from_local_datetime`. **La
  bascule** : `Horloge::aujourd_hui()` appelait depuis la 35-5a0 la fonction privée `jour()` (jour UTC) ; `jour()` est remplacée
  par `jour_suisse`, et `aujourd_hui()` devient `jour_suisse(self.maintenant())` — le serveur, le MCP, et les tests qui passent
  par `aujourd_hui_test()` changent de jour ensemble (C-e350-29). `chrono-tz` 0.10.4 est déjà au `Cargo.lock` (par `rxing` 0.7.1, features par défaut, base IANA 2025b
  précompilée : ni `/usr/share/zoneinfo` ni `TZ` dans l'image) : ajouté aux dépendances de `kesh-core` à la même version,
  features par défaut, sans nouveau paquet (constater par `cargo tree -p kesh-core -e features` à l'étape 0). Pas de `TZ`,
  pas de `chrono::Local`. Le compte de la garde de `horloge.rs` reste 1 **à chaque étape** (V4-4) : le squelette d'AC1 déclare
  `jour_suisse` avec le corps de `jour()` (`instant.date_naive()`, toujours le jour UTC), supprime `jour()` et fait appeler
  `jour_suisse` par `aujourd_hui()` ; la bascule ne change ensuite que le corps de `jour_suisse`. Les tests d'AC1 comparent à des
  `NaiveDate::from_ymd_opt` fixes, sans `.date_naive()`.
- **D2 — `kesh-db` ne lit plus l'horloge** pour une date : **posé par la 35-5a0** (sa fiche, D2 et la fermeture des
  appelants). Rappel : une lecture de l'horloge par requête, passée **à la fois** au bloqueur (rang 5,
  `settlement_cancellation.rs:172`) et à la contre-passation (`journal_entries.rs:2935`).
- **D3 — L'horloge vit dans `AppState` et dans `Lecture`** (C-e350-21) : **posé par la 35-5a0** — trait `Horloge`,
  `HorlogeFixe`, `HorlogeSysteme` (construit dans `main.rs` et `horloge_de_test()` seulement), `horloge_de_test()` partagée par
  `new_for_tests` et les littéraux de test, `Lecture::new(pool, horloge)`, câblage du MCP en trois lignes courtes et ses
  gardes (une garde de la ligne `horloge_outils` dans une liste **voisine, non `LIB_NOMME_MCP`**), les neuf sites de la 35-5a
  déjà branchés sur l'horloge, `services::factures::reglements` qui lit la date une fois. M6 (câblage du MCP sur
  `HorlogeSysteme`) est tuée par la garde de construction de `HorlogeSysteme` de la 35-5a0.
- **D4 — La garde de l'inventaire** (C-e350-24), **posée par la 35-5a0** et décrite ici : un test,
  `crates/kesh-api/tests/date_du_jour_garde.rs` (sur le modèle de `local-date-guard.test.ts`), parcourt
  **`crates/*/src/**/*.rs` et `crates/*/tests/**/*.rs`** ; l'unité du compte est **l'occurrence** après réduction des
  blancs (validation P3, V3-7), les comptes s'écrivent **depuis l'exécution de la garde**, non depuis les tableaux :
  - **Texte lu** : lignes dont le début, espaces retirés, est `//` (`//`, `///`, `//!`) **écartées** ; puis tout blanc (saut de
    ligne compris) entre `)` et `.`, et autour de `.` dans une chaîne de méthodes, réduit — si bien qu'une forme coupée sur
    plusieurs lignes est vue (les 5 sites ci-dessus). Un commentaire **de fin de ligne** reste compté : s'il en paraît un, il
    se réécrit.
  - **Formes relevées**, en expressions bornées (`\b` en tête, parenthèse en queue : V2-5 de la validation P2 — « casse
    ignorée » sur `UTC_DATE` seul relevait `Utc.from_utc_datetime(` à `kesh-api/src/exports/csv_tables.rs:98`) :
    - Rust, casse respectée, sur **tout récepteur** : `\.date_naive\s*\(`, `\.naive_utc\s*\(\s*\)\s*\.\s*date\s*\(`,
      `\.naive_local\s*\(`, `\bUtc::today\b`, `\bLocal::`, et `\bUtc::now\s*\(\s*\)\s*\.\s*(year|month|day|format)\s*\(` ;
    - SQL, casse ignorée : `\bUTC_DATE\s*\(`, `\bCURDATE\s*\(`, `\bCURRENT_DATE\b`,
      `\bDATE\s*\(\s*(NOW|UTC_TIMESTAMP|CURRENT_TIMESTAMP)\b`.
    Aujourd'hui, `.date_naive()` et `naive_utc().date()` n'ont dans `src` aucun récepteur autre que `Utc::now()`, les formes
    `Utc::now().year(`… ne rendent que `admin.rs:722` (`Utc::now().format("%Y%m%dT%H%M%S%3f")`, horodatage), `DATE(UTC_TIMESTAMP`
    et `CURRENT_DATE` rien : la forme large ne coûte rien. Elle inventorie les sites non résolus, non les formes qui marchent
    (`CLAUDE.md`). **Angle mort assumé**, écrit au Change Log : un jour tiré d'une variable d'instant (`let n =
    Utc::now().naive_utc(); n.date()`) n'est pas relevé — `.date()` sur tout récepteur rendrait des centaines de lignes
    légitimes (`NaiveDateTime` des montages).
  - **Liste close d'exemptions, par fichier, en égalité stricte (`==`)** : chaque entrée `{ fichier, nombre, story, motif }`.
    Un fichier absent de la liste doit compter **0**. Un compte qui diffère du nombre écrit, **dans un sens ou dans l'autre**,
    rougit : la liste se ramène au compte réel à chaque conversion, comme un cliquet. **Entrées permanentes** :
    `kesh-core/src/horloge.rs` (1, le `.date_naive()` du jour) ; `kesh-api/src/routes/admin.rs` pour sa ligne `:722` (1,
    horodatage du nom de sauvegarde, un instant), à côté de ses entrées 35-5b. **Le fichier de la garde est exclu nommément**
    du balayage (ses sondes et ses motifs contiennent les formes) ; une sonde prouve qu'un autre fichier du même dossier est
    bien lu.
  - **Fichiers communs** : la liste d'exemptions est éditée par la 35-5a0, la 35-5a, la 35-5b, la 35-5c, dans cet ordre (chaque
    story part de la précédente fusionnée) ; une story sœur qui ajoute un test (35-2a, 35-3, 35-4, 35-6, 35-8) n'y écrit pas de
    date UTC (règle aux *Fichiers communs*), et l'intégration relance la garde après chaque fusion.
  - **Après la 35-5a**, la liste porte : les 19 sites de production de la 35-5b et ses commentaires restants ; les modules de
    tests de `src` non convertis (`journal_entries.rs` : 50 moins ceux que la 35-5a convertit ; `invoices.rs` : 6, 35-5b) ;
    les fichiers de `crates/*/tests` laissés à la 35-5b ou à la 35-5c (verdict ci-dessous), à leur compte exact. Les fichiers
    convertis par la 35-5a n'y figurent pas (compte 0).
  - Sondes au Change Log : positive (un `Utc::now().date_naive()` ajouté à un fichier non exempté rougit ; la même forme
    coupée sur deux lignes aussi) et de compte (une exemption portée à `nombre + 1` rougit).
- **D5 — Les tests ne dépendent plus de l'heure réelle** pour les sites convertis : un test de `kesh-db` passe la date qu'il
  asserte ; un test HTTP qui asserte une date fixe l'horloge de son `AppState` (`HorlogeFixe`). Un montage qui veut « un
  exercice qui couvre aujourd'hui » prend le jour du serveur par **`aujourd_hui_test()`**, helper partagé par crate de tests
  **posé par la 35-5a0** (`crates/kesh-api/tests/common/jour.rs`, `crates/kesh-db/tests/support/jour.rs`,
  `crates/kesh-report/tests/support/jour.rs`) : il rend l'`aujourd_hui()` de l'horloge de test, donc le jour UTC jusqu'à la
  bascule et le jour suisse après, sans retouche (C-e350-29). Les `mod tests` de `src` écrivent en ligne
  `HorlogeFixe(Utc::now()).aujourd_hui()` ou une date fixe ; **ils ne lisent pas `KESH_TEST_INSTANT`** (V4-2) : la forme en
  ligne n'y est admise que si le test ne date rien lui-même ; sinon l'appel reçoit **la date que le montage tient** (ne jamais
  placer `HorlogeFixe(Utc::now()).aujourd_hui()`, jour suisse après la bascule, à côté d'un montage en `Utc::now().date_naive()` :
  deux horloges mêlées). Inventaire à l'étape 0, au Change Log, ligne par ligne :
  `git grep -nE 'HorlogeFixe\(\s*(chrono::)?Utc::now' -- 'crates/*/src/*' 'crates/*/tests/*'`. La nuit se teste par instants
  fixés.
  - **Fichiers mixtes** (un site 35-5a et un site 35-5b, V4-7) : la partie 35-5a passe à l'horloge fixée ou à
    `aujourd_hui_test()` ; la partie 35-5b garde l'heure réelle jusqu'à la 35-5b (ses sites lisent encore `Utc::now()`, que
    `HorlogeFixe` n'atteint pas : `bank_accounts_closed_e2e.rs:677-687`, `closedOn`, en est l'exemple).
  - **Fixer l'horloge d'un `AppState` de test** (V2-7) : le champ est `pub` — `let mut state = AppState::new_for_tests(…);
    state.horloge = Arc::new(HorlogeFixe(instant));` ; un helper `crates/kesh-api/tests/common/horloge.rs`,
    `pub fn avec_horloge(state: AppState, instant: DateTime<Utc>) -> AppState`, évite de le réécrire dans chaque `spawn_app`.
  - **La preuve de nuit** (C-e350-26, mécanisme **posé par la 35-5a0**) : `horloge_de_test()` — donc `new_for_tests`, les
    littéraux de test et `aujourd_hui_test()` — prend une `HorlogeFixe` à l'instant de `KESH_TEST_INSTANT` (RFC 3339) si la
    variable est posée ; lue à chaque appel, sans `static` ni `OnceLock` ; `main.rs` ne la lit jamais ; gardes de
    durcissement (fiche 35-5a0, D5). Ici, deux usages (AC9) : **la nuit réelle reproduite de jour**,
    `KESH_TEST_INSTANT="$(date -u +%F)T23:30:00Z"` — les montages non convertis et MariaDB restent au jour UTC D, le serveur et
    les montages convertis passent au jour suisse D+1, exactement comme à 00 h 30 ; et **deux instants fixés** pour les
    fichiers 35-5a (passage d'année, heure d'hiver). `faketime` est absent de la station (`command -v faketime` vide, aucune
    `libfaketime`).
- **D6 — Docs et commentaires** : `api-external.md:272` dit « du jour, à Zurich » ; les commentaires qui disent UTC et
  deviennent faux sont réécrits : `bank_accounts.rs:794-795` (« date du jour (UTC, comme le complément de l'écran) »),
  `opening_complement.rs:22` (« aujourd'hui (date UTC, passée en paramètre) ») ; `companies.rs:986-989` est réécrit par la
  35-5a0 (V4-8 ; le relevé `git grep -nE '^\s*//.*(Utc::now|UTC)'` sur les fichiers des neuf sites ne rend que ces cinq
  lignes, `bank_accounts.rs:228` et `:1757` étant à la 35-5b) ; **et** le doc-comment de `synthese_accueil`,
  `kesh-api/src/services/rapports.rs:862-864` (« ⚠️ La date est celle de l'horloge en UTC, **comme à la contre-passation** »,
  lu ; validation P5, hors remédiation) : il devient faux **ici**, la contre-passation passant à Zurich quand l'accueil reste
  en UTC jusqu'à la 35-5b ; la 35-5a en retire la seule clause « comme à la contre-passation », et la 35-5b réécrit le reste
  avec le site 21 (sa fiche le liste déjà) ; puis `/usr/bin/grep -n UTC` sur chaque fichier touché, résultat
  au Change Log (greper la valeur, non la formulation). **Le manuel** (V4-1) : les deux phrases « en temps universel » sont
  réécrites — `user-manual.tex:995`, l'item devient « \textbf{Le jour est celui de Zurich}, même entre minuit et deux heures du
  matin » (ou disparaît au profit de la définition unique) ; `:2458`, la phrase « Le jour est celui du serveur, en temps
  universel… (#1022, comme pour le mode compléter) » est retirée ou réécrite de même. Puis le manuel dit une fois, à l'endroit
  où il définit les écritures datées du jour, que « la date du jour est celle de Zurich, même la nuit », renvoyé depuis le
  solde d'ouverture et le verrou de période ; il distingue l'écran (la date du **poste**, `todayLocal`, #889) du serveur
  (Zurich), qui coïncident pour un poste réglé sur la Suisse.
- **D7 — L'écran du verrou suit** (`settings/+page.svelte:563-567`) : `maxLockDate` = la veille du jour local, `todayLocal`
  rendant une chaîne : `new Date(y, m − 1, d − 1)` sur ses champs, remise en `AAAA-MM-JJ` par `todayLocal(…)` ; l'entrée
  d'allowlist `local-date-guard.test.ts:55-58` est retirée, et le commentaire de ce fichier qui compte ses entrées passe de
  « huit » à « sept » (à recompter). `maxLockDate` est une constante évaluée à l'initialisation du script : le Vitest
  (`settings-page.test.ts`) **fixe le fuseau** par `pinTimeZone('Europe/Zurich')` (précédent `local-date.test.ts:8`,
  `pin-time-zone.test-helper.ts` ; Vitest n'a pas de fuseau global et la CI tourne en UTC), active des faux minuteurs **sur la
  seule `Date`** — `vi.useFakeTimers({ toFake: ['Date'] })`, comme `local-date.test.ts:19-20`, `vi.useRealTimers()` en
  `afterEach` (V4-9 : des minuteurs entièrement factices bloqueraient `findByTestId`) —, puis pose
  `vi.setSystemTime(new Date('2026-03-10T23:30:00Z'))` — le même instant que l'AC4 serveur — **avant** le rendu, et lit
  l'attribut `max` du champ : `2026-03-10` (le code de départ, veille UTC, rend `2026-03-09`). Sans le fuseau fixé, une date
  construite en champs locaux rendait le test vert avant correction sous TZ=UTC (validation P3, V3-3).
  *(Annoté le 2026-11-13, clôture, C3 de la revue P2 : la règle « la veille du jour **local** » de D7 est **remplacée** par
  « la veille du jour **de Zurich** » — `maxLockDate = dayBefore(todayInZurich())` (`local-date.ts`), commit `527f57839`,
  remédiation de la revue P1 (E1) : la veille du jour du poste proposait, sur un poste à l'est de Zurich, une borne que le
  serveur refuse. Vitest épinglé aussi à New York et à Athènes. Le texte ci-dessus reste tel qu'écrit à sa date.)*

## Critères d'acceptation

1. **AC1 — La fonction pure** (`kesh-core`, tests unitaires). Le jour suisse ne diffère du jour UTC qu'entre 22:00Z ou
   23:00Z et minuit UTC : les instants sont pris dans cette fenêtre, de part et d'autre de chaque changement d'heure.

   | Instant | Heure à Zurich | Jour attendu | Ce qu'il attrape |
   |---|---|---|---|
   | `2025-12-31T23:30:00Z` | 00 h 30 CET | `2026-01-01` | jour UTC (M1) |
   | `2026-07-01T22:30:00Z` | 00 h 30 CEST | `2026-07-02` | fuseau fixe +1 h (M2) |
   | `2026-03-28T22:30:00Z` | 23 h 30 CET | `2026-03-28` | fuseau fixe +2 h |
   | `2026-03-28T23:30:00Z` | 00 h 30 CET | `2026-03-29` | jour UTC, veille du passage à l'heure d'été |
   | `2026-03-29T22:30:00Z` | 00 h 30 CEST, première nuit d'été | `2026-03-30` | fuseau fixe +1 h |
   | `2026-10-24T22:30:00Z` | 00 h 30 CEST | `2026-10-25` | fuseau fixe +1 h, veille du retour à l'heure d'hiver |
   | `2026-10-25T22:30:00Z` | 23 h 30 CET | `2026-10-25` | fuseau fixe +2 h (l'inverse de M2) |
   | `2026-06-15T10:00:00Z` | 12 h CEST | `2026-06-15` | le jour |
   | `2026-06-30T21:59:59Z` | 23 h 59 min 59 s CEST | `2026-06-30` | borne : décalage d'une demi-heure, `<` contre `≤` |
   | `2026-06-30T22:00:00Z` | 00 h CEST | `2026-07-01` | borne, été |
   | `2025-12-31T22:59:59Z` | 23 h 59 min 59 s CET | `2025-12-31` | borne, hiver |
   | `2025-12-31T23:00:00Z` | 00 h CET | `2026-01-01` | borne, hiver |

   Et `HorlogeFixe(i).aujourd_hui() == jour_suisse(i)` pour chacun.
2. **AC2 — Solde d'ouverture la nuit** (#1022). Montage (lu à `opening_complement.rs:255-283`, branche (b)) : exercice 2025
   **ouvert**, premier exercice, son premier jour **verrouillé** (borne au `2025-06-30`, posée en SQL) ; exercice 2026
   ouvert. Horloge fixée à `2025-12-31T23:30:00Z`. `POST /bank-accounts` avec `openingBalance`, puis
   `POST /bank-accounts/{id}/opening-balance` sur un autre compte → écriture datée **`2026-01-01`, dans l'exercice 2026** (le
   code de départ la date du `2025-12-31` dans l'exercice 2025). Même chose pour le complément des *Soldes de départ*
   (`POST /opening-balances/complete`, `opening_balances.rs:604`) et son statut (`GET`, `:244` : `branch: Today`, date
   `2026-01-01`).
3. **AC3 — Contre-passation et annulation la nuit** (#929, en partie) : horloge fixée à `2025-12-31T23:30:00Z`, exercices
   2025 et 2026 ouverts : `POST /journal-entries/{id}/reverse` → écriture inverse datée `2026-01-01` dans 2026 (le code de
   départ : `2025-12-31` dans 2025). **Un cas d'acceptation par chemin qui contre-passe** (V2-2 de la validation P2), même
   montage, la date de l'écriture inverse assertée à `2026-01-01` dans l'exercice 2026 :
   - annulation d'un règlement de facture client, `POST /invoices/{id}/settlements/{settlement_id}/cancel`
     (`invoice_settlements_write.rs:1023`, `cancel_settlement_in_tx`) ;
   - annulation d'une facture fournisseur validée, `POST /supplier-invoices/{id}/cancel` (`supplier_invoices.rs:2600`) ;
   - annulation du règlement d'une facture fournisseur, `POST /supplier-invoices/{id}/settlement/cancel` (`:2871`) ;
   - annulation d'un rapprochement, `POST /reconciliation/transactions/{id}/cancel` (`reconciliation_cancel.rs:700-723`).
   Sous M7 (la contre-passation relit l'horloge réelle), ces cas rougissent. Et le refus : avec le seul exercice 2025 ouvert
   (aucun ne couvre le `2026-01-01`), l'annulation d'un règlement est refusée au rang 5 (`settlement_cancellation.rs:172`)
   **par le refus nommé**, et rien n'est écrit (le code de départ accepte et contre-passe au `2025-12-31`).
4. **AC4 — Verrou de période** (#907) : horloge fixée à `2026-03-10T23:30:00Z` (00 h 30 le 11 mars à Zurich).
   - `lock_books` jusqu'au `2026-03-10` : **accepté** (le code de départ refuse) ; jusqu'au `2026-03-11` : refusé —
     réponse `400 INVALID_INPUT`, clé `settings-books-lock-bound-not-past` (constante Rust `BOOKS_LOCK_BOUND_NOT_PAST`,
     `companies.rs:972` ; `errors.rs:4778-4795`).
   - `unlock_books` : borne courante au `2026-03-10` **posée en SQL**, comme à l'AC2 (V4-5 : sur la 35-5a0, `lock_books` la
     refuserait et le test tomberait au montage ; `unlock_books` contrôle la date, `companies.rs:1099-1100`, avant la garde
     `vise > avant`, `:1128-1131`), cible `2026-03-10` — un retrait sans effet, qui écrit tout de même `books.unlocked` —
     **acceptée** (le code de départ refuse : `2026-03-10 >= ` jour UTC) ; cible `2026-03-11` → refusée, même réponse. Une
     cible au `2026-03-09` passe déjà sur le code de départ : elle ne prouve rien.
   - Écran (D7) : fuseau fixé à `Europe/Zurich` (`pinTimeZone`), instant `2026-03-10T23:30:00Z` : `maxLockDate` vaut
     `2026-03-10` (le code de départ : `2026-03-09`) ; l'allowlist n'a plus l'entrée du verrou.
5. **AC5 — Premier exercice de l'assistant** (site 17, C-e350-23) : finalisation du chemin B
   (`routes/onboarding.rs:1043`) à `2025-12-31T23:30:00Z` → l'exercice créé s'appelle « Exercice 2026 » et couvre 2026 (le
   code de départ : « Exercice 2025 »).
6. **AC6 — (à la 35-5a0)** `kesh-db` sans horloge : sa fiche, D2 et AC5 (V4-3).
7. **AC7 — Les services et le MCP au jour suisse** (C-e350-21, plomberie de la 35-5a0) : un `AppState` à
   `HorlogeFixe(2025-12-31T23:30:00Z)` donne `state.lecture().aujourd_hui() == 2026-01-01` (la 35-5a0 : `2025-12-31`) ;
   `services::factures::reglements` rend, pour un règlement dont l'exercice du jour suisse est clos, le motif du rang 5 (route
   `GET /invoices/{id}/settlements`) ; de même `GET` des candidats au rattachement (`routes/reconciliation.rs:6587`,
   `get_entry_candidates` → `entry_cancel_outlook`, rang 5). L'AC fonctionnel par un outil du MCP est à la 35-5b.
8. **AC8 — La garde** (D4) : la liste se ramène aux comptes réels après les conversions de cette story (relevés par
   l'exécution de la garde) ; sondes du Change Log (positive, multi-ligne, compte + 1).
9. **AC9 — Tests sûrs la nuit** (C-e350-22, C-e350-26) : après cette story, aucun test ne rougit entre minuit et 1 h (2 h en
   été) à Zurich du fait d'un site de la 35-5a. Les fichiers marqués **35-5a** au tableau ci-dessous sont convertis (date
   fixe, horloge fixée, ou `aujourd_hui_test()`) et comptent 0 à la garde ; le `mod tests` de `journal_entries.rs` convertit
   les tests qui assertent une date de contre-passation. **Deux preuves, au Change Log** (`kesh-gate-logs/e355a/nuit.log`) :
   - **la nuit réelle reproduite de jour, sur tous les binaires de tests** (validation P3, V3-1) :
     `KESH_TEST_INSTANT="$(date -u +%F)T23:30:00Z"` — montages non convertis et MariaDB au jour UTC D, serveur et montages
     convertis au jour suisse D+1. **Chaque rouge** se rattache à un fichier classé 35-5b ou 35-5c, avec son motif ; un rouge
     dans un fichier 35-5a, dans un appelant de la fermeture de la 35-5a0, ou dû à un site 1–9, est un **défaut** de cette
     story. Le verdict « 35-5c » des fichiers « à lire » (`credit_card_e2e.rs`, `reconciliation_link_e2e.rs`, les lignes du
     second tableau) s'établit par cette exécution, non par affirmation ;
   - **deux instants fixés** pour les binaires des fichiers 35-5a : `KESH_TEST_INSTANT=2025-12-31T23:30:00Z` (passage
     d'année) puis `2026-03-10T23:30:00Z` (heure d'hiver) : verts.
   - **Binaires que la variable n'atteint pas** (V4-2) : le `lib` de chaque crate (`mod tests` de `src`, dont
     `journal_entries.rs`, qui appelle `reverse_in_tx` à `:3984` et `:5824`). Pour eux, **la preuve se fait par la valeur** :
     une date de nuit fixée là où l'attendu en dépend (comme la 35-5b, D9), et l'inventaire des `HorlogeFixe(…Utc::now…)` de
     D5. `kesh-report/tests` (`open_items_invariant.rs:182`, `:218` ; `aged_receivables.rs:609`, `vat_report_write_off.rs:291`)
     est atteint par son `support/jour.rs` (35-5a0).
   - **Hors passage d'année, sans risque** (V4-10) : la nuit reproduite de jour vaut pour la nuit du jour où l'on lance ; les
     montages par année du jour (`Utc::now()…year()`) sont tous dans des fichiers 35-5a — `bank_account_opening_balance_e2e.rs:172`,
     `kesh-db/tests/filet_bilan_clos.rs:66`, `letterings.rs:1458`, `opening_complement_repository.rs:878`,
     `fiscal_years_repository.rs:1863`, et `kesh-db/src/repositories/journal_entries.rs:4457` (`mod tests` de `src`, binaire que la
     variable n'atteint pas : preuve par la valeur ; validation P5, V5-2) —, plus `fiscal_years_e2e.rs:1284` (35-5b, `seed_demo`) ; les autres s'appuient sur
     l'exercice 2020-2030 de `seed_accounting_company` (`kesh-db/src/test_fixtures.rs:71`). Relevé à confirmer à l'étape 0.
10. **AC10 — Le chemin de jour ne change pas** : en journée (`2026-06-15T10:00:00Z`), les dates écrites sont celles d'avant ;
    tests existants verts sans retouche de leurs attendus, hors AC9 et **hors le test d'AC2 de la 35-5a0** (V4-3), dont
    l'attendu à `HorlogeFixe(2025-12-31T23:30:00Z)` passe de `2025-12-31` à `2026-01-01`, dans l'exercice ouvert de son
    montage (`seed_accounting_company`, 2020-2030, nommé par la 35-5a0 ; validation P5, V5-3) : c'est le test
    que la 35-5a retourne, nommé aux *Tâches*.
11. **AC11 — Docs, manuel, CHANGELOG, PR** : D6 ; **`/usr/bin/grep -nE 'temps universel|encore la veille'
    docs/manual/fr/user-manual.tex` vide, et le même contrôle sur le PDF aplati** (`pdftotext user-manual.pdf - | tr '\n' ' ' |
    tr -s ' '`, puis les deux motifs), au Change Log (V4-1 ; validation P5, V5-1 : « encore la veille » porte l'énoncé faux
    même si les trois mots « en temps universel » sont retirés seuls) ; PDF
    régénéré, contrôlé aplati ; CHANGELOG `## [0.17.7] — Non publié`,
    *Corrigé* : un point pour #1022, un pour #907, et la 35-5a **ouvre** le point #929 (contre-passation, annulation,
    premier exercice) que la 35-5b **complète** sans en écrire un second (C-e350-8, comme 35-2a / 35-2b). La PR porte
    `closes #1022`, `closes #907`, `refs #929`. **Aucune clé i18n** ajoutée ni modifiée, les quatre locales inchangées : le
    message `settings-books-lock-bound-not-past` (« antérieure à aujourd'hui ») reste vrai.
12. **AC12 — Aucune migration** : `git diff --stat` ne touche pas `crates/kesh-db/migrations/` ; le dire au Change Log.

### Verdict des 38 fichiers de `crates/*/tests` (provisoire, à confirmer à l'étape 0)

Critère : **35-5a** si le fichier fait dater ou juger par le serveur (route, service) un site 1–9 en le comparant au jour
qu'il calcule, ou appelle une fonction de D2 ; **35-5b** si son attendu change avec un site de la 35-5b ; **35-5c** sinon
(jour du montage sans comparaison avec le jour du serveur, ou marge de plusieurs jours). Un appel **direct** à une fonction
de D2 lui passe la date que le test tient déjà : sans risque la nuit, puisque le test la fournit. Un fichier qui atteint à
la fois un site de la 35-5a et un de la 35-5b : la partie 35-5a passe à l'horloge fixée ou à `aujourd_hui_test()`, la partie
35-5b garde l'heure réelle jusqu'à la 35-5b (D5, V4-7).
Le verdict est **provisoire** ; l'exécution « nuit réelle » d'AC9 le tranche.

| Fichier (sites) | Verdict | Motif (indice relevé) |
|---|---|---|
| `kesh-api/tests/accounts_e2e.rs` (1 + 1 multi-ligne) | 35-5c | exercice de l'année pour le retypage |
| `admin_full_import_e2e.rs` (4) | 35-5c pour ses 4 sites ; **35-5a** pour l'appel `journal_entries::reverse` de `:1809` (signature) | bornes à `today - 30` : marge |
| `bank_account_opening_balance_e2e.rs` (2) | **35-5a** | solde d'ouverture (sites 1–2) |
| `bank_accounts_closed_e2e.rs` (6) | **35-5a** pour `:323`, `:386`, `:500`, `:1651` ; 35-5b pour `:678`, `:685` | règlement puis annulation (rang 5) ; `closedOn` (site 11) |
| `credit_card_e2e.rs` (3) | 35-5c | dates saisies de facture et de règlement (à confirmer : aucune annulation) |
| `dates_bornes_e2e.rs` (3) | **35-5a** | verrou au jour |
| `dunning_reminders_e2e.rs` (1) | 35-5b | sites 13, 22 |
| `filet_bilan_clos_e2e.rs` (3 + 1 multi-ligne) | **35-5a** | contre-passation du jour |
| `fiscal_years_e2e.rs` (1 + 1 multi-ligne) | **35-5a** pour `:1064-1072` ; 35-5b pour `:1284` | `path_b_finalize_creates_fiscal_year` (`:1044`) asserte « Exercice {année UTC} » (site 17, lu) ; `seed_demo` (site 26) |
| `homepage_summary_e2e.rs` (1) | 35-5b | accueil (site 21) ; ses `journal_entries::reverse` directs reçoivent `today()` (signature) |
| `invoice_delete_e2e.rs` (2) | 35-5c | dates de facture ; la dévalidation **supprime** l'écriture (`kesh-db invoices.rs:1470-1476`, aucune contre-passation) ; `:300` pose la borne en SQL à `today − 30` |
| `invoice_unvalidate_e2e.rs` (6) | 35-5c | dates de facture ; même motif (validation P2, V2-10) |
| `journal_entry_reversal_e2e.rs` (19 + 1 multi-ligne) | **35-5a** | contre-passation, `:319` asserte le jour |
| `letterings_e2e.rs` (1 + 1 multi-ligne) | **35-5a** | annulation de règlement (à confirmer) |
| `mcp_enveloppe_e2e.rs` (1) | 35-5c | date saisie d'une pièce |
| `mcp_outils_e2e.rs` (1) | 35-5b | montage des outils (balance âgée, postes ouverts) |
| `onboarding_audit_e2e.rs` (1, `Local::now`) | 35-5b | date de facture dans l'exercice de démonstration (site 26) |
| `onboarding_e2e.rs` (1) | 35-5b ; **35-5a** pour toute assertion de l'exercice du chemin B | site 26 ; site 17 |
| `opening_balances_e2e.rs` (1) | **35-5a** | complément (sites 3–4) |
| `open_items_e2e.rs` (1) | 35-5b | `asOf` (site 18) |
| `period_lock_e2e.rs` (19) | **35-5a** | verrou (sites 7–8) |
| `reconciliation_e2e.rs` (1) | **35-5a** | annulation d'un rapprochement, contre-passation du jour (`:4285` à lire) |
| `reconciliation_link_e2e.rs` (1) | 35-5c, sauf annulation | `:757` jour de facture et de transaction (à lire) |
| `reconciliation_rules_e2e.rs` (1) | **35-5a** | contre-passation du jour (`:1920-1926`, lu) |
| `rejeu_interblocage_e2e.rs` (1) | **35-5a** | « l'exercice ouvert qui couvre aujourd'hui », annulations |
| `services_motif.rs` (3) | 35-5b | traces des rapports (sites 18–21) |
| `kesh-db/tests/dunning_eligibility.rs` (1) | 35-5b | site 22 |
| `filet_bilan_clos.rs` (3) | **35-5a** | contre-passation, bloqueur (signature) |
| `fiscal_years_repository.rs` (1) | **35-5a** | `reverse` (signature) |
| `invoice_settlement.rs` (4) | **35-5a** | bloqueur, annulation |
| `letterings.rs` (2) | **35-5a** | verrou `:2348`, signature |
| `open_items.rs` (1) | **35-5a** | verrou, contre-passation (signature) |
| `opening_complement_repository.rs` (2) | **35-5a** | complément (`today` passé) |
| `reconciliation_cancel.rs` (1) | **35-5a** | `:440` asserte la date de la contre-passation |
| `supplier_invoices_draft.rs` (1) | **35-5a** | `cancel` (signature) |
| `supplier_invoices_repository.rs` (4) | **35-5a** | bloqueur, annulation |
| `support/lettering_documents.rs` (1) | **35-5a** | module des fichiers convertis (marge `today - n`) |
| `kesh-report/tests/open_items_invariant.rs` (1) | **35-5a** | `reverse` (signature, `:182`) |

Décompte provisoire (validation P2) : **23** fichiers en 35-5a (dont deux en partie, `bank_accounts_closed_e2e.rs` et
`fiscal_years_e2e.rs`), **8** en 35-5b (plus leurs parts), **7** en 35-5c (dont `admin_full_import_e2e.rs`, dont l'appel de
`:1809` change ici) — 38. À recompter.

**Second tableau — appelants des fonctions de D2 sans site de jour**, **adaptés par la 35-5a0** (la compilation l'impose ; la
fermeture complète, avec les 158 appels des écritures fournisseur, est dans sa fiche). Leur verdict de nuit se tranche ici,
par l'exécution « nuit réelle » d'AC9 :

| Fichier | Appel | Action (35-5a0) | Nuit (35-5a) |
|---|---|---|---|
| `kesh-api/tests/invoice_echeancier_e2e.rs` | `lock_books` `:2014` | date fixe ou `aujourd_hui_test()` | à lire : borne comparée au jour ? |
| `supplier_settlement_cancel_e2e.rs` | `lock_books` `:749` | idem | à lire (annulation au jour) |
| `supplier_invoice_response_e2e.rs` | `commit_with_view`, `get_settlement_view_in_tx` `:434-496` | date passée | à lire |
| `kesh-db/tests/invoice_write_off.rs` | `settlement_cancel_blocker`, `lock_books`, `get_view` `:649-1558` | date passée | à lire |
| `journal_entries_modification.rs` | `reverse_in_tx` `:365`, `:473` | date fixe | sûr (date fournie) |
| `lettering_documents.rs` | `lock_books`, `unlock_books` `:755-1040` | date fixe ou `aujourd_hui_test()` | à lire |
| `lettering_documents_backfill.rs` | `unlock_books`, `reverse` ×6 `:289-772` | idem | à lire |
| `migrations_upgrade_path.rs` | `supplier_invoices::cancel_in_tx` `:1508`, `:1521` ; `get_settlement_view` `:1640` | date fixe | sûr (date fournie) |
| `bank_account_closed_inventory.rs`, `letterings_lexical.rs` | gardes lexicales sur les noms | aucune, à constater vertes | — |
| les 23 fichiers des appels `supplier_invoices::{validate,…}`, ceux de `cancel_settlement(` et de `reconciliation_cancel::*` | voir la fiche 35-5a0 | `aujourd_hui_test()` ou date tenue | par AC9 |

Règle de nuit pour ces fichiers : un appel qui reçoit `aujourd_hui_test()` suit le jour du serveur ; si le montage du même test
date ses pièces par `Utc::now()`, il se désaccorde la nuit après la bascule — un tel montage passe aussi au helper, dans la
35-5a.

## Tests à écrire, et la preuve du rouge

Écrire d'abord, lancer **sur la 35-5a0 fusionnée**, journal `kesh-gate-logs/e355a/rouges.log`. La 35-5a0 est le squelette : les
neuf sites lisent déjà l'horloge, et `Horloge::aujourd_hui()` rend le jour UTC (fonction privée `jour()`). Les rouges sont
donc **déterministes** et disent le défaut : AC1 (`jour_suisse` n'existe pas : squelette commité d'abord — `jour_suisse` reprend
le corps de `jour()`, qui disparaît, et `aujourd_hui()` l'appelle : un seul `.date_naive()` dans `horloge.rs`, la garde reste
verte (V4-4) — puis rouge par attendu — `2025-12-31` attendu `2026-01-01`), AC2, AC3 et AC5 (la veille, et l'exercice
précédent), AC4 (`2026-03-10` refusé ; le Vitest, `2026-03-09`), AC7 (`2025-12-31`). Ils ne dépendent pas du jour réel où on
les lance. La bascule (D1) les met au vert.

**Mutations** : M1 `jour_suisse` rend le jour UTC ; M2 l'été sans +2 h (fuseau fixe +1) ; M3 un site de la 35-5a qui lit
`Utc::now()` (garde, et AC2 à AC5) ; M4 `lock_books` comparé au jour UTC ; M5 une exemption de la garde ajoutée sans site (la
garde rougit sur le compte) ; M7 la contre-passation de l'annulation relit l'horloge au lieu de recevoir la date du bloqueur
(AC3 : les cas d'acceptation par chemin) ; M9 `maxLockDate` remis à la veille UTC (AC4, Vitest au fuseau fixé). M6 et M8
(câblage du MCP, `KESH_TEST_INSTANT`) sont à la 35-5a0.

## Tâches

- [x] Étape 0 : partir de la 35-5a0 fusionnée ; recompter les sites et les tests ; confirmer les deux tableaux de verdict,
      ligne lue citée ; `cargo tree` pour `chrono-tz` ; Change Log.
- [x] Squelette `jour_suisse`, tests, rouges consignés.
- [x] `kesh-core` : `jour_suisse`, bascule de `Horloge::aujourd_hui()`. Écran du verrou (D7). Garde ramenée aux comptes réels.
- [x] Retourner le test d'AC2 de la 35-5a0 : à `HorlogeFixe(2025-12-31T23:30:00Z)`, contre-passation au `2026-01-01`, dans
      l'exercice 2020-2030 de son montage (exception nommée à AC10).
- [x] Tests convertis (AC9), inventaire des `HorlogeFixe(…Utc::now…)` (D5), et les preuves de nuit ; commentaires, docs, manuel
      (`:995`, `:2458`), CHANGELOG.
- [ ] Gates : `test-fast.sh --ci` **complet à chaque commit de code** (exception `kesh-db`), `scripts/gate-ci.sh` au dernier
      commit de code, frontend complet, E2E complet au dernier commit de code. *(2026-11-13 : `test-fast` et frontend au
      passage en `review` et à la remédiation P1 ; `gate-ci` au passage en `review` ; **pas** de gate complet à chaque commit
      (consignes de l'Epic 35 du 2026-11-13, qui remplacent cette ligne) ; **E2E et `gate-ci` final : à la clôture**.)*

## Dev Notes

### Hors périmètre

- Les 19 sites restants de la production : **35-5b** ; les tests de `src` et de `crates/*/tests` que ni la 35-5a ni la 35-5b
  ne convertissent, et les specs E2E qui n'assertent aucune date d'un site converti : **35-5c** (C-e350-22).
- Les horodatages (`created_at`, `UTC_TIMESTAMP(6)`, jetons, `imported_at`, le nom horodaté des sauvegardes
  `admin.rs:722`, `Utc::now().format("%Y%m%dT%H%M%S%3f")`) restent en UTC : ce sont des instants, non des dates (#669 pour le
  `time_zone` de la session).
- **Le jour d'un instant enregistré** (C-e350-25) — un horodatage UTC ramené à un jour par `.date()` ou `DATE(…)` — reste en
  UTC dans cette story. Sites relevés : `dunning_eligibility.rs:128` (`lr.date()`, le jour du dernier rappel comparé au jour
  du serveur) et `routes/invoices.rs:1525` (`paid_at` du CSV de l'échéancier) : **35-5b** ; `reconciliation.rs:237`
  (`DATE(rmd.last_sent_at)` dans la date d'exigibilité, en SQL) : **#1053** ; `reconciliation.rs:435` (la contrainte
  `chk_invoices_paid_at_after_date`, `DATE(paid_at) >= date - INTERVAL 1 DAY`) : sa tolérance d'un jour absorbe l'écart, et
  la changer demanderait une migration — hors périmètre, sans issue.

### La nuit, entre la 35-5a et la 35-5b

Un **E2E complet** lancé entre 00 h et 01 h (02 h en été) à Zurich confronte, jusqu'à la 35-5c, un serveur au jour suisse à
des specs qui calculent un jour UTC (`journal-entries.spec.ts`, `invoices-settlement-cancel.spec.ts`,
`invoice-write-off.spec.ts`, `helpers/api-fixtures.ts`). Les specs qui assertent une date d'un site de la 35-5a passent dans
cette story au helper existant `todayInZurich()` (`frontend/tests/e2e/helpers/api-fixtures.ts:390-392`) — au moins, à
confirmer à l'étape 0 : `invoices-settlement-cancel.spec.ts`, `journal-entries.spec.ts`, `invoice-write-off.spec.ts`,
`supplier-invoice-draft-cycle.spec.ts` ; liste écrite au Change Log, la 35-5c part d'elle. Un rouge d'une autre spec à cette
heure n'est pas une régression : le nommer contre cette note.

### Découpage (C-e350-28) — la dérogation C-e350-27 tombe

La validation P2 avait levé le signal (plus de cinq modules) et la fiche y répondait par une dérogation : « le mécanisme ne se
coupe pas ». **La validation P3 (V3-4) a montré la prémisse fausse** : la fiche prescrivait elle-même un squelette compilable
— plomberie et appelants, comportement inchangé —, c'est-à-dire exactement le *story-zero* de `CLAUDE.md`. Décision de
l'orchestrateur : la plomberie part à la **35-5a0** (comportement strictement inchangé, jour UTC, tests verts sans retouche,
garde posée) ; la **35-5a** garde la bascule sur Zurich, ses critères, les conversions de tests qu'elle impose et les preuves
de nuit. La sûreté de nuit reste d'un seul tenant, dans la 35-5a.

### Fichiers communs — règle d'union C-e350-8

`opening_balances.rs` avec la **35-6**, qui n'y touche plus que son `mod tests` (un test de parité, C-e350-19) — ici `:244`,
`:604` en production : hunks distincts ; `bank_accounts.rs` avec la **35-5b** (ici `:1007`, `:1252`,
`:794-795` ; la 35-5b `:1802`, `:228`, `:1757`) ; `journal_entries.rs` avec la **35-8**, qui attend la 35-5c ;
**avec la 35-3** : `kesh-db/src/repositories/supplier_invoices.rs` (35-3 `:1521-1530`, `:1698-1727` ; ici les signatures
`:2462`, `:2600`, `:2750`, `:2871`), `invoice_settlements_write.rs` (35-3 `:679-680` ; ici `:878`, `:1023`),
`crates/kesh-db/tests/supplier_invoices_repository.rs` (35-3 `:2250-2257` ; ici `:1586`, `:1862`, `:2700`, `:2795`) — hunks
distincts, la seconde fusionnée se rebase sur la première. ⚠️ **Conflit de compilation possible** (validation P3, V3-2) : un test
neuf de la 35-3 qui appellerait `supplier_invoices::validate` / `pay` ne compilerait plus après la **35-5a0** ; à ce jour ses
tests neufs appellent `create_validated`, hors de la fermeture (validation P1 de la 35-5a0, V1-7) : à constater à
l'intégration ; la règle d'ordre est à l'epic et dans la fiche 35-5a0 (la seconde fusionnée ajoute la date aux appels de
l'autre, gate complet) ;
`routes/invoices.rs`, `routes/supplier_invoices.rs`,
`routes/imported_supplier_invoices.rs`, `routes/reconciliation.rs`, `services/factures_fournisseur.rs` (chaîne des
appelants) ; `docs/api-external.md:272` : 35-5a seule pour cette ligne (la 35-6 ne touche plus `api-external.md`, C-e350-19 ;
la 35-5b y a `:252`, `:292`, `:443`) ; `user-manual.tex` avec la **35-4** (`:3107`), la **35-6** (`:2456` — **contigu** au
`:2458` que la 35-5a réécrit, V4-1 : hunks distincts mais voisins, union à la main), la **35-3** et la **35-2a** (paragraphes
distincts) ; `frontend/src/lib/shared/utils/local-date-guard.test.ts` avec la **35-5c** (qui l'étend
aux specs, partant de celle-ci fusionnée) ; `settings/+page.svelte` (35-5a seule) ; `CHANGELOG.md`. `lib.rs`,
`services/poignees.rs`, `mcp_gardes_lexicales.rs`, `services_motif.rs` : 35-5a0 ; `tests/common/mcp.rs` avec la 35-5a0 et la
**35-5b** (qui y ajoute une entrée à horloge fixée). `crates/kesh-api/tests/date_du_jour_garde.rs` : créé par la 35-5a0, sa liste
d'exemptions éditée ensuite par la 35-5a, la 35-5b puis la 35-5c (hunks de la même table, dans cet ordre).
**Tests neufs des stories sœurs** (35-2a, 35-2b, 35-3, 35-4, 35-6, 35-8 — V2-14) : la garde à égalité stricte lit
`crates/*/tests/**` ; un test neuf qui écrit `Utc::now().date_naive()` dans un fichier à compte 0 (ex.
`supplier_invoices_repository.rs`, où la 35-3 ajoute `:2250-2257`) la fait rougir. Règle : tout test neuf prend
`aujourd_hui_test()` ou une date fixe ; à l'union, l'intégration relance la garde et corrige le test, non la liste.

### Ce qui doit rester vrai

- Aucune migration ; aucune donnée existante réécrite. Une écriture passée la nuit avant la 0.17.7 garde sa date.
- Les refus existants (exercice fermé, période verrouillée) ne changent pas, seule la date qu'ils jugent. Fixtures
  synthétiques seulement.
- La lecture seule du MCP (Q11) : la `Lecture` des outils garde le pool en lecture seule ; l'horloge n'y ajoute aucun accès.

### Références

- Issues #929, #1022, #907 ; #889 (pendant de l'écran, `local-date.ts`) ; classement `epic35-classement-bugs.md` § 8.3.
- `CLAUDE.md` : § *Exception `kesh-db`* ; § *Règle de splitting préventif* ; § *Inventorier les sites NON RÉSOLUS*.

## Dev Agent Record

### Agent Model Used

Opus 5.5 (agent de développement, worktree `kesh-e355a`, clé `e355a`), 2026-11-13.

### Debug Log References

Journaux sous `/home/gcorbaz/devel/kesh-gate-logs/e355a/` : `rouges.log` (et `rouges-kesh-core.log`, `rouges-api{,-2,-3}.log`,
`rouges-vitest.log`), `verts-api.log`, `verts-vitest.log`, `nuit.log` (synthèse d'AC9) et `nuit-{N0,N1,N2,C0…C9}.log`
(environnement et commit en tête), `mut/` (mutations), `captures/` et `captures.log`, `manuel-make.log`, `gate-*.log`.

### Completion Notes List

- **Commits** : `076d63ed3` squelette (vert) ; `c2e3e788c` bascule, tests neufs, écran, docs, manuel `.tex`, CHANGELOG ;
  `ada7e1d61` conversions d'AC9 (huit fichiers, garde) ; `c5972ca13` passage d'année de `period_lock_e2e` ; puis le commit de
  compte rendu (fiche, PDF, `sprint-status.yaml`, prompt de revue). **Aucune migration** (AC12 :
  `git diff --stat eabafed5f HEAD -- crates/kesh-db/migrations` vide).
  *(Annoté le 2026-11-13, remédiation de la revue P1, A7 : « squelette (vert) » veut dire `cargo test -p kesh-core horloge`
  (7 passés) et `clippy -p kesh-core`, lancés à la main **sans journal** ; aucun gate complet n'a tourné sur `076d63ed3` ni sur
  `ada7e1d61` (le passage `nuit-N3.log` s'est arrêté à la compilation) ; les tests ciblés après la bascule, verts après
  l'alignement du refus d'AC3, n'ont pas de journal propre — ils sont couverts par `nuit-C0.log` et `gate-test-fast.log` à
  `c5972ca13`. La remédiation P1 ajoute le commit de code `527f57839`.)*
- Choix : `e35-choix-autonomes-e355a.md`, C-e355a-1 à -7.
- Voir le Change Log (entrée du développement) pour l'étape 0, les rouges, l'inventaire par exécution, les mutations et les gates.

### File List

**Production** : `crates/kesh-core/{Cargo.toml,src/horloge.rs}` (`jour_suisse`, bascule) ; `Cargo.lock` (une arête,
`kesh-core → chrono-tz` 0.10.4, déjà au graphe) ; commentaires D6 : `crates/kesh-api/src/routes/bank_accounts.rs:794-795`,
`crates/kesh-db/src/repositories/opening_complement.rs:22`, `crates/kesh-api/src/services/rapports.rs:862-864` ;
`frontend/src/routes/(app)/settings/+page.svelte` (D7).

**Tests** : neufs — `crates/kesh-api/tests/jour_suisse_nuit_e2e.rs` (AC2, AC3, AC4, AC7), `crates/kesh-api/tests/common/horloge.rs`
(`avec_horloge`) ; modifiés — `kesh-core/src/horloge.rs` (`mod tests`, AC1), `kesh-api/tests/{horloge_routes_e2e (AC10, test
retourné), horloge_etat (AC7), fiscal_years_e2e (AC5, année du chemin B), common/mod.rs, date_du_jour_garde (comptes),
bank_account_opening_balance_e2e, opening_balances_e2e, dates_bornes_e2e, filet_bilan_clos_e2e, letterings_e2e,
reconciliation_rules_e2e, rejeu_interblocage_e2e, period_lock_e2e}.rs`, `kesh-db/tests/fiscal_years_repository.rs` ;
`frontend/src/routes/(app)/settings/settings-page.test.ts` (AC4 écran), `frontend/src/lib/shared/utils/local-date-guard.test.ts`.

**Docs** : `docs/api-external.md:272`, `docs/manual/fr/user-manual.{tex,pdf}` (§ *La date du jour est celle de Zurich*, `:737`,
complément `:1013`, solde d'ouverture `:2488`, verrou `:711`), `docs/testing.md:119`, `CHANGELOG.md` (`## [0.17.7] — Non publié`, *Corrigé* : #1022, #907, #929 en
partie). Aucune clé i18n, aucune spec E2E (C-e355a-6).

## Change Log

- 2026-11-12 — Fiche rédigée (Epic 35, découpage préventif C-e350-14). Validation non lancée.
- 2026-11-12 — **Validation P1** (Opus 5.5, `kesh-gate-logs/e35-5a-validation-p1.md` : 0 CRITICAL, 0 HIGH, 7 MEDIUM, 9 LOW)
  remédiée par l'agent de rédaction (worktree `kesh-e355`), avec le HIGH V1-1 de la validation P1 de la 35-5b
  (`e35-5b-validation-p1.md`), dont la décision structurante tombe ici. Prémisses lues au code de `release/0.17.6`.
  - **Horloge** (V1-6 ; 35-5b V1-1) : l'horloge vit dans `AppState` **et dans `Lecture`** (C-e350-21) — `Lecture::new(pool,
    horloge)` sans défaut, `AppState::lecture()` et le câblage du MCP (`lib.rs:1403`) passent `state.horloge` ; gardes
    lexicales `mcp_gardes_lexicales.rs:169`, `:575` et `FABRIQUES_V2` mises à jour ; AC7 ; `services::factures::reglements`
    lit la date une fois.
  - **V1-1** : le squelette branche déjà les neuf sites, `jour_suisse` rendant le jour UTC ; rouges déterministes.
  - **V1-2** : AC1 réécrit avec des instants de la fenêtre 22:00Z–minuit, de part et d'autre des changements d'heure ;
    `with_timezone(&Europe::Zurich).date_naive()` prescrit.
  - **V1-3** : D4 récrit (C-e350-24) — égalité stricte par fichier, commentaires `//` écartés, formes larges sur tout
    récepteur et coupées sur plusieurs lignes, pathspec `'crates/*/src/*'` ; `companies.rs:986-989` réécrit (D6) ;
    `aged_receivables.rs:27` reste vrai (« pas `UTC_DATE()` en dur ») et, commentaire, n'est plus compté.
  - **V1-4** : verdict des 38 fichiers (tableau), helper `aujourd_hui_suisse` posé dès cette story, garde étendue à
    `crates/*/tests` dès cette story (fichiers convertis à 0) ; AC9.
  - **V1-5** : site 17 (`onboarding.rs:1043`) déplacé en 35-5a (C-e350-23), AC5 ; partage 9 / 19.
  - **V1-7** : classe « jour d'un instant enregistré » nommée et attribuée (C-e350-25) : `dunning_eligibility.rs:128` et
    `routes/invoices.rs:1525` à la 35-5b, `reconciliation.rs:237` en issue, `:435` sans suite (tolérance, migration).
  - **LOW** : V1-8 (une lecture par requête, D2, M7) ; V1-9 (montage d'AC2) ; V1-10 (`400 INVALID_INPUT`, clé, montage
    d'`unlock_books`) ; V1-11 (commentaires `bank_accounts.rs:794-795`, `opening_complement.rs:22`, `companies.rs:986-989` ;
    ceux de la 35-5b dans sa fiche) ; V1-12 (35 specs, `:3498`/`:3499`) ; V1-13 (paire 35-3 / 35-5a, 35-6 au manuel,
    routes de la chaîne des appelants : Fichiers communs et C-e350-8) ; V1-14 (16 littéraux d'`AppState`, `HorlogeFixe`
    `pub` dans `kesh-core`, `Horloge: Send + Sync`, File List attendue) ; V1-15 (CHANGELOG ouvert ici, complété par la
    35-5b ; mots-clés de la PR) ; V1-16 (aucune clé i18n ; note sur l'E2E de nuit).
  - Du rapport de la 35-5b, versé ici : V1-7 (écran du verrou, D7, AC4) ; V1-6 (`api-external.md:252`, `:292` restent à la
    35-5b ; la phrase du manuel distingue le poste et Zurich) — **réfuté en partie** : « `user-manual.tex:260` parle de la
    date du poste » ne se lit pas au code (`/usr/bin/grep -n 'du poste' docs/manual/fr/user-manual.tex` : vide ; `:259-261`
    dit « ouvert à la date du jour ») ; la distinction reste écrite, pour qu'elle ne manque pas.
  - Aucune migration. Statut `ready-for-dev` ; la validation P2 est à lancer par l'orchestrateur.
- 2026-11-12 — **Validation P2** (Sonnet, `kesh-gate-logs/e35-5a-validation-p2.md` : 0 CRITICAL, 0 HIGH, 5 MEDIUM, 9 LOW ;
  série P1 → P2 : 7 → 5 MEDIUM) remédiée par l'agent de rédaction (worktree `kesh-e355`). Prémisses lues au code de
  `release/0.17.6`.
  - **V2-1** : câblage du MCP en trois lignes courtes (`horloge_outils`, `lecture_pool`, `lecture_outils`), gardes réécrites sur
    elles, `FABRIQUES_V2` `:2071` et `:2083` nommées, garde neuve de `let horloge_outils = state.horloge.clone();` ; M6 attrapée
    par les gardes seules (D3, AC7). `Lecture::horloge()` ajoutée (squelette de la 35-5b, V2-3 de sa validation P2).
  - **V2-2** : AC3 porte un cas d'acceptation par chemin qui contre-passe (règlement client, facture fournisseur, règlement
    fournisseur, rapprochement), routes lues à `lib.rs:664`, `:670`, `:762`, `:903`.
  - **V2-3** : `fiscal_years_e2e.rs:1064-1072` (`path_b_finalize_creates_fiscal_year`, lu) en 35-5a ; `:1284` reste 35-5b.
  - **V2-4** : chaîne de production complétée (fonctions de vue, `routes/imported_supplier_invoices.rs`,
    `services/factures_fournisseur.rs`) ; second tableau des neuf appelants de tests sans site de jour ;
    `admin_full_import_e2e.rs:1809` en 35-5a pour son appel ; homonyme `invoice_reminders::cancel_in_tx` écarté.
  - **V2-5** : formes bornées `\b…\(` (le faux positif `Utc.from_utc_datetime(`, `csv_tables.rs:98`, lu), fichier de la garde
    exclu nommément avec une sonde, entrée permanente `admin.rs:722`, ordre d'édition de la liste par 35-5a → 35-5b → 35-5c ;
    formes `Utc::now().year(`… et `DATE(UTC_TIMESTAMP` ajoutées, angle mort de la variable d'instant écrit (V1-6 de la
    validation P1 de la 35-5c, réglé ici puisque la garde naît ici).
  - **LOW** : V2-6 (9 littéraux dans 7 fichiers, recomptés par `rate_limiter_send_email:`) ; V2-7 (`state.horloge = …`,
    `common/horloge.rs`, specs E2E nommées) ; V2-8 (section *Dérogation règle de splitting*, C-e350-27) ; V2-9 (la 35-6 ne
    touche plus `api-external.md` ; `opening_balances.rs` par son seul `mod tests` — précision de l'orchestrateur) ; V2-10
    (`invoice_unvalidate_e2e.rs`, `invoice_delete_e2e.rs` en 35-5c : la dévalidation supprime l'écriture) ; V2-11 (AC6 : « la
    production ») ; V2-12 (quatre instants de bord à AC1) ; V2-13 (point d'observation du Vitest, « huit » → « sept ») ; V2-14
    (règle des tests neufs des stories sœurs).
  - Ajout : preuve de nuit par `KESH_TEST_INSTANT` (C-e350-26 ; `faketime` absent de la station), AC9 et M8.
  - Aucun finding réfuté. Signal de découpage : levé (plus de cinq modules) et **non suivi**, motivé par la dérogation
    (C-e350-27) ; les défauts de P2 sont distincts de ceux de P1, aucun recyclage. Aucune migration. Statut `ready-for-dev`.
- 2026-11-12 — **Validation P3** (Opus, `kesh-gate-logs/e35-5a-validation-p3.md` : 0 CRITICAL, 0 HIGH, 4 MEDIUM, 3 LOW ;
  série 7 → 5 → 4 MEDIUM) remédiée par l'agent de rédaction (worktree `kesh-e355`). Prémisses lues au code de `release/0.17.6`.
  - **V3-4 → découpage** (décision de l'orchestrateur, C-e350-28) : la dérogation C-e350-27 tombe, sa prémisse étant fausse ; la
    plomberie part à la **35-5a0** (fiche neuve, comportement inchangé, jour UTC, garde posée) ; ici restent la bascule
    (`jour_suisse`, corps d'`Horloge::aujourd_hui()`, C-e350-29), AC1 à AC5, AC7 au jour suisse, l'écran, les conversions et les
    preuves de nuit. AC6 renvoie à la 35-5a0.
  - **V3-1** : AC9 porte une exécution de **tous** les binaires avec `KESH_TEST_INSTANT="$(date -u +%F)T23:30:00Z"`, lancée de
    jour, chaque rouge rattaché à 35-5b ou 35-5c ; les littéraux d'`AppState` de test prennent `horloge_de_test()` (35-5a0).
  - **V3-2** : la fermeture des appelants (écritures fournisseur par `commit_with_view`, `cancel_settlement`,
    `reconciliation_cancel::*`, routes `supplier_invoices.rs:829-990`, `invoices.rs:1361`, `reconciliation.rs:5938`, `:6587` ;
    158 appels dans 23 fichiers, `git grep` relu) passe à la fiche 35-5a0 ; `from_view` (`:272`) n'est pas un appelant ; le
    conflit de compilation avec la 35-3 est écrit (fiche 35-5a0, epic).
  - **V3-3** : Vitest du verrou à fuseau fixé (`pinTimeZone`, `local-date.test.ts:8` lu) et à l'instant de l'AC4 serveur.
  - **LOW** : V3-5 (durcissement — gardes de compte de `KESH_TEST_INSTANT`, des appels de `new_for_tests`/`horloge_de_test`
    et des constructions de `HorlogeSysteme`, `panic!` sur valeur invalide, ligne de `docs/testing.md` — porté à la 35-5a0 ;
    une feature `test-support` écartée, C-e350-29) ; V3-6 (garde `horloge_outils` dans une liste voisine, non
    `LIB_NOMME_MCP`, `mcp_gardes_lexicales.rs:207-211` lu) ; V3-7 (unité : l'occurrence ; comptes écrits depuis l'exécution).
  - Helper de test renommé `aujourd_hui_test()` (il rend le jour UTC dans la 35-5a0, suisse après la bascule : le nom
    `aujourd_hui_suisse` aurait menti entre les deux). `reconciliation.rs:237` : **#1053**.
  - Aucun finding réfuté. Signal de découpage : suivi (C-e350-28). Aucune migration. Statut `ready-for-dev`.
- 2026-11-12 — **Validation P4** (Opus, `kesh-gate-logs/e35-5a-validation-p4.md` : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 8 LOW ; série
  7 → 5 → 4 → 2 MEDIUM) remédiée par l'agent de rédaction (worktree `kesh-e355`). Prémisses lues au code de `release/0.17.6`.
  - **V4-1** : le manuel dit déjà « en temps universel » à `user-manual.tex:995` et `:2458` (`/usr/bin/grep -n 'temps universel'`,
    lu) ; § *Ce qui se lit* corrigé, D6 réécrit ces deux phrases, AC11 contrôle `.tex` et PDF aplati, union avec la 35-6 au
    `:2456` (contigu).
  - **V4-2** : binaires que `KESH_TEST_INSTANT` n'atteint pas nommés (`lib` de chaque crate) et prouvés par la valeur ;
    `kesh-report/tests` reçoit son `support/jour.rs` (porté à la 35-5a0) ; règle et inventaire des `HorlogeFixe(…Utc::now…)` (D5).
  - **LOW** : V4-3 (test d'AC2 de la 35-5a0 retourné, exception d'AC10, tâche ; AC6 → D2 et AC5 de la 35-5a0) ; V4-4 (fonction
    privée `jour()` posée par la 35-5a0, remplacée par `jour_suisse` : compte 1 à chaque étape ; tests d'AC1 sans `.date_naive()`) ;
    V4-5 (borne d'`unlock_books` posée en SQL ; cible égale = retrait sans effet, accepté, `companies.rs:1099-1100`, `:1128-1131`
    lus) ; V4-6 (définitions et appels distingués, ici et à la 35-5a0) ; V4-7 (fichiers mixtes) ; V4-8 (`companies.rs:986-989`
    réécrit par la 35-5a0) ; V4-9 (`vi.useFakeTimers({ toFake: ['Date'] })`, `useRealTimers` en `afterEach`) ; V4-10 (montages
    par année du jour, tous 35-5a ou 35-5b : AC9).
  - Aucun finding réfuté. Signal de découpage : non levé (MEDIUM neufs, aucun recyclage). Aucune migration. Statut
    `ready-for-dev`.
- 2026-11-12 — **Validation P5, passe ciblée** (Sonnet, sur la remédiation `330a5780` et `9702ea6d` ;
  `kesh-gate-logs/e35-5a-validation-p5-ciblee.md` : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 3 LOW), LOW appliqués par l'agent de
  rédaction (worktree `kesh-e355`) ; prémisses lues au code de `release/0.17.6`.
  - **V5-1** : AC11 contrôle aussi `encore la veille` (`.tex` et PDF aplati) — `user-manual.tex:995` et `:2458`, dans la même
    phrase que « en temps universel ».
  - **V5-2** : `kesh-db/src/repositories/journal_entries.rs:4457` (`chrono::Utc::now().naive_utc().date().year()`, `mod tests` de
    `src`) ajouté au relevé V4-10 d'AC9.
  - **V5-3** : le montage d'AC2 est nommé dans la 35-5a0 (`seed_accounting_company`, exercice 2020-2030 ouvert,
    `test_fixtures.rs:71`, `:119`) ; AC10 et la tâche de retournement disent « l'exercice 2020-2030 de son montage » au lieu
    de « l'exercice 2026 ».
  - Hors remédiation : le doc-comment `services/rapports.rs:862-864` (« comme à la contre-passation ») devient faux à la 35-5a ;
    D6 en retire cette clause, la 35-5b réécrit le reste avec le site 21 ; *File List*.
  - **Synthèse de la validation** (`CLAUDE.md` § *Review Iteration Rule*) : cinq passes, **7 → 5 → 4 → 2 → 0 MEDIUM** au-dessus
    de LOW (aucun CRITICAL ; P1 à P4 complètes, P5 ciblée sur la dernière remédiation, qui ne touche aucun code de
    production : la fiche seule). Modèles : P1 Opus, P2 Sonnet, P3 Opus, P4 Opus, P5 Sonnet (ciblée). Décisions de
    reclassement : aucune ; aucun finding réfuté sur la série. Découpage : la dérogation C-e350-27 est tombée en P3 au profit
    du découpage **C-e350-28** (35-5a0 pour la plomberie, comportement inchangé ; la 35-5a pour la bascule au jour suisse).
    **Validation terminée** sur une passe qui ne rend aucun finding au-dessus de LOW. Aucune migration. Statut
    `ready-for-dev`, après la fusion de la 35-5a0.
- 2026-11-13 — **Développement** (Opus 5.5, worktree `kesh-e355a`, base `eabafed5f` = 35-5a0 close, qui porte aussi la 35-3).
  Journaux : `/home/gcorbaz/devel/kesh-gate-logs/e355a/`.
  - **Étape 0.** LOW de la validation P5 : V5-1 (AC11 contrôle aussi « encore la veille ») tenu au contrôle final ; V5-2
    (`journal_entries.rs:4457`, année du jour dans le `mod tests`) : ligne lue, l'année n'est comparée à aucun jour du serveur
    dans ce module (aucune horloge) ; V5-3 : le test retourné est `horloge_routes_e2e.rs` (montage `seed_accounting_company`,
    2020-2030). `chrono-tz` : `/usr/bin/grep -n -A4 '^name = "chrono-tz"' Cargo.lock` → `0.10.4` ; `cargo tree -i chrono-tz -e
    features` → par `rxing` 0.7.1, `default` ; ajouté à `kesh-core` à la même version, `Cargo.lock` gagne une ligne (l'arête).
    **Transmis par la 35-5a0** : `configuration_transmise.rs` compte `NOM_VARIABLE_INSTANT` 2 — inchangé, la garde reste verte ;
    `horloge_de_test` par `env_nonempty` — inchangé ; **E3** (lecture unique avant l'attente d'un verrou) : pas au manuel,
    C-e355a-4 ; montages et horloge — `homepage_summary_e2e.rs` reste 35-5b (son `today()` est le site 21) ; Lecture du MCP tenue
    lexicalement, AC7 en ajoute la preuve par `state.lecture().aujourd_hui()` ; #1105 non traitée ici ; **F3** (instant à plus
    de trois jours, minuit, bascule d'exercice) : joué, voir l'inventaire. Inventaire D5 :
    `git grep -nE 'HorlogeFixe\(\s*(chrono::)?Utc::now' -- 'crates/*/src/*' 'crates/*/tests/*'` → `middleware/auth.rs:302`
    (état des tests du middleware sur `stub_pool`, ne date rien) et deux chaînes de `date_du_jour_garde.rs` (:610 commentaire,
    :802 sonde). Le `mod tests` de `journal_entries.rs` : ses deux `reverse_in_tx` reçoivent la date de leur montage
    (`:3991` `jour`, `:5845` `today`) — sûrs la nuit par la valeur, aucun à convertir ; il reste à 50 à la garde, classé 35-5c.
  - **Rouges** (`rouges.log`), sur le squelette `076d63ed3` (`jour_suisse` au corps UTC, compte de la garde de `horloge.rs` à 1) :
    AC1 3/8 (M1 compris) ; HTTP 18/26 — AC2, les cinq chemins et le refus d'AC3, AC4 ×2, AC5, AC7 (lecture), le test retourné ;
    six rouges de la garde, une seule cause (l'année du chemin B convertie, exemption ramenée de 2 à 1) ; Vitest 3/62 (M9).
    Le premier montage d'AC7 était invalide (exercice 2026 clos d'avance : `LATER_FISCAL_YEAR_CLOSED`), le second rendait le rang
    2 bis quel que soit le jour ; le troisième (sans exercice 2026) rougit pour la bonne raison (`rouges-api-3.log`) —
    C-e355a-3, **écart à la lettre d'AC7** (« exercice clos » → « aucun exercice ouvert au jour suisse »).
  - **Bascule** (`c2e3e788c`) : `jour_suisse` = `instant.with_timezone(&chrono_tz::Europe::Zurich).date_naive()` ; AC1 8/8 ;
    tests ciblés 66/67 puis verts après alignement du refus d'AC3 sur `400 FISCAL_YEAR_INVALID` (le refus du rang 5 au clic,
    `api-external.md:572`, au lieu du texte du motif de la liste). Écran (D7) : veille de `todayLocal` sur ses champs ; l'entrée
    d'allowlist du verrou retirée (huit → sept), et les sondes de la garde frontend, qui visaient `ALLOWLIST[3]` par position,
    désignent l'entrée par son fichier. Vitest ciblé 69/69 (`verts-vitest.log`).
  - **Inventaire par exécution (AC9)** — synthèse et tri ligne par ligne : `kesh-gate-logs/e355a/nuit.log`. **Sonde positive** :
    sur `c2e3e788c`, la nuit réelle reproduite (`KESH_TEST_INSTANT=2026-11-13T23:30:00Z`) rend 2 rouges que la fiche n'avait
    pas nommés — `bank_account_opening_balance_e2e.rs:731`, `opening_balances_e2e.rs:1472` (borne posée au jour UTC, jugée au
    jour suisse) —, absents sans instant (N0, 5357/5357) : la méthode voit. Le passage d'année (2025-12-31T23:30Z) rend en outre
    4 rouges de `period_lock_e2e` (pièces à `aujourd_hui - 5..10` hors de l'exercice civil du montage) : défaut de passage
    d'année d'un fichier 35-5a, corrigé (`c5972ca13`). Les **huit fichiers classés 35-5a** sont convertis à
    `aujourd_hui_test()` (`ada7e1d61`) et sortent de la garde. **Campagne sur `c5972ca13`** (11 passages, base remise à neuf
    avant chacun) : référence, nuit réelle, minuit ±1 s, −3 jours, −45 jours → **0 rouge sur 5357** (hors un intermittent MCP
    sous charge à 22:59:59Z, sans lecture de date, rejoué isolé) ; instants de la fiche (2025-12-31, 2026-03-10) → 5 rouges
    communs, les mêmes qu'une sonde **de jour** au 11 mars 2026 : montages à **dates fixes de 2026** que l'instant précède,
    artefact de l'instant (C-e355a-7, **écart à la lettre d'AC9**, déclaré) ; passage d'année 2026→2027 et sonde de jour au
    15 janvier 2027 → 9 puis 13 rouges : montages à **exercice fixe 2026**, qui rougiront chaque jour de 2027 (issue demandée).
    Le verdict des fichiers « à lire » (`credit_card_e2e`, `reconciliation_link_e2e`, second tableau) : **35-5c**, établi par
    l'exécution (aucun rouge la nuit). **Angles morts** écrits au § 4 de `nuit.log` (binaire servi, SQL de montage, ignorés,
    au-delà de −45 jours et du 15 janvier 2027).
  - **Specs E2E** : aucune n'asserte une date que pose un site de la 35-5a (C-e355a-6, lignes lues) ; liste vide, transmise à
    la 35-5c.
  - **La garde** (AC8), recomptée depuis la source : 39 → **31** entrées (huit retirées, `fiscal_years_e2e.rs` 2 → 1, l'entrée
    de `journal_entries.rs` requalifiée 35-5c) ; verte à l'exécution (dans chaque passage de la campagne). Sondes : positive et
    multi-ligne (`sonde_une_forme_ajoutee_rougit_entiere_ou_coupee`), compte + 1 (`sonde_une_exemption_portee_a_plus_un_rougit`),
    vertes ; M5 (une exemption sans site) la fait rougir.
  - **Mutations** (`mut/`, restauration `git checkout --` puis `touch`) : M1 (jour UTC) 16 rouges ; M2 (fuseau fixe +1 h) 1 rouge,
    le cas d'été `2026-07-01T22:30:00Z` ; M3 (contre-passation lue sur `Utc::now()`) 8 ; M4 (`lock_books` au jour UTC) 7, dont
    `ac4_verrouiller_jusqu_a_hier_la_nuit` ; M5 5 (garde) ; M7 (contre-passation de l'annulation sur l'horloge réelle) 1,
    `ac3_annulation_d_un_reglement_client_la_nuit` ; M9 (écran, veille UTC) 3. **Toutes tuées.** M6 et M8 : 35-5a0.
  - **Docs** (D6, AC11) : `api-external.md:272` « du jour, à Zurich » ; commentaires `bank_accounts.rs:794-795`,
    `opening_complement.rs:22`, clause « comme à la contre-passation » retirée de `rapports.rs:862-864` ; `docs/testing.md:119` ;
    manuel : § *La date du jour est celle de Zurich, même la nuit* (`user-manual.tex:737`, écran du poste et serveur distingués),
    renvois depuis le verrou (`:711`), le complément (`:1013`) et le solde d'ouverture (`:2488`). Contrôle :
    `/usr/bin/grep -nE 'temps universel|encore la veille' docs/manual/fr/user-manual.tex` → vide ; PDF régénéré
    (`make user`, 3 passes, `manuel-make.log`), aplati (`pdftotext … | tr '\n' ' ' | tr -s ' '`), les deux motifs → vides.
    `/usr/bin/grep -n UTC` sur les fichiers de production touchés : ne restent que les mentions des sites 35-5b
    (`bank_accounts.rs:228`, `:1757` ; `rapports.rs:724`, `:863`) et, dans `horloge.rs`, l'UTC comme référence décrite. CHANGELOG :
    trois points *Corrigé* (#1022, #907, #929 en partie). Aucune clé i18n.
  - **Captures** (thème clair, poste à Zurich, horloge du navigateur fixée ; `captures/`, `captures.log`) : borne `max` = 10 mars
    à 00 h 30 le 11 mars, 31 décembre à 00 h 30 le 1er janvier, la veille le jour ; chaque état avant et après saisie de la borne.
    Le serveur servi ne lit pas `KESH_TEST_INSTANT` : l'acceptation nocturne côté serveur ne se capture pas, elle est prouvée par
    AC4.
  - **Écarts** : consigne Edit — `prettier --write` lancé une fois sur `local-date-guard.test.ts` (configuration par défaut,
    322 lignes réécrites), aussitôt défait par `git checkout --` et les quatre éditions refaites à l'outil Edit ; `cargo fmt`
    pour les retours de ligne (C-e355a0-2). Scripts Python seulement pour les mutations (`mut/mut-e355a.py`) et pour corriger
    quatre numéros de ligne du relevé hors dépôt `nuit.log`.
  - **Gates du passage en `review`**, au dernier commit de code `c5972ca13`, bases remises à neuf (consignes
    `e35-dev-commun.md`) : `scripts/test-fast.sh --ci` (fmt et clippy du workspace compris) **5357 exécutés, 5357 passés, 13
    ignorés** (`gate-test-fast.log`) ; `scripts/gate-ci.sh kesh_ci_e355a` **5364 passés, 0 échoué, 17 ignorés** (`gate-ci.log`,
    `gate-ci.journal`) ; frontend complet — `npm run check` 0 erreur (29 avertissements, aucun dans un fichier de la story),
    `lint-i18n-ownership` vert, `test:unit` **3192 passés, 232 fichiers**, `build` vert (`gate-front-*.log`). **E2E : non
    lancé**, à la clôture (avec `gate-ci`, à la demande de l'orchestrateur). Hors story : `mcp_transport_e2e::
    abandon_au_delai_le_permis_reste_pris` rougit **isolé** 3 fois sur 3 (`mcp-isole-{1,2,3}.log`) et passe dans la suite
    complète ; la story ne touche ni le test ni `src/mcp` (diff vide) — issue demandée. Statut : `review`.
- 2026-11-13 — **Revue de code P1** (Opus, trois lentilles sur `eabafed5f..c5972ca13` ; `kesh-gate-logs/e355a-review-p1-{B,E,A}.md` :
  B 1 MEDIUM, 2 LOW ; E 1 MEDIUM, 5 LOW ; A 1 MEDIUM, 6 LOW ; **2 MEDIUM distincts** — B1 = A1, E1 —, vérifiés au code par
  l'orchestrateur) **remédiée** par l'agent de développement, un seul commit de code `527f57839`. Déjà tracés hors story :
  #1125 (tests à dates fixes de 2026), #1126 (test MCP qui rougit isolé).
  - **E1 (MEDIUM)** — l'écran du verrou prenait la veille du jour **du poste** (`settings/+page.svelte:567-570`) quand le serveur
    juge au jour de Zurich (`companies.rs:1015`, `if through >= aujourd_hui`) : sur un poste à l'est de Zurich, la borne
    proposée était refusée. Cherché d'abord un helper existant (`/usr/bin/grep -rn "Europe/Zurich" src` hors tests : seul
    `reports.api.ts:203`, privé, qui formate un horodatage) ; écrit `todayInZurich` (`Intl.DateTimeFormat`, `timeZone:
    'Europe/Zurich'`) et `dayBefore` (calendrier, sans fuseau) dans `local-date.ts` ; `maxLockDate = dayBefore(todayInZurich())`.
    **Rouges d'abord** (`p1-rouges-vitest.log`, `c5972ca13` + tests neufs) : 2 sur 65 — poste à New York (`2026-03-10T23:30:00Z`,
    attendu `2026-03-10`, reçu `2026-03-09`) et à Athènes (`2026-03-10T22:30:00Z`, attendu `2026-03-09`, reçu `2026-03-10`, borne
    que le serveur aurait refusée). Après : 96/96 sur les trois fichiers ciblés (`p1-verts-vitest-2.log`). Tests ajoutés aussi
    pour `todayInZurich` (quatre instants × quatre fuseaux de poste) et `dayBefore` (six dates, dont le 29 février). Mutation
    **M10** (le jour du poste remis, par l'import `todayLocal as todayInZurich`) : **tuée**, 2 rouges (`mut/M10.log`). Manuel
    `:711-713` et note `sec:date-du-jour`, CHANGELOG (#907) : « la veille du jour de Zurich, quel que soit le fuseau de votre poste ».
  - **B1 = A1 (MEDIUM)** — option prudente de l'orchestrateur : **le seed n'est pas converti ici** (site 26, 35-5b). Les textes
    deviennent vrais : manuel « le premier exercice que l'assistant crée pour votre société », CHANGELOG idem (titre et corps
    de #929), et l'exercice de la société de démonstration nommé parmi ce qui reste au jour UTC.
  - **LOW traités** : **B2/A2/E5** — la liste de ce qui reste en UTC (manuel, CHANGELOG) dit « notamment » et nomme la
    démonstration, la date de clôture d'un compte bancaire quand elle n'est pas donnée (`bank_accounts.rs:1802`), la balance
    âgée (`rapports.rs:732`) et la fin d'un rapport de projet cumulé (`rapports.rs:780`), sans rien convertir. **B3/E6** —
    `rapports.rs:862-865` dit le jour UTC (`Utc::now()`, 35-5b) « non celui de l'horloge de l'état » ; `kesh-db/tests/support/jour.rs:6`
    « le jour de Zurich depuis la Story 35-5a » (`git grep -nE "UTC aujourd"` → vide ensuite). **E2** — AC1 gagne le premier
    minuit d'heure d'hiver à la seconde (`2026-10-25T22:59:59Z` → 25, `23:00:00Z` → 26) ; la seconde du changement d'heure
    (01:00:00Z) est dite sans effet sur le jour (commentaire). **E3** — `ac4_verrouiller_jusqu_a_hier_une_nuit_d_ete`
    (`2026-07-01T22:30:00Z`, CEST : jusqu'au 1er juillet accepté, au 2 refusé), par un helper commun avec le cas d'hiver, et un
    Vitest d'été de nuit ; M2 (fuseau fixe +1 h), rejouée sur `527f57839`, est désormais tuée **aussi côté serveur** par ce cas
    (`mut/M2-p1.log` : 2 rouges, `le_jour_est_celui_de_zurich` et `ac4_verrouiller_jusqu_a_hier_une_nuit_d_ete`) ; M1 rejouée,
    5 rouges (`mut/M1-p1.log`). **E4** — `Europe::Paris` (Berlin, Vienne, Rome) à la place de `Zurich` est une mutation
    **équivalente** pour tout instant postérieur à 1981 : dit au doc-comment de `jour_suisse` et ici. **A3** — cases des Tâches
    cochées (celle des gates laissée ouverte et annotée : E2E et `gate-ci` à la clôture) ; recompte de l'inventaire de
    production : `git grep -nE 'Utc::now\(\)\.(date_naive|naive_utc\(\)\.date)\(\)|UTC_DATE|CURDATE|Local::now' <rev> --
    'crates/*/src/*' | wc -l` → **82** à `eabafed5f` comme à `527f57839` (92 de la fiche − 9 sites convertis par la 35-5a0 − 1
    commentaire réécrit ; la 35-5a ne touche aucun de ces sites, elle change le corps de `jour_suisse`) ; V5-2, ligne lue :
    `journal_entries.rs:4477` (`:4457` sur `release/0.17.6`, décalée par la 35-5a0) `let year = chrono::Utc::now().naive_utc().date().year();`.
    **A5** — C-e355a-7 annoté : pour les deux tests de fichiers 35-5a qui rougissent sous les instants de la fiche, la nuit
    d'hiver est prouvée par C1 et le passage d'année par C2b, verts. **A6** — exceptions d'**AC10** déclarées : outre le test de
    `horloge_routes_e2e.rs`, deux tests de `kesh-core/src/horloge.rs` (35-5a0) suivent la bascule (motif D1) —
    `le_jour_est_le_jour_utc` remplacé par `le_jour_est_celui_de_zurich` et `l_horloge_fixe_rend_le_jour_suisse`, et l'attendu de
    `instant_de_test_rogne_et_ramene_un_decalage_en_utc` passé de `2025-12-31` à `2026-01-01`. **A7** — les quatre journaux sans
    en-tête (`rouges-kesh-core.log`, `rouges-vitest.log`, `verts-vitest.log`, `mut/M9.log`) annotés de leur arbre ; le Dev
    Agent Record annoté (aucun journal pour « squelette (vert) » ni pour les tests ciblés verts après l'alignement d'AC3 ; aucun
    gate complet sur `076d63ed3` ni `ada7e1d61`).
  - **LOW non traités** (issues demandées) : **A4** (rouges et mutations relevés à un seul site des tests multi-sites : M3 sur
    `journal_entries.rs` seulement, M7 sur `invoice_settlements_write.rs` seulement ; la couverture des autres sites d'AC2 et
    d'AC3 par ces mutations est un **raisonnement**, non une exécution).
  - **Écart** : la restauration de M10 par `git checkout --` a aussi défait la correction de `+page.svelte`, **non commitée** ;
    constaté au `git diff --stat` suivant, réappliquée à l'outil Edit, Vitest rejoué vert (`p1-verts-vitest-2.log`). M10 avait
    bien été jouée sur la version corrigée (son motif est l'import corrigé). Ensuite, le code a été commité **avant** les
    mutations Rust (M1-p1, M2-p1), que `git checkout` restaure au commit. Captures non reprises : sur un poste réglé sur la
    Suisse, l'écran rend la même borne qu'avant la remédiation.
  - **Manuel** : PDF régénéré (`p1-manuel-make.log`, 2 passes) ; aplati (`p1-pdf-aplati.log`) : « temps universel » / « encore
    la veille » → 0 ; les trois phrases neuves présentes ; « premier exercice créé par l'assistant » → 0.
  - **Gates de la remédiation** (consignes `e35-dev-commun.md` : `test-fast` et frontend complet, bases remises à neuf ; ni
    `gate-ci` ni E2E), au commit de code `527f57839` : `scripts/test-fast.sh --ci` **5358 exécutés, 5358 passés, 13 ignorés**
    (`p1-gate-test-fast.log` ; +1 : la nuit d'été d'AC4) ; frontend — `check` 0 erreur (29 avertissements, aucun dans un fichier
    de la story), `lint-i18n-ownership` vert, `test:unit` **3217 passés, 232 fichiers**, `build` vert (`p1-gate-front-*.log`).
    **`gate-ci` et E2E : à la clôture.** Statut : `review` ; passe suivante à lancer sur `527f57839`.

### 2026-11-13 — Clôture — synthèse finale

- **Trend de la revue de code** (`CLAUDE.md` § *Review Iteration Rule*) : P1 (Opus, trois lentilles B/E/A, sur
  `eabafed5f..c5972ca13`) **2 MEDIUM distincts** (B1 = A1 : textes de l'assistant ; E1 : borne de l'écran au jour du poste) et
  12 LOW → remédiation `527f57839` (un commit de code) → P2 **ciblée** (Sonnet, une lentille, sur `527f57839` ; prompt
  `4642b082d`) **0 au-dessus de LOW, 3 LOW** : C1 et C2 tracés (#1128, #1129), C3 (doc) appliqué ici — D7 annotée de son
  remplacement par la veille du jour de Zurich. Boucle close sur une passe sans finding au-dessus de LOW, après une
  remédiation qui ne touche aucun code (C3). Rapports hors dépôt : `kesh-gate-logs/e355a-review-p1-{B,E,A}.md` et le rapport
  de la P2 ciblée tenu par l'orchestrateur. Aucun reclassement, aucun finding réfuté. Signal de découpage : non levé.
- **Issues** tracées par l'orchestrateur : #1125 (tests à dates ou exercices fixes de 2026, qui rougiront dès 2027), #1126 (test
  MCP `abandon_au_delai_le_permis_reste_pris` rouge isolé), #1127 (A4 : rouges et mutations relevés à un seul site des tests
  multi-sites), #1128 (C1 de la P2 : un commentaire de `local-date-guard.test.ts` décrit encore la borne comme la veille de
  `todayLocal`) et #1129 (C2 : la borne `max` de l'écran est figée à l'ouverture de la page).
- **Gates de clôture** au dernier commit de code `527f57839` (HEAD `4642b082d`, sans différence de code :
  `git diff --stat 527f57839 HEAD -- crates frontend` vide), règle de l'Epic 35 : `scripts/gate-ci.sh kesh_ci_e355a` **5365
  passés, 0 échoué, 17 ignorés** (`kesh-gate-logs/e355a/cloture-gate-ci.log`, `cloture-gate-ci.journal`) ; **E2E complet**
  sur `kesh_e2e_e355a` recréée à neuf, migrée et semée, backend `target/debug/kesh-api` et frontend rebâtis au commit
  (`cloture-build.log`), port 3410, montage de `docs/testing.md` (`cloture-e355a.sh`) : **403 passés, 7 échecs, 18 ignorés**
  (`cloture-e2e.log`, 18,5 min). Les sept échecs sont, un par un, ceux de la **KF-029 (#97)** au tableau de `docs/testing.md`
  § « Les échecs attendus » : `mode-expert.spec.ts:26`, `:41` ; `onboarding-path-b.spec.ts:65`, `:92` ; `onboarding.spec.ts:57`,
  `:77`, `:150`. Aucun échec changeant, aucun imputable à la story. Backend arrêté par son PID (737979), port libre avant et
  après.
- **Récapitulatif des gates de la story** : passage en `review` (`c5972ca13`) — `test-fast` 5357/5357, `gate-ci` 5364/0/17,
  Vitest 3192 ; remédiation P1 (`527f57839`) — `test-fast` 5358/5358, Vitest 3217 ; clôture (`527f57839`) — `gate-ci` 5365/0/17,
  E2E 403/7 KF-029/18. Inventaire de nuit : `nuit.log` (nuit réelle, minuit ±1 s, −3 et −45 jours : 0 rouge sur 5357).
  Mutations : M1, M2 (aussi côté serveur), M3, M4, M5, M7, M9, M10 tuées. **Aucune migration.**
- Statut : **`done`**.
