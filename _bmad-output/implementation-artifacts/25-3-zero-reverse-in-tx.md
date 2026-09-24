# Story 25.3-zero : `reverse_in_tx` — la contre-passation composable

Status: review

**Issues : aucune qu'elle ferme.** Elle est le **socle** de la 25-3-a ([#414]) et de la 25-3-b
([#418]), qui ne peuvent pas être écrites sans elle. Commits en `refs #414`.

⛔ **Issue du DÉCOUPAGE de la 25-3**, arbitré par Guy le 2026-09-24 sur le critère que la fiche
mère annonçait elle-même : *« si le geste commun ne se laisse pas écrire une seule fois pour les
trois appelants, sortir le geste en story-zéro »*. Il ne s'est pas laissé écrire — cf.
`25-3-annuler-reglement-et-rapprochement.md`, passé en `split`, qui reste la **source des faits**.

## Le fait qui l'impose

`journal_entries::reverse` **ouvre et commite sa propre transaction** :

```rust
pub async fn reverse(pool: &MySqlPool, company_id: i64, id: i64, user_id: i64)
    -> Result<JournalEntryWithLines, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;
    …
```

Or annuler un règlement suppose, **dans une seule transaction** : contre-passer l'écriture,
défaire l'état dérivé (ligne de règlement côté client, colonnes de statut côté fournisseur, lien de
rapprochement côté banque) et journaliser. **Avec le contrat actuel, c'est impossible** : la
contre-passation serait commitée avant le reste, et un échec ultérieur laisserait une écriture
inverse sans son effet — *une facture qui se dit impayée alors que le grand livre dit le contraire.*

⚠️ **Et l'alternative est pire.** `supplier_invoices::cancel` (`supplier_invoices.rs:754-860`)
contourne déjà le problème en **réécrivant la contre-passation à la main** : elle relit les lignes,
inverse débit et crédit, et appelle `create_in_tx` — qui passe toujours `None`. ⛔ **Elle ne pose
donc JAMAIS `reverses_entry_id` et ne consulte JAMAIS `reversal_blocker`.** C'est une seconde
implémentation parallèle, et la 25-3 allait en écrire une troisième.

*Le refactor n'est pas un confort : c'est ce qui empêche la troisième.*

## Acceptance Criteria

1. **`reverse_in_tx` existe et porte toute la logique.** Signature sur le modèle de
   `create_in_tx` / `create_in_tx_inner` du même fichier : elle prend `&mut Transaction`, ne fait
   **ni `begin` ni `commit`**, et ne rollback pas — l'appelant, propriétaire de la transaction, en
   est responsable (le drop de `Transaction` déclenche le rollback de `sqlx`).

2. **`reverse` devient un mince enveloppement** : `begin`, appel, `commit`. ⛔ **Aucune logique n'y
   subsiste** — sinon les appelants transactionnels ne l'auraient pas.

3. ⛔ **ZÉRO changement de comportement.** C'est un refactor pur : mêmes empêchements dans le même
   ordre, même verrou, même date d'exercice, même audit, mêmes erreurs. *Une story-zéro qui change
   le comportement fait porter à la suivante des défauts qu'elle n'a pas commis.*

4. **La route existante ne change pas.** `routes/journal_entries.rs:460` est le **seul** appelant
   de `reverse` — vérifié. Il continue d'appeler le wrapper.

5. **Les tests existants passent sans être modifiés**, et c'est le critère qui prouve l'AC 3.
   ⚠️ **Si un test doit être retouché, c'est que le comportement a bougé** : le signaler plutôt que
   d'ajuster le test.

6. **Un test neuf prouve la composabilité** : dans une transaction fournie, appeler `reverse_in_tx`
   **puis échouer délibérément**, et vérifier qu'**aucune** écriture inverse ne subsiste après le
   rollback. ⛔ *C'est la seule propriété que ce refactor apporte, donc la seule qui doit être
   prouvée.* À défaut, la story ne serait qu'un déplacement de lignes.

7. **Le doc-comment dit POURQUOI la fonction est scindée**, et nomme `supplier_invoices::cancel`
   comme le précédent à ne pas imiter — sans quoi le prochain développeur réécrira une quatrième
   contre-passation.

8. **Gate complet** — la story touche `kesh-db`.

## Tasks / Subtasks

- [x] **T1 — Extraire `reverse_in_tx`** (AC 1, 2, 3), wrapper compris.
- [x] **T2 — Le test de composabilité** (AC 6) et les doc-comments (AC 7).
- [x] **T3 — Gate complet** (AC 8) et PR en `refs #414`.

## Dev Notes

### Ce que cette story ne fait pas

- Elle **n'annule rien** : ni règlement, ni rapprochement. C'est la 25-3-a et la 25-3-b.
- Elle ne touche **pas** `supplier_invoices::cancel`, dont la contre-passation manuelle est un
  défaut **antérieur** : le corriger revient à la 25-3-a, qui rouvre cette fonction pour l'AC 5 de
  la fiche mère. ⚠️ **Le signaler ici pour que la 25-3-a ne le découvre pas.**

### Ce qui rend ce refactor délicat

`reverse` enchaîne, dans sa transaction : un verrou `FOR UPDATE` sur l'origine, le recensement des
empêchements (`reversal_blocker`, **huit** variantes), une garde sur les comptes archivés, la
recherche d'un exercice **ouvert** couvrant la date, la lecture ordonnée des lignes, la création de
l'écriture inverse avec `reverses_entry_id`, et l'audit. ⛔ **L'ordre de ces étapes est le contrat**,
et l'AC 3 interdit d'y toucher.

### References

- [Source: crates/kesh-db/src/repositories/journal_entries.rs:1395] — `reverse`, à scinder ;
  `create_in_tx` / `create_in_tx_inner` du même fichier donnent le patron exact.
- [Source: crates/kesh-db/src/repositories/supplier_invoices.rs:754] — `cancel`, la contre-passation
  écrite à la main : **anti-exemple**.
- [Source: crates/kesh-db/src/errors.rs] — `ReversalBlocker`, **huit** variantes.

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context) — implémentation.

