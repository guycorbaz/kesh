# Story 34-7 : le renommage d'un exercice est rejoué sur interblocage (#945)

Status: ready-for-dev

*Fiche rédigée le 2026-11-10 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e34`, branche `chore/epic-34-kickoff`,
numéros de ligne de `a75835db`. Release **0.17.6**, Epic 34. **Aucune migration.** Registre : `e34-choix-autonomes-e343.md`,
C-e343-8. Commits en `refs #945` ; à la PR : `closes #945`.*

## Story

En tant que Comptable, je veux que le renommage d'un exercice qui croise une clôture ou une validation de facture fournisseur
aboutisse, ou soit refusé pour son vrai motif. Il ne doit pas finir en erreur 500.

## Ce que le code dit (prémisses établies)

- **Le défaut est là.** `update_fiscal_year` (`crates/kesh-api/src/routes/fiscal_years.rs:297-324`) appelle
  `fiscal_years::update_name` (`:313-321`) **sans enveloppe de rejeu**. Ses voisines en ont une : création (`:286`), clôture
  (`:346`) et réouverture (`:407`).
- **Le cycle.** `update_name` (`crates/kesh-db/src/repositories/fiscal_years.rs:547-626`) tient l'exercice
  (`LOCK_IN_COMPANY_SQL`, `:162-163`, `FOR UPDATE`, lu à `:567-572`). Puis il cherche un homonyme par une lecture verrouillante
  (`:585-596`, `WHERE company_id = ? AND name = ? AND id <> ? FOR UPDATE`, sur l'index
  `uq_fiscal_years_company_name`, `crates/kesh-db/test-schema/*.sql:593`).
  - La clôture tient ses antérieurs puis l'exercice. Renommer un exercice au nom d'un antérieur pendant sa clôture ferme le
    cycle.
  - Depuis la 33-1a, la validation d'une facture fournisseur tient son exercice, puis demande les autres exercices de son
    année : même cycle quand deux exercices commencent la même année.
  - Victime, le renommage rend **500**, alors que le pré-contrôle l'aurait refusé en 400 (doublon de nom). Aucune donnée
    n'est faussée : la transaction est annulée.
  - Le registre le dit : `crates/kesh-api/tests/audit_route_registry.rs:125-133`, point (iv). La 33-1a l'a écrit à
    C-e331-2 (`e33-choix-autonomes-e331.md:21`).
- **Le rejeu est sûr.**
  - `update_name` est **une** transaction : verrou, contrôle, `UPDATE`, relecture, audit `fiscal_year.updated`, commit.
  - Ses refus métier sortent en `DbError::Invariant` (`:556-560`, `:599`), et `NotFound` après un `rollback` (`:574-578`).
  - Le prédicat de `retry_on_deadlock` ne rejoue que la 1213 (`kesh-db/src/retry.rs:151-157,172-175`).
  - Le nom (`String`) est cloné à chaque tentative, comme `motif` à la réouverture (`fiscal_years.rs:407-418`).
- **La preuve a un patron déterministe** : `fiscal_year_reopen_is_replayed_when_it_is_the_deadlock_victim`
  (`crates/kesh-api/tests/rejeu_interblocage_e2e.rs:2486-2529`). Il emploie `transaction_lourde`, `victime` (`:346-367`),
  `exercice_sql` (`:1346`) et `CaptureRejeu`. La route attend sur un motif de requête, la lourde ferme le cycle, et la route
  est la victime.
- **Le registre des routes.** La colonne de la route reste `SansEcritureAuJournal` (`audit_route_registry.rs:514`). Seule la
  prose change : le point (iv) cesse de nommer le renommage comme candidat, et le point (vi) l'ajoute aux routes rejouées
  quand même, « vingt-quatre » devenant « vingt-cinq » (`:159`). **Recompter** cette liste à la source avant d'écrire le
  nombre.
- **L'API documentée.** `docs/api-external.md:1036` liste les routes rejouées, dont « la création et la clôture d'un exercice,
  `POST /fiscal-years` et `POST /fiscal-years/{id}/close` ». `PUT /fiscal-years/{id}` est ouverte aux clés (`:235`) : elle
  s'ajoute.
