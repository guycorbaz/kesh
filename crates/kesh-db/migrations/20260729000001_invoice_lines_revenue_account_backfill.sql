-- Story 16-1a-bis (#152, CR #265) — backfill du compte de produit sur le parc
-- de factures et d'avoirs DÉJÀ validés avant le déploiement de la 16-1a.
--
-- ============================================================================
-- POURQUOI CETTE MIGRATION EXISTE
-- ============================================================================
--
-- La 16-1a matérialise le compte de produit effectif dans `invoice_lines` à la
-- transition `draft -> validated` (sa décision D2). Mais une facture DÉJÀ
-- validée n'y repasse jamais (`update` rejette tout statut <> 'draft') et
-- aucune autre écriture ne touche `invoice_lines` après validation : après
-- l'`ADD COLUMN` de `20260727000001`, ces lignes portent `NULL` DÉFINITIVEMENT.
--
-- Or `NULL` est le seul cas qui existe en production à l'instant du
-- déploiement. La 16-1a seule protège donc un ensemble VIDE, et 100 % du parc
-- validé reste exposé au bug qu'elle ferme :
--
--   T1   facture validée, ligne NULL, défaut société = 3000  -> crédit 3000
--   T1+  l'administrateur change le défaut                   -> 3200
--   T2   avoir total émis (relit le défaut COURANT)          -> débit  3200
--
-- Résidu permanent : bilan équilibré, compte de résultat FAUX, aucun signal.
--
-- Le backfill est PRÉVENTIF, pas curatif (décision D-B7) : il arme les
-- factures passées pour que leurs avoirs À VENIR extournent le compte
-- réellement crédité. Pour un couple facture/avoir déjà soldé, l'écriture
-- d'avoir existe et a réellement débité l'autre compte — c'est irréversible.
-- Le backfill ENREGISTRE alors fidèlement les deux comptes, qui DIFFÈRENT
-- légitimement ; forcer leur égalité falsifierait les pièces sans corriger la
-- moindre écriture. Corriger un couple soldé exige une écriture de
-- reclassement : un acte comptable de l'utilisateur, hors de portée d'une
-- migration.
--
-- ============================================================================
-- LE CRITÈRE D'UNICITÉ EN TROIS CONDITIONS (D-B2) — LE CŒUR DE CE FICHIER
-- ============================================================================
--
-- Source de vérité : l'écriture comptable que la pièce a EFFECTIVEMENT
-- produite, JAMAIS `settings.default_revenue_account_id` courant (D-B1). Si
-- l'administrateur a déjà changé ce défaut, backfiller avec la valeur du jour
-- écrirait un compte que la facture n'a JAMAIS crédité : on FABRIQUERAIT la
-- corruption au lieu de la fermer.
--
-- Mais la pièce probante n'est PAS garantie intacte. La structure canonique
-- « [0] débit créance / [1] crédit produit / [2..] crédit TVA » décrit ce que
-- `generate_invoice_journal_lines` PRODUIT, pas ce que la table CONTIENT :
--
--   - `journal_entries` ne porte AUCUNE colonne `source` / `origin` / `is_auto`
--     (schéma `20260412000001_journal_entries.sql:17-36`) — une écriture
--     générée est indiscernable d'une écriture manuelle ;
--   - `PUT /api/v1/journal-entries/{id}` est exposée et n'a AUCUNE garde de
--     provenance (`kesh-api/src/lib.rs` -> `routes/journal_entries.rs` ->
--     `repositories/journal_entries.rs`) ;
--   - `journal_entries::update` DELETE puis ré-INSERT les lignes en
--     RÉATTRIBUANT les `line_order` à partir de zéro.
--
-- Les deux identifications naïves échouent donc SILENCIEUSEMENT : la
-- positionnelle (`line_order = 2`) parce que la 2e ligne peut être n'importe
-- quoi après ré-INSERT ; celle par élimination (« tout ce qui n'est ni la
-- créance ni la TVA ») parce qu'une écriture éditée peut en laisser zéro ou
-- plusieurs. Et l'erreur n'est pas neutre : un mauvais compte écrit ici
-- devient LA vérité de la facture, que la 16-1a D5 recopiera dans tout avoir
-- futur. Un backfill approximatif fabrique exactement la corruption qu'il
-- existe pour fermer, en lui donnant l'apparence d'une donnée établie.
--
-- D'où la règle : ne backfiller QUE s'il existe dans l'écriture EXACTEMENT UNE
-- ligne remplissant les trois conditions
--
--   (1) `credit > 0`    (donc `debit = 0`, exclusivité garantie par
--                        `chk_jel_debit_credit_exclusive`) ;
--   (2) son `account_id` n'est ni la créance ni la TVA due de la config,
--       les valeurs NULL étant IGNORÉES (cf. bloc NULL-safe ci-dessous) ;
--   (3) son `credit` est ÉGAL à `invoices.total_amount` — le HT.
--
-- Plus, sur le volet FACTURE UNIQUEMENT (cf. le bloc dédié au-dessus de
-- l'`UPDATE` (1)) :
--
--   (4) son compte est IMPUTABLE (`accounts.postable = TRUE`), parce que
--       `invoice_lines.revenue_account_id` est une source de vérité pour
--       l'avenir, pas un instantané du passé.
--
-- La condition (3) est LE discriminant que ni la position ni l'élimination ne
-- donnent : c'est elle qui distingue une écriture canonique d'une écriture
-- retouchée. Il n'existe pas de colonne `total_ht` ; ne pas la chercher.
--
-- Sûreté numérique : `journal_entry_lines.credit/debit` et
-- `invoices.total_amount` / `credit_notes.total_amount` sont TOUS
-- `DECIMAL(19,4)` — exact en MariaDB, même échelle des deux côtés, aucune
-- comparaison flottante. Les deux valeurs sont identiques PAR CONSTRUCTION :
-- le helper pousse `credit = total_ht = Σ line_total`, et `total_amount` vient
-- de `compute_total` = `Σ compute_line_total`, la MÊME fonction qui écrit
-- `line_total`.
--
-- Zéro candidat, ou plusieurs -> la ligne reste `NULL`. C'EST LA
-- SPÉCIFICATION, PAS UN DÉFAUT (AC-B2) : le backfill est délibérément
-- incomplet. Une post-condition « aucune ligne validée ne reste NULL » serait
-- FAUSSE, et la poursuivre reviendrait à écrire un compte arbitraire sur des
-- données comptables réelles — l'inverse exact de l'objectif. Le décompte du
-- reliquat s'obtient par les requêtes de diagnostic du CHANGELOG (D-B4) ;
-- cette migration ne crée AUCUNE table et n'écrit AUCUN log (D-B4 : une table
-- de rapport ferait tomber `backup_inventory_matches_schema` et entrerait par
-- accident dans le périmètre de l'export d'installation).
--
-- ============================================================================
-- POURQUOI `<=>` ET SURTOUT PAS `<>` (D-B3) — LE PIÈGE DIRIMANT
-- ============================================================================
--
-- Les TROIS colonnes TVA de `company_invoice_settings` sont `NULL` par défaut :
-- l'INSERT d'onboarding ne les énumère pas, le lazy-create insère
-- `(company_id)` seul, et aucune migration ne les renseigne
-- (`20260614000001_vat_accounts_config.sql` crée bien les comptes 1171/2206
-- mais ne pointe jamais les colonnes de config dessus).
--
-- PORTÉE EXACTE — ne pas surestimer, ne pas sous-estimer. Une société dont
-- `default_vat_payable_account_id` est `NULL` n'a pu valider QUE des factures
-- SANS TVA : `validate_invoice` passe cette colonne à
-- `generate_invoice_journal_lines` (`invoices.rs:1794`), qui échoue en
-- `ConfigurationRequired` dès que `total_vat > 0` (`invoices.rs:1497-1500`).
-- La population concernée n'est donc PAS « 100 % du parc » mais les
-- installations EXONÉRÉES DE TVA — cas parfaitement réel en Suisse sous le
-- seuil de CHF 100'000, et cœur de cible de Kesh.
--
-- Écrite `jel.account_id <> cis.default_vat_payable_account_id`, la
-- comparaison à NULL rend le prédicat NULL en logique ternaire SQL, la ligne
-- n'est JAMAIS candidate, et le backfill NO-OPE INTÉGRALEMENT SUR CES
-- INSTALLATIONS — migration en succès, reliquat « très élevé ». Ce mode de
-- défaillance est RIGOUREUSEMENT INDISCERNABLE DU SUCCÈS, puisque le
-- paragraphe précédent pré-autorise un reliquat élevé comme comportement
-- conservateur normal. Seul le test dédié `backfills_when_vat_config_is_null`
-- l'attrape.
--
-- *(Portée corrigée en passe 2 de `bmad-code-review` : la rédaction d'origine
-- affirmait « NULL sur TOUTE installation » donc « no-op sur 100 % du parc ».
-- Le mécanisme est réel, son rayon était surestimé — et le raisonnement faux
-- aurait pu conduire un mainteneur à la conclusion inverse : « mon install a
-- la config, donc `<>` est sûr ».)*
--
-- D'où `NOT (a <=> b)`, l'égalité NULL-safe de MariaDB : une colonne de
-- configuration non renseignée n'exclut rien. JAMAIS `<>` / `!=` / `NOT IN`,
-- qui propagent NULL.
--
-- Le `LEFT JOIN` sur `company_invoice_settings` procède de la même logique un
-- cran plus haut : une société SANS ligne de configuration ne doit pas voir
-- ses factures écartées en silence par un INNER JOIN. En pratique la ligne
-- existe toujours pour une facture validée (`get_or_create_default_in_tx`
-- fait `INSERT IGNORE` sur le chemin de validation), mais le backfill ne doit
-- pas dépendre de cette propriété : absence de config = ensemble d'exclusion
-- vide, exactement comme une colonne NULL.
--
-- L'ensemble d'exclusion se limite à la créance et à la TVA DUE.
-- `default_vat_recoverable_account_id` et `default_vat_decompte_account_id`
-- n'apparaissent JAMAIS dans une écriture de vente (le helper ne reçoit que le
-- compte de TVA due) : les inclure n'ajouterait rien et multiplierait les
-- occasions de propager un NULL.
--
-- L'exclusion de la créance, elle, ne peut RIEN ajouter et peut RETIRER. La
-- ligne de créance est déjà éliminée par (1) : c'est un DÉBIT sur l'écriture
-- facture, un CRÉDIT sur celle d'avoir. Mais la clause ne porte pas sur cette
-- ligne-là — elle porte sur `jel.account_id` de TOUTE ligne. Si
-- `default_receivable_account_id` désigne aujourd'hui un compte qui figure
-- comme crédit de produit dans des écritures passées (réglage changé depuis,
-- ou plan atypique — `account_type` n'est jamais vérifié à la configuration),
-- une facture parfaitement canonique perd son unique candidat et reste `NULL`.
--
-- Conservée malgré tout : le pire cas reste CONSERVATEUR (une ligne à `NULL`,
-- jamais un compte faux), le scénario suppose une configuration inhabituelle,
-- et la symétrie avec le volet avoir aide à la lecture. Mais ce n'est PAS de
-- la « défense en profondeur » — une clause dont le meilleur cas est de ne
-- rien faire et le pire de supprimer un vrai positif est un risque net. À
-- garder en tête si le reliquat de diagnostic paraît inexplicablement élevé.
--
-- *(Requalifiée en passe 2 de `bmad-code-review`, arbitrage Guy du
-- 2026-07-29 : garder la clause, corriger le commentaire qui la décrivait
-- comme son contraire.)*
--
-- ============================================================================
-- PORTÉE, IDEMPOTENCE, COMPATIBILITÉ
-- ============================================================================
--
-- Portée (D-B5) : factures `validated` et avoirs `issued` seulement. Les
-- `draft` restent NULL — c'est le sens même de la liaison tardive (16-1a D2).
-- Les factures `cancelled` sont exclues DÉLIBÉRÉMENT, et NON « parce qu'elles
-- n'auraient pas d'écriture » : elles en ont toujours une. Le seul chemin vers
-- ce statut est l'émission d'un avoir, et `uq_credit_notes_invoice` interdit
-- un SECOND avoir — aucun résidu futur n'est possible, il n'y a rien à
-- prévenir. Conséquence assumée : sur une facture créditée,
-- `credit_note_lines.revenue_account_id` sera renseigné alors que
-- `invoice_lines.revenue_account_id` restera NULL. Visible à l'export CSV,
-- sans effet comptable.
--
-- Le prédicat `journal_entry_id IS NOT NULL` est une garde défensive
-- REDONDANTE avec `chk_invoices_validated_has_je` (et son pendant
-- `chk_credit_notes_issued_has_je`) : une facture validée a TOUJOURS une
-- écriture, et la FK `ON DELETE RESTRICT` interdit de la supprimer après coup.
--
-- Idempotence (D-B6) : chaque `UPDATE` est gardé par `revenue_account_id IS
-- NULL` et fondé sur un critère DÉTERMINISTE — un re-jeu recalcule le même
-- résultat et n'a aucun effet. Le backfill est donc INTRINSÈQUEMENT
-- IDEMPOTENT, comme ceux de `20260628000001_supplier_invoices.sql` et
-- `20260722000001_accounts_role_postable.sql`. Le verdict `tracked-by-sqlx` de
-- `docs/migrations-idempotence-audit.md` tient à l'absence d'`IF NOT EXISTS`,
-- PAS au backfill.
--
-- Non-breaking : aucune opération `DROP` / `RENAME` / `MODIFY COLUMN`, aucune
-- création de table, deux `UPDATE` de données uniquement.
-- -> PAS de bump `kesh_version_min_required` (P1/P2 de CLAUDE.md), donc PAS de
-- bump de version Cargo (P2-bis).
--
-- Ordonnancement (T-B0) : ce fichier DOIT porter un timestamp strictement
-- postérieur à `20260727000001_invoice_lines_revenue_account.sql`, qui crée la
-- colonne — `sqlx::migrate!` exécute dans l'ordre LEXICOGRAPHIQUE du nom de
-- fichier. 20260729 > 20260727 : vérifié.
--
-- Faisabilité MariaDB : la restriction ER 1093 (« can't specify target table
-- for update in FROM clause ») ne s'applique pas — la sous-requête de
-- candidats lit `invoices` / `journal_entry_lines` / `company_invoice_settings`
-- (miroir : `credit_notes`), JAMAIS la table cible. Précédent d'`UPDATE`
-- multi-table en migration : `20260628000001_supplier_invoices.sql:115`. Pas
-- de CTE (`WITH`) : aucune migration du dépôt n'en contient.

-- ---------------------------------------------------------------------------
-- (1) Factures validées — `credit`, `invoices.total_amount`.
-- ---------------------------------------------------------------------------
--
-- `MIN(jel.account_id)` n'est pas un choix parmi plusieurs : le `HAVING
-- COUNT(*) = 1` garantit qu'il n'y a qu'une ligne dans le groupe, donc `MIN`
-- EST cette ligne. L'agrégat n'est là que parce que SQL l'exige sur une
-- colonne non groupée.
-- Condition (4), volet FACTURE UNIQUEMENT : le compte doit être IMPUTABLE.
--
-- `invoice_lines.revenue_account_id` n'est PAS un instantané du passé : c'est
-- la source de vérité que la 16-1a D5 recopiera dans TOUT avoir futur. Y écrire
-- un compte collectif non-imputable produirait une donnée que l'application
-- elle-même REFUSE à la saisie (`RevenueAccountRejection::NotPostable`) — le
-- backfill fabriquerait un état inatteignable par le chemin nominal.
--
-- Le cas est atteignable : la validation d'écriture tourne avec
-- `enforce_postable = false`, donc une écriture ancienne A PU créditer un
-- compte collectif. Un tel candidat est écarté AVANT le `HAVING`, donc la
-- ligne reste `NULL` — comportement conservateur déjà spécifié par AC-B2.
--
-- Le volet avoir (2) n'a délibérément PAS cette garde : une contre-passation
-- doit viser les MÊMES comptes que l'écriture d'origine, quelle qu'ait été
-- leur évolution de configuration (`credit_notes.rs:405-409`). La dissymétrie
-- est voulue et suit celle des deux champs.
--
-- *(Ajouté en passe 2 de `bmad-code-review`, arbitrage Guy du 2026-07-29.
-- La passe 1 avait écarté ce point en invoquant le rationale de la
-- contre-passation — qui ne s'applique qu'au volet (2).)*
UPDATE invoice_lines il
    INNER JOIN (
        SELECT i.id AS invoice_id, MIN(jel.account_id) AS account_id
        FROM invoices i
            INNER JOIN journal_entry_lines jel ON jel.entry_id = i.journal_entry_id
            INNER JOIN accounts a ON a.id = jel.account_id AND a.postable = TRUE
            LEFT JOIN company_invoice_settings cis ON cis.company_id = i.company_id
        WHERE i.status = 'validated'
          AND i.journal_entry_id IS NOT NULL
          AND jel.credit > 0
          AND NOT (jel.account_id <=> cis.default_receivable_account_id)
          AND NOT (jel.account_id <=> cis.default_vat_payable_account_id)
          AND jel.credit = i.total_amount
        GROUP BY i.id
        HAVING COUNT(*) = 1
    ) c ON c.invoice_id = il.invoice_id
    SET il.revenue_account_id = c.account_id
    WHERE il.revenue_account_id IS NULL;

-- ---------------------------------------------------------------------------
-- (2) Miroir avoir — `debit` au lieu de `credit`, `credit_notes.total_amount`
--     au lieu d'`invoices.total_amount`, statut `issued`.
-- ---------------------------------------------------------------------------
--
-- Le compte est déterminé INDÉPENDAMMENT de la facture d'origine (D-B1) :
-- c'est ce que l'écriture de contre-passation a réellement débité. Pour un
-- couple antérieur au déploiement dont le défaut société a changé entre les
-- deux émissions, ce compte DIFFÈRE légitimement de celui de la facture
-- (D-B7). C'est le résultat correct, et il ne faut surtout pas chercher à
-- l'harmoniser.
--
-- `credit_notes.total_amount` est bien du HT, miroir strict
-- d'`invoices.total_amount` (commentaire de schéma
-- `20260627000001_credit_notes.sql:19`). Les avoirs partiels n'existent pas :
-- `create_credit_note` snapshote TOUTES les `invoice_lines` sans filtre.
UPDATE credit_note_lines cnl
    INNER JOIN (
        SELECT cn.id AS credit_note_id, MIN(jel.account_id) AS account_id
        FROM credit_notes cn
            INNER JOIN journal_entry_lines jel ON jel.entry_id = cn.journal_entry_id
            LEFT JOIN company_invoice_settings cis ON cis.company_id = cn.company_id
        WHERE cn.status = 'issued'
          AND cn.journal_entry_id IS NOT NULL
          AND jel.debit > 0
          AND NOT (jel.account_id <=> cis.default_receivable_account_id)
          AND NOT (jel.account_id <=> cis.default_vat_payable_account_id)
          AND jel.debit = cn.total_amount
        GROUP BY cn.id
        HAVING COUNT(*) = 1
    ) c ON c.credit_note_id = cnl.credit_note_id
    SET cnl.revenue_account_id = c.account_id
    WHERE cnl.revenue_account_id IS NULL;
