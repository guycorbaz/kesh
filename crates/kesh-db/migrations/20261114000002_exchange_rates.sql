-- Non breaking (P1, P3) : CREATE TABLE seulement (deux tables système, une
-- table applicative). Un binaire 0.17.6 ne démarre de toute façon pas sur une
-- base migrée (sqlx : migration appliquée inconnue) ; aucune table existante
-- n'est modifiée ; kesh_version_min_required reste à 0.17.6.
--
-- Story 37-2 (Epic 37, multi-devises) — les cours de change.
--
-- 1. _kesh_fetched_exchange_rates (SYSTÈME) : les cours TELS QUE PUBLIÉS par la
--    source (AFC / OFDF : cours mensuels et journaliers), une ligne par
--    (source, devise, genre, date), la DERNIÈRE valeur publiée. Une correction
--    de la source remplace la valeur de cette table et ne touche jamais
--    exchange_rates. rate_date = le 1er du mois (month) ou la date DEMANDÉE
--    (day) ; source_date = la date de publication lue (<datum>) pour day, NULL
--    pour month. first_fetched_at n'est jamais mis à jour (date d'obtention du
--    calendrier de récupération) ; fetched_at = dernière récupération qui a
--    confirmé la ligne ; value_changed_at = récupération qui a apporté la
--    valeur actuelle. Horodatages en UTC, posés par le code.
-- 2. _kesh_exchange_rate_fetch (SYSTÈME) : singleton de l'état de la dernière
--    récupération, codes d'erreur CLASSÉS (jamais le texte d'une erreur
--    réseau). Colonne triggered_by et non trigger (mot réservé). Ligne créée
--    paresseusement par le code (INSERT … ON DUPLICATE KEY UPDATE).
-- 3. exchange_rates (APPLICATIVE, sauvegardée) : le cours RETENU par la
--    société, une ligne par (société, devise, genre, date), par unité de
--    devise, avec sa source. Le CHECK de source ne porte que le FORMAT : les
--    valeurs admises (afc, manual) sont tenues par le code, et une source
--    future s'ajoute sans DDL (une valeur inconnue se lit sans erreur).
--    updated_by NULL = ligne écrite par Kesh (copie automatique d'un cours
--    récupéré). acknowledged_rate / acknowledged_at : acquittement, par la
--    société, d'une correction de la source, à cette valeur (ligne afc
--    seulement). Relation LOGIQUE avec _kesh_fetched_exchange_rates, sans clé
--    étrangère (table système, vidée à volonté, absente de la sauvegarde).
--
-- Collation (choix C-37-2-1, comme C-37-1-1) : les colonnes ASCII (currency,
-- source, kind) sont en CHARACTER SET ascii COLLATE ascii_general_ci et NON
-- ascii_bin — le serveur marque BINARY une colonne en collation binaire, et
-- sqlx-mysql refuse de la décoder en String. La casse reste refusée par le
-- schéma : chaque CHECK compare en BINARY. currency a la définition
-- d'accounts.currency.
--
-- Fenêtre d'import : exchange_rates est inscrite à la liste close des
-- absences admises de kesh-db::backup (ABSENCES_ADMISES_EPIC_37) — une
-- sauvegarde antérieure, forcément mono-devise, se réimporte sans elle ; la
-- fenêtre ne bouge pas. Les deux tables système sont hors de
-- TABLES_TO_TRUNCATE : jamais exportées ni restaurées.
--
-- Aucune écriture de données (P7 : rien à trier). Ligne ajoutée à
-- docs/migrations-idempotence-audit.md (P5). Idempotence : tracked-by-sqlx.

