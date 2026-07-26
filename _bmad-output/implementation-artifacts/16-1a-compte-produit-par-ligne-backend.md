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

Après validation, **aucune ligne d'une facture validée ne porte `NULL`**. C'est stable : une facture validée est immuable (`update` rejette tout statut ≠ `draft`).

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

Le bug est **antérieur à la story** (aujourd'hui déjà, facture = `défaut(T1)`, avoir = `défaut(T2)`), mais la story réécrit exactement ces deux helpers et érige l'invariant en décision — même logique que D4-bis.

### D2-bis — La matérialisation ne suffit pas : il faut un BACKFILL du parc existant

**Trou du patch de la passe 3, trouvé en passe 4.** D2 ne se déclenche que dans `validate_invoice`, c'est-à-dire à la seule transition `draft → validated`. Une facture **déjà validée** avant le déploiement n'y repassera **jamais** — `update` rejette tout statut ≠ `draft` (`invoices.rs:841`, `:1271`) et aucune autre écriture ne touche `invoice_lines` après validation. Après l'`ADD COLUMN` d'AC1, ces lignes portent `NULL` **définitivement**.

Or la spec l'écrit elle-même : le cas `NULL` est « le seul qui existe en production aujourd'hui ». Donc **100 % des factures déjà validées** — dont celles de l'instance de production en fonction — restent exposées au résidu que D2 vient de fermer pour les factures futures. Le patch de la passe 3 protégeait un ensemble **vide** à l'instant du déploiement.

**Décision** : la migration d'AC1 comporte un **backfill**, dont la source de vérité est l'**écriture comptable réellement générée**, pas le défaut courant.

**Pourquoi pas le défaut courant** : si l'administrateur a déjà changé `default_revenue_account_id` par le passé, backfiller avec la valeur d'aujourd'hui écrirait un compte que la facture n'a **jamais** crédité — on fabriquerait la corruption au lieu de la fermer. L'écriture générée est la **pièce probante** (c'est l'argument même de D2), et pour toute facture pré-existante elle contient **exactement une** ligne de crédit produit (structure `[0]` créance / `[1]` produit / `[2..]` TVA, docstring `invoices.rs:1120-1155`).

**Portée** : `invoice_lines` des factures `status = 'validated'` **et** `journal_entry_id IS NOT NULL`, plus le miroir sur `credit_note_lines` depuis l'écriture de contre-passation. Les factures `draft` restent `NULL` (D2), les `cancelled` sans écriture sont hors périmètre.

**Points à trancher en `dev-story`, à documenter dans le Change Log** : l'identification de la ligne de crédit produit dans l'écriture (par `line_order` positionnel — ordre canonique garanti par `create_in_tx` — ou par élimination des comptes créance et TVA), et le comportement pour une facture validée dont l'écriture aurait été supprimée par une voie directe (laisser `NULL` et le **compter**, pas échouer la migration).

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

**Ne PAS utiliser `company_invoice_settings::get_or_create_default_in_tx` sur le chemin de saisie.** Cette fonction fait un `INSERT IGNORE` puis un JOIN `accounts.active = TRUE` (`company_invoice_settings.rs:480-494`) et échoue en `InactiveOrInvalidAccounts` si le défaut société est archivé : **créer ou modifier un simple brouillon régresserait en `400`** pour une société dans cet état, sur une facture qui ne référence peut-être aucun compte. Elle prendrait en outre un verrou supplémentaire dans une transaction dont l'ordre est déjà contraint (`invoices.rs:806-812`, « ordre de verrous global companies → projects → invoices »). Ligne absente ou colonne `NULL` → `exempt_ids` vide, la contrainte `postable` s'applique alors sans exception.

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
- **AC2-bis** — **Backfill du parc existant (D2-bis)**, dans la même migration : `invoice_lines` des factures `status = 'validated' AND journal_entry_id IS NOT NULL` reçoivent le compte **effectivement crédité par leur écriture générée** ; miroir sur `credit_note_lines` depuis l'écriture de contre-passation. **Ne PAS** backfiller depuis `settings.default_revenue_account_id` courant (fabriquerait la corruption si le défaut a changé par le passé). Les lignes non backfillables (écriture absente) restent `NULL` et sont **comptées**, la migration ne doit pas échouer. Post-condition testée par `migrations_fresh_install` **et** par un test sur base pré-remplie : aucune ligne de facture validée avec écriture ne reste `NULL`.
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
  - Critère appliqué au compte par défaut : **`account_type = Revenue` uniquement.**
    - **`active` est déjà garanti en amont, hors du contrôle de cette story** — ne pas le re-tester, le cas est **inatteignable** depuis `validate_invoice`. Preuve : la config est chargée par `get_or_create_default_in_tx` (`invoices.rs:1310-1312`) dont le chemin « ligne existante » JOINe `accounts av ON av.id = cis.default_revenue_account_id AND av.active = TRUE` (`company_invoice_settings.rs:482`) et retourne `InactiveOrInvalidAccounts` si le défaut est archivé — **avant** tout point d'accroche possible pour AC8-bis.
    - **`postable` reste exempté** (même arbitrage que D3-bis).
    - `account_type` est en revanche un **vrai** trou : la route de configuration le vérifie à la pose (`routes/company_invoice_settings.rs:117-120`) mais rien ne le revérifie après un retypage par `accounts::update`.
  - Le message d'erreur le désigne explicitement comme « le compte de produit par défaut de la société », **pas** par un numéro de ligne — aucune ligne ne le porte.

### D. Backend — moteur comptable

- **AC9** — `generate_invoice_journal_lines` ventile le crédit produit : une ligne de crédit **par compte effectif**, montants `> 0`, tri `account_id` ASC (D4). La ligne `[0]` débit créance et les lignes TVA par taux sont **inchangées**. Lignes `NULL` et lignes pointant explicitement le défaut société fusionnent en une seule ligne.
- **AC9-bis** — **Matérialisation à la validation (D2)** : `validate_invoice` écrit le compte effectif dans `invoice_lines.revenue_account_id` pour toute ligne `NULL`, dans la **même transaction** que la création de l'écriture, **avant** l'appel à `create_in_tx`. Post-condition vérifiable : aucune ligne d'une facture de statut `validated` n'a `revenue_account_id IS NULL`.
  **La copie en mémoire doit être mutée aussi — l'`UPDATE` SQL seul ne suffit pas.** `lines_before` est chargé par `fetch_lines` à l'étape (1), donc **avant** la matérialisation, et il est réutilisé deux fois après elle :
  - `invoice_snapshot_json(&invoice_after, &lines_before)` pour le snapshot d'audit `"after"` (`invoices.rs:1448`) — qu'AC9-bis exige **post**-matérialisation ;
  - `ValidatedInvoice.lines` (`invoices.rs:1478`), **délibérément non re-fetché** post-commit (décision antérieure documentée `invoices.rs:1114`, « évite une fenêtre de race sur les lignes ») et rendu tel quel dans la réponse HTTP.
  Sans mutation en mémoire, l'audit et la réponse de l'endpoint de validation afficheraient `revenueAccountId: null` pour des lignes que la base vient de matérialiser — contredisant l'invariant même que la story introduit. **Muter les entrées de `lines_before` dont `revenue_account_id` était `NULL`** juste après l'`UPDATE`, **sans** réintroduire de re-fetch DB (la garantie anti-race de `invoices.rs:1114` doit être préservée).
- **AC10** — La section `# Équilibre par construction` de la docstring (`invoices.rs:1137-1142`) est **réécrite** — pas seulement complétée — pour couvrir la ventilation par compte en plus du filtre par taux, en reprenant l'argument de D4. L'hypothèse `F-OPUS-2` et la section `# Erreurs` restent à jour.
- **AC11** — `generate_credit_note_journal_lines` (`credit_notes.rs:139`) débite par compte, en miroir exact (D5) ; sa signature passe à `lines: &[(Decimal, Decimal, Option<i64>)]`. Sa docstring « inverse exact » reste vraie et est mise à jour. **Le site d'appel est mis à jour en conséquence** : `create_credit_note` (`credit_notes.rs:320-328`) construit aujourd'hui des paires via `.map(|l| (l.line_total, l.vat_rate))` — il doit produire des **triplets** `(l.line_total, l.vat_rate, l.revenue_account_id)`. Le paramètre scalaire `revenue_account_id` du helper devient le **repli** appliqué aux triplets dont le 3ᵉ membre est `None`, exactement comme côté facture.
- **AC11-bis** — Comportement D5-bis implémenté : compte du snapshot devenu `active = FALSE` → échec de l'émission de l'avoir avec message nommant ligne et compte. `postable` et `account_type` **ne sont pas** re-vérifiés côté avoir.
- **AC12** — **Non-régression, ancrée sur l'existant** : les tests unitaires actuels de `generate_invoice_journal_lines` (`invoices.rs:1996-2165`, 8 sites d'appel) passent **sans modification de leurs assertions** après la ventilation (leurs fixtures ont toutes `revenue_account_id = None`). Seule l'adaptation de signature est tolérée.
- **AC12-bis** — **Non-régression de la saisie** (corollaire du piège de D3-bis) : la **création** et la **modification** d'un brouillon réussissent pour une société dont `settings.default_revenue_account_id` est **archivé** ou `NULL`, tant qu'aucune ligne ne référence explicitement un compte invalide. Test d'intégration explicite — c'est le chemin le plus fréquent de l'application, et le seul AC qui attrape l'usage accidentel de `get_or_create_default_in_tx` à la saisie.
- **AC13** — Le rapport TVA n'est **pas** affecté : `kesh-report/src/vat_report.rs` ne lit que `default_vat_payable_account_id` / `default_vat_recoverable_account_id`, jamais un compte de produit. Un test d'intégration sur une facture **multi-comptes × multi-taux** vérifie que `reconciliation_status` reste `ok` (fichier `crates/kesh-report/tests/vat_report_reconciliation.rs`, nouveau cas).
- **AC13-bis** — D4-bis : `validate_invoice` (et l'émission d'avoir) rejettent une pièce dont `total_ht + total_vat == 0` avec une erreur métier `400` actionnable, au lieu du `500` SQL actuel sur `chk_jel_debit_credit_exclusive`.

