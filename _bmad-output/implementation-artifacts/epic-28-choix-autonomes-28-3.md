# Epic 28 — registre des choix autonomes, Story 28-3 (à verser par union dans `epic-28-choix-autonomes.md`)

*Registre tenu par union (jamais par écrasement). Chaque choix : contexte, option retenue, alternatives, réversibilité.
Fiche : `28-3-sauvegarde-instantane-commun.md`. Choix faits à la création de la fiche (2026-10-13), en autonomie, sur
`6f971d76` (rétrospective de l'Epic 27), branche `story/28-3`. Le fichier d'epic du kickoff
(`epic-28-recette-v0-13-2.md`, worktree `kesh-28-0`) n'était pas encore mergé : lu en place.*

---

## Story 28-3 — La sauvegarde lit ses tables dans un instantané commun (#633)

### C-28-3-1 — #465 (export de souveraineté) hors de la story : même outil, autre story

- **Contexte** : l'epic demande de trancher le sort de #465, « le traiter dans la même story avec le même utilitaire
  (un bug de plus fermé) ou l'en séparer par écrit ». Au sol, `build_global_export`
  (`crates/kesh-api/src/exports/global.rs:248`) fait **29** lectures (`grep -c "(pool, company_id"` → 29) dans
  **24** modules de dépôt distincts, toutes déclarées `pool: &MySqlPool` ; deux d'entre elles,
  `company_invoice_settings::get_or_create_default` et `company_dunning_settings::get_or_create_default`, **écrivent**
  (création paresseuse) et ouvrent leur propre transaction (`pool.begin()`, `company_invoice_settings.rs:72`,
  `company_dunning_settings.rs:40`) — incompatibles avec un instantané en lecture seule, à réordonner. Plusieurs de
  ces fonctions (`accounts::list_by_company`, `contacts::list_by_company`, `bank_accounts::list_by_company`,
  `projects::list_by_company`) servent aussi les écrans.
- **Retenu** : **séparer**. La 28-3 ferme #633 et livre l'outil réutilisable (`kesh_db::snapshot::ReadSnapshot`) ;
  #465 reste ouvert, référencé (`refs #465`, jamais `closes`), et le commentaire de `global.rs` nomme l'outil. Motif
  décidable : le § *Règle de splitting préventif* (plus de cinq modules) se déclencherait d'office — 24 modules de
  dépôt —, et le précédent du dépôt prescrit alors une story-zéro qui pose le patron (celle-ci) suivie d'une story de
  rollout mécanique. **Proposition à l'orchestrateur** : une story 28-3b (ou une story d'un epic ultérieur) « l'export
  de souveraineté lit dans un instantané », rollout de `ReadSnapshot` aux 29 lectures (signatures génériques sur
  `sqlx::Executor`, qui laissent compiler les appelants `&pool` existants) et réordonnancement des deux créations
  paresseuses **avant** l'ouverture de l'instantané. Elle fermerait un bug de plus pour la règle du décompte.
