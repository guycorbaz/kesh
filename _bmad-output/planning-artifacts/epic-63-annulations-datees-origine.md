# Epic 63 — Annulations datées de l'origine

*Kickoff du 2026-11-27 (décision du Project Lead du 2026-11-27 : la story de correction de la release v0.21.0).* Release visée :
**v0.21.0, vendredi 2026-12-04**, avec l'Epic 53. Aucune migration. L'epic n'entre dans `main` que terminé.

## Objet

Trois défauts **P1** de même classe que #1674 (Epic 53, story 53-6a) : une annulation faite depuis la fiche d'une pièce contre-passe son
écriture **à la date du jour** ; faite après la fin de l'exercice de l'écriture, elle place l'écriture et son inverse dans deux exercices —
bilan faux, et pour un solde ou une facture fournisseur, résultat et TVA faux.

## Story

| Story | Issues | Objet |
|---|---|---|
| 63-1 — Les annulations depuis la fiche contre-passent dans l'exercice de l'écriture, à sa date | #1752, #1753, #1754 (P1) | annuler un règlement client ou un solde (#1752), un paiement fournisseur (#1753), une facture fournisseur (#1754) contre-passe à la date de l'écriture, dans son exercice ; exercice clos : refus (inchangé) ; période verrouillée : refus annoncé sur la fiche. Fiche `_bmad-output/implementation-artifacts/63-1-annulations-datees-origine.md`. |

Une story. ⚠️ **Story de solde** : CLAUDE.md en veut une par epic ; pour un epic d'une seule story de correction, la fiche propose de verser sa dette à la story de solde de l'Epic 53 (53-5) ou en issues — **décision du Project Lead** (Q-D de la fiche).

## Dépendances

- **Dépend de la 53-6a** (#1674, Epic 53) : la 63-1 réemploie la datation `CancellationDating::OriginDate` et le rang `PeriodLocked` de la
  queue commune qu'elle pose. **Branche** : `story/63-1-annulations-datees-origine`, à repartir d'`epic/53-ecritures-justes` **une fois la
  53-6a fusionnée** (la fiche a été rédigée sur une branche partie de `story/53-6-annulation-rapprochement-exercice` à `6b71d4c21`).
- Recoupe la **53-6b** (détection des contre-passations tombées dans un autre exercice) : question Q-B de la fiche — l'étendre aux trois gestes.
- Bloque : aucune.

## Fichiers et modules (déclarés par la fiche)

- `kesh-db` : `repositories/invoice_settlements_write.rs`, `supplier_invoices.rs` (gestes, vue, `commit_with_view`),
  `settlement_cancellation.rs` et `errors.rs` (doc seulement).
- `kesh-api` : `routes/invoices.rs`, `routes/supplier_invoices.rs`, `routes/imported_supplier_invoices.rs` (appels de `commit_with_view`),
  `services/factures.rs`, `services/factures_fournisseur.rs`, `services/mod.rs`, `errors.rs`.
- `kesh-i18n` : les quatre catalogues.
- Frontend : `lib/features/invoices/`, `lib/features/supplier-invoices/`, `lib/shared/utils/settlement-cancel-blocked.ts`, pages
  `routes/(app)/invoices/[id]` et `routes/(app)/supplier-invoices/[id]`.
- Documentation : `docs/manual/fr/user-manual.tex`, `admin-manual.tex`, `docs/api-external.md`.

Cinq modules (borne de la règle de splitting : « plus de cinq ») ; découpage de repli écrit dans la fiche (63-1a client, 63-1b fournisseur).

## Recoupements avec l'Epic 53

- **53-6a** : mêmes fichiers (`invoice_settlements_write.rs`, `supplier_invoices.rs`, `settlement_cancellation.rs`, `kesh-api/src/errors.rs`,
  catalogues, `user-manual.tex` — encadré du verrou de période, `sec:date-du-jour` —, `api-external.md`) ; la 63-1 part de la 53-6a fusionnée.
- **53-2** (#525, #1198, migration probable) : `invoice_settlements_write.rs` (correction de TVA du solde) — fonctions distinctes ; la
  seconde fusionnée reprend la première.
- **53-6b** : la détection ne lit aujourd'hui que les annulations de rapprochement (Q-B de la fiche 63-1).
- Hors Epic 53 : **Epic 38** (contributeur, datation choisie de `/reverse`) touche le socle de contre-passation — à contrôler à la revue du
  code d'intégration.

## Ordre d'intégration

53-6a fusionnée dans `epic/53-ecritures-justes`, puis 63-1 développée depuis elle ; l'Epic 63 entre dans la release v0.21.0 avec l'Epic 53
(une seule entrée dans `main` chacun). Notes de version : ⚠️ en tête, avec celle de la 53-6a (même règle, trois gestes de plus).
