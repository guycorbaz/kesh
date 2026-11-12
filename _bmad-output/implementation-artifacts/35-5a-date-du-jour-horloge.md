# Story 35-5a : la date du jour est celle de Zurich pour ce qui écrit dans les livres ou les borne — solde d'ouverture, contre-passation, annulation de règlement, verrou de période, premier exercice de l'assistant — par une horloge injectable (#1022, #907, #929)

Status: ready-for-dev

*Fiche rédigée le 2026-11-12 par l'agent de kickoff de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, par le workflow
`bmad-create-story` ; remédiée après la validation P1 le même jour (worktree `kesh-e355`, Change Log). Née du **découpage
préventif** de la 35-5 (C-e350-14) : la 35-5a pose le motif — horloge, fonction du jour suisse, horloge dans `AppState` **et
dans `Lecture`** (C-e350-21), garde de l'inventaire — et convertit les sites qui écrivent dans les livres ou les bornent ;
la **35-5b** convertit les sites restants de la production, la **35-5c** les tests restants (C-e350-22). Release **0.17.7**.
**Aucune migration** (C-e350-16). Boucle complète : règle comptable, `kesh-db` touché (gate complet même en cours de
boucle). Code lu sur **`release/0.17.6`** (`4e1c768a` ; vérifié identique à `5e2a5818` pour `crates/` et `frontend/`).
Registres : `e35-choix-autonomes-e350.md` (C-e350-8, -14, -16, -21 à -25) ; `e35-choix-autonomes-e355a.md` pour la story,
à créer au premier choix.*

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
ceux qui deviennent faux se réécrivent (D6) : `companies.rs:986-989` ici, les autres à la 35-5b. `aged_receivables.rs:27`
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
Les signatures de `cancel_in_tx`, `cancel_settlement_in_tx` et des bloqueurs changent dans `supplier_invoices.rs` (`:2462`,
`:2600`, `:2750`, `:2871`) et `invoice_settlements_write.rs` (`:878`, `:1023`) ; la chaîne remonte à `routes/invoices.rs`,
`routes/supplier_invoices.rs`, `routes/reconciliation.rs`. Le compilateur donne la liste close.
`reverse_at_next_fiscal_year_start*` ne lit pas le jour.

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
dans son corps ; mais `AppState` se construit aussi **par littéral** en **16 endroits, dans 9 fichiers** (`git grep -n 'AppState {'` :
`lib.rs`, `main.rs`, `middleware/auth.rs`, `tests/common/mcp.rs`, `invoice_frozen_pdf_e2e.rs`, `invoice_send_email_e2e.rs`,
`password_recovery_e2e.rs`, `services_motif.rs`, `setup_admin_e2e.rs`), que `new_for_tests` ne couvre pas.

**Les docs qui disent UTC ou « du jour »** : `docs/api-external.md:272` (« sinon du jour **UTC** »), 35-5a ; `:252`
(`closedOn`), `:292` (accueil), `:443` (postes ouverts), 35-5b. Le manuel utilisateur parle de « la date du jour » sans dire
le fuseau (`user-manual.tex:259-261`, `:355`, `:709`, `:722`, `:994`, `:1449-1482`, `:1900`, `:2011`, `:2085`) ; il ne dit
nulle part « date du poste » (`/usr/bin/grep -n 'du poste' docs/manual/fr/user-manual.tex` : vide).

## Décisions (C-e350-16, C-e350-21 à -25)