- `services_motif.rs:2408-2422` liste `update_fiscal_year` parmi les handlers sur `state.pool` : inchangé, la route garde
  `state.pool`.

## Acceptance Criteria

1. **AC-1 — le défaut de l'issue, déterministe.** Les exercices M (2010) et N (2011), posés par `exercice_sql`
   (`rejeu_interblocage_e2e.rs:1346-1360`), qui nomme N `"Exercice 2011 (rejeu)"`. Une transaction lourde tient N
   `FOR UPDATE`. `PUT /fiscal-years/{M}` avec ce nom attend sur la lecture de doublon (motifs de `victime` : `AND name = ?` et
   `FOR UPDATE`), puis la lourde demande M.
   - Attendu : **400** « Un exercice avec ce nom existe déjà » (`error-fiscal-year-name-duplicate`), aucune entrée d'audit
     `fiscal_year.updated`, un rejeu capturé sous `"fiscal_years::update_name"`.
   - **Rouge d'avant** : 500, et aucun rejeu.
2. **AC-2 — un renommage valide rejoué aboutit**, par un montage déterministe (validation P2, 34-7-F1).
   - La transaction obtenue par **`transaction_lourde`** (`rejeu_interblocage_e2e.rs:217-233`, 500 lignes de lest, ce qui fait
     de la route la victime, la plus légère) **insère** ensuite, sans la valider, un exercice au nom visé. Une transaction
     légère qui n'insérerait qu'une ligne pourrait devenir la victime à la place de la route (validation P3, 34-7-F1) :
     `INSERT INTO fiscal_years (company_id, name, start_date, end_date, status)`, avec des dates d'une autre année que M et N,
     pour ne pas heurter `uq_fiscal_years_company_start_date`. Le schéma n'a aucun déclencheur sur `fiscal_years`, seulement
     des `CHECK` (`test-schema/0001_schema_squash.sql:585-600`).
   - La route tient M, puis sa lecture de doublon attend le verrou de la ligne insérée.
   - La lourde demande M : cycle, et la route est la victime. La lourde **annule**, si bien que la ligne insérée disparaît.
   - Au rejeu, aucun doublon : **200**, M renommé, **une seule** entrée d'audit `fiscal_year.updated`, un rejeu capturé.
   - ⚠️ **À établir à l'étape 0** : qu'une lecture `FOR UPDATE` sur l'index unique attend bien la ligne insérée non validée,
     et que la route est bien choisie comme victime, la lourde étant plus lourde. Si le montage ne ferme pas le cycle, le
     dire au Change Log et s'en tenir à l'AC-1. Ne pas fabriquer une victime qui ne prouverait rien.
   - *(Le montage d'origine, où la lourde tenait M d'abord, ne pouvait former aucun cycle : la route y attend son premier
     verrou sans rien tenir.)*
3. **AC-3 — rien d'autre ne change.** Les tests existants du renommage passent sans modification : nom vide, trop long,
   doublon, exercice clos renommable, autre société → 404, audit `{before, after}`.
4. **AC-4 — textes.**
   - Le doc-comment de `update_fiscal_year` dit le rejeu et son motif, sur le modèle de `:398-406`.
   - Le registre `audit_route_registry.rs` :
     - le point (iv) (`:125-133`) ne nomme plus le renommage comme candidat ;
     - le point (vi) (`:159`) l'ajoute aux routes rejouées, avec son test, comme la réouverture, et son nombre est recompté
       à la source ;
     - l'en-tête qui énumère les preuves dynamiques (`:88-95`, « tests 2 à 5, 7, 9 et 10 ») est relu ; s'il énumère les
       routes `SansEcritureAuJournal` rejouées de la même manière, le test neuf s'y ajoute.
     - ⚠️ **Ne pas toucher `assert_eq!(examinees, 25)`** (`:1291-1294`) : il compte la colonne `Rejouee`, que cette route ne
       rejoint pas (elle reste `SansEcritureAuJournal`). Le nombre du point (vi) peut lui aussi devenir 25 : c'est une
       coïncidence, et la règle « greper la valeur » ne doit pas propager l'un à l'autre (validation P1, 34-7-F1).
   - `docs/api-external.md:1036` et le CHANGELOG *Corrigé* (`refs #945`) sont mis à jour.

