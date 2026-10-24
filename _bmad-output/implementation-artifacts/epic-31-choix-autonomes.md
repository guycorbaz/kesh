# Epic 31 — registre des choix autonomes

Chaque choix tranché en autonomie : contexte, option retenue, alternatives écartées, réversibilité. Numérotation
`C-<story>-<k>`. Mise à jour **par union** (`docs/agents-orchestration.md` § 1) : chaque branche ajoute ses entrées dans
un registre propre à sa story (`epic-31-choix-autonomes-<clé>.md`), que l'orchestrateur verse ici au merge.

---

## Story 31-0 — Kickoff et actions des rétrospectives (doc seulement)

### C-31-0-1 — #503 d'abord, l'outillage et Rust en parallèle, la répétition à blanc en dernier

- **Contexte** : Guy fixe #503 en premier ; la 29-7 est reportée après #503 ; les actions C3, A2, A3, A10 et #577 sont
  reconduites ou laissées à l'orchestrateur.
- **Retenu** : 31-1 (#503) d'abord ; 31-2 (outillage) et 31-3 (Rust 1.97.1) **en parallèle** de la 31-1, dans leurs
  worktrees — elles ne touchent ni le schéma ni les écrans ; 31-4 (répétition à blanc) après le merge de la 31-1 ;
  une seule release v0.15.0 en fin d'epic.
- **Écartées** : publier la 31-1 seule en v0.14.1 — Guy reprend sa recette « à la release suivante », une seule suffit ;
  repousser l'outillage à un epic de dette — C3 et #577 touchent le filet des gates de cet epic même.
- **Réversible** : oui.

### C-31-0-2 — #577 : recréer le smoke E2E en CI, plutôt que corriger le texte

- **Contexte** : le `CLAUDE.md` affirme un smoke E2E en CI qui n'existe pas (#577, A-C7). Guy a répondu « ok » sans
  choisir entre recréer le smoke et corriger le texte ; l'orchestrateur avait recommandé de le recréer.
- **Retenu** : recréer un job minimal (login puis un appel API authentifié), dans la 31-2. C'est le seul filet si une
  story sans frontend casse le harnais, mode d'échec déjà payé (#285).
- **Écartée** : réécrire le `CLAUDE.md` pour dire que la CI ne lance aucun E2E — moins coûteux, mais laisse le
  harnais sans aucun contrôle automatique au moment où Kesh porte des livres réels.
- **Réversible** : oui (le job se retire).

### C-31-0-3 — Quatre petits volets d'outillage dans une seule story

- **Contexte** : C3 (droits du conteneur de dev), #577 (smoke), A2 (Docker authentifié), A3 (`.svelte-kit/`) sont
  petits, indépendants et sans code de production.
- **Retenu** : une story 31-2 à quatre volets, revue en une boucle. Rust 1.97.1 à part (31-3), parce que le nouveau
  clippy peut toucher tout le workspace.
- **Écartée** : quatre stories — quatre boucles de revue et quatre PR pour du YAML et un fichier SQL.
- **Réversible** : oui.

### C-31-0-4 — Les réglages nouveaux se font dans l'interface

- **Contexte** : Guy souhaite, à terme, que presque toute la configuration passe par l'interface et que le `.env` ne
  porte presque plus rien ; « pour l'instant, restons comme ça ».
- **Retenu** : l'Epic 31 ne migre aucun réglage existant ; mais tout réglage **nouveau** (fréquence et rétention de la
  sauvegarde automatique) se fait dans l'interface, pas par une variable d'environnement de plus — sauf impossibilité
  établie par la fiche.
- **Écartée** : ajouter des variables `KESH_BACKUP_*` — c'est le chemin que Guy veut quitter.
- **Réversible** : oui.

### C-31-0-5 — #695 ouverte au kickoff

- **Contexte** : la limite de 500 transactions de la réconciliation (choix C-29-9-3) n'avait pas d'issue, pour ne pas
  faire remonter le compte ; Guy, le 2026-10-23 : « si c'est nécessaire, on ouvre tout de même une issue ».
- **Retenu** : issue #695 ouverte (`enhancement`), hors périmètre de cet epic.
- **Réversible** : —

### C-31-0-6 — Le témoin du nombre de tables du NAS n'est pas bloquant

- **Contexte** : la requête des tables hors InnoDB rend « aucune ligne » sur le NAS (Guy, 2026-10-23), mais ne prouve
  rien si le nom de la base n'est pas `kesh` ; le témoin `COUNT(*) ≥ 40` reste à lancer.
- **Retenu** : la 31-1 ne le suppose pas — la sauvegarde automatique s'appuie sur l'instantané de la 28-3, qui vaut pour
  InnoDB ; la question reste posée à Guy à la liste de pré-release (action B6 de la rétrospective 28).
- **Réversible** : —

---

## Versé par union : `epic-31-choix-autonomes-311.md`

