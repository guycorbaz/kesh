# Story 31.1 : Une sauvegarde qui restaure tout, faite par Kesh

## Status

**ready-for-dev** (trois clés) — fiche créée le 2026-10-23 en autonomie (`bmad-create-story`, Opus 5.5), sur
`1848012d` (kickoff de l'Epic 31), worktree `/home/gcorbaz/devel/kesh-31-1`, branche `story/31-1`. Validation P1 à
lancer (`31-1-validate-prompt-p1.md`, **Sonnet**, lentilles R et F).

⛔ **Trois stories, une fiche, trois PR successives** (C-31-1-1) :

| Clé au registre de sprint | Contenu | AC | PR |
|---|---|---|---|
| **`31-1a-fichiers-dans-la-sauvegarde`** | format `.keshbackup` 2 : les fichiers référencés par la base dans `files/`, nommés par leur empreinte, inventoriés au manifeste, lus dans l'instantané ; import qui vérifie, écrit les fichiers sous `KESH_DOCUMENTS_DIR` puis restaure la base ; ancien format accepté ; export assemblé hors mémoire, import reçu sur disque ; manuel d'administration (format, volume, ce qui reste hors du fichier) | AC1 à AC14 | **`refs #503`** — ⛔ jamais `closes` |
| **`31-1b-restauration-installation-neuve`** | le parcours *installation neuve → `/setup` → restaurer une sauvegarde* sans passer par l'assistant ; l'écran dit ce que le fichier ne contient pas (`.env`) et ce que la restauration a rendu (pièces restaurées, pièces manquantes) ; E2E de bout en bout ; manuel pas à pas | AC15 à AC22 | **`refs #503`** |
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
§ 31-1. Cible : **v0.15.0**. Choix : `epic-31-choix-autonomes-311.md` (C-31-1-1 à C-31-1-24). Réglages nouveaux dans
l'interface, pas en variable d'environnement (C-31-0-4).

## Prémisses de #503, établies au code (`1848012d`)

| # | Prémisse | Verdict | Preuve |
|---|---|---|---|
| P-1 | L'export crée un `files/` vide | **vraie** | `crates/kesh-api/src/admin_backup/export.rs:95` (`zip.add_directory("files/", …)`, commentaire « aucun binaire stocké en v0.2 ») ; l'issue citait l. 76-79, le fichier a bougé depuis (Story 28-3) |
| P-2 | L'import refuse un `files/` non vide | **vraie** | `admin_backup/import.rs:79-96` (`files_non_empty`, l. 83 et 91) ; et exige sa présence (l. 98-102) |
| P-3 | Après restauration, un justificatif et un PDF figé répondent **410** | **vraie sur une autre machine, à nuancer sur la même** | `routes/imported_supplier_invoices.rs:725` (`SourceDocumentGone`) ; `routes/issued_invoice_pdf.rs:365` (`InvoicePdfGone`). ⚠️ Restaurée **sur la même installation**, une sauvegarde retrouve ses fichiers : Kesh ne supprime jamais un document (`grep -rn remove_file crates/kesh-api/src` : seulement le fichier source de l'inbox, `inbox_import.rs:839`, et les temporaires). Le 410 frappe la restauration sur un **autre** dossier de documents — le cas de #503 |
| P-4 | Les fichiers référencés par la base sont « les justificatifs et les PDF figés » | **vraie, et l'inventaire est clos** | deux appelants de production de `document_storage::store_document` : `inbox_import.rs:580` (documents fournisseur → `imported_supplier_invoices.storage_path`, `file_hash`) et `routes/issued_invoice_pdf.rs:432` (PDF figé → `invoices.pdf_storage_path`, `pdf_sha256`, migration `20261003000001`). Les documents joints après coup à une facture fournisseur (30-2c) sont des lignes de `imported_supplier_invoices`. Aucune autre colonne `*storage_path*` (`grep -rn storage_path crates/kesh-db/migrations`). Avoirs, rappels, rapports : rendus à la demande, jamais stockés |
| P-5 | Nommage par empreinte SHA-256 | **déjà vrai sur disque** | `document_storage.rs:104-110` : nom `{sha256hex}.{ext}` (extension en minuscules, liste blanche, `is_safe_ext`) ; le contenu d'un nom donné est donc fixé par le nom |
| P-6 | « Passage en streaming à l'export » | **faux aujourd'hui** | l'archive est **entièrement assemblée en mémoire** (`export.rs:72` `Cursor<Vec<u8>>`) ; seule la *livraison* passe par un fichier temporaire au-delà de `KESH_ADMIN_EXPORT_INMEM_MB` (`routes/admin.rs:53-58`, `stream_via_tempfile`). Avec les fichiers, l'archive entière tiendrait en mémoire |
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
| 31-1a | `kesh-db` (`backup.rs` : fichiers référencés dans l'instantané), `kesh-api` (`admin_backup/*`, `routes/admin.rs`, `document_storage.rs`, `config.rs`), docs (manuel d'administration, CHANGELOG) | 3 |
| 31-1b | frontend (onboarding, `admin-restore`, `admin-backup`), `kesh-i18n`, docs (manuels FR), E2E | 4 |
| 31-1c | `kesh-db` (migration, squash, dépôt, `post_restore`, inventaire), `kesh-api` (tâche de fond, routes, `AppState`, `main.rs`), frontend (panneau, bandeau), `kesh-i18n`, docs | 5 |

Fusionnées, la 31-1a et la 31-1b dépasseraient cinq modules ; la 31-1c seule en atteint cinq (C-31-1-1).

## Critères d'acceptation

### Partie a — `31-1a-fichiers-dans-la-sauvegarde`

**AC1 — Inventaire des fichiers référencés, lu dans l'instantané.** `kesh_db::backup` expose
`ReferencedFile { storage_path: String, sha256: String }` et l'`InstallationExport` porte `referenced_files:
Vec<ReferencedFile>` (dédoublonnés par `storage_path`, triés), lus par `export_installation_in` **dans le même
`ReadSnapshot`** que les tables, après elles :
`SELECT storage_path, file_hash FROM imported_supplier_invoices` ∪ `SELECT pdf_storage_path, pdf_sha256 FROM invoices
WHERE pdf_storage_path IS NOT NULL`. Toutes les lignes de `imported_supplier_invoices` comptent, quel que soit leur
statut (`discarded` compris : la ligne reste restaurée et son document téléchargeable). Une même empreinte vue sous
deux `storage_path` différents, ou un `storage_path` dont le préfixe hexadécimal n'est pas l'empreinte déclarée, est
une **anomalie** : le fichier est classé `mismatch` (AC3), jamais choisi au hasard.

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
ne sont pas emportés.

**AC4 — L'export n'assemble plus l'archive en mémoire.** `build_keshbackup` écrit l'archive dans un `Write + Seek`
fourni par l'appelant (`build_keshbackup_into`) ; les fichiers sont lus **un par un** (`spawn_blocking`) et copiés dans
l'entrée, jamais tous en mémoire. `GET /admin/full-export` l'écrit dans un
`tempfile::SpooledTempFile` de seuil `KESH_ADMIN_EXPORT_INMEM_MB` (sémantique de la variable inchangée : en mémoire
jusqu'au seuil, sur disque au-delà), débordant **dans `KESH_ADMIN_BACKUP_DIR`** (`spooled_tempfile_in`, `tempfile` 3.27
— monté, contrairement au `/tmp` du conteneur), puis le diffuse. La sauvegarde pré-import l'écrit directement dans son
fichier `.partial` (`write_backup_file` devient générique sur la source, C-31-1-7). `tempfile` passe des
`[dev-dependencies]` aux `[dependencies]` de `kesh-api` (déjà au `Cargo.lock`). La garde lexicale
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
sous-dossier sous `files/`.

**AC7 — Import : empreintes vérifiées avant tout effet.** Chaque fichier de `files/` est lu en flux à travers un
`Sha256` avec une borne `byteSize + 1` (une entrée qui dépasse sa taille déclarée → 400, défense contre l'archive
piégée) ; une empreinte différente → 400 `InvalidBackupStructure` (« intégrité SHA-256 invalide pour le fichier … »).
Ces contrôles ont lieu **avant** le verrou, la sauvegarde de sécurité et tout `DELETE`. Cohérence avec la base
restaurée : tout fichier de `manifest.files` doit être référencé par une ligne restaurée (`imported_supplier_invoices`
ou `invoices` du backup), avec la même empreinte (`file_hash` / `pdf_sha256`) — sinon 400 ; un fichier référencé par
le backup qui n'est ni dans `files` ni dans `filesNotIncluded` → 400 (sauvegarde incomplète ou falsifiée).

**AC8 — Import : les fichiers sont écrits avant de rendre la main.** Dans `run_backup_and_restore`, après la
sauvegarde de sécurité et **avant** le `DELETE` des tables : chaque fichier de l'archive est écrit sous
`KESH_DOCUMENTS_DIR` par la primitive factorisée de `document_storage` (temporaire, `fsync`, `rename`). Un fichier de
même nom déjà présent **et intact** est laissé (compté restauré) ; présent mais **altéré**, il est remplacé
atomiquement (journalisé `warn!`). Un échec d'écriture (dossier non inscriptible, disque plein) → la restauration est
**refusée**, la base n'est pas touchée (`AdminFullImportFailed`, 500, message « aucune donnée n'a été remplacée ») ;
les fichiers déjà écrits restent (ils sont nommés par leur contenu : inoffensifs, C-31-1-6). Aucun fichier du dossier
des documents n'est jamais supprimé par l'import.

**AC9 — Ce que l'import rend.** La réponse gagne `filesRestored` (fichiers de l'archive écrits ou déjà intacts) et
`filesMissing` : nombre de fichiers référencés par la base **restaurée** qui, après l'import, ne sont pas lisibles et
intacts dans `KESH_DOCUMENTS_DIR` (calculé après l'écriture, pour les deux formats : une sauvegarde 1 restaurée sur la
même installation rend 0, sur une machine neuve rend le nombre de pièces perdues). L'audit `admin.full_import` porte
`files_restored`, `files_missing`, `source_format_version`. La réponse ne nomme aucun fichier (le journal du serveur,
si : `warn!` par fichier manquant).

**AC10 — Ancien format accepté, et c'est testé.** Un `.keshbackup` de format 1 produit par le code de la 0.14.0
(fixture **figée** dans les tests, construite par le constructeur de test existant `build_test_backup` avec
`formatVersion: 1`) s'importe : `200`, `filesRestored = 0`. Les tests actuels de l'import (format 1) restent verts sans
modification de leur assertion.

**AC11 — Import reçu sur disque.** `read_upload` copie le champ `file` **par morceaux** (`field.chunk()`) dans un
fichier temporaire de `KESH_ADMIN_BACKUP_DIR` (`tempfile::NamedTempFile::new_in`, `0600`, supprimé à la fin, succès
ou échec) ; `parse_and_verify` lit un `Read + Seek` (`ZipArchive<File>`), plus un `&[u8]`. Les tables NDJSON restent
lues en mémoire (taille de la base). Un dossier de sauvegarde non inscriptible → 500 `AdminPreImportBackupFailed`
(rien n'a commencé ; message existant).

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

**AC19 — Ce que la restauration a rendu.** Le succès de l'import affiche, avant la déconnexion, `filesRestored` et,
si `filesMissing > 0`, un avertissement « N pièces référencées sont introuvables après la restauration : elles
répondront « document indisponible » ; restaurez le dossier des documents depuis une autre sauvegarde ». Le délai avant
déconnexion laisse lire l'avis (le panneau attend un clic **« Se reconnecter »** quand un avertissement est affiché,
C-31-1-11).

**AC20 — E2E de bout en bout (`frontend/tests/e2e/backup-restore-new-install.spec.ts`).** (1) `seedTestState('with-company')`,
connexion ; un document fournisseur importé par l'inbox (fixture synthétique `spc_e2e_invoice.png`, helper
`inbox.ts`) et une facture validée dont le PDF est figé par un téléchargement ; (2) export par l'écran, fichier
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
si le fichier dépasse 2048 Mio) ; (2) `/setup` ; (3) *Restaurer une sauvegarde* depuis l'assistant ; (4) se reconnecter
avec les identifiants **de la sauvegarde** ; (5) vérifier : une écriture, un document fournisseur, un PDF de facture ;
lire l'avis des pièces manquantes. Le § *Test de restauration périodique* (l. ~1573) renvoie à cette procédure. Captures
(thème clair) du lien de l'assistant et de l'avis de fin d'import, intégrées au manuel si l'usage du manuel le permet
(`docs/manual/fr/screenshots/`), sinon dans `kesh-gate-logs/`. PDF régénéré.

**AC22 — i18n et textes.** Clés neuves en **quatre locales** (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`),
fallbacks Svelte identiques au français ; `npm run lint-i18n-ownership` vert ; `admin-backup-page-description` (« … dans
un fichier .keshbackup unique … ») complétée : « … les données **et les pièces justificatives** … ».

### Partie c — `31-1c-sauvegarde-automatique`

**AC23 — Migration `20261024000001_backup_schedule.sql` (non breaking).** Table **système** `_kesh_backup_schedule`
(singleton, `id TINYINT PRIMARY KEY CHECK (id = 1)`), hors `TABLES_TO_TRUNCATE` (C-31-1-15) :
`enabled BOOLEAN NOT NULL DEFAULT TRUE`, `interval_hours INT NOT NULL DEFAULT 24 CHECK (interval_hours IN (6, 12, 24,
168))`, `retention_count INT NOT NULL DEFAULT 7 CHECK (retention_count BETWEEN 1 AND 60)`, `last_attempt_at
DATETIME(3) NULL`, `last_success_at DATETIME(3) NULL`, `last_success_file VARCHAR(255) NULL`, `last_success_bytes
BIGINT NULL`, `last_status VARCHAR(32) NULL CHECK (last_status IN ('ok','failed','insufficient_space','disk_full'))`,
`last_files_not_included INT NULL`, `version INT NOT NULL DEFAULT 1`, `updated_at DATETIME(3) …`; `ENGINE=InnoDB`,
charset et collation explicites. Puis `INSERT INTO _kesh_backup_schedule (id) VALUES (1);`. Politique : **non
breaking** (`CREATE TABLE` d'une entité neuve + ligne singleton) → **pas** de bump `kesh_version_min_required`, pas de
bump des crates dans la story (la release 0.15.0 les fera, `prepare-release.sh`) ; mais, comme toute migration, une
0.14.0 refuse de démarrer sur une base migrée (`CLAUDE.md` § P1) : le retour à la 0.14.0 passe par la sauvegarde
d'avant la mise à jour — écrit au CHANGELOG. P5 : ligne au tableau de `docs/migrations-idempotence-audit.md`
(`tracked-by-sqlx`) + **les deux sites du total** (82 → 83) + compteur de partition, recomptés (`ls … | wc -l`). P6 :
`grep -rn "migrations.len()\|apply_migrations_up_to" crates/` inspecté, résultat écrit au Dev Agent Record. P7 :
l'`INSERT` est trié en `EXEMPT_MIGRATIONS` (« INSERT sur _kesh_backup_schedule, table système hors
TABLES_TO_TRUNCATE : jamais exportée ni restaurée » — précédent `_kesh_version`, `post_restore.rs:523`) ; la table ne
déplace pas la fenêtre d'importabilité (`creates_application_table` ne compte que l'inventaire). Squash de test
régénéré (`scripts/regen-test-schema.sh`) ; `test_schema_guard` vert. `backup_inventory_matches_schema` exclut
`_kesh_backup_schedule` comme `_kesh_version`.