### Debug Log References

- L'extraction a demandé trois corrections mécaniques, toutes dues au passage
  d'une `Transaction` possédée à une `&mut Transaction` : `&mut *tx` devient
  `&mut **tx` (3 sites), et `&mut tx` devient `tx` là où la transaction est
  passée telle quelle (4 sites).

### Completion Notes List

- ⛔ **L'AC 3 est MESURÉ, non affirmé** : `git diff --stat` sur
  `crates/kesh-api/tests/` et `crates/kesh-db/tests/` est **vide**, et les 27
  tests de `journal_entry_reversal_e2e` passent sans retouche. *Un refactor qui
  se dit « sans changement de comportement » et qui ajuste ses tests ne prouve
  rien.*
- ⛔ **Le test de composabilité TRANCHE, et c'est prouvé** : en simulant
  l'ancien contrat — un `COMMIT` au milieu de `reverse_in_tx` —, il rougit sur
  assertion en nommant la cause (« une écriture inverse a SURVÉCU au rollback de
  l'appelant »). *Sans cette épreuve, la story ne serait qu'un déplacement de
  lignes.*
- Le rollback du test est **implicite** : la transaction est droppée sans
  `commit`, exactement comme si l'étape suivante de l'appelant avait échoué —
  c'est le scénario réel, pas une simulation.

### File List

| Fichier | Nature |
|---|---|
| `crates/kesh-db/src/repositories/journal_entries.rs` | `reverse_in_tx` extraite, `reverse` réduite à un wrapper, 1 test neuf |

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Étape | Note |
|---|---|---|
| 2026-09-24 | spec | Story-zéro née du **découpage de la 25-3** (arbitrage de Guy), sur le critère que la fiche mère annonçait : le geste commun ne s'écrit pas une seule fois. ⛔ **Fait qui l'impose** : `reverse` ouvre et commite sa propre transaction, donc rien ne peut être annulé atomiquement avec elle. ⚠️ **Et le contournement existe déjà dans le dépôt** : `supplier_invoices::cancel` réécrit la contre-passation à la main, ne pose **jamais** `reverses_entry_id` et ne consulte **jamais** `reversal_blocker` — la 25-3 allait en écrire une troisième. *Le refactor n'est pas un confort : c'est ce qui empêche la troisième.* |
