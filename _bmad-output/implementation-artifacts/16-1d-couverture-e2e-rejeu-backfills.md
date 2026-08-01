# Story 16.1d : Rejeu des backfills après restore — couverture de bout en bout

## Status

ready-for-dev

## Story

**As a** mainteneur de Kesh,
**I want** que le rejeu des backfills après un restore soit **prouvé de bout en bout**, et que chaque test censé discriminer une décision de conception soit **vérifié par mutation**,
**so that** le mécanisme livré par 16-1c ne repose pas sur des tests qui seraient verts pour la mauvaise raison — le mode d'échec que ce dépôt a déjà subi trois fois.

Issue : **#281**. Sous-story de l'Epic 16, **dépend strictement de 16-1c**, qui livre le registre, les classes, l'ordre et les garde-fous.

**Doit partir dans la MÊME PR et la MÊME v0.9.0 que 16-1c et 16-1a-bis.** 16-1c seule livrerait un mécanisme sans preuve de bout en bout, ce que la § « E2E Testing » de `CLAUDE.md` n'autorise pas : le défaut que la story ferme naît de l'**interaction** entre l'import et des migrations que la PR ne touche pas, et seul un test réel le voit.

### Provenance du split (passe 5 de `validate` sur 16-1c)

**Arbitrage de Guy, 2026-08-01.** Le second critère de la § « Règle de splitting préventif » de `CLAUDE.md` s'est déclenché **trois passes de suite** (`CRITICAL → CRITICAL`, puis `MEDIUM → HIGH`, puis `HIGH → HIGH`). Mais le signal ne désignait pas ce que la règle vise :

| Passe | Findings | Où ils portent |
|---|---|---|
| 1 (Sonnet) | 2 CRIT / 2 HIGH / 3 MED / 1 LOW | conception |
| 2 (Opus) | 1 CRIT / 4 HIGH / 6 MED / 8 LOW | conception — périmètre divisé par 2 |
| 3 (Haiku) | 0 / 0 / 2 MED / 1 LOW | résidus de refonte |
| 4 (Sonnet) | 0 / 1 HIGH / 2 MED / 3 LOW | **montages de test — 100 %** |
| 5 (Opus) | 0 / 3 HIGH / 4 MED / 4 LOW | **montages de test — 100 %** |

**Aucune décision de conception n'a bougé depuis la passe 2**, et les deux lentilles Opus de la passe 5 la déclarent saine avec vérification énumérée. Ce qui ne convergeait pas était une **section**, pas une story trop large — d'où un split par **nature de contenu** et non par module.

**Le split est sûr** : 16-1c conserve tous ses garde-fous de niveau unitaire et d'intégration `kesh-db` (dont le test de transactionnalité), donc elle reste vérifiable seule. Cette story ajoute la couche qu'aucun test unitaire ne peut donner.

---

## Contexte

Tout le contexte technique — le mécanisme du bug, la fenêtre d'importabilité, les deux classes, l'ordre croissant, le registre à 2 entrées — est dans **16-1c**, `_bmad-output/implementation-artifacts/16-1c-rejeu-backfills-apres-restore.md`. **Le lire en entier avant d'écrire un test.** Ce qui suit ne répète que ce dont les montages dépendent directement.

### Les quatre faits dont les montages dépendent

1. **Un backup n'est importable que depuis un binaire ≥ `20260715000001`** (dernière migration créatrice de table applicative). `parse_and_verify` exige l'égalité exacte des tables dans les deux sens (`import.rs:117-136`).
2. **Retirer une colonne des `columnNames` du manifeste est sûr** ; supprimer des **lignes** NDJSON ne l'est pas (le `rowCount` n'est pas recalculé par `rezip`), et en **modifier** une exige de réécrire le `sha256` avec le helper local (`admin_full_import_e2e.rs:186`).
3. **`accounts.postable` ne peut pas valoir `TRUE` sur un compte à rôle `CurrentYearResult` ni à enfants actifs** : `effective_postable` (`accounts.rs:126-131`) le force à `false` à la création comme à la mise à jour, et le seed applique la fonction jumelle `is_postable`. Tout montage qui suppose l'inverse doit passer par du SQL direct.
4. **`HAVING COUNT(*) = 1`** (`20260729000001:264`) compte les candidats de l'**écriture entière**. Ajouter une ligne d'écriture ne crée pas un cas de plus : cela en fait **deux**, et inverse le verdict du test.

