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

Ce site porte déjà, **écrite par 16-1**, l'ancre de cette story. *(« site 3/5 » renvoie aux **cinq** endroits d'`InvoiceForm.svelte` qui construisent une `LineState`, numérotés par 16-1b sous le marqueur `AC9-bis — site N/5` ; seuls les sites **2** — `addFreeLine` — et **3** — `onProductSelect` — concernent cette story, les trois autres recopiant une valeur déjà décidée.)*

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

`company_invoice_settings.default_revenue_account_id BIGINT NULL`, FK vers `accounts` `ON DELETE RESTRICT` (`crates/kesh-db/migrations/20260417000001_invoice_validation.sql:39-47`). Cette story en fait le **jumeau au niveau article**. La convention `ON DELETE RESTRICT` est unanime dans le dépôt — **13** FK vers `accounts` (`grep -rn "REFERENCES accounts" crates/kesh-db/migrations/*.sql | wc -l`), motif écrit en toutes lettres dans `20260727000001_invoice_lines_revenue_account.sql:21-23`. *(Ce commentaire annonce « 11 » : c'était exact quand 16-1a l'a écrit, avant son propre `ALTER TABLE`. Recompter, ne pas relire.)*

---

## Décisions

- **D1 — Aucun `product_id`, aucune relation persistante.** Le compte est recopié dans la ligne au moment du choix de l'article, au site `onProductSelect` que 16-1 a préparé. **Motif** : c'est le geste que #144 demande (*« servirait de valeur par défaut lors de la création de lignes »*), c'est l'arbitrage de kickoff de Guy (pré-remplissage **sans reprise rétroactive**), et c'est la sémantique de snapshot déjà appliquée aux trois autres champs recopiés. Introduire `product_id` serait une story distincte, bien plus large (sémantique du `ON DELETE`, rapports « produits vendus », cascade d'édition) et **non demandée**.

- **D2 — `products.default_revenue_account_id BIGINT NULL`**, FK `ON DELETE RESTRICT`, index dédié. Miroir strict de `company_invoice_settings.default_revenue_account_id`. `NULL` signifie « cet article n'impose rien » — la ligne retombe alors sur le comportement livré par 16-1, c'est-à-dire `revenueAccountId: null`, donc le compte par défaut **de la société**, résolu à la validation.

- **D3 — Le compte de l'article est validé sur TROIS critères, `postable` EXCLU — miroir exact du réglage société.** À l'enregistrement de la fiche (`create` **et** `update`) :

  1. **exister et appartenir à la société** — les deux ne font qu'**un** critère, sanctionné par une **seule** variante de rejet, `RevenueAccountRejection::UnknownOrCrossCompany` : un compte d'une autre société est rendu indiscernable d'un compte inexistant, c'est la garde anti-IDOR ;
  2. `active` ;
  3. `account_type = Revenue`.

  **`postable` n'est PAS contrôlé ici** — c'est le quatrième critère des lignes, et il est délibérément écarté.

  **Motif, et il est établi par le code jumeau, pas par analogie** : `company_invoice_settings::validate_account` (`crates/kesh-api/src/routes/company_invoice_settings.rs:94-121`) ne vérifie **jamais** `postable` — `grep -n "postable" crates/kesh-api/src/routes/company_invoice_settings.rs` ne rend **aucune ligne**. Le réglage société, qui est le patron dont cette story se réclame, laisse donc délibérément passer un compte non imputable au moment du *réglage*, et ne le sanctionne qu'à la *validation de facture* — c'est là que D3-bis intervient.

  ⚠️ **Une rédaction antérieure de cette décision imposait les quatre critères, `postable` compris. Elle créait un blocage silencieux sur un champ que l'utilisateur ne touche pas**, et le scénario est concret : un article porte un compte valide ; plus tard un administrateur rend ce compte non imputable par un flux sans aucun rapport (`accounts::update`) ; l'utilisateur renomme l'article ou change son prix ; `is_no_op_change` rend `false` puisque le nom a changé, l'`UPDATE` est tenté, et la validation **rejette en 400 à cause du compte** — que l'utilisateur n'a pas touché et qu'il ne peut débloquer qu'en vidant le champ au passage. C'est le symétrique exact du problème que D3-bis a été inventée pour résoudre côté société, réintroduit un cran plus tôt dans le cycle de vie. *(Relevé en passe 1 de `validate`.)*

  **Où le signal est-il rendu, alors ?** Là où 16-1 l'a déjà construit : au niveau de la ligne de facture, par le marqueur d'invalidité et le blocage (cf. **D4**). Un compte devenu inutilisable ne se découvre pas en éditant le catalogue — il se découvre au moment où l'on s'en sert.

