# Story 31.1 : Une sauvegarde qui restaure tout, faite par Kesh

## Status

**ready-for-dev** (trois clés) — fiche créée le 2026-10-23 en autonomie (`bmad-create-story`, Opus 5.5), sur
`1848012d` (kickoff de l'Epic 31), worktree `/home/gcorbaz/devel/kesh-31-1`, branche `story/31-1`. Validation P1
faite (Sonnet, R 0/0/10/8, F 0/2/9/8) et remédiée le 2026-10-23 ; validation P2 faite (Opus, R 0/0/8/8, F 0/1/7/9)
et remédiée le 2026-10-23 ; validation P3 faite (Sonnet, R 0/0/5/7, F 0/0/4/11) et remédiée le 2026-10-23 ;
validation **P4** à lancer (`311-validate-prompt-p4.md`, **Opus**, lentilles R et F).

⛔ **Trois stories, une fiche, trois PR successives** (C-31-1-1) :

| Clé au registre de sprint | Contenu | AC | PR |
|---|---|---|---|
| **`31-1a-fichiers-dans-la-sauvegarde`** | format `.keshbackup` 2 : les fichiers référencés par la base dans `files/`, nommés par leur empreinte, inventoriés au manifeste, lus dans l'instantané ; import qui vérifie (lectures bornées), écrit les fichiers sous `KESH_DOCUMENTS_DIR` puis restaure la base, sans dépendre de la connexion du client ; ancien format accepté ; export assemblé hors mémoire, import reçu sur disque ; manuel d'administration (format, volume, proxy inverse, ce qui reste hors du fichier) | AC1 à AC14, AC6b, AC8b | **`refs #503`** — ⛔ jamais `closes` |
| **`31-1b-restauration-installation-neuve`** | le parcours *installation neuve → `/setup` → restaurer une sauvegarde* sans passer par l'assistant ; l'écran dit ce que le fichier ne contient pas (`.env`) et ce que la restauration a rendu (pièces restaurées, pièces manquantes) ; l'export et l'import échappent au délai de 30 s du client HTTP ; E2E de bout en bout ; manuel pas à pas | AC15 à AC22, AC18b | **`refs #503`** |
| **`31-1c-sauvegarde-automatique`** | Kesh écrit seul, à intervalle régulier, une sauvegarde complète dans `KESH_ADMIN_BACKUP_DIR/auto/`, avec rétention ; réglage et état dans *Administration → Sauvegarde complète* ; « Sauvegarder maintenant » ; liste et téléchargement des sauvegardes du serveur ; échec signalé (écran et bandeau) ; disque plein ; concurrence ; migration (table système) | AC23 à AC40 | **`closes #503`** (sur la PR seulement) |

**Ordre** : la 31-1a part de `main` ; la 31-1b après le merge de la 31-1a ; la 31-1c après celui de la 31-1b (elle
réutilise l'écriture sur disque de la 31-1a et l'écran de la 31-1b). Aucune release entre elles : la v0.15.0 attend la
fin de l'epic (C-31-0-1). La clé mère `31-1-sauvegarde-complete` passe au statut `split`.

## Story

**As a** administrateur d'une installation Kesh qui tient désormais des livres réels (premier cas : Guy, depuis le
2026-10-23),
**I want** que la sauvegarde `.keshbackup` contienne tout ce qu'il faut pour remettre l'installation debout sur une
machine neuve — la base **et** les pièces (documents fournisseur, PDF figés des factures émises) —, et que Kesh la
fasse lui-même, régulièrement, en me disant quand elle échoue,
**so that** une panne, un disque perdu ou un changement de serveur ne me coûte ni une écriture ni une pièce
justificative, sans dépendre d'une procédure SSH ni de ma mémoire.

