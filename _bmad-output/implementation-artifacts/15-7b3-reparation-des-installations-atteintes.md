# Story 15.7b3 : Les installations déjà atteintes par #528 et #542 sont réparées, au démarrage et à la restauration

Status: done

<!-- Née le 2026-10-08 du découpage de la 15-7b2 (choix C-15-7-40), à la passe de validation P3 de
     celle-ci : la clause de coupe de la 15-7b2 (C-15-7-39) s'est déclenchée — deux MEDIUM nés de la
     remédiation P2 (F3-1, F3-2) après une P2 à MEDIUM —, et la coupe s'est faite **selon la cause** :
     les deux MEDIUM portent sur la réparation des installations déjà atteintes, que la P2 avait ajoutée
     (AC 14 de la 15-7b2, C-15-7-38). La 15-7b2 garde la prévention (la remise à zéro n'orpheline plus
     personne, le démarrage n'ajoute plus de société provisoire) ; cette fiche porte la réparation.
     Choix applicables de `epic-15-choix-autonomes.md` : C-15-7-38 (réparation au démarrage — sa
     conception est reprise ici), C-15-7-40 (découpage), C-15-7-41 (clés orphelines révoquées — **révisé**
     par C-15-7-45 : révoquées **puis** repointées), C-15-7-42 (LOW de la P3 ; rejeu), C-15-7-44 (fonction
     partagée, statut — **révisé** par C-15-7-45 et C-15-7-47 pour le nom et l'extraction), C-15-7-45 (une
     règle et une fonction pour les principaux orphelins, posée par la 15-7b2), C-15-7-46 (réparation de
     #542 au démarrage, venue de la 15-7b2), C-15-7-47 (démarrage non bloquant, nom de la fonction et de
     l'action, installation sans utilisateur), C-15-7-49 (détail des clés, #546, sessions, LOW).
     Version d'origine de l'AC 14 et du test 15 : fiche 15-7b2 au commit `ffcdcf8d`.
     Validation : P1 appliquée (1 HIGH, 6 MEDIUM, 12 LOW distincts) ; P2 appliquée (4 MEDIUM, 11 LOW
     distincts ; C-15-7-50) ; P3 appliquée (2 MEDIUM, 13 LOW distincts ; C-15-7-53) ; P4 ciblée à
     lancer. -->

**Issues** : **`closes #528`**, **`closes #542`** — **sur la PR**, pas sur les commits intermédiaires
(`refs #N`) : le dépôt merge en squash. La 15-7b2 en ferme les **causes** (`refs #528`, `refs #542`) ;
cette fiche répare les installations que ces causes ont déjà atteintes, et c'est elle qui clôt les deux
issues. **`refs #546`** (lignes de démonstration orphelines, non nettoyées ici — limite écrite).

**Dépendances** : **la 15-7b2, mergée** (et donc les 15-7b1, 15-7a2, 15-7a1, ainsi que la **15-11**
`15-11-configuration-transmise`, que la 15-7b2 exige mergée avant elle — C-15-7-51 ; R3-8 de la P3) — elle pose
**`companies::reattach_orphan_principals_in_tx`** (C-15-7-45), la fonction unique qui traite les
principaux orphelins (utilisateurs rattachés, clés d'API actives orphelines révoquées puis repointées),
**que cette fiche appelle et ne réécrit pas** ; ainsi que `companies::insert_stub`, les constantes du stub
descendues dans `kesh-db`, et la garde `company_count == 0` du bootstrap (#542). ⚠️ **Même release
visée** (v0.13.0) que la 15-7b2, afin que #528 et #542 se ferment dans la release qui en ferme les
causes ; ce n'est **pas** une contrainte de sûreté : entre les deux, une installation atteinte reste
dans l'état d'aujourd'hui (routes en erreur, `seed-demo` en 500 sur plusieurs sociétés provisoires),
sans trou de piste ni effacement (C-15-7-44).
⚠️ **Renvois de ligne** : ils valent sur la base `b74c3dac` (revérifiés à `cf40085f`) ; la 15-7b2
réécrit `auth/bootstrap.rs` (AC 13, `insert_stub`), **mais aussi** `docs/manual/fr/admin-manual.tex`
(`:1250-1259`, `:1314`, les dix tableaux de `sec:env-vars`, mis en page par la 15-11a, AC12 (j)) et le CHANGELOG : les renvois au
manuel (`:1314`, `:1637-1650`, `:1704-1717`, `:1734-1749`, `:1793`, `:1904-1906`, `:2054-2063`) se
décaleront. À relocaliser **par symbole** dans le code, **par titre de section** dans le manuel, au
développement (R3-7 de la P3).

## Story

En tant qu'**administrateur d'une installation déjà touchée par #528 ou #542** (une remise à zéro d'une
version antérieure a effacé la société sans rattacher personne ; ou des redémarrages avant la création
de l'administrateur ont ajouté des sociétés provisoires),
je veux que **Kesh répare l'installation sans que j'aie à le demander**, au démarrage de la nouvelle
version comme à la restauration d'une sauvegarde de cette installation,
afin que **les écrans de la société cessent d'échouer**, sans qu'**un accès créé pendant une
démonstration devienne pour autant un accès vivant à ma comptabilité**.

## Décompte de modules

À la granularité de C-15-7-8 : `kesh-db/repositories/companies` (fonction de réparation, suppression des
sociétés provisoires superflues), `kesh-api/auth/bootstrap` (appel au démarrage), `kesh-api/routes/admin`
(appel dans la transaction de restauration), `kesh-api/audit_labels`, `kesh-i18n` — **cinq modules**,
plus les manuels, `docs/api-external.md` et le CHANGELOG. Recompté après la P1 : la réparation de #542
(C-15-7-46) vit dans `companies`, déjà compté ; la règle des principaux orphelins est **appelée**, posée par
la 15-7b2 ; `kesh-db/repositories/api_keys` n'est pas modifié. Recompté après la P2 : la lecture des
références de `companies` dans `information_schema` (AC 1, étape 2 — C-15-7-50) vit dans `companies`,
déjà compté ; le test de schéma (test 6) vit dans `crates/kesh-db/tests/companies_repository.rs`, déjà
touché par le test 3. Le seuil (« plus de cinq ») n'est **pas** franchi : aucune dérogation à écrire.

## Ce que l'inventaire a établi

1. **L'état à réparer, #528** (F-1 de la P2 de la 15-7b2) : le parcours démonstration → remise à zéro
   (v0.12.x) → `language` (nouvelle société, personne rattaché) → production → `finalize` **passe** —
   aucune route d'onboarding ne lit la société de l'utilisateur (`get_company`, `MIN(id)`) —, et laisse
   une installation **finalisée**, **une** société, `users.company_id` et `api_keys.company_id`
   **morts**. Ni la remise à zéro (refusée à l'étape ≥ 7) ni `language` (plus appelée) ne la réparent.
   Toute installation touchée redémarre pour passer à la version qui porte cette fiche. Une installation
   restée **sans** société (remise à zéro v0.12.x, langue pas encore choisie) est réparée par `language`
   ou la remise à zéro de la 15-7b2, qui appellent la même fonction (C-15-7-45) ; ses clés actives
   orphelines sont révoquées **dès le démarrage** (AC 1), sans attendre. ⚠️ Ce recours suppose
   l'installation **restée à une étape d'onboarding où `language` ou la remise à zéro sont atteignables**
   (F3-2 de la P3) : une archive **sans société** (prise entre une remise à zéro v0.12.x et le choix de la
   langue) **restaurée sur une installation déjà onboardée** garde l'étape de la destination
   (`onboarding_state` n'est pas restaurée ; `force_onboarding_done_if_eligible`, `backup.rs:587-614`,
   ne force rien sans société) — `language` n'est plus appelée, la remise à zéro est refusée à l'étape
   ≥ 7, le démarrage n'a pas de société cible : les utilisateurs restent orphelins **sans aucun chemin de
   rattachement** (les clés actives, elles, sont révoquées par la restauration même). **Limite
   assumée**, sans mécanisme neuf (AC 2, « ne fait pas ») : l'archive doit avoir été exportée pendant
   l'onboarding, hors du parcours normal ; le recours est d'importer une autre archive.
2. **Une clé d'API orpheline est inerte aujourd'hui** (F3-1 de la P3 de la 15-7b2) :
   `find_active_auth_by_key_hash` (`repositories/api_keys.rs:117-141`) rend le `company_id` de la clé,
   et `get_company_for` (`helpers.rs:61-72`) répond 500 sur un id mort. La repointer **sans la révoquer**
   la réactiverait en silence, alors qu'elle a pu être créée pendant une démonstration (instance
   « sandbox, training », `.env.example:273-279`) — et que, contrairement aux utilisateurs, aucun
   administrateur ne sait qu'elle existe. La laisser révoquée **sans** la repointer laisserait une clé
   étrangère pendante (`fk_api_keys_company`) et la clé invisible sur la page des clés, qui filtre par
   société (`list_by_company`, `:79-98`). D'où : révoquée **puis** repointée (C-15-7-45).
3. **Le mécanisme de révocation existe** : `api_keys::revoke_in_tx(tx, company_id, id, version)`
   (`repositories/api_keys.rs:149-171`) — `revoked_at = NOW(3)`, `version + 1`, filtré `id`,
   `company_id`, `version`, `revoked_at IS NULL` ; `rows_affected == 0` ⇒ `OptimisticLockConflict`.
   Une clé révoquée n'authentifie plus (`find_active_auth_by_key_hash` filtre `revoked_at IS NULL`,
   `:133`). Il est employé par la fonction de la 15-7b2, pas directement ici.
4. **La restauration d'une sauvegarde rouvre l'état** (F3-2 de la P3 de la 15-7b2) : sauvegarder une
   installation atteinte (`.keshbackup`) puis la restaurer — c'est la voie de migration documentée —
   ramène `users.company_id` mort, sans redémarrage à la suite. Inventaire des écrivains de `companies`
   et de `users.company_id` : `auth/bootstrap.rs` (insertion du stub), `routes/onboarding.rs` (branche
   « aucune société », couverte par la 15-7b2), `kesh_seed::reset_demo` (couverte par la 15-7b2), et
   l'**import d'installation** — `routes/admin.rs`, `run_backup_and_restore` (`:221-469`), qui vide puis
   réinsère toutes les tables par `kesh_db::backup::restore_tables_in_tx` (`:288-291`) — **seul site non
   résolu**, couvert ici. (Les `INSERT INTO companies` de `kesh-db/src/backup.rs` — `:674`, `:791`,
   `:951` — sont dans `mod tests`, `:617`.)
5. **La transaction de restauration n'est pas sous `retry_with`** : `run_backup_and_restore` ouvre
   `state.pool.begin()` (`:232-236`), prend `_kesh_version … FOR UPDATE` (`:237`), et rend toute erreur
   en `AppError::AdminFullImportFailed` ; une erreur annule l'import entier (la sauvegarde pré-import
   est déjà écrite sur disque, `:259-261`), l'administrateur le relance. Après `restore_tables_in_tx`,
   la transaction tient déjà en exclusif **toutes** les lignes de `companies`, `users` et `api_keys`
   (vidées puis réinsérées) : la réparation n'y ajoute aucun verrou neuf sur une ligne qu'une autre
   transaction pourrait tenir.
6. **Le démarrage n'a pas de trafic concurrent** : `ensure_admin_user` (`main.rs:181`) tourne avant
   `TcpListener::bind` (`main.rs:369`), et une erreur qu'il **rend** arrête le processus
   (`std::process::exit(1)`, `:185`). Deux instances démarrant sur la même base — cas que le module dit
   tolérer (`auth/bootstrap.rs:4-6`) — se **sérialisent** sur le verrou `companies … FOR UPDATE` de
   l'étape 1 : la seconde attend le `commit` de la première, puis ne trouve plus rien à réparer (F7 de la
   P1). **Angle mort assumé** (F-3 de la P2, reformulé par R3-6 de la P3) : à **deux** instances, il
   n'existe pas — la suppression exige plus d'une société visible (AC 1, étape 2) et chaque instance
   répare **avant** d'insérer son propre stub (AC 2) : la seconde voit au plus le stub de la première,
   rien à supprimer. L'angle mort réel exige **deux stubs déjà nés** de la course sur la garde
   `company_count == 0` (deux instances simultanées sur base vide), puis un **redémarrage** ou une
   **troisième** instance dont la réparation supprime le stub qu'une instance encore en vol, n'ayant pas
   créé son administrateur, s'apprête à désigner (1452 à l'insertion de l'utilisateur, non rattrapé ⇒
   `exit(1)`). Kesh tourne en **une** instance (`docker-compose.yml`) ; un redémarrage lève l'état.
   Écrit, non traité.
7. **L'import ne déconnecte que celui qui l'a lancé** (F5 de la P1 — la rédaction précédente disait
   « tout le monde », c'était faux) : le serveur rend `"sessionInvalidated": true` et c'est **le client**
   de l'importateur qui le redirige vers la connexion (`frontend/src/lib/features/admin-restore/admin-restore.api.ts:6`).
   Le JWT est sans état (`middleware/auth.rs:64-72`) et porte le `company_id` lu au login ou au refresh
   (`routes/auth.rs:247`, `:437`, `:539`) : les sessions ouvertes **dans d'autres navigateurs** gardent
   l'id d'avant l'import jusqu'à l'expiration du jeton (`KESH_JWT_EXPIRY_MINUTES`, défaut 15 min, plus
   60 s de tolérance), et `refresh_tokens` est elle-même restaurée. Même péremption qu'au démarrage
   (15-7b2, AC 4, « Limite »).
8. **#542, l'état à réparer** (F4-3 de la P4 de la 15-7b2, C-15-7-46) : sans utilisateur ni variable
   d'administrateur, le bootstrap v0.12.x insérait une société provisoire **à chaque démarrage** ; la
   15-7b2 ferme la cause (garde `company_count == 0`). Une installation déjà touchée porte plusieurs
   sociétés `is_stub = TRUE` ; `/setup` rattache l'administrateur à `MIN(id)` (`routes/setup.rs:130`), et
   tous les sites mono-société lisent `MIN(id)` — les autres sont superflues, sans utilisateur ni clé.
   Qu'elles n'aient reçu **aucune** donnée n'est pas supposé mais **vérifié** (F-2 de la P2) : une société
   provisoire peut porter un plan comptable et des exercices en cours d'onboarding
   (`crates/kesh-db/tests/company_invoice_settings_repository.rs:745-746`), et **29** clés étrangères
   désignent `companies` (squash), dont **quatre** en `ON DELETE CASCADE` (`users`, `bank_profiles`,
   `contact_persons`, `email_templates`) : un `DELETE` les effacerait **en silence**, sans 1451. D'où la
   garde de l'AC 1, étape 2, sur **toutes** les références. `seed_demo` y rend 500 (`len() != 1`,
   `kesh-seed/src/lib.rs:105-111`) : le choix « Explorer » échoue. Aucun écran n'atteint la remise à zéro
   sur cet état (le bouton vit dans le bandeau de démonstration, F4-3) : la réparation ne peut être
   qu'automatique, au démarrage.