CREATE TABLE _kesh_fetched_exchange_rates (
    source VARCHAR(12) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
        COMMENT 'Source des cours (afc aujourd''hui ; format seul au schéma)',
    currency CHAR(3) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
        COMMENT 'Code ISO 4217, majuscules',
    kind VARCHAR(5) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
        COMMENT 'month = cours mensuel, day = cours journalier',
    rate_date DATE NOT NULL
        COMMENT '1er du mois (month) ou date demandée (day)',
    rate_per_unit DECIMAL(20,10) NOT NULL
        COMMENT 'published_rate / published_unit, arrondi à 10 décimales',
    published_unit INT NOT NULL
        COMMENT 'Unité publiée (1, 100, 1000, 10000…)',
    published_rate DECIMAL(19,6) NOT NULL
        COMMENT 'Cours publié pour l''unité',
    source_date DATE NULL
        COMMENT 'Date de publication lue (day), NULL pour month',
    first_fetched_at DATETIME(3) NOT NULL
        COMMENT 'Première obtention (UTC), jamais mise à jour',
    fetched_at DATETIME(3) NOT NULL
        COMMENT 'Dernière récupération qui a confirmé la ligne (UTC)',
    value_changed_at DATETIME(3) NOT NULL
        COMMENT 'Récupération qui a apporté la valeur actuelle (UTC)',
    PRIMARY KEY (source, currency, kind, rate_date),
    INDEX idx_kesh_fetched_exchange_rates_unit (source, kind, rate_date),
    CONSTRAINT chk_kesh_fetched_exchange_rates_source
        CHECK (BINARY source REGEXP '^[a-z]{2,12}$'),
    CONSTRAINT chk_kesh_fetched_exchange_rates_kind
        CHECK (BINARY kind IN ('month', 'day')),
    CONSTRAINT chk_kesh_fetched_exchange_rates_month_day
        CHECK (BINARY kind <> 'month' OR DAY(rate_date) = 1),
    CONSTRAINT chk_kesh_fetched_exchange_rates_source_date
        CHECK ((BINARY kind = 'day') = (source_date IS NOT NULL)),
    CONSTRAINT chk_kesh_fetched_exchange_rates_rate
        CHECK (rate_per_unit > 0),
    CONSTRAINT chk_kesh_fetched_exchange_rates_unit
        CHECK (published_unit > 0),
    CONSTRAINT chk_kesh_fetched_exchange_rates_currency
        CHECK (BINARY currency REGEXP '^[A-Z]{3}$')
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE _kesh_exchange_rate_fetch (
    id TINYINT UNSIGNED NOT NULL PRIMARY KEY DEFAULT 1,
    triggered_by ENUM('auto', 'manual') NOT NULL
        COMMENT 'auto = tâche ou connexion, manual = POST /exchange-rates/fetch',
    last_run_at DATETIME(3) NOT NULL
        COMMENT 'Fin de la dernière récupération (UTC)',
    last_status VARCHAR(16) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
        COMMENT 'ok, error ou not_published (le pire d''un tour)',
    last_error VARCHAR(64) CHARACTER SET ascii COLLATE ascii_general_ci NULL
        COMMENT 'Code classé (timeout, connect, http_status:<n>, malformed…), jamais un texte libre',
    last_kind VARCHAR(5) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
        COMMENT 'Genre de l''unité en cause',
    last_rate_date DATE NOT NULL
        COMMENT 'Date de l''unité en cause (1er du mois ou jour demandé)',
    CONSTRAINT chk_kesh_exchange_rate_fetch_single_row CHECK (id = 1),
    CONSTRAINT chk_kesh_exchange_rate_fetch_status
        CHECK (BINARY last_status IN ('ok', 'error', 'not_published')),
    CONSTRAINT chk_kesh_exchange_rate_fetch_kind
        CHECK (BINARY last_kind IN ('month', 'day'))
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE exchange_rates (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    company_id BIGINT NOT NULL,
    currency CHAR(3) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
        COMMENT 'Code ISO 4217, majuscules, jamais CHF',
    kind VARCHAR(5) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
        COMMENT 'month = cours mensuel, day = cours journalier',
    rate_date DATE NOT NULL
        COMMENT '1er du mois (month) ou jour (day)',
    rate DECIMAL(20,10) NOT NULL
        COMMENT 'Cours retenu par la société, par unité de devise',
    source VARCHAR(12) CHARACTER SET ascii COLLATE ascii_general_ci NOT NULL
        COMMENT 'afc (copie ou reprise d''une valeur publiée) ou manual (cours saisi) ; format seul au schéma',
    acknowledged_rate DECIMAL(20,10) NULL
        COMMENT 'Valeur publiée (par unité) dont la correction a été acquittée par la société',
    acknowledged_at DATETIME(3) NULL
        COMMENT 'Acquittement (UTC)',
    version INT NOT NULL DEFAULT 1,
    updated_by BIGINT NULL
        COMMENT 'Auteur de la dernière écriture ; NULL = ligne écrite par Kesh (copie)',
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    CONSTRAINT fk_exchange_rates_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT fk_exchange_rates_updated_by FOREIGN KEY (updated_by) REFERENCES users(id) ON DELETE RESTRICT,
    CONSTRAINT uq_exchange_rates_key UNIQUE (company_id, currency, kind, rate_date),
    INDEX idx_exchange_rates_company_kind_date (company_id, kind, rate_date),
    CONSTRAINT chk_exchange_rates_currency
        CHECK (BINARY currency REGEXP '^[A-Z]{3}$' AND BINARY currency <> BINARY 'CHF'),
    CONSTRAINT chk_exchange_rates_kind
        CHECK (BINARY kind IN ('month', 'day')),
    CONSTRAINT chk_exchange_rates_month_day
        CHECK (BINARY kind <> 'month' OR DAY(rate_date) = 1),
    CONSTRAINT chk_exchange_rates_rate
        CHECK (rate > 0),
    CONSTRAINT chk_exchange_rates_source
        CHECK (BINARY source REGEXP '^[a-z]{2,12}$'),
    CONSTRAINT chk_exchange_rates_ack_pair
        CHECK ((acknowledged_rate IS NULL) = (acknowledged_at IS NULL)),
    CONSTRAINT chk_exchange_rates_ack_afc
        CHECK (acknowledged_rate IS NULL OR BINARY source = 'afc')
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
