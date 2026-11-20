# Story 40-7d : écrans, manuel et E2E du compte Stripe (phase 1)

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/40-stripe`, chemins relevés sur `origin/main` au commit `fd04099f`. Epic 40,
**phase 1 KISS** (décision de Beat du 2026-11-20). **Dépend de** : 40-7b, 40-7c ; les écrans PayPal de la phase 1 de
l'Epic 38 (38-11d), dont elle reprend les composants. Dernière story de la phase 1.*

## Story

En tant que **Comptable**, je veux régler mon compte Stripe, importer son rapport d'un bouton, voir ce qui a été
rattaché et écrit, et parcourir la liste des écarts avec un lien vers chaque transaction ou écriture, et je veux que le
manuel me dise comment exporter le bon rapport de Stripe et ce que Kesh en fait.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-40-7d-1` | Le formulaire d'un compte bancaire offre les réglages Stripe (compte Stripe, compte de frais, compte de versement), à côté de ceux de PayPal, par les mêmes composants. | AC-1 |
| `FR-40-7d-2` | Un bouton *Importer un rapport Stripe* envoie le fichier, puis le rapprochement, et montre le compte rendu. | AC-2 |
| `FR-40-7d-3` | Une liste des écarts, par compte Stripe et période, avec lien vers la transaction ou l'écriture, et le geste existant de rattachement pour une réception proposée. | AC-3 |
| `FR-40-7d-4` | Manuel utilisateur : exporter le rapport (colonne `source_id`), régler, importer, rapprocher, lire les écarts, limites. | AC-4 |
| `NFR-40-7d-1` | E2E du parcours sur la fixture fictive ; captures de chaque état touché (recette visuelle). | AC-5 |

## Critères d'acceptation

1. **AC-1 — Réglages.** Page *Comptes bancaires* (`frontend/src/routes/(app)/bank-accounts/+page.svelte`, client
   `frontend/src/lib/features/bank-accounts/bank-accounts.api.ts`) : choix du prestataire (aucun, PayPal, Stripe) dans
   le bloc que 38-11d a posé ; pour Stripe, libellé du compte, compte de frais (sélecteur de comptes de charge, filtré CHF
   ou devise du compte lié), compte bancaire de versement (sélecteur des comptes bancaires de même devise, comptes Stripe
   exclus). Les refus de 40-7b AC-2 s'affichent sur leur champ (codes traduits aux quatre locales). Aucun composant
   recopié de PayPal : le bloc commun est paramétré (DRY ; s'il est propre à PayPal, il est généralisé ici, tests
   Vitest de 38-11d inchangés en assertions).
2. **AC-2 — Import.** Page *Import bancaire* (`frontend/src/routes/(app)/bank-import/+page.svelte`) : bouton *Importer un
   rapport Stripe* (visible si un compte Stripe existe), choix du compte Stripe, fichier ; envoi à `POST
   /api/v1/stripe-imports` avec `match=true`. Compte rendu : par devise, insérées et doublons ; versements lus ; puis
   ventes rattachées, frais et versements écrits, réceptions proposées, nombre d'écarts avec lien vers la liste. Refus
   `STRIPE_CSV_INVALID` sans `source_id` : message qui dit où ajouter la colonne dans Stripe. Rappel affiché : « les
   ventes viennent des boutiques ; importez-les avant le rapport Stripe ».
3. **AC-3 — Écarts.** Vue *Écarts Stripe* (sous l'import bancaire, comme la page PayPal de 38-11d si elle existe ;
   sinon route `frontend/src/routes/(app)/bank-import/stripe/+page.svelte`) : compte Stripe, période, bouton *Relancer
   le rapprochement* (`POST /api/v1/stripe-matching`) ; tableau `GET /api/v1/stripe-matching` : date, genre d'écart
   (libellé traduit pour chaque code de 40-7c AC-7), montant et devise, identifiant, lien vers la transaction
   (`/bank-import/[id]`) ou l'écriture ; pour `RECEIPT_TO_CONFIRM`, lien vers le dialogue existant *Rattacher à une
   écriture existante* de la transaction proposée. État vide : « Aucun écart ».
4. **AC-4 — Manuel.** `docs/manual/fr/user-manual.tex`, section neuve « Compte Stripe » après les comptes prestataires de
   PayPal (38-11d), `.tex` seulement (PDF à la release) : exporter le rapport *Rapprochement des versements → Détaillé*
   avec `source_id`, en points décimaux, et le fuseau voulu (la date des lignes est celle du fuseau d'export) ; le
   schéma comptable (vente de la boutique au transit avec l'identifiant au libellé ; frais ; versement) ; ce que fait le
   rapprochement ; chaque écart et quoi faire ; la réception sur Wise ou à la banque à confirmer, jamais comptée deux
   fois ; limites (remboursements et retours de versement à passer à la main ; rattachement en devise seul, les autres
   gestes de la réconciliation restant en CHF ; aucun code TVA ; aucune conversion entre devises). Manuel
   d'administration : rien en phase 1 (aucune variable). Axe de revue : le manuel contre le code et le PDF régénéré
   localement (règle du dépôt), sans commit du PDF.
5. **AC-5 — E2E et captures.** `frontend/tests/e2e/stripe-import.spec.ts` (patron `bank-import.spec.ts`, aides
   `frontend/tests/e2e/helpers/bank-import-fixtures.ts`) : comptes Stripe CHF/EUR/USD et Wise fictifs créés par l'API,
   écritures de vente fictives par l'API, import de la fixture de 40-7a par l'écran, compte rendu attendu, liste des
   écarts (au moins `CHARGE_WITHOUT_SALE`, `SALE_WITHOUT_CHARGE`, `RECEIPT_TO_CONFIRM`), confirmation d'une réception par
   le dialogue, écart disparu. Captures (thème clair) de chaque état touché, hors dépôt (`kesh-gate-logs/`). Vitest des
   composants neufs ; `npm run check`, `lint-i18n-ownership` (préfixe des clés par fonctionnalité), compteur
   `sitesTotal` de `i18n-keys.test.ts` recompté.

## Tâches

- [ ] T1 — Bloc de réglages prestataire généralisé, champs Stripe (AC-1).
- [ ] T2 — Import et compte rendu (AC-2).
- [ ] T3 — Vue des écarts (AC-3).
- [ ] T4 — Manuel (AC-4).
- [ ] T5 — E2E, Vitest, captures (AC-5).

## Stratégie de tests

Vitest (formulaire, compte rendu, libellés d'écarts, état vide) ; E2E complet au dernier commit de code ; tests
`textes_coherents.rs` du manuel s'il y a lieu.

## Modules touchés (3)

`frontend` (bank-accounts, bank-import, E2E), `kesh-i18n` (s'il manque un message serveur), `docs/manual/fr`.

## Note de version

*Ajouté* — Écrans du compte Stripe : réglages, import du rapport, liste des écarts ; section « Compte Stripe » du
manuel utilisateur.

## Notes de développement

- Remplace l'ancienne 40-6 (`superseded`) pour la phase 1.

## Change Log

- 2026-11-20 — fiche rédigée (phase 1 KISS), non validée.
