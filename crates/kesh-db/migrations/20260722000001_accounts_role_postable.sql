-- Story 14-3a (Epic 14, refs #269) — Rôles de comptes explicites + postabilité.
--
-- PRINCIPE : chaque compte porte un RÔLE explicite ; le numéro ne sert JAMAIS à
-- déduire le rôle. Le plan comptable suisse (norme Käfer/PME) est un usage, pas
-- une obligation légale — l'utilisateur peut renuméroter, ajouter, supprimer.
-- Tout code applicatif qui écrirait `WHERE number = '1100'` est un piège
-- silencieux (il en reste 6 dans company_invoice_settings.rs, supprimés par la
-- Story 14-3b qui consommera les rôles posés ici).
--
-- Colonnes ajoutées :
--   * `role`      — 10 valeurs à liste fermée (CHECK BINARY, calqué sur
--                   chk_accounts_type), NULL = aucun rôle. VARCHAR(32) plutôt
--                   qu'un ENUM MySQL pour rester cohérent avec `account_type`
--                   (VARCHAR(20) + CHECK) et garder les impls sqlx manuelles.
--   * `postable`  — `false` = compte titre/de regroupement, ou compte de
--                   résultat que l'application calcule (modèle temps réel
--                   virtuel de la Story 14-1). NOT NULL DEFAULT TRUE.
--                   ⚠️ Story 14-3a POSE la colonne mais ne l'applique PAS à la
--                   saisie d'écriture — c'est la Story 14-3b.
--   * `singleton_role` — colonne générée VIRTUAL, cf. bloc suivant.
--
-- UNICITÉ PARTIELLE DES RÔLES SINGLETON
-- MariaDB n'a pas de UNIQUE partiel natif (pas de syntaxe `WHERE active = TRUE`).
-- On reprend à l'identique le « Workaround Option A » déjà utilisé pour
-- `reconciliation_rules.active_uniq` (20260513000001_reconciliation_rules.sql:54) :
-- une colonne synthétique GENERATED ALWAYS AS ... VIRTUAL qui vaut le rôle si le
-- compte est actif ET si le rôle est singleton, NULL sinon. Le UNIQUE sur
-- (company_id, singleton_role) exploite la convention SQL « NULL n'est jamais
-- égal à NULL » → les comptes sans rôle, les rôles multi-valués (EquityCapital,
-- EquityOther) et les comptes archivés ne participent pas à la contrainte.
-- Pré-requis : MariaDB >= 10.6 pour un UNIQUE sur colonne VIRTUAL (compose
-- épinglé sur mariadb:10.11 — OK).
--
-- Le `active AND` n'est PAS cosmétique : sans lui, un compte archivé squatterait
-- son rôle singleton à vie et son remplaçant actif ne pourrait jamais le
-- recevoir (409 permanent causé par un compte mort). Le code remplacé par cette
-- story filtre déjà `AND active = true` (company_invoice_settings.rs:275).
-- Corollaire assumé : réactiver un compte dont le rôle a été repris échoue —
-- l'API le détecte en amont et répond 409 ACCOUNT_ROLE_ALREADY_ASSIGNED.
--
-- ⚠️ LISTE DES SINGLETONS SYNCHRONISÉE À TROIS ENDROITS :
--   1. le CASE WHEN ci-dessous ;
--   2. kesh_db::entities::account::AccountRole::is_singleton() ;
--   3. kesh_core::chart_of_accounts::AccountRole::is_singleton().
-- Le test `singleton_list_matches_sql_generation_expression` (kesh-db) compare
-- la liste Rust à GENERATION_EXPRESSION lue dans information_schema.
--
-- ALGORITHM=INSTANT NON applicable : l'ajout d'une colonne VIRTUAL est bien
-- instantané (aucune ligne réécrite), mais l'ADD CONSTRAINT UNIQUE construit un
-- index. Ne pas le tenter (ERROR 1845). Sans conséquence pratique : ~84 comptes
-- par société.
--
-- Aucun index secondaire ajouté au-delà du UNIQUE (YAGNI, cf.
-- 20260531000001_bank_accounts_archived.sql:7-13) : les lookups par rôle de la
-- Story 14-3b sont scopés `company_id` et la table fait < 200 lignes par
-- société ; le UNIQUE (company_id, singleton_role) sert déjà d'index pour les
-- rôles singleton, qui sont précisément ceux que 14-3b interrogera.
--
-- Idempotence (docs/migrations-idempotence-audit.md) : tracked-by-sqlx. L'ALTER
-- n'a pas d'IF NOT EXISTS (convention majoritaire du repo) — une ré-exécution
-- hors sqlx échouerait (1060 sur le premier ADD COLUMN). Les trois UPDATE de
-- backfill sont en revanche intrinsèquement idempotents (`WHERE role IS NULL`
-- pour les rôles ; `SET postable = FALSE` est un point fixe).
--
-- NON-BREAKING → PAS de bump `kesh_version_min_required` (politique P1/P3
-- CLAUDE.md) : ADD COLUMN nullable + ADD COLUMN NOT NULL DEFAULT + nouvelle
-- contrainte. Un binaire antérieur énumère des colonnes explicites partout sur
-- `accounts` (repositories/accounts.rs COLUMNS et FIND_BY_ID_SQL — aucun
-- `SELECT *` dans le workspace), ne verra donc jamais ces colonnes ; ses INSERT
-- laissent `role` à NULL (aucun conflit sur le UNIQUE) et `postable` à son
-- défaut.

