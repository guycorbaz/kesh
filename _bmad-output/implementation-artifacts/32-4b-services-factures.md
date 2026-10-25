# Story 32.4b : la couche de services — rollout des lectures : factures clientes, échéancier, règlements, factures fournisseur

Status: ready-for-dev

*Fiche rédigée le 2026-10-25, en autonomie (Opus 5.5), worktree `kesh-32-4b`, branche `story/32-4b` partie de `main`
`592ee430` (la 32-4a fusionnée). Registre : `epic-32-choix-autonomes-324b.md` (C-32-4b-1 à C-32-4b-18). Clé courte :
`324b`. Issue de l'epic : #723 (CR « Serveur MCP intégré à Kesh »), en `refs`. Les numéros de ligne cités sont ceux de
`592ee430`. Modèle direct : la 32-4a (`32-4a-services-ecritures-contacts.md`), dont les leçons de revue sont intégrées
dès cette fiche (§ *Les leçons de la 32-4a*). Remédiée après les validations P1 (Sonnet ×2) et P2 (Opus ×2) : Change Log.*

## Story

En tant que Guy, qui branchera un agent sur ses livres réels par le serveur MCP de la 0.16.0, je veux que la liste et
la fiche des factures clientes, l'échéancier, les règlements d'une facture, et la liste et la fiche des factures
fournisseur se lisent **par les mêmes services** pour l'API et pour le MCP — contrôle d'accès, validation de la demande,
bornes de pagination, défauts de tri et filtre de société compris —, afin que l'outil MCP de la 32-9a ne puisse ni
contourner une règle écrite dans un handler, ni lire une page sans plafond, ni recevoir un tri ou un filtre que l'écran
ne reçoit pas.

## Origine

- `planning-artifacts/epic-32-serveur-mcp.md` § *32-4* (l. 304-321) : « **32-4b — factures** : `routes/invoices`
  (liste, détail, échéances, règlements), `routes/supplier_invoices` (liste, détail), balance âgée (`reports`, déjà
  passé par la 32-3 : appel seulement) — **3** avec `services` ». **AC de l'epic** : tests d'API inchangés et verts ;
  chaque service porte son contrôle d'accès ; aucun handler de lecture des modules visés n'appelle plus un dépôt
  directement (inventaire clos, garde lexicale si elle est simple). **Risque nommé** : une liste paginée dont les bornes
  (`clamp(1, 200)`) se perdent — **c'est ici** que ce plafond existe (fait 3).
- L'annotation qui suit l'AC (l. 318-321) : la 32-4a a laissé deux lectures de ses fichiers hors services, faute
  d'outil ; Guy a confirmé cette lecture le 2026-10-24 (fiche 32-4a, Change Log, « Q1 confirmée par Guy »), pour
  ces deux lectures-là. La 32-4b l'applique à l'export CSV (C-32-4b-1), écart qu'elle soumet à Guy (Q2) et que l'epic
  porte en annotation datée (validation P1, F3 = R12).
- `planning-artifacts/epic-32-spike-mcp.md` l. 925-926 : « **32-4a / 32-4b** — AC ajouté : aucun service de lecture
  n'appelle un `get_or_create_*` ni un dépôt qui écrit ; inventaire clos des appels » (AC14 ici) ; l. 961 : les bornes
  des outils de la 32-9a sont « choisies pour rester sous 10 000 jetons par réponse » — elles le seront **à l'outil**,
  sous les plafonds du service (C-32-4b-6).
- **Le consommateur** : 32-9a (l. 425-427) — « factures clientes (liste, détail, échéances, impayées), balance âgée,
  factures fournisseur (liste, détail) », portée `invoices:read` (C-32-0-7 : « factures clientes et fournisseur,
  échéances, balance âgée »). « Impayées » est l'échéancier à son défaut (`paymentStatus` absent → `Unpaid`, fait 4).
- **Q11 (Guy, 2026-10-24 au soir, `epic-32-spike-mcp.md` l. 1061-1067)** : la lecture seule est **tenue par
  l'application** — les outils n'atteignent que des services qui reçoivent une `Lecture` —, la connexion du MCP se
  déclarant en outre `READ ONLY` (mode de session) ; **aucun second utilisateur MariaDB** ; « les mots par
  construction disparaissent ». La fiche n'en préjuge rien d'autre ; elle corrige les six sites que cette
  réponse contredit ou rend périmés, dans `services/`, `lib.rs` et `tests/common/` (AC17 (3), C-32-4b-12, C-32-4b-16).
- **Le motif** : `services/mod.rs` (l. 1-139, règles R1 à R8) ; la 32-4a l'a appliqué à cinq lectures
  (`services/{ecritures,postes_ouverts,carnet}.rs`). Une lecture **sans trace** rend directement (R5) : aucune des six
  lectures de cette story n'écrit au journal d'audit (fait 7).

## Les leçons de la 32-4a, intégrées dès la fiche

Ses trois passes de revue de code ont toutes porté sur la **couverture**, non sur le code déplacé, jugé fidèle (fiche
32-4a, Change Log, revues P1 à P3) :

1. **Chaque champ recopié** de la demande vers la requête du dépôt — filtres, `sort_by`, `sort_direction`, `limit`,
   `offset` — a **une ligne de test par la route** qui le distingue **et une mutation nommée**, au service **et** au
   handler (P1 B1 = E1 = E2 ; P2-1 : `sort_by`, `sort_dir` et `offset` « avaient leur ligne » sur le papier, aucune ne
   les exerçait). Ici : AC11 (d) et (e), mutations M15 à M19, toutes **jouées** (C-32-4b-10).
2. **Tout tri par défaut** se teste avec au moins **deux lignes distinguables**, en ordre rendu, au service **et** par
   la route (P2-1, C-32-4a-24) : AC11 (f), M20.
3. **Toute lecture « qui ne lit pas la société »** se fige par un test (P1 A3 = E3) : ici `factures::detail` et
   `factures::echeances` (fait 5), AC12 (d), M7.

## Ce que le code dit (relevé au code le 2026-10-25, `592ee430`)

1. **Les deux fichiers visés portent 20 handlers, dont 7 lectures** (`grep -n '^pub async fn'`) :
   `routes/invoices.rs` **13** — lectures `list_invoices` l. 597, `get_invoice` l. 679, `list_due_dates_handler`
   l. 1141, `list_invoice_settlements_handler` l. 1427, `export_due_dates_csv_handler` l. 1552 ; mutants
   `create_invoice` l. 708, `update_invoice` l. 793, `delete_invoice` l. 854, `validate_invoice_handler` l. 872,
   `unvalidate_invoice_handler` l. 919, `settle_invoice_handler` l. 1274, `write_off_invoice_handler` l. 1343,
   `cancel_invoice_settlement_handler` l. 1491 — ; `routes/supplier_invoices.rs` **7** — lectures
   `list_supplier_invoices` l. 551, `get_supplier_invoice` l. 569 ; mutants `create_supplier_invoice` l. 586,
   `pay_supplier_invoice` l. 611, `cancel_supplier_invoice` l. 644, `cancel_supplier_invoice_settlement` l. 676 ; et
   `scan_qr_supplier_invoice` l. 782, `POST`, qui **ne lit pas la base** (pure analyse du texte SPC, l. 783-788, aucun
   `State`). `async fn` toutes visibilités (motif `^\s*(pub(\([^)]*\))?\s+)?async\s+fn\b` sur les lignes de code
   tronquées au `#[cfg(test)]`, l. 1696 et 790) : **15** et **8** — `invoices.rs` y ajoute deux aides non `pub`,
   `with_draft_rounding_preview` (l. 299) et `ensure_contact_belongs_to_company` (l. 552) ; `supplier_invoices.rs`, la
   méthode indentée `SupplierInvoiceResponse::load_with_settlement_cancellation` (l. 204).
2. **RBAC** (`lib.rs`) : les six lectures migrées sont dans `authenticated_routes` (tous rôles) — `GET /api/v1/invoices`
   l. 897, `/invoices/due-dates` l. 902-905, `/invoices/{id}` l. 906, `/invoices/{id}/settlements` l. 914-917,
   `/supplier-invoices` l. 937-940, `/supplier-invoices/{id}` l. 941-944. L'export CSV de l'échéancier est dans
   `comptable_routes` (l. 608-611, « restreint au rôle Comptable+ », l. 899-901). Exigence API des six services :
   `Consultation`.
3. **Pagination — trois régimes, et deux d'entre eux REFUSENT au lieu d'écrêter** :
   - liste des factures (`list_invoices`, l. 605-616) : `limit` absent → `DEFAULT_LIST_LIMIT` 20 (l. 55) ; hors de
     `1..=MAX_LIST_LIMIT` (100, l. 54) → **400** « `limit doit être compris entre 1 et 100` » ; `offset` absent → 0,
     négatif → **400** « `offset doit être positif ou nul` » ;
   - échéancier (`build_due_dates_query`, l. 1044-1066) : les mêmes deux refus, plus `offset > 1_000_000`
     (`MAX_OFFSET`, l. 1058) → **400** « `offset doit être ≤ 1000000` » ;
   - factures fournisseur (`list_supplier_invoices`, l. 557-558) : `unwrap_or(50).clamp(1, 200)` et
     `unwrap_or(0).max(0)` — **écrêtage**, le `clamp(1, 200)` que l'epic nomme.
   **Aucun des trois dépôts n'écrête** : `invoices::list_by_company_paginated` lie `query.limit` et `query.offset`
   tels quels (`kesh-db/src/repositories/invoices.rs:899-901`), comme `supplier_invoices::list`
   (`supplier_invoices.rs:228-232`). La borne du handler est donc la **seule**. **Aucun test d'API** ne fige ces refus
   ni ces bornes : `grep -rnE 'limit doit être|offset doit|status invalide|search doit faire|dueBefore doit|dateFrom doit être antérieur' crates/kesh-api/tests`
   → rien.
4. **Validation, dans l'ordre** — douze refus `AppError::Validation` aux messages écrits en dur :
   - liste des factures (`list_invoices`) : **après** la lecture de la société (`get_company_for`, l. 603) — `limit`
     (l. 606-610), `offset` (l. 611-616), `search` rogné, vide → absent, plus de 100 caractères (`MAX_SEARCH_LEN`, l. 56)
     → « `search doit faire au plus 100 caractères` » (l. 619-632), `status` hors de `draft|validated|cancelled` →
     « `status invalide` » (l. 634-638), `dateFrom > dateTo` → « `dateFrom doit être antérieur ou égal à dateTo` »
     (l. 640-646) ; **cinq** refus ; défauts du tri : `sort_by` `unwrap_or_default()` (`InvoiceSortBy::Date`,
     `kesh-db/src/repositories/invoices.rs:124-131`), `sort_direction` `unwrap_or(SortDirection::Desc)` (l. 659-660) ;
     `payment_status` et `due_before` à `None`, `paused` recopié par `PausedFilter::from` (l. 651-658) ;
   - échéancier (`build_due_dates_query`, **aucune** lecture avant) : `limit`, `offset`, plafond d'`offset`, `search`,
     `dateFrom > dateTo` (l. 1080-1086), `dueBefore < dateFrom` → « `dueBefore doit être ≥ dateFrom` » (l. 1090-1096),
     `dueBefore < dateTo` → « `dueBefore doit être ≥ dateTo` » (l. 1100-1104) ; **sept** refus ; puis la requête :
     `status` **forcé** à `Some("validated")` (l. 1109), `payment_status` défaut **`Unpaid`** (l. 1116-1122), `paused`
     **forcé** à `None` (« INVARIANT ANTI-DISSIMULATION », l. 1124-1132), tri défaut `DueDate` / `Asc` (l. 1133-1134).
   Les dates (`dateFrom`, `dateTo`, `dueBefore`) sont des `Option<NaiveDate>` du DTO (l. 130-132, 1002-1006) : une date
   mal formée est un refus de l'extracteur `Query` d'Axum, **avant** le handler. `ListInvoicesQuery.paused` et
   `ListDueDatesQuery.payment_status` sont des enums de route (`PausedParam` l. 978-982, `PaymentStatusParam`
   l. 953-958), convertis vers ceux du dépôt par `From` (l. 960-969, 984-992).
