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

**Le split n'ouvre aucune fenêtre de corruption comptable** : toute la correction du moteur (facture **et** avoir, décision D5) est dans 16-1a. 16-1a est livrable seule — colonne nullable + champ API rétro-compatible, sans UI, donc sans effet observable tant que 16-1b n'est pas là.

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

---

## Décisions de conception

### D1 — Le repli est `company_invoice_settings.default_revenue_account_id`, PAS le rôle `DefaultRevenue`

Une ligne sans compte se poste sur `settings.default_revenue_account_id`, exactement comme aujourd'hui.

**Motif** : la colonne est la source de vérité au runtime et reste **configurable par l'utilisateur** dans les Réglages ; le rôle `DefaultRevenue` ne sert qu'à **pré-remplir cette colonne à l'onboarding** (14-3b, cf. docstring `company_invoice_settings.rs:236`). Résoudre par rôle au posting serait un **changement de comportement non demandé** : il écraserait silencieusement le choix d'un utilisateur qui a délibérément pointé un autre compte dans les Réglages. Le message d'erreur `ConfigurationRequired("default_revenue_account_id")` reste inchangé.

**Garde-fou de test obligatoire (AC25)** : par défaut post-onboarding, `settings.default_revenue_account_id` **et** le compte portant le rôle `DefaultRevenue` sont le **même** compte — un test qui ne les dissocie pas passerait aussi bien avec une résolution par rôle. Le test doit donc les configurer sur **deux comptes différents**.

### D2 — `revenue_account_id` NULLABLE, liaison tardive (NULL n'est jamais matérialisé à la création)

La colonne est `BIGINT NULL`. `NULL` signifie « utiliser le défaut société **au moment de la validation** ». On ne copie pas le défaut dans la ligne à la création du brouillon.

