# Prompt — validation P2 de la spec, Story 31-1 (une sauvegarde qui restaure tout, faite par Kesh)

*Versionné le 2026-10-23. Passe **complète**, deux lentilles, contexte frais chacune, en lecture seule stricte.
Rotation (D6, rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 a été
faite par **Sonnet** et remédiée par Opus : P2 est confiée à **Opus** pour les deux lentilles — en sachant que la
remédiation a été écrite par le même modèle, ce qui rend l'axe 0 ci-dessous d'autant plus nécessaire.*

Dépôt `/home/gcorbaz/devel/kesh-31-1`, branche `story/31-1` (base `44ef2079`). Fiche :
`_bmad-output/implementation-artifacts/31-1-sauvegarde-complete.md` (une fiche, **trois** stories : 31-1a AC1-AC14 +
AC6b, AC8b ; 31-1b AC15-AC22 + AC18b ; 31-1c AC23-AC40). Registre de la story :
`_bmad-output/implementation-artifacts/epic-31-choix-autonomes-311.md` (C-31-1-1 à 36 ; **25 à 36 sont neufs**). Epic :
`_bmad-output/planning-artifacts/epic-31-tenir-les-livres.md` § 31-1 ; registre de l'epic `epic-31-choix-autonomes.md`
(C-31-0-1 à 6, dont **C-31-0-4 : réglages nouveaux dans l'interface**). Issue #503 et ses deux commentaires de Guy —
**ils font foi** (`gh api repos/guycorbaz/kesh/issues/503` et `…/503/comments`, lecture seule). Rapports de la P1 :
`/home/gcorbaz/devel/kesh-gate-logs/311-validate-p1-R.md` et `311-validate-p1-F.md` ; leur traitement est l'entrée
« Validation P1 » du Change Log de la fiche. Rétrospectives `epic-29-retro-2026-10-23.md` et
`epic-30-retro-2026-10-23.md`, § 4 et § 7. Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le
manuel nommé, les axes déclarés, les sites NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ;
§ Règle de splitting préventif ; § Migration breaking policy P1-P8 ; § Issue Tracking Rule ; § Recette visuelle) ;
`docs/agents-orchestration.md` (dont § 6.1, privilèges).

Le diff de la remédiation se lit par `git diff 2f1da588 HEAD -- _bmad-output/` (lecture seule).

⚠️ Guy tient désormais sa **comptabilité réelle** dans Kesh, et possède des sauvegardes 0.14.0 : tout ce qui rendrait
une sauvegarde existante non importable, ou qui pourrait perdre une pièce, est au moins **HIGH**.

## Axe 0 — d'abord : ce que la remédiation a introduit

*La sévérité ne stagne pas, elle se déplace vers ce qu'on vient d'écrire* (`CLAUDE.md` § La passe ciblée). Avant tout
autre axe, relis le diff de la remédiation et cherche :

- les **contradictions** entre un AC remédié et un AC, une tâche, une mutation, un angle mort ou un choix qui ne l'a pas
  été (exemples à contrôler : statut `partial` contre AC24, AC31(a) « tout statut autre que `ok` », AC36, AC37, AC40 ;
  `last_complete_file` contre le décompte de l'AC36 « restent les 3 plus récentes » ; `BackupEnv` contre toute
  mention restante de `FreeSpace` ; la fenêtre de verrou de l'AC11/AC27 contre l'AC7 « avant le verrou » et le corps
  actuel de `run_backup_and_restore` ; le `tokio::spawn` de l'AC8b contre le verrou `FOR UPDATE` tenu par la
  transaction, l'audit, et l'extraction `State`/`CurrentUser` du handler) ;
- les **conceptions nouvelles** qui n'ont jamais été relues : AC6b (bornes, ensemble d'entrées attendu — rejette-t-il
  une sauvegarde **authentique** de format 2 ou 1 ? que fait-il de l'entrée `files/` elle-même, d'entrées dossier
  `data/` éventuelles ?) ; AC8b (la tâche détachée survit-elle à l'arrêt du processus ? libère-t-elle le verrou ?) ;
  AC10/T-a0 (la fixture peut-elle réellement être produite au commit `44ef2079` par un test d'intégration ? le
  téléchargement d'un PDF figé y est-il possible ? la fixture contient-elle un secret ou une donnée réelle ?) ;
  AC18b (les options du client sont-elles compatibles avec `requestRaw`, le rafraîchissement sur 401 et
  `fetchWithRetry` ; une heure est-elle cohérente avec le `proxy_read_timeout` prescrit au manuel ?) ; AC28
  (relecture de l'archive) ; AC29 (`last_complete_file` supprimé à la main, renommé, ou sorti de `auto/`) ; AC30
  (`ErrorKind::QuotaExceeded` et `StorageFull` existent-ils, stables, sur la chaîne `rust-toolchain.toml` ? codes bruts
  28 et 122 justes sous Linux ?) ; AC31 (cadence de 30 minutes, cas `last_success_at` nul) ; AC32 (liste réduite à deux
  sortes : la sauvegarde pré-import a-t-elle bien un nom reconnaissable par une expression, cf. `routes/admin.rs:538`) ;
- les **affirmations neuves** de la fiche et du registre (numéros de ligne, noms de fonctions, nombre de sites,
  comportements d'API de crates) — chacune se revérifie, comme une prémisse ;
- les **recomptes** de la note « Validation P1 » (43 AC, 26 tâches, 39 mutations, 36 choix) et la cohérence des
  libellés de tâches et de mutations avec les AC qu'ils citent.

Un défaut introduit par la remédiation se signale comme tel (`introduit en P1`), avec le passage du diff.

## Axes prioritaires (communs aux deux lentilles)

1. **L'inventaire est-il vrai ?** Chaque ligne P-1 à P-14 et « Ce qui existe et se réutilise » se revérifie au code
   (`grep -nF`, `sed -n`) — la P1 les a jugées justes ; contrôle ce qu'elle déclare n'avoir pas exercé (fin des
   rapports P1 : `Dockerfile` et propriétaire de `./backup`, `docker-compose.dev.yml`, les `.ftl`, les specs
   `backup-restore*.spec.ts` face à la tâche de fond de 5 minutes, `audit_label_registry.rs`, `docs/testing.md`,
   `docs/operations.md`, manuels DE/IT/EN). Une demande de #503 ou des commentaires de Guy laissée sans AC est au moins
   **HIGH**.
2. **Périmètre et découpage** : chaque AC se rattache-t-il à #503 ? Chaque sous-story est-elle livrable et testable
   seule ? Le décompte des modules tient-il — et en particulier le classement de `scripts/regen-test-schema.sh` sous
   `kesh-db` (C-31-1-36) et du client HTTP sous la 31-1b (C-31-1-25 : entre le merge de la 31-1a et celui de la 31-1b,
   l'écran d'export est-il cassé pour une installation avec pièces, et cela compte-t-il sans release intermédiaire) ?
3. **Format et compatibilité** : un manifeste 1 relu ; la fixture 0.14.0 réelle ; la cohérence de l'AC7 limitée au
   format 2 ; colonne `pdf_storage_path` absente ; règle d'anomalie par ligne (AC1) pour **chaque** cas : deux sociétés,
   `discarded`, facture refigée (ancienne et nouvelle empreinte), `filesNotIncluded`, même chemin dans deux lignes avec
   la même empreinte. Peut-elle refuser une sauvegarde **authentique** ?
4. **Migration** (P1-P8, 31-1c) : non breaking à juste titre (colonnes et `CHECK` nouveaux) ; `INSERT` singleton et
   triage P7 ; audit P5 et ses **trois** sites ; P6 ; squash et sa garde ; la table ne déplace pas la fenêtre
   d'importabilité.
5. **Manuel contre code** (obligatoire) : `docs/manual/fr/admin-manual.tex` § *Sauvegarde et restauration*
   (l. 1450-1965) et les exemples de proxy inverse (l. 320-345 Nginx, ~358 Caddy, ~377 Traefik, ~616 Portail DSM),
   `user-manual.tex` (l. 1155-1161, 1909-1910), `docs/api-external.md` (l. 721, 751). L'**inventaire clos** de l'AC14
   est-il réellement clos ? Rejoue le grep des valeurs (`503`, `512`, `n'emporte`, `sans les fichiers`, `aucun binaire`,
   `files/`, `formatVersion`, `50M`, `300s`) sur tout le dépôt et confronte-le au tableau ; un site manquant est un
   finding. Ce que les AC14, AC21, AC34 et AC39 prescrivent d'écrire sera-t-il vrai au code prescrit (activée par
   défaut, `./backup/auto`, plafond 2048, `partial`, dernière complète protégée, volume, proxy) ? Contrôle aussi le
   **PDF**, aplati vers la **sortie standard seulement** :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`.
6. **Recomptes** : 43 AC, 26 tâches (T-a0 à T-a9 avec T-a4b et T-a6b ; T-b1 à T-b7 ; T-c1 à T-c7), 39 mutations
   (M-a1 à M-a17, M-b1 à M-b6, M-c1 à M-c16), 36 choix, deux specs E2E neuves, chaque numéro de ligne cité — recompte
   depuis la source.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque demande de #503 et de ses commentaires couverte par un AC testable, chaque AC par une tâche et par un test
  **qui rougirait** ; chaque mutation tuée par le test nommé (une mutation équivalente non déclarée est un finding —
  contrôle en particulier les **neuves** M-a12 à M-a17, M-b4 à M-b6, M-c11 à M-c16, et M-a7 reformulée) ; chaque fait,
  chemin et numéro de ligne revérifié. La liste des « gardes existantes qui rougiront » est-elle désormais complète ?
  Les tests T-a4b et T-a6b sont-ils réalisables tels qu'écrits (observateur du débordement dans le dossier de
  sauvegarde, refus d'écriture, aucun temporaire résiduel) ? Écrans : rôles, états, largeur étroite, thème clair ;
  captures des AC21 et AC40 — chaque état touché y est-il, et chaque état listé est-il atteignable ? Textes i18n ×4
  (dont `admin-restore-error-unknown-outcome`) et fallbacks.
- **F — Faisabilité, sécurité, verrous et cohérence (adversaire de la conception)** :
  - **Sécurité** : traversée de chemin à l'export (AC3, `open_document`) et à l'import (AC6, AC6b) ; téléchargement des
    sauvegardes du serveur (AC32 : deux sortes, fichiers réguliers) ; archive piégée (AC6b : bornes réellement
    appliquées à **toutes** les lectures, y compris celles de `check_schema_compat` et de la restauration) ; fermeture
    aux clés d'API et aux non-Admin ; modes `0600`/`0700`.
  - **Verrous et ordre** : relis `run_backup_and_restore` (`routes/admin.rs:225-495`) et `full_import` ; avec le
    `tokio::spawn` de l'AC8b et la nouvelle fenêtre de l'AC11/AC27 — ordre d'acquisition `backup_lock` → `FOR UPDATE`
    identique partout ? un verrou peut-il rester tenu par une tâche détachée qui panique ? l'export qui rend
    `backup_lock` avant la diffusion laisse-t-il un temporaire partagé ? la réservation `last_attempt_at <=> ?` avec
    des horodatages liés depuis l'application.
  - **Instantané et fichiers** : `referenced_files` lus dans le même `ReadSnapshot` ; un fichier peut-il changer entre
    l'instantané et sa lecture (refigeage, réimport) ?
  - **Faisabilité** : `tempfile` 3.27 (`spooled_tempfile_in`, `tempfile_in`) ; `zip` 2.4.2 (`ZipArchive<File>`, noms en
    double, `Stored`) ; `rustix::fs::statvfs` ; `field.chunk()` sous `DefaultBodyLimit` ; `ErrorKind` sur la chaîne du
    dépôt ; les neuf sites `AppState` ; les gardes lexicales (`build_keshbackup_ne_lit_la_base_qu_une_fois`,
    `avant_sauvegarde_branchee_aux_appels_de_l_import`, la neuve de l'AC8b) restent-elles exactes et non contournables ?
  - **Disque** : seuil de l'AC30 ; rétention avec `last_complete_file` (peut-elle supprimer la seule sauvegarde valide,
    un fichier qui n'est pas le sien, ou ne plus rien supprimer jamais ?) ; `partial` permanent d'une installation
    restaurée naguère sans ses documents (C-31-1-4) : le bandeau reste-t-il allumé à vie, et est-ce acceptable ?

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve**
(commande exécutée et sortie copiée), correction proposée, et la mention `introduit en P1` quand le défaut vient de la
remédiation. Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés
ET non exercés** (axe 0 compris) — un « 0 finding » sans elle ne compte pas comme passe. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/311-validate-p2-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/311-validate-p2-F.md` (lentille F) ; dernier message : le chemin du rapport, le
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
