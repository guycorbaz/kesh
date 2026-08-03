# Story 16.2 : Compte de produit par défaut sur la fiche produit

## Status

ready-for-dev

## Story

**As a** fiduciaire ou indépendant qui facture des natures de prestations distinctes,
**I want** que chaque article du catalogue porte **son** compte de produit, et que ce compte pré-remplisse la ligne de facture quand je choisis l'article,
**so that** l'imputation comptable soit décidée **une fois** sur la fiche produit plutôt que ressaisie à chaque facture — où elle est répétitive et donc oubliée.

Issue : **#144**. Sous-story de l'Epic 16 « Facturation avancée », cible **v0.9.0**. **Dépend strictement de 16-1**, qui a livré le champ `invoice_lines.revenue_account_id`, le sélecteur de compte par ligne et les helpers partagés — cette story ne fait que **remplir** un champ que 16-1 a créé.

---

## Contexte — trois faits établis par relevé, dont un qui renverse la prémisse

### 1. Le pont catalogue → facture existe déjà, et 16-1 y a laissé l'ancre

La formulation d'origine de l'Epic supposait que « `invoice_lines` ne porte aucun `product_id`, donc un compte sur la fiche produit n'a aucun chemin vers l'écriture ». **La première moitié est vraie, la conclusion ne l'est pas.**

Il n'existe effectivement **aucune** colonne produit sur les tables de lignes :

```
$ grep -rn "product_id" crates/kesh-db/migrations/*.sql
(aucune sortie)
```

Mais un pont **d'interface** existe : `InvoiceForm.svelte` propose deux boutons, « Ligne libre » et « **Depuis catalogue** » (`:862-871`). Le second ouvre `ProductPicker.svelte`, et le choix d'un article appelle `onProductSelect(p)` (`:420-430`), qui **recopie** `p.name` → `description`, `p.unitPrice`, `p.vatRate` dans la nouvelle ligne.

Ce site porte déjà, **écrite par 16-1**, l'ancre de cette story :

```ts
function onProductSelect(p: ProductResponse) {
    lines.push({
        description: p.name,
        quantity: '1',
        unitPrice: p.unitPrice,
        vatRate: p.vatRate,
        // AC9-bis — site 3/5. Idem : 16-2 y branchera le compte du produit.
        revenueAccountId: null,
        _uiKey: nextUiKey(),
    });
}
```

**Conséquence de cadrage** : le pré-remplissage est un **geste de recopie au moment du choix**, exactement comme `description`/`unitPrice`/`vatRate` le sont déjà. Il ne demande **aucune** colonne `product_id`, **aucune** relation persistante, **aucun** changement du contrat des lignes de facture.

### 2. Le lien est une copie, jamais une référence — et c'est déjà la règle du dépôt

`CreateInvoiceLineRequest` (`frontend/src/lib/features/invoices/invoices.types.ts:190-206`) ne porte aucun `productId`, et rien de l'acte de sélection n'est transmis au backend. Une ligne issue du catalogue est **indiscernable** d'une ligne saisie à la main portant les mêmes valeurs.

Ce n'est pas un manque à combler ici : c'est la sémantique voulue, énoncée par le doc-module de `crates/kesh-db/src/entities/invoice.rs:1-5` — *« Le catalogue n'est qu'un accélérateur de saisie »*. Une facture doit rester **auto-descriptive** : changer le prix d'un article demain ne doit pas réécrire les factures d'hier.

### 3. Le patron « compte par défaut » existe déjà au niveau société

`company_invoice_settings.default_revenue_account_id BIGINT NULL`, FK vers `accounts` `ON DELETE RESTRICT` (`crates/kesh-db/migrations/20260417000001_invoice_validation.sql:39-47`). Cette story en fait le **jumeau au niveau article**. La convention `ON DELETE RESTRICT` est unanime dans le dépôt — 11 FK vers `accounts`, motif écrit en toutes lettres dans `20260727000001_invoice_lines_revenue_account.sql:21-23`.

---

## Décisions