- **D4 — Un compte devenu inutilisable est pré-rempli quand même, puis signalé par le marqueur de 16-1.** Si l'article référence un compte archivé, retypé ou devenu non imputable, `onProductSelect` recopie la valeur telle quelle ; le sélecteur de ligne l'affiche, le marque `markInvalid` et bloque l'enregistrement en nommant la ligne. **Motif** : c'est l'arbitrage rendu par Guy en 16-1b — *afficher + signaler + bloquer* — et la machinerie existe déjà (`isAccountUnusable`, `AccountAutocomplete markInvalid`, `allowClear`). L'alternative (ne pas pré-remplir, retomber en silence sur le défaut société) **cacherait** à l'utilisateur que la fiche produit est à corriger.

- **D5 — Le `PUT` produit reste full-replace, et cette story en aggrave la portée.** `products::update` écrit les quatre colonnes métier inconditionnellement (`repositories/products.rs:310-313`) ; `ProductUpdate` n'a aucun champ sentinelle « ne pas toucher ». Ajouter une cinquième colonne signifie qu'**un client d'API qui relit puis réécrit une fiche sans la clé `defaultRevenueAccountId` efface le compte**, sans erreur. C'est exactement le CR **#278**, dont l'arbitrage (story 16-1a) est de ne pas durcir le contrat maintenant. **Obligations de cette story** : le frontend envoie **toujours** le champ ; le CHANGELOG porte l'avertissement aux clients d'API ; aucune tentative de sémantique « conserver » n'est faite ici.

- **D6 — Le compte de charge (achat) est HORS PÉRIMÈTRE.** #144 le pose comme *« optionnel / à discuter »*. Il n'existe **aucun** chemin entre le catalogue et les factures fournisseur : `supplier_invoice_lines.expense_account_id` est une FK **obligatoire** saisie à la main (`20260628000001_supplier_invoices.sql:79-96`), et aucun sélecteur d'article n'existe sur ce formulaire. Livrer le champ sans son chemin produirait une colonne que rien ne lit — le mode d'échec que 16-1c a documenté sous le nom de « code mort qui paraît fonctionner ». **À rouvrir en story dédiée si le besoin se confirme.**

- **D7 — Aucune reprise rétroactive.** Arbitrage de kickoff. La migration est du **DDL pur** : aucun `UPDATE`, aucun `INSERT`. Les articles existants naissent à `NULL`, ce qui préserve strictement le comportement actuel.

- **D8 — La frontière de la fenêtre d'upgrade RESTE à 34 : bumper `total` ET la fenêtre.** La nouvelle migration casse volontairement `upgrade_path_preserves_data` (`crates/kesh-db/tests/migrations_upgrade_path.rs:88-92`), dont l'assertion `assert_eq!(total, 57)` passe à **58**. La fenêtre s'écrit `total - 23`, ce qui donne aujourd'hui 34 migrations montées. **Laisser `- 23` avec `total = 58` déplacerait la frontière à 35** — le test cesserait de vérifier l'upgrade depuis l'état historique qu'il a été écrit pour couvrir, et ce glissement se répéterait à chaque migration. Elle passe donc à **`total - 24`**.

  **Motif** : la frontière 34 matérialise un état historique du schéma, pas une distance au présent. Une fenêtre qui rétrécit d'un cran à chaque migration finit par ne plus tester aucun upgrade réel — c'est la version lente du mode d'échec que le garde-fou **P6** existe pour attraper. L'assertion de montage (`:35-46`) énonce elle-même les deux branches de l'arbitrage ; celle-ci est tranchée ici plutôt que laissée au développeur. *(Décision ajoutée en passe 1 de `validate`, qui a relevé que l'AC exigeait un choix sans fournir de critère.)*

---

## Règle de splitting préventif — évaluation, et arbitrage en attente

*(Section ajoutée en passe 1 de `validate`, qui a relevé que le seuil n'avait jamais été évalué. Il aurait dû l'être **avant** de lancer `bmad-create-story`.)*

Le premier critère de la § « Règle de splitting préventif » est **plus de 5 modules distincts**. Recompté à la granularité que 16-1a et 16-1b ont elles-mêmes utilisée pour justifier leur split, cette story en touche **sept** :

| # | Module | Nature de la touche |
|---|---|---|
| 1 | `crates/kesh-db/migrations` | 1 fichier neuf, DDL pur |
| 2 | `crates/kesh-db` entités + `repositories/products` | champ + 2 helpers + `COLUMNS` |
| 3 | `crates/kesh-api/routes/products` | 3 DTO + validation |
| 4 | `crates/kesh-api/exports/csv_tables` | **une ligne** d'en-tête + une de valeur |
| 5 | `crates/kesh-i18n/locales` | quelques clés × 4 locales |
| 6 | `frontend` page catalogue + `features/products` | sélecteur + types + client API |
| 7 | `frontend/components/invoices/InvoiceForm.svelte` | **une ligne** de pré-remplissage |

