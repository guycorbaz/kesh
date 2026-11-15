# Story 36-16c : changer le compte analytique des lignes d'une écriture passée, même rapprochée ou lettrée (#1167)

Status: backlog

*Fiche rédigée le 2026-11-15 par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-16`, par le workflow
`bmad-create-story`. **Troisième des trois stories nées du découpage de la 36-16** (`36-16-reimputation-analytique.md`, `Status:
split` ; C-36-16-10). **Part de la 36-16a fusionnée** dans `epic/36-0.18.0`, dont elle réemploie le dépôt
(`project_reassignment.rs`, `period_exposure_in_tx`), le dialogue partagé (plusieurs rangées, D12), l'accusé de période close et
les décisions communes D0 à D12 (C-36-16-1 à -9) : la fiche **y renvoie**. Indépendante de 36-16b. Phase 2, release **0.18.0**.
**Aucune migration.** **`kesh-db` touché : gate complet même en cours de boucle.** Voie 1 (bases `kesh_e3616c`,
`kesh_e2e_e3616c`, `kesh_ci_e3616c`). Code lu à **`fed34a9c7`** — relu à l'étape 0 sur la base de départ. `Status: backlog` tant
que 36-16a n'est pas fusionnée. Validation non lancée.*

**Issues** : `refs #1167` (dernière des trois : la PR de release peut porter le mot-clé de fermeture) ; aucun mot-clé dans un
commit. **Vocabulaire** : celui de la 36-15 fusionnée (de-CH, it-CH, en-CH en attente de l'arbitrage de Guy).

## Story

En tant que titulaire qui catégorise ses transactions bancaires par des écritures de contrepartie, je veux **changer le compte
analytique des lignes d'une écriture déjà passée** — rapprochée, lettrée, ou d'un exercice clos —, sans la modifier ni la
contre-passer, afin de corriger mes rapports analytiques ; une écriture contre-passée et sa contre-passation doivent changer
ensemble, pour que les rapports restent équilibrés.

## Ce qui se lit au code (`fed34a9c7`)

- **Le projet est par ligne** sur l'écriture manuelle : création `journal_entries.rs:521` (`line.project_id.or(new.project_id)`),
  modification `:1681` ; manuel `user-manual.tex:3782-3783` (« un sélecteur de projet *par ligne* ») ; écran
  `routes/(app)/journal-entries/[id]/+page.svelte:626` (libellé par ligne), `:200-202` (`projectLabel`).
- **Le `PUT` d'une écriture est gelé** par `modification_blocker` (`journal_entries.rs:1260`) : exercice clos (`:1280-1282`),
  postérieur clos (`:1283-1288`), garde de propriété (`modification_guard`, `:1112`, qui reprend `reversal_blockers` hors
  `AccountArchived` — dont `MatchedBankTransaction`, `errors.rs:59`), verrou de période (`:1293-1300`), lettrage
  (`lettering_guard`). Une écriture **rapprochée** — le cas de toutes les contreparties bancaires — n'est donc pas réimputable
  aujourd'hui. Et le `PUT` **supprime puis réinsère** les lignes (`INSERT` `:1672`) : il perdrait le lettrage, d'où son refus.
- **La propriété** : `document_owners` (`journal_entries.rs:2272`), `DocumentKind` (`:2113-2128` : `Invoice`, `CreditNote`,
  `SupplierInvoice`, `Settlement`, `BankTransaction`).
- **La contre-passation** recopie les lignes **dans l'ordre** : lecture `… ORDER BY line_order FOR UPDATE` (`:2921-2923`), copie
  `:2942-2951` ; lien `reverses_entry_id` (`migrations/20260828000002_journal_entries_reversal.sql:44`), une seule contre-passation
  par écriture (`ReversalBlocker::AlreadyReversed`, `uq_journal_entries_reverses`), une contre-passation ne se contre-passe pas
  (`IsAReversal`).
- **Paires déliées** des stories a et b : un règlement client annulé (ligne `invoice_settlements` supprimée,
  `invoice_settlements_write.rs:1043`), un paiement fournisseur annulé (`supplier_invoices.rs:2929`) ou détaché
  (`journal_entries.rs:1074-1093`) — aucune pièce ne les possède plus : ce geste les atteint.
- **Contrôle des projets d'une modification** : `journal_entries.rs:1517-1530` (projets neufs seuls, `validate_taggable_in_tx`).
- **Route voisine** : `POST /api/v1/journal-entries/{id}/reverse` (`lib.rs:570`, bloc `comptable_routes` ouvert
  `:536`).

## Décisions

*D0 à D12 : celles de **36-16a**.* Propres à cette story (C-36-16-3) :

- **Dc1 — Route** : `POST /api/v1/journal-entries/{id}/reassign-project`, corps `{ "lines": [{ "lineId": n, "projectId": n | null
  }], "version": n, "acknowledgeClosedPeriods"?: bool }` ; réponse : l'écriture (forme du `GET`) et `journalEntryIds`. Contrat :
  ajout seulement.
- **Dc2 — Par ligne** : seules les lignes nommées changent, chacune nommée une fois ; ligne d'une autre écriture, doublon, liste
  vide → `400 VALIDATION_ERROR`. *Écartée* : un compte unique pour l'écriture (écrase la ventilation).
- **Dc3 — La paire** : si l'écriture est contre-passée, ou est une contre-passation, la ligne de **même `line_order`** de l'autre
  écriture prend le même compte ; `journalEntryIds` nomme les deux ; la période close s'évalue sur les deux. *Écartée* : apparier
  par compte et montant (ambigu dès deux lignes identiques).
- **Dc4 — Refus des écritures de pièce** : si l'écriture, ou l'autre de la paire, a un propriétaire autre que `BankTransaction`
  (`document_owners`) → **`409 PROJECT_REASSIGNMENT_USE_DOCUMENT`**, `details { documentKind, documentId, documentLabel }` ; le geste
  est celui de la pièce (36-16a, 36-16b). Rapprochée, lettrée, exercice clos ou période verrouillée : **acceptée** (D5, accusé).
- **Dc5 — Écran** : *Changer le compte analytique* sur la fiche de l'écriture pour Admin et Comptable, sauf écriture de pièce (le
  motif existant de la fiche reste) ; le dialogue partagé avec une rangée par ligne (compte et montant pour libellé).
