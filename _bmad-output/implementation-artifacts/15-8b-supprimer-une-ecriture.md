# Story 15.8b : Supprimer une écriture tant que son exercice est ouvert, et l'historique sur la fiche

Status: ready-for-dev

<!-- Issue de la story 15-8, DÉCOUPÉE le 2026-10-08 après la validation P2 (choix C-15-8-17 de
     `epic-15-choix-autonomes.md`). Elle porte la SUPPRESSION, l'historique visible sur la fiche et le retrait
     d'`ENTRY_IS_POSTED`. Choix applicables : C-15-8-2, C-15-8-6, C-15-8-8, C-15-8-12, C-15-8-14, C-15-8-17,
     C-15-8-19, C-15-8-20. Historique des passes P1 et P2 : Change Log de l'index `15-8-modifier-une-ecriture.md`. -->

**Issue : [#532]** — ⛔ **`closes #532`** (dans le titre ou le corps de la PR : le dépôt merge en squash). ⚠️ URGENTE.

**Dépendances** — ⛔ **la 15-8a passe AVANT** (`15-8a-modifier-une-ecriture.md`), et la 15-5a avant elle. Cette story
**réutilise** ce que la 15-8a pose, sans le redéfinir : le cadre D1 (dont le paiement détaché), la garde
`modification_guard` / `modification_refusal`, `ModificationGuard`, `DbError::EntryNotModifiable`, `ModificationBlocker`
et les trois champs du détail, la table de correspondance code d'écran ↔ refus, le formulaire et la fiche. Elle se
branche sur `main` **après le merge de la 15-8a** ; tous les numéros de ligne ci-dessous, relevés sur `main` avant les
deux merges, se **revérifient** (T0).

## Story

**As a** personne qui tient les livres,
**I want** pouvoir supprimer une écriture saisie à la main — l'écriture d'ouverture comprise — dans le même cadre que
la modification, et voir sur sa fiche qu'elle a été modifiée et par qui,
**so that** une écriture saisie par erreur disparaît sans écriture inverse, sans que sa disparition ni ses corrections
cessent d'être apparentes.

## Ce que cette story ajoute à la 15-8a

| geste | cadre |
|---|---|
| **`DELETE /api/v1/journal-entries/{id}`** | le cadre D1 de la 15-8a, à l'identique : exercice ouvert, ni contre-passée ni contre-passation, aucune pièce, pas un paiement détaché, date postérieure à `books_locked_through` ; trace `journal_entry.deleted` avec l'instantané complet et l'acteur (utilisateur **ou clé d'API**) |
| **historique sur la fiche** | mention « Modifiée » quand `version > 1`, lien « Historique » vers le journal d'audit filtré (C-15-8-6) |
| **retrait du gel** | `DbError::EntryIsPosted`, le code `ENTRY_IS_POSTED`, la clé `journal-entries-blocked-posted` et `enforce_immutability` disparaissent |

## Décisions

### D1 — `DELETE` : la garde de la 15-8a

`delete_in_tx` (`journal_entries.rs:1005`) garde ses deux appelants. Le paramètre **`enforce_immutability` est renommé
`enforce_ownership`** et change de sens : à l'étape 3-ter, le refus inconditionnel `EntryIsPosted` est remplacé par
`modification_guard` → `modification_refusal` (15-8a D2), **dans la transaction**. Un paramètre
**`actor_api_key_id: Option<i64>`** s'ajoute, et l'audit `journal_entry.deleted` passe de `NewAuditLogEntry::user`
(`:1114`) à `NewAuditLogEntry::for_actor` — sans quoi une suppression par clé d'API serait attribuée au créateur de la
clé (C-15-8-8 : « la trace porte la clé »). Signatures : `delete_by_id(pool, company_id, id, user_id,
actor_api_key_id)` et `delete_in_tx(tx, company_id, id, user_id, actor_api_key_id, enforce_ownership)` ; la route passe
`current_user.api_key_id`.