- **D1 — Europe/Zurich, constante unique**, dans `kesh-core` : `pub fn jour_suisse(instant: DateTime<Utc>) -> NaiveDate`,
  pure, écrite `instant.with_timezone(&chrono_tz::Europe::Zurich).date_naive()` — jamais ambiguë dans ce sens (un instant
  UTC a une seule heure locale), contrairement à `from_local_datetime`. `chrono-tz` 0.10.4 est déjà au `Cargo.lock` (par
  `rxing` 0.7.1, features par défaut, base IANA 2025b précompilée : ni `/usr/share/zoneinfo` ni `TZ` dans l'image) :
  ajouté aux dépendances de `kesh-core` à la même version, features par défaut, sans nouveau paquet (constater par
  `cargo tree -p kesh-core -e features` à l'étape 0). Pas de `TZ`, pas de `chrono::Local`.
- **D2 — `kesh-db` ne lit plus l'horloge** pour une date : `reverse_in_tx`, `reverse_owned_in_tx`, `reverse`,
  `settlement_entry_cancel_blocker`, `settlement_cancel_blocker`, `lock_books`, `unlock_books` (et leurs intermédiaires,
  dont `cancel_in_tx`, `cancel_settlement_in_tx` et les lectures qui calculent le bloqueur) reçoivent `aujourd_hui:
  NaiveDate`, ajouté **en dernier argument**. `ReversalDating::Today` porte la date ou la reçoit. **Une lecture de l'horloge
  par requête**, hors de toute boucle de rejeu : la valeur lue est passée **à la fois** au bloqueur (rang 5,
  `settlement_cancellation.rs:172`) et à la contre-passation (`journal_entries.rs:2935`) — aujourd'hui deux lectures, qui
  peuvent différer à minuit (le refus nommé du rang 5 se tait, `FiscalYearInvalid` sort plus loin).
- **D3 — L'horloge vit dans `AppState` et dans `Lecture`** (C-e350-21) :
  - **`kesh-core`** : trait `pub trait Horloge: Send + Sync { fn maintenant(&self) -> DateTime<Utc>; fn aujourd_hui(&self)
    -> NaiveDate { jour_suisse(self.maintenant()) } }` et **`HorlogeFixe(pub DateTime<Utc>)`**, `pub` (non `#[cfg(test)]`)
    pour les tests d'intégration de toutes les crates ; aucune I/O.
  - **`kesh-api`** : `HorlogeSysteme` (le seul `Utc::now()` qui sert une date), champ `pub horloge: Arc<dyn Horloge>` de
    `AppState` — `HorlogeSysteme` dans `main.rs` et dans `new_for_tests` (défaut dans le corps) ; les **16 littéraux**
    ci-dessus reçoivent le champ (un test qui date quelque chose y met `HorlogeFixe`).
  - **`Lecture` porte l'horloge** : `pub struct Lecture { pool, horloge: Arc<dyn Horloge> }`, `Lecture::new(pool, horloge)`
    **sans valeur par défaut** (le compilateur donne la liste close des constructions), `pub fn aujourd_hui(&self) ->
    NaiveDate`. `AppState::lecture()` passe `self.horloge.clone()` ; le câblage du MCP, `state.horloge.clone()` — la ligne
    figée devient `mcp::LectureOutils::nouvelle(services::poignees::Lecture::new(pool_outils.clone(), state.horloge.clone()));`,
    reportée aux deux gardes `mcp_gardes_lexicales.rs:169`, `:575` et à `FABRIQUES_V2` (`services_motif.rs:2071`, `:2083`).
    Ainsi les routes lisent `state.horloge`, les services `lecture.aujourd_hui()`, et le MCP — qui n'a que sa `Lecture` —
    reçoit la même horloge que l'API sans champ neuf dans `EtatMcp` ni `ContexteMcp`.
  - Les **fonctions pures** (conversion en réponse, analyse d'un paramètre) et `kesh-db` reçoivent une `NaiveDate`.
  - `services::factures::reglements` (`services/factures.rs:534-558`) lit `lecture.aujourd_hui()` **une fois** et la passe à
    `settlement_cancel_blocker` pour chaque règlement.
  - Les **43** constructions de tests passent une horloge : `HorlogeSysteme` (ou `HorlogeFixe` si elles datent quelque
    chose). C'est un changement mécanique, dans le même commit que la signature.
- **D4 — La garde de l'inventaire** (C-e350-24) : un test, `crates/kesh-api/tests/date_du_jour_garde.rs` (sur le modèle de
  `local-date-guard.test.ts`), parcourt **`crates/*/src/**/*.rs` et `crates/*/tests/**/*.rs`** :
  - **Texte lu** : lignes dont le début, espaces retirés, est `//` (`//`, `///`, `//!`) **écartées** ; puis tout blanc (saut de
    ligne compris) entre `)` et `.`, et autour de `.` dans une chaîne de méthodes, réduit — si bien qu'une forme coupée sur
    plusieurs lignes est vue (les 5 sites ci-dessus). Un commentaire **de fin de ligne** reste compté : s'il en paraît un, il
    se réécrit.
  - **Formes relevées**, sur **tout récepteur** : `.date_naive()`, `.naive_utc().date()`, `.naive_local()`, `Utc::today`,
    `Local::` (dont `Local::now`, `Local::today`), et en SQL `UTC_DATE`, `CURDATE`, `CURRENT_DATE`, `DATE(NOW` (casse
    ignorée). Aujourd'hui, `.date_naive()` et `naive_utc().date()` n'ont dans `src` aucun récepteur autre que `Utc::now()` :
    la forme large ne coûte rien. Elle inventorie les sites non résolus, non les formes qui marchent (`CLAUDE.md`).
  - **Liste close d'exemptions, par fichier, en égalité stricte (`==`)** : chaque entrée `{ fichier, nombre, story, motif }`.
    Un fichier absent de la liste doit compter **0**. Un compte qui diffère du nombre écrit, **dans un sens ou dans l'autre**,
    rougit : la liste se ramène au compte réel à chaque conversion, comme un cliquet. Entrée permanente : le fichier de
    `jour_suisse` dans `kesh-core` (1, son `.date_naive()`).
  - **Après la 35-5a**, la liste porte : les 19 sites de production de la 35-5b et ses commentaires restants ; les modules de
    tests de `src` non convertis (`journal_entries.rs` : 50 moins ceux que la 35-5a convertit ; `invoices.rs` : 6, 35-5b) ;
    les fichiers de `crates/*/tests` laissés à la 35-5b ou à la 35-5c (verdict ci-dessous), à leur compte exact. Les fichiers
    convertis par la 35-5a n'y figurent pas (compte 0).
  - Sondes au Change Log : positive (un `Utc::now().date_naive()` ajouté à un fichier non exempté rougit ; la même forme
    coupée sur deux lignes aussi) et de compte (une exemption portée à `nombre + 1` rougit).
- **D5 — Les tests ne dépendent plus de l'heure réelle** pour les sites convertis : un test de `kesh-db` passe la date qu'il
  asserte ; un test HTTP qui asserte une date fixe l'horloge de son `AppState` (`HorlogeFixe`). Un montage qui veut « un
  exercice qui couvre aujourd'hui » prend le jour suisse par **un helper partagé par crate de tests**, posé dès cette story :
  `crates/kesh-api/tests/common/jour.rs` et `crates/kesh-db/tests/support/jour.rs`, `pub fn aujourd_hui_suisse() ->
  NaiveDate { kesh_core::jour_suisse(Utc::now()) }` (la forme `jour_suisse(Utc::now())` n'est pas relevée par la garde ;
  `kesh-report/tests` l'écrit en ligne, un seul site). La nuit se teste par instants fixés.
- **D6 — Docs et commentaires** : `api-external.md:272` dit « du jour, à Zurich » ; les commentaires qui deviennent faux sont
  réécrits : `bank_accounts.rs:794-795` (« date du jour (UTC, comme le complément de l'écran) »),
  `opening_complement.rs:22` (« aujourd'hui (date UTC, passée en paramètre) »), `companies.rs:986-989` (« `Utc::now().date_naive()`,
  la même horloge que la contre-passation » : dire que le verrou et la contre-passation reçoivent la même date, celle de
  l'horloge de la requête) ; puis `/usr/bin/grep -n UTC` sur chaque fichier touché, résultat au Change Log (greper la
  valeur, non la formulation). Le manuel dit une fois, à l'endroit où il définit les écritures datées du jour, que « la
  date du jour est celle de Zurich, même la nuit », renvoyé depuis le solde d'ouverture et le verrou de période ; il
  distingue l'écran (la date du **poste**, `todayLocal`, #889) du serveur (Zurich), qui coïncident pour un poste réglé sur
  la Suisse.
- **D7 — L'écran du verrou suit** (`settings/+page.svelte:563-567`) : `maxLockDate` = la veille de `todayLocal()`, calculée
  sur les champs locaux ; l'entrée d'allowlist `local-date-guard.test.ts:55-58` est retirée.

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
   départ : `2025-12-31` dans 2025). L'annulation d'un règlement, avec le seul exercice 2025 ouvert (aucun ne couvre le
   `2026-01-01`), est refusée au rang 5 (`settlement_cancellation.rs:172`) **par le refus nommé**, et rien n'est écrit (le
   code de départ accepte et contre-passe au `2025-12-31`) ; la même requête lit l'horloge une fois (D2) : un test du dépôt
   passe la même date au bloqueur et à la contre-passation.
4. **AC4 — Verrou de période** (#907) : horloge fixée à `2026-03-10T23:30:00Z` (00 h 30 le 11 mars à Zurich).
   - `lock_books` jusqu'au `2026-03-10` : **accepté** (le code de départ refuse) ; jusqu'au `2026-03-11` : refusé —
     réponse `400 INVALID_INPUT`, clé `settings-books-lock-bound-not-past` (constante Rust `BOOKS_LOCK_BOUND_NOT_PAST`,
     `companies.rs:972` ; `errors.rs:4778-4795`).
   - `unlock_books` : borne courante au `2026-03-10` (posée par `lock_books` à la même heure fixée), cible `2026-03-10` →
     **acceptée** (le code de départ refuse : `2026-03-10 >= ` jour UTC) ; cible `2026-03-11` → refusée, même réponse. Une
     cible au `2026-03-09` passe déjà sur le code de départ : elle ne prouve rien.
   - Écran (D7) : `maxLockDate` vaut `2026-03-10` quand le poste est au `2026-03-11T00:30` local (Vitest,
     `vi.setSystemTime` sur une date construite en champs locaux) ; l'allowlist n'a plus l'entrée du verrou.
5. **AC5 — Premier exercice de l'assistant** (site 17, C-e350-23) : finalisation du chemin B
   (`routes/onboarding.rs:1043`) à `2025-12-31T23:30:00Z` → l'exercice créé s'appelle « Exercice 2026 » et couvre 2026 (le
   code de départ : « Exercice 2025 »).
6. **AC6 — `kesh-db` sans horloge** : les fonctions de D2 reçoivent la date ; leurs tests passent une date fixe ; aucun
   `Utc::now()` qui sert une date dans les fichiers de `kesh-db` touchés (garde).
7. **AC7 — L'horloge atteint les services et le MCP** (C-e350-21) : `Lecture::new` exige une horloge ; `AppState::lecture()`
   et le câblage du MCP passent `state.horloge` — les gardes lexicales (`mcp_gardes_lexicales.rs:169`, `:575` ;
   `services_motif.rs`, `FABRIQUES_V2`) portent la ligne nouvelle et restent vertes ; un test construit un `AppState` à
   `HorlogeFixe(2025-12-31T23:30:00Z)` et constate `state.lecture().aujourd_hui() == 2026-01-01` ; `services::factures::reglements`
   rend, pour un règlement dont l'exercice du jour suisse est clos, le motif du rang 5 (route `GET /invoices/{id}/settlements`).
   L'AC fonctionnel par un outil du MCP est à la 35-5b (`kesh_balance_agee`), qui convertit le premier service servi par le MCP.
8. **AC8 — La garde** (D4) : rouge sur le code de départ (les 9 sites de la 35-5a et les fichiers de tests qu'elle convertit
   ne sont pas exemptés) ; verte après, liste ramenée au compte exact de chaque fichier ; sondes du Change Log (positive,
   multi-ligne, compte + 1).
9. **AC9 — Tests sûrs la nuit, verdict par fichier** (C-e350-22) : après cette story, aucun test ne rougit entre minuit et
   1 h (2 h en été) à Zurich du fait d'un site de la 35-5a. Le verdict des **38 fichiers** de `crates/*/tests` est tenu au
   Change Log (tableau ci-dessous, à confirmer ligne lue citée) ; les fichiers marqués **35-5a** sont convertis (date fixe,
   horloge fixée, ou helper `aujourd_hui_suisse`) et comptent 0 à la garde ; ceux qui restent sont à leur compte exact.
   Le module `mod tests` de `kesh-db/src/repositories/journal_entries.rs` convertit dans cette story les tests qui appellent
   une fonction de D2 ou assertent une date de contre-passation ; les autres vont à la 35-5c.
10. **AC10 — Le chemin de jour ne change pas** : en journée (`2026-06-15T10:00:00Z`), les dates écrites sont celles d'avant ;
    tests existants verts sans retouche de leurs attendus, hors AC9.
11. **AC11 — Docs, manuel, CHANGELOG, PR** : D6 ; PDF régénéré, contrôlé aplati ; CHANGELOG `## [0.17.7] — Non publié`,
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
la fois un site de la 35-5a et un de la 35-5b fixe l'horloge, faute de quoi il ne peut être juste la nuit dans l'intervalle.

| Fichier (sites) | Verdict | Motif (indice relevé) |
|---|---|---|
| `kesh-api/tests/accounts_e2e.rs` (1 + 1 multi-ligne) | 35-5c | exercice de l'année pour le retypage |
| `admin_full_import_e2e.rs` (4) | 35-5c | bornes à `today - 30` : marge |
| `bank_account_opening_balance_e2e.rs` (2) | **35-5a** | solde d'ouverture (sites 1–2) |
| `bank_accounts_closed_e2e.rs` (6) | **35-5a** pour `:323`, `:386`, `:500`, `:1651` ; 35-5b pour `:678`, `:685` | règlement puis annulation (rang 5) ; `closedOn` (site 11) |
| `credit_card_e2e.rs` (3) | 35-5c | dates saisies de facture et de règlement (à confirmer : aucune annulation) |
| `dates_bornes_e2e.rs` (3) | **35-5a** | verrou au jour |
| `dunning_reminders_e2e.rs` (1) | 35-5b | sites 13, 22 |
| `filet_bilan_clos_e2e.rs` (3 + 1 multi-ligne) | **35-5a** | contre-passation du jour |
| `fiscal_years_e2e.rs` (1 + 1 multi-ligne) | 35-5b | `seed_demo` (site 26) ; `:1068` à lire |
| `homepage_summary_e2e.rs` (1) | 35-5b | accueil (site 21) ; ses `journal_entries::reverse` directs reçoivent `today()` (signature) |
| `invoice_delete_e2e.rs` (2) | **35-5a** | dévalidation (contre-passation) ; `:300` marge 30 j, `:163` à lire |
| `invoice_unvalidate_e2e.rs` (6) | **35-5a** | dévalidation, date de contre-passation |
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

Décompte provisoire : **24** fichiers en 35-5a (dont un en partie), **9** en 35-5b (plus la part de
`bank_accounts_closed_e2e.rs`), **5** en 35-5c — 38. À recompter.

## Tests à écrire, et la preuve du rouge

Écrire d'abord, lancer sur le code de départ, journal `kesh-gate-logs/e355a/rouges.log`.

**Le squelette compilable** est commité d'abord : `jour_suisse` déclarée **et qui rend encore le jour UTC**
(`instant.date_naive()`, c'est-à-dire M1), le trait `Horloge`, `HorlogeFixe`, `HorlogeSysteme`, `AppState.horloge`,
`Lecture` à horloge, et **les neuf sites déjà branchés** sur `state.horloge.aujourd_hui()` / `lecture.aujourd_hui()`, la date
passée à `kesh-db` (D2). Ainsi les rouges sont **déterministes** et disent le défaut, non l'absence de branchement :
AC1 (`2025-12-31` attendu `2026-01-01`), AC2, AC3 et AC5 (la veille, et l'exercice précédent), AC4 (`2026-03-10` refusé). Ils
ne dépendent pas du jour réel où on les lance. L'implémentation de `jour_suisse` les met au vert. AC8 rouge sur la garde.

**Mutations** : M1 `jour_suisse` rend le jour UTC ; M2 l'été sans +2 h (fuseau fixe +1) ; M3 un site de la 35-5a qui lit
`Utc::now()` ; M4 `lock_books` comparé au jour UTC ; M5 une exemption de la garde ajoutée sans site (la garde rougit sur le
compte) ; M6 le câblage du MCP construit sa `Lecture` avec `HorlogeSysteme` au lieu de `state.horloge` (AC7, garde lexicale
et test) ; M7 la contre-passation de l'annulation relit l'horloge au lieu de recevoir la date du bloqueur (AC3).

## Tâches

- [ ] Étape 0 : recompter l'inventaire (28 / 56 / 112 / 35, 8 commentaires), les appelants, les 43 constructions de `Lecture`
      et les 16 littéraux d'`AppState` ; confirmer le verdict des 38 fichiers, ligne lue citée ; `cargo tree` pour
      `chrono-tz` ; Change Log.
- [ ] Squelette compilable branché (ci-dessus), tests, rouges consignés.
- [ ] `kesh-core` : `jour_suisse`, trait `Horloge`, `HorlogeFixe`. `kesh-db` : date en paramètre (D2). `kesh-api` :
      `AppState.horloge`, `HorlogeSysteme`, `Lecture` à horloge, câblage du MCP et ses gardes lexicales, sites 1–8 et 17,
      `services::factures::reglements` ; helpers `aujourd_hui_suisse` ; garde D4. Écran du verrou (D7).
- [ ] Tests convertis (AC9) ; commentaires, docs, manuel, CHANGELOG.
- [ ] Gates : `test-fast.sh --ci` **complet à chaque commit de code** (exception `kesh-db`), `scripts/gate-ci.sh` au dernier
      commit de code, frontend complet, E2E complet au dernier commit de code.

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
  (`DATE(rmd.last_sent_at)` dans la date d'exigibilité, en SQL) : issue à ouvrir ; `reconciliation.rs:435` (la contrainte
  `chk_invoices_paid_at_after_date`, `DATE(paid_at) >= date - INTERVAL 1 DAY`) : sa tolérance d'un jour absorbe l'écart, et
  la changer demanderait une migration — hors périmètre, sans issue.

### La nuit, entre la 35-5a et la 35-5b

Un **E2E complet** lancé entre 00 h et 01 h (02 h en été) à Zurich confronte, jusqu'à la 35-5c, un serveur au jour suisse à
des specs qui calculent un jour UTC (`journal-entries.spec.ts`, `invoices-settlement-cancel.spec.ts`,
`invoice-write-off.spec.ts`, `helpers/api-fixtures.ts`). Les specs qui assertent une date d'un site de la 35-5a passent dans
cette story au helper existant `todayInZurich()` (`frontend/tests/e2e/helpers/api-fixtures.ts:390-392`) ; un rouge d'une
autre spec à cette heure n'est pas une régression : le nommer contre cette note.

### Fichiers communs — règle d'union C-e350-8

`opening_balances.rs` (la 35-6 ne le touche plus, C-e350-19) ; `bank_accounts.rs` avec la **35-5b** (ici `:1007`, `:1252`,
`:794-795` ; la 35-5b `:1802`, `:228`, `:1757`) ; `journal_entries.rs` avec la **35-8**, qui attend la 35-5c ;
**avec la 35-3** : `kesh-db/src/repositories/supplier_invoices.rs` (35-3 `:1521-1530`, `:1698-1727` ; ici les signatures
`:2462`, `:2600`, `:2750`, `:2871`), `invoice_settlements_write.rs` (35-3 `:679-680` ; ici `:878`, `:1023`),
`crates/kesh-db/tests/supplier_invoices_repository.rs` (35-3 `:2250-2257` ; ici `:1586`, `:1862`, `:2700`, `:2795`) — hunks
distincts, la seconde fusionnée se rebase sur la première ; `routes/invoices.rs`, `routes/supplier_invoices.rs`,
`routes/reconciliation.rs` (chaîne des appelants) ; `docs/api-external.md:272` avec la **35-6** (même ligne, à unir à la
main) ; `user-manual.tex` avec la **35-4** (`:3107`), la **35-6** (`:2456`), la **35-3** et la **35-2a** (paragraphes
distincts) ; `frontend/src/lib/shared/utils/local-date-guard.test.ts` et `settings/+page.svelte` (35-5a seule) ;
`CHANGELOG.md`. `lib.rs`, `services/poignees.rs`, `mcp_gardes_lexicales.rs`, `services_motif.rs` : 35-5a seule.

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

### Debug Log References

### Completion Notes List

- Fiche rédigée par le workflow `bmad-create-story` (analyse du code de `release/0.17.6`) ; validation à lancer.

### File List

*Attendue (à confirmer)* : `crates/kesh-core/Cargo.toml`, `crates/kesh-core/src/` (module du jour et de l'horloge, `lib.rs`) ;
`crates/kesh-db/src/repositories/{journal_entries,settlement_cancellation,companies,invoice_settlements_write,reconciliation_cancel,supplier_invoices,opening_complement}.rs` ;
`crates/kesh-api/src/{lib,main}.rs`, `middleware/auth.rs`, `services/{poignees,factures}.rs`,
`routes/{bank_accounts,opening_balances,companies,journal_entries,onboarding,invoices,supplier_invoices,reconciliation}.rs` ;
`crates/kesh-api/tests/date_du_jour_garde.rs`, `tests/common/jour.rs`, `tests/common/mcp.rs`, `mcp_gardes_lexicales.rs`,
`services_motif.rs`, les 10 fichiers des constructions de `Lecture`, les 9 fichiers des littéraux d'`AppState`, les fichiers
**35-5a** du verdict ; `crates/kesh-db/tests/support/jour.rs` ; `crates/kesh-report/tests/open_items_invariant.rs` ;
`frontend/src/routes/(app)/settings/+page.svelte`, `frontend/src/lib/shared/utils/local-date-guard.test.ts`, les specs E2E
qui assertent une date d'un site de la 35-5a ; `docs/api-external.md`, `docs/manual/fr/user-manual.{tex,pdf}`, `CHANGELOG.md`.

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