5. **Les lectures et la société** :
   - `list_invoices` lit la société **d'abord** (l. 603), puis valide, puis lit la page sur `current_user.company_id`
     (l. 665-666) ;
   - `get_invoice` ne lit **aucune** société : `find_by_id_with_lines(&state.pool, current_user.company_id, id)` → 404
     (l. 684-687), `amount_settled` puis `amount_due` (l. 693-698), puis `with_draft_rounding_preview(&state, …)`
     (l. 699-704), qui lit le réglage `round_to_5_centimes` **pour un brouillon seulement** (l. 299-313 ; « Lecture
     pure (revue de code P1, lentille B) : un `GET` ne doit rien écrire », l. 306) ;
   - `list_due_dates_handler` ne lit **aucune** société : la page et le résumé en parallèle (`tokio::join!`,
     l. 1155-1159), le résumé sur la requête **sans** `payment_status` (l. 1148-1154, « B15 ») ;
   - `list_invoice_settlements_handler` : société (l. 1432), facture → 404 (l. 1434-1436), puis **une** transaction
     (`state` / `.pool` / `.begin()` coupé par rustfmt, l. 1440-1444) portant `list_for_invoice(&mut *tx, …)` et, pour
     chaque règlement, `settlement_cancel_blocker(&mut tx, …)` (l. 1445-1452), refermée par **`commit`** (l. 1467-1469 ;
     « ⛔ Une transaction de lecture, pour un seul instantané », l. 1437-1439). **Aucune garde** ne fige cette forme
     (`grep -rn list_invoice_settlements_handler crates` → `lib.rs:916` seul) ;
   - `list_supplier_invoices` et `get_supplier_invoice` lisent la société d'abord (l. 556, 574) ; le détail passe par
     `SupplierInvoiceResponse::load_with_settlement_cancellation(&state.pool, company.id, id)` (l. 575-577), méthode
     `pub` qui sert aussi trois mutants du fichier (l. 631, 655, 687-691) et trois sites de
     `routes/imported_supplier_invoices.rs` (l. 140, 555, 660) : `supplier_invoices::get_settlement_view` → 404, puis
     une mise en forme pure (l. 204-226).
6. **Les deux aides hors corps** de `invoices.rs` lisent `state.pool` et servent des mutants : `with_draft_rounding_preview`
   (l. 308 ; appelée par `get_invoice` et trois mutants, l. 700, 784, 846, 941) et `ensure_contact_belongs_to_company`
   (l. 557, par `create_invoice` et `update_invoice`). Relevé du détecteur de l'AC12 de la 32-4a (`\bstate\s*\.\s*pool\b`
   sur le code aplati, commentaires ôtés) : **40** occurrences dans `invoices.rs` — 38 dans les corps des 13 handlers,
   **2** dans ces aides —, **15** dans `supplier_invoices.rs`, toutes dans des corps ; aucune chaîne n'y porte `//` ni
   `/*`. ⚠️ **Le trou que la 32-4b rencontre** : `get_invoice` passe `&state` — non `state.pool` — à une aide qui lit
   le pool ; le détecteur (ii) de la 32-4a ne le voit pas (sa limite « une aide appelée par `list_accounts` »,
   `services_motif.rs:2236-2237`), et le compte au niveau du fichier de (iii) suppose qu'**aucune** occurrence n'est
   hors d'un corps — faux pour `invoices.rs`. D'où la garde (ix) et l'inventaire des aides de l'AC13.
7. **Aucune des six lectures n'écrit, ni ne verrouille — relevé au premier niveau, à fermer par l'AC14** (chemins
   `kesh-db/src/repositories/`) :
   - appels directs : `invoices::{list_by_company_paginated, find_by_id_with_lines, due_dates_summary}` (`SELECT`
     seuls, l. 829-1024), `invoice_settlements::{amount_settled, amount_due, list_for_invoice}` (`SELECT`, l. 485-563),
     `invoice_settlements_write::settlement_cancel_blocker` (l. 740 → `settlement_cancel_blocker_unlinking`, l. 752, un
     `SELECT` l. 760-773, puis `settlement_cancellation::settlement_entry_cancel_blocker`, l. 87, dont le doc-comment dit
     « **Lecture seule** : aucun verrou n'est posé ici », l. 68), `company_invoice_settings::round_to_5_centimes`
     (`SELECT … unwrap_or(true)`, l. 529-541 — **pas** de `get_or_create`), `supplier_invoices::list` (l. 209-238),
     `supplier_invoices::get_settlement_view` (l. 1771-1807 : `pool.begin()`, `FIND_SCOPED_SQL`, `fetch_lines`,
     `supplier_settlement_cancel_blocker`, `supplier_invoice_cancel_blocker`,
     `payment_batches::last_confirmed_batch_for_invoice`, `imported_supplier_invoices::find_by_supplier_invoice_id_scoped`,
     puis `commit` ; « ⚠️ Aucune de ces lectures ne doit devenir verrouillante », l. 1768-1769), `helpers::company_for_ids`
     → `companies::find_by_id` ;
   - les `FOR UPDATE` de ces fichiers (relevé `awk` par fonction englobante) sont **tous** dans des fonctions d'écriture
     hors de ces chemins — `create_in_tx`, `pay_in_tx_as`, `cancel_in_tx`, `cancel_settlement_in_tx`,
     `lock_for_bank_settlement_in_tx`, `guard_not_in_generated_batch` (`supplier_invoices.rs`) ; `settle_invoice`,
     `write_off_invoice`, `cancel_settlement_in_tx` (`invoice_settlements_write.rs`) ; `find_scoped_for_update_in_tx`,
     `update`, `delete`, `unvalidate`, `validate_invoice`, `set_dunning_pause` (`invoices.rs`) ;
     `get_or_create_default_in_tx`, `update`, `designated_account_id`… (`company_invoice_settings.rs`) ; aucun dans
     `settlement_cancellation.rs`.
   Ce relevé n'est **pas** la fermeture : les appels transitifs (`letterings::document_group_frozen_by_periods`,
   `journal_entries::reversal_blockers`, `fiscal_years::has_open_covering_date`, ce qu'appellent
   `supplier_invoice_cancel_blocker` l. 1224 et `supplier_settlement_cancel_blocker_unlinking` l. 1498…) se suivent à
   l'AC14, au patron du fait 6 de la 32-4a. **Établi par lecture statique** ; l'exécution (AC12 (a)) le confirme sur les
   branches que les fixtures exercent, la `Lecture` de test refusant écriture **et** `FOR UPDATE` (1792, mesuré par
   `ac11_c_un_verrou_est_refuse_sur_la_lecture_de_test`, `services_rollout.rs:347-365`).
8. **La balance âgée est déjà un service** : `get_aged_receivables` (`routes/reports.rs:435-442`) appelle
   `rapports::balance_agee(&acces, &state.lecture(), Affichage)` (`services/rapports.rs:661-670`, exigence
   `EXIGENCES_BALANCE_AGEE` : affichage `Consultation` / `InvoicesRead`, export `Comptable` / `None`, l. 166-178) ; sa
   ligne de trace est dans `TABLE_DES_TRACES` de `services_motif.rs`. **Rien à déplacer**, `routes/reports.rs` n'est pas
   touché : « appel seulement » se lit « l'outil de la 32-9a l'appellera tel quel ».
9. **Le motif tel qu'il est après la 32-4a** : `Portee { BooksRead, InvoicesRead, ContactsRead }` (`services/acces.rs:22-31`) ;
   huit fichiers dans `services/` (`services_motif.rs:2363-2368`) ; `HANDLERS_MIGRES` à six (l. 2124-2131) ;
   `SITES_NON_RESOLUS` à trois fichiers (l. 2138-2168) ; `SERVICES_DU_ROLLOUT` à trois (l. 2292-2296) ;
   `corps_des_handlers` n'ouvre que les lignes qui **commencent** par `pub async fn` (l. 2026-2050 — une méthode
   indentée n'est pas un handler) ; `services_rollout.rs` porte la matrice `Svc` / `SERVICES` à cinq (l. 373-388) et
   `jouer` (l. 409-452).
