# Epic 38 — Comptes PayPal : téléchargement et imputation automatiques

**Cadrage : 2026-11-15** (branche `epic/38-paypal`), **révisé le même jour sur B16–B21** (§ *Révision B16–B21*)
**puis sur B22–B26** (§ *Révision B22–B26*). Demande de Beat (contributeur), décisions B1–B26 de
`epics-38-40-cadrage.md`, qui **priment** sur ce fichier là où il diverge (B22 remplace B16 sur les domaines fermés).
Kickoff (décomptes d'issues et de bugs, validation de ce fichier) **à tenir** : `38-0-kickoff` reste au backlog tant
que l'Epic 37 n'est pas développé.

**Une seule migration breaking** (B22), dans **38-3** : `source_format = 'PROVIDER_API'` pour l'en-tête d'un
passage, et `reconciliation_rules.match_type` passé d'`ENUM` à `VARCHAR` pour les critères neufs ;
`kesh_version_min_required` relevé **en dernière instruction**, les dix crates bumpés **dans le même commit** (P2,
P2-bis) — **numéro de release à Guy** (Q1). Toutes les autres migrations de l'epic créent des tables ou ajoutent des
colonnes `NULL`. `account_kind` ne reçoit toujours aucune valeur neuve (B16 maintenu). Détail en E19.

**Numéro : 38** (35 et 36 réservés ailleurs ; 39 Wise, **40 Stripe**, provisoire en attendant Guy, B25). Une branche par
story, fusionnée localement dans `epic/38-paypal`, puis **une PR** vers le dépôt de Guy (remote `beat`), **après celle
de l'Epic 37**. Pas d'issue GitHub à ce stade (Beat n'a qu'un accès en lecture) : le détail vit ici et dans les fiches ;
issues et PR s'ouvrent à la fin.

**Cible** : la release qui suivra celle de l'Epic 37 (numéro, CHANGELOG daté, macros des manuels, tag : par Guy).

## Intention

Un compte PayPal *business* se tient dans Kesh **sans saisie** : Kesh télécharge chaque jour (ou à la demande) les
mouvements de chaque solde de devise, les classe (vente, achat, frais, conversion, remboursement, litige, retrait), et
les **propose** à l'imputation — brut au compte de produit ou de charge désigné par une **règle d'affectation**, TVA
séparée selon le **code TVA par défaut du compte**, frais au **sous-compte de frais** du prestataire et de la devise —
pour une acceptation en lot. Rien n'est écrit d'office. Le relevé annuel PayPal (PDF, sans API) se joint à la main au
compte et à l'exercice.

L'epic pose d'abord le **socle commun** (B14) que réutiliseront Wise (39) et Stripe (40), puis le connecteur PayPal,
les écrans, et le relevé joint.

## Dépendance à l'Epic 37 — et l'engagement D8

L'Epic 38 **ne se développe qu'après l'Epic 37** (fiches 37-1 à 37-5 `ready-for-dev`, aucune livrée au
2026-11-15). Il en consomme :

- la **devise portée par le compte** (37-1, T3) et son verrou dès la première ligne ;
- la table **`exchange_rates`** par société et le cours effectif `find_company_rate_in_tx` (37-2, C-37-1 amendé U1) ;
- la **ligne en devise** `foreign { amount, rate | chf }`, le contrôle central `ACCOUNT_REQUIRES_CURRENCY`, la règle
  d'une seule devise étrangère par écriture `FX_MULTIPLE_CURRENCIES`, `rate_from_chf` et A32 (37-3a) ;
- la **conversion par le compte de transit**, `POST /journal-entries/conversions`, `completeDifference: true` (37-3b,
  R7, V3, **A31**) ;
- les **comptes de différence de change propres** d'un compte en devise, `fx_accounts_for` (37-1, S2) ;
- le **client HTTP borné** (`reqwest` rustls, URL constante, délai et taille bornés) et le patron « source objet-safe
  dans `AppState` + source factice en `KESH_TEST_MODE` + route `_test` » (37-2 AC-3, C-37-6) ;
- la **liste close d'absences admises** à l'import d'une sauvegarde antérieure (37-1 AC-6, R1).

**Engagement D8 de l'Epic 37**, tenu ici (38-2) : un compte bancaire Kesh peut être **lié à un compte du plan déjà en
devise**, en gardant son numéro, son historique et ses soldes. C'est le verrou C-37-9 (`ACCOUNT_IN_FOREIGN_CURRENCY`
sur un lien bancaire) que 38-2 lève, **pour le seul compte bancaire qui porte des réglages de prestataire** (E11 : une
ligne `payment_provider_accounts`, posée dans la même transaction que le lien) ; le compte reste `account_kind =
'bank'`. L'issue promise par D8 s'ouvre avec la PR de l'Epic 37 ; celle de l'Epic 38 la ferme.

## Décisions de Beat reprises (B1–B15, résumé)

