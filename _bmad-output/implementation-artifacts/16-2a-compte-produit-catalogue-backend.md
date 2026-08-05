# Story 16.2a : Compte de produit sur la fiche produit — socle backend

## Status

review

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

  ⚠️ **Les trois critères sont CALQUÉS sur `validate_account` — elle n'est jamais APPELÉE.** Deux raisons, chacune suffisante : elle est **privée à son module** (`async fn`, sans `pub` — `grep -n "fn validate_account"` le montre, et aucun appel n'existe hors de son fichier), et son retour est un `AppError::Validation(format!("{field_label} : compte introuvable"))` — chaîne française **en dur**, sans i18n ni code applicatif, soit exactement ce que **D10** interdit. **Réimplémenter les trois contrôles dans `routes/products.rs`** ; la rendre `pub` pour la réutiliser serait un contresens, on hériterait du chemin d'erreur que D10 existe pour éviter.

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

  **Le motif machine-lisible : réutiliser `revenue_account_rejection_code`, ne jamais le réécrire.** Le corps porte, à côté de `code` et `message`, un `details.reason` alimenté par `revenue_account_rejection_code(reason)` (`kesh-api/src/errors.rs:53-60`) — le `match` à quatre branches qui rend `"UNKNOWN_OR_CROSS_COMPANY"`, `"INACTIVE"`, `"NOT_REVENUE"`, `"NOT_POSTABLE"`. Il est **déjà** ce que consomme l'erreur jumelle (`:2265`, `:2299`). ⚠️ **Ne pas en écrire un second** : un `match` recopié sur le même enum est précisément la duplication que la § *Code Quality Rules* interdit, et il divergerait au premier variant ajouté. *(Point relevé en passe 2 : D10 nommait l'enum sans nommer ce helper, ce qui laissait la porte ouverte à sa redérivation.)*

---

## Acceptance Criteria

- **AC-A1 — Schéma** (D2). Migration ajoutant `products.default_revenue_account_id BIGINT NULL`, `fk_products_default_revenue_account` vers `accounts(id)` `ON DELETE RESTRICT`, et `INDEX idx_products_default_revenue_account`. **DDL pur** — aucun statement d'écriture de données.

- **AC-A2 — Entité et repository.** `Product`, `NewProduct`, `ProductUpdate` portent `default_revenue_account_id: Option<i64>`.

  ⚠️ **DEUX listes de colonnes écrites à la main, qui ne dérivent pas l'une de l'autre** : la constante `COLUMNS` (`repositories/products.rs:22-23`) **et** `FIND_BY_ID_SCOPED_SQL` (`:26`), seconde chaîne littérale indépendante alimentant **six** `query_as::<_, Product>` (`:168`, `:202`, `:275`, `:332`, `:374`, `:416`). Les deux l'incluent, ainsi que l'`INSERT` (`:146`) et l'`UPDATE` (`:311`). `Product` dérive `sqlx::FromRow` : un champ absent d'un `SELECT` produit un `ColumnNotFound` **à l'exécution** — donc **toutes** les routes produits en 500.

  ⚠️ **DEUX helpers écrits à la main, chacun avec son défaut SILENCIEUX.** Ni l'un ni l'autre ne dérive de la struct — le compilateur ne dira rien.
  - `is_no_op_change` (`:256-261`) compare **quatre** champs. Non étendu, une modification portant **uniquement** sur le compte est prise pour un no-op : `UPDATE` court-circuité, `version` inchangée, **aucun audit — et l'API répond succès**.
  - `product_snapshot_json` (`:32-43`) énumère **huit** clés. Non étendu, l'audit `{before, after}` de `product.updated` **tait la seule chose qui a changé** : une modification portant sur le compte y est enregistrée sans qu'aucune des huit clés ne la reflète. L'entrée **ment par omission**, ce qui est pire qu'une entrée absente.

    ⚠️ **NE PAS écrire que les deux objets seraient « identiques » — ils ne le sont jamais.** La clé `"version"` (`:41`) fait partie des huit, l'`UPDATE` exécute `version = version + 1` (`:312`), et `after` est refetché **après** (`:332`). Donc `before != after` est **vrai à tout coup** dès qu'un `UPDATE` réel a lieu, que la colonne soit ou non au snapshot. **Conséquence directe sur la preuve** : une assertion d'inégalité *globale* ne discrimine rien et resterait verte sous la mutation 4 — cf. AC-A6.

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
  4. champ retiré de **`product_snapshot_json`** → le test de l'audit rougit. ⚠️ **Ce test doit asserter LA CLÉ, pas l'objet** : `before["defaultRevenueAccountId"] != after["defaultRevenueAccountId"]`, et la présence même de la clé dans les deux objets. Une assertion `before != after` **globale** serait verte sous cette mutation — `"version"` diffère toujours (cf. AC-A2) — donc ne prouverait rien tout en en ayant la forme ;
  5. variante de **D10** remplacée par `AppError::Validation(…)` → le test qui asserte **le code `PRODUCT_REVENUE_ACCOUNT_INVALID` et le sujet « l'article »** rougit. ⚠️ **Sans cette mutation, D10 n'est discriminée par rien** : le code retomberait à `VALIDATION_ERROR` et le sujet au message générique, sans qu'aucun test ne bouge — c'est le mode d'échec que la parente avait relevé sur le troisième sujet et laissé ouvert.

  **Un test attendu qui ne rougit pas invalide le montage** : le corriger avant d'aller plus loin.

- **AC-A7 — Gate complet.** `scripts/test-fast.sh` (fmt + clippy `-D warnings` + nextest workspace) sur l'**état final**, exit 0, **verdict lu dans le log**. ⚠️ Cette story touche `crates/kesh-db/migrations/` et un repository : la § « Test Locally First » → « Pendant une boucle de revue » **interdit le gate ciblé**, y compris entre deux passes.

---

## Tasks / Subtasks

- [x] **T-A1 — Migration et garde-fous** (AC-A1, AC-A5)
  - [x] `crates/kesh-db/migrations/<timestamp>_products_default_revenue_account.sql` — `ADD COLUMN` + FK + index, DDL pur.
  - [x] Ligne au tableau de `docs/migrations-idempotence-audit.md`, verdict **`tracked-by-sqlx`** avec ses codes **1060 / 1061** (AC-A5) ; compteurs **recomptés** : 57 → 58 aux **deux** sites du total, et `tracked-by-sqlx` 52 → 53.
  - [x] `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` ; `assert_eq!(total, 57)` → **58** **ET** fenêtre `total - 23` → **`total - 24`** (D8). ⚠️ Bumper `total` seul déplace la frontière à 35, **silencieusement**.
  - [x] Exécuter `every_data_backfill_migration_is_triaged` pour **constater** que le DDL pur ne déclenche rien.
- [x] **T-A2 — Entité et repository** (AC-A2)
  - [x] Champ dans `Product` / `NewProduct` / `ProductUpdate`.
  - [x] **Les DEUX listes de colonnes** : `COLUMNS` **et** `FIND_BY_ID_SCOPED_SQL`, plus `INSERT` et `UPDATE`.
  - [x] **Les DEUX helpers** : `is_no_op_change` **et** `product_snapshot_json`.
  - [x] Sites de construction en littéral — `NewProduct` / `ProductUpdate` ne dérivent **pas** `Default`, donc **la compilation casse** à chacun. **`cargo build` fait foi : c'est lui l'énumération, pas cette ligne.** Un seul site mérite d'être nommé, parce qu'il est **hors périmètre thématique** et que rien d'autre n'y renvoie : `crates/kesh-db/tests/kf005_fulltext_index_e2e.rs:123`. *(Passe 2 : une énumération partielle figurait ici — elle omettait `repositories/products.rs:753` et `:1150`, et comptait « deux sites » dans `routes/products.rs` là où il y en a un de chaque type. Lister à moitié donne l'impression d'avoir listé.)*