**Le seuil est franchi. Deux lectures s'opposent, et l'arbitrage appartient au Project Lead.**

- **En faveur du split** : la règle est un seuil, pas une appréciation, et elle existe précisément parce que l'estimation « c'est petit » est le biais qu'elle corrige. 16-1 a été splittée sur ce même critère et s'en est bien portée.
- **En faveur d'une dérogation** : le décompte est gonflé par des modules touchés d'**une seule ligne** (#4 et #7). Surtout, un split naturel « backend / frontend » séparerait la colonne de son **unique consommateur** — le pré-remplissage — et livrerait une sous-story dont rien ne lit la donnée, c'est-à-dire exactement le « code mort qui paraît fonctionner » que **D6** invoque pour exclure le compte de charge. La substance est ici : une colonne, un sélecteur, une ligne de recopie.

### Dérogation règle de splitting — arbitrage rendu

**Guy, 2026-08-03 : DÉROGATION.** La story reste unique.

**Justification retenue** : deux des sept modules sont touchés d'**une seule ligne** (#4 export CSV, #7 pré-remplissage), et le seul split naturel — backend / frontend — séparerait la colonne de son **unique consommateur**, livrant une sous-story dont rien ne lit la donnée. C'est le « code mort qui paraît fonctionner » que **D6** invoque pour exclure le compte de charge : appliquer la règle ici produirait exactement le défaut qu'elle cherche à prévenir ailleurs.

**Risque accepté** : si une passe `N+1` de `validate` remonte une sévérité **égale ou supérieure** à la passe `N`, le second critère de la règle se déclenche et l'arbitrage est **rouvert** — sans discussion. C'est le garde-fou qui rend cette dérogation tenable.

---

## Acceptance Criteria

- **AC1 — Schéma** (D2). Migration ajoutant `products.default_revenue_account_id BIGINT NULL`, contrainte `fk_products_default_revenue_account` vers `accounts(id)` `ON DELETE RESTRICT`, et `INDEX idx_products_default_revenue_account`. **DDL pur** — la migration ne contient aucun statement d'écriture de données.

- **AC2 — Entité et repository.** `Product`, `NewProduct` et `ProductUpdate` portent le champ `default_revenue_account_id: Option<i64>`. La constante `COLUMNS` (`repositories/products.rs:22-23`) l'inclut, ainsi que les `INSERT` et l'`UPDATE`.

  ⚠️ **DEUX helpers écrits À LA MAIN doivent être étendus, et les oublier produit deux défauts SILENCIEUX distincts.** Aucun des deux ne dérive de la struct : ils énumèrent les champs un par un, donc le compilateur ne dira rien.

  - `is_no_op_change` (`:256-261`) compare **quatre** champs. Non étendu, une modification portant **uniquement** sur le compte est prise pour un no-op : l'`UPDATE` est court-circuité, la `version` ne bouge pas, **aucun audit n'est écrit — et l'API répond succès**. L'utilisateur voit son choix accepté et rien n'est enregistré.
  - `product_snapshot_json` (`:32-43`) énumère **huit** clés (`id`, `companyId`, `name`, `description`, `unitPrice`, `vatRate`, `active`, `version`). Non étendu, l'audit `{before, after}` de `product.updated` rend deux objets **identiques** pour un changement de compte : la piste d'audit affirme qu'il ne s'est rien passé. C'est plus grave qu'une absence d'entrée — c'est une entrée qui ment.

  Chacun des deux vient avec son test : modifier le seul compte doit bumper la `version`, écrire l'audit, et l'audit doit montrer `before ≠ after` sur ce champ.

- **AC3 — API.** `CreateProductRequest` et `UpdateProductRequest` acceptent `defaultRevenueAccountId` (camelCase, `Option<i64>`, `#[serde(default)]`) ; `ProductResponse` le restitue. Le compte est validé selon **D3** — **trois** critères, `postable` exclu ; un compte invalide rend **400** avec un code applicatif dédié et le motif du rejet, en réutilisant l'enum `RevenueAccountRejection` (`crates/kesh-db/src/errors.rs:14-27`) plutôt qu'en dupliquant ses variantes.

- **AC4 — Fiche produit (frontend).** Le formulaire de `products/+page.svelte` porte un sélecteur de compte, alimenté par **`fetchAccounts(true)`** — le flag `includeArchived` est **obligatoire** : sans lui un compte archivé n'est plus résoluble et le champ **paraît vide** au lieu d'être signalé, piège qui a coûté une passe de revue à 16-1b et qui ne se manifeste que le jour où un compte est archivé. Props `requiredAccountType="Revenue"` et `allowClear`. À l'édition le champ est hydraté depuis la réponse serveur ; à la soumission il est **toujours** envoyé (valeur ou `null`), jamais omis — cf. **D5**. Le champ est **facultatif** : une fiche sans compte s'enregistre sans friction.

- **AC5 — Pré-remplissage.** `onProductSelect` (`InvoiceForm.svelte:420-430`) pose `revenueAccountId: p.defaultRevenueAccountId ?? null`. **Le commentaire d'ancrage « la Story 16-2 y branchera » est remplacé** par la description du comportement livré. Le site `addFreeLine` (`:407-418`) **reste à `null`** — une ligne libre ne vient d'aucun article ; son commentaire d'ancrage est corrigé lui aussi, faute de quoi il annoncerait un travail déjà fait.

- **AC6 — Un compte devenu inutilisable est signalé, pas caché** (D4). Une facture montée depuis un article dont le compte est archivé affiche le libellé, le marqueur d'invalidité, et refuse l'enregistrement en nommant la ligne. **Aucun code nouveau n'est attendu ici** : l'AC vérifie que le pré-remplissage passe bien par la machinerie de 16-1, et le test qui le prouve est le livrable.

- **AC7 — Export CSV.** `serialize_products_csv` (`crates/kesh-api/src/exports/csv_tables.rs:364-397`) porte une liste de colonnes **codée en dur** ; elle inclut la nouvelle colonne, à sa position dans `COLUMNS`.

- **AC8 — i18n.** Les libellés nouveaux existent dans les **4** locales (`crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`), sous le préfixe `product-` **tant qu'ils sont consommés depuis la page catalogue**, qui vit hors du périmètre du lint. ⚠️ **Tout libellé qui finirait consommé depuis un composant partagé sous `src/lib/features/` doit porter un préfixe global** (`error`, `tooltip`, `common`, `mode`, `shortcut`, `demo`) — sans quoi `lint-i18n-ownership` échoue. Le passer **en prop** depuis la page évite la question ; c'est l'erreur exacte commise en 16-1b, où une clé a dû être renommée après un lint rouge. Détail en Dev Notes. ⚠️ Les locales vivent dans un **crate Rust** : le gate inclut donc `cargo test --workspace` même pour un changement perçu comme frontend.

- **AC9 — Garde-fous de migration.** Les **quatre** obligations de la § « Migration breaking policy » sont tenues, chacune **recomptée à la source** :
  - **P5** — `docs/migrations-idempotence-audit.md` porte une ligne pour la nouvelle migration, et les compteurs sont **recomptés** : l'en-tête `## Table d'audit (N migrations)` **et** la ligne `Total` passent de **57** à **58**, la somme des trois compteurs de partition — `yes`, `tracked-by-sqlx`, `no` — égale ce total. *(Les trois compteurs de partition ne valent pas le total — les aligner dessus casserait l'invariant qu'ils servent à tenir.)*
  - **P6** — la nouvelle migration **casse volontairement** `upgrade_path_preserves_data` : `crates/kesh-db/tests/migrations_upgrade_path.rs:88-89` porte `assert_eq!(total, 57)`. Le porter à **58** **et** passer la fenêtre à `total - 24`, conformément à **D8** : la frontière reste à **34**. Ne pas se contenter de bumper `total`, ce qui déplacerait la frontière à 35 et ferait rétrécir la couverture d'un cran à chaque migration future.
  - **P7** — la migration étant du **DDL pur**, elle ne déclenche pas le détecteur de backfills et n'a besoin **ni** d'entrée au registre `POST_RESTORE_BACKFILLS` **ni** d'exemption. ⚠️ **Le vérifier en exécutant `every_data_backfill_migration_is_triaged`**, pas en le supposant : le détecteur classe sur le premier mot-clé du statement et sa largeur est précisément ce qui a échoué deux fois pendant la spécification de 16-1c.
  - **P1/P2** — `ADD COLUMN` nullable ⇒ **non-breaking** ⇒ **aucun** bump de `kesh_version_min_required`, donc aucun bump de version Cargo (P2-bis).

- **AC10 — Discrimination prouvée par mutation.** Au moins **cinq** mutations exécutées et consignées au Dev Agent Record :
  1. `onProductSelect` repose `revenueAccountId: null` au lieu du compte de l'article → le test de bout en bout du pré-remplissage doit rougir, **et lui seul**.
  2. La validation D3 est retirée du `create`/`update` produit → le test du compte invalide doit rougir.
  3. Le champ est retiré du **payload HTTP** envoyé par la fiche produit → seul un test qui traverse réellement HTTP peut l'attraper. *(C'est la mutation la plus instructive de 16-1b : ni Vitest ni les tests Rust ne voient une clé qui disparaît entre les deux.)*
  4. Le champ est retiré de **`is_no_op_change`** → le test « modifier le seul compte bumpe la `version` » doit rougir. Sans cette mutation, rien ne prouve que le court-circuit no-op a été étendu — et son échec est **muet** : l'API répond succès.
  5. Le champ est retiré de **`product_snapshot_json`** → le test « l'audit montre `before ≠ after` sur le compte » doit rougir. Sans cette mutation, rien ne prouve que la piste d'audit dit vrai.

  ⚠️ **Les mutations 4 et 5 sont ajoutées en passe 2**, sur convergence de deux lentilles : AC2 désignait ces deux helpers comme les défauts les plus graves de la story, et **aucune des trois mutations d'origine ne les discriminait**. Une spec qui nomme un risque sans le faire prouver ne l'a pas traité.

  **Un test attendu qui ne rougit pas invalide le montage** : le corriger avant d'aller plus loin.

- **AC11 — Gate.** `scripts/test-fast.sh` complet **et** le gate frontend (`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`) **et** l'E2E Playwright, sur l'**état final**, exit 0 exigé, non présumé d'un run antérieur.

  ⚠️ **Cette story touche `kesh-db`** (migration + repository) : la § « Test Locally First » → « Pendant une boucle de revue » **interdit le gate ciblé** ici, y compris entre deux passes de revue. Gate complet à chaque fois.

- **AC12 — Documentation synchronisée.** Le **CHANGELOG** porte la fonctionnalité **et** l'avertissement aux clients d'API de **D5** (le `PUT` full-replace efface un `defaultRevenueAccountId` omis). La **feuille de route du `README`** est vérifiée : l'Epic 16 reste « 🚧 En cours » tant que #144 et #151 ne sont pas tous deux livrés, et la section « Fonctionnalités » ne doit pas annoncer comme acquis ce qui ne l'est pas. *(Critère ajouté en passe 1 de `validate` : l'obligation CHANGELOG ne découlait que d'une décision et d'une tâche mal étiquetée, donc d'aucun critère cochable — et la vérification README, que la story sœur 16-1b portait en AC18, avait disparu.)*

---

## Tasks / Subtasks

- [ ] **T1 — Migration et garde-fous** (AC1, AC9)
  - [ ] `crates/kesh-db/migrations/<timestamp>_products_default_revenue_account.sql` — `ADD COLUMN` + FK `ON DELETE RESTRICT` + index. DDL pur.
  - [ ] Ligne au tableau de `docs/migrations-idempotence-audit.md`, **compteurs recomptés** (57 → 58 aux deux sites, partition cohérente).
  - [ ] `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` — inspecter **chaque** site ; porter `assert_eq!(total, 57)` à 58 et **écrire** l'arbitrage de fenêtre.
  - [ ] Exécuter `every_data_backfill_migration_is_triaged` pour **constater** que le DDL pur ne déclenche rien.
- [ ] **T2 — Backend : entité, repository, API** (AC2, AC3)
  - [ ] Champ dans `Product` / `NewProduct` / `ProductUpdate`, `COLUMNS`, `INSERT`, `UPDATE`.
  - [ ] **Étendre les DEUX helpers écrits à la main** — `is_no_op_change` (`:256-261`) **et** `product_snapshot_json` (`:32-43`). Cf. AC2 : les oublier produit deux défauts silencieux distincts, et **aucun des deux ne fait échouer la compilation**.
  - [ ] **Site de construction hors périmètre thématique** : `NewProduct` est aussi construit en littéral dans `crates/kesh-db/tests/kf005_fulltext_index_e2e.rs:123` (test de l'index full-text KF-005). La struct ne dérive pas `Default`, donc l'ajout du champ **casse la compilation** de ce fichier — échec bruyant, mais site à toucher que rien d'autre ne signale.
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
  - [ ] **E2E** (suffixe `.spec.ts` **obligatoire**) : parcours « fiche produit avec compte → facture depuis catalogue → la ligne porte le compte », **plus** le cas AC6 — assigner un compte, l'**archiver ensuite**, monter une facture depuis l'article, constater le marqueur et le refus d'enregistrement.
  - [ ] **Scoping multi-tenant** : un test refuse un compte appartenant à une **autre** société (`UnknownOrCrossCompany`). L'enum le couvre, mais rien ne le prouve tant qu'aucun test ne l'exerce.
  - [ ] Les **trois mutations** d'AC10, exécutées, consignées avec leur sortie, fichiers restaurés à l'identique.
- [ ] **T7 — Documentation** (AC12)
  - [ ] CHANGELOG : la fonctionnalité **et** l'avertissement `PUT` full-replace de D5.
  - [ ] `README.md` — feuille de route et section « Fonctionnalités » vérifiées (même si la conclusion est « rien à changer »).
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

### La branche ne part pas de `main` — mécanique du rebase

Cette branche est issue de `story/16-1-compte-produit-par-ligne`, non de `main` : les ancres de cette spec sont invérifiables sur `main` tant que la **PR #284** n'est pas mergée.

⚠️ **#284 sera mergée en *squash*** — c'est la convention du dépôt (précédents : PR #275 squash `dc3ece04`, PR #200, PR #267). Un `git rebase main` naïf rejouerait alors les **53 commits** de 16-1 un par un contre un `main` qui les contient déjà **sous forme d'un seul commit**, produisant un conflit à chaque fichier partagé. La forme correcte est :

```sh
git checkout main && git pull --ff-only
git rebase --onto main <sha-du-point-de-fork> story/16-2-compte-produit-catalogue
```

où `<sha-du-point-de-fork>` vaut **`0ce6e13a`** — le sommet de `story/16-1-compte-produit-par-ligne` au moment où cette branche en est issue. *(Le relever maintenant : après un squash-merge, `git merge-base` ne le retrouve plus.)* Seuls les commits **propres à 16-2** sont alors rejoués.

**Corollaire sur la PR** : n'ouvrir la PR de 16-2 **qu'après** le merge de #284 et le rebase. Ouverte avant, elle embarquerait tout le diff de 16-1 et serait illisible.

### Le lint d'appartenance des clés i18n

`npm run lint-i18n-ownership` ne parcourt que **`src/lib/features`** (`frontend/scripts/lint-i18n-ownership.*:17`) et n'y tolère qu'un préfixe correspondant au dossier, ou l'un des **six** espaces globaux : `error`, `tooltip`, `common`, `mode`, `shortcut`, `demo`.

La page catalogue vit dans `src/routes/(app)/products/`, **hors du périmètre du lint** — un préfixe `product-` y est donc sans risque. Mais toute clé nouvelle qui finirait consommée depuis un composant partagé sous `features/` (par exemple `AccountAutocomplete`, qui vit dans `features/journal-entries/`) **doit** porter un préfixe global. C'est exactement l'erreur commise en 16-1b, où une clé a dû être renommée `account-label-default-suffix` → `common-account-default-suffix` après un lint rouge. Faire passer un libellé **en prop** depuis la page évite le problème à la racine.

### Conventions de test du dépôt

- Intégration `kesh-db` : `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]`, fixtures `kesh_db::test_fixtures::{SeededCompany, seed_accounting_company}`.
- E2E Playwright : le suffixe **`.spec.ts` est obligatoire** — un fichier `*.test.ts` dans `tests/e2e/` est **silencieusement ignoré**.
- Les locales étant un crate Rust, un changement i18n impose `cargo test --workspace`.

### References

- Issue **#144** — l'énoncé et ses critères d'acceptation d'origine.
- Story **16-1a** — `revenue_account_id`, matérialisation à la validation (`invoices.rs:1672-1727`), règles de validité (`:514-590`), décision **D3-bis**.
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

**Deux affirmations d'un rapport d'exploration ont été réfutées avant d'entrer dans la spec** : un garde-fou `assert_eq!(total, 55)` cité dans `migrations_upgrade_path.rs` — inexistant (`grep` sans sortie), la valeur réelle étant **57** à `:88-89` ; et le décompte de migrations, recompté à **57** — sur les **deux** sites qui portent le total (l'en-tête `## Table d'audit (N migrations)` et la ligne `Total`), la somme des **trois** compteurs de partition valant ce total sans qu'aucun ne l'égale individuellement. *(Une rédaction antérieure disait « recompté aux trois sites », ce qui rejouait mot pour mot la règle fausse que la § P5 de `CLAUDE.md` raconte avoir dû corriger deux fois. Relevé en passe 1 de `validate`.)* La discipline de la § « Propagation post-patch » s'applique aux rapports d'agents comme au reste : une ancre fausse coûte une passe de revue entière.

