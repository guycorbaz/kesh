# Story 28.3 : La sauvegarde d'installation lit ses tables dans un instantané commun

## Status

**ready-for-dev** — fiche créée le 2026-10-13 en autonomie (`bmad-create-story`, Opus 5.5), sur `6f971d76`
(rétrospective de l'Epic 27), branche `story/28-3`, worktree `/home/gcorbaz/devel/kesh-28-3`, port **3036**,
bases `kesh_283` / `kesh_e2e_283`. Choix consignés au registre `epic-28-choix-autonomes-28-3.md`
(**C-28-3-1 à C-28-3-24**, à verser par union dans `epic-28-choix-autonomes.md`). 12 AC (AC1 à AC12).
Validation P1 faite (Sonnet, lentilles R et F : 3 MEDIUM, 16 LOW), remédiée (C-28-3-15 à 23) ; validation P2 à
lancer : `28-3-validate-prompt-p2.md` (Opus, lentilles R et F, C-28-3-24).
PR : **`closes #633`**, **`refs #465`** (jamais `closes #465` : C-28-3-1).

## Story

**As an** Administrateur qui sauvegarde une installation où l'on saisit,
**I want** que le fichier `.keshbackup` — celui que je télécharge comme celui que Kesh écrit avant chaque import —
photographie la base **à un seul instant**,
**so that** une saisie, une suppression d'import ou une validation faite pendant la sauvegarde ne laisse jamais dans
le fichier une ligne sans son parent, que la restauration recréerait telle quelle.

Issue : [#633](https://github.com/guycorbaz/kesh/issues/633) (bug, **P2**), relevée en validation P2 de la Story 27-1
(lentille R, R2-1). Jumeau : [#465](https://github.com/guycorbaz/kesh/issues/465) (export de souveraineté CSV, P2),
**hors périmètre** (C-28-3-1). Epic : `_bmad-output/planning-artifacts/epic-28-recette-v0-13-2.md` § *28-3* (fichier
du kickoff, worktree `kesh-28-0`, non encore mergé à la création de cette fiche). Cible : **v0.13.3**.

⚠️ **Pourquoi maintenant** : depuis le 2026-10-13, l'instance du NAS porte de vraies transactions. Une sauvegarde
incohérente n'est plus une hypothèse d'école : c'est la pièce qu'on restaure le jour où il faut restaurer.

### Le défaut, au sol

`build_keshbackup` (`crates/kesh-api/src/admin_backup/export.rs:45`) lit la base en **41 lectures indépendantes
sur le pool** (80 requêtes : 2 + 39 × 2, voir ci-dessous), chacune en autocommit, donc chacune dans sa propre vue :

```rust
let min_required = read_min_required(pool).await.map_err(map_db)?;      // export.rs:46
let instance_id = read_instance_id(pool).await.map_err(map_db)?;        // export.rs:47
for &table in TABLES_TO_TRUNCATE {                                       // export.rs:60
    let export = export_table(pool, table).await.map_err(map_db)?;      // export.rs:61
```

et `export_table` (`crates/kesh-db/src/backup.rs:173`) fait elle-même **deux** requêtes sur le pool
(`non_generated_columns`, `backup.rs:157`, puis le `SELECT` des lignes, `backup.rs:201`). Aucune transaction : deux
tables lues l'une après l'autre peuvent l'être sur deux connexions différentes, à deux instants différents.

`TABLES_TO_TRUNCATE` (`backup.rs:34`) est ordonnée **enfants → parents** : `bank_transactions` est lue **avant**
`bank_imports`, `invoice_lines` avant `invoices`, `journal_entry_lines` avant `journal_entries`, etc. Une suppression
de parent committée entre les deux lectures — `bank_imports::delete_unreconciled_in_tx` (Story 27-1,
`repositories/bank_imports.rs:449`) en est un écrivain atteignable d'un clic — laisse dans le fichier des
transactions **sans leur import**. La restauration (`restore_tables_in_tx`, `backup.rs:444`) charge avec
`FOREIGN_KEY_CHECKS=0` : elle les recrée telles quelles, et rien ne le signale.

Le code le reconnaît lui-même, `repositories/bank_imports.rs:447-448` :

```rust
/// - Une sauvegarde prise pendant une suppression peut porter des transactions
///   orphelines (#633 ; export global CSV, #465).
```

**Deux appelants**, tous deux concernés (`grep -rn "build_keshbackup(" crates --include=*.rs`, hors commentaires) :

| Site | Rôle |
|---|---|
| `crates/kesh-api/src/routes/admin.rs:39` | `full_export` — *Administration → Sauvegarde complète* |
| `crates/kesh-api/src/routes/admin.rs:263` | `run_backup_and_restore` — sauvegarde de sécurité **pré-import**, prise sous le verrou d'installation (`SELECT … FROM _kesh_version … FOR UPDATE`, `admin.rs:239`) |

## Critères d'acceptation

### AC1 — Une seule connexion, un seul instantané, en lecture seule

Toutes les lectures de la sauvegarde — `kesh_version_min_required`, l'identifiant d'installation, et, pour chaque
table de `TABLES_TO_TRUNCATE`, ses colonnes (`INFORMATION_SCHEMA.COLUMNS`) et ses lignes — passent par **une seule
connexion**, dans **une seule transaction** ouverte ainsi, sur cette connexion et dans cet ordre :

```sql
SET TRANSACTION ISOLATION LEVEL REPEATABLE READ;            -- vaut pour la transaction suivante seulement
START TRANSACTION WITH CONSISTENT SNAPSHOT, READ ONLY;
```

- `SET TRANSACTION` **sans** `SESSION` : il ne vaut que pour la transaction suivante et ne laisse rien sur la
  connexion rendue au pool (C-28-3-2). Il est **obligatoire** même si le serveur est en `REPEATABLE READ` par défaut :
  sous `READ COMMITTED`, `WITH CONSISTENT SNAPSHOT` est sans effet et chaque `SELECT` reprend une vue neuve — le
  défaut reviendrait **en silence**.
- `WITH CONSISTENT SNAPSHOT` : la vue est prise **à l'ouverture**, non à la première lecture (AC5 (b)).
- `READ ONLY` : toute écriture dans la transaction est refusée par le serveur (AC5 (c)) ; la transaction n'obtient
  pas d'identifiant d'écriture.
- La transaction se **clôt avant** la compression ZIP (C-28-3-5) — par `ROLLBACK` (rien à valider) — et, sur toute
  erreur, la connexion revient au pool **hors transaction** (le `Drop` de `sqlx::Transaction` émet le `ROLLBACK`).
  Une erreur du `ROLLBACK` final (`finish`) fait **échouer** la sauvegarde (C-28-3-20) : `export_installation` la
  propage, même si toutes les tables ont été lues.
- Ouverture, sur une `PoolConnection` acquise : d'abord le `SET TRANSACTION …` par `conn.execute("SET TRANSACTION …")`
  — un `&str` passé à `Executor::execute`, donc en **protocole texte**, comme le `BEGIN` de sqlx ; **pas**
  `sqlx::query(…)`, qui passerait en protocole préparé (C-28-3-19) ; puis `sqlx::Transaction::begin(conn,
  Some(<instruction>))` sur cette même connexion. `Transaction::begin` (`sqlx-core-0.8.6/src/transaction.rs:99`) est
  exportée mais marquée `#[doc(hidden)]` (l. 98) : hors contrat semver, c'est une **dépendance à épingler** (commentaire
  de `snapshot.rs`, et Dev Notes § *Risque de montée de sqlx*) ; c'est pourtant la seule qui rende un type `'static`
  après un `SET` sur la même connexion (C-28-3-19). Elle contrôle `in_transaction()` après l'instruction
  (`sqlx-mysql-0.8.6/src/transaction.rs:33-34`). ⛔ **Jamais** un `START TRANSACTION` brut par `execute` hors de cette
  API : sqlx ne saurait pas la connexion en transaction et la rendrait au pool **transaction ouverte**.
- Échec du `SET TRANSACTION` : `conn.close_on_drop()` avant de rendre l'erreur — la connexion ne retourne pas au pool
  avec un état incertain. Échec de `Transaction::begin` (la connexion lui a été cédée) : le niveau « pour la prochaine
  transaction » peut rester posé sur une connexion rendue au pool ; résidu **assumé**, écrit au registre (C-28-3-19).

### AC2 — Le type, la structure et deux gardes lexicales tiennent l'inventaire des lectures

Un type neuf, `ReadSnapshot` (module neuf `crates/kesh-db/src/snapshot.rs`, C-28-3-3), enveloppe la transaction de
l'AC1. Il n'a que deux constructeurs — `ReadSnapshot::begin(pool: &MySqlPool)` (public) et
`ReadSnapshot::begin_on(conn: PoolConnection<MySql>)` (`pub(crate)`, pour l'AC5 (a)) — et une fin explicite
(`finish`, qui fait le `ROLLBACK` et rend l'erreur éventuelle). Il donne accès à la connexion pour l'exécution des
requêtes (`&mut MySqlConnection`, par une méthode ou `DerefMut`), jamais au pool ni à la `PoolConnection`.

Les **quatre** fonctions de lecture de la sauvegarde de `backup.rs` prennent `&mut ReadSnapshot` au lieu de
`&MySqlPool` :

| Fonction | Ligne aujourd'hui | Requête |
|---|---|---|
| `non_generated_columns` (privée) | `backup.rs:145` | `INFORMATION_SCHEMA.COLUMNS` |
| `export_table` | `backup.rs:173` | `SELECT <colonnes> FROM <table>` |
| `read_min_required` | `backup.rs:307` | `_kesh_version` |
| `read_instance_id` | `backup.rs:321` | `MIN(id) FROM companies` |

**Ce que chaque mécanisme garantit, et rien de plus** (C-28-3-15) :

1. **Le type** : aucune des **quatre** lectures existantes ne peut plus s'appeler sur le pool — un tel appel ne
   compile pas. Le type n'interdit **pas** d'ouvrir un **second** `ReadSnapshot::begin(pool)` (le constructeur est
   public), ni d'écrire une lecture **neuve** `sqlx::query(…).fetch_*(pool)` là où un pool est en portée.
2. **La structure** : le cœur `export_installation_in` (AC4) ne reçoit **aucun pool** — seulement le `ReadSnapshot`
   et l'observateur ; une lecture neuve écrite **dans le cœur** n'a donc pas de pool sur lequel s'exécuter, et ne
   compile pas tant que sa signature n'en reçoit pas un.
3. **Deux gardes lexicales** couvrent les deux seules fonctions qui tiennent un pool : dans le code de production de
   chacune (corps de la fonction, commentaires `//` et `///` retirés), l'identifiant `pool` (mot entier, `\bpool\b`)
   apparaît **exactement deux fois** — le paramètre et l'unique appel :
   - **(i)** `export_installation` (`kesh-db/src/backup.rs`, test `export_installation_n_ouvre_qu_un_instantane`) :
     `pool: &MySqlPool` et `ReadSnapshot::begin(pool)` ;
   - **(ii)** `build_keshbackup` (`kesh-api/src/admin_backup/export.rs`, `mod tests`, test
     `build_keshbackup_ne_lit_la_base_qu_une_fois`) : `pool: &MySqlPool` et `export_installation(pool)`.
   Assertion de montage de chaque garde : la fonction est trouvée et son corps n'est pas vide (sinon la garde rendrait
   zéro et passerait à vide). Elles tuent M10 et M11.

Ce qui reste hors des gardes, et c'est écrit comme angle mort : une donnée que lirait un **appelant**
(`full_export`, `run_backup_and_restore`) sur le pool et qu'il passerait à `build_keshbackup` par un paramètre neuf
— le compte de (ii) ne bougerait pas ; le détecteur restant est la revue. `column_constraints` (`backup.rs:388`, import) **ne change pas** : il sert
la restauration, non la sauvegarde (AC9).

### AC3 — Une seule fonction de `kesh-db` lit l'installation ; `kesh-api` n'assemble plus que le ZIP

`kesh_db::backup::export_installation(pool: &MySqlPool) -> Result<InstallationExport, DbError>` ouvre le
`ReadSnapshot`, contrôle le moteur des tables sauvegardées (AC4, garde de moteur), lit `min_required`, `instance_id`
et chaque table **dans l'ordre de `TABLES_TO_TRUNCATE`**, clôt l'instantané (`finish`, dont l'erreur se propage :
C-28-3-20), et rend :

```rust
pub struct InstallationExport {
    pub min_required: String,
    pub instance_id: i64,
    /// Une entrée par table de `TABLES_TO_TRUNCATE`, dans son ordre.
    pub tables: Vec<(&'static str, TableExport)>,
}
```

`build_keshbackup` (`admin_backup/export.rs`) **garde son nom et sa signature** (`pool: &MySqlPool`) — la garde
lexicale `avant_sauvegarde_branchee_aux_appels_de_l_import` (`routes/admin.rs:870-914`) compte ses appels — et ne fait
plus **qu'un** appel à la base : `export_installation(pool)`. Le reste (SHA-256, `BackupTableMeta`, entrées
`data/<table>.ndjson`, `files/`, `manifest.json` en dernier, `KeshBackupMeta`) est **inchangé** pour un même état de
base : même ordre d'entrées, mêmes noms, mêmes NDJSON, même manifeste **hors** son horodatage `exportDate`
(`manifest.rs:55`, `Utc::now()` : deux exports d'un même état ne sont donc jamais identiques octet pour octet, ni
avant ni après la story). Ce que contrôlent `admin_full_export_e2e.rs` et consorts est cette structure. Les erreurs restent
`AppError::AdminFullExportFailed` (via `map_db`), converties en `AdminPreImportBackupFailed` par
`avant_sauvegarde` à l'import — comportement inchangé.

