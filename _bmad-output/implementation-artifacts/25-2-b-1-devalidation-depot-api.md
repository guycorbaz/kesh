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

⛔ **Et le MANUEL UTILISATEUR le dit au singulier — il devient donc faux dès ce merge.**
`docs/manual/fr/user-manual.tex:539-541` : « *une écriture ne se supprime plus, mais il subsiste
**un** chemin qui en creuse : la suppression définitive d'une facture validée* ». Dès cette story
il y en a **deux**, et le second est atteignable **par clé API**. *Un nombre écrit dans un manuel
est précisément ce qui se périme sans que rien ne rougisse.*

**Traitement, et c'est un arbitrage de coût, pas un oubli** : le site est **déjà inscrit** aux
ancrages de manuel de la 25-2-b-2 (son AC 7, l. 195), qui réécrit ce paragraphe en retirant le
chemin de suppression — la phrase y redevient vraie **au singulier**, la dévalidation restant alors
le seul chemin. Régénérer ici les trois PDF pour les réécrire au merge suivant coûte deux fois le
même geste pour une fenêtre d'un seul merge. ⚠️ **Mais la fenêtre existe**, et si l'ordre des deux
filles changeait, ou si la b-2 tardait, ce paragraphe devrait être corrigé **avant** toute release.
*(Relevé en passe 2 de revue, en reprenant à la main l'axe « manuels » qu'une lentille avait déclaré
exercé sans le montrer.)*

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

Claude Opus 5 (1M context) — implémentation.

### Debug Log References

- **Le gate a rougi sur un module que la branche ne touche pas** :
  `journal_entries::tests::deux_creations_concurrentes_n_obtiennent_pas_le_meme_numero`,
  `ForeignKeyViolation` sur `kesh.invoices`. Cause : mes tests de refus laissaient
  la facture liée à son écriture, et `cleanup_journal_entries` avale l'échec de FK
  par un `.ok()` final. C'est **KF-039** (#310). Fermé par `nettoyer_facture`, qui
  efface la facture **puis** l'écriture — la FK étant `ON DELETE RESTRICT`. Six
  factures orphelines traînaient déjà en base de gate ; base reconstruite avant le
  gate final.
- **`devalider_refuse_une_periode_verrouillee` a rougi d'abord pour rien** : la
  borne était posée sur la date de la *facture*, alors que le stub d'écriture de
  `force_validate` porte `CURDATE()`. La borne se lit désormais sur l'écriture.

### Completion Notes List

- **Quatre manques relevés par moi-même avant d'ouvrir la boucle de revue**, en
  reprenant les AC un à un plutôt qu'en faisant confiance à mon propre compte
  rendu : le test du motif **6 par la dévalidation** (AC 11), le **cas positif**
  de la garde d'exercice (AC 5 — sans lui, une garde refusant *toute* redatation
  passait), les **deux cas de clé API** (AC 2), et les **trois doc-comments**
  (T3-bis). Aucun n'aurait été vu par un gate.
- **AC 11 — mutations.** ⛔ **Ce paragraphe affirmait « sept gardes prouvées,
  les cinq empêchements » — c'était FAUX**, et la passe 1 de revue l'a établi :
  le motif 2 (créditée) n'était atteint par **aucun** test, et la moitié
  `invoice_settlements` du motif 1 non plus. Une mutation ne peut pas avoir été
  « vue rouge » sur un code que rien n'exerce. *Le compte rendu est redevenu le
  lieu du défaut, dans la story même qui prêche le recompte.*
  **État après remédiation**, chacune vue rouge **sur assertion** : les cinq
  empêchements (le règlement partiel sans `paid_at` et l'avoir ont reçu leur
  test en passe 1), le contrôle de version en tête — mutation qui ne cassait
  rien sans le test d'ordre, l'`UPDATE` final portant lui-même
  `AND version = ?` —, et la remontée du verrou de période depuis `delete_in_tx`
  (`&& false` sur la garde 3-quater).
- **Ce que la story ne fait pas**, et qui reste à la 25-2-b-2 : le retrait de la
  branche `validated` d'`invoices::delete`, l'écran, les manuels, le `CHANGELOG`
  et `closes #440`.

### File List