**Statut : `ready-for-dev`, non validée.** La boucle `bmad-create-story validate` n'a pas encore tourné.

### Passe 1 de `bmad-create-story validate`

**2026-08-03 — Sonnet, trois lentilles (BlindHunter aveugle, EdgeCaseHunter avec accès dépôt, AcceptanceAuditor contre #144 et `CLAUDE.md`), contexte frais. 12 findings : 1 CRITICAL, 3 HIGH, 4 MEDIUM, 4 LOW.** Modèle différent d'Opus, qui a rédigé la spec.

**Le CRITICAL est une rupture entre un AC et sa tâche.** AC2 porte l'avertissement le plus sévère du document — deux helpers écrits à la main, dont `product_snapshot_json` dont l'oubli produit « une entrée d'audit qui ment ». Or la tâche T2, la checklist que le développeur suit réellement, ne citait que `is_no_op_change`. Un dev travaillant tâche par tâche pouvait cocher T2 en ayant reproduit exactement le défaut que l'AC décrit. T2 porte désormais les deux helpers, et le troisième site de construction de `NewProduct` (`kf005_fulltext_index_e2e.rs:123`) qu'aucune tâche ne nommait.

**Le HIGH le plus utile a tué une décision : D3 bloquait l'édition d'un champ que l'utilisateur ne touche pas.** La rédaction d'origine imposait les quatre critères de validité des lignes, `postable` compris, à l'enregistrement de la fiche produit. Scénario : un administrateur rend non imputable, par un flux sans rapport, un compte qu'un article référence ; l'utilisateur renomme l'article ; `is_no_op_change` rend `false`, l'`UPDATE` part, et la validation **rejette en 400 à cause du compte**. L'EdgeCaseHunter a établi que le mécanisme jumeau ne fait justement **pas** cela — `grep -n "postable" crates/kesh-api/src/routes/company_invoice_settings.rs` ne rend **aucune ligne**. D3 est réécrite sur **trois** critères, `postable` exclu ; le signal reste où 16-1 l'a construit, au niveau de la ligne.

**Le deuxième HIGH : la § « Règle de splitting préventif » n'avait jamais été évaluée**, alors que c'est un contrôle à faire **avant** `bmad-create-story`. Recompté à la granularité de 16-1a/16-1b : **sept** modules, seuil franchi. Une section dédiée pose les deux lectures et **attend l'arbitrage du Project Lead** ; elle tient lieu de dérogation d'ici là.

**Le troisième HIGH est une auto-ironie utile.** Le Change Log affirmait avoir « recompté le décompte de migrations aux **trois** sites du document d'audit » — formulation qui rejoue **mot pour mot** la règle fausse que la § P5 de `CLAUDE.md` raconte avoir dû corriger deux fois. Le total vit sur **deux** sites ; les **trois** compteurs de partition *somment* à ce total sans qu'aucun ne l'égale. Corrigé, et le paragraphe dit désormais pourquoi.

**Deux décomptes faux, convergés par deux lentilles indépendantes** : `product_snapshot_json` énumère **huit** clés et non neuf ; et « 11 FK vers `accounts` » était le chiffre **historique** figé dans un commentaire de migration — recompté, il en existe **13**. Les deux relevés sont exactement la classe d'erreur que la spec elle-même dénonçait dans son Change Log.

**Trois ancres décalées d'une ligne** en fin de bloc (`addFreeLine` 407-**418**, matérialisation 1672-**1727**, `serialize_products_csv` 364-**397**) : aucune ne mène au mauvais endroit, mais le motif était systématique — la plage excluait à chaque fois la ligne de clôture.

**Les MEDIUM restants, tous appliqués** : AC9 annonçait « trois obligations » et en listait quatre (P5, P6, P7, **P1/P2**) ; AC9/P6 exigeait un arbitrage sans fournir de critère, désormais tranché par **D8** — la frontière reste à 34, donc `total - 24`, faute de quoi la fenêtre d'upgrade rétrécirait d'un cran à chaque migration ; AC8 énonçait la règle de préfixe i18n sans sa réserve, qui ne vivait qu'en Dev Notes ; T7 était étiquetée `AC11` alors qu'AC11 ne parle que du gate, de sorte que **l'obligation CHANGELOG n'était rattachée à aucun critère cochable** — d'où **AC12**, qui reprend aussi la vérification README que la story sœur 16-1b portait en AC18 ; et la mécanique du rebase après un **squash-merge** de #284, que la spec prescrivait sans dire comment (un `git rebase main` naïf rejouerait 53 commits déjà squashés).

**Ce que les trois lentilles ont validé positivement** : les six critères d'acceptation de #144 sont couverts, l'exclusion du compte de charge (**D6**) est jugée solide et non un rétrécissement non autorisé, le classement **P7** en DDL pur est exact contre le détecteur réellement implémenté, **P1/P3** confirme l'absence de bump `min_required`, la reprise de l'avertissement P5 sur les compteurs de partition est fidèle **sans l'inverser**, l'ancre P6 `assert_eq!(total, 57)` est exacte, les citations des décisions des stories sœurs (D3-bis, « afficher + signaler + bloquer ») sont exactes mot pour mot, et le CHANGELOG ne porte aujourd'hui aucun avertissement `PUT` pour les produits — la demande est donc un complément, pas une duplication.

**Trend** : `1 CRIT / 3 HIGH / 4 MED / 4 LOW`. **Boucle NON convergée** — une passe 2 est requise, avec un modèle différent (Haiku) et un contexte frais.

### Passe 2 de `bmad-create-story validate`

**2026-08-03 — Haiku 4.5, trois lentilles, contexte frais. 1 HIGH, 3 MEDIUM retenus ; 8 findings ÉCARTÉS pour erreur de catégorie.** Modèle différent de Sonnet (passe 1) et d'Opus (rédaction).

**La passe a produit un faux positif massif, et il est instructif.** L'EdgeCaseHunter a rendu 13 findings dont **huit** disent, sous des formes variées, que « le champ n'existe pas dans les DTO / les entités / `COLUMNS` / l'export CSV / les types TypeScript / le formulaire », et que « `onProductSelect` pose encore `null` ». Il a même rangé cela en **CRITICAL** sous le titre « rupture de chaîne ».

**C'est l'objet même de la story.** Une spécification décrit un travail à faire ; auditer le code *avant* implémentation et signaler l'absence du champ revient à reprocher au plan de la maison de ne pas être la maison. Les huit sont écartés en bloc. C'est une variante du mode d'échec que la § « Haiku-specific guardrails » de `CLAUDE.md` décrit — non pas une erreur d'indexation de diff cette fois, mais une confusion entre l'**état présent** et l'**état cible**.

**Le « CRITICAL » est réfuté en ground-truth.** Il soutenait que la chaîne `ProductPicker` → `onProductSelect` ne pourrait pas transporter le champ. Vérification : `ProductPicker.svelte:19` déclare `onSelect: (p: ProductResponse) => void` et `:59` passe l'objet **complet** ; `listProducts` rend des `ProductResponse` pleins. La chaîne est intacte dès que `ProductResponse` porte le champ — ce qu'AC3 exige. Écarté.

**Le HIGH retenu est réel et vient du BlindHunter** : le titre de **D3** annonçait « TROIS critères » quand l'énumération s'en lisait quatre. Les deux se réconcilient — « exister » et « appartenir à la société » ne font qu'un critère, sanctionné par une **seule** variante de rejet, `UnknownOrCrossCompany`, précisément pour rendre un compte d'une autre société indiscernable d'un compte inexistant (garde anti-IDOR). D3 énumère désormais ses trois critères en liste numérotée, et dit pourquoi le premier est composite.

**Le MEDIUM le plus utile est une convergence de deux lentilles, et il porte sur la preuve.** AC2 désigne `is_no_op_change` et `product_snapshot_json` comme les deux défauts les plus graves de la story — l'un fait répondre succès sans rien écrire, l'autre fait mentir la piste d'audit. Or **aucune des trois mutations d'AC10 ne les discriminait**. Une spec qui nomme un risque sans le faire prouver ne l'a pas traité : AC10 porte désormais **cinq** mutations, les deux nouvelles ciblant chacune un helper.

**Deux MEDIUM de précision** : « site 3/5 » était cité sans que le document dise jamais ce que sont les cinq sites — il s'agit des cinq constructions de `LineState` numérotées par 16-1b, dont **deux seulement** concernent cette story ; et les trois compteurs de partition du document d'audit n'étaient pas nommés (`yes`, `tracked-by-sqlx`, `no`), ce qui rendait l'instruction de recomptage inexécutable telle quelle.

**LOW appliqués** : la référence au SHA du point de fork était circulaire (« le `0ce6e13a` inscrit ici même ») — il est désormais posé en clair, avec la raison de le relever **avant** le squash-merge ; et T6 nomme les tests attendus plutôt que de les laisser implicites, dont le scénario de scoping multi-tenant qu'aucune tâche n'exerçait.

**Ce que les lentilles ont validé positivement, avec commandes** : les **six** corrections de la passe 1 sont vérifiées exactes une par une — `grep postable` sur `company_invoice_settings.rs` sans sortie, l'arithmétique de **D8** (`57-23 = 58-24 = 34`), les **huit** clés et **quatre** champs des deux helpers, les **treize** FK, la construction littérale de `NewProduct` dans `kf005_fulltext_index_e2e.rs` sans `Default`, et le fait que le dépôt merge bien en **squash** (PR #275 → un seul commit `dc3ece04`). La structure du document est jugée complète face à ses cinq stories sœurs, la dérogation au splitting conforme à ce que `CLAUDE.md` exige, les **six** critères de #144 tous couverts, et le scoping multi-tenant assuré par l'enum réutilisée.

**Trend** : `1 CRIT / 3 HIGH / 4 MED / 4 LOW` → **`0 CRIT / 1 HIGH / 3 MED / 2 LOW`**. Rotation Sonnet → Haiku.

⚠️ **Contrôle du garde-fou de la dérogation** : la sévérité maximale **décroît** (`CRITICAL` → `HIGH`) et le volume aussi. Le second critère de la § « Règle de splitting préventif » — sévérité `N+1` **≥** `N` — **ne se déclenche pas**. La dérogation tient.

**Boucle NON convergée** — 1 HIGH et 3 MEDIUM subsistaient avant patch, donc une **passe 3** est requise, avec un troisième modèle (Opus) et un contexte frais.
