# Story 36-16b : changer le compte analytique d'une facture fournisseur, même payée (#1167)

Status: backlog

*Fiche rédigée le 2026-11-15 par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-16`, par le workflow
`bmad-create-story`. **Deuxième des trois stories nées du découpage de la 36-16** (`36-16-reimputation-analytique.md`, `Status:
split` ; C-36-16-10). **Part de la 36-16a fusionnée** dans `epic/36-0.18.0`, dont elle réemploie le dépôt
(`project_reassignment.rs`), le dialogue partagé, l'accusé de période close et les décisions communes D0 à D13 (C-36-16-1 à -9, -11 à -13) :
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
  `settlement_journal_entry_id`. **Lu sous le verrou de la facture — établi** : `pay_in_tx_as` (`supplier_invoices.rs:1976`)
  commence par « (1) Verrou facture + garde statut » et `let inv = sqlx::query_as::<_, SupplierInvoice>(&format!("{FIND_SCOPED_SQL}
  FOR UPDATE"))` (`:1988-1989`) ; `:2163` lit `inv.project_id` de **cette** lecture. Le rapprochement n'emploie le projet de sa
  lecture ordinaire que pour les `details` d'un refus (`routes/reconciliation.rs:3441`). Conséquence : le geste, qui prend la même
  ligne `FOR UPDATE` (D9 de 36-16a), et le paiement se **sérialisent** ; un paiement commité avant le geste est relu par lui sous
  verrou et réimputé, un paiement qui suit lit le nouveau compte. **Aucun verrou optimiste n'est en jeu** ici (au contraire de
  l'encaissement client), et rien n'est modifié dans `pay_in_tx_as`.
- **Annulation** — `supplier_invoices.rs:2636-2640` : `status = 'cancelled'`, vide `settlement_*`, **garde**
  `purchase_journal_entry_id` ; la contre-passation de l'achat se retrouve par `reverses_entry_id`. Une facture annulée **alors que
  payée** garde un paiement « détaché », retrouvé par la seule trace d'audit (`journal_entries.rs:1074-1093`,
  `DETACHED_SETTLEMENT_TRACES`).
- **Ce qui délie** : l'annulation du paiement vide `settlement_journal_entry_id` (`supplier_invoices.rs:2929`) ; la dévalidation
  supprime l'achat (`supplier_invoices.rs:1399`, « purchase_journal_entry_id = NULL »).
- **Brouillon** : `update_draft_in_tx` (`supplier_invoices.rs:758`, « project_id = ? … AND status = 'draft' »).
- **Écran** : `frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte:812-814` (`data-testid="supplier-invoice-project"`),
  liste des projets `:241`.
- **Route voisine** : `POST /api/v1/supplier-invoices/{id}/settlement/cancel` (`lib.rs:682-683`), `comptable_routes` (`lib.rs:536`) ;
  registre `audit_route_registry.rs:498` (`supplier_invoices::unvalidate_supplier_invoice`, `Traced`, `Rejouee`).

## Décisions

*D0 à D13 : celles de **36-16a** (C-36-16-1 à -9, -11 à -13), appliquées à la facture fournisseur : accusé de période close
**réservé à l'Administrateur en session** (`403 PROJECT_REASSIGNMENT_CLOSED_PERIOD_ADMIN_ONLY` pour un Comptable ou une clé,
C-36-16-11) ; audit écrit par le dépôt avant le `COMMIT`, avec l'`Acteur` ; versions +1 (pièce et chaque écriture touchée) ;
lectures verrouillantes et ordre des refus de D9 ; rien-à-faire comparé aux lignes (C-36-16-12) ; dialogue partagé au contrat de
D12 (une rangée, `id` = la facture).* Propres à cette story :

- **Db1 — Route** : `POST /api/v1/supplier-invoices/{id}/reassign-project`, même corps et même réponse que la facture client
  (`{ invoice, journalEntryIds }`, la facture mise en forme comme par `GET /supplier-invoices/{id}`). Contrat : ajout seulement.
  Dépôt : `reassign_supplier_invoice(pool, acteur: Acteur, may_acknowledge: bool, company_id, id, project_id, version,
  acknowledge)` ; appel frontend `reassignSupplierInvoiceProject` dans `frontend/src/lib/features/projects/projects.api.ts`.
