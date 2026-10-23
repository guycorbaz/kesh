# Prompt — validation P4 de la spec, Story 31-1 (une sauvegarde qui restaure tout, faite par Kesh)

*Versionné le 2026-10-23. Passe **complète**, deux lentilles, contexte frais chacune, en lecture seule stricte.
Rotation (D6, rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 :
Sonnet ; P2 : Opus ; P3 : Sonnet ; les trois remédiations ont été écrites par Opus. P4 est confiée à **Opus** pour les
deux lentilles. Une boucle ne se clôt que sur une passe à **0 MEDIUM+**.*

Dépôt `/home/gcorbaz/devel/kesh-31-1`, branche `story/31-1` (base `44ef2079`). Fiche :
`_bmad-output/implementation-artifacts/31-1-sauvegarde-complete.md` (une fiche, **trois** stories : 31-1a AC1-AC14 +
AC6b, AC8b ; 31-1b AC15-AC22 + AC18b ; 31-1c AC23-AC40). Registre de la story :
`_bmad-output/implementation-artifacts/epic-31-choix-autonomes-311.md` (C-31-1-1 à 51 ; **45 à 51 sont neufs** ; ils
amendent C-31-1-4, 7, 23, 41 sans les réécrire). Epic : `_bmad-output/planning-artifacts/epic-31-tenir-les-livres.md`
§ 31-1 ; registre de l'epic `epic-31-choix-autonomes.md` (C-31-0-1 à 6, dont **C-31-0-4 : réglages nouveaux dans
l'interface**). Issue #503 et ses deux commentaires de Guy — **ils font foi** (`gh api repos/guycorbaz/kesh/issues/503`
et `…/503/comments`, lecture seule). Rapports précédents : `/home/gcorbaz/devel/kesh-gate-logs/311-validate-p{1,2,3}-{R,F}.md` ;
leur traitement est aux entrées « Validation P1 », « Validation P2 » et « Validation P3 » du Change Log de la fiche.
Arbitrages de l'orchestrateur, **à ne pas rouvrir** : C-31-1-36 (`scripts/regen-test-schema.sh` compté avec `kesh-db`)
et C-31-1-40 (`partial` au panneau seulement). Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le
manuel nommé, les axes déclarés, les sites NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ;
§ Règle de splitting préventif ; § Migration breaking policy P1-P8 ; § Issue Tracking Rule ; § Pattern batch) ;
`docs/agents-orchestration.md` (dont § 6.1, privilèges).

Le diff de la remédiation P3 se lit par `git diff c7639e80 df727023 -- _bmad-output/` (lecture seule).

⚠️ Guy tient désormais sa **comptabilité réelle** dans Kesh, et possède des sauvegardes 0.14.0 : tout ce qui rendrait
une sauvegarde existante — ou une sauvegarde 0.15 **authentique** — non importable, ou qui pourrait perdre une pièce,
est au moins **HIGH**.

⚠️ **Piège d'outil constaté en P3** : le `grep` de ce poste est `ugrep`. Sur un motif comme `.{150}` appliqué à du texte
UTF-8, il s'arrête en erreur (`exceeds complexity limits`) et **ne rend rien** — la P2 a lu cette erreur comme un
résultat vide et réfuté à tort un finding. Un `grep` vide sur un motif complexe se contre-vérifie par `awk` (`index()`)
avant d'être cité comme preuve d'absence.

## Axe 0 — d'abord : ce que la remédiation P3 a écrit

*La sévérité ne stagne pas, elle se déplace vers ce qu'on vient d'écrire* (`CLAUDE.md` § La passe ciblée) : en P3, les
deux MEDIUM convergents (R1 = F1, R5 = F2) venaient de la remédiation P2. Attaque d'abord le diff de la P3, avant tout
autre axe. Questions à trancher, chacune au code ou aux sources des crates (versions du `Cargo.lock`) :

