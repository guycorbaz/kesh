# Epic 38 — Comptes PayPal : téléchargement et imputation automatiques

*Depuis le 2026-11-20 : **phase 1 KISS** — import du CSV PayPal à la demande et rapprochement (§ *Phase 1 KISS*, qui prime) ; le reste passe en phase 2.*

**Cadrage : 2026-11-15** (branche `epic/38-paypal`), **révisé le même jour sur B16–B21** (§ *Révision B16–B21*)
**puis sur B22–B26** (§ *Révision B22–B26*), arbitré O1–O12 après la rédaction des fiches et aligné sur la validation
P1 (§ *Validation P1*, découpage de 38-3 en 38-3a et 38-3b), puis révisé sur B27–B28 (§ *Révision B27–B28*). Demande
de Beat (contributeur), décisions B1–B28 de
`epics-38-40-cadrage.md`, qui **priment** sur ce fichier là où il diverge (B22 remplace B16 sur les domaines fermés).
Kickoff (décomptes d'issues et de bugs, validation de ce fichier) **à tenir** : `38-0-kickoff` reste au backlog tant
que l'Epic 37 n'est pas développé.

**Une seule migration breaking** (B22, placée en **38-2** par O1) : la contrainte d'IBAN d'un compte `bank` remplacée
(un compte prestataire n'a pas d'IBAN), et `reconciliation_rules.match_type` passé d'`ENUM` à `VARCHAR` pour les
critères neufs ; `kesh_version_min_required` relevé **en dernière instruction**, les dix crates bumpés **dans le même
commit** (P2, P2-bis) — **numéro de release à Guy** (Q1). `source_format = 'PROVIDER_API'` n'exige aucun DDL et
s'écrit à partir de 38-3b, sous ce même relèvement. Toutes les autres migrations de l'epic créent des tables ou
ajoutent des colonnes `NULL`. `account_kind` ne reçoit toujours aucune valeur neuve (B16 maintenu). Détail en E19.
*(Reportée en phase 2 (2026-11-20) — voir § *Phase 1 KISS*.)*

**Numéro : 38** (35 et 36 réservés ailleurs ; 39 Wise, **40 Stripe**, provisoire en attendant Guy, B25). Une branche par
story, fusionnée localement dans `epic/38-paypal`, puis **une PR** vers le dépôt de Guy (remote `beat`), **après celle
de l'Epic 37**. Pas d'issue GitHub à ce stade (Beat n'a qu'un accès en lecture) : le détail vit ici et dans les fiches ;
issues et PR s'ouvrent à la fin.

**Cible** : la release qui suivra celle de l'Epic 37 (numéro, CHANGELOG daté, macros des manuels, tag : par Guy).

**Release** : un epic = une release ; **16 stories**. Cet epic **dépasse la borne** de six stories des epics de
correction ou d'amélioration (c'est une fonctionnalité nouvelle, ce que le kickoff déclare) ; **son découpage en
plusieurs epics est à décider avec le Project Lead à la soumission** (décision de Beat, 2026-11-16).
*(Reportée en phase 2 (2026-11-20) — voir § *Phase 1 KISS*.)*

**Règles de travail à plusieurs** (alignement du 2026-11-16, `CLAUDE.md` § *Travailler à plusieurs développeurs*) :
aucune ligne n'est écrite au `CHANGELOG.md` en cours d'epic (chaque fiche porte sa section *Note de version*, assemblée
à la release) ; les PDF des manuels ne sont jamais commités dans la branche (le `.tex` seul change, les PDF se
régénèrent à la release) ; les compteurs (`LIB_ROUTES`, audit des migrations, `sitesTotal`, `data_count`,
`EMPTY_BEFORE_RESET`, `Cargo.lock`, nombre de crates, nombre de contrôles de clôture) se recomptent depuis la source
après chaque reprise de `main`.

## Phase 1 KISS (décision de Beat du 2026-11-20)

⛔ **Cette section prime sur tout le reste du fichier.** Ce qui suit à partir de « Modules touchés » est
l'**historique** de la phase de cadrage du 2026-11-15 au 2026-11-17 ; les décisions qui ne valent plus y sont marquées
« reportée en phase 2 (2026-11-20) ». Source : la page approuvée par Beat (`data-not-committed/epic-38-kiss.html`,
version 2 du 2026-11-20, hors dépôt) et les notes sans donnée personnelle
`kesh-gate-logs/epic-38-kiss/paypal-2024-comprendre.md`.

**Pourquoi.** Beat doit rendre sa comptabilité 2025 le **dimanche 2026-11-23**. L'epic de seize stories (coffre des
secrets, client de l'API PayPal, téléchargement planifié, TVA par défaut, groupes d'écritures, relevé joint) ne tient
pas dans ce délai. La phase 1 garde le chemin le plus court : **un import CSV à la demande, un rapprochement par
identifiant de transaction, la liste des écarts**, développable en une nuit.

### Périmètre

- **Import unique, à la demande** (bouton à l'écran et route de l'API), du CSV PayPal « Balance affecting
  transactions » d'un exercice. Aucun automatisme, aucun appel réseau.
