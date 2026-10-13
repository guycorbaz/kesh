# Story 28.3 : La sauvegarde d'installation lit ses tables dans un instantané commun

## Status

**done** — revue de code close le 2026-10-13 (P1, Sonnet, lentilles B, E, A : 0 MEDIUM+, 13 LOW bruts, 9 distincts,
appliqués sans toucher la production, C-28-3-40 à 43) ; gates complets verts sur `629a3b1e`, postérieur au dernier commit de code `6940b560`
(backend 3526/3526, Vitest 1395/1395, E2E 267 passés + 7 KF-029). Développement fait le 2026-10-13
(`bmad-dev-story`, Opus 5.5). Validation close le 2026-10-13 (P3 : 0 MEDIUM+, LOW appliqués).
Fiche créée le 2026-10-13 en autonomie (`bmad-create-story`, Opus 5.5), sur `6f971d76`
(rétrospective de l'Epic 27), branche `story/28-3`, worktree `/home/gcorbaz/devel/kesh-28-3`, port **3036**,
bases `kesh_283` / `kesh_e2e_283`. Choix consignés au registre de l'epic `epic-28-choix-autonomes.md`
(**C-28-3-1 à C-28-3-43** ; le registre séparé `epic-28-choix-autonomes-28-3.md` y a été versé par union au rebase
sur le kickoff, puis supprimé). 12 AC (AC1 à AC12).
Validation P1 faite (Sonnet, lentilles R et F : 3 MEDIUM, 16 LOW), remédiée (C-28-3-15 à 23) ; validation P2 faite
(Opus, lentilles R et F : 5 MEDIUM, 14 LOW), remédiée par simplification (C-28-3-25 à 31) ; validation P3 faite
(Sonnet, lentilles R et F, `28-3-validate-prompt-p3.md`, C-28-3-32 : 0 MEDIUM, 15 LOW bruts, 11 distincts), LOW appliqués — boucle close.
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
du kickoff, lu dans le worktree `kesh-28-0` à la création de cette fiche, mergé depuis — `5aad8b35`). Cible : **v0.13.3**.

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
  erreur survenue **une fois l'instantané ouvert**, la connexion revient au pool **hors transaction** (le `Drop` de
  `sqlx::Transaction` émet le `ROLLBACK`). Les erreurs **de l'ouverture** elle-même ont leurs résidus, au dernier
  point de cette liste.
  Une erreur du `ROLLBACK` final (`finish`) fait **échouer** la sauvegarde (C-28-3-20) : `export_installation` la
  propage, même si toutes les tables ont été lues. Que `finish` **rende** l'erreur est établi par l'AC5 (h) (connexion
  tuée par `KILL`) ; que `export_installation` la **propage** (un `?` sur `finish`) n'est exercé par aucun test
  d'**exécution** : l'observateur du cœur y est vide, et le rendre injectable demanderait de changer la production.
  Depuis la revue de code P1 (E1, L1, A-5 ; C-28-3-41), c'est la garde lexicale (g)(i) qui le tient — elle exige la
  sous-chaîne `snap.finish().await?` dans le corps d'`export_installation` et tue la mutation
  `let _ = snap.finish().await;` (P1-MB). Reste hors de sa vue une propagation écrite autrement (un `match` qui rend
  l'erreur) : elle ferait rougir la garde, jamais la taire.
- **Plan B, si l'AC5 (a-RC) rougit sur le code correct** (le `SET TRANSACTION` sans portée ne serait pas pris en compte
  par le `START … WITH CONSISTENT SNAPSHOT` qui le suit — fait non vérifié) : le dev ne corrige pas le test ; il
  remplace, dans `begin_on`, le `SET TRANSACTION` par la lecture du niveau de session (`@@tx_isolation`), un
  `SET SESSION TRANSACTION ISOLATION LEVEL REPEATABLE READ`, puis, **après** le `START`, le rétablissement du niveau lu
  (protocole texte, `close_on_drop` sur tout échec de ces instructions) ; l'AC5 (f) reste l'oracle de la session
  inchangée. Décision écrite au registre (C-28-3-33 ou suivant) et à la fiche, sans question (validation P3, F3-8).