### AC4 — Le cœur est observable, pour le test, sans changer le chemin de production

`export_installation` délègue à une fonction **privée** du module (nom au choix, par exemple
`export_installation_in`) qui prend un `ReadSnapshot` déjà ouvert et un **observateur** appelé :

- une fois **après l'ouverture de l'instantané et avant la première lecture** (étape `Ouvert`) ;
- une fois **après la lecture de chaque table** (étape `ApresTable(&'static str)`).

L'observateur a **cette** forme, prescrite (C-28-3-21) : `&mut (dyn FnMut(Etape) -> BoxFuture<'static, ()> + Send)`,
où `BoxFuture` est un alias **local** du module, `Pin<Box<dyn Future<Output = ()> + Send + 'a>>` (std seule :
`kesh-db` ne dépend pas du crate `futures`, et la story n'en ajoute pas — `grep -n futures crates/kesh-db/Cargo.toml`
ne rend rien) ; `Etape` est `Copy`. `AsyncFnMut` est écarté : dans un cœur appelé depuis un handler
axum, dont le futur doit être `Send`, l'inférence d'auto-trait y est fragile. Le test capture dans la fermeture un
clone du pool et un compteur partagé (`Arc<AtomicUsize>`). `export_installation` lui passe un observateur **vide**
(`&mut |_| Box::pin(async {})`). Aucun drapeau, aucune variable d'environnement, aucune `feature` : la production
emprunte **exactement** le chemin testé (C-28-3-6). La signature du cœur ne reçoit **aucun pool** (AC2, point 2).

