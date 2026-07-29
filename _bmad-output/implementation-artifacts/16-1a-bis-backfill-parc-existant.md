# Story 16.1a-bis : Backfill du compte de produit sur le parc de factures existant

## Status

review

## Story

**As a** utilisateur de Kesh dont l'instance contient déjà des factures validées (dont l'instance de production en fonction),
**I want** que la colonne `revenue_account_id` introduite par 16-1a soit **renseignée rétroactivement** sur mes factures et avoirs déjà validés, à partir de l'écriture comptable qu'ils ont réellement produite,
**so that** un avoir **émis à partir de maintenant** sur une facture ancienne **extourne le compte effectivement crédité** par cette facture, et non le compte de produit par défaut tel qu'il se trouve être configuré ce jour-là — ce qui laisserait un résidu permanent, invisible au bilan et faux au compte de résultat.

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

**Quelle population cette story protège exactement — à lire avec D-B7.** Le tableau ci-dessus se lit dans **deux temps distincts**, et la story n'agit que sur l'un des deux :

| Situation au moment du déploiement | Ce que fait cette story |
|---|---|
| Facture validée, **avoir pas encore émis** (T2 dans le futur) — **le cas de l'immense majorité des factures** | **Le bug est fermé.** La ligne est backfillée à 3000 ; l'avoir futur, qui copiera le compte depuis `invoice_lines` (16-1a D5) au lieu de relire la configuration, débitera 3000. Aucun résidu. |
| Facture validée **et avoir déjà émis** (T2 déjà passé) | **Rien n'est réparé** — l'écriture d'avoir existe, elle a débité 3200, c'est irréversible. Le backfill se contente d'**enregistrer fidèlement** les deux comptes réellement mouvementés (3000 et 3200), qui **diffèrent légitimement**. Cf. **D-B7**. |

Autrement dit, le backfill est **préventif**, pas curatif : il arme les factures passées pour que leurs avoirs **à venir** soient corrects. Corriger un couple déjà soldé exigerait une écriture de reclassement — un **acte comptable de l'utilisateur**, hors de portée d'une migration.

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

### D-B7 — Le backfill ENREGISTRE le résidu historique, il ne le RÉPARE pas

**Décision, et clarification cardinale du périmètre.** Pour un couple facture / avoir **antérieur au déploiement**, les deux comptes backfillés peuvent légitimement **différer** — et le backfill doit les écrire tels quels.

**Preuve du mécanisme** : `create_credit_note` relit la configuration **au moment de l'émission de l'avoir** — `company_invoice_settings::get_or_create_default_in_tx` puis `settings.default_revenue_account_id` (`credit_notes.rs:275-282`) — indépendamment de ce que la facture a réellement crédité. C'est **le bug décrit dans le § Contexte** : facture créditée sur 3000 à T1, avoir débité sur 3200 à T2 après changement du défaut.

Les avoirs déjà en base ont été produits par ce code. Leur écriture de contre-passation porte donc, dans ce cas, un compte **différent** de celui de la facture. Le backfill, dont la source de vérité est l'écriture réelle (D-B1), écrira fidèlement `3000` côté facture et `3200` côté avoir.

**C'est le comportement voulu.** Le résidu comptable **existe déjà dans les écritures** — il est passé, il est irréversible, et les pièces le documentent. Le rôle du backfill est de le rendre **explicite et lisible**, pas de le maquiller : réécrire l'un des deux pour forcer l'égalité falsifierait les pièces sans corriger la moindre écriture.

**Ce que la story corrige, et ce qu'elle ne corrige pas** :

- pour les avoirs **futurs**, 16-1a D5 supprime la relecture des `settings` et copie le compte depuis `invoice_lines` — le résidu ne se reproduira plus ;
- pour les avoirs **passés**, rien ne peut être corrigé sans passer une écriture de reclassement, ce qui est un **acte comptable** relevant de l'utilisateur, pas d'une migration.

**Conséquence pour 16-1b** : l'écran de détail peut afficher, pour une facture et son avoir, **deux comptes de produit différents**. Ce n'est pas une anomalie du backfill et l'UI ne doit ni le signaler comme une erreur, ni tenter de l'harmoniser.

