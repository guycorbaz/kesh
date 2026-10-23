# Prompt — validation P5 de la spec, Story 31-1 (une sauvegarde qui restaure tout, faite par Kesh)

*Versionné le 2026-10-24. Passe **complète**, deux lentilles, contexte frais chacune, en lecture seule stricte.
Rotation (D6, rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 :
Sonnet ; P2 : Opus ; P3 : Sonnet ; P4 : Opus ; les quatre remédiations ont été écrites par Opus. P5 est confiée à
**Sonnet** pour les deux lentilles. Une boucle ne se clôt que sur une passe à **0 MEDIUM+**.*

Dépôt `/home/gcorbaz/devel/kesh-31-1`, branche `story/31-1` (base `44ef2079`). Fiche :
`_bmad-output/implementation-artifacts/31-1-sauvegarde-complete.md` (une fiche, **trois** stories : 31-1a AC1-AC14 +
AC6b, AC8b ; 31-1b AC15-AC22 + AC18b ; 31-1c AC23-AC40). Registre de la story :
`_bmad-output/implementation-artifacts/epic-31-choix-autonomes-311.md` (C-31-1-1 à 58 ; **52 à 58 sont neufs depuis la
P4** ; 53 amende C-31-1-47, 54 amende C-31-1-23 et C-31-1-45, 58 précise 52, sans les réécrire). Epic :
`_bmad-output/planning-artifacts/epic-31-tenir-les-livres.md` § 31-1 ; registre de l'epic `epic-31-choix-autonomes.md`
(C-31-0-1 à 6, dont **C-31-0-4 : réglages nouveaux dans l'interface**). Issue #503 et ses deux commentaires de Guy —
**ils font foi** (`gh api repos/guycorbaz/kesh/issues/503` et `…/503/comments`, lecture seule). Rapports précédents :
`/home/gcorbaz/devel/kesh-gate-logs/311-validate-p{1,2,3,4}-{R,F}.md` ; leur traitement est aux entrées « Validation
P1 » à « Validation P4 » du Change Log de la fiche. Arbitrages de l'orchestrateur, **à ne pas rouvrir** : C-31-1-36
(`scripts/regen-test-schema.sh` compté avec `kesh-db`), C-31-1-40 (`partial` au panneau seulement) et C-31-1-52 (F12
intégré à la 31-1c : mention « trop grande pour être restaurée par l'écran » dans la liste ; sa **forme**, C-31-1-58,
reste discutable). Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes
déclarés, les sites NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Règle de splitting
préventif ; § Migration breaking policy P1-P8 ; § Issue Tracking Rule ; § Pattern batch) ; `docs/agents-orchestration.md`
(dont § 6.1, privilèges).

Le diff de la remédiation P4 se lit par `git diff 6f0828cf ee8ba4cb -- _bmad-output/` (lecture seule).

⚠️ Guy tient désormais sa **comptabilité réelle** dans Kesh, et possède des sauvegardes 0.14.0 : tout ce qui rendrait
une sauvegarde existante — ou une sauvegarde 0.15 **authentique** — non importable, ou qui pourrait perdre une pièce,
est au moins **HIGH**.

⚠️ **Piège d'outil** : le `grep` de ce poste est `ugrep`. Sur un motif comme `.{150}` appliqué à du texte UTF-8, il
s'arrête en erreur (`exceeds complexity limits`) et **ne rend rien** — la P2 a lu cette erreur comme un résultat vide.
Un `grep` vide sur un motif complexe se contre-vérifie par `awk` (`index()`) avant d'être cité comme preuve d'absence.
⚠️ Et une sortie de `sed -n A,Bp | cat -n` se numérote **à partir de 1**, pas de A : la P4 a décalé d'une ligne des
numéros pourtant justes (R10, réfuté).

## Axe 0 — d'abord : ce que la remédiation P4 a écrit

*La sévérité ne stagne pas, elle se déplace vers ce qu'on vient d'écrire* (`CLAUDE.md` § La passe ciblée). La P4 a
**simplifié** quatre exigences et en a **ajouté** plusieurs. Attaque d'abord ce diff, avant tout autre axe. Questions à
trancher, chacune au code ou aux sources des crates (versions du `Cargo.lock`) :

- **Passe unique et `abort_file` (AC3, AC4, T-a4, T-a4b, M-a21, C-31-1-53)** : `zip` 2.4.2 `ZipWriter::abort_file`
  (`write.rs:1134`) retire-t-il bien l'entrée **en cours** quand d'autres entrées la précèdent (condition
  `rewind_safe`), et la suivante écrase-t-elle proprement ses octets ? `finalize` (`:1501-1524`) produit-il une archive
  que `ZipArchive::new` relit quand l'entrée abandonnée était plus longue que tout ce qui suit ? Le `SpooledTempFile`
  (avant et après son débordement), le clone d'un `File` (`try_clone`) et l'écrivain de test de l'AC30 (`StorageFull`
  après N octets) supportent-ils le `seek` en arrière qu'`abort_file` exige ? La distinction « erreur de lecture du
  fichier → `abort_file` + `unreadable` / erreur d'écriture de l'archive → abandon » est-elle réalisable par la boucle
  écrite à la main que l'AC3 prescrit, et `abort_file` peut-il lui-même échouer (et alors ?) ? `byteSize` lu par
  `metadata()` puis copie à travers `take(byteSize + 1)` : un fichier plus court ou plus long que sa taille est-il bien
  `altered` ? Les tests nommés (dossier à la place du fichier ; altéré suivi d'un sain, aller-retour) rougissent-ils sous
  M-a21 et M-a2 ? La mutation déclarée équivalente en P3 a-t-elle vraiment disparu partout (fiche, registre) ?
- **Réservation retirée (AC26, AC27, AC32, AC36, T-c2, C-31-1-54)** : reste-t-il, dans la fiche ou le registre vivant,
  une mention qui prête encore une protection entre processus, un `Skipped` sur réservation perdue, `claim_attempt`, ou
  M-c7 ? `record_attempt` sans condition, enregistré **après** le verrou et **avant** l'écriture : l'ordre convient-il à
  `is_due` (la tentative compte-t-elle même si `insufficient_space` sort aussitôt) ? Le nettoyage des `.partial`
  (AC28, `auto/` et pré-import) est-il juste dans un seul processus ?
- **Garde de l'export dans le `spawn_blocking` (AC4, AC27, AC36, M-c24, C-31-1-55)** : `OwnedMutexGuard<()>` (tokio
  1.52.3) entre-t-il dans la fermeture `remplir` (`Send + 'static`) ? Le garde est-il rendu **avant** la diffusion, et
  le reste-t-il si l'assemblage échoue ou panique ? La garde lexicale prescrite (nom du garde dans la fermeture passée à
  `build_keshbackup` dans `full_export`) est-elle décidable, et tue-t-elle M-c24 ?
