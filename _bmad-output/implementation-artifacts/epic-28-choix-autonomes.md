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

## Développement de la Story 28-3 (2026-10-13, Opus 5.5)

### C-28-3-33 — `orphelins` reçoit aussi le pool

- **Contexte** : la fiche prescrit `orphelins(&InstallationExport) -> Vec<String>`, mais le helper lit les clés
  étrangères dans `information_schema.KEY_COLUMN_USAGE` : il lui faut un exécuteur.
- **Retenu** : `orphelins(pool: &MySqlPool, export: &InstallationExport)`, lecture du schéma sur le pool du test (le
  schéma ne change pas pendant le test). L'assertion de montage (la clé de l'import est vue) est dans le helper.
- **Écartées** : lire les clés dans l'instantané (le helper s'exécute après `finish`) ; une liste de clés écrite en
  dur (elle ne verrait pas une table neuve).
- **Réversible** : oui.

### C-28-3-34 — Observateur factorisé en `fn`, montage rendu en tuple, scénario commun

- **Contexte** : (a-défaut), (a-RC) et (b) partagent montage, observateur et attendus ; la preuve `git log -L` vise
  des `fn`.
- **Retenu** : cinq helpers `fn` (`monter_import` → `(company_id, import_id)`, `lignes`, `orphelins`,
  `observateur_qui_supprime` → `impl FnMut(Etape) -> BoxFuture<'static, ()> + Send`, fermeture annotée,
  `sauvegarder_pendant_suppression`) ; chaque test tient en trois lignes. Aucune `struct` de montage, pour que la
  preuve par fonction couvre tout.
- **Réversible** : oui.

### C-28-3-35 — `Etape`, `BoxFuture` et le cœur restent privés au module

- **Contexte** : la fiche dit le cœur « privé » ; rien hors de `backup.rs` n'a besoin de `Etape`.
- **Retenu** : `enum Etape` et `type BoxFuture` privés, `export_installation_in` privée ; `InstallationExport` et
  `export_installation` publics. `ReadSnapshot::begin_on` en `pub(crate)`.
- **Réversible** : oui.

### C-28-3-36 — Le helper lexical de `build_keshbackup` est local à `export.rs`

- **Contexte** : (g)(i) et (g)(ii) découpent le corps d'une fonction de la même façon, dans deux crates ; le helper
  `code_de_production` de `routes/admin.rs` est privé à son `mod tests`.
- **Retenu** : une fonction de découpe dans chaque `mod tests` (une quinzaine de lignes), comme le précédent de
  `routes/admin.rs`. Duplication assumée, de test seulement.
- **Écartées** : un helper public dans `kesh_db::test_fixtures`, module compilé en production — y mettre une analyse
  lexicale de sources mêlerait outillage de test et code livré.
- **Réversible** : oui.

### C-28-3-37 — Garde G18 : le nom de table du remède de moteur est une occurrence assumée

- **Contexte** : le remède du manuel recrée `fk_contact_persons_company … REFERENCES companies(id)` ; la garde G18
  (`textes_coherents.rs`, une installation = une société) rougit sur le mot `companies`.
- **Retenu** : fragment assumé `REFERENCES companies(id) ON DELETE CASCADE` (« nom de table », comme le précédent du
  README), `MULTI_SOCIETE_TOTAL` 16 → 17, commentaire daté.
- **Écartées** : renvoyer au fichier de migration au lieu d'écrire la requête (remède moins complet, contraire à la
  validation P3).
- **Réversible** : oui.

### C-28-3-38 — Le test (h) est gardé ; (d) porte un témoin positif

- **Contexte** : la fiche autorisait à retirer (h) s'il était non déterministe ; (d) n'avait que des lectures
  « après ».
