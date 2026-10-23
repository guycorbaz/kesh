# Story 31.1 : Une sauvegarde qui restaure tout, faite par Kesh

## Status

**ready-for-dev** (trois clés) — fiche créée le 2026-10-23 en autonomie (`bmad-create-story`, Opus 5.5), sur
`1848012d` (kickoff de l'Epic 31), worktree `/home/gcorbaz/devel/kesh-31-1`, branche `story/31-1`. Validation P1
faite (Sonnet, R 0/0/10/8, F 0/2/9/8) et remédiée le 2026-10-23 ; validation **P2** à lancer
(`311-validate-prompt-p2.md`, **Opus**, lentilles R et F).

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
§ 31-1. Cible : **v0.15.0**. Choix : `epic-31-choix-autonomes-311.md` (C-31-1-1 à C-31-1-36). Réglages nouveaux dans
l'interface, pas en variable d'environnement (C-31-0-4).

## Prémisses de #503, établies au code (`1848012d`)

| # | Prémisse | Verdict | Preuve |
|---|---|---|---|
| P-1 | L'export crée un `files/` vide | **vraie** | `crates/kesh-api/src/admin_backup/export.rs:95` (`zip.add_directory("files/", …)`, commentaire « aucun binaire stocké en v0.2 ») ; l'issue citait l. 76-79, le fichier a bougé depuis (Story 28-3) |
| P-2 | L'import refuse un `files/` non vide | **vraie** | `admin_backup/import.rs:79-96` (`files_non_empty`, l. 83 et 91) ; et exige sa présence (l. 98-102) |
| P-3 | Après restauration, un justificatif et un PDF figé répondent **410** | **vraie sur une autre machine, à nuancer sur la même** | `routes/imported_supplier_invoices.rs:725` (`SourceDocumentGone`) ; `routes/issued_invoice_pdf.rs:365` (`InvoicePdfGone`). ⚠️ Restaurée **sur la même installation**, une sauvegarde retrouve ses fichiers : Kesh ne supprime jamais un document (`grep -rn remove_file crates/kesh-api/src` : seulement le fichier source de l'inbox, `inbox_import.rs:839`, et les temporaires). Le 410 frappe la restauration sur un **autre** dossier de documents — le cas de #503 |
| P-4 | Les fichiers référencés par la base sont « les justificatifs et les PDF figés » | **vraie, et l'inventaire est clos** | deux appelants de production de `document_storage::store_document` : `inbox_import.rs:580` (documents fournisseur → `imported_supplier_invoices.storage_path`, `file_hash`) et `routes/issued_invoice_pdf.rs:432` (PDF figé → `invoices.pdf_storage_path`, `pdf_sha256`, migration `20261003000001`). Les documents joints après coup à une facture fournisseur (30-2c) sont des lignes de `imported_supplier_invoices`. Aucune autre colonne `*storage_path*` (`grep -rn storage_path crates/kesh-db/migrations`). Avoirs, rappels, rapports : rendus à la demande, jamais stockés |
| P-5 | Nommage par empreinte SHA-256 | **déjà vrai sur disque** | `document_storage.rs:104-110` : nom `{sha256hex}.{ext}` (extension en minuscules, liste blanche, `is_safe_ext`) ; le contenu d'un nom donné est donc fixé par le nom |
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
| 31-1c | `kesh-db` (migration, squash **et son script `scripts/regen-test-schema.sh`**, `tests/test_schema_guard.rs`, dépôt, `post_restore`, inventaire), `kesh-api` (tâche de fond, routes, `AppState`, `main.rs`, `tests/admin_pat_denied_e2e.rs`), frontend (panneau, bandeau), `kesh-i18n`, docs | 5 |

Fusionnées, la 31-1a et la 31-1b dépasseraient cinq modules ; la 31-1c seule en atteint cinq (C-31-1-1). La
remédiation P1 n'ajoute de module à aucune sous-story : le correctif du client HTTP (AC18b) va dans la 31-1b, qui
porte déjà le frontend ; `scripts/regen-test-schema.sh` est compté avec `kesh-db`, dont il produit le squash
(`crates/kesh-db/test-schema/`) et dont la garde le contrôle (C-31-1-36).

## Critères d'acceptation

### Partie a — `31-1a-fichiers-dans-la-sauvegarde`

**AC1 — Inventaire des fichiers référencés, lu dans l'instantané.** `kesh_db::backup` expose
`ReferencedFile { storage_path: String, sha256: String }` et l'`InstallationExport` porte `referenced_files:
Vec<ReferencedFile>` (dédoublonnés par `storage_path`, triés), lus par `export_installation_in` **dans le même
`ReadSnapshot`** que les tables, après elles :
`SELECT storage_path, file_hash FROM imported_supplier_invoices` ∪ `SELECT pdf_storage_path, pdf_sha256 FROM invoices
WHERE pdf_storage_path IS NOT NULL`. Toutes les lignes de `imported_supplier_invoices` comptent, quel que soit leur
statut (`discarded` compris : la ligne reste restaurée et son document téléchargeable). La clé est le
`storage_path` : **deux chemins de même empreinte sont deux fichiers** (même contenu importé sous `x.jpg` et
`x.jpeg`, ou par deux sociétés — `UNIQUE (company_id, file_hash)` ne l'interdit pas), tous deux inclus. Sont des
**anomalies**, classées `mismatch` (AC3) sans lecture du fichier : (i) un `storage_path` dont le préfixe hexadécimal
n'est pas l'empreinte déclarée **par la même ligne** ; (ii) un même `storage_path` déclaré avec deux empreintes
différentes par deux lignes (aucune n'est choisie) ; (iii) un `storage_path` qui ne satisfait pas
`is_safe_storage_path` **et** `^[0-9a-f]{64}\.[a-z0-9]{1,8}$` (C-31-1-28).

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
"filesNotIncluded": [ { "path": "<sha>.<ext>", "reason": "missing" | "altered" | "mismatch" } ]
```

(champs `#[serde(default)]` : un manifeste 1 se relit avec deux listes vides). Un fichier **absent** du dossier des
documents, **altéré** (empreinte différente) ou en **anomalie** (AC1) n'empêche pas la sauvegarde : il est listé dans
`filesNotIncluded` et **n'est pas** écrit dans l'archive (C-31-1-4). Les fichiers du dossier que la base ne référence pas
ne sont pas emportés. **Aucun chemin n'est joint au dossier des documents avant validation** : la lecture passe par une
fonction de `document_storage` (`open_document(dir, storage_path) -> Result<File, ReadDocumentError>`, qui applique
`is_safe_storage_path`, rendue `pub(crate)`, comme `read_document` le fait déjà) — jamais `documents_dir.join(…)` dans
`export.rs`. L'entrée dossier `files/` est écrite **même sans aucun fichier** (le contrôle de présence de l'import,
`import.rs:98-102`, l'exige pour le format 1 et le 2).

**AC4 — L'export n'assemble plus l'archive en mémoire.** `build_keshbackup` écrit l'archive dans un `Write + Seek`
fourni par l'appelant (`build_keshbackup_into`) ; les fichiers sont lus **un par un** (`spawn_blocking`) et copiés dans
l'entrée, jamais tous en mémoire. `GET /admin/full-export` l'écrit dans un
`tempfile::SpooledTempFile` de seuil `KESH_ADMIN_EXPORT_INMEM_MB` (sémantique de la variable inchangée : en mémoire
jusqu'au seuil, sur disque au-delà), débordant **dans `KESH_ADMIN_BACKUP_DIR`** (`spooled_tempfile_in`, `tempfile` 3.27
— monté, contrairement au `/tmp` du conteneur), puis le diffuse. **Le dossier est créé s'il n'existe pas** (`0700`,
récursif) par la fonction que `write_pre_import_backup` utilise déjà (`routes/admin.rs:518-533`, factorisée en
`ensure_backup_dir`), avant d'ouvrir le temporaire — sur une machine neuve hors compose, il peut manquer. La sauvegarde pré-import l'écrit directement dans son
fichier `.partial` (`write_backup_file` devient générique sur la source, C-31-1-7). `tempfile` passe des
`[dev-dependencies]` aux `[dependencies]` de `kesh-api` (déjà au `Cargo.lock`). La primitive rend une erreur **typée**
qui conserve l'`io::Error` d'origine (`BackupWriteError { stage, source: io::Error }`) ; l'adaptation vers
`AppError::AdminPreImportBackupFailed` se fait chez l'appelant pré-import, pas dans la primitive (la 31-1c en a besoin
pour distinguer `disk_full`, AC30). La garde lexicale
`build_keshbackup_ne_lit_la_base_qu_une_fois` s'adapte au nouveau corps et reste exacte (le pool n'est lu que par
`export_installation(pool)`).

