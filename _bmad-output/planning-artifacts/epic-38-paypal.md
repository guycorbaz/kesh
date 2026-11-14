# Epic 38 — Comptes PayPal : téléchargement et imputation automatiques

**Cadrage : 2026-11-15** (branche `epic/38-paypal`). Demande de Beat (contributeur), décisions B1–B15 de
`epics-38-40-cadrage.md`, qui **priment** sur ce fichier là où il diverge. Kickoff (décomptes d'issues et de bugs,
validation de ce fichier) **à tenir** : `38-0-kickoff` reste au backlog tant que l'Epic 37 n'est pas développé.

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
sur un lien bancaire) que 38-2 lève, **pour le seul type « prestataire »**. L'issue promise par D8 s'ouvre avec la PR de
l'Epic 37 ; celle de l'Epic 38 la ferme.

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

## Inventaire du code existant (relevé le 2026-11-15)

| Domaine | Ce qui existe | Où |
|---|---|---|
| Comptes bancaires | `account_kind` `bank` \| `credit_card`, domaine fermé à l'octet, IBAN facultatif, `card_last4` ; migration **breaking** (min 0.17.6) | `migrations/20261110000001_bank_accounts_without_iban.sql:39-55` |
| | `BankAccountKind`, `FromStr` **refuse l'inconnu** ; `admits_ledger_type` (carte : passif seul) | `kesh-db/src/entities/bank_account.rs:21-69` |
| | dépôt (colonnes lues, création, `PUT`) | `kesh-db/src/repositories/bank_accounts.rs:54, 65, 769` |
| | routes `/api/v1/bank-accounts…` | `kesh-api/src/lib.rs:938-954` |
| Transactions bancaires | `bank_imports` (`source_format` VARCHAR sans CHECK, `file_hash`, période), `bank_transactions` (`amount`, `currency`, `reference`, `details`, `end_to_end_id`, `transaction_id`, `counterparty_*`, `status` `pending`\|`reconciled`) | `migrations/20260504000001_bank_imports.sql:19-88` |
| | `BankImportSourceFormat` (`CAMT053_V04`, `_V08`, `CSV`, `MANUAL`), `FromStr` refuse l'inconnu : `MANUAL` a coûté une migration breaking | `kesh-db/src/entities/bank_import.rs:26-49` ; `migrations/20261030000001_bank_imports_manual_source.sql` |
| | `BankTransactionStatus` | `kesh-db/src/entities/bank_transaction.rs:19-25` |
| | clé de doublon (date, montant, référence normalisée, compte), fenêtre et alias | `kesh-core/src/bank_imports.rs:295-360, 598-640` ; `kesh-db/src/repositories/bank_transactions.rs:83` |
| | empreinte d'aperçu d'import | `kesh-api/src/routes/bank_import_fingerprint.rs:90-180` |
| | routes d'import CAMT/CSV et relevé manuel | `kesh-api/src/lib.rs:831-851` |
| Réconciliation | `post_accept` (lot, *Pattern batch*), `accept_one_rule`, `_invoice`, `_split`, `FailedProposal` | `kesh-api/src/routes/reconciliation.rs:571, 1593, 3518, 3946` (fichier de **7 439 lignes**) |
| | propositions, rattachement d'un virement entre ses comptes (34-11b), liaison à une écriture existante (34-4) | `reconciliation.rs:999, 6373, 6656` ; `kesh-db/src/repositories/reconciliation_link.rs:90, 358, 535` ; `lib.rs:874-932` |
| | écriture d'une règle : **deux lignes** (banque, contrepartie) | `kesh-reconciliation/src/manual.rs:68` |
| Règles | `reconciliation_rules` : `match_type` **ENUM** (`counterparty_contains`, `_exact`, `reference_contains`, `iban_exact`), une règle → un compte, `UNIQUE (company_id, match_type, active_uniq)`, `default_project_id` | `migrations/20260513000001_reconciliation_rules.sql:36-69` ; `20260704000001_…default_project.sql:12` |
| | `rule_matches`, `first_matching_rule` | `kesh-reconciliation/src/rules.rs:46-95` ; `ReconciliationMatchType` `kesh-db/src/entities/reconciliation_rule.rs:32` |
| TVA | `vat_rates` (libellé, taux, validité) — **aucun code TVA, aucun genre** (dû, préalable, acquisitions) | `migrations/20260428000001_vat_rates.sql` ; `kesh-api/src/routes/vat.rs:59-462` |
| | comptes TVA des réglages de facturation (`default_vat_payable/recoverable/decompte_account_id`) | `migrations/20260614000001_vat_accounts_config.sql:54-56` |
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
  passage). Chaque solde de devise est un **compte bancaire** Kesh de type `payment_provider`, rattaché à la connexion
  et à un compte du plan **de la devise du solde** (B1). *Raison* : un jeu de secrets et un téléchargement servent
  trois soldes ; la réconciliation, elle, reste par compte, comme les relevés des banques.
