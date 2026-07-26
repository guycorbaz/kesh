# Story 16.1a-bis : Backfill du compte de produit sur le parc de factures existant

## Status

ready-for-dev

## Story

**As a** utilisateur de Kesh dont l'instance contient déjà des factures validées (dont l'instance de production en fonction),
**I want** que la colonne `revenue_account_id` introduite par 16-1a soit **renseignée rétroactivement** sur mes factures et avoirs déjà validés, à partir de l'écriture comptable qu'ils ont réellement produite,
**so that** un avoir émis plus tard **extourne le compte effectivement crédité** par la facture d'origine, et non le compte de produit par défaut tel qu'il se trouve être configuré ce jour-là — ce qui laisserait un résidu permanent, invisible au bilan et faux au compte de résultat.

Issue : **#152**. Rattaché au CR **#265**. Sous-story de l'Epic 16 « Facturation avancée ».

**Dépend strictement de 16-1a** : la colonne `invoice_lines.revenue_account_id` et son pendant `credit_note_lines` sont créés là-bas. Cette story ne fait que les **remplir pour le passé**.

### Provenance du split (passe 6 de `validate` sur 16-1a)

Le backfill était initialement la décision **D2-bis** de 16-1a, née d'un finding CRITICAL de la passe 4. Sur les passes 5 et 6 de `bmad-create-story validate`, **7 findings sur 10 — dont les 3 HIGH — portaient sur lui seul**, tandis que le reste de 16-1a (schéma, moteur de ventilation, avoir, validation) n'était plus remis en cause depuis la passe 3. La sévérité de 16-1a a cessé de décroître (P5 : 1 HIGH → P6 : 2 HIGH), cochant le second critère de la § « Règle de splitting préventif » de `CLAUDE.md`.

**Arbitrage de Guy, 2026-07-26 : extraction en story dédiée.** Motif : le backfill a un **profil de risque étranger** au reste de 16-1a — c'est une migration de **données comptables réelles**, one-shot, non rejouable au sens métier, essentiellement du SQL, alors que 16-1a est du schéma et de la logique applicative testable unitairement. Les deux ne se revoient pas avec le même regard, et le backfill saturait les passes adversariales de 16-1a au point de masquer d'éventuels défauts ailleurs.

**Le split n'ouvre aucune fenêtre de corruption** : sans cette story, le parc antérieur conserve **exactement** le comportement d'aujourd'hui (repli sur le défaut courant au moment de l'avoir). Le backfill est **strictement additif** — il ferme un bug pré-existant, il n'en introduit aucun. C'est ce qui rend le split sûr, et c'est la différence avec le split 16-1a / 16-1b, où D5 devait impérativement rester avec sa cause.

---

## Contexte

### Le bug fermé par cette story

`settings.default_revenue_account_id` est **mutable** par l'utilisateur dans les Réglages (`company_invoice_settings.rs:173`). Pour une ligne de facture sans compte (`NULL`), la chaîne est : ligne `NULL` → snapshot d'avoir `NULL` → repli résolu **au moment de l'avoir**, sur la configuration relue à T2 (`credit_notes.rs:275-282`). D'où :

| T | Événement | Écriture |
|---|---|---|
| T1 | Facture validée, ligne `NULL`, défaut = 3000 | **crédit 3000** = HT |
| T1+ | L'administrateur change le défaut → 3200 | — |
| T2 | Avoir total émis | **débit 3200** = HT |

Résidu permanent au crédit de 3000 et au débit de 3200. **Bilan équilibré, compte de résultat faux, aucun signal.**

16-1a ferme ce trou pour les factures **futures**, en matérialisant le compte effectif dans `invoice_lines` à la validation (sa décision D2). Mais cette matérialisation ne se déclenche qu'à la transition `draft → validated` : une facture **déjà validée** n'y repasse **jamais** (`update` rejette tout statut ≠ `draft`, `invoices.rs:841`, `:1271`) et aucune autre écriture ne touche `invoice_lines` après validation. Après l'`ADD COLUMN` de 16-1a, ces lignes portent `NULL` **définitivement**.

Or `NULL` est **le seul cas qui existe en production aujourd'hui** — aucune facture existante ne porte de compte de ligne. **16-1a seule protège donc un ensemble vide à l'instant du déploiement**, et 100 % du parc validé reste exposé. C'est la raison d'être de cette story.

