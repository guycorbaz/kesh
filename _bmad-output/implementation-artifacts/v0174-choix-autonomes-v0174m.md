# Release 0.17.4 — registre des choix autonomes, story v0174-mise-a-jour (#847)

Registre propre à la story `v0174-mise-a-jour` (clé `v0174m`), **versé par union** dans le registre de la release par
l'orchestrateur (`docs/agents-orchestration.md` § 1). Chaque choix : contexte, option retenue, alternatives écartées,
réversibilité. Numéros de ligne de `7bd4d5c0` (le merge de la rétrospective de la série 0.17 sur `main`) sauf mention.

Fiche : `v0174-mise-a-jour.md`. Issue #847 (P2), en `refs` seulement dans les commits.

---

### C-v0174m-1 — Le format de la sauvegarde : un `.keshbackup` de format 2, à l'inventaire du schéma présent

- **Contexte** : le point de conception de #847 — le binaire neuf, avant migration, lit une base au schéma de la version
  de départ. `export_installation` (`crates/kesh-db/src/backup.rs:474`) parcourt `TABLES_TO_TRUNCATE` (`:46`), la liste
  **du binaire neuf** : une table créée par une migration en attente n'existe pas encore, et `export_table` (`:214`)
  échoue alors en `Invariant("table '…' has no non-generated columns")` (`non_generated_columns` rend une liste vide) ;
  à l'inverse, une table que la version neuve supprimerait manquerait à la sauvegarde. Et l'import de la version de
  départ exige un manifeste dont les tables sont **exactement** son propre inventaire (`parse_and_verify`,
  `crates/kesh-api/src/admin_backup/import.rs:234-249`).
- **Retenu** : une lecture dont l'inventaire est **celui du schéma présent** — les tables de
  `information_schema.TABLES` de `DATABASE()` dont le nom ne commence pas par `_` — **même prédicat que la garde**
  `backup_inventory_matches_schema` (aucun filtre sur `TABLE_TYPE`), le `_` filtré en Rust par `starts_with('_')` et non
  par un `LIKE` échappé (amendé à la validation P1, F1-9 ; C-v0174m-24). Elle
  rend exactement le `TABLES_TO_TRUNCATE` de la version qui a créé ce schéma : à chaque version, le test
  `backup_inventory_matches_schema` (`backup.rs:848`) impose « tables du schéma = `TABLES_TO_TRUNCATE` + tables
  système », et toutes les tables système portent le préfixe `_` (`_sqlx_migrations`, `_kesh_version`,
  `_kesh_backup_schedule`, `_kesh_installation_settings`), aucune table applicative ne le porte. Les colonnes sont déjà
  lues dynamiquement (`non_generated_columns`, `:182`) ; l'écriture (`write_keshbackup`) et le manifeste ne changent pas
  de forme. La version de départ importe donc ce fichier comme une sauvegarde qu'elle aurait prise elle-même. Une garde
  neuve fige la convention (toute table système porte le préfixe `_`, aucune table de `TABLES_TO_TRUNCATE` ne le porte).