- **Deux comptes PayPal**, plusieurs devises : **un compte bancaire Kesh par compte PayPal et par devise**, lié au
  compte du plan de la devise (comptes en devise de l'Epic 37) — levée de C-37-9 pour ce seul cas (engagement D8 de
  l'Epic 37).
- **Réglages par compte bancaire** : le compte PayPal (libellé) et le **compte de frais**.
- **Les ventes viennent des boutiques** : écritures déjà passées (débit du compte PayPal de la devise, crédit des
  ventes 3132 ou 3140, « TXN <id> » ou « transaction <id> » dans le libellé, montant brut). Un encaissement PayPal se
  **rattache** à l'écriture de vente de même identifiant, même montant brut et même devise — par le rattachement à une
  écriture existante (v017-3a, `find_entry_candidates` / `link_entry_in_tx`, `reconciliation_link.rs:870`, `:1156`),
  étendu aux transactions en devise. **Seuls les frais s'écrivent**, au compte de frais.
- **Écarts listés** : encaissement sans vente ; vente sur le compte PayPal sans encaissement ; montant ou devise
  différents.
- **Achats par règle** : règles d'affectation existantes, plus deux critères neufs — **type PayPal** (exact) et
  **e-mail exact** — et un **sens** facultatif.
- **Conversion** : même montant CHF des deux côtés (Q3). Kesh n'admet qu'une devise étrangère par écriture
  (`FX_MULTIPLE_CURRENCIES`, 37-3a) : une conversion EUR → USD s'écrit en **deux écritures** liées, par le compte de
  conversion, au même CHF — la forme de 2024 (compte de transit 9901), et celle de `create_conversion` (37-3b).
- **Retrait** : virement vers le compte choisi (le compte bancaire principal par défaut).
- **Doublons** (même identifiant de transaction sur le même compte) ignorés ; **Status ≠ Completed** écarté ; lignes
  `Memo` et paires de retenue neutres écartées ; **le reste « à examiner »** (remboursements, litiges, conversions
  sans paire, lignes sans règle).
- **Aucun code TVA** (Q4 : aucun au grand livre 2024 sur ces lignes ; la TVA ne touche que les ventes « CH » des
  boutiques, par leur compte de vente).
- **Ventes 2025 des boutiques** : importées hors Kesh par un script sur l'API existante (`POST
  /api/v1/journal-entries`, R1, recommandé) ; l'import des boutiques dans Kesh est en phase 2.

### Règles

- Kesh **propose**, l'utilisateur **accepte** (règle « un appariement automatique propose, il ne crée jamais ») ;
  rien n'est écrit à la lecture de la revue ; l'acceptation suit le *Pattern batch*.
- Une ligne PayPal avec frais donne **deux transactions** (brut et frais, `transaction_id` suffixé `:FEE`) : le brut
  se rattache à la vente, les frais s'écrivent.
- Le CSV est lu **par les noms de colonnes** (anglais) ; une autre langue est refusée en nommant les colonnes
  manquantes.
- Les dates sont celles de l'export, sans conversion de fuseau (limite au manuel).
- Méthode du dépôt inchangée (fiche, validation, développement, revue en boucle, gate complet, E2E au dernier commit
  de code) ; elle se tient en une nuit parce que les fiches sont courtes, non parce qu'on la saute.

### Stories de la phase 1

| Clé | Objet | Modules (≤ 5) | Dépend de | Migration |
|---|---|---|---|---|
| `38-11a` | lecteur du CSV PayPal | `kesh-import` | — | non |
| `38-11b` | réglages du compte PayPal et import ventilé par devise, doublons | `kesh-db`, `kesh-api`, `kesh-import` (lecture), `kesh-i18n` | 38-11a, Epic 37 | **oui, la seule** |
| `38-11c` | rapprochement : ventes par TXN, frais, règles type et e-mail, conversions, retraits, écarts, « à examiner » | `kesh-reconciliation`, `kesh-db`, `kesh-api`, `kesh-i18n` | 38-11b | non |
| `38-11d` | écrans, manuel utilisateur (`.tex`), `docs/api-external.md`, E2E | `frontend`, `kesh-i18n`, manuel, docs, E2E | 38-11b, 38-11c | non |

Quatre stories, story de solde non prévue (la dette tracée en issues à la PR). Les clés `38-11a` à `38-11d` évitent
toute collision avec les fiches 38-1 à 38-10, gardées comme historique et passées à `deferred` au registre.

### Migration

**Une**, `20261120000001_paypal_import.sql` (38-11b), **non breaking** au sens du ⚠️ de P1 :
`kesh_version_min_required` reste à 0.17.6, aucune version Cargo relevée. Horodatage après les dernières de `main`
(`20261118000001_grouped_settlements`), de l'Epic 49 (`20261118000002`) et de l'Epic 47 (`20261119000001`). Contenu :
`bank_accounts` (`provider`, `provider_account`, `fee_account_id`, contrainte d'IBAN reposée sous le même nom pour
admettre un compte PayPal sans IBAN) ; `bank_transactions` (`provider_type`, `counterparty_email`, un index) ;
`reconciliation_rules` (deux valeurs **ajoutées en fin** de l'`ENUM` `match_type`, sous `-- breaking-skip-bump:`
justifié — P4 —, et `direction`). DDL seul (P7 sans objet) ; P5, P6, P8 à la fiche. ⚠️ **Point à confirmer à la
revue** : l'ajout de valeurs à l'`ENUM` est un `MODIFY COLUMN`, que P3 classe CRITICAL sans relèvement ; la
justification P4 est celle de `20261118000001` (aucun binaire antérieur ne lit la base migrée ni une sauvegarde à
colonnes neuves). Si la revue la refuse, l'alternative est deux colonnes de filtre `NULL` sur `reconciliation_rules`
(sans nouveau type), au prix d'un critère principal obligatoire.

### Fichiers communs (travail à plusieurs)

| Fichier ou module | Touché par | Recoupe |
|---|---|---|
| `crates/kesh-db/migrations/`, `test-schema/0001_schema_squash.sql`, `docs/migrations-idempotence-audit.md` | 38-11b | Epics 47, 49 (une migration chacun) : compteurs **recomptés** après chaque fusion |
| `crates/kesh-db/src/backup.rs` (`ABSENCES_ADMISES_EPIC_37`) | 38-11b | Epics 47, 49 s'ils ajoutent des colonnes |
| `crates/kesh-api/src/exports/csv_tables.rs` | 38-11b | Epics 47, 49 |
| `crates/kesh-api/src/routes/bank_accounts.rs` | 38-11b | — |
| `crates/kesh-api/src/routes/reconciliation.rs`, `crates/kesh-db/src/repositories/reconciliation_link.rs` | 38-11c | Epic 48 (corrections) à surveiller |
| `crates/kesh-reconciliation/src/rules.rs`, `routes/reconciliation_rules.rs` | 38-11c | — |
| `crates/kesh-api/src/errors.rs`, catalogues i18n (quatre locales), `i18n-keys.test.ts` (`sitesTotal`) | toutes | tous : chacun son bloc, compteur recompté |
| `frontend/src/routes/(app)/bank-accounts/`, `features/reconciliation/rules/` | 38-11d | `v0200-menu-sous-groupes` (menu) |
| `docs/manual/fr/user-manual.tex`, `docs/api-external.md` | 38-11d | Epics 47, 49 ; PDF jamais commités dans la branche |
| `sprint-status.yaml` | toutes | bloc propre de l'Epic 38 |

### Dépendances

- **Epic 37** (dans `main` depuis la 0.19.0, `70be12a6`) : devise d'un compte, cours retenus
  (`find_company_rate_in_tx`, `exchange_rates.rs:365`), lignes en devise, `create_conversion` (37-3b).
- **Base** : la branche reprend `main` (fusion du 2026-11-20, `43ceb541`) ; l'Epic 37 y est.
- **Release** : à convenir avec le Project Lead (un epic = une release). Pour ses livres 2025, Beat ne fait tourner un
  build de la branche sur ses données réelles qu'en connaissance de cause : la migration y devient figée (P8), et son
  horodatage doit rester après toute migration de `main` publiée ensuite.

### Ce qui passe en phase 2

Coffre des secrets et connexions (38-1) ; client et téléchargement par l'API PayPal, planifié et à la demande (38-3a,
38-3b, 38-7) ; codes d'événement (38-8) ; boutique par préfixe du numéro de facture, critères article et code
d'événement, filtres devise et compte (le reste de 38-4) ; code TVA par défaut d'un compte (38-5) ; écritures de vente,
de remboursement et de litige proposées (le reste de 38-6a et 38-6b) ; rattachement en lot et groupe d'écritures
(38-6c1, 38-6c2) ; écrans des connexions (38-9a) ; relevé joint et contrôle de clôture (38-10) ; import des boutiques
dans Kesh ; Stripe (Epic 40). La migration breaking de l'ancienne 38-2 (O1) et la question Q1 qui en dépendait
tombent avec elle.

## Modules touchés

*(Reportée en phase 2 (2026-11-20) — voir § *Phase 1 KISS*.)*

*(Liste tirée des fiches ; elle se relit à chaque reprise de `main`, pour voir tôt les recouvrements avec les autres
epics en cours.)*

| Fiche | Crates | Frontend | Manuels et docs |
|---|---|---|---|
| 38-1 | `kesh-db` (migration), `kesh-core`, `kesh-api` (config, secrets, routes) | — | manuel d'administration (`KESH_SECRETS_KEY`), `docs/testing.md`, `CLAUDE.md` (bloc E2E) |
| 38-2 | `kesh-db` (migration breaking), `kesh-api`, `kesh-core` | — | manuel d'administration (une phrase) |
| 38-3a | `kesh-db` (migration), `kesh-core`, `kesh-api`, `kesh-i18n` | — | — (faits transmis à 38-9a) |
| 38-3b | `kesh-providers` (neuf), `kesh-api`, `kesh-db`, `kesh-i18n` | — | — (faits transmis à 38-9a) |
| 38-4 | `kesh-db` (migration), `kesh-reconciliation`, `kesh-api`, `kesh-i18n` | libellés | manuel utilisateur (relu, 38-9b écrit) |
| 38-5 | `kesh-db` (migration), `kesh-core`, `kesh-api` | accounts, réglages | manuel utilisateur |
| 38-6a | `kesh-reconciliation`, `kesh-api`, `kesh-db`, `kesh-i18n` | — | `docs/api-external.md` |
| 38-6b | `kesh-reconciliation`, `kesh-api`, `kesh-db`, `kesh-i18n` | — | — |
| 38-6c1 | `kesh-db`, `kesh-api`, `kesh-i18n` | — | `docs/api-external.md` |
| 38-6c2 | `kesh-db` (migration, sauvegarde), `kesh-api`, `kesh-i18n` | — | `docs/api-external.md` |
| 38-7 | `kesh-providers` (paypal), `kesh-api` | — | — |
| 38-8 | `kesh-providers` (paypal), `kesh-core` | — | — |
| 38-9a | — | provider-connections (neuf), bank-accounts, i18n | manuel d'administration, manuel utilisateur |
| 38-9b | — | reconciliation, règles, i18n, E2E | manuel utilisateur |
| 38-10 | `kesh-db` (migration), `kesh-api` (routes, sauvegarde, closing-checks) | bank-accounts, fiscal-years | manuels FR, brochure, `README.md`, `docs/api-external.md` |

**Dépendance à l'Epic 36 (38-6c2)** : à revoir après la fusion de l'Epic 36 — Guy construit #823 (36-6, une
transaction rattachée à plusieurs écritures) ; réutiliser son mécanisme plutôt que doubler la table
`bank_transaction_extra_entries`.

**Contrôles de clôture** : l'Epic 37, qui sort avant, prend K9 (« comptes en devise non réévalués », bloquant) ; le
relevé joint (38-10) est **K10**, et l'avant-clôture compte dix contrôles (décision du 2026-11-16).

## Intention

*(Reportée en phase 2 (2026-11-20) — voir § *Phase 1 KISS*.)*

Un compte PayPal *business* se tient dans Kesh **sans saisie** : Kesh télécharge chaque jour (ou à la demande) les
mouvements de chaque solde de devise, les classe (vente, achat, frais, conversion, remboursement, litige, retrait), et
les **propose** à l'imputation — brut au compte de produit ou de charge désigné par une **règle d'affectation**, TVA
séparée selon le **code TVA par défaut du compte**, frais au **sous-compte de frais** du prestataire et de la devise —
pour une acceptation en lot. Rien n'est écrit d'office. Le relevé annuel PayPal (PDF, sans API) se joint à la main au
compte et à l'exercice.

L'epic pose d'abord le **socle commun** (B14) que réutiliseront Wise (39) et Stripe (40), puis le connecteur PayPal,
les écrans, et le relevé joint.

## Dépendance à l'Epic 37 — et l'engagement D8

*(Historique, 2026-11-15. L'Epic 37 est dans `main` depuis la 0.19.0 ; la levée de C-37-9 (D8) est reprise par 38-11b. Numéros de ligne périmés : ceux des fiches 38-11a à 38-11d, relevés le 2026-11-20, font foi.)*

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

*(Historique, 2026-11-15. L'Epic 37 est dans `main` depuis la 0.19.0 ; la levée de C-37-9 (D8) est reprise par 38-11b. Numéros de ligne périmés : ceux des fiches 38-11a à 38-11d, relevés le 2026-11-20, font foi.)*

| Domaine | Ce qui existe | Où |
|---|---|---|
| Comptes bancaires | `account_kind` `bank` \| `credit_card`, domaine fermé à l'octet, `iban` `NULL` mais **exigé d'un `bank`** (`chk_bank_accounts_bank_has_iban`, `:48-49` ; `validate_bank_input`, `routes/bank_accounts.rs:557-570`) — seule la carte en est dispensée ; `card_last4` ; migration **breaking** (min 0.17.6) | `migrations/20261110000001_bank_accounts_without_iban.sql:39-55` |
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

*(Reportée en phase 2 (2026-11-20) — voir § *Phase 1 KISS*.)*

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
  reçoivent des valeurs neuves, **sous un seul relèvement**, celui de la migration breaking de **38-2** (O1, E19) :
  `bank_imports.source_format` (`'PROVIDER_API'`, sans DDL) et `reconciliation_rules.match_type` (critères neufs, E14 ;
  `MODIFY COLUMN` en 38-2). Le décodage tolérant arrive avec la **première écriture** d'une valeur neuve :
  **`BankImportSourceFormat`** en **38-3a**, **`ReconciliationMatchType`** en **38-4** — chacun gagne une variante
  `Other(String)` (le `FromStr` cesse de refuser l'inconnu ; `Copy` perdu, sites suivis aux fiches) : une
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
  `provider_fetch_run_id`). `BankImportSourceFormat::ProviderApi` neuf, décodage tolérant (E4, 38-3a) ; la colonne est un
  `VARCHAR(32)` sans `CHECK` (`20260504000001_bank_imports.sql:25`) : **aucun DDL**, seul le relèvement de
  `kesh_version_min_required` (38-2) protège un binaire antérieur, dont le `FromStr` refuse l'inconnu (précédent `MANUAL`,
  `20261030000001`). Affichage « Téléchargé de PayPal (connexion, passage) » — liste et détail des imports, boîte de
  suppression, export CSV de `bank_imports` (colonne `provider_fetch_run_id` ajoutée, `csv_tables.rs:702-720`),
  audit, libellé du frontend (`bank-import-display.ts:14-36`). Aucun comportement ne dépend de `source_format`
  (lecteurs relevés à l'inventaire : affichage seul). La suppression d'un tel en-tête par `DELETE /bank-imports/{id}`
  est **refusée** (`PROVIDER_IMPORT_NOT_DELETABLE`) : le recouvrement (`overlap()`, trois jours par défaut) ne
  retéléchargerait pas ce qu'elle effacerait. *Raison* : DRY (la réconciliation lit `bank_transactions` et rien d'autre) ; le détour par `MANUAL`
  (valeur inexacte, ex-Q11) est retiré (B22).
- **E6 — Doublon par l'identifiant du prestataire.** Colonne neuve `provider_tx_id`, `UNIQUE (bank_account_id,
  provider_tx_id)` (NULL multiples admis : CAMT, CSV et manuel inchangés). Insertion idempotente, **connu ou inséré
  décidé par une lecture préalable des identifiants connus** — jamais par `rows_affected`, que `CLIENT_FOUND_ROWS`
  (posé par sqlx) rend égal à 1 dans les deux cas — puis `INSERT` des seuls absents (`ON DUPLICATE KEY` sans écrasement
  en filet) ; chaque passage **recouvre** les derniers jours (`Connector::overlap()` : `overlapDays` des réglages, 1 à
  30, **défaut 3** — délai de 3 h de PayPal, statuts tardifs, O11). La clé de doublon par fenêtre (`kesh-core/src/bank_imports.rs:295`) ne s'applique pas.
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
  `account_kind = 'bank'`, sans IBAN, sans QR-IBAN ni carte. ⚠️ *Corrigé (O1)* : un `bank` sans IBAN **n'est pas**
  admis depuis 34-11a1 — seule la carte l'est (`chk_bank_accounts_bank_has_iban`) ; la migration breaking de 38-2
  remplace cette contrainte par `chk_bank_accounts_primary_has_iban` (le principal garde un IBAN), et l'IBAN d'un `bank`
  ordinaire reste exigé par `validate_bank_input`. Son caractère « prestataire » vit
  dans la table applicative neuve **`payment_provider_accounts`** : `bank_account_id` (PK, FK), `connection_id` (FK
  `payment_provider_connections`), `currency` (devise du solde), `fee_account_id` (compte de frais **de cette devise**,
  B3), `version` ; colonnes `NULL` à venir pour d'autres comptes liés (Epics 39, 40) ; `UNIQUE (connection_id,
  currency)`. **Toute référence à un compte est une colonne FK `ON DELETE RESTRICT`**, gardée à l'archivage du compte
  visé (un compte de frais ou lié ne s'archive pas tant qu'un réglage le désigne) — jamais une clé du JSON (B23,
  E24). Gardes, au serveur : compte du plan lié **actif, de bilan (actif)**, **dans la devise du solde** — CHF
  admis pour le solde CHF ; `fee_account_id` de charge, actif, dans la même devise (`FEE_ACCOUNT_INVALID`) ; le
  regroupement des frais est son **compte parent** (`accounts.parent_id`), aucun réglage de plus ; jamais principal
  (`is_primary` refusé, la QR-facture lit le compte principal, `invoice_pdf_service.rs:332`). La ligne de réglages et
  le lien au compte du plan en devise se posent **dans la même transaction** ; c'est elle qui lève C-37-9 (D8). Le
  compte du plan d'un compte prestataire, en devise **ou en CHF**, n'est lié à aucun autre compte bancaire, dans les
  deux sens (`PROVIDER_ACCOUNT_LEDGER_SHARED`, O8 étendu, D-P2-4). Retirer
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
  d'`ENUM` à **`VARCHAR(40) NOT NULL`** `ascii_bin`, `CHECK` de format seul (`^[a-z][a-z_]{1,39}$`, O2), dans la
  migration breaking de **38-2** (O1, E19), valeurs existantes conservées ; les valeurs admises sont tenues par l'API à l'écriture (aujourd'hui par la désérialisation de l'enum en 400 ;
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
  - **Conversion** : la paire (E7) entre **deux devises étrangères** se propose comme **une conversion** de 37-3b,
    `completeDifference: true` (B4, A31), cours AFC des deux côtés, différence aux comptes propres du compte en
    devise (S2) ; les deux transactions sont réconciliées ensemble. Coût donné à part par le prestataire → ligne de
    frais au compte de frais de la devise de la moitié qui le porte, **moitié créditée comprise** (B27 : ligne au
    débit de l'écriture complétée, admise par A33 de l'Epic 37, différence calculée **après** elle — jamais dans la
    différence de change). **Paire dont un côté est en CHF** (la plus courante, C-38-6b-2 bis) : **une seule écriture** ordinaire —
    débit du compte qui reçoit, crédit du compte qui paie, une seule devise étrangère, frais de chaque moitié à son compte de frais (B27), ligne
    de différence aux comptes de change du compte en devise —, sans compte de transit ni fonction de 37-3b (qui ne vaut
    qu'entre deux devises étrangères) ; les deux transactions sont réconciliées sur cette écriture (34-11b).
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
- **E19 — Migrations : une seule breaking, dans 38-2 (B22, O1).** *Où et pourquoi là* : 38-2 est la **première**
  story qui en a besoin — un compte prestataire `bank` sans IBAN viole `chk_bank_accounts_bank_has_iban`
  (`20261110000001_bank_accounts_without_iban.sql:48-49`), que seul un `DROP CONSTRAINT` lève. Le passage de
  `match_type` en `VARCHAR` et l'écriture de `PROVIDER_API` (sans DDL, à partir de 38-3b) partagent **ce** relèvement :
  **une** migration breaking, **un** relèvement ; 38-3a et 38-4 n'ont que des migrations non breaking. *(La rédaction
  plaçait la breaking en 38-3 ; O1 l'a déplacée.)*
  - **Fichier** de 38-2 (`<date>000002_payment_provider_accounts.sql`), en tête `-- BREAKING (P1, P3 : MODIFY COLUMN) :
    relève kesh_version_min_required à <X.Y.Z>` (patron `20261110000001`). Contenu : (1) `CREATE TABLE
    payment_provider_accounts` ; (2) `DROP CONSTRAINT chk_bank_accounts_bank_has_iban, ADD CONSTRAINT
    chk_bank_accounts_primary_has_iban` ; (3) `ALTER TABLE reconciliation_rules MODIFY COLUMN match_type VARCHAR(40)
    CHARACTER SET ascii COLLATE ascii_bin NOT NULL` + `CHECK` de format `^[a-z][a-z_]{1,39}$` (O2) — les quatre
    valeurs existantes restent, aucune ligne réécrite ; l'index unique se reconstruit à l'`ALTER`, sans `DROP INDEX` ;
    (4) **en dernière instruction** `UPDATE _kesh_version SET kesh_version_min_required = '<X.Y.Z>' WHERE id = 1;` (P2).
    **Aucun DDL** pour `source_format` (`VARCHAR(32)` sans `CHECK`).
  - **P2-bis** : les dix crates passent à `<X.Y.Z>` **dans le même commit** que la migration (`kesh-providers` naît
    après, en 38-3b, à la version courante : onze crates ensuite) ; gate **runtime complet** (`admin_backup_e2e`,
    `admin_full_import_e2e`, `migrations_fresh_install`) avant `done`. **`<X.Y.Z>` est à Guy (Q1)** : la fiche 38-2 ne
    se développe pas sans lui ; s'il change après coup, la migration, non publiée, se corrige délibérément (P8 : bases
    de dev reconstruites). Les assertions de la version minimale courante (`0.17.6`) se trient (38-2 AC-1).
  - **P3/P4** : pas de `breaking-skip-bump` (un binaire antérieur ne décode ni `'PROVIDER_API'` ni les critères neufs :
    `FromStr` refusant l'inconnu, `entities/bank_import.rs:59`, `entities/reconciliation_rule.rs:68` ; et il garderait
    l'ancienne garantie d'IBAN).
  - **P5** : une ligne à `docs/migrations-idempotence-audit.md` par fichier, compteurs recomptés à la source.
  - **P6** : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` à chaque fiche qui ajoute une migration.
  - **P7** : l'`UPDATE _kesh_version` écrit une table système jamais restaurée → `EXEMPT_MIGRATIONS`, justification
    écrite (précédent `20261030000001`) ; les autres migrations de l'epic n'écrivent pas de données.
  - **P8** : aucune migration appliquée ne se retouche.
  - **Décodage tolérant ensuite (E4)** : `BankImportSourceFormat` gagne `Other(String)` en 38-3a,
    `ReconciliationMatchType` en 38-4 (première écriture de chaque valeur neuve) ; la migration de 38-2 ne change que
    le DDL. Wise (39) et Stripe (40) n'ont plus besoin de relever `kesh_version_min_required` pour ces deux colonnes.
  - **Retour** : un binaire antérieur refuse de démarrer sur la base migrée et d'importer un `.keshbackup` de cette
    version (version minimale) ; **le retour à la version précédente passe uniquement par la sauvegarde d'avant la mise
    à jour** (`backup/pre-upgrade/`, #847, et celle que les notes exigent), à écrire en tête des notes de version et au
    manuel d'administration (38-9a). Le SQL de retour de la révision B16–B21 est **retiré**. Recette de montée
    (`scripts/recette-montee.sh`) depuis l'image publiée précédente, jouée en 38-2 (étendue par 38-3a) et à la
    release.
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
  de l'exercice (`GET /fiscal-years/{id}/closing-checks`, avertissement non bloquant) — **K10** (K9 est celui de
  l'Epic 37, décision du 2026-11-16).
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
  deux comptes en devise) se rattache des deux côtés comme un virement entre ses comptes (34-11b). **(3) À plusieurs
  écritures (B28)** : les frais PayPal de 2024 sont des écritures à part ; une transaction (net) se rattache à un
  **groupe** de 2 à 5 écritures existantes dont les mouvements sur le compte prestataire somment exactement à son
  montant (vente au brut + écriture de frais), à l'unité et en lot (`journalEntryIds`) ; `matched_entry_id` garde
  l'écriture principale, les autres vont dans une table neuve `bank_transaction_extra_entries` (**migration non
  breaking** de 38-6c2) ; une **paire** (±3 jours, somme exacte, aucune écriture seule candidate) est suggérée et
  compte comme « déjà comptabilisée » ; l'annulation défait tout le groupe (38-6c2 AC-1 à AC-4 ; (1) et (2) :
  38-6c1).