- **Db2 — L'ensemble des écritures** (D2) : achat (`purchase_journal_entry_id`), paiement courant (`settlement_journal_entry_id`),
  et la contre-passation de chacun — collecte générique de 36-16a avec **`include_reversals = vrai`** (`reverses_entry_id IN (…)`,
  ce qui prend celle de l'achat d'une annulée : cas réel, `supplier_invoices.rs:2636-2640` garde le lien d'achat). **Le paiement détaché**
  d'une annulée payée n'est **pas** pris (palliatif d'audit de #541, lecture fragile ; comptes de bilan seuls, invisible des
  rapports analytiques) : réimputable par le geste de 36-16c. *Écartée* : le lire par `DETACHED_SETTLEMENT_TRACES` — une trace
  étrangère restaurée peut désigner un paiement sans rapport (`journal_entries.rs:1086-1092`, « faux positif possible après restauration »).
- **Db3 — Paiement concurrent (D9, établi)** : sérialisation par le `FOR UPDATE` de la facture, des deux côtés (§ *Ce qui se lit
  au code*). L'étape 0 relit `supplier_invoices.rs:1988-1989` et `:2163` sur la base de départ ; `pay_in_tx_as` n'est pas modifié.
- **Db4 — Audit** : `supplier_invoice.project_reassigned` (35 caractères, `audit_log.action VARCHAR(64)`), mêmes `details` que D6.

## Critères d'acceptation

*États construits par les routes (validation, `POST …/pay`, `POST …/cancel`), jamais par un `INSERT` direct.*

1. **AC1** — Facture fournisseur **payée** au compte P : le geste vers Q rend `200` ; la facture et chaque ligne de l'achat et du
   paiement portent Q ; `journalEntryIds` les liste.
2. **AC2** — Facture **annulée** (ouverte puis annulée) : l'achat et sa contre-passation portent Q ; facture annulée **alors que
   payée** : achat et contre-passation à Q, le paiement détaché garde P.
3. **AC3** — Rien d'autre ne change (instantané hors `project_id`, `version`, `updated_at` : montants, comptes, dates, numéros dont
   le numéro interne, lettrage, rapprochement, statut, lot de paiement, document).
4. **AC4** — Versions : facture +1, chaque écriture touchée +1, hors ensemble inchangée.
5. **AC5** — **Paiement concurrent** : un paiement non commité tient la facture ; le geste lancé sur une autre connexion attend, puis
   réimpute le paiement commité ; un paiement fait après le geste porte Q.
6. **AC6** — Période close : Comptable ou clé → `403 PROJECT_REASSIGNMENT_CLOSED_PERIOD_ADMIN_ONLY` ; Administrateur en session →
   `409 PROJECT_REASSIGNMENT_CLOSED_PERIOD` sans accusé, `200` avec ; rien d'écrit aux refus.
7. **AC7** — Refus dans l'ordre de D9 (36-16a) : `404` → version `409 OPTIMISTIC_LOCK_CONFLICT` → rien à faire `200` → compte visé
   `404` / archivé `409 ILLEGAL_STATE_TRANSITION` → période ; Lecture ou clé `read` `403` d'abord.