*(Versé le 2026-10-24 par la Story 32-0 — action A2 de la rétrospective de l'Epic 31. Contenu recopié sans retouche depuis la deuxième ligne du registre de la story, qui reste en place et auquel renvoient la fiche et `sprint-status.yaml`.)*

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

### C-31-1-93 — Un dossier `auto/` qui ne se liste pas fait échouer le tour avant l'écriture

- **Contexte** : revue de code P2, M1 — un `auto/` inscriptible et traversable mais illisible (`0o300`) acceptait
  l'écriture ; `appliquer_la_retention`, sur l'échec de `read_dir`, journalisait et rendait 0. La sauvegarde était
  annoncée réussie, la rétention ne tournait plus, et le disque se remplissait en silence — alors que le manuel (§ *Les
  sauvegardes du serveur*) dit que « la sauvegarde automatique suivante échoue, et le panneau comme le bandeau le
  disent ».
- **Retenu** : `ecrire` vérifie, juste après la création du dossier, que `auto/` se liste (`std::fs::read_dir`) ; sinon
  le tour s'arrête **avant toute écriture** en `failed`, journalisé. C'est le point le plus simple : un seul contrôle,
  dans le `spawn_blocking` qui touche déjà le disque, au même endroit que l'échec de création du dossier, et qui ne
  laisse aucune archive orpheline. Le manuel est vrai tel qu'il est écrit (rien à changer, PDF inchangé). La rétention
  garde son repli journalisé, qui ne vaut plus que pour un dossier changé entre le contrôle et la fin du tour.
- **Rejeté** : remonter l'échec de la rétention après l'écriture — une archive écrite et relue serait annoncée en
  échec, alors que `last_success_*` est déjà enregistré ; et restreindre le manuel à « l'écriture est refusée » — il
  décrirait le trou au lieu de le fermer.
- **Réversible** : oui.

---

## Versé par union : `epic-31-choix-autonomes-312.md`

*(Versé le 2026-10-24 par la Story 32-0 — action A2 de la rétrospective de l'Epic 31. Contenu recopié sans retouche depuis la deuxième ligne du registre de la story, qui reste en place et auquel renvoient la fiche et `sprint-status.yaml`.)*

### C-31-2-1 — Le montage de dev dépend du répertoire de lancement : un garde, pas un correctif du compose

- **Contexte** : le conteneur `kesh-mariadb-dev` monte `/home/gcorbaz/devel/kesh-chore-tmpfs/scripts/mariadb-init`, un
  répertoire vide (le worktree n'existe plus) ; `kesh` n'a donc que `kesh`.*. Le compose monte `./scripts/mariadb-init`,
  relatif au répertoire de lancement.
- **Retenu** : règle « créé depuis le dépôt principal » écrite + garde `scripts/check-dev-grants.sh` branché sur
  `test-fast.sh` (égalité d'ensembles de droits, dérivée du fichier versionné).
- **Écartées** : chemin absolu en dur (casse les autres postes) ; variable d'environnement de plus (un réglage à
  oublier) ; script `dev-db.sh` qui refuse hors dépôt principal (une story, issue demandée).
- **Réversible** : oui.

### C-31-2-2 — Le smoke tourne contre l'image Docker de la PR

- **Contexte** : #577 demande un job minimal ; deux montages possibles.
- **Retenu** : job `e2e-smoke`, `needs: docker-build`, image reconstruite par le cache `gha`, lancée en
  `--network=host` avec `KESH_TEST_MODE`, `KESH_HOST=127.0.0.1`, `KESH_COOKIE_SECURE=false`, base `kesh` sur le service
  MariaDB en utilisateur `kesh`. Une spec neuve `smoke.spec.ts` (login UI + `GET /auth/me` authentifié + sonde négative
  sans cookie).
- **Écartée** : `cargo build` + artefact `frontend-build` (second build complet, binaire de débogage, pas ce qu'on
  publie). Repli si le cache ne tient pas.
- **Réversible** : oui.

### C-31-2-3 — Docker Hub : pas de contournement conditionnel pour les forks

- **Contexte** : les PR de forks et de Dependabot n'ont pas les secrets.
- **Retenu** : `credentials` inconditionnels sur les trois services ; limite écrite dans `docs/ci.md` ; repli
  `mirror.gcr.io` noté, non adopté. Dépôt à un seul développeur, ni fork ni `dependabot.yml`.
- **Écartée** : expression conditionnelle sur `credentials` (invérifiable hors runner, fragile pour un cas inexistant).
- **Réversible** : oui.

### C-31-2-4 — La CI emploie `kesh` avec les droits du fichier de dev, plus `root`

- **Contexte** : la CI est `root` (plus large que la production) ; un test à privilège excessif y passe.
- **Retenu** : le job `backend` applique `01-dev-grants.sql` en `root` après l'attente de MariaDB, `DATABASE_URL` en
  `kesh` ; migrations et seed restent en `root` (outillage). Non vérifiable hors CI : tout rouge est trié, nommé et
  tracé, jamais annulé en silence.
- **Écartée** : laisser `root` (conserve l'asymétrie que C3 dénonce) ; modifier `release.yml` (publication, issue demandée).
- **Réversible** : oui.

### C-31-2-5 — `.dockerignore` reçoit aussi `/.svelte-kit/`

- **Contexte** : mécanique, même symptôme que le `.gitignore` racine. **Retenu** : ajouté. **Réversible** : oui.

### C-31-2-6 — Ordre de livraison 4, 3, 1, 2 et un commit par volet

- **Retenu** : du plus mécanique au plus coûteux ; un volet défaillant n'arrête pas les autres. **Réversible** : oui.

### C-31-2-7 — Le smoke n'est pas rendu obligatoire par la story

- **Retenu** : aucun réglage de branche protégée modifié ; la décision est demandée à Guy via l'orchestrateur.
- **Réversible** : oui.

### C-31-2-8 — Le garde ne remet jamais le conteneur en état

- **Retenu** : il nomme la remise en état (recréation depuis le dépôt principal) sans l'exécuter : elle efface les
  bases de tous les agents (`docs/agents-orchestration.md` § 2). L'orchestrateur la fait, hors gate. **Réversible** : oui.

### C-31-2-9 — Prémisse corrigée : le conteneur a MOINS de droits que le fichier

- **Contexte** : la rétrospective 29 § 4.2 et l'epic le disaient plus doté (`GRANT ALL *.*`). Constat 2026-10-23 :
  `kesh`.* seul, montage vide. **Retenu** : la fiche corrige ; à noter à la rétrospective 31, aucune issue ne rouvre C3.

### C-31-2-10 — #698 intégrée à la 31-2 (choix de l'orchestrateur)

- **Contexte** : la fiche demandait d'ouvrir en P3 le fait que le smoke de `release.yml` interroge l'image publiée avec
  l'utilisateur `root` ; ouverte (#698). La règle de release fixe le plafond au tag v0.15.0 à moins de 136 issues
  ouvertes ; le compte est à 138.
- **Retenu** : intégrer #698 au volet 1 de la 31-2 — le smoke de release applique `scripts/mariadb-init/01-dev-grants.sql`
  (ou un `GRANT` aux droits de la production) puis démarre l'image avec l'utilisateur `kesh`, comme le job `backend` de
  `ci.yml` dans la même story. La PR porte `closes #698`. Ne se vérifie qu'en CI de release : le dire.
- **Écartée** : laisser #698 ouverte — même mécanisme que le volet 1, même correction.
- **Réversible** : oui.
- **Rappel** : #699 (montage relatif) reste ouverte ; le garde du volet 1 la signale sans la prévenir.

### Amendements du 2026-10-23 (validation P1 ; les entrées ci-dessus ne sont pas réécrites)

- **Amende C-31-2-10** (R-2, F6) : la prescription « applique `01-dev-grants.sql` (ou un `GRANT`…) » est **retirée**. Le
  service de `release.yml` crée déjà `kesh` (`MARIADB_USER: kesh`, `release.yml:22-26`) avec `ALL` sur `kesh`.*, soit les
  droits de la production ; la seule modification est `DATABASE_URL` (`release.yml:94`) → `mysql://kesh:kesh_dev@
  127.0.0.1:3306/kesh`, sans fichier de droits ni `GRANT` (le fichier de dev ajouterait `\_sqlx\_test%`.* et `kesh%`.*,
  que la production n'a pas). Porté par l'AC18 et la tâche T4 de la fiche. Ne se vérifie qu'au tag v0.15.0.
- **Amende C-31-2-4** (R-1) : l'« écartée » « modifier `release.yml` » vaut pour `credentials` et le reste de la
  publication, **non** pour le `DATABASE_URL` du smoke, repris par C-31-2-10.
- **Amende C-31-2-9** (F2) : provenance du `GRANT ALL *.*` de la rétrospective 29 — le fichier versionné de `main` n'a
  jamais porté de droit global (seulement deux commits intermédiaires de la 22-5, corrigés en passe 1) ; le montage est
  vide depuis le 2026-10-11 ; le `*.*` a été **posé à la main par un agent** (journal de l'Epic 15, 2026-10-10). D'où
  l'interdit de tout `GRANT`/`REVOKE` manuel sur le MariaDB de dev partagé, ajouté à `docs/agents-orchestration.md` § 2 par la
  fiche (AC5). Le conteneur a été recréé par l'orchestrateur le 2026-10-23 vers 18:13Z depuis le dépôt principal : plus de
  recréation à demander (AC7, T7).

### C-31-2-11 — Docker Hub authentifié aussi pour le build des images (F7)

- **Contexte** : `credentials` ne couvre que les services MariaDB ; les jobs `docker-build` et `e2e-smoke` tirent `rust`,
  `node` et `debian` (Dockerfile) anonymement. Retirer l'avertissement A-C9 aurait laissé un texte qui dit moins que le
  dépôt.
- **Retenu** : `docker/login-action@v3` (mêmes secrets) avant le build de ces deux jobs (`release.yml` le fait déjà) ;
  AC13 compte les étapes de connexion en plus des `credentials`. Les forks échouent bruyamment, comme pour C-31-2-3.
- **Écartée** : réécrire l'avertissement au lieu de le retirer (garde la dépendance anonyme, pour un coût de deux étapes
  YAML). **Réversible** : oui.

### C-31-2-12 — Le garde a des sources injectables (R-5, F1, F3)

- **Contexte** : depuis la recréation du conteneur, aucun état défaillant réel n'existe pour éprouver le garde ; AC3 ne
  portait que sur des `SHOW GRANTS`.
- **Retenu** : acquisition et comparaison séparées pour les droits (`--grants-from-stdin`, lu en `-N -B -r`,
  normalisation minimale qui garde les antislashs, fixture capturée) et pour le montage (`--mount-source <répertoire>`,
  `cmp` avec le fichier du worktree) ; `PATH` sans `docker` testé. Mot de passe par `MYSQL_PWD`.
- **Écartée** : normaliser en retirant les antislashs (rend `_sqlx_test%` non échappé indiscernable du bon droit).
  **Réversible** : oui.

### Amendements du 2026-10-23 (validation P2 ; les entrées ci-dessus ne sont pas réécrites)

- **Amende C-31-2-4** (R2-8) : le « Retenu » dit « après l'attente de MariaDB » ; l'étape de droits emploie le client
  `mariadb`, installé par « Install mariadb-client » (`ci.yml:156-157`), **après** « Wait for MariaDB ». Elle se place donc
  après « Install mariadb-client » et avant « Apply migrations » (déjà dans la fiche, 1c).
- **Amende C-31-2-11** (F2-4, R2-5, F2-11) : la connexion `docker/login-action` précède `docker/setup-buildx-action` dans
  les trois jobs qui construisent (`docker-build`, `e2e-smoke`, `release.yml` — remontée de `:44` avant `:41`), le builder
  tirant `moby/buildkit` depuis Docker Hub ; `e2e-smoke` porte lui-même `setup-buildx` (cache `gha`). Que l'image du
  builder soit authentifiée est une hypothèse sur le comportement de l'action, constatée au journal du premier run.
- **Amende C-31-2-12** (F2-5, F2-6, F2-7) : le mot de passe est exporté puis transmis sans valeur (`docker exec -e
  MYSQL_PWD`) ; seule la chaîne du condensé est retirée de la ligne `USAGE` ; un attendu à zéro `GRANT` est refusé ; le
  garde distingue « absent » (trois cas, rend 0 en le disant) de « en erreur » (non nul, cause nommée).

### C-31-2-13 — Le test du garde est lancé par une enveloppe Rust dans `kesh-db` (R2-1, F2-1)

- **Contexte** : la fiche affirmait qu'aucun test automatique ne lance `scripts/tests/*.test.sh` ; faux, car
  `crates/kesh-db/tests/prepare_release_script.rs` lance `prepare-release.test.sh` (nextest, CI) et refuse un test muet.
- **Retenu** : `crates/kesh-db/tests/check_dev_grants_script.rs`, calquée sur ce précédent, lance
  `check-dev-grants.test.sh` ; toutes ses acquisitions sont injectées (jamais le conteneur réel). Conséquence assumée :
  `kesh-db` est touché, donc gate complet même en cours de boucle (`CLAUDE.md`).
- **Écartée** : un test local jamais lancé (test muet : M3 ne rougirait jamais en CI) ; une étape YAML d'une ligne dans
  `ci.yml` (ne tourne pas au gate local).
- **Réversible** : oui.

### C-31-2-14 — `docker-compose.dev.yml:107-108` et `README.md:183` rejoignent la story (R2-2, F2-3)

- **Contexte** : l'inventaire d'AC5, enfin exécutable, rend deux sites que la fiche ne nommait pas : un commentaire du
  compose de dev (« les droits globaux dont `#[sqlx::test]` a besoin », faux) et un `mysql://root:…` de `README.md`.
- **Retenu** : les deux sont réécrits (mécanique, même symptôme) ; le commentaire n'exige aucune recréation du conteneur.
- **Réversible** : oui.

### C-31-2-15 — Signal de recyclage en validation P3 : le garde des droits se réduit à ce qu'il doit voir (choix de l'orchestrateur, 2026-10-24)

- **Contexte** : la validation P3 (Opus) rend 5 MEDIUM (R) et 6 MEDIUM (F), après 3 et 5 en P2 — sévérité égale, et
  **tous** portent sur le garde `check-dev-grants.sh` tel que les remédiations P1 et P2 l'ont étendu (source du montage
  comparée par `cmp`, port de `DATABASE_URL`, distinction fine « absent / en erreur » par le texte des messages de
  `docker`, options `--grants-from-stdin` et `--mount-source`). C'est le **recyclage** du `CLAUDE.md` (§ *Règle de
  splitting préventif*, amendement D5) : chaque correction ajoutait une surface que la passe suivante trouvait fautive.
  La sortie que le `CLAUDE.md` désigne comme la meilleure est un **retrait**.
- **Retenu** : le garde ne fait plus que ce qui aurait détecté l'incident du 2026-10-23 — **comparer l'ensemble des
  droits de `kesh` sur le conteneur à celui du fichier versionné**. Conception :
  1. `docker` s'injecte par la variable `KESH_DOCKER` (défaut `docker`) : les tests passent un `docker` factice qui
     **consigne son argv**, et tous les cas sont de bout en bout, sans option d'injection propre au garde.
  2. Si `docker` est introuvable, ou si le conteneur n'est pas **en marche** (`docker ps --filter name=^<nom>$ --filter
     status=running -q` vide) : message « garde sauté : … » et code 0. Aucune lecture de message d'erreur de `docker`.
  3. Sinon : `SHOW GRANTS` par `docker exec`, le mot de passe transmis **par le nom** de la variable
     (`-e MYSQL_PWD`, valeur héritée de l'environnement), jamais par sa valeur dans l'argv ; échec d'`exec` → code non
     nul avec la cause ; ensembles différents → code non nul, différence affichée et commande de remise en état
     **nommée** (jamais exécutée).
  4. **Retirés** : la comparaison du port de `DATABASE_URL`, la comparaison de la source du montage, les options
     `--grants-from-stdin` et `--mount-source`. La règle « conteneur créé depuis le dépôt principal » reste une règle
     **écrite** (`docs/testing.md`, `docs/agents-orchestration.md`) ; le garde ne prétend pas la vérifier, et #699
     reste ouverte pour une prévention.
  5. Le câblage s'asserte sur **l'unique** ligne d'invocation **non commentée** de `test-fast.sh`.
- **Écartées** : redécouper la 31-2 (le recyclage est confiné au garde, les trois autres volets ont convergé) ;
  poursuivre l'extension (chaque passe en a trouvé le défaut suivant).
- **Réversible** : oui — une vérification retirée pourra revenir avec sa propre story si un incident la motive.

### Amendements du 2026-10-24 (validation P3 ; les entrées ci-dessus ne sont pas réécrites)

- **Amende C-31-2-1** (R3-5) : « un garde la rend visible » est inexact. Le garde ne signale que la **conséquence** (des
  droits qui ne sont plus ceux du fichier) ; un conteneur monté depuis un worktree encore présent, au fichier identique, passe.
  La fiche l'écrit comme angle mort (1b) et renvoie la prévention à #699.
- **Remplace C-31-2-12** (C-31-2-15, R3-2, R3-3) : les options `--grants-from-stdin` et `--mount-source`, la comparaison de la
  source du montage et celle du port ne sont plus dans la fiche. La seule injection est `KESH_DOCKER` ; tous les cas de test
  sont de bout en bout avec un `docker` factice qui consigne son argv. Ce qui reste de C-31-2-12 : `-N -B -r`, la
  normalisation minimale qui garde les antislashs, la fixture capturée, `MYSQL_PWD` sans valeur dans l'argv.
- **Amende C-31-2-12 / C-31-2-15** (F-L4, R3-L7, F-3) : le mot de passe vient de la clé `MARIADB_PASSWORD` du compose
  (`docker-compose.dev.yml:65`, avec `MARIADB_USER` et `MARIADB_DATABASE`), non de `DATABASE_URL` que le garde ne lit plus ;
  le garde lit ses fichiers relativement à son emplacement, ce qui permet au test de bâtir un mini-dépôt (`mktemp -d`) pour
  changer l'attendu (R3-L6). Aucun cas ne dépend d'une variable d'environnement héritée.
- **Amende C-31-2-14** (F-6) : l'inventaire d'AC5 gagne `README.md:193` et le doc-comment de
  `crates/kesh-api/tests/reconciliation_e2e.rs:8249-8250` (ajusté en T4) ; le décompte des modules passe à trois
  (`kesh-db`, `kesh-api`, `frontend`) — la règle de splitting ne joue toujours pas. La règle « depuis le dépôt principal »
  est portée aussi par les recettes `up`/`restart` (R3-L9).
- **Amende C-31-0-3** (F-L7) : « petits, indépendants » et « pour du YAML et un fichier SQL » ne décrivent plus la 31-2 : un
  couplage (le job `e2e-smoke` n'existe qu'au volet 2) et `kesh-db` touché pour l'enveloppe du test du garde (C-31-2-13).
  Le registre commun n'est pas réécrit ici.

### Amendements du 2026-10-24 (validation P4 ; ne réétendent pas C-31-2-15)

- **Précise C-31-2-15** (R4-1, F4-1, F4-2) : le condensé de `SHOW GRANTS` figure sur l'unique ligne `ON *.*`, que tout droit
  global posé à la main remplace. Le masquage vaut donc pour toute ligne `ON *.*` (jeton fixe), l'affichage d'un écart n'emploie
  jamais les mots `IDENTIFIED BY PASSWORD`. Ce n'est pas un contrôle de plus : c'est la comparaison existante, correctement définie.
- **Précise C-31-2-13** (R4-3, F4-7) : le test du garde est hermétique (mini-dépôt, `01-dev-grants.sql` de référence écrit par le
  test) ; seule l'assertion de câblage lit le vrai `test-fast.sh`. Une branche qui modifie le vrai fichier de droits ne fait donc
  rougir que le garde local, jamais la CI.
- **Écart à l'epic** (R4-9) : « rend le montage fiable » devient « le signale » (C-31-2-1, C-31-2-15) ; à porter à la rétrospective 31.


### Choix du développement (2026-10-23 ; agent de développement, ne réécrivent aucune entrée ci-dessus)

### C-31-2-16 — La ligne `USAGE` attendue n'est ajoutée que si le fichier ne porte aucun droit `ON *.*`

- **Contexte** : sur MariaDB, un droit global *remplace* la ligne `USAGE ON *.*` au lieu de s'y ajouter (fait établi en
  validation P4). Un fichier de droits qui accorderait un jour un droit global ferait attendre deux lignes `*.*` là où le
  serveur n'en rend qu'une : le garde rougirait pour toujours, même sur un conteneur conforme.
- **Retenu** : l'attendu reçoit `GRANT USAGE ON *.* TO \`<user>\`@\`%\`` seulement si aucune de ses lignes ne porte
  ` ON *.* TO `. C'est la définition correcte de l'attendu, non un contrôle de plus (C-31-2-15 tient).
- **Écartée** : ajouter `USAGE` inconditionnellement (faux rouge permanent dans un cas que le fichier interdit
  aujourd'hui, mais qu'il n'empêche pas). **Réversible** : oui.

### C-31-2-17 — Une clé `MARIADB_*` présente plusieurs fois dans le compose de dev est refusée

- **Contexte** : le garde lit `MARIADB_USER`, `MARIADB_PASSWORD` et `MARIADB_DATABASE` par `grep` ; un second service qui
  porterait la même clé rendrait la lecture ambiguë.
- **Retenu** : une seule occurrence admise, sinon non nul, clé et nombre d'occurrences nommés — même famille que « clé
  absente, vide ou interpolée » (fiche 1b). **Écartée** : lire le seul bloc `mariadb:` (analyse YAML en shell, fragile).
  **Réversible** : oui.

### C-31-2-18 — L'extraction des `GRANT` est en awk POSIX, éprouvée sous gawk et mawk

- **Contexte** : le poste a gawk, `ubuntu-latest` a mawk par défaut ; le test du garde tourne en CI par son enveloppe.
- **Retenu** : aucune extension gawk ; le test a été rejoué avec `awk` → `mawk 1.3.4` (105 assertions vertes).
  **Réversible** : oui.

### C-31-2-19 — Le garde dit aussi sa réussite

- **Retenu** : « garde des droits : droits de kesh conformes à 01-dev-grants.sql » (code 0) ; sans lui, `test-fast.sh`
  n'affichait rien entre l'en-tête de l'étape et la suivante, et l'AC7 ne se constatait qu'au code de retour.
  **Réversible** : oui.

### C-31-2-20 — Smoke : sonde négative par la fixture `request`, nettoyage dans le premier test

- **Contexte** : `resolveBackendUrl` n'est pas exporté ; un `afterEach({ page })` ouvre une page vide pour la sonde
  négative, et `clearAuthStorage` y journalise un `SecurityError` sur `localStorage`.
- **Retenu** : la sonde négative emploie la fixture `request` de Playwright (contexte neuf, sans cookie, sur le `baseURL`
  de la configuration, qui lit le même `KESH_BACKEND_URL`) ; `clearAuthStorage` est appelé dans le `finally` du premier
  test. Aucun helper réécrit ni exporté. **Réversible** : oui.

### C-31-2-21 — `website/index.html:189` n'est pas modifié (assumé par écrit, AC5)

- **Retenu** : la commande `docker compose -f docker-compose.dev.yml up -d` du guide public suit un `git clone` dans la
  même recette : elle se lance, par construction, depuis le dépôt principal. La page est en anglais et publique ; y
  ajouter la règle des worktrees parlerait à un lecteur qui n'en a pas. **Réversible** : oui.

### C-31-2-22 — Le CHANGELOG range la 31-2 sous « Interne, invisible de l'utilisateur »

- **Retenu** : sous-section déjà employée par le CHANGELOG ; aucune des quatre entrées ne change ce que voit
  l'utilisateur de Kesh. Section `## [0.15.0] — Non publié` créée par la 31-2 (la 31-3 l'écrit aussi : union au rebase).
  **Réversible** : oui.

---

## Versé par union : `epic-31-choix-autonomes-313.md`

*(Versé le 2026-10-24 par la Story 32-0 — action A2 de la rétrospective de l'Epic 31. Contenu recopié sans retouche depuis la deuxième ligne du registre de la story, qui reste en place et auquel renvoient la fiche et `sprint-status.yaml`.)*

Versé par union dans `epic-31-choix-autonomes.md` au merge.

## Story 31-3

### C-31-3-1 — Une story courte, sans validation multi-passes

- **Contexte** : le bump touche une ligne de configuration, un `FROM`, sept sites de documentation (six lignes de la validation P1 plus `DOCKER_START.md`, validation P2 R8/F1) et d'éventuels
  correctifs clippy mécaniques ; pas de logique, pas de schéma.
- **Retenu** : une seule passe P1 **ciblée**, une lentille F, Haiku, sur la fiche (**révisé** : la fiche a changé de périmètre avec
  C-31-3-6, une P2 **complète** Sonnet, lentilles R et F, la suit) ; la revue de code se fera sur le diff
  réel (surtout les correctifs de lints).
- **Écartées** : deux lentilles Sonnet (R et F) comme pour une story fonctionnelle — coût sans objet ici.
- **Révisé le 2026-10-23 (validation P2, R5)** — titre et « Écartées » ci-dessus restent tels qu'à leur date ; ils ne
  valent plus. La fiche ayant intégré la garde G21 (C-31-3-6), la validation suit la règle commune (`CLAUDE.md` § *Review
  Iteration Rule*, `docs/agents-orchestration.md` § 3) : **passes complètes, Sonnet et Opus en alternance, lentilles R et F,
  jusqu'à une passe à 0 MEDIUM+ ou 8 passes**. La P2 (Opus) a rendu 6 MEDIUM + 1 MEDIUM ; la P3 (Sonnet) suit.
- **Réversible** : oui.

### C-31-3-2 — Tag Docker `rust:1.97-bookworm` (mineur), comme `1.96` aujourd'hui

- **Contexte** : le `Dockerfile` fixe le tag mineur ; `rust-toolchain.toml` (copié dans l'image) fixe le patch.
- **Retenu** : garder la convention du tag mineur ; le dev vérifie l'existence du tag, sinon `rust:1.97.1-bookworm`.
- **Écartée** : épingler un digest — hors périmètre, cela change la politique de mise à jour de l'image.
- **Réversible** : oui.

### C-31-3-3 — Les toolchains 1.96 restent installées sur la station

- **Contexte** : les **15 worktrees** de la station (dépôt principal, Epics 15, 29, 30 et 31, releases 0.13.4 et 0.14.0)
  portent tous `channel = "1.96.1"` dans leur `rust-toolchain.toml` (relevé du 2026-10-23, validation P1 F2), tant qu'ils
  n'ont pas rebasé sur `main` après ce merge.
- **Retenu** : le nettoyage (AC6) porte sur les **cibles cargo** de worktrees terminés, pas sur les toolchains.
- **Écartée** : `rustup toolchain uninstall 1.96.1` — casserait les branches qui n'ont pas rebasé.
- **Réversible** : oui (réinstallation).

### C-31-3-4 — Un lint nouveau se corrige ; l'exception se pose au site, jamais globalement

- **Contexte** : la règle de l'action A10 (« gate vert avec le nouveau clippy ») ne dit pas quoi faire d'un lint qui
  demanderait un changement de comportement.
- **Retenu** : correction mécanique par défaut ; sinon `#[allow]` local justifié et inscrit ici et au Dev Agent
  Record ; un lint non mécanique reçoit **les deux** : l'`allow` au site et une issue de suivi de son correctif, demandée à
  l'orchestrateur (une seule conduite, validation P1 F5).
- **Écartée** : `-A clippy::x` en CI ou `#![allow]` de crate — éteint le lint pour tout le code futur.
- **Réversible** : oui.

### C-31-3-5 — Les mentions historiques de « Rust 1.96 » ne se réécrivent pas

- **Contexte** : `CHANGELOG.md` (v0.4.0) et le tableau de versions du README disent « Rust 1.96 » pour la v0.4.0.
- **Retenu** : laissées telles quelles — relevés datés (`CLAUDE.md` : un compte rendu daté ne se « met pas à jour »).
- **Écartée** : réécrire ces mentions en 1.97 — cela ferait mentir la v0.4.0 sur la chaîne avec laquelle elle a été bâtie.
- **Réversible** : oui.

### C-31-3-6 — #697 intégrée à la 31-3 (choix de l'orchestrateur)

- **Contexte** : la fiche demandait d'ouvrir une issue pour l'absence de garde entre le `FROM` du `Dockerfile` et
  `rust-toolchain.toml` ; ouverte (#697). La règle de release fixe le plafond au tag v0.15.0 à moins de 136 issues
  ouvertes, et le compte est à 138 après trois issues de traçage nées des fiches.
- **Retenu** : intégrer #697 à la 31-3 — une garde dans `crates/kesh-api/tests/textes_coherents.rs` qui lit les deux
  valeurs et rougit si la version mineure diffère ; la PR de la 31-3 porte `closes #697`. C'est le site naturel : la 31-3
  bumpe justement les deux.
- **Écartée** : laisser #697 ouverte — la garde est petite et la 31-3 en est l'occasion.
- **Réversible** : oui.
- **Justification (validation P2, R6 — arbitrage de l'orchestrateur)** : la règle du périmètre minimal
  (`docs/agents-orchestration.md` § 3 : un défaut voisin ne s'intègre que si sa correction est mécanique) **admet le budget
  d'issues comme motif d'intégration** — décision de Guy du 2026-10-23, `CLAUDE.md` § *Priorités des défauts*, « on trace
  toujours » : l'issue existe (#697), elle est tracée, et la fermer ici ramène le compte vers le plafond. La garde est
  petite et isolée (un test pur plus la lecture de deux fichiers, aucun code de production), et la 31-3 touche justement
  ces deux fichiers. Le corps de #697 disait « non mécanique » : c'est la fiche qui fait foi ; l'orchestrateur lèvera le
  « hors périmètre » de l'issue par un commentaire.
- **Annotation du 2026-10-23 (validation P3, R5)** : l'attribution ci-dessus est inexacte — `docs/agents-orchestration.md`
  § 3 ne pose que la règle du mécanique ; le budget d'issues comme motif **accepté** d'intégrer un défaut voisin est écrit
  au `CLAUDE.md` § *Priorités des défauts* (« ⛔ On trace toujours »), qui fait foi. La garde n'étant pas strictement
  mécanique, c'est ce second texte, et non le § 3, qui fonde l'intégration.

### C-31-3-7 — numéro non attribué

- Aucun choix ne porte ce numéro (trou de numérotation né des remédiations de la validation ; relevé par la validation P3,
  R4/F3). Il n'est **pas** réattribué et les suivants ne sont pas renumérotés : C-31-3-8 et C-31-3-9 sont cités par la
  fiche et par les rapports de passe.

### C-31-3-8 — G21 vit dans `textes_coherents.rs`, pas dans `configuration_transmise.rs`

- **Contexte** : `configuration_transmise.rs` lit déjà le `Dockerfile` (`lire("Dockerfile")` l. 1897, `dockerfile_fixe`
  l. 468-480) ; `textes_coherents.rs` garde des textes, dispose de `racine()`/`lire()`, de `regex` et de G19 comme modèle
  d'extraction adossée à une positive (validation P2, R14).
- **Retenu** : `textes_coherents.rs` — c'est l'emplacement que nomme #697, la garde compare deux *textes* de
  configuration, et la lecture qu'offre `configuration_transmise.rs` ne sert pas (`dockerfile_fixe` ne regarde que
  l'étape finale, après le dernier `FROM`, alors que G21 cherche l'étape `rust:`) : rien à mutualiser, la règle DRY n'est
  pas en jeu. G21 est libre (aucune occurrence dans les 15 arbres de travail).
- **Écartée** : `configuration_transmise.rs` — ses gardes G14-G17 portent sur l'environnement transmis à l'image, pas sur
  sa chaîne de build.
- **Réversible** : oui.

### C-31-3-9 — `rust-toolchain.toml` d'abord, puis les correctifs de lints

- **Contexte** : la première rédaction relevait les lints sous `+1.97.1`, corrigeait, puis bumpait. Sous 1.96.1, le gate
  ciblé ne voit pas les lints corrigés, et un correctif employant une API stabilisée en 1.97 (cas courant des lints
  `manual_*`, aucun `rust-version`/`msrv` ne le bornant) ne compile pas (validation P2, R3).
- **Retenu** : T2 bumpe `rust-toolchain.toml` seul, avant les correctifs ; chaque commit compile sous la chaîne qu'il porte ;
  le gate ciblé de T3 tourne sous 1.97.1. Le désaccord transitoire avec le `Dockerfile` est sans conséquence (G21 arrive à
  T4, et son test du dépôt est alors réellement rouge avant le bump du `Dockerfile`).
- **Écartée** : corriger sous `+1.97.1` en laissant 1.96.1 dans le fichier, en exigeant que chaque correctif compile aussi
  sous 1.96.1 — contrainte inutile et invérifiable commit par commit.
- **Réversible** : oui.

### C-31-3-10 — Sous-section `### Modifié`, et non `### Changed`, au CHANGELOG

- **Contexte** : T8 prescrit d'ajouter la ligne de la chaîne « sous `### Changed` ». Le `CHANGELOG.md` nomme ses
  sous-sections en français depuis la v0.4.0 (`### Ajouté`, `### Modifié`, `### Corrigé` : douze `### Modifié`) ; les trois
  `### Changed` restants appartiennent à des versions antérieures (l. 852, 870, 901). `scripts/prepare-release.sh` ne lit
  aucune sous-section (`grep` : aucune occurrence de `Changed` ni de `Modifié`).
- **Retenu** : `### Modifié`, convention vivante du fichier ; c'est l'intention de la fiche (la section des changements),
  écrite dans la langue du fichier.
- **Écartée** : `### Changed` à la lettre — une section 0.15.0 seule à mélanger les deux langues ; une autre story qui
  ajouterait `### Modifié` à la même section créerait deux sous-sections homologues au rebase.
- **Réversible** : oui.

---

## Versé par union : `epic-31-choix-autonomes-314.md`

*(Versé le 2026-10-24 par la Story 32-0 — action A2 de la rétrospective de l'Epic 31. Contenu recopié sans retouche depuis la deuxième ligne du registre de la story, qui reste en place et auquel renvoient la fiche et `sprint-status.yaml`.)*

Versé par union dans `epic-31-choix-autonomes.md` au merge.

## Story 31-4

### C-31-4-1 — Une story de documentation exécutée, avec un script et une garde, sans code produit

- **Contexte** : le cahier des charges de la 29-7 est « doc seulement, écrite en l'exécutant ». Deux manques en
  découlent : il faut une sauvegarde synthétique de format 2 reproductible (sans quoi la procédure ne se rejoue pas à
  la release), et rien n'empêche que les contrôles anti-écrasement (R1, R2) s'effacent du manuel au fil des éditions.
- **Retenu** : le manuel (`.tex` et PDF), un script de fixture (`scripts/repetition-fixture.sh`) et son test sans
  serveur, une garde de texte G22, l'entrée CHANGELOG. Aucune modification du produit, des compose ni de `.env.example`.
- **Écartées** : (a) le manuel seul, sans garde — R1 (HIGH) est le défaut qui a fait retirer la story ; le laisser tenir
  par la seule vigilance reproduit le mode d'échec ; (b) corriger les compose (noms et port en dur) dans cette story —
  cela touche l'installation de Guy et sort du périmètre ; demandé en issue.
- **Réversible** : oui.

### C-31-4-2 — La sauvegarde synthétique se fabrique par un script contre une instance source en `KESH_TEST_MODE`

- **Contexte** : il faut un `.keshbackup` de format 2, avec deux sortes de pièces, sans aucune donnée réelle.
- **Retenu** : un script `bash` + `curl` + `jq`, qui reprend à la lettre les appels HTTP de
  `fiscal-year-closing-journey.spec.ts` et de `backup-restore-new-install.spec.ts` (`_test/seed` d'abord), contre une
  instance dev `cargo run` en mode test sur la base `kesh_314`. Il refuse de tourner hors `KESH_TEST_MODE`.
  *(Amendé en validation P1, 2026-10-24 — R7, R8, F8, F9 : « à la lettre » devient « repris » ; voir C-31-4-11.)*
- **Écartées** : (a) saisir la fixture à la main par l'interface — non reproductible ; (b) une spec Playwright
  « fixture » — alourdit la suite E2E pour un besoin d'exploitation, et la suite a déjà ses 7 à 10 échecs attendus ;
  (c) exporter un jeu de `~/kesh-test/` — jamais (données réelles).
- **Réversible** : oui (le script est un fichier).

### C-31-4-3 — La répétition ne se joue jamais sur l'hôte qui porte la production

- **Contexte** : `docker-compose.yml` fixe `container_name: kesh-api` / `kesh-mariadb` ; le compose Synology fixe
  `kesh-api`. Deux piles ne coexistent pas sur un hôte, et une collision partielle est le terrain exact de R1.
- **Retenu** : la procédure se joue sur une **autre machine** ; contrôle d'hôte (`docker ps -a`) avant tout ; le nom de
  projet Compose est posé (`COMPOSE_PROJECT_NAME=kesh-repetition`) pour que `down -v` ne vise que la copie.
  *(Amendé en validation P1, 2026-10-24 — R1 HIGH, F3, F4 : le nom de projet n'est plus confié au `.env`, que le
  téléchargement de `.env.example` écrasait et qu'un `COMPOSE_PROJECT_NAME` du shell dominait ; il est porté par `-p
  kesh-repetition` sur chaque commande (C-31-4-10). Le contrôle d'hôte vérifie le code de retour de `docker ps`, les
  noms exacts `kesh-api`/`kesh-mariadb` et les étiquettes du projet, au début **et** à la fin.)*
- **Écartées** : surcharger les `container_name` et le réseau par un override pour cohabiter sur un même hôte — **non éprouvé** (la fiche ne l'affirme ni ne le promet) et plus risqué que d'avoir un hôte distinct, la collision partielle étant le terrain de R1 ; le défaut (noms et port en dur) est demandé en issue.
- **Réversible** : oui.

### C-31-4-4 — Au développement, l'image est celle de `main` ; l'image publiée est rejouée à la release

- **Contexte** : le format 2 (31-1a, 31-1b) n'existe dans aucune image publiée avant la 0.15.0. La procédure s'écrit
  pourtant maintenant, et avant la clôture de 2026.
- **Retenu** : le `docker-compose.override.yml` épingle `image:` ; au développement, sur `gcorbaz/kesh:314` construite
  depuis le worktree ; le manuel dit « la version relevée » ; le **rejeu sur l'image publiée** est demandé à la
  release (action à l'orchestrateur), et la réserve est écrite au manuel (AC10) tant qu'il n'a pas eu lieu.
  *(Précisé en validation P1, 2026-10-24 — F, faits vérifiés : `gcorbaz/kesh:314` affiche `0.14.0` à `/health` tant que
  les crates ne sont pas bumpées, alors que `gcorbaz/kesh:0.14.0` publiée refuse le format 2 (`manifest.rs` l. 25) ; au
  développement, l'image locale se substitue à `<version relevée>` et `docker image inspect` au `pull` (T8). L'argument
  « l'anti-downgrade refuse une copie plus ancienne » est retiré : voir C-31-4-12.)*
- **Écartées** : attendre la 0.15.0 pour écrire la fiche — fait glisser une story qui précède la clôture de 2026 sans
  gain ; écrire sans exécuter — c'est le mode d'échec que la story existe pour éviter.
- **Réversible** : oui.

### C-31-4-5 — La destruction offre un chemin sans `sudo`

- **Contexte** : les dossiers `log/ inbox/ documents/ backup/` de la copie sont créés par Docker, en root ; le cahier
  des charges ne prévoit que `sudo` (F5). La station des agents n'a pas de `sudo` (`sudo -n true` refusé) ; un lecteur du
  manuel peut ne pas en avoir non plus.
- **Retenu** : deux chemins écrits : `sudo rm -rf` et un conteneur jetable (`docker run --rm -v … --entrypoint sh <image
  déjà tirée> -c 'rm -rf …'`), garde de chemin non vide. L'image est celle de MariaDB, déjà présente sur la machine.
- **Écartées** : un `chown` récursif par `sudo` — même besoin de `sudo` ; faire tourner le conteneur sous l'utilisateur
  de l'hôte (`user:` dans l'override) — change le comportement du produit (le conteneur écrit en root par conception).
- **Réversible** : oui.

### C-31-4-6 — La copie n'écoute que sur la boucle locale

- **Contexte** : `docker-compose.yml` publie `"80:80"` sur toutes les interfaces ; la copie porte les données réelles de
  Guy sur sa station. Le port 80 peut aussi être pris.
- **Retenu** : `ports: !override ["127.0.0.1:<port>:80"]` dans le `docker-compose.override.yml` de la copie
  (Compose ≥ 2.24 ; essai à blanc de `docker compose config` réussi en 2.40.3) ; le manuel nomme la version minimale.
  *(Précisé en validation P1, 2026-10-24 : l'override est **nommé** par un second `-f` — avec un `-f` explicite,
  Compose ne le charge plus d'office (constat T3 (b)).)*
  *(Précisé en remédiation P2, 2026-10-24 — R1, F8 : la version ≥ 2.24 devient un contrôle (`docker compose version
  --short`, contrôle 1) ; le contrôle 8 exige **une seule** publication de `kesh-api`, `host_ip: 127.0.0.1`, aucun
  `published: "80"` ni `0.0.0.0` — sous une Compose qui fusionnerait les listes, `80:80` subsisterait et l'ancien
  contrôle passait. R9 LOW : l'override pose aussi `restart: "no"`.)*
- **Écartées** : changer `KESH_PORT` — ne change pas la publication ; éditer le compose téléchargé — le contrôle de
  l'AC3 porte alors sur un fichier modifié à la main et perd son sens.
- **Réversible** : oui.

### C-31-4-7 — Une garde G22 épingle les contrôles, pas l'exactitude

- **Contexte** : `textes_coherents.rs` tient déjà G19 (URL au tag) et G16/G20 sur des jetons du manuel.
- **Retenu** : G22 exige les jetons `@mariadb:3306`, `docker compose config`, `COMPOSE_PROJECT_NAME`, l'ordre
  contrôle-avant-`up`, l'absence de `*_HOST_DIR` assigné dans un listing, et une positive adossée (sous-section non vide).
  Elle est dite *limitée* : l'exactitude de la procédure n'est portée que par le journal d'exécution.
  *(Amendé en validation P1, 2026-10-24 — R3, F10, R L3 : jetons `config --services`, `name: kesh-repetition`,
  `@mariadb:3306` (second filet), `docker context`, `/health`, `étapes 2 à 5` ; ordre lu **par position** dans les seuls
  listings, avec une relecture `config` exigée entre l'override et `up -d` ; positive chiffrée (60 lignes, 8 commandes) ;
  (e) nouvelle : chaque `docker compose` des listings porte `-p kesh-repetition` et `--project-directory`. G14 étant
  aveugle à la forme `-p` (angle mort écrit), c'est G22 (e) qui tient ces commandes.)*
  *(Remplacé en validation P2, 2026-10-24, par C-31-4-14 : G22 se réduit à des présences dans le source brut ; ordre,
  seuils et (e) sont retirés, et la forme de chaque commande n'est plus tenue que par le journal et la T8.)*
- **Écartées** : aucune garde (voir C-31-4-1) ; une garde qui « exécute » les commandes du manuel — elle exigerait
  Docker en CI et un réseau.
- **Réversible** : oui.

### C-31-4-8 — La restauration est un renvoi, pas une recopie

- **Contexte** : la 31-1b a écrit `Restaurer sur une nouvelle machine` (`sec:restauration-machine-neuve`), qui décrit
  `/setup`, l'assistant et l'import.
- **Retenu** : la répétition renvoie à cette section et ne redit que ce qui est propre à la copie ; placée juste après
  `Test de restauration périodique`, qu'elle complète (même exigence OLICo art. 10 al. 1).
  *(Amendé en validation P1, 2026-10-24 — R4, F6 : le renvoi ne vaut que pour les **étapes 2 à 5** de la section ; son
  étape 1 reprend `KESH_SMTP_*` et les dossiers de l'ancien `.env`, l'inverse de la copie, et le manuel le dit en toutes
  lettres. R L5 : la répétition ne vaut test OLICo que si l'export de souveraineté et son SHA-256 sont faits avant les
  gestes de clôture.)*
  *(Précisé en remédiation P2, 2026-10-24 — F4 : la place « juste après » est **maintenue**, et le renvoi relatif
  « cf. sous-section précédente » de l'encadré de `sec:backup-dsm` (`admin-manual.tex` l. 1847), qui désignerait la
  nouvelle sous-section, devient `\S\ref{sec:test-restauration}`. Écartée : placer la sous-section après
  `sec:backup-dsm` — elle s'éloignerait du test qu'elle peut remplacer, et un renvoi relatif resterait à la merci du
  prochain déplacement.)*
- **Écartées** : recopier la marche (deux textes à tenir d'accord — DRY) ; la placer en *Mise à jour* (elle n'est pas une
  mise à jour).
- **Réversible** : oui.

### C-31-4-9 — Indépendance de la 31-1c

- **Contexte** : la sauvegarde automatique se développe en parallèle.
- **Retenu** : la procédure prend un `.keshbackup` quel qu'il soit ; elle ne cite pas la 31-1c. Si elle est mergée avant le
  développement **et vue fonctionner**, le dev peut ajouter une phrase ; sinon, rien.
- **Écartées** : décrire une fonction non mergée — le manuel affirmerait ce qui n'a pas tourné.
- **Réversible** : oui.

### C-31-4-10 — Chaque commande Compose porte `-p`, `--project-directory` et des `-f` explicites (validation P1)

- **Contexte** : R1 (HIGH) de la validation P1 — le nom de projet posé dans le `.env` était écrasé par le téléchargement
  de `.env.example` et dominé par un `COMPOSE_PROJECT_NAME` du shell (que `scripts/synology/recette.sh` l. 33 exporte) ;
  sans `-f`, Compose cherche un compose dans les dossiers parents ; `DOCKER_HOST` ou un contexte distant fait jouer la
  copie ailleurs. Une copie dont le projet prendrait le nom `kesh` viserait le service `mariadb` du compose de dev.
- **Retenu** : `docker compose -p kesh-repetition --project-directory "$COPIE" -f "$COPIE/docker-compose.yml" [-f
  "$COPIE/docker-compose.override.yml"] …` sur **chaque** commande ; `$COPIE` absolu, nommé `kesh-repetition-*` ;
  neutralisation préalable de `COMPOSE_*`, `DOCKER_HOST`, `DOCKER_CONTEXT`, `KESH_*`, `MARIADB_*` et lecture de
  `docker context inspect` (adresse `unix://`). Les comportements de Compose invoqués sont des **constats T3**.
- **Écartées** : (a) `COMPOSE_PROJECT_NAME` dans le `.env` — c'est le défaut ; (b) `name: kesh-repetition` dans
  l'override seul — une variable d'environnement l'emporte ; (c) une fonction ou un alias shell qui porte les options —
  plus court, mais la garde G22 (e) ne lirait plus chaque commande, et une session sans la fonction retomberait sur les
  défauts implicites.
- **Réversible** : oui.

### C-31-4-11 — La fixture prend sa boîte en argument, refuse tout non-2xx, et son test tourne au gate (validation P1)

- **Contexte** : R7, R8, F8, F9 — le dépôt d'un document écrivait dans `KESH_INBOX_DIR`, par défaut
  `/tmp/kesh-e2e/inbox` (partagé) ; hors mode test, `_test/seed` rend 404 **ou** 405 (repli `ServeDir`) ; la facture du
  helper est datée du jour ; les spécifications exportent par l'écran ; M6 n'avait aucun test qui la tue ; un test shell
  que rien ne lance se tait.
- **Retenu** : trois arguments (`<url> <sortie> <boîte>`) sans défaut ni héritage ; refus de l'URL non locale, des
  dossiers non vides, de l'instance injoignable et de **tout** code non 2xx de `_test/seed` ; poignée de main sur
  `accepted[]` ; facture au `2026-06-15` ; export écrit sur `GET /api/v1/admin/full-export` ; test au `curl` factice
  (404, 405, 200) lancé par `crates/kesh-api/tests/repetition_fixture_script.rs`, calqué sur
  `crates/kesh-db/tests/prepare_release_script.rs`.
  *(Précisé en remédiation P2, 2026-10-24 — R4/F7 : la fixture s'authentifie par `POST /api/v1/auth/login` avec
  `admin`/`admin123`, identifiants du preset écrits en **constantes** du script (preset de test, instance en mode test
  sur la boucle locale : pas un secret), et passe `accessToken` (corps camelCase) en `Authorization: Bearer` ; un login
  refusé arrête tout, cas ajouté au test (M15). Écartés : un quatrième argument ou une variable d'environnement — le
  mot de passe d'un preset de test n'a rien à protéger, et l'environnement est ce que les trois arguments excluent.
  F3 : la validation d'une facture ne fige pas son PDF ; la fixture crée un contact et un compte bancaire **principal**,
  puis appelle `GET /api/v1/invoices/<id>/pdf` (200, `%PDF`) et exige `X-Kesh-Backup-Files: 2` à l'export. R13 :
  le document fournisseur est `frontend/tests/e2e/fixtures/spc_e2e_invoice.png`, copié. R11 : l'enveloppe Rust reprend
  la garde contre le test muet du modèle.)*
- **Écartées** : (a) refuser sur « 404 » — un 405 passerait ; (b) un faux serveur HTTP — plus lourd que le `curl` factice
  du précédent `prepare-release.test.sh` ; (c) laisser le test hors gate, lancé à la main en T7 — test muet.
- **Réversible** : oui.

### C-31-4-12 — La version de la copie se contrôle à `/health`, elle ne se déduit pas de l'anti-downgrade (validation P1)

- **Contexte** : R9, F1 — l'import ne refuse que si `kesh_version_min_required` de la source dépasse le binaire
  (`kesh-db/src/version.rs` l. 270-275) ; une sauvegarde de 0.15.1 s'importe dans une 0.15.0 sans table nouvelle. La
  fiche affirmait le contraire.
- **Retenu** : contrôle 9 de l'AC3 — après `up -d`, `version` de `/health` égale **exactement** la version relevée sur la
  production ; sinon `down -v` et arrêt. `pull kesh-api` avant `up` pour qu'une version mal écrite échoue au tirage, et
  non par une construction tentée dans `$COPIE`.
- **Écartées** : (a) garder l'argument — faux ; (b) n'exiger qu'une version « supérieure ou égale » — la répétition
  montrerait d'autres gestes que ceux de la production, sans le signaler.
- **Réversible** : oui.

### C-31-4-13 — Le chemin « Paramètres → Exercices comptables » du manuel utilisateur, corrigé par la 31-4 (choix de l'orchestrateur, 2026-10-24)

- **Contexte** : la remédiation P1 demandait une issue (P3, `bug`) : `docs/manual/fr/user-manual.tex` l. 794 et 845 écrit
  « \emph{Paramètres} → \emph{Exercices comptables} », alors que le menu range l'écran dans le groupe **Administration**
  (`frontend/src/routes/(app)/+layout.svelte`, groupe `nav-administration`, item `nav-fiscal-years`). Vérifié par
  l'orchestrateur. C'est la famille de #585 (Story 30-3), que la garde `les_chemins_cites_aux_catalogues_existent` ne voit
  pas : elle contrôle que chaque libellé existe, non qu'il appartienne au groupe cité.
- **Retenu** : la 31-4, qui renvoie justement à ces pages du manuel utilisateur, corrige les deux occurrences (et toute
  autre « Paramètres → <item du groupe Administration> » que le grep par la forme rendrait), PDF régénéré. Correction
  mécanique (budget d'issues, `CLAUDE.md` § *Priorités des défauts*). Une garde de hiérarchie n'est **pas** ajoutée ici.
- **Écartée** : ouvrir une issue — le compte est à 137 pour un plafond de « moins de 136 » au tag.
- **Réversible** : oui.
- *(Élargi et précisé en remédiation P2, 2026-10-24 — R6.)* Le même chemin s'écrit dans le **manuel d'administration**,
  `admin-manual.tex` l. 1445 et 1451 : la story le corrige aussi (AC13) ; `grep -nE 'Paramètres[^.]{0,60}Exercices'
  docs/manual/fr/*.tex` rend ces quatre lignes et elles seules ; T4 régénère et contrôle `user-manual.pdf` et
  `admin-manual.pdf` (grep à zéro résultat sur le PDF aplati). **Précision de fait, à porter à l'orchestrateur** :
  l'ancien chemin n'est pas une impasse — la page *Paramètres* (`/settings`, rangée sous *Administration*) porte une carte
  *Exercices comptables* et son bouton *Gérer* (`routes/(app)/settings/+page.svelte` l. 582-590, Story 3.7). Le défaut
  est l'écart entre deux chemins pour un même écran, celui des quatre lignes n'étant pas l'entrée du menu ; la
  correction reste juste. Les chemins `Paramètres → Modèles d'e-mail`, `→ Clés API`… désignent des cartes de cette page
  et ne sont pas touchés. Entrée `Corrigé` au CHANGELOG (AC12).

### C-31-4-14 — Validation P2 : la garde G22 réduite, le dossier de la copie fixé hors de `$HOME` (choix de l'orchestrateur, 2026-10-24)

- **Contexte** : la validation P2 (Sonnet) rend 5 MEDIUM (R) et 6 MEDIUM (F) ; les deux lentilles convergent sur la garde
  G22 telle que la remédiation P1 l'a étendue (lecture « par position », seuil de 60 lignes et 8 commandes, critère (e)
  « chaque `docker compose` porte `-p` et `--project-directory` ») : `manuel::normaliser` aplatit les lignes, (c)/(d)/(e)
  deviennent indécidables, (e) rougirait sur `docker compose ls`/`version`. C'est le même motif que le garde de la 31-2
  (C-31-2-15) : une garde lexicale qu'on étend pour qu'elle « prouve » la procédure finit par ne rien prouver.
  Et le contrôle 3 (`findmnt`) ne peut pas reconnaître un dossier synchronisé par Synology Drive Client (dossier local).
- **Retenu** :
  1. **G22 se réduit à des présences**, lues dans le source brut de la sous-section (`manuel::section`, avant
     normalisation) : les jetons `-p kesh-repetition`, `docker compose config`, `--services`, `/health`, la garde du
     `rm -rf`, et l'**absence** de toute ligne active qui assigne une variable `KESH_*_HOST_DIR`. Plus d'ordre par position,
     plus de seuils, plus de critère (e). Ce que G22 ne prouve pas — l'ordre et la justesse des gestes — est porté par le
     **journal d'exécution réelle** (T3, T8), et la fiche le dit.
  2. **Le dossier de la copie vit sous `/var/tmp/kesh-repetition-<date>`**, créé par `mkdir` **sans** `-p` (échec s'il
     existe), hors de `$HOME` — donc hors de tout dossier que Synology Drive Client synchronise ; le contrôle 3 vérifie
     ce préfixe et l'absence de lien symbolique (`test ! -L`), et la garde du `rm -rf` compare le `realpath` à ce préfixe.
     Les téléchargements du navigateur y sont déplacés avant usage.
- **Écartées** : poursuivre l'extension de G22 (chaque passe en trouve le défaut suivant) ; détecter la synchronisation
  (aucun moyen fiable).
- **Réversible** : oui.
- *(Précisé au report dans la fiche, remédiation P2, 2026-10-24.)* (1) Le jeton littéral `docker compose config` ne
  peut pas figurer dans un manuel conforme au *principe directeur* (`-p` et `--project-directory` s'intercalent entre
  `compose` et `config`) : G22 exige `config --services` à sa place, avec `-p kesh-repetition`, `/health`, et pour la
  garde du `rm -rf` les jetons `realpath -e`, `test ! -L` et `/var/tmp`. Les jetons tiennent dans une constante unique
  et un test les retire un à un **en mémoire** (R11). Les commentaires LaTeX sont retirés avant la lecture
  (`manuel::sans_commentaires`, F1). Le motif d'absence est non ancré (`export KESH_…_HOST_DIR=` vu, R3).
  (2) Le dossier se crée par `mkdir -m 0700` : `/var/tmp` est partagé entre utilisateurs et la copie porte les données
  réelles. La garde du `rm -rf` compare `dirname` à `/var/tmp`, le nom au préfixe, exige `test -d`, `test ! -L` et
  l'égalité du `realpath -e` au chemin écrit (F5, R8) ; la vacuité se vérifie au contrôle 3, avant le premier fichier
  posé (F11). (3) Mutations retirées : M2 (ordre), M8 (critère (e)), M9 (jeton « étapes 2 à 5 », non retenu par ce
  choix ; le renvoi se contrôle au PDF) ; ajoutées : M12 à M16.

### C-31-4-15 — Signal de recyclage en validation P3 : retrait de l'échafaudage, la procédure seule (choix de l'orchestrateur, 2026-10-24)

- **Contexte** : P1 Opus 0/1/19 MEDIUM → P2 Sonnet 0/0/11 → P3 Opus R 0/0/7, F 0/1/3 : la sévérité ne baisse plus, et la
  grande majorité des findings de P2 et P3 portent sur ce que les remédiations ont **ajouté autour** de la procédure — le
  script de fixture et son test shell, l'enveloppe Rust qui le lance, la garde G22, leurs mutations (le HIGH de P3 : la
  fixture ne peut pas figer un PDF sur le preset de test). C'est le **recyclage** du `CLAUDE.md` (§ *Règle de splitting*,
  amendement D5) ; le signal est **déclaré au Project Lead** dans le compte rendu de l'orchestrateur. La 31-4 est une story
  de **documentation** : son livrable est une procédure **exécutée**, non un outillage.
- **Retenu — retrait** :
  1. **Plus de script de fixture, ni de test shell, ni d'enveloppe Rust, ni de garde G22** (G19, existante, suffit pour
     les URL). La sauvegarde synthétique est **exportée par l'écran** (*Administration* → sauvegarde) depuis une instance
     de **démonstration** de la station (« Charger la démo »), au format 2 ; le journal d'exécution dit combien de pièces
     elle porte — aucune exigence de PDF figé.
  2. **La justesse de la procédure est portée par son exécution réelle**, journalisée hors dépôt (T3, T8), et par la
     relecture du manuel (`.tex` et PDF) en revue de code.
  3. **Restent exigés**, parce qu'ils protègent la production : la garde du `rm -rf` écrite exactement (`case … in
     kesh-repetition-*) ;; *) false ;; esac` dans le `if`, `test -d`, `test ! -L`, `realpath -e`, parent `/var/tmp`), essai
     avec un **nom faux** ; le contrôle 8 borné au bloc `ports:` de `kesh-api` (le `0.0.0.0` de `KESH_HOST` n'est pas une
     publication) ; tous les fichiers sensibles (exemplaire `.keshbackup`, exports de souveraineté de la production et de la
     copie, téléchargements) **déplacés dans `$COPIE`** et nommés à la destruction ; l'**arrêt** défini (destruction de
     l'AC7, puis reprise au contrôle 1) et l'exemplaire réel posé **après** le contrôle 9.
  4. **C-31-4-13** : les chemins « Paramètres → Exercices comptables » de l'**application** (messages ×4, `notify.ts`,
     `opening-balances`) restent valides — la page *Paramètres* porte une carte *Exercices comptables* — et sont écrits
     **hors périmètre** ; seuls les manuels sont corrigés, et l'entrée `Corrigé` du CHANGELOG dit « les manuels ».
- **Écartées** : redécouper la 31-4 (le recyclage est confiné à l'échafaudage) ; poursuivre (chaque passe trouvait le
  défaut suivant de ce qu'on venait d'ajouter).
- **Réversible** : oui — une fixture scriptée pourra revenir avec sa propre story.

### C-31-4-16 — Validation P3 remédiée : ce que le retrait de C-31-4-15 emporte, et ce qui reste (agent de remédiation, 2026-10-24)

- **Contexte** : C-31-4-15 retire l'échafaudage. Il en découle que C-31-4-2 (script de fixture), C-31-4-7 (G22), C-31-4-11
  (arguments et refus de la fixture) et le point 1 de C-31-4-14 (G22 réduite) sont **sans objet** ; ils restent écrits
  ci-dessus comme relevés datés, non réécrits.
- **Retenu** :
  1. **Source de la sauvegarde** : une instance de démonstration ordinaire (assistant, étape *démonstration*), exportée par
     l'écran. Ce que la démonstration porte n'est pas établi à la fiche : le développeur le relève au journal (T1) et saisit
     à l'écran une régularisation 1300/2300 si elle manque. Aucune exigence de PDF ni de nombre de pièces ; le critère de
     l'import devient « 0 pièce manquante ».
  2. **Les gardes qui restent** relisent la sous-section sans qu'on les étende : G19 (M5), G14, G15, G16 (T5).
  3. **Aucune dette nouvelle** : les dix sites de l'application (C-31-4-15, point 4) sont déjà nommés ; aucune issue
     n'est demandée (budget d'issues, C-31-4-13), le résidu est écrit en *Hors périmètre* et à l'AC13.
- **Écartées** : conserver M12/M13 réécrites en « toutes les occurrences » (elles protégeaient une garde retirée) ;
  rattacher une issue P4 aux dix sites (le compte des issues ne baisse pas).
- **Réversible** : oui.

### C-31-4-17 — Validation P4 remédiée : la source de la sauvegarde se prépare à l'écran, et rien de sensible ne précède `$COPIE` (agent de remédiation, 2026-10-24)

- **Contexte** : P4 (Sonnet), R 0/0/3/5 et F 0/0/1/6, aucun finding réfuté. Trois constats de fond, tous nés de C-31-4-15/16
  ou de leur lecture. (1) `kesh_seed::seed_demo` (`crates/kesh-seed/src/lib.rs` l. 139-143, 272-296) ne pose que la société,
  le plan PME, **un exercice**, les réglages de facturation et les taux de TVA : ni écriture, ni exercice suivant, ni pièce ;
  C-31-4-16 point 1 (« 0 pièce manquante », *N* possiblement nul) rendait le critère de l'AC4 **vrai par construction** et
  la source inapte au geste 2 de l'AC5. (2) L'« arrêt » sur le reste d'une répétition antérieure visait un `$COPIE` qui
  n'existe pas encore. (3) L'étape 0 déposait le `.keshbackup` de la production et l'export de souveraineté dans le
  dossier de téléchargements **avant** tout contrôle.
- **Retenu** :
  1. **T1 prescrit des gestes à l'écran, journalisés**, sur l'instance de démonstration avant l'export : exercice suivant ;
     écriture 1300/2300 du dernier jour de l'exercice courant ; au moins une pièce (fixture synthétique du dépôt
     `spc_e2e_invoice.png`, déposée dans le dossier d'import de l'instance source puis *Documents à traiter → Importer le
     dossier*) ; PDF figé d'une facture **si l'écran le permet**, sinon « non éprouvé ». **Critère de l'AC4 : *N* ≥ 1 et 0
     manquante** ; *N* = 0 est un échec de préparation de la source. Le renvoi aux étapes 2 à 5 de *Restaurer sur une
     nouvelle machine* dit lesquelles de leurs vérifications s'appliquent. Cela **remplace le point 1 de C-31-4-16**
     (l'entrée reste écrite, non réécrite).
  2. **L'étape 0 se joue après les contrôles 1 à 3** et enregistre ses fichiers directement dans
     `$COPIE/telechargements/` (créé par `mkdir -m 0700`, sans `-p`, après la vérification de vacuité). Un arrêt aux
     contrôles 1 ou 2 n'a donc rien à nettoyer ; après l'étape 0, l'arrêt retire à la main un exemplaire atterri dans le
     dossier du navigateur (repli du constat (s)).
  3. **L'arrêt sur reste d'une répétition antérieure** a un chemin écrit : dossier existant → `COPIE` y est affectée et
     (b) s'applique avec sa garde ; dossier disparu → `docker rm -f` / `volume rm` / `network rm` appliqués aux seuls
     identifiants rendus par les filtres d'étiquette, éprouvé **à blanc** (constat (u)), exécution réelle « non éprouvée »
     (AC10).
  4. LOW : R4 (`mkdir` de `telechargements`), R5 (contrôle 1 seule issue corrective), R6 (un seul `if` pour les deux
     gestes sans `sudo` ; deux refus de plus constatés à l'essai (iii)), R7 (annotation datée au Change Log), R8 (trois
     variables `DOCKER_*` de plus), F2 (`COPIE` rétablie après chaque cas), F3 (port ≠ 80, > 1023), F4 (ôter le `#`
     des deux `MARIADB_*`), F5 (machine Linux), F6 (en-tête lu à l'onglet Réseau, à constater), F7 (commandes en
     listing).
- **Écartées** : un script de remise à zéro des restes (C-31-4-15 : aucun outillage) ; un `docker compose -p kesh-repetition
  down -v` sans `-f` (le geste que le principe directeur s'interdit ; à constater peut-être en T3, non autorisé ici) ;
  `prune`.
- **Réversible** : oui.

### C-31-4-18 — Validation P5 remédiée : R3 propagé jusqu'au bout, et le critère du manuel n'est pas celui du développement (agent de remédiation, 2026-10-24)

- **Contexte** : P5 (Opus, passe ciblée sur le seul commit de la remédiation P4), T 0/0/2/5, aucun finding réfuté. (1) R3
  (étape 0 après le contrôle 3, enregistrement direct dans `$COPIE/telechargements/`) laissait trois énoncés à l'ancien
  état : la parenthèse « aucun secret réel dans `/var/tmp` » (fausse depuis R3), l'exemplaire « créé hors de `$COPIE` »,
  le « déplacé » du contrôle 3. (2) C-31-4-17 avait écrit à l'AC4, qui décrit **le manuel**, un critère (*N* ≥ 1, échec de
  préparation, renvois à la pièce et au PDF de la T1) qui ne vaut que pour la sauvegarde synthétique : une production sans
  pièce y aurait constaté un « échec », et la vérification des écritures et des factures de l'étape 5 avait disparu.
- **Retenu** :
  1. Le secret réel est dans `/var/tmp` dès l'étape 0 ; la garantie est la **destruction de `$COPIE` à tout arrêt**
     (AC7 (d)). `source.keshbackup` se pose par `mv` depuis `$COPIE/telechargements/` après le contrôle 9, puis `chmod
     0600`. Le « hors de `$COPIE` » ne reste que pour le repli du constat (s).
  2. **Deux critères séparés** : au manuel, 0 pièce manquante et autant de pièces restaurées que l'export en a annoncé,
     plus les vérifications de l'étape 5 (écriture récente, liste des factures) ; au développement (T1, Dev Notes), *N*
     ≥ 1. Cela **précise** le point 1 de C-31-4-17 (l'entrée reste écrite, non réécrite).
  3. LOW T3 à T7, mécaniques (voir le Change Log) ; le repli (i) → (ii) est écrit et (i) rangé parmi les « non
     éprouvés » de l'AC10.
- **Écartées** : une garde lexicale sur l'ordre des étapes (C-31-4-15 : aucun outillage) ; abandonner l'égalité avec le
  compte de l'export, plus forte que *N* ≥ 1 et valable dans les deux cas (une lecture de deux nombres à l'écran).
- **Réversible** : oui.

### C-31-4-19 — Étape 0 du développement : le prérequis « exporte en format 2 » se lit sans rien télécharger (agent de développement, 2026-10-24)

- **Contexte** : la P8 (validation terminée au plafond) a laissé deux MEDIUM corrigés par l'orchestrateur sans passe de
  vérification. Recontrôle : la correction de M2 proposait de vérifier le prérequis par « la version affichée » (fausse au
  développement : l'image de `main` affiche `0.14.0` et exporte en format 2) ou par « un export d'essai » (il déposerait
  un secret hors de `$COPIE`, avant le contrôle 1).
- **Retenu** : le prérequis se lit sur la page *Administration → Sauvegarde complète* de la production, dont la
  description nomme « les pièces justificatives » depuis la 31-1a (absente à la 0.14.0, vérifié par `git show v0.14.0`).
  Le nombre de pièces annoncé se lit à l'onglet Réseau (`X-Kesh-Backup-Files`) ou dans `manifest.json` : l'écran ne
  l'affiche pas. LOW mécaniques des P6 et P8 appliqués ; le reste consigné au Dev Agent Record.
- **Écartées** : la version (≥ 0.15.0) comme seul critère (imposerait une substitution de plus au développement, et rien
  ne la lit sans `/health`) ; l'export d'essai.
- **Réversible** : oui (texte de fiche et de manuel).

### C-31-4-20 — Développement : ce que la répétition réelle a changé à la procédure (agent de développement, 2026-10-24)

- **Contexte** : la procédure jouée réellement (constats à blanc, répétition, second passage depuis le PDF) a contredit la
  fiche en six points (Dev Agent Record, *Développement*).
- **Retenu** :
  1. contrôle 1 étendu à `DATABASE_URL` (exportée par le profil du shell de la station ; le compose Synology la lit) ;
     filtre écrit en deux lignes (`COMPOSE_|KESH_|MARIADB_|DATABASE_URL=` puis `DOCKER_…`) pour tenir dans la page ;
  2. contrôles 7 et 8 lus en JSON (`config --format json | jq`), override en YAML « en ligne » : le texte d'un PDF ne
     conserve pas les espaces ; `jq`, `unzip`, `curl`, `openssl` contrôlés au contrôle 1 ;
  3. boucle d'attente de `/health` après `up -d` ;
  4. nombre de pièces : onglet Réseau, ou `unzip -l … | grep -cE ' files/[^ ]+$'` ; exports de souveraineté nommés
     `souverainete-production.zip` / `souverainete-copie.zip` et comparés **table par table** (le SHA-256 global diffère
     par construction) ; la répétition ne prétend pas remplir l'étape 3 du test périodique à la lettre ;
  5. pilotage de l'écran par Chromium de Playwright (Firefox snap non pilotable ; son accès à `/var/tmp` par le portail
     « non éprouvé ») ; facture de la démonstration validée à 0 % ;
  6. listings de la sous-section en `\footnotesize`, dans un groupe local.
- **Écartées** : ajouter une garde lexicale (C-31-4-15) ; corriger l'étape 3 du test périodique dans cette story (défaut
  voisin, non mécanique : issue demandée) ; régler le compte de TVA de la démonstration (code du produit : issue demandée).
- **Réversible** : oui (texte du manuel).

### C-31-4-21 — Revue de code P1 : le commit `9c59dec4` retiré, l'étape 3 du test périodique renvoyée à #709 (orchestrateur, 2026-10-24)

- **Contexte** : `9c59dec4` réécrivait l'étape 3 de `sec:test-restauration` (comparaison table par table au lieu du
  SHA-256 de l'archive) et ajoutait une seconde entrée « Corrigé » au CHANGELOG — exactement ce que C-31-4-20 écartait
  (défaut voisin non mécanique). La lentille A (MEDIUM-1 à 3) l'a relevé : contradiction avec la fiche, l'AC6 et l'AC12,
  gate complet antérieur au commit, critère d'un contrôle légal (OLICo art. 10 al. 1) relâché sur la foi d'une seule
  exécution de démonstration.
- **Retenu** : retrait par `728012ba` (revert) ; le point devient l'**issue #709** (étape 3, SHA-256 inatteignable).
  Les findings A-1, A-2, A-3, E-3 (MEDIUM-3) et la partie de B-9 qui visaient ce commit sont **sans objet**. Le manuel
  revient à la rédaction de C-31-4-20 : la répétition ne prétend pas remplir l'étape 3 à la lettre.
- **Écartées** : garder le commit et amender fiche, AC et registre (l'arbitrage du critère revient à Guy, non à une passe).
- **Réversible** : oui (texte du manuel ; #709 tranche).
- *Annotation datée (revue de code P2, 2026-10-24, A-M3)* : « sans objet » était faux pour la dernière puce d'A-1 — les
  phrases « elle peut tenir lieu de ce test » (`sec:test-restauration`) et « La répétition tient lieu du test … seulement
  si » (procès-verbal) ne venaient pas de `9c59dec4` et sont restées après le retrait. Corrigé par C-31-4-25.

### C-31-4-22 — Revue de code P1 remédiée : les variables lues dans le fichier, les comparaisons qui ne se taisent plus, les cibles nommées (agent de remédiation, 2026-10-24)

- **Contexte** : P1 (Sonnet, lentilles B, E, A), hors `9c59dec4` : quatre MEDIUM retenus (E-1/B-3, E-2/B-2, B-1, E-4).
- **Retenu** :
  1. **Variables interpolées : l'ensemble clos, lu dans le fichier.** Au tag `v0.14.0`, `git show v0.14.0:docker-compose.yml
     | grep -oE '\$\{[A-Za-z_]+' | sort -u` rend 45 noms : 39 `KESH_*`, 4 `MARIADB_*`, `RUST_LOG`, et `NOM` (dans un
     commentaire, l. 108) ; aucune variable sans accolades, aucune entrée `environment:` sans valeur (qui transmettrait la
     valeur du terminal), pas d'`env_file`. Seule `RUST_LOG` échappait au filtre. Le filtre du contrôle 1 (et du contrôle
     final) gagne `RUST_LOG=` ; surtout, le contrôle 4 relève, **dans le compose téléchargé**, chaque `${NOM` / `$NOM`
     et affiche ceux que le terminal exporte (`printenv`) — attendu : rien. Ce n'est plus une énumération de préfixes,
     c'est l'inventaire des sites du fichier même, valable pour le compose de la 0.15.0 et des suivantes. Essayé sur une
     copie synthétique du compose du tag (`env -i … RUST_LOG=debug KESH_LANG=fr`) : `posée : KESH_LANG`, `posée : RUST_LOG`.
  2. **Comparaison table par table** : la boucle écrit `production : N tables` / `copie : N tables` par `test -s` et
     `wc -l` ; une ligne absente = fichier d'empreintes vide = comparaison **non faite** (le `diff` de deux fichiers vides
     se tait) ; deux `N` différents = exports de tables différentes. Essayé sur deux ZIP synthétiques : cas normal (deux
     lignes, une différence `audit_log.csv`) et archive de la copie absente (une seule ligne).
  3. **Cibles nommées** : l'étape 0 nomme le `.keshbackup` `production.keshbackup` (commandes de comptage et de pose
     reprises) ; les replis `mv` nomment leur cible (`souverainete-production.zip`, `production.keshbackup`,
     `souverainete-copie.zip`) — le navigateur propose le même nom aux deux exports de souveraineté (journal T8). `mv -n`
     écarté : sous coreutils 9.7, il ne déplace pas et rend 0 sans rien dire (essayé).
  4. **Étape 0 ordonnée** : l'export de souveraineté puis, aussitôt, le `.keshbackup`, sans écriture entre eux ; une
     écriture intercalée apparaît comme la différence d'une autre table qu'`audit_log.csv` (dit aux deux endroits).
  5. LOW traités : « Arrêt » renvoie à *En cas d'arrêt*, reprise avec un nouveau dossier (date ou suffixe `-2`) (B-4,
     E LOW-1) ; trois listings séparés pour la destruction — sans `sudo`, avec `sudo`, `rm -rf` gardé seul avant le premier
     démarrage —, « un seul se tape » (B-5, L5) ; exemples `0.14.0` déclarés tels face au prérequis 0.15.0 — l'URL reste au
     tag `v0.14.0`, G19 l'exige (B-8) ; volume et réseau nommés (L1) ; espace disque renvoyé à sa source, « jusqu'à
     environ trois fois » dans `/var/tmp`, décompte corrigé (L3) ; « sept formes » (L4) ; « suivez chaque lien » (L7) ;
     « ne modifie aucune donnée » au lieu de « ne touche jamais » (B-6).
- **Écartées** : les LOW consignés au Change Log (B-7, B-10, E LOW-2 à LOW-5, A L2, L6).
- **Réversible** : oui (texte du manuel).

### C-31-4-23 — Revue de code P2 : l'encadré dit ce que la répétition réelle n'a pas joué (agent de remédiation, 2026-10-24)

- **Contexte** : B2-M1 = A-M1 (même défaut) — l'encadré « jouée en entier », « vus refuser », et le CHANGELOG « de bout en
  bout », alors que les commandes de `6186edfc` ne figurent dans aucun des trois journaux de la répétition.
- **Retenu** : retirer la promesse, ne pas rejouer (consigne de l'orchestrateur : aucune répétition Docker). L'encadré
  dit « jouée, dans une rédaction antérieure à celle-ci », et une puce nouvelle, en tête, nomme ce qui n'a jamais été joué
  dans une répétition réelle : l'inventaire du contrôle 4 et la boucle de comparaison (essayés sur fichiers synthétiques
  seulement, C-31-4-22) ; `RUST_LOG=`, les `mv` à cible nommée, `production.keshbackup`, et les ajouts de cette passe
  (export de la copie avant l'étape 5, `[ -n "$COPIE" ]` du contrôle final, la marche *À la destruction ou au contrôle
  final*) — sans « essayé », faute d'essai attesté. « vus refuser » est borné à « lors de cette exécution », l'inventaire
  du contrôle 4 dit non éprouvé. CHANGELOG nuancé dans le même sens.
- **Écartées** : rejouer les commandes en lecture seule sur la station (interdit par la consigne de la passe).
- **Réversible** : oui.

### C-31-4-24 — Revue de code P2 : la variable du terminal au contrôle 4 se rattache à l'exception du contrôle 1 (agent de remédiation, 2026-10-24)

- **Contexte** : A-M2 (= B2-L1) — l'introduction des contrôles et l'AC3 réservent l'issue corrective au contrôle 1 ; le
  contrôle 4 (`6186edfc`) en porte une seconde (`unset` et refaire).
- **Retenu** : rattacher le contrôle 4 à l'exception, en amendant l'AC3 (annotation datée) et l'introduction du manuel :
  « sauf une variable du terminal, aux contrôles 1 et 4 ». Justification : l'écart est **de même nature** que celui du
  contrôle 1 (une variable exportée), et Compose n'a encore rien lu — `unset` ne laisse aucune trace. En faire un arrêt
  détruirait `$COPIE` et ferait refaire les **deux exports de la production**, c'est-à-dire de nouvelles écritures au
  journal d'audit de la production, sans aucun gain de sûreté sur la copie. Le contrôle 4 dit aussi que c'est sa seule
  issue corrective : tout autre écart y reste un arrêt.
- **Écartée** : l'arrêt (plus coûteux, plus d'interactions avec la production, même résultat).
- **Réversible** : oui.

### C-31-4-25 — Revue de code P2 : la répétition ne remplace pas le test de restauration périodique (agent de remédiation, 2026-10-24)

- **Contexte** : A-M3 — l'AC6 interdit de prétendre que la répétition vaut ce test quand les empreintes ne peuvent pas
  être égales (constat (h)) ; le manuel disait « elle peut tenir lieu de ce test » et « tient lieu … seulement si ».
  C-31-4-21 l'avait classé sans objet à tort (annotation datée sous C-31-4-21).
- **Retenu** : les deux affirmations retirées ; le manuel écrit, à `sec:test-restauration`, à la comparaison et au
  procès-verbal, que la répétition **ne remplace pas** le test tant que son critère n'est pas tranché, avec renvoi en clair
  à l'issue #709 (sans mot-clé de fermeture). L'export de souveraineté de la production n'est plus conditionné à « tenir
  lieu du test » mais à la comparaison table par table, qui reste ce que la répétition établit. Le CHANGELOG le dit.
- **Écartée** : amender l'AC6 pour laisser « au lecteur de décider » (l'arbitrage du critère revient à Guy, #709).
- **Réversible** : oui (#709 tranche).

### C-31-4-26 — Revue de code P2 : l'export de la copie avant l'étape 5 de la restauration (agent de remédiation, 2026-10-24)

- **Contexte** : E-M1 — l'étape 5 de *Restaurer sur une nouvelle machine* fait télécharger le PDF d'une facture ; le
  premier téléchargement d'une facture validée non figée la fige. Vérifié au code : `invoice_pdf.rs` l. 9-12 (« le
  premier téléchargement écrit donc (fichier, colonnes, audit `invoice.pdf_frozen`) »), `invoices::freeze_pdf`
  (`crates/kesh-db/src/repositories/invoices.rs` l. 2737-2755 : `UPDATE invoices SET pdf_storage_path, pdf_sha256,
  pdf_frozen_at = NOW(3), pdf_language`), colonnes `pdf_sha256`, `pdf_frozen_at`, `pdf_language` d'`invoices.csv`
  (`csv_tables.rs` l. 539-541, 570-572). Fait avant l'export de la copie, ce téléchargement fait différer `invoices.csv`.
- **Retenu** : réordonner **et** nommer la cause. La restauration renvoie aux étapes 2 à 4 ; l'export de souveraineté de
  la copie se fait « aussitôt reconnecté », avant l'étape 5 et les gestes ; la comparaison se clôt par « Puis faites
  l'étape 5 ». La liste des causes d'écart nomme « une écriture dans la copie avant son export (un PDF de facture
  téléchargé avant lui : `invoices.csv` diffère) ». Côté production, l'étape 0 nomme le téléchargement d'un PDF jamais
  téléchargé parmi les écritures à éviter entre les deux exports (même mécanisme). AC4 et AC6 annotées.
- **Réversible** : oui.

### C-31-4-27 — Revue de code P2 : une destruction qui échoue a sa marche (agent de remédiation, 2026-10-24)

- **Contexte** : E-M2 — rien n'était prévu si `down -v` ou le `docker run` échoue, ni si le contrôle final ne rend pas
  « rien » ; le volume restant porte la comptabilité restaurée.
- **Retenu** : une entrée *À la destruction ou au contrôle final* dans *En cas d'arrêt* : ne pas s'arrêter là ; lever la
  cause visible ; lister par les trois filtres d'étiquette du contrôle 2 ; retirer par la boucle du cas *Reste*
  (identifiants rendus par les filtres, jamais un nom écrit à la main, jamais `prune`) ; le dossier par la garde (premier
  ou deuxième listing) ; rejouer le contrôle final. S'il rend encore des lignes : la copie n'est pas détruite, le noter au
  procès-verbal, aucune autre répétition sur la machine, reprendre dès la cause levée. Le cas *Reste* (ii) nomme le
  listing (premier ou deuxième, l'ancienne copie ayant pu démarrer) et finit par le contrôle final (E LOW-5, traité en
  passant).
- **Réversible** : oui.
- *(Annotation du 2026-10-24, revue de code P3, E M-1 = B3-1 : le critère « s'il rend encore des lignes » était faux
  pour le cas où seul le dossier survit — aucune ligne ne s'affichait. Remplacé par C-31-4-28.)*

### C-31-4-28 — Revue de code P3 : un contrôle final décidable (agent de remédiation, 2026-10-24)

- **Contexte** : E M-1 = B3-1 (origine : remédiation P2, `[ -n "$COPIE" ] && test ! -e …`). Un `COPIE` vide ou non
  posée (terminal neuf), ou un dossier qui survit, ne rendaient **aucune** ligne, comme un succès ; l'« Attendu : rien,
  sauf `copie détruite` » et le critère « s'il rend encore des lignes » de la marche *À la destruction ou au contrôle
  final* concluaient alors à tort à la destruction.
- **Retenu** : la dernière commande du contrôle final devient un `if … elif … else … fi` à quatre issues, dont une seule
  de succès : `échec : COPIE n'est pas posée` (`-z`) ; `échec : COPIE n'est pas le chemin de la copie` (même `dirname`
  et même `case … *) false ;; esac` que la garde du `rm -rf`, nié par `!`) ; `échec : … existe encore` (`-e` **ou**
  `-L`, un lien pendant compte comme un reste) ; `copie détruite`. L'« Attendu » dit que cette ligne est **requise** et
  que son absence, comme une ligne `échec :`, est un échec ; la marche *À la destruction…* dit « une ligne des filtres,
  une ligne `échec :`, ou rien du tout », puis « tant qu'il ne rend pas la seule ligne `copie détruite` ». E L-1 traité
  avec : un terminal neuf repose `COPIE` (nom daté, `ls -d /var/tmp/kesh-repetition-*`) et refait le contrôle 1 avant
  tout `docker run` ; le cas *Reste* dit quel chemin poser quand le dossier a disparu. Essayé sur onze formes de `COPIE`
  sous un dossier synthétique (`/home/gcorbaz/devel/kesh-gate-logs/314-p3-essais/`, `/var/tmp` remplacé par lui, détruit
  ensuite) : vide, non posée, dossier présent, avec barre finale, lien, lien pendant, autre nom, relatif, le dossier
  synthétique lui-même, dossier absent, et dossier détruit → seules les deux formes absentes rendent `copie détruite` ;
  `bash -n` et `dash` acceptent `! case … esac`. Jamais joué dans une répétition réelle : l'encadré le dit.
- **Écartée** : garder `test ! -e` et n'écrire que la phrase « l'absence de la ligne est un échec » (B3-1, remède d'une
  phrase) — la sortie vide resterait la même pour trois situations distinctes, et le lecteur devrait se souvenir de la
  phrase pour la juger.
- **Réversible** : oui.

### C-31-4-29 — Revue de code P3 : ce que les exports écrivent dans la production (agent de remédiation, 2026-10-24)

- **Contexte** : E L-5 — « la production n'en garde que la trace … au journal d'audit » est inexact à la lettre.
  Vérifié au code : `crates/kesh-api/src/exports/global.rs` l. 304 et 313 appellent
  `company_invoice_settings::get_or_create_default` et `company_dunning_settings::get_or_create_default`, dont
  l'`INSERT IGNORE` (`company_invoice_settings.rs` l. 75, `company_dunning_settings.rs` l. 52) crée la ligne de réglages
  si elle manque ; les pages de ces réglages font de même à leur première ouverture (`get_or_create_default` dans
  `routes/company_invoice_settings.rs`, `ensure_seeded_in_tx` dans `routes/company_dunning_settings.rs`).
- **Retenu** : *Ce qu'elle ne fait pas* dit « aucune donnée **comptable** », et nomme ce cas : si ces réglages n'ont jamais
  été enregistrés, l'export de souveraineté en crée les lignes, aux valeurs par défaut, comme la première ouverture de
  leur page. Sans effet sur la comparaison (ligne créée avant le `.keshbackup`, ordre de l'étape 0).
- **Réversible** : oui.

---

## Versé par union : `epic-31-choix-autonomes-315.md`

*(Versé le 2026-10-24 par la Story 32-0 — action A2 de la rétrospective de l'Epic 31. Contenu recopié sans retouche depuis la deuxième ligne du registre de la story, qui reste en place et auquel renvoient la fiche et `sprint-status.yaml`.)*

Versé par union dans `epic-31-choix-autonomes.md` au merge.

## Story 31-5

### C-31-5-0 — Emplacement de ce registre

- **Contexte** : le prompt de l'orchestrateur nomme `_bmad-output/planning-artifacts/epic-31-choix-autonomes-315.md` ;
  les consignes communes (`docs/agents-orchestration.md` § 1) et tous les registres de l'epic (`-311`, `-312`, `-313`)
  sont dans `_bmad-output/implementation-artifacts/`.
- **Retenu** : `implementation-artifacts/`, à côté des autres, pour que l'union se fasse au même endroit.
- **Écartée** : `planning-artifacts/` — un registre isolé que l'union ne verrait pas.
- **Réversible** : oui (un `git mv`).

### C-31-5-1 — Une seule story, quatre volets indépendants

- **Contexte** : cinq modules de premier niveau (`kesh-db`, `kesh-seed`, `kesh-api`, `kesh-i18n`, `frontend`) ; la règle
  de découpage préventif découpe **au-delà** de cinq.
- **Retenu** : une fiche, quatre volets A-D sans fichier de code commun (seuls les catalogues `kesh-i18n` sont partagés
  par C et D, en lignes distinctes).
- **Écartées** : quatre stories (quatre boucles de validation et de revue pour des correctifs d'une à six lignes de
  production) ; deux stories frontend/backend (découpage sans rapport avec les défauts).
- **Réversible** : oui — un volet qui résiste en revue se détache en story propre sans toucher les autres.

### C-31-5-2 — #687 et #645 écartées

- **Contexte** : les deux sont réelles. #687 : l'issue ne propose « aucune [piste] mécanique » (arrondi à la création
  qui change HT/TVA et la complétion d'une pièce importée, ou refonte de `pay_in_tx`). #645 : définir les jetons change
  l'apparence de tout l'écran et l'accessibilité, l'issue demande une story propre.
- **Retenu** : écartées, restent ouvertes ; #687 est la plus utile à Guy des deux (factures fournisseur réelles) et
  mérite une story à elle.
- **Écartée** : forcer quatre parmi les six en y mettant #687 à la place de #504 ou #469 — ce serait prendre la moins
  courte et la plus risquée pour tenir un chiffre.
- **Réversible** : oui.

### C-31-5-3 — #707 : corriger la démonstration, pas le chemin production

- **Contexte** : `insert_with_defaults_in_tx` sert la démonstration **et** la finalisation production ; aucun des deux ne
  désigne les comptes de TVA (colonnes nullables depuis la 18-1a, `18-1a-comptes-tva.md:109-110` ; refus de validation
  sans TVA due décidé par la 18-1b, `18-1b-comptabilisation-ventes.md:122-125`, AC5 — la désignation par
  l'administrateur en est la conséquence ; « voulu depuis la 18-1a » jusqu'à la validation P3, F5). Pour un non-assujetti (Guy), le refus `CONFIGURATION_REQUIRED`
  d'une facture saisie par erreur avec un taux est un garde-fou.
- **Retenu** : une fonction distincte appelée par `seed_demo` seulement (deux depuis la validation P1, C-31-5-13 : la lecture verrouillante des comptes, puis l'application) ; le chemin production garde `NULL`, et le manuel
  le dit (AC3) ; un test fige ce comportement production.
- **Écartées** : désigner les comptes dans `insert_with_defaults_in_tx` (change le comportement de toute installation
  neuve, retire le garde-fou du non-assujetti) ; manuel seul pour la démonstration (l'issue admet cette voie, mais elle
  attend de la démonstration un essai sans réglage — formule de l'issue, non du produit, annotation de la validation
  P2 —, et le correctif est court).
- **Réversible** : oui.

### C-31-5-4 — #707 : n'agir que sur une ligne de réglages née dans la transaction

- **Contexte** : `insert_with_defaults_in_tx` rend `inserted` ; si la ligne existait (`false`), elle a pu être lue et
  modifiée par quelqu'un, et une mise à jour sans `version + 1` casserait le verrou optimiste.
- **Retenu** : appel conditionné à `invoice_settings_created` ; `COALESCE` en plus (aucun écrasement). Mutation M4
  probablement équivalente, à déclarer.
- **Écartée** : mettre à jour toujours, avec `version + 1` — gère un cas que la démonstration (étape 2 → 3) ne produit
  pas.
- **Réversible** : oui.

### C-31-5-5 — #707 : l'entrée d'audit `installation.demo_seeded` reste telle quelle

- **Contexte** : elle porte `invoice_settings_created`, pas le détail des comptes.
- **Retenu** : inchangée (périmètre minimal).
- **Écartée** : y ajouter les trois comptes — utile mais hors du défaut.
- **Réversible** : oui.

### C-31-5-6 — #589 : la capture montre l'écran, la valeur de l'infobulle se lit au DOM

- **Contexte** : la recette visuelle exige une capture de tout contrôle visible touché ; une infobulle native `title`
  n'apparaît pas dans une capture Playwright.
- **Retenu** : capture de l'écran à trois exercices clos (boutons désactivés/actif) + valeur de `title` relevée au DOM et
  recopiée au Dev Agent Record ; Vitest l'assure.
- **Écartée** : remplacer l'infobulle native par un composant d'infobulle pour pouvoir la capturer — change le contrôle.
- **Réversible** : oui.

### C-31-5-7 — #504 : la traduction ne touche pas le rapprochement

- **Contexte** : l'issue demande de vérifier qu'aucun rapprochement ne s'appuie sur « Facture ».
- **Retenu** : établi au code — `reference_score` (`kesh-reconciliation/src/matching.rs:231-256`) compare le numéro de
  facture normalisé par contenance ; le préfixe n'y entre pas. Aucun test de rapprochement à ajouter.
- **Écartée** : garder le français « pour la banque » — la communication est lue par le débiteur, dans sa langue.
- **Réversible** : oui.

### C-31-5-8 — #504 : une clé dédiée, sans variable

- **Contexte** : réutiliser `invoice-pdf-title` (même valeur aujourd'hui) couplerait le texte lu par la banque au titre
  imprimé ; une clé `Facture { $number }` ferait entrer les marques d'isolation Fluent (U+2068/U+2069) dans la QR,
  aucun `set_use_isolating(false)` n'étant posé.
- **Retenu** : `invoice-qr-message-prefix` sans placeable, `format!("{prefix} {n}")` côté Rust, dans `kesh-api` (pas dans
  `I18N_KEYS` de `kesh-qrbill`, qui n'est pas touché).
- **Écartées** : `invoice-pdf-title` ; clé à variable ; clé ajoutée à `kesh-qrbill` (un sixième module pour rien).
- **Réversible** : oui.

### C-31-5-9 — #504 : pas de refigeage des PDF émis

- **Contexte** : depuis la 25-6-b, le PDF d'une facture est figé au premier rendu ; refigeage manuel seulement.
- **Retenu** : les PDF figés gardent leur communication française ; manuel le dit (AC10).
- **Écartée** : refigeage d'office (change une pièce émise, ce que la 25-6-b interdit).
- **Réversible** : sans objet.

### C-31-5-10 — #469 : le détail `chrono` d'une date mal formée n'est plus montré

- **Contexte** : `parse_date` renvoie aujourd'hui `{nom} invalide ({e})`, `e` étant le message anglais de `chrono`.
- **Retenu** : message traduit qui nomme le paramètre et le format attendu (`AAAA-MM-JJ`), sans `e`.
- **Écartée** : garder `e` en variable — un fragment anglais dans un message allemand ou italien.
- **Réversible** : oui.

### C-31-5-11 — #469 : `t_args` passe en `pub(crate)`

- **Contexte** : seule `t` est exposée ; `t_args` retire déjà les marques d'isolation.
- **Retenu** : élargir sa visibilité (DRY : pas de seconde fonction qui ferait la même chose dans `audit_log.rs`).
- **Écartée** : `format!` sur le rendu de `t` — perd l'ordre des mots propre à chaque langue.
- **Réversible** : oui.

### C-31-5-12 — Classe voisine non traitée : les autres `AppError::Validation` en français

- **Contexte** : des dizaines de routes rendent des `AppError::Validation` écrits en français ; #469 ne couvre que le
  journal d'audit et aucune issue ouverte ne couvre le reste.
- **Retenu** : hors périmètre ; issue demandée à l'orchestrateur (« on trace toujours »), même si elle fait remonter
  le compte — **ouverte le 2026-10-24 : #710** (annotation de la validation P1 ; comprise dans le recompte de 141).
- **Écartée** : les traduire ici — ni mécanique ni court (283 lignes hors du journal d'audit au relevé grossier du
  2026-10-24, `grep -rn "AppError::Validation(" crates | grep -v "t(" | grep -v audit_log.rs | wc -l` ; 289 sans le
  dernier filtre, chiffre écrit ici jusqu'à la validation P2, qui comptait les six d'`audit_log.rs`).
- **Réversible** : sans objet.

### C-31-5-13 — #707 : les comptes de TVA se verrouillent AVANT la ligne de réglages (validation P1, R4/F4)

- **Contexte** : la fonction prescrite à la rédaction verrouillait les trois comptes de TVA **après** l'`INSERT IGNORE`
  de `insert_with_defaults_in_tx`, d'où `accounts → company_invoice_settings → accounts`, contraire à l'ordre écrit
  pour `seed_demo` (`docs/MULTI-TENANT-SCOPING-PATTERNS.md:326`, doc de `final_body`). L'argument de la fiche (aucun
  concurrent sur une ligne neuve) ne couvrait pas une transaction tenant l'un des comptes.
- **Retenu** : deux fonctions — `lock_vat_role_accounts_in_tx` (lecture verrouillante des trois comptes, appelée avant
  `insert_with_defaults_in_tx` ; sa forme — une requête `IN (…) ORDER BY id FOR UPDATE` à la validation P1 — est
  remplacée par trois lectures ponctuelles à la validation P2, C-31-5-18) et `designate_vat_accounts_in_tx` (un `UPDATE ... COALESCE` de la
  ligne de réglages, sans lecture de compte, appelée après, si `invoice_settings_created`). L'ordre documenté est tenu
  sans exception ; la ligne 326 et le doc de `final_body` sont précisés (comptes de TVA inclus) dans la même PR.
- **Écartées** : garder l'ordre inverse et écrire l'exception (risque faible, mais une exception de plus au Pattern 5
  pour un gain nul) ; ajouter un paramètre à `insert_with_defaults_in_tx` (toucherait le chemin production, C-31-5-3).
- **Réversible** : oui.

### C-31-5-14 — Contrôles de l'AC16 décidables (validation P1, R1/F2/F3)

- **Contexte** : `grep -rn 'format!("Facture' crates` rend deux lignes, dont `supplier_invoices.rs:512` (libellé
  d'écriture de production), et les deux « exceptions » nommées (`:816`, `qr_decode.rs:225`) ne contiennent pas
  `format!` ; le contrôle d'`audit_log.rs` était ligne à ligne, faux dès que rustfmt met l'argument à la ligne suivante,
  et contredisait l'AC11.
- **Retenu** : (a) contrôle #504 = zéro ligne hors `crates/kesh-db/src/repositories/supplier_invoices.rs` **et** compte
  de `format!("Facture fournisseur` dans ce fichier égal à 1 — l'exception unique et nommée, avec sa raison ;
  (b) contrôle #469 multi-ligne par aplatissement (`tr '\n' ' '`, jeton qui suit `AppError::Validation(`), six jetons,
  tous `t`/`t_args` après correctif ; le **garde** reste le test de l'AC12, l'AC11 renvoie à l'AC16.
- **Écartées** : `grep -Pzo` (moins lisible, dépend de PCRE) ; supprimer le contrôle et ne garder que le test.
- **Réversible** : oui.

### C-31-5-15 — Les six clés du journal d'audit entrent à la garde G9 (validation P1, F9)

- **Contexte** : `les_replis_rust_suivent_le_catalogue` (G9, `crates/kesh-api/tests/textes_coherents.rs:680`) est une
  table fermée ; sans entrée, les replis Rust des six `t`/`t_args` pourraient dériver du catalogue `fr-CH`.
- **Retenu** : une quatrième entrée `crates/kesh-api/src/routes/audit_log.rs` avec les six clés (paramètres
  `{ $field }` → `{nom}`, `{ $max }` → `{maximum}`). La clé `invoice-qr-message-prefix` n'y entre pas : sans repli Rust.
- **Écartée** : laisser G9 telle quelle (facultatif, mais une dérive silencieuse pour quelques lignes).
- **Réversible** : oui.

### C-31-5-16 — Le budget d'issues n'est pas tenu par cette story : déficit déclaré, arbitrage au Project Lead (validation P1, R3/F1 ; rendu inconditionnel à la validation P2, F2 ; recompté à la validation P3, F4 = R8)

- **Contexte** : *(relevé de la validation P1, daté ; recompte vivant à l'Origine de la fiche : 143 ouvertes à la
  validation P3, #711 et #712 ouvertes, 138 attendues au tag)* recompte du 2026-10-24 (validation P1) : 141 ouvertes (#708, #709, #710 ouvertes ce jour) ; − #503 − 4 = 136, pas sous
  le plafond « moins de 136 » ; + l'issue annoncée des LOW résiduels de la 31-1c = 137 ; + l'issue des libellés
  d'écriture (C-31-5-17), non ouverte, si elle s'ouvre = 138.
- **Retenu** : écrire les deux comptes dans l'*Origine* — **137** attendues au tag si rien d'autre ne s'ouvre (deux
  retraits manquent), **138** si l'issue des libellés s'ouvre (trois manquent) — et les soumettre au Project Lead par
  l'orchestrateur : d'autres défauts corrigés ou publication au-dessus du plafond. Aucune fermeture pour le chiffre.
  *(La validation P1 n'écrivait que 137 et « deux retraits », sans l'issue que la fiche soumet elle-même : chiffre
  conditionnel sans le dire, F2 de la P2.)*
- **Recompte de la validation P3** (2026-10-24, `gh issue list --state open --limit 500 --json number --jq length`) :
  **143** ouvertes, #711 (LOW résiduels de la 31-1c) et #712 (libellés d'écriture, C-31-5-17) comprises ;
  143 − #503 − les quatre = **138** attendues au tag ; plafond < 136 : **trois** retraits manquent ; arbitrage au
  Project Lead. Plus aucune hypothèse : les deux issues annoncées sont ouvertes, et la règle de type des comptes de TVA
  n'en demande pas (C-31-5-19). Les comptes 137/138 ci-dessus sont des relevés datés de la P2.
- **Écartée** : ajouter d'office #687 ou #645 (écartées pour risque, C-31-5-2) ; taire le déficit.
- **Réversible** : sans objet (constat).

### C-31-5-17 — Libellés d'écriture en français en dur : hors périmètre, tracés par #712 (validation P1 ; issue ouverte à la validation P3)

- **Contexte** : `supplier_invoices.rs:512` et `:941`, `invoice_settlements_write.rs:277`, `credit_notes.rs:590`
  composent en dur des descriptions d'écritures en français ; ce ne sont pas des textes adressés au débiteur (hors
  classe de #504) ; aucune issue ouverte ne les couvre.
- **Retenu** : inventoriés au *Hors périmètre* ; tracés par **#712** (« Les libellés des écritures générées (facture
  fournisseur, règlement, avoir) sont écrits en dur en français »), ouverte le 2026-10-24 par l'orchestrateur en
  application de « on trace toujours » (F2 de la validation P3), comprise dans le recompte de 143 (C-31-5-16). La
  question de savoir s'ils doivent suivre la langue comptable de la société reste à trancher sur l'issue. Seul
  `supplier_invoices.rs:512` intéresse l'AC16 (exception nommée). *(Jusqu'à la validation P2 : « non ouverte, soumise au
  Project Lead ».)*
- **Écartée** : les traduire ici (ni mécanique ni dans le défaut).
- **Réversible** : oui.

### C-31-5-18 — #707 : trois lectures ponctuelles, l'ordre des verrous est celui du code (validation P2, R1 = F1)

- **Contexte** : la P1 prescrivait `… singleton_role IN (?, ?, ?) ORDER BY id FOR UPDATE` en annonçant un « ordre des
  verrous par clé primaire ». C'est faux : InnoDB pose ses verrous au fil de l'index parcouru —
  `uq_accounts_company_singleton_role (company_id, singleton_role)`, ordre alphabétique des rôles —, le tri vient après ;
  et l'ensemble même des verrous dépend du plan (accès par plages sur l'index unique, ou par le seul préfixe
  `company_id`, verrous *next-key* sur tous les comptes de la société). Le remède du dépôt, `FORCE INDEX (PRIMARY)`
  (`company_invoice_settings.rs:897-905`), suppose une liste d'`id`, que la requête n'a pas.
- **Retenu** : trois lectures `SELECT id, account_type, postable FROM accounts WHERE company_id = ? AND singleton_role = ?
  FOR UPDATE`, sur le patron d'`insert_with_defaults_in_tx` (`:1076-1105`), dans l'ordre écrit dans le code —
  `VatPayable`, `VatRecoverable`, `VatSettlement` (ordre permuté à la validation P3 en `VatRecoverable`, `VatPayable`,
  `VatSettlement`, C-31-5-21) ; chacune est un accès `const` à l'index unique, verrouille l'entrée de
  l'index puis l'enregistrement du compte, et ne laisse qu'un verrou d'intervalle pour un rôle absent. Le doc-comment, les
  AC1, Dev Notes et T2 disent cet ordre-là, sans promettre l'ordre des identifiants. L'ordre **entre tables**
  (`accounts → company_invoice_settings`, Pattern 5) reste tenu.
- **Écartées** : garder l'`IN` en corrigeant seulement le texte (plan dépendant de l'optimiseur, verrous possiblement
  étendus à toute la société) ; la forme en deux temps du précédent (lecture non verrouillante des `id`, puis
  `FORCE INDEX (PRIMARY) … id IN (…) ORDER BY id FOR UPDATE`) — rien, dans cette transaction, ne requiert l'ordre
  par `id`, et la lecture préalable non verrouillante ouvre une fenêtre entre les deux temps.
- **Réversible** : oui.

### C-31-5-19 — #707 : le filtre de type est celui du réglage, écrit une fois en `kesh-db` ; test et mutations des filtres (validation P2, R2, F10)

- **Contexte** : R2 — la branche « un rôle sans compte utilisable donne `None` » n'avait ni test ni mutation, alors
  qu'elle peut se produire (`VatPayable` admis sur un actif par la règle du rôle ; fenêtre de #538 entre la validation
  du plan et la dernière transaction). F10 — la fiche recopiait les types de la route, troisième copie d'une règle de
  type ; l'orchestrateur demandait de réutiliser `accepts_account_type` (`accounts.rs:89`). Établi au code :
  `accepts_account_type` (`kesh-core/src/chart_of_accounts/mod.rs:161`) est la règle du **rôle** — `Asset` ou
  `Liability` pour les trois rôles de TVA, appliquée à l'écriture de tout compte qui porte un rôle ; la réappliquer est
  une tautologie et laisse passer un `VatPayable` porté par un actif, que la route refuse (`Liability` exigé,
  `routes/company_invoice_settings.rs:337-364`). La règle du **réglage** n'existe qu'à la route, dans `kesh-api`, que
  `kesh-db` n'importe pas.