### Ce qui n'est PAS dans cette story

- Le registre, les classes, l'ordre, les extraits SQL, le câblage dans le restore, les six garde-fous fail-loud, la documentation → **16-1c**.
- Le test de **transactionnalité** (échec injecté pendant le rejeu) reste en 16-1c : il vit au niveau `kesh-db` via `replay_with_registry`, seul niveau où l'échec est observable — par HTTP, `AppError::AdminFullImportFailed` rend un `500` générique dont le détail est loggé et jamais exposé.

---

## Acceptance Criteria

- **AC-D1 — Les six cas de bout en bout**, dans `crates/kesh-api/tests/admin_full_import_e2e.rs` :

Le test de bout en bout est **indispensable et non substituable** : le défaut naît de l'**interaction** entre l'import et des migrations que la PR ne touche pas.

**Montage — la voie praticable est de muter la base AVANT l'export**, pas de forger le backup. Retirer une entrée de `columnNames` du manifeste est sûr (les octets NDJSON ne bougent pas, le SHA-256 et le `rowCount` restent valides — c'est ce que font déjà `full_import_refuses_schema_mismatch_400:474` et `full_import_refuses_missing_required_column_400:504`). **Supprimer** des lignes NDJSON casse le `rowCount`, que `rezip` ne recalcule pas (`parse_and_verify` le vérifie, `import.rs:163-166`). **Modifier** une ligne est en revanche praticable — à condition de réécrire `manifest["tables"][t]["sha256"]` avec le helper local `sha256_hex` (`admin_full_import_e2e.rs:186`), ce que fait déjà `full_import_rolls_back_on_insert_failure` (`:561-566`). La mutation de la base **avant** l'export reste néanmoins préférée quand l'état visé est atteignable : pour C1-bis, faire `UPDATE invoice_lines SET revenue_account_id = NULL` sur la base source **puis** `export_backup`.

