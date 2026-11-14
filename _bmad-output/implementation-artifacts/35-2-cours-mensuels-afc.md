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
- `<monatsmittelkurs><monat>AAAA-MM</monat><devise code="usd"><waehrung>1 USD</waehrung><kurs>0.8846</kurs>…` —
  73 devises en 2024-12 ; unité 1 ou 100 (`100 JPY`) ; code en minuscules ;
- mois non publié : élément racine vide → « pas encore publié » ;
- la réponse peut porter un autre mois que celui demandé (sans paramètre : mois courant) : **le `<monat>` lu fait foi** ;
  un écart avec le mois demandé est une erreur (rien n'est écrit).

## Critères d'acceptation

1. **AC-1 — Migration** `20261114000002_exchange_rates.sql` (non breaking, DDL seul) :
   - `_kesh_exchange_rates (currency CHAR(3), month DATE, rate_per_unit DECIMAL(20,10) NOT NULL, published_unit
     SMALLINT NOT NULL, published_rate DECIMAL(19,6) NOT NULL, fetched_at DATETIME(3) NOT NULL, PRIMARY KEY
     (currency, month), CHECK (DAY(month) = 1), CHECK (rate_per_unit > 0), CHECK (published_unit > 0))` ;
   - `_kesh_exchange_rate_fetch` singleton (`id = 1`) : `last_run_at`, `last_status` (`ok|error|not_published`),
     `last_error VARCHAR(500)`, `last_month DATE` ; ligne initiale insérée par la migration **ou** créée paresseusement
     (choisir le paresseux : aucune écriture de données, P7 sans objet ; sinon étendre `regen-test-schema.sh` et le
     garde comme pour `_kesh_backup_schedule`) ;
   - inscrites à `TABLES_SYSTEME` (`crates/kesh-db/src/backup.rs` :949), absentes de l'inventaire `.keshbackup` (test) ;
     `migrations.sha384`, audit d'idempotence, squash, P6.
2. **AC-2 — Analyse XML** dans `kesh-core` (pure, sans I/O) : `parse_monthly_rates(xml) -> Result<MonthlyRates>`
   (mois, liste `{currency, published_unit, published_rate, rate_per_unit}`), `rate_per_unit = published_rate /
   published_unit` exact (`Decimal`) ; code mis en majuscules et validé (trois lettres) ; racine vide → `NotPublished` ;
   XML mal formé, unité illisible, cours ≤ 0, doublon de devise → erreur. Fixture versionnée **synthétique** (forme du
   XML réel, valeurs inventées — aucun cours officiel n'est versionné : conditions d'utilisation SIX) avec unités 1 et
   100 ; tests du cas vide et des refus.
3. **AC-3 — Client HTTP** (`kesh-api`, module `exchange_rates`) : `reqwest` en dépendance de production,
   `default-features = false`, `rustls-tls` ; URL de base **constante** (surchargée seulement en mode test, pour les
   tests) ; délai total 20 s ; réponse limitée à 1 MiB (lecture bornée) ; `User-Agent: Kesh/<version>` sans donnée
   personnelle. Écriture en **une transaction** : `INSERT … ON DUPLICATE KEY UPDATE` de toutes les devises du mois ;
   un échec n'écrit rien et renseigne `_kesh_exchange_rate_fetch`.
4. **AC-4 — Récupération automatique.** Une boucle sur le modèle de `auto_backup::spawn` (`auto_backup.rs:788`,
   lancée de `main.rs` seulement, garde lexicale équivalente), cadence 6 h, sautée si **aucune** société n'a
   `multi_currency_enabled` (aucun trafic sortant en mono-devise) ; à chaque tour : récupère le **mois courant** s'il
   manque, et le **mois précédent** s'il manque ; un échec se journalise (`tracing::warn!`) et se retente au tour
   suivant ; jamais bloquant au démarrage. Non auditée (C-35-6).
5. **AC-5 — API.**
   - `GET /api/v1/exchange-rates?month=AAAA-MM[&currency=USD]` : cours du mois (toutes devises ou une) ;
   - `GET /api/v1/exchange-rates/status` : état de `_kesh_exchange_rate_fetch` ;
   - `POST /api/v1/exchange-rates/fetch { year, month }` (Administrateur ou Comptable ; mois ≤ mois courant ; ≥ 2000) :
     récupère ce mois, **auditée sous l'utilisateur** (`exchange_rates.fetched`, mois et nombre de devises) ; réponses
     `200 {month, count}`, `404 RATE_NOT_PUBLISHED`, `502 RATE_SOURCE_UNAVAILABLE` (message sans URL interne ni
     trace) ;
   - `GET /api/v1/exchange-rates/lookup?currency=USD&date=AAAA-MM-JJ` : cours du mois de la date ; s'il manque, une
     récupération à la demande est tentée **une fois** (même audit) ; sinon `404 RATE_UNAVAILABLE {currency, month}`
     (jamais 1.0).
   - Toutes ces routes exigent le mode multi-devises de la société courante (`409 MULTI_CURRENCY_DISABLED`).
6. **AC-6 — Écran** *Paramètres → Cours de change* (visible en multi-devises) : choix du mois, tableau
   (devise, unité publiée, cours publié, cours par unité, date de récupération), filtre par devise, bouton
   « Récupérer ce mois », état de la dernière récupération automatique (date, statut, erreur) ; mention de la source et
   de son caractère non contraignant. Captures d'écran.
7. **AC-7 — Restauration.** Après import d'une sauvegarde, la table est vide ou partielle : le premier `lookup` la
   complète ; aucun test ne suppose un cours présent sans l'avoir posé.
8. **AC-8 — i18n, manuels.** Quatre locales ; manuel utilisateur (écran, source, récupération) ; manuel
   d'administration : accès sortant HTTPS vers `www.backend-rates.bazg.admin.ch` requis en multi-devises, aucun en
   mono-devise ; PDF régénérés.

## Tâches

- [ ] Migration + inscriptions (AC-1).
- [ ] `kesh-core::exchange_rates` (AC-2) ; `kesh-db::repositories::exchange_rates` (upsert en transaction, lecture,
      état).
- [ ] `kesh-api` : client (AC-3), boucle (AC-4, `main.rs`), routes (AC-5, `lib.rs`), codes d'erreur, audit.
- [ ] Frontend : page `settings/exchange-rates`, entrée de menu, types, API, Vitest (AC-6).
- [ ] Tests : parseur (fixture), client contre un serveur HTTP local (délai, 500, XML tronqué, corps trop gros, mois
      différent → rien d'écrit), boucle sautée en mono-devise, démarrage avec source injoignable, audit à la demande et
      pas en automatique ; E2E (écran, récupération servie par un faux serveur en mode test).
- [ ] Manuels, CHANGELOG.

## Notes de développement

- Modèle de boucle : `crates/kesh-api/src/auto_backup.rs` (`spawn` :788, `un_tour` :810, `CADENCE` :58), lancement
  `main.rs:467`, garde `auto_backup::tests::lancement_par_main_seulement`.
- `reqwest` est déjà au `Cargo.lock` (0.12.28, dev-dependency de `kesh-api`) ; `rustls` y est aussi.
- Pas de production des cours dans le dépôt : la fixture est synthétique (C-35-1, conditions SIX).
- Pas d'audit automatique : `audit_log.user_id` est `NOT NULL` (C-35-6).

## Change Log

- 2026-11-14 — fiche rédigée.
