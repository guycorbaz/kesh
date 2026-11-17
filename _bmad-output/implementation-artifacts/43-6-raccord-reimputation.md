# Story 43-6 : « Changer le compte analytique » d'une facture réglée en groupe — raccord avec l'Epic 42 (#823, #1167)

Status: ready-for-dev

*Fiche rédigée le 2026-11-17 au soir par l'agent de rédaction de l'Epic 43 (Opus 5.5), workflow `bmad-create-story`
(ex-T-R de la 43-1 ; validation P1, M5). Code lu à `origin/epic/42-reimputation-analytique` pour `project_reassignment.rs`.
Release **0.19.0**. **En dernier**, sur la branche réunie que désigne l'orchestrateur (42-1 et 43-1a fusionnées, 42-2 et
43-1b si closes). Décision C-43-11 ; **repli approuvé par Guy** (Q3, 2026-11-17).*

**Issues** : `refs #823`, `refs #1167`.

## Le problème

`reassign_invoice` (`project_reassignment.rs:410`) lit les écritures de règlement de la facture (`SELECT journal_entry_id
FROM invoice_settlements …`, :443-453), puis `lock_entries_and_lines_in_tx` (:143), `entries_that_change` (:204),
`period_exposure_in_tx` (:232) et `reassign_lines_in_tx` (:352-363, `UPDATE journal_entry_lines SET project_id = ? WHERE
entry_id IN (…)`) raisonnent **par écriture**. Sur une écriture groupée (43-1a), il réimputerait la ligne de banque, l'écart
et les lignes des autres factures, sans le dire.

## Deux issues, dans cet ordre

**Repli — R (obligatoire si 42-1 et 43-1a sont publiées ensemble ; prêt mercredi soir)** : le geste **refuse** une facture
dont un règlement porte un identifiant de ligne : `409 PROJECT_REASSIGN_GROUPED_SETTLEMENT` (`details.invoiceId`), rien
d'écrit, message traduit dans les quatre locales, phrase au manuel utilisateur. Une garde de quelques lignes, en tête de
`reassign_invoice`, sous le verrou de la facture. Test : un groupe, réimputation refusée ; un règlement unitaire,
réimputation inchangée. Côté fournisseur, même garde dans le geste de la 42-2 si elle est fusionnée, sinon une issue.

**Complet — C (si le temps le permet ; remplace R)** :

- la lecture des cibles devient **par ligne** : pour un règlement à identifiant de ligne, la cible est cette ligne ; sinon
  toute l'écriture (comportement de la 42-1) ;
- `reassign_lines_in_tx` prend des identifiants de lignes (ou un second chemin par lignes, sans dupliquer l'`UPDATE`) ;
- `entries_that_change` et le calcul « rien à faire » ne regardent que les lignes ciblées ; `period_exposure_in_tx` compte
  les écritures **des lignes ciblées** ;
- l'audit du geste nomme les lignes réimputées ;
- tests : groupe de deux factures à projets différents, réimputation de l'une — ses ligne de vente et ligne de créance
  changent, l'autre facture, la banque et l'écart non ; exposition à une période close comptée une fois ; côté
  fournisseur si 42-2 fusionnée, sinon issue.

## Critères d'acceptation

- **AC1** — R ou C est livré ; jamais aucun des deux si 42-1 et 43-1a sont publiées ensemble.
- **AC2** — Un règlement unitaire est réimputé exactement comme par la 42-1 : ses tests passent sans modification.
- **AC3** — Manuel utilisateur et `docs/api-external.md` disent le comportement livré.

## Note de version

- **Modifié** — « Changer le compte analytique » d'une facture réglée avec d'autres par un même virement : *(R)* refusé,
  avec un message qui le dit / *(C)* seule la ligne de la facture change (refs #823, refs #1167).

## Dev Notes

Modules : `repositories/project_reassignment` (et sa route), i18n, manuel. Le second arrivé résout dans sa branche ; une
contradiction avec la 42-2 se règle avec son auteur, Guy arbitre.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-17 — Fiche rédigée (ex-T-R de la 43-1).