**Ne PAS** écrire d'AC affirmant l'égalité des deux comptes : elle est fausse dans le seul scénario qui motive l'existence de cette story.

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
  5. miroir avoir : avoir `issued` à écriture canonique → ligne backfillée avec le compte **réellement débité par l'écriture de l'avoir** (D-B1), déterminé **indépendamment** de la facture d'origine ;
  6. **couple facture / avoir DIVERGENT — le cas qui compte (D-B7)** : facture validée à T1 sur défaut = 3000, défaut changé en 3200, avoir émis à T2 → le backfill écrit **3000 sur la facture et 3200 sur l'avoir**. Les deux valeurs **diffèrent, et c'est le résultat correct**. Un test qui affirmerait l'égalité échouerait ici — et un test construit sans changer le défaut entre les deux passerait trivialement sans rien prouver, exactement l'écueil signalé au cas 1.
  *(Le cas « facture validée **sans écriture** » n'est **PAS** testé : il est **inconstructible** — `chk_invoices_validated_has_je` l'interdit en base et `fk_invoices_journal_entry … ON DELETE RESTRICT` empêche de supprimer l'écriture après coup. Tenter la fixture ne produit qu'une violation de CHECK.)*
- **AC-B4** — **Idempotence** : rejouer le backfill sur une base déjà backfillée ne change **aucune** ligne (garde `IS NULL` + critère déterministe). Testé.
- **AC-B5** — `docs/migrations-idempotence-audit.md` : ligne ajoutée au tableau détaillé avec verdict **`tracked-by-sqlx`** (justifié par l'absence d'`IF NOT EXISTS`, **pas** par le backfill, qui est idempotent — D-B6), **ET** récapitulatif agrégé de bas de fichier mis à jour en cohérence (`Total` et `Idempotence tracked-by-sqlx` chacun +1). L'invariant « Idempotence `no` : 0 » est **préservé**. Garde-fou **P5** de `CLAUDE.md`.
- **AC-B6** — Migration **non-breaking** → **pas** de bump `kesh_version_min_required`, donc **pas** de bump de version Cargo (P1/P2/P2-bis). Le vérifier explicitement.
- **AC-B7** — **Aucune table n'est créée par cette story** (D-B4). Corollaire vérifié : `backup_inventory_matches_schema` (`backup.rs:577-606`) reste vert sans toucher `TABLES_TO_TRUNCATE`, et les compteurs d'export (`exports_global_e2e.rs:621` 20 entrées, `:633`) sont inchangés.
- **AC-B8** — CHANGELOG `[Non publié]` : entrée orientée utilisateur **avec les requêtes de diagnostic de D-B4**, au titre des notes de déploiement, et mention explicite que les factures dont l'écriture a été modifiée manuellement ne sont pas reprises.
- **AC-B9** — Gate « Test Locally First » backend complet vert (`cargo fmt --all -- --check`, `cargo build --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`). Les suites `migrations_fresh_install` et `admin_backup_e2e` doivent passer — cette story touche une migration.

---

## Tasks / Subtasks

- [x] **T-B0** — **Ordonnancement de la migration.** Le fichier de migration de cette story DOIT porter un timestamp **strictement postérieur** à celui de l'`ADD COLUMN` de 16-1a : `sqlx::migrate!("./migrations")` (`crates/kesh-db/src/lib.rs:23`) exécute dans l'**ordre lexicographique du nom de fichier**, et un backfill jouant avant la création de la colonne échoue en `Unknown column 'revenue_account_id'`. Vérifier l'état de 16-1a **avant** de choisir le timestamp. Si les deux stories sont développées en parallèle sur des branches distinctes, **merger 16-1a en premier** ou regrouper les deux dans la même PR (cohérent `feedback_pr_grouping` : les deux stories touchent le même répertoire de migrations). L'échec est bruyant et attrapé en CI, pas silencieux — mais il coûte un cycle.
- [x] **T-B1** — Migration de backfill `invoice_lines` : `UPDATE … JOIN (SELECT … GROUP BY … HAVING COUNT(*) = 1) c`, critère de D-B2, condition (2) en `<=>` NULL-safe (D-B3).
- [x] **T-B2** — Miroir `credit_note_lines` (`debit`, `credit_notes.total_amount`), même critère.
- [x] **T-B3** — Ligne du tableau **et** compteurs agrégés de `docs/migrations-idempotence-audit.md`, verdict `tracked-by-sqlx` + justification d'idempotence (AC-B5, D-B6).
- [x] **T-B4** — Tests sur base pré-remplie : les 5 cas d'AC-B3 + l'idempotence d'AC-B4.
- [x] **T-B5** — CHANGELOG avec requêtes de diagnostic (AC-B8) + gate backend complet (AC-B9).