9. **Les autres lignes orphelines de #279 restent** (F4 de la P1) : la remise à zéro v0.12.x
   (`kesh-seed/src/lib.rs:244-283`) n'effaçait que huit tables ; les lignes de démonstration de
   `vat_rates`, `company_invoice_settings`, contacts, produits, factures, avoirs gardent un `company_id`
   mort. Elles sont inertes à la lecture (requêtes scopées), mais pendantes. **Non nettoyées ici** :
   **#546** (ouverte par l'orchestrateur). Limite écrite au manuel (AC 6).
10. **Aucun changement de schéma** : ni migration, ni P1–P8.

## Acceptance Criteria

**1. Une fonction de réparation, partagée par les deux appelants** (C-15-7-44, C-15-7-47). Dans
`kesh-db/src/repositories/companies.rs` :

```rust
pub enum RepairTrigger { Startup, Restore { actor_user_id: i64, triggered_by_user: i64 } }
pub struct InstallationRepair {
    pub company_id: Option<i64>,          // la société de rattachement, None si zéro ou plusieurs
    pub stub_companies_removed: Vec<i64>,  // #542, démarrage seulement
    pub principals: OrphanPrincipals,      // rendu par reattach_orphan_principals_in_tx (15-7b2)
}
pub async fn repair_installation_in_tx(
    tx: &mut Transaction<'_, MySql>,
    trigger: RepairTrigger,
) -> Result<Option<InstallationRepair>, DbError>
```

