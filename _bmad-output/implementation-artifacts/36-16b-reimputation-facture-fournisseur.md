# Story 36-16b : changer le compte analytique d'une facture fournisseur, même payée (#1167)

Status: backlog

*Fiche rédigée le 2026-11-15 par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-16`, par le workflow
`bmad-create-story`. **Deuxième des trois stories nées du découpage de la 36-16** (`36-16-reimputation-analytique.md`, `Status:
split` ; C-36-16-10). **Part de la 36-16a fusionnée** dans `epic/36-0.18.0`, dont elle réemploie le dépôt
(`project_reassignment.rs`), le dialogue partagé, l'accusé de période close et les décisions communes D0 à D12 (C-36-16-1 à -9) :
la fiche **y renvoie** sans les répéter. Phase 2, release **0.18.0**. **Aucune migration.** **`kesh-db` touché : gate complet
même en cours de boucle.** Voie 1 (bases `kesh_e3616b`, `kesh_e2e_e3616b`, `kesh_ci_e3616b`). Code lu à **`fed34a9c7`** — les
lignes se relisent à l'étape 0 sur la base de départ (36-16a fusionnée). `Status: backlog` tant que 36-16a n'est pas fusionnée ;
l'orchestrateur la passe à `ready-for-dev`. Validation non lancée.*

**Issues** : `refs #1167`, aucun mot-clé de fermeture dans un commit. **Vocabulaire** : celui de la 36-15 fusionnée (« compte
analytique » ; de-CH, it-CH, en-CH en attente de l'arbitrage de Guy).

## Story

En tant que titulaire qui suit ses dépenses par compte analytique, je veux **changer le compte analytique d'une facture
fournisseur quel que soit son statut** — payée, annulée, d'un exercice clos compris —, d'un seul geste, afin que la facture, son
achat, son paiement et la contre-passation de l'achat passent au nouveau compte, sans rien défaire.

## Ce qui se lit au code (`fed34a9c7`)

- **Colonne** : `supplier_invoices.project_id` (`migrations/20260702000002_supplier_invoices_project.sql`), lue par
  `FIND_SCOPED_SQL` (`supplier_invoices.rs:42-45`, avec `purchase_journal_entry_id`, `settlement_journal_entry_id`).
- **Achat** — `supplier_invoices.rs:1777` (`project_id,`, « Propagation document-level »), projet lu `:1585`, contrôlé `:1611-1612`
  (`validate_taggable_in_tx`) ; lié par `purchase_journal_entry_id`.
- **Paiement** — `supplier_invoices.rs:2163` (`project_id: inv.project_id,`, « Le règlement hérite du projet de la facture »),
  chemin commun du paiement direct, du lot pain.001 et du rapprochement (`pay_in_tx_as`, `routes/reconciliation.rs:3426`) ; lié par
  `settlement_journal_entry_id`. **Lecture sous verrou non établie** (étape 0, D9 de 36-16a).
- **Annulation** — `supplier_invoices.rs:2636-2640` : `status = 'cancelled'`, vide `settlement_*`, **garde**
  `purchase_journal_entry_id` ; la contre-passation de l'achat se retrouve par `reverses_entry_id`. Une facture annulée **alors que
  payée** garde un paiement « détaché », retrouvé par la seule trace d'audit (`journal_entries.rs:1074-1093`,
  `DETACHED_SETTLEMENT_TRACES`).
- **Ce qui délie** : l'annulation du paiement vide `settlement_journal_entry_id` (`supplier_invoices.rs:2929`) ; la dévalidation
  supprime l'achat (`supplier_invoices.rs:1399`, « purchase_journal_entry_id = NULL »).
- **Brouillon** : `update_draft_in_tx` (`supplier_invoices.rs:758`, « project_id = ? … AND status = 'draft' »).
- **Écran** : `routes/(app)/supplier-invoices/[id]/+page.svelte:812-814` (`data-testid="supplier-invoice-project"`), liste des
  projets `:241`.
