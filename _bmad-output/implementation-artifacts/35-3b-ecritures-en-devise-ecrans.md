# Story 35-3b : écritures en devise — soldes d'ouverture, formulaire et écrans

Status: ready-for-dev

*Fiche rédigée le 2026-11-14 (Opus 5.5), branche `epic/35-multi-devises`. Epic 35, C-35-4, C-35-5. Suit **35-3a**
(découpe de l'ancienne 35-3 à la rédaction). **Aucune migration.***

## Story

En tant que comptable en multi-devises, je veux saisir à l'écran une écriture en devise avec le cours officiel
proposé, saisir mes propres cours pour une conversion et y ajouter d'un clic la différence de change, et ouvrir mes
comptes en devise avec leur solde en devise, afin de tenir mes livres sans passer par l'API.

## Critères d'acceptation

1. **AC-1 — Soldes d'ouverture.** L'écran et l'API des soldes d'ouverture (`routes/opening_balances.rs`
   `generate_opening_balances` :345 → `journal_entries::create_opening_entry` :655) et le complément
   (`opening_complement.rs` `create_opening_complement` :460) acceptent, pour un compte en devise, le **solde en
   devise** et un **cours** (proposé : cours AFC du mois de la date d'ouverture, par `lookup` de 35-2 ; modifiable ;
   `rate_source` `afc` ou `manual`) ; le CHF est calculé par la règle de 35-3a (AC-3) ; un compte CHF reste inchangé.
2. **AC-2 — Formulaire d'écriture** (`JournalEntryForm.svelte`), en multi-devises seulement :
   - ligne sur un compte en devise : champs *Montant (USD)*, *Cours* (prérempli par `lookup` du mois de la date ;
     indicateur « AFC » ou « manuel » ; modifié à la main → `manual`), *CHF* calculé en lecture seule ; changer la date
     repropose les cours AFC des lignes dont le cours n'a pas été modifié à la main ;
   - cours indisponible (`RATE_UNAVAILABLE`) : message, saisie manuelle exigée ;
   - **ligne d'écart** : écriture déséquilibrée en CHF avec au moins une ligne en devise → bouton « Ajouter la
     différence de change » : une ligne CHF sur le compte de gains (écart au crédit) ou de pertes (au débit) désigné en
     35-1 ; au-delà de n × 0.005 CHF, le texte précise qu'il ne s'agit pas d'un arrondi ; rien n'est ajouté sans clic
     (C-35-4) ; comptes non désignés ou invalides : le bouton explique quoi régler ;
   - calcul côté client avec `big.js`, même arrondi que le serveur (`MidpointAwayFromZero`), cas `x.xx5` testés des
     deux côtés avec les mêmes valeurs ;
   - en mono-devise : formulaire strictement inchangé (test de non-régression).
3. **AC-3 — Affichage** : détail et liste des écritures montrent montant en devise et cours à côté du CHF ; le
   formulaire de modification les recharge.
4. **AC-4 — Sélecteurs** des factures, avoirs, règlements, factures fournisseur, lots, réconciliation, comptes
   bancaires liés, comptes par défaut : les comptes en devise n'y apparaissent pas (inventaire clos des composants au
   Change Log, à partir de l'inventaire serveur de 35-3a).
5. **AC-5 — E2E** : écriture USD au cours AFC (cours servis en mode test), conversion EUR → USD avec ligne d'écart,
   soldes d'ouverture d'un compte USD, mono-devise inchangé ; captures d'écran des états (thème clair).
6. **AC-6 — Manuel utilisateur** : saisir une écriture en devise, cours proposé et manuel, conversion EUR → USD pas à
   pas, soldes d'ouverture en devise ; i18n quatre locales ; PDF contrôlé.

## Tâches

- [ ] Soldes d'ouverture : serveur et écran (AC-1).
- [ ] Formulaire, `balance.ts`, détail, liste (AC-2, AC-3), Vitest.
- [ ] Sélecteurs (AC-4).
- [ ] E2E, captures, manuel, CHANGELOG (AC-5, AC-6).

## Notes de développement

- `frontend/src/lib/features/journal-entries/` : `JournalEntryForm.svelte` (575 lignes), `balance.ts` (`parseAmount`
  :68, `computeBalance` :83, `formatSwissAmount` :129), `journal-entries.types.ts`.
- Écrans des soldes d'ouverture : `frontend/src/routes/(app)/settings/opening-balances/`.

## Change Log

- 2026-11-14 — fiche rédigée (découpe de 35-3).