⚠️ `delete_in_tx` respecte **déjà** la règle de sérialisation (15-8a D2) : `delete_by_id` ouvre la transaction et le
premier acte de `delete_in_tx` est son `FOR UPDATE` joint (écriture **et** exercice, étape 2, `:1014-1019`) ; la borne
de verrou se lit après (requête `:1075-1081`). Le doc-comment le dit désormais ; le test de concurrence de l'AC 5 le
tient.

| appelant | valeur | effet |
|---|---|---|
| `delete_by_id` (la route) | `true` | étape 3-ter : `modification_guard` → `modification_refusal` |
| `invoices::unvalidate` (`invoices.rs:1654`) | `false`, `actor_api_key_id = None` | la facture supprime **sa** propre écriture — la garde `OwnedByInvoice` n'a pas de sens ici. ⚠️ `unvalidate` ne reçoit aujourd'hui aucun `actor_api_key_id` (`invoices.rs:1486-1492`) : **hors périmètre**, signalé à l'orchestrateur |

Précédence (inchangée hormis 3-ter) : `FiscalYearClosed` (400) → `EntryIsReversed` (409, étape 3-bis, existante) →
**garde de la 15-8a** (409, si `enforce_ownership` ; `Owned { AlreadyReversed }` n'y arrive jamais, l'étape 3-bis l'a
pris) → `PeriodLocked` (400) → instantané, audit `journal_entry.deleted`, `DELETE` (CASCADE).

⛔ **La contre-passation elle-même (`IS_A_REVERSAL`) est refusée par la garde** : la clé étrangère `RESTRICT` ne
protège que l'**origine** — rien ne s'opposerait à l'effacement d'une contre-passation. C'est un cas où la garde apporte
un refus que la base n'apporterait pas ; le **paiement détaché** (`DETACHED_SUPPLIER_SETTLEMENT`, 15-8a D1) en est un
autre : aucune colonne ne le référence plus. Les deux sont testés (AC 2, AC 4).

⚠️ **Le numéro n'est jamais réattribué** (compteur de la 25-2-c) : la suppression creuse un **trou**, expliqué par
l'instantané `journal_entry.deleted`. Le manuel (§ Numérotation) doit le dire — il affirme aujourd'hui qu'« une
écriture ne se supprime plus ».

### D2 — Rejeu sur interblocage, comme le `PUT`

Le handler `DELETE` est enveloppé dans `retry_with(DEFAULT_MAX_DEADLOCK_ATTEMPTS, is_deadlock_error, …)`
(C-15-8-19), comme le `PUT` de la 15-8a. Son ordre de verrous est celui de la contre-passation — **écriture et exercice
ensemble** (jointure), puis, au `DELETE`, les contrôles des clés étrangères qui visent l'écriture — ; il entre donc dans les cycles **par les exercices** que la 25-7 nomme (création ou règlement
qui tient l'exercice puis insère). Le rejeu coûte trois lignes, et la transaction est rejouée entière, sans effet hors
d'elle : il n'y a pas de raison de s'en passer. La ligne de Pattern 5 posée par la 15-8a
(`docs/MULTI-TENANT-SCOPING-PATTERNS.md`, « Deny list ») s'étend au `DELETE`.

### D3 — L'écriture d'ouverture supprimée

| geste | effet sur l'écran « Soldes de départ » |
|---|---|
| **supprimer** l'ouverture, seule écriture de la société | la société redevient vierge → **la génération est de nouveau proposée** (statut `READY`) — exactement la procédure que la 24-4b avait fermée (24-4b D7). ⚠️ La nouvelle ouverture reçoit le **numéro 2**, pas 1 : le compteur de la 25-2-c ne réattribue jamais un numéro (`un_numero_libere_n_est_jamais_reattribue`, `journal_entries.rs:2087`). **Accepté** (C-15-8-12) : le trou est expliqué par l'instantané `journal_entry.deleted` ; le manuel (§ soldes de départ) le dit, l'AC 7 l'asserte |
| supprimer l'ouverture quand d'autres écritures existent | génération toujours refusée ; ses comptes sans autre mouvement deviennent complétables, et le complément est daté du premier jour du premier exercice s'il est ouvert et hors période verrouillée (25-7, arbitrage 2) |

Le **complément** de 25-7 se supprime comme l'ouverture (204).

