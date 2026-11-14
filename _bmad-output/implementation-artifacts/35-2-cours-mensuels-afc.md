# Story 35-2 : table des cours mensuels et récupération des cours officiels (AFC / OFDF)

Status: ready-for-dev

*Fiche rédigée le 2026-11-14 (Opus 5.5), branche `epic/35-multi-devises`. Epic 35, C-35-1, C-35-6. Suit 35-1.
**Migration non breaking** (tables système `_kesh_*` seulement).*

## Story

En tant que comptable en multi-devises, je veux que Kesh récupère seul, chaque mois, les cours mensuels moyens publiés
pour la TVA (AFC, données de l'OFDF), pour toutes les devises, et que je puisse les consulter et en demander la
récupération pour un mois donné, afin que mes écritures en devise reçoivent le cours officiel sans saisie.

## Source (établie le 2026-11-14, fichier d'epic § *Source des cours*)

- `GET https://www.backend-rates.bazg.admin.ch/api/xmlavgmonth?j=<AAAA>&m=<M>` ;
- `<monatsmittelkurs xmlns="https://www.backend-rates.bazg.admin.ch/avgratesmonth" …><monat>AAAA-MM</monat><devise
  code="usd"><land_de>…</land_de><land_fr>…</land_fr><land_it>…</land_it><waehrung>1 USD</waehrung><kurs>0.8846</kurs>…`
  — UTF-8, espace de noms par défaut, 73 devises et 14 Ko en 2024-12 ; code en minuscules ; cours à quatre décimales ;
- **l'unité est un entier positif** en tête de `<waehrung>` : relevé 2024-12, 19 devises à `1`, 42 à `100`, 7 à
  `1000` (CLP, CRC, KZT, NGN, LKR, TZS, HUF), 5 à `10000` (IDR, KHR, COP, LBP, VND) — jamais « 1 ou 100 » seulement ;
  des cours très petits existent (`1 ARS` → `0.0009`) ;
- **le cours du mois courant est publié en début de mois** (relevé le 2026-11-14 : `xmlavgmonth` sans paramètre
  rendait `2026-11`) ;
- mois non publié : élément racine vide → « pas encore publié » ;
- la réponse peut porter un autre mois que celui demandé (sans paramètre : mois courant) : **le `<monat>` lu fait foi** ;
  un écart avec le mois demandé est une erreur (rien n'est écrit).

## Critères d'acceptation

1. **AC-1 — Migration** `20261114000002_exchange_rates.sql` (non breaking, DDL seul) :
   - `_kesh_exchange_rates (currency CHAR(3) CHARACTER SET ascii COLLATE ascii_bin, month DATE, rate_per_unit DECIMAL(20,10) NOT NULL, published_unit
     INT NOT NULL, published_rate DECIMAL(19,6) NOT NULL, fetched_at DATETIME(3) NOT NULL, PRIMARY KEY
     (currency, month), CHECK (DAY(month) = 1), CHECK (rate_per_unit > 0), CHECK (published_unit > 0),
     CHECK (BINARY currency REGEXP '^[A-Z]{3}$'))` — `INT` et non `SMALLINT` (unités jusqu'à 10000 relevées) ;
     `currency` a **la même définition** qu'`accounts.currency` (35-1 AC-1) et `journal_entry_lines.currency`
     (35-3a AC-1), et le `CHECK` porte `BINARY` comme dans ces deux fiches : la table, en `utf8mb4_unicode_ci` par
     défaut, confondrait sinon `usd` et `USD` ;
   - `_kesh_exchange_rate_fetch` singleton : `id` (`CHECK (id = 1)`), `triggered_by ENUM('auto','manual')` (**pas
     `trigger`** : mot réservé de MariaDB ; le champ JSON de l'API s'appelle `trigger`, lu depuis cette colonne),
     `last_run_at`, `last_status` (`CHECK IN ('ok','error','not_published')`), `last_error VARCHAR(64)` — **un code
     classé**, jamais le texte d'une erreur `reqwest` (qui porte l'URL) : `timeout`, `connect`, `http_status:<n>`,
     `too_large`, `malformed`, `month_mismatch`, `no_valid_rate`, `invalid_rates:<n>` —, `last_month DATE`. Le tour
     d'une récupération écrit **une** ligne : `ok` (avec `last_error = invalid_rates:<n>` si des devises ont été
     écartées), `not_published` ou `error` ; un tour de la boucle qui traite deux mois écrit le **pire** des deux
     statuts (`error` > `not_published` > `ok`) et `last_month` du mois en cause (à égalité, le plus récent). **Ligne créée
     paresseusement** (aucune écriture de données dans la migration, P7 sans objet ; le dump `--no-data` de
     `regen-test-schema.sh` n'a rien à réinjecter), écrite par `INSERT … ON DUPLICATE KEY UPDATE` ;
   - inscrites à `TABLES_SYSTEME` (`crates/kesh-db/src/backup.rs` :949, lue par `backup_inventory_matches_schema` et
     `convention_du_prefixe_des_tables_systeme`), absentes de l'inventaire `.keshbackup` ; `migrations.sha384`, ligne et
     compteurs recomptés de `docs/migrations-idempotence-audit.md` (P5), squash régénéré, P6 ;
   - test de montée sur le modèle d'`installation_settings_adds_only_its_system_table`
     (`crates/kesh-db/tests/migrations_upgrade_path.rs:1766`), monté **par version**
     (`common::migrations_before(VERSION, LABEL)`, `tests/common/mod.rs:75`, P6) : deux tables neuves, système, vides ; aucune table applicative ajoutée, donc une sauvegarde 0.17.6 reste
     importable.
2. **AC-2 — Analyse XML** dans `kesh-core` (pure, sans I/O) : `parse_monthly_rates(xml) -> Result<MonthlyRates>`
   (mois, liste `{currency, published_unit, published_rate, rate_per_unit}`), `rate_per_unit = published_rate /
   published_unit` (`Decimal`, exact pour les unités puissances de dix relevées), **arrondi à 10 décimales**
   (`round_dp_with_strategy(10, RoundingStrategy::MidpointAwayFromZero)` — **pas** `round_dp`, qui arrondit
   l'équidistant au pair ; même règle que C-35-4 —, la précision de la colonne : la valeur lue par `FX_RATE_NOT_AFC`
   est celle qui est stockée) ; code mis en majuscules et validé (trois lettres). Parseur `quick-xml = "0.42"`
   (déjà au `Cargo.lock` par `kesh-import` et `kesh-payment`, dépendance neuve de `kesh-core`), lecture **par noms
   locaux** (la racine porte un `xmlns` par défaut), éléments inconnus ignorés (`land_de`, `land_fr`, `land_it`).
   - **Unité** : entier positif en tête de `<waehrung>` (1, 100, 1000, 10000 relevés — aucune liste fermée), le code
     qui suit doit égaler l'attribut `code` (casse ignorée).
   - **Tolérance par devise** : une devise invalide (cours ≤ 0, unité ou code illisible, code et `<waehrung>`
     discordants, doublon — les deux occurrences) est **écartée**, **comme toute devise que la base refuserait** :
     unité > 1 000 000, cours publié ≥ 10⁹ (borne de prudence, plus stricte que les 13 chiffres entiers de
     `DECIMAL(19,6)`) ou à plus de 6 décimales (que la colonne arrondirait en silence, rendant `published_rate /
     published_unit` différent du `rate_per_unit` stocké), `rate_per_unit` arrondi à 0 ou > 10⁶ (borne du cours d'une
     ligne en devise, 35-3a AC-3 : un cours de la table que la ligne refuserait ne servirait à rien ; plus stricte
     que la borne 10¹⁰ de `DECIMAL(20,10)`) — sans quoi un `INSERT` refusé annulerait la transaction du mois entier (500) au
     lieu d'écarter la seule devise fautive ; elle est rendue dans `MonthlyRates.rejected` et journalisée
     par l'appelant (`tracing::warn!`, `last_error = invalid_rates:<n>`) ; les autres s'écrivent. Une seule devise
     fautive (p. ex. un cours publié `0.0000`) ne bloque donc jamais USD ni EUR.
   - **Refus du mois entier** seulement si : XML mal formé, `<monat>` absent ou illisible, ou aucune devise valide
     (`NoValidRate`). Racine vide → `NotPublished`.
   - Fixture versionnée **synthétique** (forme du XML réel, `xmlns` compris, valeurs inventées — aucun cours officiel
     n'est versionné : conditions d'utilisation SIX) avec unités 1, 100, 1000 et 10000 et un cours à 0.0009 ; tests du
     cas vide, des refus du mois et des écarts par devise.
3. **AC-3 — Source des cours et client HTTP** (`kesh-api`, module `exchange_rates`).
   - **Trait `RateSource`** (`fn fetch_month(&self, year, month) -> Pin<Box<dyn Future<Output = Result<String,
     SourceError>> + Send + '_>>` : **objet-safe sans `async_trait`**, comme `Mailer`, `mail/mod.rs:12` et `:32` — un
     `async fn` dans un trait n'est pas utilisable en `dyn`), porté par `AppState` (`rate_source: Arc<dyn
     RateSource>`, précédent `mailer: Arc<dyn Mailer>` / `test_mock_mailer`, `lib.rs:84-98`, et `BackupEnv` de la
     sauvegarde automatique ; `AppState::new_for_tests` reçoit une source factice par défaut).
   - **Champs neufs d'`AppState`** : `rate_source`, `rate_fetch_lock: Arc<tokio::sync::Mutex<()>>`, `rate_fetch_memo`
     (la mémoire des 15 minutes de l'AC-5, `Arc<std::sync::Mutex<HashMap<(i32, u32), …>>>`) et
     `test_rate_source: Option<MockRateSource>` (poignée du mode test, comme `test_mock_mailer`). **Sites de
     construction littérale à compléter** (inventaire au `grep -rn "AppState {"`, à relancer) : `main.rs:439`,
     `middleware/auth.rs:285`, `AppState::new_for_tests` (`lib.rs:119`), et les tests `password_recovery_e2e.rs:126`,
     `invoice_frozen_pdf_e2e.rs:101`, `invoice_send_email_e2e.rs:104`, `:1036`, `:1150`, `setup_admin_e2e.rs:82` ;
     l'échec de compilation les inventorie.
   - **Production** : `HttpRateSource`, URL de base **constante** `https://www.backend-rates.bazg.admin.ch`, **aucune
     variable d'environnement** ne la change. Les tests Rust construisent `HttpRateSource::with_base_url(…)` contre un
     serveur axum local en HTTP. ⚠️ **Visibilité** : `with_base_url`, `un_tour` (AC-4) et `MockRateSource` sont
     `#[doc(hidden)] pub` (précédent `AppState::new_for_tests`, `lib.rs:119`) — un `pub(crate)` ou un
     `#[cfg(test)]` est invisible des tests d'intégration de `crates/kesh-api/tests/` (précédent de l'Epic 31,
     HIGH en validation) ; ou bien ces tests vivent dans le `mod tests` du module, à déclarer alors ici.
   - **`KESH_TEST_MODE`** : `main.rs` substitue une **source factice** (XML synthétique en mémoire, aucun appel
     sortant ; la boucle n'est pas lancée en mode test, AC-4), pilotée, par la poignée `test_rate_source`, par un endpoint `POST /api/v1/_test/exchange-rates/source` (réponse
     servie : XML, vide, erreur, délai ; **XML synthétique par défaut** tant que rien n'est piloté, **généré pour le
     mois demandé** (`<monat>` = `(j, m)` de l'appel : un XML figé sur un mois rendrait `month_mismatch` pour tout
     autre ; un XML piloté est servi tel quel) ; valeurs du défaut fixées et écrites au module, que les specs
     E2E — 35-3b comprise — lisent pour leurs assertions) monté avec les autres routes `_test` ; il entre au registre (`TEST_ENDPOINT_ROUTES` : +1,
     détail des totaux à l'AC-5). La recette E2E de `CLAUDE.md` et de `docs/testing.md` reste inchangée.
   - `reqwest` en dépendance de production, `default-features = false`, `rustls-tls` ; `.use_rustls_tls()` explicite
     (la dev-dependency, `Cargo.toml:109`, garde les fonctionnalités par défaut — dont `default-tls`, native-tls — et
     ses `json`, `multipart`, `cookies` explicites, qui s'unifient dans les builds de test ; `cookies` n'active aucun
     magasin sans `.cookie_store(true)`) ; `redirect(Policy::none())` ; `https_only(true)` pour l'URL de production ; `connect_timeout` 5 s et délai
     total 20 s ; pas de magasin de cookies (la source pose `rates_service_session`) ; réponse limitée à 1 MiB
     (`Content-Length` contrôlé puis lecture par `chunk()` bornée) ; `User-Agent: Kesh/<version>` sans donnée
     personnelle. `Cargo.lock` change (l'entrée `reqwest` gagne les dépendances `rustls` ; crates déjà au lock, la liste exacte se
     constate au build et non ici) et se commite ; `rustls-tls` embarque `webpki-roots`, l'image a de toute façon `ca-certificates`
     (`Dockerfile:20`) ; le build Docker (`cargo build --release -p kesh-api`) n'a rien d'autre à changer.
   - **Une seule récupération à la fois** (`rate_fetch_lock`, `tokio::sync::Mutex` dans `AppState`, comme
     `backup_lock` ; **la boucle l'attend** — elle ne saute pas son tour —, un `POST /fetch` l'attend aussi). Le verrou
     se prend **par mois récupéré**, jamais pour un tour entier de la boucle (qui en traite deux : 2 × 20 s) : l'attente
     d'un `POST /fetch` est alors bornée par **une** requête sortante (20 s) et son écriture. Sous le verrou, le
     `POST /fetch` relit la mémoire de l'AC-5 avant d'appeler la source. Le client du frontend appelle `POST /fetch`
     avec `timeoutMs: 60_000` (`CallOptions`, `api-client.ts:61` ; le défaut de 30 s, `:42`, ne couvre pas 20 s
     d'attente + 20 s de requête : il rendrait `TIMEOUT`, lu comme issue inconnue) ; écriture en **une transaction** : `INSERT … ON DUPLICATE KEY UPDATE` des devises valides du mois ; un échec du mois
     n'écrit aucun cours.
4. **AC-4 — Récupération automatique.** Une boucle `exchange_rates::spawn(state)` sur le modèle de
   `auto_backup::spawn` (`auto_backup.rs:788`) : **premier tour 5 minutes après le démarrage**, puis toutes les 6 h
   (`interval_at(now + PREMIER_TOUR, CADENCE)`, `MissedTickBehavior::Skip`), chaque tour dans son propre
   `tokio::spawn` (une panique est journalisée et n'arrête pas la boucle). Lancée de `main.rs` **seulement**, avec un
   commentaire de renvoi à la garde comme `main.rs:463-466`, et **pas en `KESH_TEST_MODE`** (`if
   !config.test_mode`) : un premier tour à cinq minutes tomberait au milieu de la suite E2E, avec la réponse que
   la spec en cours aurait pilotée, et remplirait la table sous une autre spec — les tours se testent en Rust
   (`un_tour`, AC-3) ; garde lexicale
   `exchange_rates::tests::lancement_par_main_seulement` (jeton `exchange_rates::spawn(` : une fois dans `main.rs`,
   zéro dans `lib.rs`), qui **réutilise** l'aide `code()` d'`auto_backup` (extraite ou partagée, refs #727) au lieu de
   la recopier (DRY). Sautée si **aucune** société n'a `multi_currency_enabled` (aucun trafic sortant en
   mono-devise) ; à chaque tour : récupère le **mois courant** s'il manque (publié en début de mois), et le **mois
   précédent** s'il manque — « manque » = aucune ligne pour ce mois. « Mois courant » s'entend en
   **Europe/Zurich**, non en UTC (même principe que #889, qui est une correction du frontend : le backend n'a aucune
   aide de fuseau). `chrono-tz` n'est aujourd'hui au `Cargo.lock` (0.10.4) que par `rxing` : l'ajouter comme
   **dépendance directe** de `kesh-api` (`chrono-tz = "0.10"`) et poser **une** fonction `mois_courant_zurich(now)`,
   partagée par la boucle et par la borne du `POST /fetch`, testée aux deux bords du mois (31 → 1er, UTC contre
   Zurich). Un échec se journalise (`tracing::warn!`), s'écrit dans
   `_kesh_exchange_rate_fetch` (`triggered_by = 'auto'`) et se retente au tour suivant ; jamais bloquant au démarrage.
   Non auditée (C-35-6). **Seule la boucle** écrit `triggered_by = 'auto'`.
5. **AC-5 — API.**
   - `GET /api/v1/exchange-rates?month=AAAA-MM[&currency=USD]` : cours du mois (toutes devises ou une) ;
   - `GET /api/v1/exchange-rates/status` : état de `_kesh_exchange_rate_fetch` (`trigger`, date, statut, code
     d'erreur) ;
   - `POST /api/v1/exchange-rates/fetch { year, month }` — dans **`comptable_routes`** (Administrateur ou Comptable ;
     mois ≥ 2000-01 et ≤ mois courant en Europe/Zurich, sinon 400) : récupère ce mois, **auditée sous l'utilisateur**
     (`exchange_rates.fetched`, mois, nombre de devises écrites, écartées et dont le cours a changé (`changed`) — écrite **sur un succès seulement** : un
     404 ou un 503 n'écrit que `_kesh_exchange_rate_fetch`) ; écrit `_kesh_exchange_rate_fetch` avec
     `triggered_by = 'manual'` ; réponses `200 {month: "AAAA-MM-01", count, rejected}` (`count` et `rejected` sont des
     **nombres** ; les codes écartés vont au journal du serveur), `404 RATE_NOT_PUBLISHED`,
     `503 RATE_SOURCE_UNAVAILABLE` (code explicite, message sans URL ni trace — **jamais 502** : le frontend lit 502 et
     504 comme une issue inconnue venue d'un proxy, `frontend/src/lib/shared/utils/unknown-outcome.ts`). ⚠️ Mais
     **tout 503** allume aujourd'hui le bandeau « service dégradé » (`apiHealth.setDegraded()`,
     `api-client.ts:393-394`, avant toute lecture du corps), conçu pour une base injoignable : la fonction d'API du
     frontend qui appelle `POST /fetch` (seule entrée, partagée par cet écran, 35-3b et 35-5) passe une option neuve
     de `CallOptions` qui ne l'allume pas (`degradedOn503: false`, nom au développement), avec son test Vitest. **Tout**
     échec de récupération se rend ainsi — `timeout`, `connect`, `http_status`, `too_large`, mais aussi `malformed`,
     `month_mismatch` et `no_valid_rate` (la source a répondu, inutilisablement) : jamais 500 ; le code classé va à
     `last_error`, pas à l'utilisateur. Un mois `not_published` ou en échec n'est pas redemandé à la source avant 15
     minutes (`rate_fetch_memo`, clé `(année, mois)`, alimentée aussi par la boucle) : la réponse d'origine est
     rendue, sans nouvelle ligne d'audit ni écriture de `_kesh_exchange_rate_fetch` ;
   - `GET /api/v1/exchange-rates/lookup?currency=USD&date=AAAA-MM-JJ` — **en lecture seule**, sans appel sortant ni
     écriture ni audit, ouverte à tout rôle authentifié : cours du mois de la date
     (`200 { currency, month: "AAAA-MM-01", ratePerUnit, publishedUnit, publishedRate }`, décimaux en chaînes, forme
     lue par 35-3b), ou `404 RATE_UNAVAILABLE`, `details: { currency: "USD", month: "AAAA-MM-01" }` (jamais 1.0) —
     **forme unique** de ce code, que le `DbError::RateUnavailable` de 35-3a rend à l'identique, et d'où l'écran tire
     `{year, month}` du `POST /fetch`. L'écran de saisie (35-3b) et la réévaluation (35-5)
     appellent `POST /fetch` sur ce 404 (rôles Administrateur et Comptable), puis relisent ; sans cours, saisie
     manuelle. Devise `CHF` (devise principale, sans cours) → 400 ; devise absente de la liste ISO 4217 de 35-1
     (`kesh-core::currency`) → 400 `CURRENCY_UNKNOWN` (code de 35-1) ; `month` ou `date` absents ou illisibles (les deux `GET`) → 400.
   - Toutes ces routes exigent le mode multi-devises de la société courante : **`409 MULTI_CURRENCY_DISABLED`**, même
     statut que dans la 35-1.
   - Nouvelles variantes `AppError` (`errors.rs`) et leur traduction côté frontend pour `RATE_NOT_PUBLISHED`,
     `RATE_SOURCE_UNAVAILABLE`, `RATE_UNAVAILABLE`.
   - **Audit et registres** : action `exchange_rates.fetched` ajoutée à `ACTIONS` (`crates/kesh-api/src/audit_labels.rs`,
     liste triée, garde `tests/audit_label_registry.rs`) ; type d'entité `installation` avec `AUDIT_ENTITY_ID_NONE`
     (précédent `admin.backup_run`, `routes/backup_schedule.rs:177-191`, `emit_audit` après coup — la table est
     globale et `audit_log.entity_id` est `NOT NULL`) ; clé `audit-log-action-exchange-rates-fetched` dans les quatre
     locales. `POST /fetch` entre dans `LIB_ROUTES` (`tests/audit_route_registry.rs`, `Traced, SansEcritureAuJournal`) :
     `the_registry_partition_is_what_the_story_declares` gagne **+1 route et +1 tracée** (état au 2026-11-14 : 146 routes,
     139 tracées, total registre 149 ; **ces chiffres s'écrivent en deltas** — la 35-1 et la 35-5 ajoutent aussi leur
     route mutante, l'ordre d'intégration décide du chiffre absolu, à recompter au code), et le total registre **+2**
     avec l'endpoint `_test` (entrée `("post", "/exchange-rates/source", Exempt(…), Exemptee(…))` de
     `TEST_ENDPOINT_ROUTES`) ; la phrase du manuel d'administration (`admin-manual.tex:3217`, « N des M routes à verbe
     mutant » et « N + 4 + 3 = M », gardée par `le_manuel_dit_le_compte_du_registre`) suit le chiffre recompté.
6. **AC-6 — Écran** *Paramètres → Cours de change* (visible en multi-devises) : choix du mois, tableau
   (devise, unité publiée, cours publié, cours par unité, date de récupération), filtre par devise, bouton
   « Récupérer ce mois » (Administrateur et Comptable seulement), état de la dernière récupération (automatique ou
   manuelle, d'après `trigger` ; date, statut, code d'erreur traduit) ; mention de la source et de son caractère non
   contraignant. Carte sur `settings/+page.svelte`. Captures d'écran.
7. **AC-7 — Restauration.** Les deux tables sont système : un import de sauvegarde ne les touche pas. Sur une
   installation neuve restaurée, elles sont vides : la boucle ou `POST /fetch` les complète ; aucun test ne suppose un
   cours présent sans l'avoir posé. ⚠️ **Isolation des E2E** : système, elles échappent à `truncate_all`
   (`TABLES_TO_TRUNCATE`, `backup.rs:46`) que lancent `POST /_test/seed` et `/_test/reset`
   (`test_endpoints.rs:189`, `:325`) ; sans plus, un cours posé par une spec survit à la suivante, qui attendrait
   un `404 RATE_UNAVAILABLE`. Ces deux routes vident donc aussi `_kesh_exchange_rates` et
   `_kesh_exchange_rate_fetch`, et `clear_transient_test_state` (`test_endpoints.rs:349`) vide `rate_fetch_memo` et
   remet la source factice à son défaut. Test Rust de ce vidage.
8. **AC-8 — i18n, manuels.** Quatre locales (écran, codes d'erreur, libellé d'audit) ; manuel utilisateur (écran,
   source, récupération) ; manuel d'administration : accès sortant HTTPS vers `www.backend-rates.bazg.admin.ch`
   requis en multi-devises, aucun en mono-devise, mandataire par `HTTPS_PROXY` (lu par `reqwest`) ; phrase du
   registre des routes (AC-5) ; PDF régénérés.
9. **AC-9 — Cours manuels.** Aucun cours manuel n'entre dans `_kesh_exchange_rates` (C-35-1) : un cours saisi à la
   main vit **sur la ligne d'écriture** (`rate_source = 'manual'`, 35-3a/35-3b), audité avec l'écriture.

## Tâches

- [ ] Migration + inscriptions (AC-1).
- [ ] `kesh-core::exchange_rates` (AC-2) ; `kesh-db::repositories::exchange_rates` (upsert en transaction, lecture,
      état). La lecture d'un cours `(devise, mois)` prend une **connexion ou une transaction** (`&mut
      MySqlConnection`, comme les fonctions `*_in_tx` du dépôt) : 35-3a (AC-4, 4) la lit dans la transaction de
      l'écriture, 35-5 sous ses verrous — aucun second accès. L'upsert d'un mois déjà présent qui **change** un
      `rate_per_unit` existant le journalise (`tracing::warn!`, devise, ancien et nouveau cours) et le compte
      (`changed` dans le détail d'audit de `POST /fetch`) : les lignes `afc` déjà passées gardent leur cours, mais
      leur `PUT` serait refusé `FX_RATE_NOT_AFC` (35-3a) — l'écart ne doit pas être muet.
- [ ] `kesh-api` : trait `RateSource`, `HttpRateSource`, source factice et endpoint `_test` (AC-3), boucle (AC-4,
      `main.rs`), routes (AC-5, `lib.rs` : `GET` dans `authenticated_routes`, `POST /fetch` dans
      `comptable_routes`), variantes `AppError`, audit (`audit_labels.rs`), registre des routes et phrase du manuel.
- [ ] Frontend : page `settings/exchange-rates`, carte des paramètres, types, API, codes d'erreur, Vitest (AC-6).
- [ ] Tests : parseur (fixture : unités 1/100/1000/10000, devise écartée, mois refusé) ; client contre un serveur HTTP
      local (délai, 500, redirection non suivie, XML tronqué, corps trop gros, mois différent → rien d'écrit) ;
      `un_tour` sauté en mono-devise ; `un_tour` contre une source injoignable rend sans paniquer, écrit `error` et
      `triggered_by = 'auto'`, ne touche aucun cours ; `lookup` n'écrit rien et ne sort pas ; `POST /fetch` refusé au rôle
      Consultation, audité, `triggered_by = 'manual'`, 409 `MULTI_CURRENCY_DISABLED` en mono-devise (les quatre routes),
      400 hors bornes de mois, mémoire de 15 minutes (un mois `not_published` n'est pas redemandé, ni audité deux
      fois), `malformed` / `month_mismatch` / `no_valid_rate` → 503 sans rien écrire, `lookup` `CHF` → 400,
      `mois_courant_zurich` aux bords du mois, devise aux bornes hors `DECIMAL` écartée sans annuler les autres,
      `last_status` d'un tour à deux mois (le pire) ; aucune trace d'audit en automatique ; deux `POST /fetch` simultanés
      sérialisés (le second attend au plus un mois, AC-3) ; `POST /_test/seed` et `/reset` vident les deux tables et
      la mémoire (AC-7) ; source factice : XML par défaut au mois demandé ; un `rate_per_unit` changé au refetch est
      compté ; Vitest : un 503 de `POST /fetch` n'allume pas le bandeau dégradé ; E2E (écran, récupération servie par
      la source factice du mode test).
- [ ] Manuels, CHANGELOG, `Cargo.lock`.

## Notes de développement

- Modèle de boucle : `crates/kesh-api/src/auto_backup.rs` (`spawn` :788, `un_tour` :803, `CADENCE` :58), lancement
  `main.rs:467`, garde `auto_backup::tests::lancement_par_main_seulement` (:1204, aide `code()` :1190).
- `reqwest` est déjà au `Cargo.lock` (0.12.28, dev-dependency de `kesh-api`, défauts + `json`, `multipart`,
  `cookies`) ; `rustls`
  0.23.46 (ring) y est aussi.
- Le cours du mois courant est publié en début de mois (relevé le 2026-11-14 : sans paramètre, `2026-11`).
- Pas de production des cours dans le dépôt : la fixture est synthétique (C-35-1, conditions SIX).
- Pas d'audit automatique : `audit_log.user_id` est `NOT NULL` (C-35-6).

## Change Log

- 2026-11-14 — fiche rédigée.
- 2026-11-14 — validation P1 (Opus) : 0 CRITICAL, 2 HIGH, 8 MEDIUM, 9 LOW ; remédiés ; alignement de 35-3b/35-5 sur
  POST /fetch et du tableau de l'epic portés par l'orchestrateur.
- 2026-11-14 — validation P2 (Sonnet) : 0 CRITICAL, 0 HIGH, 8 MEDIUM, 8 LOW ; remédiés dans la fiche sauf P2-14, P2-15
  et P2-16 (inter-fiches : 35-1 et 35-5, 35-3b, fichier d'epic).
- 2026-11-14 — validation P3 (Opus) : 0 CRITICAL, 0 HIGH, 5 MEDIUM, 9 LOW (plus 2 LOW inter-fiches) ; remédiés dans la
  fiche sauf 35-2-P3-IF1 (35-3a, forme des `details` de `RATE_UNAVAILABLE`) et 35-2-P3-IF2 (35-3b, appel de
  `POST /fetch` par la fonction d'API de 35-2). Choix de conception de cette passe, à confirmer par la P4 : boucle
  non lancée en mode test (P3-1), verrou pris par mois (P3-3), option de `CallOptions` contre le bandeau dégradé
  (P3-2), audit de `POST /fetch` sur succès seulement (P3-13).
