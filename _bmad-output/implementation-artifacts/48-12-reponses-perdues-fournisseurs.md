# Story 48-12 : Réponses perdues — factures fournisseur (lot B, comptes bancaires, reporté à la 0.21.0)

Status: drafted (validation en cours ; `ready-for-dev` se pose à la clôture de la validation, par l'orchestrateur)

Issues dans le périmètre : #975, #985, #986 (toutes `bug`, **P2**) — **lot A**. Release 0.20.0, Epic 48. Voie : frontend (aucun code Rust, aucune migration).
**Hors périmètre actif** : #984 (comptes bancaires) et l'archivage d'un compte sur 409 — **lot B, reporté à la 0.21.0** par décision de Guy du 2026-11-19 (réintégrable dans la 0.20.0 si les ressources le permettent, décision de Guy) ; il reste écrit plus bas, sans AC actifs.

## Story

En tant que comptable qui tient des factures fournisseur réelles (dettes, lots de paiement pain.001),
je veux qu'une réponse perdue (délai, coupure) ou un conflit de version ne me fasse ni enregistrer deux fois la même
facture, ni remplacer en silence l'écriture d'un autre, ni croire échouée une écriture qui a réussi,
afin que mes dettes restent justes et qu'aucune facture ne soit payée deux fois, sans que je perde jamais ma saisie.

**Usage réel.** Guy saisit ses factures fournisseur et en tire des lots pain.001. Un doublon de brouillon, une fois validé,
double la dette (compte 2000) et peut être payé deux fois. C'est le seul des trois défauts qui touche les livres ; les deux
autres font croire à un échec ou font perdre du temps. Le correctif ne supprime jamais une saisie, **ne reprend jamais un champ
de lui-même** (aucune fusion automatique) : il **relit avant de proposer de réessayer**, et, en cas de conflit sur une
modification, il **montre les deux versions côte à côte** et laisse l'utilisateur choisir.

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
5. **(Lot B, reporté — #984 et l'archivage)** : voir la section « Lot B — reporté à la 0.21.0 », qui porte la lecture du code.
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
| `updateBankAccount`, `bank-accounts/+page.svelte:575` (`edit-full`) | 409 sans relecture (#984) | **B, reporté à la 0.21.0** (hors périmètre actif) |
| `archiveBankAccount`, `bank-accounts/+page.svelte:586-598` | 409 sans relecture, version périmée renvoyée (V1-6) | **B, reporté à la 0.21.0** (hors périmètre actif) |

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

**Décompte au grain du fichier d'épique** (`main:…/epic-48-corrections-0-20-0.md:205`, `:210-214`), les modules **modifiés**, **sans le lot B** (V5-3) :
1. `features/supplier-invoices` ; 2. la route `supplier-invoices` (`new`, `[id]/edit`) ; 3. `features/imported-supplier-invoices`
(`listImported` reçoit un paramètre `CallOptions`, `imported-supplier-invoices.api.ts:50`, V5-3) ; 4. `shared/utils`
(`unknown-outcome.ts`, doc-comment) ; 5. `kesh-i18n` ; 6. les manuels. **Six modules, au-dessus du plafond de cinq.** (Avec le lot B
réintégré : huit, par `features/bank-accounts` et la route `bank-accounts`.)
**Règle** : « la 48-12 se découpe si elle dépasse cinq modules, ou écrit sa dérogation » (`:218`) : la dérogation est **maintenue**,
sur le nouveau décompte.

## Dérogation règle de splitting

Décision du Project Lead (Guy) du 2026-11-19, option (a) de l'orchestrateur : la 48-12 ne se découpe pas. **À reconfirmer** avec la
réduction au lot A (le nombre de modules reste six, mais ce ne sont plus les mêmes ; le décompte précédent était faux, V5-3).
- **Six modules** (liste ci-dessus) pour **un seul motif** — « relire, juger, montrer » (D1) — sur **une seule famille d'écrans**, les factures fournisseur : une création, une complétion, une modification. Aucun module n'est propre à un lot : le lot B, reporté, ne fait pas partie du périmètre.
- **Ce motif sort du critère du `CLAUDE.md`** (qui vise un découpage créant des cycles de dépendances ou des fusions intermédiaires intestables) : la dérogation est un **choix du Project Lead**, non une exemption de droit. Découper trois écrans voisins qui partagent `resend-guard.ts`, les catalogues, `sitesTotal` et le manuel créerait des fiches qui se recoupent.
- **Signaux de recyclage** : le HIGH de la P3 (V3-1) est né du correctif de la P2 (V2-2) ; le HIGH de la P5 (V5-1) est né du correctif de la P4 (V4-8). Les deux signaux ont été **déclarés au Project Lead par l'orchestrateur** (le premier le 2026-11-19, le second le 2026-11-19) ; la réponse est la **réduction de la story au lot A** et la **suppression de la fusion automatique** (décision de Guy du 2026-11-19).

## Décisions

**D1 (commune). Un motif : relire, juger, montrer ; aucune clé d'idempotence serveur.**
Chaque écran, sur un refus qui peut venir d'une écriture déjà faite (409 de version, `IMPORT_NOT_PENDING_COMPLETION`) ou sur une
issue inconnue (`isSettlementOutcomeUnknown`), **relit l'état écrit**, le **compare à la saisie** et dit lequel des trois cas
vaut : *déjà enregistré*, *rien n'a changé ailleurs*, *conflit* (un seul nom par verdict, repris par D4 et les tests). La saisie reste toujours en place. **Une relecture qui
échoue se détecte explicitement et se dit** ; aucun jugement ne se fait sur un état périmé, et aucune version n'est adoptée alors.
*Écartée — clé d'idempotence serveur* : elle exige un stockage durable des clés, donc une **migration**, et la release n'en
admet qu'une (celle de la 49-1) ; en mémoire, elle serait perdue au redémarrage du conteneur. *Écartée — désactiver le bouton
après une issue inconnue* : l'utilisateur dont l'écriture n'a pas eu lieu serait bloqué et devrait ressaisir. *Écartée —
détection serveur sur le contenu* : refuserait une facture légitimement semblable ; le dépôt signale, il ne tranche pas
(`CLAUDE.md`, « un appariement propose, il ne crée jamais »).

**D1-bis (commune). Règle de comparaison (V1-5).** Un jugement compare `stored` (relu) à la saisie **sous la forme que le serveur
range**, jamais littéralement : montants, quantités et taux comparés en décimal (`Big.eq`, quatre décimales : `"100.0000"` égale
`"100.00"`) ; IBAN, QR-IBAN et référence normalisés comme le serveur (`normalizeIban`, `supplier-invoice-form.ts:334` ; blancs,
insécables compris, retirés) ; `trim`, et vide ⇔ `null`. **La référence de paiement (V2-6, V3-2)** se normalise comme le serveur (`supplier_invoices.rs:458-473`) : avec un QR-IBAN, tous les blancs sont retirés (`normalize_payment_field`) ; sans, elle est **rognée aux deux bouts par l'ensemble de blancs du serveur** (`trim_matches(is_payment_blank)`, `:471` ; `is_payment_blank` définie à `:378` : `char::is_whitespace`, dont U+0085, plus U+200B et U+FEFF), puis **vide après normalisation ⇔ `null`** (`present`, `:384`). `String.prototype.trim` ne retire ni U+200B ni U+0085 : une **fonction pure** de `resend-guard.ts` reproduit cet ensemble, testée avec un U+200B final collé. **Les lignes (V2-2, V3-1)** : l'égalité exige d'abord **le même nombre de lignes**, puis, **position par position, chaque ligne égale sur ses cinq champs** (description, quantité, prix unitaire, taux de TVA, compte de charge), chacun normalisé, puis **l'en-tête, dont les champs sont énumérés (V4-1)** : les neuf du corps du `PUT` — `contactId`, `supplierInvoiceNumber` (`trim`, vide ⇔ `null`), `invoiceDate`, `dueDate` (vide ⇔ `null`), `projectId`, et, **seulement sans QR verrouillé**, `creditorIban`, `creditorQrIban` (`normalizeIban`), `paymentReference` (règle ci-dessus), `expectedPaymentAmount` (décimal, vide ⇔ `null`, `Big.eq`) ; **omettre l'un d'eux jetterait une correction non écrite** (IBAN et montant attendu partent au lot pain.001). Une ligne **supprimée** ou **ajoutée** ne rend donc jamais *déjà enregistré* (le serveur remplace toutes les lignes, `repositories/supplier_invoices.rs:802`, `:585-599`). Le total, dérivé, ne sert qu'à l'**affichage** de l'avis. **Le côté « saisie » (V3-5)** est le **corps tel qu'il est parti** (le même objet que le `PUT`, issu de `linesForServer` et `paymentFields`, `supplier-invoice-form.ts:203-212`, `:328-345`, et `fNumber.trim() || null`), jamais le formulaire brut (« 12,50 », « 1'250 »), sur lequel `Big` lèverait ou différerait ; le côté « base » est la **réponse du serveur**, non les champs rouverts par `formLineFromResponse`. Le cas « je viens d'enregistrer moi-même » (réponse perdue, puis second clic) **doit** rendre
*déjà enregistré*, jamais *conflit* : un test le fixe avec les formes du serveur (`"100.0000"`, IBAN sans blancs), et
une mutation retire le normaliseur. Ces fonctions sont pures et testées à part (D7).

### Lot A — factures fournisseur

**D2 (#975). Relire les brouillons et signaler un brouillon semblable, sans jamais refuser ni affirmer.**
Après une issue inconnue de `createSupplierInvoice`, l'écran lit la liste des brouillons (requête ci-dessous) et cherche les
brouillons de **même fournisseur, même date de facture, même numéro** (`trim`, absent = absent) ; **quand le numéro manque,
le total normalisé (D1-bis) entre dans la comparaison** (sans numéro, deux factures légitimes du même jour se confondraient).
Le libellé est **neutre** : « Un brouillon semblable existe déjà », jamais « votre facture a été enregistrée » (V1-2). Chaque
brouillon trouvé montre son total, une marque « total différent de votre saisie » quand il diffère, et un lien vers sa fiche, **ouvert dans un nouvel onglet** (`target="_blank" rel="noopener"`, V2-3) : un lien ordinaire quitterait la page et ferait perdre une saisie que la fiche dit conserver ; le lien de D3, lui, peut rester ordinaire (la facture existe).
Verdicts : *trouvé* ; *non trouvé* (« Aucune facture semblable n'a été enregistrée : vous pouvez réessayer ») ; *non vérifiable*
(relecture en échec, détectée explicitement, ou plus de résultats que la page n'en rend ; message dédié et lien vers la liste).
**Requête de relecture (V4-3), avec ce que le serveur offre, sans le toucher** : `ListSupplierInvoicesQuery` n'a aucun filtre par fournisseur, date ou numéro. L'écran envoie `status: ['draft']` (un **tableau**), `q` = **le numéro saisi s'il existe**, sinon **le nom du fournisseur**, **tronqué au préfixe de 128 caractères** (`RECHERCHE_MAX_CARACTERES = 128`, `services/factures_fournisseur.rs:70` ; au-delà, le serveur rend 400 ; un nom de contact peut compter 255 caractères) (`q` cherche le numéro interne, le numéro du fournisseur et le nom : un `q` sur le nom rend aussi des homonymes, **filtrés côté client sur `contactId`**, y compris avec un `q` par numéro : `q` cherche aussi le numéro d'un autre fournisseur, `repositories/supplier_invoices.rs:329-331` ; test et mutation « omettre le filtre `contactId` » : un faux *trouvé* fait abandonner une vraie saisie), `limit: 100` (le tri par défaut est la date décroissante), et **parcourt les pages** (`offset`) jusqu'à `total`, avec un **plafond écrit de 5 pages** ; au-delà, verdict *non vérifiable*, **dit à l'écran** (jamais silencieux), avec le lien vers la liste. Le **total de la saisie** vient de `sumTtc` (`supplier-invoice-form.ts:88`, parité avec le serveur), comparé à `totalAmount` en `Big.eq`. Le cas courant est **vérifiable** : avec un numéro, `q` rend quelques brouillons ; sans numéro, les brouillons d'un seul fournisseur tiennent largement dans 500 lignes. Un test couvre un fournisseur dont les brouillons dépassent une page (parcours) et un autre qui dépasse le plafond (*non vérifiable* dit).
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
  été enregistrée » (la relecture ne lit que le document : la facture peut avoir été validée depuis, le lien montre son état) avec le lien `/supplier-invoices/<id>`, saisie en place, envoi désactivé ;
- *encore à traiter* : « Aucune facture n'a été enregistrée : vous pouvez réessayer », bouton actif ;
- *retiré* (ni l'un ni l'autre, relecture réussie) ou 409 dont le document est introuvable : le message actuel
  (`supplier-invoices-err-not-pending`) ;
- *relecture en échec* (détectée explicitement) : message d'issue inconnue, **jamais** « déjà traité » ;
- *document `completed` dont `supplierInvoiceId` est nul* (V4-5 ; pratiquement inatteignable : `delete_draft` détache le document, `ON DELETE SET NULL`) : message d'issue inconnue, jamais le lien.
L'avis est neutre sur l'auteur. *Angle mort assumé (V1-18)* : un document complété puis **détaché** par un tiers (il redevient
`to_complete`, `supplier_invoice_id` à `NULL`) se complète de nouveau : une complétion perdue, suivie d'un détachement puis d'un
renvoi, crée une seconde facture. Rare ; écrit, non traité.

**D4 (#986). Relire la facture, juger, et en cas de conflit montrer les deux versions côte à côte : aucune reprise sans clic.**
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

**D6. Garde synchrone contre le double envoi.** `SupplierInvoiceForm.submit` (`:376`) revient d'emblée si `saving` ; même garde sur les deux boutons du conflit de D4 (« Garder ma version », « Reprendre la version enregistrée »). (La garde des comptes bancaires est au lot B, reporté.)

**D7. Où vit la logique.** Pure et testable à part, dans `frontend/src/lib/features/supplier-invoices/` : `resend-guard.ts`
(machine d'états et verdicts de D2, jugement de D4 sans fusion, comparaisons de D1-bis) ; les avis de D3 et le **composant de conflit
côte à côte** de D4 sont dans le même dossier. `isSettlementOutcomeUnknown` (`unknown-outcome.ts:65`) est réutilisée telle quelle ;
son doc-comment (`:56-62`) est réécrit, ainsi que celui de `[id]/edit/+page.svelte:21-22` (« `OPTIMISTIC_LOCK_CONFLICT` invite à recharger »).

## Lot B — reporté à la 0.21.0 (réintégrable dans la 0.20.0 si les ressources le permettent, décision de Guy)

**Sans AC actifs** : cette section ne fait pas partie du périmètre de la story. Elle est conservée pour que le lot revienne sans réécriture. Il couvre **#984** (édition d'un compte bancaire) et l'**archivage** d'un compte sur 409 (V1-6). À sa réintégration : modules ajoutés `features/bank-accounts` et la route `bank-accounts` (**huit modules** au grain de l'épique), garde de double envoi de `submitCreate`, `submitUpdateFull` (après `e.preventDefault()`, `:468`, `:548-549`) et `confirmArchive`, avec la règle de D1-bis (`trim` des comptes bancaires comme `str::trim`, `bank_accounts.rs:506`, `:549`, test U+0085) et le **conflit sans fusion de D4** (« votre version » / « la version enregistrée », deux choix explicites) au lieu de la fusion `companyEditConflict` ci-dessous, qui est **écartée** pour la même raison que pour D4 ; à reconfirmer par une validation du lot.

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

## Critères d'acceptation

**Lot A (actifs)**
1. **AC-1 (#975).** Création sans document, réponse perdue après l'écriture : l'écran relit et signale « un brouillon semblable existe déjà » (total, marque « total différent » le cas échéant, lien en nouvel onglet). Un seul brouillon existe tant que l'utilisateur n'a pas cliqué « Enregistrer quand même ».
2. **AC-2 (#975).** Réponse perdue avant l'écriture : « aucune facture semblable » ; le renvoi crée un seul brouillon ; la saisie n'a jamais été perdue. Cas « brouillon préexistant, requête perdue avant écriture » : l'écran **n'affirme pas** que la saisie est enregistrée, et le renvoi reste possible.
3. **AC-3 (#975).** Relecture impossible : l'écran le dit, renvoie à la liste, n'écrit rien sans « Enregistrer quand même ».
4. **AC-4 (#975, machine d'états).** « Enregistrer quand même » envoie **sans relire** (exactement un `POST`) ; un clic ordinaire après verdict *non trouvé* relit une fois ; modifier fournisseur, date, numéro ou total remet le verdict à zéro.
5. **AC-5 (#985).** Complétion perdue : document complété = avis et lien, envoi désactivé ; **encore à traiter = « aucune facture enregistrée, réessayez »** ; retiré = message actuel ; relecture en échec = message d'issue inconnue.
6. **AC-6 (#986).** Modification enregistrée dont la réponse est perdue : reconnue (*déjà enregistré*), retour à la fiche, y compris avec les formes du serveur (`"100.0000"`, IBAN sans blancs, QRR avec espaces). **Un conflit n'est jamais fusionné** : *conflit* affiche « votre version » et « la version enregistrée » côte à côte (en-tête et lignes, champs en désaccord marqués) avec deux boutons, « Garder ma version » (renvoi avec la version relue) et « Reprendre la version enregistrée » (le formulaire prend la version relue) ; **tant qu'aucun bouton n'est cliqué, le formulaire et la version sont inchangés** ; aucun champ n'est repris en silence. Une ligne qui ne diffère que par sa description ou son compte, une ligne **supprimée** ou **ajoutée**, **un seul champ** quelconque de D1-bis non écrit (neuf d'en-tête, cinq de ligne) rendent *conflit*, jamais *déjà enregistré*. Scénario V5-1 : issue inconnue, relecture en échec, champ rétabli à sa valeur de base, 409, relecture réussie → *conflit* montré, rien repris. Relecture en échec : aucun jugement, aucune version adoptée. `ILLEGAL_STATE_TRANSITION` ou facture relue non brouillon : renvoi à la fiche (`supplier-invoices-edit-not-draft`) ; relecture en 404 : « Cette facture n'existe plus ».
7. **AC-7.** Deux événements `submit` consécutifs sur le formulaire des factures n'envoient qu'une requête ; les deux boutons du conflit portent la même garde.

**Communs**
8. **AC-8 (production).** Aucun fichier de `kesh-api`, `kesh-db`, `kesh-qrbill`, `InvoiceForm.svelte` ; ni facture client, ni avoir, ni PDF touchés ; la page d'édition ne s'ouvre que sur un brouillon (`edit/+page.svelte:86`).
9. **AC-9.** Clés neuves (D2, D3, D4 : conflit, 404, repli) dans les quatre locales ; `sitesTotal` et les gardes de repli recomptés depuis la source.
10. **AC-10 (manuel).** Les phrases devenues fausses sont **réécrites** dans `docs/manual/fr/user-manual.tex` (seul manuel existant : `docs/manual/{de,en,it}/` ne portent qu'un `README.md`) ; le contrôle final greppe « recharger la page » et « ne relit pas encore » (absents après la réécriture) et vérifie **positivement** « Enregistrer quand même » et « Garder ma version » (au moins une occurrence dans le `.tex` et le PDF aplati après `make fr`, non commité). Le manuel ne dit pas « déjà été traité » (il dit « déjà traité », `:2593`) : ce motif n'est pas contrôlé.

**Lot B — reporté** : aucun AC actif (relecture, version relue après avis, garde de double envoi, archivage : à réécrire à la réintégration, sur le conflit sans fusion).

## Tests à écrire, et la preuve du rouge

Chaque test neuf rougit sans le correctif ; journaux dans `/home/gcorbaz/devel/kesh-gate-logs/e4812/`. Les fixtures portent les **formes du serveur** (`"100.0000"`, IBAN sans blancs).

| Test | Fichier | Rougit parce que |
|---|---|---|
| verdicts *trouvé* / *non trouvé* / *non vérifiable* (dont fournisseur à plus d'une page de brouillons, et plus que le plafond de 5 pages) ; clé avec numéro ; clé avec total quand le numéro manque ; vrai doublon **et** facture légitime semblable ; **brouillon d'un autre fournisseur de même numéro, et homonyme → pas *trouvé*** ; `q` tronqué à 128 caractères | `resend-guard.test.ts` (neuf) | la fonction n'existe pas ; mutations : retirer `trim`, ôter le total quand le numéro manque, **omettre le filtre `contactId`** |
| machine d'états : « Enregistrer quand même » = un `POST`, sans relecture ; reset à la modification | `resend-guard.test.ts`, `new-page.test.ts` | aujourd'hui le second clic poste sans signal ; mutation : « relire aussi au clic Enregistrer quand même » |
| **comparateur, un test par champ (V5-4)** : test paramétré « seul le champ F diffère → jamais `already-saved`, verdict `conflict` » sur les **quatorze** champs — neuf d'en-tête (`contactId`, `supplierInvoiceNumber`, `invoiceDate`, `dueDate`, `projectId`, `creditorIban`, `creditorQrIban`, `paymentReference`, `expectedPaymentAmount`) et cinq de ligne (description, quantité, prix unitaire, taux de TVA, compte de charge) ; condition QR (champs de paiement exclus quand le QR est verrouillé) | `resend-guard.test.ts` | mutation « **retirer un champ du comparateur** », jouée pour **chacun** des quatorze champs (tableau de mutations, une ligne par champ) |
| jugement : `already-saved` (formes du serveur) / `unchanged` / `conflict` / `status ≠ draft` ; **`stored` moins sa dernière ligne, plus une ligne → jamais `already-saved`** ; QR-IBAN avec QRR saisie avec espaces → `already-saved` ; référence libre avec un U+200B final → `already-saved`, référence de seuls blancs du serveur ⇔ `null` | `resend-guard.test.ts` | idem ; mutations : normaliseur décimal, champs QR, total et nombre seuls, **`zip` sans contrôle de longueur**, normalisation de la référence, `trim` seul |
| création : brouillon trouvé → libellé neutre, marque « total différent » ; relecture impossible → aucun `POST` sans « quand même » | `new-page.test.ts` | aujourd'hui le second clic poste |
| complétion : 409 → avis + lien ; **encore à traiter → réessayer** ; retiré → message actuel ; relecture en échec | `new-page.test.ts` | aujourd'hui, message d'échec ; mutations : « complété sans `supplierInvoiceId` » traité comme complété, « déjà traité » sur relecture en échec |
| édition : 409 → `already-saved` → succès ; **409 concurrent → conflit côte à côte, formulaire et version inchangés avant tout clic** ; « Garder ma version » → renvoi avec la version relue ; « Reprendre la version enregistrée » → le formulaire prend la version relue ; **issue inconnue, relecture en échec, champ rétabli à sa base, 409, relecture → conflit montré, rien repris (V5-1)** ; relecture en échec → aucun jugement ni version ; **`ILLEGAL_STATE_TRANSITION` → relecture → fiche** ; **relecture 404 → « Cette facture n'existe plus »** | `edit-page.test.ts` | aujourd'hui, message « rechargez » ; mutations : **« reprise silencieuse d'un champ »**, « adopter la version sans clic », « adopter sur relecture en échec », « fusion d'en-tête automatique » |
| double `submit` synchrone ; double clic sur « Garder ma version » | `SupplierInvoiceForm.test.ts`, `edit-page.test.ts` | pas de garde ; mutation : retirer la garde `saving` |
| **E2E** : `route.fetch()` + `route.abort()` sur `POST /api/v1/supplier-invoices` (puis second clic ; l'API confirme **un** brouillon avant « quand même »), sur `POST …/complete` (y compris **coupure avant écriture** : le document reste à traiter, l'avis dit « réessayez »), sur le `PUT` ; **conflit** : modification concurrente par l'API, puis « Garder ma version » et « Reprendre la version enregistrée » | `supplier-invoice-lost-response.spec.ts` (neuf) | aujourd'hui deux brouillons / échec annoncé à tort / pas de conflit montré |

E2E exigé : défaut visible dans le navigateur et à la frontière HTTP. **Filtrer `route.request().method()` et ne couper qu'une fois** : la route du `PUT` (`/supplier-invoices/<id>`) intercepterait sinon la relecture `GET` de D4 ; de même le `POST /supplier-invoices` et la relecture `GET /supplier-invoices?status=draft` ; précédent `supplier-invoice-unknown-outcome.spec.ts:163` et `:283`. Recette visuelle (`CLAUDE.md`) : captures, thème clair, de chaque état neuf (dont le conflit côte à côte) dans `kesh-gate-logs/e4812/captures/`. Chaque mutation est suivie d'un `touch` du fichier restauré. *(Les tests du lot B, reporté, sont à écrire à sa réintégration.)*

## Tâches

**Lot A.** A1 : `resend-guard.ts` (D1-bis, D2 et sa requête de relecture, D4 sans fusion, D7) et ses tests ; `listSupplierInvoices` et `listImported` reçoivent `CallOptions` (V2-4). A2 : `new/+page.svelte` (D2, D3) et `SupplierInvoiceForm.svelte` (D6, bouton « Enregistrer quand même », avis). A3 : `[id]/edit/+page.svelte` (D4 : conflit côte à côte, deux boutons, remontage du formulaire sur `initial` repris, doc-comment) ; composant de conflit ; texte de repli de `supplier-invoices-err-stale`. A4 : `unknown-outcome.ts` (doc-comment).
**Communs.** C0 (étape 0) : relire les points 2 à 4 de « Ce qui se lit au code » contre la branche d'épique à jour. C1 : catalogues (quatre locales), `i18n-keys.test.ts` (`sitesTotal`) et gardes de repli recomptés. C2 : manuel français (`user-manual.tex`). C3 : E2E et captures. C4 : gates. C5 : Dev Agent Record, Change Log, Note de version.
**Lot B (reporté).** B1 : `bank-accounts/+page.svelte` (conflit sans fusion, archivage, garde, valeurs lisibles) ; `listBankAccounts` reçoit `CallOptions`. B2 : tests de la page.

## Manuels

Seul `docs/manual/fr/user-manual.tex` existe : `docs/manual/de/`, `it/` et `en/` ne portent qu'un `README.md` (« traductions non encore créées »). **Aucun équivalent DE/IT/EN n'est à écrire.** Les ajouts portent « depuis la version~0.20.0 » (`docs-numeros-de-release`). `git grep -n "brouillon" -- docs/manual/fr/user-manual.tex` ; à **réécrire**, non compléter :
- `user-manual.tex:2208-2210` (« Si la facture a été modifiée entre-temps dans une autre session, l'enregistrement est refusé et vous invite à recharger la page ») : décrire la relecture, « déjà enregistré », et le **conflit côte à côte** avec ses deux choix, « Garder ma version » et « Reprendre la version enregistrée » (rien n'est repris sans clic) ;
- la phrase **« La page de *modification* d'un brouillon ne relit pas encore la facture : après une coupure pendant l'enregistrement, rechargez-la avant de réessayer. »** (fin de la ligne `:2368` et lignes `:2369-2370`) : **supprimer la phrase seule**, pas la ligne entière (`:2368` commence par « refuse, et la fiche relit la facture pour montrer ce qui est écrit. », qui reste vrai) ; le PDF aplati la contient (`pdftotext … | tr '\n' ' ' | tr -s ' ' | grep -c 'ne relit pas encore la facture'` rend 1), à vérifier absente après `make fr` ;
- à **ajouter** : une phrase sous « Créer la facture d'un document » (`user-manual.tex:2682`, la complétion ; non `:2234`) et sous la saisie (`:2082`) : après une réponse perdue Kesh relit et ne double pas sans « Enregistrer quand même ».
- `admin-manual.tex` : aucun changement attendu (contrat d'API inchangé ; ses mentions `:2768-2842` sont de l'historique de versions) ; à constater à l'étape 0. Les PDF ne se commitent pas. *(Le paragraphe des comptes bancaires est au lot B, reporté.)*

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

Factures fournisseur : après une réponse perdue (coupure, délai), Kesh relit au lieu de laisser un second clic doubler le brouillon (il signale le brouillon semblable et demande « Enregistrer quand même »), reconnaît une modification ou une complétion déjà enregistrée au lieu de l'annoncer en échec ; en cas de conflit sur la modification d'un brouillon, il montre côte à côte votre version et la version enregistrée et laisse choisir, sans rien reprendre de lui-même (refs #975, refs #985, refs #986).

## Dev Notes

- **Hors périmètre** : le lot B (#984, archivage), reporté à la 0.21.0 ; voir l'inventaire (issues #1354 à #1362, #1363 « description tronquée à 44 caractères + « … » sur le PDF » et #1364 « facture validée trop longue pour son PDF » — **toutes deux hors de cette story** —, #1365 à #1373 ; #1373 porte aussi `users/+page.svelte:122`, `:168`, `:209`, `:254`).
- **Fichiers communs** (tableau du fichier d'épique) : `i18n-keys.test.ts` (`sitesTotal`) et les catalogues, avec 48-1, 48-2, 48-10, 48-13, 48-15 **et l'Epic 49** (fichiers communs du fichier d'épique) ; manuels `.tex`. `supplier-invoices/[id]/+page.svelte` (48-1) et `SupplierCreatePanel.svelte` (48-15, fusionnée) ne sont **pas** touchés. Côté serveur : rien.
- **Modules** : voir « Modules et découpage » et « Dérogation règle de splitting » (six, sans le lot B).
- **Ce qui doit rester vrai** : aucune saisie n'est jamais vidée par un refus ; **aucun champ n'est repris sans clic** ; une facture client, un avoir et leur PDF sont inchangés ; la règle « un appariement propose, il ne crée jamais » (le signalement ne refuse rien).
- **Coordination** : la PR #1327 de Beat ne touche aucun de ces fichiers.
- **Références** : fiches 34-5, 34-6, 34-8 ; `company-conflict.ts` (patron écarté pour D4) ; `unknown-outcome.ts` ; `supplier-invoice-unknown-outcome.spec.ts`.
- **Statut** : `drafted` jusqu'à la fin de la validation ; `48-12-reponses-perdues-fournisseurs: ready-for-dev` se pose ensuite (orchestrateur, `sprint-status.yaml`).

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
