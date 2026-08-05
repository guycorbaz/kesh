# Story 16.2 : Compte de produit par défaut sur la fiche produit

## Status

superseded-by-split

⚠️ **Cette story a été SPLITTÉE le 2026-08-03 en [`16-2a`](16-2a-compte-produit-catalogue-backend.md) (socle backend) et [`16-2b`](16-2b-selecteur-et-prefill-frontend.md) (sélecteur et pré-remplissage). Ne pas l'implémenter.** Elle est conservée pour son historique : quatre passes de `bmad-create-story validate`, deux déclenchements du garde-fou de la dérogation au splitting, et le détail des 30+ findings dont les deux filles héritent à l'état corrigé.

⚠️ **TROISIÈME affirmation fausse, relevée à la passe 1 de `bmad-code-review` le 2026-08-05 — et celle-ci a réellement nui.** Ce document décrit une **chaîne de repli à trois maillons** `ligne → article → société` (`:405`, et la tâche `:260` qui prescrit d'en faire état au manuel). **Elle n'existe pas.** Il n'y a **aucun** lien persistant entre une ligne de facture et l'article dont elle vient — `grep -rn "product_id" crates/kesh-db/migrations/*.sql` ne rend rien, et l'entité `invoice.rs:58-66` ne le porte pas. Le compte de l'article est **recopié une fois**, au moment du choix dans le catalogue ; une ligne **vidée** retombe sur le défaut société, jamais sur l'article. ⚠️ **La formulation d'ici s'est propagée aux TROIS livrables documentaires de 16-2b** — manuel utilisateur (`:583`, `:603`, **PDF publié**), CHANGELOG et README — et n'a été rattrapée qu'en revue. Les trois sont corrigés ; c'est le cas d'école d'une erreur de spec qui traverse l'implémentation sans qu'aucun test ne morde, la documentation n'ayant rien qui rougisse. Formulation qui fait foi : `16-2b`, § *Correction de passe 1*.

⚠️ **Deux affirmations de ce document sont FAUSSES, et corrigées dans [`16-2a`](16-2a-compte-produit-catalogue-backend.md)** — relevées à sa passe 1 de `validate` le 2026-08-04. Elles sont laissées ici telles quelles, ce document étant une trace : (1) **D2 (ligne 70)** invoque `grep -nF "idx_company_invoice_settings"` comme rendant « aucune sortie » — il en rend **une** (`20260419000003:11`, index sur `created_at`) ; seule la formulation restreinte « aucun index **sur sa colonne** », employée plus bas au récit de la passe 3, est exacte. (2) **D2 encore** : « InnoDB l'exige sur la colonne enfant d'une FK » — InnoDB **crée** cet index lui-même, il n'exige rien de l'auteur du DDL ; le motif réel est le patron 16-1a. Voir **D2** et **D10** de 16-2a pour les formulations qui font foi.

*(Résidu de rédaction corrigé le 2026-08-04 : ce bloc annonçait encore « boucle en cours, 3 passes, non convergée » et un passage prochain à `ready-for-dev`, sous un statut `superseded-by-split` — la contradiction d'états que la passe 3 avait précisément relevée sur cette story, laissée en place par le patch du split. La boucle n'est pas « en cours » : elle est close par le split, et c'est aux deux filles de converger.)*

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

- **D2 — `products.default_revenue_account_id BIGINT NULL`**, FK `ON DELETE RESTRICT`, index dédié. Jumeau de `company_invoice_settings.default_revenue_account_id` pour la colonne et la FK. ⚠️ **Pas un miroir strict** : ce réglage société ne porte **aucun** index (`grep -nF "idx_company_invoice_settings"` → aucune sortie). L'index vient du patron 16-1a, et sa justification est qu'**InnoDB l'exige sur la colonne enfant d'une FK** — pas un besoin de requête, aucune ne filtre `products` par compte. *(Formulation corrigée en passe 3.)* `NULL` signifie « cet article n'impose rien » — la ligne retombe alors sur le comportement livré par 16-1, c'est-à-dire `revenueAccountId: null`, donc le compte par défaut **de la société**, résolu à la validation.

- **D3 — Le compte de l'article est validé sur TROIS critères, `postable` EXCLU — miroir exact du réglage société.** À l'enregistrement de la fiche (`create` **et** `update`) :

  1. **exister et appartenir à la société** — les deux ne font qu'**un** critère, sanctionné par une **seule** variante de rejet, `RevenueAccountRejection::UnknownOrCrossCompany` : un compte d'une autre société est rendu indiscernable d'un compte inexistant, c'est la garde anti-IDOR ;
  2. `active` ;
  3. `account_type = Revenue`.

  **`postable` n'est PAS contrôlé ici** — c'est le quatrième critère des lignes, et il est délibérément écarté.

  **Motif, et il est établi par le code jumeau, pas par analogie** : `company_invoice_settings::validate_account` (`crates/kesh-api/src/routes/company_invoice_settings.rs:94-121`) ne vérifie **jamais** `postable` — `grep -n "postable" crates/kesh-api/src/routes/company_invoice_settings.rs` ne rend **aucune ligne**. Le réglage société, qui est le patron dont cette story se réclame, laisse donc délibérément passer un compte non imputable au moment du *réglage*, et ne le sanctionne qu'à la *validation de facture* — c'est là que D3-bis intervient.

  📎 **Ce qu'est D3-bis, puisque ce document s'y appuie trois fois** *(décision de la story **16-1a**, rappelée ici pour que la spec se lise seule)* : à la validation d'une facture, le critère `postable` est **exempté** pour le compte égal au `default_revenue_account_id` de la société. Motif : un réglage société posé avant que le compte ne devienne non imputable ne doit pas bloquer toute validation de facture. C'est une clause de **grand-père**, pas une autorisation générale — d'où son non-report ici.

  ⚠️ **Une rédaction antérieure de cette décision imposait les quatre critères, `postable` compris. Elle créait un blocage silencieux sur un champ que l'utilisateur ne touche pas**, et le scénario est concret : un article porte un compte valide ; plus tard un administrateur rend ce compte non imputable par un flux sans aucun rapport (`accounts::update`) ; l'utilisateur renomme l'article ou change son prix ; `is_no_op_change` rend `false` puisque le nom a changé, l'`UPDATE` est tenté, et la validation **rejette en 400 à cause du compte** — que l'utilisateur n'a pas touché et qu'il ne peut débloquer qu'en vidant le champ au passage. C'est le symétrique exact du problème que D3-bis a été inventée pour résoudre côté société, réintroduit un cran plus tôt dans le cycle de vie. *(Relevé en passe 1 de `validate`.)*

  **Où le signal est-il rendu, alors ?** Là où 16-1 l'a déjà construit : au niveau de la ligne de facture, par le marqueur d'invalidité et le blocage (cf. **D4**). Un compte devenu inutilisable ne se découvre pas en éditant le catalogue — il se découvre au moment où l'on s'en sert.

- **D9 — La validation ne se déclenche QUE si le compte CHANGE.** Sur l'`update`, les trois critères de **D3** ne s'appliquent que lorsque `changes.default_revenue_account_id != before.default_revenue_account_id`. Une valeur **inchangée** est laissée passer, quel que soit l'état devenu du compte.

  ⚠️ **Sans cette décision, D3 ne fermait le piège qu'à moitié — et sur le déclencheur le moins fréquent.** Le scénario qui a fait retirer `postable` en passe 1 s'applique **mot pour mot** aux deux critères que D3 conserve, `active` et `account_type` : archiver un compte est une action d'administration **bien plus courante** que basculer `postable`, et rien n'inspecte les référents à l'archivage (`PUT /api/v1/accounts/{id}/archive`). L'utilisateur renomme un article, `is_no_op_change` rend `false`, l'`UPDATE` part, et la validation rejette sur un champ qu'il n'a pas touché.

  **Et l'issue y est pire que pour `postable`** : un compte archivé est **absent des propositions** du sélecteur (`AccountAutocomplete.svelte:165-172` filtre sur `a.active`), donc l'utilisateur ne peut **ni conserver ni remplacer** la valeur. Seul `allowClear` le sort de l'impasse — en perdant l'imputation qu'il avait choisie. Renommer un article deviendrait impossible dès qu'un compte a été archivé ailleurs.

  **Aggravant, et c'est ce qui rend la décision urgente** : le test E2E qu'exige **T6** (assigner un compte, l'archiver, monter une facture) **crée délibérément cet état**. Sans D9, ce test rendrait l'article non éditable, et le montage échouerait pour une raison sans rapport avec ce qu'il vise.

  **Le critère est décidable et local** : comparer deux `Option<i64>`. Il ne demande aucune exemption, aucune liste, aucun cas particulier. *(Décision ajoutée en passe 3 de `validate`.)*

- **D4 — Un compte devenu inutilisable est pré-rempli quand même, puis signalé par le marqueur de 16-1.** Si l'article référence un compte archivé, retypé ou devenu non imputable, `onProductSelect` recopie la valeur telle quelle ; le sélecteur de ligne l'affiche, le marque `markInvalid` et bloque l'enregistrement en nommant la ligne. **Motif** : c'est l'arbitrage rendu par Guy en 16-1b — *afficher + signaler + bloquer* — et la machinerie existe déjà (`isAccountUnusable`, `AccountAutocomplete markInvalid`, `allowClear`). L'alternative (ne pas pré-remplir, retomber en silence sur le défaut société) **cacherait** à l'utilisateur que la fiche produit est à corriger.

- **D5 — Le `PUT` produit reste full-replace, et cette story en aggrave la portée.** `products::update` écrit les quatre colonnes métier inconditionnellement (`repositories/products.rs:310-313`) ; `ProductUpdate` n'a aucun champ sentinelle « ne pas toucher ». Ajouter une cinquième colonne signifie qu'**un client d'API qui relit puis réécrit une fiche sans la clé `defaultRevenueAccountId` efface le compte**, sans erreur. C'est exactement le CR **#278**, dont l'arbitrage (story 16-1a) est de ne pas durcir le contrat maintenant. **Obligations de cette story** : le frontend envoie **toujours** le champ ; le CHANGELOG porte l'avertissement aux clients d'API ; aucune tentative de sémantique « conserver » n'est faite ici.

- **D6 — Le compte de charge (achat) est HORS PÉRIMÈTRE.** #144 le pose comme *« optionnel / à discuter »*. Il n'existe **aucun** chemin entre le catalogue et les factures fournisseur : `supplier_invoice_lines.expense_account_id` est une FK **obligatoire** saisie à la main (`20260628000001_supplier_invoices.sql:79-96`), et aucun sélecteur d'article n'existe sur ce formulaire. Livrer le champ sans son chemin produirait une colonne que rien ne lit — le mode d'échec que 16-1c a documenté sous le nom de « code mort qui paraît fonctionner ». **À rouvrir en story dédiée si le besoin se confirme.**

- **D7 — Aucune reprise rétroactive.** Arbitrage de kickoff. La migration est du **DDL pur** : aucun `UPDATE`, aucun `INSERT`. Les articles existants naissent à `NULL`, ce qui préserve strictement le comportement actuel.

- **D10 — Les écritures de journal sont HORS PÉRIMÈTRE, et c'est une exclusion, pas un oubli.** #144 demande le compte « proposé par défaut lors de l'ajout du produit sur **une facture/écriture** ». Cette story ne couvre que la facture. **Motif, identique à D6** : `JournalEntryForm.svelte` ne porte **aucun** sélecteur d'article — `ProductPicker` n'est importé que par `InvoiceForm` — donc il n'existe aucun geste « ajouter un produit à une écriture » à pré-remplir. Brancher le champ là où rien ne le déclenche produirait du code mort. **À rouvrir en story dédiée si un sélecteur d'article arrive un jour sur la saisie d'écriture.** *(Exclusion écrite en passe 3 : les deux Change Logs antérieurs affirmaient « les six critères de #144 sont couverts » alors que le troisième l'est à moitié.)*

- **D8 — La frontière de la fenêtre d'upgrade RESTE à 34 : bumper `total` ET la fenêtre.** La nouvelle migration casse volontairement `upgrade_path_preserves_data` (`crates/kesh-db/tests/migrations_upgrade_path.rs:88-92`, l'assertion elle-même étant à `:89`), dont `assert_eq!(total, 57)` passe à **58**. La fenêtre s'écrit `total - 23`, ce qui donne aujourd'hui 34 migrations montées. **Laisser `- 23` avec `total = 58` déplacerait la frontière à 35** — le test cesserait de vérifier l'upgrade depuis l'état historique qu'il a été écrit pour couvrir, et ce glissement se répéterait à chaque migration. Elle passe donc à **`total - 24`**.

  **Motif** : la frontière 34 matérialise un état historique du schéma, pas une distance au présent. Une fenêtre qui rétrécit d'un cran à chaque migration finit par ne plus tester aucun upgrade réel — c'est la version lente du mode d'échec que le garde-fou **P6** existe pour attraper. L'assertion de montage (`:35-46`) énonce elle-même les deux branches de l'arbitrage ; celle-ci est tranchée ici plutôt que laissée au développeur. *(Décision ajoutée en passe 1 de `validate`, qui a relevé que l'AC exigeait un choix sans fournir de critère.)*