- **D1 — Aucun `product_id`, aucune relation persistante.** Le compte est recopié dans la ligne au moment du choix de l'article, au site `onProductSelect` que 16-1 a préparé. **Motif** : c'est le geste que #144 demande (*« servirait de valeur par défaut lors de la création de lignes »*), c'est l'arbitrage de kickoff de Guy (pré-remplissage **sans reprise rétroactive**), et c'est la sémantique de snapshot déjà appliquée aux trois autres champs recopiés. Introduire `product_id` serait une story distincte, bien plus large (sémantique du `ON DELETE`, rapports « produits vendus », cascade d'édition) et **non demandée**.

- **D2 — `products.default_revenue_account_id BIGINT NULL`**, FK `ON DELETE RESTRICT`, index dédié. Miroir strict de `company_invoice_settings.default_revenue_account_id`. `NULL` signifie « cet article n'impose rien » — la ligne retombe alors sur le comportement livré par 16-1, c'est-à-dire `revenueAccountId: null`, donc le compte par défaut **de la société**, résolu à la validation.

- **D3 — Le compte de l'article est validé à l'enregistrement de la fiche, avec le prédicat des lignes MOINS l'exemption D3-bis.** Les quatre critères de `validate_line_revenue_accounts_in_tx` (`invoices.rs:514-590`) s'appliquent : existe et appartient à la société, `active`, `account_type = Revenue`, `postable`. **L'exemption D3-bis n'est PAS reprise** : elle existe pour ne pas rejeter un réglage société **préexistant** devenu non imputable, pas pour autoriser la **création** de nouvelles références vers un compte non imputable. Un article ne peut donc pas désigner un compte non imputable, même si c'est le défaut de la société.

  ⚠️ **Conséquence à assumer, et à ne pas découvrir en revue** : sur une société dont le compte de produit par défaut est non imputable (cas que D3-bis tolère), un article **ne peut pas** désigner explicitement ce compte-là. Il doit rester à `NULL`, ce qui produit exactement le même résultat comptable — la ligne suit le défaut société. Le cas est donc sans perte fonctionnelle.

- **D4 — Un compte devenu inutilisable est pré-rempli quand même, puis signalé par le marqueur de 16-1.** Si l'article référence un compte archivé, retypé ou devenu non imputable, `onProductSelect` recopie la valeur telle quelle ; le sélecteur de ligne l'affiche, le marque `markInvalid` et bloque l'enregistrement en nommant la ligne. **Motif** : c'est l'arbitrage rendu par Guy en 16-1b — *afficher + signaler + bloquer* — et la machinerie existe déjà (`isAccountUnusable`, `AccountAutocomplete markInvalid`, `allowClear`). L'alternative (ne pas pré-remplir, retomber en silence sur le défaut société) **cacherait** à l'utilisateur que la fiche produit est à corriger.

- **D5 — Le `PUT` produit reste full-replace, et cette story en aggrave la portée.** `products::update` écrit les quatre colonnes métier inconditionnellement (`repositories/products.rs:310-313`) ; `ProductUpdate` n'a aucun champ sentinelle « ne pas toucher ». Ajouter une cinquième colonne signifie qu'**un client d'API qui relit puis réécrit une fiche sans la clé `defaultRevenueAccountId` efface le compte**, sans erreur. C'est exactement le CR **#278**, dont l'arbitrage (story 16-1a) est de ne pas durcir le contrat maintenant. **Obligations de cette story** : le frontend envoie **toujours** le champ ; le CHANGELOG porte l'avertissement aux clients d'API ; aucune tentative de sémantique « conserver » n'est faite ici.

- **D6 — Le compte de charge (achat) est HORS PÉRIMÈTRE.** #144 le pose comme *« optionnel / à discuter »*. Il n'existe **aucun** chemin entre le catalogue et les factures fournisseur : `supplier_invoice_lines.expense_account_id` est une FK **obligatoire** saisie à la main (`20260628000001_supplier_invoices.sql:79-96`), et aucun sélecteur d'article n'existe sur ce formulaire. Livrer le champ sans son chemin produirait une colonne que rien ne lit — le mode d'échec que 16-1c a documenté sous le nom de « code mort qui paraît fonctionner ». **À rouvrir en story dédiée si le besoin se confirme.**