- **Route voisine** : `POST /api/v1/supplier-invoices/{id}/settlement/cancel` (`lib.rs:682-683`), `comptable_routes` (`lib.rs:536`) ;
  registre `audit_route_registry.rs:498` (`supplier_invoices::unvalidate_supplier_invoice`, `Traced`, `Rejouee`).

## Décisions

*D0 à D12 : celles de **36-16a** (C-36-16-1 à -9), appliquées à la facture fournisseur.* Propres à cette story :

- **Db1 — Route** : `POST /api/v1/supplier-invoices/{id}/reassign-project`, même corps et même réponse que la facture client
  (`{ invoice, journalEntryIds }`, la facture mise en forme comme par `GET /supplier-invoices/{id}`). Contrat : ajout seulement.
- **Db2 — L'ensemble des écritures** (D2) : achat (`purchase_journal_entry_id`), paiement courant (`settlement_journal_entry_id`),
  et la contre-passation de chacun (`reverses_entry_id IN (…)`, ce qui prend celle de l'achat d'une annulée). **Le paiement détaché**
  d'une annulée payée n'est **pas** pris (palliatif d'audit de #541, lecture fragile ; comptes de bilan seuls, invisible des
  rapports analytiques) : réimputable par le geste de 36-16c. *Écartée* : le lire par `DETACHED_SETTLEMENT_TRACES` — une trace
  étrangère restaurée peut désigner un paiement sans rapport (`journal_entries.rs:1086-1092`, « faux positif possible après restauration »).
- **Db3 — Étape 0 (D9)** : établir que `pay_in_tx_as` lit `project_id` **sous** le verrou de la facture (`supplier_invoices.rs:2163`
  et la lecture qui le précède, ligne lue citée au Change Log). S'il ne l'est pas, la lecture passe sous verrou dans la story
  (limite voisine) ou une issue est demandée à l'orchestrateur.
- **Db4 — Audit** : `supplier_invoice.project_reassigned` (35 caractères, `audit_log.action VARCHAR(64)`), mêmes `details` que D6.

## Critères d'acceptation

1. **AC1** — Facture fournisseur **payée** au compte P : le geste vers Q rend `200` ; la facture et chaque ligne de l'achat et du
   paiement portent Q ; `journalEntryIds` les liste.
2. **AC2** — Facture **annulée** : l'achat et sa contre-passation portent Q ; le paiement détaché d'une annulée payée garde P.
3. **AC3** — Rien d'autre ne change (instantané hors `project_id`, `version`, `updated_at` : montants, comptes, dates, numéros dont
   le numéro interne, lettrage, rapprochement, statut, lot de paiement).
4. **AC4** — Période close : `409 PROJECT_REASSIGNMENT_CLOSED_PERIOD` sans accusé, `200` avec.
5. **AC5** — Refus : compte archivé `409 ILLEGAL_STATE_TRANSITION`, inconnu `404`, autre société `404`, version `409
   OPTIMISTIC_LOCK_CONFLICT`, Lecture ou clé `read` `403` ; rien d'écrit.
