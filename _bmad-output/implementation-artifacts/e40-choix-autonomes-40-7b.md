# Choix autonomes — Story 40-7b (Epic 40, phase 1)

Registre par union (docs/agents-orchestration.md § 1) ; numérotation `C-40-7b-<k>`.

## C-40-7b-1 — Corps JSON : un objet `stripe` à côté de `paypal`

- **Contexte** : la fiche (AC-2) vise l'objet générique de la 39-11b ; cette branche ne le porte pas (seule la phase 1 de
  l'Epic 38 est fusionnée : objet `paypal`, `PaypalSettings`).
- **Choix** : un objet `stripe: { account, feeAccountId, payoutBankAccountId }` servi par les **mêmes fonctions** que
  `paypal` (type interne commun `ProviderSettings`, une garde 1 à 5, un `INSERT`/`UPDATE`). `paypal` et `stripe` ensemble :
  `400 VALIDATION_ERROR` ; `payoutBankAccountId` dans `paypal` : `400 VALIDATION_ERROR` ; `paypal: null` ne retire pas un
  compte Stripe (et inversement).
- **Alternatives écartées** : renommer `paypal` en `provider` (rupture de contrat, relève de la 39-11b).
- **Réversibilité** : à la fusion avec la 39-11b, `stripe` et `paypal` se replient sur son objet générique ; le type interne
  `ProviderSettings` est déjà celui des gardes.

## C-40-7b-2 — Cœur d'import commun : la forme de la 38-12a

- **Contexte** : AC-7, T0 : ni 38-11b ni 39-11b n'avaient factorisé le cœur dans cette branche. Consigne du coordinateur
  (2026-11-21) : s'aligner sur celui que la 38-12a extrait (`kesh-wt38p2`).
- **Choix** : `routes/provider_imports.rs` et `routes/paypal_imports.rs` recopiés de la 38-12a (noms `ProviderLine`,
  `ImportRequest`, `ImportSource`, `load_balances`, `import_lines`, `dedupe_lines`) ; ajouts minimaux listés au Dev Agent Record
  (`value_date`, aides de formulaire et de bornes). La route Stripe n'a que lecteur, ventilation et lignes neutres.
- **Alternatives écartées** : ma première forme (`run` / `ProviderImport` / `ProviderEntry`), abandonnée.
- **Réversibilité** : à l'intégration, fusionner les trois écarts du Dev Agent Record.

## C-40-7b-3 — Prédicat « compte prestataire » : déjà unique

- Les six refus `BANK_ACCOUNT_IS_PROVIDER` jugent `provider_settings.is_provider()` (`provider IS NOT NULL`) ; aucun
  changement n'était nécessaire pour Stripe (couvert par `les_autres_chemins_refusent_un_compte_stripe`).

## C-40-7b-4 — CHECK de la migration

- `chk_bank_accounts_payout_stripe` s'écrit `payout IS NULL OR (provider IS NOT NULL AND BINARY provider = 'stripe')` :
  la forme de la fiche (`… OR BINARY provider = 'stripe'`) laisserait passer, par une CHECK qui vaut NULL, une destination
  sur un compte sans prestataire. Prouvé par `stripe_import_schema.rs`.

## C-40-7b-5 — Compte de versement et garde 5

- Changer de **prestataire** sur un compte qui porte des transactions est refusé (`PROVIDER_ACCOUNT_IN_USE`) ; changer le
  compte de versement ou de frais reste admis.
- Un compte qui se désigne lui-même comme destination est refusé avec `reason = stripe`.
