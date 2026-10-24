# Story 31.1 : Une sauvegarde qui restaure tout, faite par Kesh

## Status

**ready-for-dev** (trois clés) — fiche créée le 2026-10-23 en autonomie (`bmad-create-story`, Opus 5.5), sur
`1848012d` (kickoff de l'Epic 31), worktree `/home/gcorbaz/devel/kesh-31-1`, branche `story/31-1`. Validation P1
faite (Sonnet, R 0/0/10/8, F 0/2/9/8) et remédiée le 2026-10-23 ; validation P2 faite (Opus, R 0/0/8/8, F 0/1/7/9)
et remédiée le 2026-10-23 ; validation P3 faite (Sonnet, R 0/0/5/7, F 0/0/4/11) et remédiée le 2026-10-23 ;
validation P4 faite (Opus, R 0/0/3/11, F 0/0/4/7) et remédiée le 2026-10-24 ; validations P5 à P8 faites,
**terminée au plafond de huit passes** (Change Log). **31-1a : done** (revue de code P1 terminée le 2026-10-24) ;
**31-1b : review** (développement terminé le 2026-10-24 ; revue de code P1 à lancer, `311b-review-prompt-p1.md`,
**Sonnet**, lentilles B, E, A) ; 31-1c : ready-for-dev.

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
§ 31-1. Cible : **v0.15.0**. Choix : `epic-31-choix-autonomes-311.md` (C-31-1-1 à C-31-1-58). Réglages nouveaux dans
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
| Plusieurs processus Kesh sur une même base | une instance par base (déploiement documenté) | **aucune** protection entre processus (le verrou est en mémoire ; la réservation en base de la P2, qui ne protégeait rien, est retirée en P4, C-31-1-54) ; écrit en angle mort et au manuel |
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
La remédiation P4 non plus : la passe unique et `abort_file`, la promotion de `regex` et l'union faite en Rust restent
dans `kesh-db` et `kesh-api` (31-1a) ; les commentaires des deux `docker-compose*.yml` (31-1a, 31-1c) sont de la
documentation d'exploitation, comptés avec docs comme `.env.example` ; `importMaxMib`, `is_overdue` et le garde de
l'export dans le `spawn_blocking` restent dans `kesh-api`, le frontend, `kesh-i18n` et docs (31-1c) ; la réservation
retirée enlève du code sans module ; recompté : 3, 4, 5.

## Critères d'acceptation

### Partie a — `31-1a-fichiers-dans-la-sauvegarde`

**AC1 — Inventaire des fichiers référencés, lu dans l'instantané.** `kesh_db::backup` expose
`ReferencedFile { storage_path: String, sha256: String }` et l'`InstallationExport` porte `referenced_files:
Vec<ReferencedFile>` (dédoublonnés par `storage_path`, triés), lus par `export_installation_in` **dans le même
`ReadSnapshot`** que les tables, après elles :
`SELECT storage_path, file_hash FROM imported_supplier_invoices` et `SELECT pdf_storage_path, pdf_sha256 FROM invoices
WHERE pdf_storage_path IS NOT NULL`, **deux requêtes** (ou `UNION ALL`), jamais un `UNION` ni un `DISTINCT` SQL : les
colonnes sont en `utf8mb4_unicode_ci`, insensible à la casse et aux espaces finaux, qui fusionnerait `x.pdf` et `X.PDF`
de deux lignes alors que l'import compare les chemins par égalité de chaînes (AC7) — la ligne fusionnée ne serait ni
dans `files` ni dans `filesNotIncluded`, et la sauvegarde authentique refusée en 400 (F6, P4). Le dédoublonnage et la
détection des anomalies (ii) se font **en Rust, par égalité d'octets**. Toutes les lignes de `imported_supplier_invoices` comptent, quel que soit leur
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
l'ouverture ou pendant la lecture — droits, `EIO`, dossier à la place du fichier ; journalisée `warn!` avec
`?kind`) ou en **anomalie** (AC1) n'empêche pas la sauvegarde : il est listé dans `filesNotIncluded` et **n'est pas**
écrit dans l'archive (C-31-1-4, C-31-1-47 — un seul fichier illisible ne doit pas faire échouer chaque sauvegarde).
**Une seule passe par fichier** (C-31-1-53, P4 : la P3 en prescrivait deux sur la prémisse fausse que le crate `zip`
ne sait pas retirer une entrée commencée — `ZipWriter::abort_file`, `zip-2.4.2/src/write.rs:1134`, la retire, et
`finalize`, `:1501-1524`, réécrit le pied si des octets abandonnés le dépassent) : ouverture par `open_document`
(`NotFound` → `missing`, autre erreur → `unreadable`, aucune entrée commencée), `byteSize` lu par `metadata()` du
descripteur ouvert, `start_file("files/<path>")`, puis une **boucle de copie écrite à la main** (lecture d'un tampon
dans le fichier à travers `take(byteSize + 1)`, mise à jour d'un `Sha256`, écriture dans l'entrée) — pas `io::copy`,
qui confondrait l'erreur de lecture du fichier et l'erreur d'écriture de l'archive. À la fin : empreinte différente du
`sha256` déclaré, ou nombre d'octets différent de `byteSize` → `abort_file()` et raison `altered` ; **erreur de
lecture** du fichier en cours de copie → `abort_file()` et raison `unreadable` ; seule une **erreur d'écriture** de
l'archive (ou une erreur d'`abort_file` lui-même) abandonne la sauvegarde (`BackupWriteError` : export 500, sauvegarde
pré-import en échec donc import refusé, sauvegarde automatique `failed` ou `disk_full`). Chaque pièce est donc lue
**une fois** ; aucune entrée fausse n'entre dans l'archive, si bien qu'une archive que l'import refuserait (AC7) ne
sort jamais. ⚠️ À établir au dev par un test (T-a4b) : une archive dont une entrée a été abandonnée **au milieu**,
suivie d'une entrée saine, se relit par `ZipArchive::new` et s'importe. Les fichiers du dossier que la base ne référence pas
ne sont pas emportés. **Aucun chemin n'est joint au dossier des documents avant validation** : la lecture passe par une
fonction de `document_storage` (`open_document(dir, storage_path) -> Result<File, ReadDocumentError>`, qui applique
`is_safe_storage_path`, rendue `pub(crate)`, comme `read_document` le fait déjà) — jamais `documents_dir.join(…)` dans
`export.rs`. Comme l'AC1 classe déjà `mismatch` tout chemin hors forme, aucun test d'export n'atteint ce contrôle : il
se prouve par un test **unitaire direct** d'`open_document` (`"../x"`, `"/etc/hostname"`, `"a/b"`, `""` →
`InvalidPath`, sans toucher au disque), qui tue M-a12 (R3, P3). L'entrée dossier `files/` est écrite **même sans aucun fichier** (le contrôle de présence de l'import,
`import.rs:98-102`, l'exige pour le format 1 et le 2), et **avant** les entrées `files/<nom>` : c'est l'ordre que le
test « altéré suivi d'un sain » de T-a4b exerce avec `abort_file` (L3 de la P7). Une archive de format 2 portant cette
entrée est la forme normale ; l'entrée `files/` appartient à l'ensemble attendu de l'AC6b (2) et n'est pas une
« entrée `files/*` sans ligne » (T4 de la P6).

**AC4 — L'export n'assemble plus l'archive en mémoire.** Deux temps, et deux seulement (C-31-1-46, P3 : la forme de la
P2 — un écrivain empruntant le `File`, des copies en `spawn_blocking` séparés, un `write_backup_file` sur `tokio::fs` —
ne compilait pas ou copiait des Gio sur un fil de l'exécuteur) :

1. **lecture de la base**, asynchrone : `export_installation(pool)` (instantané, tables et `referenced_files` en
   mémoire, comme aujourd'hui — quelques dizaines de Mio) ;
2. **tout le reste dans un seul `tokio::task::spawn_blocking`**, par une fonction **synchrone**
   `write_keshbackup<W: Write + Seek>(export: &InstallationExport, documents_dir: &Path, w: W) -> Result<(W,
   KeshBackupMeta), BackupWriteError>` : écriture des tables, lecture des fichiers **un par un** (`std::fs`, une passe,
   AC3 ; jamais tous en mémoire), `finish`. La fermeture **possède** tout ce qu'elle touche (l'`InstallationExport`, les
   chemins, l'écrivain ; à la 31-1c un `Arc<dyn BackupEnv>` ; pour l'export manuel, le garde de `backup_lock`, AC27) :
   aucun emprunt, aucun `tokio::fs` mêlé au `ZipWriter` synchrone, **aucun `block_in_place`** (il panique sous le
   runtime mono-fil des `#[tokio::test]`).

**Un seul point d'entrée** (C-31-1-56, R7, P4) : les deux temps sont tenus par **une** fonction asynchrone,
`build_keshbackup(pool, documents_dir, remplir)`, qui appelle `export_installation(pool)` puis
`spawn_blocking(move || remplir(export, documents_dir))` et rend ce que rend `remplir` ; `remplir` est une fermeture
`FnOnce(InstallationExport, PathBuf) -> Result<T, BackupWriteError> + Send + 'static`, propre à chaque appelant, qui
appelle `write_keshbackup` sur son écrivain (export : le `SpooledTempFile` ; pré-import : `write_backup_file` ; 31-1c :
`write_backup_file` et `wrap_writer`). `build_keshbackup` reste **le seul appelant d'`export_installation(` dans le
code de production de `kesh-api`** ; aucun appelant ne le contourne.

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
garde lexicale `build_keshbackup_ne_lit_la_base_qu_une_fois` reste sur le corps de `build_keshbackup` et reste exacte
(`pool` y apparaît deux fois : paramètre et `export_installation(pool)`) ; elle gagne une seconde assertion : dans le
code de production de `crates/kesh-api/src` (lignes `//` écartées, `mod tests` exclus), `export_installation(`
apparaît **exactement une fois**, dans `build_keshbackup` — un appelant qui lirait la base lui-même la fait rougir.
`write_keshbackup` ne reçoit aucun pool : l'invariant « une seule lecture » est tenu par le type dans le
`spawn_blocking`, et par la garde au-dehors.

**AC5 — Ce que l'export rend, et ce qu'il dit.** La réponse de `GET /admin/full-export` porte deux en-têtes,
`X-Kesh-Backup-Files: <n inclus>` et `X-Kesh-Backup-Files-Not-Included: <n>` (exposés à l'écran par la 31-1b). L'audit
`admin.full_export` ajoute `file_count`, `files_bytes`, `files_not_included` au détail. Un fichier absent ou altéré est
journalisé (`warn!`, chemin et raison, jamais le contenu) ; un fichier `unreadable` aussi, avec `?kind`. Le chemin,
venu de la base et possiblement hors forme (AC1 (iii) : `../x`, retour à la ligne), se journalise **en `Debug`**
(`?path`), jamais `%path` ni interpolé dans le message (F8, P4 ; même règle qu'à l'AC6).

**AC6 — Import : formats acceptés.** `parse_and_verify` accepte `formatVersion` 1 **et** 2, refuse au-delà
(`InvalidBackupStructure`, message « formatVersion N non supporté (maximum 2) »). Format 1 : règle actuelle inchangée
(`files/` présent et vide). Format 2 : `files/` présent ; **toute ligne de `manifest.files` a son entrée**
`files/<path>` (une ligne sans entrée → 400) — l'autre moitié de la bijection, une entrée `files/*` sans ligne, est
déjà refusée par le contrôle d'ensemble de l'AC6b (2), qui seul la porte (R2, P3 : la porter deux fois rendait M-a4
équivalente) ; chaque nom satisfait
`^[0-9a-f]{64}\.[a-z0-9]{1,8}$` **et** `is_safe_storage_path`, et son préfixe égale le `sha256` déclaré ; aucun
sous-dossier sous `files/`. Cette expression (et celles des noms de sauvegardes, AC29, AC32) est du code de
production : `regex` passe des `[dev-dependencies]` aux `[dependencies]` de `kesh-api` (`Cargo.toml:83`, déjà au
`Cargo.lock` ; R10, F11, P4), comme `tempfile` (AC4). Les chemins de `filesNotIncluded` ne sont soumis à **aucune** expression : ils ne servent
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
atomiquement (journalisé `warn!`, chemin en `?path`). Un échec d'écriture (dossier non inscriptible, disque plein) → la restauration est
**refusée**, la base n'est pas touchée (`AdminFullImportFailed`, 500, message « aucune donnée n'a été remplacée ») ;
les fichiers déjà écrits restent (ils sont nommés par leur contenu : inoffensifs, C-31-1-6). Aucun fichier du dossier
des documents n'est jamais supprimé par l'import. L'écriture des fichiers (`std::fs`, `fsync`) s'exécute en
`spawn_blocking` ; à la fin de la boucle, le dossier des documents lui-même est synchronisé
(`File::open(documents_dir).and_then(|d| d.sync_all())` ; F15, P3) pour que les entrées de dossier, et non seulement
les contenus, soient sur disque avant le `COMMIT` — non testable (durabilité après coupure), dit au doc-comment. Un
échec de **cette** synchronisation (certains montages réseau ou FUSE refusent `fsync` sur un dossier, `EINVAL`) est
journalisé `warn!` et **ne refuse pas** l'import : les contenus sont déjà synchronisés un à un, et en faire une cause
de refus permanent rendrait la restauration impossible sur ces montages (F7, P4). L'archive
parvient à cette écriture par `ParsedBackup`, qui **porte** la `ZipArchive<R>` ouverte par `parse_and_verify` (le
temporaire d'import est anonyme et ne se rouvre pas par chemin, AC11 ; F9, P3). Le `spawn_blocking` est `'static` et
`ZipArchive::by_name` exige `&mut self` : `run_backup_and_restore` reçoit donc `ParsedBackup` **par valeur** (il en
déplace l'archive dans la fermeture, `Option::take` si le reste de `ParsedBackup` sert encore après, et la fermeture la
rend avec son résultat) — jamais `&parsed` (R13, P4). Ce que l'ordre « fichiers avant la base » achète se teste par sa négation : écrire les fichiers
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
si : `warn!` par fichier manquant, chemin en `?path`, F8, P4). Le calcul hache chaque fichier référencé présent (des Gio
sur la même installation) : il s'exécute en `spawn_blocking`, comme l'écriture (F10, R13, P4). Le cas « même
installation → 0 » est **testé** (T-a6b, R2, P4) : c'est le seul qui distingue un calcul sur la destination d'un calcul
sur le backup (M-a9), et c'est le cas réel d'une restauration sur le NAS d'origine.

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
puis copie le champ `file` **par morceaux** (`field.chunk()`) dans un fichier temporaire **anonyme** de ce dossier,
écrit par `tokio::fs::File::from_std` (jamais un `std::fs::File` sur un fil de l'exécuteur ; F10, P4) puis rendu en
`std::fs::File` (`into_std`) pour `parse_and_verify`, qui tourne en `spawn_blocking` (AC7)
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
#503 décrit, désormais annoncé). Et la sauvegarde **format 1 restaurée sur la même installation** (dossier des
documents A, qui porte encore les deux fichiers) : `filesRestored = 0`, **`filesMissing = 0`**, les deux `GET` rendent
`200` — sous M-a9 (calcul sur le backup), 2 (R2, P4).

**AC14 — Manuel d'administration et CHANGELOG.** `docs/manual/fr/admin-manual.tex` : le format 2 et ce qu'il emporte
(§ *Export/import d'installation*, l. 1826) ; les phrases « ni dans le `.keshbackup` » / « sans les fichiers » / note
de bas de page #503 (l. ~1465, 1474, 1828, tableau l. ~1950-1963) **corrigées** ; ce qui n'est pas emporté (fichiers non
référencés, `filesNotIncluded`, `.env`, réglage de la sauvegarde automatique — ce dernier écrit par la 31-1c) ; le
volume, le débordement dans `KESH_ADMIN_BACKUP_DIR`, le nouveau défaut de `KESH_ADMIN_IMPORT_MAX_MB` (tableau l. ~783 et
§ *Variables d'environnement*) ; une sauvegarde de la 0.15.0 ne s'importe pas dans une 0.14.x, et le manuel nomme le
message que l'administrateur verra : une sauvegarde portant au moins une pièce est refusée par la 0.14.0 **avant** la
lecture de `formatVersion` (`import.rs:91-96` : « le dossier files/ doit être vide »), d'où à l'écran « Fichier de
sauvegarde invalide ou corrompu » (`INVALID_BACKUP_STRUCTURE`) devant une sauvegarde saine ; une sauvegarde sans
aucune pièce est refusée par `formatVersion` (`:111-116`) (R11, P4).
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
le dire, avec l'ordre de grandeur « taille du dossier des documents + base ». Et le **pic** d'un import, sans seuil
d'espace préalable (seule l'automatique en a un, AC30) : l'envoi reçu (AC11), la sauvegarde pré-import et les pièces
écrites (AC8) coexistent, soit environ **trois fois** la taille de l'archive sur le volume du dossier de sauvegarde,
souvent celui de la base (F9, P4) — *jusqu'à* environ trois fois, selon que le dossier de sauvegarde et le dossier des
documents partagent ou non le volume (le cas des compose fournis), et moins sur une machine neuve dont la sauvegarde
pré-import ne porte aucune pièce (R7 de la P5, appliqué à l'étape 0 du développement). Les documents **en attente** dans
`KESH_INBOX_DIR` (pas encore importés, ou déplacés en `failed/`) ne sont référencés par aucune ligne et ne sont pas
emportés : le manuel le dit avec les fichiers non référencés (F6 de la P5).

**Inventaire clos des phrases à traiter** (grep des valeurs `503`, `512`, `n'emporte`, `sans les fichiers`, `aucun
binaire`, `files/`, `formatVersion`, relevé en P1 ; **étendu en P2** aux sujets que le code de la 31-1a ou de la 31-1c
rend faux — `KESH_ADMIN_BACKUP_DIR`, `INMEM`, `temporaire`, `./backup`, `keshbackup` — sur `docs/`, `crates/`,
`frontend/`, `website/`, `.env.example`, `README.md` ; T-a8 et T-c7 le rejouent et traitent tout site neuf).

**Clos par la forme** (F4, P3 : l'extension de la P2 avait été rejouée sur les `.tex` et `config.rs`, pas sur
`.env.example`, `README.md` ni les doc-comments voisins — huit sites manquaient). L'inventaire des **sujets** est
**l'ensemble des lignes que rend cette commande**, et non une liste qu'on complète :

```sh
P='KESH\\?_ADMIN\\?_(BACKUP\\?_DIR|EXPORT\\?_INMEM\\?_MB|IMPORT\\?_MAX\\?_MB)|admin_backup_dir|admin_export_inmem_mib|admin_import_max_mib|build_keshbackup|pré-import|pre-import|sauvegarde de sécurité|in-memory|spill'
{ grep -rniE "$P" docs/manual/fr/*.tex docs/api-external.md README.md .env.example website docker-compose*.yml
  grep -rniE "^\s*//.*($P)" crates/kesh-api/src crates/kesh-db/src; }
```

(`\\?_` : le LaTeX écrit `KESH\_ADMIN\_…`, qu'un motif sans échappement manque — c'est ainsi que `admin-manual.tex:196`
échappait au grep des noms. `-i` et `docker-compose*.yml` depuis la P4 (F3) : les deux compose que l'exploitant copie
portaient « **S**auvegarde de sécurité … », que la commande de la P3 manquait deux fois — fichiers hors liste, motif
sensible à la casse.) À `6f0828cf`, elle rend **103 lignes** (27 du manuel d'administration, 9 de `.env.example`, 2 du
`README.md`, 4 de `docker-compose.yml`, 4 de `docker-compose.prod.yml`, 57 de commentaires dans `crates/` — la 57ᵉ,
`config.rs:255`, n'apparaît qu'en `-i` ; 94 à `c7639e80` sans `-i` ni compose). **Chaque ligne** est soit au tableau
ci-dessous, soit **ajoutée au tableau du Dev Agent Record avec son traitement**, soit triée « intacte » au Dev Agent
Record avec sa raison (code d'exemple de compose, historique, sans objet) — une ligne que la 31-1 rend fausse ou
incomplète n'est jamais triée intacte (R9, P4) ; au dev, la commande est rejouée sur le code modifié et toute ligne
neuve est triée de même. Les six lignes `KESH_ADMIN_*: ${…}` des deux compose (transmission des variables) sont
intactes. Le grep des **valeurs** (P1) reste à rejouer à côté (T-a8).

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
| `docker-compose.yml:154-158`, `docker-compose.prod.yml:172-176` (« Sauvegarde de sécurité prise avant chaque import … à purger par l'exploitant (une sauvegarde par import) ») | jumeaux de `.env.example:187-192` : le dossier porte aussi les temporaires de l'export et de l'import, et chaque sauvegarde pré-import pèse la base **plus** les pièces (31-1a) ; puis `auto/` et sa rétention (31-1c, AC39) (F3, P4) |
| `docker-compose.prod.yml:165-168` (« bind mounts … que copie Hyper Backup (la base, elle, se sauvegarde par le dump du manuel) ») | hors de la commande (aucun sujet), relevé par F3 : complété à la 31-1c — Kesh sauvegarde aussi la base et les pièces dans `./backup/auto` |
| `crates/kesh-api/src/config.rs:895-896` (« l'export `.keshbackup` spille sur fichier temporaire ») | jumeau de `:240-243` : emplacement du temporaire (31-1a) (R9, P4) |
| `crates/kesh-api/src/config.rs:1268-1269` (doc de `DEFAULT_ADMIN_BACKUP_DIR` : « sauvegarde de sécurité prise avant chaque import ») | les temporaires (31-1a), puis `auto/` (31-1c) (R9, P4) |
| `.env.example:220-223` et `:255-257` (« `./backup` (sauvegarde pré-import, sans variable d'hôte) ») | `./backup` porte aussi `auto/` : complétés à la 31-1c (R9, P4) |
| `crates/kesh-api/src/routes/admin.rs:52` (« DC8 — in-memory sous plafond, au-delà spill fichier temporaire ») | réécrit avec l'AC4 (31-1a) (R9, P4) |
| `crates/kesh-api/src/admin_backup/export.rs:1-12` (en-tête de module : « `build_keshbackup` assemble le conteneur ZIP », lecture par `export_installation`) | réécrit (31-1a, AC4 : `build_keshbackup` générique sur le remplissage, `write_keshbackup`) (R9, P4) |
| `admin-manual.tex:1838` (« Kesh lit toutes les tables dans un seul instantané ») | complété (31-1a) : les pièces sont lues hors de l'instantané, mais nommées par leur contenu et jamais réécrites (P-3) (R9, P4) |
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
absent, altéré, illisible ou incohérent avec la base) — voir le journal du serveur » (la raison `mismatch` compte
aussi, R12, P4) ; si `X-Kesh-Backup-Exceeds-Import-Limit` est posé,
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
immédiatement, `AdminRestorePanel.svelte:119-128`). Le toast `admin-restore-toast-success` (« Import réussi — vous
allez être déconnecté. », l. 120-122) est **retiré** : l'avis du panneau le remplace, et « vous allez être déconnecté »
deviendrait faux tant que le clic n'a pas eu lieu (R14, P4) ; sa clé est supprimée des quatre locales.

**AC20 — E2E de bout en bout (`frontend/tests/e2e/backup-restore-new-install.spec.ts`).** (1) `seedTestState('with-company')`,
connexion ; un document fournisseur importé par l'inbox (fixture synthétique `spc_e2e_invoice.png` passée par
`uniqueCopy` de `helpers/inbox.ts` — des octets propres au spec, pour que la suppression de l'étape 3 ne retire pas le
fichier d'une autre spec) et une facture validée dont le PDF est figé par un téléchargement ; (2) export par l'écran, fichier
enregistré par le runner ; (3) `seedTestState('setup-required')` **puis suppression des deux fichiers** du dossier des
documents (`DOCUMENTS_DIR` de `helpers/inbox.ts` ; garde bruyante : chaque fichier doit exister avant suppression et
ne plus exister après — sinon le spec échoue, il ne se tait pas) ; (4) `/setup` (nouvel admin), assistant, lien
**« Restaurer une sauvegarde »**, import du fichier, confirmation ; (5) avis de l'AC19 avec « 2 pièces restaurées »,
lu **avant** le clic *Se reconnecter* (qui seul déclenche la redirection, C-31-1-39), puis reconnexion avec les
identifiants de la sauvegarde ; (6) l'application s'ouvre (pas l'assistant) ; **la comptabilité est revenue** : la
facture de l'étape (1) figure dans la liste des factures avec son numéro, et son écriture au journal (même montant)
(R3, P4 : le commentaire de Guy du 2026-10-10 sur #503 nomme la comptabilité en premier) ; le document
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
en important l'archive de (e), le fichier **toujours absent** du dossier (corrigé en P4, R4 : un fichier retiré
**après** l'export est réécrit par l'import depuis l'archive, et `filesMissing` vaut alors 0) ; (f) et (i) **par interception réseau** (`page.route` : en-tête
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
jour plus tard (C-31-1-17). **Horloge revenue en arrière** (C-31-1-57, F4, P4) : un `last_success_at` ou un
`last_attempt_at` **postérieur** à `now` (horloge partie dans le futur puis corrigée par NTP, NAS redémarré sur une
horloge matérielle remise à zéro) vaut **nul** dans ces deux clauses — sans quoi la différence négative rendrait
`is_due` faux, et la sauvegarde s'arrêterait sans échec ni bandeau jusqu'à ce que l'horloge rattrape la valeur
enregistrée ; journal `warn!` « horloge en arrière ». Pas de boucle : le tour suivant enregistre `last_attempt_at =
now`, qui n'est plus dans le futur. Tests aux bornes (égalité, une milliseconde avant, désactivé, jamais tenté,
**dernière réussite dans le futur → dû**, **dernière tentative dans le futur → dû**).

**`auto_backup::is_overdue(schedule, now) -> bool`** (même module, fonction pure ; C-31-1-57) : vrai si et seulement si
`enabled` **et** `last_success_at` non nul **et** (`now - last_success_at ≥ 2 × interval` **ou** `last_success_at >
now`). C'est le **second cas du bandeau** (AC31 (b)), calculé **par le serveur** et rendu par `GET backup-schedule`
(`overdue`, AC32) : le navigateur ne compare aucune date à sa propre horloge, qui peut différer de celle du serveur
de quelques secondes (une réussite « dans le futur » pour le navigateur allumerait le bandeau à tort) ou de plusieurs
heures. Tests aux bornes (deux intervalles moins une milliseconde → faux ; égalité → vrai ; dans le futur → vrai ;
`last_success_at` nul → faux ; désactivé → faux).

**AC26 — La tâche de fond.** `main.rs` (et lui seul — ni `build_router`, ni les tests) lance une tâche
`auto_backup::spawn(state.clone())` : premier contrôle **5 minutes** après le démarrage, puis toutes les **5 minutes**
(`tokio::time::interval`, `MissedTickBehavior::Skip`) ; à chaque tour, lit le réglage, et si `is_due` appelle
`run_once(&state, env.clone(), now, Trigger::Schedule)`. **Signature unique** (fixée en P2, F17 ; `env` possédé depuis
la P3, pour entrer dans le `spawn_blocking` de l'AC4) : `run_once(state: &AppState, env: Arc<dyn BackupEnv>, now:
NaiveDateTime, trigger: Trigger) -> Result<RunOutcome, …>`, où `RunOutcome` vaut `Skipped` (verrou pris, AC27 : rien
d'écrit ; la « réservation perdue » de la P2 est retirée en P4, C-31-1-54) ou `Done(statut)` ; en production, `env` est `Arc::new(RealBackupEnv)` (construit par
`spawn` et par la route `POST …/run`, pas porté par `AppState`) ; les tests passent leur `BackupEnv` et leur `now`.
Toutes les mentions de `run_once` de cette fiche désignent cette signature. La boucle de fond ignore `Skipped` (journal
`info!`). Une erreur de lecture est journalisée et le tour suivant réessaie ; une panique dans un tour n'arrête pas la
boucle (le tour s'exécute dans un `tokio::spawn` dont le `JoinError` est journalisé, C-31-1-18). **Le lancement se
prouve** (R4, P3 : c'est la demande de #503, « faite par Kesh lui-même », et rien ne rougissait si `main.rs` oubliait
l'appel) : garde lexicale dans `auto_backup.rs` (`mod tests`, sur `include_str!` de `main.rs` et `lib.rs`) —
`auto_backup::spawn(` apparaît **exactement une fois** dans `main.rs` et **aucune** dans `lib.rs` (où elle se
déclencherait dans chaque test monté par `build_router`), **lignes `//` écartées** (R8, P4 : un appel mis en
commentaire, le geste de débogage typique, garderait sinon le compte à 1 ; `main.rs` l'écrira
`kesh_api::auto_backup::spawn(`, que le motif trouve) ; elle tue M-c21. La cadence (5 min, `Skip`) et l'isolation
d'une panique ne sont **pas** testées : quelques lignes relues en revue ; les tester demanderait d'extraire et
d'horloger la boucle (angle mort écrit).

**AC27 — Une seule sauvegarde à la fois.** `AppState` gagne `backup_lock: Arc<tokio::sync::Mutex<()>>` (défaut dans
`new_for_tests` ; **neuf sites** mis à jour : les huit littéraux `AppState { … }` — `main.rs:355`,
`middleware/auth.rs:267`, `tests/password_recovery_e2e.rs:127`, `tests/invoice_frozen_pdf_e2e.rs:102`,
`tests/invoice_send_email_e2e.rs:103,1034,1147`, `tests/setup_admin_e2e.rs:81` — et le `Self { … }` de
`AppState::new_for_tests`, `lib.rs:105` ; recompter au dev par `grep -rnE "AppState \{$" --include=*.rs crates | grep
-vE "struct |impl |fn "` — huit lignes, plus le `Self {` ; sans le filtre, la commande rend onze lignes : les huit,
plus la définition `lib.rs:49`, l'`impl` `lib.rs:89` et la signature `fn test_state() -> AppState {`
`middleware/auth.rs:240` — « douze … et un commentaire » corrigé en P4, R10). **Une seule règle par
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
- l'**export manuel** prend `lock_owned().await` et **déplace le garde dans la fermeture `remplir`** qu'il passe à
  `build_keshbackup` (AC4), donc dans le `spawn_blocking` d'assemblage : le garde est rendu à la fin de l'assemblage,
  quel que soit le sort du client (C-31-1-55, F1, P4).

Toujours un garde **possédé** (`lock_owned` / `try_lock_owned`), jamais un garde emprunté. Garde lexicale dans
`routes/admin.rs` et `auto_backup.rs` (`mod tests`) — une garde contre l'inattention, pas contre une forme délibérée
(une variable intermédiaire la contourne ; F11, P3) : aucune occurrence de `backup_lock.lock(` ni
`backup_lock.try_lock(` (formes empruntées) ; `try_lock_owned` n'apparaît **que** dans `auto_backup.rs` (jamais dans
`routes/admin.rs`) ; dans `full_import`, `lock_owned` se trouve dans la fermeture du `spawn` ; dans `full_export`, le
nom du garde apparaît dans la fermeture passée à `build_keshbackup` (tue M-c24). **Ordre** : `backup_lock` est toujours pris **avant** `_kesh_version FOR UPDATE`
(seul l'import prend les deux : `run_backup_and_restore` ouvre le `FOR UPDATE` après) — l'ordre inverse ferait tenir
le verrou de ligne pendant l'attente d'un export de plusieurs minutes. **Fenêtre de détention** (C-31-1-30) : l'import le
prend **après** la réception de l'envoi et `parse_and_verify` (un client lent ne bloque pas les sauvegardes pendant
l'envoi) et le garde jusqu'à la fin de la restauration (sauvegarde de sécurité, écriture des fichiers, restauration) ;
l'export manuel le prend pour l'**assemblage** de l'archive dans son temporaire et le **rend avant la diffusion** (un
téléchargement lent ne bloque rien). Le garde vivant dans le `spawn_blocking`, un client parti pendant l'assemblage
(proxy au délai plus court, onglet fermé) abandonne le futur du handler **sans relâcher le verrou** : l'assemblage
orphelin va au bout puis rend le garde, et l'essai suivant attend — jamais N assemblages simultanés débordant dans
`KESH_ADMIN_BACKUP_DIR` (F1, P4 : la P3 laissait le garde au handler, et chaque nouvel essai prenait le verrou pendant
que les assemblages abandonnés continuaient). Le temporaire de l'assemblage orphelin, anonyme, se perd ; un seul à la
fois. Une fois le verrou pris, `run_once` enregistre sa tentative (`record_attempt` : `UPDATE … SET last_attempt_at =
?`, sans condition) avant d'écrire. **Plus de réservation conditionnelle en base** (C-31-1-54, F5, P4) : la forme de la
P2 (`WHERE last_attempt_at <=> ?` sur la valeur lue par `run_once` lui-même) ne fermait que les quelques millisecondes
entre sa propre lecture et son `UPDATE`, ne protégeait pas `Trigger::Manual`, et laissait le nettoyage des `.partial`
(AC28) supprimer celui d'un autre processus ; dans un seul processus, le verrou suffit. Kesh se déploie à **une
instance par base**, et rien ne protège deux processus l'un de l'autre (angle mort écrit, manuel). L'instantané de la base (Story 28-3) et l'immuabilité des fichiers nommés par
leur contenu (P-3) garantissent une archive cohérente même pendant une saisie.

