# Story 38-6c2 : rattachement d'une transaction à un groupe d'écritures (B28) — paires suggérées, annulation du groupe

Status: deferred

*(2026-11-20 — reportée en phase 2 par la décision du responsable de l'epic (phase 1 KISS, § « Phase 1 KISS » du fichier d'epic) ; validée le 2026-11-15, gardée telle quelle comme base de la phase 2. Ce qu'en reprend la phase 1 est dit dans les fiches 38-11a à 38-11d.)*

> **À revoir après la fusion de l'Epic 36** : Guy construit #823 (36-6, une transaction rattachée à plusieurs
> écritures) ; réutiliser son mécanisme plutôt que doubler la table `bank_transaction_extra_entries`.

*Fiche née le 2026-11-15 du découpage de 38-6c (D-P2-3, `38-p2-inter-fiches.md`) : elle en porte les AC-12 à AC-15,
renumérotés **AC-1 à AC-4** (AC-12 → AC-1, AC-13 → AC-2, AC-14 → AC-3, AC-15 → AC-4), et la migration de l'ancien
AC-9 (dans l'AC-1). Historique, rédaction, validations P1 et P2 : `38-6c-rattachement-en-lot-et-api.md` (`split`).
Branche `epic/38-paypal`, numéros de ligne relevés sur `8b3d548d` et, pour l'inventaire de l'AC-1, sur `57b26a5f`.
Epic 38 (`_bmad-output/planning-artifacts/epic-38-paypal.md`) : E15, E21, C-38-0, C-38-12, C-38-13, C-38-14 ;
décision du responsable de l'epic **B28** (`epics-38-40-cadrage.md`). **Dépend de 38-6c1** (base de mouvement `MovementBasis`,
`entry_movements(basis)`, `entry_sql(mode, basis)`, `entry_matches_for_page`, `link-suggestions`, lot
`link-entries`, `diagnose_entry_link`, `expected_version` — étendus ici, jamais recopiés) et, à travers elle, de 38-3a,
38-2 et de l'Epic 37. **Se développe avant 38-6a** (D-P2-3 : 38-6a lit `pairCount` pour « déjà comptabilisée »,
AC-3). Fiches sœurs : 38-6b (ses poses de `matched_entry_id`, AC-1 (f)), 38-9b (écran des paires et des groupes,
manuel). **Une migration non breaking** (`bank_transaction_extra_entries`, AC-1 ; l'epic n'en attendait pas —
signalée par la révision B27–B28, C-38-0).*

## Story

En tant que comptable dont les livres 2024 portent la vente PayPal **au brut** et ses frais par une **écriture à
part** (B28), je veux rattacher la transaction téléchargée à **ces deux écritures** — et que Kesh me suggère la paire
quand aucune écriture seule ne correspond —, sans qu'aucune écriture ne soit créée ni doublée ; et, si je me suis
trompé, annuler le rattachement du groupe entier en une fois.

## Exigences

Reprises de 38-6c (B28), renumérotées au découpage (`FR-38-6c-8..10` → `FR-38-6c2-1..3` ; `NFR-38-6c-5..6` →
`NFR-38-6c2-1..2`).