- **D7 — Aucune reprise rétroactive.** Arbitrage de kickoff. La migration est du **DDL pur** : aucun `UPDATE`, aucun `INSERT`. Les articles existants naissent à `NULL`, ce qui préserve strictement le comportement actuel.

---

## Acceptance Criteria

- **AC1 — Schéma.** Migration ajoutant `products.default_revenue_account_id BIGINT NULL`, contrainte `fk_products_default_revenue_account` vers `accounts(id)` `ON DELETE RESTRICT`, et `INDEX idx_products_default_revenue_account`. **DDL pur** — la migration ne contient aucun statement d'écriture de données.

- **AC2 — Entité et repository.** `Product`, `NewProduct` et `ProductUpdate` portent le champ `default_revenue_account_id: Option<i64>`. La constante `COLUMNS` (`repositories/products.rs:22-23`) l'inclut, ainsi que les `INSERT` et l'`UPDATE`.

  ⚠️ **DEUX helpers écrits À LA MAIN doivent être étendus, et les oublier produit deux défauts SILENCIEUX distincts.** Aucun des deux ne dérive de la struct : ils énumèrent les champs un par un, donc le compilateur ne dira rien.

  - `is_no_op_change` (`:256-261`) compare **quatre** champs. Non étendu, une modification portant **uniquement** sur le compte est prise pour un no-op : l'`UPDATE` est court-circuité, la `version` ne bouge pas, **aucun audit n'est écrit — et l'API répond succès**. L'utilisateur voit son choix accepté et rien n'est enregistré.
  - `product_snapshot_json` (`:32-43`) énumère **neuf** clés. Non étendu, l'audit `{before, after}` de `product.updated` rend deux objets **identiques** pour un changement de compte : la piste d'audit affirme qu'il ne s'est rien passé. C'est plus grave qu'une absence d'entrée — c'est une entrée qui ment.

  Chacun des deux vient avec son test : modifier le seul compte doit bumper la `version`, écrire l'audit, et l'audit doit montrer `before ≠ after` sur ce champ.

- **AC3 — API.** `CreateProductRequest` et `UpdateProductRequest` acceptent `defaultRevenueAccountId` (camelCase, `Option<i64>`, `#[serde(default)]`) ; `ProductResponse` le restitue. Le compte est validé selon **D3** ; un compte invalide rend **400** avec un code applicatif dédié et le motif du rejet, en réutilisant l'enum `RevenueAccountRejection` (`crates/kesh-db/src/errors.rs:14-27`) plutôt qu'en dupliquant ses variantes.

- **AC4 — Fiche produit (frontend).** Le formulaire de `products/+page.svelte` porte un sélecteur de compte, alimenté par **`fetchAccounts(true)`** — le flag `includeArchived` est **obligatoire** : sans lui un compte archivé n'est plus résoluble et le champ **paraît vide** au lieu d'être signalé, piège qui a coûté une passe de revue à 16-1b et qui ne se manifeste que le jour où un compte est archivé. Props `requiredAccountType="Revenue"` et `allowClear`. À l'édition le champ est hydraté depuis la réponse serveur ; à la soumission il est **toujours** envoyé (valeur ou `null`), jamais omis — cf. **D5**. Le champ est **facultatif** : une fiche sans compte s'enregistre sans friction.

- **AC5 — Pré-remplissage.** `onProductSelect` (`InvoiceForm.svelte:420-430`) pose `revenueAccountId: p.defaultRevenueAccountId ?? null`. **Le commentaire d'ancrage « la Story 16-2 y branchera » est remplacé** par la description du comportement livré. Le site `addFreeLine` (`:407-417`) **reste à `null`** — une ligne libre ne vient d'aucun article ; son commentaire d'ancrage est corrigé lui aussi, faute de quoi il annoncerait un travail déjà fait.

- **AC6 — Un compte devenu inutilisable est signalé, pas caché** (D4). Une facture montée depuis un article dont le compte est archivé affiche le libellé, le marqueur d'invalidité, et refuse l'enregistrement en nommant la ligne. **Aucun code nouveau n'est attendu ici** : l'AC vérifie que le pré-remplissage passe bien par la machinerie de 16-1, et le test qui le prouve est le livrable.