B1 un compte Kesh par solde et par devise (comptes en devise de l'Epic 37) · B2 nouveau solde listé et proposé, jamais
créé d'office · B3 frais à un sous-compte par prestataire et par devise, regroupés sous un compte parent ; transaction
passée au brut · B4 conversions par le compte de transit, `completeDifference: true` · B5 critères neufs des règles
(courriel, préfixe de facture, nom d'article, code d'événement, sens), proposition seulement, `accept_batch` · B6 TVA
séparée selon la règle TVA par défaut du compte (impôt sur les acquisitions compris) · B7 remboursement = inverse de
l'écriture d'origine ; litige → file « à examiner » ; seuls les mouvements qui changent le solde · B8 date
« télécharger depuis » par compte de prestataire · B9 quotidien automatique ou à la demande, bouton dans les deux cas
· B10 relevé PayPal joint à la main · B11 secrets chiffrés en base par une clé d'environnement neuve, saisis par un
Administrateur, jamais réaffichés · B12 aucun appel réseau en CI, réponses enregistrées anonymisées · B13 (Stripe,
40) · B14 socle commun dans l'Epic 38 · B15 hors périmètre : factures, décompte TVA.

**Suite (B16–B21)** : B16 compte `bank` + réglages de connexion en table neuve, passages en table propre (*aucune
migration breaking* : remplacé par B22) · B17 2024 déjà importé : rattachement **en lot** aux écritures existantes,
sans doublon · B18 compte distinct de TVA due sur acquisitions (p. ex. 2201) · B19 sous-comptes de ventes par devise ;
critère « boutique / domaine » ; analytique non tranchée · B20 remboursements et frais tels que rapportés ; relevés =
sommes de contrôle · B21 un script externe (WHMCS, CB Paid Subscriptions) crée des écritures ou les rattache par
l'API.

**Suite (B22–B26)** : B22 **une** migration breaking acceptée — `source_format = 'PROVIDER_API'` et types neufs de
`match_type`, sans détour ; comptes liés et devise restent aux réglages de connexion · B23 options propres au
prestataire dans une colonne `settings JSON` lue en struct typée, comptes liés en colonnes FK, secrets à part · B24
boutique PayPal = **préfixe du numéro de facture** (`invoice_id`) · B25 Q3 reportée ; Stripe = Epic 40 (provisoire)
· B26 FR et NFR numérotées par fiche (forme — issues ou fiches — à Guy).

## Inventaire du code existant (relevé le 2026-11-15)

| Domaine | Ce qui existe | Où |
|---|---|---|
| Comptes bancaires | `account_kind` `bank` \| `credit_card`, domaine fermé à l'octet, IBAN facultatif, `card_last4` ; migration **breaking** (min 0.17.6) | `migrations/20261110000001_bank_accounts_without_iban.sql:39-55` |
| | `BankAccountKind`, `FromStr` **refuse l'inconnu** ; `admits_ledger_type` (carte : passif seul) | `kesh-db/src/entities/bank_account.rs:21-69` |
| | dépôt (colonnes lues, création, `PUT`) | `kesh-db/src/repositories/bank_accounts.rs:54, 65, 769` |
| | lecteurs de `account_kind` (relevés pour E19) | `repositories/accounts.rs:482` ; `invoice_settlements_write.rs:203-218` ; `routes/bank_accounts.rs:97, 194, 403-406, 462, 555-578, 972, 1141, 1406-1411, 2048, 2110` ; `routes/companies.rs:422-438` ; `routes/onboarding.rs:665` ; `exports/csv_tables.rs:661-680` |
| | routes `/api/v1/bank-accounts…` | `kesh-api/src/lib.rs:938-954` |
| Transactions bancaires | `bank_imports` (`source_format` VARCHAR sans CHECK, `file_hash`, période), `bank_transactions` (`amount`, `currency`, `reference`, `details`, `end_to_end_id`, `transaction_id`, `counterparty_*`, `status` `pending`\|`reconciled`) | `migrations/20260504000001_bank_imports.sql:19-88` |
| | `BankImportSourceFormat` (`CAMT053_V04`, `_V08`, `CSV`, `MANUAL`), `FromStr` refuse l'inconnu : `MANUAL` a coûté une migration breaking | `kesh-db/src/entities/bank_import.rs:26-66` ; `migrations/20261030000001_bank_imports_manual_source.sql` |
| | **`bank_transactions.import_id NOT NULL`**, FK `ON DELETE CASCADE` ; `bank_imports.imported_by_user_id NOT NULL` | `migrations/20260504000001_bank_imports.sql:33, 58, 79` |
| | lecteurs de `source_format` : affichage seul (`MANUAL` → « Saisie manuelle ») ; aucun comportement n'en dépend | `routes/bank_imports.rs:113, 284, 994-1016, 1740, 1928, 2159` ; `routes/bank_statements_manual.rs:120, 180, 370, 407` ; `exports/csv_tables.rs:702-720` ; `frontend/src/lib/features/bank-import/bank-import-display.ts:14-36` |
| | `BankTransactionStatus` | `kesh-db/src/entities/bank_transaction.rs:19-25` |
| | clé de doublon (date, montant, référence normalisée, compte), fenêtre et alias | `kesh-core/src/bank_imports.rs:295-360, 598-640` ; `kesh-db/src/repositories/bank_transactions.rs:83` |
| | empreinte d'aperçu d'import | `kesh-api/src/routes/bank_import_fingerprint.rs:90-180` |
| | routes d'import CAMT/CSV et relevé manuel | `kesh-api/src/lib.rs:831-851` |
| Réconciliation | `post_accept` (lot, *Pattern batch*), `accept_one_rule`, `_invoice`, `_split`, `FailedProposal` | `kesh-api/src/routes/reconciliation.rs:571, 1593, 3518, 3946` (fichier de **7 439 lignes**) |
| | propositions, rattachement d'un virement entre ses comptes (34-11b), liaison à une écriture existante (v017-3a) | `reconciliation.rs:999, 6373, 6548, 6656` ; `kesh-db/src/repositories/reconciliation_link.rs:90, 358, 535` ; `lib.rs:874-932` |
| | signal « déjà comptabilisée » (34-4) : `already_booked` par proposition, deux lectures par page ; le serveur ne refuse rien | `reconciliation.rs:119-140, 1060-1095` ; `reconciliation_link.rs:886-1000` (`count_already_booked_for_page`) |
| | ⚠️ **CHF seulement** : `transaction_can_be_linked_to_entry` (montant non nul **et** `currency == "CHF"`) ; `get_proposals` saute toute transaction hors CHF | `reconciliation_link.rs:535-537` ; `reconciliation.rs:1098-1104` |
| | clés d'API `read-write` admises sur les routes Comptable (écritures, rapprochement, `link-entry`) ; `require_not_pat` sur les seules routes d'administration | `lib.rs:518, 523-545, 921-929` |
| | écriture d'une règle : **deux lignes** (banque, contrepartie) | `kesh-reconciliation/src/manual.rs:68` |
| Règles | `reconciliation_rules` : `match_type` **ENUM** (`counterparty_contains`, `_exact`, `reference_contains`, `iban_exact`), une règle → un compte, `UNIQUE (company_id, match_type, active_uniq)`, `default_project_id` | `migrations/20260513000001_reconciliation_rules.sql:36-69` ; `20260704000001_…default_project.sql:12` |
| | `rule_matches`, `first_matching_rule` | `kesh-reconciliation/src/rules.rs:46-95` ; `ReconciliationMatchType` `kesh-db/src/entities/reconciliation_rule.rs:32` |
| | lecteurs de `match_type` (relevés pour E19) | `rules.rs:46-70` ; `routes/reconciliation_rules.rs:63, 82` ; `kesh-api/src/errors.rs:1150-1159, 3304-3316` ; `exports/csv_tables.rs:1015-1059` ; `frontend/…/rules/RuleFormModal.svelte:26, 104, 154`, `RulesList.svelte:31, 141` |
| Réglages JSON | `bank_profiles.column_mapping JSON NOT NULL` + `CHECK (JSON_VALID(column_mapping))` ; lu en chaîne (`column_mapping_json`) puis en struct par `parse_column_mapping` (`serde_json::from_str`, erreur `DbError::Invariant`) ; struct `ColumnMapping` en `#[serde(deny_unknown_fields)]` (contrat : refus bruyant d'une clé inconnue) | `migrations/20260505000001_bank_profiles.sql:27, 48-49` ; `kesh-db/src/entities/bank_profile.rs:31-57` ; `kesh-import/src/csv/profile.rs:36-48` |
| | `mcp_server_settings` : tableaux JSON avec `CHECK (JSON_VALID(…) AND JSON_TYPE(…) = 'ARRAY')` | `migrations/20261025000001_mcp_keys_and_server_settings.sql:71-83` |
| Version minimale | précédent de relèvement : `UPDATE _kesh_version SET kesh_version_min_required = '0.17.0'` seule instruction, exemptée P7 | `migrations/20261030000001_bank_imports_manual_source.sql` |
| TVA | `vat_rates` (libellé, taux, validité) — **aucun code TVA, aucun genre** (dû, préalable, acquisitions) | `migrations/20260428000001_vat_rates.sql` ; `kesh-api/src/routes/vat.rs:59-462` |
| | comptes TVA des réglages de facturation (`company_invoice_settings.default_vat_payable/recoverable/decompte_account_id`) — **aucun compte d'impôt sur les acquisitions** | `migrations/20260614000001_vat_accounts_config.sql:53-56` |
| | calcul `line_vat_amount` (HT × taux) — **aucune extraction depuis un brut** | `kesh-core/src/accounting/vat.rs:40` |
| | assistant d'achat **à l'écran seulement** (3 lignes) | `frontend/src/lib/features/journal-entries/vat-purchase.ts:73` ; `VatPurchaseAssistant.svelte` |
| | **rapport TVA : TVA due tirée des seules factures**, préalable lu au grand livre | `kesh-report/src/vat_report.rs:1-25` |
| | **aucun code TVA par défaut sur un compte** (B6) | — |
| Pièces | stockage `{sha256}.{ext}` sous `KESH_DOCUMENTS_DIR` ; seules les factures fournisseur importées et les PDF figés y vont | `kesh-api/src/document_storage.rs:1-15` ; sauvegarde : `kesh-api/src/admin_backup/import.rs:115-120` |
| Tâches planifiées | sauvegarde automatique (`interval_at`, un tour par `tokio::spawn`) | `kesh-api/src/auto_backup.rs:788` ; lancée `main.rs:467` |
| | récupération des cours (spécifiée, non livrée) | `37-2-cours-mensuels-afc.md` AC-3, AC-4 |
| Secrets | clés d'API et MCP **hachées** SHA-256 (jamais chiffrées) ; jetons de renouvellement en empreinte | `migrations/20260605000001_api_keys.sql:23` ; `…mcp_keys_and_server_settings.sql:52` |
| | refus des gabarits `<…>` au démarrage | `kesh-api/src/config.rs:650, 683` (`is_template_placeholder`) |
| | **aucun chiffrement symétrique** dans le dépôt (pas d'`aes-gcm`, de `chacha20poly1305`) | `crates/*/Cargo.toml` |
| | refus d'une clé d'API sur une route (`require_not_pat`) | `kesh-api/src/middleware/rbac.rs:58` |
| HTTP sortant | `reqwest` **en dev-dependency seulement** ; 37-2 le passe en production (rustls) | `kesh-api/Cargo.toml:109` ; 37-2 AC-3 |
| Sauvegarde | tables applicatives (ordre enfants → parents), tables conservées à la remise à zéro, tables système `_kesh_*` | `kesh-db/src/backup.rs:46, 137, 949` |

## Décisions d'architecture (E1…)

- **E1 — Une connexion, plusieurs comptes.** Table applicative `payment_provider_connections` (société, `provider`
  `paypal`|…, libellé « PayPal Lightning », `fetch_from` (B8), `schedule` `daily`|`manual` (B9), état du dernier
  passage, `created_by_user_id`, **`settings JSON`** — E24). Chaque solde de devise est un **compte bancaire** Kesh ordinaire (`account_kind =
  'bank'`, sans IBAN) qui porte une ligne de **réglages de prestataire** (E11) le rattachant à la connexion, et lié à un
  compte du plan **de la devise du solde** (B1). *Raison* : un jeu de secrets et un téléchargement servent trois
  soldes ; la réconciliation, elle, reste par compte, comme les relevés des banques.
- **E2 — Secrets hors sauvegarde.** Les secrets vont dans une table **système** `_kesh_provider_secrets` (clé :
  connexion), chiffrés **AES-256-GCM** (nonce aléatoire, AAD = société + connexion + nom du secret + version de clé),
  clé tirée de la variable neuve **`KESH_SECRETS_KEY`** (32 octets en base64 ; gabarit refusé comme
  `is_template_placeholder`). Variable absente : Kesh démarre, les connexions sont inutilisables
  (`PROVIDER_SECRETS_KEY_MISSING`, 409). Une restauration complète **efface** les secrets (précédent : jetons de
  renouvellement révoqués à la restauration, 0.17.3) ; la connexion passe « secret à saisir ». *Raison* : une
  sauvegarde emportée ailleurs n'emporte aucun accès à un compte de paiement ; aucun secret ne dépend d'une clé que la
  sauvegarde n'a pas. **Décidé (ex-Q4)** : les secrets se ressaisissent après toute restauration, même avec la même
  clé ; le manuel d'administration le dit. **Rotation de `KESH_SECRETS_KEY` (ex-Q5)** : hors périmètre, tracée en
  issue avec la PR (C-38-13).
- **E3 — Saisie des secrets.** Administrateur seul, par **session** (écran ou `Authorization: Bearer`), jamais par une
  clé d'API (`require_not_pat`, précédent V6) ; écriture seule : les réponses rendent `secretSet` et
  `secretUpdatedAt`, jamais la valeur ; audit sans la valeur. Le contrôle « Tester la connexion » obtient un jeton et
  lit les soldes.
- **E4 — Une migration breaking, puis des lectures tolérantes (patron A30, B22).** Deux colonnes existantes
  reçoivent des valeurs neuves, **une seule fois**, par la migration breaking de 38-3 (E19) :
  `bank_imports.source_format` (`'PROVIDER_API'`) et `reconciliation_rules.match_type` (critères neufs, E14). À partir
  de ce binaire, **`BankImportSourceFormat` et `ReconciliationMatchType` se décodent avec une variante
  `Other(String)`** (le `FromStr` cesse de refuser l'inconnu ; `Copy` perdu, à vérifier aux sites à la fiche) : une
  valeur ajoutée par les Epics 39 et 40 ne demande ni migration breaking ni relèvement. Lecture d'une valeur inconnue :
  en-tête affiché « Source inconnue (`<valeur>`) », sans autre effet ; règle d'un type inconnu **jamais appliquée**,
  listée « critère d'une version ultérieure », non modifiable (`RULE_CRITERION_UNKNOWN`, 409) — refus sans
  écriture fausse. Pour Wise et Stripe, `PROVIDER_API` sert tel quel (le prestataire est porté par la connexion) :
  seuls d'éventuels critères neufs ajouteraient des valeurs, sans DDL (`match_type` devient `VARCHAR` sans `CHECK`,
  comme `source_format`). Les colonnes **créées par cet epic** — `provider`, genre de mouvement, motif d'examen, sens,
  genre TVA d'un compte, état d'un passage — se lisent aussi avec `Other(String)`. `BankAccountKind` **ne change pas**
  (B16 maintenu).
- **E5 — Passages dans leur table ; l'en-tête `bank_imports` reste, en `PROVIDER_API` (B22).** Table applicative
  `provider_fetch_runs` (connexion, déclencheur `daily`|`manual`, utilisateur, fenêtre, début, fin, état, erreur,
  comptes, nombres insérés ou mis à jour, point de reprise). **Relevé au code** : une transaction ne peut pas exister
  sans en-tête — `bank_transactions.import_id` est `NOT NULL`, FK `ON DELETE CASCADE` (`20260504000001_bank_imports.sql:58,
  79`) ; le rendre `NULL` serait un `MODIFY COLUMN` qu'un binaire antérieur ne décoderait plus (`import_id: i64`), donc
  breaking. Chaque passage crée donc, **par compte touché** (au moins une transaction insérée), une ligne `bank_imports`
  portant la colonne neuve **`provider_fetch_run_id`** (NULL, FK `provider_fetch_runs`), période = fenêtre, `file_hash`
  = SHA-256 des enregistrements normalisés, `statement_id` NULL, soldes NULL (le solde vit en E20), `filename` =
  `telechargement-<provider>-<du>-<au>` (vrai pour tout lecteur), `imported_by_user_id` = l'utilisateur du passage à la
  demande, le `created_by_user_id` de la connexion pour le passage quotidien (colonne `NOT NULL`, `:33`).
  **`source_format = 'PROVIDER_API'`** (B22) : la valeur dit vrai — lignes obtenues de l'API d'un prestataire, sans
  fichier ; elle sert aux trois prestataires (le prestataire est porté par la connexion, jointe par
  `provider_fetch_run_id`). `BankImportSourceFormat::ProviderApi` neuf, décodage tolérant (E4) ; la colonne est un
  `VARCHAR(32)` sans `CHECK` (`20260504000001_bank_imports.sql:25`) : **aucun DDL**, seul le relèvement de
  `kesh_version_min_required` protège un binaire antérieur, dont le `FromStr` refuse l'inconnu (précédent `MANUAL`,
  `20261030000001`). Affichage « Téléchargé de PayPal (connexion, passage) » — liste et détail des imports, boîte de
  suppression, export CSV de `bank_imports` (colonne `provider_fetch_run_id` ajoutée, `csv_tables.rs:702-720`),
  audit, libellé du frontend (`bank-import-display.ts:14-36`). Aucun comportement ne dépend de `source_format`
  (lecteurs relevés à l'inventaire : affichage seul). La suppression d'un tel en-tête par `DELETE /bank-imports/{id}`
  est **refusée** (`PROVIDER_IMPORT_NOT_DELETABLE`) : le recouvrement de trois jours ne retéléchargerait pas ce qu'elle
  effacerait. *Raison* : DRY (la réconciliation lit `bank_transactions` et rien d'autre) ; le détour par `MANUAL`
  (valeur inexacte, ex-Q11) est retiré (B22).
- **E6 — Doublon par l'identifiant du prestataire.** Colonne neuve `provider_tx_id`, `UNIQUE (bank_account_id,
  provider_tx_id)` (NULL multiples admis : CAMT, CSV et manuel inchangés). Insertion idempotente
  (`INSERT … ON DUPLICATE KEY` sans écrasement) ; chaque passage **recouvre** les trois derniers jours (délai de 3 h de
  PayPal, statuts tardifs). La clé de doublon par fenêtre (`kesh-core/src/bank_imports.rs:295`) ne s'applique pas.
- **E7 — Une transaction de prestataire = un mouvement du solde.** `amount` = **net** (ce qui change le solde),
  plus `gross_amount` et `fee_amount` ; frais **dans** la même transaction (B3 : brut par la règle, frais au
  sous-compte). Une conversion PayPal donne **deux** transactions (une par solde), reliées par
  `related_provider_tx_id`.
- **E8 — Colonnes génériques, pas de JSON brut.** `bank_transactions` gagne `movement_kind`, `event_code`,
  `counterparty_email`, `invoice_ref` (PayPal : `invoice_id`, dont le préfixe désigne la boutique, B24),
  `item_names`, `related_provider_tx_id`, `review_reason` (toutes NULL ; les lectures du dépôt nomment leurs
  colonnes, `repositories/bank_transactions.rs:21` : un binaire antérieur les ignore). **Plus de `shop_ref`** ni de
  réglage `shop_source` : la boutique se lit au préfixe d'`invoice_ref` (B24, E14). Le connecteur remplit aussi les
  colonnes existantes quand elles disent vrai : `counterparty_name` = nom du payeur ou du marchand, `reference` =
  numéro de facture du prestataire, `details` = objet de la transaction. *Raison* :
  les règles filtrent en SQL et en Rust sur des champs nommés, communs aux trois prestataires ; la réponse brute n'est
  pas gardée (données personnelles des payeurs, B12).
- **E9 — Nouveau crate `kesh-providers`.** Types communs (`ProviderMovement`, `MovementKind`), trait de transport
  objet-safe (forme `RateSource` de 37-2 : `Pin<Box<dyn Future…>>`), connecteurs (`paypal` ici, `wise`, `stripe`
  ensuite), validation stricte des réponses (R5 de 37). Sans dépendance à `kesh-db` (comme `kesh-import`, qui reste
  publiable et n'est pas touché). L'implémentation `reqwest` vit dans `kesh-api` et **réutilise le client borné de
  37-2**. *Raison* : le classement et la pagination se testent sur réponses enregistrées sans base ni réseau (B12).
- **E10 — Planification.** Une tâche unique (patron `auto_backup::spawn`, `auto_backup.rs:788`) parcourt les
  connexions `daily` une fois par jour ; `POST /provider-connections/{id}/fetch` télécharge à la demande (Comptable ou
  Administrateur). Un verrou par connexion empêche deux passages simultanés. Passage automatique non audité (`tracing`
  + état affiché), passage à la demande audité (précédent C-37-6). Aucun passage en `KESH_TEST_MODE` ; transport
  factice piloté par une route `_test`.
- **E11 — Compte bancaire prestataire = compte `bank` + réglages de connexion (B16).** Le compte bancaire reste
  `account_kind = 'bank'`, sans IBAN (admis depuis 34-11a1), sans QR-IBAN ni carte. Son caractère « prestataire » vit
  dans la table applicative neuve **`payment_provider_accounts`** : `bank_account_id` (PK, FK), `connection_id` (FK
  `payment_provider_connections`), `currency` (devise du solde), `fee_account_id` (compte de frais **de cette devise**,
  B3), `version` ; colonnes `NULL` à venir pour d'autres comptes liés (Epics 39, 40) ; `UNIQUE (connection_id,
  currency)`. **Toute référence à un compte est une colonne FK `ON DELETE RESTRICT`**, gardée à l'archivage du compte
  visé (un compte de frais ou lié ne s'archive pas tant qu'un réglage le désigne) — jamais une clé du JSON (B23,
  E24). Gardes, au serveur : compte du plan lié **actif, de bilan (actif)**, **dans la devise du solde** — CHF
  admis pour le solde CHF ; `fee_account_id` de charge, actif, dans la même devise (`FEE_ACCOUNT_INVALID`) ; le
  regroupement des frais est son **compte parent** (`accounts.parent_id`), aucun réglage de plus ; jamais principal
  (`is_primary` refusé, la QR-facture lit le compte principal, `invoice_pdf_service.rs:332`). La ligne de réglages et
  le lien au compte du plan en devise se posent **dans la même transaction** ; c'est elle qui lève C-37-9 (D8). Retirer
  la ligne est refusé tant que le compte est lié à un compte du plan en devise, a des transactions de prestataire ou un
  solde vu (`PROVIDER_ACCOUNT_IN_USE`). Lier un compte bancaire **existant** garde son numéro, son historique et ses
  soldes. Le binaire de l'epic lit cette table partout où un compte prestataire doit être écarté : compte principal,
  lots de paiement pain.001 (déjà refusés sans IBAN depuis 34-11a1 : **à vérifier à la fiche**), assistant
  d'installation. `account_kind` reste `'bank'` (B16, maintenu par B22).
- **E12 — Nouveau solde (B2).** Le passage lit les soldes de la connexion ; un solde de devise sans compte Kesh (sans
  ligne `payment_provider_accounts` de cette devise) est gardé dans `provider_balances_seen` (connexion, devise,
  premier vu, ignoré ou non) et **listé** à l'écran ; ses
  transactions ne sont pas téléchargées tant que l'Administrateur n'a pas créé ou lié le compte. Jamais de création
  d'office.
- **E13 — Code TVA par défaut d'un compte (B6).** `accounts.default_vat_kind` (`sale` : TVA due incluse dans le
  montant ; `purchase` : impôt préalable inclus ; `acquisition` : impôt sur les acquisitions, calculé sur le montant,
  deux lignes égales dû/préalable ; NULL : sans TVA) et `accounts.default_vat_rate_id` (FK `vat_rates`). Extraction
  pure dans `kesh-core` (`vat_from_gross` = round(brut × t ÷ (100 + t), 2) ; acquisitions = round(montant × t ÷ 100,
  2)). Lignes de TVA **en devise principale**, au cours de la ligne du compte de produit ou de charge (S5, A13). Comptes
  de TVA lus aux réglages existants (`company_invoice_settings`) : TVA due des ventes au compte de TVA due, impôt
  préalable au compte d'impôt préalable, et **impôt sur les acquisitions au compte neuf
  `default_vat_acquisition_account_id`** (B18, colonne `NULL` des mêmes réglages, compte de passif, p. ex. 2201) — la
  ligne de préalable des acquisitions va au compte d'impôt préalable. Un réglage manquant → `FailedProposal`
  `VAT_ACCOUNT_NOT_CONFIGURED`, `details.designation` ∈ `payable`, `recoverable`, `acquisition`.
- **E14 — Règles étendues (B5, B19, B22, B24) : types neufs de `match_type`, filtres en colonnes.** Plus de table
  enfant ni de clé inerte (B22). **Recensement des critères** de B5, B19 et B24 :

  | Critère demandé | Devient | Compare à |
  |---|---|---|
  | courriel du payeur ou du marchand (B5) | `match_type = 'counterparty_email_exact'` | `bank_transactions.counterparty_email`, normalisé (casse, espaces) |
  | préfixe de facture (B5) = **boutique / domaine** (B19, B24) | `match_type = 'invoice_prefix'`, libellé « Boutique (préfixe du numéro de facture) » | début d'`invoice_ref` (PayPal : `invoice_id`), casse et espaces ignorés |
  | nom d'article (B5) | `match_type = 'item_name_contains'` | `item_names` (un des noms contient la valeur) |
  | code d'événement du prestataire (B5) | `match_type = 'event_code_prefix'` | `event_code` (p. ex. `T0006` ou `T00`) |
  | sens entrée / sortie (B5) | **filtre**, colonne `direction` NULL (`in`\|`out`) | signe de `amount` |
  | devise (B19, vente par devise) | **filtre**, colonne `currency` NULL (`CHAR(3)`) | `bank_transactions.currency` |
  | compte bancaire (« par connexion ») | **filtre**, colonne `bank_account_id` NULL, FK `ON DELETE RESTRICT` | compte de la transaction |

  *Pourquoi le sens est un filtre* : seul, il n'est pas discriminant (il rapprocherait toute entrée) ; il départage une
  vente et un achat d'un même courriel. **Un critère principal par règle** (`match_type`/`match_value`, décidé ex-Q7),
  plus les filtres facultatifs, applicables aussi aux quatre critères existants. **Migration** : `match_type` passe
  d'`ENUM` à **`VARCHAR(40) NOT NULL`** sans `CHECK` dans la migration breaking de 38-3 (E19), valeurs existantes
  conservées ; les valeurs admises sont tenues par l'API à l'écriture (aujourd'hui par la désérialisation de l'enum en 400 ;
  désormais `Other` refusé, `RULE_CRITERION_UNKNOWN`) et le décodage est tolérant (E4) — Wise et Stripe ajoutent un critère sans DDL. Les **filtres** et
  l'**unicité** sont la migration **non breaking** de 38-4 : trois colonnes `NULL`, une colonne générée
  `filter_key` (sens, devise, compte, `-` pour NULL) et l'index unique `uq_reconciliation_rules_match_active`
  (`20260513000001:65`) remplacé par `(company_id, match_type, active_uniq, filter_key)` — **une règle par boutique et
  par devise** est admise (B19), deux règles de mêmes type, valeur et filtres restent refusées
  (`RECONCILIATION_RULE_DUPLICATE`). `rule_matches` (`rules.rs:46`) évalue le critère puis les filtres ; priorité
  inchangée (`priority`, `id`). Le compte de la règle peut être **en devise** : il ne s'applique alors qu'aux
  transactions de cette devise (`RULE_ACCOUNT_CURRENCY_MISMATCH` par proposition). **Ventes (B19)** : une règle
  `invoice_prefix` par boutique et par devise vers le sous-compte de ventes de la devise ; l'analytique
  (`default_project_id`, `20260704000001`) peut porter la catégorie, **reporté** (Q3, B25). Les routes de règles
  gardent leur forme (`matchType`, `matchValue`) et gagnent `direction`, `currency`, `bankAccountId` ; l'export CSV des
  règles ajoute les trois colonnes.
- **E15 — Propositions de prestataire.** Une transaction de prestataire reçoit au plus une proposition, construite par
  une fonction pure de `kesh-reconciliation` : compte du prestataire (net, en devise) ; contrepartie de la règle (brut
  hors TVA) ; TVA (E13) ; frais au `fee_account_id`. Cours = cours retenu `afc` de la société pour le mois
  (`find_company_rate_in_tx`), A32 sinon ; cours manquant → `RATE_UNAVAILABLE` par proposition. Acceptation par
  `POST /reconciliation/accept`, variante neuve `provider` (*Pattern batch* : toute erreur par proposition en
  `failed[]`, identifiant `bank_transaction_id`, codes constants). Aucune écriture d'office. `get_proposals` saute
  aujourd'hui toute transaction hors CHF (`reconciliation.rs:1098-1104`) : le saut est levé **pour les seuls comptes
  prestataires** (E11). **Déjà comptabilisée (34-4, B17)** : une proposition de prestataire porte `alreadyBooked` ;
  l'écran l'exclut de « tout accepter », et le lot exige `confirmAlreadyBooked: true` par proposition, sinon
  `FailedProposal` `TRANSACTION_ALREADY_BOOKED` — écart assumé à 34-4 AC-4 (« le serveur ne refuse rien »), justifié
  par le volume : un lot de 2024 doublerait sinon les écritures en silence. La voie de ces transactions est le
  rattachement en lot (E21).
- **E16 — Mouvements particuliers.**
  - **Conversion** : la paire (E7) se propose comme **une conversion** de 37-3b, `completeDifference: true` (B4,
    A31), cours AFC des deux côtés, différence aux comptes propres du compte en devise (S2) ; les deux transactions
    sont réconciliées ensemble. Coût donné à part par le prestataire → ligne de frais.
  - **Retrait vers la banque / dépôt depuis la banque** : rattachement des deux côtés comme un virement entre ses
    comptes (34-11b, `reconciliation.rs:6373`), le côté banque venant du relevé CAMT.
  - **Remboursement (B7, B20)** : rapproché de l'origine par `related_provider_tx_id` ; proposé comme l'**inverse** de
    l'écriture d'origine — mêmes comptes, **montants tels que le prestataire les rapporte** : brut remboursé, frais
    rendus (`fee_amount` du remboursement, nul si PayPal n'en rend pas), net ; la TVA est extraite du brut remboursé au
    taux de l'origine (E13), sans prorata calculé par Kesh. Origine absente ou non réconciliée → file « à examiner ».
    *(Ex-Q6, close par B20.)*
  - **Litige, rétrofacturation, mouvement inconnu** : `review_reason` posé, **aucune proposition** ; file « à
    examiner » ; l'utilisateur passe une écriture manuelle (34-3) ou marque examiné.
  - **Blocages, réserves, autorisations** sans effet sur le solde : non téléchargés (`balance_affecting_records_only`).
- **E17 — Relevé joint (B10, B20).** Table applicative `bank_account_documents` (compte bancaire, exercice, genre
  `statement`, période, chemin, empreinte, nom d'origine, auteur, **solde de clôture et devise** saisis à la main,
  facultatifs — la somme de contrôle de E20), fichiers par `document_storage`, emportés par la sauvegarde
  (`ParsedBackup::referenced_files`, `admin_backup/import.rs:115`). Wise (39) y range le PDF obtenu par l'API.
- **E18 — Fenêtre d'import des sauvegardes.** Les tables et colonnes applicatives neuves de l'Epic 38 s'ajoutent à la
  **liste close d'absences admises** de 37-1 AC-6 (R1) : une sauvegarde antérieure se réimporte (sans connexion ni
  donnée de prestataire).
- **E19 — Migrations : une seule breaking, dans 38-3 (B22).** *Où et pourquoi là* : 38-3 est la **première** story
  qui écrit une valeur neuve dans une colonne existante à domaine fermé (`source_format = 'PROVIDER_API'`, à chaque
  passage) ; les types neufs de `match_type` (38-4) dépendent de 38-3 (colonnes `invoice_ref`, `event_code`…) et
  viennent après. Les deux changements partagent donc **une** migration et **un** relèvement ; 38-4 n'a plus qu'une
  migration non breaking. La migration breaking est un **fichier à part** de la migration non breaking de 38-3 (tables
  et colonnes), pour que son exemption P7 ne couvre qu'elle.
  - **Fichier** `<date>000002_provider_api_source_and_rule_types.sql` (nom à la fiche), en tête `-- BREAKING (P1) :
    relève kesh_version_min_required à <X.Y.Z>` (patron `20261030000001`). Contenu : (1) `ALTER TABLE
    reconciliation_rules MODIFY COLUMN match_type VARCHAR(40) NOT NULL` — `MODIFY COLUMN <type>` (P3 : bump
    obligatoire) ; les quatre valeurs existantes restent, aucune ligne réécrite ; l'index unique qui porte la colonne
    se reconstruit à l'`ALTER` (vérifier à la fiche qu'aucun `DROP INDEX` n'est requis) ; (2) **aucun DDL** pour
    `source_format` (`VARCHAR(32)` sans `CHECK`) ; (3) **en dernière instruction** `UPDATE _kesh_version SET
    kesh_version_min_required = '<X.Y.Z>' WHERE id = 1;` (P2).
  - **P2-bis** : les dix crates passent à `<X.Y.Z>` **dans le même commit** que la migration ; gate **runtime complet**
    (`admin_backup_e2e`, `admin_full_import_e2e`, `migrations_fresh_install`) avant `done`. **`<X.Y.Z>` est à Guy
    (Q1)** : la fiche 38-3 ne s'écrit pas sans lui ; s'il change après coup, la migration, non publiée, se corrige
    délibérément (P8 : bases de dev reconstruites).
  - **P3/P4** : pas de `breaking-skip-bump` (un binaire antérieur ne décode ni `'PROVIDER_API'` ni les critères neufs :
    `FromStr` refusant l'inconnu, `entities/bank_import.rs:59`, `entities/reconciliation_rule.rs:68`).
  - **P5** : une ligne à `docs/migrations-idempotence-audit.md` par fichier, compteurs recomptés à la source.
  - **P6** : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` à chaque fiche qui ajoute une migration.
  - **P7** : l'`UPDATE _kesh_version` écrit une table système jamais restaurée → `EXEMPT_MIGRATIONS`, justification
    écrite (précédent `20261030000001`) ; les autres migrations de l'epic n'écrivent pas de données.
  - **P8** : aucune migration appliquée ne se retouche.
  - **Décodage tolérant ensuite (E4)** : `BankImportSourceFormat` et `ReconciliationMatchType` gagnent `Other(String)`
    dans la même story ; Wise (39) et Stripe (40) n'ont plus besoin de relever `kesh_version_min_required` pour ces
    deux colonnes.
  - **Retour** : un binaire antérieur refuse de démarrer sur la base migrée et d'importer un `.keshbackup` de cette
    version (version minimale) ; **le retour à la version précédente passe uniquement par la sauvegarde d'avant la mise
    à jour** (`backup/pre-upgrade/`, #847, et celle que les notes exigent), à écrire en tête des notes de version et au
    manuel d'administration (38-9a). Le SQL de retour de la révision B16–B21 est **retiré**. Recette de montée
    (`scripts/recette-montee.sh`) depuis l'image publiée précédente, jouée en 38-3 et à la release.
  - `account_kind` : inchangé, seul `'bank'` est écrit pour un compte prestataire (lecteurs relevés à l'inventaire).
  - Colonnes neuves de `bank_transactions`, `bank_imports`, `reconciliation_rules`, `accounts`,
    `company_invoice_settings` : `NULL`, lues par listes de colonnes nommées ; tables neuves : `CREATE TABLE`. Liste close
    R1 (E18) à chaque fiche.
- **E20 — Le relevé est la somme de contrôle (B20).** Chaque passage enregistre, par devise, le solde rapporté et son
  heure (`provider_balance_snapshots` : connexion, devise, `as_of`, solde, passage) ; le premier passage lit aussi le
  solde **à `fetch_from`** (PayPal : `as_of_time`). Deux contrôles, **écarts signalés** (jamais d'écriture) :
  (a) **complétude du téléchargement** — solde à `fetch_from` + Σ nets téléchargés ≠ solde rapporté → « téléchargement
  incomplet » sur la connexion et le compte, avec l'écart ; (b) **écart comptable** — solde du compte du plan **en
  devise** à `as_of` ≠ solde rapporté → écart affiché à l'écran du compte, ventilé en transactions non rapprochées et
  reste inexpliqué. Le **relevé joint** (38-10, PDF annuel ou mensuel) porte son solde de clôture par devise, saisi à
  la main ; il est comparé au solde comptable à sa date, et un écart s'ajoute aux contrôles préalables à la clôture
  de l'exercice (`GET /fiscal-years/{id}/closing-checks`, avertissement non bloquant).
- **E21 — Rattachement en lot aux écritures existantes (B17).** Les livres 2024 de Beat sont déjà passés par l'API
  (recette de l'Epic 37) ; les transactions téléchargées de 2024 se **rattachent** à ces écritures, sans en créer.
  Socle : le rattachement à une écriture déjà passée (v017-3a, `get_entry_candidates` `reconciliation.rs:6548`,
  `post_link_entry` `:6656`) et le signal « déjà comptabilisée » (34-4, `count_already_booked_for_page`
  `reconciliation_link.rs:934-1000`, mêmes textes SQL que le dialogue). **Deux extensions** : (1) **en devise** —
  `transaction_can_be_linked_to_entry` n'admet que le CHF (`reconciliation_link.rs:535-537`) ; pour un compte
  prestataire dont le compte du plan est en devise X et une transaction en X, le mouvement `M` se lit sur le montant
  en devise des lignes (37-3a), même tolérance, mêmes conditions ; (2) **en lot** — `GET
  /reconciliation/link-suggestions?bankAccountId&from&to` rend, par transaction en attente, l'écriture **univoque**
  (une seule candidate, aucun règlement candidat, aucune déjà rapprochée ailleurs) ou « ambiguë » (liste) ; `POST
  /reconciliation/link-entries` rattache un lot `[{bankTransactionId, journalEntryId, version}]` (*Pattern batch* :
  `ENTRY_NOT_LINKABLE`, `ENTRY_ALREADY_LINKED`, `AMOUNT_MISMATCH`, `VERSION_CONFLICT` en `failed[]`). Rien n'est
  rattaché d'office : l'utilisateur coche (écran 38-9b). Une conversion (deux transactions, une écriture qui touche
  deux comptes en devise) se rattache des deux côtés comme un virement entre ses comptes (34-11b).
- **E22 — L'API des scripts externes (B21).** Un script hors de Kesh (exports CSV de WHMCS et de CB Paid
  Subscriptions) passe par une **clé d'API `read-write`**. Relevé au code : `POST /journal-entries` (écritures
  multi-lignes, en devise par 37-3a), `POST /reconciliation/accept`, `/manual` et `…/transactions/{id}/link-entry` sont
  des routes Comptable où les clés `read-write` sont admises (`lib.rs:523-545, 921-929` ; `require_not_pat` ne garde
  que l'administration, `:518`). **Manques**, comblés par 38-6c : (1) retrouver la transaction de prestataire d'une
  facture — aucune route ne cherche par identifiant du prestataire ni par numéro de facture : `GET
  /bank-accounts/{id}/transactions?providerTxId&invoiceRef&status&from&to` (lecture, tout rôle, clés admises) ; (2)
  rattacher une transaction **en devise** (E21, (1)) ; (3) le lot `link-entries` (E21), clés admises. Le script crée
  l'écriture puis la rattache (deux appels) ; un rattachement échoué laisse l'écriture, que le signal « déjà
  comptabilisée » montre ensuite sur la transaction. Les routes des connexions et des secrets restent fermées aux clés
  (E3).
- **E23 — Aucun outil MCP neuf (décidé, ex-Q8).** Les écritures passées par les propositions se lisent par
  `kesh_ecritures` (`mcp/outils/ecritures.rs:198`) ; connexions, passages et file « à examiner » ne sont pas exposés.
- **E24 — Réglages d'une connexion : `settings JSON` typé par prestataire (B23).** Kesh n'a pas de magasin générique
  de paramètres ; il a des colonnes `JSON` lues en struct. **Patron suivi** : `bank_profiles.column_mapping JSON NOT
  NULL` + `CHECK (JSON_VALID(column_mapping))` (`20260505000001_bank_profiles.sql:27, 48-49`), lu en chaîne
  (`column_mapping_json`) puis par `BankProfile::parse_column_mapping` (`serde_json::from_str`, échec →
  `DbError::Invariant`, `entities/bank_profile.rs:50-57`) en `ColumnMapping` `#[serde(deny_unknown_fields)]`
  (`kesh-import/src/csv/profile.rs:36-48` : une clé inconnue est **refusée bruyamment**, jamais ignorée) ; `CHECK
  (JSON_TYPE(…) = …)` comme `mcp_server_settings` (`20261025000001:81-83`). MariaDB : `JSON` = `LONGTEXT` validé, pas
  de JSONB ; aucune requête ne filtre dans le JSON.
  - **Colonne** `payment_provider_connections.settings JSON NOT NULL DEFAULT '{}'`, `CHECK (JSON_VALID(settings) AND
    JSON_TYPE(settings) = 'OBJECT')` (38-1).
  - **Lecture** : `settings_json: String` dans l'entité, `parse_settings(provider)` → `ProviderSettings::Paypal(PaypalSettings)`
    | `Other` (prestataire inconnu : connexion listée, inutilisable, `PROVIDER_UNKNOWN`) ; structs dans
    `kesh-core::providers::settings` (serde seul, sans I/O ; lues par `kesh-api` et `kesh-providers`),
    `#[serde(deny_unknown_fields, rename_all = "camelCase")]`, champs ajoutés plus tard en `Option` + `default`. Un JSON
    illisible → connexion en erreur `PROVIDER_SETTINGS_INVALID`, aucun passage, jamais de repli silencieux.
  - **Écriture** : seule l'API écrit, après désérialisation dans la struct du prestataire puis resérialisation
    (aucun JSON reçu stocké tel quel) ; `PATCH` avec `version`.
  - **Ce qui va dans le JSON** — les options **propres au prestataire**, sans référence à un compte ni secret :
    - **PayPal** : `clientId` (identifiant de l'application REST : non secret chez PayPal, affiché pour reconnaître
      l'application ; le *client secret* va au coffre, E2) ; `accountEmail` et `payerId` du compte *business*, posés
      par « Tester la connexion » (38-7), affichés, et servant à reconnaître un virement entre les deux comptes PayPal
      de Beat ; `eventKindOverrides` facultatif (code `T…` → genre de mouvement, pour un code que la table de 38-8 ne
      classe pas ; jamais `sale` pour un code inconnu sans ce réglage explicite — à trancher à la fiche 38-8).
    - **Wise (39)** : `profileId` ; empreinte de la clé publique SCA déposée et sa date (renouvellement ≈ 90 jours ; la
      clé privée au coffre) ; type et langue du relevé PDF ; identifiants de solde par devise (`balanceIds`) si la fiche
      ne préfère pas une colonne.
    - **Stripe (40)** : `accountId` (`acct_…`) ; version d'API épinglée (`Stripe-Version`) ; contrôle `livemode`.
  - **Ce qui n'y va pas** : `fetch_from`, `schedule`, état du dernier passage (colonnes communes, lues par le
    planificateur) ; **toute référence à un compte** (frais par devise, compte intermédiaire de Stripe) : colonnes FK de
    `payment_provider_accounts` (E11) ; **secrets** : `_kesh_provider_secrets` chiffrée (E2).

## Registre des contrats entre stories

- **C-38-0 — Une seule migration breaking, dans 38-3** (toutes) : E19, B22 ; relèvement en dernière instruction,
  crates bumpés dans le même commit, numéro de release à Guy (Q1). Toute autre fiche qui découvre une valeur neuve à
  écrire dans une colonne existante à domaine fermé, un `MODIFY`/`CHANGE COLUMN` ou un `DROP` s'arrête et le dit.
- **C-38-1 — Clé et chiffrement** (38-1) : `KESH_SECRETS_KEY`, `kesh-api::secrets::{seal, open}`, AES-256-GCM, version
  de clé dans l'enregistrement ; rotation : hors périmètre, issue (C-38-13). Seule voie de lecture d'un secret : le
  transport d'un passage. Secrets effacés à toute restauration (E2).
- **C-38-2 — Connexions** (38-1) : `payment_provider_connections` (`id`, `company_id`, `provider`, `label`,
  `fetch_from`, `schedule`, `last_run_at`, `last_run_status`, `last_error`, `created_by_user_id`, `settings` JSON typé (E24), `version`) ; routes
  `GET|POST /api/v1/provider-connections`, `GET|PATCH|DELETE /{id}`, `PUT /{id}/secrets`, `POST /{id}/test` ;
  suppression refusée tant qu'une ligne `payment_provider_accounts` y est rattachée (`PROVIDER_CONNECTION_IN_USE`) ;
  `settings` validé par la struct du prestataire (`PROVIDER_SETTINGS_INVALID`), `PROVIDER_UNKNOWN` en lecture.
- **C-38-3 — Réglages de compte prestataire** (38-2) : E11 ; `payment_provider_accounts` ; routes `GET|POST
  /api/v1/provider-connections/{id}/accounts` (créer le compte bancaire `bank` et ses réglages, ou lier un compte
  bancaire existant), `PATCH|DELETE /…/accounts/{bankAccountId}` ; codes `PROVIDER_ACCOUNT_CURRENCY_MISMATCH`,
  `PROVIDER_ACCOUNT_DUPLICATE_CURRENCY`, `FEE_ACCOUNT_INVALID`, `PROVIDER_ACCOUNT_IN_USE`,
  `PROVIDER_ACCOUNT_CANNOT_BE_PRIMARY` ; levée de C-37-9 pour un compte bancaire porteur de réglages ; `account_kind`
  toujours `bank`.
- **C-38-4 — Transaction de prestataire** (38-3) : colonnes E6, E7, E8 ; `movement_kind` ∈ `sale`, `purchase`, `fee`,
  `conversion`, `refund`, `dispute`, `withdrawal`, `deposit`, `other` (tolérant) ; `review_reason` ∈ `dispute`,
  `refund_origin_missing`, `unknown_event`, `other` (tolérant).
- **C-38-5 — Passage** (38-3) : E5 ; `provider_fetch_runs`, `bank_imports.provider_fetch_run_id`, en-tête
  `source_format = 'PROVIDER_API'` (migration breaking, E19), affiché « Téléchargé de … », suppression refusée
  (`PROVIDER_IMPORT_NOT_DELETABLE`) ; `BankImportSourceFormat` et `ReconciliationMatchType` tolérants (E4) ;
  `kesh_providers::Connector` (`balances(as_of)`, `movements(window, page)`) ; fenêtre ≤ 31 jours, découpée ;
  recouvrement de 3 jours ; fin = maintenant − 3 h ; début = max(`fetch_from`, dernier passage réussi − 3 jours) ; un
  échec garde ce qui est inséré, l'état dit où reprendre ; soldes rapportés et contrôle de complétude (E20, (a)).
- **C-38-6 — Règles** (38-4) : E14 ; `matchType` ∈ quatre valeurs existantes + `counterparty_email_exact`,
  `invoice_prefix` (boutique, B24), `item_name_contains`, `event_code_prefix` ; filtres facultatifs `direction`,
  `currency`, `bankAccountId` ; unicité `(type, valeur, filtres)` ; `RULE_CRITERION_UNKNOWN` (écriture d'une valeur
  inconnue : 400 ; modification d'une règle d'un type inconnu : 409).
- **C-38-7 — TVA par défaut** (38-5) : E13 ; `PUT /accounts/{id}` gagne `defaultVatKind`, `defaultVatRateId` ;
  réglages TVA gagnent `defaultVatAcquisitionAccountId` (B18) ; `kesh_core::accounting::vat::{vat_from_gross,
  acquisition_vat}` et leur miroir TS, tenus par une fixture partagée.
- **C-38-8 — Propositions** (38-6a, 38-6b) : E15, E16 ; variante `provider` de `accept`, `confirmAlreadyBooked` ; codes
  `RATE_UNAVAILABLE`, `VAT_ACCOUNT_NOT_CONFIGURED`, `RULE_ACCOUNT_CURRENCY_MISMATCH`, `FEE_ACCOUNT_INVALID`,
  `PROVIDER_TX_UNDER_REVIEW`, `TRANSACTION_ALREADY_BOOKED`, `MULTI_CURRENCY_DISABLED` (409 hors lot, `failed[]` dans le
  lot).
- **C-38-9 — File « à examiner »** (38-6b) : `GET /api/v1/reconciliation/review` ; `POST
  /reconciliation/transactions/{id}/review-done` (Comptable).
- **C-38-10 — Pièces de compte** (38-10) : E17 ; `GET|POST /api/v1/bank-accounts/{id}/documents`,
  `GET|PATCH|DELETE /…/documents/{docId}` ; PDF seul, taille bornée ; solde de clôture facultatif ; écart au solde
  comptable aux contrôles de clôture (E20).
- **C-38-14 — Rattachement aux écritures existantes et API des scripts** (38-6c) : E21, E22 ; `GET
  /reconciliation/link-suggestions`, `POST /reconciliation/link-entries` (lot), `GET
  /bank-accounts/{id}/transactions` (recherche) ; `link-entry` et `entry-candidates` en devise ; clés `read-write`
  admises sur les écritures et le rattachement, lecture pour la recherche ; codes `ENTRY_NOT_LINKABLE`,
  `ENTRY_ALREADY_LINKED`, `AMOUNT_MISMATCH`, `VERSION_CONFLICT`.
- **C-38-11 — Tests** (toutes) : aucun appel réseau en CI ; fixtures enregistrées **anonymisées** (courriels, noms,
  identifiants remplacés ; montants et codes gardés) sous `crates/kesh-providers/tests/fixtures/paypal/` ; tests de
  réponses hostiles (taille, structure, montants, dates, devises, pagination incohérente).
- **C-38-12 — Registre des routes** : chaque route neuve entre à `LIB_ROUTES` (et `TEST_ENDPOINT_ROUTES` pour la route
  `_test`), décompte en deltas à chaque fiche.
- **C-38-13 — Tracé pour plus tard** (issues avec la PR) : rotation de `KESH_SECRETS_KEY` (décidé, ex-Q5) ; TVA due
  des ventes hors facture et impôt sur les acquisitions dans le rapport TVA (voir *Point TVA pour Guy*) ; catégorie
  de vente par analytique (B19, Q3, reportée B25). *(L'unicité des règles par sens ou devise n'y est plus : réglée par
  `filter_key`, E14.)* *(Les
  outils MCP n'y sont plus : décidé, aucun outil neuf, E23.)*

## Stories

| Story | Objet | Modules (≤ 5) | Dépend de | Migration |
|---|---|---|---|---|
| 38-0 | kickoff : décomptes, validation de ce fichier, numéro de release (Q1) | — | — | — |
| 38-1 | coffre des secrets et connexions de prestataire, réglages `settings JSON` (E24) | `kesh-db`, `kesh-core` (structs de réglages), `kesh-api` (config, secrets, routes) | Epic 37 | oui, non breaking (`payment_provider_connections`, `_kesh_provider_secrets`) |
| 38-2 | compte bancaire `bank` + réglages de prestataire, lié à un compte en devise existant (D8), compte de frais par devise | `kesh-db`, `kesh-api` (bank_accounts, provider-connections), `kesh-core` (gardes) | 38-1 | oui, non breaking (`payment_provider_accounts`) |
| 38-3 | téléchargement : cadre `kesh-providers`, transactions de prestataire, doublon, passages (`PROVIDER_API`), planifié et à la demande, soldes vus et rapportés, contrôle de complétude ; **migration breaking** et décodage tolérant | `kesh-providers` (neuf), `kesh-db`, `kesh-core`, `kesh-api` | 38-2, **numéro de release (Guy, Q1)** | oui : une non breaking (`provider_fetch_runs`, `provider_balances_seen`, `provider_balance_snapshots` ; colonnes `bank_transactions`, `bank_imports.provider_fetch_run_id`) **et la seule breaking de l'epic** (`match_type` → `VARCHAR`, `min_required` relevé, crates bumpés, E19) |
| 38-4 | critères neufs des règles (types de `match_type`, dont boutique = préfixe de facture), filtres sens, devise, compte | `kesh-db`, `kesh-reconciliation`, `kesh-api` (reconciliation_rules) | 38-3 | oui, non breaking (trois colonnes de filtre, `filter_key`, index unique remplacé) |
| 38-5 | code TVA par défaut d'un compte, extraction de la TVA, compte d'impôt sur les acquisitions | `kesh-db`, `kesh-core`, `kesh-api` (accounts, réglages TVA), `frontend` (accounts, réglages) | Epic 37 | oui, non breaking (`accounts` × 2, `company_invoice_settings` × 1) |
| 38-6a | propositions de prestataire : vente, achat, frais (brut, TVA, frais, cours) et acceptation en lot ; « déjà comptabilisée » | `kesh-reconciliation`, `kesh-api` (reconciliation), `kesh-db` | 38-3, 38-4, 38-5 | non attendue |
| 38-6b | conversions, retraits, remboursements (tels que rapportés), litiges ; file « à examiner » | `kesh-reconciliation`, `kesh-api` (reconciliation, conversions), `kesh-db` | 38-6a | non attendue |
| 38-6c | rattachement aux écritures existantes : en devise, en lot (2024, B17) ; API des scripts externes (B21) | `kesh-db` (reconciliation_link, bank_transactions), `kesh-api` (reconciliation, bank_accounts) | 38-3 | non attendue |
| 38-7 | client PayPal : OAuth2, Transaction Search, fenêtres de 31 jours, pagination, soldes (`as_of_time`) | `kesh-providers` (paypal), `kesh-api` (transport) | 38-3 | non |
| 38-8 | correspondance des codes d'événement PayPal vers les mouvements ; `invoice_id` → `invoice_ref` (boutique, B24) ; surcharges `eventKindOverrides` | `kesh-providers` (paypal), `kesh-core` (réglages) | 38-7 | non |
| 38-9a | écrans : connexions, réglages, secrets, soldes proposés, comptes prestataires, contrôles de solde | `frontend` (provider-connections neuf, bank-accounts), i18n, manuel d'administration (sauvegarde d'avant la mise à jour obligatoire, E19) | 38-1 à 38-3, 38-7 | non |
| 38-9b | écrans : règles étendues, propositions multi-lignes, rattachement en lot, file « à examiner », E2E | `frontend` (reconciliation, règles), i18n, manuel utilisateur, E2E | 38-4 à 38-6c, 38-8 | non |
| 38-10 | relevé joint à la main (compte et exercice ou mois), solde de clôture, écart aux contrôles de clôture | `kesh-db`, `kesh-api` (routes, sauvegarde, closing-checks), `frontend` (bank-accounts) | 38-2, 38-3 | oui, non breaking (`bank_account_documents`) |

**Migrations : sept fichiers, dont une seule breaking** (38-1, 38-2, 38-3 × 2 — une non breaking, **la breaking** —,
38-4, 38-5, 38-10) ; seule la breaking écrit une donnée (`_kesh_version`, exemptée P7) ; les autres n'en écrivent pas
(sauf découverte à la fiche). **Ordre** : 38-1 → 38-2 → 38-3 → (38-4, 38-5, 38-6c, 38-7 en parallèle
possibles) → 38-8 → 38-6a → 38-6b → 38-9a → 38-9b ; 38-10 dès 38-3. 38-6c avant 38-6a de préférence : les transactions
de 2024 se rattachent avant que des propositions ne leur soient faites. Une story qui découvre une migration non
prévue le dit avant de l'écrire (C-38-0).

Chaque fiche porte ses **exigences numérotées** (B26) : `FR-38-n-k` (fonctionnelles), `NFR-38-n-k` (non
fonctionnelles) ; récapitulatif au § *Exigences (FR/NFR) par story*. Les gardes communes (P1–P8, *Pattern batch*,
aucun appel réseau en CI, recette visuelle) ne s'y répètent que là où la story les porte en propre.

### 38-0 — Kickoff

- *Intention* : ouvrir l'epic sur des décomptes mesurés et un fichier validé.
- *Périmètre* : décomptes d'issues et de bugs (§ *Priorités des défauts*), objectif du tag, validation de ce fichier
  en boucle, numéro de release obtenu de Guy (Q1).
- **FR**
  1. `FR-38-0-1` — Compter les issues ouvertes et les bugs, et écrire au fichier d'epic le compte du tag précédent et
     l'objectif du tag.
  2. `FR-38-0-2` — Valider ce fichier en boucle jusqu'à une passe sans finding au-dessus de LOW.
  3. `FR-38-0-3` — Obtenir de Guy le numéro de la release qui porte la migration breaking (Q1), avant la fiche 38-3.
- **NFR**
  1. `NFR-38-0-1` — Décomptes recomptés à la source, commande et périmètre écrits avec le nombre.

### 38-1 — Coffre des secrets et connexions de prestataire

- *Intention* : un Administrateur crée une connexion (PayPal, libellé, date de départ, réglage quotidien ou à la
  demande, réglages du prestataire) et y dépose ses secrets (secret de l'application REST), chiffrés, jamais réaffichés.
- *Périmètre* : migration (deux tables, E1, E2, colonne `settings JSON` de E24), `kesh-api::config`
  (`KESH_SECRETS_KEY`), module `secrets` (C-38-1), structs de réglages `kesh-core::providers::settings` (E24), routes
  C-38-2, effacement des secrets à la restauration, audit, tables à `backup.rs` (applicative à `TABLES_TO_TRUNCATE`,
  système à la liste `_kesh_*`), liste close R1 (E18). Dépendance neuve : `aes-gcm` (à déclarer avec sa licence).
- *Risques* : clé perdue = secrets illisibles (ressaisie, dit au manuel) ; aucun secret dans les journaux, erreurs,
  réponses, audit ni sauvegarde (tests de recherche de la valeur) ; remise à zéro de la société (connexion vidée,
  secrets effacés) ; `KESH_SECRETS_KEY` à écrire au manuel d'administration, aux exemples de `.env` et à
  `docker-compose` ; un secret glissé dans `settings` (la struct n'a aucun champ secret ; `deny_unknown_fields`).
- **FR**
  1. `FR-38-1-1` — Un Administrateur crée, lit, modifie et supprime une connexion : prestataire, libellé, date
     « télécharger depuis », réglage `daily`|`manual`, réglages du prestataire (`settings`).
  2. `FR-38-1-2` — Il dépose ou remplace les secrets de la connexion ; la réponse dit seulement `secretSet` et
     `secretUpdatedAt`.
  3. `FR-38-1-3` — Les réglages sont validés par la struct du prestataire avant écriture (`PROVIDER_SETTINGS_INVALID`) ;
     un prestataire inconnu se liste sans être utilisable (`PROVIDER_UNKNOWN`).
  4. `FR-38-1-4` — La suppression d'une connexion est refusée tant qu'un compte prestataire y est rattaché
     (`PROVIDER_CONNECTION_IN_USE`).
  5. `FR-38-1-5` — Une restauration complète efface les secrets ; la connexion passe « secret à saisir ».
- **NFR**
  1. `NFR-38-1-1` — Secrets chiffrés AES-256-GCM (nonce aléatoire, AAD, version de clé) par `KESH_SECRETS_KEY` ;
     variable absente : Kesh démarre, connexions inutilisables (`PROVIDER_SECRETS_KEY_MISSING`, 409) ; gabarit refusé.
  2. `NFR-38-1-2` — Aucun secret dans les journaux, erreurs, réponses, audit, exports ni sauvegarde (test qui cherche
     la valeur partout).
  3. `NFR-38-1-3` — Routes des connexions et des secrets réservées à une session d'Administrateur (`require_not_pat`).
  4. `NFR-38-1-4` — Migration non breaking ; `settings` sous `CHECK (JSON_VALID … JSON_TYPE = 'OBJECT')` ; tables à
     `backup.rs` et à la liste close R1.

### 38-2 — Compte bancaire prestataire : compte `bank` + réglages (D8, B16)

- *Intention* : chaque solde de devise d'une connexion devient un compte bancaire Kesh ordinaire, lié à un compte du
  plan déjà en devise (numéro, historique et soldes gardés), avec ses réglages de prestataire : connexion, devise,
  compte de frais de la devise.
- *Périmètre* : migration **non breaking** (`CREATE TABLE payment_provider_accounts`, E11 ; comptes liés en colonnes
  FK, B23) ; routes C-38-3 ; levée de `ACCOUNT_IN_FOREIGN_CURRENCY` (C-37-9) pour un compte porteur de réglages, posés
  dans la même transaction que le lien (création et `PATCH` du lien du compte bancaire) ; gardes de devise et de
  compte de frais ; garde d'archivage d'un compte désigné ; exclusion du compte principal, des lots de paiement et de
  l'assistant ; exports (colonnes de la table neuve) ; tables à `backup.rs` (ordre enfants → parents) et liste close
  R1 (E18). `BankAccountKind` **inchangé**.
- *Risques* : **chaque site qui doit écarter un compte prestataire** lit désormais la table neuve (inventaire au
  `grep -rn "is_primary\|bank_accounts" crates/kesh-api/src/routes` à la fiche) ; retrait des réglages d'un compte lié
  en devise (refusé, sinon compte ordinaire lié à un compte en devise, contraire à C-37-9) ; verrou de devise du compte
  du plan (37-1) contre lien concurrent ; un compte bancaire archivé ou clos (28-4a) porteur de réglages ; montée sans
  perte depuis l'image publiée de l'Epic 37.
- **FR**
  1. `FR-38-2-1` — Pour un solde de devise, créer un compte bancaire `bank` sans IBAN avec ses réglages, ou poser les
     réglages sur un compte bancaire existant.
  2. `FR-38-2-2` — Lier ce compte à un compte du plan **déjà en devise**, en gardant numéro, historique et soldes (D8,
     levée de C-37-9 pour ce seul compte).
  3. `FR-38-2-3` — Désigner le compte de frais de la devise (charge, actif, même devise) ; le regroupement est son
     compte parent.
  4. `FR-38-2-4` — Une seule devise par connexion (`PROVIDER_ACCOUNT_DUPLICATE_CURRENCY`) ; retrait des réglages
     refusé si le compte est utilisé (`PROVIDER_ACCOUNT_IN_USE`).
  5. `FR-38-2-5` — Un compte prestataire n'est jamais principal et n'entre ni aux lots pain.001 ni à l'assistant.
- **NFR**
  1. `NFR-38-2-1` — Gardes au serveur, sous le verrou de devise du compte du plan, dans la transaction du lien.
  2. `NFR-38-2-2` — Comptes désignés en colonnes FK `ON DELETE RESTRICT`, archivage refusé tant qu'un réglage les
     désigne (B23).
  3. `NFR-38-2-3` — Migration non breaking ; `account_kind` toujours `'bank'`.

### 38-3 — Téléchargement : cadre commun, et la migration breaking

- *Intention* : Kesh télécharge les mouvements d'une connexion, chaque jour ou à la demande, sans doublon, depuis la
  date de départ, et liste les soldes sans compte.
- *Périmètre* : crate `kesh-providers` (types, trait `Connector` objet-safe, validation), colonnes de
  `bank_transactions` (C-38-4) et `bank_imports.provider_fetch_run_id`, tables `provider_fetch_runs`,
  `provider_balances_seen`, `provider_balance_snapshots`, passage (C-38-5, E5, E6, E10, E12, E20 (a)), en-tête
  `PROVIDER_API` affiché « Téléchargé de … » et non supprimable, route `POST /provider-connections/{id}/fetch`,
  transport factice en `KESH_TEST_MODE` + route `_test`, connecteur factice pour les tests. **Migration breaking**
  (E19) : `match_type` → `VARCHAR(40)`, relèvement de `kesh_version_min_required` en dernière instruction, dix crates
  bumpés dans le même commit, `EXEMPT_MIGRATIONS` ; `BankImportSourceFormat::ProviderApi` et `Other`,
  `ReconciliationMatchType` tolérant et ses quatre variantes neuves déclarées (évaluées en 38-4).
- *Risques* : statut d'une transaction qui change après coup (en attente → réussie, inversée) ; passage concurrent
  (verrou) ; reprise après échec au milieu d'une fenêtre ; réconciliation d'une transaction non encore classée ;
  volume (trois ans × trois soldes) ; heure locale contre UTC des dates du prestataire (date comptable = date en
  Europe/Zurich, à écrire) ; **chaque affichage de `source_format`** (inventaire) ; perte de `Copy` sur les deux
  enums (sites à suivre) ; utilisateur de la connexion désactivé (auteur du passage quotidien) ; ordre de restauration
  (`provider_fetch_runs` avant `bank_imports`) ; numéro de release non fixé (Q1, bloquant pour la fiche).
- **FR**
  1. `FR-38-3-1` — Télécharger les mouvements d'une connexion depuis `fetch_from`, chaque jour (`daily`) ou à la
     demande (`POST …/fetch`, Comptable ou Administrateur), dans les deux réglages.
  2. `FR-38-3-2` — Sans doublon : `provider_tx_id` unique par compte, insertion idempotente, recouvrement de trois
     jours.
  3. `FR-38-3-3` — Chaque passage est enregistré ; chaque compte touché reçoit un en-tête `bank_imports` en
     `PROVIDER_API`, affiché « Téléchargé de … », dont la suppression est refusée (`PROVIDER_IMPORT_NOT_DELETABLE`).
  4. `FR-38-3-4` — Un solde de devise sans compte Kesh est listé et proposé (B2) ; ses transactions ne se
     téléchargent pas.
  5. `FR-38-3-5` — Les soldes rapportés s'enregistrent ; un téléchargement incomplet est signalé avec l'écart (E20 a).
  6. `FR-38-3-6` — Une source ou un type de règle inconnus se lisent sans erreur (E4).
- **NFR**
  1. `NFR-38-3-1` — La seule migration breaking de l'epic : `UPDATE _kesh_version … kesh_version_min_required` en
     dernière instruction, crates bumpés dans le même commit, gate runtime complet avant `done` (P2, P2-bis).
  2. `NFR-38-3-2` — Retour à la version précédente par la seule sauvegarde d'avant la mise à jour ; recette de
     montée depuis l'image publiée précédente.
  3. `NFR-38-3-3` — Un passage à la fois par connexion ; un échec garde l'inséré et dit où reprendre.
  4. `NFR-38-3-4` — Aucun appel réseau en CI ; transport factice piloté par une route `_test`, inactif hors
     `KESH_TEST_MODE`.

### 38-4 — Critères neufs des règles d'affectation

- *Intention* : une règle reconnaît une transaction de prestataire par courriel, **boutique (préfixe du numéro de
  facture, B24)**, nom d'article ou code d'événement, et se restreint par sens, devise et compte — pour proposer le bon
  sous-compte de ventes de la devise et de la boutique.
- *Périmètre* : E14 ; migration non breaking (trois colonnes de filtre, `filter_key`, index unique remplacé) ;
  `rule_matches` étendu (`rules.rs:46`) et ses appelants (`first_matching_rule`, revalidation d'`accept_one_rule`) ;
  garde de devise du compte de la règle ; routes `reconciliation_rules` (types neufs, filtres facultatifs,
  `RULE_CRITERION_UNKNOWN`) ; export CSV des règles ; libellés des types aux quatre locales.
- *Risques* : les règles existantes ne changent pas de comportement (tests de non-régression des quatre critères) ;
  normalisation du courriel et du préfixe (casse, espaces) ; priorité entre règles larges et étroites (inchangée :
  `priority`) ; `match_type` non modifiable aujourd'hui (`reconciliation_rules.rs:121`) — à garder ou ouvrir à la
  fiche ; un préfixe qui en contient un autre (`LG` et `LGT` : la priorité tranche, écrit au manuel).
- *Boutique / domaine (B24, Q10 close)* : chez PayPal, la boutique se reconnaît au **préfixe d'`invoice_id`**, que le
  connecteur pose dans `invoice_ref` (38-8). Une règle `invoice_prefix` par boutique et par devise.
- **FR**
  1. `FR-38-4-1` — Quatre types neufs de critère : `counterparty_email_exact`, `invoice_prefix` (boutique),
     `item_name_contains`, `event_code_prefix`.
  2. `FR-38-4-2` — Trois filtres facultatifs, applicables à tout critère : sens, devise, compte bancaire.
  3. `FR-38-4-3` — Une règle par boutique et par devise est admise ; deux règles de mêmes type, valeur et filtres sont
     refusées (`RECONCILIATION_RULE_DUPLICATE`).
  4. `FR-38-4-4` — Un compte de règle en devise ne s'applique qu'aux transactions de cette devise.
  5. `FR-38-4-5` — Une règle d'un type inconnu ne s'applique jamais et ne se modifie pas (`RULE_CRITERION_UNKNOWN`).
- **NFR**
  1. `NFR-38-4-1` — Non-régression des quatre critères existants et de leur priorité.
  2. `NFR-38-4-2` — Normalisation unique (une fonction) pour l'écriture et l'évaluation.
  3. `NFR-38-4-3` — Migration non breaking.

### 38-5 — Code TVA par défaut d'un compte

- *Intention* : un compte de produit ou de charge porte son genre et son taux de TVA par défaut, que les propositions
  appliquent (B6) et que la saisie manuelle pourra reprendre.
- *Périmètre* : E13 ; colonnes `accounts`, champ du formulaire de compte, fonctions pures et miroir TS, gardes (taux
  actif à la date, compte de résultat seulement) ; **compte d'impôt sur les acquisitions** aux réglages TVA (B18 :
  `company_invoice_settings.default_vat_acquisition_account_id`, passif, actif, en CHF ; champ de l'écran des
  réglages).
- *Risques* : arrondi de l'extraction (centime) ; taux changé en cours d'exercice (taux en vigueur à la date de la
  transaction, par `valid_from`/`valid_to`) ; un compte en devise : TVA en devise principale (S5).
- **FR**
  1. `FR-38-5-1` — Un compte de produit ou de charge porte un genre de TVA par défaut (`sale`, `purchase`,
     `acquisition`, aucun) et un taux.
  2. `FR-38-5-2` — Kesh extrait la TVA d'un brut et calcule l'impôt sur les acquisitions (deux lignes égales).
  3. `FR-38-5-3` — Les réglages TVA désignent le compte d'impôt sur les acquisitions (B18).
  4. `FR-38-5-4` — Le taux appliqué est celui en vigueur à la date de la transaction.
- **NFR**
  1. `NFR-38-5-1` — Arrondi au centime ; fonctions pures en Rust et miroir TS tenus par une fixture partagée.
  2. `NFR-38-5-2` — Lignes de TVA en devise principale pour un compte en devise.
  3. `NFR-38-5-3` — Migration non breaking.

### 38-6a — Propositions de prestataire : ventes, achats, frais

- *Intention* : chaque vente ou achat téléchargé reçoit une proposition complète, acceptée en lot.
- *Périmètre* : E15 ; constructeur pur multi-lignes dans `kesh-reconciliation` (à côté de `manual.rs:68`, qui reste
  à deux lignes) ; branchement dans `get_proposals` (`reconciliation.rs:999`, saut hors CHF levé pour les comptes
  prestataires, `:1098`) et `post_accept` (`:1593`), variante `provider` ; frais seuls (`movement_kind = fee`) au
  compte de frais de la devise ; `alreadyBooked` et `confirmAlreadyBooked` (E15, B17).
- *Risques* : **`reconciliation.rs` fait 7 439 lignes** : le code neuf va dans un module à part, appelé depuis le
  répartiteur ; équilibre en devise principale à plusieurs lignes en devise (arrondis, 37-3a) ; exercice mono-devise ;
  compte de la règle archivé entre proposition et acceptation (revalidation sous verrou, précédent `accept_one_rule`).
- **FR**
  1. `FR-38-6a-1` — Une vente ou un achat reçoit une proposition : compte prestataire au net, contrepartie de la règle
     au brut hors TVA, TVA du compte (E13), frais au compte de frais de la devise.
  2. `FR-38-6a-2` — Un mouvement de frais seul se propose au compte de frais de la devise.
  3. `FR-38-6a-3` — Les propositions s'acceptent en lot (`POST /reconciliation/accept`, variante `provider`).
  4. `FR-38-6a-4` — Une transaction déjà comptabilisée porte `alreadyBooked` et n'est acceptée qu'avec
     `confirmAlreadyBooked: true`.
  5. `FR-38-6a-5` — Le cours est le cours AFC retenu du mois, A32 sinon ; cours manquant → `RATE_UNAVAILABLE`.
- **NFR**
  1. `NFR-38-6a-1` — *Pattern batch* : toute erreur par proposition en `failed[]`, codes constants ; aucune écriture
     d'office.
  2. `NFR-38-6a-2` — Revalidation sous verrou à l'acceptation.
  3. `NFR-38-6a-3` — Code neuf hors de `reconciliation.rs`, constructeur pur testé sans base.

### 38-6b — Conversions, retraits, remboursements, litiges ; file « à examiner »

- *Intention* : les mouvements qui ne sont ni vente ni achat se proposent juste, ou attendent un humain.
- *Périmètre* : E16 ; paire de conversion vers `POST /journal-entries/conversions` (A31) ; rattachement des retraits
  (34-11b) ; inverse d'une écriture d'origine ; file (C-38-9).
- *Risques* : paire de conversion dont une moitié manque (passage interrompu) ; remboursement partiel (montants
  rapportés, TVA extraite du brut remboursé, B20) ; frais rendus en partie ou pas du tout ; litige résolu en faveur du
  vendeur (deux mouvements qui s'annulent) ; annulation d'une réconciliation de paire (les deux côtés).
- **FR**
  1. `FR-38-6b-1` — Une paire de conversion se propose comme une conversion de 37-3b, `completeDifference: true`.
  2. `FR-38-6b-2` — Un retrait vers la banque ou un dépôt depuis elle se rattache des deux côtés comme un virement
     entre ses comptes (34-11b).
  3. `FR-38-6b-3` — Un remboursement se propose comme l'inverse de l'écriture d'origine, aux montants rapportés.
  4. `FR-38-6b-4` — Litige, rétrofacturation, mouvement inconnu ou origine absente vont à la file « à examiner », sans
     proposition ; « examiné » se marque (`review-done`).
- **NFR**
  1. `NFR-38-6b-1` — Les deux moitiés d'une conversion se réconcilient et s'annulent ensemble.
  2. `NFR-38-6b-2` — Aucune écriture d'office ; *Pattern batch*.

### 38-6c — Rattachement aux écritures existantes ; API des scripts externes (B17, B21)

- *Intention* : les transactions téléchargées de 2024, déjà passées par l'API, se rattachent **en lot** à leurs
  écritures, sans doublon ; un script externe (WHMCS, CB Paid Subscriptions) retrouve une transaction de prestataire,
  crée l'écriture et la rattache, par une clé d'API.
- *Périmètre* : E21, E22, C-38-14 ; rattachement **en devise** (`transaction_can_be_linked_to_entry`,
  `reconciliation_link.rs:535`, et les textes SQL du dialogue et de `count_already_booked_for_page`, une seule règle,
  C-v017-3-7) ; suggestions univoques et lot `link-entries` (*Pattern batch*) ; recherche `GET
  /bank-accounts/{id}/transactions` ; clés `read-write` sur `link-entries`. Écran : 38-9b.
- *Risques* : **l'égalité « lecture par page = fonctions du dialogue »** (test `s34_4_ac8_…`) tenue en devise ; une
  écriture qui touche deux comptes prestataires (conversion, retrait) ; écriture de 2024 passée au brut avec frais
  séparés (le mouvement sur le compte prestataire reste le net : à vérifier sur les écritures de la recette 37) ;
  exercice 2024 clos ou livres verrouillés (le rattachement n'écrit pas d'écriture : admis, comme v017-3a — à
  confirmer à la fiche) ; performance sur une année (≈ milliers de transactions × écritures du compte).
- **FR**
  1. `FR-38-6c-1` — Une transaction en devise se rattache à une écriture existante sur le montant en devise.
  2. `FR-38-6c-2` — Kesh suggère, par transaction en attente, l'écriture univoque ou la liste ambiguë.
  3. `FR-38-6c-3` — Un lot de rattachements s'envoie en une requête (`POST /reconciliation/link-entries`).
  4. `FR-38-6c-4` — Une transaction de prestataire se cherche par identifiant du prestataire, numéro de facture,
     statut et période.
  5. `FR-38-6c-5` — Un script externe crée l'écriture et la rattache par une clé d'API `read-write`.
- **NFR**
  1. `NFR-38-6c-1` — Une seule règle de rattachement : lecture par page égale aux fonctions du dialogue, en devise.
  2. `NFR-38-6c-2` — *Pattern batch* ; rien de rattaché d'office.
  3. `NFR-38-6c-3` — Une année de transactions se suggère en temps borné (mesure à la fiche).
  4. `NFR-38-6c-4` — Routes des connexions et des secrets toujours fermées aux clés d'API.

### 38-7 — Client PayPal

- *Intention* : lire, pour une connexion PayPal, les soldes et les transactions qui changent le solde.
- *Périmètre* : OAuth2 `client_credentials` (`clientId` des réglages, secret du coffre ; jeton gardé en mémoire
  jusqu'à expiration), `GET /v1/reporting/transactions` (`start_date`/`end_date` ≤ 31 jours, `fields=all`,
  `page_size=500`, `page`, `balance_affecting_records_only=Y`), `GET /v1/reporting/balances` ; plafond de 10 000
  résultats → la fenêtre se redécoupe ; trois ans d'historique → `fetch_from` antérieur refusé à l'écran et borné au
  passage ; « Tester la connexion » pose `accountEmail` et `payerId` aux réglages (E24) ; erreurs (401, 403 permission
  *Transaction Search* manquante, 429, 5xx) en états lisibles ; URL de base constante (*live*), autre base seulement
  en test.
- *Risques* : permission manquante sur l'application REST (message clair) ; pagination incohérente ; délai de 3 h ;
  fuseau des dates PayPal.
- **FR**
  1. `FR-38-7-1` — Obtenir un jeton OAuth2 avec `clientId` et le secret de la connexion.
  2. `FR-38-7-2` — Lire les transactions par fenêtres de 31 jours au plus, toutes les pages, en redécoupant au-delà
     de 10 000 résultats.
  3. `FR-38-7-3` — Lire les soldes par devise, à une date donnée (`as_of_time`).
  4. `FR-38-7-4` — « Tester la connexion » lit les soldes et pose `accountEmail` et `payerId` aux réglages.
  5. `FR-38-7-5` — Chaque erreur PayPal devient un état lisible de la connexion.
- **NFR**
  1. `NFR-38-7-1` — URL de base constante, client HTTP borné de 37-2 (délai, taille).
  2. `NFR-38-7-2` — Jeton en mémoire seulement, jamais écrit.
  3. `NFR-38-7-3` — Validation stricte des réponses ; tests sur fixtures anonymisées et réponses hostiles.

### 38-8 — Codes d'événement PayPal

- *Intention* : chaque transaction PayPal devient un `ProviderMovement` classé, brut, frais, net, devise, liens.
- *Périmètre* : table de correspondance par famille de codes `T00xx` (paiements : vente ou achat selon le signe),
  `T01xx` (frais), `T02xx` (conversion), `T03xx`/`T04xx` (dépôt, retrait banque), `T11xx` (inversions,
  remboursements), `T12xx` (ajustements, rétrofacturations → litige), `T15xx`/`T21xx` (blocages, réserves : ignorés
  s'ils ne changent pas le solde), le reste → `other` (file). **À valider sur les réponses enregistrées de Beat** :
  chaque code relevé reçoit un test. **Boutique (B24)** : `transaction_info.invoice_id` → `invoice_ref` (et
  `reference`), sans transformation ; la règle `invoice_prefix` en lit le début (38-4). Surcharges facultatives
  `eventKindOverrides` aux réglages (E24), à trancher à la fiche.
- *Risques* : un code inconnu ne se classe jamais en vente (→ `other`) ; signe des frais ; regroupement des deux
  moitiés d'une conversion ; `invoice_id` absent (vente sans boutique : aucune règle `invoice_prefix` ne s'applique).
- **FR**
  1. `FR-38-8-1` — Chaque code `T…` se classe en genre de mouvement selon la table ; un code inconnu → `other`.
  2. `FR-38-8-2` — Brut, frais, net, devise, payeur, articles et liens (`related_provider_tx_id`) se lisent de la
     réponse.
  3. `FR-38-8-3` — `invoice_id` se pose dans `invoice_ref` et `reference` (critère boutique, B24).
  4. `FR-38-8-4` — Un réglage de connexion peut classer un code non couvert (si retenu à la fiche).
- **NFR**
  1. `NFR-38-8-1` — Chaque code relevé sur les réponses de Beat a son test ; aucun code inconnu ne devient une vente.
  2. `NFR-38-8-2` — Classement pur, sans base ni réseau.

### 38-9a et 38-9b — Écrans

- *38-9a* : liste et fiche des connexions (réglages, secrets en saisie seule, « Tester », « Télécharger maintenant »,
  état du dernier passage), soldes proposés (créer ou lier un compte, B2), compte bancaire prestataire et compte de
  frais de la devise, contrôles de solde (E20 : téléchargement incomplet, écart comptable) ; manuel d'administration
  (clé, secrets à ressaisir après restauration, permissions PayPal, recette, **sauvegarde d'avant la mise à jour
  obligatoire et seul retour**, E19).
- *38-9b* : types neufs et filtres au formulaire des règles (boutique = préfixe du numéro de facture) ; propositions
  multi-lignes (aperçu des lignes, TVA, frais, cours, « déjà comptabilisée ») ; acceptation en lot ; **rattachement en
  lot** (suggestions univoques à cocher, ambiguës à trancher une à une, E21) ; file « à examiner » ; manuel utilisateur
  (et la voie API des scripts, E22) ; E2E (contre le transport factice) et captures d'écran (*Recette visuelle*).
- *Risques* : un secret ne revient jamais dans le DOM ; textes sans « automatique » trompeur (proposer, non
  comptabiliser).
- **FR (38-9a)**
  1. `FR-38-9a-1` — Écrans de liste et de fiche des connexions, réglages compris ; secrets en saisie seule.
  2. `FR-38-9a-2` — Boutons « Tester la connexion » et « Télécharger maintenant », état du dernier passage.
  3. `FR-38-9a-3` — Soldes sans compte listés, avec « créer » ou « lier » un compte (B2).
  4. `FR-38-9a-4` — Réglages d'un compte prestataire et compte de frais de la devise.
  5. `FR-38-9a-5` — Contrôles de solde affichés : téléchargement incomplet, écart comptable.
- **NFR (38-9a)**
  1. `NFR-38-9a-1` — Aucun secret dans le DOM ni les réponses lues par l'écran.
  2. `NFR-38-9a-2` — Textes aux quatre locales ; manuel d'administration et PDF à jour (sauvegarde d'avant la mise à
     jour, `KESH_SECRETS_KEY`).
  3. `NFR-38-9a-3` — Captures d'écran de chaque état touché.
- **FR (38-9b)**
  1. `FR-38-9b-1` — Formulaire des règles : types neufs (dont boutique) et filtres sens, devise, compte.
  2. `FR-38-9b-2` — Propositions multi-lignes lisibles (lignes, TVA, frais, cours, « déjà comptabilisée »), acceptées
     en lot.
  3. `FR-38-9b-3` — Rattachement en lot : univoques à cocher, ambiguës une à une.
  4. `FR-38-9b-4` — File « à examiner » et action « examiné ».
- **NFR (38-9b)**
  1. `NFR-38-9b-1` — E2E contre le transport factice (téléchargement → règle → proposition → acceptation, et
     rattachement en lot).
  2. `NFR-38-9b-2` — Textes « proposer », jamais « comptabiliser automatiquement » ; quatre locales ; manuel
     utilisateur et PDF à jour.
  3. `NFR-38-9b-3` — Captures d'écran de chaque état touché.

### 38-10 — Relevé joint à la main

- *Intention* : le relevé annuel PayPal (PDF, sans API) se range au compte et à l'exercice, et son solde de clôture
  sert de **somme de contrôle** (B20).
- *Périmètre* : E17, C-38-10, E20 (b) ; écran du compte bancaire (onglet *Relevés*, écart au solde comptable en
  devise) ; contrôles de clôture de l'exercice (avertissement non bloquant) ; sauvegarde complète.
- *Risques* : fichier qui n'est pas un PDF ; taille ; suppression d'un exercice ou d'un compte (refus tant qu'une
  pièce y est rangée, ou suppression de la pièce : à trancher à la fiche).
- **FR**
  1. `FR-38-10-1` — Joindre un relevé PDF à un compte bancaire, pour un exercice ou une période.
  2. `FR-38-10-2` — Saisir le solde de clôture et la devise du relevé (facultatif).
  3. `FR-38-10-3` — Afficher l'écart entre ce solde et le solde comptable à sa date.
  4. `FR-38-10-4` — Ajouter cet écart aux contrôles de clôture de l'exercice (avertissement non bloquant).
- **NFR**
  1. `NFR-38-10-1` — PDF seul, taille bornée, fichier rangé par `document_storage`.
  2. `NFR-38-10-2` — Pièces emportées et restaurées par la sauvegarde complète.
  3. `NFR-38-10-3` — Migration non breaking.

## Exigences (FR/NFR) par story

Récapitulatif (B26) ; le texte de chaque exigence est dans la fiche. **Forme définitive — issues GitHub ou fiches —
à Guy** (Q12).

| Story | FR | NFR |
|---|---|---|
| 38-0 | 1 décomptes · 2 validation · 3 numéro de release | 1 décomptes recomptés |
| 38-1 | 1 connexion CRUD · 2 secrets en écriture seule · 3 réglages validés · 4 suppression refusée si utilisée · 5 secrets effacés à la restauration | 1 AES-256-GCM, clé d'environnement · 2 aucun secret nulle part · 3 session d'Administrateur seule · 4 migration non breaking, JSON contrôlé |
| 38-2 | 1 créer ou poser les réglages · 2 lien à un compte en devise (D8) · 3 compte de frais de la devise · 4 une devise par connexion, retrait refusé si utilisé · 5 jamais principal ni pain.001 | 1 gardes sous verrou · 2 FK `RESTRICT`, archivage gardé · 3 non breaking, `bank` |
| 38-3 | 1 téléchargement quotidien ou à la demande · 2 sans doublon · 3 passage et en-tête `PROVIDER_API` · 4 solde sans compte proposé · 5 complétude · 6 lecture tolérante | 1 **la** migration breaking (P2, P2-bis) · 2 retour par la sauvegarde, recette de montée · 3 un passage à la fois, reprise · 4 aucun réseau en CI |
| 38-4 | 1 quatre types neufs · 2 trois filtres · 3 une règle par boutique et devise · 4 compte en devise · 5 type inconnu inerte | 1 non-régression · 2 normalisation unique · 3 non breaking |
| 38-5 | 1 genre et taux par compte · 2 extraction, acquisitions · 3 compte d'acquisitions · 4 taux à la date | 1 centime, Rust = TS · 2 TVA en devise principale · 3 non breaking |
| 38-6a | 1 proposition vente/achat · 2 frais seuls · 3 lot `provider` · 4 déjà comptabilisée · 5 cours | 1 *Pattern batch* · 2 revalidation sous verrou · 3 module à part, pur |
| 38-6b | 1 conversion · 2 retrait/dépôt · 3 remboursement inverse · 4 file « à examiner » | 1 paire liée · 2 aucune écriture d'office |
| 38-6c | 1 rattachement en devise · 2 suggestions · 3 lot · 4 recherche · 5 script externe | 1 une seule règle · 2 *Pattern batch* · 3 temps borné · 4 connexions fermées aux clés |
| 38-7 | 1 OAuth2 · 2 transactions par fenêtres · 3 soldes · 4 « Tester » · 5 erreurs lisibles | 1 URL constante, client borné · 2 jeton en mémoire · 3 validation stricte |
| 38-8 | 1 classement des codes · 2 champs lus · 3 `invoice_id` → `invoice_ref` · 4 surcharges | 1 test par code relevé · 2 classement pur |
| 38-9a | 1 écrans des connexions · 2 Tester, Télécharger · 3 soldes proposés · 4 comptes prestataires · 5 contrôles de solde | 1 aucun secret au DOM · 2 i18n, manuel admin · 3 captures |
| 38-9b | 1 formulaire des règles · 2 propositions multi-lignes · 3 rattachement en lot · 4 file « à examiner » | 1 E2E · 2 textes, i18n, manuel utilisateur · 3 captures |
| 38-10 | 1 joindre un PDF · 2 solde de clôture · 3 écart affiché · 4 contrôles de clôture | 1 PDF borné · 2 sauvegarde · 3 non breaking |

**Total : 64 FR et 41 NFR** sur 14 stories (38-9a et 38-9b comptées à part), recomptés au `grep` des listes
numérotées des fiches.

## Ce que réutiliseront les Epics 39 (Wise) et 40 (Stripe)

**Réutilisé tel quel** : coffre des secrets et connexions (38-1), réglages de compte prestataire et compte de frais
(38-2), cadre de téléchargement, passages, doublon, planification, soldes vus et contrôles (38-3), règles étendues
(38-4), TVA par défaut (38-5), propositions et file (38-6a, 38-6b), rattachement en lot et API des scripts (38-6c),
écrans génériques (38-9a, 38-9b), pièces de compte (38-10). **Aucune migration breaking pour 39 et 40** : l'en-tête
`PROVIDER_API` sert tel quel ; les valeurs neuves (`provider`, genres de mouvement, critères de règle éventuels) vont
dans des colonnes lues avec tolérance (E4) — `match_type` en `VARCHAR` sans `CHECK` compris ; les options propres au
prestataire vont dans `settings` (E24 : struct `WiseSettings`, `StripeSettings`) ; un compte de plus (p. ex. le compte
intermédiaire de Stripe) est une colonne FK `NULL` de `payment_provider_accounts` (B23).

**Ajouté par 39 (Wise)** : connecteur `kesh-providers::wise` (jeton personnel, profils, soldes, relevés
`balance-statements`) ; **authentification forte** (clé privée gardée au coffre, clé publique déposée chez Wise,
renouvellement environ tous les 90 jours) ; **PDF de relevé téléchargé par l'API** et rangé par 38-10 ; achats par
carte Wise (classement).

**Ajouté par 40 (Stripe)** : connecteur `kesh-providers::stripe` (clé restreinte en lecture, *balance transactions*,
*payouts*) ; le **compte intermédiaire** « Débiteurs Stripe » par devise (B13) : vente au compte intermédiaire,
versement rattaché au relevé bancaire comme un virement entre ses comptes (34-11b) ; créance à la clôture.

## Point TVA pour Guy (B6) — à lire

Beat **est assujetti** à la TVA ; sa société sépare la TVA de ses ventes PayPal et Stripe et paie l'impôt sur les
acquisitions de ses achats de services à l'étranger. Le report des **dix P1 du jalon « Vague 2 »** reposait sur ceci
(`CLAUDE.md`, *Priorités des défauts*) : « le seul utilisateur réel de Kesh n'est pas assujetti […] ce report tombe dès
que Kesh tient, ou est proposé pour tenir, les livres d'un assujetti ». **Cet epic le propose.** Deux constats au code
s'y ajoutent :

- le **rapport TVA** tire la TVA due des **seules factures** (`kesh-report/src/vat_report.rs:1-25`) : la TVA due
  séparée par ces propositions, hors facture, **n'y figure pas** ; l'impôt sur les acquisitions non plus ;
- Kesh n'a **aucun code TVA** (chiffres du décompte) : `vat_rates` ne porte qu'un libellé et un taux.

L'impôt sur les acquisitions va à un **compte distinct** (B18, 38-5 : `default_vat_acquisition_account_id`, p. ex.
2201), ce qui le rend lisible au grand livre en attendant un rapport TVA qui le reprenne (C-38-13).

Beat ne dépose pas de décompte par Kesh cette année et passe la TVA par l'API et ces imports seulement (B6, B15) :
l'epic **sépare** la TVA dans les écritures, il **ne la déclare pas**. La décision de lever ou de maintenir le report
des P1 TVA revient à Guy.

## Révision B16–B21 (2026-11-15)

Sur les décisions de Beat B16–B21 et celles de l'orchestrateur (ex-Q4, Q5, Q7, Q8) :

*(Historique : les points « aucune migration breaking », en-tête `MANUAL`, table enfant à clé inerte et `shop_ref`
sont remplacés par la révision B22–B26 ci-dessous.)*

- **Plus aucune migration breaking** (B16, E19) : `account_kind`, `match_type` et `source_format` ne reçoivent aucune
  valeur neuve ; `kesh_version_min_required` inchangé ; preuve par l'inventaire des lecteurs (E19). Six migrations,
  toutes non breaking. Le SQL de retour s'écrit au manuel (38-9a).
- **Compte prestataire** (E11, 38-2) : compte `bank` + table `payment_provider_accounts` (connexion, devise, compte de
  frais de la devise) ; la levée de C-37-9 suit la ligne de réglages, non un type de compte.
- **Passages** (E5, 38-3) : table `provider_fetch_runs` ; l'en-tête `bank_imports` reste obligatoire
  (`import_id NOT NULL`, relevé au code), en `MANUAL` — valeur inexacte assumée (aucune ne l'est), jamais affichée
  comme telle, suppression refusée (Q11 pour Guy).
- **Règles** (E14, 38-4) : table enfant `reconciliation_rule_conditions` ; ligne mère `reference_contains` à clé
  inerte ; un critère principal + filtres sens, devise, compte ; critère neuf **boutique / domaine** sur `shop_ref`,
  champ PayPal **incertain** (38-8, Q10).
- **2024** (B17, E21) : story neuve **38-6c** — rattachement en devise et en lot aux écritures existantes, sur le
  signal 34-4 ; les propositions de prestataire exigent `confirmAlreadyBooked` (E15).
- **API des scripts** (B21, E22) : écritures et `link-entry` déjà ouverts aux clés `read-write` ; manquaient la
  recherche d'une transaction de prestataire, le rattachement en devise et le lot — dans 38-6c.
- **TVA** (B18, E13, 38-5) : compte distinct d'impôt sur les acquisitions aux réglages TVA.
- **Ventes** (B19, E14) : une règle par boutique et par devise vers le sous-compte de ventes de la devise ;
  analytique non tranchée (Q3 réduite).
- **Remboursements et frais** (B20, E16) : tels que rapportés ; **relevé = somme de contrôle** (E20) : complétude du
  téléchargement, écart comptable, solde de clôture du relevé joint aux contrôles de clôture (38-10).
- **Décisions de l'orchestrateur** : secrets ressaisis après toute restauration (E2) ; rotation de la clé hors
  périmètre, issue (C-38-13) ; aucun outil MCP neuf, `kesh_ecritures` suffit (E23).
- **Stories** : 38-6c ajoutée (14 stories, 38-0 compris) ; 38-2 non breaking ; 38-4 ne dépend plus de 38-2 que par
  38-3 ; 38-10 dépend de 38-3 (soldes rapportés).

## Révision B22–B26 (2026-11-15)

Sur les décisions de Beat B22–B26 (`epics-38-40-cadrage.md`, *Décisions de Beat — suite, après la révision
B16–B21*) :

- **Une migration breaking, dans 38-3** (B22, E19, C-38-0) : `source_format = 'PROVIDER_API'` (sans DDL) et
  `reconciliation_rules.match_type` d'`ENUM` à `VARCHAR(40)` ; `UPDATE _kesh_version … kesh_version_min_required` en
  dernière instruction (P2), dix crates bumpés dans le même commit (P2-bis), exemption P7, ligne d'audit P5. Placée en
  38-3, première story qui écrit une valeur neuve, pour un seul relèvement ; fichier à part de la migration non
  breaking de 38-3. Retour à la version précédente par la seule sauvegarde d'avant la mise à jour ; **le SQL de retour
  est retiré**. Numéro de release à Guy (**Q1 rouverte**).
- **Décodage tolérant** (E4) : `BankImportSourceFormat` et `ReconciliationMatchType` gagnent `Other(String)` — Wise
  et Stripe n'auront plus à relever la version minimale pour ces colonnes.
- **Détours retirés** : en-tête `MANUAL` (E5), table enfant `reconciliation_rule_conditions` et clé inerte
  `reference_contains` (E14), `RULE_CRITERIA_CONFLICT`. Le critère neuf **est** un `match_type`.
- **Critères recensés** (E14) : `match_type` neufs `counterparty_email_exact`, `invoice_prefix` (boutique),
  `item_name_contains`, `event_code_prefix` ; **filtres** en colonnes neuves `direction`, `currency`,
  `bank_account_id` (migration non breaking de 38-4), unicité élargie par `filter_key` — une règle par boutique et par
  devise est admise (la limite d'unicité de C-38-13 tombe).
- **Réglages de connexion** (B23, E24) : `payment_provider_connections.settings JSON`, `CHECK (JSON_VALID …
  JSON_TYPE = 'OBJECT')`, lu en struct par prestataire (`deny_unknown_fields`, patron `ColumnMapping` /
  `parse_column_mapping`) ; PayPal : `clientId`, `accountEmail`, `payerId`, `eventKindOverrides` ; comptes liés en
  colonnes FK (E11) ; secrets au coffre (E2).
- **Boutique PayPal** (B24) : préfixe d'`invoice_id`, posé dans `invoice_ref` (38-8) ; `shop_ref` et `shop_source`
  retirés ; **Q10 close**.
- **Stripe = Epic 40** (B25), provisoire en attendant Guy ; Q3 reportée.
- **Exigences** (B26) : FR et NFR numérotées dans chaque fiche (38-0 ajoutée comme fiche) et récapitulées (§
  *Exigences (FR/NFR) par story*) : **64 FR, 41 NFR**. Forme définitive à Guy (Q12).
- **Stories** : sept fichiers de migration (une breaking) ; 38-3 dépend du numéro de release ; 38-1 touche `kesh-core`
  (structs de réglages) ; 38-8 ne touche plus `kesh-api`.

## Questions ouvertes

*Closes* : Q2 (rattachement en lot, B17, E21), Q3 sauf l'analytique (B18, B19), Q4 et Q5 (décidées, E2), Q6 (B20,
E16), Q7 (un critère principal + filtres, E14), Q8 (aucun outil MCP, E23), **Q10** (boutique = préfixe
d'`invoice_id`, B24), **Q11** (`PROVIDER_API` retenu, B22).

- **Q1 — Numéro de la release (Guy), rouverte par B22** : la migration breaking de 38-3 fige la version dans son SQL
  (`kesh_version_min_required`) et impose le bump des crates dans le même commit (P2-bis). Proposition : la mineure
  suivante (`0.18.0`), l'Epic 37 n'en relevant aucune. **Bloquant pour la fiche 38-3**, pas pour les autres.
- **Q3 — Catégorie de vente (reste de Q3, B19)** : sous-comptes de ventes par devise et par catégorie, ou catégorie
  portée par l'analytique (`default_project_id` de la règle) ? **Reportée** (B25) ; l'epic marche dans les deux cas
  (une règle par boutique et par devise).
- **Q9 — Numéro 40 pour Stripe** : retenu **provisoirement** (B25) ; confirmation de Guy attendue.
- **Q12 — Forme des exigences (B26, Guy)** : FR et NFR en issues GitHub ou dans les fiches ?

## Règles de cet epic

- Méthode du dépôt (`CLAUDE.md`) : fiche → validation en boucle → développement → revue en boucle (Sonnet ↔ Opus pour
  les passes complètes, passe ciblée en fin de boucle), gate local complet avant la PR ; splitting préventif (≤ 5
  modules par story).
- **Production** : l'instance de Beat n'est jamais écrite par le travail de cet epic. Recette sur ses comptes PayPal
  réels, avec une application REST en lecture seule (*Transaction Search*), sur une **instance jetable** (B12).
- Aucun appel réseau en CI ; aucune réponse réelle non anonymisée dans le dépôt.
- On trace toujours : un défaut voisin non traité se note au registre (C-38-13), pour devenir une issue avec la PR.
