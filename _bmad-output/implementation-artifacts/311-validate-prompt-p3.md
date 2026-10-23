# Prompt — validation P3 de la spec, Story 31-1 (une sauvegarde qui restaure tout, faite par Kesh)

*Versionné le 2026-10-23. Passe **complète**, deux lentilles, contexte frais chacune, en lecture seule stricte.
Rotation (D6, rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 :
Sonnet ; P2 : Opus ; les deux remédiations ont été écrites par Opus. P3 est confiée à **Sonnet** pour les deux lentilles
— un modèle orthogonal à l'auteur des patches.*

Dépôt `/home/gcorbaz/devel/kesh-31-1`, branche `story/31-1` (base `44ef2079`). Fiche :
`_bmad-output/implementation-artifacts/31-1-sauvegarde-complete.md` (une fiche, **trois** stories : 31-1a AC1-AC14 +
AC6b, AC8b ; 31-1b AC15-AC22 + AC18b ; 31-1c AC23-AC40). Registre de la story :
`_bmad-output/implementation-artifacts/epic-31-choix-autonomes-311.md` (C-31-1-1 à 44 ; **37 à 44 sont neufs**, et ils
amendent C-31-1-5, 11, 19, 20, 23, 27, 29, 30 sans les réécrire). Epic : `_bmad-output/planning-artifacts/epic-31-tenir-les-livres.md`
§ 31-1 ; registre de l'epic `epic-31-choix-autonomes.md` (C-31-0-1 à 6, dont **C-31-0-4 : réglages nouveaux dans
l'interface**). Issue #503 et ses deux commentaires de Guy — **ils font foi** (`gh api repos/guycorbaz/kesh/issues/503`
et `…/503/comments`, lecture seule). Rapports précédents : `/home/gcorbaz/devel/kesh-gate-logs/311-validate-p1-{R,F}.md`
et `311-validate-p2-{R,F}.md` ; leur traitement est aux entrées « Validation P1 » et « Validation P2 » du Change Log de
la fiche. Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les
sites NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Règle de splitting préventif ;
§ Migration breaking policy P1-P8 ; § Issue Tracking Rule) ; `docs/agents-orchestration.md` (dont § 6.1, privilèges).

Le diff de la remédiation P2 se lit par `git diff edb39a96 640ffec2 -- _bmad-output/` (lecture seule).

⚠️ Guy tient désormais sa **comptabilité réelle** dans Kesh, et possède des sauvegardes 0.14.0 : tout ce qui rendrait
une sauvegarde existante — ou une sauvegarde 0.15 **authentique** — non importable, ou qui pourrait perdre une pièce,
est au moins **HIGH**.

## Axe 0 — d'abord : ce que la remédiation P2 a écrit

*La sévérité ne stagne pas, elle se déplace vers ce qu'on vient d'écrire* (`CLAUDE.md` § La passe ciblée) : sur les
deux passes précédentes, plus de la moitié des MEDIUM venaient de la remédiation d'avant. Attaque d'abord le diff de la
P2, avant tout autre axe. Questions à trancher, chacune au code ou aux sources des crates :

- **Retraits** (la P2 a retiré plutôt qu'ajouté) : le retrait de l'expression sur `filesNotIncluded` (AC6,
  C-31-1-37) ouvre-t-il quelque chose — journal, en-tête, réponse, calcul de `filesMissing`, AC7 (égalité de chaînes),
  doublons entre listes ? Le retrait du refus des doublons et de `MAX_ENTRIES` (AC6b, C-31-1-38) : la justification
  « seule l'entrée retenue est lue, et vérifiée par son empreinte » tient-elle pour `manifest.json` lui-même et pour
  les `data/*.ndjson` ? L'angle mort de `ZipArchive::new` est-il borné comme la fiche le dit (lis `read.rs` autour de
  l. 690-712 de `zip` 2.4.2) ? Le retrait de `partial` du bandeau de l'application (C-31-1-40) laisse-t-il un cas
  qu'une demande de #503 ou un commentaire de Guy exige de voir hors du panneau ?