**AC28 — Écrire la sauvegarde.** `run_once` crée `KESH_ADMIN_BACKUP_DIR/auto/` (`0700`, `DirBuilder` récursif comme
`write_pre_import_backup`), supprime d'abord les `kesh-auto-*.keshbackup.partial` qu'il y trouve (les siens, laissés par
un arrêt brutal — le verrou garantit qu'aucun n'est en cours **dans ce processus**, et Kesh tourne à une instance par
base, AC27), puis écrit
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
**Même nettoyage pour l'import** (F9, P4) : sous `backup_lock`, avant sa sauvegarde de sécurité, l'import supprime les
fichiers réguliers `kesh-pre-import-*.keshbackup.partial` de `KESH_ADMIN_BACKUP_DIR` (laissés par un arrêt brutal pendant
une sauvegarde pré-import, exclus de la liste de l'AC32 et sinon jamais supprimés — tolérable à quelques dizaines de
Mio, plus à « base + toutes les pièces ») ; un échec de suppression est journalisé et ne refuse pas l'import. Test
(AC36) : un `.partial` pré-import déposé disparaît après un import.

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
réguliers du premier niveau de `KESH_DOCUMENTS_DIR` (un `read_dir` et un `metadata()` par fichier, **dans le
`spawn_blocking`** de l'AC4, jamais sur un fil de l'exécuteur ; F10, P4 ; `store_document` n'écrit qu'à ce niveau) — majorant des pièces que l'archive emportera, connu **avant** la première sauvegarde et
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
`wrap_writer(file: File) -> Box<dyn WriteSeek>`, où `WriteSeek` est un trait du module, `pub trait WriteSeek: Write + Seek + Send {}` — **visibilité** : les tests de l'AC36 vivent dans un crate externe (`crates/kesh-api/tests/admin_auto_backup_e2e.rs`), donc le module `kesh_api::auto_backup` et ce qu'ils emploient sont **`pub`** : `BackupEnv`, `RealBackupEnv`, `WriteSeek`, `run_once`, `RunOutcome`, `Trigger`, le statut porté par `RunOutcome::Done` (`FailureStatus` et le type de statut de succès que l'AC28 nomme), le type d'erreur de `run_once`, et `spawn` — ce dernier parce que `main.rs` est un crate binaire distinct (sinon E0603, ou `private_interfaces` sous `-D warnings`) ; `classify_write_error` reste privée et se teste dans un `mod tests` du module avec une implémentation générale `impl<T: Write + Seek + Send> WriteSeek for T {}` — un objet-trait n'admet qu'**un** trait non automatique (E0225), d'où ce trait d'union (validation P5, R1) ; l'écrivain de test implémente `Write + Seek` — l'écrivain **possède** un **clone** du fichier
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
morte) se voit aussi (C-31-1-20). Ce second cas est le champ `overdue` de `GET backup-schedule`, calculé par le serveur
(`is_overdue`, AC25), qui le tient aussi pour vrai quand `last_success_at` est **postérieur** à l'heure du serveur
(horloge revenue en arrière, C-31-1-57) ; le frontend ne compare aucune date à l'horloge du navigateur. **`partial` n'allume pas le bandeau de l'application** (C-31-1-40, retrait de la P2) :
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
- `GET /api/v1/admin/backup-schedule` → réglage + état + `nextDueAt` (calculé par la même logique que `is_due` ;
  `now` si une valeur est dans le futur) + `overdue` (`is_overdue`, AC25, AC31 ; C-31-1-57) + `backupDir` **non exposé** (le chemin reste au journal, comme pour la sauvegarde pré-import) ;
