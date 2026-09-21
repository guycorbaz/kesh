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

**Dans la 25-2-b-2** : le retrait de la branche `validated` d'`invoices::delete` et les **huit**
tests qui en dépendent, l'écran, les E2E des deux cycles, les manuels, le `CHANGELOG`, et `closes #440`.

⚠️ **Entre les deux stories, deux chemins détruisent une écriture de facture** : la suppression
directe (#219) et la dévalidation. État intermédiaire **assumé et borné**, et il faut dire ce qu'il
est : la suppression directe ne garde que **trois** des huit motifs (`paid_at`, créditée, rappels),
plus ceux de `delete_in_tx` — elle ignore le règlement **partiel**, `emailed_at` et le
rapprochement. ⚠️ **Ce n'est pas une régression de cette story : c'est l'état de `main` depuis
#219**, et la 25-2-b-2 retire ce chemin. La 25-2-c a par ailleurs fermé la réattribution des
numéros d'écriture.

## Acceptance Criteria

1. **La transition existe.** `POST /api/v1/invoices/{id}/unvalidate`, montée dans
   **`comptable_routes`** (`crates/kesh-api/src/lib.rs`) au même endroit que
   `POST /api/v1/invoices/{id}/validate` : **Administrateur et Comptable**, arbitrage de Guy du
   2026-09-19. ⚠️ **« Comme validate » ne porte que sur le MONTAGE, pas sur le contrat** :
   `validate_invoice_handler` (`routes/invoices.rs:793-797`) ne prend **aucun corps**, alors que la
   dévalidation en prend un.
   **Contrat** : corps `{ "version": n }` — convention du dépôt (`routes/invoices.rs:117`,
   `routes/journal_entries.rs:111` pour la contre-passation) —, réponse `Json<InvoiceResponse>`
   comme la validation.
   Elle repasse la facture en `draft`, met `journal_entry_id` à `NULL`, **puis** supprime
   l'écriture — dans cet ordre, la FK `journal_entry_id … ON DELETE RESTRICT` l'imposant.
   ⛔ **Un seul `UPDATE`** posant `status`, `journal_entry_id = NULL` et `version + 1` : deux
   `UPDATE` **dans l'ordre « `NULL` d'abord »** violeraient `chk_invoices_validated_has_je`
   (`status <> 'validated' OR journal_entry_id IS NOT NULL`) dès le premier. ⚠️ Dans l'ordre
   inverse ils ne la violeraient pas — la prescription tient pour l'atomicité et le `version + 1`,
   pas seulement pour la contrainte.

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

5. **Un brouillon numéroté ne change pas d'exercice.** `PUT /api/v1/invoices/{id}` refuse, sous
   `INVOICE_NUMBER_FISCAL_YEAR_MISMATCH` (`409`), une date **dont l'exercice couvrant diffère de
   celui qui couvre la date actuellement stockée**.
   ⛔ **Le mécanisme, et pourquoi il est correct** : `invoices` n'a pas de `fiscal_year_id`, et un
   gabarit de numéro peut n'avoir aucun marqueur d'année (`{SEQ}` seul suffit,
   `kesh-core/src/invoice_format.rs`) — l'exercice d'origine **n'est pas reconstructible depuis le
   numéro**. On compare donc **deux exercices couvrants** : celui de la date stockée et celui de la
   date proposée. C'est un invariant inductif : tant que la garde tient, l'exercice qui couvre la
   date d'une facture numérotée **est** celui qui a émis son numéro.
   **Tests** : dévalider, redater dans l'exercice **suivant** → refus ; redater dans le **même**
   exercice → accepté. Tous deux atteignables par les chemins applicatifs (valider → dévalider →
   `PUT`).
   ⚠️ **La revalidation ne porte PAS « la même garde »** (passe 3 de la mère) : `validate_invoice`
   garde ce qu'elle garde déjà — un exercice **ouvert** couvrant la date. Une fois le `PUT` gardé,
   l'état « brouillon numéroté dont la date a changé d'exercice » devient **inatteignable**, et
   c'est **ce test-là**, et lui seul, qui se construit en base à la main.
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
     `ReversalBlocker` (`crates/kesh-db/src/errors.rs:57-106` — l'enum, puis son `impl` et son
     `code()`), un `code()` par motif, `409` pour les codes neufs ; les motifs 5, 6 et 8 gardent
     leurs variantes et statuts existants (`FISCAL_YEAR_CLOSED` 400, `PERIOD_LOCKED` 400,
     `ENTRY_IS_REVERSED` 409).
   - **Les codes**, que le renvoi à la mère ne couvre pas : `INVOICE_HAS_SETTLEMENTS` (motif 1 —
     il couvre aussi `paid_at IS NOT NULL`), `INVOICE_CREDITED`, `INVOICE_HAS_REMINDERS`,
     `INVOICE_EMAILED`, et — motif 7 — ⛔ **`MATCHED_BANK_TRANSACTION`, le code canonique
     EXISTANT** (`crates/kesh-db/src/errors.rs:102`), déjà exposé au frontend et asserté en E2E :
     le même état ne reçoit pas un second nom. *(La mère écrivait `ENTRY_IS_RECONCILED` ; corrigé en
     passe 1 de cette fille.)*
   - **Le test de précédence** pose deux empêchements de part et d'autre de l'appel — p. ex.
     **rappel** (avant) **et exercice clos** (dans `delete_in_tx`) — et vérifie que le premier est
     rendu.

7. **`emailed_at` : refus sec, non levable** — arbitrage de Guy du 2026-09-16. ⚠️ **Deux limites
   s'écrivent, elles ne se comblent pas** : la garde n'attrape que ce que Kesh **sait** avoir envoyé
   (un PDF téléchargé puis transmis à la main ne laisse aucune trace) ; et pendant l'expédition d'un
   e-mail, `emailed_at` n'étant posé qu'après l'envoi, une dévalidation concurrente peut encore
   passer. Le renversement (« marquer avant d'expédier ») a été **écarté en passe 3** : cf. fiche
   mère, AC 3.
   ⛔ **Et la moitié qui coûte, à écrire aussi** : `mark_emailed` n'a **volontairement** aucune
   condition de statut (`invoices.rs`, « review 20-3b1 Pass 1 ECH-1 » — si le statut bascule pendant
   l'envoi, le fait doit être tracé), et aucun CHECK ne l'interdit. Dans cette fenêtre, la facture
   devient donc un **brouillon portant `emailed_at`** : librement modifiable alors que le client
   détient le document, et — une fois **revalidé** — **définitivement indévalidable**, le motif 4
   n'étant pas levable. Une impasse qu'on atteint par accident vaut d'être écrite ; la sortie est
   l'avoir.

8. **L'audit nomme le geste.** `invoice.unvalidated`, avec un instantané de l'état avant (numéro,
   total, écriture supprimée), en plus du `journal_entry.deleted` que `delete_in_tx` journalise.
   **Trois registres bougent** :
   - `crates/kesh-api/src/audit_labels.rs` — le code entre dans `ACTIONS`, et
     `audit-log-action-invoice-unvalidated` dans les **quatre** locales ; ⚠️ si le code d'action
     n'est pas un littéral au site d'insertion, **compléter** l'entrée existante de
     `repositories/invoices.rs` dans `SITES_INDIRECTS`, ne pas en créer une seconde ;
   - `crates/kesh-api/tests/audit_route_registry.rs` — totaux en dur **105 → 106** et **108 → 109**
     (`:448`, `:458`), mentions en prose (`:20`, `:24-25`, `:246`), **et la partition
     `traced` 87 → 88** (`:450`) — ⚠️ **avec le message qui la ventile** : il dit aujourd'hui
     « 73 tracées avant la story, plus ses 14 », et 73 + 14 ne feraient plus 88. *Un total doit
     rester cohérent avec sa propre ventilation.*
   ⚠️ **Le compteur « 123 libellés » du `CHANGELOG` n'est PAS bumpé ici** : cette story ne touche
   pas le `CHANGELOG`. C'est bien elle qui rend le chiffre faux, et c'est la **25-2-b-2** qui le
   porte (son AC 10), avec l'entrée utilisateur du cycle complet. Le chiffre vit dans la section
   « Non publié », donc rien n'est publié faux entre les deux merges.

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
- [ ] **T3-bis — Les trois doc-comments qui deviennent faux DÈS cette story** : `unvalidate` est le
      **second** appelant à passer `enforce_immutability = false`, or `journal_entries.rs:943` et
      `:982` disent « seul appelant », et `invoices.rs:1339-1340` « seul site du dépôt » — la phrase court sur deux lignes. ⚠️ **Aucun test
      ne les garde** — et `journal_entries.rs:945-946` rappelle lui-même qu'« un doc-comment périmé
      a déjà égaré une journée entière de revue ». Ils se réécrivent **ici**, pas en b-2 : c'est le
      code de cette story qui les périme. *(Relevé en passe 1 ; la b-2 ne garde que la réécriture
      liée au retrait de la branche.)*
- [ ] **T4 — Tests et mutations** (AC 11).
- [ ] **T5 — Gates complets** (AC 12) et PR en `refs #440`.

## Dev Notes

### Ce que cette story ne fait pas

- Elle ne retire **pas** la branche `validated` d'`invoices::delete` : c'est la 25-2-b-2, avec les
  **huit** tests qui en dépendent — 5 dans `invoices.rs`, 2 dans `invoice_delete_e2e.rs` (dont
  `delete_validated_in_locked_period_returns_400_period_locked`, ajouté par la 25-2-b-zero), 1
  Playwright. ⚠️ La fiche mère en annonçait cinq : recompté en passe 1 de validation de la b-2.
- Elle ne touche **ni l'écran, ni les manuels, ni le `CHANGELOG`**.
- Elle ne ferme pas #440, et **ne rouvre pas** #219.

### Règle de splitting

Modules touchés : `kesh-db`, `kesh-api`, `kesh-i18n`, `docs/` — **quatre**, sous le seuil de cinq.
Née d'une non-convergence, cette fille l'écrit plutôt que de le laisser déduire. ⚠️ Le signal à
surveiller en passe 2 n'est pas la sévérité en soi, mais sa **nature** : un `HIGH` de découpage est
attendu et se corrige ; un `HIGH` de **conception** relancerait le critère.

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
| 2026-09-21 | validate P1 | **Passe 1, une lentille Opus**, prompt versionné `25-2-b-1-validate-prompt-p1.md`, **sept axes exercés**. **2 HIGH, 6 MEDIUM, 5 LOW** — ⛔ **tous des défauts du DÉCOUPAGE, aucun de la conception** : c'est précisément ce qu'une fille doit se faire dire. HIGH 1 — l'AC 5 prescrivait une garde sur « l'exercice du numéro », que la phrase suivante déclarait non reconstructible : en comprimant, j'avais supprimé les deux tests de la mère, seul indice du mécanisme. Rétabli, et le mécanisme écrit : **comparer les deux exercices couvrants**, invariant inductif. HIGH 2 — le compteur « 123 libellés » du `CHANGELOG` : cette story le rend faux et s'interdit d'y toucher, la sœur ne le reprenait pas ; **transféré nommément à l'AC 10 de la b-2**. MEDIUM : `unvalidate` devient le **second** appelant à `false` et périme trois doc-comments **dès ce merge** (T3-bis, ramenés ici) ; la partition `traced` laissait sa ventilation « 73 + 14 » ; « les deux portent leurs gardes » était **faux** de la suppression directe, qui n'en garde que trois sur huit ; le contrat HTTP n'était pas spécifié et « comme validate » le contredisait (`validate_invoice_handler` ne prend **aucun** corps) ; ⛔ `ENTRY_IS_RECONCILED` **doublait** un code canonique publié — remplacé par `MATCHED_BANK_TRANSACTION` ; la fenêtre de l'envoi laisse un brouillon porteur d'`emailed_at`, **indévalidable à jamais une fois revalidé**. LOW : ordre imposé par la FK rétabli, justification du `UPDATE` unique qualifiée, les cinq codes nommés, `errors.rs:57-106`, § *Règle de splitting*. Vérifié exact et à ne pas refaire : compteurs de routes (105/108 complets), « 123 libellés » (28+93+2), clé i18n, `SITES_INDIRECTS`, le passage des clés `read-write` par `comptable_routes`, la précédence du code mergé, l'absence de site confondant « numéroté » et « validé ». |
| 2026-09-21 | validate P2 — **close** | **Passe 2, une lentille Sonnet** (P1 était Opus), prompt versionné `25-2-b-1-validate-prompt-p2.md`, **sept axes exercés, aucun laissé**. **0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW** — et le LOW est de nature **découpage** : la citation `invoices.rs:1339`, resserrée par ma tâche T3-bis là où la mère citait une plage, coupait une phrase qui court sur deux lignes. Corrigée. ⛔ **Aucun finding de CONCEPTION** — c'est le signal qui comptait : la sévérité `HIGH → LOW` confirme que le découpage était la bonne remédiation, et non un report du problème. Vérifié depuis la source et **à ne pas refaire** : le contrat de `validate_invoice_handler` (aucun corps), la convention `version` du dépôt, `comptable_routes` sans garde anti-clé, `errors.rs:57-106` et `MATCHED_BANK_TRANSACTION` à `:102`, la précédence réelle `5, 8, 6` du code mergé, les compteurs du registre de routes, `SITES_INDIRECTS`, les quatre locales, et — point neuf — **`find_covering_date` existe sans filtre de statut**, ce qui rend le mécanisme de l'AC 5 réalisable sans lecture dupliquée. Axe 2 tranché : **aucun test existant ne rougit au merge de b-1 seule**, l'état « brouillon numéroté » n'existant pas encore sur `main`. |
