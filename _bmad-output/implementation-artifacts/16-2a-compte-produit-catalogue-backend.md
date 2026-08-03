# Story 16.2a : Compte de produit sur la fiche produit — socle backend

## Status

ready-for-dev

## Story

**As a** mainteneur de Kesh,
**I want** que la fiche produit puisse porter **son** compte de produit, persisté et validé,
**so that** la surface utilisateur livrée par **16-2b** ait une donnée à lire — et que la colonne naisse avec ses garde-fous de migration plutôt qu'après coup.

Issue : **#144**. Sous-story de l'Epic 16, cible **v0.9.0**. **Née du split de 16-2** (4 passes de `validate`, cf. `16-2-compte-produit-catalogue.md` pour l'historique).

⚠️ **Doit partir dans la MÊME PR que 16-2b.** Seule, cette story livre une colonne que **rien ne lit** — le « code mort qui paraît fonctionner » que **D6** invoque par ailleurs. C'est l'objection qui a fait refuser le split deux fois ; elle est levée par la contrainte de PR, pas par le split.

**Dépend de 16-1** (PR #284, ouverte, CI verte). La branche est issue de `story/16-1-…` et non de `main` — cf. Dev Notes pour le rebase.

---

## Contexte

Trois faits, tous établis par relevé et **vérifiés à quatre reprises** par les passes de `validate` de la story parente.

1. **Aucun `product_id` n'existe sur les tables de lignes** — `grep -rn "product_id" crates/kesh-db/migrations/*.sql` → aucune sortie. Le lien catalogue → facture est une **recopie** au moment du choix, pas une référence (cf. **D1**), et il vit entièrement en **16-2b**.
2. **Le patron « compte par défaut » existe au niveau société** : `company_invoice_settings.default_revenue_account_id BIGINT NULL`, FK `ON DELETE RESTRICT` (`20260417000001_invoice_validation.sql:39-47`). Convention unanime — **13** FK vers `accounts` (`grep -rn "REFERENCES accounts" crates/kesh-db/migrations/*.sql | wc -l`). *(Le commentaire de `20260727000001:21-23` annonce « 11 » : exact quand 16-1a l'a écrit. Recompter, ne pas relire.)*
3. **Le réglage société ne valide PAS `postable`** — `grep -n "postable" crates/kesh-api/src/routes/company_invoice_settings.rs` → aucune sortie. C'est le patron dont **D3** se réclame.

---

## Décisions

- **D1 — Aucun `product_id`, aucune relation persistante.** Rappel de cadrage ; le geste de recopie est en 16-2b. Introduire `product_id` serait une story distincte et non demandée.

- **D2 — `products.default_revenue_account_id BIGINT NULL`**, FK `ON DELETE RESTRICT`, index dédié. `NULL` = « cet article n'impose rien », la ligne suit alors le défaut société. ⚠️ **Pas un « miroir strict » du réglage société** : celui-ci ne porte **aucun** index (`grep -nF "idx_company_invoice_settings"` → aucune sortie). L'index se justifie parce qu'**InnoDB l'exige sur la colonne enfant d'une FK**, pas par un besoin de requête — aucune ne filtre `products` par compte.

- **D3 — Validation sur TROIS critères, `postable` EXCLU.** À l'enregistrement :
  1. **exister et appartenir à la société** — un seul critère, une seule variante de rejet `RevenueAccountRejection::UnknownOrCrossCompany`, qui rend un compte d'une autre société indiscernable d'un compte inexistant (garde anti-IDOR) ;
  2. `active` ;
  3. `account_type = Revenue`.

  **`postable` est écarté** parce que le code jumeau l'écarte : `company_invoice_settings::validate_account` (`routes/company_invoice_settings.rs:94-121`) ne le contrôle jamais. Le sanctionner ici bloquerait l'édition d'un article sur un champ non touché.

  📎 **`D3-bis` (story 16-1a), rappelée pour que cette spec se lise seule** : à la validation d'une facture, `postable` est **exempté** pour le compte égal au `default_revenue_account_id` de la société. C'est une clause de **grand-père** protégeant un réglage préexistant — pas une autorisation générale, d'où son non-report ici.

- **D4 — La validation ne se déclenche QUE si le compte CHANGE.** Sur l'`update`, **D3** ne s'applique que si `changes.default_revenue_account_id != before.default_revenue_account_id`. Une valeur **inchangée** passe, quel que soit l'état devenu du compte.

  ⚠️ **Sans cette décision, D3 ne fermait le piège qu'à moitié.** Le raisonnement qui écarte `postable` vaut **mot pour mot** pour `active` : archiver un compte est **bien plus fréquent** que basculer `postable`, et rien n'inspecte les référents à l'archivage. L'utilisateur renomme un article, `is_no_op_change` rend `false`, l'`UPDATE` part, et la validation rejette sur un champ qu'il n'a pas touché. Pire : un compte archivé est **absent des propositions** du sélecteur, donc l'utilisateur ne peut **ni conserver ni remplacer** la valeur.

  **Placement — tranché, parce que la question n'est pas neutre.** La comparaison exige `before`, et `update_product` (`routes/products.rs:295-322`) n'en lit **aucun** aujourd'hui : le seul existe **dans la transaction** de `products::update` (`repositories/products.rs:275-298`). **La validation reste à la route**, avec un `products::find_by_id` supplémentaire **avant** l'appel au repository — patron déjà accepté dans le dépôt : `company_invoice_settings.rs:148` fetche son état courant à la route, hors transaction. Le verrou optimiste couvre la fenêtre : une modification concurrente du **produit** fait diverger `version` et rend `OptimisticLockConflict` ; une modification concurrente du **compte référencé** ne touche pas `products.version` — et c'est précisément le cas que D4 veut laisser passer.

  ⚠️ **Ne PAS déplacer la validation dans le repository** sous prétexte que `before` y est gratuit : les erreurs remonteraient en `DbError` au lieu d'`AppError::Validation`, et les tests de route deviendraient inopérants.

- **D5 — Le `PUT` reste full-replace, et cette story en aggrave la portée.** `products::update` écrit ses colonnes métier inconditionnellement (`:310-313`) ; `ProductUpdate` n'a aucun champ sentinelle. Un client d'API qui relit puis réécrit **sans** la clé `defaultRevenueAccountId` **efface le compte**, sans erreur. C'est le CR **#278**, différé par arbitrage. Obligations : l'avertissement au CHANGELOG (**16-2b**, AC-B7) ; aucune tentative de sémantique « conserver » ici.

- **D6 — Le compte de charge (achat) est HORS PÉRIMÈTRE.** #144 le pose « optionnel / à discuter ». Aucun chemin n'existe entre le catalogue et les factures fournisseur : `supplier_invoice_lines.expense_account_id` est une FK **obligatoire** saisie à la main (`20260628000001:79-96`), et aucun sélecteur d'article n'existe sur ce formulaire. Livrer le champ sans son chemin produirait une colonne que rien ne lit.

- **D7 — Aucune reprise rétroactive.** Arbitrage de kickoff. Migration en **DDL pur** — aucun `UPDATE`, aucun `INSERT`. Les articles existants naissent à `NULL`, comportement actuel strictement préservé.

- **D8 — La frontière de la fenêtre d'upgrade RESTE à 34 : bumper `total` ET la fenêtre.** La migration casse volontairement `upgrade_path_preserves_data` (`crates/kesh-db/tests/migrations_upgrade_path.rs:88-92`, assertion à `:89`) : `assert_eq!(total, 57)` passe à **58**, et la fenêtre `total - 23` à **`total - 24`**. Laisser `- 23` déplacerait la frontière à 35 et la ferait rétrécir d'un cran à **chaque** migration future — la version lente du mode d'échec que **P6** existe pour attraper.

- **D9 — Les écritures de journal sont HORS PÉRIMÈTRE.** #144 demande le pré-remplissage « sur une facture **/ écriture** ». `JournalEntryForm.svelte` ne porte **aucun** sélecteur d'article — `ProductPicker` n'est importé que par `InvoiceForm` (`grep -rn "ProductPicker" frontend/src`). Il n'existe donc aucun geste à pré-remplir. À rouvrir si un sélecteur d'article arrive un jour sur la saisie d'écriture.

---

## Acceptance Criteria

- **AC-A1 — Schéma** (D2). Migration ajoutant `products.default_revenue_account_id BIGINT NULL`, `fk_products_default_revenue_account` vers `accounts(id)` `ON DELETE RESTRICT`, et `INDEX idx_products_default_revenue_account`. **DDL pur** — aucun statement d'écriture de données.

- **AC-A2 — Entité et repository.** `Product`, `NewProduct`, `ProductUpdate` portent `default_revenue_account_id: Option<i64>`.

  ⚠️ **DEUX listes de colonnes écrites à la main, qui ne dérivent pas l'une de l'autre** : la constante `COLUMNS` (`repositories/products.rs:22-23`) **et** `FIND_BY_ID_SCOPED_SQL` (`:26`), seconde chaîne littérale indépendante alimentant **six** `query_as::<_, Product>` (`:168`, `:202`, `:275`, `:332`, `:374`, `:416`). Les deux l'incluent, ainsi que l'`INSERT` (`:146`) et l'`UPDATE` (`:311`). `Product` dérive `sqlx::FromRow` : un champ absent d'un `SELECT` produit un `ColumnNotFound` **à l'exécution** — donc **toutes** les routes produits en 500.

  ⚠️ **DEUX helpers écrits à la main, chacun avec son défaut SILENCIEUX.** Ni l'un ni l'autre ne dérive de la struct — le compilateur ne dira rien.
  - `is_no_op_change` (`:256-261`) compare **quatre** champs. Non étendu, une modification portant **uniquement** sur le compte est prise pour un no-op : `UPDATE` court-circuité, `version` inchangée, **aucun audit — et l'API répond succès**.
  - `product_snapshot_json` (`:32-43`) énumère **huit** clés. Non étendu, l'audit `{before, after}` de `product.updated` rend deux objets **identiques** : une entrée qui **ment**, ce qui est pire qu'une entrée absente.

- **AC-A3 — API et validation** (D3, D4). `CreateProductRequest` et `UpdateProductRequest` acceptent `defaultRevenueAccountId` (camelCase, `Option<i64>`, `#[serde(default)]`) ; `ProductResponse` le restitue. La validation applique **D3** — trois critères — et **ne se déclenche, sur l'`update`, que si le compte change** (**D4**, avec son placement à la route). Un compte invalide rend **400**, code applicatif **`PRODUCT_REVENUE_ACCOUNT_INVALID`**, motif issu de `RevenueAccountRejection` (`crates/kesh-db/src/errors.rs:14-27`) — l'enum est **réutilisée**, jamais dupliquée.

  ⚠️ **Le message exige un TROISIÈME sujet.** Le formateur de `crates/kesh-api/src/errors.rs:75-86` ne connaît que la **ligne de facture** (`Some(n)`) et **« le compte de produit par défaut de la société »** (`None`). Une fiche produit n'est ni l'un ni l'autre : réutiliser tel quel ferait lire, à qui édite un article, *« le compte de produit par défaut de la société : le compte 3400 est archivé »* — un message désignant **un autre objet**, qui envoie corriger un réglage non touché. Ajouter une clé Fluent + fallback dans la famille `invoice-line-account-subject-*`, **côté `kesh-api`**, dans les **4** locales.

  ⚠️ **`#[serde(default)]` n'est pas cosmétique** : sans lui, un `Option<T>` reste **obligatoire dans le JSON** en serde, et l'omission de la clé casse **toute intégration API existante**, y compris celles qui ignorent le nouveau champ.

- **AC-A4 — Export CSV.** `serialize_products_csv` (`crates/kesh-api/src/exports/csv_tables.rs:364-397`) inclut la colonne, à sa position dans `COLUMNS`. ⚠️ **Deux listes écrites à la main que rien ne relie** — l'en-tête (`:368-379`) et l'enregistrement (`:382-393`) : un décalage entre elles rend le CSV **silencieusement faux sur toutes les colonnes suivantes**.

- **AC-A5 — Garde-fous de migration**, les **quatre**, chacun **recompté à la source** :
  - **P5** — `docs/migrations-idempotence-audit.md` porte une ligne pour la migration ; l'en-tête `## Table d'audit (N migrations)` **et** la ligne `Total` passent de **57** à **58** ; la somme des trois compteurs de partition — `yes`, `tracked-by-sqlx`, `no` — égale ce total. ⚠️ Les compteurs de partition **ne valent pas** le total ; les aligner dessus casserait l'invariant. Le verdict d'idempotence de la nouvelle ligne est à **décider et écrire**, c'est la seule donnée qu'aucun recomptage ne donne.
  - **P6** — `grep -rn "migrations.len()\|apply_migrations_up_to" crates/`, inspecter **chaque** site ; appliquer **D8** : `total` → 58 **ET** fenêtre → `total - 24`.
  - **P7** — DDL pur, donc ni entrée au registre ni exemption. ⚠️ **Le constater en exécutant `every_data_backfill_migration_is_triaged`**, pas le supposer.
  - **P1/P2** — `ADD COLUMN` nullable ⇒ non-breaking ⇒ **aucun** bump de `kesh_version_min_required`, donc aucun bump Cargo.

- **AC-A6 — Discrimination prouvée par mutation.** **Quatre** mutations exécutées et consignées au Dev Agent Record, chacune tuant le test visé **et lui seul** :
  1. validation **D3** retirée → le test du compte invalide (au niveau **route**) rougit ;
  2. condition de **D4** retirée (validation inconditionnelle) → le test « renommer un article dont le compte a été archivé » rougit ;
  3. champ retiré de **`is_no_op_change`** → le test « modifier le seul compte bumpe la `version` » rougit ;
  4. champ retiré de **`product_snapshot_json`** → le test « l'audit montre `before ≠ after` » rougit.

  **Un test attendu qui ne rougit pas invalide le montage** : le corriger avant d'aller plus loin.

- **AC-A7 — Gate complet.** `scripts/test-fast.sh` (fmt + clippy `-D warnings` + nextest workspace) sur l'**état final**, exit 0, **verdict lu dans le log**. ⚠️ Cette story touche `crates/kesh-db/migrations/` et un repository : la § « Test Locally First » → « Pendant une boucle de revue » **interdit le gate ciblé**, y compris entre deux passes.

---

## Tasks / Subtasks

- [ ] **T-A1 — Migration et garde-fous** (AC-A1, AC-A5)
  - [ ] `crates/kesh-db/migrations/<timestamp>_products_default_revenue_account.sql` — `ADD COLUMN` + FK + index, DDL pur.
  - [ ] Ligne au tableau de `docs/migrations-idempotence-audit.md` avec son **verdict d'idempotence décidé** ; compteurs **recomptés** (57 → 58 aux deux sites du total, partition cohérente).
  - [ ] `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` ; `assert_eq!(total, 57)` → **58** **ET** fenêtre `total - 23` → **`total - 24`** (D8). ⚠️ Bumper `total` seul déplace la frontière à 35, **silencieusement**.
  - [ ] Exécuter `every_data_backfill_migration_is_triaged` pour **constater** que le DDL pur ne déclenche rien.
- [ ] **T-A2 — Entité et repository** (AC-A2)
  - [ ] Champ dans `Product` / `NewProduct` / `ProductUpdate`.
  - [ ] **Les DEUX listes de colonnes** : `COLUMNS` **et** `FIND_BY_ID_SCOPED_SQL`, plus `INSERT` et `UPDATE`.
  - [ ] **Les DEUX helpers** : `is_no_op_change` **et** `product_snapshot_json`.
  - [ ] Site de construction hors périmètre thématique : `NewProduct` en littéral dans `crates/kesh-db/tests/kf005_fulltext_index_e2e.rs:123`. La struct ne dérive pas `Default` → **la compilation casse**, échec bruyant mais site que rien d'autre ne signale.
- [ ] **T-A3 — API, validation, message** (AC-A3)
  - [ ] DTO + `ProductResponse`, contrat camelCase, `#[serde(default)]`.
  - [ ] Validation **D3** à la route, **conditionnée par D4** — ajouter le `products::find_by_id` avant l'appel au repository, sur le patron de `company_invoice_settings.rs:148`.
  - [ ] **Troisième sujet** du message de rejet : clé Fluent + fallback dans `crates/kesh-i18n/locales/*/messages.ftl`, **4** locales.
- [ ] **T-A4 — Export CSV** (AC-A4)
  - [ ] Étendre `serialize_products_csv`, en-tête **et** valeurs.
  - [ ] **Étendre le test d'export** `crates/kesh-api/tests/exports_global_e2e.rs` : sa fixture porte `("products.csv", 0)` — **zéro ligne**, donc l'en-tête n'est observé par rien (`grep -rnF "unit_price,vat_rate" crates/` → aucune sortie). Semer un produit **avec** un compte, asserter en-tête **et** valeur. *(Complément qu'exigeait 16-1a en AC14.)*
- [ ] **T-A5 — Tests et preuve** (AC-A3, AC-A6)
  - [ ] Intégration `kesh-db` : le champ persiste et se relit ; **modifier le SEUL compte bumpe la `version` et écrit l'audit** ; **l'audit montre `before ≠ after`** ; une modification sans changement reste un no-op.
  - [ ] **Au niveau ROUTE, pas repository** — la validation vit dans `routes/products.rs` et rend un **400** : rejet d'un compte inactif, d'un compte non-`Revenue`, et **scoping multi-tenant** (compte d'une autre société).
  - [ ] **D4** : modifier le nom d'un article dont le compte a été **archivé entre-temps** doit **réussir**.
  - [ ] **Non-régression du contrat HTTP** : payload **sans la clé**, et payload avec `"defaultRevenueAccountId": null` — les deux valent `NULL`, jamais 400.
  - [ ] **Message de rejet** : le sujet rendu désigne bien **l'article**, pas le réglage société.
  - [ ] Les **quatre** mutations d'AC-A6, exécutées, consignées avec leur sortie, fichiers restaurés à l'identique.
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

### La branche ne part pas de `main` — mécanique du rebase

Branche issue de `story/16-1-compte-produit-par-ligne` : les ancres sont invérifiables sur `main` tant que la **PR #284** n'est pas mergée.

⚠️ **#284 sera mergée en *squash*** (convention du dépôt : PR #275 → un seul commit `dc3ece04`). Un `git rebase main` naïf rejouerait les 53 commits de 16-1 contre un `main` qui les contient déjà squashés. Forme correcte :

```sh
git checkout main && git pull --ff-only
git rebase --onto main 0ce6e13a story/16-2-...
```

`0ce6e13a` est le point de fork — **le relever maintenant**, un squash le rend introuvable par `git merge-base`.

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

**Statut : `ready-for-dev`, non validée.** Une boucle `validate` propre reste à lancer sur ce document.