- [x] **T-A3 — API, validation, message** (AC-A3)
  - [x] DTO + `ProductResponse`, contrat camelCase, `#[serde(default)]` sur les **entrées**. En **sortie**, suivre le patron du fichier : `ProductResponse` ne porte **aucun** `skip_serializing_if` (`grep -c` → 0), donc le champ est **toujours présent**, à `null` quand il est absent — comme `description`. ⚠️ Ne pas l'omettre conditionnellement : ce serait le seul champ du DTO à se comporter ainsi.
  - [x] Validation **D3** à la route, **conditionnée par D4** — ajouter le `products::find_by_id(&state.pool, company.id, id)` avant l'appel au repository, sur le patron de `company_invoice_settings.rs:151`. Les trois critères sont **réimplémentés**, `validate_account` étant privée (D3). Si le `find_by_id` rend `None`, **court-circuiter en 404** plutôt que de poursuivre sans `before` — le repository rendrait de toute façon un 404, mais l'écrire ici évite un chemin implicite.
  - [x] **Variante d'erreur dédiée (D10)** : `AppError::ProductRevenueAccountInvalid(RevenueAccountRejection)` + son bras `IntoResponse` rendant le code **`PRODUCT_REVENUE_ACCOUNT_INVALID`** et un `details.reason` issu de **`revenue_account_rejection_code`** (`errors.rs:53-60`) — **réutilisé, jamais réécrit**. ⚠️ **Ne pas** passer par `AppError::Validation` (code figé à `VALIDATION_ERROR`), **ni** par `DbError` (interdit par D4).
  - [x] **Message désignant l'article** : famille **`product-revenue-account-*`** — fallback Rust au bras `IntoResponse`, entrées Fluent dans les **4** locales de `crates/kesh-i18n/locales/*/messages.ftl`. ⚠️ **Ne pas** étendre `invoice-line-account-subject-*` ni toucher `format_rejected_revenue_accounts`.
- [x] **T-A4 — Export CSV** (AC-A4)
  - [x] Étendre `serialize_products_csv`, en-tête **et** valeurs.
  - [x] **Renforcer le BON test d'export** — `export_global_zip_includes_archived_products` (`crates/kesh-api/tests/exports_global_e2e.rs:1271`). Il sème **déjà deux produits** par `INSERT` direct et n'asserte qu'un `rowCount == 2` (`:1296`) : le **contenu** n'est vérifié nulle part, et l'en-tête non plus (`grep -rnF "unit_price,vat_rate" crates/` → aucune sortie). Donner un compte à **l'un des deux seulement** — sa liste de colonnes est écrite à la main (`:1273-1276`), l'y ajouter — puis asserter **l'en-tête, la valeur du produit qui porte un compte, ET la cellule VIDE de celui qui n'en porte pas**. *(Complément qu'exigeait 16-1a en AC14.)*

    ⚠️ **La cellule vide n'est pas un détail décoratif : c'est l'assertion qui attrape le décalage de colonnes.** Un en-tête et une valeur pris sur la **même** ligne peuvent être décalés du même cran et concorder entre eux. `fmt_opt_i64` (`csv_tables.rs:108-110`) rend bien la chaîne vide sur `None` et sert déjà 13 colonnes — mais rien ne le vérifie pour celle-ci, et c'est la ligne à `NULL` qui révèle un décalage, pas celle qui est remplie.

    ⚠️ **NE PAS semer de produit dans `export_global_zip_empty_company_explicit_row_count_map`** (`:854`), dont la fixture porte `("products.csv", 0)` à `:906`. Son nom énonce son invariant : y ajouter une ligne casse son assertion et contredit son contrat. C'est la fixture **vide** qui rend ce test discriminant — la confondre avec « le test des produits » coûte une assertion juste.
