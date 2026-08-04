# Story 16.2a : Compte de produit sur la fiche produit — socle backend

## Status

ready-for-dev

## Story

**As a** mainteneur de Kesh,
**I want** que la fiche produit puisse porter **son** compte de produit, persisté et validé,
**so that** la surface utilisateur livrée par **16-2b** ait une donnée à lire — et que la colonne naisse avec ses garde-fous de migration plutôt qu'après coup.

Issue : **#144**. Sous-story de l'Epic 16, cible **v0.9.0**. **Née du split de 16-2** (4 passes de `validate`, cf. `16-2-compte-produit-catalogue.md` pour l'historique).

⚠️ **Doit partir dans la MÊME PR que 16-2b.** Seule, cette story livre une colonne que **rien ne lit** — le « code mort qui paraît fonctionner » que **D6** invoque par ailleurs. C'est l'objection qui a fait refuser le split deux fois ; elle est levée par la contrainte de PR, pas par le split.

**Dépend de 16-1** — dépendance **levée le 2026-08-04** : la PR #284 est mergée en squash (`b499eee4`), et la branche a été rebasée sur ce `main`. Les ancres de 16-1 sont désormais vérifiables sur `main`. Cf. Dev Notes pour la trace de la mécanique.

---

## Contexte

Trois faits, tous établis par relevé et **vérifiés à quatre reprises** par les passes de `validate` de la story parente.

