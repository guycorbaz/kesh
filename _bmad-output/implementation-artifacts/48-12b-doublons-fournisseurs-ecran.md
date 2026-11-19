# Story 48-12b : Écran — refus de doublon visible, repassage en brouillon et complétion d'un document dont la réponse est perdue

Status: drafted (validation à reprendre ; `ready-for-dev` se pose à la clôture de la validation, par l'orchestrateur)

Issues : **#975** (côté écran) et **#985** (complétion d'un document importé annoncée en échec) — `bug`, **P2**. Release 0.20.0, Epic 48. Voie : **frontend**. Pas de migration, aucun code Rust. **Dépend de la 48-12a** (`48-12a-doublons-fournisseurs-serveur.md` : codes `409 SUPPLIER_INVOICE_DUPLICATE` et `409 SUPPLIER_INVOICE_PROBABLE_DUPLICATE`, `details`, champ `confirmNotDuplicate` — création, `PUT`, complétion et **dévalidation d'une facture annulée** —, audit) : elle se fusionne **avant**. **Les deux fiches sortent dans la même release : si la 48-12b n'est pas close à la coupure, les deux partent ensemble à la release suivante** ; aucune sortie isolée de la 48-12a. Fiche-mère : `48-12-reponses-perdues-fournisseurs.md`.

## Story

En tant que comptable qui enregistre une facture fournisseur,
je veux que, si Kesh refuse la facture comme doublon, l'écran **me montre la facture déjà enregistrée** sans perdre ma saisie (et me laisse confirmer quand le doute est permis), que le refus d'un repassage en brouillon soit **lisible**, et que la complétion d'un document dont la réponse s'est perdue **dise la vérité**,
afin de ne jamais croire échouée une écriture réussie, ni enregistrer deux fois la même facture par un second clic.

**Usage réel.** À la **création**, un second clic après une réponse perdue aboutit désormais, grâce à la règle de la 48-12a, au refus de doublon : la correction de #975 est visible ici. Pour #985, le second clic d'une complétion rend `IMPORT_NOT_PENDING_COMPLETION` alors que la facture existe : l'écran doit le dire. **À la modification**, le second clic après une réponse perdue rend un conflit de version (`OPTIMISTIC_LOCK_CONFLICT`), **hors périmètre** : c'est #986, reporté à la 0.21.0.

## Ce qui se lit au code

Tout est lu à `af4e0e313`.

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
   (`ImportNotPendingCompletion { current_status }`) ; son doc-comment le dit (`:125-126` : « un nouvel essai rend `409
   IMPORT_NOT_PENDING_COMPLETION`, et rien n'est perdu ni faussé »). Côté écran, `supplier-invoice-form.ts:484-490` traduit ce
   code en « Ce document a déjà été traité ou retiré de la liste. » : un échec, alors que la facture existe. Le document
   `completed` porte `supplierInvoiceId` (`imported-supplier-invoices.types.ts`, interface `ImportedSupplierInvoice`) et
   `listImported('completed')` existe (`imported-supplier-invoices.api.ts:50-52`).
4. **Un décor d'E2E existe pour la réponse perdue.** `frontend/tests/e2e/supplier-invoice-unknown-outcome.spec.ts:1-12` : la
   coupure est réelle (`route.fetch()` envoie au serveur, puis `route.abort()` jette la réponse) ; un `abort()` seul n'écrirait rien.


5. **Le formulaire est le seul à connaître la saisie.** Le corps n'est formé que dans `submit()` (`SupplierInvoiceForm.svelte:452-461` : `const base …`, `const req = qrLocked ? base : {...base, ...paymentFields(…)}`), puis remis à la page par `onSubmit(req)` (`:464`) ; `saving` est local (`:273`, `:462`, `:475`, bouton `:977`) ; l'erreur est attrapée **dans le formulaire** (`:465-473`, `describeSubmitError`, puis `formError`). Les champs vivent dans le formulaire : `fContactId`, `fNumber`, `fInvoiceDate`, `fDueDate`, `fProjectId`, `fIban`, `fReference`, `fExpectedAmount`, `fLines` (`:220-228`). Le formulaire ne lit `initial` qu'une fois.
6. **Deux prédicats d'issue inconnue.** `isUnknownOutcome` (`shared/utils/unknown-outcome.ts:29-37` : `TIMEOUT`, `NETWORK_ERROR`, 502, 504) et `isSettlementOutcomeUnknown` (`:65-68` : tout 5xx et toute erreur qui n'est pas une `ApiError` ; doc-comment : une panne pendant le `COMMIT` remonte en 503 alors que la base a pu valider).
7. **Le repassage en brouillon est un geste de la fiche.** `supplier-invoices/[id]/+page.svelte` (fichier de la **48-1**) porte « Repasser en brouillon » ; la 48-12a lui ajoute un refus `409 SUPPLIER_INVOICE_DUPLICATE` / `PROBABLE_DUPLICATE` avec `details` (la facture **active**) et `confirmNotDuplicate`.

## Décisions

**D1. Le refus à l'écran (création, modification, complétion).** `describeSubmitError` (`supplier-invoice-form.ts`) reconnaît les deux codes et le formulaire (`SupplierInvoiceForm.svelte`) affiche, **saisie intacte**, un avis qui **montre la facture existante** (numéro interne s'il existe, sinon « brouillon n° … » par son identifiant ; **lien vers sa fiche dans un nouvel onglet**, `target="_blank" rel="noopener"`). **Par chemin** : à la **création**, la réponse perdue suivie d'un second clic aboutit à ce refus (AC-1) ; à la **complétion**, au statut du document (D2) ; à la **modification**, le second clic rend `OPTIMISTIC_LOCK_CONFLICT` (message actuel, **#986, 0.21.0**, dit aux Dev Notes). L'**avis** et le bouton valent pour les trois écrans lorsque le serveur refuse un doublon (par exemple une modification qui change le numéro ou la date). Pour `SUPPLIER_INVOICE_PROBABLE_DUPLICATE`, l'avis porte le bouton **« Ce n'est pas un doublon, enregistrer »** ; pour `SUPPLIER_INVOICE_DUPLICATE`, **aucun bouton** (l'utilisateur corrige le numéro, en ajoutant un caractère s'il recommence sa numérotation en cours d'année, ou ouvre la facture existante).

**D1bis. Le bouton envoie la saisie montrée ; toute modification efface l'avis (V1-2).** *Choix (a), justifié* : **toute** modification d'un champ de la saisie après l'avis (fournisseur, numéro, date, échéance, projet, IBAN, QR-IBAN, référence, montant attendu, ligne) **efface l'avis et le bouton**, par un instantané de l'état du formulaire pris à l'affichage de l'avis et comparé à chaque changement. Le bouton renvoie alors le **corps qui est parti** (le `req` gardé de l'envoi refusé), qui **est** la saisie montrée : aucune correction faite après l'avis ne peut être perdue en silence, puisqu'elle efface l'avis et oblige à un envoi ordinaire. *Écartée — (b) `submit({ confirmNotDuplicate: true })` qui reforme le corps depuis l'état courant* : la confirmation porterait sur une saisie qui n'est plus celle qu'on a montrée (le serveur jugerait sur le nouveau contenu, avec une confirmation donnée pour l'ancien).

**D1ter. Une issue inconnue ne mène pas à un troisième brouillon (V1-3, V1-5).** Le formulaire retient que le **dernier envoi** (ordinaire ou de confirmation) a eu une **issue inconnue** (`isSettlementOutcomeUnknown`, qui couvre aussi le 503 après `COMMIT` ; un faux positif est sans danger, le serveur refusant le renvoi identique). Alors :
- à la **création**, le message d'issue inconnue est **propre** : « L'enregistrement a peut-être abouti : vérifiez la liste des factures de ce fournisseur avant de renvoyer » (avec le lien vers la liste), non le message générique « Réessayez ultérieurement » (`api-client.ts:367`) ;
- l'avis de refus qui suit **le dit** : « Votre envoi précédent a peut-être été enregistré : la facture ci-dessous est peut-être la vôtre » ; il offre **deux liens** (la facture citée, la liste des factures du fournisseur) ;
- pour un refus **probable**, le bouton « Ce n'est pas un doublon, enregistrer » **n'apparaît qu'après un clic sur l'un de ces deux liens** (le formulaire retient le clic) : l'utilisateur ne peut pas confirmer à l'aveugle une confirmation dont la réponse s'est perdue ; après un clic, le bouton apparaît, libellé inchangé ;
- sans issue inconnue précédente, le bouton est offert d'emblée.
Le signalement **propose, ne bloque jamais** : le bouton revient toujours après un clic. *Écartée — relecture automatique des brouillons côté écran* : la 48-12a a déplacé la protection dans le serveur.

**D1quater. Le refus du repassage en brouillon (fiche d'une facture annulée).** Sur `supplier-invoices/[id]/+page.svelte`, « Repasser en brouillon » d'une facture **annulée** qui rend `SUPPLIER_INVOICE_DUPLICATE` ou `SUPPLIER_INVOICE_PROBABLE_DUPLICATE` affiche le même avis (la facture **active** en `details`, lien en nouvel onglet) ; pour le refus probable, le même bouton « Ce n'est pas un doublon, enregistrer » renvoie le geste avec `confirmNotDuplicate: true`. La facture reste annulée tant que le refus tient. Le fichier est commun avec la **48-1** (réimputation) : le second arrivé résout. Même module que les autres écrans (route `supplier-invoices`).

**D2 (#985). Sur `IMPORT_NOT_PENDING_COMPLETION` ou issue inconnue, relire le document : quatre issues (V1-3).** Prédicat d'issue inconnue : **`isSettlementOutcomeUnknown`** (couvre le 503 après `COMMIT`, point 6 du code). Relire `listImported('completed')` puis, au besoin, `listImported('to_complete')` (aucun n'est paginé ; `listImported` reçoit un paramètre `CallOptions` et la relecture passe `{ retry: false }`) :
- *complété avec facture* (`supplierInvoiceId` renseigné) : **avis**, non erreur : « Ce document a déjà été traité : la facture a été enregistrée **telle qu'envoyée la première fois** ; ouvrez-la pour vérifier » (la saisie à l'écran peut différer de la facture enregistrée ; la relecture ne lit que le document : la facture peut avoir été validée depuis, le lien montre son état), avec le lien `/supplier-invoices/<id>`, saisie en place, envoi désactivé ;
- *encore à traiter* : « Aucune facture n'est enregistrée **pour l'instant** : vous pouvez réessayer » (une requête expirée peut encore tourner au serveur ; le renvoi est sans danger, le serveur le refuse ensuite), bouton actif ;
- *retiré* (ni l'un ni l'autre, relecture réussie) ou 409 dont le document est introuvable : le message actuel (`supplier-invoices-err-not-pending`) ;
- *relecture en échec* (détectée explicitement) : message d'issue inconnue, **jamais** « déjà traité » ;
- *document `completed` dont `supplierInvoiceId` est nul* (pratiquement inatteignable : `delete_draft` détache le document, `ON DELETE SET NULL`) : message d'issue inconnue, jamais le lien.
L'avis est neutre sur l'auteur. *Angle mort assumé* : un document complété puis **détaché** par un tiers (il redevient `to_complete`) peut se compléter de nouveau ; **après la 48-12a, un renvoi identique est refusé** (la facture d'origine existe toujours : refus de doublon, absolu avec un numéro, probable sans), si bien que l'angle mort se réduit au renvoi **modifié ou confirmé**. Rare ; écrit, non traité.

**D2bis. Le mécanisme entre la page et le formulaire (V1-4).** Le formulaire reçoit **deux props facultatives** de la page : `onRefusal?: (err) => Promise<RefusalNotice | null>` et `locked?: boolean`. Quand `onSubmit` rejette, le formulaire appelle **d'abord** `onRefusal(err)` s'il existe : la page de **complétion** y relit le document (D2) et rend un `RefusalNotice` (`{ kind: 'completed' | 'pending' | 'gone' | 'unknown', invoiceId? }`) ; le formulaire **affiche cet avis à la place de `formError`** (l'ancien message « Ce document a déjà été traité ou retiré de la liste » n'est **jamais** affiché en même temps) et, pour `completed`, la page passe `locked = true` qui désactive l'envoi (avec `saving`). Si `onRefusal` rend `null`, le chemin habituel (`describeSubmitError`) s'applique. Les autres pages ne passent pas `onRefusal`. *Écartée — relancer l'erreur d'origine après la relecture* : afficherait l'avis vrai **et** le message trompeur de #985.

**D3. Garde synchrone contre le double envoi (écran).** `SupplierInvoiceForm.submit` (`:376`) revient d'emblée si `saving` : sans elle, deux événements `submit` émis avant le rendu qui désactive le bouton envoient deux requêtes dont la seconde serait **refusée** (doublon) mais produirait un avis trompeur ; le bouton de confirmation porte la même garde.

**D4. Où vit la logique.** **Écran** : `supplier-invoice-form.ts` (cas des deux codes, `details` lus, prédicats), `supplier-invoices.types.ts` (`CreateSupplierInvoiceRequest` reçoit `confirmNotDuplicate?: boolean`, hérité par le corps du `PUT` et transmis par `completeImport` ; le corps de la dévalidation aussi), un composant d'avis dans `features/supplier-invoices/` ; `imported-supplier-invoices.api.ts` (`listImported` + `CallOptions`).

## Critères d'acceptation

1. **AC-1 (#975, création).** Réponse perdue après l'écriture, puis second clic : l'écran montre l'avis de doublon (lien vers la facture existante, nouvel onglet), la saisie est intacte, **un seul** brouillon existe (avec un numéro ; sans numéro, l'avis probable).
2. **AC-2.** Refus probable : bouton « Ce n'est pas un doublon, enregistrer » qui renvoie, une seule fois, **le corps de l'envoi refusé** avec `confirmNotDuplicate: true` ; **toute** modification d'un champ après l'avis (échéance, IBAN, référence, montant attendu, projet comprises) efface l'avis et le bouton ; refus absolu : aucun bouton. Valable lorsque le serveur refuse un doublon, à la création, à la modification et à la complétion.
3. **AC-3 (issue inconnue, V1-3).** Après une issue inconnue (envoi ordinaire ou confirmation coupée), l'avis dit que l'envoi précédent a peut-être été enregistré, offre la facture citée et la liste des factures du fournisseur, et **le bouton de confirmation n'apparaît qu'après un clic sur l'un des deux liens** ; à la création, le message d'issue inconnue invite à vérifier la liste avant de renvoyer.
4. **AC-4 (#985).** Complétion perdue : document complété = avis (« telle qu'envoyée la première fois ») et lien, envoi désactivé, **l'ancien message n'apparaît pas** ; encore à traiter = « aucune facture n'est enregistrée pour l'instant, vous pouvez réessayer » ; retiré = message actuel ; relecture en échec = message d'issue inconnue ; document complété sans `supplierInvoiceId` = message d'issue inconnue.
5. **AC-5 (repassage en brouillon).** Sur la fiche d'une facture annulée, « Repasser en brouillon » refusé par la règle affiche l'avis avec la facture active (nouvel onglet) ; refus probable : bouton qui renvoie avec `confirmNotDuplicate: true` ; refus absolu : aucun bouton ; la facture reste annulée.
6. **AC-6.** Deux événements `submit` consécutifs n'envoient qu'une requête ; le bouton de confirmation porte la même garde.
7. **AC-7.** Clés neuves (avis, bouton, messages de création et de complétion, avis de repassage) dans les quatre locales ; `sitesTotal` et les gardes de repli recomptés depuis la source.
8. **AC-8 (manuel).** `docs/manual/fr/user-manual.tex` décrit l'avis de doublon, le bouton de confirmation, le message de la complétion et le refus du repassage en brouillon ; le contrôle final vérifie **positivement** « Ce n'est pas un doublon » dans le `.tex` et **« Ce n’est pas un doublon »** (apostrophe typographique U+2019) dans le PDF aplati après `make fr` (non commité).
9. **AC-9 (production).** Aucun fichier Rust, aucune facture client ni avoir ni PDF touché ; la saisie n'est jamais vidée par un refus.

## Tests à écrire, et la preuve du rouge

Chaque test neuf rougit sans le correctif ; journaux dans `/home/gcorbaz/devel/kesh-gate-logs/e4812b/`.

| Test | Fichier | Mutation qui doit le rougir |
|---|---|---|
| `describeSubmitError` des deux codes (`details` lus) ; avis + lien `target="_blank"` ; bouton (probable seulement) renvoie **le corps de l'envoi refusé** avec le drapeau, une seule fois ; double `submit` synchrone | `supplier-invoice-form.test.ts`, `SupplierInvoiceForm.test.ts`, `new-page.test.ts`, `edit-page.test.ts` | pas de garde ; drapeau envoyé sans clic ; bouton aussi sur le refus absolu ; **bouton qui reforme le corps depuis l'état courant** |
| **avis probable puis correction de l'échéance, de l'IBAN, de la référence, du montant attendu, du projet** → l'avis et le bouton disparaissent ; une ligne modifiée aussi | `SupplierInvoiceForm.test.ts` | « avis conservé après modification de l'échéance » (et des autres champs, une mutation par champ) |
| **issue inconnue (V1-3)** : envoi coupé → message propre ; renvoi → avis « peut-être la vôtre », **sans bouton** ; clic sur le lien → bouton ; confirmation coupée, renvoi → même règle | `SupplierInvoiceForm.test.ts`, `new-page.test.ts` | bouton offert sans clic sur le lien ; avis identique après une issue inconnue |
| complétion (D2, D2bis) : 409 → avis « telle qu'envoyée » + lien, **ancien message absent**, envoi désactivé ; encore à traiter → « pour l'instant » ; retiré → message actuel ; relecture en échec ; complété sans `supplierInvoiceId` | `new-page.test.ts` | **relancer l'erreur d'origine après la relecture** ; traiter « complété sans `supplierInvoiceId` » comme complété ; « déjà traité » sur relecture en échec |
| fiche d'une annulée : « Repasser en brouillon » refusé → avis + lien ; probable → bouton → `confirmNotDuplicate: true` ; absolu → aucun bouton | `[id]/supplier-invoice-detail.test.ts` (existant ou neuf, nom à vérifier) | bouton aussi sur le refus absolu |
| **E2E** : `route.fetch()` + `route.abort()` sur `POST /api/v1/supplier-invoices` (numéro saisi), second clic → avis de doublon avec lien, un seul brouillon (contrôle par l'API) ; sans numéro, avis probable puis confirmation → deux brouillons ; **confirmation coupée → renvoi : avis « peut-être la vôtre », sans bouton avant le clic sur le lien** ; coupure de `POST …/complete` (le serveur a **complété**) → avis « déjà traité » **et** lien (« réessayez » ne passe pas) ; coupure **avant l'envoi** (`route.abort()` seul) → « réessayez » ; fiche d'une annulée : refus de repassage | `supplier-invoice-duplicate.spec.ts` (neuf) | aujourd'hui deux brouillons |

E2E exigé : défaut visible dans le navigateur et à la frontière HTTP ; **filtrer `route.request().method()` et ne couper qu'une fois** ; précédent `supplier-invoice-unknown-outcome.spec.ts:163`, `:283`. Recette visuelle (`CLAUDE.md`) : captures, thème clair, de chaque état neuf (avis absolu, avis probable avec bouton, avis après issue inconnue sans bouton, avis de complétion, refus de repassage) dans `kesh-gate-logs/e4812b/captures/`. Chaque mutation est suivie d'un `touch` du fichier restauré.

## Tâches

E1 : `supplier-invoice-form.ts` (cas des deux codes, `details`, prédicats), `supplier-invoices.types.ts` (`confirmNotDuplicate`), `SupplierInvoiceForm.svelte` (avis, bouton, instantané de la saisie, drapeau d'issue inconnue, props `onRefusal` et `locked`, garde), composant d'avis (D1 à D1ter, D2bis, D3). E2 : `new/+page.svelte` (D2 : relecture du document, `onRefusal`, `locked` ; `listImported` reçoit un `CallOptions`, `{ retry: false }`), `[id]/edit/+page.svelte` (avis du formulaire), `[id]/+page.svelte` (D1quater, avec la 48-1). E3 : catalogues de l'écran (quatre locales), `sitesTotal`, gardes de repli. E4 : manuel utilisateur français. E5 : E2E et captures. C0 : relire les points 1 à 3 du code contre la branche d'épique à jour (48-12a fusionnée). C1 : gates. C2 : Dev Agent Record, Change Log, Note de version.

## Manuels et documentation

Seul `docs/manual/fr/user-manual.tex` existe pour l'utilisateur : **aucun équivalent DE/IT/EN à écrire**. Ajouts « depuis la version~0.20.0 » : l'avis de doublon et le bouton de confirmation (saisie `:2072-2090`, brouillon `:2195`), le refus du repassage en brouillon (`:888`), le message de la complétion (`:2682`, « Créer la facture d'un document »). La règle elle-même est décrite par la 48-12a. Les PDF ne se commitent pas. Le contrôle du PDF aplati se fait avec l'apostrophe **typographique**.

## Gates

Selon `e0190-dev.md` et `e35-dev-commun.md` § *Quels gates, quand*. Voie frontend, aucun Rust modifié ; **les catalogues vivent dans `crates/kesh-i18n/locales/`, lus par des tests Rust** (`loader.rs` `parity_between_locales`, `kesh-api/tests/textes_coherents.rs`) : les gates Rust se lancent au passage en `review`.
- Entre les passes : `npm --prefix frontend run check`, `lint-i18n-ownership`, vitest ciblé.
- **Au passage en `review`** : `scripts/test-fast.sh --ci`, puis `scripts/gate-ci.sh`, plus `npm run check`, `lint-i18n-ownership`, `test:unit`, `build`.
- **À la clôture** : `gate-ci` et **E2E complet** au dernier commit de code (montage `docs/testing.md`, `KESH_COOKIE_SECURE=false`, `KESH_TEST_MODE=true` des deux côtés) ; « vert » = les seuls échecs attendus.
Sous `scripts/mem-guard.sh`, une commande lourde à la fois, `flock` du verrou de la voie.

## Note de version

Factures fournisseur : quand Kesh refuse une facture comme doublon, l'écran montre la facture déjà enregistrée (lien dans un nouvel onglet) et garde la saisie ; pour un doublon probable (même date et même montant, sans numéro à comparer), un bouton « Ce n'est pas un doublon, enregistrer » permet de confirmer, sauf après une réponse perdue où il faut d'abord vérifier la liste des factures du fournisseur ; le repassage en brouillon d'une facture annulée refusé par la règle est expliqué. Une complétion de document dont la réponse s'est perdue dit désormais que la facture est enregistrée au lieu d'annoncer un échec (refs #975, refs #985).

## Dev Notes

- **Dépendance et règle de sortie** : **48-12a fusionnée avant** (codes, `details`, `confirmNotDuplicate`, dévalidation). **Les deux sortent dans la même release** ; si la 48-12b n'est pas close à la coupure, les deux partent ensemble à la release suivante ; aucune sortie isolée de la 48-12a.
- **Modules** : cinq au grain du fichier d'épique (`features/supplier-invoices`, route `supplier-invoices` — `new`, `[id]/edit` et la fiche `[id]` —, `features/imported-supplier-invoices` — `listImported` reçoit un `CallOptions` —, `kesh-i18n`, manuel français) ; découpage décidé par Guy le 2026-11-19. L'ajout du refus de repassage reste dans la route `supplier-invoices` : le décompte ne change pas.
- **Fichiers communs** : catalogues i18n, `i18n-keys.test.ts` (`sitesTotal`), manuels `.tex` avec les autres stories de l'Epic 48 et l'Epic 49 ; `supplier-invoices/[id]/+page.svelte` **avec la 48-1** (le second arrivé résout) ; `SupplierCreatePanel.svelte` (48-15, fusionnée) et `InvoiceForm.svelte` (Epic 49) ne sont pas touchés.
- **Ce qui doit rester vrai** : aucune saisie n'est jamais vidée par un refus ; la confirmation ne vaut que pour la saisie montrée ; le refus absolu n'a pas de bouton ; après une issue inconnue, pas de confirmation à l'aveugle.
- **Hors périmètre** : **#986** (conflit de version à la modification : le second clic d'une modification dont la réponse s'est perdue rend un conflit de version, message actuel), le lot comptes bancaires, reportés à la 0.21.0 ; la correction du doc-comment de `shared/utils/unknown-outcome.ts` (issue P4 à ouvrir par l'orchestrateur).
- **Statut** : `drafted` ; `ready-for-dev` se pose à la clôture de la validation (orchestrateur).

## Dev Agent Record

(vide)

## Change Log

- 2026-11-19 — Création (Claude Sonnet 5.5, `bmad-create-story`) : fiche **découpée de la 48-12** (décision de Guy du 2026-11-19) ; porte l'écran de #975 et #985. Dépend de la 48-12a. Historique des passes P1 à P6 : fiche-mère.
- 2026-11-19 — Validation P1 de la 48-12b (Opus, 0/0/4/8 ; `kesh-gate-logs/e4812/validation-12b-p1.md`), remédiation (Sonnet). **Décisions de Guy du 2026-11-19 appliquées** : année civile et dévalidation d'une annulée (48-12a) ⇒ la 48-12b **ajoute à son périmètre l'avis du refus de repassage en brouillon** sur la fiche (D1quater, AC-5), sans changer le décompte de cinq modules (route `supplier-invoices`) ; **sortie commune** des deux fiches écrite (en-tête, Dev Notes). **Findings** : V1-1 la promesse « second clic → avis de doublon » se limite à la **création** ; modification = #986 reporté, dit en D1, Story et Dev Notes ; V1-2 **choix (a)** : toute modification après l'avis l'efface, le bouton renvoie le corps de l'envoi refusé (D1bis ; mutation « bouton qui reforme le corps ») ; V1-3 règle contre le troisième brouillon : après une issue inconnue, avis « peut-être la vôtre » et bouton **seulement après un clic** sur la facture citée ou la liste du fournisseur (D1ter, AC-3, E2E) ; V1-4 mécanisme `onRefusal` / `locked` entre la page et le formulaire, ancien message jamais affiché avec l'avis (D2bis, mutation « relancer l'erreur d'origine ») ; V1-5 message propre d'issue inconnue à la création (D1ter) ; V1-6 angle mort du détachement réécrit après la 48-12a ; V1-7 « pour l'instant » ; V1-8 E2E déterministe (« déjà traité » + lien, « réessayez » testé séparément par `route.abort()` seul) ; V1-9 apostrophe typographique U+2019 au contrôle du PDF ; V1-10 prédicat `isSettlementOutcomeUnknown` nommé et justifié ; V1-11 citation `:125-126`, `confirmNotDuplicate` au type de requête (D4) ; V1-12 « telle qu'envoyée la première fois ». **Relecture contre les sept décisions de Guy et contre la 48-12a** : l'écran n'implante aucune règle (le serveur décide), la confirmation vaut pour la saisie montrée, la dévalidation est affichée. Cinq modules.
