# Story 25.2-b-zero : Le verrou de période protège aussi de la suppression

Status: review

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

- [x] **T1 — La garde** (AC 1-5) : étendre la requête de l'étape 2 à `entry_date` ; **après**
      l'étape 3-ter, lire la borne par un `SELECT` **non verrouillant** puis refuser. Lue là, elle ne
      prend aucun verrou sur `companies` : aucun cycle possible avec `lock_books` / `unlock_books`,
      qui ne verrouillent que `companies`. ⚠️ Le champ `attempted` de `PeriodLocked` porte ici la
      date d'une écriture **existante**, non une date qu'on a tenté de poser : les messages des
      quatre locales (« celle-ci est datée du… ») restent justes, et le nom du champ ne change pas.
- [x] **T2 — Tests et mutations** (AC 6).
- [x] **T3 — Manuels, PDF, CHANGELOG** (AC 7).
- [x] **T4 — Gates complets** — gate backend **complet** et non ciblé : la story touche un
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

Claude Opus 5 (1M context).

### Debug Log References

### Completion Notes List

### File List

| Fichier | État |
|---|---|
| `crates/kesh-db/src/repositories/journal_entries.rs` | modifié — étape 2 étendue à `entry_date`, étape 3-quater (la garde), doc-comment de `delete_in_tx` ; **4 tests** neufs et deux helpers |
| `crates/kesh-api/tests/invoice_delete_e2e.rs` | modifié — **1 test** neuf, helper `create_validated_invoice_on` (l'existant le réutilise) |
| `docs/manual/fr/user-manual.tex` + `.pdf` | modifiés — la suppression refusée en période verrouillée |
| `docs/manual/fr/admin-manual.tex` + `.pdf` | modifiés — idem |
| `CHANGELOG.md` | modifié — « Corrigé » |
| `README.md` | modifié — ligne E25 |

## Change Log

| Date | Étape | Note |
|---|---|---|
| 2026-09-19 | spec | Story détachée de la 25-2-b, sur arbitrage de Guy. Le trou a été trouvé par l'orchestrateur en vérifiant les findings de la passe 1 de validation de la 25-2-b — **aucune lentille ne l'avait vu**. Issue **#443** ouverte le même jour (`bug_report`, labels `bug` + `triage`). |
| 2026-09-19 | validate P1 — **close** | **Une lentille Sonnet**, prompt versionné `25-2-b-zero-validate-prompt-p1.md`. **0 au-dessus de LOW, 2 LOW**, sept axes déclarés exercés. L'inventaire des autres chemins qui effaceraient une écriture est **complet** : pas de suppression d'avoir, `supplier_invoices::cancel` contre-passe, aucune FK en `CASCADE` vers les écritures, aucune `UPDATE` de production sur leurs dates ou montants ; purges de société hors périmètre à bon droit. LOW corrigés : position et nature de la lecture de la borne fixées à T1 ; sens du champ `attempted` écrit. Relevé hors périmètre, versé à la 25-2-b : `admin-manual.tex:1799` et `README.md:218` affirment déjà faux qu'une écriture n'est « ni modifiable ni supprimable, sans exception ». |
| 2026-09-19 | dev T1→T4 | **La garde** : étape 3-quater de `delete_in_tx`, **après** le gel, lecture non verrouillante de la borne, seuil inclusif, `entry_date` ramenée par la requête de verrouillage de l'étape 2 (aucune seconde lecture). **5 tests neufs** (périmètre : `main` → ce commit) — 4 de dépôt, 1 de bout en bout. ⚠️ **Écart à l'AC 6, assumé** : la borne est posée en **SQL direct**, non par `companies::lock_books`, qui refuse de *reculer* une borne — un test ne pourrait pas poser la sienne derrière celle d'un run précédent ; elle est retirée avant l'assertion, et le gate l'a confirmé (`books_locked_through` à `NULL` après le run). **Quatre mutations, toutes tuées sur assertion** : garde neutralisée → `…du_jour_de_la_borne` rouge (dépôt) et `…locked_period_returns_400…` rouge (`left: 204, right: 400`) ; `<=` en `<` → `…du_jour_de_la_borne` rouge ; garde déplacée avant le gel → `le_gel_parle_avant_le_verrou_de_periode` rouge ; fichier restauré à l'identique après chacune (`cmp`). **Gate backend complet : 2401/2401**, 4 ignorés, base remise à zéro et contrôlée ; `fmt`, `clippy -D warnings` verts. **E2E : 213 passés, 10 échecs** — les 7 de la KF-029, et **3 de pollution** : `product-revenue-account:133` et `products:185` passent rejoués seuls, `sidebar-navigation:75` échoue encore seul sur la base polluée mais **passe deux fois sur une base `kesh_e2e` reconstruite**. ⚠️ Trois échecs de pollution, un de plus que la fourchette connue (1 à 2). Frontend non rejoué : identique à `main` (`git diff --quiet main -- frontend`). Manuels : PDF régénérés, phrases neuves contrôlées à plat dans les deux. |
| 2026-09-19 | revue P1 | **Passe 1, une lentille Sonnet**, prompt versionné `25-2-b-zero-review-prompt-p1.md`, cinq axes exercés. **1 HIGH, 2 MEDIUM, 1 LOW**, tous retenus après vérification. **HIGH — les tests datés « il y a trente jours » paniquaient en janvier** : `setup` ne garantit qu'un exercice couvrant aujourd'hui ; si l'exercice 2020-2030 du seed a été clos par un test, `ensure_open_fiscal_year` recrée l'année civile, et la date tombe hors exercice. ⚠️ Conditionnel sur la base de dev, mais atteignable. Corrigé **par construction** : l'écriture est datée du jour, la borne — posée en SQL, donc libre d'être dans le présent — vaut le jour même ou la veille. **MEDIUM** — un `unwrap` entre la pose et le retrait de la borne pouvait la laisser posée : erreurs désormais recueillies, borne retirée, puis levées ; et `setup` **retire toute borne** d'office (auto-réparation, comme l'exercice ouvert de #140). **MEDIUM** — la liste des refus de `sec:suppression-facture` ignorait la période verrouillée : quatrième entrée ajoutée, PDF régénéré et contrôlé à plat. **LOW** — la liste des étapes de `delete_by_id` sautait 3-bis, 3-ter et la neuve. ⚠️ La même liste omettait déjà les **rappels** (#260) : laissé à la 25-2-b, qui refond la section. **Mutations rejouées** sur les tests réécrits : trois, toutes tuées. **Gate complet : 2401/2401**, base remise à zéro. La remédiation ne touche **aucune ligne de code de production** — `mod tests`, un doc-comment, le manuel. |