- **Ajouts** : l'ordre de l'AC6b (manifeste par `by_name`, puis ensemble attendu pour le format 2, puis tables, puis
  fichiers) est-il réalisable avec l'API de `zip` 2.4.2 et compatible avec la boucle actuelle de `parse_and_verify`
  (`import.rs:39-175`) ? Rejette-t-il une sauvegarde **authentique** de format 1 ou 2 (entrée `files/`, entrées dossier
  éventuelles) ? La restauration **en flux** (AC8, T-a5 : `restore_document(…, source: impl Read, byte_size, …)`)
  est-elle compatible avec l'écriture atomique factorisée de `store_document` et avec la lecture d'un
  `ZipArchive<File>` dans un `spawn_blocking` ? Les **gardes possédés** de l'AC27 (`lock_owned`, `try_lock_owned` sur
  `Arc<tokio::sync::Mutex<()>>`) existent-ils dans la version de `tokio` du `Cargo.lock`, et la garde lexicale
  proposée (aucune forme empruntée, `lock_owned` dans la fermeture du `spawn`) est-elle exacte et non contournable ?
  L'ordre `backup_lock` → `FOR UPDATE` est-il le seul possible partout ? Le seuil de l'AC30
  (`max(last_success_bytes, documents_bytes) × 1,2 + 256 Mio`, premier niveau de `KESH_DOCUMENTS_DIR`) est-il un
  majorant réel (`store_document` n'écrit-il vraiment qu'au premier niveau ?) ; `wrap_writer<'a>(&'a mut File)` est-il
  compatible avec un `write_backup_file` asynchrone (`tokio::fs`) et un `build_keshbackup_into` synchrone ? Les noms à
  la milliseconde (AC28, AC29, AC32 : `\d{8}T\d{9}Z`, et pour `pre-import` `\d{8}T\d{9}-\d+-\d+`) correspondent-ils au
  format réellement écrit (`routes/admin.rs:535-541`, `%Y%m%dT%H%M%S%3f`) ? Le test de débordement de T-a4b (dossier
  non inscriptible, seuil 0 → 500) rougit-il sous M-a19 **et** reste-t-il vert sans elle — que fait `ensure_backup_dir`
  d'un dossier existant non inscriptible, et que fait `SpooledTempFile` de seuil 0 à la première écriture ? Le clic
  *Se reconnecter* systématique (AC19, C-31-1-39) contredit-il l'AC16, l'AC20, l'AC21, les tests existants de
  `AdminRestorePanel.test.ts` ? Les 502/504 de l'AC18b : Kesh n'en émet-il vraiment aucun (y compris par une couche
  d'`axum`/`tower`) ?
- **Contradictions** : chaque AC remédié contre les AC, tâches, mutations, angles morts, captures et choix qui ne
  l'ont pas été (exemples : AC31 contre AC33, AC37, AC39, AC40 (o) ; AC27 contre AC8b, AC11, AC32, AC35, C-31-1-23 et
  C-31-1-30 ; AC6 contre AC1, AC3, AC9, T-a1 ; la signature unique de `run_once` (AC26) contre AC35 et AC36).
- **Affirmations neuves** de la fiche et du registre (numéros de ligne, nombre de sites, comportements de crates,
  inventaire de l'AC14 étendu, constat R6 réfuté pour `admin-manual.tex:2199`) — chacune se revérifie, comme une
  prémisse.
- **Recomptes** de la note « Validation P2 » (43 AC, 26 tâches, 46 mutations dont 19 M-a, 7 M-b, 20 M-c, 44 choix) et
  cohérence des libellés de mutations neuves (M-a18, M-a19, M-b7, M-c17 à M-c20) avec les tests qu'elles nomment.

Un défaut introduit par la remédiation P2 se signale comme tel (`introduit en P2`), avec le passage du diff.

## Axes prioritaires (communs aux deux lentilles)

1. **L'inventaire est-il vrai ?** P-1 à P-14 et « Ce qui existe et se réutilise » au code (`grep -nF`, `sed -n`) ;
   contrôle surtout ce que les passes P1 et P2 déclarent n'avoir pas exercé (fin de leurs rapports :
   `docker-compose.dev.yml`, les `.ftl` de-CH/it-CH/en-CH, manuels DE/IT/EN, `audit_label_registry.rs`,
   `audit_route_registry.rs`, `reset_demo` contre `_kesh_backup_schedule`, `scripts/regen-test-schema.sh` l. 254 et
   273-327, `test_schema_guard.rs` l. 416). Une demande de #503 ou des commentaires de Guy laissée sans AC est au moins
   **HIGH**.
2. **Périmètre et découpage** : chaque AC se rattache-t-il à #503 ? Chaque sous-story est-elle livrable et testable
   seule ? Le décompte des modules (31-1a : 3, 31-1b : 4, 31-1c : 5) tient-il après la P2 ?
3. **Format et compatibilité** : manifeste 1 relu ; fixture 0.14.0 réelle ; AC7 limité au format 2 ; colonne
   `pdf_storage_path` absente ; règle d'anomalie par ligne (AC1) pour chaque cas, **aller-retour compris** (ligne hors
   forme → export → import). Peut-elle refuser une sauvegarde authentique ?
