# Story 16.1a : Compte de produit par ligne — socle backend

## Status

review

## Story

**As a** indépendant / PME / fiduciaire qui facture des **natures de prestations différentes** (honoraires, prestations de services, marchandises, produits annexes),
**I want** que chaque **ligne** de facture puisse porter son propre **compte de produit**, avec repli sur le compte de produit par défaut de la société quand rien n'est précisé,
**so that** l'écriture comptable générée à la validation **ventile le crédit produit sur les bons comptes** au lieu de tout créditer sur un compte unique — ce qui rend mon compte de résultat exploitable sans reclassement manuel a posteriori.

Issue : **#152**. Rattaché au CR **#265**. Socle de l'Epic 16 « Facturation avancée ».

**Périmètre de cette sous-story** : DB, entités, API, moteur comptable (facture **et** avoir), exports CSV, tests backend. Le sélecteur dans le formulaire de facture est en **16-1b**.

### Provenance du split (passe 1 de `validate`)

La story 16-1 initiale touchait **8 modules distincts** (seuil de la § « Règle de splitting préventif » de `CLAUDE.md` : 5). Split acté par Guy le 2026-07-26 en :

- **16-1a** (cette story) — backend : `kesh-db/migrations`, `kesh-db/repositories/invoices`, `kesh-db/repositories/credit_notes`, `kesh-api/routes/invoices`, `kesh-api/exports` = **5 modules** *au moment du split*. **Le livré en compte 7** (recompté en passe 5) : deux décisions ultérieures l'ont élargi sans que ce décompte suive — `kesh-api/routes/credit_notes` (arbitrage Guy en revue passe 1, exposition de `revenue_account_id` sur la réponse d'avoir) et `crates/kesh-i18n` (amendement de périmètre en revue passe 2). Sans conséquence rétroactive — le split a eu lieu et la revue converge — mais l'énoncé qui le justifiait n'était plus à jour.
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

- Le **sélecteur dans le formulaire de facture**, le déplacement d'`AccountAutocomplete` et le **doc-sync utilisateur (manuels LaTeX)** → **16-1b**.
  - ⚠️ **Amendé en revue passe 2** : l'**i18n backend** figurait dans cette exclusion, mais elle est **livrée par 16-1a** et devait l'être — les messages d'erreur d'AC7/AC8/AC11-bis sont composés côté `kesh-api` et exigent leurs clés dans `crates/kesh-i18n/locales/*/messages.ftl` (10 clés × 4 locales). L'exclusion visait l'i18n **frontend**. De même, le **README** est mis à jour ici, la § « Synchroniser le planning du README » de `CLAUDE.md` l'imposant à chaque commit. Les **manuels LaTeX** restent bien en 16-1b : documenter un sélecteur que l'utilisateur ne peut pas encore atteindre serait prématuré.
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

#### D1-bis — Le défaut société reste un **prérequis de configuration**, même si aucune ligne ne s'y replie

*(Limitation assumée, arbitrée par Guy le 2026-07-28 en revue de code passe 2 — remontée indépendamment par deux lentilles.)*

`validate_invoice` déballe `settings.default_revenue_account_id` par `.ok_or_else(|| ConfigurationRequired(…))` **inconditionnellement** (`invoices.rs:1593-1595`), avant tout calcul par ligne. Une société dont la colonne est `NULL` et dont **toutes** les lignes portent un compte explicite — le cas d'usage nominal du CR #265 — est donc refusée à la validation, pour un compte dont la ventilation n'a aucun besoin.

**Pourquoi c'est conservé tel quel** :

- L'erreur est **bruyante et nommée** (`CONFIGURATION_REQUIRED`, champ `default_revenue_account_id`), pas une perte silencieuse. Le contournement est immédiat : renseigner un défaut, même jamais utilisé.
- Le traitement est **identique à `default_receivable_account_id`**, déballé deux lignes plus haut. Rendre l'un paresseux sans l'autre créerait une asymétrie non justifiée.
- Le rendre paresseux exigerait de passer **les deux générateurs d'écritures** de `i64` à `Option<i64>` (`invoices.rs:1415`, `credit_notes.rs:165`), leurs sites d'appel et ~23 tests unitaires, plus une branche d'erreur nouvelle (« ligne `NULL` mais aucun défaut »). C'est un refactor du cœur comptable pour un cas de configuration que l'onboarding rend rare — mauvais rapport risque/bénéfice immédiatement après une revue.

**Portée** : atteignable uniquement si la colonne est **délibérément vidée** ; l'onboarding la renseigne. Le cas « ligne de configuration présente, colonne `NULL` » est désormais couvert par `draft_crud_survives_null_company_default_column` (la **saisie** passe ; c'est la **validation** qui exige le défaut).

**Argument versé en passe 3 (la décision ne change pas, mais il éclaire la dette).** Le finding est remonté par une lentille différente à chacune des passes 2 et 3 — trois fois au total. La passe 3 apporte un angle que l'arbitrage initial n'avait pas : ce n'est pas seulement une fonctionnalité manquante, c'est une **incohérence interne entre trois chemins du même repository**. `create()` (`:640`) et `update()` (`:1102`) lisent le défaut en `Option<i64>` et **tolèrent** son absence — l'exemption D3-bis ne s'applique alors simplement à aucun compte ; seul `validate_invoice` (`:1593`) le déballe par `.ok_or_else(…)` et refuse. Concrètement : **on accepte d'enregistrer le brouillon, puis on refuse de le valider**, sur une configuration inchangée. Si la dette est un jour reprise, c'est cette asymétrie — et non le cas d'usage « je ventile tout explicitement » — qui en constitue le meilleur argument.

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

### D2-bis — Le parc existant est traité par la story 16-1a-bis (EXTRAITE)

D2 ne se déclenche que dans `validate_invoice`, à la seule transition `draft → validated`. Une facture **déjà validée** avant le déploiement n'y repassera **jamais** (`update` rejette tout statut ≠ `draft`, `invoices.rs:841`, `:1271`). Après l'`ADD COLUMN` d'AC1, ses lignes portent `NULL` **définitivement** — et `NULL` est *le seul cas qui existe en production aujourd'hui*. **16-1a seule protège donc un ensemble vide à l'instant du déploiement.**

Le correctif — un **backfill du parc existant depuis les écritures comptables réellement générées** — faisait initialement partie de cette story (décision D2-bis, AC2-bis, volet backfill de T1). Il a été **extrait dans la story `16-1a-bis` « Backfill du parc existant »** le 2026-07-26, sur arbitrage de Guy, après que le second critère de la § « Règle de splitting préventif » a été coché : sur les passes 5 et 6 de `validate`, **7 findings sur 10, dont les 3 HIGH, portaient sur le seul backfill**, tandis que le reste de 16-1a n'était plus remis en cause depuis la passe 3.

**Le split est sûr, et c'est démontrable** : sans 16-1a-bis, le parc antérieur conserve **exactement** le comportement d'aujourd'hui (repli sur le défaut courant au moment de l'avoir). Le backfill est **strictement additif** — il ferme un bug pré-existant, il n'en introduit aucun. C'est la différence avec D5, qui devait impérativement rester dans la même story que sa cause.

**Ce que 16-1a doit en retenir** :

- **ne rien backfiller** — la migration d'AC1 se limite à `ADD COLUMN` + index + FK ;
- l'invariant d'AC9-bis ne vaut donc **que** pour les factures validées **par ce binaire** (cf. la borne posée en D2 et reprise dans AC9-bis) ;
- **aucune post-condition globale** du type « aucune ligne de facture validée n'a `revenue_account_id IS NULL` » ne doit être écrite ni testée : elle serait fausse tant que 16-1a-bis n'est pas livrée, et elle le resterait partiellement après (le backfill de 16-1a-bis est délibérément conservateur).

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

