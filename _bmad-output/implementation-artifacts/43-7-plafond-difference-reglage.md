# Story 43-7 : le plafond de l'écart d'un règlement groupé, réglé par l'administrateur (#823)

Status: ready-for-dev

*Fiche rédigée le 2026-11-17 dans la nuit par l'agent de rédaction de l'Epic 43 (Opus 5.5), workflow `bmad-create-story`,
sur la décision de Guy du 2026-11-17 (Q4 : plafond configurable, 5.00 CHF par défaut) ; **alignée le 2026-11-18 sur le
contrat figé de la 43-1a** (`kesh-gate-logs/e43/t3.md`, commit `241839568` ; code dans `/home/gcorbaz/devel/kesh-43-1a`)
et sur la passe ciblée « fiches contre `t3.md` » (HIGH-2, MEDIUM-4, LOW-9). Release **0.19.0**. **Petite et coupable**
(C-43-18) : la colonne est posée par la migration de la 43-1a (C-43-24) et lue par le geste ; sans cette story, le plafond
reste à sa valeur par défaut, non modifiable. Décisions C-43-24 et C-43-25 (fichier d'épique), qui font foi. Validation :
à lancer.*

**Issues** : `refs #823`.

## Story

En tant qu'administrateur, je veux régler l'**écart** que Kesh admet dans un règlement de plusieurs factures — moins-payé
ou trop-perçu, comptabilisé sur le compte que l'utilisateur choisit —, pour l'adapter à ce que je rencontre (frais
retenus, arrondis de clients, etc.).

## Ce qui se lit au code

- Livré par la 43-1a (branche `story/43-1a-reglement-groupe-modele`) : la colonne
  `company_invoice_settings.group_settlement_max_difference DECIMAL(19,4) NOT NULL DEFAULT 5.0000`, `CHECK` 0 à 100
  (`migrations/20261118000001_grouped_settlements.sql`, commentaire « Plafond de l'écart admis dans un règlement groupé ») ;
  le champ d'entité (`entities/company_invoice_settings.rs:74`), la liste `COLUMNS` (`repositories/company_invoice_settings.rs:49`),
  le lecteur `group_settlement_max_difference` (:67) et la constante `DEFAULT_GROUP_DIFFERENCE_MAX` = 5.0000 (:62-63).
- **Pas encore** livré — le **chemin d'écriture** : `CompanyInvoiceSettingsUpdate`, la requête `UPDATE … SET …` (:448-457)
  et **`is_no_op_change`** (:334-354, court-circuit « aucun changement » de la KF-004, pour ne pas incrémenter la version
  inutilement) ignorent la colonne ; `routes/company_invoice_settings.rs` aussi (`git grep group_settlement_max
  crates/kesh-api/src/routes` : aucun résultat).
- Route : `GET` et `PUT /api/v1/company/invoice-settings` (`crates/kesh-api/src/lib.rs:289-290`, :1148-1149 ;
  `routes/company_invoice_settings.rs:295`, :325 ; `PUT` Admin seul, module :5-9 ; audit en enveloppe `{before, after}`,
  doc de `update`) ; écran *Administration → Facturation* (`frontend/src/routes/(app)/settings/invoicing/+page.svelte`,
  compte d'arrondi :64, :211, :274) ; `docs/api-external.md:976` (route d'administration, fermée aux clés).

## Critères d'acceptation

- **AC1 — Lecture et écriture, les trois sites du dépôt.** `GET` rend `groupSettlementMaxDifference` en chaîne à **deux
  décimales** (la colonne en porte quatre : une valeur à quatre décimales, posée par SQL direct, s'affiche arrondie au
  centime ; le `PUT` n'écrit qu'au centime). `PUT` l'accepte, facultatif (absent → inchangé, comme les champs voisins qu'un
  client ancien n'envoie pas, patron du module :119). Le champ entre dans **les trois sites** :
  1. `CompanyInvoiceSettingsUpdate` ;
  2. la requête `UPDATE` (:448-457) ;
  3. **`is_no_op_change`** (:334-354) — sans lui, un `PUT` qui ne change **que** le plafond est jugé « sans changement » :
     `200`, rien d'écrit, version inchangée.

  Et dans la sérialisation `{before, after}` de l'audit.
- **AC2** — Contrôles du `PUT` : ≥ 0, au **centime** (deux décimales au plus), ≤ **100.00** ; montant ambigu (« 1.250 »)
  refusé par la règle des montants des réglages si elle est déjà posée, sinon la saisie de l'écran l'impose ; refus `400
  VALIDATION_ERROR` qui nomme le champ.
- **AC3** — Audit : l'enveloppe `{before, after}` que la route écrit déjà pour les réglages porte l'ancienne et la nouvelle
  valeur, avec l'acteur utilisateur ; aucune action neuve.
- **AC4** — Écran : un champ « **Écart admis pour un règlement de plusieurs factures** (moins-payé ou trop-perçu) », avec son
  aide (« 0 : aucun écart admis, seul l'arrondi sous le centime passe ; au plus 100.00 »), saisi et affiché à **deux
  décimales** ; i18n quatre locales. « Frais bancaires » n'y figure pas : ce mot ne désigne que le compte prérempli de la
  43-3.
- **AC5** — Effet : le geste groupé (43-1a, 43-1b) lit la valeur au moment du geste — un test règle 1.00 puis, **avec un
  compte choisi** (`differenceAccountId`), refuse un écart `e = −1.01` **et** `e = +1.01` (`DifferenceTooLarge`) et
  accepte `e = −1.00` et `e = +1.00` ; le plafond se jugeant d'abord (`group_difference`), le refus est
  `DifferenceTooLarge` même sans compte ; la recherche de la 43-2 rend `maxDifference` = ce réglage, à deux décimales.
- **AC6** — Manuel d'administration (`.tex`) : le réglage, sa valeur par défaut, son effet (écart dans les deux sens,
  compte choisi, l'arrondi sous le centime n'y est pas soumis) ; `docs/api-external.md` (champ de
  `PUT /company/invoice-settings`, route d'administration).

## Tests

Route : valeurs limites (0, 0.01, 100.00, 100.01, −0.01, 1.005) ; **un `PUT` qui ne change que le plafond l'écrit** (valeur
relue, version incrémentée, audit `{before, after}`) ; Admin seul (Comptable `403`, clé d'API refusée) ; audit avec
l'acteur. Vitest de l'écran (saisie, deux décimales, refus affiché). Effet : AC5 dans les deux sens. Mutations : retirer la
borne haute (le test 100.01 rougit) ; **oublier le champ dans `is_no_op_change`** (le test « ne change que le plafond »
rougit).

## Tâches

- [ ] **T0** — Étape 0 : patron du module ; les trois sites d'AC1 relus à la tête de la branche réunie.
- [ ] **T1** — AC1 à AC3 (dépôt : trois sites ; route ; audit).
- [ ] **T2** — AC4, i18n. — [ ] **T3** — AC5, AC6. — [ ] **T4** — Gates, revue.

## Note de version

- **Ajouté** — *Administration → Facturation* : l'écart admis pour un règlement de plusieurs factures (moins-payé ou
  trop-perçu, sur le compte choisi), 5.00 par défaut (refs #823).

## Dev Notes

Modules : `routes/company_invoice_settings`, `repositories/company_invoice_settings` (`CompanyInvoiceSettingsUpdate`,
`UPDATE`, `is_no_op_change`), l'écran des réglages de facturation, i18n, manuels. Quatre. `kesh-db` touché : gate complet en
boucle.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-17 (nuit) — Fiche rédigée (décision de Guy, Q4).
- 2026-11-18 — Alignée sur le contrat `t3.md` : HIGH-2 (les trois sites d'écriture, dont `is_no_op_change` ; test « ne
  change que le plafond » ; mutation), MEDIUM-4 (vocabulaire : un écart dans les deux sens au compte choisi, plus de
  « frais bancaires » dans la Story, le libellé et la note de version ; AC5 dans les deux sens, avec compte), LOW-9
  (échelle : deux décimales à l'écran et à l'API, quatre en colonne ; audit `{before, after}` établi).
