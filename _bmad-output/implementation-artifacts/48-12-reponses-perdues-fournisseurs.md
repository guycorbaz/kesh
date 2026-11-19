# Story 48-12 : Refus serveur des factures fournisseur en double, et complétion d'un document dont la réponse est perdue

Status: drafted (conception refondue le 2026-11-19 ; validation à reprendre ; `ready-for-dev` se pose à la clôture de la validation, par l'orchestrateur)

Issues dans le périmètre : **#975** (création : réponse perdue puis second clic = second brouillon) et **#985** (complétion d'un document importé annoncée en échec) — toutes deux `bug`, **P2**. Release 0.20.0, Epic 48.
**Reportés à la 0.21.0** (décisions de Guy du 2026-11-19), conservés plus bas **sans AC actifs** : **#986** (conflit de version à la modification d'une facture fournisseur) et le **lot comptes bancaires** (#984, archivage). **Contrairement aux versions précédentes de cette fiche, la correction de #975 est dans le serveur** : une règle anti-doublon sous verrou, sans migration, pour toute création et toute modification, quel que soit le chemin (écran, complétion d'un document importé, API par clé).

## Story

En tant que comptable qui tient des factures fournisseur réelles (dettes, lots de paiement pain.001),
je veux que Kesh **refuse d'enregistrer deux fois la même facture fournisseur** (même numéro chez le même fournisseur, ou même
date et même montant sans numéro à comparer), quel que soit le chemin par lequel elle arrive,
afin que mes dettes restent justes et qu'aucune facture ne soit payée deux fois, sans que je perde jamais ma saisie.

**Usage réel.** Guy saisit ses factures fournisseur et en tire des lots pain.001. Un doublon de brouillon, une fois validé,
double la dette (compte 2000) et peut être payé deux fois. Une réponse perdue suivie d'un second clic (#975) n'est qu'une cause
parmi d'autres (double saisie, document déjà saisi à la main, intégration par clé d'API) : la règle se pose donc **là où l'écriture
se fait**. Le correctif ne supprime jamais une saisie : le refus la laisse intacte et montre la facture existante.

