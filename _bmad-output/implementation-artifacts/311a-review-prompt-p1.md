# Prompt — revue de code P1 de la Story 31-1a (les pièces dans la sauvegarde), trois lentilles, Sonnet

Tu relis le développement de la **Story 31-1a** de Kesh (comptabilité suisse) : format `.keshbackup` **2** — les pièces
du dossier des documents référencées par la base voyagent dans `files/`, inventoriées au manifeste (`files`,
`filesNotIncluded`) ; export assemblé hors mémoire (débordement dans `KESH_ADMIN_BACKUP_DIR`) ; import reçu sur disque,
vérifié par lectures bornées, exécuté dans une tâche détachée attendue, qui écrit les pièces **avant** de remplacer la
base ; plafond d'import à 2048 ; fixture `.keshbackup` produite par la vraie 0.14.0 ; manuels. Issue #503 (la 31-1a ne
la ferme pas). Worktree `/home/gcorbaz/devel/kesh-31-1`, branche `story/31-1`, base `origin/main` = `b5fd9fb1`. Le diff
à relire, **aplati** (un seul diff, pas la suite des commits) : `git -C /home/gcorbaz/devel/kesh-31-1 diff
b5fd9fb1...HEAD`. Langue du rapport : français. Modèle : **Sonnet**.

Lis d'abord : la fiche `_bmad-output/implementation-artifacts/31-1-sauvegarde-complete.md` — **partie a seulement**
(AC1 à AC14, AC6b, AC8b ; tâches T-a0 à T-a9 ; mutations M-a1 à M-a22 ; Dev Agent Record de la 31-1a) — et le
registre `_bmad-output/implementation-artifacts/epic-31-choix-autonomes-311.md` (C-31-1-59 à C-31-1-68 pour le
développement). Le Dev Agent Record déclare des gates, des chiffres et des mutations : **ne les prends pas pour argent
comptant**, recoupe-les avec les journaux `/home/gcorbaz/devel/kesh-gate-logs/311a-*.log` (lecture seule). Les parties
b et c de la fiche (31-1b, 31-1c) sont **hors périmètre** : un défaut qui ne vit qu'en elles n'est pas un finding.

Rapports, un par lentille : `/home/gcorbaz/devel/kesh-gate-logs/311a-review-p1-B.md`, `…-E.md`, `…-A.md`. Chaque
rapport porte : un bilan CRITICAL / HIGH / MEDIUM / LOW ; chaque finding avec sa preuve (sortie `grep -nF` ou `sed -n`
copiée) ; la liste des **axes exercés** et celle des **axes non exercés** — un « 0 finding » sans cette liste ne compte
pas comme passe.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. Aucun fichier temporaire : compare
  par des commandes enchaînées (pipe), lis les PDF vers la sortie standard seulement.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`).
- **N'exécute rien qui écrit, compile ou exécute du code** : ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`,
  ni `scripts/mem-guard.sh`, ni `make`, ni `latexmk`, ni `cargo` (aucune sous-commande), ni `rustc`, ni `npm`, ni
  `docker`, ni `unzip` vers un dossier, ni `python3`, ni `sed -i`, ni `perl`. Aucune commande `gh` qui écrit ;
  `gh issue view 503` est permis.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection),
  `cat`, `ls`, `find`, `sha256sum`, `unzip -l` et `unzip -p <archive> <entrée>` (vers la sortie standard),
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention.