- `PUT /api/v1/admin/backup-schedule` `{ enabled, intervalHours, retentionCount, version }` → 200 ; valeurs hors
  domaine → 400 `VALIDATION_ERROR` ; version périmée → 409 `OPTIMISTIC_LOCK_CONFLICT` (code existant) ; audit
  `admin.backup_schedule_updated` (`before`/`after`) dans la même transaction. **Seul `update_settings` incrémente
  `version`** : `record_attempt` et `record_*` ne la touchent pas (sinon une sauvegarde terminée entre le chargement du
  panneau et *Enregistrer* produirait un 409 sans cause visible) ; leurs horodatages sont liés **depuis l'application**
  (`Utc::now().naive_utc()`), jamais `NOW(3)` (le fuseau de session MariaDB fausserait `<=>` et `is_due`) ;
- `POST /api/v1/admin/backup-schedule/run` → exécute `run_once(…, Trigger::Manual)` **sans** la condition `is_due`,
  dans une tâche `tokio::spawn` attendue (comme l'import, AC8b : un client parti n'interrompt pas l'écriture), attend la
  fin, rend l'état ; `RunOutcome::Skipped` (verrou pris par une autre sauvegarde, que `run_once` a tenté lui-même,
  AC27) → 409 `BACKUP_IN_PROGRESS` (code neuf : variante `AppError::BackupInProgress` d'`errors.rs`,
  clé `error-backup-in-progress` en quatre locales). Le handler ne touche pas `backup_lock`. Le frontend l'appelle
  avec `LONG_OPERATION_TIMEOUT_MS`, sans rejeu (AC18b) ;