- **Verrou (AC26, AC27, AC32, AC35, C-31-1-45)** : `run_once` prend lui-même `backup_lock` en `try_lock_owned` et rend
  `RunOutcome::Skipped` ; le handler de `POST …/run` n'acquiert rien et traduit `Skipped` en 409 ; la boucle ignore
  `Skipped` ; l'import (`lock_owned().await` dans la fermeture du `spawn`) et l'export manuel (`lock_owned().await`
  pour l'assemblage) restent acquéreurs. Toutes les mentions de la fiche (AC, tâches, mutations M-c6, M-c17, M-c18,
  angles morts, § Pièges) sont-elles désormais **une** règle ? AC35 et ses deux mutations sont-elles tuables telles
  qu'écrites ? La réservation perdue rendue en `Skipped` (donc 409 pour `POST …/run`) est-elle juste ? Le garde de
  l'export, relâché quand le client part alors que le `spawn_blocking` continue (AC27, angle mort P3), ouvre-t-il un
  cas pire que celui que la fiche écrit ? L'ordre `backup_lock` → `_kesh_version FOR UPDATE` reste-t-il le seul ?
- **Assemblage (AC4, AC28, AC30, T-a4, T-a5, T-c3, C-31-1-46)** : lecture asynchrone de la base, puis **un seul**
  `spawn_blocking` autour de `write_keshbackup<W: Write + Seek>` synchrone ; `write_backup_file` synchrone à fermeture
  de remplissage sur un clone (`File::try_clone`) dont l'original reçoit `sync_all` ; `wrap_writer(File) -> Box<dyn
  Write + Seek + Send>` ; `Arc<dyn BackupEnv>`. Est-ce **compilable** tel qu'écrit (bornes `Send`/`'static` du
  `spawn_blocking`, `SpooledTempFile` repris au retour pour la diffusion, `ZipWriter<W>::finish` qui rend `W`) ? Un
  `sync_all` sur l'original voit-il les écritures faites par le clone ? Le `seek` du `ZipWriter` sur un clone partage-t-il
  la position avec l'original, et cela gêne-t-il ? La sauvegarde pré-import, appelée sous le verrou `FOR UPDATE` dans la
  tâche de l'import, s'accommode-t-elle de ce `spawn_blocking` ? Les gardes lexicales existantes de `routes/admin.rs`
  (`avant_sauvegarde_branchee_aux_appels_de_l_import`, `echecs_avant_sauvegarde_tous_convertis`, l. 881 et 957) et
  `build_keshbackup_ne_lit_la_base_qu_une_fois` restent-elles tenables, et la fiche dit-elle juste ce qui rougit et ce
  qui deviendrait muet ?