**AC24 — Ce qui est vérifié de cette table.** Tests : la ligne singleton existe après migration ; une seconde ligne est
refusée ; `interval_hours = 5` et `retention_count = 0` refusés ; l'export d'installation **ne contient pas** la table ;
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
`new_for_tests` ; les six littéraux `AppState { … }` mis à jour). La sauvegarde automatique le prend en `try_lock` (pris
→ tour sauté, rien d'écrit en base, nouveau contrôle 5 minutes plus tard) ; « Sauvegarder maintenant », l'export
manuel et l'import le prennent en `lock().await` — l'import pour **toute** sa durée (sauvegarde de sécurité, écriture
des fichiers, restauration). Puis la sauvegarde **réserve** son tour en base : `UPDATE _kesh_backup_schedule SET
last_attempt_at = ? WHERE id = 1 AND last_attempt_at <=> ?` (valeur lue) ; `rows_affected = 0` → un autre processus l'a
prise, abandon silencieux (`debug!`). L'instantané de la base (Story 28-3) et l'immuabilité des fichiers nommés par
leur contenu (P-3) garantissent une archive cohérente même pendant une saisie.

**AC28 — Écrire la sauvegarde.** `run_once` crée `KESH_ADMIN_BACKUP_DIR/auto/` (`0700`, `DirBuilder` récursif comme
`write_pre_import_backup`), supprime d'abord les `kesh-auto-*.keshbackup.partial` qu'il y trouve (les siens, laissés par
un arrêt brutal — le verrou garantit qu'aucun n'est en cours), puis écrit `kesh-auto-<AAAAMMJJ>T<HHMMSS>Z.keshbackup`
(UTC) par `build_keshbackup_into` dans le `.partial` de `write_backup_file` (`0600`, renommage, jamais d'écrasement).
Succès : `last_status = 'ok'`, `last_success_at`, `last_success_file`, `last_success_bytes`,
`last_files_not_included` ; journal `info!`.

**AC29 — Rétention.** Après un succès **seulement** : les fichiers de `auto/` dont le nom satisfait exactement
`^kesh-auto-\d{8}T\d{6}Z\.keshbackup$` sont triés par nom (= par date), et les plus anciens au-delà de
`retention_count` sont supprimés ; **jamais** un autre fichier (`kesh-pre-import-*`, fichier déposé à la main, nom
voisin), jamais la sauvegarde qui vient d'être écrite. Un échec de suppression est journalisé et n'efface pas le succès.
Un échec de sauvegarde ne supprime **rien**.

**AC30 — Disque plein, avant et pendant.** Avant d'écrire : espace libre du système de fichiers de `auto/`
(`rustix::fs::statvfs`, dépendance directe ajoutée — `rustix` 1.1.4 est au `Cargo.lock` ; C-31-1-19) comparé à
`last_success_bytes × 1,2 + 256 Mio` (ou 256 Mio s'il n'y a jamais eu de succès) : insuffisant → rien n'est écrit,
`last_status = 'insufficient_space'`, `warn!` avec les deux nombres. Pendant : une erreur `ENOSPC` (ou toute erreur
d'écriture) supprime le `.partial` (déjà fait par `write_backup_file`), `last_status = 'disk_full'` (ENOSPC) ou
`'failed'` (autre) ; **la rétention ne tourne pas**, les sauvegardes existantes restent. Le seuil protège surtout les
voisins du disque (la base elle-même, sur un NAS, partage souvent le volume).

**AC31 — Échec signalé.** Tout `last_status` autre que `ok` est visible : (a) dans le panneau (AC33), bandeau d'erreur
(`role="alert"`) qui dit la cause en clair, l'heure de la dernière tentative et de la dernière réussite ; (b) dans
l'application, pour un **Administrateur** seulement, un bandeau discret sous l'en-tête (`routes/(app)/+layout.svelte`)
« La dernière sauvegarde automatique a échoué — Administration → Sauvegarde complète » quand `last_status ≠ ok`, **ou**
quand la sauvegarde est activée et que `last_success_at` est nul ou plus vieux que **deux** intervalles (« aucune
sauvegarde automatique réussie depuis le … ») — une sauvegarde qui cesse de tourner sans échouer (tâche morte) se voit
aussi (C-31-1-20). Un journal `error!` accompagne chaque échec. Pas d'entrée d'audit par sauvegarde réussie (bruit
quotidien) ; l'état vit dans la table.

**AC32 — Routes (Administrateur, refusées aux clés d'API).** Sous `admin_routes` (RBAC `require_admin_role`), chaque
handler appelle `ensure_not_pat` :
- `GET /api/v1/admin/backup-schedule` → réglage + état + `nextDueAt` (calculé par la même logique que `is_due`) +
  `backupDir` **non exposé** (le chemin reste au journal, comme pour la sauvegarde pré-import) ;
- `PUT /api/v1/admin/backup-schedule` `{ enabled, intervalHours, retentionCount, version }` → 200 ; valeurs hors
  domaine → 400 `VALIDATION_ERROR` ; version périmée → 409 `OPTIMISTIC_LOCK_CONFLICT` (code existant) ; audit
  `admin.backup_schedule_updated` (`before`/`after`) dans la même transaction ;
- `POST /api/v1/admin/backup-schedule/run` → exécute `run_once(…, Trigger::Manual)` **sans** la condition `is_due`,
  attend la fin, rend l'état ; verrou pris par une autre sauvegarde → 409 `BACKUP_IN_PROGRESS` (code neuf) ;
- `GET /api/v1/admin/backups` → liste des fichiers `*.keshbackup` de `KESH_ADMIN_BACKUP_DIR` **et** de `auto/` (nom,
  `kind` = `auto` | `pre-import` | `other`, taille, date de modification), plus récents en tête ; les `.partial` exclus ;
- `GET /api/v1/admin/backups/{kind}/{name}` → téléchargement en flux ; `name` validé par expression régulière
  (`kesh-auto-…` ou `kesh-pre-import-…` selon `kind`) **puis** retrouvé dans la liste du dossier (jamais concaténé tel
  quel : pas de traversée) ; 404 sinon ; audit best-effort `admin.backup_downloaded` (nom, taille).
`audit_route_registry.rs`, `audit_label_registry.rs` et les libellés d'audit (quatre locales) suivent. `docs/api-external.md`
inchangé : ces routes relèvent de la fermeture générale des routes d'administration aux clés (§ 4) — vérifié, à
écrire au Dev Agent Record.

**AC33 — L'écran.** `AdminBackupPanel` gagne une section **« Sauvegarde automatique »** : interrupteur *Activée*,
fréquence (*Toutes les 6 heures* / *12 heures* / *Chaque jour* / *Chaque semaine*), *Nombre de sauvegardes conservées*
(1–60), *Enregistrer* ; état : dernière réussite (date locale, taille lisible, nombre de pièces non incluses s'il y en
a), dernière tentative et son statut, prochaine prévue ; bouton **« Sauvegarder maintenant »** (désactivé pendant
l'exécution) ; liste **« Sauvegardes sur le serveur »** (nom, type, taille, date, *Télécharger*). Rappel permanent : les
sauvegardes du serveur sont dans le dossier de sauvegarde de l'installation (`./backup/auto` avec les compose fournis),
sur le **même disque** que Kesh — à copier ailleurs (Hyper Backup) ; elles contiennent des secrets. Les réglages de la
sauvegarde automatique ne voyagent pas dans le `.keshbackup` (ils appartiennent à l'installation, comme le `.env`).

**AC34 — Télécharger une sauvegarde du serveur** remplace, pour la sauvegarde pré-import, le geste SSH du manuel
(l. ~1902-1907, `sudo cp … chown`) : le manuel donne désormais le téléchargement par l'écran comme voie normale, SSH
en secours.

**AC35 — Concurrence avec l'import, vérifiée.** Test d'intégration : pendant un import (verrou tenu par un test au
moyen d'un `backup_lock` pris à la main), `run_once(Trigger::Schedule)` saute sans écrire ni modifier la table ;
`POST …/run` rend 409 `BACKUP_IN_PROGRESS`. Après la libération, `run_once` écrit.

**AC36 — Tests d'intégration de la sauvegarde automatique** (`crates/kesh-api/tests/admin_auto_backup_e2e.rs`) : succès
(fichier `0600`, dossier `0700`, archive importable : `parse_and_verify` vert, fichiers inclus) ; rétention (8
sauvegardes déposées + une neuve, `retention_count = 3` → restent les 3 plus récentes, un `kesh-pre-import-x` et un
`autre.keshbackup` intacts) ; espace insuffisant (seuil injecté par paramètre de test : la fonction de mesure est
passée à `run_once` derrière un trait `FreeSpace`, C-31-1-19) → `insufficient_space`, aucun fichier ; écriture
impossible (dossier `auto` remplacé par un fichier) → `failed`, aucune suppression ; réservation perdue (une autre
connexion avance `last_attempt_at` entre la lecture et l'`UPDATE` — observateur de test) → rien n'est écrit ;
`.partial` résiduel supprimé ; routes : 403 non-admin, 403 clé d'API, 400 domaine, 409 version, téléchargement 404 sur
`../x`, sur un nom hors expression et sur un nom absent.

**AC37 — Vitest.** Panneau : rendu de l'état (ok, échec de chaque type, jamais tenté, désactivé), enregistrement
(payload exact, conflit 409 → rechargement proposé), « Sauvegarder maintenant » (désactivé pendant, 409 affiché),
liste et téléchargement ; bandeau du layout : affiché pour Admin seulement, dans les deux cas de l'AC31, absent quand
`ok` et récent.

**AC38 — E2E (`frontend/tests/e2e/backup-schedule.spec.ts`).** Admin : changer la fréquence et la rétention,
enregistrer, recharger la page → valeurs gardées ; « Sauvegarder maintenant » → état « réussie » et une ligne `auto` dans
la liste ; télécharger cette ligne → fichier non vide dont le manifeste a `formatVersion: 2` ; réglages remis aux défauts
en fin de spec (la table survit aux `seedTestState`). Comptable : le menu n'offre pas la page, l'URL directe rend
l'écran d'erreur 403 existant.

**AC39 — Manuel et CHANGELOG.** `admin-manual.tex` : sous-section **« La sauvegarde automatique »** (activée par défaut
à la mise à jour, fréquence, rétention, emplacement `./backup/auto`, espace requis et refus, état et bandeau, *Sauvegarder
maintenant*, téléchargement, ce qu'elle ne remplace pas : la copie hors du disque) ; tableau *Quelle méthode choisir ?*
(l. ~1950-1963) mis à jour ; *Stratégie recommandée* (l. 1461) ; sauvegarde pré-import (téléchargement, AC34) ; mise à
jour vers la 0.15.0 (migration non breaking, retour à la 0.14.0 par la sauvegarde d'avant). PDF régénéré. CHANGELOG
`[Unreleased]`. README § Fonctionnalités (ligne *Export/import d'installation*, l. 49).

**AC40 — Captures (thème clair)** de chaque état touché : section *Sauvegarde automatique* (jamais tentée, réussie,
en échec), « Sauvegarder maintenant » en cours, liste des sauvegardes, bandeau du layout ; dans
`/home/gcorbaz/devel/kesh-gate-logs/311c-captures/`, citées au Dev Agent Record et au prompt de revue.

## Tasks / Subtasks

### 31-1a

- [ ] T-a1 Tests rouges : AC2 (deux gardes), AC6/AC7 (unitaires de `parse_and_verify` : format 2 valide, bijection,
  nom invalide, préfixe ≠ sha, taille dépassée, sous-dossier, format 3 refusé), AC10 (format 1), AC13 (intégration).
- [ ] T-a2 `kesh-db` : `ReferencedFile`, lecture dans `export_installation_in` (AC1) ; tests de l'instantané étendus
  (une facture figée **après** l'ouverture de l'instantané n'apparaît pas dans `referenced_files` — observateur).
- [ ] T-a3 `manifest.rs` : version 2, `files`, `filesNotIncluded` (`serde(default)`).
- [ ] T-a4 `export.rs` : `build_keshbackup_into<W: Write + Seek>`, lecture et vérification des fichiers (AC3), méta
  étendue (AC5) ; `routes/admin.rs` : `SpooledTempFile` (AC4), en-têtes (AC5, AC12), audit.
- [ ] T-a5 `document_storage.rs` : cœur d'écriture atomique factorisé ; `restore_document(dir, storage_path, bytes,
  expected_sha)` (AC8) ; `write_backup_file` générique sur la source (AC4).
- [ ] T-a6 `import.rs` : lecture `Read + Seek`, format 2 (AC6, AC7) ; `routes/admin.rs` : `read_upload` sur disque
  (AC11), écriture des fichiers avant le `DELETE` (AC8), comptes et audit (AC9).
- [ ] T-a7 `config.rs` : défaut 2048 (AC12) et son test ; `.env.example`.
- [ ] T-a8 Manuels, CHANGELOG (AC14) ; grep des valeurs (`503`, `512`, `files/`, `n'emporte`, `sans les fichiers`,
  `aucun binaire`) sur tout le dépôt — code, doc-comments, i18n, manuels, `website/` — et traitement de chaque site
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

### 31-1c

- [ ] T-c1 Migration, squash, audit P5, P6, P7, inventaire (AC23, AC24).
- [ ] T-c2 `kesh-db` : dépôt `backup_schedule` (`get`, `update_settings` avec version, `claim_attempt`, `record_*`).
- [ ] T-c3 `kesh-api/src/auto_backup.rs` : `is_due`, `run_once`, `spawn`, `FreeSpace` (AC25-AC30) ; `AppState.backup_lock`
  (AC27) ; branchement dans `main.rs`.
- [ ] T-c4 Routes et registres (AC32), libellés d'audit ×4.
- [ ] T-c5 Frontend : panneau (AC33), bandeau (AC31), API ; Vitest (AC37).
- [ ] T-c6 Tests d'intégration (AC35, AC36), E2E (AC38), captures (AC40).
- [ ] T-c7 Manuel, CHANGELOG, README (AC34, AC39).

## Dev Notes

### Tests (récapitulatif, rouges d'abord)

| Sous-story | Rust | Vitest | E2E |
|---|---|---|---|
| 31-1a | gardes AC2 (2), `parse_and_verify` format 2 (≥ 8), instantané (1), intégration AC13 (2), AC10 (1), écriture des fichiers et refus (AC8 : non inscriptible, altéré remplacé) | — | aucune spec neuve (l'écran ne change pas) ; suite complète au dernier commit de code |
| 31-1b | — | ≥ 8 | `backup-restore-new-install.spec.ts` |
| 31-1c | migration (AC24), `is_due` (≥ 6), intégration AC35/AC36 | ≥ 10 | `backup-schedule.spec.ts` |

### Gardes existantes qui changent de sens ou rougiront

- `import.rs` tests `parse_rejects_future_format_version` (forge `Some(2)` → désormais **valide** : passer à 3) et
  `parse_rejects_non_empty_files_dir` (vaut encore pour le format 1 : forcer `formatVersion: 1`).
- `manifest.rs` `manifest_json_has_canonical_shape` (`formatVersion` 1 → 2, `files` présent).
- `export.rs` `build_keshbackup_ne_lit_la_base_qu_une_fois` (corps renommé ; garder la garde exacte).
- `routes/admin.rs` garde `avant_sauvegarde_branchee_aux_appels_de_l_import` (compte « deux `build_keshbackup` » —
  le nom change).
- `admin_full_export_e2e.rs:477` (`admin_export_inmem_mib = 0`, chemin de diffusion) ; `admin_backup_e2e.rs` (structure
  du ZIP : `files/` vide → vide **seulement** sans fichier référencé).
- `config.rs` tests du défaut 512 ; `configuration_transmise.rs` (variables inchangées, vérifier).
- `backup.rs` `backup_inventory_matches_schema` et le test de l'instantané qui énumère les tables lues (`TABLES_TO_TRUNCATE
  + _kesh_version`, l. ~1773) ; `post_restore.rs` `every_data_backfill_migration_is_triaged` ; `test_schema_guard.rs`.
- `audit_route_registry.rs`, `audit_label_registry.rs` (routes et actions neuves de la 31-1c).
- `lint-i18n-ownership`, `i18n-keys.test.ts` ; `onboarding-layout.test.ts` (route neuve) ; `boutons-rouges-inventory.test.ts`
  si un bouton destructif apparaît (aucun prévu).

### Mutations (motif unique vérifié, restauration `git checkout -- f && touch f`, sous le verrou de gate)

| # | Mutation | Test qui doit la tuer |
|---|---|---|
| M-a1 | AC1 : retirer la branche `invoices` de l'union | intégration AC13 (PDF absent → 410) |
| M-a2 | AC3 : inclure un fichier altéré au lieu de le lister | unitaire export « altéré non inclus » |
| M-a3 | AC6 : supprimer le contrôle « préfixe = sha » | `parse_and_verify` nom ≠ empreinte |
| M-a4 | AC6 : ne pas exiger la bijection (entrée sans ligne acceptée) | `parse_and_verify` entrée orpheline |
| M-a5 | AC7 : comparer à `meta.sha256` d'un autre fichier / sauter la vérification | `parse_and_verify` empreinte fausse |
| M-a6 | AC7 : supprimer la borne `byteSize + 1` | entrée plus grande que déclarée |
| M-a7 | AC8 : écrire les fichiers **après** le `DELETE` | refus d'écriture : la base doit rester intacte |
| M-a8 | AC8 : ignorer l'erreur d'écriture | idem (200 au lieu de 500) |
| M-a9 | AC9 : `filesMissing` calculé sur le backup au lieu de la destination | AC13 format 1 (`filesMissing = 2`) |
| M-a10 | AC6 : accepter `formatVersion` 3 | format 3 refusé |
| M-a11 | AC1 : lire les fichiers référencés **hors** instantané (sur le pool) | garde lexicale + test de l'observateur |
| M-b1 | AC15 : lien visible pour un non-Admin | Vitest du lien |
| M-b2 | AC19 : déconnexion immédiate malgré `filesMissing > 0` | Vitest *Se reconnecter* |
| M-b3 | AC18 : ignorer l'en-tête `Not-Included` | Vitest avis export |
| M-c1 | AC25 : `≥` → `>` sur l'intervalle | `is_due` à l'égalité |
| M-c2 | AC25 : retirer la clause `last_attempt_at` | `is_due` « échec il y a 10 min » |
| M-c3 | AC29 : rétention sur `kesh-*` au lieu du motif exact | rétention : `kesh-pre-import-x` intact |
| M-c4 | AC29 : rétention aussi après échec | écriture impossible → aucune suppression |
| M-c5 | AC30 : seuil d'espace ignoré | `insufficient_space` |
| M-c6 | AC27 : `lock().await` au lieu de `try_lock` dans la tâche | AC35 (le test attend un saut, pas une attente) |
| M-c7 | AC27 : réservation sans `<=> ?` | réservation perdue |
| M-c8 | AC32 : téléchargement par concaténation du nom | `../x` → 404 |
| M-c9 | AC31 : bandeau sans la condition « deux intervalles » | Vitest bandeau « aucune réussite depuis » |
| M-c10 | AC23/AC24 : table ajoutée à `TABLES_TO_TRUNCATE` | AC24 (export ne contient pas la table ; un backup 0.14 s'importe) |

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
  (si le panneau devient trop long), deux specs E2E.
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