**AC5 — Ce que l'export rend, et ce qu'il dit.** La réponse de `GET /admin/full-export` porte deux en-têtes,
`X-Kesh-Backup-Files: <n inclus>` et `X-Kesh-Backup-Files-Not-Included: <n>` (exposés à l'écran par la 31-1b). L'audit
`admin.full_export` ajoute `file_count`, `files_bytes`, `files_not_included` au détail. Un fichier absent ou altéré est
journalisé (`warn!`, chemin et raison, jamais le contenu).

**AC6 — Import : formats acceptés.** `parse_and_verify` accepte `formatVersion` 1 **et** 2, refuse au-delà
(`InvalidBackupStructure`, message « formatVersion N non supporté (maximum 2) »). Format 1 : règle actuelle inchangée
(`files/` présent et vide). Format 2 : `files/` présent ; **bijection** entre les entrées `files/*` et
`manifest.files` (une entrée sans ligne, ou une ligne sans entrée → 400) ; chaque nom satisfait
`^[0-9a-f]{64}\.[a-z0-9]{1,8}$` **et** `is_safe_storage_path`, et son préfixe égale le `sha256` déclaré ; aucun
sous-dossier sous `files/`. Les chemins de `filesNotIncluded` satisfont la même expression (ils ne servent jamais de
chemin, mais un nom hors forme signale une archive forgée → 400) ; un chemin présent **à la fois** dans `files` et
dans `filesNotIncluded`, ou deux fois dans l'une des listes → 400.

**AC6b — Archive piégée : lectures bornées (les deux formats).** `parse_and_verify` ne lit plus aucune entrée sans
borne : `manifest.json` à travers `take(MANIFEST_MAX_BYTES + 1)` (64 Mio) ; chaque `data/<table>.ndjson` à travers
`take(TABLE_ENTRY_MAX_BYTES + 1)` (1 Gio, constante nommée, indépendante de la taille déclarée par l'en-tête ZIP, qui
est déclarative) ; dépassement → 400 `InvalidBackupStructure` (« entrée … trop grande »). Avant toute lecture :
`zip.len()` ≤ `MAX_ENTRIES` (1 000 000) ; un nom d'entrée en double → 400 ; pour le format 2, toute entrée hors de
l'ensemble attendu (`manifest.json`, `data/<table>.ndjson` des tables du manifeste, `files/`, `files/<nom>` de
`manifest.files`) → 400. Les entrées de `files/` restent bornées par `byteSize + 1` (AC7). C-31-1-29.

**AC7 — Import : empreintes vérifiées avant tout effet.** Chaque fichier de `files/` est lu en flux à travers un
`Sha256` avec une borne `byteSize + 1` (une entrée qui dépasse sa taille déclarée → 400, défense contre l'archive
piégée) ; une empreinte différente → 400 `InvalidBackupStructure` (« intégrité SHA-256 invalide pour le fichier … »).
Ces contrôles ont lieu **avant** le verrou, la sauvegarde de sécurité et tout `DELETE`, dans un `spawn_blocking`
(lecture disque et SHA-256 de plusieurs Gio : jamais sur un fil de l'exécuteur asynchrone). **Cohérence avec la base
restaurée — format 2 seulement** (un format 1 n'a ni `files` ni `filesNotIncluded` : la clause ne s'y applique pas,
AC6, AC10) : tout fichier de `manifest.files` doit être référencé par une ligne restaurée (`imported_supplier_invoices`
ou `invoices` du backup), avec la même empreinte (`file_hash` / `pdf_sha256`) — sinon 400 ; un fichier référencé par
le backup qui n'est ni dans `files` ni dans `filesNotIncluded` → 400 (sauvegarde incomplète ou falsifiée). Une colonne
de référence **absente** des `columnNames` d'une table du backup (`invoices.pdf_storage_path` n'existe pas avant la
migration `20261003000001`) vaut « aucune référence » pour cette table — ni `panic`, ni 400 ; idem pour le calcul de
`filesMissing` (AC9).

**AC8 — Import : les fichiers sont écrits avant de rendre la main.** Dans `run_backup_and_restore`, après la
sauvegarde de sécurité et **avant** le `DELETE` des tables : chaque fichier de l'archive est écrit sous
`KESH_DOCUMENTS_DIR` par la primitive factorisée de `document_storage` (temporaire, `fsync`, `rename`). Un fichier de
même nom déjà présent **et intact** est laissé (compté restauré) ; présent mais **altéré**, il est remplacé
atomiquement (journalisé `warn!`). Un échec d'écriture (dossier non inscriptible, disque plein) → la restauration est
**refusée**, la base n'est pas touchée (`AdminFullImportFailed`, 500, message « aucune donnée n'a été remplacée ») ;
les fichiers déjà écrits restent (ils sont nommés par leur contenu : inoffensifs, C-31-1-6). Aucun fichier du dossier
des documents n'est jamais supprimé par l'import. L'écriture des fichiers (`std::fs`, `fsync`) s'exécute en
`spawn_blocking`. Ce que l'ordre « fichiers avant la base » achète se teste par sa négation : écrire les fichiers
**après le `COMMIT`** laisserait, sur un refus d'écriture, une base déjà remplacée (M-a7).

**AC8b — L'import ne dépend pas de la connexion du client.** Une fois l'envoi reçu sur disque (AC11), le reste de
l'import (vérification, sauvegarde de sécurité, écriture des fichiers, restauration) s'exécute dans une tâche
`tokio::spawn` que le handler **attend** : si le client abandonne (délai du navigateur ou du proxy, onglet fermé), hyper
abandonne le futur du handler mais pas la tâche, qui va au bout (succès ou refus, audit et journal compris) — au lieu
d'une restauration interrompue au milieu de `restore_tables_in_tx`. Un `JoinError` (panique) rend 500 et est journalisé.
Garde lexicale dans `routes/admin.rs` (`mod tests`) : le corps de `full_import` contient `tokio::spawn(` et l'appel de
`run_backup_and_restore` se trouve **dans** la fermeture de ce `spawn` (C-31-1-26).

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
son doc-comment dit pourquoi. Les tests actuels de l'import (format 1, `build_test_backup`) restent verts sans
modification de leur assertion (C-31-1-31).