- **Retenu** : les trois types du réglage écrits **une fois** dans `kesh-db`, en une constante nommée
  (`VAT_SETTING_ACCOUNT_TYPES`, C-31-5-22), **employée par la route** depuis la validation P3 (F1 = R4) : la route
  (`kesh-api/src/routes/company_invoice_settings.rs`) importe déjà `kesh_db::entities::account::AccountType` (l. 25-26)
  et `kesh_db::repositories::company_invoice_settings` (l. 28), et ses trois appels de `validate_account` (l. 337-364)
  passent `expected: AccountType` par valeur (`Copy`) : trois arguments remplacés, comportement inchangé, une seule
  copie de la règle. Le sens de dépendance utile est `kesh-api → kesh-db`, que la P2 n'avait pas examiné. Précédent à
  ne pas reproduire : `designated_account_id` (`kesh-db/src/repositories/company_invoice_settings.rs:495-520`), deux
  copies (« Mêmes exigences que sa validation (`validate_account_of`, `kesh-api`) ») qu'aucun garde ne tient d'accord.
  *(Jusqu'à la validation P2 : « la route n'est pas touchée », constante « jumelle » de la route.)* Test `kesh-db` de la lecture à
  quatre cas (heureux ; `VatPayable` retypé en actif ; `VatRecoverable` non imputable ; `VatSettlement` archivé) ;
  mutations M7a (filtre de type retiré) et M7b (filtre `postable` retiré), chacune tuée par un cas nommé ; M8 pour le
  test de garde du chemin production (R3).
