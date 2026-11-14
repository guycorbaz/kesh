# Story 35-2 : table des cours (mensuels AFC et journaliers), cours saisis par la société, récupération des cours officiels (AFC / OFDF)

Status: ready-for-dev

*Fiche rédigée le 2026-11-14 (Opus 5.5), branche `epic/35-multi-devises`. Epic 35, C-35-1 (amendé par D9/D10, CR-1),
C-35-6, D9, D10. Suit 35-1. **Migration non breaking** : deux tables système `_kesh_*` et **une table applicative**
`exchange_rate_overrides`, qui **referme la fenêtre d'import** des sauvegardes antérieures (AC-1, AC-7).*

## Story

En tant que comptable en multi-devises, je veux que Kesh récupère seul, chaque mois, les cours mensuels AFC (fixés
d'avance pour le mois, publiés pour la TVA par l'AFC, données de l'OFDF), pour toutes les devises, qu'il récupère à la
demande le cours journalier OFDF d'une date (la date de clôture), que je puisse consulter ces cours, **saisir mon propre
cours** pour une devise et un mois ou un jour — jamais écrasé par une récupération — et reprendre d'un clic le cours
AFC, afin que mes écritures en devise et ma réévaluation reçoivent le bon cours sans saisie.

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
- **Terminologie** (fichier d'epic) : ces cours ne sont **pas des moyennes** — ils sont fixés d'avance pour le mois.
  Textes, libellés et manuels disent « cours mensuel AFC », jamais « cours moyen » (le nom allemand de la page ESTV,
  *Monatsmittelkurse*, et la racine XML `monatsmittelkurs` restent ce qu'ils sont : des noms de la source).

### Cours journaliers (D10, relevés le 2026-11-14 par deux appels `GET`)

- `GET https://www.backend-rates.bazg.admin.ch/api/xmldaily?d=<AAAAMMJJ>` (le paramètre `locale` n'a pas d'effet sur
  les données) ;
- `<wechselkurse xmlns="https://www.backend-rates.bazg.admin.ch/xmldaily" …><datum>31.12.2024</datum><zeit>04:15:03</zeit>
  <gueltigkeit>01.01.2025,02.01.2025,03.01.2025</gueltigkeit><devise code="eur"><land_de>…</land_de><land_fr>…</land_fr>
  <land_it>…</land_it><land_en>…</land_en><waehrung>1 EUR</waehrung><kurs>0.94947</kurs></devise>…` — espace de noms
  **différent** de celui des cours mensuels, élément `land_en` en plus ; **cinq décimales** ; mêmes unités (relevé du
  30.06.2024 : 74 devises, 19 à `1`, 43 à `100`, 7 à `1000`, 5 à `10000`, 17 Ko) ; `d=20241231` rend USD 0.91193 et
  EUR 0.94947, les cours de la réévaluation des livres 2024 de Beat (D10) ;
- **un jour sans publication rend la dernière publication antérieure** : `d=20240630` (un dimanche) rend
  `<datum>28.06.2024</datum>` (le vendredi), `<gueltigkeit>29.06.2024,30.06.2024,01.07.2024</gueltigkeit>`. Le cours
  journalier **d'une date D** est donc, par définition dans Kesh, celui que rend `xmldaily?d=D` : la publication du
  jour D, ou la dernière qui la précède. `<gueltigkeit>` n'est pas lu (Kesh suit la convention des livres de Beat :
  la publication **datée** du 31.12, non celle « valable » le 31.12) ;
- **pas de publication : HTTP 404, corps JSON** — relevé le 2026-11-14 pour `d=20301231` : `404`
  `{"error":"Aucune donnée disponible","message":"Aucun taux de change trouvé pour la période demandée"}` (et non une
  racine vide, comme `xmlavgmonth`). Le client traite un **404** de `xmldaily` comme « pas encore publié »
  (`NotPublished`, → `404 RATE_NOT_PUBLISHED`, mémorisé 15 minutes), **jamais** comme `http_status:404` : `fetch_day` le
  rend par une variante propre de `SourceError` (p. ex. `SourceError::NotPublished`), que l'appelant traite comme la
  racine vide ; tout autre statut non 2xx de `xmldaily`, et **tout** non 2xx de `xmlavgmonth` (404 compris), reste
  une erreur (`http_status:<n>`). Relevé aussi : `d=20000103` rend `<datum>03.01.2000</datum>` et `d=20000101` rend
  `<datum>31.12.1999</datum>` (la borne 2000-01-01 de l'AC-5 est servie) ; les éléments `<land_en/>` peuvent être
  **vides** (ignorés) ; `<gueltigkeit>` peut ne porter qu'une date ;
- contrôle de la réponse : `<datum>` lu ≤ D et D − `<datum>` ≤ 7 jours, sinon erreur `date_mismatch` (rien n'est
  écrit) ; racine vide → « pas encore publié », comme pour `xmlavgmonth` (par prudence : le cas réel relevé est le 404) ; une date ≥ aujourd'hui (Europe/Zurich) n'est pas demandée (AC-5) : la publication du jour tombe
  vers 04:15 et, avant, la source rendrait celle de la veille sous la date du jour.

## Critères d'acceptation

1. **AC-1 — Migration** `20261114000002_exchange_rates.sql` (non breaking, DDL seul) :
   - `_kesh_exchange_rates (currency CHAR(3) CHARACTER SET ascii COLLATE ascii_bin, kind VARCHAR(5) CHARACTER SET
     ascii COLLATE ascii_bin NOT NULL, rate_date DATE NOT NULL, rate_per_unit DECIMAL(20,10) NOT NULL, published_unit
     INT NOT NULL, published_rate DECIMAL(19,6) NOT NULL, source_date DATE NULL, fetched_at DATETIME(3) NOT NULL,
     PRIMARY KEY (currency, kind, rate_date), CHECK (BINARY kind IN ('month','day')), CHECK (kind <> 'month' OR
     DAY(rate_date) = 1), CHECK ((kind = 'day') = (source_date IS NOT NULL)), CHECK (rate_per_unit > 0),
     CHECK (published_unit > 0), CHECK (BINARY currency REGEXP '^[A-Z]{3}$'))` — **les cours AFC bruts seulement**
     (mensuels et journaliers), globaux, jamais un cours saisi ; `rate_date` = le 1er du mois pour `month`, la date
     **demandée** D pour `day` ; `source_date` = le `<datum>` lu pour `day` (≤ D, cf. *Cours journaliers*), `NULL` pour
     `month` ; `INT` et non `SMALLINT` (unités jusqu'à 10000 relevées) ; `published_rate DECIMAL(19,6)` porte les cinq
     décimales des cours journaliers ; `currency` a **la même définition** qu'`accounts.currency` (35-1 AC-1) et
     `journal_entry_lines.currency` (35-3a AC-1), et le `CHECK` porte `BINARY` comme dans ces deux fiches : la table,
     en `utf8mb4_unicode_ci` par défaut, confondrait sinon `usd` et `USD` ;
   - **`exchange_rate_overrides`** (table **applicative**, D9 ; les cours saisis par la société) : `id BIGINT
     AUTO_INCREMENT PRIMARY KEY`, `company_id BIGINT NOT NULL` (FK `companies`, `RESTRICT`), `currency` (même
     définition, `CHECK BINARY currency REGEXP '^[A-Z]{3}$' AND BINARY currency <> BINARY 'CHF'`), `kind` et
     `rate_date` (mêmes définitions et mêmes `CHECK` que ci-dessus), `rate DECIMAL(20,10) NOT NULL` (`CHECK rate > 0`,
     **par unité**, comme `rate_per_unit`), `version INT NOT NULL DEFAULT 1`, `updated_by BIGINT NOT NULL` (FK `users`,
     `RESTRICT`, comme `api_keys`), `created_at`, `updated_at` `DATETIME(3) NOT NULL` ; `UNIQUE (company_id, currency,
     kind, rate_date)`. Inscrite à `TABLES_TO_TRUNCATE` (`crates/kesh-db/src/backup.rs:46`), **avant `companies` et
     `users`** (enfant des deux, sans enfant propre ; place proposée : après `mcp_server_settings`, autre table de
     réglages de société), absente de `RESET_PRESERVED_TABLES` (:136) : un cours saisi est une donnée de la société,
     vidée par la remise à zéro (`reset_partition_is_guarded_by_the_schema` reste vert : la table est aussi enfant de
     `companies`). Sauvegardée dans le `.keshbackup` par l'export générique (`export_table`, colonnes lues de
     `INFORMATION_SCHEMA`) ; export CSV utilisateur : ajoutée à `TABLES_EXPORTEES` (`crates/kesh-api/src/exports/global.rs:116`,
     garde `export_couvre_toutes_les_tables` :673) avec son sérialiseur dans `csv_tables.rs` (cours à 10 décimales,
     **jamais** par `fmt_decimal`, comme 35-3a AC-8), ou à `TABLES_HORS_EXPORT` avec un motif écrit — choix au
     développement, exportée de préférence ;
   - ⚠️ **La fenêtre d'import se referme** (conséquence assumée, CR-1) : `parse_and_verify` (`crates/kesh-api/src/admin_backup/import.rs:175`)
     exige que le manifeste porte **exactement** l'inventaire `TABLES_TO_TRUNCATE` (« table '…' absente du
     manifeste » → 400) : **un `.keshbackup` écrit par la 0.17.6 ou avant ne se réimporte plus** dans la version qui
     porte cette story. À écrire au CHANGELOG (section *Changed*, en tête des notes avec la sauvegarde d'avant la mise à
     jour) et au manuel d'administration ; l'option `--import` de `scripts/recette-montee.sh` (phase 5) **n'est pas jouée à cette release** : elle
     importe le `.keshbackup` de l'image précédente et exige un 200 (`recette-montee.sh:651`, `echec 5` sinon), or la
     version de cette story le refuse en 400 — le script n'est **pas modifié** (la release suivante retrouve une image
     précédente dont les sauvegardes sont importables) ; le journal de la release dit que la phase 5 est omise et
     pourquoi (le retour à la version précédente par le dump et son import dans **l'ancien** binaire, phase 4, reste
     joué). Conséquences au code, **toutes dans cette story** (précédents : Story
     32-5a, création de `mcp_keys` ; Story 25-2-c) :
     - `registry_entries_are_within_import_window` (`crates/kesh-db/src/post_restore.rs:939`) rougit : l'unique entrée
       de `POST_RESTORE_BACKFILLS` (:210), `20261106000001_supplier_invoice_internal_number` (classe B), devient hors
       fenêtre. Elle passe à `RETIRED_BACKFILLS` (:271) et à `EXEMPT_MIGRATIONS` avec la justification « Hors fenêtre
       depuis 20261114000002 (exchange_rate_overrides, Story 35-2) : … », patron de `20261010000001` (:625) ; le registre
       redevient **vide**, et son commentaire d'en-tête (:211-217) le dit ; compteurs codés en dur recomptés au code
       (« Hors fenêtre » 9 → 10, :1140 ; nombre total d'exemptions + 1, :1219 et suivantes), chacun avec sa ligne
       d'historique ;
     - `full_import_report_mirrors_the_production_registry` (`crates/kesh-api/tests/admin_full_import_e2e.rs:1561`) :
       son assertion de montage `!report.is_empty()` (:1595, « le registre de production porte au moins une entrée »,
       revenue à la Story 33-1a) **rougit** avec le registre vide ; elle reprend la forme que la Story 32-5a lui avait
       donnée (`git show b723e3d9:crates/kesh-api/tests/admin_full_import_e2e.rs`, :1589-1596 : rapport vide) et le
       commentaire de tête (:954, « le registre de production a été VIDE de la 32-5a à la 33-1a ») est annoté, sans
       réécrire l'historique ;
     - **deux** tests lisent l'entrée par l'aide `entree_numero_interne` (`crates/kesh-api/tests/admin_full_import_e2e.rs`
       :5551, qui cherche l'entrée dans le rapport du registre de production et **panique** — `.expect("le rapport porte
       la numérotation interne")` — quand il est vide) : `full_import_of_a_0_17_4_backup_gives_the_numbers` (:5563,
       appel :5619) **et** `full_import_of_a_0_17_5_backup_keeps_the_numbers` (:5638, appel :5669). Les deux passent au
       patron `replay_retired` (:972), appelé après `import_ok` et **avant** les assertions sur les numéros (sans registre,
       l'import seul ne pose plus les numéros de la sauvegarde 0.17.4) ; l'aide lit alors l'entrée dans le rapport de
       `replay_retired` (issues attendues inchangées : `REPLAYED_SENTINELS_ABSENT`, 3 lignes, pour la 0.17.4 ; `SKIPPED`,
       0 ligne, pour la 0.17.5) ;
     - le test d'AC-6 de 35-1 (« un `.keshbackup` écrit par la 0.17.6 se réimporte ») devient, ici, un test de **refus**
       (400, table `exchange_rate_overrides` absente du manifeste) ; l'aller-retour d'une sauvegarde de la version
       courante reste vert ;
   - `_kesh_exchange_rate_fetch` singleton : `id TINYINT UNSIGNED PRIMARY KEY` (`CHECK (id = 1)`),
     `triggered_by ENUM('auto','manual') NOT NULL` (**pas
     `trigger`** : mot réservé de MariaDB ; le champ JSON de l'API s'appelle `trigger`, lu depuis cette colonne),
     `last_run_at DATETIME(3) NOT NULL` (UTC), `last_status VARCHAR(16) NOT NULL` (`CHECK IN ('ok','error','not_published')`),
     `last_error VARCHAR(64) NULL` — **un code
     classé**, jamais le texte d'une erreur `reqwest` (qui porte l'URL) : `timeout`, `connect`, `http_status:<n>`,
     `too_large`, `malformed`, `month_mismatch`, `date_mismatch`, `no_valid_rate`, `invalid_rates:<n>` —,
     `last_kind VARCHAR(5) … NOT NULL` (`month` | `day`, même `CHECK`) et `last_rate_date DATE NOT NULL` (le 1er du mois
     ou le jour demandé). Le tour
     d'une récupération écrit **une** ligne : `ok` (avec `last_error = invalid_rates:<n>` si des devises ont été
     écartées), `not_published` ou `error` ; un tour de la boucle qui traite deux mois écrit le **pire** des deux
     statuts (`error` > `not_published` > `ok`) et `last_rate_date` du mois en cause (à égalité, le plus récent). **Ligne créée
     paresseusement** (aucune écriture de données dans la migration, P7 sans objet ; le dump `--no-data` de
     `regen-test-schema.sh` n'a rien à réinjecter), écrite par `INSERT … ON DUPLICATE KEY UPDATE` ;
   - les deux tables `_kesh_*` inscrites à `TABLES_SYSTEME` (`crates/kesh-db/src/backup.rs` :949, lue par
     `backup_inventory_matches_schema` et `convention_du_prefixe_des_tables_systeme`), absentes de l'inventaire
     `.keshbackup` ; `exchange_rate_overrides` à `TABLES_TO_TRUNCATE` (ci-dessus) ; `migrations.sha384`, ligne et
     compteurs recomptés de `docs/migrations-idempotence-audit.md` (P5), squash régénéré, P6, P7 (DDL seul :
     `every_data_backfill_migration_is_triaged` n'a rien à trier pour ce fichier ; c'est la **fenêtre**, non le triage,
     qui bouge) ;
   - test de montée sur le modèle d'`installation_settings_adds_only_its_system_table`
     (`crates/kesh-db/tests/migrations_upgrade_path.rs:1766`), monté **par version**
     (`common::migrations_before(VERSION, LABEL)`, `tests/common/mod.rs:75`, P6) : deux tables système neuves, vides,
     et **exactement une** table applicative ajoutée, `exchange_rate_overrides`, vide (l'assertion
     `application_tables(&pool).await == before` du modèle devient « `before` plus cette table ») ; aucune ligne
     existante modifiée.
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
   - **Cours journaliers** (D10) : `parse_daily_rates(xml) -> Result<DailyRates>` (date lue `<datum>` au format
     `JJ.MM.AAAA`, mêmes listes `{currency, published_unit, published_rate, rate_per_unit}` et `rejected`) ; racine
     `wechselkurse`, lecture par noms locaux (espace de noms `…/xmldaily`), éléments inconnus ignorés (`zeit`,
     `gueltigkeit`, `land_*` dont `land_en`). **Une seule** analyse des `<devise>` (unité, cours, tolérance par devise,
     bornes, arrondi à 10 décimales) partagée par les deux fonctions (DRY) : seules la racine et la date diffèrent.
     Refus du jour entier : XML mal formé, `<datum>` absent ou illisible, aucune devise valide ; racine vide →
     `NotPublished`. Le contrôle `<datum>` ≤ D et D − `<datum>` ≤ 7 jours (`date_mismatch`) est celui de l'appelant
     (`kesh-api`, comme `month_mismatch`).
   - **Bornes d'un cours seul** (propriétaire : cette story, qui précède 35-3a et 35-5) : fonction publique de
     `kesh-core` (nom au développement, p. ex. `exchange_rates::check_rate_bounds`) — décimal > 0, au plus 10 décimales
     **après `normalize()`** (zéros terminaux ignorés), ≤ 10⁶. Elle rend une **erreur de type propre**, neutre (p. ex.
     `RateBoundsError { NotPositive, Scale, TooLarge }`), **jamais** une variante de `CoreError` liée à un code : chaque
     appelant la traduit — le `PUT` d'un cours saisi (AC-5) et 35-5 en `FX_RATE_INVALID`, 35-3a en
     `FX_VALUE_OUT_OF_RANGE { line }` (35-3a AC-3, 35-3a-CR1-P1-IF1), le parseur en devise écartée (filtre
     `rate_per_unit`) ; 35-3a l'appelle depuis son contrôle par ligne (AC-3) et 35-5 pour les cours de
     sa requête (AC-4) — **une seule** fonction.
   - Fixtures versionnées **synthétiques** (forme du XML réel, `xmlns` compris, valeurs inventées — aucun cours officiel
     n'est versionné : conditions d'utilisation SIX), une mensuelle et une journalière, avec unités 1, 100, 1000 et
     10000, un cours à 0.0009 et, pour la journalière, un cours à cinq décimales ; tests du cas vide, des refus du mois
     ou du jour et des écarts par devise.
3. **AC-3 — Source des cours et client HTTP** (`kesh-api`, module `exchange_rates`).
   - **Trait `RateSource`** (`fn fetch_month(&self, year, month) -> Pin<Box<dyn Future<Output = Result<String,
     SourceError>> + Send + '_>>` **et** `fn fetch_day(&self, date: NaiveDate) -> …` de même forme (`xmldaily?d=AAAAMMJJ`,
     D10) : **objet-safe sans `async_trait`**, comme `Mailer` (type `MailFuture`, `mail/mod.rs:33`, trait `:69`) — un
     `async fn` dans un trait n'est pas utilisable en `dyn`), porté par `AppState` (`rate_source: Arc<dyn
     RateSource>`, précédent `mailer: Arc<dyn Mailer>` / `test_mock_mailer`, `lib.rs:84-98`, et `BackupEnv` de la
     sauvegarde automatique ; `AppState::new_for_tests` reçoit une source factice par défaut).
   - **Champs neufs d'`AppState`** : `rate_source`, `rate_fetch_lock: Arc<tokio::sync::Mutex<()>>`, `rate_fetch_memo`
     (la mémoire des 15 minutes de l'AC-5, `Arc<std::sync::Mutex<HashMap<(RateKind, NaiveDate), …>>>`, clé = genre et
     `rate_date` : le 1er du mois ou le jour) et
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
     servie : XML, vide, erreur, délai, **par genre** `month` | `day` ; **XML synthétique par défaut** tant que rien
     n'est piloté, **généré pour le mois ou le jour demandé** (`<monat>` = `(j, m)` de l'appel, `<datum>` = la date
     demandée : un XML figé rendrait `month_mismatch` ou `date_mismatch` pour tout autre ; un XML piloté est servi tel
     quel, ce qui permet de servir un `<datum>` antérieur pour un dimanche) ; cours journaliers par défaut **distincts**
     des mensuels (pour que les tests distinguent le repli de 35-5) ; valeurs du défaut fixées et écrites au module, que les specs
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
     se prend **par mois ou par jour récupéré** (le genre et la date forment l'unité de récupération, comme la clé de la mémoire), jamais pour un tour entier de la boucle (qui en traite deux : 2 × 20 s) : l'attente
     d'un `POST /fetch` est alors bornée par **deux** requêtes sortantes au plus devant la sienne (le mois en cours de la
     boucle et un autre `POST /fetch` — le verrou tokio est équitable, premier arrivé premier servi), soit 3 × 20 s dans
     le pire cas, plus les écritures — borne valable pour **un** autre demandeur : avec plus de demandeurs simultanés
     (plusieurs onglets), elle s'allonge d'autant ; pour la tenir, un `POST /fetch` qui a **attendu** le verrou et
     trouve, une fois le verrou pris, le mois (ou le jour) écrit par une récupération **commencée après son arrivée** (`fetched_at`,
     pris par la récupération **avant** son appel sortant, postérieur à l'instant d'arrivée du `POST`) rend ce
     résultat (200, `count` des devises présentes, `rejected` 0) sans rappeler la source, sans ligne d'audit et sans
     écriture de `_kesh_exchange_rate_fetch` (il n'a rien récupéré). Sous le
     verrou, le `POST /fetch` relit la mémoire de l'AC-5 avant d'appeler la source (un mois ou un jour **présent** se redemande,
     c'est la demande : la mémoire ne retient que `not_published` et les échecs) ; **un succès retire l'entrée** du
     mois (ou du jour) de la mémoire (sans quoi un échec mémorisé rendrait 503 pendant 15 minutes sur un mois que la boucle vient
     d'écrire). La boucle juge « manque » (AC-4) **sous le verrou**, non avant : sinon elle redemanderait un mois que
     le `POST /fetch` devant elle vient d'écrire. Le travail du `POST /fetch` (verrou, appel, transaction, état,
     audit) tourne dans un `tokio::spawn` attendu par le handler, comme « Sauvegarder maintenant »
     (`run_backup_now`, `routes/backup_schedule.rs:159`) : l'abandon du client (délai de 90 s, onglet fermé) n'annule
     pas une récupération entre le `COMMIT` des cours et l'audit. Le client du frontend appelle `POST /fetch` avec `timeoutMs: 90_000` (`CallOptions`, `api-client.ts:61` ; le
     défaut de 30 s, `:42`, et même 60 s ne couvrent pas ce pire cas : ils rendraient `TIMEOUT`, lu comme issue
     inconnue) ; écriture en **une transaction** : `INSERT … ON DUPLICATE KEY UPDATE` des devises valides du mois ou du jour, dans `_kesh_exchange_rates` **seulement** ; un échec
     n'écrit aucun cours. ⛔ **Une récupération ne lit ni n'écrit jamais `exchange_rate_overrides`** (D9 : un cours
     saisi n'est jamais remplacé) ; test : un cours saisi pour (USD, mois) reste, avec sa `version`, après un
     `POST /fetch` du même mois qui change le cours AFC, et le cours effectif reste le cours saisi.
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
   Non auditée (C-35-6). **Seule la boucle** écrit `triggered_by = 'auto'`. **La boucle ne récupère que des mois** :
   un cours journalier (D10) ne se récupère qu'à la demande (`POST /fetch { date }`, AC-5), par l'écran *Cours de
   change* ou par l'aperçu de la réévaluation (35-5) ; « manque » ne compte que les lignes `kind = 'month'`. Elle ne
   touche jamais `exchange_rate_overrides`.
5. **AC-5 — API.**
   - **Cours effectif** (D9) : pour une société, une devise, un genre et une date, **cours saisi
     (`exchange_rate_overrides`) s'il existe, sinon cours AFC brut (`_kesh_exchange_rates`)**. Une seule fonction de
     dépôt le résout (nom au développement, p. ex. `exchange_rates::find_effective_rate_in_tx(conn, company_id,
     currency, kind, rate_date) -> Result<Option<EffectiveRate>, DbError>`, `EffectiveRate { effective_rate,
     afc: Option<ExchangeRate>, override_: Option<RateOverride> }`, `&mut MySqlConnection` comme `find_rate_in_tx`) :
     toutes les lectures ci-dessous, 35-3b et 35-5 passent par elle ; aucun autre code ne compose les deux tables.
   - `GET /api/v1/exchange-rates?month=AAAA-MM[&currency=USD]` (cours mensuels) **ou** `?date=AAAA-MM-JJ[&currency=USD]`
     (cours journaliers ; `month` et `date` ensemble ou ni l'un ni l'autre → 400) : toutes les devises qui ont un cours
     AFC **ou** un cours saisi pour ce mois ou ce jour (union), ou une seule ;
   - `GET /api/v1/exchange-rates/status` : état de `_kesh_exchange_rate_fetch` (`trigger`, date, statut, code
     d'erreur, genre et date du cours) ;
   - `POST /api/v1/exchange-rates/fetch { year, month }` (cours mensuel) **ou** `{ date: "AAAA-MM-JJ" }` (cours
     journalier, D10 ; les deux formes ensemble ou aucune → 400) — dans **`comptable_routes`** (Administrateur ou
     Comptable ; mois ≥ 2000-01 et ≤ mois courant en Europe/Zurich ; jour ≥ 2000-01-01 et **strictement antérieur** au
     jour courant en Europe/Zurich, cf. *Cours journaliers* — même aide de fuseau que l'AC-4, qui rend le jour et dont
     `mois_courant_zurich` se déduit ; sinon 400) : récupère ce mois ou ce jour, **auditée sous l'utilisateur**
     (`NewAuditLogEntry::from_current_user`, méthode du trait `AuditActor`, `audit.rs:24` : une clé d'API est attribuée comme telle ;
     `exchange_rates.fetched`, genre, mois ou jour, `<datum>` lu pour un jour, nombre de devises écrites, écartées et dont le cours a changé (`changed`) — écrite **sur un succès seulement** : un
     404 ou un 503 n'écrit que `_kesh_exchange_rate_fetch`) ; écrit `_kesh_exchange_rate_fetch` avec
     `triggered_by = 'manual'` ; réponses `200 {kind, rateDate: "AAAA-MM-JJ", sourceDate|null, count, rejected}` (`count` et `rejected` sont des
     **nombres** ; les codes écartés vont au journal du serveur), `404 RATE_NOT_PUBLISHED`,
     `503 RATE_SOURCE_UNAVAILABLE` (code explicite, message sans URL ni trace — **jamais 502** : le frontend lit 502 et
     504 comme une issue inconnue venue d'un proxy, `frontend/src/lib/shared/utils/unknown-outcome.ts`). ⚠️ Mais
     **tout 503** allume aujourd'hui le bandeau « service dégradé » (`apiHealth.setDegraded()`,
     `api-client.ts:393-394`, avant toute lecture du corps), conçu pour une base injoignable : la fonction d'API du
     frontend qui appelle `POST /fetch` (seule entrée, partagée par cet écran, 35-3b et 35-5) passe une option neuve
     de `CallOptions` qui ne l'allume pas (`degradedOn503: false`, nom au développement), avec son test Vitest. ⚠️ Elle
     ne couvre que le **503** : un `TIMEOUT` ou une coupure du client allume le même bandeau par la branche `catch`
     (`api-client.ts:421-422`, `apiHealth.setDegraded()`), ce qui reste voulu (le serveur ne répond pas) — à dire dans le
     test, qui ne doit pas affirmer le contraire. **Tout**
     échec de récupération se rend ainsi — `timeout`, `connect`, `http_status`, `too_large`, mais aussi `malformed`,
     `month_mismatch`, `date_mismatch` et `no_valid_rate` (la source a répondu, inutilisablement) : jamais 500 ; le code classé va à
     `last_error`, pas à l'utilisateur. Un mois ou un jour `not_published` ou en échec n'est pas redemandé à la source avant 15
     minutes (`rate_fetch_memo`, clé `(genre, rate_date)`, alimentée aussi par la boucle) : la réponse d'origine est
     rendue, sans nouvelle ligne d'audit ni écriture de `_kesh_exchange_rate_fetch` ;
   - `GET /api/v1/exchange-rates/lookup?currency=USD&date=AAAA-MM-JJ[&kind=month|day]` (`kind` absent = `month`) —
     **en lecture seule**, sans appel sortant ni écriture ni audit, ouverte à tout rôle authentifié : **cours effectif**
     du mois de la date (`month`) ou de la date elle-même (`day`) —
     `200 { currency, kind, rateDate: "AAAA-MM-JJ", effectiveRate, afcRate, overridden, publishedUnit, publishedRate,
     sourceDate }` (décimaux en chaînes ; `afcRate`, `publishedUnit`, `publishedRate` à `null` sans cours AFC,
     `sourceDate` à `null` hors `day` ; `overridden` vrai si un cours saisi existe ; forme lue par 35-3b et 35-5 ; le
     champ `ratePerUnit` de la forme d'avant CR-1 n'existe plus) —, ou `404 RATE_UNAVAILABLE` quand il n'y a **ni**
     cours saisi **ni** cours AFC, `details: { currency: "USD", month: "AAAA-MM-01" }` pour `month` (jamais 1.0) —
     **forme unique** de ce code pour un cours mensuel, que le `DbError::RateUnavailable` de 35-3a rend à l'identique,
     et d'où l'écran tire `{year, month}` du `POST /fetch` ; pour `day`, la même forme **plus** `date: "AAAA-MM-JJ"`
     (`month` = le 1er du mois de cette date), d'où l'écran tire `{ date }` du `POST /fetch`. L'écran de saisie (35-3b) et la réévaluation (35-5)
     appellent `POST /fetch` sur ce 404 (rôles Administrateur et Comptable), puis relisent ; sans cours, saisie
     manuelle. Devise `CHF` (devise principale, sans cours) → 400 `Validation`, sans code dédié ; devise absente de la liste ISO 4217 de 35-1
     (`kesh-core::currency`) → 400 `CURRENCY_UNKNOWN` (code de 35-1) ; `month` ou `date` absents ou illisibles (les deux `GET`) → 400 ;
     `kind` hors `month|day` → 400. Un cours saisi sans cours AFC rend 200 (`afcRate: null`) : aucun `POST /fetch`
     n'est alors nécessaire. **Forme `day`** : `DbError::RateUnavailable { currency, month }` (35-3a, 35-5) ne porte
     pas de date ; le handler de `lookup` pour `kind=day` construit donc **directement** l'`AppError` 404 (`details`
     avec `date`), sans passer par `DbError`, dont la variante reste à deux champs.
   - **Cours saisis par la société** (D9) — deux routes dans **`comptable_routes`** (Administrateur ou Comptable ;
     Consultation 403), **auditées** sous l'utilisateur (`from_current_user`) **dans la transaction** de l'écriture :
     - `PUT /api/v1/exchange-rates/overrides { currency, kind: "month"|"day", rateDate: "AAAA-MM-JJ", rate, version? }`
       pose ou remplace le cours saisi : `rateDate` le 1er du mois pour `month` (sinon 400), 2000-01-01 ≤ `rateDate`
       ≤ 2099-12-31 (sinon 400) ; `rate` en chaîne décimale, **par unité** (1 JPY, jamais 100 JPY), contrôlé par la
       fonction des bornes d'un cours seul (AC-2) → 400 **`FX_RATE_INVALID`**, `details: { currency }` (code **créé
       ici**, avec sa variante `AppError` et sa traduction dans les quatre locales ; 35-5 le **réutilise** pour les
       cours de sa requête) ; `currency` comme aux `GET` (`CHF` → 400, inconnue → `CURRENCY_UNKNOWN`, lue telle
       quelle). Concurrence optimiste : `version` absent = création (une ligne déjà présente → 409
       `OPTIMISTIC_LOCK_CONFLICT` ; **y compris** deux créations simultanées : la violation de l'unicité `(company_id, currency, kind, rate_date)` est rattrapée et rendue 409, jamais 500) ; `version` présent = remplacement de cette version (ligne absente ou autre version
       → 409 `OPTIMISTIC_LOCK_CONFLICT`) ; `version` + 1 à chaque remplacement. Rend `200` avec la forme de `lookup`
       (plus `override: { id, version, updatedAt }`). Un cours saisi **égal** au cours AFC est conservé tel quel (il ne
       suivra pas une republication de l'AFC tant qu'il n'est pas retiré par ↑, AC-6) ;
     - `DELETE /api/v1/exchange-rates/overrides?currency=USD&kind=month&rateDate=AAAA-MM-JJ&version=N` retire le cours
       saisi (bouton ↑ de l'écran : « reprendre le cours AFC ») ; `version` **obligatoire** (absente ou illisible → 400) ; `204` ; ligne absente → 404 `NOT_FOUND` ; autre
       version → 409 `OPTIMISTIC_LOCK_CONFLICT`. Le cours effectif redevient le cours AFC, ou aucun ;
     - audit : actions `exchange_rate_override.created`, `exchange_rate_override.updated`,
       `exchange_rate_override.deleted` (précédent `company_invoice_settings.created`/`.updated`), type d'entité neuf
       `exchange_rate_override` (`ENTITY_TYPES`, `audit_labels.rs:24`, liste triée : entre `email_template` et
       `export`), identifiant = `exchange_rate_overrides.id`, détails `{ currency, kind, rateDate, before, after,
       afcRate }` (cours en chaînes) ; clés `audit-log-action-exchange-rate-override-{created,updated,deleted}` et
       `audit-log-entity-exchange-rate-override` dans les quatre locales ;
     - ⛔ un cours saisi **ne réécrit aucune écriture** : les lignes déjà passées gardent leur cours (35-3a) ; seul le
       cours **proposé** des saisies suivantes change (35-3b, 35-5).
   - Toutes ces routes exigent le mode multi-devises de la société courante : **`409 MULTI_CURRENCY_DISABLED`**, même
     statut que dans la 35-1.
   - Nouvelles variantes `AppError` (`errors.rs`) et leur traduction côté frontend pour `RATE_NOT_PUBLISHED`,
     `RATE_SOURCE_UNAVAILABLE`, `RATE_UNAVAILABLE`. **Propriétaire unique de `RATE_UNAVAILABLE`** : cette story (elle
     précède 35-3a) crée la variante `DbError::RateUnavailable { currency, month }` (`crates/kesh-db/src/errors.rs`,
     `enum DbError` :668) **et** son bras de la correspondance `DbError` → `AppError` (`crates/kesh-api/src/errors.rs`,
     404, `details` ci-dessus) ; le handler de `lookup` s'en sert
     sur `find_effective_rate_in_tx` → `None`. 35-3a et 35-5 la **réutilisent**, sans la redéfinir (35-5 la rend pour
     un cours mensuel, la forme `month` ; la forme `day` n'est rendue que par `lookup`).
   - **Formes des réponses** (camelCase, décimaux en chaînes, comme `lookup`) : `GET /exchange-rates` →
     `{ kind, rateDate: "AAAA-MM-JJ", rates: [{ currency, effectiveRate, afcRate, overridden, publishedUnit,
     publishedRate, sourceDate, fetchedAt, override: { id, version, updatedAt } | null }] }` (`afcRate`, `publishedUnit`, `publishedRate`, `sourceDate` et `fetchedAt` à `null` pour une devise qui n'a qu'un cours saisi ; tri par devise ; liste
     vide si ni cours AFC ni cours saisi pour ce mois ou ce jour) ; `GET /exchange-rates/status` → `{ trigger,
     lastRunAt, status, error, kind, rateDate }`, tous `null` tant qu'aucune récupération n'a eu lieu (la ligne est
     créée paresseusement, AC-1).
     `fetchedAt` et `lastRunAt` sont **stockés en UTC** (`Utc::now().naive_utc()`, comme `auto_backup`) et **affichés en
     heure locale suisse** par l'écran (leçon de #889).
   - **Ordre des refus** (même ordre que 35-1/35-3a) : authentification et rôle (401, 403) → société en mono-devise
     (`409 MULTI_CURRENCY_DISABLED`) → paramètres illisibles ou hors bornes (400, dont `CHF`) → devise inconnue
     (`400 CURRENCY_UNKNOWN`) → cours absent (404) ; pour le `PUT` d'un cours saisi, après la devise inconnue :
     `400 FX_RATE_INVALID`, puis la version (`409 OPTIMISTIC_LOCK_CONFLICT`). La devise se lit **telle quelle**, sans normalisation : `usd` →
     `400 CURRENCY_UNKNOWN`.
   - **Audit et registres** : action `exchange_rates.fetched` ajoutée à `ACTIONS` (`crates/kesh-api/src/audit_labels.rs`,
     liste triée, garde `tests/audit_label_registry.rs`) ; type d'entité `installation` avec `AUDIT_ENTITY_ID_NONE`
     (précédent `admin.backup_run`, `routes/backup_schedule.rs:177-191`, `emit_audit` après coup — la table est
     globale et `audit_log.entity_id` est `NOT NULL`) ; clé `audit-log-action-exchange-rates-fetched` dans les quatre
     locales. `POST /fetch`, `PUT` et `DELETE /exchange-rates/overrides` entrent dans `LIB_ROUTES`
     (`tests/audit_route_registry.rs`, chacune `Traced, SansEcritureAuJournal` — aucune n'écrit au journal comptable ;
     précédent `company_invoice_settings::update_invoice_settings`, :387) :
     `the_registry_partition_is_what_the_story_declares` gagne **+3 routes et +3 tracées** (état au 2026-11-14 : 146 routes,
     139 tracées, total registre 149 ; **ces chiffres s'écrivent en deltas** — la 35-1 et la 35-5 ajoutent aussi leur
     route mutante, l'ordre d'intégration décide du chiffre absolu, à recompter au code), et le total registre **+4**
     avec l'endpoint `_test` (entrée `("post", "/exchange-rates/source", Exempt(…), SansEcritureAuJournal)` de
     `TEST_ENDPOINT_ROUTES`, sur le modèle de `/password-reset-token`, `audit_route_registry.rs:620` : il n'écrit
     rien au journal d'audit ; `Exemptee` signifie « écrit au journal et n'est pas rejouée », réservé à `full_import`, `/seed`,
     `/reset`) ; la phrase du manuel d'administration (`admin-manual.tex:3217`, « N des M routes à verbe
     mutant » et « N + 4 + 3 = M », gardée par `le_manuel_dit_le_compte_du_registre`) suit le chiffre recompté.
     Le second test, `the_replay_partition_is_what_the_story_declares` (`audit_route_registry.rs:1450`), compte aussi :
     `sans_ecriture` **+4** (le `POST /fetch`, le `PUT` et le `DELETE` des cours saisis, et l'endpoint `_test`),
     `exemptees` et `rejouees` inchangés, `tout.len()` +4 (:1502) ; commentaires
     et `//!` du module recomptés, en deltas. *(Alignement de l'orchestrateur sur 35-1-P4-IF-1.)*
6. **AC-6 — Écran** *Paramètres → Cours de change* (visible en multi-devises) : deux vues, **mensuelle** (choix du
   mois) et **jour** (choix d'une date ; date proposée par défaut : la date de fin du dernier exercice terminé avant
   aujourd'hui, la date de clôture que lit la réévaluation, 35-5 ; toute date admise) ; tableau (devise, unité
   publiée, cours publié, **cours par unité éditable**, date de récupération et, en vue jour, date de publication
   `sourceDate` quand elle diffère de la date choisie), filtre par devise, bouton « Récupérer ce mois » / « Récupérer
   ce jour » (Administrateur et Comptable seulement), état de la dernière récupération (automatique ou manuelle,
   d'après `trigger` ; genre, date, statut, code d'erreur traduit) ; mention de la source et de son caractère non
   contraignant ; libellés « cours mensuel AFC » et « cours journalier », jamais « cours moyen ». Carte sur
   `settings/+page.svelte`.
   - **Cours éditable** (D9) : chaque cours effectif est un champ (Administrateur et Comptable ; lecture seule pour
     Consultation), validé à l'écran par les bornes de l'AC-2 (`AmountFormatOptions { maxDecimals: 10,
     maxIntegerDigits: 7 }`, comme 35-3b AC-2) ; une valeur changée s'enregistre à la validation du champ (`PUT`, avec
     la `version` du cours saisi s'il existe) ; un `409 OPTIMISTIC_LOCK_CONFLICT` recharge la vue avec un message.
     **Sous le champ**, pour **tout cours saisi existant** — qu'il diffère du cours AFC, lui soit égal ou qu'il n'y ait
     pas de cours AFC (D9 : un cours saisi doit toujours pouvoir être retiré, et seul ↑ le retire) — la mention
     « AFC : 0.8846 » (ou « AFC : non publié ») et un bouton **↑** qui appelle le `DELETE` ; libellé accessible
     « Reprendre le cours AFC », ou « Retirer le cours saisi » sans cours AFC (la ligne disparaît alors de la vue). Un
     cours saisi **égal** au cours AFC est en outre marqué « saisi » : il ne suit pas une republication de l'AFC tant
     qu'il n'est pas retiré par ↑. Sans cours saisi, ni ↑ ni marque. Une devise sans aucun
     cours n'a pas de ligne (le cours se saisit alors par un champ « Ajouter un cours » : devise de la liste ISO 4217,
     cours). Une récupération qui change le cours AFC laisse le cours saisi et met à jour la mention « AFC : … ». Un champ **vidé** ne supprime rien : la valeur affichée est restaurée, sans appel (seul ↑ retire un cours saisi) ; une valeur égale au cours effectif affiché n'envoie rien. En vue jour, « Récupérer ce jour » est **désactivé** pour une date ≥ aujourd'hui (Europe/Zurich), avec la raison affichée (le serveur la refuse, AC-5) ; la date proposée par défaut est, **sans exercice terminé**, la veille (Europe/Zurich).
   - Vitest : ↑ présent pour tout cours saisi (différent, égal au cours AFC — marqué « saisi » —, sans cours AFC —
     libellé « Retirer le cours saisi ») et absent sans cours saisi ; ↑ → `DELETE` avec la bonne version ; 409 → rechargement ;
     Consultation sans champ éditable ; cours JPY saisi par unité. Captures d'écran (vue mensuelle avec un cours saisi
     et son ↑, vue jour avec `sourceDate` d'un dimanche, lecture seule).
7. **AC-7 — Restauration.** Les deux tables `_kesh_*` sont système : un import de sauvegarde ne les touche pas. Sur une
   installation neuve restaurée, elles sont vides : la boucle ou `POST /fetch` les complète ; aucun test ne suppose un
   cours présent sans l'avoir posé. **`exchange_rate_overrides`, applicative, est sauvegardée et restaurée** avec la
   société : après une restauration, les cours saisis sont là et sont les cours effectifs, même avant toute
   récupération AFC (test d'aller-retour : cours saisi à 10 décimales, `version`, `updated_by` identiques). Un
   `.keshbackup` antérieur à cette story est refusé (AC-1). ⚠️ **Isolation des E2E** : système, elles échappent à `truncate_all`
   (`TABLES_TO_TRUNCATE`, `backup.rs:46`) que lancent `POST /_test/seed` et `/_test/reset`
   (`test_endpoints.rs:189`, `:325`) ; sans plus, un cours posé par une spec survit à la suivante, qui attendrait
   un `404 RATE_UNAVAILABLE`. (`exchange_rate_overrides` est dans `TABLES_TO_TRUNCATE` : vidée sans autre geste.) Ces
   deux routes vident donc aussi `_kesh_exchange_rates` et
   `_kesh_exchange_rate_fetch`, et `clear_transient_test_state` (`test_endpoints.rs:349`) vide `rate_fetch_memo` et
   remet la source factice à son défaut. Test Rust de ce vidage. Le commentaire de `test_fixtures.rs` (:365-373), qui liste
   les tables système hors `TABLES_TO_TRUNCATE`, gagne les deux tables neuves. La remise à zéro des données de
   démonstration (`kesh_seed::reset_demo`, `crates/kesh-seed/src/lib.rs:510`, audit `installation.reset`) ne vide
   que les tables applicatives : elle **laisse** les cours AFC, données officielles de l'installation, et **vide**
   les cours saisis, données de la société — voulu, à dire au manuel. Aucun outil du serveur MCP ne lit la table des cours (une ligne porte son cours, 35-3a) : hors périmètre.
8. **AC-8 — i18n, manuels.** Quatre locales (écran, codes d'erreur dont `FX_RATE_INVALID`, libellés d'audit et de
   type d'entité) ; manuel utilisateur (écran, source, cours mensuel AFC fixé d'avance, cours journalier, cours saisi
   et bouton ↑, récupération) ; manuel d'administration : accès sortant HTTPS vers `www.backend-rates.bazg.admin.ch`
   requis en multi-devises, aucun en mono-devise, mandataire par `HTTPS_PROXY` (lu par `reqwest`) ; **un
   `.keshbackup` 0.17.6 ou antérieur ne se réimporte pas dans cette version** (AC-1) ; phrase du registre des routes
   (AC-5) ; PDF régénérés.
9. **AC-9 — Cours manuels** (amendé par D9). Deux niveaux, sans recouvrement :
   - **cours saisi par la société** pour (devise, mois) ou (devise, jour) : table applicative
     `exchange_rate_overrides`, jamais `_kesh_exchange_rates` (qui ne porte que les cours AFC bruts) ; audité à sa
     pose et à son retrait ; il devient le cours **proposé** ;
   - **cours d'une ligne** : il vit **sur la ligne d'écriture** (35-3a), audité avec l'écriture. Une ligne au cours
     effectif est `afc` si ce cours égale le cours AFC mensuel du mois (`afcRate`), `manual` sinon (cours saisi qui
     diffère, ou cours AFC absent) — c'est le client qui la qualifie (35-3b) ; le contrôle `FX_RATE_NOT_AFC` de 35-3a
     compare toujours au cours AFC **brut**, jamais au cours saisi. Aucune nouvelle valeur de `rate_source`.

## Tâches

- [ ] Migration + inscriptions (AC-1), dont `exchange_rate_overrides` dans `TABLES_TO_TRUNCATE`, `TABLES_EXPORTEES`
      (ou exclusion motivée) ; fenêtre d'import refermée : registre de rejeu vidé, `20261106000001` passé à
      `RETIRED_BACKFILLS` et `EXEMPT_MIGRATIONS` (« Hors fenêtre depuis 20261114000002 »), compteurs recomptés,
      tests d'import 0.17.4 et 0.17.5 au patron `replay_retired`, test de 35-1 AC-6 inversé en refus.
- [ ] `kesh-core::exchange_rates` (AC-2 : deux analyses, une seule analyse des `<devise>`, bornes d'un cours seul) ;
      `kesh-db::repositories::exchange_rates` (upsert en transaction, lecture, état, cours saisis). La lecture d'un
      cours AFC **brut** — **nom fixé ici, repris tel quel par 35-3a** :
      `find_rate_in_tx(conn: &mut MySqlConnection, currency: &str, kind: RateKind, rate_date: NaiveDate) ->
      Result<Option<ExchangeRate>, DbError>` (`None` = cours absent ; ne lit **jamais** `exchange_rate_overrides`) ; la
      lecture du **cours effectif** — `find_effective_rate_in_tx(conn, company_id, currency, kind, rate_date)` (AC-5),
      seule à composer cours saisi et cours AFC, reprise par `lookup`, les `GET`, 35-3b (par `lookup`) et 35-5. Toutes
      deux prennent une **connexion ou une transaction** (`&mut
      MySqlConnection`, comme les fonctions `*_in_tx` du dépôt) : 35-3a (AC-4, 4) lit `find_rate_in_tx(…, Month, 1er
      du mois)` dans la transaction de l'écriture ; 35-5 lit les cours effectifs **avant** sa transaction, sur une
      connexion du pool (35-5 AC-2 : aucune lecture de cours ni récupération sous le verrou de l'exercice) — aucun
      second accès. Écriture d'un cours saisi : `upsert_override` / `delete_override` en transaction avec leur audit,
      version contrôlée. L'upsert d'un mois déjà présent qui **change** un
      `rate_per_unit` existant le journalise (`tracing::warn!`, devise, ancien et nouveau cours) et le compte
      (`changed` dans le détail d'audit de `POST /fetch`) : les lignes `afc` déjà passées gardent leur cours, mais
      leur `PUT` serait refusé `FX_RATE_NOT_AFC` (35-3a) — l'écart ne doit pas être muet.
- [ ] `kesh-api` : trait `RateSource`, `HttpRateSource`, source factice et endpoint `_test` (AC-3), boucle (AC-4,
      `main.rs`), routes (AC-5, `lib.rs` : `GET` dans `authenticated_routes`, `POST /fetch`, `PUT` et `DELETE
      /exchange-rates/overrides` dans `comptable_routes`), variantes `AppError` (dont `FX_RATE_INVALID`), audit
      (`audit_labels.rs` : trois actions et un type d'entité), registre des routes et phrase du manuel.
- [ ] Frontend : page `settings/exchange-rates` (vues mensuelle et jour, champs éditables, mention AFC et ↑), carte
      des paramètres, types, API (fonctions `lookup`, `fetch`, `putOverride`, `deleteOverride` ; la forme de `lookup`
      est celle de l'AC-5), codes d'erreur, Vitest (AC-6).
- [ ] Tests : parseur (fixture : unités 1/100/1000/10000, devise écartée, mois refusé) ; client contre un serveur HTTP
      local (délai, 500, redirection non suivie, XML tronqué, corps trop gros, mois différent → rien d'écrit) ;
      `un_tour` sauté en mono-devise ; `un_tour` contre une source injoignable rend sans paniquer, écrit `error` et
      `triggered_by = 'auto'`, ne touche aucun cours ; `lookup` n'écrit rien et ne sort pas ; `POST /fetch` refusé au rôle
      Consultation, audité, `triggered_by = 'manual'`, 409 `MULTI_CURRENCY_DISABLED` en mono-devise (les quatre routes),
      400 hors bornes de mois, mémoire de 15 minutes (un mois `not_published` n'est pas redemandé à la source : un
      seul appel sortant, aucune ligne d'audit, une seule écriture de `_kesh_exchange_rate_fetch`), `malformed` / `month_mismatch` / `no_valid_rate` → 503 sans rien écrire, `lookup` `CHF` → 400,
      `mois_courant_zurich` aux bords du mois, devise aux bornes hors `DECIMAL` écartée sans annuler les autres,
      `last_status` d'un tour à deux mois (le pire) ; aucune trace d'audit en automatique ; deux `POST /fetch` simultanés
      sérialisés (le second attend la fin du premier, AC-3 ; un seul appel sortant à la fois) ; `POST /_test/seed` et `/reset` vident les deux tables et
      la mémoire (AC-7) ; source factice : XML par défaut au mois demandé ; un `rate_per_unit` changé au refetch est
      compté ; un `POST /fetch` qui a attendu le verrou derrière une récupération réussie du même mois ne rappelle pas
      la source ; un succès efface l'échec mémorisé du mois ; Vitest : un 503 de `POST /fetch` n'allume pas le bandeau dégradé ; E2E (écran, récupération servie par
      la source factice du mode test).
- [ ] Tests CR-1 (D9, D10) : parseur journalier (fixture, cinq décimales, `land_en` ignoré, racine vide) ; `fetch_day`
      contre le serveur local : `<datum>` antérieur de deux jours (dimanche) accepté et stocké sous la date demandée
      avec `source_date`, `<datum>` postérieur ou antérieur de plus de 7 jours → `date_mismatch`, rien d'écrit ;
      `POST /fetch { date }` : aujourd'hui et demain → 400, audité avec le genre ; la boucle ne demande jamais de jour ;
      cours effectif : saisi seul, AFC seul, les deux (le saisi gagne), aucun (404, `details.date` pour `day`) ;
      `PUT` : création, remplacement, version absente sur ligne existante et version périmée → 409, `rate` 0, négatif,
      11 décimales, > 10⁶ → `FX_RATE_INVALID`, `rateDate` du 15 en `month` → 400, `CHF` → 400, Consultation → 403,
      mono-devise → 409, audit `created`/`updated` avec avant et après ; `DELETE` : 204 et audit `deleted`, absent →
      404, version périmée → 409 ; une récupération ne touche aucun cours saisi (AC-3) ; société B ne voit pas le
      cours saisi de A ; aller-retour `.keshbackup` des cours saisis ; remise à zéro les vide et laisse les cours AFC ;
      E2E : saisir un cours, le voir proposé, ↑ le retire ; `xmldaily` 404 JSON (relevé du 2026-11-14) →
      `NotPublished`, `RATE_NOT_PUBLISHED`, mémorisé, rien d'écrit ; 404 de `xmlavgmonth`, 500 de l'une ou l'autre →
      `RATE_SOURCE_UNAVAILABLE` ;
      `<land_en/>` vide et `<gueltigkeit>` d'une seule date acceptés ; `lookup` `kind` hors `month|day` → 400, forme `day`
      sans cours → 404 avec `details.date` ; `GET /exchange-rates` d'une devise à cours saisi seul (`fetchedAt` `null`) ;
      cours saisi égal au cours AFC conservé (`overridden` vrai) ; deux `PUT` de création simultanés → un 200 et un 409 ;
      `DELETE` sans `version` → 400 ; `full_import_report_mirrors_the_production_registry` vert avec le registre vide.
- [ ] Manuels, CHANGELOG, `Cargo.lock`.

## Notes de développement

- Modèle de boucle : `crates/kesh-api/src/auto_backup.rs` (`spawn` :788, `un_tour` :803, `CADENCE` :58), lancement
  `main.rs:467`, garde `auto_backup::tests::lancement_par_main_seulement` (:1204, aide `code()` :1190).
- `reqwest` est déjà au `Cargo.lock` (0.12.28, dev-dependency de `kesh-api`, défauts + `json`, `multipart`,
  `cookies`) ; `rustls`
  0.23.46 (ring) y est aussi.
- Le cours du mois courant est publié en début de mois (relevé le 2026-11-14 : sans paramètre, `2026-11`).
- Pas de production des cours dans le dépôt : les fixtures sont synthétiques (C-35-1, conditions SIX).
- Cours journaliers relevés le 2026-11-14 (deux `GET`, `d=20241231` et `d=20240630`) : voir *Cours journaliers*.
- Fenêtre d'import : `parse_and_verify_avec_inventaire` (`admin_backup/import.rs:187`) compare le manifeste à
  `TABLES_TO_TRUNCATE` dans les deux sens ; précédent de fermeture : Story 32-5a (`mcp_keys`, commentaire de
  `POST_RESTORE_BACKFILLS`, `post_restore.rs:211-217`).
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
- 2026-11-14 — validation P4 (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 9 LOW (plus 3 LOW inter-fiches) ; remédiés dans la
  fiche sauf 35-2-P4-IF1 (35-3a : `DbError::RateUnavailable` créée par 35-2, nom `find_rate_in_tx`), 35-2-P4-IF2
  (35-3b : `timeoutMs` 90 s et non 60 s) et 35-2-P4-IF3 (35-1 : décomptes du registre des routes en deltas). Choix de
  cette passe, à confirmer : propriétaire unique de `RATE_UNAVAILABLE` = 35-2 (P4-1), `timeoutMs: 90_000` (P4-4),
  formes des réponses `GET` fixées (P4-7).
- 2026-11-14 — validation P5 (Opus) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 6 LOW ; remédiés dans la fiche sauf aucun
  (aucun finding inter-fiches). MEDIUM P5-1 : l'endpoint `_test` classé `SansEcritureAuJournal` et non `Exemptee`
  (`sans_ecriture` +2, `exemptees` inchangé). LOW : borne d'attente du verrou et récupération attendue non rejouée
  (P5-2), mémoire effacée sur succès (P5-3), `POST /fetch` dans un `tokio::spawn` (P5-4), DDL de
  `_kesh_exchange_rate_fetch` complété (P5-5), `reset_demo`, MCP et `from_current_user` nommés (P5-6), `CHF` → 400
  `Validation` et les deux `errors.rs` distingués (P5-7).
- 2026-11-14 — validation P6 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 3 LOW ; remédiés dans la fiche. P6-1 :
  `fetched_at` de la récupération attendue défini (pris avant l'appel sortant) et le raccourci sans audit ni écriture
  d'état ; P6-2 : `from_current_user` nommé comme méthode du trait `AuditActor` ; P6-3 : le précédent
  `/password-reset-token` écrit un jeton en base, la fiche dit donc « rien au journal d'audit ». Inter-fiches, non
  appliqué : l'entrée P5 du Change Log de 35-1 (IF-1) parle encore de `exemptees` +1 (historique ; l'AC-5 de 35-2 fait foi).
- 2026-11-14 — révision CR-1 (D9, D10, terminologie ; Opus) : cours journaliers OFDF (`xmldaily`, relevé en deux
  `GET` : espace de noms propre, cinq décimales, un jour sans publication rend la publication antérieure — `<datum>`
  lu, `date_mismatch` au-delà de 7 jours) ; `_kesh_exchange_rates` gagne `kind` et `rate_date` (PK `(currency, kind,
  rate_date)`) et `source_date` ; table applicative neuve `exchange_rate_overrides` (cours saisis, sauvegardée,
  vidée à la remise à zéro) qui **referme la fenêtre d'import** (registre de rejeu vidé, `20261106000001` hors
  fenêtre, sauvegarde 0.17.6 refusée) ; cours effectif = saisi ?? AFC (`find_effective_rate_in_tx`), `lookup` et
  `GET` rendent `effectiveRate`, `afcRate`, `overridden` (fin de `ratePerUnit`) ; `POST /fetch { date }` ; `PUT` et
  `DELETE /exchange-rates/overrides` (auditées, version, `FX_RATE_INVALID` et bornes d'un cours seul créés ici) ;
  registre des routes +3 routes / +4 au total ; écran à cours éditables, « AFC : … » et ↑, vue jour ; AC touchés :
  Story, Source, AC-1, AC-2, AC-3, AC-4, AC-5, AC-6, AC-7, AC-8, AC-9, Tâches, Notes ; « cours moyens » retiré.
- 2026-11-14 — validation CR-1 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 4 MEDIUM, 4 LOW ; remédiés dans la fiche (aucun
  finding inter-fiches). MEDIUM : `xmldaily` rend un **404 JSON** sans publication (relevé `d=20301231`), non une racine
  vide — traité `NotPublished` (CR1-P1-1) ; le test `full_import_report_mirrors_the_production_registry`
  (`admin_full_import_e2e.rs:1595`) rougit avec le registre vide, absent des conséquences de la fermeture de fenêtre
  (CR1-P1-2) ; la phase 5 de `recette-montee.sh` exige un 200 (:651) et ne peut « attendre un refus » : omise à cette
  release, script inchangé (CR1-P1-3) ; deux créations simultanées d'un cours saisi → 409 et non 500 (CR1-P1-4). LOW :
  forme `day` de `RATE_UNAVAILABLE` sans date dans `DbError` (CR1-P1-5), `fetchedAt` `null` d'un cours saisi seul
  (CR1-P1-6), champ vidé, bouton jour désactivé, date par défaut sans exercice (CR1-P1-7), `version` du `DELETE`
  obligatoire (CR1-P1-8). Non appliqué, à trancher par Beat : un cours saisi **égal** au cours AFC est marqué « saisi »
  sans ↑ (AC-6) et ne peut donc plus être retiré depuis l'écran ni suivre une republication de l'AFC — D9 dit « un
  bouton ↑ qui reprend le cours AFC » ; proposition : ↑ pour tout cours saisi existant. Axes exercés : citations de code
  (backup.rs, post_restore.rs, import.rs, audit_labels.rs, audit_route_registry.rs, mail/mod.rs, lib.rs,
  auto_backup.rs, api-client.ts, Cargo.lock, Dockerfile, exports/global.rs, recette-montee.sh), sites de D9/D10
  (sauvegarde, restauration, fenêtre, remise à zéro, export CSV, audit, RBAC, i18n, manuels, MCP), source journalière
  (3 GET : `20000103`, `20301231`, `20000101`), provenance `afc`/`manual`, tests. Non exercés : compilation, gate,
  rendu de l'écran.
- 2026-11-14 — validation CR-1 P2 (Opus) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 2 LOW ; remédiés dans la fiche sauf aucun
  (aucun finding inter-fiches). MEDIUM : `full_import_of_a_0_17_5_backup_keeps_the_numbers` (`admin_full_import_e2e.rs`
  :5638, appel d'`entree_numero_interne` :5669, `.expect` qui panique sur le registre vide) rougit aussi, absent des
  conséquences de la fermeture de fenêtre : les deux tests passent à `replay_retired` (CR1-P2-1) ; arbitrage de
  l'orchestrateur : ↑ pour **tout** cours saisi existant, égal au cours AFC ou sans cours AFC compris (AC-6, Vitest)
  (CR1-P2-2) ; arbitrage : la fonction des bornes d'un cours seul rend une erreur de type propre (`RateBoundsError`),
  traduite par chaque appelant — aligne l'AC-2 sur 35-3a AC-3 (CR1-P2-3). LOW : le 404 de `xmldaily` passe par une
  variante propre de `SourceError`, un 404 de `xmlavgmonth` reste `http_status:404` (CR1-P2-4) ; « autre 404 » du test
  précisé (CR1-P2-5). Axes exercés : diff de la révision et de la remédiation P1 ; citations relues au code
  (`admin_full_import_e2e.rs` :954, :972, :1561-1600, :5475, :5551, :5563, :5638 ; `b723e3d9` :1580-1600 ;
  `post_restore.rs` :205-232, :271, :1135-1145, :1210-1230, boucles des tests sur le registre ; `post_restore_class_a.rs`
  :75-100 ; `recette-montee.sh` :640-660 ; `api-client.ts` :42, :61, :388-425 ; `kesh-db/src/errors.rs` :675-679, :1721 ;
  `backup.rs` :46, :137-160) ; points de contact 35-3a AC-3 et 35-5 AC-4 (bornes) et 35-3b (↑ de la ligne, distinct) ;
  source journalière, un GET : `d=20240401` (lundi de Pâques) rend `<datum>28.03.2024` (jeudi saint, quatre jours :
  la borne de 7 jours tient sur un pont de Pâques). Non exercés : compilation, gate, rendu de l'écran, i18n et manuels
  au-delà de la fiche (rien de neuf depuis la P1).
- 2026-11-14 — validation CR-1 P3 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW ; remédié dans la fiche (aucun
  finding inter-fiches). LOW : phrase des bornes d'un cours seul redondante après l'ajout de CR1-P2-3 (« Elle sert ici au
  `PUT` … » répétait « le `PUT` … le parseur ») fusionnée (CR1-P3-1). Axes exercés : diff 796ae5a9..272a5289 de la fiche ;
  citations relues (`admin_full_import_e2e.rs` :972, :5551, :5563, :5619, :5638, :5669) ; cohérence avec D9 (epic), 35-3a
  AC-3 (`RateBoundsError`, `FX_VALUE_OUT_OF_RANGE { line }`), 35-5 AC-4 (`FX_RATE_INVALID`) et AC-6 (↑ propre à 35-2),
  35-3b (↑ de la ligne, distinct), `last_error` `http_status:<n>`, tests de l'AC-3. Non exercés : compilation, gate, rendu
  de l'écran, rien hors du diff.
