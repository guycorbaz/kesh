# Story 25.4-c2 : Le verrou à l'acceptation d'un rapprochement

Status: ready-for-dev

**Issue : [#480]** — ⛔ la PR porte `closes #480`, titre ET corps (§ *Issue Tracking Rule*).

**Née du découpage de la 25-4-c** (validation P3, sévérité P2 → P3 non décroissante ; accord de
Guy le 2026-09-29). Les findings P3 F1–F3 de la 25-4-c sont la source des faits, recontrôlés ici
dans le code. **Empilée sur la 25-4-c** (PR #483, non mergée) : branche
`story/25-4-c2-verrou-acceptation` tirée de `4c36ac99`. La 25-4-c fait d'une facture partiellement
réglée un candidat **ordinaire** du rapprochement : la course décrite ici devient atteignable d'un
clic, d'où l'ordre.

## Story

En tant que comptable,
je veux qu'un rapprochement et un règlement manuel enregistrés en même temps sur la même facture ne
puissent pas la régler deux fois,
afin que le compte clients ne devienne jamais créditeur sans que rien ne le signale.

## Le défaut, vérifié dans le code

**Isolation réelle** : MariaDB **10.11** en dev, en CI et en production (`docker-compose*.yml`,
`.github/workflows/ci.yml`), `REPEATABLE-READ`, `innodb_snapshot_isolation = OFF`, attente de verrou
50 s (relevé le 2026-09-30 sur `kesh-mariadb-dev`). Conséquence : une lecture **simple** lit
l'**instantané** de la transaction (fixé à sa première lecture InnoDB) ; une lecture **verrouillante**
(`FOR UPDATE`) et un `UPDATE` lisent la **dernière version validée**.

### Comment l'acceptation se protège aujourd'hui : le verrou optimiste

`accept_one_invoice` (`crates/kesh-api/src/routes/reconciliation.rs:1067`) lit la facture sans
verrou (`find_invoice_by_id_for_company`, étape 5, `:1115`), retient `invoice_version_pre`
(`:1255`), lit le reste dû sans verrou (`invoice_settlements::amount_due`, garde de trop-perçu
`:1341`), écrit l'encaissement, puis termine par
`UPDATE invoices … WHERE … AND version = ?` (`:1510-1518`). Cet `UPDATE` lit la version **courante** :
si une autre transaction a validé une écriture qui a **incrémenté `version`** depuis l'instantané, il
ne touche aucune ligne et la proposition sort en `RECONCILIATION_INVOICE_NOT_ELIGIBLE`
(`race_during_update`, `:1527-1533`) — son point de sauvegarde est annulé, rien n'est écrit.

**Le verrou optimiste ne vaut que si TOUT écrit qui change le reste dû incrémente `version`.**

### L'inventaire des écrivains de `invoice_settlements` (ensemble clos)

`grep -rn "INSERT INTO invoice_settlements\|DELETE FROM invoice_settlements\|UPDATE invoice_settlements\|invoice_settlements::create_in_tx" crates/*/src`
(le 2026-09-30) :

| Écrivain | Site | Incrémente `version` ? |
|---|---|---|
| Acceptation d'un rapprochement | `reconciliation.rs:1428` (création) + `:1510` | **toujours** |
| Règlement manuel — `settle_invoice` | `invoice_settlements_write.rs:218` + `:233-246` | ⛔ **seulement au solde** (`if fully_settled`) |
| Annulation d'un règlement — `cancel_settlement_in_tx` (règlement manuel **et** annulation d'un rapprochement, `reconciliation_cancel.rs:352`) | `invoice_settlements_write.rs:451` + `:464-470` | **toujours** (les deux branches) |
| Rejeu post-restauration | `post_restore/20260828000001_invoice_settlements_type.sql:23` | hors ligne (import d'installation) |
| Tests | `invoices.rs:4220`, `:4257` | `mod tests` |

**Au-delà des règlements** — le reste dû soustrait aussi l'**avoir émis**
(`INVOICE_AMOUNT_DUE_DERIVED_SQL`, `invoice_settlements.rs:105-106`), et l'acceptation exige le statut
`validated`. Les autres écrivains qui touchent ces grandeurs, vérifiés sûrs :

| Écrivain | Site | Incrémente `version` ? |
|---|---|---|
| Émission d'un avoir — `create_credit_note` | `credit_notes.rs:283` (`FOR UPDATE` en tête) + `:586` (`status = 'cancelled'`) | **toujours** |
| Dévalidation — `invoices::unvalidate` | `invoices.rs:1453` (`FOR UPDATE` en tête) + `:1577` | **toujours** |
| Lignes de facture (`invoice_lines`) | brouillon seulement : une facture validée est immuable | sans objet |

L'`UPDATE … AND version = ? AND status = 'validated'` d'`accept_one_invoice` refuse donc déjà ces deux
courses. **C'est l'ensemble complet** qu'énonce l'invariant de l'AC 1.

**Le trou est unique** : un règlement manuel **partiel** ne touche pas `version`. D'où les deux
courses :

1. **Acceptation contre règlement manuel partiel** — l'acceptation fige son instantané, le règlement
   manuel partiel (400 sur 1 000) valide **sans** incrémenter `version`, l'acceptation lit le reste
   périmé (1 000), passe la garde de trop-perçu avec 1 000, et son `UPDATE … version = ?` **réussit**.
   Facture réglée 1 400 pour 1 000 : **compte clients créditeur**, sans erreur.
2. **Acceptation contre acceptation** (deux comptes bancaires, donc deux `GET_LOCK` distincts) — la
   seconde est **déjà** refusée par le verrou optimiste, puisque l'acceptation incrémente toujours.
   À garder tel quel.

Symétriquement, un règlement manuel qui suit une acceptation en cours attend sur la ligne `invoices`
(son `FOR UPDATE`, `:63-68`, est sa **première** instruction ; l'acceptation tient un verrou exclusif
depuis son `UPDATE`) puis lit un état à jour : ce sens-là est sûr.

### ⛔ Pourquoi PAS un `FOR UPDATE` à l'acceptation (P3 F1 de la 25-4-c)

Posé au chargement de la facture, il lirait `version` **à jour** — donc le contrôle `version = ?`
passerait toujours — alors que `amount_due`, lecture simple, lirait encore l'**instantané**
(`reconciliation_cancel.rs:292-293` et `users.rs:100-107` documentent la même règle). Il **désarmerait**
le verrou optimiste qui refuse aujourd'hui la course n° 2, sans fermer la n° 1. Le patron de
`settle_invoice` ne se transpose pas : son `FOR UPDATE` est la première instruction de SA transaction,
alors que dans un lot l'instantané est fixé dès la première proposition.

### Les interblocages, et leur issue actuelle (P3 F3)

Le lot garde, d'une proposition à l'autre, le verrou d'exercice (`find_open_covering_date … FOR
UPDATE`, `:1362`) et celui de la numérotation des écritures ; le règlement manuel et l'annulation
prennent la **facture puis** l'exercice (`invoice_settlements_write.rs:68` → `:174` ;
`reconciliation_cancel.rs:296` → `:309`). Sur deux comptes bancaires distincts, InnoDB peut donc
choisir l'acceptation comme victime d'un interblocage (1213) — **préexistant** (le commentaire
`reconciliation.rs:3616-3619` le décrit déjà pour l'annulation). Il annule **toute** la transaction ;
l'instruction fautive ressort en `FailedProposal` `DATABASE_ERROR`, puis `ROLLBACK TO SAVEPOINT`
(`:979`) échoue faute de point de sauvegarde, le `?` produit `ReconciliationError::Database`, et la
réponse est un **HTTP 500** — sans rejeu : `retry_with` n'existe que sur l'annulation (`:3624-3646`).
Une attente dépassée (1205) n'annule, elle, que l'instruction (`innodb_rollback_on_timeout` par
défaut) : le point de sauvegarde survit, la proposition sort en échec au bout de 50 s.

## Acceptance Criteria

**AC 1 — L'invariant.** Tout écrit qui change le reste dû d'une facture incrémente `invoices.version`
dans la **même** transaction. Concrètement : `settle_invoice` incrémente `version` (et `updated_at`)
**à chaque règlement**, partiel compris — `paid_at` n'est posé qu'au solde, comme aujourd'hui. Le
doc-comment de `settle_invoice` et celui d'`accept_one_invoice` (au `UPDATE … version = ?`) énoncent
l'invariant et nomment les **deux** inventaires ci-dessus (règlements, et avoir / dévalidation) : le
prochain écrivain qui y manquerait doit trouver la règle écrite là où il écrit.

**AC 2 — La course n° 1 est refusée.** Une acceptation dont l'instantané précède un règlement manuel
partiel validé depuis sort en `RECONCILIATION_INVOICE_NOT_ELIGIBLE` (`race_during_update`), sans rien
écrire : aucun règlement, aucune écriture d'encaissement, transaction bancaire toujours `pending`.
**Test déterministe** (sans `sleep`, AC 5).

**AC 3 — La course n° 2 reste refusée.** Deux acceptations du même solde sur la même facture, depuis
deux comptes bancaires : une acceptée, l'autre refusée, jamais deux règlements. Test déterministe.

**AC 4 — Un interblocage se rejoue.** Quand la transaction du lot est annulée par InnoDB (1213), la
route d'acceptation **rejoue toute l'opération** — transaction neuve, verrou de compte repris —, par
`retry_with`, patron de `post_cancel_reconciliation` (`reconciliation.rs:3612-3646`). ⚠️ L'erreur qui
remonte aujourd'hui n'est **pas** le 1213 mais l'échec du `ROLLBACK TO SAVEPOINT` (1305, mode d'échec
connu du couple interblocage / point de sauvegarde) : `accept_batch` doit reconnaître qu'une
transaction a été annulée sous lui et le faire remonter par une **erreur typée dédiée** (par exemple
un variant `ReconciliationError::TransactionAborted`, posé explicitement quand `ROLLBACK TO SAVEPOINT`
échoue en 1305), que le prédicat de rejeu **de la route d'acceptation** reconnaît, au même titre que
`is_deadlock_error`. ⛔ **Ne pas élargir `kesh_db::retry::is_deadlock_sqlx` / `is_deadlock_error`**
(`retry.rs:71-85`, 1213 seulement) : ils sont partagés par tout le crate, et reclasser tout 1305 en
interblocage masquerait ailleurs un vrai défaut de point de sauvegarde. ⛔ **Ne pas classer en lisant
le texte d'un message d'erreur** : le code d'erreur MySQL, pas la chaîne.
Chemin à tenir, vérifié dans le code : le prédicat de `retry_with` voit un **`AppError`** (patron
`:3635`) ; la variante typée traverse donc le `match lock_result` de la route (exhaustif, un bras à
ajouter) vers une variante d'`AppError` que le prédicat reconnaît, et qui, faute de rejeu possible,
rend un 500. Un 1213 qui remonte **directement** par un `?` (`SAVEPOINT`, `RELEASE SAVEPOINT`) arrive
déjà en `AppError::Database(DbError::Sqlx(1213))` : `is_deadlock_error` le couvre. `GET_LOCK` est un
verrou de **session**, relâché par `with_account_lock` même quand la closure échoue
(`mutex.rs:66-160`) : une nouvelle tentative, transaction neuve, le reprend proprement. Après `DEFAULT_MAX_DEADLOCK_ATTEMPTS` tentatives, l'erreur finale reste un 500.
Test : un interblocage provoqué de façon déterministe (deux connexions,
`attendre_une_requete_en_cours`) est rejoué et la proposition finit acceptée. Si un interblocage
déterministe s'avère impossible à monter, le Dev Agent Record le dit et le test porte sur la
reconnaissance de l'annulation (1305 après `ROLLBACK`) — **jamais** un test qui passe à vide.

**AC 5 — Tests déterministes, et qui auraient échoué avant.** Chaque test de course :
- met l'acceptation en attente **à un point connu** (par exemple : une connexion de test tient
  `FOR UPDATE` sur la ligne `fiscal_years` que l'acceptation va verrouiller en (d), **après** sa garde
  de trop-perçu), attend qu'elle y soit (`kesh_db::test_fixtures::attendre_une_requete_en_cours`,
  `test_fixtures.rs:514` ; patron `credit_notes_repository.rs:594-640`), fait valider l'écriture
  concurrente, puis relâche ;
- ⚠️ le règlement manuel concurrent ne doit pas dépendre du verrou que tient le test : le dater dans
  un **autre exercice ouvert** que celui que l'acceptation attend, ou tout autre montage qui évite de
  bloquer le règlement derrière le verrou de test ;
- ⚠️ ne met dans le lot **que** la proposition sous test : `ROLLBACK TO SAVEPOINT` ne relâche pas les
  verrous de ligne pris après le point de sauvegarde (seuls ceux des lignes insérées) — une
  proposition antérieure sur la même facture garderait la ligne `invoices` verrouillée jusqu'à la fin
  du lot et fausserait le montage ;
- est **éprouvé par mutation** : l'incrément de `version` retiré du règlement partiel, le test AC 2
  échoue (double règlement) ; le rejeu retiré, le test AC 4 échoue.

**AC 6 — Rien d'autre ne change.** Aucun `FOR UPDATE` ajouté dans `accept_one_invoice` ; aucune
modification de la formule du reste dû ; l'ordre des gardes (score avant trop-perçu) intact. Les
écrans qui gardent `version` après un règlement la relisent : `+page.svelte` de la facture reprend
`res.invoice` (`invoices/[id]/+page.svelte:435-436`) — vérifier que la réponse du règlement porte la
facture **relue après validation**, sans quoi un `pauseDunning` qui suit (`:512`, version en garde)
sortirait en 409. Les tests existants qui assertent la `version` d'une facture après un règlement
partiel sont mis à jour, et le Dev Agent Record les nomme.

**AC 7 — Textes.** `docs/api-external.md` ne documente **pas** `POST /api/v1/reconciliation/accept`
(la section *Annuler un rapprochement bancaire*, `:287`, ne couvre que la consultation et
l'annulation) : y ajouter une entrée courte pour l'acceptation — accès, corps, succès partiel
`{ accepted, failed }`, et le rejeu d'un interblocage transitoire, dans les termes de l'annulation
(`:291`). CHANGELOG `[0.12.1]` *Fixed*. Le manuel : relire la section *Réconciliation
bancaire* et le règlement manuel — n'y rien écrire s'ils ne promettent rien sur la concurrence, et le
dire dans le Dev Agent Record.

## Tasks / Subtasks

- [ ] **T1 — l'invariant** (AC 1, 6) : `settle_invoice` incrémente `version` à chaque règlement ;
  doc-comments ; réponse du règlement relue après validation ; tests existants de `version`.
- [ ] **T2 — le rejeu** (AC 4) : `accept_batch` reconnaît la transaction annulée (erreur typée
  dédiée) ; route d'acceptation sous `retry_with` (une fonction « une tentative », comme
  `cancel_reconciliation_once`), prédicat local à la route.
- [ ] **T3 — tests de course et mutations** (AC 2, 3, 5).
- [ ] **T4 — textes** (AC 7).
- [ ] **T5 — gates** : backend complet (base remise à zéro — ⛔ `kesh-db` touché : gate complet même en
  cours de boucle), frontend complet, **E2E complet**.

## Dev Notes

### Ce qu'il ne faut pas faire

- ⛔ **`FOR UPDATE` sur la facture dans `accept_one_invoice`** — il désarme le verrou optimiste (voir
  plus haut). Toute autre lecture verrouillante qui rafraîchirait `invoice_version_pre` aussi.
- ⛔ **Classer une erreur par son texte** (`contains("Deadlock")`) : le code MySQL.
- ⛔ **Élargir `is_deadlock_error` au 1305** : prédicat partagé ; la reconnaissance du point de
  sauvegarde perdu reste locale à l'acceptation (`reconciliation.rs` est le seul site à utiliser des
  `SAVEPOINT` nommés du dépôt).
- ⛔ **Un test de course avec `sleep`** : il passe à vide un jour sur deux (`tests-qui-prouvent-moins`).
- ⛔ **Changer le niveau d'isolation** (`READ COMMITTED`, `SERIALIZABLE`) pour régler la course :
  `pool.rs:17-21` assume REPEATABLE READ pour tout le crate.
- ⚠️ **Mise à niveau de MariaDB** : à partir de la 11.6, `innodb_snapshot_isolation` vaut `ON` par
  défaut — une lecture verrouillante ou un `UPDATE` sur une ligne modifiée depuis l'instantané échoue
  alors (1020, *Record has changed since last read*) au lieu de lire la dernière version. Le verrou
  optimiste reste juste dans les deux régimes, mais l'erreur change de forme : le dire dans le
  doc-comment d'`accept_one_invoice`, pour que la mise à niveau ne le découvre pas en production.

### Où regarder

| Fichier | Pourquoi |
|---|---|
| `crates/kesh-api/src/routes/reconciliation.rs:700-990` | route d'acceptation, `with_account_lock`, `accept_batch` et ses points de sauvegarde |
| `crates/kesh-api/src/routes/reconciliation.rs:1067-1575` | `accept_one_invoice` : lectures, gardes, `UPDATE … version = ?` |
| `crates/kesh-api/src/routes/reconciliation.rs:3612-3680` | patron du rejeu (`post_cancel_reconciliation`, `cancel_reconciliation_once`) |
| `crates/kesh-db/src/repositories/invoice_settlements_write.rs:40-260` | `settle_invoice` : verrou, garde, incrément conditionnel à changer |
| `crates/kesh-db/src/repositories/invoice_settlements_write.rs:371-480` | `cancel_settlement_in_tx` : incrément inconditionnel, le modèle |
| `crates/kesh-db/src/retry.rs:39-160` | `retry_with`, `is_deadlock_error`, `DEFAULT_MAX_DEADLOCK_ATTEMPTS` |
| `crates/kesh-reconciliation/src/mutex.rs:66-120` | `with_account_lock` (`GET_LOCK`), à reprendre à chaque tentative |
| `crates/kesh-db/src/test_fixtures.rs:505-540`, `crates/kesh-db/tests/credit_notes_repository.rs:594-640` | attente déterministe d'une requête bloquée |
| `crates/kesh-api/tests/reconciliation_e2e.rs` | `setup_company`, `seed_validated_invoice`, `post_accept_one`, `seed_partially_settled_vat_invoice` (25-4-c) |
| `frontend/src/routes/(app)/invoices/[id]/+page.svelte:430-440, 505-520` | version relue après règlement, `pauseDunning` |

### Gardes-fous du dépôt

- Aucune migration. `kesh-db` touché (repository) ⇒ **gate complet même en cours de boucle**.
- Modules : `kesh-db`, `kesh-api` (+ `docs`) — sous le seuil de découpage.
- ⚠️ La base partagée : les tests de course sont des `#[sqlx::test]` (base éphémère), jamais sur
  `kesh` — deux connexions du même pool éphémère.

## Questions pour Guy

Aucune bloquante. Un choix est fait par défaut et révisable : **le rejeu (AC 4) est inclus**. L'issue
#480 le pose comme question (« `retry_with` ? ») ; sans lui, la story laisse un 500 sur un
interblocage que la 25-4-c rend plus probable, et le patron existe déjà sur l'annulation.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-09-30** — Créée depuis #480 et les findings P3 F1–F3 de la 25-4-c, recontrôlés dans le code :
  isolation réelle relevée sur MariaDB 10.11 ; inventaire clos des écrivains de `invoice_settlements`
  — **un seul** n'incrémente pas `version` (le règlement manuel partiel) ; la solution retenue est
  l'invariant d'incrément, **pas** un `FOR UPDATE` ; rejeu sur interblocage inclus par défaut.
- **2026-09-30** — Validation P1 (Sonnet, prompt `25-4-c2-validate-prompt-p1.md`) : **1 HIGH,
  2 MEDIUM, 2 LOW**, vérifiés. HIGH : `is_deadlock_error` ne teste que le 1213 (`retry.rs:71-85`), l'AC 4
  le nommait pour reconnaître un 1305 → erreur typée dédiée, prédicat local, interdiction d'élargir le
  prédicat partagé. MEDIUM : `api-external.md` ne documente pas `/accept` → une entrée à écrire, pas une
  phrase. MEDIUM : l'inventaire ne couvrait que `invoice_settlements` → avoir et dévalidation ajoutés
  (vérifiés : `FOR UPDATE` en tête, incrément inconditionnel). LOW : `pool.rs:17-21`,
  `reconciliation_cancel.rs:292-293`. Remarque intégrée à l'AC 5 : `ROLLBACK TO SAVEPOINT` ne relâche
  pas les verrous de ligne.
- **2026-09-30** — Validation P2 (Haiku, prompt `25-4-c2-validate-prompt-p2.md`) : 1 CRITICAL, 1 HIGH,
  1 MEDIUM annoncés — **tous écartés, erreur de catégorie** : la lentille reproche au code de ne pas
  encore porter ce que la fiche prévoit (variante typée, `retry_with`, entrée d'API), ce qui est
  l'objet de l'implémentation. Axes utiles, preuves jointes : inventaire avoir / dévalidation vérifié ;
  **aucun contre-exemple** à l'invariant (relève aussi la suspension des rappels, qui incrémente
  `version` : une acceptation concurrente est refusée, sens prudent). Axe mal exercé (faisabilité du
  rejeu) **repris par l'orchestrateur** : `GET_LOCK` de session relâché sur erreur, prédicat sur
  `AppError`, 1213 direct déjà couvert — précisions ajoutées à l'AC 4. **Validation close : 0 > LOW.**

  **Bilan** — P1 Sonnet 1H/2M/2L → P2 Haiku 0 > LOW (3 annoncés, écartés). Modèles : Sonnet, Haiku.

[#480]: https://github.com/guycorbaz/kesh/issues/480