### D4 — L'écran : « Supprimer » et l'historique

**Fiche (`frontend/src/routes/(app)/journal-entries/[id]/+page.svelte`)**, sur ce que la 15-8a y a posé :

- bouton **« Supprimer »** (`data-testid="delete-entry"`) à côté de « Modifier », gouverné par **le même** `modifiable`
  (le cadre est le même) : ⛔ **absent, pas grisé**, quand `modifiable` est faux — le motif est déjà affiché par la
  15-8a ;
- **« Supprimer »** ouvre une confirmation (dialogue `role="dialog"`, patron de la confirmation de contre-passation de
  la même page) qui dit que le numéro **ne sera pas réattribué** et que la suppression est **inscrite au journal
  d'audit** ; confirmé → `deleteJournalEntry` (rétabli par l'inversion de la 15-8a, sans appelant jusqu'ici) → toast
  `journal-entry-deleted`, retour à `/journal-entries` ; un refus (409 de course, `FISCAL_YEAR_CLOSED`,
  `PERIOD_LOCKED`) → toast du message serveur, fiche rechargée ;
- **historique** : mention « Modifiée » (`journal-entry-modified`) quand `version > 1` (`data-testid="entry-modified"`),
  et lien « Historique » (`journal-entry-history`, `data-testid="entry-history-link"`) vers
  `/audit-log?entityType=journal_entry&entityId={id}` — l'écran du journal d'audit lit déjà ces filtres depuis l'URL
  (`audit-log/+page.svelte:120-125`) ;
- ⛔ **rôle Consultation** (C-15-8-14) : « Supprimer » et le lien « Historique » sont **absents** (le journal d'audit est
  refusé à Consultation, 403, `lib.rs:707-723`) — « Modifier » et « Contre-passer » l'étant déjà depuis la 15-8a.

