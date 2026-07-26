# Story 16.1a : Compte de produit par ligne — socle backend

## Status

ready-for-dev

## Story

**As a** indépendant / PME / fiduciaire qui facture des **natures de prestations différentes** (honoraires, prestations de services, marchandises, produits annexes),
**I want** que chaque **ligne** de facture puisse porter son propre **compte de produit**, avec repli sur le compte de produit par défaut de la société quand rien n'est précisé,
**so that** l'écriture comptable générée à la validation **ventile le crédit produit sur les bons comptes** au lieu de tout créditer sur un compte unique — ce qui rend mon compte de résultat exploitable sans reclassement manuel a posteriori.

Issue : **#152**. Rattaché au CR **#265**. Socle de l'Epic 16 « Facturation avancée ».

**Périmètre de cette sous-story** : DB, entités, API, moteur comptable (facture **et** avoir), exports CSV, tests backend. Le sélecteur dans le formulaire de facture est en **16-1b**.

### Provenance du split (passe 1 de `validate`)

La story 16-1 initiale touchait **8 modules distincts** (seuil de la § « Règle de splitting préventif » de `CLAUDE.md` : 5). Split acté par Guy le 2026-07-26 en :

- **16-1a** (cette story) — backend : `kesh-db/migrations`, `kesh-db/repositories/invoices`, `kesh-db/repositories/credit_notes`, `kesh-api/routes/invoices`, `kesh-api/exports` = **5 modules**.
- **16-1b** — frontend : `frontend/components/invoices`, `frontend/features/journal-entries`, `frontend/features/reconciliation`, i18n, doc-sync = **5 modules**.

**Le split n'ouvre aucune fenêtre de corruption comptable** : toute la correction du moteur (facture **et** avoir, décision D5) est dans 16-1a.

**16-1a est livrable seule** — colonne nullable + champ API rétro-compatible, sans UI. **Nuance à ne pas perdre : « sans UI » ne signifie pas « sans effet ».** Le champ est **écrivable par API** dès 16-1a, et `InvoiceForm.svelte` reconstruit ses lignes à partir de 4 champs en dur (`initLines()`, `:107-113`) avant de les renvoyer telles quelles en `UpdateInvoiceRequest` (`:365-368`) : combiné au `#[serde(default)]` d'AC6, un enregistrement depuis l'UI **remet silencieusement à `NULL`** tout compte posé par API. Effacement borné et acceptable tant qu'aucun sélecteur n'existe, mais c'est une **dépendance dure de 16-1b**, pas un détail d'UI.

---

## Contexte

### Provenance du contexte d'epic (à lire avant de chercher ailleurs)