### Ce qui existe aujourd'hui

- **`invoices.journal_entry_id`** — lien facture → écriture, FK `ON DELETE RESTRICT` (`migrations/20260417000001_invoice_validation.sql:54-56`). Une facture `validated` a **toujours** une écriture : `chk_invoices_validated_has_je` (`migrations/20260417000002_invoice_validated_journal_entry_check.sql`) l'impose, et la FK `RESTRICT` interdit de supprimer l'écriture après coup.
- **`credit_notes.journal_entry_id`** — idem côté avoir, `CHECK (status <> 'issued' OR (credit_note_number IS NOT NULL AND journal_entry_id IS NOT NULL))` (`20260627000001_credit_notes.sql:54`).
- **Structure produite par `generate_invoice_journal_lines`** (`invoices.rs:1156`, doc `:1120-1155`) : `[0]` débit créance = TTC, `[1]` **crédit produit unique** = `total_ht`, `[2..N]` crédit TVA due par taux.
- **`journal_entry_lines.line_order`** — `INT NOT NULL`, `UNIQUE (entry_id, line_order)`, assigné séquentiellement `(idx as i32) + 1` à l'insertion (`journal_entries.rs:272`).
- **`generate_credit_note_journal_lines`** (`credit_notes.rs:139`) — « inverse exact » : la contre-passation débite ce que la facture a crédité.
- **Précédents de backfill en migration** : `20260628000001_supplier_invoices.sql:115` (`UPDATE … INNER JOIN accounts … WHERE … IS NULL`, multi-table) et `20260722000001_accounts_role_postable.sql` (**12 `UPDATE` de backfill**). Les deux sont classés `tracked-by-sqlx` et décrits comme **intrinsèquement idempotents** dans `docs/migrations-idempotence-audit.md`.

### Ce qui n'est PAS dans cette story

- La colonne elle-même, l'entité, l'API, le moteur de ventilation, la matérialisation à la validation, les avoirs futurs → **16-1a**.
- Toute la surface utilisateur → **16-1b**.
- La **réconciliation facture ↔ écriture après édition manuelle de l'écriture** (cf. D-B2) → hors périmètre, et hors périmètre de l'Epic 16.

---

## Décisions de conception

### D-B1 — La source de vérité est l'écriture générée, JAMAIS le défaut courant

**Décision** : le compte backfillé est celui que l'écriture comptable de la pièce a **effectivement** crédité (facture) ou débité (avoir).

**Pourquoi pas le défaut courant** : si l'administrateur a déjà changé `default_revenue_account_id` par le passé, backfiller avec la valeur d'aujourd'hui écrirait un compte que la facture n'a **jamais** crédité — on **fabriquerait** la corruption au lieu de la fermer. L'écriture générée est la **pièce probante**, et c'est l'argument même de la décision D2 de 16-1a.

### D-B2 — La pièce probante n'est pas garantie intacte : le backfill doit être CONSERVATEUR

**C'est la décision structurante de cette story, et l'origine du split.**

La structure `[0]` créance / `[1]` produit / `[2..]` TVA décrit ce que `generate_invoice_journal_lines` **produit**, pas ce que la table **contient** au moment de la migration.

**Ground-truth (vérifié 2026-07-26)** : l'écriture d'une facture validée est **éditable par l'utilisateur**.

- `journal_entries` ne porte **aucune** colonne `source` / `origin` / `is_auto` — une écriture générée par une validation de facture est **indiscernable** d'une écriture manuelle (schéma complet `migrations/20260412000001_journal_entries.sql:17-36`) ;
- `PUT /api/v1/journal-entries/{id}` est exposée (`kesh-api/src/lib.rs:303` → `routes/journal_entries.rs:517` → `repositories/journal_entries.rs:805`) et n'a **aucune garde de provenance** : ses seules gardes sont l'exercice clos, le verrou optimiste, l'équilibre en partie double et la validité des comptes ;
- `journal_entries::update` fait `DELETE FROM journal_entry_lines WHERE entry_id = ?` puis ré-INSERT en **réattribuant les `line_order` de zéro** (`:1005`).