**Ordre conseillé** : T-B0 → T-B1 → T-B4 (partiel, cas facture) → T-B2 → T-B4 (complet) → T-B3 → T-B5.

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
5. **Base vierge** — `migrations_fresh_install` ne prouve **rien** ici. Tous les tests d'AC-B3 exigent une base pré-remplie. Précédent direct et fonctionnel du pattern « appliquer n-1 migrations, insérer en SQL brut, appliquer la dernière » : `crates/kesh-db/tests/accounts_role_backfill.rs`.
6. **Vouloir « réparer » le couple facture / avoir divergent (D-B7)** — le réflexe naturel est d'affirmer que les deux comptes doivent être égaux. C'est **faux** pour tout couple antérieur au déploiement dont le défaut société a changé entre-temps : l'avoir a été généré en relisant `settings` à T2 (`credit_notes.rs:275-282`). Le backfill **enregistre** ce résidu, il ne le corrige pas — le corriger serait un acte comptable, pas une migration.
7. **Ordre des migrations (T-B0)** — un timestamp antérieur à l'`ADD COLUMN` de 16-1a fait échouer la migration. Bruyant, mais coûte un cycle CI.

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

## Dev Agent Record

### Implementation Plan

Ordre suivi, conforme à l'ordre conseillé de la section Tasks : **T-B0 → T-B1 + T-B2 → T-B4 → T-B3 → T-B5**. T-B1 et T-B2 ont été écrites d'un seul tenant : ce sont deux `UPDATE` du même fichier de migration, et le miroir avoir n'est que la transposition `credit`/`debit` du critère de la facture — les séparer aurait produit deux fichiers de migration là où le dépôt en attend un.

**T-B0 (ordonnancement) — tranché en ground-truth avant toute écriture.** La migration de 16-1a est `20260727000001_invoice_lines_revenue_account.sql`, **dernière du dépôt** au moment du dev. Le backfill porte donc `20260729000001`, strictement postérieur en ordre lexicographique. Les deux stories vivant sur la même branche, la question du merge dans le mauvais ordre ne se pose pas.

### Debug Log

**Vérification de discrimination des tests (mutation testing) — le seul contrôle qui prouve que D-B3 est verrouillée.**

Les 8 tests passaient du premier coup, ce qui ne prouve rien : un test vert peut l'être parce que le code est correct **ou** parce qu'il n'observe rien. La spec affirme que la condition (2) écrite `<>` au lieu de `<=>` ferait no-oper le backfill *de façon indiscernable du succès*. J'ai donc muté la migration livrée (`<=>` → `<>`, les 2 occurrences des `UPDATE`) et rejoué la suite :

| | Résultat |
|---|---|
| Migration livrée (`<=>`) | **8/8 verts** |
| Migration mutée (`<>`) | **7 verts, 1 rouge** — `backfills_when_vat_config_is_null` seul |

Message de l'échec du mutant : `left: [None, None]` / `right: [Some(4), Some(4)]`.

Deux enseignements, tous deux conformes à ce que la spec annonçait sans l'avoir démontré :

1. `backfills_when_vat_config_is_null` est bien le **seul** filet contre le piège n° 0. Sans lui, la régression passerait les 7 autres tests.
2. Le mode de défaillance est donc **empiriquement** indiscernable du succès, et non seulement en théorie.

Migration restaurée depuis sauvegarde après la mutation, `<=>` re-vérifié présent.

### Completion Notes

**Ce qui a été livré** — une migration de 2 `UPDATE`, 8 tests sur base pré-remplie, un module de test partagé, la ligne d'audit d'idempotence, l'entrée CHANGELOG, et la propagation sur 5 sites que la story rendait faux.

**Écart de conception assumé vs la spec — `LEFT JOIN` sur `company_invoice_settings`.** La spec ne se prononce pas sur le type de jointure. J'ai retenu `LEFT JOIN` : un `INNER JOIN` écarterait en silence toutes les factures d'une société **sans ligne de configuration**, reproduisant un cran plus haut exactement le mode de défaillance que D-B3 ferme (no-op indiscernable du succès). En pratique la ligne existe toujours pour une facture validée (`get_or_create_default_in_tx` fait `INSERT IGNORE` sur le chemin de validation), mais faire dépendre le backfill de cette propriété serait fragile. Absence de config = ensemble d'exclusion vide, exactement comme une colonne `NULL`.