- [x] **T-A5 — Tests et preuve** (AC-A3, AC-A6)
  - [x] Intégration `kesh-db` : le champ persiste et se relit ; une modification sans changement reste un no-op.
  - [x] **DEUX tests distincts, surtout pas un seul** — ils sont les cibles respectives des mutations 3 et 4, et fusionnés ils cesseraient de discriminer :
    - **(a) bump de `version`** — modifier le SEUL compte fait passer `version` de N à N+1 et écrit une entrée d'audit ;
    - **(b) contenu de l'audit** — `before["defaultRevenueAccountId"] != after["defaultRevenueAccountId"]`, les deux clés étant **présentes**. ⚠️ **Jamais `before != after` globalement** (AC-A2 : `"version"` diffère toujours).

    ⚠️ **Pourquoi deux tests.** Sous la mutation 3 — champ retiré d'`is_no_op_change` —, « modifier le seul compte » devient un **no-op** : `products::update` rollback et retourne avant d'écrire l'audit (`:300-308`, audit à `:339-343`). Un test unique portant les deux assertions rougirait donc sous les mutations **3 et 4**, ce qui contredirait « chacune tue le test visé **et lui seul** ». (b) doit exercer un `UPDATE` qui aboutit **quelle que soit** la mutation 3 — donc modifier **aussi** un autre champ.
  - [x] **Au niveau ROUTE, pas repository** — la validation vit dans `routes/products.rs` et rend un **400**, **à la création comme à la modification** : rejet d'un compte inactif, d'un compte non-`Revenue`, et **scoping multi-tenant** (compte d'une autre société).
  - [x] **D4, les DEUX sens** — la condition doit être exercée dans ses deux directions, faute de quoi une condition **inversée** passerait :
    - compte **inchangé** et devenu invalide (archivé entre-temps) → renommer l'article **réussit** ;
    - compte **changé** vers une valeur invalide (active mais non-`Revenue`, p. ex.) → **400**. ⚠️ La mutation 2 ne couvre que le premier sens : elle rend la validation *inconditionnelle*, elle ne teste pas l'inversion du prédicat.
  - [x] **Retirer un compte déjà posé** — `Some(n) → null` est un **changement** au sens de D4, mais il n'y a **rien à valider** : la garde doit sauter D3 quand la nouvelle valeur est `None`. Sans ce test, une implémentation qui valide dès que « le compte change » rendrait un compte **impossible à retirer une fois posé**.
  - [x] **Non-régression du contrat HTTP** : payload **sans la clé**, et payload avec `"defaultRevenueAccountId": null` — les deux valent `NULL`, jamais 400.
  - [x] **Forme de l'erreur (D10)** : le corps du 400 porte `error.code == "PRODUCT_REVENUE_ACCOUNT_INVALID"` — **asserter le code, pas seulement le statut** — et son message désigne **l'article**, jamais le réglage société.
  - [x] Les **cinq** mutations d'AC-A6, exécutées, consignées avec leur sortie, fichiers restaurés à l'identique.
- [x] **T-A6 — Gate** (AC-A7) — complet, état final, exit 0, verdict lu dans le log.

### Review Findings

`bmad-code-review` **passe 1** — 2026-08-04, Opus 5, trois couches (Blind Hunter aveugle, Edge Case Hunter, Acceptance Auditor). Diff `main...HEAD` de la branche `story/16-2-compte-produit-catalogue`, 21 fichiers / 2389 lignes, 16-2a **et** 16-2b confondues. Les findings de 16-2b sont dans son propre story file.

**Décisions à rendre**

- [x] [Review][Decision] **(RÉSOLU — arbitrage de Guy, 2026-08-05 : variante (c), garder D3 et retirer le code mort)** **`postable` non validé (D3) — la conséquence n'est pas celle que D3 anticipait, et le bras d'erreur qui la nommerait est mort** — `blind+edge`, **HIGH**. D3 exclut `postable` délibérément, en se calquant sur `company_invoice_settings::validate_account`. Mais le jumeau bénéficie d'une **exemption en aval** que l'article n'a pas : `InvoiceForm.svelte:313` pose `postableExemptAccountId: invoiceSettings?.defaultRevenueAccountId`, et **seulement** lui. Un compte `Revenue` devenu non imputable, porté par un article, est donc accepté par `POST`/`PUT /products` (`routes/products.rs:305-343`, trois critères — vérifié : `grep -nF "postable" crates/kesh-api/src/routes/products.rs` ne rend que trois **commentaires**, lignes 294/386/388, aucun prédicat), puis **bloque chaque ligne de facture qui en découle** (`crates/kesh-db/tests/invoices_line_revenue_account.rs:535`, `account-validity.ts:62`), sans que la fiche produit ne signale quoi que ce soit (D-B2). Corollaire mesuré : `AppError::ProductRevenueAccountInvalid` n'est construit qu'aux quatre sites de `validate_revenue_account` (`:318`, `:323`, `:330`, `:337` — trois rejets distincts, jamais `NotPostable`), donc le bras `RevenueAccountRejection::NotPostable` **ajouté par ce diff** à `errors.rs:998` et les **4** clés `product-revenue-account-not-postable` sont du **code mort**. Trois issues possibles : (a) ajouter le 4e critère — le bras et les 4 clés deviennent atteignables ; (b) garder D3 et étendre l'exemption `postableExemptAccountId` au compte recopié depuis l'article ; (c) garder D3 et **retirer** le bras mort et les 4 clés. Arbitrage de Guy.

  **Arbitrage rendu par Guy le 2026-08-05 — variante (c).** Motif : (a) et (b) changent toutes deux un **comportement**, l'une à la fiche article, l'autre à la facture, hors du périmètre que quatre passes de `validate` ont scellé ; (c) ne change **rien** de ce que l'application fait — elle supprime seulement du code qu'aucun chemin n'atteint, et qui donnait à croire le contraire.

  **Ce que la variante (c) laisse en place, et qu'il faut savoir** : un compte `Revenue` devenu **non imputable** reste accepté par `POST`/`PUT /products`, puis **bloque chaque ligne de facture** qui en découle, sans que la fiche article ne le signale (D-B2). C'est le même enfermement que la limitation **L1** de 16-2b, par un déclencheur différent — l'archivage y devient ici une bascule de `postable`. **Versé à la même issue de remédiation que L1, [#286](https://github.com/guycorbaz/kesh/issues/286)**, dont il partage la sortie : un écran listant les articles au compte devenu inutilisable les couvre tous les deux. L'issue porte explicitement les **deux** déclencheurs.

  **Appliqué** : les 4 clés `product-revenue-account-not-postable` retirées des 4 locales ; le bras `RevenueAccountRejection::NotPostable` **conservé** pour l'exhaustivité du `match` — un `unreachable!()` tuerait la task sans trace, ce que le garde-fou défensif du `CLAUDE.md` proscrit — mais il journalise en `tracing::error!` et retombe sur `common-account-invalid`, clé **déjà traduite dans les 4 locales** (vérifié : `grep -rn "^common-account-invalid" crates/kesh-i18n/locales/*/messages.ftl` → 4 sorties). Un commentaire à `errors.rs:1010` interdit d'y remettre une clé dédiée sans ajouter d'abord le critère qui la rendrait atteignable.

**Patches**

- [x] [Review][Patch] **(CORRIGÉ — gate complet REJOUÉ sur l'état final : `2115 tests run: 2115 passed, 4 skipped`, 3154 s, exit 0, verdict lu dans le log ; détail et contrôle de composition au Dev Agent Record)** **AC-A7 affirme un gate « sur l'état final » qui a précédé une modification d'un crate Rust** — `auditor`, **HIGH**. `512bec37` (18:26:46) déclare le gate complet ; `285c3a4f` (18:31:29) ajoute 20 lignes aux quatre `crates/kesh-i18n/locales/*/messages.ftl`. L'état gaté n'est plus l'état final, et la § *Gate ciblé pendant une boucle de revue* interdit ici le ciblage (la story touche `crates/kesh-db/migrations/` et un repository). Rejouer le gate complet sur `HEAD`, puis n'écrire que ce qui a tourné. [`16-2a-compte-produit-catalogue-backend.md:265`]
- [x] [Review][Patch] **(CORRIGÉ — clé Fluent et repli Rust, les deux sites)** **Le message FR est tautologique — « n'est pas un compte de produit » pour un compte de produit** — `blind`, **MEDIUM**. Les trois autres locales nomment le sujet (`de` : « Das **Konto** dieses Artikels ist kein Ertragskonto » ; `it` : « Il **conto** di questo articolo » ; `en` : « The **account** of this product ») ; seul le FR — la langue de service — énonce « X n'est pas X ». Le texte est figé **deux fois** : la clé Fluent et le repli Rust. Aucun test ne lit ce message (`create_rejects_non_revenue_account` n'asserte que `code` et `details.reason`). [`crates/kesh-i18n/locales/fr-CH/messages.ftl:1448`, `crates/kesh-api/src/errors.rs:996`]
- [x] [Review][Patch] **(CORRIGÉ — cinquième champ inscrit à la ligne 43)** **`docs/optimistic-locking-patterns.md` n'enregistre pas le champ ajouté à `is_no_op_change`** — `edge`, **MEDIUM**. Le tableau liste encore quatre champs pour cinq, alors que sa ligne 48 avait bien reçu `default_revenue_account_id` pour le jumeau `company_invoice_settings.rs` en 16-1a. Vérifié : `is_no_op_change` compare bien les cinq (`repositories/products.rs:266-271`), c'est la doc qui est en retard. [`docs/optimistic-locking-patterns.md:43`]
- [x] [Review][Patch] **(CORRIGÉ — test `put_without_the_key_erases_an_existing_account` ajouté ; deux assertions de MONTAGE le protègent de passer à vide : l'article doit porter un compte au départ, et la clé doit être ABSENTE et non à `null`)** **La branche d'effacement du `PUT` n'est couverte par aucun test** — `edge`, **MEDIUM**. Le comportement full-replace est **assumé** (D5, CHANGELOG, issue #278) et n'est pas remis en cause ici — mais il n'est épinglé par rien : les trois `.put()` de `products_revenue_account_e2e.rs` (`:408`, `:459`, `:507`) portent **tous** la clé, et le seul test de clé absente (`absent_key_and_explicit_null_both_mean_no_account:532`) ne fait que des `POST`. Ajouter un `PUT` sans la clé sur un produit **qui porte un compte**, et asserter que le compte est effacé — pinning d'un comportement documenté, pas un changement. [`crates/kesh-api/tests/products_revenue_account_e2e.rs:532`]
- [x] [Review][Patch] **(CORRIGÉ — `.ok()` remplacé par un `unwrap_or_else` qui panique en NOMMANT la cause probable)** **`seed_revenue_account` avale l'échec de son propre nettoyage** — `blind`, **LOW**. Le `.ok()` sur le `DELETE FROM accounts` jette le `Result` ; ces tests tournent sur `test_pool()` — base **partagée**, contrairement aux `#[sqlx::test]` du reste du diff — et aucun ne supprime le compte `39xx` en fin de test. Un run interrompu laisse un produit référencer le compte, le `DELETE` viole alors `fk_products_default_revenue_account` en silence, et le test panique sur l'`INSERT` : le diagnostic est envoyé à un endroit qui n'est pas celui du défaut. [`crates/kesh-db/src/repositories/products.rs:1176`]
- [x] [Review][Patch] **(CORRIGÉ — « conserver » conservé et sourcé sur le filtre `a.active` ; « remplacer » rétabli comme POSSIBLE)** **Le commentaire de D4 affirme « ni conserver ni remplacer » — la seconde moitié est fausse** — `blind`, **LOW**. « Conserver » est exact : `AccountAutocomplete.svelte:183-186` filtre `a.active`, un compte archivé est bien absent des propositions. « Remplacer » ne l'est pas : choisir un compte valide à la place a toujours été possible et déclenche la validation normalement. *(La prémisse plus large du finding d'origine — « le sélecteur propose les comptes archivés » — est **réfutée** par ce même filtre ; seule la formulation du commentaire reste à corriger.)* [`crates/kesh-api/src/routes/products.rs:390`]
- [x] [Review][Patch] **(CORRIGÉ — `review` aux trois sites ; **pas** `done`, la boucle n'étant pas convergée)** **Statut `ready-for-dev` alors que toutes les tâches sont cochées et le code livré** — `auditor`, **LOW**. Dans le story file (`:5`) **et** dans `sprint-status.yaml:262-263`. Vaut aussi pour 16-2b.

---

`bmad-code-review` **passe 2** — 2026-08-05, **Sonnet** (modèle ≠ passe 1, contexte frais), trois lentilles, **diff aplati** `main...HEAD`. **Aucune couche n'a échoué.** L'Acceptance Auditor rend **0 finding > LOW** après recompte des invariants à la source. Sévérité maximale **MEDIUM** — en **décroissance** depuis la passe 1 (HIGH), donc le 2ᵉ critère du garde-fou de splitting n'est **pas** déclenché.

**Patches**

- [x] [Review][Patch] **(CORRIGÉ — deux gardes explicites `!active` et `version` posées AVANT la validation, avec les MÊMES variantes d'erreur que le repository ; le commentaire qui affirmait l'inverse est rectifié sur place)** **La validation D4 s'exécute AVANT les contrôles d'autorité, et le commentaire voisin affirme l'inverse** — `edge`, **MEDIUM**. `update_product` lit `before` par un `find_by_id` **hors transaction** dont le SQL ne filtre **ni `active` ni `version`** (`FIND_BY_ID_SCOPED_SQL`, `repositories/products.rs:32-34` — vérifié : `WHERE id = ? AND company_id = ?`, rien d'autre). La garde D4 et `validate_revenue_account` s'exécutent donc **avant** que `products::update()` n'atteigne ses propres verrous (`:297-306`). Deux conséquences reproductibles : (1) un `PUT` sur un article **archivé** rend `PRODUCT_REVENUE_ACCOUNT_INVALID` au lieu d'« impossible de modifier un article archivé » ; (2) en **édition concurrente**, un client dont le `version` est périmé reçoit une erreur nommant un **compte qu'il n'a jamais voulu changer** — le `before` frais diverge de sa vision — au lieu d'`OPTIMISTIC_LOCK_CONFLICT`, ce qui **masque le vrai conflit**. ⚠️ **Le commentaire immédiatement au-dessus affirme la propriété que cet ordre contredit** : « le verrou optimiste couvre la fenêtre — une modification concurrente du **produit** fait diverger `version` et rend `OptimisticLockConflict` ». C'est faux dans ce chemin précis. [`crates/kesh-api/src/routes/products.rs:408-414`]
- [x] [Review][Patch] **(CORRIGÉ — test `create_accepts_non_postable_revenue_account_d3` + helper `set_account_not_postable` ; RAYON DE MUTATION MESURÉ = 1, ce test et lui seul rougit quand on ajoute le 4e critère, mesuré `--no-fail-fast` pour qu'un test non joué ne se confonde pas avec un vert)** **L'exclusion de `postable` par D3 — que l'arbitrage de la passe 1 a délibérément CONSERVÉE — n'est verrouillée par aucun test** — `edge`, **MEDIUM**. `grep -c "postable" crates/kesh-api/tests/products_revenue_account_e2e.rs crates/kesh-db/src/repositories/products.rs` → **0 et 0**. Aucun test ne pose un compte `Revenue` **non imputable** pour asserter qu'il est **accepté**. Un refactor futur « alignant » `validate_revenue_account` sur son jumeau des lignes de facture — qui, lui, contrôle `postable` — inverserait D3 **sans qu'aucun test ne rougisse**. C'est le mode d'échec du test muet, sur la décision même que la passe 1 a choisi de garder : **une décision gardée sans test n'est pas gardée**. Tout le reste du diff pratique la mutation ; ce point y échappe. [`crates/kesh-api/tests/products_revenue_account_e2e.rs`]
- [x] [Review][Patch] **(CORRIGÉ aux QUATRE sites — code `:73` et son renvoi jumeau, spec `16-2a:104`, parente `16-2:179` ; le contrat vrai est énoncé et rattaché au test qui le verrouille)** **Le commentaire de `#[serde(default)]` énonce un fait FAUX sur serde, et il vient de la spec** — `blind`, **MEDIUM**. Le doc-comment affirme que « sans lui, un `Option<T>` reste **obligatoire dans le JSON** en serde ». **Réfuté par exécution**, pas par raisonnement — crate scratch avec les versions du workspace : `SansDefault { name: String, default_revenue_account_id: Option<i64> }` **sans** l'attribut, sur `{"name":"x"}` → `Ok(SansDefault { name: "x", default_revenue_account_id: None })`. `serde_derive` traite `Option<T>` comme un cas spécial. L'attribut est **inoffensif** et le comportement testé est **correct** ; c'est la **raison** qui est fausse, et elle égare quiconque la lira pour un autre champ. ⚠️ **Quatre sites**, et l'origine est la **spec** : `16-2a:104`, la parente `16-2:179`, `routes/products.rs:74`, plus le renvoi jumeau `:88`. Même schéma que le « repli à trois maillons » de la passe 1 — une affirmation fausse de spec recopiée dans le code. [`crates/kesh-api/src/routes/products.rs:74`]
- [x] [Review][Patch] **(CORRIGÉ — section « Course acceptée » au doc-comment, disant AUSSI pourquoi elle n'est pas à fermer : la fermer contredirait D4)** **La course entre la validation du compte et l'écriture n'est ni fermée ni documentée** — `blind`, **LOW**. `validate_revenue_account` est un `SELECT` **hors transaction** exécuté avant `products::create` / `products::update`. Entre les deux, un tiers peut archiver le compte : l'écriture réussit (la FK n'exige que l'existence de la ligne, pas `active = TRUE` — `ON DELETE RESTRICT` ne protège que de la suppression), et le client reçoit **200/201** pour un compte déjà inutilisable. L'état résultant est **toléré par conception** (c'est précisément ce que D4 organise), donc la course n'est pas à *fermer* — mais elle est **non dite**, là où le dépôt commente explicitement ses courses acceptées (KF-004 sur `is_no_op_change`). [`crates/kesh-api/src/routes/products.rs:300`]
- [x] [Review][Patch] **(CORRIGÉ — l'ancre est remplacée par une désignation PAR NOM du `$derived` `active`, insensible aux décalages ; une nouvelle ancre chiffrée aurait vieilli pareil)** **Ancre périmée introduite par la passe 1 elle-même** — `blind`, **LOW**. Le commentaire de D4 corrigé en passe 1 cite `AccountAutocomplete.svelte:183-186` pour le filtre `a.active` — qui est à la ligne **203** dans l'état final. Ce sont les ~20 lignes de props `id`/`describedBy` ajoutées **par le même diff** qui l'ont décalé : l'ancre était donc **fausse à l'instant où elle a été écrite**. Récidive exacte du motif consigné à la rétro — « les 3 LOW de la P4 étaient 3 ancres fausses introduites par la P3 ». [`crates/kesh-api/src/routes/products.rs:374`]

---

`bmad-code-review` **passe 3** — 2026-08-05, **Haiku 4.5** (rotation Opus → Sonnet → Haiku complète), trois lentilles, diff **aplati** `main...HEAD`. **Aucune couche n'a échoué.**

## ✅ BOUCLE CONVERGÉE — critère d'arrêt atteint

**Trend : `2H/3M/3L` → `0H/4M/2L` → `0H/0M/2L`.** Décroissance **monotone**, plafond de 8 passes jamais approché. Plus aucun finding au-dessus de **LOW**, sur les trois lentilles : Acceptance Auditor **0 finding**, Blind Hunter **0 > LOW**, Edge Case Hunter **2 LOW**.

⚠️ **Le mode d'échec connu de Haiku ne s'est PAS manifesté cette fois** — aucun `CRITICAL` de type « le patch X n'a pas été appliqué ». Les trois garde-fous étaient en place : diff **aplati**, obligation de greper **le fichier source et jamais le `.patch`**, interdiction de reconstruire de mémoire un état antérieur. La consigne explicite « un rapport vide est un résultat attendu à la troisième passe » y a probablement contribué autant que les deux autres : sans elle, un modèle sous pression fabrique de la gravité pour paraître utile.

**Les 2 LOW — consignés, non retenus, avec le motif**

- **Asymétrie de couverture `postable` entre création et modification** — `edge`, **LOW**. Le test `create_accepts_non_postable_revenue_account_d3` verrouille D3 sur le chemin de **création** ; aucun test symétrique ne pose un compte non imputable par un `PUT`. **Fortement nuancé par vérification** : `grep -n "validate_revenue_account("` rend **une seule définition** (`:334`) et **deux appels** — `create_product:388` et `update_product:479`. Le test de création verrouille donc **le code même que la modification emprunte** ; un refactor ajoutant le critère à cette fonction partagée ferait rougir le test existant. Le trou résiduel n'existe que si un refactor plaçait le critère **hors** de la fonction, directement dans `update_product` — cas peu vraisemblable, et que la duplication du code rendrait visible en revue.
- **Pas de test « no-op avec un compte non imputable »** — `edge`, **LOW**, et **la lentille le réfute elle-même** : `is_no_op_change` compare des **valeurs** (`before.default_revenue_account_id == changes.default_revenue_account_id`), jamais l'**état** du compte référencé. Que `postable` bascule ne peut donc pas changer le verdict de no-op. Couverture cosmétique, pas défaut.

**Aucun patch appliqué en passe 3** — la § *Review Iteration Rule* fixe l'arrêt à « uniquement des LOW », et corriger au-delà relancerait un gate complet pour du cosmétique.

**Réfutés en passe 1** (consignés pour qu'une passe suivante ne les rejoue pas)

- **« La migration crée un second index sur une colonne qui en a déjà un »** — réfuté par **exécution** sur la base de gate, pas par raisonnement : `SELECT INDEX_NAME FROM information_schema.STATISTICS WHERE TABLE_SCHEMA='kesh_gate' AND COLUMN_NAME='revenue_account_id'` rend **exactement une** entrée par table (`idx_invoice_lines_revenue_account`, `idx_credit_note_lines_revenue_account`) — or `20260727000001` emploie **le même patron à la ligne près** (`ALTER … ADD COLUMN + ADD CONSTRAINT`, puis `CREATE INDEX` séparé) que la migration de cette story. Aucun index doublon n'est produit.
- **« Le `PUT` full-replace efface le compte d'une intégration tierce »** — comportement réel, mais **décidé** (D5), **documenté** (CHANGELOG, avertissement « Intégrations par API ») et **tracé** (issue #278). Seul son défaut de couverture de test est retenu, ci-dessus.
- **« L'enveloppe d'erreur devrait passer par `build_response` »** — réfuté à la lecture de la signature : `build_response(status, code, message)` ne porte **aucun** champ `details` (`errors.rs:946-957`), et **une trentaine** de bras du même `match` construisent leur `json!` à la main **pour cette raison précise** (`grep -c '"details"' crates/kesh-api/src/errors.rs` → 30+). Le nouveau bras suit donc le patron du fichier ; c'est `build_response` qui est le cas particulier — celui des erreurs sans détail.

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

### La race que D4 ne discute pas, et qui est acceptée

**D4** documente la course où le compte devient invalide **sans changer** — c'est son objet même. La course **symétrique** existe et n'est nulle part écrite : le compte **change** vers une valeur valide, la route la valide par un `SELECT` hors transaction, puis un autre administrateur archive ce compte avant que l'`UPDATE` du repository ne parte. `products::update` ne revérifie **que** `products.version` (`:275-298`) — jamais l'état du compte référencé, et D4 interdit précisément d'y déplacer la validation.

**Acceptée, et sans remède à tenter ici.** L'état atteint — un article référençant un compte devenu invalide — est **exactement** celui que D4 déclare nominal dans le cas « inchangé ». Le fermer exigerait de revalider dans la transaction, c'est-à-dire le placement que D4 écarte pour des raisons qui n'ont pas bougé. Le patron cité comme précédent (`company_invoice_settings.rs:151`) porte d'ailleurs la même course depuis toujours. **Rien à implémenter : c'est une limite connue, écrite pour ne pas être redécouverte en revue de code comme un défaut de cette story.**

### Conventions de test

`#[sqlx::test(migrator = "kesh_db::MIGRATOR")]`, fixtures `kesh_db::test_fixtures::{SeededCompany, seed_accounting_company}`. Les locales vivant dans un **crate Rust**, tout changement i18n impose `cargo test --workspace`.

⚠️ **Le preset E2E `with-data` crée déjà un produit** — `seed_contact_and_product` (`test_fixtures.rs:482-483`) insère `'CI Product'`, appelé par `routes/test_endpoints.rs:203`. Il naîtra au compte `NULL`. *(Une rédaction antérieure de la story parente affirmait le contraire ; réfuté en passe 4.)*

### References

- Story **16-2b** — surface utilisateur (sélecteur, pré-remplissage, i18n, doc-sync). **Même PR obligatoire.**
- Story parente **16-2** — `16-2-compte-produit-catalogue.md`, archivée : 4 passes de `validate` et leur historique.
- Story **16-1a** — `revenue_account_id`, règles de validité (`invoices.rs:514-590`), décision **D3-bis**, et le patron d'AC14 pour le test d'export.
- `CLAUDE.md` § « Migration breaking policy » et § « Test Locally First ».
- CR **#278** — durcissement du contrat des API en écriture, différé.

---

## Dev Agent Record

### Agent Model Used

Opus 5 (`bmad-dev-story`, 2026-08-04).

### Debug Log References

**Trois pièges d'environnement, tous rencontrés et consignés.**

1. **La base de dev `kesh` porte la pollution documentée** — compte `1000` à `postable = 0`, celle qui produit 26 faux échecs `journal_entries`. Le gate ne peut pas y tourner.
2. **`kesh_gate` s'est révélée DÉSYNCHRONISÉE** : `_sqlx_migrations` ne contenait qu'**une** ligne (`20260404000001`) alors que le schéma était complet — `sqlx migrate run` y échoue en `1050 Table 'companies' already exists`. Plutôt que de supprimer une base, une base **neuve `kesh_gate2`** a été créée, migrée aux **58** migrations et seedée au minimum (1 société, 1 utilisateur `Admin`, 5 comptes tous `active` **et** `postable`). C'est elle qui porte le gate.
3. **Le séparateur CSV du dépôt est `;`**, pas `,` (convention suisse), et la colonne du plan comptable s'appelle **`number`**, pas `code`. Deux montages de test corrigés à ce titre.

### Completion Notes List

**T-A1 — migration et les quatre garde-fous.** `20260804000001`, DDL pur. P5 : ligne d'audit avec verdict **`tracked-by-sqlx`** et ses codes **1060 / 1061**, sur le patron du jumeau littéral `20260727000001` ; compteurs **recomptés à la source** — 57 → 58 aux **deux** sites du total, `tracked-by-sqlx` 52 → 53, partition `53 + 5 + 0 = 58`. P6 : bump **conjoint** de `total` **et** de la fenêtre (`total - 23` → `total - 24`), frontière maintenue à **34** comme l'impose **D8**. P7 : **constaté** en exécutant `every_data_backfill_migration_is_triaged`, pas supposé — vert. P1/P2 : non-breaking, aucun bump.

⚠️ **Le grep de propagation a rapporté trois résidus que le bump seul aurait laissés** : le message d'assertion d'`apply_migrations_up_to` et le doc-comment de la fonction citaient encore `total - 23` / `total == 57`. C'est le symptôme que ce fichier documente **sur lui-même** — « trois sites du même symptôme dans le même fichier, découverts un par passe » lors de la revue de 16-1a. Cette fois ils sont tombés d'un coup.

**T-A2** — les **deux** listes de colonnes, les **deux** helpers, l'`INSERT` et l'`UPDATE`. Le compilateur a énuméré les **huit** sites de construction littérale, dont `kf005_fulltext_index_e2e.rs:123`, le seul hors périmètre thématique — la spec avait raison de ne nommer que celui-là et de laisser `cargo build` faire le reste.

**T-A3** — validation **D3** à la route (trois critères, `postable` exclu), conditionnée par **D4**, avec le `find_by_id` ajouté. Variante **D10** dédiée, code `PRODUCT_REVENUE_ACCOUNT_INVALID`, `details.reason` via `revenue_account_rejection_code` **réutilisé**, et la famille `product-revenue-account-*` dans les **4** locales. Le formateur de 16-1a n'est pas touché.

**T-A4** — export CSV, en-tête **et** valeurs à la même position ; le test d'export asserte désormais l'en-tête, la valeur **et la cellule vide** du produit sans compte, plus l'alignement de la colonne suivante.

**T-A5 — LES CINQ MUTATIONS SONT EXÉCUTÉES, ET LEUR RAYON EST CONSIGNÉ.**

| Mutation | Rouges | Verts |
|---|---|---|
| 1 — D3 neutralisée (`active`, `account_type`) | **3** : `create_rejects_inactive_account`, `create_rejects_non_revenue_account`, `changing_account_to_invalid_is_rejected` | 5 |
| 2 — condition D4 retirée | **1** : `renaming_succeeds_when_unchanged_account_became_invalid` | 7 |
| 3 — champ retiré d'`is_no_op_change` | **1** : `changing_only_the_account_bumps_version_and_writes_audit` **(a)** | 19, dont **(b)** |
| 4 — champ retiré de `product_snapshot_json` | **1** : `audit_shows_the_account_change_key_by_key` **(b)** | 19, dont **(a)** |
| 5 — variante D10 → `AppError::Validation` | **4** : tous ceux qui assertent le code | 4 |

**Les mutations 3 et 4 se tuent en miroir, et c'est le résultat qui valide la scission** imposée en passe 3 de `validate` : (a) rouge / (b) vert, puis (b) rouge / (a) vert. Le test (b) modifie **aussi** le nom pour que son `UPDATE` aboutisse malgré la mutation 3 ; fusionnés, les deux auraient rougi ensemble et les mutations auraient cessé de discriminer.

Fichiers restaurés à l'identique après chaque essai, `git diff` vide vérifié.

⚠️ **UNE ERREUR DE MÉTHODE, ET SON SYMPTÔME MUET.** Entre les mutations 3 et 4, `git checkout` a été employé pour restaurer un fichier dont les **quatre tests n'étaient pas encore commités** : ils ont été **effacés**. Le symptôme fut rassurant et faux — la mutation 4 a rendu « **16 passed, 0 failed** », un vert parfait, alors que quatre tests avaient **disparu au lieu d'échouer**. C'est exactement le mode d'échec du test muet que le dépôt a payé sur `backfill_skips_archived_accounts`. **Seul le décompte l'a attrapé** : 16 au lieu de 20. Tests réécrits, **commités** (`af02d966`), puis mutations 3 et 4 rejouées.

➡️ **Règle à verser à la rétro Epic 16 : commiter AVANT toute campagne de mutation.** `git checkout` ne restaure que ce que l'index connaît ; sur du travail non commité, il détruit. Et un test supprimé ne rougit pas — il disparaît, ce que seul un contrôle de **composition** révèle, jamais un « 0 failed ».

**T-A6 — GATE COMPLET VERT, VERDICT LU DANS LE LOG.**

```
FMT_EXIT=0     CLIPPY_EXIT=0     NEXTEST_EXIT=0
Summary [3862.986s] 2114 tests run: 2114 passed, 4 skipped
```

DB `kesh_gate2`, 64 minutes, sur l'**état final**. Aucun ciblage : la story touche `crates/kesh-db/migrations/` et un repository, ce que la § *Test Locally First* interdit de cibler, y compris entre deux passes.

**Le contrôle qui compte est celui de la COMPOSITION, pas du total.** 2102 → **2114**, soit **+12** — et 12 est exactement ce que la story ajoute : **8** tests de route (`#[sqlx::test]` de `products_revenue_account_e2e.rs`) et **4** tests de repository. Recompté à la source, pas relu. C'est le contrôle que la passe 1 de revue de 16-1c avait pris en défaut, où *neuf* tests livrés pour huit exigés masquaient deux manques : un total qui monte pendant que la couverture baisse ne se voit qu'en recomptant la composition.

#### Gate REJOUÉ en passe 1 de revue — l'état ci-dessus n'était plus l'état final

*(Finding `auditor` HIGH : le gate de `512bec37` [18:26:46] précédait `285c3a4f` [18:31:29], qui ajoutait 20 lignes aux quatre `crates/kesh-i18n/locales/*/messages.ftl`. Un gate qui ne porte pas sur l'état publié ne prouve rien de l'état publié.)*

```
▶ cargo fmt --check          ▶ cargo clippy --workspace --all-targets -- -D warnings
Summary [3154.350s] 2115 tests run: 2115 passed, 4 skipped
✅ Gate backend rapide vert.
```

**exit 0**, DB `kesh_gate2`, 53 minutes, sur l'état final incluant **tous** les patches de la passe 1. Verdict **lu dans le log**, jamais déduit d'un code de retour — un `script > log ; echo EXIT=$?` rend l'exit du `echo`.

**Composition, à nouveau** : 2114 → **2115**, soit **+1**, et ce 1 est exactement `put_without_the_key_erases_an_existing_account`, le test ajouté par la remédiation du finding `edge` MEDIUM. Le décompte se referme sur lui-même.

⚠️ **Deux pièges d'exécution payés ce jour** — consignés en détail au Dev Agent Record de **16-2b** (§ *Gate d'AC-B9*) : la base de gate est **`kesh_gate2`** et non `kesh_gate`, périmée d'une migration et au `_sqlx_migrations` corrompu — un premier rejeu y est mort à 1479/2115 sur un unique `Unknown column 'default_revenue_account_id'`, invisible des `#[sqlx::test]` ; et `cargo fmt --check` a mordu d'emblée sur le test ajouté en remédiation.

### File List

- `crates/kesh-db/migrations/20260804000001_products_default_revenue_account.sql` *(nouveau)*
- `crates/kesh-db/src/entities/product.rs`
- `crates/kesh-db/src/repositories/products.rs`
- `crates/kesh-db/tests/kf005_fulltext_index_e2e.rs`
- `crates/kesh-db/tests/migrations_upgrade_path.rs`
- `crates/kesh-api/src/errors.rs`
- `crates/kesh-api/src/routes/products.rs`
- `crates/kesh-api/src/exports/csv_tables.rs`
- `crates/kesh-api/tests/products_revenue_account_e2e.rs` *(nouveau)*
- `crates/kesh-api/tests/exports_global_e2e.rs`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`
- `docs/migrations-idempotence-audit.md`

**Touchés en passe 1 de `bmad-code-review`** *(l'inventaire d'une story n'est opposable qu'à jour — c'est lui que la passe suivante confrontera au diff)* :

- `crates/kesh-api/src/errors.rs` — message FR détautologisé ; bras `NotPostable` rendu bruyant (`tracing::error!`) et replié sur `common-account-invalid`, variante (c) de l'arbitrage.
- `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl` — **retrait** des 4 clés `product-revenue-account-not-postable`, inatteignables.
- `crates/kesh-api/src/routes/products.rs` — commentaire de D4 corrigé (« remplacer » est possible).
- `crates/kesh-api/tests/products_revenue_account_e2e.rs` — `put_without_the_key_erases_an_existing_account` *(nouveau test)*.
- `crates/kesh-db/src/repositories/products.rs` — `.ok()` du nettoyage de `seed_revenue_account` remplacé par une panique nommant la cause.
- `docs/optimistic-locking-patterns.md` — cinquième champ de `is_no_op_change` inscrit.
- `_bmad-output/implementation-artifacts/16-2-compte-produit-catalogue.md` — errata sur la « chaîne de repli à trois maillons », qui n'existe pas et dont ce document est la source.

## Change Log

**2026-08-05 — Passe 3 de `bmad-code-review` — ✅ BOUCLE CONVERGÉE** (**Haiku 4.5**, rotation Opus → Sonnet → Haiku complète, contexte frais, diff aplati). **0 HIGH, 0 MEDIUM, 2 LOW** — critère d'arrêt de la § *Review Iteration Rule* atteint.

**Trend des trois passes : `2H/3M/3L` → `0H/4M/2L` → `0H/0M/2L`.** Décroissance monotone ; le garde-fou de splitting n'a jamais été approché, non plus que le plafond de 8 passes.

**Aucun patch en passe 3.** Les 2 LOW sont consignés avec leur motif de non-retenue : le premier est fortement nuancé par la vérification que `validate_revenue_account` est **une seule fonction partagée** par la création et la modification — le test de création verrouille donc le chemin des deux ; le second est réfuté par la lentille elle-même, `is_no_op_change` comparant des valeurs et jamais l'état du compte.

**Le mode d'échec connu de Haiku ne s'est pas manifesté** — aucun faux `CRITICAL` de type « patch non appliqué », là où les stories 8-5a-bis et 9-2b en avaient produit quatre. Trois garde-fous étaient en place : diff aplati, obligation de greper le **fichier source** et non le `.patch`, interdiction de reconstruire de mémoire un état antérieur. À verser à la rétro Epic 16 : la consigne explicite « un rapport vide est un résultat attendu » compte autant que les garde-fous techniques.

**Aucun gate rejoué** — la passe 3 n'a modifié aucune ligne de code. Le dernier gate vaut donc pour l'état final : **2116/2116**, exit 0.

**2026-08-05 — Passe 2 de `bmad-code-review`** (**Sonnet**, modèle ≠ passe 1, contexte frais, diff **aplati** `main...HEAD`). **0 HIGH, 4 MEDIUM, 2 LOW** — sévérité maximale en **décroissance** (HIGH → MEDIUM), donc le 2ᵉ critère du garde-fou de splitting n'est pas déclenché. **Boucle NON convergée — passe 3 due.** L'**Acceptance Auditor rend zéro finding** après recompte des invariants à la source.

**Le finding qui comptait — un ordre d'erreurs qui envoie l'utilisateur corriger le mauvais champ.** `find_by_id` ne filtre ni `active` ni `version` ; la validation D4 s'exécutait donc avant les verrous de `products::update()`. En **édition concurrente**, le `before` frais diverge de ce que le client croit avoir sous les yeux : il recevait une erreur nommant **un compte qu'il n'a jamais voulu changer**, au lieu d'`OPTIMISTIC_LOCK_CONFLICT` — le vrai conflit était **masqué**. Deux gardes explicites posées avant la validation, avec les **mêmes variantes** que le repository, qui reste la source de vérité transactionnelle. ⚠️ Le commentaire voisin **affirmait la propriété que cet ordre contredisait** (« le verrou optimiste couvre la fenêtre ») — rectifié sur place.

**Une décision gardée sans test n'est pas gardée.** L'arbitrage de la passe 1 a délibérément **conservé** D3 (exclusion de `postable`) ; `grep -c "postable"` rendait pourtant **0** sur les deux fichiers de test. Un refactor « alignant » `validate_revenue_account` sur son jumeau des lignes de facture — geste qui passerait pour une **correction** — aurait inversé D3 **en silence**, suite entière verte. Test ajouté, **rayon de mutation mesuré à 1**, en `--no-fail-fast` pour qu'un test non joué ne se confonde pas avec un vert.

**Un fait faux réfuté par EXÉCUTION.** Le commentaire de `#[serde(default)]` affirmait que sans lui un `Option<T>` « reste obligatoire dans le JSON ». Crate scratch aux versions du workspace : clé absente → `None` **sans** l'attribut. L'attribut est inoffensif, la **raison** était fausse. Le grep de propagation a montré l'origine : la **spec** (`16-2a:104`, parente `16-2:179`) — **même schéma que le « repli à trois maillons »** de la passe 1, une affirmation de spec recopiée dans le code. Quatre sites corrigés.

**Deux LOW, dont un qui vise la passe 1 elle-même** : l'ancre `AccountAutocomplete.svelte:183-186`, écrite en passe 1, était fausse **à l'instant où elle a été écrite** — décalée à la ligne 203 par les props ajoutées **dans le même diff**. Remplacée par une désignation **par nom**, insensible aux décalages : une nouvelle ancre chiffrée aurait vieilli pareil. Et la course validation/écriture est désormais **écrite**, avec la raison de ne **pas** la fermer — la fermer contredirait D4.

**Gate complet vert sur l'état final** : `2116 tests run: 2116 passed (6 slow), 4 skipped`, 3775 s, exit 0, DB `kesh_gate2`, verdict lu dans le log. Composition : 2115 → **2116**, ce +1 étant exactement le test qui verrouille D3. Le frontend n'a pas été touché par cette passe — son gate et les E2E du commit précédent restent valides.

**Campagne de mutation conduite sur COPIE de sauvegarde, pas par `git checkout`** : le travail n'était pas commité, et c'est précisément ainsi que quatre tests ont été détruits en 16-2a.

**2026-08-05 — Passe 1 de `bmad-code-review`** (Opus 5, trois lentilles, diff `main...HEAD` couvrant 16-2a **et** 16-2b). **1 HIGH décision + 1 HIGH + 3 MEDIUM + 3 LOW** côté 16-2a, tous clos. **Boucle NON convergée — passe 2 due** (LLM ≠ Opus, contexte frais, diff aplati).

**La décision arbitrée par Guy — variante (c)** : garder D3 et retirer le code mort plutôt que changer un comportement hors du périmètre scellé par quatre passes de `validate`. Les 4 clés `product-revenue-account-not-postable` disparaissent des locales ; le bras du `match` subsiste, journalise et retombe sur une clé déjà traduite. Ce que la variante **laisse en place** — un compte non imputable accepté sur la fiche puis bloquant sur la facture — rejoint la limitation **L1** de 16-2b dans l'**[issue #286](https://github.com/guycorbaz/kesh/issues/286)**, qui porte les deux déclencheurs.

**Le HIGH de gate n'était pas un défaut de code mais de chronologie** : le gate déclaré précédait de cinq minutes une modification de quatre fichiers d'un crate Rust. Rejoué sur l'état final : **2115/2115**, exit 0. Le décompte se referme — 2114 → 2115, et ce +1 est exactement le test ajouté par la remédiation d'un autre finding.

**Le grep de propagation a rapporté ce que les trois lentilles avaient manqué** : le repli « à trois maillons » corrigé dans le manuel, le CHANGELOG et le README subsistait dans la story **parente**, qui en est la **source** — c'est de là qu'il avait contaminé les trois. Errata ajouté sans réécrire un document conservé pour son historique.

**Deux pièges d'exécution, 38 minutes payées** : la base de gate est `kesh_gate2` et non `kesh_gate`, périmée d'une migration — défaut **invisible des `#[sqlx::test]`**, qui rejouent tout le `MIGRATOR` sur une base éphémère, et que seul un test `--lib` sur `test_pool()` pouvait révéler. Note de mémoire corrigée avec le contrôle de cinq secondes qui l'aurait évité.

**2026-08-03 — Story née du split de 16-2**, arbitré par Guy après que le garde-fou de la dérogation se soit déclenché **deux fois** (passes 3 et 4 de `validate`, sévérité maximale `HIGH` maintenue). Le contenu repris est dans son **état corrigé de la passe 4** : il incorpore les findings des quatre passes, dont le placement de **D4** à la route (irréalisable là où la parente le plaçait), la seconde liste de colonnes `FIND_BY_ID_SCOPED_SQL`, le troisième sujet i18n du message de rejet, et le test d'export CSV qui n'observait rien.

**Ce que le split corrige, et ce qu'il ne corrige pas.** Il ne corrige aucun défaut de conception — les quatre passes ont validé la conception. Il réduit la **surface** sur laquelle une passe adversariale doit tenir un mental-model cohérent : la parente avait atteint ~460 lignes, cinq Change Logs et dix décisions, et sa comptabilité dérivait à chaque remédiation — huit des treize findings de la passe 4 portaient sur les décomptes des Change Logs eux-mêmes, pas sur la story.

**2026-08-04 — Passe 1 de `validate`** (Opus 5, trois lentilles jouées par l'orchestrateur, ground-truth systématique). **3 HIGH, 4 MEDIUM, 3 LOW**, tous remédiés. **La conception n'a pas bougé** — ce qui manquait était la **forme de l'erreur**, que les quatre passes de la parente n'avaient pas éprouvée.

**Les deux HIGH de fond ont la même racine, et une seule issue.** AC-A3 exigeait *à la fois* la validation à la route (D4 l'impose et interdit le repository) *et* le code `PRODUCT_REVENUE_ACCOUNT_INVALID`. Or `AppError::Validation` rend un code **figé** à `VALIDATION_ERROR` (`errors.rs:956-958`), et le seul chemin du dépôt produisant un code spécifique sur ce sujet passe par `DbError` — la voie que D4 interdit. **Les deux moitiés de l'AC se contredisaient**, et un dev les suivant à la lettre aurait livré `VALIDATION_ERROR` sans qu'aucun test ne bouge. Second HIGH, même racine : le « troisième sujet » était prescrit sans son mécanisme, alors que le formateur choisit par `match` sur un `Option<i32>` — **structure qui n'admet pas de troisième cas**. Arbitrage de Guy : **variante `AppError` dédiée** (nouvelle **D10**), qui règle les deux et ne touche à rien de ce que 16-1a a livré et mergé.

**Le troisième HIGH envoyait casser un test.** T-A4 prescrivait de semer un produit là où la fixture porte `("products.csv", 0)` — mais ce site appartient à `export_global_zip_empty_company_explicit_row_count_map`, **dont le nom énonce l'invariant**. Pendant ce temps, `export_global_zip_includes_archived_products` sème déjà deux produits et n'asserte qu'un `rowCount` : c'était lui, l'assertion faible à renforcer.

**Deux MEDIUM sur D2, tous deux hérités de la parente et aggravés par la réécriture.** La commande de preuve citée — `grep -nF "idx_company_invoice_settings"` — **rend une sortie** (`20260419000003:11`, un index sur `created_at`) : elle paraît réfuter la phrase qu'elle accompagne. La parente disait « aucun index **sur sa colonne** », formulation exacte, élargie ici à tort. Et la justification substituée est fausse : **InnoDB n'exige pas** l'index de l'auteur du DDL, il le **crée lui-même** — c'est précisément pourquoi le réglage société fonctionne sans en déclarer. Le motif réel, perdu à la réécriture, est le patron 16-1a.

Autres remédiations : verdict d'idempotence **tranché** (`tracked-by-sqlx`, 52 → 53) au lieu d'être laissé « à décider » alors que le jumeau littéral `20260727000001` l'avait déjà fixé ; ancre `company_invoice_settings.rs:148` → **`:151`** (`:148` est un commentaire) et `:94-121` → `:94-123` ; signature de `products::find_by_id` explicitée (`company_id` **avant** `id`) ; `validate_account` **bornée** à l'argument qu'elle sert, son style de message étant l'opposé de ce qu'exige AC-A3 ; énumération des sites littéraux rendue honnête (elle en laissait quatre, mode d'échec bruyant) ; **cinquième mutation** ajoutée, sans laquelle D10 n'était discriminée par rien.

**Vérifié sain, à ne pas refaire** : les compteurs de migrations sont **exacts** — 57 sur disque = 57 lignes de tableau = les deux sites du total, partition `5 + 52 + 0`, et `assert_eq!(total, 57)` ; l'arithmétique de D8 (57→58 **et** `-23`→`-24`) est juste. Les quinze autres sites `apply_migrations_up_to` passent par `migrations_before(version)`, **insensibles par construction** — D8 vise bien le seul site positionnel. Le **backup dérive ses colonnes d'`INFORMATION_SCHEMA`** (`backup.rs:114`) : rien à maintenir, et P7 (DDL pur, rien à rejouer) tient. Plus quatorze ancres confirmées au grep : 13 FK, zéro `product_id`, zéro `postable` dans le réglage société, le commentaire « 11 » périmé, les six `query_as` aux lignes annoncées, `is_no_op_change` à quatre champs, `product_snapshot_json` à huit clés, `NewProduct` sans `Default`, `ProductPicker` importé du seul `InvoiceForm`, `CI Product`, `expense_account_id NOT NULL`, et l'en-tête CSV observé nulle part.

**2026-08-04 — Passe 2 de `validate`** (Sonnet, **trois lentilles indépendantes en sous-agents**, contexte frais chacune : adversariale, exhaustivité des chemins, cohérence du contrat). **1 HIGH, 7 MEDIUM, 3 LOW** après déduplication ; tous remédiés, aucun écarté.

**LE HIGH — la preuve d'audit avait la forme d'une preuve sans en être une.** AC-A2 affirmait que, le champ non ajouté à `product_snapshot_json`, l'audit rendrait deux objets **« identiques »** ; et AC-A6 en tirait une mutation dont le test était « l'audit montre `before ≠ after` ». **C'est faux, et la fausseté vidait la mutation 4 de tout contenu** : la clé `"version"` fait partie des huit du snapshot (`:41`), l'`UPDATE` exécute `version = version + 1` (`:312`) et `after` est refetché **après** (`:332`) — donc `before != after` est **vrai à tout coup**, que la colonne soit au snapshot ou non. Un test écrit littéralement d'après l'AC serait resté **vert sous sa propre mutation**. L'assertion porte désormais sur **la clé**, jamais sur l'objet. *(La lentille invoquait aussi `updated_at` — vérification faite, il n'est **pas** dans le snapshot ; l'argument tient sur `version` seule, et c'est suffisant.)*

**Deux MEDIUM sont des défauts INTRODUITS par la remédiation de la passe 1** — le mode d'échec que ce dépôt mesure depuis 16-1a. (1) Le patch de passe 1 écrivait « `validate_account` est **invoquée** pour ce seul point », juste sous une ancre `fichier:ligne` : lisible comme un appel de code, alors que la fonction est **privée** (`async fn` sans `pub`, aucun appel hors de son fichier) et que son retour est le `AppError::Validation` que D10 interdit. Un dev l'aurait rendue `pub` et aurait hérité du chemin d'erreur qu'on venait d'écarter. (2) Ce même patch listait « les autres sites » de construction littérale en en omettant deux (`:753`, `:1150`) et en comptant « deux sites » dans `routes/products.rs` là où il y en a un de chaque type — **lister à moitié donne l'impression d'avoir listé**. Les deux lentilles concernées ont convergé sur ce second point.

**Le chevauchement des mutations 3 et 4**, que la formulation de T-A5 rendait inévitable : les deux assertions vivaient dans un même scénario « modifier le SEUL compte ». Or sous la mutation 3, ce scénario devient un **no-op** — `products::update` rollback avant d'écrire l'audit (`:300-308` vs `:339-343`) — donc le test de la mutation 4 aurait rougi **aussi**, contredisant « chacune tue le test visé et lui seul ». Scindé en deux tests, celui du contenu d'audit modifiant **aussi** un autre champ.

**Trois chemins non traités**, tous devenus des tests nommés : **retirer un compte déjà posé** (`Some → null` est un changement au sens de D4 mais n'a rien à valider — sans la garde, un compte devient **impossible à retirer une fois posé**) ; **changer le compte VERS une valeur invalide** (la mutation 2 rend la validation inconditionnelle, elle n'attrape pas un prédicat **inversé**) ; et **la cellule CSV vide** du produit sans compte — c'est elle qui révèle un décalage de colonnes, un en-tête et une valeur pris sur la même ligne pouvant être décalés du même cran et concorder.

Autres remédiations : **`revenue_account_rejection_code`** (`errors.rs:53-60`) nommé dans D10, faute de quoi un second `match` à quatre branches sur le même enum serait redérivé ; branche `None` du `find_by_id` tranchée (404) ; ancre `test_fixtures.rs:482` → `:482-483`, **propagée à 16-2b** ; et la **course symétrique** de D4 — compte validé à la route puis archivé avant l'`UPDATE` — écrite en Dev Notes comme limite acceptée, pour qu'une revue de code ne la redécouvre pas comme un défaut de cette story.

**Ce que la passe 2 a validé et qu'il ne faut pas re-vérifier** : la traçabilité `D1..D10 → AC → T` est complète, sans AC orphelin ni tâche sans AC ; les décomptes recomptés à la source par deux lentilles indépendantes (57 migrations, partition `5+52+0`, 13 FK, six `query_as`, quatre champs d'`is_no_op_change`, huit clés de snapshot, quatre locales) ; l'arithmétique de D8 ; le restore d'un backup **antérieur** à la migration (colonne nullable jamais `required`, donc import sans erreur, conforme à D7) et d'un backup **postérieur** sur binaire ancien (400 `IMPORT_SCHEMA_MISMATCH`) ; l'absence de route `DELETE` sur les comptes, qui rend le `ON DELETE RESTRICT` inatteignable par l'API ; et la frontière avec 16-2b, contrôlée dans les deux sens — aucun travail réclamé deux fois, aucun oublié.

**2026-08-04 — Passe 3 de `validate`** (Haiku 4.5, trois lentilles, contexte frais, **diff aplati `main..HEAD`** conformément à la § *Haiku-specific guardrails*). **0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW** — **BOUCLE CONVERGÉE**, critère d'arrêt de la § *Review Iteration Rule* atteint.

**Le seul LOW, remédié** : AC-A3 prescrivait les entrées (`#[serde(default)]`) sans rien dire de la **sortie**. Tranché sur le patron du fichier — `ProductResponse` ne porte aucun `skip_serializing_if`, le champ est donc toujours présent et vaut `null` quand il est absent, comme `description`.

**Les trois rapports sont ÉNUMÉRÉS, et c'est ce qui les rend opposables.** Un « 0 finding » ne vaut que par ce qu'il déclare avoir contrôlé : les décomptes ont été recomptés **à la source** par deux lentilles indépendantes (57 migrations aux deux sites, partition `5+52+0`, 13 FK, six `query_as`, quatre champs, huit clés, quatre locales, cinq mutations) ; la traçabilité `D1..D10 → AC → T` a été reparcourue sans AC orphelin ni tâche sans AC ; et la discrimination des cinq mutations a été confrontée une à une aux tests de T-A5, **y compris la scission (a)/(b) issue de la passe 2** — qui tient. La lentille d'exhaustivité a par ailleurs vérifié que `seed_accounting_company` sème bien un compte `Revenue` (**3000**) *et* un non-`Revenue` (**4000**), donc que les deux cas de rejet sont **constructibles** avec la fixture prescrite.

**Aucune hallucination à réfuter** — contrairement au précédent Haiku documenté au `CLAUDE.md`, les trois lentilles ont respecté la consigne de ground-truth et n'ont produit aucun CRITICAL/HIGH à écarter.

**Trend de la boucle : `3H/4M/3L` → `1H/7M/3L` → `0/0/0/1L`.** Rotation complète **Opus → Sonnet → Haiku**, trois lentilles par passe, plafond de 8 passes jamais approché. La sévérité décroît de façon **monotone** : le garde-fou de la § *Règle de splitting préventif* ne se déclenche à aucune passe.

**Statut : `ready-for-dev`, VALIDÉE.** Prête pour `bmad-dev-story`. ⚠️ **Rappel non négociable : même PR que 16-2b** — seule, cette story livre une colonne que rien ne lit.
