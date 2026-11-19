# Story 48-12 : Réponses perdues — factures fournisseur et comptes bancaires

Status: ready-for-dev

Issues : #975, #984, #985, #986 (toutes `bug`, **P2**). Release 0.20.0, Epic 48. Voie : frontend (aucun code Rust, aucune migration).

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
   `SupplierInvoiceForm.svelte:464-478` attrape l'erreur et l'affiche sans rien relire ; `saving` retombe à faux
   (`:476-478`), le bouton se réactive (`:977`). Un `TIMEOUT` ou un `NETWORK_ERROR` après le commit donne donc un second
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
   l'écriture concurrente). La fiche exige donc de montrer d'abord, comme la 34-6 pour la société. Les gestes sans saisie
   (archiver, clore, rouvrir) relisent déjà la liste : `refreshAfterVersionConflict`, `:660-662`.
6. **Le précédent de Kesh : relire, juger, montrer, jamais d'idempotence serveur.** `git grep -n -i "idempot"` ne trouve aucune
   clé d'idempotence dans `crates/` ni dans `frontend/src` (les occurrences sont des commentaires sur des gestes rejouables,
   ex. `bank_accounts.rs:1644`). Les patrons existants sont tous « relecture + jugement » :
   - 34-6 (#953), la société : `frontend/src/lib/features/settings/company-conflict.ts` — `companyEditConflict` (fusion à trois
     voies, issues `unchanged` / `already-saved` / `changed`), `conflictNotice`, `CompanyConflictNotice.svelte`, branchés dans
     `settings/+page.svelte:295-320` ;
   - 34-5 (#951) et 34-8 (#938), règlements et fiche fournisseur : `unknown-outcome.ts:65-68` (`isSettlementOutcomeUnknown`),
     dont le doc-comment (`:56-62`) écrit que la page d'édition (#986), la complétion (#985) et la création (#975) **ne sont pas
     couvertes** : c'est le périmètre de cette story.
7. **Un décor d'E2E existe pour la réponse perdue.** `frontend/tests/e2e/supplier-invoice-unknown-outcome.spec.ts:1-12` : la
   coupure est réelle (`route.fetch()` envoie au serveur, puis `route.abort()` jette la réponse) ; un `abort()` seul n'écrirait rien.

## Inventaire par le symptôme

Symptôme : « une écriture dont la réponse peut se perdre se renvoie sans relecture, et crée un doublon ou écrase ». Commande :

```sh
git grep -nE "apiClient\.(post|put)(<[^>]*>)?\(" -- 'frontend/src/lib/features/*.api.ts' 'frontend/src/lib/features/*/*.api.ts' ':!*.test.ts'
git grep -lE "isUnknownOutcome|isSettlementOutcomeUnknown" -- frontend/src ':!*.test.ts'
```

La seconde commande rend : `AutoBackupSection.svelte`, `AdminRestorePanel.svelte`, `mcp-sends-format.ts`, `unknown-outcome.ts`,
`invoices/[id]/+page.svelte`, `invoices/due-dates/+page.svelte`, `supplier-invoices/[id]/+page.svelte`. **Aucun écran de
création n'est parmi eux.** Les créations appelées depuis un écran (même commande, `createX(` dans `routes/` et `lib/**/*.svelte`) :

| Site | Écran | Rejeu sans relecture | Décision |
|---|---|---|---|
| `createSupplierInvoice` | `supplier-invoices/new` | doublon de brouillon (#975) | **dans la story** |
| `completeImport` | `supplier-invoices/new?document=` | échec annoncé à tort (#985) | **dans la story** |
| `updateSupplierInvoice` | `supplier-invoices/[id]/edit` | échec annoncé à tort (#986) | **dans la story** |
| `updateBankAccount` | `bank-accounts` (`edit-full`) | 409 sans relecture (#984) | **dans la story** |
| `updateBankAccountJournalLink` | `BankAccountJournalLinkForm.svelte:94-102` (version de la prop, `catch` ne relit pas) | 409 en boucle sur la version périmée | hors périmètre, **issue à ouvrir** (O1) |
| `createInvoice` | `InvoiceForm.svelte` | doublon de facture client | hors périmètre, **issue à ouvrir** (O2) — touche `InvoiceForm.svelte`, fichier de l'Epic 49 |
| `createJournalEntry` | `JournalEntryForm.svelte` | doublon d'écriture | hors périmètre, **issue à ouvrir** (O3) |
| `createPaymentBatch` | `payment-batches/+page.svelte` | doublon de lot de paiement | hors périmètre, **issue à ouvrir** (O4) |
| `createCreditNote` | `invoices/[id]/+page.svelte` | doublon d'avoir | hors périmètre, **issue à ouvrir** (O5) |
| `createContact` | `SupplierCreatePanel.svelte`, `contacts/+page.svelte` | doublon de contact | hors périmètre, **issue à ouvrir** (O6) |
| `createBankAccount` | `bank-accounts` (création) | doublon de compte | hors périmètre, **issue à ouvrir** (O7), à vérifier d'abord : un IBAN doublon est peut-être déjà refusé |
| `createAccount`, `createProduct`, `createProject`, `createFiscalYear`, `createBankProfile`, `createApiKey` | écrans correspondants | doublon (la plupart sont refusés par une unicité du schéma, à constater) | hors périmètre, **issue à ouvrir** (O8, une seule constatation à vérifier, scindée si elle se confirme) |

La règle du dépôt dit une issue par problème : l'orchestrateur ouvre O1 à O8 (O8 se scinde par écran dès qu'un site se confirme).
Le **module** posé ici (voir D1) est conçu pour servir à ces sites sans réécriture.

## Décisions

**D1. Un motif commun : relire, juger, montrer ; aucune clé d'idempotence serveur.**
Chaque écran, sur un refus qui peut venir d'une écriture déjà faite (409 de version, `IMPORT_NOT_PENDING_COMPLETION`) ou sur une
issue inconnue (`isSettlementOutcomeUnknown`), **relit l'état écrit**, le **compare à la saisie** et dit lequel des trois cas
vaut : *déjà enregistré*, *rien n'a été écrit*, *autre chose a été écrit*. La saisie reste toujours en place.
*Écartée — clé d'idempotence serveur* : elle exige un stockage durable des clés, donc une **migration**, et la release n'en
admet qu'une (celle de la 49-1) ; en mémoire, elle serait perdue au redémarrage du conteneur et ne protégerait pas le cas qui
compte (coupure puis redémarrage ou bascule). *Écartée — désactiver le bouton après une issue inconnue* : l'utilisateur dont
l'écriture n'a pas eu lieu serait bloqué et devrait ressaisir. *Écartée — « détection de la création déjà faite » côté serveur
sur le contenu* : refuserait une facture légitimement identique (deux factures du même montant, même jour, même fournisseur) ;
le dépôt signale, il ne tranche pas (`CLAUDE.md`, « un appariement propose, il ne crée jamais »).

**D2. #975 : relire les brouillons et signaler un brouillon identique, sans jamais refuser.**
Après une issue inconnue de `createSupplierInvoice`, l'écran lit `listSupplierInvoices({ status: 'draft', … })` et cherche les
brouillons de **même fournisseur, même date de facture, même numéro** (comparé après `trim`, absent = absent). Trois verdicts :
*trouvé* (liste des brouillons trouvés, chacun avec son total et un lien vers sa fiche, et le bouton d'envoi devient
« Enregistrer quand même »), *non trouvé* (« Aucune facture identique n'a été enregistrée : vous pouvez réessayer »),
*non vérifiable* (relecture en échec, ou plus de résultats que la page n'en rend : même bouton « Enregistrer quand même », et un
lien vers la liste). **La relecture se refait au clic suivant**, avant l'envoi, tant que le dernier essai a laissé l'issue
inconnue : elle couvre ainsi le cas où le serveur a fini d'écrire après la première relecture. Le total n'entre pas dans la
comparaison (le total de l'écran et celui du serveur peuvent différer d'un arrondi ; un faux négatif refabriquerait le doublon) ;
il s'affiche pour que l'utilisateur juge.
*Écartée — comparer aussi le total* : un faux négatif de calcul recrée le défaut.
*Angle mort assumé* : une requête restée en vol côté serveur au-delà de la relecture du clic suivant (délai de 30 s dépassé,
écriture tardive). Il se ferme seulement par une clé durable : **issue à ouvrir (O9)**, à planifier avec une migration.

**D3. #985 : sur `IMPORT_NOT_PENDING_COMPLETION` ou issue inconnue, relire les documents `completed`.**
`listImported('completed')`, recherche de `doc.id` : s'il porte un `supplierInvoiceId`, l'écran affiche un **avis**, non une
erreur : « Ce document a déjà été traité : la facture a été enregistrée en brouillon. » avec un lien vers
`/supplier-invoices/<id>`, la saisie restant en place et le bouton d'envoi désactivé (un nouvel envoi ne peut que re-refuser).
Sinon (document retiré, relecture en échec) : le message actuel (`supplier-invoices-err-not-pending`). L'avis est neutre sur
l'auteur : que ce soit le premier clic de l'utilisateur ou un collègue, la facture existe et le lien la montre.

**D4. #986 : relire la facture, juger, ne reprendre la version fraîche qu'après avoir montré.**
Sur un 409 de version ou une issue inconnue du `PUT`, la page relit `getSupplierInvoice(id, { retry: false })` et juge, par une
fonction pure (D7) qui compare `stored` à la saisie sur les seuls champs **envoyés** (avec QR, les coordonnées de paiement ne
partent pas : elles ne se comparent pas) :
- *déjà enregistré* (`stored` égale la saisie) : succès, notification « Brouillon enregistré », retour à la fiche ;
- *rien n'a changé ailleurs* (`stored` égale la copie lue à l'ouverture, hors version) : message « rechargé, vous pouvez
  enregistrer de nouveau », version fraîche adoptée ;
- *autre chose a été écrit* : avis qui **nomme les champs** en désaccord avec la valeur enregistrée (fournisseur, numéro, date,
  échéance, projet, lignes — pour les lignes, le total TTC et le nombre de lignes enregistrés), saisie intacte ; la version
  fraîche n'est adoptée **qu'à ce moment**, de sorte que le clic suivant dit, par l'avis, ce qu'il remplace (patron 34-6).
Relecture en échec : « Conflit de version et rechargement impossible — rechargez la page » (aucune version adoptée).
*Écartée — fusion à trois voies des lignes* : sans identifiant de ligne stable, la fusion ligne à ligne inventerait des
rapprochements ; un avis qui nomme et laisse décider suffit.

**D5. #984 : relire la liste, juger champ par champ avec `companyEditConflict`, montrer avant de remplacer.**
`submitUpdateFull` garde une **copie de base** prise à l'ouverture (`openEditFull`, `:385`). Sur `OPTIMISTIC_LOCK_CONFLICT`
(et sur une issue inconnue), `reload()` puis jugement du compte relu contre la base et la saisie sur les champs
`bankName`, `description`, `iban`, `qrIban`, `isPrimary`, `journalAccountId`, `cardLast4` (chacun rendu en texte avant
comparaison). *Déjà enregistré* : formulaire fermé, toast « déjà enregistré ». *Sinon* : la version relue est adoptée, les
champs non touchés prennent la valeur relue (le second clic ne rétablit pas l'ancienne valeur sur la correction d'un collègue),
les champs en désaccord sont **nommés avec la valeur enregistrée** et un avis dit qu'enregistrer de nouveau les remplacera. Si le
compte a disparu de la liste relue, `closeOrphanMode` agit déjà (message `staleNotice`).
**Réutilisation sans déplacer** : `companyEditConflict`, `conflictNotice` et `CompanyConflictNotice.svelte` (lignes sous les
champs seulement : leurs textes `settings-company-conflict-changed`, `-merged`, `-empty` sont neutres) restent dans
`features/settings/` et s'importent ; l'avis de bloc (« le compte bancaire a été modifié entre-temps… ») est un petit texte
propre à l'écran : **deux clés neuves, quatre locales**. *Écartée — déplacer la logique en `shared/`* : toucherait le module de la
34-6 et porterait la story à six modules pour un gain de dix lignes.

**D6. Une garde synchrone contre le double envoi.** `SupplierInvoiceForm.submit` (`:376`) revient d'emblée si `saving` : sans
cela, deux événements `submit` émis avant le rendu qui désactive le bouton envoient deux requêtes. Même garde en tête de
`submitCreate` et `submitUpdateFull` de la page des comptes bancaires (`submitting`).

**D7. Où vit la logique.** Pure et testable à part, dans `frontend/src/lib/features/supplier-invoices/` :
`resend-guard.ts` (verdict de D2, jugement de D4, comparaison de formes) ; l'avis de D3 et celui de D4 sont des composants du
même dossier. `isSettlementOutcomeUnknown` (`unknown-outcome.ts:65`) est **réutilisée telle quelle** ; son doc-comment
(`:56-62`) est réécrit, puisqu'il dit que ces trois écrans ne sont pas couverts.

## Critères d'acceptation

1. **AC-1 (#975).** Création sans document, réponse perdue après l'écriture : après la coupure l'écran relit les brouillons et
   montre le brouillon déjà créé (lien vers sa fiche) ; **le renvoi n'est plus un clic muet** : le bouton dit « Enregistrer quand
   même ». Cliquer sur le lien ne crée rien. Un seul brouillon existe tant que l'utilisateur n'a pas choisi « Enregistrer quand même ».
2. **AC-2 (#975).** Création dont la réponse est perdue **avant** l'écriture : l'écran dit qu'aucune facture identique n'existe et
   le renvoi crée un seul brouillon. La saisie n'a jamais été perdue.
3. **AC-3 (#975).** Relecture impossible : l'écran le dit, renvoie à la liste, et n'écrit rien sans le clic « Enregistrer quand même ».
4. **AC-4 (#975).** La relecture est refaite au clic suivant tant que l'essai précédent est resté d'issue inconnue.
5. **AC-5 (#985).** Complétion dont la réponse est perdue : le second clic ne rend pas un échec ; l'écran affiche l'avis « déjà
   traité » avec le lien vers la facture créée, saisie en place, envoi désactivé. Document retiré : message actuel.
6. **AC-6 (#986).** Modification enregistrée dont la réponse est perdue : le second clic ne dit pas « échec » ; la page le
   reconnaît (« Brouillon enregistré ») et renvoie à la fiche. Modification concurrente différente : avis qui nomme les champs,
   saisie intacte, version fraîche adoptée seulement à ce moment. Relecture impossible : message dédié, aucune version adoptée.
7. **AC-7 (#984).** Après un 409 sur l'édition d'un compte bancaire : la liste est relue ; *déjà enregistré* ferme le formulaire
   avec un toast ; sinon les champs en désaccord sont nommés avec la valeur enregistrée, les champs non touchés prennent la valeur
   relue, et le second clic ne part qu'avec la version relue, **après** cet avis. Aucun chemin n'envoie la version fraîche sans
   avoir montré l'écriture concurrente.
8. **AC-8 (D6).** Deux événements `submit` consécutifs sur chacun des trois formulaires n'envoient qu'une requête.
9. **AC-9 (production).** Rien ne s'écrit sur une facture client, un avoir ni leur PDF : la story ne touche aucun fichier de
   `kesh-api`, `kesh-db`, `kesh-qrbill`, `InvoiceForm.svelte`. Une facture fournisseur déjà validée n'est jamais modifiée par ces
   écrans (la page d'édition ne s'ouvre que sur un brouillon, `edit/+page.svelte:86`).
10. **AC-10.** Quatre locales : toutes les clés neuves existent en fr-CH, de-CH, it-CH, en-CH ; les gardes i18n (`sitesTotal`,
    `i18n-un-repli-par-cle`, `i18n-libelle-en-dur`) sont recomptées depuis la source.

## Tests à écrire, et la preuve du rouge

Chaque test neuf rougit sans le correctif ; journaux dans `/home/gcorbaz/devel/kesh-gate-logs/e4812/`.

| Test | Fichier | Rougit parce que |
|---|---|---|
| verdicts trouvé / non trouvé / non vérifiable, comparaison sans total | `resend-guard.test.ts` (neuf) | la fonction n'existe pas (mutation : retirer `trim` du numéro, ajouter le total) |
| jugement `already-saved` / `unchanged` / `changed` d'une facture, lignes comprises, champs QR exclus | `resend-guard.test.ts` | idem ; mutation : comparer aussi les champs QR |
| création : issue inconnue → relecture → brouillon trouvé → bouton « Enregistrer quand même », aucun second `POST` | `new-page.test.ts` | aujourd'hui le second clic poste |
| création : relecture refaite au clic suivant | `new-page.test.ts` | idem (mutation : ne relire qu'une fois) |
| complétion : 409 `IMPORT_NOT_PENDING_COMPLETION` → avis + lien, envoi désactivé | `new-page.test.ts` | aujourd'hui, message d'échec |
| édition : 409 → `stored` égale la saisie → succès et `goto` | `edit-page.test.ts` | aujourd'hui, message « rechargez » |
| édition : 409 concurrent → avis nommant les champs, version adoptée seulement après | `edit-page.test.ts` | idem (mutation : adopter la version avant l'avis) |
| double `submit` synchrone | `SupplierInvoiceForm.test.ts`, `bank-accounts-page.test.ts` | pas de garde |
| compte bancaire : 409 → relecture → `already-saved` ferme ; `changed` nomme les champs ; le second envoi porte la version relue | `bank-accounts-page.test.ts` | aujourd'hui, ni relecture ni avis |
| **E2E — création** : `route.fetch()` + `route.abort()` sur `POST /api/v1/supplier-invoices`, puis second clic ; contrôle par l'API qu'**un seul** brouillon existe avant « Enregistrer quand même » | `supplier-invoice-lost-response.spec.ts` (neuf) | aujourd'hui deux brouillons |
| **E2E — complétion et édition** (même coupure sur le `POST …/complete` et le `PUT`) | idem | échec annoncé à tort |
| **E2E — comptes bancaires** : modification concurrente par l'API entre l'ouverture et l'envoi ; avis, puis second clic | `bank-accounts-conflict.spec.ts` (neuf) | aucun avis aujourd'hui |

L'E2E est exigé : le défaut se voit dans le navigateur et à la frontière HTTP (coupure réelle). Il suit le patron de
`supplier-invoice-unknown-outcome.spec.ts` (décor `with-company`, fournisseur créé par l'API). Recette visuelle (`CLAUDE.md`) :
captures, thème clair, de chaque état neuf (brouillon trouvé, non vérifiable, avis « déjà traité », avis de conflit d'édition,
avis de compte bancaire) dans `kesh-gate-logs/e4812/captures/`, jointes au Dev Agent Record et au prompt de revue. Chaque mutation
est suivie d'un `touch` du fichier restauré (cargo/vite gardent le binaire muté).

## Tâches

1. Étape 0 : relire les points 2 à 5 de « Ce qui se lit au code » contre la branche d'épique mise à jour ; si l'un a bougé, le dire au Dev Agent Record.
2. `resend-guard.ts` et ses tests (D2, D4, D7).
3. `new/+page.svelte` : verdict de D2, avis de D3 ; `SupplierInvoiceForm.svelte` : garde D6, bouton « Enregistrer quand même », emplacement de l'avis.
4. `[id]/edit/+page.svelte` : D4 (avis, copie de base, version).
5. `bank-accounts/+page.svelte` : D5 et D6 ; avis de bloc.
6. `unknown-outcome.ts` : doc-comment réécrit (D7).
7. Catalogues i18n (quatre locales), `i18n-keys.test.ts` (`sitesTotal`) et les gardes de repli, recomptés depuis la source.
8. Manuel FR : voir « Manuels ».
9. E2E et captures ; gates (voir « Gates »).
10. Dev Agent Record, Change Log, Note de version.

## Manuels

`git grep -n "brouillon" -- docs/manual/fr/user-manual.tex` : la section « Le brouillon : modifier, valider, supprimer »
(`user-manual.tex:2195`, `sec:brouillon-fournisseur`) décrit la modification d'un brouillon ; la création et la complétion sont
décrites vers `:2082` et `:2234`. Le comportement du 409 de la société est déjà décrit à `:1524` (#953) : **le patron à
reprendre, avec les mêmes tournures**. À écrire dans `user-manual.tex` (français ; `de`/`it`/`en` suivent par leur propre
`.tex`, sans PDF) :
- une phrase sous la création et la complétion : après une réponse perdue, Kesh relit, montre la facture déjà enregistrée et
  ne la double jamais sans « Enregistrer quand même » ;
- un paragraphe sous la modification du brouillon (#986) : « déjà enregistré », ou les champs modifiés entre-temps, nommés ;
- un paragraphe sous `sec:comptes-bancaires` (`:2802`) : le conflit de version, sur le modèle de `:1524`.
`admin-manual.tex` : aucun changement attendu — le contrat de l'API ne change pas (ses mentions de `POST /api/v1/supplier-invoices`, `:2768-2842`, sont de l'historique de versions) ; à constater à l'étape 0. Les PDF ne se commitent pas.

## Gates

Selon `e35-dev-commun.md` § *Quels gates, quand* : voie frontend ; aucune modification Rust, donc `cargo` n'est pas lancé pour
cette story (à écrire au Dev Agent Record comme tel). Entre les passes : `npm --prefix frontend run check`,
`npm --prefix frontend run lint-i18n-ownership`, vitest ciblé des fichiers touchés. À la clôture : `npm run check`, `lint-i18n-ownership`,
`test:unit`, `build`, puis l'**E2E complet** (montage `docs/testing.md`, `KESH_COOKIE_SECURE=false`, `KESH_TEST_MODE=true` des deux
côtés) au dernier commit de code ; « vert » = les seuls échecs attendus. Sous `scripts/mem-guard.sh`, une commande lourde à la fois,
`flock` du verrou de la voie frontend.

## Note de version

Factures fournisseur : après une réponse perdue (coupure, délai), Kesh relit au lieu de laisser un second clic doubler le brouillon
(il montre le brouillon déjà enregistré et demande « Enregistrer quand même »), reconnaît une modification ou une complétion déjà
enregistrée au lieu de l'annoncer en échec, et, sur l'écran des comptes bancaires, montre l'écriture concurrente avant de proposer
de la remplacer (refs #975, refs #984, refs #985, refs #986).

## Dev Notes

- **Hors périmètre** (issues O1 à O9 ci-dessus, à ouvrir par l'orchestrateur avant la revue) : tous les autres écrans de création,
  `BankAccountJournalLinkForm` et la clé durable d'idempotence (O9).
- **Fichiers communs** (tableau du fichier d'épique) : `i18n-keys.test.ts` (`sitesTotal`) et les catalogues, avec 48-1, 48-2, 48-10,
  48-13, 48-15 ; manuels `.tex`. `supplier-invoices/[id]/+page.svelte` (48-1) n'est **pas** touchée ici ; `SupplierCreatePanel.svelte`
  (48-15, fusionnée) non plus. Côté serveur : rien (la mention « peut-être les mêmes route et repository » du kickoff tombe, D1).
- **Modules touchés** (grain du domaine frontend, crate) : `frontend/features/supplier-invoices`, `frontend/features/imported-supplier-invoices`
  (lecture seulement : `listImported('completed')` existe déjà, aucune édition probable), `frontend/features/bank-accounts` (+ la page),
  `kesh-i18n`, manuels. **Cinq au plus, pas de découpage.** Les routes `supplier-invoices/new`, `[id]/edit` et `bank-accounts` sont
  rattachées à leur domaine. Un sixième module (déplacer `company-conflict.ts` en `shared/`) a été écarté pour cette raison (D5).
- **Ce qui doit rester vrai** : aucune saisie n'est jamais vidée par un refus ; une facture client, un avoir et leur PDF sont
  inchangés ; `lines` d'une facture fournisseur validée ne sont jamais modifiées par la page d'édition ; la règle « un appariement
  propose, il ne crée jamais » (le signalement du brouillon identique ne refuse rien).
- **Coordination** : la PR #1327 de Beat ne touche aucun de ces fichiers.
- **Références** : fiches 34-5, 34-6, 34-8 ; `company-conflict.ts` ; `unknown-outcome.ts` ; `supplier-invoice-unknown-outcome.spec.ts`.
- **Statut à poser** (`sprint-status.yaml`, par l'orchestrateur) : `48-12-reponses-perdues-fournisseurs: ready-for-dev`.

## Dev Agent Record

(vide)

## Change Log

- 2026-11-19 — Création de la fiche (Claude Sonnet 5.5, `bmad-create-story`). Lecture au code à `af4e0e313`. Écart relevé à l'issue #984 : sa prémisse (« écrase ») ne se vérifie plus à la tête de la branche (l'écran renvoie la version périmée et rend de nouveau 409, sans rien expliquer) ; l'exigence de montrer avant de remplacer est reprise de l'issue.