## Tâches

- [ ] **T1 — rouge d'abord** : le test de l'AC-1, `fiscal_year_rename_is_replayed_when_it_is_the_deadlock_victim`, dans
  `rejeu_interblocage_e2e.rs`. Il est rouge (500) avant le correctif ; à journaliser dans `kesh-gate-logs/e343/rouges.log`.
- [ ] **T2** : l'enveloppe dans `update_fiscal_year`, avec `map_update_error` appliqué au résultat de l'enveloppe.
- [ ] **T3** : AC-2, tenté et dit.
- [ ] **T4** : AC-4.

### Mutations nommées

- **M1** : l'enveloppe retirée → rouge AC-1 (500, pas de rejeu).
- **M2** : `retry_on_deadlock_with("fiscal_years::update_name", 1, …)`, soit une seule tentative → rouge AC-1 (500, aucun rejeu
  capturé) et AC-2. *(Remplace la M2 d'origine, `map_update_error` dans la fermeture, qui ne compile pas :
  `retry_on_deadlock` exige un `Future` de `Result<T, DbError>`, `kesh-db/src/retry.rs:151-157`. Validation P2, 34-7-F2.)*

## Gates

Gate ciblé `binary(rejeu_interblocage_e2e)` entre les passes. **Gate complet** avant tout push et au dernier commit de code :
la route est dans `kesh-api` seule, mais le test lit la base. Aucune modification de `kesh-db`. E2E complet au dernier commit
de code : aucun écran ne change, et la règle du `CLAUDE.md` s'applique quand même.

## Ce qui ne se traite pas ici

Les autres candidates du point (iv), sans cycle démontré : `PUT /invoices/{id}` quand le projet change,
`POST /projects/{id}/archive`, les huit routes d'étape de l'onboarding. Elles restent écrites au registre. Aucune issue n'est
à ouvrir : elles sont nommées et assumées.

## Recoupements

- `rejeu_interblocage_e2e.rs` : **34-5** y met la version dans le corps du test de règlement (`:550-623`). Fusion par union,
  sans conflit de ligne attendu.
- `docs/api-external.md:1036` : la même ligne que 34-8 ne touche pas.

## Change Log

- 2026-11-10 : fiche créée (Opus 5.5).
- 2026-11-10 : **validation P1 (Sonnet) : 0 M, 2 L, tous remédiés** (Opus 5.5). Rapport
  `kesh-gate-logs/e34-validate-p1-e343.md`.
  - **F1** : `audit_route_registry.rs:1291-1294` lu (`examinees, 25`, colonne `Rejouee`). La mise en garde est à l'AC-4.
  - **F2** : l'en-tête `:88-95` lu. Sa relecture est ajoutée à l'AC-4, au conditionnel : il énumère des tests de routes
    `Rejouee` et deux routes d'exercice, sans la réouverture.
  - Le rapport confirme M2 (`retry_on_deadlock` générique en `DbError` : sans objet si la forme ne compile pas), sans
    changement.
- 2026-11-10 : **validation P2 (Opus) : 1 M, 1 L, tous remédiés** (Opus 5.5). Rapport
  `kesh-gate-logs/e34-validate-p2-e343.md`. Prémisses relues : `retry.rs:151-157`, schéma de `fiscal_years`
  (`0001_schema_squash.sql:585-600`, aucun déclencheur).
  - **F1 (M), neuf** : le montage de l'AC-2 ne pouvait former aucun cycle. Le montage déterministe proposé, par une ligne
    insérée et non validée, est adopté (arbitrage de l'orchestrateur), avec ses points à établir à l'étape 0.
  - **F2 (L)** : M2 remplacée par une mutation qui compile.
- 2026-11-10 : **validation P3 (Sonnet) : 0 M, 1 L, remédié** (Opus 5.5). **F1** : l'AC-2 nomme `transaction_lourde`
  (`:217-233` lu).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