8. **AC8** — Audit `supplier_invoice.project_reassigned`, écrit dans la transaction, acteur compris (clé d'API), libellé ×4 ;
   facture et lignes déjà à Q → rien d'écrit ; une ligne divergente est réalignée ; `null` retire le compte.
9. **AC9** — Écran de la facture fournisseur : le dialogue partagé de 36-16a (avertissement et *Confirmer* pour l'Administrateur,
   message du `403` pour le Comptable) ; Lecture et brouillon sans bouton.
10. **AC10** — Route au registre `audit_route_registry.rs` ; `audit_acteur_garde.rs` vert sans entrée neuve.
11. **AC11** — Manuel (sous-section de 36-16a étendue à la facture fournisseur et au paiement détaché) et `docs/api-external.md`
    (route, ordre des refus, paragraphe « Interblocages » `:1103`) ; textes ×4 selon D11.

## Tests à écrire, et la preuve du rouge

Premier rouge : route `404` avant le correctif (`kesh-gate-logs/e36-16b/rouges.log`) ; puis mutations (E5, `kesh-gate-logs/e36-16b/mut/`).

| Test | Lieu | Mutation |
|---|---|---|
| T1 payée réimputée (achat, paiement) | `crates/kesh-api/tests/project_reassignment_e2e.rs` (par les routes) | retirer `settlement_journal_entry_id` de l'ensemble |
| T2 annulée, et annulée alors que payée : achat et contre-passation ; détaché intact | idem | `include_reversals = faux` pour la fournisseur |
| T3 instantané hors `project_id`, `version`, `updated_at` identique | idem | `UPDATE` qui touche `account_id` (sonde) |
| T4 versions | idem | retirer le `version = version + 1` des écritures, puis de la facture |
| T5 paiement concurrent (deux connexions : `pay_in_tx_as` ouvert, geste en attente, commit) | `crates/kesh-db/tests/project_reassignment.rs` | geste qui lit la facture sans `FOR UPDATE` (s'il ne rougit pas à l'exécution, la déclarer équivalente au Change Log, avec la raison) |
| T6 invariant : achat et paiement créés portent le projet de la facture | idem | `project_id: None` à `supplier_invoices.rs:2163` |
| T7 période close par rôle, ordre des refus, acteur et audit dans la transaction | `project_reassignment_e2e.rs` | `403` réservé aux clés seules ; `Acteur::utilisateur` au lieu de `Acteur::from` ; permuter version et rien-à-faire |
| T8 écran | `frontend/src/routes/(app)/supplier-invoices/[id]/supplier-invoice-display-page.test.ts` | masquer le bouton ; ne pas rafraîchir le libellé |

**E2E exigé** — cas ajouté à `frontend/tests/e2e/project-reassignment.spec.ts` (de 36-16a) : facture fournisseur **payée** préparée
par l'API, geste au navigateur, `GET` de l'achat et du paiement : lignes au nouveau compte. Rouge d'abord (`e2e-rouge.log`).

## Tâches

- [ ] **Étape 0** — Db3 ; relire les lignes citées sur la base de départ ; LOW de la validation.
- [ ] **kesh-db** — `reassign_supplier_invoice` dans `project_reassignment.rs`, sur les génériques de 36-16a (`include_reversals =
  vrai`) ; audit avant le `COMMIT`.
- [ ] **kesh-api** — `supplier_invoices::reassign_project_handler` (`may_acknowledge`, `Acteur::from`), route, rejeu, registre,
  `audit_labels.rs`, paragraphe « Interblocages ».
- [ ] **kesh-i18n** — libellé d'audit et textes ×4, près des blocs `supplier-invoices-detail-*`.
- [ ] **frontend** — `reassignSupplierInvoiceProject` dans `features/projects/projects.api.ts` ; bouton et dialogue partagé sur
  `supplier-invoices/[id]/+page.svelte`.
- [ ] Tests, mutations, E2E ; captures (`kesh-gate-logs/e36-16b/captures/`) ; manuel et `api-external.md`.
- [ ] **Gates** : ceux de 36-16a (voie 1, `kesh-db` → `test-fast --ci` en boucle ; `gate-ci kesh_ci_e3616b` au passage en review ;
  `gate-ci` et E2E complet à la clôture).

## Note de version

*Ajouté* — **Changer le compte analytique d'une facture fournisseur, même payée** : le même geste que pour la facture client
réimpute la facture, son achat, son paiement et, pour une facture annulée, la contre-passation de l'achat ; en période close, il
est réservé à l'Administrateur, après le même avertissement. Pour les intégrations : `POST
/api/v1/supplier-invoices/{id}/reassign-project` (refs #1167).

## Dev Notes

### Modules touchés

`kesh-db` ; `kesh-api/routes/supplier_invoices` ; `kesh-i18n` ; `frontend/src/lib/features/projects` (appel d'API) ;
`frontend/src/routes/(app)/supplier-invoices/[id]`. **Cinq modules de code** (`features/supplier-invoices` non touché), plus la
documentation.

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
- 2026-11-15 — Propagation de la remédiation de la validation P1 de 36-16a (`kesh-gate-logs/e36-16/validation-36-16a-p1.md`) :
  analyse du paiement concurrent établie (`pay_in_tx_as` lit la facture `FOR UPDATE`, `supplier_invoices.rs:1988-1989`, et le
  projet de cette lecture, `:2163` : sérialisation, rien à modifier ; AC5, T5) ; accusé réservé à l'Administrateur (C-36-16-11) ;
  versions (AC4, T4) ; `include_reversals = vrai` éprouvé sur un état réel (T2) ; audit dans la transaction avec l'`Acteur`, ordre
  des refus, rien-à-faire sur les lignes ; appel dans `features/projects/projects.api.ts` (cinq modules) ; paragraphe
  « Interblocages ».