| Fichier | Nature |
|---|---|
| `crates/kesh-db/src/errors.rs` | `UnvalidationBlocker` (5 variantes), 3 variantes de `DbError`, leurs `code()` |
| `crates/kesh-db/src/repositories/invoices.rs` | `unvalidate`, garde d'exercice du `PUT`, `validate_invoice` (numéro conservé), **10** tests de dépôt (`#[tokio::test]` : 30 → 40 de `main` à `HEAD`), doc-comment T3-bis |
| `crates/kesh-db/src/repositories/journal_entries.rs` | deux doc-comments T3-bis |
| `crates/kesh-db/src/repositories/fiscal_years.rs` | `find_covering_date_in_tx` |
| `crates/kesh-api/src/routes/invoices.rs` | `UnvalidateInvoiceRequest`, `unvalidate_invoice_handler` |
| `crates/kesh-api/src/lib.rs` | la route, dans `comptable_routes` |
| `crates/kesh-api/src/errors.rs` | correspondance HTTP des trois variantes |
| `crates/kesh-api/src/audit_labels.rs` | `invoice.unvalidated` dans `ACTIONS` |
| `crates/kesh-api/tests/audit_route_registry.rs` | 105→106, 108→109, `traced` 87→88 et sa ventilation |
| `crates/kesh-api/tests/invoice_unvalidate_e2e.rs` | **6** tests E2E (`grep -c '#[sqlx::test'`) |
| `crates/kesh-i18n/locales/{fr,de,en,it}-CH/messages.ftl` | 8 clés × 4 locales |
| `docs/api-external.md` | la route, son corps, ses codes |

## Change Log

