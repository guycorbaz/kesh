-- BREAKING (P1, P3 : MODIFY COLUMN) : relève kesh_version_min_required à 0.17.6
-- (dernière instruction). Une 0.17.5 lit `bank_accounts.iban` en `String` : un
-- compte sans IBAN ferait échouer le décodage de toute liste de comptes (accueil,
-- paramètres, lots de paiement), et l'import d'une sauvegarde ignorerait
-- `account_kind`, rendant bancaire une carte de crédit. Le bump refuse ce
-- binaire au démarrage et à l'import d'une sauvegarde, en le nommant.
--
-- Story 34-11a1 (#972) — un compte sans IBAN (C-e341-13, 14, 17).
--
-- 1. iban devient nullable : un compte sans IBAN (carte de crédit, et plus tard
--    d'autres types) n'en porte pas. La contrainte
--    chk_bank_accounts_iban_nonempty reste telle quelle : un CHECK dont
--    l'expression vaut NULL est satisfait, si bien qu'elle continue de refuser
--    un IBAN vide ou fait d'espaces, et laisse passer NULL.
-- 2. account_kind : le type du compte se déclare (bank | credit_card), il ne se
--    déduit pas d'un IBAN nul. Les comptes existants prennent 'bank' par le
--    DEFAULT, c'est-à-dire exactement leur nature d'avant.
-- 3. card_last4 : les quatre derniers chiffres d'une carte, seuls gardés.
-- 4. Invariants au schéma (C-e341-17) :
--    - un compte bancaire a un IBAN ;
--    - une carte n'a ni IBAN, ni QR-IBAN, et n'est jamais le compte principal
--      (le principal porte la QR-facture, qui lit son IBAN) ;
--    - les quatre chiffres ne vont qu'à une carte, et sont quatre chiffres.
--
-- Aucune écriture de données applicatives : les lignes existantes satisfont les
-- nouvelles contraintes par construction (iban NOT NULL jusqu'ici, type 'bank'
-- par défaut, card_last4 NULL). Le seul statement d'écriture est le bump
-- _kesh_version, table système jamais restaurée, d'où l'entrée
-- EXEMPT_MIGRATIONS (P7). Ligne ajoutée à docs/migrations-idempotence-audit.md
-- (P5).
ALTER TABLE bank_accounts
    MODIFY COLUMN iban VARCHAR(34) NULL
        COMMENT 'IBAN normalisé sans espaces ; NULL pour un compte sans IBAN (carte de crédit)',
    ADD COLUMN account_kind VARCHAR(20) NOT NULL DEFAULT 'bank'
        COMMENT 'Type du compte : bank (IBAN obligatoire) ou credit_card (sans IBAN)',
    ADD COLUMN card_last4 CHAR(4) NULL
        COMMENT 'Quatre derniers chiffres d''une carte ; le numéro complet n''est jamais gardé',
    ADD CONSTRAINT chk_bank_accounts_account_kind
        CHECK (account_kind IN ('bank', 'credit_card')),
    ADD CONSTRAINT chk_bank_accounts_bank_has_iban
        CHECK (account_kind <> 'bank' OR iban IS NOT NULL),
    ADD CONSTRAINT chk_bank_accounts_card_shape
        CHECK (account_kind <> 'credit_card' OR (iban IS NULL AND qr_iban IS NULL AND is_primary = FALSE)),
    ADD CONSTRAINT chk_bank_accounts_card_last4
        CHECK (card_last4 IS NULL OR (account_kind = 'credit_card' AND card_last4 REGEXP '^[0-9]{4}$'));

UPDATE _kesh_version SET kesh_version_min_required = '0.17.6' WHERE id = 1;
