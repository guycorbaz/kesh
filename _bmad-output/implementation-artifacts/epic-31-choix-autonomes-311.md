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

*(Marque ajoutée le 2026-10-24 au développement de la 31-1a, LOW R10 de la validation P5 : la réservation en base est
retirée par C-31-1-54 ; l'entrée n'est pas réécrite.)*

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

### C-31-1-45 — `run_once` prend lui-même le verrou ; `POST …/run` traduit `Skipped` en 409 (amende C-31-1-41)

*(Marque ajoutée le 2026-10-24, LOW R10 de la P5 : la « réservation perdue » est retirée par C-31-1-54.)*

- **Contexte** : validation P3 (R1 = F1, nés de la remédiation P2) — l'AC35 faisait prendre `backup_lock` par `run_once`,
  l'AC27 et l'AC32 par le handler de `POST …/run`, qui déplaçait ensuite le garde dans la tâche appelant `run_once` ;
  `tokio::sync::Mutex` n'est pas réentrant : l'appel manuel se serait toujours trouvé « en cours » (ou bloqué).
- **Retenu** : une seule règle par acquéreur. `run_once` prend le verrou en `try_lock_owned` quel que soit le
  déclencheur et rend `RunOutcome::Skipped` s'il est pris (ou si la réservation en base est perdue) ; la boucle de fond
  ignore `Skipped` ; le handler de `POST …/run` n'acquiert rien, lance `run_once` dans un `tokio::spawn` attendu et
  traduit `Skipped` en 409 `BACKUP_IN_PROGRESS`. L'import et l'export manuel gardent `lock_owned().await` (import dans
  la fermeture de son `spawn`). M-c6 et M-c17 reformulées ; garde lexicale : `try_lock_owned` dans `auto_backup.rs`
  seulement.
