# Story 15.8b : Supprimer une écriture tant que son exercice est ouvert, et l'historique sur la fiche

Status: done

<!-- Issue de la story 15-8, DÉCOUPÉE le 2026-10-08 après la validation P2 (choix C-15-8-17 de
     `epic-15-choix-autonomes.md`). Elle porte la SUPPRESSION, l'historique visible sur la fiche et le retrait
     d'`ENTRY_IS_POSTED`. Choix applicables : C-15-8-1, C-15-8-2, C-15-8-5 (posé par la 15-8a, employé ici),
     C-15-8-6, C-15-8-7, C-15-8-8, C-15-8-12, C-15-8-13, C-15-8-14, C-15-8-17, C-15-8-19, C-15-8-20, et ceux de la
     validation P3 : C-15-8-22 à C-15-8-27 et C-15-8-29 (C-15-8-23 corrige C-15-8-19 ; C-15-8-24 corrige
     C-15-8-5, C-15-8-10, C-15-8-12 et C-15-8-13 ; C-15-8-28, la dérogation, ne vaut que pour la 15-8a). Historique
     des passes P1 et P2 : Change Log de l'index `15-8-modifier-une-ecriture.md` ; P3 : Change Log de cette fiche. -->

**Issue : [#532]** — **`refs #532`** : l'issue a été **fermée par la 15-8a** (PR #553, `52a9b19b`) ; la PR de cette story la référence sans la fermer (clôture de la validation, 2026-10-08).

**Dépendances** — ⛔ **la 15-8a passe AVANT** (`15-8a-modifier-une-ecriture.md`) ; la 15-5a est **mergée**
(`b11a074a`). Cette story **réutilise** ce que la 15-8a pose, sans le redéfinir : le cadre D1 (dont le paiement détaché
et l'exercice postérieur clos), la garde `modification_guard(conn: &mut MySqlConnection, …)` / `modification_refusal`,
`ModificationGuard`, `DbError::EntryNotModifiable`, `DbError::LaterFiscalYearClosed`, `ModificationBlocker` et les
trois champs du détail, la table de correspondance code d'écran ↔ refus (onze codes), le formulaire et la fiche. Elle
se branche sur `main` **après le merge de la 15-8a**. ⚠️ **Les numéros de ligne ci-dessous sont ceux de `main` avant
la 15-5a** : la table des décalages mesurés au rebase sur la 15-5a est en T0 de la 15-8a (`journal_entries.rs` : `+10`
à partir de `:82` ; `kesh-db/src/errors.rs` : `+115` pour les variantes, `+122` pour `code()`) ; la 15-8a les décalera
encore — tout se **revérifie** (T0). ⛔ **À la clôture de la validation, les sites décisifs ont été remesurés sur
`52a9b19b` (15-8a mergée)** : table « Décalages mesurés après la 15-8a » en tête des Dev Notes. Elle fait foi sur les
numéros de ligne de cette fiche.

## Story

**As a** personne qui tient les livres,
**I want** pouvoir supprimer une écriture saisie à la main — l'écriture d'ouverture comprise — dans le même cadre que
la modification, et voir sur sa fiche qu'elle a été modifiée et par qui,
**so that** une écriture saisie par erreur disparaît sans écriture inverse, sans que sa disparition ni ses corrections
cessent d'être apparentes.

## Ce que cette story ajoute à la 15-8a

| geste | cadre |
|---|---|
| **`DELETE /api/v1/journal-entries/{id}`** | le cadre D1 de la 15-8a, à l'identique : exercice ouvert, **aucun exercice postérieur clos** (C-15-8-22), ni contre-passée ni contre-passation, aucune pièce, pas un paiement détaché, date postérieure à `books_locked_through` ; trace `journal_entry.deleted` avec l'instantané complet et l'acteur (utilisateur **ou clé d'API**) |
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
premier acte de `delete_in_tx` est son `FOR UPDATE` joint (écriture **et** exercice, étape 2, `:1014-1019`) ; la
première lecture ordinaire — `reversed_by`, étape 3-bis (`:1043`) — vient après, la borne de verrou ensuite (requête
`:1075-1081`). **T1 : le doc-comment le dit** (finding R3-5 — il ne le dit pas aujourd'hui) ; le test de concurrence de
l'AC 5 le tient.

⛔ **L'exercice postérieur clos** (C-15-8-22, finding F1 de la validation P3 ; cadre D1 de la 15-8a). Le bilan est
cumulatif (`balance_sheet.rs:9-11`) et `fiscal_years::close` n'exige pas que l'exercice précédent soit clos : supprimer
une écriture de N réécrirait le bilan d'un N+1 clos. **Étape 2-bis**, neuve, **juste après** le `FOR UPDATE` joint et
**avant** toute lecture ordinaire (3-bis ouvre la vue) : le `SELECT` joint gagne `fy.start_date`, puis
`fiscal_years::find_later_closed_in_tx(tx, company_id, fy.start_date)` (verrouillante, état courant — la garde LIFO de
`reopen`) ; un exercice trouvé → `DbError::LaterFiscalYearClosed` (400 `LATER_FISCAL_YEAR_CLOSED`, posée par la 15-8a).
⚠️ **Seulement si `enforce_ownership`** (C-15-8-29) : la dévalidation (`invoices::unvalidate`, `enforce_ownership =
false`) garde son comportement (AC 3) — le même trou y existe, défaut préexistant **signalé pour une issue** avec la
création et le règlement (15-8a, Hors périmètre).

| appelant | valeur | effet |
|---|---|---|
| `delete_by_id` (la route) | `true` | étape 3-ter : `modification_guard` → `modification_refusal` |
| `invoices::unvalidate` (`invoices.rs:1654`) | `false`, `actor_api_key_id = None` | la facture supprime **sa** propre écriture — la garde `OwnedByInvoice` n'a pas de sens ici. ⚠️ `unvalidate` ne reçoit aujourd'hui aucun `actor_api_key_id` (`invoices.rs:1486-1492`) : **hors périmètre**, signalé à l'orchestrateur |

Précédence (inchangée hormis 2-bis et 3-ter) : `FiscalYearClosed` (400) → **`LaterFiscalYearClosed`** (400, étape
2-bis, si `enforce_ownership`) → `EntryIsReversed` (409, étape 3-bis, existante) → **garde de la 15-8a** (409, si
`enforce_ownership` ; `Owned { AlreadyReversed }` n'y arrive jamais, l'étape 3-bis l'a pris) → `PeriodLocked` (400) →
instantané, audit `journal_entry.deleted`, `DELETE` (CASCADE). ⛔ **Cette précédence est testée paire par paire**
(AC 4-bis) : aujourd'hui, **un seul** test tient un ordre — `le_gel_parle_avant_le_verrou_de_periode`
(`journal_entries.rs:2509`), que le retrait du gel vide de son sens (finding R3-1 / F2).

⛔ **Trois refus que la base n'apporterait pas — la garde est la seule barrière** :

- **la contre-passation elle-même** (`IS_A_REVERSAL`) : la clé étrangère `RESTRICT` ne protège que l'**origine** —
  rien ne s'opposerait à l'effacement d'une contre-passation ;
- **le paiement détaché** (`DETACHED_SUPPLIER_SETTLEMENT`, 15-8a D1) : aucune colonne ne le référence plus ;
- ⛔ **l'écriture rapprochée** (`MATCHED_BANK_TRANSACTION`) — le pire des trois (finding F5) :
  `bank_transactions.matched_entry_id` est en **`ON DELETE SET NULL`** (`fk_bank_transactions_matched_entry`,
  `test-schema/0001_schema_squash.sql:230` ; `reconciliation_cancel.rs:20-22` : « la FK est ON DELETE SET NULL, le lien
  s'effacerait en silence »). Sans la garde, la suppression **réussirait** et laisserait la transaction bancaire
  `reconciled` **sans lien**, en silence.

Les trois sont testés (AC 2, AC 4) ; pour le rapprochement, l'AC 4 asserte **en plus** que la ligne
`bank_transactions` est intacte (`matched_entry_id` inchangé, statut inchangé). *(Les autres clés étrangères entrantes
sont `RESTRICT` — `squash:486`, `:696`, `:746`, `:779`, `:1082`, `:1085` — ou `CASCADE`, `:802`, les lignes.)*

⚠️ **Le numéro n'est jamais réattribué** (compteur de la 25-2-c) : la suppression creuse un **trou**, expliqué par
l'instantané `journal_entry.deleted`. Le manuel (§ Numérotation) doit le dire — il affirme aujourd'hui qu'« une
écriture ne se supprime plus ».

### D2 — Rejeu sur interblocage, comme le `PUT`

Le handler `DELETE` est enveloppé dans `retry_with(DEFAULT_MAX_DEADLOCK_ATTEMPTS, is_deadlock_error, …)`
(C-15-8-19), comme le `PUT` de la 15-8a — **par uniformité, pas pour un cycle connu** (finding F7). Son ordre de
verrous est **écriture et exercice ensemble** (jointure) → exercices postérieurs (étape 2-bis) → au `DELETE`, les
contrôles des clés étrangères qui visent l'écriture et l'intervalle de ses lignes. Il ne fait **aucun `INSERT`** hors
de l'audit : il ne prend ni la sentinelle `companies`, ni un projet, ni un compte (vérifié : `delete_in_tx` ne porte
que des `SELECT`, l'`INSERT` d'`audit_log` et des `DELETE`, `:1005-1132`). **Aucun cycle connu** à ce jour — la phrase
de la rédaction P2, « il entre dans les cycles par les exercices que la 25-7 nomme », n'était étayée nulle part ; elle
est retirée. Le rejeu coûte trois lignes, la transaction est rejouée entière, sans effet hors d'elle : il protège d'un
cycle **futur** pour un coût négligeable. La ligne de Pattern 5 posée par la 15-8a (`docs/MULTI-TENANT-SCOPING-PATTERNS.md`,
« Deny list ») s'étend au `DELETE` en le disant : ordre, « aucun cycle connu », rejeu par uniformité.

### D3 — L'écriture d'ouverture supprimée

| geste | effet sur l'écran « Soldes de départ » |
|---|---|
| **supprimer** l'ouverture, seule écriture de la société | la société redevient vierge → **la génération est de nouveau proposée** (statut `READY`) — exactement la procédure que la 24-4b avait fermée (24-4b D7). ⚠️ La nouvelle ouverture reçoit le **numéro 2**, pas 1 : le compteur de la 25-2-c ne réattribue jamais un numéro (`un_numero_libere_n_est_jamais_reattribue`, `journal_entries.rs:2087`). **Accepté** (C-15-8-12) : le trou est expliqué par l'instantané `journal_entry.deleted` ; le manuel (§ soldes de départ) le dit, l'AC 7 l'asserte |
| supprimer l'ouverture quand d'autres écritures existent | génération toujours refusée ; ses comptes sans autre mouvement deviennent complétables, et le complément est daté du premier jour du premier exercice s'il est ouvert et hors période verrouillée (25-7, arbitrage 2). ⚠️ **Un compte de l'ouverture qui a d'autres mouvements** (typiquement la banque) **ne redevient pas complétable** (`count_by_company > 0` refuse la génération, `opening_balances.rs:262-266` ; le complément n'offre que les comptes jamais mouvementés) : son solde de départ ne se ressaisit qu'à la main, par une écriture d'OD (finding F10). Le manuel (§ soldes de départ) le dit ; la confirmation de suppression ne le dit pas (l'ouverture n'est pas identifiable structurellement) |

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
   'api_key'`, `actor_api_key_id`) ; la **création suivante** du même exercice ne reprend **pas** le numéro (compteur
   25-2-c).
2. **Supprimer une contre-passation** → 409 `IS_A_REVERSAL` (D1 : la base ne l'aurait pas refusé).
3. **Dévalidation inchangée** : `invoices::unvalidate` (`enforce_ownership = false`) supprime toujours l'écriture de
   sa facture — ses tests existants restent verts, sans réécriture.
4. **Gardes** : pour **chacune** des pièces de l'AC 6 de la 15-8a (facture, avoir, facture fournisseur — achat et
   règlement —, règlement client, solde `write_off`, transaction bancaire rapprochée) le `DELETE` rend **409 sous le
   code du motif** avec `details.documentId`, sur le **même montage** (`monter_les_pieces`, celui
   d'`every_document_owned_entry_is_refused` et de son pendant `PUT` de la 15-8a) ; une écriture contre-passée → 409 `ENTRY_IS_REVERSED` ; le **paiement détaché** d'une facture
   fournisseur annulée → 409 `DETACHED_SUPPLIER_SETTLEMENT` ; **exercice clos seul** → 400 `FISCAL_YEAR_CLOSED` ;
   **exercice postérieur clos seul** (N ouvert, N+1 clos) → 400 `LATER_FISCAL_YEAR_CLOSED` nommant N+1, et 204 après
   réouverture de N+1 (C-15-8-22) ; date ≤ borne (seuil inclusif) → 400 `PERIOD_LOCKED`. Après chaque refus, lignes et
   en-tête identiques. **Écriture rapprochée** : en plus du 409, la ligne `bank_transactions` est **intacte**
   (`matched_entry_id` et statut inchangés) — la clé étrangère est `ON DELETE SET NULL`, la garde est la seule barrière
   (D1, finding F5).
4-bis. **Précédence du `DELETE`** (findings R3-1, F2 de la validation P3) — comme l'AC 7 de la 15-8a pour le `PUT`,
   chaque paire montée avec **les deux** causes, au niveau de l'API **et** dans `mod tests` :
   - **pièce et période verrouillée** — une écriture **de facture** datée ≤ `books_locked_through` → **409
     `OWNED_BY_INVOICE`**, pas 400 `PERIOD_LOCKED` (le cas de **toute** facture ancienne ; c'est aussi ce que l'écran
     affiche, `modification_blocker` rendant la pièce) ;
   - **exercice clos et pièce** → **400 `FISCAL_YEAR_CLOSED`** ;
   - **exercice clos et exercice postérieur clos** → `FISCAL_YEAR_CLOSED` ; **exercice postérieur clos et pièce** →
     `LATER_FISCAL_YEAR_CLOSED` ; **exercice postérieur clos et contre-passée** → `LATER_FISCAL_YEAR_CLOSED` ;
   - **exercice clos et contre-passée** → 400 (le test e2e `a_closed_fiscal_year_answers_before_both_conflicts`,
     réécrit, T6) ;
   - **contre-passation (`IS_A_REVERSAL`) et période verrouillée** → `IS_A_REVERSAL`.
   Dans `mod tests`, **un test d'ordre** remplace `le_gel_parle_avant_le_verrou_de_periode` :
   `la_garde_parle_avant_le_verrou_de_periode` — une écriture **de facture** sous la borne, `enforce_ownership = true`
   → `EntryNotModifiable(Owned { OwnedByInvoice, .. })`, l'écriture reste. ⛔ **Mutation à tuer** (déclarée au Dev
   Agent Record) : **permuter les étapes 3-ter et 3-quater** (garde et verrou de période) — ce test et la paire « pièce
   et période verrouillée » doivent rougir ; et **permuter 2-bis et 3** — la paire « exercice clos et exercice
   postérieur clos » doit rougir.
5. **Concurrence** — `delete_waits_for_a_concurrent_reversal_then_refuses`, dans
   `crates/kesh-db/tests/journal_entries_modification.rs` (le fichier de la 15-8a) : B contre-passe E par
   `reverse_in_tx` sans commiter ; A lance `delete_by_id(E)` ; le test attend A bloquée sur son verrou, avec les motifs
   **du `SELECT` de `delete_in_tx`** (`:1014-1019` : `["je.fiscal_year_id", "FOR UPDATE"]` — pas ceux du `PUT`, que ce
   `SELECT` ne contient pas : `attendre_une_requete_en_cours` paniquerait à 10 s, `test_fixtures.rs:585-588`) ; B
   commite ; A rend `EntryIsReversed` et E est intacte. ⛔ **Mutation à tuer** (déclarée au Dev Agent Record) :
   placer une lecture ordinaire (la borne) avant le `FOR UPDATE` — la vue se fige avant le commit de B, `reversed_by` ne
   voit pas la contre-passation, la garde non plus, et A **ne rend plus `EntryIsReversed`** : elle bute sur la clé
   étrangère `RESTRICT` de `reverses_entry_id` (`squash:779`, erreur 1451) — le test, qui exige le **code**, rougit
   (findings R3-2, F6 : la rédaction P2 disait à tort « fait passer A »). ⚠️ Ce test prouve la sérialisation **parce
   qu'il exige le code** ; pour les motifs **sans** clé étrangère `RESTRICT` (paiement détaché, rapprochement en
   `SET NULL`), la propriété repose sur le même verrou et la même règle, sans test de concurrence propre — limite
   nommée. **Exercice postérieur clos sous concurrence** : le pendant de l'AC 8 de la 15-8a pour le `DELETE`
   (`delete_waits_for_a_concurrent_close_of_a_later_year_then_refuses`, motifs du `SELECT` de
   `find_later_closed_in_tx`), avec sa mutation (lecture ordinaire des exercices postérieurs).
6. **IDOR, rôles, rejeu** : `DELETE` sur un `id` d'une autre société ou inexistant → **404**, jamais 409 ;
   Consultation → **403** avant tout autre contrôle ; le handler est enveloppé dans `retry_with`
   (`grep -nF "retry_with" crates/kesh-api/src/routes/journal_entries.rs` rend le `PUT` **et** le `DELETE`) ; Pattern 5
   nomme le `DELETE`.
7. **Soldes de départ** (D3) : après **suppression** de l'ouverture seule écriture de la société, le statut redevient
   `READY` et une nouvelle génération réussit, sous le **numéro 2** (asserté — C-15-8-12) ; après suppression de
   l'ouverture quand d'autres écritures existent, ses comptes sans autre mouvement figurent dans `completableAccounts`.
   Une écriture de **complément** (25-7) se supprime (204).
8. **Détail** : la table de correspondance de la 15-8a (AC 12) gagne la colonne `DELETE` — pour **chacun** des onze
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
    — et trier à la main (leçon de la 24-4b). **Sortie attendue, liste fermée** (finding F4 ; **remesurée sur
    `52a9b19b`**, finding M1 de la validation P4 — C-15-8b-1). **Restent**, et eux seuls : des fichiers sous
    `_bmad-output/` (story files historiques, registre des choix, `sprint-status.yaml`, prompts de validation, ces
    fiches — qui **ne se réécrivent pas** quand ce sont des archives) ; dans `CHANGELOG.md`, la seule section
    `## [0.13.0]`, qui dit `ENTRY_IS_POSTED` retiré ; `docs/api-external.md:221` — parenthèse **historique** posée par la
    15-8a (« de la v0.12.0 à la v0.12.1, ce `PUT` rendait toujours 409 `ENTRY_IS_POSTED` »), un fait passé qui
    **reste** (et sa jumelle pour le `DELETE`, si T7 l'écrit) ; et `frontend/src/lib/shared/i18n-keys.test.ts:174` —
    paragraphe **historique** de la ventilation (« 1630 → 1619 à la 24-4b »), qui **reste**. **Doivent avoir disparu**
    (inventaire de `52a9b19b` : **38** lignes hors `_bmad-output/` — 3 qui restent ci-dessus, `CHANGELOG.md:15`
    [0.13.0] — l'entrée de la 15-8a, dont la mention « que le `DELETE` rend encore » est réécrite —, `api-external.md:221`
    et `i18n-keys.test.ts:174` ; **35** qui disparaissent) :
    `kesh-db/src/errors.rs:884`, `:886`, `:906`, `:1035` ; `kesh-api/src/errors.rs:3069`, `:3123`, `:3125`, `:3127` ;
    `kesh-db/src/repositories/journal_entries.rs:1542`, `:1551-1552`, `:1571`, `:1589`, `:1592`, `:1621`, `:1666-1667`,
    `:1678`, `:2731`, `:3008`, `:3032`, `:3056`, `:3118`, `:3124-3125` ; `kesh-db/src/repositories/invoices.rs:1442`,
    `:1650` ; la clé dans les quatre `messages.ftl` (fr `:361`, de/en/it `:367`) ;
    `kesh-api/tests/journal_entry_reversal_e2e.rs:2260` (`deleting_a_posted_entry_is_refused`), `:2360` (message
    d'assertion de `a_reversed_entry_answers_reversed_not_posted`), `:2474` (moitié `DELETE` de
    `the_opening_entry_is_modifiable_and_still_reversable`) — tests réécrits, T6 ; et
    `frontend/src/lib/features/journal-entries/journal-entries.api.ts:70` — ⛔ **tranché : DEDANS, à faire disparaître.**
    L'inversion de la 15-8a **n'a pas effacé** ce site : elle a rétabli `deleteJournalEntry` avec un doc-comment qui dit
    « Sans appelant dans la 15-8a : le `DELETE` rend encore 409 `ENTRY_IS_POSTED` » (`:68-71`). Cette story lui donne
    son appelant (D4) et rend la phrase fausse : le doc-comment est **réécrit** (T4). ⚠️ `frontend/tests/e2e/journal-entries.spec.ts:285`,
    que la rédaction P3 listait, **ne rend plus rien** : la 15-8a a reformulé le bloc (`:278-290`) — il sort de la liste.
    Toute autre ligne est un résidu.
11. **Audit** : `journal_entry.deleted` reste dans `ACTIONS` ; `audit_route_registry.rs:83` garde le `DELETE` à `Traced`.
12. **Documentation** : manuel utilisateur et administrateur, `docs/api-external.md`, `CHANGELOG.md` et
    `README.md:29` (C-15-8-26) disent la suppression (cf. Dev Notes) ; PDF régénérés. ⛔ **Contrôle** (C-15-8-27,
    findings R3-6, F9) — **sur les deux PDF aplatis, qui font foi** (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' ' |
    grep -oiE "<motif>"` : le balisage TeX coupe les tournures dans le `.tex`, et `grep -o` compte les occurrences),
    et sur `crates/kesh-i18n/locales/fr-CH/messages.ftl` :
    `écriture ne se supprime plus|une seule voie fait encore disparaître|ni modifiable ni supprimable|ni modifiée ni supprimée|ne se modifie ni ne se supprime|ni modification ni suppression`
    ne rend **rien**, **et** le motif de l'AC 17 de la 15-8a ne rend toujours **rien** ; passages conservés nommés au Dev
    Agent Record. **Ligne de base** (PDF commités, avant la 15-8a, `grep -o` exécuté le 2026-10-08) : **5**
    occurrences côté utilisateur, **2** côté administrateur, **0** dans le `.ftl` — dont `user-manual.tex:481` (« ne
    peut plus être ni modifiée ni supprimée »), qu'aucun des deux motifs de la rédaction P2 ne voyait (finding R3-6). ⚠️ **Tournure prescrite pour le paiement détaché**
    (`user-manual.tex:1331-1337`, FAQ `:2140`) : « La fiche de ce paiement n'offre ni « Modifier » ni « Supprimer » :
    elle affiche le motif, et la contre-passation reste disponible. » — la tournure naturelle « ne se modifie ni ne se
    supprime » est **exactement** un motif, un manuel juste ferait rougir le contrôle (finding F9). Même règle pour le
    texte de la clé `journal-entries-modify-blocked-detached-settlement` (Dev Notes, i18n). ⚠️ `user-manual.tex:1213`
    « Une facture validée, elle, ne se supprime plus » est **vrai** et le motif ne le prend pas (« **écriture** ne se
    supprime plus »).

## Tâches

- [x] **T0 — Branche et inventaire au sol** (AC 10, 12) : brancher sur `main` après le merge de la 15-8a ; revérifier
      les numéros de ligne cités ; exécuter les greps des Dev Notes et relever leur **sortie complète** au Dev Agent
      Record, triée ligne par ligne.
- [x] **T1 — `delete_in_tx`** (AC 1–5, 4-bis) : `enforce_immutability` → `enforce_ownership`, étape 3-ter = garde de
      la 15-8a (`&mut **tx`) ; étape 2-bis neuve — `fy.start_date` au `SELECT` joint, `find_later_closed_in_tx` si
      `enforce_ownership` (C-15-8-22, C-15-8-29) ; tests de précédence de l'AC 4-bis et leurs deux mutations ; `actor_api_key_id` sur `delete_by_id` et `delete_in_tx`, audit par `for_actor` ; `invoices::unvalidate`
      passe `None` ; doc-comments de `delete_by_id` (`:924-949`) et `delete_in_tx` (`:967-1004`) **réécrits** (⛔ la
      24-4b a payé un MEDIUM pour un doc-comment resté à l'octet près) ; commentaires d'`invoices.rs:1442` et `:1650` ;
      en-tête de module (`:1-40`, « corriger par contre-passation plutôt que par suppression ») revu ; appelants de test
      de `delete_in_tx` dans `mod tests` (`:2123`, `:2182`, helper `:2439`) mis à la nouvelle signature (finding F12) ;
      tests de concurrence (AC 5, dont l'exercice postérieur clos) et leurs mutations
- [x] **T2 — Retrait du gel** (AC 10) : `EntryIsPosted` et son doc, sa branche `kesh-api` et son commentaire
      (`:2986-2996`, ce qu'il en reste), la clé ×4, doc de `PeriodLocked`
- [x] **T3 — Route** (AC 1, 6, 8) : `api_key_id` passé au `DELETE` ; handler enveloppé dans `retry_with` ;
      doc-comment du `DELETE` (`:609-610`, « asymétrie volontaire avec UPDATE ») revu ; tests de clé d'API (AC 1),
      colonne `DELETE` de la table de correspondance (AC 8), paires de précédence au niveau de l'API (AC 4-bis)
- [x] **T4 — Écran** (AC 9, 10) : « Supprimer », confirmation, « Modifiée », « Historique », rôle Consultation ;
      doc-comment de `deleteJournalEntry` (`journal-entries.api.ts:67-71`) réécrit — il annonce encore le 409 du gel ; clés i18n ×4
      (liste fermée de D4) ; `i18n-keys.test.ts` (`ATTENDU.sitesTotal`) **avec sa ventilation** ;
      `e2e-selecteurs-traduits.test.ts` selon les sélecteurs réellement employés
- [x] **T5 — Playwright** (AC 9) — `tests/e2e/journal-entries.spec.ts`, parcours **depuis la fiche** remplaçant les
      specs de liste de suppression retirées par la 24-4b :
  - [x] « suppression avec confirmation » → `supprimer depuis la fiche : confirmation, retour à la liste, l'écriture a disparu`
  - [x] « annulation suppression » → `annuler la suppression : la fiche reste, l'écriture aussi`
  - [x] neufs : `après modification : « Modifiée » et « Historique », qui ouvre le journal d'audit filtré` ; `rôle
        Consultation : ni Supprimer ni Historique` ; `écriture de facture : ni Modifier ni Supprimer, le motif est affiché`
        (étend la spec de la 15-8a)
- [x] **T6 — Tests qui changent de sens** (liste ci-dessous) : réécrits, **pas** supprimés en bloc ; chaque test retiré
      nommé au Dev Agent Record avec son remplaçant
- [x] **T7 — Documentation** (AC 12) — cf. Dev Notes ; `make fr`, PDF commités, contrôle de l'AC 12 **sur les PDF
      aplatis** (il fait foi) et sur `fr-CH/messages.ftl`, table des matières ; `README.md:29` (C-15-8-26)
- [x] **T8 — Gates** (⛔ complets — exception `kesh-db`)
  - [x] base remise à zéro (KF-039), `scripts/test-fast.sh` sous `mem-guard`
  - [x] `npm run check` · `lint-i18n-ownership` · `test:unit` · `build`
  - [x] suite Playwright **complète au dernier commit de code**, jugée fichier par fichier contre `docs/testing.md`
        § « Les échecs attendus »

## Hors périmètre

- Tout ce que la 15-8a porte (modification, garde, formulaire, motif d'écran).
- **Audit par clé d'API de la dévalidation** (`invoices::unvalidate`), de la création et de la contre-passation :
  défaut préexistant, signalé à l'orchestrateur pour une issue.
- **Réinitialiser le compteur** quand la société redevient vierge : non (C-15-8-12).

## Dev Notes

### Décalages mesurés après la 15-8a (`52a9b19b`, clôture de la validation)

Les numéros cités dans cette fiche ont été écrits sur `main` **avant** la 15-5a et la 15-8a. Remesurés à la clôture de
la validation ; **cette table fait foi** (T0 la revérifie sur la branche).

| site cité dans la fiche | sur `52a9b19b` |
|---|---|
| `kesh-db/src/errors.rs:626-638`, `:647`, `:768` (`EntryIsPosted`, doc de `PeriodLocked`, `code()`) | `:884-886` (doc et variante), `:906` (doc de `PeriodLocked`), `:1035` (bras de `code()`) |
| `kesh-api/src/errors.rs:2986-2996`, `:3042-3050`, `:3046` | commentaire `:3064-3072`, branche `:3123-3130` (clé `:3127`) |
| `journal_entries.rs:924-949` (doc de `delete_by_id`) | `:1535-1560` ; fonction `:1561-1576` |
| `journal_entries.rs:967-1004` (doc de `delete_in_tx`) | `:1578-1615` ; fonction `:1616-1742` |
| `journal_entries.rs:1005` / `:1014-1019` (`SELECT … FOR UPDATE` joint) | `:1616` / `:1625-1631` |
| `:1043` (3-bis, `reversed_by`) · `:1047-1066` (3-ter, gel) · `:1075-1081` (borne) · `:1114` (`NewAuditLogEntry::user`) | `:1654` · `:1658-1668` · `:1686-1692` · `:1725` |
| `journal_entries.rs:2087` (`un_numero_libere_n_est_jamais_reattribue`) | `:2698` |
| `mod tests` : `:2120-2121` (commentaire), `:2123`, `:2182` (appelants), helper `:2396-2444`/`:2439`, `:2465`, `le_gel_parle_avant_le_verrou_de_periode` `:2509` | `:2731`, `:2734`, `:2794` ; helper `supprimer_sous_borne` `:3030-3060` (doc `:3008`) ; `:3118-3125` ; `le_gel…` `:3121` |
| garde de la 15-8a : `modification_guard` / `modification_refusal` | `journal_entries.rs:998` / `:1047` ; `fiscal_years::find_later_closed_in_tx` `fiscal_years.rs:639` |
| `invoices.rs:1442`, `:1650`, `:1654` (appel de `delete_in_tx`), `unvalidate` `:1486-1492` | **inchangés** |
| `kesh-api/src/routes/journal_entries.rs:609-623` (handler `DELETE`) | `:700-716` ; `retry_with` du `PUT` `:675` |
| `kesh-api/tests/journal_entry_reversal_e2e.rs` : `:727` (`deleting_a_reversed_entry_is_refused_but_bulk_delete_still_works`) · `every_document_owned_entry_is_refused` | `:827` · `:714` |
| idem : `:1231` (`deleting_a_posted_entry_is_refused`) · `:1317`/`:1331` · `:1341` · `:1445` | `:2252` · `:2346`/`:2360` · `:2370` · `:2488` |
| idem : `the_opening_entry_is_frozen_but_still_correctable` (`:1386`) | ⚠️ **renommé par la 15-8a** `the_opening_entry_is_modifiable_and_still_reversable` (`:2419`) ; sa moitié `DELETE` encore refusée `:2472-2474` |
| `kesh-api/tests/audit_route_registry.rs:83` (`DELETE` → `Traced`) | `:84` |
| `frontend/…/journal-entries.api.ts:58` | `:67-74` (doc-comment `:67-71`, AC 10) |
| `frontend/tests/e2e/journal-entries.spec.ts:285` | bloc `:278-290`, reformulé par la 15-8a — ne contient plus `ENTRY_IS_POSTED` |
| manuels (`user-manual.tex`, `admin-manual.tex`) | ⚠️ **non remesurés ici** : la 15-8a les a réécrits — les sites se retrouvent par les motifs de l'AC 12 (T0/T7) |

### Les sites de la suppression — inventaire au sol (2026-10-08)

Commandes : celles de la 15-8a (T0). Sites de la **suppression** :

**Code**

| site | ce qu'il dit / fait | à faire |
|---|---|---|
| `kesh-db/src/errors.rs:626-638`, `:647`, `:768` | variante `EntryIsPosted`, son doc, sa mention dans le doc de `PeriodLocked` | retirer ; doc de `PeriodLocked` reformulé |
| `kesh-db/src/repositories/journal_entries.rs:924-1132` (`delete_by_id`, `delete_in_tx`, dont `:931`, `:939-940`, `:960-961`, `:978-996`, `:1047-1066`) | gel | réécrire (T1) |
| `journal_entries.rs:1-40` (en-tête de module) | « corriger par contre-passation plutôt que par suppression » | revu |
| `journal_entries.rs` `mod tests` : `le_gel_parle_avant_le_verrou_de_periode` (~`:2509`), le helper qui passe `enforce_immutability` (~`:2396-2444`), `:2465`, commentaire `:2120-2121` | test du gel — le **seul** test d'ordre du `DELETE` | ⛔ **remplacé par un test d'ordre**, pas réécrit en test de refus seul : `la_garde_parle_avant_le_verrou_de_periode` (écriture **de facture** sous la borne → `EntryNotModifiable(Owned { OwnedByInvoice, .. })`, AC 4-bis) ; un test séparé garde le cas « écriture manuelle de période verrouillée → `PeriodLocked` » (findings R3-1, F2 : la réécriture de la rédaction P2 ne tenait plus aucun ordre) |
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
| `a_reversed_entry_answers_reversed_not_posted` (`:1317`, message `:1331` « pas ENTRY_IS_POSTED ») | distingue `ENTRY_IS_REVERSED` du gel sur le `DELETE` | ⛔ **devient muet** au retrait du gel (une écriture contre-passée n'est pas « possédée » : rien ne reste à distinguer) → **renommé et réécrit** `a_reversed_entry_answers_reversed_on_put_and_delete` : `PUT` **et** `DELETE` → 409 `ENTRY_IS_REVERSED`, message d'assertion sans `ENTRY_IS_POSTED` (findings R3-1, F4) |
| `a_closed_fiscal_year_answers_before_both_conflicts` (`:1341`) | « l'exercice clos précède les **deux** 409 » | ⛔ l'un des deux 409 n'existe plus → **renommé et réécrit** `a_closed_fiscal_year_answers_before_any_conflict` : exercice clos **et** contre-passée **et** pièce montées ensemble, `PUT` et `DELETE` → 400 `FISCAL_YEAR_CLOSED` (AC 4-bis) |

**Frontend** : `[id]/+page.svelte` (D4) ; `i18n-keys.test.ts` ; `tests/e2e/journal-entries.spec.ts` (T5) ;
`e2e-selecteurs-traduits.test.ts`.

**i18n** — quatre locales : `journal-entries-blocked-posted` **retirée** ; clés neuves de D4 ;
`journal-entries-modify-blocked-detached-settlement` (posée par la 15-8a) : étendue à la suppression **sans** la tournure
« ne se modifie ni ne se supprime », qui est un motif du contrôle de l'AC 12 (finding F9) — FR : « Ce paiement
appartient à une facture fournisseur annulée : il reste figé. Corrigez-le par une contre-passation. » (le numéro de la
facture suffixé par le mappage, comme pour les autres pièces — 15-8a D2) ;
`journal-entries-modify-blocked-later-fiscal-year-closed` (posée par la 15-8a) couvre déjà la suppression (« elle reste
figée tant qu'il l'est »).

**Manuel utilisateur** (`docs/manual/fr/user-manual.tex`) — sur l'état laissé par la 15-8a :

| ligne (avant la 15-8a) | à faire |
|---|---|
| `:478-494` § « Modifier une écriture » | renommée « Modifier ou supprimer une écriture » (entrée **7.4** de la table des matières) ; la suppression : même cadre, confirmation, numéro non réattribué, trace `journal_entry.deleted` ; « Modifiée » et « Historique » sur la fiche |
| `:554-555` § contre-passation | « En usage courant, une seule voie fait encore disparaître une écriture » devient faux : la suppression d'une écriture manuelle est une seconde voie, tracée |
| `:572-585` § Numérotation | « Une écriture ne se supprime plus » ; « si la dernière écriture disparaît — par la dévalidation de sa facture » : la suppression d'une écriture manuelle creuse aussi un trou ; le numéro n'est jamais repris |
| `:631` § clôture | la clôture ferme aussi la suppression — celle de l'exercice de l'écriture **ou d'un exercice postérieur** (C-15-8-22) |
| `:667` *keshnote* soldes de départ | supprimer l'ouverture quand elle est seule rouvre la génération (D3) — la nouvelle ouverture prenant le **numéro 2** (C-15-8-12) ; quand d'autres écritures existent, un compte de l'ouverture **déjà mouvementé ailleurs** (la banque) ne redevient pas complétable : il se ressaisit par une OD (D3, finding F10) |
| `:1331-1337`, `:2140` | le paiement détaché ne se supprime pas non plus — par la **tournure prescrite** de l'AC 12, qui remplace celle de la 15-8a |
| `:2115-2140` FAQ | la suppression |

**Manuel administrateur** (`docs/manual/fr/admin-manual.tex`) : `:1796` (une clé `read-write` peut aussi
**supprimer** une écriture manuelle, tracée avec la clé) ; `:1835` (ce qui en reste après la 15-8a) ; `:1837` (« une
seule voie fait encore disparaître une écriture […] Ce n'est pas une brèche dans l'immutabilité » → la suppression
d'une écriture manuelle est une seconde voie, tracée) ; `:1838` (« modifications et suppressions **antérieures au
gel** » → et les suppressions postérieures, sous `journal_entry.deleted`).

**Autres documents** : `docs/api-external.md` (§ 7, sous-section de la 15-8a étendue au `DELETE`, tableau des refus de
D1) ; `docs/MULTI-TENANT-SCOPING-PATTERNS.md` (ligne du `DELETE`) ; `CHANGELOG.md` (section `## [0.13.0]` : la
suppression, et `ENTRY_IS_POSTED` **retiré** — changement de contrat pour une intégration ; il n'a jamais figuré dans
`docs/api-external.md`) ; `README.md:29` — ⛔ **éditée par cette story seule** (C-15-8-26 ; la 15-8a n'y touche pas) :
« écritures validées » devient « écritures validées, modifiables et supprimables tant que l'exercice est ouvert » (la
rédaction P2 citait « … et supprimables » comme s'il existait déjà : findings R3-4, F3).

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

Claude Opus 5.5 (agent de développement, autonomie complète — consignes de l'Epic 15).

### Debug Log References

- ⛔ **Cible cargo** : la cible partagée `target/` mélangeait les worktrees (un `kesh-api` compilé contre le `kesh-db`
  d'une autre branche, sans recompilation) — tous les gates ci-dessous ont tourné sur la cible **dédiée**
  `target-158` (C-15-8b-2).
- Gate ciblé (cible dédiée, base `kesh_158` remise à zéro) : `journal_entries_modification`,
  `journal_entry_reversal_e2e`, `opening_balances_e2e`, `period_lock_e2e`, tests `journal_entries` et `invoices` de la
  lib `kesh-db` — **209 passés, 0 échec**.
- **Gate complet** (`scripts/test-fast.sh`, base `kesh_158` remise à zéro juste avant — DROP/CREATE, migrations,
  seed) : fmt, clippy `-D warnings`, nextest — **2825 passés, 0 échec, 4 ignorés**.
- Frontend : `npm run check` 0 erreur (27 avertissements, aucun dans les fichiers touchés) ; `lint-i18n-ownership`
  PASS ; `test:unit` **1036 passés / 110 fichiers** ; `build` vert.
- **E2E complet** (Playwright, backend `target-158` sur le port 3001, base `kesh_e2e_158` remise à zéro —
  DROP/CREATE + migrations, sans seed SQL : le harnais sème par `/_test/seed` —, `KESH_COOKIE_SECURE=false`,
  `KESH_TEST_MODE=true` des deux côtés, SMTP factice, inbox/documents dans `/tmp/kesh-e2e-158`), sur l'arbre du commit
  de dev : **242 passés, 12 échecs, 19 ignorés**. Jugés fichier par fichier contre `docs/testing.md` § « Les échecs
  attendus » : **8 attendus** (7 KF-029 — `mode-expert:26`, `:41`, `onboarding-path-b:65`, `:92`, `onboarding:57`,
  `:77`, `:150` — et `sidebar-navigation:75`, KF-046) ; **4 hors liste** — `accounts:54`, `auth:22`,
  `bank-account-journal-link:86`, `bank-accounts-crud:76` (timeouts sur `#username` / `#coord-address`, profil
  KF-053) — **4/4 verts rejoués seuls** : pollution. Aucune spec de `journal-entries.spec.ts` en échec (les trois
  neuves et les deux étendues passent).

### Completion Notes List

**T0 — inventaire.** Fait à la clôture de la validation (table « Décalages mesurés après la 15-8a », AC 10 refermée).
Grep de l'AC 10 **après** la story, hors `_bmad-output/` : **5 lignes** (« 6 » écrit au dev, recompté à la revue P1, A5), exactement la liste fermée — `CHANGELOG.md:15`
et `:17` (section `[0.13.0]` : l'entrée de la 15-8a, puis celle de la suppression qui dit `ENTRY_IS_POSTED` retiré),
`docs/api-external.md:221` (parenthèse historique du `PUT`) et `:259` (sa jumelle pour le `DELETE`, écrite par T7),
`frontend/src/lib/shared/i18n-keys.test.ts:174` (paragraphe historique). Les 35 autres sites ont disparu.

**T1 — `delete_in_tx`.** `enforce_immutability` → `enforce_ownership` ; étape 2-bis (`fy.start_date` au `SELECT`
joint, `find_later_closed_in_tx` si `enforce_ownership`, **lu** avant l'étape 3, **rendu** après elle) ; étape 3-ter =
`modification_guard` → `modification_refusal` ; `actor_api_key_id` et audit `for_actor` ; `unvalidate` passe `None,
false`. Doc-comments de `delete_by_id`/`delete_in_tx` et en-tête de module réécrits (l'en-tête affirmait encore que
supprimer la dernière écriture « fait réattribuer son numéro » — faux depuis la 25-2-c, corrigé). Tests d'ordre dans
`mod tests` montés dans une transaction annulée (C-15-8b-3).

**Mutations** (déclarées, toutes **tuées**, fichier restauré par `cp` puis `touch`, `cmp` identique à l'original) :

| mutation | rougit |
|---|---|
| M1 — permuter 3-ter et 3-quater (garde après le verrou de période) | `la_garde_parle_avant_le_verrou_de_periode`, `la_contre_passation_parle_avant_le_verrou_de_periode`, `the_precedence_of_the_delete_refusals_is_fixed` |
| M2 — rendre le verdict de 2-bis avant l'étape 3 | `l_exercice_clos_parle_avant_l_exercice_posterieur`, `the_precedence_of_the_delete_refusals_is_fixed` |
| M3 — lecture ordinaire (la borne) avant le `FOR UPDATE` | `delete_waits_for_a_concurrent_reversal_then_refuses` — A rend `ForeignKeyViolation` (1451 sur `fk_journal_entries_reverses`), exactement ce que l'AC 5 prédisait |
| M4 — `find_later_closed` (lecture ordinaire) au lieu de `find_later_closed_in_tx` | `delete_waits_for_a_concurrent_close_of_a_later_year_then_refuses` |

**T2 — retrait du gel.** `DbError::EntryIsPosted`, son bras de `code()`, son mappage `kesh-api` et son commentaire,
la clé `journal-entries-blocked-posted` ×4 retirés ; doc de `PeriodLocked` reformulé. Clé
`journal-entries-modify-blocked-detached-settlement` réécrite ×4 + repli Rust + repli TS (tournure sans motif de
l'AC 12).

**T3 — route.** `DELETE` enveloppé dans `retry_with` (`grep -nF "retry_with" crates/kesh-api/src/routes/journal_entries.rs`
rend le `PUT` `:675` et le `DELETE`), `api_key_id` transmis, doc-comment réécrit. Pas de test de rejeu propre
(C-15-8b-6).

**T4 — écran.** « Supprimer » (absent quand `modifiable` est faux), confirmation `role="dialog"`, toast, retour à la
liste, refus → toast + rechargement ; « Modifiée » (`version > 1`, tous rôles) et « Historique »
(Administrateur/Comptable) — C-15-8b-4. Huit clés ×4 (liste fermée de D4). `sitesTotal` 1876 → 1885, ventilé (+9,
`grep -o` aux deux bornes : 10 → 19). Aucun sélecteur traduit neuf (`e2e-selecteurs-traduits` vert sans changement).
Doc-comment de `deleteJournalEntry` réécrit.

**T5 — Playwright** (`journal-entries.spec.ts`, 21 → 24 tests) : neufs « supprimer depuis la fiche… », « annuler la
suppression… », « après modification : « Modifiée » et « Historique »… » ; étendus et renommés « écriture de facture :
ni Modifier ni Supprimer, le motif est affiché » et « rôle Consultation : ni Modifier, ni Contre-passer, ni Supprimer,
ni Historique ».

**T6 — tests qui changent de sens** (aucun supprimé sans remplaçant) :

| retiré / changé | remplaçant |
|---|---|
| `deleting_a_posted_entry_is_refused` | `deleting_a_manual_entry_removes_it_traces_it_and_never_reuses_its_number` (AC 1) |
| `a_reversed_entry_answers_reversed_not_posted` | `a_reversed_entry_answers_reversed_on_put_and_delete` |
| `a_closed_fiscal_year_answers_before_both_conflicts` | `a_closed_fiscal_year_answers_before_any_conflict` (exercice clos + contre-passée, + pièce ; `PUT` et `DELETE`) |
| `the_opening_entry_is_modifiable_and_still_reversable` | `the_opening_entry_is_modifiable_reversable_and_deletable` (moitié `DELETE` inversée, 204) |
| `each_screen_code_maps_to_its_put_refusal` | `each_screen_code_maps_to_its_put_and_delete_refusal` (colonne `DELETE`, AC 8 ; `modifiable` → 204) |
| `an_entry_of_a_closed_year_stays_correctable` | inchangé de nom, `DELETE` → 400 ajouté |
| `le_gel_parle_avant_le_verrou_de_periode` (`mod tests`) | `la_garde_parle_avant_le_verrou_de_periode` + `la_route_refuse_une_ecriture_manuelle_de_periode_verrouillee` |
| `deleting_a_reversed_entry_is_refused_but_bulk_delete_still_works` | **reste** |

Neufs : `a_read_write_key_deletes_and_is_traced_as_the_key` (AC 1), `every_document_owned_entry_refuses_the_delete_and_stays_intact`
(AC 2, 4 — transaction bancaire assertée intacte), `period_lock_closed_year_and_later_year_refuse_the_delete` (AC 4),
`the_precedence_of_the_delete_refusals_is_fixed` (AC 4-bis), deux tests de concurrence (AC 5), deux tests de soldes de
départ (AC 7, C-15-8b-5), sept tests d'ordre et `la_devalidation_ne_voit_pas_l_exercice_posterieur` en `mod tests`.
**Décompte** (`grep -c '#\[sqlx::test\|#\[tokio::test\]'`, de `52a9b19b`/`42b13a62` au commit de dev) :
`journal_entry_reversal_e2e.rs` 37 → 41, `opening_balances_e2e.rs` 29 → 31, `journal_entries_modification.rs` 5 → 7,
`journal_entries.rs` (`mod tests`) 49 → 57 ; `journal-entries.spec.ts` 21 → 24.

**T7 — documentation.** Manuel utilisateur (§ 7.4 renommée « Modifier ou supprimer une écriture » — table des matières
vérifiée dans le PDF —, Supprimer, Modifiée/Historique, note de contre-passation « deux voies », Numérotation, clôture,
soldes de départ : numéro 2 et compte déjà mouvementé ressaisi par OD, paiement détaché et FAQ par la tournure
prescrite, FAQ « revenir en arrière ») ; manuel administrateur (`:1797` clé `read-write`, `:1836`, `:1838` « deux
voies », `:1839` suppressions tracées) ; `docs/api-external.md` (section « Supprimer une écriture », 9 refus, tableau des
routes, deux codes) ; Pattern 5 (ligne `DELETE`, deny list à deux entrées) ; `CHANGELOG.md` ; `README.md:29`. `make fr`
— 0 « ?? ». **Contrôle AC 12 sur les PDF aplatis** : motif 15-8b → **0 / 0**, `.ftl` → **0** (ligne de base sur les
PDF de `52a9b19b` : **2** côté utilisateur, **1** côté administrateur — les nombres 5/2 de la fiche datent d'avant la
15-8a) ; motif de l'AC 17 de la 15-8a → **0 / 0 / 0**. Passage conservé, relu à la main : `user-manual.tex` « Quand
l'écriture ne se modifie pas, elle ne se supprime pas non plus » (vrai, hors motif).

**Hors périmètre, à signaler à l'orchestrateur** : l'audit par clé d'API de la dévalidation (`unvalidate` ne reçoit
pas de clé) ; l'exercice postérieur clos non contrôlé sur la dévalidation (C-15-8-29, fixé par un test) — issue
#543 ou neuve ; le mélange de la cible cargo partagée entre worktrees (C-15-8b-2).

**Revue de code P1 et intégration sur `origin/main`** (clôture, Opus 5.5). Remédiation au commit `8cfb3759` (sept LOW,
détail au Change Log), branche rebasée sur `ef39dd54` (15-5c). Tous les gates ci-dessous ont tourné sur **l'état rebasé,
au commit `8cfb3759`**, cible cargo dédiée `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh/target-158` :

- **Gate complet backend** (`scripts/test-fast.sh`, `DATABASE_URL` → `kesh_158` remise à zéro juste avant — DROP/CREATE,
  migrations, seed) : fmt, clippy `-D warnings`, nextest — **2825 passés, 0 échec, 4 ignorés**. Le nombre ne bouge pas :
  A3 ajoute un cas à un test existant, pas un test.
- **Frontend** : `npm run check` 0 erreur (27 avertissements, aucun dans les fichiers touchés) ; `lint-i18n-ownership`
  PASS ; `test:unit` **1086 passés / 111 fichiers** (le saut depuis 1036 vient de la 15-5c, intégrée) ; `build` vert.
  `sitesTotal` **1904 → 1913** sur l'état rebasé (+9 de cette story, inchangé par la remédiation : E-2 reformule une clé
  existante), vérifié par le test lui-même.
- **E2E complet** (binaire `target-158` copié, backend sur le port **3008** — le 3001 était tenu par un autre projet,
  C-15-8b-7 —, base `kesh_e2e_158` remise à zéro sans seed SQL, montage complet : `KESH_COOKIE_SECURE=false`,
  `KESH_TEST_MODE=true` des deux côtés, SMTP factice, `/health` → `smtpConfigured:true`, inbox/documents neufs) :
  **247 passés, 7 échecs, 19 ignorés**. Les 7 sont les **7 KF-029** de `docs/testing.md` (`mode-expert:26`, `:41`,
  `onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`, `:150`) ; aucun hors liste, rien à rejouer. Les huit specs de
  `journal-entries.spec.ts` sur la fiche passent, dont la spec Consultation étendue (A2).
- **Non fait** : aucune mutation rejouée pour A2/A3 (les deux cas ajoutés sont lus, pas mutés).

### File List

- `CHANGELOG.md`, `README.md`
- `crates/kesh-api/src/errors.rs`, `crates/kesh-api/src/routes/journal_entries.rs`
- `crates/kesh-api/tests/journal_entry_reversal_e2e.rs`, `crates/kesh-api/tests/opening_balances_e2e.rs`,
  `crates/kesh-api/tests/period_lock_e2e.rs`
- `crates/kesh-db/src/errors.rs`, `crates/kesh-db/src/repositories/invoices.rs`,
  `crates/kesh-db/src/repositories/journal_entries.rs`, `crates/kesh-db/tests/journal_entries_modification.rs`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`
- `docs/MULTI-TENANT-SCOPING-PATTERNS.md`, `docs/api-external.md`
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/user-manual.pdf`, `docs/manual/fr/admin-manual.tex`,
  `docs/manual/fr/admin-manual.pdf`
- `frontend/src/lib/features/journal-entries/blocker-messages.ts`,
  `frontend/src/lib/features/journal-entries/journal-entries.api.ts`, `frontend/src/lib/shared/i18n-keys.test.ts`,
  `frontend/src/routes/(app)/journal-entries/[id]/+page.svelte`, `frontend/tests/e2e/journal-entries.spec.ts`
- Revue P1 : `frontend/src/routes/(app)/journal-entries/+page.svelte` (commentaire, E-1/A1)
- `_bmad-output/implementation-artifacts/15-8b-supprimer-une-ecriture.md`, `sprint-status.yaml`,
  `epic-15-choix-autonomes.md` (C-15-8b-2 à C-15-8b-6)

### Change Log

| date | ce qui s'est passé |
|---|---|
| 2026-10-08 | **Créée par découpage** de la 15-8 après la validation P2 (choix C-15-8-17) ; spec, P1 et P2 au Change Log de l'index `15-8-modifier-une-ecriture.md`. Remédiations de la P2 propres à la suppression : motifs du test de concurrence `DELETE` (R2-8), rejeu sur interblocage du `DELETE` (F2, C-15-8-19), paiement détaché refusé au `DELETE` (F3, C-15-8-20), appelants de test de `delete_in_tx` (F12), sortie attendue du grep de l'AC 10 (R2-12), contrôle aplati élargi (R2-4, F5). **Recompte** (cette fiche) : 12 AC, 9 tâches (T0 à T8), 5 décisions (D1 à D5). |
| 2026-10-08 | **Validation P3** (deux lentilles **Sonnet**, contexte frais, lecture seule ; prompt versionné `15-8b-validate-prompt-p3.md` ; rapports `target/gate-logs/15-8b-p3-{R,F}.md`). **R** : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 6 LOW ; **F** : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 8 LOW. Doublons : R3-1 = F2, R3-2 = F6, R3-3 = F8, R3-4 = F3 — soit **2 MEDIUM et 11 LOW distincts**. **Tout appliqué**, sur décisions de l'orchestrateur : **exercice postérieur clos** (F1, MEDIUM) — cadre, étape 2-bis de `delete_in_tx` (`fy.start_date`, `find_later_closed_in_tx` avant la première lecture ordinaire), seulement sur le chemin de la route (C-15-8-22, C-15-8-29) ; **précédence du `DELETE`** (R3-1, F2, MEDIUM) — AC 4-bis : paires de causes dont pièce + borne → 409 `OWNED_BY_INVOICE` et exercice clos + pièce → 400 `FISCAL_YEAR_CLOSED`, test d'ordre `la_garde_parle_avant_le_verrou_de_periode` à la place du test du gel, mutations « permuter 3-ter et 3-quater » et « permuter 2-bis et 3 » ; tests e2e qui changent de sens ajoutés (`:1317`/`:1331`, `:1341`). **LOW** : mutation de l'AC 5 décrite juste (A bute sur la clé `RESTRICT`, 1451 — R3-2, F6) ; choix applicables complétés (C-15-8-1, 5, 7, 13 ; renvois corrigés par C-15-8-24 — R3-3, F8) ; `README.md:29` édité par cette story seule, forme finale (R3-4, F3 — C-15-8-26) ; « T1 : le doc-comment le dit » (R3-5) ; motif de l'AC 12 complété (`ni modifiée ni supprimée`), contrôle sur les PDF aplatis, ligne de base mesurée 5 / 2 / 0 (R3-6 — C-15-8-27) ; inventaire des résidus d'`ENTRY_IS_POSTED` en liste fermée (F4) ; `MATCHED_BANK_TRANSACTION` (`ON DELETE SET NULL`) parmi les refus que seule la garde apporte, transaction bancaire assertée intacte (F5) ; D2 : « aucun cycle connu, rejeu par uniformité » (F7) ; tournure prescrite du paiement détaché, texte de sa clé sans motif (F9) ; compte déjà mouvementé non re-complétable, au manuel (F10) ; `test_fixtures.rs:585-588`. R3-7 (`unvalidate` sans clé d'API) : rien à corriger dans la fiche, **re-signalé à l'orchestrateur**. **Signal D5** : MEDIUM en P2 et P3 ; défauts distincts, aucun né d'une remédiation de cette fiche — pas de découpage (4 modules de premier niveau, F). **Propagation** : grep des symptômes dans les trois fiches 15-8 (`dix`, `fait passer A`, `les deux 409`, `et supprimables`, `583-586`, `coût nul`). **Recompte** (cette fiche, `grep`) : **13** critères (AC 1 à 12 et 4-bis), 9 tâches (T0 à T8), 5 décisions (D1 à D5). |
| 2026-10-08 | **Validation close** (agent de développement, avant T0, sur `52a9b19b` — 15-8a mergée). Reste de la P4 ciblée (Haiku, `target/gate-logs/15-8b-p4-ciblee.md`) : **1 MEDIUM** (M1 — l'AC 10 se disait « liste fermée » et contenait un « à vérifier »). **Tranché au code** : `journal-entries.api.ts` porte **encore** `ENTRY_IS_POSTED` (`:70`, doc-comment de `deleteJournalEntry` posé par la 15-8a) — l'inversion ne l'a pas effacé ; il est **dedans**, à faire disparaître (T4 le réécrit). **Inventaire refait sur `HEAD`** : 38 lignes hors `_bmad-output/` ; deux sites **neufs** apportés par la 15-8a — `CHANGELOG.md:15` (son entrée, réécrite dans la même section) et `docs/api-external.md:221` (parenthèse historique, qui reste) ; `journal-entries.spec.ts:285` **sort** de la liste (bloc reformulé par la 15-8a). AC 10 réécrite en liste réellement fermée (3 qui restent, 35 qui disparaissent, nommés ligne à ligne) — C-15-8b-1. **Numéros de ligne** : table « Décalages mesurés après la 15-8a » ajoutée en tête des Dev Notes (les décalages vont jusqu'à +1021 lignes dans `journal_entry_reversal_e2e.rs`) ; `the_opening_entry_is_frozen_but_still_correctable` a été **renommé** par la 15-8a. AC 4 : le montage commun s'appelle `monter_les_pieces`. **`closes #532` → `refs #532`** : l'issue a été fermée par la 15-8a. Aucun CRITICAL/HIGH/MEDIUM ouvert : **validation close**. |
| 2026-10-08 | **Développement** (`bmad-dev-story`, Opus 5.5, autonomie). T0–T8 faits : `DELETE` dans le cadre de la 15-8a (étape 2-bis, garde 3-ter, audit par acteur, `retry_with`), retrait du gel, écran (Supprimer, confirmation, « Modifiée », « Historique »), Playwright, documentation (manuels régénérés, contrôle AC 12 à 0 sur les PDF aplatis). Quatre mutations déclarées, quatre tuées. Gates : complet backend 2825/2825 ; frontend check/lint/1036 unitaires/build verts ; E2E 242 passés, 8 échecs attendus + 4 de pollution verts rejoués seuls. Choix C-15-8b-2 à C-15-8b-6. |
| 2026-10-08 | **Revue de code P1** (trois lentilles **Sonnet** — Blind Hunter, Edge Case Hunter, Acceptance Auditor —, contexte frais, lecture seule ; prompt versionné `15-8b-review-prompt-p1.md` ; rapports `target/gate-logs/15-8b-review-p1-{B,E,A}.md`). **B** : 0 C / 0 H / 0 M / 4 L ; **E** : 0 C / 0 H / 0 M / 5 L ; **A** : 0 C / 0 H / 0 M / 5 L — **14 LOW**, dont deux doublons (E-1 = A1, B-4 = E-5), soit **12 distincts**. **Corrigés** (commit `8cfb3759`, sept LOW distincts) : E-1/A1 commentaire de la liste (« la suppression reste refusée ») ; E-2 `error-fiscal-year-closed-generic` étendu à « supprimée » dans les 4 locales et son repli Rust, sans clé neuve (C-15-8b-8) ; A4 phrase orpheline du doc-comment de `delete_by_id` rattachée au renvoi qu'elle motive ; A5 « 6 lignes » → **5** au Dev Agent Record (recompté) ; B-1 paragraphe de `i18n-keys.test.ts` daté (24-4b, jusqu'aux 15-8a/15-8b) ; A2 spec Playwright Consultation : l'écriture est modifiée avant, « Modifiée » exigé visible (PUT factorisé en `modifierLibelle`) ; A3 `period_lock_closed_year_and_later_year_refuse_the_delete` finit sur une borne la veille → 204, sur le chemin de la route. **Acceptés (LOW, motif)** : B-2 test qui fige C-15-8-29 — voulu, à inverser quand #543 sera traitée (le doc-comment le dit) ; B-3 tests d'ordre sur lignes partagées (verrous dans une transaction annulée) — hypothèse non observée, piste nommée si un `journal_entries::tests` rougit ; B-4 = E-5 dialogue sans Échap ni piège de focus — même patron que la contre-passation voisine, à reprendre avec elle ; E-3 verrou des exercices postérieurs pris même quand l'exercice est clos — choix de la fiche (lire sous verrou, rendre après), couvert par `retry_with` ; E-4 branche `fk_invoices_journal_entry` d'`errors.rs` devenue inatteignable par la route — filet de sécurité conservé. **Propagation** : grep des symptômes (`reste refusée`, `ajoutée ou modifiée` et ses trois traductions, `Cette précision`, `6 lignes`) — corrigés partout où ils s'appliquent ; restent, voulus, la variante datée `error-fiscal-year-closed` (4 locales, `errors.rs:312` et `:1461`), rendue par la saisie et la modification mais pas par le `DELETE`. **Intégration** : rebase des quatre commits de la story sur `origin/main` (`ef39dd54`, 15-5c) ; conflits : registre (union, les C-15-5c avant les C-15-8b), `sprint-status.yaml` (en-têtes `last_updated` des deux côtés), `i18n-keys.test.ts` (`sitesTotal` **1904 + 9 = 1913**, commentaires des deux stories conservés), `user-manual.pdf` (**régénéré** depuis le `.tex` fusionné automatiquement — 79 pages, 0 référence indéfinie, contrôles : section 15-8b et rapprochement réécrit présents) ; CHANGELOG fusionné sans conflit ; brochure et manuel administrateur non touchés par le rebase. Gates sur l'état rebasé : backend 2825/2825, frontend 1086/1086, E2E 247 / 7 KF-029 (Dev Agent Record). **Statut `review`** jusqu'à la passe ciblée sur `8cfb3759`. |
- **2026-10-08 — Revue de code P2 ciblée (Haiku, une lentille, prompt `15-8b-review-prompt-p2-ciblee.md`) sur
  `8cfb3759` : 0 finding.** Rapport : `target/gate-logs/15-8b-review-p2-ciblee.md`. Le rapport ne listant pas ses axes
  non exercés, l'orchestrateur a repris l'axe porteur : `error-fiscal-year-closed-generic` n'est rendu qu'à
  `crates/kesh-api/src/errors.rs:3126` (refus générique d'un exercice clos), et sa formulation (« ajoutée, modifiée ou
  supprimée ») est juste pour tout flux qui l'atteint. **Boucle de revue CLOSE** (P1 Sonnet ×3 : 0 au-dessus de LOW,
  14 LOW dont 7 corrigés → P2 ciblée Haiku : 0). Statut `done`.