- **Écartées** : inclure #465 ici (24 modules, deux écrivains à réordonner, rayon d'impact sur les écrans) ; ne rien
  dire (l'epic demande une trace écrite).
- **Réversible** : oui — l'outil est conçu pour le rollout.

### C-28-3-2 — Mécanisme : `SET TRANSACTION … REPEATABLE READ` puis `START TRANSACTION WITH CONSISTENT SNAPSHOT, READ ONLY`, une connexion

- **Contexte** : MariaDB 10.11, toutes tables InnoDB (C-28-3-9). Il faut que toutes les lectures voient le même état,
  sans bloquer les saisies.
- **Retenu** : sur une `PoolConnection`, `SET TRANSACTION ISOLATION LEVEL REPEATABLE READ` (transaction suivante
  seulement), puis `sqlx::Transaction::begin(conn, Some("START TRANSACTION WITH CONSISTENT SNAPSHOT, READ ONLY"))`.
  L'isolation est **forcée** parce qu'une configuration serveur en `READ COMMITTED` rendrait `WITH CONSISTENT SNAPSHOT`
  sans effet, en silence ; `WITH CONSISTENT SNAPSHOT` fixe l'instant à l'ouverture ; `READ ONLY` fait refuser toute
  écriture par le serveur. Fin par `ROLLBACK`.
- **Écartées** : `pool.begin()` simple (isolation non forcée, instant fixé à la première lecture, pas de lecture
  seule — le précédent `routes/journal_entries.rs:474` s'en contente pour un écran, non pour une sauvegarde) ;
  `LOCK TABLES … READ` ou `FLUSH TABLES WITH READ LOCK` (bloquent toutes les saisies pendant la sauvegarde, le second
  exige le privilège `RELOAD`) ; `SET SESSION TRANSACTION` (laisserait le niveau sur une connexion du pool : AC5 (f)) ;
  un `START TRANSACTION` brut par `execute` (sqlx ne compterait pas la transaction et rendrait la connexion ouverte).
- **Réversible** : oui.

### C-28-3-3 — Un type `ReadSnapshot` dans un module neuf `kesh_db::snapshot`, exigé par les lectures de sauvegarde

- **Contexte** : la consigne demande que **toutes** les lectures passent par la connexion de l'instantané, inventaire
  clos à l'appui. Un inventaire écrit se périme ; un type ne se périme pas.
- **Retenu** : `export_table`, `non_generated_columns`, `read_min_required`, `read_instance_id` prennent
  `&mut ReadSnapshot` : un appel sur le pool ne compile plus. Module à part (et non dans `backup.rs`) parce que l'outil
  est destiné à l'export global (C-28-3-1), qui n'a rien à faire de `backup`.
- **Écartées** : `&mut MySqlConnection` (n'atteste pas qu'on est dans un instantané) ; un `Transaction<'_, MySql>` nu
  (idem) ; une garde lexicale sur le code source (contournable, et redondante avec le type).
- **Réversible** : oui.
- *Amendé par C-28-3-15 (validation P1, F1/R6) : « un appel sur le pool ne compile plus » ne vaut que pour les quatre
  lectures existantes ; la garde lexicale écartée ici est reprise, en complément du type, pour les deux fonctions qui
  tiennent un pool.*

### C-28-3-4 — `kesh-db` lit toute l'installation en une fonction ; `build_keshbackup` garde nom et signature

- **Contexte** : aujourd'hui, `kesh-api` orchestre la lecture table par table (`export.rs:46-61`).
- **Retenu** : `kesh_db::backup::export_installation(pool) -> InstallationExport` ; `build_keshbackup` n'appelle plus
  que cette fonction, puis assemble le ZIP. Nom et signature de `build_keshbackup` conservés : la garde lexicale
  `avant_sauvegarde_branchee_aux_appels_de_l_import` (`routes/admin.rs:880`) les compte, et le handler d'export comme
  l'import n'ont rien à changer.
- **Écartées** : passer le `ReadSnapshot` à `kesh-api` (l'instantané franchirait la frontière du crate et sa durée
  dépendrait du code de compression) ; renommer `build_keshbackup` (casse la garde sans gain).
- **Réversible** : oui.

### C-28-3-5 — Tout lire, clore l'instantané, puis compresser

- **Contexte** : aujourd'hui chaque table est compressée dans le ZIP aussitôt lue, et son NDJSON libéré.
- **Retenu** : lire les 39 tables dans l'instantané, le **clore**, puis assembler le ZIP. La transaction ne dure que
  le temps des lectures. Coût : le pic mémoire passe d'« une table non compressée + le ZIP » à « toutes les tables non
  compressées + le ZIP » — quelques Mio pour une installation de PME, l'ordre de grandeur que vise Kesh.
- **Écartées** : compresser dans l'instantané par un puits synchrone passé à `kesh-db` (garde le profil mémoire mais
  allonge la transaction et mêle les erreurs ZIP aux erreurs de base) — à reprendre si une installation réelle
  rendait le pic gênant.
- **Réversible** : oui.

### C-28-3-6 — Le test : un observateur dans le chemin réel, non une sonde sur `processlist` ni un `LOCK TABLES`

- **Contexte** : il faut une écriture concurrente **entre deux lectures**, déterministe, sans `sleep`, et un test qui
  rougisse sur le code actuel.
- **Retenu** : la fonction privée de lecture prend un observateur asynchrone appelé après l'ouverture et après chaque
  table ; la production lui passe un observateur vide. Le test y fait la suppression **par l'écrivain réel**
  (`bank_imports::delete_unreconciled_in_tx`) sur une autre connexion. Déterministe par construction : la lecture
  suivante n'est émise qu'après le retour de l'observateur. Aucun drapeau ni `feature`.
- **Écartées** : bloquer la lecture de `bank_imports` par un `LOCK TABLES bank_imports WRITE` tenu par le test et
  sonder `information_schema.PROCESSLIST` jusqu'à voir la sauvegarde en attente de verrou de métadonnées — faisable,
  mais la suppression faite **sous** `LOCK TABLES` ne peut toucher `bank_transactions` (cascade) sans la verrouiller
  aussi, ce qui bloquerait la lecture précédente : le scénario devrait couper les clés étrangères et cesserait d'être
  celui de l'écrivain réel ; une sonde à délai (proscrite).
- **Réversible** : oui.

### C-28-3-7 — L'assertion du test : aucune ligne orpheline, sur toutes les clés étrangères du schéma

- **Contexte** : le scénario est une suppression d'import ; le défaut vaut pour toute paire parent/enfant.
- **Retenu** : un helper lit les clés étrangères dans `information_schema.KEY_COLUMN_USAGE` et contrôle chacune sur
  le NDJSON produit. L'assertion vaut pour tout scénario futur sans être réécrite ; son assertion de montage (elle voit
  `fk_bank_transactions_import`) la garde d'être muette.