- **Fichier illisible (AC3, M-a20, C-31-1-47)** : raison `unreadable`, deux passes, abandon de la sauvegarde sur
  divergence à la copie. Une archive que l'import refuserait peut-elle encore sortir (export, pré-import, automatique) ?
  L'import accepte-t-il la valeur `unreadable` dans `filesNotIncluded` (désérialisation du manifeste) ? Le test `chmod
  000` est-il fiable (garde « échoue bruyamment en `root` ») ? La déclaration d'équivalence du re-hachage tient-elle ?
- **Inventaire (AC14, T-a8, C-31-1-48)** : rejoue la commande écrite à l'AC14 (elle doit rendre **94 lignes** à
  `c7639e80` ; sur `HEAD` aussi, la fiche étant hors de son périmètre). Chaque ligne est-elle au tableau ou triable
  « intacte » ? Le motif manque-t-il une forme (autre graphie LaTeX, `README` hors `README.md`, `docker-compose*.yml`,
  `docs/testing.md`) qui porte une phrase que la 31-1 rend fausse ?
- **Bornes (AC6b, AC7, AC8, M-a14, C-31-1-49)** : borne cumulée `TABLES_MAX_BYTES`, `ReadLimits`, `byteSize` plafonné et
  `saturating_add`. Une sauvegarde authentique peut-elle être refusée (installation dont les tables dépassent 1 Gio au
  total, `byteSize` légitime au-dessus du plafond d'import) ? Les deux sont-ils au moins écrits en angle mort ?
- **Routes (AC32, C-31-1-50)** : `ensure_not_pat` sur le seul téléchargement ; `GUARDED_BY_HANDLER` et le compte de 6
  chemins discriminants contre `admin_pat_denied_e2e.rs` (`admin_head_paths`, l. 840-870) ; `(8, 25)` et 33 constructeurs.
- **Lancement de la boucle (AC26, M-c21)** : la garde lexicale (`auto_backup::spawn(` une fois dans `main.rs`, aucune
  dans `lib.rs`) rougit-elle sous M-c21, et peut-elle passer à vide ? Que reste-t-il, en angle mort, de la demande de
  #503 « faite par Kesh lui-même » ?
- **Contradictions** : chaque AC remédié contre les AC, tâches, mutations, angles morts, captures et choix qui ne l'ont
  pas été (ex. : AC3 et AC5/AC18 sur les raisons ; AC30 et AC33/AC39 sur `insufficient_space` ; AC21/AC40 et les voies
  d'obtention des captures ; C-31-1-45 contre C-31-1-23 et C-31-1-41).
- **Affirmations neuves** (numéros de ligne, comportements de `zip` 2.4.2 `result.rs:52-64`, `tokio` 1.52.3,
  `tempfile` 3.27.0, branches `story/31-2` et `story/31-3` et leur section `## [0.15.0] — Non publié`, annotation datée
  de la réfutation P2) — chacune se revérifie, comme une prémisse.
- **Recomptes** de la note « Validation P3 » (43 AC, 26 tâches, 48 mutations dont 20 M-a, 7 M-b, 21 M-c, 51 choix) et
  cohérence des libellés de mutations neuves ou reformulées (M-a4, M-a6, M-a12, M-a14, M-a20, M-c6, M-c17, M-c21) avec
  les tests qu'elles nomment.

Un défaut introduit par la remédiation P3 se signale comme tel (`introduit en P3`), avec le passage du diff.

## Axes prioritaires (communs aux deux lentilles)

1. **L'inventaire est-il vrai ?** P-1 à P-14 et « Ce qui existe et se réutilise » au code (`grep -nF`, `sed -n`) ;
   contrôle surtout ce que les passes précédentes déclarent n'avoir pas exercé (fin de leurs rapports :
   `docker-compose.dev.yml`, les `.ftl` de-CH/it-CH/en-CH, `audit_label_registry.rs`, `audit_route_registry.rs`,
   `scripts/regen-test-schema.sh` l. 254 et 273-327, `test_schema_guard.rs` l. 416, `kesh-seed`,
   `crates/kesh-db/src/repositories`, les E2E et `helpers/inbox.ts`). Une demande de #503 ou de ses commentaires laissée
   sans AC est au moins **HIGH**.
2. **Périmètre et découpage** : chaque AC se rattache-t-il à #503 ? Chaque sous-story est-elle livrable et testable
   seule ? Le décompte des modules (31-1a : 3, 31-1b : 4, 31-1c : 5) tient-il après la P3 ?
3. **Format et compatibilité** : manifeste 1 relu ; fixture 0.14.0 réelle ; AC7 limité au format 2 ; colonne
   `pdf_storage_path` absente ; règle d'anomalie par ligne (AC1) pour chaque cas, **aller-retour compris** ; valeur
   `unreadable`. Peut-elle refuser une sauvegarde authentique ?
4. **Migration** (P1-P8, 31-1c) : non breaking à juste titre ; `INSERT` singleton et triage P7 ; audit P5 et ses trois
   sites ; P6 ; squash et sa garde ; fenêtre d'importabilité inchangée.