- **E22 — L'API des scripts externes (B21).** Un script hors de Kesh (exports CSV de WHMCS et de CB Paid
  Subscriptions) passe par une **clé d'API `read-write`**. Relevé au code : `POST /journal-entries` (écritures
  multi-lignes, en devise par 37-3a), `POST /reconciliation/accept`, `/manual` et `…/transactions/{id}/link-entry` sont
  des routes Comptable où les clés `read-write` sont admises (`lib.rs:523-545, 921-929` ; `require_not_pat` ne garde
  que l'administration, `:518`). **Manques**, comblés par 38-6c1 : (1) retrouver la transaction de prestataire d'une
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
      l'application ; le *client secret* va au coffre, E2) ; `payerId` du compte *business*, **seul champ posé par le
      serveur** (« Tester la connexion », 38-7), affiché, et servant à reconnaître un virement entre les deux comptes
      PayPal de Beat ; `eventKindOverrides` facultatif (code `T…` → genre de mouvement, pour un code que la table de
      38-8 classe `other` ; 38-8 AC-7) ; `overlapDays` facultatif (recouvrement de relecture, 1 à 30, **défaut 3**,
      O11). Ces deux derniers sont modifiables par le client (`PATCH`). **`accountEmail` retiré** (O10 : aucune API ne
      le rend).
    - **Wise (39)** : `profileId` ; empreinte de la clé publique SCA déposée et sa date (renouvellement ≈ 90 jours ; la
      clé privée au coffre) ; type et langue du relevé PDF ; identifiants de solde par devise (`balanceIds`) si la fiche
      ne préfère pas une colonne.
    - **Stripe (40)** : `accountId` (`acct_…`) ; version d'API épinglée (`Stripe-Version`) ; contrôle `livemode`.
  - **Ce qui n'y va pas** : `fetch_from`, `schedule`, état du dernier passage (colonnes communes, lues par le
    planificateur) ; **toute référence à un compte** (frais par devise, compte intermédiaire de Stripe) : colonnes FK de
    `payment_provider_accounts` (E11) ; **secrets** : `_kesh_provider_secrets` chiffrée (E2).

## Registre des contrats entre stories

*(Reportée en phase 2 (2026-11-20) — voir § *Phase 1 KISS*.)*