**Fenêtre assumée (dette LOW documentée).** L'exemption est indexée sur `settings.default_revenue_account_id` **au moment de chaque contrôle**. Si l'administrateur change le défaut entre la saisie et le posting, une ligne pointant explicitement l'**ancien** défaut non-postable est acceptée à la saisie puis rejetée au posting — même donnée, verdicts opposés. Comportement assumé : l'alternative (figer l'exemption dans la ligne à la saisie) rendrait le brouillon dépendant d'un état de configuration périmé, ce que D2 refuse par ailleurs. Le message du posting doit **le rendre lisible**. ~~« Ligne {n} : le compte {numéro} n'est pas imputable et n'est plus le compte de produit par défaut de la société — choisissez un autre compte »~~ — **formulation corrigée en revue de code passe 1 (2026-07-28)** : elle n'est vraie que dans la fenêtre décrite ici, alors que le variant `NotPostable` se déclenche aussi — et bien plus souvent — pour un compte non imputable qui n'a **jamais** été le défaut de la société. Dire « n'est **plus** le défaut » y présuppose un état faux et envoie l'utilisateur inspecter ses réglages de société. Formulation retenue, vraie dans les deux cas : « Ligne {n} : le compte {numéro} n'est pas imputable — choisissez un autre compte ». Hors périmètre des tests d'AC18 (fenêtre de configuration, aucune correction comptable en jeu).

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
- **AC2-bis** — **Aucun backfill dans cette story.** La migration se limite à `ADD COLUMN` + index + FK. Le traitement du parc existant est **entièrement** porté par `16-1a-bis` (cf. D2-bis). Corollaire à respecter : **ne pas** écrire de post-condition globale « aucune ligne de facture validée n'a `revenue_account_id IS NULL` » — elle est fausse tant que 16-1a-bis n'est pas livrée, et partiellement fausse après (son backfill est délibérément conservateur). Les post-conditions de cette story se testent **sur la facture que le test vient de valider**, jamais par un `COUNT(*)` global.
- **AC3** — Migration **non-breaking** (`ADD COLUMN` nullable + index + FK) → **pas** de bump `kesh_version_min_required` (politique P1/P2), donc **pas** de bump de version Cargo (P2-bis). Le vérifier explicitement.
- **AC4** — `docs/migrations-idempotence-audit.md` : ligne ajoutée au tableau détaillé avec verdict et justification, **ET** récapitulatif agrégé de bas de fichier mis à jour en cohérence (verdict attendu `tracked-by-sqlx`, la migration n'utilisant pas `IF NOT EXISTS`). Garde-fou **P5**. ⚠️ **Les compteurs se RECOMPTENT depuis le tableau, ils ne s'incrémentent pas** — cet AC prescrivait « `Total` actuellement 55, `tracked-by-sqlx` actuellement 44, +1 chacun », et le `44` était **faux de 7** : la revue de code passe 1 a établi 52 réels, corrigé le fichier, et cet AC n'avait pas suivi (rectifié en passe 5). Valeurs vraies après la story, recomptées : **56 = 52 `tracked-by-sqlx` + 4 `yes` + 0 `no`**. Ré-incrémenter depuis les anciens nombres à la prochaine migration ré-armerait exactement la dérive que P5 existe pour empêcher.

### B. Backend — entité et tous les sites de colonnes

- **AC5** — L'entité `InvoiceLine` porte `revenue_account_id: Option<i64>`, et **les 3 sites qui listent les colonnes `invoice_lines`** sont mis à jour — sans quoi `sqlx::query_as` échoue au runtime :
  1. `LINE_COLUMNS` (`invoices.rs:43`) — utilisé par `insert_lines` (`:386`) et `fetch_lines` (`:425`) ;
  2. `list_all_lines_by_company` (`invoices.rs:1937-1944`) — **liste en dur avec préfixes `il.`**, alimente l'export ZIP global ;
  3. `create_credit_note` (`credit_notes.rs:328`) — lit `invoice_lines` pour le snapshot d'avoir. ⚠️ **Ancre et qualification rectifiées en passe 6** : cet AC désignait `:266-268` comme une **liste en dur**, mais l'implémentation a remplacé cette duplication par un **emprunt de `invoices::LINE_COLUMNS`** (écart assumé, documenté au Dev Agent Record). L'ancienne ancre pointe aujourd'hui sur un bloc `use` sans rapport. Ce site n'est donc plus à maintenir à la main — il suit `LINE_COLUMNS`.
  `invoice_snapshot_json` (`invoices.rs:51-60`) inclut le compte dans le snapshot d'audit.
- **AC5-bis** — Côté avoir, l'entité `CreditNoteLine` (`crates/kesh-db/src/entities/credit_note.rs:45`) porte le champ, et **les 4 sites** suivants sont mis à jour : `credit_note_snapshot_json` (`credit_notes.rs:36-56`), `fetch_credit_note_lines` (`:63-64`), le second `SELECT` de `get()` (`:90-91`), et l'`INSERT credit_note_lines` (`:376-390`). Il n'existe **pas** de constante `LINE_COLUMNS` côté avoir — soit en introduire une, soit traiter les 3 `SELECT` un par un ; ne pas en oublier.

  **Nature exacte du changement, site par site** — chaque `SELECT` ajoute `revenue_account_id` à sa liste de colonnes ; l'`INSERT` (`:376-390`) l'ajoute **à la fois** à la liste de colonnes **et** à la chaîne de `.bind()` (`.bind(line.revenue_account_id)`), et son `VALUES` gagne un `?` ; les deux `*_snapshot_json` ajoutent la clé au JSON. Un `SELECT` mis à jour sans son `INSERT`, ou l'inverse, ne casse pas la compilation.

- **AC5-ter** — ~~**Décompte de référence : 8 sites au total** — 6 listes de colonnes SQL (`invoices.rs:39` / `:1937` / `credit_notes.rs:266` / `:63` / `:90` / `:376`) + 2 snapshots d'audit (`invoices.rs:51` / `credit_notes.rs:36`). S'y ajoute **1 site d'appel** (`credit_notes.rs:320-328`, cf. AC11) qui, lui, est attrapé par le compilateur. Si le décompte ne tombe pas sur 8 + 1, il manque quelque chose.~~

  ⚠️ **SUPERSEDED — ne pas utiliser comme checklist.** Le décompte « 8 + 1 » a été démenti à l'implémentation, puis une seconde fois en revue. Il a bougé dans **les deux sens** : `insert_lines` (`invoices.rs`), `is_no_op_change` et `InvoiceLineResponse` n'étaient pas comptés ; à l'inverse le `SELECT` en dur de `credit_notes.rs:266` a **disparu** (il emprunte désormais `invoices::LINE_COLUMNS`, cf. Dev Agent Record) ; et la revue de passe 1 a **ajouté** `CreditNoteLineResponse` (`routes/credit_notes.rs`). Un critère numérique figé sur une cible mouvante ne peut pas servir de garde-fou — c'est le `grep` du symptôme sur le dépôt qui fait foi, pas le compte. Le livrable, lui, couvre tous les sites réels (vérifié indépendamment en passes 1 et 2).

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
- **AC9-bis** — **Matérialisation à la validation (D2)** : `validate_invoice` écrit le compte effectif dans `invoice_lines.revenue_account_id` pour toute ligne `NULL`, dans la **même transaction** que la création de l'écriture, **avant** l'appel à `create_in_tx`. **Post-condition vérifiable, bornée à son domaine réel (passe 6)** : aucune ligne de **la facture que le test vient de valider** n'a `revenue_account_id IS NULL`. **Surtout PAS** un `COUNT(*) = 0` global sur `invoice_lines` filtré `status = 'validated'` : sur une base contenant des factures validées **avant** le déploiement, il **échouerait** — celles-ci ne repassent jamais par `validate_invoice` et gardent `NULL` (cf. D2-bis / `16-1a-bis`). Le dev n'aurait alors que deux issues : dénaturer l'AC, ou perdre du temps à découvrir la contradiction.
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
- **AC14-bis** — **Aucun export CSV des lignes d'avoir n'existe** : `grep -n "credit_note" crates/kesh-api/src/exports/` revient vide, et les 20 entrées du ZIP (`exports_global_e2e.rs:621-634`) n'en contiennent pas. Aucune action requise de ce côté. Les compteurs existants ne sont **pas** affectés par un `ADD COLUMN` : ni `assert_eq!(entries.len(), 20)` (nombre de **fichiers**), ni `TABLES_TO_TRUNCATE` (**37** tables — recompté en passe 5 ; le « 23 » écrit ici recopiait un commentaire périmé d'`admin_backup_e2e.rs:126`, sans conséquence sur la conclusion). La sauvegarde générique (`crates/kesh-db/src/backup.rs`) lit les colonnes dynamiquement via `non_generated_columns` — **auto-adaptée**, aucune modification.
  **Condition de validité de cette clause (passe 6)** : elle tient parce que **cette story ne crée aucune table**. Si une passe ultérieure introduisait une table, cette clause et le périmètre de l'export/import d'installation devraient être révisés ensemble — `backup_inventory_matches_schema` (`backup.rs:577-606`) tomberait et imposerait de modifier `TABLES_TO_TRUNCATE`. La même garde vaut pour `16-1a-bis`, qui a explicitement écarté la table de rapport pour ce motif.

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
- **AC24** — CHANGELOG `[Non publié]` : entrée orientée utilisateur. Les **manuels LaTeX** sont traités en **16-1b** (documenter un sélecteur que l'utilisateur ne peut pas encore atteindre serait prématuré). ⚠️ **Le README, lui, est mis à jour ICI** — rectifié en passe 5 : cet AC disait « le README et les manuels sont traités en 16-1b », en contradiction frontale avec l'amendement de périmètre de la passe 2 (§ « Ce qui n'est PAS dans 16-1a ») **et** avec la branche, qui modifie bien `README.md`. La § « Synchroniser le planning du README » de `CLAUDE.md` l'impose à chaque commit.

---

## Tasks / Subtasks

- [x] **T1** — Migration `invoice_lines.revenue_account_id` + `credit_note_lines.revenue_account_id` + index + FK `ON DELETE RESTRICT`. **Aucun backfill** (extrait en `16-1a-bis`). Ligne du tableau **et** compteurs agrégés de `docs/migrations-idempotence-audit.md`, verdict `tracked-by-sqlx` justifié par les `ADD COLUMN` **sans** `IF NOT EXISTS` (erreur 1060 au re-jeu hors sqlx). **Ne pas** écrire « non idempotente » : le fichier maintient l'invariant « Idempotence `no` : 0 » (`migrations-idempotence-audit.md:85` — ancre rectifiée en passe 6 : la réparation du fichier par la **passe 1 de cette même story** a décalé la section « Statistiques » de 14 lignes, sans que ce renvoi suive) et un verdict `no` ferait diverger les compteurs d'AC4 (AC1-AC4).
- [x] **T2** — Entités `InvoiceLine` / `CreditNoteLine` + **les 8 sites** listés en AC5 / AC5-bis (6 listes de colonnes SQL + 2 snapshots d'audit), décompte de référence en AC5-ter (AC5, AC5-bis). ⚠️ **AC5-ter est SUPERSEDED** (revue passe 2) — ne pas s'en servir comme checklist, cf. sa mention.
- [x] **T3** — API : DTOs création/modification avec `#[serde(default)]` + réponse de lecture (AC6 — la clause de test des deux formes de payload y est déjà portée).
- [x] **T4** — Helper de validation batchée des comptes de ligne, réutilisable saisie + posting, avec exemption D3-bis et message multi-lignes (D6, D3-bis).
- [x] **T5** — Branchement de T4 à la saisie (`invoices.rs:459` création, `:816` modification), en lisant le défaut par un `SELECT` nu — **jamais** `get_or_create_default_in_tx` (D3-bis) (AC7, AC12-bis).
- [x] **T6** — `generate_invoice_journal_lines` : ventilation `BTreeMap` + docstring réécrite (AC9, AC10).
- [x] **T7** — Branchement de T4 au posting dans `validate_invoice`, `SELECT` sans verrou, commentaire ABBA ; ensemble re-validé = comptes de ligne ∪ défaut si ≥ 1 ligne `NULL` ; **matérialisation du compte effectif** avant `create_in_tx` (D2) (AC8, AC8-bis, AC9-bis).
- [x] **T8** — `generate_credit_note_journal_lines` en miroir + copie du compte à la création de l'avoir + garde D5-bis (AC11, AC11-bis).
- [x] **T9** — Rejet des pièces à montant total nul (AC13-bis, AC21).
- [x] **T10** — Export CSV `invoice_lines` + extension du test `exports_global_e2e` (AC14).
- [x] **T11** — Tests unitaires (AC15, AC16) et d'intégration (**AC12**, AC12-bis, AC17 **les deux cas**, AC18, AC18-bis, AC19, AC20, AC21), dont le cas TVA multi-comptes × multi-taux (AC13). *(AC12 rattaché en passe 5 : il n'était cité par aucune tâche — orphelin de traçabilité, alors que la passe 7 de `validate` déclarait « aucun orphelin ». Il est tenu en fait, les 8 tests `gen_lines_*` pré-existants passant sans retouche d'assertion.)*
- [x] **T12** — CHANGELOG (AC24) + gate backend complet (AC23).

**Ordre conseillé** : T1 → T2 → T6 (le helper d'abord, testable en isolation) → T4 → T5 → T7 → T8 → T9 → T3 → T10 → T11 → T12.

### Review Findings

**Passe 1 de `bmad-code-review`** — 2026-07-28, 3 lentilles Sonnet (BlindHunter / EdgeCaseHunter / AcceptanceAuditor) + vérification d'orchestrateur (Opus 5). Tous les findings CRITICAL/HIGH/MEDIUM ont été vérifiés ground-truth (`grep -nF` + bornage de la fonction englobante) avant rétention. 0 faux positif retenu.

**Gate backend rejoué hors sandbox sur la DB `kesh_gate`** — *état constaté à l'**ouverture** de la passe 1* : `fmt` ✅ / `build` ✅ / `clippy -D warnings` ✅ 0 warning / `nextest` ❌ **exit 100** — AC23 **NON satisfait à ce moment-là**, contrairement à ce qu'affirmait T12.

> ✅ **Refermé en fin de passe 1** — après application des patches, le gate complet ressort **exit 0, 2059/2059**. Voir « Gate final — VERT » dans le Change Log ci-dessous. **AC23 est satisfait**, T12 peut rester coché. Cette ligne-ci décrit le point de départ, pas l'état livré.

- [x] [Review][Patch] **`CreditNoteLineResponse` n'expose pas `revenue_account_id`** — asymétrie avec `InvoiceLineResponse` (`routes/invoices.rs:159,174`), qui l'expose. Le client HTTP ne peut pas savoir quel compte chaque ligne d'avoir a débité, alors que le CHANGELOG annonce une ventilation « en miroir ». *Arbitrage Guy 2026-07-28 : le champ est ajouté dans 16-1a* — cohérent avec le périmètre annoncé (« API + moteur comptable facture ET avoir », D5). [`crates/kesh-api/src/routes/credit_notes.rs:32-52`]
- [x] [Review][Patch] **Le test AC14 échoue — la fixture ne produit aucune ligne de facture, le gate est rouge** [`crates/kesh-api/tests/exports_global_e2e.rs:472-493`]
- [x] [Review][Patch] **Le snapshot d'audit `before` est pollué par la mutation en place de `lines_before`** [`crates/kesh-db/src/repositories/invoices.rs:1692,1824`]
- [x] [Review][Patch] **Le message `NotPostable` affirme « n'est plus le compte de produit par défaut »** alors que la condition est « n'est pas le défaut » [`crates/kesh-api/src/errors.rs:103-108` + les 4 `.ftl`]
- [x] [Review][Patch] **AC6 — le test de désérialisation exigé est absent** (clé omise vs `"revenueAccountId": null` explicite) [`crates/kesh-api/src/routes/invoices.rs:81`]
- [x] [Review][Patch] **Compteur `tracked-by-sqlx` faux : 45 déclaré vs 52 réel** — dérive pré-existante de 7, propagée mécaniquement par l'incrément 44→45 [`docs/migrations-idempotence-audit.md:71`]
- [x] [Review][Defer] **AC5-ter — le décompte « 8 + 1 » de la spec est faux**, 3 sites réels non comptés [spec AC5-ter] — deferred, défaut de spec sans conséquence sur le livrable (le code couvre bien les 3 sites)
- [x] [Review][Defer] **Chemin avoir : un compte introuvable est étiqueté `Inactive` et son numéro sort vide** [`crates/kesh-db/src/repositories/credit_notes.rs:432-444`] — deferred, chemin inatteignable (FK `ON DELETE RESTRICT`)
- [x] [Review][Defer] **Duplication de la collecte des sites** — sentinelle `0: i32` côté avoir vs `Option<i32>` côté facture [`crates/kesh-db/src/repositories/credit_notes.rs:387-394`] — deferred, dette de lisibilité LOW

**Passe 2** — 2026-07-28, 3 lentilles **Opus** (rotation : Sonnet en passe 1) sur **diff aplati `main..HEAD`**. 25 findings bruts → 21 après déduplication. **0 CRITICAL, 0 HIGH survivant** — les 2 HIGH remontés ont été réfutés en ground-truth. 3 convergences entre lentilles indépendantes.

- [x] [Review][Patch] **Le commentaire de `update()` promet un grandfathering par ligne qui n'existe pas** — le code re-valide tous les comptes de `changes.lines` sans comparer à `before_lines` ; le tag projet, lui, compare réellement. *Convergence EdgeCaseHunter + AcceptanceAuditor.* [`crates/kesh-db/src/repositories/invoices.rs:1081-1087`]
- [x] [Review][Patch] **La spec déclarait « AC23 NON satisfait » sans pointer vers sa clôture**, 261 lignes plus loin [spec `:332`]
- [x] [Review][Patch] **`assert!(msg.contains("2"))` satisfait par le compte `"3200"`** — l'assertion passerait même si le sujet disparaissait du message [`crates/kesh-api/src/errors.rs:2634`]
- [x] [Review][Patch] **Doc de `generate_credit_note_journal_lines` : dit « snapshot `credit_note_lines` », le code lit `invoice_lines`** [`crates/kesh-db/src/repositories/credit_notes.rs:157`]
- [x] [Review][Patch] **`reason` absent du body d'erreur avoir**, présent côté facture pour la même structure [`crates/kesh-api/src/errors.rs:2289`]
- [x] [Review][Patch] **AC5-ter marqué SUPERSEDED** — le décompte a bougé dans les deux sens, dont un site **ajouté par la revue de passe 1** [spec `:253`]
- [x] [Review][Patch] **Le périmètre excluait i18n et README que la branche livre** — exclusion amendée (visait l'i18n *frontend* ; les manuels restent en 16-1b) [spec `:67`]
- [x] [Review][Patch] **AC12-bis : branche « colonne `NULL` » non testée** (seuls « archivé » et « ligne absente » l'étaient) → `draft_crud_survives_null_company_default_column`
- [x] [Review][Decision] **`PUT` sans `revenueAccountId` efface le compte** — *convergence BlindHunter + EdgeCaseHunter.* **Arbitrage Guy : CR dédié plutôt que correctif ici** → **issue #278** « Durcir le contrat des API en écriture ». Mesure provisoire : avertissement clients API au CHANGELOG. Motif : une sémantique « conserver » est impossible tant que les lignes n'ont pas d'identité stable (`update` fait `DELETE`+`INSERT`), et réapparier par position rouvrirait le piège que la passe 5 de `validate` a réfuté.
- [x] [Review][Decision] **Défaut société exigé même si toutes les lignes sont explicites** — *convergence BlindHunter + EdgeCaseHunter.* **Arbitrage Guy : garder + documenter** → nouvelle décision **D1-bis**. Erreur bruyante, contournement trivial, cohérence avec `default_receivable_account_id`.
- [x] [Review][Decision] **Avoir sur facture antérieure à 16-1a : contrôle du défaut COURANT, résidu silencieux** — **Arbitrage Guy : test de frontière** → `legacy_invoice_credit_note_falls_back_to_current_default_known_limitation`, qui chiffre le résidu et **devra changer de verdict** quand 16-1a-bis livrera le backfill.
- [x] [Review][Defer] **Double isolation BiDi Fluent** dans `error.message` — deferred, motif préexistant dans tout le dépôt
- [x] [Review][Defer] **`creditNoteTotalZero` sans test car inatteignable** par ce binaire — deferred
- [x] [Review][Defer] **Positions 1-based et `unwrap()` fragiles** dans le test d'export — deferred
- [x] [Review][Defer] **Manuels LaTeX** → 16-1b, documenter un sélecteur inatteignable serait prématuré
- [x] [Review][Defer] **`migrations_upgrade_path` : la fenêtre ne couvre pas l'état pré-10-2 annoncé** (frontière 34, migration 10-2 = 27ᵉ) — commentaire rendu factuel, correction de la frontière = décision de périmètre

**Passe 3** — 2026-07-28, 3 lentilles **Sonnet** (rotation : Opus en passe 2) sur **diff aplati `main..HEAD`** (4048 lignes). **6 findings : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 3 LOW.** Aucune convergence inter-lentilles cette fois — chaque finding vient d'une seule lentille, ce qui est cohérent avec un diff déjà passé deux fois au filtre.

- [x] [Review][Patch] **La doc de `generate_credit_note_journal_lines` affirme un verrou qui n'existe pas** — le correctif de passe 2 avait écrit que les `invoice_lines` sont « verrouillées » ; **aucun `FOR UPDATE` ne porte sur cette table**, son `SELECT` est nu. Les seuls `FOR UPDATE` du diff visent `accounts` (D5-bis) et `company_invoice_settings` (D3-bis). *BlindHunter, MEDIUM, avec incertitude explicite levée en ground-truth : le verrou réel est sur `invoices`.* Doc réécrite pour nommer la protection **indirecte** et signaler qu'une seconde voie de mutation la casserait. [`crates/kesh-db/src/repositories/credit_notes.rs:157-177`]
- [x] [Review][Patch] **`draft_crud_survives_null_company_default_column` ne couvre pas la branche qu'il annonce** — le test de passe 2 n'utilisait qu'un compte **déjà imputable** et n'assertait que la persistance ; il serait passé même si `.flatten()` avait confondu « colonne `NULL` » et « ligne absente », c'est-à-dire même si la branche qu'il prétend protéger avait disparu. *EdgeCaseHunter, MEDIUM.* Seconde assertion ajoutée : compte explicite **non imputable** → rejet `NotPostable`, miroir exact de `missing_settings_row_disables_postable_exemption`. ⚠️ **Le commentaire accompagnant ce patch a lui-même été corrigé en passe 4** — il présentait cette seconde assertion comme « discriminante », ce qu'elle n'est pas. [`crates/kesh-db/tests/invoices_line_revenue_account.rs:1117-1200`]
- [x] [Review][Decision] **Défaut société exigé même si toutes les lignes sont explicites** — **troisième remontée du même finding**, par une lentille différente à chaque passe. *EdgeCaseHunter, MEDIUM.* **L'arbitrage Guy de passe 2 (garder + documenter, D1-bis) n'est pas rouvert**, mais la passe 3 apporte un angle neuf que l'arbitrage initial n'avait pas : ce n'est pas une fonctionnalité manquante, c'est une **incohérence interne entre trois chemins du même repository** — `create()` et `update()` tolèrent le défaut absent (`Option<i64>`), seul `validate_invoice` le déballe et refuse. Argument versé à D1-bis pour le jour où la dette sera reprise. [`crates/kesh-db/src/repositories/invoices.rs:1604-1606` vs `:640`, `:1102`]
- [x] [Review][Patch] **`generate_invoice_journal_lines` promu `pub(crate)` — portée plus large que l'intention documentée.** *BlindHunter, LOW.* Le commentaire « aucun appel de production hors de ce module » était une convention, pas une garantie ; un appel futur depuis n'importe où dans `kesh-db` court-circuiterait la re-validation D3/D3-bis et la matérialisation D2 **sans que le compilateur ne dise rien**. → `pub(in crate::repositories)`, qui couvre exactement le besoin (module frère `credit_notes`). [`crates/kesh-db/src/repositories/invoices.rs:1423`]
- [x] [Review][Patch] **Décompte de tests de la File List faux : « 22 » déclaré pour 25 réels.** *AcceptanceAuditor, LOW.* Même symptôme que la dérive d'AC5-ter et celle de `migrations-idempotence-audit.md` — un nombre écrit une fois n'est jamais recompté. Corrigé **en recomptant la source** (`grep -c '#\[sqlx::test'`) : 23 au dev, +2 en passe 2 = **25**. Le « 22 » d'origine était déjà faux d'une unité.
- [x] [Review][Patch] **T2 renvoie encore à AC5-ter comme « décompte de référence »** sans note de statut, alors que la passe 2 l'a marqué SUPERSEDED. *AcceptanceAuditor, LOW.* Avertissement ajouté à T2.

**Passe 4** — 2026-07-29, 3 lentilles **Haiku 4.5** (rotation : Sonnet en passes 1 et 3, Opus en passe 2) sur **diff aplati mono-commit** de 35 fichiers, conformément à la mitigation § « Haiku-specific guardrails » de `CLAUDE.md`. **1 finding : 0 CRITICAL, 0 HIGH, 1 MEDIUM.** Aucun finding affirmant une absence de code — la vérification `grep -nF` obligatoire n'a donc eu aucun candidat à réfuter, et **aucune hallucination n'a été observée sur cette passe**.

- [x] [Review][Patch] **La seconde assertion ajoutée en passe 3 ne discrimine pas la branche qu'elle prétend distinguer.** *BlindHunter, MEDIUM — vérifié en ground-truth par l'orchestrateur avant rétention.* Le rejet `NotPostable` découle de la condition `!postable && Some(account_id) != default_revenue_account_id` avec `default = None` : il vaut **quelle que soit** la branche qui a produit ce `None`. Or `.flatten()` fait converger « colonne `NULL` » et « ligne absente » sur exactement ce `None` — c'est sa raison d'être. L'assertion épingle donc une **équivalence**, pas un discriminant, et le commentaire de passe 3 affirmait l'inverse. **Le remède proposé par la lentille est en revanche incohérent** (« tester un compte qui *est* le défaut société, rendu non imputable ») : dans ce test le défaut est `NULL` par construction, il n'existe aucun compte par défaut à rendre non imputable — diagnostic retenu, remède écarté. ⚠️ **La correction rédigée en réponse à ce finding était elle-même fautive et a été reprise en passe 5** : elle attribuait à l'assertion (1) le monopole du *décodage* d'un `NULL` en `Option<Option<i64>>`, ce que le code contredit. [`crates/kesh-db/tests/invoices_line_revenue_account.rs:1117-1200`]

**Passe 5** — 2026-07-29, 3 lentilles **Opus** (rotation : Haiku en passe 4) sur diff aplati mono-commit. Lentilles recentrées là où les défauts se logent réellement à ce stade : (a) les **correctifs des passes 1 à 4** eux-mêmes, (b) les **frontières** non explorées du code, (c) la **cohérence interne du document** après 11 passes d'amendement. **14 findings : 0 CRITICAL, 0 HIGH, 6 MEDIUM, 8 LOW.** La lentille « frontières » rend **0 finding** avec une trace vérifiable. **Aucun finding ne porte sur le code livré** : 13 portent sur de la documentation, 1 sur une omission du CHANGELOG.

- [x] [Review][Patch] **Le correctif de passe 4 affirmait une exclusivité fausse** : « l'assertion (1) est la seule à exercer le décodage d'un `NULL` en `Option<Option<i64>>` ». Faux — la lecture du défaut est déclenchée par la seule présence d'un compte explicite sur une ligne (`invoices.rs:640`, `if !sites.is_empty()`), et l'assertion (2) porte elle aussi un compte explicite : **les deux** décodent. Sur un refactor vers `Option<i64>` nu, l'assertion (2) échouerait également (`expect_invalid_accounts` panique sur tout variant ≠ `InvalidRevenueAccounts`). **MEDIUM.** Réécrit sans aucune claim d'exclusivité : ce qui est propre à ce test est son **montage**, pas l'une de ses assertions. [`invoices_line_revenue_account.rs:1117-1200`]
- [x] [Review][Patch] **Le patch de passe 2 sur `migrations_upgrade_path.rs` n'était pas propagé** : le doc-comment de la fonction (`:55-58`) portait encore « 23 migrations appliquées + les 5 dernières », contredit 47 lignes plus bas par le commentaire inline corrigé (34 / 22) — et c'est le doc-comment qu'un survol de signature expose en premier. **MEDIUM.** Risque concret : ré-armer exactement la régression que le garde-fou **P6** vient d'être codifié pour empêcher. [`migrations_upgrade_path.rs:55-58`]
- [x] [Review][Patch] **Le doc-comment de `CreditNoteLineResponse.revenue_account_id` décrivait un contrat que le parc dément.** Il affirmait « toujours renseigné » et « `Option` uniquement pour refléter la colonne nullable » : les deux sont faux pour toute facture validée avant 16-1a — soit l'essentiel du parc en exploitation, 16-1a-bis n'étant pas livrée. `null` n'y signifie pas « aucune imputation » mais « défaut société au moment de l'avoir ». **MEDIUM.** Doc réécrite avec renvoi au test de frontière et au CHANGELOG. [`routes/credit_notes.rs:39-47`]
- [x] [Review][Patch] **AC4 et la File List prescrivaient encore le compteur faux** (`44→45`) que la passe 1 avait établi comme dérivé de 7 et corrigé dans `docs/`. Le patch n'avait pas été grepé sur la spec — **dont un AC formel**. **MEDIUM.** Un dev ré-appliquant AC4 littéralement à la prochaine migration ré-armait la dérive. [spec AC4, File List]
- [x] [Review][Patch] **Contradiction frontale AC24 ↔ § « Ce qui n'est PAS dans 16-1a » sur le README** : l'amendement de passe 2 disait « le README est mis à jour ici », AC24 disait « traité en 16-1b ». La branche modifie bien `README.md` — c'est AC24 qui était faux. **MEDIUM.** [spec AC24]
- [x] [Review][Patch] **Le « piège n° 2 » restait une checklist active bâtie sur AC5-ter, marqué SUPERSEDED** — la passe 3 avait propagé l'avertissement à T2 et pas à lui — **et** son décompte « 4 listes en dur » était démenti par le livré (`create_credit_note` emprunte désormais `LINE_COLUMNS`, il en reste 3). **MEDIUM.** Piège rendu qualitatif. [spec, Dev Notes]
- [x] [Review][Patch] **Compteur de clés i18n : 10 réelles, « 6 » déclaré** en Dev Agent Record et en File List. **LOW.**
- [x] [Review][Patch] **Compteur de tests unitaires `invoices.rs` : 5 réels, « 6 » déclaré** (le pendant avoir, lui, est juste). **LOW.**
- [x] [Review][Patch] **`TABLES_TO_TRUNCATE` : 37 tables, « 23 » déclaré** en AC14-bis, recopié d'un commentaire périmé hors story. La conclusion d'AC14-bis reste vraie. **LOW.**
- [x] [Review][Patch] **Le décompte de modules qui justifie le split (« 5 ») était démenti par deux amendements ultérieurs de la même spec** — `kesh-api/routes/credit_notes` (passe 1) et `crates/kesh-i18n` (passe 2) portent le livré à 7. Sans conséquence rétroactive. **LOW.**
- [x] [Review][Patch] **AC12 était orphelin de traçabilité** — cité par aucune tâche, alors que la passe 7 de `validate` déclarait « aucun orphelin ». Tenu en fait. Rattaché à T11. **LOW.**
- [x] [Review][Patch] **CHANGELOG : la limitation D1-bis n'était signalée nulle part côté utilisateur**, alors qu'elle vise exactement le cas d'usage nominal du CR #265 (toutes les lignes explicitement ventilées) et le seul atteignable aujourd'hui, le champ n'étant écrivable que par API. **LOW.** Puce ajoutée.
- [x] [Review][Patch] **Étiquette de commentaire périmée** : le bloc de `generate_invoice_journal_lines` s'ouvrait encore sur `// pub(crate) :` que le patch de passe 3 avait précisément retiré. **LOW.** [`invoices.rs:1418`]
- [x] [Review][Patch] **Résidus rédactionnels** : ancre interne « ligne 172 » glissée à `:189` ; `list_all_lines_by_component` (fonction inexistante) → `list_all_lines_by_company`. **LOW.**

**Passe 6** — 2026-07-29, 3 lentilles **Sonnet** (rotation : Opus en passe 5), recentrées sur le seul terrain encore actif : vérité des commentaires contre le code, recomptage intégral de la spec, fidélité de la documentation utilisateur. **5 findings : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 2 LOW.** La lentille « documentation utilisateur » rend **0 finding**, trace à l'appui. **Les trois MEDIUM ont une cause commune inédite : ce sont des renvois cassés par les réparations que cette story a elle-même effectuées.**

- [x] [Review][Patch] **Le message d'assertion de `apply_migrations_up_to` portait encore `total - 8` et `total == 39`.** **MEDIUM.** **Troisième site du même symptôme dans le même fichier**, après le commentaire de frontière (passe 2) et le doc-comment de la fonction (passe 5) — un par passe. Mon patch de passe 5 avait grepé le symptôme sur le dépôt mais **pas dans le fichier qu'il patchait**. Le message est lu au moment exact où un dev ajoute une migration, c'est-à-dire au point de décision que le garde-fou **P6** existe pour protéger. Corrigé, et la chronologie des trois correctifs inscrite dans le code pour que le motif soit visible. [`migrations_upgrade_path.rs:37-44`]
- [x] [Review][Patch] **AC5 point 3 pointait sur `credit_notes.rs:266-268` comme « liste en dur »** — site disparu : l'implémentation a remplacé la duplication par un emprunt de `invoices::LINE_COLUMNS` (`:328`), et l'ancienne ancre désigne aujourd'hui un bloc `use`. **MEDIUM** : c'est un **AC formel actif**, et la correction n'existait que dans le Dev Agent Record et dans AC5-ter, lequel est explicitement marqué SUPERSEDED — donc dans les deux seuls endroits qu'un reviewer est fondé à ne pas suivre. Ancre et qualification rectifiées, ici et dans la table d'ancres des Dev Notes. [spec AC5, Dev Notes]
- [x] [Review][Patch] **T1 et la table d'ancres citaient `migrations-idempotence-audit.md:71` / `:68-71`** pour l'invariant « Idempotence `no` : 0 » et les compteurs — ancres exactes **au moment où elles ont été écrites**, décalées de 14 lignes par la **réparation du fichier effectuée en passe 1 de cette même story**. L'invariant est à `:85`, les compteurs à `:82-85`. **MEDIUM.** Corriger un fichier sans greper les renvois qui pointent dessus, y compris depuis le document qui applique le correctif. [spec T1, Dev Notes]
- [x] [Review][Patch] **Ancre décalée d'un cran** : le commentaire de passe 5 citait `invoices.rs:640, if !sites.is_empty()` — `:640` porte `explicit_line_account_sites`, le `if` est à `:641`. Affirmation de fond exacte. **LOW.** [`invoices_line_revenue_account.rs`]
- [x] [Review][Patch] **« 5 sites de colonnes » pour `credit_notes.rs` sans base de comptage explicite**, irréconciliable avec les « 4 sites » d'AC5-bis sans convention écrite. **LOW.** Base explicitée : 4 listes tenues à la main + 1 `SELECT` passé à l'emprunt.

**Passe 7** — 2026-07-29, 2 lentilles **Haiku 4.5** (rotation : Sonnet en passe 6), à mandat **strictement mécanique** — validation exhaustive des ancres `fichier:ligne`, et des renvois croisés `AC`/`D`/`T` + noms de tests et de fonctions cités. Registre choisi délibérément : c'est celui où Haiku est fiable, et il vise exactement le mode d'échec démontré par la passe 6. **2 findings : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 2 LOW.** **Critère d'arrêt de la § « Review Iteration Rule » atteint.**

- [x] [Review][Patch] **Ancre `LINE_COLUMNS` décalée** : `invoices.rs:39` désigne le doc-comment, la constante est à `:43`. **LOW.** Corrigée aux 2 sites actifs (AC5 point 1, table d'ancres) ; l'occurrence d'AC5-ter est laissée telle quelle, l'élément étant barré et SUPERSEDED.
- [x] [Review][Patch] **Une citation d'AC5-ter ne signalait pas son statut SUPERSEDED** — dans le Change Log de la passe 7 de `validate`, section historique. **LOW.** Annotée plutôt que réécrite : c'est une trace d'époque. L'annotation relève que **les deux** vérifications revendiquées par cette ligne ont été réfutées depuis (le décompte « 8 sites » et les compteurs « 55 / 44 »), ce qui en fait l'illustration la plus nette de la leçon du lot — relire une valeur n'est pas la vérifier.
- **2 faux positifs réfutés en ground-truth** : la lentille d'ancres signalait `crates/kesh-api/src/tests/exports_global_e2e.rs` comme chemin inexistant. Il l'est — mais `grep -n "exports_global_e2e"` sur le document montre que **toutes** ses occurrences portent déjà le chemin correct `crates/kesh-api/tests/`. Écartés conformément à la § « Haiku-specific guardrails ».

---

## Dev Notes

### Ancres ground-truth (re-vérifiées en passe 1 de `validate`, 2026-07-26, commit `ef6cdf52`)

| Élément | Emplacement |
|---|---|
| Schéma `invoice_lines` | `crates/kesh-db/migrations/20260416000001_invoices.sql:41` |
| Schéma `credit_note_lines` | `crates/kesh-db/migrations/20260627000001_credit_notes.sql:63` |
| Contrainte `chk_jel_debit_credit_exclusive` | `crates/kesh-db/migrations/20260412000001_journal_entries.sql:46` |
| Convention FK vers `accounts` (11 sites, toutes RESTRICT) | `grep -rn "REFERENCES accounts" crates/kesh-db/migrations/` |
| `LINE_COLUMNS` | `crates/kesh-db/src/repositories/invoices.rs:43` (ancre rectifiée en passe 7 : `:39` désignait le doc-comment, la constante est à `:43`) |
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
| **Avoir : `SELECT invoice_lines` du snapshot** (emprunte `LINE_COLUMNS` depuis l'implémentation — ancre rectifiée en passe 6, l'ancienne `:266-268` pointait sur un bloc `use`) | `credit_notes.rs:328` |
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
| Compteurs audit idempotence (**à RECOMPTER depuis le tableau**, jamais à incrémenter) | `docs/migrations-idempotence-audit.md:82-85` (ancre rectifiée en passe 6) |
| **`journal_entries` — AUCUNE colonne `source`/`origin`** (schéma complet) | `crates/kesh-db/migrations/20260412000001_journal_entries.sql:17-36` |
| **`line_order` réattribué à chaque `update`** (`(idx as i32) + 1`) | `crates/kesh-db/src/repositories/journal_entries.rs:1005` ; création `:272` |
| **`PUT /journal-entries/{id}` — aucune garde de provenance** | `crates/kesh-api/src/lib.rs:303` → `routes/journal_entries.rs:517` → `repositories/journal_entries.rs:805` |
| Avoir créé directement en `issued` (pas d'état `draft` intermédiaire) | `credit_notes.rs:357-362` ; CHECK `20260627000001_credit_notes.sql:50`, `:54` |

### Pièges, par ordre de coût

0. **Croire que cette story protège le parc existant (D2-bis)** — elle ne le fait **pas**. La matérialisation de D2 ne touche que les transitions `draft → validated` : à l'instant du déploiement, elle protège un ensemble **vide**. C'est `16-1a-bis` qui traite le passé. Conséquence pratique ici : **ne pas** écrire de post-condition ni de test global sur `invoice_lines`, et **ne pas** ajouter d'`UPDATE` de backfill à la migration d'AC1 « pour bien faire » — le backfill correct est nettement plus subtil qu'il n'y paraît (l'écriture d'une facture validée est éditable par l'utilisateur), et un backfill approximatif **fabrique** la corruption qu'il prétend fermer. Cf. `16-1a-bis` D-B2 et D-B3.
1. **L'avoir (D5)** — le plus coûteux si oublié : corruption comptable silencieuse, équation du bilan toujours équilibrée, donc **aucun signal**. Le test AC17 « les deux écritures s'annulent compte par compte » est le garde-fou.
2. **Les listes de colonnes SQL écrites en dur** — un oubli ne casse pas la compilation : `sqlx::query_as` échoue au **runtime**, potentiellement seulement sur le chemin d'export ou d'avoir, donc pas forcément dans les tests rapides. C'est le vrai piège ; il est **qualitatif**. ⚠️ **Ne pas s'en servir comme d'un décompte** (rectifié en passe 5) : cette entrée disait « les 8 sites (AC5 / AC5-bis / AC5-ter), dont 4 listes en dur », or **AC5-ter est marqué SUPERSEDED** et dit lui-même de ne pas l'utiliser comme checklist, et le « 4 » est démenti par le livré — `create_credit_note` emprunte désormais `invoices::LINE_COLUMNS` (`credit_notes.rs:328`), il reste **3** listes en dur (`credit_notes.rs:67`, `:95`, `:532`) plus 2 côté `invoice_lines` (`list_all_lines_by_company`, préfixes `il.` obligatoires à cause du JOIN, et l'`INSERT`). Un critère numérique sur cible mouvante est un mauvais garde-fou : c'est la leçon qui a fait marquer AC5-ter SUPERSEDED en passe 2, et elle vaut aussi ici.
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

### Split 16-1a → 16-1a + 16-1a-bis — 2026-07-26 (arbitrage Guy, suite à la passe 6)

**Second critère de la § « Règle de splitting préventif » coché** : la sévérité de 16-1a a cessé de décroître (P5 : 1 HIGH → P6 : 2 HIGH). **Arbitrage du Project Lead : extraction du backfill.**

Motif retenu : sur les passes 5 et 6, **7 findings sur 10 — dont les 3 HIGH — portaient sur le seul backfill D2-bis**, alors que le reste de 16-1a (schéma, moteur de ventilation, avoir, validation à la saisie et au posting) n'était plus remis en cause depuis la passe 3. Le backfill a un **profil de risque étranger** à cette story : migration de **données comptables réelles**, one-shot, essentiellement du SQL, là où 16-1a est du schéma et de la logique applicative testable unitairement. Il saturait les passes adversariales au point de masquer d'éventuels défauts ailleurs.

**Extrait vers `16-1a-bis-backfill-parc-existant.md`** : décision D2-bis complète (critère d'unicité en trois conditions, `<=>` NULL-safe, portée, décompte par requête de diagnostic, verdict d'idempotence), AC2-bis et ses post-conditions, le volet backfill de T1, le piège n°0, et les ancres ground-truth correspondantes. Le contenu part dans son **état convergé des passes 4 à 6**, sans régression — mais n'ayant jamais été revu comme un tout autonome, `16-1a-bis` démarre sa propre boucle `validate` en passe 1.

**Ce qui reste dans 16-1a** : D2-bis devient un **pointeur** documentant pourquoi cette story ne protège **que** les factures futures, avec trois consignes opérationnelles (ne rien backfiller dans la migration d'AC1 ; borner l'invariant d'AC9-bis aux factures validées par ce binaire ; n'écrire **aucune** post-condition globale sur `invoice_lines`). AC2-bis, T1 et le piège n°0 sont réécrits en conséquence.

**Le split est sûr, et c'est démontrable** : sans 16-1a-bis, le parc antérieur conserve **exactement** le comportement d'aujourd'hui. Le backfill est **strictement additif** — il ferme un bug pré-existant, il n'en introduit aucun. C'est la différence avec D5, qui devait impérativement rester avec sa cause, et le split avait d'ailleurs été **décliné** en passe 3 pour cette raison précise.

**Effet sur la boucle de revue** : la passe 7 de 16-1a porte sur un périmètre **allégé et stable**. Si elle converge, 16-1a est scellée indépendamment de la maturité du backfill.

### Passe 7 de `validate` — 2026-07-26 (Sonnet, contexte frais, périmètre allégé)

**0 finding.** Passe entièrement dédiée aux **cicatrices du split** et aux patches non revus.

- **Résidus du split — aucun.** Grep exhaustif de `backfill`, `D2-bis`, `AC2-bis`, `parc`, `total_amount`, `<=>`, `diagnostic` : hors Change Log historique, toutes les occurrences sont dans le pointeur D2-bis, dans AC2-bis (« aucun backfill dans cette story ») ou dans le piège n°0 réécrit. **Aucun mécanisme de backfill n'a fui** — critère d'unicité, `<=>` NULL-safe et requête de diagnostic sont partis en entier vers `16-1a-bis`. T1 et les ancres ground-truth sont cohérents.
- **Le patch HIGH de la passe 6 sur AC8-bis tient, démonstration refaite indépendamment** : `get_or_create_default_in_tx` occupe bien `:87-107` et se réduit à `INSERT IGNORE` + `SELECT … FOR UPDATE`, **sans JOIN ni contrôle d'`active`** ; le JOIN est bien dans `insert_with_defaults_in_tx` (`:403-506`, JOIN réel à `:483` — 1 ligne de dérive sur l'ancre `:482`, négligeable), dont `routes/onboarding.rs:720` est le **seul** site d'appel du dépôt. Le patch est cohérent partout où il devait se propager (AC8-bis, AC18-bis 3 cas, D3-bis, AC12-bis), et le 3ᵉ cas d'AC18-bis est **constructible** : `accounts::archive` (`accounts.rs:472`) ne contrôle que les enfants actifs, jamais le référencement par `company_invoice_settings.default_revenue_account_id`.
- **Testabilité** : AC12 satisfiable (fixture `make_line` sans le champ, signature du helper inchangée à 4 arguments, le scalaire servant de repli) ; AC17, AC18, AC18-bis, AC19, AC20, AC21 reposent tous sur des chemins réels et atteignables.
- **Traçabilité** D1→D7 → AC → T1-T12 complète, aucun orphelin, saut `AC21 → AC23` cohérent avec la suppression d'AC22 en passe 3, plus aucun renvoi fantôme hors Change Log.
- **Aucune contradiction interne** : D2 vs AC9-bis (bornage « par ce chemin » cohérent des deux côtés), D3 vs D3-bis vs AC8-bis (4 critères sur les comptes explicites, 2 sur le défaut implicite), D4 vs D4-bis (niveaux différents et compatibles), D5 vs D5-bis.
- **Large échantillon d'ancres re-vérifié** sans divergence, dont les 8 sites d'AC5-ter, les points d'accroche de saisie (`:459` / `:816`), `validate_lines_accounts_in_tx` (confirmé **sans** `account_type`), et les compteurs de `migrations-idempotence-audit.md` (Total 55, `tracked-by-sqlx` 44).
  > ⚠️ **Annotation de la passe 7 de `bmad-code-review` — les DEUX vérifications de cette ligne ont depuis été réfutées.** Conservée telle quelle : c'est une trace d'époque, pas un énoncé courant. (a) Le décompte « 8 sites » d'AC5-ter est **SUPERSEDED** depuis la passe 2 de revue de code — il avait bougé dans les deux sens, et AC5-ter porte désormais la consigne de ne pas s'en servir comme checklist ; le décompte courant est en AC5 / AC5-bis. (b) Les compteurs « Total 55 / `tracked-by-sqlx` 44 » étaient **faux de 7** — cf. la « Leçon de méthode » de la passe 1 de revue de code, ci-dessous. Une vérification qui relit une valeur au lieu de recompter sa source ne vérifie rien : cette ligne en est l'illustration la plus nette du lot, puisqu'elle revendique explicitement d'avoir vérifié les deux choses qui étaient fausses.

**Trend** : 28 → 4 → 6 → 3 → 2 → 8 → **0**. **Critère d'arrêt de la § « Review Iteration Rule » atteint** : plus aucun finding, a fortiori aucun au-dessus de LOW. **16-1a est convergée et scellée**, en 7 passes (plafond de 8 non atteint), indépendamment de la maturité de `16-1a-bis`. Le split de la passe 6 a produit exactement l'effet recherché.

### Passe 1 de `bmad-code-review` — 2026-07-28 (Sonnet ×3 lentilles + vérification orchestrateur Opus 5)

**2 HIGH, 4 MEDIUM, 3 LOW.** Rotation respectée : le dev ayant été fait par Opus 5, les 3 lentilles tournent sur Sonnet (§ « Review Iteration Rule »). Tous les findings au-dessus de LOW vérifiés ground-truth (`grep -nF` + **bornage de la fonction englobante**) avant rétention — **0 faux positif retenu, 0 hallucination**.

**Le gate a été rejoué hors sandbox, et il était ROUGE** (`nextest` exit 100) alors que T12 le cochait. Le test d'AC14 `export_global_zip_invoice_lines_expose_revenue_account` échouait sur sa propre assertion de garde : il s'appuie sur `seed_with_full_data`, fixture bornée `exports_global_e2e.rs:311-418` qui crée comptes, écritures, contacts, produit et comptes bancaires — **mais aucune facture**. AC14 n'a donc jamais été vérifié. Réparé en montant la facture et ses 2 lignes (une avec compte explicite, une à `NULL`) dans le test lui-même : étendre la fixture aurait cassé les autres tests du fichier, qui assertent des décomptes exacts dessus.

**HIGH — le snapshot d'audit `before` était corrompu.** `validate_invoice` charge `lines_before` (`:1555`), la **mute en place** (`:1692`) pour matérialiser le compte, puis la sert aux **deux** clés `before` et `after` du snapshot (`:1824-1825`). Le commentaire du code trahit l'angle mort : il justifie la mutation par le besoin du snapshot « after » et de la réponse HTTP, sans voir la troisième utilisation. Conséquence : la transition `NULL` → compte effectif — **la seule que cette story introduit** — devenait irrécupérable depuis le journal d'audit. Corrigé par une copie `lines_pre_materialization` figée avant la mutation. Le test `audit_before_snapshot_predates_materialization` a été **vérifié par mutation** : sans le correctif il échoue avec `obtenu Number(4)`, l'id du compte matérialisé.

**MEDIUM — le message `NotPostable` mentait, et la spec en était la source.** Le variant se déclenche sur « non imputable **et** différent du compte par défaut », mais le message affirmait « n'est **plus** le compte de produit par défaut », présupposant un état antérieur faux dans le cas courant. La formulation venait de la spec elle-même (§ « Fenêtre assumée », aujourd'hui `:189` — l'ancre « ligne 172 » de cette phrase avait glissé, rectifiée en passe 5), où elle était **vraie pour le seul cas décrit** — l'admin change le défaut entre saisie et posting. Corrigé dans les 4 locales, le fallback Rust, **la spec** et le CHANGELOG (qui portait la même tournure « n'est plus imputable »).

**MEDIUM — `CreditNoteLineResponse` n'exposait pas `revenue_account_id`** là où `InvoiceLineResponse` le fait. Arbitrage Guy : ajouté dans 16-1a, cohérent avec le périmètre annoncé (« API + moteur comptable facture ET avoir »).

**MEDIUM — AC6 exigeait un test de désérialisation absent.** `grep -rn "revenueAccountId" crates/ --include=*.rs` ne retournait que les 2 macros `json!` de snapshot. Le `#[serde(default)]` était correct mais non couvert — or c'est le seul garde-fou contre son retrait accidentel, qui casserait toute intégration PAT sans cesser de compiler.

**MEDIUM — dérive de 7 sur `migrations-idempotence-audit.md`.** Le compteur annonçait `tracked-by-sqlx : 45` pour **52** réelles. Cause racine : **le tableau est scindé en deux blocs** (lignes 21-65, puis 11 lignes orphelines tombées *sous* la section « Maintenance future », où elles ne se rendent même pas comme un tableau) ; les statistiques n'ont jamais compté le second bloc. Compteur corrigé + avertissement posé. **La scission elle-même n'est pas réparée** (hors périmètre validé) — à traiter avant le prochain ajout de migration.

**Leçon de méthode.** La passe 7 de `validate` déclare avoir « re-vérifié les compteurs de `migrations-idempotence-audit.md` (Total 55, `tracked-by-sqlx` 44) » — **sept passes adversariales ont donc confirmé un nombre faux de 7**. Une vérification qui relit la valeur sans recompter la source ne vérifie rien. Symétrique exact de la leçon de la passe 6 sur le bornage des fonctions. Second enseignement : l'AcceptanceAuditor a classé **AC14 « satisfait »** sur analyse statique, verdict réfuté par l'exécution — « les tests existent » ≠ « les tests passent », et seul le gate réellement exécuté tranche.

**Reportés (3, tous LOW ou sans effet sur le livrable)** : le décompte « 8 + 1 » d'AC5-ter démenti à l'implémentation (défaut de spec, le code couvre bien les 3 sites manquants) ; l'étiquetage `Inactive` d'un compte introuvable côté avoir avec numéro vide (chemin rendu inatteignable par la FK `ON DELETE RESTRICT`) ; la duplication de la collecte des sites entre facture et avoir (sentinelle `0: i32` vs `Option<i32>`). Consignés dans `deferred-work.md`.

**Fausse piste ouverte puis refermée par l'orchestrateur** : le filtre `if *amount > Decimal::ZERO` sur les agrégats par compte déséquilibrerait l'écriture si un agrégat pouvait être négatif — mais `chk_invoice_lines_line_total_non_negative` (migration `20260416000002`) l'interdit au niveau DB. L'équilibre tient par construction, dans les deux fonctions de ventilation.

#### Régression révélée par le gate complet — le couplage « ma migration est la dernière »

Le gate post-patch (2059 tests) a fait tomber **2 tests supplémentaires**, dans des fichiers que la story ne touche pas. Cause unique : **ajouter une migration décale la position de toutes les précédentes**, et trois tests indexaient les migrations **par position**.

| Test | Comportement | Garde-fou présent |
|---|---|---|
| `migrations_upgrade_path::upgrade_path_preserves_data` | ❌ échec en 0,041 s | `assert_eq!(total, 55)` codé en dur, **délibérément** |
| `accounts_role_backfill::backfill_matches_seed_for_every_chart` | ❌ échec | assertion de montage (`role` ne doit pas exister) |
| `accounts_role_backfill::backfill_skips_archived_accounts` | ⚠️ **passait à vide** | **aucun** |

Le troisième est le plus instructif : `total - 1` incluant désormais la migration 14-3a, le backfill tournait **avant** l'insertion des comptes de test. Ses rôles ressortaient `NULL` — assertion verte — non parce que le backfill avait correctement écarté le compte archivé, mais **parce qu'il n'avait jamais tourné sur ces lignes**. Un test devenu muet, qui ne détecterait plus aucune régression réelle du backfill.

**Ce mode d'échec est invisible en revue de diff** : il ne naît ni du code écrit ni de la spec, mais de l'**interaction** entre la migration ajoutée et des tests qu'elle ne touche pas. Ni les 7 passes de `validate`, ni les 3 lentilles de revue ne pouvaient le voir. **Seul le gate réellement exécuté le révèle** — et pour un des trois, seule l'existence d'une assertion de montage l'a rendu visible plutôt que silencieux.

**Correctifs.** Dans `accounts_role_backfill.rs`, suppression du couplage : le helper `migrations_before_role_backfill()` **résout l'index par version** (`20260722000001`), rendant le montage insensible à toute migration future — fix structurel, pas garde ponctuel. Dans `migrations_upgrade_path.rs`, le couplage positionnel est **intentionnel et documenté** (il matérialise une frontière historique, et son commentaire annonçait précisément ce piège) : respecté tel quel, `total` 55 → 56 et fenêtre 21 → 22, ce qui **préserve la frontière à 34** (56−22 = 55−21), plus la ligne 16-1a à l'inventaire commenté.

**À retenir pour toute story ajoutant une migration** : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` fait partie du travail, au même titre que la ligne de `migrations-idempotence-audit.md` exigée par la politique P5. **Codifié le 2026-07-28 en garde-fou P6** de `CLAUDE.md` § « Migration breaking policy », avec ses deux critères alternatifs (résolution par version, ou couplage positionnel assumé **avec** garde-fou fail-loud). P5 a été complétée dans la foulée : les compteurs se **recomptent depuis le tableau**, ils ne s'incrémentent pas de confiance.

#### Réparation de `docs/migrations-idempotence-audit.md`

Le tableau était **scindé en deux blocs** : 11 lignes des Epics 20/21 (`20260705000001` → `20260715000001`) avaient été ajoutées **sous** la section « Maintenance future » au lieu du tableau — invisibles au rendu markdown et absentes de tout comptage. D'où la dérive de 7. Les 11 lignes ont été remontées à leur place chronologique (déplacement scripté, sous assertions : 56 lignes avant/après, tri par version, unicité, aucune résiduelle sous les Statistiques), l'intitulé figé à « 27 migrations » corrigé en « 56 », et une consigne explicite posée dans « Maintenance future » — la ligne s'ajoute **à l'intérieur** du tableau, jamais en fin de fichier. Déclaré ≡ réel désormais vérifié sur les 4 compteurs (56 / 52 / 4 / 0).

#### Gate final — VERT

`cargo fmt --all -- --check` ✅ · `cargo build --workspace --all-targets` ✅ · `cargo clippy --workspace --all-targets -- -D warnings` ✅ 0 warning · `cargo nextest run --no-fail-fast` ✅ **exit 0 — 2059 tests run : 2059 passed, 4 skipped**, sur DB `kesh_gate` migrée.

Rappel : avant cette passe, le même gate sortait **exit 100 avec 2 échecs** (+1 test devenu muet), alors que T12 le cochait comme vert. **AC23 est satisfait à partir d'ici, pas avant.**

**Bilan de la passe 1** : 2 HIGH + 4 MEDIUM patchés, 3 LOW reportés, 3 régressions de couplage aux migrations corrigées, 1 document d'audit réparé, 2 garde-fous codifiés dans `CLAUDE.md`. **Une passe 2 est requise** par la § « Review Iteration Rule » (findings > LOW en passe 1) : contexte frais, LLM différent, et **diff aplati `HEAD vs main`** plutôt que la séquence de commits — le diff a bougé de 9 correctifs dont 3 hors périmètre initial de la story.

### Passe 2 de `bmad-code-review` — 2026-07-28 (Opus ×3 lentilles)

Consignée en détail dans la § « Review Findings » ci-dessus (11 entrées) et dans la File List, sans entrée narrative ici. **25 findings bruts → 21 dédupliqués, 0 CRITICAL, 0 HIGH survivant** — les 2 HIGH remontés ont été **réfutés en ground-truth** (il n'existe aucun export CSV des lignes d'avoir ; le chemin backup/restore mappe par **nom** de colonne, introspecté depuis `information_schema`, donc insensible à l'ajout). 3 convergences inter-lentilles, dont les 2 arbitrages de Guy (issue **#278** pour le `PUT` effaçant ; **D1-bis** pour le défaut société exigé). Gate : `nextest` exit 0, **2061/2061**. Commit `6deca569`.

### Passe 3 de `bmad-code-review` — 2026-07-28 (Sonnet ×3 lentilles, diff aplati)

**6 findings : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 3 LOW.** Trois patchés, un arbitré sans correctif, deux corrections documentaires. Détail par finding en § « Review Findings ».

**Le fil conducteur de cette passe est la remédiation elle-même** : trois des six findings portent sur du texte ou du code **écrit par les passes 1 et 2**, pas par le dev.

- La doc de `generate_credit_note_journal_lines` a été **fausse en passe 1** (« le triplet vient du snapshot `credit_note_lines` »), **imprécise après le correctif de passe 2** (« lignes verrouillées » — aucun `FOR UPDATE` ne porte sur `invoice_lines`), et n'est exacte qu'à partir d'ici. Deux passes pour une phrase de doc-comment : c'est le mode d'échec `feedback_review_patch_needs_test` appliqué à de la documentation, où aucun test ne peut mordre.
- Le test `draft_crud_survives_null_company_default_column`, **créé par la passe 2** pour couvrir AC12-bis, ne couvrait pas la branche qu'il annonçait : son compte explicite était déjà imputable, donc il serait passé même si `.flatten()` avait cessé de distinguer « colonne `NULL` » de « ligne absente ». Un test qui ne peut pas échouer pour la raison qu'il invoque n'est pas une couverture. Réparé par une seconde assertion (compte non imputable → rejet `NotPostable`) — même geste que le test de mutation de la 21-6a. *(Le commentaire décrivant ce patch a été rectifié en passe 4 : voir ci-dessous.)*
- Le décompte « 22 tests » de la File List était faux de 3 (25 réels). **Troisième occurrence du même symptôme sur cette seule story**, après AC5-ter et les compteurs de `migrations-idempotence-audit.md`. Recompté à la source cette fois.

**Le seul finding de fond** — `generate_invoice_journal_lines` exposé en `pub(crate)` — est une **borne de portée**, pas un bug : la phrase « aucun appel de production hors de ce module » était une convention que rien ne faisait respecter. `pub(in crate::repositories)` la transforme en garantie du compilateur. Fix structurel, cohérent avec la leçon de la 21-6b : fermer la classe plutôt que colmater le cas.

**Le MEDIUM non patché** est la **troisième remontée** du défaut société exigé même quand toutes les lignes sont explicites — par une lentille différente à chaque passe. L'arbitrage de Guy (garder + documenter, D1-bis) tient ; la passe 3 y verse l'argument qui manquait : l'incohérence n'est pas « une fonctionnalité absente », c'est que **`create()` et `update()` tolèrent le défaut absent quand `validate_invoice` le refuse** — on enregistre le brouillon puis on refuse de le valider, à configuration inchangée.

**Note de traçabilité — passe interrompue puis reconstituée.** La session qui menait cette passe est morte pendant son gate ; les 4 correctifs étaient sur disque, le rapport de findings ne l'était pas. Il a été **récupéré intégralement** depuis les transcripts des trois sous-agents (`~/.claude/projects/…/subagents/agent-*.jsonl`, conservés hors `/tmp`), et non reconstruit par déduction depuis le diff. Les sévérités, les lentilles d'origine et les points explicitement jugés sains ci-dessus sont donc ceux réellement produits par les reviewers. **Enseignement d'outillage** : un rapport de revue doit être **écrit sur disque dès sa production**, pas gardé en contexte — ici le filet de sécurité a tenu par chance, pas par conception.

**Ce que la passe 3 a explicitement jugé sain** (utile pour ne pas re-litiger en passe 4) : équilibre débit/crédit dans les deux sens de ventilation ; les 4 locales portent exactement les 10 mêmes clés ; `is_no_op_change` inclut bien `revenue_account_id` (sans quoi un changement de compte seul serait silencieusement perdu — KF-004) ; `migrations_before_role_backfill()` résout bien **par version** et l'arithmétique `total - 22 = 34` reste cohérente ; les 4 compteurs de `migrations-idempotence-audit.md` (56 / 52 / 4 / 0) **recomptés indépendamment par deux lentilles** ; le test `..._known_limitation` chiffre un résidu réel et échouerait sur une correction prématurée. L'AcceptanceAuditor a par ailleurs **ré-exécuté** le gate sur le périmètre du diff (25/25 sur le fichier de test central, 302/302 `kesh-api --lib`, 137/137 sur 8 suites e2e) plutôt que de le lire dans la spec.

**Bilan de la passe 3** : 4 correctifs, 1 arbitrage confirmé et argumenté. **Une passe 4 est requise** par la § « Review Iteration Rule » (3 MEDIUM en passe 3). Rotation : passes 1 et 3 en Sonnet, passe 2 en Opus → **la passe 4 tourne en Haiku 4.5**, sur diff aplati mono-commit, avec vérification ground-truth `grep -nF` obligatoire sur tout CRITICAL/HIGH affirmant une absence (§ « Haiku-specific guardrails »).

### Passe 4 de `bmad-code-review` — 2026-07-29 (Haiku 4.5 ×3 lentilles, diff aplati mono-commit)

**1 finding : 0 CRITICAL, 0 HIGH, 1 MEDIUM.** Détail en § « Review Findings ».

**Le finding poursuit la chaîne des trois passes précédentes** : la passe 3 avait corrigé un test créé par la passe 2 ; la passe 4 corrige le **commentaire** de ce correctif de passe 3. Le patch disait de sa nouvelle assertion qu'elle était « discriminante » et qu'elle portait « le comportement qui distingue réellement cette branche ». C'est faux, et à contresens : `.flatten()` existe précisément pour faire **converger** « colonne `NULL` » et « ligne absente » sur un même `None`, de sorte qu'**aucune** assertion placée en aval ne peut les distinguer. Le commentaire inversait donc les rôles des deux assertions. Les deux sont conservées. ⚠️ **La rédaction de remplacement était à son tour fautive** — elle attribuait à la première assertion le monopole du décodage — et a été reprise en passe 5 ; voir ci-dessous.

**Ce que cette passe dit de la rotation des modèles.** Les trois lentilles ont tracé le même chemin de code. L'EdgeCaseHunter a explicitement déroulé la condition `!postable && Some(account_id) != default_revenue_account_id` avec `default = None`, conclu « ✓ » et rendu **0 finding** ; l'AcceptanceAuditor a ratifié l'assertion comme forçant « le chemin colonne NULL » et rendu **0 finding**. Seul le BlindHunter — le seul à ne rien lire d'autre que le diff, donc le seul à ne pas pouvoir se raccrocher à l'intention affichée — a vu que la conclusion ne découlait pas de la prémisse. **Deux « 0 finding » sur trois auraient déclaré la convergence à tort.** C'est la quatrième confirmation sur ce lot que le « 0 finding » de Haiku n'est pas un résultat mais une absence de résultat.

**Le remède de la lentille était faux, son diagnostic juste.** Le BlindHunter proposait de tester « un compte qui *est* le défaut société, rendu non imputable » — impossible ici, le défaut vaut `NULL` par construction dans ce test. Un finding correct peut arriver avec une correction incorrecte : appliquer le remède sans vérifier le diagnostic aurait produit un test incohérent. Retenu : le diagnostic. Écarté : le remède.

**Imprécision de rapport relevée, sans conséquence** : l'AcceptanceAuditor décrit les compteurs de `docs/migrations-idempotence-audit.md` comme passant de « 44→45 » — c'est la valeur **erronée** que la passe 1 avait précisément corrigée (dérive de 7 : 45 déclaré pour 52 réels). Les compteurs du fichier ont été **recomptés à la source** par l'orchestrateur : 56 lignes de tableau = 52 `tracked-by-sqlx` + 4 `yes` + 0 `no`, identiques aux « Statistiques » déclarées. Défaut du rapport, pas du dépôt.

**Bilan de la passe 4** : 1 correctif documentaire propagé aux 5 sites du symptôme (2 dans le test, 3 dans la spec), conformément à la § « Propagation post-patch » de `CLAUDE.md`. **Une passe 5 est requise** par la § « Review Iteration Rule » — le finding est MEDIUM. La sévérité décroît de façon monotone depuis la passe 1 (2 HIGH + 4 MEDIUM → 0 HIGH → 3 MEDIUM → 1 MEDIUM), ce qui est le profil d'une revue qui travaille et non d'une story trop large : la § « Règle de splitting préventif » n'est pas déclenchée. Rotation : **la passe 5 tourne en Opus** (passe 4 en Haiku), sur le seul périmètre restant utile.

### Passe 5 de `bmad-code-review` — 2026-07-29 (Opus ×3 lentilles recentrées)

**14 findings : 0 CRITICAL, 0 HIGH, 6 MEDIUM, 8 LOW. Aucun ne porte sur le code livré.** Treize sont des défauts de documentation, un est une omission du CHANGELOG. Détail en § « Review Findings ».

**Les lentilles ont été recentrées plutôt que répétées.** Après quatre passes, reposer « le code satisfait-il la spec ? » à l'identique ne rapportait plus rien — les passes 3 et 4 y avaient répondu AC par AC, tests exécutés à l'appui. Les trois lentilles de la passe 5 ont donc porté sur (a) les **correctifs des passes 1 à 4 eux-mêmes**, (b) les **frontières du code** que les passes précédentes avaient le moins explorées, (c) la **cohérence interne du document** après onze passes d'amendement cumulées. **Le rendement valide le recentrage** : la lentille (b), la seule à chercher des défauts de code, rend **0 finding** — avec une trace vérifiable, ce qui en fait un résultat et non un silence. Les 14 findings viennent des lentilles (a) et (c).

**Le mode d'échec dominant est nommé, et il n'a pas changé : la remédiation appliquée à son site signalé et pas à son symptôme.** Trois des six MEDIUM sont **un résidu par patch de revue antérieur** — le compteur d'idempotence corrigé en passe 1 mais laissé faux dans AC4 et la File List ; le périmètre README amendé en passe 2 mais laissé contredit par AC24 ; l'avertissement « AC5-ter est SUPERSEDED » propagé en passe 3 à T2 mais pas au piège n° 2, qui continuait d'offrir un décompte comme garde-fou de complétude. C'est littéralement la § « Propagation post-patch » de `CLAUDE.md`, appliquée cette fois à la **spec** plutôt qu'au code. Le geste manquant est toujours le même : greper le **symptôme**, pas le site.

**Le second axe est le défaut chronique de ce document : cinq compteurs faux subsistaient**, tous démentis par un recomptage à la source de moins d'une minute — clés i18n (10, pas 6), tests unitaires facture (5, pas 6), `TABLES_TO_TRUNCATE` (37, pas 23), modules du split (7 livrés, pas 5), et le `44→45` d'AC4 déjà cité. Aucun n'avait de conséquence sur le livrable, **et c'est précisément ce qui les rend durables** : rien ne casse, donc personne ne recompte. À noter que `docs/migrations-idempotence-audit.md`, réparé en passe 1, est aujourd'hui le seul artefact du lot dont tous les compteurs sont vrais — parce qu'il porte désormais la consigne de les recompter.

**Le finding le plus instructif est celui qui me vise.** La passe 4 avait corrigé le commentaire de passe 3 (« assertion discriminante ») en écrivant que l'assertion (1) était **la seule** à exercer le décodage du `NULL`. Faux pour la même raison exactement : la lecture du défaut est déclenchée par la seule présence d'un compte explicite sur une ligne, et l'assertion (2) en porte un. **Deux rédactions successives ont donc affirmé, sur le même bloc de dix lignes, une propriété que personne n'avait vérifiée contre le code** — la seconde en corrigeant la première. La leçon n'est pas « mieux relire » : c'est qu'un commentaire qui attribue un **rôle exclusif** à un fragment de test est une affirmation testable, et qu'il faut la traiter comme telle ou ne pas l'écrire. La rédaction retenue ne revendique plus aucune exclusivité et décrit le **montage** du test, qui est ce qui lui est réellement propre.

**Une réserve levée sans être promue en finding.** La lentille (a) a relevé que la docstring de `generate_credit_note_journal_lines`, réécrite en passe 3, énumère « le seul écrivain de `invoice_lines.revenue_account_id` après validation » sans compter `backup::restore_tables_in_tx`, qui réécrit la table — colonnes introspectées, `revenue_account_id` comprise — sans prendre le verrou sur `invoices`. Elle ne l'a pas retenue comme finding : la restauration remplace `invoice_lines` **et** `credit_note_lines` depuis un même instantané, donc le miroir facture/avoir reste cohérent. L'énumération était « incomplète à la lettre, pas fausse en effet » — la précision a néanmoins été ajoutée, parce que c'est exactement le genre d'énumération approximative qui a déjà coûté trois passes sur cette même docstring.

**Bilan de la passe 5** : 14 correctifs, dont 3 dans le code (tous des commentaires ou doc-comments), 10 dans la spec, 1 dans le CHANGELOG. **La § « Review Iteration Rule » impose une passe 6** — 6 MEDIUM, même si aucun ne touche le livrable. Rotation : **passe 6 en Sonnet** (passe 5 en Opus), sur le périmètre où tout se joue désormais, la documentation. La § « Règle de splitting préventif » reste non déclenchée : le maximum de sévérité décroît sans discontinuer depuis la passe 1 (2 HIGH → 0 HIGH → MEDIUM → MEDIUM → MEDIUM) et **aucun finding ne remet en cause l'implémentation depuis la passe 2**.

### Passe 6 de `bmad-code-review` — 2026-07-29 (Sonnet ×3 lentilles)

**5 findings : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 2 LOW.** La lentille « documentation utilisateur » rend **0 finding**, avec une trace vérifiable point par point — elle a notamment confirmé la puce D1-bis ajoutée au CHANGELOG en passe 5, et établi deux faits que la story n'avait pas écrits : le **frontend ne connaît pas encore le champ** (`invoices.types.ts`), donc toute sauvegarde depuis l'interface omet la clé et remet les lignes sur le défaut — ce qui rend l'issue **#278** nettement plus concrète ; et l'avoir n'est bloqué que sur compte **archivé**, jamais sur un compte devenu non imputable ou retypé, ce que le CHANGELOG ne sur-promet pas.

**Les trois MEDIUM ont une cause commune que les passes précédentes n'avaient pas isolée : ce sont des renvois cassés par les réparations que cette story a elle-même effectuées.** Le motif est nouveau et il est ironique — chacune des trois réparations était **bonne** ; c'est le fait de ne pas avoir grepé ce qui *pointait dessus* qui a créé le défaut suivant.

| Réparation faite par la story | Renvoi qu'elle a cassé, découvert en passe 6 |
|---|---|
| Passe 1 — `migrations-idempotence-audit.md` réparé, 11 lignes remontées dans le tableau | **T1** et la table d'ancres citaient `:71` et `:68-71` ; la section « Statistiques » a glissé de 14 lignes, l'invariant est à `:85` |
| Implémentation — `create_credit_note` cesse de dupliquer une liste de colonnes et emprunte `LINE_COLUMNS` | **AC5 point 3** désignait toujours `credit_notes.rs:266-268` comme « liste en dur » ; l'ancre pointe désormais sur un bloc `use` |
| Passe 5 — doc-comment de `upgrade_path_preserves_data` corrigé | le **message d'assertion** de `apply_migrations_up_to`, 60 lignes plus bas dans le même fichier, portait encore `total - 8` / `total == 39` |

**Le troisième est le plus sévère pour la méthode, parce qu'il est le mien.** J'ai grepé le symptôme sur le dépôt en passe 5 — ce qui a bien écarté les story files historiques — mais **pas à l'intérieur du fichier que j'étais en train de patcher**. `migrations_upgrade_path.rs` portait donc trois copies du même symptôme, découvertes une par passe : commentaire de frontière (passe 2), doc-comment (passe 5), message d'assertion (passe 6). Et ce message est lu au moment exact où un développeur ajoute une migration — le point de décision que le garde-fou **P6** existe pour protéger. La chronologie des trois correctifs est désormais inscrite dans le fichier, pour que le prochain lecteur voie le motif au lieu de le redécouvrir.

**Enseignement, à verser à la rétrospective Epic 16** : la § « Propagation post-patch » de `CLAUDE.md` demande de greper le **symptôme** plutôt que le **site**. Cette passe montre qu'il manque une seconde moitié à la règle — greper aussi ce qui **pointe vers** ce qu'on vient de corriger. Un compteur réparé, une fonction déplacée, une duplication supprimée : chacun laisse derrière lui des ancres `fichier:ligne` et des renvois devenus faux, et **le document qui applique le correctif est le premier à les porter**. Les trois MEDIUM de cette passe sont exactement cela, et aucun n'aurait été trouvé par le grep du symptôme seul.

**Bilan de la passe 6** : 5 correctifs — 2 dans le code (message d'assertion, ancre), 3 dans la spec. **Une passe 7 est requise** par la § « Review Iteration Rule » (3 MEDIUM). Rotation : **passe 7 en Haiku 4.5** (garde-fous § « Haiku-specific guardrails » : diff aplati mono-commit, `grep -nF` obligatoire sur tout CRITICAL/HIGH affirmant une absence). Trend : 14 findings en passe 5 → 5 en passe 6, sévérité maximale stable à MEDIUM depuis la passe 3, **aucun finding sur le code livré depuis la passe 2**.

### Passe 7 de `bmad-code-review` — 2026-07-29 (Haiku 4.5 ×2 lentilles mécaniques) — **CONVERGÉE**

**2 findings : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 2 LOW.** Le critère d'arrêt de la § « Review Iteration Rule » est atteint : plus rien au-dessus de LOW. Plafond de 8 passes non atteint.

**Le mandat a été rendu mécanique à dessein.** Les passes 5 et 6 ayant montré que les défauts restants étaient des **renvois** (ancres décalées, citations d'éléments SUPERSEDED, compteurs relus), la passe 7 ne demande plus de jugement mais de la vérification exhaustive : les ~150 ancres `fichier:ligne` du document, une par une ; tous les identifiants `AC`/`D`/`T` définis et cités ; tous les noms de tests et de fonctions cités comme preuve. C'est le registre où Haiku 4.5 est fiable — son mode d'échec documenté sur ce projet est le jugement sur diff multi-commit, pas le `grep`.

**Résultat de la validation d'ancres** : **150 testées, 0 cassée.** Toutes les ancres des sections formelles — critères d'acceptation et décisions — sont exactes. Un seul décalage réel (`LINE_COLUMNS` annoncé à `:39`, constante à `:43` : l'ancre tombait sur son doc-comment). Les 13 autres écarts signalés sont des **chemins abrégés** (`vat_report.rs:220` pour `crates/kesh-report/src/vat_report.rs:220`), convention constante du document et non un défaut.

**Résultat de la validation des renvois croisés** : 12 décisions, 30 critères d'acceptation, 12 tâches — **tous définis, tous cités, aucun orphelin**. AC12, orphelin jusqu'à la passe 5, est bien rattaché à T11. Les 25 tests d'intégration et les 13 fonctions cités comme preuve **existent tous** avec l'orthographe annoncée. Une seule citation d'AC5-ter ne signalait pas son statut SUPERSEDED, dans une section historique.

**2 faux positifs Haiku, réfutés par `grep`** — la lentille d'ancres annonçait le chemin `crates/kesh-api/src/tests/exports_global_e2e.rs` comme inexistant. Il l'est bien, mais aucune occurrence du document ne le porte : toutes écrivent déjà `crates/kesh-api/tests/`. Écartés conformément à la § « Haiku-specific guardrails ». Sur ce lot, la discipline de vérification ground-truth aura écarté **6 affirmations fausses de reviewers** (2 HIGH en passe 2, 2 ici, plus le remède incohérent de la passe 4 et le classement « hors mandat » d'un vrai résidu en passe 6).

#### Bilan de la boucle de revue — 7 passes

| Passe | Modèle | Findings | Max sévérité |
|---|---|---|---|
| 1 | Sonnet ×3 | 9 (+3 régressions de couplage révélées par le gate) | **HIGH** ×2 |
| 2 | Opus ×3 | 21 (25 bruts) | MEDIUM |
| 3 | Sonnet ×3 | 6 | MEDIUM |
| 4 | Haiku ×3 | 1 | MEDIUM |
| 5 | Opus ×3 | 14 | MEDIUM |
| 6 | Sonnet ×3 | 5 | MEDIUM |
| 7 | Haiku ×2 | **2** | **LOW** |

**Aucun finding ne porte sur le code livré depuis la passe 2.** Les passes 3 à 7 n'ont trouvé que de la documentation — et, de façon écrasante, **de la documentation écrite par les passes de revue elles-mêmes**. Le compte final est éloquent : sur les 28 findings des passes 3 à 7, **au moins 12 corrigent un artefact produit par une passe antérieure**, dont trois séries où un même bloc a été repris trois ou quatre fois d'affilée.

**Les trois enseignements du lot, à verser à la rétrospective Epic 16** :

1. **La remédiation est la première source de défauts, et le rester.** Ce n'est plus une hypothèse : c'est mesuré sur sept passes. La contre-mesure connue — « un patch vient AVEC son test » — ne couvre pas la documentation, où aucun test ne mord. Ce qui protège, c'est de traiter toute affirmation d'un commentaire comme **testable** : « X est le seul à… », « toujours renseigné », « ce test échouerait si… » sont des assertions à vérifier ou à ne pas écrire.
2. **La § « Propagation post-patch » a une seconde moitié manquante.** Elle demande de greper le **symptôme**. Il faut aussi greper **ce qui pointe vers ce qu'on vient de corriger** — un compteur réparé, une fonction déplacée, une duplication supprimée laissent derrière eux des ancres et des renvois faux, et le document qui applique le correctif est le premier à les porter. Les trois MEDIUM de la passe 6 sont exactement cela, et le grep du symptôme seul n'en aurait trouvé aucun.
3. **Un « 0 finding » n'a de valeur que s'il est tracé.** Quatre rapports vides sur ce lot n'ont rien prouvé ; à l'inverse, les trois « 0 finding » des passes 5 à 7 énumèrent ce qui a été contrôlé et comment, ce qui en fait des résultats opposables. La demande explicite d'une section « vérifié et jugé sain » dans le mandat suffit à produire cette différence.

#### Gate final de la boucle de revue — VERT, sur l'état réellement livré

**Deux gates complets, pas un.** La distinction n'est pas cosmétique : c'est exactement le piège relevé en passe 1, où T12 cochait un gate qui était rouge.

| Run | État validé | Verdict |
|---|---|---|
| 1 | patches de la passe 3 | `exit 0` — **2061/2061**, 4 skipped, 51 min |
| 2 | + tous les correctifs des passes 4 à 7 | `exit 0` — **2061/2061**, 4 skipped, 51 min |

`cargo fmt --all -- --check` ✅ · `cargo clippy --workspace --all-targets -- -D warnings` ✅ 0 warning · `cargo nextest run` ✅ sur DB `kesh_gate` migrée, dans les deux runs.

Le second run était nécessaire et **n'a pas été présumé** à partir du premier : les correctifs des passes 4 à 7 sont certes des commentaires et doc-comments, mais l'un d'eux modifie une **chaîne de message d'assertion** (`apply_migrations_up_to`), donc du code. Reporter le vert du premier run sur un état qu'il n'a pas vu aurait reproduit, à l'échelle de la boucle, le défaut que la passe 1 a documenté.

**AC23 est satisfait sur l'état livré.**

---

## Dev Agent Record

### Agent Model Used

Opus 5 (1M context) — `bmad-dev-story`, 2026-07-27, passe unique.

### Debug Log References

Trois défauts interceptés pendant l'implémentation, tous avant le gate final :

1. **`Unknown column 'account_number'` (runtime, 4 tests d'avoir rouges).** Les
   requêtes de contrôle des comptes nommaient `account_number` ; la colonne du
   plan comptable s'appelle `number` (`20260411000001_accounts.sql:9`).
   `sqlx::query_as` n'étant pas vérifié à la compilation, le défaut ne se
   manifeste qu'à l'exécution. Corrigé sur les 3 sites, puis grepé sur tout le
   dépôt (`SELECT id, account_number` → 0 occurrence résiduelle).
2. **Correctif incomplet du même symptôme.** Le premier passage n'avait traité
   que 2 des 3 sites — le `SELECT … active = TRUE` de la garde D5-bis était
   resté. Exactement le mode d'échec de la § « Propagation post-patch » de
   `CLAUDE.md` : c'est le grep du **symptôme** sur le dépôt, et non la
   correction du site signalé, qui l'a fermé.
3. **Sandbox réseau.** `PoolTimedOut` sur `127.0.0.1:3306` — les tests
   d'intégration exigent l'exécution hors sandbox (MariaDB n'est pas dans
   l'allowlist réseau).

### Completion Notes List

**Trois sites que le décompte d'AC5-ter (« 8 + 1 ») ne comptait pas** et que le
compilateur n'attrape pas non plus. À reprendre au décompte de `bmad-code-review` :

- **`INSERT INTO invoice_lines` de `insert_lines`** (`invoices.rs`) — symétrique
  exact de l'`INSERT credit_note_lines` (`credit_notes.rs:376`) qui, lui, **est**
  compté. Sans lui, un compte posé par API n'aurait jamais été persisté à la
  création : le `SELECT` relit la ligne juste après, donc l'API aurait répondu
  `null` sans erreur.
- **`is_no_op_change`** (`invoices.rs:912`) — le court-circuit KF-004 compare les
  lignes champ par champ. Sans y ajouter `revenue_account_id`, changer
  **uniquement** le compte d'une ligne était vu comme un no-op : modification
  silencieusement perdue, avec un `200 OK` et la version inchangée. Couvert par
  le test `changing_only_the_account_is_not_a_no_op`.
- **`InvoiceLineResponse`** (`routes/invoices.rs`) — la réponse de lecture, que
  T3 mentionne mais qu'aucun AC ne rattache aux « sites ».

**Écart assumé vs. la spec — `LINE_COLUMNS` passée `pub(crate)`.** AC5-bis
laissait le choix « introduire une constante côté avoir, ou traiter les 3
`SELECT` un par un ». Aucune des deux : la liste de colonnes `invoice_lines`
dupliquée en dur dans `credit_notes.rs:266` a été remplacée par un emprunt de
`invoices::LINE_COLUMNS`, sur le modèle de `FIND_INVOICE_SCOPED_SQL` déjà
`pub(crate)` et déjà partagé entre ces deux modules. Motif : c'est la
duplication elle-même qui produit le piège n°2 de la story (échec runtime, pas
compilation) ; la supprimer vaut mieux que la documenter. Il reste **deux**
listes en dur pour `invoice_lines` (`list_all_lines_by_company` — préfixes
`il.` obligatoires à cause du JOIN — et l'`INSERT`), toutes deux inévitables.

**Erreurs typées plutôt que `AppError::Validation(format!(…))`.** D7 prescrivait
la convention `format!("Ligne {n} : …")` de `routes/invoices.rs`. Elle est
**inapplicable telle quelle** : la validation exige un aller-retour DB, elle vit
donc dans le repository, qui retourne `DbError` — et le mapping HTTP de
`DbError` **remplace tout message du repository par un texte fixe traduit**
(`errors.rs`). Un `format!` côté repo n'aurait jamais atteint le client. C'est
la leçon déjà tirée sur `AccountParentArchived` (14-3a, dont la docstring dit
mot pour mot « le message explicatif écrit côté repository n'atteignait jamais
le client »). Deux variantes structurées ont donc été ajoutées —
`InvalidRevenueAccounts` et `CreditNoteRevenueAccountsArchived`, portant
`Vec<RejectedRevenueAccount>` — et `kesh-api` compose le message localisé. **Le
résultat visible respecte D7** (« Ligne 3 : le compte 3200 est archivé »), et le
`details.rejected[]` du body reste exploitable par 16-1b. **10** clés i18n
ajoutées aux **4** locales (recompté en passe 5 : 8 pour les erreurs typées de ce
paragraphe, 2 pour le rejet à montant nul de T9 ; le « 6 » écrit ici et en File
List était faux des deux côtés).

**AC8-bis, ensemble re-validé.** `{ comptes de ligne explicites } ∪ { défaut
société si ≥ 1 ligne NULL }`. Sur le défaut : `account_type` **et** `active`
(rétabli en passe 6), `postable` exempté. Le site est identifié par
`line_number: None`, ce que l'API rend par « le compte de produit par défaut de
la société » — jamais par un numéro, aucune ligne ne le portant.

**AC9-bis, matérialisation.** `UPDATE … WHERE revenue_account_id IS NULL` dans la
transaction, avant `create_in_tx`, **suivi de la mutation de `lines_before` en
mémoire** — sans re-fetch, pour préserver la garantie anti-race documentée sur
`ValidatedInvoice`. Le test `credit_note_uses_materialized_account_not_current_default`
assert les deux : la base **et** `ValidatedInvoice.lines`.

**AC12 tenu.** Les 8 tests unitaires existants de `generate_invoice_journal_lines`
passent **sans qu'une seule assertion soit retouchée** — seule la fixture
`make_line` gagne le champ (via un `make_line_on` qui la généralise). Idem pour
les 4 tests du helper d'avoir, où seule l'arité des tuples change.

**AC2-bis / D2-bis respectés.** La migration se limite à `ADD COLUMN` + index +
FK. Aucun backfill, aucune post-condition globale sur `invoice_lines` : les
assertions portent toutes sur *la facture que le test vient de valider*.

**Non-régression à surveiller en revue (comportement conforme à la spec, mais
divergent d'un précédent).** À la **modification** d'un brouillon, le contrôle
des comptes de ligne re-tourne à chaque enregistrement non-no-op — il n'y a
**pas** de grandfathering « la valeur n'a pas changé » comme en 19-2 pour le tag
projet. Éditer la date d'un brouillon dont une ligne pointe un compte
entre-temps archivé est donc rejeté. C'est ce qu'AC7 prescrit (« création **et**
modification », sans clause de grandfathering) et c'est défendable — le
brouillon ne serait de toute façon pas validable — mais c'est un choix, pas une
évidence.

### File List

**Nouveaux fichiers**

- `crates/kesh-db/migrations/20260727000001_invoice_lines_revenue_account.sql`
- `crates/kesh-db/tests/invoices_line_revenue_account.rs` (**25** tests d'intégration — 23 au dev, +2 en passe 2 de revue ; le « 22 » d'origine était déjà faux d'une unité, même symptôme que les compteurs d'AC5-ter et de l'audit d'idempotence : un nombre écrit une fois n'est jamais recompté)

**Schéma, entités, erreurs**

- `crates/kesh-db/src/entities/invoice.rs` — `InvoiceLine.revenue_account_id`, `NewInvoiceLine.revenue_account_id`
- `crates/kesh-db/src/entities/credit_note.rs` — `CreditNoteLine.revenue_account_id`
- `crates/kesh-db/src/errors.rs` — `RevenueAccountRejection`, `RejectedRevenueAccount`, 2 variantes `DbError` + `error_code()`

**Repositories**

- `crates/kesh-db/src/repositories/invoices.rs` — `LINE_COLUMNS` (+`pub(crate)`), `invoice_snapshot_json`, `insert_lines`, `list_all_lines_by_company`, `is_no_op_change`, `read_default_revenue_account_id`, `validate_line_revenue_accounts_in_tx`, `explicit_line_account_sites`, branchements `create`/`update`, `generate_invoice_journal_lines` (ventilation + docstring), `validate_invoice` (rejet à zéro, re-validation, matérialisation), **5** tests unitaires ajoutés (recompté en passe 5 ; le pendant avoir, lui, en compte bien 6)
- `crates/kesh-db/src/repositories/credit_notes.rs` — **4** listes de colonnes `credit_note_lines` tenues à la main (snapshot `:56`, deux `SELECT` `:67` et `:95`, `INSERT` `:540`) **+ 1** `SELECT invoice_lines` (`:328`) passé à l'emprunt de `invoices::LINE_COLUMNS`, donc plus à maintenir — d'où le « 5 » de ce décompte, base explicitée en passe 6 ; `generate_credit_note_journal_lines` (triplets + ventilation), garde D5-bis, rejet à zéro, 6 tests unitaires

**API**

- `crates/kesh-api/src/errors.rs` — mapping des 2 variantes, `format_rejected_revenue_accounts`, `revenue_account_rejection_code`, 2 codes `InvalidInput`, 4 tests
- `crates/kesh-api/src/routes/invoices.rs` — DTO `#[serde(default)]`, `InvoiceLineResponse`, `validate_line`
- `crates/kesh-api/src/exports/csv_tables.rs` — colonne CSV
- `crates/kesh-api/src/routes/invoice_email.rs` — fixture

**i18n** — `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` (**10** clés × 4 locales, recomptées en passe 5)

**Documentation** — `docs/migrations-idempotence-audit.md` (ligne ajoutée ; compteurs **recomptés** et non incrémentés — 56 = 52 `tracked-by-sqlx` + 4 `yes` + 0 `no` ; l'ancien « 44→45 » relayait la dérive de 7 corrigée en passe 1), `CHANGELOG.md`

**Tests adaptés (fixtures uniquement)** — `crates/kesh-db/tests/{credit_notes_repository,invoice_ttc_parity,invoices_validate_vat,kf005_fulltext_index_e2e}.rs`, `crates/kesh-api/tests/{contact_payment_terms,invoice_delete,invoice_echeancier,invoice_pdf,invoice_send_email,reports,vat_report}_e2e.rs`, `crates/kesh-report/tests/aged_receivables.rs`

**Tests étendus** — `crates/kesh-api/tests/exports_global_e2e.rs` (AC14), `crates/kesh-report/tests/vat_report_reconciliation.rs` (AC13)

### File List — complément des passes de revue

*(Ajouté le 2026-07-28 : la liste ci-dessus datait de `bmad-dev-story` et ne couvrait donc pas les fichiers touchés ensuite par les passes 1 et 2 de `bmad-code-review`. Écart relevé en audit de documentation.)*

**Passe 1 — correctifs**

- `crates/kesh-api/src/routes/credit_notes.rs` — champ `revenue_account_id` ajouté à `CreditNoteLineResponse` + son `impl From` (arbitrage Guy)
- `crates/kesh-db/tests/accounts_role_backfill.rs` — helper `migrations_before_role_backfill()` (résolution **par version**), 2 sites d'appel, doc de module ; ferme la régression qui rendait `backfill_skips_archived_accounts` muet
- `crates/kesh-db/tests/migrations_upgrade_path.rs` — `total` 55 → 56, fenêtre 21 → 22 (frontière 34 préservée), inventaire commenté

**Passe 1 — codification de politique**

- `CLAUDE.md` — garde-fou **P6** (couplage positionnel des migrations) + complément à **P5** (recompter les compteurs depuis le tableau). Commit isolé `0da698db`.

**Passe 2 — correctifs**

- `crates/kesh-db/src/repositories/invoices.rs` — commentaire « grandfathering » de `update()` corrigé
- `crates/kesh-db/src/repositories/credit_notes.rs` — doc de `generate_credit_note_journal_lines` (source réelle du triplet)
- `crates/kesh-api/src/errors.rs` — assertion `"Ligne 2"`, champ `reason` sur le body d'erreur avoir
- `crates/kesh-db/tests/invoices_line_revenue_account.rs` — 2 tests ajoutés (AC12-bis colonne `NULL`, frontière du parc antérieur)
- `crates/kesh-db/tests/migrations_upgrade_path.rs` — commentaire garde-fou rendu factuel + dérive documentaire consignée

**Passe 3 — correctifs**

- `crates/kesh-db/src/repositories/credit_notes.rs` — doc de `generate_credit_note_journal_lines` : nature **exacte** de la protection (verrou sur `invoices`, pas sur `invoice_lines`) + avertissement pour toute seconde voie de mutation
- `crates/kesh-db/src/repositories/invoices.rs` — `generate_invoice_journal_lines` : `pub(crate)` → **`pub(in crate::repositories)`**, la convention devient une garantie du compilateur
- `crates/kesh-db/tests/invoices_line_revenue_account.rs` — `draft_crud_survives_null_company_default_column` : 2ᵉ assertion ajoutée (compte non imputable → `NotPostable`), l'exemption D3-bis désactivée est enfin assertée
- *(spec)* argument D1-bis versé en passe 3 ; T2 renvoie vers AC5-ter marqué SUPERSEDED ; décompte de tests **22 → 25**, recompté à la source

**Passe 4 — correctifs**

- `crates/kesh-db/tests/invoices_line_revenue_account.rs` — doc-comment et commentaire inline de `draft_crud_survives_null_company_default_column` : la mention « discriminante » retirée *(rédaction reprise en passe 5 — cf. son bloc)*
- *(spec)* les 3 renvois à une « assertion discriminante » rectifiés — § « Review Findings » passe 3, Change Log passe 3, File List passe 3

**Passe 5 — correctifs**

- `crates/kesh-db/tests/invoices_line_revenue_account.rs` — doc-comment de `draft_crud_survives_null_company_default_column` réécrit **sans claim d'exclusivité** : les deux assertions décodent le `NULL`, ce qui est propre au test est son montage
- `crates/kesh-db/tests/migrations_upgrade_path.rs` — doc-comment de `upgrade_path_preserves_data` : « 23 appliquées / 5 dernières » → `total - 22` (**34**) / **22**, avec renvoi au commentaire de frontière qu'il contredisait
- `crates/kesh-api/src/routes/credit_notes.rs` — doc de `CreditNoteLineResponse.revenue_account_id` : `null` signifie « défaut société au moment de l'avoir », pas « aucune imputation » ; renvoi au test de frontière et au CHANGELOG
- `crates/kesh-db/src/repositories/credit_notes.rs` — docstring de `generate_credit_note_journal_lines` : réserve sur `backup::restore_tables_in_tx`, seul autre écrivain de la colonne, et pourquoi elle ne rompt pas l'invariant
- `crates/kesh-db/src/repositories/invoices.rs` — étiquette `// pub(crate) :` périmée → `// Visibilité :`
- `CHANGELOG.md` — puce ajoutée sur la limitation **D1-bis** (défaut société obligatoire même quand toutes les lignes sont explicites), absente jusque-là de toute doc utilisateur
- *(spec)* 10 correctifs : AC4 et File List (compteur d'idempotence), AC24 (README), piège n° 2 (AC5-ter SUPERSEDED + décompte), AC14-bis (`TABLES_TO_TRUNCATE` 37), décompte de modules du split (7), clés i18n (10), tests unitaires facture (5), AC12 rattaché à T11, ancre `:189`, `list_all_lines_by_company`

**Passe 6 — correctifs**

- `crates/kesh-db/tests/migrations_upgrade_path.rs` — message d'assertion de `apply_migrations_up_to` : `total - 8` / `total == 39` → `total - 22` / `total == 56`, avec la décision explicite à prendre (élargir la fenêtre ou tenir la frontière) et le renvoi au garde-fou P6 ; chronologie des trois correctifs du fichier inscrite au commentaire de frontière
- `crates/kesh-db/tests/invoices_line_revenue_account.rs` — ancre `invoices.rs:640` → `:641`
- *(spec)* AC5 point 3 (ancre `credit_notes.rs:328` + qualification « emprunt » et non « liste en dur »), T1 et table d'ancres (`migrations-idempotence-audit.md:85` / `:82-85`), base de comptage des sites de `credit_notes.rs` explicitée

**Passe 7 — correctifs**

- *(spec)* ancre `LINE_COLUMNS` `:39` → `:43` aux 2 sites actifs ; annotation du bulletin de la passe 7 de `validate`, dont les deux vérifications revendiquées ont été réfutées depuis

**Hors story, sur la branche** — `README.md` (feuille de route Epic 16, commit `6f7ccec2` antérieur au dev).