1. **Aucun `product_id` n'existe sur les tables de lignes** — `grep -rn "product_id" crates/kesh-db/migrations/*.sql` → aucune sortie. Le lien catalogue → facture est une **recopie** au moment du choix, pas une référence (cf. **D1**), et il vit entièrement en **16-2b**.
2. **Le patron « compte par défaut » existe au niveau société** : `company_invoice_settings.default_revenue_account_id BIGINT NULL`, FK `ON DELETE RESTRICT` (`20260417000001_invoice_validation.sql:39-47`). Convention unanime — **13** FK vers `accounts` (`grep -rn "REFERENCES accounts" crates/kesh-db/migrations/*.sql | wc -l`). *(Le commentaire de `20260727000001:21-23` annonce « 11 » : exact quand 16-1a l'a écrit. Recompter, ne pas relire.)*
3. **Le réglage société ne valide PAS `postable`** — `grep -n "postable" crates/kesh-api/src/routes/company_invoice_settings.rs` → aucune sortie. C'est le patron dont **D3** se réclame.

---

## Décisions

- **D1 — Aucun `product_id`, aucune relation persistante.** Rappel de cadrage ; le geste de recopie est en 16-2b. Introduire `product_id` serait une story distincte et non demandée.

- **D2 — `products.default_revenue_account_id BIGINT NULL`**, FK `ON DELETE RESTRICT`, **index nommé dédié**. `NULL` = « cet article n'impose rien », la ligne suit alors le défaut société.

  ⚠️ **Pas un « miroir strict » du réglage société** : `company_invoice_settings` ne déclare **aucun index sur `default_revenue_account_id`** — vérifiable au DDL (`20260417000001_invoice_validation.sql:39-47` : la colonne et sa FK `fk_cis_revenue`, sans `CREATE INDEX` associé). ⚠️ **Ne pas invoquer ici `grep -nF "idx_company_invoice_settings"`** : il rend **une** sortie — `20260419000003:11`, un index sur `created_at` — donc une commande qui *paraît* réfuter la phrase qu'elle accompagne.

  **Pourquoi déclarer l'index alors, si le jumeau n'en a pas ?** Non parce qu'InnoDB l'exigerait de nous : InnoDB **crée lui-même** l'index nécessaire à une FK quand aucun ne convient — c'est exactement pourquoi le réglage société fonctionne sans en déclarer. Le motif est **le patron de 16-1a** (`idx_invoice_lines_revenue_account`, `20260727000001`) : un index nommé explicitement est inspectable, porte une convention de nommage stable, et ne dépend pas d'un choix implicite du moteur. Aucun besoin de requête ne le motive — rien ne filtre `products` par compte.

- **D3 — Validation sur TROIS critères, `postable` EXCLU.** À l'enregistrement :
  1. **exister et appartenir à la société** — un seul critère, une seule variante de rejet `RevenueAccountRejection::UnknownOrCrossCompany`, qui rend un compte d'une autre société indiscernable d'un compte inexistant (garde anti-IDOR) ;
  2. `active` ;
  3. `account_type = Revenue`.

  **`postable` est écarté** parce que le code jumeau l'écarte : `company_invoice_settings::validate_account` (`routes/company_invoice_settings.rs:94-123`) contrôle l'existence, la société, `active` et `account_type` — **jamais** `postable`. Le sanctionner ici bloquerait l'édition d'un article sur un champ non touché.

  ⚠️ **`validate_account` est invoquée pour ce seul point, et pour rien d'autre.** Ce n'est **pas** le patron de la forme de l'erreur : elle rend des chaînes françaises **en dur** (`AppError::Validation(format!("{field_label} : compte introuvable"))`), sans i18n ni code applicatif — à l'opposé de ce qu'exigent **D10** et **AC-A3**. Copier sa mécanique de message serait une régression.

  📎 **`D3-bis` (story 16-1a), rappelée pour que cette spec se lise seule** : à la validation d'une facture, `postable` est **exempté** pour le compte égal au `default_revenue_account_id` de la société. C'est une clause de **grand-père** protégeant un réglage préexistant — pas une autorisation générale, d'où son non-report ici.

- **D4 — La validation ne se déclenche QUE si le compte CHANGE.** Sur l'`update`, **D3** ne s'applique que si `changes.default_revenue_account_id != before.default_revenue_account_id`. Une valeur **inchangée** passe, quel que soit l'état devenu du compte.

  ⚠️ **Sans cette décision, D3 ne fermait le piège qu'à moitié.** Le raisonnement qui écarte `postable` vaut **mot pour mot** pour `active` : archiver un compte est **bien plus fréquent** que basculer `postable`, et rien n'inspecte les référents à l'archivage. L'utilisateur renomme un article, `is_no_op_change` rend `false`, l'`UPDATE` part, et la validation rejette sur un champ qu'il n'a pas touché. Pire : un compte archivé est **absent des propositions** du sélecteur, donc l'utilisateur ne peut **ni conserver ni remplacer** la valeur.

  **Placement — tranché, parce que la question n'est pas neutre.** La comparaison exige `before`, et `update_product` (`routes/products.rs:295-322`) n'en lit **aucun** aujourd'hui : le seul existe **dans la transaction** de `products::update` (`repositories/products.rs:275-298`). **La validation reste à la route**, avec un `products::find_by_id(&state.pool, company.id, id)` supplémentaire **avant** l'appel au repository (signature à `repositories/products.rs:197-200` — `company_id` **avant** `id`, scopé anti-IDOR) — patron déjà accepté dans le dépôt : `company_invoice_settings.rs:151` fetche son état courant à la route, hors transaction. Le verrou optimiste couvre la fenêtre : une modification concurrente du **produit** fait diverger `version` et rend `OptimisticLockConflict` ; une modification concurrente du **compte référencé** ne touche pas `products.version` — et c'est précisément le cas que D4 veut laisser passer.

  ⚠️ **Ne PAS déplacer la validation dans le repository** sous prétexte que `before` y est gratuit : les erreurs remonteraient en `DbError` au lieu d'`AppError::Validation`, et les tests de route deviendraient inopérants.

- **D5 — Le `PUT` reste full-replace, et cette story en aggrave la portée.** `products::update` écrit ses colonnes métier inconditionnellement (`:310-313`) ; `ProductUpdate` n'a aucun champ sentinelle. Un client d'API qui relit puis réécrit **sans** la clé `defaultRevenueAccountId` **efface le compte**, sans erreur. C'est le CR **#278**, différé par arbitrage. Obligations : l'avertissement au CHANGELOG (**16-2b**, AC-B7) ; aucune tentative de sémantique « conserver » ici.

- **D6 — Le compte de charge (achat) est HORS PÉRIMÈTRE.** #144 le pose « optionnel / à discuter ». Aucun chemin n'existe entre le catalogue et les factures fournisseur : `supplier_invoice_lines.expense_account_id` est une FK **obligatoire** saisie à la main (`20260628000001:79-96`), et aucun sélecteur d'article n'existe sur ce formulaire. Livrer le champ sans son chemin produirait une colonne que rien ne lit.

- **D7 — Aucune reprise rétroactive.** Arbitrage de kickoff. Migration en **DDL pur** — aucun `UPDATE`, aucun `INSERT`. Les articles existants naissent à `NULL`, comportement actuel strictement préservé.

- **D8 — La frontière de la fenêtre d'upgrade RESTE à 34 : bumper `total` ET la fenêtre.** La migration casse volontairement `upgrade_path_preserves_data` (`crates/kesh-db/tests/migrations_upgrade_path.rs:88-92`, assertion à `:89`) : `assert_eq!(total, 57)` passe à **58**, et la fenêtre `total - 23` à **`total - 24`**. Laisser `- 23` déplacerait la frontière à 35 et la ferait rétrécir d'un cran à **chaque** migration future — la version lente du mode d'échec que **P6** existe pour attraper.

- **D9 — Les écritures de journal sont HORS PÉRIMÈTRE.** #144 demande le pré-remplissage « sur une facture **/ écriture** ». `JournalEntryForm.svelte` ne porte **aucun** sélecteur d'article — `ProductPicker` n'est importé que par `InvoiceForm` (`grep -rn "ProductPicker" frontend/src`). Il n'existe donc aucun geste à pré-remplir. À rouvrir si un sélecteur d'article arrive un jour sur la saisie d'écriture.

- **D10 — La forme de l'erreur : une variante `AppError` DÉDIÉE.** *(Arbitrage de Guy, passe 1 de `validate`.)*

  `AppError::ProductRevenueAccountInvalid(RevenueAccountRejection)`, avec son bras dans `IntoResponse` rendant `build_response(StatusCode::BAD_REQUEST, "PRODUCT_REVENUE_ACCOUNT_INVALID", &msg)`.

  ⚠️ **Sans cette décision, AC-A3 était insatisfiable — les deux moitiés se contredisaient.** `AppError::Validation(msg)` rend `build_response(400, **"VALIDATION_ERROR"**, &msg)` (`crates/kesh-api/src/errors.rs:956-958`) : le code est **figé**, il n'y a aucun endroit où glisser `PRODUCT_REVENUE_ACCOUNT_INVALID`. Et le seul chemin du dépôt produisant un code spécifique sur ce sujet est le mapping de `DbError::InvalidRevenueAccounts` (`errors.rs:2250-2258`, code déclaré à `kesh-db/src/errors.rs:254`) — c'est-à-dire précisément la voie `DbError` que **D4** interdit. Exiger la route **et** un code applicatif imposait donc une troisième voie ; elle n'existait pas, il faut la créer.

  **Cette même décision règle le « troisième sujet ».** Le formateur `format_rejected_revenue_accounts` choisit son sujet par `match r.line_number { Some(n) => …, None => … }` sur `RejectedRevenueAccount.line_number: Option<i32>` (`kesh-db/src/errors.rs:41`) : **un `Option` n'admet pas de troisième cas**. Ajouter une clé Fluent n'y suffit pas — il faudrait élargir la struct, donc reprendre trois sites de construction (`invoices.rs:552`, `:573`, `credit_notes.rs:461`) et deux sites de test (`errors.rs:2615`, `invoices_line_revenue_account.rs:190`), **tous livrés et déjà mergés sur `main`**. La variante dédiée porte son propre message : `RejectedRevenueAccount` reste **intacte**, le formateur de 16-1a aussi.

  **Ce qui est réutilisé, ce qui ne l'est pas.** Réutilisé : l'enum `RevenueAccountRejection` (`kesh-db/src/errors.rs:14-27`) pour le **motif** — jamais dupliquée, jamais réénumérée. Non réutilisés : `RejectedRevenueAccount`, `format_rejected_revenue_accounts`, et la famille de clés `invoice-line-account-subject-*` — qui nomment des **lignes de facture**, pas des articles. La nouvelle clé vit dans sa propre famille (`product-revenue-account-*`), et le sujet y est écrit en clair.

---

## Acceptance Criteria

- **AC-A1 — Schéma** (D2). Migration ajoutant `products.default_revenue_account_id BIGINT NULL`, `fk_products_default_revenue_account` vers `accounts(id)` `ON DELETE RESTRICT`, et `INDEX idx_products_default_revenue_account`. **DDL pur** — aucun statement d'écriture de données.

- **AC-A2 — Entité et repository.** `Product`, `NewProduct`, `ProductUpdate` portent `default_revenue_account_id: Option<i64>`.

  ⚠️ **DEUX listes de colonnes écrites à la main, qui ne dérivent pas l'une de l'autre** : la constante `COLUMNS` (`repositories/products.rs:22-23`) **et** `FIND_BY_ID_SCOPED_SQL` (`:26`), seconde chaîne littérale indépendante alimentant **six** `query_as::<_, Product>` (`:168`, `:202`, `:275`, `:332`, `:374`, `:416`). Les deux l'incluent, ainsi que l'`INSERT` (`:146`) et l'`UPDATE` (`:311`). `Product` dérive `sqlx::FromRow` : un champ absent d'un `SELECT` produit un `ColumnNotFound` **à l'exécution** — donc **toutes** les routes produits en 500.

  ⚠️ **DEUX helpers écrits à la main, chacun avec son défaut SILENCIEUX.** Ni l'un ni l'autre ne dérive de la struct — le compilateur ne dira rien.
  - `is_no_op_change` (`:256-261`) compare **quatre** champs. Non étendu, une modification portant **uniquement** sur le compte est prise pour un no-op : `UPDATE` court-circuité, `version` inchangée, **aucun audit — et l'API répond succès**.
  - `product_snapshot_json` (`:32-43`) énumère **huit** clés. Non étendu, l'audit `{before, after}` de `product.updated` rend deux objets **identiques** : une entrée qui **ment**, ce qui est pire qu'une entrée absente.

- **AC-A3 — API et validation** (D3, D4, D10). `CreateProductRequest` et `UpdateProductRequest` acceptent `defaultRevenueAccountId` (camelCase, `Option<i64>`, `#[serde(default)]`) ; `ProductResponse` le restitue. La validation applique **D3** — trois critères — et **ne se déclenche, sur l'`update`, que si le compte change** (**D4**, avec son placement à la route). Un compte invalide rend **400** par la variante dédiée de **D10** — code applicatif **`PRODUCT_REVENUE_ACCOUNT_INVALID`**, motif issu de `RevenueAccountRejection` (`crates/kesh-db/src/errors.rs:14-27`), enum **réutilisée**, jamais dupliquée.

  ⚠️ **Le message doit désigner L'ARTICLE.** Le formateur existant (`crates/kesh-api/src/errors.rs:71-86`) ne connaît que deux sujets — la **ligne de facture** (`Some(n)`) et **« le compte de produit par défaut de la société »** (`None`) — et ne peut pas en accueillir un troisième (**D10**). Le réutiliser ferait lire, à qui édite un article, *« le compte de produit par défaut de la société : le compte 3400 est archivé »* : un message qui désigne **un autre objet** et envoie corriger un réglage non touché.

  **Concrètement** : une famille de clés **propre**, `product-revenue-account-*`, avec son fallback Rust **dans `crates/kesh-api/src/errors.rs`** (au bras `IntoResponse` de la variante) et ses entrées Fluent **dans `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`** — les **4** locales. ⚠️ **Ne pas étendre `invoice-line-account-subject-*`** : ces clés sont consommées par le formateur de 16-1a, qu'on ne touche pas.

  ⚠️ **`#[serde(default)]` n'est pas cosmétique** : sans lui, un `Option<T>` reste **obligatoire dans le JSON** en serde, et l'omission de la clé casse **toute intégration API existante**, y compris celles qui ignorent le nouveau champ.

- **AC-A4 — Export CSV.** `serialize_products_csv` (`crates/kesh-api/src/exports/csv_tables.rs:364-397`) inclut la colonne, à sa position dans `COLUMNS`. ⚠️ **Deux listes écrites à la main que rien ne relie** — l'en-tête (`:368-379`) et l'enregistrement (`:382-393`) : un décalage entre elles rend le CSV **silencieusement faux sur toutes les colonnes suivantes**.

- **AC-A5 — Garde-fous de migration**, les **quatre**, chacun **recompté à la source** :
  - **P5** — `docs/migrations-idempotence-audit.md` porte une ligne pour la migration ; l'en-tête `## Table d'audit (N migrations)` **et** la ligne `Total` passent de **57** à **58** ; la somme des trois compteurs de partition — `yes`, `tracked-by-sqlx`, `no` — égale ce total. ⚠️ Les compteurs de partition **ne valent pas** le total ; les aligner dessus casserait l'invariant.

    **Verdict d'idempotence : `tracked-by-sqlx`** — donc le compteur qui bouge est `tracked-by-sqlx` (**52 → 53**), `yes` restant à 5 et `no` à 0. Ce n'est pas un choix à refaire : `20260727000001` est le **jumeau littéral** de cette migration (`ADD COLUMN` + FK `ON DELETE RESTRICT` + `CREATE INDEX`, sans `IF NOT EXISTS`) et porte déjà ce verdict, avec ses codes d'erreur MariaDB — **1060** sur le `ADD COLUMN` rejoué, **1061** sur le `CREATE INDEX`. La ligne les cite, comme toutes ses voisines. *(Le verdict `yes` est réservé aux migrations rejouables sans erreur — cf. l'arbitrage du 2026-07-29 sur `20260729000001`, seule du tableau dans ce cas.)*
  - **P6** — `grep -rn "migrations.len()\|apply_migrations_up_to" crates/`, inspecter **chaque** site ; appliquer **D8** : `total` → 58 **ET** fenêtre → `total - 24`.
  - **P7** — DDL pur, donc ni entrée au registre ni exemption. ⚠️ **Le constater en exécutant `every_data_backfill_migration_is_triaged`**, pas le supposer.
  - **P1/P2** — `ADD COLUMN` nullable ⇒ non-breaking ⇒ **aucun** bump de `kesh_version_min_required`, donc aucun bump Cargo.

- **AC-A6 — Discrimination prouvée par mutation.** **Cinq** mutations exécutées et consignées au Dev Agent Record, chacune tuant le test visé **et lui seul** :
  1. validation **D3** retirée → le test du compte invalide (au niveau **route**) rougit ;
  2. condition de **D4** retirée (validation inconditionnelle) → le test « renommer un article dont le compte a été archivé » rougit ;
  3. champ retiré de **`is_no_op_change`** → le test « modifier le seul compte bumpe la `version` » rougit ;
  4. champ retiré de **`product_snapshot_json`** → le test « l'audit montre `before ≠ after` » rougit ;
  5. variante de **D10** remplacée par `AppError::Validation(…)` → le test qui asserte **le code `PRODUCT_REVENUE_ACCOUNT_INVALID` et le sujet « l'article »** rougit. ⚠️ **Sans cette mutation, D10 n'est discriminée par rien** : le code retomberait à `VALIDATION_ERROR` et le sujet au message générique, sans qu'aucun test ne bouge — c'est le mode d'échec que la parente avait relevé sur le troisième sujet et laissé ouvert.

  **Un test attendu qui ne rougit pas invalide le montage** : le corriger avant d'aller plus loin.

- **AC-A7 — Gate complet.** `scripts/test-fast.sh` (fmt + clippy `-D warnings` + nextest workspace) sur l'**état final**, exit 0, **verdict lu dans le log**. ⚠️ Cette story touche `crates/kesh-db/migrations/` et un repository : la § « Test Locally First » → « Pendant une boucle de revue » **interdit le gate ciblé**, y compris entre deux passes.

---

## Tasks / Subtasks

- [ ] **T-A1 — Migration et garde-fous** (AC-A1, AC-A5)
  - [ ] `crates/kesh-db/migrations/<timestamp>_products_default_revenue_account.sql` — `ADD COLUMN` + FK + index, DDL pur.
  - [ ] Ligne au tableau de `docs/migrations-idempotence-audit.md`, verdict **`tracked-by-sqlx`** avec ses codes **1060 / 1061** (AC-A5) ; compteurs **recomptés** : 57 → 58 aux **deux** sites du total, et `tracked-by-sqlx` 52 → 53.
  - [ ] `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` ; `assert_eq!(total, 57)` → **58** **ET** fenêtre `total - 23` → **`total - 24`** (D8). ⚠️ Bumper `total` seul déplace la frontière à 35, **silencieusement**.
  - [ ] Exécuter `every_data_backfill_migration_is_triaged` pour **constater** que le DDL pur ne déclenche rien.
- [ ] **T-A2 — Entité et repository** (AC-A2)
  - [ ] Champ dans `Product` / `NewProduct` / `ProductUpdate`.
  - [ ] **Les DEUX listes de colonnes** : `COLUMNS` **et** `FIND_BY_ID_SCOPED_SQL`, plus `INSERT` et `UPDATE`.
  - [ ] **Les DEUX helpers** : `is_no_op_change` **et** `product_snapshot_json`.
  - [ ] Sites de construction en littéral — `NewProduct` / `ProductUpdate` ne dérivent **pas** `Default`, donc **la compilation casse** à chacun. Échec bruyant : `cargo build` les énumère tous, cette liste n'a pas à être exhaustive pour être sûre. Le seul **hors périmètre thématique** est `crates/kesh-db/tests/kf005_fulltext_index_e2e.rs:123` ; les autres sont dans les fichiers déjà touchés (`repositories/products.rs:504`, `:632`, `:651`, `:686`, et les deux de `routes/products.rs`).
- [ ] **T-A3 — API, validation, message** (AC-A3)
  - [ ] DTO + `ProductResponse`, contrat camelCase, `#[serde(default)]`.
  - [ ] Validation **D3** à la route, **conditionnée par D4** — ajouter le `products::find_by_id(&state.pool, company.id, id)` avant l'appel au repository, sur le patron de `company_invoice_settings.rs:151`.
  - [ ] **Variante d'erreur dédiée (D10)** : `AppError::ProductRevenueAccountInvalid(RevenueAccountRejection)` + son bras `IntoResponse` rendant le code **`PRODUCT_REVENUE_ACCOUNT_INVALID`**. ⚠️ **Ne pas** passer par `AppError::Validation` (code figé à `VALIDATION_ERROR`), **ni** par `DbError` (interdit par D4).
  - [ ] **Message désignant l'article** : famille **`product-revenue-account-*`** — fallback Rust au bras `IntoResponse`, entrées Fluent dans les **4** locales de `crates/kesh-i18n/locales/*/messages.ftl`. ⚠️ **Ne pas** étendre `invoice-line-account-subject-*` ni toucher `format_rejected_revenue_accounts`.
- [ ] **T-A4 — Export CSV** (AC-A4)
  - [ ] Étendre `serialize_products_csv`, en-tête **et** valeurs.
  - [ ] **Renforcer le BON test d'export** — `export_global_zip_includes_archived_products` (`crates/kesh-api/tests/exports_global_e2e.rs:1271`). Il sème **déjà deux produits** par `INSERT` direct et n'asserte qu'un `rowCount == 2` (`:1296`) : le **contenu** n'est vérifié nulle part, et l'en-tête non plus (`grep -rnF "unit_price,vat_rate" crates/` → aucune sortie). Donner un compte à l'un des deux — sa liste de colonnes est écrite à la main (`:1273-1276`), l'y ajouter — puis asserter **en-tête et valeur**. *(Complément qu'exigeait 16-1a en AC14.)*

    ⚠️ **NE PAS semer de produit dans `export_global_zip_empty_company_explicit_row_count_map`** (`:854`), dont la fixture porte `("products.csv", 0)` à `:906`. Son nom énonce son invariant : y ajouter une ligne casse son assertion et contredit son contrat. C'est la fixture **vide** qui rend ce test discriminant — la confondre avec « le test des produits » coûte une assertion juste.