**AC11 — Import reçu sur disque.** `read_upload` crée au besoin `KESH_ADMIN_BACKUP_DIR` (`ensure_backup_dir`, AC4),
puis copie le champ `file` **par morceaux** (`field.chunk()`) dans un fichier temporaire **anonyme** de ce dossier
(`tempfile::tempfile_in` : sans nom dans le dossier, il disparaît avec le descripteur, même après un `kill -9` ou un
OOM — aucun résidu de plusieurs Gio ; C-31-1-33) ; `parse_and_verify` lit un `Read + Seek` (`ZipArchive<File>`), plus
un `&[u8]`. Les tables NDJSON restent lues en mémoire, bornées (AC6b). Dossier absent → créé ; dossier non inscriptible
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
(§ *Export/import d'installation*, l. ~1828) ; les phrases « ni dans le `.keshbackup` » / « sans les fichiers » / note
de bas de page #503 (l. ~1465, 1474, 1828, tableau l. ~1950-1963) **corrigées** ; ce qui n'est pas emporté (fichiers non
référencés, `filesNotIncluded`, `.env`, réglage de la sauvegarde automatique — ce dernier écrit par la 31-1c) ; le
volume, le débordement dans `KESH_ADMIN_BACKUP_DIR`, le nouveau défaut de `KESH_ADMIN_IMPORT_MAX_MB` (tableau l. ~783 et
§ *Variables d'environnement*) ; une sauvegarde de la 0.15.0 ne s'importe pas dans une 0.14.x (`formatVersion`).
`.env.example` : commentaire du plafond à 2048. PDF régénéré. Entrée `[Unreleased]` du CHANGELOG (`### Ajouté`).
`docs/manual/fr/user-manual.tex:1909-1910` (« n'emporte pas les documents ») corrigé.

**Proxy inverse** (C-31-1-25) : l'exemple Nginx (`admin-manual.tex:329-341`) gagne un bloc `location /api/v1/admin/`
avec `client_max_body_size 2048M;` (égal au plafond de Kesh, à relever avec lui), `proxy_read_timeout 3600s;`,
`proxy_send_timeout 3600s;` et `proxy_request_buffering off;` — le `client_max_body_size 50M` et le `proxy_read_timeout
300s` du bloc général refuseraient (413) ou couperaient l'import et l'export d'une sauvegarde avec pièces. Caddy
(l. ~360) et Traefik (l. ~377, délais de lecture des points d'entrée) : une phrase chacun sur les limites de taille et
de durée à vérifier pour la version utilisée ; Portail DSM (l. ~618) : idem, sans inventer de chemin d'écran non vérifié.
Volume : la sauvegarde pré-import contient désormais toutes les pièces, une copie à chaque import, jamais purgée ;
le dire, avec l'ordre de grandeur « taille du dossier des documents + base ».

**Inventaire clos des phrases à traiter** (grep des valeurs `503`, `512`, `n'emporte`, `sans les fichiers`, `aucun
binaire`, `files/`, `formatVersion`, relevé en P1 ; T-a8 le rejoue et traite tout site neuf) :

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
| `crates/kesh-api/src/admin_backup/export.rs` (commentaire « aucun binaire stocké en v0.2 », l. ~95) | réécrit |
| `crates/kesh-api/src/config.rs:249`, `:927-948` (doc et messages « défaut 512 ») ; `.env.example:186` | → 2048 |
| `website/roadmap.html`, brochure | laissés à la release, noté au Dev Agent Record |

### Partie b — `31-1b-restauration-installation-neuve`

**AC15 — Restaurer avant de configurer.** Sur l'assistant (`routes/onboarding/+page.svelte`), aux étapes **0 à 2**
(avant le choix démo / production), un lien secondaire **« Restaurer une sauvegarde »** mène à `/onboarding/restore`,
page neuve qui rend `AdminRestorePanel` (réutilisé, C-31-1-10) ; la garde de `routes/onboarding/+layout.ts` la laisse
passer comme `/onboarding`. Le lien n'apparaît que pour un **Administrateur** (le compte créé par `/setup` l'est) ; un
autre rôle ne le voit pas, et le serveur refuse de toute façon (403).

**AC16 — Après l'import, l'installation est celle de la sauvegarde.** Le panneau garde son comportement (déconnexion,
`/login`) ; reconnecté avec les identifiants **de la sauvegarde**, l'utilisateur arrive dans l'application, pas dans
l'assistant (`force_onboarding_done_if_eligible`, inchangé) — vérifié en E2E (AC20).

**AC17 — Ce que le fichier ne contient pas, dit avant.** `AdminBackupPanel` et `AdminRestorePanel` affichent un avis
permanent : « Le fichier ne contient pas la configuration du serveur (fichier `.env` : messagerie, secret de session,
dossiers…). Conservez-la à part : sur une nouvelle machine, recréez-la avant d'importer. » (clé
`admin-backup-not-included-env`, quatre locales).

**AC18 — Ce que l'export a omis, dit après.** `downloadFullExport` lit les en-têtes de l'AC5 et de l'AC12 : si
`X-Kesh-Backup-Files-Not-Included > 0`, avis d'avertissement « N pièces référencées n'ont pas pu être incluses (fichier
absent ou altéré sur le serveur) — voir le journal du serveur » ; si `X-Kesh-Backup-Exceeds-Import-Limit` est posé,
avis « Ce fichier dépasse la taille d'import autorisée (N Mio) : pour le restaurer, relevez `KESH_ADMIN_IMPORT_MAX_MB` ».
En-têtes absents (serveur ancien) : aucun avis.

**AC18b — L'export et l'import échappent au délai de 30 s du client HTTP** (C-31-1-25). `frontend/src/lib/shared/utils/api-client.ts`
arme un délai de 30 s (`REQUEST_TIMEOUT_MS`, l. 42) jusqu'à l'arrivée des en-têtes, et rejoue un `GET` jusqu'à quatre
fois sur `TIMEOUT` (`fetchWithRetry`, l. 329) ; or le serveur n'envoie les en-têtes de l'export qu'après avoir assemblé
toute l'archive, et ne répond à l'import qu'après la restauration — des minutes avec les pièces. `getBlob` et
`postFormData` gagnent un paramètre d'options `{ timeoutMs?: number; retry?: boolean }` (défauts inchangés : 30 s, rejeu
des méthodes idempotentes) ; une constante exportée `LONG_OPERATION_TIMEOUT_MS = 60 * 60_000` (une heure) ;
`downloadFullExport` et `uploadFullImport` l'emploient avec `retry: false` (la 31-1c fera de même pour « Sauvegarder
maintenant », AC32). Sur `TIMEOUT` ou `NETWORK_ERROR` de l'**import**, le panneau ne dit plus « L'état précédent a été
préservé » (il n'en sait rien) mais « La réponse du serveur n'est pas arrivée : l'import a pu aboutir. Reconnectez-vous
pour vérifier l'état de l'installation. » (clé `admin-restore-error-unknown-outcome`, quatre locales) ; le serveur, lui,
va au bout (AC8b). Tests Vitest de `api-client` (horloge simulée) : un appel par défaut est abandonné à 30 s ; un appel
avec `LONG_OPERATION_TIMEOUT_MS` ne l'est pas à 30 s ni à 59 min, l'est à 60 min ; un `getBlob` avec `retry: false` n'est
**pas** rejoué sur `TIMEOUT` (un seul `fetch`) ; `downloadFullExport` et `uploadFullImport` passent bien l'option
(espion sur `apiClient`). Test du panneau : `TIMEOUT` → le message d'issue inconnue, pas « état préservé ».

**AC19 — Ce que la restauration a rendu.** Le succès de l'import affiche, avant la déconnexion, `filesRestored` et,
si `filesMissing > 0`, un avertissement « N pièces référencées sont introuvables après la restauration : elles
répondront « document indisponible » ; restaurez le dossier des documents depuis une autre sauvegarde ». Le délai avant
déconnexion laisse lire l'avis (le panneau attend un clic **« Se reconnecter »** quand un avertissement est affiché,
C-31-1-11).

