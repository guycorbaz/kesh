# Story 48-12 : Réponses perdues — fiche-mère (renvoi vers la 48-12a et la 48-12b)

Status: renvoi (la story est **découpée** ; voir `48-12a-doublons-fournisseurs-serveur.md` et `48-12b-doublons-fournisseurs-ecran.md`, toutes deux **`ready-for-dev`** depuis le 2026-11-19 — la 48-12a, validation terminée à la P5 (0/0/0/9), la 48-12b, validation terminée à la P6 (0/0/0/9) ; sortie commune ; `sprint-status.yaml` : `48-12-reponses-perdues-fournisseurs: split`, `48-12a-doublons-fournisseurs-serveur` et `48-12b-doublons-fournisseurs-ecran` : `ready-for-dev`)

**Cette fiche n'est plus une story à développer.** Décisions de Guy du 2026-11-19 : la 48-12 se découpe en **48-12a** (serveur, #975 : règle anti-doublon sous verrou, cinq modules, pas d'AC écran) puis **48-12b** (écran, #975 côté écran et #985, cinq modules, **dépend de la 48-12a**). Elle est **conservée** pour ce que les deux fiches ne portent pas : l'**inventaire par le symptôme** (sites hors périmètre et issues), les **reports à la 0.21.0** (#986, lot comptes bancaires, avec les findings P6 non traités) et l'**historique** des passes de validation (Change Log). *Choix* : un renvoi court plutôt qu'un retrait, pour ne pas perdre ces trois éléments ; l'orchestrateur remplace la clé `48-12-reponses-perdues-fournisseurs` de `sprint-status.yaml` par `48-12a-…` et `48-12b-…`.

| Fiche | Objet | Issues | Modules |
|---|---|---|---|
| `48-12a-doublons-fournisseurs-serveur.md` | règle anti-doublon dans le serveur (numéro comparé par **année civile** ; date et total ; confirmation tracée ; dévalidation d'une annulée contrôlée) | #975 | 5 : `kesh-core`, `kesh-db`, `kesh-api`, `kesh-i18n`, documentation |
| `48-12b-doublons-fournisseurs-ecran.md` | refus visible à l'écran, confirmation, complétion d'un document dont la réponse est perdue | #975 (écran), #985 | 5 : `features/supplier-invoices`, route `supplier-invoices`, `features/imported-supplier-invoices`, `kesh-i18n`, manuel français |

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
| `createSupplierInvoice`, `new/+page.svelte:149` | doublon de brouillon (#975) | 48-12a puis 48-12b |
| `completeImport`, `new/+page.svelte:149` | échec annoncé à tort (#985) ; doublon refusé par la règle serveur | 48-12a puis 48-12b |
| `updateSupplierInvoice`, `[id]/edit/+page.svelte:130` | la **règle anti-doublon** s'applique aussi à la modification (D6) ; le **conflit de version** (#986) est reporté à la 0.21.0 | 48-12a ; #986 reporté |
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

La règle de la 48-12a (D1) est **propre aux factures fournisseur** ; les sites ci-dessus relèvent d'issues distinctes.

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
- 2026-11-19 — **Découpage en 48-12a (serveur) et 48-12b (écran)**, décision de Guy du 2026-11-19 (découpage proposé à la refonte, approuvé ; sept modules au grain de l'épique, quatre + cinq). Deux fiches distinctes, chacune avec ses AC, tâches, tests, mutations et Note de version ; la 48-12b dépend de la 48-12a. **Règle d'exercice (option (a), décision de Guy)** : le critère 1 (même numéro du fournisseur) ne compare que les factures du **même exercice** (période de la date de facture) ; le critère 2 est inchangé (même date implique même exercice) ; hors de tout exercice, la fiche retient l'**année civile** (D2bis de la 48-12a). La « conséquence » de la numérotation annuelle, écrite à la refonte, est **retirée**. D6 (la règle ne se rejoue à la modification que si fournisseur, numéro, date ou total change) approuvé par Guy. Cette fiche devient un renvoi (inventaire, reports 0.21.0, historique).
- 2026-11-19 — **Remédiation de la validation P1 des deux fiches** (12a Opus 0/1/6/5 ; 12b Opus 0/0/4/8 ; rapports `validation-12a-p1.md` et `validation-12b-p1.md`). **Décisions de Guy du 2026-11-19** : critère 1 par **année civile** (la règle d'exercice, D2bis de la 48-12a, est retirée ; la numérotation en cours d'année se contourne par un caractère ajouté, « 15-b ») ; **dévalidation d'une facture annulée** contrôlée comme une création (refus, mêmes codes et `details`) ; **sortie commune** de la 48-12a et de la 48-12b. Modules : 48-12a cinq (`kesh-core` s'ajoute), 48-12b cinq. Détail et renvois : Change Log de chaque fiche.