**Garde de moteur, à l'exécution** (C-28-3-17). Première lecture du cœur, après l'étape `Ouvert` et dans
l'instantané : `SELECT TABLE_NAME, ENGINE FROM information_schema.TABLES WHERE TABLE_SCHEMA = DATABASE()`, filtrée
en Rust sur les tables de `TABLES_TO_TRUNCATE` et `_kesh_version`. Le cœur **échoue fermé**
(`DbError::Invariant`, message nommant la table et son moteur) si l'une n'est pas `InnoDB` — `ENGINE` se lit en
`Option<String>`, comme dans `test_schema_guard.rs:166`, et un `NULL` compte comme « pas InnoDB » —, ou si l'une manque
(compte trouvé ≠ `TABLES_TO_TRUNCATE.len() + 1`). Motif : 13 des 51 `CREATE TABLE` des migrations n'ont pas de clause
`ENGINE` et prennent, sur la base servie, le moteur par défaut du **serveur** — qu'aucun fichier du dépôt ne fixe ;
une garde sur la seule base de test ne verrait pas une table non transactionnelle en exploitation, qui échapperait à
l'instantané sans signal. La fonction de contrôle est `pub(crate)` et rend le nombre de tables contrôlées (AC5 (e1)).

### AC5 — Tests de l'instantané (`crates/kesh-db/src/backup.rs`, `mod tests`, `#[sqlx::test(migrations = "./test-schema")]`)

Montage commun : `seed_accounting_company` (`test_fixtures.rs:80`), un compte bancaire, **un** import portant au
moins **deux** transactions (`bank_imports::create_with_transactions`, `repositories/bank_imports.rs:59` ; modèle de
montage : `crates/kesh-db/tests/bank_transactions_repository.rs:119`). Helper de test
`orphelins(&InstallationExport) -> Vec<String>` : pour **chaque** clé étrangère du schéma lue dans
`information_schema.KEY_COLUMN_USAGE` (`REFERENCED_TABLE_NAME IS NOT NULL`, `TABLE_SCHEMA = DATABASE()`), toute
valeur non nulle de la colonne fille dans le NDJSON de la table fille doit figurer parmi les valeurs de la colonne
référencée dans le NDJSON de la table parente ; le helper rend la liste des violations
(`"bank_transactions.import_id=17 → bank_imports.id absent"`). Assertion de montage du helper : il voit **au moins**
la FK `fk_bank_transactions_import` (sinon il serait muet).

**(a) Suppression concurrente entre deux lectures — le test qui rougit avant la correction.**
L'observateur, à l'étape `ApresTable("bank_transactions")`, supprime l'import **par l'écrivain réel**
(`bank_imports::delete_unreconciled_in_tx` dans une transaction neuve du pool, validée), **sur une autre connexion**.
Attendus :

1. **assertion de montage** : après la sauvegarde, la base ne contient plus l'import ni ses transactions (le geste
   concurrent a bien eu lieu — sans elle, le test passerait à vide) ; l'observateur a bien été appelé pour
   `bank_transactions` (compteur) ;
2. la sauvegarde contient l'import **et** ses transactions (elle montre la base d'avant la suppression) ;
3. `orphelins(&export)` est **vide**.

Le test est exécuté deux fois sur le même montage : **(a-RC)** sur une connexion dont la session a été passée en
`SET SESSION TRANSACTION ISOLATION LEVEL READ COMMITTED` avant `ReadSnapshot::begin_on(conn)` — il prouve que le
`SET TRANSACTION … REPEATABLE READ` de l'AC1 est effectif ; **(a-défaut)** par `ReadSnapshot::begin(&pool)`.

⛔ **Le test doit avoir rougi avant la correction — et c'est le MÊME texte de test qui rougit puis passe au vert**
(C-28-3-16). Pour que ce soit possible, l'API du test existe dès T1 : T1 pose un `ReadSnapshot` **dégradé** — même
type, mêmes constructeurs `begin`/`begin_on`, même accès connexion, même `finish`, mais `begin`/`begin_on` ne font
qu'acquérir ou garder une `PoolConnection`, **sans** `SET TRANSACTION` ni `START TRANSACTION`, et `finish` ne fait
que rendre la connexion. Les quatre lectures, le cœur et l'observateur sont, dès T1, sur `&mut ReadSnapshot` ; les
lectures y sont donc en autocommit sur **une** connexion, et la suppression concurrente est vue. Résultat attendu à
T1, et à copier au Dev Agent Record : (a-défaut), (a-RC) et (b) rouges — assertion 2 ou 3, orphelins listés. T2 ne
change que l'**intérieur** de `ReadSnapshot` (`begin_on` ouvre l'instantané de l'AC1, `finish` fait le `ROLLBACK`,
le champ devient une `sqlx::Transaction<'static, MySql>`) : les trois tests passent au vert **sans qu'une ligne de
`mod tests` ne change** — le Dev Agent Record le prouve par `git diff <commit T1> <commit T2> -- ` restreint au
module de test (sortie vide sur ces trois tests).

**(b) L'instantané est pris à l'ouverture.** L'observateur, à l'étape `Ouvert`, supprime l'import de la même façon.
Attendu : la sauvegarde contient l'import et ses transactions. (Sans `WITH CONSISTENT SNAPSHOT`, la vue naîtrait à la
première lecture, **après** la suppression : M2.)

**(c) La transaction est en lecture seule.** Une écriture exécutée **dans** un `ReadSnapshot` (par exemple
`UPDATE companies SET name = name`) est refusée par le serveur ; le test vérifie le **numéro** d'erreur MariaDB
(`1792`, `ER_CANT_EXECUTE_IN_READ_ONLY_TRANSACTION`) et non un message. ⚠️ `DatabaseError::code()` rend le
**SQLSTATE** (`25006`), non le numéro (`sqlx-mysql-0.8.6/src/error.rs:15-17`) : le numéro se lit par
`err.as_database_error()` → `try_downcast_ref::<MySqlDatabaseError>()` → `number()` (même fichier, l. 24-26). Un
test qui comparerait `code()` à `"1792"` rougirait sur le code correct.