Issue : [#503](https://github.com/guycorbaz/kesh/issues/503) (`enhancement`, **P1**) et ses deux commentaires de Guy
(2026-10-10 : restauration sur un système neuf, `.env`, volume ; 2026-10-23 : sauvegarde locale automatique et complète
faite par Kesh, `.env` hors sauvegarde pour l'instant). Epic : `_bmad-output/planning-artifacts/epic-31-tenir-les-livres.md`
§ 31-1. Cible : **v0.15.0**. Choix : `epic-31-choix-autonomes-311.md` (C-31-1-1 à C-31-1-51). Réglages nouveaux dans
l'interface, pas en variable d'environnement (C-31-0-4).

## Prémisses de #503, établies au code (`1848012d`)

| # | Prémisse | Verdict | Preuve |
|---|---|---|---|
| P-1 | L'export crée un `files/` vide | **vraie** | `crates/kesh-api/src/admin_backup/export.rs:95` (`zip.add_directory("files/", …)`, commentaire « aucun binaire stocké en v0.2 ») ; l'issue citait l. 76-79, le fichier a bougé depuis (Story 28-3) |
| P-2 | L'import refuse un `files/` non vide | **vraie** | `admin_backup/import.rs:79-96` (`files_non_empty`, l. 83 et 91) ; et exige sa présence (l. 98-102) |
| P-3 | Après restauration, un justificatif et un PDF figé répondent **410** | **vraie sur une autre machine, à nuancer sur la même** | `routes/imported_supplier_invoices.rs:725` (`SourceDocumentGone`) ; `routes/issued_invoice_pdf.rs:365` (`InvoicePdfGone`). ⚠️ Restaurée **sur la même installation**, une sauvegarde retrouve ses fichiers : Kesh ne supprime jamais un document (`grep -rn remove_file crates/kesh-api/src` : seulement le fichier source de l'inbox, `inbox_import.rs:839`, et les temporaires). Le 410 frappe la restauration sur un **autre** dossier de documents — le cas de #503 |
| P-4 | Les fichiers référencés par la base sont « les justificatifs et les PDF figés » | **vraie, et l'inventaire est clos** | deux appelants de production de `document_storage::store_document` : `inbox_import.rs:580` (documents fournisseur → `imported_supplier_invoices.storage_path`, `file_hash`) et `routes/issued_invoice_pdf.rs:432` (PDF figé → `invoices.pdf_storage_path`, `pdf_sha256`, migration `20261003000001`). Les documents joints après coup à une facture fournisseur (30-2c) sont des lignes de `imported_supplier_invoices`. Aucune autre colonne `*storage_path*` (`grep -rn storage_path crates/kesh-db/migrations`). Avoirs, rappels, rapports : rendus à la demande, jamais stockés |
| P-5 | Nommage par empreinte SHA-256 | **déjà vrai sur disque** | `document_storage.rs:111` : nom `{sha256hex}.{ext}` (extension en minuscules, contrôlée par `is_safe_ext`, l. 60-61 : alphanumérique, 8 caractères au plus — pas une liste blanche ; numéros corrigés en P2) ; le contenu d'un nom donné est donc fixé par le nom |
| P-6 | « Passage en streaming à l'export » | **faux aujourd'hui** | l'archive est **entièrement assemblée en mémoire** (`export.rs:66` `Cursor<Vec<u8>>` — « l. 72 » écrit à la création, corrigé en P1) ; seule la *livraison* passe par un fichier temporaire au-delà de `KESH_ADMIN_EXPORT_INMEM_MB` (`routes/admin.rs:53-58`, `stream_via_tempfile`). Avec les fichiers, l'archive entière tiendrait en mémoire |
| P-7 | Volume à l'import (`KESH_ADMIN_IMPORT_MAX_MB`, 512 Mo) | **vraie, et l'import lit tout en mémoire** | `routes/admin.rs:214` (`field.bytes()`), plafond `lib.rs:259` ; défaut 512, bornes [1, 10240] (`config.rs:927-946`) |
| P-8 | Instantané cohérent (Story 28-3) | **vraie** | `kesh_db::backup::export_installation` (`backup.rs:380`) lit toutes les tables dans un seul `ReadSnapshot`, clos avant l'assemblage ; garde lexicale `export.rs` `build_keshbackup_ne_lit_la_base_qu_une_fois` |
| P-9 | La sauvegarde de sécurité pré-import | **vraie, et elle suivra** | `routes/admin.rs:264` : même cœur `build_keshbackup`, sous le verrou `_kesh_version FOR UPDATE` (l. 239) ; elle emportera donc les fichiers elle aussi |
| P-10 | Planificateur de tâches dans le backend | **absent** | `main.rs` : aucune tâche périodique (un nettoyage **au démarrage**, l. 206-217) ; un seul `tokio::spawn`, détaché, pour un e-mail (`routes/auth.rs:687`) ; pas d'arrêt gracieux (`axum::serve` nu, l. 389). La 31-1c pose la **première** tâche de fond |
| P-11 | Dossier de sauvegarde local | **vraie** | `KESH_ADMIN_BACKUP_DIR`, défaut `/data/backup` (`config.rs` `DEFAULT_ADMIN_BACKUP_DIR`), monté sur `./backup` par les deux compose (`docker-compose.yml:159`, `docker-compose.prod.yml:177`) |
| P-12 | « Nouvelle installation (base vierge, `/setup`), import du `.keshbackup` » | **impraticable aujourd'hui — à corriger** | après `/setup`, l'utilisateur atterrit sur `/onboarding` (`setup.spec.ts`), et la garde de `routes/(app)/+layout.ts:33-44` renvoie à l'assistant tant que `step < 6` (production) ou `< 3` (démo) : `/admin/restore` est **injoignable** sans configurer d'abord une société ou charger la démo. Le serveur, lui, n'impose rien (route d'administration sans garde d'onboarding, `lib.rs:251-259`) et force l'onboarding « terminé » après l'import (`backup.rs` `force_onboarding_done_if_eligible`). D'où la 31-1b |
| P-13 | Une table de réglages neuve « comme les autres » | **interdite** | toute table de `TABLES_TO_TRUNCATE` doit figurer au manifeste (`import.rs` couverture, l. ~114-121) : une table applicative neuve rendrait **toutes les sauvegardes existantes non importables** (motif écrit dans `20261003000001_invoices_frozen_pdf.sql:8-11`). Guy a désormais des sauvegardes 0.14.0 de livres réels. D'où une table **système**, hors inventaire (C-31-1-15) |
| P-14 | `.env` hors sauvegarde | **vraie** | la configuration n'est lue que de l'environnement (`config.rs`) ; aucune table ne la porte |

### Ce qui existe et se réutilise (DRY)

- **Écriture atomique d'un document** : `document_storage::store_document` (temporaire, `fsync`, `rename`). La
  restauration d'un fichier en factorise le cœur (C-31-1-6), elle ne le recopie pas.
- **Écriture sûre d'une sauvegarde** : `routes/admin.rs` `write_backup_file` (`.partial` `0600`, `create_new`, refus
  d'écraser, renommage) et `write_pre_import_backup` (dossier `0700` récursif). La sauvegarde automatique les réutilise.
- **Instantané** : `export_installation_in(snap, obs)` et son observateur de test (Story 28-3) ; la liste des fichiers
  référencés s'y lit, dans le même instantané.
- **Garde de l'import** : `parse_and_verify` (structure, empreintes, couverture, colonnes) ; elle s'étend au format 2.
- **E2E** : `frontend/tests/e2e/helpers/inbox.ts` (`DOCUMENTS_DIR`, garde bruyante si le runner et le backend ne voient
  pas le même dossier, l. 82-83 et 160) ; `seedTestState('setup-required')` (`setup.spec.ts`).
- **Écran** : `features/admin-backup/AdminBackupPanel.svelte`, `features/admin-restore/AdminRestorePanel.svelte`.

## Périmètre minimal

**Dedans** : les trois critères de #503, les deux commentaires de Guy, et ce qui les rend vrais au code.

**Dehors** :

| Hors périmètre | Raison | Suite |
|---|---|---|
| Configuration `.env` dans la sauvegarde | décision de Guy (2026-10-23) | dite à l'écran et au manuel (AC17, AC21) |
| Restaurer **depuis** une sauvegarde du serveur sans la télécharger | non demandé ; le téléchargement (AC34) suffit à l'import | proposé à l'orchestrateur comme `enhancement` éventuel (rapport) |
| Copie hors site (cloud, NAS distant) | « sauvegarde locale » ; Hyper Backup copie déjà `./backup` | — |
| Chiffrement du `.keshbackup` | inchangé : un secret, à chiffrer pour le transit (manuel) | — |
| Fichiers non référencés du dossier des documents (rendus orphelins) | « tous les fichiers référencés par la base » | écrit au manuel |
| Plusieurs processus Kesh sur une même base | une instance par base (déploiement documenté) | la réservation conditionnelle (AC27) protège tout de même la sauvegarde automatique ; écrit en angle mort |
| Archives de plus de 4 Gio | non mesuré | angle mort écrit (C-31-1-9) |

### Décompte des modules (règle de découpage)

| Sous-story | Modules | Nombre |
|---|---|---|
| 31-1a | `kesh-db` (`backup.rs` : fichiers référencés dans l'instantané), `kesh-api` (`admin_backup/*`, `routes/admin.rs`, `document_storage.rs`, `config.rs`, fixture 0.14.0 sous `tests/fixtures/`), docs (manuel d'administration, `docs/api-external.md`, CHANGELOG) | 3 |
| 31-1b | frontend (onboarding, `admin-restore`, `admin-backup`, `shared/utils/api-client.ts`), `kesh-i18n`, docs (manuels FR), E2E | 4 |
| 31-1c | `kesh-db` (migration, squash **et son script `scripts/regen-test-schema.sh`**, `tests/test_schema_guard.rs`, dépôt, `post_restore`, inventaire), `kesh-api` (tâche de fond, routes, `AppState`, `main.rs`, `errors.rs` — variante `BackupInProgress` —, `tests/admin_pat_denied_e2e.rs`), frontend (panneau, bandeau), `kesh-i18n`, docs | 5 |

Fusionnées, la 31-1a et la 31-1b dépasseraient cinq modules ; la 31-1c seule en atteint cinq (C-31-1-1). La
remédiation P1 n'ajoute de module à aucune sous-story : le correctif du client HTTP (AC18b) va dans la 31-1b, qui
porte déjà le frontend ; `scripts/regen-test-schema.sh` est compté avec `kesh-db`, dont il produit le squash
(`crates/kesh-db/test-schema/`) et dont la garde le contrôle (C-31-1-36). La remédiation P2 n'en ajoute pas
davantage : la garde de rôle de `/onboarding/restore` est du frontend (31-1b) ; les gardes possédés, les noms à la
milliseconde et le seuil `documents_bytes` restent dans `kesh-api` (31-1c) ; recompté : 3, 4, 5. La remédiation P3
non plus : l'assemblage en un seul `spawn_blocking`, la raison `unreadable` et l'inventaire étendu (`.env.example`,
`README.md`, doc-comments) restent dans `kesh-api` et docs (31-1a) ; le verrou pris par `run_once`, la garde du
lancement par `main.rs` et la variante `BackupInProgress` d'`errors.rs` dans `kesh-api` (31-1c) ; recompté : 3, 4, 5.

## Critères d'acceptation

### Partie a — `31-1a-fichiers-dans-la-sauvegarde`

**AC1 — Inventaire des fichiers référencés, lu dans l'instantané.** `kesh_db::backup` expose
`ReferencedFile { storage_path: String, sha256: String }` et l'`InstallationExport` porte `referenced_files:
Vec<ReferencedFile>` (dédoublonnés par `storage_path`, triés), lus par `export_installation_in` **dans le même
`ReadSnapshot`** que les tables, après elles :
`SELECT storage_path, file_hash FROM imported_supplier_invoices` ∪ `SELECT pdf_storage_path, pdf_sha256 FROM invoices
WHERE pdf_storage_path IS NOT NULL`. Toutes les lignes de `imported_supplier_invoices` comptent, quel que soit leur
statut (`discarded` compris : la ligne reste restaurée et son document téléchargeable). La clé est le
`storage_path` : **deux chemins de même empreinte sont deux fichiers** (même contenu importé par deux sociétés sous
deux extensions, `x.jpg` et `x.jpeg` — dans une même société, `UNIQUE (company_id, file_hash)` l'interdit, si bien que
le test de T-a4b seede deux sociétés), tous deux inclus. Sont des
**anomalies**, classées `mismatch` (AC3) sans lecture du fichier : (i) un `storage_path` dont le préfixe hexadécimal
n'est pas l'empreinte déclarée **par la même ligne** ; (ii) un même `storage_path` déclaré avec deux empreintes
différentes par deux lignes (aucune n'est choisie) ; (iii) un `storage_path` qui ne satisfait pas l'**une** des deux
règles, `is_safe_storage_path` ou `^[0-9a-f]{64}\.[a-z0-9]{1,8}$` (C-31-1-28 ; formulation corrigée en P3 : « et » se
lisait « aucune des deux »). Chaque chemin en anomalie est écrit **une
seule fois** dans `filesNotIncluded`, **tel quel** (même hors forme : l'import l'accepte, AC6, C-31-1-37).

**AC2 — Garde de l'inventaire (classe close).** Un test lit `information_schema.COLUMNS` du schéma et exige que
l'ensemble des colonnes dont le nom contient `storage_path` soit exactement {`imported_supplier_invoices.storage_path`,
`invoices.pdf_storage_path`} ; un second test (lexical, sur le code de production de `crates/`) exige que les appelants
de `store_document(` soient exactement `inbox_import.rs` et `routes/issued_invoice_pdf.rs`. Une colonne ou un appelant
neuf fait rougir l'un des deux, message qui nomme l'AC1 à mettre à jour.

**AC3 — Format 2 : les fichiers dans `files/`.** `BACKUP_FORMAT_VERSION` passe à **2**. L'export écrit, pour chaque
fichier référencé **présent et intact** (empreinte du contenu lu = `sha256` déclaré = préfixe du nom), une entrée
`files/<storage_path>` en méthode **Stored** (PDF et images sont déjà compressés), mode `0644`. Le manifeste gagne :

```json
"files": [ { "path": "<sha>.<ext>", "sha256": "<sha>", "byteSize": 12345 } ],
"filesNotIncluded": [ { "path": "<sha>.<ext>", "reason": "missing" | "altered" | "mismatch" | "unreadable" } ]
```

(champs `#[serde(default)]` : un manifeste 1 se relit avec deux listes vides). Un fichier **absent** du dossier des
documents (`NotFound`), **altéré** (empreinte différente), **illisible** (`unreadable` : toute autre erreur d'I/O à
l'ouverture ou pendant la passe de hachage — droits, `EIO`, dossier à la place du fichier ; journalisée `warn!` avec
`?kind`) ou en **anomalie** (AC1) n'empêche pas la sauvegarde : il est listé dans `filesNotIncluded` et **n'est pas**
écrit dans l'archive (C-31-1-4, C-31-1-47 — un seul fichier illisible ne doit pas faire échouer chaque sauvegarde).
**Deux passes par fichier** : le crate `zip` ne sait pas retirer une entrée commencée, donc l'empreinte se calcule
**avant** d'ouvrir l'entrée (passe 1 : classement) ; la copie dans l'entrée (passe 2) **re-hache** ce qu'elle copie, à
travers `take(byteSize + 1)`. Une divergence de la passe 2 (empreinte, taille, ou erreur d'I/O) **abandonne toute la
sauvegarde** (`BackupWriteError`, aucune archive rendue ni renommée : export 500, sauvegarde pré-import en échec donc
import refusé, sauvegarde automatique `failed`) — une archive que l'import refuserait (AC7) ne sort jamais. La fenêtre
est celle d'une lecture sur un fichier nommé par son contenu et jamais réécrit (P-3) : l'abandon est le cas d'école, pas
l'usage (C-31-1-47). Les fichiers du dossier que la base ne référence pas
ne sont pas emportés. **Aucun chemin n'est joint au dossier des documents avant validation** : la lecture passe par une
fonction de `document_storage` (`open_document(dir, storage_path) -> Result<File, ReadDocumentError>`, qui applique
`is_safe_storage_path`, rendue `pub(crate)`, comme `read_document` le fait déjà) — jamais `documents_dir.join(…)` dans
`export.rs`. Comme l'AC1 classe déjà `mismatch` tout chemin hors forme, aucun test d'export n'atteint ce contrôle : il
se prouve par un test **unitaire direct** d'`open_document` (`"../x"`, `"/etc/hostname"`, `"a/b"`, `""` →
`InvalidPath`, sans toucher au disque), qui tue M-a12 (R3, P3). L'entrée dossier `files/` est écrite **même sans aucun fichier** (le contrôle de présence de l'import,
`import.rs:98-102`, l'exige pour le format 1 et le 2).

**AC4 — L'export n'assemble plus l'archive en mémoire.** Deux temps, et deux seulement (C-31-1-46, P3 : la forme de la
P2 — un écrivain empruntant le `File`, des copies en `spawn_blocking` séparés, un `write_backup_file` sur `tokio::fs` —
ne compilait pas ou copiait des Gio sur un fil de l'exécuteur) :

1. **lecture de la base**, asynchrone : `export_installation(pool)` (instantané, tables et `referenced_files` en
   mémoire, comme aujourd'hui — quelques dizaines de Mio) ;
2. **tout le reste dans un seul `tokio::task::spawn_blocking`**, par une fonction **synchrone**
   `write_keshbackup<W: Write + Seek>(export: &InstallationExport, documents_dir: &Path, w: W) -> Result<(W,
   KeshBackupMeta), BackupWriteError>` : écriture des tables, lecture des fichiers **un par un** (`std::fs`, deux passes,
   AC3 ; jamais tous en mémoire), `finish`. La fermeture **possède** tout ce qu'elle touche (l'`InstallationExport`, les
   chemins, l'écrivain ; à la 31-1c un `Arc<dyn BackupEnv>`) : aucun emprunt, aucun `tokio::fs` mêlé au `ZipWriter`
   synchrone, **aucun `block_in_place`** (il panique sous le runtime mono-fil des `#[tokio::test]`).

`GET /admin/full-export` passe à `write_keshbackup` un `tempfile::SpooledTempFile` de seuil
`KESH_ADMIN_EXPORT_INMEM_MB` (sémantique de la variable inchangée : en mémoire jusqu'au seuil, sur disque au-delà),
débordant **dans `KESH_ADMIN_BACKUP_DIR`** (`spooled_tempfile_in`, `tempfile` 3.27 — monté, contrairement au `/tmp` du
conteneur), le reprend au retour du `spawn_blocking`, puis le diffuse. **Le dossier est créé s'il n'existe pas**
(`0700`, récursif) par le code que `write_pre_import_backup` porte déjà (`routes/admin.rs:523-530`, factorisé en
`ensure_backup_dir`), avant d'ouvrir le temporaire — sur une machine neuve hors compose, il peut manquer. La sauvegarde
pré-import (et, à la 31-1c, l'automatique) passe par `write_backup_file`, qui devient **synchrone** (`std::fs`,
appelée dans ce même `spawn_blocking`) et prend une fermeture de remplissage (C-31-1-7) : elle ouvre le `.partial`
(`0600`, `create_new`) en `std::fs::File`, passe à la fermeture un **clone** (`File::try_clone`, même descripteur), puis
appelle `sync_all()` sur l'original, refuse d'écraser et renomme. `tempfile` passe des `[dev-dependencies]` aux
`[dependencies]` de `kesh-api` (déjà au `Cargo.lock`). La primitive rend une erreur **typée** qui conserve
l'`io::Error` d'origine (`BackupWriteError { stage, source: io::Error }` ; une `ZipError` y entre par
`io::Error::from(ZipError)`, jamais par `format!`, F10) ; l'adaptation vers `AppError::AdminPreImportBackupFailed` se
fait chez l'appelant pré-import, pas dans la primitive (la 31-1c en a besoin pour distinguer `disk_full`, AC30). La
garde lexicale `build_keshbackup_ne_lit_la_base_qu_une_fois` s'adapte au nouveau corps et reste exacte (le pool n'est
lu que par `export_installation(pool)` ; `write_keshbackup` ne reçoit aucun pool).

**AC5 — Ce que l'export rend, et ce qu'il dit.** La réponse de `GET /admin/full-export` porte deux en-têtes,
`X-Kesh-Backup-Files: <n inclus>` et `X-Kesh-Backup-Files-Not-Included: <n>` (exposés à l'écran par la 31-1b). L'audit
`admin.full_export` ajoute `file_count`, `files_bytes`, `files_not_included` au détail. Un fichier absent ou altéré est
journalisé (`warn!`, chemin et raison, jamais le contenu) ; un fichier `unreadable` aussi, avec `?kind`.

**AC6 — Import : formats acceptés.** `parse_and_verify` accepte `formatVersion` 1 **et** 2, refuse au-delà
(`InvalidBackupStructure`, message « formatVersion N non supporté (maximum 2) »). Format 1 : règle actuelle inchangée
(`files/` présent et vide). Format 2 : `files/` présent ; **toute ligne de `manifest.files` a son entrée**
`files/<path>` (une ligne sans entrée → 400) — l'autre moitié de la bijection, une entrée `files/*` sans ligne, est
déjà refusée par le contrôle d'ensemble de l'AC6b (2), qui seul la porte (R2, P3 : la porter deux fois rendait M-a4
équivalente) ; chaque nom satisfait
`^[0-9a-f]{64}\.[a-z0-9]{1,8}$` **et** `is_safe_storage_path`, et son préfixe égale le `sha256` déclaré ; aucun
sous-dossier sous `files/`. Les chemins de `filesNotIncluded` ne sont soumis à **aucune** expression : ils ne servent
jamais de chemin (ni lecture, ni écriture, ni jointure au dossier des documents), et l'export y écrit tel quel le
`storage_path` hors forme d'une ligne en anomalie (AC1 (iii)) — les y refuser rendrait inimportable, par la même
version, une sauvegarde authentique (C-31-1-37, retrait de la règle posée en P1). Ils ne sont journalisés qu'en
`Debug` (`?path`), jamais interpolés. Un chemin présent **à la fois** dans `files` et dans `filesNotIncluded`, ou deux
fois dans l'une des listes → 400.

**AC6b — Archive piégée : lectures bornées (les deux formats).** `parse_and_verify` ne lit plus aucune entrée sans
borne, et dans cet ordre : (1) `manifest.json` d'abord, par `by_name`, à travers `take(MANIFEST_MAX_BYTES + 1)`
(64 Mio), puis désérialisé (le format est alors connu) ; (2) pour le format 2, chaque nom de `file_names()` est
comparé à l'ensemble attendu (`manifest.json`, `data/<table>.ndjson` des tables du manifeste, `files/`, `files/<nom>`
de `manifest.files`) **avant toute autre lecture** — un nom hors de l'ensemble → 400 (le format 1 garde la règle
actuelle, `import.rs:85-88` : entrée hors de `manifest.json`, `data/`, `files/` → 400) ; (3) les
`data/<table>.ndjson` à travers une borne **cumulée** `TABLES_MAX_BYTES` (1 Gio pour **l'ensemble** des tables, non par
table — F5, P3 : une borne par table laissait 39 × 1 Gio en mémoire ; le reste de chaque entrée se lit à travers
`take(reste + 1)`), indépendante de la taille déclarée par l'en-tête ZIP, qui est déclarative ; (4) les entrées de
`files/` à travers `take(byteSize.saturating_add(1))` (AC7) — un `byteSize` supérieur au plafond d'import
(`KESH_ADMIN_IMPORT_MAX_MB`) est refusé en 400 dès la lecture du manifeste, avant toute entrée (F6, P3 : `u64::MAX + 1`
paniquait en débogage). Dépassement → 400 `InvalidBackupStructure` (« entrée … trop grande »). Les bornes
(`MANIFEST_MAX_BYTES`, `TABLES_MAX_BYTES`) sont des constantes en production, passées à `parse_and_verify` par une
structure `ReadLimits` que les tests réduisent (« table trop grande » se teste avec quelques Kio, pas avec 1 Gio sous
`mem-guard`). **Retirés en P2** (C-31-1-38) : le refus des noms
en double — `zip` 2.4.2 les fusionne à l'ouverture (`read.rs:72-75`, `IndexMap`), si bien qu'aucune API publique ne les
voit et qu'aucun test ne les construit (l'écrivain les refuse, `write.rs:1050`) ; le doublon est inoffensif, seule
l'entrée retenue est lue et elle est vérifiée par son empreinte — et la borne `MAX_ENTRIES`, qui arrivait **après**
`ZipArchive::new`, lequel a déjà décodé tout le répertoire central (`read.rs:706-710`) : ce coût n'est borné que par
la taille reçue (plafond d'import, AC12) ; angle mort écrit. C-31-1-29, C-31-1-38.

**AC7 — Import : empreintes vérifiées avant tout effet.** Chaque fichier de `files/` est lu en flux à travers un
`Sha256` avec une borne `byteSize.saturating_add(1)` (une entrée qui dépasse sa taille déclarée → 400, message « trop
grande », défense contre l'archive piégée) ; une empreinte différente → 400 `InvalidBackupStructure` (« intégrité SHA-256 invalide pour le fichier … »).
Ces contrôles ont lieu **avant** le verrou, la sauvegarde de sécurité et tout `DELETE`, dans un `spawn_blocking`
(lecture disque et SHA-256 de plusieurs Gio : jamais sur un fil de l'exécuteur asynchrone). **Cohérence avec la base
restaurée — format 2 seulement** (un format 1 n'a ni `files` ni `filesNotIncluded` : la clause ne s'y applique pas,
AC6, AC10) : tout fichier de `manifest.files` doit être référencé par une ligne restaurée (`imported_supplier_invoices`
ou `invoices` du backup), avec la même empreinte (`file_hash` / `pdf_sha256`) — sinon 400 ; un fichier référencé par
le backup qui n'est ni dans `files` ni dans `filesNotIncluded` → 400 (sauvegarde incomplète ou falsifiée) — la
comparaison est une égalité de chaînes, si bien qu'un chemin hors forme que l'export a listé (AC1 (iii)) satisfait la
règle ; il compte ensuite dans `filesMissing` (AC9 : `read_document` le refuse). Une colonne
de référence **absente** des `columnNames` d'une table du backup (`invoices.pdf_storage_path` n'existe pas avant la
migration `20261003000001`) vaut « aucune référence » pour cette table — ni `panic`, ni 400 ; idem pour le calcul de
`filesMissing` (AC9).

**AC8 — Import : les fichiers sont écrits avant de rendre la main.** Dans `run_backup_and_restore`, après la
sauvegarde de sécurité et **avant** le `DELETE` des tables : chaque fichier de l'archive est écrit sous
`KESH_DOCUMENTS_DIR` par la primitive factorisée de `document_storage` (temporaire, `fsync`, `rename`), **en flux** :
l'entrée ZIP est copiée dans le temporaire à travers `take(byteSize.saturating_add(1))` et un `Sha256`, l'empreinte comparée avant
le `rename` — jamais le fichier entier en mémoire (une entrée qui déclare plusieurs Gio ne fait donc pas tomber le
processus ; elle remplit au pire le disque, ce qui est un refus d'écriture ci-dessous). Un fichier de
même nom déjà présent **et intact** est laissé (compté restauré) ; présent mais **altéré**, il est remplacé
atomiquement (journalisé `warn!`). Un échec d'écriture (dossier non inscriptible, disque plein) → la restauration est
**refusée**, la base n'est pas touchée (`AdminFullImportFailed`, 500, message « aucune donnée n'a été remplacée ») ;
les fichiers déjà écrits restent (ils sont nommés par leur contenu : inoffensifs, C-31-1-6). Aucun fichier du dossier
des documents n'est jamais supprimé par l'import. L'écriture des fichiers (`std::fs`, `fsync`) s'exécute en
`spawn_blocking` ; à la fin de la boucle, le dossier des documents lui-même est synchronisé
(`File::open(documents_dir)?.sync_all()`, une ligne ; F15, P3) pour que les entrées de dossier, et non seulement les
contenus, soient sur disque avant le `COMMIT` — non testable (durabilité après coupure), dit au doc-comment. L'archive
parvient à cette écriture par `ParsedBackup`, qui **porte** la `ZipArchive<R>` ouverte par `parse_and_verify` (le
temporaire d'import est anonyme et ne se rouvre pas par chemin, AC11 ; F9, P3). Ce que l'ordre « fichiers avant la base » achète se teste par sa négation : écrire les fichiers
**après le `COMMIT`** laisserait, sur un refus d'écriture, une base déjà remplacée (M-a7).

**AC8b — L'import ne dépend pas de la connexion du client.** Une fois l'envoi reçu sur disque (AC11), le reste de
l'import (vérification, sauvegarde de sécurité, écriture des fichiers, restauration) s'exécute dans une tâche
`tokio::spawn` que le handler **attend** : si le client abandonne (délai du navigateur ou du proxy, onglet fermé), hyper
abandonne le futur du handler mais pas la tâche, qui va au bout (succès ou refus, audit et journal compris) — au lieu
d'une restauration interrompue au milieu de `restore_tables_in_tx`. Un `JoinError` (panique) rend 500 et est journalisé
— branche **non testée** (il faudrait injecter une panique dans le chemin de production) : angle mort écrit (R12, P3).
Garde lexicale dans `routes/admin.rs` (`mod tests`) : le corps de `full_import` contient `tokio::spawn(` et l'appel de
`run_backup_and_restore` se trouve **dans** la fermeture de ce `spawn` (C-31-1-26). À la 31-1c, le `backup_lock` de
l'import est pris **dans** cette même fermeture, avant `run_backup_and_restore` (AC27) : le garde vit dans la tâche,
pas dans le handler.

**AC9 — Ce que l'import rend.** La réponse gagne `filesRestored` (fichiers de l'archive écrits ou déjà intacts) et
`filesMissing` : nombre de fichiers référencés par la base **restaurée** qui, après l'import, ne sont pas lisibles et
intacts dans `KESH_DOCUMENTS_DIR` (calculé après l'écriture, pour les deux formats : une sauvegarde 1 restaurée sur la
même installation rend 0, sur une machine neuve rend le nombre de pièces perdues). L'audit `admin.full_import` porte
`files_restored`, `files_missing`, `source_format_version`. La réponse ne nomme aucun fichier (le journal du serveur,
si : `warn!` par fichier manquant).

**AC10 — Ancien format accepté, et c'est testé sur un fichier réel.** Un `.keshbackup` de format 1 produit par le
**vrai** exporteur de la 0.14.0 s'importe : `200`, `filesRestored = 0`, `filesMissing` égal au nombre de pièces que
référence la fixture (le dossier des documents du test est vide). La fixture est **produite avant tout changement de
code**, au commit `44ef2079` (base de la branche, code de la 0.14.0 inchangé), par un test `#[ignore]` de
`admin_full_export_e2e.rs` (`generate_fixture_0_14_0`) qui seede une base de test (société, un document fournisseur
écrit par `store_document`, une facture figée), appelle `GET /api/v1/admin/full-export` et écrit la réponse dans
`crates/kesh-api/tests/fixtures/keshbackup-0.14.0-format1.keshbackup` ; elle est versionnée **dans le premier commit**
de la 31-1a, avec son empreinte SHA-256 au Dev Agent Record ; données synthétiques seulement (aucune donnée de Guy).
Le test d'import la lit telle quelle (`include_bytes!`), jamais régénérée ensuite — le générateur reste `#[ignore]` et
son doc-comment dit pourquoi. Les tests actuels de l'import restent verts sans modification de leur assertion
(C-31-1-31), mais **ils ne restent pas des tests du format 1** (constat corrigé en P2) : `build_test_backup` écrit
`format_version_override.unwrap_or(BACKUP_FORMAT_VERSION)` (`import.rs:308`), si bien que ceux qui passent `None`
deviennent des tests du format 2 **sans fichier** (bijection vide, valide) ; et le `rezip` de
`admin_full_import_e2e.rs:263-281` recopie le `formatVersion` de l'export (2). Le format 1 n'est couvert que par la
fixture réelle, par les tests qui forcent `Some(1)` et par AC13 (sauvegarde format 1) — c'est voulu, et dit ici.

**AC11 — Import reçu sur disque.** `read_upload` crée au besoin `KESH_ADMIN_BACKUP_DIR` (`ensure_backup_dir`, AC4),
puis copie le champ `file` **par morceaux** (`field.chunk()`) dans un fichier temporaire **anonyme** de ce dossier
(`tempfile::tempfile_in` : sans nom dans le dossier, il disparaît avec le descripteur, même après un `kill -9` ou un
OOM — aucun résidu de plusieurs Gio ; C-31-1-33 ; vrai **par construction** et non testé : un temporaire anonyme
n'apparaît dans aucun `read_dir`, si bien qu'un test « aucun temporaire laissé » passerait sous toute mutation — retiré
de T-a6b en P2) ; `parse_and_verify` est générique sur `R: Read + Seek` (`File` en production, `Cursor` dans
les tests), plus un `&[u8]`, et rend la `ZipArchive<R>` dans `ParsedBackup` (AC8). Les tables NDJSON restent lues en mémoire, bornées (AC6b). Dossier absent → créé ; dossier non inscriptible
→ 500 `AdminPreImportBackupFailed` (rien n'a commencé ; message existant). Le verrou d'installation `_kesh_version FOR
UPDATE` (et, à la 31-1c, `backup_lock`) n'est pris **qu'après** la réception et `parse_and_verify` (C-31-1-30).

**AC12 — Plafond d'import relevé, et l'export prévient.** `KESH_ADMIN_IMPORT_MAX_MB` : défaut **2048** (au lieu de 512),
bornes inchangées [1, 10240] (C-31-1-8). Si l'archive produite par un export dépasse ce plafond, l'en-tête
`X-Kesh-Backup-Exceeds-Import-Limit: <plafond en Mio>` est posé (la 31-1b l'affiche) — un fichier qu'on ne pourrait
pas réimporter sans toucher au `.env` ne doit pas être découvert le jour de la panne.

**AC13 — De bout en bout, sur une installation vierge (test d'intégration Rust).** Dans
`crates/kesh-api/tests/admin_full_import_e2e.rs` : installation A seedée avec un document fournisseur (fichier écrit
par `store_document` dans le dossier A) et une facture figée (fichier + quatre colonnes) ; export ; puis base
**vidée** (`truncate_all` + `seed_stub_company_only`, l'état de `/setup`) **et dossier des documents B vide** ; admin
créé ; import ; assertions : `200`, `filesRestored = 2`, `filesMissing = 0` ; les deux fichiers existent dans B avec
leur contenu ; `GET` du document fournisseur et du PDF figé rendent `200` et les octets d'origine (plus de 410). Et la
même chose avec une sauvegarde **format 1** : `filesMissing = 2`, les deux `GET` rendent 410 (le comportement que
#503 décrit, désormais annoncé).

**AC14 — Manuel d'administration et CHANGELOG.** `docs/manual/fr/admin-manual.tex` : le format 2 et ce qu'il emporte
(§ *Export/import d'installation*, l. 1826) ; les phrases « ni dans le `.keshbackup` » / « sans les fichiers » / note
de bas de page #503 (l. ~1465, 1474, 1828, tableau l. ~1950-1963) **corrigées** ; ce qui n'est pas emporté (fichiers non
référencés, `filesNotIncluded`, `.env`, réglage de la sauvegarde automatique — ce dernier écrit par la 31-1c) ; le
volume, le débordement dans `KESH_ADMIN_BACKUP_DIR`, le nouveau défaut de `KESH_ADMIN_IMPORT_MAX_MB` (tableau l. ~783 et
§ *Variables d'environnement*) ; une sauvegarde de la 0.15.0 ne s'importe pas dans une 0.14.x (`formatVersion`).
`.env.example` : commentaire du plafond à 2048. PDF régénéré. CHANGELOG : section **`## [0.15.0] — Non publié`**,
l'en-tête exact qu'exige `scripts/prepare-release.sh` (l. 248 ; il refuse toute autre forme et plus d'une section) —
**créée** par la première des stories de l'epic à merger (31-1a, 31-2 ou 31-3, qui la créent chacune sur leur
branche), **complétée** sinon ; à l'union, une seule section (R7, P3 : `[Unreleased]` n'existe pas au CHANGELOG).
Rubrique `### Ajouté`.
`docs/manual/fr/user-manual.tex:1909-1910` (« n'emporte pas les documents ») corrigé.

**Proxy inverse** (C-31-1-25) : l'exemple Nginx (`admin-manual.tex:329-341`) gagne un bloc `location /api/v1/admin/`
avec `client_max_body_size 2048M;` (égal au plafond de Kesh, à relever avec lui), `proxy_read_timeout 3600s;`,
`proxy_send_timeout 3600s;` et `proxy_request_buffering off;` — le `client_max_body_size 50M` et le `proxy_read_timeout
300s` du bloc général refuseraient (413) ou couperaient l'import et l'export d'une sauvegarde avec pièces. Un `location`
n'hérite pas des directives `proxy_*` d'un autre : le bloc neuf **répète** le `proxy_pass` et les quatre
`proxy_set_header` de l'exemple (`admin-manual.tex:330-334`), sans quoi Nginx ne relaie pas ou perd `X-Forwarded-*`
(F8, P3). Caddy
(l. ~360) et Traefik (l. ~377, délais de lecture des points d'entrée) : une phrase chacun sur les limites de taille et
de durée à vérifier pour la version utilisée ; Portail DSM (l. ~618) : idem, sans inventer de chemin d'écran non vérifié.
Volume : la sauvegarde pré-import contient désormais toutes les pièces, une copie à chaque import, jamais purgée ;
le dire, avec l'ordre de grandeur « taille du dossier des documents + base ».

**Inventaire clos des phrases à traiter** (grep des valeurs `503`, `512`, `n'emporte`, `sans les fichiers`, `aucun
binaire`, `files/`, `formatVersion`, relevé en P1 ; **étendu en P2** aux sujets que le code de la 31-1a ou de la 31-1c
rend faux — `KESH_ADMIN_BACKUP_DIR`, `INMEM`, `temporaire`, `./backup`, `keshbackup` — sur `docs/`, `crates/`,
`frontend/`, `website/`, `.env.example`, `README.md` ; T-a8 et T-c7 le rejouent et traitent tout site neuf).

**Clos par la forme** (F4, P3 : l'extension de la P2 avait été rejouée sur les `.tex` et `config.rs`, pas sur
`.env.example`, `README.md` ni les doc-comments voisins — huit sites manquaient). L'inventaire des **sujets** est
**l'ensemble des lignes que rend cette commande**, et non une liste qu'on complète :

```sh
P='KESH\\?_ADMIN\\?_(BACKUP\\?_DIR|EXPORT\\?_INMEM\\?_MB|IMPORT\\?_MAX\\?_MB)|admin_backup_dir|admin_export_inmem_mib|admin_import_max_mib|build_keshbackup|pré-import|pre-import|sauvegarde de sécurité|in-memory|spill'
{ grep -rnE "$P" docs/manual/fr/*.tex docs/api-external.md README.md .env.example website
  grep -rnE "^\s*//.*($P)" crates/kesh-api/src crates/kesh-db/src; }
```

(`\\?_` : le LaTeX écrit `KESH\_ADMIN\_…`, qu'un motif sans échappement manque — c'est ainsi que `admin-manual.tex:196`
échappait au grep des noms.) À `c7639e80`, elle rend **94 lignes** (27 du manuel d'administration, 9 de
`.env.example`, 2 du `README.md`, 56 de commentaires dans `crates/`). **Chaque ligne** est soit au tableau ci-dessous,
soit triée « intacte » au Dev Agent Record avec sa raison (code d'exemple de compose, historique, sans objet) ; au dev,
la commande est rejouée sur le code modifié et toute ligne neuve est triée de même. Le grep des **valeurs** (P1) reste
à rejouer à côté (T-a8).

| Site | Traitement |
|---|---|
| `admin-manual.tex:783` (tableau, `512`), `:1945` (« défaut 512 ») | → 2048 |
| `admin-manual.tex:1465`, `:1474` (+ note de bas de page), `:1828`, tableau `:1950-1963` | corrigés (le `.keshbackup` 2 emporte les pièces) |
| `admin-manual.tex:1608` (Hyper Backup), `:1765` (restauration DSM, `documents/`), `:886` | relus ; la copie du dossier des documents reste recommandée pour les sauvegardes faites par dump |
| `admin-manual.tex:2026` (mise à jour vers la 0.14.0), `:2044` (scripts de dump) | **vrais à leur date / pour leur objet** : laissés, triés au Dev Agent Record |
| `user-manual.tex:1909-1910` | corrigé |
| `user-manual.tex:1155`, `:1161` (fichier figé disparu après une restauration) | relus ; vrai pour une sauvegarde 1 ou un dump : nuancé |
| `docs/api-external.md:721`, `:751` (`410 … n'emporte pas les documents (#503)`) | corrigés : le 410 ne frappe plus qu'une restauration d'un format 1 ou d'un dump sans le dossier |
| `crates/kesh-api/src/errors.rs:612-616` (doc-comment « `files/` non-vide, `formatVersion > 1` ») | réécrit |
| `crates/kesh-api/src/admin_backup/import.rs:1-12` (en-tête : « `files/` vide », « `formatVersion ≤ 1` ») | réécrit |
| `crates/kesh-api/src/admin_backup/manifest.rs:12-15` (« Figée à `1` ») | réécrit |
| `crates/kesh-api/src/admin_backup/export.rs` (commentaire « aucun binaire stocké en v0.2 », l. 94 ; en-tête de module « `files/` # dossier vide (forward-compat) », l. 19) | réécrits (31-1a) |
| `crates/kesh-api/src/admin_backup/import.rs:93` (message rendu au client « le dossier files/ doit être vide (aucun binaire en v0.2) ») | précisé « format 1 » (31-1a) |
| `crates/kesh-api/src/config.rs:249`, `:927-948` (doc et messages « défaut 512 ») ; `.env.example:185` (commentaire « défaut 512 ») et `:186` (valeur) | → 2048 |
| `admin-manual.tex:196` (« Hors Docker, posez aussi `KESH_ADMIN_BACKUP_DIR` … sans quoi tout import d'installation est refusé ») | **devenu faux à la 31-1a** : l'export aussi l'exige (AC4) — « tout import et tout export » (31-1a) |
| `admin-manual.tex:782` (« au-delà : streaming ») et `:1946` (« streamé via un fichier temporaire ») | l'emplacement du temporaire change (`KESH_ADMIN_BACKUP_DIR`, AC4, aujourd'hui `std::env::temp_dir()`, `routes/admin.rs:80`) — réécrits (31-1a) |
| `admin-manual.tex:784` (tableau, « Dossier de sauvegarde de sécurité pré-import »), `:843`, `:1947` (§ *Variables*, même description) | le dossier porte désormais aussi le temporaire de l'export et de l'import (31-1a) ; `auto/` ajouté à la 31-1c (AC39) |
| `admin-manual.tex:1466`, `:1608` (`./backup` décrit comme dossier de la seule sauvegarde pré-import) | complétés à la 31-1c (AC39 : `auto/`, volume) |
| `admin-manual.tex:2268` (mise à jour : ajouter le montage de la sauvegarde pré-import) | vrai pour son objet (le montage) : laissé, trié au Dev Agent Record |
| `admin-manual.tex:2194`, `:2199` (le compose de la 0.13.0 monte `./backup`, sauvegarde pré-import) | **vrais à leur date** : laissés, triés au Dev Agent Record (R11, P3 ; la P2 avait cru `:2199` hors sujet sur la foi d'un `grep` en erreur, cf. Change Log P3) |
| `admin-manual.tex:1721`, `:1739` (`backup/` dans le dossier du compose, instantanés et visibilité), `:1887` (filet de l'import), `:1958` (ligne Hyper Backup du tableau) | vrais après la 31-1 : laissés ; `:1958` refaite avec le tableau par l'AC39 |
| `.env.example:88-90` (« Hors Docker, poser aussi `KESH_ADMIN_BACKUP_DIR` … sans quoi tout import d'installation est refusé ») et `README.md:93` (même phrase) | **devenus faux à la 31-1a** : l'export aussi l'exige (AC4) — « tout import et tout export » (31-1a), jumeaux de `admin-manual.tex:196` |
| `.env.example:182-183` (« spillée vers un fichier temporaire et streamée ») | emplacement du temporaire : `KESH_ADMIN_BACKUP_DIR` (31-1a), jumeau de `admin-manual.tex:782`, `:1946` |
| `.env.example:187-192` (« Dossier de sauvegarde de sécurité prise avant chaque import ») | le dossier porte aussi les temporaires de l'export et de l'import (31-1a), puis `auto/` (31-1c, AC39) |
| `crates/kesh-api/src/config.rs:240-243` (doc d'`admin_export_inmem_mib` : « spillée vers un fichier temporaire ») et `:252-261` (doc d'`admin_backup_dir` : « sauvegarde de sécurité pré-import », « l'import est refusé ») | réécrits (31-1a ; `auto/` ajouté à la 31-1c) |
| `crates/kesh-api/src/admin_backup/export.rs:44-55` (doc de `build_keshbackup` : « DC8 : l'assemblage est in-memory ») | réécrit (31-1a, AC4 : lecture de la base, puis `write_keshbackup` en un `spawn_blocking`) |
| `crates/kesh-api/src/routes/admin.rs:118-130` (doc de `full_import` : séquence en cinq temps, « cœur `build_keshbackup` ») | réécrit (31-1a : réception sur disque, tâche détachée, écriture des fichiers avant la base) |
| `website/roadmap.html`, brochure | laissés à la release, noté au Dev Agent Record |

### Partie b — `31-1b-restauration-installation-neuve`

**AC15 — Restaurer avant de configurer.** Sur l'assistant (`routes/onboarding/+page.svelte`), aux étapes **0 à 2**
(avant le choix démo / production), un lien secondaire **« Restaurer une sauvegarde »** mène à `/onboarding/restore`,
page neuve qui rend `AdminRestorePanel` (réutilisé, C-31-1-10) ; la garde de `routes/onboarding/+layout.ts` la laisse
passer comme `/onboarding`. Le lien n'apparaît que pour un **Administrateur** (le compte créé par `/setup` l'est) ; un
autre rôle ne le voit pas, et le serveur refuse de toute façon (403). La page porte la même garde de rôle que
`(app)/admin/backup/+page.ts:14-16` (`routes/onboarding/restore/+page.ts` : non-Admin → `redirect(302, '/onboarding')`),
si bien qu'elle ne s'affiche pas non plus par URL directe (L7, P2).

**AC16 — Après l'import, l'installation est celle de la sauvegarde.** Le panneau garde son comportement (déconnexion,
`/login`), déclenché désormais par le clic *Se reconnecter* (AC19) ; reconnecté avec les identifiants **de la sauvegarde**, l'utilisateur arrive dans l'application, pas dans
l'assistant (`force_onboarding_done_if_eligible`, inchangé) — vérifié en E2E (AC20).

**AC17 — Ce que le fichier ne contient pas, dit avant.** `AdminBackupPanel` et `AdminRestorePanel` affichent un avis
permanent : « Le fichier ne contient pas la configuration du serveur (fichier `.env` : messagerie, secret de session,
dossiers…). Conservez-la à part : sur une nouvelle machine, recréez-la avant d'importer. » (clé
`admin-backup-not-included-env`, quatre locales).

**AC18 — Ce que l'export a omis, dit après.** `downloadFullExport` lit les en-têtes de l'AC5 et de l'AC12 : si
`X-Kesh-Backup-Files-Not-Included > 0`, avis d'avertissement « N pièces référencées n'ont pas pu être incluses (fichier
absent, altéré ou illisible sur le serveur) — voir le journal du serveur » ; si `X-Kesh-Backup-Exceeds-Import-Limit` est posé,
avis « Ce fichier dépasse la taille d'import autorisée (N Mio) : pour le restaurer, relevez `KESH_ADMIN_IMPORT_MAX_MB` ».
En-têtes absents (serveur ancien) : aucun avis.

**AC18b — L'export et l'import échappent au délai de 30 s du client HTTP** (C-31-1-25). `frontend/src/lib/shared/utils/api-client.ts`
arme un délai de 30 s (`REQUEST_TIMEOUT_MS`, l. 42) jusqu'à l'arrivée des en-têtes, et rejoue un `GET` jusqu'à quatre
fois sur `TIMEOUT` (`fetchWithRetry`, l. 329) ; or le serveur n'envoie les en-têtes de l'export qu'après avoir assemblé
toute l'archive, et ne répond à l'import qu'après la restauration — des minutes avec les pièces. `getBlob` et
`postFormData` gagnent un paramètre d'options `{ timeoutMs?: number; retry?: boolean }` (défauts inchangés : 30 s, rejeu
des méthodes idempotentes) ; une constante exportée `LONG_OPERATION_TIMEOUT_MS = 60 * 60_000` (une heure) ;
`downloadFullExport` et `uploadFullImport` l'emploient avec `retry: false` (la 31-1c fera de même pour « Sauvegarder
maintenant », AC32). **Issue inconnue de l'import** (constat corrigé en P2) : aujourd'hui, `TIMEOUT` et
`NETWORK_ERROR` sont des `ApiError` (`status: 0`, `api-client.ts:289-292`) qui tombent dans le `default` de
`importErrorMessage` (`AdminRestorePanel.svelte:81-84`) et affichent « Le serveur ne répond pas. Réessayez
ultérieurement. » — qui invite à **réimporter** une installation peut-être déjà remplacée ; « L'état précédent a été
préservé » n'est atteint que par une erreur non `ApiError` (l. 86-89). Il en va de même d'un **502 ou 504** rendu par
un proxy inverse dont le délai est inférieur à la durée de l'import (Kesh n'émet ni l'un ni l'autre : `grep -rn
"BAD_GATEWAY\|GATEWAY_TIMEOUT" crates/kesh-api/src` est vide), alors que la tâche détachée de l'AC8b va au bout. Pour
ces quatre cas — `TIMEOUT`, `NETWORK_ERROR`, statut 502, statut 504 — de `POST full-import`, le panneau affiche « La
réponse du serveur n'est pas arrivée : l'import a pu aboutir. Reconnectez-vous pour vérifier l'état de
l'installation. » (clé `admin-restore-error-unknown-outcome`, quatre locales). Le 503 reste au `default` (Kesh
l'émet, `errors.rs:4236`). Tests Vitest de `api-client` (horloge simulée) : un appel par défaut est abandonné à 30 s ; un appel
avec `LONG_OPERATION_TIMEOUT_MS` ne l'est pas à 30 s ni à 59 min, l'est à 60 min ; un `getBlob` avec `retry: false` n'est
**pas** rejoué sur `TIMEOUT` (un seul `fetch`) ; `downloadFullExport` et `uploadFullImport` passent bien l'option
(espion sur `apiClient`). Tests du panneau (neufs : `AdminRestorePanel.test.ts` n'en a aucun sur ces cas
aujourd'hui) : `TIMEOUT`, `NETWORK_ERROR`, 502 et 504 → le message d'issue inconnue, ni « Le serveur ne répond pas »
ni « Erreur serveur (504) » ; 503 → message par défaut.

**AC19 — Ce que la restauration a rendu.** Le succès de l'import affiche, avant la déconnexion, `filesRestored` et,
si `filesMissing > 0`, un avertissement « N pièces référencées sont introuvables après la restauration : elles
répondront « document indisponible » ; restaurez le dossier des documents depuis une autre sauvegarde ». Après **tout**
import réussi, avec ou sans avertissement, le panneau attend un clic **« Se reconnecter »** avant la déconnexion et la
redirection vers `/login` : l'avis se lit sans délai à régler, et l'E2E l'observe sans course contre une redirection
(C-31-1-39, qui remplace la règle conditionnelle de C-31-1-11 ; constat R7 de la P2 : le panneau redirige aujourd'hui
immédiatement, `AdminRestorePanel.svelte:119-128`).

**AC20 — E2E de bout en bout (`frontend/tests/e2e/backup-restore-new-install.spec.ts`).** (1) `seedTestState('with-company')`,
connexion ; un document fournisseur importé par l'inbox (fixture synthétique `spc_e2e_invoice.png` passée par
`uniqueCopy` de `helpers/inbox.ts` — des octets propres au spec, pour que la suppression de l'étape 3 ne retire pas le
fichier d'une autre spec) et une facture validée dont le PDF est figé par un téléchargement ; (2) export par l'écran, fichier
enregistré par le runner ; (3) `seedTestState('setup-required')` **puis suppression des deux fichiers** du dossier des
documents (`DOCUMENTS_DIR` de `helpers/inbox.ts` ; garde bruyante : chaque fichier doit exister avant suppression et
ne plus exister après — sinon le spec échoue, il ne se tait pas) ; (4) `/setup` (nouvel admin), assistant, lien
**« Restaurer une sauvegarde »**, import du fichier, confirmation ; (5) avis de l'AC19 avec « 2 pièces restaurées »,
lu **avant** le clic *Se reconnecter* (qui seul déclenche la redirection, C-31-1-39), puis reconnexion avec les
identifiants de la sauvegarde ; (6) l'application s'ouvre (pas l'assistant) ; le document
fournisseur s'affiche dans son aperçu et le PDF de la facture se télécharge (`200`, octets égaux à ceux d'avant) ; les
deux fichiers existent de nouveau dans le dossier. Le spec remet l'état (`seedTestState`) en fin de test.

**AC21 — Manuel pas à pas.** `admin-manual.tex`, sous-section neuve **« Restaurer sur une nouvelle machine »**
(`\label{sec:restauration-machine-neuve}`) : (1) installer Kesh (compose, `.env` recréé depuis `.env.example` et la
copie conservée de l'ancien — `KESH_JWT_SECRET` neuf possible, seules les sessions en dépendent ; `KESH_ADMIN_IMPORT_MAX_MB`
si le fichier dépasse 2048 Mio ; **le proxy inverse** réglé pour la taille et la durée de l'envoi, § de l'AC14) ;
(2) `/setup` ; (3) *Restaurer une sauvegarde* depuis l'assistant ; (4) se reconnecter
avec les identifiants **de la sauvegarde** ; (5) vérifier : une écriture, un document fournisseur, un PDF de facture ;
lire l'avis des pièces manquantes. Le § *Test de restauration périodique* (l. ~1573) renvoie à cette procédure. PDF
régénéré. **Captures (thème clair) de chaque état touché**, dans `/home/gcorbaz/devel/kesh-gate-logs/311b-captures/`,
citées au Dev Agent Record et au prompt de revue (celles qui servent au manuel copiées dans
`docs/manual/fr/screenshots/` si l'usage du manuel le permet) : (a) assistant, étape 0, avec le lien *Restaurer une
sauvegarde* ; (b) `/onboarding/restore` ; (c) avis permanent `.env` sur `AdminBackupPanel` ; (d) le même sur
`AdminRestorePanel` ; (e) avis d'export « N pièces non incluses » ; (f) avis d'export « dépasse la taille d'import » ;
(g) fin d'import `filesMissing = 0` (avis des pièces restaurées et bouton *Se reconnecter*) ; (h) fin d'import
`filesMissing > 0` (avertissement et bouton *Se reconnecter*) ; (i) message d'issue inconnue de l'import (AC18b).
**Comment chaque état s'atteint** (R8, P3) : (e) en retirant un fichier du dossier des documents avant l'export ; (h)
en en retirant un avant l'import ; (f) et (i) **par interception réseau** (`page.route` : en-tête
`X-Kesh-Backup-Exceeds-Import-Limit` ajouté à la réponse de l'export ; réponse de l'import remplacée par un 504) — une
archive de plus de 2048 Mio ou une coupure réelle ne se fabriquent pas en E2E. Le Dev Agent Record dit, pour chaque
capture, par quelle voie elle a été obtenue.

**AC22 — i18n et textes.** Clés neuves en **quatre locales** (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`),
fallbacks Svelte identiques au français ; `npm run lint-i18n-ownership` vert ; `admin-backup-page-description` (« … dans
un fichier .keshbackup unique … ») complétée : « … les données **et les pièces justificatives** … ».

### Partie c — `31-1c-sauvegarde-automatique`

**AC23 — Migration `20261024000001_backup_schedule.sql` (non breaking).** Table **système** `_kesh_backup_schedule`
(singleton, `id TINYINT PRIMARY KEY CHECK (id = 1)`), hors `TABLES_TO_TRUNCATE` (C-31-1-15) :
`enabled BOOLEAN NOT NULL DEFAULT TRUE`, `interval_hours INT NOT NULL DEFAULT 24 CHECK (interval_hours IN (6, 12, 24,
168))`, `retention_count INT NOT NULL DEFAULT 7 CHECK (retention_count BETWEEN 1 AND 60)`, `last_attempt_at
DATETIME(3) NULL`, `last_success_at DATETIME(3) NULL`, `last_success_file VARCHAR(255) NULL`, `last_success_bytes
BIGINT NULL`, `last_status VARCHAR(32) NULL CHECK (last_status IN ('ok','partial','failed','insufficient_space',
'disk_full'))` (`partial` : sauvegarde écrite, mais `filesNotIncluded` non vide, AC28), `last_files_not_included INT
NULL`, `last_complete_file VARCHAR(255) NULL` (nom de la plus récente sauvegarde automatique **complète**, que la
rétention ne supprime jamais, AC29), `version INT NOT NULL DEFAULT 1`, `updated_at DATETIME(3) …`; `ENGINE=InnoDB`,
charset et collation explicites. Puis `INSERT INTO _kesh_backup_schedule (id) VALUES (1);`. Politique : **non
breaking** (`CREATE TABLE` d'une entité neuve + ligne singleton) → **pas** de bump `kesh_version_min_required`, pas de
bump des crates dans la story (la release 0.15.0 les fera, `prepare-release.sh`) ; mais, comme toute migration, une
0.14.0 refuse de démarrer sur une base migrée (`CLAUDE.md` § P1) : le retour à la 0.14.0 passe par la sauvegarde
d'avant la mise à jour — écrit au CHANGELOG. P5 : ligne au tableau de `docs/migrations-idempotence-audit.md`
(`tracked-by-sqlx`) + **les deux sites du total** (82 → 83 : en-tête `## Table d'audit (N migrations)` l. 17 et ligne
`Total` l. 108, dont la ventilation par story gagne `+ 1 Story 31-1c`) + compteur de partition `tracked-by-sqlx` (72 → 73,
l. 110 ; `yes` reste 10 ; somme 83), recomptés (`ls … | wc -l`). P6 :
`grep -rn "migrations.len()\|apply_migrations_up_to" crates/` inspecté, résultat écrit au Dev Agent Record. P7 :
l'`INSERT` est trié en `EXEMPT_MIGRATIONS` (« INSERT sur _kesh_backup_schedule, table système hors
TABLES_TO_TRUNCATE : jamais exportée ni restaurée » — précédent `_kesh_version`, `post_restore.rs:523`) ; la table ne
déplace pas la fenêtre d'importabilité (`creates_application_table` ne compte que l'inventaire). **Squash de test** :
le squash est un dump `--no-data` (`regen-test-schema.sh:254`) qui ne réinjecte que la ligne de `_kesh_version`
(`:273-327`) ; régénéré tel quel, il porterait la table **vide**, et tout test monté sur le squash lirait une ligne
absente. Le script gagne donc la réinjection de la ligne `_kesh_backup_schedule` (relevée dans la base migrée, comme
`_kesh_version`, jamais codée en dur ; valeurs contrôlées par expression avant réinjection), et
`crates/kesh-db/tests/test_schema_guard.rs` une garde `squash_seeds_the_backup_schedule_row` (sur le modèle de
`squash_seeds_the_kesh_version_row`, l. 416) ; squash régénéré ; `test_schema_guard` vert.
`backup_inventory_matches_schema` exclut `_kesh_backup_schedule` comme `_kesh_version`.

**AC24 — Ce qui est vérifié de cette table.** Tests : la ligne singleton existe après migration ; une seconde ligne est
refusée ; `interval_hours = 5`, `retention_count = 0` et `last_status = 'x'` refusés ; la ligne existe **aussi sur le
squash** (garde de l'AC23) ; l'export d'installation **ne contient pas** la table ;
un import **ne la modifie pas** (réglages et état de la destination conservés) ; `reset_demo` ne la touche pas.

**AC25 — Le moment d'une sauvegarde (fonction pure).** `auto_backup::is_due(schedule, now) -> bool` : vrai si et
seulement si `enabled` **et** (`last_success_at` nul ou `now - last_success_at ≥ interval`) **et** (`last_attempt_at`
nul ou `now - last_attempt_at ≥ min(interval, 1 h)`) — un échec se retente au bout d'une heure, pas en boucle, pas un
jour plus tard (C-31-1-17). Tests aux bornes (égalité, une milliseconde avant, désactivé, jamais tenté).

**AC26 — La tâche de fond.** `main.rs` (et lui seul — ni `build_router`, ni les tests) lance une tâche
`auto_backup::spawn(state.clone())` : premier contrôle **5 minutes** après le démarrage, puis toutes les **5 minutes**
(`tokio::time::interval`, `MissedTickBehavior::Skip`) ; à chaque tour, lit le réglage, et si `is_due` appelle
`run_once(&state, env.clone(), now, Trigger::Schedule)`. **Signature unique** (fixée en P2, F17 ; `env` possédé depuis
la P3, pour entrer dans le `spawn_blocking` de l'AC4) : `run_once(state: &AppState, env: Arc<dyn BackupEnv>, now:
NaiveDateTime, trigger: Trigger) -> Result<RunOutcome, …>`, où `RunOutcome` vaut `Skipped` (verrou pris ou réservation
perdue, AC27 : rien d'écrit) ou `Done(statut)` ; en production, `env` est `Arc::new(RealBackupEnv)` (construit par
`spawn` et par la route `POST …/run`, pas porté par `AppState`) ; les tests passent leur `BackupEnv` et leur `now`.
Toutes les mentions de `run_once` de cette fiche désignent cette signature. La boucle de fond ignore `Skipped` (journal
`info!`). Une erreur de lecture est journalisée et le tour suivant réessaie ; une panique dans un tour n'arrête pas la
boucle (le tour s'exécute dans un `tokio::spawn` dont le `JoinError` est journalisé, C-31-1-18). **Le lancement se
prouve** (R4, P3 : c'est la demande de #503, « faite par Kesh lui-même », et rien ne rougissait si `main.rs` oubliait
l'appel) : garde lexicale dans `auto_backup.rs` (`mod tests`, sur `include_str!` de `main.rs` et `lib.rs`) —
`auto_backup::spawn(` apparaît **exactement une fois** dans `main.rs` et **aucune** dans `lib.rs` (où elle se
déclencherait dans chaque test monté par `build_router`) ; elle tue M-c21. La cadence (5 min, `Skip`) et l'isolation
d'une panique ne sont **pas** testées : quelques lignes relues en revue ; les tester demanderait d'extraire et
d'horloger la boucle (angle mort écrit).

**AC27 — Une seule sauvegarde à la fois.** `AppState` gagne `backup_lock: Arc<tokio::sync::Mutex<()>>` (défaut dans
`new_for_tests` ; **neuf sites** mis à jour : les huit littéraux `AppState { … }` — `main.rs:355`,
`middleware/auth.rs:267`, `tests/password_recovery_e2e.rs:127`, `tests/invoice_frozen_pdf_e2e.rs:102`,
`tests/invoice_send_email_e2e.rs:103,1034,1147`, `tests/setup_admin_e2e.rs:81` — et le `Self { … }` de
`AppState::new_for_tests`, `lib.rs:105` ; recompter au dev par `grep -rnE "AppState \{$" --include=*.rs crates | grep
-vE "struct |impl |fn "` — huit lignes, plus le `Self {` ; sans le filtre, la commande rend douze lignes : la
définition, l'`impl`, une signature `fn test_state() -> AppState {` et un commentaire). **Une seule règle par
acquéreur** (C-31-1-45, P3 : la forme de la P2 faisait prendre le verrou à la fois par le handler de `POST …/run` et
par `run_once`, ce qu'un mutex non réentrant ne permet pas) :

- **`run_once` prend lui-même le verrou**, quel que soit `trigger`, en `try_lock_owned()` sur un clone de l'`Arc` —
  pris → `Ok(RunOutcome::Skipped)`, rien d'écrit en base ni sur disque (journal `info!` « tour sauté : une autre
  sauvegarde est en cours ») ;
- la **boucle de fond** ignore `Skipped` (nouveau contrôle 5 minutes plus tard) ;
- le handler de **`POST …/run`** n'acquiert rien : il lance `tokio::spawn` d'une fermeture qui appelle `run_once(…,
  Trigger::Manual)`, attend le `JoinHandle`, et traduit `Skipped` en **409 `BACKUP_IN_PROGRESS`** ; le garde vit dans la
  tâche **par construction** (un client parti n'interrompt ni l'écriture ni la détention) ;
- l'**import** prend `lock_owned().await` dans la fermeture de son `spawn` (AC8b), après `parse_and_verify` ;
- l'**export manuel** prend `lock_owned().await` pour l'assemblage.

Toujours un garde **possédé** (`lock_owned` / `try_lock_owned`), jamais un garde emprunté. Garde lexicale dans
`routes/admin.rs` et `auto_backup.rs` (`mod tests`) — une garde contre l'inattention, pas contre une forme délibérée
(une variable intermédiaire la contourne ; F11, P3) : aucune occurrence de `backup_lock.lock(` ni
`backup_lock.try_lock(` (formes empruntées) ; `try_lock_owned` n'apparaît **que** dans `auto_backup.rs` (jamais dans
`routes/admin.rs`) ; dans `full_import`, `lock_owned` se trouve dans la fermeture du `spawn`. **Ordre** : `backup_lock` est toujours pris **avant** `_kesh_version FOR UPDATE`
(seul l'import prend les deux : `run_backup_and_restore` ouvre le `FOR UPDATE` après) — l'ordre inverse ferait tenir
le verrou de ligne pendant l'attente d'un export de plusieurs minutes. **Fenêtre de détention** (C-31-1-30) : l'import le
prend **après** la réception de l'envoi et `parse_and_verify` (un client lent ne bloque pas les sauvegardes pendant
l'envoi) et le garde jusqu'à la fin de la restauration (sauvegarde de sécurité, écriture des fichiers, restauration) ;
l'export manuel le prend pour l'**assemblage** de l'archive dans son temporaire et le **rend avant la diffusion** (un
téléchargement lent ne bloque rien). L'export n'est pas détaché : un client parti pendant l'assemblage abandonne le
futur du handler, donc **relâche le garde**, tandis que le `spawn_blocking` d'assemblage (AC4), que rien n'interrompt,
va au bout dans son temporaire anonyme puis le perd ; une sauvegarde peut alors s'y superposer — sans effet sur les
données (instantané, fichiers immuables), seulement sur le disque et le processeur (F11, P3 : la P2 disait « sans
effet », ce qui était inexact). Puis `run_once` **réserve** son tour en base : `UPDATE _kesh_backup_schedule SET
last_attempt_at = ? WHERE id = 1 AND last_attempt_at <=> ?` (valeur lue) ; `rows_affected = 0` → un autre processus l'a
prise, `Skipped` (`debug!`). L'instantané de la base (Story 28-3) et l'immuabilité des fichiers nommés par
leur contenu (P-3) garantissent une archive cohérente même pendant une saisie.

**AC28 — Écrire la sauvegarde.** `run_once` crée `KESH_ADMIN_BACKUP_DIR/auto/` (`0700`, `DirBuilder` récursif comme
`write_pre_import_backup`), supprime d'abord les `kesh-auto-*.keshbackup.partial` qu'il y trouve (les siens, laissés par
un arrêt brutal — le verrou garantit qu'aucun n'est en cours), puis écrit
`kesh-auto-<AAAAMMJJ>T<HHMMSSmmm>Z.keshbackup` (UTC, **millisecondes comprises**, dérivé du `now` passé à `run_once` —
C-31-1-44, P2 : à la seconde, deux sauvegardes rapprochées, ou les appels successifs de l'AC36, se seraient heurtées au
refus d'écraser de `write_backup_file` — `create_new`, `routes/admin.rs:591`, et contrôle du nom final, `:615-619`)
par `write_keshbackup` dans le `.partial` de `write_backup_file` (`0600`, renommage, jamais d'écrasement), le tout dans
le seul `spawn_blocking` de l'AC4. Sous `backup_lock` les sauvegardes sont successives et
chacune dure plus d'une milliseconde (instantané, `fsync`) ; une collision résiduelle (`AlreadyExists`) vaut `failed`,
journalisée, sans autre traitement. Avant de la tenir pour écrite, `run_once` **la relit** : ouverture `ZipArchive`,
lecture du `manifest.json` — ce qui prouve que l'annuaire central et le manifeste sont lisibles, **pas** l'intégrité
des entrées `data/` et `files/` (leurs CRC ne sont pas relus : ce serait relire toute l'archive ; dit au manuel, F11) ;
un échec vaut `failed`, le fichier fautif est supprimé, la rétention ne tourne pas.
Succès : `last_success_at`, `last_success_file`, `last_success_bytes`, `last_files_not_included`, et
`last_status = 'ok'` si `filesNotIncluded` est **vide** (alors `last_complete_file` = ce fichier), **`'partial'`** sinon
— une sauvegarde à qui il manque des pièces n'est pas une sauvegarde complète pour le panneau (AC33) ni pour la
rétention (AC29) (C-31-1-27) ; journal `info!` (`warn!` pour `partial`, avec le nombre de pièces).

**AC29 — Rétention.** Après un succès (`ok` ou `partial`) **seulement** : les fichiers **réguliers** de `auto/`
(`symlink_metadata`, un lien n'est jamais suivi ni supprimé) dont le nom satisfait exactement
`^kesh-auto-[0-9]{8}T[0-9]{9}Z\.keshbackup$` (`[0-9]` et non `\d`, qui est Unicode dans le crate `regex` ; F7, P3)
sont triés par nom (= par date), et les plus anciens au-delà de
`retention_count` sont supprimés ; **jamais** un autre fichier (`kesh-pre-import-*`, fichier déposé à la main, nom
voisin), jamais la sauvegarde qui vient d'être écrite, **jamais `last_complete_file`** — la dernière sauvegarde
complète survit à une suite de sauvegardes `partial` (dossier des documents démonté après une mise à jour du NAS : sans
cette règle, `retention_count` jours suffiraient à ne plus garder aucune sauvegarde portant les pièces). Elle compte
alors en plus des `retention_count` conservées. Un échec de suppression est journalisé et n'efface pas le succès.
Un échec de sauvegarde (`failed`, `disk_full`, `insufficient_space`) ne supprime **rien**.

**AC30 — Disque plein, avant et pendant.** Avant d'écrire : espace libre du système de fichiers de `auto/`
(`rustix::fs::statvfs`, `f_bavail × f_frsize` — l'espace accessible, non `f_bfree`, qui compterait la réserve de
`root` ; dépendance directe ajoutée — `rustix` 1.1.4 est au `Cargo.lock` ; C-31-1-19) comparé au seuil
`max(last_success_bytes, documents_bytes) × 1,2 + 256 Mio`, où `documents_bytes` est la somme des tailles des fichiers
réguliers du premier niveau de `KESH_DOCUMENTS_DIR` (un `read_dir` et un `metadata()` par fichier ; `store_document`
n'écrit qu'à ce niveau) — majorant des pièces que l'archive emportera, connu **avant** la première sauvegarde et
après un remontage du dossier des documents (C-31-1-42, P2 : sans lui, la première sauvegarde automatique, activée par
défaut et sans historique, écrivait jusqu'à `ENOSPC` sur le volume de la base) ; `last_success_bytes` nul compte 0.
Insuffisant → rien n'est écrit, `last_status = 'insufficient_space'`, `warn!` avec les deux nombres. Comme un échec
ne fait jamais tourner la rétention (AC29), baisser le nombre conservé ne libère rien tant qu'aucune sauvegarde ne
réussit : **l'issue est dite à l'écran** (F13, P3) — le message du statut `insufficient_space` au panneau (AC31, AC33)
porte la marche à suivre (les deux nombres restent au journal : la table ne les garde pas) : « libérez de la place sur le disque du dossier de
sauvegarde, par exemple en supprimant d'anciennes sauvegardes de `./backup/auto` ; le nombre conservé ne s'applique
qu'après une sauvegarde réussie » ; le manuel le répète (AC39). Pas de suppression depuis l'écran (C-31-1-51). Pendant : toute erreur d'écriture supprime le
`.partial` (déjà fait par `write_backup_file`) ; la classification se fait sur l'`io::Error` conservée par la primitive
(AC4), jamais sur un texte : fonction pure `classify_write_error(&io::Error) -> FailureStatus`, `disk_full` si
`kind()` vaut `ErrorKind::StorageFull` (ENOSPC) ou `ErrorKind::QuotaExceeded` (EDQUOT, quota d'un dossier partagé
Synology ; stables sur la chaîne 1.96.1 du dépôt) — **`kind()` d'abord** : une `ZipError` convertie par
`io::Error::from` garde son `kind()` mais perd `raw_os_error()` (`zip-2.4.2/src/result.rs:52-64`, F10, P3) ; le repli
`raw_os_error()` 28 ou 122 ne vaut que pour une erreur née hors du crate ; `'failed'` sinon (C-31-1-32). **La
rétention ne tourne pas**, les sauvegardes existantes restent. Le trait d'injection de l'AC36 (`BackupEnv: Send +
Sync`, qui remplace `FreeSpace`, passé en `Arc<dyn BackupEnv>`) porte deux méthodes : `free_bytes(dir)` et
`wrap_writer(file: File) -> Box<dyn Write + Seek + Send>` — l'écrivain **possède** un **clone** du fichier
(`try_clone`, même descripteur, AC4), si bien qu'il entre dans le `spawn_blocking` ; l'original reste à
`write_backup_file`, qui appelle `sync_all()` dessus une fois l'archive finie (F10 de la P2 : sur btrfs et sur un
partage réseau, `ENOSPC`/`EDQUOT` surgissent souvent au `fsync` ; C-31-1-46, P3 : l'emprunt `&'a mut File` de la P2
n'entrait pas dans un `spawn_blocking`) ; `classify_write_error` s'applique aussi à l'erreur de cette synchronisation.
L'implémentation de production rend le clone tel quel ; celle de test un écrivain qui échoue avec
`ErrorKind::StorageFull` après N octets.
`documents_bytes` n'est pas injecté : les tests déposent de vrais fichiers dans leur dossier des documents. Le seuil
protège surtout les voisins du disque (la base elle-même, sur un NAS, partage souvent le volume).

**AC31 — Échec signalé.** (a) **Dans le panneau** (AC33), tout `last_status` autre que `ok` est visible, avec la
cause en clair, l'heure de la dernière tentative et de la dernière réussite : un **échec** (`failed`,
`insufficient_space`, `disk_full`) en bandeau d'erreur (`role="alert"`) ; `partial` en **avertissement**
(`role="status"`, « incomplète : N pièces non incluses ») — un seul rôle par statut (R8, P2). (b) **Dans
l'application**, pour un **Administrateur** seulement et **seulement si la sauvegarde automatique est activée** (une
fois désactivée, un ancien échec n'allume plus rien : plus aucun tour ne mettrait `last_status` à jour — R8, F15, P2),
un bandeau discret sous l'en-tête (`routes/(app)/+layout.svelte`, **`role="status"`** — un `role="alert"` ferait rougir
`reports.spec.ts:242`, qui exige zéro `alert`, sur une base E2E dont la dernière réussite date de plus de deux
intervalles ; L5, P2) : « La dernière sauvegarde automatique a échoué — Administration → Sauvegarde complète » quand
`last_status` est un échec ; **ou** quand `last_success_at` est **non nul** et plus vieux que **deux** intervalles
(« aucune sauvegarde automatique réussie depuis le … ») — une sauvegarde qui cesse de tourner sans échouer (tâche
morte) se voit aussi (C-31-1-20). **`partial` n'allume pas le bandeau de l'application** (C-31-1-40, retrait de la P2) :
sur une installation dont des pièces manquent durablement (C-31-1-4 — le cas même de #503), il s'allumerait à vie sans
geste pour l'éteindre et apprendrait à l'ignorer, y compris l'échec qui partage sa place ; le dossier des documents
démonté, que la P1 voulait rendre visible, se voit déjà dans toute l'application (chaque pièce répond « document
indisponible »), au panneau et au journal (`warn!`), et la rétention garde la dernière sauvegarde complète (AC29). Un
`last_success_at` **nul** n'allume pas ce second cas (sinon le bandeau s'allumerait à chaque installation et pendant
les cinq minutes qui suivent chaque mise à jour, C-31-1-34) : une sauvegarde jamais tentée se lit dans le panneau
(« jamais tentée »), une jamais réussie mais tentée se voit par le premier cas. **Lecture** : le layout lit
`GET backup-schedule` à son chargement puis toutes les 30 minutes tant que l'application est ouverte, et après chaque
action du panneau ; un échec de cette lecture n'affiche **aucun** bandeau et ne casse pas le layout (pas de toast). Un
journal `error!` accompagne chaque échec. Pas d'entrée d'audit par sauvegarde réussie (bruit quotidien) ; l'état vit
dans la table.

**AC32 — Routes (Administrateur, refusées aux clés d'API).** Sous `admin_routes` (RBAC `require_admin_role`), dont la
couche anti-clé refuse déjà toute clé d'API, comme pour `users` et `email-templates` (`admin_pat_denied_e2e.rs:58-77`) ;
**seul le téléchargement** (`GET backups/{kind}/{name}`), qui livre une archive comme `full-export`, porte en plus
`ensure_not_pat` dans son handler, à l'image de `full_export` et `full_import` (`routes/admin.rs:37, 143`) — défense en
profondeur là où sort le secret, et non partout (C-31-1-50, R12, P3) :
- `GET /api/v1/admin/backup-schedule` → réglage + état + `nextDueAt` (calculé par la même logique que `is_due`) +
  `backupDir` **non exposé** (le chemin reste au journal, comme pour la sauvegarde pré-import) ;
- `PUT /api/v1/admin/backup-schedule` `{ enabled, intervalHours, retentionCount, version }` → 200 ; valeurs hors
  domaine → 400 `VALIDATION_ERROR` ; version périmée → 409 `OPTIMISTIC_LOCK_CONFLICT` (code existant) ; audit
  `admin.backup_schedule_updated` (`before`/`after`) dans la même transaction. **Seul `update_settings` incrémente
  `version`** : `claim_attempt` et `record_*` ne la touchent pas (sinon une sauvegarde terminée entre le chargement du
  panneau et *Enregistrer* produirait un 409 sans cause visible) ; leurs horodatages sont liés **depuis l'application**
  (`Utc::now().naive_utc()`), jamais `NOW(3)` (le fuseau de session MariaDB fausserait `<=>` et `is_due`) ;
- `POST /api/v1/admin/backup-schedule/run` → exécute `run_once(…, Trigger::Manual)` **sans** la condition `is_due`,
  dans une tâche `tokio::spawn` attendue (comme l'import, AC8b : un client parti n'interrompt pas l'écriture), attend la
  fin, rend l'état ; `RunOutcome::Skipped` (verrou pris par une autre sauvegarde, que `run_once` a tenté lui-même, ou
  réservation perdue, AC27) → 409 `BACKUP_IN_PROGRESS` (code neuf : variante `AppError::BackupInProgress` d'`errors.rs`,
  clé `error-backup-in-progress` en quatre locales). Le handler ne touche pas `backup_lock`. Le frontend l'appelle
  avec `LONG_OPERATION_TIMEOUT_MS`, sans rejeu (AC18b) ;
- `GET /api/v1/admin/backups` → liste des fichiers **réguliers** (`symlink_metadata` : un lien n'est jamais listé)
  `kesh-auto-*.keshbackup` de `auto/` et `kesh-pre-import-*.keshbackup` de `KESH_ADMIN_BACKUP_DIR` dont le nom satisfait
  l'expression de son `kind` (`auto` : `^kesh-auto-[0-9]{8}T[0-9]{9}Z\.keshbackup$` ; `pre-import` :
  `^kesh-pre-import-[0-9]{8}T[0-9]{9}-[0-9]+-[0-9]+\.keshbackup$`, forme du nom écrit par `write_pre_import_backup`,
  `routes/admin.rs:537-542`, à revérifier au dev) — nom, `kind`, taille, date de modification, plus récents en tête ;
  aucun autre fichier n'est listé (pas de `kind = other`, C-31-1-35) ; les `.partial` exclus ;
- `GET /api/v1/admin/backups/{kind}/{name}` → téléchargement en flux ; `name` validé par l'expression de son `kind`
  **puis** retrouvé dans la liste du dossier (jamais concaténé tel quel : pas de traversée ; fichier régulier
  seulement) ; 404 sinon ; audit best-effort `admin.backup_downloaded` (nom, taille).
`audit_route_registry.rs`, `audit_label_registry.rs` et les libellés d'audit (quatre locales) suivent ;
`tests/admin_pat_denied_e2e.rs` (`ADMIN_COUPLES`, l. 53, et le compte « 28 constructeurs », l. 46 → **33**) gagne les
cinq couples neufs — dont deux à paramètres de chemin ; la répartition codée en dur de
`read_only_pat_is_stopped_by_the_right_guard_on_each_couple` passe de `(5, 23)` à **`(8, 25)`** (trois `GET` de plus
atteignent la couche, un `PUT` et un `POST` de plus sont arrêtés par le gate de portée ; commentaire « Les 5 `get` »,
l. 785, mis à jour) ; et **le seul `GET` neuf qui porte `ensure_not_pat`**, le téléchargement, entre dans
`GUARDED_BY_HANDLER` de `head_couples_are_denied_too` (l. 846) — son `403` ne discrimine pas la couche ; les deux autres
`GET` neufs (`backup-schedule`, `backups`) n'ont pas de garde propre et **prouvent** la couche : le compte des chemins
discriminants passe de 4 à **6**, à écrire dans le commentaire du test (R5 de la P2, R12 de la P3 : la P2 faisait
porter `ensure_not_pat` aux trois et les sortait tous de la preuve). `docs/api-external.md` : rien à ajouter **pour ces routes**, qui relèvent de
la fermeture générale des routes d'administration aux clés (§ 4) — vérifié, à écrire au Dev Agent Record (le fichier
change par ailleurs à la 31-1a, AC14).

**AC33 — L'écran.** `AdminBackupPanel` gagne une section **« Sauvegarde automatique »** : interrupteur *Activée*,
fréquence (*Toutes les 6 heures* / *12 heures* / *Chaque jour* / *Chaque semaine*), *Nombre de sauvegardes conservées*
(1–60), *Enregistrer* ; état : dernière réussite (date locale, taille lisible, nombre de pièces non incluses s'il y en
a — statut `partial` dit « incomplète », en avertissement, `role="status"`, AC31), dernière sauvegarde complète si
elle diffère (marquée « introuvable sur le serveur » si `last_complete_file` n'est pas dans la liste des sauvegardes de
l'AC32 — fichier supprimé ou déplacé à la main ; L6, P2), dernière
tentative et son statut, prochaine prévue ; bouton **« Sauvegarder maintenant »** (désactivé pendant
l'exécution) ; liste **« Sauvegardes sur le serveur »** (nom, type, taille, date, *Télécharger*). Rappel permanent : les
sauvegardes du serveur sont dans le dossier de sauvegarde de l'installation (`./backup/auto` avec les compose fournis),
sur le **même disque** que Kesh — à copier ailleurs (Hyper Backup) ; elles contiennent des secrets ; chacune pèse à peu
près la base **plus** le dossier des documents (`retention_count + 1` copies dans `auto/`, plus une par import dans le
dossier de sauvegarde, jamais purgée). Les réglages de la
sauvegarde automatique ne voyagent pas dans le `.keshbackup` (ils appartiennent à l'installation, comme le `.env`).
Le statut `insufficient_space` porte la marche à suivre de l'AC30. **Textes** : la règle de l'AC22 (quatre locales,
fallbacks Svelte identiques au français, `lint-i18n-ownership` vert, `i18n-keys.test.ts`) vaut pour **toutes** les
chaînes de la 31-1c — section, statuts, rappels, bandeau, libellés d'audit, `error-backup-in-progress` (R9, P3).

**AC34 — Télécharger une sauvegarde du serveur** remplace, pour la sauvegarde pré-import, le geste SSH du manuel
(l. ~1902-1907, `sudo cp … chown`) : le manuel donne désormais le téléchargement par l'écran comme voie normale, SSH
en secours.

**AC35 — Concurrence avec l'import, vérifiée.** Test d'intégration : pendant un import (verrou tenu par le test au
moyen d'un `backup_lock.clone().lock_owned()` pris à la main), `run_once(&state, env, now, Trigger::Schedule)` rend
`Ok(RunOutcome::Skipped)` sans écrire ni modifier la table — l'appel est enveloppé dans
`tokio::time::timeout(Duration::from_secs(5), …)` et le test exige `is_ok()` (« la tâche doit sauter, pas attendre ») :
sous la mutation M-c6 (`run_once` en `lock_owned().await`) il **échoue** en 5 s au lieu de pendre le gate
(`.config/nextest.toml` n'a pas de `slow-timeout`) ; `POST …/run`, le même verrou tenu, rend 409 `BACKUP_IN_PROGRESS`
(même enveloppe ; sous la mutation M-c17, le handler qui rend l'état en 200 sur `Skipped`, le test rougit). Après la
libération, `run_once` écrit.

**AC36 — Tests d'intégration de la sauvegarde automatique** (`crates/kesh-api/tests/admin_auto_backup_e2e.rs`) : succès
(fichier `0600`, dossier `0700`, archive importable : `parse_and_verify` vert, fichiers inclus) ; rétention (8
sauvegardes déposées + une neuve, `retention_count = 3` → restent les 3 plus récentes, un `kesh-pre-import-x` et un
`autre.keshbackup` intacts) ; espace insuffisant (seuil injecté par paramètre de test : la fonction de mesure est
passée à `run_once` derrière le trait `BackupEnv`, AC30, C-31-1-19) → `insufficient_space`, aucun fichier ;
**première sauvegarde sans historique** (`last_success_bytes` nul, dossier des documents de test portant des fichiers
dont la somme dépasse l'espace libre injecté) → `insufficient_space`, aucun fichier (C-31-1-42, tue M-c19) ; deux
`run_once` successifs à des `now` distincts d'une milliseconde → deux fichiers, deux noms (C-31-1-44) ; écriture
impossible (dossier `auto` remplacé par un fichier) → `failed`, aucune suppression ; **disque saturé pendant
l'écriture** (écrivain de test `StorageFull` après N octets) → `disk_full`, `.partial` supprimé, aucune suppression de
rétention (8 sauvegardes déposées restent 8) ; `classify_write_error` unitaire (`StorageFull`, `QuotaExceeded`, code
brut 28, `PermissionDenied` → `failed`) ; **sauvegarde incomplète** (une pièce référencée retirée du dossier des
documents) → `partial`, `last_complete_file` inchangé, et sur une suite de `retention_count + 2` sauvegardes `partial`
la dernière complète **reste** ; une sauvegarde complète suivante la remplace dans `last_complete_file` et la rétention
retrouve son compte ; archive relue (AC28) : un écrivain de test qui tronque l'archive → `failed`, fichier supprimé ;
`version` inchangée par `claim_attempt`/`record_*` (un `PUT` avec la version lue avant le tour → 200) ; réservation perdue (une autre
connexion avance `last_attempt_at` entre la lecture et l'`UPDATE` — observateur de test) → rien n'est écrit ;
`.partial` résiduel supprimé ; garde lexicale du garde possédé (AC27 : aucune forme empruntée de `backup_lock`,
`try_lock_owned` dans `auto_backup.rs` seulement, `lock_owned` dans la fermeture du `spawn` de `full_import` ; tue
M-c18) ; garde lexicale du lancement (AC26 ; tue M-c21) ; liste et téléchargement : un lien symbolique
`kesh-auto-…keshbackup` vers un autre fichier n'est ni listé ni servi ; routes : 403 non-admin, 403 clé d'API, 400
domaine, 409 version, téléchargement 404 sur `../x`, sur un nom hors expression et sur un nom absent ; **audits et
état** (R12, P3) : un `PUT` écrit `admin.backup_schedule_updated` avec `before` et `after` exacts, et un `PUT` refusé
(400) n'en écrit aucun ; un téléchargement écrit `admin.backup_downloaded` (nom, taille) ; `nextDueAt` de
`GET backup-schedule` égale `last_success_at + interval` après un succès, et l'heure de la tentative + 1 h après un
échec (la logique d'`is_due`).

**AC37 — Vitest.** Panneau : rendu de l'état (ok, `partial`, échec de chaque type, jamais tenté, désactivé),
enregistrement (payload exact, conflit 409 → rechargement proposé), « Sauvegarder maintenant » (désactivé pendant, 409
affiché, appel avec `LONG_OPERATION_TIMEOUT_MS` sans rejeu), liste et téléchargement, `partial` en avertissement
(`role="status"`, aucun `role="alert"`), `last_complete_file` absent de la liste → « introuvable » ; bandeau du layout
(`role="status"`) : affiché pour Admin seulement, dans les deux cas de l'AC31 (échec, aucune réussite depuis deux
intervalles), **absent quand `partial`**, **absent quand la sauvegarde est désactivée** même si `last_status` est un
échec (tue M-c20), absent quand `ok` et récent, absent quand `last_success_at` est nul et `last_status` nul, absent
quand la lecture échoue.

**AC38 — E2E (`frontend/tests/e2e/backup-schedule.spec.ts`).** Admin : changer la fréquence et la rétention,
enregistrer, recharger la page → valeurs gardées ; « Sauvegarder maintenant » → état « réussie » et une ligne `auto` dans
la liste ; télécharger cette ligne → fichier non vide dont le manifeste a `formatVersion: 2` ; réglages remis aux défauts
en fin de spec (la table survit aux `seedTestState`). Comptable : le menu n'offre pas la page, l'URL directe
**redirige vers `/`** (garde de route existante, `routes/(app)/admin/backup/+page.ts:15`, `redirect(302, '/')`) — le
refus 403 de l'API est vérifié côté Rust (AC36).

**AC39 — Manuel et CHANGELOG.** `admin-manual.tex` : sous-section **« La sauvegarde automatique »** (activée par défaut
à la mise à jour, fréquence, rétention et protection de la dernière sauvegarde complète, statut « incomplète »,
emplacement `./backup/auto`, espace requis et refus, volume (base + documents par copie), état et bandeau, *Sauvegarder
maintenant*, téléchargement, horloge déréglée, ce que la relecture d'une archive garantit et ne garantit pas (AC28),
le statut « incomplète » visible au panneau et non au bandeau (C-31-1-40), ce qu'elle ne remplace pas : la copie hors
du disque) ; tableau *Quelle méthode choisir ?*
(l. ~1950-1963) mis à jour ; *Stratégie recommandée* (l. 1461) ; sauvegarde pré-import (téléchargement, AC34) ; mise à
jour vers la 0.15.0 (migration non breaking, retour à la 0.14.0 par la sauvegarde d'avant) ; l'issue d'un
`insufficient_space` (AC30). PDF régénéré. CHANGELOG : section `## [0.15.0] — Non publié` (AC14). README § Fonctionnalités
(ligne *Export/import d'installation*, l. 49).

**AC40 — Captures (thème clair)** de chaque état touché, dans `/home/gcorbaz/devel/kesh-gate-logs/311c-captures/`,
citées au Dev Agent Record et au prompt de revue : section *Sauvegarde automatique* (a) jamais tentée, (b) désactivée,
(c) réussie, (d) `partial`, (e) `failed`, (f) `insufficient_space`, (g) `disk_full` ; (h) « Sauvegarder maintenant » en
cours ; (i) 409 `BACKUP_IN_PROGRESS` affiché ; (j) conflit de version (rechargement proposé) ; (k) erreur de validation ;
(l) liste des sauvegardes remplie ; (m) liste vide ; bandeau du layout (n) échec, (o) *retirée en P2* (`partial`
n'allume plus le bandeau, C-31-1-40 ; lettre conservée pour ne pas décaler les suivantes), (p) « aucune
sauvegarde réussie depuis … » ; (q) la section à largeur étroite (390 px). Les états qu'on ne peut atteindre par
l'écran se posent ainsi, ce que le Dev Agent Record dit capture par capture (R8, P3) : les **statuts** (d), (e), (f),
(g), (n), (p) en base de test par `UPDATE` avant la capture ; (h) et (i), états d'un **verrou en mémoire** qu'aucun
`UPDATE` ne pose et qu'une sauvegarde de quelques Kio ne tient pas assez longtemps, **par interception réseau**
(`page.route` sur `POST …/run` : réponse retardée pour (h), réponse 409 `BACKUP_IN_PROGRESS` pour (i)).

## Tasks / Subtasks

### 31-1a

- [ ] T-a0 **Avant tout changement de code**, sur `44ef2079` : générateur `#[ignore]` et fixture
  `tests/fixtures/keshbackup-0.14.0-format1.keshbackup` (AC10), commit dédié, empreinte au Dev Agent Record.
- [ ] T-a1 Tests rouges : AC2 (deux gardes), AC6/AC7 (unitaires de `parse_and_verify` : format 2 valide, ligne du
  manifeste sans entrée, nom invalide, préfixe ≠ sha, taille dépassée, sous-dossier, format 3 refusé, chemin dans les
  deux listes, chemin de `filesNotIncluded` hors forme **accepté**), AC6b (manifeste trop grand, tables trop grandes **en
  cumul** — deux tables chacune sous la borne, leur somme au-dessus —, `byteSize` supérieur au plafond d'import → 400
  sans panique, entrée inattendue — dont une entrée `files/*` sans ligne — refusée en format 2 avant la lecture de
  toute table ; bornes réduites par `ReadLimits` ; « nom en double » retiré en P2, C-31-1-38), AC10 (fixture réelle),
  AC13 (intégration).
- [ ] T-a2 `kesh-db` : `ReferencedFile`, lecture dans `export_installation_in` (AC1) ; tests de l'instantané étendus
  (une facture figée **après** l'ouverture de l'instantané n'apparaît pas dans `referenced_files` — observateur).
- [ ] T-a3 `manifest.rs` : version 2, `files`, `filesNotIncluded` (`serde(default)`).
- [ ] T-a4 `export.rs` : `write_keshbackup<W: Write + Seek>` synchrone, appelée dans un seul `spawn_blocking` après
  `export_installation` (AC4) ; lecture des fichiers en deux passes, raisons dont `unreadable` (AC3) ; méta étendue
  (AC5) ; `routes/admin.rs` : `SpooledTempFile` (AC4), en-têtes (AC5, AC12), audit.
- [ ] T-a4b Tests de l'export (≥ 9, rouges d'abord) : fichier `missing`, `altered`, `mismatch` (préfixe ≠ empreinte de la
  ligne ; même chemin, deux empreintes) → listé dans `filesNotIncluded`, absent de l'archive (c'est le test qui tue M-a2) ;
  fichier **illisible** (`chmod 000` dans le dossier des documents du test ; le test vérifie d'abord qu'il ne peut pas le
  lire et **échoue bruyamment** s'il le peut — gate en `root` —) → `unreadable`, l'export rend **200** et les autres
  pièces sont incluses (tue M-a20) ;
  deux chemins de même empreinte (deux sociétés, AC1) → tous deux inclus ; `storage_path = "/etc/hostname"` et
  `"../x.pdf"` en base → `mismatch`, aucune lecture hors du dossier ; **aller-retour** : la même base → export →
  import sur une base vidée → **200**, `filesMissing` compte ces lignes (une sauvegarde authentique s'importe ; tue
  M-a18) ; entrée dossier `files/` présente sans aucun fichier ; manifeste 1 relu
  avec deux listes vides ; en-têtes `X-Kesh-Backup-Files*` (valeurs exactes) et champs d'audit (AC5) ;
  `X-Kesh-Backup-Exceeds-Import-Limit` posé / non posé (plafond de test réduit) (AC12) ; débordement du `SpooledTempFile`
  **dans** `KESH_ADMIN_BACKUP_DIR`, observé par son effet (le temporaire est anonyme : aucun `read_dir` ne le voit,
  C-31-1-43) — `Config` de test dont `admin_backup_dir` est un dossier **propre au test, distinct de
  `std::env::temp_dir()`**, existant et rendu non inscriptible (`0500`) : seuil `KESH_ADMIN_EXPORT_INMEM_MB = 0` →
  l'export **échoue** (500), seuil élevé → 200 ; sous la mutation M-a19 (`spooled_tempfile` sans `_in`, débordement
  dans `/tmp`), le premier cas rend 200 et rougit. Le test commence par vérifier qu'il ne peut pas créer de fichier dans
  ce dossier, et **échoue bruyamment** s'il le peut (gate lancé en `root`) au lieu de passer à vide ; dossier de
  sauvegarde absent → créé (AC4).
- [ ] T-a5 `document_storage.rs` : cœur d'écriture atomique factorisé ; `restore_document(dir, storage_path, source:
  impl Read, byte_size, expected_sha)` — copie en flux à travers `take(byte_size.saturating_add(1))` et `Sha256` (AC8 ; la signature
  à `bytes` de la création lisait le fichier entier en mémoire, F4, P2), puis `sync_all` du dossier (AC8) ;
  `open_document` et son test unitaire direct (AC3 ; tue M-a12) ; `write_backup_file` **synchrone**, à fermeture de
  remplissage sur un clone du `File` (AC4) — ses tests (`routes/admin.rs:804`, `:830`) réécrits en synchrone.
- [ ] T-a6 `import.rs` : `parse_and_verify<R: Read + Seek>`, `ReadLimits`, `ParsedBackup` portant la `ZipArchive<R>`,
  format 2 (AC6, AC6b, AC7, AC8) ; `routes/admin.rs` : `read_upload` sur disque
  (AC11), écriture des fichiers avant le `DELETE` (AC8), import dans une tâche `spawn` attendue (AC8b), comptes et
  audit (AC9).
- [ ] T-a6b Tests de l'import (≥ 8, rouges d'abord) : cohérence AC7 en format 2 (fichier de `manifest.files` non
  référencé ; empreinte ≠ `file_hash`/`pdf_sha256` ; fichier référencé ni dans `files` ni dans `filesNotIncluded` →
  400) ; la même absence en format 1 → **200** (fixture réelle) ; table `invoices` sans colonne `pdf_storage_path` →
  ni `panic` ni 400, `filesMissing` calculé sans elle ; refus d'écriture (dossier des documents non inscriptible) →
  500 et base intacte ; fichier présent altéré → remplacé ; entrée `files/` plus grande que son `byteSize`, **dont le
  `sha256` déclaré est celui du contenu entier** (seule la borne la refuse ; sans borne, l'empreinte concorderait :
  c'est ce qui tue M-a6 ; R12, P3) → 400 « trop grande » ; dossier de sauvegarde absent → créé, import 200 ; dossier de
  sauvegarde existant **non inscriptible** (`0500`, même garde « échoue bruyamment en `root` ») → 500
  `AdminPreImportBackupFailed`, base intacte (R12, P3) ; garde lexicale de l'AC8b.
  (« Aucun temporaire laissé » retiré en P2 : vrai par construction, aucun test ne pouvait rougir — AC11.)
- [ ] T-a7 `config.rs` : défaut 2048 (AC12) et son test ; `.env.example`.
- [ ] T-a8 Manuels, `docs/api-external.md`, proxy inverse, CHANGELOG (AC14) ; grep des valeurs (`503`, `512`,
  `files/`, `n'emporte`, `sans les fichiers`, `aucun binaire`, `formatVersion`, `50M`, `300s`) sur tout le dépôt — code,
  doc-comments, i18n, manuels (`.tex` **et** PDF aplati), `docs/`, `website/` — confronté à l'inventaire clos de l'AC14 :
  chaque site de l'inventaire traité, chaque site **neuf** ajouté au tableau du Dev Agent Record avec son traitement
  (les phrases de `website/roadmap.html` et de la brochure sont laissées à la release, à noter au Dev Agent Record) ;
  **et** la commande des sujets de l'AC14 rejouée, chacune de ses lignes triée (94 à `c7639e80`).
- [ ] T-a9 Mutations (§ Mutations), gate complet (`kesh-db` touché), E2E complet au dernier commit de code.

### 31-1b

- [ ] T-b1 Vitest rouges : lien de l'assistant (Admin seulement, étapes 0–2), avis `.env`, avis de l'export (trois cas
  d'en-têtes), avis de fin d'import (`filesMissing` 0 / > 0 ; bouton *Se reconnecter* dans **les deux** cas, aucune
  redirection avant le clic), garde de rôle de `/onboarding/restore`.
- [ ] T-b2 `routes/onboarding/restore/+page.svelte` et `+page.ts` (garde de rôle), lien sur `+page.svelte` (AC15).
- [ ] T-b3 `admin-backup.api.ts` (en-têtes), `AdminBackupPanel`, `AdminRestorePanel` (AC17-AC19).
- [ ] T-b4 i18n ×4 (AC22).
- [ ] T-b5 E2E AC20 ; captures (AC21).
- [ ] T-b6 Manuel (AC21), PDF.
- [ ] T-b7 `api-client.ts` : options `{ timeoutMs, retry }` de `getBlob` et `postFormData`,
  `LONG_OPERATION_TIMEOUT_MS` ; `downloadFullExport`, `uploadFullImport` ; message d'issue inconnue de l'import pour
  `TIMEOUT`, `NETWORK_ERROR`, 502, 504 (i18n ×4) ; tests Vitest rouges d'abord (AC18b).

### 31-1c

- [ ] T-c1 Migration, réinjection de la ligne singleton par `scripts/regen-test-schema.sh` et garde
  `squash_seeds_the_backup_schedule_row`, squash régénéré, audit P5 (trois sites), P6, P7, inventaire (AC23, AC24).
- [ ] T-c2 `kesh-db` : dépôt `backup_schedule` (`get`, `update_settings` avec version, `claim_attempt`, `record_*`).
- [ ] T-c3 `kesh-api/src/auto_backup.rs` : `is_due`, `run_once` (relecture de l'archive, statut `partial`,
  `last_complete_file`, noms à la milliseconde, seuil `documents_bytes`, verrou pris en `try_lock_owned` et
  `RunOutcome::Skipped`), `spawn`, `BackupEnv` (`Arc<dyn BackupEnv>`, `wrap_writer` qui prend un clone du `File` par
  valeur), `classify_write_error` (AC25-AC30) ; `AppState.backup_lock` aux neuf sites, gardes possédés et leur garde
  lexicale (AC27) ; branchement dans `main.rs` et sa garde lexicale (AC26).
- [ ] T-c4 Routes et registres (AC32 : `audit_route_registry`, `audit_label_registry`, `admin_pat_denied_e2e`
  — `ADMIN_COUPLES`, 33 constructeurs, `(8, 25)`, le seul `GET` de téléchargement dans `GUARDED_BY_HANDLER`, 6 chemins
  discriminants) ; `AppError::BackupInProgress` ; libellés d'audit ×4.
- [ ] T-c5 Frontend : panneau (AC33), bandeau (AC31), API ; Vitest (AC37) ; i18n ×4 de toutes les chaînes de la 31-1c
  (règle de l'AC22, AC33).
- [ ] T-c6 Tests d'intégration (AC35, AC36), E2E (AC38), captures (AC40).
- [ ] T-c7 Manuel, CHANGELOG, README (AC34, AC39).

## Dev Notes

### Tests (récapitulatif, rouges d'abord)

| Sous-story | Rust | Vitest | E2E |
|---|---|---|---|
| 31-1a | gardes AC2 (2), `parse_and_verify` format 2 et lectures bornées (≥ 12, T-a1), instantané (1), `open_document` direct (1), export (≥ 9, T-a4b), import et cohérence (≥ 8, T-a6b, dont la garde lexicale de l'AC8b), intégration AC13 (2), AC10 sur la fixture réelle (1) | — | aucune spec neuve (l'écran ne change pas) ; suite complète au dernier commit de code |
| 31-1b | — | ≥ 16 (dont ≥ 5 pour `api-client` et ≥ 4 pour le message d'issue inconnue — `TIMEOUT`, `NETWORK_ERROR`, 502, 504 —, AC18b ; garde de rôle de `/onboarding/restore`) | `backup-restore-new-install.spec.ts` |
| 31-1c | migration et squash (AC23, AC24), `is_due` (≥ 6), `classify_write_error` (≥ 4), garde lexicale des gardes possédés (AC27), garde lexicale du lancement par `main.rs` (AC26), intégration AC35/AC36 | ≥ 14 | `backup-schedule.spec.ts` |

Le délai du client (AC18b) n'est **pas** prouvé par un E2E : les fixtures E2E pèsent quelques Ko et s'exportent en
moins d'une seconde, si bien qu'un E2E vert ne dirait rien du délai. Il l'est par les tests Vitest de `api-client` à
horloge simulée, qui figent les valeurs (30 s par défaut, une heure pour les appels longs, aucun rejeu).

### Gardes existantes qui changent de sens ou rougiront

- `import.rs` tests `parse_rejects_future_format_version` (forge `Some(2)` → désormais **valide** : passer à 3) et
  `parse_rejects_non_empty_files_dir` (vaut encore pour le format 1 : forcer `formatVersion: 1`).
- `manifest.rs` `manifest_json_has_canonical_shape` (`formatVersion` 1 → 2, `files` présent).
- `export.rs` `build_keshbackup_ne_lit_la_base_qu_une_fois` (corps renommé ; garder la garde exacte).
- `routes/admin.rs` garde `avant_sauvegarde_branchee_aux_appels_de_l_import` (l. 881 ; motifs `check_schema_compat(` et
  `build_keshbackup(`, compte `(1, 2)`, l. 911-915) — **rougit** quand l'appel pré-import change de forme (lecture de la
  base, puis `write_keshbackup` en `spawn_blocking`), puis **deviendrait muette** si l'on n'ajustait que le compte : sa
  liste d'identifiants s'étend aux nouveaux appelants (R6, P3).
- `routes/admin.rs` garde `echecs_avant_sauvegarde_tous_convertis` (l. 957 ; segment du début de
  `run_backup_and_restore` à `write_pre_import_backup(`, compte `(directes, par_avant_sauvegarde) == (3, 1)`, l. 975-984)
  — **rougit** si la conversion `.map_err(avant_sauvegarde)` quitte ce segment pour `write_pre_import_backup` ; le
  segment ou le compte se réécrit en gardant l'invariant (tout `?` avant la sauvegarde converti), jamais en baissant le
  compte seul (R6, P3).
- `admin_full_export_e2e.rs:477` et `:497` (`admin_export_inmem_mib = 0`, chemin de diffusion), `:254`, `:274`
  (structure du ZIP, `files/`), **`:296`** (`assert_eq!(manifest["formatVersion"], 1)` → 2) ; `admin_backup_e2e.rs`
  (structure du ZIP : `files/` vide → vide **seulement** sans fichier référencé).
- **`admin_full_import_e2e.rs:489-493`** (forge `formatVersion = 2` et attend 400 : devient valide → forger 3) ; son
  `rezip` (l. 263-281) recopie le manifeste de l'export, donc un **format 2** après le bump (constat corrigé en P2 : la
  fiche le disait format 1) — et il ne recopie que `data/`, si bien qu'un test qui seede un document puis passe par
  `rezip` perdrait la bijection ; aucun ne le fait aujourd'hui, à garder en tête.
- `import.rs` `build_test_backup` (l. 308) : `None` produit désormais un format 2 sans fichier (AC10).
- `config.rs` : tests du défaut 512, littéraux `admin_import_max_mib: 512` des `Config` de test (l. 513, 1626), doc
  (l. 249) et messages (l. 927-948) ; `.env.example:186` ; `configuration_transmise.rs` (variables inchangées, vérifier).
- `routes/admin.rs` : la garde lexicale neuve de l'AC8b ; les tests de `write_backup_file`
  (`write_backup_file_n_ecrase_pas`, l. 804, `write_backup_file_passe_par_un_partiel`, l. 830 — `async` sur `tokio::fs`
  aujourd'hui) réécrits pour la primitive **synchrone** à fermeture de remplissage, erreur typée (AC4) : **rougissent**
  à la compilation, pas de risque de mutisme.
- `tests/admin_pat_denied_e2e.rs` — 31-1c, cinq couples neufs : `ADMIN_COUPLES` et « 28 constructeurs » (→ 33) ;
  répartition `(5, 23)` → `(8, 25)` et commentaire « Les 5 `get` » (l. 785), qui **rougit** ; `GUARDED_BY_HANDLER`
  (l. 846) gagne le seul `GET` de téléchargement, qui porte `ensure_not_pat` — **silencieux** sinon : le seuil « ≥ 4
  discriminants » resterait vert avec une preuve gonflée d'un chemin ; les deux autres `GET` neufs comptent, eux, parmi
  les discriminants (4 → 6 ; AC32, R5 de la P2, R12 de la P3).
- `scripts/regen-test-schema.sh` et `tests/test_schema_guard.rs` (ligne singleton, 31-1c).
- `backup.rs:1763` `tables_sauvegardees_innodb_et_toutes_exportees` : relu (itère sur les tables attendues, non
  affecté par `referenced_files` ; à étendre seulement si l'implémentation y ajoute la lecture).
- Les **neuf** sites `AppState { … }` / `Self { … }` (AC27, 31-1c).
- `api-client.test.ts` (ou équivalent) — 31-1b. `AdminRestorePanel.test.ts` n'a **aucun** test sur `TIMEOUT` (`grep -n
  TIMEOUT` vide ; la P1 citait une garde qui n'existe pas) : les tests de l'issue inconnue sont neufs (AC18b) ; ceux
  qui attendent une redirection immédiate après un import réussi changent (AC19, *Se reconnecter* toujours).
- `backup.rs` `backup_inventory_matches_schema` et le test de l'instantané qui énumère les tables lues (`TABLES_TO_TRUNCATE
  + _kesh_version`, l. ~1773) ; `post_restore.rs` `every_data_backfill_migration_is_triaged` ; `test_schema_guard.rs`.
- `audit_route_registry.rs`, `audit_label_registry.rs` (routes et actions neuves de la 31-1c).
- `lint-i18n-ownership`, `i18n-keys.test.ts` ; `onboarding-layout.test.ts` (route neuve) ; `boutons-rouges-inventory.test.ts`
  si un bouton destructif apparaît (aucun prévu).

### Mutations (motif unique vérifié, restauration `git checkout -- f && touch f`, sous le verrou de gate)

| # | Mutation | Test qui doit la tuer |
|---|---|---|
| M-a1 | AC1 : retirer la branche `invoices` de l'union | intégration AC13 (PDF absent → 410) |
| M-a2 | AC3 : inclure un fichier altéré au lieu de le lister | T-a4b « altéré non inclus » |
| M-a3 | AC6 : supprimer le contrôle « préfixe = sha » | `parse_and_verify` nom ≠ empreinte |
| M-a4 | AC6 : accepter une ligne de `manifest.files` sans entrée `files/` (reformulée en P3 : « entrée sans ligne » était refusée aussi par l'AC6b (2), mutation équivalente) | `parse_and_verify` ligne du manifeste sans entrée |
| M-a5 | AC7 : comparer à `meta.sha256` d'un autre fichier / sauter la vérification | `parse_and_verify` empreinte fausse |
| M-a6 | AC7 : supprimer la borne `byteSize + 1` | entrée plus grande que déclarée **dont le `sha256` déclaré est celui du contenu entier** (T-a6b) |
| M-a7 | AC8 : écrire les fichiers **après le `COMMIT`** (reformulée en P1 : « après le `DELETE` » restait dans la transaction, donc annulée par l'échec — mutation équivalente) | refus d'écriture : la base doit rester intacte (sous la mutation, elle est déjà remplacée) |
| M-a8 | AC8 : ignorer l'erreur d'écriture | idem (200 au lieu de 500) |
| M-a9 | AC9 : `filesMissing` calculé sur le backup au lieu de la destination | AC13 format 1 (`filesMissing = 2`) |
| M-a10 | AC6 : accepter `formatVersion` 3 | format 3 refusé |
| M-a11 | AC1 : lire les fichiers référencés **hors** instantané (sur le pool) | garde lexicale + test de l'observateur |
| M-a12 | AC3 : `open_document` sans `is_safe_storage_path` (`documents_dir.join(storage_path)` direct) | test unitaire direct d'`open_document` (`"../x"`, `"/etc/hostname"`, `"a/b"`, `""` → `InvalidPath`) — reciblé en P3 : par l'export, l'AC1 classe `mismatch` avant tout appel, et T-a4b restait vert |
| M-a13 | AC1 : classer `mismatch` deux chemins de même empreinte | T-a4b « deux chemins, une empreinte → deux inclus » |
| M-a14 | AC6b : borne des tables `data/` appliquée par table au lieu d'en cumul | T-a1 « tables trop grandes en cumul » (bornes réduites par `ReadLimits`) |
| M-a15 | AC7 : appliquer la cohérence fichier ↔ ligne au format 1 | AC10, fixture réelle (200 attendu) |
| M-a16 | AC8b : appeler `run_backup_and_restore` hors du `tokio::spawn` | garde lexicale de l'AC8b |
| M-a17 | AC4/AC11 : ne pas créer le dossier de sauvegarde | T-a6b « dossier absent → 200 » |
| M-a18 | AC6 : réappliquer l'expression des noms aux chemins de `filesNotIncluded` | T-a4b aller-retour (ligne `"../x.pdf"` → export → import 200) |
| M-a19 | AC4 : `spooled_tempfile` sans `_in` (débordement dans `/tmp`) | T-a4b dossier de sauvegarde non inscriptible, seuil 0 → 500 |
| M-a20 | AC3 : erreur d'I/O autre que `NotFound` propagée par `?` (toute la sauvegarde échoue) au lieu de `unreadable` | T-a4b fichier illisible → 200, `unreadable` |
| M-b1 | AC15 : lien visible pour un non-Admin | Vitest du lien |
| M-b2 | AC19 : déconnexion immédiate après un import réussi (reformulée en P2 : *Se reconnecter* vaut pour tout succès) | Vitest *Se reconnecter* (`filesMissing = 0` et `> 0`) |
| M-b3 | AC18 : ignorer l'en-tête `Not-Included` | Vitest avis export |
| M-b4 | AC18b : `downloadFullExport` sans l'option longue | Vitest (espion : option passée ; abandon à 30 s) |
| M-b5 | AC18b : `retry: false` ignoré par `fetchWithRetry` | Vitest « un seul `fetch` sur `TIMEOUT` » |
| M-b6 | AC18b : `TIMEOUT` de l'import → message par défaut (« Le serveur ne répond pas. Réessayez ultérieurement. », le comportement actuel ; reformulée en P2) | Vitest du panneau |
| M-b7 | AC18b : 504 de l'import → message par défaut (« Erreur serveur (504) ») | Vitest du panneau, 504 |
| M-c1 | AC25 : `≥` → `>` sur l'intervalle | `is_due` à l'égalité |
| M-c2 | AC25 : retirer la clause `last_attempt_at` | `is_due` « échec il y a 10 min » |
| M-c3 | AC29 : rétention sur `kesh-*` au lieu du motif exact | rétention : `kesh-pre-import-x` intact |
| M-c4 | AC29 : rétention aussi après échec | écriture impossible → aucune suppression |
| M-c5 | AC30 : seuil d'espace ignoré | `insufficient_space` |
| M-c6 | AC27 : `run_once` en `lock_owned().await` au lieu de `try_lock_owned` | AC35 (`run_once` sous `tokio::time::timeout` de 5 s : échoue au lieu de pendre) |
| M-c7 | AC27 : réservation sans `<=> ?` | réservation perdue |
| M-c8 | AC32 : téléchargement par concaténation du nom | `../x` → 404 |
| M-c9 | AC31 : bandeau sans la condition « deux intervalles » | Vitest bandeau « aucune réussite depuis » |
| M-c10 | AC23/AC24 : table ajoutée à `TABLES_TO_TRUNCATE` | AC24 (export ne contient pas la table ; un backup 0.14 s'importe) |
| M-c11 | AC28 : `partial` enregistré comme `ok` | AC36 « sauvegarde incomplète → `partial` » ; Vitest du panneau `partial` (avertissement) |
| M-c12 | AC29 : rétention qui ne protège pas `last_complete_file` | AC36 « suite de `partial` : la dernière complète reste » |
| M-c13 | AC30 : `StorageFull` classé `failed` | `classify_write_error` unitaire ; AC36 disque saturé → `disk_full` |
| M-c14 | AC30 : rétention lancée après `disk_full` | AC36 disque saturé : 8 sauvegardes restent 8 |
| M-c15 | AC32 : `record_*` incrémente `version` | AC36 « `PUT` avec la version lue avant le tour → 200 » |
| M-c16 | AC23 : réinjection de la ligne singleton retirée du script (squash sans ligne) | `squash_seeds_the_backup_schedule_row` |
| M-c17 | AC27/AC32 : le handler de `POST …/run` rend l'état en 200 sur `RunOutcome::Skipped` au lieu de 409 (reformulée en P3 : le handler ne prend plus le verrou) | AC35 (409 attendu, sous `timeout(5 s)`) |
| M-c18 | AC27 : garde emprunté (`backup_lock.lock().await`) tenu par le handler de l'import, hors de la tâche | garde lexicale des gardes possédés (AC27, AC36) |
| M-c19 | AC30 : seuil sans `documents_bytes` | AC36 « première sauvegarde sans historique → `insufficient_space` » |
| M-c20 | AC31 : bandeau du layout sans la condition « activée » | Vitest « désactivée + échec → aucun bandeau » |
| M-c21 | AC26 : retirer `auto_backup::spawn(state.clone())` de `main.rs` | garde lexicale du lancement (une occurrence dans `main.rs`, aucune dans `lib.rs`) |

Une mutation équivalente se **déclare** (pourquoi aucun test ne peut la tuer), elle ne se tait pas. **Déclarée
équivalente au niveau des tests** (P3) : retirer le re-hachage de la passe de copie (AC3) — la divergence entre les
deux passes exige qu'un fichier nommé par son contenu change pendant une lecture ; la provoquer demanderait un
observateur glissé entre les deux passes, pour un cas que l'immuabilité des fichiers rend théorique (C-31-1-47). Le
contrôle reste exigé du code et relu en revue.

### Angles morts assumés (à écrire aux doc-comments et au manuel)

- Les fichiers du dossier des documents que la base ne référence plus (PDF orphelins de deux rendus concurrents,
  `issued_invoice_pdf.rs:21-23`) ne sont ni sauvegardés ni restaurés.
- Archive de plus de 4 Gio : non mesurée (`zip` 2.4.2 écrit le ZIP64 par entrée sur option `large_file` ; les entrées
  sont petites, le volume total ne l'est peut-être pas) — test non fait, écrit au manuel comme limite (C-31-1-9).
- La sauvegarde automatique ne tourne que si Kesh tourne ; une tâche arrêtée sans échec se voit par le bandeau « aucune
  réussite depuis deux intervalles », pas autrement.
- Deux processus sur une même base : la réservation conditionnelle évite deux sauvegardes automatiques simultanées ;
  le verrou en mémoire ne protège pas un import d'un processus contre la sauvegarde de l'autre (l'instantané reste
  cohérent).
- L'espace libre est mesuré **avant** ; un autre programme peut remplir le disque pendant l'écriture (cas `disk_full`).
- Pas d'arrêt gracieux : un arrêt du conteneur pendant une sauvegarde laisse un `.partial`, nettoyé au tour suivant.
- Horloge du serveur déréglée vers le futur : un nom `kesh-auto-…` daté du futur passe pour le plus récent et survit
  aux suivants (la rétention trie par nom) ; inoffensif pour les données, coûteux en disque — écrit au manuel.
- Le téléchargement de l'export par le navigateur reçoit l'archive entière dans un `Blob` (mémoire ou disque selon le
  navigateur) avant de l'enregistrer : pour plusieurs Gio, c'est la limite du navigateur, non mesurée. Le passage à un
  téléchargement direct (lien) perdrait la lecture des en-têtes de l'AC18 et le message d'erreur (C-31-1-25).
- Délai côté client d'une heure (AC18b) : une opération plus longue est abandonnée par l'écran, pas par le serveur
  (AC8b) ; le message d'issue inconnue le dit.
- Une sauvegarde automatique **jamais tentée** (tâche morte dès le démarrage) n'allume pas le bandeau ; elle se lit dans
  le panneau (« jamais tentée ») (C-31-1-34).
- Les sauvegardes pré-import emportent désormais les pièces et ne sont jamais purgées : le volume se surveille à la main
  (manuel).
- *(P2)* L'ouverture d'une archive (`ZipArchive::new`) décode tout le répertoire central avant que Kesh puisse compter
  les entrées : sa mémoire n'est bornée que par la taille reçue (plafond d'import). L'import est réservé à
  l'Administrateur et refusé aux clés d'API ; lire soi-même l'EOCD (et l'EOCD64) pour borner le compte avant l'ouverture
  n'est pas fait (C-31-1-38).
- *(P2, borne cumulée depuis la P3)* Des tables dont le NDJSON dépasse 1 Gio au total (`TABLES_MAX_BYTES`) s'exportent,
  mais leur import est refusé ; aucun avertissement à l'export (sans rapport avec les volumes réels, où la base pèse
  quelques dizaines de Mio).
- *(P2)* La relecture d'une sauvegarde automatique (AC28) ne vérifie pas les CRC des entrées : une archive abîmée après
  l'annuaire central peut être tenue pour bonne, et la rétention supprimer une plus ancienne saine.
- *(P2)* Une sauvegarde automatique `partial` n'allume pas le bandeau de l'application (C-31-1-40) : une pièce perdue
  après coup ne se voit qu'au panneau, au journal et, à l'usage, par « document indisponible ».
- *(P3)* La borne des tables est désormais **cumulée** (1 Gio pour toutes, AC6b) ; l'entrée qui la franchit est
  « trop grande » même si chaque table, seule, passerait. La mémoire de l'import reste « 1 Gio de NDJSON + leur
  conversion en valeurs » (un multiple, non mesuré), plus le répertoire central (ci-dessus).
- *(P3)* Une sauvegarde **du serveur** (automatique ou pré-import) plus grosse que `KESH_ADMIN_IMPORT_MAX_MB` n'est pas
  signalée dans la liste (AC33) : l'avertissement de l'AC12 ne protège que l'export manuel (F12, C-31-1-51). Le manuel
  dit de relever le plafond quand le dossier des documents grossit.
- *(P3)* Le `JoinError` de la tâche d'import (AC8b), la cadence de la boucle de fond et l'isolation d'une panique dans
  un tour (AC26) ne sont pas testés.
- *(P3)* Un client parti pendant l'assemblage d'un export manuel relâche le verrou, l'assemblage continue jusqu'au bout
  et peut se superposer à une sauvegarde (AC27) — coût disque et processeur, pas de risque pour les données.

### Pièges connus du dépôt, applicables ici

- **Gate complet même en boucle** : `kesh-db` est touché (31-1a : `backup.rs` ; 31-1c : migration, dépôt).
- **P8** : la migration de la 31-1c, une fois appliquée sur une base persistante, ne se modifie plus — pas même un
  commentaire. Base E2E à reconstruire si on la retouche avant le merge.
- **Base de gate remise à zéro** avant tout gate complet (`CLAUDE.md`).
- **Montage E2E à la lettre** (`docs/testing.md`), `KESH_ADMIN_BACKUP_DIR` **et** `KESH_DOCUMENTS_DIR` inscriptibles et
  identiques côté runner (les deux specs neuves en dépendent ; garde bruyante de `helpers/inbox.ts`).
- **Tâche de fond en E2E** : la sauvegarde automatique est activée par défaut ; sur le backend E2E elle écrira, 5
  minutes après le démarrage, une sauvegarde dans `/tmp/kesh-e2e/backup/auto` — attendu, inoffensif ; aucun spec ne doit
  en dépendre ni s'en gêner (le verrou rend `POST …/run` 409 si elle tourne au même instant : le spec de l'AC38 réessaie
  une fois). Le bandeau du layout est `role="status"` (AC31) : il ne peut pas faire rougir les specs qui exigent zéro
  `alert` (`reports.spec.ts:242`).
- **Aucune donnée réelle** : fixtures synthétiques (`spc_e2e_invoice.png`, PDF rendus par Kesh sur la base de
  démonstration). `~/kesh-test/` ne s'ouvre pas.
- Édition du code par l'outil Edit seulement ; ni `sed -i`, ni `python3`.

### Project Structure Notes

- Neufs : `crates/kesh-api/src/auto_backup.rs`, `crates/kesh-db/src/repositories/backup_schedule.rs`,
  `crates/kesh-db/migrations/20261024000001_backup_schedule.sql`, `crates/kesh-api/tests/admin_auto_backup_e2e.rs`,
  `frontend/src/routes/onboarding/restore/+page.svelte`, `frontend/src/lib/features/admin-backup/BackupSchedule*.svelte`
  (si le panneau devient trop long), deux specs E2E, `crates/kesh-api/tests/fixtures/keshbackup-0.14.0-format1.keshbackup`
  (31-1a, T-a0).
- Le nom de la migration suit la dernière (`20261015000001`) ; s'il en arrive une autre d'ici le dev (31-2, 31-3 n'en
  prévoient pas), renuméroter **avant** toute application sur une base persistante.

### References

- `crates/kesh-api/src/admin_backup/{export,import,manifest}.rs`, `crates/kesh-api/src/routes/admin.rs`,
  `crates/kesh-api/src/document_storage.rs`, `crates/kesh-db/src/backup.rs`, `crates/kesh-db/src/snapshot.rs`,
  `crates/kesh-db/src/post_restore.rs`, `crates/kesh-api/src/config.rs`, `crates/kesh-api/src/main.rs`,
  `crates/kesh-api/src/lib.rs` (`AppState`, `admin_routes`).
- Stories 17-3a/b/c (format, export, import), 15-13b (sauvegarde pré-import), 25-6-b (PDF figé), 28-3 (instantané),
  30-2 (documents fournisseur).
- `docs/manual/fr/admin-manual.tex` § *Sauvegarde et restauration* (l. 1450-1965).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

### 2026-10-23 — Création de la fiche (Opus 5.5, autonomie)

Fiche créée sur `1848012d`. Prémisses de #503 établies au code (P-1 à P-14) : deux corrigées — l'export n'est pas en
flux (P-6), et la restauration « base vierge, `/setup`, import » est impraticable sans passer l'assistant (P-12) ; une
nuancée — le 410 ne frappe que la restauration sur un autre dossier de documents (P-3). Découpage en trois clés
(C-31-1-1). Migration non breaking dans la 31-1c seulement (C-31-1-15). Choix C-31-1-1 à C-31-1-24 au registre
`epic-31-choix-autonomes-311.md`.

### 2026-10-23 — Validation P1 (Sonnet 5.5, lentilles R et F) et remédiation (Opus 5.5)

Rapports : `kesh-gate-logs/311-validate-p1-R.md` (**0 CRITICAL / 0 HIGH / 10 MEDIUM / 8 LOW**) et
`311-validate-p1-F.md` (**0 / 2 / 9 / 8**). Chaque finding vérifié au code (`grep -nF`, `sed -n`) avant correction ;
**aucun réfuté**. Recoupements entre lentilles, traités une fois : R1-M1 = F7 (squash), R6-M6 = F6 (`disk_full`),
R8-M8 ⊃ F12/F13 (gardes, sites `AppState`), R3-M3 ≈ F19 (inventaire des phrases), R15-L4 = F3 (règle des empreintes),
R16-L5 ⊂ F4 (`is_safe_storage_path` privée), R17-L6 et R18-L7 ⊂ F5/AC6b, R14-L3 = F16 (`version`). Nuances : F1 —
le délai de 30 s court jusqu'aux **en-têtes** seulement (`fetchWithTimeout`, `clearTimeout` au retour de `fetch`), ce
qui suffit à le rendre bloquant puisque l'export n'envoie ses en-têtes qu'après l'assemblage ; F10 — un backup sans
`pdf_storage_path` est importable aujourd'hui (une colonne nullable absente passe la règle (c2) de
`check_schema_compat`, `import.rs:200`), d'où l'exigence « colonne absente = aucune référence » ; R8 et F12 donnent le même compte
(8 littéraux + `new_for_tests` = 9 sites).

Corrections :

- **F1 (HIGH)** : AC18b neuf (31-1b) — options `{ timeoutMs, retry }` du client, une heure sans rejeu pour l'export,
  l'import et « Sauvegarder maintenant », message d'issue inconnue, tests Vitest à horloge simulée ; AC8b neuf (31-1a)
  — import exécuté dans un `tokio::spawn` attendu, garde lexicale ; AC32 (même chose pour `POST …/run`) ; proxy inverse
  du manuel (AC14, AC21) ; C-31-1-25, C-31-1-26. Mutations M-a16, M-b4, M-b5, M-b6.
- **F2 (HIGH)** : statut `partial`, colonne `last_complete_file`, rétention qui ne la supprime jamais, bandeau (AC23,
  AC28, AC29, AC31, AC33, AC36, AC37) ; C-31-1-27 ; M-c11, M-c12.
- F3/R15 : AC1 (anomalie par ligne, C-31-1-28 ; M-a13). F4/R16 : AC1, AC3 (`open_document`, `is_safe_storage_path`
  `pub(crate)`), AC6 (chemins de `filesNotIncluded`, doublons entre listes) ; M-a12. F5/R18 : AC4, AC11
  (`ensure_backup_dir`) ; M-a17. F6/R6 : AC4 (erreur typée), AC30 (`classify_write_error`, `BackupEnv`), AC36 ;
  C-31-1-32 ; M-c13, M-c14. F7/R1 : AC23, AC24, T-c1 ; M-c16 ; C-31-1-36 (module). F8 : AC11, AC27 ; C-31-1-30. F9 :
  AC6b neuf ; C-31-1-29 ; M-a14. F10 : AC7 (format 2 seulement, colonne absente) ; M-a15. F11 : AC14, AC21. R2 : AC38
  (redirection vers `/`). R3/F19 : AC14, inventaire clos en tableau. R4 : M-a7 reformulée (« après le `COMMIT` »). R5 :
  AC35 (`tokio::time::timeout`). R7 : AC10, T-a0 (fixture produite à `44ef2079`) ; C-31-1-31. R8/F12/F13 : AC27 (neuf
  sites nommés), AC32 (`admin_pat_denied_e2e`), § Gardes. R9 : T-a4b, T-a6b, tableau des tests. R10 : AC21, AC40
  (états énumérés, `311b-captures/`).
- LOW traités : R11 (`export.rs:66`), R12 (trois sites de l'audit P5, AC23), R13 (cadence et échec de lecture du bandeau,
  AC31), R14/F16 (AC32 ; M-c15), R17 (`spawn_blocking`, AC7, AC8), R19 (`uniqueCopy`, AC20), F14 (relecture de l'archive
  avant la rétention, AC28 ; horloge déréglée en angle mort), F15 (C-31-1-35), F17 (C-31-1-34), F18 (C-31-1-33 ; volume
  au manuel et à l'écran).

Découpage : la remédiation n'ajoute de module à aucune sous-story (31-1a : 3, 31-1b : 4, 31-1c : 5, en comptant
`scripts/regen-test-schema.sh` avec `kesh-db`, C-31-1-36 — signalé pour arbitrage). Pas de redécoupage.

Recomptes (depuis la fiche, après remédiation) : **43 AC** (AC1-AC40 + AC6b, AC8b, AC18b) ; **26 tâches** (T-a0 à
T-a9 avec T-a4b, T-a6b : 12 ; T-b1 à T-b7 : 7 ; T-c1 à T-c7 : 7) ; **39 mutations** (M-a1 à M-a17, M-b1 à M-b6, M-c1 à
M-c16) ; **36 choix** au registre (`grep -c '^### C-31-1-'`). Valeurs corrigées grepées sur la fiche et le registre
(`six littéraux`, `403 existant`, `NamedTempFile`, `export.rs:72`, `FreeSpace`, `kind = other`, `C-31-1-24`) : aucun
résidu hors des mentions historiques voulues (C-31-1-19 au registre, la note « l. 72 » de P-6).

### 2026-10-23 — Validation P2 (Opus 5.5, lentilles R et F) et remédiation (Opus 5.5)

Rapports : `kesh-gate-logs/311-validate-p2-R.md` (**0 CRITICAL / 0 HIGH / 8 MEDIUM / 8 LOW**) et
`311-validate-p2-F.md` (**0 / 1 / 7 / 9**). Chaque finding vérifié avant correction : au dépôt (`grep -nF`, `sed -n`)
et, pour `zip` 2.4.2 et `tempfile` 3.27.0, dans les sources du registre cargo — fusion des doublons
(`read.rs:72-75`), `len()` sur la table dédoublonnée (`:949-950`), refus de l'écrivain (`write.rs:1050`), décodage du
répertoire central dans `ZipArchive::new` (`read.rs:706-710`), débordement anonyme (`spooled.rs:126-129`,
`file/mod.rs:83`, `create_unlinked`/`O_TMPFILE` de `file/imp/unix.rs`). **Réfuté en partie** : R6 cite
`admin-manual.tex:2199` parmi les phrases sur `./backup` — la ligne parle des montages `./documents`, `./inbox`,
`./log` (`sed -n 2199p | grep -o ".\{150\}backup"` : vide) ; les vrais sites, relevés par `grep -n "pré-import\|avant
chaque import"`, sont 784, 843, 1466, 1608, 1947 et 2268, tous portés au tableau. **Nuancé** : F4 propose de borner
`MAX_ENTRIES` avant l'ouverture — faisable seulement en lisant soi-même l'EOCD ; écarté (C-31-1-38), la borne est
retirée et l'angle mort écrit. Recoupements traités une fois : R1 = F4 (doublons), R2 = F1, R3 = F5, R4 = F2, R8 ⊃ F15,
L1 = F8 (prémisse), L3 = F9, R6 ⊃ F16.

**Origine.** Nés de la remédiation P1 : F1/R2 (règle des chemins de `filesNotIncluded`), R1/F4 (bornes de l'AC6b),
R3/F5 (tests du débordement et du temporaire), F3 (verrou et tâche détachée), F7 et R8 (b) (`partial` au bandeau,
double rôle), F8/L1 (prémisse de l'AC18b), L6, L8, F10, F11, F12, F13, F17. D'origine (fiche de création), non vus en
P1 : R4/F2 (rendu porteur en P1 par l'AC35 et l'AC40), R5, R6, R7, R8 (a)/F15, F6, L2, L3/F9, L4, L5, L7, F14, F16.

**Retraits** (le défaut venait d'une exigence ; retirée plutôt que compensée) :

- règle « les chemins de `filesNotIncluded` satisfont l'expression » (AC6) → C-31-1-37 ;
- « nom d'entrée en double → 400 » et `MAX_ENTRIES` (AC6b, T-a1) → C-31-1-38 ;
- test « aucun temporaire laissé » (T-a6b), vrai par construction (AC11) ;
- variante « dossier inspecté par un observateur / rien dans `temp_dir()` » du test de débordement (T-a4b) ;
- `partial` comme cas du bandeau de l'application (AC31, AC37, capture AC40 (o)) → C-31-1-40 ;
- condition « si un avertissement est affiché » du clic *Se reconnecter* (AC19) → C-31-1-39.

**Corrections** :

- **F1/R2 (HIGH)** : AC1 (chemin en anomalie écrit une fois, tel quel), AC6, AC7 (égalité de chaînes, puis
  `filesMissing`) ; T-a4b aller-retour ; M-a18.
- **R4/F2** : AC27 — `try_lock` pour « Sauvegarder maintenant » (409), attente pour l'export manuel et l'import ;
  AC32, AC35 ; M-c17.
- **F3** : AC27, AC8b, AC32 — gardes possédés, celui de l'import pris dans la fermeture du `spawn`, celui de
  `POST …/run` déplacé dans la tâche ; garde lexicale ; ordre `backup_lock` puis `FOR UPDATE` ; AC36 ; M-c18 ;
  C-31-1-41.
- **R1/F4** : AC6b réordonnée (manifeste d'abord, ensemble attendu ensuite, avant toute autre lecture) ; AC8 et T-a5 —
  restauration **en flux** (`restore_document(…, source: impl Read, byte_size, …)`) ; T-a6b (entrée plus grande que
  déclarée) ; angle mort de l'ouverture.
- **R3/F5** : T-a4b — dossier de sauvegarde de test distinct de `temp_dir()`, non inscriptible : seuil 0 → 500, seuil
  élevé → 200 ; échec bruyant en `root` ; M-a19 ; C-31-1-43.
- **F6** : AC30 — seuil `max(last_success_bytes, documents_bytes) × 1,2 + 256 Mio` ; AC36 ; M-c19 ; C-31-1-42.
- **F7/R8/F15/L5** : AC31 réécrite (bandeau de l'application : échecs et inactivité, seulement si activée,
  `role="status"` ; `partial` au panneau, avertissement `role="status"`) ; AC33, AC37, AC39, AC40 ; M-c11 reformulée ;
  M-c20 ; Pièges (`reports.spec.ts:242`).
- **F8/L1** : AC18b — prémisse corrigée (le message actuel est « Le serveur ne répond pas. Réessayez ultérieurement. »),
  issue inconnue étendue à `NETWORK_ERROR`, 502 et 504 ; tests du panneau neufs ; § Gardes ; M-b6 reformulée ; M-b7 ;
  T-b7.
- **R5** : AC32 et § Gardes — 33 constructeurs, `(8, 25)`, trois `GET` dans `GUARDED_BY_HANDLER` ; T-c4.
- **R6/F16** : AC14 — inventaire étendu par sujet (`KESH_ADMIN_BACKUP_DIR`, `INMEM`, temporaire, `./backup`), sites
  ajoutés : `admin-manual.tex:196`, `:782`, `:784`, `:843`, `:1466`, `:1608`, `:1946`, `:1947`, `:2268`,
  `export.rs:19`, `import.rs:93`, `.env.example:185`.
- **R7** : AC19, AC16, AC20 (5), AC21 (g), T-b1 ; M-b2 reformulée ; C-31-1-39.
- LOW : L2 (AC10, § Gardes), L3/F9 (AC28, AC29, AC32 : noms à la milliseconde ; C-31-1-44), L4 (P-5, AC4, AC14), L6
  (AC33), L7 (AC15, T-b2), L8 (AC1), F10 (AC30 : `wrap_writer` emprunte le `File`, `sync_all` classé), F11 (AC28,
  angle mort), F12 (angle mort), F13 (AC27 : commande filtrée, 8 + 1), F14 (AC30 : `f_bavail × f_frsize`), F17 (AC26 :
  signature unique de `run_once`, AC35).

Découpage : recompté, 31-1a : 3 modules, 31-1b : 4, 31-1c : 5 — la remédiation n'en ajoute à aucune ; pas de
redécoupage.

Recomptes (depuis la fiche et le registre, après remédiation) : **43 AC** (inchangé) ; **26 tâches** (inchangé) ;
**46 mutations** (M-a1 à M-a19, M-b1 à M-b7, M-c1 à M-c20 : 19 + 7 + 20 ; +7 depuis la P1 : M-a18, M-a19, M-b7, M-c17
à M-c20) ; **44 choix** (`grep -c '^### C-31-1-'` ; +8 : C-31-1-37 à C-31-1-44). Valeurs corrigées grepées sur la
fiche et le registre (`nom en double`, `MAX_ENTRIES`, `lock().await`, `\d{6}Z`, `HHMMSS>`, `(5, 23)`,
`28 constructeurs`, `sans bouton`, `aucun temporaire`, `restore_document(`, `wrap_writer(File`, `s'il n'y a jamais eu`,
`satisfont la même expression`, `pour le bandeau`, `l. ~1828`, `518-533`, `104-110`, `liste blanche`) : aucun résidu hors
des mentions historiques voulues (retraits datés, entrées passées du registre C-31-1-11 et C-31-1-29, que de nouveaux
choix amendent sans les réécrire).

### 2026-10-23 — Validation P3 (Sonnet 5.5, lentilles R et F) et remédiation (Opus 5.5)

Rapports : `kesh-gate-logs/311-validate-p3-R.md` (**0 CRITICAL / 0 HIGH / 5 MEDIUM / 7 LOW**) et
`311-validate-p3-F.md` (**0 / 0 / 4 / 11**). Chaque finding vérifié avant correction, au dépôt (`grep -nF`, `sed -n`) et
dans les sources du registre cargo : `tokio` 1.52.3 (`mutex.rs:618, 750`, non réentrant), `zip` 2.4.2
(`result.rs:52-64` : `From<ZipError> for io::Error` garde `kind()`, emballe l'erreur et perd `raw_os_error()`),
`routes/admin.rs:583-640` (`write_backup_file` sur `tokio::fs`), `:591` (`create_new`), `:615-619` (nom final),
`:537-542` (nom pré-import), `:881`, `:911-915`, `:957`, `:975-984` (gardes), `:37`, `:143` (seuls `ensure_not_pat` du
fichier), `admin_pat_denied_e2e.rs:40-77, 840-870`, `document_storage.rs:24-36` (`ReadDocumentError::Io`), `:66`
(`is_safe_storage_path` privée), `.env.example:84-92, 176-196`, `README.md:93`, `config.rs:238-262`,
`export.rs:40-58`, `routes/admin.rs:112-132`, `admin-manual.tex:325-342, 1721, 1739, 1887, 1958, 2194, 2199`,
`CHANGELOG.md:1-15`, `scripts/prepare-release.sh:244-250`, les branches `story/31-2` et `story/31-3` (toutes deux créent
`## [0.15.0] — Non publié`). **Aucun finding réfuté.** Recoupements traités une fois : R1 = F1, R5 = F2, R12 (AC1 iii)
= F14.

**Origine.** Nés de la remédiation P2 : R1/F1 (verrou de `run_once`), R5/F2 (`wrap_writer` emprunteur et
`spawn_blocking`), R2 (M-a4 rendue équivalente par l'AC6b réordonnée), F4 (inventaire étendu sans `.env.example`,
`README.md` ni doc-comments), F11 (« sans effet » de l'export abandonné). D'origine, non vus en P1 ni en P2 : F3
(fichier illisible), R3 (M-a12 non tuable par l'export, déjà relevé sous une autre forme en P1), R4 (lancement de la
boucle non testé), R6 à R10, R12, F5 à F10, F12 à F15. R11 corrige une réfutation de la P2.

**Annotation datée de l'entrée P2** (sans la réécrire) : la réfutation partielle de R6 (« `admin-manual.tex:2199` …
parle des montages `./documents`, `./inbox`, `./log` ; `sed -n 2199p | grep -o ".\{150\}backup"` : vide ») était
**fausse** : le `grep` de ce poste est `ugrep`, qui s'arrête en erreur (`exceeds complexity limits`) sur ce motif et ne
rend rien — une erreur lue comme un résultat vide. `awk 'NR==2199{print index($0,"backup")}'` rend 903 : la ligne
mentionne bien `./backup`. `:2194` et `:2199` entrent à l'inventaire (vrais à leur date, laissés).

**Simplifications** (une règle là où la P2 en avait posé deux) :

- **R1/F1** — `run_once` prend lui-même le verrou (`try_lock_owned`, `RunOutcome::Skipped`) ; le handler de
  `POST …/run` n'acquiert rien et traduit `Skipped` en 409 ; la boucle ignore `Skipped` (AC26, AC27, AC32, AC35 ;
  M-c6, M-c17 reformulées ; C-31-1-45, qui amende C-31-1-41).
- **R5/F2** — lecture de la base, puis **un seul** `spawn_blocking` autour d'une fonction synchrone `write_keshbackup` ;
  `write_backup_file` synchrone à fermeture de remplissage sur un clone du `File`, `sync_all` sur l'original ;
  `wrap_writer(File)` par valeur, `Arc<dyn BackupEnv>` ; ni `tokio::fs` mêlé au `ZipWriter`, ni `block_in_place`
  (AC4, AC28, AC30, T-a4, T-a5, T-c3 ; C-31-1-46).
- **R2** — l'AC6 ne porte plus que « ligne sans entrée » ; « entrée sans ligne » reste au seul AC6b (2) ; M-a4
  reformulée, tuable.
- **R12** — `ensure_not_pat` retiré des handlers neufs sauf le téléchargement (AC32, § Gardes ; C-31-1-50).

**Corrections** :

- **F3** — raison `unreadable`, deux passes, abandon de la sauvegarde sur divergence à la copie (AC3, AC5, AC18, T-a4,
  T-a4b ; M-a20 ; re-hachage déclaré équivalent au niveau des tests ; C-31-1-47).
- **F4** — huit sites ajoutés au tableau de l'AC14 ; inventaire des sujets **défini par une commande** (94 lignes à
  `c7639e80`), T-a8 (C-31-1-48). **R11** — `admin-manual.tex:1721`, `:1739`, `:1887`, `:1958`, `:2194`, `:2199` au
  tableau.
- **R3** — test unitaire direct d'`open_document`, M-a12 reciblée (AC3, T-a5).
- **R4** — garde lexicale du lancement par `main.rs` (AC26, AC36, T-c3) ; M-c21 ; cadence et panique en angle mort.
- LOW : R6 (§ Gardes : `echecs_avant_sauvegarde_tous_convertis`, `avant_sauvegarde_branchee…` « rougit, puis muette
  si… », tests de `write_backup_file`) ; R7 (AC14, AC39 : `## [0.15.0] — Non publié`, union avec les 31-2/31-3) ; R8
  (AC21, AC40 : voies d'obtention des captures, `page.route`) ; R9 (AC33, T-c4, T-c5, modules de la 31-1c :
  `errors.rs`) ; R10 (AC28, AC32 : numéros de ligne) ; R12 (AC1 (iii), M-a6 et son test, AC8b `JoinError` en angle
  mort, T-a6b dossier de sauvegarde non inscriptible, AC36 audits et `nextDueAt`) ; F5 (AC6b borne cumulée, `ReadLimits`,
  M-a14 reformulée ; C-31-1-49) ; F6 (`saturating_add`, `byteSize` plafonné) ; F7 (`[0-9]`, AC29, AC32) ; F8 (Nginx,
  AC14) ; F9 (`ParsedBackup` porte l'archive, AC8, AC11, T-a6) ; F10 (AC4, AC30 : `kind()` d'abord) ; F11 (AC27 :
  formulation, garde lexicale « contre l'inattention ») ; F12 (angle mort ; C-31-1-51) ; F13 (AC30, AC33, AC39 : issue
  dite à l'écran ; C-31-1-51) ; F14 (= R12) ; F15 (AC8 : `sync_all` du dossier des documents).

**À ouvrir en issue (proposé à l'orchestrateur)** : F12 — un indicateur « dépasse la taille d'import » dans la liste des
sauvegardes du serveur (`enhancement`, P4), non traité ici (C-31-1-51).

Découpage : recompté, 31-1a : 3 modules (`kesh-db`, `kesh-api`, docs — `.env.example` et `README.md` sont des docs),
31-1b : 4, 31-1c : 5 (`errors.rs` est dans `kesh-api`) ; pas de redécoupage.

Recomptes (depuis la fiche et le registre, après remédiation) : **43 AC** (`grep -cE '^\*\*AC[0-9]+b? '`, inchangé) ;
**26 tâches** (12 + 7 + 7, inchangé) ; **48 mutations** (`grep -cE '^\| M-'` : M-a1 à M-a20, M-b1 à M-b7, M-c1 à
M-c21 — 20 + 7 + 21 ; +2 depuis la P2 : M-a20, M-c21 ; reformulées : M-a4, M-a6, M-a12, M-a14, M-c6, M-c17) ; **51
choix** (`grep -c '^### C-31-1-'` ; +7 : C-31-1-45 à C-31-1-51). Valeurs corrigées grepées sur la fiche et le registre
(`build_keshbackup_into`, `TABLE_ENTRY_MAX_BYTES`, `wrap_writer<'a>`, `&'a mut File`, `[Unreleased]`, `\d`, `600-606`,
`535-541`, `chaque handler appelle`, `4 à 7`, `env: &dyn BackupEnv`, `&env`, `take(byteSize + 1)`, `byte_size + 1`,
`try_lock`, `lock().await`) : aucun résidu hors des mentions historiques voulues (entrées passées du Change Log et du
registre — C-31-1-23, C-31-1-41 —, que C-31-1-45 amende sans les réécrire ; la mention de l'emprunt de la P2 à l'AC30 ;
`take(byteSize + 1)` de la passe de copie de l'export, AC3, où `byteSize` vient de la passe de hachage et non d'un
manifeste).