- **FR**
  1. `FR-38-6c2-1` *(B28)* — Une transaction se rattache, à l'unité comme en lot, à un **groupe** de deux à cinq
     écritures existantes dont les mouvements sur le compte du prestataire somment **exactement** à son montant
     (typiquement la vente au brut et l'écriture de frais séparée de 2024). → AC-1, AC-2.
  2. `FR-38-6c2-2` *(B28)* — Kesh suggère une **paire** (deux écritures datées à ±3 jours de la transaction, de somme
     exacte) quand aucune écriture seule ne correspond : univoque, ou « ambiguë » ; une paire compte comme « déjà
     comptabilisée » (34-4). → AC-3.
  3. `FR-38-6c2-3` *(B28)* — L'annulation d'une transaction rattachée à un groupe défait **tout le groupe** en une
     fois. → AC-4.
- **NFR**
  1. `NFR-38-6c2-1` *(B28)* — Migration **non breaking** (`CREATE TABLE` seul) ; une écriture appartient au plus à un
     groupe, tenu par la base (`UNIQUE (journal_entry_id)`) et par la règle sous verrou. → AC-1.
  2. `NFR-38-6c2-2` *(B28)* — « Écriture rapprochée » a **une** définition (fragment SQL partagé, lien direct **ou**
     membre d'un groupe), lue par **tous** les sites de l'inventaire clos de l'AC-1 ; aucun ne relit
     `matched_entry_id` seul. → AC-1, AC-4.
  3. `NFR-38-6c2-3` *(ajoutée au découpage)* — Le groupe et la paire se posent dans les textes et les fonctions de
     38-6c1 (une seule règle : lecture par page égale au dialogue, paires comprises ; *Pattern batch* au lot ; trois
     lectures par page au plus, sous la cible de temps de 38-6c1 AC-8). → AC-2, AC-3.

## Écarts à l'epic

1. **Écritures 2024 « au brut, frais à part » — tranché par B28 : elles se rattachent.** Dans les livres 2024 du responsable de l'epic,
   les frais PayPal sont des **écritures à part** (B28) : une vente passée au brut (débit PayPal 100) et ses frais par
   une autre écriture (crédit PayPal 3.20) ; aucune écriture seule ne vaut 96.80. L'epic (E21) ne prévoyait que « une
   transaction ↔ une écriture » ; or le modèle actuel ne peut pas dire plus : **une** colonne,
   `bank_transactions.matched_entry_id` (`20260504000001_bank_imports.sql:71`, FK `ON DELETE SET NULL`), une écriture
   par transaction — l'inverse (deux transactions sur une écriture, 34-11b) étant déjà admis. D'où une **table de
   liaison** pour les écritures **en plus** de la première (AC-1, migration non breaking), le rattachement à un groupe
   à l'unité et en lot (AC-2), la suggestion d'une **paire** (AC-3) et l'annulation du groupe (AC-4). La règle
   « somme de plusieurs écritures » **propose et ne tranche jamais** (*Un appariement automatique propose, il ne crée
   jamais*) : la paire n'est suggérée que si aucune écriture seule ne correspond, datée à ±3 jours, de somme exacte,
   et l'utilisateur coche ; plusieurs paires possibles → « ambiguë ». Une écriture de frais **mensuelle** (frais
   d'un mois en une écriture) ne forme pas de paire exacte avec une vente : elle reste hors de cette règle et se traite
   à la main (limite écrite au doc-comment et au manuel, 38-9b). Si les frais sont une ligne de la même écriture, la
   règle d'une écriture (38-6c1) suffit (inchangé).
2. **Une migration que l'epic n'attendait pas** (C-38-0) : `CREATE TABLE` seul, signalée à la révision B27–B28 ;
   `kesh_version_min_required` non relevé par elle (celui de la release est celui de 38-2).

## Critères d'acceptation

1. **AC-1 — Le modèle du groupe, sa migration et la définition unique de « rapprochée »** (`FR-38-6c2-1`,
   `NFR-38-6c2-1`, `NFR-38-6c2-2`, B28). **Relevé au code** : un lien est `bank_transactions.matched_entry_id` (une
   écriture par transaction ; jusqu'à deux transactions par écriture, 34-11b, `linkable_side`,
   `reconciliation_link.rs:739-748`) ; aucune table de lien. **Forme retenue, la plus étroite** : `matched_entry_id`
   garde l'**écriture principale** du groupe — tous ses lecteurs actuels restent justes pour elle —, et les **autres**
   écritures vont dans une table neuve `bank_transaction_extra_entries (bank_transaction_id BIGINT NOT NULL,
   journal_entry_id BIGINT NOT NULL, company_id BIGINT NOT NULL, created_at DATETIME(3) NOT NULL, PRIMARY KEY
   (bank_transaction_id, journal_entry_id), UNIQUE (journal_entry_id), FK bank_transactions / journal_entries /
   companies ON DELETE RESTRICT)`. `RESTRICT` et non `SET NULL` : un lien ne s'efface jamais en silence (en-tête de
   `reconciliation_cancel.rs`, « jamais par la suppression ») ; `UNIQUE (journal_entry_id)` : une écriture est membre
   d'un groupe au plus, tenu par la base. **Écriture principale** : celle de plus grand `|M|`, puis plus petit
   identifiant — décidée par le serveur, jamais par l'ordre du corps. Un lien simple n'écrit **rien** dans la table
   (aucune ligne réécrite, aucun rejeu).
   - **Migration, non breaking** (C-38-0, B28 ; ex-AC-9 de 38-6c). **Un** fichier,
     `<date>000001_bank_transaction_extra_entries.sql` : `CREATE TABLE` seul — ni `DROP`, ni `MODIFY`, ni valeur
     neuve dans une colonne existante, **aucune donnée écrite** : non breaking au sens de P1/P3,
     `kesh_version_min_required` **non relevé** par ce fichier (le relèvement de la release est celui de 38-2,
     C-38-0) ; comme toute migration, il empêche le binaire précédent de démarrer (sqlx), retour par la sauvegarde
     d'avant la mise à jour (E19). Gardes : ligne à `docs/migrations-idempotence-audit.md` et compteurs **recomptés**
     (P5) ; `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` et chaque site inspecté (P6) ; aucun triage
     P7 (aucun `INSERT`, `UPDATE`, `DELETE`) — vérifié par `every_data_backfill_migration_is_triaged`, vert sans
     exemption ; squash `crates/kesh-db/test-schema/` mis à jour (`test_schema_guard`) ; `.keshbackup` : la table entre
     à l'inventaire, à l'export et à l'import complet (ordre des FK : après `bank_transactions` et `journal_entries`),
     test `admin_full_import_e2e` avec un groupe restauré à l'identique ; export CSV des tables
     (`exports/csv_tables.rs`, patron de `bank_transactions` `:759`). **Inscriptions** (validation P2 de 38-6c, au
     patron de 38-2 et 38-3a) : `TABLES_TO_TRUNCATE` (`crates/kesh-db/src/backup.rs:46`, enfants → parents) **avant**
     `bank_transactions` et `journal_entries` ; absente de `RESET_PRESERVED_TABLES` (la remise à zéro la vide) ;
     `backup_inventory_matches_schema` vert ; **liste close des absences admises** (37-1 AC-6, R1, E18) :
     `bank_transaction_extra_entries` en `Absence::Table` — sans elle, `parse_and_verify` (`admin_backup/import.rs`,
     « table '…' absente du manifeste ») refuserait **toute** sauvegarde antérieure ; test d'import d'une sauvegarde
     d'avant l'Epic 38 étendu (aucun groupe) ; export CSV : `TABLES_EXPORTEES` (`exports/global.rs`) et sérialiseur
     écrit à la main avec son cas d'en-tête, **ou** `TABLES_HORS_EXPORT` avec son motif — choix déclaré au Change Log ;
     sites de dénombrement (même grep que 38-1/38-2/38-3a) : `admin_full_export_e2e.rs:~281` (`data_count`, cumulé),
     `onboarding_audit_e2e.rs` (`EMPTY_BEFORE_RESET`). Gate complet en cours de boucle (exception `kesh-db`).
   - **Fragment partagé** (seule définition, `reconciliation_link.rs`) : `LINKED_TRANSACTIONS_OF(e)` = transactions
     `bt` telles que `bt.matched_entry_id = e.id` **ou** une ligne `(bt.id, e.id)` de la table ; `IN_GROUP(e)` = `e`
     est membre d'un groupe (ligne de la table, ou principale d'une transaction qui a une ligne). `linkable_side`,
     `HAS_LINK` et `transfer_other_side` le lisent ; la condition 4 (38-6c1) refuse en plus toute écriture `IN_GROUP`
     (un membre de groupe n'a pas d'autre côté).
   - **Inventaire clos des lecteurs de `matched_entry_id` qui demandent « l'écriture est-elle rapprochée ? »**
     (relevé par `grep -rn matched_entry_id crates/*/src`, sur `57b26a5f`, à recompter à l'étape 0 du développement —
     38-6c1, développée avant, en ajoute ; 38-6a et 38-6b, développées après (D-P2-3), s'y rangent par le relevé (f)
     —, citation au Change Log ; chacun **résolu** ou **écarté
     par construction**, aucun laissé muet) :
     (chemins sous `crates/kesh-db/src/repositories/` sauf mention ; `bank_accounts.rs` est celui du dépôt, non de
     `kesh-api/src/routes/`) (a) `journal_entries.rs:2340-2344` (`document_owners`, rang 7 : motif `MatchedBankTransaction` des refus de
     modification, de contre-passation **et de suppression** — `invoices.rs:1506`, `journal_entries.rs:1813`) —
     **résolu** : lit le fragment, un membre est non modifiable, non contre-passable et non supprimable comme la
     principale (sans quoi la FK `RESTRICT` rendrait une erreur de base au lieu du refus nommé) ; (b)
     `bank_accounts.rs:1884-1898` (écart du relevé : mouvements postérieurs des écritures rapprochées) — **résolu** :
     le fragment, faute de quoi l'écriture de frais d'un groupe manquerait au calcul ; (c) `reconciliation_link.rs`
     (`linkable_side`, `HAS_LINK`, `transfer_other_side`, et le `NOT EXISTS` des règlements `:161`) — **résolu**
     (fragment) pour les trois premiers, **par construction** pour le quatrième ; (d) `reconciliation_cancel.rs:202`,
     `:670` — **résolu** par l'AC-4 ; (e) `invoices.rs:1622`, `supplier_invoices.rs:1318`, `:1445`,
     `settlement_cancellation.rs:142` (écritures de factures et de règlements) — **par construction** : un membre de
     groupe n'est **jamais** une écriture possédée par une pièce (AC-2, condition G4) ; (f) `routes/reconciliation.rs`
     (`is_reconciled`, poses de `matched_entry_id` des huit chemins sur une écriture **neuve**) et `bank_imports.rs`
     (`is_reconciled`) — portent sur la transaction, non sur l'écriture : inchangés ; **et les poses de
     `matched_entry_id` de 38-6b** (paire de conversion, paire à moitié CHF, virement de retrait ou de dépôt) — **par
     construction** : elles portent sur une écriture neuve de l'acceptation, ou sur l'écriture d'un virement par la
     règle de 38-6c1 (conditions 4 et G6 : une écriture `IN_GROUP` n'a jamais d'autre côté), jamais sur un membre de
     groupe ; (g) `crates/kesh-api/src/exports/csv_tables.rs:759`, sauvegarde — ci-dessus. Un test par site résolu : un groupe dont le
     **membre** (non la principale) est visé — refus de contre-passation directe `MatchedBankTransaction`, écart du
     relevé égal à celui d'un lien simple de même mouvement.
   - **Teardowns de test qui suppriment des écritures** (la FK `RESTRICT` de la table neuve les ferait échouer sur
     un groupe résiduel — famille KF-039) : `journal_entries::delete_all_by_company` (`journal_entries.rs:2085`, `pub`,
     sans appelant de production) et `accounts.rs` `drop_company_deep` (`:1464`, `panic!` sur échec) vident la table
     neuve **avant** les écritures ; `test_fixtures::truncate_all` la couvre par `TABLES_TO_TRUNCATE`. Un test de
     chacun avec un groupe en base.
   - **Lecture par l'API des scripts** : `matchedEntryId` (DTO `TransactionResponse`, 38-6c1 AC-6) ne porte que
     l'écriture **principale** ; les autres membres ne se lisent qu'à `groupEntryIds` (vue d'annulation) — limite
     écrite à `docs/api-external.md` avec les sections du groupe.
   Tests : schéma (FK, `UNIQUE`), mutation « fragment remplacé par `matched_entry_id` seul » sur (a) et (b) → rouge.
2. **AC-2 — Rattacher un groupe, à l'unité et en lot** (`FR-38-6c2-1`, `NFR-38-6c2-3`, B28). `POST
   /reconciliation/transactions/{id}/link-entry` admet, **en plus** de `{ journalEntryId }` (inchangé),
   `{ journalEntryIds: [..] }` — exactement un des deux, sinon 400 `VALIDATION_ERROR` ; 2 à 5 identifiants distincts
   (> 0), sinon 400. **Aucune route neuve** (la forme « deux temps » — rattacher puis ajouter — est écartée : entre
   les deux appels, la transaction serait `reconciled` sur un montant faux).
   - **Lot** (38-6c1 AC-5, étendu) : chaque élément porte `journalEntryId` **ou** `journalEntryIds` (exactement un des
     deux) ; ni l'un ni les deux, `journalEntryIds` de longueur hors 2..=5 ou portant un doublon → 400 ; `linked[]`
     gagne `journalEntryIds` (additif : tout le groupe, `[journalEntryId]` pour un lien simple ; `journalEntryId` =
     l'écriture principale).
   - **Règle du groupe** `{e₁…e_k}` pour `t` (une seule définition, `group_sql`, mêmes textes que 38-6c1 AC-1 :
     `entry_movements(basis)`, `ENTRY_NOT_REVERSED_NOR_SETTLEMENT`, fragment de l'AC-1) : condition 1 sur `t`
     (inchangée) ; **G1** chaque `eᵢ` de la société ; **G2** chaque `eᵢ` mouvemente le compte du grand livre, `Mᵢ ≠ 0`
     (une écriture qui ne le touche pas n'entre pas dans un groupe) ; **G3** `Σ Mᵢ = t.amount`, égalité décimale exacte
     dans la base (CHF ou devise, 38-6c1 AC-1) ; **G4** chaque `eᵢ` est une écriture **propre** : ni
     `invoices.journal_entry_id`, ni ligne `invoice_settlements`, ni `supplier_invoices.journal_entry_id` /
     `settlement_journal_entry_id` (critères du classement `reconciliation_cancel::classify`) ; **G5** conditions 5 et
     6 pour chaque `eᵢ` ; **G6** aucune `eᵢ` n'a de transaction liée (`LINKED_TRANSACTIONS_OF`) ni n'est `IN_GROUP` —
     un groupe n'a pas d'« autre côté ».
   - **Verrous** : ceux du geste unitaire (verrou nommé → `t` `FOR UPDATE`), puis **chaque** `eᵢ` `FOR UPDATE` dans
     l'ordre **croissant** des identifiants, **puis seulement** la première lecture ordinaire et la requête restreinte
     à `e.id IN (…)` — la vue de lecture naît après le **dernier** verrou de ligne (invariant de `link_entry_in_tx`,
     mutation M10) ; écriture : `UPDATE bank_transactions SET matched_entry_id = <principale>, status = 'reconciled'
     …` (gardes `status = 'pending' AND matched_entry_id IS NULL`, comme le lien simple) puis `INSERT` des autres ;
     `expected_version` (38-6c1 AC-5) contrôlée au même rang. Trace `reconciliation.entry_linked`, **une** par
     transaction, champ additif `journal_entry_ids` (en snake_case comme les clés existantes de cette trace,
     `reconciliation_link.rs`, et `docs/api-external.md:811`) ; aucune action neuve. La réponse de `…/link-entry`
     (`LinkEntryResponse`, `{ bankTransactionId, journalEntryId }`) gagne le champ additif `journalEntryIds` (la
     principale seule pour un lien simple).
   - **Refus** — geste unitaire : 409 `RECONCILIATION_ENTRY_NOT_LINKABLE` (code inchangé ; le message reformulé par
     38-6c1 gagne la clause « … ou les mouvements des écritures choisies ne somment pas à son montant », quatre
     locales, repli de `errors.rs` égal au catalogue `fr-CH`, G9) ; lot : `failed[]` avec le diagnostic
     (`diagnose_entry_link` étendu au groupe, lecture après refus, n'admet jamais) — `ENTRY_GROUP_SUM_MISMATCH` (G3 :
     `{ transactionAmount, movements: [{ journalEntryId, movement }], sum }`) ; `ENTRY_NOT_LINKABLE`
     (`{ journalEntryId, reason }`, `reason` ∈ ceux de 38-6c1 + `entry_not_touching_account` (G2),
     `entry_owned_by_document` (G4)) ; `ENTRY_ALREADY_LINKED` (G6 : `{ journalEntryId, linkedBankTransactionIds }`) ;
     `AMOUNT_MISMATCH` n'est pas rendu pour un groupe (G3 le remplace) ; ordre : premier membre fautif dans l'ordre
     croissant des identifiants, puis la somme.
   - **Exercice clos** (ex-AC-11 de 38-6c, étendu) : le groupe n'écrit aucune écriture, comme le lien simple — vente et
     frais de 2024 dans un exercice clos se rattachent (200).
   - **Registres et documentation** : aucune route neuve, `LIB_ROUTES` inchangé (C-38-12 : delta 0) ;
     `docs/api-external.md` (sections écrites par 38-6c1) : `journalEntryIds` à `…/link-entry` et dans le lot (2 à 5,
     exclusif de `journalEntryId`), règle de la somme exacte, écriture principale, `ENTRY_GROUP_SUM_MISMATCH` et les
     `reason` neufs d'`ENTRY_NOT_LINKABLE` ; relu contre le code, même PR. Écran et manuel : 38-9b (liste au Change
     Log). Aucun outil MCP neuf (E23).
   - Tests HTTP (compte EUR et compte CHF) : vente +100.00, frais −3.20 (écriture à part), transaction +96.80 → groupe
     rattaché, principale = la vente, une ligne dans la table, trace avec les deux ; même lot avec `journalEntryIds`
     inversés → même principale ; somme fausse (+96.70) → `ENTRY_GROUP_SUM_MISMATCH` ; membre déjà lié à une autre
     transaction → `ENTRY_ALREADY_LINKED` ; membre qui ne touche pas le compte → `entry_not_touching_account` ; membre
     écriture de facture → `entry_owned_by_document` ; tentative de lien **simple** d'une autre transaction à
     l'écriture de frais d'un groupe → refusée (condition 4, `IN_GROUP`) ; `journalEntryIds` de longueur 1 ou 6, ou
     avec doublon, ou avec `journalEntryId` → 400 ; concurrence : deux lots qui groupent la même écriture de frais avec
     deux ventes → un seul groupe, l'autre `ENTRY_ALREADY_LINKED` (montage de T-race-3) ; groupe dans un exercice clos
     → 200.
3. **AC-3 — Paires suggérées ; « déjà comptabilisée »** (`FR-38-6c2-2`, `NFR-38-6c2-3`, B28). Une **paire** de `t` :
   deux écritures `{a, b}` satisfaisant G1, G2, G4, G5, G6, `M_a + M_b = t.amount`, datées toutes deux à
   `ENTRY_PAIR_WINDOW_DAYS` = 3 jours au plus de la date de paiement de `t`, **et** `t` sans aucune écriture seule
   candidate (une écriture seule exacte prime : pas de paire à côté). **Une seule définition, deux lectures**
   (validation P2 de 38-6c) : un texte SQL, `PAIR_MEMBERS(basis)` — les écritures qui peuvent entrer dans une paire
   (G1, G2, G4, G5, G6, mouvement `M ≠ 0` sur le compte, datées dans une plage `[d₁, d₂]` liée en paramètre), sans
   auto-jointure — et **une** fonction pure, `pairs_for(t, members) -> Vec<Pair>` (somme exacte, fenêtre de
   `ENTRY_PAIR_WINDOW_DAYS` = 3 jours autour de la date de paiement de `t`, tri par somme des écarts de jours puis
   identifiants, plafond `ENTRY_PAIRS_LIMIT` = 20). Le dialogue (`find_entry_pairs`, lu par `GET …/entry-candidates`,
   qui gagne `pairs`, additif : `[{ entries: [PageEntry, PageEntry] }]`) lit `PAIR_MEMBERS` sur la plage de `t` ;
   `entry_matches_for_page` (38-6c1 AC-3) le lit **une fois par page**, sur l'union des plages (de la plus petite date
   de paiement − 3 jours à la plus grande + 3 jours), puis appelle `pairs_for` par transaction — d'où **trois**
   lectures par page au plus (règlements, écritures, membres de paire), jamais une par transaction, et aucune
   condition recopiée en Rust hors de `pairs_for`. `PageMatches` gagne `pairs` ; le test d'égalité de 38-6c1 AC-3
   (`s34_4_ac8_…`) compare aussi les paires (identifiants et ordre).
   - **Suggestions** (38-6c1 AC-4, étendu, additif) : genre **`uniquePair`** — **aucune** écriture seule ni aucun
     règlement candidat, et **une** paire, dont aucune des deux écritures n'est candidate (seule ou en paire) d'une
     autre transaction de la page ; l'élément porte `entries` (les deux `PageEntry`, mouvement compris) ;
     `ambiguous` s'étend aux transactions qui n'ont que des paires : motif **`several_pairs`**, `entry_shared` étendu
     aux paires, champ `pairs` (≤ 20) ; `none` = aucune écriture, aucun règlement, **aucune paire** candidats.
     Documentation : `uniquePair`, `pairs`, `several_pairs` à `docs/api-external.md` (§ `link-suggestions` et
     `entry-candidates`).
   - **« Déjà comptabilisée » (34-4)** : une paire **ne gonfle pas** `entry_count` (qui doit égaler la longueur de
     `find_entry_candidates`, 38-6c1 AC-3) ; `AlreadyBookedCounts` gagne **`pair_count`** (longueur de
     `find_entry_pairs`, ≤ 20) et le DTO `AlreadyBooked` de `get_proposals` (`routes/reconciliation.rs:140-145`, doc `:128-139`) le
     champ additif **`pairCount`** ; le signal est « vrai » dès qu'un des quatre nombres est non nul — sans quoi 38-6a
     proposerait une **seconde** écriture pour une transaction de 2024 passée au brut avec frais à part, et
     `confirmAlreadyBooked` ne la garderait pas (E15) ; test : transaction à paire seule → `pairCount` 1,
     `entryCount` 0, `alreadyBooked` non nul. (38-6a porte `pairCount` dans `TRANSACTION_ALREADY_BOOKED` ; 38-9b
     l'affiche.)
   - Les groupes de trois écritures et plus ne sont **jamais suggérés** (combinatoire) : ils se forment à la main
     (AC-2), limite écrite au doc-comment et au manuel (38-9b).
   - **Temps** : mesure de 38-6c1 AC-8 reprise avec les paires, même montage, **même cible** (≤ 2 s par page de 500) ;
     cible manquée signalée au Project Lead avant la revue.
   Tests HTTP : paire univoque ; deux paires → `ambiguous` `several_pairs` ; paire hors fenêtre (4 jours) → `none` ;
   écriture seule exacte et paire → seule l'écriture (`unique`) ; écriture partagée entre les paires de deux
   transactions de la page → `entry_shared` ; aucune écriture en base après l'appel.
4. **AC-4 — Annuler un groupe** (`FR-38-6c2-3`, `NFR-38-6c2-2`, B28). `POST /reconciliation/transactions/{id}/cancel`
   d'une transaction rattachée à un groupe défait **tout le groupe**, dans une transaction : verrou de `t`, puis des
   écritures du groupe dans l'ordre croissant ; **le lien d'abord** — `matched_entry_id` à `NULL` et `DELETE` des
   lignes de la table pour `t` —, **puis** chaque écriture traitée comme le genre `Entry` du classement (G4 le
   garantit) : contre-passée par `journal_entries::reverse_in_tx`, comme l'annulation d'un lien simple (v017-3a) ;
   un refus sur **une** écriture (exercice clos, verrou de période — rangs de `entry_cancel_chain`) refuse **tout**,
   rien d'écrit. La vue de l'annulation (`entry_cancel_outlook`) porte en plus `groupEntryIds` et le motif du premier
   refus. **Un seul nom pour les contre-passations d'un groupe : `reversalJournalEntryIds`** (inter-fiches P2 ; le nom
   `groupReversalJournalEntryIds` proposé par 38-9b n'est pas retenu) — dans l'ordre croissant des écritures du groupe :
   - trace d'annulation existante (`reconciliation.cancelled`, clés en camelCase) : gagne `journalEntryIds` et
     `reversalJournalEntryIds` (additifs) ;
   - réponse de `…/cancel` — `CancelReconciliationResponse` (`routes/reconciliation.rs:5834` ; miroir TS
     `reconciliation.types.ts:226`), construite depuis `ReconciliationCancellation` (`reconciliation_cancel.rs:490`) :
     gagne `reversalJournalEntryIds` (additif, **`null` hors groupe**, comme les champs `paired…` de 38-6b AC-8) ;
   - `matchedEntryId` et `reversalJournalEntryId` gardent la principale et sa contre-passation (inchangés).
   ⚠️ **Limite existante, étendue** : annuler un rattachement contre-passe ; dans un exercice clos (2024),
   l'annulation est donc refusée — un groupe rattaché à tort y reste ; à tracer en issue avec la PR (C-38-13), non
   traité ici. Documentation : annulation de tout le groupe à `docs/api-external.md` (contre-passation de chaque
   écriture, `reversalJournalEntryIds`, refus dans un exercice clos). Tests : annulation d'un groupe → deux
   contre-passations, `reversalJournalEntryIds` de longueur 2, table vide pour `t`, `t` `pending` ; lien simple →
   `reversalJournalEntryIds` `null` ; exercice clos → refus, rien d'écrit ; mutation « liens retirés après la
   contre-passation » → rouge (motif `MatchedBankTransaction`).

## Tâches

- [ ] Migration `bank_transaction_extra_entries`, audit P5, P6, squash, sauvegarde (inventaire, `TABLES_TO_TRUNCATE`,
      `Absence::Table`), export CSV, dénombrements (AC-1).
- [ ] `kesh-db::repositories::reconciliation_link` : fragment `LINKED_TRANSACTIONS_OF` / `IN_GROUP` lu par
      `linkable_side`, `HAS_LINK`, `transfer_other_side`, condition 4 ; `group_sql`, verrous et écritures du groupe ;
      `diagnose_entry_link` étendu (AC-1, AC-2) ; `PAIR_MEMBERS`, `pairs_for`, `find_entry_pairs`, `pairs` de
      `entry_matches_for_page`, `pair_count` (AC-3).
- [ ] Sites de l'inventaire (AC-1) : `journal_entries` (`document_owners`), `bank_accounts` (écart du relevé),
      `reconciliation_cancel` (AC-4).
- [ ] `kesh-api` : `journalEntryIds` à `link-entry` et au lot, `linked[].journalEntryIds`, `LinkEntryResponse` (AC-2) ;
      `uniquePair`, `pairs`, `several_pairs` aux suggestions, `pairs` à `entry-candidates`, `pairCount` (AC-3) ;
      `reversalJournalEntryIds` de `CancelReconciliationResponse`, vue `groupEntryIds` (AC-4) ; message reformulé
      (clause du groupe, quatre locales) ; `docs/api-external.md`.
- [ ] Tests HTTP des AC-1 à AC-4 ; extension de `s34_4_ac8_…` aux paires ; mesure (AC-3).
- [ ] Gate complet (base remise à zéro avant ; exception `kesh-db`, gate complet aussi en cours de boucle),
      `scripts/gate-ci.sh` au dernier commit de code ; frontend non touché (gate frontend au push) ; E2E complet au
      dernier commit de code ; recette de montée (`scripts/recette-montee.sh`) avec la table neuve.

## Note de version

*(Texte prêt à assembler au `CHANGELOG.md` à la release.)*

- **Ajouté** : une transaction d'un prestataire de paiement se rattache à plusieurs écritures (vente au brut et frais
  passés à part), Kesh suggère la paire, et l'annulation défait le groupe entier ; aucune écriture n'est créée ni
  doublée. Migration non breaking, mais le retour à la version précédente passe par la sauvegarde d'avant la mise à
  jour.

**Compteurs** (audit des migrations — total aux deux sites et partition —, schéma de test, `LIB_ROUTES`, sites
`matched_entry_id`, `data_count`, `Cargo.lock`) : recomptés depuis la source après chaque reprise de `main`.

## Notes de développement

- **L'invariant le plus cher** reste celui de 38-6c1 : **une seule règle** (C-v017-3-7). Le groupe et la paire se
  posent **dans les textes partagés** (`entry_movements`, `linkable_side`, `other_side_predicate`, le fragment de
  l'AC-1), jamais dans une copie ; `pairs_for` est la seule logique Rust neuve, pure et testée sans base.
- **Ordre des verrous** du groupe : verrou nommé → `t` `FOR UPDATE` → `eᵢ` `FOR UPDATE` croissants → première lecture
  ordinaire → requête restreinte → `UPDATE` puis `INSERT`. L'annulation (AC-4) : `t`, puis les écritures croissantes,
  le lien retiré **avant** les contre-passations (sinon la garde `MatchedBankTransaction` les refuse).
- **Pourquoi `matched_entry_id` garde la principale** : ses lecteurs actuels (huit chemins, vues, sauvegarde) restent
  justes sans être touchés ; seuls ceux qui demandent « l'écriture est-elle rapprochée ? » lisent le fragment
  (inventaire clos, AC-1).
- **Réutilisations** : tout ce que pose 38-6c1 (`MovementBasis`, `entry_matches_for_page`, lot, diagnostic,
  `expected_version`) ; `journal_entries::reverse_in_tx` ; `reconciliation_cancel::classify`.
- **Hors périmètre** : groupes de trois écritures et plus **suggérés** (formés à la main seulement, AC-3) ; écriture
  de frais mensuelle groupant plusieurs ventes (Écart 1) ; annulation d'un groupe dans un exercice clos (AC-4,
  issue C-38-13) ; écran et manuel (38-9b).
- **Taille** (règle de splitting) : modules touchés — (1) `kesh-db` dépôts de réconciliation (`reconciliation_link`,
  `reconciliation_cancel`) ; (2) `kesh-db` migration, squash et sauvegarde ; (3) `kesh-db` lecteurs de l'inventaire
  (`journal_entries`, `bank_accounts`) ; (4) `kesh-api` (routes de réconciliation, exports et sauvegarde) ; (5)
  `kesh-i18n` et `docs/api-external.md`. Cinq au plus.

## Change Log

- 2026-11-15 — découpage de 38-6c (D-P2-3, Opus) : fiche née de `38-6c-rattachement-en-lot-et-api.md` (désormais
  `split`, contenu historique conservé) ; AC-12 à AC-15 renumérotés AC-1 à AC-4 ; la migration de l'ancien AC-9
  (`bank_transaction_extra_entries`, inscriptions de sauvegarde, P5, P6, P7, squash) jointe à l'AC-1 ; reçus de
  38-6c1 : `journalEntryIds` du lot et `linked[].journalEntryIds`, genre `uniquePair`, motif `several_pairs`, champs
  `entries`/`pairs`, troisième lecture par page, clause « groupe » du message, groupe dans un exercice clos, sections
  « groupe » de `docs/api-external.md`. `FR-38-6c-8..10` → `FR-38-6c2-1..3`, `NFR-38-6c-5..6` → `NFR-38-6c2-1..2`,
  **`NFR-38-6c2-3` ajoutée** (une seule règle, *Pattern batch*, temps borné, paires comprises — portées en 38-6c par
  `NFR-38-6c-1..3`, restées en 38-6c1) ; recompté : 3 FR, 3 NFR. Points inter-fiches P2 appliqués : relevé (f) de
  l'AC-1 — poses de `matched_entry_id` de 38-6b (paire, paire à moitié CHF, virement) « par construction » ;
  **un seul nom** pour les contre-passations d'un groupe, `reversalJournalEntryIds` (trace et
  `CancelReconciliationResponse`, `null` hors groupe ; `groupReversalJournalEntryIds` de 38-9b écarté — à aligner en
  38-9b). Dépendances : après 38-6c1, avant 38-6a.
- 2026-11-15 — validation P3 (Sonnet) : fiche neuve relue en entier, citations vérifiées au code et renvois à
  38-6c1 (AC-1, AC-3 à AC-5, AC-8) contrôlés ; 1 MEDIUM, 2 LOW ; remédiés dans la fiche : teardowns de test
  (`delete_all_by_company`, `drop_company_deep`) à vider avant les écritures (MEDIUM), `AlreadyBooked` cité
  `:140-145` et non `:118-124` (LOW), `matchedEntryId` = principale seule documenté (LOW) ; chemin de
  `bank_accounts.rs` (dépôt) précisé ; aucun finding inter-fiches.
- 2026-11-15 — validation P4 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 2 LOW ; remédiés. Remédiation P3 relue au
  code : `delete_all_by_company` (`journal_entries.rs:2078`, DELETE `:2085`, `pub`, appelants en `mod tests` seuls),
  `drop_company_deep` (`accounts.rs:1462-1479`, `panic!` sur échec), `test_fixtures::truncate_all`
  (`test_fixtures.rs:395`) sur `TABLES_TO_TRUNCATE` (`backup.rs:46`), `AlreadyBooked` (`routes/reconciliation.rs:140`,
  doc `:128-137`), `TransactionResponse` (`routes/bank_imports.rs:304`) : exacts ; les autres suppressions d'écritures
  de test (`invoices.rs:4460`, `journal_entries.rs:3784`) visent des écritures par identifiant, hors groupe. P4-1 (LOW)
  le préfixe « chemins sous `kesh-db/src/repositories/` » rendait (g) faux → `crates/kesh-api/src/exports/csv_tables.rs`.
  P4-2 (LOW, hors remédiation) l'inventaire disait 38-6a et 38-6b « développées avant » 38-6c2, contre D-P2-3 → corrigé.
  Aucun finding inter-fiches.
- 2026-11-16 — alignement sur les règles de travail à plusieurs (Sonnet) : en tête, renvoi à revoir après la fusion de
  l'Epic 36 (#823, 36-6) ; section *Note de version* ajoutée ; compteurs « recomptés après chaque reprise de `main` ».
  Aucun PDF ni CHANGELOG commité dans cette fiche.
