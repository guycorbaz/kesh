# Epic 38 — Comptes PayPal : téléchargement et imputation automatiques

**Cadrage : 2026-11-15** (branche `epic/38-paypal`), **révisé le même jour sur B16–B21** (§ *Révision B16–B21*).
Demande de Beat (contributeur), décisions B1–B21 de `epics-38-40-cadrage.md`, qui **priment** sur ce fichier là où il
diverge. Kickoff (décomptes d'issues et de bugs, validation de ce fichier) **à tenir** : `38-0-kickoff` reste au
backlog tant que l'Epic 37 n'est pas développé.

**Aucune migration breaking** (B16) : toutes les migrations de l'epic créent des tables ou ajoutent des colonnes
`NULL` ; `kesh_version_min_required` **inchangé** ; aucune valeur neuve dans une colonne existante à domaine fermé
(`account_kind`, `match_type`, `source_format`) — preuve au code en E19.

**Numéro : 38** (35 et 36 réservés ailleurs ; 39 Wise, 40 Stripe — numéro 40 à confirmer par Guy). Une branche par
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

**Suite (B16–B21)** : B16 aucune migration breaking — compte `bank` + réglages de connexion en table neuve, critères
en table enfant, passages en table propre · B17 2024 déjà importé : rattachement **en lot** aux écritures existantes,
sans doublon · B18 compte distinct de TVA due sur acquisitions (p. ex. 2201) · B19 sous-comptes de ventes par devise ;
critère « boutique / domaine » ; analytique non tranchée · B20 remboursements et frais tels que rapportés ; relevés =
sommes de contrôle · B21 un script externe (WHMCS, CB Paid Subscriptions) crée des écritures ou les rattache par
l'API.

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
  passage, `created_by_user_id`). Chaque solde de devise est un **compte bancaire** Kesh ordinaire (`account_kind =
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
- **E4 — Les valeurs neuves vivent dans les colonnes neuves, lues avec tolérance (patron A30).** Aucune colonne
  existante ne reçoit de valeur neuve (B16, E19). Les colonnes **créées par cet epic** — `provider`, genre de mouvement,
  motif d'examen, critère de règle, sens, genre TVA d'un compte, état d'un passage — se lisent avec une variante
  `Other(String)` : une valeur ajoutée par les Epics 39 et 40 ne demande ni migration breaking ni relèvement de
  `kesh_version_min_required`. `BankAccountKind`, `ReconciliationMatchType` et `BankImportSourceFormat` **ne changent
  pas** (ni variante, ni décodage).
- **E5 — Passages dans leur table ; l'en-tête `bank_imports` reste, avec une valeur existante.** Table applicative
  `provider_fetch_runs` (connexion, déclencheur `daily`|`manual`, utilisateur, fenêtre, début, fin, état, erreur,
  comptes, nombres insérés ou mis à jour, point de reprise). **Relevé au code** : une transaction ne peut pas exister
  sans en-tête — `bank_transactions.import_id` est `NOT NULL`, FK `ON DELETE CASCADE` (`20260504000001_bank_imports.sql:58,
  79`) ; le rendre `NULL` serait un `MODIFY COLUMN` qu'un binaire antérieur ne décoderait plus (`import_id: i64`), donc
  breaking. Chaque passage crée donc, **par compte touché** (au moins une transaction insérée), une ligne `bank_imports`
  portant la colonne neuve **`provider_fetch_run_id`** (NULL, FK `provider_fetch_runs`), période = fenêtre, `file_hash`
  = SHA-256 des enregistrements normalisés, `statement_id` NULL, soldes NULL (le solde vit en E20), `filename` =
  `telechargement-<provider>-<du>-<au>` (vrai pour tout lecteur), `imported_by_user_id` = l'utilisateur du passage à la
  demande, le `created_by_user_id` de la connexion pour le passage quotidien (colonne `NOT NULL`, `:33`).
  **`source_format = 'MANUAL'`** — et ce n'est pas le format : **aucune valeur existante n'est exacte** pour un
  téléchargement. `MANUAL` est retenu parce qu'au code il désigne un relevé **sans fichier, dont Kesh construit les
  lignes** (nom et empreinte synthétiques, `bank-import-display.ts:4-8`, `bank_statements_manual.rs:370`), ce qui est
  vrai d'un passage ; `CSV` affirmerait un fichier et un profil qui n'existent pas. Son libellé « Saisie manuelle »
  serait faux : **le binaire de l'epic ne l'affiche jamais pour un en-tête qui porte `provider_fetch_run_id`** — liste
  et détail des imports, boîte de suppression, export CSV de `bank_imports` (colonne `provider_fetch_run_id` ajoutée,
  `csv_tables.rs:702-720`), audit — et dit « Téléchargé de PayPal (connexion, passage) ». Aucun comportement ne dépend
  de `source_format` (lecteurs relevés à l'inventaire : affichage seul). La suppression d'un tel en-tête par
  `DELETE /bank-imports/{id}` est **refusée** (`PROVIDER_IMPORT_NOT_DELETABLE`) : le recouvrement de trois jours ne
  retéléchargerait pas ce qu'elle effacerait. *Raison* : B16, DRY (la réconciliation lit `bank_transactions` et rien
  d'autre), et Wise/Stripe sans migration.
- **E6 — Doublon par l'identifiant du prestataire.** Colonne neuve `provider_tx_id`, `UNIQUE (bank_account_id,
  provider_tx_id)` (NULL multiples admis : CAMT, CSV et manuel inchangés). Insertion idempotente
  (`INSERT … ON DUPLICATE KEY` sans écrasement) ; chaque passage **recouvre** les trois derniers jours (délai de 3 h de
  PayPal, statuts tardifs). La clé de doublon par fenêtre (`kesh-core/src/bank_imports.rs:295`) ne s'applique pas.
- **E7 — Une transaction de prestataire = un mouvement du solde.** `amount` = **net** (ce qui change le solde),
  plus `gross_amount` et `fee_amount` ; frais **dans** la même transaction (B3 : brut par la règle, frais au
  sous-compte). Une conversion PayPal donne **deux** transactions (une par solde), reliées par
  `related_provider_tx_id`.
- **E8 — Colonnes génériques, pas de JSON brut.** `bank_transactions` gagne `movement_kind`, `event_code`,
  `counterparty_email`, `invoice_ref`, `item_names`, `shop_ref` (B19, E14), `related_provider_tx_id`, `review_reason`
  (toutes NULL ; les lectures du dépôt nomment leurs colonnes, `repositories/bank_transactions.rs:21` : un binaire
  antérieur les ignore). Le connecteur remplit aussi les colonnes existantes quand elles disent vrai :
  `counterparty_name` = nom du payeur ou du marchand, `reference` = numéro de facture du prestataire, `details` =
  objet de la transaction. *Raison* :
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
  currency)`. Gardes, au serveur : compte du plan lié **actif, de bilan (actif)**, **dans la devise du solde** — CHF
  admis pour le solde CHF ; `fee_account_id` de charge, actif, dans la même devise (`FEE_ACCOUNT_INVALID`) ; le
  regroupement des frais est son **compte parent** (`accounts.parent_id`), aucun réglage de plus ; jamais principal
  (`is_primary` refusé, la QR-facture lit le compte principal, `invoice_pdf_service.rs:332`). La ligne de réglages et
  le lien au compte du plan en devise se posent **dans la même transaction** ; c'est elle qui lève C-37-9 (D8). Retirer
  la ligne est refusé tant que le compte est lié à un compte du plan en devise, a des transactions de prestataire ou un
  solde vu (`PROVIDER_ACCOUNT_IN_USE`). Lier un compte bancaire **existant** garde son numéro, son historique et ses
  soldes. Le binaire de l'epic lit cette table partout où un compte prestataire doit être écarté : compte principal,
  lots de paiement pain.001 (déjà refusés sans IBAN depuis 34-11a1 : **à vérifier à la fiche**), assistant
  d'installation. Un binaire antérieur voit un compte bancaire ordinaire sans IBAN (B16).
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
- **E14 — Règles étendues (B5, B16, B19) : table enfant, `match_type` inchangé.** Table applicative neuve
  **`reconciliation_rule_conditions`** (`rule_id` PK et FK `ON DELETE CASCADE`, `criterion` NULL, `criterion_value`
  NULL, `direction` NULL `in`|`out`, `currency` NULL, `bank_account_id` NULL FK). **Un critère principal par règle**
  (décidé, ex-Q7) — l'ancien (`match_type`/`match_value`) **ou** un critère neuf (`criterion`), jamais les deux
  combinés — plus des **filtres** facultatifs sens, devise et compte bancaire. Critères neufs : `payer_email_exact`
  (normalisé : casse, espaces), `item_name_contains`, `event_code_prefix`, **`shop_equals`** (B19 : domaine ou
  boutique, comparé à `bank_transactions.shop_ref`, voir 38-8) ; le préfixe de facture passe par le critère existant
  `reference_contains` (le connecteur pose le numéro de facture dans `reference`, E8). **Ligne mère d'une règle à
  critère neuf** : `match_type = 'reference_contains'` (valeur existante) et `match_value =
  'kesh-rule:' + SHA-256 hex` de (critère, valeur normalisée, sens, devise, compte) — une **clé inerte**, qu'aucune
  référence bancaire ne contient : un lecteur qui ignore la table enfant (binaire antérieur, retour par SQL) n'y
  rapproche rien, et l'unicité existante `UNIQUE (company_id, match_type, active_uniq)` (`20260513000001:65`) tient
  l'unicité des règles neuves sans toucher l'index. L'API ne rend jamais cette clé : elle rend `criterion` et
  `criterionValue` ; l'export CSV des règles ajoute les colonnes de la table enfant. *Rejeté* : `active = FALSE` sur la
  ligne mère (aussi inerte, mais fausse la colonne `active` lue par toutes les listes et `find_active_for_company`).
  Les filtres s'appliquent aussi aux quatre critères existants ; leur unicité reste `(type, valeur)` — deux règles
  « Migros » qui ne diffèrent que par le sens sont refusées (`RECONCILIATION_RULE_DUPLICATE`), limite écrite au
  manuel. `rule_matches` (`rules.rs:46`) reçoit la règle **avec** sa ligne enfant : critère neuf évalué s'il est posé,
  sinon `match_type` ; filtres ensuite ; priorité inchangée (`priority`, `id`). Le compte de la règle peut être **en
  devise** : il ne s'applique alors qu'aux transactions de cette devise (`RULE_ACCOUNT_CURRENCY_MISMATCH` par
  proposition). **Ventes par devise (B19)** : une règle par boutique et par devise (filtre `currency`), vers le
  sous-compte de ventes de la devise ; l'analytique (`default_project_id`, `20260704000001`) peut porter la catégorie,
  **non tranché** (Q3). *Raison* : un seul critère principal garde le moteur et sa priorité ; sens, devise et compte
  suffisent à séparer vente et achat d'un même courriel.
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
- **E19 — Migrations : aucune breaking (B16).** Toutes créent des tables ou ajoutent des colonnes `NULL` ; aucune ne
  touche un `CHECK`, un `ENUM`, un type ni une ligne existante ; **`kesh_version_min_required` inchangé**, aucun
  relèvement des crates imposé (la version de la release reste à Guy). **Preuve au code qu'aucune valeur inconnue
  n'apparaît à un binaire antérieur** (lecteurs relevés à l'inventaire) :
  - `account_kind` — seul `'bank'` est écrit pour un compte prestataire ; les lecteurs (`BankAccountKind::from_str`,
    `entities/bank_account.rs:21-69` ; `accounts.rs:482` ; `invoice_settlements_write.rs:203-218` ;
    `routes/bank_accounts.rs`, `companies.rs:422-438`, `onboarding.rs:665`, `csv_tables.rs:661-680`) ne voient que
    des valeurs qu'ils connaissent ;
  - `match_type` — une règle à critère neuf porte `'reference_contains'` et une clé inerte (E14) ; les lecteurs
    (`ReconciliationMatchType`, `entities/reconciliation_rule.rs:32` ; `rules.rs:46-70` ; `reconciliation_rules.rs:63,
    82` ; `csv_tables.rs:1015` ; écran des règles) décodent, et `rule_matches` n'y rapproche rien ;
  - `source_format` — un passage écrit `'MANUAL'`, valeur connue depuis la 0.17.0 (E5) ; lecteurs relevés à
    l'inventaire, tous d'affichage ;
  - colonnes neuves de `bank_transactions` et `bank_imports` — lues par listes de colonnes nommées
    (`repositories/bank_transactions.rs:21`, `bank_imports.rs:30`) : ignorées ;
  - tables neuves — inconnues d'un binaire antérieur. ⚠️ En pratique un binaire antérieur **ne démarre pas** sur une
    base migrée (`sqlx` refuse une migration appliquée inconnue, P1 de `CLAUDE.md`) et refuse un `.keshbackup` dont
    l'inventaire de tables diffère : B16 protège surtout le **retour par SQL** (supprimer les tables et colonnes neuves
    et les lignes de `_sqlx_migrations`). Ce SQL de retour s'écrit au manuel d'administration et se joue à la recette ;
    il **supprime d'abord** les règles à critère neuf (lignes mères repérées par la table enfant) et les en-têtes de
    passage (transactions en cascade), qui deviendraient sinon des règles et des « saisies manuelles » sans objet.
  Garde-fous P5 (audit d'idempotence), P6 (sites positionnels), P7 (aucune migration de l'epic n'écrit de données :
  `EXEMPT_MIGRATIONS` n'est pas touché, sauf découverte à la fiche) et liste close R1 (E18) à chaque fiche.
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

## Registre des contrats entre stories

- **C-38-0 — Aucune migration breaking** (toutes) : E19 ; une fiche qui découvre une valeur neuve à écrire dans une
  colonne existante à domaine fermé, un `MODIFY`/`CHANGE COLUMN` ou un `DROP` s'arrête et le dit (B16).
- **C-38-1 — Clé et chiffrement** (38-1) : `KESH_SECRETS_KEY`, `kesh-api::secrets::{seal, open}`, AES-256-GCM, version
  de clé dans l'enregistrement ; rotation : hors périmètre, issue (C-38-13). Seule voie de lecture d'un secret : le
  transport d'un passage. Secrets effacés à toute restauration (E2).
- **C-38-2 — Connexions** (38-1) : `payment_provider_connections` (`id`, `company_id`, `provider`, `label`,
  `fetch_from`, `schedule`, `last_run_at`, `last_run_status`, `last_error`, `created_by_user_id`, `shop_source` NULL (B19, choisi en 38-8), `version`) ; routes
  `GET|POST /api/v1/provider-connections`, `GET|PATCH|DELETE /{id}`, `PUT /{id}/secrets`, `POST /{id}/test` ;
  suppression refusée tant qu'une ligne `payment_provider_accounts` y est rattachée (`PROVIDER_CONNECTION_IN_USE`).
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
  `source_format = 'MANUAL'` jamais affiché comme tel, suppression refusée (`PROVIDER_IMPORT_NOT_DELETABLE`) ;
  `kesh_providers::Connector` (`balances(as_of)`, `movements(window, page)`) ; fenêtre ≤ 31 jours, découpée ;
  recouvrement de 3 jours ; fin = maintenant − 3 h ; début = max(`fetch_from`, dernier passage réussi − 3 jours) ; un
  échec garde ce qui est inséré, l'état dit où reprendre ; soldes rapportés et contrôle de complétude (E20, (a)).
- **C-38-6 — Règles** (38-4) : E14 ; `reconciliation_rule_conditions` ; routes de règles inchangées, champs neufs
  facultatifs (`criterion`, `criterionValue`, `direction`, `currency`, `bankAccountId`) ; `matchType` et `matchValue`
  absents ou nuls dans la réponse d'une règle à critère neuf ; `RULE_CRITERIA_CONFLICT` si les deux critères
  principaux sont fournis.
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
  des ventes hors facture et impôt sur les acquisitions dans le rapport TVA (voir *Point TVA pour Guy*) ; unicité des
  règles à critère existant par sens ou devise (E14, limite) ; catégorie de vente par analytique (B19, Q3). *(Les
  outils MCP n'y sont plus : décidé, aucun outil neuf, E23.)*

## Stories

| Story | Objet | Modules (≤ 5) | Dépend de | Migration |
|---|---|---|---|---|
| 38-0 | kickoff : décomptes, validation de ce fichier | — | — | — |
| 38-1 | coffre des secrets et connexions de prestataire | `kesh-db`, `kesh-api` (config, secrets, routes) | Epic 37 | oui, non breaking (`payment_provider_connections`, `_kesh_provider_secrets`) |
| 38-2 | compte bancaire `bank` + réglages de prestataire, lié à un compte en devise existant (D8), compte de frais par devise | `kesh-db`, `kesh-api` (bank_accounts, provider-connections), `kesh-core` (gardes) | 38-1 | oui, non breaking (`payment_provider_accounts`) |
| 38-3 | téléchargement : cadre `kesh-providers`, transactions de prestataire, doublon, passages, planifié et à la demande, soldes vus et rapportés, contrôle de complétude | `kesh-providers` (neuf), `kesh-db`, `kesh-core`, `kesh-api` | 38-2 | oui, non breaking (`provider_fetch_runs`, `provider_balances_seen`, `provider_balance_snapshots` ; colonnes `bank_transactions`, `bank_imports.provider_fetch_run_id`) |
| 38-4 | critères neufs des règles d'affectation (dont boutique / domaine), filtres sens, devise, compte | `kesh-db`, `kesh-reconciliation`, `kesh-api` (reconciliation_rules) | 38-3 | oui, non breaking (`reconciliation_rule_conditions`) |
| 38-5 | code TVA par défaut d'un compte, extraction de la TVA, compte d'impôt sur les acquisitions | `kesh-db`, `kesh-core`, `kesh-api` (accounts, réglages TVA), `frontend` (accounts, réglages) | Epic 37 | oui, non breaking (`accounts` × 2, `company_invoice_settings` × 1) |
| 38-6a | propositions de prestataire : vente, achat, frais (brut, TVA, frais, cours) et acceptation en lot ; « déjà comptabilisée » | `kesh-reconciliation`, `kesh-api` (reconciliation), `kesh-db` | 38-3, 38-4, 38-5 | non attendue |
| 38-6b | conversions, retraits, remboursements (tels que rapportés), litiges ; file « à examiner » | `kesh-reconciliation`, `kesh-api` (reconciliation, conversions), `kesh-db` | 38-6a | non attendue |
| 38-6c | rattachement aux écritures existantes : en devise, en lot (2024, B17) ; API des scripts externes (B21) | `kesh-db` (reconciliation_link, bank_transactions), `kesh-api` (reconciliation, bank_accounts) | 38-3 | non attendue |
| 38-7 | client PayPal : OAuth2, Transaction Search, fenêtres de 31 jours, pagination, soldes (`as_of_time`) | `kesh-providers` (paypal), `kesh-api` (transport) | 38-3 | non |
| 38-8 | correspondance des codes d'événement PayPal vers les mouvements ; champ de boutique (`shop_ref`) | `kesh-providers` (paypal), `kesh-api` (réglage de connexion) | 38-7 | non (`shop_source` est posé NULL par 38-1) |
| 38-9a | écrans : connexions, secrets, soldes proposés, comptes prestataires, contrôles de solde | `frontend` (provider-connections neuf, bank-accounts), i18n, manuel d'administration (SQL de retour, E19) | 38-1 à 38-3, 38-7 | non |
| 38-9b | écrans : règles étendues, propositions multi-lignes, rattachement en lot, file « à examiner », E2E | `frontend` (reconciliation, règles), i18n, manuel utilisateur, E2E | 38-4 à 38-6c, 38-8 | non |
| 38-10 | relevé joint à la main (compte et exercice ou mois), solde de clôture, écart aux contrôles de clôture | `kesh-db`, `kesh-api` (routes, sauvegarde, closing-checks), `frontend` (bank-accounts) | 38-2, 38-3 | oui, non breaking (`bank_account_documents`) |

**Migrations : six, toutes non breaking** (38-1, 38-2, 38-3, 38-4, 38-5, 38-10) ; aucune n'écrit de données (P7 :
rien à trier, sauf découverte à la fiche). **Ordre** : 38-1 → 38-2 → 38-3 → (38-4, 38-5, 38-6c, 38-7 en parallèle
possibles) → 38-8 → 38-6a → 38-6b → 38-9a → 38-9b ; 38-10 dès 38-3. 38-6c avant 38-6a de préférence : les transactions
de 2024 se rattachent avant que des propositions ne leur soient faites. Une story qui découvre une migration non
prévue le dit avant de l'écrire (C-38-0).

### 38-1 — Coffre des secrets et connexions de prestataire

- *Intention* : un Administrateur crée une connexion (PayPal, libellé, date de départ, réglage quotidien ou à la
  demande) et y dépose ses secrets (identifiant et secret de l'application REST), chiffrés, jamais réaffichés.
- *Périmètre* : migration (deux tables, E1, E2), `kesh-api::config` (`KESH_SECRETS_KEY`), module `secrets` (C-38-1),
  routes C-38-2, effacement des secrets à la restauration, audit, tables à `backup.rs` (applicative à
  `TABLES_TO_TRUNCATE`, système à la liste `_kesh_*`), liste close R1 (E18). Dépendance neuve : `aes-gcm` (à déclarer
  avec sa licence).
- *Risques* : clé perdue = secrets illisibles (ressaisie, dit au manuel) ; aucun secret dans les journaux, erreurs,
  réponses, audit ni sauvegarde (tests de recherche de la valeur) ; remise à zéro de la société (connexion vidée,
  secrets effacés) ; `KESH_SECRETS_KEY` à écrire au manuel d'administration, aux exemples de `.env` et à
  `docker-compose`.

### 38-2 — Compte bancaire prestataire : compte `bank` + réglages (D8, B16)

- *Intention* : chaque solde de devise d'une connexion devient un compte bancaire Kesh ordinaire, lié à un compte du
  plan déjà en devise (numéro, historique et soldes gardés), avec ses réglages de prestataire : connexion, devise,
  compte de frais de la devise.
- *Périmètre* : migration **non breaking** (`CREATE TABLE payment_provider_accounts`, E11) ; routes C-38-3 ; levée
  de `ACCOUNT_IN_FOREIGN_CURRENCY` (C-37-9) pour un compte porteur de réglages, posés dans la même transaction que le
  lien (création et `PATCH` du lien du compte bancaire) ; gardes de devise et de compte de frais ; exclusion du compte
  principal, des lots de paiement et de l'assistant ; exports (colonnes de la table neuve) ; tables à `backup.rs`
  (ordre enfants → parents) et liste close R1 (E18). `BankAccountKind` **inchangé**.
- *Risques* : **chaque site qui doit écarter un compte prestataire** lit désormais la table neuve (inventaire au
  `grep -rn "is_primary\|bank_accounts" crates/kesh-api/src/routes` à la fiche) ; retrait des réglages d'un compte lié
  en devise (refusé, sinon compte ordinaire lié à un compte en devise, contraire à C-37-9) ; verrou de devise du compte
  du plan (37-1) contre lien concurrent ; un compte bancaire archivé ou clos (28-4a) porteur de réglages ; recette de
  montée depuis l'image publiée de l'Epic 37 (montée sans perte, retour par la sauvegarde et par le SQL de E19).

### 38-3 — Téléchargement : cadre commun

- *Intention* : Kesh télécharge les mouvements d'une connexion, chaque jour ou à la demande, sans doublon, depuis la
  date de départ, et liste les soldes sans compte.
- *Périmètre* : crate `kesh-providers` (types, trait `Connector` objet-safe, validation), colonnes de
  `bank_transactions` (C-38-4) et `bank_imports.provider_fetch_run_id`, tables `provider_fetch_runs`,
  `provider_balances_seen`, `provider_balance_snapshots`, passage (C-38-5, E5, E6, E10, E12, E20 (a)), en-tête
  `MANUAL` affiché « Téléchargé de … » et non supprimable, route `POST /provider-connections/{id}/fetch`, transport
  factice en `KESH_TEST_MODE` + route `_test`, connecteur factice pour les tests.
- *Risques* : statut d'une transaction qui change après coup (en attente → réussie, inversée) ; passage concurrent
  (verrou) ; reprise après échec au milieu d'une fenêtre ; réconciliation d'une transaction non encore classée ;
  volume (trois ans × trois soldes) ; heure locale contre UTC des dates du prestataire (date comptable = date en
  Europe/Zurich, à écrire) ; **chaque affichage de `source_format`** (inventaire) qui dirait « Saisie manuelle » pour
  un passage ; utilisateur de la connexion désactivé (auteur du passage quotidien) ; ordre de restauration
  (`provider_fetch_runs` avant `bank_imports`).

### 38-4 — Critères neufs des règles d'affectation

- *Intention* : une règle reconnaît une transaction de prestataire par courriel, numéro de facture, nom d'article,
  code d'événement ou **boutique / domaine** (B19), et se restreint par sens, devise et compte — pour proposer le bon
  sous-compte de ventes de la devise et de la boutique.
- *Périmètre* : E14 ; migration non breaking (`reconciliation_rule_conditions`) ; `rule_matches` étendu
  (`rules.rs:46`) et ses appelants (`first_matching_rule`, revalidation d'`accept_one_rule`) ; clé inerte de la ligne
  mère ; garde de devise du compte de la règle ; routes `reconciliation_rules` (champs facultatifs, rétrocompatibles,
  `RULE_CRITERIA_CONFLICT`) ; export CSV des règles.
- *Risques* : les règles existantes ne changent pas de comportement (tests de non-régression des quatre critères) ;
  **aucun site ne rend ni n'évalue la clé inerte** (inventaire des lecteurs de `match_value`) ; normalisation du
  courriel (casse, espaces) ; priorité entre règles larges et étroites (inchangée : `priority`) ; modification d'une
  règle qui passe d'un critère existant à un critère neuf (ligne mère réécrite, enfant posé, même transaction).
- *Boutique / domaine* : **aucun champ « domaine » dans Transaction Search** (doc PayPal connue : `transaction_info`
  porte `invoice_id`, `custom_field`, `transaction_subject`, `transaction_note` ; `cart_info.item_details[]` porte
  `item_name`, `item_code` ; `payer_info` le courriel et le nom du **payeur** ; `shipping_info` l'adresse de
  **livraison** ; `store_info.store_id` pour les ventes en magasin). Le domaine se déduit donc d'un champ que la
  boutique remplit — vraisemblablement `custom_field` ou un préfixe d'`invoice_id` (WHMCS, CB Paid Subscriptions) —
  ⚠️ **incertain** : à constater sur les réponses enregistrées de Beat (38-8, Q10). 38-4 compare à `shop_ref`, que
  38-8 remplit selon un réglage de la connexion.

### 38-5 — Code TVA par défaut d'un compte

- *Intention* : un compte de produit ou de charge porte son genre et son taux de TVA par défaut, que les propositions
  appliquent (B6) et que la saisie manuelle pourra reprendre.
- *Périmètre* : E13 ; colonnes `accounts`, champ du formulaire de compte, fonctions pures et miroir TS, gardes (taux
  actif à la date, compte de résultat seulement) ; **compte d'impôt sur les acquisitions** aux réglages TVA (B18 :
  `company_invoice_settings.default_vat_acquisition_account_id`, passif, actif, en CHF ; champ de l'écran des
  réglages).
- *Risques* : arrondi de l'extraction (centime) ; taux changé en cours d'exercice (taux en vigueur à la date de la
  transaction, par `valid_from`/`valid_to`) ; un compte en devise : TVA en devise principale (S5).

### 38-6a — Propositions de prestataire : ventes, achats, frais

- *Intention* : chaque vente ou achat téléchargé reçoit une proposition complète, acceptée en lot.
- *Périmètre* : E15 ; constructeur pur multi-lignes dans `kesh-reconciliation` (à côté de `manual.rs:68`, qui reste
  à deux lignes) ; branchement dans `get_proposals` (`reconciliation.rs:999`, saut hors CHF levé pour les comptes
  prestataires, `:1098`) et `post_accept` (`:1593`), variante `provider` ; frais seuls (`movement_kind = fee`) au
  compte de frais de la devise ; `alreadyBooked` et `confirmAlreadyBooked` (E15, B17).
- *Risques* : **`reconciliation.rs` fait 7 439 lignes** : le code neuf va dans un module à part, appelé depuis le
  répartiteur ; équilibre en devise principale à plusieurs lignes en devise (arrondis, 37-3a) ; exercice mono-devise ;
  compte de la règle archivé entre proposition et acceptation (revalidation sous verrou, précédent `accept_one_rule`).

### 38-6b — Conversions, retraits, remboursements, litiges ; file « à examiner »

- *Intention* : les mouvements qui ne sont ni vente ni achat se proposent juste, ou attendent un humain.
- *Périmètre* : E16 ; paire de conversion vers `POST /journal-entries/conversions` (A31) ; rattachement des retraits
  (34-11b) ; inverse d'une écriture d'origine ; file (C-38-9).
- *Risques* : paire de conversion dont une moitié manque (passage interrompu) ; remboursement partiel (montants
  rapportés, TVA extraite du brut remboursé, B20) ; frais rendus en partie ou pas du tout ; litige résolu en faveur du
  vendeur (deux mouvements qui s'annulent) ; annulation d'une réconciliation de paire (les deux côtés).

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

### 38-7 — Client PayPal

- *Intention* : lire, pour une connexion PayPal, les soldes et les transactions qui changent le solde.
- *Périmètre* : OAuth2 `client_credentials` (jeton gardé en mémoire jusqu'à expiration), `GET
  /v1/reporting/transactions` (`start_date`/`end_date` ≤ 31 jours, `fields=all`, `page_size=500`, `page`,
  `balance_affecting_records_only=Y`), `GET /v1/reporting/balances` ; plafond de 10 000 résultats → la fenêtre se
  redécoupe ; trois ans d'historique → `fetch_from` antérieur refusé à l'écran et borné au passage ; erreurs (401,
  403 permission *Transaction Search* manquante, 429, 5xx) en états lisibles ; URL de base constante (*live*), autre
  base seulement en test.
- *Risques* : permission manquante sur l'application REST (message clair) ; pagination incohérente ; délai de 3 h ;
  fuseau des dates PayPal.

### 38-8 — Codes d'événement PayPal

- *Intention* : chaque transaction PayPal devient un `ProviderMovement` classé, brut, frais, net, devise, liens.
- *Périmètre* : table de correspondance par famille de codes `T00xx` (paiements : vente ou achat selon le signe),
  `T01xx` (frais), `T02xx` (conversion), `T03xx`/`T04xx` (dépôt, retrait banque), `T11xx` (inversions,
  remboursements), `T12xx` (ajustements, rétrofacturations → litige), `T15xx`/`T21xx` (blocages, réserves : ignorés
  s'ils ne changent pas le solde), le reste → `other` (file). **À valider sur les réponses enregistrées de Beat** :
  chaque code relevé reçoit un test. **Champ de boutique (B19)** : réglage de la connexion `shop_source` ∈
  `custom_field`, `invoice_prefix` (partie avant un séparateur), `item_name`, `store_id` (tolérant), choisi sur les
  réponses enregistrées ; le connecteur en tire `shop_ref` (E8).
- *Risques* : un code inconnu ne se classe jamais en vente (→ `other`) ; signe des frais ; regroupement des deux
  moitiés d'une conversion.

### 38-9a et 38-9b — Écrans

- *38-9a* : liste et fiche des connexions (secrets en saisie seule, « Tester », « Télécharger maintenant », état du
  dernier passage), soldes proposés (créer ou lier un compte, B2), compte bancaire prestataire et compte de frais de la
  devise, contrôles de solde (E20 : téléchargement incomplet, écart comptable) ; manuel d'administration (clé,
  secrets à ressaisir après restauration, permissions PayPal, recette, SQL de retour de E19).
- *38-9b* : critères neufs et filtres au formulaire des règles ; propositions multi-lignes (aperçu des lignes, TVA,
  frais, cours, « déjà comptabilisée ») ; acceptation en lot ; **rattachement en lot** (suggestions univoques à
  cocher, ambiguës à trancher une à une, E21) ; file « à examiner » ; manuel utilisateur (et la voie API des scripts,
  E22) ; E2E (contre le transport factice) et captures d'écran (*Recette visuelle*).
- *Risques* : un secret ne revient jamais dans le DOM ; textes sans « automatique » trompeur (proposer, non
  comptabiliser).

### 38-10 — Relevé joint à la main

- *Intention* : le relevé annuel PayPal (PDF, sans API) se range au compte et à l'exercice, et son solde de clôture
  sert de **somme de contrôle** (B20).
- *Périmètre* : E17, C-38-10, E20 (b) ; écran du compte bancaire (onglet *Relevés*, écart au solde comptable en
  devise) ; contrôles de clôture de l'exercice (avertissement non bloquant) ; sauvegarde complète.
- *Risques* : fichier qui n'est pas un PDF ; taille ; suppression d'un exercice ou d'un compte (refus tant qu'une
  pièce y est rangée, ou suppression de la pièce : à trancher à la fiche).

## Ce que réutiliseront les Epics 39 (Wise) et 40 (Stripe)

**Réutilisé tel quel** : coffre des secrets et connexions (38-1), réglages de compte prestataire et compte de frais
(38-2), cadre de téléchargement, passages, doublon, planification, soldes vus et contrôles (38-3), règles étendues
(38-4), TVA par défaut (38-5), propositions et file (38-6a, 38-6b), rattachement en lot et API des scripts (38-6c),
écrans génériques (38-9a, 38-9b), pièces de compte (38-10). Aucune migration breaking : les valeurs neuves de 39 et 40
(`provider`, genres de mouvement, `shop_source`) vont dans les colonnes de l'epic, lues avec tolérance (E4) ; un
compte de plus (p. ex. le compte intermédiaire de Stripe) est une colonne `NULL` de `payment_provider_accounts`.

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

## Questions ouvertes

*Closes* : Q1 (aucune migration breaking, B16), Q2 (rattachement en lot, B17, E21), Q3 sauf l'analytique (B18,
B19), Q4 et Q5 (décidées, E2), Q6 (B20, E16), Q7 (un critère principal + filtres, E14), Q8 (aucun outil MCP, E23).

- **Q3 — Catégorie de vente (reste de Q3, B19)** : sous-comptes de ventes par devise et par catégorie, ou catégorie
  portée par l'analytique (`default_project_id` de la règle) ? Non tranché par Beat ; l'epic marche dans les deux cas
  (une règle par boutique et par devise).
- **Q9 — Numéro 40 pour Stripe** : à confirmer par Guy (cadrage).
- **Q10 — Champ de boutique (B19, Beat)** : quel champ ses boutiques (WHMCS, CB Paid Subscriptions) remplissent-elles
  chez PayPal — `custom_field`, préfixe d'`invoice_id`, nom d'article ? À constater d'abord sur ses réponses
  enregistrées (38-8) ; la question ne se pose que si elles ne le disent pas.
- **Q11 — En-tête `MANUAL` d'un passage (E5, Guy)** : aucune valeur existante de `source_format` n'est exacte pour un
  téléchargement ; `MANUAL` (« sans fichier ») est retenu et jamais affiché comme tel par le binaire de l'epic. Guy
  préfère-t-il une valeur neuve (`PROVIDER_API`) au prix d'une migration breaking ? B16 dit non ; la question ne
  rouvre B16 que s'il le demande.

## Règles de cet epic

- Méthode du dépôt (`CLAUDE.md`) : fiche → validation en boucle → développement → revue en boucle (Sonnet ↔ Opus pour
  les passes complètes, passe ciblée en fin de boucle), gate local complet avant la PR ; splitting préventif (≤ 5
  modules par story).
- **Production** : l'instance de Beat n'est jamais écrite par le travail de cet epic. Recette sur ses comptes PayPal
  réels, avec une application REST en lecture seule (*Transaction Search*), sur une **instance jetable** (B12).
- Aucun appel réseau en CI ; aucune réponse réelle non anonymisée dans le dépôt.
- On trace toujours : un défaut voisin non traité se note au registre (C-38-13), pour devenir une issue avec la PR.
