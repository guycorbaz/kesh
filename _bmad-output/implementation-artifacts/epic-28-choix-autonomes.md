# Epic 28 — registre des choix autonomes

*Registre tenu par union (jamais par écrasement). Chaque choix : contexte, option retenue, alternatives, réversibilité.
Fichier d'epic : `planning-artifacts/epic-28-recette-v0-13-2.md`. Consignes : `docs/agents-orchestration.md`.*

---

## Story 28-0 — Kickoff et actions reportées (doc seulement)

Choix faits au kickoff (2026-10-13), en autonomie, sur `6f971d76` (rétrospective de l'Epic 27).

### C-28-0-1 — Les consignes des agents dans un document propre, non dans `docs/testing.md`

- **Contexte** : Q5 de la rétrospective de l'Epic 27 (B6 de l'Epic 26, A8 de l'Epic 15) — verser au dépôt les
  consignes des agents et le verrou de gate. Deux lieux possibles : une section de `docs/testing.md`, ou un document
  à part.
- **Retenu** : `docs/agents-orchestration.md`. Les consignes portent pour moitié sur autre chose que les tests —
  autonomie et registre des choix, interdits (`git push`, `gh`, `prepare-release.sh`), lentilles de revue, édition
  du code, données réelles — et s'adressent à un autre lecteur (l'orchestrateur qui rédige un prompt d'agent).
  Ce qui relève des tests y **renvoie** à `docs/testing.md` (recette E2E, base jetable) plutôt que de le recopier.
  Le `CLAUDE.md` y renvoie en une ligne, § *Plafonds mémoire*, là où le verrou prolonge `mem-guard.sh`.
- **Écartées** : section de `docs/testing.md` (le document grossirait d'un sujet qui n'est pas le sien) ; `_bmad/`
  (réservé au cadre BMAD installé, réécrit par une réinstallation).
- **Réversible** : oui (déplacement de fichier).

### C-28-0-2 — Aucun chemin personnel : le répertoire des journaux nommé `$GATE_LOGS`

- **Contexte** : la consigne demande de ne garder aucun chemin personnel inutile. Le verrou vit dans
  `kesh-gate-logs/.gate.lock`, répertoire voisin des worktrees sur la station.
- **Retenu** : convention `$GATE_LOGS` définie en tête du document (« voisin de `kesh/` et des `kesh-<clé>/` ») ;
  le `CLAUDE.md` écrit `<kesh-gate-logs>/.gate.lock`. Les références à la mémoire de l'orchestrateur, au registre
  « epic-15 » et au fichier de reprise de l'Epic 26 sont généralisées (`epic-<N>-choix-autonomes.md`).
- **Écartées** : le chemin absolu de la station (personnel, non portable) ; une variable d'environnement réellement
  lue par un script (aucun script ne prend le verrou aujourd'hui : ce serait du code, hors du périmètre doc).
- **Réversible** : oui.

### C-28-0-3 — B7 : la recette E2E du `CLAUDE.md` est un défaut, corrigé par recopie intégrale de `docs/testing.md`

- **Contexte** : `CLAUDE.md:180` portait `KESH_JWT_SECRET='<32+ octets>'` et `KESH_ADMIN_PASSWORD='<12+ caractères>'`.
  `is_template_placeholder` (`crates/kesh-api/src/config.rs`) refuse au démarrage toute valeur de la forme `<…>` :
  recopiée, la recette ne démarre pas. Le bloc omettait aussi `KESH_INBOX_DIR`, `KESH_DOCUMENTS_DIR` et les quatre
  `KESH_SMTP_*` (cinq specs rouges hors sujet), et `KESH_TEST_MODE` côté runner.
- **Retenu** : c'est un défaut ; le bloc est remplacé par celui de `docs/testing.md` § *Prérequis Playwright local*,
  déclaré comme faisant foi, valeurs factices concrètes, avec une note datée qui dit pourquoi elles ne doivent pas
  redevenir des gabarits.
- **Écartées** : ne remplacer que les deux gabarits (la recette serait restée incomplète — c'est la recette du
  `CLAUDE.md` qui avait induit la troisième récidive du 2026-09-16) ; renvoyer seulement à `docs/testing.md` sans
  bloc (la section explique un piège sur ce bloc même).
- **Réversible** : oui.

### C-28-0-4 — B4 versé au `CLAUDE.md`, § *Recompter ses propres comptes rendus*

- **Contexte** : B4 de la rétrospective de l'Epic 27 prévoyait la consigne en mémoire, « puis au `CLAUDE.md` sur
  accord ». Guy a répondu à la Q5 que les actions reportées entrent dans cette release.
- **Retenu** : un paragraphe sous la règle existante (ne jamais réécrire l'entrée d'une passe passée ; corriger
  par annotation datée ce qui était faux à sa date), et la même consigne au § 3 de `docs/agents-orchestration.md`.
- **Écartée** : attendre un accord explicite sur le seul B4 (la Q5 couvre les actions reportées).
- **Réversible** : oui.

### C-28-0-5 — A-C9 (Epic 15) versé : la CI dépend de Docker Hub

- **Contexte** : amendement proposé, « à retirer si A2 est faite ». Vérifié au kickoff : `image: mariadb:10.11` dans
  `ci.yml:50` et `release.yml:21`, aucun `credentials`.
- **Retenu** : une phrase au § *Quand sauter*, avec sa condition de retrait.
- **Réversible** : oui.

### C-28-0-6 — « dix P1 de la TVA » ; la ligne datée du triage du 2026-10-08 reste à « 9 »

- **Contexte** : B6 de l'Epic 27. `gh issue list --milestone "Vague 2 — …" --state open --label P1` rend 10 (#390 à
  #397, #401, #525). Le grep de la valeur trouve aussi « 14 P1 (9 dans le jalon de la TVA) » au paragraphe « Point
  de départ, au triage du 2026-10-08 ».
- **Retenu** : le paragraphe du report passe à dix, avec la commande ; la ligne du triage reste à 9 — relevé daté,
  juste à sa date (B4).
- **Réversible** : oui.

### C-28-0-7 — Actions reportées non traitées par la 28-0, et pourquoi

Ne demandent pas qu'un commit de documentation, ou relèvent d'une décision qui n'est pas la mienne :

| Action | Sort | Raison |
|---|---|---|
| A-C2 (Epic 15) — D5 bis, une dérogation vaut pour une passe | **non versé** | change une règle de découpage ; la rétrospective de l'Epic 27 (§ 3.2) relève que D5 « se lit encore de deux façons » : à trancher par Guy, à la rétrospective de l'Epic 28 |
| A-C3, A-C4, A-C5 (Epic 15) | **versés dans `docs/agents-orchestration.md`** (§ 3, 4, 6), non au `CLAUDE.md` | ce sont des consignes d'orchestration ; le `CLAUDE.md` y renvoie |
| A-C7 / A9 (Epic 15) — #577, smoke E2E en CI | **listé** | décision de Guy (recréer le smoke ou réécrire le texte) ; le `CLAUDE.md` et `docs/ci.md` portent déjà le constat |
| A-C8 (Epic 15) — prémisse P1 de la *Migration breaking policy* | **listé** | fait à vérifier à l'exécution (`MIGRATOR.ignore_missing`, refus `VersionMissing`) avant de l'écrire ; candidat à la 28-4, qui ajoute une migration |
| A2 (Epic 15) — authentifier Docker Hub en CI | **listé** | modifie `ci.yml` / `release.yml` (code de CI), décision de Guy |
| A3 (Epic 15) — six `.svelte-kit/` versionnés à la racine | **listé** | retrait de fichiers + `.gitignore`, pas de la documentation ; candidat à un `chore` |
| A7 (Epic 15) — combler les comptes rendus | **listé** | lourd ; ne se fait que par annotations datées (C-28-0-4) ; lot dédié |
| A10 (Epic 15) — Rust 1.97.1 | **listé** | code et gate complet |
| A1, A6 (b, c), A12 (Epic 15) | **listé** | décisions ou réponses de Guy (choix de schéma, Synology, jalon de l'Epic 15) |
| A4 (Epic 15) ; B3 (Epic 26) / B5 (Epic 27) — priorités de #618 et #621, jalons de #602, #606, #612 | **listé, demandé à l'orchestrateur** | écritures `gh` (labels, jalons), interdites à l'agent ; #618 et #621 portent toujours `triage` sans priorité au kickoff |
| B2 (Epic 27) — recette visuelle au `CLAUDE.md` | **reporté à la rétrospective de l'Epic 28**, comme l'écrit l'action | appliquée dès cet epic par le fichier d'epic (§ *Les stories*) |
| B4 (Epic 26) / Q6 (Epic 27) — #530 | **fait par Guy** | fermée le 2026-10-13 |
| Q1 (Epic 26) / Q2 (Epic 27) — révisions de #620, #621 | **fait par Guy** | supprimées le 2026-10-13 |

- **Réversible** : oui (chaque ligne peut être reprise par une story ou un `chore`).

---

## Story 28-3 — La sauvegarde lit ses tables dans un instantané commun (#633)

*Versé par union le 2026-10-13 (remédiation de la validation P2) depuis `epic-28-choix-autonomes-28-3.md`, supprimé ensuite. Choix C-28-3-1 à 14 faits à la création de la fiche sur `6f971d76`, branche `story/28-3`, avant le merge du kickoff.*

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
- *Annotation du 2026-10-13 (validation P2, F2-9) : « elle ne ment pas » n'est établi que pour une DDL qui reconstruit la table ; une DDL `INSTANT` peut laisser lire la structure neuve — voir C-28-3-31.*

### C-28-3-9 — Une garde de moteur : toutes les tables sauvegardées en InnoDB

- **Contexte** : un instantané InnoDB ne couvre pas une table MyISAM, Aria ou MEMORY. Au sol, 40 tables sur 40
  (39 de `TABLES_TO_TRUNCATE` + `_kesh_version`) sont InnoDB (squash : 40 `CREATE TABLE`, 40 `ENGINE=InnoDB`).
- **Retenu** : un test (AC5 (e)) lit `information_schema.TABLES.ENGINE` pour chacune et nomme la fautive. C'est
  l'hypothèse dont dépend toute la promesse : elle se contrôle, elle ne se suppose pas.
- **Écartées** : se fier au squash (il dit l'état d'aujourd'hui, non celui de la prochaine migration).
- **Réversible** : oui.
- *Amendé par C-28-3-17 (validation P1, F2/R11) : le contrôle passe à l'exécution, dans l'instantané, et fait
  échouer la sauvegarde ; le test seul ne contrôlait que la base de test.*
- *Annotation du 2026-10-13 (validation P3, R3-1, F3-6) : l'amendement C-28-3-17 est lui-même retiré — le contrôle
  revient aux tests (AC5 (e), `test_schema_guard.rs`) et la limite passe au manuel ; voir C-28-3-25.*

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
- *Annotation du 2026-10-13 (validation P2) : la preuve par `git diff` est remplacée par `git log -L` par fonction — voir C-28-3-27.*

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
- *Annotation du 2026-10-13 (validation P2) : retiré de l'exécution — voir C-28-3-25.*

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
- *Annotation du 2026-10-13 (validation P3, R3-1, F3-6) : « épinglée » du titre est inexact — la dépendance est
  **signalée en commentaire**, non épinglée (`Cargo.toml` porte `0.8`, seul `Cargo.lock` tient la 0.8.6) ; voir
  C-28-3-30. Le protocole texte vaut aussi pour les `SET` émis par les tests (validation P3, F3-2).*

### C-28-3-20 — Une erreur du `ROLLBACK` final fait échouer la sauvegarde (R12)

- **Contexte** : la fiche ne disait pas si une erreur de `finish`, après lecture complète, devait faire échouer
  l'export.
- **Retenu** : elle se propage (`AdminFullExportFailed`, ou refus de l'import en pré-import). Les données lues sont
  cohérentes, mais une erreur de base en fin de sauvegarde signale une connexion en mauvais état ; l'administrateur
  relance. Échouer fermé est la règle du reste de la story.
- **Écartées** : journaliser et rendre les données (plus permissif, gain nul pour une opération rare et relançable).
- **Réversible** : oui.
- *Annotation du 2026-10-13 (validation P3, F3-6, F3-7) : « échouer fermé est la règle » vaut pour une erreur de base
  survenue dans l'instantané, non pour le moteur des tables, que la story choisit de ne pas contrôler à l'exécution
  (C-28-3-25). La propagation de l'erreur de `finish` est testée par un `KILL` de la connexion, ou écrite en angle
  mort si ce test s'avère non déterministe (fiche, AC1 ; tranché au Dev Agent Record).*

### C-28-3-21 — L'observateur a la forme `BoxFuture`, alias local, sans dépendance neuve (F6 (i))

- **Contexte** : la fiche laissait `AsyncFnMut` « au choix du dev » ; dans un futur qui doit être `Send` (handler
  axum), l'inférence d'auto-trait y est fragile. `kesh-db` ne dépend pas de `futures`.
- **Retenu** : `&mut (dyn FnMut(Etape) -> BoxFuture<'static, ()> + Send)`, `BoxFuture` alias local
  `Pin<Box<dyn Future<Output = ()> + Send + 'a>>` ; observateur vide en production.
- **Écartées** : `AsyncFnMut` ; ajouter `futures` à `kesh-db` pour un alias.
- **Réversible** : oui.
- *Annotation du 2026-10-13 (validation P3, R3-1, R3-6) : l'alias prescrit a deux paramètres,
  `BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>` (C-28-3-30) ; un observateur factorisé en
  variable porte l'annotation de retour `-> BoxFuture<'static, ()>`.*

### C-28-3-22 — `@@in_transaction`, et sinon `INNODB_TRX` : le test tranche (F7)

- **Contexte** : `@@in_transaction` n'est utilisé nulle part dans le dépôt ; son existence en MariaDB 10.11 n'est pas
  établie sans exécuter une requête.
- **Retenu** : AC5 (d) lit `@@in_transaction` ; si le serveur le refuse, repli sur
  `SELECT COUNT(*) FROM information_schema.INNODB_TRX WHERE trx_mysql_thread_id = CONNECTION_ID()` ; (d1) et (d2)
  lisent une table avant de rendre la connexion, pour que le repli reste discriminant sous M6. Le Dev Agent Record
  écrit quelle forme a servi.
- **Réversible** : oui.
- *Annotation du 2026-10-13 (validation P2) : le repli `INNODB_TRX` est retiré — voir C-28-3-26.*

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

## Remédiation de la validation P2 (2026-10-13, Opus 5.5)

*Rapports : `kesh-gate-logs/28-3-validate-p2-R.md` (3 MEDIUM, 5 LOW), `28-3-validate-p2-F.md` (2 MEDIUM, 9 LOW).
Signal D5 levé (P1 3 MEDIUM → P2 5 MEDIUM, dont 4 nés de la remédiation P1) ; décision de l'orchestrateur, sur
l'autonomie déléguée par Guy le 2026-10-13 : pas de découpage, remédiation par simplification.*

### C-28-3-25 — La garde de moteur quitte l'exécution : tests, et manuel (R2-1, R2-2, F2-10, R2-6 ; remplace C-28-3-17)

- **Contexte** : la garde de P1 faisait échouer fermé toute sauvegarde, donc tout import, dès qu'une table
  sauvegardée n'était pas InnoDB. L'installation réelle (NAS de Guy) tourne le paquet MariaDB de Synology, dont le
  moteur par défaut n'est pas vérifié ; deux tables (`contacts`, `contact_persons`, recompte de P2 : 2 des 40, non
  13 des 51) prennent ce défaut. Un refus permanent de sauvegarde et d'import y serait pire que le défaut gardé, et
  ses messages (« réessayez », « dossier inscriptible ») n'en disaient pas la cause.
- **Retenu** : aucun contrôle de moteur à l'exécution, pas même un avertissement journalisé (l'option la plus simple :
  pas de requête de plus dans l'instantané, pas de branche à tester ni de mutation survivante). Contrôle **dans les
  tests** : squash (`ENGINE=InnoDB` sur ses 40 tables), `test_schema_guard.rs` (moteur du vrai schéma = moteur du
  squash), AC5 (e) (toute table sauvegardée est InnoDB sur la base de test, et `export_installation` les rend
  toutes). **Manuel d'administration** : l'instantané ne vaut que pour les tables InnoDB ; vérification
  `SELECT TABLE_NAME, ENGINE FROM information_schema.TABLES WHERE TABLE_SCHEMA = DATABASE() AND (ENGINE IS NULL OR
  ENGINE <> 'InnoDB')` ; remède `ALTER TABLE … ENGINE=InnoDB` après un dump, et `default_storage_engine=InnoDB`.
  Vérification sur le NAS **demandée à l'orchestrateur**, en lecture seule, avant la release, non bloquante pour la
  story. M9 redéfinie (mutation du squash), M12 retirée, 80 requêtes dans l'instantané. Ferme R2-1 (plus rien ne
  peut bloquer l'installation réelle), R2-2 (plus d'échec fermé sans diagnostic : la limite et le remède sont au
  manuel), F2-10 (plus de promesse d'échec), R2-6 (plus de branche « table manquante » à l'exécution ; l'assertion de
  montage de (e) compte les tables).
- **Écartées** : garder l'échec fermé et livrer le diagnostic (R2-2) — la cause resterait un refus permanent sur une
  installation dont le moteur n'est pas connu ; un avertissement journalisé — un signal que personne ne lit, au prix
  d'une requête et d'un test ; exiger la requête sur le NAS **avant** la story — elle ne dépend pas du code.
- **Réversible** : oui (rétablir une garde, journalisée ou fermée, après vérification du NAS).

### C-28-3-26 — `@@in_transaction`, vérifié par le test ; repli sans privilège (F2-1 ; remplace C-28-3-22)

- **Contexte** : le repli `information_schema.INNODB_TRX` exige le privilège `PROCESS`, que l'utilisateur de dev
  `kesh` n'a pas (`reconciliation_e2e.rs:7244-7248`) ; la CI tourne en `root` : le test aurait passé en CI et échoué
  sur le poste de dev, dans le seul cas où il servait.
- **Retenu** : `SELECT @@in_transaction` → `0` ; le test vérifie lui-même que la variable existe en 10.11 (une
  erreur le fait échouer). À défaut, repli **sans privilège** : `SET TRANSACTION READ WRITE` sur la connexion
  réacquise doit réussir ; dans une transaction ouverte, le serveur le refuse en `1568`
  (`ER_CANT_CHANGE_TX_CHARACTERISTICS`, non vérifié, tranché sous M6). Le Dev Agent Record dit quelle forme a servi.
- **Écartées** : `INNODB_TRX` (privilège) ; `SHOW ENGINE INNODB STATUS` (privilège `PROCESS` aussi).
- **Réversible** : oui.

### C-28-3-27 — Preuve « même texte, rouge puis vert » par `git log -L` par fonction (F2-2 ; remplace la preuve de C-28-3-16)

- **Contexte** : « `git diff` T1→T2 restreint au module de test, vide » n'était pas exécutable : `mod tests` est un
  bloc de `backup.rs`, aucun pathspec ne l'isole, et T2 y ajoute des tests neufs ; « T2 ne change que l'intérieur »
  contredisait T2.
- **Retenu** : AC5 (a) et T2 disent que T2 change l'intérieur de `ReadSnapshot` côté production et ajoute des tests
  neufs, sans toucher les trois tests (a-défaut), (a-RC), (b) ni leurs helpers. Preuve : pour chacune de ces
  fonctions, `git log --oneline -L '/fn <nom>(/,/^    }$/:crates/kesh-db/src/backup.rs' <T1>..<T2>` → sortie vide,
  copiée ; plage contrôlée sur le commit T2. Forme à expressions plutôt que la forme `-L :<fonction>:` de la
  consigne de l'orchestrateur : le dépôt n'a pas de `.gitattributes` `*.rs diff=rust`, et le pilote par défaut, qui ne
  reconnaît qu'une ligne non indentée comme en-tête de fonction, étendrait la plage d'un test indenté jusqu'aux tests
  neufs de T2 — sortie non vide sur un texte inchangé.
- **Écartées** : `-L :<fonction>:` tel quel (faux rouge, ci-dessus) ; ajouter un `.gitattributes` (changement du dépôt
  hors périmètre, pour une preuve) ; déplacer les trois tests dans un fichier à part (ils appellent le cœur privé).
- **Réversible** : oui.

### C-28-3-28 — La promesse du manuel bornée aux saisies concurrentes (R2-3)

- **Contexte** : « le fichier ne porte jamais une ligne sans son en-tête » promettait plus que l'instantané : une base
  qui porte déjà des orphelins (restaurée depuis une sauvegarde incohérente d'avant la 0.13.3, chargée en
  `FOREIGN_KEY_CHECKS=0`) les recopie.
- **Retenu** : « une saisie concurrente ne peut plus y laisser une ligne sans son en-tête » ; « une base qui en porte
  déjà les garde, et l'export les reproduit » — manuel et CHANGELOG.
- **Écartées** : faire détecter les orphelins par l'export (changement de comportement hors périmètre).
- **Réversible** : oui.

### C-28-3-29 — Le cœur emprunte l'instantané ; `begin` passe par `begin_on` (R2-7, F2-7)

- **Contexte** : la fiche ne disait pas qui clôt l'instantané ni par où les tests atteignent le cœur ; `begin_on`
  (`pub(crate)`) et le champ d'`Etape::ApresTable` risquaient `dead_code` sous `clippy -D warnings`.
- **Retenu** : `export_installation_in(snap: &mut ReadSnapshot, obs: …)`, `finish` par l'appelant
  (`export_installation`, ou le test) ; chemin public couvert par AC5 (e) et AC6 ; `begin(pool)` = `pool.acquire()`
  puis `begin_on(conn)` ; `#[cfg_attr(not(test), allow(dead_code))]` motivé sur le champ d'`Etape`.
- **Écartées** : cœur qui prend l'instantané par valeur et le clôt (les tests (c), (d) n'auraient plus la main sur la
  fin) ; `#[allow(dead_code)]` sans condition (masquerait un vrai code mort).
- **Réversible** : oui.

### C-28-3-30 — Signatures sqlx exactes et résidus de l'ouverture (F2-5, F2-8, F2-11)

- **Contexte** : l'alias `BoxFuture<'a>` à un paramètre était employé avec deux ; `Transaction::begin` attend un
  `Option<Cow<'static, str>>` ; `BeginFailed` n'était pas décrit ; « épingler » laissait croire à un `=0.8.6`.
- **Retenu** : `type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;` ;
  `Some("START TRANSACTION …".into())` ; `BeginFailed` nommé comme résidu sans remède (connexion cédée), « sur toute
  erreur » borné à l'instantané ouvert ; dépendance « signalée en commentaire », version tenue par `Cargo.lock` seul,
  non épinglée.
- **Écartées** : épingler `sqlx = "=0.8.6"` (touche tout le workspace ; la montée de sqlx est une décision à part).
- **Réversible** : oui.

### C-28-3-31 — Deux angles morts de plus, écrits (F2-6, F2-9)

- **Contexte** : la garde lexicale compte l'identifiant `pool`, un alias (`let p = pool;`) la contourne ; une DDL
  `INSTANT` concurrente ne fait pas forcément échouer la lecture.
- **Retenu** : alias du pool ajouté à l'angle mort de l'AC2 (détecteur : la revue) ; DDL : « la sauvegarde échoue, ou
  lit la structure neuve », non « jamais incohérente » (C-28-3-8 complété).
- **Écartées** : faire rougir la garde sur tout `= pool` (heuristique de plus, sans fermer d'autres formes) ; tester
  une DDL concurrente (déjà écarté, C-28-3-8).
- **Réversible** : oui.

### C-28-3-32 — Validation P3 en Sonnet, lentilles R et F, axe premier sur les régressions de la remédiation P2

- **Contexte** : rotation D6 (P1 Sonnet, P2 Opus) ; la remédiation P2 retire un mécanisme et touche AC1-AC5, AC7, AC8,
  AC10, AC11, T2, T6, les mutations, les Dev Notes : trop large pour une passe ciblée.
- **Retenu** : passe complète **Sonnet**, deux lentilles, mêmes axes et interdits que P2 (tout interpréteur,
  `mariadb`), plus un axe premier : les régressions de la remédiation P2, avec la question « un défaut né de cette
  remédiation est-il un recyclage ? ». Prompt `28-3-validate-prompt-p3.md`.
- **Réversible** : oui.
