# Story 43-7 : le plafond de la différence d'un règlement groupé, réglé par l'administrateur (#823)

Status: ready-for-dev

*Fiche rédigée le 2026-11-17 dans la nuit par l'agent de rédaction de l'Epic 43 (Opus 5.5), workflow `bmad-create-story`,
sur la décision de Guy du 2026-11-17 (Q4 : plafond configurable, 5.00 CHF par défaut). Code lu à `8b50fd195`. Release
**0.19.0**. **Petite et coupable** (C-43-18) : la colonne est posée par la migration de la 43-1a (C-43-24) et lue par le
geste ; sans cette story, le plafond reste à sa valeur par défaut, non modifiable. Part de la 43-1a à son commit T1, qui
livre la colonne, le champ de l'entité des réglages, ses listes de colonnes et le lecteur `group_settlement_max_difference`
(43-1a AC3 bis, M5 de sa P3) : la 43-7 n'ajoute pas ce champ elle-même.
Décision C-43-24 (fichier d'épique), qui fait foi. Validation : à lancer.*

**Issues** : `refs #823`.

## Story

En tant qu'administrateur, je veux régler l'écart que Kesh admet entre un mouvement du relevé et les factures qu'il règle
(frais bancaires), pour l'adapter aux frais de ma banque.

## Ce qui se lit au code (`8b50fd195`)

- Table `company_invoice_settings` (créée par `20260417000001_invoice_validation.sql:35`) : porte
  `default_rounding_account_id` et `default_bank_fees_account_id` (lus par `company_invoice_settings.rs:628-670`). La 43-1a
  y ajoute `group_settlement_max_difference DECIMAL(19,4) NOT NULL DEFAULT 5.0000` (`CHECK` 0 à 100).
- Route : `GET` et `PUT /api/v1/company/invoice-settings` (`crates/kesh-api/src/lib.rs:289-290`, :1148-1149 ;
  `routes/company_invoice_settings.rs:295`, :325 ; `PUT` Admin seul, module :5-9) ; écran *Administration → Facturation*
  (`frontend/src/routes/(app)/settings/invoicing/+page.svelte`, compte d'arrondi :64, :211, :274) ;
  `docs/api-external.md:976` (la route y est nommée comme route d'administration, fermée aux clés).

## Critères d'acceptation

- **AC1** — `GET` rend `groupSettlementMaxDifference` (chaîne, deux décimales) ; `PUT` l'accepte, facultatif (absent →
  inchangé, comme les champs voisins qu'un client ancien n'envoie pas — vérifier le patron du module, :119).
- **AC2** — Contrôles du `PUT` : ≥ 0, au **centime** (deux décimales au plus), ≤ **100.00** ; montant ambigu (« 1.250 »)
  refusé par la règle des montants des réglages si elle est déjà posée, sinon la saisie de l'écran l'impose ; refus `400
  VALIDATION_ERROR` qui nomme le champ.
- **AC3** — Audit : l'enregistrement trace l'ancienne et la nouvelle valeur avec l'acteur utilisateur, dans l'entrée
  d'audit que la route écrit déjà pour les réglages (ou, s'il n'y en a pas, une action `company.invoice_settings_updated`
  enregistrée dans `audit_labels.rs`) — à établir à l'étape 0, ligne lue citée.
- **AC4** — Écran : un champ « Écart admis pour un règlement de plusieurs factures (frais bancaires) », à côté du compte
  de frais bancaires, avec son aide (« 0 : aucun écart admis ; au plus 100.00 ») ; i18n quatre locales.
- **AC5** — Effet : le geste groupé (43-1a, 43-1b) lit la valeur au moment du geste — un test règle 1.00 puis refuse un
  frais de 1.01 et accepte 1.00 ; la recherche de la 43-2 rend `maxDifference` = ce réglage.
- **AC6** — Manuel d'administration (`.tex`) : le réglage, sa valeur par défaut, son effet ; `docs/api-external.md`
  (champ de `PUT /company/invoice-settings`, route d'administration).

## Tests

Route : valeurs limites (0, 0.01, 100.00, 100.01, −0.01, 1.005) ; Admin seul (Comptable `403`, clé d'API refusée) ; audit
avec l'acteur. Vitest de l'écran (saisie, refus affiché). Mutation : retirer la borne haute (le test 100.01 rougit).

## Tâches

- [ ] **T0** — Étape 0 : patron du module et de l'audit des réglages. — [ ] **T1** — AC1 à AC3.
- [ ] **T2** — AC4, i18n. — [ ] **T3** — AC5, AC6. — [ ] **T4** — Gates, revue.

## Note de version

- **Ajouté** — *Administration → Facturation* : l'écart admis pour un règlement de plusieurs factures (frais bancaires),
  5.00 par défaut (refs #823).

## Dev Notes

Modules : `routes/company_invoice_settings`, `repositories/company_invoice_settings`, l'écran des réglages de facturation,
i18n, manuels. Quatre.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-17 (nuit) — Fiche rédigée (décision de Guy, Q4).
