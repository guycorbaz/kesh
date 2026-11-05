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
- **Retenu** : une lecture dont l'inventaire est **celui du schéma présent** — les tables de base
  (`information_schema.TABLES`, `TABLE_TYPE = 'BASE TABLE'`) de `DATABASE()` dont le nom ne commence pas par `_`. Elle
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
- **Retenu** : un **plan** lu avant `run` : (i) `_sqlx_migrations` absente ou vide → *installation neuve*, pas de
  sauvegarde ; (ii) une ligne `success = 0` → *migration interrompue*, pas de sauvegarde (le journal renvoie à la plus
  récente sauvegarde avant mise à jour : `sqlx` refusera de toute façon, et sauvegarder un schéma à moitié migré
  pourrait évincer, par la rétention, la seule sauvegarde utile) ; (iii) aucune migration connue (montante) absente →
  *à jour*, pas de sauvegarde ; (iv) sinon → *à appliquer*, avec leur nombre et la première version. Une migration
  appliquée inconnue ne change pas le plan : `check_downgrade_protection` ou `sqlx` refuse le démarrage ensuite.
- **Écartées** : comparer `kesh_version_last_applied` à la version du binaire (une version sans migration déclencherait
  une sauvegarde inutile ; une version de développement avec migration n'en déclencherait pas) ; sauvegarder à chaque
  démarrage (coût et disque).
- **Réversible** : oui.

### C-v0174m-4 — Un échec de la sauvegarde refuse le démarrage ; l'échappatoire est une variable qui nomme la version

- **Contexte** : #847, point 2 ; c'est un réglage de démarrage (l'interface n'est pas encore servie).
- **Retenu** : tout échec (dossier, espace, lecture de la base, écriture, relecture, version de départ trop ancienne)
  → message clair au journal et `exit(1)` **avant** `MIGRATOR.run` : la base est intacte, l'image précédente redémarre
  dessus. Échappatoire : `KESH_SKIP_PRE_UPGRADE_BACKUP=<version>` — elle ne vaut que si sa valeur est **exactement** la
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

### C-v0174m-6 — Rétention : les trois plus récentes

- **Contexte** : chaque sauvegarde pèse la base plus les documents ; une boucle de redémarrage (`restart: unless-stopped`)
  après un échec de migration non « sale » referait une sauvegarde à chaque tentative.
- **Retenu** : après une sauvegarde **réussie et relue**, Kesh garde les **trois** plus récentes de `pre-upgrade/`
  (fichiers réguliers au nom exact de l'expression), jamais celle qui vient d'être écrite ; un échec ne supprime rien.
  Constante, sans réglage (pas de migration ; une valeur dans `.env` serait une variable de plus pour un cas rare).
- **Écartées** : ne jamais purger (comme les pré-import : le disque du NAS se remplit d'une copie complète par version) ;
  réutiliser le nombre de la sauvegarde automatique (lu dans `_kesh_backup_schedule`, qu'une base antérieure à la 0.15.0
  n'a pas, et réglage d'un autre usage).
- **Réversible** : oui.

### C-v0174m-7 — Seuil d'espace : celui de la sauvegarde automatique

- **Contexte** : `seuil_d_espace` (`auto_backup.rs:510`) = `max(dernière, documents) × 1,2 + 256 Mio`.
- **Retenu** : la même fonction, avec pour « dernière » la plus grande sauvegarde présente dans `auto/` et
  `pre-upgrade/`, et l'espace mesuré par `BackupEnv::free_bytes` (injectable, pour les tests). En dessous : refus (C-4).
- **Écartées** : ne rien mesurer (un disque plein se découvrirait en pleine écriture, ce qui reste possible mais rare) ;
  mesurer la base (`information_schema` ne donne qu'une estimation, et la sauvegarde automatique ne le fait pas).
- **Réversible** : oui.

### C-v0174m-8 — Version de départ antérieure à la 0.15.0 ou inconnue : refus, sauf échappatoire

- **Contexte** : une sauvegarde de format 2 ne s'importe que dans une 0.15.0 ou plus récente (`manifest.rs:18-27`) ; la
  lecture des pièces exige `invoices.pdf_storage_path` (migration `20261003000001`). Écrire depuis une 0.14.x un fichier
  que la 0.14.x ne peut pas importer serait un filet qui ne retient rien, et le dire seulement au journal le rendrait muet.
- **Retenu** : si `kesh_version_last_applied` ne se lit pas comme une version, ou est inférieure à 0.15.0, Kesh refuse la
  migration avec un message qui dit pourquoi et renvoie au vidage manuel de la base (`kesh-dump.sh` sur Synology, `mariadb-dump` sinon) puis à l'échappatoire.
- **Écartées** : écrire la sauvegarde quand même avec un avertissement ; la sauter en silence.
- **Réversible** : oui. Sans portée pour la production (0.17.3).

### C-v0174m-9 — Le journal de démarrage et l'écran

- **Contexte** : #847, point 3.
- **Retenu** : quatre lignes `info` (plan : nombre de migrations, de → vers ; base lue : tables, lignes, durée ; pièces :
  emportées, octets, non incluses ; fichier : chemin, taille, durée, et le renvoi au manuel « Revenir à la version
  précédente »), `warn` si des pièces manquent ou si le fichier dépasse `KESH_ADMIN_IMPORT_MAX_MB` (la version de départ
  ne pourrait l'importer par l'écran qu'en relevant ce plafond). À l'écran : la liste *Sauvegardes sur le serveur* montre
  ces fichiers, type « Avant mise à jour », téléchargeables (`BackupKind::PreUpgrade`, `kind = "pre-upgrade"`). Aucune
  entrée au journal d'audit (aucun utilisateur n'a agi ; le fichier et le journal du serveur suffisent ; le
  téléchargement, lui, est déjà audité).
- **Écartées** : un bandeau après la mise à jour (état à stocker : une migration) ; une entrée d'audit signée par le plus
  ancien administrateur, comme la réparation (`installation.repaired`) — bruit pour un geste sans décision.
- **Réversible** : oui.

### C-v0174m-10 — La restauration : base recréée vide, image précédente, assistant

- **Contexte** : la version de départ refuse de démarrer sur la base migrée (migration appliquée inconnue de `sqlx`,
  `min_required` le cas échéant) : on ne peut pas importer par son écran sans lui rendre une base qu'elle accepte.
- **Retenu** : la procédure du manuel — arrêter Kesh ; prendre un vidage de la base migrée (`kesh-dump.sh` sur Synology, `mariadb-dump` sinon ; filet,
  garde G16) ; recréer la base **vide** sous le même nom (`DROP DATABASE` / `CREATE DATABASE` par le compte root du conteneur
  MariaDB — les droits de `kesh`, posés au niveau de la base, survivent : **à établir en recette**) ; reprendre l'image
  précédente ; `/setup` puis « Restaurer une sauvegarde » de l'assistant (31-1b) avec le fichier `pre-upgrade/…`. Le
  dossier des documents reste en place (une pièce présente et intacte est laissée telle quelle,
  `AlreadyIntact`, `document_storage.rs:248`). Jouée en recette de 0.17.2 à 0.17.4 (la 0.17.3 a ajouté une migration ; voir C-11).
- **Écartées** : une base sous un autre nom (exige un `GRANT` par root, que le manuel éviterait) ; restaurer par le binaire
  neuf (son inventaire n'est pas celui du fichier).
- **Réversible** : oui (texte du manuel).

### C-v0174m-11 — La recette se joue de 0.17.2 à 0.17.4, quelle que soit la story `v0174-session`

- **Contexte** : la sauvegarde ne se déclenche que si la version d'arrivée porte une migration. Seule `v0174-session` peut
  en porter une en 0.17.4 (consigne commune) ; si elle n'en porte pas, rien ne se passe de la 0.17.3 à la 0.17.4.
- **Retenu** : recette avec l'image publiée `gcorbaz/kesh:0.17.2` (88 migrations) → l'image de la branche (89 ou plus) →
  refus de la 0.17.2 → restauration → redémarrage de la 0.17.2 sur des données et des pièces revenues. Si `v0174-session`
  ajoute une migration, la recette se rejoue aussi de 0.17.3 à 0.17.4 après l'intégration.
- **Écartées** : attendre une migration de la 0.17.4 pour jouer la recette.
- **Réversible** : oui.

### C-v0174m-12 — Deux lots dans une fiche, dérogation de splitting déclarée

- **Contexte** : modules touchés — `kesh-db/backup` (lecture à l'inventaire du schéma), `kesh-api` (module neuf,
  `main.rs`, `config.rs`), `kesh-api/auto_backup` (liste et rétention), `frontend/admin-backup`, `kesh-i18n` (une clé,
  quatre locales) : six au sens de la *Règle de splitting préventif*.
- **Retenu** : une fiche, deux lots — **A** (sauvegarde, refus, journal, échappatoire, manuel, CHANGELOG) et **B** (le type
  « Avant mise à jour » dans la liste du serveur : `BackupKind`, frontend, i18n, captures), B mécanique et dépendant du
  nom fixé par A. Dérogation déclarée au Project Lead dans le rapport.
- **Écartées** : deux fiches (B seule n'a pas de sens sans A ; la boucle de validation porterait deux fois la même
  conception).
- **Réversible** : oui (le découpage peut se faire à la validation si le signal de stagnation se lève).

### C-v0174m-13 — Aucune migration ; #848 hors périmètre

- **Contexte** : consigne commune ; #848 porte sur le nom du fichier **téléchargé** depuis l'écran
  (`routes/admin.rs:92-95`).
- **Retenu** : aucune migration (rétention constante, rien n'est stocké en base). #848 ne se règle pas du même geste :
  le nom de la sauvegarde avant mise à jour est neuf et porte déjà l'heure ; celui de l'export manuel n'est pas touché.
- **Écartées** : —
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