- **E2 — Secrets hors sauvegarde.** Les secrets vont dans une table **système** `_kesh_provider_secrets` (clé :
  connexion), chiffrés **AES-256-GCM** (nonce aléatoire, AAD = société + connexion + nom du secret + version de clé),
  clé tirée de la variable neuve **`KESH_SECRETS_KEY`** (32 octets en base64 ; gabarit refusé comme
  `is_template_placeholder`). Variable absente : Kesh démarre, les connexions sont inutilisables
  (`PROVIDER_SECRETS_KEY_MISSING`, 409). Une restauration complète **efface** les secrets (précédent : jetons de
  renouvellement révoqués à la restauration, 0.17.3) ; la connexion passe « secret à saisir ». *Raison* : une
  sauvegarde emportée ailleurs n'emporte aucun accès à un compte de paiement ; aucun secret ne dépend d'une clé que la
  sauvegarde n'a pas.
- **E3 — Saisie des secrets.** Administrateur seul, par **session** (écran ou `Authorization: Bearer`), jamais par une
  clé d'API (`require_not_pat`, précédent V6) ; écriture seule : les réponses rendent `secretSet` et
  `secretUpdatedAt`, jamais la valeur ; audit sans la valeur. Le contrôle « Tester la connexion » obtient un jeton et
  lit les soldes.
- **E4 — Décodage tolérant dès cette version (patron A30).** `BankImportSourceFormat`, le genre de mouvement, le
  motif d'examen, le genre TVA d'un compte et `provider` se lisent avec une variante `Other(String)` : une valeur
  ajoutée par les Epics 39 et 40 ne demande ni migration breaking ni relèvement de `kesh_version_min_required`.
  `BankAccountKind` et `ReconciliationMatchType` deviennent tolérants **dans la même migration breaking** qui leur
  ajoute des valeurs (E14).
- **E5 — Un seul format d'import : `PROVIDER_API`.** Chaque passage crée, par compte touché, une ligne `bank_imports`
  `source_format = 'PROVIDER_API'`, période = fenêtre demandée, `file_hash` = SHA-256 des enregistrements normalisés.
  Le prestataire se lit sur la connexion du compte. *Raison* : DRY, et Wise/Stripe sans migration breaking.
