# Epic 34 — choix autonomes de la 34-5 (clé e345)

*Registre tenu en autonomie (Opus 5.5) le 2026-11-10, au développement de la Story 34-5 (#951). Worktree
`/home/gcorbaz/devel/kesh-e345`, branche `story/34-5-reglement-idempotent`, base `665de151`. Les choix de la fiche sont
au registre du groupe, `e34-choix-autonomes-e343.md` (C-e343-1 à 5 et 13). Mise à jour par union, jamais par écrasement.*

## C-e345-1 — deux appelants gardent une lecture locale de la version

- **Contexte.** C-e343-13 retient une aide par crate de test. Deux appelants ne peuvent pas l'atteindre proprement :
  `kesh-db/tests/support/lettering_documents.rs`, fichier d'appui inclus par `#[path]` **dans deux crates** (`kesh-db`
  et `kesh-api/tests/rejeu_interblocage_e2e.rs`), et `kesh-report/tests/aged_receivables.rs`.
- **Retenu.** Une lecture locale de `SELECT version FROM invoices WHERE id = ?` dans chacun, juste avant l'appel.
- **Écartées.** Un `#[path]` imbriqué vers `support/reglement_version.rs` depuis `lettering_documents.rs` : il marche
  (chemin relatif au fichier), mais compile une seconde copie de l'aide dans les binaires qui l'incluent déjà, et rend
  le fichier d'appui dépendant d'un autre pour une ligne. Pour `kesh-report`, la fiche prescrivait déjà la lecture locale.
- **Réversible.** Oui.

## C-e345-2 — `kesh-db/tests` : la version en huitième argument, sauf dans `invoice_settlement.rs`

- **Contexte.** Sept fichiers de `kesh-db/tests` appellent `settle_invoice`. Dans `invoice_settlement.rs` (16 appels), le
  renommage en `settle_at_current_version(` garde les arguments. Dans les cinq autres, un appel chacun, le renommage
  aurait laissé `invoice_settlements_write` importé sans usage dans `filet_bilan_clos.rs` (avertissement, donc clippy
  rouge).
- **Retenu.** Les cinq fichiers à un appel gardent `invoice_settlements_write::settle_invoice(` et passent
  `reglement_version::version_de(…)` en huitième argument ; `invoice_settlement.rs` emploie l'aide entière. Les
  règlements successifs (`:204/:221`, `:2228/:2246` à la base) passent la version de la sortie du précédent, comme la
  fiche le demande. `invoices_validate_vat.rs` passe `v.invoice.version`, la version rendue par la validation.
- **Réversible.** Oui.

## C-e345-3 — les tests HTTP de l'AC-1 à l'AC-6 dans `invoice_settlement_response_e2e.rs`

- **Retenu.** Le fichier de la 33-6, qui a déjà le montage (`validated_invoice`, `spawn_app`) et l'aide `settle`. Une
  fonction libre `settle_request` sert à la fois `TestApp` et les tâches de l'AC-3 (règle DRY).
- **Écartée.** Un fichier neuf `invoice_settlement_idempotence_e2e.rs`, qui aurait recopié le montage.
- **Réversible.** Oui.

## C-e345-4 — deux états de plus, une relecture paramétrée, un encadré à part

- **Retenu.** Dans la fiche et l'échéancier : `settleOutcome` gagne `conflict` et `conflict-blocked` ;
  `rereadAfterSettle(blocking, arrival)` remplace `rereadAfterUnknownOutcome` ; `rereadFromBlocked()`, appelé par
  *Relire*, choisit l'arrivée d'après l'état bloquant (`conflict-blocked` → `conflict`, `blocked` → `verify`) ;
  `settleBlocked` (dérivé) désactive le geste dans les deux états bloquants. L'encadré du conflit est distinct
  (`settle-conflict`, `settle-conflict-reread` ; `due-dates-settle-conflict`, `due-dates-settle-conflict-reread`). Le
  libellé *Relire* reprend la clé `invoice-settle-unknown-reread`, dont le texte ne parle pas d'issue.
- **Pas de notification** (`notifyError`) sur le 409 : l'encadré, persistant, suffit, comme pour l'issue inconnue.
- **Réversible.** Oui.

## C-e345-5 — l'échéancier nomme la facture du conflit par `settleUnknownNumber`