**Conséquence à signaler (règle « numéro identique = refus absolu »).** Un fournisseur dont la numérotation **repart à un** chaque
année, ou qui réutilise un numéro, verra sa nouvelle facture refusée sans exception tant que l'ancienne n'est pas **annulée** ; la
voie de sortie est d'annuler l'ancienne (les annulées sont exclues) ou de saisir un numéro distinct (par exemple préfixé de l'année).

## Ce qui se lit au code

Tout est lu à `af4e0e313` (tête de la branche d'épique, 48-13 et 48-15 fusionnées).

1. **Aucun rejeu automatique d'une écriture.** `frontend/src/lib/shared/utils/api-client.ts:321-322` : `isIdempotentMethod` ne
   vaut vrai que pour `undefined`, `GET`, `HEAD`. Un `POST` ou un `PUT` dont la réponse se perd n'est jamais renvoyé par le
   client ; c'est l'utilisateur qui le renvoie, par un second clic. Délai : `REQUEST_TIMEOUT_MS = 30_000` (`api-client.ts:42`).
2. **#975, création.** `new/+page.svelte:148-149` : `save(req)` appelle `createSupplierInvoice(req)` (`POST /supplier-invoices`,
   `supplier-invoices.api.ts:54-58`). Le corps (`CreateSupplierInvoiceRequest`, `supplier_invoices.rs:337-349`) ne porte ni
   version ni clé. Le serveur crée un brouillon à chaque appel (`create_supplier_invoice`, `supplier_invoices.rs:731-752`).
   `SupplierInvoiceForm.svelte:465-474` attrape l'erreur et l'affiche sans rien relire ; `saving` retombe à faux
   (`:475`), le bouton se réactive (`:977`). Un `TIMEOUT` ou un `NETWORK_ERROR` après le commit donne donc un second
   brouillon au second clic. Aucun garde-fou serveur : `supplier_invoices` n'a pas d'unicité sur (société, fournisseur, numéro)
   (`migrations/20260628000001_supplier_invoices.sql`).
3. **#985, complétion.** `new/+page.svelte:149` : `completeImport(doc.id, req)` (`imported-supplier-invoices.api.ts:58-62`).
   Le serveur verrouille le document et refuse un second passage : `imported_supplier_invoices.rs:190-193`
   (`ImportNotPendingCompletion { current_status }`) ; son doc-comment le dit (`:126-127` : « un nouvel essai rend `409
   IMPORT_NOT_PENDING_COMPLETION`, et rien n'est perdu ni faussé »). Côté écran, `supplier-invoice-form.ts:484-490` traduit ce
   code en « Ce document a déjà été traité ou retiré de la liste. » : un échec, alors que la facture existe. Le document
   `completed` porte `supplierInvoiceId` (`imported-supplier-invoices.types.ts`, interface `ImportedSupplierInvoice`) et
   `listImported('completed')` existe (`imported-supplier-invoices.api.ts:50-52`).
4. **Le serveur n'a aucun contrôle de doublon.** `create_draft_in_tx` (`crates/kesh-db/src/repositories/supplier_invoices.rs:650`) enchaîne
   `check_draft_input_in_tx` (`:502` : lignes non vides, projet, fournisseur `is_supplier`, forme des lignes, comptes de charge), le calcul
   du total (`draft_total`), l'`INSERT` (`:657-679`), les lignes (`insert_lines`), la relecture et l'audit `supplier_invoice.created` (`:684-693`).
   `supplier_invoices` n'a **aucune unicité** sur (société, fournisseur, numéro) (`migrations/20260628000001_supplier_invoices.sql`). Les deux seules
   écritures d'un brouillon passent par `create_draft_in_tx` et `update_draft_in_tx` (`:756`) ; leurs appelants hors tests : `routes/supplier_invoices.rs:740` (création, enveloppée par
   `kesh_db::retry::retry_on_deadlock`), `:817-834` (modification : `lock_draft_in_tx` `FOR UPDATE` sur la facture puis `update_draft_in_tx`),
   `routes/imported_supplier_invoices.rs:203` (complétion : staging verrouillé, `new_from_document`, puis `create_draft_in_tx`) ; `kesh-seed/src/pieges.rs:613` appelle `create_validated`
   (données de démonstration). Le service MCP des factures fournisseur ne crée ni ne modifie (`services/factures_fournisseur.rs` : lecture).
5. **Un précédent exact : le refus 409 qu'on lève par un champ de confirmation.** `accounts::archive` reçoit `confirm_designated: bool`
   (`crates/kesh-db/src/repositories/accounts.rs:1249`) et rend `DbError::AccountDesignatedInSettings { … }` (`:1290-1292`, déclarée `kesh-db/src/errors.rs:1584`, code `:2488`) ;
   la route lit `confirm_designated` du corps (`routes/accounts.rs:225`, `:540`) ; `kesh-api/src/errors.rs:4428` la range dans un sous-match **exhaustif** sur `DbError`
   (« pas de `_ =>` », `:4228-4230`) ; l'API le documente (`docs/api-external.md`, `confirmDesignated`). La règle neuve suit ce patron.
6. **Verrous.** Aucun code ne prend `FOR UPDATE` sur `contacts` (`git grep -n "FOR UPDATE" -- crates/kesh-db/src/repositories | grep -i contacts` ne rend qu'un commentaire, `contacts.rs:562`) ;
   `check_draft_input_in_tx` est écrit **sans verrou** (doc-comment `:493`, C-v017-4-28) et la création ne prend que les verrous **partagés** des clés étrangères, d'où le rejeu sur interblocage
   (`retry_on_deadlock`, `retry_app_on_deadlock` pour le `PUT` et la complétion, C-v017-4-37). La règle devra donc **prendre un verrou exclusif** (D3) et le dire.
7. **Contrat de l'API.** `docs/api-external.md:659` (`POST /api/v1/supplier-invoices`, « inchangé »), `:1015` (complétion : corps de `POST /supplier-invoices`), `:1021` (validation des coordonnées de paiement) ;
   `docs/manual/fr/admin-manual.tex` : un paragraphe « Pour les intégrations » par version (`:2588`, `:2618`, `:2665`). Les actions d'audit se déclarent dans `kesh-api/src/audit_labels.rs:209-214`
   et se traduisent (`messages.ftl:3541-3548`, quatre locales) ; `audit_acteur_garde.rs:364`, `:370` suit `create_draft_in_tx` et `update_draft_in_tx`.
7. **Un décor d'E2E existe pour la réponse perdue.** `frontend/tests/e2e/supplier-invoice-unknown-outcome.spec.ts:1-12` : la
   coupure est réelle (`route.fetch()` envoie au serveur, puis `route.abort()` jette la réponse) ; un `abort()` seul n'écrirait rien.

## Inventaire par le symptôme

Symptôme : « une écriture dont la réponse peut se perdre se renvoie sans relecture, et crée un doublon (création) ou boucle sur un
409 / écrase (mise à jour) ». Commandes (rejouées à `af4e0e313` ; `sed` ne fait que raccourcir les chemins) :

```sh
# 1. créations appelées depuis un écran
git grep -nE "await create[A-Z][A-Za-z]*\(" -- 'frontend/src/routes' 'frontend/src/lib/*.svelte' 'frontend/src/lib/**/*.svelte' ':!*.test.ts'
# 2. mises à jour appelées depuis un écran
git grep -nE "await update[A-Z][A-Za-z]*\(" -- 'frontend/src/routes' 'frontend/src/lib/*.svelte' 'frontend/src/lib/**/*.svelte' ':!*.test.ts'
# 3. (V2-8, V3-3) écritures sous d'autres verbes, cherchées par le symptôme et non par le nom : tout appel direct à `apiClient`, hors tests et hors `*.api.ts`
git grep -nE "apiClient\.(post|put|patch|delete|postFormData)\b" -- 'frontend/src/**/*.svelte' 'frontend/src/**/*.ts' ':!*.test.ts' ':!*.api.ts'
# 4. écrans qui reconnaissent une issue inconnue (la sortie est ci-dessous)
git grep -lE "isUnknownOutcome|isSettlementOutcomeUnknown" -- frontend/src ':!*.test.ts'
```

La commande 3 rend sept appels directs (copiés) : `lib/app/stores/mode.svelte.ts:57` (`PUT /profile/mode`) et `lib/app/stores/session-idle.svelte.ts:373` (`POST /auth/activity`), qui **ne sont pas des écritures de livres** ; `routes/login/+page.svelte:56` (connexion) ; et `routes/(app)/users/+page.svelte:122` (`POST /users`), `:168` (`PUT /users/{id}`), `:209` (`PUT …/disable`), `:254` (`PUT …/reset-password`), classés ci-dessous. La commande 4 rend sept fichiers : `AutoBackupSection.svelte`, `AdminRestorePanel.svelte`, `mcp-sends-format.ts`, `unknown-outcome.ts`,
`invoices/[id]/+page.svelte`, `invoices/due-dates/+page.svelte`, `supplier-invoices/[id]/+page.svelte` : **aucun écran de création**.

Sortie de la commande 1 (25 sites, copiée) :

```
lib/components/invoices/InvoiceForm.svelte:732                 createInvoice
lib/features/bank-import/BankImportUpload.svelte:292           createBankImport
lib/features/bank-import/BankProfileForm.svelte:115            createBankProfile
lib/features/bank-import/ManualStatementForm.svelte:182        createManualStatement
lib/features/contacts/ContactPersonsManager.svelte:56          createContactPerson
lib/features/journal-entries/ConversionAssistant.svelte:266    createConversion
lib/features/journal-entries/JournalEntryForm.svelte:671       createJournalEntry
lib/features/mcp-prompts/McpPromptsList.svelte:67              createMcpPrompt
lib/features/mcp-server/McpKeysSection.svelte:152              createMcpKey
lib/features/open-items/OpenItemsScreen.svelte:322             createLettering
lib/features/reconciliation/rules/RuleFormModal.svelte:106     createRule
lib/features/supplier-invoices/SupplierCreatePanel.svelte:128  createContact
routes/(app)/accounts/+page.svelte:254                         createAccount
routes/(app)/bank-accounts/+page.svelte:510                    createBankAccount
routes/(app)/contacts/+page.svelte:539                         createContact
routes/(app)/invoices/[id]/+page.svelte:114                    createCreditNote
routes/(app)/payment-batches/+page.svelte:114                  createPaymentBatch
routes/(app)/products/+page.svelte:385                         createProduct
routes/(app)/settings/api-keys/+page.svelte:183                createApiKey
routes/(app)/settings/dunning/+page.svelte:111                 createDunningLevel
routes/(app)/settings/fiscal-years/+page.svelte:214            createFiscalYear
routes/(app)/settings/projects/+page.svelte:118                createProject
routes/(app)/settings/vat-rates/+page.svelte:121 et :153       createVatRate (deux)
routes/(app)/supplier-invoices/new/+page.svelte:149            createSupplierInvoice / completeImport
```

La commande 2 rend **26** sites ; ceux qui envoient une `version` ou bouclent sur un 409 sont classés ci-dessous.

### Dans la story

| Site | Symptôme | Lot |
|---|---|---|
| `createSupplierInvoice`, `new/+page.svelte:149` | doublon de brouillon (#975) | S puis E |
| `completeImport`, `new/+page.svelte:149` | échec annoncé à tort (#985) ; doublon refusé par la règle serveur | S puis E |
| `updateSupplierInvoice`, `[id]/edit/+page.svelte:130` | la **règle anti-doublon** s'applique aussi à la modification (D6) ; le **conflit de version** (#986) est reporté à la 0.21.0 | S ; #986 reporté |
| `updateBankAccount`, `bank-accounts/+page.svelte:575` (`edit-full`) | 409 sans relecture (#984) | reporté à la 0.21.0 |
| `archiveBankAccount`, `bank-accounts/+page.svelte:586-598` | 409 sans relecture, version périmée renvoyée (V1-6) | reporté à la 0.21.0 |

### Hors périmètre, couvert par une issue ouverte par l'orchestrateur (#1354 à #1362)

Correspondance (ordre de l'ouverture) : #1354 facture client, #1355 écriture, **#1356 lot de paiement** (rattachée à la 0.20.0 par Guy, hors de cette story), #1357 avoir, #1358 contact, #1359 compte bancaire, #1360 créations diverses, #1361 liaison compte bancaire (409), #1362 clé d'idempotence durable.

| Site | Symptôme | Issue (objet) |
|---|---|---|
| `createInvoice`, `InvoiceForm.svelte:732` (fichier de l'Epic 49) | doublon de facture client | facture client |
| `createJournalEntry`, `JournalEntryForm.svelte:671` | doublon d'écriture | écriture |
| `createPaymentBatch`, `payment-batches/+page.svelte:114` | doublon de lot de paiement | lot de paiement |
| `createCreditNote`, `invoices/[id]/+page.svelte:114` | doublon d'avoir | avoir |
| `createContact`, `SupplierCreatePanel.svelte:128`, `contacts/+page.svelte:539` | doublon de contact | contact |
| `createBankAccount`, `bank-accounts/+page.svelte:510` | doublon de compte (à vérifier : un IBAN doublon est peut-être refusé) | compte bancaire |
| `createAccount` `:254`, `createProduct` `:385`, `createProject` `:118`, `createFiscalYear` `:214`, `createBankProfile` `:115`, `createApiKey` `:183` | doublon (la plupart refusés par une unicité du schéma, à constater) | créations diverses |
| `updateBankAccountJournalLink`, `BankAccountJournalLinkForm.svelte:86` et `:111` | 409 en boucle sur la version de la prop | liaison compte bancaire (409) |
| angle mort de D2 | requête en vol au-delà de la relecture | clé d'idempotence durable (migration) |

### Hors périmètre, **absents de la table de la première rédaction** (issues ouvertes par l'orchestrateur)

| Site (fichier:ligne) | Symptôme | Gravité |
|---|---|---|
| `createConversion`, `ConversionAssistant.svelte:266` (`POST /journal-entries/conversions`) — **#1365** (rattachée à la 0.20.0 par Guy, hors de cette story) | un rejeu écrit **deux écritures au journal** de plus | **la plus élevée** (erreur comptable, candidat P1) |
| `createManualStatement`, `ManualStatementForm.svelte:182` — **#1366** | relevé saisi à la main doublé (transactions à rapprocher en double) | élevée, à constater (dédoublonnage éventuel) |
| `createBankImport`, `BankImportUpload.svelte:292` — **#1367** | réimport d'un relevé | à constater (le fichier est peut-être déjà refusé par son empreinte) |
| `createLettering`, `OpenItemsScreen.svelte:322` — **#1368** | lettrage rejoué | moyenne |
| `updateProject` (comptes analytiques), `settings/projects/+page.svelte:115` — **#1369** ; avec `version: mode.project.version` et un `catch` qui n'écrit que `formError` (`:123-126`) | **boucle 409** sur la version périmée, comme #984 | moyenne |
| `updateContact`, `contacts/+page.svelte:536` — **#1370** ; `updateProduct`, `products/+page.svelte:382` — **#1371** | le formulaire se ferme sur 409 : saisie perdue | moyenne (selon la lecture du validateur, à relire) |
| **#1372** : `createRule` `RuleFormModal.svelte:106`, `createDunningLevel` `dunning/+page.svelte:111`, `createVatRate` `vat-rates/+page.svelte:121`, `:153`, `createContactPerson` `ContactPersonsManager.svelte:56`, `createMcpKey` `McpKeysSection.svelte:152`, `createMcpPrompt` `McpPromptsList.svelte:67` | doublon de paramétrage | basse |
| autres `update*` de la commande 2 (`updateAccount`, `updateDunningLevel`, `updateFiscalYear`, `updateVatRate`, `updateRule`, `updateEmailTemplate`, `updateInvoiceSettings`…) **et** écritures sous d'autres verbes : `postRevaluation` (`RevaluationPanel.svelte:172` ; la réévaluation calcule un écart incrémental, `fx_revaluation.rs:317`), `generateOpeningBalances` (`settings/opening-balances/+page.svelte:410`), `sendReminderBatch` (`invoices/reminders/+page.svelte:155`) ; appels directs de la commande 3 dans `users/+page.svelte` : `:122` (création), `:168`, `:209` (désactivation), `:254` (réinitialisation du mot de passe) | **non examinés**, sondés seulement, aucun jugé défectueux | à constater : issue **#1373** |

La règle de D1 est **propre aux factures fournisseur** ; les sites ci-dessus relèvent d'issues distinctes.

## Modules et découpage

**Décompte au grain du fichier d'épique** (`main:…/epic-48-corrections-0-20-0.md:205`, `:210-214`), les modules **modifiés** :
1. `kesh-db` (repository `supplier_invoices.rs`, `errors.rs`, tests) ; 2. `kesh-api` (`routes/supplier_invoices.rs`, `errors.rs`, `audit_labels.rs`, tests) ;
3. `kesh-i18n` (quatre locales : messages des refus, libellé d'audit, textes de l'écran) ; 4. `frontend/features/supplier-invoices` ;
5. la route `supplier-invoices` (`new`, `[id]/edit`) ; 6. `features/imported-supplier-invoices` (`listImported` reçoit un `CallOptions`, D9) ;
7. la documentation : `docs/api-external.md`, manuel français (utilisateur et administration). **Sept modules, au-dessus du plafond de cinq : aucune dérogation.**
**Le découpage proposé** (au rapport, à décider par le Project Lead) suit les lots de la fiche : **48-12a (lot S, serveur)** : `kesh-db`, `kesh-api`, `kesh-i18n`, `docs/api-external.md` = **quatre**, livrable seul (l'API est protégée, l'écran affiche encore un message générique) ; **48-12b (lot E, écran)** : `features/supplier-invoices`, route `supplier-invoices`, `features/imported-supplier-invoices`, `kesh-i18n`, manuel français = **cinq**, après la 48-12a. La fiche est rangée en lots pour qu'il ne demande aucune réécriture.

## Décisions

### Lot S — la règle, dans le serveur

**D1. La règle anti-doublon (décisions de Guy du 2026-11-19).** Pour toute **création** et toute **modification** d'une facture fournisseur, sous verrou, sans migration :
1. **Même société, même fournisseur (`contactId`), même numéro de facture du fournisseur** (non vide, comparé après normalisation, D2) → **refus absolu** `409 SUPPLIER_INVOICE_DUPLICATE`, sans exception ni sortie ;
2. **Même société, même fournisseur, même date de facture, même total TTC** → **refus par défaut** `409 SUPPLIER_INVOICE_PROBABLE_DUPLICATE`, **avec une sortie explicite** : un champ de requête `confirmNotDuplicate: true` que l'écran n'envoie qu'après le clic sur « Ce n'est pas un doublon, enregistrer » ; la confirmation est **inscrite au journal d'audit** (D5) ;
3. **si les deux factures portent un numéro du fournisseur, seul le numéro décide** : deux numéros différents ne sont **jamais** un doublon, même date et même montant ; le critère 2 ne s'applique que si **l'une au moins** des deux n'a pas de numéro ;
4. les factures **annulées** (`status = 'cancelled'`) sont **exclues** de la comparaison ; brouillons, validées et payées comptent ;
5. le **numéro reste facultatif** ; les données existantes ne bougent pas : le contrôle ne vaut que pour une nouvelle saisie ou une modification (D6).
Les deux refus portent `details : { supplierInvoiceId, internalNumber, status }` (identifiant, numéro interne — `null` pour un brouillon jamais validé — et statut de **la facture existante** ; s'il y en a plusieurs, celle d'identifiant le plus petit).
*Écartée — relecture et signalement côté écran* (conception précédente) : elle ne protège ni la double saisie, ni la complétion, ni l'API, et laisse une fenêtre quand la requête est encore en vol. *Écartée — clé d'idempotence durable* : migration, exclue de la release. *Écartée — contrainte `UNIQUE` en base* : migration, et elle ne porterait pas la règle 2.

**D2. Normalisation et comparaison.** Une fonction **pure** de `kesh-db` (testée seule) normalise le numéro : tous les blancs retirés (`char::is_whitespace`, plus U+200B et U+FEFF, l'ensemble de `is_payment_blank`, `routes/supplier_invoices.rs:378`), puis `to_lowercase()` ; vide après normalisation = **absent**. Le total comparé est le **total TTC calculé** (`draft_total`) du côté nouveau, et `total_amount` stocké du côté existant, en égalité de valeur décimale. La comparaison se fait **en Rust** sur les factures **du même fournisseur** (`SELECT id, internal_number, supplier_invoice_number, status, invoice_date, total_amount FROM supplier_invoices WHERE company_id = ? AND contact_id = ? AND status <> 'cancelled'` et, à la modification, `AND id <> ?`) : une normalisation SQL des blancs Unicode serait fragile, et le nombre de factures d'un fournisseur reste petit. Ordre de décision : **d'abord** le refus absolu (règle 1) sur toutes les candidates, **puis** le refus probable (règle 2, avec la règle 3) ; la facture citée est la plus petite en identifiant parmi celles du même refus.

**D3. Verrou.** Avant la comparaison, la transaction prend `SELECT id FROM contacts WHERE id = ? AND company_id = ? FOR UPDATE` sur **la ligne du fournisseur** : deux envois simultanés pour le même fournisseur se **sérialisent**, le second voit la facture du premier à la comparaison et est refusé. Le rejeu sur interblocage des trois chemins existe déjà (point 6 du code). L'ordre des verrous est écrit au doc-comment : **création** : fournisseur puis `INSERT` ; **modification** : facture (`lock_draft_in_tx`) puis fournisseur ; **complétion** : staging puis fournisseur. Un test de concurrence (deux créations simultanées, un seul succès) et un test sans verrou (mutation) le prouvent. *Écartée — verrou sur la société (`companies`)* : sérialise toutes les créations de la société. *Écartée — pas de verrou* : deux envois simultanés passeraient (« TOCTOU »).

**D4. Ordre des refus.** La comparaison se fait **après** `check_draft_input_in_tx` (lignes, projet, fournisseur, forme, comptes : les refus de saisie passent d'abord, et le total n'est calculable qu'après) et **avant** l'`INSERT`. Pour la complétion, `ImportNotPendingCompletion` (statut du staging), puis les règles « document » (`new_from_document`), puis cette règle : un document déjà complété rend toujours `409 IMPORT_NOT_PENDING_COMPLETION`, jamais un refus de doublon (D9). Un refus **annule** la transaction : le document reste `to_complete`, rien n'est écrit.

**D5. Confirmation et audit.** `CreateSupplierInvoiceRequest` (donc le corps du `PUT`, qui l'aplatit, `routes/supplier_invoices.rs:654-658`, et celui de la complétion) reçoit `confirm_not_duplicate: bool` (`#[serde(default)]`, `confirmNotDuplicate`), transmis au repository sous la forme d'un paramètre explicite (patron `confirm_designated`, `DuplicatePolicy { Refuse, Confirmed }`) : `create_draft_in_tx` et `update_draft_in_tx` gardent leur nom, reçoivent la politique, et les appelants de tests et de démonstration passent `Refuse`. La confirmation **ne lève jamais le refus absolu** (règle 1). Quand la règle 2 **aurait refusé** (une facture existante répond au critère) et que `Confirmed` la laisse passer, le repository écrit, **dans la même transaction**, une entrée d'audit `supplier_invoice.duplicate_confirmed` (acteur, facture créée ou modifiée, `details : { supplierInvoiceId, internalNumber }` de la facture existante qui aurait refusé). Elle se déclare dans `audit_labels.rs` et se traduit dans les quatre catalogues (`audit-log-action-supplier-invoice-duplicate-confirmed`). Un `confirmNotDuplicate: true` sans doublon probable n'écrit **rien** de plus.

**D6. À la modification.** La règle ne se rejoue que si l'un des quatre éléments qui la fondent **change** par rapport à la facture avant modification : **fournisseur, numéro (normalisé), date de facture, total TTC**. Sans changement de ces quatre, la modification passe (corriger une description, une échéance ou un IBAN d'une facture qui était déjà, avant 0.20.0, le double d'une autre ne doit pas être bloqué : « les données existantes ne bougent pas »). La facture modifiée est exclue de la comparaison (`id <> ?`). Même verrou (D3).

**D7. Contrat de l'API (changement à signaler).** Deux codes neufs, `409 SUPPLIER_INVOICE_DUPLICATE` et `409 SUPPLIER_INVOICE_PROBABLE_DUPLICATE`, avec `details` ; un champ de requête neuf, facultatif, `confirmNotDuplicate`. Une intégration qui créait deux fois la même facture **reçoit désormais un refus**. Documenté dans `docs/api-external.md` (`POST` et `PUT /supplier-invoices`, complétion `POST /imported-supplier-invoices/{id}/complete`, au même endroit que « Les coordonnées de paiement se valident à un seul endroit »), dans le paragraphe « Pour les intégrations » du manuel d'administration, et à la **Note de version** (« changement de contrat »). Messages (clés `error-supplier-invoice-duplicate`, `error-supplier-invoice-probable-duplicate`, quatre locales) : une phrase pour l'utilisateur, dans la langue de l'installation, sans le numéro de la facture existante dans le texte (il est dans `details`).

### Lot E — l'écran, et la complétion

**D8. Le refus à l'écran.** `describeSubmitError` (`supplier-invoice-form.ts`) reconnaît les deux codes et le formulaire (`SupplierInvoiceForm.svelte`) affiche, **saisie intacte**, un avis qui **montre la facture existante** (numéro interne s'il existe, sinon « brouillon n° … » par son identifiant ; **lien vers sa fiche dans un nouvel onglet**, `target="_blank" rel="noopener"`). Pour `SUPPLIER_INVOICE_PROBABLE_DUPLICATE`, l'avis porte le bouton **« Ce n'est pas un doublon, enregistrer »** : il renvoie la même saisie avec `confirmNotDuplicate: true`, **une seule fois** ; toute modification du fournisseur, du numéro, de la date ou d'une ligne **efface l'avis** (la confirmation ne vaut que pour la saisie qui a été montrée). Pour `SUPPLIER_INVOICE_DUPLICATE`, **aucun bouton** : l'utilisateur corrige le numéro ou ouvre la facture existante. **La réponse perdue suivie d'un second clic aboutit à ce refus** : c'est la correction de #975, sans relecture ni verdict côté écran. Valable pour la création, la modification (`[id]/edit`) et la complétion (même formulaire).

**D9 (#985). Sur `IMPORT_NOT_PENDING_COMPLETION` ou issue inconnue, relire le document : quatre issues (V1-3).**
Relire `listImported('completed')` puis, au besoin, `listImported('to_complete')` (aucun n'est paginé ; `listImported` reçoit un paramètre `CallOptions` et la relecture passe `{ retry: false }`, V2-4) :
- *complété avec facture* (`supplierInvoiceId` renseigné) : **avis**, non erreur : « Ce document a déjà été traité : la facture a
  été enregistrée » (la relecture ne lit que le document : la facture peut avoir été validée depuis, le lien montre son état) avec le lien `/supplier-invoices/<id>`, saisie en place, envoi désactivé ;
- *encore à traiter* : « Aucune facture n'a été enregistrée : vous pouvez réessayer », bouton actif ;
- *retiré* (ni l'un ni l'autre, relecture réussie) ou 409 dont le document est introuvable : le message actuel
  (`supplier-invoices-err-not-pending`) ;
- *relecture en échec* (détectée explicitement) : message d'issue inconnue, **jamais** « déjà traité » ;
- *document `completed` dont `supplierInvoiceId` est nul* (V4-5 ; pratiquement inatteignable : `delete_draft` détache le document, `ON DELETE SET NULL`) : message d'issue inconnue, jamais le lien.
L'avis est neutre sur l'auteur. *Angle mort assumé (V1-18)* : un document complété puis **détaché** par un tiers (il redevient
`to_complete`, `supplier_invoice_id` à `NULL`) se complète de nouveau : une complétion perdue, suivie d'un détachement puis d'un
renvoi, crée une seconde facture. Rare ; écrit, non traité.

**D9 (suite, simplification).** Avec la règle serveur, D9 ne garde que ce qui reste vrai : le second clic d'une complétion dont la réponse s'est perdue rend `IMPORT_NOT_PENDING_COMPLETION` (le statut du staging se juge avant la règle de doublon, D4) ; l'écran **relit le document** pour dire la vérité. Les issues de D9 ci-dessus restent ; aucune relecture des brouillons n'est faite.

**D10. Garde synchrone contre le double envoi (écran).** `SupplierInvoiceForm.submit` (`:376`) revient d'emblée si `saving` : sans elle, deux événements `submit` émis avant le rendu qui désactive le bouton envoient deux requêtes dont la seconde serait **refusée** (doublon) mais produirait un avis trompeur ; le bouton de confirmation porte la même garde.

**D11. Où vit la logique.** **Serveur** : la règle et la normalisation dans `crates/kesh-db/src/repositories/supplier_invoices.rs` (ou un module voisin pur), les variantes `DbError::SupplierInvoiceDuplicate { … }` et `SupplierInvoiceProbableDuplicate { … }` (`errors.rs`, code `:2449`-`:2488`), leur rangement dans le sous-match exhaustif de `kesh-api/src/errors.rs` (autour de `:4428`). **Écran** : `supplier-invoice-form.ts` (cas des deux codes), un petit composant d'avis dans `features/supplier-invoices/`.

## Critères d'acceptation

**Lot S (serveur)**
1. **AC-S1 (règle 1).** Une création, une modification ou une complétion dont le fournisseur et le numéro normalisé (blancs retirés, casse ignorée) sont ceux d'une facture non annulée de la société rend `409 SUPPLIER_INVOICE_DUPLICATE`, `details.supplierInvoiceId` / `internalNumber` / `status` de l'existante ; **rien n'est écrit** ; `confirmNotDuplicate: true` n'y change rien.
2. **AC-S2 (règle 2).** Sans numéro à comparer (l'un au moins des deux n'en a pas), même fournisseur, même date, même total TTC : `409 SUPPLIER_INVOICE_PROBABLE_DUPLICATE` ; avec `confirmNotDuplicate: true` la facture s'enregistre et **une entrée d'audit `supplier_invoice.duplicate_confirmed`** est écrite dans la même transaction.
3. **AC-S3 (règle 3).** Deux factures portant **chacune** un numéro différent ne sont jamais un doublon, même date et même montant.
4. **AC-S4 (règle 4).** Une facture annulée ne compte pas : on recrée une facture au numéro d'une annulée, ou au même date/total.
5. **AC-S5 (règle 5 et D6).** Le numéro reste facultatif ; une modification qui ne change ni le fournisseur, ni le numéro, ni la date, ni le total passe même si la facture était déjà le double d'une autre ; une modification exclut la facture elle-même de la comparaison.
6. **AC-S6 (concurrence, D3).** Deux créations simultanées de la même facture : **une seule** réussit, l'autre rend `SUPPLIER_INVOICE_DUPLICATE` (ou `PROBABLE_` selon le cas) ; aucun interblocage ne remonte (rejeu existant).
7. **AC-S7 (ordre, D4).** Un refus de saisie (ligne, compte, projet, fournisseur non fournisseur) passe avant le refus de doublon ; pour la complétion, un document déjà complété rend `IMPORT_NOT_PENDING_COMPLETION` ; un refus de doublon laisse le document `to_complete`.
8. **AC-S8 (chemins).** La règle vaut par l'écran, la complétion et une **clé d'API `read-write`** (`POST` et `PUT /supplier-invoices`, `POST …/complete`) ; les données de démonstration (`kesh-seed`) et les tests existants **ne dépendent pas d'un doublon** (fixtures corrigées, jamais la règle).
9. **AC-S9 (contrat).** `docs/api-external.md` décrit les deux codes, `details` et `confirmNotDuplicate` ; le manuel d'administration, « Pour les intégrations », le signale ; la Note de version dit « changement de contrat ». Aucune migration ; aucune facture client, aucun avoir, aucun PDF touché.

**Lot E (écran et complétion)**
10. **AC-E1 (#975).** Création : réponse perdue après l'écriture, puis second clic : l'écran montre l'avis de doublon (lien vers la facture existante, nouvel onglet), la saisie est intacte, **un seul** brouillon existe.
11. **AC-E2.** Refus probable : bouton « Ce n'est pas un doublon, enregistrer » qui renvoie une fois avec `confirmNotDuplicate: true` ; toute modification de la saisie efface l'avis ; refus absolu : aucun bouton.
12. **AC-E3 (#985).** Complétion perdue : document complété = avis et lien, envoi désactivé ; encore à traiter = « aucune facture enregistrée, réessayez » ; retiré = message actuel ; relecture en échec = message d'issue inconnue ; document complété sans `supplierInvoiceId` = message d'issue inconnue.
13. **AC-E4.** Deux événements `submit` consécutifs n'envoient qu'une requête ; le bouton de confirmation porte la même garde.
14. **AC-E5.** Clés neuves dans les quatre locales (messages d'erreur, libellé d'audit, avis de l'écran) ; `sitesTotal` et les gardes de repli recomptés depuis la source.
15. **AC-E6 (manuel).** `docs/manual/fr/user-manual.tex` (seul manuel utilisateur existant) décrit la règle (numéro identique refusé, date et montant identiques à confirmer, annulées exclues, l'exemple de la numérotation qui repart à un) ; le contrôle final vérifie **positivement** « Ce n'est pas un doublon » dans le `.tex` et le PDF aplati après `make fr` (non commité).

## Tests à écrire, et la preuve du rouge

Chaque test neuf rougit sans le correctif ; journaux dans `/home/gcorbaz/devel/kesh-gate-logs/e4812/`. **Rouge d'abord** : chaque test est écrit et joué rouge avant le code, ou rougi par mutation.

| Test | Fichier | Mutation qui doit le rougir |
|---|---|---|
| normalisation : `"FA 2026-1"`, `"fa2026-1"`, `"FA\u{A0}2026-1"`, `" FA2026-1\u{200B}"` égales ; vide ou blancs seuls = absent | unitaire `kesh-db` (fonction pure) | retirer `to_lowercase`, ne retirer que l'espace ASCII, ne pas retirer U+200B |
| règle 1 : même fournisseur et même numéro → refus ; `details` exacts ; `confirm` sans effet ; autre fournisseur ou autre société → accepté | `kesh-db/tests/supplier_invoice_duplicate.rs` (neuf) | omettre le filtre `contact_id`, omettre le filtre société, laisser `Confirmed` lever la règle 1 |
| règle 2 : sans numéro, même date et même total → refus probable ; `Confirmed` passe et écrit `supplier_invoice.duplicate_confirmed` ; total ou date différents → accepté | idem | omettre le total, omettre la date, ne pas écrire l'audit |
| règle 3 : deux numéros différents, même date et même total → accepté ; un numéro et un absent, même date et total → refus probable | idem | « le numéro ne décide pas » (appliquer la règle 2 aux deux numéros) |
| règle 4 : annulée exclue (numéro et date/total) ; brouillon, validée, payée comptent | idem | retirer `status <> 'cancelled'` |
| modification : exclut la facture elle-même ; changement de fournisseur, de numéro, de date, de total rejoue la règle ; aucun changement des quatre = passe même si doublon préexistant | idem | retirer `id <> ?`, rejouer toujours la règle |
| concurrence : deux `tokio::spawn` de `create_draft` identiques → un `Ok`, un refus | idem | retirer le `FOR UPDATE` du fournisseur |
| ordre : refus de saisie (compte non imputable) avant le refus de doublon ; refus de doublon ne laisse aucune ligne | idem | comparer avant `check_draft_input_in_tx` |
| API : `POST` deux fois → `201` puis `409` (codes, `details`) ; avec `confirmNotDuplicate` ; `PUT` ; clé `read-write` ; complétion : doublon → `409`, document `to_complete` ; second passage → `IMPORT_NOT_PENDING_COMPLETION` | `kesh-api/tests/supplier_invoice_duplicate_e2e.rs` (neuf) | idem ; ignorer le champ `confirmNotDuplicate` du corps |
| audit : `supplier_invoice.duplicate_confirmed` listée (`audit_labels.rs`), traduite quatre locales, garde `audit_acteur_garde.rs` à jour | gardes existantes | retirer la traduction d'une locale |
| écran : `describeSubmitError` des deux codes ; avis + lien `target="_blank"` ; bouton (probable seulement) envoie une fois avec le drapeau ; une modification efface l'avis ; double `submit` synchrone | `supplier-invoice-form.test.ts`, `SupplierInvoiceForm.test.ts`, `new-page.test.ts` | pas de garde ; drapeau envoyé sans clic ; avis conservé après modification |
| complétion (D9) : 409 → avis + lien ; encore à traiter → réessayer ; retiré → message actuel ; relecture en échec ; complété sans `supplierInvoiceId` | `new-page.test.ts` | traiter « complété sans `supplierInvoiceId` » comme complété, « déjà traité » sur relecture en échec |
| **E2E** : `route.fetch()` + `route.abort()` sur `POST /api/v1/supplier-invoices` (numéro saisi), second clic → avis de doublon avec lien, un seul brouillon (contrôle par l'API) ; sans numéro, doublon probable puis confirmation → deux brouillons ; coupure de `POST …/complete` → avis « déjà traité » ou « réessayez » | `supplier-invoice-duplicate.spec.ts` (neuf) | aujourd'hui deux brouillons |

E2E exigé : défaut visible dans le navigateur et à la frontière HTTP ; **filtrer `route.request().method()` et ne couper qu'une fois** ; précédent `supplier-invoice-unknown-outcome.spec.ts:163`, `:283`. Recette visuelle (`CLAUDE.md`) : captures, thème clair, de chaque état neuf (avis absolu, avis probable avec bouton, avis de complétion) dans `kesh-gate-logs/e4812/captures/`. Chaque mutation est suivie d'un `touch` du fichier restauré (cargo garde le binaire muté).

**Étape 0 obligatoire** : jouer `scripts/test-fast.sh --no-lint` **avant** le code pour relever les tests existants qui créent deux factures au même fournisseur avec le même numéro ou la même date et le même total (`kesh-db/tests/supplier_invoices_*`, `supplier_invoice*e2e`, `kesh-seed/src/pieges.rs:613`) ; leurs **fixtures** se corrigent (numéros distincts, ou `Confirmed` pour la règle 2) ; la règle 1 n'a pas de sortie.

## Tâches

**Lot S.** S1 : `DbError` (deux variantes, codes, `errors.rs` de `kesh-db`) et sous-match exhaustif de `kesh-api/src/errors.rs`. S2 : fonction pure de normalisation et de comparaison ; `DuplicatePolicy` ; contrôle sous verrou dans `create_draft_in_tx` et `update_draft_in_tx` (D2 à D6) ; audit `supplier_invoice.duplicate_confirmed`. S3 : `routes/supplier_invoices.rs` (champ `confirm_not_duplicate`, transmission à la création et au `PUT`) et `routes/imported_supplier_invoices.rs` (complétion). S4 : `audit_labels.rs`, catalogues (quatre locales : messages d'erreur, libellé d'audit). S5 : `docs/api-external.md` et manuel d'administration (« Pour les intégrations »). S6 : tests Rust et correction des fixtures.
**Lot E.** E1 : `supplier-invoice-form.ts`, `SupplierInvoiceForm.svelte`, composant d'avis, D8 et D10 ; `new/+page.svelte` (D3, `listImported` reçoit `CallOptions`, `{ retry: false }` sur la relecture). E2 : catalogues de l'écran, `sitesTotal`, gardes de repli. E3 : manuel utilisateur français. E4 : E2E et captures.
**Communs.** C0 (étape 0, ci-dessus). C1 : gates (voir « Gates »). C2 : Dev Agent Record, Change Log, Note de version.

## Manuels et documentation

Seul `docs/manual/fr/user-manual.tex` existe pour l'utilisateur (`docs/manual/{de,it,en}/` ne portent qu'un `README.md`, « traductions non encore créées ») : **aucun équivalent DE/IT/EN à écrire**. Les ajouts portent « depuis la version~0.20.0 » (`docs-numeros-de-release`).
- **Manuel utilisateur** (`user-manual.tex`, saisie `:2072-2090`, brouillon `:2195`, complétion `:2682`) : décrire la règle (numéro du fournisseur identique = refusé sans exception ; même date et même montant sans numéro à comparer = refusé, avec le bouton « Ce n'est pas un doublon, enregistrer » ; annulées exclues ; l'exemple de la numérotation qui repart à un et ses deux sorties).
- **Manuel d'administration** (`admin-manual.tex`, un paragraphe « Pour les intégrations » par version, p. ex. `:2588`, `:2618`, `:2665`) : le changement de contrat de l'API (deux codes, `details`, `confirmNotDuplicate`), à la section de la 0.20.0 (à créer si elle n'existe pas encore, avec l'orchestrateur).
- **`docs/api-external.md`** (`:659`, `:1015`, `:1021`) : voir D7. Les PDF ne se commitent pas.

## Gates

Selon `e0190-dev.md` et `e35-dev-commun.md` § *Quels gates, quand*. **Cette story modifie du Rust et des catalogues** : gate Rust complet.
- Entre les passes : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run -E 'binary(<le binaire touché>)'` ; `npm --prefix frontend run check`, `lint-i18n-ownership`, vitest ciblé.
- **Exception `kesh-db`** (`CLAUDE.md`) : dès qu'un patch touche un repository, **gate complet même en cours de boucle** ; base remise à zéro avant tout gate complet.
- **Au passage en `review`** : `scripts/test-fast.sh --ci`, puis `scripts/gate-ci.sh`, plus `npm run check`, `lint-i18n-ownership`, `test:unit`, `build`.
- **À la clôture** : `gate-ci` et **E2E complet** au dernier commit de code (montage `docs/testing.md`, `KESH_COOKIE_SECURE=false`, `KESH_TEST_MODE=true` des deux côtés) ; « vert » = les seuls échecs attendus.
Sous `scripts/mem-guard.sh`, une commande lourde à la fois, `flock` du verrou de la voie (base et Rust).

## Note de version

**Changement de contrat de l'API.** Kesh refuse d'enregistrer deux fois la même facture fournisseur : même fournisseur et même numéro (comparé sans blancs ni casse) = refus `409 SUPPLIER_INVOICE_DUPLICATE`, sans exception ; même fournisseur, même date et même total sans numéro à comparer = refus `409 SUPPLIER_INVOICE_PROBABLE_DUPLICATE`, que l'écran lève par « Ce n'est pas un doublon, enregistrer » et qu'une intégration lève par `confirmNotDuplicate: true` (inscrit au journal d'audit) ; les factures annulées ne comptent pas. La règle vaut à la création, à la modification et à la complétion d'un document, et l'écran montre la facture existante. Une complétion dont la réponse s'est perdue dit désormais la vérité au lieu d'annoncer un échec (refs #975, refs #985).

## Reportés à la 0.21.0 — sans AC actifs

Décisions de Guy du 2026-11-19. Ce qui suit **ne fait pas partie du périmètre** ; il est conservé pour que #986 et le lot comptes bancaires reviennent sans réécriture. Les renvois « D… » de cette section (D1-bis, D4, D5, D6, D7) visent **la numérotation d'avant la refonte**, non celle des décisions actives ci-dessus. Les conceptions écrites ci-dessous (conflit côte à côte, D1-bis, D5) **ont reçu des findings de la passe P6** qui ne sont **pas traités** : à reprendre avant toute réintégration.

**Findings P6 à traiter à la réintégration** (`kesh-gate-logs/e4812/validation-p6.md`, 0/1/2/3, troisième signal « recycle » dans la règle de conflit) : V6-1 (HIGH) la base remise à `stored` sur conflit rend `unchanged` au clic ordinaire suivant et adopte la version fraîche sans clic ; V6-2 la source de « Garder ma version » et la garde des deux boutons ne sont pas définies (la page n'a ni la saisie courante ni `saving`) ; V6-3 `status ≠ draft` jette la saisie en renvoyant à la fiche ; V6-4 « Reprendre la version enregistrée » destructif sans avertissement, repli `supplier-invoices-err-stale` mort ; V6-5 mutations manquantes pour `ILLEGAL_STATE_TRANSITION`, 404, `unchanged` ; V6-6 fichier d'épique à aligner. La conception de #986 est à reprendre **sur la base de la règle serveur de cette fiche** (qui réduit déjà les cas de 409 dus à un doublon).

### #986 — conflit de version à la modification (conception écrite avant la refonte)

**Lecture du code** (point 4) :

4. **#986, édition.** `[id]/edit/+page.svelte:128-132` : `updateSupplierInvoice(id, { ...req, version: invoice.version })`.
   `invoice.version` n'est jamais relu après l'envoi. Une réponse perdue après un `PUT` réussi, puis un second clic, part avec la
   version d'avant : `409 OPTIMISTIC_LOCK_CONFLICT` (`supplier_invoices.rs:772-808`, `lock_draft_in_tx` avec `req.version`).
   `supplier-invoice-form.ts:440-448` dit « modifiée entre-temps : rechargez la page, puis refaites votre correction » : faux
   quand la modification est précisément celle qu'on a envoyée, et recharger fait perdre la saisie.

**D1-bis (comparateur du conflit, #986). Règle de comparaison (V1-5).** Un jugement compare `stored` (relu) à la saisie **sous la forme que le serveur
range**, jamais littéralement : montants, quantités et taux comparés en décimal (`Big.eq`, quatre décimales : `"100.0000"` égale
`"100.00"`) ; IBAN, QR-IBAN et référence normalisés comme le serveur (`normalizeIban`, `supplier-invoice-form.ts:334` ; blancs,
insécables compris, retirés) ; `trim`, et vide ⇔ `null`. **La référence de paiement (V2-6, V3-2)** se normalise comme le serveur (`supplier_invoices.rs:458-473`) : avec un QR-IBAN, tous les blancs sont retirés (`normalize_payment_field`) ; sans, elle est **rognée aux deux bouts par l'ensemble de blancs du serveur** (`trim_matches(is_payment_blank)`, `:471` ; `is_payment_blank` définie à `:378` : `char::is_whitespace`, dont U+0085, plus U+200B et U+FEFF), puis **vide après normalisation ⇔ `null`** (`present`, `:384`). `String.prototype.trim` ne retire ni U+200B ni U+0085 : une **fonction pure** de `resend-guard.ts` reproduit cet ensemble, testée avec un U+200B final collé. **Les lignes (V2-2, V3-1)** : l'égalité exige d'abord **le même nombre de lignes**, puis, **position par position, chaque ligne égale sur ses cinq champs** (description, quantité, prix unitaire, taux de TVA, compte de charge), chacun normalisé, puis **l'en-tête, dont les champs sont énumérés (V4-1)** : les neuf du corps du `PUT` — `contactId`, `supplierInvoiceNumber` (`trim`, vide ⇔ `null`), `invoiceDate`, `dueDate` (vide ⇔ `null`), `projectId`, et, **seulement sans QR verrouillé**, `creditorIban`, `creditorQrIban` (`normalizeIban`), `paymentReference` (règle ci-dessus), `expectedPaymentAmount` (décimal, vide ⇔ `null`, `Big.eq`) ; **omettre l'un d'eux jetterait une correction non écrite** (IBAN et montant attendu partent au lot pain.001). Une ligne **supprimée** ou **ajoutée** ne rend donc jamais *déjà enregistré* (le serveur remplace toutes les lignes, `repositories/supplier_invoices.rs:802`, `:585-599`). Le total, dérivé, ne sert qu'à l'**affichage** de l'avis. **Le côté « saisie » (V3-5)** est le **corps tel qu'il est parti** (le même objet que le `PUT`, issu de `linesForServer` et `paymentFields`, `supplier-invoice-form.ts:203-212`, `:328-345`, et `fNumber.trim() || null`), jamais le formulaire brut (« 12,50 », « 1'250 »), sur lequel `Big` lèverait ou différerait ; le côté « base » est la **réponse du serveur**, non les champs rouverts par `formLineFromResponse`. Le cas « je viens d'enregistrer moi-même » (réponse perdue, puis second clic) **doit** rendre
*déjà enregistré*, jamais *conflit* : un test le fixe avec les formes du serveur (`"100.0000"`, IBAN sans blancs), et
une mutation retire le normaliseur. Ces fonctions sont pures et testées à part (D7).

**D4 (#986, conflit sans fusion). Relire la facture, juger, et en cas de conflit montrer les deux versions côte à côte : aucune reprise sans clic.**
Sur un 409 de version, un **`409 ILLEGAL_STATE_TRANSITION`** (le brouillon n'en est plus un ; `describeSubmitError` n'a aujourd'hui aucun
cas pour lui) ou une issue inconnue du `PUT`, la page relit `getSupplierInvoice(id, { retry: false })` et juge par une fonction
pure (D7, comparaison de D1-bis, verdicts nommés `already-saved` / `unchanged` / `conflict`, **sans réutiliser `kind` de
`companyEditConflict`**). **Il n'y a plus aucune fusion automatique** (décision de Guy du 2026-11-19, qui règle V5-1 et V5-2) :
- **`stored.status !== 'draft'`** : un collègue a validé entre-temps (`lock_draft_in_tx` refuse le statut avant la version,
  `repositories/supplier_invoices.rs:719` puis `:725` : cette branche vient d'un `ILLEGAL_STATE_TRANSITION` ou d'une issue
  inconnue, jamais d'un 409 de version) ; renvoi à la fiche avec `supplier-invoices-edit-not-draft`, sans comparaison ;
- **relecture qui rend 404** (brouillon supprimé) : message propre (« Cette facture n'existe plus »), non « rechargez » ;
- **`already-saved`** (même nombre de lignes, chacune des lignes sur ses cinq champs, et les neuf champs d'en-tête égalent le corps
  envoyé, D1-bis) : succès, notification, retour à la fiche ;
- **`unchanged`** (`stored` égale la copie de base, mêmes règles d'égalité, hors version) : « rechargé, vous pouvez enregistrer de
  nouveau », version fraîche adoptée (aucun champ ne change) ;
- **`conflict`** (tout le reste) : la page affiche **côte à côte « Votre version » et « La version enregistrée »** — **en-tête et
  lignes**, les champs en désaccord **marqués** (les neuf champs d'en-tête, et chaque ligne de chaque côté, position par position, avec ses cinq champs) —
  et propose **deux choix explicites** :
  - **« Garder ma version »** : le formulaire est renvoyé **avec la version relue** (c'est ce clic seul qui l'adopte) ;
  - **« Reprendre la version enregistrée »** : le formulaire **prend la version relue** (il est remonté sur `initial={stored}`, car `SupplierInvoiceForm` ne lit `initial` qu'une fois), avec la version relue.
  **Rien n'est repris sans clic** : tant que l'utilisateur n'a pas choisi, le formulaire garde ses valeurs et la version reste celle d'avant. Les coordonnées de paiement se montrent et se comparent en un seul groupe (IBAN ou QR-IBAN, référence) : aucune reprise champ par champ n'existe, donc aucun état incohérent à construire (`BOTH_IBANS`). Les deux boutons portent la garde de D6.
- **relecture en échec** : « Conflit de version et rechargement impossible — rechargez la page », **aucun jugement, aucune version adoptée** (la base n'est pas fiable ; le scénario « envoi écrit mais réponse perdue, relecture en échec, champ rétabli à sa valeur de base, 409 » aboutit à un *conflit* montré, jamais à une reprise silencieuse).
**Copie de base** : prise à l'ouverture, remise à `stored` à chaque relecture réussie. **Relecture** : `{ retry: false }`.
**Texte de repli** : la branche `OPTIMISTIC_LOCK_CONFLICT` de `describeSubmitError` (`supplier-invoice-form.ts:440-448`, clé `supplier-invoices-err-stale`) est **gardée comme repli** et son texte réécrit (« Cette facture a été modifiée entre-temps : rechargez la page pour voir son état. »), quatre locales ; ses tests (`supplier-invoice-form.test.ts`) suivent.
*Écartée — fusion automatique des champs (patron de la 34-6, `companyEditConflict`)* : elle peut, après une issue inconnue, rétablir en silence la propre correction de l'utilisateur ; les coordonnées de paiement forment un groupe couplé ; elle a produit deux HIGH recyclés. *Écartée — fusion à trois voies des lignes* : sans identifiant de ligne stable, elle inventerait des rapprochements. *Écartée — reprendre la version fraîche dès la relecture* : c'est l'écrasement silencieux de #984, sous une autre forme.


### Lot B — comptes bancaires (réintégrable dans la 0.20.0 si les ressources le permettent, décision de Guy)

**Sans AC actifs** : cette section ne fait pas partie du périmètre de la story. Elle est conservée pour que le lot revienne sans réécriture. Il couvre **#984** (édition d'un compte bancaire) et l'**archivage** d'un compte sur 409 (V1-6). À sa réintégration : modules ajoutés `features/bank-accounts` et la route `bank-accounts` (**huit modules** au grain de l'épique), garde de double envoi de `submitCreate`, `submitUpdateFull` (après `e.preventDefault()`, `:468`, `:548-549`) et `confirmArchive`, avec la règle de D1-bis (`trim` des comptes bancaires comme `str::trim`, `bank_accounts.rs:506`, `:549`, test U+0085) et le **conflit sans fusion de D4** (« votre version » / « la version enregistrée », deux choix explicites) au lieu de la fusion `companyEditConflict` ci-dessous, qui est **écartée** pour la même raison que pour D4 ; à reconfirmer par une validation du lot, **et la question du plafond de cinq modules (huit avec lui) à rouvrir**.

**Lecture du code (reprise du point 5)**

5. **#984, comptes bancaires — la prémisse de l'issue a vieilli.** L'issue cite `bank-accounts/+page.svelte:413-421`
   (commit d2260575) et un second clic qui « écrase ». À la tête de la branche, `submitUpdateFull`
   (`bank-accounts/+page.svelte:548-584`) envoie `version: formVersion` (`:564`, `:573`), valeur posée **une fois** à l'ouverture
   (`openEditFull`, `:385`), et son `catch` n'écrit que `formError = errorMessageOf(err)` (`:578-580`) : **aucune relecture**.
   Le second clic renvoie donc la **même version périmée** et rend de nouveau 409 : l'écran ne remplace rien, mais il n'explique
   rien non plus, et ne dit pas que la modification est déjà enregistrée quand la réponse s'était perdue. Le défaut réel est
   « un 409 sans relecture » ; le risque d'écrasement naît du **correctif naïf** (reprendre la version fraîche sans montrer
   l'écriture concurrente). La fiche exige donc de montrer d'abord, comme la 34-6 pour la société. Parmi les gestes sans saisie, **clore** (`confirmClose`, `:627-628`) et **rouvrir** (`confirmReopen`, `:646-647`) relisent la
   liste sur 409 (`refreshAfterVersionConflict`, `:660-662`) ; **archiver ne le fait pas** : `confirmArchive` (`:586-598`) n'a
   qu'un `toast.error(errorMessageOf(err))` dans son `catch` et renvoie `ba.version` périmé à chaque essai, la même boucle « 409 sans
   relecture ». L'archivage fait partie du lot B (décision de l'orchestrateur, V1-6), **reporté à la 0.21.0**.

**Décision d'origine (D5), à réviser avant réintégration** — la fusion ci-dessous est celle écrite avant la décision du 2026-11-19 ; elle ne doit **pas** être réimplantée telle quelle :

**D5 (#984 et archivage, à réviser). Relire la liste, juger champ par champ, montrer avant de remplacer.**
`submitUpdateFull` garde une **copie de base** prise à l'ouverture (`openEditFull`, `:385`). Sur `OPTIMISTIC_LOCK_CONFLICT` (et
sur une issue inconnue), l'écran lit **lui-même** le résultat de `listBankAccounts` (`reload()` avale ses erreurs, `:159-171` :
`Promise.allSettled`, `loadError` seul ; il rend désormais un booléen de succès ; **la lecture à part est écartée** : elle perdrait `closeOrphanMode` et les filtres d'affichage, V5-9) : **échec → message
dédié, aucune version adoptée, jamais « vous pouvez enregistrer »** (V1-4). Succès : jugement du compte relu contre la base et la
saisie sur `bankName`, `description`, `iban`, `qrIban`, `isPrimary`, `journalAccountId`, `cardLast4`, avec les normaliseurs de
D1-bis (**pour les comptes bancaires, `trim` comme `str::trim` du serveur**, `bank_accounts.rs:506`, `:549`, qui retire U+0085 et non U+FEFF, à l'inverse de `String.prototype.trim` : un test avec un U+0085 final évite un avis superflu, V4-10). *Déjà enregistré* : formulaire fermé, toast. *Sinon* : version relue adoptée **avec l'avis affiché**, champs non touchés
repris de la valeur relue, champs en désaccord **nommés avec la valeur enregistrée** ; un avis dit qu'enregistrer de nouveau
les remplacera. Compte disparu de la liste : `closeOrphanMode` agit déjà (`staleNotice`).
**Relecture (V2-4)** : `listBankAccounts` reçoit un paramètre `CallOptions` et la relecture de conflit passe `{ retry: false }`. **Copie de base et champs (V2-7)** : la base devient `stored` à chaque relecture réussie ; pour une **carte**, seuls les champs envoyés se jugent (`bankName`, `description`, `cardLast4`, `journalAccountId` ; `submitUpdateFull` `:557-566`) ; `isPrimary` et `journalAccountId` sont rendus en chaîne avant d'entrer dans `companyEditConflict` (type `string | null | undefined`).
**Archivage** : `confirmArchive` appelle `refreshAfterVersionConflict(err)` comme `confirmClose` (même fichier, même boucle) ;
son échec de relecture se dit de même.
**Valeurs affichées (V1-15)** : `journalAccountId` s'affiche par le numéro et le nom du compte (liste `accounts` déjà chargée par
la page), `isPrimary` par Oui/Non, `cardLast4` et le reste tels quels ; `CompanyConflictNotice` affichant `notice.stored[field]`
brut, la page lui passe des valeurs **déjà formatées**. Clés neuves : l'avis de bloc, les deux mots Oui/Non s'ils n'existent pas
(`git grep` à l'étape 0), le message « relecture impossible », quatre locales.
**Réutilisation sans déplacer** : `companyEditConflict`, `conflictNotice`, `CompanyConflictNotice.svelte` (lignes sous les
champs ; textes `settings-company-conflict-changed`, `-merged`, `-empty` neutres) restent dans `features/settings/`.
*Écartée — déplacer en `shared/`* : toucherait le module de la 34-6.


## Dev Notes

- **Hors périmètre** : #986 (conflit de version à la modification) et le lot comptes bancaires (#984, archivage), reportés à la 0.21.0 (voir « Reportés ») ; la correction du doc-comment de `shared/utils/unknown-outcome.ts` (issue P4 à ouvrir par l'orchestrateur) ; les autres écrans de création (inventaire : issues #1354 à #1362, #1363 « description tronquée à 44 caractères + « … » sur le PDF » et #1364 « facture validée trop longue pour son PDF » — **toutes deux hors de cette story** —, #1365 à #1373 ; #1373 porte aussi `users/+page.svelte:122`, `:168`, `:209`, `:254`).
- **Fichiers communs** (tableau du fichier d'épique) : catalogues i18n (quatre locales), `i18n-keys.test.ts` (`sitesTotal`), `audit_route_registry.rs`, manuels `.tex`, avec 48-1, 48-2, 48-10, 48-13, 48-15 **et l'Epic 49** ; `routes/supplier_invoices.rs` et `repositories/supplier_invoices.rs` avec la **48-1** (réimputation d'une facture fournisseur) : le second arrivé résout. `SupplierCreatePanel.svelte` (48-15, fusionnée) et `InvoiceForm.svelte` (Epic 49) ne sont **pas** touchés.
- **Modules** : sept, découpage proposé 48-12a (serveur, quatre) / 48-12b (écran, cinq) ; voir « Modules et découpage ».
- **Ce qui doit rester vrai** : aucune saisie n'est jamais vidée par un refus ; aucune migration ; une facture client, un avoir et leur PDF inchangés ; **les données existantes ne bougent pas** (aucun backfill, aucun refus rétroactif) ; le refus absolu n'a pas de sortie ; le signalement probable propose, la confirmation est tracée à l'audit.
- **Coordination** : la PR #1327 de Beat ne touche aucun de ces fichiers.
- **Références** : fiche 34-8 ; `accounts::archive` et `confirm_designated` (#486) ; `supplier-invoice-unknown-outcome.spec.ts` ; `accounts.rs:1249`, `:1290`.
- **Statut** : `drafted` ; `48-12-reponses-perdues-fournisseurs: ready-for-dev` se pose à la clôture de la validation (orchestrateur, `sprint-status.yaml`).

## Dev Agent Record

(vide)

## Change Log

- 2026-11-19 — Création de la fiche (Claude Sonnet 5.5, `bmad-create-story`). Lecture au code à `af4e0e313`. Écart relevé à l'issue #984 : sa prémisse (« écrase ») ne se vérifie plus à la tête de la branche (l'écran renvoie la version périmée et rend de nouveau 409, sans rien expliquer) ; l'exigence de montrer avant de remplacer est reprise de l'issue.
- 2026-11-19 — Validation P1 (Opus, 0/0/10/8 ; `kesh-gate-logs/e4812/validation-p1.md`), remédiation (Sonnet) : V1-1 machine d'états de D2 (« quand même » n'envoie sans relire) ; V1-2 libellé neutre, total quand le numéro manque ; V1-3 trois issues de D3 ; V1-4 relecture en échec explicite (D1, D4, D5) ; V1-5 règle de comparaison D1-bis ; V1-6 phrase corrigée, archivage dans le lot B ; V1-7 inventaire rejoué, commandes et sortie écrites, sites absents ajoutés ; V1-8 deux phrases du manuel à réécrire ; V1-9 gates Rust ajoutés ; V1-10 deux lots, six modules au grain de l'épique, décision en attente ; V1-11 à V1-18 (citations, liste des modules, tests manquants, filtre de méthode E2E, valeurs lisibles, statut non brouillon, commandes, angle mort du détachement). Aucun fait de la remédiation n'a été relu par une seconde passe : P2 requise (Sonnet).
- 2026-11-19 — Validation P2 (Sonnet, 0/0/2/11 ; `kesh-gate-logs/e4812/validation-p2.md`), remédiation : V2-1 manuel français seul, AC et tâche C2 corrigés ; V2-2 égalité des lignes sur cinq champs, test et mutation ; V2-3 lien en nouvel onglet ; V2-4 `retry: false` et `CallOptions` sur les trois relectures ; V2-5 garde après `preventDefault()` ; V2-6 référence normalisée comme le serveur ; V2-7 base mise à jour à chaque relecture, champs d'une carte, valeurs en chaîne ; V2-8 26 sites, commande 4, sites non examinés vers #1373 ; V2-9 phrase du manuel supprimée seule, « depuis la version 0.20.0 » ; V2-10 mutations nommées ; V2-11 texte de repli de `supplier-invoices-err-stale` réécrit ; V2-12 AC renumérotés (AC-1 à AC-13) ; V2-13 statut `drafted`. Numéros d'issues reportés (#1356, #1365 à #1373). Une passe P3 (Opus), qui peut être ciblée sur ce commit, reste à faire.
- 2026-11-19 — Validation P3 ciblée (Opus, 0/1/2/2 ; `kesh-gate-logs/e4812/validation-p3-ciblee.md`), remédiation (Sonnet) : V3-1 (HIGH) l'égalité « déjà enregistré » exige le même nombre de lignes, puis chaque ligne sur cinq champs (D1-bis, D4, AC-6, tests « ligne en moins / en plus », mutation `zip` sans contrôle de longueur) ; V3-2 référence libre rognée par l'ensemble de blancs du serveur, vide ⇔ `null`, test U+200B ; V3-3 commande 3 élargie à `frontend/src` hors `*.api.ts`, sortie copiée, `users/+page.svelte:122`, `:168`, `:209`, `:254` vers #1373, stores `mode` et `session-idle` classés hors livres ; V3-4 commandes renumérotées 1 à 4 ; V3-5 la saisie comparée est le corps envoyé, la base la réponse du serveur. Dérogation à la règle de splitting ajoutée (décision de Guy du 2026-11-19), « découpage en attente » et 48-12a / 48-12b retirés. Statut laissé à `drafted`.
  **Table des renvois** (AC / mutations M / décisions D) : AC-1 : D2, D1-bis ; M : `trim`, total sans numéro. AC-2, AC-3 : D1, D2. AC-4 : D2 ; M : relire au clic « quand même ». AC-5 : D3 ; M : complété sans `supplierInvoiceId`, « déjà traité » sur relecture en échec. AC-6 : D4, D1-bis ; M : normaliseur décimal, champs QR, total et nombre seuls, **`zip` sans contrôle de longueur**, normalisation de la référence. AC-7 : D6 ; M : retirer la garde. AC-8 : D5, D1-bis ; M : `normalizeIban`, version adoptée sur relecture en échec. AC-9 : D5 ; M : ne pas relire au second essai. AC-10 : D6 ; M : garde avant `preventDefault()`. AC-11 : tous. AC-12 : D4 (repli), D5. AC-13 : manuel français.
- 2026-11-19 — Validation P4 (Sonnet, 0/0/3/8 ; `kesh-gate-logs/e4812/validation-p4.md`), remédiation : V4-1 les neuf champs d'en-tête énumérés (D1-bis), nommés dans l'avis de D4, test par champ de paiement, mutation « omettre `expectedPaymentAmount` » ; V4-2 signal « recycle » (HIGH de P3, V3-1, né du correctif de P2, V2-2) **déclaré au Project Lead par l'orchestrateur le 2026-11-19**, la dérogation dit que son motif sort du critère du `CLAUDE.md` et qu'elle est une décision du Project Lead ; V4-3 requête de relecture de D2 définie sans toucher au serveur (`status: ['draft']`, `q` = numéro sinon nom du fournisseur, filtre `contactId`, `limit: 100`, 5 pages au plus, *non vérifiable* dit à l'écran, total par `sumTtc`) ; V4-4 `ILLEGAL_STATE_TRANSITION` et 404 déclencheurs de D4 ; V4-5 document `completed` sans `supplierInvoiceId`, libellé « la facture a été enregistrée » ; V4-6 AC-13 : contrôle positif de « Enregistrer quand même », motif vide retiré ; V4-7 table des renvois complétée, vocabulaire des verdicts unifié ; V4-8 fusion d'en-tête de D4 et avis « remplacera » ; V4-9 citations `:384`, `:471` ; V4-10 `str::trim` pour les comptes bancaires, test U+0085 ; V4-11 #1363 et #1364 nommées, Epic 49 aux fichiers communs.
  **Table des renvois (P4)** : AC-1 : D2 (requête), D1-bis ; M : `trim`, total sans numéro, plafond de pages. AC-2, AC-3 : D1, D2. AC-4 : D2 ; M : relire au clic « quand même ». AC-5 : D3 ; M : complété sans `supplierInvoiceId`, « déjà traité » sur relecture en échec. AC-6 : D4, D1-bis ; M : normaliseur décimal, champs QR, total et nombre seuls, `zip` sans contrôle de longueur, **omettre un champ d'en-tête**, normalisation de la référence, **adopter la version sans avis, adopter sur relecture en échec**. AC-7 : D6 ; M : retirer la garde. AC-8 : D5, D1-bis ; M : `normalizeIban`, version adoptée sur relecture en échec. AC-9 : D5 ; M : ne pas relire au second essai. AC-10 : D6 ; M : garde avant `preventDefault()`. AC-11 : tous. AC-12 : **D2, D3**, D4 (repli), D5 (clés neuves). AC-13 : manuel français (D2, D3, D4, D5).
- 2026-11-19 — Annotation (V5-7) : l'entrée P4 ci-dessus annonçait « vocabulaire des verdicts unifié » ; c'était faux à sa date (D1 disait « rien n'a été écrit », D4 « rien n'a changé ailleurs »). Corrigé à la P5 : *déjà enregistré* / *rien n'a changé ailleurs* / *conflit*.
- 2026-11-19 — Validation P5 (Opus, 0/1/4/4 ; `kesh-gate-logs/e4812/validation-p5.md`), **second signal « recycle »** (le HIGH V5-1 naît de la fusion d'en-tête V4-8), **déclaré au Project Lead par l'orchestrateur** ; **décision de Guy du 2026-11-19 : la story est réduite au lot A** (#975, #985, #986) ; le lot B (#984, archivage) est reporté à la 0.21.0, conservé sans AC actifs. Remédiation (Sonnet) : V5-1 et V5-2 **plus aucune fusion automatique** ; D4 affiche côte à côte « votre version » et « la version enregistrée » avec deux choix explicites, rien n'est repris sans clic (AC-6, tests, mutation « reprise silencieuse d'un champ ») ; V5-3 modules recomptés sans le lot B : **six** (`features/supplier-invoices`, route `supplier-invoices`, `features/imported-supplier-invoices`, `shared/utils`, `kesh-i18n`, manuels), dérogation **maintenue** sur ce décompte et à reconfirmer ; V5-4 test paramétré sur les quatorze champs de D1-bis, mutation « retirer un champ » pour chacun ; V5-5 signaux déclarés ci-dessus ; V5-6 AC et tests `ILLEGAL_STATE_TRANSITION` et 404 ; V5-7 vocabulaire des verdicts aligné (annotation ci-dessus) ; V5-8 filtre `contactId` testé, `q` tronqué à 128 caractères, `order` retiré ; V5-9 D5 retient le booléen de `reload()` (lot B reporté).
  **Table des renvois (P5)** : AC-1 : D2, D1-bis ; M : `trim`, total sans numéro, filtre `contactId`, plafond de pages. AC-2, AC-3 : D1, D2. AC-4 : D2 ; M : relire au clic « quand même ». AC-5 : D3 ; M : complété sans `supplierInvoiceId`, « déjà traité » sur relecture en échec. AC-6 : D4, D1-bis ; M : retirer un champ du comparateur (quatorze), `zip` sans contrôle de longueur, normaliseur décimal, normalisation de la référence, reprise silencieuse d'un champ, version adoptée sans clic ou sur relecture en échec, fusion d'en-tête automatique. AC-7 : D6 ; M : retirer la garde. AC-8 : tous (revue). AC-9 : D2, D3, D4 (clés neuves). AC-10 : manuel français (D2, D3, D4). Lot B : aucun AC actif.
- 2026-11-19 — Cinq modules, **dérogation devenue sans objet après la réduction et le retrait de la retouche de `shared/utils`** (décision de l'orchestrateur). Le **décompte de six** annoncé de la P1 à la P5 était faux à ses dates : la première rédaction omettait `features/imported-supplier-invoices` (`listImported` reçoit un `CallOptions`) et comptait `shared/utils` ; sans le lot B, au grain de l'épique, le décompte était six (avec `shared/utils`), et il tombe à **cinq** en retirant la retouche du doc-comment de `unknown-outcome.ts` (devenu une limite notée en D7, issue P4 à ouvrir par l'orchestrateur). La section « Dérogation règle de splitting » et ses mentions sont retirées ; les entrées P3 à P5 ci-dessus, qui la décrivent, ne se corrigent pas (historique à leur date). Lot B reporté : décompte de huit avec lui. Relecture entière de la fiche après la recomposition par script : restes corrigés (point 6 du « Ce qui se lit au code », D7, tâche A4, Dev Notes, note de limite du lot B).
  **Table des renvois** : AC-1 : D2, D1-bis. AC-2, AC-3 : D1, D2. AC-4 : D2. AC-5 : D3. AC-6 : D4, D1-bis. AC-7 : D6. AC-8 : tous. AC-9 : D2, D3, D4. AC-10 : manuel français. Modules : cinq, aucune dérogation. Mutations : voir le tableau des tests (inchangé).
- 2026-11-19 — **Refonte de la conception, décisions de Guy du 2026-11-19** : la story **change de conception** et **la boucle de validation repart** sur cette conception (déclaré au Project Lead par l'orchestrateur ; troisième signal « recycle » : le HIGH de la P6, V6-1, naît de la règle de conflit de #986). **Périmètre** : #975 et #985 ; **#986 part en 0.21.0** avec le lot comptes bancaires, sa conception (D1-bis, D4, D5, findings P6) conservée sous « Reportés ». **Nouvelle règle, dans le serveur** (sous verrou, sans migration, création **et** modification, tout chemin) : numéro de fournisseur identique (non vide, blancs retirés, casse ignorée) = refus absolu `409 SUPPLIER_INVOICE_DUPLICATE` ; même fournisseur, même date et même total TTC = refus par défaut `409 SUPPLIER_INVOICE_PROBABLE_DUPLICATE`, levé par `confirmNotDuplicate` et tracé à l'audit ; si les deux factures portent un numéro, seul le numéro décide ; annulées exclues ; numéro facultatif ; données existantes intactes. Décisions de conception de la fiche (Sonnet) : verrou `FOR UPDATE` sur la ligne du fournisseur ; comparaison en Rust sur les factures du fournisseur ; règle rejouée à la modification seulement si fournisseur, numéro, date ou total changent ; `DuplicatePolicy` sur le patron `confirm_designated` ; audit `supplier_invoice.duplicate_confirmed` ; changement de contrat de l'API à la Note de version, à `docs/api-external.md` et au manuel d'administration. **Retirés** : D2 (relecture des brouillons, verdicts, requête de relecture, plafond de pages) et D4 (conflit côte à côte) comme conceptions actives ; D3 (#985) et la garde de double envoi (D10) restent. **Modules** : sept au grain de l'épique (`kesh-db`, `kesh-api`, `kesh-i18n`, `features/supplier-invoices`, route `supplier-invoices`, `features/imported-supplier-invoices`, documentation), **aucune dérogation** ; découpage proposé 48-12a (serveur, quatre) / 48-12b (écran, cinq). **Conséquence signalée** : un fournisseur dont la numérotation repart à un voit ses nouvelles factures refusées tant que les anciennes ne sont pas annulées.
