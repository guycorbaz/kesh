-- Story 24-2 (#371) : l'écriture d'encaissement d'une facture client.
--
-- NON-BREAKING (CREATE TABLE seul — un binaire antérieur l'ignore) → PAS de bump
-- `kesh_version_min_required`, donc pas de bump de version Cargo (P1/P2/P2-bis).
-- DDL PUR — aucun `UPDATE`, aucun `INSERT` : ne relève NI du registre
-- `POST_RESTORE_BACKFILLS` NI d'une exemption motivée (garde-fou P7).
--
-- ## Pourquoi une table et non une colonne
--
-- Le symétrique fournisseur porte `supplier_invoices.settlement_journal_entry_id`,
-- au singulier, parce qu'une facture fournisseur se règle en une fois. Côté
-- client, non : un débiteur paie en plusieurs virements, et un avoir peut
-- éteindre une partie de la créance. Une colonne unique ne peut pas le dire.
--
-- ⚠️ Cette table est aussi le SUBSTRAT DU LETTRAGE (epic 15, gelé faute d'avoir
-- quoi que ce soit à lettrer avant cette story). Rapprocher des crédits d'un
-- débit sur le compte de créance jusqu'à extinction, c'est la même opération vue
-- de plus loin — ne pas la concevoir contre elle.
--
-- ## Ce que la table NE stocke PAS, délibérément
--
-- Ni le solde résiduel, ni un état « partiellement payée ». Un montant dû rangé
-- en colonne dérive du grand livre à la première divergence, et on aurait
-- recréé un chiffre qui ment — le défaut même que la vague 1 corrige. Le
-- résiduel se CALCULE : TTC − avoir émis − SUM(amount).
--
-- ## `company_id` présent, et c'est le patron du voisin
--
-- `invoice_reminders` (20260715000001), l'autre enfant récent de `invoices`, le
-- porte avec sa FK et son index composite. `journal_entry_lines` ne le porte
-- pas, mais c'est un enfant d'`journal_entries` — une exception documentée, pas
-- la règle.
--
-- ## `ON DELETE` asymétrique, et chaque branche a sa raison
--
-- CASCADE sur la facture (patron `invoice_reminders`, aligné sur la suppression
-- définitive #219). RESTRICT sur l'écriture : une écriture comptable référencée
-- ne se supprime pas, elle se contre-passe.
--
-- ## Les deux contraintes qui portent une règle métier
--
-- `uq_invoice_settlements_entry` : une écriture d'encaissement règle UNE facture.
--   Le règlement groupé — un virement soldant trois factures — n'est pas couvert
--   par cette story ; la contrainte le rend IMPOSSIBLE plutôt que silencieusement
--   faux.
-- `chk_invoice_settlements_amount_positive` : un encaissement négatif serait un
--   remboursement, qui se passe en contre-passation et non en montant signé.

CREATE TABLE invoice_settlements (
    id BIGINT NOT NULL AUTO_INCREMENT,
    company_id BIGINT NOT NULL,
    invoice_id BIGINT NOT NULL,
    journal_entry_id BIGINT NOT NULL,
    amount DECIMAL(19,4) NOT NULL,
    settled_on DATE NOT NULL,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    CONSTRAINT fk_invoice_settlements_invoice
        FOREIGN KEY (invoice_id) REFERENCES invoices(id) ON DELETE CASCADE,
    CONSTRAINT fk_invoice_settlements_company
        FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT fk_invoice_settlements_entry
        FOREIGN KEY (journal_entry_id) REFERENCES journal_entries(id) ON DELETE RESTRICT,
    CONSTRAINT chk_invoice_settlements_amount_positive CHECK (amount > 0),
    UNIQUE KEY uq_invoice_settlements_entry (journal_entry_id),
    INDEX idx_invoice_settlements_company_invoice (company_id, invoice_id),
    INDEX idx_invoice_settlements_invoice (invoice_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
