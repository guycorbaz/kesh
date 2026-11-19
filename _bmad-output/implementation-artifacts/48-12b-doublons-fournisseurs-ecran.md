# Story 48-12b : Écran — refus de doublon visible et complétion d'un document dont la réponse est perdue

Status: drafted (validation à faire ; `ready-for-dev` se pose à la clôture de la validation, par l'orchestrateur)

Issues : **#975** (côté écran) et **#985** (complétion d'un document importé annoncée en échec) — `bug`, **P2**. Release 0.20.0, Epic 48. Voie : **frontend**. Pas de migration, aucun code Rust. **Dépend de la 48-12a** (`48-12a-doublons-fournisseurs-serveur.md` : codes `409 SUPPLIER_INVOICE_DUPLICATE` et `409 SUPPLIER_INVOICE_PROBABLE_DUPLICATE`, `details`, champ `confirmNotDuplicate`, audit) : elle se fusionne **avant**. Fiche-mère : `48-12-reponses-perdues-fournisseurs.md`.

## Story

En tant que comptable qui enregistre une facture fournisseur,
je veux que, si Kesh refuse la facture comme doublon, l'écran **me montre la facture déjà enregistrée** sans perdre ma saisie (et me laisse confirmer quand le doute est permis), et que la complétion d'un document dont la réponse s'est perdue **dise la vérité**,
afin de ne jamais croire échouée une écriture réussie, ni enregistrer deux fois la même facture par un second clic.

**Usage réel.** Un second clic après une réponse perdue aboutit désormais, grâce à la règle de la 48-12a, au refus de doublon : la correction de #975 est visible ici. Pour #985, le second clic d'une complétion rend `IMPORT_NOT_PENDING_COMPLETION` alors que la facture existe : l'écran doit le dire.

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
   (`ImportNotPendingCompletion { current_status }`) ; son doc-comment le dit (`:126-127` : « un nouvel essai rend `409
   IMPORT_NOT_PENDING_COMPLETION`, et rien n'est perdu ni faussé »). Côté écran, `supplier-invoice-form.ts:484-490` traduit ce
   code en « Ce document a déjà été traité ou retiré de la liste. » : un échec, alors que la facture existe. Le document
   `completed` porte `supplierInvoiceId` (`imported-supplier-invoices.types.ts`, interface `ImportedSupplierInvoice`) et
   `listImported('completed')` existe (`imported-supplier-invoices.api.ts:50-52`).
4. **Un décor d'E2E existe pour la réponse perdue.** `frontend/tests/e2e/supplier-invoice-unknown-outcome.spec.ts:1-12` : la
   coupure est réelle (`route.fetch()` envoie au serveur, puis `route.abort()` jette la réponse) ; un `abort()` seul n'écrirait rien.


## Décisions

**D1. Le refus à l'écran.** `describeSubmitError` (`supplier-invoice-form.ts`) reconnaît les deux codes et le formulaire (`SupplierInvoiceForm.svelte`) affiche, **saisie intacte**, un avis qui **montre la facture existante** (numéro interne s'il existe, sinon « brouillon n° … » par son identifiant ; **lien vers sa fiche dans un nouvel onglet**, `target="_blank" rel="noopener"`). Pour `SUPPLIER_INVOICE_PROBABLE_DUPLICATE`, l'avis porte le bouton **« Ce n'est pas un doublon, enregistrer »** : il renvoie la même saisie avec `confirmNotDuplicate: true`, **une seule fois** ; toute modification du fournisseur, du numéro, de la date ou d'une ligne **efface l'avis** (la confirmation ne vaut que pour la saisie qui a été montrée). Pour `SUPPLIER_INVOICE_DUPLICATE`, **aucun bouton** : l'utilisateur corrige le numéro ou ouvre la facture existante. **La réponse perdue suivie d'un second clic aboutit à ce refus** : c'est la correction de #975, sans relecture ni verdict côté écran. Valable pour la création, la modification (`[id]/edit`) et la complétion (même formulaire).

**D2 (#985). Sur `IMPORT_NOT_PENDING_COMPLETION` ou issue inconnue, relire le document : quatre issues (V1-3).**
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

**D2 (suite).** Avec la règle serveur (48-12a), D2 ne garde que ce qui reste vrai : le second clic d'une complétion dont la réponse s'est perdue rend `IMPORT_NOT_PENDING_COMPLETION` (le statut du staging se juge avant la règle de doublon, 48-12a D4) ; l'écran **relit le document** pour dire la vérité. Les issues de D2 ci-dessus restent ; aucune relecture des brouillons n'est faite.

**D3. Garde synchrone contre le double envoi (écran).** `SupplierInvoiceForm.submit` (`:376`) revient d'emblée si `saving` : sans elle, deux événements `submit` émis avant le rendu qui désactive le bouton envoient deux requêtes dont la seconde serait **refusée** (doublon) mais produirait un avis trompeur ; le bouton de confirmation porte la même garde.

**D4. Où vit la logique.** **Écran** : `supplier-invoice-form.ts` (cas des deux codes), un petit composant d'avis dans `features/supplier-invoices/`.


## Critères d'acceptation

1. **AC-1 (#975).** Création : réponse perdue après l'écriture, puis second clic : l'écran montre l'avis de doublon (lien vers la facture existante, nouvel onglet), la saisie est intacte, **un seul** brouillon existe (cas du numéro ; sans numéro, l'avis probable).
2. **AC-2.** Refus probable : bouton « Ce n'est pas un doublon, enregistrer » qui renvoie une fois avec `confirmNotDuplicate: true` ; toute modification de la saisie (fournisseur, numéro, date, ligne) efface l'avis ; refus absolu : aucun bouton. Valable pour la création, la modification (`[id]/edit`) et la complétion.
3. **AC-3 (#985).** Complétion perdue : document complété = avis et lien, envoi désactivé ; encore à traiter = « aucune facture enregistrée, réessayez » ; retiré = message actuel ; relecture en échec = message d'issue inconnue ; document complété sans `supplierInvoiceId` = message d'issue inconnue.
4. **AC-4.** Deux événements `submit` consécutifs n'envoient qu'une requête ; le bouton de confirmation porte la même garde.
5. **AC-5.** Clés neuves (avis, bouton, messages de complétion) dans les quatre locales ; `sitesTotal` et les gardes de repli recomptés depuis la source.
6. **AC-6 (manuel).** `docs/manual/fr/user-manual.tex` décrit l'avis de doublon, le bouton de confirmation et le message de la complétion ; le contrôle final vérifie **positivement** « Ce n'est pas un doublon » dans le `.tex` et le PDF aplati après `make fr` (non commité).
7. **AC-7 (production).** Aucun fichier Rust, aucune facture client ni avoir ni PDF touché ; la saisie n'est jamais vidée par un refus.

## Tests à écrire, et la preuve du rouge

Chaque test neuf rougit sans le correctif ; journaux dans `/home/gcorbaz/devel/kesh-gate-logs/e4812b/`.

| Test | Fichier | Mutation qui doit le rougir |
|---|---|---|
| `describeSubmitError` des deux codes (`details` lus) ; avis + lien `target="_blank"` ; bouton (probable seulement) envoie une fois avec le drapeau ; une modification efface l'avis ; double `submit` synchrone | `supplier-invoice-form.test.ts`, `SupplierInvoiceForm.test.ts`, `new-page.test.ts`, `edit-page.test.ts` | pas de garde ; drapeau envoyé sans clic ; avis conservé après modification ; bouton aussi sur le refus absolu |
| complétion (D2) : 409 → avis + lien ; encore à traiter → réessayer ; retiré → message actuel ; relecture en échec ; complété sans `supplierInvoiceId` | `new-page.test.ts` | traiter « complété sans `supplierInvoiceId` » comme complété, « déjà traité » sur relecture en échec |
| **E2E** : `route.fetch()` + `route.abort()` sur `POST /api/v1/supplier-invoices` (numéro saisi), second clic → avis de doublon avec lien, un seul brouillon (contrôle par l'API) ; sans numéro, doublon probable puis confirmation → deux brouillons ; coupure de `POST …/complete` → avis « déjà traité » ou « réessayez » | `supplier-invoice-duplicate.spec.ts` (neuf) | aujourd'hui deux brouillons |

E2E exigé : défaut visible dans le navigateur et à la frontière HTTP ; **filtrer `route.request().method()` et ne couper qu'une fois** ; précédent `supplier-invoice-unknown-outcome.spec.ts:163`, `:283`. Recette visuelle (`CLAUDE.md`) : captures, thème clair, de chaque état neuf (avis absolu, avis probable avec bouton, avis de complétion) dans `kesh-gate-logs/e4812b/captures/`. Chaque mutation est suivie d'un `touch` du fichier restauré.

## Tâches

E1 : `supplier-invoice-form.ts` (cas des deux codes, `details`), `SupplierInvoiceForm.svelte` (avis, bouton, garde), composant d'avis (D1, D3). E2 : `new/+page.svelte` (D2 : relecture du document ; `listImported` reçoit un `CallOptions`, `{ retry: false }`), `[id]/edit/+page.svelte` (avis du formulaire). E3 : catalogues de l'écran (quatre locales), `sitesTotal`, gardes de repli. E4 : manuel utilisateur français. E5 : E2E et captures. C0 : relire les points 1 à 3 du code contre la branche d'épique à jour (48-12a fusionnée). C1 : gates. C2 : Dev Agent Record, Change Log, Note de version.

## Manuels et documentation

Seul `docs/manual/fr/user-manual.tex` existe pour l'utilisateur : **aucun équivalent DE/IT/EN à écrire**. Ajouts « depuis la version~0.20.0 » : l'avis de doublon et le bouton de confirmation (saisie `:2072-2090`, brouillon `:2195`), le message de la complétion (`:2682`, « Créer la facture d'un document »). La règle elle-même est décrite par la 48-12a. Les PDF ne se commitent pas.

## Gates

Selon `e0190-dev.md` et `e35-dev-commun.md` § *Quels gates, quand*. Voie frontend, aucun Rust modifié ; **les catalogues vivent dans `crates/kesh-i18n/locales/`, lus par des tests Rust** (`loader.rs` `parity_between_locales`, `kesh-api/tests/textes_coherents.rs`) : les gates Rust se lancent au passage en `review`.
- Entre les passes : `npm --prefix frontend run check`, `lint-i18n-ownership`, vitest ciblé.
- **Au passage en `review`** : `scripts/test-fast.sh --ci`, puis `scripts/gate-ci.sh`, plus `npm run check`, `lint-i18n-ownership`, `test:unit`, `build`.
- **À la clôture** : `gate-ci` et **E2E complet** au dernier commit de code (montage `docs/testing.md`, `KESH_COOKIE_SECURE=false`, `KESH_TEST_MODE=true` des deux côtés) ; « vert » = les seuls échecs attendus.
Sous `scripts/mem-guard.sh`, une commande lourde à la fois, `flock` du verrou de la voie.

## Note de version

Factures fournisseur : quand Kesh refuse une facture comme doublon, l'écran montre la facture déjà enregistrée (lien dans un nouvel onglet) et garde la saisie ; pour un doublon probable (même date et même montant, sans numéro à comparer), un bouton « Ce n'est pas un doublon, enregistrer » permet de confirmer. Une complétion de document dont la réponse s'est perdue dit désormais que la facture est enregistrée au lieu d'annoncer un échec (refs #975, refs #985).

## Dev Notes

- **Dépendance** : **48-12a fusionnée avant** (codes, `details`, `confirmNotDuplicate`).
- **Modules** : cinq au grain du fichier d'épique (`features/supplier-invoices`, route `supplier-invoices`, `features/imported-supplier-invoices` — `listImported` reçoit un `CallOptions` —, `kesh-i18n`, manuel français) ; découpage décidé par Guy le 2026-11-19.
- **Fichiers communs** : catalogues i18n, `i18n-keys.test.ts` (`sitesTotal`), manuels `.tex` avec les autres stories de l'Epic 48 et l'Epic 49 ; `supplier-invoices/[id]/+page.svelte` (48-1) et `SupplierCreatePanel.svelte` (48-15, fusionnée) ne sont pas touchés ; `InvoiceForm.svelte` (Epic 49) non plus.
- **Ce qui doit rester vrai** : aucune saisie n'est jamais vidée par un refus ; la confirmation ne vaut que pour la saisie montrée ; le refus absolu n'a pas de bouton.
- **Hors périmètre** : #986 (conflit de version) et le lot comptes bancaires, reportés à la 0.21.0 ; la correction du doc-comment de `shared/utils/unknown-outcome.ts` (issue P4 à ouvrir par l'orchestrateur).
- **Statut** : `drafted` ; `ready-for-dev` se pose à la clôture de la validation (orchestrateur).

## Dev Agent Record

(vide)

## Change Log

- 2026-11-19 — Création (Claude Sonnet 5.5, `bmad-create-story`) : fiche **découpée de la 48-12** (décision de Guy du 2026-11-19) ; porte l'écran de #975 et #985, à partir de la refonte (`a5026a3f2`). Dépend de la 48-12a. Historique des passes P1 à P6 : fiche-mère.