- **Écartées** : passer le garde en paramètre de `run_once` (une seconde signature de fait, et la boucle comme le
  handler devraient le prendre) ; garder le verrou au handler (la contradiction elle-même). Le motif qui avait écarté
  cette option en P2 (C-31-1-41 : « lancer la tâche pour apprendre qu'elle ne peut pas tourner ») est un coût
  négligeable — un `spawn` qui rend aussitôt.
- **Réversible** : oui.

### C-31-1-46 — L'assemblage d'une archive tient dans un seul `spawn_blocking` synchrone

*(Marque ajoutée le 2026-10-24, LOW R10 de la P5, T3 de la P6, L1 de la P7 : « deux passes » est amendé par C-31-1-53
— une seule passe — et la signature `Box<dyn Write + Seek + Send>` par les amendements datés P5 à P8 en fin de
registre — `Box<dyn WriteSeek>`, entrée `files/` toujours écrite.)*

- **Contexte** : validation P3 (R5 = F2, nés de la remédiation P2) — `wrap_writer<'a>(&'a mut File)` ne pouvait entrer
  dans un `spawn_blocking` (fermeture `'static`), `write_backup_file` était `async` sur `tokio::fs` alors que le
  `ZipWriter` est synchrone, et les copies de fichiers « en `spawn_blocking` » ne pouvaient pas écrire dans l'entrée.
- **Retenu** : lecture de la base en asynchrone (`export_installation`), puis tout le reste — tables, fichiers en deux
  passes, `finish` — dans une fonction synchrone `write_keshbackup<W: Write + Seek>` appelée dans **un seul**
  `spawn_blocking` qui possède l'export, les chemins, l'écrivain et, à la 31-1c, un `Arc<dyn BackupEnv>` ;
  `write_backup_file` devient synchrone, à fermeture de remplissage, et remplit un clone (`try_clone`) du `.partial`
  dont l'original reçoit `sync_all` ; `wrap_writer(File) -> Box<dyn Write + Seek + Send>` prend ce clone par valeur ;
  aucun `tokio::fs` mêlé au `ZipWriter`, aucun `block_in_place`.
- **Écartées** : `block_in_place` (panique sous le runtime mono-fil des tests) ; un pont `SyncIoBridge` (un `Write`
  synchrone sur un `AsyncWrite`, qui bloque quand même un fil) ; un `spawn_blocking` par fichier (l'écrivain ZIP ne peut
  pas y entrer).
- **Réversible** : oui, avant le développement.

### C-31-1-47 — Fichier illisible : listé `unreadable` ; divergence à la copie : la sauvegarde est abandonnée

*(Marque ajoutée le 2026-10-24, LOW R10 de la P5 : la seconde passe et l'abandon sur divergence sont amendés par
C-31-1-53 — une passe, `abort_file`, raison `altered` ou `unreadable`.)*

- **Contexte** : validation P3 (F3, d'origine) — `open_document` rend `Io(…)` pour un fichier présent mais illisible
  (droits restaurés par DSM, `EIO`) ; la fiche ne disait pas si toute la sauvegarde échouait, et ne disait pas que la
  copie, faite après le hachage, pouvait diverger et produire une archive que l'import refuse.
- **Retenu** : toute erreur d'I/O autre que `NotFound` à la passe de hachage → raison `unreadable` dans
  `filesNotIncluded`, la sauvegarde continue (cohérent avec C-31-1-4) ; la passe de copie re-hache, et toute divergence
  (empreinte, taille, I/O) **abandonne la sauvegarde entière** — jamais d'archive partielle ni d'entrée fausse. Test du
  fichier illisible (`chmod 000`, garde bruyante en `root`) et M-a20 ; le re-hachage est déclaré non testable (mutation
  équivalente au niveau des tests).
- **Écartées** : une seule passe (copie et hachage ensemble — un fichier altéré, déjà écrit, ne pourrait plus être
  exclu : `zip` ne retire pas une entrée) ; lister le fichier divergent au lieu d'abandonner (l'entrée est déjà
  commencée dans l'archive).
- **Réversible** : oui.

### C-31-1-48 — L'inventaire des sujets de l'AC14 est l'ensemble des lignes d'une commande

- **Contexte** : validation P3 (F4, résidu de la P2) — l'extension de l'inventaire par sujets avait été rejouée sur
  les `.tex` et `config.rs`, pas sur `.env.example`, `README.md` ni les doc-comments : huit sites manquaient, dans une
  liste qui se disait close.
- **Retenu** : la commande `grep` écrite à l'AC14 (motif tolérant `\_` du LaTeX, commentaires seulement dans `crates/`)
  définit l'inventaire : 94 lignes à `c7639e80`, chacune au tableau ou triée « intacte » au Dev Agent Record ; rejouée
  au dev sur le code modifié. Les huit sites de F4 et ceux de R11 (`admin-manual.tex:1721`, `:1739`, `:1887`, `:1958`,
  `:2194`, `:2199`) sont au tableau.
- **Écartées** : compléter la liste à la main (le défaut même) ; un test automatique (des phrases, non du code).
- **Réversible** : oui.

### C-31-1-49 — Bornes de l'import : cumulées pour les tables, paramétrables pour les tests

- **Contexte** : validation P3 (F5, F6) — une borne de 1 Gio par table laissait 39 Gio en mémoire ; un test « table
  trop grande » aurait exigé une entrée de plus de 1 Gio ; `byteSize + 1` paniquait sur `u64::MAX` en débogage.
- **Retenu** : `TABLES_MAX_BYTES` (1 Gio) cumulé sur toutes les `data/*.ndjson` ; `MANIFEST_MAX_BYTES` inchangé ; les
  deux passés à `parse_and_verify` par `ReadLimits` (constantes en production, réduites en test) ; `byteSize` supérieur
  au plafond d'import refusé en 400 à la lecture du manifeste, `take(byteSize.saturating_add(1))` partout.
- **Écartées** : abaisser la borne par table (39 × une borne plus petite reste un produit) ; borner par la taille
  déclarée de l'en-tête ZIP (déclarative).
- **Réversible** : oui.

### C-31-1-50 — `ensure_not_pat` sur le seul téléchargement d'une sauvegarde du serveur

- **Contexte** : validation P3 (R12) — la P2 mettait `ensure_not_pat` dans chacun des cinq handlers neufs, alors que la
  couche anti-clé d'`admin_routes` les refuse déjà (les `users` et `email-templates` s'en passent) ; les trois `GET`
  sortaient alors de la preuve de `head_couples_are_denied_too`.
- **Retenu** : la couche suffit pour quatre routes ; seul `GET backups/{kind}/{name}`, qui livre une archive pleine de
  secrets comme `full-export`, porte en plus `ensure_not_pat` (comme `full_export` et `full_import`) et entre dans
  `GUARDED_BY_HANDLER` ; chemins discriminants 4 → 6.
- **Écartées** : la garde partout (la preuve s'appauvrit sans gain) ; nulle part (le précédent de `full_export`
  pour une route qui livre une archive).
- **Réversible** : oui.

### C-31-1-51 — Sauvegardes du serveur : pas d'indicateur de taille d'import, pas de suppression depuis l'écran

- **Contexte** : validation P3 (F12, F13) — une sauvegarde du serveur peut dépasser le plafond d'import sans que la liste
  le dise ; après `insufficient_space`, baisser le nombre conservé ne libère rien tant qu'aucune sauvegarde ne réussit.
- **Retenu** : F12 en angle mort écrit (fiche et manuel : relever le plafond quand le dossier des documents grossit) ;
  F13 par le texte — le message du statut `insufficient_space` au panneau et le manuel donnent la marche à suivre
  (supprimer d'anciennes sauvegardes de `./backup/auto`) ; « un échec ne supprime rien » est conservé.
- **Écartées** : un booléen « dépasse la taille d'import » dans `GET backups` (fonctionnalité neuve, à demander en
  issue si l'usage le réclame) ; une rétention appliquée avant l'écriture (elle contredit M-c4 et M-c14 et supprime sur
  un échec) ; un bouton « Supprimer » dans la liste (destructif, hors périmètre de #503).
- **Réversible** : oui.

### C-31-1-52 — F12 intégré, pas d'issue (choix de l'orchestrateur, 2026-10-24 ; amende C-31-1-51)

- **Contexte** : la remédiation P3 laissait en angle mort (C-31-1-51) qu'aucun signal ne prévienne qu'une sauvegarde du
  serveur dépasse la taille d'import (`KESH_ADMIN_IMPORT_MAX_MB`), et proposait une issue. Le compte d'issues ouvertes est
  à 139 ; le plafond au tag v0.15.0 est « moins de 136 », que les quatre fermetures prévues (#503, #577, #697, #698)
  atteignent tout juste (135).
- **Retenu** : intégrer à la 31-1c — dans la liste des sauvegardes du serveur, une sauvegarde dont la taille dépasse la
  taille d'import porte une mention « trop grande pour être restaurée par l'écran (limite N Mio) » ; un test Vitest et une
  mutation ; le manuel le dit. Budget d'issues comme motif d'intégration, accepté par Guy le 2026-10-23 (`CLAUDE.md`
  § *Priorités des défauts*).
- **Écartée** : ouvrir une issue `enhancement` — le point est mécanique et tient dans l'écran que la 31-1c construit.
- **Réversible** : oui.

### C-31-1-53 — Une seule passe par fichier, `abort_file` (amende C-31-1-47)

- **Contexte** : validation P4 (F2, né de la remédiation P3) — C-31-1-47 écartait la passe unique au motif que « `zip`
  ne retire pas une entrée » ; `zip` 2.4.2 expose `ZipWriter::abort_file` (`write.rs:1134`), s'en sert lui-même
  (`ok_or_abort_file`, `:889`) et réécrit le pied à la finalisation si des octets abandonnés le dépassent
  (`:1501-1524`). La prémisse fausse coûtait une double lecture de chaque pièce, un abandon de toute la sauvegarde sur
  une erreur de lecture survenue en seconde passe, et une mutation déclarée équivalente.
- **Retenu** : une passe — ouverture (`missing` / `unreadable` sans entrée commencée), `start_file`, boucle de copie
  écrite à la main (lecture à travers `take(byteSize + 1)`, `Sha256`, écriture), puis empreinte ou taille différente →
  `abort_file` et `altered`, erreur de lecture → `abort_file` et `unreadable` ; seule une erreur d'écriture de l'archive
  abandonne la sauvegarde. Tests : dossier à la place du fichier suivi d'un sain (M-a21), altéré suivi d'un sain avec
  aller-retour (établit au dev la relecture d'une archive à entrée abandonnée). Aucune mutation équivalente.
- **Écartées** : garder deux passes « par choix » (le coût de lecture et les deux traitements d'une même cause
  restaient) ; `io::copy` (confond l'erreur de lecture et l'erreur d'écriture).
- **Réversible** : oui, avant le développement — si le test d'aller-retour montrait qu'une archive à entrée abandonnée
  ne se relit pas, revenir à C-31-1-47.

### C-31-1-54 — Pas de réservation en base ; une instance par base (amende C-31-1-23 et C-31-1-45)

- **Contexte** : validation P4 (F5, d'origine) — la réservation `WHERE last_attempt_at <=> ?` portait sur la valeur
  lue par `run_once` lui-même après son verrou, sans réévaluer `is_due` : un second processus lisant après la
  réservation du premier réussissait la sienne. Elle ne fermait que quelques millisecondes, ne protégeait pas
  `Trigger::Manual`, et le nettoyage des `.partial` restait capable de supprimer celui de l'autre processus ; dans un
  seul processus, le verrou suffit.
- **Retenu** : retirer la réservation (AC27), `Skipped` sur réservation perdue, le test « réservation perdue » et M-c7 ;
  `record_attempt` enregistre la tentative sans condition ; l'angle mort dit qu'aucune protection n'existe entre
  processus, et le manuel qu'il faut une instance par base.
- **Écartées** : faire réévaluer `is_due` à `run_once` sur la valeur réservée (protège la seule boucle planifiée, pas
  `Manual` ni les `.partial`, pour un déploiement que Kesh ne prévoit pas) ; un verrou consultatif MariaDB
  (`GET_LOCK`) — une connexion tenue pendant toute la sauvegarde, pour le même déploiement non prévu.
- **Réversible** : oui.

### C-31-1-55 — Le garde de l'export manuel vit dans le `spawn_blocking` d'assemblage

- **Contexte** : validation P4 (F1) — la P3 laissait le garde au handler de `full_export` : un client parti (proxy au
  délai plus court) relâchait le verrou pendant que l'assemblage continuait, et chaque nouvel essai en lançait un autre,
  N assemblages débordant ensemble dans `KESH_ADMIN_BACKUP_DIR`, le volume de `auto/` et souvent de la base.
- **Retenu** : `lock_owned().await` dans le handler, garde déplacé dans la fermeture `remplir` passée à
  `build_keshbackup` ; rendu à la fin de l'assemblage, avant la diffusion. Garde lexicale (le nom du garde dans la
  fermeture de `full_export`) ; M-c24. L'angle mort *(P3)* disparaît.
- **Écartées** : détacher l'export dans un `tokio::spawn` comme l'import (même effet, une tâche de plus) ; garder
  l'angle mort.
- **Réversible** : oui.

### C-31-1-56 — `build_keshbackup` reste l'unique lecteur de la base, générique sur le remplissage

- **Contexte** : validation P4 (R7) — l'AC4 de la P3 avait trois appelants au contenu de `spawn_blocking` différent
  et disait seulement « corps renommé » : la garde `build_keshbackup_ne_lit_la_base_qu_une_fois` pouvait rougir sur son
  montage ou veiller sur un `build_keshbackup` devenu code mort.
- **Retenu** : `build_keshbackup(pool, documents_dir, remplir)` asynchrone appelle `export_installation(pool)` puis
  `spawn_blocking(move || remplir(export, documents_dir))` ; chaque appelant passe sa fermeture (qui appelle
  `write_keshbackup` sur son écrivain). La garde reste exacte sur son corps et gagne « `export_installation(` une seule
  fois dans la production de `kesh-api` ». La garde `avant_sauvegarde_branchee_aux_appels_de_l_import` garde ses motifs
  et son compte ; la fermeture se nomme avant l'appel pour ne pas couper l'instruction au premier `;`.
- **Écartées** : remplacer la garde par la seule signature de `write_keshbackup` (laisse chaque appelant libre de lire
  la base autrement) ; une garde lexicale « `export_installation(` une fois par appelant » (trois sites à tenir au lieu
  d'un).
- **Réversible** : oui.

### C-31-1-57 — Horloge revenue en arrière : `last_*` futur vaut nul ; le retard se calcule au serveur

- **Contexte** : validation P4 (F4, d'origine) — un `last_success_at` ou `last_attempt_at` postérieur à `now` rendait
  `is_due` faux et le bandeau éteint jusqu'à ce que l'horloge rattrape la valeur : la sauvegarde s'arrêtait en silence,
  la panne même que #503 veut rendre visible. Calculé dans le navigateur, le second cas du bandeau comparerait en outre
  la date du serveur à l'horloge du navigateur.
- **Retenu** : `is_due` tient une valeur future pour nulle (`warn!`) ; fonction pure `is_overdue` (activée, réussite
  non nulle, plus vieille que deux intervalles **ou** future), exposée par `GET backup-schedule` (`overdue`) et lue
  telle quelle par le bandeau ; `nextDueAt` vaut `now` si une valeur est future. Tests aux bornes, M-c22 ; M-c9 reciblée
  sur `is_overdue`.
- **Écartées** : une tolérance dans le navigateur (un seuil arbitraire, et l'horloge du navigateur reste la référence) ;
  ignorer le cas (panne silencieuse).
- **Réversible** : oui.

### C-31-1-58 — `importMaxMib` une fois par réponse de `GET backups`

- **Contexte** : validation P4 (R1) — C-31-1-52 retient une mention « trop grande pour être restaurée par l'écran
  (limite N Mio) » dans la liste, mais aucune route ne transmet le plafond d'import au frontend.
- **Retenu** : `GET backups` rend `{ importMaxMib, items }` ; le panneau compare chaque taille à `importMaxMib ×
  1 048 576` (strictement supérieure → mention). Test Rust (valeur égale à la `Config`), Vitest (au-dessus, égal,
  au-dessous), M-c23, capture (r), clé `admin-backup-server-too-large` ×4, phrase du manuel.
- **Écartées** : un booléen par ligne (la même information répétée, et la limite N ne s'afficherait pas) ; une route
  de configuration générale (hors périmètre).
- **Réversible** : oui.

**Amendement de C-31-1-46 (2026-10-24, validation P5, R1)** : `Box<dyn Write + Seek + Send>` n'est pas un type valide
(E0225) ; l'écrivain s'écrit `Box<dyn WriteSeek>`, avec `trait WriteSeek: Write + Seek + Send {}` et son implémentation
générale. **Et (R2)** : en format 2, aucune entrée de répertoire `files/` n'est écrite, pour que `abort_file` puisse toujours
rembobiner.

**Amendement (2026-10-24, validation P6)** : la seconde moitié de l'amendement précédent (« aucune entrée de répertoire
`files/` en format 2 ») est **retirée** — sa prémisse était fausse (`add_directory` pose `data_start`, `zip-2.4.2`
`write.rs:1040-1042`, `:1422`). L'entrée `files/` reste écrite. `WriteSeek` est `pub(crate)`.

**Amendement (2026-10-24, validation P7)** : `WriteSeek` est **`pub`**, non `pub(crate)` — les tests de l'AC36 sont dans un
crate externe ; le module `auto_backup` expose en `pub` `BackupEnv`, `RealBackupEnv`, `WriteSeek`, `run_once`, `RunOutcome`,
`RunTrigger`, `spawn`. **État final de C-31-1-46** : `Box<dyn WriteSeek>` ; l'entrée `files/` reste écrite.

**Amendement (2026-10-24, validation P8)** : `RunTrigger` se lit `Trigger` ; la liste `pub` du module `auto_backup` comprend
aussi `FailureStatus`, le statut de succès et le type d'erreur de `run_once` ; l'importabilité de l'AC36 se prouve par la
route `POST …/full-import` (`admin_backup` reste `pub(crate)`).


---

## Développement de la 31-1a (2026-10-24, Opus 5.5)

### C-31-1-59 — La fixture 0.14.0 est produite au tag `v0.14.0`, non à `44ef2079`

- **Contexte** : l'AC10 et T-a0 nomment `44ef2079` (base de la branche à la création de la fiche) ; l'orchestrateur
  prescrit le tag `v0.14.0` (`5a128aec`), dans un worktree temporaire, sous la chaîne que porte ce tag. La branche a
  depuis été reportée sur `b5fd9fb1` (Rust 1.97.1, Story 31-3).
- **Retenu** : worktree temporaire `git worktree add … v0.14.0`, générateur `generate_fixture_0_14_0` (`#[ignore]`)
  copié dans ce worktree, exécuté sous Rust 1.96.1 (`rust-toolchain.toml` du tag) avec la cible cargo du worktree de la
  story, puis worktree retiré. `git diff --stat 5a128aec 44ef2079 -- crates Cargo.toml Cargo.lock rust-toolchain.toml`
  est **vide** : le code est le même aux deux commits ; le tag est la référence publiée, donc la plus forte.
- **Écartée** : produire la fixture sur la branche avant tout changement (chaîne 1.97.1 et `textes_coherents.rs` de la
  31-3 : le code de l'exporteur est identique, mais la fixture ne sortirait plus d'une version publiée).
- **Réversible** : oui (la fixture se régénère au tag par le même générateur).

### C-31-1-60 — Type d'erreur de `build_keshbackup` : `BackupError { Db, Write, Task }` (R4 = F3 de la P5)

- **Contexte** : `build_keshbackup` lit la base (`DbError`), rend l'erreur de `remplir` (`BackupWriteError`, dont la 31-1c
  classera l'`io::Error`) et peut perdre sa tâche (`JoinError`) ; l'import doit tout convertir par `avant_sauvegarde`.
- **Retenu** : `pub enum BackupError { Db(DbError), Write(BackupWriteError), Task(String) }`, `impl From<BackupError> for
  AppError` (→ `AdminFullExportFailed`, détail au journal) ; `avant_sauvegarde` devient `fn(impl Into<AppError>)`, si bien
  que l'appel de la sauvegarde pré-import garde la forme `….await.map_err(avant_sauvegarde)?` que les gardes lexicales
  exigent ; l'export écrit `.map_err(AppError::from)`.
- **Écartées** : rendre `AppError` depuis `build_keshbackup` (perdrait l'`io::Error` dont la 31-1c a besoin) ; un
  `Box<dyn Error>` (le `kind()` se perdrait derrière un `downcast`).
- **Réversible** : oui.

### C-31-1-61 — L'échec d'écriture des pièces réutilise `AdminFullImportFailed` et son message

- **Contexte** : l'AC8 demande un refus 500 « aucune donnée n'a été remplacée » quand les pièces ne s'écrivent pas.
- **Retenu** : `AppError::AdminFullImportFailed` (code `ADMIN_FULL_IMPORT_FAILED`), dont le message existant dit déjà
  « L'état précédent a été préservé ; une sauvegarde de sécurité a été écrite avant l'opération » — exact ici : la
  sauvegarde de sécurité est écrite, la base n'est pas touchée. Aucune clé i18n neuve (la 31-1a n'a pas d'i18n).
- **Écartée** : une variante neuve (code et quatre locales pour un message équivalent).
- **Réversible** : oui (la 31-1b peut distinguer le cas à l'écran si elle le juge utile).

### C-31-1-62 — Classement des pièces en Rust, côté `kesh-api` ; `kesh-db` rend les couples bruts

- **Contexte** : l'AC1 dédoublonne par chemin et classe trois anomalies, dont la forme `^[0-9a-f]{64}\.[a-z0-9]{1,8}$`
  et `is_safe_storage_path`, qui vivent dans `kesh-api` (`document_storage`).
- **Retenu** : `kesh_db::backup::read_referenced_files` rend les couples (chemin, empreinte) **distincts** (égalité
  d'octets, `BTreeSet`), triés, lus dans l'instantané ; `export::classer_les_pieces` (fonction pure, testée seule) les
  partage en « à lire » et « incohérents ». Un même chemin sous deux empreintes apparaît donc deux fois dans
  `referenced_files`, et une seule fois, en `mismatch`, au manifeste.
- **Écartée** : classer dans `kesh-db` (dépendance aux règles de nommage de `kesh-api`, ou duplication de l'expression).
- **Réversible** : oui.

### C-31-1-63 — Taille d'une pièce contrôlée deux fois à l'import ; M-a6 (borne seule) déclarée équivalente

- **Contexte** : M-a6 retire la borne `take(byteSize + 1)`. L'import compare **aussi** le nombre d'octets lus à
  `byteSize` (plus long → « trop grande », plus court → « taille différente ») : sans la borne, l'entrée est lue en
  entier, puis refusée par la comparaison, avec le même message.
- **Retenu** : garder les deux. La borne protège la mémoire et le temps (une entrée qui déclare 10 octets et en porte
  plusieurs Gio n'est lue qu'au-delà de 11 octets) ; la comparaison refuse une entrée plus courte. M-a6 (borne seule)
  est **équivalente** au niveau de l'observable (aucun test ne mesure la quantité lue) ; la variante M-a6b (borne **et**
  comparaison retirées) est tuée par le test « entrée plus grande que déclarée, empreinte du contenu entier ».
- **Écartée** : retirer la comparaison pour que la seule borne refuse (une entrée plus courte que déclarée passerait).
- **Réversible** : oui.

### C-31-1-64 — Un manifeste de format 1 qui déclare des pièces est refusé

- **Contexte** : `files` et `filesNotIncluded` sont `#[serde(default)]` ; un manifeste `formatVersion: 1` peut
  techniquement les porter.
- **Retenu** : 400 « un manifeste de format 1 ne déclare aucune pièce » — la 0.14.0 n'en écrivait pas, et l'import
  n'irait pas les lire (format 1 : `files/` doit être vide).
- **Écartée** : les ignorer (une archive incohérente s'importerait sans le dire).
- **Réversible** : oui.

### C-31-1-65 — `restore_document` distingue « écrite » et « remplacée »

- **Contexte** : l'AC8 journalise en `warn!` une pièce présente mais altérée, remplacée par celle de la sauvegarde.
- **Retenu** : `RestoreOutcome::{Written, Replaced, AlreadyIntact}` ; `Replaced` dès qu'un fichier de ce nom existait et
  n'avait pas l'empreinte attendue (ou n'était pas lisible) ; `ecrire_les_pieces` journalise `Replaced` en `warn!`.
- **Réversible** : oui.

### C-31-1-66 — `write_pre_import_backup` porte l'appel de `build_keshbackup` ; les deux gardes s'étendent

- **Contexte** : la sauvegarde pré-import crée le dossier, nomme le fichier et appelle `build_keshbackup` avec une
  fermeture `remplir` qui passe par `write_backup_file`. Laissé dans `run_backup_and_restore`, le `?` de la fermeture et
  sa conversion rompaient le décompte de la garde `echecs_avant_sauvegarde_tous_convertis`.
- **Retenu** : `write_pre_import_backup(state)` porte tout (dossier, nom, `remplir` nommée avant l'appel,
  `build_keshbackup(…).await.map_err(avant_sauvegarde)?`) ; la garde 20 compte `write_pre_import_backup` parmi les
  fonctions de l'import ; la garde des conversions porte désormais **deux** segments (début de
  `run_backup_and_restore` → appel de `write_pre_import_backup` : (3, 0) ; corps de `write_pre_import_backup` → fin de
  l'instruction `build_keshbackup` : (1, 1)), l'invariant « tout `?` d'avant la sauvegarde est converti » intact.
- **Écartée** : baisser un compte sans étendre le segment (la garde ne verrait plus la sauvegarde elle-même).
- **Réversible** : oui.

### C-31-1-67 — `TMPDIR` n'est plus lu : l'entrée du registre des lectures est retirée

- **Contexte** : `configuration_transmise.rs` (L) inventoriait `std::env::temp_dir()` de `stream_via_tempfile`, retiré
  par l'AC4 (débordement dans `KESH_ADMIN_BACKUP_DIR`) ; l'entrée périmée rougissait.
- **Retenu** : l'entrée et la constante `ADMIN` retirées, l'en-tête du test dit qu'un retour de `TMPDIR` rougirait.
- **Réversible** : oui.

### C-31-1-68 — Montage commun des tests de pièces dans `tests/support/pieces.rs`

- **Contexte** : les tests d'export et d'import (deux binaires) seedent tous deux une facture figée par la route réelle
  et un document écrit par `store_document`.
- **Retenu** : un module `tests/support/pieces.rs` inclus par `#[path]` dans les deux binaires (patron de
  `kesh-db/tests/support/`), aucun binaire de test neuf ; le générateur de la fixture garde son montage en ligne, tel
  qu'il a tourné au tag `v0.14.0`.
- **Réversible** : oui.

## Développement de la 31-1b (2026-10-24, Opus 5.5)

### C-31-1-69 — L'avis `.env` est un composant partagé, possédé par `admin-backup`

- **Contexte** : l'AC17 nomme **une** clé (`admin-backup-not-included-env`) pour deux panneaux de deux dossiers ; la
  règle `lint-i18n-ownership` refuse qu'un fichier de `features/admin-restore/` appelle une clé `admin-backup-*`.
- **Retenu** : `features/admin-backup/BackupEnvNotice.svelte` porte la clé et le texte ; `AdminRestorePanel` importe le
  composant (`data-testid="backup-env-notice"` sur les deux écrans). Une clé, un repli, un texte.
- **Écartées** : une seconde clé `admin-restore-*` au même texte (deux replis à tenir égaux) ; une entrée
  `KNOWN_VIOLATIONS` (dette #30 étendue pour rien).
- **Réversible** : oui.

### C-31-1-70 — Les avis chiffrés sont écrits neutres au nombre

- **Contexte** : `i18nMsg` ne fait qu'une substitution de `{ $var }` (aucun sélecteur Fluent côté client) ; les textes
  de la fiche (« N pièces référencées n'ont pas pu être incluses », « N pièces référencées sont introuvables… »,
  « 2 pièces restaurées ») donneraient « 1 pièces ».
- **Retenu** : le nombre après deux-points — « Pièces référencées que le fichier n'a pas pu inclure : N (…) — voir le
  journal du serveur. Après une restauration, elles répondront « document indisponible ». », « Pièces justificatives
  restaurées : N. », « Pièces référencées introuvables après la restauration : N. … ». Le sens et les éléments de la
  fiche sont gardés ; l'E2E lit le nombre dans l'élément dédié (`admin-restore-files-restored`).
- **Écartées** : « pièce(s) » (accords impossibles du verbe) ; deux clés singulier / pluriel (logique de choix dans
  chaque écran).
- **Réversible** : oui.

### C-31-1-71 — « Se reconnecter » aussi après une issue inconnue ; formulaire figé après un succès

- **Contexte** : après une réponse d'import perdue (AC18b), le message dit « Reconnectez-vous pour vérifier » ; après un
  import réussi, la session est invalidée et l'installation remplacée.
- **Retenu** : le bouton « Se reconnecter » est proposé dans les deux cas ; après un succès, le choix du fichier et
  « Importer » sont désactivés (un second import ne se lance pas d'un écran dont la session est morte).
- **Écartée** : laisser l'utilisateur chercher seul la reconnexion (le menu utilisateur est absent de l'assistant).
- **Réversible** : oui.

### C-31-1-72 — Lien de l'assistant et page `/onboarding/restore`

- **Retenu** : le lien est placé sous le contenu de l'étape, avec une invite (« Vous avez une sauvegarde de Kesh
  (.keshbackup) ? ») ; il exige un état d'onboarding chargé (`onboardingState.loaded`), une étape ≤ 2 et le rôle
  `Admin`. La page `/onboarding/restore` charge les messages i18n au montage (elle peut s'ouvrir avant le choix de la
  langue) et porte un lien « ← Retour à l'assistant ». Sa garde de rôle renvoie un non-Admin vers **`/onboarding`**, et
  non vers `/` comme `(app)/admin/restore/+page.ts` : `/` le renverrait de toute façon à l'assistant, en deux sauts.
- **Réversible** : oui.

### C-31-1-73 — En-têtes de l'export lus strictement

- **Retenu** : `headerCount` n'accepte qu'un entier décimal non signé (`^\d+$`, après `trim`) et sûr ; toute autre
  valeur (absente, vide, `abc`, `1e3`, `-1`, trop grande) vaut `null`, jamais `NaN`. Un avis ne s'affiche que pour une
  valeur > 0.
- **Réversible** : oui.

### C-31-1-74 — Les refus de l'import vont au manuel, pas à `docs/api-external.md`

- **Contexte** : la revue P1 de la 31-1a confie à la 31-1b les refus propres au format 2, les bornes de lecture, le 410
  d'une pièce non emportée et les angles morts E-3 / E-4 « au manuel et à `docs/api-external.md` ».
- **Retenu** : `docs/api-external.md` ne documente pas `full-export` / `full-import`, refusés aux clés d'API (route
  d'administration, `ensure_not_pat`) : seul le 410 des deux lectures de document y est corrigé (B-L3). Refus, bornes,
  E-3 et E-4 vont au manuel d'administration (liste des refus et deux limites).
- **Réversible** : oui.

### C-31-1-75 — E2E : pas de poignée de main de l'inbox ; trois parcours

- **Contexte** : `inboxHandshake` dépose une sonde puis l'**écarte** ; un document écarté reste une ligne référencée,
  que la sauvegarde emporte — le compte « 2 pièces restaurées » de l'AC20 deviendrait 3.
- **Retenu** : la spec ne fait pas de poignée de main ; la garde bruyante de suppression des pièces (chaque fichier
  existe avant, n'existe plus après) détecte le même défaut de montage. Deux parcours s'ajoutent à celui de l'AC20 :
  pièce absente avant l'export → avis d'export, puis avis de pièces manquantes à l'import (captures e, h) ; plafond
  d'import dépassé et import en 504 par interception réseau (captures f, i).
- **Réversible** : oui.

### C-31-1-76 — Les captures restent hors dépôt

- **Retenu** : `docs/manual/fr/admin-manual.tex` n'emploie aucune capture d'écran (`grep -c includegraphics` : 0) ;
  les neuf captures (a) à (i) restent dans `kesh-gate-logs/311b-captures/`, prises par l'E2E quand
  `KESH_E2E_CAPTURES_DIR` est posé.
- **Réversible** : oui.

### C-31-1-77 — Trois mutations hors fiche

- **Retenu** : M-b8 (garde de rôle de `/onboarding/restore` neutralisée), M-b9 (`timeoutMs` ignoré, délai de 30 s
  rétabli) et M-b10 (condition d'étape du lien retirée), parce que la fiche ne garde ni la garde de rôle, ni la valeur du
  délai long, ni la borne d'étape par une mutation.
- **Réversible** : oui.

### C-31-1-78 — Revue P1 : aucun avis ne cite un message qu'aucun écran n'affiche

- **Contexte** : B-1 ; « document indisponible » figurait dans deux avis et dans le manuel, sans être le texte d'aucun
  écran (« Le document n'a pas été restauré. », « Le PDF émis de cette facture est introuvable… »).
- **Retenu** : les avis disent « introuvables » sans guillemets ; le manuel cite les deux messages réels.
- **Réversible** : oui.

### C-31-1-79 — Revue P1 : après une issue inconnue, l'import est bloqué jusqu'au rechargement

- **Contexte** : E-LOW-3 / A-L5 ; C-31-1-71 ne figeait le formulaire qu'après un succès.
- **Retenu** : `unknownOutcome` désactive le fichier et « Importer » ; recharger la page les rouvre (choix délibéré, après
  vérification). Le message reste prudent quand rien n'est parti (E-LOW-4, B-6 : consignés).
- **Écartée** : un bouton « Réessayer quand même » (un geste destructeur de plus à l'écran).
- **Réversible** : oui.

### C-31-1-80 — Revue P1 : `/onboarding/restore` bornée aux étapes 0 à 2

- **Contexte** : E-LOW-2 ; le lien était borné, la page non.
- **Retenu** : la garde attend le layout (`await parent()`) puis renvoie à `/onboarding` au-delà de l'étape 2, comme un
  non-Admin.
- **Réversible** : oui.

## Développement de la 31-1c (2026-10-24, Opus 5.5)

### C-31-1-81 — « Sauvegarder maintenant » est audité ; les actions d'audit s'écrivent en littéral au site d'appel

- **Contexte** : le registre des routes mutantes exige qu'une route soit `Traced` ou exemptée avec le numéro d'une issue
  qui la suit ; `POST …/run` écrit une sauvegarde (un secret) sur le serveur, par le geste d'un administrateur. L'AC31
  n'exclut de l'audit que la sauvegarde **automatique** réussie (bruit quotidien).
- **Retenu** : `admin.backup_run` (statut, fichier), best-effort comme `admin.full_export` ; libellé ×4. Les deux audits
  best-effort de `routes/backup_schedule.rs` construisent leur `NewAuditLogEntry` au site d'appel, l'action en toutes
  lettres : l'aide `emit_audit` ne reçoit que l'entrée (la garde `audit_label_registry` ne voit pas un `&str` relayé, et
  l'inscrire aux sites indirects aurait réécrit les trois mentions « huit » du fichier).
- **Écartées** : exempter la route (aucune issue ne la suit) ; inscrire un neuvième site indirect.
- **Réversible** : oui.

### C-31-1-82 — Seuls les `.partial` abandonnés (non modifiés depuis 10 minutes) sont supprimés

- **Contexte** : l'AC28 (et F9) prescrivent de supprimer les `.partial` résiduels avant d'écrire (auto) et avant la
  sauvegarde de sécurité (import). Le premier gate ciblé a rougi au hasard sur deux tests d'import différents : les
  tests d'intégration partagent `/tmp` comme dossier de sauvegarde (`Config::from_fields_for_test`), et le nettoyage
  d'un import supprimait le `.partial` **en cours** d'un import parallèle (renommage en `ENOENT`, 500). C'est le mode
  d'échec que la fiche écrit en angle mort pour deux processus sur une même base — ici rencontré.
- **Retenu** : un `.partial` n'est supprimé que si sa date de modification date d'au moins `PARTIEL_ABANDONNE_APRES`
  (10 minutes) : un écrivain actif avance cette date à chaque écriture. Date illisible : le fichier reste. Le verrou en
  mémoire garde l'exclusion dans un processus ; le délai ferme le cas « deux processus » pour le nettoyage (pas pour
  deux écritures simultanées, toujours un angle mort). Test : un `.partial` récent reste, un `.partial` vieilli d'une
  heure disparaît (auto et pré-import). Mutation complémentaire M-c25. Manuel et `.env.example` le disent.
- **Écartées** : donner aux tests un dossier propre (corrige les tests, pas le défaut) ; ne rien nettoyer.
- **Réversible** : oui (la constante).

### C-31-1-83 — E2E : états posés par interception réseau ; manifeste lu sans bibliothèque ZIP

- **Contexte** : l'AC40 pose les statuts (d) à (g), (n), (p) « en base de test par `UPDATE` » ; Playwright n'a aucun
  accès à la base (aucune spec n'en a, aucune dépendance MariaDB côté frontend). R8 (P5) : l'AC38 lit le manifeste
  sans bibliothèque ZIP.
- **Retenu** : tous les états non atteignables par l'écran sont posés par `page.route` sur `GET backup-schedule` (et
  `PUT`, `POST …/run`, `GET backups` comme la fiche le prévoyait pour (h)–(k), (r)) — le rendu est le même, la source
  de l'état est la réponse de l'API. Le parcours réel (réglage, « Sauvegarder maintenant », liste, téléchargement)
  passe par le vrai serveur. Le manifeste est lu par un analyseur des en-têtes locaux de 20 lignes et `zlib`
  (`inflateRawSync`) : `formatVersion === 2` est vérifié, comme l'AC38 le demande.
- **Écartées** : une dépendance `mysql2` aux tests E2E ; ramener l'assertion à la signature `PK` (R8).
- **Réversible** : oui.

### C-31-1-84 — Découpage : `routes/backup_schedule.rs`, `backup-schedule.api.ts`, état partagé du bandeau

- **Contexte** : `routes/admin.rs` dépasse 1200 lignes ; R9 (P5) : le rafraîchissement du bandeau « après chaque action
  du panneau » n'avait pas de mécanisme.
- **Retenu** : les cinq handlers dans un module neuf ; côté frontend, `backup-schedule.api.ts` (API, `bannerKind`,
  `exceedsImportLimit`), `backup-schedule.svelte.ts` (état partagé : le panneau le pose après chaque action, le bandeau
  le lit et le relit toutes les 30 minutes), `AutoBackupSection.svelte` et `BackupScheduleBanner.svelte`, tous sous
  `features/admin-backup` (propriété des clés `admin-backup-*`). La garde lexicale des gardes possédés lit aussi
  `routes/backup_schedule.rs`, qui ne doit pas nommer `backup_lock`.
- **Réversible** : oui.

### C-31-1-85 — `nextDueAt` suit la règle d'`is_due` ; `run_once` tronque `now` à la milliseconde

- **Contexte** : R6 (P5) — « `now` si une valeur est future » contredisait `is_due` quand seule `last_success_at` est
  future et qu'une tentative est récente. Les colonnes sont en `DATETIME(3)`, le nom du fichier à la milliseconde.
- **Retenu** : `next_due_at = max(réussite + intervalle, tentative + min(intervalle, 1 h), now)`, chaque valeur future
  tenue pour nulle ; `is_due` en est la comparaison à `now` (une seule règle). `run_once` tronque `now` à la
  milliseconde : nom, `last_attempt_at` et `last_success_at` disent le même instant. Le texte du bandeau garde la date
  de la dernière réussite, même datée du futur (second volet de R6 non appliqué : la date affichée est exacte).
- **Réversible** : oui.

### C-31-1-86 — Dossier des documents absent : 0 ; relecture dans le `spawn_blocking`

- **Contexte** : F2 et F4 (P5).
- **Retenu** : `taille_des_documents` rend 0 pour un dossier absent (aucune pièce encore) et pour toute autre erreur de
  lecture, journalisée ; un fichier disparu entre la lecture du dossier et ses métadonnées est sauté. La relecture de
  l'AC28 se fait dans la même fermeture que l'écriture, après le renommage. Un espace libre illisible vaut `failed`.
- **Réversible** : oui.

### C-31-1-87 — « Sauvegarder maintenant » rend 200 quel que soit le statut ; tailles en unités localisées

- **Contexte** : l'AC32 dit « attend la fin, rend l'état » ; l'écran affiche des tailles lisibles.
- **Retenu** : `POST …/run` rend 200 et l'état, l'échec se lisant dans `lastStatus` (seul `Skipped` est une erreur
  HTTP, 409) ; une erreur de lecture ou d'écriture de l'état en base reste un 500. Les tailles s'affichent par
  `Intl.NumberFormat` en unités décimales localisées (o, ko, Mo, Go) sans clé de catalogue ; la mention « limite N
  Mio » garde l'unité binaire de `KESH_ADMIN_IMPORT_MAX_MB`. La date d'une ligne de la liste est la date de
  modification du fichier.
- **Réversible** : oui.

### C-31-1-88 — Un dossier `auto` illisible n'emporte pas la liste des sauvegardes du serveur

- **Contexte** : revue de code P1, L-B2 / E-4 — `fichiers_reguliers` propageait la première erreur (`read_dir` sur un
  `auto` qui n'est pas un dossier, entrée ou métadonnées illisibles) : `GET /admin/backups` rendait 500, et la voie
  normale du manuel pour récupérer une sauvegarde pré-import tombait avec elle.
- **Retenu** (la forme la plus simple qui tient) : dans `lister_les_sauvegardes`, une erreur de lecture du dossier
  `auto` est **journalisée** et ses lignes manquent ; les sauvegardes pré-import restent servies et téléchargeables.
  Dans `fichiers_reguliers`, une **entrée** illisible (entrée du dossier, métadonnées) est sautée et journalisée au
  lieu de faire échouer les autres — pour tous ses appelants : la rétention en garde alors plus, jamais moins ; la
  taille des documents ne la compte pas (le seuil d'espace peut en être sous-estimé d'autant, le disque plein pendant
  l'écriture restant attrapé par `disk_full`). Seul un dossier de sauvegarde **lui-même** illisible rend encore 500
  (rien ne peut alors être listé). L'état dit le problème sans champ neuf : le tour suivant ne peut pas écrire dans
  `auto` et enregistre `failed` (panneau et bandeau) — rien de plus n'est ajouté à l'API.
- **Rejeté** : un indicateur `autoDirUnreadable` dans la réponse (API, écran, quatre locales pour un cas que l'état
  dit déjà au tour suivant).
- **Réversible** : oui.

### C-31-1-89 — Le téléchargement ouvre sans suivre les liens ; disparu depuis la liste → 404

- **Contexte** : revue de code P1, L-B1 / E-5 — `File::open` suivait un lien posé dans la fenêtre liste → ouverture,
  et un fichier supprimé dans cette fenêtre (rétention) rendait 500.
- **Retenu** : `auto_backup::ouvrir_la_sauvegarde` ouvre en `O_RDONLY | O_NOFOLLOW | O_CLOEXEC` (`rustix`, déjà
  dépendance) puis vérifie par `fstat` un fichier régulier ; `ENOENT`, `ELOOP` ou non régulier → `Ok(None)` → 404,
  comme un nom absent ; toute autre erreur → 500. L'ouverture passe par `spawn_blocking`. `Content-Length` reste la
  taille lue à la liste (les archives sont immuables après leur renommage).
- **Réversible** : oui.

### C-31-1-90 — Un `.partial` daté du futur d'au moins dix minutes est abandonné

- **Contexte** : revue de code P1, E-3 — `duration_since` en erreur (date future) rendait le fichier « récent » à
  jamais : un `.partial` laissé avant un recul d'horloge n'était jamais supprimé.
- **Retenu** : l'abandon se juge sur l'**écart absolu** entre la date de modification et `maintenant` : au moins dix
  minutes, dans un sens ou dans l'autre. Une date légèrement future (moins de dix minutes : horloge d'un partage réseau
  un peu en avance, écriture peut-être en cours ailleurs) protège encore le fichier. Écrit au manuel (§ *Horloge du
  serveur déréglée*).
- **Rejeté** : la règle « une date future protège toujours », écrite seulement — elle laisse le fichier pour toujours.
- **Réversible** : oui.

### C-31-1-91 — Le délai de reprise est une heure, sans `min` ; « au moins deux intervalles »

- **Contexte** : revue de code P1, L-B4 — `delai_de_reprise = min(intervalle, 1 h)` avec des fréquences de 6, 12, 24 et
  168 h (la `CHECK` de la table et `ALLOWED_INTERVAL_HOURS`) : le `min` était mort, et le manuel promettait « ou de
  l'intervalle, s'il est plus court ». Le manuel disait aussi « plus de deux intervalles » pour un bandeau calculé par
  `>=`.
- **Retenu** : la constante `DELAI_DE_REPRISE = 1 h` remplace le `min` (code et texte disent la même chose) ; le
  manuel dit « retentée au bout d'une heure » et « au moins deux intervalles », le CHANGELOG « depuis au moins deux
  intervalles ». Garde G22 de `textes_coherents.rs`. Si une fréquence inférieure à une heure était un jour admise, le
  délai de reprise serait à revoir avec elle (il dépasserait l'intervalle).
- **Rejeté** : garder le `min` et corriger le seul texte — du code mort qui décrit une règle que rien n'exerce.
- **Réversible** : oui.

### C-31-1-92 — Le rappel de l'écran dit le volume de l'AC33, le manuel son pic

- **Contexte** : revue de code P1, F-A2 — le rappel `admin-backup-auto-reminder` s'arrêtait à « Chacune pèse à peu
  près la base plus le dossier des documents ».
- **Retenu** : le rappel dit ce que l'AC33 exige — « le dossier auto en garde le nombre conservé plus une, et le
  dossier de sauvegarde y ajoute une sauvegarde par import, jamais purgée » —, quatre locales, repli Svelte identique
  au fr-CH. Le pic pendant une écriture (une de plus, le `.partial`) reste au manuel seul (§ *Volume*), qui le dit
  déjà : l'écran garde une phrase courte.
- **Réversible** : oui.
