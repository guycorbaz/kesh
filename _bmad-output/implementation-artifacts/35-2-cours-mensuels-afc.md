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
   - `_kesh_exchange_rates (currency CHAR(3), month DATE, rate_per_unit DECIMAL(20,10) NOT NULL, published_unit
     INT NOT NULL, published_rate DECIMAL(19,6) NOT NULL, fetched_at DATETIME(3) NOT NULL, PRIMARY KEY
     (currency, month), CHECK (DAY(month) = 1), CHECK (rate_per_unit > 0), CHECK (published_unit > 0),
     CHECK (BINARY currency REGEXP '^[A-Z]{3}$'))` — `INT` et non `SMALLINT` (unités jusqu'à 10000 relevées) ; le
     `CHECK` binaire parce que la collation `_ci` confondrait `usd` et `USD` ;
   - `_kesh_exchange_rate_fetch` singleton : `id` (`CHECK (id = 1)`), `trigger ENUM('auto','manual')`, `last_run_at`,
     `last_status` (`CHECK IN ('ok','error','not_published')`), `last_error VARCHAR(64)` — **un code classé**, jamais
     le texte d'une erreur `reqwest` (qui porte l'URL) : `timeout`, `connect`, `http_status:<n>`, `too_large`,
     `malformed`, `month_mismatch`, `no_valid_rate`, `invalid_rates:<n>` —, `last_month DATE`. **Ligne créée
     paresseusement** (aucune écriture de données dans la migration, P7 sans objet ; le dump `--no-data` de
     `regen-test-schema.sh` n'a rien à réinjecter), écrite par `INSERT … ON DUPLICATE KEY UPDATE` ;
   - inscrites à `TABLES_SYSTEME` (`crates/kesh-db/src/backup.rs` :949, lue par `backup_inventory_matches_schema` et
     `convention_du_prefixe_des_tables_systeme`), absentes de l'inventaire `.keshbackup` ; `migrations.sha384`, ligne et
     compteurs recomptés de `docs/migrations-idempotence-audit.md` (P5), squash régénéré, P6 ;
   - test de montée sur le modèle d'`installation_settings_adds_only_its_system_table`
     (`crates/kesh-db/tests/migrations_upgrade_path.rs:1763`), monté **par version** (`migrations_before(VERSION)`,
     P6) : deux tables neuves, système, vides ; aucune table applicative ajoutée, donc une sauvegarde 0.17.6 reste
     importable.
2. **AC-2 — Analyse XML** dans `kesh-core` (pure, sans I/O) : `parse_monthly_rates(xml) -> Result<MonthlyRates>`
   (mois, liste `{currency, published_unit, published_rate, rate_per_unit}`), `rate_per_unit = published_rate /
   published_unit` exact (`Decimal`) ; code mis en majuscules et validé (trois lettres). Parseur `quick-xml = "0.42"`
   (déjà au `Cargo.lock` par `kesh-import` et `kesh-payment`, dépendance neuve de `kesh-core`), lecture **par noms
   locaux** (la racine porte un `xmlns` par défaut), éléments inconnus ignorés (`land_de`, `land_fr`, `land_it`).
   - **Unité** : entier positif en tête de `<waehrung>` (1, 100, 1000, 10000 relevés — aucune liste fermée), le code
     qui suit doit égaler l'attribut `code` (casse ignorée).
   - **Tolérance par devise** : une devise invalide (cours ≤ 0, unité ou code illisible, code et `<waehrung>`
     discordants, doublon — les deux occurrences) est **écartée**, rendue dans `MonthlyRates.rejected` et journalisée
     par l'appelant (`tracing::warn!`, `last_error = invalid_rates:<n>`) ; les autres s'écrivent. Une seule devise
     fautive (p. ex. un cours publié `0.0000`) ne bloque donc jamais USD ni EUR.
   - **Refus du mois entier** seulement si : XML mal formé, `<monat>` absent ou illisible, ou aucune devise valide
     (`NoValidRate`). Racine vide → `NotPublished`.
   - Fixture versionnée **synthétique** (forme du XML réel, `xmlns` compris, valeurs inventées — aucun cours officiel
     n'est versionné : conditions d'utilisation SIX) avec unités 1, 100, 1000 et 10000 et un cours à 0.0009 ; tests du
     cas vide, des refus du mois et des écarts par devise.
3. **AC-3 — Source des cours et client HTTP** (`kesh-api`, module `exchange_rates`).
   - **Trait `RateSource`** (`async fn fetch_month(year, month) -> Result<String, SourceError>`), porté par `AppState`
     (`rate_source: Arc<dyn RateSource>`, précédent `mailer: Arc<dyn Mailer>` / `test_mock_mailer`, `lib.rs:84-98`,
     et `BackupEnv` de la sauvegarde automatique ; `AppState::new_for_tests` reçoit une source factice par défaut).
   - **Production** : `HttpRateSource`, URL de base **constante** `https://www.backend-rates.bazg.admin.ch`, **aucune
     variable d'environnement** ne la change. Les tests Rust construisent `HttpRateSource::with_base_url(…)`
     (`pub(crate)` ou `#[cfg(test)]`) contre un serveur axum local en HTTP.
   - **`KESH_TEST_MODE`** : `main.rs` substitue une **source factice** (XML synthétique en mémoire, aucun appel
     sortant — la boucle comprise), pilotée par un endpoint `/api/v1/_test/exchange-rates/source` (réponse servie :
     XML, vide, erreur, délai) monté avec les autres routes `_test` ; il entre au registre (`TEST_ENDPOINT_ROUTES`,
     total 149 → 150). La recette E2E de `CLAUDE.md` et de `docs/testing.md` reste inchangée.
   - `reqwest` en dépendance de production, `default-features = false`, `rustls-tls` ; `.use_rustls_tls()` explicite
     (la dev-dependency garde ses fonctionnalités par défaut — native-tls, cookies — qui s'unifient dans les builds de
     test) ; `redirect(Policy::none())` ; `https_only(true)` pour l'URL de production ; `connect_timeout` 5 s et délai
     total 20 s ; pas de magasin de cookies (la source pose `rates_service_session`) ; réponse limitée à 1 MiB
     (`Content-Length` contrôlé puis lecture par `chunk()` bornée) ; `User-Agent: Kesh/<version>` sans donnée
     personnelle. `Cargo.lock` change (entrée `reqwest` : `rustls`, `tokio-rustls`, `webpki-roots` 1.0.11, déjà au
     lock) et se commite ; `rustls-tls` embarque `webpki-roots`, l'image a de toute façon `ca-certificates`
     (`Dockerfile:20`) ; le build Docker (`cargo build --release -p kesh-api`) n'a rien d'autre à changer.
   - **Une seule récupération à la fois** (verrou `tokio::sync::Mutex` dans `AppState`, comme `backup_lock`) ;
     écriture en **une transaction** : `INSERT … ON DUPLICATE KEY UPDATE` des devises valides du mois ; un échec du mois
     n'écrit aucun cours.
4. **AC-4 — Récupération automatique.** Une boucle `exchange_rates::spawn(state)` sur le modèle de
   `auto_backup::spawn` (`auto_backup.rs:788`) : **premier tour 5 minutes après le démarrage**, puis toutes les 6 h
   (`interval_at(now + PREMIER_TOUR, CADENCE)`, `MissedTickBehavior::Skip`), chaque tour dans son propre
   `tokio::spawn` (une panique est journalisée et n'arrête pas la boucle). Lancée de `main.rs` **seulement**, avec un
   commentaire de renvoi à la garde comme `main.rs:466` ; garde lexicale
   `exchange_rates::tests::lancement_par_main_seulement` (jeton `exchange_rates::spawn(` : une fois dans `main.rs`,
   zéro dans `lib.rs`), qui **réutilise** l'aide `code()` d'`auto_backup` (extraite ou partagée, refs #727) au lieu de
   la recopier (DRY). Sautée si **aucune** société n'a `multi_currency_enabled` (aucun trafic sortant en
   mono-devise) ; à chaque tour : récupère le **mois courant** s'il manque (publié en début de mois), et le **mois
   précédent** s'il manque — « manque » = aucune ligne pour ce mois. « Mois courant » s'entend en
   **Europe/Zurich**, non en UTC (précédent #889). Un échec se journalise (`tracing::warn!`), s'écrit dans
   `_kesh_exchange_rate_fetch` (`trigger = 'auto'`) et se retente au tour suivant ; jamais bloquant au démarrage.
   Non auditée (C-35-6). **Seule la boucle** écrit `trigger = 'auto'`.
5. **AC-5 — API.**
   - `GET /api/v1/exchange-rates?month=AAAA-MM[&currency=USD]` : cours du mois (toutes devises ou une) ;
   - `GET /api/v1/exchange-rates/status` : état de `_kesh_exchange_rate_fetch` (`trigger`, date, statut, code
     d'erreur) ;
   - `POST /api/v1/exchange-rates/fetch { year, month }` — dans **`comptable_routes`** (Administrateur ou Comptable ;
     mois ≥ 2000-01 et ≤ mois courant en Europe/Zurich, sinon 400) : récupère ce mois, **auditée sous l'utilisateur**
     (`exchange_rates.fetched`, mois, nombre de devises écrites et écartées) ; écrit `_kesh_exchange_rate_fetch` avec
     `trigger = 'manual'` ; réponses `200 {month, count, rejected}`, `404 RATE_NOT_PUBLISHED`,
     `503 RATE_SOURCE_UNAVAILABLE` (code explicite, message sans URL ni trace — **jamais 502** : le frontend lit 502 et
     504 comme une issue inconnue venue d'un proxy, `frontend/src/lib/shared/utils/unknown-outcome.ts`). Un mois
     `not_published` ou en échec n'est pas redemandé à la source avant 15 minutes (état en mémoire) : la réponse
     d'origine est rendue ;
   - `GET /api/v1/exchange-rates/lookup?currency=USD&date=AAAA-MM-JJ` — **en lecture seule**, sans appel sortant ni
     écriture ni audit, ouverte à tout rôle authentifié : cours du mois de la date, ou
     `404 RATE_UNAVAILABLE {currency, month}` (jamais 1.0). L'écran de saisie (35-3b) et la réévaluation (35-5)
     appellent `POST /fetch` sur ce 404 (rôles Administrateur et Comptable), puis relisent ; sans cours, saisie
     manuelle. Devise `CHF` ou hors ISO → 400.
   - Toutes ces routes exigent le mode multi-devises de la société courante : **`409 MULTI_CURRENCY_DISABLED`**, même
     statut que dans la 35-1.
   - Nouvelles variantes `AppError` (`errors.rs`) et leur traduction côté frontend pour `RATE_NOT_PUBLISHED`,
     `RATE_SOURCE_UNAVAILABLE`, `RATE_UNAVAILABLE`.
   - **Audit et registres** : action `exchange_rates.fetched` ajoutée à `ACTIONS` (`crates/kesh-api/src/audit_labels.rs`,
     liste triée, garde `tests/audit_label_registry.rs`) ; type d'entité `installation` avec `AUDIT_ENTITY_ID_NONE`
     (précédent `admin.backup_run`, `routes/backup_schedule.rs:177-191`, `emit_audit` après coup — la table est
     globale et `audit_log.entity_id` est `NOT NULL`) ; clé `audit-log-action-exchange-rates-fetched` dans les quatre
     locales. `POST /fetch` entre dans `LIB_ROUTES` (`tests/audit_route_registry.rs`, `Traced, SansEcritureAuJournal`) :
     `the_registry_partition_is_what_the_story_declares` passe de 146 à 147 routes et de 139 à 140 tracées (à cumuler
     avec les routes de la 35-1), le total registre de 149 à 150 avec l'endpoint `_test` ; la phrase du manuel
     d'administration (`admin-manual.tex:3217`, « 139 des 146 routes à verbe mutant » et « 139 + 4 + 3 = 146 », gardée
     par `le_manuel_dit_le_compte_du_registre`) suit.
6. **AC-6 — Écran** *Paramètres → Cours de change* (visible en multi-devises) : choix du mois, tableau
   (devise, unité publiée, cours publié, cours par unité, date de récupération), filtre par devise, bouton
   « Récupérer ce mois » (Administrateur et Comptable seulement), état de la dernière récupération (automatique ou
   manuelle, d'après `trigger` ; date, statut, code d'erreur traduit) ; mention de la source et de son caractère non
   contraignant. Carte sur `settings/+page.svelte`. Captures d'écran.
7. **AC-7 — Restauration.** Les deux tables sont système : un import de sauvegarde ne les touche pas. Sur une
   installation neuve restaurée, elles sont vides : la boucle ou `POST /fetch` les complète ; aucun test ne suppose un
   cours présent sans l'avoir posé.
8. **AC-8 — i18n, manuels.** Quatre locales (écran, codes d'erreur, libellé d'audit) ; manuel utilisateur (écran,
   source, récupération) ; manuel d'administration : accès sortant HTTPS vers `www.backend-rates.bazg.admin.ch`
   requis en multi-devises, aucun en mono-devise, mandataire par `HTTPS_PROXY` (lu par `reqwest`) ; phrase du
   registre des routes (AC-5) ; PDF régénérés.
9. **AC-9 — Cours manuels.** Aucun cours manuel n'entre dans `_kesh_exchange_rates` (C-35-1) : un cours saisi à la
   main vit **sur la ligne d'écriture** (`rate_source = 'manual'`, 35-3a/35-3b), audité avec l'écriture.

## Tâches

- [ ] Migration + inscriptions (AC-1).
- [ ] `kesh-core::exchange_rates` (AC-2) ; `kesh-db::repositories::exchange_rates` (upsert en transaction, lecture,
      état).
- [ ] `kesh-api` : trait `RateSource`, `HttpRateSource`, source factice et endpoint `_test` (AC-3), boucle (AC-4,
      `main.rs`), routes (AC-5, `lib.rs` : `GET` dans `authenticated_routes`, `POST /fetch` dans
      `comptable_routes`), variantes `AppError`, audit (`audit_labels.rs`), registre des routes et phrase du manuel.
- [ ] Frontend : page `settings/exchange-rates`, carte des paramètres, types, API, codes d'erreur, Vitest (AC-6).
- [ ] Tests : parseur (fixture : unités 1/100/1000/10000, devise écartée, mois refusé) ; client contre un serveur HTTP
      local (délai, 500, redirection non suivie, XML tronqué, corps trop gros, mois différent → rien d'écrit) ;
      `un_tour` sauté en mono-devise ; `un_tour` contre une source injoignable rend sans paniquer, écrit `error` et
      `trigger = 'auto'`, ne touche aucun cours ; `lookup` n'écrit rien et ne sort pas ; `POST /fetch` refusé au rôle
      Consultation, audité, `trigger = 'manual'` ; aucune trace d'audit en automatique ; deux `POST /fetch` simultanés
      sérialisés ; E2E (écran, récupération servie par la source factice du mode test).
- [ ] Manuels, CHANGELOG, `Cargo.lock`.

## Notes de développement

- Modèle de boucle : `crates/kesh-api/src/auto_backup.rs` (`spawn` :788, `un_tour` :803, `CADENCE` :58), lancement
  `main.rs:467`, garde `auto_backup::tests::lancement_par_main_seulement` (:1204, aide `code()` :1190).
- `reqwest` est déjà au `Cargo.lock` (0.12.28, dev-dependency de `kesh-api`, fonctionnalités par défaut) ; `rustls`
  0.23.46 (ring) y est aussi.
- Le cours du mois courant est publié en début de mois (relevé le 2026-11-14 : sans paramètre, `2026-11`).
- Pas de production des cours dans le dépôt : la fixture est synthétique (C-35-1, conditions SIX).
- Pas d'audit automatique : `audit_log.user_id` est `NOT NULL` (C-35-6).

## Change Log

- 2026-11-14 — fiche rédigée.
- 2026-11-14 — validation P1 (Opus) : 0 CRITICAL, 2 HIGH, 8 MEDIUM, 9 LOW ; remédiés ; alignement de 35-3b/35-5 sur
  POST /fetch et du tableau de l'epic portés par l'orchestrateur.