- **C-38-0 — Une seule migration breaking, dans 38-2** (toutes) : E19, B22, O1 ; relèvement en dernière instruction,
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
  `PROVIDER_ACCOUNT_CANNOT_BE_PRIMARY`, `PROVIDER_ACCOUNT_HAS_TRANSACTIONS` (pose de réglages refusée sur un compte
  bancaire qui porte déjà des transactions, `details.transactionCount` ; 38-2 AC-3), `PROVIDER_ACCOUNT_LEDGER_SHARED`
  (O8 étendu au CHF, D-P2-4) ; levée de C-37-9 pour un compte bancaire porteur de réglages ; `account_kind`
  toujours `bank`.
- **C-38-4 — Transaction de prestataire** (schéma 38-3a, écriture 38-3b) : colonnes E6, E7, E8 ; `movement_kind` ∈ `sale`, `purchase`, `fee`,
  `conversion`, `refund`, `dispute`, `withdrawal`, `deposit`, `other` (tolérant) ; `review_reason` ∈ `dispute`,
  `refund_origin_missing`, `unknown_event`, `other` (tolérant).
- **C-38-5 — Passage** (**38-3a** : schéma, en-tête, gardes ; **38-3b** : connecteur et passage) : E5 ;
  `provider_fetch_runs` (dont `truncated_from`, `ignored_count`), `bank_imports.provider_fetch_run_id`, en-tête
  `source_format = 'PROVIDER_API'` (sous le relèvement de 38-2, E19), affiché « Téléchargé de … », suppression refusée
  (`PROVIDER_IMPORT_NOT_DELETABLE`), imports de fichier refusés sur un compte prestataire
  (`BANK_IMPORT_PROVIDER_ACCOUNT`) ; `BankImportSourceFormat` tolérant (E4, 38-3a) ; passages, soldes vus et
  instantanés **supprimés avec leur connexion** (38-3a AC-4) ; `kesh_providers::Connector` (`balances(as_of)`,
  `movements(window, page)` → `MovementPage { …, skipped, sums }`, page unique admise, `max_window()`,
  `history_start(now)`, `overlap()`, `max_movements_per_page()` — plafond déclaré par le connecteur, lu par
  `validate_page` —, `test()` → `ConnectorTest { account_ref, balances }`, forme générique de « Tester la connexion »,
  38-3b AC-2, 38-7 AC-8) ; `ConnectorError` (`code()` au passage, `app_code()` à `lastError`) ; transport à
  bornes par requête ; fenêtre ≤ `max_window()` (31 jours), redécoupée **dans le connecteur** au besoin (O10) ;
  recouvrement `overlap()` (défaut 3 jours, O11) ; fin = maintenant − 3 h ; début = max(`fetch_from`, point de reprise
  du compte − `overlap()`, `history_start`) ; un échec garde ce qui est inséré, l'état dit où reprendre ; soldes
  rapportés et contrôle de complétude (E20, (a)).
- **C-38-6 — Règles** (38-4) : E14 ; `matchType` ∈ quatre valeurs existantes + `counterparty_email_exact`,
  `invoice_prefix` (boutique, B24), `item_name_contains`, `event_code_prefix` ; filtres facultatifs `direction`,
  `currency`, `bankAccountId` ; unicité `(type, valeur, filtres)` ; `ReconciliationMatchType` tolérant (E4, posé par
  38-4) ; `RULE_CRITERION_UNKNOWN` (écriture d'une valeur inconnue : 400 ; modification d'une règle d'un type inconnu :
  409).
- **C-38-7 — TVA par défaut** (38-5) : E13 ; `PUT /accounts/{id}` gagne `defaultVatKind`, `defaultVatRateId` ;
  réglages TVA gagnent `defaultVatAcquisitionAccountId` (B18) ; `kesh_core::accounting::vat::{vat_from_gross,
  acquisition_vat}` et leur miroir TS, tenus par une fixture partagée.
- **C-38-8 — Propositions** (38-6a, 38-6b) : E15, E16 ; variante `provider` de `accept`, `confirmAlreadyBooked` ; codes
  `RATE_UNAVAILABLE`, `VAT_ACCOUNT_NOT_CONFIGURED`, `VAT_KIND_MISMATCH`, `VAT_KIND_UNKNOWN`, `VAT_RATE_UNAVAILABLE`,
  `RULE_ACCOUNT_CURRENCY_MISMATCH`, `FEE_ACCOUNT_INVALID` (raison `missing` comprise), `PROVIDER_TX_UNDER_REVIEW`,
  `PROVIDER_MOVEMENT_KIND_UNSUPPORTED`, `PROVIDER_AMOUNTS_INCONSISTENT`, `PROVIDER_ACCOUNT_REQUIRED`,
  `PROVIDER_PROPOSAL_REQUIRED` (**seul** code du refus d'une proposition non `provider` sur un compte prestataire,
  posé par 38-3a et repris par 38-6a — D-P2-1 ; `PROVIDER_TX_REQUIRES_PROVIDER_PROPOSAL` n'existe plus ; sur un
  compte prestataire, `rule` et `split` restent refusés, une facture n'est proposée que si `fee_amount = 0` — D-P2-2 ;
  une transaction qui correspond à une facture mais porte des frais n'a **aucune proposition** : file « à examiner »,
  motif calculé `invoice_with_fee` (facture lue **au brut**, `gross_amount`, 38-6a C-38-6a-7), `PROVIDER_TX_UNDER_REVIEW { reason: "invoice_with_fee" }` à l'acceptation —
  D-P2-5),
  `TRANSACTION_ALREADY_BOOKED`, `MULTI_CURRENCY_DISABLED` ; et ceux de 38-6b :
  `PROVIDER_CONVERSION_PAIR_INVALID`, `PROVIDER_PAIR_ACCOUNT_LOCKED`, `PROVIDER_PAIR_CHANGED`,
  `PROVIDER_TRANSFER_ACCOUNT_INVALID`, `PROVIDER_TRANSFER_CURRENCY_MISMATCH`, `PROVIDER_REFUND_ORIGIN_INVALID` (409
  hors lot, `failed[]` dans le lot).
- **C-38-9 — File « à examiner »** (38-6b) : `GET /api/v1/reconciliation/review` ; `POST
  /reconciliation/transactions/{id}/review-done` (Comptable ; action d'audit `reconciliation.review_done`,
  `LIB_ROUTES` +1).
- **C-38-10 — Pièces de compte** (38-10) : E17 ; `GET|POST /api/v1/bank-accounts/{id}/documents`,
  `GET|PATCH|DELETE /…/documents/{docId}` ; PDF seul, taille bornée ; solde de clôture facultatif ; écart au solde
  comptable aux contrôles de clôture (E20).
- **C-38-14 — Rattachement aux écritures existantes et API des scripts** (38-6c1 ; groupe : 38-6c2) : E21, E22 ; `GET
  /reconciliation/link-suggestions`, `POST /reconciliation/link-entries` (lot), `GET
  /bank-accounts/{id}/transactions` (recherche) ; `link-entry` et `entry-candidates` en devise ; clés `read-write`
  admises sur les écritures et le rattachement, lecture pour la recherche ; codes `ENTRY_NOT_LINKABLE`,
  `ENTRY_ALREADY_LINKED`, `AMOUNT_MISMATCH`, `VERSION_CONFLICT` ; la voie des scripts est documentée par 38-6c1 à
  `docs/api-external.md` (dans son périmètre ; le groupe par 38-6c2). **Groupe (B28, 38-6c2)** : `journalEntryIds` (2 à 5, exclusif de
  `journalEntryId`) à `POST …/transactions/{id}/link-entry` et par élément du lot ; `linked[]` porte
  `journalEntryIds` ; code neuf `ENTRY_GROUP_SUM_MISMATCH`, raisons `entry_not_touching_account` et
  `entry_owned_by_document` d'`ENTRY_NOT_LINKABLE` ; suggestions : genre `uniquePair`, motif `several_pairs`, `pairs`
  (aussi à `GET …/entry-candidates`) ; une paire rend `alreadyBooked` vrai (`pairCount`, 34-4, lu par 38-6a) ;
  annulation de tout le groupe par `…/cancel` (vue : `groupEntryIds` ; trace et réponse
  `CancelReconciliationResponse` : `reversalJournalEntryIds`, `null` hors groupe — un seul nom) ; table
  `bank_transaction_extra_entries` et fragment unique « écriture rapprochée » (lien direct ou membre de groupe) lu par
  tous les sites de l'inventaire de 38-6c2 AC-1.
  Aucune route neuve.
- **C-38-11 — Tests** (toutes) : aucun appel réseau en CI ; fixtures enregistrées **anonymisées** (courriels, noms,
  identifiants remplacés ; montants et codes gardés) sous `crates/kesh-providers/tests/fixtures/paypal/` ; tests de
  réponses hostiles (taille, structure, montants, dates, devises, pagination incohérente).
- **C-38-12 — Registre des routes** : chaque route neuve entre à `LIB_ROUTES` (et `TEST_ENDPOINT_ROUTES` pour la route
  `_test`), décompte en deltas à chaque fiche.