**`MIN(jel.account_id)` n'est pas un arbitrage.** Le `HAVING COUNT(*) = 1` garantit un groupe d'une seule ligne : `MIN` **est** cette ligne. L'agrégat n'est là que parce que SQL l'exige sur une colonne non groupée.

**Extraction DRY — `tests/common/mod.rs`.** Le montage « appliquer N migrations, insérer en SQL brut, appliquer le reste » existait déjà dans `accounts_role_backfill.rs` (Story 14-3a). Plutôt que d'en faire une seconde copie, `apply_migrations_up_to` et la résolution **par version** ont été extraites dans `tests/common/mod.rs`, et le test 14-3a bascule dessus. Bénéfice au-delà du style : la résolution par version devient le **chemin unique et évident**, ce qui sert directement le garde-fou **P6**. La duplication résiduelle dans `migrations_upgrade_path.rs` est **laissée volontairement** — son helper porte un message d'assertion fail-loud propre à sa frontière positionnelle assumée, contrat différent de la résolution par version ; le fusionner diluerait précisément le garde-fou qu'il incarne.

**Garde-fou P6 appliqué (`grep -rn "migrations.len()\|apply_migrations_up_to" crates/`).** 3 sites inspectés. `migrations_upgrade_path.rs` est un site positionnel **assumé avec garde-fou fail-loud** : son `assert_eq!(total, 56)` a été porté à **57** et la fenêtre `total - 22` à `total - 23`, de sorte que la frontière reste **constante à 34** — c'est ce que son propre commentaire prescrit. Les 5 sites de nombres de ce fichier ont été mis à jour ensemble (assertion, message d'assertion du helper, doc-comment de la fonction, commentaire de frontière, prose historique) : le fichier documente lui-même qu'en 16-1a *trois copies du même symptôme y ont été découvertes une par passe*, faute d'avoir grepé à l'intérieur du fichier patché.

**Garde-fou P5 appliqué.** La ligne d'audit a été ajoutée **et** les compteurs **recomptés depuis le tableau** par script (57 lignes / `yes` 4 / `tracked-by-sqlx` 53 / `no` 0), avec contrôle croisé tableau ↔ fichiers `.sql` sur disque : **aucun écart**. Les compteurs annoncés avant modification étaient exacts (redressés en 16-1a).

**AC-B6 vérifié explicitement** : aucun `UPDATE _kesh_version` dans la migration, aucune opération `DROP`/`RENAME`/`MODIFY COLUMN`, aucune table créée → pas de bump `kesh_version_min_required`, donc pas de bump Cargo (workspace inchangé à **0.8.0**).

**Propagation post-patch — 5 sites que cette story rendait faux.** Le grep de `16-1a-bis` sur le dépôt a rendu 8 sites. Trois disent « validée avant 16-1a **et non traitée par 16-1a-bis** » et restent **exacts** (une ligne peut toujours ne pas être reprise). Ont été corrigés :

| Site | Ce qui devenait faux |
|---|---|
| `CHANGELOG.md` — puce « Factures validées avant cette version » | annonçait la limitation et « une mise à jour dédiée » : **remplacée** par la puce de reprise, sinon deux puces se contredisaient dans la même version |
| `CHANGELOG.md` — puce 16-1a, « pour toute facture validée **à partir de cette version** » | la ventilation miroir des avoirs couvre désormais aussi le parc antérieur backfillé |
| `routes/credit_notes.rs` — doc de `revenue_account_id` | affirmait « le backfill (16-1a-bis) **n'est pas livré**, ce qui couvre aujourd'hui l'essentiel du parc » |
| `routes/credit_notes.rs` — renvoi de test | pointait le nom d'avant renommage |
| `invoices_line_revenue_account.rs` — test de frontière 16-1a | cf. ci-dessous |

**Le test de frontière de 16-1a ne « change pas de verdict », et c'est correct.** Son doc-comment annonçait « le jour où le backfill est livré, ce test DOIT changer de verdict ». Il n'en est rien : le test remet la ligne à `NULL` **après** que les migrations ont tourné, donc le backfill ne la voit jamais. Ce qu'il décrit n'a jamais été le comportement du *parc antérieur* mais celui d'**une ligne `NULL`, quelle qu'en soit la raison** — ce qui reste vrai. Ce qui a changé est la **population** : de « tout le parc validé » à « les seules pièces dont l'écriture a été retouchée à la main ». Le test est donc conservé, **renommé** `legacy_invoice_…` → `null_line_credit_note_falls_back_to_current_default_known_limitation`, et son doc-comment ainsi que ses messages d'assertion recadrés. Le supprimer aurait perdu la seule caractérisation exécutable du résidu subsistant.