Dans la transaction de l'appelant, **dans cet ordre** :
  1. `SELECT id, is_stub FROM companies ORDER BY id FOR UPDATE` (Pattern 5 : `companies` d'abord).
  2. **Démarrage seulement** (`RepairTrigger::Startup`), s'il y a **plus d'une** société : supprime les
     **sociétés provisoires superflues** (#542, C-15-7-46, C-15-7-50) — `is_stub = TRUE` et **aucune
     ligne d'aucune table** ne la désigne. La liste des colonnes qui désignent `companies(id)` n'est
     **pas recopiée** : elle est **lue à l'exécution**, par une fonction publique
     `companies::company_referencing_columns(conn: &mut MySqlConnection) -> Result<Vec<(String, String)>, DbError>`
     (table, colonne) — appelée par `repair_installation_in_tx` avec `&mut **tx`, et par le test 6 (a)
     avec `&mut *pool.acquire().await?` (R3-5 de la P3) —, dans `information_schema.KEY_COLUMN_USAGE` — `TABLE_SCHEMA = DATABASE()`,
     `REFERENCED_TABLE_SCHEMA = DATABASE()`, `REFERENCED_TABLE_NAME = 'companies'`,
     `REFERENCED_COLUMN_NAME = 'id'` (patron : colonnes du stub lues par `information_schema`, C-15-7-32
     de la 15-7b2). Elle couvre ainsi les tables en `ON DELETE CASCADE` (inventaire § 8) et toute table
     future ; `users` et `api_keys` y sont comme les autres. Pour chaque candidate, une requête
     `SELECT EXISTS(SELECT 1 FROM \`t\` WHERE \`c\` = ?)` par colonne (identifiants issus du schéma,
     jamais d'une entrée, entre accents graves). `audit_log.company_id`, pointeur logique **sans** clé
     étrangère (squash `:103`), n'y figure pas et n'empêche rien : une société sans utilisateur n'a pas
     d'entrée signée par l'un des siens. Si **toutes** les sociétés sont superflues, la plus petite
     (`MIN(id)`, celle que `routes/setup.rs:130` rattache) est conservée. **Chaque suppression sous un
     `SAVEPOINT`** (`SAVEPOINT repair_stub` ; `DELETE FROM companies WHERE id = ?` ;
     `RELEASE SAVEPOINT`) : une erreur sur une société ⇒ `ROLLBACK TO SAVEPOINT`, la société **reste en
     place**, `tracing::warn!` (« société provisoire {id} conservée : {e} »), et la réparation **continue** — les autres sociétés et
     les principaux orphelins (étape 5) sont traités quand même ; l'id n'entre pas dans
     `stub_companies_removed`. Si le `ROLLBACK TO SAVEPOINT` échoue lui-même (un 1213, ou un 1205 sous
     `innodb_rollback_on_timeout = ON`, annule la transaction entière et le point de sauvegarde avec
     elle : le `ROLLBACK TO` répond 1305, et la session est alors **hors transaction** — continuer
     écrirait en autocommit), la réparation s'arrête et rend **l'erreur d'origine, celle du `DELETE`**
     (le 1213), **jamais** celle du `ROLLBACK TO` (un 1305 masquerait la cause) ; l'erreur du
     `ROLLBACK TO` est journalisée en `warn!` — même règle que le `rollback` de l'AC 2 (F3-6 de la P3).
     Une erreur du `SAVEPOINT` ou du `RELEASE SAVEPOINT` est **rendue** telle quelle (R3-4). L'erreur
     rendue suit l'AC 2 (démarrage) ou l'AC 3 (restauration). **Forme d'émission** : celle du précédent
     du dépôt, `accept_batch` de la réconciliation (`crates/kesh-api/src/routes/reconciliation.rs:1026-1072`,
     `sqlx::query(&format!("SAVEPOINT {sp}"))`, `RELEASE`, `ROLLBACK TO`, exercés par ses tests
     d'intégration), ou `sqlx::raw_sql` — nom de point de sauvegarde **constant** (`repair_stub`), jamais
     dérivé d'une entrée. Le prédicat `is_savepoint_lost` du précédent (`:1092-1096`, code 1305, privé à
     `kesh-api`) **n'est pas nécessaire ici** : toute erreur du `ROLLBACK TO` arrête la réparation, sans
     distinguer le 1305 — rien à dupliquer ni à descendre dans `kesh-db` (R3-4 de la P3).
     **Pas à la restauration** : une archive est l'état que l'administrateur a choisi ; le démarrage
     suivant le fait (limite écrite).
  3. Société de rattachement : **exactement une** société restante ⇒ `Some(id)` ; **zéro** ou
     **plusieurs** ⇒ `None` (aucun rattachement ne serait sûr).
  4. **Installation sans utilisateur** (`SELECT COUNT(*) FROM users` = 0 — R1-4 de la P1) : rien de plus
     n'est fait ; si l'étape 2 a supprimé des sociétés, `info!` au journal du serveur **émis par la
     fonction elle-même** (elle rend `None`, l'appelant n'a rien à journaliser — R2-8 de la P2), **sans
     entrée d'audit** (aucun utilisateur pour la signer ; installation jamais configurée) — `Ok(None)`.
     Aucune clé d'API ne peut exister sur une base sans utilisateur : `fk_api_keys_created_by`
     (`REFERENCES users`, squash `:84`) interdit une clé sans son créateur (R2-7). Cela remplace le `DbError::Invariant` de la rédaction précédente, qui aurait signalé une
     erreur à chaque démarrage sur un état déjà inerte.
  5. Appelle **`companies::reattach_orphan_principals_in_tx(tx, target)`** (15-7b2, C-15-7-45) avec la
     société de l'étape 3 : les clés d'API **actives** orphelines sont **révoquées quel que soit le
     nombre de sociétés**, puis, s'il y a une société de rattachement, utilisateurs orphelins rattachés
     et **toutes** les clés orphelines (dont celles qui viennent d'être révoquées) repointées vers elle.
     La règle n'est **pas** réécrite ici.
  6. Rien de fait (aucune société supprimée, `principals.is_empty()`) ⇒ `Ok(None)`, rien d'écrit — c'est
     le cas de toute installation saine, à chaque démarrage. ⚠️ `None` a donc **deux** sens (R3-3 de la
     P3) : « rien fait » (ici) **ou** « sociétés provisoires supprimées sur une base sans utilisateur »
     (étape 4) ; dans le second, la transaction **a écrit**, et c'est le `commit` de l'appelant qui rend
     les suppressions durables.
  7. Écrit **une** entrée **`installation.repaired`** (`entity_type = "installation"`,
     `entity_id = AUDIT_ENTITY_ID_NONE`) — `details = {"company_id" (null si aucune société de
     rattachement), "stub_companies_removed": [ids], "users_repointed", "user_ids", "api_keys_revoked":
     [{"id", "name", "created_by_user_id", "created_at", "last_used_at"}], "api_keys_repointed",
     "trigger"}` où `trigger` vaut `"startup"` ou `"restore"`, plus `"triggered_by_user"` pour `Restore`
     (C-15-7-49 ; `api_keys` ne porte ni préfixe ni partie affichable du secret — seule l'empreinte
     `key_hash`, jamais écrite au journal) ; **acteur** : pour `Startup`, l'**administrateur actif** de
     plus petit `id` (`role = 'Admin' AND active = TRUE`), à défaut l'utilisateur de plus petit `id` ;
     pour `Restore`, `actor_user_id`, que l'appelant passe (AC 3). Son `company_id` (sous-SELECT sur
     `users`, `repositories/audit_log.rs:91-92`) est la société de rattachement quand il y en a une :
     l'entrée s'écrit **après** le rattachement. **Sans** société de rattachement, il désigne la société
     de l'acteur — morte sur une installation sans société : l'entrée existe en base mais le journal, qui
     filtre par société (`audit_log.rs:204-219`), ne l'affiche pas (limite écrite au manuel, AC 6).
  8. `Ok(Some(InstallationRepair { … }))`. **Ni `begin` ni `commit`** : c'est l'appelant.

⚠️ **L'acteur du démarrage n'a rien fait** (F3-3 de la P3 de la 15-7b2) : il n'y a ni `CurrentUser`, ni
jeton, ni acteur « système » (`ActorType` ne connaît que `User` et `ApiKey`). **Choix écrit** (C-15-7-42) :
l'entrée est signée par le plus ancien administrateur actif, et `details.trigger = "startup"` dit qu'elle
est le fait du démarrage ; le doc-comment de la fonction et le manuel (AC 6) le disent en toutes lettres,
sur le patron de `books.restored` (`routes/admin.rs:366-373`, « un administrateur qui n'a rien
déverrouillé »).

**2. Au démarrage** (F-1 de la P2 de la 15-7b2, C-15-7-38, C-15-7-47). Dans `ensure_admin_user`
(`auth/bootstrap.rs`), **en tête**, **avant la lecture des compteurs** `company_count` et `user_count`
(`auth/bootstrap.rs:51-60`, « lecture unique … partagée par toutes les branches ») et donc avant la
répartition par cas (R2-3 de la P2 = F-3) : `pool.begin()`, `repair_installation_in_tx(&mut tx,
RepairTrigger::Startup)`, puis `commit` **dans tous les cas** (sur `None` aussi : la transaction a pu
supprimer des sociétés provisoires d'une base sans utilisateur, AC 1 étape 4 — R3-3 de la P3). Les compteurs
reflètent ainsi l'état **réparé**. **Invariant**, à écrire au doc-comment : la réparation n'insère ni ne
supprime aucune ligne de `users`, et ne ramène jamais le nombre de sociétés à zéro (la plus petite est
conservée, AC 1 étape 2) ; la garde `company_count == 0` du cas 2 (15-7b2) et la relecture
`ORDER BY id LIMIT 1` du cas `(0, true)` prennent donc les mêmes décisions qu'avant la réparation. Sur
`Some`, **l'appelant** journalise le rapport par un `info!` ; sur `None` après suppression de sociétés
sans utilisateur, c'est la fonction (AC 1, étape 4 — R2-8). **Une erreur ne refuse pas le démarrage** (F6 de la P1) : `rollback`
*best-effort* (son erreur éventuelle journalisée en `warn!`, jamais substituée), **`tracing::error!`** avec
le détail (« réparation de l'installation au démarrage : {e} — démarrage poursuivi, installation
inchangée »), et `ensure_admin_user` **poursuit** sa répartition par cas comme aujourd'hui — l'installation
démarre dans l'état qu'elle avait (mode dégradé : écrans de la société en erreur, mais connexion et
export de sauvegarde disponibles, `full_export` ne lisant aucune société, `routes/admin.rs:32-67`). Motif :
entre la 15-7b2 et cette fiche, une installation atteinte vit déjà dans cet état sans trou ; une
réparation qui échoue ne doit pas la rendre **moins** utilisable qu'avant. **Pas de `retry_with`** (R3-8
de la P3 de la 15-7b2) : **angle mort assumé** — le démarrage précède l'ouverture du port (inventaire
§ 6), aucune transaction HTTP ne peut interbloquer ; une seconde instance se sérialise sur l'étape 1
(sauf un redémarrage ou une troisième instance pendant qu'une instance née de la course sur base vide
n'a pas encore créé son administrateur, angle mort assumé, inventaire § 6) ; un
1205/1213 ne viendrait que d'un client SQL externe, et un redémarrage le lève (l'erreur est journalisée,
non fatale). **Cas laissés** : *aucune* société — les clés actives orphelines sont révoquées ici ; les
utilisateurs sont rattachés par `language` ou la remise à zéro (15-7b2), qui appellent la même fonction,
**tant que l'installation est à une étape où l'une d'elles est atteignable** ; une archive sans société
restaurée sur une installation **déjà onboardée** laisse ses utilisateurs orphelins **sans chemin de
rattachement** — limite assumée, sans mécanisme neuf (inventaire § 1, F3-2 de la P3) ;
*plusieurs* sociétés non superflues et des principaux orphelins — **atteint par aucun chemin connu**
(une remise à zéro v0.12.x efface toutes les sociétés, et #542 n'ajoute de stub que sans utilisateur) :
clés révoquées, utilisateurs non rattachés, écrit en limite. Le jeton déjà émis reste périmé jusqu'à son
renouvellement (15-7b2, AC 4, « Limite »).

**3. À la restauration d'une sauvegarde** (F3-2 de la P3 de la 15-7b2). Dans `run_backup_and_restore`
(`routes/admin.rs`), **après** le backfill `client_number_canonical` (`:323-344`, étape 5-ter) et la
lecture d'`audit_uid` (commentaire `:346-353`, requête `:354-358`, `:359-363`), **avant** l'entrée
`books.restored` (`:397`) et `admin.full_import` (`:446`) : `repair_installation_in_tx(&mut tx,
RepairTrigger::Restore { actor_user_id: audit_uid, triggered_by_user: current_user.user_id })`, erreur ⇒
`AppError::AdminFullImportFailed(format!("réparation de l'installation : {e}"))` — l'import est annulé en
entier, **inchangé** par rapport à toute autre erreur de l'import (C-15-7-47 : la règle non bloquante ne
vaut qu'au démarrage). L'acteur est celui d'`admin.full_import` (« `MIN(id)` des administrateurs du jeu
restauré », `:346-353`), pour la même raison. **Effet de bord, voulu** (F11 de la P1) : placée avant ces
deux entrées, la réparation fait que leur `company_id` (sous-SELECT sur `users`) désigne la société
vivante pour une archive atteinte — elles deviennent visibles au journal, alors qu'elles portaient l'id
mort. ⚠️ `admin.full_import` est écrite à **chaque** import ; `books.restored` **seulement si le verrou
de période recule** (`a_recule`, `routes/admin.rs:387-392`) — l'effet ne vaut pour elle que dans ce cas,
et le test 2 ne l'asserte pas (R2-2 de la P2 : même sous-SELECT, même mécanisme ; un montage de recul
du verrou n'apprendrait rien de plus). **Rejeu** : la route **n'est pas** sous `retry_with` (inventaire § 5) et cette fiche ne l'y met
pas — un interblocage annule l'import, que l'administrateur relance ; la réparation n'ajoute aucun
verrou neuf (inventaire § 5). Le corps de la réponse HTTP de l'import est **inchangé** ; le rapport vit
au journal d'audit, comme celui des backfills (`:420-424`). La suppression des sociétés provisoires
superflues ne s'y fait pas (AC 1, étape 2).

**4. Ce qu'aucune clé ne devient** (C-15-7-41, révisé par C-15-7-45) : après la réparation, **aucune**
clé d'API dont la société était effacée n'authentifie — ni au démarrage, ni après une restauration, **ni
après un `language` ou une remise à zéro ultérieurs** (la fonction de la 15-7b2 révoque avant tout
repointage, sur tous les chemins). Une clé révoquée et repointée **paraît** sur la page des clés de la
société (`list_by_company`, `include_revoked = true`, page de gestion), avec son nom et le statut
« révoquée » ; sans société de rattachement, elle reste révoquée au `company_id` mort et paraîtra à son
repointage par `language` ou la remise à zéro. Le détail de l'entrée d'audit nomme chaque clé (AC 1,
étape 7).

**5. Une action neuve, déclarée et libellée dans les quatre catalogues** (`ACTIONS` triée, quatre `.ftl`
près de `audit-log-action-installation-reset`, posée par la 15-7b2). Le libellé est **neutre** (R1-9 de
la P1) : l'entrée peut porter un rattachement, des clés révoquées, des sociétés supprimées, ou une seule
de ces trois choses (C-15-7-47) :

| Action | Clé | fr-CH | de-CH | it-CH | en-CH |
|---|---|---|---|---|---|
| `installation.repaired` | `audit-log-action-installation-repaired` | Réparation de l'installation | Reparatur der Installation | Riparazione dell'installazione | Installation repaired |

Aucune route neuve : le registre des routes (`audit_route_registry.rs`) est inchangé. La garde qui verra
le littéral de l'action écrit dans `kesh-db` est **`crates/kesh-api/tests/audit_label_registry.rs`** (elle
balaie tous les `crates/*/src`, `:151-160`, forme `NewAuditLogEntry::user(`) ; `audit_log_e2e.rs:531`
(`actions.len() == ACTIONS.len()`) suit sans changement (F12 de la P1).

**6. Les doc-comments, les manuels, la documentation d'API et le CHANGELOG disent la réparation** —
PDF régénérés (`make fr`) et contrôlés aplatis (`pdftotext … | tr '\n' ' ' | tr -s ' '`) :

| Site | Après la 15-7b2 | Après la 15-7b3 |
|---|---|---|
| `admin-manual.tex:1314` (`\item \textbf{Parcours}`) | la réinitialisation et sa condition (`KESH_PRODUCTION_RESET`) ; installation touchée par \#528 réparée par la réinitialisation ou au choix de la langue (15-7b2) | ajoute **une phrase de renvoi** (F-1 de la P2) : une installation déjà touchée par \#528 ou \#542 est réparée au premier démarrage de la nouvelle version — voir § *Procédure de mise à jour standard* et § *Dépannage* |
| `admin-manual.tex:1704-1717` (§ *Procédure de mise à jour standard*, après l'étape 6 « Vérifier les logs ») | — | **le texte principal** (F-1 de la P2) : au **premier démarrage** de la nouvelle version, une installation déjà touchée par \#528 est réparée — les utilisateurs qui désignaient la société effacée sont rattachés à la société restante (s'il n'y en a qu'une), les clés d'API qui la désignaient sont **révoquées** puis rattachées à elle (elles paraissent, révoquées et sous leur nom, sur la page des clés : **à recréer si elles servent** — une clé créée en démonstration ne devient pas un accès à la comptabilité sans geste de l'administrateur) ; les **sociétés provisoires en trop** d'une installation touchée par \#542 — celles qu'aucune donnée ne désigne — sont supprimées au même moment, **y compris avant la création de l'administrateur** (sans entrée au journal dans ce cas : personne pour la signer — R2-1 de la P2) ; une société provisoire qui porte la moindre donnée est laissée en place ; l'opération s'inscrit au journal (« Réparation de l'installation »), **signée par le plus ancien administrateur actif bien que personne ne l'ait faite** — son détail porte `trigger : startup`, les clés révoquées (numéro, nom, créateur, dates de création et de dernière utilisation) et les sociétés supprimées ; sur une installation restée **sans** société, l'entrée n'est pas visible au journal (elle désigne la société effacée) ; la session ouverte doit être rouverte (reconnexion, ou au plus la durée de session plus une minute) — d'ici là, certains écrans répondent par une erreur, d'autres s'affichent vides ; si le journal du serveur porte « réparation de l'installation au démarrage » avec une erreur, voir § *Dépannage*. **Limite** : les données de démonstration d'une remise à zéro antérieure (taux, réglages de facturation, contacts, articles, factures, avoirs) qui désignent la société effacée restent en base, invisibles et inertes (\#546) |
| `admin-manual.tex:2054` (§ *Dépannage* ; sous-section neuve après « Kesh ne démarre pas », `:2056`) | — | « Réparation de l'installation en échec au démarrage » (F-1 de la P2) : **symptôme** — Kesh démarre, mais les écrans de la société restent en erreur, et `docker compose logs kesh-api` porte « réparation de l'installation au démarrage » suivi de l'erreur (le service s'appelle `kesh-api` dans `docker-compose.yml:30` et `docker-compose.prod.yml:54` ; `kesh` n'existe que dans `docker-compose.dev.yml` — R3-2 = F3-3 de la P3) ; **conduite** — Kesh démarre quand même, installation inchangée : exporter une sauvegarde (l'export fonctionne), relancer une fois (`docker compose up -d`, pas `restart`), et si le message revient, ouvrir un ticket en le joignant ; une société provisoire laissée en place porte un avertissement distinct (« société provisoire … conservée »), sans effet sur le reste de la réparation ; pour revenir en arrière, **renvoi** au § *Rollback en cas d'échec* (ligne suivante — F3-4 de la P3). **Au passage, même section** (R3-2 de la P3, décision de l'orchestrateur) : la sous-section voisine « Kesh ne démarre pas » nomme un service qui n'existe pas — `:2063` `docker compose logs kesh \| tail -50` devient `docker compose logs kesh-api \| tail -50`, et `:2058` « Le container `kesh` … montre `kesh` en `Restarting` » devient `kesh-api` (deux fois) ; de même au § *Logs applicatifs*, `:1905` et `:1906` (`docker compose logs --tail=100 kesh`, `--since=1h kesh`) deviennent `kesh-api` — relevé par `grep -n "compose logs" docs/manual/fr/admin-manual.tex` à `cf40085f` : **ces deux lignes sont fausses aussi**, contrairement à l'avis initial qui les tenait pour justes ; `:276`, `:579`, `:1712`, `:1904` (`kesh-api`) et `:560` (« Nom du projet : `kesh` », nom de projet Synology, non de service) sont justes et ne bougent pas |
| `admin-manual.tex:1734-1749` (§ *Rollback en cas d'échec*) | — | une phrase, après l'étape 3 (« si les migrations DB ont été appliquées… ») (F3-4 de la P3, qui déplace ici le « Retour arrière » de R2-11 de la P2) : revenir à l'image précédente **ne défait pas** la réparation de l'installation faite au premier démarrage — les clés révoquées restent révoquées (aucune réactivation possible), les sociétés provisoires supprimées ne reviennent pas ; seule la restauration de la sauvegarde d'avant la mise à jour les rend ; la réparation elle-même n'impose aucune version minimale — revenir à la version précédente n'est possible que si **aucune autre** migration de la release ne relève la version minimale exigée, à vérifier au CHANGELOG (phrase à confirmer à la préparation de la release) |
| `admin-manual.tex:1250-1259` (matrice du démarrage) | lignes « vide & non », « vide & oui » (15-7b2) | ligne « existant & non » : « No-op » devient « Réparation si nécessaire (\#528, \#542), sinon no-op » (F7 de la P1 : la matrice dit ce que fait le cas 3) ; sous le tableau, une phrase : à chaque démarrage, avant ces cas, Kesh répare une installation touchée par \#528 ou \#542 — rien sur une installation saine ; un échec est journalisé et n'empêche pas le démarrage |
| `admin-manual.tex`, § *Reprises de données rejouées à l'import* (`:1637-1650`) | — | un point : l'import d'une sauvegarde d'une installation touchée par \#528 fait la même réparation (sauf la suppression des sociétés provisoires, faite au démarrage suivant), **dans la même opération** (tout ou rien), signée comme l'import ; **celui qui importe** est déconnecté ; les **autres sessions ouvertes** (autres navigateurs, autres utilisateurs) gardent l'ancienne société jusqu'à l'expiration de leur jeton (au plus la durée de session plus une minute) — elles voient des écrans en erreur ou vides : se reconnecter (F5 de la P1). **Limite** (F3-2 de la P3) : une sauvegarde **sans société** (exportée pendant l'onboarding, après une remise à zéro d'une version antérieure) importée sur une installation déjà configurée laisse ses utilisateurs sans société, et rien ne les rattache ensuite — importer une autre sauvegarde |
| `admin-manual.tex:1793` (§ *Clés API (PAT)*, gestion et révocation) | — | une phrase : une clé peut avoir été **révoquée par Kesh** lors de la réparation d'une installation touchée par \#528 (entrée « Réparation de l'installation » du journal) ; elle paraît révoquée dans la liste ; en créer une neuve (F10 de la P1) |
| `docs/api-external.md` § 3 (« La liste affiche aussi … le statut ») | — | une phrase : après la mise à jour, une clé qui désignait une société effacée (\#528) est révoquée par Kesh et répond `401 UNAUTHENTICATED` ; elle paraît « révoquée » dans la liste ; en créer une neuve (F10 de la P1) |
| `user-manual.tex:297` (§ *Clés API (accès programmatique)*) | — | une phrase (F-4 de la P2) : une clé peut paraître « révoquée » sans que personne l'ait révoquée — Kesh l'a fait en réparant une installation touchée par \#528 ; elle n'authentifie plus : en créer une neuve (PDF `user-manual.pdf` régénéré) |

Les cellules tiennent sur une ligne dans cette fiche ; au manuel, le texte de la procédure de mise à jour
et celui du dépannage s'écrivent en plusieurs phrases, sans les renvois aux passes (R3-6 de la P3 de la
15-7b2).
Doc-comments : `ensure_admin_user` (`auth/bootstrap.rs`, doc de tête `:39-50`) dit la réparation, son
acteur et son caractère non bloquant ; la **matrice `//!` du module** (`auth/bootstrap.rs:8-17`, cas 3
« no-op ») et la phrase « tolérant aux race conditions » (`:4-6`) disent la réparation qui précède les
cas et la sérialisation par `companies` (F7 de la P1), ainsi que l'invariant des compteurs (AC 2) ; la
doc de tête de `main.rs` (`:19`, « 5. Bootstrap admin (`ensure_admin_user`) si la table users est
vide ») dit que l'étape répare aussi l'installation à **chaque** démarrage (R2-10 de la P2) ;
`run_backup_and_restore` reçoit un commentaire
d'étape (« 5-quater ») sur le patron des étapes 5-bis et 5-ter.
**CHANGELOG** : sous `## [0.13.0] — Non publié`, `### Corrigé` : une installation déjà touchée par #528
est réparée **au premier démarrage** de la nouvelle version, et à la **restauration** d'une sauvegarde
d'une telle installation — utilisateurs rattachés, clés d'API orphelines **révoquées** puis rattachées à
la société (visibles, révoquées, sur la page des clés ; à recréer) ; une installation touchée par #542
perd ses sociétés provisoires en trop au démarrage (« Explorer » ne répond plus 500) ; entrée
`installation.repaired` ; la session ouverte doit être rouverte ; un échec de la réparation est
journalisé et n'empêche pas le démarrage. Restent en base les données de démonstration orphelines d'une
remise à zéro antérieure (#546). ⚠️ La section `## [0.13.0] — Non publié` **existe sur `main`**
(ouverte par les 15-5a et 15-5b, `CHANGELOG.md:11` d'`origin/main`) mais pas encore sur la branche de
l'Epic 15-7 : l'entrée s'y **ajoute après rebase** sur `main`, sans recréer la section (F-5 de la P2) ;
la sous-section `### Corrigé` y figure à `origin/main` le 2026-10-08 (relu par `git show
origin/main:CHANGELOG.md`) — **la créer si elle est absente après rebase**, sous `### Modifié` (F3-8 de la
P3) ; conflit attendu au merge avec les entrées des fiches sœurs, à résoudre par ajout.
⚠️ **Greper la valeur, sans casse** (`grep -niE`) : `\b528\b`, `\b542\b`, `\b546\b`,
`installation.repaired`, `principals.reattached` (ne doit plus rien rendre hors historique), `révoqu`,
`rattach`, `Reprises de données`, `déconnect`, `provisoire`, `Dépannage`, `Rollback` ; et, pour le
nom du service (R3-2 de la P3), `grep -nE "compose (logs|ps).*\bkesh\b" docs/manual/fr/*.tex | grep -v kesh-api`,
qui doit ne plus rien rendre ; le tout sur `docs/manual/fr/*.tex`
(admin **et** utilisateur), les **deux** PDF aplatis,
`docs/api-external.md`, `README.md`, `website/`.

## Tasks / Subtasks

- [x] **T1 — `kesh-db`** (AC 1, 4) — `RepairTrigger`, `InstallationRepair`, `repair_installation_in_tx` (sociétés provisoires superflues au démarrage — références lues par `company_referencing_columns` dans `information_schema`, une suppression par `SAVEPOINT` —, société de rattachement, base sans utilisateur, appel de `reattach_orphan_principals_in_tx` de la 15-7b2, entrée) ; doc-comment (acteur du démarrage, entrée invisible sans société).
- [x] **T2 — Démarrage** (AC 2) — appel en tête d'`ensure_admin_user`, **avant la lecture des compteurs**, transaction, `info!` ; **erreur non fatale** (`rollback` *best-effort*, `error!`, démarrage poursuivi) ; doc de tête (invariant des compteurs), matrice `//!` du module, doc de tête de `main.rs:19`.
- [x] **T3 — Restauration** (AC 3) — appel dans `run_backup_and_restore`, étape « 5-quater », erreur ⇒ `AdminFullImportFailed`.
- [x] **T4 — Libellés** (AC 5) — `ACTIONS`, quatre `.ftl`.
- [x] **T5 — Tests** (Dev Notes § Tests) et mutations.
- [x] **T6 — Manuels, PDF, `docs/api-external.md`, CHANGELOG, doc-comments** (AC 6) — admin (renvoi `:1314`, procédure de mise à jour, dépannage — dont le nom du service `kesh-api` aux quatre lignes fausses, `:1905`, `:1906`, `:2058`, `:2063` —, rollback, matrice, reprises, clés) et utilisateur (`:297`) ; CHANGELOG après rebase sur `main`.
- [x] **T7 — Gates** : `kesh-db` touché ⇒ gate complet même en cours de boucle ; base remise à zéro avant ; E2E complet au dernier commit de code (aucun changement frontend attendu ; l'E2E reste la règle).

## Dev Notes

- **Pourquoi une fonction `kesh-db` et non deux appels `kesh-api`** : les deux appelants vivent dans deux
  modules de `kesh-api` (`auth/bootstrap`, `routes/admin`) ; la règle doit être **la même** aux deux
  endroits — et **la même** que celle de la remise à zéro et de `language` pour les principaux
  orphelins, d'où l'appel de `reattach_orphan_principals_in_tx` (15-7b2) au lieu d'une seconde copie
  (C-15-7-45). Précédents d'un helper `kesh-db` qui écrit sa propre entrée d'audit :
  `repositories/companies.rs:380` et `:508`, `repositories/reconciliation_rules.rs:215` (F12 de la P1 ;
  `onboarding::record_step_completed_in_tx`, cité auparavant, n'existera qu'avec la 15-7a2).
- **Contrôle de clé étrangère à la révocation** (F8 de la P1) : sans société de rattachement, la clé est
  révoquée (`revoked_at`, `version`) avec son `company_id` mort sous `FOREIGN_KEY_CHECKS = 1`. Les index
  d'`api_keys` sont `uq_api_keys_key_hash`, `fk_api_keys_created_by`, `idx_api_keys_company (company_id)`
  et `idx_api_keys_created` (squash) ; aucun ne contient `revoked_at` ni `version`. InnoDB ne recontrôle
  la clé étrangère d'une ligne enfant que si la colonne de la clé change — **attendu, non vérifié**
  contre MariaDB : les tests 1 (variante « aucune société ») et 3 (b') le prouvent (une erreur 1452 les
  ferait rougir). Si un index futur combinait `company_id` et `revoked_at` et que ces tests rougissaient,
  c'est ici qu'il faut regarder ; corroboration : `touch_last_used` (`api_keys.rs:182`) met déjà à jour
  des clés orphelines. Avec une société de rattachement, la clé est repointée à la suite : plus de
  référence pendante.
- **Idempotence** (R2-6 de la P2) : après une réparation, plus aucune clé **active** n'est orpheline ;
  les utilisateurs orphelins ne sont rattachés que s'il existe **une** société de rattachement (sans
  société, ou avec plusieurs sociétés non superflues, ils restent orphelins — AC 2, « Cas laissés ») ; les
  sociétés provisoires restantes sont désignées par une donnée ou conservées à dessein (`MIN(id)`, ou
  `SAVEPOINT` en échec). Le démarrage suivant n'écrit rien : la fonction de la 15-7b2 ne trouve plus de
  clé active orpheline, et une société conservée parce que référencée l'est encore (test 1).
- **Ce qui ne change pas** : les tests existants du bootstrap (`auth/bootstrap.rs`, `mod tests`) ne posent
  aucun `company_id` mort — vérifier par grep de `company_id` dans le fichier au développement, sortie
  collée au Dev Agent Record ; ceux de l'import (`admin_full_import_e2e.rs`, et aussi
  `admin_backup_e2e.rs`, `admin_pat_denied_e2e.rs` — R1-7 de la P1 : jeux sains, assertions par action,
  `admin_backup_e2e.rs:293-307`, `:487-497`) restaurent des jeux sains, donc sans entrée neuve —
  vérifier qu'aucun n'asserte une **séquence exacte** d'`audit_log` qui rougirait. Un test du bootstrap
  qui monterait plusieurs sociétés provisoires sans utilisateur verrait désormais les superflues
  supprimées : grep de `insert_stub`/`insert_stub_company` dans les tests au développement. **Les tests
  d'intégration qui appellent `ensure_admin_user`** (F3-5 de la P3) : `grep -rln ensure_admin_user
  crates/kesh-api/tests` en rend **quinze** à `cf40085f` (`auth_cookies_e2e`, `i18n_e2e`, `companies_e2e`,
  `onboarding_e2e`, `period_lock_e2e`, `opening_balances_e2e`, `rbac_e2e`, `users_e2e`,
  `onboarding_path_b_e2e`, `auth_e2e`, `spa_resilience`, `fiscal_years_e2e`,
  `journal_entry_reversal_e2e`, `profile_e2e`, `invoice_pdf_e2e`) ; la réparation s'y exécute désormais à
  chaque appel. Pour **chacun**, contrôler qu'aucun montage ne pose, **avant** l'appel, plus d'une
  société dont une provisoire sans référence, ni un principal au `company_id` mort (échantillon de la P3 :
  `auth_e2e.rs:177`, `fiscal_years_e2e.rs`, `opening_balances_e2e.rs`, `journal_entry_reversal_e2e.rs`
  créent leur seconde société **après** l'appel, non provisoire) ; sortie de la commande et verdict par
  fichier collés au Dev Agent Record.
- **Déclencheur de test** (tests 5 et 6 (c)) : mêmes pré-requis que le test 8 de la 15-7b2 (L4-4 de sa P4) —
  `CREATE TRIGGER` par `sqlx::raw_sql`, corps `SIGNAL SQLSTATE '45000'`, journal binaire inactif sur le
  montage — **alignés sur la 15-7b2** (R5-5 de sa P5, C-15-7-52 ; R3-8 de la P3) : `SELECT @@log_bin`
  relevé puis `assert_eq!(log_bin, 0, …)` avec un message qui **nomme le pré-requis** (`SUPER` ou
  `log_bin_trust_function_creators` exigés sinon), **jamais** de sortie anticipée (un `return` ferait un
  test muet) ni de `SET GLOBAL` ; un échec de montage ne se lit pas comme un échec du test. Même montage
  que le test 8 de la 15-7b2 : le factoriser en helper de test si les deux fiches le portent.
- **Garde `EXISTS` et `SAVEPOINT` : ce qui rend la garde observable** (R3-1 = F3-1 de la P3). Sur les
  29 clés étrangères qui désignent `companies`, **25** sont en `RESTRICT` : pour elles, la garde `EXISTS`
  et le `SAVEPOINT` **se recouvrent** — si la garde laissait passer une société désignée, le `DELETE`
  lèverait 1451, le `ROLLBACK TO SAVEPOINT` l'absorberait, et la société resterait en place ; seule
  différence, un `warn!` « société provisoire … conservée » que rien n'asserte. Seules les **quatre**
  tables en `ON DELETE CASCADE` (`users`, `bank_profiles`, `contact_persons`, `email_templates`) rendent
  la garde **observable** : sans elle, le `DELETE` réussit et efface leurs lignes en silence. C'est
  **voulu** — défense en profondeur : la garde protège les cascades, le `SAVEPOINT` rattrape ce que la
  garde n'aurait pas vu (une table ajoutée entre la lecture du schéma et le `DELETE`, un déclencheur) —
  et c'est pourquoi la mutation qui exclut une table de la liste porte sur une table en **CASCADE**
  (mutation 9, troisième variante). Le test 4 (ii) prouve le **comportement** (un stub désigné par une
  clé d'API reste), **non** la garde : la clé étrangère d'`api_keys` est en `RESTRICT`.
- **Statut de la fiche** : `backlog` jusqu'à la convergence de sa validation (décision de l'orchestrateur,
  C-15-7-44), à la différence des fiches sœurs restées `ready-for-dev` pendant leurs passes.

### Tests

Montage commun des états orphelins (R1-6 de la P1) : sur une **connexion détachée du pool** —
`pool.acquire().await?.detach()` (sqlx 0.8 : la connexion ne retourne jamais au pool), fermée en fin de
montage — `SET FOREIGN_KEY_CHECKS=0` de session, puis `users.company_id` et **toutes** les lignes
d'`api_keys`, **y compris la clé déjà révoquée**, portés à un id **inexistant** (sans quoi une clé
révoquée non orpheline laisserait verte la mutation du filtre `revoked_at IS NULL`). Clés créées par
`kesh_api::auth::api_key::generate_pat` + `api_keys::create_in_tx` (R1-5), jeton gardé.

| # | Test | AC |
|---|---|---|
| 1 | **Démarrage** — dans `mod tests` de `crates/kesh-api/src/auth/bootstrap.rs`, attribut `#[sqlx::test(migrations = "../kesh-db/test-schema")]`. Montage, **dans cet ordre** (R1-3, F9 de la P1) : une société ; un utilisateur **Comptable** U ; un **administrateur inactif** A0 ; un **administrateur actif** A1 — de sorte que A1 ne soit ni le plus petit utilisateur ni le plus petit administrateur ; **deux** clés d'API (créateur A1) dont une **déjà révoquée** ; état orphelin (montage commun). `ensure_admin_user(&pool, &test_config_no_env())` ⇒ `Ok(3)` ; les trois utilisateurs désignent l'unique société ; la clé active est **révoquée** (`revoked_at` non nul, `version` + 1) **et** désigne la société ; `find_active_auth_by_key_hash` de son condensat rend `None` ; la clé déjà révoquée désigne la société, `revoked_at` et `version` **inchangés** ; `list_by_company(société, true)` rend les **deux** clés ; **une** entrée `installation.repaired`, `details = {company_id, stub_companies_removed: [], users_repointed: 3, user_ids: [trois ids], api_keys_revoked: [{id, name, created_by_user_id, created_at, last_used_at} de la clé active], api_keys_repointed: 2, trigger: "startup"}`, **acteur = A1**, `audit_log.company_id` = la société. Second appel ⇒ aucune entrée neuve. **Variante saine** : principaux rattachés ⇒ aucune entrée. **Variante sans administrateur actif** : U et A0 seulement ⇒ acteur = U. **Variante deux sociétés** : **les deux** sociétés **non provisoires** (`companies::create`, ou la première passée `is_stub = FALSE` — R2-4 de la P2 : une première société provisoire serait superflue et supprimée), principaux orphelins ⇒ la seconde n'est pas supprimée ; la clé active est révoquée et garde son `company_id` mort, aucun utilisateur rattaché, `api_keys_repointed = 0`, une entrée `company_id: null`. **Variante aucune société** (montage sans ligne `companies`) ⇒ clé active révoquée, non repointée, utilisateurs inchangés, une entrée `company_id: null` | 1, 2, 4 |
| 2 | **Restauration** — dans `crates/kesh-api/tests/admin_full_import_e2e.rs` (helpers `seed_admin`, `forge_jwt`, `export_backup`, `post_import`). Montage (R1-1 de la P1) : un administrateur A (`seed_admin`) puis un **second administrateur B d'id supérieur** ; une clé d'API active créée par A (jeton gardé) ; état orphelin (montage commun) ; **avant** l'import (R1-5), une requête `GET /api/v1/companies/current` authentifiée par le jeton de la clé ⇒ **500** (`get_company_for` sur l'id mort : le jeton est accepté par l'authentification, ce qui donne son sens au 401 final) ; JWT forgé **pour B** avec l'id de la société **vivante** (`forge_jwt(B, "Admin", société)`) ; export, puis import de cette archive par B ⇒ 200 ; après : A et B désignent l'unique société, la clé est révoquée et désigne la société, la même requête par son jeton ⇒ **401** ; une entrée `installation.repaired` avec `trigger: "restore"`, **`triggered_by_user` = B**, **acteur = A** = acteur d'`admin.full_import`, **≠ B** ; l'entrée `admin.full_import` de cet import a pour `company_id` la société vivante (F11 de la P1). `books.restored` n'est **pas** assertée : elle n'est écrite que si le verrou de période recule (`a_recule`, `routes/admin.rs:387-392`), ce que le montage ne fait pas (R2-2 de la P2, AC 3). ⚠️ Si la route d'export devait exiger une société vivante (elle ne le fait pas, `routes/admin.rs:32-67`), forger l'archive à la place (`unzip`, réécriture de `data/users.ndjson` et `data/api_keys.ndjson`, empreinte du manifeste recalculée par `sha256_hex`, `rezip`) | 3, 4 |
| 3 | **Fonction seule** — dans `crates/kesh-db/tests/companies_repository.rs`, `#[sqlx::test(migrations = "./test-schema")]` : (a) **clés seules** orphelines, utilisateurs sains ⇒ `Some`, `user_ids` vide, `api_keys_revoked` = la clé, `api_keys_repointed = 1` ; (b) base **sans** société, utilisateurs orphelins, **aucune** clé ⇒ `None`, rien d'écrit (rattachement laissé à `language`) ; (b') base sans société, une clé active orpheline ⇒ `Some`, `company_id: None`, clé révoquée, non repointée ; (c) `Restore { actor_user_id: X, triggered_by_user: Y }` avec **X ≠ Y** (deux utilisateurs) ⇒ acteur = X, `details.triggered_by_user` = Y | 1 |
| 4 | **#542 au démarrage** (venu du test 6e de la 15-7b2, C-15-7-46) — dans `mod tests` de `auth/bootstrap.rs` : (i) `ensure_admin_user` avec variables d'administrateur (stub et administrateur) puis **deux** stubs de plus par `companies::insert_stub` (redémarrages v0.12.x simulés), sans utilisateur ni clé ⇒ démarrage suivant : une seule société, celle de l'administrateur (même id) ; entrée `stub_companies_removed` = les deux ids, `users_repointed = 0` ; `kesh_seed::seed_demo(&pool, &locale, UiMode::Guided, version)` passe ensuite (n'est plus en 500) — version lue par `kesh_db::repositories::onboarding::init_state(&pool)` (ou `get_state` si l'état existe), `Locale` de la configuration de test ; appel **direct** de `kesh_seed::seed_demo` (`kesh-seed/src/lib.rs:72-77`), dont le seul appelant existant est `routes/onboarding.rs:191` (`seed_demo(&state.pool, &state.config.locale, ui_mode, current.version)`) — patron de l'appel ; `onboarding_e2e.rs:581` (`fresh_install_stub_cleared_by_seed_demo`, par les routes HTTP) n'est que le patron de l'**état attendu** (R2-5 de la P2, R3-9 de la P3). (ii) Comme (i), mais le second stub porte une **clé d'API** ⇒ il n'est **pas** supprimé — ce test prouve le comportement, non la garde (`api_keys` est en `RESTRICT` : la garde et le `SAVEPOINT` s'y recouvrent, Dev Notes ; R3-1 = F3-1 de la P3). (iii) **Sans utilisateur** : trois stubs ⇒ deux supprimés, **aucune** entrée d'audit, `Ok(0)`. (iv) **Montage défensif** (#542 cumulé avec #528, aucun chemin connu) : trois stubs sans principal, `users.company_id` mort ⇒ la société conservée est `MIN(id)`, `stub_companies_removed` = deux ids, `users_repointed = 1` | 1, 2 |
| 5 | **Échec non bloquant au démarrage** (F6 de la P1) — dans `mod tests` de `auth/bootstrap.rs` : montage du test 1, puis déclencheur `BEFORE INSERT ON audit_log` (`SIGNAL SQLSTATE '45000'`, cf. Dev Notes) ⇒ `ensure_admin_user` rend **`Ok(3)`** (pas d'erreur) ; état **inchangé** (utilisateurs orphelins, clé active non révoquée, aucune entrée) ; `DROP TRIGGER`, nouvel appel ⇒ réparé comme au test 1 | 2 |
| 6 | **Références des sociétés provisoires** (F-2 de la P2, C-15-7-50) — dans `crates/kesh-db/tests/companies_repository.rs`, `#[sqlx::test(migrations = "./test-schema")]` : (a) **schéma** — `company_referencing_columns` rend un ensemble qui **contient** `(users, company_id)`, `(api_keys, company_id)`, `(bank_profiles, company_id)`, `(contact_persons, company_id)`, `(email_templates, company_id)` (les quatre `ON DELETE CASCADE` et `api_keys`), ainsi que `(accounts, company_id)` et `(fiscal_years, company_id)` (RESTRICT) — inclusion d'une liste écrite, **non** égalité avec une seconde lecture du même `information_schema` (elle serait verte par construction) ; (b) **une seule table suffit** — un administrateur sur la société `MIN(id)`, puis, pour chaque table de la liste {`bank_profiles` (CASCADE, **obligatoire**), `email_templates` (CASCADE), `accounts` (RESTRICT)}, un stub supplémentaire désigné par **une seule** ligne de cette table ; `repair_installation_in_tx(Startup)` ⇒ ces stubs **restent**, leurs lignes filles **aussi** (la variante CASCADE prouve qu'aucune n'est effacée en silence), et un stub sans aucune référence est supprimé ; (c) **SAVEPOINT** — trois stubs superflus, un utilisateur et une clé d'API active, tous deux au `company_id` mort (montage commun), déclencheur `BEFORE DELETE ON companies` qui `SIGNAL` pour **un** des deux stubs à supprimer (pré-requis du test 5, Dev Notes) ⇒ `Ok(Some)` ; ce stub **reste**, l'autre est supprimé et **seul** dans `stub_companies_removed` ; deux sociétés restent, donc `company_id: None` (AC 1, étape 3) et l'utilisateur n'est pas rattaché ; la clé est **révoquée** quand même (étape 5 atteinte) ; une entrée écrite | 1 |

⚠️ **Prouver que les tests mordent** — chaque mutation seule, puis le fichier **touché** :
1. retirer, **dans** `ensure_admin_user`, l'appel à `repair_installation_in_tx` (test 1 rouge : principaux toujours orphelins — R2-9 de la P2) ;
2. retirer la condition « exactement une société » (passer la première société comme cible) — variante deux sociétés du test 1 rouge (rattachement à la première) ;
3. écrire l'entrée sans condition — variante saine du test 1 rouge (une entrée à chaque démarrage) ;
4. retirer, **dans** `run_backup_and_restore`, l'appel à `repair_installation_in_tx` (test 2 rouge) ;
5. signer l'entrée de la restauration par `current_user` **au site d'appel** (passer `current_user.user_id` comme `actor_user_id`) — test 2 rouge (acteur B au lieu de A) ;
6. signer l'entrée de la restauration par `triggered_by_user` **dans** la fonction — test 3 (c) rouge ;
7. retirer `active = TRUE` du choix de l'acteur du démarrage — test 1 rouge (acteur A0) ; retirer `role = 'Admin'` — test 1 rouge (acteur U) ;
8. propager l'erreur de la réparation au démarrage (`?`) — test 5 rouge (`Err`) ;
9. retirer la suppression des sociétés provisoires superflues — test 4 (i) rouge ; retirer `is_stub = TRUE` du prédicat — variante deux sociétés du test 1 rouge (la seconde société non provisoire, qu'aucune ligne ne désigne — `companies::create` n'écrit aucune ligne fille —, est supprimée et les principaux rattachés à la première) ; exclure **`bank_profiles`** de la liste des références (`AND TABLE_NAME <> 'bank_profiles'`) — test 6 (b) rouge sur la variante `bank_profiles` (stub supprimé et sa ligne `bank_profiles` effacée **en cascade**, sans 1451) ; la table est en `ON DELETE CASCADE` à dessein : une table en `RESTRICT` (`api_keys`, variante de la P2) laisserait la mutation **muette**, le 1451 étant absorbé par le `SAVEPOINT` (Dev Notes ; R3-1 = F3-1 de la P3) — `bank_profiles` plutôt que `users` parce que le test 6 (b) la monte déjà, sans montage neuf ;
10. retirer la branche « sans utilisateur » — test 4 (iii) rouge ;
11. supprimer les sociétés superflues aussi à la restauration — sans test : choix écrit (AC 1, étape 2), non une propriété observable sur un jeu sain ;
12. remplacer la lecture d'`information_schema` par la liste en dur {`users`, `api_keys`} — test 6 (a) rouge, et 6 (b) rouge sur la variante CASCADE (stub et ligne `bank_profiles` effacés) ;
13. retirer le `SAVEPOINT` (laisser l'erreur du `DELETE` remonter) — test 6 (c) rouge (`Err`, clé non révoquée, aucune entrée) ;
14. supprimer aussi `MIN(id)` quand **toutes** les sociétés sont superflues — test 4 (iii) rouge (trois suppressions au lieu de deux ; F3-7 de la P3).
Les mutations **de la règle des principaux** (repointer sans révoquer, révoquer sans repointer, retirer le
filtre `revoked_at IS NULL`, révoquer les clés saines) appartiennent à la fonction de la 15-7b2 et à son
test 6f ; les tests 1 et 2 ci-dessus les voient aussi.

### Ce que la story ne fait pas

- **Le rejeu sur interblocage** : ni au démarrage (angle mort assumé, AC 2), ni à la restauration (la route
  n'est pas sous `retry_with`, AC 3) — C-15-7-42.
- **La réactivation d'une clé révoquée** : impossible par conception ; l'administrateur en crée une neuve.
- **Plusieurs sociétés non superflues et des principaux orphelins** : clés révoquées, utilisateurs non
  rattachés — cas laissé (AC 2), atteint par aucun chemin connu.
- **Les sociétés provisoires superflues à la restauration** : supprimées au démarrage suivant (AC 1,
  étape 2).
- **Les lignes de démonstration orphelines** d'une remise à zéro antérieure (`vat_rates`,
  `company_invoice_settings`, contacts, produits, factures, avoirs) : **#546** ; limite écrite au manuel.
- **La connexion abandonnée de `restore_tables_in_tx`** : #540, inchangée.
- **Un redémarrage (ou une troisième instance) pendant qu'une instance née de la course sur base vide
  n'a pas encore créé son administrateur** : angle mort assumé (inventaire § 6, reformulé par R3-6 de la
  P3) — Kesh tourne en une instance.
- **Une archive sans société restaurée sur une installation déjà onboardée** : utilisateurs orphelins
  sans chemin de rattachement (clés révoquées par la restauration) — limite assumée, aucun mécanisme neuf
  (inventaire § 1, AC 2, manuel ; F3-2 de la P3).
- **Les sociétés provisoires désignées par une donnée** : conservées, même superflues pour le reste
  (AC 1, étape 2) ; elles ne gênent que `seed_demo` (`len() != 1`), cas atteint par aucun chemin connu.

### References

- Issues #528, #542, #546, #540.
- Fiche d'origine `15-7b2-remise-a-zero.md` (AC 14 et test 15 au commit `ffcdcf8d` ; AC 4 pour
  `reattach_orphan_principals_in_tx`, test 6f) ; fiche index `15-7-trace-onboarding.md`.
- Rapports de la P3 de la 15-7b2 : `target/gate-logs/15-7b2-p3-{R,F}.md` ; de la P4 de la 15-7b2 :
  `15-7b2-p4-{R,F}.md` ; de la P1 de cette fiche : `15-7b3-p1-{R,F}.md` (non versionnés) ; prompts
  `15-7b2-validate-prompt-p3.md`, `15-7b3-validate-prompt-p1.md`.
- Code : `repositories/api_keys.rs` (`revoke_in_tx`, `find_active_auth_by_key_hash`, `list_by_company`),
  `routes/admin.rs` (`run_backup_and_restore`, `full_export`), `auth/bootstrap.rs` (`ensure_admin_user`,
  matrice `//!`), `main.rs:181`, `:185`, `:369`, `admin-restore.api.ts:6`.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`claude-opus-5-5`), worktree `kesh-15-7b3`, base `e892dcfa`.

