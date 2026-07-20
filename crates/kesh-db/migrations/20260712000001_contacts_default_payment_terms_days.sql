-- Story 21-1 (#245) : délai de paiement structuré (jours) sur le contact.
--
-- Le texte libre `default_payment_terms` (VARCHAR(100), FR28) est conservé en
-- lecture pour les contacts non migrés ; quand `default_payment_terms_days`
-- est renseigné, il prime (libellé auto-généré côté API dans la langue du
-- contact, échéance de facture pré-calculée `date + jours`).
--
-- Non-breaking (ADD COLUMN nullable) → pas de bump `kesh_version_min_required`
-- (politique Story 10-2 P1).

ALTER TABLE contacts
    ADD COLUMN default_payment_terms_days INT NULL;

ALTER TABLE contacts
    ADD CONSTRAINT chk_contacts_payment_terms_days
        CHECK (default_payment_terms_days IS NULL
               OR (default_payment_terms_days >= 0 AND default_payment_terms_days <= 365));