- [ ] **T-A5 — Tests et preuve** (AC-A3, AC-A6)
  - [ ] Intégration `kesh-db` : le champ persiste et se relit ; **modifier le SEUL compte bumpe la `version` et écrit l'audit** ; **l'audit montre `before ≠ after`** ; une modification sans changement reste un no-op.
  - [ ] **Au niveau ROUTE, pas repository** — la validation vit dans `routes/products.rs` et rend un **400** : rejet d'un compte inactif, d'un compte non-`Revenue`, et **scoping multi-tenant** (compte d'une autre société).
  - [ ] **D4** : modifier le nom d'un article dont le compte a été **archivé entre-temps** doit **réussir**.
  - [ ] **Non-régression du contrat HTTP** : payload **sans la clé**, et payload avec `"defaultRevenueAccountId": null` — les deux valent `NULL`, jamais 400.
  - [ ] **Forme de l'erreur (D10)** : le corps du 400 porte `error.code == "PRODUCT_REVENUE_ACCOUNT_INVALID"` — **asserter le code, pas seulement le statut** — et son message désigne **l'article**, jamais le réglage société.
  - [ ] Les **cinq** mutations d'AC-A6, exécutées, consignées avec leur sortie, fichiers restaurés à l'identique.
- [ ] **T-A6 — Gate** (AC-A7) — complet, état final, exit 0, verdict lu dans le log.

---

## Dev Notes

### Ce que cette story ne doit PAS faire

- **Ne pas ajouter `product_id`** (D1). Si l'implémentation semble l'exiger, le périmètre a dérivé.
- **Ne pas reprendre le parc** (D7) : aucune migration de données.
- **Ne pas toucher au compte de charge / achat** (D6).
- **Ne pas brancher les écritures de journal** (D9).
- **Ne pas toucher au frontend** — le sélecteur, le pré-remplissage et la doc-sync sont en **16-2b**.
- **Ne pas corriger le contrat `PUT`** (D5) — c'est #278, arbitré ailleurs.
- **Ne pas dupliquer** `RevenueAccountRejection`.
- **Ne pas toucher au formateur de 16-1a** (D10) — ni `format_rejected_revenue_accounts`, ni `RejectedRevenueAccount`, ni les clés `invoice-line-account-subject-*`. Ce code est livré et mergé ; la variante dédiée existe pour ne pas y revenir.
- **Ne pas rendre l'erreur par `AppError::Validation`** (D10) : son code est figé à `VALIDATION_ERROR`, l'AC-A3 serait fausse sans que rien ne rougisse.

### La branche partait de 16-1 — rebase **fait** le 2026-08-04

*(Section conservée pour sa trace : la manœuvre est exécutée, il n'y a plus rien à faire ici.)*

Branche issue de `story/16-1-compte-produit-par-ligne`, dont les ancres étaient invérifiables sur `main` tant que la **PR #284** n'était pas mergée. Elle l'a été **en squash** (`b499eee4`), comme prévu — un `git rebase main` naïf aurait rejoué les 53 commits de 16-1 contre un `main` qui les contient déjà. Forme employée :

```sh
git checkout main && git pull --ff-only          # ef6cdf52 → b499eee4
git rebase --onto main 0ce6e13a story/16-2-compte-produit-catalogue
```

`0ce6e13a` — le point de fork — avait été **relevé avant le merge**, un squash le rendant introuvable par `git merge-base`. Résultat : 7 commits rejoués, zéro conflit, `main..HEAD` ne contient bien que ces 7 commits. Les story files de 16-2 sont bit-à-bit inchangés (`git diff` de l'ancien au nouveau sommet, restreint à `16-2*` : vide) et la fusion de `sprint-status.yaml`, seul fichier écrit des deux côtés, a préservé les huit entrées.

### Conventions de test

`#[sqlx::test(migrator = "kesh_db::MIGRATOR")]`, fixtures `kesh_db::test_fixtures::{SeededCompany, seed_accounting_company}`. Les locales vivant dans un **crate Rust**, tout changement i18n impose `cargo test --workspace`.

⚠️ **Le preset E2E `with-data` crée déjà un produit** — `seed_contact_and_product` (`test_fixtures.rs:482`) insère `'CI Product'`, appelé par `routes/test_endpoints.rs:203`. Il naîtra au compte `NULL`. *(Une rédaction antérieure de la story parente affirmait le contraire ; réfuté en passe 4.)*

### References

- Story **16-2b** — surface utilisateur (sélecteur, pré-remplissage, i18n, doc-sync). **Même PR obligatoire.**
- Story parente **16-2** — `16-2-compte-produit-catalogue.md`, archivée : 4 passes de `validate` et leur historique.
- Story **16-1a** — `revenue_account_id`, règles de validité (`invoices.rs:514-590`), décision **D3-bis**, et le patron d'AC14 pour le test d'export.
- `CLAUDE.md` § « Migration breaking policy » et § « Test Locally First ».
- CR **#278** — durcissement du contrat des API en écriture, différé.

---

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

**2026-08-03 — Story née du split de 16-2**, arbitré par Guy après que le garde-fou de la dérogation se soit déclenché **deux fois** (passes 3 et 4 de `validate`, sévérité maximale `HIGH` maintenue). Le contenu repris est dans son **état corrigé de la passe 4** : il incorpore les findings des quatre passes, dont le placement de **D4** à la route (irréalisable là où la parente le plaçait), la seconde liste de colonnes `FIND_BY_ID_SCOPED_SQL`, le troisième sujet i18n du message de rejet, et le test d'export CSV qui n'observait rien.

**Ce que le split corrige, et ce qu'il ne corrige pas.** Il ne corrige aucun défaut de conception — les quatre passes ont validé la conception. Il réduit la **surface** sur laquelle une passe adversariale doit tenir un mental-model cohérent : la parente avait atteint ~460 lignes, cinq Change Logs et dix décisions, et sa comptabilité dérivait à chaque remédiation — huit des treize findings de la passe 4 portaient sur les décomptes des Change Logs eux-mêmes, pas sur la story.

**2026-08-04 — Passe 1 de `validate`** (Opus 5, trois lentilles jouées par l'orchestrateur, ground-truth systématique). **3 HIGH, 4 MEDIUM, 3 LOW**, tous remédiés. **La conception n'a pas bougé** — ce qui manquait était la **forme de l'erreur**, que les quatre passes de la parente n'avaient pas éprouvée.

**Les deux HIGH de fond ont la même racine, et une seule issue.** AC-A3 exigeait *à la fois* la validation à la route (D4 l'impose et interdit le repository) *et* le code `PRODUCT_REVENUE_ACCOUNT_INVALID`. Or `AppError::Validation` rend un code **figé** à `VALIDATION_ERROR` (`errors.rs:956-958`), et le seul chemin du dépôt produisant un code spécifique sur ce sujet passe par `DbError` — la voie que D4 interdit. **Les deux moitiés de l'AC se contredisaient**, et un dev les suivant à la lettre aurait livré `VALIDATION_ERROR` sans qu'aucun test ne bouge. Second HIGH, même racine : le « troisième sujet » était prescrit sans son mécanisme, alors que le formateur choisit par `match` sur un `Option<i32>` — **structure qui n'admet pas de troisième cas**. Arbitrage de Guy : **variante `AppError` dédiée** (nouvelle **D10**), qui règle les deux et ne touche à rien de ce que 16-1a a livré et mergé.

**Le troisième HIGH envoyait casser un test.** T-A4 prescrivait de semer un produit là où la fixture porte `("products.csv", 0)` — mais ce site appartient à `export_global_zip_empty_company_explicit_row_count_map`, **dont le nom énonce l'invariant**. Pendant ce temps, `export_global_zip_includes_archived_products` sème déjà deux produits et n'asserte qu'un `rowCount` : c'était lui, l'assertion faible à renforcer.

**Deux MEDIUM sur D2, tous deux hérités de la parente et aggravés par la réécriture.** La commande de preuve citée — `grep -nF "idx_company_invoice_settings"` — **rend une sortie** (`20260419000003:11`, un index sur `created_at`) : elle paraît réfuter la phrase qu'elle accompagne. La parente disait « aucun index **sur sa colonne** », formulation exacte, élargie ici à tort. Et la justification substituée est fausse : **InnoDB n'exige pas** l'index de l'auteur du DDL, il le **crée lui-même** — c'est précisément pourquoi le réglage société fonctionne sans en déclarer. Le motif réel, perdu à la réécriture, est le patron 16-1a.

Autres remédiations : verdict d'idempotence **tranché** (`tracked-by-sqlx`, 52 → 53) au lieu d'être laissé « à décider » alors que le jumeau littéral `20260727000001` l'avait déjà fixé ; ancre `company_invoice_settings.rs:148` → **`:151`** (`:148` est un commentaire) et `:94-121` → `:94-123` ; signature de `products::find_by_id` explicitée (`company_id` **avant** `id`) ; `validate_account` **bornée** à l'argument qu'elle sert, son style de message étant l'opposé de ce qu'exige AC-A3 ; énumération des sites littéraux rendue honnête (elle en laissait quatre, mode d'échec bruyant) ; **cinquième mutation** ajoutée, sans laquelle D10 n'était discriminée par rien.

**Vérifié sain, à ne pas refaire** : les compteurs de migrations sont **exacts** — 57 sur disque = 57 lignes de tableau = les deux sites du total, partition `5 + 52 + 0`, et `assert_eq!(total, 57)` ; l'arithmétique de D8 (57→58 **et** `-23`→`-24`) est juste. Les quinze autres sites `apply_migrations_up_to` passent par `migrations_before(version)`, **insensibles par construction** — D8 vise bien le seul site positionnel. Le **backup dérive ses colonnes d'`INFORMATION_SCHEMA`** (`backup.rs:114`) : rien à maintenir, et P7 (DDL pur, rien à rejouer) tient. Plus quatorze ancres confirmées au grep : 13 FK, zéro `product_id`, zéro `postable` dans le réglage société, le commentaire « 11 » périmé, les six `query_as` aux lignes annoncées, `is_no_op_change` à quatre champs, `product_snapshot_json` à huit clés, `NewProduct` sans `Default`, `ProductPicker` importé du seul `InvoiceForm`, `CI Product`, `expense_account_id NOT NULL`, et l'en-tête CSV observé nulle part.

**Statut : `ready-for-dev`. Boucle NON convergée** — 3 HIGH en passe 1 imposent une **passe 2** (§ *Review Iteration Rule*), sur un LLM autre qu'Opus et en contexte frais.