### Debug Log References

#### T0 — relecture de la fiche contre `e892dcfa` (2026-10-09, avant tout code)

La fiche date du 2026-10-08 ; depuis, la 15-7b2 (qu'elle appelle), les 15-13a/b, 15-6c/d, 15-1a-i/ii et
15-14a ont été mergées. Écarts relevés, **avant** de coder :

- **E1 — statut.** L'en-tête disait `Status: backlog` alors que la validation est close (P4 ciblée,
  sprint-status `ready-for-dev`) : passé à `ready-for-dev` puis `in-progress` (Change Log).
- **E2 — ce que la 15-7b2 a réellement posé : conforme.**
  `companies::reattach_orphan_principals_in_tx(tx, target: Option<i64>) -> Result<OrphanPrincipals, DbError>`
  (`repositories/companies.rs:306`) ; `OrphanPrincipals { user_ids: Vec<i64>, api_keys_revoked:
  Vec<RevokedApiKey>, api_keys_repointed: u64 }` + `is_empty()` ; `RevokedApiKey { id, name,
  created_by_user_id, created_at, last_used_at }` (`Serialize`, aucune empreinte). Règle « révoquer
  puis repointer » conforme (révocation de toute clé active orpheline quelle que soit la cible, puis,
  avec cible, rattachement des utilisateurs et repointage de toutes les clés orphelines sans toucher
  `version` ni `revoked_at`). **Pré-condition écrite** : l'appelant tient `companies … FOR UPDATE` ;
  ordre `users` puis `api_keys`. `companies::insert_stub<E: Executor>(executor, Language) -> i64` (`:173`)
  conforme ; garde `company_count == 0` des cas 1 et 2 du bootstrap présente.
