# Story 38-5 : code TVA par défaut d'un compte, extraction de la TVA, compte d'impôt sur les acquisitions

Status: ready-for-dev

*Fiche rédigée le 2026-11-15 (Opus), branche `epic/38-paypal`, numéros de ligne relevés sur `8b3d548d`. Epic 38
(`_bmad-output/planning-artifacts/epic-38-paypal.md`) : E4, E13, E18, E23, C-38-0, C-38-7, C-38-8 (consommateur),
C-38-13 ; décisions de Beat B6, B18 (`epics-38-40-cadrage.md`) ; *Point TVA pour Guy*. **Dépend de l'Epic 37** (37-1 :
`accounts.currency`, verrou des deux côtés C-37-9 par `DESIGNATED_ACCOUNT_COLUMNS`, liste close des absences admises
de 37-1 AC-6 ; 37-3a : produit CHF = round(montant × cours, 2), AC-3 ; 37-3b : patron de la fixture partagée Rust/TS).
**Ne dépend d'aucune autre story de l'Epic 38** : se développe en parallèle de 38-1 à 38-4. **Migration non breaking**
(DDL seul ; O1 : la seule migration breaking de l'epic est celle de 38-2, celle-ci n'en dépend pas). Aucune route neuve. Consommée par 38-6a (propositions) et 38-6b (remboursements).*

## Story

En tant que comptable d'une société assujettie à la TVA, je veux qu'un compte de produit ou de charge porte son
**genre** de TVA par défaut (TVA due incluse dans une vente, impôt préalable inclus dans un achat, impôt sur les
acquisitions d'un service acheté à l'étranger) et son **taux**, et désigner dans les réglages un **compte distinct de
TVA due sur les acquisitions**, afin que les propositions de Kesh séparent la TVA de mes ventes et achats PayPal sans
saisie — au taux en vigueur à la date de chaque transaction.

## Exigences

Reprises de la fiche d'epic (§ 38-5), complétées là où un AC l'exige.

- **FR**
  1. `FR-38-5-1` — Un compte de produit ou de charge porte un genre de TVA par défaut (`sale`, `purchase`,
     `acquisition`, aucun) et un taux. → AC-1, AC-3, AC-4, AC-8.
  2. `FR-38-5-2` — Kesh extrait la TVA d'un brut et calcule l'impôt sur les acquisitions (deux lignes égales). → AC-2.
  3. `FR-38-5-3` — Les réglages TVA désignent le compte d'impôt sur les acquisitions (B18). → AC-1, AC-6, AC-8.
  4. `FR-38-5-4` — Le taux appliqué est celui en vigueur à la date de la transaction. → AC-5.
  5. `FR-38-5-5` *(ajoutée par la fiche)* — Une seule lecture rend, pour un compte et une date, le traitement TVA à
     appliquer ou le motif qui l'empêche ; 38-6a et 38-6b n'en écrivent pas d'autre. → AC-5.
- **NFR**
  1. `NFR-38-5-1` — Arrondi au centime ; fonctions pures en Rust et miroir TS tenus par une fixture partagée. → AC-2.
  2. `NFR-38-5-2` — Lignes de TVA en devise principale pour un compte en devise. → AC-2.
  3. `NFR-38-5-3` — Migration non breaking. → AC-1.

## Écarts à l'epic

1. **« Taux » = une catégorie, résolue à la date.** E13 et C-38-7 posent `default_vat_rate_id` (FK `vat_rates`) ;
   FR-38-5-4 exige le taux en vigueur à la date de la transaction. Or une ligne de `vat_rates` porte **une** période
   (`valid_from`, `valid_to`, `20260428000001_vat_rates.sql:6-7`) et une catégorie (`normal`, `reduced`, `special`,
   `exempt`, `custom` — `20260613000001_vat_rates_crud.sql`, `entities/vat_rate.rs:47-53`), et l'API garantit au plus
   une ligne **active** par catégorie à une date (`has_overlap_in_category`, `repositories/vat_rates.rs:140-165`,
   appelé à la création et à la modification, `routes/vat.rs:322`, `:411`). La fiche lit donc le taux désigné comme
   **sa catégorie** : à la date D, le taux appliqué est la ligne de cette catégorie en vigueur à D
   (`find_for_category_at_date`, `repositories/vat_rates.rs:89-108`, existante). Un compte désigné « normal 8.1 % »
   prend 7.7 % pour une transaction de 2023 si la ligne 7.7 % existe ; sinon la résolution échoue (AC-5). La colonne
   reste `default_vat_rate_id` (contrat C-38-7).
2. **Codes neufs** que l'epic ne nomme pas : `DEFAULT_VAT_INVALID` (400, écriture d'un compte, AC-4) ; et, pour 38-6a /
   38-6b, deux motifs de résolution (`VAT_KIND_UNKNOWN`, `VAT_RATE_UNAVAILABLE`, AC-5) en **complément de C-38-8**
   — à reprendre tels quels par 38-6a, qui les rend en `FailedProposal`.