**AC20 — E2E de bout en bout (`frontend/tests/e2e/backup-restore-new-install.spec.ts`).** (1) `seedTestState('with-company')`,
connexion ; un document fournisseur importé par l'inbox (fixture synthétique `spc_e2e_invoice.png` passée par
`uniqueCopy` de `helpers/inbox.ts` — des octets propres au spec, pour que la suppression de l'étape 3 ne retire pas le
fichier d'une autre spec) et une facture validée dont le PDF est figé par un téléchargement ; (2) export par l'écran, fichier
enregistré par le runner ; (3) `seedTestState('setup-required')` **puis suppression des deux fichiers** du dossier des
documents (`DOCUMENTS_DIR` de `helpers/inbox.ts` ; garde bruyante : chaque fichier doit exister avant suppression et
ne plus exister après — sinon le spec échoue, il ne se tait pas) ; (4) `/setup` (nouvel admin), assistant, lien
**« Restaurer une sauvegarde »**, import du fichier, confirmation ; (5) avis de l'AC19 avec « 2 pièces restaurées »,
reconnexion avec les identifiants de la sauvegarde ; (6) l'application s'ouvre (pas l'assistant) ; le document
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
(g) fin d'import `filesMissing = 0` (sans bouton *Se reconnecter*) ; (h) fin d'import `filesMissing > 0` avec le bouton
*Se reconnecter* ; (i) message d'issue inconnue de l'import (AC18b).

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
`run_once(&state, now, Trigger::Schedule)`. Une erreur de lecture est journalisée et le tour suivant réessaie ; une
panique dans un tour n'arrête pas la boucle (le tour s'exécute dans un `tokio::spawn` dont le `JoinError` est
journalisé, C-31-1-18).

**AC27 — Une seule sauvegarde à la fois.** `AppState` gagne `backup_lock: Arc<tokio::sync::Mutex<()>>` (défaut dans
`new_for_tests` ; **neuf sites** mis à jour : les huit littéraux `AppState { … }` — `main.rs:355`,
`middleware/auth.rs:267`, `tests/password_recovery_e2e.rs:127`, `tests/invoice_frozen_pdf_e2e.rs:102`,
`tests/invoice_send_email_e2e.rs:103,1034,1147`, `tests/setup_admin_e2e.rs:81` — et le `Self { … }` de
`AppState::new_for_tests`, `lib.rs:105` ; recompter au dev par `grep -rnE "AppState \{" --include=*.rs crates`). La
sauvegarde automatique le prend en `try_lock` (pris → tour sauté, rien d'écrit en base, journal `info!` « tour sauté :
une autre sauvegarde est en cours », nouveau contrôle 5 minutes plus tard) ; « Sauvegarder maintenant », l'export
manuel et l'import le prennent en `lock().await`. **Fenêtre de détention** (C-31-1-30) : l'import le prend **après** la
réception de l'envoi et `parse_and_verify` (un client lent ne bloque pas les sauvegardes pendant l'envoi) et le garde
jusqu'à la fin de la restauration (sauvegarde de sécurité, écriture des fichiers, restauration) ; l'export manuel le
prend pour l'**assemblage** de l'archive dans son temporaire et le **rend avant la diffusion** (un téléchargement lent ne
bloque rien). Puis la sauvegarde **réserve** son tour en base : `UPDATE _kesh_backup_schedule SET
last_attempt_at = ? WHERE id = 1 AND last_attempt_at <=> ?` (valeur lue) ; `rows_affected = 0` → un autre processus l'a
prise, abandon silencieux (`debug!`). L'instantané de la base (Story 28-3) et l'immuabilité des fichiers nommés par
leur contenu (P-3) garantissent une archive cohérente même pendant une saisie.