- **E3 — détail de l'entrée.** L'entrée `installation.reset` de la 15-7b2 porte `users_repointed`
  (nombre) **sans** `user_ids` ; l'AC 1 étape 7 de cette fiche demande les deux : appliqué tel
  qu'écrit ici, divergence assumée (l'entrée de réparation désigne des utilisateurs qui n'ont rien fait).
- **E4 — renvois de ligne décalés** (relocalisés par symbole) : `bootstrap.rs` compteurs `:52-60`,
  doc de tête `:37-50`, matrice `//!` `:8-17` (le cas 1 y dit déjà #542) ; `main.rs` doc `:19`,
  `ensure_admin_user` `:190`, `exit(1)` `:194`, `TcpListener::bind` `:380` (fiche : `:181`, `:185`,
  `:369`) ; `routes/admin.rs` `run_backup_and_restore` `:225` (fiche `:221`), `restore_tables_in_tx`
  `:295`, 5-ter `:329-349`, `audit_uid` `:355-370`, `a_recule` `:393-397`, `books.restored` `:398-418`,
  `admin.full_import` `:446-459`.
- **E5 — `seed_demo` a changé de signature** (15-7b1) : `kesh_seed::seed_demo(pool, locale, actor:
  SeedActor)` et la dernière transaction exige `onboarding_state.step_completed == 2`. Le test 4 (i)
  appelle donc `seed_demo(&pool, &Locale, (admin, None))` après avoir porté l'état à l'étape 2 (la fiche
  écrivait `(pool, locale, UiMode::Guided, version)`).
- **E6 — tests d'intégration qui appellent `ensure_admin_user` : 18, non 15.** `grep -rln
  ensure_admin_user crates/kesh-api/tests | sort` rend en plus `filet_bilan_clos_e2e`, `letterings_e2e`,
  `onboarding_audit_e2e`. Contrôle des montages à risque (`grep -cE
  "insert_stub|is_stub|FOREIGN_KEY_CHECKS *= *0|seed_stub_company_only"`) : 0 pour quinze fichiers ;
  `onboarding_e2e` (2) et `onboarding_path_b_e2e` (1) ne le portent qu'en commentaires (`:578-579`,
  `:449`) ; `onboarding_audit_e2e` (39) monte ses états **après** `ensure_admin_user` (montage commun
  `bootstrap`, `:137-145` ; test 13 `:1136`) — aucun montage ne pose, **avant** l'appel, une société
  provisoire superflue ni un principal au `company_id` mort. Verdict confirmé par le gate complet (§
  Completion Notes).
- **E7 — schéma : conforme.** 29 clés étrangères vers `companies` dans le squash, dont 4 en
  `ON DELETE CASCADE` (`grep -ic "references \`\?companies"`, puis `| grep -ic cascade`) ; aucune table
  ajoutée par les 15-1a-i/ii ne désigne `companies` en cascade.
- **E8 — registre des routes : inchangé** (108 tracées / 4 / 2 sur 114, `audit_route_registry.rs:636-639`) :
  aucune route neuve.
- **E9 — restauration.** `post_restore` n'est pas touché (aucune migration) ; la garde lexicale
  `echecs_avant_sauvegarde_tous_convertis` (`routes/admin.rs`) ne couvre que le segment **avant**
  `write_pre_import_backup` : l'étape 5-quater, placée après la lecture d'`audit_uid`, n'y entre pas.
- **E10 — CHANGELOG.** `## [0.13.0] — Non publié` et `### Corrigé` existent sur la branche (rebasée sur
  `main`) ; l'entrée #542 de la 15-7b2 (« Un redémarrage avant la création de l'administrateur n'ajoute
  plus de société provisoire ») y est : l'entrée de la réparation s'ajoute à côté.