**(d) La connexion revient propre.** Le test monte un **pool d'une seule connexion** sur la base du test
(`MySqlPoolOptions::new().max_connections(1).connect_with((*pool.connect_options()).clone())`), si bien que la
connexion réacquise après la sauvegarde est **nécessairement** celle qui a porté l'instantané — assertion de montage :
`CONNECTION_ID()` identique avant et après. `sqlx::Transaction` ne rend pas sa connexion : `finish` et le `drop` la
remettent au pool, d'où ce montage. **(d1)** `begin_on`, une lecture (`read_min_required`), puis `finish` ;
**(d2)** `begin_on`, la même lecture, puis `drop` sans `finish` (chemin d'erreur). Dans les deux cas, la connexion réacquise n'est plus en transaction : `SELECT @@in_transaction`
rend `0`. Si le serveur refuse `@@in_transaction` (fait non vérifié, F7 de la validation P1), le test bascule sur
`SELECT COUNT(*) FROM information_schema.INNODB_TRX WHERE trx_mysql_thread_id = CONNECTION_ID()` (`0` attendu) —
**le test tranche**, et le Dev Agent Record écrit laquelle des deux formes a servi (C-28-3-22). ⚠️ Ce repli ne voit
que les transactions qu'InnoDB a enregistrées ; d'où la lecture que (d1) et (d2) font avant de rendre la connexion,
qui garde le repli discriminant sous M6.

**(e) Toutes les tables sauvegardées sont transactionnelles — et sinon la sauvegarde échoue** (C-28-3-17, garde de
moteur de l'AC4).
**(e1)** Sur la base du test, la garde rend `Ok(n)` avec `n = TABLES_TO_TRUNCATE.len() + 1` (assertion de montage :
aucune table oubliée par la garde).
**(e2)** Après `ALTER TABLE _kesh_version ENGINE=MyISAM` sur la base éphémère du test (`_kesh_version` ne porte
aucune clé étrangère, squash l. 23-31 — choisie pour cela : une table liée par clé étrangère ne changerait pas de
moteur ; acceptation de cet `ALTER` non vérifiée, tranchée par le test), `export_installation`
rend `Err(DbError::Invariant(m))` avec `m` contenant `_kesh_version` et `MyISAM`. Une table neuve créée en `MyISAM`,
`Aria` ou `MEMORY`, en test comme sur un serveur d'exploitation réglé autrement, ferait de même échouer la sauvegarde
au lieu de lui échapper **sans que rien ne le voie**.

**(f) Le niveau d'isolation de la session n'est pas modifié.** Même montage que (d) : sur la connexion passée en
`SET SESSION TRANSACTION ISOLATION LEVEL READ COMMITTED`, après une sauvegarde par `begin_on` puis `finish`, la connexion
réacquise rend à `SELECT @@tx_isolation` toujours `READ-COMMITTED` : le `SET TRANSACTION` de l'AC1 ne vaut que pour la
transaction de l'instantané. (MariaDB 10.11 : `@@tx_isolation` porte le niveau de **session** ; c'est lui qu'on
contrôle ici, non celui de la transaction en cours.)

**(g) Les deux gardes lexicales de l'AC2** — (i) `export_installation_n_ouvre_qu_un_instantane` (`backup.rs`,
`mod tests`, test ordinaire `#[test]`, lecture de `include_str!("backup.rs")`) et (ii)
`build_keshbackup_ne_lit_la_base_qu_une_fois` (`admin_backup/export.rs`, `mod tests` **neuf** — le fichier n'en a pas aujourd'hui, `grep -n "cfg(test)"` ne rend
rien —, `include_str!("export.rs")`),
sur le modèle de `avant_sauvegarde_branchee_aux_appels_de_l_import` (`routes/admin.rs:880`) et de son helper
`code_de_production` (`routes/admin.rs:936-943` : fichier tronqué à la première ligne `#[cfg(test)]`, lignes `//`
écartées — ce qui écarte aussi `///`). Corps de la fonction : de la ligne commençant par
`pub async fn export_installation(` (resp. `pub async fn build_keshbackup(`) — la parenthèse distingue
`export_installation_in` — jusqu'à la première ligne réduite à `}` en colonne 0.

### AC6 — Le chemin de `kesh-api` reste couvert, sans régression

Les suites existantes restent vertes **sans modification de leurs assertions** : `crates/kesh-api/tests/admin_full_export_e2e.rs`,
`admin_full_import_e2e.rs`, `admin_backup_e2e.rs`, et les tests de `routes/admin.rs` (dont la garde lexicale de
l'AC3). Les tests de `backup.rs` qui appellent `export_table(&pool, …)` (`backup.rs:884`, `:910`, `:1209`) et
`crates/kesh-db/tests/accounts_role_backfill.rs:237` passent par un `ReadSnapshot` (adaptation mécanique, assertions
inchangées).

### AC7 — Pré-import : l'instantané est pris sous le verrou, sans interblocage

Le comportement de `run_backup_and_restore` est inchangé dans son ordre : verrou `FOR UPDATE` sur `_kesh_version`
dans `tx` (`admin.rs:239`), **puis** `build_keshbackup(&state.pool)` sur une **autre** connexion du pool, **puis** la
restauration dans `tx`. La lecture de `_kesh_version` par l'instantané est une lecture **non verrouillante** : elle ne
bloque pas sur le `FOR UPDATE` tenu par `tx` (comme aujourd'hui). Le commentaire `admin.rs:254-262` (« `build_keshbackup`
lit via le pool (connexions séparées) ») est réécrit : une connexion séparée, un instantané pris après le verrou.
Seul changement de comportement, voulu : sur une base dont une table n'est pas InnoDB, la garde de moteur (AC4) fait
échouer la sauvegarde de sécurité, donc refuser l'import (`AdminPreImportBackupFailed`, chemin d'erreur existant) —
un import ne s'engage pas sans sauvegarde cohérente. Aucun
test neuf exigé ici : `admin_full_import_e2e.rs` exerce déjà ce chemin de bout en bout (un interblocage le ferait
rougir ou pendre).

### AC8 — Les commentaires qui décrivent le défaut sont mis à jour (propagation)

- `crates/kesh-db/src/repositories/bank_imports.rs:447-448` : la sauvegarde d'installation ne porte plus
  d'orphelins ; **reste** l'export global CSV, #465.
- `crates/kesh-api/src/admin_backup/export.rs` : en-tête (`:1-13` — et le « (22) » périmé de `:11`, à remplacer par
  « une par table de `TABLES_TO_TRUNCATE` », **sans nombre**) et doc-comment de `build_keshbackup`.
- `crates/kesh-db/src/backup.rs` : doc du module (`:1-17`) et de `export_table` ; et le commentaire `:487-488`, qui
  cite `export.rs:55` (déjà périmé : la boucle est à `export.rs:60`) et `:96` — à **dépouiller des numéros de
  ligne** (« la boucle de `export_installation` »), puisque la story déplace la boucle de crate.
- `crates/kesh-api/src/exports/global.rs:21-24` **reste** tel quel (#465 ouvert) ; ajouter seulement que la
  sauvegarde d'installation, elle, lit dans un instantané (`kesh_db::snapshot::ReadSnapshot`), réutilisable pour #465.
- Grep de propagation **avant** la passe suivante, sur des jetons discriminants (le grep large d'origine,
  `"633\|orphelin"`, rendait des dizaines de lignes sans rapport — `migrations.sha384`, `bootstrap.rs`…) :
  ```sh
  grep -rnE "#633|lit via le pool|connexions séparées|torn-read|Lectures successives|sans instantané" \
    crates docs CHANGELOG.md --include=*.rs --include=*.tex --include=*.md
  grep -rnE "export\.rs:[0-9]+" crates docs
  ```
  Au sol (`6f971d76`) le premier rend `exports/global.rs:21` (reste), `routes/admin.rs:257-258` (à réécrire, AC7) et
  `repositories/bank_imports.rs:448` (à réécrire) ; le second, `backup.rs:488` seul. Chaque ligne restante se
  justifie au Dev Agent Record.

### AC9 — La restauration n'est pas concernée, et la fiche le prouve

Aucune modification de `restore_tables_in_tx`, `force_onboarding_done_if_eligible` (`backup.rs:444`, `:626`),
`post_restore.rs`, `admin_backup/import.rs`. Motif, vérifié au sol : la restauration **écrit** dans **une** transaction
(`tx`, ouverte `admin.rs:236`, validée `admin.rs:489`) sous le verrou d'installation ; ses seules lectures hors `tx`
sont celles de `check_schema_compat` (`column_constraints`, `INFORMATION_SCHEMA`), qui portent sur le **schéma**, non
sur les données, et que rien ne modifie à l'exécution (migrations au démarrage, `main.rs:149`, avant l'écoute
`main.rs:382`). Le défaut de #633 est un défaut de **lecture** des données ; la restauration ne le partage pas.