- **C-38-13 — Tracé pour plus tard** (issues avec la PR) : rotation de `KESH_SECRETS_KEY` (décidé, ex-Q5) ; TVA due
  des ventes hors facture et impôt sur les acquisitions dans le rapport TVA (voir *Point TVA pour Guy*) ; catégorie
  de vente par analytique (B19, Q3, reportée B25) ; annulation d'un rattachement dans un exercice clos (refusée parce
  qu'elle contre-passe ; B28, 38-6c2 AC-4) ; règlement d'une facture au brut avec une ligne de frais sur un compte
  prestataire (une facture n'y est proposée que si `fee_amount = 0`, D-P2-2). *(L'unicité des règles par sens ou devise n'y est plus : réglée par
  `filter_key`, E14.)* *(Les
  outils MCP n'y sont plus : décidé, aucun outil neuf, E23.)*

## Stories

*(Reportée en phase 2 (2026-11-20) — voir § *Phase 1 KISS*.)*

| Story | Objet | Modules (≤ 5) | Dépend de | Migration |
|---|---|---|---|---|
| 38-0 | kickoff : décomptes, validation de ce fichier, numéro de release (Q1) | — | — | — |
| 38-1 | coffre des secrets et connexions de prestataire, réglages `settings JSON` (E24) | `kesh-db`, `kesh-core` (structs de réglages), `kesh-api` (config, secrets, routes) | Epic 37 | oui, non breaking (`payment_provider_connections`, `_kesh_provider_secrets`) |
| 38-2 | compte bancaire `bank` + réglages de prestataire, lié à un compte en devise existant (D8), compte de frais par devise ; **la migration breaking** (O1) | `kesh-db`, `kesh-api` (bank_accounts, provider-connections), `kesh-core` (gardes) | 38-1, **numéro de release (Guy, Q1)** | oui, **la seule breaking de l'epic** (`payment_provider_accounts`, contrainte d'IBAN remplacée, `match_type` → `VARCHAR`, `min_required` relevé, crates bumpés, E19) |
| 38-3 | *découpée en 38-3a et 38-3b (`split`, D-P1-1 : six modules)* | — | — | — |
| 38-3a | téléchargement, schéma : colonnes de prestataire, passages, soldes vus et rapportés, point de reprise ; lecture tolérante de `PROVIDER_API` ; suppression d'en-tête, imports de fichier et propositions existantes refusés sur un compte prestataire ; passages supprimés avec leur connexion | `kesh-db`, `kesh-core`, `kesh-api`, `kesh-i18n` | 38-1, 38-2 | oui, non breaking (`provider_fetch_runs`, `provider_balances_seen`, `provider_balance_snapshots` ; colonnes `bank_transactions`, `bank_imports.provider_fetch_run_id`, `payment_provider_accounts.fetched_through`) |
| 38-3b | téléchargement, passage : crate `kesh-providers` (`Connector`, validation, factice), transport à bornes par requête, fabrique, passage sans doublon, planifié et à la demande, état et historique, soldes rapportés et contrôle de complétude | `kesh-providers` (neuf), `kesh-api`, `kesh-db`, `kesh-i18n` | 38-1, 38-2, 38-3a | non |
| 38-4 | critères neufs des règles (types de `match_type`, dont boutique = préfixe de facture), filtres sens, devise, compte ; `ReconciliationMatchType` tolérant | `kesh-db`, `kesh-reconciliation`, `kesh-api` (reconciliation_rules), `frontend` (libellés), i18n | 38-2, 38-3a | oui, non breaking (trois colonnes de filtre, `filter_key`, index unique remplacé) |
| 38-5 | code TVA par défaut d'un compte, extraction de la TVA, compte d'impôt sur les acquisitions | `kesh-db`, `kesh-core`, `kesh-api` (accounts, réglages TVA), `frontend` (accounts, réglages) | Epic 37 | oui, non breaking (`accounts` × 2, `company_invoice_settings` × 1) |
| 38-6a | propositions de prestataire : vente, achat, frais (brut, TVA, frais, cours) et acceptation en lot ; « déjà comptabilisée » | `kesh-reconciliation`, `kesh-api` (reconciliation), `kesh-db` | 38-3a, 38-4, 38-5, 38-6c1, 38-6c2 | non attendue |
| 38-6b | conversions (une écriture si un côté est en CHF ; frais de chaque moitié au compte de frais, B27), retraits, remboursements (tels que rapportés), litiges ; file « à examiner » | `kesh-reconciliation`, `kesh-api` (reconciliation, conversions), `kesh-db` | 38-2, 38-3a, 38-6a, 38-6c1, 38-6c2, 37-3b (A33) | non attendue |
| 38-6c | *découpée en 38-6c1 et 38-6c2 (`split`, D-P2-3 : plus de cinq modules)* | — | — | — |
| 38-6c1 | rattachement aux écritures existantes : en devise, suggestions, en lot (2024, B17) ; recherche et API des scripts externes (B21) ; exercice clos | `kesh-db` (reconciliation_link), `kesh-api` (reconciliation, bank_accounts), `kesh-i18n`, `docs/api-external.md` | 38-3a | non |
| 38-6c2 | rattachement à un groupe de 2 à 5 écritures (B28), paires suggérées et « déjà comptabilisées », annulation du groupe | `kesh-db` (reconciliation_link, reconciliation_cancel, lecteurs de l'inventaire, migration, sauvegarde), `kesh-api` (reconciliation, exports), `kesh-i18n`, `docs/api-external.md` | 38-6c1 | oui, non breaking (`bank_transaction_extra_entries`, B28) |
| 38-7 | client PayPal : OAuth2, Transaction Search, fenêtres de 31 jours redécoupées dans le connecteur, pagination, soldes (`as_of_time`), historique de trois ans | `kesh-providers` (paypal), `kesh-api` (transport) | 38-3b | non |
| 38-8 | correspondance des codes d'événement PayPal vers les mouvements ; `invoice_id` → `invoice_ref` (boutique, B24) ; surcharges `eventKindOverrides` | `kesh-providers` (paypal), `kesh-core` (réglages) | 38-7 | non |
| 38-9a | écrans : connexions, réglages, secrets, soldes proposés, comptes prestataires, contrôles de solde | `frontend` (provider-connections neuf, bank-accounts), i18n, manuel d'administration (sauvegarde d'avant la mise à jour obligatoire, E19) | 38-1, 38-2, 38-3a, 38-3b, 38-7 | non |
| 38-9b | écrans : règles étendues, propositions multi-lignes, rattachement en lot, file « à examiner », E2E | `frontend` (reconciliation, règles), i18n, manuel utilisateur, E2E | 38-4 à 38-6c2, 38-8 | non |
| 38-10 | relevé joint à la main (compte et exercice ou mois), solde de clôture, écart aux contrôles de clôture | `kesh-db`, `kesh-api` (routes, sauvegarde, closing-checks), `frontend` (bank-accounts) | 38-2, 38-3a | oui, non breaking (`bank_account_documents`) |

**Migrations : sept fichiers, dont une seule breaking** (38-1, **38-2 — la breaking** (O1) —, 38-3a, 38-4, 38-5,
38-6c2 — ajoutée par B28, en 38-6c avant le découpage D-P2-3 —, 38-10) ; seule la breaking écrit une donnée
(`_kesh_version`, exemptée P7) ; les autres n'en écrivent pas (sauf
découverte à la fiche). **Ordre** : 38-1 → 38-2 → 38-3a → (38-3b, 38-4, 38-5, 38-6c1 en parallèle possibles) → 38-6c2
(après 38-6c1) ; 38-7 (après 38-3b) → 38-8 → 38-6a (après 38-4, 38-5, 38-6c2) → 38-6b → 38-9a → 38-9b ; 38-10 dès
38-3a. 38-6c1 et 38-6c2 avant 38-6a : les transactions de 2024 se rattachent avant que des propositions ne leur
soient faites, et 38-6a lit `pairCount` (38-6c2 AC-3) pour « déjà comptabilisée ». Une story qui découvre une
migration non prévue le dit avant de l'écrire (C-38-0).

Chaque fiche porte ses **exigences numérotées** (B26) : `FR-38-n-k` (fonctionnelles), `NFR-38-n-k` (non
fonctionnelles) ; récapitulatif au § *Exigences (FR/NFR) par story*. Les gardes communes (P1–P8, *Pattern batch*,
aucun appel réseau en CI, recette visuelle) ne s'y répètent que là où la story les porte en propre.

### 38-0 — Kickoff

*(Fait le 2026-11-15.)*

- *Intention* : ouvrir l'epic sur des décomptes mesurés et un fichier validé.
- *Périmètre* : décomptes d'issues et de bugs (§ *Priorités des défauts*), objectif du tag, validation de ce fichier
  en boucle, numéro de release obtenu de Guy (Q1).
- **FR**
  1. `FR-38-0-1` — Compter les issues ouvertes et les bugs, et écrire au fichier d'epic le compte du tag précédent et
     l'objectif du tag.
  2. `FR-38-0-2` — Valider ce fichier en boucle jusqu'à une passe sans finding au-dessus de LOW.
  3. `FR-38-0-3` — Obtenir de Guy le numéro de la release qui porte la migration breaking (Q1), avant le
     développement de 38-2 (O1).
- **NFR**
  1. `NFR-38-0-1` — Décomptes recomptés à la source, commande et périmètre écrits avec le nombre.

### 38-1 — Coffre des secrets et connexions de prestataire

*(Reportée en phase 2 (2026-11-20) — voir § *Phase 1 KISS*.)*

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

*(Reportée en phase 2 (2026-11-20), sauf ce que reprend 38-11b (trois colonnes de `bank_accounts`, levée de C-37-9, migration non breaking) — voir § *Phase 1 KISS*.)*

- *Intention* : chaque solde de devise d'une connexion devient un compte bancaire Kesh ordinaire, lié à un compte du
  plan déjà en devise (numéro, historique et soldes gardés), avec ses réglages de prestataire : connexion, devise,
  compte de frais de la devise.