- **`build_keshbackup` générique (AC4, § Gardes, C-31-1-56)** : signature `build_keshbackup(pool, documents_dir,
  remplir)` et fermeture `FnOnce(InstallationExport, PathBuf) -> Result<T, BackupWriteError> + Send + 'static` —
  compilable, et chaque appelant (export, pré-import sous `FOR UPDATE`, automatique) peut-il y loger son écrivain, son
  garde ou son `Arc<dyn BackupEnv>` ? L'assertion neuve « `export_installation(` une seule fois dans la production de
  `kesh-api` » est-elle exacte aujourd'hui (`grep -rnF "export_installation(" crates/kesh-api/src`) ? La fiche dit-elle
  juste ce qui arrive à `avant_sauvegarde_branchee_aux_appels_de_l_import` (instruction coupée au premier `;` d'une
  fermeture en ligne) ?
- **`importMaxMib` (AC32, AC33, AC36, AC37, AC39, AC40 (r), M-c23, C-31-1-52, C-31-1-58)** : forme `{ importMaxMib,
  items }` ; la comparaison stricte `size > importMaxMib × 1 048 576` contre ce qu'accepte réellement l'import
  (`DefaultBodyLimit`, `lib.rs:253-260`, qui compte l'enveloppe multipart) — l'écart est-il bien écrit ? Les tests et la
  mutation rougissent-ils ? La capture (r) est-elle atteignable par la voie donnée ? Le changement de forme de la
  réponse de `GET backups` est-il propagé partout où la fiche décrit cette liste ?
- **Horloge en arrière (AC25, AC31, AC32, AC36, AC37, AC39, M-c9, M-c22, C-31-1-57)** : `is_due` tient une valeur
  future pour nulle, `is_overdue` est calculé au serveur (`overdue`), `nextDueAt` vaut `now` si une valeur est future.
  Peut-il en naître une boucle de sauvegardes, un bandeau qui ne s'éteint pas, ou une contradiction avec l'AC31 (cas
  « `last_success_at` nul ») ? M-c9 reciblée sur `is_overdue` et la Vitest du bandeau qui ne lit plus que `overdue`
  laissent-elles un comportement du bandeau sans test ?
- **R2, R3, R4, R5 (AC9, AC13, AC20, AC21, AC36, AC40, M-a9, M-c3, M-c4, M-c8)** : chaque mutation reciblée est-elle
  tuée par le test désormais nommé (raisonne comme la P4 : sous la mutation, quel nombre ou quel statut le test
  observe-t-il) ? L'assertion comptable de l'AC20 (6) est-elle observable par l'écran (liste des factures, journal) ?
- **Inventaire (AC14, T-a8, F3, R9)** : rejoue la commande de l'AC14 (`-i`, compose compris) — elle doit rendre **103
  lignes** sur `HEAD` (code inchangé depuis `44ef2079`). Chaque ligne est-elle au tableau ou triable « intacte » ? Les
  sept sites ajoutés par R9 et les trois des compose sont-ils exacts (numéros, phrases) ?
- **Petites corrections** : AC1 (deux requêtes, dédoublonnage en Rust, M-a22 et son test `X….pdf` / `x….pdf` — le
  test rougit-il sous un `UNION` SQL ?) ; AC8 (`sync_all` du dossier non bloquant ; `ParsedBackup` par valeur) ; AC11
  (`tokio::fs::File::from_std` puis `into_std`) ; AC14 (message qu'une 0.14.0 affiche, `import.rs:85-116`) ; AC19
  (toast `admin-restore-toast-success` retiré des quatre locales — `lint-i18n-ownership`, `i18n-keys.test.ts`,
  `AdminRestorePanel.test.ts` en dépendent-ils ?) ; § Gardes (`migrations_upgrade_path.rs`, `EXEMPT_MIGRATIONS.len()`,
  `assert_eq!(discriminating, 6)`) ; AC6 (`regex` aux `[dependencies]`).
- **Affirmations neuves** (numéros de ligne cités par la remédiation P4, comportements de `zip` 2.4.2, `tokio` 1.52.3)
  et l'unique **réfutation** de la P4 (R10, premier point) — chacune se revérifie, comme une prémisse.
- **Recomptes** de la note « Validation P4 » : 43 AC, 26 tâches, **52 mutations** (22 M-a, 7 M-b, 23 M-c — M-c7
  retirée, numéro non réattribué), 58 choix, 103 lignes d'inventaire ; et cohérence des mutations neuves ou reciblées
  (M-a9, M-a21, M-a22, M-c3, M-c4, M-c8, M-c9, M-c21, M-c22, M-c23, M-c24) avec les tests qu'elles nomment.

Un défaut introduit par la remédiation P4 se signale comme tel (`introduit en P4`), avec le passage du diff.

## Axes prioritaires (communs aux deux lentilles)

1. **L'inventaire est-il vrai ?** P-1 à P-14 et « Ce qui existe et se réutilise » au code (`grep -nF`, `sed -n`) ;
   contrôle surtout ce que les passes précédentes déclarent n'avoir pas exercé (fin de leurs rapports : P-1 à P-14 une
   à une, `audit_label_registry.rs`, `audit_route_registry.rs`, `kesh-seed`, `crates/kesh-db/src/repositories`, les
   E2E et `helpers/inbox.ts`, les `.ftl` de-CH/it-CH/en-CH, `AdminRestorePanel.test.ts`, `onboarding-layout.test.ts`,
   `api-client.test.ts`). Une demande de #503 ou de ses commentaires laissée sans AC est au moins **HIGH**.
2. **Périmètre et découpage** : chaque AC se rattache-t-il à #503 ? Chaque sous-story est-elle livrable et testable
   seule ? Le décompte des modules (31-1a : 3, 31-1b : 4, 31-1c : 5) tient-il après la P4 ?
3. **Format et compatibilité** : manifeste 1 relu ; fixture 0.14.0 réelle ; AC7 limité au format 2 ; colonne
   `pdf_storage_path` absente ; règle d'anomalie par ligne (AC1) pour chaque cas, **aller-retour compris** ; valeur
   `unreadable` ; archive portant une entrée abandonnée. Peut-elle refuser une sauvegarde authentique ?
4. **Migration** (P1-P8, 31-1c) : non breaking à juste titre ; `INSERT` singleton et triage P7 (`EXEMPT_MIGRATIONS`,
   fondement `Durable`) ; audit P5 et ses trois sites ; P6 (`migrations_upgrade_path.rs`, `total` **et** `N`) ; squash
   et sa garde ; fenêtre d'importabilité inchangée.
5. **Manuel contre code** (obligatoire) : `docs/manual/fr/admin-manual.tex` § *Sauvegarde et restauration*
   (l. 1450-1965), les exemples de proxy inverse (l. 320-345 Nginx, ~358 Caddy, ~377 Traefik, ~616 Portail DSM), les
   sites de l'inventaire (dont ceux ajoutés en P4 : `docker-compose.yml:154-158`, `docker-compose.prod.yml:165-176`,
   `config.rs:895-896, 1268-1269`, `.env.example:220-223, 255-257`, `routes/admin.rs:52`, `export.rs:1-12`,
   `admin-manual.tex:1838`), `user-manual.tex` (l. 1155-1161, 1909-1910), `docs/api-external.md` (l. 721, 751). Ce
   que les AC14, AC21, AC34 et AC39 prescrivent d'écrire sera-t-il vrai au code prescrit (en particulier : une instance
   par base, l'horloge dans les deux sens, la mention « trop grande », le pic de disque d'un import, le message vu par
   une 0.14.0) ? Contrôle aussi le **PDF**, aplati vers la **sortie standard seulement** : `pdftotext
   docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`.