### AC10 — Manuel d'administration (`docs/manual/fr/admin-manual.tex`, § *Exporter (sauvegarde complète)*, l. 1823 et suivantes)

La phrase « Cette opération est en lecture seule (aucun effet de bord sur les données). » (`:1825`, PDF identique)
est **fausse au code d'aujourd'hui** (C-28-3-18) : après la production du fichier, le handler `full_export` écrit une
entrée du journal d'audit, `admin.full_export` (`emit_full_export_audit`, `routes/admin.rs:44`, corps `:653-692`,
`audit_log::insert_in_tx` puis `commit`), à titre best-effort (son échec n'empêche pas le téléchargement). Elle est
**remplacée**, puis suivie du paragraphe neuf, en substance :

> L'export ne modifie aucune donnée comptable ni aucun réglage : il ajoute seulement, une fois le fichier produit,
> une entrée au journal d'audit (`admin.full_export`) — entrée que le fichier, pris avant elle, ne contient donc pas.
>
> Kesh lit toutes les tables dans **un seul instantané** de la base, pris au début de l'export : ce qui est saisi,
> modifié ou supprimé pendant qu'il se prépare n'y figure pas, et le fichier ne porte jamais une ligne sans son
> en-tête — une transaction sans son import, une ligne de facture sans sa facture. L'export ne bloque aucune saisie.
> La sauvegarde de sécurité prise avant un import (plus bas) est lue de la même façon. Si une table de la base n'est
> pas transactionnelle (moteur autre qu'InnoDB, par un réglage du serveur), l'export **échoue** plutôt que de produire
> un fichier qui ne serait pas cohérent. ⚠️ *Avant la 0.13.3*, un export pris pendant une saisie pouvait porter de
> telles lignes ; une sauvegarde de cette époque que l'on compte garder se **refait**.

Et, au même endroit ou au § *Quelle méthode de sauvegarde choisir ?* (`:1901`), une phrase : l'**export global**
(CSV, *Administration → Export global*) ne bénéficie **pas encore** de cette garantie (#465). Côté manuel utilisateur
(`docs/manual/fr/user-manual.tex`, § *Exports de souveraineté*, après « Lancer l'export », `:2593-2595`) : une phrase
honnête — l'export lit ses tables l'une après l'autre ; une saisie faite pendant qu'il se prépare peut n'y figurer
qu'en partie ; le lancer quand personne ne saisit (#465).

**PDF régénérés** (`make fr` dans `docs/manual/`, sous `flock`), et contrôlés **aplatis** :
`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "un seul instantané[^.]*"` (et de
même pour le manuel utilisateur), **et** l'absence de l'ancienne promesse :
`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -c "aucun effet de bord"` → `0` (de même
`grep -c "aucun effet de bord" docs/manual/fr/admin-manual.tex` → `0`). Aucune modification de version des manuels
(macro `\keshVersion` : gate de release, pas de story).

⚠️ **PDF et stories sœurs** (C-28-3-23) : 28-1 et 28-2 avancent en parallèle et peuvent toucher les mêmes manuels.
Au rebase, le `.tex` se fusionne par union ; les PDF (binaires) ne se fusionnent **jamais** : reprendre ceux de
`main`, puis les **régénérer** (`make fr`, sous `flock`) depuis le `.tex` fusionné, et refaire le contrôle aplati.

### AC11 — CHANGELOG

Sous `## [0.13.3] — Non publié` (à créer en tête des versions si absent — les stories sœurs 28-1 et 28-2 peuvent la
créer aussi : fusion par union au rebase), `### Corrigé` :

> - **La sauvegarde d'installation photographie la base à un seul instant ([#633](https://github.com/guycorbaz/kesh/issues/633)).**
>   Jusqu'ici, le fichier `.keshbackup` — l'export d'*Administration → Sauvegarde complète* comme la sauvegarde de
>   sécurité prise avant un import — lisait ses tables l'une après l'autre : une saisie ou une suppression faite
>   pendant la sauvegarde pouvait y laisser des lignes sans leur parent (par exemple des transactions sans leur
>   import), que la restauration recréait telles quelles. Toutes les tables sont désormais lues dans un instantané
>   unique, en lecture seule, qui ne bloque aucune saisie ; si une table n'est pas transactionnelle (moteur autre
>   qu'InnoDB), la sauvegarde échoue au lieu de produire un fichier incohérent. Une sauvegarde prise sous une version antérieure pendant
>   que l'on saisissait se refait. L'export global CSV n'en bénéficie pas encore ([#465](https://github.com/guycorbaz/kesh/issues/465)).

### AC12 — Aucune migration, aucun changement d'API ni d'écran

Aucun fichier sous `crates/kesh-db/migrations/` ni `test-schema/` ; `kesh_version_min_required` inchangé ;
`docs/migrations-idempotence-audit.md` et `post_restore.rs` (P5, P7) non concernés. Format `.keshbackup` inchangé
(un fichier produit par la 0.13.3 s'importe dans une 0.13.2, et réciproquement). Route, codes d'erreur, i18n,
frontend : inchangés.

## Tasks / Subtasks

- [ ] **T0 — Montage** (AC —) : bases `kesh_283` et `kesh_e2e_283` (migrations du worktree, seed
  `scripts/seed-dev-db.sql`), port `3036`, `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-28-3/target`, `node_modules`
  par `cp -al` depuis le dépôt principal. Gates lourds sous `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock`.
- [ ] **T1 — Le test rouge d'abord** (AC2 point 1, AC3, AC4, AC5 (a), (b)) : `crates/kesh-db/src/snapshot.rs` avec
  le `ReadSnapshot` **dégradé** (C-28-3-16 : `begin`, `begin_on`, accès connexion, `finish` ; aucune transaction),
  `pub mod snapshot;` dans `lib.rs` ; les quatre lectures de `backup.rs` sur `&mut ReadSnapshot` ;
  `InstallationExport`, `export_installation` et son cœur observable (sans pool, AC4) ; `build_keshbackup` sur
  `export_installation` et adaptation des quatre tests appelant `export_table` — pour que le **workspace** compile à
  ce commit ; helper `orphelins` ; tests (a-défaut), (a-RC), (b). Exécuter, constater le **rouge** des trois, copier
  la sortie au Dev Agent Record. Commit (`test(28-3): … rouge avant la correction (refs #633)`).
- [ ] **T2 — L'instantané** (AC1, AC2 point 3 (i), AC4 garde de moteur) : l'intérieur de `ReadSnapshot` seul
  (`SET TRANSACTION` en protocole texte, `close_on_drop` sur son échec, `Transaction::begin`, `ROLLBACK` de `finish`,
  commentaire de dépendance `#[doc(hidden)]` — C-28-3-19) ; garde de moteur dans le cœur. Les tests (a-défaut),
  (a-RC), (b) passent **sans modification** (`git diff` du module de test copié, AC5 (a)). Tests (c), (d1), (d2), (e1),
  (e2), (f), (g)(i).
- [ ] **T3 — `kesh-api`** (AC2 point 3 (ii), AC3, AC6, AC7) : garde (g)(ii) dans un `mod tests` neuf de
  `admin_backup/export.rs` ; commentaire `admin.rs:254-262` ; suites de l'AC6 vertes.
- [ ] **T4 — Propagation** (AC8) : commentaires, grep de propagation copié au Dev Agent Record.
- [ ] **T5 — Documentation** (AC10, AC11) : manuels, PDF régénérés et contrôlés aplatis, CHANGELOG.
- [ ] **T6 — Mutations et gates** (AC1-AC12) : campagne M1-M12 (ci-dessous) ; base remise à zéro **avant** le gate
  complet ; gate complet `scripts/test-fast.sh` (**`kesh-db` touché : gate complet même en boucle de revue**) ;
  `npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build` ; **E2E complet au dernier
  commit de code** (aucun écran ne change, mais la règle D7 ne connaît pas d'exception).

### Mutations à exécuter (T6)

Chaque mutation : motif vérifié unique, appliquée, test ciblé rejoué, restauration `git checkout -- <fichier>` +
`touch` (cargo garde sinon le binaire muté). Résultat attendu écrit ; une mutation qui survit est un défaut du test
ou une équivalence **à démontrer**, jamais à déclarer.

| # | Mutation | Test qui doit rougir |
|---|---|---|
| M1 | le cœur lit chaque table par une connexion neuve du pool (retour au code d'avant, par un `ReadSnapshot::begin(pool)` **par table**) | AC5 (a-défaut), (a-RC) |
| M2 | `START TRANSACTION WITH CONSISTENT SNAPSHOT, READ ONLY` → `START TRANSACTION READ ONLY` | AC5 (b) |
| M3 | suppression du `SET TRANSACTION ISOLATION LEVEL REPEATABLE READ` | AC5 (a-RC) |
| M4 | `SET TRANSACTION …` → `SET SESSION TRANSACTION …` | AC5 (f) |
| M5 | suppression de `, READ ONLY` | AC5 (c) |
| M6 | `ReadSnapshot` ouvre la transaction par un `execute("START TRANSACTION …")` brut sur la `PoolConnection`, hors de `sqlx::Transaction` (le piège de l'AC1) ; `finish` se contente de rendre la connexion | AC5 (d1), (d2) — `@@in_transaction` à `1`. ⚠️ Un `finish` qui oublie le `ROLLBACK` **mais** garde la `sqlx::Transaction` est, lui, **équivalent** (le `Drop` de sqlx l'émet) : à consigner comme tel, non comme survivant |
| M7 | l'observateur n'est plus appelé après chaque table | AC5 (a) — assertion de montage (compteur) |
| M8 | le helper `orphelins` ne lit plus que les FK d'une table (filtre `TABLE_NAME = 'invoices'`) | AC5 (a) — assertion de montage du helper |
| M9 | une table de `TABLES_TO_TRUNCATE` exclue de la garde de moteur (`skip(1)` sur la liste contrôlée) | AC5 (e1) — compte rendu par la garde |
| M10 | `build_keshbackup` relit `min_required` dans un **second** instantané (`kesh_db::snapshot::ReadSnapshot::begin(pool)` puis `read_min_required`) — **compile** (C-28-3-15) | AC5 (g)(ii) — `pool` trois fois. Aucun test fonctionnel ne la tue (`min_required` ne change qu'au démarrage) : c'est la raison d'être de la garde |
| M11 | `export_installation` ouvre un second `ReadSnapshot::begin(pool)` pour l'une des lectures — compile | AC5 (g)(i) — `pool` trois fois |
| M12 | la garde de moteur rend `Ok` sans regarder `ENGINE` | AC5 (e2) |

## Dev Notes

### Inventaire clos des lectures de la sauvegarde, au sol (2026-10-13, `6f971d76`)

Commandes, et sorties copiées :

```text
$ grep -n "fetch_\|execute(" crates/kesh-db/src/backup.rs | sed -n 1,5p
157:    .fetch_all(pool)          ← non_generated_columns (INFORMATION_SCHEMA.COLUMNS)
201:        .fetch_all(pool)      ← export_table (SELECT des lignes)
311:    .fetch_one(pool)          ← read_min_required (_kesh_version)
323:        .fetch_one(pool)      ← read_instance_id (MIN(id) FROM companies)
399:    .fetch_all(pool)          ← column_constraints — IMPORT, hors sauvegarde
```

(les lignes suivantes, `449` et au-delà, sont sur `&mut **tx` : restauration.)

```text
$ grep -rn "build_keshbackup\|export_table(\|read_min_required\|read_instance_id" crates --include=*.rs \
    | grep -v "^crates/kesh-db/src/backup.rs:.*fn "
crates/kesh-api/src/admin_backup/export.rs:46:    let min_required = read_min_required(pool)…
crates/kesh-api/src/admin_backup/export.rs:47:    let instance_id = read_instance_id(pool)…
crates/kesh-api/src/admin_backup/export.rs:61:            let export = export_table(pool, table)…
crates/kesh-api/src/routes/admin.rs:39:    let (bytes, meta) = build_keshbackup(&state.pool).await?;
crates/kesh-api/src/routes/admin.rs:263:    let (backup_bytes, _meta) = build_keshbackup(&state.pool)
crates/kesh-db/src/backup.rs:884:        let export = export_table(&pool, "reconciliation_rules")      (test)
crates/kesh-db/src/backup.rs:910:        let export = export_table(&pool, "companies")                 (test)
crates/kesh-db/src/backup.rs:1209:        let export = export_table(&pool, "companies")…                (test)
crates/kesh-db/tests/accounts_role_backfill.rs:237:    let export = kesh_db::backup::export_table(&pool, "accounts")  (test)
```

(plus des mentions en commentaire : `admin_backup/mod.rs:10`, `export.rs:3`, `routes/admin.rs:41`, `:129`, `:258`,
`:498`, `:873-914` — garde lexicale.)

**Ensemble clos** : quatre sites SQL de lecture de données ou de schéma pour la sauvegarde, trois appels depuis
`kesh-api`, deux appelants de `build_keshbackup`, quatre tests. Après T1, les quatre fonctions exigent
`&mut ReadSnapshot` : aucune d'elles ne peut plus s'appeler sur le pool. Ce n'est **pas** une interdiction de tout
site neuf (AC2, C-28-3-15) : le cœur sans pool et les deux gardes lexicales (AC5 (g)) couvrent les deux fonctions qui
tiennent un pool. La story **ajoute** une lecture, celle de la garde de moteur (`information_schema.TABLES`, AC4), dans
l'instantané. **Sites non résolus après la story** : aucun pour la sauvegarde **dans les formes couvertes** ; angle
mort écrit à l'AC2 (une lecture faite par un appelant et passée en paramètre) ; l'export global (`exports/global.rs`,
29 lectures sur le pool) reste hors périmètre, nommé (#465, C-28-3-1).

`grep -rn "kesh_db::backup::" crates` : les autres usages (`kesh-seed`, `admin_backup/import.rs`, tests e2e) ne
touchent que `TABLES_TO_TRUNCATE`, `reset_cleared_tables`, `TableRestore`, `column_constraints`,
`parse_ndjson_rows`, `restore_tables_in_tx`, `force_onboarding_done_if_eligible` — aucune lecture de sauvegarde.

### Moteurs de stockage, au sol

```text
$ S=crates/kesh-db/test-schema/0001_schema_squash.sql
$ grep -ci 'CREATE TABLE' $S ; grep -ci 'ENGINE=InnoDB' $S
40
40
```

Les 40 tables du squash (régénéré depuis `SHOW CREATE TABLE`, donc fidèle au moteur réel) : les 39 de
`TABLES_TO_TRUNCATE` (recompté : `sed -n '/^pub const TABLES_TO_TRUNCATE/,/^];/p' crates/kesh-db/src/backup.rs | grep -c '^    "'`
→ 39) + `_kesh_version`. Toutes InnoDB. Les migrations portent 51 `CREATE TABLE` pour 38 `ENGINE=InnoDB` (tables
recréées, ou créées sans clause et donc au moteur par défaut **du serveur**) : le squash fait foi pour la base de
test — `test_schema_guard.rs` compare d'ailleurs le moteur du squash à celui du vrai schéma (`schema_facts`, l. 155-165)
—, mais **pas** pour une base servie dont le serveur aurait un autre `default_storage_engine` (aucun fichier du dépôt
ne le fixe : `grep -rniE "default.storage.engine"` sans résultat). D'où la garde **à l'exécution** de l'AC4
(C-28-3-17), qui contrôle la base réellement sauvegardée. `_sqlx_migrations` n'est pas lue par la sauvegarde.

### DDL concurrente, durée, purge — ce que l'instantané change et ce qu'il ne change pas

- **DDL** : Kesh n'en exécute qu'au démarrage (`MIGRATOR.run`, `main.rs:149`), avant d'écouter (`main.rs:382`) ; les
  `TRUNCATE` de `test_fixtures.rs:401` sont de test ; la restauration et la remise à zéro procèdent par `DELETE`
  (`backup.rs:496`, `kesh-seed/src/lib.rs:586`). Une DDL concurrente ne peut donc venir que d'un second processus
  (un autre conteneur qui démarre et migre sur la même base). Dans ce cas : une table **déjà lue** est protégée par
  le verrou de métadonnées que la transaction garde jusqu'à sa fin — la DDL **attend** la fin de la sauvegarde ; une
  table **pas encore lue** et reconstruite entre-temps fait échouer sa lecture (`ER_TABLE_DEF_CHANGED`, 1412) : la
  sauvegarde échoue, **fermée** (`AdminFullExportFailed`), jamais incohérente. Angle mort assumé, non testé
  (C-28-3-8).
- **Durée** : la transaction couvre 81 requêtes (1 + 2 + 39 × 2 : garde de moteur, `min_required` et
  `instance_id`, puis colonnes et lignes de chaque table), sans compression (C-28-3-5) ; quelques secondes au plus
  sur une installation de PME. Elle **ne pose aucun verrou de ligne** (lectures cohérentes non verrouillantes) :
  aucune saisie n'attend la sauvegarde.
- **Purge** : la vue retient la purge InnoDB des versions plus récentes qu'elle pendant sa durée — même effet qu'un
  `mariadb-dump --single-transaction` (celui de `scripts/synology/kesh-dump.sh:8`), négligeable à cette échelle.
- **Pool** : une connexion tenue pendant la sauvegarde (aujourd'hui : une connexion à la fois, prise et rendue par
  requête). À l'import, deux connexions (`tx` + instantané) sur les 5 du pool (`main.rs:75`) — comme aujourd'hui en pic.
- **Streaming** : `full_export` streame **après** `build_keshbackup` (`admin.rs:39-60`) ; l'instantané est clos
  avant toute écriture de réponse.
- **Mémoire** (C-28-3-5) — angle mort écrit : le pic passe à « toutes les tables non compressées + le ZIP ». Le
  plafond `KESH_ADMIN_EXPORT_INMEM_MB` (manuel d'administration `:1897`, « plafond d'assemblage en mémoire de
  l'export ») ne borne que la **livraison** du ZIP déjà assemblé (`admin.rs:53-54`), ni ce pic ni celui
  d'aujourd'hui ; `audit_log` est le candidat au gros volume. La story ne change pas le manuel sur ce point (son
  énoncé n'est pas rendu plus faux) ; à reprendre si une installation réelle rendait le pic gênant.

### Risque de montée de sqlx — `Transaction::begin` est `#[doc(hidden)]`

`sqlx-core-0.8.6/src/transaction.rs:98-99` : `#[doc(hidden)] pub fn begin(...)`. Exportée, mais hors contrat semver.
Les alternatives documentées ne conviennent pas : `Connection::begin_with` (`connection.rs:59`) **emprunte** la
connexion et ne rend pas de type `'static` ; `Pool::begin_with` (`pool/mod.rs:391-399`) acquiert une connexion
**neuve**, sur laquelle le `SET TRANSACTION` n'a pas été posé. `snapshot.rs` épingle la dépendance en commentaire
(« dépend de `sqlx::Transaction::begin` (`doc(hidden)`), sqlx 0.8.6 ; contrôlé par AC5 (d) à chaque montée de
sqlx ») ; à une montée de sqlx, AC5 (a-RC), (d) et (f) sont les tests qui diraient qu'elle a changé (C-28-3-19).

### Gardes existantes — ce qu'elles demandent, ou non

- `crates/kesh-db/tests/test_schema_guard.rs` : `every_sqlx_test_attribute_is_accounted_for` accepte tout attribut
  portant `"./test-schema"` (`SQUASH_SPELLINGS`, l. 64) : les tests neufs de `backup.rs` n'ont **rien** à inscrire à
  `ALLOWED_REAL_MIGRATOR_FILES`.
- `crates/kesh-api/tests/textes_coherents.rs` lit les manuels (`manuels_fr()`) : aucune de ses gardes ne vise le
  sujet, mais les textes de l'AC10 y passent — le gate complet les exerce.
- `CHANGELOG.md` : `grep -rln "CHANGELOG.md" crates --include=*.rs` ne rend **aucun** fichier ; seul
  `crates/kesh-db/tests/prepare_release_script.rs` traite d'un CHANGELOG (« datation du CHANGELOG », l. 7), celui que
  le script de release manipule. La section `## [0.13.3] — Non publié` ne devrait gêner aucun test — **non vérifié
  par exécution**, tranché par le gate complet.
- `routes/admin.rs:880` (garde lexicale des appels de `build_keshbackup`) : compte inchangé (AC3).

### État actuel des fichiers modifiés — ce qui doit être préservé

- `admin_backup/export.rs` : ordre des entrées ZIP (`data/<table>.ndjson` dans l'ordre de `TABLES_TO_TRUNCATE`, puis
  `files/`, puis `manifest.json`), options de compression (`Deflated`, `0o644`), `KeshBackupMeta { byte_size,
  table_count = TABLES_TO_TRUNCATE.len(), total_rows }`, messages d'erreur `zip …`. Aucune émission d'audit dans
  `build_keshbackup` (elle reste au handler, `admin.rs:44`).
- `backup.rs` : format NDJSON (ordre des clés, `decode_cell`, `%Y-%m-%dT%H:%M:%S%.f`), garde du back-quote, erreur
  `Invariant` si aucune colonne non générée — inchangés ; seuls changent le type du premier paramètre et l'exécuteur.
- `routes/admin.rs` : la garde lexicale (`:870-914`) compte **un** `check_schema_compat(` et **deux**
  `build_keshbackup(` dans le code de production hors commentaires `//` : ne pas en ajouter, ne pas en retirer.

### Pièges connus du dépôt, applicables ici

- **Test muet** : l'assertion de montage de l'AC5 (a) (le geste concurrent a eu lieu) et celle du helper (il voit la
  FK) sont obligatoires — précédent `backfill_skips_archived_accounts` (16-1a).
- **Mutation restaurée, binaire périmé** : `touch` après chaque restauration.
- **Base de gate piégée** : remise à zéro inconditionnelle avant le gate complet (§ CLAUDE.md, KF-039 #310).
- **`kesh-db` touché** : gate complet même en cours de boucle de revue.
- **Édition du code par l'outil Edit uniquement** (consigne 14) ; pas de `sed -i`.
- **sqlx et `START TRANSACTION` brut** : seul `Transaction::begin(conn, Some(stmt))` tient le compte de profondeur de
  transaction ; voir AC1.
- **`SET TRANSACTION` et les valeurs de session** : sous MariaDB, `@@tx_isolation` reflète le niveau de **session** ;
  le niveau « pour la transaction suivante » posé par `SET TRANSACTION` ne s'y lit pas forcément — ne pas fonder sur
  cette variable la preuve que l'instantané est en `REPEATABLE READ` : cette preuve est le comportement (AC5 (a-RC)) ;
  la variable ne sert qu'à l'AC5 (f) (la session n'a pas changé).

### Faits MariaDB tirés de la documentation, non exécutés à la création de la fiche

Écrits d'après la documentation de MariaDB, **non exécutés** ici (la fiche n'a lancé aucune requête) ; chacun est
établi, ou réfuté, par le test qui le porte — le dev le note au Dev Agent Record :

| Fait | Test qui l'établit |
|---|---|
| `START TRANSACTION WITH CONSISTENT SNAPSHOT, READ ONLY` est accepté par 10.11 | tous ceux de l'AC5 |
| sous `READ COMMITTED`, `WITH CONSISTENT SNAPSHOT` ne fige pas la vue | AC5 (a-RC) **sous M3** |
| une écriture en `READ ONLY` est refusée en `1792` | AC5 (c) |
| `@@in_transaction` est une variable de session lisible (sinon repli `INNODB_TRX`, C-28-3-22) | AC5 (d) |
| `SET TRANSACTION ISOLATION LEVEL REPEATABLE READ` sans portée l'emporte sur une session `READ COMMITTED` au moment où `START TRANSACTION WITH CONSISTENT SNAPSHOT` assigne la vue | AC5 (a-RC) — rouge **sur le code correct** s'il était faux |
| `START TRANSACTION … READ ONLY` laisse `SERVER_STATUS_IN_TRANS` posé (sinon sqlx rend `BeginFailed`) | tous ceux de l'AC5 (échec à l'ouverture) |
| `information_schema.TABLES` se lit dans une transaction `READ ONLY` | AC5 (e1), et tous ceux de l'AC5 (garde de moteur en tête du cœur) |
| `ALTER TABLE _kesh_version ENGINE=MyISAM` est accepté (table sans clé étrangère, `CHECK` compris) | AC5 (e2) |
| `@@tx_isolation` reflète le niveau de session, et `SET TRANSACTION` sans `SESSION` ne le change pas | AC5 (f) **sous M4** |
| une `DELETE` concurrente n'attend pas la transaction de lecture (lectures non verrouillantes, verrou de métadonnées partagé compatible) | AC5 (a) — il pendrait sinon |

Si l'un d'eux est faux, la fiche est fausse à cet endroit : corriger la fiche et le registre, non le test.

### Découpage préventif — non requis

Modules touchés : `kesh-db` (`snapshot.rs` neuf, `backup.rs`, `lib.rs`, `repositories/bank_imports.rs` — commentaire,
`tests/accounts_role_backfill.rs`), `kesh-api` (`admin_backup/export.rs`, `routes/admin.rs` — commentaire,
`exports/global.rs` — commentaire), documentation. Moins de cinq modules métier de premier niveau. Inclure #465 aurait
porté le périmètre à **24** modules de dépôt (C-28-3-1) : c'est le motif du découpage, appliqué d'office.

### Project Structure Notes

- `crates/kesh-db/src/snapshot.rs` : module neuf, documenté (`//!` + `///`), réutilisable par l'export global (#465).
- Aucun fichier neuf dans `kesh-api`.

### References

- Issue #633, issue #465 (`gh issue view --json`).
- `crates/kesh-api/src/admin_backup/export.rs:45-110` ; `crates/kesh-db/src/backup.rs:34-90, 145-325` ;
  `crates/kesh-api/src/routes/admin.rs:32-60, 230-300, 489, 870-914` ; `crates/kesh-db/src/repositories/bank_imports.rs:426-449`.
- sqlx 0.8.6 : `sqlx-core-0.8.6/src/transaction.rs:99` (`Transaction::begin`), `sqlx-mysql-0.8.6/src/transaction.rs:19-40`.
- MariaDB 10.11 (`docker-compose.yml:4`, `.github/workflows/ci.yml:50`) : `START TRANSACTION [WITH CONSISTENT SNAPSHOT
  | READ ONLY | READ WRITE] [, …]` ; `SET TRANSACTION ISOLATION LEVEL` sans `SESSION`/`GLOBAL` = transaction suivante.
- Précédent d'instantané de lecture dans le dépôt : `routes/journal_entries.rs:474-480` (`pool.begin()` +
  `rollback`), sans `READ ONLY` ni isolation forcée.
- Registre : `epic-28-choix-autonomes-28-3.md`.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-10-13 — Fiche créée (`bmad-create-story`, Opus 5.5, autonomie) : 12 AC, 7 tâches (T0-T6), 10 mutations
  (M1-M10), choix C-28-3-1 à C-28-3-14. #465 séparé (C-28-3-1).
- 2026-10-13 — **Validation P1** (Sonnet, lentilles R et F, contexte frais, lecture seule ; prompt
  `28-3-validate-prompt-p1.md`, rapports `kesh-gate-logs/28-3-validate-p1-{R,F}.md`) : **0 CRITICAL, 0 HIGH,
  3 MEDIUM, 16 LOW** (19 bruts — F : 1 MEDIUM, 6 LOW ; R : 2 MEDIUM, 10 LOW —, 15 distincts après les convergences
  F1≈R6, F2=R11, F5=R8, F6≈R1). Remédiation (Opus 5.5) :
  - **F1/R6 (MEDIUM)** — AC2 réécrit sur la garantie réelle (le type tient les quatre lectures existantes, pas les
    lectures neuves) ; cœur sans pool ; deux gardes lexicales neuves, AC5 (g)(i) et (ii) ; M10 requalifiée
    (compile, tuée par (g)(ii)), M11 neuve (tuée par (g)(i)) ; inventaire clos reformulé. C-28-3-15.
  - **R1 (MEDIUM)** — `ReadSnapshot` dégradé dès T1 : (a-défaut), (a-RC) et (b) rougissent puis passent au vert sans
    changer une ligne de `mod tests`, preuve par `git diff` ; T1 rend aussi le workspace compilable (adaptation de
    `kesh-api` avancée de T3 à T1). C-28-3-16 ; F6 (ii) résolu du même geste.
  - **R2 (MEDIUM)** — AC10 : la phrase « aucun effet de bord » du manuel d'administration (`:1825`) est remplacée
    (l'export écrit `admin.full_export` au journal d'audit), contrôle aplati du PDF ajouté. C-28-3-18.
  - LOW : F2/R11 garde de moteur **à l'exécution**, échec fermé, (e) refondu en (e1)/(e2), M9 redéfinie, M12 neuve
    (C-28-3-17) ; F3 `number()` après downcast ; F4 `close_on_drop` sur échec du `SET`, résidu du `START` assumé ;
    F5/R8 dépendance `#[doc(hidden)]` épinglée, `SET TRANSACTION` en protocole texte (C-28-3-19) ; F6 (i) forme
    `BoxFuture` prescrite, alias local sans dépendance neuve (C-28-3-21) ; F7 repli `INNODB_TRX`, le test tranche
    (C-28-3-22) ; R3 « octet pour octet » corrigé (`exportDate`) ; R4 « 41 lectures (80 requêtes) », et 81 après la
    garde de moteur ; R5 gardes existantes nommées (chemin réel de `prepare_release_script.rs` : `kesh-db/tests`) ;
    R7 grep de propagation discriminant, site `backup.rs:488` ajouté ; R9 PDF au rebase (C-28-3-23) ; R10 mémoire en
    angle mort ; R12 erreur de `finish` propagée (C-28-3-20). Décomptes après remédiation : 12 AC, 7 tâches (T0-T6),
    12 mutations (M1-M12), choix C-28-3-1 à 24.
