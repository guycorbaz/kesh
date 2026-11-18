# Story 43-6 : « Changer le compte analytique » refusé sur une facture réglée en groupe — raccord avec la 42-1 (#823, #1167)

Status: ready-for-dev

*Fiche rédigée le 2026-11-17 au soir par l'agent de rédaction de l'Epic 43 (Opus 5.5), workflow `bmad-create-story`
(ex-T-R de la 43-1 ; validation P1, M5) ; **récrite dans la nuit** sur les décisions de l'orchestrateur : la 42-2 et la 42-3
sortent de la 0.19.0, un seul code de refus. Code lu à `origin/epic/42-reimputation-analytique` pour
`project_reassignment.rs`. Release **0.19.0**. **En dernier**, sur la branche réunie que désigne l'orchestrateur (42-1 et
43-1a fusionnées). Décision C-43-11. **Ne sort jamais** si la 42-1 et la 43-1a sont publiées ensemble (C-43-18).*

**Issues** : `refs #823`, `refs #1167`.

## Le problème

En 0.19.0, seul le geste **client** de la 42-1 existe. `reassign_invoice` (`project_reassignment.rs:410`) lit les écritures
de règlement de la facture (`SELECT journal_entry_id FROM invoice_settlements …`, :443-453), puis
`lock_entries_and_lines_in_tx` (:143), `entries_that_change` (:204), `period_exposure_in_tx` (:232) et
`reassign_lines_in_tx` (:352-363, `UPDATE journal_entry_lines SET project_id = ? WHERE entry_id IN (…)`) raisonnent **par
écriture**. Sur une écriture groupée (43-1a), il réimputerait la ligne de banque, les lignes de différence et les lignes des
autres factures, sans le dire.

## Décision

Le geste **refuse** une facture dont un règlement porte un identifiant de ligne :

- code **`PROJECT_REASSIGNMENT_GROUPED_SETTLEMENT`**, `409`, `details: { "invoiceId", "journalEntryId" }` (l'écriture
  groupée ; la plus petite si la facture en a plusieurs) ;
- **position** : dans `reassign_invoice`, **juste après le contrôle de version** de la facture, avant toute lecture des
  écritures et tout verrou de ligne ; rien d'écrit ;
- **critère** : `EXISTS (SELECT 1 FROM invoice_settlements WHERE invoice_id = ? AND company_id = ? AND
  journal_entry_line_id IS NOT NULL)` — **la facture porte une ligne attribuée**, et non « l'écriture est partagée » : une
  facture qui ne serait plus qu'une parmi zéro autre sur l'écriture (cas fournisseur détaché, en 0.20.0) reste refusée
  plutôt que de réécrire l'écriture entière ;
- message traduit dans les quatre locales ; phrase au manuel utilisateur (§ du geste de la 42-1) ; `docs/api-external.md`
  (route du geste, code et `details`) ; libellé d'erreur à l'écran du geste (dialogue de la 42-1).

*Pour la 0.20.0 (hors de cette story)* : la 42-2 reprendra le même prédicat par ligne sur
`supplier_invoices.settlement_journal_entry_line_id`, y compris pour une facture **détachée** d'un groupe
(`kesh-gate-logs/e42/validation-42-2-p2.md`, V2-1) — refusée ou limitée à sa ligne, jamais l'écriture entière. Une
réimputation limitée à la ligne (plutôt qu'un refus) pourra alors être proposée pour les deux familles.

## Critères d'acceptation

- **AC1** — Une facture réglée dans un groupe : `409 PROJECT_REASSIGNMENT_GROUPED_SETTLEMENT`, `details` exacts, aucune
  ligne réécrite (empreinte des lignes de l'écriture groupée inchangée), aucune trace d'audit du geste.
- **AC2** — Un règlement unitaire est réimputé exactement comme par la 42-1 : ses tests passent sans modification.
- **AC3** — Le refus vient après le contrôle de version : une version périmée sur une facture groupée rend le refus de
  version de la 42-1, non le code neuf (test qui fixe l'ordre).
- **AC4** — Manuel, `docs/api-external.md`, i18n (quatre locales, `sitesTotal` recompté), libellé à l'écran.

## Tests

Groupe de deux factures (43-1a), réimputation de l'une → refus ; règlement unitaire → réimputé ; ordre version puis
groupe. Mutation : retirer la garde (AC1 rougit, les lignes de l'autre facture changent).

## Note de version

- **Modifié** — « Changer le compte analytique » d'une facture réglée avec d'autres par un même virement est refusé, avec
  un message qui le dit (refs #823, refs #1167).

## Dev Notes

Modules : `repositories/project_reassignment` (et son erreur), la route de la 42-1 si elle traduit l'erreur, i18n,
manuel, `docs/api-external.md`. Petite : prête **mercredi soir** sur une branche de travail réunie.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-17 — Fiche rédigée (ex-T-R de la 43-1), deux issues (repli, raccord complet).
- 2026-11-17 (nuit) — Récrite : 42-2 et 42-3 hors 0.19.0 ; une seule issue, le refus
  `PROJECT_REASSIGNMENT_GROUPED_SETTLEMENT` après le contrôle de version, critère par ligne ; la 42-2 reprend le prédicat
  en 0.20.0.