10. **Documentation** : `docs/api-external.md` ne nomme que `GET /invoices`, `GET /invoices/{id}` (l. 215) et la fiche
    fournisseur (l. 552-566, 723-757 : champs d'annulation et de document), et les règlements d'une facture (l. 497 :
    « **`GET /api/v1/invoices/{id}/settlements`** — lecture », `cancellable`, `cancelBlockedBy`…) — **aucune** borne,
    aucun refus de liste, ni l'échéancier. *(Validation P1, R8 : la l. 497 manquait.)* `docs/MULTI-TENANT-SCOPING-PATTERNS.md:273-281` nomme les six lectures de la
    32-3 et de la 32-4a. `services/mod.rs:132-139` et `services/poignees.rs:9-12` disent la lecture seule « décidée par
    Guy (Q11) » à venir et « la garantie réelle […] celle de la base, en 32-6 ». Depuis la réponse Q11 (origine),
    `mod.rs:139` et `mod.rs:52-53` (« la 32-6 l'imposera par la base ») la **contredisent** ; `poignees.rs:9-12`,
    `lib.rs:147-149` et `:158-160` (doc-comments d'`AppState::lecture()` et `trace()`) et `tests/common/lecture_seule.rs:8-9`
    sont **périmés** (ils disent la décision à prendre ; et la l. 8, « Ce n'est **pas** la lecture seule de production »,
    devient inexacte : la connexion du MCP emploiera le même mode de session — validation P2, R2 = F6). Relevé de la valeur sur `crates/` entier (validation P1, R4) :
    `grep -rnE 'limité au .SELECT.|utilisateur .SELECT.|l.imposera par|sera celle de la base|est décidée par Guy|rien ici n.en préjuge|que décide la 32-6|n.est \*\*pas\*\* la lecture seule de production' crates/`
    → ces six sites (dix lignes : `lib.rs` 148, 159 ; `poignees.rs` 10, 11 ; `mod.rs` 52, 135, 136, 139 ;
    `lecture_seule.rs` 8, 9), rien d'autre. *(Motif resserré à la validation P2, R3 : il vise les formules périmées, non
    une citation juste de la décision ; il gagne la l. 8 de `lecture_seule.rs`, R2 = F6.)*

## Le rollout (ce que la story déplace)

| Handler (l.) | Route | Service | Exigence API / MCP | Demande | Rend |
|---|---|---|---|---|---|
| `list_invoices` (597) | `GET /invoices` | `factures::lister` | Consultation / InvoicesRead | `DemandeListeFactures` | `InvoiceListResult` |
| `get_invoice` (679) | `GET /invoices/{id}` | `factures::detail` | Consultation / InvoicesRead | `id: i64` | `FicheFacture` |
| `list_due_dates_handler` (1141) | `GET /invoices/due-dates` | `factures::echeances` | Consultation / InvoicesRead | `DemandeEcheances` | `Echeancier { page, resume }` |
| `list_invoice_settlements_handler` (1427) | `GET /invoices/{id}/settlements` | `factures::reglements` | Consultation / InvoicesRead | `id: i64` | `Vec<ReglementFacture>` |
| `list_supplier_invoices` (551) | `GET /supplier-invoices` | `factures_fournisseur::lister` | Consultation / InvoicesRead | `DemandeListeFacturesFournisseur { limit, offset }` | `PageFacturesFournisseur` |
| `get_supplier_invoice` (569) | `GET /supplier-invoices/{id}` | `factures_fournisseur::detail` | Consultation / InvoicesRead | `id: i64` | `SupplierInvoiceSettlementView` |

**Restent dans leur handler**, figés comme tels (AC13 (iii)) : `export_due_dates_csv_handler` (CSV, Comptable+, aucun
outil — C-32-4b-1 ; il emploie la validation **du service**, C-32-4b-4), `scan_qr_supplier_invoice` (ne lit pas la
base), et les **douze** mutants (C-32-0-4).

## Inventaire clos des lectures — visées, et laissées hors du périmètre

**Visées** (six) : le tableau ci-dessus. **Dans les fichiers visés, laissées** :

| Lecture | Raison |
|---|---|
| `export_due_dates_csv_handler` — `GET /invoices/due-dates/export.csv` | un export de fichier, Comptable+ (fait 2) : aucun outil de la 32-9a ne rend un CSV ; sa validation passe par la fonction pure du service, sans copie (C-32-4b-1, C-32-4b-4) |
| `scan_qr_supplier_invoice` — `POST /supplier-invoices/scan-qr` | aucune lecture de la base (fait 1) |
| `SupplierInvoiceResponse::load_with_settlement_cancellation` | méthode `pub`, non un handler : elle reste pour les trois mutants et les trois sites de `imported_supplier_invoices.rs` (fait 5), recomposée sur la mise en forme pure que le handler migré emploie (C-32-4b-5) |
| `with_draft_rounding_preview`, `ensure_contact_belongs_to_company` | aides des mutants (fait 6), inchangées ; `get_invoice` cesse d'appeler la première (AC4) |

**Hors des fichiers visés** (lectures voisines des factures, aucune nommée par la 32-9a — C-32-0-4) :

| Lecture | Module | Raison |
|---|---|---|
| `GET /reports/aged-receivables` (et son export) | `routes/reports.rs` | **déjà** un service (fait 8) — rien à faire |
| `GET /invoices/{id}/reminders` | `routes/dunning_reminders.rs` (`list_reminder_history`, `lib.rs:907-910`) | historique des rappels : aucun outil |
| `GET /dunning/…` (`list_reminders`, Comptable) | `routes/dunning_reminders.rs` (`lib.rs:640-643`) | aucun outil |
| `GET /invoices/{id}/pdf` | `routes/invoice_pdf.rs` (`lib.rs:918-921`) | un document : 32-9b (« documents »), métadonnées seulement en 0.16.0 |
| `GET /invoices/{id}/email-preview`, `…/reminder-preview` | `routes/invoice_email.rs` (Comptable) | aperçus de courriel : aucun outil |
| `GET /credit-notes`, `/credit-notes/{id}`, `/{id}/pdf` | `routes/credit_notes.rs` (`lib.rs:924-935`) | avoirs : aucun outil nommé |
| `GET /supplier-invoices/{id}/source-document` | `routes/imported_supplier_invoices.rs` (`lib.rs:948-951`) | une pièce : 32-9b, qui la migrera si elle la sert |
| `GET /imported-supplier-invoices*` | `routes/imported_supplier_invoices.rs` (Comptable) | documents à traiter : aucun outil |
| `GET /payment-batches*` | `routes/payment_batches.rs` | lots de paiement : hors couche de services (C-32-0-4) |
| `GET /company/invoice-settings` | `routes/company_invoice_settings.rs` (`lib.rs:966-969`) | **écrit** (`get_or_create_default`, spike l. 598) : ne peut pas devenir un service de lecture tel quel |

## Critères d'acceptation

### A — Les modules et leurs types

1. **AC1 — deux modules de services.** `crates/kesh-api/src/services/{factures,factures_fournisseur}.rs` neufs
   (C-32-4b-2), déclarés `pub mod` dans `services/mod.rs` (ordre alphabétique, les sept autres ne bougent pas), sous
   son `forbid(unsafe_code)`, qu'ils ne redéclarent ni ne déplacent (R8). Chaque élément public porte son doc-comment.
   **Aucun type d'Axum, `AppState` ni `CurrentUser`** dans leurs lignes de code (R1, garde (vi) : `RESIDUS_VI` **ne
   change pas**). **Aucune `Trace`** (fait 7) : leurs lignes de code ne contiennent ni `\bTrace\b` ni `\btrace\b`
   (garde (viii)). **Aucune portée neuve** : `InvoicesRead` existe (`acces.rs:26-27`, « Lecture des factures, dont la
   balance âgée des débiteurs ») ; son doc-comment gagne « clientes et fournisseur, échéancier, règlements »
   (C-32-4b-3). Les types des demandes sont ceux du dépôt pour les filtres (`InvoiceSortBy`, `PaymentStatusFilter`,
   `PausedFilter`, `kesh_db::repositories::invoices`) et de `kesh-core` pour le sens (`SortDirection`), non les enums
   de route (C-32-4b-7).
2. **AC2 — le contrôle d'accès en tête.** Chacun des six services commence par `acces.exiger(EXIGENCE_…)?` (R2),
   **avant** la validation et toute lecture ; une constante `pub const EXIGENCE_<OPÉRATION>: Exigence` par service, à
   côté de lui (R3), aux valeurs du tableau du *Rollout*. Le rôle API **égale** celui de la route (fait 2).

### B — Les services

3. **AC3 — `factures::lister`.** `(&ContexteAcces, &Lecture, DemandeListeFactures)`. Après `exiger` :
   `company_for_ids` **d'abord** (`Internal` si la société manque — l'ordre du fait 5, conservé) ; puis les **cinq**
   refus du fait 4, **dans cet ordre, messages identiques au caractère près**, `limit` absent → `LIMITE_DEFAUT` (20) et
   **refusé** hors de `1..=LIMITE_MAX` (100) — **jamais écrêté** (C-32-4b-6) —, `offset` absent → 0 et refusé s'il
   est négatif, `search` rogné, vide → absent, refusé au-delà de `RECHERCHE_MAX` (100) caractères ; puis
   `InvoiceListQuery` avec les défauts du fait 4 (`sort_by` défaut du dépôt, `sort_direction` défaut `Desc`,
   `payment_status`/`due_before` à `None`, `paused` recopié) ; puis
   `invoices::list_by_company_paginated(lecture.pool(), acces.company_id(), query)`. Les contrôles communs à la liste et
   à l'échéancier (`limit`, `offset`, `search`) s'écrivent **une fois**, dans des fonctions privées du module, appelées
   dans l'ordre de chacun (C-32-4b-4).
4. **AC4 — `factures::detail`.** `(&ContexteAcces, &Lecture, id)`. Après `exiger` : **aucune lecture de société**
   (fait 5 — comportement conservé, C-32-4b-8) ; `find_by_id_with_lines(lecture.pool(), acces.company_id(), id)` →
   `AppError::Database(DbError::NotFound)` ; `amount_settled` puis `amount_due` (`map_err(AppError::Database)`, comme
   l. 693-698) ; puis, **pour un brouillon seulement**, `company_invoice_settings::round_to_5_centimes(lecture.pool(),
   facture.company_id)`. Rend `FicheFacture { facture, lignes, montant_regle, reste_du, arrondi_5_centimes:
   Option<bool> }` (`None` : non brouillon, réglage non lu). Le handler compose `InvoiceResponse::from_parts(…)
   .with_settlement(…)` puis, si `Some(r)`, `.with_rounding_preview(r)` ; il **n'appelle plus**
   `with_draft_rounding_preview` (C-32-4b-9), qui reste aux trois mutants, inchangée.
5. **AC5 — `factures::echeances` et `factures::requete_echeances`.** `requete_echeances(DemandeEcheances) ->
   Result<InvoiceListQuery, AppError>` : **fonction pure** (aucun accès, aucune lecture), qui porte les **sept** refus
   du fait 4 dans leur ordre et les constantes de l'échéancier (`status` forcé `"validated"`, `payment_status` défaut
   `Unpaid`, `paused` forcé `None`, tri défaut `DueDate` / `Asc`) — le corps de `build_due_dates_query` (l. 1044-1137),
   déplacé, commentaires « B1 », « B11 », « B16 », « B21 » et l'invariant D10 compris — ⚠️ aucun texte neuf de `src/`
   ne nomme `build_due_dates_query`, pas même pour dire d'où vient le code (le contrôle de l'AC8 le veut absent de
   `crates/kesh-api/src` ; l'origine se dit « l'ancienne construction de l'échéancier du handler », ou se lit au
   Dev Agent Record ; validation P2, F3). `echeances(&ContexteAcces,
   &Lecture, DemandeEcheances) -> Result<Echeancier, AppError>` : `exiger`, puis `requete_echeances`, puis la requête
   du résumé **sans** `payment_status` (« B15 »), puis la page et le résumé en parallèle (`tokio::join!` sur
   `lecture.pool()`) ; **aucune lecture de société** (fait 5). L'export CSV appelle `requete_echeances` à la place de
   `build_due_dates_query`, **après** sa lecture de société et avec `limit`/`offset` à `None` (l. 1561-1570) — son ordre
   ne change pas ; son commentaire « M4 » (l. 1562-1566), qui nomme `build_due_dates_query`, nomme `requete_echeances`.
   La conversion `ListDueDatesQuery → DemandeEcheances` est **une** fonction du handler, partagée par la liste et
   l'export (C-32-4b-4). `build_due_dates_query` disparaît.
6. **AC6 — `factures::reglements`.** `(&ContexteAcces, &Lecture, id)`. Après `exiger` : `company_for_ids` (`Internal`),
   `find_by_id_with_lines(lecture.pool(), company.id, id)` → `NotFound`, puis **une** transaction
   `lecture.pool().begin()` portant `list_for_invoice(&mut *tx, …)` et, par règlement, `settlement_cancel_blocker(&mut
   tx, …)`, refermée par **`commit`** (comme l. 1467-1469 ; C-32-4b-11). Rend `Vec<ReglementFacture { reglement,
   motif: Option<SettlementCancelHit> }>` dans l'ordre du dépôt ; la mise en forme `InvoiceSettlementResponse` reste au
   handler. Une **garde lexicale neuve** au `#[cfg(test)] mod tests` de `services/factures.rs`,
   `reglements_lisent_dans_une_transaction` (au patron de `get_journal_entry_reads_in_one_transaction`,
   `services/ecritures.rs`) : sur le corps de `pub async fn reglements(` lu par `include_str!("factures.rs")` tronqué à
   `#[cfg(test)]`, débarrassé de ses blancs — `pool().begin()` compté **une** fois, `acquire()` **zéro** fois,
   `list_for_invoice(&mut*tx,` et `settlement_cancel_blocker(&mutt` (sans blancs : `&mut tx` → `&muttx`) chacun **une**
   fois, et aucun des deux sur `.pool()` (C-32-4b-11).
