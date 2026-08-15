-- Story 22-1 (#294, #295) : unicité CANONIQUE du numéro de client.
--
-- La 16-3b portait l'unicité sur la valeur saisie elle-même, via la colonne
-- générée `client_number_uniq` — trois chemins la défaisaient : composition
-- Unicode (NFC vs NFD), caractère invisible encastré (ZWSP d'un copier-coller),
-- et collation héritée du serveur (`contacts` ne déclarait aucun COLLATE).
--
-- Ici, l'unicité déménage sur une colonne de COMPARAISON dédiée,
-- `client_number_canonical`, calculée EN RUST à l'écriture
-- (`kesh_core::text::canonical_key` : retrait des invisibles de largeur nulle
-- → NFKC → trim → repli de casse). La valeur affichée reste intacte.
--
-- ⚠️ COLLATE utf8mb4_bin EXPLICITE : la canonicalisation Rust a déjà replié
-- tout ce qui doit l'être — l'égalité d'index doit être l'égalité d'OCTETS.
-- Hériter du défaut du serveur reproduirait #295 sur la colonne neuve (une
-- collation UCA accent-insensible fusionnerait `cli-é1` et `cli-e1`, deux
-- canoniques légitimement distinctes).
--
-- La contrainte GARDE SON NOM (`uq_contacts_company_client_number`) : le
-- mapping d'erreur (`map_contact_error` → 409 CLIENT_NUMBER_ALREADY_EXISTS)
-- reconnaît la contrainte par son nom et ne change pas.
--
-- DDL PUR : le remplissage du parc existant est fait par
-- `backfill_client_number_canonical` (kesh-db), appelée au BOOT et en fin
-- d'IMPORT — décision D6 de la story. MariaDB ne sait ni normaliser NFKC ni
-- retirer un jeu ouvert d'invisibles, un backfill SQL est impossible. Le seul
-- statement d'écriture ci-dessous est le bump `_kesh_version`, table système
-- jamais exportée → EXEMPT_MIGRATIONS (P7), même triage que 20260714000002.
--
-- ---------------------------------------------------------------------------
-- BREAKING (P1/P3) : DROP de `client_number_uniq` (recréée sur la canonique) et
-- surtout changement de SÉMANTIQUE d'écriture — un binaire < 0.10.0 écrirait
-- `client_number` sans sa canonique : la ligne resterait NULL sur la colonne
-- d'unicité et échapperait DÉFINITIVEMENT à la contrainte, rouvrant #294 en
-- silence. D'où le bump `kesh_version_min_required` en dernière instruction
-- (P2) et le bump Cargo des 10 crates dans le même commit (P2-bis).
ALTER TABLE contacts
    DROP INDEX uq_contacts_company_client_number,
    DROP COLUMN client_number_uniq,
    ADD COLUMN client_number_canonical VARCHAR(50)
        CHARACTER SET utf8mb4 COLLATE utf8mb4_bin NULL
        COMMENT 'Forme canonique de client_number (kesh_core::text::canonical_key, Story 22-1) — colonne de comparaison, jamais affichée'
        AFTER client_number,
    ADD COLUMN client_number_uniq VARCHAR(50)
        CHARACTER SET utf8mb4 COLLATE utf8mb4_bin
        GENERATED ALWAYS AS (
            CASE WHEN active THEN client_number_canonical ELSE NULL END
        ) VIRTUAL,
    ADD CONSTRAINT uq_contacts_company_client_number UNIQUE (company_id, client_number_uniq);

UPDATE _kesh_version SET kesh_version_min_required = '0.10.0' WHERE id = 1;