Sur une base réelle, l'écriture d'une facture peut donc porter ses lignes dans un autre ordre, sur d'autres comptes, en nombre différent. Les deux méthodes d'identification naïves échouent **silencieusement** :

- **positionnelle** (`line_order = 2`) — après ré-INSERT, la 2ᵉ ligne peut être n'importe quoi, y compris une ligne de TVA ou une charge ;
- **par élimination** (tout ce qui n'est ni la créance ni la TVA) — une écriture éditée peut en laisser **zéro** ou **plusieurs**.

Et l'erreur n'est pas neutre : un mauvais compte écrit dans `invoice_lines` devient **la** vérité de la facture, que 16-1a D5 recopiera dans tout avoir futur. **Un backfill approximatif fabrique exactement la corruption qu'il existe pour fermer**, en lui donnant en plus l'apparence d'une donnée établie. Sur les écritures non éditées — la quasi-totalité du parc — l'identification reste parfaitement fiable : le défaut est le manque de **discrimination**, pas la méthode.

**Décision — règle du backfill** : ne backfiller **que** si la ligne de produit est identifiable **sans ambiguïté**, c'est-à-dire s'il existe dans l'écriture **exactement une** ligne remplissant les trois conditions :

1. `credit > 0` (donc `debit = 0`, garanti exclusif par `chk_jel_debit_credit_exclusive`) ;
2. son `account_id` n'appartient pas à l'ensemble d'exclusion `E = { cis.default_receivable_account_id, cis.default_vat_payable_account_id }`, **les valeurs `NULL` étant ignorées** (une colonne de configuration non renseignée n'exclut rien) ;
3. son `credit` est **égal à `invoices.total_amount`** (le HT, `Σ line_total` — cf. docstring `invoices.rs:1128`, qui écrit littéralement « `total_ht` (HT, = `invoices.total_amount`, DC9) »). **Il n'existe pas de colonne `total_ht`** ; ne pas la chercher.

La condition (3) est le **discriminant** que ni la position ni l'élimination ne donnent : c'est elle qui distingue une écriture canonique d'une écriture retouchée. Zéro candidat, ou plusieurs → la ligne reste `NULL` et est **dénombrable** (D-B4).

**Miroir avoir** : `debit` au lieu de `credit`, et `credit_notes.total_amount` au lieu d'`invoices.total_amount`.