- `GET /api/v1/admin/backups` → `{ "importMaxMib": N, "items": [ … ] }` : `importMaxMib` est la taille d'import de
  l'installation (`config.admin_import_max_mib`, `KESH_ADMIN_IMPORT_MAX_MB`), **une fois** par réponse et non par
  ligne — c'est une propriété de l'installation (C-31-1-52, C-31-1-58 ; R1, P4 : aucune route ne transmet aujourd'hui ce
  plafond au frontend) ; `items` est la liste des fichiers **réguliers** (`symlink_metadata` : un lien n'est jamais listé)
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
discriminants passe de 4 à **6**, et l'assertion `discriminating >= 4` (l. 861) devient
**`assert_eq!(discriminating, 6)`** — un seuil, même relevé à 6, resterait vert si l'on oubliait le téléchargement dans
`GUARDED_BY_HANDLER` (7 ≥ 6) ; l'égalité le voit (R14, P4) (R5 de la P2, R12 de la P3 : la P2 faisait
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
l'exécution) ; liste **« Sauvegardes sur le serveur »** (nom, type, taille, date, *Télécharger*) ; toute ligne dont la
taille dépasse `importMaxMib × 1 048 576` octets (AC32) porte la mention **« trop grande pour être restaurée par
l'écran (limite N Mio) »** (clé `admin-backup-server-too-large`, quatre locales ; C-31-1-52) — comparaison stricte,
l'enveloppe multipart de l'envoi ajoutant quelques centaines d'octets que l'écran ne compte pas (une archive à
quelques octets du plafond peut être refusée sans mention : écrit au manuel). Rappel permanent : les
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
(fichier `0600`, dossier `0700`, archive importable — prouvée par la route `POST /api/v1/admin/full-import`, qui rend 200 et restaure les fichiers ; le module `admin_backup` reste `pub(crate)`, décision de la Story 17-3a : il manipule des secrets) ; rétention (8
sauvegardes déposées + une neuve, `retention_count = 3` → restent les 3 plus récentes, un `kesh-pre-import-x` et un
`autre.keshbackup` intacts) ; espace insuffisant (seuil injecté par paramètre de test : la fonction de mesure est
passée à `run_once` derrière le trait `BackupEnv`, AC30, C-31-1-19) → `insufficient_space`, aucun fichier, **8
sauvegardes déposées restent 8** (R5, P4 : un échec ne supprime rien) ;
**première sauvegarde sans historique** (`last_success_bytes` nul, dossier des documents de test portant des fichiers
dont la somme dépasse l'espace libre injecté) → `insufficient_space`, aucun fichier (C-31-1-42, tue M-c19) ; deux
`run_once` successifs à des `now` distincts d'une milliseconde → deux fichiers, deux noms (C-31-1-44) ; écriture
impossible (dossier `auto` remplacé par un fichier) → `failed` (aucune sauvegarde n'y est déposable : ce test ne
prouve pas « aucune suppression », les trois suivants si) ; **disque saturé pendant
l'écriture** (écrivain de test `StorageFull` après N octets, `retention_count = 3`) → `disk_full`, `.partial` supprimé,
aucune suppression de rétention (8 sauvegardes déposées restent 8 — sous `retention_count = 3`, la rétention en
garderait 3 ; R5, P4) ; `classify_write_error` unitaire (`StorageFull`, `QuotaExceeded`, code
brut 28, `PermissionDenied` → `failed`) ; **sauvegarde incomplète** (une pièce référencée retirée du dossier des
documents) → `partial`, `last_complete_file` inchangé, et sur une suite de `retention_count + 2` sauvegardes `partial`
la dernière complète **reste** ; une sauvegarde complète suivante la remplace dans `last_complete_file` et la rétention
retrouve son compte ; archive relue (AC28) : un écrivain de test qui tronque l'archive → `failed`, fichier supprimé,
8 sauvegardes déposées restent 8 (`retention_count = 3` ; R5, P4) ;
`version` inchangée par `record_attempt`/`record_*` (un `PUT` avec la version lue avant le tour → 200) ;
`.partial` résiduel supprimé ; `.partial` pré-import résiduel supprimé par un import (AC28, F9, P4) ; garde lexicale du garde possédé (AC27 : aucune forme empruntée de `backup_lock`,
`try_lock_owned` dans `auto_backup.rs` seulement, `lock_owned` dans la fermeture du `spawn` de `full_import`, garde de
`full_export` nommé dans la fermeture passée à `build_keshbackup` ; tue M-c18 et M-c24) ; garde lexicale du lancement
(AC26, lignes `//` écartées ; tue M-c21) ; liste et téléchargement : un lien symbolique
`kesh-auto-…keshbackup` vers un autre fichier n'est ni listé ni servi (tue M-c8) ; `GET backups` rend `importMaxMib`
égal à `admin_import_max_mib` de la `Config` de test (valeur non par défaut, par exemple 7) (C-31-1-52, R1, P4) ;
`GET backup-schedule` rend `overdue` vrai pour une `last_success_at` dans le futur (C-31-1-57) ; routes : 403 non-admin, 403 clé d'API, 400
domaine, 409 version, téléchargement 404 sur `../x`, sur un nom hors expression et sur un nom absent ; **audits et
état** (R12, P3) : un `PUT` écrit `admin.backup_schedule_updated` avec `before` et `after` exacts, et un `PUT` refusé
(400) n'en écrit aucun ; un téléchargement écrit `admin.backup_downloaded` (nom, taille) ; `nextDueAt` de
`GET backup-schedule` égale `last_success_at + interval` après un succès, et l'heure de la tentative + 1 h après
l'échec **d'un tour planifié** (qui ne part qu'une fois `last_success_at + interval` atteint) (la logique d'`is_due` : après l'échec
d'un « Sauvegarder maintenant » peu après une réussite, c'est `last_success_at + interval` qui l'emporte ; R14, P4).

**AC37 — Vitest.** Panneau : rendu de l'état (ok, `partial`, échec de chaque type, jamais tenté, désactivé),
enregistrement (payload exact, conflit 409 → rechargement proposé), « Sauvegarder maintenant » (désactivé pendant, 409
affiché, appel avec `LONG_OPERATION_TIMEOUT_MS` sans rejeu), liste et téléchargement, `partial` en avertissement
(`role="status"`, aucun `role="alert"`), `last_complete_file` absent de la liste → « introuvable » ; mention « trop
grande pour être restaurée par l'écran (limite N Mio) » sur une ligne dont la taille dépasse `importMaxMib × 1 048 576`,
absente sur une ligne égale à cette valeur et sur une ligne plus petite (C-31-1-52, R1, P4 ; tue M-c23) ; bandeau du
layout (`role="status"`) : affiché pour Admin seulement, dans les deux cas de l'AC31 (échec ; `overdue` vrai rendu par
le serveur, « aucune réussite depuis … »), absent quand `overdue` est faux et `last_status` vaut `ok` — le composant ne
compare aucune date à l'horloge du navigateur (C-31-1-57), **absent quand `partial`**, **absent quand la sauvegarde est désactivée** même si `last_status` est un
échec (tue M-c20), absent quand `last_success_at` est nul et `last_status` nul, absent quand la lecture échoue.

**AC38 — E2E (`frontend/tests/e2e/backup-schedule.spec.ts`).** Admin : changer la fréquence et la rétention,
enregistrer, recharger la page → valeurs gardées ; « Sauvegarder maintenant » → état « réussie » et une ligne `auto` dans
la liste ; télécharger cette ligne → fichier non vide dont le manifeste a `formatVersion: 2` ; réglages remis aux défauts
en fin de spec (la table survit aux `seedTestState`). Comptable : le menu n'offre pas la page, l'URL directe
**redirige vers `/`** (garde de route existante, `routes/(app)/admin/backup/+page.ts:15`, `redirect(302, '/')`) — le
refus 403 de l'API est vérifié côté Rust (AC36).

**AC39 — Manuel et CHANGELOG.** `admin-manual.tex` : sous-section **« La sauvegarde automatique »** (activée par défaut
à la mise à jour, fréquence, rétention et protection de la dernière sauvegarde complète, statut « incomplète »,
emplacement `./backup/auto`, espace requis et refus, volume (base + documents par copie), état et bandeau, *Sauvegarder
maintenant*, téléchargement, la mention « trop grande pour être restaurée par l'écran » et ce qu'elle demande (relever
`KESH_ADMIN_IMPORT_MAX_MB`, et le proxy inverse avec lui ; une archive à quelques octets du plafond peut être refusée
sans mention) (C-31-1-52), horloge déréglée **dans les deux sens** (vers le futur : un nom daté du futur survit à la
rétention ; en arrière : la sauvegarde repart aussitôt et le bandeau s'allume jusqu'à la réussite suivante,
C-31-1-57), une seule instance de Kesh par base (rien ne protège deux processus l'un de l'autre, C-31-1-54), ce que la
relecture d'une archive garantit et ne garantit pas (AC28),
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
sauvegarde réussie depuis … » ; (q) la section à largeur étroite (390 px) ; (r) une ligne de la liste portant la
mention « trop grande pour être restaurée par l'écran (limite N Mio) » (C-31-1-52, P4). Les états qu'on ne peut atteindre par
l'écran se posent ainsi, ce que le Dev Agent Record dit capture par capture (R8, P3) : les **statuts** (d), (e), (f),
(g), (n), (p) en base de test par `UPDATE` avant la capture ; (h) et (i), états d'un **verrou en mémoire** qu'aucun
`UPDATE` ne pose et qu'une sauvegarde de quelques Kio ne tient pas assez longtemps, **par interception réseau**
(`page.route` sur `POST …/run` : réponse retardée pour (h), réponse 409 `BACKUP_IN_PROGRESS` pour (i)). Depuis la P4
(R4) : (a) « jamais tentée » par `UPDATE` remettant `last_attempt_at`, `last_success_at` et `last_status` à `NULL` (la
tâche de fond écrit 5 minutes après le démarrage) ; (j) conflit de version et (k) erreur de validation par
interception de `PUT backup-schedule` (réponses 409 `OPTIMISTIC_LOCK_CONFLICT` et 400 `VALIDATION_ERROR` : les champs
de l'écran ne produisent pas de valeur hors domaine) ; (r) mention « trop grande pour être restaurée par l'écran »
(C-31-1-52) par interception de `GET backups` (taille d'une ligne portée au-delà de `importMaxMib` × 1 Mio : une
sauvegarde E2E pèse quelques Kio, et `importMaxMib` ne descend pas sous 1).

## Tasks / Subtasks

### 31-1a

- [x] T-a0 **Avant tout changement de code**, sur `44ef2079` : générateur `#[ignore]` et fixture
  `tests/fixtures/keshbackup-0.14.0-format1.keshbackup` (AC10), commit dédié, empreinte au Dev Agent Record.
- [x] T-a1 Tests rouges : AC2 (deux gardes), AC6/AC7 (unitaires de `parse_and_verify` : format 2 valide, ligne du
  manifeste sans entrée, nom invalide, préfixe ≠ sha, taille dépassée, sous-dossier, format 3 refusé, chemin dans les
  deux listes, chemin de `filesNotIncluded` hors forme **accepté**), AC6b (manifeste trop grand, tables trop grandes **en
  cumul** — deux tables chacune sous la borne, leur somme au-dessus —, `byteSize` supérieur au plafond d'import → 400
  sans panique, entrée inattendue — dont une entrée `files/*` sans ligne — refusée en format 2 avant la lecture de
  toute table ; bornes réduites par `ReadLimits` ; « nom en double » retiré en P2, C-31-1-38), AC10 (fixture réelle),
  AC13 (intégration).
- [x] T-a2 `kesh-db` : `ReferencedFile`, lecture dans `export_installation_in` (AC1) ; tests de l'instantané étendus
  (une facture figée **après** l'ouverture de l'instantané n'apparaît pas dans `referenced_files` — observateur) ;
  les deux lectures sans `UNION` ni `DISTINCT` SQL, dédoublonnage en Rust par égalité d'octets ; test : `X….pdf` et
  `x….pdf` de même empreinte déclarée (deux sociétés) → deux chemins, `x….pdf` inclus et `X….pdf` en `mismatch`
  (préfixe hors `[0-9a-f]`), aller-retour **200** (F6, P4 ; tue M-a22 : sous un `UNION` SQL, une ligne disparaît et
  l'import refuse en 400).
- [x] T-a3 `manifest.rs` : version 2, `files`, `filesNotIncluded` (`serde(default)`).
- [x] T-a4 `export.rs` : `build_keshbackup(pool, documents_dir, remplir)` générique, seul appelant
  d'`export_installation(` (AC4, C-31-1-56) ; `write_keshbackup<W: Write + Seek>` synchrone, appelée par chaque
  `remplir` ; lecture des fichiers en **une passe**, boucle de copie à la main, `abort_file` sur `altered` et
  `unreadable` (AC3, C-31-1-53) ; `regex` aux `[dependencies]` (AC6) ; méta étendue
  (AC5) ; `routes/admin.rs` : `SpooledTempFile` (AC4), en-têtes (AC5, AC12), audit.
- [x] T-a4b Tests de l'export (≥ 11, rouges d'abord) : fichier `missing`, `altered`, `mismatch` (préfixe ≠ empreinte de la
  ligne ; même chemin, deux empreintes) → listé dans `filesNotIncluded`, absent de l'archive (c'est le test qui tue M-a2) ;
  fichier **illisible** (`chmod 000` dans le dossier des documents du test ; le test vérifie d'abord qu'il ne peut pas le
  lire et **échoue bruyamment** s'il le peut — gate en `root` —) → `unreadable`, l'export rend **200** et les autres
  pièces sont incluses (tue M-a20) ; **dossier à la place du fichier** (s'ouvre, échoue à la lecture, `EISDIR`, donc
  **après** `start_file`) suivi d'un fichier sain → `unreadable`, export 200, l'archive se relit et le fichier sain y
  est (tue M-a21) ; **fichier altéré suivi d'un fichier sain** (entrée abandonnée au milieu par `abort_file`) →
  `altered`, l'entrée altérée absente, et l'archive **s'importe** (aller-retour 200, le fichier sain restauré : établit
  au dev la prémisse de C-31-1-53, F2, P4) ;
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
- [x] T-a5 `document_storage.rs` : cœur d'écriture atomique factorisé ; `restore_document(dir, storage_path, source:
  impl Read, byte_size, expected_sha)` — copie en flux à travers `take(byte_size.saturating_add(1))` et `Sha256` (AC8 ; la signature
  à `bytes` de la création lisait le fichier entier en mémoire, F4, P2), puis `sync_all` du dossier (AC8) ;
  `open_document` et son test unitaire direct (AC3 ; tue M-a12) ; `write_backup_file` **synchrone**, à fermeture de
  remplissage sur un clone du `File` (AC4) — ses tests (`routes/admin.rs:804`, `:830`) réécrits en synchrone.
- [x] T-a6 `import.rs` : `parse_and_verify<R: Read + Seek>`, `ReadLimits`, `ParsedBackup` portant la `ZipArchive<R>`,
  format 2 (AC6, AC6b, AC7, AC8) ; `routes/admin.rs` : `read_upload` sur disque
  (AC11), écriture des fichiers avant le `DELETE` (AC8), import dans une tâche `spawn` attendue (AC8b), comptes et
  audit (AC9).
- [x] T-a6b Tests de l'import (≥ 9, rouges d'abord) : cohérence AC7 en format 2 (fichier de `manifest.files` non
  référencé ; empreinte ≠ `file_hash`/`pdf_sha256` ; fichier référencé ni dans `files` ni dans `filesNotIncluded` →
  400) ; la même absence en format 1 → **200** (fixture réelle) ; table `invoices` sans colonne `pdf_storage_path` →
  ni `panic` ni 400, `filesMissing` calculé sans elle ; refus d'écriture (dossier des documents non inscriptible) →
  500 et base intacte ; fichier présent altéré → remplacé ; entrée `files/` plus grande que son `byteSize`, **dont le
  `sha256` déclaré est celui du contenu entier** (seule la borne la refuse ; sans borne, l'empreinte concorderait :
  c'est ce qui tue M-a6 ; R12, P3) → 400 « trop grande » ; dossier de sauvegarde absent → créé, import 200 ; dossier de
  sauvegarde existant **non inscriptible** (`0500`, même garde « échoue bruyamment en `root` ») → 500
  `AdminPreImportBackupFailed`, base intacte (R12, P3) ; garde lexicale de l'AC8b ; **format 1 restauré sur la même
  installation** (dossier des documents portant encore les fichiers) → `filesMissing = 0` (AC9, AC13 ; tue M-a9 ; R2,
  P4). (L'échec du `sync_all` du dossier des documents, AC8, n'est pas testé : aucun système de fichiers du poste ne le
  refuse ; branche relue en revue, angle mort écrit — F7, P4.)
  (« Aucun temporaire laissé » retiré en P2 : vrai par construction, aucun test ne pouvait rougir — AC11.)
- [x] T-a7 `config.rs` : défaut 2048 (AC12) et son test ; `.env.example`.
- [x] T-a8 Manuels, `docs/api-external.md`, proxy inverse, CHANGELOG (AC14) ; grep des valeurs (`503`, `512`,
  `files/`, `n'emporte`, `sans les fichiers`, `aucun binaire`, `formatVersion`, `50M`, `300s`) sur tout le dépôt — code,
  doc-comments, i18n, manuels (`.tex` **et** PDF aplati), `docs/`, `website/` — confronté à l'inventaire clos de l'AC14 :
  chaque site de l'inventaire traité, chaque site **neuf** ajouté au tableau du Dev Agent Record avec son traitement
  (les phrases de `website/roadmap.html` et de la brochure sont laissées à la release, à noter au Dev Agent Record) ;
  **et** la commande des sujets de l'AC14 rejouée (`-i`, compose compris), chacune de ses lignes triée (103 à
  `6f0828cf` ; 94 à `c7639e80` par la commande de la P3).
- [x] T-a9 Mutations (§ Mutations), gate complet (`kesh-db` touché), E2E complet au dernier commit de code.

### 31-1b

- [x] T-b1 Vitest rouges : lien de l'assistant (Admin seulement, étapes 0–2), avis `.env`, avis de l'export (trois cas
  d'en-têtes), avis de fin d'import (`filesMissing` 0 / > 0 ; bouton *Se reconnecter* dans **les deux** cas, aucune
  redirection avant le clic), garde de rôle de `/onboarding/restore`.
- [x] T-b2 `routes/onboarding/restore/+page.svelte` et `+page.ts` (garde de rôle), lien sur `+page.svelte` (AC15).
- [x] T-b3 `admin-backup.api.ts` (en-têtes), `AdminBackupPanel`, `AdminRestorePanel` (AC17-AC19 ; toast
  `admin-restore-toast-success` retiré, clé supprimée des quatre locales).
- [x] T-b4 i18n ×4 (AC22).
- [x] T-b5 E2E AC20 ; captures (AC21).
- [x] T-b6 Manuel (AC21), PDF.
- [x] T-b7 `api-client.ts` : options `{ timeoutMs, retry }` de `getBlob` et `postFormData`,
  `LONG_OPERATION_TIMEOUT_MS` ; `downloadFullExport`, `uploadFullImport` ; message d'issue inconnue de l'import pour
  `TIMEOUT`, `NETWORK_ERROR`, 502, 504 (i18n ×4) ; tests Vitest rouges d'abord (AC18b).

### 31-1c

- [ ] T-c1 Migration, réinjection de la ligne singleton par `scripts/regen-test-schema.sh` et garde
  `squash_seeds_the_backup_schedule_row`, squash régénéré, audit P5 (trois sites), P6, P7, inventaire (AC23, AC24).
- [ ] T-c2 `kesh-db` : dépôt `backup_schedule` (`get`, `update_settings` avec version, `record_attempt` sans condition,
  `record_*`).
- [ ] T-c3 `kesh-api/src/auto_backup.rs` : `is_due` (horloge en arrière), `is_overdue` (AC25, C-31-1-57), `run_once` (relecture de l'archive, statut `partial`,
  `last_complete_file`, noms à la milliseconde, seuil `documents_bytes`, verrou pris en `try_lock_owned` et
  `RunOutcome::Skipped`), `spawn`, `BackupEnv` (`Arc<dyn BackupEnv>`, `wrap_writer` qui prend un clone du `File` par
  valeur), `classify_write_error` (AC25-AC30) ; `AppState.backup_lock` aux neuf sites, gardes possédés et leur garde
  lexicale (AC27) ; branchement dans `main.rs` et sa garde lexicale (AC26).
- [ ] T-c4 Routes et registres (AC32 : `audit_route_registry`, `audit_label_registry`, `admin_pat_denied_e2e`
  — `ADMIN_COUPLES`, 33 constructeurs, `(8, 25)`, le seul `GET` de téléchargement dans `GUARDED_BY_HANDLER`,
  `assert_eq!(discriminating, 6)`) ; `importMaxMib` dans `GET backups`, `overdue` dans `GET backup-schedule` ;
  `AppError::BackupInProgress` ; libellés d'audit ×4 ; nettoyage des `.partial` pré-import (AC28).
- [ ] T-c5 Frontend : panneau (AC33, dont la mention « trop grande pour être restaurée par l'écran »), bandeau (AC31,
  lu sur `overdue`), API ; Vitest (AC37) ; i18n ×4 de toutes les chaînes de la 31-1c
  (règle de l'AC22, AC33).
- [ ] T-c6 Tests d'intégration (AC35, AC36), E2E (AC38), captures (AC40).
- [ ] T-c7 Manuel, CHANGELOG, README (AC34, AC39).

## Dev Notes

### Tests (récapitulatif, rouges d'abord)

| Sous-story | Rust | Vitest | E2E |
|---|---|---|---|
| 31-1a | gardes AC2 (2), `parse_and_verify` format 2 et lectures bornées (≥ 12, T-a1), instantané et lectures de l'AC1 (2, T-a2), `open_document` direct (1), export (≥ 11, T-a4b), import et cohérence (≥ 9, T-a6b, dont la garde lexicale de l'AC8b), intégration AC13 (3), AC10 sur la fixture réelle (1) | — | aucune spec neuve (l'écran ne change pas) ; suite complète au dernier commit de code |
| 31-1b | — | ≥ 16 (dont ≥ 5 pour `api-client` et ≥ 4 pour le message d'issue inconnue — `TIMEOUT`, `NETWORK_ERROR`, 502, 504 —, AC18b ; garde de rôle de `/onboarding/restore`) | `backup-restore-new-install.spec.ts` |
| 31-1c | migration et squash (AC23, AC24), `is_due` (≥ 8, dont deux d'horloge en arrière), `is_overdue` (≥ 5), `classify_write_error` (≥ 4), garde lexicale des gardes possédés (AC27), garde lexicale du lancement par `main.rs` (AC26), intégration AC35/AC36 | ≥ 14 | `backup-schedule.spec.ts` |

Le délai du client (AC18b) n'est **pas** prouvé par un E2E : les fixtures E2E pèsent quelques Ko et s'exportent en
moins d'une seconde, si bien qu'un E2E vert ne dirait rien du délai. Il l'est par les tests Vitest de `api-client` à
horloge simulée, qui figent les valeurs (30 s par défaut, une heure pour les appels longs, aucun rejeu).

### Gardes existantes qui changent de sens ou rougiront

- `import.rs` tests `parse_rejects_future_format_version` (forge `Some(2)` → désormais **valide** : passer à 3) et
  `parse_rejects_non_empty_files_dir` (vaut encore pour le format 1 : forcer `formatVersion: 1`).
- `manifest.rs` `manifest_json_has_canonical_shape` (`formatVersion` 1 → 2, `files` présent).
- `export.rs` `build_keshbackup_ne_lit_la_base_qu_une_fois` : `build_keshbackup` garde son nom et reste l'unique
  lecteur de la base (AC4, C-31-1-56) ; la garde reste exacte sur son corps et gagne l'assertion « `export_installation(`
  une seule fois dans la production de `kesh-api` » (R7, P4 : la P3 disait « corps renommé » sans dire quelle fonction
  restait l'unique point d'entrée, et la garde pouvait veiller sur du code mort). Son motif d'ouverture
  (`starts_with("pub async fn build_keshbackup(")`) ne reconnaît pas une signature générique (`build_keshbackup<T, F>(`) :
  il passe à `pub async fn build_keshbackup` sans parenthèse, et le compte de `pool` se recompte sur la signature réelle
  (R3 de la P5, appliqué au développement).
- `routes/admin.rs` garde `avant_sauvegarde_branchee_aux_appels_de_l_import` (l. 881 ; motifs `check_schema_compat(` et
  `build_keshbackup(`, compte `(1, 2)`, l. 911-915) — les deux appelants de `routes/admin.rs` appellent toujours
  `build_keshbackup(` (AC4), le compte reste `(1, 2)` ; mais elle découpe une instruction jusqu'au `;` suivant, et la
  fermeture `remplir` passée en argument en contient : écrite en ligne, elle couperait l'instruction **avant**
  `.map_err(avant_sauvegarde)` et la garde **rougirait** à tort. La fermeture se nomme donc avant l'appel (`let remplir
  = move |…| …;`), ou la garde se réécrit en gardant l'invariant (tout appel de l'import converti) ; jamais en baissant
  le compte (R6, P3 ; R7, P4).
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
  les discriminants (4 → 6 ; AC32, R5 de la P2, R12 de la P3) ; le seuil `>= 4` (l. 861) devient
  `assert_eq!(discriminating, 6)` (R14, P4).
- `scripts/regen-test-schema.sh` et `tests/test_schema_guard.rs` (ligne singleton, 31-1c).
- `crates/kesh-db/tests/migrations_upgrade_path.rs` `upgrade_path_preserves_data` (l. 120, `total, 82`) — 31-1c :
  **rougit** (82 → 83) ; bumper `total` **et** `N` du même pas (48 → 49, frontière 34 inchangée) — ne bumper que `total`
  rendrait le test **muet** (fenêtre plus étroite d'une migration, le fichier le dit l. 142-144) (R6, P4).
- `crates/kesh-db/src/post_restore.rs` `EXEMPT_MIGRATIONS.len() == 19` (l. 1217-1218) — 31-1c : **rougit** à l'entrée
  neuve (AC23) ; 19 → 20, l'entrée déclare son fondement **`Durable`** (patron de `_kesh_version`, commentaire « 18 → 19 »
  l. 1214-1215) ; le compte « 8 exemptions Hors fenêtre » (l. 1122) reste 8 (R6, P4).
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
| M-a9 | AC9 : `filesMissing` calculé sur le backup au lieu de la destination | T-a6b / AC13 format 1 restauré **sur la même installation** (`filesMissing = 0` ; sous la mutation, 2) — reciblé en P4 (R2) : « AC13 format 1 (`filesMissing = 2`) » donnait le même nombre sous la mutation |
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
| M-a21 | AC3 : erreur de **lecture** pendant la copie traitée comme une erreur d'écriture (sauvegarde abandonnée) au lieu d'`abort_file` + `unreadable` | T-a4b dossier à la place du fichier, suivi d'un fichier sain → 200, `unreadable` (P4, C-31-1-53) |
| M-a22 | AC1 : `UNION` SQL au lieu de deux lectures dédoublonnées en Rust | T-a2 `X….pdf` / `x….pdf` → aller-retour 200 (P4, F6) |
| M-b1 | AC15 : lien visible pour un non-Admin | Vitest du lien |
| M-b2 | AC19 : déconnexion immédiate après un import réussi (reformulée en P2 : *Se reconnecter* vaut pour tout succès) | Vitest *Se reconnecter* (`filesMissing = 0` et `> 0`) |
| M-b3 | AC18 : ignorer l'en-tête `Not-Included` | Vitest avis export |
| M-b4 | AC18b : `downloadFullExport` sans l'option longue | Vitest (espion : option passée ; abandon à 30 s) |
| M-b5 | AC18b : `retry: false` ignoré par `fetchWithRetry` | Vitest « un seul `fetch` sur `TIMEOUT` » |
| M-b6 | AC18b : `TIMEOUT` de l'import → message par défaut (« Le serveur ne répond pas. Réessayez ultérieurement. », le comportement actuel ; reformulée en P2) | Vitest du panneau |
| M-b7 | AC18b : 504 de l'import → message par défaut (« Erreur serveur (504) ») | Vitest du panneau, 504 |
| M-c1 | AC25 : `≥` → `>` sur l'intervalle | `is_due` à l'égalité |
| M-c2 | AC25 : retirer la clause `last_attempt_at` | `is_due` « échec il y a 10 min » |
| M-c3 | AC29 : rétention sur `kesh-*` au lieu du motif exact | rétention : « restent les 3 plus récentes » `auto` (sous la mutation, `kesh-pre-import-x`, trié le plus récent par nom — `kesh-p` > `kesh-a` —, prend une des trois places et survit ; reciblé en P4, R5) |
| M-c4 | AC29 : rétention aussi après échec | AC36 `insufficient_space`, archive tronquée (`failed`) et `disk_full` : 8 sauvegardes déposées restent 8 sous `retention_count = 3` (reciblé en P4, R5 : dans le test « écriture impossible », aucune sauvegarde n'est déposable) |
| M-c5 | AC30 : seuil d'espace ignoré | `insufficient_space` |
| M-c6 | AC27 : `run_once` en `lock_owned().await` au lieu de `try_lock_owned` | AC35 (`run_once` sous `tokio::time::timeout` de 5 s : échoue au lieu de pendre) |
| M-c8 | AC32 : téléchargement par concaténation du nom | AC36 lien symbolique `kesh-auto-…keshbackup` → non servi (reciblé en P4, R5 : `../x` échoue à l'expression du `kind` avant toute concaténation) |
| M-c9 | AC25/AC31 : `is_overdue` sans la condition « deux intervalles » (vrai dès un intervalle) | `is_overdue` aux bornes (deux intervalles moins une milliseconde → faux) — reciblé en P4 : le calcul est passé au serveur (C-31-1-57) |
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
| M-c21 | AC26 : retirer `auto_backup::spawn(state.clone())` de `main.rs`, **ou le mettre en commentaire** | garde lexicale du lancement (une occurrence hors lignes `//` dans `main.rs`, aucune dans `lib.rs`) |
| M-c22 | AC25 : retirer la clause « `last_*` postérieur à `now` vaut nul » d'`is_due`, ou « `last_success_at > now` » d'`is_overdue` | `is_due` « dernière réussite / tentative dans le futur → dû » ; `is_overdue` « dans le futur → vrai » (P4, F4) |
| M-c23 | AC33 : mention « trop grande » sans la comparaison à `importMaxMib` (toujours, jamais, ou `≥` au lieu de `>`) | Vitest : ligne au-dessus → mention ; ligne égale et ligne au-dessous → aucune (P4, R1, C-31-1-52) |
| M-c24 | AC27 : garde de l'export manuel tenu par le handler, hors de la fermeture passée à `build_keshbackup` | garde lexicale des gardes possédés (AC27, AC36) (P4, F1) |

**Retirée en P4** : M-c7 (« réservation sans `<=> ?` ») — la réservation conditionnelle elle-même est retirée
(C-31-1-54) ; le numéro n'est pas réattribué.

Une mutation équivalente se **déclare** (pourquoi aucun test ne peut la tuer), elle ne se tait pas. Aucune n'est
déclarée équivalente depuis la P4 : le re-hachage de la seconde passe, déclaré équivalent en P3, a disparu avec la
seconde passe (C-31-1-53) ; le contrôle d'empreinte de la passe unique est tué par T-a4b « altéré non inclus » (M-a2).

### Angles morts assumés (à écrire aux doc-comments et au manuel)

- Les fichiers du dossier des documents que la base ne référence plus (PDF orphelins de deux rendus concurrents,
  `issued_invoice_pdf.rs:21-23`) ne sont ni sauvegardés ni restaurés.
- Archive de plus de 4 Gio : non mesurée (`zip` 2.4.2 écrit le ZIP64 par entrée sur option `large_file` ; les entrées
  sont petites, le volume total ne l'est peut-être pas) — test non fait, écrit au manuel comme limite (C-31-1-9).
- La sauvegarde automatique ne tourne que si Kesh tourne ; une tâche arrêtée sans échec se voit par le bandeau « aucune
  réussite depuis deux intervalles », pas autrement.
- Deux processus sur une même base : **rien** ne les protège l'un de l'autre (P4, C-31-1-54 : la réservation
  conditionnelle de la P2, qui disait l'éviter, ne fermait qu'une fenêtre de quelques millisecondes et est retirée).
  Deux sauvegardes automatiques peuvent alors s'écrire en même temps, le nettoyage des `.partial` de l'un peut supprimer
  le `.partial` en cours de l'autre (sauvegarde `failed`), et le verrou en mémoire ne protège pas un import de l'un
  contre la sauvegarde de l'autre (l'instantané reste cohérent). Kesh se déploie à une instance par base (manuel).
- L'espace libre est mesuré **avant** ; un autre programme peut remplir le disque pendant l'écriture (cas `disk_full`).
- Pas d'arrêt gracieux : un arrêt du conteneur pendant une sauvegarde laisse un `.partial`, nettoyé au tour suivant.
- Horloge du serveur déréglée vers le futur : un nom `kesh-auto-…` daté du futur passe pour le plus récent et survit
  aux suivants (la rétention trie par nom) ; inoffensif pour les données, coûteux en disque — écrit au manuel. (Vers le
  passé, l'horloge revenue en arrière n'est plus un angle mort : `is_due` et `is_overdue` la traitent, C-31-1-57.)
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
- *(P3, levé en P4)* Une sauvegarde du serveur plus grosse que `KESH_ADMIN_IMPORT_MAX_MB` **est** désormais signalée
  dans la liste (AC32 `importMaxMib`, AC33 ; C-31-1-52, qui amende C-31-1-51). Reste : la comparaison ne compte pas
  l'enveloppe multipart de l'envoi, si bien qu'une archive à quelques centaines d'octets sous le plafond peut être
  refusée sans mention (écrit au manuel).
- *(P3)* Le `JoinError` de la tâche d'import (AC8b), la cadence de la boucle de fond et l'isolation d'une panique dans
  un tour (AC26) ne sont pas testés.
- *(P4)* L'échec de la synchronisation du dossier des documents (AC8, montages qui refusent `fsync` sur un dossier)
  n'est pas testé : journalisé, il ne refuse pas l'import.
- *(P4)* Un client parti pendant l'assemblage d'un export manuel ne relâche plus le verrou (le garde vit dans le
  `spawn_blocking`, C-31-1-55) ; l'archive de cet assemblage orphelin, dans un temporaire anonyme, se perd — une à la
  fois. (L'angle mort *(P3)* « relâche le verrou » est levé.)

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

31-1a : Opus 5.5 (Claude Code), 2026-10-24, en autonomie.
31-1b : Opus 5.5 (Claude Code), 2026-10-24, en autonomie (worktree `kesh-31-1b`, branche `story/31-1b` sur `1ee0583e`).

### Debug Log References

- `/home/gcorbaz/devel/kesh-gate-logs/311a-fixture.log` — production de la fixture 0.14.0.
- `311a-red-kesh-db.log`, `311a-red-integration.log` — tests rouges constatés.
- `311a-cible-1.log` — premier gate ciblé `kesh-api` (lib + deux binaires d'intégration).
- `311a-gate-1.log` (rouge, `configuration_transmise`), `311a-gate-2.log`, `311a-gate-3.log` — gates complets.
- `311a-mut-M-a*.log` — campagne de mutations (diff de la mutation en tête de chaque journal).
- `311a-make.log` — PDF des manuels.
- `311a-e2e-run.log`, `311a-fe-*.log`, `311a-e2e.log`, `311a-e2e-backend.log` — gate frontend et E2E complet.

### Completion Notes List

**31-1a — T-a0, fixture 0.14.0 (C-31-1-59).** Produite par `generate_fixture_0_14_0` (`#[ignore]`,
`admin_full_export_e2e.rs`) dans un worktree temporaire au tag `v0.14.0` (`5a128aec`, Rust 1.96.1), avec
`CARGO_TARGET_DIR` du worktree de la story ; le worktree temporaire est retiré aussitôt. Résultat :
`crates/kesh-api/tests/fixtures/keshbackup-0.14.0-format1.keshbackup`, **12 490 octets**, SHA-256
**`68516ceb92183353166788242c471314ccf0e0de2a7eaa36dc9906b38aa7e006`**. Contrôlée par `unzip` : `formatVersion: 1`,
`keshVersion: 0.14.0`, `files/` vide, une ligne `imported_supplier_invoices` (`a6e62407….pdf`) et une facture figée
(`pdf_storage_path` `6a818379….pdf`) — deux pièces référencées, aucune emportée. Données synthétiques (société de
`seed_accounting_company`, « Client Fictif », IBAN d'exemple de la documentation SIX déjà employé par les tests).

**31-1a — ce qui est livré** (commits `33684111` fixture, `844d522e` code, `3586d3c5` documentation) :

- **kesh-db** (AC1) : `ReferencedFile`, `InstallationExport.referenced_files`, `read_referenced_files` — deux requêtes
  dans l'instantané, après les tables, dédoublonnage par égalité de chaînes (`BTreeSet`), aucun `UNION` (C-31-1-62).
- **document_storage** : `write_atomically` (cœur commun à `store_document`, comportement inchangé),
  `open_document` (seul point de jointure d'un chemin), `sha256_of`, `document_is_intact`, `restore_document` en flux
  (`Written` / `Replaced` / `AlreadyIntact`, C-31-1-65), `is_safe_storage_path` en `pub(crate)`.
- **manifest** : `BACKUP_FORMAT_VERSION = 2`, `files`, `filesNotIncluded` (`serde(default)`), `NotIncludedReason`.
- **export** : `build_keshbackup<T, F>(pool, documents_dir, remplir)` — unique lecteur de la base, un seul
  `spawn_blocking` ; `write_keshbackup<W: Write + Seek>` ; `copier_piece` en une passe avec `abort_file` ;
  `classer_les_pieces` ; `BackupWriteError` (l'`io::Error` conservée) et `BackupError` (C-31-1-60).
- **import** : `parse_and_verify<R: Read + Seek>(reader, &ReadLimits)` — manifeste borné, inventaire des noms avant toute
  lecture, borne cumulée des tables, pièces bornées et hachées, cohérence pièces ↔ base (format 2 seul), refus d'un
  format 1 qui déclarerait des pièces (C-31-1-64) ; `ParsedBackup<R>` porte l'archive ; `ecrire_les_pieces` (dossier
  synchronisé, échec journalisé sans refus) ; `compter_les_pieces_manquantes`.
- **routes/admin** : export dans un `SpooledTempFile` débordant dans `KESH_ADMIN_BACKUP_DIR` (créé au besoin), en-têtes
  `X-Kesh-Backup-Files`, `-Files-Not-Included`, `-Exceeds-Import-Limit`, audit `file_count` / `files_bytes` /
  `files_not_included` ; import reçu par morceaux dans un temporaire anonyme, vérification en `spawn_blocking`, tout le
  reste dans un `tokio::spawn` attendu, pièces écrites **avant** `restore_tables_in_tx`, `filesRestored` /
  `filesMissing` en réponse et à l'audit (avec `source_format_version`) ; `write_backup_file` synchrone à fermeture de
  remplissage ; `write_pre_import_backup(state)` porte la sauvegarde pré-import (C-31-1-66) ; `avant_sauvegarde`
  générique.
- **config** : `DEFAULT_ADMIN_IMPORT_MAX_MIB = 2048` (constante unique ; littéraux de test et messages la reprennent).
- **Dépendances** : `regex` et `tempfile` passés aux `[dependencies]` de `kesh-api` ; `Cargo.lock` inchangé.

**Tests rouges constatés avant le code** :

- `kesh-db` (`311a-red-kesh-db.log`) — lecture remplacée par `Vec::new()` le temps du constat :
  `referenced_files_lit_documents_et_pdf_figes` **FAIL** (`left: []`, `right: [ReferencedFile { … AAAA….pdf … }, …]`) ;
  `colonnes_storage_path_inventaire_clos` passe (garde de l'état présent, verte par nature).
- Intégration (`311a-red-integration.log`), sur le code d'avant (`manifest.rs` mis de côté par `git stash` le temps du
  run) : **73 tests, 50 passés, 23 échoués** — les 12 tests neufs d'export, l'assertion `formatVersion == 2` du test de
  structure, et 10 tests neufs d'import (`ac10…`, `ac13…` ×2, `format_1_restauree_sur_la_meme_installation`, les trois
  refus de cohérence, `piece_presente_alteree_remplacee`, `table_invoices_sans_colonne_de_pdf`,
  `refus_d_ecriture_des_pieces_laisse_la_base_intacte`). Verts dès l'origine, et c'est attendu : `import_cree_le_dossier…`
  et `import_dossier_de_sauvegarde_non_inscriptible` (l'ancien code créait déjà le dossier et refusait un dossier non
  inscriptible : ils gardent M-a17 et l'AC11), `full_import_refuses_unknown_format_400` porté à 3.
- Les tests unitaires de `parse_and_verify`, `document_storage`, `export` et `routes/admin` ont été écrits **avec** les
  nouvelles signatures (rouges de compilation, non consignés comme rouges de comportement) ; leur pouvoir de détection
  est établi par la campagne de mutations ci-dessous. ⚠️ Écart de méthode déclaré : pour `kesh-db` et `manifest.rs`, le
  code de production a été écrit avant les tests, le rouge a été constaté ensuite en défaisant le code le temps du run.

**Gates réellement exécutés** (périmètre : branche `story/31-1`, base `b5fd9fb1`) :

- Gate complet backend (`scripts/test-fast.sh --ci`, fmt + clippy `-D warnings` + nextest, base `kesh_311a` remise à
  zéro avant chaque run) : `311a-gate-1.log` **rouge** (1 échec : `configuration_transmise lectures`, entrée `TMPDIR`
  périmée — C-31-1-67) ; `311a-gate-2.log` **3907 passés, 7 ignorés** (au commit `844d522e`) ; `311a-gate-3.log`
  **3907 passés, 7 ignorés** (au commit `3586d3c5`, dernier commit de code : doc-comments d'`errors.rs` et
  `admin_backup/mod.rs`). Les 7 ignorés : les 6 de `main` et le générateur `generate_fixture_0_14_0`.
- Frontend au commit `3586d3c5` (le frontend n'est pas touché) : `npm run check` **0 erreur, 29 avertissements** (5081
  fichiers) ; `lint-i18n-ownership` **PASS** ; `npm run test:unit` **1820 tests, 156 fichiers, tous passés** ;
  `npm run build` **vert**.
- **E2E complet** au commit `3586d3c5` (recette de `docs/testing.md` à la lettre, port 3311, base `kesh_e2e_311a`
  remise à zéro, dossiers `/tmp/kesh-e2e-311a/*` passés au backend **et** au runner, `/health` :
  `smtpConfigured: true`) : **307 passés, 7 échoués, 17 ignorés** (12,5 min, `311a-e2e.log`). Les 7 échecs, jugés
  fichier par fichier contre `docs/testing.md` § *Les échecs attendus* : `mode-expert.spec.ts:26`, `:41`,
  `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150` — **les sept KF-029 (#97)** ; ni KF-052
  ni pollution sur ce run. Aucun échec hors liste.

**Décompte des tests** (recompté aux deux bornes, `#[test]` / `#[tokio::test]` / `#[sqlx::test]`, de `b5fd9fb1` à
`HEAD`) : `kesh-db/src/backup.rs` 25 → 28 ; `admin_backup/export.rs` 1 → 5 ; `admin_backup/import.rs` 6 → 23 ;
`admin_backup/manifest.rs` 1 → 3 ; `document_storage.rs` 7 → 14 ; `routes/admin.rs` 8 → 10 (deux tests renommés et
réécrits en synchrone : `write_backup_file_en_0600`, `ensure_backup_dir_dossiers_crees_et_existants`) ;
`tests/admin_full_export_e2e.rs` 7 → 20 (dont le générateur `#[ignore]`) ; `tests/admin_full_import_e2e.rs` 43 → 55 ;
soit **60** fonctions de test neuves. `config.rs` et `configuration_transmise.rs` : nombre inchangé, assertions mises à
jour.

**Mutations** (campagne sous le verrou de gate, journaux `311a-mut-<M>.log` ; chaque mutation appliquée par l'outil
d'édition sur un motif unique, restaurée par `git checkout -- <f>` puis `touch <f>`, arbre propre vérifié après chacune) :

| # | Mutation appliquée | Test(s) rejoué(s) | Résultat |
|---|---|---|---|
| M-a1 | boucle des PDF figés de `read_referenced_files` → `.take(0)` | `referenced_files_lit…`, `ac13_format_2…` | **tuée** (2/2 rouges) |
| M-a2 | `copier_piece` sans le contrôle d'empreinte | `export_piece_alteree…` | **tuée** |
| M-a3 | import sans « préfixe = empreinte » | `format_2_prefixe_different…` | **tuée** — par le message : l'archive est refusée plus loin (cohérence), le test exige le refus de nom |
| M-a4 | import sans le contrôle « ligne du manifeste sans entrée » | `format_2_ligne_du_manifeste_sans_entree` | **tuée** — par le message : la lecture de la pièce échoue ensuite (défense en profondeur) |
| M-a5 | import sans la comparaison d'empreinte d'une pièce | `format_2_empreinte_fausse` | **tuée** |
| M-a6 | borne `take(byteSize + 1)` → `take(u64::MAX)` | `format_2_entree_plus_grande…` | **équivalente** (C-31-1-63) : la comparaison des tailles refuse avec le même message ; seule la quantité lue change, que rien n'observe |
| M-a6b | borne **et** comparaisons de taille retirées | `format_2_entree_plus_grande…` | **tuée** (l'archive est acceptée) |
| M-a7 | pièces écrites **après le `COMMIT`** | `refus_d_ecriture_des_pieces…` | **tuée** — sur l'assertion « la base n'a pas été remplacée » |
| M-a8 | erreur d'écriture des pièces ignorée | `refus_d_ecriture_des_pieces…` | **tuée** |
| M-a9 | `filesMissing` calculé sur le backup | `format_1_restauree_sur_la_meme_installation`, `ac13_format_1…` | **tuée** par le premier ; le second reste vert (même nombre, comme la fiche l'annonçait) |
| M-a10 | `formatVersion` 3 accepté | `parse_rejects_future_format_version`, `full_import_refuses_unknown_format_400` | **tuée** (2/2) |
| M-a11 | fichiers référencés lus dans un second instantané, sur le pool | `referenced_files_lus_dans_l_instantane`, `export_installation_n_ouvre_qu_un_instantane` | **tuée** (2/3 rouges ; `referenced_files_lit…` reste vert) |
| M-a12 | `open_document` sans `is_safe_storage_path` | `open_document_refuse_les_chemins_hors_forme` | **tuée** |
| M-a13 | deux chemins de même empreinte classés `mismatch` | `classer_les_pieces_garde_deux…`, `export_deux_chemins…` | **tuée** (2/2) |
| M-a14 | borne des tables par table au lieu du cumul | `tables_trop_grandes_en_cumul` | **tuée** |
| M-a15 | cohérence appliquée au format 1 | `coherence_appliquee_au_seul_format_2`, `ac10_la_sauvegarde_de_la_0_14_0…` | **tuée** (2/2) |
| M-a16 | import exécuté sans `tokio::spawn` (même comportement) | `import_dans_une_tache_detachee`, `ac13_format_2…` | **tuée** par la garde lexicale (le test de comportement reste vert, attendu) |
| M-a17 | `full_import` ne crée plus le dossier de sauvegarde | `import_cree_le_dossier…` | **tuée** |
| M-a18 | expression des noms réappliquée à `filesNotIncluded` | `format_2_non_inclus_hors_forme_accepte`, `export_chemins_hors_forme…` | **tuée** (2/2) |
| M-a19 | `spooled_tempfile` sans `_in` | `export_deborde_dans_le_dossier_de_sauvegarde` | **tuée** |
| M-a20 | erreur d'ouverture propagée (sauvegarde abandonnée) | `export_piece_illisible…` | **tuée** |
| M-a21 | erreur de lecture pendant la copie traitée en erreur d'écriture | `export_dossier_a_la_place_de_la_piece` | **tuée** |
| M-a22 | `UNION` SQL au lieu de deux lectures | `referenced_files_lit…`, `export_chemins_de_casse…` | **tuée** (2/2) |

Bilan : 22 mutations de la fiche exercées (+ la variante M-a6b) ; **21 tuées, 1 déclarée équivalente** (M-a6, avec la
variante qui en tue l'intention).

**Choix ajoutés au registre** : C-31-1-59 (fixture au tag), C-31-1-60 à C-31-1-68.

**Inventaire de l'AC14** — la commande des sujets de la fiche (`-i`, compose compris) rejouée sur `HEAD` rend **123
lignes** (103 à `6f0828cf` ; l'écart vient des doc-comments neufs de la 31-1a qui nomment ces sujets) :
35 du manuel d'administration, 8 de `.env.example`, 2 du `README.md`, 4 de chaque compose, 70 de commentaires dans
`crates/`. Tri, ligne par ligne, regroupé par traitement :

| Lignes | Traitement |
|---|---|
| `admin-manual.tex:196` | « tout import et tout export » (jumeau de `.env.example:88`, `README.md:93`, tous trois traités) |
| `admin-manual.tex:345`, `:367`, `:395`, `:531`, `:659` | **neuves** : bloc Nginx `location /api/v1/admin/`, paragraphe du proxy, Caddy, Traefik, Portail DSM |
| `admin-manual.tex:805-807` (tableau), `:1991-1993` (§ *Variables*) | 2048, débordement dans le dossier de sauvegarde, temporaires |
| `admin-manual.tex:1489`, `:1851`, `:1861`, `:1914`, `:1933`, `:1935`, `:1942`, `:1946` | corrigées ou **neuves** (stratégie, introduction, instantané et pièces, *Ce que la sauvegarde emporte*, refus de l'import, *Les pièces à l'import*, volume de la sauvegarde pré-import) |
| `admin-manual.tex:866` (note des montages), `:1631` (Hyper Backup), `:1925` (import destructeur), `:1944` (titre), `:1950-1951` (copie SSH) | **intactes** : vraies après la 31-1a (le téléchargement par l'écran est la 31-1c, AC34) |
| `admin-manual.tex:2240`, `:2245` (compose 0.13.0), `:2314` (montage `./backup`) | **intactes** : vraies à leur date / pour leur objet |
| `admin-manual.tex:2283-2285`, `:2299-2301`, `docker-compose*.yml:131-133` / `:153-155` | **intactes** : transmission des variables (`${…:-}`) |
| `.env.example:88`, `:183`, `:185`, `:189`, `:190`, `:197` | traitées (export aussi ; débordement ; 2048 ; dossier des temporaires) |
| `.env.example:226`, `:261` (`./backup`, sauvegarde pré-import) | **intactes** : complétées à la 31-1c (`auto/`) |
| `docker-compose.yml:154`, `docker-compose.prod.yml:172` | traitées (une sauvegarde par import pèse la base **plus** les pièces ; temporaires) |
| `README.md:93` | traitée ; `README.md:225` (ligne de la v0.13.0 de la feuille de route) **intacte** : historique |
| `crates/` — doc-comments et commentaires écrits ou réécrits par la 31-1a (`admin_backup/{mod,export,import}.rs`, `routes/admin.rs`, `config.rs:243-266`, `:901-903`, `:934-935`, `:961-963`, `:1279`, `:1285-1286`, `errors.rs:481-504`) | **traités** |
| `config.rs:968`, `:1094`, `:1783`, `:1829` ; `errors.rs:2110` ; `lib.rs:253` ; `routes/admin.rs:825` ; `exports/global.rs:604` et `routes/reconciliation.rs:963` (« in-memory » sans rapport) | **intactes** : exactes ou sans objet |
| `kesh-db/src/backup.rs:642` (« compter les tables du manifeste (`build_keshbackup`) ») | **intacte, mais devenue inexacte** : c'est désormais `write_keshbackup` qui compte, et le renvoi `import.rs:129` est périmé. Commentaire d'un test de `kesh-db`, relevé après le dernier gate : laissé pour ne pas rouvrir le gate et l'E2E sur une ligne de commentaire ; **LOW à corriger au prochain commit de code** (revue P1) |

Grep des **valeurs** (`503`, `512`, `n'emporte`, `sans les fichiers`, `aucun binaire`, `formatVersion`, `50M`, `300s`)
sur `crates/`, `docs/`, `website/`, `frontend/src/`, `README.md`, `.env.example`, les compose et `CLAUDE.md` : plus
aucun « n'emporte pas » vivant ; restent, légitimes, les mentions historiques (« 512 avant », « défaut 512 jusqu'à la
0.14.x »), `VARCHAR(512)` d'une migration, la taille d'une image de test (`inbox_import_e2e.rs:269`), le `50M` / `300s`
du bloc Nginx général (voulus, le bloc d'administration les relève), `admin-manual.tex:2072` (passer à la 0.14.0, vrai
à sa date), `website/roadmap.html:368` (déjà au futur), et le paragraphe *Current release* du `CLAUDE.md` (« le
`.keshbackup` n'emporte pas … #503, ouverte ») — vrai pour la v0.14.0 publiée, à réécrire à la release 0.15.0. PDF
contrôlés aplatis (`pdftotext … | tr '\n' ' '`) : « Ce que la sauvegarde emporte », le bloc `location /api/v1/admin/`,
« défaut 2048 depuis la 0.15.0 » présents ; aucune référence non résolue (`??`) ; zéro « n'emporte pas » / « sans les
fichiers » dans `admin-manual.pdf`. `website/roadmap.html`, brochure : laissés à la release.

**Écarts à la fiche et aux consignes** :

- **Édition hors de l'outil Edit, une fois** : le premier lot de tests d'export a été ajouté à
  `tests/admin_full_export_e2e.rs` par `cat >> … <<'EOF'` (bloc reproduit ensuite et corrigé à l'outil Edit, dont une
  assertion mal formée que ce bloc portait). Tout le reste du code a été édité par Edit / Write ; `cargo fmt` a reformaté.
- AC8 : le message de refus d'écriture des pièces est celui d'`AdminFullImportFailed` (C-31-1-61), non un texte neuf.
- M-a6 déclarée équivalente (C-31-1-63) ; M-a3 et M-a4 tuées par le message, l'archive étant refusée plus loin.
- Rouges : constatés après coup, en défaisant le code, pour `kesh-db` et l'intégration ; les unitaires ont été écrits
  avec les signatures (cf. ci-dessus).
- La garde `echecs_avant_sauvegarde_tous_convertis` et le test 20 sont **étendus** à `write_pre_import_backup`
  (C-31-1-66), sans baisse de compte.
- `configuration_transmise.rs` : entrée `TMPDIR` retirée (C-31-1-67) — garde que la fiche ne listait pas.
- `full_import_refuses_when_backup_cannot_be_written` exporte par une seconde instance au dossier utilisable (l'export
  exige désormais lui aussi un dossier de sauvegarde inscriptible).

---

**31-1b — premiers commits : LOW confiés par la revue P1 de la 31-1a** (`dbf6206d`) : commentaire
`kesh-db/src/backup.rs:642` (A-L1) réécrit sans numéro de ligne ; manuel d'administration — refus propres au format 2
(entrée inattendue, `byteSize` au-delà du plafond, pièce plus grande que déclarée, nom hors forme, manifeste de format 1
qui déclare des pièces), bornes de lecture (64 Mio, 1 Gio cumulé), deux limites assumées (E-3 pièces écrites avant un
refus tardif ; E-4 verrou de `_kesh_version` tenu pendant la sauvegarde pré-import, second import en échec au-delà de
`innodb_lock_wait_timeout`), 410 d'une pièce non emportée (§ PDF figés) ; `docs/api-external.md` : le 410 des deux
lectures de document (B-L3). Les refus de l'import ne vont **pas** à `api-external.md`, qui ne documente pas les routes
d'administration, refusées aux clés d'API (C-31-1-74). L'avertissement de l'en-tête d'excès (A-L2) est désormais vrai à
l'écran, et le manuel le dit.

**31-1b — ce qui est livré** (commits `a4012e69` code, `e060e4c4` E2E, `0bb317c6` manuel, `7f977a04` et `7cc8a3e4`
gardes, `17c7a8d6` jetons de couleur, `08ef4bdd` garde des jetons) :

- **AC15** : lien « Restaurer une sauvegarde » sur `routes/onboarding/+page.svelte` (étapes ≤ 2, `Admin`, état chargé) ;
  page `routes/onboarding/restore/+page.svelte` (rend `AdminRestorePanel`, charge les messages, lien de retour) et sa
  garde de rôle `+page.ts` (non-Admin → `/onboarding`, C-31-1-72).
- **AC17** : `features/admin-backup/BackupEnvNotice.svelte`, rendu par les deux panneaux (C-31-1-69).
- **AC18** : `downloadFullExport` rend `{ filesNotIncluded, exceedsImportLimitMib }` lus strictement (C-31-1-73) ;
  `AdminBackupPanel` affiche les deux avis, effacés à chaque export.
- **AC18b** : `api-client.ts` — `CallOptions { timeoutMs, retry }`, `LONG_OPERATION_TIMEOUT_MS` (une heure), options
  conservées au rejeu après refresh ; `getBlob` / `postFormData` à défauts inchangés ; export et import en
  `{ timeoutMs: LONG_OPERATION_TIMEOUT_MS, retry: false }` ; `AdminRestorePanel` : `issueInconnue` (`TIMEOUT`,
  `NETWORK_ERROR`, 502, 504) → `admin-restore-error-unknown-outcome`, « Se reconnecter » proposé (C-31-1-71) ; 503 au
  défaut.
- **AC19** : succès → avis « Import réussi », pièces restaurées, avertissement des pièces manquantes ; aucune redirection
  avant « Se reconnecter » (logout best-effort, redirection garantie) ; formulaire figé ; toast
  `admin-restore-toast-success` retiré du code et des quatre catalogues.
- **AC20** : `frontend/tests/e2e/backup-restore-new-install.spec.ts`, trois parcours (C-31-1-75).
- **AC21** : manuel — § *Restaurer sur une nouvelle machine* (`sec:restauration-machine-neuve`, étapes 1 à 5), § *Ce que
  dit l'écran à la fin de l'import*, renvoi depuis *Test de restauration périodique* et depuis *Premier démarrage* ; PDF
  régénéré (`make fr` sous verrou, `311b-make.log` ; PDF des deux autres manuels, seulement redatés, non commités).
- **AC22** : 12 clés neuves ×4 locales, une modifiée (`admin-backup-page-description`), une retirée ; replis identiques
  au fr-CH (vérifiés clé par clé par `grep -F` de la valeur fr-CH dans les sources Svelte) ; CHANGELOG `[0.15.0]` et
  README.

**Tests rouges constatés avant le code** (`311b-red-vitest.log`) : 27 tests en échec et un fichier non chargé
(`onboarding-restore-page.test.ts`, `./+page` absent) — les quatre de `api-client-long-operation` sur la constante, le
délai long et `retry: false` (le « 30 s par défaut » et le témoin à cinq `fetch` passaient : comportement d'avant),
trois du lien de l'assistant, quatre d'`admin-backup.api`, un d'`admin-restore.api`, cinq d'`AdminBackupPanel`, dix
d'`AdminRestorePanel` (dont le test historique du succès, réécrit pour « Se reconnecter »). Verts après le code
(`311b-green-vitest.log` : 9 fichiers, 97 tests).

**Décompte des tests** (de `1ee0583e` à `HEAD`, tests **exécutés**, boucles comprises) : Vitest **+39** —
`api-client-long-operation.test.ts` 0 → 6, `admin-backup.api.test.ts` 9 → 13, `AdminBackupPanel.test.ts` 4 → 10,
`admin-restore.api.test.ts` 2 → 3, `AdminRestorePanel.test.ts` 5 → 15, `onboarding-restore-link.test.ts` 0 → 9,
`onboarding-restore-page.test.ts` 0 → 3 ; la suite passe de 1820 à 1859, cohérent. E2E **+3**. Rust : aucun test
neuf ; deux bornes de garde mises à jour (`i18n-keys.test.ts` `sitesTotal` 2279 → 2289, ventilé au fichier ;
`i18n-un-repli-par-cle.test.ts` 341 → 344) et deux fragments de G18-ter (`textes_coherents.rs`).

**Mutations** (sous le verrou, script `perl` à motif unique vérifié, restauration `git checkout -- f && touch f`, arbre
propre vérifié ; journaux `311b-mut-M-b*.log`, diff en tête) :

| # | Mutation appliquée | Test(s) rejoué(s) | Résultat |
|---|---|---|---|
| M-b1 | lien : `role === 'Admin'` → `currentUser !== undefined` | `onboarding-restore-link.test.ts` | **tuée** (3 rouges : Comptable, Consultation, sans utilisateur) |
| M-b2 | `await relogin()` aussitôt après un import réussi | `AdminRestorePanel.test.ts` | **tuée** (3 rouges) |
| M-b3 | `filesNotIncluded: null` (en-tête ignoré) | `admin-backup.api.test.ts` | **tuée** (2) |
| M-b4 | `getBlob(FULL_EXPORT_URL)` sans l'option longue | `admin-backup.api.test.ts` | **tuée** (l'espion) |
| M-b5 | `retry !== false` retiré de `fetchWithRetry` | `api-client-long-operation.test.ts` | **tuée** (2) |
| M-b6 | `TIMEOUT` retiré d'`issueInconnue` | `AdminRestorePanel.test.ts` | **tuée** |
| M-b7 | `504` retiré d'`issueInconnue` | `AdminRestorePanel.test.ts` | **tuée** |
| M-b8 *(hors fiche)* | garde de `/onboarding/restore` neutralisée | `onboarding-restore-page.test.ts` | **tuée** (2) |
| M-b9 *(hors fiche)* | `timeoutMs` ignoré (30 s rétablis) | `api-client-long-operation.test.ts` | **tuée** (2) |
| M-b10 *(hors fiche)* | condition d'étape du lien retirée | `onboarding-restore-link.test.ts` | **tuée** (3) |

Bilan : 7 mutations de la fiche, 7 tuées ; 3 complémentaires (C-31-1-77), 3 tuées ; aucune équivalente. Les mutations
ont tourné au commit `0bb317c6` ; le code muté n'a pas changé ensuite, hors les classes de couleur de `17c7a8d6`.

**Captures (AC21, thème clair)** — `kesh-gate-logs/311b-captures/`, prises par la spec E2E quand
`KESH_E2E_CAPTURES_DIR` est posé, réécrites par le run final : (a) `a-assistant-etape-0-lien-restaurer.png`, (b)
`b-onboarding-restore.png`, (g) `g-fin-import-sans-piece-manquante.png` et (c) `c-backup-panneau-avis-env.png` — parcours
réel de l'AC20 ; (d) `d-restore-panneau-avis-env.png`, (e) `e-export-pieces-non-incluses.png` (pièce retirée du dossier
**avant** l'export) et (h) `h-fin-import-pieces-manquantes.png` (cette archive importée, la pièce toujours absente) —
parcours réel ; (f) `f-export-depasse-taille-import.png` (en-tête ajouté à la réponse réelle par `page.route`) et (i)
`i-import-issue-inconnue.png` (import remplacé par un 504, `page.route`) — interception réseau. Non copiées au dépôt :
le manuel d'administration n'emploie aucune capture (C-31-1-76). La capture (i) a révélé que l'encart d'erreur des deux
panneaux était **sans couleur** (jetons `danger` inexistants), et l'avertissement de fond sans teinte (`bg-warning-soft`
inexistant) : corrigés en `17c7a8d6` (défaut voisin à correction mécanique).

**Gates réellement exécutés** (périmètre : branche `story/31-1b`, base `1ee0583e`) :

- Au commit `7cc8a3e4` : gate backend `311b-gate-1.log` **rouge** (1 échec, `textes_coherents
  les_ecrans_multi_societe_sont_assumes` — G18-ter, fragment de la description changé par l'AC22 ; corrigé en
  `7f977a04`), puis `311b-gate-2.log` **3908 passés, 7 ignorés** (au commit `7f977a04`) ; frontend `311b-fe-unit.log`
  **rouge** (1 : `i18n-un-repli-par-cle` 344 ≠ 341, corrigé en `7cc8a3e4`), puis au commit `7cc8a3e4` : `check` 0 erreur,
  29 avertissements ; `lint-i18n-ownership` PASS ; Vitest **1859 tests, 159 fichiers** ; `build` vert ; **E2E complet**
  (`311b-e2e.log`) **311 passés, 8 échoués, 17 ignorés** (12,6 min) : `mode-expert.spec.ts:26`, `:41`,
  `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150` — les sept KF-029 (#97) — et
  `sidebar-navigation.spec.ts:75` (KF-052 #424 ou pollution, `docs/testing.md` § *Les échecs attendus* ; non rejoué
  isolément) : 8, dans la fourchette 7 à 10, aucun hors liste.
- Au commit `17c7a8d6` (bases remises à zéro) : backend `311b-gate-3.log` **3908 passés, 7 ignorés** ; `check` 0
  erreur, 29 avertissements ; `lint-i18n-ownership` PASS ; Vitest `311b-fe-unit-3.log` **rouge** (1 :
  `jetons-non-definis.test.ts`, `danger` sans emploi — corrigé en `08ef4bdd`) ; `build` vert ; E2E `311b-e2e-3.log`
  **312 passés, 7 échoués, 17 ignorés** (12,7 min) — les sept KF-029, rien d'autre.
- **Au dernier commit de code `08ef4bdd`** (bases `kesh_311b` et `kesh_e2e_311b` remises à zéro, un seul passage sous
  le verrou) : gate backend `311b-gate-4.log` (`scripts/test-fast.sh --ci` : fmt, clippy `-D warnings`, nextest)
  **3908 passés, 7 ignorés** ; `check` **0 erreur, 29 avertissements** (5090 fichiers) ; `lint-i18n-ownership`
  **PASS** ; Vitest `311b-fe-unit-4.log` **1859 tests, 159 fichiers, tous passés** ; `build` **vert** ; **E2E complet**
  (`311b-e2e-4.log`, port 3312, `kesh_e2e_311b`, dossiers `/tmp/kesh-e2e-311b/*` passés au backend **et** au runner,
  `/health` : `smtpConfigured: true`) **312 passés, 7 échoués, 17 ignorés** (12,7 min). Les 7 échecs, jugés fichier par
  fichier contre `docs/testing.md` § *Les échecs attendus* : `mode-expert.spec.ts:26`, `:41`,
  `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150` — **les sept KF-029 (#97)** ; ni KF-052
  ni pollution ; aucun échec hors liste. Les trois parcours de `backup-restore-new-install.spec.ts` passent, et les
  captures de l'AC21 sont celles de ce run.

**Écarts à la fiche et aux consignes** :

- Textes des avis neutres au nombre (C-31-1-70) : « Pièces justificatives restaurées : 2. » au lieu de « 2 pièces
  restaurées » (AC20), et formulations voisines pour les AC18 et AC19.
- La spec E2E ne fait pas de poignée de main de l'inbox (C-31-1-75) ; elle porte deux parcours de plus que l'AC20.
- Défaut voisin intégré (correction mécanique) : jetons de couleur des encarts (`17c7a8d6`) ; la garde
  `jetons-non-definis.test.ts` l'a relevé — son nom `danger` n'a plus d'emploi — et sa table fermée perd cette entrée
  (`08ef4bdd`, `refs #645`). Les **autres** emplois de jetons non définis (`bg-warning-soft` ×8 hors story,
  `bg-error-soft` ×3…) restent à #645, déjà ouverte : aucune issue neuve.
- Un dossier généré `frontend/src/.svelte-kit/` a été commité par erreur dans `a4012e69` et retiré par `0bd5c55b`
  (l'amendement du commit a été refusé par le système de droits) ; il ne s'est pas reformé ensuite, sa cause n'est pas
  établie — il n'est couvert que par `/.svelte-kit/` à la racine et `frontend/.svelte-kit/` du `.gitignore`.
- Mutations appliquées par un script `perl` (permis pour les mutations), le code lui-même édité à l'outil Edit.
- `/tmp/kesh-e2e-311b/` n'a pas pu être vidé ni supprimé (suppression récursive refusée par le système de droits) :
  à retirer par l'orchestrateur.
- `website/roadmap.html` et la brochure : laissés à la release, comme pour la 31-1a.

### File List

31-1a (de `b5fd9fb1` à `HEAD`) :

- `crates/kesh-db/src/backup.rs`
- `crates/kesh-api/Cargo.toml`
- `crates/kesh-api/src/admin_backup/mod.rs`, `export.rs`, `import.rs`, `manifest.rs`
- `crates/kesh-api/src/document_storage.rs`
- `crates/kesh-api/src/routes/admin.rs`
- `crates/kesh-api/src/config.rs`
- `crates/kesh-api/src/errors.rs`
- `crates/kesh-api/tests/admin_full_export_e2e.rs`, `admin_full_import_e2e.rs`, `configuration_transmise.rs`
- `crates/kesh-api/tests/support/pieces.rs` (neuf)
- `crates/kesh-api/tests/fixtures/keshbackup-0.14.0-format1.keshbackup` (neuf)
- `docs/manual/fr/admin-manual.tex`, `admin-manual.pdf`, `user-manual.tex`, `user-manual.pdf`
- `docs/api-external.md`, `CHANGELOG.md`, `README.md`, `.env.example`, `docker-compose.yml`, `docker-compose.prod.yml`
- `_bmad-output/implementation-artifacts/31-1-sauvegarde-complete.md`, `epic-31-choix-autonomes-311.md`,
  `sprint-status.yaml`, `311a-review-prompt-p1.md` (neuf)

31-1b (de `1ee0583e` à `HEAD`) :

- `frontend/src/lib/shared/utils/api-client.ts`, `api-client-long-operation.test.ts` (neuf)
- `frontend/src/lib/features/admin-backup/admin-backup.api.ts`, `admin-backup.api.test.ts`, `AdminBackupPanel.svelte`,
  `AdminBackupPanel.test.ts`, `BackupEnvNotice.svelte` (neuf)
- `frontend/src/lib/features/admin-restore/admin-restore.api.ts`, `admin-restore.api.test.ts`,
  `AdminRestorePanel.svelte`, `AdminRestorePanel.test.ts`
- `frontend/src/routes/onboarding/+page.svelte`, `onboarding-restore-link.test.ts` (neuf),
  `restore/+page.svelte`, `restore/+page.ts`, `restore/onboarding-restore-page.test.ts` (neufs)
- `frontend/src/lib/shared/i18n-keys.test.ts`, `i18n-un-repli-par-cle.test.ts`,
  `frontend/src/lib/components/ui/jetons-non-definis.test.ts`
- `frontend/tests/e2e/backup-restore-new-install.spec.ts` (neuf)
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`
- `crates/kesh-api/tests/textes_coherents.rs`, `crates/kesh-db/src/backup.rs` (commentaire)
- `docs/manual/fr/admin-manual.tex`, `admin-manual.pdf`, `docs/api-external.md`, `CHANGELOG.md`, `README.md`
- `_bmad-output/implementation-artifacts/31-1-sauvegarde-complete.md`, `epic-31-choix-autonomes-311.md`,
  `sprint-status.yaml`, `311b-review-prompt-p1.md` (neuf)

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

### 2026-10-24 — Validation P4 (Opus 5.5, lentilles R et F) et remédiation (Opus 5.5)

Rapports : `kesh-gate-logs/311-validate-p4-R.md` (**0 CRITICAL / 0 HIGH / 3 MEDIUM / 11 LOW**) et
`311-validate-p4-F.md` (**0 / 0 / 4 / 7**). Le prompt annonçait un erratum en fin du rapport F : il n'y en a pas
(`grep -ni erratum` vide ; le rapport se termine sur « Axes exercés et non exercés »). Chaque finding vérifié avant
correction, au dépôt (`grep -nF`, `sed -n`, `awk`) et dans les sources du registre cargo : `zip` 2.4.2
(`write.rs:1134` `abort_file`, `:889` `ok_or_abort_file`, `:1501-1524` pied réécrit), `tokio` 1.52.3 (`lock_owned`,
`OwnedMutexGuard` `Send + 'static`), `crates/kesh-api/Cargo.toml:79-83` (`regex` sous `[dev-dependencies]`),
`import.rs:85-116` (refus de `files/` non vide avant `formatVersion`), `migrations_upgrade_path.rs:120, 142-144, 199`,
`post_restore.rs:1122, 1214-1218`, `admin_pat_denied_e2e.rs:848-861` (`discriminating >= 4`),
`AdminRestorePanel.svelte:120-122` et les quatre `.ftl` (`admin-restore-toast-success`), `docker-compose.yml:131-133,
154-158`, `docker-compose.prod.yml:153-155, 165-176`, `config.rs:895-896, 1268-1269`, `.env.example:180-193, 220-223,
255-257`, `routes/admin.rs:52, 875-930`, `export.rs:1-12`, `admin-manual.tex:1838`. La commande de l'AC14, rejouée en
`-i` avec les compose, rend **103** lignes (94 + 8 des compose + `config.rs:255`).

**Réfuté** : R10, premier point — les numéros de `.env.example` de l'AC14 et du § Gardes sont **justes** : `sed -n
180,193p .env.example | cat -n` place le commentaire « défaut 512 » l. 185, la valeur l. 186 et le bloc « Dossier de
sauvegarde de sécurité » l. 187-192 (le rapport les décalait d'une ligne). Les trois autres points de R10 sont exacts
(onze lignes sans filtre ; `regex` dépendance de test ; liste des vérifiés).

**Origine.** Nés de la remédiation P3 : F2 (deux passes sur une prémisse fausse), F3 (commande de l'AC14), F7
(`sync_all` à `?`), R4 (voie de la capture (h)), R13 (archive empruntée par un `spawn_blocking`). Nés du choix C-31-1-52
(postérieur à la P3) : R1. D'origine, non vus en P1 à P3 : R2, R3, R5 à R12, R14, F4, F5, F6, F8 à F11. F1 reprend un
angle mort écrit en P3.

**Simplifications** (une exigence retirée plutôt que compensée) :

- **F2** — **une seule passe** par fichier : `abort_file` puis `altered` ou `unreadable` ; retirés la double lecture,
  l'abandon de toute la sauvegarde sur une erreur de lecture en seconde passe, et la mutation déclarée équivalente du
  re-hachage (AC3, AC4, T-a4, T-a4b ; M-a21 ; C-31-1-53, qui amende C-31-1-47). Établi au dev par un aller-retour
  (entrée abandonnée au milieu suivie d'une saine).
- **F5** — réservation conditionnelle en base retirée, avec `Skipped` sur réservation perdue, son test et **M-c7** ;
  `claim_attempt` devient `record_attempt` sans condition ; angle mort et § Périmètre disent qu'aucune protection n'existe
  entre processus (AC26, AC27, AC32, AC36, T-c2 ; C-31-1-54, qui amende C-31-1-23 et C-31-1-45).
- **F1** — le garde de l'export manuel entre dans la fermeture `remplir`, donc dans le `spawn_blocking` ; l'angle mort
  *(P3)* « relâche le verrou » disparaît (AC4, AC27, AC36 ; M-c24 ; C-31-1-55).
- **R7** — `build_keshbackup(pool, documents_dir, remplir)` reste l'unique lecteur de la base ; la garde existante reste
  exacte, et `avant_sauvegarde_branchee_aux_appels_de_l_import` garde ses motifs et son compte (AC4, § Gardes, T-a4 ;
  C-31-1-56).

**Corrections** :

- **R1 (C-31-1-52 reporté)** — `importMaxMib` dans `GET backups` (`{ importMaxMib, items }`), mention « trop grande pour
  être restaurée par l'écran (limite N Mio) » (AC32, AC33), test Rust (AC36), Vitest au-dessus / égal / au-dessous
  (AC37), M-c23, capture (r) (AC40), clé `admin-backup-server-too-large` ×4, phrase du manuel (AC39) ; angle mort *(P3)*
  réécrit « levé en P4 » ; en-tête « C-31-1-1 à C-31-1-58 » ; C-31-1-58.
- **R2** — test « format 1 restauré sur la même installation → `filesMissing = 0` » (AC9, AC13, T-a6b) ; M-a9 reciblée.
- **R3** — AC20 (6) : la facture et son écriture au journal reviennent.
- **F3** — commande de l'AC14 en `-i`, compose inclus, 103 lignes ; sites des compose au tableau ; **R9** — règle
  « au tableau, au tableau du Dev Agent Record, ou intacte avec raison » et sept sites ajoutés (`config.rs:895-896`,
  `:1268-1269`, `.env.example:220-223`, `:255-257`, `routes/admin.rs:52`, `export.rs:1-12`, `admin-manual.tex:1838`).
- **F4** — `is_due` : `last_*` futur vaut nul ; `is_overdue` au serveur, champ `overdue`, le bandeau ne lit plus
  l'horloge du navigateur (AC25, AC31, AC32, AC36, AC37, AC39, T-c3 ; M-c22, M-c9 reciblée ; C-31-1-57).
- LOW : R4 (AC21 (h) ; AC40 (a), (j), (k)) ; R5 (M-c3, M-c4, M-c8 reciblées ; 8 sauvegardes déposées dans les tests
  `failed`, `insufficient_space`, `disk_full` sous `retention_count = 3`) ; R6 (§ Gardes : `upgrade_path_preserves_data`,
  `EXEMPT_MIGRATIONS.len()`) ; R8 (AC26, M-c21 : lignes `//` écartées) ; R10 (AC27 : onze lignes ; `regex`) ; R11
  (AC14 : message vu par une 0.14.0) ; R12 (AC18 : `mismatch`) ; R13 (AC8 : `ParsedBackup` par valeur ; AC9 en
  `spawn_blocking`) ; R14 (`assert_eq!(discriminating, 6)` ; toast `admin-restore-toast-success` retiré, AC19, T-b3 ;
  `nextDueAt` après l'échec d'un tour planifié) ; F6 (AC1 : deux requêtes, dédoublonnage en Rust ; T-a2 ; M-a22) ; F7
  (AC8 : échec du `sync_all` du dossier journalisé, angle mort) ; F8 (AC5, AC8, AC9 : `?path`) ; F9 (AC14 : pic de
  disque au manuel ; AC28 : nettoyage des `.partial` pré-import, test AC36) ; F10 (AC11 : `tokio::fs::File::from_std` ;
  AC30 : `documents_bytes` dans le `spawn_blocking` ; AC9) ; F11 (= R10, `regex` aux `[dependencies]`, AC6, T-a4).

**Annotation datée de l'entrée P3** (2026-10-24, sans la réécrire) : sa ligne « **À ouvrir en issue** … F12 … non
traité ici (C-31-1-51) » est caduque — l'orchestrateur a écarté l'issue et intégré F12 à la 31-1c (C-31-1-52), dont
la forme est fixée par C-31-1-58. Aucune issue n'est à ouvrir pour F12.

Découpage : recompté, 31-1a : 3 modules (`kesh-db`, `kesh-api`, docs — les commentaires des compose comptés avec docs),
31-1b : 4, 31-1c : 5 ; la remédiation n'en ajoute à aucune ; pas de redécoupage.

Recomptes (depuis la fiche et le registre, après remédiation) : **43 AC** (`grep -cE '^\*\*AC[0-9]+b? '`, inchangé) ;
**26 tâches** (`grep -cE '^- \[ \] T-'`, inchangé : le test de F6 est versé à T-a2) ; **52 mutations** (`grep -cE '^\|
M-'` : 22 + 7 + 23 — M-a1 à M-a22, M-b1 à M-b7, M-c1 à M-c24 sans M-c7 ; +5 : M-a21, M-a22, M-c22, M-c23, M-c24 ; −1 :
M-c7 ; reciblées : M-a9, M-c3, M-c4, M-c8, M-c9, M-c21) ; **58 choix** (`grep -c '^### C-31-1-'` ; 52 à `6f0828cf`, +6 :
C-31-1-53 à C-31-1-58). Valeurs corrigées grepées sur la fiche et le registre (`réservation`, `claim_attempt`, `deux
passes`, `passe 2`, `passe de hachage`, `re-hach`, `douze`, `M-c7`, `relâche le garde`, `≥ 4`, `C-31-1-51`, `94 lignes`,
`en en retirant`) : aucun résidu hors des mentions historiques voulues (entrées passées du Change Log et du registre —
C-31-1-23, C-31-1-45, C-31-1-47, C-31-1-51 —, que les choix neufs amendent sans les réécrire ; « 94 à `c7639e80` »
gardé à l'AC14 comme relevé daté ; les « ≥ 4 » du tableau des tests, qui comptent des tests).

### Validation P5 (2026-10-24) — Sonnet, R 0/0/1/9, F 0/0/0/6 ; MEDIUM remédié par l'orchestrateur

- **R1 (MEDIUM ; F1 de la lentille F, coté LOW)** — `Box<dyn Write + Seek + Send>` n'est pas un type valide (E0225 : un
  seul trait non automatique par objet-trait). Corrigé à l'AC30 : trait d'union `WriteSeek: Write + Seek + Send` et son
  implémentation générale ; `Box<dyn WriteSeek>`. Registre : amendement daté de C-31-1-46 (sans réécrire l'entrée).
- **R2 (LOW, retenu parce qu'il touche la taille déclarée)** — `abort_file` ne rembobine que si les entrées suivantes ont un
  `data_start` connu ; l'entrée de répertoire `files/` écrite par `add_directory` n'en a pas. Règle fixée à l'AC3 : **aucune
  entrée de répertoire `files/`** n'est écrite en format 2 (les entrées `files/<nom>` suffisent, et le format 1 garde la
  sienne) ; l'archive ne porte donc pas d'octets morts après un abandon.
- Autres LOW (R3 à R10, F2 à F6) laissés au développement, qui les applique à l'étape 0 ou les consigne : R3 (garde de
  signature de `build_keshbackup`), R4/F3 (type d'erreur), R5, R6, R7, R8 (lecture ZIP dans Playwright), R9, R10 (marquer
  les entrées amendées), F2 (`KESH_DOCUMENTS_DIR` absent ⇒ 0), F4, F5 (M-a20 muette sous root), F6 (dossier d'entrée).
- Passe suivante : ciblée P6 (Haiku) sur le commit de cette remédiation.

### Validation P6 ciblée (2026-10-24) — Haiku, lentille T sur `c9c36ae8` : 0/1/1/3 ; remédiée par l'orchestrateur

- **T2 (MEDIUM), vérifié aux sources** : la prémisse de R2 (P5) était **fausse** — `add_directory` appelle `start_entry`
  (`zip-2.4.2/src/write.rs:1422`), qui pose `data_start` (`:1040-1042`) ; l'entrée de répertoire `files/` ne bloque donc pas
  le rembobinage d'`abort_file` (`:1146-1153`). **T1 (HIGH)**, conséquence directe : retirer l'entrée `files/` en format 2
  contredisait l'AC3, l'AC6 et T-a4b, et l'import actuel (`has_files_dir`) aurait refusé une sauvegarde sans pièce.
  **Défait** : la phrase ajoutée à l'AC3 en `c9c36ae8` est retirée ; l'entrée `files/` reste écrite, en format 1 comme en 2.
  *Une remédiation a produit le défaut suivant : c'est l'orchestrateur qui l'a écrite, sans relire la source qu'il
  croyait connue.*
- **T5 (LOW)** : `pub(crate) trait WriteSeek` (lint `private_interfaces`).
- T3, T4 (LOW) : laissés au développement (T4 : une archive de format 2 portant l'entrée `files/` est la forme normale).
- Passe suivante : ciblée P7 (Sonnet) sur ce commit.

### Validation P7 ciblée (2026-10-24) — Sonnet, lentille T sur `HEAD~2..HEAD` : 0/1/0/3 ; remédiée par l'orchestrateur

- **H1 (HIGH)**, né de la remédiation P6 (T5) : `pub(crate) trait WriteSeek` aurait été invisible des tests d'intégration
  de l'AC36, qui vivent dans un crate externe, et la fiche ne déclarait nulle part la visibilité de `BackupEnv`. Corrigé à
  l'AC30 : `pub trait WriteSeek`, et la liste des éléments **`pub`** du module `auto_backup` écrite.
- Axe 1 confirmé : l'état final de l'entrée `files/` est cohérent (AC3, AC6, AC6b, T-a4b) ; le Change Log ne reçoit que
  des ajouts. LOW laissés au développement (L1 : les deux amendements de C-31-1-46 se lisent ensemble ; entrée d'origine
  non marquée).
- *Deuxième remédiation de l'orchestrateur, deuxième défaut introduit* (P5 → T1, P6 → H1) : la règle « la sévérité se
  déplace vers ce qu'on vient d'écrire » vaut pour l'orchestrateur comme pour les agents.
- Passe suivante : ciblée P8 (Opus) sur ce seul commit — **dernière passe permise** par le plafond de huit.

### Validation P8 ciblée (2026-10-24) — Opus, lentille T sur `6e1db7f7` : 0/0/3/5 — **plafond de huit passes atteint**

- **M1** (né de la remédiation P7, de l'orchestrateur) : `RunTrigger` n'existe pas, le type s'appelle `Trigger` partout ;
  corrigé dans la liste `pub` de l'AC30 et au registre.
- **M2** : la liste `pub` omettait le statut de `RunOutcome::Done` et l'erreur de `run_once` ; complétée (`FailureStatus`,
  statut de succès, type d'erreur) ; `spawn` est `pub` à cause de `main.rs` (crate binaire), `classify_write_error` reste
  privée, testée en `mod tests`.
- **M3** : l'AC36 prouvait l'importabilité par `parse_and_verify`, inaccessible d'un crate externe (`admin_backup` est
  `pub(crate)`, Story 17-3a) ; la preuve passe par la route `POST …/full-import`.
- LOW L1 à L5 laissés au développement (L5 : les entrées P5 à P7 portent la date du 2026-10-24, leurs commits celle du
  2026-10-23 au fuseau UTC — jour de travail continu, non corrigé).
- **Fin de la validation au plafond** (`CLAUDE.md` § *Review Iteration Rule* : huit passes au plus). Trend : P1 Sonnet R
  0/0/10/8 F 0/2/9/8 → P2 Opus R 0/0/8/8 F 0/1/7/9 → P3 Sonnet R 0/0/5/7 F 0/0/4/11 → P4 Opus R 0/0/3/11 F 0/0/4/7 → P5
  Sonnet R 0/0/1/9 F 0/0/0/6 → P6 ciblée Haiku 0/1/1/3 → P7 ciblée Sonnet 0/1/0/3 → P8 ciblée Opus 0/0/3/5. **Les trois
  MEDIUM de la P8 sont corrigés par l'orchestrateur sans passe de vérification** — l'agent de développement les recontrôle
  à son étape 0, et la revue de code P1 les couvre. *Les P6 à P8 n'ont trouvé que des défauts introduits par les
  remédiations de l'orchestrateur lui-même* (P5 → T1, P6 → H1, P7 → M1) : la règle de la passe ciblée s'est vérifiée sur
  celui qui l'applique.

### 2026-10-24 — Développement de la 31-1a, étape 0 : recontrôle de la P8 et LOW des P5 à P8 (Opus 5.5)

- **M1 (P8) recontrôlé** : `grep -rn RunTrigger _bmad-output docs crates` ne rend que l'entrée de ce Change Log qui
  rapporte la P8 et l'amendement P7 du registre, suivi de l'amendement P8 qui le corrige — relevés datés, laissés. La
  fiche dit `Trigger` partout (AC26, AC27, AC30, AC32, AC35). Conforme.
- **M2 (P8) recontrôlé** : la liste `pub` de l'AC30 nomme `FailureStatus`, le statut de succès et le type d'erreur de
  `run_once`, `spawn` motivé par `main.rs`, `classify_write_error` privée. Conforme (31-1c).
- **M3 (P8) recontrôlé** : l'AC36 prouve l'importabilité par la route `POST /api/v1/admin/full-import` et laisse
  `admin_backup` en `pub(crate)`. Conforme (31-1c).
- **LOW appliqués (mécaniques, 31-1a)** : R3 (P5, motif d'ouverture de la garde `build_keshbackup_ne_lit_la_base_qu_une_fois`,
  § Gardes) ; R7 (P5, « jusqu'à environ trois fois », AC14) ; F6 (P5, documents en attente de l'inbox non emportés,
  AC14) ; T4 (P6) et L3 (P7) (entrée `files/` de format 2 : forme normale, écrite avant les pièces, AC3) ; R10 (P5),
  T3 (P6), L1 (P7) : marques datées sur les entrées amendées C-31-1-23, 45, 46 et 47 du registre, sans les réécrire.
- **LOW consignés, tranchés au développement de la 31-1a** : R4 (P5) = F3 (P5), type d'erreur de `build_keshbackup`
  (choix au registre) ; F5 (P5), M-a20 sous `root` : le test échoue bruyamment s'il peut lire le fichier, comme T-a4b le
  prescrit.
- **LOW consignés pour la 31-1c** (non touchés ici) : R5, R6, R8, R9 (P5) ; F2, F4 (P5) ; L1 à L4 (P8 — phrase de l'AC30
  disloquée, motif de `spawn`, `classify_write_error` testée en `mod tests`, visibilité de `backup_lock`). L5 (P8, dates)
  laissé.

### 2026-10-24 — Développement de la 31-1a (Opus 5.5) → review

T-a0 à T-a9 faits (détail au Dev Agent Record) : fixture 0.14.0 réelle produite au tag `v0.14.0` (C-31-1-59) ;
format `.keshbackup` 2, export hors mémoire, import reçu sur disque et détaché, pièces écrites avant la base, plafond
d'import 2048 ; manuels FR et PDF, `api-external`, CHANGELOG, `.env.example`, compose, README. Gate complet backend
3907 passés / 7 ignorés au dernier commit de code (`3586d3c5`) ; frontend : check 0 erreur, Vitest 1820, build vert ;
E2E 307 passés / 7 échecs (les sept KF-029) / 17 ignorés. Mutations : 21 tuées, M-a6 équivalente (C-31-1-63), variante
M-a6b tuée. Choix C-31-1-60 à 68. Écarts déclarés (une édition par `cat >>`, rouges constatés après coup pour `kesh-db`
et l'intégration). Revue de code P1 à lancer : `311a-review-prompt-p1.md` (Sonnet, lentilles B, E, A).

### Revue de code P1 de la 31-1a (2026-10-24) — Sonnet, lentilles B, E, A : 0 MEDIUM+ après reclassement — revue terminée

B 0/0/1/5, E 0/0/0/7, A 0/0/0/5 (`kesh-gate-logs/311a-review-p1-{B,E,A}.md`). **B-M1 reclassé LOW par l'orchestrateur, sur
preuve** : « `abort_file` ne tronque pas, l'archive serait illisible par les outils qui cherchent le pied dans les
derniers 64 Kio » — `zip-2.4.2/src/write.rs` `finalize` (≈ l. 1501-1524) invalide l'ancien pied et **réécrit** le
répertoire central et le pied à la fin réelle du fichier quand des octets abandonnés le dépassent ; la lentille E l'a
établi aussi. Restent deux effets mineurs, consignés : `byte_size` compte les octets morts d'une entrée abandonnée, et le
contenu d'une pièce écartée subsiste dans l'archive. LOW consignés pour la 31-1b ou la release (aucun ne touche la
production) : A-L1 (commentaire `kesh-db/src/backup.rs:642` périmé — premier commit de code de la 31-1b), A-L2/L3/L5 et
B-L1/B-L3 (le manuel et `api-external.md` à compléter : refus propres au format 2, bornes de lecture, 410 d'une pièce non
emportée, en-tête `X-Kesh-Backup-Exceeds-Import-Limit` sans écran avant la 31-1b), A-L4 (`formatVersion: 0`, comportement
inchangé), B-L2, B-L4, B-L5, E-2 à E-7 (dont E-3 — pièces écrites avant un refus tardif, non nettoyées — et E-4 — verrou
de `_kesh_version` tenu pendant la sauvegarde pré-import —, angles morts à porter au manuel par la 31-1b). Statut de la
31-1a : `done`.

### 2026-10-24 — Développement de la 31-1b (Opus 5.5) → review

LOW de la revue P1 de la 31-1a traités en premier commit (commentaire de `backup.rs`, refus du format 2, bornes, 410
d'une pièce non emportée, angles morts E-3 et E-4 ; C-31-1-74). T-b1 à T-b7 faits (détail au Dev Agent Record) : lien
« Restaurer une sauvegarde » de l'assistant et page `/onboarding/restore` gardée ; avis `.env` partagé ; avis d'export
(pièces non incluses, plafond dépassé) ; bilan de l'import et « Se reconnecter » ; appels longs hors délai de 30 s, sans
rejeu, et issue inconnue d'un import (`TIMEOUT`, `NETWORK_ERROR`, 502, 504) ; i18n ×4 ; E2E de bout en bout et deux
parcours d'états ; manuel *Restaurer sur une nouvelle machine* et PDF ; CHANGELOG, README. Défaut voisin intégré : jetons
de couleur inexistants des encarts. Vitest 1820 → 1859 ; mutations 10 tuées sur 10 (dont 3 hors fiche). Choix
C-31-1-69 à 77. Revue de code P1 à lancer : `311b-review-prompt-p1.md` (Sonnet, lentilles B, E, A).

### Revue de code P1 de la 31-1b (2026-10-24) — Sonnet, lentilles B, E, A : B 0/0/1/5, E 0/0/0/6, A 0/0/0/6 — remédiée (Opus 5.5)

Rapports : `kesh-gate-logs/311b-review-p1-{B,E,A}.md`.

*Annotation datée (2026-10-24), relevés ci-dessus non réécrits* : la branche a été **reportée sur `origin/main`** par
l'orchestrateur (`rebase --onto`, arbre inchangé) ; les hashes cités au Dev Agent Record de la 31-1b et à l'entrée
« Développement de la 31-1b » sont ceux d'avant le report. Correspondance : `dbf6206d` → `d3a6b64b`, `a4012e69` →
`850503e5`, `0bd5c55b` → `98a32c15`, `e060e4c4` → `2a14ebb4`, `0bb317c6` → `f3719339`, `7f977a04` → `67ddcfc7`,
`7cc8a3e4` → `c0aed5f7`, `17c7a8d6` → `44d30f57`, `08ef4bdd` → `3e3377da`, `e7189cf6` → `1fcb7732` ; base `1ee0583e`
→ `a3e4bd62` (merge-base avec `origin/main`). *Seconde annotation (A-L1)* : « 12 clés neuves ×4 locales, une modifiée »
du Dev Agent Record est faux à sa date : **11 neuves**, une modifiée (`admin-backup-page-description`), une retirée —
recompté par `git diff origin/main...HEAD -- crates/kesh-i18n/locales/fr-CH/messages.ftl | grep -E '^[+-][a-z]'` (le
compte « 12 » incluait la ligne modifiée).

**Remédié (commit `9d6f3184`)** :

- **B-1 (MEDIUM)** — le manuel citait « N pièces référencées n'ont pas pu être incluses », texte de la fiche que l'écran
  n'affiche pas. **Toutes** les citations d'écran ajoutées au manuel par les 31-1a et 31-1b ont été relevées (`git diff
  origin/main...HEAD -- docs/manual/fr/admin-manual.tex | grep -oE '« [^»]+ »'`, 13 citations) et confrontées au
  catalogue fr-CH : onze exactes ; deux fausses. La citation de B-1, alignée (« Pièces référencées que le fichier n'a pas
  pu inclure : N ») ; et **« document indisponible »**, cité à cinq endroits du manuel **et dans deux de mes propres avis
  d'écran** (`admin-backup-files-not-included`, `admin-restore-files-missing`, quatre locales et replis), alors qu'aucun
  écran ne le dit : un document fournisseur absent affiche « Le document n'a pas été restauré. », un PDF figé absent « Le
  PDF émis de cette facture est introuvable… ». Les avis disent désormais « elles seront introuvables » / « Restaurez le
  dossier des documents… », le manuel cite les deux vrais messages. Grep de la valeur sur `docs`, `crates`,
  `frontend/src`, `frontend/tests`, CHANGELOG, README : zéro reste (« Dokument nicht verfügbar », « documento non
  disponibile », « document unavailable » compris). PDF régénéré (`311b-make-2.log`), contrôlé aplati.
- **A-L3** — `admin-restore-confirm-body` (quatre locales et repli) ne promet plus « Vous serez déconnecté » : « À la
  fin, votre session sera fermée : vous vous reconnecterez… ». Test Vitest neuf.
- **B-2** — le manuel conditionne « l'application s'ouvre sans l'assistant » (société configurée et administrateur dans
  la sauvegarde, `force_onboarding_done_if_eligible`).
- **B-3** — le commentaire du 503 d'`issueInconnue` nomme l'angle mort (coupure de la base au `COMMIT`, rendue en 503).
- **E-LOW-3 / A-L5** — après une issue inconnue, le choix du fichier et « Importer » sont désactivés (recharger la page
  rouvre l'import) ; le manuel le dit, et dit que le message apparaît aussi quand rien n'est parti (E-LOW-4, B-6) ; le
  critère « identifiants acceptés » de la note est remplacé par la vérification des données ou de l'audit (A-L5). Deux
  tests Vitest neufs (issue inconnue → import bloqué ; 409 → import proposé), une assertion E2E (bouton désactivé après le
  504).
- **E-LOW-2** — `/onboarding/restore` suit la borne du lien : un Administrateur au-delà de l'étape 2 est renvoyé à
  `/onboarding` (`+page.ts`, `await parent()` puis `onboardingState`) ; test de garde réécrit (7 cas).
- **E-LOW-5 / A-L4** — assertions E2E exactes : « Pièces justificatives restaurées : 2. » (`toHaveText`), « … : 0. » et
  « … introuvables après la restauration : 1. » au second parcours, « … n'a pas pu inclure : 1 ( » à l'export.

**Consignés, non traités** (LOW, sans effet comptable) : B-4 (`timeoutMs` ≤ 0 abandonne aussitôt — aucun appelant ne le
passe), B-5 et E-LOW-6 (`headerCount` : valeur hors forme ou en-tête dupliqué par un proxy → aucun avis), E-LOW-1
(`authState.logout()` sans délai : un serveur muet fige « Se reconnecter »), A-L6 (`frontend/src/.svelte-kit/` non
couvert par le `.gitignore` — à décider par l'orchestrateur).

**Mutations de la remédiation** (sous le verrou, `311b-mut-M-b1{1,2,3}.log`) : M-b11 (« Importer » sans
`unknownOutcome`) **tuée** ; M-b12 (borne d'étape de la garde retirée) **tuée** (2) ; M-b13 (sélecteur de fichier sans
`unknownOutcome`) **tuée**.

**Gates après la remédiation** (au commit `9d6f3184`, bases remises à zéro, un seul passage sous le verrou) :
backend `311b-gate-5.log` **rouge** — 3907 passés, 1 échoué, 7 ignorés :
`kesh-i18n loader::tests::la_cloture_d_exercice_ne_parle_pas_comme_un_panneau`, le de-CH neuf
d'`admin-restore-confirm-body` employait « geschlossen », réservé aux panneaux ; `check` **0 erreur, 29 avertissements**
(5091 fichiers) ; `lint-i18n-ownership` **PASS** ; Vitest **1866 tests, 159 fichiers** (+7 : trois
d'`AdminRestorePanel`, quatre de la garde de `/onboarding/restore`) ; `build` **vert** ; **E2E complet**
(`311b-e2e-5.log`) **311 passés, 8 échoués, 17 ignorés** — les sept KF-029 (`mode-expert:26`, `:41`,
`onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`, `:150`) et `sidebar-navigation.spec.ts:75` (KF-052 #424 ou
pollution, non rejoué isolément), 8 dans la fourchette 7 à 10 de `docs/testing.md` ; les trois parcours de
`backup-restore-new-install.spec.ts` passent. Correctif `ea51180b` (« beendet », un texte allemand, ni code ni test) :
gate ciblé `kesh-i18n` 41/41, puis **gate backend complet** `311b-gate-6.log` au commit `ea51180b` : **3908 passés, 7
ignorés**. Ni Vitest ni l'E2E ne lisent ce texte allemand : ils n'ont pas été rejoués après `ea51180b`.

Passe suivante : **P2 ciblée** (Haiku, une lentille T sur le seul diff `1fcb7732..ea51180b`),
`311b-review-prompt-p2-ciblee.md`.

### Revue de code P2 ciblée de la 31-1b (2026-10-24) — Haiku, lentille T sur `1fcb7732..ea51180b` : 0/0/2/5 ; remédiée par l'orchestrateur

Les deux MEDIUM, vérifiés au code par l'orchestrateur (règle des MEDIUM de Haiku), sont **réels** :
- **M-1** — le commentaire « angle mort » de `AdminRestorePanel.svelte` (B-3 de la P1) disait qu'un échec au `COMMIT`
  rend un 503 ; `routes/admin.rs:603-605` le convertit en `AdminFullImportFailed` (500). Commentaire réécrit : ce que
  l'écran montre réellement, le cas ordinaire (`COMMIT` échoué annulé, état préservé) et le cas rare (coupure après
  validation, issue inconnue présentée comme un échec ordinaire). *Aucun code de production changé.*
- **M-2** — le manuel disait qu'une sauvegarde « prise avant la fin de l'assistant » le rouvre ; c'est la société
  **provisoire** qui décide (`force_onboarding_done_if_eligible`, `kesh-db/src/backup.rs` : société non provisoire et
  administrateur) — une sauvegarde prise après la saisie des coordonnées, ou de démonstration, s'ouvre directement.
  Manuel corrigé, PDF régénéré (`make admin`), contrôlé aplati.
- LOW : L-1 corrigé (commentaire de `admin-restore.api.ts` : les deux messages réels du 410) ; L-2 à L-5 consignés
  (L-2 : deux clés modifiées et non une ; L-4 : la garde d'étape en démonstration n'est pas testée ; L-5 : garde inopérante
  si l'état d'assistant ne se charge pas — l'autorité reste le RBAC du serveur).
- Gates : la remédiation ne change que deux commentaires et le manuel ; l'E2E complet est rejoué au dernier commit de code
  de la story (règle D7). Passe suivante : ciblée P3 (Sonnet) sur ce commit.

### Gates au dernier commit de code `06cc4ad4` de la 31-1b (2026-10-24)

Bases `kesh_311b` et `kesh_e2e_311b` remises à zéro (DROP/CREATE, `sqlx migrate run`, `seed-dev-db.sql`) avant les
gates ; aucun code modifié. Journaux : `kesh-gate-logs/311b-final-{backend,frontend,e2e,e2e-backend}.log`.

| Gate | Résultat |
|---|---|
| `scripts/test-fast.sh` (fmt, clippy, nextest) | vert — **3908 tests exécutés, 3908 passés, 7 ignorés** (161 s) |
| `npm run check` | rc 0 (avertissements `state_referenced_locally` préexistants) |
| `npm run lint-i18n-ownership` | rc 0 |
| `npm run test:unit` | **1866 tests passés, 159 fichiers** |
| `npm run build` | rc 0 |
| E2E Playwright complet (port 3312, `KESH_INBOX_DIR`, `KESH_DOCUMENTS_DIR`, `KESH_ADMIN_BACKUP_DIR` au backend et au runner) | **312 passés, 7 échoués, 17 ignorés** (12,6 min) |

Les 7 échecs E2E sont exactement les sept de la KF-029 (#97) de `docs/testing.md` § *Les échecs attendus* :
`mode-expert.spec.ts:26` et `:41`, `onboarding-path-b.spec.ts:65` et `:92`, `onboarding.spec.ts:57`, `:77`, `:150`.
Aucun échec hors liste ; KF-052 (`sidebar-navigation:75`) n'a pas déclenché.