**AC28 — Écrire la sauvegarde.** `run_once` crée `KESH_ADMIN_BACKUP_DIR/auto/` (`0700`, `DirBuilder` récursif comme
`write_pre_import_backup`), supprime d'abord les `kesh-auto-*.keshbackup.partial` qu'il y trouve (les siens, laissés par
un arrêt brutal — le verrou garantit qu'aucun n'est en cours), puis écrit `kesh-auto-<AAAAMMJJ>T<HHMMSS>Z.keshbackup`
(UTC) par `build_keshbackup_into` dans le `.partial` de `write_backup_file` (`0600`, renommage, jamais d'écrasement).
Avant de la tenir pour écrite, `run_once` **la relit** : ouverture `ZipArchive`, lecture du `manifest.json` (annuaire
central et manifeste lisibles) — un échec vaut `failed`, le fichier fautif est supprimé, la rétention ne tourne pas.
Succès : `last_success_at`, `last_success_file`, `last_success_bytes`, `last_files_not_included`, et
`last_status = 'ok'` si `filesNotIncluded` est **vide** (alors `last_complete_file` = ce fichier), **`'partial'`** sinon
— une sauvegarde à qui il manque des pièces n'est pas un succès pour le bandeau (AC31) ni pour la rétention (AC29)
(C-31-1-27) ; journal `info!` (`warn!` pour `partial`, avec le nombre de pièces).

**AC29 — Rétention.** Après un succès (`ok` ou `partial`) **seulement** : les fichiers **réguliers** de `auto/`
(`symlink_metadata`, un lien n'est jamais suivi ni supprimé) dont le nom satisfait exactement
`^kesh-auto-\d{8}T\d{6}Z\.keshbackup$` sont triés par nom (= par date), et les plus anciens au-delà de
`retention_count` sont supprimés ; **jamais** un autre fichier (`kesh-pre-import-*`, fichier déposé à la main, nom
voisin), jamais la sauvegarde qui vient d'être écrite, **jamais `last_complete_file`** — la dernière sauvegarde
complète survit à une suite de sauvegardes `partial` (dossier des documents démonté après une mise à jour du NAS : sans
cette règle, `retention_count` jours suffiraient à ne plus garder aucune sauvegarde portant les pièces). Elle compte
alors en plus des `retention_count` conservées. Un échec de suppression est journalisé et n'efface pas le succès.
Un échec de sauvegarde (`failed`, `disk_full`, `insufficient_space`) ne supprime **rien**.

**AC30 — Disque plein, avant et pendant.** Avant d'écrire : espace libre du système de fichiers de `auto/`
(`rustix::fs::statvfs`, dépendance directe ajoutée — `rustix` 1.1.4 est au `Cargo.lock` ; C-31-1-19) comparé à
`last_success_bytes × 1,2 + 256 Mio` (ou 256 Mio s'il n'y a jamais eu de succès) : insuffisant → rien n'est écrit,
`last_status = 'insufficient_space'`, `warn!` avec les deux nombres. Pendant : toute erreur d'écriture supprime le
`.partial` (déjà fait par `write_backup_file`) ; la classification se fait sur l'`io::Error` conservée par la primitive
(AC4), jamais sur un texte : fonction pure `classify_write_error(&io::Error) -> FailureStatus`, `disk_full` si
`kind()` vaut `ErrorKind::StorageFull` (ENOSPC) ou `ErrorKind::QuotaExceeded` (EDQUOT, quota d'un dossier partagé
Synology ; stables sur la chaîne 1.96.1 du dépôt), ou si `raw_os_error()` vaut 28 ou 122 ; `'failed'` sinon
(C-31-1-32). **La rétention ne tourne pas**, les sauvegardes existantes restent. Le trait d'injection de l'AC36
(`BackupEnv`, qui remplace `FreeSpace`) porte deux méthodes : `free_bytes(dir)` et `wrap_writer(File) -> Box<dyn Write +
Seek + Send>` — l'implémentation de production rend le fichier tel quel ; celle de test rend un écrivain qui échoue avec
`ErrorKind::StorageFull` après N octets. Le seuil protège surtout les
voisins du disque (la base elle-même, sur un NAS, partage souvent le volume).

**AC31 — Échec signalé.** Tout `last_status` autre que `ok` est visible : (a) dans le panneau (AC33), bandeau d'erreur
(`role="alert"`) qui dit la cause en clair, l'heure de la dernière tentative et de la dernière réussite ; (b) dans
l'application, pour un **Administrateur** seulement, un bandeau discret sous l'en-tête (`routes/(app)/+layout.svelte`)
« La dernière sauvegarde automatique a échoué — Administration → Sauvegarde complète » quand `last_status` est un
échec (`failed`, `insufficient_space`, `disk_full`) ; « La dernière sauvegarde automatique est incomplète : N pièces
n'ont pas pu être incluses — … » quand il vaut `partial` ; **ou** quand la sauvegarde est activée et que
`last_success_at` est **non nul** et plus vieux que **deux** intervalles (« aucune sauvegarde automatique réussie
depuis le … ») — une sauvegarde qui cesse de tourner sans échouer (tâche morte) se voit aussi (C-31-1-20). Un
`last_success_at` **nul** n'allume pas ce second cas (sinon le bandeau s'allumerait à chaque installation et pendant
les cinq minutes qui suivent chaque mise à jour, C-31-1-34) : une sauvegarde jamais tentée se lit dans le panneau
(« jamais tentée »), une jamais réussie mais tentée se voit par le premier cas. **Lecture** : le layout lit
`GET backup-schedule` à son chargement puis toutes les 30 minutes tant que l'application est ouverte, et après chaque
action du panneau ; un échec de cette lecture n'affiche **aucun** bandeau et ne casse pas le layout (pas de toast). Un
journal `error!` accompagne chaque échec. Pas d'entrée d'audit par sauvegarde réussie (bruit quotidien) ; l'état vit
dans la table.

**AC32 — Routes (Administrateur, refusées aux clés d'API).** Sous `admin_routes` (RBAC `require_admin_role`), chaque
handler appelle `ensure_not_pat` :
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
  fin, rend l'état ; verrou pris par une autre sauvegarde → 409 `BACKUP_IN_PROGRESS` (code neuf). Le frontend l'appelle
  avec `LONG_OPERATION_TIMEOUT_MS`, sans rejeu (AC18b) ;
- `GET /api/v1/admin/backups` → liste des fichiers **réguliers** (`symlink_metadata` : un lien n'est jamais listé)
  `kesh-auto-*.keshbackup` de `auto/` et `kesh-pre-import-*.keshbackup` de `KESH_ADMIN_BACKUP_DIR` dont le nom satisfait
  l'expression de son `kind` (`auto` : `^kesh-auto-\d{8}T\d{6}Z\.keshbackup$` ; `pre-import` : celle du nom écrit par
  `write_pre_import_backup`, relevée au code au dev) — nom, `kind`, taille, date de modification, plus récents en tête ;
  aucun autre fichier n'est listé (pas de `kind = other`, C-31-1-35) ; les `.partial` exclus ;
- `GET /api/v1/admin/backups/{kind}/{name}` → téléchargement en flux ; `name` validé par l'expression de son `kind`
  **puis** retrouvé dans la liste du dossier (jamais concaténé tel quel : pas de traversée ; fichier régulier
  seulement) ; 404 sinon ; audit best-effort `admin.backup_downloaded` (nom, taille).
`audit_route_registry.rs`, `audit_label_registry.rs` et les libellés d'audit (quatre locales) suivent ;
`tests/admin_pat_denied_e2e.rs` (`ADMIN_COUPLES`, l. 53, et le compte « 28 constructeurs », l. 46) gagne les cinq couples
neufs — dont deux à paramètres de chemin. `docs/api-external.md` : rien à ajouter **pour ces routes**, qui relèvent de
la fermeture générale des routes d'administration aux clés (§ 4) — vérifié, à écrire au Dev Agent Record (le fichier
change par ailleurs à la 31-1a, AC14).

**AC33 — L'écran.** `AdminBackupPanel` gagne une section **« Sauvegarde automatique »** : interrupteur *Activée*,
fréquence (*Toutes les 6 heures* / *12 heures* / *Chaque jour* / *Chaque semaine*), *Nombre de sauvegardes conservées*
(1–60), *Enregistrer* ; état : dernière réussite (date locale, taille lisible, nombre de pièces non incluses s'il y en
a — statut `partial` dit « incomplète », en avertissement), dernière sauvegarde complète si elle diffère, dernière
tentative et son statut, prochaine prévue ; bouton **« Sauvegarder maintenant »** (désactivé pendant
l'exécution) ; liste **« Sauvegardes sur le serveur »** (nom, type, taille, date, *Télécharger*). Rappel permanent : les
sauvegardes du serveur sont dans le dossier de sauvegarde de l'installation (`./backup/auto` avec les compose fournis),
sur le **même disque** que Kesh — à copier ailleurs (Hyper Backup) ; elles contiennent des secrets ; chacune pèse à peu
près la base **plus** le dossier des documents (`retention_count + 1` copies dans `auto/`, plus une par import dans le
dossier de sauvegarde, jamais purgée). Les réglages de la
sauvegarde automatique ne voyagent pas dans le `.keshbackup` (ils appartiennent à l'installation, comme le `.env`).

**AC34 — Télécharger une sauvegarde du serveur** remplace, pour la sauvegarde pré-import, le geste SSH du manuel
(l. ~1902-1907, `sudo cp … chown`) : le manuel donne désormais le téléchargement par l'écran comme voie normale, SSH
en secours.

**AC35 — Concurrence avec l'import, vérifiée.** Test d'intégration : pendant un import (verrou tenu par un test au
moyen d'un `backup_lock` pris à la main), `run_once(Trigger::Schedule)` saute sans écrire ni modifier la table —
l'appel est enveloppé dans `tokio::time::timeout(Duration::from_secs(5), …)` et le test exige `is_ok()` (« la tâche doit
sauter, pas attendre ») : sous la mutation M-c6 il **échoue** en 5 s au lieu de pendre le gate (`.config/nextest.toml`
n'a pas de `slow-timeout`) ; `POST …/run` rend 409 `BACKUP_IN_PROGRESS` (même enveloppe). Après la libération,
`run_once` écrit.

**AC36 — Tests d'intégration de la sauvegarde automatique** (`crates/kesh-api/tests/admin_auto_backup_e2e.rs`) : succès
(fichier `0600`, dossier `0700`, archive importable : `parse_and_verify` vert, fichiers inclus) ; rétention (8
sauvegardes déposées + une neuve, `retention_count = 3` → restent les 3 plus récentes, un `kesh-pre-import-x` et un
`autre.keshbackup` intacts) ; espace insuffisant (seuil injecté par paramètre de test : la fonction de mesure est
passée à `run_once` derrière le trait `BackupEnv`, AC30, C-31-1-19) → `insufficient_space`, aucun fichier ; écriture
impossible (dossier `auto` remplacé par un fichier) → `failed`, aucune suppression ; **disque saturé pendant
l'écriture** (écrivain de test `StorageFull` après N octets) → `disk_full`, `.partial` supprimé, aucune suppression de
rétention (8 sauvegardes déposées restent 8) ; `classify_write_error` unitaire (`StorageFull`, `QuotaExceeded`, code
brut 28, `PermissionDenied` → `failed`) ; **sauvegarde incomplète** (une pièce référencée retirée du dossier des
documents) → `partial`, `last_complete_file` inchangé, et sur une suite de `retention_count + 2` sauvegardes `partial`
la dernière complète **reste** ; une sauvegarde complète suivante la remplace dans `last_complete_file` et la rétention
retrouve son compte ; archive relue (AC28) : un écrivain de test qui tronque l'archive → `failed`, fichier supprimé ;
`version` inchangée par `claim_attempt`/`record_*` (un `PUT` avec la version lue avant le tour → 200) ; réservation perdue (une autre
connexion avance `last_attempt_at` entre la lecture et l'`UPDATE` — observateur de test) → rien n'est écrit ;
`.partial` résiduel supprimé ; liste et téléchargement : un lien symbolique `kesh-auto-…keshbackup` vers un autre
fichier n'est ni listé ni servi ; routes : 403 non-admin, 403 clé d'API, 400 domaine, 409 version, téléchargement 404 sur
`../x`, sur un nom hors expression et sur un nom absent.

**AC37 — Vitest.** Panneau : rendu de l'état (ok, `partial`, échec de chaque type, jamais tenté, désactivé),
enregistrement (payload exact, conflit 409 → rechargement proposé), « Sauvegarder maintenant » (désactivé pendant, 409
affiché, appel avec `LONG_OPERATION_TIMEOUT_MS` sans rejeu), liste et téléchargement ; bandeau du layout : affiché pour
Admin seulement, dans les trois cas de l'AC31 (échec, `partial`, aucune réussite depuis deux intervalles), absent quand
`ok` et récent, absent quand `last_success_at` est nul et `last_status` nul, absent quand la lecture échoue.

**AC38 — E2E (`frontend/tests/e2e/backup-schedule.spec.ts`).** Admin : changer la fréquence et la rétention,
enregistrer, recharger la page → valeurs gardées ; « Sauvegarder maintenant » → état « réussie » et une ligne `auto` dans
la liste ; télécharger cette ligne → fichier non vide dont le manifeste a `formatVersion: 2` ; réglages remis aux défauts
en fin de spec (la table survit aux `seedTestState`). Comptable : le menu n'offre pas la page, l'URL directe
**redirige vers `/`** (garde de route existante, `routes/(app)/admin/backup/+page.ts:15`, `redirect(302, '/')`) — le
refus 403 de l'API est vérifié côté Rust (AC36).

**AC39 — Manuel et CHANGELOG.** `admin-manual.tex` : sous-section **« La sauvegarde automatique »** (activée par défaut
à la mise à jour, fréquence, rétention et protection de la dernière sauvegarde complète, statut « incomplète »,
emplacement `./backup/auto`, espace requis et refus, volume (base + documents par copie), état et bandeau, *Sauvegarder
maintenant*, téléchargement, horloge déréglée, ce qu'elle ne remplace pas : la copie hors du disque) ; tableau *Quelle méthode choisir ?*
(l. ~1950-1963) mis à jour ; *Stratégie recommandée* (l. 1461) ; sauvegarde pré-import (téléchargement, AC34) ; mise à
jour vers la 0.15.0 (migration non breaking, retour à la 0.14.0 par la sauvegarde d'avant). PDF régénéré. CHANGELOG
`[Unreleased]`. README § Fonctionnalités (ligne *Export/import d'installation*, l. 49).

**AC40 — Captures (thème clair)** de chaque état touché, dans `/home/gcorbaz/devel/kesh-gate-logs/311c-captures/`,
citées au Dev Agent Record et au prompt de revue : section *Sauvegarde automatique* (a) jamais tentée, (b) désactivée,
(c) réussie, (d) `partial`, (e) `failed`, (f) `insufficient_space`, (g) `disk_full` ; (h) « Sauvegarder maintenant » en
cours ; (i) 409 `BACKUP_IN_PROGRESS` affiché ; (j) conflit de version (rechargement proposé) ; (k) erreur de validation ;
(l) liste des sauvegardes remplie ; (m) liste vide ; bandeau du layout (n) échec, (o) `partial`, (p) « aucune
sauvegarde réussie depuis … » ; (q) la section à largeur étroite (390 px). Les états qu'on ne peut atteindre par
l'écran (statuts d'échec) se posent en base de test par `UPDATE` avant la capture, ce que le Dev Agent Record dit.

## Tasks / Subtasks

### 31-1a

- [ ] T-a0 **Avant tout changement de code**, sur `44ef2079` : générateur `#[ignore]` et fixture
  `tests/fixtures/keshbackup-0.14.0-format1.keshbackup` (AC10), commit dédié, empreinte au Dev Agent Record.
- [ ] T-a1 Tests rouges : AC2 (deux gardes), AC6/AC7 (unitaires de `parse_and_verify` : format 2 valide, bijection,
  nom invalide, préfixe ≠ sha, taille dépassée, sous-dossier, format 3 refusé, chemin dans les deux listes, chemin de
  `filesNotIncluded` hors forme), AC6b (manifeste trop grand, table trop grande, nom en double, entrée inattendue en
  format 2), AC10 (fixture réelle), AC13 (intégration).
- [ ] T-a2 `kesh-db` : `ReferencedFile`, lecture dans `export_installation_in` (AC1) ; tests de l'instantané étendus
  (une facture figée **après** l'ouverture de l'instantané n'apparaît pas dans `referenced_files` — observateur).
- [ ] T-a3 `manifest.rs` : version 2, `files`, `filesNotIncluded` (`serde(default)`).
- [ ] T-a4 `export.rs` : `build_keshbackup_into<W: Write + Seek>`, lecture et vérification des fichiers (AC3), méta
  étendue (AC5) ; `routes/admin.rs` : `SpooledTempFile` (AC4), en-têtes (AC5, AC12), audit.
- [ ] T-a4b Tests de l'export (≥ 8, rouges d'abord) : fichier `missing`, `altered`, `mismatch` (préfixe ≠ empreinte de la
  ligne ; même chemin, deux empreintes) → listé dans `filesNotIncluded`, absent de l'archive (c'est le test qui tue M-a2) ;
  deux chemins de même empreinte → tous deux inclus ; `storage_path = "/etc/hostname"` et `"../x.pdf"` en base →
  `mismatch`, aucune lecture hors du dossier ; entrée dossier `files/` présente sans aucun fichier ; manifeste 1 relu
  avec deux listes vides ; en-têtes `X-Kesh-Backup-Files*` (valeurs exactes) et champs d'audit (AC5) ;
  `X-Kesh-Backup-Exceeds-Import-Limit` posé / non posé (plafond de test réduit) (AC12) ; débordement du `SpooledTempFile`
  **dans** `KESH_ADMIN_BACKUP_DIR` (seuil 0, dossier de test inspecté pendant l'assemblage par un observateur, ou
  vérification qu'aucun fichier n'est créé dans `std::env::temp_dir()`) ; dossier de sauvegarde absent → créé (AC4).
- [ ] T-a5 `document_storage.rs` : cœur d'écriture atomique factorisé ; `restore_document(dir, storage_path, bytes,
  expected_sha)` (AC8) ; `write_backup_file` générique sur la source (AC4).
- [ ] T-a6 `import.rs` : lecture `Read + Seek`, format 2 (AC6, AC7) ; `routes/admin.rs` : `read_upload` sur disque
  (AC11), écriture des fichiers avant le `DELETE` (AC8), import dans une tâche `spawn` attendue (AC8b), comptes et
  audit (AC9).
- [ ] T-a6b Tests de l'import (≥ 7, rouges d'abord) : cohérence AC7 en format 2 (fichier de `manifest.files` non
  référencé ; empreinte ≠ `file_hash`/`pdf_sha256` ; fichier référencé ni dans `files` ni dans `filesNotIncluded` →
  400) ; la même absence en format 1 → **200** (fixture réelle) ; table `invoices` sans colonne `pdf_storage_path` →
  ni `panic` ni 400, `filesMissing` calculé sans elle ; refus d'écriture (dossier des documents non inscriptible) →
  500 et base intacte ; fichier présent altéré → remplacé ; aucun temporaire laissé dans `KESH_ADMIN_BACKUP_DIR` après
  succès **et** après échec (AC11) ; dossier de sauvegarde absent → créé, import 200 ; garde lexicale de l'AC8b.
- [ ] T-a7 `config.rs` : défaut 2048 (AC12) et son test ; `.env.example`.
- [ ] T-a8 Manuels, `docs/api-external.md`, proxy inverse, CHANGELOG (AC14) ; grep des valeurs (`503`, `512`,
  `files/`, `n'emporte`, `sans les fichiers`, `aucun binaire`, `formatVersion`, `50M`, `300s`) sur tout le dépôt — code,
  doc-comments, i18n, manuels (`.tex` **et** PDF aplati), `docs/`, `website/` — confronté à l'inventaire clos de l'AC14 :
  chaque site de l'inventaire traité, chaque site **neuf** ajouté au tableau du Dev Agent Record avec son traitement
  (les phrases de `website/roadmap.html` et de la brochure sont laissées à la release, à noter au Dev Agent Record).
- [ ] T-a9 Mutations (§ Mutations), gate complet (`kesh-db` touché), E2E complet au dernier commit de code.

### 31-1b

- [ ] T-b1 Vitest rouges : lien de l'assistant (Admin seulement, étapes 0–2), avis `.env`, avis de l'export (trois cas
  d'en-têtes), avis de fin d'import (`filesMissing` 0 / > 0, bouton *Se reconnecter*).
- [ ] T-b2 `routes/onboarding/restore/+page.svelte`, lien sur `+page.svelte` (AC15).
- [ ] T-b3 `admin-backup.api.ts` (en-têtes), `AdminBackupPanel`, `AdminRestorePanel` (AC17-AC19).
- [ ] T-b4 i18n ×4 (AC22).
- [ ] T-b5 E2E AC20 ; captures (AC21).
- [ ] T-b6 Manuel (AC21), PDF.
- [ ] T-b7 `api-client.ts` : options `{ timeoutMs, retry }` de `getBlob` et `postFormData`,
  `LONG_OPERATION_TIMEOUT_MS` ; `downloadFullExport`, `uploadFullImport` ; message d'issue inconnue de l'import (i18n ×4) ;
  tests Vitest rouges d'abord (AC18b).

### 31-1c

- [ ] T-c1 Migration, réinjection de la ligne singleton par `scripts/regen-test-schema.sh` et garde
  `squash_seeds_the_backup_schedule_row`, squash régénéré, audit P5 (trois sites), P6, P7, inventaire (AC23, AC24).
- [ ] T-c2 `kesh-db` : dépôt `backup_schedule` (`get`, `update_settings` avec version, `claim_attempt`, `record_*`).
- [ ] T-c3 `kesh-api/src/auto_backup.rs` : `is_due`, `run_once` (relecture de l'archive, statut `partial`,
  `last_complete_file`), `spawn`, `BackupEnv`, `classify_write_error` (AC25-AC30) ; `AppState.backup_lock` aux neuf
  sites (AC27) ; branchement dans `main.rs`.
- [ ] T-c4 Routes et registres (AC32 : `audit_route_registry`, `audit_label_registry`, `admin_pat_denied_e2e`
  `ADMIN_COUPLES`), libellés d'audit ×4.
- [ ] T-c5 Frontend : panneau (AC33), bandeau (AC31), API ; Vitest (AC37).
- [ ] T-c6 Tests d'intégration (AC35, AC36), E2E (AC38), captures (AC40).
- [ ] T-c7 Manuel, CHANGELOG, README (AC34, AC39).

## Dev Notes

### Tests (récapitulatif, rouges d'abord)

| Sous-story | Rust | Vitest | E2E |
|---|---|---|---|
| 31-1a | gardes AC2 (2), `parse_and_verify` format 2 et lectures bornées (≥ 12, T-a1), instantané (1), export (≥ 8, T-a4b), import et cohérence (≥ 7, T-a6b, dont la garde lexicale de l'AC8b), intégration AC13 (2), AC10 sur la fixture réelle (1) | — | aucune spec neuve (l'écran ne change pas) ; suite complète au dernier commit de code |
| 31-1b | — | ≥ 13 (dont ≥ 5 pour `api-client` et le message d'issue inconnue, AC18b) | `backup-restore-new-install.spec.ts` |
| 31-1c | migration et squash (AC23, AC24), `is_due` (≥ 6), `classify_write_error` (≥ 4), intégration AC35/AC36 | ≥ 12 | `backup-schedule.spec.ts` |

Le délai du client (AC18b) n'est **pas** prouvé par un E2E : les fixtures E2E pèsent quelques Ko et s'exportent en
moins d'une seconde, si bien qu'un E2E vert ne dirait rien du délai. Il l'est par les tests Vitest de `api-client` à
horloge simulée, qui figent les valeurs (30 s par défaut, une heure pour les appels longs, aucun rejeu).

### Gardes existantes qui changent de sens ou rougiront

- `import.rs` tests `parse_rejects_future_format_version` (forge `Some(2)` → désormais **valide** : passer à 3) et
  `parse_rejects_non_empty_files_dir` (vaut encore pour le format 1 : forcer `formatVersion: 1`).
- `manifest.rs` `manifest_json_has_canonical_shape` (`formatVersion` 1 → 2, `files` présent).
- `export.rs` `build_keshbackup_ne_lit_la_base_qu_une_fois` (corps renommé ; garder la garde exacte).
- `routes/admin.rs` garde `avant_sauvegarde_branchee_aux_appels_de_l_import` (compte « deux `build_keshbackup` » —
  le nom change).
- `admin_full_export_e2e.rs:477` et `:497` (`admin_export_inmem_mib = 0`, chemin de diffusion), `:254`, `:274`
  (structure du ZIP, `files/`), **`:296`** (`assert_eq!(manifest["formatVersion"], 1)` → 2) ; `admin_backup_e2e.rs`
  (structure du ZIP : `files/` vide → vide **seulement** sans fichier référencé).
- **`admin_full_import_e2e.rs:489-493`** (forge `formatVersion = 2` et attend 400 : devient valide → forger 3) ; son
  constructeur de backup (l. ~274, `add_directory("files/")`) reste un format 1.
- `config.rs` : tests du défaut 512, littéraux `admin_import_max_mib: 512` des `Config` de test (l. 513, 1626), doc
  (l. 249) et messages (l. 927-948) ; `.env.example:186` ; `configuration_transmise.rs` (variables inchangées, vérifier).
- `routes/admin.rs` : la garde lexicale neuve de l'AC8b ; les tests de `write_backup_file` (type d'erreur typé, AC4).
- `tests/admin_pat_denied_e2e.rs` (`ADMIN_COUPLES`, « 28 constructeurs ») — 31-1c, cinq couples neufs.
- `scripts/regen-test-schema.sh` et `tests/test_schema_guard.rs` (ligne singleton, 31-1c).
- `backup.rs:1763` `tables_sauvegardees_innodb_et_toutes_exportees` : relu (itère sur les tables attendues, non
  affecté par `referenced_files` ; à étendre seulement si l'implémentation y ajoute la lecture).
- Les **neuf** sites `AppState { … }` / `Self { … }` (AC27, 31-1c).
- `api-client.test.ts` (ou équivalent) et `AdminRestorePanel.test.ts` (message `admin-restore-error-generic` sur
  `TIMEOUT`) — 31-1b.
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
| M-a4 | AC6 : ne pas exiger la bijection (entrée sans ligne acceptée) | `parse_and_verify` entrée orpheline |
| M-a5 | AC7 : comparer à `meta.sha256` d'un autre fichier / sauter la vérification | `parse_and_verify` empreinte fausse |
| M-a6 | AC7 : supprimer la borne `byteSize + 1` | entrée plus grande que déclarée |
| M-a7 | AC8 : écrire les fichiers **après le `COMMIT`** (reformulée en P1 : « après le `DELETE` » restait dans la transaction, donc annulée par l'échec — mutation équivalente) | refus d'écriture : la base doit rester intacte (sous la mutation, elle est déjà remplacée) |
| M-a8 | AC8 : ignorer l'erreur d'écriture | idem (200 au lieu de 500) |
| M-a9 | AC9 : `filesMissing` calculé sur le backup au lieu de la destination | AC13 format 1 (`filesMissing = 2`) |
| M-a10 | AC6 : accepter `formatVersion` 3 | format 3 refusé |
| M-a11 | AC1 : lire les fichiers référencés **hors** instantané (sur le pool) | garde lexicale + test de l'observateur |
| M-a12 | AC3 : `documents_dir.join(storage_path)` sans `is_safe_storage_path` | T-a4b `"/etc/hostname"`, `"../x.pdf"` → `mismatch` |
| M-a13 | AC1 : classer `mismatch` deux chemins de même empreinte | T-a4b « deux chemins, une empreinte → deux inclus » |
| M-a14 | AC6b : retirer la borne `take` des tables `data/` | T-a1 « table trop grande » |
| M-a15 | AC7 : appliquer la cohérence fichier ↔ ligne au format 1 | AC10, fixture réelle (200 attendu) |
| M-a16 | AC8b : appeler `run_backup_and_restore` hors du `tokio::spawn` | garde lexicale de l'AC8b |
| M-a17 | AC4/AC11 : ne pas créer le dossier de sauvegarde | T-a6b « dossier absent → 200 » |
| M-b1 | AC15 : lien visible pour un non-Admin | Vitest du lien |
| M-b2 | AC19 : déconnexion immédiate malgré `filesMissing > 0` | Vitest *Se reconnecter* |
| M-b3 | AC18 : ignorer l'en-tête `Not-Included` | Vitest avis export |
| M-b4 | AC18b : `downloadFullExport` sans l'option longue | Vitest (espion : option passée ; abandon à 30 s) |
| M-b5 | AC18b : `retry: false` ignoré par `fetchWithRetry` | Vitest « un seul `fetch` sur `TIMEOUT` » |
| M-b6 | AC18b : `TIMEOUT` de l'import → message « état préservé » | Vitest du panneau |
| M-c1 | AC25 : `≥` → `>` sur l'intervalle | `is_due` à l'égalité |
| M-c2 | AC25 : retirer la clause `last_attempt_at` | `is_due` « échec il y a 10 min » |
| M-c3 | AC29 : rétention sur `kesh-*` au lieu du motif exact | rétention : `kesh-pre-import-x` intact |
| M-c4 | AC29 : rétention aussi après échec | écriture impossible → aucune suppression |
| M-c5 | AC30 : seuil d'espace ignoré | `insufficient_space` |
| M-c6 | AC27 : `lock().await` au lieu de `try_lock` dans la tâche | AC35 (`tokio::time::timeout` de 5 s : échoue au lieu de pendre) |
| M-c7 | AC27 : réservation sans `<=> ?` | réservation perdue |
| M-c8 | AC32 : téléchargement par concaténation du nom | `../x` → 404 |
| M-c9 | AC31 : bandeau sans la condition « deux intervalles » | Vitest bandeau « aucune réussite depuis » |
| M-c10 | AC23/AC24 : table ajoutée à `TABLES_TO_TRUNCATE` | AC24 (export ne contient pas la table ; un backup 0.14 s'importe) |
| M-c11 | AC28 : `partial` enregistré comme `ok` | AC36 « sauvegarde incomplète → `partial` » ; Vitest bandeau `partial` |
| M-c12 | AC29 : rétention qui ne protège pas `last_complete_file` | AC36 « suite de `partial` : la dernière complète reste » |
| M-c13 | AC30 : `StorageFull` classé `failed` | `classify_write_error` unitaire ; AC36 disque saturé → `disk_full` |
| M-c14 | AC30 : rétention lancée après `disk_full` | AC36 disque saturé : 8 sauvegardes restent 8 |
| M-c15 | AC32 : `record_*` incrémente `version` | AC36 « `PUT` avec la version lue avant le tour → 200 » |
| M-c16 | AC23 : réinjection de la ligne singleton retirée du script (squash sans ligne) | `squash_seeds_the_backup_schedule_row` |

Une mutation équivalente se **déclare** (pourquoi aucun test ne peut la tuer), elle ne se tait pas.

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
  une fois).
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