1. `crates/kesh-api/src/admin_backup/export.rs` : `build_keshbackup` (un seul `spawn_blocking`, aucun emprunt dans la
   tâche), `write_keshbackup`, `copier_piece` (une passe ; `abort_file` sur `altered` / `unreadable` ; seule une erreur
   d'**écriture** abandonne la sauvegarde ; `large_file`), `classer_les_pieces`, `BackupWriteError` / `BackupError`.
   Une archive fausse peut-elle sortir ? Une pièce peut-elle être lue hors du dossier des documents ?
2. `crates/kesh-api/src/admin_backup/import.rs` : ordre des lectures (manifeste borné, inventaire des noms **avant** toute
   autre lecture, tables sous une borne **cumulée**, pièces à travers `take(byteSize + 1)`), cohérence pièces ↔ base,
   `referenced_files` (colonne absente), `ecrire_les_pieces` (synchronisation du dossier), `compter_les_pieces_manquantes`.
   Panique possible (dépassement, indexation, `u64::MAX`) ? Chemin hors forme joint au dossier ?
3. `crates/kesh-api/src/document_storage.rs` : `write_atomically` (factorisé avec `store_document` — comportement de
   `store_document` inchangé ?), `open_document`, `restore_document` (flux, taille, empreinte avant le renommage,
   temporaire nettoyé), `document_is_intact`.
4. `crates/kesh-api/src/routes/admin.rs` : `full_export` (`SpooledTempFile` dans `KESH_ADMIN_BACKUP_DIR`, rembobinage,
   en-têtes), `full_import` (réception par morceaux, `tokio::spawn` attendu, ce qui reste dans le handler),
   `run_backup_and_restore` (pièces **avant** `restore_tables_in_tx`, sous le verrou), `write_pre_import_backup`,
   `write_backup_file` synchrone (clone du descripteur, `sync_all` sur l'original, `.partial`), les gardes lexicales
   réécrites (test 20, `echecs_avant_sauvegarde_tous_convertis`, `import_dans_une_tache_detachee`) : l'invariant
   qu'elles tiennent a-t-il été gardé, ou affaibli ?
5. `crates/kesh-db/src/backup.rs` : `read_referenced_files` (deux requêtes, pas d'`UNION`, dédoublonnage en Rust), lu
   dans l'instantané ; les trois tests neufs.
6. Documentation : `docs/manual/fr/admin-manual.tex` **et son PDF** (`docs/manual/fr/admin-manual.pdf`, aplati),
   `docs/manual/fr/user-manual.tex` **et son PDF**, `docs/api-external.md`, `CHANGELOG.md` (§ `[0.15.0] — Non publié`),
   `.env.example`, `README.md`, `docker-compose.yml`, `docker-compose.prod.yml`. Le manuel promet-il quelque chose que le
   code ne fait pas (ex. un en-tête « affiché à l'écran » — l'écran est la 31-1b), ou omet-il un refus que le code
   oppose ? Bloc Nginx : répète-t-il le `proxy_pass` et les quatre `proxy_set_header` ?

## Lentille E — Edge cases (chaque branche)

1. Export : pour chaque issue de `copier_piece` (ouverture `NotFound` / autre erreur ; `metadata` en erreur ; lecture en
   erreur au milieu ; taille lue ≠ `metadata` ; empreinte ≠ déclarée ; écriture de l'archive en erreur ; `abort_file` en
   erreur), dis quel test l'exerce, ou qu'aucun ne l'exerce. Une pièce **modifiée pendant la lecture** (taille qui change
   entre `metadata` et la fin) : quel sort ? Une archive dont une entrée a été abandonnée **au milieu** se relit-elle
   (le test `export_piece_alteree_abandonnee_et_archive_importable` le prouve-t-il vraiment : l'entrée altérée est-elle
   bien **avant** la saine dans l'archive) ?
2. Import : formats 0, 1, 2, 3 ; manifeste de format 1 qui déclare des pièces ; entrée `files/` absente en format 2 sans
   pièce ; `files/x/` ; nom en majuscules ; `byteSize` 0 ; pièce de taille déclarée plus petite / plus grande ; même
   chemin dans `files` et `filesNotIncluded` ; chemin de `filesNotIncluded` hors forme (accepté) ; colonne
   `pdf_storage_path` absente ; colonne `pdf_sha256` absente mais `pdf_storage_path` présente ; ligne à `storage_path`
   `null`. Pour chacun : 400, 200 ou panique, et le test qui le fixe.
3. Écriture des pièces : dossier absent (créé ?), dossier non inscriptible, fichier présent intact / altéré / illisible ;
   échec au milieu de la boucle (quelles pièces restent ?) ; la base reste-t-elle intacte dans **chaque** cas d'échec
   d'avant le `DELETE` ? Et le temporaire de réception, anonyme : disparaît-il sur chaque chemin d'erreur ?
4. Concurrence et annulation : client parti pendant l'envoi (avant le `spawn`), pendant la vérification, pendant la
   restauration ; deux imports simultanés (verrou `FOR UPDATE`) ; export simultané d'un import (l'instantané).
5. Les mutations déclarées (M-a1 à M-a22, M-a6b) : pour chacune, le diff du journal `311a-mut-<M>.log` est-il la
   mutation décrite, le test qui la tue est-il celui que la fiche nomme, et l'échec porte-t-il sur l'assertion qui compte
   (et non sur un effet de bord) ? M-a6 est déclarée **équivalente** (C-31-1-63), M-a3 et M-a4 tuées par le message
   (le refus tombe plus loin) : le raisonnement tient-il ?

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC1 à AC14, AC6b et AC8b, rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la
preuve. En particulier :

- **AC1, AC2** : deux requêtes sans `UNION`, dédoublonnage en Rust, anomalies (i) (ii) (iii) ; les deux gardes
  d'inventaire (colonnes `storage_path`, appelants de `store_document(` — le motif voit-il `restore_document(` ?).
- **AC3** : une passe, `abort_file`, `open_document` seul à joindre un chemin au dossier (`grep -n "join(" export.rs`).
- **AC4** : `build_keshbackup` seul appelant d'`export_installation(` en production ; un seul `spawn_blocking` ; aucun
  `block_in_place` ; `tempfile` et `regex` aux `[dependencies]` sans paquet neuf (`git diff b5fd9fb1...HEAD --
  Cargo.lock`).
- **AC5, AC12** : en-têtes et champs d'audit, valeurs exactes dans les tests.
- **AC6, AC6b, AC7** : chaque refus prescrit a-t-il son test, et ce test vérifie-t-il le **message** ou seulement le
  statut ?
- **AC8, AC8b, AC9** : ordre « pièces avant la base » ; tâche détachée ; `filesRestored` / `filesMissing` calculés sur
  la destination ; audit `files_restored`, `files_missing`, `source_format_version`.
- **AC10** : la fixture `crates/kesh-api/tests/fixtures/keshbackup-0.14.0-format1.keshbackup` : `sha256sum` égal à celui
  du Dev Agent Record ; `unzip -p … manifest.json` : `formatVersion: 1`, `keshVersion: 0.14.0` ; deux pièces
  référencées ; données synthétiques (aucun IBAN autre que les exemples de la documentation SIX, aucun nom réel).
- **AC11** : réception par morceaux, temporaire anonyme, verrou pris après la vérification.
- **AC13** : les trois cas (format 2 vierge, format 1 vierge, format 1 même installation).
- **AC14** : rejoue la commande d'inventaire des sujets de la fiche (§ AC14, `-i`, compose compris) et confronte chaque
  ligne au tableau de la fiche et au tri du Dev Agent Record ; rejoue le grep des valeurs (`503`, `512`, `n'emporte`,
  `sans les fichiers`, `aucun binaire`, `formatVersion`, `50M`, `300s`) ; contrôle les **PDF** aplatis.
- **Gates** : les chiffres du Dev Agent Record (nextest, Vitest, `check`, `lint-i18n-ownership`, `build`, E2E et liste
  des échecs jugés contre `docs/testing.md` § *Les échecs attendus*) correspondent-ils **aux journaux** ? Le gate complet
  et l'E2E ont-ils tourné **au dernier commit de code** (compare au `git log --oneline b5fd9fb1..HEAD` : les commits
  postérieurs ne doivent toucher que des artefacts et des comptes rendus) ?
- **Écarts déclarés** au Dev Agent Record : sont-ils exacts et complets ?