- **E6 — Doublon par l'identifiant du prestataire.** Colonne neuve `provider_tx_id`, `UNIQUE (bank_account_id,
  provider_tx_id)` (NULL multiples admis : CAMT, CSV et manuel inchangés). Insertion idempotente
  (`INSERT … ON DUPLICATE KEY` sans écrasement) ; chaque passage **recouvre** les trois derniers jours (délai de 3 h de
  PayPal, statuts tardifs). La clé de doublon par fenêtre (`kesh-core/src/bank_imports.rs:295`) ne s'applique pas.
- **E7 — Une transaction de prestataire = un mouvement du solde.** `amount` = **net** (ce qui change le solde),
  plus `gross_amount` et `fee_amount` ; frais **dans** la même transaction (B3 : brut par la règle, frais au
  sous-compte). Une conversion PayPal donne **deux** transactions (une par solde), reliées par
  `related_provider_tx_id`.
- **E8 — Colonnes génériques, pas de JSON brut.** `bank_transactions` gagne `movement_kind`, `event_code`,
  `counterparty_email`, `invoice_ref`, `item_names`, `related_provider_tx_id`, `review_reason` (toutes NULL). *Raison* :
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
- **E11 — Compte bancaire prestataire.** `account_kind = 'payment_provider'` : sans IBAN, ni QR-IBAN, ni carte, jamais
  principal ; `provider_connection_id` obligatoire ; lié à un compte du plan **actif, de bilan (actif)**, **dans la
  devise du solde** — CHF admis pour le solde CHF ; `UNIQUE (provider_connection_id, currency)`. Lier un compte existant
  garde son numéro, son historique et ses soldes (D8). `fee_account_id` : compte de charge **dans la même devise**, le
  sous-compte de frais (B3) ; le regroupement est son **compte parent** (`accounts.parent_id`), aucun réglage de plus.
- **E12 — Nouveau solde (B2).** Le passage lit les soldes de la connexion ; un solde de devise sans compte Kesh est
  gardé dans `provider_balances_seen` (connexion, devise, premier vu, ignoré ou non) et **listé** à l'écran ; ses
  transactions ne sont pas téléchargées tant que l'Administrateur n'a pas créé ou lié le compte. Jamais de création
  d'office.
- **E13 — Code TVA par défaut d'un compte (B6).** `accounts.default_vat_kind` (`sale` : TVA due incluse dans le
  montant ; `purchase` : impôt préalable inclus ; `acquisition` : impôt sur les acquisitions, calculé sur le montant,
  deux lignes égales dû/préalable ; NULL : sans TVA) et `accounts.default_vat_rate_id` (FK `vat_rates`). Extraction
  pure dans `kesh-core` (`vat_from_gross` = round(brut × t ÷ (100 + t), 2) ; acquisitions = round(montant × t ÷ 100,
  2)). Lignes de TVA **en devise principale**, au cours de la ligne du compte de produit ou de charge (S5, A13). Comptes
  de TVA lus aux réglages existants ; un réglage manquant → `FailedProposal` `VAT_ACCOUNT_NOT_CONFIGURED`.
- **E14 — Règles étendues (B5).** `match_type` gagne `counterparty_email_exact`, `invoice_prefix`,
  `item_name_contains`, `event_code_prefix` ; deux **filtres facultatifs** s'y ajoutent : `direction`
  (`in`|`out`) et `bank_account_id` (règle propre à un compte). L'unicité devient `(company_id, match_type,
  active_uniq, direction, bank_account_id)`. Le compte de la règle peut être **en devise** : il ne s'applique alors
  qu'aux transactions de cette devise (`RULE_ACCOUNT_CURRENCY_MISMATCH` par proposition). *Raison* : un seul critère
  principal garde le moteur actuel (`rule_matches`) et sa priorité ; sens et compte suffisent à séparer vente et achat
  d'un même courriel.
- **E15 — Propositions de prestataire.** Une transaction de prestataire reçoit au plus une proposition, construite par
  une fonction pure de `kesh-reconciliation` : compte du prestataire (net, en devise) ; contrepartie de la règle (brut
  hors TVA) ; TVA (E13) ; frais au `fee_account_id`. Cours = cours retenu `afc` de la société pour le mois
  (`find_company_rate_in_tx`), A32 sinon ; cours manquant → `RATE_UNAVAILABLE` par proposition. Acceptation par
  `POST /reconciliation/accept`, variante neuve `provider` (*Pattern batch* : toute erreur par proposition en
  `failed[]`, identifiant `bank_transaction_id`, codes constants). Aucune écriture d'office.
- **E16 — Mouvements particuliers.**
  - **Conversion** : la paire (E7) se propose comme **une conversion** de 37-3b, `completeDifference: true` (B4,
    A31), cours AFC des deux côtés, différence aux comptes propres du compte en devise (S2) ; les deux transactions
    sont réconciliées ensemble. Coût donné à part par le prestataire → ligne de frais.
  - **Retrait vers la banque / dépôt depuis la banque** : rattachement des deux côtés comme un virement entre ses
    comptes (34-11b, `reconciliation.rs:6373`), le côté banque venant du relevé CAMT.
  - **Remboursement (B7)** : rapproché de l'origine par `related_provider_tx_id` ; proposé comme l'**inverse** de
    l'écriture d'origine (mêmes comptes, TVA comprise, montants du remboursement ; partiel au prorata). Origine absente
    ou non réconciliée → file « à examiner ».
  - **Litige, rétrofacturation, mouvement inconnu** : `review_reason` posé, **aucune proposition** ; file « à
    examiner » ; l'utilisateur passe une écriture manuelle (34-3) ou marque examiné.
  - **Blocages, réserves, autorisations** sans effet sur le solde : non téléchargés (`balance_affecting_records_only`).
- **E17 — Relevé joint (B10).** Table applicative `bank_account_documents` (compte bancaire, exercice, genre
  `statement`, chemin, empreinte, nom d'origine, auteur), fichiers par `document_storage`, emportés par la sauvegarde
  (`ParsedBackup::referenced_files`, `admin_backup/import.rs:115`). Wise (39) y range le PDF obtenu par l'API.
- **E18 — Fenêtre d'import des sauvegardes.** Les tables et colonnes applicatives neuves de l'Epic 38 s'ajoutent à la
  **liste close d'absences admises** de 37-1 AC-6 (R1) : une sauvegarde antérieure se réimporte (sans connexion ni
  donnée de prestataire).
- **E19 — Migrations : une breaking, les autres non.** Les valeurs neuves de `account_kind` (CHECK à l'octet) et de
  `match_type` (ENUM) font échouer le décodage d'un binaire antérieur : **une seule migration breaking** (38-2) les pose
  toutes deux, élargit les CHECK, et relève `kesh_version_min_required` à la version de la release (P2, P2-bis :
  crates relevées dans le même commit). `PROVIDER_API` n'a pas de CHECK mais un binaire antérieur le refuserait : il
  est **posé par la même migration** (aucune ligne, commentaire SQL) — la mention suffit puisque la version minimale
  est déjà relevée. Toutes les autres migrations de l'epic sont non breaking (`CREATE TABLE`, `ADD COLUMN` NULL).
- **E20 — Le relevé de solde.** Chaque passage enregistre le solde rapporté par le prestataire par devise et la date ;
  l'écran du compte le compare au solde comptable (contrôle, jamais d'écriture).

## Registre des contrats entre stories

- **C-38-1 — Clé et chiffrement** (38-1) : `KESH_SECRETS_KEY`, `kesh-api::secrets::{seal, open}`, AES-256-GCM, version
  de clé dans l'enregistrement ; rotation : hors périmètre (question Q5). Seule voie de lecture d'un secret : le
  transport d'un passage.
- **C-38-2 — Connexions** (38-1) : `payment_provider_connections` (`id`, `company_id`, `provider`, `label`,
  `fetch_from`, `schedule`, `last_run_at`, `last_run_status`, `last_error`, `version`) ; routes
  `GET|POST /api/v1/provider-connections`, `GET|PATCH|DELETE /{id}`, `PUT /{id}/secrets`, `POST /{id}/test` ;
  suppression refusée tant qu'un compte bancaire y est rattaché (`PROVIDER_CONNECTION_IN_USE`).
- **C-38-3 — Compte prestataire** (38-2) : E11 ; codes `PROVIDER_ACCOUNT_CURRENCY_MISMATCH`,
  `PROVIDER_ACCOUNT_DUPLICATE_CURRENCY`, `FEE_ACCOUNT_INVALID` ; levée de C-37-9 pour ce seul type.
- **C-38-4 — Transaction de prestataire** (38-3) : colonnes E6, E7, E8 ; `movement_kind` ∈ `sale`, `purchase`, `fee`,
  `conversion`, `refund`, `dispute`, `withdrawal`, `deposit`, `other` (tolérant) ; `review_reason` ∈ `dispute`,
  `refund_origin_missing`, `unknown_event`, `other` (tolérant).
- **C-38-5 — Passage** (38-3) : `kesh_providers::Connector` (`balances`, `movements(window, page)`) ; fenêtre ≤ 31
  jours, découpée ; recouvrement de 3 jours ; fin = maintenant − 3 h ; début = max(`fetch_from`, dernier passage
  réussi − 3 jours) ; un échec garde ce qui est inséré, l'état dit où reprendre.
- **C-38-6 — Règles** (38-4) : E14 ; routes de règles inchangées, champs neufs facultatifs.
- **C-38-7 — TVA par défaut** (38-5) : E13 ; `PUT /accounts/{id}` gagne `defaultVatKind`, `defaultVatRateId` ;
  `kesh_core::accounting::vat::{vat_from_gross, acquisition_vat}` et leur miroir TS, tenus par une fixture partagée.
- **C-38-8 — Propositions** (38-6a, 38-6b) : E15, E16 ; variante `provider` de `accept` ; codes
  `RATE_UNAVAILABLE`, `VAT_ACCOUNT_NOT_CONFIGURED`, `RULE_ACCOUNT_CURRENCY_MISMATCH`, `FEE_ACCOUNT_INVALID`,
  `PROVIDER_TX_UNDER_REVIEW`, `MULTI_CURRENCY_DISABLED` (409 hors lot, `failed[]` dans le lot).
- **C-38-9 — File « à examiner »** (38-6b) : `GET /api/v1/reconciliation/review` ; `POST
  /reconciliation/transactions/{id}/review-done` (Comptable).
- **C-38-10 — Pièces de compte** (38-10) : E17 ; `GET|POST /api/v1/bank-accounts/{id}/documents`,
  `GET|DELETE /…/documents/{docId}` ; PDF seul, taille bornée.
- **C-38-11 — Tests** (toutes) : aucun appel réseau en CI ; fixtures enregistrées **anonymisées** (courriels, noms,
  identifiants remplacés ; montants et codes gardés) sous `crates/kesh-providers/tests/fixtures/paypal/` ; tests de
  réponses hostiles (taille, structure, montants, dates, devises, pagination incohérente).
- **C-38-12 — Registre des routes** : chaque route neuve entre à `LIB_ROUTES` (et `TEST_ENDPOINT_ROUTES` pour la route
  `_test`), décompte en deltas à chaque fiche.
- **C-38-13 — Tracé pour plus tard** (issues avec la PR) : rotation de `KESH_SECRETS_KEY` ; outils MCP de lecture des
  connexions et de la file ; TVA due des ventes hors facture dans le rapport TVA (voir *Point TVA pour Guy*).

## Stories

| Story | Objet | Modules (≤ 5) | Dépend de | Migration |
|---|---|---|---|---|
| 38-0 | kickoff : décomptes, validation de ce fichier | — | — | — |
| 38-1 | coffre des secrets et connexions de prestataire | `kesh-db`, `kesh-api` (config, secrets, routes) | Epic 37 | oui, non breaking (`payment_provider_connections`, `_kesh_provider_secrets`) |
| 38-2 | compte bancaire « prestataire » lié à un compte en devise existant (D8), sous-compte de frais | `kesh-db`, `kesh-api` (bank_accounts), `kesh-core` (gardes) | 38-1 | **oui, breaking** (E19 : `account_kind`, `match_type`, `PROVIDER_API`) |
| 38-3 | téléchargement : cadre `kesh-providers`, transactions de prestataire, doublon, passage planifié et à la demande, soldes vus | `kesh-providers` (neuf), `kesh-db`, `kesh-core`, `kesh-api` | 38-2 | oui, non breaking (colonnes `bank_transactions`, `provider_balances_seen`) |
| 38-4 | critères neufs des règles d'affectation | `kesh-db`, `kesh-reconciliation`, `kesh-api` (reconciliation_rules) | 38-2, 38-3 | oui, non breaking (`direction`, `bank_account_id`, unicité refaite) ; les valeurs d'ENUM sont posées par 38-2 |
| 38-5 | code TVA par défaut d'un compte, extraction de la TVA | `kesh-db`, `kesh-core`, `kesh-api` (accounts), `frontend` (accounts) | Epic 37 | oui, non breaking (`accounts` × 2) |
| 38-6a | propositions de prestataire : vente, achat, frais (brut, TVA, frais, cours) et acceptation en lot | `kesh-reconciliation`, `kesh-api` (reconciliation), `kesh-db` | 38-3, 38-4, 38-5 | non attendue |
| 38-6b | conversions, retraits, remboursements, litiges ; file « à examiner » | `kesh-reconciliation`, `kesh-api` (reconciliation, conversions), `kesh-db` | 38-6a | non attendue |
| 38-7 | client PayPal : OAuth2, Transaction Search, fenêtres de 31 jours, pagination, soldes | `kesh-providers` (paypal), `kesh-api` (transport) | 38-3 | non |
| 38-8 | correspondance des codes d'événement PayPal vers les mouvements | `kesh-providers` (paypal) | 38-7 | non |
| 38-9a | écrans : connexions, secrets, soldes proposés, comptes prestataires | `frontend` (provider-connections neuf, bank-accounts), i18n, manuel d'administration | 38-1 à 38-3, 38-7 | non |
| 38-9b | écrans : règles étendues, propositions multi-lignes, file « à examiner », E2E | `frontend` (reconciliation, règles), i18n, manuel utilisateur, E2E | 38-4 à 38-6b, 38-8 | non |
| 38-10 | relevé joint à la main (compte et exercice) | `kesh-db`, `kesh-api` (routes, sauvegarde), `frontend` (bank-accounts) | 38-2 | oui, non breaking (`bank_account_documents`) |

**Ordre** : 38-1 → 38-2 → 38-3 → (38-4, 38-5, 38-7 en parallèle possibles) → 38-8 → 38-6a → 38-6b → 38-9a → 38-9b ;
38-10 dès 38-2. Une story qui découvre une migration non prévue le dit avant de l'écrire.

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

### 38-2 — Compte bancaire « prestataire » (D8)

- *Intention* : chaque solde de devise d'une connexion devient un compte bancaire Kesh lié à un compte du plan déjà en
  devise, numéro, historique et soldes gardés ; un sous-compte de frais par compte.
- *Périmètre* : migration **breaking** (E19), `BankAccountKind::PaymentProvider` et décodage tolérant,
  `admits_ledger_type` (actif seul), levée de `ACCOUNT_IN_FOREIGN_CURRENCY` pour ce type (C-37-9), gardes de devise
  (C-38-3), `fee_account_id`, routes `bank-accounts` (création, `PUT`, `PATCH`), relevé QR et lots de paiement qui
  **excluent** ce type (aucun IBAN), exports.
- *Risques* : **chaque lecture de `account_kind`** (QR-facture `invoice_pdf_service.rs`, empreinte d'import
  `bank_import_fingerprint.rs`, assistant `onboarding.rs`, règlements `invoice_settlements_write.rs` : inventaire au
  `grep -rln BankAccountKind`) ; verrou de devise du compte du plan (37-1) contre lien concurrent ; recette de montée
  depuis l'image publiée de l'Epic 37 (refus du binaire précédent, retour par la sauvegarde) ; P2-bis.

### 38-3 — Téléchargement : cadre commun

- *Intention* : Kesh télécharge les mouvements d'une connexion, chaque jour ou à la demande, sans doublon, depuis la
  date de départ, et liste les soldes sans compte.
- *Périmètre* : crate `kesh-providers` (types, trait `Connector` objet-safe, validation), colonnes de
  `bank_transactions` (C-38-4), `provider_balances_seen`, passage (C-38-5, E5, E6, E10, E12, E20), route
  `POST /provider-connections/{id}/fetch`, transport factice en `KESH_TEST_MODE` + route `_test`, connecteur factice
  pour les tests.
- *Risques* : statut d'une transaction qui change après coup (en attente → réussie, inversée) ; passage concurrent
  (verrou) ; reprise après échec au milieu d'une fenêtre ; réconciliation d'une transaction non encore classée ;
  volume (trois ans × trois soldes) ; heure locale contre UTC des dates du prestataire (date comptable = date en
  Europe/Zurich, à écrire).

### 38-4 — Critères neufs des règles d'affectation

- *Intention* : une règle reconnaît une transaction de prestataire par courriel, préfixe de facture, nom d'article ou
  code d'événement, et se restreint par sens et par compte.
- *Périmètre* : E14 ; `rule_matches` étendu (`rules.rs:46`) ; unicité refaite ; garde de devise du compte de la
  règle ; routes `reconciliation_rules` (champs facultatifs, rétrocompatibles).
- *Risques* : les règles existantes ne changent pas de comportement (tests de non-régression des quatre critères) ;
  normalisation du courriel (casse, espaces) ; priorité entre règles larges et étroites (inchangée : `priority`).

### 38-5 — Code TVA par défaut d'un compte

- *Intention* : un compte de produit ou de charge porte son genre et son taux de TVA par défaut, que les propositions
  appliquent (B6) et que la saisie manuelle pourra reprendre.
- *Périmètre* : E13 ; colonnes `accounts`, champ du formulaire de compte, fonctions pures et miroir TS, gardes (taux
  actif à la date, compte de résultat seulement).
- *Risques* : arrondi de l'extraction (centime) ; taux changé en cours d'exercice (taux en vigueur à la date de la
  transaction, par `valid_from`/`valid_to`) ; un compte en devise : TVA en devise principale (S5).

### 38-6a — Propositions de prestataire : ventes, achats, frais

- *Intention* : chaque vente ou achat téléchargé reçoit une proposition complète, acceptée en lot.
- *Périmètre* : E15 ; constructeur pur multi-lignes dans `kesh-reconciliation` (à côté de `manual.rs:68`, qui reste
  à deux lignes) ; branchement dans `get_proposals` (`reconciliation.rs:999`) et `post_accept` (`:1593`), variante
  `provider` ; frais seuls (`movement_kind = fee`) au sous-compte.
- *Risques* : **`reconciliation.rs` fait 7 439 lignes** : le code neuf va dans un module à part, appelé depuis le
  répartiteur ; équilibre en devise principale à plusieurs lignes en devise (arrondis, 37-3a) ; exercice mono-devise ;
  compte de la règle archivé entre proposition et acceptation (revalidation sous verrou, précédent `accept_one_rule`).

### 38-6b — Conversions, retraits, remboursements, litiges ; file « à examiner »

- *Intention* : les mouvements qui ne sont ni vente ni achat se proposent juste, ou attendent un humain.
- *Périmètre* : E16 ; paire de conversion vers `POST /journal-entries/conversions` (A31) ; rattachement des retraits
  (34-11b) ; inverse d'une écriture d'origine ; file (C-38-9).
- *Risques* : paire de conversion dont une moitié manque (passage interrompu) ; remboursement partiel et TVA au
  prorata ; litige résolu en faveur du vendeur (deux mouvements qui s'annulent) ; annulation d'une réconciliation de
  paire (les deux côtés).

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
  chaque code relevé reçoit un test.
- *Risques* : un code inconnu ne se classe jamais en vente (→ `other`) ; signe des frais ; regroupement des deux
  moitiés d'une conversion.

### 38-9a et 38-9b — Écrans

- *38-9a* : liste et fiche des connexions (secrets en saisie seule, « Tester », « Télécharger maintenant », état du
  dernier passage), soldes proposés (créer ou lier un compte, B2), compte bancaire prestataire et sous-compte de frais,
  contrôle de solde (E20) ; manuel d'administration (clé, permissions PayPal, recette).
- *38-9b* : critères neufs au formulaire des règles ; propositions multi-lignes (aperçu des lignes, TVA, frais,
  cours) ; acceptation en lot ; file « à examiner » ; manuel utilisateur ; E2E (contre le transport factice) et
  captures d'écran (*Recette visuelle*).
- *Risques* : un secret ne revient jamais dans le DOM ; textes sans « automatique » trompeur (proposer, non
  comptabiliser).

### 38-10 — Relevé joint à la main

- *Intention* : le relevé annuel PayPal (PDF, sans API) se range au compte et à l'exercice.
- *Périmètre* : E17, C-38-10 ; écran du compte bancaire (onglet *Relevés*) ; sauvegarde complète.
- *Risques* : fichier qui n'est pas un PDF ; taille ; suppression d'un exercice ou d'un compte (refus tant qu'une
  pièce y est rangée, ou suppression de la pièce : à trancher à la fiche).

## Ce que réutiliseront les Epics 39 (Wise) et 40 (Stripe)

**Réutilisé tel quel** : coffre des secrets et connexions (38-1), compte prestataire et sous-compte de frais (38-2),
cadre de téléchargement, doublon, planification, soldes vus (38-3), règles étendues (38-4), TVA par défaut (38-5),
propositions et file (38-6a, 38-6b), écrans génériques (38-9a, 38-9b), pièces de compte (38-10). Aucune migration
breaking attendue (E4, E5).

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

Beat ne dépose pas de décompte par Kesh cette année et passe la TVA par l'API et ces imports seulement (B6, B15) :
l'epic **sépare** la TVA dans les écritures, il **ne la déclare pas**. La décision de lever ou de maintenir le report
des P1 TVA revient à Guy.

## Questions ouvertes

- **Q1 — Version minimale** : numéro de la release qui portera l'Epic 38, pour le relèvement de
  `kesh_version_min_required` de 38-2 (E19) — par Guy, comme pour l'Epic 37.
- **Q2 — Exercice 2024 déjà importé** : les livres 2024 de Beat sont importés par l'API (Epic 37). Avec
  `fetch_from = 1.1.2024` (B8), les transactions 2024 trouveront des écritures déjà passées : les **lier en lot** aux
  écritures existantes (extension de 34-4, `reconciliation.rs:6656`) ou partir du 1.1.2025 ? À trancher par Beat.
- **Q3 — Impôt sur les acquisitions** : un compte de TVA due distinct (p. ex. 2201) aux réglages, ou le compte de TVA
  due existant (`default_vat_payable_account_id`) ? Et le compte de produit en devise ou en CHF pour les ventes en USD
  (les deux admis par 37-3a) : choix de Beat dans son plan.
- **Q4 — Secrets et sauvegarde** (E2) : effacer les secrets à toute restauration convient-il à Beat (ressaisie après
  une restauration), ou faut-il les emporter chiffrés quand la clé est la même ?
- **Q5 — Rotation de `KESH_SECRETS_KEY`** : hors périmètre ici (C-38-13) ; acceptable ?
- **Q6 — Frais d'un remboursement** : PayPal rend en partie ses frais lors d'un remboursement : ligne de frais
  inverse au prorata, ou montant rendu par PayPal tel quel (proposé ici : tel quel) ?
- **Q7 — Code d'événement et sens suffisent-ils** (E14), ou Beat a-t-il des règles qui combinent deux critères
  principaux (p. ex. courriel **et** nom d'article) ? Sinon, une règle par paire.
- **Q8 — MCP** : des outils de lecture (connexions, file « à examiner ») sont-ils voulus dans cet epic ?
- **Q9 — Numéro 40 pour Stripe** : à confirmer par Guy (cadrage).

## Règles de cet epic

- Méthode du dépôt (`CLAUDE.md`) : fiche → validation en boucle → développement → revue en boucle (Sonnet ↔ Opus pour
  les passes complètes, passe ciblée en fin de boucle), gate local complet avant la PR ; splitting préventif (≤ 5
  modules par story).
- **Production** : l'instance de Beat n'est jamais écrite par le travail de cet epic. Recette sur ses comptes PayPal
  réels, avec une application REST en lecture seule (*Transaction Search*), sur une **instance jetable** (B12).
- Aucun appel réseau en CI ; aucune réponse réelle non anonymisée dans le dépôt.
- On trace toujours : un défaut voisin non traité se note au registre (C-38-13), pour devenir une issue avec la PR.