- **E11 — tri d'`ACTIONS`.** `installation.repaired` se range entre `installation.demo_seeded` et
  `installation.reset`.
- **E12 — montage du déclencheur.** Celui du test 8 de la 15-7b2 vit dans
  `crates/kesh-api/tests/onboarding_audit_e2e.rs` (`reset_failure_erases_nothing_and_never_returns_its_connection`) ;
  la fiche demande de le factoriser si les deux fiches le portent : voir C-15-7b3-1.

### Completion Notes List

**Ce qui est livré** (commits `059dc9a7` code et tests, `d7f4c1f3` documentation ; dernier commit de
code et de documentation = `d7f4c1f3`, sur lequel tous les gates ci-dessous ont tourné) :

- **T1** — `companies::{RepairTrigger, InstallationRepair, company_referencing_columns,
  repair_installation_in_tx}` (`crates/kesh-db/src/repositories/companies.rs`), dans l'ordre de
  l'AC 1 : `companies … FOR UPDATE` ; au démarrage seulement et à plus d'une société, sociétés
  provisoires qu'aucune ligne d'aucune table ne désigne (colonnes lues dans
  `information_schema.KEY_COLUMN_USAGE`, une requête `EXISTS` par colonne, identifiants entre accents
  graves), `MIN(id)` conservée si toutes le sont, chaque `DELETE` sous `SAVEPOINT repair_stub`
  (échec ⇒ `ROLLBACK TO`, `warn!` « société provisoire {id} conservée : {e} », réparation poursuivie ;
  échec du `ROLLBACK TO` ⇒ erreur **d'origine** rendue, celle du `ROLLBACK TO` en `warn!`) ;
  société de rattachement ; base sans utilisateur ⇒ `info!` et `Ok(None)` ; appel de
  `reattach_orphan_principals_in_tx` (15-7b2, non réécrite) ; rien fait ⇒ `Ok(None)` ; entrée
  `installation.repaired` (C-15-7b3-2). Ni `begin` ni `commit`. Écart de forme : `&mut *tx` au lieu du
  `&mut **tx` de la fiche pour `company_referencing_columns` (clippy `explicit_auto_deref`).
- **T2** — `ensure_admin_user` appelle en tête `repair_installation_at_startup` (privée) : une
  transaction, `commit` dans tous les cas, `info!` sur `Some`, **erreur non fatale** (`rollback`
  *best-effort* en `warn!`, `error!` « réparation de l'installation au démarrage : {e} — démarrage
  poursuivi, installation inchangée »). Doc de tête (invariant des compteurs), matrice `//!` (cas 3,
  sérialisation par `companies`), doc de tête de `main.rs`.
- **T3** — étape « 5-quater » de `run_backup_and_restore`, après la lecture d'`audit_uid`, avant
  `books.restored` et `admin.full_import` ; `Restore { actor_user_id: audit_uid, triggered_by_user:
  current_user.user_id }` ; erreur ⇒ `AdminFullImportFailed("réparation de l'installation : …")`.
- **T4** — `installation.repaired` dans `ACTIONS` (entre `demo_seeded` et `reset`) et dans les quatre
  catalogues (libellés de l'AC 5).
- **T5** — **18 tests neufs** (périmètre `e892dcfa..d7f4c1f3`, recomptés par `grep -c '#\[sqlx::test'`
  aux deux bornes) : `auth/bootstrap.rs` 10 → 20 (test 1 et ses quatre variantes, test 4 (i) à (iv),
  test 5), `companies_repository.rs` 22 → 29 attributs `#[sqlx::test]` (tests 3 (a), (b), (b'), (c), 6 (a), (b), (c) ; borne rectifiée à la revue P1, A4 : « 21 → 28 » comptait les tests exécutés, non les attributs),
  `admin_full_import_e2e.rs` 35 → 36 (test 2). Helpers partagés dans `test_fixtures`
  (C-15-7b3-1) ; le test 8 de la 15-7b2 y est raccordé. **Mutations : 16 jouées, 16 rouges** — les
  quatorze numérotées de la fiche sauf la 11 (sans test, choix écrit), avec les variantes 7a/7b et
  9a/9b/9c ; script et journal `kesh-gate-logs/157b3-mutations.{py,log}` (fichier restauré et
  `touch` après chaque mutation). Première passe : la mutation 4 ne compilait pas (motif mal formé),
  corrigée et rejouée, puis les seize rejouées avec un verdict strict (« `FAIL [` » présent).
- **T6** — manuel d'administration, manuel utilisateur, PDF (`make -B fr`), `docs/api-external.md`,
  CHANGELOG `[0.13.0]` `### Corrigé` (C-15-7b3-3) ; les quatre lignes `docker compose logs kesh`
  (`--tail=100`, `--since=1h`, `| tail -50`) et les deux `kesh` du symptôme « Kesh ne démarre pas »
  passées à `kesh-api`. Contrôles : `grep -nE "compose (logs|ps).*\bkesh\b" docs/manual/fr/*.tex | grep
  -v kesh-api` muet ; PDF aplatis (`pdftotext | tr '\n' ' ' | tr -s ' '`, apostrophes typographiques
  normalisées) : les dix phrases-témoins du manuel d'administration et les deux du manuel utilisateur
  présentes ; journal LaTeX 57 → 54 `Overfull` après correction des trois créés ici.
- **T7** — gates sur `d7f4c1f3`, bases `kesh_157b3` et `kesh_e2e_157b3` reconstruites avant :
  - **backend** `scripts/test-fast.sh` (fmt + clippy `-D warnings` + nextest) : **3215/3215, 4 ignorés**
    (3197 de la 15-7b2 + 18) — `kesh-gate-logs/157b3-gate-complet-1.log` ;
  - **frontend** : `npm run check` 0 erreur (27 avertissements préexistants), `lint-i18n-ownership`
    PASS, Vitest **1164/1164** (115 fichiers), `npm run build` vert — `157b3-frontend.log` ;
  - **E2E complet** (port 3020, recette de `docs/testing.md`, `KESH_ADMIN_BACKUP_DIR` inscriptible) :
    **239 passés, 15 échecs, 19 ignorés** — 7 **KF-029** attendus (`mode-expert:26`, `:41`,
    `onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`, `:150`) + 8 hors liste
    (`invoice-frozen-pdf:74`, `invoices:715`, `supplier-invoice-scan:39`, `bank-accounts-crud:143`,
    `bank-import:186`, `contacts:69`, `:113`, `:149`) — six en `waiting for locator('#username')`, deux (`invoice-frozen-pdf:74`, `invoices:715`) sur un élément absent, et ce sont les deux dont la trace porte `ERR_NETWORK_CHANGED` (rectifié à la revue P1, A3 : « tous » était faux),
    **8/8 verts rejoués seuls** ; les deux traces conservées portent `ERR_NETWORK_CHANGED` (12 et 44
    occurrences) : **KF-053 (#478)**. Run après 12:00 UTC (KF-045 hors jeu). Journaux
    `157b3-e2e.log`, `157b3-e2e-rejeu.log`, `test-results` copiés dans
    `157b3-e2e-test-results-run1/`. Backend arrêté par son PID.
  - tmpfs de `kesh-mariadb-dev` relevé avant et après (lecture seule) : 1,5 G / 8 G, 18 %, inchangé.

**Contrôle des tests appelant `ensure_admin_user`** (Dev Notes, F3-5) : voir T0, E6 — dix-huit
fichiers, aucun montage à risque avant l'appel ; confirmé par le gate complet vert.

**Choix consignés** : C-15-7b3-1 (helpers de montage partagés), C-15-7b3-2 (acteur en une requête ;
`user_ids` au détail), C-15-7b3-3 (lieu du texte au manuel, textes devenus faux, ajouts hors liste,
débordements).

**Non fait, à dessein** : revue de code (non lancée, consigne) ; rien sur `kesh-mariadb-dev` hors des
deux bases de la story.

### File List

- `crates/kesh-db/src/repositories/companies.rs` — `RepairTrigger`, `InstallationRepair`, `company_referencing_columns`, `repair_installation_in_tx`
- `crates/kesh-db/tests/support/installations_atteintes.rs` — `rendre_principaux_orphelins`, `poser_declencheur_en_echec` (d'abord dans `crates/kesh-db/src/test_fixtures.rs`, revenu à sa version de `e892dcfa` à la revue P1)
- `docs/MULTI-TENANT-SCOPING-PATTERNS.md` — ligne de la réparation (revue P1)
- `crates/kesh-db/tests/companies_repository.rs` — tests 3 et 6
- `crates/kesh-api/src/auth/bootstrap.rs` — appel au démarrage, doc-comments, tests 1, 4, 5
- `crates/kesh-api/src/main.rs` — doc de tête
- `crates/kesh-api/src/routes/admin.rs` — étape 5-quater
- `crates/kesh-api/src/audit_labels.rs` — `installation.repaired`
- `crates/kesh-api/tests/admin_full_import_e2e.rs` — test 2
- `crates/kesh-api/tests/onboarding_audit_e2e.rs` — test 8 de la 15-7b2 raccordé au helper
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` — libellé
- `docs/manual/fr/admin-manual.tex`, `docs/manual/fr/user-manual.tex` et leurs PDF (la brochure, régénérée par `make -B fr`, est revenue à sa version d'`origin/main` à la revue P1)
- `docs/api-external.md`, `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/15-7b3-reparation-des-installations-atteintes.md`, `sprint-status.yaml`, `epic-15-choix-autonomes.md`

## Change Log

- 2026-10-09 — **Revue de code close** : passe P2 ciblée (Haiku, prompt `b36b2114`, rapport
  `/home/gcorbaz/devel/kesh-gate-logs/15-7b3-review-p2-ciblee.md`) sur `d38aeb45` : **0 CRITICAL, 0 HIGH, 0 MEDIUM**, 2 LOW
  (un `mod #[path]` inséré entre des blocs `use` dans `admin_full_import_e2e.rs:38-40` et `onboarding_audit_e2e.rs:28-30`,
  cosmétique ; une remarque sur la garde de liste vide, sans correction demandée) — laissés en l'état. Les quatre axes
  exercés : garde de liste vide (Invariant, connexion détachée sans pollution), aides déplacées (plus aucune référence
  à l'ancien emplacement, `test_fixtures.rs` identique à `e892dcfa`), tests neufs probants, manuel (`restart` justifié,
  recettes `up -d` préservées). Trend : P1 (Sonnet ×3) 3 MEDIUM → P2 ciblée (Haiku) 0. Dernier commit de code :
  `d38aeb45`, dont les gates (backend 3218/3218, Vitest 1164/1164, E2E 245 / 7 KF-029 + 2 KF-053 rejoués verts seuls,
  19 mutations rouges) tiennent. Statut **done**.

- 2026-10-08 — **Née du découpage de la 15-7b2** à sa passe de validation P3 (choix C-15-7-40). Reprend
  l'AC 14 de la 15-7b2 (réparation au démarrage, C-15-7-38), son test 15 et ses trois mutations, la
  ligne `installation.principals_reattached` de son AC 9, la part « démarrage » de ses AC 10, 11 et 12.
  **Remédiés à la naissance** : F3-1 (MEDIUM — une clé orpheline n'est plus repointée mais **révoquée**,
  ses ids au journal, le manuel le dit : AC 1 étape 6, AC 4, AC 5, AC 6 — C-15-7-41) ; F3-2 (MEDIUM — la
  même réparation dans la transaction de restauration : AC 3, test 2 — C-15-7-44) ; F3-3 (LOW — acteur du
  démarrage écrit comme choix, AC 1) ; R3-7 (LOW — `ensure_admin_user(&pool, &…)`) ; R3-8 (LOW — pas de
  rejeu : angle mort assumé au démarrage, route de restauration hors `retry_with`, AC 2 et 3 —
  C-15-7-42). Recompte : **6 AC**, **7 tâches**, **3 tests** (1, 2, 3), **7 mutations**, **1 action**
  d'audit. Validation : P1 complète à lancer.
- 2026-10-08 — **Passe de validation P1** (prompt versionné `15-7b3-validate-prompt-p1.md` ; deux
  lentilles **Opus** en contexte frais, R auditeur d'acceptation et F full-scope adversary ; rapports
  `target/gate-logs/15-7b3-p1-{R,F}.md`, non versionnés), remédiée avec la P4 de la 15-7b2 (décisions de
  l'orchestrateur). Bruts, recomptés depuis les rapports : R **2 MEDIUM, 7 LOW** ; F **1 HIGH,
  5 MEDIUM, 6 LOW**. Après fusion (R1-2 = F1 ; R1-3 = F9) : **0 CRITICAL, 1 HIGH, 6 MEDIUM, 12 LOW
  distincts**.

  | finding | sévérité | lentilles | objet | sort |
  |---|---|---|---|---|
  | F1 = R1-2 | HIGH | F, R | les clés orphelines d'une installation sans société (ou à plusieurs) restent actives et sont repointées — réveillées — par `language` ou la remise à zéro de la 15-7b2 | **une règle, une fonction** : `reattach_orphan_principals_in_tx`, posée par la 15-7b2, révoque toute clé active orpheline avant tout repointage, sur les quatre chemins ; appelée ici quel que soit le nombre de sociétés (AC 1 étape 5, AC 2, AC 4 ; dépendance écrite) — C-15-7-45 |
  | F2 | MEDIUM | F | révoquer sans repointer : clé étrangère pendante, clé invisible ; rejet de « révoquer **et** repointer » circulaire | révoquée **puis** repointée quand une société de rattachement existe ; visible, révoquée, sur la page des clés (AC 4) — C-15-7-45 révise C-15-7-41 |
  | F3 | MEDIUM | F | ids seuls : l'administrateur ne sait pas quelle intégration recréer | `api_keys_revoked: [{id, name, created_by_user_id, created_at, last_used_at}]` ; **aucun préfixe** au schéma (seule l'empreinte `key_hash`, jamais journalisée) — C-15-7-49 |
  | F4 | MEDIUM | F | autres lignes orphelines de #279 non nommées | non nettoyées : inventaire § 9, « ne fait pas », manuel, CHANGELOG ; renvoi à **#546** — C-15-7-49 |
  | F5 | MEDIUM | F | « l'import déconnecte déjà tout le monde » faux | inventaire § 7 et ligne du manuel réécrits : seul l'importateur est redirigé ; les autres sessions gardent l'ancien id jusqu'à expiration — C-15-7-49 |
  | F6 | MEDIUM | F | un échec de la réparation refuse le démarrage, sans recours | **non bloquant** : `error!`, démarrage poursuivi, installation inchangée ; restauration inchangée (import annulé) ; test 5, mutation 8, manuel (quoi faire) — C-15-7-47 |
  | R1-1 | MEDIUM | R | mutation « signer par `current_user` » ne rougit aucun test | test 2 : second administrateur B d'id supérieur émet l'import, acteur A ≠ `triggered_by_user` B ; test 3 (c) à deux ids distincts ; mutations 5 et 6 séparées |
  | R1-3 = F9 | LOW | R, F | acteur du démarrage non discriminé | montage U, A0 inactif, A1 actif ; variante sans administrateur actif ; mutation 7 |
  | R1-4 | LOW | R | `Invariant` « aucun utilisateur » | base sans utilisateur : rien d'autre que les sociétés superflues, sans entrée, `Ok(None)` (AC 1 étape 4) — C-15-7-47 |
  | R1-5 | LOW | R | 401 final non verrouillé | requête par le jeton **avant** l'import ⇒ 500 ; création par `generate_pat` + `create_in_tx` |
  | R1-6 | LOW | R | « connexion hors du pool » imprécis ; clé révoquée non orpheline | `pool.acquire().await?.detach()` ; **toutes** les lignes d'`api_keys` au `company_id` mort (montage commun) |
  | R1-7 | LOW | R | deux fichiers d'import oubliés | `admin_backup_e2e.rs`, `admin_pat_denied_e2e.rs` au contrôle (Dev Notes) |
  | R1-8 | LOW | R | renvois décalés | `.env.example:273-279` ; trois `INSERT INTO companies` en `mod tests` (`:674`, `:791`, `:951`). **Le renvoi `:346-353` était juste** (commentaire `:346-353`, requête `:354-358`, relu par `grep -n`) : R1-8 et F12 le décalaient chacun dans un sens ; précisé en trois plages |
  | R1-9 | LOW | R | libellé unique pour trois issues | libellé **neutre** « Réparation de l'installation » ; action renommée `installation.repaired`, fonction `repair_installation_in_tx` (l'entrée porte aussi les sociétés supprimées) — C-15-7-47 |
  | F7 | LOW | F | angle mort « seconde instance » contredit le module | inventaire § 6 : sérialisation par l'étape 1 ; matrice `//!` du module et ligne « existant & non » du manuel (AC 6) |
  | F8 | LOW | F | mécanisme de la non-vérification de la FK non écrit | Dev Notes (index d'`api_keys`, test qui rougirait) ; sans objet quand la clé est repointée |
  | F10 | LOW | F | § *Clés API* du manuel et `docs/api-external.md` absents | deux lignes à l'AC 6 |
  | F11 | LOW | F | `books.restored`/`admin.full_import` changent de société | écrit à l'AC 3, asserté au test 2 |
  | F12 | LOW | F | renvois ; précédents ; garde du littéral | `main.rs:185` ; précédents `companies.rs:380`, `:508`, `reconciliation_rules.rs:215` ; garde `audit_label_registry.rs` nommée (AC 5) |

  **Décisions de l'orchestrateur appliquées** : F1/R1-2 et F2 (C-15-7-45) ; F3, F4, F5 (C-15-7-49) ; F6
  (C-15-7-47) ; R1-1 ; tous les LOW. **Élargissement venu de la P4 de la 15-7b2** (F4-3) : la réparation
  des installations touchées par #542 (sociétés provisoires superflues) passe ici, au démarrage, avec
  l'ex-test 6e de la 15-7b2 (test 4) — `closes #542` (C-15-7-46). **Règle de découpage** : recomptée,
  **cinq modules** (la suppression des stubs vit dans `companies`, déjà compté) — non franchie, aucune
  dérogation. **Signal D5** : la sévérité de la P1 (HIGH) est celle d'une première passe ; le HIGH n'est
  pas né d'une remédiation de cette fiche mais d'une décision (C-15-7-41) appliquée à un chemin sur trois,
  défaut de **propagation** déclaré au Project Lead avec celui de la 15-7b2 (son Change Log P4). La
  remédiation change la conception (dépendance à la 15-7b2, #542, démarrage non bloquant) : **la
  validation n'est pas close**, P2 complète à lancer. Propagation : grep de `attach_all_principals`,
  `attach_users_in_tx`, `repair_orphan_principals`, `principals_reattached`, `principals-reattached`,
  `OrphanRepair`, `sans la repointer`, `déconnect`, `tout le monde`, `refuse le démarrage`,
  `exactement une`, `stub_companies_removed`, `\b542\b`, `\b546\b` sur les fiches 15-7*, l'index, le
  registre et `sprint-status.yaml`. Recompte : **6 AC**, **7 tâches**, **5 tests** (1 à 5), **11 mutations**
  numérotées (dont la 7 et la 9 en deux et trois variantes ; la 11 sans test, écrite), **1 action**
  d'audit.
- 2026-10-08 — **Passe de validation P2** (prompt versionné `15-7b3-validate-prompt-p2.md` ; deux
  lentilles **Sonnet** en contexte frais, R regression hunter et F full-scope adversary ; rapports
  `target/gate-logs/15-7b3-p2-{R,F}.md`, non versionnés), remédiée sur décisions de l'orchestrateur
  (C-15-7-50). Bruts, recomptés depuis les rapports : R **2 MEDIUM, 9 LOW** ; F **2 MEDIUM, 3 LOW**.
  Après fusion (R2-3 = F-3) : **0 CRITICAL, 0 HIGH, 4 MEDIUM, 11 LOW distincts**. Trend : P1 1 HIGH,
  6 MEDIUM, 12 LOW → P2 0 HIGH, 4 MEDIUM, 11 LOW.

  | finding | sévérité | lentilles | objet | sort |
  |---|---|---|---|---|
  | F-2 | MEDIUM | F | la garde « superflue » ne contrôlait que `users` et `api_keys` sur 29 clés étrangères ; quatre en `ON DELETE CASCADE` effaceraient en silence ; un 1451 bloquait toute la réparation | une société provisoire n'est supprimée que si **aucune ligne d'aucune table** ne la désigne, liste lue dans `information_schema.KEY_COLUMN_USAGE` (`company_referencing_columns`) ; chaque suppression sous `SAVEPOINT`, échec ⇒ société conservée, `warn!`, réparation poursuivie (AC 1 étape 2, inventaire § 8, test 6, mutations 12 et 13) |
  | F-1 | MEDIUM | F | note d'exploitation placée dans l'étape « Parcours » de l'assistant | texte principal au § *Procédure de mise à jour standard* (`:1704-1717`) et au § *Dépannage* (`:2054`, sous-section neuve après `:2056`) ; `:1314` ne garde qu'un renvoi (AC 6) |
  | R2-1 | MEDIUM | R | manuel : stubs supprimés « dès qu'un utilisateur existe », contraire à l'AC 1 | « y compris avant la création de l'administrateur (sans entrée au journal) » (AC 6) |
  | R2-2 | MEDIUM | R | test 2 : `books.restored` n'existe que si le verrou recule | assertion limitée à `admin.full_import` ; AC 3 précise `a_recule` ; pas de variante de recul (même sous-SELECT, rien de plus à apprendre) |
  | R2-3 = F-3 | LOW | R, F | place de la réparation vs compteurs ; deux instances sur base vide | **avant la lecture des compteurs**, invariant écrit (AC 2) ; deux instances simultanées sur base vide : angle mort assumé (inventaire § 6, « ne fait pas ») |
  | R2-4 | LOW | R | variante deux sociétés : première société possiblement provisoire | les deux non provisoires (test 1) |
  | R2-5 | LOW | R | `seed_demo` : version d'onboarding non nommée | `onboarding::init_state`/`get_state`, `UiMode::Guided`, patron `onboarding_e2e.rs:581` (test 4) |
  | R2-6 | LOW | R | idempotence : « plus aucun utilisateur orphelin » faux | reformulée (Dev Notes) |
  | R2-7 | LOW | R | motif « une clé d'une base sans utilisateur n'authentifie pas » creux | `fk_api_keys_created_by` (squash `:84`) : aucune clé sans créateur (AC 1 étape 4) |
  | R2-8 | LOW | R | deux sites de journalisation non coordonnés | la fonction journalise le cas sans utilisateur, l'appelant le cas `Some` (AC 1 étape 4, AC 2) |
  | R2-9 | LOW | R | mutations 1 et 4 ambiguës | « retirer, dans X, l'appel à `repair_installation_in_tx` » |
  | R2-10 | LOW | R | doc de tête de `main.rs:19` absente | ajoutée aux doc-comments (AC 6, T2) |
  | R2-11 | LOW | R | « cette version ne relève pas la version minimale » engage toute la release | « cette réparation n'impose aucune version minimale », à confirmer à la préparation de la release (ligne Dépannage, AC 6) |
  | F-4 | LOW | F | manuel utilisateur `:297` (« Clés API ») hors AC 6 | une phrase, PDF utilisateur régénéré (AC 6, T6) |
  | F-5 | LOW | F | section `## [0.13.0]` du CHANGELOG absente de la branche | existe sur `main` (`CHANGELOG.md:11` d'`origin/main`) : l'entrée s'y ajoute après rebase (AC 6) |

  **Décisions de l'orchestrateur appliquées** : F-2, F-1, R2-1, R2-2 (assertion limitée à
  `admin.full_import`, aucune variante de recul), R2-3 = F-3, F-4, F-5, tous les LOW — C-15-7-50.
  **Fiche 15-7b2 non modifiée** : aucun finding ne l'exige (la fonction partagée et la garde
  `company_count == 0` sont appelées telles quelles ; l'invariant des compteurs est écrit ici).
  **Règle de découpage** : recomptée, **cinq modules** (`information_schema` lu dans `companies`, déjà
  compté) — non franchie. **Signal D5** : sévérité en baisse (HIGH → MEDIUM), non déclenché ; déclaré :
  F-2, R2-1 et R2-2 portent sur des ajouts de la remédiation P1 (suppression des stubs, base sans
  utilisateur, assertion F11), défauts **distincts** de ceux de la P1, non recyclés. La remédiation
  change la conception de l'étape 2 (lecture du schéma, `SAVEPOINT`) : **validation non close**, P3
  complète à lancer. Propagation : grep de `NOT EXISTS`, `1451`, `dès qu'un utilisateur`,
  `books.restored`, `cette version ne relève`, `avant la répartition`, `retirer l'appel d`, `n'ont jamais
  reçu`, `\b528\b`, `\b542\b` sur les fiches 15-7*, l'index, le registre et `sprint-status.yaml` ; les
  occurrences restantes sont historiques (Change Logs, registre) ou justes (inventaire § 8, AC 3).
  Recompte : **6 AC**, **7 tâches**, **6 tests** (1 à 6), **13 mutations** numérotées (dont la 7 et la 9
  en deux et trois variantes ; la 11 sans test, écrite), **1 action** d'audit, **5 modules**.
- 2026-10-08 — **Passe de validation P3** (prompt versionné `15-7b3-validate-prompt-p3.md` ; deux
  lentilles **Opus** en contexte frais, R regression hunter et F full-scope adversary ; rapports
  `target/gate-logs/15-7b3-p3-{R,F}.md`, non versionnés), remédiée sur décisions de l'orchestrateur
  (C-15-7-53). Bruts, recomptés depuis les rapports : R **2 MEDIUM, 7 LOW** ; F **1 MEDIUM, 7 LOW**.
  Après fusion (R3-1 = F3-1 ; R3-2 = F3-3, retenu au plus haut, MEDIUM) : **0 CRITICAL, 0 HIGH,
  2 MEDIUM, 13 LOW distincts**. Trend : P1 1 HIGH, 6 MEDIUM, 12 LOW → P2 0 HIGH, 4 MEDIUM, 11 LOW → P3
  0 HIGH, 2 MEDIUM, 13 LOW. Modèles : P1 Opus ×2, P2 Sonnet ×2, P3 Opus ×2.

  | finding | sévérité | lentilles | objet | sort |
  |---|---|---|---|---|
  | R3-1 = F3-1 | MEDIUM | R, F | la troisième variante de la mutation 9 (exclure `api_keys`, table en `RESTRICT`) est **muette** depuis le `SAVEPOINT` de la P2 : le 1451 est absorbé, le stub reste, le test 4 (ii) reste vert | variante portée sur **`bank_profiles`** (CASCADE, déjà montée au test 6 (b), sans montage neuf ; `users` écarté pour cette raison) — test 6 (b) rouge ; Dev Notes : pour les 25 tables en `RESTRICT`, la garde et le `SAVEPOINT` se recouvrent, seules les quatre CASCADE rendent la garde observable — voulu (défense en profondeur), écrit ; test 4 (ii) dit prouver le comportement, non la garde |
  | R3-2 = F3-3 | MEDIUM | R (MEDIUM), F (LOW) | `docker compose logs kesh` : le service s'appelle `kesh-api` | cellule *Dépannage* en `kesh-api` ; correction **au passage** des lignes fausses préexistantes du manuel : `:2063`, `:2058` (deux occurrences), **et `:1905`, `:1906`** — l'avis initial tenait ces deux dernières pour justes, le `grep -n "compose logs"` à `cf40085f` les montre fausses (`--tail=100 kesh`, `--since=1h kesh`) ; grep de contrôle ajouté à l'AC 6, T6 |
  | F3-2 | LOW | F | archive sans société restaurée sur une installation onboardée : `language` et la remise à zéro inatteignables, utilisateurs orphelins sans recours | **limite assumée**, aucun mécanisme neuf : inventaire § 1, « Cas laissés » de l'AC 2 borné, « ne fait pas », phrase au § *Reprises* du manuel (AC 6) |
  | F3-4 | LOW | F | « Retour arrière » au Dépannage, à côté d'un § *Rollback en cas d'échec* ignoré ; la réparation n'est pas défaite par un retour d'image | ligne neuve à l'AC 6 pour `admin-manual.tex:1734-1749` (retour d'image ne défait pas la réparation ; seule la sauvegarde d'avant la mise à jour la rend ; version minimale) ; le Dépannage n'y fait que renvoyer |
  | F3-5 | LOW | F | quinze fichiers de tests appellent `ensure_admin_user`, non nommés | commande, liste et contrôle par fichier au Dev Agent Record (Dev Notes) |
  | F3-6 | LOW | F | `SAVEPOINT` : erreur rendue et forme d'émission non dites | l'erreur **d'origine** (celle du `DELETE`, ex. 1213) est rendue, jamais le 1305 du `ROLLBACK TO`, qui est journalisé ; forme du précédent `reconciliation.rs:1026-1072` ou `sqlx::raw_sql`, nom constant (AC 1 étape 2) |
  | F3-7 | LOW | F | aucune mutation pour « `MIN(id)` conservée » | mutation 14 — test 4 (iii) rouge |
  | F3-8 | LOW | F | `### Corrigé` supposé présent sous `[0.13.0]` | présent à `origin/main` le 2026-10-08 (relu) ; « le créer s'il est absent après rebase » (AC 6) |
  | R3-3 | LOW | R | « sur `None` aussi : la transaction n'a rien écrit » faux depuis l'étape 4 | `commit` dans tous les cas, motif corrigé (AC 2) ; les deux sens de `None` écrits (AC 1 étape 6) |
  | R3-4 | LOW | R | précédent du `SAVEPOINT` non cité ; échec de `RELEASE` non dit ; lieu du prédicat 1305 | précédent cité ; erreur de `SAVEPOINT`/`RELEASE` rendue ; `is_savepoint_lost` **inutile ici** (toute erreur du `ROLLBACK TO` arrête la réparation) : rien à dupliquer (AC 1 étape 2) |
  | R3-5 | LOW | R | type du paramètre de `company_referencing_columns` | `&mut MySqlConnection` ; appels `&mut **tx` et `&mut *pool.acquire().await?` (AC 1 étape 2) |
  | R3-6 | LOW | R | angle mort « deux instances sur base vide » inatteignable tel qu'écrit | reformulé : deux stubs nés de la course, puis redémarrage ou troisième instance pendant qu'une instance en vol n'a pas créé son administrateur (inventaire § 6, AC 2, « ne fait pas ») |
  | R3-7 | LOW | R | avertissement des renvois limité à `bootstrap.rs` ; `:1638` décalé | étendu au manuel et au CHANGELOG (relocaliser par titre de section) ; `:1637-1650` |
  | R3-8 | LOW | R | dépendance à la 15-11 absente ; pré-requis `log_bin` divergent de la 15-7b2 | 15-11 nommée (C-15-7-51) ; `assert_eq!(log_bin, 0, …)` nommant le pré-requis, sans sortie anticipée ni `SET GLOBAL` (Dev Notes) |
  | R3-9 | LOW | R | test 4 (i) : le patron cité n'appelle pas `seed_demo` | appel direct, patron `routes/onboarding.rs:191` ; `onboarding_e2e.rs:581` patron de l'état attendu seulement |

  **Décisions de l'orchestrateur appliquées** : R3-1 = F3-1, R3-2 = F3-3, F3-2, tous les LOW — C-15-7-53.
  ⚠️ **Écart avec la décision 2** : l'orchestrateur tenait `admin-manual.tex:1905-1906` pour justes,
  « à ne pas toucher si le grep le confirme » ; le grep **ne le confirme pas** (`kesh`, non `kesh-api`) :
  les deux lignes rejoignent la correction (consigné en C-15-7-53). **Signal D5** (déclaré au Project
  Lead) : un MEDIUM après une P2 à MEDIUM ; R3-1 = F3-1 est un **recyclage d'une remédiation** — le
  `SAVEPOINT` posé par la P2 a rendu muette une mutation que la P2 n'a pas revue — **traité localement**
  (une variante de mutation, une note), **pas de découpage** : le défaut ne touche ni la conception ni un
  sixième module ; R3-2 = F3-3 est un fait recopié du manuel, distinct. **Règle de découpage** : cinq
  modules, inchangés — non franchie. **Fiche 15-7b2 non modifiée** (le montage `log_bin` y est déjà juste).
  La remédiation ne change pas la conception (texte d'AC, manuel, mutations, Dev Notes) : **P4 ciblée** à
  lancer, braquée sur ce commit. Propagation : grep de `logs kesh`, `kesh |`, `--tail=100 kesh`,
  `Cas laissés`, `n'a rien écrit`, `api_keys' `, `TABLE_NAME <>`, `deux instances`, `au même instant`,
  `1638`, `log_bin`, `Retour arrière`, `Corrigé`, `onboarding_e2e.rs:581`, `is_savepoint_lost`,
  `company_referencing_columns(conn` sur les fiches 15-7*, l'index, le registre et `sprint-status.yaml` ;
  les occurrences restantes sont historiques (Change Logs P1/P2, registre) ou justes. Recompte : **6 AC**,
  **7 tâches**, **6 tests** (1 à 6), **14 mutations** numérotées (dont la 7 et la 9 en deux et trois
  variantes ; la 11 sans test, écrite), **1 action** d'audit, **5 modules**.
- **2026-10-08 — Validation P4 ciblée (Haiku, une lentille, prompt `15-7b3-validate-prompt-p4-ciblee.md`) :
  1 MEDIUM et 1 LOW rapportés, tous deux écartés par l'orchestrateur.** Rapport : `target/gate-logs/15-7b3-p4-ciblee.md`.
  Le MEDIUM (« les quatre lignes `docker compose logs kesh` du manuel admin ne sont pas corrigées par
  `2bc8dc32` ») est une **erreur de catégorie** : une remédiation de spec ne touche pas le manuel ; la correction
  de `:1905`, `:1906`, `:2058`, `:2063` est prescrite à T6 (vérifié : `grep -n ":1905" …` sur cette fiche) et sera
  faite au développement. Le LOW porte sur le message du commit `2bc8dc32` (13 mutations annoncées, 14 réelles) :
  un message de commit publié ne se réécrit pas ; le Change Log, qui fait foi, dit 14. Les autres axes de la passe
  (mutation 9 `bank_profiles` en CASCADE, erreurs sous `SAVEPOINT`, propagation) sont déclarés exercés et sans
  écart. **Validation CLOSE.** Trend : P1 1 HIGH / 6 MEDIUM → P2 4 MEDIUM → P3 2 MEDIUM (recyclage, signal D5)
  → P4 ciblée 0 retenu. Modèles : Opus ×2, Sonnet ×2, Opus ×2, Haiku (ciblée).
- **2026-10-08 — Coordination avec la 15-11a (C-15-7-55)** : `admin-manual.tex:1704-1717` (§ *Procédure de mise à jour standard*) est une **zone partagée** : la 15-11a y réécrit le point 3 (consigne datée, deux `lstlisting`, encadré « Relisez votre `.env` »). Ordre de merge : **15-11a d'abord**. Au rebase, la 15-7b3 se **relocalise par le texte** (`\subsection{Procédure de mise à jour standard}`, l'item « Vérifier les logs », `\subsection{Rollback en cas d'échec}`) et place son texte **après** l'énumération et l'encadré de la 15-11a, **sans les réécrire** ; `:1734-1749` (§ Rollback) est décalé par l'insertion ; le PDF est régénéré (`make fr`) par celle qui merge en second. Toute recette de redémarrage de cette fiche dit `docker compose up -d` (`restart` ne relit pas `.env`) : la seule, du *Dépannage* (T6, ligne `:2054`), corrigée ; les autres occurrences de « redémarrage » décrivent le comportement du serveur, non une recette.
- **2026-10-09 — Développement (Opus 5.5, worktree `kesh-15-7b3`, base `e892dcfa`).** En-tête
  `Status: backlog` corrigé en `ready-for-dev` (validation close, P4 ciblée — le sprint-status le
  disait déjà) puis `in-progress` au démarrage du développement. T0 écrit au Dev Agent Record (E1 à
  E12) avant tout code.
- **2026-10-09 — Développement achevé, statut `review`.** Commits `9710a01e` (T0), `059dc9a7` (code et
  tests), `d7f4c1f3` (documentation). Gates sur `d7f4c1f3` : backend 3215/3215 (4 ignorés), Vitest
  1164/1164, E2E 239 passés / 15 échecs = 7 KF-029 + 8 KF-053 (#478, verts rejoués seuls). 16/16
  mutations rouges. 18 tests neufs. Choix C-15-7b3-1 à 3. `origin/main` n'avait pas avancé au gate
  final (`git log HEAD..origin/main` vide). Revue de code non lancée.

- **2026-10-09 — Revue de code P1** (Sonnet ×3, prompt `38b830bf` ; rapports
  `kesh-gate-logs/15-7b3-review-p1-{B,E,A}.md`). Bruts : B 2 MEDIUM / 5 LOW, E 0 MEDIUM / 5 LOW, A 1 MEDIUM /
  6 LOW ; distincts : **3 MEDIUM** (B-1, B-2, A1), **14 LOW** (B-4 = E3 = A6 fusionnés). Remédiation
  `d38aeb45`, sur décisions de l'orchestrateur (C-15-7b3-4, C-15-7b3-5). **Elle touche du code de
  production** : la garde E1 (`company_referencing_columns` refuse une liste vide) et le retrait des deux
  aides de `kesh_db::test_fixtures` (compilé avec la production) ; le reste est tests et textes.

  | finding | sévérité | sort |
  |---|---|---|
  | B-1 | MEDIUM | test `repair_on_restore_keeps_superfluous_stubs` ; mutation 15 rouge |
  | A1 | MEDIUM | test 2 bis `full_import_is_undone_when_the_repair_fails` (import en 500, installation d'avant intacte) ; mutation 16 rouge ; le manuel garde « tout ou rien » |
  | B-2 | MEDIUM | Dépannage : `docker compose restart kesh-api` (chaque démarrage rejoue la réparation ; `up -d` ne redémarre pas un conteneur inchangé) ; les recettes `up -d` des 15-7b2/15-11a (recharger `.env`) intactes |
  | B-4 = E3 = A6 | LOW | aides de montage dans `crates/kesh-db/tests/support/installations_atteintes.rs`, incluses par `#[path]` ; `test_fixtures.rs` revenu à sa version de `e892dcfa` (C-15-7b3-4) |
  | E1 | LOW | garde de liste vide dans `company_referencing_columns` ⇒ `Invariant` ; test `company_referencing_columns_refuses_an_empty_answer` ; mutation 17 rouge |
  | B-3 | LOW | doc-comments exacts : sur base sans société, la sérialisation passe par `users` et `api_keys` (raisonné, non testé) |
  | B-5 | LOW | journal : chaque mutation nomme désormais ses tests rouges ; verdict « ROUGE » = présence de `FAIL [`, distinct de « NE COMPILE PAS » ; la 11 est écrite au journal comme non jouée (sans test, choix de la fiche), la 15 en est l'équivalent testé |
  | B-6 | LOW | accepté tel quel : `startup_keeps_a_stub_referenced_by_an_api_key` prouve le comportement, non la garde (son doc-comment le dit) ; la garde est prouvée par `repair_keeps_every_referenced_stub` (CASCADE) |
  | B-7 | LOW | manuel et CHANGELOG : « le journal d'audit excepté, qui n'en retient que le numéro » |
  | A2 | LOW | **chemin non testé, écrit** : l'échec du `ROLLBACK TO SAVEPOINT` (1213/1205 ayant déjà annulé la transaction) et les erreurs de `SAVEPOINT`/`RELEASE` ne sont exercés par aucun test ni aucune mutation — provoquer un interblocage réel au milieu de la boucle n'a pas de montage simple |
  | A3, A4 | LOW | Completion Notes rectifiées (formulation des huit échecs KF-053 ; bornes 22 → 29) |
  | A5 | LOW | écart de montage écrit : la variante « sans administrateur actif » garde A1, **inactif**, au lieu de ne monter que U et A0 ; l'acteur attendu (U) est le même, et la mutation 7a rougit par le test principal |
  | A7 | LOW | ligne de la réparation à la table des verrous de `MULTI-TENANT-SCOPING-PATTERNS.md` ; brochure PDF remise à sa version d'`origin/main` ; la réparation sort des « routes à verbe mutant » au § *Journal d'audit* |
  | E2 | LOW | **angle mort écrit** : le démarrage ne prend pas `_kesh_version FOR UPDATE` ; un démarrage concurrent d'un import peut interbloquer (victime possiblement l'import, qui s'annule et se relance). Écrit aussi à la table des verrous |
  | E4 | LOW | **angle mort écrit** : 29 `EXISTS` par société provisoire sous verrou ; une installation à des centaines de stubs (#542) allonge un seul démarrage, puis converge |
  | E5 | LOW | couvert en partie (échec à la restauration : test 2 bis ; archive à plusieurs sociétés : test B-1) ; restent **écrits** : stub désigné par une table CASCADE au niveau du démarrage (prouvé au niveau dépôt), et le chemin `ROLLBACK TO` en échec (A2) |

  **Gates sur `d38aeb45`** (bases reconstruites avant) : backend `scripts/test-fast.sh` **3218/3218**, 4
  ignorés (3215 + 3 tests neufs) ; frontend check 0 erreur, lint-i18n PASS, Vitest **1164/1164**, build vert ;
  **E2E complet 245 passés, 9 échecs, 19 ignorés** = 7 KF-029 + `reminders:49` et `vat-rates:55`, tous deux en
  `waiting for #username`, traces à `ERR_NETWORK_CHANGED` (26 et 12), **verts rejoués seuls** (2/2) : KF-053
  (#478). **Mutations : 19 jouées, 19 rouges** (les 16 d'origine + 15, 16, 17), chacune avec son test rouge nommé
  (`kesh-gate-logs/157b3-mutations.log`, `157b3-mutations-p1.out`) ; la 12 a dû être rejouée seule, son motif
  ayant changé avec la garde E1. PDF d'administration régénéré, contrôlé aplati (cinq phrases-témoins) ; 54
  `Overfull`, inchangé. Propagation : grep de `up -d`, `restart`, `aucune donnée ne désigne`,
  `test_fixtures::rendre`, `test_fixtures::poser`, `verbe mutant` sur le dépôt ; occurrences restantes justes.
  `origin/main` n'a pas avancé (pas de rebase).