### E. Exports

- **AC14** — `serialize_invoice_lines_csv` (`csv_tables.rs:459`) expose `revenue_account_id` dans l'en-tête et les enregistrements. Le test `crates/kesh-api/tests/exports_global_e2e.rs` est étendu pour vérifier la nouvelle colonne d'`invoice_lines.csv`.
- **AC14-bis** — **Aucun export CSV des lignes d'avoir n'existe** : `grep -n "credit_note" crates/kesh-api/src/exports/` revient vide, et les 20 entrées du ZIP (`exports_global_e2e.rs:621-634`) n'en contiennent pas. Aucune action requise de ce côté. Les compteurs existants ne sont **pas** affectés par un `ADD COLUMN` : ni `assert_eq!(entries.len(), 20)` (nombre de **fichiers**), ni `TABLES_TO_TRUNCATE` (23 **tables**) dans `admin_backup_e2e.rs`. La sauvegarde générique (`crates/kesh-db/src/backup.rs`) lit les colonnes dynamiquement via `non_generated_columns` — **auto-adaptée**, aucune modification.

### F. Tests & gate

- **AC15** — Tests unitaires du helper facture : mono-compte (non-régression AC12), multi-comptes, multi-comptes × multi-taux, ligne à montant nul filtrée, lignes `NULL` + explicite-même-compte fusionnées, ordre déterministe par `account_id`.
- **AC16** — Tests unitaires du helper avoir : miroir strict de AC15.
- **AC17** — Test d'intégration **pivot de D5**, en **deux** cas :
  1. facture ventilée sur ≥ 2 comptes puis avoir total → **les deux écritures s'annulent compte par compte** (agrégat par `account_id` de l'écriture facture + celle de l'avoir = 0 sur chaque compte) ;
  2. **facture à lignes toutes `NULL`, puis `settings.default_revenue_account_id` MODIFIÉ, puis avoir** → l'avoir débite le compte **effectivement crédité par la facture**, pas le nouveau défaut. Ce second cas **doit échouer** si la matérialisation d'AC9-bis n'est pas implémentée — c'est sa raison d'être. Un test qui ne change pas le défaut entre les deux passe systématiquement et ne prouve rien.