- **Retenu.** Le même état que l'issue inconnue, posé depuis `cible.invoiceNumber` avant la relecture. Les deux avis
  sont exclusifs, un seul numéro est à retenir.
- **Écartée.** Un état `settleConflictNumber` de plus, sans usage propre.
- **Réversible.** Oui.

## C-e345-6 — `invoiceVersionViaApi` pour les specs Playwright

- **Contexte.** Huit spécifications postent un règlement par l'API.
- **Retenu.** Une aide `invoiceVersionViaApi(ctx, id)` dans `tests/e2e/helpers/api-fixtures.ts`, qui lit la version par
  `GET` comme un client ; chaque corps gagne `version: await invoiceVersionViaApi(ctx, id)`. La spec neuve de la story
  est `invoices-settle-version.spec.ts`.
- **Réversible.** Oui.

## C-e345-7 — `Passer à la 0.17.6` créée au plus court

- **Contexte.** AC-11 bis : la section est à créer, et elle se fusionne par union avec les autres stories de l'epic, dont
  la 34-11a1 qui porte une migration breaking.
- **Retenu.** Une phrase d'introduction (« Depuis une version antérieure à la 0.17.5, appliquez d'abord les sections
  suivantes ») et le paragraphe de la 34-5. Rien sur le compose, les variables, la migration ni la sauvegarde : la
  34-5 n'en porte pas, et l'écrire ici risquerait de contredire la 34-11a1.
- **Réversible.** Oui.

## C-e345-8 — `recette-montee.sh` refuse une version illisible

- **Retenu.** `VF2=$(… jq -r '.version // empty')`, contrôlé `^[0-9]+$`, sinon `echec 1 "version de la facture 2
  illisible"`, comme le reste dû juste au-dessus.
- **Écartée.** Omettre le champ quand il manque : la recette passerait sur une 0.17.5 et rendrait 422 plus loin, sur
  une 0.17.6, avec un message moins clair.
- **Réversible.** Oui.

## C-e345-9 — `post_settle` de l'échéancier lit la version par `GET`

- **Contexte.** `invoice_echeancier_e2e.rs::post_settle` ne reçoit pas de pool.
- **Retenu.** Un `GET /api/v1/invoices/{id}` avant le `POST`, comme un client. Les autres corps du fichier lisent la
  version en base par `common::invoice_version`.
- **Réversible.** Oui.

## C-e345-10 — échéancier : tout chargement réussi lève l'état bloquant (revue P1, E1-5 = B1-3)

- **Contexte.** Une relecture d'après un règlement (issue inconnue ou 409) supplantée par un chargement de filtre rendait
  `false` : l'écran restait bloqué et disait « la liste n'a pas pu être relue » sur une liste fraîche. Motif hérité de la
  33-6 pour `blocked`.
- **Retenu.** `load()`, sur un succès qui n'est pas supplanté, fait passer `blocked` à `verify` et `conflict-blocked` à
  `conflict`. Une liste relue avec succès, quel qu'en soit le filtre, porte des montants et des versions frais : c'est la
  condition que l'état bloquant attendait. `rereadAfterSettle` garde sa propre transition, désormais redondante.
- **Conséquence.** Après une relecture en échec, un changement de filtre réussi rouvre aussi le geste, sans passer par
  *Relire* ; jusqu'ici, *Relire* était le seul moyen.
- **Écartée.** Un `load()` à trois issues (réussi, supplanté, en échec) : plus long, et il faudrait encore attendre le
  chargement qui supplante.
- **Réversible.** Oui.

## C-e345-11 — échéancier : la fermeture d'un dialogue ouvert sur une autre ligne n'est pas changée ici (revue P1, E1-6)

- **Contexte.** Dans les trois branches de `handleSettleConfirm` (succès, issue inconnue, 409), `markOpen = false` et
  `markTarget = null` ferment le dialogue courant, même s'il a été rouvert sur une autre ligne pendant l'attente. Le
  comportement est hérité de la 33-6, et la 34-5 ne l'a fait que reprendre dans sa branche neuve.
- **Retenu.** Corriger le commentaire inexact de la branche 409, sans changer le comportement des trois branches. Une
  issue est demandée *(ouverte : #1007 ; annoté le 2026-11-11 à la revue P2, A2-1)*.
- **Écartée.** Ne fermer que si `markTarget === cible` dans la seule branche 409 : les trois branches divergeraient.
- **Réversible.** Oui.