- **Retenu** : (h) conservé — 20 exécutions sur 20 vertes. (d1)/(d2) vérifient aussi, **dans** l'instantané, que
  `@@in_transaction` vaut 1 (sans quoi le 0 d'après ne prouverait rien).
  *(Annotation du 2026-10-13, revue de code P1, A-3 : les 20 exécutions n'étaient tracées par aucun journal ;
  rejouées au commit `6940b560`, 20 PASS sur 20, `kesh-gate-logs/28-3-h-20-executions.log`.)*
- **Réversible** : oui.

### C-28-3-39 — T3 et T4 dans un même commit ; M3 par substitution de l'instruction

- **Contexte** : T3 et T4 touchent le même fichier (`export.rs` : garde et en-tête) ; M3 « supprime » le `SET`.
- **Retenu** : un commit `c00a7f63` pour T3 et T4, dit au Dev Agent Record. M3 remplace la valeur de la constante
  `SET_ISOLATION` par `DO 0` (instruction neutre) : le chemin de code reste identique, seul l'effet disparaît.
- **Réversible** : oui.

## Remédiation de la revue de code P1 de la Story 28-3 (2026-10-13, Opus 5.5)

Contrainte de l'orchestrateur : la boucle est close à 0 MEDIUM ; la remédiation ne touche **aucune ligne de code de
production** (tests, fiche, manuels, registre seulement).

### C-28-3-40 — `close_on_drop` sur échec du `SET` : testé, non déclaré en angle mort

- **Contexte** : A-4 (lentille A) et E2 (lentille E) — la branche `conn.close_on_drop()` de `begin_on` n'avait aucun
  test, et la suppression de la ligne survivait.
- **Retenu** : test `echec_du_set_ferme_la_connexion` (AC5 (j)) — pool d'une connexion, `BEGIN` **brut** par
  `execute` (sqlx l'ignore, `transaction_depth` reste 0), puis `begin_on` : erreur `1568` attendue, puis la connexion
  réacquise doit avoir un autre `CONNECTION_ID()` et `@@in_transaction` = 0. Faisable sans toucher la production,
  `begin_on` étant `pub(crate)`. Mutation P1-MA (suppression de `conn.close_on_drop();`) tuée : la connexion revient
  au pool, même identifiant.
- **Écartées** : déclarer l'angle mort à l'AC1 (le test est bon marché et déterministe) ; provoquer l'échec du `SET`
  par un `KILL` (par raisonnement, non essayé : une connexion morte ne revient pas utilisable au pool, avec ou sans
  `close_on_drop`, et le test ne distinguerait pas la mutation).
- **Réversible** : oui.

### C-28-3-41 — La propagation de `finish` et l'appel unique, tenus par des sous-chaînes dans les gardes lexicales

- **Contexte** : E1, L1, A-5 — `let _ = snap.finish().await;` survivait ; L3 — remplacer `export_installation(pool)`
  par une autre fonction prenant le pool laissait le compte de (g)(ii) à deux.
- **Retenu** : (g)(i) exige la sous-chaîne `snap.finish().await?` (P1-MB tuée) ; (g)(ii) exige
  `export_installation(pool)` (P1-ME, fonction de substitution `autre_lecture(pool)`, tuée). L'AC1 est reformulée
  sans « inatteignable » : la propagation n'est exercée par aucun test d'exécution, parce que l'observateur du cœur y
  est vide et que le rendre injectable changerait la production.
- **Écartées** : `export_installation_with(pool, obs)` et un test qui tue la connexion à `ApresTable(dernière)`
  (proposé par B et E) — refactor de production, interdit à cette remédiation ; à reprendre si la boucle se rouvre.
- **Réversible** : oui.

### C-28-3-42 — Le test (e) seede une société pour asserter `instance_id`

- **Contexte** : E3 — `min_required` et `instance_id` n'étaient assertés nulle part ; sur la base vide de (e), le
  repli `unwrap_or(0)` aurait masqué `read_instance_id` rendant toujours 0.
- **Retenu** : (e) appelle `seed_accounting_company` avant l'export et compare `min_required` et `instance_id` aux
  valeurs lues directement (`_kesh_version`, `MIN(companies.id)`, assertion de montage ≥ 1). P1-MC (`Ok(0)`) et
  P1-MD (autre colonne lue pour `min_required`) tuées.
- **Écartées** : un test neuf dédié (le chemin public est déjà monté par (e)) ; deux sociétés pour distinguer `MIN`
  de `MAX` (la mutation `MAX(id)` survit avec une seule société : non demandé, signalé ici).
- **Réversible** : oui.

### C-28-3-43 — Lignes orphelines avant `ADD CONSTRAINT` : une phrase, sans requête de repérage

- **Contexte** : E4 — l'ajout des clés étrangères du remède échoue s'il reste des lignes orphelines dans
  `contact_persons`, probable puisque hors InnoDB le `ON DELETE CASCADE` n'a jamais joué. Seul le manuel
  d'administration porte le remède (`grep -rn "ADD CONSTRAINT" docs/manual` : `admin-manual.tex` seul).
- **Retenu** : une phrase après le listing (« Cet ajout est refusé s'il reste … des lignes orphelines … Les supprimer,
  ou les rattacher, puis relancer l'`ALTER TABLE` »). PDF régénéré, contrôlé aplati.
- **Écartées** : une requête `NOT EXISTS` de repérage — elle nommerait `companies`, que la garde G18
  (`textes_coherents.rs`) compte comme promesse multi-société ; il faudrait une seconde occurrence assumée et
  `MULTI_SOCIETE_TOTAL` 17 → 18, pour un remède que l'administrateur peut écrire depuis la phrase.
- **Réversible** : oui.

---

## Story 28-2 — Une erreur d'API ne s'affiche plus « [object Object] » (#520)

Fiche : `28-2-erreurs-object-object.md`. Choix faits à la création de la fiche (2026-10-13), en autonomie, sur
`6f971d76` (rétrospective de l'Epic 27). *Tenus d'abord dans un registre de branche
(`epic-28-choix-autonomes-28-2.md`), versés ici par union à la remédiation de la validation P2 ; fichier de branche
supprimé.*

### C-28-2-1 — Périmètre : les classes A et B sont traitées, C légitime, D assumée, E conforme

- **Contexte** : l'issue vise le motif `e instanceof Error ? e.message : String(e)`. L'inventaire au sol (fiche,
  § Inventaire) trouve 20 conversions affichées d'une valeur attrapée : 4 qui rendent « [object Object] » pour une
  `ApiError` (A), 16 qui la traitent mais gardent un repli `String` (B) ; 3 conversions dans un `console.*` (C) ;
  ≈ 23 `catch` qui n'affichent rien hors `ApiError` (D) ; de nombreux replis `i18nMsg` génériques (E).
- **Retenu** : A et B résolus par un utilitaire unique ; C légitimes par critère (journaliser n'est pas afficher) ;
  D angle mort assumé, issue de suivi demandée (C-28-2-10) ; E laissés tels quels (traduits, jamais « [object
  Object] »).
- **Écartées** : A seul (laisserait « TypeError: … » et « [object Object] » d'un objet quelconque sur 16 sites, et
  la garde ne pourrait pas interdire `String(err)`) ; A + B + D + E (refonte d'une centaine de sites, hors registre
  de la demande, dans sept modules de plus).
- **Réversible** : oui.
- **Révisé en validation P1** : classe C passée à la valeur brute (C-28-2-19) ; classe D suivie par #644 (C-28-2-22).
- **Révisé en validation P2** : classe C laissée telle quelle, sous l'exemption `console` — C-28-2-19 retiré (C-28-2-25).

### C-28-2-2 — L'utilitaire dans un module neuf ; `isApiError` extrait et réexporté

- **Contexte** : `errorMessageOf` vit dans `api-client.ts` (15-5b). Le traduire exige `i18nMsg`, or
  `i18n.svelte.ts` importe `apiClient`. Et 18 fichiers de test simulent `$lib/shared/utils/api-client` par une
  fabrique qui ne fournit presque jamais `isApiError` ni `errorMessageOf`.
- **Retenu** : `lib/shared/utils/error-message.ts` (l'utilitaire) importe `isApiError` de
  `lib/shared/utils/api-error.ts` (neuf, sans dépendance), que `api-client.ts` réexporte. Les quatre importeurs
  d'`errorMessageOf` importent du module neuf ; `api-client.ts` ne le réexporte pas.
- **Écartées** : le laisser dans `api-client.ts` (cycle d'import et dépendance aux simulations du client) ;
  déplacer `isApiError` sans réexport (~110 importeurs à modifier).
- **Réversible** : oui.
- **Révisé en validation P2** : « ~110 importeurs » était faux — **53** fichiers (**166** appels hors tests) (R7).
- **Annoté en validation P3** (2026-10-13, P3-2) : « 166 appels » comptait 166 **occurrences** de `isApiError(` hors tests, dont la déclaration et un doc-comment — **164 appels** ; commande à la fiche, AC1.

### C-28-2-3 — Une `Error` ou une valeur inconnue rend le générique traduit, et se journalise

- **Contexte** : le message d'une `Error` du navigateur ou du code est technique et non traduit (« Failed to
  fetch », « No candidate for txId=5 », `ReconciliationProposals.svelte:206/:224`). Une seule `Error` à message
  destiné à l'utilisateur existe (`admin-backup.api.ts:39`), et son écran ne passe pas par l'utilitaire.
- **Retenu** : `i18nMsg('error-unexpected', 'Erreur inattendue.')` + `console.error('[errorMessageOf]', err)`.
- **Écartées** : afficher `err.message` d'une `Error` (texte technique, langue imprévisible) ; ne rien journaliser
  (perte du diagnostic).
- **Réversible** : oui.

### C-28-2-4 — Les messages fabriqués par le client sont traduits à la source

- **Contexte** : `NETWORK_ERROR`, `TIMEOUT`, `UNKNOWN_ERROR` portent des messages français en dur, lus par 177
  lectures de `.message` du frontend. Les traduire dans l'utilitaire seul n'aurait servi que ses vingt sites.
- **Retenu** : `i18nMsg` dans `toFetchApiError` et dans une fonction `serverStatusMessage(status)` qui factorise les
  trois `Erreur serveur (${res.status})` ; replis = textes actuels, à l'identique. Le cycle `api-client.ts` ↔
  `i18n.svelte.ts` est bénin (usages à l'appel seulement), et un test le prouve.
- **Écartées** : traduction dans l'utilitaire seul ; un module de catalogue séparé pour casser le cycle (refonte
  d'`i18n.svelte.ts` sans gain mesuré).
- **Réversible** : oui.

### C-28-2-5 — `SETUP_REQUIRED` et `UNAUTHENTICATED` restent en français

- **Contexte** : ces deux messages (`api-client.ts:407`, `:427`, `:504`, `:521`) précèdent une redirection.
- **Retenu** : hors périmètre, écrits en angle mort.
- **Écartée** : les traduire aussi (deux clés pour des textes qu'aucun écran connu n'affiche).
- **Réversible** : oui.

### C-28-2-6 — Les préfixes français en dur d'A3 et A4 disparaissent

- **Contexte** : « Erreur chargement profils : » et « Profil introuvable : » sont en dur, et le second est faux
  pour une panne réseau.
- **Retenu** : `errorMessageOf(err)` seul.
- **Écartée** : deux clés à variable `{ $message }` (le contexte est déjà donné par l'écran).
- **Réversible** : oui.

### C-28-2-7 — La garde est un test Vitest qui lit les sources, non une règle ESLint

- **Contexte** : le dépôt tient ses gardes de sources en Vitest (`lib/shared/i18n-*.test.ts`), exécutées par
  `npm run test:unit` en CI ; aucune règle ESLint maison n'existe.
- **Retenu** : `lib/shared/erreurs-en-texte.test.ts`, règles G1-G7, exemption `console` par critère, planchers
  anti-muet, auto-tests du détecteur.
- **Écartée** : un plugin ESLint (outillage neuf, sans précédent au dépôt).
- **Réversible** : oui.

### C-28-2-8 — E2E par interception `page.route`

- **Contexte** : provoquer un vrai refus du verrou des livres verrouillerait la société de `kesh_e2e`, partagée par
  toutes les specs (pollution d'état, cf. le huitième échec instable).
- **Retenu** : interception de la requête (réponse 400 JSON ; abandon réseau pour le second scénario), qui exerce
  le vrai `parseErrorResponse` / `toFetchApiError` du navigateur.
- **Écartée** : refus réel suivi d'un déverrouillage (deux écritures, un nettoyage qui peut échouer).
- **Réversible** : oui.

### C-28-2-9 — Dérogation à la règle de découpage préventif (six modules)

- **Contexte** : `shared/utils`, `settings`, `bank-import`, `bank-accounts`, `reconciliation`, `reports`.
- **Retenu** : une seule story — la logique est dans `shared/utils`, le reste est un remplacement d'une ligne par
  site ; scinder laisserait la garde rouge sur la sous-story pilote. Revue site par site exigée au prompt de revue.
  **Signal déclaré au Project Lead.**
- **Écartée** : 28-2a (utilitaire + garde) / 28-2b (vingt sites).
- **Réversible** : oui (découper avant `dev-story` si Guy le décide).
- **Révisé en validation P1, puis en validation P2** : la remédiation P1 portait la story à sept modules (deux fichiers d'infrastructure, C-28-2-19) ; C-28-2-19 retiré (C-28-2-25), elle revient à **six** modules du frontend, plus le crate **`kesh-i18n`** (catalogues FTL, données seules), que les décomptes antérieurs omettaient (F8). Dérogation maintenue, signal redéclaré au Project Lead.

### C-28-2-10 — La classe D devient une issue de suivi, P3

- **Contexte** : ≈ 23 sites dans sept modules (`credit-notes`, `invoices` ×5 écrans, `payment-batches`,
  `supplier-invoices`) n'affichent rien quand la valeur attrapée n'est pas une `ApiError` ; cas rare (corps de
  succès non JSON, erreur de programmation).
- **Retenu** : inventoriée et comptée dans la fiche ; issue demandée à l'orchestrateur, P3.
- **Écartée** : la traiter ici (découpage préventif, et défaut distinct de #520).
- **Réversible** : oui.
- **Révisé en validation P1** : l'issue est ouverte (#644, par l'orchestrateur) ; voir C-28-2-22.
- **Révisé en validation P2** : « sept modules » était faux — la parenthèse en nomme **quatre** (`credit-notes`, `invoices`, `payment-batches`, `supplier-invoices`) (R8).

### C-28-2-11 — Un test par fichier de la classe B, la garde pour le reste

- **Contexte** : `settings/api-keys` et `reports` n'ont pas de test de page.
- **Retenu** : un test par fichier B sur un rejet non-`ApiError` ; créer `api-keys-page.test.ts` ; pour `reports`,
  un cas dans un test existant s'il y en a un, sinon la garde AC6 seule, écrit au Dev Agent Record.
- **Écartée** : un test par site (vingt montages pour une ligne chacun).
- **Réversible** : oui.
- **Révisé en validation P1** : plus d'exception pour `reports` — voir C-28-2-21.

### C-28-2-12 — La section `[0.13.3] — Non publié` du CHANGELOG est créée par cette branche si absente

- **Retenu** : union avec 28-1 et 28-3 à la fusion.
- **Réversible** : oui.

### C-28-2-13 — Les littéraux français en dur voisins sont signalés, non traités

- **Contexte** : `'ID invalide'`, le message réseau de `routes/login/+page.svelte:54`, les préfixes d'`InvoiceForm`.
- **Retenu** : angle mort écrit ; l'orchestrateur vérifie si une issue i18n les couvre.
- **Réversible** : oui.

### C-28-2-14 — Recette visuelle : quatre captures hors dépôt

- **Retenu** : section `settings-books-lock`, thèmes clair et sombre, avant et après, dans
  `/home/gcorbaz/devel/kesh-gate-logs/28-2-captures/`, citées au Dev Agent Record et au prompt de revue (action B2
  de la rétrospective de l'Epic 27).
- **Écartée** : captures versionnées (poids du dépôt, aucune garde ne les relit).
- **Réversible** : oui.

---

Choix faits à la remédiation de la validation P1 (2026-10-13, Opus 5.5, autonomie). Rapports hors dépôt :
`kesh-gate-logs/28-2-validate-p1-{R,F}.md`.

### C-28-2-15 — Le manuel cite le message réseau ; il reste exact et ne change pas (F1 = R1)

- **Contexte** : `user-manual.tex:1871-1873` cite « Impossible de contacter le serveur… » (import CSV, fichier
  modifié après l'aperçu), citation coupée sur deux lignes — d'où le zéro du `grep` mono-ligne de la fiche.
- **Retenu** : AC11 réécrit ; le passage reste exact en français (repli et texte `fr-CH` identiques, garde
  `i18n-repli-divergent-actif`) ; les manuels `de/en/it` n'ont pas de `.tex` ; manuel et PDF inchangés ; contrôle
  sur texte aplati, `.tex` et PDF, sorties attendues écrites.
- **Écartées** : ajouter « dans la langue de l'installation » au passage (rien ne change pour le lecteur du manuel
  français) ; supprimer la citation (elle aide l'utilisateur à reconnaître le cas).
- **Réversible** : oui.

### C-28-2-16 — Portée des liaisons dans la garde, et échec sur portée incalculable (F2)

- **Contexte** : l'AC6 disait « recueille les liaisons » sans borner leur portée ; `corpsDeFonction` rend `null` sur
  des accolades déséquilibrées, et un `null` sauté en silence est le précédent de la 23-1a.
- **Retenu** : portée = bloc (`corpsDeFonction`) pour `catch` et `.catch` à bloc ; pour un `.catch` à
  corps-expression, de `=>` à la parenthèse qui ferme `.catch(`, même saut des littéraux — par une fonction
  `blocEquilibre(source, ouvrante, ouvre, ferme)` exportée d'`i18n-literal-reader.js`, dont `corpsDeFonction`
  devient un appel (les outils de saut des littéraux y sont privés) ; une portée `null` fait échouer la garde en
  nommant la ligne ; auto-tests accolades dans chaîne / gabarit / regex.
- **Écartées** : chercher les règles dans tout le fichier (faux positifs sur les `err` d'autres portées) ; sauter
  une portée incalculable avec un avertissement (garde muette) ; recopier la boucle dans la garde, ou exporter les
  trois outils privés du lecteur (copie de logique, ou surface publique élargie pour un seul appelant).
- **Réversible** : oui.

### C-28-2-17 — Planchers anti-muet par forme de liaison (F3)

- **Contexte** : un plancher global de 160 laissait passer la perte de toutes les liaisons `.catch` (5) ou typées (4).
- **Retenu** : fichiers ≥ 310 (317), `catch` non typé ≥ 150 (159), typé ≥ 4 (4), `.catch` lié ≥ 5 (5), total ≥ 160
  (168) ; les petites formes au plancher exact ; un plancher qui rougit se baisse avec sa raison écrite.
- **Écartée** : bornes exactes partout (rougiraient à chaque `catch` ajouté ou retiré par une story ordinaire).
- **Réversible** : oui.
- **Révisé en validation P2** : le plancher total est retiré (informatif), les valeurs attendues en fin de story sont écrites (C-28-2-28).

### C-28-2-18 — G3 interdit `instanceof Error` sous toute forme, dans tout le fichier (F8)

- **Contexte** : G3 ne visait que la forme ternaire ; `if (e instanceof Error) msg = e.message; else msg = repli;`
  n'était vue par aucune règle ; et la forme peut vivre hors d'une portée de liaison (`.reason` d'`allSettled`,
  fonction intermédiaire).
- **Retenu** : toute occurrence hors commentaires et hors appel `console`, dans tout le fichier ; à la fin de la
  story il n'en reste aucune (les 9 lignes de code disparaissent avec A, B et C).
- **Écartée** : garder la seule forme ternaire (laisse passer la forme `if`, qui affiche le même texte technique).
- **Réversible** : oui.
- **Révisé en validation P2** : des 9 lignes de code, 6 disparaissent (A et B) et 3 restent, exemptées (classe C, C-28-2-25) ; motif à bornes de mot, classification comprise (C-28-2-27).

### C-28-2-19 — Classe C : les journalisations passent la valeur brute (F5 = R8)

- **Contexte** : `hooks.client.ts:27`, `auth.svelte.ts:301`, `api-client.ts:162` journalisent
  `error instanceof Error ? error.message : String(error)` : une `ApiError` y devient « [object Object] ».
- **Retenu** : `console.*('[…]:', error)` — une ligne par site ; la console affiche l'objet (`code`, `message`,
  `status`). L'exemption `console` de la garde reste (auto-tests). Classe C : 3 → 0.
- **Écartées** : laisser en l'état (diagnostic perdu, alors que `errorMessageOf` prend soin de le conserver) ;
  un formateur de journal dédié (code neuf pour trois lignes).
- **Réversible** : oui.
- **Retiré en validation P2** (C-28-2-25) : la prémisse est fausse — les trois `catch` n'enveloppent que des `fetch` directs et ne reçoivent que des `Error` ; aucune `ApiError` n'y devient « [object Object] ». La classe C reste sous l'exemption `console`, inchangée.

### C-28-2-20 — `handleDelete` de `BankProfileList.svelte` reçoit un `try/catch` (R10)

- **Contexte** : `await deleteBankProfile(id)` sans `try/catch` : un refus devient un rejet non géré, sans message ;
  même fichier qu'A3.
- **Retenu** : `try { delete; load } catch (err) { notifyError(errorMessageOf(err)); }` — notification, la liste
  restant affichée ; test et mutation M23.
- **Écartées** : `errorMessage = …` (remplacerait toute la liste) ; l'angle mort seul (le fichier est touché, le
  correctif tient en trois lignes).
- **Réversible** : oui.

### C-28-2-21 — Tests de page neufs pour `reports` et `api-keys`, sans exception (R7)

- **Contexte** : l'AC9 exigeait un test par fichier B puis exceptait `reports` ; ni `reports` ni `api-keys` n'ont de
  test de page (`reports.api.test.ts` teste l'API).
- **Retenu** : `api-keys-page.test.ts` (B6) et `reports-page.test.ts` (B15, branche `REPORT_PERIOD_…` préservée) ;
  mutations M21, M22.
- **Écartée** : la garde AC6 seule pour `reports` (elle voit la forme, pas le rendu).
- **Réversible** : oui.

### C-28-2-22 — Classe D hors périmètre, suivie par #644 (F4, R2, R3)

- **Contexte** : décision de l'orchestrateur après la validation P1 — la correction n'est pas mécanique (silences
  voulus, `invoices/[id]/+page.svelte:186` et `:483`), et l'intégrer porterait la story de 6 à au moins 10 modules.
  Il a ouvert l'issue **#644** (bug, P3).
- **Retenu** : la fiche cite la classe D comme angle mort assumé, avec renvoi `refs #644` (PR comprise), et les deux
  silences voulus ; **aucune** autre issue de suivi n'est demandée.
- **Écartée** : intégrer la classe D (règle G8, ~25 fichiers, dérogation au découpage intenable).
- **Réversible** : oui.

### C-28-2-23 — La traduction vaut une fois le catalogue chargé ; CHANGELOG et AC10 le disent (F6 ≈ R5)

- **Contexte** : le catalogue est derrière l'authentification et chargé à l'`onMount` racine si une session est
  ouverte : `/login`, un rechargement serveur injoignable et la navigation après connexion restent en français ;
  l'E2E en `fr-CH` ne distingue pas un texte traduit d'un littéral.
- **Retenu** : angle mort n° 5 ; phrase du CHANGELOG restreinte ; l'AC10 écrit que le scénario 2 ne prouve pas la
  traduction (les tests unitaires au catalogue allemand le font) ; l'orchestrateur vérifie si une issue couvre la
  limite préexistante du chargement du catalogue.
- **Écartées** : charger le catalogue après la connexion (hors sujet, touche l'authentification) ; une instance E2E
  en `de-CH` (montage neuf pour un contrôle que l'unitaire fait déjà).
- **Réversible** : oui.
- **Révisé en validation P2** : phrase du CHANGELOG précisée — « une fois la page chargée — ou rechargée — avec une session ouverte ; la page de connexion, et l'interface jusqu'au premier rechargement qui suit une connexion, restent en français » (F11 = R6).

### C-28-2-24 — Une `ApiError` dont le `message` n'est pas une chaîne rend le générique (F7)

- **Contexte** : `isApiError` ne vérifie que `code` et `status` ; un `message` `undefined` ou numérique passerait.
- **Retenu** : ligne explicite au tableau de l'AC1 et cas de test ; `isApiError` lui-même n'est pas durci (110
  importeurs, comportement inchangé hors de l'utilitaire).
- **Écartée** : durcir `isApiError` (changerait l'aiguillage de sites que la story ne touche pas).
- **Réversible** : oui.
- **Révisé en validation P2** : « 110 importeurs » était faux — **53** fichiers importent `isApiError` du client (**166** appels hors tests) ; la décision ne change pas (R7).
- **Annoté en validation P3** (2026-10-13, P3-2) : « 166 appels » comptait 166 **occurrences** de `isApiError(` hors tests, dont la déclaration et un doc-comment — **164 appels** ; commande à la fiche, AC1.

---

Choix faits à la remédiation de la validation P2 (2026-10-13, Opus 5.5, autonomie). Rapports hors dépôt :
`kesh-gate-logs/28-2-validate-p2-{R,F}.md`.

### C-28-2-25 — C-28-2-19 retiré : la classe C reste sous l'exemption `console` (F1 = R2)

- **Contexte** : C-28-2-19 justifiait de passer la valeur brute à `console.*` par une `ApiError` qui y deviendrait
  « [object Object] ». Au sol, les trois `catch` (`hooks.client.ts:27`, `auth.svelte.ts:301`, `api-client.ts:162`)
  n'enveloppent que des `fetch` directs de `/api/v1/auth/me` (`auth.svelte.ts:229`, `api-client.ts:131`) ou ce qui
  sort de `hydrate()`, dont tout le corps réseau est gardé : ils ne reçoivent que des `Error`, dont le `message` est
  déjà journalisé. C'est ce seul changement qui portait la story à un septième module.
- **Retenu** : retirer C-28-2-19. Classe C = 3, légitimes par critère, inchangées ; `hooks.client.ts` et
  `auth.svelte.ts` hors périmètre ; G3 : 6 lignes de code disparaissent, 3 restent exemptées ; l'exemption `console`
  est exercée au parcours réel par ces trois sites. Retour à six modules (plus le crate `kesh-i18n`, C-28-2-9).
- **Écartée** : garder C-28-2-19 en réécrivant sa raison (« conserver la pile ») — gain mince (nom et pile d'une
  `Error`), deux fichiers d'infrastructure de démarrage touchés pour lui, et un module de plus à la dérogation.
- **Réversible** : oui (trois lignes, à reprendre par une story qui le voudrait).

### C-28-2-26 — Les tests B des deux modales soumettent réellement, `AccountAutocomplete` piloté (R1)

- **Contexte** : M15 devait rougir sur `ManualMatchModal.test.ts` « existant » ; or aucun test de cette modale ni de
  `TransactionSplitModal` ne soumet (`handleConfirm` sort sans contrepartie, `ManualMatchModal.svelte:126`) :
  supprimer la branche `BANK_ACCOUNT_NOT_CONFIGURED` laissait le fichier vert.
- **Retenu** : cas neufs de soumission dans les deux fichiers, avec le **vrai** `AccountAutocomplete` (focus du
  champ `input[aria-autocomplete="list"]`, `mouseDown` sur l'option — la sélection se fait au `mousedown`,
  `AccountAutocomplete.svelte:384-388` —, comptes de classe 5 à 7 en prop) ; pour la ventilation, deux lignes
  équilibrées. Deux cas par modale : rejet `new Error('boom')` → générique ; rejet 412
  `BANK_ACCOUNT_NOT_CONFIGURED` → indice affiché, qui tue M15 (B13) et M26 (B14, neuve).
- **Écartées** : simuler `AccountAutocomplete` par `vi.mock` (le montage réel est à portée — trois gestes — et
  vérifie en plus le câblage `onSelect`) ; retirer M15 et écrire l'angle mort (le montage n'est pas
  disproportionné).
- **Réversible** : oui.

### C-28-2-27 — G3 bannit tout usage de `instanceof Error`, classification comprise ; motif à bornes de mot (F3)

- **Contexte** : la raison écrite de G3 (afficher le `message` d'une `Error`) est plus étroite que la règle, qui fait
  aussi rougir `err instanceof Error && err.name === 'AbortError'` ; et un motif sans `\b` attrape `instanceof
  ErrorEvent`.
- **Retenu** : la fiche écrit que G3 bannit **tout** usage, classification comprise, et pourquoi (une règle qui
  distinguerait affichage et classification ne serait plus décidable par lecture) ; le message d'échec oriente vers
  `err.name`, `instanceof DOMException` ou `instanceof TypeError` ; motif `\binstanceof\s+Error\b` ; auto-tests
  `ErrorEvent` (négatif) et classification (positif).
- **Écartée** : exempter la classification (critère sémantique, non textuel).
- **Réversible** : oui.

### C-28-2-28 — Le plancher total est retiré ; les valeurs de fin de story sont écrites (F6 = R10)

- **Contexte** : 150 + 4 + 5 = 159 : le plancher total ≥ 160 n'ajoutait qu'une unité de contrôle ; et la fiche ne
  disait pas ce que la story elle-même fait aux comptes.
- **Retenu** : total informatif (affiché, non borné) ; colonne « attendu en fin de story » : 319 fichiers, 160 non
  typées, 4 typées, 5 `.catch`, 169 au total.
- **Écartée** : garder le plancher total (inoffensif, mais une borne sans fonction se lit comme une garantie).
- **Réversible** : oui.

### C-28-2-29 — Formes de liaison typée et `async` recueillies ; portée `null` en échec sur les trois chemins (F4, F5)

- **Contexte** : `.catch((err: unknown) => …)` et `.catch(async (err) => …)` n'étaient ni recueillis ni auto-testés
  (0 occurrence aujourd'hui, donc aucun plancher ne verrait une telle liaison future échapper) ; l'échec sur portée
  incalculable n'était écrit et testé que pour les accolades, alors que le corps-expression d'un `.catch(` et
  l'exemption `console` calculent aussi une portée (parenthèses), et qu'un `null` lu « dans l'appel » exempterait
  en silence.
- **Retenu** : les deux formes ajoutées à la liste et aux auto-tests ; un `null` est un échec nommé sur les trois
  chemins ; auto-tests « `.catch((err) => String(err)` non fermé » et « `console.error(` non fermé » ; mutation M27.
- **Écartée** : traiter un `null` d'exemption comme « hors de l'appel » (échec bruyant, mais pour une mauvaise
  raison : la garde accuserait une conversion qu'elle n'a pas localisée).
- **Réversible** : oui.

### C-28-2-30 — Les captures de la recette visuelle sont pilotées par une variable, jamais par un chemin écrit en dur (dev)

- **Contexte** : l'AC10 demande des captures du scénario 1, en thème clair et sombre, avant et après le correctif,
  **hors dépôt** (`kesh-gate-logs/28-2-captures/`). Écrire ce chemin dans un spec versionné y ferait entrer un
  chemin de la station de dev, et la spec écrirait sur le disque à chaque gate E2E complet.
- **Retenu** : `KESH_E2E_CAPTURES_DIR` (et `KESH_E2E_CAPTURES_TAG`, `avant` / `apres`) ; sans la variable, la spec
  ne capture rien. Les captures sont prises **avant** l'assertion du texte, pour que la spec rouge d'avant le
  correctif les produise quand même.
- **Écartées** : chemin absolu dans la spec ; `testInfo.outputPath` (dans `test-results/`, effacé au run suivant).
- **Réversible** : oui.

### C-28-2-31 — Le détecteur de la garde vit dans le fichier de test (dev)

- **Contexte** : la fiche prescrit des auto-tests « sur sources fictives passées à la fonction de détection ».
  La fonction peut vivre dans un module `.js` partagé ou dans le test.
- **Retenu** : dans `erreurs-en-texte.test.ts`, comme la garde modèle `i18n-libelle-en-dur.test.ts`. Seul
  `blocEquilibre`, dont la garde a besoin et que `corpsDeFonction` réutilise, entre dans `i18n-literal-reader.js`.
- **Écartée** : un module `erreurs-en-texte.js` — aucun autre consommateur, et un export de plus à maintenir.
- **Réversible** : oui.

### C-28-2-32 — G4 voit aussi `${String(<liaison>)}`, au prix d'un double constat avec G1 (dev)

- **Contexte** : l'AC6 écrit G4 « `${<liaison>}` ou `${String(<liaison>)}` dans un gabarit », et la mutation M11 (A3
  rétabli : `` `… ${String(err)}` ``) attend « garde G4 ». Or `${String(err)}` relève déjà de G1.
- **Retenu** : G4 reconnaît les deux formes ; le site ressort sous G1 **et** G4. Le message d'échec est plus long,
  mais la règle nommée par la fiche est celle qui rougit.
- **Écartée** : G4 réduite à `${x}` (M11 ne serait vue que par G1, contrairement à la fiche).
- **Réversible** : oui.

### C-28-2-33 — `formatError` (rapports) garde `return err.message` dans sa branche `ApiError` (dev)

- **Contexte** : l'AC4 dit de `formatError` (B15) que « seule la dernière ligne change ». Sa branche `ApiError`
  rend `err.message` tel quel : une `ApiError` au message vide y rendrait la chaîne vide, et `{#if errorMsg}`
  n'afficherait rien.
- **Retenu** : la fiche, à la lettre — seule la dernière ligne passe par `errorMessageOf`. Le cas du message vide
  est écrit ici et au Dev Agent Record : il relève du « message vide » (classe D, #644), non d'un « [object
  Object] ».
- **Écartée** : `return errorMessageOf(err)` aussi dans la branche `ApiError` — plus sûr, mais hors du périmètre
  écrit, et la revue P1 en jugera.
- **Réversible** : oui, une ligne.
- **Révisé le 2026-10-13, revue de code P1 (B-1 = A-4)** : `return err.message` **supprimé** ; la branche
  `ApiError` ne garde que le cas `REPORT_PERIOD_OUT_OF_FISCAL_YEAR`, tout le reste tombe sur `errorMessageOf`, qui
  rend le même texte pour un message non vide et le générique pour un message vide. Deux tests neufs dans
  `reports-page.test.ts` (message vide ; code de période sans bornes). L'option écartée ci-dessus est donc retenue.

### C-28-2-34 — Le scénario 2 de l'E2E échoue en le disant si la clé `error-network` n'est pas servie (dev)

- **Contexte** : le scénario lit le texte attendu au catalogue servi par `/api/v1/i18n/messages`. Une clé absente
  rendrait `undefined`, et `toContainText(undefined)` échouerait d'une façon trompeuse.
- **Retenu** : assertion explicite, avec son message, sur la présence de la clé avant l'interception. Avant le
  correctif, le scénario 2 rougit donc sur ce message (constaté au run « avant »).
- **Réversible** : oui.

### C-28-2-35 — M20 sous sa forme « repli en gabarit » est déclarée équivalente ; M20b la complète (dev)

- **Contexte** : la fiche attend que `i18n-entrees-a-variables.test.ts` fasse rougir M20 (repli
  d'`error-server-status` écrit en gabarit JS). Jouée contre cette garde, `i18n-keys`, `i18n-repli-divergent-actif`,
  `i18n-un-repli-par-cle` et `api-client.test.ts`, elle **survit** : la garde écarte par construction les replis
  qui ne sont pas des littéraux (`repli.kind !== 'literal'`), et le dépôt en porte déjà un, légitime
  (`reports/+page.svelte`, `reports-error-period-out-of-fiscal-year`). Le rendu est **identique** dans tous les
  chemins : catalogue chargé, le repli n'est pas lu ; catalogue vide ou clé absente, le gabarit rend
  « Erreur serveur (502) », exactement ce que rend l'interpolation du littéral.
- **Retenu** : M20 (gabarit) **déclarée équivalente** à l'observation ; la règle « le repli porte les `{ $x }` »
  reste une convention tenue en revue. La seconde forme écrite par la fiche, « site passant `{ code: … }` », est
  jouée comme **M20b** et **tuée** (maillon 3 d'`i18n-entrees-a-variables`, et `api-client.test.ts`).
- **Écartée** : étendre `i18n-entrees-a-variables` aux replis en gabarit — elle rougirait sur le site légitime des
  rapports, hors périmètre.
- **Réversible** : oui.

### C-28-2-36 — G5 relève la liaison opérande ENTIER de tout `+` / `+=` (revue de code P1, E-M1)

- **Contexte** : G5 ne voyait que `x + ''` / `'' + x`. La graphie la plus naturelle du défaut,
  `'Erreur : ' + err` (« Erreur : [object Object] »), lui échappait, alors que G4 voit `${err}`.
- **Retenu** : G5 relève `x` quand il est un opérande entier d'un `+` (à gauche, hors `x++`) ou d'un `+` / `+=` (à
  droite, hors `++x`), quelle que soit l'autre opérande. Un champ de la liaison (`err.message`, `err?.code`,
  `err[0]`, `err(…)`) n'en relève pas. Auto-tests positifs (chaîne non vide des deux côtés, gabarit, sans blancs,
  multi-ligne, `+=`) et négatifs (champs chaînés, autre variable, préfixe/suffixe de nom, `++`). Faux positif
  accepté et écrit au préambule : `+x` unaire, `x + 1` — bruyants, et sans sens sur une erreur. Parcours réel du
  dépôt : 0 constat neuf.
- **Écartée** : inscrire la forme aux angles morts (la garde ne fermerait pas la classe qu'elle annonce).
- **Réversible** : oui.

### C-28-2-37 — `BANK_ACCOUNT_NOT_CONFIGURED` des deux modales garde `errorMsg = e.message` (revue de code P1, B-2)

- **Contexte** : `ManualMatchModal` et `TransactionSplitModal` affichent `e.message` dans leur branche spécifique ;
  un message vide laisserait le drapeau levé sans texte.
- **Retenu** : laissé tel quel. Le comportement est celui d'avant la story (branche préservée, AC4) ; le cas du
  **message vide** d'une `ApiError` relève de la classe D, suivie par **#644**, non d'un « [object Object] ». Trois
  autres sites, hors du périmètre de la story, ont la même forme (`return err.message` dans une branche `ApiError`,
  repli traduit sinon) : `export/+page.svelte:28`, `supplier-invoices/import/+page.svelte:301`,
  `AdminRestorePanel.svelte:84` (grep du symptôme, revue de code P1) — même renvoi à #644.
- **Écartée** : passer ces branches par `errorMessageOf` — hors périmètre, et le drapeau rend déjà la situation
  visible.
- **Réversible** : oui.

### C-28-2-38 — Les sites B2-B4, B7/B8 et B9 reçoivent leur test d'un rejet non-`ApiError` (revue de code P1, E-L5)

- **Contexte** : l'AC9 demandait un test par fichier de la classe B ; plusieurs sites d'un même fichier n'étaient
  gardés que par G1, qui ne voit pas `(err as Error).message`.
- **Retenu** : le montage était simple — six tests neufs (création, modification et archivage des comptes
  bancaires ; création et révocation des clés d'API ; chargement du dialogue d'annulation), chacun constatant
  l'appel API ; mutations P1-M9 à P1-M14 (`(err as Error).message`) jouées. Les sites B5, B11, B12 restent couverts
  par le test unique de leur fichier, comme l'AC9 l'admet.
## Story 28-5 — Une transaction rejetée redevient atteignable (#526)

Choix faits à la fiche (2026-10-13), en autonomie, sur `5aad8b35`. Fiche :
`implementation-artifacts/28-5-transaction-rejetee-atteignable.md`.

### C-28-5-1 — Retrouver la rejetée dans le détail de son import, non par un filtre de la page *Réconciliation*

- **Contexte** : l'issue et l'epic disent « filtre ou onglet ». La transaction rejetée est **déjà** listée dans le
  détail de son import (statut brut `pending`, sans action) ; c'est aussi l'écran où la 25-3-b a placé *Annuler le
  rapprochement*.
- **Retenu** : marquer la ligne « Rejetée le … » et y poser *Annuler le rejet* ; une phrase de la page
  *Réconciliation* dit où la retrouver (C-28-5-10). Écart **assumé** à la lettre de l'issue, pour le périmètre
  minimal.
- **Écartée** : filtre ou section « Rejetées » sur la page *Réconciliation* — une route de lecture, une fonction de
  dépôt, un composant et leurs tests de plus, pour un besoin que le détail couvre.
- **Réversible** : oui ; le filtre peut s'ajouter plus tard sans rien défaire (CR si la recette le réclame).

### C-28-5-2 — Seul geste neuf : annuler le rejet

- **Contexte** : l'issue offre « rapprocher manuellement, **ou** annuler le rejet ».
- **Retenu** : annuler le rejet ; la transaction revient dans la liste des propositions, où *Accepter*, *Affecter
  manuellement* et *Éclater* existent déjà.
- **Écartée** : poser *Affecter manuellement* et *Éclater* sur la ligne rejetée du détail d'import (deux modales à
  rebrancher hors de leur page, pour éviter un clic).
- **Réversible** : oui.

### C-28-5-3 — Route unitaire `POST /api/v1/reconciliation/transactions/{id}/unreject`, `204`

- **Contexte** : le rejet est un lot (`{ rejected, failed }`) ; l'annulation d'un rapprochement est unitaire, sous
  `…/transactions/{id}/cancel`.
- **Retenu** : route unitaire, même famille de chemin que `…/cancel`, sans corps, `204 No Content` (rien à rendre :
  l'écran relit le détail). Refus en `AppError` (pas de `FailedProposal` : ce n'est pas un lot).
- **Écartées** : `POST /reconciliation/unreject` en lot (un pattern batch neuf, pour un geste qui se fait ligne à
  ligne) ; `DELETE …/rejection` (aucune route du module n'emploie `DELETE`) ; `200` avec la transaction relue (une
  lecture de plus, sans lecteur).
- **Réversible** : oui (route neuve, aucun client existant).

### C-28-5-4 — Verrou nommé du compte pris ; pas de rejeu sur interblocage

- **Contexte** : tous les gestes de réconciliation qui écrivent prennent `with_account_lock` ; seuls ceux qui écrivent
  au journal sont rejoués (`audit_route_registry.rs`, colonne `Rejeu`).
- **Retenu** : verrou pris (sérialise avec accept, reject, manual, split, cancel du même compte : un rapprochement
  manuel concurrent ne bute pas sur un conflit de `version`) ; classement `SansEcritureAuJournal`, sans enveloppe de
  rejeu. Raisonnement d'interblocage : une ligne de `bank_transactions` verrouillée, puis une insertion d'audit ; la
  suppression d'import (27-1) verrouille ses lignes par identifiant croissant sans le verrou nommé — aucun cycle (nuancé, C-28-5-30).
- **Écartée** : pas de verrou (l'`UPDATE` conditionnel suffirait à l'atomicité, mais ferait échouer en
  `OPTIMISTIC_LOCK_CONFLICT` un geste concurrent qui n'a rien de fautif).
- **Réversible** : oui.
- ⚠️ **Validation P2** (2026-10-13) : le code s'écrit `OPTIMISTIC_LOCK_CONFLICT` (`errors.rs:2924`), graphie
  corrigée ci-dessus (findings F1 = R1) ; « aucun cycle » est **nuancé** par **C-28-5-30** — la remise à zéro
  d'onboarding en ferme un (F9 = R3).

### C-28-5-5 — Lecture verrouillante d'abord, classement décisif, un seul code de refus

- **Contexte** : `reject_batch` fait un `UPDATE` gardé puis relit pour classer ; ici il faut aussi l'horodatage
  d'avant pour l'audit.
- **Retenu** : `find_by_id_for_account_for_update` sous verrou → `classify_unreject` (fonction pure, appelle
  `is_reconciled`, aucun second test de statut) → `UPDATE` scopé `id`/`company_id`/`bank_account_id`, une ligne
  exactement sinon erreur défensive tracée. Refus : `404 NOT_FOUND` (absente ou autre société, indiscernables) ;
  `409 RECONCILIATION_TRANSACTION_NOT_REJECTED` pour « non rejetée » **et** « rapprochée » (une rapprochée n'est
  jamais rejetée par les chemins réels : un code suffit, et il dit vrai).
- **Écartées** : `UPDATE` gardé d'abord (perd l'horodatage d'avant, et rend équivalentes les mutations des gardes) ;
  deux codes distincts (« rapprochée » n'est atteignable que par un état forgé).
- **Réversible** : oui. ⚠️ Fait de `kesh-db` un module touché (doc-comment « Seul appelant ») : gate complet en
  boucle.

### C-28-5-6 — Audit `reconciliation.unrejected`, ouverte aux clés `read-write`

- **Retenu** : une ligne par geste, `entity_type = 'bank_transaction'`, `details = { "previouslyRejectedAt" }`
  (casse de la trace voisine `reconciliation.cancelled`, `wasPreviouslyRejected`), écrite par `for_actor` : une clé
  est nommée. Route ouverte aux clés, comme `/reject` et `…/cancel` (la suppression d'import, fermée aux clés, est
  irréversible ; ce geste ne l'est pas).
- **Écartée** : fermer aux clés (aucun motif : geste réversible, sans écriture comptable).
- **Réversible** : oui.

### C-28-5-7 — Rôle Consultation : la mention, pas le bouton

- **Retenu** : « Rejetée le … » visible de tous les rôles qui lisent le détail d'import (route de lecture ouverte à
  tout rôle authentifié) ; bouton réservé au Comptable et à l'Administrateur par la `canManage` existante ; serveur :
  `403` par `comptable_routes`. La page *Réconciliation*, réservée de fait au Comptable, n'est pas changée pour la
  Consultation (comportement existant noté en recette, exclu).
- **Réversible** : oui.

### C-28-5-8 — Champ `autoMatchRejectedAt` additif sur `TransactionResponse`

- **Retenu** : horodatage ISO UTC ou `null`, exposé partout où le type sert (détail d'import, `GET
  …/transactions/{id}`, réponse de `…/cancel`). Les types frontend de ces deux dernières réponses ne sont pas étendus
  (aucun lecteur).
- **Écartée** : un booléen `rejected` (perd la date que l'écran affiche).
- **Réversible** : oui (champ additif).

### C-28-5-9 — Bouton en style de lien, sans dialogue de confirmation

- **Retenu** : `text-primary underline`, comme *Affecter manuellement* ; aucune confirmation : le geste se défait par
  un nouveau rejet et ne touche pas la comptabilité.
- **Écartées** : variante destructive (le geste n'est pas destructif) ; dialogue (friction sans risque à couvrir).
- **Réversible** : oui.

### C-28-5-10 — Une phrase sur la page *Réconciliation*, hors des branches de la liste

- **Retenu** : `reconciliation-labels-rejected-hint`, rendue même quand un rejet a vidé la liste (patron C38 de la
  15-5c pour le bilan du lot). Sans lien (une phrase traduite qui porte un lien se découpe mal).
- **Réversible** : oui.

### C-28-5-11 — Dérogation à la règle de découpage préventif

- **Contexte** : quatre paquets, mais huit sites de premier niveau à la granularité de l'exemple du `CLAUDE.md`.
- **Retenu** : story unique. Un seul site porte une logique neuve (la route) ; les sept autres reçoivent quelques
  lignes mécaniques (un champ, une ligne de registre, une variante, des clés, un doc-comment). Découper donnerait
  une story « route sans écran », qui ne ferme pas #526, et une story « écran », qui ne se teste pas sans elle.
- **Risque accepté** : une passe de revue doit couvrir backend et frontend ; le prompt de validation nomme les deux.
- **Réversible** : oui (découpage possible en validation si la sévérité stagne).
- ⛔ **Remplacé** à la validation P1 (2026-10-13) par **C-28-5-17** : la dérogation ne relevait pas de l'exception
  codifiée au `CLAUDE.md` (finding F3).

### C-28-5-12 — Deux auxiliaires partagés avec l'annulation d'un rapprochement

- **Contexte** : la route neuve répète deux blocs de `cancel_reconciliation_once` : la lecture scopée du
  `bank_account_id` hors verrou, et la conversion `ReconciliationError` → `AppError` qui préserve `DbError::Sqlx`.
- **Retenu** : les extraire (règle DRY), sans changer le comportement de `…/cancel`, dont les tests restent
  inchangés.
- **Écartée** : copier les deux blocs.
- **Réversible** : oui. ⚠️ Touche une région que la 28-6 ne vise pas (`accept_one_invoice`) : pas de conflit attendu.
- ⛔ **Remplacé** à la validation P1 (2026-10-13) par **C-28-5-18** : aucune extraction du convertisseur ; la route
  suit `conclude_locked_attempt` (findings F1, R2).

### C-28-5-13 — Pas de garde « compte archivé » sur la route neuve

- **Contexte** : `/reject` refuse un compte archivé en `404`.
- **Retenu** : pas de garde : un compte archivé n'a aucune transaction (archivage refusé en
  `412 BANK_ACCOUNT_HAS_TRANSACTIONS`), le cas est inatteignable, et un code sans test serait mort. La clôture de
  compte (28-4) devra inscrire cette route à son inventaire des chemins d'écriture — signalé dans la fiche.
- **Réversible** : oui.

### C-28-5-14 — Captures par `KESH_E2E_CAPTURES_DIR`, hors dépôt

- **Retenu** : convention de la 28-1 (C-28-1-29) ; six captures (détail avec rejetée, détail après annulation,
  indication de la page *Réconciliation*, × clair/sombre) dans `kesh-gate-logs/28-5-captures/` ; aucune référence de
  comparaison.
- **Réversible** : oui.

### C-28-5-15 — Date du rejet sans l'heure

- **Retenu** : `Intl.DateTimeFormat(i18nLocale(), { dateStyle: 'medium' })` — patron du journal d'audit
  (`audit-log/+page.svelte:62-64`), sans `timeStyle` : l'heure n'aide pas à retrouver une transaction, et la même
  construction n'est pas recopiée à l'identique (pas d'utilitaire partagé à extraire pour deux sites différents).
- **Réversible** : oui.

### C-28-5-16 — Tests dans `reconciliation_e2e.rs`, section de fin ; CHANGELOG par union

- **Retenu** : les tests B1-B7 vont en fin de `reconciliation_e2e.rs`, qui porte déjà les montages
  (`setup_company`, `seed_bank_transactions`, auxiliaires des clés et du verrou) — un binaire neuf les recopierait.
  La 28-6 et la 28-7, développées après, rebaseront sur cette fin de fichier. La section `## [0.13.3] — Non publié`
  du CHANGELOG est créée si absente, réunie par union sinon (précédent C-28-2-12).
- **Réversible** : oui.
- *Depuis les validations P1 et P2* : B9 et B10 s'y ajoutent (B8 reste unitaire, dans `routes/reconciliation.rs`).

### Validation P1 de la 28-5 (2026-10-13) — choix C-28-5-17 à C-28-5-24

Remédiation des rapports `28-5-validate-p1-R.md` et `28-5-validate-p1-F.md` (Sonnet 5.5), par Opus 5.5, sur les
décisions de l'orchestrateur.

### C-28-5-17 — Découpage en 28-5a (serveur) et 28-5b (écran) — décision de l'orchestrateur

- **Contexte** : huit sites de premier niveau ; la dérogation C-28-5-11 invoquait la fermeture de l'issue, motif que
  la règle de découpage préventif n'admet pas (elle n'admet que des cycles Cargo ou des fusions intermédiaires
  intestables) — finding F3.
- **Retenu** : une fiche, deux clés — `28-5a-annuler-rejet-serveur` (route, champ, audit, messages serveur, gardes,
  tests backend, `docs/api-external.md` ; PR `refs #526`) puis `28-5b-annuler-rejet-ecran` (écran, i18n de
  l'interface, libellé `already-rejected`, manuel + PDF, CHANGELOG, E2E, captures ; PR `closes #526`) ; clé mère
  `split` ; aucune release entre les deux. Précédents C-26-2-15, C-28-4-1.
- **Écartée** : garder la story unique et demander l'accord écrit de Guy (allonge la file sans gain : chaque partie
  se teste seule).
- **Réversible** : oui tant que la 28-5a n'est pas fusionnée.

### C-28-5-18 — Aucune troisième copie du convertisseur ; seule la lecture du compte est partagée

- **Contexte** : C-28-5-12 extrayait le `match` de `cancel_reconciliation_once`, seule garde du rejeu de `…/cancel`,
  qu'aucun test dynamique n'exerce (F1) ; un auxiliaire `conclude_locked_attempt` factorise déjà ce `match` pour
  `post_manual` et `post_split` (R2), et les deux conversions diffèrent (`FiscalYearClosed`, `SplitImbalance`,
  `TransactionAborted`).
- **Retenu** : la route neuve appelle `conclude_locked_attempt`, dont seule la **signature** devient générique en
  `T` (corps intact ; le `commit` d'une fermeture qui n'a rien écrit vaut un `rollback`) ; les erreurs de la
  fermeture sont rangées comme dans `post_manual_once` (`map_err(… Db(DbError::Sqlx(e)))`, `.await?`). De
  `cancel_reconciliation_once`, seule la lecture du `bank_account_id` (l. 4514-4524) passe, mot pour mot, dans un
  auxiliaire privé partagé (règle DRY) : lecture non verrouillante en autocommit, qui ne peut être victime d'un
  interblocage — le rejeu de `…/cancel` n'est pas en jeu. Son `match` reste intouché.
- **Écartées** : extraire le `match` de `cancel` (F1 : propriété non gardée) ; copier la lecture du compte (DRY) ;
  une fonction de dépôt neuve dans `kesh-db` pour cette lecture (un module de plus pour neuf lignes).
- **Réversible** : oui.

### C-28-5-19 — Garde `rows_affected() != 1` en `OptimisticLockConflict`, déclarée inatteignable

- **Contexte** : l'AC3 prescrivait `tracing::error!` + `AppError::Internal`, alors que la fermeture rend une
  `ReconciliationError` (R2) et que la garde est inatteignable, donc sans mutation tuable (F6).
- **Retenu** : `Err(ReconciliationError::Db(DbError::OptimisticLockConflict))`, patron de `post_manual_once`
  (`:3822-3826`), que `conclude_locked_attempt` rend en `AppError::Database` et que le **mapping des erreurs de
  base** rend `409 OPTIMISTIC_LOCK_CONFLICT` (`errors.rs:2922-2925`) — *graphie et producteur corrigés en validation
  P2 (F1 = R1) : la première rédaction écrivait `409 OPTIMISTIC_LOCK`, code que le serveur n'émet pas, et
  l'attribuait à `conclude_locked_attempt`* ; mutation M13 déclarée équivalente (ligne
  lue `FOR UPDATE` sous verrou par sa clé primaire) ; M14 (portée de l'`UPDATE`) équivalente aussi.
- **Écartée** : retirer la garde (un `204` muet si un jour la lecture cesse d'être verrouillante).
- **Réversible** : oui.

### C-28-5-20 — La limite de 100 propositions : exclue, suivie par #649, dite par les textes

- **Contexte** : `GET …/proposals` rend les 100 plus récentes, `hasMore` n'est lu par aucun écran (F2) ; une
  transaction ancienne dont le rejet est annulé est proposable mais non listée. Issue **#649** ouverte par
  l'orchestrateur (P3).
- **Retenu** : exclusion écrite avec renvoi à #649 ; le toast `reconciliation-unreject-success` et la puce
  *Rejeter* du manuel citent la limite au lieu de promettre un retour inconditionnel ; l'AC2 parle de transaction
  « proposable ».
- **Écartée** : lire `hasMore` et paginer (hors périmètre de #526).
- **Réversible** : oui — la résolution de #649 réécrit ces deux textes (dit dans la fiche).

### C-28-5-21 — Cohérence avec la 27-4 : AC13 et libellé `already-rejected` réécrit

- **Contexte** : l'epic demande la cohérence avec la 27-4 ; la fiche n'avait ni AC ni exclusion (R1) ; le libellé
  « … elle n’est plus proposée » devient une impasse dès qu'un geste existe.
- **Retenu** : AC13 (28-5a) — rejeter → annuler → rejeter de nouveau (B1) et accepter après annulation (B9, proposition
  `split`) ; le libellé est **complété** dans les quatre locales (28-5b), renvoyant au détail de l'import, repli de
  `failed-proposal-label.ts` égal au catalogue.
- **Écartée** : laisser le libellé tel quel (il tairait le geste au seul endroit où l'utilisateur bute sur le rejet).
- **Réversible** : oui.

### C-28-5-22 — Indication en fin de section, texte vrai pour tout rôle

- **Contexte** : « sous la liste » (manuel) n'était pas pinné (F10) ; « d'où son rejet s'annule » est faux pour la
  Consultation (R5).
- **Retenu** : paragraphe en fin de `<section>`, après le bloc des échecs partiels ; texte « … où son rejet peut être
  annulé si votre rôle le permet ».
- **Écartées** : avant les échecs partiels (les sépare de la liste qu'ils commentent) ; afficher l'indication selon
  le rôle (le composant ne reçoit pas le rôle : une prop de plus pour une phrase).
- **Réversible** : oui.

### C-28-5-23 — La lecture verrouillante : mutation M15 survivante assumée, pas de test à deux connexions

- **Contexte** : remplacer `find_by_id_for_account_for_update` par une lecture ordinaire survit aux tests séquentiels
  (R8, F6).
- **Retenu** : M15 inscrite et déclarée **survivante assumée** : la différence ne se voit que dans l'intervalle entre
  la relâche du verrou nommé et le `commit` d'un geste concurrent, que rien ne rend déterministe — angle mort déjà
  écrit au doc-comment de la lecture (Story 27-4).
- **Écartée** : un test à deux connexions (interleaving non déterministe : un test qui passerait au hasard).
- **Réversible** : oui.
- ⛔ **Remplacé** à la validation P2 (2026-10-13) par **C-28-5-25** (F2 = R2). Les trois prémisses étaient fausses :
  le doc-comment de la lecture (`repositories/reconciliation.rs:385-400`) n'écrit **aucun** angle mort — il dit
  pourquoi la lecture doit être courante ; la suppression d'import ne prend pas le verrou nommé, si bien que la
  fenêtre est sa transaction entière, qu'un test tient ouverte ; et un test déterministe sur ce patron existe à la
  27-4, qui a tué la mutation jumelle M11.

### C-28-5-24 — Ce qui va dans chaque partie : messages serveur en a, CHANGELOG et interface en b

- **Retenu** : la 28-5a porte les deux clés serveur (message du `409`, inscrit à G9 ; étiquette d'audit, exigée par
  `audit_label_registry.rs`) et `docs/api-external.md` ; la 28-5b porte les clés d'interface, le libellé réécrit, le
  manuel et le CHANGELOG — *Ajouté* (le geste et la route) et *Corrigé* (#526), un geste neuf étant un *Ajouté*
  (précédent 27-1, R9).
- **Écartée** : CHANGELOG en 28-5a (il annoncerait un geste que l'interface n'offre pas encore).
- **Réversible** : oui.

### Validation P2 de la 28-5 (2026-10-13) — choix C-28-5-25 à C-28-5-30

Remédiation des rapports `28-5-validate-p2-F.md` et `28-5-validate-p2-R.md` (Opus 5.5), par Opus 5.5, sur les
consignes de l'orchestrateur. La correction du code `OPTIMISTIC_LOCK_CONFLICT` (F1 = R1) n'est pas un choix : elle
est portée en note à C-28-5-4 et C-28-5-19.

### C-28-5-25 — Test B10 déterministe : M15 tuée (remplace C-28-5-23)

- **Contexte** : M15 (lecture ordinaire au lieu de `FOR UPDATE`) était déclarée survivante sur un argument faux, et
  le « disparue sous verrou → `404` » de l'AC3 n'avait aucun test (F2 = R2).
- **Retenu** : **B10** `unreject_of_a_transaction_deleted_under_the_lock_is_not_found`, patron de
  `post_reject_locking_reread_sees_a_deletion_committed_after_the_snapshot` (`reconciliation_e2e.rs:7225-7330`) :
  `DELETE` non validé, `…/unreject` en tâche, sonde `attendre_une_requete_en_cours` sur
  `["bank_transactions", "bank_account_id = ?"]` (motifs communs au `SELECT … FOR UPDATE` et à l'`UPDATE` prescrit,
  absents de la lecture hors verrou), deux observations à 500 ms (C-27-4-26), `commit`, puis `404 NOT_FOUND` et
  aucune trace. Sous M15 : `409 OPTIMISTIC_LOCK_CONFLICT`. M15 passe à « tuée par B10 » ; AC12 (a) n'admet plus de
  survivante assumée.
- **Écartées** : sonde sur `["bank_transactions", "FOR UPDATE"]` (sous M15, la sonde n'observerait jamais rien et le
  test rougirait sur son **montage**, non sur le statut : la mutation serait tuée pour une mauvaise raison) ;
  garder M15 survivante (argument réfuté).
- **Réversible** : oui. ⚠️ Mêmes réserves que le test de la 27-4, à reprendre au doc-comment : la sonde voit une
  requête en cours, non en attente (privilège `PROCESS` absent sur le poste de dev) ; MariaDB ≥ 11.6
  (`innodb_snapshot_isolation`).

### C-28-5-26 — Le doc-comment de `bank-import-deletion.ts` passe à la 28-5b

- **Contexte** : la 28-5a comptait six sites de premier niveau, dont un doc-comment frontend, au-delà du seuil de
  cinq qui a motivé le découpage (F5), et ce fichier lui imposait un gate frontend (R7).
- **Retenu** : la 28-5b, qui touche déjà `frontend/features/bank-import`, complète le doc-comment ; la 28-5a ne
  touche aucun fichier frontend (cinq sites). Entre les deux PR, le commentaire est incomplet, non faux.
- **Écartée** : écrire que les modules touchés en commentaire seul ne comptent pas (une exception de plus à la règle
  de découpage, pour un commentaire).
- **Réversible** : oui.

### C-28-5-27 — Renvoi de `docs/api-external.md` au paragraphe du refus `ALREADY_REJECTED`

- **Contexte** : la phrase de renvoi était prescrite l. 552 (ordre de tri du rejet), alors que la cause de
  `RECONCILIATION_ALREADY_REJECTED` est décrite l. 540 (R6).
- **Retenu** : la phrase va au paragraphe « Transaction disparue, rapprochée ou rejetée » (l. 540), seul.
- **Écartée** : aux deux paragraphes (une redite).
- **Réversible** : oui.

### C-28-5-28 — `conclude_locked_attempt` : trois retouches de texte autorisées

- **Contexte** : « corps intouché » figeait la liaison `Ok(journal_entry_id)`, fausse pour `UnrejectOutcome` (R12), et
  deux commentaires internes qui nomment deux appelants sur trois (l. 4366, 4384 — F7).
- **Retenu** : liaison renommée `Ok(value)` ; les deux commentaires nomment les trois appelants ; logique et `match`
  inchangés ; motif `` post_split` n' `` ajouté au grep du symptôme de l'AC10.
- **Écartée** : laisser les noms (un commentaire faux sur un auxiliaire partagé).
- **Réversible** : oui.

### C-28-5-29 — Ordre de l'epic : 28-5a, puis 28-6 ∥ 28-5b, puis 28-7 — décision de l'orchestrateur

- **Contexte** : le § *Ordre* de l'epic sérialisait « 28-5, 28-6, 28-7 » sans connaître le découpage ; la fiche
  laissait l'ordre 28-5b / 28-6 ouvert (F6 = R10).
- **Retenu** : la 28-5a seule touche `routes/reconciliation.rs` et `docs/api-external.md` : elle passe d'abord ; la
  28-6 et la 28-5b se développent ensuite en parallèle, avec union au rebase sur `failed-proposal-label.ts`, son
  test, `sitesTotal` (recompté par la seconde fusionnée) et la section `reconciliation-*` des catalogues ; la 28-7
  vient après la 28-6.
- **Réversible** : oui, tant qu'aucune n'est fusionnée.

### C-28-5-30 — Interblocage avec la remise à zéro d'onboarding : nommé, non rejoué

- **Contexte** : l'argument « aucun cycle » (C-28-5-4) omettait `kesh_seed::reset_demo`, qui prend `audit_log` puis
  `bank_transactions` — l'ordre inverse de l'annulation (R3) ; et il était écrit comme absolu alors que la 28-4a
  rouvrira la question (F9).
- **Retenu** : l'angle mort est **écrit** — au point (iv) d'`audit_route_registry.rs` et dans la fiche : victime
  désignée, l'annulation rend `500` sans rien de partiel, comme `post_reject` (préexistant) ; rare (installation de
  démonstration ou d'avant production) ; argument à revérifier au rebase sur la 28-4a (W30). Classement
  `SansEcritureAuJournal` maintenu.
- **Écartée** : rejouer la route sur 1213 (une enveloppe de rejeu pour un cas que `post_reject`, voisin, assume déjà ;
  le geste se refait d'un clic).
- **Réversible** : oui.

### C-28-5-31 — B8 réutilise le constructeur de transaction des tests de la 27-4 (28-5a, dev)

- **Contexte** : le `mod unreject_classification_tests` (B8) a besoin d'une `BankTransaction` de test ;
  `batch_classification_tests::tx` (27-4) la construit déjà, privée à son module.
- **Retenu** : `tx` passe en `pub(super)` (un mot et une ligne de doc) et B8 l'importe — règle DRY du `CLAUDE.md`.
- **Écartée** : recopier les 25 lignes du constructeur dans le module neuf.
- **Réversible** : oui. ⚠️ Région hors du tableau des régions de la fiche (une ligne, `batch_classification_tests`) :
  union au rebase si la 28-6 y touche.

### C-28-5-32 — L'auxiliaire `bank_account_of_transaction` vit dans la section neuve (28-5a, dev)

- **Contexte** : C-28-5-18 prescrit l'auxiliaire sans fixer sa place.
- **Retenu** : placé en tête de la section neuve, après `cancel_reconciliation_once` ; son doc-comment reprend mot
  pour mot le commentaire supprimé de `cancel` (« lu hors verrou, scopé par société : il ne change jamais pour une
  transaction ») et la note « lecture non verrouillante en autocommit ».
- **Réversible** : oui.

### C-28-5-33 — Gates frontend exécutés bien qu'aucun fichier frontend ne soit touché (28-5a, dev)

- **Contexte** : la 28-5a ne touche aucun fichier sous `frontend/` (C-28-5-26), mais plusieurs gardes Vitest
  (`i18n-keys.test.ts`, `i18n-entrees-a-variables.test.ts`…) **lisent** les catalogues `crates/kesh-i18n/locales/*`,
  auxquels la 28-5a ajoute deux clés.
- **Retenu** : `check`, `lint-i18n-ownership`, `test:unit` et `build` exécutés au dernier commit de code, sous verrou.
- **Écartée** : s'en tenir à la lettre de T7a (aucun gate frontend exigé) — une garde Vitest pouvait rougir sans
  fichier frontend modifié.
- **Réversible** : sans objet.

### C-28-5-34 — La garde « compte clos » de la 28-4 ne verrouille pas le compte : fiche corrigée (28-5a)

- **Contexte** : le § *Séquence de la route* de la fiche annonçait qu'à cause de la 28-4 (W30) `unreject` prendrait
  `bank_accounts FOR UPDATE` après la ligne de `bank_transactions`. La conception de la 28-4 a changé (C-28-4-21) :
  la garde lit la ligne du compte par une lecture cohérente non verrouillante. Note de l'orchestrateur, 2026-10-13.
- **Retenu** : la phrase de la fiche est réécrite (« la garde de la 28-4 ne prendra aucun verrou, cf. C-28-4-21 »),
  dans un commit de fiche séparé ; le code de la 28-5a n'est pas touché ; la relecture de l'argument d'interblocage
  au rebase sur la 28-4a reste prescrite.
- **Réversible** : oui (texte de fiche).

### C-28-5-35 — Terminologie du refus `transaction-not-rejected` : catalogues inchangés (28-5a, revue P1, A-LOW-1)

- **Contexte** : la lentille A relève que le message neuf dit « rapprochée » (fr-CH) et « abgestimmt » (de-CH), quand
  la clé voisine `reconciliation-errors-already-reconciled` dit « réconciliée » et « abgeglichen ».
- **Retenu** (décision de l'orchestrateur) : **les catalogues ne changent pas**. Les textes sont ceux que prescrit
  la fiche (§ *Textes*, validée), et le glossaire (`docs/i18n-glossaire.md`) admet « rapprochée » et « Abstimmung /
  abgestimmt » pour la réconciliation. Aucune ligne de production ne change : la boucle de revue reste close.
- **Écartée** : aligner sur « abgeglichen » / « réconciliée » — un changement de production en fin de boucle, pour une
  nuance que le glossaire tranche déjà.
- **Réversible** : oui (texte de catalogue, avec son repli G9).

## Story 28-1 — Les boutons d'action destructive se voient (#640)

Fiche : `28-1-boutons-destructifs.md`. Choix faits à la création de la fiche (2026-10-13), en autonomie, sur
`6f971d76` (rétrospective de l'Epic 27), branche `story/28-1`.

### C-28-1-1 — La cause : le jeton `destructive` n'existe pas dans le thème (prémisse de l'issue corrigée)

- **Contexte** : #640 et l'epic attribuent le défaut à `bg-destructive/10` « fond rouge à 10 % sous texte rouge ».
- **Constat au sol** : `frontend/src/app.css:10-28` ne définit aucun `--color-destructive` ; Tailwind v4 n'engendre
  pas de classe pour un jeton absent ; le build du 2026-10-06 contient **0** règle `destructive` dans ses sept
  feuilles. Le bouton n'a ni fond ni couleur : il hérite du texte courant. 201 occurrences du jeton dans 42
  fichiers sont dans le même cas (33 bandeaux d'erreur compris).
  *(Rectifié en validation P1 : 32 bandeaux hors `components/ui/` — C-28-1-25.)*
- **Retenu** : corriger à la racine — définir le jeton (AC1) **et** refaire la variante (AC2).
- **Écartées** : ne corriger que la variante avec des couleurs de palette (`bg-red-700`) — laisserait les autres
  usages du jeton (201 occurrences, 42 fichiers) sans effet, et le défaut d'origine en place.
- **Réversible** : oui.

### C-28-1-2 — Fond plein plutôt que contour

- **Contexte** : l'epic laisse le choix « fond plein ou contour marqué ».
- **Retenu** : fond plein `bg-destructive`, texte blanc — la convention shadcn d'origine, la plus forte distinction
  d'avec *Annuler* (`outline`) et d'avec un texte d'erreur, et la seule qui rende l'action irréversible saillante
  dans un dialogue.
- **Écartées** : contour rouge (bordure + texte) — reste proche d'un cadre d'erreur (`border-destructive
  text-destructive`, 33 bandeaux) ; deux variantes (pleine + contour) — élargit la story sans besoin établi.
  *(Rectifié en validation P1 : 32 bandeaux — C-28-1-25.)*
- **Réversible** : oui (une ligne).

### C-28-1-3 — `#b91c1c` (rouge 700), distinct de `--color-error` (`#dc2626`)

- **Contexte** : réutiliser `--color-error` était le plus simple.
- **Mesuré** : `#dc2626` tient sur le bouton (blanc dessus 4,83:1) mais **échoue** sur les 33 bandeaux
  `bg-destructive/10 text-destructive` (4,13:1 < 4,5) que le jeton activera ; `#b91c1c` : bouton 6,47:1, bandeau
  `/10` 5,45:1, `/20` 4,59:1, texte sur blanc 6,47:1.
  *(Rectifié en validation P1 : 32 bandeaux, non 33 ; `/20` vaut 4,57:1, et n'est plus cité — C-28-1-25.)*
  *(Rectifié en validation P2 : par la méthode du helper (mélanges non arrondis, arrondi au centième,
  C-28-1-27), `#dc2626` sur son bandeau vaut 4,14:1, le bandeau `/10` de `#b91c1c` 5,46:1 ; `#b91c1c` est le
  `red-700` de Tailwind **v3**, non celui de la v4 installée.)*
- **Retenu** : `--color-destructive: #b91c1c`, `--color-destructive-foreground: #ffffff`. `--color-error` inchangé.
- **Écartées** : `#dc2626` (AA perdu sur les bandeaux, et les specs axe pourraient rougir) ; `#991b1b` au repos
  (8,31:1 mais trop proche du brun, survol plus foncé impossible à distinguer).
- **Réversible** : oui (une valeur).

### C-28-1-4 — Survol : un jeton dédié plus foncé

- **Retenu** : `--color-destructive-hover: #991b1b` (blanc dessus 8,31:1), `hover:bg-destructive-hover`.
- **Écartées** : `hover:bg-destructive/90` (éclaircit, 5,58:1 mesuré, différence peu perceptible et contraste en
  baisse) ; pas de survol (la variante `default` n'en a pas sur `<button>`, mais l'issue demande des états
  distincts).
  *(Rectifié en validation P2 : `/90` vaut 5,59:1 par la méthode du helper, C-28-1-27.)*
- **Réversible** : oui.

### C-28-1-5 — Désactivé : l'opacité commune, exemptée par WCAG, et « grisé » → « désactivé » au manuel

- **Contexte** : la consigne demande un contraste mesuré pour l'état désactivé.
- **Mesuré** : `disabled:opacity-50` de la base rend un fond effectif `#dc8e8e`, texte 2,53:1. WCAG 1.4.3 exempte
  le texte d'un composant inactif ; l'état reste distinct du repos (6,47:1).
  *(Rectifié en validation P2 : 2,54:1 sur le mélange non arrondi `(220 ; 141,5 ; 141,5)`, C-28-1-27.)*
- **Retenu** : garder l'opacité commune à toutes les variantes ; le manuel cessait d'être exact en disant
  « grisé » (c'est un rouge estompé) — deux phrases passent à « désactivé » (AC10).
- **Écartées** : un désactivé gris dédié (`disabled:bg-secondary`) — une seconde règle pour une seule variante,
  et l'action perd son identité ; un désactivé à 4,5:1 — se confondrait avec l'état actif.
- **Réversible** : oui.

### C-28-1-6 — Focus : le contour global bleu, vérifié au style calculé

- **Contexte** : la base porte `outline-none` et `focus-visible:ring-ring/50` (jeton `ring` absent, sans effet).
- **Lecture** : la règle `:focus-visible { outline: 2px solid #3b82f6; outline-offset: 2px }` d'`app.css:62` est
  hors couche et l'emporte sur `outline-none` (`@layer utilities`). Contraste contre la page 3,68:1 (≥ 3).
- **Retenu** : compter sur elle, ajouter `focus-visible:ring-destructive/40` à la variante, et **le prouver** à
  l'E2E (AC8 a) ; correction prescrite si la lecture est fausse.
- **Réversible** : oui.

### C-28-1-7 — « Mode sombre » : il n'existe pas ; la variante ne porte aucune classe `dark:`

- **Contexte** : l'epic demande des captures clair et sombre. Kesh n'a pas de thème sombre
  (`sonner.svelte:3`), mais le variant `dark` de Tailwind v4 est `@media (prefers-color-scheme: dark)` — vérifié
  dans le build (`dark:bg-amber-950/30`, `dark:bg-green-900`… émis sous cette requête).
- **Piège** : la variante actuelle porte `dark:bg-destructive/20` ; une fois le jeton défini, elle rendrait le
  bouton **pâle à nouveau** chez qui a son système en sombre.
- **Retenu** : aucune classe `dark:` dans la variante (garde AC7, mutation M3) ; l'E2E rejoue chaque scénario sous
  `colorScheme: 'dark'` émulé et exige le même rendu.
- **Écartée** : instaurer un thème sombre — hors périmètre.
- **Réversible** : oui.

### C-28-1-8 — Les autres usages du jeton deviennent rouges, sans édition de site

- **Contexte** : définir le jeton change l'apparence des bandeaux, textes et bordures d'erreur, et des champs
  `aria-invalid`.
- **Retenu** : l'accepter — c'est leur intention d'origine, et le contraste est garanti par C-28-1-3. Contrôle
  indépendant : les dix specs E2E qui lancent axe restent vertes.
  *(Rectifié en validation P1 : 16 fichiers exercent `color-contrast` — C-28-1-25.)*
- **Écartée** : neutraliser ces usages (les écrire en `text-error`) — des dizaines d'éditions pour garder un défaut.
- **Réversible** : oui (avec le jeton).

### C-28-1-9 — Sites stylés à la main : B et D migrés, C et E laissés et inscrits

- **Retenu** : migrer vers la variante les 2 `<Button class="bg-red-600 …">` (DRY) et les 3 `<button>` sans bordure
  qui se lisent comme du texte — dont *Annuler le rapprochement*, sur l'écran de la recette. Laisser les 6
  déclencheurs **bordés** (ils se lisent comme des boutons) et les 3 icônes ✕ de retrait de ligne, inscrits avec
  leur motif à la table fermée de la garde AC6.
- **Écartées** : tout migrer (6 boutons rouges pleins de plus dans des lignes de tableau, chargés visuellement, et
  plus de cinq modules touchés) ; une variante `destructive-outline` pour la classe C (élargit la story ; à
  proposer par issue si l'harmonisation est voulue).
  *(Remplacé en validation P2 par C-28-1-26 : les classes B et D ne sont plus migrées ; elles sont inscrites à
  la table de l'AC6, avec C et E, et forment l'angle mort 11.)*
- **Réversible** : oui.

### C-28-1-10 — Recette visuelle : style calculé + axe + captures non comparées

- **Contexte** : B2 de la rétrospective de l'Epic 27 — un test qui trouve un bouton ne prouve pas qu'on le voit.
- **Retenu** : une spec Playwright qui lit `getComputedStyle` (fond non transparent, valeur exacte, contraste
  recalculé ≥ 4,5, survol, focus, désactivé, clair et sombre), lance axe `color-contrast` sur les dialogues, et
  prend des captures **avant** ses assertions — « avant » sur le code inchangé (T0), « après » à la fin — dans
  `kesh-gate-logs/28-1-captures/`, hors dépôt.
  *(Précisé en validation P2 : captures **par état**, prises après les assertions de l'état, dans le `finally` de
  chaque step ; l'E2E ne recalcule plus de contraste — C-28-1-29, C-28-1-31.)*
- **Écartée** : `toHaveScreenshot` — références au pixel dépendantes de la plateforme et des polices, PNG à
  versionner et à régénérer, aucun précédent au dépôt, et la CI ne lance pas l'E2E ; il mesure la ressemblance,
  non la visibilité. Les propriétés calculées visent exactement le défaut (fond transparent).
- **Réversible** : oui.

### C-28-1-11 — Deux gardes Vitest, parce que la CI n'exécute pas l'E2E

- **Retenu** : AC7 (variante, jetons définis, contrastes recalculés depuis `app.css`) et AC6 (inventaire clos des
  boutons rouges posés à la main, par `svelte/compiler`, patron `bank-name-inventory.test.ts`). La CI lance
  `npm run test:unit` (`ci.yml:214`), non Playwright (#577).
- **Écartée** : une garde par expression régulière sur les balises (`onclick={() => …}` contient `>`).
- **Réversible** : oui.

### C-28-1-12 — Un seul calcul de contraste, dans `tests/e2e/helpers/contrast.ts`

- **Retenu** : fonctions pures partagées par la spec E2E et la garde AC7, testées par `contrast.test.ts` (Vitest
  ramasse `tests/**/*.test.ts`, Playwright l'ignore par `testMatch`).
  *(Rectifié en validation P2 : la spec E2E n'en importe plus rien ; seul consommateur, la garde AC7 (3) —
  C-28-1-31.)*
- **Écartées** : deux copies (DRY) ; le placer dans `src/lib/` (code de test dans le produit, et aucune spec E2E
  n'importe aujourd'hui de `src/`).
- **Réversible** : oui.

### C-28-1-13 — Les neuf autres jetons shadcn absents : hors périmètre, issue à ouvrir

- **Contexte** : `muted`, `muted-foreground`, `foreground`, `background`, `ring`, `input`, `accent`,
  `accent-foreground`, `card` ne sont pas définis non plus ; `outline` et `ghost` n'ont pas de survol.
- **Retenu** : ne pas les traiter ici (ils ne masquent aucune fonction : *Annuler* garde sa bordure) ; demander à
  l'orchestrateur une issue P3. Même chose pour la suppression sans confirmation d'une personne de contact.
- **Réversible** : —.
- **Précision (validation P1, 2026-10-13)** : les deux issues existent — **#645** (jetons non définis, P3 ; la
  lentille R y ajoute `error-soft`, `warning-soft`, `danger`) et **#646** (confirmation, P3). Un élargissement de
  la 28-1 à ces deux sujets a été envisagé puis **annulé** par l'orchestrateur (C-28-1-17) : le choix tient.

### C-28-1-14 — La spec UX nomme le rouge d'action

- **Retenu** : une ligne à la palette de `ux-design-specification.md` et deux ratios mesurés (AC11) : la palette
  est la référence du thème, elle ne doit pas ignorer un jeton du thème.
- **Réversible** : oui.

### C-28-1-15 — Pas de découpage *(remplacé par C-28-1-24 en validation P1)*

- **Retenu** : cinq modules de premier niveau au plus, aucune règle métier, aucune migration ; les 24 sites de la
  variante ne sont pas édités.
- **Réversible** : —.

### C-28-1-16 — Le montage d'import des E2E extrait dans un helper

- **Retenu** : `login`, `mount`, `importByScreen` de `bank-import-delete.spec.ts:35-85` passent dans
  `tests/e2e/helpers/bank-import-fixtures.ts`, importé par cette spec et par la neuve (DRY ; une spec importée par
  une autre y enregistrerait ses tests deux fois).
- **Réversible** : oui.

---

### Choix de la remédiation de la validation P1 (2026-10-13)

Rapports `kesh-gate-logs/28-1-validate-p1-{R,F}.md` (hors dépôt). Bilan au Change Log de la fiche.

### C-28-1-17 — Périmètre : #640 seulement (décision de l'orchestrateur)

- **Contexte** : l'orchestrateur avait proposé aux lentilles P1 d'examiner un élargissement — définir les neuf
  autres jetons shadcn, et ajouter une confirmation à la suppression d'une personne de contact. Les lentilles ont
  mesuré qu'il n'est pas neutre : `input` sert à la fois de bordure et de fond désactivé (R1), le risque visuel
  touche l'écran entier sans filet (R2), `muted-foreground` au ton shadcn échoue AA sur `muted` (R3, F3), les
  `dark:` inertes s'activeraient (F4), la confirmation engage i18n, test de composant et manuel (R7, F5), et le
  tout porte la story au-delà de cinq modules (R6).
- **Retenu (décision de l'orchestrateur, non de l'agent)** : l'élargissement est **annulé**. La 28-1 se limite
  au jeton `destructive` (et `-foreground`, `-hover`), à la variante, aux boutons rouges stylés à la main
  (AC5), aux gardes et à la recette visuelle. Les autres jetons → **#645** ; la confirmation → **#646**. La fiche
  les cite comme angles morts 1 et 2, avec renvoi.
  *(Rectifié en validation P3 : l'AC5 n'édite plus aucun site, les boutons stylés à la main restent tels quels,
  inscrits à la table de l'AC6 — C-28-1-26.)*
- **Effet sur les findings** : F3, F5, R1, R2, R3, R7, R8 se ferment par le retrait du périmètre ; F4 n'est
  traité que pour la part qui touche la variante. Le commentaire déjà posté sur #640 (« la Story 28-1 traite
  l'ensemble ») est devenu faux : à rectifier par l'orchestrateur.
- **Réversible** : oui — #645 et #646 restent ouvertes et portent les mesures des lentilles.

### C-28-1-18 — Les gardes AC7 (1)(2) lisent la variante seule

- **Contexte** (F1, HIGH) : `buttonVariants({ variant: 'destructive' })` rend base + variante fusionnées ; la base
  porte `dark:aria-invalid:…`, `bg-clip-padding`, `text-sm`, `focus-visible:border-ring` : « aucun `dark:` » et
  « tout jeton est défini » auraient rougi au premier lancement, sur des classes hors story.
- **Retenu** : lire `buttonVariants.variants.variant.destructive` (exposé par tailwind-variants,
  `chunk-RZF76H2U.js:254`) ; à défaut, la ligne `destructive:` de `button.svelte`. La sortie fusionnée ne sert qu'à
  (1-bis) : présence des classes de la variante, absence de `focus-visible:border-ring` remplacée.
- **Écartée** : filtrer la sortie fusionnée en retirant les classes de la base — une liste de plus à tenir, qui
  dérive avec `button.svelte`.
- **Réversible** : oui.

### C-28-1-19 — Valeurs lues après stabilisation ; `parseRgb` refuse ce qu'il ne sait pas lire

- **Contexte** (F2 = R9, MEDIUM ; F8, LOW) : la base porte `transition-all` (150 ms) ; une lecture juste après
  `hover()`/`focus` rend une couleur intermédiaire. Et Chromium rend une couleur `color-mix(in oklab, …)` sous une
  syntaxe `oklab(`/`color(` que `parseRgb` ne lit pas.
- **Retenu** : `expect(locator).toHaveCSS(...)` pour toute valeur ; `expect.poll` quand la valeur alimente un
  calcul de contraste. `parseRgb` lit `rgb(`/`rgba(` et **lève** sur toute autre syntaxe en nommant la chaîne ;
  tests unitaires des deux formes, des deux refus et de trois ratios connus.
- **Écartées** : `page.waitForTimeout(200)` (dépend de la durée, fragile) ; désactiver les transitions par une
  feuille injectée (la spec mesurerait un rendu que l'utilisateur n'a pas).
  *(Rectifié en validation P2 : `parseRgb` et `expect.poll` sont retirés — l'E2E n'assert que des valeurs par
  `expect.soft(…).toHaveCSS`, et ne recalcule aucun contraste, C-28-1-31.)*
- **Réversible** : oui.

### C-28-1-20 — La variante fixe sa propre bordure de focus

- **Contexte** (R11, MEDIUM) : la base porte `focus-visible:border-ring` ; `ring` n'est pas défini (#645), si bien
  que la bordure de focus reste transparente aujourd'hui — et deviendrait **bleue** sur un bouton rouge plein le
  jour où #645 définira `ring`, sans que rien ne le décide.
- **Retenu** : `focus-visible:border-destructive` dans la variante (tailwind-merge remplace la classe de la base).
  Le focus de la variante ne dépend plus d'aucun jeton absent : anneau rouge 40 % (non asserté : syntaxe
  `color-mix`), bordure `rgb(185, 28, 28)` (assertée, AC3, AC8 (a)), contour global bleu (asserté). Gardes AC7 (1)
  et (1-bis), mutation M19.
- **Écartées** : retirer `focus-visible:border-ring` de la base (touche toutes les variantes, relève de #645) ;
  ne rien faire et l'écrire en angle mort (le défaut se déclencherait par une story sans rapport).
- **Réversible** : oui (une classe).

### C-28-1-21 — *Annuler le rapprochement* du détail d'import passe à `outline`, comme sur la fiche facture

- **Contexte** (R14, LOW, à trancher) : la fiche prévoyait `destructive xs` pour `detail-tx-cancel-reconciliation`,
  alors que le même bouton est `outline sm` sur la fiche de la facture (`InvoiceSettlements.svelte:106-111`) et que
  le manuel dit « le même bouton apparaît aussi sur la fiche de la facture » (`user-manual.tex:2019-2020`).
- **Retenu** : `<Button variant="outline" size="sm" class="ml-2">`. C'est un déclencheur : il ouvre
  `CancelReconciliationDialog`, dont la confirmation est `destructive` (classe A) — même hiérarchie que les
  déclencheurs neutres des Dev Notes. Le défaut de #640 (« se lit comme du texte », souligné à côté d'un lien) est
  corrigé par la forme de bouton ; le manuel reste vrai ; aucun module de plus.
- **Écartées** : `destructive` sur les deux écrans (un bouton rouge plein par ligne de tableau — écarté pour la
  classe C par C-28-1-9 —, et `features/invoices` en huitième module) ; garder les deux formes et documenter la
  différence (le manuel promet l'identité).
- **Trié, non tranché** : la confirmation `invoice-settlement-cancel-confirm` (`invoices/[id]/+page.svelte:1666-1674`)
  est `default`, quand les deux autres confirmations de la page sont `destructive`. Elle se voit ; ce n'est pas
  #640. Signalée à l'orchestrateur, sans demande d'issue.
  *(**Annulé** en validation P2 par C-28-1-26 : avec `outline`, sans fond ni survol tant que #645 n'est pas fait
  et une bordure à 1,23:1, le bouton devenait moins visible qu'aujourd'hui — F8, R4. Il reste un `<button>` brut,
  qui devient rouge par l'AC4 ; angle mort 11.)*
- **Réversible** : oui (une valeur de `variant`).

### C-28-1-22 — Garde AC6 : clé non ambiguë, seconde table des sites migrés, assertion (e)

- **Contexte** (R10, MEDIUM ; F9, LOW) : la clé `fichier:élément` confondait les deux `<button>` de
  `supplier-invoices/[id]` ; un site migré rendu à un `<button class="ml-2">` nu ou à un `Button` sans `variant`
  ne rougissait rien ; deux des cinq sites n'étaient vus par aucun test visuel.
- **Retenu** : clé `chemin#data-testid` (texte source, gabarit compris), sinon `chemin@élément:rang` ; échec si
  deux clés coïncident ; pas de numéro de ligne. Seconde table fermée des cinq sites migrés (élément `Button`,
  `variant` et `class` littéraux), mutation M18. Assertion (e) dans `reconciliation-cancel.spec.ts:139`
  (`border-top-width: 1px`, `border-top-color` de `--color-border` — non `border-top-style`, déjà `solid` par le
  préflight). `contact-person-delete-{id}` : par transitivité (garde structurelle + scénario (c)), angle mort 8.
- **Écartées** : clé par numéro de ligne (rougit sur toute édition sans rapport) ; monter une personne de contact
  en E2E (dialogue d'édition, montage lourd pour un bouton que la garde structurelle tient déjà).
  *(Rectifié en validation P2 : seconde table « sites migrés », M18 et assertion (e) retirées avec la migration
  (C-28-1-26) ; la clé s'écrit par `source.slice` de la valeur, gabarit JavaScript compris (C-28-1-28) ; la
  table compte quatorze entrées.)*
- **Réversible** : oui.

### C-28-1-23 — Garde d'ensemble clos des classes sans jeton, facultative, en table fermée renvoyée à #645

- **Contexte** (R4, MEDIUM) : « neuf jetons » était une liste ; la lentille R en a trouvé trois de plus
  (`error-soft`, `warning-soft`, `danger`). Une liste de formes ne révèle pas ce qu'elle omet.
- **Retenu** : AC7 (4), **facultatif** (décision de l'orchestrateur) : l'inventaire des classes de couleur que
  **Tailwind lui-même** ne résout pas (`@tailwindcss/node` : `compile` + `build`, ou
  `__unstable__loadDesignSystem(...).candidatesToCss`), comparé à une table fermée de douze noms renvoyés à #645 ;
  rouge dans les deux sens (nom neuf, nom devenu défini). La table ne se remplit pas depuis la sortie de la garde :
  tout écart avec la liste des lentilles est examiné nom par nom. Abandonnée, et dite, si l'API ne se charge pas.
- **Écartée** : une liste de noms à exclure (palette Tailwind, mots-clés, utilitaires non chromatiques) — une
  énumération de plus, ouverte par nature.
  *(Rectifié en validation P2 : treize noms, `surface-hover` compris ; la garde n'est plus facultative ; voie
  retenue `candidatesToCss`, dépendance déclarée, repli écrit — C-28-1-30.)*
- **Réversible** : oui (fichier de test isolé).

### C-28-1-24 — Sept modules : dérogation à la règle de découpage (remplace C-28-1-15)

- **Contexte** (R6, MEDIUM) : C-28-1-15 disait « cinq modules au plus » ; sa propre liste en comptait sept.
  Recompté sur le périmètre réduit : sept modules de code (`ui/button`, `app.css`, `features/contacts`,
  `features/bank-import`, `routes/(app)/bank-import`, `routes/(app)/settings`, `routes/design-system`).
- **Retenu** : ne pas découper, et l'écrire (fiche, § *Dérogation règle de splitting*) : cinq des sept modules
  reçoivent une édition mécanique d'une à trois lignes ; le découpage prescrit (patron puis rollout) scinderait
  la garde AC6, qui est l'inventaire des sites ; le rollout est revu fichier par fichier. Risque accepté écrit.
  Signal déclaré au Project Lead par l'orchestrateur.
- **Écartée** : 28-1 (jetons, variante, gardes) + 28-1b (cinq sites) — deux tables AC6 à écrire deux fois, et
  l'issue ne se clôt qu'à la seconde.
  *(Sans objet depuis la validation P2 : le périmètre réduit compte trois modules — C-28-1-26.)*
- **Réversible** : oui, tant que le développement n'a pas commencé.

### C-28-1-25 — Décomptes rectifiés (validation P1)

- **R13** : 33 → **32** bandeaux hors `components/ui/` (`grep -rn 'bg-destructive/10' src --include=*.svelte
  --include=*.ts | grep -v '\.test\.' | grep -v components/ui | wc -l` → 32 ; les 32 portent aussi
  `border-destructive` et `text-destructive`). Vaut pour C-28-1-1, C-28-1-2, C-28-1-3.
- **R12** : `/20` → **4,57:1** (mélange `(241 ; 209,6 ; 209,6)`), et retiré du commentaire d'`app.css` et de
  l'AC4 : plus aucun usage applicatif. Vaut pour C-28-1-3.
- **F7** : 5,45:1 est exact sur le mélange non arrondi `(248 ; 232,3 ; 232,3)` — ce que rend Chromium et ce que
  calcule `blend` ; 5,46 si l'on arrondit à `#f8e8e8`. Écrit à l'AC1.
- **R5** : « dix specs axe » → **19** fichiers à `.analyze(` ; **16** exercent `color-contrast` au moins une fois
  (15 sans `disableRules`, plus `invoices` par `:96` et `:231` — la lentille R en comptait 15) ; **3** la
  neutralisent (`users:134`, `email-templates:203`, `reminders:298`). Vaut pour C-28-1-8.
- **Bilan de la lentille R** : déclaré 2 HIGH / 8 MEDIUM / 6 LOW, titré 2 / 9 / 5 (même total de 16).
  *(Rectifié en validation P2 : la puce F7 inversait les deux valeurs — 5,45 est celle du pixel arrondi
  `#f8e8e8`, 5,46 celle du mélange non arrondi que calcule `blend` ; seule 5,46 est citée, C-28-1-27.)*
- **Réversible** : —.

### Choix de la remédiation de la validation P2 (2026-10-13)

Rapports `kesh-gate-logs/28-1-validate-p2-{R,F}.md` (hors dépôt). Bilan au Change Log de la fiche.

### C-28-1-26 — Périmètre réduit au strict nécessaire pour fermer #640 (décision de l'orchestrateur)

- **Contexte** : signal D5. La P1 avait rendu 3 HIGH et une douzaine de MEDIUM ; la P2 rend une douzaine de
  MEDIUM distincts, dont la majorité naissent de la remédiation P1 (C-28-1-19 à 25) — recyclage, non découverte de
  défauts d'origine. Le périmètre de la P1 comptait sept modules et une dérogation contestée (R6, F9).
- **Retenu (décision de l'orchestrateur, autonomie déléguée par Guy le 2026-10-13, non de l'agent)** : pas de
  découpage formel ; la story se limite au jeton (`--color-destructive`, `-foreground`, `-hover`, contrastes
  mesurés), à la variante `destructive` du `Button`, à ses gardes Vitest (AC7 (1) à (4)), à la table fermée de
  l'AC6 (sans migration), à la page `/design-system`, à une recette E2E centrée sur *Supprimer l'import* et son
  dialogue, aux manuels, à la spec UX et au CHANGELOG. **Retirés** : la migration des cinq boutons stylés à la
  main (ancien AC5), C-28-1-21 (*Annuler le rapprochement* en `outline`, annulé), la table « sites migrés », M18,
  les scénarios (c), (d), (e), l'exemple `xs`, les captures des 17 fichiers (C-28-1-28). Renvoyés à #645 :
  déclencheurs `ghost` destructifs (R2), bordure `outline` (R4), `surface-hover` (R1).
- **Effet** : trois modules (`lib/components/ui`, `app.css`, `routes/design-system`), la règle de découpage n'est
  plus franchie, C-28-1-24 est sans objet. Les cinq sites retirés forment l'angle mort 11 ; issue à ouvrir par
  l'orchestrateur.
  *(Rectifié en validation P3 : l'issue est **#648** ; un seul des cinq sites, `bank-import/[id]:143`, dépend du
  jeton — C-28-1-36.)*
- **Écartées** : découper en 28-1 (jetons, variante, gardes) et 28-1b (cinq sites) — la 28-1b ne porte pas le
  symptôme de #640 et peut naître de l'issue de l'angle mort 11 ; poursuivre la boucle sur le périmètre P1 —
  c'est le recyclage que D5 désigne.
- **Réversible** : oui (les sites retirés restent décrits aux Dev Notes et dans l'issue demandée).

### C-28-1-27 — Une seule méthode de calcul des ratios, celle du helper

- **Contexte** (F1 = R3, MEDIUM ; R10, LOW) : l'AC1 inversait 5,45 et 5,46 ; la fiche citait des valeurs
  arrondies au pixel (2,53, 4,13) à côté de valeurs non arrondies (4,57), et des valeurs tronquées.
- **Retenu** : tout ratio cité est celui que calculent `blend` + `contrastRatio` du helper — mélange à alpha
  **non arrondi**, ratio **arrondi au centième**. Valeurs : bandeau `/10` 5,46 ; désactivé 2,54 ; `#dc2626` sur son
  bandeau 4,14 ; `/90` 5,59 ; inchangées : 6,47, 8,31, 3,68, 3,76, 4,83 (`#dc2626` sur blanc), 1,23, 14,63. Le
  commentaire d'`app.css`, l'AC1, l'AC3, M4, la spec UX (avec le point décimal) et le CHANGELOG suivent.
- **Écartée** : l'arrondi au pixel 8 bits (ce que peint Chromium) — le test unitaire du helper rougirait contre
  la valeur citée, et deux méthodes dans une fiche est ce qui a produit l'inversion.
- **Réversible** : oui.

### C-28-1-28 — Les 24 usages de la variante vérifiés par construction ; clé de l'AC6 par le texte source

- **Contexte** (R5, MEDIUM) : l'epic et #640 demandent de « vérifier chacun des 17 sites à l'écran » ; l'E2E n'en
  montre que trois usages. (R11 = F3) : le `data-testid` de `ContactPersonsManager:116` est un gabarit JavaScript
  seul dans des accolades, et la fiche ne disait pas comment l'écrire en clé.
- **Retenu** : les 17 fichiers et 24 usages sont vérifiés **par construction** — la variante est leur seule
  source de style (AC7 (1)-(2)), et l'AC6 (f) garde qu'aucun ne lui passe de `class` (aucun aujourd'hui) ; l'E2E
  en montre un échantillon, `design-system:69`, `DeleteBankImportAction:73` et `:139`. Clé : `source.slice` de la
  **valeur** de l'attribut, guillemets exclus pour une valeur entre guillemets, accolades comprises pour une
  `ExpressionTag` seule, sans normalisation ; les cinq clés des classes B et D écrites à l'AC6.
- **Écartées** : un scénario de captures des quinze autres fichiers (montages lourds — factures, rapprochement,
  restauration —, pour une variante que la garde tient déjà) ; normaliser les gabarits en `{…}` (deux écritures
  d'un même site, et la tentation de réécrire l'attribut, que l'AC5 interdit).
- **Réversible** : oui.

### C-28-1-29 — Captures par état, prises même quand une assertion rougit ; une base neuve par test d'import

- **Contexte** (F5, F6, F7, MEDIUM ; F12, LOW) : une seule capture par scénario, prise avant les assertions, ne
  montrait ni survol, ni focus, ni désactivé ; en T0, un `expect` rouge arrêtait le test avant la capture du
  dialogue ; la seconde passe de (b) réimportait le même fichier et butait sur l'avertissement de doublon ;
  *Annuler* n'avait pas de `data-testid`.
- **Retenu** : chaque état est un `test.step` ; assertions de valeur en `expect.soft` ; capture dans le
  `finally` du step, après les assertions ; passes clair et sombre en **tests distincts** ; (b) remet la base à
  zéro en `beforeEach` (`seedTestState('with-company')`) et importe une fois par test ; *Annuler* visé par
  `[data-slot="dialog-footer"] [data-slot="button"]:not([data-testid])`, compté à 1 avant usage. Douze captures.
- **Écartées** : un import partagé en `beforeAll` (exige une page hors des fixtures de test, et un échec de la
  passe claire pourrait laisser l'import dans un état inattendu pour la sombre) ; un `data-testid` neuf sur
  *Annuler* (édite `features/bank-import`, un module de plus pour un sélecteur) ; toutes les captures avant toute
  assertion (elles montreraient des états non stabilisés).
  *(Complété en validation P3 : délais fixés — C-28-1-32 ; captures cadrées au-delà de la boîte de bordure,
  pointeur éloigné avant le focus — C-28-1-33.)*
- **Réversible** : oui.

### C-28-1-30 — AC7 (4) : `candidatesToCss` de `@tailwindcss/node`, déclaré ; plus facultative

- **Contexte** (F13, LOW ; R1 = F2, MEDIUM) : `@tailwindcss/node` n'est qu'une dépendance transitive ; `compile`
  exige des options tues et son `build` est cumulatif ; la frontière du relevé n'était pas fixée ; la table
  attendue manquait `surface-hover`. L'orchestrateur range la garde parmi ce qui est **gardé**.
- **Retenu** : `__unstable__loadDesignSystem(css, { base })` puis `candidatesToCss` ; `@tailwindcss/node` en
  `devDependencies` à la version exacte de `tailwindcss` ; frontière = début de jeton (séparateurs
  `` [\s"'`{}()] ``) ; table de treize noms ; repli par les `--color-*` d'`app.css` et de `tailwindcss/theme.css` si
  l'API ne se charge pas, écrit au Debug Log comme moins sûr. La garde n'est plus facultative ; M20 est
  inconditionnelle.
- **Écartées** : `compile` + `build` (ne décide pas candidat par candidat) ; s'appuyer sur la dépendance
  transitive (une mise à jour de `@tailwindcss/vite` peut la retirer sans signal).
  *(Rectifié en validation P3 : le `=` s'ajoute aux séparateurs — C-28-1-34 ; `@tailwindcss/node` prend le même
  spécificateur que `tailwindcss`, non la version exacte, sous une assertion de version — C-28-1-35.)*
- **Réversible** : oui (un fichier de test, une ligne de `package.json`).

### C-28-1-31 — L'E2E n'assert que des valeurs ; le helper sert la seule garde AC7 (3)

- **Contexte** (F15, LOW) : un contraste calculé sur une valeur que l'assertion précédente vient de fixer est une
  constante ; il ne tue rien de plus.
- **Retenu** : l'E2E assert les valeurs (`toHaveCSS`) et lance axe dans le dialogue ; les contrastes sont
  contrôlés par l'AC7 (3), depuis `app.css`. Le helper perd `parseRgb` (plus de consommateur) et gagne
  `parseHex` ; il reste à `tests/e2e/helpers/contrast.ts`, chemin vérifié ramassé par Vitest et ignoré par
  Playwright. Ses tests portent des cas « foncé en premier » (F4 : M16).
- **Écartées** : garder le calcul en E2E « pour la forme » (un test qui prouve moins) ; déplacer le helper dans
  `src/lib/` (code de test dans le produit, C-28-1-12).
- **Réversible** : oui.

### Choix de la remédiation de la validation P3 (2026-10-13)

Rapports `kesh-gate-logs/28-1-validate-p3-{R,F}.md` (hors dépôt). Bilan au Change Log de la fiche.

### C-28-1-32 — Délais de la spec E2E : 120 s par test, 2 s par assertion de valeur

- **Contexte** (F1 = R2, MEDIUM) : `playwright.config.ts` ne fixe aucun délai ; défauts de Playwright, 30 s par
  test et 5 s par assertion. En phase rouge (T0), cinq assertions de (a) expirent (≈ 25 s), trois de (b) (≈ 15 s,
  plus montage) : le test serait interrompu avant ses derniers steps, et les « douze captures » de C-28-1-29 ne
  seraient pas toutes prises.
- **Retenu** : `test.describe.configure({ timeout: 120_000 })` en tête de spec ; toute assertion de valeur par
  `const softExpect = expect.configure({ soft: true, timeout: 2_000 })` (la transition est de 150 ms) ; les
  assertions de montage gardent le délai par défaut et ne sont pas `soft`. Un test fini en timeout en T0 n'est pas
  le rouge attendu et se documente.
- **Écartées** : `test.slow()` seul (triple le délai à 90 s, sans borner les assertions : la marge dépend du
  montage) ; un délai global dans `playwright.config.ts` (touche toutes les specs pour le besoin d'une seule).
- **Réversible** : oui (deux lignes de la spec).

### C-28-1-33 — Captures cadrées au-delà de la boîte de bordure ; pointeur éloigné avant le focus

- **Contexte** (F2, MEDIUM ; F5, LOW) : `locator.screenshot()` rogne à la `boundingBox()` de l'élément
  (`playwright-core/lib/server/screenshotter.js`, `screenshotElement`) ; l'anneau `ring-3` et le contour
  `:focus-visible` (2 px décalés de 2 px) sont hors de cette boîte, si bien que `a-focus` ne pouvait montrer ni
  l'un ni l'autre — alors que l'AC3 et l'angle mort 10 s'appuient sur elle. Et le pointeur, resté sur le bouton
  après `hover()`, laissait le fond de survol sur la capture du focus.
- **Retenu** : les quatre captures de (a) et `b-bouton` par `page.screenshot({ clip })` sur la boîte du bouton
  élargie de 8 px (bornée à 0), après `scrollIntoViewIfNeeded()` ; `b-dialogue` reste `locator.screenshot()`.
  `page.mouse.move(0, 0)` avant le step de focus, et le fond asserté au repos dans ce step.
- **Écartées** : retirer « vu sur la capture » de l'AC3 et de l'angle mort 10 (l'anneau ne serait plus vu par
  personne) ; capturer le conteneur parent (cadre dépendant de la mise en page de `/design-system`).
- **Réversible** : oui (un helper de la spec).

### C-28-1-34 — AC7 (4) : le `=` est un séparateur ; auto-tests du relevé ; M22

- **Contexte** (F3 = R3, MEDIUM) : sans `=` parmi les séparateurs, `class:bg-accent={…}` livre `bg-accent=`, que
  l'ancre `$` rejette ; les 15 directives `class:` de couleur du dépôt (16 à préfixe de couleur, `class:text-right`
  compris) échappaient au relevé. Aucun des treize noms ne dépend aujourd'hui d'une directive seule, mais un
  `class:text-inexistant={x}` neuf passait — alors que l'AC6, elle, lit les directives (M17).
- **Retenu** : séparateurs `` [\s"'`{}()=] `` ; auto-tests sur source synthétique (directive → nom relevé ;
  `context-menu` → rien ; préfixes, `!` et opacité retirés) ; mutation M22 (le `=` retiré). Le `=` ajouté ne fait
  naître aucun jeton neuf hors directives (relevé au 2026-10-13). Un jeton suivi d'une ponctuation de prose
  (`text-foo.`) reste écarté : angle mort 12.
- **Écartée** : traiter les `ClassDirective` par leur `name` au moyen de `svelte/compiler`, comme l'AC6 (la garde
  porte aussi sur les `.ts`, qu'un AST Svelte ne couvre pas ; deux chemins de relevé au lieu d'un).
- **Réversible** : oui.

### C-28-1-35 — `@tailwindcss/node` au même spécificateur que `tailwindcss`, sous assertion de version

- **Contexte** (F6, LOW) : épinglé exact à 4.2.2 à côté de `tailwindcss` en `^4.2.2`, il divergerait au premier
  `tailwindcss` 4.3 — `@tailwindcss/node` dépend de `tailwindcss` exactement à sa propre version et en
  embarquerait une copie : la garde déciderait avec un autre moteur que celui du build, sans signal.
- **Retenu** : `^4.2.2`, comme `tailwindcss` et `@tailwindcss/vite` ; en tête de la garde, versions de
  `@tailwindcss/node` et de `tailwindcss` égales (`readFileSync` des deux `package.json`) et absence de
  `node_modules/@tailwindcss/node/node_modules/tailwindcss`, avec le message « monter les trois ensemble ».
- **Écartée** : la contrainte écrite dans le seul doc-comment (une consigne qu'aucun test ne contrôle).
- **Réversible** : oui.

### C-28-1-36 — Décomptes et références rectifiés (validation P3)

- **R1** (MEDIUM) : `accounts:717` est la confirmation du changement de type (*Changer le type quand même*), à côté
  d'un *Annuler* `outline` (`:712`), non un archivage : **17** confirmations parmi les 24 usages, **13** à côté
  d'un *Annuler* `outline`, 4 `ghost` ; 13 dans un `Dialog.Root`, 4 en ligne (`dunning:300`, `vat-rates:366`,
  `api-keys:404`, `bank-accounts:586`). L'AC8 dit « 13 des 24 ».
- **R4** : **six** `<button>` bruts en `text-destructive` (trois bordés de la classe C, *Annuler le rapprochement*,
  deux ✕), non quatre ; `JournalEntryForm:465` ne rougit qu'au survol.
- **R5** : le CHANGELOG nomme aussi *Annuler le lot*, *Annuler le règlement*, *Annuler la facture* et les croix de
  retrait de ligne, qui deviennent rouges.
- **R6** : C-28-1-17 porte sa rectification. **R7** : le contrôle `variant="outline"` de la T9 est retiré (muet :
  le fichier ne porte aucun `variant=`). **R8** : la fiche cite **#648** ; un seul des cinq sites dépend du jeton
  (la rectification de #648 et du commentaire de #640 revient à l'orchestrateur). **R9** : la ligne du doublon est
  `BankImportUpload.svelte:344`. **R10** : septième `ghost` à icône destructif, `fiscal-years:417` (*Clôturer*).
  **F4** : les titres de tests portent le schéma.
- **Rectification (validation P4 ciblée, 2026-10-13)** : deux valeurs de cette entrée étaient fausses à leur date.
  **R1** : 12 confirmations dans un `Dialog.Root` et **5** hors, non 13 et 4 — `journal-entries/[id]:557` est dans
  un overlay `role="dialog"` maison (`:526-530`), non dans un `Dialog.Root` (P4-1). **R9** : la ligne du doublon est
  `BankImportUpload.svelte:345` ; `:344` est celle de `balanceMismatch` (P4-2).
- **Réversible** : —.

### C-28-1-37 — `@tailwindcss/node` installé à `4.2.2`, sur un `node_modules` copié (développement)

- **Contexte** : `npm install --save-dev '@tailwindcss/node@^4.2.2'` a résolu la dernière version admise, **4.3.3**,
  et écrit `^4.3.3` dans `package.json` — exactement la divergence de moteur que C-28-1-35 interdit
  (`tailwindcss` installé : 4.2.2). Le `node_modules` du worktree, lui, était lié en dur (`cp -al`) au dépôt
  principal : un `npm install` y aurait réécrit des fichiers partagés.
- **Retenu** : `node_modules` du worktree remplacé par une **copie** indépendante (`cp -a`), puis
  `npm install --save-dev '@tailwindcss/node@4.2.2'`, qui écrit `"^4.2.2"` (préfixe par défaut de npm) — le
  spécificateur de `tailwindcss`. Lockfile : l'entrée racine gagne la ligne, et l'entrée `node_modules/@tailwindcss/node`
  perd `"license": "MIT"` (normalisation de npm 9.2) ; rien d'autre (`git diff --stat` : 1 insertion, 1 suppression).
- **Écartée** : garder `^4.3.3` (moteur de la garde ≠ moteur du build ; l'assertion de version aurait rougi) ;
  monter les trois paquets en 4.3.3 (hors périmètre de #640).
- **Réversible** : oui.

### C-28-1-38 — Page `/design-system` : l'exemple désactivé est un 25e usage de la variante

- **Contexte** : l'AC9 ajoute `<Button variant="destructive" disabled data-testid="ds-destructive-disabled">` et
  un carré de palette au-dessus de la section *Boutons* : la fiche compte « 24 usages » au 2026-10-13, et cite
  `design-system:69` pour l'exemple existant.
- **Retenu** : 25 usages au `HEAD` de la story (24 + l'exemple désactivé), toujours dans 17 fichiers (`design-system` en
  porte désormais trois) ; l'exemple existant passe de `:69` à `:70` (carré de palette inséré
  l. 44). La garde (f) ne compte pas les usages : elle exige seulement qu'aucun ne porte de `class` ; son assertion
  de montage exige « au moins 24 ».
- **Réversible** : —.

### C-28-1-39 — Garde AC6 : deux clés lues au sol, table écrite depuis la fiche

- **Contexte** : la table fermée doit être écrite depuis l'inventaire de la fiche, non recopiée de la sortie de la
  garde (défaut de l'AC7 de l'Epic 23). La fiche nomme les `data-testid` d'`api-keys` et des classes B, D, E en
  toutes lettres, mais ceux de `bank-accounts:477` et `RulesList:175` seulement par leur forme (`archive-button-{id}`,
  `delete-button-{id}`).
- **Retenu** : table écrite depuis la fiche (14 entrées, jetons des classes B à E) ; la première exécution n'a
  rougi que sur ces deux clés, dont le texte source a été lu au sol (`grep -nF`) : `archive-button-{ba.id}`
  (`bank-accounts/+page.svelte:479`), `delete-button-{r.id}` (`RulesList.svelte:178`). Aucun autre écart.
- **Réversible** : —.

### C-28-1-40 — Garde AC7 (4) : l'API de Tailwind se charge, pas de repli

- **Contexte** : la fiche prévoit un repli si `__unstable__loadDesignSystem` ne se charge pas sous Vitest.
- **Retenu** : chargé sous `// @vitest-environment node` par `await import('@tailwindcss/node')` dans un
  `beforeAll` (60 s) ; le repli n'est pas écrit (code mort). Premier relevé, avant les jetons : **quatorze** noms non
  résolus, les treize de la table **et** `destructive` (garde rouge sur ce seul nom) ; après la T1, les treize.
  Aucun faux positif de prose ou d'attribut SVG au relevé.
- **Réversible** : oui (le repli s'écrirait si une version future cassait l'API instable).

### C-28-1-41 — Montage de la garde (f) : `>= 25` relevé par l'analyseur, non l'égalité

- **Contexte** : revue de code P1, E3 = A-L3. L'assertion de montage de (f) exigeait « au moins 24 » quand le `HEAD`
  en compte 25 (C-28-1-38), et comptait par expression régulière, non par l'analyseur qu'emploie (f).
- **Retenu** : `boutonsDestructifs` (analyseur `svelte/compiler`) partagé par (f) et par le montage ; le montage
  exige `parAnalyseur === parTexte` (un analyseur aveuglé rendrait moins que le texte) et `parAnalyseur >= 25`.
  Plus trois auto-tests synthétiques de `destructifsAvecClasse` (attribut, directive, cas négatifs).
- **Écartée** : l'égalité exacte à 25. Un usage **ajouté** est déjà tenu par (f) ; l'égalité ferait rougir chaque
  story qui ajoute un bouton de suppression, sans rien protéger de plus, et l'habitude de relever le chiffre sans
  regarder viderait l'assertion de son sens. Le `>=` voit un usage **retiré** tant que personne n'en ajoute ;
  au-delà, l'angle mort 7 reste écrit.
  *[Annotation de la revue de code P2, P2-3 : « déjà tenu par (f) » était inexact — (f) ne contrôle qu'une `class`
  ou une directive `class:` ; un usage ajouté sans elles n'est simplement pas fautif. Le choix reste le même ;
  le commentaire du test est corrigé.]*
- **Réversible** : oui (une ligne).

### C-28-1-42 — Lockfile : la ligne `"license": "MIT"` rétablie à la main

- **Contexte** : revue de code P1, B3 = E5. `npm install` (npm 9.2) avait retiré `"license": "MIT"` de l'entrée
  `node_modules/@tailwindcss/node` (C-28-1-37) — bruit de normalisation, sans effet.
- **Retenu** : ligne rétablie par l'outil Edit, sans résolution npm (aucun `npm install`) : la valeur est celle
  du `package.json` publié du paquet, et le diff du lockfile contre `bbd0a9a1` se réduit à l'unique insertion
  `"@tailwindcss/node": "^4.2.2"` de l'entrée racine. JSON relu (`JSON.parse`), `npm run build` et Vitest
  repassés au gate.
- **Écartée** : régénérer avec une autre version de npm (résolution, risque d'autres écarts) ; laisser la ligne.
- **Réversible** : oui. Un futur `npm install` sous npm 9.2 pourra la retirer de nouveau.

### C-28-1-43 — Relevé AC7 (4) : `!` final et préfixes `shadow-`, `from-`, `via-`, `to-` ajoutés

- **Contexte** : revue de code P1, E4.
- **Retenu** : le `!` final de Tailwind v4 est retiré comme l'initial ; `MOTIF` reçoit `shadow`, `from`, `via`,
  `to`, et les préfixes composés `ring-offset`, `inset-ring`, `inset-shadow` (avant leur racine, pour que le nom
  ne commence pas par `offset-`) ; `nomDe` lit le nom dans le groupe du motif. Deux auto-tests synthétiques.
  Mesuré : le relevé élargi ne fait naître **aucun** nom non résolu neuf (garde verte, table inchangée) — les
  candidats neufs du dépôt sont `shadow-sm`, `shadow-md`, `shadow-lg`, que Tailwind résout.
- **Écrits en angles morts** (en-tête de la garde) plutôt que traités : les `.js` de `src/` et `app.html` (aucune
  classe de couleur au 2026-10-13 ; `app.html` ne porte que `<meta name="text-scale">`) ; la table tient des noms,
  si bien qu'un `bg-danger` neuf passe (voulu, #645 traite le nom).
- **Réversible** : oui.

### C-28-1-44 — E2E (b) : attente des animations et nœud axe résolu par son `target`

- **Contexte** : revue de code P1, E1. axe ne vérifiait que `violations == []` ; un nœud `incomplete` passait à
  vide, et l'analyse pouvait courir pendant le fondu d'ouverture du dialogue.
- **Retenu** : avant toute mesure du dialogue, `expect.poll` sur `document.getAnimations()` en cours, animations
  infinies exclues (un indicateur de chargement ne finit jamais) ; puis *Supprimer définitivement* doit figurer
  dans les `passes` de `color-contrast` et non dans ses `incomplete`. Le nœud se reconnaît en résolvant dans la
  page le sélecteur `target[0]` donné par axe : le premier essai, sur `node.html`, rougissait à tort — axe tronque
  le `html` d'un élément long, et la classe du bouton fait passer le `data-testid` au-delà de la coupe.
- **Écartée** : `getAnimations()` du seul dialogue (le voile, `Dialog.Overlay`, est un frère et anime aussi) ;
  une seconde analyse bornée au bouton (deux analyses pour une question).

### C-28-5-36 — L'utilisateur Consultation de l'E2E est désactivé, non supprimé (28-5b, dev)

- **Contexte** : AC11 prescrit « import et utilisateur supprimés en `finally` ». Kesh n'a aucune route de
  suppression d'utilisateur (`lib.rs` : `GET`/`PUT /api/v1/users/{id}`, `PUT …/disable`, `PUT …/reset-password`).
- **Retenu** : l'import est supprimé, l'utilisateur **désactivé** (`PUT /api/v1/users/{id}/disable`) en `finally`,
  par le contexte d'API de l'administrateur ouvert avant le changement de session ; nom unique à l'exécution.
- **Écartée** : ajouter une route de suppression d'utilisateur — hors périmètre ; ne rien faire — l'utilisateur
  resterait actif dans la base E2E.
- **Réversible** : oui.

### C-28-5-37 — `UnrejectTransactionAction` porte lui-même la condition `isRejected` (28-5b, dev)

- **Contexte** : AC5 dit que le composant rend la mention et le bouton « pour toute transaction rejetée », sans
  fixer qui teste la condition.
- **Retenu** : le composant ne rend **rien** hors d'une rejetée (patron de `DeleteBankImportAction`, qui ne rend rien
  sans `canManage`) ; la page le monte sur chaque ligne, dans la colonne *Rapprochement*. V2 couvre « non rejetée ⇒
  rien », V3 le montage par la page.
- **Écartée** : `{#if isRejected(tx)}` dans la page — la condition et son rendu vivraient dans deux fichiers.
- **Réversible** : oui.

### C-28-5-38 — Captures sans auxiliaire partagé, un test par schéma pour le scénario 1 (28-5b, dev)

- **Contexte** : AC11 demande de reprendre l'auxiliaire de captures de la 28-1 (C-28-1-29) « s'il est sur `main` ».
  Il n'y est pas : `grep -rn KESH_E2E_CAPTURES_DIR frontend/tests` ne rend que `error-messages.spec.ts` (28-2), qui
  écrit ses captures en ligne.
- **Retenu** : un auxiliaire local `capturePath(testInfo, nom)` — `KESH_E2E_CAPTURES_DIR` s'il est posé, sinon
  `testInfo.outputPath()` ; le scénario 1 tourne **une fois par schéma** (`page.emulateMedia` en tête de test),
  avec son propre relevé unique, et prend ses trois captures — six au total.
- **Écartée** : basculer le schéma en cours de test (patron de la 28-2) — la capture « après annulation » exige un
  second rejet, donc un second parcours.
- **Réversible** : oui ; l'auxiliaire de la 28-1, s'il arrive sur `main`, le remplacera.

### C-28-5-39 — E2E : la limite des 100 propositions, risque hérité du patron (28-5b, dev)

- **Contexte** : le scénario 1 vérifie que la transaction revient dans la liste de la réconciliation, qui ne montre
  que les 100 plus récentes en attente (#649) ; la fixture CAMT date ses transactions de mai 2026.
- **Retenu** : patron de `reconciliation-deleted-import.spec.ts`, qui fait la même hypothèse (les specs nettoient
  leurs imports en `finally`) ; l'échec, s'il survenait, se lirait sur l'assertion `reconciliation-row`.
- **Écartée** : réécrire la date de réservation — elle touche la période du relevé et l'avertissement de solde.
- **Réversible** : oui.

### C-28-5-40 — CHANGELOG : la phrase prescrite, suivie de son mode d'emploi (28-5b, dev)

- **Contexte** : AC9 (b) prescrit les deux entrées *Ajouté* et *Corrigé* ; le CHANGELOG s'adresse aux utilisateurs.
- **Retenu** : chaque entrée commence par la phrase prescrite, mot pour mot, et la suit d'une ou deux phrases — où se
  trouve le bouton, qui le voit, la trace d'audit, la limite des 100 (#649) pour *Ajouté* ; ce qui était cassé pour
  *Corrigé*.
- **Réversible** : oui.
---

## Story 28-6 — Le rapprochement refuse de régler une facture client par un débit ([#482])

Choix faits à la fiche (2026-10-13), en autonomie, sur `5aad8b35`. Fiche :
`28-6-rapprochement-debit-facture.md`.

### C-28-6-1 — Prémisse de l'issue corrigée : la base refuse l'écriture, le défaut est le refus présenté comme une panne

- **Contexte** : #482 écrit que le rapprochement « écrirait un règlement de montant négatif ». À la lecture,
  `settlement_journal_lines` pose `debit = montant` (négatif), que `chk_jel_debit_credit_exclusive` /
  `chk_jel_debit_nonneg` refusent (4025 → `CheckConstraintViolation` → bras `_` de
  `project_error_to_failed_proposal` → `DATABASE_ERROR`), point de sauvegarde annulé.
- **Retenu** : la fiche le dit comme **hypothèse** et exige de la constater au rouge (T1), code observé consigné ;
  si quoi que ce soit était écrit, le défaut serait un P1, à signaler. Priorité P2 maintenue : comportement faux
  (refus annoncé comme panne, rejeu encouragé), la seule garde étant une contrainte de base.
- **Écartées** : croire l'issue (P1 implicite) ; ne rien corriger au motif que rien n'est écrit (une contrainte de
  base n'est pas une règle métier, et le message est faux).
- **Réversible** : oui. **Demandé à l'orchestrateur** : un commentaire sur #482 après T1, avec le code observé.

### C-28-6-2 — Code dédié `RECONCILIATION_TRANSACTION_NOT_CREDIT`, un seul pour le débit et le nul

- **Contexte** : l'issue demande « un code dédié ». Les autres variantes (`split`, `rule`, `manual`, `split`
  standalone) refusent déjà le montant nul en `VALIDATION_ERROR` + `zero_amount_transaction`.
- **Retenu** : un code neuf, `details.reason` = `debit_transaction` | `zero_amount_transaction`, plus
  `transactionAmount` (forme de `RECONCILIATION_OVERPAYMENT`). Un libellé lisible, un seul code à traiter pour une
  intégration.
- **Écartées** : `VALIDATION_ERROR` (libellé « Erreur de validation », muet) ; deux codes distincts (deux libellés
  pour une seule règle) ; `RECONCILIATION_SCORE_TOO_LOW` (faux : le score n'y est pour rien).
- **Réversible** : non une fois publié (code de contrat) — d'où le choix avant la v0.13.3.

### C-28-6-3 — La garde dans `accept_one_invoice`, après le statut et avant la facture ; pas au pré-vol

- **Retenu** : sur la transaction **rechargée sous le verrou du compte** (la relecture n'a pas de `FOR UPDATE` ;
  le verrou est celui du compte bancaire, pris par `post_accept` — précision de la validation P1, F1), entre
  l'étape 4 et l'étape 5. Les refus du pré-vol (`BANK_TRANSACTION_NOT_FOUND`, `RECONCILIATION_ALREADY_RECONCILED`,
  `RECONCILIATION_ALREADY_REJECTED`) la précèdent (R6). Le refus ne dépend que
  de la transaction : il précède la lecture de la facture (`INVOICE_NOT_FOUND`), l'éligibilité et le score.
- **Amendé en validation P1 (F1, R6)** : la version d'origine disait « sous verrou » sans le nommer, et ne
  plaçait pas les refus du pré-vol dans l'ordre ; les deux précisions ci-dessus sont celles de la remédiation P1
  (marqueur ajouté à l'étape 0 du développement, finding P2-2 de la validation P2).
- **Écartées** : au pré-vol de `post_accept` (dupliquerait la règle, et le pré-vol est commun aux trois variantes,
  dont deux admettent un débit) ; après le score (laisserait `SCORE_TOO_LOW` répondre pour un débit sans référence).
- **Réversible** : oui.

### C-28-6-4 — Garde écrite `amount <= Decimal::ZERO`, comparaison et non test de signe

- **Retenu** : même forme que `get_proposals` ; la règle « un règlement de facture exige un encaissement » écrite
  deux fois, proposition et acceptation, les deux doc-comments se nommant. Le zéro négatif est un nul.
- **Écartée** : une fonction partagée par `get_proposals` et `accept_one_invoice` (le filtre de `get_proposals`
  est une ligne dans une boucle de lecture ; l'extraire toucherait une région que la 28-5 lit aussi, sans gain).
- **Réversible** : oui.

### C-28-6-5 — Aucune garde de sens sur `split`, `rule`, `manual` et l'éclatement standalone

- **Contexte** : l'epic demande de « vérifier les autres `accept_one_*` pour la même absence de garde ».
- **Retenu** : vérifié, **pas la même absence** : ils n'écrivent aucun règlement de facture ; un débit y est
  légitime (dépense, remboursement), ce que `rules.rs:8` et `validate_split_balance` (`abs()`) disent déjà ; le nul
  y est déjà refusé (`:2447`, `:2870`, `:3659`, `:4109`).
- *Annotation du 2026-10-13 (revue de code P1, A3)* : ces numéros, relevés au `5aad8b35`, visaient déjà des lignes
  voisines et ont bougé depuis. Les sites du refus du nul sont, au `289a7fed` : `accept_one_split` `:2518`,
  `accept_one_rule` `:2941` (`VALIDATION_ERROR`, `details.reason = "zero_amount_transaction"`), `post_manual` `:3727`,
  `post_split` `:4177` (`400 VALIDATION_ERROR`) — à retrouver par `grep -n zero_amount_transaction`, le nom de la
  fonction faisant foi.
- **Réversible** : oui.

### C-28-6-6 — Libellé frontend et clé i18n, bien que le refus ne soit pas atteignable depuis l'écran

- **Contexte** : l'écran ne soumet que la candidate proposée, et `get_proposals` ne propose jamais de facture pour
  un débit. Le code n'apparaîtra donc, en pratique, qu'à une intégration.
- **Retenu** : un `case` et une clé dans les quatre locales — `failed-proposal-label.test.ts` déclare couvrir tous
  les codes de `failed[]` (patron de la 15-5c) ; un code sans libellé s'afficherait « Refus non reconnu (CODE) » le
  jour où l'écran permettrait de choisir une autre candidate. Coût : un `case`, quatre lignes, deux compteurs.
- **Écartée** : rien côté frontend (périmètre plus étroit, mais l'invariant « tous les codes » deviendrait faux).
- **Réversible** : oui.

### C-28-6-7 — Pas de capture d'écran

- **Retenu** : la story ne change ce que l'écran affiche dans aucun parcours atteignable (cf. C-28-6-6) ; la règle
  de recette visuelle de l'epic vise la 28-6 « si elle change ce que l'écran affiche ».
- **Réversible** : oui.

### C-28-6-8 — Manuels vérifiés, non modifiés

- **Retenu** : `user-manual.tex:1919` (« créditrice, en CHF ») reste vrai ; aucun texte ne promet l'acceptation d'un
  débit ; le manuel utilisateur n'est pas modifié. Le contrat d'API, lui, est modifié (AC7). Contrôle du PDF aplati
  par les passes de revue.
- **Amendé en validation P1 (R1, MEDIUM)** : la version d'origine disait que « le manuel d'administration renvoie à
  `docs/api-external.md` pour les codes » — faux : `admin-manual.tex:1992-1994` (paragraphe « Intégrations par
  l'API : les lots de réconciliation », 0.13.2) **nomme** trois codes de `failed[]` et annonce un changement de
  contrat d'`accept` aux intégrations par clé d'API, avant de renvoyer à `docs/api-external.md`. Ce paragraphe
  reste vrai après la story ; la ligne que la 0.13.3 lui doit est l'objet de C-28-6-11.
- **Écartée** : une phrase au manuel utilisateur sur un refus que l'écran ne peut pas produire.
- **Réversible** : oui.

### C-28-6-9 — Le symétrique (crédit ↔ facture fournisseur) n'existe pas ; les avoirs non plus

- **Retenu** : aucun chemin ne lie une transaction bancaire à une facture fournisseur ni à un avoir (inventaire de la
  fiche) : rien à refuser, l'absence est écrite. Le remboursement d'un client par un débit passe par l'affectation
  manuelle, sans lien de facture : catégorie C, pas le défaut de #482.
- **Réversible** : oui (à reprendre si un rapprochement fournisseur est un jour ajouté).

### C-28-6-10 — Périmètre minimal, exclusions écrites

- **Retenu** : une règle à un site, son libellé, son contrat, son CHANGELOG ; quatre modules de code. Exclus par
  écrit : pré-vol, autres variantes, harmonisation des codes du nul, `get_proposals`, score, `classify_payment`,
  manuel utilisateur, captures, bras `_` de `project_error_to_failed_proposal`. *(Validation P1 : « manuels »
  devient « manuel utilisateur » — le manuel d'administration reçoit une ligne, C-28-6-11.)*
- **Réversible** : oui.

### C-28-6-11 — Le manuel d'administration nomme le code neuf, dans la section de la 0.13.3 (validation P1, R1)

- **Contexte** : la 0.13.2 a dit aux intégrations par clé d'API, au manuel d'administration, le changement de
  contrat d'`accept` (codes nommés, renvoi à `docs/api-external.md`). Ce paragraphe a été écrit à la **préparation
  de release** (`d415e66c`), dans une sous-section « Passer à la 0.13.2 » ouverte par une **story** (27-2,
  `e04122dc`). La 28-6 change à son tour ce contrat (AC7). Au `5aad8b35`, aucune sous-section « Passer à la
  0.13.3 » n'existe.
- **Retenu** : à T5, si la sous-section de la 0.13.3 existe au rebase, la 28-6 y écrit une ligne — paragraphe
  d'intégrations par l'API, sur le modèle de la 0.13.2 — qui nomme `RECONCILIATION_TRANSACTION_NOT_CREDIT` pour les
  intégrations par clé d'API, et régénère le PDF (`make fr`, sous `flock`). Sinon, elle ne crée pas la
  sous-section, et **la préparation de la release v0.13.3 écrit cette ligne** en ouvrant la sous-section : le
  présent choix est sa consigne, la puce du CHANGELOG (AC8) son entrée. Le Dev Agent Record dit lequel des deux
  cas s'est produit.
- **Écartées** : ne rien écrire au manuel d'administration (le précédent de la 0.13.2 laisserait croire aux
  intégrations que la 0.13.3 ne change pas le contrat d'`accept`) ; ouvrir la sous-section dans la 28-6 (son
  intitulé et ses consignes de mise à jour — migrations, retour, scripts — relèvent de la version entière, que la
  préparation de release connaît, non une story isolée).
- **Réversible** : oui.

### C-28-6-12 — `-0.00` : cas de la seule fonction pure, précondition assertée, mutation M8 (validation P1, F3 = R5)

- **Contexte** : `bank_transactions.amount` est `DECIMAL(18,2)` : un zéro négatif lu d'un CSV s'écrit et se relit
  `0.00`. La fiche rangeait `-0.00` parmi les cas de la garde comme s'il pouvait l'atteindre.
- **Retenu** : `-0.00` reste un cas de l'AC3 (défense en profondeur de la fonction pure), dit inatteignable par la
  base ; le test asserte d'abord `dec!(-0.00).is_sign_negative()` (sans quoi le cas serait un doublon muet de
  0.00) ; mutation M8 — classification par test de signe — tuée par ce cas.
- *Annotation du 2026-10-13 (revue de code P1, A4)* : la précondition a rougi au premier run — `dec!(-0.00)` n'est
  **pas** un zéro négatif, le macro le normalise. Le montage est `Decimal::new(0, 2)` + `set_sign_negative(true)` :
  voir **C-28-6-15**. Le reste de ce choix tient.
- **Écartées** : retirer le cas (le piège « test de signe » ne serait plus fixé par rien) ; le tester de bout en
  bout (impossible : la base ne le rend pas).
- **Réversible** : oui.

### C-28-6-13 — La divergence du montant nul entre `invoice` et les autres variantes est dite au contrat (validation P1, F4)

- **Contexte** : un nul est refusé en `VALIDATION_ERROR` + `details.reason = "zero_amount_transaction"` pour `split`
  et `rule` (et en `400` pour le rapprochement manuel et ventilé), en `RECONCILIATION_TRANSACTION_NOT_CREDIT` avec
  la même `reason` pour une facture (C-28-6-2, C-28-6-5).
- **Retenu** : une demi-phrase dans `docs/api-external.md` (AC7), pour qu'une intégration qui teste
  `details.reason` sous `VALIDATION_ERROR` sache que le cas de la facture a un autre code. Pas d'harmonisation
  (exclusion C-28-6-10 maintenue).
- **Écartée** : harmoniser les codes (changement de contrat de trois variantes, hors #482).
- **Réversible** : oui.

### C-28-6-14 — « Aucune garde sur les autres variantes » tenu par quatre tests existants nommés, sans mutation neuve (validation P1, R3)

- **Retenu** : l'AC4 nomme, par variante, un test existant qui accepte un débit et rougirait si une garde de sens y
  était ajoutée (`accept_with_explicit_split_type_runs_split_flow`,
  `accept_with_rule_creates_journal_entry_and_increments_count`,
  `post_manual_creates_journal_entry_and_marks_transaction_reconciled`,
  `split_creates_journal_entry_with_n_plus_1_lines`) ; la revue de code vérifie qu'ils n'ont pas été modifiés.
- **Écartée** : une mutation par variante (quatre mutations pour prouver une **absence** de code que la story
  n'écrit pas ; les tests nommés suffisent à la tenir).
- **Réversible** : oui.

### C-28-6-15 — Le zéro négatif de l'AC3 se construit par `set_sign_negative`, non par `dec!(-0.00)` (développement)

- **Contexte** : l'AC3 prescrivait `dec!(-0.00)`, avec une précondition `assert!(dec!(-0.00).is_sign_negative())`.
  Au premier run du module `non_credit_settlement_tests`, **la précondition a rougi** : le macro de
  `rust_decimal_macros` normalise `-0.00` en un zéro sans signe. Le cas aurait été un doublon de `0.00`, passant à
  vide — exactement ce que la précondition devait empêcher.
- **Retenu** : `Decimal::new(0, 2)` puis `set_sign_negative(true)` ; la précondition (`is_sign_negative()` et
  `is_zero()`) est gardée telle quelle, et un commentaire dit pourquoi le macro n'est pas employé. La mutation M8
  (test de signe) est jouée sur ce montage.
- **Écartées** : supprimer le cas (la défense en profondeur de C-28-6-12 tomberait) ; `Decimal::from_str("-0.00")`
  (comportement de normalisation non vérifié, même risque que le macro).
- **Réversible** : oui (test seul).

### C-28-6-16 — Remédiation de la revue de code P1 : amorcer les valeurs plutôt que réduire les comptes rendus ; un test de course pour l'ordre statut → sens

- **Contexte** : revue P1 (B, E, A : 0 MEDIUM, 11 LOW). B-LOW-1 = E1 : le compteur de numéros de `Etat286` comparait
  `[]` à `[]` ; E2 : `bankTransactionId == invoiceId` dans trois montages ; E3 : l'ordre « statut avant sens »
  n'était tenu par aucun test ; E4 : AC2-4 ne comparait pas les écritures côté débit.
- **Retenu** : (1) **amorcer** — `montage_286` pose une ligne `journal_entry_number_sequences` à 10 (`INSERT IGNORE`)
  et une facture brouillon de décalage avant la vraie, et asserte `tx_id != inv_id` ; le doc-comment d'`Etat286`
  dit en outre ce que le relevé prouve (un filet contre une écriture partielle qui survivrait, non la garde) —
  mutation **M9** (garde retirée **et** `ROLLBACK TO SAVEPOINT` neutralisé) pour montrer qu'il discrimine ;
  (2) un **test de course** `a_debit_reconciled_while_waiting_for_the_lock_is_refused_for_its_status` (verrou du compte
  tenu, transaction passée `reconciled` pendant l'attente) — mutation **M11** (garde avant l'étape 4) ; (3) AC2-4
  compare les écarts d'écritures, de lignes, de règlements, de traces et de numéros (1, 2, 1, 2 ; 10 → 11) ;
  mutation **M10** (`invoice_id` passé pour `bank_transaction_id`).
- **Écartées** : réduire le commentaire et le Dev Agent Record seuls (le relevé serait resté vide ; amorcer coûte une
  requête) ; une écriture validée réelle pour amorcer le compteur (plus lourd, sans gain : seul le compteur importe) ;
  déclarer « garde avant le statut » équivalente (elle ne l'est pas : un débit rapproché pendant l'attente du verrou
  rendrait le code neuf).
- **Réversible** : oui (tests et commentaires ; aucune ligne de production exécutée ne change).

[#482]: https://github.com/guycorbaz/kesh/issues/482
## Story 28-4 — Clore un compte bancaire, en lecture seule ([#642])

Choix faits à la fiche (2026-10-13), en autonomie, sur `5aad8b35`. Fiche :
`implementation-artifacts/28-4-cloture-compte-bancaire.md`.

### C-28-4-1 — Une fiche, deux stories (28-4a serveur, 28-4b écran) ; dérogation motivée pour la 28-4a

- **Contexte** : plus de cinq modules (`kesh-db`, quatre modules de `kesh-api`, `kesh-i18n`, trois du frontend, les
  manuels) — § *Règle de splitting préventif*. Leçon de l'epic : poser le périmètre minimal dès la fiche.
- **Retenu** : précédent C-26-2-15 — une fiche, deux clés, deux PR successives, clé mère au statut `split`. 28-4a :
  état, gestes, refus serveur sur l'inventaire clos, registre statique, messages serveur (`refs #642`). 28-4b : écran,
  sélecteurs, manuels, CHANGELOG, captures (`closes #642`). Aucune release entre les deux. La 28-4a reste au-dessus de
  cinq modules : dérogation (un seul pattern de garde, mécanique ; l'inventaire clos n'a de sens que complet ; touches à
  `routes/reconciliation.rs` réduites aux pré-vols). Revue du rollout fichier par fichier.
- **Écartées** : une story unique (trop large pour une passe adversariale fiable) ; trois stories (socle / gardes /
  écran) — un état « clos » sur `main` qui ne tiendrait pas sur une partie des chemins, et un registre statique
  impossible à mettre au vert à moitié.
- **Écart à la consigne de l'orchestrateur** : la consigne demandait la clé `28-4-cloture-compte-bancaire:
  ready-for-dev` ; le précédent C-26-2-15 met la clé mère en `split` et les deux clés filles en `ready-for-dev`. Retenu
  le précédent, signalé au rapport.
- **Précision de la validation P1** (R14) : la 28-4a **dépasse elle-même cinq modules** (sept dépôts de `kesh-db`,
  quatre modules de `kesh-api`, `kesh-i18n`). C'est une **dérogation** à la règle de découpage, et non l'exception que
  la règle prévoit (cycles Cargo, merges intermédiaires impossibles à tester) ; elle écarte aussi le « story-zero
  pilote ». Elle se motive par le pattern unique et l'inventaire qui n'a de sens que complet — au Project Lead d'en
  juger.
- **Réversible** : oui (fusionner les deux clés avant le dev).

### C-28-4-2 — Un état distinct de l'archivage, porté par une seule colonne `closed_on DATE NULL`

- **Contexte** : #642 laisse le choix entre étendre `archived` et un état distinct. `archived` signifie aujourd'hui
  « disparu » : `404` anti-énumération partout, exclu des listes. Clos doit rester **lisible**.
- **Retenu** : colonne `closed_on DATE NULL` — `NULL` = ouvert, une date = clos depuis cette date. Une seule colonne
  porte l'état et la date (pas de booléen redondant qui pourrait la contredire). Les deux états se combinent : un compte
  clos sans transaction peut être archivé après réouverture.
- **Écartées** : étendre `archived` (perd la lecture) ; un `ENUM status` (`open`/`closed`/`archived`) qui remplacerait
  `archived` (migration breaking, réécriture de tous les filtres existants) ; deux colonnes `closed_at DATETIME` +
  `closing_date DATE` (horodatage déjà à l'audit).
- **Réversible** : à coût de migration.

### C-28-4-3 — Date de clôture facultative, jour UTC par défaut, sans borne

- **Contexte** : #642 veut une date facultative. Un contrôle (pas dans le futur, pas avant la dernière transaction)
  bute sur le fuseau (le serveur raisonne en UTC, l'utilisateur en heure suisse) et ne protège rien : c'est l'état qui
  garde.
- **Retenu** : `closedOn` optionnel à l'API, défaut `Utc::now().date_naive()` (précédent `opening_balances.rs`) ;
  l'écran l'envoie toujours, prérempli à la date locale. Aucune borne serveur ; angle mort écrit.
- **Écartées** : date obligatoire ; bornes serveur.
- **Réversible** : oui.

### C-28-4-4 — Transactions non rapprochées et lots générés : la clôture est refusée

- **Contexte** : #642, « à trancher : refus de clore, ou clôture qui les laisse consultables ».
- **Retenu** : refus `412` avec le décompte (`BANK_ACCOUNT_HAS_PENDING_TRANSACTIONS`,
  `BANK_ACCOUNT_HAS_OPEN_PAYMENT_BATCHES`). Une transaction en attente sur un compte en lecture seule ne pourrait plus
  jamais être rapprochée : clore la laisserait muette. Le refus établit aussi les invariants I1 et I2, sur lesquels
  reposent six écrivains de la réconciliation et l'annulation de lot (pas de garde à y poser).
- **Écartée** : clôture qui les laisse consultables (comptabilité incomplète, sans signal).
- **Réversible** : oui.

### C-28-4-5 — Le compte principal ne se clôt pas, et la base l'interdit

- **Contexte** : le principal est l'IBAN de la QR-facture (`find_primary`). Un principal clos imprimerait un compte
  fermé.
- **Retenu** : `412 BANK_ACCOUNT_CANNOT_CLOSE_PRIMARY` (désigner d'abord un autre principal) ; et contrainte
  `CHECK (closed_on IS NULL OR is_primary = FALSE)`, qui rend l'invariant I3 structurel contre tout écrivain présent ou
  futur (leçon *invariant dans le temps*).
- **Écartées** : clore le principal unique (comme l'archivage, AC#10 de la v014-1) — la QR-facture porterait un compte
  clos ; transférer d'office le statut de principal (Kesh choisirait seul).
- **Réversible** : oui (migration pour la contrainte).

### C-28-4-6 — Solde du compte comptable lié non nul : avertir, ne pas refuser

- **Contexte** : #642, « avertir ou refuser ». Le compte comptable peut être partagé entre plusieurs comptes bancaires,
  ou absent ; un frais de clôture peut arriver après.
- **Retenu** : l'écran avertit (solde `currentBalance` déjà dans la liste) ; le serveur ne refuse pas.
- **Écartée** : refus serveur (bloquerait un compte comptable partagé, et une clôture légitime avant l'écriture de
  frais).
- **Réversible** : oui.

### C-28-4-7 — Le compte comptable d'un compte clos reste « lié à une banque »

- **Contexte** : `first_active_bank_account_linked_to` refuse qu'un compte comptable lié à une banque soit désigné
  compte de créance ; il filtre `archived`, pas encore la clôture.
- **Retenu** : il **garde** les comptes clos (le compte a servi de banque ; son historique est bancaire) ; les réglages
  de facturation passent `includeClosed` à vrai pour rester alignés (second paramètre booléen, C-28-4-20).
- **Écartée** : libérer le compte comptable à la clôture (un 1020 deviendrait désignable comme débiteurs avec son
  historique bancaire).
- **Réversible** : oui.

### C-28-4-8 — Clore et rouvrir sont ouverts aux clés d'API `read-write`

- **Contexte** : la 27-1 a fermé la suppression d'un import aux clés (destructif). L'archivage est ouvert.
- **Retenu** : ouverts, comme l'archivage — gestes réversibles et tracés ; une clé `read` est refusée par le
  middleware. Les refus d'écriture sur un compte clos s'appliquent aux clés comme à l'interface.
- **Écartée** : fermer aux clés (rien de destructif à protéger).
- **Réversible** : oui.

### C-28-4-9 — `409 BANK_ACCOUNT_CLOSED` pour une écriture refusée ; `412` pour une clôture impossible

- **Contexte** : codes HTTP du dépôt — `412` pour une précondition de l'archivage, `409` pour un conflit d'état.
- **Retenu** : écrire sur un compte clos est un conflit avec son état → `409` ; ne pas pouvoir clore est une
  précondition non remplie → `412`, comme l'archivage. Dans les lots de réconciliation, `failed[]` par proposition
  (§ *Pattern batch*, précédent 8-5b). Le refus se place après le `409` de version des gestes existants.
- **Réversible** : oui.

### C-28-4-10 — Rien ne s'écrit sur un compte clos, archivage compris, sauf la réouverture

- **Contexte** : « lecture seule » admet des exceptions tentantes (corriger la description, archiver, annuler un
  règlement ancien).
- **Retenu** : une seule règle, décidable : toute écriture qui vise le compte ou une ligne qui lui est rattachée est
  refusée ; la réouverture (tracée) est le seul geste. Corriger le passé = rouvrir, corriger, reclore — trois traces.
- **Écartée** : exceptions par geste (chacune à justifier, chacune une garde à manquer).
- **Réversible** : oui.

### C-28-4-11 — Pas de bump `min_required` ; le retour arrière se dit sans affirmer son mécanisme

- **Contexte** : `ADD COLUMN` nullable et `ADD CONSTRAINT CHECK` sont non breaking (P1/P3). Constat statique : sqlx
  0.8.6, `ignore_missing: false`, jamais activé dans le dépôt — une 0.13.2 refuserait de démarrer (`VersionMissing`).
  Prémisse A-C8 de l'Epic 15, non exécutée ici.
- **Retenu** : aucun bump ; le manuel d'administration prescrit la restauration de la sauvegarde d'avant mise à jour,
  sans affirmer le mécanisme. A-C8 reste à verser au `CLAUDE.md` après vérification à l'exécution (hors périmètre).
- **Réversible** : oui.

### C-28-4-12 — L'archivé est refusé aux gestes qui créent, jamais aux annulations *(réécrit en validation P1)*

> *Annotation de la validation P2 (2026-10-13) : les fonctions de garde sont renommées `ensure_open_in_tx` et
> `ensure_open_unarchived_in_tx`, et ne verrouillent plus ; W17 garde son étape (0) au sol, sans relecture (C-28-4-21).
> Le texte ci-dessous est celui de la P1.*

- **Contexte** : à l'inventaire, `invoice_settlements_write.rs:131-135` et `supplier_invoices.rs:753-756` lisent le
  compte bancaire sans contrôler `archived` ; `create_batch` et l'import ne le contrôlent qu'en lecture non
  verrouillante — défaut voisin, préexistant. Décision de l'orchestrateur à la validation P1 (F3 = R12) : l'intégrer.
- **Retenu** : refus de l'archivé **aux seuls gestes qui créent** — W8 et W27 (instruction `FOR UPDATE` existante
  étendue), W17-W19 et W22-W23 (`lock_open_unarchived_for_write`) —, mappé vers l'erreur **existante** de chaque
  chemin (`DbError::NotFound` ; `IllegalStateTransition` pour le lot ; `404 BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` à
  l'import), AC distinct (AC6-bis), `### Corrigé` du CHANGELOG et note d'`api-external.md` dans la PR de la 28-4a.
  ⛔ Les annulations (W9, W24, W25, W28, W29, W30) **ne lisent pas** `archived` : `lock_open_for_write` ne lit que
  `closed_on`. Raison : un compte archivé porte des règlements par virement et des lots (l'archivage ne compte que
  les transactions bancaires) et ne se rouvre pas ; refuser l'annulation la rendrait définitivement impossible.
- **Écartées** : le choix initial de la fiche (hors périmètre, issue à ouvrir) — l'orchestrateur l'a écarté ; une
  garde commune lisant `archived` pour tous les sites (impasse des annulations, F3) ; un code d'erreur neuf pour
  l'archivé (trois comportements d'archivé coexisteraient, F3 point 5).
- **Réversible** : oui (chaque site se retire seul).

### C-28-4-13 — Les comptes clos masqués par défaut, avec une bascule, comme les archivés

- **Contexte** : #642 veut des listes « avec filtre ».
- **Retenu** : bascule « Afficher les comptes clos » sur l'écran de gestion, défaut masqué ; le toast de clôture dit
  comment retrouver le compte. Par défaut, `GET /bank-accounts` et `/companies/current` excluent les clos : les
  sélecteurs d'écriture ne les proposent plus sans changement de code.
- **Écartée** : clos toujours affichés sur l'écran de gestion (contraire à « retirer de l'usage » ; incohérent avec les
  archivés).
- **Réversible** : oui.

*Choix C-28-4-14 à C-28-4-20 : remédiation de la validation P1 (2026-10-13, Opus 5.5, autonomie).*

### C-28-4-14 — Ordre des verrous : pièces → compte → exercice ; `accept_one_invoice` sans verrou

> *Annotation de la validation P2 (2026-10-13) : **partiellement remplacé par C-28-4-21**. La garde ne verrouille
> plus : la règle d'ordre « pièces → compte → exercice » ne gouverne plus que W8 et W27, qui verrouillaient déjà le
> compte au sol ; le cycle W9 / W24 n'existe plus. Restent en vigueur : la garde de W8 dans l'instruction de
> `settle_invoice` et non dans `create_in_tx`, `accept_one_invoice` sans garde (I1), R5 et M19 sans objet.*
>
> *Annotation de la validation P3 (2026-10-13) : les **positions** de cette entrée (W9, W25, W29 avant le verrou
> d'exercice ; W24 entre les étapes 1 et 2) sont **remplacées** par la règle de placement de C-28-4-21 (annotation P3) :
> une garde non verrouillante posée avant un verrou y fait naître la vue avant l'attente (R1 = F1, R2 = F2). Le
> « W24 après l'étape 2 » écarté ici l'était pour un cycle qui ne vaut que pour une garde **verrouillante** : il est
> désormais retenu.*

- **Contexte** : F1 (HIGH) — la règle « garde après les verrous métier » plaçait le compte après l'exercice dans W9,
  W25, W29, l'inverse de `settle_invoice` et `pay_in_tx` : cycle. F4 = R2 — W24 « après les étapes 1-2 » formait un
  cycle avec l'import (verrou de trou). Décision de l'orchestrateur : garde juste après le verrou de pièce, avant tout
  verrou d'exercice ; pas de verrou dans `accept_one_invoice`.
- **Retenu** : règle « pièces → `bank_accounts` → exercice / écritures / compteur », position écrite par chemin
  (table du § *Concurrence*). W8 : `closed_on` lu dans l'instruction de `settle_invoice` (l. 131-135), plus dans
  `invoice_settlements::create_in_tx` ; `accept_one_invoice` tenu par I1 et le pré-vol R1 ; l'appariement R5 et la
  mutation M19 deviennent sans objet. W24 entre l'étape 1 et l'étape 2 : sans cycle avec l'import (démontré à la
  fiche) ; cycle résiduel avec W9, dont les deux routes sont rejouées sur `1213`.
- **Écartées** : garder la règle d'origine (cycle) ; W24 après l'étape 2 (cycle avec l'import, non rejoué) ; une
  garde dans `create_in_tx` (après l'exercice d'`accept_one_invoice`) ; un verrou neuf lu avant l. 1964 dans
  `accept_one_invoice` (inutile sous I1).
- **Réversible** : oui (positions, à coût de tests).

### C-28-4-15 — W28 (annulation d'une facture fournisseur payée) est gardé

- **Contexte** : F2 = R1 (HIGH) — `supplier_invoices::cancel_in_tx` annule une facture ouverte **ou payée** et remet
  `settlement_bank_account_id` à `NULL` ; le classement « SansCompteBancaire » était faux.
- **Retenu** : gardé comme W29 (si `settlement_bank_account_id` non nul, entre le verrou facture et le verrou de
  l'écriture d'achat), ligne de l'AC6, test T-a27, mutations M39 (position) et M40 (retrait), manuel.
  *Annotation de la validation P3 (2026-10-13) : position remplacée — la garde suit le verrou de l'écriture d'achat et
  de son exercice (C-28-4-21, annotation P3) ; M39 rétablie avec l'objet inverse (garde avancée avant ce verrou),
  tuée par T-a48 (c).*
- **Écartée** : exception assumée (annuler une facture payée sur un compte clos resterait permis) — contraire à
  C-28-4-10.
- **Réversible** : oui.

### C-28-4-16 — Archivage : verrou de ligne et refus « clos » avant le décompte

> *⛔ **Remplacé par C-28-4-21** en validation P2 (2026-10-13). Texte de la P1 conservé ci-dessous.*

- **Contexte** : F6 — la route d'archivage compte les transactions avant le `FOR UPDATE` : un compte clos qui porte des
  imports rendait `412 HAS_TRANSACTIONS`, dont le message propose la clôture.
- **Retenu** : sur le patron de la clôture, `lock_open_for_write` juste après la sentinelle, puis les décomptes ;
  `archive_for_company` relit sous le verrou tenu. Les autres refus gardent leur ordre relatif. Effet voulu : le décompte
  voit tout import qui a pris la garde (L5 fermée pour les imports, avec AC6-bis).
- **Écartée** : refus « clos » lu sur le pré-vol non verrouillant (ne ferme rien sous concurrence).
- **Réversible** : oui.

### C-28-4-17 — L'inversion `companies` → `bank_accounts` par clé étrangère reste un angle mort écrit

> *⛔ **Remplacé par C-28-4-21** en validation P2 (2026-10-13) : la garde ne verrouillant plus, elle n'étend
> l'inversion à aucun chemin ; seule reste l'inversion au sol de W8 / W27. Texte de la P1 conservé ci-dessous.*

- **Contexte** : constat de la remédiation P1 — un écrivain gardé tient le compte en exclusif puis insère des lignes à
  clé étrangère vers `companies` (verrou partagé), alors que les routes de gestion prennent la sentinelle `companies`
  puis le compte. Préexistant pour `settle_invoice` et `pay_in_tx` ; la garde l'étend à W9, W17, W22, W24, W25, W28,
  W29.
- **Retenu** : angle mort écrit (AC7, doc-comment) ; les écrivains rejoués le subissent sans dommage ; `500` rare
  possible sur l'import, la création de lot et les trois routes de gestion à sentinelle. À trancher par l'orchestrateur
  s'il veut une issue.
- **Écartées** : ajouter un rejeu aux routes non rejouées (hors #642) ; prendre la sentinelle dans chaque écrivain
  (sérialiserait toute la société).
- **Réversible** : oui.

### C-28-4-18 — La tuile de l'accueil et les Paramètres ne montrent plus un compte clos

- **Contexte** : R7 — la tuile lit `listBankAccounts(false)` ; un compte clos peut garder un solde comptable non nul.
- **Retenu** : la tuile et la liste des Paramètres excluent les clos, sans code neuf ; le solde se lit sur l'écran
  *Comptes bancaires* (bascule) et au plan comptable ; le manuel le dit (§ *Solde comptable*).
- **Écartée** : garder sur la tuile les clos à solde non nul (logique d'écran neuve pour un cas que la clôture
  signale déjà).
- **Réversible** : oui.

### C-28-4-19 — Messages : le `412` d'archivage garde la voie de la 27-1 ; le refus du principal parle des clos

- **Contexte** : R3 — la réécriture prescrite effaçait la voie « supprimer les imports non rapprochés, puis archiver »,
  qui sert encore ; F9 — un compte clos ne bloque plus l'archivage du principal.
- **Retenu** : le message `has-transactions` garde sa phrase de la 27-1 à l'octet et ajoute la clôture ;
  `cannot-archive-primary` dit « ni archivé ni clos » ; quatre locales, replis Rust, G9.
- **Écartée** : ne proposer que la clôture (fait disparaître une voie valide).
- **Réversible** : oui.

### C-28-4-20 — `listBankAccounts(includeArchived, includeClosed)` : deux booléens

- **Contexte** : R9 — la signature actuelle prend un booléen ; un objet d'options casserait les appelants qui passent
  `false`.
- **Retenu** : second paramètre booléen ; aucun appelant existant ne change.
- **Écartée** : objet d'options (réécriture de tous les appelants).
- **Réversible** : oui.

*Choix C-28-4-21 et C-28-4-22 : remédiation de la validation P2 (2026-10-13, Opus 5.5, autonomie, sur décision de
l'orchestrateur).*

### C-28-4-21 — La garde « compte clos » ne verrouille rien ; aucun verrou ajouté, aucun ordre changé

> *Annotation de la validation P3 (2026-10-13, Opus 5.5, sur décision de l'orchestrateur) — la simplification est
> **maintenue sur le principe** (la garde reste non verrouillante), mais elle avait un défaut de **placement** (R1 = F1,
> HIGH, confirmé par l'orchestrateur sur `reconciliation_cancel.rs:246-250`) : en rendant non verrouillante une garde
> laissée à sa position P1, la P2 a posé une lecture ordinaire **avant** le verrou d'exercice de W9, W25, W28, W29
> (et avant l'étape 2 de W24, R2 = F2). Sous `REPEATABLE READ`, la vue naît alors avant l'attente ; une annulation qui
> attend derrière une clôture d'exercice relirait l'exercice « ouvert » et passerait — une erreur comptable. Le texte
> ci-dessous est celui de la P2 ; il est **complété** ainsi :*
>
> 1. ***Règle de placement*** — *chaque garde se pose après le dernier verrou que le chemin prend avant sa première
>    lecture ordinaire, de sorte que la story ne déplace jamais la naissance de la vue cohérente.* Doc-comments du sol
>    qui l'imposent : `reconciliation_cancel.rs:247-252` et `:296`, `supplier_invoices.rs:1040-1045`,
>    `invoice_settlements_write.rs:858-861`, `bank_imports.rs:389-391` et `:420-425`. Positions : W9 entre (3) et (4) ;
>    W25 entre (2-bis) et (3) ; W28 après `supplier_invoices.rs:1070-1080` ; W29 après (1-bis) ; W24 après l'étape 2 ;
>    W30 après le verrou de sa ligne ; W22 inchangée (vue déjà née dans la route).
> 2. ***Critère à deux volets*** (AC7) — le point 3 ci-dessous gagne : *« aucun déplacement de la naissance de la
>    vue : la première lecture non verrouillante de chaque chemin reste celle du sol »* ; la table *Position par
>    chemin* gagne la colonne qui le prouve.
> 3. ***Tests et mutations*** — M36 à M39, déclarées « sans objet » au point *Conséquences tirées*, sont **rétablies**
>    avec l'objet inverse (garde avancée avant le verrou d'exercice) : M36 tuée par le test existant
>    `reconciliation_cancel.rs:128`, M37 à M39 par T-a48 (entrelacement par virement, neuf) ; M54 (W24 avancée)
>    équivalente sous MariaDB 10.11.
> 4. ***L1 reclassée P2*** (R6 = F8) — le point 4 ci-dessous disait « P3 au plus » : la grille du `CLAUDE.md` donne
>    **P2** (« comportement faux sans effet sur les comptes »). L1 reste une **dette assumée et motivée** (la fermer
>    exige une garde verrouillante, donc les cycles de la P1), et s'écrit aussi à `docs/api-external.md` (les
>    intégrations par clé d'API ne lisent pas le manuel ; R6 b). Ouvrir une issue `P2` qui la trace : à la décision
>    de l'orchestrateur.
> 5. ***Écartée en P3*** — reprendre une garde verrouillante pour fermer le défaut : inutile, le placement suffit, et
>    c'est la voie qui avait ouvert les cycles de la P2.
>
> *Réversibilité inchangée : le placement est une modification locale par site.*

- **Contexte** : la validation P2 rend 1 HIGH et 9 MEDIUM, presque tous nés de la remédiation P1 — le verrou exclusif
  de la garde, combiné aux verrous partagés que les `INSERT` à clé étrangère posent sur `companies`, ouvrait des cycles
  neufs avec les routes de gestion (R1, F5), avec `unreject` de la 28-5 (F1, HIGH), avec `accept_one_invoice` (F2) ;
  l'archivage verrouillé avant son décompte (C-28-4-16) créait un cycle avec W9 et un test irréalisable (R2, F3). Chaque
  correctif P1 ouvrait le défaut suivant : le **recyclage** que désigne l'amendement D5 de la règle de découpage.
  Décision de l'orchestrateur : simplifier la conception plutôt que corriger la sentinelle.
- **Retenu** :
  1. la garde de chaque chemin d'écriture est une **lecture cohérente non verrouillante** de `closed_on` (et
     `archived` pour W22), dans la transaction de l'écrivain — fonctions `bank_accounts::ensure_open_in_tx` et
     `ensure_open_unarchived_in_tx` (renommées : elles ne verrouillent plus), SQL en constantes contrôlées par T-a45 ;
     là où le chemin lit **déjà** la ligne du compte au sol, `closed_on` s'ajoute à cette instruction (verrouillée :
     W8, W27, W13-W15 ; non verrouillée : W17 étape (0), W20 étape (2), pré-vols) — aucune instruction neuve ;
  2. l'archivage garde sa séquence au sol ; il gagne le refus `409 BANK_ACCOUNT_CLOSED` dans son pré-vol (avant le
     décompte : un compte clos qui porte des imports rend `409`, non `412`) et dans le `FOR UPDATE` existant
     d'`archive_for_company` ; plus de sentinelle déplacée, plus de verrou avant le décompte ;
  3. critère décidable de l'AC7 : *la story n'ajoute aucun verrou de ligne, explicite ou implicite par clé étrangère,
     à aucun chemin de `origin/main`, et ne change l'ordre des verrous d'aucun* ; la table *Position par chemin* le
     prouve ligne à ligne (« verrou ajouté : non ; ordre changé : non ») ; les deux chemins neufs (clôture,
     réouverture) ne prennent qu'un verrou, leur propre ligne, en première instruction ;
  4. angle mort **L1** : une écriture dont la vue cohérente précède le commit d'une clôture peut encore aboutir sur le
     compte clos (REPEATABLE READ) — une opération réelle de plus, sans erreur comptable, I1 ou I2 défait pour ce
     compte jusqu'à ce qu'on rouvre, traite et clôt ; classe P3 au plus, acceptable pour #642 ; écrit à l'AC7, au
     doc-comment d'`ensure_open_in_tx` et au manuel **utilisateur** (AC14) ; ni au manuel d'administration (ce n'est
     pas un geste d'installation ou de mise à jour), ni en issue.
  Aucun chemin de l'inventaire ne déroge au critère : le remède `retry_app_on_deadlock` des deux côtés n'a pas lieu de
  servir.
- **Conséquences tirées** : tests T-a32, T-a39, T-a40, T-a41, T-a44 retirés ; mutations M10, M35 à M39, M49, M50 sans
  objet ; M10-bis tuable (l'étape (0) devient la seule garde de W17) ; M51 (équivalente), M52 et M53, T-a45 à T-a47
  ajoutés ; CHANGELOG de l'AC6-bis réduit au règlement vers un archivé (la course archivage / import ou lot reste L5).
- **Écartées** :
  - corriger la conception P1 en enveloppant de `retry_app_on_deadlock` les cinq routes non rejouées et `unreject`
    (option des deux lentilles) : faisable, mais traite les cycles **que la story crée**, au prix de cinq enveloppes
    hors #642 et d'une 28-5 à reprendre ;
  - prendre `companies` en partagé en tête de chaque écrivain : change la doctrine d'ordre de tout le dépôt et ouvre un
    cycle avec les flux qui prennent l'exercice avant la sentinelle de projets (F, option 2) ;
  - garde en `LOCK IN SHARE MODE` : ajoute un verrou, et une montée S → X sur W8 / W27 ;
  - garder un verrou exclusif pour fermer L1 : c'est la conception P1, et son coût est ce qui vient d'être mesuré.
- **Réversible** : oui — revenir à une garde verrouillante est une modification locale des deux fonctions et des
  positions, mais elle rouvre l'analyse de cycles de la P2 ; à ne faire qu'avec une story dédiée.

### C-28-4-22 — Un second inventaire clos : les appelants de `invoice_settlements::create_in_tx`

- **Contexte** : F6 de la P2 — la garde de W8 vit chez un appelant (C-28-4-14) ; le registre de l'AC8, indexé sur la
  fonction qui porte le SQL, ne peut pas dire vrai pour W8 et ne verrait pas un quatrième appelant sans garde.
- **Retenu** : le test de l'AC8 extrait aussi les appels `invoice_settlements::create_in_tx(` hors tests et
  commentaires et exige un statut par appelant (trois sur `5aad8b35` : `settle_invoice` gardé, `accept_one_invoice`
  I1, `write_off_invoice` sans compte bancaire), diff dans les deux sens ; extracteur repris de
  `letterings_lexical.rs` (`appels_et_suite`), dont la liste des trois créateurs reste.
- **Écartées** : remettre la garde dans `create_in_tx` (tomberait après l'exercice d'`accept_one_invoice` — moins grave
  sans verrou, mais `accept_one_invoice` n'en a pas besoin et le refus y deviendrait un code mort) ; l'écrire comme
  angle mort du doc-comment (laisse le mode d'échec ouvert, alors que l'extracteur existe).
- **Réversible** : oui.

*Choix C-28-4-23 : remédiation de la validation P3 (2026-10-13, Opus 5.5, autonomie). La règle de placement et le
reclassement de L1 sont des annotations de C-28-4-21, non des choix neufs.*

### C-28-4-23 — `cancel_batch` (W21) : une sortie d'état autorisée, et non un invariant

- **Contexte** : F8 de la P3 — le registre de l'AC8 classait W21 `Invariant("I2", "un compte clos n'a aucun lot
  generated")`. Sous L1, un lot `generated` peut exister sur un compte clos, et `cancel_batch`, non gardé, est
  précisément le geste qui l'en fait sortir (c'est le remède de L1). La raison écrite était donc fausse.
- **Retenu** : un cinquième statut au registre, `SortieAutorisee("<raison>")` — geste qui ne fait que sortir de l'état
  qu'un invariant interdit sur un compte clos, laissé sans garde à dessein —, porté par W21 seul.
- ⚠️ **Exception à C-28-4-10, dite comme telle** : annuler un lot `generated` écrit sur un compte clos. Elle ne se
  présente que dans une situation que L1 a créée, et ne fait que rétablir I2.
- **Écartées** : garder le statut `Invariant` en réécrivant la raison (le statut dirait encore « hors d'atteinte »,
  ce qui est faux) ; garder W21 (`Gardee`) — cohérent avec C-28-4-10, mais oblige à rouvrir le compte pour annuler
  un lot que L1 a laissé passer ; le remède « rouvrir, traiter, clore » reste de toute façon possible.
- **Réversible** : oui (une variante de statut, un site).

[#642]: https://github.com/guycorbaz/kesh/issues/642