| Date | Étape | Note |
|---|---|---|
| 2026-09-21 | spec | Story née du **découpage** de la 25-2-b, arbitré par Guy après une sévérité `HIGH → HIGH` entre les passes 2 et 3. Elle hérite des trois passes de validation de la fiche mère : rien n'y est réécrit, tout y est **référencé**. |
| 2026-09-21 | validate P1 | **Passe 1, une lentille Opus**, prompt versionné `25-2-b-1-validate-prompt-p1.md`, **sept axes exercés**. **2 HIGH, 6 MEDIUM, 5 LOW** — ⛔ **tous des défauts du DÉCOUPAGE, aucun de la conception** : c'est précisément ce qu'une fille doit se faire dire. HIGH 1 — l'AC 5 prescrivait une garde sur « l'exercice du numéro », que la phrase suivante déclarait non reconstructible : en comprimant, j'avais supprimé les deux tests de la mère, seul indice du mécanisme. Rétabli, et le mécanisme écrit : **comparer les deux exercices couvrants**, invariant inductif. HIGH 2 — le compteur « 123 libellés » du `CHANGELOG` : cette story le rend faux et s'interdit d'y toucher, la sœur ne le reprenait pas ; **transféré nommément à l'AC 10 de la b-2**. MEDIUM : `unvalidate` devient le **second** appelant à `false` et périme trois doc-comments **dès ce merge** (T3-bis, ramenés ici) ; la partition `traced` laissait sa ventilation « 73 + 14 » ; « les deux portent leurs gardes » était **faux** de la suppression directe, qui n'en garde que trois sur huit ; le contrat HTTP n'était pas spécifié et « comme validate » le contredisait (`validate_invoice_handler` ne prend **aucun** corps) ; ⛔ `ENTRY_IS_RECONCILED` **doublait** un code canonique publié — remplacé par `MATCHED_BANK_TRANSACTION` ; la fenêtre de l'envoi laisse un brouillon porteur d'`emailed_at`, **indévalidable à jamais une fois revalidé**. LOW : ordre imposé par la FK rétabli, justification du `UPDATE` unique qualifiée, les cinq codes nommés, `errors.rs:57-106`, § *Règle de splitting*. Vérifié exact et à ne pas refaire : compteurs de routes (105/108 complets), « 123 libellés » (28+93+2), clé i18n, `SITES_INDIRECTS`, le passage des clés `read-write` par `comptable_routes`, la précédence du code mergé, l'absence de site confondant « numéroté » et « validé ». |
| 2026-09-22 | review P1 | **Passe 1, trois lentilles Sonnet** en contexte frais (l'implémentation est d'Opus 5), prompt versionné `25-2-b-1-review-prompt-p1.md`, **tous les axes déclarés exercés par les trois**. **3 HIGH, 4 MEDIUM, 3 LOW.** ⛔ **Les trois HIGH portent sur ce que la story croyait tenir, et deux sur mon propre compte rendu.** **HIGH C-1 — le motif 2 (créditée) n'avait AUCUN test**, ni dépôt ni E2E, alors que le Dev Agent Record affirmait « les cinq empêchements prouvés par mutation » : une mutation ne peut pas avoir été vue rouge sur un code que rien n'exerce. **HIGH C-2 — la moitié `invoice_settlements` du motif 1 était muette** : seul `paid_at` était posé, si bien que retirer `settlement.is_some() ||` — la moitié que le code lui-même déclare indispensable, parce qu'elle seule attrape le règlement **partiel** — ne faisait rougir rien. Les deux tests neufs tuent leur mutation **sur assertion**. **HIGH B-1 — le handler relisait la facture APRÈS le commit** : exactement l'aller-retour que la revue P3 de la Story 5.2 avait retiré de `validate_invoice_handler`, commentaire à l'appui. Conséquence atteignable : une suppression concurrente du brouillon dans la fenêtre, et l'appelant reçoit `404` sur une dévalidation **faite et auditée**. `unvalidate` rend désormais `(Invoice, Vec<InvoiceLine>)`. MEDIUM : la fuite de `bank_accounts`/`bank_imports` du helper bancaire (KF-039, base partagée) ; **aucun test HTTP n'exerçait un refus** — écrire celui-ci a révélé que `documentNumber` porte, pour le motif « envoyée », l'**adresse du destinataire** et non un numéro, désormais écrit dans `docs/api-external.md` ; le **corps** de la réponse n'était jamais inspecté ; `ILLEGAL_STATE_TRANSITION` manquait au tableau des refus. LOW : le compte de gardes « trois » laissé faux dans le doc-comment par ma propre réécriture (`unvalidate` en porte cinq) — *une conséquence de l'ancien énoncé ayant survécu à sa réécriture* ; la redondance de `devalider_refuse_une_version_perimee`, **laissée** ; le variant `InvoiceMustBeUnvalidatedFirst` sans appelant, **délibéré** (b-2). ⚠️ **Signalé, hors périmètre** : `settle_invoice_handler` (`routes/invoices.rs:1192`) porte le **même** re-fetch post-commit, sur `main` — à ouvrir en issue, non traité ici. Décomptes recomptés : 10 tests de dépôt (`#[tokio::test]` 30 → 40), 6 E2E. Gate backend complet **2417/2417** sur base reconstruite, résidu nul sur six tables contrôlées. |
| 2026-09-22 | review P2 | **Passe 2, deux lentilles** — **R** sur Opus, braquée sur le seul commit de remédiation `d13804ba` ; **D** sur Haiku, périmètre complet en diff **aplati**. Deux et non trois : la remédiation touchait du code de production, donc la boucle ne pouvait pas se clore, et l'endroit à relire était ce qu'on venait d'écrire. **0 CRITICAL, 0 HIGH, 3 MEDIUM, 2 LOW** — sévérité `3H → 0H`. ⛔ **Les trois MEDIUM sont TOUS dans ce que la remédiation a écrit sur le code**, et aucun dans le code lui-même : le motif tient une passe de plus. **F-R1 — le paragraphe sur `details` que la passe 1 venait d'ajouter à `docs/api-external.md` était faux sur trois de ses quatre affirmations** : `documentNumber` porte le numéro de **la facture elle-même** pour trois motifs sur cinq, `documentId` est `null` quand le refus vient du seul `paid_at`, et deux codes réputés « vides » émettent un `details` complet. Remplacé par un tableau motif par motif. ⚠️ **Et en l'écrivant j'ai refait la faute une quatrième fois** — « les **quatre** autres codes » suivi de **cinq** — recompté depuis le tableau (10 refus = 5 avec `details` + 5 sans). **F-R2 — le doc-comment corrigé au LOW de la passe 1 annonçait cinq gardes et en énumérait six** : le règlement **partiel** n'est pas une garde de plus, c'est la **première élargie**. *Une conséquence de l'ancien énoncé a survécu à sa réécriture, dans la réécriture qui corrigeait déjà ce défaut.* **F-R3 — le doc-comment de `ValidatedInvoice` s'était collé à `unvalidate`**, insérée juste au-dessus sans ligne vide : rustdoc l'attachait à ma fonction, dont la ligne de résumé annonçait « résultat d'une validation réussie », et laissait la struct nue. Rendu à sa struct. LOW : les assertions de corps s'arrêtaient avant `invoiceNumber` et `version`, et `is_null()` passait à vide sur un champ **absent** (`get()` distingue) ; la seconde moitié du test HTTP ne tue aucune mutation — couverture légitime de la correspondance HTTP, **à ne pas compter comme garde prouvée**. ⚠️ **Repris à la main sur un axe que la lentille D déclarait exercé sans le montrer** : le manuel utilisateur (`user-manual.tex:540`) annonce **un** chemin qui creuse la numérotation des écritures, et il y en a **deux** dès ce merge — limite écrite au § *Périmètre*, site déjà inscrit aux ancrages de la b-2. Vérifié sain et **à ne pas refaire** (lentille R) : les deux appelants de la signature changée, la cohérence facture/lignes sous le `FOR UPDATE`, la forme de la réponse inchangée (`fetch_lines` et `find_by_id_with_lines` émettent la même requête), le montage des deux tests neufs contre les cinq CHECK du schéma, l'ordre des nettoyages contre les FK, la jointure bancaire. ⚠️ **Signalé pour la b-2** : `invoices::delete` ne lit que `paid_at`, donc une facture à règlement **partiel** reste supprimable et `ON DELETE CASCADE` effacerait la ligne de règlement en silence. Gate backend complet **2417/2417** sur base reconstruite ; résidu nul sur **sept** tables contrôlées. |
| 2026-09-22 | review P3 — **close** | **Passe 3, CIBLÉE** (une lentille Sonnet, prompt versionné `25-2-b-1-review-prompt-p3-ciblee.md`) sur le seul commit `7552d823`, dont il était **vérifié** qu'il ne touche aucune ligne de code exécutable. Son axe n'était donc pas le code, mais **chaque affirmation que le commit ajoute, confrontée au code** — parce que ce motif-là tenait depuis trois passes. **0 CRITICAL, 0 HIGH, 1 MEDIUM, 0 LOW.** Le MEDIUM : ma cellule neuve datait de « la v0.10 » l'apparition des lignes de règlement, or `20260827000001_invoice_settlements.sql` n'est ancêtre **ni de v0.11.1 ni d'aucune version antérieure à v0.12.0** — établi par `git merge-base --is-ancestor`. Corrigé, et **en citant ce qui ne se périme pas** : la version d'introduction nommée par ce qu'elle a introduit, comme le fait déjà `errors.rs`. Tout le reste s'est vérifié juste, et la lentille l'a établi plutôt que de l'affirmer : les cinq cellules du tableau `details` contre les cinq sites de construction et `build_response`, les deux `null` conditionnels, le total « 10 = 5 + 5 » recompté sur les **deux** tableaux, les trois motifs sur cinq, les gardes 3 contre 5 de `delete_in_tx` et la garde **élargie**, le doc-comment rendu à `ValidatedInvoice` (et aucun autre décroché dans les deux fichiers), les assertions neuves du test, et la citation `user-manual.tex:539-541` avec son inscription aux ancrages de la b-2. Axe déclaré **non exercé** : le `2417/2417` et le résidu nul, invérifiables sans lancer le gate — interdit par le prompt, et exécuté par l'orchestrateur. ⛔ **BOUCLE CLOSE** au critère explicite du `CLAUDE.md` : la remédiation de cette passe ne touche **aucune ligne de code de production** — c'est une cellule de tableau dans un document. Trend : `3H 4M 3L → 0H 3M 2L → 0H 1M 0L`. ⚠️ **Trouvé par le grep de propagation, HORS PÉRIMÈTRE et NON corrigé** : `docs/api-external.md:312` avertit que le correctif de la KF-036 (#167) « n'est pas dans la v0.9.0 » et « sera livré à la prochaine version publiée » — il est publié **depuis la v0.10.0**, et le paragraphe annonce donc à un lecteur d'aujourd'hui qu'une **faille de sécurité est encore ouverte**. Signalé à Guy : c'est une communication publique sur une faille, elle ne se tranche pas en passant. |
| 2026-09-21 | validate P2 — **close** | **Passe 2, une lentille Sonnet** (P1 était Opus), prompt versionné `25-2-b-1-validate-prompt-p2.md`, **sept axes exercés, aucun laissé**. **0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW** — et le LOW est de nature **découpage** : la citation `invoices.rs:1339`, resserrée par ma tâche T3-bis là où la mère citait une plage, coupait une phrase qui court sur deux lignes. Corrigée. ⛔ **Aucun finding de CONCEPTION** — c'est le signal qui comptait : la sévérité `HIGH → LOW` confirme que le découpage était la bonne remédiation, et non un report du problème. Vérifié depuis la source et **à ne pas refaire** : le contrat de `validate_invoice_handler` (aucun corps), la convention `version` du dépôt, `comptable_routes` sans garde anti-clé, `errors.rs:57-106` et `MATCHED_BANK_TRANSACTION` à `:102`, la précédence réelle `5, 8, 6` du code mergé, les compteurs du registre de routes, `SITES_INDIRECTS`, les quatre locales, et — point neuf — **`find_covering_date` existe sans filtre de statut**, ce qui rend le mécanisme de l'AC 5 réalisable sans lecture dupliquée. Axe 2 tranché : **aucun test existant ne rougit au merge de b-1 seule**, l'état « brouillon numéroté » n'existant pas encore sur `main`. |