*(Redondance assumée : l'exclusion de la créance en (2) est inoffensive mais inutile — sur l'écriture facture la créance est un **débit**, déjà éliminée par (1) ; sur l'écriture d'avoir elle est un crédit, éliminée par le (1) miroir. Seule l'exclusion TVA porte réellement. Conservée pour lisibilité et défense en profondeur.)*

### D-B3 — La condition (2) DOIT être écrite en SQL NULL-safe : sinon le backfill no-ope sur 100 % du parc

**Piège dirimant, et le plus vicieux du dossier.** Les colonnes de configuration sont toutes nullables, et les **trois colonnes TVA sont `NULL` sur toute installation** :

- l'`INSERT` d'onboarding ne les énumère pas (`company_invoice_settings.rs:452-458` : seulement `invoice_number_format`, `default_receivable_account_id`, `default_revenue_account_id`, `default_payable_account_id`, `default_sales_journal`, `journal_entry_description_template`) ;
- le lazy-create insère `(company_id)` seul (`:92`, `INSERT IGNORE INTO company_invoice_settings (company_id) VALUES (?)`) ;
- aucune migration ne les renseigne (`grep -rn "SET default_vat" crates/kesh-db/migrations/` → **vide** ; `20260614000001_vat_accounts_config.sql` crée bien les comptes `1171` / `2206` mais ne pointe jamais les colonnes de config dessus).

Écrite naïvement `jel.account_id <> cis.default_vat_payable_account_id`, la comparaison à `NULL` rend le prédicat `NULL` en logique ternaire SQL, la ligne n'est **jamais** candidate, et le backfill **no-ope intégralement** — migration en succès, décompte « très élevé ».

**Ce qui rend ce mode de défaillance particulièrement dangereux** : la présente spec **pré-autorise** explicitement un décompte élevé comme un comportement conservateur normal (D-B2). Le bug est donc **rigoureusement indiscernable du succès** sans un test dédié.

**Décision** : utiliser l'opérateur NULL-safe de MariaDB.

```sql
NOT (jel.account_id <=> cis.default_receivable_account_id)
AND NOT (jel.account_id <=> cis.default_vat_payable_account_id)
```

**Jamais** `<>` / `!=` / `NOT IN`, qui propagent `NULL`.

**Pourquoi `E` ne contient que ces deux comptes** : `default_vat_recoverable_account_id` et `default_vat_decompte_account_id` n'apparaissent **jamais** dans une écriture de vente — `generate_invoice_journal_lines` ne reçoit que le compte de TVA due (`invoices.rs:1387`). Les inclure n'ajoute rien et multiplie les occasions de propager un `NULL`.

### D-B4 — Le décompte des lignes non backfillées : aucun artefact nouveau

Une migration sqlx est un fichier `.sql` pur exécuté par `MIGRATOR.run()` (`kesh-db/src/lib.rs:23`, appelé depuis `kesh-api/src/main.rs:138`) : elle n'a **aucun canal de restitution**. Deux options ont été envisagées et **écartées** :

- *table de rapport* — **refusée** : toute nouvelle table fait tomber `backup_inventory_matches_schema` (`backup.rs:577-606`), impose de mettre à jour `TABLES_TO_TRUNCATE`, et fait donc entrer la table dans le périmètre de l'**export/import d'installation** — décision d'architecture qui serait prise par accident, pour faire passer un test ;
- *log applicatif au démarrage* — **refusée** : ajoute dans `main.rs` une requête de comptage rejouée à chaque boot indéfiniment, pour un besoin ponctuel de déploiement.

**Décision** : le décompte est obtenu par une **requête de diagnostic documentée**, exécutable à tout moment, sans état persistant :

```sql
-- Factures
SELECT COUNT(*) FROM invoice_lines il JOIN invoices i ON i.id = il.invoice_id
WHERE i.status = 'validated' AND il.revenue_account_id IS NULL;
-- Avoirs
SELECT COUNT(*) FROM credit_note_lines cnl JOIN credit_notes cn ON cn.id = cnl.credit_note_id
WHERE cn.status = 'issued' AND cnl.revenue_account_id IS NULL;
```

Elle est consignée au CHANGELOG au titre des notes de déploiement, et c'est **elle** que les tests assertent. **Aucune table, aucun log, aucune ligne dans `main.rs`.**

### D-B5 — Portée : `validated` / `issued` seulement ; les `cancelled` sont exclues délibérément

**Périmètre** : `invoice_lines` des factures `status = 'validated'` **et** `journal_entry_id IS NOT NULL` (ce second prédicat est une garde défensive **redondante** avec `chk_invoices_validated_has_je`), plus le miroir sur `credit_note_lines` des avoirs `issued`.

Les factures `draft` restent `NULL` — c'est le sens même de la liaison tardive (16-1a D2).

Les factures `cancelled` sont exclues **délibérément**, et **non** « parce qu'elles n'auraient pas d'écriture » — elles en ont toujours une. Le seul chemin qui produit ce statut est l'émission d'un avoir (`credit_notes.rs:398`, `UPDATE invoices SET status = 'cancelled' … AND status = 'validated'`). Le motif réel est que `uq_credit_notes_invoice UNIQUE (invoice_id)` (`20260627000001_credit_notes.sql:58`) interdit un **second** avoir : **aucun résidu futur n'est possible**, il n'y a rien à prévenir.

**Conséquence assumée** : sur une facture créditée, `credit_note_lines.revenue_account_id` sera renseigné alors que `invoice_lines.revenue_account_id` restera `NULL`. C'est visible dans l'export CSV (16-1a AC14) et **sans effet comptable**.

### D-B6 — Migration non-breaking et backfill intrinsèquement idempotent

Le backfill est un `UPDATE` gardé par `revenue_account_id IS NULL` et fondé sur un critère **déterministe** : un re-jeu recalcule le même résultat et n'a aucun effet. Il est donc **intrinsèquement idempotent**, exactement comme les backfills de `20260628000001_supplier_invoices.sql` et `20260722000001_accounts_role_postable.sql` (« **12 UPDATE de backfill** en revanche intrinsèquement idempotents »), tous deux classés `tracked-by-sqlx`.

**Ne pas** qualifier la migration de « non idempotente » : `docs/migrations-idempotence-audit.md` maintient l'invariant **« Idempotence `no` : 0 »** (`:71`), et un verdict `no` ferait diverger les compteurs.

Aucune opération `DROP` / `RENAME` / `MODIFY COLUMN` → migration **non-breaking** au sens P1 de `CLAUDE.md` → **pas** de bump `kesh_version_min_required`, donc **pas** de bump de version Cargo (P2-bis).

---

## Acceptance Criteria

- **AC-B1** — Migration de backfill : `invoice_lines.revenue_account_id` des factures `status = 'validated' AND journal_entry_id IS NOT NULL` reçoit le compte identifié par le **critère d'unicité en trois conditions** de D-B2 ; miroir sur `credit_note_lines` des avoirs `issued` (`debit`, `credit_notes.total_amount`).
  - **Ne PAS** backfiller depuis `settings.default_revenue_account_id` courant (D-B1).
  - **Ne PAS** identifier la ligne par `line_order` seul (D-B2).
  - **Ne PAS** écrire la condition (2) avec `<>` / `!=` / `NOT IN` (D-B3).
- **AC-B2** — **Le backfill est délibérément incomplet, et c'est la spécification.** Toute ligne dont le compte n'est pas identifiable sans ambiguïté — écriture éditée, zéro ou plusieurs candidats — **reste `NULL`**, et la migration **réussit**. Une post-condition « aucune ligne validée ne reste `NULL` » serait **fausse** et pousserait le dev à relâcher le critère jusqu'à ce qu'elle passe, c'est-à-dire à écrire un compte arbitraire sur des données comptables réelles — l'inverse exact de l'objectif.
- **AC-B3** — Post-conditions testées, **sur base pré-remplie** (`migrations_fresh_install` ne prouve rien ici : il n'y a rien à backfiller sur une base vierge) :
  1. facture validée à écriture **canonique** → ligne backfillée avec le compte crédité par l'écriture, **même si `settings.default_revenue_account_id` a changé depuis**. Sans ce changement de défaut, le test passerait aussi avec un backfill depuis le défaut courant et ne prouverait rien (D-B1) ;
  2. facture validée dont l'écriture a été **éditée** de sorte qu'aucune ligne ne crédite exactement `total_amount` → la ligne reste `NULL`, la migration **réussit**, et la requête de diagnostic de D-B4 retourne le compte attendu ;
  3. **société dont `default_vat_payable_account_id` est `NULL`** — c'est-à-dire **toute** société non configurée manuellement, le cas par défaut — facture validée à écriture canonique → la ligne **est** backfillée. **C'est le seul test qui attrape la propagation `NULL` de D-B3** ; sans lui, un backfill qui no-ope intégralement est indiscernable d'un backfill conservateur ;
  4. facture `draft` → reste `NULL` ; facture `cancelled` → reste `NULL` (D-B5) ;
  5. miroir avoir : avoir `issued` à écriture canonique → ligne backfillée ; le compte backfillé de l'avoir **égale** celui de la facture d'origine quand les deux sont identifiables.
  *(Le cas « facture validée **sans écriture** » n'est **PAS** testé : il est **inconstructible** — `chk_invoices_validated_has_je` l'interdit en base et `fk_invoices_journal_entry … ON DELETE RESTRICT` empêche de supprimer l'écriture après coup. Tenter la fixture ne produit qu'une violation de CHECK.)*
- **AC-B4** — **Idempotence** : rejouer le backfill sur une base déjà backfillée ne change **aucune** ligne (garde `IS NULL` + critère déterministe). Testé.
- **AC-B5** — `docs/migrations-idempotence-audit.md` : ligne ajoutée au tableau détaillé avec verdict **`tracked-by-sqlx`** (justifié par l'absence d'`IF NOT EXISTS`, **pas** par le backfill, qui est idempotent — D-B6), **ET** récapitulatif agrégé de bas de fichier mis à jour en cohérence (`Total` et `Idempotence tracked-by-sqlx` chacun +1). L'invariant « Idempotence `no` : 0 » est **préservé**. Garde-fou **P5** de `CLAUDE.md`.
- **AC-B6** — Migration **non-breaking** → **pas** de bump `kesh_version_min_required`, donc **pas** de bump de version Cargo (P1/P2/P2-bis). Le vérifier explicitement.
- **AC-B7** — **Aucune table n'est créée par cette story** (D-B4). Corollaire vérifié : `backup_inventory_matches_schema` (`backup.rs:577-606`) reste vert sans toucher `TABLES_TO_TRUNCATE`, et les compteurs d'export (`exports_global_e2e.rs:621` 20 entrées, `:633`) sont inchangés.
- **AC-B8** — CHANGELOG `[Non publié]` : entrée orientée utilisateur **avec les requêtes de diagnostic de D-B4**, au titre des notes de déploiement, et mention explicite que les factures dont l'écriture a été modifiée manuellement ne sont pas reprises.
- **AC-B9** — Gate « Test Locally First » backend complet vert (`cargo fmt --all -- --check`, `cargo build --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`). Les suites `migrations_fresh_install` et `admin_backup_e2e` doivent passer — cette story touche une migration.

---

## Tasks / Subtasks

- [ ] **T-B1** — Migration de backfill `invoice_lines` : `UPDATE … JOIN (SELECT … GROUP BY … HAVING COUNT(*) = 1) c`, critère de D-B2, condition (2) en `<=>` NULL-safe (D-B3).
- [ ] **T-B2** — Miroir `credit_note_lines` (`debit`, `credit_notes.total_amount`), même critère.
- [ ] **T-B3** — Ligne du tableau **et** compteurs agrégés de `docs/migrations-idempotence-audit.md`, verdict `tracked-by-sqlx` + justification d'idempotence (AC-B5, D-B6).
- [ ] **T-B4** — Tests sur base pré-remplie : les 5 cas d'AC-B3 + l'idempotence d'AC-B4.
- [ ] **T-B5** — CHANGELOG avec requêtes de diagnostic (AC-B8) + gate backend complet (AC-B9).

**Ordre conseillé** : T-B1 → T-B4 (partiel, cas facture) → T-B2 → T-B4 (complet) → T-B3 → T-B5.

---

## Dev Notes

### Ancres ground-truth (vérifiées 2026-07-26, passes 5 et 6 de `validate` sur 16-1a)

| Élément | Emplacement |
|---|---|
| Lien facture → écriture (FK RESTRICT) | `crates/kesh-db/migrations/20260417000001_invoice_validation.sql:54-56` |
| **`CHECK` : facture `validated` ⇒ écriture non-NULL** | `crates/kesh-db/migrations/20260417000002_invoice_validated_journal_entry_check.sql` |
| Lien avoir → écriture + CHECK `issued` | `crates/kesh-db/migrations/20260627000001_credit_notes.sql:36`, `:47-48`, `:54` |
| `uq_credit_notes_invoice` (un seul avoir par facture) | `20260627000001_credit_notes.sql:58` |
| Schéma `journal_entries` / `journal_entry_lines` — **aucune colonne `source`** | `crates/kesh-db/migrations/20260412000001_journal_entries.sql:17-36`, lignes `:38-56` |
| `chk_jel_debit_credit_exclusive` | `20260412000001_journal_entries.sql:46` |
| **`line_order` réattribué à chaque `update`** | `crates/kesh-db/src/repositories/journal_entries.rs:1005` ; création `:272` |
| **`PUT /journal-entries/{id}` — aucune garde de provenance** | `crates/kesh-api/src/lib.rs:303` → `routes/journal_entries.rs:517` → `repositories/journal_entries.rs:805` |
| Structure de l'écriture facture (doc + fn) | `crates/kesh-db/src/repositories/invoices.rs:1120-1155` (doc), `:1156` (fn) |
| `total_ht` = `invoices.total_amount` (écrit littéralement) | `invoices.rs:1128` |
| `invoices.total_amount DECIMAL(19,4)` | `20260416000001_invoices.sql:24` |
| Compte de TVA due passé au helper | `invoices.rs:1387` |
| Helper avoir « inverse exact » | `crates/kesh-db/src/repositories/credit_notes.rs:139` |
| Bascule facture → `cancelled` (seul chemin) | `credit_notes.rs:398` |
| **Colonnes TVA de config — nullables** | `crates/kesh-db/migrations/20260614000001_vat_accounts_config.sql:54-56` |
| **`INSERT` onboarding — n'énumère PAS les colonnes TVA** | `crates/kesh-db/src/repositories/company_invoice_settings.rs:452-458` |
| **Lazy-create `(company_id)` seul** | `company_invoice_settings.rs:92` |
| Défaut société mutable par l'utilisateur | `company_invoice_settings.rs:173` |
| Précédent backfill multi-table en migration | `crates/kesh-db/migrations/20260628000001_supplier_invoices.sql:115` |
| Précédent 12 `UPDATE` de backfill idempotents | `20260722000001_accounts_role_postable.sql` ; audit `docs/migrations-idempotence-audit.md` |
| Invariant « Idempotence `no` : 0 » | `docs/migrations-idempotence-audit.md:71` (compteurs `:68-70`) |
| `MIGRATOR` et son appel | `crates/kesh-db/src/lib.rs:23` ; `crates/kesh-api/src/main.rs:138` |
| `backup_inventory_matches_schema` | `crates/kesh-db/src/backup.rs:577-606` |

### Pièges, par ordre de coût

0. **La propagation `NULL` de la condition (2) (D-B3)** — le pire, parce qu'il est **indiscernable du succès** : la migration passe, le décompte est élevé, et la spec autorise explicitement un décompte élevé. Sans le cas 3 d'AC-B3, personne ne le verra jamais. Écrire `<=>`, pas `<>`.
1. **Le backfill trop confiant (D-B2)** — écrire un compte **faux** sur des factures validées réelles, que 16-1a D5 recopiera ensuite dans tout avoir futur. L'identification positionnelle paraît solide parce que le **code de génération** est déterministe — mais l'écriture est **éditable après coup**. Le discriminant obligatoire est le montant égal à `invoices.total_amount`.
2. **La post-condition « plus aucun `NULL` »** — elle est **fausse** (AC-B2) et un dev qui cherche à la faire passer ira mécaniquement écrire un compte arbitraire. Ne jamais tester par un `COUNT(*) = 0` global.
3. **`total_ht` n'est pas une colonne** — c'est `invoices.total_amount` (et `credit_notes.total_amount` côté avoir).
4. **Le verdict d'idempotence (D-B6)** — écrire « non idempotente » casse l'invariant `no : 0` du fichier d'audit et fait diverger les compteurs d'AC-B5. Le backfill **est** idempotent ; c'est l'absence d'`IF NOT EXISTS` qui justifie `tracked-by-sqlx`.
5. **Base vierge** — `migrations_fresh_install` ne prouve **rien** ici. Tous les tests d'AC-B3 exigent une base pré-remplie.

### Faisabilité SQL (vérifiée)

- **MariaDB 10.11** (`docker-compose.yml:4`, `.github/workflows/ci.yml:34`).
- La restriction ER 1093 (« can't specify target table for update in FROM clause ») **ne s'applique pas** : la sous-requête de candidats lit `journal_entry_lines` / `invoices` / `company_invoice_settings`, jamais la table cible `invoice_lines` (miroir : jamais `credit_note_lines`).
- Précédent direct de multi-table `UPDATE` en migration : `20260628000001_supplier_invoices.sql:115`.
- sqlx exécute bien plusieurs instructions par fichier (`20260627000001_credit_notes.sql` en contient 5).
- **Ne pas utiliser de CTE (`WITH`)** — aucune migration du dépôt n'en contient (`grep -n "^WITH " crates/kesh-db/migrations/*.sql` → vide). Préférer `UPDATE … JOIN (SELECT … GROUP BY … HAVING COUNT(*) = 1) c`, qui a un précédent.

### Sûreté numérique de la condition (3) — vérifiée, ne pas ré-instruire

`journal_entry_lines.debit` / `credit` et `invoices.total_amount` / `credit_notes.total_amount` sont **tous** `DECIMAL(19,4)` — exact en MariaDB, même échelle des deux côtés, aucune comparaison flottante. Les deux valeurs sont identiques **par construction** : `generate_invoice_journal_lines` pousse `credit: total_ht = Σ line_total` (`invoices.rs:1178`, `:1198`), tandis qu'`invoices.total_amount` vient de `compute_total` = `Σ compute_line_total` (`:379-383`), la **même** fonction qui écrit `line_total` (`:393`). Une facture validée étant immuable, les deux ne peuvent pas diverger après coup.

**Aucune facture ancienne n'a de structure incompatible** : la seule version antérieure du helper (remplacée par l'Epic 18, commit `654dba7d`) produisait 2 lignes avec `credit = invoice_before.total_amount` — la condition (3) y est vraie *a fortiori*.

### Propagation post-patch (§ CLAUDE.md)

Après chaque patch de remédiation, **grep le symptôme sur tout le dépôt** avant la passe suivante. Symptômes propres à cette story : `revenue_account_id`, `total_ht`, `line_order`, `default_vat_payable`, `<=>`, `IS NULL`, « idempotent », les compteurs de `migrations-idempotence-audit.md`. **Greper aussi 16-1a et 16-1b** : les trois stories partagent le même vocabulaire et une correction ici a souvent un jumeau là-bas.

### References

- Issue **#152**, CR **#265**.
- Story **16-1a** — socle backend dont dépend celle-ci (colonne, entité, moteur, matérialisation D2).
- Story **16-1b** — surface utilisateur ; son AC6-bis doit tolérer un `revenue_account_id` `NULL` côté avoir pour les pièces antérieures non backfillables.
- Stories antérieures : **12-1** (avoirs et contre-passation), **18-1b** (helper d'écriture facture), **14-3a** (précédent de 12 `UPDATE` de backfill en migration), **12-2** (précédent de backfill multi-table).
- `CLAUDE.md` : politique de migration (P1-P5), Review Iteration Rule, propagation post-patch, règle de splitting préventif.

---

## Change Log

### Création par split de 16-1a — 2026-07-26 (passe 6 de `validate`)

Story issue de l'extraction de la décision **D2-bis** de 16-1a et de tout son corpus (critère de backfill, AC2-bis, volet backfill de T1, piège n°0), sur arbitrage de Guy après déclenchement du second critère de la § « Règle de splitting préventif » sur 16-1a (P5 : 1 HIGH → P6 : 2 HIGH, sévérité non décroissante).

**Motif** : sur les passes 5 et 6, **7 findings sur 10 — dont les 3 HIGH — portaient sur le seul backfill**, alors que le reste de 16-1a n'était plus remis en cause depuis la passe 3. Profil de risque étranger (migration de données comptables réelles vs schéma et logique applicative), et saturation des passes adversariales de 16-1a.

**Le contenu est repris dans l'état convergé des passes 4 à 6**, sans régression :

| Origine | Apport |
|---|---|
| P4 (CRITICAL) | Existence même du backfill — 16-1a seule protège un ensemble **vide** à l'instant du déploiement |
| P4 | Source = l'écriture générée, **jamais** le défaut courant (D-B1) |
| P5 (HIGH) | L'écriture est **éditable** (aucune colonne `source`, `PUT` sans garde de provenance, `line_order` réattribué) → critère d'unicité conservateur avec le **montant** pour discriminant (D-B2) |
| P5 | Suppression de la post-condition « plus aucun `NULL` », **fausse** et pousse-au-crime (AC-B2) |
| P6 (HIGH) | Condition (2) en **`<=>` NULL-safe** — les colonnes TVA sont `NULL` partout, un `<>` fait no-oper le backfill sur 100 % du parc de façon indiscernable du succès (D-B3) |
| P6 (MEDIUM) | Décompte tranché : **aucun artefact nouveau**, requête de diagnostic (D-B4) |
| P6 (MEDIUM) | Cas de test « facture validée sans écriture » retiré — **inconstructible** (`chk_invoices_validated_has_je`) |
| P6 (LOW) | `total_ht` → `invoices.total_amount` ; motif d'exclusion des `cancelled` corrigé (D-B5) ; verdict d'idempotence redressé (D-B6) |

**Statut de revue** : le corpus a été revu en passes 5 (Haiku + orchestrateur) et 6 (Opus) **au sein de 16-1a**. En tant que story autonome, il n'a **jamais** été revu comme un tout cohérent — notamment son en-tête, sa portée, ses AC renumérotés et le miroir avoir, désormais traité au même rang que la facture et non comme un corollaire. **Passe 1 de `validate` requise sur cette story**, contexte frais.