- **Écartées** : réutiliser `accepts_account_type` (filtre inopérant, cf. contexte — **écart assumé à la consigne de
  l'orchestrateur**, signalé dans le rapport) ; déplacer la règle du réglage en `kesh-core` et y brancher la route (un
  crate de plus pour un résultat qu'atteint la constante de `kesh-db`, déjà importable par la route) ; garder deux
  copies avec un test de garde `kesh-api` (deux copies et un test pour ce qu'un import supprime) ; garder deux copies
  et ouvrir une issue (dette évitable, F1 de la P3).
- **Réversible** : oui.

### C-31-5-20 — #469 : préfixe `audit-log-query-error-*` pour les six refus du serveur (validation P2, R9)

- **Contexte** : `audit-log-filter-*` (`fr-CH` l. 2719-2726) désigne les libellés du formulaire de filtre de l'écran, et
  `audit-log-error-date-range` (l. 2739) le contrôle de plage fait par l'écran ; `audit-log-api-key` (l. 2734) occupe
  déjà `audit-log-api-`. `grep -rn "audit-log-query" crates frontend/src` ne rend rien. Le précédent serveur du même
  fichier est `audit-log-export-error-too-large`.
- **Retenu** : `audit-log-query-error-*` (les refus portent sur les paramètres de la requête de consultation), dans
  l'espace `audit-log-` comme le précédent.
- **Écartées** : `audit-log-filter-error-*` (mêle refus du serveur et libellés de l'écran) ; `audit-log-api-error-*`
  (voisin de `audit-log-api-key`, sans rapport) ; `error-audit-log-*` (rompt avec le précédent du fichier).
- **Réversible** : oui (clés neuves, aucune lue par le frontend).

### C-31-5-21 — #707 : ordre des trois lectures permuté, `VatRecoverable` d'abord (validation P3, R3)

- **Contexte** : l'ordre retenu à la P2 (`VatPayable`, `VatRecoverable`, `VatSettlement`, soit 2200, 1171, 2206 au plan
  PME) est l'inverse, pour ses deux premiers comptes, du verrou partagé par identifiant croissant de
  `lock_designated_accounts_in_tx` (`company_invoice_settings.rs:865-916`, 1171 puis 2200), seul ordre opposé identifié *(prémisse de la P3, réfutée en P4 : voir « Rectifié » ci-dessous)* ;
  `bulk_create_from_chart_in_tx` crée les comptes par longueur puis numéro, d'où 1171 < 2200 < 2206 en identifiants.
- **Rectifié (validation P4, P4-1)** : l'« opposition » invoquée n'existe pas — les appelants de
  `lock_designated_accounts_in_tx` prennent `SALE` (créance, `VatPayable`) ou `PURCHASE` (dette, `VatRecoverable`),
  jamais 1171 et 2200 ensemble (`company_invoice_settings.rs:696-700`, vérifié par l'orchestrateur). La permutation est
  gardée pour la seule lisibilité (ordre des identifiants au plan standard), sans effet sur les interblocages.
- **Retenu** : `VatRecoverable`, `VatPayable`, `VatSettlement` — sans coût, il suit au plan standard l'ordre des
  identifiants ; le doc-comment dit qu'il coïncide avec l'ordre des identifiants au plan
  standard sans le garantir (un rôle réattribué peut le défaire), et le rejeu `is_seed_retryable` reste le filet.
- **Écartée** : garder l'ordre de la P2 (aussi sûr, l'opposition invoquée n'existant pas — P4-1 ; écartée pour la seule lisibilité).
- **Réversible** : oui.

### C-31-5-22 — #707 : définitions de `VatRoleAccounts` et de la constante des types (validation P3, F6)

- **Contexte** : l'AC1 prescrivait un type de retour et une constante sans nom, champs ni emplacement.
- **Retenu** : dans `crates/kesh-db/src/repositories/company_invoice_settings.rs`, `pub`, atteignables par
  `kesh_db::repositories::company_invoice_settings::…` : `#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)] pub
  struct VatRoleAccounts { pub payable, pub recoverable, pub settlement: Option<i64> }` (les trois réglages, dans
  l'ordre des colonnes) ; `pub struct VatSettingAccountTypes { pub payable, pub recoverable, pub settlement: AccountType }`
  et `pub const VAT_SETTING_ACCOUNT_TYPES` = `Liability` / `Asset` / `Liability`. Lecture typée
  `(i64, AccountType, bool)` (`AccountType: Decode<MySql>`, `entities/account.rs:43-66`).
- **Écartées** : un tableau `[AccountType; 3]` indexé (la position d'un réglage n'a pas de nom, une erreur d'indice ne
  rougit pas) ; trois constantes isolées (trois noms à importer pour une seule règle).
- **Réversible** : oui (noms internes, aucun n'est d'API).

### C-31-5-23 — #469 : la garde des comptes cités lit l'année d'une date ISO comme une date (développement, 2026-10-24)

- **Contexte** : le gate complet après T2 a rougi `textes_coherents::les_comptes_cites_en_exemple_existent_dans_les_plans_livres`
  sur la clé neuve `audit-log-query-error-date-out-of-range` (« … entre 1000-01-01 et 9999-12-31 ») : `nombres_isoles`
  lisait `9999` comme un numéro de compte, absent des plans livrés (`1000` passait, Caisse). Les deux bornes doivent
  figurer dans le message : `une_date_hors_plage_est_refusee_pour_la_plage_et_non_le_format` l'exige.
- **Retenu** : correctif mécanique de la garde (test seulement) — un nombre de quatre chiffres sans décimales suivi
  exactement de `-MM-JJ` (`^-\d{2}-\d{2}(\D|$)`) est l'année d'une date ISO, non un compte ; écrit dans la liste des
  exceptions du doc-comment de la garde.
- **Écartées** : reformuler le message sans les bornes (contredit le test existant et l'AC11) ; écrire les bornes
  autrement (`9 999`, en lettres : texte moins exact pour un nom d'API) ; une exemption par clé (une liste à tenir).
- **Réversible** : oui. **Angle mort créé** : un compte faux écrit `NNNN-MM-JJ` n'est plus lu — forme sans usage.

### C-31-5-24 — #469 : formulation du refus de date mal formée, et `max` passé en texte (développement, 2026-10-24)

- **Contexte** : l'AC11 retire le détail `chrono` (`({e})`, anglais) sans fixer la nouvelle formule ; `{ $max }` peut
  être passé à Fluent en nombre ou en texte.
- **Retenu** : « `{ $field }` invalide : date attendue au format AAAA-MM-JJ » (et ses trois traductions, chaque
  locale avec son gabarit : JJJJ-MM-TT, AAAA-MM-GG, YYYY-MM-DD) ; `field` et `max` passés en **texte**
  (`to_string()`), dans le code comme dans le test de l'AC12, pour qu'aucun formatage numérique de locale n'entre
  dans la comparaison.
- **Écartée** : `max` en nombre Fluent (sans effet pour 32 et 64, mais un formatage de locale possible au-delà de
  999, sans gain).
- **Réversible** : oui.

### C-31-5-25 — #707 : la route lit la constante par une variable locale (développement, 2026-10-24)

- **Contexte** : l'AC1 prescrit que les trois arguments de type deviennent `VAT_SETTING_ACCOUNT_TYPES.payable`, etc.,
  « aucune autre ligne de la route ne change ».
- **Retenu** : une ligne `let vat_types = company_invoice_settings::VAT_SETTING_ACCOUNT_TYPES;` et deux lignes de
  commentaire avant les trois appels, puis `vat_types.payable` / `.recoverable` / `.settlement` — trois chemins
  complets répétés étaient plus longs que les arguments qu'ils remplacent. Comportement identique (`AccountType` `Copy`).
- **Écartée** : le chemin complet à chaque argument (conforme à la lettre, plus verbeux).
- **Réversible** : oui (écart à la lettre de l'AC1, déclaré au Dev Agent Record).

### C-31-5-26 — recette E2E d'un worktree : le défaut voisin est intégré, aucune issue (revue de code P1, 2026-10-24)

- **Contexte** : le premier run E2E du développement (298 / 11 / 17 / 10 non exécutés) a rougi quatre specs qui
  déposent ou lisent un fichier : le backend tournait sur des dossiers propres au worktree, le runner sur les défauts
  `/tmp/kesh-e2e/{inbox,documents}` de `frontend/tests/e2e/helpers/inbox.ts`. Le Dev Agent Record en accusait
  `docs/testing.md` et demandait une issue ; la revue de code P1 (A1, MEDIUM) a établi que `docs/testing.md`
  l. 244-250 le disait déjà, et que seul `docs/agents-orchestration.md` § 4, point 2, le disait à moitié.
- **Retenu** : le défaut voisin est intégré à la story — précision portée par l'orchestrateur au commit `adff757c`
  (« des dossiers propres au worktree se passent au backend ET au runner ») ; le Dev Agent Record est annoté, non
  réécrit ; `docs/agents-orchestration.md` entre à la File List ; **aucune issue n'est ouverte**, la documentation
  n'ayant plus rien de faux à suivre.
- **Écartée** : ouvrir l'issue demandée par le Record (elle aurait décrit un défaut absent de `docs/testing.md`, et
  fait remonter le compte d'issues sans rien tracer de réel).
- **Réversible** : oui (documentation seule).
