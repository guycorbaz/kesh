# Story 2.6: Onboarding — comptes facturation pré-remplis

Status: backlog

## Contexte

Créée en backlog le 2026-04-14 par Guy Corbaz lors du démarrage de la Story 5.2. Les comptes par défaut de facturation (`default_receivable_account_id`, `default_revenue_account_id`) sont obligatoires pour valider une facture (Story 5.2), sinon 400 `CONFIGURATION_REQUIRED`. UX sous-optimale : l'utilisateur découvre la contrainte à la 1re validation.

## Story

As a **nouvel utilisateur Kesh**,
I want **que les comptes par défaut de facturation soient pré-sélectionnés à la fin de l'onboarding (avec comptes standard Suisse : 1100 Clients, 3000 Ventes)**,
so that **je puisse valider ma première facture sans détour par une page de config**.

## Scope envisagé (à affiner via `bmad-create-story`)

- Pendant le flow onboarding (Story 2.2/2.3), après chargement du plan comptable (3.1), poser automatiquement `company_invoice_settings.default_receivable_account_id = account(1100)` et `default_revenue_account_id = account(3000)` si existants.
- Si le plan comptable chargé n'a pas ces codes (plans alternatifs), afficher un écran de sélection optionnel à l'étape finale de l'onboarding.
- Skip possible — l'utilisateur peut configurer plus tard via `/settings/invoicing`.

## Dépendances

- Story 5.2 (création de la table `company_invoice_settings`) : **prérequis DOIT être done**.
- Story 3.1 (plan comptable) : done.

## Acceptance Criteria

À rédiger via `bmad-create-story`.

## Change Log

| Date       | Version | Description                                                   | Auteur          |
| ---------- | ------- | ------------------------------------------------------------- | --------------- |
| 2026-04-14 | 0.1     | Stub créé en backlog pendant démarrage Story 5.2 (scope v0.6) | Claude Opus 4.6 |