6. **AC6** — Audit `supplier_invoice.project_reassigned`, acteur compris (clé d'API), libellé ×4 ; même compte → rien d'écrit ;
   `null` retire le compte.
7. **AC7** — Écran de la facture fournisseur : le dialogue partagé de 36-16a, avertissement, message ; Lecture et brouillon sans
   bouton.
8. **AC8** — Route au registre `audit_route_registry.rs` ; `audit_acteur_garde.rs` vert sans entrée neuve.
9. **AC9** — Manuel (sous-section de 36-16a étendue à la facture fournisseur et au paiement détaché) et `docs/api-external.md`
   (route, ordre des refus) ; textes ×4.

## Tests à écrire, et la preuve du rouge

Premier rouge : route `404` avant le correctif (`kesh-gate-logs/e36-16b/rouges.log`) ; puis mutations (E5, `kesh-gate-logs/e36-16b/mut/`).

| Test | Lieu | Mutation |
|---|---|---|
| T1 payée réimputée (achat, paiement) | `crates/kesh-db/tests/project_reassignment.rs` | retirer `settlement_journal_entry_id` de l'ensemble |
| T2 annulée : achat et contre-passation ; détaché intact | idem | retirer la jointure `reverses_entry_id` |
| T3 instantané hors `project_id` identique | idem | `UPDATE` qui touche `account_id` (sonde) |
| T4 invariant : achat et paiement créés portent le projet de la facture | idem | `project_id: None` à `supplier_invoices.rs:2163` |
| T5 accusé, refus, acteur | `crates/kesh-api/tests/project_reassignment_e2e.rs` | `Acteur::utilisateur` au lieu de `Acteur::from` ; court-circuit du contrôle de période |
| T6 écran | `routes/(app)/supplier-invoices/[id]/supplier-invoice-display-page.test.ts` | masquer le bouton ; ne pas rafraîchir le libellé |

**E2E exigé** — cas ajouté à `frontend/tests/e2e/project-reassignment.spec.ts` (de 36-16a) : facture fournisseur **payée** préparée
par l'API, geste au navigateur, `GET` de l'achat et du paiement : lignes au nouveau compte. Rouge d'abord (`e2e-rouge.log`).

## Tâches

- [ ] **Étape 0** — Db3 ; relire les lignes citées sur la base de départ ; LOW de la validation.
- [ ] **kesh-db** — `supplier_invoice_entry_ids_in_tx`, `reassign_supplier_invoice` dans `project_reassignment.rs`.
- [ ] **kesh-api** — `supplier_invoices::reassign_project_handler`, route, rejeu, registre, `audit_labels.rs`.
- [ ] **kesh-i18n** — libellé d'audit et textes ×4, près des blocs `supplier-invoices-detail-*`.
- [ ] **frontend** — bouton et dialogue partagé sur `supplier-invoices/[id]/+page.svelte`, appel d'API dans
  `features/supplier-invoices`.
- [ ] Tests, mutations, E2E ; captures (`kesh-gate-logs/e36-16b/captures/`) ; manuel et `api-external.md`.
- [ ] **Gates** : ceux de 36-16a (voie 1, `kesh-db` → `test-fast --ci` en boucle ; `gate-ci kesh_ci_e3616b` au passage en review ;
  `gate-ci` et E2E complet à la clôture).

## Note de version

*Ajouté* — **Changer le compte analytique d'une facture fournisseur, même payée** : le même geste que pour la facture client
réimpute la facture, son achat, son paiement et, pour une facture annulée, la contre-passation de l'achat, avec le même
avertissement en période close. Pour les intégrations : `POST /api/v1/supplier-invoices/{id}/reassign-project` (refs #1167).

## Dev Notes

### Modules touchés

`kesh-db` ; `kesh-api/routes/supplier_invoices` ; `kesh-i18n` ; `routes/(app)/supplier-invoices/[id]` (avec l'appel de
`features/supplier-invoices`). **Quatre modules de code**, plus la documentation.

### Hors périmètre

Facture client (36-16a), écriture manuelle et paiement détaché (36-16c) ; le MCP ; le vocabulaire (36-15).

### Fichiers communs

`project_reassignment.rs`, `project-reassignment.spec.ts`, manuel, `api-external.md`, `messages.ftl` : partagés avec 36-16a
(fusionnée avant) et 36-16c (en parallèle possible : blocs distincts, union à l'intégration). `supplier_invoices.rs` : la 36-11c
(0.19.0) y passera ensuite. `i18n-keys.test.ts` recompté. `CHANGELOG.md` jamais écrit.

### Ce qui doit rester vrai

Celles de 36-16a ; en plus : le numéro interne, le lot de paiement et le document de la facture ne changent pas.

### Références

Fiches `36-16-reimputation-analytique.md` (origine) et `36-16a-reimputation-facture-client.md` (décisions communes) ; #1167, #541,
#868 ; stories 19-3, 25-3-a-2, v0172-annulees.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-15 — Création (Opus 5.5), par découpage de `36-16-reimputation-analytique.md` (C-36-16-10). Reprend son lot B et le point
  d'étape 0 sur `supplier_invoices.rs:2163`. Décisions communes : 36-16a. Code lu à `fed34a9c7`. Aucune migration.