---

## Règle de splitting préventif — évaluation et arbitrages

*(Section ajoutée en passe 1 de `validate`, qui a relevé que le seuil n'avait jamais été évalué. Il aurait dû l'être **avant** de lancer `bmad-create-story`.)*

Le premier critère de la § « Règle de splitting préventif » est **plus de 5 modules distincts**. Recompté à la granularité que 16-1a et 16-1b ont elles-mêmes utilisée pour justifier leur split, cette story en touche **huit** *(sept dans la rédaction d'origine : la passe 3 a établi que doc-sync compte comme module chez 16-1b, qui l'inscrit dans ses cinq)* :

| # | Module | Nature de la touche |
|---|---|---|
| 1 | `crates/kesh-db/migrations` | 1 fichier neuf, DDL pur |
| 2 | `crates/kesh-db` entités + `repositories/products` | champ + 2 helpers + `COLUMNS` |
| 3 | `crates/kesh-api/routes/products` | 3 DTO + validation |
| 4 | `crates/kesh-api/exports/csv_tables` | **une ligne** d'en-tête + une de valeur |
| 5 | `crates/kesh-i18n/locales` | quelques clés × 4 locales |
| 6 | `frontend` page catalogue + `features/products` | sélecteur + types + client API |
| 7 | `frontend/components/invoices/InvoiceForm.svelte` | **une ligne** de pré-remplissage |
| 8 | doc-sync — CHANGELOG, README, manuel LaTeX | compté comme module chez 16-1b |