6. **Recomptes** : 43 AC, 26 tâches, 52 mutations, 58 choix, deux specs E2E neuves, chaque numéro de ligne cité —
   recompte depuis la source.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque demande de #503 et de ses commentaires couverte par un AC testable, chaque AC par une tâche et par un test
  **qui rougirait** ; chaque mutation tuée par le test nommé (une mutation équivalente non déclarée est un finding) ;
  chaque fait, chemin et numéro de ligne revérifié. La liste des « gardes existantes qui rougiront » distingue-t-elle
  celles qui **rougissent** de celles qui deviendraient **muettes** ? Écrans : rôles ARIA, états, largeur étroite,
  thème clair ; captures des AC21 et AC40 — chaque état touché y est-il, chaque état listé est-il atteignable par la
  voie que la fiche donne ? Textes i18n ×4 et fallbacks, pour la 31-1b **et** la 31-1c (dont la clé retirée et la clé
  neuve `admin-backup-server-too-large`).
- **F — Faisabilité, sécurité, verrous et cohérence (adversaire de la conception)** :
  - **Sécurité** : traversée à l'export (AC3, `open_document` et son test direct) et à l'import (AC6, AC6b) ; chemins
    de `filesNotIncluded` libres — jamais joints, journalisés en `?path` seulement ; téléchargement des sauvegardes du
    serveur (AC32, seul porteur d'`ensure_not_pat`) ; archive piégée (bornes appliquées à **toutes** les lectures) ;
    fermeture aux clés d'API et aux non-Admin ; modes `0600`/`0700`.
  - **Verrous et ordre** : relis `run_backup_and_restore` (`routes/admin.rs:225-495`), `full_import` et `full_export`
    contre l'AC27 réécrite ; un garde possédé dans un `spawn_blocking` qui panique est-il rendu ? Un export qui attend
    le verrou pendant un import de plusieurs minutes pose-t-il problème au client (délai d'une heure, AC18b) ?
  - **Instantané et fichiers** : `referenced_files` dans le même `ReadSnapshot` ; lecture unique des fichiers hors
    instantané — est-ce sans conséquence ?
  - **Disque et durabilité** : seuil de l'AC30 (`documents_bytes` dans le `spawn_blocking`) ; rétention avec
    `last_complete_file` ; `sync_all` sur l'original après écriture par le clone ; `sync_all` du dossier des documents
    non bloquant (AC8) ; entrée abandonnée à mi-copie (`abort_file`) et suppression du `.partial` dans tous les chemins
    d'échec.

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve**
(commande exécutée et sortie copiée), correction proposée — en préférant, quand c'est une exigence qui crée le défaut,
la **retirer ou la simplifier** plutôt qu'en ajouter une autre —, et la mention `introduit en P4` (ou `en P3`, `en P2`,
`en P1`) quand le défaut vient d'une remédiation. Toute affirmation de présence ou d'absence = sortie de `grep -nF`
copiée (et, pour un motif complexe, contre-vérifiée). ⛔ **La liste des axes exercés ET non exercés** (axe 0 compris) —
un « 0 finding » sans elle ne compte pas comme passe. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/311-validate-p5-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/311-validate-p5-F.md` (lentille F) ; dernier message : le chemin du rapport, le
bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` — **aucun fichier
temporaire**, ni dans `/tmp` ni dans un dossier de travail : comparaisons en pipe seulement. Lecture seule stricte :
**aucun `git checkout`, `git switch`, `git checkout --detach` ni aucune autre forme** ; aucune commande qui écrit,
compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`,
`scripts/regen-test-schema.sh`), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`stash`/`fetch`/`pull`/`reset`/
`rebase`/`tag`/`push`, `cargo`, `npm`, `npx`, `node`, **`python3`, `python`, `perl`, `ruby` et tout autre
interpréteur**, `sed -i`, `docker`, **`mariadb`, `mysql` et toute requête SQL**, `bash` sur un script du dépôt, `gh` en
écriture. `pdftotext` vers la **sortie standard** seulement. Aucune donnée réelle de Guy ne se lit ni ne se cite (rien
de `~/kesh-test/`). Autorisés : lecture, `grep`, `sed -n`, `awk` en lecture, `ls`, `git log`/`show`/`diff`/`blame`,
`gh api` en lecture, lecture des sources des crates sous `~/.cargo/registry/src/` (versions du `Cargo.lock`),
`pdftotext … -`.
