-- Story 25-2-c (#381) — LE NUMÉRO D'ÉCRITURE VIENT D'UN COMPTEUR.
--
-- `create_in_tx` tirait son numéro d'un `SELECT COALESCE(MAX(entry_number), 0) + 1`.
-- L'`UNIQUE (company_id, fiscal_year_id, entry_number)` garantit l'unicité À UN
-- INSTANT DONNÉ — ni la contiguïté, ni l'univocité DANS LE TEMPS :
--
--   * supprimer l'écriture n° 42 au milieu laisse un TROU définitif — visible,
--     et un contrôleur en demandera l'explication ;
--   * supprimer la DERNIÈRE fait RÉATTRIBUER son numéro à une écriture au
--     contenu différent — muet, et rien ne le signalera jamais.
--
-- C'est la réattribution que cette migration ferme. Le trou subsiste et reste
-- explicable : il correspond à une facture supprimée, que le journal d'audit
-- nomme.
--
-- ⚠️ LE REMÈDE N'EST PAS INVENTÉ ICI. `invoice_number_sequences`
-- (20260417000001) est déjà un compteur persistant, de portée
-- `(company_id, fiscal_year_id)` — EXACTEMENT celle d'`entry_number` — et le
-- motif est déjà réemployé par `credit_note_number_sequences`. Le `MAX + 1` des
-- écritures était l'exception du dépôt ; cette table l'aligne. Noms de
-- contraintes, colonnes et ordre calqués sur son modèle, jusqu'au `version`.
--
-- P2 (`CLAUDE.md` § Migration breaking policy) : NON-BREAKING. Un binaire
-- antérieur ignore une table qu'il ne lit pas, et continue de servir son
-- `MAX + 1`. Aucun `kesh_version_min_required`, donc aucun bump Cargo.

CREATE TABLE journal_entry_number_sequences (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    company_id BIGINT NOT NULL,
    fiscal_year_id BIGINT NOT NULL,
    next_number BIGINT NOT NULL DEFAULT 1 COMMENT 'Prochain numéro à attribuer. Ne redescend JAMAIS : c''est toute la propriété qui manquait au MAX+1.',
    version INT NOT NULL DEFAULT 1,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    CONSTRAINT fk_jens_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT fk_jens_fiscal_year FOREIGN KEY (fiscal_year_id) REFERENCES fiscal_years(id) ON DELETE RESTRICT,
    CONSTRAINT uq_jens_company_fy UNIQUE (company_id, fiscal_year_id),
    CONSTRAINT chk_jens_next_positive CHECK (next_number >= 1)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- AMORÇAGE depuis l'existant.
--
-- ⛔ SANS CETTE ÉTAPE, LA MIGRATION FAIT L'INVERSE DE SON BUT : sur une
-- installation en service, le compteur partirait de 1 et réattribuerait aussitôt
-- des numéros déjà portés — la contrainte d'unicité refuserait l'insertion, et
-- la première écriture suivante échouerait.
--
-- Une écriture n'appartenant qu'à un seul exercice (`fiscal_year_id` NOT NULL,
-- jamais réécrit), le `GROUP BY` rend une ligne par couple réellement mouvementé.
-- Les couples sans écriture n'ont pas de ligne : `next_number_for` les crée à la
-- demande, comme le fait son modèle.
INSERT INTO journal_entry_number_sequences (company_id, fiscal_year_id, next_number)
SELECT company_id, fiscal_year_id, MAX(entry_number) + 1
  FROM journal_entries
 GROUP BY company_id, fiscal_year_id;