- **Écartées** : un contrôle ad hoc `bank_transactions.import_id ∈ bank_imports.id` (vrai, mais ne survit pas au
  scénario suivant).
- **Réversible** : oui.

### C-28-3-8 — DDL concurrente : angle mort assumé, fermé par construction

- **Contexte** : une DDL pendant la sauvegarde changerait la structure lue.
- **Retenu** : non testée. Kesh n'exécute de DDL qu'au démarrage, avant d'écouter (`main.rs:149`, `:382`) ; un second
  processus qui migrerait la même base attend, sur les tables déjà lues, la fin de la sauvegarde (verrou de
  métadonnées tenu jusqu'à la fin de la transaction), et fait échouer la lecture des autres
  (`ER_TABLE_DEF_CHANGED`) : la sauvegarde échoue, elle ne ment pas. Écrit dans les Dev Notes.
- **Écartées** : un test de DDL concurrente (scénario hors de tout usage de Kesh, coûteux à rendre déterministe).
- **Réversible** : oui.

### C-28-3-9 — Une garde de moteur : toutes les tables sauvegardées en InnoDB

- **Contexte** : un instantané InnoDB ne couvre pas une table MyISAM, Aria ou MEMORY. Au sol, 40 tables sur 40
  (39 de `TABLES_TO_TRUNCATE` + `_kesh_version`) sont InnoDB (squash : 40 `CREATE TABLE`, 40 `ENGINE=InnoDB`).
- **Retenu** : un test (AC5 (e)) lit `information_schema.TABLES.ENGINE` pour chacune et nomme la fautive. C'est
  l'hypothèse dont dépend toute la promesse : elle se contrôle, elle ne se suppose pas.
- **Écartées** : se fier au squash (il dit l'état d'aujourd'hui, non celui de la prochaine migration).
- **Réversible** : oui.
- *Amendé par C-28-3-17 (validation P1, F2/R11) : le contrôle passe à l'exécution, dans l'instantané, et fait
  échouer la sauvegarde ; le test seul ne contrôlait que la base de test.*

### C-28-3-10 — La restauration n'est pas touchée

- **Contexte** : la consigne demande de le confirmer.
- **Retenu** : confirmé au sol (fiche, AC9) : la restauration écrit dans une transaction unique sous le verrou
  d'installation ; ses lectures hors transaction ne portent que sur le schéma. Aucune modification de
  `restore_tables_in_tx`, `post_restore.rs`, `import.rs`.
- **Réversible** : sans objet.

### C-28-3-11 — Pré-import : l'instantané se prend sur une seconde connexion, après le verrou

- **Contexte** : `run_backup_and_restore` tient `FOR UPDATE` sur `_kesh_version` dans `tx`, puis appelle
  `build_keshbackup`.
- **Retenu** : inchangé dans l'ordre ; l'instantané, ouvert **après** le verrou, voit tout ce qui était validé avant
  lui ; sa lecture de `_kesh_version` est non verrouillante, donc sans attente sur le `FOR UPDATE` (comme aujourd'hui).
  Lire la sauvegarde **dans** `tx` a été écarté : `tx` est en lecture-écriture et sert la restauration, et sa vue
  serait fixée par la première lecture ordinaire, après le verrou — même instant, sans lecture seule ni isolation
  forcée.
- **Réversible** : oui.

### C-28-3-12 — Documentation : la garantie au manuel d'administration, la limite de l'export global écrite des deux côtés

- **Contexte** : le manuel d'administration dit l'export « en lecture seule (aucun effet de bord) » et rien de sa
  cohérence ; le manuel utilisateur ne dit rien de celle de l'export global.
- **Retenu** : la garantie au § *Exporter (sauvegarde complète)*, avec le conseil de refaire une sauvegarde
  antérieure prise pendant une saisie ; une phrase sur l'export global (#465) dans les deux manuels, puisque la
  garantie neuve rend la différence trompeuse si elle n'est pas dite. CHANGELOG `### Corrigé` sous
  `## [0.13.3] — Non publié`.
- **Écartées** : se taire sur #465 (le lecteur généraliserait la garantie).
- **Réversible** : oui.
- *Amendé par C-28-3-18 (validation P1, R2) : la phrase « aucun effet de bord » est fausse au code et se remplace.*

### C-28-3-13 — Ni migration, ni changement de format, ni d'écran

- **Retenu** : aucun fichier de migration, `min_required` inchangé, format `.keshbackup` identique (un fichier 0.13.3
  s'importe dans une 0.13.2) ; aucun texte i18n. P5, P7, P8 non concernés. L'E2E complet tourne quand même au dernier
  commit de code (règle D7, sans exception).
- **Réversible** : sans objet.

### C-28-3-14 — Une seule passe de validation préparée, Sonnet, lentilles R et F

- **Contexte** : rotation D6 (Sonnet ↔ Opus pour les passes complètes) ; la fiche est écrite par Opus.
- **Retenu** : P1 en **Sonnet**, deux lentilles (R : relecture contre le code ; F : faisabilité, concurrence et
  sémantique MariaDB/sqlx), prompt versionné `28-3-validate-prompt-p1.md`. Les passes suivantes reviennent à
  l'orchestrateur.
- **Réversible** : oui.

---

## Remédiation de la validation P1 (2026-10-13, Opus 5.5)

*Rapports : `kesh-gate-logs/28-3-validate-p1-F.md` (1 MEDIUM, 6 LOW), `28-3-validate-p1-R.md` (2 MEDIUM, 10 LOW).*

### C-28-3-15 — La garantie de l'AC2 dite exactement : type, structure, deux gardes lexicales (F1/R6)

- **Contexte** : la fiche disait l'inventaire « tenu par le compilateur » et M10 « non représentable ». Faux : le
  constructeur `ReadSnapshot::begin(pool)` est public, si bien qu'un **second** instantané se compile (c'est la forme
  de M1), et une lecture neuve `sqlx::query(…).fetch_*(pool)` se compile partout où un pool est en portée. M10 compilait
  donc, et aucun test ne la tuait (`min_required` est constant pendant la vie du processus).
- **Retenu** : (1) le type garantit que les **quatre** lectures existantes ne s'appellent plus sur le pool, et la
  fiche ne dit plus rien de plus ; (2) le cœur `export_installation_in` ne reçoit **aucun pool** ; (3) deux gardes
  lexicales — `export_installation` et `build_keshbackup` portent chacune l'identifiant `pool` exactement deux fois
  (paramètre et unique appel), sur le modèle de `avant_sauvegarde_branchee_aux_appels_de_l_import`. M10 requalifiée
  (compile, tuée par la garde (ii)), M11 neuve (tuée par la garde (i)). Angle mort écrit : une donnée lue par un
  appelant et passée en paramètre.
- **Écartées** : rendre `ReadSnapshot::begin` `pub(crate)` (proposition de F1) — l'outil est destiné à l'export
  global (#465, autre module de `kesh-api`), qui doit pouvoir l'ouvrir ; et cela n'empêcherait pas la lecture neuve
  sur le pool. Retirer M10 (on perdrait la seule forme qui motive la garde (ii)).
- **Réversible** : oui (deux tests à retirer).

### C-28-3-16 — `ReadSnapshot` dégradé dès T1 : le même texte de test rougit puis verdit (R1, F6 (ii))

- **Contexte** : T1 écrivait le cœur « sur le pool » alors que le test devait ouvrir un `ReadSnapshot` qui n'existait
  qu'à T2 : le test de T1 et celui de T2 auraient été deux textes, et « rouge avant, vert après » n'aurait rien prouvé.
- **Retenu** : T1 pose le type avec son API définitive mais un intérieur **dégradé** (connexion du pool, aucune
  transaction ; lectures en autocommit) ; les quatre lectures, le cœur et `build_keshbackup` y sont branchés dès T1,
  pour que le workspace compile. (a-défaut), (a-RC) et (b) rougissent à T1 ; T2 ne change que l'intérieur du type ;
  preuve : `git diff` T1→T2 du module de test, vide sur ces trois tests.
- **Écartées** : un test à T1 sur `&MySqlPool` réécrit à T2 (deux textes) ; une `feature` qui bascule l'intérieur
  (proscrite par C-28-3-6).
- **Réversible** : oui.

### C-28-3-17 — Garde de moteur à l'exécution, dans l'instantané, échec fermé (F2/R11)

- **Contexte** : le test (e) d'origine lisait la base de test (moteurs du squash). 13 des 51 `CREATE TABLE` des
  migrations n'ont pas de clause `ENGINE` et prennent, en exploitation, le moteur par défaut du serveur, qu'aucun
  fichier du dépôt ne fixe. Une table non transactionnelle échapperait à l'instantané sans signal.
- **Retenu** : première lecture du cœur, `information_schema.TABLES` (`TABLE_NAME`, `ENGINE` en `Option<String>`) ;
  `DbError::Invariant` nommant la table si une table sauvegardée n'est pas InnoDB, ou manque. Tests (e1) (compte =
  `TABLES_TO_TRUNCATE.len() + 1`) et (e2) (`ALTER TABLE _kesh_version ENGINE=MyISAM` sur la base éphémère : la
  sauvegarde échoue en nommant la table). Conséquence voulue : sur une telle base, l'import est refusé
  (`AdminPreImportBackupFailed`). Manuel et CHANGELOG le disent. Une requête de plus (81 dans l'instantané).
- **Écartées** : écrire la limite aux seules Dev Notes (l'utilisateur ne la verrait jamais) ; journaliser sans
  échouer (le fichier resterait incohérent, en silence).
- **Réversible** : oui.

### C-28-3-18 — Le manuel ne dit plus « aucun effet de bord » (R2)

- **Contexte** : `admin-manual.tex:1825` (PDF identique) promet un export « en lecture seule (aucun effet de bord sur
  les données) » ; le handler `full_export` écrit, après le fichier, une entrée `admin.full_export` au journal d'audit
  (`routes/admin.rs:44`, `:653-692`, best-effort). L'AC10 greffait la garantie neuve sur cette phrase.
- **Retenu** : la phrase est remplacée — l'export ne modifie aucune donnée comptable ni aucun réglage ; il ajoute
  seulement une entrée d'audit, que le fichier, pris avant elle, ne contient pas. Contrôle : `grep -c "aucun effet de
  bord"` → `0` sur le `.tex` et sur le PDF aplati.
- **Écartées** : laisser la phrase (défaut antérieur à la story, mais la story réécrit ce paragraphe) ; déplacer
  l'audit avant la production du fichier (changement de comportement hors périmètre).
- **Réversible** : oui.

### C-28-3-19 — Ouverture de l'instantané : protocole texte, `close_on_drop`, dépendance `#[doc(hidden)]` épinglée (F4, F5/R8)

- **Contexte** : `sqlx::query("SET TRANSACTION …")` passerait en protocole préparé, dont l'acceptation par MariaDB pour
  cette instruction n'est pas établie ; un échec entre le `SET` et le `START` rendrait au pool une connexion portant le
  niveau « pour la prochaine transaction » ; `Transaction::begin` est `#[doc(hidden)]`.
- **Retenu** : `conn.execute("SET TRANSACTION …")` sur un `&str` (protocole texte, comme le `BEGIN` de sqlx) ; sur
  échec du `SET`, `conn.close_on_drop()` avant de rendre l'erreur. Sur échec de `Transaction::begin`, la connexion a
  été cédée à sqlx : **résidu assumé** — l'échec d'un `START TRANSACTION` après un `SET` réussi est, en pratique, une
  connexion rompue, que le `ping` de `return_to_pool` écarte ; sinon, la prochaine transaction de cette connexion est
  forcée en REPEATABLE READ — sans effet si c'est le niveau du serveur (le défaut de MariaDB), réel s'il a été réglé
  en READ COMMITTED. Commentaire de dépendance dans `snapshot.rs` et Dev Notes § *Risque de
  montée de sqlx* (alternatives documentées examinées et écartées : `Connection::begin_with` emprunte la connexion,
  `Pool::begin_with` en prend une neuve).
- **Écartées** : `close_on_drop` systématique (chaque sauvegarde fermerait sa connexion ; (d) et (f) perdraient leur
  objet) ; une instruction multiple `SET …; START …` dans `Transaction::begin` (dépend du drapeau multi-instructions du
  client, non établi).
- **Réversible** : oui.

### C-28-3-20 — Une erreur du `ROLLBACK` final fait échouer la sauvegarde (R12)

- **Contexte** : la fiche ne disait pas si une erreur de `finish`, après lecture complète, devait faire échouer
  l'export.
- **Retenu** : elle se propage (`AdminFullExportFailed`, ou refus de l'import en pré-import). Les données lues sont
  cohérentes, mais une erreur de base en fin de sauvegarde signale une connexion en mauvais état ; l'administrateur
  relance. Échouer fermé est la règle du reste de la story.
- **Écartées** : journaliser et rendre les données (plus permissif, gain nul pour une opération rare et relançable).
- **Réversible** : oui.

### C-28-3-21 — L'observateur a la forme `BoxFuture`, alias local, sans dépendance neuve (F6 (i))

- **Contexte** : la fiche laissait `AsyncFnMut` « au choix du dev » ; dans un futur qui doit être `Send` (handler
  axum), l'inférence d'auto-trait y est fragile. `kesh-db` ne dépend pas de `futures`.
- **Retenu** : `&mut (dyn FnMut(Etape) -> BoxFuture<'static, ()> + Send)`, `BoxFuture` alias local
  `Pin<Box<dyn Future<Output = ()> + Send + 'a>>` ; observateur vide en production.
- **Écartées** : `AsyncFnMut` ; ajouter `futures` à `kesh-db` pour un alias.
- **Réversible** : oui.

### C-28-3-22 — `@@in_transaction`, et sinon `INNODB_TRX` : le test tranche (F7)

- **Contexte** : `@@in_transaction` n'est utilisé nulle part dans le dépôt ; son existence en MariaDB 10.11 n'est pas
  établie sans exécuter une requête.
- **Retenu** : AC5 (d) lit `@@in_transaction` ; si le serveur le refuse, repli sur
  `SELECT COUNT(*) FROM information_schema.INNODB_TRX WHERE trx_mysql_thread_id = CONNECTION_ID()` ; (d1) et (d2)
  lisent une table avant de rendre la connexion, pour que le repli reste discriminant sous M6. Le Dev Agent Record
  écrit quelle forme a servi.
- **Réversible** : oui.

### C-28-3-23 — PDF des manuels au rebase : régénérés, jamais fusionnés (R9)

- **Contexte** : 28-1, 28-2 et 28-3 avancent en parallèle ; les PDF sont des binaires.
- **Retenu** : au rebase, union sur le `.tex`, PDF repris de `main` puis régénérés (`make fr`, sous `flock`), contrôle
  aplati refait.
- **Réversible** : sans objet.

### C-28-3-24 — Validation P2 en Opus, lentilles R et F, axe premier sur les régressions de la remédiation

- **Contexte** : rotation D6 (Sonnet ↔ Opus pour les passes complètes) ; P1 était en Sonnet ; la remédiation a
  touché AC1 à AC5, AC7, AC8, AC10, AC11, les tâches et les mutations — trop large pour une passe ciblée.
- **Retenu** : passe complète **Opus**, deux lentilles, mêmes axes et interdits que P1, plus un axe premier : les
  régressions introduites par cette remédiation. Prompt versionné `28-3-validate-prompt-p2.md`.
- **Réversible** : oui.
