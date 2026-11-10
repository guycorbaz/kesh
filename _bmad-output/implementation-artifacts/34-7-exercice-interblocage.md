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
2. **AC-2 — un renommage valide rejoué aboutit.** Même montage, mais la route renomme M en un nom libre et la lourde tient M
   d'abord.
   - Si ce montage ne ferme pas le cycle de façon déterministe (un verrou d'intervalle sur un nom libre ne bloque pas un autre
     verrou d'intervalle), **le dire au Change Log et s'en tenir à l'AC-1**. L'AC-1 prouve à lui seul que l'enveloppe est
     posée.
   - Ne pas fabriquer une victime qui ne prouverait rien.
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
- **M2** : `map_update_error` appliqué **dans** la fermeture, avant l'enveloppe : l'`Invariant` devient une `AppError`.
  Vérifier que le prédicat `DbError` ne voit plus la 1213 → rouge AC-1. Si la forme ne compile pas, la mutation est sans
  objet, et cela s'écrit au Change Log.

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

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