3. **Le onzième réglage désigné.** `default_vat_acquisition_account_id` entre à `DESIGNATED_ACCOUNT_COLUMNS`
   (`repositories/company_invoice_settings.rs:46-57`, dix aujourd'hui ; un test, `accounts_designated_archive_e2e.rs:1077`,
   rougit sinon). Ce qui suit la liste suit le réglage : l'avertissement d'archivage de 33-8 (#486) et le verrou des
   deux côtés de 37-1 AC-4 (`ACCOUNT_CURRENCY_IN_USE`, `ACCOUNT_IN_FOREIGN_CURRENCY`), qui compte « les dix » — le
   décompte de 37-1 passe à onze ; à écrire au Change Log de cette story et à signaler à la fiche 37-1 si elle n'est
   pas encore développée.
4. **Compte d'acquisitions distinct du compte de TVA due** : refusé s'il est égal (AC-6). B18 demande un compte
   distinct ; l'epic ne dit pas ce que fait Kesh d'un réglage égal. Le refuser garde le grand livre lisible (raison
   d'être de B18, *Point TVA pour Guy*).
5. **MCP** : `kesh_plan_comptable` n'expose pas le genre ni le taux par défaut (AC-9) — exclusion justifiée, tracée au
   registre C-38-13.

## Critères d'acceptation

1. **AC-1 — Migration** `<AAAAMMJJ>000001_default_vat_of_accounts.sql` (version strictement postérieure à toute
   migration de la branche au moment de la fusion ; cette story pouvant fusionner avant 38-3a ou 38-4, son numéro se
   relit à la fusion et le fichier, **jamais appliqué hors des bases de dev**, se renomme alors délibérément — P8 :
   bases de dev reconstruites), **non breaking**, DDL seul :
   - `ALTER TABLE accounts ADD COLUMN default_vat_kind VARCHAR(16) CHARACTER SET ascii COLLATE ascii_bin NULL, ADD
     COLUMN default_vat_rate_id BIGINT NULL, ADD CONSTRAINT fk_accounts_default_vat_rate FOREIGN KEY
     (default_vat_rate_id) REFERENCES vat_rates(id) ON DELETE RESTRICT, ADD CONSTRAINT chk_accounts_default_vat_kind
     CHECK (default_vat_kind IS NULL OR BINARY default_vat_kind REGEXP '^[a-z_]{1,16}$'), ADD CONSTRAINT
     chk_accounts_default_vat_pair CHECK ((default_vat_kind IS NULL) = (default_vat_rate_id IS NULL))` — format seul
     pour le genre (E4 : colonne lue avec `Other(String)`, aucune énumération au schéma, précédent A28 de 37-2) ; le
     couple est tout ou rien ;
   - `ALTER TABLE company_invoice_settings ADD COLUMN default_vat_acquisition_account_id BIGINT NULL, ADD CONSTRAINT
     fk_cis_vat_acquisition FOREIGN KEY (default_vat_acquisition_account_id) REFERENCES accounts(id) ON DELETE
     RESTRICT` — patron des trois comptes de TVA (`20260614000001_vat_accounts_config.sql:1-10`) ; **aucun compte
     créé** (à la différence de ce précédent, qui insérait 1171 et 2206 : aucune donnée écrite, B18 laisse le choix
     du compte, p. ex. 2201, à l'utilisateur).
   **Non breaking** (P1 : colonnes `NULL`, FK, `CHECK` sur des colonnes neuves ; aucune valeur neuve dans une colonne
   existante à domaine fermé — C-38-0). **P7** : aucune donnée écrite (`ALTER` seul). **P5** : une ligne à
   `docs/migrations-idempotence-audit.md` (`tracked-by-sqlx`), en-tête et ligne `Total` recomptés à la source,
   compteurs de partition recomptés depuis le tableau. **P6** : grep `migrations.len()\|apply_migrations_up_to`
   exécuté, sites inspectés, au Change Log. **P8** : aucune migration appliquée retouchée. Squash de test régénéré
   (`scripts/regen-test-schema.sh`). Ordre de `TABLES_TO_TRUNCATE` (`kesh-db/src/backup.rs:46`) : `accounts` (`:105`)
   est vidée **avant** `vat_rates` (`:110`) et restaurée après elle (ordre inverse) — la FK neuve est satisfaite sans
   changer la liste ; `company_invoice_settings` (`:95`) avant `accounts`, comme ses dix autres FK. Le commentaire de
   `vat_rates` à `backup.rs:110` (« aucune table ne référence vat_rates ») devient **faux** : réécrit (« référencée
   par `accounts.default_vat_rate_id`, RESTRICT, Story 38-5 ») ; de même le doc-comment de
   `find_for_category_at_date` (`repositories/vat_rates.rs:81-88`, « destinée à l'assistant de saisie de facture »)
   gagne son second lecteur, `default_vat_at` (AC-5). Grep du symptôme (`ne référence vat_rates`) au Change Log. Gate **complet**
   même en cours de boucle (exception `kesh-db`).
2. **AC-2 — Fonctions pures** (`kesh-core::accounting::vat`, à côté de `line_vat_amount`, `vat.rs:40-44` ; toute
   l'arithmétique en `Decimal`, arrondi `Money::round_to_centimes`, `MidpointAwayFromZero`) :
   - `vat_from_gross(gross: Decimal, rate_percent: Decimal) -> Decimal` = round(brut × t ÷ (100 + t), 2) ; `t = 0` → 0 ;
     symétrique (`vat_from_gross(-x, t) == -vat_from_gross(x, t)`, l'arrondi s'éloignant de zéro des deux côtés) ;
   - `acquisition_vat(amount: Decimal, rate_percent: Decimal) -> Decimal` = round(montant × t ÷ 100, 2) — **délègue à
     `line_vat_amount`** (une seule règle de calcul HT × taux, DRY ; nom gardé pour C-38-7) ;
   - `VatTreatment { SaleIncluded, PurchaseIncluded, Acquisition }` et `split_default_vat(treatment, amount,
     rate_percent) -> DefaultVatSplit { base, vat }` — `amount` est une **grandeur** (≥ 0 ; le sens est celui de
     l'appelant, qui passe `|G|`, 38-6a AC-8, 38-6b ; un montant négatif → résultat symétrique, **sans**
     `debug_assert` : les tests tournent en debug, et la fixture porte des négatifs) : `SaleIncluded` et
     `PurchaseIncluded` : `vat = vat_from_gross(amount, t)`, `base = amount − vat` (par soustraction : `base + vat ==
     amount` exactement, l'écriture s'équilibre par construction) ; `Acquisition` : `base = amount`, `vat =
     acquisition_vat(amount, t)` (l'appelant écrit deux lignes de `vat`, l'une au débit de l'impôt préalable, l'autre
     au crédit du compte d'acquisitions). Doc-comment : la ventilation des comptes (TVA due, impôt préalable,
     acquisitions — E13) est écrite par 38-6a, non ici ;
   - **Devise** (`NFR-38-5-2`, S5 et A13 de l'Epic 37) : pour une ligne en devise, la ventilation se calcule **sur le
     montant en devise**, et la TVA passe en **CHF** au cours de la ligne du compte de produit ou de charge : `vat_chf =
     round(vat × cours, 2)`, par le produit de 37-3a AC-3 (pas de second produit ici) ; la base en CHF est celle de la
     ligne en devise (37-3a) ; l'écart d'un centime éventuel entre CHF des lignes relève de l'équilibre de 37-3a (C-37-4)
     et de 38-6a. Un test pur compose `split_default_vat` avec le produit de 37-3a sur un cas EUR (brut 108.10 EUR,
     8.1 %, cours 0.9295) et fixe le résultat attendu.
   - **Miroir TS** (`NFR-38-5-1`) dans `frontend/src/lib/features/journal-entries/vat-purchase.ts`, à côté de
     `lineVatAmount` (`:37-42`) : `vatFromGross(gross, ratePercent)`, `acquisitionVat` (= `lineVatAmount`),
     `splitDefaultVat(treatment, amount, ratePercent)` (`big.js`, `Big.roundHalfUp`, qui arrondit l'équidistant loin
     de zéro comme `MidpointAwayFromZero`) ;
   - **Fixture partagée** `crates/kesh-core/tests/fixtures/vat_default_split.json` (patron de
     `fx_rate_from_chf.json`, 37-3b AC-2 : même fichier lu par le test Rust et par le Vitest, aucune copie) — au moins :
     taux 8.1, 3.8, 2.6, 0 ; bruts 0.01, 1.00, 108.10, 100.00, 999 999 999.99 ; équidistants (`vat_from_gross` dont le
     quotient tombe sur un demi-centime : un cas construit et vérifié à la main au Change Log) ; montants négatifs
     (symétrie) ; acquisitions 100.00 à 8.1 → 8.10. Test Rust (`kesh-core/tests/`) et Vitest
     (`vat-purchase.test.ts`) parcourent toute la fixture.
3. **AC-3 — Modèle.** `kesh-db::entities::account` : `DefaultVatKind { Sale, Purchase, Acquisition, Other(String) }`
   (décodage **tolérant**, E4 : une valeur inconnue se lit sans erreur ; `as_str` rend la valeur brute ;
   `treatment(&self) -> Option<VatTreatment>` — `None` pour `Other`) ; `Account` (`:222-244`) gagne
   `default_vat_kind: Option<DefaultVatKind>`, `default_vat_rate_id: Option<i64>` ; `NewAccount` (`:249-259`) les deux
   champs (`None` par défaut ; `NewAccount::new` les pose à `None`, les créations en masse des plans livrés —
   `bulk_create_from_chart*`, `repositories/accounts.rs:1218`, `:1256` — n'en posent aucun) ; `AccountUpdate`
   (`:301-306`) un champ à **trois états** `default_vat: Option<Option<(DefaultVatKind, i64)>>` (inchangé / retiré /
   posé), résolu **dans le dépôt** sous le `FOR UPDATE` de `update` (`repositories/accounts.rs:622`, `:636`), patron de
   `currency` de 37-1 AC-4 ; `columns!` (`repositories/accounts.rs:27-31`) lit les deux colonnes ; `is_no_op_change`
   (`:530-539`) les compare ; `account_snapshot_json` (`:57-72`) les porte (`defaultVatKind`, `defaultVatRateId`) — sans
   quoi le diff d'audit d'un changement de TVA serait vide, précédent écrit `:66-67`. Littéraux à compléter : l'échec de
   compilation les inventorie (`AccountUpdate {` 26 sites, ` Account {` 3, `NewAccount {` une cinquantaine — 13 dans
   `repositories/accounts.rs`, le reste en tests —, hors déclarations, recomptés au développement : `grep -rn "NewAccount {"
   crates --include=*.rs`, passe P1 du 2026-11-15).
4. **AC-4 — Écriture d'un compte** (`FR-38-5-1`). `POST /api/v1/accounts` et `PUT /api/v1/accounts/{id}`
   (`routes/accounts.rs:35-47`, `:76-114` ; rôles inchangés : Comptable+, clés d'API `read-write` admises,
   `lib.rs:525-536`) gagnent `defaultVatKind` et `defaultVatRateId` :
   - désérialisés en `Option<Option<String>>` et `Option<Option<i64>>` par `present_or_null` (`:59-65`) — **absent =
     inchangé** au `PUT` (pas de 400 sur l'absence, à la différence de `role` : rien ne s'efface, et un client qui ignore
     le champ — clé d'API, écran d'une version antérieure, MCP — n'est pas cassé ; précédent `currency` de 37-1 AC-4) ;
     à la création, absent = `null` ;
   - **couple** : les deux absents → inchangé ; un seul présent → 400 `DEFAULT_VAT_INVALID` (`reason:
     "pair_incomplete"`) ; `kind` `null` ⇒ `rateId` `null` (sinon `rate_without_kind`) ; `kind` non nul ⇒ `rateId`
     requis (`rate_required`) ;
   - `kind` ∉ {`sale`, `purchase`, `acquisition`} → 400 `DEFAULT_VAT_INVALID` (`kind_unknown`) — le genre est lu en
     chaîne et validé, jamais rejeté par serde (le 422 natif d'Axum de `Json`, `routes/accounts.rs:8`) ;
   - **type du compte** (à l'état résultant, `account_type` du corps) : `sale` ⇒ `Revenue` ; `purchase`, `acquisition`
     ⇒ `Expense` ; sinon 400 `DEFAULT_VAT_INVALID` (`account_type`, `details.accountType`) — « compte de résultat
     seulement » (epic) ; un **retypage** d'un compte qui porte un genre incompatible avec le nouveau type, sans que
     le même `PUT` retire le genre (`defaultVatKind: null`), est refusé du même code — la garde se place avec
     `check_role_account_type` (`repositories/accounts.rs:80`, contrôle de forme, avant la transaction) pour un couple
     présent, et dans la transaction, après la lecture `FOR UPDATE`, pour un couple absent (valeur en place) ;
   - **taux** : `rateId ≤ 0` → 400 ; absent de la société → 400 `DEFAULT_VAT_INVALID` (`rate_not_found`) ; ligne
     inactive (`active = FALSE`) → 400 (`rate_inactive`) — contrôlés **seulement si le couple change** (patron « validé
     s'il change » des réglages, `routes/company_invoice_settings.rs:186-199`) : un taux désactivé après coup ne bloque
     pas un `PUT` sans rapport ; lu dans la transaction du dépôt ;
   - `DEFAULT_VAT_INVALID` : code neuf, 400, `details: { field: "defaultVatKind" | "defaultVatRateId", reason, … }`,
     message aux quatre locales ; ordre des refus écrit au doc-comment de `update`, à la suite de celui de 37-1 (après
     `FX_ACCOUNT_INVALID`) ;
   - `AccountResponse` (`:138-157`) expose `defaultVatKind` (chaîne brute, `Other` compris) et `defaultVatRateId`,
     **omis quand `NULL`** (`skip_serializing_if`, patron de `currency` de 37-1) : le JSON d'un compte sans TVA par
     défaut reste identique ; type TS `defaultVatKind?: string; defaultVatRateId?: number`.
   Aucun lien entre genre et devise du compte : un compte de ventes en EUR porte `sale` comme un compte CHF
   (NFR-38-5-2 est un calcul, AC-2). Tests HTTP (`kesh-api/tests/accounts_*`) : chaque refus avec son code et son
   `reason` ; création et modification nominales ; `PUT` sans les champs → couple inchangé ; `PUT` qui les retire ;
   retypage refusé puis admis avec retrait ; taux désactivé après coup puis `PUT` du seul libellé → 200 ; audit
   `account.updated` avec le couple dans `before`/`after`.
5. **AC-5 — Résolution à la date** (`FR-38-5-4`, `FR-38-5-5`). `kesh_db::repositories::accounts::default_vat_at(conn:
   &mut MySqlConnection, company_id, account_id, date: NaiveDate) -> Result<DefaultVatResolution, DbError>` —
   **seule** lecture du traitement TVA d'un compte pour une écriture, appelée par 38-6a et 38-6b dans leur transaction
   (lectures ordinaires ; aucune écriture) :
   - `DefaultVatResolution::None` — le compte n'a pas de genre ;
   - `::Resolved { treatment: VatTreatment, rate_percent: Decimal, vat_rate_id: i64 }` — `vat_rate_id` est celui de la
     ligne **en vigueur à `date`** dans la catégorie de la ligne désignée (`find_for_category_at_date`, `vat_rates.rs:89-108`, **qui prend aujourd'hui un `&MySqlPool`** : **décidé** — la requête
     passe en variante `find_for_category_at_date_in_conn(conn: &mut MySqlConnection, …)`, la version du pool lui
     délègue (une seule requête, DRY) ; 38-6a AC-8 (7) lit dans sa transaction, `get_proposals` sur une connexion
     prise au pool) ; ce peut être la ligne désignée ou une autre de même catégorie ;
   - `::VatKindUnknown { kind }` — genre `Other` (écrit par une version ultérieure) → 38-6a : `FailedProposal`
     `VAT_KIND_UNKNOWN` ; jamais traité comme « sans TVA » (aucune écriture fausse) ;
   - `::VatRateUnavailable { category, date }` — aucune ligne active de la catégorie à `date` → 38-6a :
     `VAT_RATE_UNAVAILABLE`.
   La garde « taux actif » de l'AC-4 vaut à l'écriture ; la résolution à la date est la seule qui vaut pour une
   transaction. Tests (base, `kesh-db/tests/`) : catégorie `normal` avec deux lignes (7.7 % jusqu'au 2023-12-31, 8.1 %
   dès le 2024-01-01 — bornes `valid_to` **exclusives**, `vat_rates.rs:83-84`) : 2023-12-31 → 7.7, 2024-01-01 → 8.1 ;
   compte désigné sur la ligne 7.7 % → 8.1 % en 2024 (catégorie, Écart 1) ; date avant toute ligne →
   `VatRateUnavailable` ; ligne désignée désactivée mais une autre active à la date → `Resolved` ; genre `Other` posé
   par SQL → `VatKindUnknown` ; compte sans genre → `None`.
6. **AC-6 — Compte d'impôt sur les acquisitions** (`FR-38-5-3`, B18). `PUT /api/v1/company/invoice-settings`
   (`routes/company_invoice_settings.rs:307`, Administrateur, session seule) :
   - `UpdateInvoiceSettingsRequest` (`:96-144`) gagne `default_vat_acquisition_account_id: Option<Option<i64>>`
     (`crate::helpers::double_option`) — **absent : préservé ; `null` : effacé** (patron du compte créanciers,
     `:114-120`) ; `InvoiceSettingsResponse` (`:41`) et `CompanyInvoiceSettings`/`…Update`
     (`entities/company_invoice_settings.rs:21`, `:77`) l'exposent ; `COLUMNS` du dépôt (`:33-40`) le lit ;
   - validé par `validate_account` (`:169`) avec un type neuf de `VatSettingAccountTypes`
     (`repositories/company_invoice_settings.rs:1264-1283`) : `acquisition: AccountType::Liability` — la règle reste
     écrite **une fois** (C-31-5-19) ; actif et imputable s'il change, de la société (comportement de
     `validate_account_of`, `:200-240`) ; libellé « Compte TVA due sur acquisitions » ;
   - **distinct du compte de TVA due** résolu (`default_vat_payable_account_id`) : égal → 400 `VALIDATION_ERROR`
     (« le compte d'impôt sur les acquisitions doit être distinct du compte TVA due », Écart 4) ; contrôlé seulement si
     l'un des deux change ;
   - en devise : refusé par la garde de 37-1 (C-37-9), qui lit `DESIGNATED_ACCOUNT_COLUMNS` (AC-7) — aucune garde
     propre ici ;
   - audit : `company_invoice_settings.updated` (existant) ; `settings_snapshot_json`
     (`repositories/company_invoice_settings.rs:94`) porte le champ.
   - **Sites du dépôt qui énumèrent les colonnes à la main** (inventaire de la passe P1 ; `COLUMNS` seule ne suffit pas) :
     `is_no_op_change` (`:287-300`, sinon un `PUT` qui ne change que ce compte est pris pour un no-op) ; le `UPDATE …
     SET` et ses `bind` dans `update` (`:398-420` et suivants) ; la liste **préfixée `cis.`** du `SELECT` de la branche
     `rows == 0` d'`insert_with_defaults_in_tx` (`:1230-1250`, doublon de `COLUMNS` : un oubli n'y échoue qu'à ce
     chemin) ; `get_or_create_default` et `_in_tx` (commentaire `MIRROR`, à relire) ; `settings_snapshot_json`. Littéraux
     de test à compléter (compilation) : `kesh-db/tests/company_invoice_settings_repository.rs`,
     `kesh-api/tests/onboarding_path_b_e2e.rs`, `kesh-db/tests/invoice_lines_revenue_account_backfill.rs`,
     `kesh-api/tests/accounts_designated_archive_e2e.rs`. Test : un `PUT` qui ne change que le compte d'acquisitions
     est écrit (version +1) ; la branche `rows == 0` d'`insert_with_defaults_in_tx` rend la colonne.
   La ligne de **préalable** des acquisitions va au compte d'impôt préalable existant (`default_vat_recoverable`, E13) ;
   un réglage manquant à l'usage → `VAT_ACCOUNT_NOT_CONFIGURED` `designation: "acquisition"` (38-6a, C-38-8) — non
   contrôlé ici. Tests HTTP : posé, préservé si absent, effacé par `null`, type faux, archivé, égal au compte TVA due,
   d'une autre société.
7. **AC-7 — Réglage désigné** (Écart 3). `DESIGNATED_ACCOUNT_COLUMNS` (`[(&str, &str); 10]`, `company_invoice_settings.rs:46`) passe à **onze** :
   `("default_vat_acquisition_account_id", "vatAcquisition")`, type du tableau et doc-comment (`:42`, « Les dix
   comptes ») compris ; le test qui compare la liste aux colonnes `default_%_account_id`
   (`accounts_designated_archive_e2e.rs:1077-1078`) reste vert, **mais la seconde assertion (`:1079-1083`,
   `DESIGNATED_ACCOUNT_COLUMNS == TEN_SETTINGS`, « les codes de l'API sont stables ») rougit** : la constante
   `TEN_SETTINGS` (`:37`, `[…; 10]`) et ses lecteurs (`:142`, `:202`, l'intitulé « chacun des dix réglages », `:198`)
   passent à onze (`ELEVEN_SETTINGS`), avec un cas d'archivage du compte d'acquisitions ; le commentaire
   `frontend/src/routes/(app)/accounts/+page.svelte:796` (« vaut pour les dix réglages ») est mis à jour ; l'archivage d'un compte ainsi
   désigné rend 409 `ACCOUNT_DESIGNATED_IN_SETTINGS` qui le nomme (`errors.rs:3587`) ; l'écran d'archivage traduit le
   code (`designatedSettingLabel`, `frontend/src/routes/(app)/accounts/+page.svelte:347-371` : cas `vatAcquisition`).
   Test : archiver le compte d'acquisitions → 409 avec `vatAcquisition` ; avec `confirmDesignated` → 200.
8. **AC-8 — Écrans** (Comptable+ pour le plan, Administrateur pour les réglages, comme aujourd'hui).
   - *Plan comptable* (`routes/(app)/accounts/+page.svelte`, création `:204-205` et modification `:234-268`) : pour
     un compte `Revenue` ou `Expense`, un champ « TVA par défaut » — `Revenue` : « Aucune » / « Ventes — TVA due
     incluse » ; `Expense` : « Aucune » / « Achats — impôt préalable inclus » / « Acquisitions — impôt sur les
     acquisitions (service acheté à l'étranger) » — et, si un genre est choisi, le **taux** (liste `GET
     /api/v1/vat-rates`, `lib.rs:1054`, taux actifs, libellé et catégorie) ; le texte d'aide dit que le taux suit la
     date de chaque transaction dans sa catégorie (Écart 1). Le champ disparaît pour un compte `Asset`/`Liability` ;
     changer le type d'un compte qui porte un genre propose de le retirer (sinon le serveur refuse,
     `DEFAULT_VAT_INVALID`, message affiché sous le champ par `details.field`). Le `PUT` envoie toujours le couple
     (l'écran le connaît), **sauf** un genre `Other` laissé tel quel : le couple est alors **omis** (absent = inchangé,
     AC-4) — l'envoyer rendrait 400 `kind_unknown` et empêcherait de corriger le seul libellé du compte. Un genre
     `Other` lu s'affiche « Genre d'une version ultérieure (`x`) », non modifiable sans le retirer. Vitest : `PUT` du
     libellé d'un compte à genre `Other` sans le couple. La liste porte une colonne ou une pastille « TVA » (genre et taux). Vitest : options par type,
     couple envoyé, retrait au retypage, affichage d'`Other`.
   - *Administration → Facturation* (`routes/(app)/settings/invoicing/+page.svelte`, patron des sélecteurs de TVA
     `:54-60`, `:106-122`, `:181-183`, `:238`, `:475-481`) : sélecteur « Compte TVA due sur acquisitions (Passif) »
     (comptes `Liability`, `withCurrentAccount`), envoyé au `PUT` ; Vitest.
   - **Captures d'écran** (*Recette visuelle*, thème clair, hors dépôt) : formulaire de compte (produit, charge,
     bilan), liste, réglages, refus affiché.
9. **AC-9 — Sauvegarde, export, MCP.**
   - `.keshbackup` : les trois colonnes passent seules ; aller-retour d'un compte à TVA par défaut et du réglage testé.
   - **Liste close des absences admises** (37-1 AC-6, E18) : `accounts.default_vat_kind`,
     `accounts.default_vat_rate_id`, `company_invoice_settings.default_vat_acquisition_account_id`, avec la version
     de la migration de l'AC-1 ; test d'import d'une sauvegarde antérieure (patron de 37-1) : colonnes retirées → 200,
     comptes sans TVA par défaut.
   - **Export CSV** : `serialize_accounts_csv` (`exports/csv_tables.rs:265`) gagne `default_vat_kind`,
     `default_vat_rate_id` ; `serialize_company_invoice_settings_csv` (`:953`) gagne
     `default_vat_acquisition_account_id` ; `chaque_colonne_du_schema_est_exportee_ou_ecartee` (`:2377`) vert ; tests
     unitaires d'une ligne renseignée.
   - **Remise à zéro** : `accounts`, `vat_rates` et `company_invoice_settings` sont toutes vidées
     (`reset_cleared_tables`, aucune n'est dans `RESET_PRESERVED_TABLES`, `backup.rs:136-144`) — rien à faire ;
     `reset_partition_is_guarded_by_the_schema` vert.
   - **MCP** (E23) : `kesh_plan_comptable` (`mcp/outils/plan_comptable.rs:31-43`) **n'est pas étendu** — ajouter deux
     champs impose de re-mesurer les tailles de sortie (`mcp_outils_e2e.rs`, précédent 37-1 AC-8) pour une donnée de
     configuration qu'un script lit par `GET /accounts` ; exclusion écrite au Change Log et portée au registre C-38-13
     (issue avec la PR, « on trace toujours »).
10. **AC-10 — i18n et manuels.**
    - Clés neuves (libellés de l'AC-8, `DEFAULT_VAT_INVALID` et ses motifs, libellé du réglage, cas `vatAcquisition`)
      aux quatre locales ; `lint-i18n-ownership` vert (`lint-i18n-ownership.js:177-186` : sous `features/accounts/`,
      premier segment `accounts` — `accounts-`, non `account-` ; sous `features/settings/`, `settings-`) : préfixes
      **`accounts-`** (plan comptable), **`settings-invoicing-`** (réglage, patron de
      `routes/(app)/settings/invoicing/+page.svelte`), `error-` pour les messages serveur de `DEFAULT_VAT_INVALID` ;
      décompte recompté par `grep -c` aux deux bornes.
    - **Manuel utilisateur** (`docs/manual/fr/user-manual.tex`) : au plan comptable, « TVA par défaut d'un compte »
      (genres, taux par catégorie à la date de la transaction, compte de résultat seulement, à quoi elle sert —
      propositions des prestataires de paiement, 38-6a —, et ce qu'elle ne fait **pas** : la saisie manuelle et
      l'assistant d'achat ne la lisent pas encore) ; aux réglages de facturation (section *Comptes par défaut*, `:1791-1792`, et le paragraphe des comptes désignés
      contrôlés à l'usage, `:491` — y ajouter le compte d'acquisitions, que 38-6a contrôle à l'usage ; `:219` est le
      texte de l'assistant d'installation, à compléter d'un mot seulement) : le compte d'impôt sur les acquisitions, distinct, et ⚠️ **ce que le rapport TVA ne fait pas** — il tire la TVA
      due des seules factures (`kesh-report/src/vat_report.rs:1-25`, manuel `:3655`) : la TVA séparée par les
      propositions et l'impôt sur les acquisitions **n'y figurent pas** (C-38-13, *Point TVA pour Guy*). PDF régénéré
      (`make fr` dans `docs/manual/`) et contrôlé aplati (`pdftotext … | tr '\n' ' ' | tr -s ' '`), les deux phrases
      grepées.

## Tâches

- [ ] Migration (AC-1), ligne d'audit P5 et compteurs, grep P6, squash régénéré, liste close R1 (AC-9).
- [ ] `kesh-core` : `vat_from_gross`, `acquisition_vat`, `VatTreatment`, `split_default_vat` ; fixture partagée et
      test Rust ; test de composition avec le produit CHF de 37-3a (AC-2).
- [ ] `kesh-db` : `DefaultVatKind`, entité, `NewAccount`, `AccountUpdate` à trois états, `columns!`,
      `is_no_op_change`, `account_snapshot_json`, gardes dans `create`/`update` (AC-3, AC-4) ; `default_vat_at` (AC-5) ;
      réglage d'acquisitions, `VAT_SETTING_ACCOUNT_TYPES.acquisition`, `DESIGNATED_ACCOUNT_COLUMNS` (AC-6, AC-7).
- [ ] `kesh-api` : DTO et validation des comptes, `DEFAULT_VAT_INVALID` dans `errors.rs` (AC-4) ; réglages de
      facturation (AC-6) ; export CSV (AC-9).
- [ ] Frontend : miroir TS et Vitest sur la fixture (AC-2) ; écrans (AC-8) ; i18n (AC-10) ; captures.
- [ ] Manuel utilisateur et PDF (AC-10).
- [ ] Gate complet (base remise à zéro avant), `scripts/gate-ci.sh` au dernier commit de code, frontend complet,
      E2E complet au dernier commit de code (les specs du plan comptable et des réglages de facturation doivent rester
      vertes ; une spec neuve du champ « TVA par défaut » — créer un compte de produit à `sale` 8.1 %, relire —, la
      valeur traversant la frontière HTTP).

## Notes de développement

- **Ce que la story ne fait pas** : aucune écriture n'est ventilée ici. 38-6a applique `default_vat_at` et
  `split_default_vat` aux propositions ; la saisie manuelle et l'assistant d'achat (`VatPurchaseAssistant.svelte`,
  `vat-purchase.ts:73`) ne lisent pas la TVA par défaut (« pourra reprendre », epic) — suite possible, hors périmètre,
  à tracer si Beat la veut.
- **Pourquoi pas une colonne `category`** au lieu d'un identifiant de taux : C-38-7 nomme `defaultVatRateId`, et
  l'identifiant garde le lien à une ligne réelle (FK `RESTRICT`, libellé affiché) ; la catégorie s'en lit. Si la
  validation préfère une catégorie, c'est une décision d'epic (C-38-7) à remonter, non à trancher ici.
- **Taux `custom`** : l'unicité par catégorie à une date vaut aussi pour `custom` (`has_overlap_in_category` ne
  distingue pas les catégories) — deux taux « custom » ne coexistent pas à une date : un compte désigné sur un taux
  personnalisé suit les autres lignes `custom`. À écrire au manuel en une phrase.
- **Rapport TVA** : inchangé (B15, P1 « Vague 2 ») ; la phrase du manuel (AC-10) est le seul geste.
- Références : `kesh-core/src/accounting/vat.rs:1-44` (règle d'arrondi par ligne, FR55) ;
  `kesh-core/src/types/money.rs:66` ; `frontend/src/lib/features/journal-entries/vat-purchase.ts:1-42` (parité
  Rust/TS, `Big.roundHalfUp`) ; 37-3b AC-2 (fixture `fx_rate_from_chf.json`).

## Change Log

- 2026-11-15 — rédaction (Opus).
- 2026-11-15 — validation P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 3 LOW ; remédiés dans la fiche sauf 38-5-P1-5
  (LOW, arbitrage) et la note inter-fiches sur 37-1 (38-5-P1-1) : P1-1 (MEDIUM) le test `TEN_SETTINGS` rougit au
  onzième réglage (AC-7) ; P1-2 (MEDIUM) sites du dépôt des réglages qui énumèrent les colonnes à la main
  (`is_no_op_change`, `UPDATE`, liste `cis.` d'`insert_with_defaults_in_tx`) et littéraux `NewAccount` (AC-6, AC-3) ;
  P1-3 (MEDIUM) `find_for_category_at_date` est sur le pool, contrat de `default_vat_at` tranché en variante `_in_conn`
  (AC-5) ; P1-4 (LOW) renvois du manuel (`:219` est l'assistant d'installation ; `:491` et `:1791` sont les bons sites,
  AC-10) ; P1-5 (LOW, non appliqué) l'égalité du compte d'acquisitions n'est refusée qu'avec la TVA due (O6) : le
  compte de décompte et le compte créanciers (passifs aussi) restent admis ; P1-6 (LOW) renvoi à O1 en en-tête.
- 2026-11-15 — validation P2 (Opus) : 3 findings (0 CRITICAL, 0 HIGH, 0 MEDIUM, 3 LOW) ; remédiés dans la fiche, aucun
  report inter-fiches. LOW : P2-1 le commentaire « aucune table ne référence vat_rates » (`backup.rs:110`) devient faux
  avec la FK neuve, et le doc-comment de `find_for_category_at_date` (`vat_rates.rs:81-88`) gagne un lecteur (AC-1) ;
  P2-2 `split_default_vat` : `debug_assert` sur un montant négatif contredisait la symétrie testée par la fixture
  (tests en debug) → retiré, l'appelant passe `|G|` (AC-2) ; P2-3 l'écran « envoie toujours le couple » : avec un genre
  `Other` en place, tout `PUT` rendait 400 `kind_unknown` → couple omis s'il n'est pas touché (AC-8). Relu sans
  écart : diff `e80f7695..7ef077cc` de la fiche ; `company_invoice_settings.rs` (dépôt) `:40-58, 92-96, 285-301,
  396-400, 1226-1232, 1262-1284`, `MIRROR` `:121-159` ; route `:41, 96-144, 169, 180-200, 307, 365-371` ;
  `accounts_designated_archive_e2e.rs:37, 142, 198, 202, 1075-1084` ; `accounts/+page.svelte:347, 795-797` ;
  `vat_rates.rs:80-108, 140-165` ; `vat.rs:36-44` ; `money.rs:62-70` ; `vat-purchase.ts:30-45` ; `routes/accounts.rs:55-66,
  76-114, 139, 215` ; `repositories/accounts.rs:27, 57, 80, 530, 622` ; `entities/account.rs:222, 249, 301` ;
  `backup.rs:93-111, 136-144` ; `csv_tables.rs:265, 953` ; `plan_comptable.rs:31-43` ; `user-manual.tex:219, 491,
  1791-1792, 3655` ; 37-1 AC-4 (`:420-446`, notes de dépendance onze et O3) ; 38-6a AC-8 et 38-6b (appel par `|G|`,
  `default_vat_at`). Axes non exercés : aucune exécution ; MidpointAwayFromZero contre `Big.roundHalfUp` relu à la
  documentation de mémoire (big.js : équidistant loin de zéro), non exécuté.
- 2026-11-15 — préfixes i18n (constat de la validation de l'Epic 40, Opus) : AC-10 ne nommait aucun préfixe ;
  précisé contre `lint-i18n-ownership.js:177-186` — `accounts-` (et non `account-`, refusé sous `features/accounts/`),
  `settings-invoicing-`, `error-` (global). Rédaction seule.