**Clés i18n neuves** — quatre locales : `journal-entry-delete`, `journal-entry-delete-confirm-{title,message,cancel,delete}`,
`journal-entry-deleted` (noms repris de `d2910022`, message de confirmation **réécrit** : numéro non réattribué, trace
d'audit), `journal-entry-modified`, `journal-entry-history`. Toutes employées par la **route** `[id]/+page.svelte`, hors
du contrôle de `lint-i18n-ownership`.

### D5 — Le retrait du gel

`DbError::EntryIsPosted` (`kesh-db/src/errors.rs:626-638`, `:647`, `:768`), sa branche de mappage
(`kesh-api/src/errors.rs:3042-3050`), sa clé `journal-entries-blocked-posted` (quatre catalogues — seul consommateur :
`errors.rs:3046`) et le paramètre `enforce_immutability` disparaissent. Le doc de `PeriodLocked` est reformulé.
⚠️ **Une clé lue par le backend ne se retire pas sans greper `crates/`** : `t(key, default)` rend **la clé brute** si
le bundle est chargé et la clé absente (le piège S1-C1 de la 24-4b).

## Critères d'acceptation

1. **Supprimer une écriture manuelle** → **204** ; l'écriture et ses lignes ont disparu ; une entrée
   `journal_entry.deleted` porte l'instantané complet et l'acteur (utilisateur, ou clé d'API : `actor_type =
   'ApiKey'`, `actor_api_key_id`) ; la **création suivante** du même exercice ne reprend **pas** le numéro (compteur
   25-2-c).
2. **Supprimer une contre-passation** → 409 `IS_A_REVERSAL` (D1 : la base ne l'aurait pas refusé).
3. **Dévalidation inchangée** : `invoices::unvalidate` (`enforce_ownership = false`) supprime toujours l'écriture de
   sa facture — ses tests existants restent verts, sans réécriture.
4. **Gardes** : pour **chacune** des pièces de l'AC 6 de la 15-8a (facture, avoir, facture fournisseur — achat et
   règlement —, règlement client, solde `write_off`, transaction bancaire rapprochée) le `DELETE` rend **409 sous le
   code du motif** avec `details.documentId`, sur le **même montage** (`every_document_owned_entry_is_refused`, tel que
   la 15-8a l'a étendu) ; une écriture contre-passée → 409 `ENTRY_IS_REVERSED` ; le **paiement détaché** d'une facture
   fournisseur annulée → 409 `DETACHED_SUPPLIER_SETTLEMENT` ; **exercice clos seul** → 400 `FISCAL_YEAR_CLOSED` ; date
   ≤ borne (seuil inclusif) → 400 `PERIOD_LOCKED`. Après chaque refus, lignes et en-tête identiques.
5. **Concurrence** — `delete_waits_for_a_concurrent_reversal_then_refuses`, dans
   `crates/kesh-db/tests/journal_entries_modification.rs` (le fichier de la 15-8a) : B contre-passe E par
   `reverse_in_tx` sans commiter ; A lance `delete_by_id(E)` ; le test attend A bloquée sur son verrou, avec les motifs
   **du `SELECT` de `delete_in_tx`** (`:1014-1019` : `["je.fiscal_year_id", "FOR UPDATE"]` — pas ceux du `PUT`, que ce
   `SELECT` ne contient pas : `attendre_une_requete_en_cours` paniquerait à 10 s, `test_fixtures.rs:583-586`) ; B
   commite ; A rend `EntryIsReversed` et E est intacte. ⛔ **Mutation à tuer** (déclarée au Dev Agent Record) :
   placer la lecture de la borne avant le `FOR UPDATE` fait passer A — le test doit rougir.
6. **IDOR, rôles, rejeu** : `DELETE` sur un `id` d'une autre société ou inexistant → **404**, jamais 409 ;
   Consultation → **403** avant tout autre contrôle ; le handler est enveloppé dans `retry_with`
   (`grep -nF "retry_with" crates/kesh-api/src/routes/journal_entries.rs` rend le `PUT` **et** le `DELETE`) ; Pattern 5
   nomme le `DELETE`.
7. **Soldes de départ** (D3) : après **suppression** de l'ouverture seule écriture de la société, le statut redevient
   `READY` et une nouvelle génération réussit, sous le **numéro 2** (asserté — C-15-8-12) ; après suppression de
   l'ouverture quand d'autres écritures existent, ses comptes sans autre mouvement figurent dans `completableAccounts`.
   Une écriture de **complément** (25-7) se supprime (204).
8. **Détail** : la table de correspondance de la 15-8a (AC 12) gagne la colonne `DELETE` — pour **chacun** des dix
   codes d'écran, le `DELETE` rend le refus associé (le même que le `PUT`, `ALREADY_REVERSED` ↔ `ENTRY_IS_REVERSED`
   compris) ; `modifiable = true` → `DELETE` → 204. Table écrite en dur dans le test.
9. **Écran** : depuis la fiche, « Supprimer » demande confirmation puis ramène à la liste, l'écriture a disparu ;
   annuler la confirmation laisse la fiche ; le bouton est **absent** avec le motif quand l'écriture n'est pas
   modifiable ; « Modifiée » et « Historique » apparaissent après une modification ; le lien ouvre le journal d'audit
   filtré sur l'écriture ; au rôle **Consultation**, « Supprimer » et « Historique » sont absents. Couvert par
   **Playwright** — specs nommées en T5.
10. **Retraits** : `DbError::EntryIsPosted`, le code `ENTRY_IS_POSTED`, la clé `journal-entries-blocked-posted` (quatre
    catalogues) et `enforce_immutability` n'existent plus ; ⛔ greper **le dépôt entier** —
    `grep -rn "ENTRY_IS_POSTED\|EntryIsPosted\|enforce_immutability\|blocked-posted" . --exclude-dir={node_modules,target,.git,.svelte-kit}`
    — et trier à la main (leçon de la 24-4b). Sortie attendue : **uniquement** des fichiers sous `_bmad-output/`
    (story files historiques, registre des choix, `sprint-status.yaml`, prompts de validation, ces fiches — qui **ne se
    réécrivent pas** quand ce sont des archives), et la section `## [0.13.0]` du `CHANGELOG.md` qui dit `ENTRY_IS_POSTED`
    retiré. Toute autre ligne est un résidu.
11. **Audit** : `journal_entry.deleted` reste dans `ACTIONS` ; `audit_route_registry.rs:83` garde le `DELETE` à `Traced`.
12. **Documentation** : manuel utilisateur et administrateur, `docs/api-external.md`, `CHANGELOG.md` disent la
    suppression (cf. Dev Notes) ; PDF régénérés. ⛔ **Contrôle**, sur les deux `.tex` **et** sur les deux PDF aplatis :
    `grep -ciE "écriture ne se supprime plus|une seule voie fait encore disparaître|ni modifiable ni supprimable|ne se modifie ni ne se supprime|ni modification ni suppression"`
    rend **0**, **et** le motif de l'AC 17 de la 15-8a rend toujours **0** ; passages conservés nommés au Dev Agent
    Record — ⚠️ `user-manual.tex:1213` « Une facture validée, elle, ne se supprime plus » est **vrai** et le motif ne le
    prend pas (« **écriture** ne se supprime plus »).

## Tâches

- [ ] **T0 — Branche et inventaire au sol** (AC 10, 12) : brancher sur `main` après le merge de la 15-8a ; revérifier
      les numéros de ligne cités ; exécuter les greps des Dev Notes et relever leur **sortie complète** au Dev Agent
      Record, triée ligne par ligne.
- [ ] **T1 — `delete_in_tx`** (AC 1–5) : `enforce_immutability` → `enforce_ownership`, étape 3-ter = garde de la
      15-8a ; `actor_api_key_id` sur `delete_by_id` et `delete_in_tx`, audit par `for_actor` ; `invoices::unvalidate`
      passe `None` ; doc-comments de `delete_by_id` (`:924-949`) et `delete_in_tx` (`:967-1004`) **réécrits** (⛔ la
      24-4b a payé un MEDIUM pour un doc-comment resté à l'octet près) ; commentaires d'`invoices.rs:1442` et `:1650` ;
      en-tête de module (`:1-40`, « corriger par contre-passation plutôt que par suppression ») revu ; appelants de test
      de `delete_in_tx` dans `mod tests` (`:2123`, `:2182`, helper `:2439`) mis à la nouvelle signature (finding F12) ;
      test de concurrence (AC 5) et sa mutation
- [ ] **T2 — Retrait du gel** (AC 10) : `EntryIsPosted` et son doc, sa branche `kesh-api` et son commentaire
      (`:2986-2996`, ce qu'il en reste), la clé ×4, doc de `PeriodLocked`
- [ ] **T3 — Route** (AC 1, 6, 8) : `api_key_id` passé au `DELETE` ; handler enveloppé dans `retry_with` ;
      doc-comment du `DELETE` (`:609-610`, « asymétrie volontaire avec UPDATE ») revu ; tests de clé d'API (AC 1),
      colonne `DELETE` de la table de correspondance (AC 8)
- [ ] **T4 — Écran** (AC 9) : « Supprimer », confirmation, « Modifiée », « Historique », rôle Consultation ; clés i18n ×4
      (liste fermée de D4) ; `i18n-keys.test.ts` (`ATTENDU.sitesTotal`) **avec sa ventilation** ;
      `e2e-selecteurs-traduits.test.ts` selon les sélecteurs réellement employés
- [ ] **T5 — Playwright** (AC 9) — `tests/e2e/journal-entries.spec.ts`, parcours **depuis la fiche** remplaçant les
      specs de liste de suppression retirées par la 24-4b :
  - [ ] « suppression avec confirmation » → `supprimer depuis la fiche : confirmation, retour à la liste, l'écriture a disparu`
  - [ ] « annulation suppression » → `annuler la suppression : la fiche reste, l'écriture aussi`
  - [ ] neufs : `après modification : « Modifiée » et « Historique », qui ouvre le journal d'audit filtré` ; `rôle
        Consultation : ni Supprimer ni Historique` ; `écriture de facture : ni Modifier ni Supprimer, le motif est affiché`
        (étend la spec de la 15-8a)
- [ ] **T6 — Tests qui changent de sens** (liste ci-dessous) : réécrits, **pas** supprimés en bloc ; chaque test retiré
      nommé au Dev Agent Record avec son remplaçant
- [ ] **T7 — Documentation** (AC 12) — cf. Dev Notes ; `make fr`, PDF commités, contrôle de l'AC 12, table des matières
- [ ] **T8 — Gates** (⛔ complets — exception `kesh-db`)
  - [ ] base remise à zéro (KF-039), `scripts/test-fast.sh` sous `mem-guard`
  - [ ] `npm run check` · `lint-i18n-ownership` · `test:unit` · `build`
  - [ ] suite Playwright **complète au dernier commit de code**, jugée fichier par fichier contre `docs/testing.md`
        § « Les échecs attendus »

## Hors périmètre

- Tout ce que la 15-8a porte (modification, garde, formulaire, motif d'écran).
- **Audit par clé d'API de la dévalidation** (`invoices::unvalidate`), de la création et de la contre-passation :
  défaut préexistant, signalé à l'orchestrateur pour une issue.
- **Réinitialiser le compteur** quand la société redevient vierge : non (C-15-8-12).

## Dev Notes

### Les sites de la suppression — inventaire au sol (2026-10-08)

Commandes : celles de la 15-8a (T0). Sites de la **suppression** :

**Code**

| site | ce qu'il dit / fait | à faire |
|---|---|---|
| `kesh-db/src/errors.rs:626-638`, `:647`, `:768` | variante `EntryIsPosted`, son doc, sa mention dans le doc de `PeriodLocked` | retirer ; doc de `PeriodLocked` reformulé |
| `kesh-db/src/repositories/journal_entries.rs:924-1132` (`delete_by_id`, `delete_in_tx`, dont `:931`, `:939-940`, `:960-961`, `:978-996`, `:1047-1066`) | gel | réécrire (T1) |
| `journal_entries.rs:1-40` (en-tête de module) | « corriger par contre-passation plutôt que par suppression » | revu |
| `journal_entries.rs` `mod tests` : `le_gel_parle_avant_le_verrou_de_periode` (~`:2509`), le helper qui passe `enforce_immutability` (~`:2396-2444`), `:2465`, commentaire `:2120-2121` | test du gel | réécrire : avec `enforce_ownership = true`, une écriture manuelle de période verrouillée rend `PeriodLocked` |
| `kesh-db/src/repositories/invoices.rs:1442`, `:1650` | commentaires `enforce_immutability = false` | renommer, reformuler |
| `kesh-api/src/errors.rs:2986-2996` (ce que la 15-8a en a laissé), `:3042-3050` | commentaire et branche du gel | retirer |
| `kesh-api/src/routes/journal_entries.rs:609-623` | commentaire du `DELETE` | réécrire |
| `kesh-api/tests/period_lock_e2e.rs:3-10` | ce que la 15-8a en a laissé sur le `DELETE` | reformuler |
| `kesh-api/tests/journal_entry_reversal_e2e.rs:1095` (en-tête du bloc), `:1378-1383` | ce que la 15-8a en a laissé sur le `DELETE` | reformuler |

**Tests de la 24-4b qui changent de sens** — `crates/kesh-api/tests/journal_entry_reversal_e2e.rs` (part du `DELETE`) :

| test | aujourd'hui | devient |
|---|---|---|
| `deleting_a_posted_entry_is_refused` (`:1231`) | DELETE → 409 | AC 1 (204, audit, numéro non réattribué) |
| `the_opening_entry_is_frozen_but_still_correctable` (`:1386`) | moitié `DELETE` encore refusée après la 15-8a | ⛔ **inversé** : AC 7 |
| `an_entry_of_a_closed_year_stays_correctable` (`:1445`) | contre-passable en exercice clos | reste ; ajouter `DELETE` → 400 |
| `deleting_a_reversed_entry_is_refused_but_bulk_delete_still_works` (`:727`) | `ENTRY_IS_REVERSED` | **reste** |

**Frontend** : `[id]/+page.svelte` (D4) ; `i18n-keys.test.ts` ; `tests/e2e/journal-entries.spec.ts` (T5) ;
`e2e-selecteurs-traduits.test.ts`.

**i18n** — quatre locales : `journal-entries-blocked-posted` **retirée** ; clés neuves de D4 ;
`journal-entries-modify-blocked-detached-settlement` (posée par la 15-8a) : « … ne se modifie ni ne se supprime ».

**Manuel utilisateur** (`docs/manual/fr/user-manual.tex`) — sur l'état laissé par la 15-8a :

| ligne (avant la 15-8a) | à faire |
|---|---|
| `:478-494` § « Modifier une écriture » | renommée « Modifier ou supprimer une écriture » (entrée **7.4** de la table des matières) ; la suppression : même cadre, confirmation, numéro non réattribué, trace `journal_entry.deleted` ; « Modifiée » et « Historique » sur la fiche |
| `:554-555` § contre-passation | « En usage courant, une seule voie fait encore disparaître une écriture » devient faux : la suppression d'une écriture manuelle est une seconde voie, tracée |
| `:572-585` § Numérotation | « Une écriture ne se supprime plus » ; « si la dernière écriture disparaît — par la dévalidation de sa facture » : la suppression d'une écriture manuelle creuse aussi un trou ; le numéro n'est jamais repris |
| `:631` § clôture | la clôture ferme aussi la suppression |
| `:667` *keshnote* soldes de départ | supprimer l'ouverture quand elle est seule rouvre la génération (D3) — la nouvelle ouverture prenant le **numéro 2** (C-15-8-12) |
| `:1331-1337`, `:2140` | le paiement détaché ne se supprime pas non plus |
| `:2115-2140` FAQ | la suppression |

**Manuel administrateur** (`docs/manual/fr/admin-manual.tex`) : `:1796` (une clé `read-write` peut aussi
**supprimer** une écriture manuelle, tracée avec la clé) ; `:1835` (ce qui en reste après la 15-8a) ; `:1837` (« une
seule voie fait encore disparaître une écriture […] Ce n'est pas une brèche dans l'immutabilité » → la suppression
d'une écriture manuelle est une seconde voie, tracée) ; `:1838` (« modifications et suppressions **antérieures au
gel** » → et les suppressions postérieures, sous `journal_entry.deleted`).

**Autres documents** : `docs/api-external.md` (§ 7, sous-section de la 15-8a étendue au `DELETE`, tableau des refus de
D1) ; `docs/MULTI-TENANT-SCOPING-PATTERNS.md` (ligne du `DELETE`) ; `CHANGELOG.md` (section `## [0.13.0]` : la
suppression, et `ENTRY_IS_POSTED` **retiré** — changement de contrat pour une intégration ; il n'a jamais figuré dans
`docs/api-external.md`) ; `README.md:29` (« … et supprimables »).

**À NE PAS toucher** : les migrations publiées (P8) que nomme la 15-8a ; les story files historiques ; les fichiers
archivés de `docs/`.

### Pièges

- ⚠️ Les motifs d'`attendre_une_requete_en_cours` du `DELETE` sont ceux du `SELECT` de `delete_in_tx`, pas ceux du
  `PUT` (AC 5).
- ⚠️ **Base de gate piégée** (KF-039) : remise à zéro avant chaque gate complet.
- ⚠️ Haiku et les diffs multi-commits : diff aplati `main..HEAD`.

### Références

- Issue **#532** ; Story **15-8a** ; Story **24-4b** (gel) ; Story **25-2-b-zero** (#443, verrou sur la suppression) ;
  Story **25-2-c** (#381, compteur) ; Story **25-7** (#445).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log

| date | ce qui s'est passé |
|---|---|
| 2026-10-08 | **Créée par découpage** de la 15-8 après la validation P2 (choix C-15-8-17) ; spec, P1 et P2 au Change Log de l'index `15-8-modifier-une-ecriture.md`. Remédiations de la P2 propres à la suppression : motifs du test de concurrence `DELETE` (R2-8), rejeu sur interblocage du `DELETE` (F2, C-15-8-19), paiement détaché refusé au `DELETE` (F3, C-15-8-20), appelants de test de `delete_in_tx` (F12), sortie attendue du grep de l'AC 10 (R2-12), contrôle aplati élargi (R2-4, F5). **Recompte** (cette fiche) : 12 AC, 9 tâches (T0 à T8), 5 décisions (D1 à D5). |
