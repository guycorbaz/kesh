# Story 25.2-b-zero : Le verrou de période protège aussi de la suppression

Status: ready-for-dev

**Issue : [#443]**, ouverte le 2026-09-19. Détachée de la **25-2-b** (#440) sur arbitrage de Guy du
même jour : le trou existe sur `main` indépendamment de la dévalidation, et le fermer d'abord allège
une story qui frôlait le seuil de découpage.

## Story

En tant que **comptable qui a déclaré un trimestre de TVA**,
je veux qu'**aucune écriture d'une période verrouillée ne puisse disparaître**,
afin que **les totaux d'une période close ne changent plus dans mon dos** — ce que le verrou de
période promet déjà, et ne tient qu'à moitié.

## Le défaut, établi

Le verrou de période (Story 24-4c, #380) est contrôlé à un seul endroit : la **création** d'une
écriture, `create_in_tx_inner` (`crates/kesh-db/src/repositories/journal_entries.rs`, étape 0-bis —
lecture de `companies.books_locked_through` — et garde `entry_date <= locked_through` plus bas).

`delete_in_tx` (`journal_entries.rs:988`) ne le lit pas. La 24-4c pouvait s'en passer tant que le
gel de la 24-4b refusait **toute** suppression — et il la refuse bien sur la route
`DELETE /api/v1/journal-entries/{id}` (`enforce_immutability = true`, `409 ENTRY_IS_POSTED`). Mais
un chemin y échappe par construction : `invoices::delete` sur une facture **validée** (#219) appelle
`delete_in_tx(…, false)` (`invoices.rs:1350`). Supprimer aujourd'hui une facture validée datée d'un
trimestre verrouillé efface son écriture, et le rapport TVA — recalculé à la volée — change sans
aucun signal.

Le manuel dit le contraire, **deux fois** :

- `docs/manual/fr/user-manual.tex:464` — « Aucune écriture ne pourra plus y être datée, **par aucun
  chemin** » ;
- `docs/manual/fr/admin-manual.tex:1798` — « aucune écriture ne peut plus être datée d'une période
  déjà déclarée […], par aucun chemin ».

⚠️ Les deux phrases parlent de **dater**, et restent littéralement vraies : on ne *date* pas une
écriture en la supprimant. Mais c'est exactement la lecture qui laisse passer le défaut — la
promesse faite au lecteur est que les totaux d'une période verrouillée ne bougent plus.

## Acceptance Criteria

1. **`delete_in_tx` refuse une écriture datée d'une période verrouillée.** Si
   `companies.books_locked_through` est posé et que `entry_date <= books_locked_through` — seuil
   **inclusif**, le même qu'à la création —, la suppression est refusée par
   `DbError::PeriodLocked { locked_through, attempted }`, variante **existante** (code
   `PERIOD_LOCKED`, `400`), sans nouvelle variante ni nouveau code.

2. **La garde vit DANS `delete_in_tx`, jamais chez l'appelant** — pour la raison qui y fixe déjà
   `enforce_immutability` : posée chez `invoices::delete`, elle laisserait la fonction nue pour le
   prochain appelant, et la 25-2-b (dévalidation) en ajoute précisément un.

3. **Elle s'évalue APRÈS le gel (étape 3-ter).** Sur la route gelée `DELETE /journal-entries/{id}`,
   une écriture de période verrouillée reste refusée en `409 ENTRY_IS_POSTED`, **comme aujourd'hui** :
   évaluer la période avant le gel changerait ce contrat en `400 PERIOD_LOCKED` sans qu'aucune story
   ne l'ait demandé. C'est aussi la règle de la création : *le verrou de période parle en dernier*.
   Ordre résultant dans `delete_in_tx` : exercice clos → contre-passée → gel → **période**.

4. **La borne se lit SANS verrou, comme à la création.** `create_in_tx_inner` documente pourquoi
   (étape 0-bis) : un `FOR SHARE` sur `companies` sérialiserait chaque opération derrière toute pose
   de borne, pour un gain nul — et, dans `delete_in_tx`, qui verrouille d'abord
   `journal_entries JOIN fiscal_years`, il **inverserait l'ordre global des verrous**
   (companies → … → fiscal_years), au risque d'un interblocage ABBA. Le commentaire de la garde
   renvoie à celui de la création plutôt que de le recopier.

5. **La date comparée est celle de l'écriture** (`journal_entries.entry_date`), lue par la requête
   de verrouillage existante de l'étape 2 — qui ne la ramène pas aujourd'hui : l'étendre, ne pas
   ajouter une seconde lecture de la ligne.

6. **Tests** (dépôt, `journal_entries.rs` `mod tests`) :
   - `delete_in_tx(…, false)` sur une écriture datée **du jour de la borne** → `PeriodLocked`
     (le seuil est inclusif) ;
   - datée du **lendemain** de la borne → supprimée ;
   - **sans borne** → supprimée ;
   - `delete_in_tx(…, true)` sur une écriture de période verrouillée → **`EntryIsPosted`**, et non
     `PeriodLocked` (AC 3 : l'ordre).
   Et bout en bout (`crates/kesh-api/tests/`) : `DELETE /api/v1/invoices/{id}` sur une facture
   **validée** de période verrouillée → `400 PERIOD_LOCKED`, facture **et** écriture intactes.
   ⛔ **Chaque test est prouvé par mutation** : garde retirée → les tests « refusé » rougissent sur
   leur assertion ; `<=` muté en `<` → le test du seuil rougit ; garde déplacée avant le gel → le test
   d'ordre rougit.
   ⚠️ **Le montage pose la borne, puis la retire.** Les tests de dépôt partagent la base de dev : une
   borne laissée posée ferait tomber le gate suivant (règle *« un gate laisse la base piégée »*,
   KF-039). La retirer **avant** l'assertion, pour qu'elle parte même si le test rougit. ⚠️ La pose
   passe par `companies::lock_books`, qui refuse une borne future ou du jour : dater les écritures
   du test **avant** la borne, elle-même antérieure à aujourd'hui.

7. **Les manuels.** Les deux phrases citées plus haut deviennent vraies au sens où le lecteur les
   comprend : ajouter que le verrou empêche aussi de **supprimer** une écriture de la période (ce qui
   n'arrive plus que par la suppression d'une facture validée). PDF régénérés, **contrôlés à plat**
   (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`). Et `CHANGELOG.md`, section « Corrigé ».

8. **#443 se ferme** — `closes #443` dans le **titre ET le corps** de la PR ; `refs` sur les commits.

## Tasks / Subtasks

- [ ] **T1 — La garde** (AC 1-5) : étendre la requête de l'étape 2 à `entry_date`, lire la borne,
      refuser après l'étape 3-ter.
- [ ] **T2 — Tests et mutations** (AC 6).
- [ ] **T3 — Manuels, PDF, CHANGELOG** (AC 7).
- [ ] **T4 — Gates complets** — gate backend **complet** et non ciblé : la story touche un
      repository de `kesh-db`. E2E complète avant le push.

## Dev Notes

### Ce que cette story ne fait pas

- Elle ne touche **pas** à `invoices::delete` : la suppression d'une facture validée reste possible
  hors période verrouillée, et c'est la **25-2-b** qui la remplace par la dévalidation.
- Elle ne ferme **pas** le second trou relevé le même jour — une facture **partiellement** réglée se
  supprime, la garde ne lisant que `paid_at` : il vit dans la branche `validated` d'`invoices::delete`,
  que la 25-2-b retire. Le corriger ici réécrirait du code condamné.

### Appelants de `delete_in_tx` (inventaire au `44c6842f`)

| Appelant | `enforce_immutability` | Effet de la garde |
|---|---|---|
| `journal_entries::delete_by_id` (route `DELETE /journal-entries/{id}`) | `true` | aucun : le gel refuse avant (AC 3) |
| `invoices::delete`, facture validée | `false` | **le défaut fermé** |
| tests de la 25-2-c (`journal_entries.rs` `mod tests`) | `false` | écritures datées du jour : hors de toute borne |

### References

- [Source: crates/kesh-db/src/repositories/journal_entries.rs] — `create_in_tx_inner` étape 0-bis
  et la garde `PeriodLocked` ; `delete_in_tx` et ses étapes 2, 3, 3-bis, 3-ter.
- [Source: crates/kesh-db/src/repositories/invoices.rs:1350] — l'appel `delete_in_tx(…, false)`.
- [Source: crates/kesh-api/tests/period_lock_e2e.rs] — les tests du verrou de la 24-4c, et leurs
  helpers de pose et de levée.
- [Source: _bmad-output/implementation-artifacts/25-2-b-devalidation-facture.md] — la story d'où
  celle-ci est détachée (AC 3, motif 6).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Étape | Note |
|---|---|---|
| 2026-09-19 | spec | Story détachée de la 25-2-b, sur arbitrage de Guy. Le trou a été trouvé par l'orchestrateur en vérifiant les findings de la passe 1 de validation de la 25-2-b — **aucune lentille ne l'avait vu**. Issue **#443** ouverte le même jour (`bug_report`, labels `bug` + `triage`). |