7. **AC7 — `factures_fournisseur::lister` et `::detail`.** `lister(&ContexteAcces, &Lecture,
   DemandeListeFacturesFournisseur)` : `exiger`, `company_for_ids` (`Internal`), puis `limit` défaut `LIMITE_DEFAUT`
   (50) et `clamp(1, LIMITE_MAX)` (200), `offset` défaut 0 et `max(0)`, puis `supplier_invoices::list(lecture.pool(),
   company.id, limit, offset)` ; rend `PageFacturesFournisseur { items, total, limit, offset }` (les valeurs
   **écrêtées**, celles que la route rend aujourd'hui, l. 560-565). `detail(&ContexteAcces, &Lecture, id)` : `exiger`,
   `company_for_ids` (`Internal`), puis `get_settlement_view(lecture.pool(), company.id, id)` → `NotFound` ; rend la
   vue. Dans `routes/supplier_invoices.rs`, la mise en forme de `load_with_settlement_cancellation` (l. 212-225) devient
   `SupplierInvoiceResponse::from_view(vue)` (pure, `pub`), que la méthode garde pour ses appelants et que le handler
   migré emploie — **une** mise en forme, deux chemins (C-32-4b-5) ; la signature de la méthode ne change pas
   (`imported_supplier_invoices.rs` non touché).
8. **AC8 — handlers minces, comportement inchangé.** Chacun des six handlers ne fait plus que : extraire la requête,
   construire `ContexteAcces::depuis_api(&current_user)` et la demande, appeler son service avec `&state.lecture()`,
   mettre en forme la réponse. Ses **seuls** usages de `state` sont `State(state)` et `state.lecture()` (garde (ix)).
   Statuts, corps, en-têtes, ordre des erreurs : identiques. Les DTO restent au handler, inchangés (C-32-4b-7). Contrôles
   au dernier commit, sortie recopiée au Dev Agent Record :
   - `grep -nE '\b(MAX_LIST_LIMIT|DEFAULT_LIST_LIMIT|MAX_SEARCH_LEN)\b' crates/kesh-api/src/routes/invoices.rs` → rien ;
   - `grep -nE 'clamp\(|\.max\(0\)|unwrap_or\(50\)' crates/kesh-api/src/routes/supplier_invoices.rs` → rien ;
   - `sed -n '1,/^#\[cfg(test)\]/p' crates/kesh-api/src/routes/invoices.rs | grep -c 'AppError::Validation'` → **22**
     (34 aujourd'hui, moins les douze du fait 4) — un refus recopié au handler sous une autre forme se verrait ici ;
   - `grep -rnF 'build_due_dates_query' crates/kesh-api/src` → rien (aujourd'hui l. 1044, 1146, 1565, 1570 ; le commentaire
     de l'export compris, AC5 ; validation P1, F1 = R7) — `crates/kesh-api/tests/dunning_reminders_e2e.rs:717`, hors
     `src/`, reste aux *Points à tracer*.
   Les mutants, l'export (hors l'appel de `requete_echeances`, de la conversion partagée et de son commentaire « M4 »)
   et `scan_qr` ne changent pas.

### C — Les tests

9. **AC9 — les tests existants restent verts ; les seules modifications sont celles-ci** (liste fermée) :
   (1) `tests/services_motif.rs` : la garde étendue de l'AC13 ; le compte de `services/` à **dix** fichiers
   (l. 2363-2368 et son message) ; rien d'autre. (2) `tests/services_rollout.rs` **complété, non recréé** : des
   variantes ajoutées à `Svc` et à `SERVICES` (l. 373-388), des bras à `portee_exigee` (l. 390-395) et à `jouer`
   (l. 409-452), des cas ajoutés à `ac9_c_le_refus_precede_validation_et_lecture`, `ac11_a_…` et `ac11_b_aucune_lecture_d_une_autre_societe`,
   et des tests neufs ; **les bras, lignes et assertions des cinq services de la 32-4a restent identiques**, hors les longueurs de tableau
   (`[Svc; 5]` l. 381, la table de `ac9_c_…` typée `; 8]` l. 551) et le doc-comment de `Cas::Invalide` (l. 402), qui
   nomme aussi `limit = Some(0)` (validation P2, F4) ;
   `ac11_a_les_cinq_services_lisent_en_lecture_seule` est renommé `ac11_a_les_services_lisent_en_lecture_seule`
   (C-32-4b-13), son doc-comment et celui du module mis à jour, comme les commentaires qui disent « les cinq services »
   (l. 4, 368, 371, 481, 933 ; validation P1, R14) ; `Livres` et `poser_livres` gagnent les fixtures de factures
   (C-32-4b-15), les assertions des huit tests de la 32-4a qui appellent `poser_livres` (l. 486, 546, 639, 683, 863,
   941, 995, 1048) restant identiques et vraies — vérifiées une à une à l'étape 0 ; les tests neufs portent le préfixe
   `ac324b_` (validation P1, F6). (3) `tests/invoice_echeancier_e2e.rs` : les seules liaisons de l'AC18. (4)
   `tests/common/lecture_seule.rs` : son seul doc-comment (l. 8-9, AC17 (3)). (5) Aucun autre fichier de `tests/`.
   Contrôles : `git diff --name-only $(git merge-base HEAD origin/main) HEAD -- crates/kesh-api/tests` (sans filtre :
   un fichier ajouté compte aussi ; validation P1, R13) → **exactement** `crates/kesh-api/tests/common/lecture_seule.rs`,
   `crates/kesh-api/tests/invoice_echeancier_e2e.rs`, `crates/kesh-api/tests/services_motif.rs` et
   `crates/kesh-api/tests/services_rollout.rs` ; `diff` des
   `mod tests` de `routes/invoices.rs` et de `mod scan_qr_tests` de `routes/supplier_invoices.rs` (patron de l'AC8 de la
   32-4a : `sed -n '/^#\[cfg(test)\]/,$p'` aux deux bornes) → **vides**. Fichiers qui doivent rester verts, relevé
   **large** (`grep -lE '/api/v1/invoices|/api/v1/supplier-invoices' crates/kesh-api/tests/*.rs` → **25** fichiers au
   2026-10-25) : le développement relève, au site, ceux qui frappent l'une des **six** routes en `GET` (au moins
   `invoice_echeancier_e2e`, `invoice_ttc_e2e`, `dunning_reminders_e2e`, `idor_multi_tenant_e2e`,
   `supplier_settlement_cancel_e2e`, `bank_accounts_closed_e2e`, `inbox_import_e2e`) et recopie la liste et les comptes
   **réellement exécutés**. S'y ajoutent les huit gardes de sources de l'AC8 de la 32-4a (`audit_label_registry`,
   `textes_coherents`, `admin_pat_denied_e2e`, `audit_route_registry`, `kesh-db/tests/test_schema_guard.rs` — chaque
   `#[sqlx::test(` neuf tient sur une ligne et nomme `../kesh-db/test-schema` —, `configuration_transmise`,
   `bank_account_closed_inventory`, `kesh-db/tests/letterings_lexical.rs`).
10. **AC10 — un test de refus par service, et son contrôle positif** (matrice `Svc` étendue à **onze** services). Pour
    chacun des six neufs, assertion nommée par cas, en `matches!` (`AppError` n'est pas `PartialEq`) :
    - (a) MCP **sans** la portée (portées vides ; puis `BooksRead` seule, `ContactsRead` seule) → `Forbidden` ;
    - (b) MCP avec `InvoicesRead` → `Ok` ;
    - (c-val) le refus précède la validation — MCP sans portée, demande qui échouerait → `Forbidden` ; le même cas
      **avec** la portée rend l'erreur attendue (témoin) : `limit = Some(0)` pour `factures::lister` et
      `factures::echeances` (`Validation`) ; sans objet pour `detail`, `reglements`, `factures_fournisseur::*`
      (aucun refus d'analyse ; le fournisseur écrête) — dit au Dev Agent Record ;
    - (c-lec) le refus précède la lecture — `id` inexistant pour `factures::detail`, `factures::reglements`,
      `factures_fournisseur::detail` (`NotFound`) ; société inexistante pour `factures::lister`, `factures::reglements`,
      `factures_fournisseur::{lister, detail}` (`Internal`) ; sans objet pour `factures::echeances` (rien de nommé ni de
      lu avant la page) ;
    - (e) API `Consultation` → `Ok`.
11. **AC11 — la validation, les bornes, les filtres et le tri, au service et par la route.** Fixtures **synthétiques**
    (AC9 de la 32-4a : `seed_accounting_company`, exercice 2020-2030). **Deux montages** (C-32-4b-15, validation P1,
    F4) : pour la matrice (AC10, AC12), `poser_livres` pose aussi, **par les dépôts**, un brouillon, une facture validée
    partiellement réglée (`invoices::create`, `invoices::validate_invoice`, `invoice_settlements_write::settle_invoice`
    en `SettlementChoice::InternalAccount` sur 1000, `kesh-db/src/repositories/invoice_settlements_write.rs:67-75`) et
    une facture fournisseur (au patron de `supplier_settlement_cancel_e2e.rs:109-177`) ; pour l'AC11 (d) à (g), chaque
    test pose ses propres fixtures — **au moins quatre** factures, `created_at` posé par un `UPDATE` explicite et
    distinct pour chacune (le départage `i.id DESC` et `DATETIME(3)` rendraient `CreatedAt` ambigu ; validation P1, F5),
    dates d'échéance en 2021 (échue) et en 2030 (à échoir), indépendantes de l'horloge :
    - (a) **table des refus**, au service, message attendu **en entier** : les cinq de `lister` et les sept de
      `requete_echeances`, chacun seul ; les **bornes acceptées** — `limit` 1 et 100, `offset` 0, `offset` 1 000 000
      (échéancier), `search` de 100 caractères, **`search = " " + "a"×100 + " "`** (102 caractères bruts, 100 une fois
      rognés : accepté, même résultat que `"a"×100`) **pour la liste et pour l'échéancier** — **témoin de M12b**, le
      dépôt rognant lui-même la recherche (`kesh-db/src/repositories/invoices.rs:428`, `:986`) : seule la longueur
      comparée à `RECHERCHE_MAX` distingue le rognage du service (validation P2, R1 = F1) —, `dateFrom == dateTo`,
      `dueBefore == dateFrom` (sans `dateTo`), `dueBefore == dateTo` (sans `dateFrom`, ou avec `dateFrom ≤ dateTo` ;
      validation P2, R5), `search = "   "` (aucun filtre : même total que sans) — et leurs voisines refusées (`limit` 0 et 101, `offset`
      −1, et 1 000 001 pour l'échéancier seul — la liste n'a pas de plafond d'`offset` ; validation P1, R10 —, `search`
      de 101 caractères) ; des **cas d'ordre** : `limit = 0` **et** `offset = −1` → message de
      `limit` ; `offset = −1` et `status = "x"` → `offset` ; `search` de 101 caractères et `status = "x"` → `search` ;
      `status = "x"` et `dateFrom > dateTo` → `status` ; échéancier : `offset = 1 000 001` et `search` trop long →
      `offset` ; `dateFrom > dateTo` et `dueBefore < dateFrom` → `dateFrom doit être…` ; `dueBefore < dateFrom` et
      `dueBefore < dateTo` → `dueBefore doit être ≥ dateFrom` ;
    - (b) **table des bornes** de `factures_fournisseur::lister` (`table_des_bornes(50, 200)`, l'aide de la 32-4a) ; et
      les défauts des deux listes clientes : `(None, None)` → `limit` 20, `offset` 0 ;
    - (c) **par la route** (montage de `services_rollout.rs`, joué **avant** T2 : T1 (i)) : `GET /api/v1/invoices?limit=101`,
      `?limit=0`, `?offset=-1`, `GET /api/v1/invoices/due-dates?offset=1000001` → 400, code `VALIDATION_ERROR`, message
      de la table ; `GET /api/v1/invoices` → `limit` 20 ; `GET /api/v1/supplier-invoices?limit=1000&offset=-3` → 200,
      `limit` 200, `offset` 0 ; `GET /api/v1/supplier-invoices` → `limit` 50 ; `GET /api/v1/invoices?status=cancelled`
      → 200 ;
    - (d) **chaque filtre de la liste, par la route** (une ligne par filtre, seul, sur des fixtures qu'il sépare) :
      `search` (nom du contact, numéro de facture), `status` (`draft`, `validated`), `contactId`, `dateFrom`, `dateTo`
      (bornes incluses), `paused` (`paused`, `not-paused`, `all`), `limit=1&offset=1` (offset positif) ; **chaque
      filtre de l'échéancier**, de même : `search`, `contactId`, `dateFrom`, `dateTo`, `dueBefore`, `paymentStatus`
      (`all`, `paid`, `unpaid`, `overdue`), `limit=1&offset=1` ; les **constantes** de l'échéancier : un brouillon n'y
      paraît jamais, **y compris avec `paymentStatus=all`** (`status` forcé : sans `all`, le dépôt l'exclut lui-même ;
      validation P1, R6), une facture suspendue **et** une non suspendue y paraissent toutes deux (`paused` forcé ;
      R5), une facture payée n'y paraît pas sans `paymentStatus` (défaut `Unpaid`) ; le **résumé** est le même pour
      `paymentStatus=paid` et sans paramètre — contrat de la route, qui ne distingue **pas** le retrait « B15 » : le
      dépôt ignore lui-même `payment_status` (M21d, équivalente) ; la liste fournisseur : `limit=1&offset=1` ;
    - (e) **chaque colonne et chaque sens du tri, par la route, en ordre rendu** : liste — `sortBy` = `Date`, `DueDate`,
      `TotalAmount`, `ContactName`, `CreatedAt`, chacun dans les deux sens, sur des fixtures où les cinq ordres
      **diffèrent** deux à deux (les quatre factures ci-dessus) ; échéancier — de même, sur **au moins quatre factures validées** (le brouillon n'y paraît pas ; validation P2, F7),
      cinq ordres distincts deux à deux ;
    - (f) **tri par défaut, au service et par la route**, en ordre rendu, avec au moins deux lignes distinguables :
      liste → `Date` `Desc` (ordre distinct de `DueDate` `Desc` et de `Date` `Asc`) ; échéancier → `DueDate` `Asc` ;
      fournisseur → `invoice_date DESC, id DESC` (l'ordre du dépôt, `supplier_invoices.rs:228`, non déplacé : test
      seulement) ; au service, la demande **vide** sous un contexte MCP `InvoicesRead` (le chemin de la 32-9a) ;
    - (g) **le détail** : une facture **partiellement** réglée par la route → `amountSettled` et `amountDue` distincts et
      justes ; un brouillon → `roundingIsPreview: true` (réglage lu) ; une facture validée → réglage non lu
      (`arrondi_5_centimes` à `None`, au service). Les tests existants `the_preview_follows_the_setting` et
      `the_invoice_carries_its_frozen_or_previewed_rounding` (`invoice_echeancier_e2e.rs:1467-1545`) restent les
      témoins de l'aperçu par la route.
12. **AC12 — lecture seule, filtre de société, lectures qui ne lisent pas la société.** Avec l'aide commune
    `lecture_seule` (`tests/common/lecture_seule.rs`) :
    - (a) les **six** services **réussissent** sur cette `Lecture`, sur des fixtures qui exercent : un brouillon (le
      réglage lu), une facture validée **portant un règlement** (la transaction de `reglements` et son `commit` en
      lecture seule, `settlement_cancel_blocker` exécuté), une facture fournisseur (la transaction et le `commit` de
      `get_settlement_view`) ; les branches **non** atteintes (motifs de refus présents, pièce source, lot confirmé…)
      sont **nommées** au Dev Agent Record, non revendiquées ;
    - (b) **filtre de société** : une seconde société (`seconde_societe`, l. 229-256) ; un contexte MCP de cette
      société, `InvoicesRead`, demande la fiche et les règlements d'une facture de la première, et la fiche d'une
      facture fournisseur de la première → `AppError::Database(DbError::NotFound)` ; les deux listes et l'échéancier →
      aucune ligne de la première (validation P1, R9) ;
    - (c) **ordres** : société inexistante **et** `limit = 0` pour `factures::lister` → `Internal` (la société est lue
      avant la validation) ; société inexistante et facture inexistante pour `factures::reglements` et
      `factures_fournisseur::detail` → `Internal` ;
    - (d) **ce qui ne lit pas la société, figé** (leçon 3) : contexte MCP sur une société inexistante, `InvoicesRead`
      — `factures::detail` d'un `id` quelconque → `NotFound` (non `Internal`) ; `factures::echeances` → `Ok`, page
      vide, résumé à zéro. Nommé aux doc-comments des deux services.
13. **AC13 — la garde lexicale, étendue (R8), dans `services_motif.rs`.** Même détecteur (`MOTIF_POOL`, corps aplati,
    `sans_commentaires`), même définition du corps (`corps_des_handlers`, nom exact) que la 32-4a.
    - (ii) `HANDLERS_MIGRES` gagne les six handlers du *Rollout* (douze au total) ;
    - (iii) `SITES_NON_RESOLUS` gagne `routes/invoices.rs` (**13** `pub async fn`, **15** `async fn`, neuf handlers sur
      `state.pool` : les huit mutants et `export_due_dates_csv_handler`) et `routes/supplier_invoices.rs` (**7**, **8**,
      les quatre mutants) ; et un **cinquième champ** à chaque ligne, les **aides hors corps** qui lisent
      `state.pool` — `&[]` pour les trois fichiers de la 32-4a, `["ensure_contact_belongs_to_company",
      "with_draft_rounding_preview"]` pour `invoices.rs`, `&[]` pour `supplier_invoices.rs`. Une aide est une fonction
      de premier niveau dont la ligne d'ouverture **commence** par `fn ` ou `async fn ` (corps : jusqu'à la première
      ligne égale à `}`, comme un handler). L'égalité au niveau du fichier devient : occurrences du fichier = somme
      sur les corps des handlers **+** somme sur les corps des aides ; et l'ensemble des aides qui lisent égale la liste
      attendue. Relevé attendu au dernier commit : `invoices.rs` **30** = 28 + 2, `supplier_invoices.rs` **11** = 11 + 0
      (aujourd'hui 40 = 38 + 2 et 15 = 15 + 0) ; les trois fichiers de la 32-4a inchangés, **11 / 8 / 8**
      = somme des corps, aucune aide (relevé à `592ee430` ; les « 15 / 10 / 12 » de la fiche 32-4a étaient ceux d'avant
      sa migration — validation P1, R1) ;
    - (vi) le compte de `services/` à **dix** ; (viii) `SERVICES_DU_ROLLOUT` à **cinq** (`services/factures.rs`,
      `services/factures_fournisseur.rs` ajoutés ; l'assertion `sources.len()` à 5) ;
    - (ix) **neuve — les seuls usages de `state` d'un handler migré.** Pour chacun des douze `HANDLERS_MIGRES`, sur le
      corps aplati sans commentaires : il contient `State(state)` (au motif `State\s*\(\s*state\s*\)`), et **chaque**
      occurrence de `\bstate\b` est l'une des formes permises — `State(state)`, `state.lecture()`, `state.trace()`,
      `state.i18n` (motifs à blancs tolérés : `state\s*\.\s*(lecture|trace)\s*\(\s*\)`, `state\s*\.\s*i18n\b`) ; le
      compte des occurrences de `\bstate\b` égale la somme des comptes des formes permises (C-32-4b-9). **Témoins** au
      test : la garde trouve une forme interdite dans `f(&state, x)`, `state.pool`, `let st = &state;`, `state.clone()`,
      et rejette un corps témoin sans `State(state)` (`State(AppState { pool, .. })`) ; elle accepte `&state.lecture()`,
      `state . lecture ()`, `&state.i18n`. Elle ferme, **pour les handlers migrés**, les formes de #738 B4 « aide
      appelée », « liaison renommée » et « déstructuration », que la 32-4a laissait aux limites (C-32-4b-9) ;
    - (i), (v-1), (v-2), (vii) **inchangées** — la v-2 compte toujours **deux** fabriques.
    ⚠️ La garde (ix) ne lit que les corps des **handlers migrés** : une aide synchrone appelée par l'un d'eux reçoit
    forcément `state` ou l'un de ses champs, que (ix) refuse ; mais une aide appelée **depuis une aide** n'est vue que
    par (iii). Ce que ces gardes ne voient pas : § *Ce que les gardes ne voient pas*.

### D — Inventaire, mutations, gates, documentation

14. **AC14 — inventaire clos des écritures et des verrous sur les six chemins.** Écrit au doc-comment de
    `services/mod.rs` (point 3 étendu) et au Dev Agent Record, au patron de l'AC13 de la 32-4a : la **fermeture des
    appels** (directs et transitifs, `const` SQL et aides pures comprises) est relevée au code et **listée** ; puis
    `grep -nE "INSERT|UPDATE |DELETE|FOR UPDATE|get_or_create"` sur le **code** (lignes de commentaire écartées) des
    fichiers de la fermeture, chacun tronqué à son `#[cfg(test)]` (numéros relevés et cités), chaque occurrence rangée
    sous sa fonction ou sa `const` ; **dans** la fermeture, **aucune** occurrence n'est attendue (fait 7) — une
    branche non prise, comme le `Lecture::Verrouillante` de `lettering_guard` pour la 32-4a, se nomme avec la preuve
    qu'elle n'est pas prise ; toute autre est un défaut à porter au Project Lead. Point (2) inchangé
    (`api_keys::touch_last_used`, middleware).
15. **AC15 — mutations qui doivent rougir** (une à la fois, motif vérifié unique, restauration `git checkout -- <f> &&
    touch <f>`, diff résiduel nul ; chacune rougit **au moins une assertion nommée**, relevée à son message — une
    erreur de compilation ou de montage n'est pas un rouge ; une ligne par mutation au Dev Agent Record, diffs aux
    journaux hors dépôt, C-32-4a-21) — **100** : **99 à tuer, et M21d, équivalente déclarée** (validation P1, R2) :
    - **Accès** — **M1a-f** : `exiger` retiré de chacun des six services → AC10 (a) ; **M2a-b** : `exiger` après la
      validation (`factures::lister`, `factures::echeances`) → AC10 (c-val) ; **M3a-e** : `exiger` après la première
      lecture (`factures::{lister, detail, reglements}`, `factures_fournisseur::{lister, detail}`) → (c-lec) ;
      **M4a** : `EXIGENCE_DETAIL` des factures à `BooksRead` ; **M4b** : celle de `factures_fournisseur::lister` à
      `ContactsRead` → (a) ; **M5** : `role_api` de `factures::reglements` à `Comptable` → (e) ;
    - **Société et ordre** — **M6** : `factures::lister` valide avant de lire la société → AC12 (c) ; **M7a-b** :
      `company_for_ids` ajouté en tête de `factures::detail`, de `factures::echeances` → AC12 (d) ; **M8a-b** :
      `factures::reglements`, `factures_fournisseur::detail` lisent la facture avant la société → AC12 (c) ; **M9a-b** :
      **filtre de société perdu** — dans `factures::detail` et `factures_fournisseur::detail`, la société passée au
      dépôt est lue de la facture elle-même (`let cid: i64 = sqlx::query_scalar("SELECT company_id FROM invoices WHERE
      id = ?").bind(id).fetch_one(lecture.pool()).await.map_err(kesh_db::errors::map_db_error)?;`, resp.
      `supplier_invoices`) → AC12 (b) ;
    - **Validation et bornes** — **M10a** : contrôle de `limit` retiré ; **M10b** : `1..=` → `0..=` ; **M10c** : défaut
      20 → 100 ; **M10d** : contrôle d'`offset` négatif retiré → AC11 (a), (b), (c) ; **M11a** : plafond d'`offset`
      retiré ; **M11b** : `>` → `>=` ; **M11c** : contrôle `dueBefore < dateTo` retiré ; **M11d** : `dueBefore <
      dateFrom` placé avant `dateFrom > dateTo` → AC11 (a) ; **M12a** : longueur de `search` `>` → `>=` ; **M12b** :
      `search` non rogné → AC11 (a), la ligne `" " + "a"×100 + " "` (refusée sous la mutation : 102 > 100), pour la liste
      et pour l'échéancier ; **M13** : `cancelled` retiré de la liste blanche du statut → AC11 (c) ; **M14a-d** :
      fournisseur — plafond perdu (`clamp(1, MAX)` → `max(1)`), plancher perdu (`→ min(MAX)`), `max(0)` retiré, défaut
      50 → 200 → AC11 (b), (c) ;
    - **Recopies (leçon 1)** — chaque champ remplacé par sa valeur neutre (`None`, `Default::default()`, ou le défaut du
      service) : **M15** au service, de `DemandeListeFactures` vers `InvoiceListQuery`, et **M16** au handler, de
      `ListInvoicesQuery` vers la demande — dix champs chacun (`search`, `status`, `contact_id`, `date_from`, `date_to`,
      `paused`, `sort_by`, `sort_direction`, `limit`, `offset`) → AC11 (c), (d), (e) ; **M17** au service
      (`requete_echeances`) et **M18** à la conversion du handler — dix champs chacun (`search`, `contact_id`,
      `date_from`, `date_to`, `due_before`, `payment_status`, `sort_by`, `sort_direction`, `limit`, `offset`) →
      AC11 (d), (e) ; **M19a-d** : `limit` et `offset` du fournisseur, au service et au handler → AC11 (c), (d) ;
    - **Défauts du tri (leçon 2)** — **M20a** : défaut de `sort_by` de la liste → `DueDate` ; **M20b** : sens → `Asc` ;
      **M20c** : défaut de l'échéancier → `Date` ; **M20d** : sens → `Desc` → AC11 (f) ;
    - **Constantes de l'échéancier** — **M21a** : `status` → `None` ; **M21b** : `paused` → `Some(PausedFilter::Paused)` ;
      **M21c** : défaut de `payment_status` → `All` → AC11 (d) (M21a par la ligne `paymentStatus=all` avec un
      brouillon présent ; M21b par la facture **non** suspendue attendue — `paused_invoice_stays_visible_in_due_dates`
      ne la tue pas, sa seule facture étant suspendue) ; **M21d** : le résumé reçoit `payment_status` — **équivalente**,
      jouée et déclarée : `due_dates_summary` ne lit pas `query.payment_status` (« Filtres ignorés : `payment_status` »,
      `kesh-db/src/repositories/invoices.rs:937-938` ; corps l. 943-1024, filtres `contact_id`, `date_from`, `date_to`,
      `due_before`, `search` seuls ; test du dépôt `test_due_dates_summary_ignores_payment_status_filter`, l. 5349) — le
      retrait « B15 » est une défense en profondeur, gardé tel quel ; le Dev Agent Record recopie sa survie ;
    - **Détail et règlements** — **M22a** : `arrondi_5_centimes` toujours `None` ; **M22b** : condition du brouillon
      inversée ; **M22c** : `montant_regle` et `reste_du` permutés → AC11 (g) et les témoins de `invoice_echeancier_e2e` ;
      **M23** : `settlement_cancel_blocker` sur une connexion acquise de la `Lecture` au lieu de `&mut tx` → garde
      `reglements_lisent_dans_une_transaction` ; **M24** : une écriture propagée sur la `Lecture` dans
      `factures::reglements` (`sqlx::query("INSERT INTO audit_log (user_id, action, entity_type, entity_id) VALUES (1,
      'invoice.read', 'invoice', 0)").execute(lecture.pool()).await.map_err(kesh_db::errors::map_db_error)?;`) →
      AC12 (a), erreur 1792 recopiée ;
    - **Gardes** — **M25** : `get_invoice` relit la facture par `state.pool` → (ii), (iii) ; **M26** : `get_invoice`
      garde l'appel `with_draft_rounding_preview(&state, …)` en plus du service → (ix) **seule** — (ii) reste vert, ce
      que le Dev Agent Record recopie : c'est le témoin que (ix) voit ce que (ii) ne voit pas ; **M27** :
      `list_supplier_invoices` déstructure `State(AppState { pool, .. })` et lit `&pool` → (ix) ; **M28** : une aide
      neuve `async fn lire_facture(state: &AppState, …)` sur `state.pool` dans `invoices.rs`, appelée par un **mutant**
      → (iii), liste des aides ; **M29** : `let _t: Option<&crate::services::poignees::Trace> = None;` dans
      `factures::detail` → (viii).
    Décompte : 6 + 2 + 5 + 2 + 1 = 16 (accès) ; 1 + 2 + 2 + 2 = 7 (société) ; 4 + 4 + 2 + 1 + 4 = 15 (bornes) ;
    10 + 10 + 10 + 10 + 4 = 44 (recopies) ; 4 (tri) ; 4 (constantes, dont M21d équivalente) ; 3 + 1 + 1 = 5 (détail) ;
    5 (gardes) — **100**, dont 99 à tuer.
16. **AC16 — gates.** `kesh-db` n'est **pas** touché (aucun dépôt, aucune migration, ni `post_restore.rs`). En boucle,
    **gate ciblé** : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, puis
    `cargo nextest run -E 'package(kesh-api) & kind(lib)'` et les binaires `services_rollout`, `services_motif`, ceux de
    l'AC9 et les huit gardes (`binary(…)` par nom, liste recopiée au Dev Agent Record). Au **dernier commit de code** :
    **gate complet** (`scripts/test-fast.sh --ci` sous `flock`, bases `kesh_324b` et `kesh_e2e_324b` remises à zéro
    d'abord, sans toucher au conteneur), les **quatre gates frontend** et l'**E2E Playwright complet** (recette de
    `docs/testing.md`, port attribué par l'orchestrateur, jugé fichier par fichier contre § *Les échecs attendus*).
    ⛔ « Rejouée au push » est interdit. Ne déclarer que ce qui a tourné, avec les chiffres.
17. **AC17 — documentation.** Rien de visible ne change (C-32-4b-14) : ni manuel, ni CHANGELOG, ni README, ni
    `docs/api-external.md`. **Le contrôle se fait quand même**, sortie recopiée : `api-external.md` l. 215, 497, 552-566,
    667, 723-757 relues contre le code déplacé (champs de la fiche fournisseur, `null` = non calculé) ;
    `grep -niE "factures|échéancier|par page|pagination" docs/manual/fr/user-manual.tex` et les PDF aplatis
    (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE ".{60}(par page|pagination|100 factures|200 factures).{60}"`)
    → aucun texte ne décrit autrement les bornes. **Textes qui bougent** : (1) la note de
    `docs/MULTI-TENANT-SCOPING-PATTERNS.md` (l. 273-281) nomme les six lectures et dit que `GET /invoices/{id}` et
    `GET /invoices/due-dates` ne lisent pas la société (C-32-4b-8) ; (2) le doc-comment de `services/mod.rs` : le tableau
    du rollout de la 32-4b, l'inventaire de l'AC14, les lectures laissées et leur raison, la balance âgée « déjà un
    service » ; (3) **Q11 — tous les sites du texte antérieur** (décision de l'orchestrateur, validation P1, R4 ;
    C-32-4b-12, C-32-4b-16) : `services/mod.rs` — R4 (l. 52-53, « la 32-6 l'imposera par la base ») et « Ce qui reste à
    la 32-6 » (l. 132-139, dont l. 134-136, ceux que le registre de la 32-5 cite « 93-95 » sur sa base `89d68704`) —,
    `services/poignees.rs:9-12`, `lib.rs:147-149` et `:158-160`, `tests/common/lecture_seule.rs:8-9`, et tout autre site que
    rend le relevé ci-dessous, disent la décision **prise** : lecture seule tenue par l'application (les outils
    n'atteignent que des services qui reçoivent une `Lecture`) et mode de session `READ ONLY` sur la connexion du MCP,
    réalisés par la 32-6, **aucun second compte MariaDB** ; la garantie repose sur le code de Kesh et ses tests, la base
    n'arrêtant pas un défaut qui contournerait le mode de session ; `lecture_seule.rs` dit ce qui distingue encore la
    `Lecture` de test de celle du MCP : le même mode de session, sur un pool de test, sans la fabrication de production
    (F6). ⚠️ Le texte écrit n'emploie aucune des chaînes du contrôle (validation P1, R3 = F2) ; la décision se cite
    librement — « la réponse de Guy à Q11 », « Q11, tranchée le 2026-10-24 » (validation P2, R3). Contrôle, sur
    `crates/` **entier** :
    `grep -rnE 'limité au .SELECT.|utilisateur .SELECT.|l.imposera par|sera celle de la base|est décidée par Guy|rien ici n.en préjuge|que décide la 32-6|n.est \*\*pas\*\* la lecture seule de production' crates/`
    → **rien** après (dix lignes, six sites avant, fait 10) ; (4) les doc-comments de module de `routes/{invoices,supplier_invoices}.rs`
    renvoient en une ligne aux services pour les lectures migrées. Aucun écran ne change : recette visuelle sans objet.
18. **AC18 — un défaut de test voisin, intégré** (décision de l'orchestrateur, validation P1 ; C-32-4b-17).
    `tests/invoice_echeancier_e2e.rs` : `seed_base` rend `(seeded.admin_user_id, seeded.company_id)` (l. 112-117) ; les
    l. 402, 453 et 500 le lient en `let (company_id, admin_id) = …`, et la l. 538 en `(_company_id, _admin_id)` —
    verts par la seule coïncidence des deux identifiants dans une base éphémère. Correction : ces quatre liaisons en
    `(admin_id, company_id)` (resp. `(_admin_id, _company_id)`), **aucune autre ligne** du fichier. Contrôles :
    `grep -nE 'let \(_?company_id, _?admin_id\) = seed_base' crates/kesh-api/tests/invoice_echeancier_e2e.rs` → rien ;
    `git diff $(git merge-base HEAD origin/main) HEAD -- crates/kesh-api/tests/invoice_echeancier_e2e.rs | grep -cE '^[-+]\s'`
    → **8** (quatre lignes ôtées, quatre ajoutées) ; le binaire `invoice_echeancier_e2e` vert, compte exécuté recopié.
    Ce que le contrôle ne prouve pas : que les trois tests auraient rougi avec des identifiants distincts — aucune
    mutation n'est demandée (correction d'un test, non d'une règle).

## Tâches

- [ ] **T0** — Worktree et branche (posés) ; bases `kesh_324b`, `kesh_e2e_324b` créées, migrées depuis les migrations du
  worktree, seedées ; port attribué par l'orchestrateur ; `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-32-4b/target` ;
  `node_modules` par `cp -al` (`docs/agents-orchestration.md` § 4). **Étape 0** : recontrôler au code les lignes citées
  des faits 1 à 10 (une ligne lue citée par fait au Dev Agent Record), et la faisabilité des fixtures de l'AC11 (une
  facture validée, un règlement en `internal_account`, une facture fournisseur, sur `seed_accounting_company`).
  ⚠️ **Le seed laisse l'arrondi à 5 centimes actif sans compte d'arrondi** (`kesh-db/src/test_fixtures.rs:203-212`) :
  chaque facture validée des fixtures a un TTC **multiple de 0.05** (patron de `create_validated_invoice`, 100.00 à
  8.10 % = 108.10), sans `disable_rounding_to_5_centimes`, qui ruinerait le cas « brouillon → réglage lu »
  (validation P2, F8). **Ordre de fusion** (décision de l'orchestrateur, validation P2, F5) : la 32-4b fusionne
  **avant** la 32-5b ; si `main` porte déjà la 32-5b au développement, recompter `services/` (garde (vi)) et
  `Portee` (AC10 (a)) sur `main` et le porter au Change Log (C-32-4b-18).
- [ ] **T1 (AC10-AC12 — rouge d'abord)** — `tests/services_rollout.rs` complété, en **deux commits** : (i) AC11 (c), et
  les lignes **par la route** de l'AC11 (d), (e), (f), (g) — elles ne réfèrent à aucun service neuf —, **jouées sur le
  code d'avant T2** : le relevé établit que les tables décrivent le code d'aujourd'hui (un rouge ici est une prémisse
  fausse de la fiche, à porter au Change Log, non à corriger dans la table) ; (ii) le reste, dont on constate qu'il ne
  compile pas avant T2.
- [ ] **T2 (AC1-AC6, AC8)** — `services/factures.rs` (`lister`, `detail`, `requete_echeances`, `echeances`,
  `reglements`, la garde de transaction) ; les quatre handlers minces ; l'export sur `requete_echeances` ; gate ciblé.
- [ ] **T3 (AC7, AC8)** — `services/factures_fournisseur.rs` ; `from_view` ; les deux handlers minces ; gate ciblé.
- [ ] **T3-bis (AC18)** — les quatre liaisons de `invoice_echeancier_e2e.rs` ; binaire `invoice_echeancier_e2e`.
- [ ] **T4 (AC9, AC13)** — amendements de `services_motif.rs` (liste fermée de l'AC9 (1)), garde (ix) et ses témoins.
- [ ] **T5 (AC14, AC17)** — inventaire de la fermeture ; les sites Q11 de l'AC17 (3) (`services/mod.rs`, `poignees.rs`,
  `lib.rs`, `tests/common/lecture_seule.rs`) et leur `grep` sur `crates/` ; textes de l'AC17.
- [ ] **T6 (AC15)** — mutations M1 à M29, chacune tuée sauf M21d (jouée, survie recopiée) (pilote hors dépôt, `kesh-gate-logs/324b-mutations-pilote.py`,
  au patron de `324a-p1r-mutations-pilote.py` ; binaire `services_rollout` seul quand la mutation ne vise pas une
  garde de `services_motif` ou un test existant nommé).
- [ ] **T7 (AC9, AC14, AC16, AC17)** — contrôles de l'AC8 et de l'AC9, gate complet et E2E complet au dernier commit de
  code ; Dev Agent Record : chiffres réellement exécutés, listes exactes, inventaire, décomptes recomptés (périmètre
  déclaré).

## Ce que les gardes ne voient pas

Écrit en clair, comme la 32-3 (C-32-3-31) et la 32-4a (AC12, *Les limites qui restent*) — chacune de ces formes
compile et laisse les gardes vertes ; **la revue de code les lit** :

- **dans `services/`** : un service qui emploie `lecture.pool()` pour écrire (seule l'exécution sur la `Lecture` de
  test le voit, sur les branches que les fixtures exercent — AC12 (a)) ; un `commit` remplacé par un `rollback` dans
  `reglements` (non observable : rien n'est écrit) ; une lecture **verrouillante** ajoutée sur une branche que les
  fixtures n'atteignent pas (la `Lecture` de test refuse le `FOR UPDATE`, mais seulement s'il est exécuté) ;
- **dans un handler migré** : un appel à une fonction **d'un autre module** qui reçoit `state.lecture()` et en fait autre
  chose qu'une lecture (la poignée ne s'ouvre que dans `services/`, mais un service d'un autre fichier pourrait
  écrire) ; une lecture **hors service** qui ne passe pas par `state` — un `static` global, un pool reçu par une
  `Extension` — que (ix) ne voit pas, faute de jeton `state` ;
- **dans les fichiers visés** : une aide appelée **par une aide** et non par un handler (vue au niveau du fichier par
  (iii) seulement si elle lit `state.pool` sous ce nom) ; les formes de #738 B4 **hors** des handlers migrés —
  déstructuration, liaison renommée, `self.pool`, `(&state).pool`, `state.clone().pool`, macro à paramètre ident —, qui
  restent aux limites de (iii) ; une chaîne contenant `//` ou `/*` (limite de `sans_commentaires`, aucune aujourd'hui
  dans les deux fichiers, fait 6) ;
- **les DTO** : un défaut serde ajouté à un DTO (un `#[serde(default = …)]` qui poserait une borne au handler, que le
  MCP ne verrait pas) — ni (ix) ni le `grep` de l'AC8 ne lisent les DTO ; la revue les lit ;
- **le comportement** : un message de refus reformulé **au service** est vu par l'AC11 (a) ; un message recopié au
  handler en plus est vu par le compte de l'AC8 ; un message **changé dans un mutant** n'est vu par rien de cette story.

## Dev Notes

### Sites

- `crates/kesh-api/src/routes/invoices.rs` — constantes l. 52-59 ; `ListInvoicesQuery` l. 120-145 ; `InvoiceResponse`
  et `with_settlement`/`with_rounding_preview` l. 268-292 ; `with_draft_rounding_preview` l. 299-313 ;
  `ensure_contact_belongs_to_company` l. 552-566 ; `list_invoices` l. 597-677 ; `get_invoice` l. 679-706 ;
  `PaymentStatusParam`/`PausedParam` l. 951-992 ; `ListDueDatesQuery` l. 994-1019 (struct l. 996) ; `build_due_dates_query`
  l. 1044-1138 ; `list_due_dates_handler` l. 1141-1180 ; `InvoiceSettlementResponse` l. 1397-1417 ;
  `list_invoice_settlements_handler` l. 1427-1471 ; `export_due_dates_csv_handler` l. 1552-1690 ; `mod tests` l. 1696.
- `crates/kesh-api/src/routes/supplier_invoices.rs` — `SupplierInvoiceResponse` l. 62-128 ;
  `load_with_settlement_cancellation` l. 204-226 ; `ListSupplierInvoicesQuery` l. 540-544 ; `list_supplier_invoices`
  l. 551-566 ; `get_supplier_invoice` l. 569-579 ; `mod scan_qr_tests` l. 790.
- `crates/kesh-api/src/services/{mod,acces,poignees}.rs` — liste des modules (l. 143-149), inventaire (l. 76-111),
  rollout de la 32-4a (l. 113-130), « Ce qui reste à la 32-6 » (l. 132-139) ; `Portee::InvoicesRead` (l. 26-27) ;
  doc-comment de `poignees.rs` (l. 9-12).
- `crates/kesh-api/tests/services_motif.rs` — l. 2091-2122 ((ii)), 2124-2131 (`HANDLERS_MIGRES`), 2138-2168
  (`SITES_NON_RESOLUS`), 2174-2205 (détecteur, `code_avant_les_tests`, `corps_du_handler`), 2207-2290 (témoins et (iii)),
  2292-2325 ((viii)), 2356-2376 ((vi)).
- `crates/kesh-api/tests/services_rollout.rs` — montage l. 55-158 ; `Livres`/`poser_livres` l. 166-226 ;
  `seconde_societe` l. 229-256 ; contextes l. 259-285 ; `Svc`/`SERVICES`/`portee_exigee`/`jouer` l. 373-452 ;
  `est_*` l. 461-476 ; `message_de_validation` l. 668 ; `table_des_bornes` l. 851.
- `crates/kesh-api/src/services/ecritures.rs` — la garde `get_journal_entry_reads_in_one_transaction`, patron de
  `reglements_lisent_dans_une_transaction`.
- `crates/kesh-api/src/helpers.rs:76-90` — `company_for_ids`, réemployée telle quelle.
- `docs/MULTI-TENANT-SCOPING-PATTERNS.md:273-281`.

### Esquisse (indicative ; noms et règles R1-R8 exigés, détail au développement)

```rust
// services/factures.rs
pub const LIMITE_DEFAUT: i64 = 20;  pub const LIMITE_MAX: i64 = 100;  pub const RECHERCHE_MAX: usize = 100;
pub const OFFSET_MAX_ECHEANCES: i64 = 1_000_000;
pub const EXIGENCE_LISTER: Exigence = Exigence { role_api: Role::Consultation, portee_mcp: Some(Portee::InvoicesRead) };
// EXIGENCE_DETAIL, EXIGENCE_ECHEANCES, EXIGENCE_REGLEMENTS : idem.
#[derive(Debug, Clone, Default)]
pub struct DemandeListeFactures { pub search: Option<String>, pub status: Option<String>, pub contact_id: Option<i64>,
    pub date_from: Option<NaiveDate>, pub date_to: Option<NaiveDate>, pub paused: Option<PausedFilter>,
    pub sort_by: Option<InvoiceSortBy>, pub sort_direction: Option<SortDirection>,
    pub limit: Option<i64>, pub offset: Option<i64> }
pub async fn lister(acces: &ContexteAcces, lecture: &Lecture, d: DemandeListeFactures) -> Result<InvoiceListResult, AppError>;
pub struct FicheFacture { pub facture: Invoice, pub lignes: Vec<InvoiceLine>, pub montant_regle: Decimal,
    pub reste_du: Decimal, pub arrondi_5_centimes: Option<bool> }
pub async fn detail(acces: &ContexteAcces, lecture: &Lecture, id: i64) -> Result<FicheFacture, AppError>;
#[derive(Debug, Clone, Default)]
pub struct DemandeEcheances { pub search: Option<String>, pub contact_id: Option<i64>, pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>, pub due_before: Option<NaiveDate>, pub payment_status: Option<PaymentStatusFilter>,
    pub sort_by: Option<InvoiceSortBy>, pub sort_direction: Option<SortDirection>,
    pub limit: Option<i64>, pub offset: Option<i64> }
pub fn requete_echeances(d: DemandeEcheances) -> Result<InvoiceListQuery, AppError>;   // pure, partagée avec l'export
pub struct Echeancier { pub page: InvoiceListResult, pub resume: DueDatesSummary }
pub async fn echeances(acces: &ContexteAcces, lecture: &Lecture, d: DemandeEcheances) -> Result<Echeancier, AppError>;
pub struct ReglementFacture { pub reglement: InvoiceSettlement, pub motif: Option<SettlementCancelHit> }
pub async fn reglements(acces: &ContexteAcces, lecture: &Lecture, id: i64) -> Result<Vec<ReglementFacture>, AppError>;
// services/factures_fournisseur.rs
pub const LIMITE_DEFAUT: i64 = 50;  pub const LIMITE_MAX: i64 = 200;
pub struct DemandeListeFacturesFournisseur { pub limit: Option<i64>, pub offset: Option<i64> }
pub struct PageFacturesFournisseur { pub items: Vec<SupplierInvoiceListItem>, pub total: i64, pub limit: i64, pub offset: i64 }
pub async fn lister(acces: &ContexteAcces, lecture: &Lecture, d: DemandeListeFacturesFournisseur) -> Result<PageFacturesFournisseur, AppError>;
pub async fn detail(acces: &ContexteAcces, lecture: &Lecture, id: i64) -> Result<SupplierInvoiceSettlementView, AppError>;
```

Les paramètres s'appellent `acces` et `lecture` (comme `plan_comptable::lister` et les services de la 32-4a) : la garde
de transaction de l'AC6 compte `pool().begin()` sur `lecture.pool().begin()`.

### Les refus et non l'écrêtage (C-32-4b-6)

Les listes clientes **refusent** une taille de page hors bornes là où la 32-4a écrêtait : c'est le comportement
d'aujourd'hui (fait 3), conservé ; le MCP le reçoit tel quel, et l'outil de la 32-9a — qui choisit ses bornes « sous
10 000 jetons » (spike l. 961) — passera une taille dans `1..=100` ou `None`. Une page de 100 factures ou de 200
factures fournisseur peut dépasser ce budget : c'est l'outil qui borne plus bas, jamais le service qui change le
plafond de l'écran.

### Points à tracer par l'orchestrateur (« on trace toujours »)

- **Deux commentaires que la story rend faux, hors de son périmètre d'écriture** : `kesh-db/src/repositories/invoices.rs:410-415`
  (« `list_due_dates_handler` appelle `list_by_company_paginated` ») et `kesh-api/tests/dunning_reminders_e2e.rs:715-718`
  (« un `build_due_dates_query` qui cesserait de poser `paused: None` ») — `kesh-db` n'est pas touché (AC16), et l'AC9
  ferme la liste des tests modifiés. À verser sur **#738**, comme les deux commentaires de `kesh-db` de la 32-4a.
- **La plage des dates des listes de factures** : `dateFrom`, `dateTo`, `dueBefore` sont des `NaiveDate` du DTO sans la
  plage 1000-9999 qu'exige `parse_as_of` — même classe que **#742** (écritures), **non établie à l'exécution**. Hors
  périmètre (comportement inchangé) ; à ajouter en commentaire sur #742.
- *(Le défaut de `seed_base` d'`invoice_echeancier_e2e.rs`, d'abord proposé ici en issue P3, est intégré à la story :
  AC18, décision de l'orchestrateur à la validation P1.)*

### Fichiers touchés (prévus)

`crates/kesh-api/src/services/{factures,factures_fournisseur}.rs` (neufs) ; `services/mod.rs`, `services/acces.rs`
(doc-comment), `services/poignees.rs` (doc-comment) ; `crates/kesh-api/src/routes/{invoices,supplier_invoices}.rs` ;
`crates/kesh-api/src/lib.rs` (deux doc-comments seuls, l. 147-149 et 158-160 — aucune route ni poignée neuve) ;
`crates/kesh-api/tests/services_rollout.rs`, `tests/services_motif.rs`, `tests/invoice_echeancier_e2e.rs` (AC18),
`tests/common/lecture_seule.rs` (doc-comment) ; `docs/MULTI-TENANT-SCOPING-PATTERNS.md` ;
`_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (annotation de l'AC de la 32-4, C-32-4b-1 — **faite** à la
validation P1, `dfc61b17` : le développement n'y touche pas ; validation P2, F2).
**Non touchés** : `kesh-db`, `helpers.rs`, `routes/reports.rs`,
`routes/imported_supplier_invoices.rs`, frontend, i18n, manuels, CHANGELOG, `docs/api-external.md`.

### Découpage préventif — verdict

Modules de premier niveau touchés : `kesh-api/services`, `kesh-api/routes/invoices`,
`kesh-api/routes/supplier_invoices`, et la racine `kesh-api/lib.rs` pour deux doc-comments (validation P1, R4) — **4** ;
quatre fichiers de tests (dont un helper, `tests/common/`) et deux documents en plus, que le critère ne compte pas. La
balance âgée (`routes/reports`) n'est **pas** touchée (fait 8). Sous le seuil de 5 : **non découpée**. Le second
critère (non-convergence) se jugera en validation.

### Risques

- **Faible — un conflit avec la 32-5b** (développée en parallèle, `kesh-32-5`) : elle ajoute `services/cles_mcp.rs` et
  porte `Portee` à cinq variantes. **Ordre décidé** : la 32-4b fusionne avant ; la 32-5b compte relativement à `main` et
  recompte à son étape 0 (C-32-4b-18).

- **Moyen — un refus ou une borne perdus.** Aucun des trois dépôts n'écrête et aucun test ne fige les douze refus
  (faits 3 et 4) : T1 (i) les fige **par la route avant** le déplacement ; M10 à M14.
- **Moyen — un filtre ou un tri mal recopié** — la leçon 1 de la 32-4a, sur deux listes à dix champs : AC11 (d), (e),
  M15 à M19, toutes jouées.
- **Faible — l'instantané des règlements** perdu au déplacement (aucune garde aujourd'hui) : garde neuve (AC6), M23.
- **Faible — l'aperçu d'arrondi** déplacé : AC11 (g), M22, et les deux témoins existants.
- **Faible — la garde (ix) trop stricte** pour un handler déjà migré : les six de `HANDLERS_MIGRES` (`list_accounts` de
  la 32-3, cinq de la 32-4a ; validation P1, R11 = F6) n'emploient que `State(state)`,
  `state.lecture()` et `state.i18n` (relevé `awk` sur leurs corps, 2026-10-25) ; un rouge sur eux au premier passage
  est une prémisse fausse, à porter au Change Log.

### Références

`CLAUDE.md` § *Test Locally First*, § *Pendant une boucle de revue*, § *Review Iteration Rule*, § *Règle de splitting
préventif*, § *Inventorier les sites NON RÉSOLUS*, § *Recompter ses propres comptes rendus* ;
`docs/agents-orchestration.md` § 2-7 ; `planning-artifacts/epic-32-serveur-mcp.md` § *32-4*, § *32-9* ;
`epic-32-spike-mcp.md` l. 925-926, 961, 1061-1067 ; fiches `32-3-services-motif.md`, `32-4a-services-ecritures-contacts.md` ;
registres `epic-32-choix-autonomes.md` (C-32-0-4, C-32-0-7), `-323.md`, `-324a.md` ; #738, #742.

## Points à soumettre à Guy

- **Q1 — les factures sous `invoices:read` portent des données de contact.** La liste des factures rend le nom du
  contact (`contactName`), la fiche fournisseur l'IBAN du créancier (`creditorIban`, `creditorQrIban`,
  `supplier_invoices.rs:70-71`) ; C-32-0-7 range ces lectures sous `invoices:read`, et les contacts ont leur propre
  portée « parce qu'ils portent des noms et des IBAN » (Q5 de l'epic, sans réponse). La story suit C-32-0-7
  (C-32-4b-3). **La question** : une clé `invoices:read` sans `contacts:read` peut-elle lire ces champs — ou faut-il
  que la 32-9a les retire, ou qu'elle exige les deux portées pour la fiche fournisseur ? Le développement peut partir :
  la réponse se traite à l'outil (32-9a) ou par une constante d'exigence, sans défaire la story.
- **Q2 — l'export CSV de l'échéancier, écart à la lettre de l'AC de l'epic** (validation P1, F3 = R12). L'AC de la 32-4
  écrit « aucun handler de lecture des modules visés n'appelle plus un dépôt directement » (`epic-32-serveur-mcp.md`,
  § 32-4) ; la story laisse `export_due_dates_csv_handler` (`GET /invoices/due-dates/export.csv`, Comptable+) sur
  `state.pool`, faute d'outil qui rende un CSV — inventorié, figé par la garde (iii), sa validation étant celle du
  service. C'est le motif de la Q1 de la 32-4a, que Guy a confirmée pour les deux lectures du lettrage ; sa
  confirmation ne couvre pas celle-ci. L'epic est annoté d'une ligne datée. **La question** : acceptez-vous la même
  lecture pour l'export — ou faut-il le migrer (un service `Comptable` / jamais servi au MCP) ? Le développement part
  sur la première ; la seconde s'ajoute sans défaire le travail. À noter aussi, une asymétrie écrite au registre
  (C-32-4b-1) : les règlements sont migrés parce que l'epic les nomme, bien que la 32-9a ne les nomme pas.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- **2026-10-25 — fiche rédigée** (Opus 5.5, en autonomie, worktree `kesh-32-4b`, base `592ee430`) : 17 AC, 100
  mutations nommées (aucune équivalente déclarée d'avance), 3 modules (non découpée) ; registre
  `epic-32-choix-autonomes-324b.md`, C-32-4b-1 à 14 ; les trois leçons de revue de la 32-4a intégrées (recopies,
  tri par défaut, lectures sans société) ; la garde (ix) neuve, pour le trou que `get_invoice` rencontre (une aide
  appelée par `&state`). Validation P1 à lancer (`324b-validate-prompt-p1.md`, Sonnet, lentilles R et F).
- **2026-10-25 — validation P1** (Sonnet ×2, contexte frais, lecture seule ; prompt `324b-validate-prompt-p1.md` ;
  rapports hors dépôt `kesh-gate-logs/324b-validate-p1-{R,F}.md`) : **R 0/0/4/10, F 0/0/1/6**, soit 21 findings,
  **19 distincts** après recoupement (R3 = F2, R7 = F1 ; R12 = F3, R11 recoupe F6 en partie) — **5 MEDIUM** (R1, R2,
  R3 = F2, R4, F1 = R7), le reste LOW. Le code déplacé et les décomptes neufs (40 / 38 / 2, 15, 34 → 22, 25, 100) se
  recomptent à la source ; les défauts sont de **rédaction des contrôles** et d'un symptôme non propagé. Remédiation
  par l'orchestrateur de la fiche (Opus 5.5), chaque prémisse relue au code :
  - **R1** — appliqué : les comptes de la 32-4a recomptés à `592ee430` par le relevé du détecteur (corps aplatis,
    commentaires ôtés) : `journal_entries.rs` 11, `letterings.rs` 8, `contacts.rs` 8, chacun égal à la somme de ses corps
    — les « 15 / 10 / 12 » étaient ceux d'avant la migration de la 32-4a (AC13 (iii)).
  - **R2** — appliqué : M21d **déclarée équivalente**, preuve au dépôt (`invoices.rs:937-938`, `:5349`) ; AC15 : « 99 à
    tuer, et M21d » ; l'assertion « B15 » de l'AC11 (d) dite contrat de la route, non témoin du retrait.
  - **R3 = F2** — appliqué : le texte prescrit n'emploie plus les chaînes du contrôle (« aucun second compte
    MariaDB ») ; le motif du `grep` porte sur les valeurs d'origine.
  - **R4** — appliqué selon la décision de l'orchestrateur : **tous** les sites antérieurs à Q11 — `services/mod.rs`
    (l. 52-53, 132-139), `poignees.rs:9-12`, `lib.rs:147-148` et `:158-159`, `tests/common/lecture_seule.rs:9` —,
    relevés par le `grep` de la valeur sur `crates/` entier (six sites, aucun autre ; les « par la base » et « ou les
    deux » du reste de `crates/` sont sans rapport, relus) ; `lib.rs` et le fichier de test entrent au périmètre ;
    modules recomptés : **4** (C-32-4b-16, mention de la 32-5 b et c au registre).
  - **F1 = R7** — appliqué : `grep -rnF`, et le commentaire « M4 » de l'export (l. 1562-1566) reformulé par l'AC5.
  - **Défaut voisin intégré** (décision de l'orchestrateur) : `seed_base` d'`invoice_echeancier_e2e.rs` — **AC18**,
    T3-bis, C-32-4b-17 ; le fichier entre à la liste fermée de l'AC9 ; la l. 538 (liaison `_`-préfixée inversée)
    comprise, relevée en relisant les appelants (`grep -n 'seed_base(&pool)'` : 32 appels, quatre inversés).
  - **F3 = R12** — appliqué : écart consigné (C-32-4b-1), epic annoté, **Q2** posée.
  - **LOW traités** : R5, R6 (AC11 (d), M21a/b), R8 (`api-external.md:497`, `:667`), R9, R10, R11 (lignes, « six
    handlers migrés »), R13 (`git diff --name-only` sans filtre), R14 (commentaires « cinq services »), F4 (deux
    montages, C-32-4b-15), F5 (quatre factures, `created_at` posé), F6 (préfixe `ac324b_`), F7 (« périmé » au fait 10).
  - **Propagation** (grep des valeurs) : `15 / 10 / 12` → seule l'entrée vivante de l'AC13 (la fiche 32-4a, relevé
    daté, non touchée) ; `aucune équivalente` → AC15 et l'entrée « fiche rédigée » ci-dessus (relevé daté, laissée) ;
    `diff-filter=MDR`, `grep -n 'build_due_dates_query'`, `utilisateur MariaDB limité`, `les trois listes`,
    `2365-2369`, `2126-2133`, `980-985`, `961-970`, `987-995` → traités ; `C-32-4b-1 à 14` → `1 à 17` dans l'en-tête
    et `sprint-status.yaml` ; `17 AC` → **18 AC**.
  - **Trend** : P1 = 5 MEDIUM (+ 14 LOW distincts). La boucle ne se clôt que sur une passe à 0 MEDIUM : **P2 complète**,
    **Opus**, lentilles R et F (`324b-validate-prompt-p2.md`).
- **2026-10-25 — validation P2** (Opus ×2, contexte frais, lecture seule, passe complète ; prompt
  `324b-validate-prompt-p2.md` ; rapports hors dépôt `kesh-gate-logs/324b-validate-p2-{R,F}.md`) : **R 0/0/1/4,
  F 0/0/1/7**, soit 13 findings, **11 distincts** (R1 = F1, R2 = F6) — **1 MEDIUM** (R1 = F1), 10 LOW. Les 21 findings de
  la P1 sont jugés **traités**, aucun mal traité ; les faits 1 à 10, la fermeture transitive (fait 7), les cent
  mutations une à une et les décomptes se recomptent à la source. Remédiation par l'orchestrateur de la fiche (Opus 5.5),
  prémisses relues au code :
  - **R1 = F1 (MEDIUM)** — appliqué : M12b n'avait pas de témoin — les deux requêtes rognent elles-mêmes la recherche
    (`kesh-db/src/repositories/invoices.rs:428`, `:986`), si bien que seule la longueur comparée à `RECHERCHE_MAX` voit
    le rognage du service. AC11 (a) : `" " + "a"×100 + " "` (102 bruts, 100 rognés) accepté, pour la liste et pour
    l'échéancier, nommé témoin de M12b à l'AC15.
  - **F5** — décision de l'orchestrateur : la 32-4b fusionne avant la 32-5b ; la 32-5b compte relativement à `main` et
    recompte à son étape 0 (C-32-4b-18, *Risques*, T0).
  - **LOW traités** : R2 = F6 (bornes `lib.rs:147-149`, `:158-160`, `lecture_seule.rs:8-9`, et la l. 8 au contrôle), R3
    (motif resserré aux formules périmées : dix lignes aujourd'hui, la citation de la décision permise), R4
    (`HANDLERS_MIGRES` l. 2124-2131), R5 (bornes acceptées de `dueBefore` précisées), F2 (annotation de l'epic dite
    faite), F3 (aucun texte neuf ne nomme `build_due_dates_query`), F4 (longueurs de tableau et doc-comment de `Cas`),
    F7 (quatre factures **validées** pour le tri de l'échéancier), F8 (TTC multiple de 0.05, arrondi du seed).
  - **Recyclage** : **non** pour le MEDIUM : M12b est un défaut d'**origine** de la
    fiche (rédaction du 2026-10-25, AC15), que la P1 n'avait pas vu, non une régression de la remédiation P1 ; il est
    distinct des cinq MEDIUM de la P1. En revanche R2 = F6, R3, F2 et F3 portent sur des textes **écrits par la
    remédiation P1** (bornes et motif de l'AC17 (3), *Fichiers touchés*, contrôle de l'AC8) : la sévérité s'est
    déplacée vers ce qu'on venait d'écrire, au niveau LOW. Pas de découpage : sévérité décroissante (5 MEDIUM → 1), le
    MEDIUM neuf est d'origine (amendement D5).
  - **Propagation** (grep des valeurs) : `2124-2133` → deux sites, traités ; `147-148`, `158-159` → fait 10, AC17 (3),
    *Fichiers touchés*, traités (l'entrée « validation P1 » du Change Log, relevé daté, garde ses bornes) ; `l. 9` de
    `lecture_seule.rs` → AC9 (4), AC17 (3), traités ; `neuf lignes` → AC17 (3) et fait 10 (« dix »), l'entrée P1 restant
    datée ; le motif du contrôle Q11 → ses deux sites (fait 10, AC17 (3)) ; `C-32-4b-1 à 17` → `1 à 18` (en-tête,
    `sprint-status.yaml`).
  - **Trend** : P1 0/0/5/14 (Sonnet ×2) → P2 0/0/1/10 (Opus ×2). La remédiation ne touche que la fiche et le registre :
    **P3 ciblée**, une lentille (regression hunter), **Sonnet**, sur le seul commit de cette remédiation
    (`324b-validate-prompt-p3.md`).
- **2026-10-25 — Validation P3, passe ciblée — VALIDATION TERMINÉE** (Sonnet, une lentille — regression hunter —,
  contexte frais, lecture seule, commit `d7783685`, prompt `324b-validate-prompt-p3.md`, rapport
  `kesh-gate-logs/324b-validate-p3.md`) : **0/0/0/1**. Le témoin de M12b, le contrôle Q11 (dix lignes), les formes
  de `dueBefore`, F3, F4, F7, F8 et l'ordre de fusion (C-32-4b-18) tiennent au code. **P3-1** (LOW : à l'AC9 (2), le
  doc-comment de `Cas::Invalide`, `services_rollout.rs:402`, ne nomme pas encore `limit = Some(0)` — lire « qui devra
  nommer aussi ») est **renvoyé à l'étape 0 du développement**, ligne lue citée au Change Log : une remédiation après
  la dernière passe ne serait relue par personne. **Trend** : P1 0/0/5/14 (Sonnet ×2) → P2 0/0/1/10 (Opus ×2) → P3
  0/0/0/1 (Sonnet). Prête pour le développement.
