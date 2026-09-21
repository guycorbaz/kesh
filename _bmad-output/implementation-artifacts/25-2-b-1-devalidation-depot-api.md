# Story 25.2-b-1 : Dévalider une facture — le dépôt et l'API

Status: ready-for-dev

**Issue : [#440]**, qu'elle **ne ferme pas** : la 25-2-b-2 le fera. Commits en `refs #440`.

⛔ **Issue du DÉCOUPAGE de la 25-2-b**, arbitré par Guy le 2026-09-21 sur le critère de
non-convergence de la § *Règle de splitting préventif* : trois passes de validation, sévérité
maximale `CRITICAL → HIGH → HIGH`. **La fiche mère `25-2-b-devalidation-facture.md` reste la source
des faits établis** — les trois passes y sont consignées, et rien de ce qui suit ne les réécrit.

**Prérequis mergés** : 25-2-c (#442, le compteur d'écritures), 25-1c-a (#439, le registre
`audit_labels.rs`), **25-2-b-zero (#443, PR #444, `b7c65be8`)** — la garde du verrou de période dans
`delete_in_tx`, sur laquelle l'AC 3 s'appuie sans la réécrire.

## Story

En tant que **personne qui facture**,
je veux **repasser une facture validée en brouillon par l'API**, sous des empêchements explicites,
afin de **réparer une erreur de facturation sans émettre un avoir** qui ne correspond à aucune
réalité commerciale.

## Périmètre

**Dans cette story** : le dépôt, la route, les empêchements, l'audit, les messages traduits, la
documentation de l'API externe.

**Dans la 25-2-b-2** : le retrait de la branche `validated` d'`invoices::delete` et les six tests
qui en dépendent, l'écran, les E2E des deux cycles, les manuels, le `CHANGELOG`, et `closes #440`.

⚠️ **Entre les deux stories, deux chemins détruisent une écriture de facture** : la suppression
directe (#219) et la dévalidation. C'est un état intermédiaire **assumé et borné** — les deux
portent leurs gardes, et la 25-2-c a fermé la réattribution des numéros d'écriture. La 25-2-b-2 le
referme.

## Acceptance Criteria

1. **La transition existe.** `POST /api/v1/invoices/{id}/unvalidate`, montée dans
   **`comptable_routes`** (`crates/kesh-api/src/lib.rs`), comme
   `POST /api/v1/invoices/{id}/validate` : **Administrateur et Comptable**, arbitrage de Guy du
   2026-09-19. Elle repasse la facture en `draft`, met `journal_entry_id` à `NULL` et supprime
   l'écriture. ⛔ **Un seul `UPDATE`** posant `status`, `journal_entry_id = NULL` et
   `version + 1` : deux `UPDATE` violeraient `chk_invoices_validated_has_je` dès le premier.

2. **Les clés API sont admises**, comme pour la validation — arbitrage de Guy : « même approche que
   Bexio », dont l'API publique porte `POST /2.0/kb_invoice/{id}/revert_issue`. ⚠️ C'est un
   **élargissement** : jusqu'ici aucune clé ne pouvait détruire l'écriture d'une facture. Tests :
   une clé `read-write` dévalide, une clé `read` reçoit le refus d'écriture habituel.

3. **Le verrou optimiste s'applique.** La dévalidation prend `version` et rend `409` en cas de
   conflit.

4. **Le numéro de facture est CONSERVÉ**, et la revalidation le reprend **sans tirer du compteur** :
   `validate_invoice` ne consulte `invoice_number_sequences` que si `invoice_number` est absent.
   Sans cela, chaque cycle brûlerait un numéro et creuserait un trou dans la séquence des factures.
   Test décisif : valider → dévalider → revalider → **même numéro**, compteur inchangé.

5. **Un brouillon numéroté ne change pas d'exercice.** `PUT /api/v1/invoices/{id}` refuse une date
   hors de l'exercice du numéro conservé — `INVOICE_NUMBER_FISCAL_YEAR_MISMATCH`, `409`. ⚠️ **La
   revalidation ne porte PAS « la même garde »** (passe 3) : `invoices` n'a pas de `fiscal_year_id`,
   et un gabarit de numéro peut n'avoir aucun marqueur d'année (`{SEQ}` seul suffit,
   `kesh-core/src/invoice_format.rs`) — l'exercice d'origine n'est pas reconstructible.
   `validate_invoice` garde ce qu'elle garde déjà. Une fois le `PUT` gardé, l'état est
   **inatteignable par les chemins applicatifs** : son test se construit en base, à la main.
   ⚠️ Un brouillon **sans** numéro n'est pas concerné.

6. **Les huit empêchements, un code distinct par motif** — tableau, justifications et précédence :
   **fiche mère, AC 3**, à appliquer tel quel. En résumé :
   - dans `unvalidate`, avant l'appel à `delete_in_tx` : réglée (lire `invoice_settlements` **OU**
     `paid_at`), créditée, rappels, envoyée, rapprochée ;
   - dans `delete_in_tx`, déjà en place : exercice clos, contre-passée, **période verrouillée**
     (#443) ;
   - précédence réelle **1, 2, 3, 4, 7, puis 5, 8, 6** ; un test pose deux empêchements, un de
     chaque côté de l'appel, et vérifie que le premier est rendu.
   - ⛔ **Pas de `IllegalStateTransition`** : un enum de **cinq** variantes sur le modèle de
     `ReversalBlocker` (`crates/kesh-db/src/errors.rs:57-104`), un `code()` par motif, `409` pour
     les cinq codes neufs ; les motifs 5, 6 et 8 gardent leurs variantes et statuts existants.

7. **`emailed_at` : refus sec, non levable** — arbitrage de Guy du 2026-09-16. ⚠️ **Deux limites
   s'écrivent, elles ne se comblent pas** : la garde n'attrape que ce que Kesh **sait** avoir envoyé
   (un PDF téléchargé puis transmis à la main ne laisse aucune trace) ; et pendant l'expédition d'un
   e-mail, `emailed_at` n'étant posé qu'après l'envoi, une dévalidation concurrente peut encore
   passer. Le renversement (« marquer avant d'expédier ») a été **écarté en passe 3** : cf. fiche
   mère, AC 3.

8. **L'audit nomme le geste.** `invoice.unvalidated`, avec un instantané de l'état avant (numéro,
   total, écriture supprimée), en plus du `journal_entry.deleted` que `delete_in_tx` journalise.
   **Trois registres bougent** :
   - `crates/kesh-api/src/audit_labels.rs` — le code entre dans `ACTIONS`, et
     `audit-log-action-invoice-unvalidated` dans les **quatre** locales ; ⚠️ si le code d'action
     n'est pas un littéral au site d'insertion, **compléter** l'entrée existante de
     `repositories/invoices.rs` dans `SITES_INDIRECTS`, ne pas en créer une seconde ;
   - `crates/kesh-api/tests/audit_route_registry.rs` — totaux en dur **105 → 106** et **108 → 109**
     (`:448`, `:458`), mentions en prose (`:20`, `:24-25`, `:246`), **et la partition
     `traced` 87 → 88** (relevé en passe 3) ;
   - le total « 123 libellés » du `CHANGELOG` (section « Non publié ») devient **124**.

9. **Les messages sont traduits** dans les quatre locales, convention `error-*`.

10. **`docs/api-external.md`** porte la route neuve, ses codes d'erreur et leurs statuts, et indique
    qu'elle est **ouverte aux clés API**. ⚠️ Sa note ² (l. 217) décrit `DELETE /invoices/{id}` comme
    une « suppression définitive » : elle sera rectifiée par la **25-2-b-2**, qui retire cette
    branche — ne pas l'anticiper ici.

11. **Tests** : un par empêchement (dont le motif 6 **par la dévalidation**), la précédence, le
    chemin nominal, le conflit de version, les deux cas de clé API, le test décisif du numéro, la
    garde d'exercice du `PUT`. ⛔ Chaque garde **prouvée par mutation**, et chaque mutation vue
    rouge **sur assertion**, jamais sur compilation.

12. **Gate complet** — la story touche `kesh-db` : gate backend complet, non ciblé. E2E complète
    avant le push.

## Tasks / Subtasks

- [ ] **T1 — Dépôt : `invoices::unvalidate`** (AC 1, 3, 6, 7) — transaction unique : verrou
      `FOR UPDATE` de la facture, empêchements 1-4 et 7, un seul `UPDATE`, `delete_in_tx`, audit.
- [ ] **T2 — Dépôt : le numéro** (AC 4, 5) — `validate_invoice` ne tire du compteur que si le numéro
      manque ; garde d'exercice du `PUT`.
- [ ] **T3 — API, i18n, registres** (AC 1, 2, 8, 9, 10).
- [ ] **T4 — Tests et mutations** (AC 11).
- [ ] **T5 — Gates complets** (AC 12) et PR en `refs #440`.

## Dev Notes

### Ce que cette story ne fait pas

- Elle ne retire **pas** la branche `validated` d'`invoices::delete` : c'est la 25-2-b-2, avec les
  **six** tests qui en dépendent (cf. fiche mère, AC 5 — dont
  `delete_validated_in_locked_period_returns_400_period_locked`, ajouté par la 25-2-b-zero).
- Elle ne touche **ni l'écran, ni les manuels, ni le `CHANGELOG`**.
- Elle ne ferme pas #440, et **ne rouvre pas** #219.

### References

- `_bmad-output/implementation-artifacts/25-2-b-devalidation-facture.md` — la fiche mère, ses trois
  passes de validation et leurs prompts versionnés (`25-2-b-validate-prompt-p1|p2|p3.md`).
- [Source: crates/kesh-db/src/repositories/invoices.rs] — `validate_invoice`, `delete`, `mark_emailed`.
- [Source: crates/kesh-db/src/repositories/journal_entries.rs] — `delete_in_tx` et ses gardes 3 à 3-quater.
- [Source: crates/kesh-db/src/errors.rs:57-104] — `ReversalBlocker`, le motif à suivre.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Étape | Note |
|---|---|---|
| 2026-09-21 | spec | Story née du **découpage** de la 25-2-b, arbitré par Guy après une sévérité `HIGH → HIGH` entre les passes 2 et 3. Elle hérite des trois passes de validation de la fiche mère : rien n'y est réécrit, tout y est **référencé**. |