### ⚠️ Contradiction interne de la spec — AC-B3 cas 6 vs D-B5

**Constatée en écrivant le test, arbitrée en faveur de D-B5, non corrigée dans les AC** (le workflow interdit au dev de modifier les Acceptance Criteria).

- **AC-B3 cas 6** annonce : « facture validée à T1 sur défaut = 3000, défaut changé en 3200, avoir émis à T2 → le backfill écrit **3000 sur la facture** et 3200 sur l'avoir ».
- **D-B5** énonce l'inverse : « sur une facture créditée, `credit_note_lines.revenue_account_id` sera renseigné alors que `invoice_lines.revenue_account_id` **restera `NULL`** ».

**Ground-truth** : l'émission d'un avoir bascule **toujours** la facture d'origine en `cancelled` (`credit_notes.rs`, `UPDATE invoices SET status = 'cancelled' … AND status = 'validated'`, et un `rows == 0` fait échouer la transaction). Une facture créditée n'est donc **jamais** `validated`, et le backfill — qui ne traite que `validated` — ne la touche pas. **La première moitié d'AC-B3 cas 6 est inatteignable ; D-B5 dit vrai.**

Un test écrit littéralement d'après l'AC aurait échoué, et le réflexe naturel aurait été d'élargir le périmètre du backfill aux `cancelled` pour le faire passer — c'est-à-dire de casser D-B5 pour satisfaire une AC fausse.

La **substance** de D-B7 est intacte et reste verrouillée par `divergent_invoice_and_credit_note_are_recorded_as_is` : chaque pièce enregistre ce que **son** écriture a mouvementé, aucune harmonisation n'est tentée, et les deux comptes diffèrent. Le test assert les trois faits, dont `invoice_line_accounts == [None, None]` avec renvoi explicite à D-B5. **Décision de Guy attendue** : corriger l'énoncé d'AC-B3 cas 6, ou consigner l'écart.

## File List

**Ajoutés**

- `crates/kesh-db/migrations/20260729000001_invoice_lines_revenue_account_backfill.sql` — les 2 `UPDATE` de backfill (T-B1, T-B2).
- `crates/kesh-db/tests/invoice_lines_revenue_account_backfill.rs` — 8 tests sur base pré-remplie (T-B4).
- `crates/kesh-db/tests/common/mod.rs` — montage de fenêtre de migrations partagé, résolution par version (garde-fou P6).

**Modifiés**

- `crates/kesh-db/tests/accounts_role_backfill.rs` — bascule sur `tests/common` (extraction DRY).
- `crates/kesh-db/tests/migrations_upgrade_path.rs` — `total` 56 → 57, fenêtre `total - 22` → `total - 23` (frontière constante à 34), 5 sites de nombres alignés.
- `crates/kesh-db/tests/invoices_line_revenue_account.rs` — test de frontière renommé et recadré sur le résidu non reprenable.
- `crates/kesh-api/src/routes/credit_notes.rs` — doc de `revenue_account_id` (le backfill est livré) + renvoi de test renommé.
- `docs/migrations-idempotence-audit.md` — ligne d'audit + compteurs recomptés (T-B3, garde-fou P5).
- `CHANGELOG.md` — puce de reprise automatique avec les requêtes de diagnostic D-B4 (T-B5, AC-B8) ; puce de limitation 16-1a remplacée ; ventilation miroir des avoirs élargie au parc antérieur.
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — statut de la story.

---

## Change Log

### `bmad-dev-story` — 2026-07-29 (Opus 5)

**Story implémentée bout-en-bout, 6/6 tâches.** Migration `20260729000001_invoice_lines_revenue_account_backfill.sql` (2 `UPDATE`, aucun DDL), 8 tests sur base pré-remplie, extraction DRY du montage de fenêtre de migrations, audit d'idempotence, CHANGELOG, propagation sur 5 sites.