- Ouverture, sur une `PoolConnection` acquise : d'abord le `SET TRANSACTION …` par `conn.execute("SET TRANSACTION …")`
  — un `&str` passé à `Executor::execute`, donc en **protocole texte**, comme le `BEGIN` de sqlx ; **pas**
  `sqlx::query(…)`, qui passerait en protocole préparé (C-28-3-19) ; puis `sqlx::Transaction::begin(conn,
  Some("START TRANSACTION WITH CONSISTENT SNAPSHOT, READ ONLY".into()))` sur cette même connexion — le paramètre est un
  `Option<Cow<'static, str>>` (`sqlx-core-0.8.6/src/transaction.rs:101`), d'où le `.into()` (C-28-3-30).
  `Transaction::begin` (`sqlx-core-0.8.6/src/transaction.rs:99`) est
  exportée mais marquée `#[doc(hidden)]` (l. 98) : hors contrat semver, c'est une dépendance **signalée en
  commentaire** — non épinglée : `kesh-db/Cargo.toml:16` porte `sqlx = { version = "0.8", … }`, et seul `Cargo.lock` tient la 0.8.6
  (commentaire de `snapshot.rs`, et Dev Notes § *Risque de montée de sqlx*, C-28-3-30) ; c'est pourtant la seule qui rende un type `'static`
  après un `SET` sur la même connexion (C-28-3-19). Elle contrôle `in_transaction()` après l'instruction
  (`sqlx-mysql-0.8.6/src/transaction.rs:33-34`). ⛔ **Jamais** un `START TRANSACTION` brut par `execute` hors de cette
  API : sqlx ne saurait pas la connexion en transaction et la rendrait au pool **transaction ouverte**.
- Échec du `SET TRANSACTION` : `conn.close_on_drop()` avant de rendre l'erreur — la connexion ne retourne pas au pool
  avec un état incertain (établi par l'AC5 (j), ajouté en revue de code P1 : C-28-3-40). Échec de `Transaction::begin` (la connexion lui a été cédée) : le niveau « pour la prochaine
  transaction » peut rester posé sur une connexion rendue au pool ; résidu **assumé**, écrit au registre (C-28-3-19).
  Cas particulier, sans remède possible puisque la connexion a été cédée : `Error::BeginFailed`
  (`sqlx-mysql-0.8.6/src/transaction.rs:33-34`) — le serveur a exécuté le `START`, mais le drapeau
  `SERVER_STATUS_IN_TRANS` manque ; la connexion repart au pool avec `transaction_depth = 0` et peut-être une
  transaction ouverte côté serveur. Résidu **assumé** lui aussi (C-28-3-30) ; s'il se produisait, tous les tests de
  l'AC5 rougiraient (fait de la Dev Notes § *Faits MariaDB*, ligne `SERVER_STATUS_IN_TRANS`).

### AC2 — Le type, la structure et deux gardes lexicales tiennent l'inventaire des lectures

Un type neuf, `ReadSnapshot` (module neuf `crates/kesh-db/src/snapshot.rs`, C-28-3-3), enveloppe la transaction de
l'AC1. Il n'a que deux constructeurs — `ReadSnapshot::begin(pool: &MySqlPool)` (public) et
`ReadSnapshot::begin_on(conn: PoolConnection<MySql>)` (`pub(crate)`, pour l'AC5 (a)) — `begin` n'étant que
`pool.acquire()` suivi de `begin_on(conn)`, si bien que `begin_on` sert aussi en production et n'est pas du code mort
sous `clippy -D warnings` (C-28-3-29) — et une fin explicite
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
   Depuis la revue de code P1, chacune exige aussi une sous-chaîne : (i) `snap.finish().await?` (E1, C-28-3-41 —
   P1-MB) ; (ii) `export_installation(pool)` (L3, C-28-3-41 — une autre fonction qui prendrait le pool laissait le
   compte à deux : P1-ME).

Ce qui reste hors des gardes, et c'est écrit comme angle mort : une donnée que lirait un **appelant**
(`full_export`, `run_backup_and_restore`) sur le pool et qu'il passerait à `build_keshbackup` par un paramètre neuf
— le compte de (ii) ne bougerait pas ; et un **alias** du pool (`let p = pool;` puis `export_installation(p)` et
`ReadSnapshot::begin(p)`), qui ramène M10 ou M11 à exactement deux `pool` (C-28-3-31). Le détecteur restant, pour
l'un comme pour l'autre, est la revue. *(Revue de code P1 : la sous-chaîne `export_installation(pool)` qu'exige
désormais (ii) ferme, par raisonnement et non par mutation, l'alias dans `build_keshbackup` ; il reste ouvert pour (i).)* `column_constraints` (`backup.rs:388`, import) **ne change pas** : il sert
la restauration, non la sauvegarde (AC9).

### AC3 — Une seule fonction de `kesh-db` lit l'installation ; `kesh-api` n'assemble plus que le ZIP

`kesh_db::backup::export_installation(pool: &MySqlPool) -> Result<InstallationExport, DbError>` ouvre le
`ReadSnapshot`, le passe au cœur (AC4), qui lit `min_required`, `instance_id` et chaque table **dans l'ordre de
`TABLES_TO_TRUNCATE`**, puis clôt elle-même l'instantané (`finish`, dont l'erreur se propage : C-28-3-20), et rend :

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

`export_installation` délègue à une fonction **privée** du module, prescrite (C-28-3-29) :
`export_installation_in(snap: &mut ReadSnapshot, obs: &mut (dyn FnMut(Etape) -> BoxFuture<'static, ()> + Send))
-> Result<InstallationExport, DbError>`. Le cœur **emprunte** l'instantané et ne le clôt pas : `finish` revient à
l'appelant — `export_installation` en production, le test sinon (AC5). Les tests (a), (b) appellent donc le cœur
privé avec un `ReadSnapshot` de `begin(&pool)` ou de `begin_on(conn)`, puis font `finish` ; le chemin
`export_installation` → cœur est couvert par l'AC5 (e) et par les suites de l'AC6. Le cœur reçoit un **observateur**
appelé :

- une fois **après l'ouverture de l'instantané et avant la première lecture** (étape `Ouvert`) ;
- une fois **après la lecture de chaque table** (étape `ApresTable(&'static str)`).

L'observateur a **cette** forme, prescrite (C-28-3-21) : `&mut (dyn FnMut(Etape) -> BoxFuture<'static, ()> + Send)`,
où `BoxFuture` est un alias **local** du module, à **deux** paramètres (C-28-3-30) :
`type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;` (std seule :
`kesh-db` ne dépend pas du crate `futures`, et la story n'en ajoute pas — `grep -n futures crates/kesh-db/Cargo.toml`
ne rend rien) ; `Etape` est `Copy`. Le champ de `Etape::ApresTable` n'est lu que sous `cfg(test)` (la production
passe un observateur vide) : il porte `#[cfg_attr(not(test), allow(dead_code))]` avec sa raison en commentaire, faute
de quoi `clippy -D warnings` rougirait sur la cible lib (C-28-3-29). `AsyncFnMut` est écarté : dans un cœur appelé depuis un handler
axum, dont le futur doit être `Send`, l'inférence d'auto-trait y est fragile. Le test capture dans la fermeture un
clone du pool et un compteur partagé (`Arc<AtomicUsize>`). `export_installation` lui passe un observateur **vide**
(`&mut |_| Box::pin(async {})`, littéral dont le type se déduit de l'attendu). Un observateur **factorisé** en
variable (tests (a), (b)) porte l'annotation de retour — `move |e: Etape| -> BoxFuture<'static, ()> { … }` — faute de
quoi son type de retour serait `Pin<Box<[bloc async]>>` et ne coercerait pas (validation P3, R3-6). Aucun drapeau, aucune variable d'environnement, aucune `feature` : la production
emprunte **exactement** le chemin testé (C-28-3-6). La signature du cœur ne reçoit **aucun pool** (AC2, point 2).

**Moteur de stockage : contrôlé dans les tests, écrit au manuel, pas à l'exécution** (C-28-3-25, qui remplace
C-28-3-17). Le cœur **ne contrôle pas** le moteur des tables qu'il lit, et la sauvegarde n'échoue jamais pour cette
raison. Le contrôle vit dans les tests : le squash de test (`ENGINE=InnoDB` sur ses 40 tables, Dev Notes § *Moteurs de stockage, au sol*),
`test_schema_guard.rs` (qui compare à chaque gate le moteur du vrai schéma migré à celui du squash, `schema_facts`, l. 154-168) et
l'AC5 (e) (toute table sauvegardée est InnoDB sur la base de test). Motif : 2 des 40 `CREATE TABLE` des migrations
(`contacts`, `contact_persons`) n'ont pas de clause `ENGINE` et prennent, sur la base servie, le moteur par défaut du
**serveur** ; or l'installation réelle (le NAS de Guy) tourne le paquet MariaDB de Synology, dont le moteur par défaut
n'est pas vérifié. Une garde qui échouerait fermée y refuserait **toute** sauvegarde et **tout** import, en
permanence : pire que le défaut qu'elle garde. Le manuel d'administration dit la limite, la vérification et le remède
(AC10).

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

**Protocole des instructions `SET` des tests** (validation P3, F3-2) : toute instruction `SET` émise par un test —
`SET SESSION TRANSACTION ISOLATION LEVEL READ COMMITTED` ((a-RC), (f)), et le repli `SET TRANSACTION READ WRITE` de (d)
— passe par `conn.execute("…")` sur un `&str` (protocole texte), comme en production (C-28-3-19), jamais par
`sqlx::query(…)`.

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
T1, et à copier au Dev Agent Record : (a-défaut), (a-RC) et (b) rouges — assertion 2 ou 3, orphelins listés. Côté
production, T2 ne change que l'**intérieur** de `ReadSnapshot` (`begin_on` ouvre l'instantané de l'AC1, `finish` fait
le `ROLLBACK`, le champ devient une `sqlx::Transaction<'static, MySql>`) ; côté `mod tests`, T2 **ajoute** des tests
neufs ((c), (d1), (d2), (e), (f), (g)(i), (h)) mais ne touche **ni** les trois tests (a-défaut), (a-RC), (b), **ni** leurs
helpers (`orphelins`, le montage commun, l'observateur s'il est factorisé). Les trois tests passent au vert sans que
leur texte ni celui de leurs helpers ne change. Preuve, au Dev Agent Record, une commande **par fonction** (chacun
des trois tests et chacun de leurs helpers), sortie copiée, **vide** attendue (C-28-3-27) :

```sh
git log --oneline -L '/fn <fonction>(/,/^    }$/:crates/kesh-db/src/backup.rs' <commit T1>..<commit T2>
```

⚠️ Forme à expressions, **non** `-L :<fonction>:` : le dépôt n'a pas de `.gitattributes` qui associe `*.rs` au pilote
`diff=rust`, et le pilote par défaut ne reconnaît comme en-tête de fonction qu'une ligne non indentée ; pour une
fonction de `mod tests` (indentée de quatre espaces), la plage s'étendrait jusqu'à la ligne non indentée suivante,
engloberait les tests neufs de T2 et rendrait une sortie non vide sur un texte inchangé. La borne de fin `^    }$`
est l'accolade fermante d'une fonction de `mod tests` formatée par `rustfmt` ; le dev vérifie, sur le commit T2, que
chaque plage couvre bien la fonction entière et elle seule (`git show <T2>:crates/kesh-db/src/backup.rs` et numéros
de ligne copiés), sans quoi une sortie vide ne prouverait rien.

Complément obligatoire (validation P3, R3-5, F3-3) — une sortie vide est aussi celle d'une plage **non résolue** (le
`fatal:` part sur stderr), et la plage commence à la ligne `fn`, si bien que l'attribut `#[sqlx::test(…)]` qui choisit
le schéma lui échappe :

1. chaque commande `git log -L` est suivie de `; echo "exit=$?"`, et `exit=0` est copié avec la sortie ;
2. **témoin positif** : la même commande sur l'intervalle `<commit T1>~1..<commit T2>` rend une sortie **non vide**
   (la fonction naît en T1) — preuve que la plage se résout et que la commande voit un changement quand il y en a un ;
3. **ajouts purs** : `git diff -U0 <commit T1> <commit T2> -- crates/kesh-db/src/backup.rs`, dont chaque hunk situé
   après la ligne `mod tests` est de la forme `@@ -N,0 +M,K @@` (aucune ligne ancienne retirée ni modifiée) — ce qui
   couvre attributs, `///` et helpers de toute forme ; toute exception est expliquée au Dev Agent Record. Les helpers
   visés sont des `fn` à quatre espaces (`orphelins`, le montage, l'observateur s'il est factorisé en `fn`).

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
`CONNECTION_ID()` identique avant et après. `CONNECTION_ID()` est `BIGINT UNSIGNED` et se lit
`SELECT CAST(CONNECTION_ID() AS SIGNED)` ; de même `SELECT CAST(@@in_transaction AS SIGNED)` (précédent
`onboarding_audit_e2e.rs:2632`, `:2656` ; validation P3, F3-1). Les lectures se font par `fetch_one` et `expect` :
une erreur rend le test rouge, jamais muet. `sqlx::Transaction` ne rend pas sa connexion : `finish` et le `drop` la
remettent au pool, d'où ce montage. **(d1)** `begin_on`, une lecture (`read_min_required`), puis `finish` ;
**(d2)** `begin_on`, la même lecture, puis `drop` sans `finish` (chemin d'erreur). Dans les deux cas, la connexion réacquise n'est plus en transaction : `SELECT @@in_transaction`
rend `0` (C-28-3-26, qui remplace C-28-3-22). L'existence de `@@in_transaction` en 10.11 n'est pas établie par
requête : **le test lui-même la vérifie** (une erreur « variable inconnue » le fait échouer, il ne passe pas à vide).
Si elle n'existe pas — erreur MariaDB **1193** (`ER_UNKNOWN_SYSTEM_VARIABLE`), lue par `number()`, et **non** une
erreur de décodage, qui appellerait un `CAST` —, le dev remplace la forme par un repli **sans privilège** : sur la connexion réacquise,
`SET TRANSACTION READ WRITE` (sans portée) doit **réussir** — dans une transaction ouverte, le serveur le refuse en
`1568` (`ER_CANT_CHANGE_TX_CHARACTERISTICS`, fait non vérifié, tranché par ce test sous M6) ; le numéro se lit comme
en (c). Le Dev Agent Record écrit laquelle des deux formes a servi, et la fiche est corrigée en conséquence. ⛔ Pas de
repli sur `information_schema.INNODB_TRX` : il exige le privilège `PROCESS`, que l'utilisateur de dev `kesh` n'a pas
(`reconciliation_e2e.rs:7244-7248`) — le test passerait en CI (`root`) et échouerait sur le poste de dev.

**(e) Toutes les tables sauvegardées sont InnoDB sur la base de test, et `export_installation` les rend toutes**
(C-28-3-25 ; garde de moteur **de test**, non d'exécution — AC4). Le test lit
`SELECT TABLE_NAME, ENGINE FROM information_schema.TABLES WHERE TABLE_SCHEMA = DATABASE()` (`ENGINE` en
`Option<String>`, comme `test_schema_guard.rs:166` ; un `NULL` compte comme « pas InnoDB »), et vérifie que chaque
table de `TABLES_TO_TRUNCATE` et `_kesh_version` y figure (assertion de montage : `TABLES_TO_TRUNCATE.len() + 1` noms
**attendus** trouvés parmi ceux de la base, qui porte aussi `_sqlx_migrations` — sans elle, une liste vide passerait
à vide) avec le moteur `InnoDB` ; le message d'échec nomme la table et
son moteur. Puis `export_installation(&pool)` rend `Ok(e)` avec `e.tables` égal, nom pour nom et dans l'ordre, à
`TABLES_TO_TRUNCATE` — c'est la couverture, dans `kesh-db`, du chemin public vers le cœur. Depuis la revue de code
P1 (E3, C-28-3-42), une société est d'abord seedée, et `e.min_required` et `e.instance_id` sont comparés aux valeurs
lues directement sur la base (`_kesh_version`, `MIN(companies.id)` ≥ 1) : mutations P1-MC (`Ok(0)`) et P1-MD (autre
colonne lue) tuées. Ce test rougit si une
migration future crée une table sauvegardée en `MyISAM`, `Aria` ou `MEMORY` par une clause explicite (le squash la
reproduirait, et `test_schema_guard.rs` ne verrait rien, les deux schémas étant d'accord). Il ne voit **pas** le moteur
par défaut d'un serveur d'exploitation : c'est l'objet du manuel (AC10).

**(f) Le niveau d'isolation de la session n'est pas modifié.** Même montage que (d) : sur la connexion passée en
`SET SESSION TRANSACTION ISOLATION LEVEL READ COMMITTED`, après une sauvegarde par `begin_on` puis `finish`, la connexion
réacquise rend à `SELECT @@tx_isolation` toujours `READ-COMMITTED` : le `SET TRANSACTION` de l'AC1 ne vaut que pour la
transaction de l'instantané. (MariaDB 10.11 : `@@tx_isolation` porte le niveau de **session** ; c'est lui qu'on
contrôle ici, non celui de la transaction en cours.)

**(h) Une erreur du `ROLLBACK` final est rendue par `finish`** (C-28-3-20 ; validation P3, F3-7). Sur un `ReadSnapshot`
de `begin(&pool)`, après une lecture, le test lit `CAST(CONNECTION_ID() AS SIGNED)` **par l'instantané**, puis tue
cette connexion depuis une autre connexion du pool (`KILL <id>` — un compte peut tuer ses propres threads, sans
privilège `PROCESS`), puis appelle `finish` : `Err` attendu. Assertion de montage : le `KILL` a réussi. Si le
comportement de sqlx rend ce test non déterministe (fait non vérifié), le dev le retire, l'écrit au Dev Agent Record
et au registre, et l'AC1 garde l'angle mort.

**(j) Un `SET TRANSACTION` en échec ferme la connexion** (ajouté en revue de code P1, A-4/E2, C-28-3-40). Même montage
que (d) : sur la connexion unique, une transaction **brute** est ouverte (`conn.execute("BEGIN")`, que sqlx ignore —
assertion de montage : `@@in_transaction` = 1), puis `begin_on(conn)` : `Err` attendu, de numéro `1568`
(`ER_CANT_CHANGE_TX_CHARACTERISTICS`). La connexion réacquise a un **autre** `CONNECTION_ID()` et n'est pas en
transaction : celle du `SET` en échec a été fermée (`close_on_drop`), non rendue au pool transaction ouverte. Test
`echec_du_set_ferme_la_connexion` ; tue la mutation « suppression de `conn.close_on_drop();` » (P1-MA).

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
Aucun changement de comportement à l'import : la story ne contrôle pas le moteur des tables à l'exécution
(C-28-3-25), et la sauvegarde de sécurité n'échoue donc pas pour une raison neuve. Aucun
test neuf exigé ici : `admin_full_import_e2e.rs` exerce déjà ce chemin de bout en bout (un interblocage le ferait
rougir ou pendre).

### AC8 — Les commentaires qui décrivent le défaut sont mis à jour (propagation)

- `crates/kesh-db/src/repositories/bank_imports.rs:447-448` : la sauvegarde d'installation ne porte plus
  d'orphelins ; **reste** l'export global CSV, #465.
- `crates/kesh-api/src/admin_backup/export.rs` : en-tête (`:1-13` — et le « (22) » périmé de `:11`, à remplacer par
  « une par table de `TABLES_TO_TRUNCATE` », **sans nombre**) et doc-comment de `build_keshbackup`.
- `crates/kesh-db/src/backup.rs` : doc du module (`:1-17`) et de `export_table` ; et le commentaire `:488-489`, qui
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
> modifié ou supprimé pendant qu'il se prépare n'y figure pas, et une saisie concurrente ne peut plus y laisser une
> ligne sans son en-tête — une transaction sans son import, une ligne de facture sans sa facture. L'export ne bloque
> aucune saisie. La sauvegarde de sécurité prise avant un import (plus bas) est lue de la même façon. Une base qui
> porte **déjà** de telles lignes, pour avoir restauré une sauvegarde incohérente d'une version antérieure, les garde,
> et l'export les reproduit (C-28-3-28). ⚠️ *Avant la 0.13.3*, un export pris pendant une saisie pouvait porter de
> telles lignes ; une sauvegarde de cette époque que l'on compte garder se **refait**.
>
> L'instantané ne vaut que pour les tables **InnoDB**, le moteur transactionnel de MariaDB ; une table d'un autre
> moteur y échappe, **sans que l'export le signale**. Sur l'image MariaDB de `docker-compose.yml`, toutes les tables
> de Kesh sont InnoDB. Une table, `contact_persons`, est toutefois créée sans moteur explicite et prend le moteur
> par défaut du serveur (une seconde, `contacts`, l'est aussi, mais les clés étrangères InnoDB qui la visent
> l'obligent à être InnoDB) : sur une base fournie par ailleurs (`docker-compose.prod.yml` — paquet MariaDB de
> Synology, serveur ou base gérée), le vérifier une fois, par le client MariaDB, en **nommant** la base de Kesh
> (`kesh` par défaut, la valeur de `MARIADB_DATABASE`) :
> `SELECT COUNT(*) FROM information_schema.TABLES WHERE TABLE_SCHEMA = 'kesh';` — **au moins 40** (sinon la base
> nommée n'est pas celle de Kesh, et la requête suivante ne prouverait rien) ; puis
> `SELECT TABLE_NAME, ENGINE FROM information_schema.TABLES WHERE TABLE_SCHEMA = 'kesh' AND (ENGINE IS NULL OR
> ENGINE <> 'InnoDB');` — **aucune ligne** attendue, ou seulement `_sqlx_migrations` (le registre des migrations,
> qui n'est pas sauvegardé : son moteur est sans effet sur l'instantané). Remède, si `contact_persons` est listée,
> après un dump (`kesh-dump.sh`) : `ALTER TABLE contact_persons ENGINE=InnoDB;`, puis `SHOW CREATE TABLE
> contact_persons;` — hors InnoDB, MariaDB n'enregistre pas les clés étrangères : si les deux lignes
> `CONSTRAINT fk_contact_persons_company` et `fk_contact_persons_contact` manquent, les recréer :
> `ALTER TABLE contact_persons ADD CONSTRAINT fk_contact_persons_company FOREIGN KEY (company_id) REFERENCES
> companies(id) ON DELETE CASCADE, ADD CONSTRAINT fk_contact_persons_contact FOREIGN KEY (contact_id) REFERENCES
> contacts(id) ON DELETE CASCADE;`. Et régler `default_storage_engine=InnoDB` dans la configuration du serveur, pour
> les tables à venir.

Et, au même endroit ou au § *Quelle méthode de sauvegarde choisir ?* (`:1901`), une phrase : l'**export global**
(CSV, *Administration → Export global*) ne bénéficie **pas encore** de cette garantie (#465). Côté manuel utilisateur
(`docs/manual/fr/user-manual.tex`, § *Exports de souveraineté*, après « Lancer l'export », `:2593-2595`) : une phrase
honnête — l'export lit ses tables l'une après l'autre ; une saisie faite pendant qu'il se prépare peut n'y figurer
qu'en partie ; le lancer quand personne ne saisit (#465).

**PDF régénérés** (`make fr` dans `docs/manual/`, sous `flock`), et contrôlés **aplatis** :
`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "un seul instantané[^.]*"` (et de
même pour le manuel utilisateur), la phrase de moteur
(`… | grep -o "ne vaut que pour les tables[^.]*"`, une occurrence attendue) et la requête de vérification
(`… | grep -c "ENGINE <> 'InnoDB'"` → au moins `1` ; `lstlisting` et ligature du PDF peuvent altérer l'apostrophe :
en cas de `0`, contrôler la forme rendue et l'écrire au Dev Agent Record), **et** l'absence de l'ancienne promesse :
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
>   unique, en lecture seule, qui ne bloque aucune saisie ; la garantie vaut pour les tables InnoDB, et le manuel
>   d'administration dit comment vérifier le moteur d'une base fournie par ailleurs. Une base qui porte déjà de telles
>   lignes les garde. Une sauvegarde prise sous une version antérieure pendant
>   que l'on saisissait se refait. L'export global CSV n'en bénéficie pas encore ([#465](https://github.com/guycorbaz/kesh/issues/465)).

### AC12 — Aucune migration, aucun changement d'API ni d'écran

Aucun fichier sous `crates/kesh-db/migrations/` ni `test-schema/` ; `kesh_version_min_required` inchangé ;
`docs/migrations-idempotence-audit.md` et `post_restore.rs` (P5, P7) non concernés. Format `.keshbackup` inchangé
(un fichier produit par la 0.13.3 s'importe dans une 0.13.2, et réciproquement). Route, codes d'erreur, i18n,
frontend : inchangés.

## Tasks / Subtasks

- [x] **T0 — Montage** (AC —) : bases `kesh_283` et `kesh_e2e_283` (migrations du worktree, seed
  `scripts/seed-dev-db.sql`), port `3036`, `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-28-3/target`, `node_modules`
  par `cp -al` depuis le dépôt principal. Gates lourds sous `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock`.
- [x] **T1 — Le test rouge d'abord** (AC2 point 1, AC3, AC4, AC5 (a), (b)) : `crates/kesh-db/src/snapshot.rs` avec
  le `ReadSnapshot` **dégradé** (C-28-3-16 : `begin`, `begin_on`, accès connexion, `finish` ; aucune transaction),
  `pub mod snapshot;` dans `lib.rs` ; les quatre lectures de `backup.rs` sur `&mut ReadSnapshot` ;
  `InstallationExport`, `export_installation` et son cœur observable (sans pool, AC4) ; `build_keshbackup` sur
  `export_installation` et adaptation des quatre tests appelant `export_table` — pour que le **workspace** compile à
  ce commit ; helper `orphelins` ; tests (a-défaut), (a-RC), (b). Exécuter, constater le **rouge** des trois, copier
  la sortie au Dev Agent Record. Commit (`test(28-3): … rouge avant la correction (refs #633)`).
- [x] **T2 — L'instantané** (AC1, AC2 point 3 (i)) : côté production, l'intérieur de `ReadSnapshot` seul
  (`SET TRANSACTION` en protocole texte, `close_on_drop` sur son échec, `Transaction::begin`, `ROLLBACK` de `finish`,
  commentaire de dépendance `#[doc(hidden)]` — C-28-3-19). Côté `mod tests`, tests neufs (c), (d1), (d2), (e), (f),
  (g)(i), (h) ; les tests (a-défaut), (a-RC), (b) et leurs helpers ne changent pas et passent au vert (preuve
  `git log -L` par fonction, sorties vides et `exit=0` copiés, témoin positif, `git diff -U0` en ajouts purs, AC5 (a),
  C-28-3-27).
- [x] **T3 — `kesh-api`** (AC2 point 3 (ii), AC3, AC6, AC7) : garde (g)(ii) dans un `mod tests` neuf de
  `admin_backup/export.rs` ; commentaire `admin.rs:254-262` ; suites de l'AC6 vertes.
- [x] **T4 — Propagation** (AC8) : commentaires, grep de propagation copié au Dev Agent Record.
- [x] **T5 — Documentation** (AC10, AC11) : manuels, PDF régénérés et contrôlés aplatis, CHANGELOG.
- [x] **T6 — Mutations et gates** (AC1-AC12) : campagne M1-M11 (ci-dessous ; M12 retirée, C-28-3-25) ; base remise à zéro **avant** le gate
  complet ; gate complet `scripts/test-fast.sh` (**`kesh-db` touché : gate complet même en boucle de revue**) ;
  `npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build` ; **E2E complet au dernier
  commit de code** (aucun écran ne change, mais la règle D7 ne connaît pas d'exception). **Demandé à l'orchestrateur, hors
  agent** (R2-1, C-28-3-25) : avant la publication de la 0.13.3, les deux requêtes de moteur de l'AC10 (témoin de compte puis liste), en lecture
  seule, sur la base du NAS **nommée** — « conforme » exige le témoin à au moins 40 et aucune ligne hors
  `_sqlx_migrations` ; résultat écrit « conforme / non conforme » au Dev Agent Record (jamais de nom de base ni d'hôte
  réels) ; non conforme, le remède du manuel s'applique avant la release. Cela ne bloque pas la story.

### Mutations à exécuter (T6)

Chaque mutation : motif vérifié unique, appliquée, test ciblé rejoué, restauration `git checkout -- <fichier>` +
`touch` (cargo garde sinon le binaire muté). M1 et M6 ne sont pas des remplacements de motif mais des
**réécritures** de l'intérieur de `snapshot.rs` : elles s'appliquent par un patch préparé (`git apply`), dont le
`git diff` est copié au journal de mutation avant le test. M9 vise **la** ligne `ENGINE=InnoDB` de `_kesh_version`,
qui figure 40 fois dans le squash : le motif unique est le couple de lignes `CONSTRAINT chk_kesh_version_single_row …`
+ `) ENGINE=InnoDB …` (validation P3, F3-8). Résultat attendu écrit ; une mutation qui survit est un défaut du test
ou une équivalence **à démontrer**, jamais à déclarer.

| # | Mutation | Test qui doit rougir |
|---|---|---|
| M1 | l'intérieur de `ReadSnapshot` redevient celui de T1 : `begin_on` garde la connexion **sans** ouvrir de transaction, `finish` la rend (lectures en autocommit). Redéfinie en P2 (R2-5/F2-4) : la forme d'origine, « une connexion neuve du pool par table », n'est pas représentable dans un cœur qui ne reçoit aucun pool (AC2 point 2) ; sa forme représentable est M11 | AC5 (a-défaut), (a-RC), (b) |
| M2 | `START TRANSACTION WITH CONSISTENT SNAPSHOT, READ ONLY` → `START TRANSACTION READ ONLY` | AC5 (b) |
| M3 | suppression du `SET TRANSACTION ISOLATION LEVEL REPEATABLE READ` | AC5 (a-RC) |
| M4 | `SET TRANSACTION …` → `SET SESSION TRANSACTION …` | AC5 (f) |
| M5 | suppression de `, READ ONLY` | AC5 (c) |
| M6 | `ReadSnapshot` ouvre la transaction par un `execute("START TRANSACTION …")` brut sur la `PoolConnection`, hors de `sqlx::Transaction` (le piège de l'AC1) ; `finish` se contente de rendre la connexion | AC5 (d1), (d2) — `@@in_transaction` à `1`. ⚠️ Un `finish` qui oublie le `ROLLBACK` **mais** garde la `sqlx::Transaction` est, lui, **équivalent** (le `Drop` de sqlx l'émet) : à consigner comme tel, non comme survivant |
| M7 | l'observateur n'est plus appelé après chaque table | AC5 (a) — assertion de montage (compteur) |
| M8 | le helper `orphelins` ne lit plus que les FK d'une table (filtre `TABLE_NAME = 'invoices'`) | AC5 (a) — assertion de montage du helper |
| M9 | dans le squash `test-schema/0001_schema_squash.sql`, `_kesh_version` passe de `ENGINE=InnoDB` à `ENGINE=MyISAM` (mutation temporaire, restaurée ; AC12 ne vise que le livrable) | AC5 (e) — moteur nommé dans le message. (`test_schema_guard.rs` rougit aussi : à noter, sans en dépendre) |
| M10 | `build_keshbackup` relit `min_required` dans un **second** instantané (`kesh_db::snapshot::ReadSnapshot::begin(pool)` puis `read_min_required`) — **compile** (C-28-3-15) | AC5 (g)(ii) — `pool` trois fois. Aucun test fonctionnel ne la tue (`min_required` ne change qu'au démarrage) : c'est la raison d'être de la garde |
| M11 | `export_installation` ouvre un second `ReadSnapshot::begin(pool)` pour l'une des lectures — compile | AC5 (g)(i) — `pool` trois fois |
| M12 | *retirée en P2* (C-28-3-25) : il n'y a plus de garde de moteur à l'exécution | — |

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
tiennent un pool. La story n'ajoute **aucune** lecture à la sauvegarde (la garde de moteur à l'exécution de la
remédiation P1 a été retirée en P2, C-28-3-25). **Sites non résolus après la story** : aucun pour la sauvegarde
**dans les formes couvertes** ; angles morts écrits à l'AC2 (une lecture faite par un appelant et passée en
paramètre ; un alias du pool) ; l'export global (`exports/global.rs`,
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
→ 39) + `_kesh_version`. Toutes InnoDB. Côté migrations, recompté en P2 (hors lignes de commentaire, C-28-3-25) :

```text
$ cd crates/kesh-db/migrations
$ grep -v '^\s*--' *.sql | grep -ci 'create table' ; grep -h -v '^\s*--' *.sql | grep -ci 'ENGINE\s*=\s*InnoDB'
40
38
$ for f in *.sql; do c=$(grep -v '^\s*--' "$f" | grep -ci 'create table'); \
    e=$(grep -v '^\s*--' "$f" | grep -ci 'ENGINE\s*=\s*InnoDB'); [ "$c" != "$e" ] && echo "$f create=$c engine=$e"; done
20260414000001_contacts.sql create=1 engine=0
20260706000001_contact_persons.sql create=1 engine=0
```

Soit **2 des 40** instructions `CREATE TABLE` sans clause `ENGINE` — `contacts` et `contact_persons` —, créées donc
au moteur par défaut **du serveur**. (Les « 51 » et « 38 » écrits jusqu'en P1 comptaient des lignes, dont 11
lignes de commentaire du côté `CREATE TABLE` ; les « 13 » en étaient la soustraction, et « tables recréées » n'avait
pas de fondement. La lentille R de P2 avait recompté **5** par un `awk` qui clôt une instruction au premier `;`,
y compris dans un commentaire `--` en fin de ligne ou une chaîne `COMMENT '…'` : `bank_imports` — l. 25,
`-- 'CAMT053_V04', … ; CSV variants` —, `bank_profiles` — l. 39, `CHECK (field_separator IN (',', ';', …))` — et
`api_keys` — l. 25, `COMMENT '… ; NULL = permanente …'` — portent bien `ENGINE=InnoDB`, l. 53, 58 et 38 de leurs
fichiers.) Le squash fait foi pour la base de
test — `test_schema_guard.rs` compare d'ailleurs le moteur du squash à celui du vrai schéma (`schema_facts`, l. 154-168)
—, mais **pas** pour une base servie dont le serveur aurait un autre `default_storage_engine` (aucun fichier du dépôt
ne le fixe : `grep -rniE "default.storage.engine"` sans résultat). La garde à l'exécution qu'en tirait la remédiation
P1 (C-28-3-17) est **retirée** (C-28-3-25) : l'installation réelle (NAS, paquet MariaDB de Synology) a un moteur par
défaut non vérifié, et une garde qui échouerait fermée y refuserait toute sauvegarde et tout import. Le contrôle reste
dans les tests (AC5 (e), `test_schema_guard.rs`) ; la limite, la vérification et le remède sont au manuel (AC10).
`_sqlx_migrations` n'est pas lue par la sauvegarde.

Exposition réelle (validation P3, R3-2, F3-4) : `contacts` est la cible de clés étrangères posées par des tables
`ENGINE=InnoDB` (`20260416000001_invoices.sql:32`, `20260627000001_credit_notes.sql:44`,
`20260628000001_supplier_invoices.sql:56`, et `contact_persons.sql:26`) ; une clé étrangère InnoDB vers une table d'un
autre moteur est refusée, si bien qu'une installation migrée a `contacts` en InnoDB. Seule `contact_persons`, cible
d'aucune clé, est exposée. Elle porte deux clés étrangères (`fk_contact_persons_company`, `fk_contact_persons_contact`)
que MariaDB n'enregistre pas hors InnoDB : le remède du manuel le dit et prescrit le contrôle (`SHOW CREATE TABLE`)
puis la recréation. `_sqlx_migrations`, créée par sqlx sans clause `ENGINE`, peut aussi être listée par la requête du
manuel ; elle n'est pas sauvegardée, le manuel le dit.

### DDL concurrente, durée, purge — ce que l'instantané change et ce qu'il ne change pas

- **DDL** : Kesh n'en exécute qu'au démarrage (`MIGRATOR.run`, `main.rs:149`), avant d'écouter (`main.rs:382`) ; les
  `TRUNCATE` de `test_fixtures.rs:401` sont de test ; la restauration et la remise à zéro procèdent par `DELETE`
  (`backup.rs:496`, `kesh-seed/src/lib.rs:586`). Une DDL concurrente ne peut donc venir que d'un second processus
  (un autre conteneur qui démarre et migre sur la même base). Dans ce cas : une table **déjà lue** est protégée par
  le verrou de métadonnées que la transaction garde jusqu'à sa fin — la DDL **attend** la fin de la sauvegarde ; une
  table **pas encore lue** et **reconstruite** entre-temps fait échouer sa lecture (`ER_TABLE_DEF_CHANGED`, 1412) :
  la sauvegarde échoue, **fermée** (`AdminFullExportFailed`). Une DDL **sans reconstruction** (`ALGORITHM=INSTANT`,
  le défaut de 10.11 pour un `ADD COLUMN` qui le permet) n'échoue pas forcément : la lecture peut rendre la structure
  neuve, colonne ajoutée à sa valeur par défaut, alors que `INFORMATION_SCHEMA.COLUMNS` a été lu à un autre moment —
  non établi (aucune requête). Ni « jamais incohérente » ni l'inverse ne sont donc affirmés : la sauvegarde échoue,
  ou lit la structure neuve. Angle mort assumé, non testé (C-28-3-8, C-28-3-31).
- **Durée** : la transaction couvre 80 requêtes (2 + 39 × 2 : `min_required` et `instance_id`, puis colonnes et
  lignes de chaque table — la 81ᵉ, celle de la garde de moteur de P1, est retirée avec elle, C-28-3-25), sans compression (C-28-3-5) ; quelques secondes au plus
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
**neuve**, sur laquelle le `SET TRANSACTION` n'a pas été posé. `snapshot.rs` **signale** la dépendance en commentaire
(« dépend de `sqlx::Transaction::begin` (`doc(hidden)`), sqlx 0.8.6 ; contrôlé par AC5 (d) à chaque montée de
sqlx ») ; elle n'est **pas épinglée** : `crates/kesh-db/Cargo.toml:16` porte `sqlx = { version = "0.8", … }`, et
seul `Cargo.lock` tient la 0.8.6 — un `cargo update` passerait à une 0.8.x suivante sans signal (C-28-3-30). À une
montée de sqlx, un changement de signature rougit à la compilation, et AC5 (a-RC), (d) et (f) sont les tests qui
diraient qu'un comportement a changé (C-28-3-19). Épingler `=0.8.6` est écarté : tout le workspace en dépend, et la
montée de sqlx est une décision à part.

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
| `@@in_transaction` est une variable de session lisible (sinon repli `SET TRANSACTION READ WRITE` → `1568` dans une transaction ouverte, C-28-3-26) | AC5 (d) — le test le vérifie lui-même |
| `SET TRANSACTION ISOLATION LEVEL REPEATABLE READ` sans portée l'emporte sur une session `READ COMMITTED` au moment où `START TRANSACTION WITH CONSISTENT SNAPSHOT` assigne la vue | AC5 (a-RC) — rouge **sur le code correct** s'il était faux |
| `START TRANSACTION … READ ONLY` laisse `SERVER_STATUS_IN_TRANS` posé (sinon sqlx rend `BeginFailed`) | tous ceux de l'AC5 (échec à l'ouverture) |
| `SET TRANSACTION READ WRITE` hors transaction réussit, et rend `1568` dans une transaction ouverte (repli seulement) | AC5 (d) sous M6, si le repli sert |
| `@@tx_isolation` reflète le niveau de session, et `SET TRANSACTION` sans `SESSION` ne le change pas | AC5 (f) **sous M4** |
| une `DELETE` concurrente n'attend pas la transaction de lecture (lectures non verrouillantes, verrou de métadonnées partagé compatible) | AC5 (a) — il pendrait sinon |

Si l'un d'eux est faux, la fiche est fausse à cet endroit : corriger la fiche et le registre, non le test. Pour la
ligne `SET TRANSACTION … l'emporte`, le remède est prescrit : le **plan B** de l'AC1.

S'y ajoutent, de la validation P3 : `CONNECTION_ID()` et `@@in_transaction` se décodent par `CAST(… AS SIGNED)` (AC5
(d)) ; un `KILL` de la connexion de l'instantané fait échouer le `ROLLBACK` de `finish` (AC5 (h)) ; hors InnoDB,
MariaDB n'enregistre pas les clés étrangères d'un `CREATE TABLE` (le manuel prescrit un contrôle par `SHOW CREATE
TABLE` plutôt que de s'y fier, AC10) ; une clé étrangère InnoDB vers une table non InnoDB est refusée (motif pour
lequel `contacts` n'est pas exposée — non exécuté, raisonnement d'après la documentation).

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
- Registre : `epic-28-choix-autonomes.md` § *Story 28-3* (le registre séparé de la création y a été versé par union).

## Dev Agent Record

### Agent Model Used

Opus 5.5 (`bmad-dev-story`, autonomie), worktree `/home/gcorbaz/devel/kesh-28-3`, port 3036, bases `kesh_283` /
`kesh_e2e_283`.

### Debug Log References

**T1 — rouge avant la correction** (commit `a914762b`, `ReadSnapshot` dégradé : aucune transaction ; commande
`cargo nextest run -p kesh-db --lib --no-fail-fast -E 'test(/sauvegarde_/)'`, journal complet
`kesh-gate-logs/28-3-t1-rouge.log`, sortie copiée) :

```text
thread 'backup::tests::sauvegarde_ignore_une_suppression_en_session_read_committed' panicked at crates/kesh-db/src/backup.rs:1545:9:
assertion `left == right` failed: la sauvegarde doit porter l'import et ses deux transactions (orphelins : ["bank_transactions.import_id=1 → bank_imports.id absent", "bank_transactions.import_id=1 → bank_imports.id absent"])
  left: (0, 2)
 right: (1, 2)
thread 'backup::tests::sauvegarde_ignore_une_suppression_entre_deux_tables' panicked at crates/kesh-db/src/backup.rs:1545:9:
assertion `left == right` failed: la sauvegarde doit porter l'import et ses deux transactions (orphelins : ["bank_transactions.import_id=1 → bank_imports.id absent", "bank_transactions.import_id=1 → bank_imports.id absent"])
  left: (0, 2)
 right: (1, 2)
thread 'backup::tests::sauvegarde_prend_l_instantane_a_l_ouverture' panicked at crates/kesh-db/src/backup.rs:1545:9:
assertion `left == right` failed: la sauvegarde doit porter l'import et ses deux transactions (orphelins : [])
  left: (0, 0)
 right: (1, 2)
     Summary [   1.475s] 3 tests run: 0 passed, 3 failed, 384 skipped
```

(a-défaut) et (a-RC) : les deux transactions lues avant la suppression, l'import absent — deux orphelins, exactement
le défaut de #633. (b) : la suppression faite avant la première lecture est vue (vue prise par requête) — ni import
ni transactions.

**T2 — vert, même texte de test** (commit `1894bf0c`). Les trois tests passent (`3 passed`). Preuve (C-28-3-27 et
complément P3), journal `kesh-gate-logs/28-3-preuve-git-log-L.log`, sorties copiées — pour chacune des huit fonctions
(`monter_import`, `lignes`, `orphelins`, `observateur_qui_supprime`, `sauvegarder_pendant_suppression`, et les trois
tests), `git log --oneline -L '/fn <f>(/,/^    }$/:crates/kesh-db/src/backup.rs' a914762b..1894bf0c` rend une sortie
**vide** et `exit=0` ; témoin positif sur `a914762b~1..1894bf0c` : sortie non vide pour chacune (70, 18, 40, 38, 66,
12, 19 et 11 lignes) ; lignes de départ au commit T2 : 1337, 1403, 1420, 1459, 1495, 1560, 1571, 1588. La plage de
`orphelins` se termine bien sur son `    }` (dernières lignes du témoin : `violations` puis `    }`). Ajouts purs :

```text
$ git diff -U0 a914762b 1894bf0c -- crates/kesh-db/src/backup.rs | grep '^@@'
@@ -1592,0 +1593,234 @@ mod tests {
```

Un seul hunk, ajout pur après le dernier test (`mod tests` est à la ligne 731 au commit T1) : T2 n'a touché ni les
trois tests, ni leurs helpers, ni leurs attributs, ni le code de production de `backup.rs` — seul `snapshot.rs` a
changé côté production.

**Mutations M1-M11 : 11/11 tuées** (journal `kesh-gate-logs/28-3-mutations.log`, sorties `28-3-mut/M<n>.log`, script
de campagne : motif vérifié unique par `str.count == 1`, M1 par `git apply` du diff T2→T1 de `snapshot.rs`, M6 par un
fichier préparé dont le diff est copié au journal ; restauration `git checkout -- <fichier>` puis `touch`, arbre
vérifié propre après chaque mutation ; sous `flock`). Chaque rouge vient de l'assertion attendue :

| # | Rouge attendu | Message observé |
|---|---|---|
| M1 | (a-défaut), (a-RC), (b) | les trois (et (c), (d1), (d2), (h)) — orphelins listés |
| M2 | (b) | `(0, 0)` au lieu de `(1, 2)` |
| M3 | (a-RC) | `(0, 2)`, deux orphelins |
| M4 | (f) | `niveau de session modifié` — `"REPEATABLE-READ"` |
| M5 | (c) | `une écriture dans l'instantané doit être refusée: MySqlQueryResult { rows_affected: 1, … }` |
| M6 | (d1), (d2) | `la connexion revient au pool en transaction` (et (h) : `finish` ne fait plus de `ROLLBACK`) |
| M7 | (a-défaut), (a-RC) | `assertion de montage : l'observateur n'a pas supprimé à l'étape ApresTable("bank_transactions")` |
| M8 | (a-défaut), (a-RC), (b) | `assertion de montage : orphelins() ne voit pas fk_bank_transactions_import (4 clés lues)` |
| M9 | (e) | `table _kesh_version : moteur Some("MyISAM")` |
| M10 | (g)(ii) | `` `pool` doit apparaître exactement deux fois `` — corps de `build_keshbackup` imprimé |
| M11 | (g)(i) | idem, corps de `export_installation` |

**Faits MariaDB tranchés** (serveur de dev `10.11.16-MariaDB`, `@@default_storage_engine = InnoDB`) :

| Fait de la fiche | Verdict | Par |
|---|---|---|
| `START TRANSACTION WITH CONSISTENT SNAPSHOT, READ ONLY` accepté, `SERVER_STATUS_IN_TRANS` posé (pas de `BeginFailed`) | **établi** | tous les tests de l'AC5 verts |
| sous `READ COMMITTED`, `WITH CONSISTENT SNAPSHOT` ne fige pas la vue | **établi** | (a-RC) rouge sous M3 |
| écriture en `READ ONLY` refusée en `1792` | **établi** | (c) vert (`number() == 1792`), rouge sous M5 |
| `@@in_transaction` lisible (avec `CAST`) | **établi** | (d1), (d2) verts ; `1` dans l'instantané, `1` après sous M6. Le repli `SET TRANSACTION READ WRITE` → `1568` n'a **pas** servi, et ce fait reste **non vérifié** |
| `SET TRANSACTION … REPEATABLE READ` sans portée l'emporte sur une session `READ COMMITTED` | **établi** | (a-RC) vert ; le plan B de l'AC1 n'a pas servi |
| `@@tx_isolation` reflète la session ; `SET TRANSACTION` sans `SESSION` ne la change pas | **établi** | (f) vert, rouge sous M4 |
| une `DELETE` concurrente n'attend pas la transaction de lecture | **établi** | (a) vert, sans pendre |
| `KILL` de la connexion → le `ROLLBACK` de `finish` échoue | **établi** | (h) vert, 20 exécutions sur 20 *(non tracé à cette date — revue P1, A-3 ; rejoué le 2026-10-13 au commit `6940b560` : 20 PASS sur 20, journal `kesh-gate-logs/28-3-h-20-executions.log`)* |
| le `Drop` de `sqlx::Transaction` rend la connexion hors transaction | **établi** | (d2) vert |
| hors InnoDB, MariaDB n'enregistre pas les clés étrangères ; une clé InnoDB vers une table non InnoDB est refusée | **non vérifié** (aucune exécution) | le manuel prescrit le contrôle `SHOW CREATE TABLE` plutôt que de s'y fier |

**Gates au dernier commit de code `90c85099`** (bases `kesh_283` et `kesh_e2e_283` supprimées, recréées, migrées et
seedées juste avant ; le conteneur MariaDB partagé n'a pas été redémarré) :

- **Backend** — `scripts/test-fast.sh` complet (fmt + clippy `-D warnings` + nextest), `DATABASE_URL` sur `kesh_283`,
  sous `flock` : **3525 tests exécutés, 3525 passés, 6 ignorés** (`Summary [154.648s]`), journal
  `kesh-gate-logs/28-3-gate-backend.log`.
- **Frontend** — `npm run check` (svelte-check : 0 erreur, 29 avertissements dans 5 fichiers, tous préexistants —
  la story ne touche aucun fichier frontend), `npm run lint-i18n-ownership` (PASS), `npm run test:unit` (**1395 tests,
  134 fichiers, tous passés**), `npm run build` (OK), sous `flock`, journal `kesh-gate-logs/28-3-gate-frontend.log`.
- **E2E complet** — recette de `docs/testing.md` à la lettre, port 3036, base `kesh_e2e_283`, `/health` :
  `smtpConfigured:true` ; aucun `Permission denied (os error 13)` au journal du backend : **265 passés, 9 échoués,
  19 ignorés** (10,8 min, run terminé à 11:26 UTC). Les 9 échecs, identifiés un par un : les **7 KF-029** (#97 —
  `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`) et
  les **2 KF-045** (#421 — `invoices.spec.ts:415` « historique des rappels affiché » et `:439` « axe-core sans
  violations », run avant 12:00 UTC). **Aucun échec hors liste.** Journaux `kesh-gate-logs/28-3-e2e*.log`.

### Completion Notes List

- **T0** : bases `kesh_283` / `kesh_e2e_283` créées, migrées (migrations du worktree) et seedées ; `node_modules`
  déjà présent en répertoire réel dans le worktree.
- **T1** (`a914762b`) : `ReadSnapshot` dégradé, les quatre lectures sur `&mut ReadSnapshot`, `InstallationExport`,
  `export_installation` et son cœur `export_installation_in` (sans pool, observateur `Etape`), `build_keshbackup` sur
  `export_installation`, quatre tests existants adaptés (`backup.rs` ×3, `accounts_role_backfill.rs`) ; tests
  (a-défaut), (a-RC), (b) rouges, sortie copiée ci-dessus.
- **T2** (`1894bf0c`) : intérieur de `ReadSnapshot` seul (`SET TRANSACTION` en protocole texte, `close_on_drop` sur
  son échec, `Transaction::begin`, `ROLLBACK` de `finish`, commentaire de dépendance `#[doc(hidden)]`) ; tests neufs
  (c), (d1), (d2), (e), (f), (g)(i), (h) ; preuve ci-dessus.
- **T3-T4** (`c00a7f63`, un seul commit : C-28-3-39) : garde (g)(ii) dans un `mod tests` neuf d'`export.rs` ;
  commentaire du pré-import (`routes/admin.rs`) ; propagation AC8. Greps de propagation au `HEAD` de code, sorties :
  le premier rend `exports/global.rs:21` (« Lectures successives, sans instantané commun » — **reste**, #465 ouvert,
  complété d'un renvoi au `ReadSnapshot`) et des mentions **neuves** de `#633` (`export.rs:8, :45, :150`,
  `routes/admin.rs:259`, `global.rs:26`, `backup.rs:15, :182, :342, :372, :1343`, `snapshot.rs:1`), toutes
  descriptives de la correction ; les anciennes (`routes/admin.rs:257-258` « lit via le pool (connexions séparées) »,
  « torn-read », `bank_imports.rs:448`) ont disparu. Le second (`export\.rs:[0-9]+`) ne rend **rien**.
- **T5** (`90c85099`) : manuel d'administration (phrase « aucun effet de bord » remplacée ; instantané ; avertissement
  « avant la 0.13.3 » ; limite InnoDB ; `contact_persons` seule exposée ; deux requêtes à base nommée, témoin ≥ 40,
  `_sqlx_migrations` ; remède avec `SHOW CREATE TABLE` et recréation des deux clés ; `default_storage_engine` ; export
  global, #465), manuel utilisateur (§ *Exports de souveraineté*), CHANGELOG `## [0.13.3] — Non publié` /
  `### Corrigé`. PDF régénérés par `make fr` (sous `flock`) : admin 3 passes, utilisateur 3 passes, stables ; le PDF de
  la brochure, régénéré sans que son `.tex` change, a été restauré (non commité). Contrôles aplatis (journaux
  `kesh-gate-logs/28-3-{admin,user}-manual.flat.txt`) : « un seul instantané… » — 1 occurrence dans chacun ; « ne vaut
  que pour les tables InnoDB… » — 1 ; `grep -c "ENGINE <> 'InnoDB'"` → `1` (apostrophe droite conservée par le
  `lstlisting`) ; « aucun effet de bord » → `0` au PDF et au `.tex` ; `_sqlx_migrations` → 1 ; recréation des clés
  rendue en entier. **Écart** : la garde G18 (`textes_coherents.rs`) rougissait sur `companies` dans le SQL du remède
  — fragment assumé ajouté, total 16 → 17 (C-28-3-37) ; grep du nombre `MULTI_SOCIETE_TOTAL` : aucun autre site.
- **Écarts à la fiche** : `orphelins` reçoit aussi le pool (C-28-3-33) ; observateur et montage factorisés en `fn`
  (C-28-3-34) ; `Etape`/`BoxFuture`/cœur privés (C-28-3-35) ; helper lexical local à `export.rs` (C-28-3-36) ; (h)
  gardé, témoin positif dans (d) (C-28-3-38) ; T3 et T4 en un commit, M3 par `DO 0` (C-28-3-39).
- **Décomptes** (périmètre `a4da521b..90c85099`, recomptés par `git diff … | grep -cE '^\+\s*(#\[sqlx::test|#\[test\])'`) :
  **11 tests neufs** — 9 `#[sqlx::test]` ((a-défaut), (a-RC), (b), (c), (d1), (d2), (e), (f), (h)) et 2 `#[test]`
  ((g)(i), (g)(ii)) ; 16 fichiers touchés (dont la fiche et `sprint-status.yaml`), aucun sous `migrations/`,
  `test-schema/`, ni `Cargo.toml` (AC12).
- **Incident d'exécution** : pendant l'attente du verrou de gate, un `pkill -f "until ! pgrep -f"` lancé pour arrêter
  mes propres boucles d'attente a tué aussi le shell qui lançait le premier E2E (le motif figurait dans sa ligne de
  commande — le piège du point 10 des consignes). Aucun processus d'un autre agent ne portait ce motif ; l'E2E a été
  relancé tel quel, et c'est son résultat qui est rapporté ci-dessus.
- **Demandé à l'orchestrateur** (hors agent, non bloquant, T6) : les deux requêtes de moteur de l'AC10 sur la base du
  NAS **nommée**, en lecture seule, avant la publication de la 0.13.3 — « conforme » exige le témoin ≥ 40 et aucune
  ligne hors `_sqlx_migrations`. Résultat : *à écrire ici par l'orchestrateur, « conforme / non conforme » seulement*.

### File List

- `crates/kesh-db/src/snapshot.rs` (neuf)
- `crates/kesh-db/src/lib.rs`
- `crates/kesh-db/src/backup.rs`
- `crates/kesh-db/src/repositories/bank_imports.rs`
- `crates/kesh-db/tests/accounts_role_backfill.rs`
- `crates/kesh-api/src/admin_backup/export.rs`
- `crates/kesh-api/src/routes/admin.rs`
- `crates/kesh-api/src/exports/global.rs`
- `crates/kesh-api/tests/textes_coherents.rs`
- `docs/manual/fr/admin-manual.tex`, `docs/manual/fr/admin-manual.pdf`
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/user-manual.pdf`
- `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/28-3-sauvegarde-instantane-commun.md`
- `_bmad-output/implementation-artifacts/28-3-review-prompt-p1.md` (neuf)
- `_bmad-output/implementation-artifacts/epic-28-choix-autonomes.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

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
- 2026-10-13 — **Validation P2** (Opus, lentilles R et F, contexte frais, lecture seule ; prompt
  `28-3-validate-prompt-p2.md`, rapports `kesh-gate-logs/28-3-validate-p2-{R,F}.md`) : **0 CRITICAL, 0 HIGH,
  5 MEDIUM, 14 LOW** (19 bruts — R : 3 MEDIUM, 5 LOW ; F : 2 MEDIUM, 9 LOW —, 17 distincts après les convergences
  R2-4=F2-3 et R2-5=F2-4). Des 5 MEDIUM, **4 sont nés de la remédiation P1** (R2-1, R2-2 : garde de moteur à
  l'exécution, C-28-3-17 ; F2-1 : repli `INNODB_TRX`, C-28-3-22 ; F2-2 : preuve `git diff` T1→T2, C-28-3-16) ; un est
  d'origine (R2-3, texte du manuel depuis la création).
  - **Signal D5** : la sévérité monte (P1 3 MEDIUM → P2 5 MEDIUM), et la hausse vient du correctif précédent — la
    forme « recyclage » que vise D5. **Décision de l'orchestrateur** (autonomie déléguée par Guy le 2026-10-13 ;
    précédents 27-3 et 27-4) : **pas de découpage**, remédiation **par simplification** — retirer les mécanismes
    neufs qui ont porté les défauts plutôt que les réparer. Remédiation (Opus 5.5) :
  - **R2-1, R2-2 (MEDIUM), F2-10 (LOW), et R2-6 (LOW) par voie de conséquence — garde de moteur retirée de
    l'exécution** : elle n'échoue plus fermé, et il n'y a pas d'avertissement journalisé non plus (option la plus
    simple ; un avertissement aurait exigé une requête de plus dans l'instantané et un test, pour un signal que
    personne ne lit). Le contrôle reste dans les tests (squash, `test_schema_guard.rs`, AC5 (e) refondu en un seul
    test qui couvre aussi `export_installation`) ; le manuel d'administration dit la limite (l'instantané ne vaut que
    pour les tables InnoDB), la vérification (`information_schema.TABLES`, `ENGINE <> 'InnoDB'`) et le remède
    (`ALTER TABLE … ENGINE=InnoDB`, `default_storage_engine`) ; CHANGELOG et AC7 alignés ; vérification sur le NAS
    demandée à l'orchestrateur, non bloquante (T6). M9 redéfinie (mutation du squash), M12 retirée ; 80 requêtes
    dans l'instantané, non plus 81. Les conséquences de l'échec fermé sur l'import (R2-2) disparaissent avec lui.
    C-28-3-25.
  - **F2-1 (MEDIUM)** — repli `INNODB_TRX` retiré (privilège `PROCESS`) ; `@@in_transaction` prescrit, son existence
    vérifiée par le test lui-même ; à défaut, repli sans privilège `SET TRANSACTION READ WRITE` → `1568` dans une
    transaction ouverte. C-28-3-26.
  - **F2-2 (MEDIUM)** — preuve « rouge avant, vert après » par `git log -L` sur chacun des trois tests et de leurs
    helpers, sortie vide attendue ; forme à expressions `/fn <nom>(/,/^    }$/` et non `:<fonction>:` (pas de
    `.gitattributes` `diff=rust` : le pilote par défaut ne borne pas une fonction indentée) ; AC5 (a) et T2 disent
    que T2 ajoute des tests neufs sans toucher les trois tests ni leurs helpers. C-28-3-27.
  - **R2-3 (MEDIUM)** — la promesse du manuel bornée : une saisie concurrente ne peut plus laisser d'orphelin ; une
    base qui en porte déjà les garde, et l'export les reproduit (AC10, AC11). C-28-3-28.
  - LOW : R2-4=F2-3 **recompté** — 2 des 40 `CREATE TABLE` sans `ENGINE` (`contacts`, `contact_persons`), commande et
    sortie copiées aux Dev Notes ; les deux lentilles divergeaient (5 selon R, 2 selon F) : l'`awk` de R clôt une
    instruction au premier `;`, y compris dans un commentaire ou une chaîne (trois faux positifs nommés) ;
    R2-5=F2-4 M1 redéfinie (intérieur de T1, tuée par (a-défaut), (a-RC), (b)) ; R2-7 cœur prescrit
    `export_installation_in(&mut ReadSnapshot, obs)`, `finish` par l'appelant, couverture du chemin public par (e)
    et l'AC6 (C-28-3-29) ; R2-8 `backup.rs:488-489` ; F2-5 alias `BoxFuture<'a, T>` à deux paramètres,
    `Some("…".into())` pour le `Option<Cow<'static, str>>` (C-28-3-30) ; F2-6 alias du pool écrit en angle mort de
    l'AC2 (C-28-3-31) ; F2-7 `begin` = `acquire` + `begin_on`, `#[cfg_attr(not(test), allow(dead_code))]` sur le champ
    d'`Etape` (C-28-3-29) ; F2-8 cas `BeginFailed` nommé, « sur toute erreur » borné à l'instantané ouvert
    (C-28-3-30) ; F2-9 DDL `INSTANT` : « échoue, ou lit la structure neuve » (C-28-3-31) ; F2-11 « épinglée »
    → « signalée en commentaire », `Cargo.lock` seul (C-28-3-30). Registre séparé versé par union dans
    `epic-28-choix-autonomes.md` au rebase sur le kickoff (`5aad8b35`), puis supprimé. Décomptes après remédiation
    (recomptés) : 12 AC, 7 tâches (T0-T6), 11 mutations actives (M1-M11, M12 retirée), choix C-28-3-1 à 32.
- 2026-10-13 — **Validation P3** (Sonnet, lentilles R et F, contexte frais, lecture seule ; prompt
  `28-3-validate-prompt-p3.md`, rapports `kesh-gate-logs/28-3-validate-p3-{R,F}.md`) : **0 CRITICAL, 0 HIGH,
  0 MEDIUM, 15 LOW** (R : 7 LOW ; F : 8 LOW), 11 distincts après les convergences R3-1=F3-6, R3-2≈F3-4, R3-4≈F3-5,
  R3-5≈F3-3. Recyclages déclarés par les lentilles : R3-1, R3-2 (partiel), R3-5 (partiel), F3-2 (partiel), F3-3, F3-6 —
  tous de propagation de texte ; aucun ne touche une ligne de production prescrite. **Boucle close** (0 MEDIUM+).
  LOW appliqués à la clôture (Opus 5.5) : registre — annotations de C-28-3-9 (chaîne -17 → -25), C-28-3-19 (« épinglée »
  → signalée ; protocole texte des tests), C-28-3-20 (échec fermé borné à l'instantané), C-28-3-21 (alias à deux
  paramètres, annotation de l'observateur factorisé) ; AC10 — seule `contact_persons` est exposée (`contacts` est
  contrainte par ses clés étrangères entrantes), remède complet (`ALTER … ENGINE=InnoDB` puis contrôle par
  `SHOW CREATE TABLE` et recréation des deux clés), requêtes à base **nommée** avec témoin de compte (≥ 40),
  `_sqlx_migrations` expliquée, et T6 aligné ; AC5 — protocole texte des `SET` des tests, `CAST(… AS SIGNED)` pour
  `CONNECTION_ID()` et `@@in_transaction` (repli sur la seule erreur 1193), preuve `git log -L` complétée (`exit=$?`,
  témoin positif `T1~1..T2`, `git diff -U0` en ajouts purs), assertion de montage de (e) précisée, test neuf (h)
  (`KILL` puis `finish` en erreur) ; AC1 — propagation de l'erreur de `finish` par `export_installation` écrite en
  angle mort, plan B si (a-RC) rougit sur le code correct ; AC4 — observateur factorisé annoté, renvoi « Moteurs de
  stockage, au sol » ; mutations — M1/M6 par patch vérifié, M9 à motif unique de deux lignes. Décomptes
  (recomptés) : 12 AC, 7 tâches (T0-T6), 11 mutations actives (M1-M11), choix C-28-3-1 à 32 ; tests de l'AC5 : (a-défaut),
  (a-RC), (b), (c), (d1), (d2), (e), (f), (g)(i), (g)(ii), (h).
  Tendance de la boucle : P1 Sonnet 3 MEDIUM → P2 Opus 5 MEDIUM (signal D5, recyclage : remédiation par
  simplification, sans découpage) → P3 Sonnet 0 MEDIUM.
- 2026-10-13 — **Développement** (`bmad-dev-story`, Opus 5.5, autonomie) : T0-T6 faites. Commits `a914762b` (T1 —
  `ReadSnapshot` dégradé, trois tests rouges, sortie copiée), `1894bf0c` (T2 — l'instantané ; preuve `git log -L`,
  `exit=0`, témoin, `git diff -U0` en ajout pur), `c00a7f63` (T3-T4), `90c85099` (T5 — manuels, PDF, CHANGELOG, garde
  G18 16 → 17). 11 tests neufs (9 `#[sqlx::test]`, 2 `#[test]`). Mutations M1-M11 : **11/11 tuées**. Faits MariaDB
  de la fiche tranchés par les tests et mutations (neuf établis *[« dix » à l'origine, faux à sa date : le tableau
  porte neuf lignes « établi » — corrigé le 2026-10-13, revue de code P1, A-1]*, deux non vérifiés : le repli `1568`, inutilisé, et le
  comportement des clés étrangères hors InnoDB, que le manuel ne suppose pas). Choix C-28-3-33 à 39. Gates au dernier
  commit de code : backend 3525/3525 (6 ignorés), Vitest 1395/1395, svelte-check 0 erreur, build OK, E2E 265 passés +
  9 échecs de la baseline (7 KF-029, 2 KF-045 matinaux). Statut `review` ; prompt de la revue P1
  `28-3-review-prompt-p1.md` (Sonnet, B/E/A).
- 2026-10-13 — **Revue de code** (`bmad-code-review`, boucle close à la P1 ; remédiation Opus 5.5, autonomie).
  Tendance : **P1 (Sonnet, lentilles B, E, A, contexte frais chacune, lecture seule) : 0 CRITICAL, 0 HIGH, 0 MEDIUM,
  13 LOW bruts** (A 5, B 3, E 5 — recomptés dans les rapports `kesh-gate-logs/28-3-review-p1-{A,B,E}.md`), **9
  distincts** (A-1 ; A-2 ; A-3 ; A-4 = E2 ; A-5 = E1 = L1 ; E3 ; E4 ; E5 = L2 ; L3). Boucle close sans P2 : aucun
  finding au-dessus de LOW, et la remédiation ne touche **aucune ligne de production** (`git diff --stat 90c85099..HEAD
  -- crates/*/src` : `backup.rs` +71 et `export.rs` +7, toutes dans `mod tests`). Remédiation, deux commits :
  `6940b560` (tests) et `629a3b1e` (fiche, registre, manuels, PDF). Ce qui a été fait :
  - A-4/E2 → test neuf `echec_du_set_ferme_la_connexion` (AC5 (j), C-28-3-40) ; le refus `1568` d'un
    `SET TRANSACTION ISOLATION LEVEL` dans une transaction ouverte est désormais **établi** (la forme `READ WRITE` du
    repli de (d) reste non vérifiée, n'ayant pas servi) ;
  - E1/L1/A-5 et L3 → sous-chaînes exigées par les gardes (g)(i) et (g)(ii) ; AC1 reformulée sans « inatteignable »
    (C-28-3-41) ;
  - E3 → (e) seede une société et asserte `min_required` et `instance_id` (C-28-3-42) ;
  - E4 → phrase sur les lignes orphelines avant `ADD CONSTRAINT`, au seul manuel qui porte le remède (admin), sans
    requête de repérage (G18, C-28-3-43) ; E5/L2 → `(issue \#465)` déplacé après « l'une après l'autre » (manuel
    utilisateur) ; PDF régénérés (`make fr`) et contrôlés aplatis (`28-3-p1-remediation-{admin,user}.flat.txt`) ;
  - A-1 et A-3 → annotations datées dans l'entrée « Développement », la table des faits et C-28-3-38 (les entrées
    passées ne sont pas réécrites) ; A-2 → plage du Status portée à C-28-3-43 (dernier choix au registre).
  Décomptes (périmètre `90c85099..629a3b1e`) : **1 test neuf** (`#[sqlx::test]`), 2 tests renforcés ((e), (g)(i)) et 1
  garde renforcée ((g)(ii)) ; **mutations P1-MA à P1-ME : 5/5 tuées**, chacune par l'assertion neuve
  (`kesh-gate-logs/28-3-p1-remediation-mutations.log`) ; (h) rejoué **20/20** (`28-3-h-20-executions.log`) ; choix
  **C-28-3-40 à 43** (4). Rebase : `origin/main` toujours au kickoff `5aad8b35`, base de la branche — sans objet.
  **Gates complets** sur `629a3b1e` (bases `kesh_283` et `kesh_e2e_283` supprimées, recréées, migrées — 80
  migrations — et seedées juste avant, conteneur non redémarré) : backend `scripts/test-fast.sh` — fmt, clippy
  `-D warnings`, nextest **3526 exécutés, 3526 passés, 6 ignorés** (`Summary [153.619s]`,
  `28-3-final-gate-backend.log`) ; frontend — svelte-check 0 erreur / 29 avertissements / 5 fichiers,
  lint-i18n-ownership PASS, Vitest **1395/1395** (134 fichiers), build OK (`28-3-final-gate-frontend.log`) ; E2E
  complet, port 3036, `/health` `smtpConfigured:true`, 11:56–12:22 UTC : **267 passés, 7 échecs, 19 ignorés**
  (`28-3-final-e2e-run.out`) — les sept échecs sont les sept KF-029 de `docs/testing.md` § « Les échecs attendus »
  (`mode-expert:26`, `:41`, `onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`, `:150`), aucun huitième ; run
  après 12:00 UTC pour l'essentiel, d'où l'absence des KF-045 ; 0 `Permission denied` au journal backend. Statut `done`.