- **AC18** — Tests d'intégration : compte invalide à la saisie (création **et** modification) ; compte devenu non-`postable` au posting ; compte **retypé** au posting (le trou que `create_in_tx` ne couvre pas) ; compte archivé au posting ; compte archivé entre validation et avoir (AC11-bis) ; plusieurs lignes invalides simultanément (le message les nomme toutes).
- **AC18-bis** — Test d'AC8-bis, **deux** cas :
  1. facture dont **toutes** les lignes sont `NULL`, `settings.default_revenue_account_id` **retypé** `Revenue → Expense` entre le brouillon et la validation → échec nommant « le compte de produit par défaut de la société ». C'est le seul critère que la story ajoute réellement sur le défaut ;
  2. même défaut rendu **non-postable** → la validation **passe** (exemption D3-bis), l'écriture est générée normalement.
  *(Le cas « défaut archivé » n'est volontairement PAS testé ici : il est déjà rejeté en amont par `get_or_create_default_in_tx`. L'ajouter produirait un test qui documente le comportement d'une autre couche, et — risque réel — pousserait le dev à retoucher l'assertion pour accepter le `400` générique, ce qui enterrerait AC8-bis au lieu de le satisfaire.)*
- **AC19** — Test D3-bis : `default_revenue_account_id` pointant sur un compte **non-postable** ; une ligne `NULL` et une ligne le désignant explicitement produisent le même verdict et la même écriture.
- **AC20** — Test D1 : `settings.default_revenue_account_id` **≠** compte portant le rôle `DefaultRevenue` (deux comptes distincts) ; une ligne sans compte se poste sur `settings.default_revenue_account_id`.
- **AC21** — Test D4-bis : facture entièrement à zéro → `400` métier, pas `500`.
- **AC23** — Gate « Test Locally First » backend complet vert (`cargo fmt --all -- --check`, `cargo build --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`). Le gate **runtime complet** est requis si le doute subsiste sur `min_required` (P2-bis) — ici la migration est non-breaking, mais les suites `migrations_fresh_install` et `admin_backup_e2e` doivent passer.
- **AC24** — CHANGELOG `[Non publié]` : entrée orientée utilisateur. Le README et les manuels LaTeX sont traités en **16-1b** (le comportement n'est pas visible utilisateur tant que l'UI n'est pas livrée).

---

## Tasks / Subtasks

- [ ] **T1** — Migration `invoice_lines.revenue_account_id` + `credit_note_lines.revenue_account_id` + index + FK `ON DELETE RESTRICT` + **backfill du parc existant depuis les écritures générées** (D2-bis) ; ligne du tableau **et** compteurs agrégés de `docs/migrations-idempotence-audit.md` — noter que le backfill rend la migration **non idempotente au sens strict** et le justifier (AC1-AC4, AC2-bis).
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

### Pièges, par ordre de coût

0. **Le backfill (D2-bis)** — sans lui, tout le travail de D2 ne protège que les factures **futures**, et le parc en production reste exposé au résidu. Invisible en test sur base vierge : `migrations_fresh_install` passe, parce qu'il n'y a rien à backfiller. Exige un test sur base pré-remplie.
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

---

## Dev Agent Record

### Agent Model Used

_(à compléter par `dev-story`)_

### Debug Log References

### Completion Notes List

### File List
