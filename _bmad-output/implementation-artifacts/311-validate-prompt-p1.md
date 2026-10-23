# Prompt — validation P1 de la spec, Story 31-1 (une sauvegarde qui restaure tout, faite par Kesh)

*Versionné le 2026-10-23. Deux lentilles, contexte frais chacune, en lecture seule stricte. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : **Sonnet** pour les deux
lentilles (la fiche a été écrite par Opus).*

Dépôt `/home/gcorbaz/devel/kesh-31-1`, branche `story/31-1` (base `44ef2079`, merge du kickoff de l'Epic 31 — même arbre que `1848012d`).
Fiche : `_bmad-output/implementation-artifacts/31-1-sauvegarde-complete.md` (une fiche, **trois** stories : 31-1a
AC1-AC14, 31-1b AC15-AC22, 31-1c AC23-AC40). Registre de la story :
`_bmad-output/implementation-artifacts/epic-31-choix-autonomes-311.md` (C-31-1-1 à 24). Epic :
`_bmad-output/planning-artifacts/epic-31-tenir-les-livres.md` § 31-1 ; registre de l'epic `epic-31-choix-autonomes.md`
(C-31-0-1 à 6, dont **C-31-0-4 : réglages nouveaux dans l'interface**). Issue #503 et ses deux commentaires de Guy —
**ils font foi** (`gh api repos/guycorbaz/kesh/issues/503` et `…/503/comments`, lecture seule). Rétrospectives
`epic-29-retro-2026-10-23.md` et `epic-30-retro-2026-10-23.md`, § 4 et § 7. Règles : `CLAUDE.md` (§ Review Iteration
Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites NON RÉSOLUS, la propagation post-patch, le
recompte des comptes rendus ; § Règle de splitting préventif ; § Migration breaking policy P1-P8 ; § Issue Tracking
Rule ; § Recette visuelle) ; `docs/agents-orchestration.md` (dont § 6.1, privilèges).

⚠️ Guy tient désormais sa **comptabilité réelle** dans Kesh, et possède des sauvegardes 0.14.0 : tout ce qui rendrait
une sauvegarde existante non importable, ou qui pourrait perdre une pièce, est au moins **HIGH**.

## Axes prioritaires (communs aux deux lentilles)

1. **L'inventaire est-il vrai ?** Chaque ligne P-1 à P-14 et « Ce qui existe et se réutilise » se revérifie au code
   (`grep -nF`, `sed -n`). En particulier : P-3 (Kesh ne supprime-t-il **vraiment** jamais un document ? cherche
   au-delà de `remove_file` : `rename`, `remove_dir_all`, `kesh-seed` `reset_demo`, routes de suppression de facture
   ou de document) ; P-4 (l'inventaire des fichiers référencés est-il **clos** ? autres appelants de `store_document`,
   autres colonnes désignant un fichier, sous un autre nom que `storage_path`) ; P-6 et P-7 (mémoire) ; P-12 (le
   parcours `/setup` → import est-il réellement impraticable ? une autre route y mène-t-elle déjà ?) ; P-13 (une table
   neuve hors `TABLES_TO_TRUNCATE` est-elle vraiment sans effet sur `parse_and_verify`, sur la fenêtre de
   `post_restore.rs`, sur `truncate_all`, `reset_demo`, le squash de test et `test_schema_guard` ?). Une prémisse
   corrigée à tort, ou une demande de #503 ou des commentaires de Guy laissée sans AC, est au moins **HIGH**.
2. **Périmètre et découpage** : chaque AC se rattache-t-il à #503 ? Chaque sous-story est-elle livrable et testable
   seule (après la 31-1a seule, une restauration sur un dossier de documents vide rend-elle les pièces ?) Le décompte
   des modules tient-il ? Une exclusion de la table *Dehors* qui laisserait #503 fermée sans sa demande tenue — en
   particulier la restauration « sans SSH » et « sur un nouveau système » — est au moins **HIGH**.
3. **Format et compatibilité** : le format 2 relit-il un manifeste 1 (`serde(default)`) ? Une sauvegarde 0.14.0
   réelle (format 1, `files/` vide, inventaire de tables actuel) s'importe-t-elle encore après les trois sous-stories ?
   Le contrôle de cohérence de l'AC7 (fichier ↔ ligne restaurée, empreinte) est-il juste pour **chaque** cas : deux
   sociétés partageant un même fichier, document `discarded`, facture refigée (ancienne et nouvelle empreinte),
   `filesNotIncluded` ? Peut-il refuser une sauvegarde **authentique** ?
4. **Migration** (P1-P8, 31-1c) : non breaking à juste titre ? `INSERT` singleton et triage P7 (précédent
   `post_restore.rs:523`) ; audit P5 et ses **deux** sites de total + compteurs de partition ; P6 ; la table ne déplace
   pas la fenêtre d'importabilité (`creates_application_table`) ; la phrase « une 0.14.0 refuse de démarrer » est-elle
   juste et écrite où il faut ?
5. **Manuel contre code** (obligatoire) : `docs/manual/fr/admin-manual.tex` § *Sauvegarde et restauration*
   (l. 1450-1965 : *Stratégie recommandée* l. 1461, PDF figés l. 1474, *Test de restauration périodique* l. 1573,
   *Export/import* l. 1826, pré-import l. 1898-1907, variables, *Quelle méthode* l. 1950) et `user-manual.tex:1909-1910`.
   Ce que le manuel dit **aujourd'hui** est-il vrai au code ? Ce que les AC14, AC21, AC34 et AC39 prescrivent d'écrire
   sera-t-il vrai au code prescrit (activée par défaut, `./backup/auto`, plafond 2048, ce qui n'est pas emporté, retour
   à la 0.14.0) ? L'inventaire des phrases à corriger (`503`, `n'emporte`, `sans les fichiers`, `512`) est-il complet —
   manuels, `CHANGELOG`, `README.md`, `website/`, i18n quatre locales, doc-comments ? Contrôle aussi le **PDF**, aplati
   vers la **sortie standard seulement** : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`.
6. **Recomptes** : 40 AC, T-a1 à T-a9, T-b1 à T-b6, T-c1 à T-c7, M-a1 à M-a11, M-b1 à M-b3, M-c1 à M-c10, deux
   specs E2E neuves, C-31-1-1 à 24, chaque numéro de ligne cité — recompte depuis la source.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque demande de #503 et de ses commentaires couverte par un AC testable, chaque AC par une tâche et par un test
  **qui rougirait** ; chaque mutation tuée par le test nommé (une mutation équivalente non déclarée est un finding —
  M-a11, M-c6 en particulier) ; chaque fait, chemin et numéro de ligne revérifié. L'E2E de l'AC20 prouve-t-il vraiment
  la restauration **des fichiers** (suppression gardée, octets comparés), et ne peut-il pas se taire ? La liste des
  « gardes existantes qui rougiront » est-elle complète (cherche les tests qui construisent ou lisent un
  `.keshbackup`, `formatVersion`, `files/`, `admin_import_max_mib`, `AppState {`) ? Écrans : rôles (Admin, Comptable,
  clé d'API), états vide / chargement / erreur / 409, largeur étroite, thème clair ; captures prescrites pour chaque
  état touché ? Textes i18n ×4 et fallbacks.
- **F — Faisabilité, sécurité, verrous et cohérence (adversaire de la conception)** :
  - **Sécurité** : traversée de chemin à l'import (`files/../`, noms absolus, liens, sous-dossiers, Unicode), au
    téléchargement des sauvegardes du serveur (AC32), archive piégée (taille déclarée, compression, nombre d'entrées) ;
    les sauvegardes du serveur restent-elles fermées aux clés d'API et aux non-Admin ; le chemin du dossier jamais
    exposé ; fichiers `0600`, dossiers `0700`.
  - **Verrous et ordre** : relis `run_backup_and_restore` (`routes/admin.rs:225-490`) — l'écriture des fichiers
    **avant** le `DELETE` sous le `FOR UPDATE` de `_kesh_version` ; le `backup_lock` en mémoire pris par l'import
    **avant** le `FOR UPDATE` : ordre d'acquisition identique partout (export, « maintenant », tâche, import) ? Un
    `lock().await` tenu pendant une requête HTTP longue bloque-t-il autre chose ? La réservation conditionnelle
    `last_attempt_at <=> ?` est-elle juste (précision `DATETIME(3)`, valeur `NULL`) ? La tâche lit-elle la base hors
    instantané d'une façon qui compte ?
  - **Instantané et fichiers** : les fichiers référencés sont-ils lus dans le **même** `ReadSnapshot` (Story 28-3) ?
    Un fichier désigné par l'instantané peut-il disparaître ou changer avant d'être lu (refigeage, réimport) ?
  - **Faisabilité** : `tempfile` 3.27 `spooled_tempfile_in`, `NamedTempFile::new_in` ; `zip` 2.4.2 (`ZipArchive<File>`,
    `CompressionMethod::Stored`, écriture dans un `SpooledTempFile`) ; `rustix::fs::statvfs` disponible avec les
    features du `Cargo.lock` ? `field.chunk()` d'axum `Multipart` sous `DefaultBodyLimit` ; `tokio::time::interval`
    dans `main.rs` sans arrêt gracieux ; les six littéraux `AppState { … }`. La garde lexicale
    `build_keshbackup_ne_lit_la_base_qu_une_fois` reste-t-elle exacte après l'AC4 ?
  - **Disque** : le seuil de l'AC30 (`× 1,2 + 256 Mio`) protège-t-il le cas d'un NAS où la base partage le volume ? La
    rétention ne peut-elle jamais supprimer la seule sauvegarde valide, ni un fichier qui n'est pas le sien ?

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve**
(commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence = sortie de
`grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas comme
passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/311-validate-p1-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/311-validate-p1-F.md` (lentille F) ; dernier message : le chemin du rapport, le
bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` — **aucun fichier
temporaire**, comparaisons en pipe seulement. Lecture seule stricte : **aucun `git checkout`, `git switch`,
`git checkout --detach` ni aucune autre forme** ; aucune commande qui écrit, compile ou exécute : `scripts/*` (dont
**`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `scripts/regen-test-schema.sh`),
`make`, `latexmk`, `xelatex`, `git commit`/`add`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`, `cargo`, `npm`,
`npx`, `node`, **`python3`, `python`, `perl`, `ruby` et tout autre interpréteur**, `sed -i`, `docker`, **`mariadb`,
`mysql` et toute requête SQL**, `bash` sur un script du dépôt, `gh` en écriture. `pdftotext` vers la **sortie
standard** seulement. Aucune donnée réelle de Guy ne se lit ni ne se cite (rien de `~/kesh-test/`). Autorisés :
lecture, `grep`, `sed -n`, `awk` en lecture, `ls`, `git log`/`show`/`diff`/`blame`, `gh api` en lecture, lecture des
sources des crates sous `~/.cargo/registry/src/` (versions du `Cargo.lock`), `pdftotext … -`.