ALTER TABLE accounts
    ADD COLUMN role VARCHAR(32) NULL
        COMMENT 'Rôle métier explicite (Story 14-3a) — NULL = aucun',
    ADD COLUMN postable BOOLEAN NOT NULL DEFAULT TRUE
        COMMENT 'FALSE = compte titre/regroupement ou compte de résultat calculé',
    ADD CONSTRAINT chk_accounts_role CHECK (
        role IS NULL OR BINARY role IN (
            BINARY 'Receivable',
            BINARY 'DefaultRevenue',
            BINARY 'Payable',
            BINARY 'VatRecoverable',
            BINARY 'VatPayable',
            BINARY 'VatSettlement',
            BINARY 'EquityCapital',
            BINARY 'EquityOther',
            BINARY 'RetainedEarnings',
            BINARY 'CurrentYearResult'
        )
    ),
    ADD COLUMN singleton_role VARCHAR(32) GENERATED ALWAYS AS (
        CASE WHEN active AND role IN (
            'Receivable',
            'DefaultRevenue',
            'Payable',
            'VatRecoverable',
            'VatPayable',
            'VatSettlement',
            'RetainedEarnings',
            'CurrentYearResult'
        ) THEN role ELSE NULL END
    ) VIRTUAL,
    ADD CONSTRAINT uq_accounts_company_singleton_role UNIQUE (company_id, singleton_role);

-- ---------------------------------------------------------------------------
-- Backfill des installations existantes — best effort documenté.
--
-- Les bases existantes ont un plan comptable créé par Kesh à partir des plans
-- livrés (pme/association/independant), qui utilisent les MÊMES numéros pour 9
-- des 10 rôles. Le numéro sert ici UNE SEULE FOIS, dans une migration de
-- données — jamais dans le code applicatif ; le principe de la story reste tenu.
--
-- LIMITE ASSUMÉE : un utilisateur ayant renuméroté (débiteurs en 1101, 1100
-- réaffecté à autre chose) recevra le rôle sur le mauvais compte, sans erreur.
-- C'est signalé au CHANGELOG et dans le manuel utilisateur : la page Plan
-- comptable permet de tout corriger.
--
-- `AND active = TRUE` : ne jamais donner un rôle singleton à un compte déjà
-- archivé, sinon il bloquerait son remplaçant actif via le UNIQUE.
-- `WHERE role IS NULL` : idempotent, n'écrase jamais un rôle déjà posé.
--
-- EquityOther couvre 2900 (PME) et 2850/2860 (association, indépendant) —
-- ensembles disjoints selon le plan, d'où un seul UPDATE pour les trois. C'est
-- possible précisément parce que EquityOther n'est PAS singleton.
-- ---------------------------------------------------------------------------

UPDATE accounts SET role = 'Receivable'        WHERE number = '1100' AND role IS NULL AND active = TRUE;
UPDATE accounts SET role = 'VatRecoverable'    WHERE number = '1171' AND role IS NULL AND active = TRUE;
UPDATE accounts SET role = 'Payable'           WHERE number = '2000' AND role IS NULL AND active = TRUE;
UPDATE accounts SET role = 'VatPayable'        WHERE number = '2200' AND role IS NULL AND active = TRUE;
UPDATE accounts SET role = 'VatSettlement'     WHERE number = '2206' AND role IS NULL AND active = TRUE;
UPDATE accounts SET role = 'EquityCapital'     WHERE number = '2800' AND role IS NULL AND active = TRUE;
UPDATE accounts SET role = 'EquityOther'       WHERE number IN ('2900', '2850', '2860') AND role IS NULL AND active = TRUE;
UPDATE accounts SET role = 'RetainedEarnings'  WHERE number = '2970' AND role IS NULL AND active = TRUE;
UPDATE accounts SET role = 'CurrentYearResult' WHERE number = '2979' AND role IS NULL AND active = TRUE;
UPDATE accounts SET role = 'DefaultRevenue'    WHERE number = '3000' AND role IS NULL AND active = TRUE;

-- Backfill #1 de `postable` : les comptes titres / de regroupement.
-- PUREMENT STRUCTUREL — aucun numéro. Un compte qui a des enfants est un
-- niveau d'agrégation, on ne poste pas dessus.
-- (MariaDB autorise l'EXISTS corrélé sur la table cible dans un UPDATE — pas
-- d'ERROR 1093, vérifié sur 10.11 ; inutile de passer par une table dérivée.)
UPDATE accounts a
   SET a.postable = FALSE
 WHERE EXISTS (SELECT 1 FROM accounts c WHERE c.parent_id = a.id);

-- Backfill #2 de `postable` : le compte de résultat de l'exercice.
-- En modèle temps réel virtuel (Story 14-1), l'application CALCULE le résultat à
-- chaque rendu du bilan ; y poster une écriture serait un double-comptage.
-- RetainedEarnings reste volontairement postable : un migrant doit pouvoir poser
-- son report à nouveau d'ouverture (persona de la Story 14-4).
-- L'ordre #1 puis #2 est conventionnel — les deux posent la même valeur, leur
-- ordre relatif n'affecte pas le résultat.
UPDATE accounts SET postable = FALSE WHERE role = 'CurrentYearResult';