| Cas | Attendu | Ce qu'il discrimine |
|---|---|---|
| **C1** — backup **sans** `invoice_lines.revenue_account_id`, facture validée canonique | les lignes portent le compte crédité par leur écriture | cas nominal de l'issue #281 |
| **C1-bis** — base source mutée à `revenue_account_id = NULL` **puis** exportée, colonne donc **présente et vide** | les lignes sont **quand même** backfillées | **classe A / rejeu inconditionnel** — tombe si l'entrée est traitée en classe B |
| **C2** — backup sans `accounts.role` **ni** `postable` | rôles réattribués, `postable` recalculé | classe B, déclenchement |
| **C2-bis** — backup portant `accounts.role` mais **pas** `postable` | le rejeu se déclenche **quand même** | **sémantique OU des sentinelles** — tombe si le dev implémente un ET. **Discriminant synthétique assumé** : les deux colonnes sont ajoutées par le même `ALTER`, donc co-présentes en réalité ; le cas se construit par `strip_column` et ne teste aucun état atteignable — il verrouille la règle pour les entrées futures |
| **C3** — backup complet, `role` / `postable` posés **à la main** sur des valeurs non standard | la donnée du backup est **intacte** | **classe B / conditionnement** |
| **C4** *(test d'intégration `kesh-db`, pas E2E — cf. note de montage)* — échec injecté **pendant le rejeu** via `replay_with_registry` et un registre fautif | l'appel rend `Err`, et après rollback la destination est **inchangée** | transactionnalité |
| **C5** — backup sans `accounts.role`, **sans `accounts.postable`** et sans `revenue_account_id` ; le candidat unique du backfill est le compte de résultat n° `2979` | la ligne reste `NULL` | **ordre croissant** — tombe si le registre est parcouru à l'envers |

**Notes de montage obligatoires** — chaque test porte une section « ce que ce test discrimine ». **Quatre** montages ont un piège nommé :

- **C4** : le patron existant `full_import_rolls_back_on_insert_failure` (`:543`) injecte l'échec dans l'`INSERT` du restore, donc **avant** le point d'insertion du rejeu — le réutiliser tel quel produit un test vert qui n'atteint jamais le rejeu. Prévoir un point d'injection propre, **plus une assertion de montage** prouvant que le rejeu a démarré.

  ⚠️ **`#[cfg(test)]` NE TRAVERSE PAS la frontière de crate — ne pas chercher à injecter par là.** `kesh-db` est une dépendance **ordinaire** de `kesh-api` (`crates/kesh-api/Cargo.toml:9`) : depuis un test d'intégration de `kesh-api`, `cfg(test)` de `kesh-db` vaut **faux**. Une entrée fautive `#[cfg(test)]` ne serait donc vue par **aucun** des sept cas — et non par tous, comme l'affirmait une rédaction antérieure de cette note. C'est le même piège que celui documenté trois sections plus bas pour `build_test_backup`.

  Et la déclarer inconditionnellement est **exclu** : le registre de production compterait 3 entrées au lieu de 2 (**16-1c**, AC-C1), dont une dont le seul rôle est de faire échouer tout restore dès que sa sentinelle manque — la mine même que cette story désamorce.

  **Décision** : rendre le registre **injectable**. `replay_post_restore_backfills(tx, tables)` délègue à une fonction `pub` `replay_with_registry(tx, tables, registry)`, appelée avec `POST_RESTORE_BACKFILLS`. **C4 devient alors un test d'intégration de `kesh-db`**, non un cas HTTP : ouvrir une transaction, restaurer, appeler `replay_with_registry` avec un registre fautif, vérifier l'`Err` puis, après rollback, que la destination est intacte. C'est le seul niveau où l'échec est **observable** — par le chemin HTTP, `AppError::AdminFullImportFailed` rend un `500` générique dont le détail est **loggé et jamais exposé** (`errors.rs`), donc indiscernable d'un échec d'`INSERT` du restore.

  **Conséquence sur le décompte** : **AC-D1 porte 6 cas E2E** (C1, C1-bis, C2, C2-bis, C3, C5) ; C4 vit en test d'intégration `kesh-db` et relève de **16-1c**. Aucune exemption n'est alors nécessaire dans les garde-fous de 16-1c (son AC-C6), puisqu'aucune entrée fautive n'entre dans le `const`.
- **C5** : retirer `postable` est **indispensable**. S'il était laissé dans le manifeste, il vaudrait déjà `FALSE` sur `2979` — le restore reposerait la valeur telle quelle et le test passerait **dans les deux ordres**, donc sans rien discriminer. Le candidat s'obtient en **REPOINTANT** l'unique ligne de crédit de produit de l'écriture — `UPDATE journal_entry_lines SET account_id = <id de 2979> WHERE entry_id = <je> AND credit = <total_amount>` — par **`sqlx::query` directement sur le pool de la base source**, jamais par l'API.

  ⚠️ **NE PAS AJOUTER de ligne : cela INVERSE le test.** `HAVING COUNT(*) = 1` (`20260729000001:264`) compte les candidats de l'écriture **entière**, et une facture canonique en produit déjà exactement un (`invoices.rs:1481-1484`, une ligne de crédit par compte effectif). Avec une ligne de plus : en **ordre correct**, `2979` est écarté par `a.postable = TRUE` et il reste **un** candidat → la ligne reçoit `3000`, l'attendu « reste `NULL` » **échoue** ; en **ordre inversé**, les deux sont candidats → `HAVING` échoue → la ligne reste `NULL` et le test **passe**. Rouge sur l'implémentation correcte, vert sur la fautive.

  ⚠️ **NE PAS créer d'écriture séparée** : `jel.entry_id = i.journal_entry_id` (`:254`) la rendrait invisible au backfill — muet dans les deux ordres.

  Le repointage préserve en outre l'équilibre débit/crédit, qu'**aucun `CHECK` DB ne protège** (`20260412000001:12`), et évite d'avoir à choisir un `line_order` sous `uq_jel_entry_order`. **Assertion de montage exigée avant l'export** : l'écriture compte exactement **un** candidat au sens du critère, et son `account_id` est celui de `2979`.

  Le repointage passe par SQL direct parce qu'**aucun chemin applicatif ne peut produire cet état** :

  - `POST /invoices` refuse un compte de produit non imputable à la saisie ;
  - `PUT /api/v1/journal-entries/{id}` non plus : `journal_entries::update` passe `enforce_postable = **true**` en dur (`repositories/journal_entries.rs:936`, commentaire `:909-910` « L'update est toujours un flux MANUEL »), et son *grandfather* D-A1 n'exempte que les comptes **déjà référencés par cette écriture** (`SELECT DISTINCT account_id FROM journal_entry_lines WHERE entry_id = ?`, `:926-928`) — or l'écriture générée par la validation n'a jamais touché `2979` ;
  - et `2979` **naît** non imputable : `effective_postable` (`repositories/accounts.rs:126-131`) force `postable = false` dès que `role = CurrentYearResult`, à la création comme à la mise à jour, et le seed applique la **même** règle via la fonction pure jumelle `kesh_core::chart_of_accounts::is_postable` (`bulk_create_from_chart`, `:839`) — c'est l'invariant « seed ≡ backfill ». Aucun `PUT /accounts` ne peut l'inverser.

  ⚠️ **Ne PAS attribuer `enforce_postable = false` au `PUT`** — ce `false` est sur le chemin **automatique** de validation de facture (`crates/kesh-db/src/repositories/invoices.rs:1813-1815`, argument de l'appel ouvert en `:1798`). Une rédaction antérieure de cette note faisait cette confusion et prescrivait un montage que l'API aurait refusé en 4xx.
- **C3** : « poser `role` / `postable` à la main sur des valeurs non standard » **ne suffit pas** — le montage littéral produirait un test **muet**. Les 10 `UPDATE` de rôle sont gardés `role IS NULL` donc no-op sur des rôles renseignés ; et les 2 `UPDATE` de `postable` visent des états que l'API **ne peut pas** produire, `effective_postable` (`accounts.rs:126-131`) forçant `postable = false` sur un compte à rôle `CurrentYearResult` ou à enfants actifs, **à la création comme à la mise à jour**. Le rejeu inconditionnel serait donc entièrement no-op et la **mutation 3 ne rougirait pas**.

  Le montage discriminant, et il est atteignable par l'API : **un rôle délibérément effacé**. `PUT /api/v1/accounts/{id}` documente `role: null` comme l'acte de retrait (`routes/accounts.rs:70`). Effacer le rôle du compte `1100` dans la base source ; un rejeu inconditionnel le réécrit en `'Receivable'`, ce qui fait tomber « la donnée du backup est intacte ». *(Ce piège est le même fait que celui de C5 — `postable` inatteignable par l'API — non propagé au moment où C5 a été corrigé.)*
- **C1** : sans facture validée dans la source, l'assertion porte sur un ensemble vide et le **cas nominal de l'issue passe à vide**. D'où la mutation dédiée en **T-D3** (mutation 5).

- **AC-D2 — Fixture métier.** `admin_full_import_e2e.rs` ne sait aujourd'hui créer qu'une **société et un utilisateur** (`seed_role:139`) : aucun plan comptable, aucune facture, aucune écriture. La fixture est donc à écrire, et son absence rendrait **C1 — le cas nominal de l'issue #281 — vert sur un ensemble vide**. Semer le plan via `kesh_core::chart_of_accounts::load_chart("Pme")` + `accounts::bulk_create_from_chart`, créer contact et produit, créer puis **valider** une facture pour produire son écriture.

  ⚠️ **Deux patrons distincts, ne pas confondre.** Pour le **plan comptable** : `crates/kesh-db/tests/company_invoice_settings_repository.rs:39-40` ou `accounts_role_backfill.rs`. **PAS** `seed_accounting_company` (`test_fixtures.rs`), qui insère **5 comptes en dur** (`1000`, `1100`, `2000`, `3000`, `4000`) **sans jamais renseigner `role`** et **sans le compte `2979`** — C5 serait alors inconstructible, son candidat n'existant pas. Pour la **facture validée par le vrai moteur** : `crates/kesh-db/tests/invoice_lines_revenue_account_backfill.rs`, test `validated_invoice_from_the_real_engine_is_recovered_by_the_backfill`.
- **AC-D3 — Discrimination prouvée par mutation, pas constatée.** Les cinq mutations de `T-D3` sont **exécutées**, et leurs résultats consignés au Dev Agent Record. **Un test attendu qui ne rougit pas invalide le montage** : le corriger avant d'aller plus loin. Un test vert ne prouve rien par lui-même — c'est l'enseignement mesuré des stories 16-1a et 16-1a-bis.
- **AC-D4 — Aucun fichier de production modifié.** Cette story n'ajoute que des tests. `git diff --stat` ne touche ni `crates/kesh-db/src/`, ni `crates/kesh-api/src/`, ni `frontend/`, ni aucune migration.

---

## Tasks / Subtasks

- [ ] **T-D1 — Fixture métier** (AC-D2)
  - [ ] Helper de montage d'une société complète : plan comptable via `bulk_create_from_chart`, contact, produit.
  - [ ] Helper de création puis **validation** d'une facture, produisant son écriture comptable.
  - [ ] Helper `strip_column(manifest, data, table, column)` sur le couple `(Value, BTreeMap<String, Vec<u8>>)` rendu par `unzip` — geste partagé par C1, C2, C2-bis, C5.
- [ ] **T-D2 — Tests de bout en bout** (AC-D1)
  - [ ] Les **6** cas E2E C1, C1-bis, C2, C2-bis, C3, C5, en réutilisant les helpers de **T-D1** et le harnais existant `spawn_app` / `export_backup` / `unzip` / `rezip` / `post_import`. **Ne pas réécrire les helpers de T-D1.**
  - [ ] *(**C4 n'est PAS à écrire ici** — le test de transactionnalité appartient à **16-1c, T5**. Sa ligne au tableau d'AC-D1 et sa note de montage ne sont conservées que pour expliquer pourquoi cette story porte **six** cas et non sept.)*
  - [ ] Note de montage « ce que ce test discrimine » sur **tous** les cas ; en particulier les pièges nommés de C1, C3 et C5 *(celui de C4 est informatif, cf. ci-dessus)*.
  - [ ] Assertions sur le rapport de rejeu : `backfills_replayed` relu dans `audit_log`, et `outcome` / `rows_affected` de la valeur de retour — dont **un cas où `outcome` vaut `Skipped`** (C3), sans quoi ce troisième état ne serait vérifié nulle part. *(Le contrat du rapport est posé par 16-1c, AC-C7 ; cette story l'exerce.)*
- [ ] **T-D3 — Preuve par mutation** (AC-D3)
  - [ ] Classe A rendue conditionnelle (sentinelle `(invoice_lines, revenue_account_id)`) → **C1-bis** doit rougir, **C1** rester vert.
  - [ ] Sentinelles en ET au lieu de OU → **C2-bis** doit rougir.
  - [ ] Classe B rendue inconditionnelle → **C3** doit rougir.
  - [ ] Registre parcouru en ordre décroissant → **C5** doit rougir.
  - [ ] Registre vidé de l'entrée `20260729000001` → **C1 et C1-bis** doivent rougir *(prouve la non-vacuité du cas nominal, qui sans cela pourrait passer sur un ensemble vide — les deux tombent, l'entrée retirée étant la seule à toucher `invoice_lines`)*.
  - [ ] Consigner les cinq résultats dans le Dev Agent Record. **Un test attendu qui ne rougit pas invalide le montage** : le corriger avant d'aller plus loin.
- [ ] **T-D4 — Gate et contrôle de périmètre** (AC-D3, AC-D4)
  - [ ] **AC-D4** : `git diff --stat` ne montre **aucun** fichier sous `crates/kesh-db/src/`, `crates/kesh-api/src/`, `frontend/` ni `crates/kesh-db/migrations/`. Si l'écriture d'un test exige une modification de production, c'est que 16-1c est incomplète — **la corriger là-bas**, pas ici.
  - [ ] `scripts/test-fast.sh` complet (fmt + clippy `-D warnings` + nextest workspace) sur l'**état final**, exit 0 exigé, non présumé d'un run antérieur. `npm run check` inutile — aucun fichier frontend touché.

---

## Dev Notes

### Les pièges de montage sont dans les AC, pas ici — les lire

Les notes de montage de **C1, C3, C4 et C5** énoncent chacune un piège qui rend le test muet ou inversé. Ils ont coûté trois passes de revue à 16-1c et **aucun n'est détectable à la lecture du test écrit** : ils ne se voient qu'en confrontant le montage au SQL de la migration et aux règles métier du dépôt. Ne pas les paraphraser, ne pas les abréger.

### Le harnais existant, et ce qu'il ne fait pas

| Élément | Ligne | Ce qu'il donne |
|---|---|---|
| `spawn_app` | `:70` | l'app Axum sur un port éphémère |
| `seed_role` | `:139` | **société + utilisateur seulement** |
| `export_backup` | `:195` | export HTTP réel → `Vec<u8>` |
| `post_import` | `:217` | import HTTP multipart |
| `unzip` / `rezip` | `:228` / `:253` | ZIP ↔ `(manifest, data)`, **sans recalcul** du `sha256` ni du `rowCount` |
| `sha256_hex` | `:186` | à réutiliser si l'on modifie une ligne NDJSON |

⚠️ **`build_test_backup` n'est PAS utilisable.** Elle vit dans `crates/kesh-api/src/admin_backup/import.rs:246`, sous `#[cfg(test)]` **privé de la lib** : un fichier `tests/*.rs` est une crate d'intégration séparée, ce code n'y est même pas compilé. Le même piège vaut pour toute tentative d'injecter un registre de test depuis ici.

### References

- Story **16-1c** — `_bmad-output/implementation-artifacts/16-1c-rejeu-backfills-apres-restore.md` : mécanisme, décisions **D-C1** (deux classes) à **D-C8**, garde-fous **AC-C6**.
- Story **16-1a-bis** — le SQL du backfill rejoué, ses décisions **D-B2** (critère d'unicité) et **D-B6** (idempotence), et le patron de montage d'une facture validée avec son écriture.
- Issue **#281** et son commentaire d'arbitrage du 2026-08-01.

---

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

**2026-08-01 — Story née du split de 16-1c en passe 5 de `bmad-create-story validate`**, arbitré par Guy. Le contenu repris (les six cas de bout en bout, leurs notes de montage, les cinq mutations) est dans son **état convergé de la passe 5** : il incorpore les corrections des passes 4 et 5, dont les trois HIGH sur les montages de C4, C5 et C3.

### Passe 1 de `bmad-create-story validate`

**2026-08-01 — Sonnet, 2 lentilles (BlindHunter, AcceptanceAuditor), contexte frais. 6 findings : 0 CRITICAL, 1 HIGH, 4 MEDIUM, 1 LOW — TOUS des résidus du split.** C'est exactement le risque que cette passe visait, et rien d'autre n'est ressorti.

- **HIGH — le patron de fixture cité était le mauvais.** `invoice_lines_revenue_account_backfill.rs` n'utilise **pas** `bulk_create_from_chart` (0 occurrence) : il passe par `seed_accounting_company`, qui insère **5 comptes en dur sans jamais renseigner `role`** et **sans le compte `2979`**. Un dev suivant la citation aurait bâti une fixture où le candidat de C5 **n'existe pas**. Deux patrons distincts sont désormais nommés séparément : le plan comptable (`company_invoice_settings_repository.rs:39-40`) et la facture validée par le vrai moteur.
- **La tâche d'exécution de C4 était cochable des DEUX côtés** — en 16-1c T5 et en 16-1d T-D2, texte quasi identique — alors que le § « Ce qui n'est PAS dans cette story » l'exclut explicitement. Contradiction interne du document, et double implémentation probable.
- **Trois renvois morts** hérités du copier-coller : `AC-C10` et `AC-C6` (qui n'existent pas dans ce document), et « la mutation dédiée en **T6** » alors que les tâches s'appellent ici `T-D1..T-D4`.
- **`T-D1` et `T-D2` revendiquaient tous deux AC-D2**, avec la puce `strip_column` répétée **mot pour mot**. T-D2 réduite à un renvoi.
- **AC-D4 n'était rattachée à aucune tâche**, là où son homologue de 16-1c (AC-C8, même nature de contrôle) l'est explicitement. Rattachée à T-D4 avec sa commande.

**Ce que les deux lentilles ont validé positivement** : les **cinq mutations** tracées une par une contre le SQL des deux migrations — chacune rougit exactement, et seulement, le cas annoncé, aucune n'est muette. Les décomptes propres à la story (six cas, cinq mutations, quatre faits, quatre pièges) recomptés exacts, ainsi que le tableau de sévérités des passes 1-5 repris de 16-1c. Toutes les ancres de code vérifiées.

**Prochaine** : passe 2, LLM ≠ Sonnet, contexte frais.

