-- Story 20-3b1 : marquage « envoyée par e-mail » (décision #16 epic-20).
--
-- `emailed_at` : horodatage du dernier envoi (pattern `paid_at`). Le renvoi
--   est autorisé et écrase la valeur ; chaque envoi est audité
--   (`invoice.emailed`).
-- `emailed_to` : destinataire réellement utilisé au dernier envoi
--   (snapshot de `contacts.email` au moment de l'envoi).
--
-- Non-breaking (ADD COLUMN nullable) → pas de bump `kesh_version_min_required`.
ALTER TABLE invoices
    ADD COLUMN emailed_at DATETIME(6) NULL,
    ADD COLUMN emailed_to VARCHAR(320) NULL;
