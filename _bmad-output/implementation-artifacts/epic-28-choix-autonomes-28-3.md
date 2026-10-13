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