**Le seuil est franchi. Deux lectures s'opposaient, et l'arbitrage appartenait au Project Lead — il est rendu ci-dessous, puis reconduit en passe 3.**

- **En faveur du split** : la règle est un seuil, pas une appréciation, et elle existe précisément parce que l'estimation « c'est petit » est le biais qu'elle corrige. 16-1 a été splittée sur ce même critère et s'en est bien portée.
- **En faveur d'une dérogation** : le décompte est gonflé par des modules touchés d'**une seule ligne** (#4 et #7) — **deux sur huit**, et non deux sur sept comme l'annonçait la rédaction d'origine. Surtout, un split naturel « backend / frontend » séparerait la colonne de son **unique consommateur** — le pré-remplissage — et livrerait une sous-story dont rien ne lit la donnée, c'est-à-dire exactement le « code mort qui paraît fonctionner » que **D6** invoque pour exclure le compte de charge. La substance est ici : une colonne, un sélecteur, une ligne de recopie.

### Dérogation règle de splitting — arbitrage rendu

**Guy, 2026-08-03 : DÉROGATION.** La story reste unique.

**Justification retenue** : deux des sept modules sont touchés d'**une seule ligne** (#4 export CSV, #7 pré-remplissage), et le seul split naturel — backend / frontend — séparerait la colonne de son **unique consommateur**, livrant une sous-story dont rien ne lit la donnée. C'est le « code mort qui paraît fonctionner » que **D6** invoque pour exclure le compte de charge : appliquer la règle ici produirait exactement le défaut qu'elle cherche à prévenir ailleurs.

**Risque accepté** : si une passe `N+1` de `validate` remonte une sévérité **égale ou supérieure** à la passe `N`, le second critère de la règle se déclenche et l'arbitrage est **rouvert** — sans discussion. C'est le garde-fou qui rend cette dérogation tenable.

#### Garde-fou déclenché en passe 3 — arbitrage reconduit

**Le garde-fou a fonctionné : la passe 3 est remontée de `1 HIGH / 3 MED` à `5 HIGH / 6 MED`, et l'arbitrage a été rouvert.**

**Guy, 2026-08-03 : DÉROGATION RECONDUITE.** La story reste unique.

**Motif** : la passe 3 diagnostique elle-même ses findings comme des **résidus de remédiation** et des **preuves manquantes**, non comme des défauts de périmètre. Le finding le plus lourd — T6 comptant trois mutations quand AC10 en exige cinq — est un symptôme non propagé, que scinder le document ne corrigerait pas ; il le dupliquerait. Les trois HIGH sur la preuve (export CSV, `fetchAccounts(true)`, `#[serde(default)]`) désignent des **tests absents**, pas une story trop large.

⚠️ **Cette reconduction consomme le garde-fou : il ne joue plus.** Si une passe 4 remonte encore, le signal ne sera plus écartable et le split devient l'issue par défaut. C'est le prix de la reconduction, et il est inscrit ici pour que la passe suivante n'ait pas à le redécouvrir.

*(Le décompte de modules est par ailleurs corrigé de sept à **huit** ci-dessus : la passe 3 a établi que doc-sync compte comme module à la granularité de 16-1b. L'arithmétique qui portait la première dérogation — « deux sur sept » — devient « deux sur huit », et l'argument s'en trouve affaibli sans être renversé.)*

---

## Acceptance Criteria

- **AC1 — Schéma** (D2). Migration ajoutant `products.default_revenue_account_id BIGINT NULL`, contrainte `fk_products_default_revenue_account` vers `accounts(id)` `ON DELETE RESTRICT`, et `INDEX idx_products_default_revenue_account`. **DDL pur** — la migration ne contient aucun statement d'écriture de données.

- **AC2 — Entité et repository.** `Product`, `NewProduct` et `ProductUpdate` portent le champ `default_revenue_account_id: Option<i64>`. **DEUX listes de colonnes écrites à la main, qui ne dérivent pas l'une de l'autre** : la constante `COLUMNS` (`repositories/products.rs:22-23`) **et** `FIND_BY_ID_SCOPED_SQL` (`:26`), seconde chaîne littérale indépendante alimentant **six** `query_as::<_, Product>` (`:168`, `:202`, `:275`, `:332`, `:374`, `:416`). Les deux l'incluent, ainsi que l'`INSERT` (`:146`) et l'`UPDATE` (`:311`). ⚠️ `Product` dérive `sqlx::FromRow` : un champ absent du `SELECT` produit un `ColumnNotFound` **à l'exécution**, pas à la compilation — donc **toutes** les routes produits en 500. *(La seconde liste manquait à la rédaction d'origine, qui se présentait pourtant comme l'inventaire exhaustif : relevé en passe 3.)*

  ⚠️ **DEUX helpers écrits À LA MAIN doivent être étendus, et les oublier produit deux défauts SILENCIEUX distincts.** Aucun des deux ne dérive de la struct : ils énumèrent les champs un par un, donc le compilateur ne dira rien.

  - `is_no_op_change` (`:256-261`) compare **quatre** champs. Non étendu, une modification portant **uniquement** sur le compte est prise pour un no-op : l'`UPDATE` est court-circuité, la `version` ne bouge pas, **aucun audit n'est écrit — et l'API répond succès**. L'utilisateur voit son choix accepté et rien n'est enregistré.
  - `product_snapshot_json` (`:32-43`) énumère **huit** clés (`id`, `companyId`, `name`, `description`, `unitPrice`, `vatRate`, `active`, `version`). Non étendu, l'audit `{before, after}` de `product.updated` rend deux objets **identiques** pour un changement de compte : la piste d'audit affirme qu'il ne s'est rien passé. C'est plus grave qu'une absence d'entrée — c'est une entrée qui ment.

  Chacun des deux vient avec son test : modifier le seul compte doit bumper la `version`, écrire l'audit, et l'audit doit montrer `before ≠ after` sur ce champ.

- **AC3 — API.** `CreateProductRequest` et `UpdateProductRequest` acceptent `defaultRevenueAccountId` (camelCase, `Option<i64>`, `#[serde(default)]`) ; `ProductResponse` le restitue. Le compte est validé selon **D3** — **trois** critères, `postable` exclu ; un compte invalide rend **400** avec le code applicatif **`PRODUCT_REVENUE_ACCOUNT_INVALID`** — nommé ici pour qu'un test l'asserte au lieu de constater ce que l'implémentation vient d'inventer — et le motif du rejet, en réutilisant l'enum `RevenueAccountRejection` (`crates/kesh-db/src/errors.rs:14-27`) plutôt qu'en dupliquant ses variantes.

  ⚠️ **Le message rendu exige un TROISIÈME sujet.** Le formateur de `crates/kesh-api/src/errors.rs:75-86` ne connaît que deux sujets : une **ligne de facture** (`Some(n)`) ou **« le compte de produit par défaut de la société »** (`None`). Une fiche produit n'est ni l'un ni l'autre : réutiliser tel quel ferait lire, à qui édite un article, *« le compte de produit par défaut de la société : le compte 3400 est archivé »* — un message qui désigne **un autre objet** et envoie corriger un réglage non touché. Ajouter une clé Fluent + fallback dans la famille `invoice-line-account-subject-*`, **côté `kesh-api`**, dans les 4 locales.

  ⚠️ **`#[serde(default)]` n'est pas cosmétique** : sans lui, un `Option<T>` reste **obligatoire dans le JSON** en serde, et l'omission de la clé fait échouer la désérialisation — cassant **toute intégration API existante**, y compris celles qui ignorent le nouveau champ. Deux tests l'exigent en T6.

- **AC4 — Fiche produit (frontend).** Le formulaire de `products/+page.svelte` porte un sélecteur de compte, alimenté par **`fetchAccounts(true)`** — le flag `includeArchived` est **obligatoire** : sans lui un compte archivé n'est plus résoluble et le champ **paraît vide** au lieu d'être signalé, piège qui a coûté une passe de revue à 16-1b et qui ne se manifeste que le jour où un compte est archivé. Props `requiredAccountType="Revenue"` et `allowClear`. À l'édition le champ est hydraté depuis la réponse serveur ; à la soumission il est **toujours** envoyé (valeur ou `null`), jamais omis — cf. **D5**. Le champ est **facultatif** : une fiche sans compte s'enregistre sans friction.

- **AC5 — Pré-remplissage.** `onProductSelect` (`InvoiceForm.svelte:420-430`) pose `revenueAccountId: p.defaultRevenueAccountId ?? null`. **Le commentaire d'ancrage — littéralement `// AC9-bis — site 3/5. Idem : 16-2 y branchera le compte du produit.` — est remplacé** par la description du comportement livré. Le site `addFreeLine` (`:407-418`) **reste à `null`** — une ligne libre ne vient d'aucun article ; son commentaire d'ancrage est corrigé lui aussi, faute de quoi il annoncerait un travail déjà fait.

- **AC6 — Un compte devenu inutilisable est signalé, pas caché** (D4). Une facture montée depuis un article dont le compte est archivé affiche le libellé, le marqueur d'invalidité, et refuse l'enregistrement en nommant la ligne. **Aucun code nouveau n'est attendu ici** : l'AC vérifie que le pré-remplissage passe bien par la machinerie de 16-1, et le test qui le prouve est le livrable.

- **AC7 — Export CSV.** `serialize_products_csv` (`crates/kesh-api/src/exports/csv_tables.rs:364-397`) porte une liste de colonnes **codée en dur** ; elle inclut la nouvelle colonne, à sa position dans `COLUMNS`.

- **AC8 — i18n.** Les libellés **énumérés ici** existent dans les **4** locales — un critère portant sur un ensemble non énuméré n'est ni cochable ni réfutable :

  1. l'**étiquette** du sélecteur sur la fiche produit (`product-form-…`) ;
  2. son **texte d'aide** disant que laisser vide fait suivre le compte par défaut de la société ;
  3. le **troisième sujet** du message de rejet exigé par AC3 (`invoice-line-account-subject-…`, côté `kesh-api`).

  Tous (`crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`), sous le préfixe `product-` **tant qu'ils sont consommés depuis la page catalogue**, qui vit hors du périmètre du lint. ⚠️ **Tout libellé qui finirait consommé depuis un composant partagé sous `src/lib/features/` doit porter un préfixe global** (`error`, `tooltip`, `common`, `mode`, `shortcut`, `demo`) — sans quoi `lint-i18n-ownership` échoue. Le passer **en prop** depuis la page évite la question ; c'est l'erreur exacte commise en 16-1b, où une clé a dû être renommée après un lint rouge. Détail en Dev Notes. ⚠️ Les locales vivent dans un **crate Rust** : le gate inclut donc `cargo test --workspace` même pour un changement perçu comme frontend.

- **AC9 — Garde-fous de migration.** Les **quatre** obligations de la § « Migration breaking policy » sont tenues, chacune **recomptée à la source** :
  - **P5** — `docs/migrations-idempotence-audit.md` porte une ligne pour la nouvelle migration, et les compteurs sont **recomptés** : l'en-tête `## Table d'audit (N migrations)` **et** la ligne `Total` passent de **57** à **58**, la somme des trois compteurs de partition — `yes`, `tracked-by-sqlx`, `no` — égale ce total. *(Les trois compteurs de partition ne valent pas le total — les aligner dessus casserait l'invariant qu'ils servent à tenir.)*
  - **P6** — la nouvelle migration **casse volontairement** `upgrade_path_preserves_data` : `crates/kesh-db/tests/migrations_upgrade_path.rs:88-89` porte `assert_eq!(total, 57)`. Le porter à **58** **et** passer la fenêtre à `total - 24`, conformément à **D8** : la frontière reste à **34**. Ne pas se contenter de bumper `total`, ce qui déplacerait la frontière à 35 et ferait rétrécir la couverture d'un cran à chaque migration future.
  - **P7** — la migration étant du **DDL pur**, elle ne déclenche pas le détecteur de backfills et n'a besoin **ni** d'entrée au registre `POST_RESTORE_BACKFILLS` **ni** d'exemption. ⚠️ **Le vérifier en exécutant `every_data_backfill_migration_is_triaged`**, pas en le supposant : le détecteur classe sur le premier mot-clé du statement et sa largeur est précisément ce qui a échoué deux fois pendant la spécification de 16-1c.
  - **P1/P2** — `ADD COLUMN` nullable ⇒ **non-breaking** ⇒ **aucun** bump de `kesh_version_min_required`, donc aucun bump de version Cargo (P2-bis).

- **AC10 — Discrimination prouvée par mutation.** Au moins **cinq** mutations exécutées et consignées au Dev Agent Record :
  1. `onProductSelect` repose `revenueAccountId: null` au lieu du compte de l'article → **les tests du pré-remplissage** doivent rougir — l'unitaire **et** l'E2E, T6 exigeant les deux sur ce comportement — **et aucun autre cas**. *(Une rédaction antérieure exigeait « un seul test rouge », insatisfiable par construction : relevé en passe 3.)*
  2. La validation D3 est retirée du `create`/`update` produit → le test du compte invalide doit rougir.
  3. Le champ est retiré du **payload HTTP** envoyé par la fiche produit → seul un test qui traverse réellement HTTP peut l'attraper. *(C'est la mutation la plus instructive de 16-1b : ni Vitest ni les tests Rust ne voient une clé qui disparaît entre les deux.)*
  4. Le champ est retiré de **`is_no_op_change`** → le test « modifier le seul compte bumpe la `version` » doit rougir. Sans cette mutation, rien ne prouve que le court-circuit no-op a été étendu — et son échec est **muet** : l'API répond succès.
  5. Le champ est retiré de **`product_snapshot_json`** → le test « l'audit montre `before ≠ after` sur le compte » doit rougir. Sans cette mutation, rien ne prouve que la piste d'audit dit vrai.

  ⚠️ **Les mutations 4 et 5 sont ajoutées en passe 2**, sur convergence de deux lentilles : AC2 désignait ces deux helpers comme les défauts les plus graves de la story, et **aucune des trois mutations d'origine ne les discriminait**. Une spec qui nomme un risque sans le faire prouver ne l'a pas traité.

  **Un test attendu qui ne rougit pas invalide le montage** : le corriger avant d'aller plus loin.

- **AC11 — Gate.** `scripts/test-fast.sh` complet **et** le gate frontend (`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`) **et** l'E2E Playwright (`cd frontend && npm run test:e2e` ; **pré-requis** : MariaDB démarré, seed CI appliqué, et `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` sur Ubuntu 26.04+ — sans lui l'installation échoue en donnant l'impression que l'E2E n'est pas exécutable), sur l'**état final**, exit 0 exigé, non présumé d'un run antérieur.

  ⚠️ **Cette story touche `kesh-db`** (migration + repository) : la § « Test Locally First » → « Pendant une boucle de revue » **interdit le gate ciblé** ici, y compris entre deux passes de revue. Gate complet à chaque fois.

- **AC12 — Documentation synchronisée.** Le **CHANGELOG** porte la fonctionnalité **et** l'avertissement aux clients d'API de **D5** (le `PUT` full-replace efface un `defaultRevenueAccountId` omis).

  ⚠️ **Le manuel utilisateur promet DÉJÀ ce champ, et décrit un repli que cette story change.** `docs/manual/fr/user-manual.tex:553` documente « **Compte produit par défaut** : par exemple 3000 » **depuis la Story 11-0**, pour un code où il n'existe pas ; et `:583` décrit la chaîne de repli à **deux** maillons (ligne → société), que **AC5** fait passer à **trois** (ligne → **article** → société). Le manuel est donc à **corriger**, pas à compléter : dire que le champ est **facultatif**, décrire le pré-remplissage, et dire ce que signifie le laisser vide. **PDF régénéré (`make fr`) et commité.** *(Obligation reprise de 16-1b **AC17**, où elle était un critère cochable ; la reléguer en tâche conditionnelle « si le geste est visible » invite à répondre non sur un geste qui l'est par construction.)*

  La **feuille de route du `README`** est vérifiée : l'Epic 16 reste « 🚧 En cours » tant que #144 et #151 ne sont pas tous deux livrés, et la section « Fonctionnalités » ne doit pas annoncer comme acquis ce qui ne l'est pas. *(Critère ajouté en passe 1 de `validate` : l'obligation CHANGELOG ne découlait que d'une décision et d'une tâche mal étiquetée, donc d'aucun critère cochable — et la vérification README, que la story sœur 16-1b portait en AC18, avait disparu.)*

---

## Tasks / Subtasks

- [ ] **T1 — Migration et garde-fous** (AC1, AC9)
  - [ ] `crates/kesh-db/migrations/<timestamp>_products_default_revenue_account.sql` — `ADD COLUMN` + FK `ON DELETE RESTRICT` + index. DDL pur.
  - [ ] Ligne au tableau de `docs/migrations-idempotence-audit.md`, **compteurs recomptés** (57 → 58 aux deux sites, partition cohérente).
  - [ ] `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` — inspecter **chaque** site ; porter `assert_eq!(total, 57)` à **58** **ET** la fenêtre `total - 23` à **`total - 24`**, conformément à **D8**. ⚠️ Bumper `total` seul déplacerait la frontière de 34 à 35, **silencieusement** : le test repasse au vert et la couverture d'upgrade rétrécit d'un cran.
  - [ ] Exécuter `every_data_backfill_migration_is_triaged` pour **constater** que le DDL pur ne déclenche rien.
- [ ] **T2 — Backend : entité, repository, API** (AC2, AC3)
  - [ ] Champ dans `Product` / `NewProduct` / `ProductUpdate`, `COLUMNS`, `INSERT`, `UPDATE`.
  - [ ] **Étendre les DEUX helpers écrits à la main** — `is_no_op_change` (`:256-261`) **et** `product_snapshot_json` (`:32-43`). Cf. AC2 : les oublier produit deux défauts silencieux distincts, et **aucun des deux ne fait échouer la compilation**.
  - [ ] **Site de construction hors périmètre thématique** : `NewProduct` est aussi construit en littéral dans `crates/kesh-db/tests/kf005_fulltext_index_e2e.rs:123` (test de l'index full-text KF-005). La struct ne dérive pas `Default`, donc l'ajout du champ **casse la compilation** de ce fichier — échec bruyant, mais site à toucher que rien d'autre ne signale.
  - [ ] Validation D3 sur `create` et `update`, en **réutilisant** `RevenueAccountRejection` — ne pas dupliquer l'enum.
  - [ ] DTO d'API + `ProductResponse`, contrat camelCase.
- [ ] **T3 — Frontend : fiche produit** (AC4, AC8)
  - [ ] Sélecteur de compte dans le formulaire, hydraté à l'édition, **toujours** envoyé à la soumission, avec un `<label for>` et sa clé i18n — **les quatre champs existants en portent un** (`+page.svelte:586`, `:598`, `:609`, `:622`), ce serait le seul sans.
  - [ ] ⚠️ **Prouver le flag `includeArchived`, ne pas se contenter de l'écrire.** 16-1b a **mesuré** que l'AC qui se présentait comme le garde-fou de ce piège n'en était pas un : le mock rend la liste complète quel que soit l'argument. **Seule** une assertion `expect(fetchAccounts).toHaveBeenCalledWith(true)` attrape la mutation `fetchAccounts(true)` → `fetchAccounts()`. Le piège s'est **rejoué** en revue de code de 16-1b, fermé seulement par un E2E archivant un compte après coup.
  - [ ] Types TS, client API, clés i18n dans les **4** locales.
- [ ] **T4 — Frontend : pré-remplissage** (AC5)
  - [ ] `onProductSelect` pose le compte de l'article ; `addFreeLine` reste à `null`.
  - [ ] **Mettre à jour le doc-comment de `ProductPicker.svelte:1-4`**, qui énumère le snapshot recopié (« name/unitPrice/vatRate ») et devient faux dès que le compte s'y ajoute. *(Site trouvé par le grep de propagation, pas par les lentilles.)*
  - [ ] **Remplacer les deux commentaires d'ancrage** — les chercher sur `16-2 y branchera`, et non sur « la Story 16-2 y branchera » qui n'existe pas tel quel (`:413-414` et `:426`) par la description du comportement livré.
- [ ] **T5 — Export CSV** (AC7)
  - [ ] Étendre `serialize_products_csv`, en-tête **et** valeurs — **deux listes écrites à la main que rien ne relie** (`:368-379` et `:381-392`) : un décalage entre elles rend le CSV **silencieusement faux sur toutes les colonnes suivantes**.
  - [ ] **Étendre le test d'export** `crates/kesh-api/tests/exports_global_e2e.rs` : sa fixture porte aujourd'hui `("products.csv", 0)` — **zéro ligne de produit**, donc l'en-tête n'est observé par rien (`grep -rnF "unit_price,vat_rate" crates/` → aucune sortie). Semer au moins un produit **avec** un compte, et asserter en-tête **et** valeur. *(Complément qu'exigeait 16-1a en AC14.)*
- [ ] **T6 — Tests et preuve** (AC6, AC10)
  - [ ] Tests d'intégration `kesh-db` : le champ **persiste et se relit** ; **modifier le SEUL compte bumpe la `version` et écrit l'audit** (test de la mutation 4) ; **l'audit montre `before ≠ after` sur le compte** (test de la mutation 5) ; une modification qui ne change rien reste un no-op.
  - [ ] **Tests au niveau de la ROUTE, pas du repository** : la validation **D3** vit dans `crates/kesh-api/src/routes/products.rs` et rend un **400**. Un test d'intégration `kesh-db` ne l'exerce pas — et la **mutation 2 ne rougirait pas**. Y placer le rejet d'un compte inactif, d'un compte non-`Revenue`, et le scoping multi-tenant.
  - [ ] **Non-régression du contrat HTTP** (AC3) : deux tests — payload **sans la clé** `defaultRevenueAccountId`, et payload avec `"defaultRevenueAccountId": null` — les deux valent `NULL` et n'échouent jamais en 400. Sans `#[serde(default)]`, l'omission casse **toute intégration API existante**, y compris celles qui ignorent le nouveau champ. *(Exigence reprise de 16-1a AC6.)*
  - [ ] **D9** : modifier le nom d'un article dont le compte a été **archivé entre-temps** doit réussir. C'est le test qui prouve que la validation ne se déclenche que sur changement.
  - [ ] Test unitaire frontend du pré-remplissage.
  - [ ] **E2E** (suffixe `.spec.ts` **obligatoire**) : parcours « fiche produit avec compte → facture depuis catalogue → la ligne porte le compte », **plus** le cas AC6 — assigner un compte, l'**archiver ensuite**, monter une facture depuis l'article, constater le marqueur et le refus d'enregistrement.
  - [ ] **Scoping multi-tenant** : un test refuse un compte appartenant à une **autre** société (`UnknownOrCrossCompany`). L'enum le couvre, mais rien ne le prouve tant qu'aucun test ne l'exerce.
  - [ ] Les **cinq** mutations d'AC10, exécutées, consignées avec leur sortie, fichiers restaurés à l'identique.
- [ ] **T7 — Documentation** (AC12)
  - [ ] CHANGELOG : la fonctionnalité **et** l'avertissement `PUT` full-replace de D5.
  - [ ] `README.md` — feuille de route et section « Fonctionnalités » vérifiées (même si la conclusion est « rien à changer »).
  - [ ] **Manuel utilisateur** (`docs/manual/fr/user-manual.tex`) : corriger `:553` (champ **facultatif**) et `:583` (repli à trois maillons), décrire le pré-remplissage. **`make fr` puis commiter les PDF.** Ce n'est pas conditionnel — cf. AC12.
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

**Statut à la création : `ready-for-dev`, non validée** — la boucle `validate` n'avait pas encore tourné. *(Elle a tourné depuis : voir les passes ci-dessous. Le statut courant est en tête de document.)*

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

### Passe 3 de `bmad-create-story validate` — **GARDE-FOU DE LA DÉROGATION DÉCLENCHÉ**

**2026-08-03 — Opus, trois lentilles, contexte frais. 0 CRITICAL, 5 HIGH, 6 MEDIUM, 5 LOW.** Rotation Sonnet → Haiku → Opus.

⚠️ **La sévérité REMONTE** : passe 2 retenait `1 HIGH / 3 MED`, la passe 3 en rend `5 HIGH / 6 MED`. Le second critère de la § « Règle de splitting préventif » — *« une passe `N+1` remonte une sévérité **égale ou supérieure** à la passe `N` »* — **se déclenche**. Conformément au risque accepté inscrit dans la dérogation, **l'arbitrage est rouvert**. Les findings ci-dessous sont consignés **avant** patch : les corriger dans un document qui serait ensuite splitté serait du travail jeté.

**Le HIGH le plus embarrassant est une récidive littérale.** Le `CRITICAL` de la passe 1 disait : *« AC2 exige deux helpers, T2 n'en cite qu'un — un dev travaillant tâche par tâche coche T2 en ayant reproduit le défaut »*. La passe 2 a porté AC10 de **trois** à **cinq** mutations… **sans toucher T6, qui compte encore « les trois mutations d'AC10 »**. Même défaut, même mécanisme, un cran plus loin, introduit par la remédiation qui corrigeait le premier. Le symptôme « décompte de mutations » n'a pas été grepé sur le document — c'est exactement la § « Propagation post-patch » de `CLAUDE.md`, non appliquée.

**Le HIGH le plus grave sur le fond : D3 laisse ouvert sur `active` ce qu'elle a fermé sur `postable`.** Le scénario qui a fait retirer `postable` — un administrateur modifie un compte par un flux sans rapport, l'utilisateur renomme l'article, l'`UPDATE` se fait rejeter sur un champ non touché — s'applique **mot pour mot** à l'archivage d'un compte, action **bien plus fréquente**. Et l'issue est pire : un compte archivé est **absent des propositions** du sélecteur (`AccountAutocomplete.svelte:165-172` filtre sur `a.active`), donc l'utilisateur ne peut **ni conserver ni remplacer** la valeur — seul `allowClear` le sort de l'impasse, en perdant l'imputation. Aggravant : **le test E2E qu'exige T6 crée délibérément cet état**.

**Trois HIGH sur la preuve, tous appuyés sur un précédent mesuré du dépôt :**

- **AC7 (export CSV) n'est discriminé par rien.** `grep -rnF "unit_price,vat_rate" crates/` → aucune sortie ; la fixture du seul test qui touche ce fichier porte `("products.csv", 0)` — **zéro ligne de produit**. Or `serialize_products_csv` écrit **deux listes indépendantes** (en-tête `:368-379`, enregistrement `:381-392`) que rien ne relie : un décalage entre elles rend le CSV **silencieusement faux sur toutes les colonnes suivantes**, gate vert. La story sœur 16-1a portait exactement le complément manquant en **AC14**.
- **Le piège `fetchAccounts(true)` est décrit deux fois et prouvé zéro fois.** 16-1b a **mesuré** que l'AC qui se présentait comme son garde-fou n'en était pas un — le mock rend la liste complète quel que soit l'argument, seule `toHaveBeenCalledWith(true)` attrape la mutation — et le piège s'est **rejoué** en revue de code, fermé seulement par un E2E archivant un compte après coup. 16-2 le décrit et n'en fait ni assertion, ni mutation, ni E2E.
- **AC3 exige `#[serde(default)]` sans le test de non-régression que 16-1a rendait obligatoire.** Sans l'attribut, l'omission de la clé fait échouer la désérialisation et **casse toute intégration API existante** — y compris celles qui ignorent le nouveau champ. La mutation 3 prouve que le *formulaire* envoie la clé, pas que le *backend* tolère son absence : les deux sont orthogonaux.

**Le cinquième HIGH : AC12 omet les manuels LaTeX, qui promettent DÉJÀ ce champ.** `docs/manual/fr/user-manual.tex:553` documente « **Compte produit par défaut** : par exemple 3000 » depuis la **Story 11-0**, pour un code où il n'existe pas ; et `:583` décrit une chaîne de repli à deux maillons (ligne → société) que **AC5 fait passer à trois** (ligne → article → société). La sœur 16-1b portait cette obligation en **AC17**, critère cochable ; 16-2 la relègue en puce conditionnelle (« **si** le geste est visible côté fiduciaire ») — c'est le défaut que la passe 1 a corrigé pour le CHANGELOG, réintroduit une puce plus bas dans la même tâche.

**Les six MEDIUM** : `FIND_BY_ID_SCOPED_SQL` est une **seconde** liste de colonnes écrite à la main, alimentant **six** `query_as::<_, Product>` — AC2 se présente comme exhaustif et l'omet, et l'échec est un `ColumnNotFound` à l'exécution ; réutiliser `RejectedRevenueAccount` fait dire au message « **le compte de produit par défaut de la société** » à quelqu'un qui édite une fiche produit, faute d'un troisième sujet dans `kesh-api` ; le décompte de modules de la dérogation est **minoré d'un** — chez 16-1b, doc-sync compte comme module, donc **huit** et non sept, ce qui affaiblit l'arithmétique « deux sur sept » qui porte la justification ; T6 range le test de la validation D3 dans `kesh-db` alors que la règle vit dans l'**API**, si bien que la mutation 2 ne rougirait pas ; AC10.1 exige qu'un seul test rougisse alors que T6 en demande **deux** sur le même comportement, exigence insatisfiable par construction ; et T1 fait porter au développeur l'arbitrage que **D8** a déjà tranché, sans en donner la valeur — un dev qui suit T1 bumpe `total` et laisse `- 23`, déplaçant la frontière à 35, **silencieusement** (le test repasse au vert).

**Les cinq LOW** : le titre de la section splitting annonce « arbitrage **en attente** » quand sa sous-section dit « arbitrage **rendu** » ; le `Status: ready-for-dev` contredit le Change Log qui se déclare non convergé ; le commentaire d'ancrage est cité sous un libellé (« la Story 16-2 y branchera ») qui n'existe pas tel quel — un `grep -F` ne rendrait rien ; `D3-bis` est invoquée trois fois et jamais énoncée, alors que c'est le motif central de D3 ; et « miroir strict » est faux sur le point qu'il sert à justifier — `company_invoice_settings` n'a **aucun** index sur sa colonne (`grep -nF "idx_company_invoice_settings"` → aucune sortie), l'index vient du patron 16-1a.

**Ce que la passe a validé positivement** : **vingt** affirmations des deux Change Logs recoupées et **toutes vraies**, dont l'arithmétique de D8, les huit clés, les quatre champs, les treize FK, les cinq marqueurs `AC9-bis` et le fait que `0ce6e13a` est bien le point de fork (`git merge-base`) ; le garde-fou de la dérogation est **fidèle au texte** de `CLAUDE.md` et non à la formulation qu'il remplace ; l'interdiction de gate ciblé est exactement ce que dit la règle ; le classement P7 en DDL pur est correct ; et plusieurs cas-limites sont **réfutés** plutôt que signalés — l'article archivé ne peut pas être choisi (`listProducts({ includeArchived: false })`), la suppression dure d'un compte n'a **aucune route**, le backup énumère ses colonnes dynamiquement, et les seeds ne créent aucun produit.

**Trend** : `1 CRIT / 3 HIGH / 4 MED / 4 LOW` → `0 / 1 HIGH / 3 MED / 2 LOW` → **`0 / 5 HIGH / 6 MED / 5 LOW`**.

**Statut : arbitrage rouvert puis reconduit ; les 16 findings sont appliqués — cf. la section de remédiation ci-dessous.**

#### Remédiation de la passe 3 — arbitrage reconduit, 16 findings appliqués

**Guy, 2026-08-03 : DÉROGATION RECONDUITE, patches appliqués.** Les 16 findings de la passe 3 sont traités ; le détail des décisions nouvelles est ci-dessous, celui des corrections ponctuelles dans le corps du document.

**Deux décisions nouvelles, et la première est la plus importante de la story.**

**D9 — la validation ne se déclenche que si le compte CHANGE.** C'était le trou que D3 laissait ouvert : le raisonnement qui avait fait retirer `postable` en passe 1 valait **mot pour mot** pour `active` et `account_type`, et l'archivage d'un compte est bien plus fréquent qu'un basculement de `postable`. Sans D9, renommer un article devenait impossible dès qu'un compte avait été archivé ailleurs — et sans issue, le compte archivé étant absent des propositions du sélecteur. Le critère retenu est local et décidable : comparer deux `Option<i64>`.

**D10 — les écritures de journal sont exclues, explicitement.** #144 demande le pré-remplissage « sur une facture **/ écriture** » ; les deux Change Logs antérieurs affirmaient « les six critères sont couverts » alors que le troisième l'était à moitié. Le motif d'exclusion est celui de D6 — `ProductPicker` n'est importé que par `InvoiceForm`, il n'existe aucun geste à pré-remplir sur la saisie d'écriture — mais il n'était pas écrit. Une exclusion tacite n'est pas une décision.

**Trois manques de preuve comblés**, chacun appuyé sur un précédent que le dépôt a payé : le test d'export CSV est étendu avec un produit **portant** un compte (sa fixture porte `("products.csv", 0)`, donc l'en-tête n'était observé par rien) ; le flag `includeArchived` exige désormais l'assertion `toHaveBeenCalledWith(true)`, seule à attraper la mutation — 16-1b l'a **mesuré**, son AC qui s'en disait le garde-fou n'en était pas un ; et `#[serde(default)]` vient avec ses deux tests de non-régression du contrat HTTP.

**Le manuel utilisateur passe de puce conditionnelle à obligation d'AC12.** Il promet ce champ **depuis la Story 11-0** pour un code où il n'existe pas, et décrit un repli à deux maillons que AC5 fait passer à trois. Il est donc à **corriger**, pas à compléter.

**Les corrections de propagation** : T6 comptait trois mutations quand AC10 en exige cinq — la récidive littérale du CRITICAL de la passe 1 — et nomme désormais les deux tests des helpers ; les tests de validation passent au niveau de la **route**, faute de quoi la mutation 2 ne rougirait pas ; T1 porte la valeur `total - 24` tranchée par D8, au lieu de laisser le développeur bumper `total` seul et déplacer la frontière **en silence** ; AC2 déclare la **seconde** liste de colonnes écrite à la main, `FIND_BY_ID_SCOPED_SQL`, dont l'oubli met toutes les routes produits en 500 ; AC3 nomme le code applicatif et exige un **troisième sujet** i18n, sans quoi l'utilisateur d'une fiche produit lirait un message désignant le réglage société ; AC8 énumère enfin les trois libellés à traduire ; AC10.1 n'exige plus « un seul test rouge », insatisfiable puisque T6 en demande deux sur ce comportement ; et AC11 porte le pré-requis `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE`.

**Les LOW** : le décompte de modules passe de sept à **huit** — doc-sync compte comme module chez 16-1b — ce qui fait de l'argument « deux touchés d'une ligne » un **deux sur huit** et l'affaiblit sans le renverser ; `D3-bis` est désormais énoncée dans le document plutôt qu'invoquée trois fois sans être définie ; « miroir strict » est corrigé — le réglage société n'a **aucun** index, celui-ci est exigé par InnoDB sur la colonne enfant d'une FK ; le commentaire d'ancrage est cité au caractère près, un `grep -F` sur l'ancienne formulation ne rendant rien ; le doc-comment de `ProductPicker` entre au périmètre de T4 ; et le `Status` du document dit enfin l'état réel — il annonçait `ready-for-dev` pendant que le Change Log se déclarait non convergé.

⚠️ **Le garde-fou de la dérogation est consommé.** Il s'est déclenché une fois et a été reconduit une fois. Si la **passe 4** remonte encore en sévérité, le signal ne sera plus écartable et le split devient l'issue par défaut.

**Prochaine : passe 4**, modèle différent d'Opus (rotation → Sonnet), contexte frais.

### Passe 4 de `bmad-create-story validate` — **GARDE-FOU DÉCLENCHÉ UNE SECONDE FOIS**

**2026-08-03 — Sonnet, trois lentilles, contexte frais. 0 CRITICAL, 5 HIGH, 4 MEDIUM, 4 LOW.**

⚠️ **La sévérité maximale reste à `HIGH` : le critère « passe `N+1` **égale ou supérieure** à `N` » se déclenche de nouveau.** La reconduction de la passe 3 avait explicitement consommé le garde-fou — « si la passe 4 remonte encore, le split devient l'issue par défaut ». C'est le cas.

**Le diagnostic de cette passe est différent des précédents, et il est plus dur pour le processus que pour la story.** Sur les 13 findings, **huit portent sur la comptabilité de mes propres Change Logs** :

- l'en-tête de la passe 3 déclare **5 HIGH** et en énumère **six** (T6-récidive, D3-`active`, AC7-CSV, `fetchAccounts`, `serde(default)`, AC12-manuels) — le sixième étant introduit sous le libellé « le cinquième HIGH » ;
- la liste « les cinq LOW » en contient **six**, dont un **repris du bloc MEDIUM** (le décompte de modules) et un (`ProductPicker`) qui ne figure dans **aucune** des trois listes ;
- **D10 et le pré-requis Playwright** sont présentés comme des corrections de la passe 3 sans qu'aucun finding déclaré ne les motive ;
- et le total « 16 findings appliqués » est donc construit sur ces chiffres.

**Une affirmation de mon Change Log est carrément fausse.** La passe 3 listait, parmi ses vérifications positives, « les seeds ne créent aucun produit ». Réfuté : `crates/kesh-db/src/test_fixtures.rs:482` fait `INSERT INTO products (…) VALUES (…, 'CI Product', …)`, et `seed_contact_and_product` est appelée par le preset E2E `with-data` (`routes/test_endpoints.rs:203`). **Tout scénario E2E de T6 utilisant ce preset trouvera donc un `'CI Product'` préexistant, au compte `NULL`** — exactement ce que le document affirme ne pas exister.

**Deux HIGH portent sur le fond, et ils convergent :**

- **D9 n'est propagée ni à AC3 ni à T2.** La décision « la validation ne se déclenche que si le compte change » n'existe que dans D9 et dans un *test* de T6. Le critère qui décrit la validation et la tâche qui l'implémente disent tous deux « validation D3 sur `create` et `update` », sans condition. **C'est le patron du CRITICAL de la passe 1, rejoué sur la décision que la passe 3 qualifiait elle-même de plus importante de la story** — quatrième récidive du même mode d'échec.
- **D9 n'est pas implémentable là où la spec la place, et le document ne le dit pas.** La comparaison exige l'état antérieur ; or `update_product` (`routes/products.rs:295-322`) ne lit aucun `before` — le seul existe **dans la transaction** du repository (`products.rs:275-298`). L'EdgeCaseHunter a trouvé le précédent qui rend la chose faisable (`company_invoice_settings.rs:148` fetche son état courant à la route, hors transaction, le verrou optimiste couvrant la fenêtre), mais **rien dans T2 ne l'indique**. Le placement naturel — le repository, où `before` est gratuit — contredirait T6, qui exige les tests au niveau route.

**Le troisième HIGH est un résidu de patch classique** : le décompte de modules corrigé « sept → huit » a été appliqué à la ligne 132 et **pas** à la ligne 138, celle qui porte la *justification retenue* de la dérogation. Le document affirme trois lignes plus bas que la correction est faite. C'est le nombre exact que ce document reproche par ailleurs à un commentaire de migration d'avoir laissé dériver.

**Les MEDIUM** : le **troisième sujet i18n** exigé par AC3 n'est discriminé par aucune mutation ni aucun test — un développeur peut réutiliser le formateur tel quel et rien ne rougirait, alors que le document traite ce même mode d'échec en HIGH quand il touche `fetchAccounts(true)` ; **D10 manque à la checklist « ce que cette story ne doit PAS faire »**, où figure pourtant sa jumelle D6 ; et le nom de l'enum est inversé dans un récit (`RejectedRevenueAccount` au lieu de `RevenueAccountRejection`).

**Les LOW**, tous ironiques : l'ancre de la seconde liste CSV est décalée (`:382-393`, pas `:381-392`) — une ancre **neuve** de la passe 3, pas un résidu ; et le motif de recherche que la passe 3 avait « corrigé » pour T4 (`16-2 y branchera`) ne trouve **qu'une** des deux ancres, la seconde étant scindée sur deux lignes (`grep -cF` → **1**).

**Ce que la passe a validé** : les **huit** modules, l'arithmétique de D8, les 13 FK, les 6 `query_as`, l'absence de `postable` dans le réglage société, le point de fork, `user-manual.tex:553`/`:583`, les quatre `<label for>`, la fixture `("products.csv", 0)`, le doc-comment de `ProductPicker`, et **la couverture complète de #144** — plus aucun engagement n'est ni couvert ni explicitement exclu, D10 ayant fermé le dernier. Les obligations de `CLAUDE.md` sont toutes portées.

**Trend** : `1 CRIT / 3 HIGH / 4 MED / 4 LOW` → `0 / 1 HIGH / 3 MED / 2 LOW` → `0 / 5-6 HIGH / 6 MED / 5 LOW` → **`0 / 5 HIGH / 4 MED / 4 LOW`**. Rotation Sonnet → Haiku → Opus → Sonnet, cycle complet.

**Statut : findings consignés, patches suspendus. Le garde-fou s'est déclenché deux fois de suite et a déjà été reconduit une fois — le split est l'issue par défaut, et la décision revient au Project Lead.**


### Split — arbitrage final

**Guy, 2026-08-03 : SPLIT.** Le garde-fou de la dérogation s'est déclenché **deux fois de suite** (passes 3 et 4, sévérité maximale `HIGH` maintenue) et avait déjà été reconduit une fois. Conformément à ce que la reconduction inscrivait — « si la passe 4 remonte encore, le split devient l'issue par défaut » — la story est scindée :

- **[16-2a](16-2a-compte-produit-catalogue-backend.md)** — migration et ses quatre garde-fous, colonne, les deux listes de colonnes, les deux helpers silencieux, validation D3 conditionnée par D4 avec son placement tranché, API et troisième sujet i18n, export CSV et son test. **Quatre** mutations.
- **[16-2b](16-2b-selecteur-et-prefill-frontend.md)** — sélecteur de fiche, pré-remplissage, types, i18n, doc-sync et manuels. **Trois** mutations.

**Les deux partent dans la même PR** : seule, 16-2a livre une colonne que rien ne lit — l'objection qui avait fait refuser le split deux fois, levée par la contrainte de PR et non par le split.

**Ce que le split corrige, et ce qu'il ne corrige pas.** Aucun défaut de conception : les quatre passes ont validé la conception, et les six critères de #144 sont couverts ou explicitement exclus. Ce qui ne convergeait pas était la **tenue du document** — ~460 lignes, cinq Change Logs, dix décisions, une comptabilité qui dérivait à chaque remédiation. **Huit des treize findings de la passe 4 portaient sur les décomptes de mes propres Change Logs**, pas sur la story : un en-tête annonçant 5 HIGH pour six énumérés, une liste de « cinq LOW » en contenant six dont un repris du bloc MEDIUM, deux corrections sans finding source, et une affirmation carrément fausse (« les seeds ne créent aucun produit », réfutée par `test_fixtures.rs:482`).

**Les 30+ findings des quatre passes sont tous reportés à l'état corrigé** dans l'une ou l'autre des filles — y compris les cinq HIGH de la passe 4, dont le placement de **D4** (irréalisable à la couche où la parente le mettait) et la propagation de cette décision aux critères et aux tâches, qui était la quatrième récidive du même mode d'échec.