- **Dc6 — Audit** : `journal_entry.project_reassigned`, entité l'écriture visée, `details { lines: [{ entryId, lineId, from, to }],
  journalEntryIds, closedPeriodsAcknowledged }`.

## Critères d'acceptation

1. **AC1** — Écriture manuelle **rapprochée** de trois lignes sur deux comptes analytiques : nommer une ligne vers Q rend `200` ;
   cette ligne seule change ; montants, comptes, dates, numéro, lettrage, rapprochement intacts.
2. **AC2** — Écriture **lettrée**, puis écriture d'un **exercice clos** : `409 PROJECT_REASSIGNMENT_CLOSED_PERIOD` sans accusé pour
   la seconde, `200` avec ; le lettrage reste.
3. **AC3** — **Paire** : sur une écriture contre-passée (et, séparément, sur sa contre-passation), la ligne de même rang de l'autre
   écriture prend le même compte ; un règlement client annulé et sa contre-passation se réimputent ainsi.
4. **AC4** — Refus : écriture de vente, de règlement, d'avoir, d'achat ou de paiement fournisseur → `409
   PROJECT_REASSIGNMENT_USE_DOCUMENT` nommant la pièce ; ligne étrangère, doublon, liste vide → `400` ; compte archivé, inconnu,
   version, rôle, portée → refus de 36-16a ; rien d'écrit.
5. **AC5** — Audit `journal_entry.project_reassigned`, acteur compris (clé d'API), libellé ×4 ; toutes lignes déjà à leur valeur →
   `200` sans écriture ni audit.
6. **AC6** — Un `PUT` lu avant le geste est refusé ensuite (`409 OPTIMISTIC_LOCK_CONFLICT`, version incrémentée, D7).
7. **AC7** — Écran : dialogue d'une rangée par ligne, avertissement, succès, libellés rafraîchis ; écriture de pièce, rôle Lecture :
   pas de bouton.
8. **AC8** — Route au registre `audit_route_registry.rs` ; `audit_acteur_garde.rs` vert sans entrée neuve.
9. **AC9** — Manuel (sous-section de 36-16a : l'écriture passée, la paire, le refus des écritures de pièce, les paires déliées) et
   `docs/api-external.md` (route, codes au § 10) ; textes ×4.

## Tests à écrire, et la preuve du rouge

Premier rouge : route `404` avant le correctif (`kesh-gate-logs/e36-16c/rouges.log`) ; puis mutations (E5, `kesh-gate-logs/e36-16c/mut/`).

| Test | Lieu | Mutation |
|---|---|---|
| T1 lignes nommées seules, rapprochée et lettrée intactes | `crates/kesh-db/tests/project_reassignment.rs` | `UPDATE` sur toutes les lignes de l'écriture |
| T2 paire par `line_order`, depuis l'original et depuis l'inverse | idem | apparier par `id` de ligne ; ne suivre la paire que dans un sens |
| T3 refus d'une écriture de pièce (chaque `DocumentKind` sauf banque) | idem | retirer le contrôle `document_owners` ; y laisser passer `Settlement` |
| T4 validation du corps (étrangère, doublon, vide) | `crates/kesh-api/tests/project_reassignment_e2e.rs` | retirer le contrôle d'appartenance |
| T5 version et `PUT` périmé | idem | ne pas incrémenter `journal_entries.version` |
| T6 accusé et acteur | idem | court-circuit du contrôle de période ; `Acteur::utilisateur` |
| T7 écran | `routes/(app)/journal-entries/[id]/journal-entry-page.test.ts` | bouton affiché sur une écriture de vente |

**E2E exigé** — cas ajouté à `frontend/tests/e2e/project-reassignment.spec.ts` : écriture de contrepartie **rapprochée** préparée
par l'API (import manuel d'un relevé et rapprochement), une ligne changée au navigateur, `GET /journal-entries/{id}` : la ligne au
nouveau compte, la transaction toujours rapprochée. Rouge d'abord (`e2e-rouge.log`).

## Tâches

- [ ] **Étape 0** — relire les lignes citées sur la base de départ (36-16a fusionnée) ; LOW de la validation.
- [ ] **kesh-db** — `reassign_entry_lines` dans `project_reassignment.rs` (paire, `document_owners`, `period_exposure_in_tx` de
  36-16a) ; variante `DbError::ProjectReassignmentUseDocument` et son code aux deux tables de `errors.rs`.
- [ ] **kesh-api** — `journal_entries::reassign_project_handler`, route, rejeu, registre, `audit_labels.rs`, mappage `409` et `details`.
- [ ] **kesh-i18n** — libellé d'audit, motif de refus et textes ×4, près des blocs `journal-entry-*`.
- [ ] **frontend** — bouton et dialogue partagé (une rangée par ligne) sur `journal-entries/[id]/+page.svelte` ; s'il faut
  étendre `ProjectReassignDialog.svelte`, ses tests de 36-16a restent verts.
- [ ] Tests, mutations, E2E ; captures (`kesh-gate-logs/e36-16c/captures/` : dialogue multi-lignes, refus d'une écriture de pièce) ;
  manuel et `api-external.md`.
- [ ] **Gates** : ceux de 36-16a (voie 1 ; `gate-ci kesh_ci_e3616c`).

## Note de version

*Ajouté* — **Changer le compte analytique d'une écriture passée** : sur la fiche d'une écriture, même rapprochée, lettrée ou d'un
exercice clos, *Changer le compte analytique* modifie les lignes choisies sans toucher au reste ; une écriture contre-passée et sa
contre-passation changent ensemble. Les écritures d'une facture se réimputent par la facture. Pour les intégrations :
`POST /api/v1/journal-entries/{id}/reassign-project` et le code `PROJECT_REASSIGNMENT_USE_DOCUMENT` (refs #1167).

## Dev Notes

### Modules touchés

`kesh-db` ; `kesh-api/routes/journal_entries` ; `kesh-i18n` ; `routes/(app)/journal-entries/[id]` ; `features/projects` (si le
dialogue s'étend). **Cinq modules de code au plus**, plus la documentation.

### Hors périmètre

Les factures (36-16a, 36-16b) ; le `PUT` et sa garde, inchangés ; le MCP.

### Fichiers communs

`project_reassignment.rs`, `ProjectReassignDialog.svelte`, `project-reassignment.spec.ts`, manuel, `api-external.md`,
`messages.ftl`, `errors.rs` : partagés avec 36-16a (fusionnée avant) et 36-16b (parallèle possible : blocs distincts, union à
l'intégration). `journal_entries.rs` : lu, non écrit (sauf exposition de `document_owners`, déjà publique). `i18n-keys.test.ts`
recompté. `CHANGELOG.md` jamais écrit.

### Ce qui doit rester vrai

Celles de 36-16a ; le lettrage et le rapprochement d'une écriture réimputée restent ; une paire est toujours sur un seul compte,
ligne à ligne.

### Références

Fiches `36-16-reimputation-analytique.md` (origine) et `36-16a-reimputation-facture-client.md` (décisions communes) ; #1167 ;
stories 19-2, 24-4a, 15-8a, 15-1b-0.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-15 — Création (Opus 5.5), par découpage de `36-16-reimputation-analytique.md` (C-36-16-10). Reprend son lot C (D4) et la
  réimputation des paires déliées. Décisions communes : 36-16a. Code lu à `fed34a9c7`. Aucune migration.