**Le fait marquant : la discrimination des tests a été prouvée, pas supposée.** Les 8 tests passaient du premier coup. Pour vérifier que le filet de D-B3 mord réellement, la migration livrée a été **mutée** (`<=>` → `<>`) et la suite rejouée : **7 verts, 1 rouge — `backfills_when_vat_config_is_null` seul**. La spec annonçait que le mode de défaillance serait « indiscernable du succès » ; c'est désormais **mesuré**, pas argumenté. Migration restaurée, `<=>` re-vérifié.

**Contradiction interne de la spec constatée** — AC-B3 cas 6 (« le backfill écrit 3000 **sur la facture** ») est **inatteignable** : l'émission d'un avoir bascule toujours la facture en `cancelled`, statut que D-B5 exclut délibérément. **D-B5 dit vrai.** Arbitré en faveur de D-B5, AC non modifiée (hors mandat du dev), test écrit sur le comportement atteignable — la substance de D-B7 reste entièrement verrouillée. Détail et ground-truth en Dev Agent Record. **En attente d'arbitrage de Guy.**

**Deux écarts de conception assumés vs la spec**, tous deux documentés en Dev Agent Record : `LEFT JOIN` sur `company_invoice_settings` (un `INNER JOIN` reproduirait le no-op silencieux de D-B3 un cran plus haut, pour les sociétés sans ligne de config) ; extraction de `tests/common/mod.rs` plutôt qu'une seconde copie du montage de 14-3a — la duplication résiduelle de `migrations_upgrade_path.rs` est laissée volontairement, son helper portant un garde-fou fail-loud propre à sa frontière positionnelle.

**Garde-fous de `CLAUDE.md` appliqués** : **P6** — 3 sites de couplage positionnel inspectés, `migrations_upgrade_path.rs` porté à `total == 57` / fenêtre `total - 23` (frontière constante à 34), ses **5** sites de nombres alignés d'un seul patch ; **P5** — ligne d'audit ajoutée et compteurs **recomptés depuis le tableau** par script avec contrôle croisé sur les `.sql` du disque, aucun écart (57 / 4 / 53 / 0) ; **P1-P2-P2-bis** — aucun `DROP`/`RENAME`/`MODIFY COLUMN`, pas de bump `min_required`, workspace inchangé à 0.8.0 (AC-B6).

**Propagation post-patch** : le grep de `16-1a-bis` a rendu 8 sites, dont **5 rendus faux par cette story** et corrigés dans le même patch — notamment le test de frontière de 16-1a, dont l'annonce « ce test DOIT changer de verdict » s'est révélée inexacte (il remet la ligne à `NULL` **après** les migrations, donc le backfill ne la voit jamais). Test conservé, renommé et recadré : ce qui a changé n'est pas son verdict mais la **population** qu'il décrit.

**Gate (AC-B9) — VERT sur l'état final** : `cargo fmt --all -- --check` ✅, `cargo build --workspace --all-targets` ✅, `cargo clippy --workspace --all-targets -- -D warnings` ✅ (0 warning, notamment aucun `dead_code` sur le module `tests/common`), suite workspace complète **2069/2069 passés, 4 skipped, exit 0** (2899 s sur DB `kesh_gate`). Les suites que la story met en risque sont toutes vertes : `migrations_fresh_install` (3), `migrations_upgrade_path` (8, dont `upgrade_path_preserves_data` et son `assert_eq!(total, 57)`), `accounts_role_backfill` (3), `admin_backup_e2e` / `backup_inventory_matches_schema` et `exports_global_e2e` (AC-B7 : aucune table créée, compteurs d'export inchangés), plus les **8 nouveaux** de `invoice_lines_revenue_account_backfill`.

**Le gate a été rejoué intégralement sur l'état final, il n'a pas été présumé.** Un premier gate avait été lancé puis **abandonné à 574/2069** : l'ajout tardif de l'assertion sur la seconde requête de diagnostic (celle des avoirs) rendait son binaire de test périmé pour cette suite. Composer « le gate d'avant + un `cargo test` ciblé » aurait été exactement le raccourci que la 16-1a s'était interdit. `kesh_gate` a été contrôlée après l'arrêt en vol (piège connu `postable = 0` sur le compte 1000 → 26 faux échecs) : **intacte**.

**Prochaine étape** : `bmad-code-review` (LLM ≠ Opus, contexte frais).

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

**Statut de revue à la création** : le corpus avait été revu en passes 5 (Haiku + orchestrateur) et 6 (Opus) **au sein de 16-1a**, mais jamais comme un tout autonome — d'où la passe 1 ci-dessous.

