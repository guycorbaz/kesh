# Story 48-12 : Réponses perdues — factures fournisseur et comptes bancaires

Status: drafted (validation en cours ; `ready-for-dev` se pose à la clôture de la validation, par l'orchestrateur)

Issues : #975, #984, #985, #986 (toutes `bug`, **P2**). Release 0.20.0, Epic 48. Voie : frontend (aucun code Rust, aucune migration).

**Deux lots ordonnés** (V1-10) : **lot A**, factures fournisseur (#975, #985, #986, tâches A1 à A4) ; **lot B**, comptes bancaires (#984 et l'archivage sur 409, tâches B1 à B2). Le décompte des modules et la dérogation de splitting sont sous « Modules et découpage » et « Dérogation règle de splitting ».

## Story

En tant que comptable qui tient des factures fournisseur réelles (dettes, lots de paiement pain.001),
je veux qu'une réponse perdue (délai, coupure) ou un conflit de version ne me fasse ni enregistrer deux fois la même
facture, ni remplacer en silence l'écriture d'un autre, ni croire échouée une écriture qui a réussi,
afin que mes dettes restent justes et qu'aucune facture ne soit payée deux fois, sans que je perde jamais ma saisie.

**Usage réel.** Guy saisit ses factures fournisseur et en tire des lots pain.001. Un doublon de brouillon, une fois validé,
double la dette (compte 2000) et peut être payé deux fois. C'est le seul des quatre défauts qui touche les livres ; les trois
autres font croire à un échec ou font perdre du temps. Le correctif ne supprime jamais une saisie : il la garde toujours, et
**relit avant de proposer de réessayer**.

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
4. **#986, édition.** `[id]/edit/+page.svelte:128-132` : `updateSupplierInvoice(id, { ...req, version: invoice.version })`.
   `invoice.version` n'est jamais relu après l'envoi. Une réponse perdue après un `PUT` réussi, puis un second clic, part avec la
   version d'avant : `409 OPTIMISTIC_LOCK_CONFLICT` (`supplier_invoices.rs:772-808`, `lock_draft_in_tx` avec `req.version`).
   `supplier-invoice-form.ts:440-448` dit « modifiée entre-temps : rechargez la page, puis refaites votre correction » : faux
   quand la modification est précisément celle qu'on a envoyée, et recharger fait perdre la saisie.
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
   relecture ». L'archivage **entre dans la story** (lot B, décision de l'orchestrateur, V1-6).
6. **Le précédent de Kesh : relire, juger, montrer, jamais d'idempotence serveur.** `git grep -n -i "idempot"` ne trouve aucune
   clé d'idempotence dans `crates/` ni dans `frontend/src` (les occurrences sont des commentaires sur des gestes rejouables,
   ex. `bank_accounts.rs:1644`). Les patrons existants sont tous « relecture + jugement » :
   - 34-6 (#953), la société : `frontend/src/lib/features/settings/company-conflict.ts` — `companyEditConflict` (fusion à trois
     voies, issues `unchanged` / `already-saved` / `changed`), `conflictNotice`, `CompanyConflictNotice.svelte`, branchés dans
     `settings/+page.svelte:140-146` (imports `:19-27`) ;
   - 34-5 (#951) et 34-8 (#938), règlements et fiche fournisseur : `unknown-outcome.ts:65-68` (`isSettlementOutcomeUnknown`),
     dont le doc-comment (`:56-62`) écrit que la page d'édition (#986), la complétion (#985) et la création (#975) **ne sont pas
     couvertes** : c'est le périmètre de cette story.
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
| `createSupplierInvoice`, `new/+page.svelte:149` | doublon de brouillon (#975) | A |
| `completeImport`, `new/+page.svelte:149` | échec annoncé à tort (#985) | A |
| `updateSupplierInvoice`, `[id]/edit/+page.svelte:130` | échec annoncé à tort (#986) | A |
| `updateBankAccount`, `bank-accounts/+page.svelte:575` (`edit-full`) | 409 sans relecture (#984) | B |
| `archiveBankAccount`, `bank-accounts/+page.svelte:586-598` | 409 sans relecture, version périmée renvoyée (V1-6) | B |

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

Le **module** de D7 est conçu pour servir à ces sites sans réécriture.

## Modules et découpage

**Décompte au grain du fichier d'épique** (`main:…/epic-48-corrections-0-20-0.md:210-214`), les modules **modifiés** :
1. `features/supplier-invoices` ; 2. la route `supplier-invoices` (`new`, `[id]/edit`) ; 3. la route `bank-accounts` ;
4. `shared/utils` (`unknown-outcome.ts`, doc-comment seul) ; 5. `kesh-i18n` ; 6. les manuels. `features/imported-supplier-invoices`
et `features/bank-accounts` ne sont **pas** modifiés (lecture seule du premier ; aucun fichier de `features/bank-accounts` dans le
lot B) : ils ne comptent pas. **Six modules, au-dessus du plafond de cinq.**
**Règle** : « la 48-12 se découpe si elle dépasse cinq modules, ou écrit sa dérogation » (`:218`). La story **reste unique**, avec la dérogation ci-dessous.

## Dérogation règle de splitting

Décision du Project Lead (Guy) du 2026-11-19, option (a) de l'orchestrateur : la 48-12 ne se découpe pas.
- **Six modules au grain du fichier d'épique** (liste ci-dessus), un de plus que le plafond de cinq.
- **Un même motif** — « relire, juger, montrer » (D1) — appliqué à **deux familles d'écrans** : les factures fournisseur (lot A) et les comptes bancaires (lot B). Le lot B n'ajoute que la route `bank-accounts` ; `kesh-i18n` et le manuel sont communs.
- **Deux lots ordonnés** : A (#975, #985, #986, tâches A1 à A4), puis B (#984 et l'archivage, B1 et B2) ; les gates et l'E2E se jouent une fois, à la fin.
- *Découper* aurait créé deux fiches dont les catalogues, `sitesTotal` et le manuel se recoupent, pour un motif identique.

## Décisions

**D1 (commune). Un motif : relire, juger, montrer ; aucune clé d'idempotence serveur.**
Chaque écran, sur un refus qui peut venir d'une écriture déjà faite (409 de version, `IMPORT_NOT_PENDING_COMPLETION`) ou sur une
issue inconnue (`isSettlementOutcomeUnknown`), **relit l'état écrit**, le **compare à la saisie** et dit lequel des trois cas
vaut : *déjà enregistré*, *rien n'a été écrit*, *autre chose a été écrit*. La saisie reste toujours en place. **Une relecture qui
échoue se détecte explicitement et se dit** ; aucun jugement ne se fait sur un état périmé, et aucune version n'est adoptée alors.
*Écartée — clé d'idempotence serveur* : elle exige un stockage durable des clés, donc une **migration**, et la release n'en
admet qu'une (celle de la 49-1) ; en mémoire, elle serait perdue au redémarrage du conteneur. *Écartée — désactiver le bouton
après une issue inconnue* : l'utilisateur dont l'écriture n'a pas eu lieu serait bloqué et devrait ressaisir. *Écartée —
détection serveur sur le contenu* : refuserait une facture légitimement semblable ; le dépôt signale, il ne tranche pas
(`CLAUDE.md`, « un appariement propose, il ne crée jamais »).

**D1-bis (commune). Règle de comparaison (V1-5).** Un jugement compare `stored` (relu) à la saisie **sous la forme que le serveur
range**, jamais littéralement : montants, quantités et taux comparés en décimal (`Big.eq`, quatre décimales : `"100.0000"` égale
`"100.00"`) ; IBAN, QR-IBAN et référence normalisés comme le serveur (`normalizeIban`, `supplier-invoice-form.ts:334` ; blancs,
insécables compris, retirés) ; `trim`, et vide ⇔ `null`. **La référence de paiement (V2-6, V3-2)** se normalise comme le serveur (`supplier_invoices.rs:458-473`) : avec un QR-IBAN, tous les blancs sont retirés (`normalize_payment_field`) ; sans, elle est **rognée aux deux bouts par l'ensemble de blancs du serveur** (`trim_matches(is_payment_blank)`, `:378`, `:470` : `char::is_whitespace`, dont U+0085, plus U+200B et U+FEFF), puis **vide après normalisation ⇔ `null`** (`present`, `:383`). `String.prototype.trim` ne retire ni U+200B ni U+0085 : une **fonction pure** de `resend-guard.ts` reproduit cet ensemble, testée avec un U+200B final collé. **Les lignes (V2-2, V3-1)** : l'égalité exige d'abord **le même nombre de lignes**, puis, **position par position, chaque ligne égale sur ses cinq champs** (description, quantité, prix unitaire, taux de TVA, compte de charge), chacun normalisé, puis l'en-tête. Une ligne **supprimée** ou **ajoutée** ne rend donc jamais *déjà enregistré* (le serveur remplace toutes les lignes, `repositories/supplier_invoices.rs:802`, `:585-599`). Le total, dérivé, ne sert qu'à l'**affichage** de l'avis. **Le côté « saisie » (V3-5)** est le **corps tel qu'il est parti** (le même objet que le `PUT`, issu de `linesForServer` et `paymentFields`, `supplier-invoice-form.ts:203-212`, `:328-345`, et `fNumber.trim() || null`), jamais le formulaire brut (« 12,50 », « 1'250 »), sur lequel `Big` lèverait ou différerait ; le côté « base » est la **réponse du serveur**, non les champs rouverts par `formLineFromResponse`. Sur les comptes bancaires, ces normaliseurs sont passés à `companyEditConflict`. Le cas « je viens d'enregistrer moi-même » (réponse perdue, puis second clic) **doit** rendre
*déjà enregistré*, jamais « modifié entre-temps » : un test le fixe avec les formes du serveur (`"100.0000"`, IBAN sans blancs), et
une mutation retire le normaliseur. Ces fonctions sont pures et testées à part (D7).

### Lot A — factures fournisseur

**D2 (#975). Relire les brouillons et signaler un brouillon semblable, sans jamais refuser ni affirmer.**
Après une issue inconnue de `createSupplierInvoice`, l'écran lit `listSupplierInvoices({ status: 'draft', … })` et cherche les
brouillons de **même fournisseur, même date de facture, même numéro** (`trim`, absent = absent) ; **quand le numéro manque,
le total normalisé (D1-bis) entre dans la comparaison** (sans numéro, deux factures légitimes du même jour se confondraient).
Le libellé est **neutre** : « Un brouillon semblable existe déjà », jamais « votre facture a été enregistrée » (V1-2). Chaque
brouillon trouvé montre son total, une marque « total différent de votre saisie » quand il diffère, et un lien vers sa fiche, **ouvert dans un nouvel onglet** (`target="_blank" rel="noopener"`, V2-3) : un lien ordinaire quitterait la page et ferait perdre une saisie que la fiche dit conserver ; le lien de D3, lui, peut rester ordinaire (la facture existe).
Verdicts : *trouvé* ; *non trouvé* (« Aucune facture semblable n'a été enregistrée : vous pouvez réessayer ») ; *non vérifiable*
(relecture en échec, détectée explicitement, ou plus de résultats que la page n'en rend ; message dédié et lien vers la liste).
**Relecture (V2-4)** : `listSupplierInvoices` reçoit un paramètre `CallOptions` et la relecture passe `{ retry: false }`, comme D4 pour `getSupplierInvoice` : sans cela, un réseau coupé fait rejouer le `GET` (~14 s, et jusqu'à ~150 s sur des délais) avec `saving` vrai, et le verdict n'arrive pas (`api-client.ts:414`, délais `:37`, `:42`) ; le précédent E2E choisit un `500` pour cela.
**Machine d'états (V1-1)** :
- l'issue inconnue déclenche **une** relecture, d'où le verdict ;
- verdict *trouvé* ou *non vérifiable* : le bouton devient « Enregistrer quand même », et **ce clic envoie, sans relecture** ;
- verdict *non trouvé* : le clic ordinaire relit **une fois** puis envoie si le résultat est encore *non trouvé* (couvre
  l'écriture tardive du serveur) ; un brouillon apparu entre-temps rend le verdict *trouvé* et n'envoie pas ;
- toute modification du fournisseur, de la date, du numéro ou du total remet le verdict à zéro (l'issue reste « inconnue » : la
  relecture se refera au clic ordinaire suivant) ;
- un envoi réussi remet tout à zéro. Le signalement **propose, ne bloque jamais** : « Enregistrer quand même » est toujours possible.
*Écartée — critère sur le seul numéro/date/fournisseur avec total en simple affichage* : sans numéro, un faux « déjà créé » fait
abandonner une vraie saisie. *Écartée — relire aussi au clic « Enregistrer quand même »* : boucle possible, second brouillon
légitime impossible.
*Angle mort assumé* : une requête restée en vol côté serveur au-delà de la relecture du clic suivant. Il se ferme seulement par
une clé durable (issue « clé d'idempotence »).

**D3 (#985). Sur `IMPORT_NOT_PENDING_COMPLETION` ou issue inconnue, relire le document : trois issues (V1-3).**
Relire `listImported('completed')` puis, au besoin, `listImported('to_complete')` (aucun n'est paginé ; `listImported` reçoit un paramètre `CallOptions` et la relecture passe `{ retry: false }`, V2-4) :
- *complété avec facture* (`supplierInvoiceId` renseigné) : **avis**, non erreur : « Ce document a déjà été traité : la facture a
  été enregistrée en brouillon » avec le lien `/supplier-invoices/<id>`, saisie en place, envoi désactivé ;
- *encore à traiter* : « Aucune facture n'a été enregistrée : vous pouvez réessayer », bouton actif ;
- *retiré* (ni l'un ni l'autre, relecture réussie) ou 409 dont le document est introuvable : le message actuel
  (`supplier-invoices-err-not-pending`) ;
- *relecture en échec* (détectée explicitement) : message d'issue inconnue, **jamais** « déjà traité ».
L'avis est neutre sur l'auteur. *Angle mort assumé (V1-18)* : un document complété puis **détaché** par un tiers (il redevient
`to_complete`, `supplier_invoice_id` à `NULL`) se complète de nouveau : une complétion perdue, suivie d'un détachement puis d'un
renvoi, crée une seconde facture. Rare ; écrit, non traité.

**D4 (#986). Relire la facture, juger, ne reprendre la version fraîche qu'après avoir montré.**
Sur un 409 de version ou une issue inconnue du `PUT`, la page relit `getSupplierInvoice(id, { retry: false })` et juge par une
fonction pure (D7, comparaison de D1-bis) sur les seuls champs **envoyés** (avec QR, les coordonnées de paiement ne partent pas) :
- **`stored.status !== 'draft'`** (V1-16) : un collègue a validé entre-temps (`lock_draft_in_tx` refuse le statut avant la
  version : `409 ILLEGAL_STATE_TRANSITION`) ; renvoi à la fiche avec `supplier-invoices-edit-not-draft`, sans verdict de comparaison ;
- *déjà enregistré* (même nombre de lignes, l'en-tête **et chacune des lignes**, sur leurs cinq champs, égalent le corps envoyé, D1-bis) : succès, notification, retour à la fiche. Une ligne qui ne diffère que par sa description ou son compte, une ligne **supprimée** ou **ajoutée** rendent *autre chose a été écrit* (ou *rien n'a changé ailleurs* si `stored` égale la base), jamais *déjà enregistré* ;
- *rien n'a changé ailleurs* (`stored` égale la copie de base, mêmes règles d'égalité, hors version) : « rechargé, vous pouvez enregistrer de
  nouveau », version fraîche adoptée ;
- *autre chose a été écrit* : avis qui **nomme les champs** en désaccord avec la valeur enregistrée (fournisseur, numéro, date,
  échéance, projet ; pour les lignes, le **numéro de chaque ligne en désaccord**, avec le total TTC et le nombre de lignes enregistrés, à titre d'affichage), saisie intacte ; la version fraîche
  n'est adoptée qu'avec cet avis affiché ;
- *relecture en échec* : « Conflit de version et rechargement impossible — rechargez la page », **aucune version adoptée**.
**Copie de base (V2-7)** : prise à l'ouverture, elle devient `stored` à chaque relecture réussie (comme en 34-6, `company-conflict.ts:7-13`) ; sinon un troisième écrit jugerait contre la base d'origine et classerait de travers « repris » et « enregistré entre-temps ». **Relecture (V2-4)** : `getSupplierInvoice(id, { retry: false })`.
**Texte de repli (V2-11)** : la branche `OPTIMISTIC_LOCK_CONFLICT` de `describeSubmitError` (`supplier-invoice-form.ts:440-448`, clé `supplier-invoices-err-stale`) est **gardée comme repli** et son texte réécrit (« Cette facture a été modifiée entre-temps : rechargez la page pour voir son état. », sans « refaites votre correction »), quatre locales ; ses tests (`supplier-invoice-form.test.ts`) suivent.
*Écartée — fusion à trois voies des lignes* : sans identifiant de ligne stable, elle inventerait des rapprochements.

**D6 (lots A et B). Garde synchrone contre le double envoi.** `SupplierInvoiceForm.submit` (`:376`) revient d'emblée si `saving` ;
même garde dans `submitCreate`, `submitUpdateFull` (**après** `e.preventDefault()`, `:468`, `:548-549`, V2-5 : une garde qui reviendrait avant laisserait le navigateur soumettre le formulaire et perdre la saisie) et `confirmArchive` (`submitting`) de la page des comptes bancaires.

**D7. Où vit la logique.** Pure et testable à part, dans `frontend/src/lib/features/supplier-invoices/` : `resend-guard.ts`
(machine d'états et verdicts de D2, jugement de D4, comparaisons de D1-bis) ; les avis de D3 et D4 sont des composants du même
dossier. `isSettlementOutcomeUnknown` (`unknown-outcome.ts:65`) est réutilisée telle quelle ; son doc-comment (`:56-62`) est
réécrit, ainsi que celui de `[id]/edit/+page.svelte:21-22` (« `OPTIMISTIC_LOCK_CONFLICT` invite à recharger »).

### Lot B — comptes bancaires

**D5 (#984 et archivage). Relire la liste, juger champ par champ, montrer avant de remplacer.**
`submitUpdateFull` garde une **copie de base** prise à l'ouverture (`openEditFull`, `:385`). Sur `OPTIMISTIC_LOCK_CONFLICT` (et
sur une issue inconnue), l'écran lit **lui-même** le résultat de `listBankAccounts` (`reload()` avale ses erreurs, `:159-171` :
`Promise.allSettled`, `loadError` seul ; il rend désormais un booléen de succès, ou l'écran lit à part) : **échec → message
dédié, aucune version adoptée, jamais « vous pouvez enregistrer »** (V1-4). Succès : jugement du compte relu contre la base et la
saisie sur `bankName`, `description`, `iban`, `qrIban`, `isPrimary`, `journalAccountId`, `cardLast4`, avec les normaliseurs de
D1-bis. *Déjà enregistré* : formulaire fermé, toast. *Sinon* : version relue adoptée **avec l'avis affiché**, champs non touchés
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

## Critères d'acceptation

**Lot A**
1. **AC-1 (#975).** Création sans document, réponse perdue après l'écriture : l'écran relit et signale « un brouillon semblable
   existe déjà » (total, marque « total différent » le cas échéant, lien en nouvel onglet). Un seul brouillon existe tant que
   l'utilisateur n'a pas cliqué « Enregistrer quand même ».
2. **AC-2 (#975).** Réponse perdue avant l'écriture : « aucune facture semblable » ; le renvoi crée un seul brouillon ; la saisie
   n'a jamais été perdue. Cas « brouillon préexistant, requête perdue avant écriture » : l'écran **n'affirme pas** que la saisie est
   enregistrée, et le renvoi reste possible.
3. **AC-3 (#975).** Relecture impossible : l'écran le dit, renvoie à la liste, n'écrit rien sans « Enregistrer quand même ».
4. **AC-4 (#975, machine d'états).** « Enregistrer quand même » envoie **sans relire** (exactement un `POST`) ; un clic ordinaire
   après verdict *non trouvé* relit une fois ; modifier fournisseur, date, numéro ou total remet le verdict à zéro.
5. **AC-5 (#985).** Complétion perdue : document complété = avis et lien, envoi désactivé ; **encore à traiter = « aucune facture
   enregistrée, réessayez »** ; retiré = message actuel ; relecture en échec = message d'issue inconnue.
6. **AC-6 (#986).** Modification enregistrée dont la réponse est perdue : reconnue, retour à la fiche, y compris avec les formes du
   serveur (`"100.0000"`, IBAN sans blancs, QRR avec espaces) ; une ligne qui ne diffère que par sa description ou son compte,
   une ligne **supprimée** (brouillon de trois lignes, la troisième retirée, envoi perdu avant l'écriture) ou **ajoutée**
   rendent « modifié entre-temps » ou « rien n'a changé ailleurs », jamais « déjà enregistré » ; concurrente différente : champs nommés, version adoptée avec l'avis ;
   relecture en échec : aucune version adoptée ; facture relue non brouillon : renvoi à la fiche.
7. **AC-7.** Deux événements `submit` consécutifs sur le formulaire des factures n'envoient qu'une requête.

**Lot B**
8. **AC-8 (#984).** Après un 409 sur l'édition d'un compte bancaire : relecture explicite ; échec = message dédié et aucune version
   adoptée ; *déjà enregistré* ferme avec un toast ; sinon champs en désaccord nommés avec des valeurs lisibles (compte, Oui/Non),
   second clic avec la version relue **après** l'avis. Aucun chemin n'envoie la version fraîche sans avoir montré.
9. **AC-9 (archivage).** Un 409 d'archivage relit la liste ; le second essai part avec la version relue.
10. **AC-10.** Garde de double envoi sur `submitCreate`, `submitUpdateFull`, `confirmArchive`, **après** `preventDefault()`
    (`defaultPrevented` vérifié).

**Communs**
11. **AC-11 (production).** Aucun fichier de `kesh-api`, `kesh-db`, `kesh-qrbill`, `InvoiceForm.svelte` ; ni facture client, ni avoir,
    ni PDF touchés ; la page d'édition ne s'ouvre que sur un brouillon (`edit/+page.svelte:86`).
12. **AC-12.** Clés neuves dans les quatre locales ; `sitesTotal` et les gardes de repli recomptés depuis la source.
13. **AC-13 (manuel).** Les phrases devenues fausses sont **réécrites** dans `docs/manual/fr/user-manual.tex` (seul manuel existant :
    `docs/manual/{de,en,it}/` ne portent qu'un `README.md` « traductions non encore créées ») ; le contrôle final greppe « recharger »,
    « ne relit pas encore » et « déjà été traité » dans ce `.tex` et dans son PDF aplati après `make fr` (non commité).

## Tests à écrire, et la preuve du rouge

Chaque test neuf rougit sans le correctif ; journaux dans `/home/gcorbaz/devel/kesh-gate-logs/e4812/`. Les fixtures portent les
**formes du serveur** (`"100.0000"`, IBAN sans blancs).

| Test | Fichier | Rougit parce que |
|---|---|---|
| verdicts *trouvé* / *non trouvé* / *non vérifiable* ; clé avec numéro ; clé avec total quand le numéro manque ; vrai doublon **et** facture légitime semblable (même jour, total différent) | `resend-guard.test.ts` (neuf) | la fonction n'existe pas ; mutations : retirer `trim`, ôter le total quand le numéro manque |
| machine d'états : « Enregistrer quand même » = un `POST`, sans relecture ; reset à la modification | `resend-guard.test.ts`, `new-page.test.ts` | aujourd'hui le second clic poste sans signal ; mutation : « relire aussi au clic Enregistrer quand même » |
| jugement d'une facture : `already-saved` (formes du serveur) / `unchanged` / `changed` / `status ≠ draft` ; champs QR exclus ; **une ligne qui ne diffère que par sa description ou son compte de charge → `changed`, jamais `already-saved`** ; **`stored` moins sa dernière ligne, et `stored` plus une ligne → jamais `already-saved`** ; QR-IBAN avec QRR saisie avec espaces → `already-saved` ; référence libre avec un U+200B final → `already-saved`, référence de seuls blancs du serveur ⇔ `null` | `resend-guard.test.ts` | idem ; mutations : retirer le normaliseur décimal, comparer les champs QR, **comparer seulement le total et le nombre de lignes**, **comparer par `zip` sans contrôle de longueur**, retirer la normalisation de la référence, la remplacer par `trim` seul |
| création : brouillon trouvé → libellé neutre, marque « total différent » ; relecture impossible → aucun `POST` sans « quand même » (AC-3) | `new-page.test.ts` | aujourd'hui le second clic poste |
| complétion : 409 → avis + lien ; **encore à traiter → réessayer** ; retiré → message actuel ; relecture en échec | `new-page.test.ts` | aujourd'hui, message d'échec ; mutations : traiter « complété sans `supplierInvoiceId` » comme « complété avec facture », afficher « déjà traité » sur relecture en échec |
| édition : 409 → `stored` égale la saisie → succès ; 409 concurrent → avis, version adoptée avec l'avis ; **relecture en échec → aucune version adoptée** ; statut ≠ draft → fiche | `edit-page.test.ts` | aujourd'hui, message « rechargez » ; mutations : « adopter la version sans avis », « adopter sur relecture en échec » |
| double `submit` synchrone ; sur la page des comptes, `defaultPrevented` vrai au second événement | `SupplierInvoiceForm.test.ts`, `bank-accounts-page.test.ts` | pas de garde ; mutations : retirer la garde `saving` / `submitting`, la placer avant `preventDefault()` |
| compte bancaire : 409 → `already-saved` ferme ; `changed` nomme les champs, valeurs lisibles ; second envoi avec la version relue ; **relecture en échec → aucune version adoptée** ; IBAN saisi avec blancs égal l'IBAN relu ; carte : seuls les champs envoyés se jugent | `bank-accounts-page.test.ts` | ni relecture ni avis aujourd'hui ; mutations : retirer `normalizeIban` du comparateur, adopter la version quand la relecture a échoué |
| archivage : 409 → relecture, version relue | `bank-accounts-page.test.ts` | pas de `refreshAfterVersionConflict` ; mutation : ne pas relire au second essai |
| **E2E lot A** : `route.fetch()` + `route.abort()` sur `POST /api/v1/supplier-invoices` (puis second clic ; l'API confirme **un** brouillon avant « quand même »), sur `POST …/complete` (y compris **coupure avant écriture** : le document reste à traiter, l'avis dit « réessayez »), sur le `PUT` | `supplier-invoice-lost-response.spec.ts` (neuf) | aujourd'hui deux brouillons / échec annoncé à tort |
| **E2E lot B** : modification concurrente par l'API entre l'ouverture et l'envoi ; avis, puis second clic ; archivage sur 409 | `bank-accounts-conflict.spec.ts` (neuf) | aucun avis aujourd'hui |

E2E exigé : défaut visible dans le navigateur et à la frontière HTTP. **Filtrer `route.request().method()` et ne couper qu'une
fois** (V1-14) : la route du `PUT` (`/supplier-invoices/<id>`) intercepterait sinon la relecture `GET` de D4 ; de même le `POST
/supplier-invoices` et la relecture `GET /supplier-invoices?status=draft` ; précédent `supplier-invoice-unknown-outcome.spec.ts:163`
et `:283`. Recette visuelle (`CLAUDE.md`) : captures, thème clair, de chaque état neuf dans `kesh-gate-logs/e4812/captures/`. Chaque
mutation est suivie d'un `touch` du fichier restauré.

## Tâches

**Lot A.** A1 : `resend-guard.ts` (D1-bis, D2, D4, D7) et ses tests ; `listSupplierInvoices` et `listImported` reçoivent `CallOptions` (V2-4). A2 : `new/+page.svelte` (D2, D3) et
`SupplierInvoiceForm.svelte` (D6, bouton « Enregistrer quand même », avis). A3 : `[id]/edit/+page.svelte` (D4, doc-comment) ; texte de repli de `supplier-invoices-err-stale` (V2-11).
A4 : `unknown-outcome.ts` (doc-comment).
**Lot B.** B1 : `bank-accounts/+page.svelte` (D5, D6, archivage, valeurs lisibles, avis) ; `listBankAccounts` reçoit `CallOptions` (V2-4). B2 : tests de la page.
**Communs.** C0 (étape 0) : relire les points 2 à 5 de « Ce qui se lit au code » contre la branche d'épique à jour. C1 : catalogues
(quatre locales), `i18n-keys.test.ts` (`sitesTotal`) et gardes de repli recomptés. C2 : manuel français (`user-manual.tex`). C3 : E2E et captures. C4 : gates.
C5 : Dev Agent Record, Change Log, Note de version.

## Manuels

Seul `docs/manual/fr/user-manual.tex` existe (V2-1) : `docs/manual/de/`, `it/` et `en/` ne portent qu'un `README.md` (« traductions
non encore créées »). **Aucun équivalent DE/IT/EN n'est à écrire** ; les traductions à venir sont l'affaire de leurs stories. Les
ajouts portent « depuis la version~0.20.0 » (`docs-numeros-de-release`). `git grep -n "brouillon" -- docs/manual/fr/user-manual.tex` ; à
**réécrire**, non compléter (V1-8) :
- `user-manual.tex:2208-2210` (« Si la facture a été modifiée entre-temps dans une autre session, l'enregistrement est refusé et
  vous invite à recharger la page ») : décrire la relecture, « déjà enregistré », les champs nommés (une ligne modifiée ailleurs
  est nommée, non confondue avec l'enregistrement de l'utilisateur) ;
- la phrase **« La page de *modification* d'un brouillon ne relit pas encore la facture : après une coupure pendant l'enregistrement,
  rechargez-la avant de réessayer. »** (fin de la ligne `:2368` et lignes `:2369-2370`, V2-9) : **supprimer la phrase seule**, pas la
  ligne entière (`:2368` commence par « refuse, et la fiche relit la facture pour montrer ce qui est écrit. », qui reste vrai) ; le PDF
  aplati la contient (`pdftotext … | tr '\n' ' ' | tr -s ' ' | grep -c 'ne relit pas encore la facture'` rend 1), à vérifier absente
  après `make fr` ;
- à **ajouter** : une phrase sous « Créer la facture d'un document » (`user-manual.tex:2682`, la complétion ; non `:2234`, qui est
  « Le document d'un brouillon ») et sous la saisie (`:2082`) : après une réponse perdue Kesh relit et ne double pas sans
  « Enregistrer quand même » ; un paragraphe sous `sec:comptes-bancaires` (`:2802`), sur le modèle de `:1524` (#953), archivage compris.
- `admin-manual.tex` : aucun changement attendu (contrat d'API inchangé ; ses mentions `:2768-2842` sont de l'historique de
  versions) ; à constater à l'étape 0. Les PDF ne se commitent pas.

## Gates

Selon `e0190-dev.md` et `e35-dev-commun.md` § *Quels gates, quand* (V1-9). Les catalogues vivent dans `crates/kesh-i18n/locales/`,
lus par des tests Rust (`loader.rs` `parity_between_locales`, `no_new_select_expression_reaches_the_frontend_dictionary`,
`kesh-api/tests/textes_coherents.rs`) : **aucune modification Rust, mais les gates Rust se lancent**.
- Entre les passes : `npm --prefix frontend run check`, `lint-i18n-ownership`, vitest ciblé.
- **Au passage en `review`** : `scripts/test-fast.sh --ci`, puis `scripts/gate-ci.sh`, plus `npm run check`, `lint-i18n-ownership`,
  `test:unit`, `build`.
- **À la clôture** : `gate-ci` et **E2E complet** au dernier commit de code (montage `docs/testing.md`, `KESH_COOKIE_SECURE=false`,
  `KESH_TEST_MODE=true` des deux côtés) ; « vert » = les seuls échecs attendus.
Sous `scripts/mem-guard.sh`, une commande lourde à la fois, `flock` du verrou de la voie.

## Note de version

Factures fournisseur : après une réponse perdue (coupure, délai), Kesh relit au lieu de laisser un second clic doubler le brouillon
(il signale le brouillon semblable et demande « Enregistrer quand même »), reconnaît une modification ou une complétion déjà
enregistrée au lieu de l'annoncer en échec ; sur l'écran des comptes bancaires, un conflit de version (modification ou
archivage) montre l'écriture concurrente avant de proposer de la remplacer (refs #975, refs #984, refs #985, refs #986).

## Dev Notes

- **Hors périmètre** : voir l'inventaire (issues #1354 à #1362, #1365 à #1373 ; #1373 porte aussi `users/+page.svelte:122`, `:168`, `:209`, `:254`).
- **Fichiers communs** (tableau du fichier d'épique) : `i18n-keys.test.ts` (`sitesTotal`) et les catalogues, avec 48-1, 48-2, 48-10,
  48-13, 48-15 ; manuels `.tex`. `supplier-invoices/[id]/+page.svelte` (48-1) et `SupplierCreatePanel.svelte` (48-15, fusionnée)
  ne sont **pas** touchés. Côté serveur : rien.
- **Modules** : voir « Modules et découpage » et « Dérogation règle de splitting » (six au grain du fichier d'épique ; story unique par décision du 2026-11-19).
- **Ce qui doit rester vrai** : aucune saisie n'est jamais vidée par un refus ; une facture client, un avoir et leur PDF sont
  inchangés ; la règle « un appariement propose, il ne crée jamais » (le signalement ne refuse rien).
- **Coordination** : la PR #1327 de Beat ne touche aucun de ces fichiers.
- **Références** : fiches 34-5, 34-6, 34-8 ; `company-conflict.ts` ; `unknown-outcome.ts` ; `supplier-invoice-unknown-outcome.spec.ts`.
- **Statut** : `drafted` jusqu'à la fin de la validation ; `48-12-reponses-perdues-fournisseurs: ready-for-dev` se pose ensuite (orchestrateur, `sprint-status.yaml`).

## Dev Agent Record

(vide)

## Change Log

- 2026-11-19 — Création de la fiche (Claude Sonnet 5.5, `bmad-create-story`). Lecture au code à `af4e0e313`. Écart relevé à l'issue #984 : sa prémisse (« écrase ») ne se vérifie plus à la tête de la branche (l'écran renvoie la version périmée et rend de nouveau 409, sans rien expliquer) ; l'exigence de montrer avant de remplacer est reprise de l'issue.
- 2026-11-19 — Validation P1 (Opus, 0/0/10/8 ; `kesh-gate-logs/e4812/validation-p1.md`), remédiation (Sonnet) : V1-1 machine d'états de D2 (« quand même » n'envoie sans relire) ; V1-2 libellé neutre, total quand le numéro manque ; V1-3 trois issues de D3 ; V1-4 relecture en échec explicite (D1, D4, D5) ; V1-5 règle de comparaison D1-bis ; V1-6 phrase corrigée, archivage dans le lot B ; V1-7 inventaire rejoué, commandes et sortie écrites, sites absents ajoutés ; V1-8 deux phrases du manuel à réécrire ; V1-9 gates Rust ajoutés ; V1-10 deux lots, six modules au grain de l'épique, décision en attente ; V1-11 à V1-18 (citations, liste des modules, tests manquants, filtre de méthode E2E, valeurs lisibles, statut non brouillon, commandes, angle mort du détachement). Aucun fait de la remédiation n'a été relu par une seconde passe : P2 requise (Sonnet).
- 2026-11-19 — Validation P2 (Sonnet, 0/0/2/11 ; `kesh-gate-logs/e4812/validation-p2.md`), remédiation : V2-1 manuel français seul, AC et tâche C2 corrigés ; V2-2 égalité des lignes sur cinq champs, test et mutation ; V2-3 lien en nouvel onglet ; V2-4 `retry: false` et `CallOptions` sur les trois relectures ; V2-5 garde après `preventDefault()` ; V2-6 référence normalisée comme le serveur ; V2-7 base mise à jour à chaque relecture, champs d'une carte, valeurs en chaîne ; V2-8 26 sites, commande 4, sites non examinés vers #1373 ; V2-9 phrase du manuel supprimée seule, « depuis la version 0.20.0 » ; V2-10 mutations nommées ; V2-11 texte de repli de `supplier-invoices-err-stale` réécrit ; V2-12 AC renumérotés (AC-1 à AC-13) ; V2-13 statut `drafted`. Numéros d'issues reportés (#1356, #1365 à #1373). Une passe P3 (Opus), qui peut être ciblée sur ce commit, reste à faire.
- 2026-11-19 — Validation P3 ciblée (Opus, 0/1/2/2 ; `kesh-gate-logs/e4812/validation-p3-ciblee.md`), remédiation (Sonnet) : V3-1 (HIGH) l'égalité « déjà enregistré » exige le même nombre de lignes, puis chaque ligne sur cinq champs (D1-bis, D4, AC-6, tests « ligne en moins / en plus », mutation `zip` sans contrôle de longueur) ; V3-2 référence libre rognée par l'ensemble de blancs du serveur, vide ⇔ `null`, test U+200B ; V3-3 commande 3 élargie à `frontend/src` hors `*.api.ts`, sortie copiée, `users/+page.svelte:122`, `:168`, `:209`, `:254` vers #1373, stores `mode` et `session-idle` classés hors livres ; V3-4 commandes renumérotées 1 à 4 ; V3-5 la saisie comparée est le corps envoyé, la base la réponse du serveur. Dérogation à la règle de splitting ajoutée (décision de Guy du 2026-11-19), « découpage en attente » et 48-12a / 48-12b retirés. Statut laissé à `drafted`.
  **Table des renvois** (AC / mutations M / décisions D) : AC-1 : D2, D1-bis ; M : `trim`, total sans numéro. AC-2, AC-3 : D1, D2. AC-4 : D2 ; M : relire au clic « quand même ». AC-5 : D3 ; M : complété sans `supplierInvoiceId`, « déjà traité » sur relecture en échec. AC-6 : D4, D1-bis ; M : normaliseur décimal, champs QR, total et nombre seuls, **`zip` sans contrôle de longueur**, normalisation de la référence. AC-7 : D6 ; M : retirer la garde. AC-8 : D5, D1-bis ; M : `normalizeIban`, version adoptée sur relecture en échec. AC-9 : D5 ; M : ne pas relire au second essai. AC-10 : D6 ; M : garde avant `preventDefault()`. AC-11 : tous. AC-12 : D4 (repli), D5. AC-13 : manuel français.