**Motif** : conserve le comportement actuel à l'identique pour toute facture existante ou créée sans préciser de compte (un brouillon suit le défaut en vigueur au posting). Matérialiser le défaut à la création introduirait une divergence de comportement pour les brouillons de longue durée, non demandée par l'issue. Le compte effectivement utilisé est de toute façon tracé dans l'**écriture comptable** générée, qui est la pièce probante.

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
- **AC3** — Migration **non-breaking** (`ADD COLUMN` nullable + index + FK) → **pas** de bump `kesh_version_min_required` (politique P1/P2), donc **pas** de bump de version Cargo (P2-bis). Le vérifier explicitement.
- **AC4** — `docs/migrations-idempotence-audit.md` : ligne ajoutée au tableau détaillé avec verdict et justification, **ET** récapitulatif agrégé de bas de fichier mis à jour en cohérence — `Total` (`:68`, actuellement 55) et `Idempotence tracked-by-sqlx` (`:70`, actuellement 44) passent chacun à +1 (verdict attendu `tracked-by-sqlx`, la migration n'utilisant pas `IF NOT EXISTS`). Garde-fou **P5**.

### B. Backend — entité et tous les sites de colonnes

- **AC5** — L'entité `InvoiceLine` porte `revenue_account_id: Option<i64>`, et **les 3 sites qui listent les colonnes `invoice_lines`** sont mis à jour — sans quoi `sqlx::query_as` échoue au runtime :
  1. `LINE_COLUMNS` (`invoices.rs:39`) — utilisé par `insert_lines` (`:386`) et `fetch_lines` (`:425`) ;
  2. `list_all_lines_by_company` (`invoices.rs:1937-1944`) — **liste en dur avec préfixes `il.`**, alimente l'export ZIP global ;
  3. `create_credit_note` (`credit_notes.rs:266-268`) — **liste en dur**, lit `invoice_lines` pour le snapshot d'avoir.
  `invoice_snapshot_json` (`invoices.rs:51-60`) inclut le compte dans le snapshot d'audit.
- **AC5-bis** — Côté avoir, l'entité `CreditNoteLine` (`crates/kesh-db/src/entities/credit_note.rs:45`) porte le champ, et **les 4 sites** suivants sont mis à jour : `credit_note_snapshot_json` (`credit_notes.rs:36-56`), `fetch_credit_note_lines` (`:63-64`), le second `SELECT` de `get()` (`:90-91`), et l'`INSERT credit_note_lines` (`:376-390`). Il n'existe **pas** de constante `LINE_COLUMNS` côté avoir — soit en introduire une, soit traiter les 3 `SELECT` un par un ; ne pas en oublier.

### C. Backend — API

- **AC6** — `CreateInvoiceLineRequest` (`routes/invoices.rs:65-71`) accepte `revenueAccountId: Option<i64>` **portant `#[serde(default)]`**. Sans cet attribut, un `Option<T>` reste **obligatoire dans le JSON** en serde : l'omission totale de la clé ferait échouer la désérialisation, cassant toute intégration PAT existante à chaque création de facture. Suivre le style de `CreateInvoiceRequest` (`:78`, `:80`, `:85`) et **pas** celui des 4 champs voisins de `CreateInvoiceLineRequest`, tous obligatoires. Idem pour le DTO de modification. La réponse de lecture restitue le champ. Un test couvre le payload **sans la clé** et le payload avec `"revenueAccountId": null` — les deux valent `NULL`.
- **AC7** — Validation à la saisie (création `invoices.rs:459` **et** modification `:816`) : société, `active`, `account_type = Revenue`, `postable` (avec l'exemption D3-bis). Batchée en une requête (D6), message nommant toutes les lignes en défaut, style `AppError::Validation` (D7).
- **AC8** — Re-validation au posting dans `validate_invoice`, `SELECT` sans verrou sur le modèle 19-4 (`invoices.rs:1290-1310`), couvrant les **quatre** critères — dont `account_type`, que `create_in_tx` ne vérifie **jamais** (D3). L'échec nomme la ou les lignes concernées. Le commentaire reprend l'accepted risk ABBA / race d'archivage de 19-3/19-4.

### D. Backend — moteur comptable

- **AC9** — `generate_invoice_journal_lines` ventile le crédit produit : une ligne de crédit **par compte effectif**, montants `> 0`, tri `account_id` ASC (D4). La ligne `[0]` débit créance et les lignes TVA par taux sont **inchangées**. Lignes `NULL` et lignes pointant explicitement le défaut société fusionnent en une seule ligne.
- **AC10** — La section `# Équilibre par construction` de la docstring (`invoices.rs:1137-1142`) est **réécrite** — pas seulement complétée — pour couvrir la ventilation par compte en plus du filtre par taux, en reprenant l'argument de D4. L'hypothèse `F-OPUS-2` et la section `# Erreurs` restent à jour.
- **AC11** — `generate_credit_note_journal_lines` (`credit_notes.rs:139`) débite par compte, en miroir exact (D5) ; sa signature passe à `lines: &[(Decimal, Decimal, Option<i64>)]`. Sa docstring « inverse exact » reste vraie et est mise à jour.
- **AC11-bis** — Comportement D5-bis implémenté : compte du snapshot devenu `active = FALSE` → échec de l'émission de l'avoir avec message nommant ligne et compte. `postable` et `account_type` **ne sont pas** re-vérifiés côté avoir.
- **AC12** — **Non-régression, ancrée sur l'existant** : les tests unitaires actuels de `generate_invoice_journal_lines` (`invoices.rs:1996-2165`, 8 sites d'appel) passent **sans modification de leurs assertions** après la ventilation (leurs fixtures ont toutes `revenue_account_id = None`). Seule l'adaptation de signature est tolérée.
- **AC13** — Le rapport TVA n'est **pas** affecté : `kesh-report/src/vat_report.rs` ne lit que `default_vat_payable_account_id` / `default_vat_recoverable_account_id`, jamais un compte de produit. Un test d'intégration sur une facture **multi-comptes × multi-taux** vérifie que `reconciliation_status` reste `ok` (fichier `crates/kesh-report/tests/vat_report_reconciliation.rs`, nouveau cas).
- **AC13-bis** — D4-bis : `validate_invoice` (et l'émission d'avoir) rejettent une pièce dont `total_ht + total_vat == 0` avec une erreur métier `400` actionnable, au lieu du `500` SQL actuel sur `chk_jel_debit_credit_exclusive`.

### E. Exports

- **AC14** — `serialize_invoice_lines_csv` (`csv_tables.rs:459`) expose `revenue_account_id` dans l'en-tête et les enregistrements. Le test `crates/kesh-api/tests/exports_global_e2e.rs` est étendu pour vérifier la nouvelle colonne d'`invoice_lines.csv`.
- **AC14-bis** — **Aucun export CSV des lignes d'avoir n'existe** : `grep -n "credit_note" crates/kesh-api/src/exports/` revient vide, et les 20 entrées du ZIP (`exports_global_e2e.rs:621-634`) n'en contiennent pas. Aucune action requise de ce côté. Les compteurs existants ne sont **pas** affectés par un `ADD COLUMN` : ni `assert_eq!(entries.len(), 20)` (nombre de **fichiers**), ni `TABLES_TO_TRUNCATE` (23 **tables**) dans `admin_backup_e2e.rs`. La sauvegarde générique (`crates/kesh-db/src/backup.rs`) lit les colonnes dynamiquement via `non_generated_columns` — **auto-adaptée**, aucune modification.

### F. Tests & gate

- **AC15** — Tests unitaires du helper facture : mono-compte (non-régression AC12), multi-comptes, multi-comptes × multi-taux, ligne à montant nul filtrée, lignes `NULL` + explicite-même-compte fusionnées, ordre déterministe par `account_id`.
- **AC16** — Tests unitaires du helper avoir : miroir strict de AC15.
- **AC17** — Test d'intégration **pivot de D5** : facture ventilée sur ≥ 2 comptes puis avoir total → **les deux écritures s'annulent compte par compte** (agrégat par `account_id` de l'écriture facture + celle de l'avoir = 0 sur chaque compte).
- **AC18** — Tests d'intégration : compte invalide à la saisie (création **et** modification) ; compte devenu non-`postable` au posting ; compte **retypé** au posting (le trou que `create_in_tx` ne couvre pas) ; compte archivé au posting ; compte archivé entre validation et avoir (AC11-bis) ; plusieurs lignes invalides simultanément (le message les nomme toutes).
- **AC19** — Test D3-bis : `default_revenue_account_id` pointant sur un compte **non-postable** ; une ligne `NULL` et une ligne le désignant explicitement produisent le même verdict et la même écriture.
- **AC20** — Test D1 : `settings.default_revenue_account_id` **≠** compte portant le rôle `DefaultRevenue` (deux comptes distincts) ; une ligne sans compte se poste sur `settings.default_revenue_account_id`.
- **AC21** — Test D4-bis : facture entièrement à zéro → `400` métier, pas `500`.
- **AC22** — Test AC6 : payload sans la clé `revenueAccountId`, et payload avec `null` explicite.
- **AC23** — Gate « Test Locally First » backend complet vert (`cargo fmt --all -- --check`, `cargo build --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`). Le gate **runtime complet** est requis si le doute subsiste sur `min_required` (P2-bis) — ici la migration est non-breaking, mais les suites `migrations_fresh_install` et `admin_backup_e2e` doivent passer.
- **AC24** — CHANGELOG `[Non publié]` : entrée orientée utilisateur. Le README et les manuels LaTeX sont traités en **16-1b** (le comportement n'est pas visible utilisateur tant que l'UI n'est pas livrée).

---

## Tasks / Subtasks

- [ ] **T1** — Migration `invoice_lines.revenue_account_id` + `credit_note_lines.revenue_account_id` + index + FK `ON DELETE RESTRICT` ; ligne du tableau **et** compteurs agrégés de `docs/migrations-idempotence-audit.md` (AC1-AC4).
- [ ] **T2** — Entités `InvoiceLine` / `CreditNoteLine` + **les 7 sites de colonnes** listés en AC5 / AC5-bis + les 2 snapshots d'audit (AC5, AC5-bis).
- [ ] **T3** — API : DTOs création/modification avec `#[serde(default)]` + réponse de lecture (AC6, AC22).
- [ ] **T4** — Helper de validation batchée des comptes de ligne, réutilisable saisie + posting, avec exemption D3-bis et message multi-lignes (D6, D3-bis).
- [ ] **T5** — Branchement de T4 à la saisie (`invoices.rs:459` création, `:816` modification) (AC7).
- [ ] **T6** — `generate_invoice_journal_lines` : ventilation `BTreeMap` + docstring réécrite (AC9, AC10).
- [ ] **T7** — Branchement de T4 au posting dans `validate_invoice`, `SELECT` sans verrou, commentaire ABBA (AC8).
- [ ] **T8** — `generate_credit_note_journal_lines` en miroir + copie du compte à la création de l'avoir + garde D5-bis (AC11, AC11-bis).
- [ ] **T9** — Rejet des pièces à montant total nul (AC13-bis, AC21).
- [ ] **T10** — Export CSV `invoice_lines` + extension du test `exports_global_e2e` (AC14).
- [ ] **T11** — Tests unitaires (AC15, AC16) et d'intégration (AC17-AC22), dont le cas TVA multi-comptes × multi-taux (AC13).
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

### Pièges, par ordre de coût

1. **L'avoir (D5)** — le plus coûteux si oublié : corruption comptable silencieuse, équation du bilan toujours équilibrée, donc **aucun signal**. Le test AC17 « les deux écritures s'annulent compte par compte » est le garde-fou.
2. **Les 7 sites de colonnes (AC5 / AC5-bis)** — 4 d'entre eux sont des listes **écrites en dur**, hors `LINE_COLUMNS`. Un oubli ne casse pas la compilation : `sqlx::query_as` échoue au **runtime**, potentiellement seulement sur le chemin d'export ou d'avoir, donc pas forcément dans les tests rapides.
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

---

## Dev Agent Record

### Agent Model Used

_(à compléter par `dev-story`)_

### Debug Log References

### Completion Notes List

### File List