- *Périmètre* : migration **breaking** (O1 : `CREATE TABLE payment_provider_accounts`, E11, comptes liés en colonnes
  FK, B23 ; contrainte d'IBAN remplacée ; `match_type` → `VARCHAR(40)` ; `min_required` relevé, crates bumpés, E19) ;
  routes C-38-3 ; levée de `ACCOUNT_IN_FOREIGN_CURRENCY` (C-37-9) pour un compte porteur de réglages, posés
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
  3. `NFR-38-2-3` — **La** migration breaking de l'epic (O1) : relèvement en dernière instruction, crates bumpés dans
     le même commit, retour par la sauvegarde d'avant la mise à jour ; `account_kind` toujours `'bank'`.

### 38-3a et 38-3b — Téléchargement (découpage D-P1-1)

*(Reportée en phase 2 (2026-11-20) — voir § *Phase 1 KISS*.)*

38-3 touchait six modules ; elle est **découpée** (statut `split`) en **38-3a** — schéma non breaking (colonnes de
prestataire, passages, soldes, point de reprise), lecture tolérante de `PROVIDER_API`, gardes (suppression d'en-tête,
imports de fichier et propositions existantes refusés sur un compte prestataire, `balance_seen`), passages supprimés
avec leur connexion — et **38-3b** — crate `kesh-providers` (`Connector` avec `history_start`, `overlap`,
`MovementPage.skipped`/`sums`), transport à bornes par requête, fabrique, passage, planification, lecture de l'état,
soldes et complétude ; aucune migration. La migration breaking est en 38-2 (O1) ; le décodage tolérant de
`ReconciliationMatchType` passe à 38-4. Fiches : `38-3a-telechargement-schema.md`, `38-3b-telechargement-passage.md` ;
leurs exigences (`FR-38-3a-k`, `FR-38-3b-k`…) remplacent celles de 38-3 ci-dessous, gardées comme **historique du
cadrage**.

*(Historique — cadrage de 38-3 avant O1 et D-P1-1.)*

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

*(Reportée en phase 2 (2026-11-20), sauf ce que reprend 38-11c (critères e-mail exact et type PayPal, sens) — voir § *Phase 1 KISS*.)*

- *Intention* : une règle reconnaît une transaction de prestataire par courriel, **boutique (préfixe du numéro de
  facture, B24)**, nom d'article ou code d'événement, et se restreint par sens, devise et compte — pour proposer le bon
  sous-compte de ventes de la devise et de la boutique.
- *Périmètre* : E14 ; `ReconciliationMatchType` tolérant et `RULE_CRITERION_UNKNOWN` (E4, repris de 38-3 après
  D-P1-1) ; migration non breaking (trois colonnes de filtre, `filter_key`, index unique remplacé) ;
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

*(Reportée en phase 2 (2026-11-20) — voir § *Phase 1 KISS*.)*

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

*(Reportée en phase 2 (2026-11-20), sauf ce que reprend 38-11c (frais, achats par règle ; aucune écriture de vente) — voir § *Phase 1 KISS*.)*

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

*(Reportée en phase 2 (2026-11-20), sauf ce que reprend 38-11c (conversions au même CHF, retraits, « à examiner ») — voir § *Phase 1 KISS*.)*

- *Intention* : les mouvements qui ne sont ni vente ni achat se proposent juste, ou attendent un humain.
- *Périmètre* : E16 ; paire de conversion vers `POST /journal-entries/conversions` (A31) ; rattachement des retraits
  (34-11b) ; inverse d'une écriture d'origine ; file (C-38-9).
- *Risques* : paire de conversion dont une moitié manque (passage interrompu) ; remboursement partiel (montants
  rapportés, TVA extraite du brut remboursé, B20) ; frais rendus en partie ou pas du tout ; litige résolu en faveur du
  vendeur (deux mouvements qui s'annulent) ; annulation d'une réconciliation de paire (les deux côtés). *B27* : frais
  de la moitié créditée d'une conversion au compte de frais de sa devise (A33), jamais dans la différence de change.
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

*(Reportée en phase 2 (2026-11-20), sauf ce que reprend 38-11c (rattachement en devise, un par un ; ni lot ni groupe) — voir § *Phase 1 KISS*.)*

*Découpée le 2026-11-15 (D-P2-3) en **38-6c1** (rattachement en devise, suggestions, lot, recherche, API des scripts,
exercice clos ; aucune migration) et **38-6c2** (groupe d'écritures B28, table `bank_transaction_extra_entries`,
paires, annulation du groupe) ; 38-6c2 avant 38-6a. Les listes ci-dessous sont celles du cadrage (`FR-38-6c-k`) ; les
fiches les renumérotent : `FR-38-6c-1..7` → `FR-38-6c1-1..7`, `NFR-38-6c-1..4` → `NFR-38-6c1-1..4`, `FR-38-6c-8..10`
→ `FR-38-6c2-1..3`, `NFR-38-6c-5..6` → `NFR-38-6c2-1..2`, plus `NFR-38-6c2-3`.*

- *Intention* : les transactions téléchargées de 2024, déjà passées par l'API, se rattachent **en lot** à leurs
  écritures, sans doublon ; un script externe (WHMCS, CB Paid Subscriptions) retrouve une transaction de prestataire,
  crée l'écriture et la rattache, par une clé d'API.
- *Périmètre* : E21, E22, C-38-14 ; rattachement **en devise** (`transaction_can_be_linked_to_entry`,
  `reconciliation_link.rs:535`, et les textes SQL du dialogue et de `count_already_booked_for_page`, une seule règle,
  C-v017-3-7) ; suggestions univoques et lot `link-entries` (*Pattern batch*) ; recherche `GET
  /bank-accounts/{id}/transactions` ; clés `read-write` sur `link-entries`. Écran : 38-9b.
- *Risques* : **l'égalité « lecture par page = fonctions du dialogue »** (test `s34_4_ac8_…`) tenue en devise ; une
  écriture qui touche deux comptes prestataires (conversion, retrait) ; écritures de 2024 passées au brut avec frais
  **à part** (B28 : rattachement à un groupe d'écritures, table de liaison, migration non breaking ; faux
  rattachement d'une paire — suggérée seulement, jamais tranchée) ;
  exercice 2024 clos ou livres verrouillés (le rattachement n'écrit pas d'écriture : admis, comme v017-3a — à
  confirmer à la fiche) ; performance sur une année (≈ milliers de transactions × écritures du compte).
- **FR**
  1. `FR-38-6c-1` — Une transaction en devise se rattache à une écriture existante sur le montant en devise.
  2. `FR-38-6c-2` — Kesh suggère, par transaction en attente, l'écriture univoque ou la liste ambiguë.
  3. `FR-38-6c-3` — Un lot de rattachements s'envoie en une requête (`POST /reconciliation/link-entries`).
  4. `FR-38-6c-4` — Une transaction de prestataire se cherche par identifiant du prestataire, numéro de facture,
     statut et période.
  5. `FR-38-6c-5` — Un script externe crée l'écriture et la rattache par une clé d'API `read-write`.
  *(La fiche ajoute FR-38-6c-6 et -7, puis, sur B28, FR-38-6c-8 groupe de 2 à 5 écritures de somme exacte, -9 paire
  suggérée et « déjà comptabilisée », -10 annulation du groupe.)*
- **NFR**
  1. `NFR-38-6c-1` — Une seule règle de rattachement : lecture par page égale aux fonctions du dialogue, en devise.
  2. `NFR-38-6c-2` — *Pattern batch* ; rien de rattaché d'office.
  3. `NFR-38-6c-3` — Une année de transactions se suggère en temps borné (mesure à la fiche).
  4. `NFR-38-6c-4` — Routes des connexions et des secrets toujours fermées aux clés d'API.
  *(Sur B28 : NFR-38-6c-5 migration non breaking, une écriture dans un groupe au plus ; -6 définition unique
  d'« écriture rapprochée ».)*

### 38-7 — Client PayPal

*(Reportée en phase 2 (2026-11-20) — voir § *Phase 1 KISS*.)*

- *Intention* : lire, pour une connexion PayPal, les soldes et les transactions qui changent le solde.
- *Périmètre* : OAuth2 `client_credentials` (`clientId` des réglages, secret du coffre ; jeton gardé en mémoire
  jusqu'à expiration), `GET /v1/reporting/transactions` (`start_date`/`end_date` ≤ 31 jours, `fields=all`,
  `page_size=500`, `page`, `balance_affecting_records_only=Y`), `GET /v1/reporting/balances` ; plafond de 10 000
  résultats → la fenêtre se redécoupe ; trois ans d'historique → `fetch_from` antérieur refusé à l'écran et borné au
  passage ; « Tester la connexion » pose `payerId` aux réglages (E24 ; `accountEmail` retiré, O10) ; fenêtre trop
  large redécoupée dans le connecteur (O10) ; erreurs (401, 403 permission
  *Transaction Search* manquante, 429, 5xx) en états lisibles ; URL de base constante (*live*), autre base seulement
  en test.
- *Risques* : permission manquante sur l'application REST (message clair) ; pagination incohérente ; délai de 3 h ;
  fuseau des dates PayPal.
- **FR**
  1. `FR-38-7-1` — Obtenir un jeton OAuth2 avec `clientId` et le secret de la connexion.
  2. `FR-38-7-2` — Lire les transactions par fenêtres de 31 jours au plus, toutes les pages, en redécoupant au-delà
     de 10 000 résultats.
  3. `FR-38-7-3` — Lire les soldes par devise, à une date donnée (`as_of_time`).
  4. `FR-38-7-4` — « Tester la connexion » lit les soldes et pose `payerId` aux réglages (`accountEmail` retiré, O10).
  5. `FR-38-7-5` — Chaque erreur PayPal devient un état lisible de la connexion.
- **NFR**
  1. `NFR-38-7-1` — URL de base constante, client HTTP borné de 37-2 (délai, taille).
  2. `NFR-38-7-2` — Jeton en mémoire seulement, jamais écrit.
  3. `NFR-38-7-3` — Validation stricte des réponses ; tests sur fixtures anonymisées et réponses hostiles.

### 38-8 — Codes d'événement PayPal

*(Reportée en phase 2 (2026-11-20) — voir § *Phase 1 KISS*.)*

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

*(Reportée en phase 2 (2026-11-20), sauf le minimum repris par 38-11d — voir § *Phase 1 KISS*.)*

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
  3. `FR-38-9b-3` — Rattachement en lot : univoques à cocher (écritures seules et **paires**, `uniquePair`),
     ambiguës une à une ; au dialogue, paires proposées et **groupe composé à la main** (2 à 5 écritures, somme
     affichée contre le montant ; 38-6c2).
  4. `FR-38-9b-4` — File « à examiner » et action « examiné ».
  5. `FR-38-9b-5` *(fiche)* — L'annulation d'une **paire** de conversion (38-6b) ou d'un **groupe** d'écritures
     (38-6c2) est annoncée avant d'être faite (les deux moitiés, ou toutes les écritures contre-passées).
- **NFR (38-9b)**
  1. `NFR-38-9b-1` — E2E contre le transport factice (téléchargement → règle → proposition → acceptation, et
     rattachement en lot).
  2. `NFR-38-9b-2` — Textes « proposer », jamais « comptabiliser automatiquement » ; quatre locales ; manuel
     utilisateur et PDF à jour.
  3. `NFR-38-9b-3` — Captures d'écran de chaque état touché.

### 38-10 — Relevé joint à la main

*(Reportée en phase 2 (2026-11-20) — voir § *Phase 1 KISS*.)*

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

*(Reportée en phase 2 (2026-11-20) — voir § *Phase 1 KISS*.)*

Récapitulatif (B26) ; le texte de chaque exigence est dans la **fiche, qui fait foi** — les listes des sections
ci-dessus sont celles du cadrage, les fiches en ont ajouté (*ajout*). **Forme définitive — issues GitHub ou fiches —
à Guy** (Q12). Recompté **aux fiches** le 2026-11-15, après la P2 et les découpages D-P1-1 et D-P2-3 (identifiants distincts de
la story dans sa fiche : `grep -oE "\bFR-38-<n>-[0-9]+" <fiche> | sort -u | wc -l`, de même pour `NFR`) ; 38-0 n'a pas
de fiche : compté à sa section ci-dessus ; 38-3 (`split`) n'est plus comptée.

| Story | FR | NFR |
|---|---|---|
| 38-0 | 3 — 1 décomptes · 2 validation · 3 numéro de release | 1 — 1 décomptes recomptés |
| 38-1 | 7 — 1 connexion CRUD · 2 secrets en écriture seule · 3 réglages validés · 4 suppression refusée si utilisée · 5 secrets effacés à la restauration · 6 remise à zéro · 7 « Tester », contrôle local | 6 — 1 AES-256-GCM, clé d'environnement · 2 aucun secret nulle part · 3 session d'Administrateur seule · 4 migration non breaking, JSON contrôlé · 5 clé changée ou chiffré altéré signalés · 6 variable transmise et documentée |
| 38-2 | 8 — 1 créer ou poser les réglages · 2 lien à un compte en devise (D8) · 3 compte de frais de la devise · 4 une devise par connexion, retrait refusé si utilisé · 5 jamais principal ni pain.001 · 6 un compte du plan, un compte prestataire · 7 bloc `provider` de la liste · 8 connexion utilisée non supprimable | 3 — 1 gardes sous verrou · 2 FK `RESTRICT`, archivage gardé · 3 **la** migration breaking (O1), `bank` |
| 38-3a | 5 — 1 schéma des mouvements, passages, soldes · 2 en-tête lu, suppression refusée · 3 source inconnue lue · 4 imports de fichier et propositions existantes refusés · 5 `balance_seen`, passages supprimés avec la connexion | 3 — 1 non breaking · 2 sauvegarde antérieure réimportée, recette étendue · 3 aucun comportement selon `source_format` |
| 38-3b | 6 — 1 téléchargement quotidien ou à la demande · 2 sans doublon, `overlap()` · 3 passage et en-tête `PROVIDER_API` · 4 solde sans compte proposé · 5 complétude · 6 état lu par le Comptable | 5 — 1 un passage à la fois, reprise · 2 aucun réseau en CI · 3 validation des réponses · 4 date à Zurich · 5 bornes par requête, aucun texte d'erreur ni secret |
| 38-4 | 6 — 1 quatre types neufs · 2 trois filtres · 3 une règle par boutique et devise · 4 compte en devise · 5 type inconnu inerte · 6 compte de règle en devise admis | 3 — 1 non-régression · 2 normalisation unique · 3 non breaking |
| 38-5 | 5 — 1 genre et taux par compte · 2 extraction, acquisitions · 3 compte d'acquisitions · 4 taux à la date · 5 traitement TVA d'un compte à une date | 3 — 1 centime, Rust = TS · 2 TVA en devise principale · 3 non breaking |
| 38-6a | 7 — 1 proposition vente/achat · 2 frais seuls · 3 lot `provider` · 4 déjà comptabilisée · 5 cours · 6 proposition bloquée visible (`blockedBy`) · 7 variante `rule` refusée sur un compte prestataire | 5 — 1 *Pattern batch* · 2 revalidation sous verrou · 3 module à part, pur · 4 équilibre exact, ligne d'arrondi · 5 comptes non prestataires inchangés |
| 38-6b | 5 — 1 conversion (une écriture si un côté est en CHF) · 2 retrait/dépôt · 3 remboursement inverse · 4 file « à examiner » · 5 file lue par l'API | 4 — 1 paire liée · 2 aucune écriture d'office · 3 une seule définition de la paire, de l'origine, de la file · 4 file calculée à la lecture |
| 38-6c1 | 7 — 1 rattachement en devise · 2 suggestions · 3 lot · 4 recherche · 5 script externe · 6 écriture sur deux comptes bancaires · 7 rattacher n'écrit aucune écriture | 4 — 1 une seule règle · 2 *Pattern batch* · 3 temps borné · 4 connexions fermées aux clés |
| 38-6c2 | 3 — 1 groupe d'écritures de somme exacte (B28) · 2 paire suggérée, « déjà comptabilisée » (B28) · 3 annulation du groupe (B28) | 3 — 1 migration non breaking, une écriture dans un groupe au plus · 2 définition unique d'« écriture rapprochée » · 3 une seule règle, paires comprises, temps borné (*ajout* au découpage) |
| 38-7 | 6 — 1 OAuth2 · 2 transactions par fenêtres · 3 soldes · 4 « Tester » (`payerId`) · 5 erreurs lisibles · 6 trois ans d'historique | 4 — 1 URL constante, client borné · 2 jeton en mémoire · 3 validation stricte · 4 aucun secret ni jeton |
| 38-8 | 6 — 1 classement des codes · 2 champs lus · 3 `invoice_id` → `invoice_ref` · 4 surcharges · 5 moitiés de conversion reliées · 6 seuls les enregistrements qui changent le solde | 3 — 1 test par code relevé · 2 classement pur · 3 classement vérifiable par la somme des nets |
| 38-9a | 7 — 1 écrans des connexions · 2 Tester, Télécharger · 3 soldes proposés · 4 comptes prestataires · 5 contrôles de solde · 6 compte prestataire reconnu dans la liste · 7 écrans existants touchés par les gardes | 4 — 1 aucun secret au DOM · 2 i18n, manuel admin · 3 captures · 4 « télécharger », « proposer » |
| 38-9b | 5 — 1 formulaire des règles · 2 propositions multi-lignes · 3 rattachement en lot · 4 file « à examiner » · 5 annulation d'une moitié de conversion annoncée | 4 — 1 E2E · 2 textes, i18n, manuel utilisateur · 3 captures · 4 composants à part |
| 38-10 | 5 — 1 joindre un PDF · 2 solde de clôture · 3 écart affiché · 4 contrôles de clôture · 5 lister, corriger, retirer | 4 — 1 PDF borné · 2 sauvegarde · 3 non breaking · 4 audit, i18n, manuel, captures |

**Total : 91 FR et 59 NFR** sur 16 stories (38-0 comprise, 38-3a et 38-3b, 38-6c1 et 38-6c2 comptées à part, 38-3
et 38-6c `split` exclues ; 38-9a et 38-9b à part), recomptés aux fiches le 2026-11-15 après l'alignement
inter-fiches de la P2 et le découpage D-P2-3 (même commande, chaque ligne égale à sa fiche, 38-0 comptée à sa
section) : 91 FR et 58 NFR avant ; 38-6c (10 FR, 6 NFR) devient 38-6c1 (7, 4) et 38-6c2 (3, 3) — **+1 NFR**,
`NFR-38-6c2-3`. *(Après B27–B28 : 91 FR et 58 NFR ; après la P1 : 88 et 56 ; avant la P1 : 64 FR et 41 NFR, compte des
seules listes du cadrage.)* ⚠️ 38-6a, 38-6b, 38-7, 38-8, 38-9a et 38-9b étaient alignées en parallèle au moment de ce
recompte (leurs décomptes lus ce jour-là égalent la table) : à recompter par la même commande après leur alignement.

## Ce que réutiliseront les Epics 39 (Wise) et 40 (Stripe)

*(Reportée en phase 2 (2026-11-20) — voir § *Phase 1 KISS*.)*

**Réutilisé tel quel** : coffre des secrets et connexions (38-1), réglages de compte prestataire et compte de frais
(38-2), cadre de téléchargement, passages, doublon, planification, soldes vus et contrôles (38-3a, 38-3b), règles étendues
(38-4), TVA par défaut (38-5), propositions et file (38-6a, 38-6b), rattachement en lot et API des scripts (38-6c1, 38-6c2),
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

*(Reportée en phase 2 (2026-11-20) — voir § *Phase 1 KISS*.)*

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

*(2026-11-20 : Q1 tombe avec la migration breaking de l'ancienne 38-2, reportée en phase 2 ; la phase 1 n'a qu'une migration non breaking. Q3, Q9, Q12 restent ouvertes pour la phase 2.)*

*Closes* : Q2 (rattachement en lot, B17, E21), Q3 sauf l'analytique (B18, B19), Q4 et Q5 (décidées, E2), Q6 (B20,
E16), Q7 (un critère principal + filtres, E14), Q8 (aucun outil MCP, E23), **Q10** (boutique = préfixe
d'`invoice_id`, B24), **Q11** (`PROVIDER_API` retenu, B22).

- **Q1 — Numéro de la release (Guy), rouverte par B22** : la migration breaking de **38-2** (O1) fige la version dans
  son SQL (`kesh_version_min_required`) et impose le bump des crates dans le même commit (P2-bis). Proposition : la
  mineure suivante (`0.18.0`), l'Epic 37 n'en relevant aucune. **Bloquant pour le développement de 38-2**, pas pour
  les autres.
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

## Arbitrages de l'orchestrateur après la rédaction des fiches (2026-11-15)

Questions des fiches tranchées par l'orchestrateur (Beat peut les contester ; Q-6b-1 et Q-38-6c-1 lui sont posées —
*annoté le 2026-11-15 : closes par B27 et B28, § Révision B27–B28*) :

- **O1 — Migration breaking déplacée en 38-2.** Un compte `bank` exige un IBAN (`chk_bank_accounts_bank_has_iban`,
  `20261110000001_bank_accounts_without_iban.sql:48-49` ; `routes/bank_accounts.rs:557-570`) : E11 était faux.
  **L'unique migration breaking de l'epic est celle de 38-2**, première story qui en a besoin : elle remplace la
  contrainte (IBAN obligatoire sauf pour un compte qui porte une ligne `payment_provider_accounts` — forme exacte à la
  fiche), passe `match_type` en `VARCHAR(40)` (CHECK de format seul, `ascii_bin`, O2) et relève
  `kesh_version_min_required` en dernière instruction (P2, P2-bis : onze crates si `kesh-providers` naît avant, sinon
  dix — à recompter au développement). 38-3 n'a plus que sa migration non breaking ; ses écritures de `PROVIDER_API`
  s'appuient sur le relèvement de 38-2 (même release). Fiches 38-2, 38-3, 38-4 à aligner.
- **O2** — `match_type` : `CHECK` de format (sans énumération) et collation `ascii_bin` admis (Q-38-3-a).
- **O3** — 38-4 : le verrou « un compte désigné par une règle ne reçoit pas de devise » (`ACCOUNT_CURRENCY_IN_USE`)
  reste ; seul « désigner un compte en devise » est levé.
- **O4** — 38-4 : la désactivation (`DELETE`) d'une règle de type inconnu est admise.
- **O5** — 38-5 : le taux TVA par défaut d'un compte désigne sa **catégorie**, résolue à la date de la transaction
  (`find_for_category_at_date`).
- **O6** — 38-5 : compte d'impôt sur les acquisitions égal au compte de TVA due → refusé.
- **O7** — 38-1 : unicité du libellé d'une connexion par société gardée (Q-38-1-b) ; le Comptable lit l'état des
  connexions par la route de lecture restreinte de 38-3 (Q-38-1-a).
- **O8** — 38-2 : délier un compte prestataire n'est admis que s'il ne porte aucune transaction téléchargée
  (Q-38-2-b) ; un compte du plan en devise est lié à **un seul** compte prestataire (Q-38-2-c). *(Étendu le
  2026-11-15, D-P2-4 : le compte du plan d'un compte prestataire, en devise **ou en CHF**, n'est lié à aucun autre
  compte bancaire, dans les deux sens — `PROVIDER_ACCOUNT_LEDGER_SHARED`, 38-2 FR-38-2-6.)*
- **O9** — 38-3 : imports de fichier refusés sur un compte prestataire (Q-38-3-c) ; découpage 38-3a/38-3b si la
  validation stagne (Q-38-3-b, règle de splitting).
- **O10** — 38-7 : `accountEmail` retiré du JSON (aucune API ne le rend) ; une fenêtre trop large se découpe **dans le
  connecteur** ; la pagination PayPal commence à 1 (`page` par défaut 1 dans la doc Transaction Search) — à constater
  sur les réponses enregistrées.
- **O11** — 38-8 : les deux moitiés d'une conversion se relient par `paypal_reference_id` (à constater sur les réponses
  de Beat) ; un paiement en attente au-delà du recouvrement de trois jours est relu par un recouvrement réglable ;
  surcharge `ignore` gardée ; `refund_origin_missing` est **calculé** (38-6b), jamais écrit (38-8 à aligner).
- **O12** — 38-9a : routes des soldes vus et de l'historique des passages arrêtées par 38-3 (désormais 38-3b) ; 38-9b :
  période par défaut du rattachement en lot = l'exercice ouvert le plus ancien.

## Validation P1 — décisions et alignement inter-fiches (2026-11-15)

Liste : `implementation-artifacts/38-p1-inter-fiches.md`.

- **D-P1-1 — 38-3 découpée** en **38-3a** (schéma non breaking, entités, lecture tolérante de `PROVIDER_API`, gardes :
  suppression d'en-tête, import de fichier, propositions existantes, `balance_seen`) et **38-3b** (crate
  `kesh-providers`, `Connector`, transport, passage, planification, contrôle des soldes ; aucune migration) — six
  modules sinon (règle de splitting). 38-3 passe `split`. **Tranché en 38-3a** : passages, soldes vus et instantanés
  sont **supprimés avec leur connexion** (cascade + `DELETE` explicite), seul `bank_imports.provider_fetch_run_id` reste
  `RESTRICT`.
- **D-P1-2** — 38-5-P1-5 (compte d'acquisitions égal à d'autres passifs) reste LOW, non traité.
- **Alignements** de ce fichier : migration breaking en **38-2** partout (en-tête, E4, E14, E19, C-38-0, table,
  sections, Q1) ; décodage tolérant à la première écriture (`PROVIDER_API` en 38-3a, critères en 38-4) ; E11 et
  inventaire (un `bank` sans IBAN n'était pas admis) ; E6 (lecture préalable des identifiants, `CLIENT_FOUND_ROWS`) ;
  E16 (conversion à moitié CHF en une écriture, C-38-6b-2 bis) ; O10/O11 (découpage dans le connecteur, `overlapDays`
  1–30 défaut 3, `accountEmail` retiré) ; dépendances de 38-6a, 38-6b ; codes de C-38-8, action d'audit et route de
  38-6b, `docs/api-external.md` de 38-6c ; récapitulatif FR/NFR recompté aux fiches (88 FR, 56 NFR) ; migrations : six
  fichiers.
- Les sections *Révision B16–B21* et *Révision B22–B26* ci-dessus restent **historiques** (elles placent la breaking en
  38-3 et gardent `accountEmail`).

## Révision B27–B28 (2026-11-15)

Sur les décisions de Beat B27 et B28 (`epics-38-40-cadrage.md`, *questions de la rédaction*) ; ferme Q-6b-1 et
Q-38-6c-1.

| Décision | Effet | Fiches et sections touchées |
|---|---|---|
| **B27** — frais donné à part sur la moitié créditée d'une conversion → compte de frais du prestataire et de la devise, jamais la différence de change | ligne au **débit** du compte de frais (compte de charge de la devise, garde de 38-2) dans l'écriture 2 de 37-3b, admise par **A33** (Epic 37, approuvé le 2026-11-15) ; différence calculée **après** ; dans la conversion à moitié CHF (une écriture), frais de chaque moitié à son compte ; limite « frais dans la différence » retirée ; compte de frais absent → `FEE_ACCOUNT_INVALID` | 38-6b (Écart 2, C-38-6b-2, C-38-6b-2 bis, AC-2, AC-9, Q-6b-1 close) ; E16 ; table des stories (38-6b dépend de A33) ; aucun FR/NFR neuf |
| **B28** — frais 2024 en écritures à part → une transaction se rattache à **plusieurs** écritures | groupe de 2 à 5 écritures de somme exacte, à l'unité et en lot (`journalEntryIds`, aucune route neuve) ; `matched_entry_id` = écriture principale, autres dans `bank_transaction_extra_entries` (**migration non breaking**, `CREATE TABLE` seul, `min_required` non relevé par elle — celui de la release reste celui de 38-2) ; définition unique d'« écriture rapprochée » sur un inventaire clos de lecteurs ; paire suggérée (±3 jours, somme exacte, sans écriture seule) comptée « déjà comptabilisée » ; annulation de tout le groupe ; codes `ENTRY_GROUP_SUM_MISMATCH`, raisons `entry_not_touching_account`, `entry_owned_by_document`, genre `uniquePair`, motif `several_pairs` | 38-6c (Écart 5 récrit, AC-4, AC-5, AC-9 à AC-11 étendus, AC-12 à AC-15, FR-38-6c-8 à -10, NFR-38-6c-5 et -6) ; E21, C-38-14, table des stories, migrations (sept fichiers), récapitulatif FR/NFR (91 FR, 58 NFR) |

- **Impact sur 38-9b** (fiche non éditée ici, à aligner) : l'écran du rattachement en lot montre les paires
  univoques à cocher (`uniquePair`, deux écritures et leurs mouvements) et les paires ambiguës à trancher ; le dialogue
  unitaire (`entry-candidates`) propose les `pairs` et permet de composer un groupe à la main (2 à 5 écritures, somme
  affichée contre le montant) ; libellés des codes neufs (`ENTRY_GROUP_SUM_MISMATCH`, raisons, `several_pairs`) aux
  quatre locales ; vue d'annulation d'un groupe (`groupEntryIds`, contre-passation de chaque écriture, refus dans un
  exercice clos) ; miroir de la règle de somme côté écran ; manuel (groupe, paire, fenêtre de 3 jours, écriture de
  frais mensuelle non couverte, limites) ; E2E « paire suggérée → cochée → rattachée ». B27 : aperçu d'une conversion
  avec la ligne de frais de la moitié créditée (écran des propositions).
- **Impact sur 38-6a** (sans édition) : `alreadyBooked` devient vrai pour une transaction à paire seule (lu par
  `count_already_booked_for_page`, 38-6c AC-14) — c'est voulu (E15 : pas de seconde écriture pour 2024).
- **Tracé pour plus tard** (C-38-13) : annulation d'un rattachement (simple ou groupe) dans un exercice clos —
  refusée, parce qu'elle contre-passe (v017-3a) ; un rattachement fait à tort en 2024 y reste.
- **Taille** : 38-6c grossit (règle neuve, inventaire de lecteurs) ; découpage 38-6c1/38-6c2 prévu si sa validation
  stagne (fiche, *Notes*). Les deux fiches sont à revalider. *(Annoté le 2026-11-15 : découpée à la P2, D-P2-3.)*

## Validation P2 — décisions et alignement inter-fiches (2026-11-15)

Liste : `implementation-artifacts/38-p2-inter-fiches.md`.

- **D-P2-1** — un seul code du refus d'une proposition non `provider` sur un compte prestataire,
  `PROVIDER_PROPOSAL_REQUIRED`, posé par 38-3a, repris par 38-6a (C-38-8).
- **D-P2-2** — sur un compte prestataire, `rule` et `split` restent refusés ; une facture n'est proposée que si
  `fee_amount = 0`, sinon la proposition `provider` ; règlement d'une facture au brut avec frais tracé (C-38-13).
- **D-P2-3 — 38-6c découpée** en **38-6c1** (AC-1 à AC-11 ; aucune migration) et **38-6c2** (AC-12 à AC-15 → AC-1 à
  AC-4, avec la migration `bank_transaction_extra_entries`) ; 38-6c `split` ; 38-6c2 après 38-6c1, avant 38-6a.
  Contre-passations d'un groupe : un seul nom, `reversalJournalEntryIds` (C-38-14).
- **D-P2-4** — O8 étendu au CHF (E11, C-38-3 : `PROVIDER_ACCOUNT_LEDGER_SHARED`, deux sens).
- **Alignements** de ce fichier : C-38-3 (`PROVIDER_ACCOUNT_HAS_TRANSACTIONS`, `PROVIDER_ACCOUNT_LEDGER_SHARED`),
  C-38-5 (`max_movements_per_page()`, `test()`), C-38-8, C-38-13, C-38-14, E11, E21, E22, O8 ; table des stories,
  dépendances, ordre, migrations (sept fichiers, 38-6c2 au lieu de 38-6c) ; § 38-9b (FR-38-9b-3 et -5 reprises des
  fiches) ; récapitulatif FR/NFR recompté aux fiches (91 FR, 59 NFR, 16 stories).

## Alignement sur les règles de travail à plusieurs (2026-11-16, Sonnet)

Sections « Release » et « Modules touchés » ajoutées ; ligne de l'Epic 36 (#823) en dépendance de 38-6c2 ; contrôle de
clôture du relevé joint passé de K9 à K10 (l'Epic 37 prend K9) ; PDF des manuels et CHANGELOG reportés à la release
(fiches : `.tex` seul, section *Note de version*) ; compteurs recomptés après chaque reprise de `main`.

## Point du 2026-11-16 — Epic 36 et règles de Guy

- **Nombre de stories** : Guy accepte que l'Epic 38 garde plus de stories que la borne de six (accord transmis par Beat
  le 2026-11-16). Le découpage reste celui des fiches, et l'epic sort en **une seule release** (« un epic = une
  release »).
- **38-6c2 et #823 (36-6)** : Guy annonce que la base de données et l'API de « une transaction, plusieurs écritures »
  sont faites. Au 2026-11-16, ce code n'est dans aucune branche publiée de `origin` (36-6 est encore `backlog` dans
  `epic/36-0.18.0`). Dès que la table et les routes sont visibles :
  - la 38-6c2 se récrit **sur ce modèle** et abandonne `bank_transaction_extra_entries` (B28), si Guy couvre les quatre
    besoins écrits dans le message à Guy : 2 à 5 écritures de somme exacte, rattachement en lot, API, annulation du
    groupe entier ;
  - elle se revalide ensuite (passes complètes, CLAUDE.md § *Review Iteration Rule*).

  Jusque-là, la 38-6c2 n'entre pas en développement.
- **Recoupements de la 0.18.0** : ils sont relevés au fichier de l'Epic 37 (§ *Recoupements avec l'Epic 36*).
  Ceux qui concernent l'Epic 38 :
  - 36-15 : le vocabulaire visible devient « compte analytique ». Il s'applique aux textes des écrans 38-9a et 38-9b, et
    à `default_project_id`, présenté comme le compte analytique d'une règle.
  - 36-10 : les dates sont celles de Zurich. Le téléchargement et ses fenêtres datées s'en servent déjà, par
    `kesh_core::horloge`.

## Point du 2026-11-17 — release 0.19.0 de Guy, règles de `main`, Epic 37 terminé

- **Epic 37 terminé** (six stories, gates et E2E verts), proposé par Beat pour la **0.19.0** (vendredi 2026-11-20),
  en second arrivé derrière l'Epic 43 de Guy ; intégration de `main` (v0.18.0) en cours dans `epic/37-multi-devises`.
- **#823 = Epic 43 de Guy**, priorité de la 0.19.0 : une transaction bancaire règle plusieurs factures, fournisseurs
  et clients. Intention (plan `release-0.19.0.md`, à confirmer à sa fiche) : lever ou remplacer
  `uq_invoice_settlements_entry`, ou une table de liaison transaction → règlements ; migration non breaking, la
  92ᵉ, arrêtée le mercredi 2026-11-19. **La 38-6c2 se récrit sur ce modèle** après la publication de la 0.19.0 :
  elle abandonne `bank_transaction_extra_entries` (B28) si l'Epic 43 couvre le groupe de 2 à 5 écritures de somme
  exacte, le lot, l'API et l'annulation du groupe ; sinon elle l'étend, avec Guy. Revalidation ensuite.
- **Base de l'epic** : `main` dit désormais « une branche d'epic n'est pas une base : un contributeur part de `main` ;
  bâtir sur une branche d'epic non publiée demande l'accord du Project Lead ». L'Epic 38 s'appuie sur l'Epic 37
  (devise du compte, cours, lignes en devise) : il **repartira de `main` après la publication de l'Epic 37**, ou
  avec l'accord de Guy avant. Le développement ne commence pas avant.
- **Release de l'Epic 38** : une seule (« un epic = une release »), après la 0.19.0. La 0.20.0 (2026-11-27) porte
  déjà une migration de Guy (note interne) ; la migration breaking de la 38-2 se numérote et s'annonce au projet
  GitHub avec Guy (Q1).
- **Montants** : la règle du montant ambigu (« 1.250 » refusé, apostrophe des milliers admise, 36-3 / 36-18a)
  s'applique aux champs de montant des écrans 38-9a et 38-9b (soldes de contrôle, relevés joints) : réemployer
  l'utilitaire commun.
- **Coordination** : un canal de courriel signé entre l'agent de Beat et celui de Guy est à l'essai depuis le
  2026-11-17 (`agent-kesh@lightning-multicom.com`) ; rien ne part sans l'accord de Beat.