**Il n'existe pas d'`epic-16.md`.** `_bmad-output/planning-artifacts/epics.md` est explicitement déclaré **obsolescent** (bandeau en tête + tableau de correspondance de la renumérotation, décision CR-009 #61 fermée `not planned`) et **ne contient aucun Epic 16** — son « Epic 14 » correspond à l'Epic 15 actuel. La source de vérité de l'Epic 16 est donc :

- `sprint-status.yaml` (séquence des stories + motif de l'ordre) ;
- les issues GitHub **#152** (cette story), **#144** (16-2), **#151** (16-3, reliquat) ;
- le CR **#265**.

Ne pas chercher de spec d'epic ailleurs, et ne pas se fier à la numérotation d'`epics.md`.

### Ce qui existe aujourd'hui

- **`invoice_lines`** (`crates/kesh-db/migrations/20260416000001_invoices.sql:41`) : `id`, `invoice_id`, `position`, `description`, `quantity`, `unit_price`, `vat_rate`, `line_total`, `created_at`. **Aucune colonne de compte, aucun `product_id`.**
- **`credit_note_lines`** (`migrations/20260627000001_credit_notes.sql:63`) : **copie snapshot** des lignes de facture, mêmes colonnes, sans compte. Porte en plus `chk_credit_note_lines_line_total_non_negative`.
- **`generate_invoice_journal_lines`** (`crates/kesh-db/src/repositories/invoices.rs:1156`, doc `:1120-1155`) produit exactement :
  - `[0]` **débit créance** = `total_ht + total_vat` (TTC), poussé **inconditionnellement** ;
  - `[1]` **crédit produit UNIQUE** = `total_ht`, poussé **inconditionnellement** ;
  - `[2..N]` **crédit TVA due**, une ligne par taux dont le montant agrégé est `> 0`, triées par taux croissant (`BTreeMap` ASC).
- **`validate_invoice`** (`invoices.rs:1245`) lit `settings.default_revenue_account_id` (`:1319-1320`) et le passe au helper (`:1383-1387`). Absent → `DbError::ConfigurationRequired("default_revenue_account_id")`.
- **Résolution par rôle (14-3b)** : `crates/kesh-db/src/repositories/company_invoice_settings.rs:236` documente que `Receivable` / `DefaultRevenue` / `Payable` servent à **pré-remplir `company_invoice_settings` à la finalisation de l'onboarding**. Ce n'est **pas** le chemin de résolution au posting.
- **Garde de comptes au posting** : `journal_entries::create_in_tx` (`journal_entries.rs:210-232`) appelle `validate_lines_accounts_in_tx` (`:65-114`) sur **tous** les comptes de l'écriture. Cf. D3 pour ce que cette garde couvre **et ne couvre pas**.
- **Re-validation du projet analytique au posting** (Story 19-4) : `invoices.rs:1290-1310` — `SELECT` simple **sans verrou**, volontairement (le commentaire documente l'inversion ABBA qu'un `FOR UPDATE` créerait ici). **C'est le patron à copier** pour le compte de ligne.
- **Validation batchée d'ids référencés par ligne** : `projects::validate_taggable_in_tx` (`projects.rs:87-121`) — une seule requête `IN (...)`, dédup interne. Appelée depuis `invoices.rs:459` (création) et `:816` (modification). **C'est le point d'accroche de la validation à la saisie.**
- **Style d'erreur de validation de ligne** : `crates/kesh-api/src/routes/invoices.rs` (`validate_line`, `:370-410`) utilise `AppError::Validation(format!("Ligne {} : …", index + 1))`. C'est la convention du fichier.
- **Export CSV** : `serialize_invoice_lines_csv` (`crates/kesh-api/src/exports/csv_tables.rs:459`), en-tête à 9 colonnes.

### Ce qui n'existe PAS (deltas à construire)

1. Colonne `revenue_account_id` sur `invoice_lines` **et** `credit_note_lines`.
2. Champ `revenueAccountId` dans `CreateInvoiceLineRequest` (`routes/invoices.rs:65-71`, aujourd'hui 4 champs) et dans la réponse de lecture.
3. Ventilation du crédit produit par compte dans le helper d'écriture facture, et du débit produit dans le helper d'avoir.
4. Validation du compte choisi (société, type `Revenue`, `postable`, `active`), à la saisie **et** au posting.

### Ce qui n'est PAS dans 16-1a

- Le **sélecteur dans le formulaire de facture**, le déplacement d'`AccountAutocomplete`, l'i18n et le doc-sync utilisateur → **16-1b**.
- Le compte porté par la **fiche produit** du catalogue → **16-2 (#144)**. 16-1a ne touche pas `products`.
- Le **compte de charge** par ligne de facture **fournisseur** → hors périmètre (#265 second volet).
- Aucun changement du **calcul de TVA** ni de la **présentation du PDF**.

**Reporté en 16-1b, mais obligatoire** : `InvoiceForm.svelte` construit ses `LineState` en **deux** endroits — `initLines()` (`:107-113`) et `reloadFromServer()` (`:418-424`) — qui doivent tous deux propager `revenueAccountId` depuis la réponse serveur. Sinon toute édition (ou tout rechargement après conflit de version) efface le compte de chaque ligne **sans message**. Tracé ici parce que la dépendance naît de 16-1a.

---

## Décisions de conception

### D1 — Le repli est `company_invoice_settings.default_revenue_account_id`, PAS le rôle `DefaultRevenue`

Une ligne sans compte se poste sur `settings.default_revenue_account_id`, exactement comme aujourd'hui.

**Motif** : la colonne est la source de vérité au runtime et reste **configurable par l'utilisateur** dans les Réglages ; le rôle `DefaultRevenue` ne sert qu'à **pré-remplir cette colonne à l'onboarding** (14-3b, cf. docstring `company_invoice_settings.rs:236`). Résoudre par rôle au posting serait un **changement de comportement non demandé** : il écraserait silencieusement le choix d'un utilisateur qui a délibérément pointé un autre compte dans les Réglages. Le message d'erreur `ConfigurationRequired("default_revenue_account_id")` reste inchangé.

**Garde-fou de test obligatoire (cf. AC20)** : par défaut post-onboarding, `settings.default_revenue_account_id` **et** le compte portant le rôle `DefaultRevenue` sont le **même** compte — un test qui ne les dissocie pas passerait aussi bien avec une résolution par rôle. Le test doit donc les configurer sur **deux comptes différents**.

### D2 — Liaison tardive : `NULL` n'est jamais matérialisé **à la création**, mais il l'est **à la validation**

La colonne est `BIGINT NULL`. Sur un **brouillon**, `NULL` signifie « utiliser le défaut société **au moment de la validation** » — on ne copie pas le défaut à la création.

**À la validation**, en revanche, `validate_invoice` **matérialise** le compte effectif : après résolution et **avant** `journal_entries::create_in_tx`, dans la même transaction :

```sql
UPDATE invoice_lines SET revenue_account_id = ?  -- settings.default_revenue_account_id
WHERE invoice_id = ? AND revenue_account_id IS NULL
```

Après validation, **aucune ligne d'une facture validée par ce chemin ne porte `NULL`**. C'est stable : une facture validée est immuable (`update` rejette tout statut ≠ `draft`). **La borne « par ce chemin » est essentielle** : les factures validées **avant** le déploiement ne repassent jamais par ici, et le backfill de D2-bis étant délibérément incomplet, certaines conserveront `NULL` définitivement. L'énoncé ne se teste donc **jamais** par un `COUNT(*) = 0` global sur `invoice_lines`.

**Motif de la partie « pas à la création »** : conserve le comportement actuel à l'identique pour toute facture existante ou créée sans préciser de compte. Matérialiser à la création ferait suivre au brouillon un défaut périmé.

**Motif de la partie « matérialisé à la validation »** — c'est le correctif du finding CRITICAL de la passe 3, et il ferme un **bug pré-existant** :

`settings.default_revenue_account_id` est **mutable** par l'utilisateur dans les Réglages (`company_invoice_settings.rs:173`). Sans matérialisation, la chaîne pour une ligne `NULL` est : ligne `NULL` → snapshot d'avoir `NULL` (D5 copie la valeur de la ligne) → repli résolu **au moment de l'avoir**, sur `get_or_create_default_in_tx` relu à T2 (`credit_notes.rs:275-282`). D'où :

| T | Événement | Écriture |
|---|---|---|
| T1 | Facture validée, ligne `NULL`, défaut = 3000 | **crédit 3000** = HT |
| T1+ | L'administrateur change le défaut → 3200 | — |
| T2 | Avoir total émis | **débit 3200** = HT |

Résidu permanent au crédit de 3000 et au débit de 3200. Bilan équilibré, **compte de résultat faux, aucun signal** — mot pour mot le mode de défaillance que D5 qualifie de « point le plus grave de la story ». La spec le fermait pour les comptes explicites et le laissait ouvert pour `NULL`, c'est-à-dire **le seul cas qui existe en production aujourd'hui** (aucune facture existante ne porte de compte de ligne).

La matérialisation rend la facture validée **auto-descriptive** et réduit le système à **un seul instant de résolution**. Elle simplifie aussi D5-bis : la re-validation `active` côté avoir porte alors sur un ensemble complet, sans compte implicite.

**Borne de cet invariant (passe 5)** — « auto-descriptive » vaut *au moment de la validation*, pas indéfiniment. L'écriture générée par une validation de facture n'est marquée par **rien** : `journal_entries` n'a **aucune** colonne `source` / `origin` / `is_auto` (schéma complet `migrations/20260412000001_journal_entries.sql:17-36`), et `PUT /api/v1/journal-entries/{id}` (`kesh-api/src/lib.rs:303` → `routes/journal_entries.rs:517` → `repositories/journal_entries.rs:805`) ne contrôle **ni la provenance ni le rattachement à une facture** — ses seules gardes sont l'exercice clos, le verrou optimiste, l'équilibre en partie double et la validité des comptes. Un utilisateur peut donc éditer l'écriture d'une facture validée : `update` fait `DELETE FROM journal_entry_lines WHERE entry_id = ?` puis ré-INSERT avec un `line_order` recalculé (`:1005`, `(idx as i32) + 1`).

Conséquence à porter, sans la corriger ici : après une telle édition, `invoice_lines.revenue_account_id` décrit l'écriture **telle qu'elle a été générée**, pas telle qu'elle est. C'est déjà mieux qu'aujourd'hui (où l'avoir se replierait sur le défaut *courant*, encore plus éloigné), donc **la story ne régresse rien** — mais l'invariant ne doit pas être invoqué comme une garantie de cohérence facture ↔ écriture. Ce découplage est **hors périmètre** de 16-1a. Sa seule conséquence opératoire est sur D2-bis, ci-dessous.

Le bug est **antérieur à la story** (aujourd'hui déjà, facture = `défaut(T1)`, avoir = `défaut(T2)`), mais la story réécrit exactement ces deux helpers et érige l'invariant en décision — même logique que D4-bis.

### D2-bis — La matérialisation ne suffit pas : il faut un BACKFILL du parc existant

**Trou du patch de la passe 3, trouvé en passe 4.** D2 ne se déclenche que dans `validate_invoice`, c'est-à-dire à la seule transition `draft → validated`. Une facture **déjà validée** avant le déploiement n'y repassera **jamais** — `update` rejette tout statut ≠ `draft` (`invoices.rs:841`, `:1271`) et aucune autre écriture ne touche `invoice_lines` après validation. Après l'`ADD COLUMN` d'AC1, ces lignes portent `NULL` **définitivement**.

Or la spec l'écrit elle-même : le cas `NULL` est « le seul qui existe en production aujourd'hui ». Donc **100 % des factures déjà validées** — dont celles de l'instance de production en fonction — restent exposées au résidu que D2 vient de fermer pour les factures futures. Le patch de la passe 3 protégeait un ensemble **vide** à l'instant du déploiement.

**Décision** : la migration d'AC1 comporte un **backfill**, dont la source de vérité est l'**écriture comptable réellement générée**, pas le défaut courant.

**Pourquoi pas le défaut courant** : si l'administrateur a déjà changé `default_revenue_account_id` par le passé, backfiller avec la valeur d'aujourd'hui écrirait un compte que la facture n'a **jamais** crédité — on fabriquerait la corruption au lieu de la fermer. L'écriture générée est la **pièce probante** (c'est l'argument même de D2).

**Mais la pièce probante n'est pas garantie intacte — le backfill doit être CONSERVATEUR (passe 5).** La structure `[0]` créance / `[1]` produit / `[2..]` TVA (docstring `invoices.rs:1120-1155`) décrit ce que `generate_invoice_journal_lines` **produit**, pas ce que la table **contient** au moment de la migration. Comme établi sous D2 (borne de l'invariant), l'écriture d'une facture validée est **éditable par l'utilisateur** via `PUT /journal-entries/{id}`, sans aucune garde de provenance, et `update` réattribue les `line_order` de zéro. Sur une base réelle, une écriture de facture peut donc porter ses lignes dans un autre ordre, sur d'autres comptes, en nombre différent.

Les deux méthodes d'identification envisagées échouent **silencieusement** sur ces écritures :

- **positionnelle** (`line_order = 2`) — après ré-INSERT, la 2ᵉ ligne peut être n'importe quoi, y compris une ligne de TVA ou une charge ;
- **par élimination** (tout ce qui n'est ni la créance ni la TVA) — une écriture éditée peut en laisser **zéro** ou **plusieurs**.

Et l'erreur n'est pas neutre : un mauvais compte écrit dans `invoice_lines` devient **la** vérité de la facture, que D5 recopiera dans tout avoir futur. **Un backfill approximatif fabrique exactement la corruption que D2-bis existe pour fermer**, en lui donnant en plus l'apparence d'une donnée établie. Sur des écritures non éditées — la quasi-totalité du parc — l'identification est en revanche parfaitement fiable.

**Règle du backfill** : ne backfiller **que** si la ligne de crédit produit est identifiable **sans ambiguïté**, c'est-à-dire s'il existe dans l'écriture **exactement une** ligne remplissant les trois conditions :

1. `credit > 0` (donc `debit = 0`, garanti exclusif par `chk_jel_debit_credit_exclusive`) ;
2. son `account_id` n'appartient pas à l'ensemble d'exclusion `E = { cis.default_receivable_account_id, cis.default_vat_payable_account_id }`, **les valeurs `NULL` étant ignorées** (une colonne de configuration non renseignée n'exclut rien) ;
3. son `credit` est **égal à `invoices.total_amount`** (le HT, `Σ line_total` — cf. docstring `invoices.rs:1128`, qui écrit littéralement « `total_ht` (HT, = `invoices.total_amount`, DC9) »). **Il n'existe pas de colonne `total_ht`** ; ne pas la chercher.

**La condition (2) DOIT être écrite en SQL NULL-safe — piège dirimant (passe 6).** Les colonnes de configuration sont toutes nullables, et les trois colonnes TVA sont **`NULL` sur toute installation** : ni l'`INSERT` d'onboarding (`company_invoice_settings.rs:452-458`, qui n'énumère que `invoice_number_format` / `default_receivable_account_id` / `default_revenue_account_id` / `default_payable_account_id` / `default_sales_journal` / `journal_entry_description_template`), ni le lazy-create (`:92`, `INSERT IGNORE … (company_id) VALUES (?)`), ni aucune migration (`grep -rn "SET default_vat" crates/kesh-db/migrations/` → vide) ne les renseigne. Écrite naïvement `jel.account_id <> cis.default_vat_payable_account_id`, la comparaison à `NULL` rend le prédicat `NULL`, la ligne n'est **jamais** candidate, et le backfill **no-ope sur 100 % du parc** — migration en succès, décompte « très élevé », c'est-à-dire **rigoureusement indiscernable du comportement conservateur que la spec pré-autorise**. Utiliser l'opérateur NULL-safe de MariaDB :

```sql
NOT (jel.account_id <=> cis.default_receivable_account_id)
AND NOT (jel.account_id <=> cis.default_vat_payable_account_id)
```

**Jamais** `<>` / `!=` / `NOT IN`, qui propagent `NULL`.

**Pourquoi `E` ne contient que ces deux comptes** : `default_vat_recoverable_account_id` et `default_vat_decompte_account_id` n'apparaissent **jamais** dans une écriture de vente — `generate_invoice_journal_lines` ne reçoit que le compte de TVA due (`invoices.rs:1387`). Les inclure n'ajoute rien et multiplie les occasions de propager un `NULL`.

La condition (3) est le discriminant que ni la position ni l'élimination ne donnent : c'est elle qui distingue une écriture canonique d'une écriture retouchée. Si le compte n'est pas identifiable ainsi — zéro candidat, ou plusieurs — la ligne reste `NULL` et est **dénombrable par la requête de diagnostic** ci-dessous (cf. AC2-bis). Vaut aussi pour le miroir sur `credit_note_lines`, avec `debit` au lieu de `credit` et `credit_notes.total_amount`.

*(Redondance assumée : l'exclusion de la créance en (2) est inoffensive mais inutile — sur l'écriture facture la créance est un **débit**, déjà éliminée par (1) ; sur l'écriture d'avoir elle est un crédit, éliminée par le (1) miroir. Seule l'exclusion TVA porte réellement. Conservée pour lisibilité et défense en profondeur.)*

**Ce que valent les `NULL` résiduels** : rigoureusement le comportement d'aujourd'hui (repli sur le défaut courant au moment de l'avoir), c'est-à-dire le bug pré-existant — mais désormais **borné et dénombrable** au lieu d'être universel. C'est le bon arbitrage : refuser d'écrire plutôt qu'écrire faux sur des données comptables réelles. Si le décompte s'avérait non-négligeable sur l'instance de production, la remédiation relève d'une story dédiée avec arbitrage utilisateur au cas par cas, pas d'une heuristique en migration.

**Comment le décompte est restitué — tranché ici, PAS en `dev-story` (passe 6)** : par **aucun artefact nouveau**. Une migration sqlx est un `.sql` pur exécuté par `MIGRATOR.run()` (`kesh-db/src/lib.rs:23`, appel `kesh-api/src/main.rs:138`) : elle n'a aucun canal de restitution. Les deux options envisagées en passe 5 sont écartées :

- *table de rapport* — **refusée** : toute nouvelle table fait tomber `backup_inventory_matches_schema` (`backup.rs:577-606`), impose de mettre à jour `TABLES_TO_TRUNCATE`, et fait donc entrer la table dans le périmètre de l'**export/import d'installation** — décision d'architecture qui serait prise par accident, pour faire passer un test, et qui contredirait AC14-bis ;
- *log applicatif au démarrage* — **refusée** : ajoute dans `main.rs` une requête de comptage rejouée à chaque boot indéfiniment, pour un besoin ponctuel de déploiement, et n'est affectée à aucune tâche.

**Décision** : le décompte est obtenu par une **requête de diagnostic documentée**, exécutable à tout moment, sans état persistant :

```sql
SELECT COUNT(*) FROM invoice_lines il JOIN invoices i ON i.id = il.invoice_id
WHERE i.status = 'validated' AND il.revenue_account_id IS NULL;
```

Elle est consignée dans le CHANGELOG (AC24) au titre des notes de déploiement, et c'est **elle** que le test d'AC2-bis assert. Aucune table, aucun log, aucune ligne dans `main.rs`.

**Portée** : `invoice_lines` des factures `status = 'validated'` **et** `journal_entry_id IS NOT NULL` (prédicat conservé comme garde défensive, **redondant** avec `chk_invoices_validated_has_je`, `migrations/20260417000002_invoice_validated_journal_entry_check.sql`), plus le miroir sur `credit_note_lines` depuis l'écriture de contre-passation. Les factures `draft` restent `NULL` (D2).

Les factures `cancelled` sont exclues **délibérément**, et non « parce qu'elles n'ont pas d'écriture » — elles en ont toujours une : le seul chemin qui produit ce statut est l'émission d'un avoir (`credit_notes.rs:398`, `UPDATE invoices SET status = 'cancelled' … AND status = 'validated'`). Le motif réel est que `uq_credit_notes_invoice UNIQUE (invoice_id)` (`20260627000001_credit_notes.sql:58`) interdit un second avoir : **aucun résidu futur n'est possible**, il n'y a rien à prévenir. Conséquence assumée : sur une facture créditée, `credit_note_lines.revenue_account_id` sera renseigné alors que `invoice_lines.revenue_account_id` restera `NULL` — visible dans l'export CSV d'AC14, sans effet comptable.

**Points à trancher en `dev-story`, à documenter dans le Change Log** : l'expression SQL exacte du critère d'unicité en dialecte MariaDB. Forme recommandée `UPDATE … JOIN (SELECT … GROUP BY … HAVING COUNT(*) = 1) c`, qui a un précédent direct dans le dépôt (`20260628000001_supplier_invoices.sql:115`, multi-table UPDATE en migration) ; **ne pas** utiliser de CTE (`WITH`) — aucune migration du dépôt n'en contient. La restriction MariaDB ER 1093 ne s'applique pas ici : la sous-requête de candidats lit `journal_entry_lines` / `invoices` / `company_invoice_settings`, jamais la table cible `invoice_lines` (miroir : jamais `credit_note_lines`). **Ne pas** retenir l'identification par `line_order` seul : elle est réfutée ci-dessus.

**Alternative écartée** : documenter le parc existant en dette catégorie B. Refusée — la § « Tech debt management » l'autoriserait, mais il s'agit d'une **corruption comptable silencieuse sur des données réelles déjà en production**, pas d'une limitation fonctionnelle. Le coût du backfill est une requête ; le coût de l'omission est un compte de résultat faux sans signal.

### D3 — Double validation : à la saisie ET au posting — sur ce que la garde existante ne couvre PAS

**Ce que `create_in_tx` valide déjà** (ground-truth `journal_entries.rs:65-114`, vérifié 2026-07-26) : pour tous les comptes de l'écriture, `company_id` correct **et** `active = TRUE`, **inconditionnellement**.

**Ce qu'il ne valide PAS** :

- `postable` — la clause n'est ajoutée que si `enforce_postable = true`, et le flux facture passe `false` (14-3b, D-A0 : « flux automatique, poste sur des comptes de config ») ;
- `account_type` — **jamais** vérifié, à aucun moment. Un compte peut être retypé `Revenue → Expense` par `accounts::update` (`accounts.rs:344`) sans qu'aucun garde-fou ne consulte `invoice_lines` ;
- la **précision de l'erreur** — l'échec remonte `DbError::InactiveOrInvalidAccounts` → `400 INACTIVE_OR_INVALID_ACCOUNTS` (`errors.rs:2161-2163`), générique, **qui ne nomme aucune ligne**.

**Correction de la passe 1** : la version initiale de cette décision affirmait « sans re-validation au posting, une facture validée pourrait créditer un compte archivé ». **C'est faux** — `active = TRUE` est inconditionnel. Le vrai trou est triple : `postable`, `account_type`, et l'illisibilité du message.

D'où :

- **À la saisie** (création `invoices.rs:459` et modification `:816`, dans la transaction, au même point d'accroche que `projects::validate_taggable_in_tx`) : rejet si le compte n'appartient pas à la société, ou n'est pas de type `Revenue`, ou n'est pas `active`, ou n'est pas `postable` (sous réserve de D3-bis).
- **Au posting** (`validate_invoice`) : **re-validation** des quatre critères, sur le modèle de la re-validation du projet analytique (`invoices.rs:1290-1310`) — `SELECT` simple **sans `FOR UPDATE`**, car le verrou sur la ligne facture est déjà détenu et prendre le sentinel `companies` ici créerait l'inversion ABBA que le commentaire de 19-4 documente. La race résiduelle d'archivage (fenêtre ms) est la **même dette LOW acceptée** qu'en 19-3/19-4 — à reprendre explicitement en commentaire.

### D3-bis — Le compte explicitement égal au défaut société est exempté de la contrainte `postable`

**Le problème** : `company_invoice_settings::update` valide `default_revenue_account_id` sur existence + société + `active` + `account_type`, **jamais `postable`** (`crates/kesh-api/src/routes/company_invoice_settings.rs`, `validate_account`). Un compte non-postable peut donc être le défaut société et fonctionner parfaitement au posting (`enforce_postable = false`). Le cas est atteignable sans intention : `effective_postable` force `postable = false` dès qu'un compte acquiert un enfant actif (`accounts.rs:126-131`).

**Sans exemption** : une ligne qui ne précise **rien** (NULL) se poste sans problème sur ce compte ; la **même** ligne, si l'utilisateur sélectionne explicitement ce même compte — le geste le plus naturel — est rejetée en `400`. Résultat comptable identique, verdict opposé.

**Décision** : la validation `postable` (saisie **et** posting) exempte le compte dont l'id est égal à `settings.default_revenue_account_id` **courant**. C'est le pattern `exempt_ids` déjà utilisé par `validate_lines_accounts_in_tx` (`journal_entries.rs:70-93`). Les trois autres critères (société, `active`, `account_type = Revenue`) restent appliqués sans exception.

**Invariant à tester** : `revenue_account_id = NULL` et `revenue_account_id = settings.default_revenue_account_id` produisent **le même verdict de validation** et **la même écriture**.

**Comment obtenir le défaut à la saisie — piège de régression.** `invoices::create` et `invoices::update` ne lisent **pas** `company_invoice_settings` aujourd'hui (vérifié : aucune occurrence dans `invoices.rs:438-500`). Le helper de T4 le lit par un `SELECT` **nu**, sans verrou ni lazy-create :

```sql
SELECT default_revenue_account_id FROM company_invoice_settings WHERE company_id = ?
```

**Ne PAS utiliser `company_invoice_settings::get_or_create_default_in_tx` sur le chemin de saisie.** Motif — corrigé en passe 6 : cette fonction (`company_invoice_settings.rs:87-107`) **écrit** (`INSERT IGNORE`) et prend un `SELECT … FOR UPDATE` sur `company_invoice_settings`, soit un verrou supplémentaire dans une transaction dont l'ordre est déjà contraint (`invoices.rs:806-812`, « ordre de verrous global companies → projects → invoices »). *(La version de la passe 3 invoquait en plus « elle échoue en `InactiveOrInvalidAccounts` si le défaut société est archivé » — **c'est faux** : cette fonction ne JOINe pas `accounts` et ne contrôle pas `active` ; le JOIN de `:482` est dans `insert_with_defaults_in_tx`, cf. AC8-bis. Le motif du verrou et de l'écriture suffit, et il est exact.)* Ligne absente ou colonne `NULL` → `exempt_ids` vide, la contrainte `postable` s'applique alors sans exception.

Au **posting**, en revanche, `settings` est déjà chargé (`invoices.rs:1310-1312`) : le réutiliser, ne pas relire.

**Fenêtre assumée (dette LOW documentée).** L'exemption est indexée sur `settings.default_revenue_account_id` **au moment de chaque contrôle**. Si l'administrateur change le défaut entre la saisie et le posting, une ligne pointant explicitement l'**ancien** défaut non-postable est acceptée à la saisie puis rejetée au posting — même donnée, verdicts opposés. Comportement assumé : l'alternative (figer l'exemption dans la ligne à la saisie) rendrait le brouillon dépendant d'un état de configuration périmé, ce que D2 refuse par ailleurs. Le message du posting doit **le rendre lisible** : « Ligne {n} : le compte {numéro} n'est pas imputable et n'est plus le compte de produit par défaut de la société — choisissez un autre compte ». Hors périmètre des tests d'AC18 (fenêtre de configuration, aucune correction comptable en jeu).

### D4 — Ventilation : `BTreeMap<i64, Decimal>` par compte effectif, montants `> 0`, tri par `account_id`

Le helper agrège les `line_total` par **compte effectif** (compte de la ligne, ou défaut société si `NULL`), n'émet une ligne de crédit que si le montant agrégé est `> 0`, et itère en ordre croissant d'`account_id`.

**Motif** : reproduit exactement le patron déjà en place pour la TVA par taux (`BTreeMap` ASC), donc déterminisme des écritures et tests stables.

**Conséquence explicite de l'agrégation par compte effectif** : des lignes `NULL` et des lignes pointant explicitement sur le défaut société **fusionnent en une seule ligne de crédit**. Ce n'est pas un effet de bord à corriger, c'est le comportement voulu (cf. l'invariant de D3-bis).

**Preuve d'équilibre** (à reprendre dans la docstring) : le débit créance vaut `total_ht + total_vat`. La ventilation ne touche **que** la répartition de `total_ht`, et `Σ_comptes (Σ_lignes line_total) = Σ_lignes line_total = total_ht` **exactement** — aucun arrondi n'intervient sur le HT (l'arrondi half-up par ligne ne concerne que la TVA, cf. `kesh_core::accounting::vat::line_vat_amount`). L'équilibre garanti par construction dans la docstring actuelle est donc **préservé**.

### D4-bis — Facture entièrement à zéro : rejet applicatif explicite (bug latent pré-existant, fermé ici)

**Bug pré-existant vérifié** : une facture dont toutes les lignes ont `unit_price = 0` (autorisé — `validate_line` exige `quantity > 0` mais `unit_price >= 0`) donne `total_ht = 0` et `total_vat = 0`. Le helper pousse alors la ligne `[0]` avec `debit = 0, credit = 0`, ce qui viole `chk_jel_debit_credit_exclusive CHECK ((debit = 0 AND credit > 0) OR (debit > 0 AND credit = 0))` (`migrations/20260412000001_journal_entries.sql:46`) → **500 SQL** aujourd'hui, avant la story.

Le filtre `> 0` de D4 **change la forme** du défaut (la ligne de crédit produit disparaît au lieu d'être à zéro) sans le corriger. Puisque la story réécrit précisément ce code, on ferme le trou :

**Décision** : `validate_invoice` rejette une facture dont `total_ht + total_vat == 0` avec une erreur métier actionnable (`400`, « facture sans montant »), avant l'appel à `journal_entries::create_in_tx`. Symétrique côté avoir. Documenté dans le Change Log conformément à la § « Issue Tracking Rule » (bug découvert dans le flux de dev d'une story liée → corrigé dans la story, pas d'issue GitHub).

### D5 — Les avoirs sont DANS le périmètre : `credit_note_lines` porte aussi le compte, la contre-passation ventile

*(Confirmé par Guy le 2026-07-26, question ouverte n°1 tranchée.)*

`credit_note_lines` reçoit également `revenue_account_id BIGINT NULL`, copié depuis la ligne de facture lors de la création de l'avoir. `generate_credit_note_journal_lines` prend `lines: &[(Decimal, Decimal, Option<i64>)]` et **débite par compte**.

**Motif — c'est le point le plus grave de la story, et l'issue #152 ne le mentionne pas.** `generate_credit_note_journal_lines` (`credit_notes.rs:139`) est documenté comme l'« **inverse exact** » du helper facture et relit aujourd'hui `settings.default_revenue_account_id` (`:280-282`). Livrer la ventilation côté facture **sans** toucher l'avoir produirait ceci : une facture créditée sur 3200 serait extournée sur 3000. Les deux écritures ne s'annulent plus → **résidu permanent au crédit de 3200 et au débit de 3000**, invisible au bilan (l'équation reste équilibrée) mais **faux au compte de résultat**. Corruption comptable silencieuse ; elle doit être fermée dans la même story que la cause.

La colonne est portée par le snapshot (plutôt que relue depuis la facture d'origine) pour garder l'avoir **auto-descriptif** — c'est déjà le parti pris de `credit_note_lines`, qui duplique toutes les autres colonnes de ligne.

### D5-bis — Compte devenu inactif entre la validation de la facture et l'émission de l'avoir : l'avoir échoue, avec un message précis

**Le chemin** : `accounts::archive` (`accounts.rs:472`) ne consulte pas `invoice_lines`. Un compte ventilé sur une facture validée peut donc être archivé, puis on tente d'émettre l'avoir.

**Ce qui se passe aujourd'hui sans décision** : `create_credit_note` recopie les comptes du snapshot et appelle `create_in_tx`, dont la garde `active = TRUE` rejette → `400 INACTIVE_OR_INVALID_ACCOUNTS` générique, **sans nommer la ligne ni le compte**.

**Options écartées** :
- *Replier sur le défaut société* — casse la propriété « inverse exact » et recrée exactement le résidu que D5 combat. **Non.**
- *Poster quand même sur le compte archivé* — impossible, la garde `active` de `create_in_tx` est inconditionnelle.

**Décision** : l'émission de l'avoir **échoue**, mais avec un message actionnable nommant la ligne et le compte (« la ligne N référence le compte X, archivé — réactivez-le pour pouvoir émettre l'avoir »). La correction comptable prime sur la commodité : un avoir sur le mauvais compte est pire qu'un avoir bloqué.

**Portée de la re-validation côté avoir** : `active` **uniquement**. On ne re-vérifie ni `postable` ni `account_type` — la contre-passation doit viser les **mêmes** comptes que l'écriture d'origine, quelle qu'ait été leur évolution de configuration entre-temps. Seule l'inactivité est bloquante, parce qu'elle est bloquante en base.

### D6 — Validation batchée, message nommant TOUTES les lignes en défaut

Une facture peut porter jusqu'à `MAX_LINES = 200` lignes (`routes/invoices.rs:58`) sur autant de comptes distincts.

**Décision** : la validation des comptes de ligne (saisie **et** posting) se fait en **une seule requête** `SELECT … WHERE company_id = ? AND id IN (…)` ramenant `(id, active, postable, account_type)`, ids dédupliqués — pattern `projects::validate_taggable_in_tx` (`projects.rs:87-121`). Le résultat est ensuite croisé avec la table `position → account_id` pour construire un message listant **toutes** les lignes en défaut, pas seulement la première.

**Motif** : une implémentation naïve en boucle ferait jusqu'à 200 requêtes DB par création/modification/validation de facture. Et le message « la ligne concernée » au singulier ne dit rien du cas où plusieurs lignes sont invalides simultanément — cas courant si un compte partagé est archivé.

### D7 — Style d'erreur : convention du fichier, PAS un code `FailedProposal`

Les erreurs de validation de ligne suivent la convention déjà en place dans `routes/invoices.rs` : `AppError::Validation(format!("Ligne {} : …", index + 1))` (`:370-410`).

**Correction de la passe 1** : la version initiale exigeait « un code d'erreur canonique dédié (pas de `format!` interpolé) ». C'était une **mauvaise application du pattern batch** de `CLAUDE.md` : l'interdiction du `format!` porte sur `FailedProposal.error_code` des endpoints `{ accepted, failed }`, pas sur les messages `AppError::Validation`. Le pattern batch **ne s'applique pas ici** — la création/modification de facture rejette la requête entière en `400` en amont de tout traitement per-ligne, ce qui relève des exceptions explicites de la règle (« body parse fail / schéma invalide »).

---

## Acceptance Criteria

### A. Base de données

- **AC1** — Migration ajoutant `revenue_account_id BIGINT NULL` à `invoice_lines`, `CONSTRAINT fk_invoice_lines_revenue_account FOREIGN KEY (revenue_account_id) REFERENCES accounts(id) ON DELETE RESTRICT` (convention **unanime** du dépôt : les 11 FK vers `accounts` sont toutes `ON DELETE RESTRICT`), + index sur la colonne.
- **AC2** — Même ajout sur `credit_note_lines` (D5).
- **AC2-bis** — **Backfill du parc existant (D2-bis)**, dans la même migration : `invoice_lines` des factures `status = 'validated' AND journal_entry_id IS NOT NULL` reçoivent le compte **effectivement crédité par leur écriture**, identifié par le **critère d'unicité en trois conditions** de D2-bis (crédit non nul ; compte hors `E = { receivable, vat_payable }` en comparaison **NULL-safe `<=>`** ; montant **égal à `invoices.total_amount`**) ; miroir sur `credit_note_lines` depuis l'écriture de contre-passation (`debit`, `credit_notes.total_amount`). **Ne PAS** backfiller depuis `settings.default_revenue_account_id` courant (fabriquerait la corruption si le défaut a changé par le passé). **Ne PAS** identifier la ligne par `line_order` seul (réfuté en passe 5 : l'écriture d'une facture validée est éditable, `line_order` est réattribué par `journal_entries::update`). **Ne PAS** écrire la condition (2) avec `<>` / `NOT IN` (réfuté en passe 6 : les colonnes TVA sont `NULL` partout, le backfill no-operait sur tout le parc).
  **Le backfill est délibérément incomplet, et c'est la spécification** : toute ligne dont le compte n'est pas identifiable sans ambiguïté — écriture éditée, zéro ou plusieurs candidats — **reste `NULL`**, est **dénombrable**, et la migration **réussit**. Une post-condition « aucune ligne validée ne reste `NULL` » serait **fausse** et pousserait le dev à relâcher le critère jusqu'à ce qu'elle passe, c'est-à-dire à écrire un compte arbitraire sur des données comptables réelles — l'inverse exact de l'objectif.
  **Post-conditions réellement testées**, sur base pré-remplie (`migrations_fresh_install` ne prouve rien ici : rien à backfiller) :
  1. facture validée à écriture **canonique** → ligne backfillée avec le compte crédité par l'écriture, **même si `settings.default_revenue_account_id` a changé depuis** (sans quoi le test passerait aussi avec un backfill depuis le défaut courant) ;
  2. facture validée dont l'écriture a été **éditée** de sorte qu'aucune ligne ne crédite exactement `total_amount` (montant du produit modifié) → la ligne reste `NULL`, la migration **réussit**, et la **requête de diagnostic** de D2-bis retourne le compte attendu (c'est elle que le test assert — il n'y a ni table ni log à interroger) ;
  3. facture `draft` → reste `NULL` (D2). *(Le cas « facture validée **sans écriture** » n'est PAS testé : il est **inconstructible** — `chk_invoices_validated_has_je` (`migrations/20260417000002_…sql`) l'interdit en base, et `fk_invoices_journal_entry … ON DELETE RESTRICT` empêche de supprimer l'écriture après coup. Tenter la fixture ne produit qu'une violation de CHECK.)* ;
  4. **société dont `default_vat_payable_account_id` est `NULL`** (c'est-à-dire **toute** société non configurée manuellement — le cas par défaut, cf. D2-bis), facture validée à écriture canonique → la ligne **est** backfillée. C'est le test qui attrape la propagation `NULL` de la condition (2) ; sans lui, un backfill qui no-ope intégralement est indiscernable d'un backfill conservateur.
- **AC3** — Migration **non-breaking** (`ADD COLUMN` nullable + index + FK) → **pas** de bump `kesh_version_min_required` (politique P1/P2), donc **pas** de bump de version Cargo (P2-bis). Le vérifier explicitement.
- **AC4** — `docs/migrations-idempotence-audit.md` : ligne ajoutée au tableau détaillé avec verdict et justification, **ET** récapitulatif agrégé de bas de fichier mis à jour en cohérence — `Total` (`:68`, actuellement 55) et `Idempotence tracked-by-sqlx` (`:70`, actuellement 44) passent chacun à +1 (verdict attendu `tracked-by-sqlx`, la migration n'utilisant pas `IF NOT EXISTS`). Garde-fou **P5**.

### B. Backend — entité et tous les sites de colonnes

- **AC5** — L'entité `InvoiceLine` porte `revenue_account_id: Option<i64>`, et **les 3 sites qui listent les colonnes `invoice_lines`** sont mis à jour — sans quoi `sqlx::query_as` échoue au runtime :
  1. `LINE_COLUMNS` (`invoices.rs:39`) — utilisé par `insert_lines` (`:386`) et `fetch_lines` (`:425`) ;
  2. `list_all_lines_by_company` (`invoices.rs:1937-1944`) — **liste en dur avec préfixes `il.`**, alimente l'export ZIP global ;
  3. `create_credit_note` (`credit_notes.rs:266-268`) — **liste en dur**, lit `invoice_lines` pour le snapshot d'avoir.
  `invoice_snapshot_json` (`invoices.rs:51-60`) inclut le compte dans le snapshot d'audit.
- **AC5-bis** — Côté avoir, l'entité `CreditNoteLine` (`crates/kesh-db/src/entities/credit_note.rs:45`) porte le champ, et **les 4 sites** suivants sont mis à jour : `credit_note_snapshot_json` (`credit_notes.rs:36-56`), `fetch_credit_note_lines` (`:63-64`), le second `SELECT` de `get()` (`:90-91`), et l'`INSERT credit_note_lines` (`:376-390`). Il n'existe **pas** de constante `LINE_COLUMNS` côté avoir — soit en introduire une, soit traiter les 3 `SELECT` un par un ; ne pas en oublier.

  **Nature exacte du changement, site par site** — chaque `SELECT` ajoute `revenue_account_id` à sa liste de colonnes ; l'`INSERT` (`:376-390`) l'ajoute **à la fois** à la liste de colonnes **et** à la chaîne de `.bind()` (`.bind(line.revenue_account_id)`), et son `VALUES` gagne un `?` ; les deux `*_snapshot_json` ajoutent la clé au JSON. Un `SELECT` mis à jour sans son `INSERT`, ou l'inverse, ne casse pas la compilation.

- **AC5-ter** — **Décompte de référence : 8 sites au total** — 6 listes de colonnes SQL (`invoices.rs:39` / `:1937` / `credit_notes.rs:266` / `:63` / `:90` / `:376`) + 2 snapshots d'audit (`invoices.rs:51` / `credit_notes.rs:36`). S'y ajoute **1 site d'appel** (`credit_notes.rs:320-328`, cf. AC11) qui, lui, est attrapé par le compilateur. Si le décompte ne tombe pas sur 8 + 1, il manque quelque chose.

### C. Backend — API

- **AC6** — `CreateInvoiceLineRequest` (`routes/invoices.rs:65-71`) accepte `revenueAccountId: Option<i64>` **portant `#[serde(default)]`**. Sans cet attribut, un `Option<T>` reste **obligatoire dans le JSON** en serde : l'omission totale de la clé ferait échouer la désérialisation, cassant toute intégration PAT existante à chaque création de facture. Suivre le style de `CreateInvoiceRequest` (`:78`, `:80`, `:85`) et **pas** celui des 4 champs voisins de `CreateInvoiceLineRequest`, tous obligatoires. Idem pour le DTO de modification. La réponse de lecture restitue le champ. Un test couvre le payload **sans la clé** et le payload avec `"revenueAccountId": null` — les deux valent `NULL`.
- **AC7** — Validation à la saisie (création `invoices.rs:459` **et** modification `:816`) : société, `active`, `account_type = Revenue`, `postable` (avec l'exemption D3-bis). Batchée en une requête (D6), message nommant toutes les lignes en défaut, style `AppError::Validation` (D7).
- **AC8** — Re-validation au posting dans `validate_invoice`, `SELECT` sans verrou sur le modèle 19-4 (`invoices.rs:1290-1310`), couvrant les **quatre** critères — dont `account_type`, que `create_in_tx` ne vérifie **jamais** (D3). L'échec nomme la ou les lignes concernées. Le commentaire reprend l'accepted risk ABBA / race d'archivage de 19-3/19-4.
- **AC8-bis** — **L'ensemble re-validé est celui des comptes EFFECTIVEMENT postés**, pas seulement celui des comptes explicites : `{ comptes de ligne non-NULL } ∪ { settings.default_revenue_account_id, si au moins une ligne est NULL }`. Une facture dont **toutes** les lignes sont `NULL` doit donc quand même voir son compte par défaut re-validé — sinon le seul cas qui existe aujourd'hui en production échappe entièrement à AC8, et un défaut archivé retombe sur le `400 INACTIVE_OR_INVALID_ACCOUNTS` générique que cette story existe pour éliminer.
  - Critères appliqués au compte par défaut : **`account_type = Revenue` ET `active`.**
    - **Correction de la passe 3, rétablie en passe 6.** La passe 3 avait retiré `active` en le déclarant « garanti en amont, cas inatteignable », sur cette preuve : « la config est chargée par `get_or_create_default_in_tx` dont le chemin *ligne existante* JOINe `accounts av … AND av.active = TRUE` (`company_invoice_settings.rs:482`) ». **L'ancre était attribuée à la mauvaise fonction.** `get_or_create_default_in_tx` occupe les lignes **87-107** et son corps entier est un `INSERT IGNORE … (company_id) VALUES (?)` suivi d'un `SELECT {COLUMNS} FROM company_invoice_settings WHERE company_id = ? FOR UPDATE` — **aucun JOIN, aucun contrôle d'`active`**. Le JOIN de la ligne `:482` appartient à `insert_with_defaults_in_tx` (fn `:403`), appelée uniquement depuis `routes/onboarding.rs:720`. Les passes 4 et 5 ont toutes deux « vérifié » cette ancre en lisant la ligne sans borner la **fonction** qui la contient.
    - Conséquence : le défaut société archivé **n'est pas** rejeté en amont. `validate_invoice` construit l'écriture, et c'est `create_in_tx` → `validate_lines_accounts_in_tx` qui rejette, en `400 INACTIVE_OR_INVALID_ACCOUNTS` **générique et anonyme** — précisément le message qu'AC8-bis existe pour éliminer, sur précisément le cas qu'AC8-bis dit couvrir. `active` est donc un trou **réel** sur le compte par défaut, au même titre qu'`account_type`, et doit être re-validé ici pour produire un message nommant « le compte de produit par défaut de la société ».
    - **`postable` reste exempté** (même arbitrage que D3-bis).
    - `account_type` est également un **vrai** trou : la route de configuration le vérifie à la pose (`routes/company_invoice_settings.rs:117-120`) mais rien ne le revérifie après un retypage par `accounts::update`.
  - Le message d'erreur le désigne explicitement comme « le compte de produit par défaut de la société », **pas** par un numéro de ligne — aucune ligne ne le porte.

### D. Backend — moteur comptable

- **AC9** — `generate_invoice_journal_lines` ventile le crédit produit : une ligne de crédit **par compte effectif**, montants `> 0`, tri `account_id` ASC (D4). La ligne `[0]` débit créance et les lignes TVA par taux sont **inchangées**. Lignes `NULL` et lignes pointant explicitement le défaut société fusionnent en une seule ligne.
- **AC9-bis** — **Matérialisation à la validation (D2)** : `validate_invoice` écrit le compte effectif dans `invoice_lines.revenue_account_id` pour toute ligne `NULL`, dans la **même transaction** que la création de l'écriture, **avant** l'appel à `create_in_tx`. **Post-condition vérifiable, bornée à son domaine réel (passe 6)** : aucune ligne de **la facture que le test vient de valider** n'a `revenue_account_id IS NULL`. **Surtout PAS** un `COUNT(*) = 0` global sur `invoice_lines` filtré `status = 'validated'` : sur base pré-remplie — celle qu'AC2-bis exige d'utiliser — il **échouerait** à cause des `NULL` résiduels que le backfill laisse volontairement, et ne laisserait au dev que deux issues, relâcher le critère d'unicité jusqu'à écrire un compte arbitraire sur des données réelles, ou perdre du temps à découvrir la contradiction.
  **La copie en mémoire doit être mutée aussi — l'`UPDATE` SQL seul ne suffit pas.** `lines_before` est chargé par `fetch_lines` à l'étape (1), donc **avant** la matérialisation, et il est réutilisé deux fois après elle :
  - `invoice_snapshot_json(&invoice_after, &lines_before)` pour le snapshot d'audit `"after"` (`invoices.rs:1448`) — qu'AC9-bis exige **post**-matérialisation ;
  - `ValidatedInvoice.lines` (`invoices.rs:1478`), **délibérément non re-fetché** post-commit (décision antérieure documentée `invoices.rs:1114`, « évite une fenêtre de race sur les lignes ») et rendu tel quel dans la réponse HTTP.
  Sans mutation en mémoire, l'audit et la réponse de l'endpoint de validation afficheraient `revenueAccountId: null` pour des lignes que la base vient de matérialiser — contredisant l'invariant même que la story introduit. **Muter les entrées de `lines_before` dont `revenue_account_id` était `NULL`** juste après l'`UPDATE`, **sans** réintroduire de re-fetch DB (la garantie anti-race de `invoices.rs:1114` doit être préservée).
- **AC10** — La section `# Équilibre par construction` de la docstring (`invoices.rs:1137-1142`) est **réécrite** — pas seulement complétée — pour couvrir la ventilation par compte en plus du filtre par taux, en reprenant l'argument de D4. L'hypothèse `F-OPUS-2` et la section `# Erreurs` restent à jour.
- **AC11** — `generate_credit_note_journal_lines` (`credit_notes.rs:139`) débite par compte, en miroir exact (D5) ; sa signature passe à `lines: &[(Decimal, Decimal, Option<i64>)]`. Sa docstring « inverse exact » reste vraie et est mise à jour. **Le site d'appel est mis à jour en conséquence** : `create_credit_note` (`credit_notes.rs:320-328`) construit aujourd'hui des paires via `.map(|l| (l.line_total, l.vat_rate))` — il doit produire des **triplets** `(l.line_total, l.vat_rate, l.revenue_account_id)`. Le paramètre scalaire `revenue_account_id` du helper devient le **repli** appliqué aux triplets dont le 3ᵉ membre est `None`, exactement comme côté facture.
- **AC11-bis** — Comportement D5-bis implémenté : compte du snapshot devenu `active = FALSE` → échec de l'émission de l'avoir avec message nommant ligne et compte. `postable` et `account_type` **ne sont pas** re-vérifiés côté avoir.
- **AC12** — **Non-régression, ancrée sur l'existant** : les tests unitaires actuels de `generate_invoice_journal_lines` (`invoices.rs:1996-2165`, 8 sites d'appel) passent **sans modification de leurs assertions** après la ventilation (leurs fixtures ont toutes `revenue_account_id = None`). Seule l'adaptation de signature est tolérée.
- **AC12-bis** — **Non-régression de la saisie** (corollaire du piège de D3-bis) : la **création** et la **modification** d'un brouillon réussissent pour une société dont `settings.default_revenue_account_id` est **archivé** ou `NULL`, tant qu'aucune ligne ne référence explicitement un compte invalide.
  **Ce test ne suffit PAS à attraper l'usage accidentel de `get_or_create_default_in_tx`** — précision de la passe 6, qui corrige une affirmation de la passe 3. Cette fonction ne contrôlant pas `active` (cf. AC8-bis), elle **n'échoue pas** sur un défaut archivé : le test passe que le dev ait utilisé le `SELECT` nu ou elle. C'est exactement le « test qui passe systématiquement et ne prouve rien » dénoncé en AC17 et AC20. AC12-bis reste utile comme non-régression fonctionnelle (le chemin le plus fréquent de l'application doit marcher), mais la garantie « pas de `get_or_create_default_in_tx` à la saisie » relève de la **revue de code sur T5**, pas de ce test. À défaut, l'ancrer sur ce qui est réellement observable : absence d'écriture et de verrou sur `company_invoice_settings` pendant `create` / `update`.
- **AC13** — Le rapport TVA n'est **pas** affecté : `kesh-report/src/vat_report.rs` ne lit que `default_vat_payable_account_id` / `default_vat_recoverable_account_id`, jamais un compte de produit. Un test d'intégration sur une facture **multi-comptes × multi-taux** vérifie que `reconciliation_status` reste `ok` (fichier `crates/kesh-report/tests/vat_report_reconciliation.rs`, nouveau cas).
- **AC13-bis** — D4-bis : `validate_invoice` (et l'émission d'avoir) rejettent une pièce dont `total_ht + total_vat == 0` avec une erreur métier `400` actionnable, au lieu du `500` SQL actuel sur `chk_jel_debit_credit_exclusive`.

### E. Exports

- **AC14** — `serialize_invoice_lines_csv` (`csv_tables.rs:459`) expose `revenue_account_id` dans l'en-tête et les enregistrements. Le test `crates/kesh-api/tests/exports_global_e2e.rs` est étendu pour vérifier la nouvelle colonne d'`invoice_lines.csv`.
- **AC14-bis** — **Aucun export CSV des lignes d'avoir n'existe** : `grep -n "credit_note" crates/kesh-api/src/exports/` revient vide, et les 20 entrées du ZIP (`exports_global_e2e.rs:621-634`) n'en contiennent pas. Aucune action requise de ce côté. Les compteurs existants ne sont **pas** affectés par un `ADD COLUMN` : ni `assert_eq!(entries.len(), 20)` (nombre de **fichiers**), ni `TABLES_TO_TRUNCATE` (23 **tables**) dans `admin_backup_e2e.rs`. La sauvegarde générique (`crates/kesh-db/src/backup.rs`) lit les colonnes dynamiquement via `non_generated_columns` — **auto-adaptée**, aucune modification.
  **Condition de validité de cette clause (passe 6)** : elle tient parce que **cette story ne crée aucune table**. Le décompte du backfill est restitué par une requête de diagnostic, pas par une table de rapport (D2-bis) — précisément pour ne pas déclencher `backup_inventory_matches_schema` (`backup.rs:577-606`), qui imposerait de modifier `TABLES_TO_TRUNCATE` et ferait entrer la table dans le périmètre de l'export/import d'installation. Si une passe ultérieure réintroduisait une table, cette clause et le périmètre d'export devraient être révisés ensemble.

### F. Tests & gate

- **AC15** — Tests unitaires du helper facture : mono-compte (non-régression AC12), multi-comptes, multi-comptes × multi-taux, ligne à montant nul filtrée, lignes `NULL` + explicite-même-compte fusionnées, ordre déterministe par `account_id`.
- **AC16** — Tests unitaires du helper avoir : miroir strict de AC15.
- **AC17** — Test d'intégration **pivot de D5**, en **deux** cas :
  1. facture ventilée sur ≥ 2 comptes puis avoir total → **les deux écritures s'annulent compte par compte** (agrégat par `account_id` de l'écriture facture + celle de l'avoir = 0 sur chaque compte) ;
  2. **facture à lignes toutes `NULL`, puis `settings.default_revenue_account_id` MODIFIÉ, puis avoir** → l'avoir débite le compte **effectivement crédité par la facture**, pas le nouveau défaut. Ce second cas **doit échouer** si la matérialisation d'AC9-bis n'est pas implémentée — c'est sa raison d'être. Un test qui ne change pas le défaut entre les deux passe systématiquement et ne prouve rien.
- **AC18** — Tests d'intégration : compte invalide à la saisie (création **et** modification) ; compte devenu non-`postable` au posting ; compte **retypé** au posting (le trou que `create_in_tx` ne couvre pas) ; compte archivé au posting ; compte archivé entre validation et avoir (AC11-bis) ; plusieurs lignes invalides simultanément (le message les nomme toutes).
- **AC18-bis** — Test d'AC8-bis, **trois** cas :
  1. facture dont **toutes** les lignes sont `NULL`, `settings.default_revenue_account_id` **retypé** `Revenue → Expense` entre le brouillon et la validation → échec nommant « le compte de produit par défaut de la société » ;
  2. même défaut rendu **non-postable** → la validation **passe** (exemption D3-bis), l'écriture est générée normalement ;
  3. **même défaut archivé** (`active = FALSE`) → échec nommant lui aussi « le compte de produit par défaut de la société », **et non** le `400 INACTIVE_OR_INVALID_ACCOUNTS` générique. *(Cas rétabli en passe 6 : la passe 3 l'avait retiré en le croyant rejeté en amont par `get_or_create_default_in_tx` — cette fonction ne contrôle pas `active`, cf. AC8-bis. L'assertion porte sur le **message**, c'est elle qui distingue AC8-bis implémenté de AC8-bis absent.)*
- **AC19** — Test D3-bis : `default_revenue_account_id` pointant sur un compte **non-postable** ; une ligne `NULL` et une ligne le désignant explicitement produisent le même verdict et la même écriture.
- **AC20** — Test D1 : `settings.default_revenue_account_id` **≠** compte portant le rôle `DefaultRevenue` (deux comptes distincts) ; une ligne sans compte se poste sur `settings.default_revenue_account_id`.
- **AC21** — Test D4-bis : facture entièrement à zéro → `400` métier, pas `500`.
- **AC23** — Gate « Test Locally First » backend complet vert (`cargo fmt --all -- --check`, `cargo build --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`). Le gate **runtime complet** est requis si le doute subsiste sur `min_required` (P2-bis) — ici la migration est non-breaking, mais les suites `migrations_fresh_install` et `admin_backup_e2e` doivent passer.
- **AC24** — CHANGELOG `[Non publié]` : entrée orientée utilisateur. Le README et les manuels LaTeX sont traités en **16-1b** (le comportement n'est pas visible utilisateur tant que l'UI n'est pas livrée).

---

## Tasks / Subtasks

- [ ] **T1** — Migration `invoice_lines.revenue_account_id` + `credit_note_lines.revenue_account_id` + index + FK `ON DELETE RESTRICT` + **backfill conservateur du parc existant** depuis les écritures, sous le critère d'unicité en trois conditions de D2-bis (montant `= invoices.total_amount` inclus, condition (2) en `<=>` NULL-safe), les lignes non identifiables restant `NULL` et dénombrables par la requête de diagnostic (D2-bis, AC2-bis) ; ligne du tableau **et** compteurs agrégés de `docs/migrations-idempotence-audit.md`. **Verdict : `tracked-by-sqlx`, et le backfill est intrinsèquement IDEMPOTENT** — garde `revenue_account_id IS NULL` + critère déterministe, donc re-jeu sans effet, exactement comme les backfills de `20260628000001_supplier_invoices.sql` et `20260722000001_accounts_role_postable.sql` (« **12 UPDATE de backfill** en revanche intrinsèquement idempotents »), tous deux classés `tracked-by-sqlx`. Le `tracked-by-sqlx` est justifié par les `ADD COLUMN` **sans** `IF NOT EXISTS` (erreur 1060 au re-jeu hors sqlx), pas par le backfill. **Ne pas** écrire « non idempotente » : le fichier maintient l'invariant « Idempotence `no` : 0 » (`migrations-idempotence-audit.md:71`) et le verdict `no` ferait diverger les compteurs d'AC4 (AC1-AC4, AC2-bis).
- [ ] **T2** — Entités `InvoiceLine` / `CreditNoteLine` + **les 8 sites** listés en AC5 / AC5-bis (6 listes de colonnes SQL + 2 snapshots d'audit), décompte de référence en AC5-ter (AC5, AC5-bis, AC5-ter).
- [ ] **T3** — API : DTOs création/modification avec `#[serde(default)]` + réponse de lecture (AC6 — la clause de test des deux formes de payload y est déjà portée).
- [ ] **T4** — Helper de validation batchée des comptes de ligne, réutilisable saisie + posting, avec exemption D3-bis et message multi-lignes (D6, D3-bis).
- [ ] **T5** — Branchement de T4 à la saisie (`invoices.rs:459` création, `:816` modification), en lisant le défaut par un `SELECT` nu — **jamais** `get_or_create_default_in_tx` (D3-bis) (AC7, AC12-bis).
- [ ] **T6** — `generate_invoice_journal_lines` : ventilation `BTreeMap` + docstring réécrite (AC9, AC10).
- [ ] **T7** — Branchement de T4 au posting dans `validate_invoice`, `SELECT` sans verrou, commentaire ABBA ; ensemble re-validé = comptes de ligne ∪ défaut si ≥ 1 ligne `NULL` ; **matérialisation du compte effectif** avant `create_in_tx` (D2) (AC8, AC8-bis, AC9-bis).
- [ ] **T8** — `generate_credit_note_journal_lines` en miroir + copie du compte à la création de l'avoir + garde D5-bis (AC11, AC11-bis).
- [ ] **T9** — Rejet des pièces à montant total nul (AC13-bis, AC21).
- [ ] **T10** — Export CSV `invoice_lines` + extension du test `exports_global_e2e` (AC14).
- [ ] **T11** — Tests unitaires (AC15, AC16) et d'intégration (AC12-bis, AC17 **les deux cas**, AC18, AC18-bis, AC19, AC20, AC21), dont le cas TVA multi-comptes × multi-taux (AC13).
- [ ] **T12** — CHANGELOG (AC24) + gate backend complet (AC23).

**Ordre conseillé** : T1 → T2 → T6 (le helper d'abord, testable en isolation) → T4 → T5 → T7 → T8 → T9 → T3 → T10 → T11 → T12.

---

## Dev Notes

### Ancres ground-truth (re-vérifiées en passe 1 de `validate`, 2026-07-26, commit `ef6cdf52`)

| Élément | Emplacement |
|---|---|
| Schéma `invoice_lines` | `crates/kesh-db/migrations/20260416000001_invoices.sql:41` |
| Schéma `credit_note_lines` | `crates/kesh-db/migrations/20260627000001_credit_notes.sql:63` |
| Contrainte `chk_jel_debit_credit_exclusive` | `crates/kesh-db/migrations/20260412000001_journal_entries.sql:46` |
| Convention FK vers `accounts` (11 sites, toutes RESTRICT) | `grep -rn "REFERENCES accounts" crates/kesh-db/migrations/` |
| `LINE_COLUMNS` | `crates/kesh-db/src/repositories/invoices.rs:39` |
| `invoice_snapshot_json` | `invoices.rs:51` |
| `insert_lines` / `fetch_lines` | `invoices.rs:386` / `:425` |
| **`list_all_lines_by_company` (colonnes EN DUR, export ZIP)** | `invoices.rs:1937-1944` |
| Doc + code du helper d'écriture facture | `invoices.rs:1120-1155` (doc), `:1156` (fn), `:1137-1142` (équilibre) |
| Tests unitaires existants du helper (8 appels) | `invoices.rs:1996-2165` |
| `validate_invoice` | `invoices.rs:1245` ; lecture du défaut `:1319-1320` ; appel helper `:1383-1387` |
| Re-validation projet au posting (patron D3) | `invoices.rs:1290-1310` |
| Points d'accroche validation à la saisie | `invoices.rs:459` (création), `:816` (modification) |
| `projects::validate_taggable_in_tx` (patron batch D6) | `crates/kesh-db/src/repositories/projects.rs:87-121` |
| `validate_lines_accounts_in_tx` (ce que la garde couvre) | `crates/kesh-db/src/repositories/journal_entries.rs:65-114` ; appel `:210-232` |
| Mapping `InactiveOrInvalidAccounts` → 400 | `crates/kesh-api/src/errors.rs:2161-2163` |
| Helper avoir « inverse exact » | `crates/kesh-db/src/repositories/credit_notes.rs:139` |
| **Avoir : snapshot `credit_note_snapshot_json`** | `credit_notes.rs:36-56` |
| **Avoir : `fetch_credit_note_lines` (EN DUR)** | `credit_notes.rs:63-64` |
| **Avoir : 2e `SELECT` de `get()` (EN DUR)** | `credit_notes.rs:90-91` |
| **Avoir : `SELECT invoice_lines` du snapshot (EN DUR)** | `credit_notes.rs:266-268` |
| Avoir : lecture du défaut à remplacer | `credit_notes.rs:280-282` ; passage `:320-328` |
| Avoir : `INSERT credit_note_lines` | `credit_notes.rs:376-390` |
| Entité `CreditNoteLine` | `crates/kesh-db/src/entities/credit_note.rs:45` |
| `CreateInvoiceLineRequest` (4 champs, aucun `serde(default)`) | `crates/kesh-api/src/routes/invoices.rs:65-71` |
| `MAX_LINES = 200` | `routes/invoices.rs:58` |
| Style d'erreur `validate_line` | `routes/invoices.rs:370-410` |
| `accounts::update` (retypage possible) / `archive` | `crates/kesh-db/src/repositories/accounts.rs:344` / `:472` |
| `effective_postable` (postable dérivé des enfants) | `accounts.rs:126-131` |
| Résolution par rôle = prefill onboarding | `crates/kesh-db/src/repositories/company_invoice_settings.rs:236` |
| Export CSV des lignes (en-tête 9 colonnes) | `crates/kesh-api/src/exports/csv_tables.rs:459` |
| Compteurs export (**non affectés** par ADD COLUMN) | `crates/kesh-api/tests/exports_global_e2e.rs:621` (20 entrées), `:633` |
| Compteurs audit idempotence (**à incrémenter**) | `docs/migrations-idempotence-audit.md:68-71` |
| **`journal_entries` — AUCUNE colonne `source`/`origin`** (schéma complet) | `crates/kesh-db/migrations/20260412000001_journal_entries.sql:17-36` |
| **`line_order` réattribué à chaque `update`** (`(idx as i32) + 1`) | `crates/kesh-db/src/repositories/journal_entries.rs:1005` ; création `:272` |
| **`PUT /journal-entries/{id}` — aucune garde de provenance** | `crates/kesh-api/src/lib.rs:303` → `routes/journal_entries.rs:517` → `repositories/journal_entries.rs:805` |
| Avoir créé directement en `issued` (pas d'état `draft` intermédiaire) | `credit_notes.rs:357-362` ; CHECK `20260627000001_credit_notes.sql:50`, `:54` |

### Pièges, par ordre de coût

0. **Le backfill (D2-bis)** — deux pièges superposés, et ils tirent en sens **opposés**.
   - *Sans backfill*, tout le travail de D2 ne protège que les factures **futures** : le parc en production reste exposé au résidu. Invisible en test sur base vierge (`migrations_fresh_install` passe, il n'y a rien à backfiller) → exige un test sur base pré-remplie.
   - *Avec un backfill trop confiant*, on écrit un compte **faux** sur des factures validées réelles, et D5 le recopiera dans tout avoir futur. L'identification positionnelle (`line_order = 2`) semble solide parce que le **code de génération** est déterministe — mais l'écriture est **éditable après coup** (`PUT /journal-entries/{id}`, aucune garde de provenance) et `update` réattribue les `line_order`. Le discriminant obligatoire est le **montant égal à `invoices.total_amount`** (il n'existe pas de colonne `total_ht`) ; sans ambiguïté ou rien.
   - *Troisième face du même piège (passe 6)* : la condition d'exclusion des comptes de TVA doit être écrite en **`<=>` NULL-safe**. Les colonnes TVA de `company_invoice_settings` sont `NULL` sur **toute** installation ; un `<>` ordinaire propage le `NULL`, aucune ligne n'est candidate, et le backfill **no-ope sur 100 % du parc** — succès de migration, décompte élevé, c'est-à-dire indiscernable du comportement conservateur voulu. C'est le mode de défaillance le plus vicieux des trois, parce que la spec elle-même pré-autorise un décompte élevé.
   - La bonne posture est donc **conservatrice** : refuser d'écrire et dénombrer, plutôt qu'écrire faux. Un dev qui cherche à faire passer une post-condition « plus aucun `NULL` » ira mécaniquement dans le mauvais sens — d'où sa suppression explicite d'AC2-bis, et la borne « par ce chemin » posée sur D2 et AC9-bis.
1. **L'avoir (D5)** — le plus coûteux si oublié : corruption comptable silencieuse, équation du bilan toujours équilibrée, donc **aucun signal**. Le test AC17 « les deux écritures s'annulent compte par compte » est le garde-fou.
2. **Les 8 sites (AC5 / AC5-bis / AC5-ter)** — 6 listes de colonnes SQL + 2 snapshots d'audit, dont 4 listes **écrites en dur** hors `LINE_COLUMNS`. Un oubli ne casse pas la compilation : `sqlx::query_as` échoue au **runtime**, potentiellement seulement sur le chemin d'export ou d'avoir, donc pas forcément dans les tests rapides. (Le 9ᵉ site, l'appel de `generate_credit_note_journal_lines` en AC11, est lui attrapé par le compilateur.)
3. **`account_type` au posting (D3)** — `create_in_tx` ne le vérifie **jamais**. Si AC8 est implémenté en copiant seulement « archivé + non-postable », un compte retypé passe et le produit atterrit sur un compte de charge. Faux sans bruit.
4. **La non-régression mono-compte (AC12)** — les 8 tests existants doivent passer sans retoucher leurs assertions. C'est ce qui rend la migration sûre pour les bases existantes, dont l'instance de production de Guy.
5. **`#[serde(default)]` (AC6)** — son absence casse **toutes** les intégrations PAT existantes au déploiement, silencieusement à la revue puisque le code compile.

### Propagation post-patch (§ CLAUDE.md)

Après chaque patch de remédiation, **grep le symptôme sur tout le dépôt** avant la passe suivante — pas seulement le site corrigé. Symptômes à balayer sur cette story : `revenue_account_id`, `LINE_COLUMNS`, `line_total, created_at` (listes de colonnes en dur), `generate_.*journal_lines`, `default_revenue_account_id`.

### Pattern batch — non applicable

Le pattern `FailedProposal` / `{ accepted, failed }` de `CLAUDE.md` **ne s'applique pas** ici : la création/modification de facture rejette la requête entière en `400` en amont de tout traitement per-ligne, ce qui relève des exceptions explicites de la règle. Cf. D7.

### References

- Issue **#152** (cette story), **#144** (16-2), **#265** (CR d'origine).
- Stories antérieures : **14-3a/14-3b** (rôles, `postable`, `enforce_postable = false` pour les flux automatiques), **19-3/19-4** (re-validation au posting, accepted risk ABBA), **12-1** (avoirs et contre-passation), **18-1b** (helper d'écriture facture + TVA due).
- `CLAUDE.md` : politique de migration (P1-P5), pattern batch, Review Iteration Rule, propagation post-patch, règle de splitting préventif.

---

## Change Log

### Passe 1 de `validate` — 2026-07-26 (Sonnet, 3 lentilles : BlindHunter / EdgeCaseHunter / AcceptanceAuditor)

28 findings bruts, tous vérifiés en ground-truth par l'orchestrateur (`grep -nF` / `Read`) avant application. Aucun faux positif écarté.

**Décision structurante** — split de 16-1 en **16-1a** (backend, cette story) et **16-1b** (frontend), sur déclenchement de la § « Règle de splitting préventif » (8 modules touchés > seuil de 5). Arbitré par Guy.

**Questions ouvertes tranchées par Guy** : D5 (avoirs) **dans le périmètre** ; D7 initial (compte invalide au formulaire) = afficher + signaler + bloquer la ligne, porté en 16-1b. La question n°3 (prefill catalogue 16-2) est levée par hypothèse : le compte du produit ne fera que **pré-remplir** la ligne côté frontend, sans reprise rétroactive.

**Corrections factuelles appliquées** (la spec initiale affirmait des choses fausses) :

| Correction | Preuve |
|---|---|
| D3 affirmait « une facture validée pourrait créditer un compte archivé » — **faux**, `active = TRUE` est inconditionnel dans `validate_lines_accounts_in_tx` | `journal_entries.rs:77` |
| Le vrai trou est `postable` (désactivé par `enforce_postable = false`), `account_type` (**jamais** vérifié) et l'illisibilité du message | `journal_entries.rs:65-114`, `errors.rs:2161` |
| AC7 exigeait « un code d'erreur canonique, pas de `format!` » — mauvaise application du pattern batch ; la convention du fichier est `AppError::Validation(format!("Ligne {} …"))` | `routes/invoices.rs:370-410` |
| AC19 initial visait « l'export des lignes d'avoir » — **inexistant** | `grep "credit_note" crates/kesh-api/src/exports/` vide ; `exports_global_e2e.rs:621-634` |
| Le « piège des compteurs d'export » visait les mauvais compteurs (20 fichiers ZIP / 23 tables, non affectés par un ADD COLUMN) ; les vrais compteurs à incrémenter sont ceux de l'audit d'idempotence | `exports_global_e2e.rs:621` ; `migrations-idempotence-audit.md:68-71` |
| Ancre « lecture du défaut `~:1370` » erronée | réel `invoices.rs:1319-1320` |

**Sites de code manquants ajoutés** : `list_all_lines_by_company` (`invoices.rs:1937`), `credit_note_snapshot_json` (`credit_notes.rs:36`), `fetch_credit_note_lines` (`:63`), 2e `SELECT` de `get()` (`:90`), `SELECT invoice_lines` du snapshot (`:266`), `INSERT credit_note_lines` (`:376`), entité `CreditNoteLine` (`entities/credit_note.rs:45`) — 4 listes de colonnes **en dur** que la spec initiale ignorait, dont l'oubli casse au runtime et non à la compilation.

**Décisions nouvelles** : D3-bis (exemption `postable` pour le compte égal au défaut société — sinon `NULL` et choix explicite du même compte ont des verdicts opposés), D4-bis (facture à montant total nul → `500` SQL pré-existant sur `chk_jel_debit_credit_exclusive`, fermé ici), D5-bis (compte archivé entre validation et avoir → échec avec message précis, `active` seul re-vérifié), D6 (validation batchée `IN (…)`, message multi-lignes — 200 lignes max sinon N+1), D7 (style d'erreur).

**AC ajoutés** : AC5-bis, AC11-bis, AC13-bis, AC14-bis, AC19 (invariant D3-bis), AC20 (garde-fou D1 — sans deux comptes distincts, le test passerait aussi avec une résolution par rôle), AC21, AC22. **AC12 ancré** sur les 8 tests existants (`invoices.rs:1996-2165`). **AC13 reformulé** (était vacueux : le rapport TVA ne lit jamais de compte de produit).

### Passe 2 de `validate` — 2026-07-26 (Haiku 4.5, 2 lentilles : correctness + EdgeCaseHunter, contexte frais)

**4 findings** (contre 28 en passe 1). Tous vérifiés en ground-truth avant application. Aucune hallucination de type « patch non appliqué » — la discipline `grep -nF` imposée aux reviewers a tenu.

| Finding | Verdict | Traitement |
|---|---|---|
| AC8 ne dit pas si le **compte par défaut de la société** fait partie de l'ensemble re-validé au posting | **Réel, et c'est le meilleur finding de la passe** — une facture à lignes toutes `NULL` (le seul cas existant en production aujourd'hui) échappait entièrement à AC8 | **AC8-bis** ajouté : l'ensemble re-validé est `{comptes de ligne} ∪ {défaut, si ≥ 1 ligne NULL}`, `postable` exempté sur le défaut (cohérent D3-bis), message le nommant explicitement. **AC18-bis** ajouté (2 cas de test) |
| T2 annonce « 7 sites de colonnes » là où AC5 + AC5-bis en énumèrent 8 | Réel — double-comptage des snapshots entre T2 et AC5-bis | **AC5-ter** ajouté : décompte de référence **8 sites** (6 listes SQL + 2 snapshots) **+ 1 site d'appel**. T2 corrigé |
| AC5-bis nomme les sites mais pas la **nature** du changement (colonne ET `.bind()`) | Réel, clarté | Clause « nature exacte du changement, site par site » ajoutée à AC5-bis |
| AC11 change la signature du helper d'avoir sans nommer le **site d'appel** `credit_notes.rs:320-328` | Réel mais **sévérité surévaluée par le reviewer** (annoncée HIGH avec un scénario « compile mais échoue au runtime » — faux, un désaccord de types est une erreur de **compilation** en Rust) | Reclassé LOW, patché quand même : AC11 nomme le site d'appel et précise la transformation paires → triplets |

**Trend** : passe 1 = 28 findings (dont plusieurs CRITICAL/HIGH) → passe 2 = 4 findings (1 MEDIUM réel, 3 clarté). Convergence monotone conforme à la § « Règle de splitting préventif » amendée (la sévérité décroît, pas de stagnation) — pas de nouveau split requis.

### Passe 3 de `validate` — 2026-07-26 (Opus, contexte frais)

**6 findings : 1 CRITICAL, 1 HIGH, 4 MEDIUM.** Tous vérifiés en ground-truth avant application. **La sévérité remonte** par rapport à la passe 2 (1 MEDIUM) — cf. arbitrage de convergence en fin de section.

Les quatre findings sérieux ont **le même parent** : la spec traitait le compte de ligne comme une donnée locale, alors que `NULL` est une **référence tardive à un état de configuration mutable**. Le correctif d'un seul d'entre eux les ferme tous.

| Finding | Verdict | Traitement |
|---|---|---|
| **CRITICAL — l'invariant « inverse exact » de D5 est rompu pour les lignes `NULL`** dès que le défaut société change entre la facture et l'avoir. La chaîne prescrite était : ligne `NULL` → snapshot d'avoir `NULL` → repli résolu à T2. Facture créditée sur `défaut(T1)`, avoir débité sur `défaut(T2)` → résidu permanent, bilan équilibré, compte de résultat faux, aucun signal. **C'est exactement le mode de défaillance que D5 qualifie de « point le plus grave »** — fermé pour les comptes explicites, laissé ouvert pour `NULL`, c'est-à-dire le **seul cas existant en production**. Aggravant : AC17 donnait une fausse assurance (un test qui ne change pas le défaut passe systématiquement) | **Réel**. Bug **pré-existant** (aujourd'hui déjà facture = `défaut(T1)`, avoir = `défaut(T2)`), mais la story réécrit ces deux helpers et érige l'invariant en décision — même logique que D4-bis | **D2 amendée** : matérialisation du compte effectif dans `invoice_lines` à la validation (`UPDATE … WHERE revenue_account_id IS NULL`), dans la transaction, avant `create_in_tx`. **AC9-bis** ajouté. **AC17 doté d'un 2ᵉ cas** qui échoue si la matérialisation manque. Le système n'a plus qu'**un seul instant de résolution** |
| **HIGH — AC18-bis (passe 2) était inatteignable** : `get_or_create_default_in_tx` (`invoices.rs:1310-1312`) JOINe `av.active = TRUE` (`company_invoice_settings.rs:482`) et rejette un défaut archivé **avant** tout point d'accroche d'AC8-bis. Le dev aurait écrit un test qui échoue, puis retouché l'assertion pour accepter le `400` générique — **enterrant** AC8-bis | Réel, vérifié `sed -n '470,500p'`. Le seul critère qu'AC8-bis apporte réellement sur le défaut est `account_type` | **AC8-bis** : critère ramené à `account_type = Revenue` seul, avec la preuve que `active` est garanti en amont. **AC18-bis** : 1ᵉʳ cas passe de « archivé » à « **retypé** » |
| MEDIUM — l'exemption D3-bis exige le défaut à la saisie, or `create`/`update` ne lisent **jamais** `company_invoice_settings`. Le dev appliquant la convention locale utiliserait `get_or_create_default_in_tx` → **création de brouillon régressée en `400`** pour une société au défaut archivé, plus un verrou dans une transaction à l'ordre contraint | Réel, vérifié (aucune occurrence dans `invoices.rs:438-500`) | D3-bis : clause « comment obtenir le défaut à la saisie » — `SELECT` nu, interdiction explicite de `get_or_create_default_in_tx`. **AC12-bis** (non-régression) ajouté |
| MEDIUM — l'exemption D3-bis est indexée sur une valeur mutable évaluée à deux instants : une ligne acceptée à la saisie devient non-validable sans que rien n'ait changé d'elle | Réel | D3-bis : « fenêtre assumée », dette LOW documentée + message d'erreur qui la rend lisible |
| MEDIUM — « 16-1a livrable seule, sans effet observable » est incomplet : `initLines()` reconstruit les lignes à partir de 4 champs et `#[serde(default)]` rend l'omission silencieuse → une intégration PAT pose un compte, l'utilisateur enregistre depuis l'UI, le compte est effacé sans message | Réel | Paragraphe « Provenance du split » nuancé + dépendance dure tracée en « Ce qui n'est PAS dans 16-1a » (2 sites de mapping, dont `reloadFromServer`) |
| MEDIUM — sur-spécification : la section F dupliquait un par un les AC comportementaux ; AC22 et AC6 portaient **la même phrase** | Réel | **AC22 supprimé** (sa clause est déjà dans AC6). La restructuration complète proposée (fusionner toute la section F dans les AC comportementaux) est **déclinée** : churn important sur 30 AC juste avant une passe de revue, risque d'introduire des incohérences supérieur au bénéfice de lisibilité. À reconsidérer si une passe ultérieure le re-signale |

**Vérifié négatif (utile à conserver)** : aucun consommateur des lignes d'écriture ne suppose « exactement une ligne de crédit produit ». `income_statement.rs:75`, `balance_sheet.rs:292/:335`, `trial_balance.rs:79`, `project_report.rs:266/:294` agrègent tous par `account_id` (`INNER JOIN … ON jel.account_id = a.id`) ; `vat_report.rs:220-226` filtre sur le seul compte de TVA due ; `journal_report.rs:113/:137` et `csv.rs:302` trient sans attendre de cardinalité. Les seuls `lines[1]` du dépôt sont des assertions de tests sur des écritures manuelles à 2 lignes. La clôture d'exercice (Epic 14) calcule par compte. **La ventilation traverse tous ces chemins sans effet — c'est précisément l'objectif de la story.**

**Trend et arbitrage de convergence** : passe 1 = 28 → passe 2 = 4 → passe 3 = 6 dont 1 CRITICAL + 1 HIGH. La sévérité **remonte**, ce qui coche formellement le second critère de la § « Règle de splitting préventif » (amendement 2026-07-26). **Re-split décliné pour 16-1a**, motivé : la remontée ne traduit pas une story trop large — les 32 findings des passes 1 et 2 tiennent tous, aucun n'est remis en cause — mais **un angle que deux passes orientées détail ne pouvaient pas atteindre** (l'invariant temporel de la référence `NULL`). Le correctif est **une décision amendée et trois AC**, pas une redécoupe ; et le finding de sur-spécification pousse dans le sens inverse (le document est trop long, pas le scope trop large). Une passe 4, modèle différent, doit confirmer la convergence.

### Passe 4 de `validate` — 2026-07-26 (Sonnet, contexte frais)

**3 findings : 1 CRITICAL, 1 HIGH, 1 MEDIUM.** Les deux premiers sont des **conséquences directes du patch de la passe 3** — illustration du mode d'échec que `feedback_review_patch_needs_test` documente : la remédiation devient la source des findings suivants.

| Finding | Verdict | Traitement |
|---|---|---|
| **CRITICAL — la matérialisation de D2 ne couvre PAS les factures déjà validées.** Elle ne se déclenche qu'à la transition `draft → validated` ; une facture déjà `validated` n'y repasse jamais (`update` rejette tout statut ≠ `draft`, `invoices.rs:841`/`:1271`) et aucune autre écriture ne touche `invoice_lines` après validation. Après l'`ADD COLUMN`, ces lignes portent `NULL` définitivement. Or la spec écrit elle-même que `NULL` est « le seul cas qui existe en production aujourd'hui » : le patch de la passe 3 protégeait un ensemble **vide** à l'instant du déploiement, et **100 % du parc validé** — dont l'instance de production en fonction — restait exposé au résidu | **Réel et sérieux** | **D2-bis** ajoutée : backfill dans la migration, **source = l'écriture comptable générée**, pas le défaut courant (backfiller depuis le défaut d'aujourd'hui fabriquerait la corruption si le défaut a changé par le passé). **AC2-bis** ajouté, **T1** étendu, nouveau piège n°0. Alternative « documenter en dette cat. B » explicitement écartée : corruption sur données réelles, pas limitation fonctionnelle |
| **HIGH — l'`UPDATE` SQL de D2 ne met pas à jour la copie en mémoire.** `lines_before` est chargé à l'étape (1), avant la matérialisation, et réutilisé après elle pour le snapshot d'audit `"after"` (`invoices.rs:1448`) et pour `ValidatedInvoice.lines` (`:1478`), **délibérément non re-fetché** post-commit (`:1114`, garantie anti-race). L'audit et la réponse HTTP auraient affiché `null` pour des lignes que la base venait de matérialiser | Réel — AC9-bis exigeait « post-matérialisation » sans dire comment l'obtenir | **AC9-bis** complété : muter `lines_before` en mémoire juste après l'`UPDATE`, **sans** re-fetch DB |
| MEDIUM — « AC25 » cité dans la prose de D1 n'existait dans aucune section formelle ni dans aucune tâche ; son contenu est déjà entièrement porté par AC20 | Réel — même classe que l'AC22 nettoyé en passe 3 | Renvoi corrigé vers AC20 |

**Vérifié négatif** : toutes les ancres ajoutées ou modifiées en passe 3 (`company_invoice_settings.rs:173`/`:480-494`/`:482`, `routes/company_invoice_settings.rs:117-120`, `invoices.rs:1310-1312`/`:806-812`, `InvoiceForm.svelte:107-113`/`:365-368`/`:418-424`) pointent exactement sur le code décrit. La cohérence D1 → D7 tient et le décompte d'AC5-ter (8 sites + 1 site d'appel) est exact.

**Trend** : 28 → 4 → 6 → 3. Volume décroissant, mais la sévérité reste au-dessus de LOW → **passe 5 requise** (contexte frais, modèle différent). Cible prioritaire de la passe 5 : le backfill de D2-bis, patch tout neuf et non revu — en particulier l'identification de la ligne de crédit produit dans les écritures existantes.

### Passe 5 de `validate` — 2026-07-26 (Haiku 4.5 ×2 lentilles, contexte frais + vérification orchestrateur)

**2 findings : 1 HIGH, 1 MEDIUM.** Les deux ont **le même parent** et portent exactement sur la cible prioritaire annoncée en fin de passe 4 : le backfill de D2-bis.

**Les deux reviewers Haiku ont rendu « 0 finding »** — et l'un d'eux a explicitement affirmé l'inverse du finding ci-dessous (« ordre **immuable** », « `line_order = 2` fiable car l'ordre est imposé et n'a jamais changé »). L'erreur est instructive : il a vérifié le **code de génération** (`journal_entries.rs:272`, déterministe — exact) et en a conclu à une propriété des **données** (jamais modifiées — faux). Le finding vient de la vérification d'orchestrateur, pas des reviewers. C'est le second précédent du dépôt où Haiku produit une assurance positive erronée là où la § « Haiku-specific guardrails » n'attendait que des faux positifs `CRITICAL` : **un « rien trouvé » de Haiku n'est pas une preuve de convergence** et doit être recoupé.

| Finding | Verdict | Traitement |
|---|---|---|
| **HIGH — la prémisse du backfill de D2-bis est fausse sur les données réelles.** D2-bis identifie le compte à backfiller dans « l'écriture comptable réellement générée », en s'appuyant sur la structure canonique `[0]` créance / `[1]` produit / `[2..]` TVA, par `line_order` positionnel ou par élimination. Or cette structure décrit ce que le helper **produit**, pas ce que la table **contient** : `journal_entries` ne porte **aucune** colonne `source`/`origin`/`is_auto` (schéma complet lu, `20260412000001_journal_entries.sql:17-36`), et `PUT /api/v1/journal-entries/{id}` (`lib.rs:303` → `routes/journal_entries.rs:517` → `repositories/journal_entries.rs:805`) n'a **aucune garde de provenance** — ses seules gardes sont exercice clos, verrou optimiste, équilibre partie double, validité des comptes. `update` fait `DELETE FROM journal_entry_lines WHERE entry_id = ?` puis ré-INSERT avec `line_order = (idx as i32) + 1` (`:1005`). L'écriture d'une facture validée est donc **éditable**, et les deux méthodes d'identification échouent en silence sur les écritures retouchées (positionnelle : la 2ᵉ ligne peut être une TVA ; élimination : zéro ou plusieurs candidats) | **Réel.** Aggravant : l'erreur n'est pas neutre. Un mauvais compte écrit dans `invoice_lines` devient **la** vérité de la facture, que D5 recopiera dans tout avoir futur — **le backfill fabriquerait la corruption qu'il existe pour fermer**, en lui donnant l'apparence d'une donnée établie. Sur les écritures non éditées (quasi-tout le parc) l'identification reste fiable : le défaut est le manque de discrimination, pas la méthode | **D2-bis réécrite** avec un **critère d'unicité en trois conditions**, dont le discriminant que ni la position ni l'élimination ne donnent : **`credit` égal au `total_ht` de la facture**. Zéro ou plusieurs candidats → la ligne reste `NULL`, **comptée**, migration réussie. **AC2-bis réécrit** : sa post-condition « aucune ligne validée ne reste `NULL` » était **fausse** et poussait activement au mauvais comportement (relâcher le critère jusqu'à ce qu'elle passe = écrire un compte arbitraire sur des données comptables réelles) — remplacée par 3 post-conditions dont **une écriture éditée qui doit rester `NULL`**. **T1** et le **piège n°0** réécrits (le piège tire désormais dans les deux sens) |
| MEDIUM — D2 affirme que la matérialisation rend la facture validée « **auto-descriptive** » et réduit le système à « **un seul instant de résolution** ». Même cause : l'écriture restant éditable après la validation, `invoice_lines.revenue_account_id` peut cesser de décrire l'écriture réelle. L'invariant est vrai *à la validation*, pas indéfiniment | Réel, mais **sans régression** : aujourd'hui l'avoir se replie sur le défaut *courant*, encore plus éloigné de l'écriture éditée. La story améliore strictement | D2 dotée d'une clause « **borne de cet invariant** » avec les ancres, et mention explicite que le découplage facture ↔ écriture est **hors périmètre** de 16-1a. Sa seule conséquence opératoire est celle du finding HIGH |

**Vérifié négatif** : (1) **pas de fenêtre d'avoir `draft`** — le CHECK `chk_credit_notes_status` autorise `'draft'` (`20260627000001_credit_notes.sql:50`) mais `create_credit_note` insère directement `'issued'` (`credit_notes.rs:357-362`, « single-step DC5 ») et c'est la **seule** fn publique de création ; aucun avoir ne peut donc être créé avant la migration puis émis après elle avec des lignes non backfillées. (2) Le lien facture → écriture (`invoices.journal_entry_id`, FK `ON DELETE RESTRICT`) empêche la **suppression** de l'écriture d'une facture validée — seule l'édition est possible. (3) Les 5 ancres d'AC9-bis (`invoices.rs:1448`, `:1478`, `:1114`, `:841`, `:1271`) et `:1310-1312` pointent exactement sur le code décrit. (4) Traçabilité D→AC→T complète, aucun AC orphelin, aucun renvoi fantôme résiduel (les « AC22 » / « AC25 » nettoyés en passes 3-4 ne subsistent que dans le Change Log historique). (5) AC12 satisfiable : les 8 tests existants ne changent que de **fixture** (`revenue_account_id: None`), pas d'assertion — les lignes `NULL` fusionnant en un compte unique, la cardinalité des écritures est inchangée.

**Trend** : 28 → 4 → 6 → 3 → 2. Volume décroissant, sévérité maximale décroissante (`CRITICAL` en P3 et P4 → `HIGH` en P5), et les deux findings sont **circonscrits au seul patch neuf de la passe 4**, sans rien remettre en cause du reste. Le second critère de la § « Règle de splitting préventif » (sévérité égale ou supérieure) n'est **pas** coché. Reste au-dessus de LOW → **passe 6 requise**. Cible prioritaire : le critère d'unicité en trois conditions de D2-bis, lui-même patch neuf et non revu — en particulier sa formulation SQL en MariaDB et le cas de la facture dont plusieurs lignes de crédit légitimes auraient le même montant que `total_ht`.

### Passe 6 de `validate` — 2026-07-26 (Opus, contexte frais)

**8 findings : 2 HIGH, 3 MEDIUM, 3 LOW.** Tous vérifiés en ground-truth par l'orchestrateur avant application, aucun faux positif écarté. Cinq des huit sont circonscrits au backfill D2-bis ; le second HIGH est une **régression de la passe 3** restée invisible deux passes durant.

| Finding | Verdict | Traitement |
|---|---|---|
| **HIGH — la condition (2) du critère de backfill, écrite naïvement, fait no-oper le backfill sur 100 % du parc.** « Ni la créance ni un compte de TVA » traduit en `jel.account_id <> cis.default_vat_payable_account_id` : les 3 colonnes TVA de `company_invoice_settings` sont **`NULL` sur toute installation** — l'`INSERT` d'onboarding ne les énumère pas (`company_invoice_settings.rs:452-458`), le lazy-create insère `(company_id)` seul (`:92`), aucune migration ne les renseigne (`grep -rn "SET default_vat" crates/kesh-db/migrations/` → vide). Comparaison à `NULL` ⇒ prédicat `NULL` ⇒ zéro candidat partout | **Réel, et c'est le pire mode de défaillance du document** : migration en succès, décompte « très élevé » — donc **indiscernable du comportement conservateur que la spec vient de pré-autoriser** en toutes lettres. Tout le bénéfice de D2-bis (le CRITICAL de la passe 4) perdu sans aucun signal | D2-bis : condition (2) réécrite en ensemble d'exclusion `E = { receivable, vat_payable }` avec **`NOT (a <=> b)` NULL-safe obligatoire**, interdiction explicite de `<>` / `!=` / `NOT IN`, et réduction de `E` (les comptes TVA récupérable / décompte n'apparaissent jamais dans une écriture de vente, `invoices.rs:1387`). **AC2-bis doté d'une 4ᵉ post-condition** : société à `default_vat_payable_account_id` `NULL` → la ligne **est** backfillée. C'est le seul test qui distingue les deux comportements |
| **HIGH — AC8-bis reposait sur une ancre attribuée à la mauvaise fonction ; la remédiation de la passe 3 a ouvert un trou.** La passe 3 avait retiré `active` des critères re-validés sur le compte par défaut, au motif que `get_or_create_default_in_tx` JOINe `av.active = TRUE` (`company_invoice_settings.rs:482`) et rejetterait un défaut archivé en amont. **Faux** : `get_or_create_default_in_tx` occupe `:87-107` et se réduit à `INSERT IGNORE` + `SELECT … FOR UPDATE`, **sans JOIN ni contrôle d'`active`** ; le JOIN de `:482` est dans `insert_with_defaults_in_tx` (`:403`), appelée seulement depuis `routes/onboarding.rs:720` | **Réel.** Les passes 4 et 5 ont toutes deux « vérifié négativement » cette ancre — en lisant la **ligne** sans borner la **fonction**. Leçon de méthode : `grep -n` sur une ancre ne suffit pas, il faut délimiter la fonction englobante. Conséquence : le défaut archivé retombe sur le `400 INACTIVE_OR_INVALID_ACCOUNTS` générique — le message même qu'AC8-bis existe pour éliminer, sur le cas même qu'il dit couvrir | **`active` rétabli** dans les critères d'AC8-bis (aux côtés d'`account_type`, `postable` restant exempté). **AC18-bis** : 3ᵉ cas « défaut archivé » rétabli, avec assertion sur le message. **D3-bis** : l'interdiction de `get_or_create_default_in_tx` à la saisie est conservée mais son motif corrigé (écriture + verrou, pas l'échec sur archivé). **AC12-bis** : sa prétention à « attraper l'usage accidentel » retirée — elle passe dans les deux cas ; la garantie est déplacée en revue de code sur T5 |
| MEDIUM — la post-condition « aucune ligne de facture validée n'a `revenue_account_id IS NULL` », déclarée **fausse** et supprimée d'AC2-bis en passe 5, subsistait **mot pour mot** en D2 et en **AC9-bis** — ce dernier étant un AC formel, donc celui que le dev implémente | Réel. Symptôme non grepé après le patch de la passe 5 (§ « Propagation post-patch » de `CLAUDE.md`, exactement le mode d'échec qu'elle décrit) | D2 et AC9-bis bornés « facture validée **par ce chemin** » / « la facture que le test vient de valider », avec interdiction explicite du `COUNT(*) = 0` global |
| MEDIUM — la 3ᵉ post-condition d'AC2-bis (« facture validée **sans écriture** → reste `NULL` ») décrit un état **interdit en base** : `chk_invoices_validated_has_je` (`migrations/20260417000002_…sql`) et `fk_invoices_journal_entry … ON DELETE RESTRICT` | Réel — même classe que le HIGH de la passe 3 (AC inatteignable). Le dev butera sur une violation de CHECK sans recours | Cas remplacé par un chemin réellement atteignable (« écriture éditée, aucune ligne ne crédite `total_amount` »), avec la preuve d'inconstructibilité conservée en note. Le prédicat `journal_entry_id IS NOT NULL` de la portée est requalifié garde défensive redondante |
| MEDIUM — le décompte des lignes non backfillées était à la fois **post-condition testée** (AC2-bis) et **mécanisme à trancher en `dev-story`** (D2-bis) — donc non vérifiable. Et l'une des deux options renvoyées, la table de rapport, fait tomber `backup_inventory_matches_schema` (`backup.rs:577-606`), impose de toucher `TABLES_TO_TRUNCATE`, et ferait entrer la table dans le périmètre de l'export d'installation — décision d'architecture prise par accident pour faire passer un test, contredisant AC14-bis | Réel | **Tranché dans la spec** : aucun artefact nouveau. Les deux options sont explicitement écartées avec leur motif ; le décompte passe par une **requête de diagnostic documentée** (consignée au CHANGELOG, AC24), et c'est elle que le test assert. **AC14-bis** doté de sa condition de validité (« cette story ne crée aucune table ») |
| LOW — `total_ht`, discriminant du critère, **n'est pas une colonne** : c'est `invoices.total_amount` (`20260416000001_invoices.sql:24` ; `grep -rn "total_ht" crates/kesh-db/migrations/` → vide). L'asymétrie avec le miroir avoir, qui nommait déjà `total_amount`, suggérait activement deux grandeurs | Réel. Erreur rattrapée au premier `Unknown column`, sans risque silencieux | Corrigé aux 4 sites (D2-bis, AC2-bis, T1, piège n°0), avec le renvoi à la docstring `invoices.rs:1128` qui écrit littéralement l'équivalence |
| LOW — le motif d'exclusion des factures `cancelled` (« sans écriture ») est faux : le seul chemin vers ce statut est l'émission d'un avoir (`credit_notes.rs:398`), elles ont donc **toujours** une écriture | Réel. La **décision** reste bonne — `uq_credit_notes_invoice` (`20260627000001:58`) interdit un second avoir, aucun résidu futur possible — mais un dev re-dérivant la portée depuis un motif faux peut élargir le `WHERE` | Motif remplacé par le vrai, et l'asymétrie qui en découle (avoir backfillé / facture `NULL`, visible dans l'export CSV d'AC14) explicitement assumée |
| LOW — T1 demandait de justifier la migration comme « **non idempotente** au sens strict », en contradiction avec AC4 (`tracked-by-sqlx`) et avec l'invariant « Idempotence `no` : 0 » du fichier d'audit | Réel. Le backfill est **intrinsèquement idempotent** (garde `IS NULL` + critère déterministe), comme ceux de `20260628000001` et `20260722000001`, tous deux `tracked-by-sqlx`. Un verdict `no` ferait diverger les compteurs prescrits par AC4 → finding MEDIUM garanti en `bmad-code-review` (P5) | T1 réécrit : verdict `tracked-by-sqlx` justifié par les `ADD COLUMN` sans `IF NOT EXISTS`, backfill qualifié idempotent avec ses deux précédents |

**Vérifié négatif (substantiel — à ne pas ré-instruire)** : (1) **la condition (3) est numériquement sûre** — `journal_entry_lines.debit`/`credit` et `invoices`/`credit_notes.total_amount` sont tous `DECIMAL(19,4)`, exact en MariaDB, même échelle ; et les deux valeurs sont identiques **par construction**, `generate_invoice_journal_lines` poussant `credit: total_ht = Σ line_total` (`invoices.rs:1178`, `:1198`) tandis que `invoices.total_amount` vient de `compute_total` = `Σ compute_line_total` (`:379-383`), la même fonction qui écrit `line_total` (`:393`). Facture validée immuable ⇒ pas de divergence ultérieure. (2) **Aucune facture ancienne n'a de structure incompatible** : la seule version antérieure du helper (remplacée par Epic 18, commit `654dba7d`) produisait 2 lignes avec `credit = invoice_before.total_amount` — la condition (3) y est vraie *a fortiori*. (3) **Aucun faux positif constructible** sur (1)+(2)+(3) réunies : les scénarios d'écriture éditée testés donnent 0 ou ≥ 2 candidats → `NULL`, comportement voulu ; le seul cas à 1 candidat « faux » suppose que l'écriture crédite réellement ce compte, ce qui est cohérent avec la définition même de D2-bis. (4) **Pas de blocage MariaDB** : ER 1093 inapplicable (la sous-requête ne lit jamais la table cible), précédent de multi-table UPDATE en migration (`20260628000001:115`), aucune CTE dans le dépôt. (5) **Traçabilité D→AC→T complète**, aucun renvoi fantôme hors Change Log historique, saut AC22 documenté. (6) **AC5-ter exact** : 8 sites + 1 site d'appel, re-vérifiés un par un.

**Trend** : 28 → 4 → 6 → 3 → 2 → **8** (2 HIGH). Volume **et** sévérité remontent → le second critère de la § « Règle de splitting préventif » est **coché**, arbitrage Project Lead requis (cf. ci-dessous). Reste largement au-dessus de LOW → **passe 7 requise**.

---

## Dev Agent Record

### Agent Model Used

_(à compléter par `dev-story`)_

### Debug Log References

### Completion Notes List

### File List