- **Écartées** :
  - (a) **vidage SQL** (`mariadb-dump`) : l'image d'exécution n'a pas de client MariaDB (`Dockerfile:20`, `ca-certificates
    curl` seulement), la base vit dans un autre conteneur, et un vidage n'emporte pas le dossier des documents ; l'écrire
    en Rust (schéma + données) ferait un second format de sauvegarde à maintenir, sans restauration par l'écran ;
  - (b) le `.keshbackup` **au `TABLES_TO_TRUNCATE` du binaire neuf** : échoue dès qu'une migration en attente crée une
    table applicative, et, quand il réussit, produit un manifeste que la version de départ refuserait si les inventaires
    diffèrent ;
  - (c) les deux à la fois : double la surface de test pour un gain que (a) n'apporte pas ici.
- **Réversible** : oui (aucune donnée, aucun format nouveau ; le fichier est un `.keshbackup` ordinaire).

### C-v0174m-2 — Le moment : après `check_downgrade_protection`, avant `MIGRATOR.run`

- **Contexte** : `main.rs:122-154` refuse un binaire plus ancien que `min_required` ; `main.rs:157` applique les
  migrations ; aucune autre écriture n'a lieu avant (le pool n'a fait que des `SELECT`).
- **Retenu** : la sauvegarde s'insère entre les deux (étape « 3c ») : un binaire refusé par la protection de version ne
  sauvegarde rien ; l'outcome `FreshInstall` saute directement l'étape.
- **Écartées** : avant `check_downgrade_protection` (sauvegarder pour un démarrage qui sera refusé) ; dans la tâche de
  fond après le démarrage (la base serait déjà migrée).
- **Réversible** : oui.

### C-v0174m-3 — Savoir s'il y a des migrations à appliquer : comparer `_sqlx_migrations` au `MIGRATOR`

- **Contexte** : `sqlx` (0.8.6, `Migrator::run_direct`) refuse une base « sale » (`dirty_version`), puis une migration
  appliquée inconnue, puis applique dans l'ordre les migrations connues absentes de `_sqlx_migrations`.
- **Retenu** : un **plan** lu avant `run` : (i) `_sqlx_migrations` absente (erreur 1146 seulement, C-v0174m-16) ou vide → *installation neuve*, pas de
  sauvegarde ; (ii) une ligne `success = 0` → *migration interrompue*, pas de sauvegarde (le journal renvoie à la plus
  récente sauvegarde avant mise à jour : `sqlx` refusera de toute façon, et sauvegarder un schéma à moitié migré
  pourrait évincer, par la rétention, la seule sauvegarde utile) ; (iii) aucune migration connue (montante) absente →
  *à jour*, pas de sauvegarde ; (iv) sinon → *à appliquer*, avec leur nombre et la première version. Une migration
  appliquée inconnue ne change pas le plan : `check_downgrade_protection` ou `sqlx` refuse le démarrage ensuite.
- **Écartées** : comparer `kesh_version_last_applied` à la version du binaire (une version sans migration déclencherait
  une sauvegarde inutile ; une version de développement avec migration n'en déclencherait pas) ; sauvegarder à chaque
  démarrage (coût et disque).
- **Réversible** : oui.

### C-v0174m-4 — Un échec de la sauvegarde refuse le démarrage ; l'échappatoire est une variable qui nomme la version — **réduit par C-v0174m-30 et C-v0174m-31** (causes en vigueur : plan illisible, dossier, espace, lecture, écriture, panique ; ni relecture ni version de départ ; échappatoire évaluée en premier)

- **Contexte** : #847, point 2 ; c'est un réglage de démarrage (l'interface n'est pas encore servie).
- **Retenu** : tout échec (plan illisible, dossier, espace, lecture de la base, écriture, relecture, version de départ
  trop ancienne) → message clair au journal et `exit(1)` **avant** `MIGRATOR.run` : cette tentative n'a rien migré ;
  l'image précédente redémarre dessus si aucune tentative précédente n'a migré (amendé à la validation P1, C-v0174m-15,
  C-v0174m-16). Échappatoire : `KESH_SKIP_PRE_UPGRADE_BACKUP=<version>` — elle ne vaut que si sa valeur est **exactement** la
  version du binaire (`CARGO_PKG_VERSION`) : elle s'éteint d'elle-même à la mise à jour suivante, si on l'oublie dans le
  `.env`. Posée avec une autre valeur, elle est ignorée et le journal le dit ; posée et utilisée, un avertissement est
  écrit. Transmise par les deux compose et décrite à `.env.example` (garde `configuration_transmise`).
- **Écartées** : un booléen (`=true`, oublié dans le `.env`, désactiverait en silence toutes les mises à jour suivantes) ;
  démarrer quand même avec un avertissement (c'est la discipline d'aujourd'hui, celle que #847 remplace) ; un réglage dans
  l'interface (impossible avant le démarrage).
- **Réversible** : oui.

### C-v0174m-5 — Emplacement et nom du fichier

- **Contexte** : #847 demande le dossier des sauvegardes et un nom portant les deux versions ; les sauvegardes du
  serveur sont `auto/kesh-auto-<UTC ms>Z.keshbackup` (`auto_backup.rs:65`, `:581`) et
  `kesh-pre-import-<UTC ms>-<pid>-<n>.keshbackup` à la racine (`routes/admin.rs:699`).
- **Retenu** : `<KESH_ADMIN_BACKUP_DIR>/pre-upgrade/kesh-pre-upgrade-<AAAAMMJJTHHMMSSmmm>Z-<de>-to-<vers>.keshbackup`.
  Horodatage UTC d'abord (le tri par nom est chronologique, comme pour `auto/`), puis la version de départ
  (`_kesh_version.kesh_version_last_applied`) et la version d'arrivée (`CARGO_PKG_VERSION`), chacune réduite aux
  caractères `[0-9A-Za-z.-]` (sinon `inconnue`). Dossier `0700`, fichier `0600`, écrit par `write_backup_file`
  (`routes/admin.rs:754` : `.partial`, `sync_all`, renommage, jamais d'écrasement). Préfixe anglais, comme les deux autres.
- **Écartées** : la racine du dossier (la rétention y côtoierait les sauvegardes pré-import) ; un nom français
  (`avant-0.17.0-depuis-0.16.1-…`, proposé par l'issue : les deux autres noms sont anglais, et l'horodatage en tête est ce
  qui rend le tri sûr).
- **Réversible** : oui (nom et dossier ne sont lus que par la liste du serveur et la rétention).

### C-v0174m-6 — Rétention : les trois plus récentes — **retiré à la validation P2 (C-v0174m-25)**

- **Contexte** : chaque sauvegarde pèse la base plus les documents ; une boucle de redémarrage (`restart: unless-stopped`)
  après un échec de migration non « sale » referait une sauvegarde à chaque tentative.
- **Retenu** : après une sauvegarde **réussie et relue**, Kesh garde les **trois** plus récentes de `pre-upgrade/`
  (fichiers réguliers au nom exact de l'expression), jamais celle qui vient d'être écrite ; un échec ne supprime rien.
  Constante, sans réglage (pas de migration ; une valeur dans `.env` serait une variable de plus pour un cas rare).
- **Écartées** : ne jamais purger (comme les pré-import : le disque du NAS se remplit d'une copie complète par version) ;
  réutiliser le nombre de la sauvegarde automatique (lu dans `_kesh_backup_schedule`, qu'une base antérieure à la 0.15.0
  n'a pas, et réglage d'un autre usage).
- **Réversible** : oui.
- *Amendé à la validation P1* : la plus ancienne sauvegarde du couple courant est en outre protégée (C-v0174m-15).

### C-v0174m-7 — Seuil d'espace : celui de la sauvegarde automatique

- **Contexte** : `seuil_d_espace` (`auto_backup.rs:510`) = `max(dernière, documents) × 1,2 + 256 Mio`.
- **Retenu** : la même fonction, avec pour « dernière » la plus grande sauvegarde présente dans `auto/` et
  `pre-upgrade/`, et l'espace mesuré par `BackupEnv::free_bytes` (injectable, pour les tests). En dessous : refus (C-4).
- **Écartées** : ne rien mesurer (un disque plein se découvrirait en pleine écriture, ce qui reste possible mais rare) ;
  mesurer la base (`information_schema` ne donne qu'une estimation, et la sauvegarde automatique ne le fait pas).
- **Réversible** : oui.

### C-v0174m-8 — Version de départ antérieure à la 0.15.0 ou inconnue : refus, sauf échappatoire — **retiré par la décision du Project Lead (C-v0174m-30)**

- **Contexte** : une sauvegarde de format 2 ne s'importe que dans une 0.15.0 ou plus récente (`manifest.rs:18-27`) . (La lecture des pièces, elle, ne fixe pas la borne : `invoices.pdf_storage_path` existe dès la 0.13.0 — motif
  retiré à la validation P1, R1-11.) Écrire depuis une 0.14.x un fichier
  que la 0.14.x ne peut pas importer serait un filet qui ne retient rien, et le dire seulement au journal le rendrait muet.
- **Retenu** : si `kesh_version_last_applied` ne se lit pas comme une version, ou est inférieure à 0.15.0, Kesh refuse la
  migration avec un message qui dit pourquoi et renvoie au vidage manuel de la base (`kesh-dump.sh` sur Synology, `mariadb-dump` sinon) puis à l'échappatoire.
- **Écartées** : écrire la sauvegarde quand même avec un avertissement ; la sauter en silence.
- **Réversible** : oui. Sans portée pour la production (0.17.3).

### C-v0174m-9 — Le journal de démarrage et l'écran — **réduit par C-v0174m-30** (en vigueur : deux lignes `info`, début et chemin du fichier écrit ; ni `warn` de pièces ni de plafond, pas d'écran ; le corps ci-dessous est l'historique)

- **Contexte** : #847, point 3.
- **Retenu** : quatre lignes `info`, écrites mot pour mot au § *Messages* de la sous-fiche `v0174-mise-a-jour-1.md`
  (plan : nombre de migrations, de → vers ; base lue : tables, lignes, durée ; pièces : emportées, octets, non incluses ;
  fichier : chemin, taille, durée, et le renvoi au manuel « Revenir à la version précédente »), `warn` si des pièces manquent ou si le fichier dépasse `KESH_ADMIN_IMPORT_MAX_MB` (la version de départ
  ne pourrait l'importer par l'écran qu'en relevant ce plafond). À l'écran : la liste *Sauvegardes sur le serveur* montre
  ces fichiers (sous-fiche 2), type « Avant mise à jour », téléchargeables (`BackupKind::PreUpgrade`, `kind = "pre-upgrade"`). Aucune
  entrée au journal d'audit (aucun utilisateur n'a agi ; le fichier et le journal du serveur suffisent ; le
  téléchargement, lui, est déjà audité).
- **Écartées** : un bandeau après la mise à jour (état à stocker : une migration) ; une entrée d'audit signée par le plus
  ancien administrateur, comme la réparation (`installation.repaired`) — bruit pour un geste sans décision.
- **Réversible** : oui.

### C-v0174m-10 — La restauration : base recréée vide, image précédente, assistant — **remplacé par C-v0174m-32** (aucune procédure de restauration neuve ; le corps ci-dessous est l'historique)

- **Contexte** : la version de départ refuse de démarrer sur la base migrée (migration appliquée inconnue de `sqlx`,
  `min_required` le cas échéant) : on ne peut pas importer par son écran sans lui rendre une base qu'elle accepte.
- **Retenu** : la procédure du manuel — arrêter Kesh ; prendre un vidage de la base migrée (`kesh-dump.sh` sur Synology, `mariadb-dump` sinon ; filet,
  garde G16) ; recréer la base **vide** sous le même nom (`DROP DATABASE` / `CREATE DATABASE`) **par le compte Kesh** de
  `DATABASE_URL`, dans les deux topologies — *amendé à la validation P1* (R1-2) : la production Synology n'a pas de
  conteneur MariaDB (`docker-compose.prod.yml` sans service `mariadb`) et la garde G16 y interdit le compte root ; le
  manuel (`admin-manual.tex:2005-2010`) et `scripts/synology/kesh-restore.sh` font déjà `DROP`/`CREATE DATABASE` par ce
  compte, qui porte `ALL PRIVILEGES` sur la base : la question des droits est tranchée par ce précédent, la recette le
  confirme (C-v0174m-19) ; reprendre l'image
  précédente ; `/setup` puis « Restaurer une sauvegarde » de l'assistant (31-1b) avec le fichier `pre-upgrade/…`. Le
  dossier des documents reste en place (une pièce présente et intacte est laissée telle quelle,
  `AlreadyIntact`, `document_storage.rs:248`). Jouée en recette en deux jambes (C-v0174m-22) et deux topologies (C-v0174m-19).
- **Écartées** : une base sous un autre nom (exige un `GRANT` par root, que le manuel éviterait) ; restaurer par le binaire
  neuf (son inventaire n'est pas celui du fichier).
- **Réversible** : oui (texte du manuel).

### C-v0174m-11 — La recette se joue de 0.17.2 à 0.17.4, quelle que soit la story `v0174-session` — **retiré par C-v0174m-30** (seule la jambe 0.15.0 de C-22 est jouée)

- **Contexte** : la sauvegarde ne se déclenche que si la version d'arrivée porte une migration. Seule `v0174-session` peut
  en porter une en 0.17.4 (consigne commune) ; si elle n'en porte pas, rien ne se passe de la 0.17.3 à la 0.17.4.
- **Retenu** : recette avec l'image publiée `gcorbaz/kesh:0.17.2` (88 migrations) → l'image de la branche (89 ou plus) →
  refus de la 0.17.2 → restauration → redémarrage de la 0.17.2 sur des données et des pièces revenues. Si `v0174-session`
  ajoute une migration, la recette se rejoue aussi de 0.17.3 à 0.17.4 après l'intégration.
- **Écartées** : attendre une migration de la 0.17.4 pour jouer la recette.
- **Réversible** : oui.
- *Amendé à la validation P1* : cette jambe ne discrimine pas l'inventaire (la 0.17.3 n'ajoute qu'une table système) ;
  une jambe 0.15.0 → branche s'y ajoute (C-v0174m-22), et la jambe 0.17.2 se joue dans la topologie de production
  (C-v0174m-19). `v0174-session` ne prévoit pas de migration (C-v0174s-2).

### C-v0174m-12 — Deux lots dans une fiche, dérogation de splitting déclarée — **remplacé par C-v0174m-18**

- **Contexte** : modules touchés — `kesh-db/backup` (lecture à l'inventaire du schéma), `kesh-api` (module neuf,
  `main.rs`, `config.rs`), `kesh-api/auto_backup` (liste et rétention), `frontend/admin-backup`, `kesh-i18n` (une clé,
  quatre locales) : six au sens de la *Règle de splitting préventif*.
- **Retenu** : une fiche, deux lots — **A** (sauvegarde, refus, journal, échappatoire, manuel, CHANGELOG) et **B** (le type
  « Avant mise à jour » dans la liste du serveur : `BackupKind`, frontend, i18n, captures), B mécanique et dépendant du
  nom fixé par A. Dérogation déclarée au Project Lead dans le rapport.
- **Écartées** : deux fiches (B seule n'a pas de sens sans A ; la boucle de validation porterait deux fois la même
  conception).
- **Réversible** : oui (le découpage peut se faire à la validation si le signal de stagnation se lève).
- *Annoté à la validation P1 (R1-10)* : la règle n'admet de dérogation qu'en cas de cycle Cargo ou de merges
  intermédiaires intestables ; aucun ici. Choix remplacé par C-v0174m-18.

### C-v0174m-13 — Aucune migration ; #848 hors périmètre

- **Contexte** : consigne commune ; #848 porte sur le nom du fichier **téléchargé** depuis l'écran
  (`routes/admin.rs:92-95`).
- **Retenu** : aucune migration (rétention constante, rien n'est stocké en base). #848 ne se règle pas du même geste :
  le nom de la sauvegarde avant mise à jour est neuf et porte déjà l'heure ; celui de l'export manuel n'est pas touché.
- **Écartées** : régler #848 dans le même geste — le nom téléchargé est celui de `routes/admin.rs:92-95`, que la story ne
  touche pas.
- **Réversible** : oui.

### C-v0174m-14 — Manifeste : `keshVersion` dit qui a écrit, le nom dit de quoi

- **Contexte** : `BackupManifest::new` fige `kesh_version = CARGO_PKG_VERSION` (`manifest.rs:116`) ; l'import ne s'en sert
  que pour l'audit (`routes/admin.rs:254`, `:601`), le contrôle de compatibilité portant sur `min_required`
  (`:223-230`).
- **Retenu** : inchangé — le manifeste dit la version qui a écrit (la version d'arrivée), `kesh_version_min_required` est
  celle de la base lue (la version de départ l'accepte), et le nom du fichier porte les deux versions. Aucun champ neuf.
- **Écartées** : écrire la version de départ dans `keshVersion` (le champ mentirait sur l'auteur) ; un champ neuf
  (`sourceVersion` : la version de départ l'ignorerait, et rien ne le lirait).
- **Réversible** : oui.

---

## Choix ajoutés à la remédiation de la validation P1 (2026-11-05)

### C-v0174m-15 — Tentatives répétées d'une même mise à jour : la plus ancienne du couple est protégée — **retiré à la validation P2 (C-v0174m-25)**

- **Contexte** (R1-1, F1-3, prémisse vérifiée : `sqlx-mysql-0.8.6/src/migrate.rs`, `apply` insère la ligne
  `success = FALSE` **dans** la transaction ; une migration qui échoue avant tout DDL est annulée sans ligne « sale ») :
  une mise à jour qui applique la migration *k* puis échoue proprement sur *k+1* laisse un schéma à moitié migré **sans**
  ligne sale ; `kesh_version_last_applied` est resté la version de départ ; chaque redémarrage (`restart: unless-stopped`)
  écrit une sauvegarde du même couple `<de>-to-<vers>`, inutilisable par la version de départ, et la rétention à trois
  finissait par supprimer la seule bonne. La Dev Note « la rétention ne peut pas évincer la bonne » était fausse.
- **Retenu** : Kesh écrit toujours une sauvegarde à chaque tentative (l'état peut avoir changé si la version de départ a
  servi entre-temps), mais la rétention ne supprime **jamais la plus ancienne sauvegarde du couple courant**
  `<de>-to-<vers>` ; elle garde en outre les trois plus récentes (au plus quatre fichiers). Le message de refus dit
  « **Cette tentative** n'a appliqué aucune migration » (toujours vrai) ; s'il existe déjà un fichier du même couple, il
  ajoute qu'une tentative précédente a pu en appliquer, nomme la plus ancienne et renvoie au manuel ; la promesse « la
  {de} peut redémarrer dessus » n'est écrite que s'il n'en existe pas. Le journal du plan dit, le cas échéant,
  « tentative n° N de la même mise à jour ».
- **Écartées** : ne jamais réécrire un couple déjà sauvegardé (fausse si la version de départ a servi après une première
  tentative avortée sans migration : la sauvegarde gardée serait périmée) ; détecter le schéma à moitié migré (la liste
  des migrations de la version de départ n'est pas connue du binaire neuf).
- **Réversible** : oui.

### C-v0174m-16 — Plan illisible : refus

- **Contexte** (R1-3, F1-2) : `check_downgrade_protection` distingue 1146 de toute autre erreur ; rien de tel n'était
  prescrit pour la lecture de `_sqlx_migrations`, et une implémentation « erreur → neuve » migrerait sans sauvegarde.
- **Retenu** : seule l'erreur 1146 (table absente) vaut *neuve* ; toute autre erreur de lecture du plan → `Refus::Plan`,
  message de refus, `exit(1)` avant `MIGRATOR.run`.
- **Écartées** : traiter l'erreur comme *à jour* ou *neuve*.
- **Réversible** : oui.

### C-v0174m-17 — L'importabilité par la version de départ se garde dans le temps et se joue en test — **réduit par C-v0174m-30** (garde de format (a) retirée ; aller-retour (b)(c) gardé)

- **Contexte** (R1-4, F1-4) : le fichier est écrit par le binaire neuf et lu par l'ancien ; un changement de
  `BACKUP_FORMAT_VERSION` (`manifest.rs:27`), du manifeste ou de `decode_cell` le rendrait silencieusement inimportable ;
  et aucun test ne rejouait l'import sur le fichier produit, alors que #847 l'exige.
- **Retenu** : (a) une garde `format_de_la_sauvegarde_avant_mise_a_jour_fige` : `BACKUP_FORMAT_VERSION == 2`, ensemble
  exact des clés du manifeste et des types décodés par `decode_cell` ; son message dit qu'un changement casse
  l'importabilité par les versions antérieures et doit être consigné (manuel, CHANGELOG) ; (b) `parse_and_verify` et
  `restore_tables_in_tx` reçoivent en interne l'inventaire à respecter (`parse_and_verify_avec_inventaire`,
  `restore_tables_in_tx_avec_inventaire`, l'inventaire de production restant `TABLES_TO_TRUNCATE` : comportement inchangé,
  appelants de production inchangés) ; (c) un test d'aller-retour discriminant : base arrêtée avant `20261027000001`
  (migration en attente qui crée des tables applicatives), sauvegarde, puis sur une **seconde** base au même état :
  `parse_and_verify_avec_inventaire` avec l'inventaire **littéral** de départ (`TABLES_TO_TRUNCATE` privé de
  `mcp_prompts` et `mcp_prompt_versions`, oracle indépendant du prédicat lu), `check_schema_compat`, restauration, et
  comparaison ligne à ligne avec la base source ; (d) la recette sur une image publiée dont l'inventaire diffère
  (C-v0174m-22).
- **Écartées** : paramétrer le format par version cible (aucun besoin aujourd'hui) ; se contenter de la recette.
- **Réversible** : oui (refactor interne, production inchangée).

### C-v0174m-18 — Deux sous-fiches sœurs, parente en `split` — **amendé par C-v0174m-30** (la sous-fiche 2 est `superseded`)

- **Contexte** (R1-10) : la *Règle de splitting préventif* n'admet pas la dérogation C-v0174m-12 ; chaque lot compte moins
  de cinq modules, le lot A est livrable seul (le fichier se récupère par le dossier), le lot B ne dépend que du nom.
- **Retenu** : `v0174-mise-a-jour.md` passe en `split` (analyse commune, conception, découpage) ;
  `v0174-mise-a-jour-1.md` (la sauvegarde avant migration : `kesh-db`, `kesh-api`, compose, manuel, CHANGELOG, recette) et
  `v0174-mise-a-jour-2.md` (le type « Avant mise à jour » à l'écran : `auto_backup.rs` liste, frontend, i18n, captures),
  sur la même branche, développées dans cet ordre, comme `v0174-session`.
- **Écartées** : une fiche avec section de dérogation (critère d'exception non rempli).
- **Réversible** : oui.

### C-v0174m-19 — Retour en arrière : une procédure, deux topologies, deux recettes — **remplacé par C-v0174m-32** (le corps ci-dessous est l'historique)

- **Contexte** (R1-2, R1-8, F1-11) : la production est la topologie Synology (Kesh seul, MariaDB du paquet DSM,
  `mem_limit: 1g`, `docker-compose.prod.yml:193`) ; le compose standard a son conteneur `mariadb`.
- **Retenu** : la base se recrée **par le compte Kesh** (client `mariadb` d'un conteneur jetable lisant
  `kesh-restore.cnf` sur Synology, comme `kesh-restore.sh` ; `docker compose exec mariadb mariadb -u"$MARIADB_USER" -p`
  dans le compose standard), nom de base `MARIADB_DATABASE` (défaut `kesh`) ; vidage de sécurité d'abord ; le fichier se
  télécharge par l'écran **avant** d'arrêter Kesh (ou se copie par SSH, `sudo cp` + `chown` comme pour la pré-import) ;
  l'image à remettre est la version `<de>` du nom du fichier, épinglée (le compose pointe `:latest`) ; ce que le retour
  perd est dit : la saisie faite depuis la mise à jour, et les réglages hors fichier (planification de la sauvegarde
  automatique, durée d'inactivité de la session), remis aux défauts ; sans `/setup` si `KESH_ADMIN_USERNAME` est posé. La
  recette se joue **dans les deux topologies** : compose standard, et topologie de production reproduite sur la station
  (conteneur MariaDB autonome sur un réseau jetable, conteneur Kesh seul joint par `DATABASE_URL`, `--memory 1g`, compte
  Kesh sans root). Si la seconde ne peut pas se monter sur la station, le Dev Agent Record le dit et le manuel marque la
  procédure Synology « non éprouvée » (précédent : la 0.16.0 pour Claude Code).
- **Écartées** : le compte root (absent sur Synology, interdit par G16) ; une base sous un autre nom (exige un `GRANT`).
- **Réversible** : oui (texte du manuel).

### C-v0174m-20 — Des pièces manquantes ne refusent pas la migration — **réduit par C-v0174m-30** (en vigueur : des pièces manquantes ne refusent pas ; sans `warn` récapitulatif ni phrase du manuel)

- **Contexte** (R1-12 b, F1-8) : AC-8 décidait sans choix qu'une pièce référencée absente, altérée ou illisible laisse la
  sauvegarde écrite et la migration se faire.
- **Retenu** : maintenu — les migrations ne touchent pas le dossier des documents, la pièce manquante l'est aussi pour la
  version de départ, et la sauvegarde automatique a le même statut « incomplète » ; un `warn` nomme le nombre et renvoie au
  journal. Réserve écrite : `unreadable` peut être transitoire (montage) ; le manuel dit de vérifier le montage du dossier
  des documents avant la mise à jour.
- **Écartées** : refuser (un dossier démonté bloquerait toute mise à jour, l'échappatoire deviendrait l'usage).
- **Réversible** : oui.

### C-v0174m-21 — L'échappatoire quand le compose ne la transmet pas — **retiré à la validation P2 (C-v0174m-28)**

- **Contexte** (R1-9, F1-6) : les compose n'ont pas d'`env_file` ; un compose antérieur à la 0.17.4 ne transmet pas
  `KESH_SKIP_PRE_UPGRADE_BACKUP`, si bien qu'une ligne au seul `.env` reste sans effet et sans avertissement.
- **Retenu** : le message de refus, le manuel et le CHANGELOG disent les deux voies : la ligne sous `environment:` du
  service `kesh-api` (compose 0.17.4 : déjà présente, valeur vide), ou, sans toucher au compose,
  `docker compose run --rm -e KESH_SKIP_PRE_UPGRADE_BACKUP=<vers> kesh-api` (une exécution, puis `docker compose up -d`).
  Le CHANGELOG conseille de reprendre la ligne dans le compose local dès la 0.17.4.
- **Écartées** : lire la variable dans un fichier du dossier de sauvegarde (un réglage de plus, caché).
- **Réversible** : oui.

### C-v0174m-22 — Une jambe de recette dont l'inventaire diffère : 0.15.0 → branche — **réduit par C-v0174m-30** (une seule jambe, 0.15.0 → branche)

- **Contexte** (F1-4) : de la 0.17.2 à la branche, `TABLES_TO_TRUNCATE` est identique (la 0.17.3 n'ajoute qu'une table
  système) : la recette C-v0174m-11 survivrait à la mutation M4. La 0.15.0 est publiée et n'a pas les tables du serveur
  MCP ajoutées en 0.16.0.
- **Retenu** : deux jambes — **0.15.0 → branche** (exerce C-v0174m-1 et la borne C-v0174m-8 : refus de la 0.15.0 sur la
  base migrée, restauration par son assistant) et **0.17.2 → branche** (retour sur la série en production). Plus, si une
  autre story de la 0.17.4 ajoute une migration, 0.17.3 → 0.17.4 à l'intégration.
- **Écartées** : la seule jambe 0.17.2.
- **Réversible** : oui.

### C-v0174m-23 — Une table étrangère au schéma : avertissement — **retiré par C-v0174m-30**

- **Contexte** (R1-15) : une table créée hors migration (à la main, par un outil) entrerait dans l'inventaire et rendrait
  le fichier inimportable par la version de départ.
- **Retenu** : si une table lue n'est pas dans le `TABLES_TO_TRUNCATE` du binaire neuf (les tables ne font que s'ajouter
  entre versions : départ ⊆ arrivée), un `warn` la nomme ; la sauvegarde est écrite quand même (elle reste la seule copie
  de l'état d'avant).
- **Écartées** : la retirer de l'inventaire (le fichier ne serait plus la photographie de la base) ; refuser.
- **Réversible** : oui.

### C-v0174m-24 — Prédicat de l'inventaire aligné sur la garde

- **Contexte** (F1-9) : la garde `backup_inventory_matches_schema` ne filtre pas `TABLE_TYPE` ; un filtre
  `BASE TABLE` divergerait d'elle sur un type que la garde verrait.
- **Retenu** : même prédicat que la garde (toutes les tables de `DATABASE()` dans `information_schema.TABLES`), filtre du
  préfixe `_` en Rust ; la garde T15 emploie le même. Amende C-v0174m-1.
- **Écartées** : `TABLE_TYPE = 'BASE TABLE'`.
- **Réversible** : oui.

---

## Choix de la remédiation de la validation P2 (2026-11-05)

*Signal de non-convergence déclaré au Project Lead par l'orchestrateur : la rétention a produit un HIGH en P1 (R1-1) et un
autre en P2 (V2-1), l'échappatoire par `docker compose run` un MEDIUM à chaque passe. Décision de l'orchestrateur :
simplifier plutôt que corriger.*

### C-v0174m-25 — Aucune suppression automatique des sauvegardes d'avant mise à jour (retire C-6 et C-15) — **réduit par C-v0174m-30** (en vigueur : aucune suppression ; la règle de choix du fichier tient en une phrase du manuel, non dans le journal)

- **Contexte** : V2-1 (prémisse confirmée : `record_boot_version`, `version.rs:325`, met `last_applied` à jour à chaque
  démarrage réussi) — la protection « plus ancienne du couple » supprime la seule sauvegarde restaurable si la version
  d'arrivée change entre deux tentatives, ou si la version de départ a servi entre deux tentatives. Chaque correctif de
  rétention ajoutait un critère, et le critère suivant un défaut.
- **Retenu** : Kesh ne supprime **jamais** un fichier `kesh-pre-upgrade-*.keshbackup`. Chaque tentative écrit le sien ; ils
  s'accumulent dans `pre-upgrade/` (quelques-uns par an) ; le manuel dit de retirer les anciens à la main une fois la mise
  à jour éprouvée. Le seuil d'espace (C-v0174m-7) reste. Le journal et le refus nomment le fichier de **cette** tentative et
  rappellent la règle : si une tentative précédente a déjà migré en partie, c'est la plus ancienne sauvegarde du dossier
  écrite après le dernier démarrage réussi de la version de départ qu'il faut restaurer.
- **Écartées** : la rétention à trois (C-6) ; la plus ancienne du couple (C-15) ; la plus ancienne depuis `last_boot_at`
  (proposée par V2-1 : un mécanisme de plus, avec un piège de fuseau, dans la zone qui recyclait).
- **Réversible** : oui (une rétention peut s'ajouter plus tard, si l'accumulation gêne).

### C-v0174m-26 — `admin_backup` devient `pub mod` — **retiré par C-v0174m-30** (tests internes)

- **Contexte** : V2-3, prémisse confirmée (`lib.rs:14` : `pub(crate) mod admin_backup;`) : le test d'aller-retour, crate
  externe, ne compilerait pas.
- **Retenu** : `pub mod admin_backup` (comme `auto_backup`, `lib.rs:18`), le test restant dans `crates/kesh-api/tests/`.
- **Écartées** : un test unitaire dans `import.rs` (il faudrait inclure les helpers de montage de `kesh-db` depuis `src/`).
- **Réversible** : oui.

### C-v0174m-27 — Un plan illisible ne propose pas l'échappatoire — **retiré par C-v0174m-30** (remplacé par C-v0174m-31)

- **Contexte** : V2-5 — l'échappatoire est évaluée après le plan ; la conseiller sur `Refus::Plan` serait un geste sans
  effet, et `{de}` n'y est pas encore lu.
- **Retenu** : `Refus::Plan` : « L'état des migrations n'a pas pu être lu. Corrigez la cause : connexion à la base ou droits
  de lecture de `_sqlx_migrations`. » — sans `{de}`, sans échappatoire. L'ordre plan → échappatoire est gardé.
- **Écartées** : évaluer l'échappatoire avant le plan (elle contournerait un état illisible, y compris un état
  « interrompu » qu'on ne saurait plus voir).
- **Réversible** : oui.

### C-v0174m-28 — Échappatoire : une seule voie (retire C-21)

- **Contexte** : V2-8 — `docker compose run --rm -e …` ne se termine pas (le processus va jusqu'à `axum::serve`) et le
  conteneur du service redémarre en même temps : deux Kesh sur une base.
- **Retenu** : la variable dans `.env`, transmise par `docker-compose.yml` et `docker-compose.prod.yml` de la 0.17.4
  (`${KESH_SKIP_PRE_UPGRADE_BACKUP:-}`) ; pour un compose antérieur, le manuel et le message disent d'y ajouter la ligne.
  Rangement dans la garde : `SANS_DEFAUT` (V2-2 : vide = absent = sauvegarde faite ; `VIDE_SIGNIFIANT` est réservé à un vide
  qui a un sens distinct de l'absence, `configuration_transmise.rs:122`, `:132`).
- **Écartées** : `docker compose run` ; un fichier témoin dans le dossier de sauvegarde.
- **Réversible** : oui.

### C-v0174m-29 — La procédure Synology du retour vit dans `sec:backup-dsm` — **retiré par C-v0174m-30** (renvoi à `sec:backup-dsm`, aucune procédure Synology neuve)

- **Contexte** : V2-19 — les gardes G14 à G16 ne lisent que les sections Synology (`common/manuel.rs:74` : `sec:synology`,
  `sec:backup-dsm`) ; une procédure Synology écrite sous *Revenir à la version précédente* leur échapperait.
- **Retenu** : la jambe Synology (vidage par `kesh-dump.sh`, recréation par le compte Kesh via `kesh-restore.cnf`) est une
  sous-section de `sec:backup-dsm`, à laquelle *Revenir à la version précédente* renvoie ; la jambe du compose standard reste
  dans la section générale.
- **Écartées** : étendre les bornes des gardes (changement d'un test sans rapport avec la story) ; écrire l'angle mort.
- **Réversible** : oui.

---

## Décision du Project Lead (2026-11-05) — version minimale

### C-v0174m-30 — Version minimale de #847

- **Contexte** : la validation n'a pas convergé — P1 2 HIGH, P2 1 HIGH, P3 1 HIGH (0/1/5/4), chacun né de la remédiation
  précédente (rétention, puis protection du couple, puis nettoyage du `.partial` sur panique) ; signal déclaré au Project
  Lead par l'orchestrateur.
- **Retenu (décision du Project Lead)** : (1) au démarrage, s'il reste des migrations (plan lu dans `_sqlx_migrations`
  contre le `MIGRATOR` ; illisible → refus), un `.keshbackup` de format 2, pièces comprises, à l'inventaire du schéma
  présent, dans `<KESH_ADMIN_BACKUP_DIR>/pre-upgrade/`, nom portant les versions de départ et d'arrivée ; (2) écriture dans
  un temporaire puis renommage ; tout échec (dossier, espace, lecture, écriture, panique) refuse le démarrage sans
  migration ; aucun nettoyage exigé du `.partial` sur panique (inoffensif, dit au manuel) ; (3) échappatoire
  `KESH_SKIP_PRE_UPGRADE_BACKUP=<version exacte>` dans le `.env`, transmise par les deux compose ; (4) jamais de
  suppression automatique ; le journal donne le chemin ; le manuel dit où trouver le fichier, comment revenir (base recréée
  vide par le compte Kesh, image précédente, assistant), qu'on retire les anciens à la main, et, en une phrase, qu'après
  une tentative qui a migré en partie on restaure le plus ancien fichier écrit pour cette mise à jour. Gardés : le test
  d'aller-retour discriminant, le refus sur échec, l'absence de sauvegarde sans migration ou sur base neuve, et les
  mutations qui tuent ces points. **Retirés** : l'affichage à l'écran (sous-fiche 2 `superseded`, issue), la règle RG
  détaillée, E1 et tout message après `MIGRATOR.run`, la garde de format, `pub mod admin_backup`, le refus de version de
  départ `< 0.15.0` (C-8), la relecture, l'avertissement de table étrangère (C-23) et W1 à W3, le nettoyage sur panique,
  la seconde jambe et la seconde topologie de recette, la procédure Synology neuve (C-29).
- **Écartées** : poursuivre la boucle de validation sur la conception complète (huit passes au plus, chacune recyclant).
- **Réversible** : oui — ce qui est retiré reste au registre et dans la sous-fiche 2 ; l'issue de l'écran le reprendra.

### C-v0174m-31 — L'échappatoire est évaluée en premier

- **Contexte** : C-v0174m-27 évaluait l'échappatoire après le plan, ce qui obligeait un message de refus particulier pour
  le plan illisible (V2-5).
- **Retenu** : après `FreshInstall`, l'échappatoire égale à la version du binaire saute tout (plan compris) ; le message de
  refus est le même pour toutes les causes, et son conseil vaut toujours. Un état « interrompu » non vu dans ce cas est
  refusé ensuite par `sqlx` (`MigrateError::Dirty`).
- **Écartées** : C-v0174m-27.
- **Réversible** : oui.

### Annotations des résidus relevés en P3 (V3-5, V3-7)

- **C-v0174m-3 (ii)** : le motif « le journal renvoie à la plus récente … pourrait évincer, par la rétention, la seule
  sauvegarde utile » est caduc (aucune rétention, C-v0174m-25) ; motif en vigueur : sauvegarder un schéma à moitié migré
  n'apporterait rien, `sqlx` refusera.
- **C-v0174m-5** et **C-v0174m-13** : les mentions de « la rétention » sont caduques (C-v0174m-25) ; le nom et le dossier
  ne sont lus que par le module lui-même.
- **C-v0174m-19** : la forme `docker compose exec mariadb mariadb -u"$MARIADB_USER" -p` est **fausse** (développée dans le
  shell de l'hôte, V2-9) ; forme en vigueur : `docker compose exec mariadb sh -c '…'` (sous-fiche 1, § *Manuel*, point 3).

## Remédiation de la validation P4 (2026-11-05)

### C-v0174m-32 — Aucune procédure de restauration neuve

- **Contexte** : V4-1 et V4-2 (HIGH) portaient sur la procédure de retour écrite par la fiche (recréation de la base sans
  vidage de sécurité ; renvoi Synology sans `CREATE DATABASE`). Décision de l'orchestrateur, dans l'esprit de la version
  minimale.
- **Retenu** : la story n'écrit aucune commande de restauration. Le manuel dit où est le fichier, lequel prendre (le plus
  ancien écrit depuis le dernier démarrage réussi de la version de départ ; aucun n'est jamais supprimé), impose de
  commencer par un vidage de sécurité de la base actuelle (§ *Script de sauvegarde mariadb-dump*, ou `kesh-dump.sh`), et
  renvoie aux procédures existantes (§ *Restaurer sur une nouvelle machine*, base vide : § *Initialisation manuelle de la
  base*). La recette joue ces procédures telles qu'écrites sur le compose standard ; un écart devient une issue, non une
  retouche ici. La commande `DROP DATABASE` de `sec:backup-dsm` (`admin-manual.tex:2183-2186`) n'est pas un défaut : elle
  sert le cas « base présente mais illisible », suivi de `kesh-restore.sh`, dont le dump recrée la base.
- **Écartées** : la procédure de recréation par le compte Kesh (C-10, C-19), qui a produit un HIGH à chaque passe.
- **Réversible** : oui.

## Choix en vigueur après la décision du Project Lead

C-v0174m-1, -2, -3 (motif annoté), -4 (réduit), -5 (annoté), -7, -9 (réduit), -13 (annoté), -14, -16, -17 (réduit),
-18 (amendé), -20 (réduit), -22 (réduit à une jambe), -24 (avec la garde T16 de la fiche), -25 (réduit), -28, -30, -31, -32.
Retirés ou remplacés : -6, -8, -10, -11, -12, -15, -19, -21, -23, -26, -27, -29.