5. **Manuel contre code** (obligatoire) : `docs/manual/fr/admin-manual.tex` § *Sauvegarde et restauration*
   (l. 1450-1965), les exemples de proxy inverse (l. 320-345 Nginx, ~358 Caddy, ~377 Traefik, ~616 Portail DSM), les
   sites de l'inventaire (dont les huit ajoutés en P3 : `.env.example:88-90, 182-183, 187-192`, `README.md:93`,
   `config.rs:240-243, 252-261`, `export.rs:44-55`, `routes/admin.rs:118-130`), `user-manual.tex` (l. 1155-1161,
   1909-1910), `docs/api-external.md` (l. 721, 751). Ce que les AC14, AC21, AC34 et AC39 prescrivent d'écrire sera-t-il
   vrai au code prescrit (en particulier l'issue d'`insufficient_space` et l'exemple Nginx répété) ? Contrôle aussi le
   **PDF**, aplati vers la **sortie standard seulement** : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' |
   tr -s ' '`.
6. **Recomptes** : 43 AC, 26 tâches, 48 mutations, 51 choix, deux specs E2E neuves, chaque numéro de ligne cité —
   recompte depuis la source.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque demande de #503 et de ses commentaires couverte par un AC testable, chaque AC par une tâche et par un test
  **qui rougirait** ; chaque mutation tuée par le test nommé (une mutation équivalente non déclarée est un finding) ;
  chaque fait, chemin et numéro de ligne revérifié. La liste des « gardes existantes qui rougiront » distingue-t-elle
  celles qui **rougissent** de celles qui deviendraient **muettes** ? Écrans : rôles ARIA, états, largeur étroite,
  thème clair ; captures des AC21 et AC40 — chaque état touché y est-il, chaque état listé est-il atteignable par la
  voie que la fiche donne ? Textes i18n ×4 et fallbacks, pour la 31-1b **et** la 31-1c.
- **F — Faisabilité, sécurité, verrous et cohérence (adversaire de la conception)** :
  - **Sécurité** : traversée à l'export (AC3, `open_document` et son test direct) et à l'import (AC6, AC6b) ; chemins
    de `filesNotIncluded` libres — jamais joints, jamais interpolés ; téléchargement des sauvegardes du serveur (AC32,
    seul porteur d'`ensure_not_pat`) ; archive piégée (bornes appliquées à **toutes** les lectures, y compris
    `check_schema_compat` et la restauration en flux) ; fermeture aux clés d'API et aux non-Admin ; modes `0600`/`0700`.
  - **Verrous et ordre** : relis `run_backup_and_restore` (`routes/admin.rs:225-495`) et `full_import` contre l'AC27
    réécrite ; un garde possédé dans une tâche qui panique est-il rendu ? `POST …/run` peut-il rendre 409 à tort, ou
    lancer deux écritures ?
  - **Instantané et fichiers** : `referenced_files` dans le même `ReadSnapshot` ; deux passes de lecture des fichiers
    hors instantané — est-ce sans conséquence ?
  - **Disque et durabilité** : seuil de l'AC30 ; rétention avec `last_complete_file` ; `sync_all` sur l'original après
    écriture par le clone ; `sync_all` du dossier des documents (AC8) ; abandon d'une sauvegarde à mi-copie (le
    `.partial` est-il supprimé dans tous les chemins ?).

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve**
(commande exécutée et sortie copiée), correction proposée — en préférant, quand c'est une exigence qui crée le défaut,
la **retirer ou la simplifier** plutôt qu'en ajouter une autre —, et la mention `introduit en P3` (ou `en P2`, `en P1`)
quand le défaut vient d'une remédiation. Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée (et,
pour un motif complexe, contre-vérifiée). ⛔ **La liste des axes exercés ET non exercés** (axe 0 compris) — un
« 0 finding » sans elle ne compte pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/311-validate-p4-R.md`
(lentille R) ou `/home/gcorbaz/devel/kesh-gate-logs/311-validate-p4-F.md` (lentille F) ; dernier message : le chemin du
rapport, le bilan par sévérité, une ligne par finding MEDIUM ou plus.

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