- **AC7 — Export CSV.** `serialize_products_csv` (`crates/kesh-api/src/exports/csv_tables.rs:364-396`) porte une liste de colonnes **codée en dur** ; elle inclut la nouvelle colonne, à sa position dans `COLUMNS`.

- **AC8 — i18n.** Les libellés nouveaux existent dans les **4** locales (`crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`), sous le préfixe `product-`. ⚠️ Les locales vivent dans un **crate Rust** : le gate inclut donc `cargo test --workspace` même pour un changement perçu comme frontend.

- **AC9 — Garde-fous de migration.** Les trois obligations de la § « Migration breaking policy » sont tenues, chacune **recomptée à la source** :
  - **P5** — `docs/migrations-idempotence-audit.md` porte une ligne pour la nouvelle migration, et les compteurs sont **recomptés** : l'en-tête `## Table d'audit (N migrations)` **et** la ligne `Total` passent de **57** à **58**, la somme des trois compteurs de partition égale ce total. *(Les trois compteurs de partition ne valent pas le total — les aligner dessus casserait l'invariant qu'ils servent à tenir.)*
  - **P6** — la nouvelle migration **casse volontairement** `upgrade_path_preserves_data` : `crates/kesh-db/tests/migrations_upgrade_path.rs:88-89` porte `assert_eq!(total, 57)`. Le porter à **58**, et **trancher explicitement** ce que la fenêtre `total - 23` (frontière à 34) doit devenir — l'assertion de `:35-46` énonce elle-même l'arbitrage : élargir la fenêtre (bumper `total` seul) ou maintenir la frontière à 34 (bumper `total` **et** la fenêtre, donc `total - 24`). Le choix doit être **écrit**, pas subi.
  - **P7** — la migration étant du **DDL pur**, elle ne déclenche pas le détecteur de backfills et n'a besoin **ni** d'entrée au registre `POST_RESTORE_BACKFILLS` **ni** d'exemption. ⚠️ **Le vérifier en exécutant `every_data_backfill_migration_is_triaged`**, pas en le supposant : le détecteur classe sur le premier mot-clé du statement et sa largeur est précisément ce qui a échoué deux fois pendant la spécification de 16-1c.
  - **P1/P2** — `ADD COLUMN` nullable ⇒ **non-breaking** ⇒ **aucun** bump de `kesh_version_min_required`, donc aucun bump de version Cargo (P2-bis).

- **AC10 — Discrimination prouvée par mutation.** Au moins **trois** mutations exécutées et consignées au Dev Agent Record :
  1. `onProductSelect` repose `revenueAccountId: null` au lieu du compte de l'article → le test de bout en bout du pré-remplissage doit rougir, **et lui seul**.
  2. La validation D3 est retirée du `create`/`update` produit → le test du compte invalide doit rougir.
  3. Le champ est retiré du **payload HTTP** envoyé par la fiche produit → seul un test qui traverse réellement HTTP peut l'attraper. *(C'est la mutation la plus instructive de 16-1b : ni Vitest ni les tests Rust ne voient une clé qui disparaît entre les deux.)*

  **Un test attendu qui ne rougit pas invalide le montage** : le corriger avant d'aller plus loin.

- **AC11 — Gate.** `scripts/test-fast.sh` complet **et** le gate frontend (`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`) **et** l'E2E Playwright, sur l'**état final**, exit 0 exigé, non présumé d'un run antérieur.

  ⚠️ **Cette story touche `kesh-db`** (migration + repository) : la § « Test Locally First » → « Pendant une boucle de revue » **interdit le gate ciblé** ici, y compris entre deux passes de revue. Gate complet à chaque fois.

---

## Tasks / Subtasks

- [ ] **T1 — Migration et garde-fous** (AC1, AC9)
  - [ ] `crates/kesh-db/migrations/<timestamp>_products_default_revenue_account.sql` — `ADD COLUMN` + FK `ON DELETE RESTRICT` + index. DDL pur.
  - [ ] Ligne au tableau de `docs/migrations-idempotence-audit.md`, **compteurs recomptés** (57 → 58 aux deux sites, partition cohérente).
  - [ ] `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` — inspecter **chaque** site ; porter `assert_eq!(total, 57)` à 58 et **écrire** l'arbitrage de fenêtre.
  - [ ] Exécuter `every_data_backfill_migration_is_triaged` pour **constater** que le DDL pur ne déclenche rien.
- [ ] **T2 — Backend : entité, repository, API** (AC2, AC3)
  - [ ] Champ dans `Product` / `NewProduct` / `ProductUpdate`, `COLUMNS`, `INSERT`, `UPDATE`, `is_no_op_change`.
  - [ ] Validation D3 sur `create` et `update`, en **réutilisant** `RevenueAccountRejection` — ne pas dupliquer l'enum.
  - [ ] DTO d'API + `ProductResponse`, contrat camelCase.
- [ ] **T3 — Frontend : fiche produit** (AC4, AC8)
  - [ ] Sélecteur de compte dans le formulaire, hydraté à l'édition, **toujours** envoyé à la soumission.
  - [ ] Types TS, client API, clés i18n dans les **4** locales.
- [ ] **T4 — Frontend : pré-remplissage** (AC5)
  - [ ] `onProductSelect` pose le compte de l'article ; `addFreeLine` reste à `null`.
  - [ ] **Remplacer les deux commentaires d'ancrage** « la Story 16-2 y branchera » (`:413-414` et `:426`) par la description du comportement livré.
- [ ] **T5 — Export CSV** (AC7)
  - [ ] Étendre `serialize_products_csv`, en-tête **et** valeurs.
- [ ] **T6 — Tests et preuve** (AC6, AC10)
  - [ ] Tests d'intégration `kesh-db` : le champ persiste, la validation rejette, le no-op reste no-op.
  - [ ] Test unitaire frontend du pré-remplissage.
  - [ ] **E2E** : parcours « fiche produit avec compte → facture depuis catalogue → la ligne porte le compte », **plus** le cas AC6 (compte archivé après coup → marqueur + blocage).
  - [ ] Les **trois mutations** d'AC10, exécutées, consignées avec leur sortie, fichiers restaurés à l'identique.
- [ ] **T7 — Documentation** (AC11)
  - [ ] CHANGELOG : la fonctionnalité **et** l'avertissement `PUT` full-replace de D5.
  - [ ] Manuel utilisateur si le geste est visible côté fiduciaire.
- [ ] **T8 — Gate complet** (AC11) — backend + frontend + E2E, état final, exit 0, **verdict lu dans le log**.

---

## Dev Notes

### Ce que cette story ne doit PAS faire

- **Ne pas ajouter `product_id`** à `invoice_lines` (D1). Si l'implémentation semble l'exiger, c'est que le périmètre a dérivé.
- **Ne pas reprendre le parc** (D7) : aucune migration de données.
- **Ne pas toucher au compte de charge / achat** (D6).
- **Ne pas tenter de corriger le contrat `PUT`** (D5) — c'est #278, arbitré ailleurs.
- **Ne pas dupliquer** `isAccountUnusable` ni `RevenueAccountRejection`. Les deux existent et sont la source unique de vérité de leur verdict ; une seconde copie divergerait au premier patch.

### Le piège de `fetchAccounts(true)`

16-1b a payé une passe de revue sur ce point : sans le flag `includeArchived`, un compte archivé n'est plus résoluble et le champ **paraît vide** au lieu d'être signalé. Le piège est silencieux — il ne se manifeste que le jour où un compte est archivé. La fiche produit doit charger les comptes de la même façon que les écrans qui affichent une valeur persistée.

### Le composant de sélection est partagé par 5 écrans

`AccountAutocomplete` (`frontend/src/lib/features/journal-entries/AccountAutocomplete.svelte`) est importé par `InvoiceForm`, `JournalEntryForm`, `VatPurchaseAssistant`, `ManualMatchModal` et `TransactionSplitModal`. Toutes les extensions de 16-1 sont des props **opt-in** dont le défaut préserve le comportement antérieur. Si cette story a besoin d'une variation, **ajouter une prop opt-in** — ne jamais changer un défaut.

### Le lint d'appartenance des clés i18n

`npm run lint-i18n-ownership` ne parcourt que **`src/lib/features`** (`frontend/scripts/lint-i18n-ownership.*:17`) et n'y tolère qu'un préfixe correspondant au dossier, ou l'un des **six** espaces globaux : `error`, `tooltip`, `common`, `mode`, `shortcut`, `demo`.

La page catalogue vit dans `src/routes/(app)/products/`, **hors du périmètre du lint** — un préfixe `product-` y est donc sans risque. Mais toute clé nouvelle qui finirait consommée depuis un composant partagé sous `features/` (par exemple `AccountAutocomplete`, qui vit dans `features/journal-entries/`) **doit** porter un préfixe global. C'est exactement l'erreur commise en 16-1b, où une clé a dû être renommée `account-label-default-suffix` → `common-account-default-suffix` après un lint rouge. Faire passer un libellé **en prop** depuis la page évite le problème à la racine.

### Conventions de test du dépôt

- Intégration `kesh-db` : `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]`, fixtures `kesh_db::test_fixtures::{SeededCompany, seed_accounting_company}`.
- E2E Playwright : le suffixe **`.spec.ts` est obligatoire** — un fichier `*.test.ts` dans `tests/e2e/` est **silencieusement ignoré**.
- Les locales étant un crate Rust, un changement i18n impose `cargo test --workspace`.

### References

- Issue **#144** — l'énoncé et ses critères d'acceptation d'origine.
- Story **16-1a** — `revenue_account_id`, matérialisation à la validation (`invoices.rs:1672-1728`), règles de validité (`:514-590`), décision **D3-bis**.
- Story **16-1b** — `AccountAutocomplete` et ses props opt-in, `account-label.ts`, `account-validity.ts`, arbitrage *afficher + signaler + bloquer*.
- Story **16-1c** — la § P7 et l'enseignement « une entrée de registre hors fenêtre est du code mort qui paraît fonctionner ».
- `CLAUDE.md` § « Migration breaking policy » (P1/P2, P5, P6, P7) et § « Test Locally First ».
- CR **#278** — durcissement du contrat des API en écriture, différé.

---

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

**2026-08-03 — Spec créée** (`bmad-create-story`, Opus 5), sur la branche `story/16-2-compte-produit-catalogue` issue de `story/16-1-compte-produit-par-ligne` — et non de `main`, qui ne porte pas encore le socle 16-1 (PR #284 ouverte, CI verte). Les ancres de cette spec sont invérifiables sur `main` tant que #284 n'est pas mergée ; **rebaser après le merge**.

**La prémisse de départ était fausse, et c'est le fait le plus utile de cette création.** L'énoncé de kickoff posait qu'« un compte sur la fiche produit n'a aucun chemin vers l'écriture tant que la ligne ne porte pas le champ ». La 16-1 a livré le champ ; mais surtout, le relevé montre qu'un pont **d'interface** existait déjà — bouton « Depuis catalogue » → `ProductPicker` → `onProductSelect` — et que **16-1 y a laissé l'ancre explicite de cette story**, à deux endroits, en toutes lettres. Le périmètre s'en trouve considérablement réduit : ni `product_id`, ni relation persistante, ni changement du contrat des lignes. C'est une recopie de plus à un endroit déjà écrit pour la recevoir.

**Deux affirmations d'un rapport d'exploration ont été réfutées avant d'entrer dans la spec** : un garde-fou `assert_eq!(total, 55)` cité dans `migrations_upgrade_path.rs` — inexistant (`grep` sans sortie), la valeur réelle étant **57** à `:88-89` ; et le décompte de migrations, recompté à **57** aux trois sites du document d'audit plutôt que relu. La discipline de la § « Propagation post-patch » s'applique aux rapports d'agents comme au reste : une ancre fausse coûte une passe de revue entière.

**Statut : `ready-for-dev`, non validée.** La boucle `bmad-create-story validate` n'a pas encore tourné.
