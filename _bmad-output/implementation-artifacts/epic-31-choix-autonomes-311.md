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
