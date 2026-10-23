# Epic 31 — registre des choix autonomes de la Story 31-1

Registre propre à la story 31-1 (#503), versé par union dans `epic-31-choix-autonomes.md` par l'orchestrateur
(`docs/agents-orchestration.md` § 1). Fiche : `31-1-sauvegarde-complete.md`.

---

### C-31-1-1 — Trois stories, une fiche

- **Contexte** : la règle de découpage préventif (plus de cinq modules) ; l'epic prévoyait deux volets.
- **Retenu** : 31-1a (fichiers dans la sauvegarde, côté serveur), 31-1b (restauration sur une installation neuve,
  côté écran, E2E de bout en bout), 31-1c (sauvegarde automatique). Une fiche, trois clés, trois PR ; `closes #503` sur
  la seule PR de la 31-1c.
- **Écartées** : deux stories (fichiers + restauration réunis : six modules) ; quatre (l'écran de la sauvegarde
  automatique à part : la 31-1c ne serait plus testable de bout en bout seule).
- **Réversible** : oui, avant le développement.

### C-31-1-2 — Seuls les fichiers référencés par la base, toutes lignes confondues

- **Contexte** : le dossier des documents peut contenir des fichiers que rien ne désigne (rendus concurrents d'un PDF
  figé) ; les documents fournisseur écartés restent des lignes.
- **Retenu** : les fichiers désignés par `imported_supplier_invoices.storage_path` (tous statuts) et
  `invoices.pdf_storage_path` ; rien d'autre.
- **Écartée** : copier tout le dossier — emporte des orphelins, et rend impossible la vérification « chaque fichier a
  sa ligne ».
- **Réversible** : oui (format versionné).

### C-31-1-3 — Fichiers stockés sans compression dans l'archive

- **Retenu** : `CompressionMethod::Stored` pour `files/` (PDF, PNG, JPEG déjà compressés) ; `Deflated` inchangé pour
  `data/` et le manifeste.
- **Écartée** : `Deflated` partout — du temps processeur pour quelques pour cent.
- **Réversible** : oui.

### C-31-1-4 — Un fichier absent ou altéré n'empêche pas la sauvegarde

- **Contexte** : une installation restaurée naguère sans ses documents (#503) a des lignes sans fichier ; refuser
  l'export la priverait de toute sauvegarde.
- **Retenu** : le fichier est listé dans `filesNotIncluded` (raison), pas inclus, signalé par en-tête, audit, journal et
  (31-1c) état de la sauvegarde automatique. L'import accepte ce manque déclaré et le compte dans `filesMissing`.
- **Écartées** : refuser l'export ; inclure un fichier altéré (on restaurerait une pièce fausse sous le nom d'une vraie).
- **Réversible** : oui.

### C-31-1-5 — Export assemblé dans un fichier temporaire « à débordement », dans le dossier de sauvegarde

- **Contexte** : l'archive était entièrement en mémoire ; `KESH_ADMIN_EXPORT_INMEM_MB` règle le seuil de diffusion.
- **Retenu** : `tempfile::SpooledTempFile` au seuil de la variable (sa sémantique est conservée), débordant dans
  `KESH_ADMIN_BACKUP_DIR` (monté) ; aucune variable neuve, aucune retirée (C-31-0-4).
- **Écartées** : `/tmp` du conteneur (couche inscriptible de l'image, non montée) ; rendre la variable inerte.
- **Réversible** : oui.

### C-31-1-6 — Fichiers restaurés avant la base, jamais supprimés

- **Retenu** : après la sauvegarde de sécurité et avant le `DELETE`, écriture atomique des fichiers ; un échec refuse
  l'import base intacte ; les fichiers déjà écrits restent (nommés par leur contenu, ils ne désignent rien de faux).
  L'écriture atomique est factorisée depuis `store_document` (DRY).
- **Écartées** : écrire après le `COMMIT` (une base qui désigne des fichiers pas encore là, et un échec qu'on ne peut
  plus annuler) ; supprimer les fichiers écrits en cas d'échec (un fichier identique pouvait préexister).
- **Réversible** : oui.

### C-31-1-7 — `write_backup_file` générique sur la source

- **Retenu** : la primitive d'écriture sûre (`.partial` `0600`, `create_new`, refus d'écraser, renommage) prend un
  écrivain plutôt que des octets ; la sauvegarde pré-import et la sauvegarde automatique y écrivent l'archive
  directement.
- **Écartée** : une seconde primitive pour les archives en flux (duplication).
- **Réversible** : oui.

### C-31-1-8 — Plafond d'import par défaut porté à 2048 Mio

- **Contexte** : le plafond de 512 Mio protégeait la mémoire ; l'import est désormais reçu sur disque. Une sauvegarde
  avec ses pièces dépasse vite 512 Mio, et la découvrir trop grande le jour de la panne obligerait à modifier le `.env`
  d'une machine neuve.
- **Retenu** : défaut 2048, bornes inchangées ; l'export prévient si son archive dépasse le plafond.
- **Écartées** : garder 512 (piège le jour de la restauration) ; supprimer la borne.
- **Réversible** : oui.

### C-31-1-9 — Archives de plus de 4 Gio : angle mort écrit

- **Retenu** : non testé, écrit au manuel et aux doc-comments ; les entrées sont petites.
- **Écartée** : un test de 4 Gio dans le gate (coût disque et durée).
- **Réversible** : —

### C-31-1-10 — Restaurer depuis l'assistant, par le panneau existant

- **Contexte** : après `/setup`, l'application renvoie à l'assistant ; `/admin/restore` est injoignable (P-12).
- **Retenu** : un lien « Restaurer une sauvegarde » aux étapes 0 à 2 de l'assistant, vers `/onboarding/restore`, qui
  rend `AdminRestorePanel` (Administrateur seulement).
- **Écartées** : exempter `/admin/restore` de la garde de `(app)/+layout.ts` (l'en-tête et le menu de l'application
  supposent une société configurée) ; documenter « passez par la démo puis importez » (charge la démonstration et écrit
  une sauvegarde de sécurité inutile).
- **Réversible** : oui.

### C-31-1-11 — Pas de déconnexion automatique quand des pièces manquent

- **Retenu** : si `filesMissing > 0`, le panneau attend un clic « Se reconnecter » ; sinon comportement actuel.
- **Écartée** : déconnexion après délai (l'avis disparaîtrait avant d'être lu).
- **Réversible** : oui.

### C-31-1-12 — Le `.env` : dit, pas exporté

- **Contexte** : Guy : « la configuration reste hors sauvegarde pour l'instant » ; le commentaire de 2026-10-10 laissait
  ouvert l'export d'une partie non secrète.
- **Retenu** : avis permanent sur les deux écrans et procédure au manuel ; rien de la configuration n'est exporté.
- **Écartée** : exporter les variables non secrètes (choix de la liste, et la configuration doit migrer vers
  l'interface à terme).
- **Réversible** : oui.

### C-31-1-13 — Sauvegarde automatique activée par défaut

- **Contexte** : la demande de Guy est une sauvegarde « faite par Kesh » ; une installation mise à jour sans geste
  resterait sinon sans sauvegarde.
- **Retenu** : activée, chaque jour, 7 conservées ; dit au CHANGELOG et au manuel (place disque, secrets dans
  `./backup/auto`).
- **Écartée** : désactivée par défaut (le défaut sûr pour le disque, pas pour les livres).
- **Réversible** : oui (réglage dans l'interface).

### C-31-1-14 — Fréquences proposées et rétention

- **Retenu** : 6 h, 12 h, 24 h (défaut), 168 h ; rétention 1 à 60 (défaut 7). Pas d'heure fixe (fuseau du conteneur en
  UTC, intervalle suffisant) ; contrainte `CHECK` en base.
- **Écartées** : expression cron libre ; heure du jour (fuseau).
- **Réversible** : oui (migration ultérieure pour élargir).

### C-31-1-15 — Réglages dans une table système hors inventaire de la sauvegarde

- **Contexte** : une table de `TABLES_TO_TRUNCATE` neuve rendrait toutes les sauvegardes existantes non importables
  (P-13) ; les réglages appartiennent à l'installation, comme le `.env`.
- **Retenu** : `_kesh_backup_schedule`, singleton, ni exportée, ni restaurée, ni vidée (convention de `_kesh_version`).
- **Écartées** : colonnes sur `companies` (restaurées depuis une sauvegarde, réglage de société pour un objet
  d'installation) ; table applicative (casse l'importabilité) ; variables d'environnement (C-31-0-4).
- **Réversible** : par migration.

### C-31-1-16 — Sous-dossier `auto/`

- **Retenu** : `KESH_ADMIN_BACKUP_DIR/auto/`, `0700` ; la rétention ne voit que ce dossier et son motif exact.
- **Écartée** : le même dossier que la sauvegarde pré-import (la rétention risquerait d'y toucher).
- **Réversible** : oui.

### C-31-1-17 — Un échec se retente au bout d'une heure

- **Retenu** : `is_due` exige un intervalle depuis la dernière réussite **et** `min(intervalle, 1 h)` depuis la
  dernière tentative.
- **Écartées** : retenter à chaque tour de 5 minutes (journal saturé, disque sollicité) ; attendre l'intervalle complet
  (un jour sans sauvegarde après une panne passagère).
- **Réversible** : oui.

### C-31-1-18 — Tâche de fond : un tour de 5 minutes, isolé

- **Retenu** : `tokio::time::interval` de 5 min, premier tour 5 min après le démarrage, chaque tour dans un
  `tokio::spawn` attendu (une panique est journalisée, la boucle continue) ; lancée par `main.rs` seulement.
- **Écartées** : une bibliothèque cron (dépendance pour un intervalle) ; `sleep` jusqu'à l'échéance (un réglage modifié
  ne serait vu qu'au réveil).
- **Réversible** : oui.

### C-31-1-19 — Espace libre mesuré par `rustix::fs::statvfs`, derrière un trait

- **Retenu** : dépendance directe à `rustix` (au `Cargo.lock`, sans `unsafe`) ; trait `FreeSpace` injecté pour que les
  tests simulent un disque presque plein. Seuil : `dernière taille × 1,2 + 256 Mio`.
- **Écartées** : `libc::statvfs` (`unsafe`) ; `fs4` (paquet neuf) ; ne compter que sur `ENOSPC` (le disque, souvent
  partagé avec la base sur un NAS, se remplirait jusqu'au bout).
- **Réversible** : oui.

### C-31-1-20 — L'échec se voit hors de la page d'administration

- **Retenu** : bandeau pour les Administrateurs dans l'application quand le dernier statut n'est pas `ok`, ou quand
  aucune réussite depuis deux intervalles (tâche morte).
- **Écartées** : la page seule (un échec qu'on ne va pas chercher est un échec muet) ; un e-mail (SMTP facultatif).
- **Réversible** : oui.

### C-31-1-21 — Audit : les réglages et les téléchargements, pas chaque sauvegarde

- **Retenu** : `admin.backup_schedule_updated` (qui a désactivé la sauvegarde se lit au journal) et
  `admin.backup_downloaded` ; l'état courant dans la table ; journal du serveur pour chaque tour.
- **Écartée** : une entrée par sauvegarde (365 par an, et l'audit voyage dans la sauvegarde).
- **Réversible** : oui.

### C-31-1-22 — Lister et télécharger les sauvegardes du serveur ; pas de restauration côté serveur

- **Contexte** : les fichiers sont `root`/`0600` ; le manuel exigeait SSH pour rapatrier la sauvegarde pré-import.
- **Retenu** : liste et téléchargement par l'écran (Administrateur, refusé aux clés) ; l'import reste un téléversement.
- **Écartée** : « Restaurer cette sauvegarde » sans téléchargement — non demandé, signalé à l'orchestrateur comme
  `enhancement` éventuel.
- **Réversible** : oui.

### C-31-1-23 — Une sauvegarde à la fois : verrou en mémoire puis réservation en base

- **Retenu** : `AppState.backup_lock` (tâche automatique en `try_lock`, export, « maintenant » et import en attente) ;
  réservation conditionnelle `last_attempt_at <=> ?` pour un second processus éventuel.
- **Écartées** : `GET_LOCK` MariaDB (fuite sur panique, connexion dédiée — écarté en 17-3c pour la même raison) ;
  aucun verrou (deux archives de plusieurs centaines de Mio en parallèle).
- **Réversible** : oui.

### C-31-1-24 — Migration non breaking, version cible 0.15.0

- **Retenu** : `20261024000001_backup_schedule.sql`, `CREATE TABLE` + ligne singleton ; pas de bump de
  `kesh_version_min_required` ni des crates dans la story (la release 0.15.0 bumpe les crates). Retour à la 0.14.0 par
  la sauvegarde d'avant la mise à jour (sqlx refuse une migration inconnue) — au CHANGELOG. La 31-1a et la 31-1b n'ont
  pas de migration ; le format 2 rend une sauvegarde 0.15.0 inimportable dans une 0.14.x (`formatVersion`), ce que dit
  le manuel.
- **Réversible** : —

---

*Choix C-31-1-25 à C-31-1-36 : remédiation de la validation P1 (Sonnet, lentilles R et F), 2026-10-23. C-31-1-19
est amendé par C-31-1-32 (le trait `FreeSpace` devient `BackupEnv`) ; son entrée d'origine n'est pas réécrite.*

### C-31-1-25 — Les appels longs échappent au délai de 30 s du client, sans rejeu

- **Contexte** : `api-client.ts` abandonne à 30 s tout appel dont les en-têtes ne sont pas arrivés, et rejoue un `GET`
  jusqu'à quatre fois ; l'export n'envoie ses en-têtes qu'après avoir assemblé l'archive, l'import ne répond qu'après la
  restauration. Avec les pièces, ces durées se comptent en minutes (F1, HIGH).
- **Retenu** : options `{ timeoutMs, retry }` sur `getBlob` et `postFormData` ; `LONG_OPERATION_TIMEOUT_MS` = une heure
  pour l'export, l'import et « Sauvegarder maintenant », sans rejeu ; message d'issue inconnue sur `TIMEOUT` de l'import ;
  tests Vitest à horloge simulée qui figent les valeurs. Le correctif va dans la 31-1b (frontend déjà dans son périmètre ;
  aucune release entre la 31-1a et la 31-1b, C-31-0-1). Exemples de proxy inverse du manuel mis à jour (taille, durée).
- **Écartées** : téléchargement direct par lien (`<a href>`) — perd la lecture des en-têtes de l'AC18 et le message
  d'erreur, et ne règle pas l'import ; réponse asynchrone (202 + sondage d'un état) — une machine à états et une route de
  plus pour un geste rare, fait par un administrateur qui attend devant l'écran ; aucun délai — un serveur figé
  laisserait le bouton désactivé sans fin.
- **Réversible** : oui.

### C-31-1-26 — L'import et « Sauvegarder maintenant » détachés de la connexion

- **Contexte** : à l'abandon du client, hyper abandonne le futur du handler ; l'import pouvait s'arrêter au milieu de la
  restauration (F1, volet serveur).
- **Retenu** : après la réception, le travail s'exécute dans un `tokio::spawn` attendu par le handler ; la tâche va au
  bout. Garde lexicale (le test d'une déconnexion réelle demanderait un banc de connexion, hors de proportion).
- **Écartée** : laisser l'annulation au rollback de la transaction (vrai pour la base, mais l'audit et le journal se
  perdent, et le client reçoit une erreur pour une opération dont il ignore l'issue).
- **Réversible** : oui.

### C-31-1-27 — Une sauvegarde incomplète n'est pas un succès ; la dernière complète ne se supprime pas

- **Contexte** : un dossier des documents démonté rend chaque sauvegarde « réussie » sans aucune pièce ; la rétention
  aurait supprimé, en `retention_count` jours, toutes les sauvegardes qui les portaient (F2, HIGH).
- **Retenu** : statut `partial` (`filesNotIncluded` non vide), visible au panneau et au bandeau ; colonne
  `last_complete_file`, que la rétention ne supprime jamais ; la rétention tourne après `ok` et `partial`.
- **Écartées** : ne pas lancer la rétention après un `partial` — sur une installation dont des pièces manquent
  durablement (C-31-1-4), plus rien ne serait jamais supprimé et le disque se remplirait ; relire le manifeste de chaque
  archive pour trouver la plus complète — coûteux et inutile, l'état est déjà en base.
- **Réversible** : par migration (avant toute application persistante : librement, P8).

### C-31-1-28 — L'anomalie se juge par ligne, la clé est le chemin

- **Contexte** : la règle « même empreinte sous deux chemins = anomalie » écartait deux fichiers intacts (même contenu
  importé en `.jpg` et `.jpeg`, ou par deux sociétés) (F3, R15).
- **Retenu** : anomalie = préfixe ≠ empreinte de la même ligne, ou même chemin sous deux empreintes, ou chemin hors
  forme ; deux chemins de même empreinte sont deux fichiers.
- **Écartée** : dédoublonner par empreinte (un des deux chemins resterait sans fichier à la restauration).
- **Réversible** : oui.

### C-31-1-29 — Lectures bornées à l'import : 64 Mio pour le manifeste, 1 Gio par table, 1 000 000 d'entrées

- **Contexte** : `parse_and_verify` lit `manifest.json` et chaque table sans borne ; une entrée compressée de 1 Mio peut
  se décompresser en dizaines de Gio (F9).
- **Retenu** : constantes nommées, indépendantes des tailles déclarées par l'archive ; ensemble d'entrées attendu en
  format 2 ; noms en double refusés.
- **Écartées** : se fier à `entry.size()` (déclaratif) ; une borne totale proportionnelle au plafond d'import (la base
  d'une installation dépasse rarement la centaine de Mio : une borne fixe par table est plus lisible).
- **Réversible** : oui (constantes).

### C-31-1-30 — Fenêtre de détention des verrous

- **Contexte** : un verrou pris avant la réception d'un envoi de 2 Gio, ou gardé pendant la diffusion d'un export, rend
  toutes les sauvegardes automatiques dépendantes d'un client lent (F8).
- **Retenu** : import : `_kesh_version FOR UPDATE` et `backup_lock` pris après la réception et `parse_and_verify` ; export
  manuel : `backup_lock` pour l'assemblage seulement, rendu avant la diffusion ; chaque tour sauté journalisé en `info!`.
- **Réversible** : oui.

### C-31-1-31 — La compatibilité du format 1 se prouve sur un fichier produit par la 0.14.0

- **Contexte** : une fixture construite par le constructeur de test que la story modifie ne révèle pas un écart avec
  ce que le vrai exporteur 0.14.0 a émis (R7).
- **Retenu** : fixture produite par le vrai exporteur au commit `44ef2079`, avant tout changement de code, versionnée,
  jamais régénérée ; générateur `#[ignore]` ; données synthétiques.
- **Écartées** : une sauvegarde réelle de Guy (données réelles, interdites) ; un fichier produit à partir du tag
  `v0.14.0` dans un worktree séparé (même code que `44ef2079` pour l'export, plus lourd).
- **Réversible** : —

### C-31-1-32 — `disk_full` se décide sur l'erreur d'OS, jamais sur un texte

- **Contexte** : `write_backup_file` aplatissait toute erreur en chaîne dans `AdminPreImportBackupFailed` (F6, R6).
- **Retenu** : erreur typée qui garde l'`io::Error` ; `classify_write_error` pure (`StorageFull`, `QuotaExceeded`, codes
  28 et 122 → `disk_full`) ; trait `BackupEnv` (espace libre + écrivain injectable) qui remplace `FreeSpace` (C-31-1-19)
  pour simuler un disque saturé pendant l'écriture.
- **Écartées** : analyser le message ; une limite `RLIMIT_FSIZE` ou un tmpfs minuscule dans le test (dépend de
  l'environnement de gate, et `RLIMIT_FSIZE` rend `EFBIG`, pas `ENOSPC`).
- **Réversible** : oui.

### C-31-1-33 — L'envoi reçu dans un temporaire anonyme

- **Contexte** : `NamedTempFile` laisse un fichier de plusieurs Gio après un `kill -9` ou un OOM (F18).
- **Retenu** : `tempfile::tempfile_in(KESH_ADMIN_BACKUP_DIR)` : sans nom, il disparaît avec le processus.
- **Écartée** : nettoyer au démarrage (un motif de nom à maintenir, et un fichier qui survit jusqu'au redémarrage).
- **Réversible** : oui.

### C-31-1-34 — Le bandeau « aucune réussite » ne s'allume qu'après une première réussite

- **Contexte** : « `last_success_at` nul » allumait le bandeau à chaque installation et pendant les cinq minutes qui
  suivent chaque mise à jour (F17).
- **Retenu** : le second cas de l'AC31 exige un `last_success_at` non nul ; une sauvegarde jamais tentée se lit au
  panneau ; une tentée et échouée relève du premier cas. Angle mort écrit : une tâche morte dès le démarrage.
- **Écartées** : délai de grâce depuis une date d'installation (colonne ou horloge de la base, fuseau de session) ;
  heure de démarrage du processus dans `AppState` (un redémarrage effacerait le signal).
- **Réversible** : oui.

### C-31-1-35 — Seules les sauvegardes de Kesh se listent et se téléchargent

- **Contexte** : `kind = other` n'avait pas d'expression et aurait servi n'importe quel fichier du dossier, y compris un
  lien symbolique (F15).
- **Retenu** : deux sortes, `auto` et `pre-import`, chacune par son expression ; fichiers réguliers seulement
  (`symlink_metadata`).
- **Écartée** : lister les autres sans téléchargement (de l'information sans geste, et un nom de fichier arbitraire à
  afficher).
- **Réversible** : oui.

### C-31-1-36 — `scripts/regen-test-schema.sh` compté avec `kesh-db`

- **Contexte** : la réinjection de la ligne singleton dans le squash (F7, R1) touche un script ; la 31-1c compte déjà
  cinq modules.
- **Retenu** : le script produit `crates/kesh-db/test-schema/` et sa garde vit dans `crates/kesh-db/tests/` : il fait
  partie du module `kesh-db` au sens de la règle de découpage (« crates Rust, packages npm, ou modules métier de premier
  niveau »). La 31-1c reste à cinq ; pas de redécoupage. Signalé au rapport pour arbitrage de l'orchestrateur.
- **Écartées** : monter tous les tests de la 31-1c sur le vrai `MIGRATOR` (le squash garderait une table vide, piège pour
  tout test futur) ; sortir la migration dans une 31-1d (elle n'est pas testable seule sans le dépôt et la tâche).
- **Réversible** : oui, avant le développement.

---

*Choix C-31-1-37 à C-31-1-44 : remédiation de la validation P2 (Opus, lentilles R et F), 2026-10-23. Ils amendent
C-31-1-11 (C-31-1-39), C-31-1-20 et C-31-1-27 (C-31-1-40), C-31-1-29 (C-31-1-38), C-31-1-23 et C-31-1-30
(C-31-1-41), C-31-1-19 (C-31-1-42), C-31-1-5 (C-31-1-43) ; les entrées d'origine ne sont pas réécrites.*

### C-31-1-37 — Les chemins de `filesNotIncluded` ne sont soumis à aucune expression

- **Contexte** : la P1 avait exigé que les chemins de `filesNotIncluded` satisfassent l'expression des noms, alors que
  l'export y écrit le `storage_path` hors forme d'une ligne en anomalie : une sauvegarde authentique devenait
  inimportable par la même version (F1 HIGH, R2).
- **Retenu** : retrait de la règle ; ces chemins ne servent jamais de chemin, l'import les compare en chaînes (complétude
  de l'AC7) et les compte dans `filesMissing` ; journalisés en `Debug` seulement ; doublons entre listes toujours
  refusés. Test d'aller-retour (ligne `"../x.pdf"` → export → import 200).
- **Écartées** : omettre ces chemins du manifeste et ne compter comme « référencé » qu'un chemin de forme valide (deux
  règles au lieu d'une, et un compte du manifeste qui diverge de l'en-tête) ; neutraliser le chemin à l'export (une
  transformation à documenter et à inverser).
- **Réversible** : oui.

### C-31-1-38 — Les bornes de l'AC6b se réduisent à ce que `zip` 2.4.2 laisse contrôler

- **Contexte** : `zip` 2.4.2 fusionne les noms en double à l'ouverture (`read.rs:72-75`) et décode tout le répertoire
  central dans `ZipArchive::new` (`read.rs:706-710`) ; « nom en double → 400 » était inobservable et intestable,
  `MAX_ENTRIES` arrivait après le coût qu'il devait prévenir (R1, F4).
- **Retenu** : manifeste lu d'abord (`by_name`, borné), puis contrôle de l'ensemble des noms attendus pour le format 2
  avant toute autre lecture ; tables bornées ; fichiers bornés par `byteSize + 1` et restaurés **en flux** (la
  signature `restore_document(…, bytes, …)` lisait le fichier entier en mémoire). Retirés : refus des doublons
  (inoffensifs : seule l'entrée retenue est lue, et vérifiée par son empreinte) et `MAX_ENTRIES` ; le coût de
  l'ouverture, borné par la taille reçue, est écrit en angle mort.
- **Écartées** : lire l'EOCD et l'EOCD64 soi-même pour compter les entrées avant l'ouverture (un analyseur ZIP partiel à
  maintenir, pour un import réservé à l'Administrateur) ; forger des octets bruts pour tester un doublon que le crate
  rend invisible.
- **Réversible** : oui.

### C-31-1-39 — « Se reconnecter » après tout import réussi

- **Contexte** : C-31-1-11 n'attendait le clic que si des pièces manquaient ; sinon la redirection est immédiate
  (`AdminRestorePanel.svelte:119-128`), et l'avis « N pièces restaurées » de l'AC19 n'était ni lisible ni observable
  par l'E2E (R7).
- **Retenu** : le panneau attend toujours le clic *Se reconnecter* après un succès ; l'avis se lit, l'E2E l'observe
  avant de cliquer.
- **Écartée** : un délai fixe avant redirection (une valeur à régler, une course dans l'E2E).
- **Réversible** : oui.

### C-31-1-40 — `partial` reste au panneau ; le bandeau de l'application ne s'allume que si la sauvegarde est activée

- **Contexte** : un `partial` durable (C-31-1-4, le cas même de #503) allumait le bandeau à vie, sans geste pour
  l'éteindre, et noyait le signal d'échec ; un échec resté en base après désactivation l'allumait aussi à vie ; le
  rôle ARIA de `partial` était double (F7, R8, F15) ; un `role="alert"` au layout ferait rougir
  `reports.spec.ts:242` (L5).
- **Retenu** : bandeau de l'application pour les échecs et pour « aucune réussite depuis deux intervalles » seulement,
  et seulement si la sauvegarde est activée, en `role="status"` ; `partial` en avertissement (`role="status"`) au
  panneau. Le dossier des documents démonté se voit déjà partout (« document indisponible »), et la rétention garde la
  dernière complète (C-31-1-27).
- **Écartées** : allumer le bandeau quand le nombre de pièces non incluses augmente (une colonne de plus, et un signal
  qui s'éteint au tour suivant alors que le dossier reste démonté) ; un compte « reconnu » réglable au panneau (une
  colonne, un champ de route et un geste de plus pour un cas que l'application montre déjà).
- **Réversible** : oui.

### C-31-1-41 — Le garde de `backup_lock` est possédé et vit dans la tâche ; `backup_lock` avant `FOR UPDATE`

- **Contexte** : l'AC27 disait `lock().await` pour « Sauvegarder maintenant », que quatre autres AC veulent en 409
  (F2, R4) ; et un garde emprunté pris dans le handler est relâché à l'abandon du client alors que la tâche détachée
  continue (F3).
- **Retenu** : sauvegarde automatique et « Sauvegarder maintenant » en `try_lock` (tour sauté / 409) ; export manuel et
  import en attente ; gardes **possédés** (`lock_owned`, `try_lock_owned`) ; l'import prend le sien dans la fermeture du
  `spawn`, `POST …/run` prend le sien dans le handler et le déplace dans la tâche ; garde lexicale qui interdit les
  formes empruntées ; ordre écrit : `backup_lock` puis `_kesh_version FOR UPDATE`.
- **Écartée** : prendre tout verrou dans la tâche, y compris pour `POST …/run` (le 409 demanderait de lancer la tâche
  pour apprendre qu'elle ne peut pas tourner).
- **Réversible** : oui.

### C-31-1-42 — Le seuil d'espace s'estime aussi depuis le dossier des documents

- **Contexte** : le seuil `last_success_bytes × 1,2 + 256 Mio` ne savait rien de la première sauvegarde (activée par
  défaut, sans historique) ni du remontage d'un dossier des documents après une suite de `partial` (F6).
- **Retenu** : `max(last_success_bytes, documents_bytes) × 1,2 + 256 Mio`, où `documents_bytes` est la somme des tailles
  des fichiers réguliers du premier niveau de `KESH_DOCUMENTS_DIR` — un majorant connu sans lire la base.
- **Écartées** : estimer depuis l'instantané (la base serait lue deux fois, ou l'instantané ouvert avant de savoir s'il
  y a la place) ; les colonnes de taille (elles n'existent pas pour toutes les pièces).
- **Réversible** : oui.

### C-31-1-43 — Le débordement dans le dossier de sauvegarde se prouve par son effet

- **Contexte** : les temporaires de `tempfile` sont anonymes (`O_TMPFILE` ou créé-puis-délié) ; aucun `read_dir` ne les
  voit, si bien que « observer le dossier » ou « vérifier `/tmp` » passait sous la mutation ; et la `Config` de test met
  `admin_backup_dir` à `/tmp`, le même dossier que `temp_dir()` (R3, F5).
- **Retenu** : dossier de sauvegarde de test distinct, rendu non inscriptible ; seuil 0 → l'export échoue, seuil élevé →
  200 ; le test échoue bruyamment s'il peut écrire dans le dossier (gate en `root`). « Aucun temporaire laissé » est
  déclaré vrai par construction, pas compté comme test.
- **Écartées** : lire `/proc/self/fd` pendant l'assemblage (un observateur à synchroniser avec l'écriture) ; injecter le
  constructeur du temporaire (une abstraction pour un test).
- **Réversible** : oui.

### C-31-1-44 — Noms des sauvegardes automatiques à la milliseconde

- **Contexte** : à la seconde, deux sauvegardes rapprochées se heurtaient au refus d'écraser de `write_backup_file`
  (L3, F9).
- **Retenu** : `kesh-auto-<AAAAMMJJ>T<HHMMSSmmm>Z.keshbackup`, dérivé du `now` passé à `run_once` ; sous `backup_lock`,
  les sauvegardes sont successives et durent chacune plus d'une milliseconde ; une collision résiduelle vaut `failed`.
- **Écartée** : pid et compteur comme la sauvegarde pré-import (un motif plus long, à faire tenir dans l'expression de
  rétention, pour un cas que le verrou rend déjà presque impossible).
- **Réversible** : oui, avant la première sauvegarde automatique écrite (le motif de rétention en dépend).
