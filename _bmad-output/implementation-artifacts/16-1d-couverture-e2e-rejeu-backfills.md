# Story 16.1d : Rejeu des backfills après restore — couverture de bout en bout

## Status

done

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
| 4 (Sonnet) | 0 / 1 HIGH / 2 MED / 3 LOW | montages de test : **3 sur 6** — dont les deux plus graves |
| 5 (Opus) | 0 / 3 HIGH / 4 MED / 4 LOW | montages de test : **7 sur 11** — dont les trois HIGH |

Aucune décision `D-C` n'a bougé depuis la passe 2 — **hors la requalification, en passe 5, de la garde des 10 `UPDATE` de rôle** — et les deux lentilles Opus de la passe 5 déclarent la conception saine avec vérification énumérée. Ce qui ne convergeait pas était une **section**, pas une story trop large : d'où un split par **nature de contenu** et non par module.

*(Le tableau ci-dessus a été recompté en passe 3 de cette story. Une rédaction antérieure y annonçait « montages de test — 100 % » pour les passes 4 et 5 : c'était un nombre **relu et non recompté**, dans la section même qui justifiait un arbitrage de process. Les proportions réelles — 3/6 et 7/11 — soutiennent la même conclusion, mais il fallait le dire juste.)*

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
| **C2** — backup sans `accounts.role` **ni** `postable` | rôles réattribués, `postable` recalculé — **les DEUX backfills de `postable`**, cf. la note ci-dessous | classe B, déclenchement |
| **C2-bis** — backup portant `accounts.role` mais **pas** `postable` | le rejeu se déclenche **quand même** | **sémantique OU des sentinelles** — tombe si le dev implémente un ET. **Discriminant synthétique assumé** : les deux colonnes sont ajoutées par le même `ALTER`, donc co-présentes en réalité ; le cas se construit par `strip_column` et ne teste aucun état atteignable — il verrouille la règle pour les entrées futures |
| **C3** — backup complet, avec le **rôle du compte `1100` délibérément effacé** (`role: null`) ; `postable` reste à sa valeur nominale | la donnée du backup est **intacte**, le rôle effacé le **reste** ; `outcome` vaut `Skipped` | **classe B / conditionnement** — cf. la note de montage, « poser des valeurs non standard » ne suffirait **pas** |
| **C4** *(test d'intégration `kesh-db`, pas E2E — cf. note de montage)* — échec injecté **pendant le rejeu** via `replay_with_registry` et un registre fautif | l'appel rend `Err`, et après rollback la destination est **inchangée** | transactionnalité |
| **C5** — backup sans `accounts.role`, **sans `accounts.postable`** et sans `revenue_account_id` ; le candidat unique du backfill est le compte de résultat n° `2979` | la ligne reste `NULL` | **ordre croissant** — tombe si le registre est parcouru à l'envers |

**Notes de montage obligatoires** — chaque test porte une section « ce que ce test discrimine ». **Quatre** montages ont un piège nommé :

- **C4** : le patron existant `full_import_rolls_back_on_insert_failure` (`:543`) injecte l'échec dans l'`INSERT` du restore, donc **avant** le point d'insertion du rejeu — le réutiliser tel quel produit un test vert qui n'atteint jamais le rejeu. Prévoir un point d'injection propre, **plus une assertion de montage** prouvant que le rejeu a démarré.

  ⚠️ **`#[cfg(test)]` NE TRAVERSE PAS la frontière de crate — ne pas chercher à injecter par là.** `kesh-db` est une dépendance **ordinaire** de `kesh-api` (`crates/kesh-api/Cargo.toml:9`) : depuis un test d'intégration de `kesh-api`, `cfg(test)` de `kesh-db` vaut **faux**. Une entrée fautive `#[cfg(test)]` ne serait donc vue par **aucun** des six cas de cette story, ni par C4 en 16-1c — et non par tous, comme l'affirmait une rédaction antérieure de cette note. C'est le même piège que celui documenté trois sections plus bas pour `build_test_backup`.

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
- **C2** : `20260722000001` porte **DEUX** `UPDATE` de `postable`, et le compte `2979` n'en couvre qu'un. Le backfill **#2** (`:177`) agit par rôle, `WHERE role = 'CurrentYearResult'` — c'est celui que `2979` exerce. Le backfill **#1** (`:161`) est **purement structurel** : il rend non imputable tout compte à **enfants actifs** jamais mouvementé, sans citer aucun numéro. Or la colonne est `NOT NULL DEFAULT TRUE` (`:77`) : retirée du manifeste, elle revient à `TRUE` sur **tout le plan**, comptes titres compris. N'asserter que `2979` laisse donc le backfill #1 entièrement invérifié — une régression de sa sous-requête `EXISTS` / `NOT EXISTS`, ou sa disparition au découpage en statements, rendrait un compte de regroupement imputable après restore sans qu'aucun cas ne rougisse. **Asserter aussi un compte titre** (le `10` du plan PME : parent de `1000`/`1010`/`1020`, non mouvementé par la fixture, donc non imputable au seed par `is_postable` comme au rejeu par le backfill #1). *(Ajouté en passe 1 de `bmad-code-review`.)*
- **C1** : sans facture validée dans la source, l'assertion porte sur un ensemble vide et le **cas nominal de l'issue passe à vide**. D'où la mutation dédiée en **T-D3** (mutation 5).

- **AC-D2 — Fixture métier.** `admin_full_import_e2e.rs` ne sait aujourd'hui créer qu'une **société et un utilisateur** (`seed_role:139`) : aucun plan comptable, aucune facture, aucune écriture. La fixture est donc à écrire, et son absence rendrait **C1 — le cas nominal de l'issue #281 — vert sur un ensemble vide**. Semer le plan via `kesh_core::chart_of_accounts::load_chart("Pme")` + `accounts::bulk_create_from_chart`, créer un contact, créer puis **valider** une facture pour produire son écriture.

  ⚠️ **Pas de produit — la rédaction d'origine en demandait un, à tort.** `NewInvoiceLine` ne porte **aucun** `product_id` (`entities/invoice.rs:90-98`) : le modèle de facturation ne lie pas les lignes à un catalogue, et le compte de produit d'une ligne vient de `revenue_account_id`, pas d'un article. Un `products::create` dans la fixture serait une ligne sans lecteur — du montage qui n'influence aucune assertion, donc du bruit qui laisse croire à une dépendance inexistante. *(Écart relevé en passe 1 de `bmad-code-review` : la tâche était cochée pour un geste non fait. C'est l'AC qui avait tort, pas l'implémentation.)*

  ⚠️ **Deux prérequis durs de `validate_invoice` que ni `seed_role` ni les patrons ci-dessus ne fournissent** — les oublier fait échouer la validation, donc ne produit **aucune écriture**, donc rend C1 vert sur un ensemble vide :
  - un **exercice ouvert couvrant la date de facture** (`invoices.rs:1730-1733`, `DbError::FiscalYearInvalid` sinon) ;
  - la ligne `company_invoice_settings` **avec `default_vat_payable_account_id`** dès que la facture porte de la TVA (`invoices.rs:1497-1500`, `ConfigurationRequired`). `insert_with_defaults` ne renseigne **pas** cette colonne. Alternative acceptable : monter la facture **sans TVA** (`vat_rate = 0` sur chaque ligne, `total_vat` restant nul évite `ConfigurationRequired`) — cas suisse légitime sous le seuil de CHF 100 000. ⚠️ **C'est une combinaison inédite, à ne pas chercher toute faite dans 16-1a-bis** : ses tests montés exonérés (`backfills_when_vat_config_is_null`) construisent leurs écritures en **SQL brut**, et son seul test passant par le vrai moteur (`validated_invoice_from_the_real_engine_is_recovered_by_the_backfill`) utilise **deux taux non nuls** et une fixture qui configure le compte de TVA. Il faut reproduire l'exonération **par le vrai moteur**.

  ⚠️ **Deux patrons distincts, ne pas confondre.** Pour le **plan comptable** : `crates/kesh-db/tests/company_invoice_settings_repository.rs:33-37`.

  ⚠️ **NE PAS s'inspirer d'`accounts_role_backfill.rs`** — son doc-module (`:28-32`) dit littéralement « insère le plan **en SQL brut** (surtout pas via `bulk_create_from_chart`, qui binderait `role`/`postable` — colonnes qui n'existent pas encore à ce stade → `ERROR 1054`) ». Sa technique repose sur un **montage partiel des migrations**, structurellement impossible ici : `admin_full_import_e2e.rs` monte `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]`, donc **toutes** les migrations. Transposé, il produirait un plan à `role = NULL` partout — exactement le défaut que cette mise en garde existe pour empêcher. *(Cité à tort comme patron en passe 1 de cette story ; c'est un contre-exemple.)* **PAS** `seed_accounting_company` (`test_fixtures.rs`), qui insère **5 comptes en dur** (`1000`, `1100`, `2000`, `3000`, `4000`) **sans jamais renseigner `role`** et **sans le compte `2979`** — C5 serait alors inconstructible, son candidat n'existant pas. Pour la **facture validée par le vrai moteur** : `crates/kesh-db/tests/invoice_lines_revenue_account_backfill.rs`, test `validated_invoice_from_the_real_engine_is_recovered_by_the_backfill`.
- **AC-D3 — Discrimination prouvée par mutation, pas constatée.** Les **six** mutations de `T-D3` sont **exécutées**, et leurs résultats consignés au Dev Agent Record. *(Cinq à l'implémentation ; la sixième ajoutée en passe 1 de `bmad-code-review`, avec l'assertion qu'elle prouve.)* **Un test attendu qui ne rougit pas invalide le montage** : le corriger avant d'aller plus loin. Un test vert ne prouve rien par lui-même — c'est l'enseignement mesuré des stories 16-1a et 16-1a-bis.
- **AC-D4 — Aucun fichier de production modifié.** Cette story n'ajoute que des tests. `git diff --stat` ne touche ni `crates/kesh-db/src/`, ni `crates/kesh-api/src/`, ni `frontend/`, ni aucune migration.

---

## Tasks / Subtasks

- [x] **T-D1 — Fixture métier et helpers de montage** (AC-D1, AC-D2)
  - [x] Helper de montage d'une société complète : plan comptable via `bulk_create_from_chart`, contact. *(« produit » retiré en passe 1 de `bmad-code-review` — cf. AC-D2 : les lignes de facture ne référencent aucun article.)*
  - [x] Helper de création puis **validation** d'une facture, produisant son écriture comptable.
  - [x] Helper `strip_column(manifest, data, table, column)` sur le couple `(Value, BTreeMap<String, Vec<u8>>)` rendu par `unzip` — geste partagé par C1, C2, C2-bis, C5.
- [x] **T-D2 — Tests de bout en bout** (AC-D1)
  - [x] Les **6** cas E2E C1, C1-bis, C2, C2-bis, C3, C5, en réutilisant les helpers de **T-D1** et le harnais existant `spawn_app` / `export_backup` / `unzip` / `rezip` / `post_import`. **Ne pas réécrire les helpers de T-D1.**
  - [x] *(**C4 n'est PAS à écrire ici** — le test de transactionnalité appartient à **16-1c, T5**. Sa ligne au tableau d'AC-D1 et sa note de montage ne sont conservées que pour expliquer pourquoi cette story porte **six** cas et non sept.)*
  - [x] Note de montage « ce que ce test discrimine » sur **tous** les cas ; en particulier les pièges nommés de C1, C3 et C5 *(celui de C4 est informatif, cf. ci-dessus)*.
  - [x] Assertions sur le rapport de rejeu : `backfills_replayed` relu dans `audit_log`, et `outcome` / `rows_affected` de la valeur de retour — dont **un cas où `outcome` vaut `Skipped`** (C3), sans quoi ce troisième état ne serait vérifié nulle part.

    ⚠️ **Les deux côtés n'ont pas la même représentation, et c'est délibéré.** Côté **valeur de retour**, `outcome` est l'enum `ReplayOutcome` — comparer à `ReplayOutcome::Skipped`. Côté **`audit_log`**, le JSON porte le **code d'archive stable** rendu par `ReplayOutcome::code()` : `"SKIPPED"`, `"REPLAYED_UNCONDITIONAL"`, `"REPLAYED_SENTINELS_ABSENT"`, plus une clé `missing_sentinels` (tableau, vide hors classe B déclenchée). Ne **pas** asserter un `format!("{:?}")` de l'enum : c'était la rédaction d'origine de 16-1c, corrigée en passe 1 de sa revue de code — une chaîne de debug Rust dans une archive relue des années plus tard dérive au premier renommage de variant. Les codes sont figés par le test `replay_outcome_codes_are_stable` de `kesh-db`.

  ⚠️ **Chaque cas asserte l'entrée QU'IL discrimine, retrouvée par sa `version`** — jamais la longueur du vecteur, jamais son ordre. Sans cette borne, deux mutations font varier le rapport de cas annoncés **verts** : la mutation 1 bascule l'entrée `20260729000001` en `Skipped` dans C3, et la mutation 5 réduit le vecteur à un élément, observé par C2, C2-bis, C3 et C5. *(Le contrat du rapport est posé par 16-1c, AC-C7 ; cette story l'exerce.)*
- [x] **T-D3 — Preuve par mutation** (AC-D3)
  - [x] Classe A rendue conditionnelle (sentinelle `(invoice_lines, revenue_account_id)`) → **C1-bis** doit rougir, **C1** rester vert.
  - [x] Sentinelles en ET au lieu de OU → **C2-bis** doit rougir.
  - [x] Classe B rendue inconditionnelle → **C3** doit rougir.
  - [x] Registre parcouru en ordre décroissant → **C5** doit rougir.
  - [x] Registre vidé de l'entrée `20260729000001` → **C1 et C1-bis** doivent rougir *(prouve la non-vacuité du cas nominal, qui sans cela pourrait passer sur un ensemble vide — les deux tombent, l'entrée retirée étant la seule à toucher `invoice_lines`)*.
  - [x] **Backfill `postable` #1 (structurel) retiré de l'extrait de classe B → C2 doit rougir, les cinq autres cas rester verts.** *(Mutation ajoutée en passe 1 de `bmad-code-review`, avec l'assertion qu'elle prouve — un patch de revue vient avec son test.)*
  - [x] Consigner les **six** résultats dans le Dev Agent Record. **Un test attendu qui ne rougit pas invalide le montage** : le corriger avant d'aller plus loin.
- [x] **T-D4 — Gate et contrôle de périmètre** (AC-D3, AC-D4)
  - [x] **AC-D4** : `git diff --stat` ne montre **aucun** fichier sous `crates/kesh-db/src/`, `crates/kesh-api/src/`, `frontend/` ni `crates/kesh-db/migrations/`. Si l'écriture d'un test exige une modification de production, c'est que 16-1c est incomplète — **la corriger là-bas**, pas ici.
  - [x] `scripts/test-fast.sh` complet (fmt + clippy `-D warnings` + nextest workspace) sur l'**état final**, exit 0 exigé, non présumé d'un run antérieur. `npm run check` inutile — aucun fichier frontend touché.

---

## Dev Notes

### Les pièges de montage sont dans les AC, pas ici — les lire

Les notes de montage de **C1, C3, C4 et C5** énoncent chacune un piège qui rend le test muet ou inversé. Ils ont coûté trois passes de revue à 16-1c et **aucun n'est détectable à la lecture du test écrit** : ils ne se voient qu'en confrontant le montage au SQL de la migration et aux règles métier du dépôt. Ne pas les paraphraser, ne pas les abréger.

### Limitation assumée — le miroir « avoir » n'est pas exercé de bout en bout

*(Relevé en passe 1 de `bmad-code-review`, EdgeCaseHunter. Reclassé en limitation documentée, pas en patch.)*

L'entrée de classe A rejoue le fichier `20260729000001` **en entier**, et ce fichier porte **deux** `UPDATE` : celui des `invoice_lines` (`:250`) et son miroir `credit_note_lines` (`:285`, structurellement distinct — `debit` au lieu de `credit`, `credit_notes.total_amount`, statut `issued`, jointure sur `cn.journal_entry_id`). **Aucun des six cas ne monte d'avoir** : le second statement tourne donc toujours sur un ensemble vide et n'est jamais vérifié fonctionnellement par un import réel.

Ce n'est pas un test vert pour la mauvaise raison, et la couverture n'est pas absente — elle est **ailleurs, et complète** :

- la **justesse SQL** du miroir est tenue par `crates/kesh-db/tests/invoice_lines_revenue_account_backfill.rs`, dont le doc-module recense nommément deux tests écrits parce que muter le seul `<=>` puis le seul `LEFT` du **2ᵉ** `UPDATE` laissait toute la suite verte ;
- le **découpage en statements** — le seul risque propre au chemin de rejeu, qui pourrait faire disparaître silencieusement le 2ᵉ `UPDATE` — est tenu par `statement_splitting_survives_the_real_traps` et `extract_carries_every_write_statement_of_its_source_migration` (16-1c) ;
- et C1 asserte désormais `rows_affected` **au décompte exact** (`== 2`), ce qui épingle la contribution nulle du miroir plutôt que de la laisser sous une borne lâche.

Ce qui resterait à gagner par un septième cas — monter un avoir émis dans la fixture — est la preuve de bout en bout que le miroir traverse *aussi* le restore. Le coût (fixture d'avoir émis, avec son écriture) est réel ; le gain marginal est faible, la couche d'interaction étant identique pour les deux statements, dans la même transaction et le même découpage. **AC-D1 énumère six cas, et n'a jamais porté les avoirs.** À reconsidérer si une entrée de classe A future portait un statement dont le chemin de rejeu diffère réellement.

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

**Claude Opus 5 (1M context)** — `bmad-dev-story`, 2026-08-02.

### Debug Log References

**Gate complet VERT sur l'état final, DB `kesh_gate` : `exit 0`, 2102/2102, 4 skipped, 3530 s (59 min)** — `fmt` + `clippy -D warnings` dans le même run. Verdict lu **dans le log**, jamais dans le code de retour du bloc englobant. Le total passe de 2096 à 2102, soit **exactement +6** : le décompte concorde avec les six cas exigés par T-D2. *(Un compte de tests n'est un indicateur de couverture que si sa composition est vérifiée — l'enseignement du CRITICAL de la passe 1 de revue de 16-1c.)*

**Les six mutations de T-D3, exécutées, avec leur sortie.** Les cinq premières portent sur `crates/kesh-db/src/post_restore.rs`, la sixième sur l'extrait SQL de classe B `crates/kesh-db/src/post_restore/20260722000001_accounts_role_postable.sql`. Fichier de production **restauré à l'identique** après chaque essai — `git diff` vide, vérifié.

| # | Mutation | Attendu | Obtenu |
|---|---|---|---|
| 1 | classe A rendue **conditionnelle** (sentinelle `(invoice_lines, revenue_account_id)`) | C1-bis rouge, **C1 vert** | ✅ après correction du montage de C1 — cf. ci-dessous |
| 2 | sentinelles en **ET** au lieu de OU | C2-bis rouge, C2 vert | ✅ C2-bis `FAILED`, C2 `ok` |
| 3 | classe B rendue **inconditionnelle** | C3 rouge | ✅ C3 `FAILED` |
| 4 | registre parcouru **à l'envers** | C5 rouge | ✅ C5 `FAILED` |
| 5 | registre **vidé** de `20260729000001` | C1 **et** C1-bis rouges | ✅ les deux `FAILED`, et C3 reste `ok` (contrôle négatif) |
| **6** | backfill `postable` **#1** (structurel) retiré de l'extrait de classe B | **C2 rouge** | ✅ `FAILED` sur `:1094` — l'assertion du compte titre ; **15/16 `ok`**, seul C2 tombe |

**La mutation 6 est ajoutée en passe 1 de `bmad-code-review`** : elle prouve l'assertion que cette passe a ajoutée, plutôt que de la constater verte. Sans elle, le patch se serait contenté d'une ligne qui passe — et la story enseigne précisément qu'un test vert ne prouve rien par lui-même. Sortie retenue :

```
thread 'full_import_replays_role_and_postable_when_both_columns_absent' panicked at
crates/kesh-api/tests/admin_full_import_e2e.rs:1094:5:
le backfill STRUCTUREL de `postable` doit aussi avoir été rejoué : 10 a des enfants actifs et aucune écriture
     Summary [45.696s] 16 tests run: 15 passed, 1 failed, 0 skipped
```

Le **contrôle négatif** est ce qui donne sa valeur à la mesure : C1, C1-bis, C2-bis, C3 et C5 restent verts. La mutation ne tue que le cas qu'elle vise, ce qui établit que la nouvelle assertion discrimine le backfill #1 **et lui seul** — elle ne recouvre pas l'assertion sur `2979` qui la précède. Fichier de production restauré à l'identique après l'essai (`git diff` vide sous `crates/kesh-db/`).

⚠️ **La mutation 1 a corrigé un montage, elle n'a pas seulement constaté une couleur — et c'est le fait marquant de cette story.** Au premier essai, elle faisait rougir **C1 et C1-bis**, là où la spec prédit « C1-bis rouge, C1 vert ». La cause n'était pas dans le code de production mais dans **C1**, qui assertait `outcome == "REPLAYED_UNCONDITIONAL"` : il épinglait donc la **classe déclarée**, rôle que la spec assigne à **C1-bis** et à lui seul. Les deux cas se recouvraient, et la mutation ne pouvait plus mesurer ce qu'elle vise — distinguer *« le mécanisme fonctionne »* de *« l'entrée est bien en classe A »*.

Assertion de C1 ramenée à « l'entrée a été **rejouée** », sans épingler à quel titre. Après correction : C1 `ok`, C1-bis `FAILED`. **Une divergence entre prédiction et mesure est un résultat, pas un incident** : c'est elle qui a révélé le recouvrement.

**Gate.** Un premier run a échoué en `exit 101` — `clippy::cmp_owned` sur `e["version"] == Value::from(version)` (comparaison qui alloue). Corrigé en `e["version"] == version`. **Ce cycle était évitable** : `fmt` + `clippy` auraient dû être passés *avant* de lancer le gate complet, pas découverts au bout du run. C'est le pré-vol de la § « Test Locally First », appliqué à l'envers.

**Un échec de décodage a coûté un aller-retour, et il est réutilisable** : `audit_log.details_json` est de type `JSON`, que MariaDB expose en **`BLOB`** — le lire en `String` échoue au niveau du protocole (`ColumnDecode … not compatible with SQL type BLOB`). Le décoder en `Vec<u8>` puis `serde_json::from_slice`.

### Completion Notes List

- **La fixture métier est le livrable le moins visible et le plus déterminant.** `seed_role` ne crée qu'une société et un utilisateur : sans plan comptable, sans exercice, sans facture validée, **C1 — le cas nominal de l'issue #281 — aurait porté sur un ensemble vide et serait vert pour rien**. C'est exactement ce que la mutation 5 mesure, et c'est pourquoi elle fait partie du jeu.
- **Le plan vient de `bulk_create_from_chart`, jamais de `seed_accounting_company`** : ce dernier insère 5 comptes en dur **sans `role`** et **sans le compte `2979`** — C5 y serait inconstructible, son candidat n'existant pas, et C2 n'aurait aucun rôle à réattribuer.
- **Facture sans TVA, validée par le vrai moteur.** `insert_with_defaults` ne renseigne pas `default_vat_payable_account_id`, que `validate_invoice` exige dès qu'il y a de la TVA. L'exonération est un cas suisse légitime, et cette combinaison — exonéré **et** par le moteur — n'existait nulle part dans le dépôt : les tests exonérés de 16-1a-bis montent leurs écritures en SQL brut, et son seul test passant par le moteur utilise deux taux non nuls.
- **Écart assumé sur la signature de `strip_column`.** La tâche T-D1 la prescrit à quatre arguments, `(manifest, data, table, column)`. Elle en porte **trois** : les octets NDJSON ne sont volontairement pas touchés — c'est précisément ce qui garde `sha256` et `rowCount` valides — et passer `data` laisserait croire qu'il est modifié. La substance de la tâche est tenue ; seule la signature diffère.
- **Aucun fichier de production modifié** (AC-D4) : `git show --stat` liste **trois** fichiers — `crates/kesh-api/tests/admin_full_import_e2e.rs`, ce story file et `sprint-status.yaml`. Un seul est du code, et il est sous `tests/`. Aucun fichier sous `crates/kesh-db/src/`, `crates/kesh-api/src/`, `frontend/`, ni aucune migration. *(La rédaction d'origine disait « ne montre **que** le `.rs` » — faux au pied de la lettre, dans une story dont le thème est justement de recompter plutôt que de relire. Corrigé en passe 1 de `bmad-code-review`.)*
- **C4 n'est pas ici** : le test de transactionnalité vit en test d'intégration `kesh-db` et relève de 16-1c. Cette story porte **six** cas, pas sept.

### File List

**Modifiés**

- `crates/kesh-api/tests/admin_full_import_e2e.rs` — fixture métier `seed_business` (plan comptable réel, exercice ouvert, réglages, contact), helper `validated_invoice` (facture exonérée validée par le vrai moteur), helpers `strip_column` / `backfill_report` / `entry` / `line_accounts` / `account_role` / `account_postable` / `import_ok`, la constante `TITLE_ACCOUNT`, et les **six** cas E2E C1, C1-bis, C2, C2-bis, C3, C5 avec leur note « ce que ce test discrimine ». *(`import_ok` et `TITLE_ACCOUNT` ajoutés en passe 1 de `bmad-code-review`.)*

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

### Passe 2 de `bmad-create-story validate`

**2026-08-01 — Haiku, 2 lentilles, contexte frais. 1 finding : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 0 LOW.**

**BHD2-1 (MEDIUM) — contradiction interne, et encore un résidu de propagation.** La ligne du tableau d'AC-D1 décrivait le montage de C3 comme « `role` / `postable` posés **à la main** sur des valeurs non standard », alors que sa propre note de montage — ajoutée en passe 5 de 16-1c — dit que **ce montage-là produit un test muet** et prescrit un **rôle délibérément effacé**. J'avais écrit la note sans corriger la ligne qu'elle contredit. Un dev concevant C3 d'après le tableau seul aurait construit le test muet que la note existe pour empêcher. Ligne du tableau réalignée.

**L'EdgeCaseHunter rend 0 finding, à créditer avec réserve.** Sa section « vérifié et jugé sain » est bien énumérée et ses conclusions tiennent, mais son **détail contient des inexactitudes** : elle cite un « T-D5 » et un « AC-D7 » qui n'existent pas (`grep -c` → 0 pour les deux ; le document ne déclare que `T-D1..T-D4` et `AC-D1..AC-D4`). Conformément à l'enseignement du dépôt, un « 0 finding » n'est opposable que sur **ce qu'il énumère exactement** : les vérifications d'ancres SQL et de décomptes de ce rapport sont recevables, son inventaire des tâches ne l'est pas.

### Passe 3 de `bmad-create-story validate`

**2026-08-01 — Opus, une lentille cumulant les trois regards, contexte frais. 8 findings : 0 CRITICAL, 0 HIGH, 6 MEDIUM, 2 LOW.**

**Le finding le plus instructif : ma remédiation de passe 1 citait un CONTRE-EXEMPLE comme patron.** Pour corriger le patron de fixture, la passe 1 avait nommé `accounts_role_backfill.rs` — dont le doc-module dit littéralement « insère le plan **en SQL brut** (surtout pas via `bulk_create_from_chart`, qui binderait `role`/`postable` — colonnes qui n'existent pas encore à ce stade → `ERROR 1054`) ». Sa technique repose sur un **montage partiel des migrations**, structurellement impossible dans un fichier qui monte `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]`. Transposée, elle aurait reproduit exactement le défaut que la passe 1 corrigeait : un plan à `role = NULL` partout. L'autre ancre de la même puce était décalée de cinq lignes et pointait sur `insert_with_defaults`. **Les deux citations de la seule puce qui comptait étaient donc fausses.**

**Le finding le plus utile au dev : la fixture omettait deux prérequis durs de `validate_invoice`.** Un **exercice ouvert** couvrant la date de facture (`invoices.rs:1730-1733`, `FiscalYearInvalid` sinon), et `default_vat_payable_account_id` dès qu'il y a de la TVA (`:1497-1500`, `ConfigurationRequired`) — que `insert_with_defaults` ne renseigne pas. Le patron d'origine les tirait tous deux de `seed_accounting_company` ; en scindant en « deux patrons distincts », la passe 1 les a laissés dans **aucun** des deux. Un dev suivant AC-D2 à la lettre obtenait un échec de validation, donc aucune écriture, donc **C1 vert sur un ensemble vide** — le mode d'échec que cette story existe pour empêcher.

**Une affirmation que j'avais servie à l'appui du split était fausse.** Le tableau de provenance annonçait « montages de test — **100 %** » pour les passes 4 et 5. Recompté contre le Change Log de 16-1c : **3 sur 6** et **7 sur 11**. Trois findings de la passe 4 et quatre de la passe 5 portaient sur des éléments **restés en 16-1c** (en-tête de T3, décompte de T4, AC-C6.4, AC-C6.6, et la requalification de la garde des 10 `UPDATE` de rôle — qui **est** un amendement de D-C1). La conclusion tient, les proportions la soutiennent, mais c'était un nombre relu et non recompté, dans la section même qui justifiait un arbitrage de process. Corrigé, et l'exception à « aucune décision `D-C` n'a bougé » est désormais énoncée.

**Deux mutations faisaient varier le rapport de cas annoncés verts** (P3-6) : la mutation 1 bascule l'entrée `20260729000001` en `Skipped` dans C3, la mutation 5 réduit le vecteur à un élément observé par quatre cas. La portée des assertions est désormais bornée — chaque cas asserte **l'entrée qu'il discrimine, retrouvée par sa `version`**, jamais la longueur ni l'ordre du vecteur.

**Résidus de split corrigés côté 16-1c** (P3-4) : un paragraphe y affirmait encore que son AC-C10 porte les six cas partis en 16-1d, et deux renvois pointaient sur la tâche `T6` supprimée.

LOW : « aucun des **sept** cas » (décompte pré-split), et la puce `strip_column` rattachée à un AC qui ne la couvre pas.

**Ce que la lentille a validé positivement, avec commandes** : les 10 ancres du harnais E2E, la sûreté du retrait de colonne établie sur `parse_ndjson_rows:286-305` (et non par analogie), la fenêtre d'importabilité, l'atteignabilité du montage de C3 par l'API (`update` ne garde le retrait de rôle par aucun contrôle), les trois interdits du montage de C5, la présence de `1100` et `2979` dans le plan cité, et **les cinq mutations tracées une par une contre le SQL — aucune muette**. La contradiction de la passe 2 est confirmée close sans résidu.

### Passe 4 de `bmad-create-story validate`

**2026-08-01 — Sonnet, une lentille cumulant les trois regards, contexte frais. 4 findings : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 1 LOW.**

**Fait notable : les trois MEDIUM portent sur 16-1c, pas sur cette story.** La lentille conclut explicitement que « la story elle-même est prête pour dev, mais 16-1c porte encore trois résidus de fork non propagés ». Le gisement s'est déplacé du contenu vers la **jointure entre les deux documents**.

- La note de montage de **C5** — le repointage de `2979`, les deux interdits, les trois puces d'inatteignabilité API — était dupliquée **verbatim** dans les deux stories. Ce montage a déjà été réécrit **trois fois** ; une quatrième correction appliquée à une seule copie les aurait fait diverger en silence. Retirée de 16-1c au profit d'un renvoi.
- « aucun des **sept** cas » subsistait dans 16-1c — le décompte pré-split. La passe 3 avait corrigé la copie de 16-1d **sans greper le symptôme** dans la story sœur : le mode d'échec exact que la § « Propagation post-patch » de `CLAUDE.md` codifie, commis sur le patch qui corrigeait un autre renvoi mort.
- `AC-C7` de 16-1c exigeait l'assertion `Skipped` « dans au moins un cas de **AC-C10** » alors que la passe 3 venait d'écrire, 28 lignes plus bas, qu'`AC-C10` ne porte plus que C4 — un document qui se contredit lui-même sur le point que le split était censé clarifier. Redirigé vers `16-1d, AC-D1`.
- LOW : « c'est le montage qu'utilise 16-1a-bis » était imprécis pour la facture sans TVA. Ses tests exonérés montent leurs écritures en **SQL brut**, et son seul test passant par le vrai moteur utilise **deux taux non nuls**. La combinaison « exonéré **et** par le vrai moteur » est inédite et doit être construite — c'est dit désormais.

**Validé positivement** : les six cas, les dix ancres du harnais, les deux prérequis de `validate_invoice`, la couverture AC ↔ tâches sans orphelin ni doublon, et les **cinq mutations** re-tracées contre le SQL — aucune muette, et leurs effets de bord inter-cas correctement neutralisés par la règle de bornage des assertions posée en passe 3. Le recompte indépendant du « 3 sur 6 » de la passe 4 de 16-1c est confirmé exact.

### Passe 5 de `bmad-create-story validate` — **BOUCLE CONVERGÉE**

**2026-08-01 — Haiku, une lentille dédiée à la JOINTURE des deux documents, contexte frais. 1 finding : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW.**

**Critère d'arrêt de la § « Review Iteration Rule » atteint** : plus rien au-dessus de `LOW`. Plafond de 8 passes non atteint.

L'unique LOW est un renvoi historique : le Change Log de 16-1c citait encore « la mutation 5 de **T6** » — tâche supprimée par le split, dont la note explicative se trouve 161 lignes plus haut. Reformulé en renvoi explicite vers `16-1d T-D3`.

**Ce que la passe a établi, et qui était l'enjeu réel** : la **table de jointure** des 18 éléments du périmètre montre que chacun est couvert exactement une fois — mécanisme, deux classes, ordre, registre, extrait, câblage, six garde-fous, les sept cas, les cinq mutations, la documentation, le gate. **Aucun élément perdu au split, aucun dupliqué.** Les 9 renvois de 16-1c vers 16-1d et les 13 renvois inverses résolvent tous. Le retrait de la note de montage de C5 hors de 16-1c est confirmé effectif (`grep REPOINTANT` → 1 occurrence en 16-1d, 0 en 16-1c), et la chaîne `AC-C7 → 16-1d AC-D1 → C3 → outcome Skipped` est vérifiée de bout en bout.

**Trend de la boucle** : `1 HIGH / 4 MED / 1 LOW` → `1 MED` → `6 MED / 2 LOW` → `3 MED / 1 LOW` → **`1 LOW`**. Rotation Sonnet → Haiku → Opus → Sonnet → Haiku.

**Fil rouge des cinq passes** : le gisement s'est déplacé de manière lisible — le **contenu** de la story aux passes 1-3, la **jointure avec 16-1c** aux passes 4-5. Et à chaque passe sauf la dernière, la majorité des findings corrigeait un artefact d'une passe antérieure. Le cas le plus net : pour corriger un patron de fixture erroné, la passe 1 a cité un fichier dont le doc-module **interdit explicitement** le geste prescrit — un contre-exemple présenté comme exemple, découvert deux passes plus tard.

**Statut : `ready-for-dev`, spec convergée.**


### Passe 1 de `bmad-code-review`

**2026-08-03 — Sonnet, trois lentilles (BlindHunter, EdgeCaseHunter, AcceptanceAuditor), contexte frais. 12 findings bruts → 3 MEDIUM, 6 LOW, 3 écartés dont 1 réfuté en ground-truth.** Modèle différent de celui qui a spécifié et implémenté la story (Opus), conformément à la § « Review Iteration Rule ».

**Le finding qui compte : un `UPDATE` entier de l'entrée de classe B n'était vérifié par aucun cas** (EdgeCaseHunter, MEDIUM). `20260722000001` porte **deux** backfills de `postable` — le #2 par rôle (`role = 'CurrentYearResult'`, `:177`) et le #1 **purement structurel** (`:161`, tout compte à enfants actifs jamais mouvementé). Les trois assertions d'imputabilité des six cas portaient **toutes sur le seul compte `2979`**, donc sur le seul backfill #2 :

```
$ grep -n "account_postable" crates/kesh-api/tests/admin_full_import_e2e.rs
496:        !account_postable(&pool, "2979").await,
568:        !account_postable(&pool, "2979").await,
737:        !account_postable(&pool, "2979").await,
```

Et la colonne est `NOT NULL DEFAULT TRUE` (`:77`) : retirée du manifeste, elle **revient à `TRUE` sur tout le plan**, comptes titres compris. Une régression de la sous-requête `EXISTS`/`NOT EXISTS` du backfill #1 — ou sa disparition au découpage en statements — aurait donc rendu un compte de regroupement imputable après restore **sans qu'aucun cas ne rougisse**. C2 asserte désormais aussi le compte titre `10` (parent de `1000`/`1010`/`1020`, jamais mouvementé), en pré-condition **et** après import, plus `rows_affected > 0` pour interdire un rejeu déclenché mais muet.

**L'AcceptanceAuditor a trouvé une tâche cochée pour un geste non fait** (MEDIUM) : AC-D2 et T-D1 exigeaient de « créer contact **et produit** », et `products::create` n'est appelé nulle part. Vérification faite, **c'est l'AC qui avait tort** : `NewInvoiceLine` ne porte aucun `product_id` (`entities/invoice.rs:90-98`), le modèle ne lie pas les lignes à un catalogue. Un produit dans la fixture aurait été du montage sans lecteur. AC et tâche corrigées plutôt que le code.

**Un troisième MEDIUM est reclassé en limitation documentée** (EdgeCaseHunter) : le miroir `credit_note_lines` du backfill de classe A n'est exercé par aucun des six cas, faute d'avoir dans la fixture. La justesse SQL du miroir est tenue par quatre tests dédiés de `invoice_lines_revenue_account_backfill.rs` — dont deux existent précisément parce que muter le 2ᵉ `UPDATE` laissait le reste vert — et le seul risque propre au chemin de rejeu (un statement perdu au découpage) est tenu par `statement_splitting_survives_the_real_traps`. Rationale complète en Dev Notes ; C1 asserte en compensation `rows_affected` au **décompte exact** (`== 2` au lieu de `>= 2`), ce qui épingle la contribution nulle du miroir au lieu de la cacher sous une borne lâche.

**Un finding réfuté par grep ground-truth.** Le BlindHunter donnait MEDIUM sur `assert_eq!(missing, vec!["accounts.role", "accounts.postable"])`, soupçonnant un ordre non garanti (itération de `HashSet` côté production) et donc un test *flaky*. Faux : `missing_sentinels` (`post_restore.rs:420-432`) itère sur le **slice de sentinelles du registre**, dans son ordre de déclaration, et `filter`/`map` le préservent. L'ordre est déterministe et contractuel. Écarté.

**Deux autres écartés** : le non-scoping par `company_id` de `account_role`/`account_postable` (`fetch_one` échoue **bruyamment** sur multi-lignes — aucun verdict faux possible, et chaque `#[sqlx::test]` du fichier n'instancie qu'une société) ; et les codes d'issue en littéraux plutôt qu'en constantes (une faute de frappe rend le test **rouge**, jamais faussement vert, et `replay_outcome_codes_are_stable` verrouille déjà les chaînes côté `kesh-db`).

**LOW appliqués** — cinq nettoyages sans effet sur la discrimination des cas :

| # | Finding | Correction |
|---|---|---|
| 1 | `rows_affected >= 2` en C1, borne lâche sur une base fraîche | `== 2`, avec la raison du décompte |
| 2 | paramètre `number_hint` de `validated_invoice` accepté puis jeté (`let _ = number_hint;`) | supprimé, 3 sites d'appel mis à jour |
| 3 | C5 forçait `revenue_account_id = NULL` par `UPDATE` **et** par `strip_column` | `UPDATE` retiré — le `strip_column` seul produit l'état, comme en C1 ; la raison est écrite sur place |
| 4 | `rezip` + `post_import` + `assert 200` copiés **six** fois (DRY, § « Code Quality Rules ») | helper `import_ok` ; les tests antérieurs gardent leur `post_import` nu, ils attendent des `4xx` |
| 5 | la numérotation saute C3 → C5 sans explication lisible dans le fichier | note en tête expliquant que C4 vit en test d'intégration `kesh-db` |

**Un LOW sur le Dev Agent Record lui-même** : il affirmait que `git diff --stat` « ne montre **que** le `.rs` », alors que le commit touche **trois** fichiers (le `.rs`, ce story file, `sprint-status.yaml`). L'exigence d'AC-D4 — aucun fichier de production — est bien tenue ; c'est la formulation qui était un nombre relu et non recompté, dans une story dont c'est le thème. Corrigée.

**Ce que les trois lentilles ont validé positivement**, chacune avec ses commandes : les six cas présents et discriminants, l'absence de C4, les quatre pièges de montage nommés (C1 non-vacuité, C3 rôle effacé, C5 repointage sans ajout de ligne ni écriture séparée) tous évités, la résolution des entrées du rapport **par `version`** sans aucun index ni longueur, les trois codes d'issue conformes à `ReplayOutcome::code()` et les trois variants exercés, l'intégrité du backup forgé contre `parse_and_verify` (`sha256`, `rowCount`, inventaire des tables), la fenêtre d'importabilité, la non-vacuité de la fixture établie en remontant jusqu'à `generate_invoice_journal_lines`, et AC-D4 par `git show --stat`.

**Une sixième mutation est née de cette passe, et c'est son résultat le plus utile.** Le patch du MEDIUM #1 ajoutait une assertion — qui passait. Elle aurait pu passer pour de mauvaises raisons (compte titre déjà non imputable pour un autre motif, ou assertion recouvrant celle sur `2979` qui la précède de dix lignes). Le backfill `postable` #1 a donc été retiré de l'extrait de classe B et C2 relancé : `FAILED` sur `:1094`, la ligne exacte, et **15/16 verts** — C1, C1-bis, C2-bis, C3 et C5 intacts. C'est le contrôle négatif qui donne sa valeur à la mesure : la mutation ne tue que le cas visé, donc la nouvelle assertion discrimine le backfill #1 **et lui seul**. Consignée en `T-D3` et au tableau du Dev Agent Record ; fichier de production restauré. *(§ « un patch de review vient AVEC son test » — ici le test est une mutation, la story n'ayant pas d'autre monnaie de preuve.)*

**Gate.** Complet et vert **sur l'état patché**, DB `kesh_gate` : `2102 tests run: 2102 passed, 4 skipped`, 3023 s (50 min), `exit 0` — verdict lu dans le log, jamais dans le code de retour du bloc englobant. Le total reste **2102** : les patches de cette passe ajoutent des **assertions**, pas des tests, et le décompte le confirme. Contrôle de composition, pas seulement de total.

**Décision de process prise pendant cette passe** (arbitrage de Guy, codifié dans `CLAUDE.md` § « Test Locally First » → « Pendant une boucle de revue ») : entre les passes d'une boucle de revue, gate **ciblé** (`fmt` + `clippy` workspace + `nextest -E 'binary(...)'`) ; gate **complet** au push, à la déclaration `done` et au dernier commit de la boucle. Deux réserves : ne jamais écrire « gate vert, N/N » pour un run ciblé, et gate complet obligatoire dès qu'un patch touche `kesh-db`. Le gate complet ci-dessus a été lancé avant cet arbitrage ; la mutation 6 est le premier run ciblé de la nouvelle règle — 50 s au lieu de 50 min.

**Verdict de la boucle : NON CONVERGÉE.** 3 MEDIUM > `LOW` → une passe 2 est requise par la § « Review Iteration Rule », avec un modèle différent (Haiku), contexte frais, sur un diff **aplati** — la § « Haiku-specific guardrails » l'exige dès la passe 2, et tout `CRITICAL`/`HIGH` affirmant l'absence d'un code attendu devra être vérifié par `grep -nF` avant d'être traité comme réel.

### Passe 2 de `bmad-code-review` — **BOUCLE CONVERGÉE**

**2026-08-03 — Haiku, trois lentilles, contexte frais, sur un diff APLATI** (`f2b2fe0d..HEAD`) comme l'exige la § « Haiku-specific guardrails » de `CLAUDE.md` dès la passe 2. **8 findings bruts → 0 CRITICAL, 0 HIGH, 0 MEDIUM, 2 LOW.** Critère d'arrêt de la § « Review Iteration Rule » atteint ; plafond de 8 passes non approché.

**EdgeCaseHunter et AcceptanceAuditor rendent chacun 0 finding**, avec vérification énumérée. L'AcceptanceAuditor produit une table de couverture des 23 exigences d'AC-D1 à AC-D4, toutes conformes, et confirme que la mutation 6 de la passe 1 prouve bien l'assertion qu'elle prétend prouver — l'ancre `:1094` correspond au code livré.

**Le BlindHunter annonçait 1 HIGH et 4 MEDIUM. Quatre sont réfutés par grep ground-truth**, et c'est exactement le cas pathologique que la § « Haiku-specific guardrails » décrit :

| Finding annoncé | Vérification | Verdict |
|---|---|---|
| **HIGH** — `.unwrap()` sur `missing_sentinels` « panique avec un message vide » | `Option::unwrap()` panique avec son message standard **et** le `file:line`. Un test qui panique **échoue** — c'est le comportement voulu, pas un signal avalé | **réfuté**, et sévérité inflationniste |
| **MED** — C1 n'asserte pas que `strip_column` a bien retiré la colonne | `grep -n -A18 "fn strip_column"` → `assert_eq!(cols.len(), before - 1, …)` : **le helper l'asserte lui-même**. Le demander à chaque site d'appel serait redondant | **réfuté** |
| **MED** — idem pour les trois `strip_column` de C5 | même helper, même assertion | **réfuté** |
| **MED** — `rows_affected > 0` en C2 trop lâche : « le backfill dégénère et oublie les comptes titres » | `grep -nF "account_postable(&pool, TITLE_ACCOUNT)"` → **deux occurrences, `:1065` et `:1095`**. Le scénario décrit est **précisément** celui que l'assertion ajoutée en passe 1 attrape, et que la mutation 6 a prouvé discriminant. La lentille a de plus attribué la détection à C2-bis et au compte `2979` — deux erreurs | **réfuté** |
| **MED** — l'assertion `\|\|` de C1 est trop lâche | Décision délibérée, établie **par la mutation 1**, documentée sur huit lignes à l'endroit même du code, et re-validée par deux lentilles de la passe 1 | **écarté** — re-signalement d'une décision documentée |

**Les deux LOW sont réels et appliqués.** Le premier est cosmétique : `assert_eq!(missing, vec!["accounts.postable"])` en C2-bis était la seule assertion du motif sans message d'échec.

**Le second a plus de valeur que sa sévérité ne le dit** : la lentille observait que C2 n'assertait qu'un seul rôle (`1100`) sur la dizaine que le backfill repose. L'objection générale ne justifiait pas d'asserter les dix — mais elle en désignait un qui compte. `2979` est le **maillon d'une chaîne** : le dernier statement de l'extrait rend non imputable « tout compte portant le rôle `CurrentYearResult` », rôle que le neuvième `UPDATE` vient de poser, et le fichier d'extrait porte l'avertissement « ORDRE À PRÉSERVER » pour cette raison précise. Si le neuvième régressait, le dernier ne trouverait plus rien et l'assertion sur `2979` non imputable tomberait **sans dire pourquoi**. Le rôle est désormais asserté explicitement : le maillon rompu est nommé.

**Gate ciblé** — premier usage de la règle codifiée cette session : `fmt --check` OK, `clippy --workspace --all-targets -D warnings` OK, `nextest -E 'binary(admin_full_import_e2e)'` → **16/16 passed**, 70 s. Aucun fichier `kesh-db` touché, le ciblage est donc autorisé ; le gate **complet** sera repassé avant le push, conformément à la réserve de la règle.

**Trend de la boucle** : `3 MED / 6 LOW` → **`0 / 0 / 0 / 2 LOW`**. Rotation Sonnet → Haiku (Opus ayant spécifié et implémenté la story, il est exclu des deux passes).

**Fil rouge des deux passes** : la passe 1 a trouvé un défaut de couverture réel — un `UPDATE` entier invérifié — et l'a corrigé **avec la mutation qui le prouve**. La passe 2 n'a rien trouvé au-dessus de `LOW`, et ses quatre findings les plus graves étaient des artefacts d'indexation ou des re-signalements de décisions documentées. Le seul apport substantiel de la passe 2 est venu d'une objection **mal étayée mais bien dirigée** : « un seul rôle asserté » était trop général pour être un finding, mais désignait un maillon qui méritait de l'être.

**Statut : revue convergée.**
