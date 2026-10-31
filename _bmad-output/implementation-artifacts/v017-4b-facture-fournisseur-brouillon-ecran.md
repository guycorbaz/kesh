# Story v017-4b : factures fournisseur — brouillon, validée, payée — l'écran

Status: ready-for-dev

*Sous-fiche issue du découpage de la Story v017-4 à sa rédaction (2026-10-31, C-v017-4-1). Même worktree et même
branche que la `v017-4a` ; **dépend de la 4a** (routes, codes, K8) : développement après le commit de son code.
Registre **commun** : `v017-4-choix-autonomes.md`. Issues : **#849**, #853, #830, #835, #831 — en `refs` seulement.
**Le texte des AC et tout le contexte sont dans la fiche parente `v017-4-facture-fournisseur-brouillon.md`.** Les
numéros de ligne cités sont ceux de `8e126cfd`.*

## Story

En tant que Guy, je veux voir et conduire à l'écran le cycle d'une facture fournisseur — un brouillon que je corrige,
que je valide, que je dévalide au besoin —, et que le formulaire ne me laisse plus enregistrer une échéance avant la
date, une date du jour qui n'est pas celle du document, une virgule que le serveur ne lit pas, ni ne m'affiche un
total arrondi qui contredit son propre message.

## Périmètre

Le frontend (liste et son banc neuf, fiche, page d'édition `/supplier-invoices/[id]/edit`, `SupplierInvoiceForm` et
ses helpers — #853, #830, #835 —, liste — #831 —, écran de clôture, écran des lots), les clés de l'écran aux quatre
catalogues, le manuel utilisateur et son PDF, l'E2E, les captures, le CHANGELOG (*Ajouté*, *Corrigé*) et le README.

## Critères d'acceptation

**AC-21 à AC-32** de la fiche parente, sous leur numéro d'origine. Mutations **M21 à M34**, M21-bis compris (M31 ajoutée par la décision
Q4, C-v017-4-24 : l'état validé s'affiche « Validée » aux quatre locales, et aucun texte de l'écran, des manuels ni de
l'API ne le nomme plus « ouverte » — AC-21, AC-29).

## Tâches

- [ ] Étape 0 — vérifier au code commité de la 4a les routes, codes et formes de réponse que l'écran consomme ; tout
      écart au Change Log.
- [ ] Bancs rouges d'abord (commit séparé) : liste (neuf), fiche, formulaire, helpers, messages, clôture, lots.
- [ ] Liste (AC-21) ; fiche (AC-22) ; édition (AC-23) ; formulaire (AC-24) ; messages (AC-25) ; clôture (AC-26) ;
      lots (AC-27) ; i18n (AC-28).
- [ ] Manuel utilisateur et PDF (AC-29).
- [ ] E2E neuf et specs adaptées (AC-30) ; captures (AC-31) ; CHANGELOG et README (AC-32).
- [ ] Mutations M21 à M34 (M21-bis compris).
- [ ] Gates frontend, gate complet et **E2E complet** au dernier commit de code de la 4b — il couvre aussi la 4a
      (dérogation, C-v017-4-31).

## Dev Agent Record

*(À remplir au développement.)*

## Change Log

- 2026-10-31 — Sous-fiche créée au découpage de la v017-4 (C-v017-4-1). Prochaine étape : validation P1 de la fiche
  parente et des deux sous-fiches (`v017-4-validate-prompt-p1.md`).
- 2026-10-31 — Décisions sur Q1 à Q5 (C-v017-4-21 à 25) : libellé « Validée » et inventaire des « ouverte » (AC-21,
  AC-29), M31 ; masquage des annulées confirmé (AC-21) ; pas de « Enregistrer et valider ».
- 2026-10-31 — Validation P1 remédiée dans la fiche parente (C-v017-4-26 à 33) : inventaire des « ouverte » par la valeur
  sur tout le dépôt, phrases du manuel réécrites (« saisissez, puis validez »), specs et helper E2E nommés, montant attendu
  et prix vide (#830), données du QR lues dans `sourceDocument`, M32 à M34. Prochaine : validation P2.
- 2026-10-31 — Validation P2 remédiée dans la fiche parente (C-v017-4-34 à 36) : prix vide refusé par le formulaire,
  specs de complétion et `fillFirstInvoiceLine` (date), règle du QR verrouillé et L7 au manuel, « deux voies » et clés
  `read-write`, lien d'écriture d'achat sur la fiche, union `SupplierInvoiceCancelCode`. Prochaine : validation P3.
