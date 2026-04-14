# Story 3.6: Journaux personnalisables

Status: backlog

## Contexte

Créée en backlog le 2026-04-14 par Guy Corbaz lors du démarrage de la Story 5.2. Actuellement, le champ `journal_entries.journal` est contraint par un CHECK SQL BINARY figé à 5 codes (`Achats`, `Ventes`, `Banque`, `Caisse`, `OD`). Guy considère les journaux personnalisables comme **indispensables** avant la v1.0 (p. ex. un journal « Salaires » distinct de `OD`, ou des journaux analytiques).

## Story

As a **utilisateur (comptable)**,
I want **pouvoir créer, renommer, et désactiver des journaux personnalisés au-delà des 5 codes figés**,
so that **mon plan de journaux corresponde à mon organisation comptable réelle (p. ex. journal Salaires, journal Caisse bis, journal Analytique)**.

## Scope envisagé (refactor structurel — à affiner via `bmad-create-story`)

- **Nouvelle table `journals`** : `id`, `company_id`, `code VARCHAR(10)`, `label VARCHAR(64)`, `active BOOLEAN`, `is_system BOOLEAN` (protège les 5 codes de base), `created_at`, `updated_at`. UNIQUE (`company_id`, `code`).
- **Migration** : `ALTER TABLE journal_entries DROP CONSTRAINT chk_journal_entries_journal` + remplacer la colonne `journal VARCHAR(10)` par `journal_id BIGINT` avec FK. Migration data : insérer les 5 codes système par company existante + migrer les `journal_entries.journal` en `journal_id`.
- **Impact cross-cutting** : `company_invoice_settings.default_sales_journal` devient `default_sales_journal_id BIGINT FK`. Story 5.2 à adapter ou à migrer.
- **Nouveaux endpoints** : `GET/POST/PUT/DELETE /api/v1/journals` (admin_routes). Validation : `is_system = true` bloque la suppression.
- **UI** : nouvelle page `/settings/journals` (Admin) avec CRUD.
- **i18n** : libellés journaux traduits côté UI mais codes stockés ASCII.

## Dépendances

- Story 3.2 (journal_entries) : done.
- Story 5.2 (company_invoice_settings) : à adapter simultanément.

## Acceptance Criteria

À rédiger via `bmad-create-story`.

## Priorité

**Indispensable avant v1.0** selon Guy. À planifier après Epic 5 (une fois la facturation stabilisée) ou plus tôt si d'autres stories exigent un journal custom.

## Change Log

| Date       | Version | Description                                                   | Auteur          |
| ---------- | ------- | ------------------------------------------------------------- | --------------- |
| 2026-04-14 | 0.1     | Stub créé en backlog pendant démarrage Story 5.2 (scope v0.6) | Claude Opus 4.6 |