### Passe 1 de `validate` — 2026-07-26 (Sonnet, contexte frais)

**2 findings : 1 HIGH, 1 MEDIUM.** Tous deux vérifiés en ground-truth avant application. Le HIGH est né **du split lui-même** : le miroir avoir, simple corollaire dans 16-1a, est traité ici au même rang que la facture — et ce changement de statut a révélé une affirmation fausse que personne n'avait interrogée en 4 passes.

| Finding | Verdict | Traitement |
|---|---|---|
| **HIGH — AC-B3 cas 5 affirmait que le compte backfillé de l'avoir « égale celui de la facture d'origine ».** C'est **faux précisément dans le scénario qui motive la story**. `create_credit_note` relit la configuration au moment de l'émission (`credit_notes.rs:275-282`, `get_or_create_default_in_tx` puis `settings.default_revenue_account_id`), indépendamment de ce que la facture a crédité — c'est le bug du tableau T1/T1+/T2 du § Contexte. Les avoirs déjà en base ont été produits par ce code : leur écriture débite légitimement un **autre** compte que la facture | **Réel.** L'AC ne laissait que deux issues, toutes deux mauvaises : un test sans changement de défaut, qui passe trivialement et **ne prouve rien** (l'écueil que le cas 1 signale explicitement, jamais repris au cas 5) ; ou un test fidèle au scénario réel, qui **échoue** — non parce que le backfill est bugué, mais parce que l'AC affirme une invariance inexistante | **D-B7 ajoutée** — « le backfill ENREGISTRE le résidu historique, il ne le RÉPARE pas » : le résidu existe déjà dans les écritures, il est passé et irréversible ; forcer l'égalité falsifierait les pièces sans corriger la moindre écriture. Ce que la story corrige (avoirs futurs, via 16-1a D5) et ce qu'elle ne corrige pas (avoirs passés, qui relèveraient d'une écriture de reclassement — acte comptable, pas migration) est explicité. **AC-B3 cas 5 réécrit** (compte déterminé indépendamment de la facture) et **cas 6 ajouté** : le couple divergent est le cas qui compte, avec les deux valeurs attendues. Piège n°6 ajouté. **Conséquence répercutée sur 16-1b** : l'écran de détail peut légitimement afficher deux comptes différents pour une facture et son avoir ; ne pas le signaler comme une anomalie, ne pas tenter de l'harmoniser |
| MEDIUM — la dépendance d'ordre avec la migration `ADD COLUMN` de 16-1a est affirmée en prose (« dépend strictement de 16-1a ») mais **jamais opérationnalisée**. `sqlx::migrate!` exécute dans l'ordre **lexicographique du nom de fichier** (`kesh-db/src/lib.rs:23`) : un timestamp mal choisi, ou un merge de PR séparées dans le mauvais ordre, fait jouer le backfill avant la création de la colonne | Réel. Échec **bruyant** (`Unknown column`), attrapé en CI — donc pas de corruption silencieuse, mais un cycle perdu, et un gap de complétude réel pour une story qui documente par ailleurs tous ses autres risques d'ordonnancement | **T-B0 ajoutée** : contrainte de timestamp explicite, vérification de l'état de 16-1a avant de choisir, et consigne de merger 16-1a en premier ou de regrouper les deux dans la même PR (cohérent `feedback_pr_grouping` — les deux touchent le même répertoire de migrations). Piège n°7 ajouté |

**Vérifié négatif (substantiel — ne pas ré-instruire)** : (1) **le compte de produit ne peut jamais collisionner avec le compte de créance** dans l'ensemble d'exclusion `E` — `validate_account` (`routes/company_invoice_settings.rs:94-120`) impose `account_type == expected` (`Asset` pour la créance, `Revenue` pour le produit) et `chk_accounts_type` (`20260411000001_accounts.sql:20`) ferme la liste : un même compte ne peut satisfaire les deux types. (2) **`credit_notes.total_amount` est bien du HT**, miroir strict d'`invoices.total_amount` (commentaire de schéma `20260627000001_credit_notes.sql:19` + code `total_ht = Σ line_total` inséré tel quel). (3) **Les avoirs partiels n'existent pas** : `create_credit_note` est la seule fonction publique de création et snapshot **toutes** les `invoice_lines` sans filtre — chaque avoir est nécessairement total. (4) Faisabilité MariaDB confirmée (ER 1093 inapplicable, aucune CTE dans le dépôt, précédent multi-table `20260628000001:115`). (5) **Pas de risque de scan complet** : `idx_jel_entry` sur `journal_entry_lines(entry_id)` (`20260412000001:47`). (6) **Aucune contrainte DB ne bloque un `UPDATE` sur une facture d'exercice clos** — la garde est purement applicative, hors périmètre d'une migration SQL. (7) Traçabilité D-B1..D-B7 → AC-B1..AC-B9 → T-B0..T-B5 complète, aucun orphelin. (8) **Précédent de test sur base pré-remplie** : `crates/kesh-db/tests/accounts_role_backfill.rs` valide le pattern exigé par T-B4.

**Trend** : passe 1 = 2 findings (1 HIGH, 1 MEDIUM). Sévérité au-dessus de LOW → **passe 2 requise** (contexte frais, modèle différent). Cible prioritaire : **D-B7 et les cas 5-6 d'AC-B3**, patch tout neuf — en particulier la cohérence du récit « enregistrer sans réparer » avec D-B1 et avec le § Contexte, et sa répercussion sur 16-1b.

### Passe 2 de `validate` — 2026-07-26 (Haiku 4.5, contexte frais + vérification orchestrateur)

**1 finding LOW.** Le reviewer a rendu « 0 finding » après avoir instruit les 6 angles imposés ; le LOW vient de la vérification d'orchestrateur — conformément à la leçon consignée en passe 5 de 16-1a (« un *rien trouvé* de Haiku n'est pas une preuve de convergence »).

| Finding | Verdict | Traitement |
|---|---|---|
| LOW — le § « **Le bug fermé par cette story** » présente le scénario T1/T1+/T2 dont **T2 est déjà passé** (« Avoir total émis »), c'est-à-dire exactement le cas que **D-B7 dit ne PAS réparer**. Le titre et le tableau promettent donc une fermeture que la décision structurante retire trois sections plus loin. La clause `so that` de l'en-tête portait la même imprécision (« un avoir émis plus tard », sans dire *plus tard que quoi*) | Réel, et **même racine que le HIGH de la passe 1** : le symptôme n'avait pas été entièrement grepé après le patch (§ « Propagation post-patch » de `CLAUDE.md`). Sévérité LOW et non HIGH parce qu'aucun AC ni aucune tâche n'en dépend — c'est du récit, pas une prescription | Tableau des **deux populations** ajouté au § Contexte : *avoir pas encore émis* (immense majorité) → **le bug est fermé** ; *avoir déjà émis* → **rien n'est réparé**, les deux comptes diffèrent légitimement (D-B7). Formule de synthèse : le backfill est **préventif, pas curatif** — il arme les factures passées pour que leurs avoirs **à venir** soient corrects. Clause `so that` de l'en-tête précisée (« un avoir **émis à partir de maintenant** sur une facture ancienne ») |

**Vérifié négatif (confirmations du reviewer, contrôlées)** : (1) **cohérence D-B7 ↔ D-B1 ↔ 16-1a D5/AC11** — 16-1a D5 fait bien copier le compte depuis la ligne de facture à la création de l'avoir, et AC11 impose la transformation du site d'appel `credit_notes.rs:320-328` en triplets ; les avoirs futurs ne reliront donc plus `settings`, ce qui rend le récit de D-B7 exact. (2) **Le cas 6 d'AC-B3 est constructible** : la configuration est mutable, `create_credit_note` relit bien à l'émission (`credit_notes.rs:276`), `uq_credit_notes_invoice` n'entrave pas le scénario, et le pattern de test sur base pré-remplie a son précédent (`accounts_role_backfill.rs`). (3) **T-B0 correcte** : `sqlx::migrate!` trie lexicographiquement, le nommage `YYYYMMDDNNNNNN_*.sql` du dépôt rend la contrainte de timestamp triviale à respecter. (4) Traçabilité D-B1..D-B7 → AC-B1..AC-B9 → T-B0..T-B5 complète, aucun orphelin. (5) Conformité `CLAUDE.md` : P1/P2/P2-bis (AC-B6), P5 audit d'idempotence (AC-B5), Test Locally First (AC-B9).

**Trend** : passe 1 = 2 findings (1 HIGH, 1 MEDIUM) → passe 2 = **1 LOW**. **Critère d'arrêt de la § « Review Iteration Rule » atteint** : plus aucun finding au-dessus de LOW. **16-1a-bis est convergée** en 2 passes.