4. **Migration** (P1-P8, 31-1c) : non breaking à juste titre ; `INSERT` singleton et triage P7 ; audit P5 et ses trois
   sites ; P6 ; squash et sa garde ; fenêtre d'importabilité inchangée.
5. **Manuel contre code** (obligatoire) : `docs/manual/fr/admin-manual.tex` § *Sauvegarde et restauration*
   (l. 1450-1965), les exemples de proxy inverse (l. 320-345 Nginx, ~358 Caddy, ~377 Traefik, ~616 Portail DSM), les
   sites ajoutés à l'inventaire par la P2 (l. 196, 782, 784, 843, 1466, 1608, 1946, 1947, 2268), `user-manual.tex`
   (l. 1155-1161, 1909-1910), `docs/api-external.md` (l. 721, 751). L'inventaire clos de l'AC14 est-il **clos** ?
   Rejoue le grep des valeurs (`503`, `512`, `n'emporte`, `sans les fichiers`, `aucun binaire`, `files/`,
   `formatVersion`, `50M`, `300s`) **et** des sujets (`KESH_ADMIN_BACKUP_DIR`, `INMEM`, `temporaire`, `./backup`,
   `keshbackup`) sur tout le dépôt ; un site manquant est un finding. Ce que les AC14, AC21, AC34 et AC39 prescrivent
   d'écrire sera-t-il vrai au code prescrit ? Contrôle aussi le **PDF**, aplati vers la **sortie standard seulement** :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`.
6. **Recomptes** : 43 AC, 26 tâches, 46 mutations, 44 choix, deux specs E2E neuves, chaque numéro de ligne cité —
   recompte depuis la source.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque demande de #503 et de ses commentaires couverte par un AC testable, chaque AC par une tâche et par un test
  **qui rougirait** ; chaque mutation tuée par le test nommé (une mutation équivalente non déclarée est un finding —
  contrôle en particulier les neuves M-a18, M-a19, M-b7, M-c17 à M-c20, et M-b2, M-b6, M-c11 reformulées) ; chaque
  fait, chemin et numéro de ligne revérifié. La liste des « gardes existantes qui rougiront » est-elle complète, et
  distingue-t-elle celles qui **rougissent** de celles qui deviendraient **muettes** ? Écrans : rôles ARIA (un seul par
  statut, `role="status"` au layout), états, largeur étroite, thème clair ; captures des AC21 et AC40 — chaque état
  touché y est-il, chaque état listé est-il atteignable ? Textes i18n ×4 et fallbacks.
- **F — Faisabilité, sécurité, verrous et cohérence (adversaire de la conception)** :
  - **Sécurité** : traversée à l'export (AC3, `open_document`) et à l'import (AC6, AC6b) ; chemins de
    `filesNotIncluded` désormais libres — jamais joints, jamais interpolés ? téléchargement des sauvegardes du serveur
    (AC32) ; archive piégée (AC6b, AC7, AC8 en flux : bornes appliquées à **toutes** les lectures, y compris
    `check_schema_compat` et la restauration) ; fermeture aux clés d'API et aux non-Admin (dont `/onboarding/restore`) ;
    modes `0600`/`0700`.
  - **Verrous et ordre** : relis `run_backup_and_restore` (`routes/admin.rs:225-495`) et `full_import` contre l'AC27
    réécrite ; un garde possédé déplacé dans une tâche qui panique est-il rendu ? `POST …/run` peut-il rendre 409 à
    tort, ou lancer deux tâches ?
  - **Instantané et fichiers** : `referenced_files` dans le même `ReadSnapshot` ; `documents_bytes` lu hors
    instantané — est-ce sans conséquence ?
  - **Disque** : seuil de l'AC30 ; rétention avec `last_complete_file` et noms à la milliseconde ; `sync_all` après
    `wrap_writer` ; statut `partial` désormais visible au seul panneau.

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve**
(commande exécutée et sortie copiée), correction proposée — en préférant, quand c'est une exigence qui crée le défaut,
la **retirer ou la simplifier** plutôt qu'en ajouter une autre —, et la mention `introduit en P2` (ou `en P1`) quand le
défaut vient d'une remédiation. Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée. ⛔ **La liste
des axes exercés ET non exercés** (axe 0 compris) — un « 0 finding » sans elle ne compte pas comme passe. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/311-validate-p3-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/311-validate-p3-F.md` (lentille F) ; dernier message : le chemin du rapport, le
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
