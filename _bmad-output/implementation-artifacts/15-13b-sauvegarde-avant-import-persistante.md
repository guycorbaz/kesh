# Story 15.13b : la sauvegarde prise avant un import survit au redémarrage, et son échec est annoncé tel quel

Status: done

<!-- Née le 2026-10-09 du découpage de la 15-13 (`15-13-mariadb-et-sauvegarde.md`, désormais fiche index)
     après la validation P3, décision de l'orchestrateur (signal D5 de recyclage levé deux passes de suite) —
     choix C-15-13-16. Le contenu vient de la fiche unique au commit 8a9bcd27, réparti sans perte (recompte
     aux deux bornes : fiche index, § « Découpage ») ; la remédiation de la validation P3 y est appliquée.
     **La numérotation de la fiche unique est conservée** (AC, tests, mutations, tâches) pour la
     traçabilité : les numéros absents vivent dans la 15-13a. Choix : C-15-13-3, -4, -6, -8, -9, -14, -16, -18,
     -19, -20, -21, -24, -25, -26, -27, -28 et -29 du registre `epic-15-choix-autonomes.md`. Statut `ready-for-dev` : convention du registre pour une
     fiche en cours de validation. -->

**Issues** : **ferme #552 et #576**. La PR porte `closes #552` **et** `closes #576` (mots-clés dans le titre
ou le corps de la PR : le dépôt merge en squash) ; les commits intermédiaires portent `refs #552` /
`refs #576`. **`refs #558`** seulement (dossiers d'hôte configurables : cette story n'en ajoute aucun,
C-15-13-3).

#576 a été absorbée en validation P1 de la fiche unique (C-15-13-9) : avec le défaut `/data/backup`,
l'échec d'écriture de la sauvegarde devient le cas **nominal** de toute instance lancée hors Docker (montage
E2E, développement), et le message affiché promettait alors une sauvegarde qui n'existe pas. C'est pourquoi
#552 et #576 ne se séparent pas : livrer le nouveau défaut sans le nouveau message rendrait le message faux
dans le cas courant.

**Dépendances** : après la **15-11a** et la **15-11b** (mergées dans `de285ea8`) — la variable
`KESH_ADMIN_BACKUP_DIR` est déjà transmise par les deux compose en `${KESH_ADMIN_BACKUP_DIR:-}`, lue par
`config::env_nonempty`, et le test `configuration_transmise.rs` garde la forme de cette transmission et les
trois montages de `kesh-api`. **Indépendante de la 15-13a en code** : aucune des deux n'appelle ce que
l'autre ajoute. Elles touchent les **mêmes fichiers**, à des endroits distincts : `docker-compose.yml`,
`.env.example`, `crates/kesh-api/src/config.rs`, `crates/kesh-api/tests/configuration_transmise.rs`,
`docs/manual/fr/admin-manual.tex` (§ *Passer à la 0.13.0*), `DOCKER_START.md`, `CHANGELOG.md` `[0.13.0]` —
et les **PDF versionnés** `admin-manual.pdf` et `marketing-brochure.pdf`, que les deux régénèrent.
La seconde mergée rebase et **recompte les gestes** du paragraphe « Pour qui garde son fichier compose »
(AC 11 f) ; un conflit sur un PDF ne se rebase pas : on fusionne les `.tex`, puis on **régénère** les PDF
(`make fr`) et on rejoue les contrôles aplatis des **deux** fiches (R4-3 de la validation P4 de la 15-13a).
**Conflits certains au merge** (F-P6-1 de la validation P6) : `CHANGELOG.md` — le paragraphe d'une seule
ligne « ⚠️ Action requise » (`:46` à la spécification, `:56` au `HEAD` `200f5e79`) porte les cinq phrases
des deux fiches, dont **une même phrase** (« Une variable que `.env` ne pose pas… prend son défaut sans
avertissement — la sauvegarde pré-import reste dans `/tmp` ») réécrite par la 15-13a au début et par la
15-13b à la fin — et `admin-manual.tex`, le paragraphe « Pour qui garde son fichier compose »
(`:1793` à la spécification, `:1796` au `HEAD`). Git fusionne par lignes : la **seconde** mergée résout à la
main, en relisant la phrase « défaut sans avertissement » **entière** et en refaisant le décompte des gestes.
**Ordre suggéré : après la 15-13a** (#551 est P2, sécurité ; #552 et #576 P3) ; l'ordre inverse
est possible sans autre effet que le recompte. Release visée : **v0.13.0**, la même que la 15-11a — la
section du manuel « Passer à la 0.13.0 » et l'entrée `[0.13.0]` du CHANGELOG sont **complétées**, pas
doublées.

**Si la 15-13b est mergée seule** (la 15-13a non encore mergée au tag) : le CHANGELOG `[0.13.0]` porte les
entrées **Corrigé** #552 et #576 et l'entrée **Sécurité** « dossiers montés ignorés » (AC 13 b) ; à
`CHANGELOG.md:46`, la fin de phrase « — la sauvegarde pré-import reste dans `/tmp` » et le décompte des
gestes sont réécrits (AC 13 c) ; les autres phrases de `:46` restent vraies (elles ne deviennent fausses
qu'avec la 15-13a). Le manuel porte les parties sauvegarde ; le paragraphe « Pour qui garde son fichier
compose » compte **trois** gestes pour chaque compose (les deux de la 15-11a, plus le montage `./backup`).

## Story

En tant qu'**exploitant d'une installation Kesh** (NAS Synology ou serveur Docker),
je veux que **la sauvegarde de sécurité prise avant un import d'installation soit écrite sur un dossier qui
survit au conteneur, et que l'échec de cette sauvegarde soit annoncé comme tel**,
afin de **pouvoir revenir en arrière après un import raté — y compris après le redémarrage qui suit souvent
l'échec —, et de ne jamais croire à une sauvegarde qui n'existe pas**.

## Le défaut, établi au sol le 2026-10-09

Faits revérifiés au code (`grep -nF`, worktree `kesh-15-13`, `HEAD` = `de285ea8`) :

### #552 — la sauvegarde pré-import vit dans `/tmp` du conteneur

| Site | Contenu |
|---|---|
| `crates/kesh-api/src/config.rs:946-947` | `let admin_backup_dir = env_nonempty("KESH_ADMIN_BACKUP_DIR").unwrap_or_else(\|\| "/tmp".to_string());` |
| `crates/kesh-api/src/config.rs:250-256` | doc-comment du champ : « défaut `/tmp`. Créé s'il n'existe pas. La purge est à la charge de l'opérateur. » |
| `crates/kesh-api/src/config.rs:939-945` | commentaire de lecture (« Défaut `/tmp` ») |
| `crates/kesh-api/src/routes/admin.rs:259-261` | `build_keshbackup` puis `write_pre_import_backup(&state.config.admin_backup_dir, …)` **sous le verrou**, avant le restore |
| `crates/kesh-api/src/routes/admin.rs:471-495` | `create_dir_all(dir)` puis `tokio::fs::write(path, bytes)` — **mode par défaut** (umask du processus : `0644` sous l'umask `022` de l'image) |
| `docker-compose.yml:121`, `docker-compose.prod.yml:151` | `KESH_ADMIN_BACKUP_DIR: ${KESH_ADMIN_BACKUP_DIR:-}` — transmise (15-11a), **aucun montage** ne lui correspond |
| `.env.example:179-182` | « Défaut /tmp (absente ou vide) — PERDU au redémarrage du conteneur ; monter un volume persistant. » puis `#KESH_ADMIN_BACKUP_DIR=/data/backup` — **aucun compose ne monte `/data/backup`** |
| `admin-manual.tex:757`, `:1664`, `:1699` | « défaut `/tmp` » ×3 |
| `CHANGELOG.md:46` | « la sauvegarde pré-import reste dans `/tmp` » |

Le conteneur `kesh-api` tourne en **root** (le `Dockerfile` n'a pas d'instruction `USER`), son système de
fichiers est éphémère : la sauvegarde disparaît à toute **recréation** du conteneur — `docker compose up -d`
après une modification de `.env` ou du compose, mise à jour de l'image —, c'est-à-dire précisément au
moment où l'on en a besoin après un import raté. Le fichier est un **secret** (condensés des mots de passe,
jetons de session : `admin-manual.tex:1654`) : le rendre persistant sur l'hôte impose de ne pas le
laisser lisible par tous (AC 9, C-15-13-4).

### #576 — un échec d'écriture de la sauvegarde est annoncé comme une sauvegarde réussie

| Site | Contenu |
|---|---|
| `crates/kesh-i18n/locales/*/messages.ftl` (`fr-CH:1478`, `de-CH:1407`, `en-CH:1407`, `it-CH:1407`) | `error-admin-full-import-failed` : « L'état précédent a été préservé (**un backup automatique a été créé** avant l'opération) » |
| `crates/kesh-api/src/errors.rs:1819-1828` | `AppError::AdminFullImportFailed` → 500, `ADMIN_FULL_IMPORT_FAILED`, cette clé — repli Rust « Échec de l'import de l'installation. L'état précédent a été préservé. » |
| `crates/kesh-api/src/routes/admin.rs:473`, `:487` | l'échec de création du dossier ou d'écriture du fichier rend **cette même** variante |
| `crates/kesh-api/src/routes/admin.rs:236`, `:240`, `:246` ; `admin_backup/import.rs:204` (via `check_schema_compat`, appelé `admin.rs:172`) | échecs **antérieurs** à la sauvegarde, même variante, même message |
| `crates/kesh-api/src/admin_backup/export.rs:103-105` (`map_db` → `AdminFullExportFailed`), appelé par `build_keshbackup` (`admin.rs:259`) | échec de lecture de la base pendant la sauvegarde : message « l'**export** n'a pas pu être généré », alors que l'utilisateur importait |

Le frontend affiche le `message` du corps pour un 500 (`frontend/src/lib/features/admin-restore/AdminRestorePanel.svelte:81-84`) : aucun code client
ne dépend de `ADMIN_FULL_IMPORT_FAILED` (`grep -rn ADMIN_FULL_IMPORT frontend/src` → vide). Les échecs
**postérieurs** à l'écriture (`admin.rs:283` à `:462`, douze sites) disent vrai : la sauvegarde existe et la
transaction est annulée.

## Acceptance Criteria

*(Numérotation de la fiche unique 15-13 conservée ; les AC 1 à 6 sont dans la 15-13a, et les sous-points
des AC 10 à 14 qui portent sur MariaDB aussi.)*

### Volet B — la sauvegarde pré-import (#552)

7. **Le défaut du code devient `/data/backup`.** (C-15-13-3.)
   (a) `config.rs:946-947` : `KESH_ADMIN_BACKUP_DIR` absente, vide ou blanche → `/data/backup` (au lieu de
   `/tmp`), par une constante nommée **publique** (`pub const DEFAULT_ADMIN_BACKUP_DIR`, `kesh_api::config`)
   — pas de littéral dupliqué dans le code. **La valeur est liée par deux tests** (R4-1 de la validation P4,
   C-15-13-24) : le test 6 b la compare au **littéral** `"/data/backup"` (le seul littéral côté test,
   voulu : c'est la valeur dont tout #552 dépend), et le test 3 exige qu'elle soit la **cible** d'une
   entrée de `MONTAGES` — donc d'un montage des deux compose (contrôle `transmission`). Une constante
   changée en `/data/backups` ou en `/tmp` rougit les deux ; comparée à elle-même seulement, elle
   dériverait au vert, et la sauvegarde retournerait sans signal dans le conteneur.
   (b) Doc-comment du champ (`config.rs:250-256`) et commentaire de lecture (`:939-945`) mis à jour :
   défaut `/data/backup`, monté par les compose distribués sur `./backup` ; **hors Docker**, poser la
   variable sur un dossier inscriptible (sinon l'import est refusé avant toute suppression, la création
   du dossier échouant, avec le message de l'AC 15). Les recettes du dépôt qui lancent un backend hors
   Docker posent la variable (AC 16). Le doc-comment du test 6 a (`config.rs:1643-1646`, « `/tmp` (et non
   un chemin vide, qui ferait écrire le backup pré-import dans le répertoire courant) ») devient faux avec
   le défaut neuf : il nomme `/data/backup` (R5-7 de la validation P5).
   (c) Les constructeurs de test (`config.rs:508`, `:1508`) **gardent** `/tmp` : ce sont des
   configurations de test, pas le défaut.

8. **Les deux compose distribués montent `./backup` sur `/data/backup`.**
   (a) `docker-compose.yml` et `docker-compose.prod.yml` : une entrée de `volumes:` du service `kesh-api`,
   **fixe** dans les deux (`- ./backup:/data/backup`), avec un commentaire : sauvegarde de sécurité prise
   avant chaque import d'installation ; **secret** ; dans le périmètre de sauvegarde du dossier du compose
   (Hyper Backup) ; à purger par l'exploitant. Aucune variable `KESH_*_HOST_DIR` neuve (C-15-13-3, #558).
   (b) `KESH_ADMIN_BACKUP_DIR: ${KESH_ADMIN_BACKUP_DIR:-}` **inchangé** dans les deux compose — le défaut
   reste celui du code (C75), les couples de la liste `AJOUTS` du test restent.
   (c) `.env.example:179-182` : commentaire réécrit (défaut `/data/backup`, monté sur `./backup` par les
   compose fournis ; une autre valeur doit être un chemin **monté**, sans quoi la sauvegarde retourne dans
   le système de fichiers éphémère du conteneur ; hors Docker, un dossier inscriptible) ; la ligne
   d'exemple reste `#KESH_ADMIN_BACKUP_DIR=/data/backup`. Les deux phrases qui **énumèrent** les montages
   fixes de `docker-compose.prod.yml` — `.env.example:208-210` (bloc des chemins d'hôte) et `:241-242`
   (`KESH_LOG_HOST_DIR`), « ses montages sont fixes — `./inbox`, `./documents`, `./log` » — deviennent
   incomplètes avec le quatrième : elles gagnent « et `./backup` (sauvegarde pré-import, sans variable
   d'hôte) » (F-P4-3 de la validation P4). Le paragraphe du mode `cargo run` natif du même
   gabarit (`.env.example:77-82`, « (d) Dev `cargo run` natif (hors Docker) sur Linux non-root », qui
   pose `KESH_PORT`) gagne la même mention : hors Docker, `KESH_ADMIN_BACKUP_DIR` sur un dossier
   inscriptible (F-P3-5).
   (d) `docker-compose.dev.yml` n'est **pas** modifié (pile de développement non distribuée ; le conteneur
   y crée `/data/backup` dans son système de fichiers — comportement de développement assumé).
   (e) **Les dossiers d'hôte montés par défaut restent hors du dépôt et du contexte de build.**
   `docker compose up` se lance depuis un clone (`DOCKER_START.md:27`, `:37`) : `./backup` y est créé (vide)
   par Docker au premier `docker compose up` — la forme courte d'un montage fait créer la source absente
   par le démon —, rempli au premier import (F-P5-5 de la validation P5), et un `git add -A` y versionnerait une sauvegarde complète — un **secret**. Le symptôme
   vaut pour chaque source de montage par défaut, pas pour `./backup` seul : `./inbox` et `./documents`
   (justificatifs) ne sont pas ignorés non plus (`git check-ignore` muet, seul `log/` l'est, `.gitignore:34`).
   Donc, **pour chaque entrée de `MONTAGES`** (sources `./log`, `./inbox`, `./documents`, `./backup`) :
   `.gitignore` porte la ligne **ancrée** `/<dossier>/` et `.dockerignore` la ligne `<dossier>/` — soit,
   ajoutées, `/inbox/`, `/documents/`, `/backup/` au premier, `log/`, `inbox/`, `documents/`, `backup/` au
   second (C-15-13-14, rectifié par C-15-13-20 ; C-15-13-19). **Le motif non ancré est refusé** (R3-1) : `backup/` dans `.gitignore` ignore aussi
   `frontend/src/routes/(app)/admin/backup/`, **versionné** (`+page.svelte`, `+page.ts` — l'écran même de
   la sauvegarde) — tout fichier neuf de cette route serait ignoré **en silence** (mesuré en validation P3 :
   `git -c core.excludesFile=<fichier portant backup/> check-ignore -v --no-index "frontend/src/routes/(app)/admin/backup/nouveau.ts"`
   → ignoré ; avec `/backup/` → non ignoré). **Une seule tolérance, nommée** : la ligne `log/` existante
   (`.gitignore:34`) est gardée telle quelle — `git ls-files | grep -E '(^|/)log/'` → 0 (aucun dossier
   `log` versionné nulle part dans le dépôt), si bien qu'elle ne masque rien aujourd'hui, et l'ancrer
   ferait réapparaître dans `git status` les `log/` que des outils de dev créeraient sous les crates ; un
   `log/` versionné ajouté demain serait masqué — risque écrit, accepté. `.dockerignore` n'est pas
   concerné par l'ancrage : ses motifs sont relatifs à la racine du contexte (`backup/` ≡ `/backup`).
   `git ls-files` ne rend aucun fichier sous ces quatre dossiers **à la racine** (rejoué en validation P4
   au `HEAD` `c702b7d5` : `git ls-files | grep -E '(^|/)(log|inbox|documents|backup)/'` ne rend que
   `frontend/src/routes/(app)/admin/backup/+page.svelte` et `+page.ts`, hors racine) : aucune ligne ancrée
   ne masque de contenu versionné. Garde : test 19.

9. **La sauvegarde est créée en mode `0600`, et jamais par-dessus un fichier existant.** (C-15-13-4,
   C-15-13-8.)
   (a) `write_pre_import_backup` (`routes/admin.rs:471`) garde le choix du nom (inchangé) et délègue
   l'écriture à une fonction factorisée **sur un chemin donné**, `write_backup_file(path: &Path, bytes:
   &[u8]) -> Result<(), AppError>` (privée au module ; appelée directement par le test 12, à travers `write_pre_import_backup` par le
   test 11 — F-P6-3 de la validation P6). **Écriture par
   fichier temporaire, puis renommage** (F-P4-6 de la validation P4, décision de l'orchestrateur,
   C-15-13-21) : elle crée `<path>.partial` — **même dossier**, donc même système de fichiers — par
   `tokio::fs::OpenOptions` avec `create_new(true)` et, sous `#[cfg(unix)]`, `mode(0o600)` ; écrit les
   octets, `sync_all` ; puis **renomme** `<path>.partial` en `<path>` (`tokio::fs::rename`, atomique sur un
   même système de fichiers). Un arrêt pendant l'écriture — client déconnecté (hyper abandonne le futur du
   handler à un `.await`), OOM, `docker stop` — ne laisse donc qu'un `.partial`, **jamais un fichier nommé
   comme une sauvegarde**. **Refus d'écraser** : `rename` remplace une cible existante sur Unix ; la
   fonction vérifie donc, avant le renommage, que `<path>` n'existe pas (`tokio::fs::try_exists`) et, sinon,
   échoue (`.partial` supprimé au mieux) ; un `try_exists` qui rend `Err` (droits sur le dossier) vaut
   **échec**, jamais « absent » — le traiter comme absent rouvrirait l'écrasement (R5-3/F-P5-8 de la
   validation P5). La fenêtre entre la vérification et le renommage n'est pas
   fermée — le nom est unique par construction (horodatage, pid, compteur) et un second écrivain du même nom
   serait déjà un défaut : angle mort écrit. Le journal `info` « backup pré-import écrit » suit le renommage
   réussi. **On ne supprime que ce qu'on a créé** (F-P5-3) : un échec de `create_new` — `.partial` déjà présent,
   écriture interrompue ou autre écrivain — ne supprime **rien**, et ce `.partial` préexistant garde son
   contenu. Si `write_all`, `sync_all`, la vérification du nom final ou `rename` échoue, le `.partial` **créé
   par l'appel** est supprimé au mieux
   (`remove_file`, échec journalisé en `warn!` sans masquer l'erreur d'origine) ; si `remove_file` échoue à
   son tour, le `warn!` **nomme le chemin** du `.partial` et dit qu'il n'est pas une sauvegarde valide.
   Dans tous ces cas, le message de l'AC 15 c (« aucune sauvegarde n'a été créée ») reste **vrai** : aucun
   fichier ne porte le nom d'une sauvegarde. Le doc-comment de `write_pre_import_backup`
   (`routes/admin.rs:467-470`) dit le mode `0600`, l'écriture par `.partial` et renommage, le refus
   d'écraser et la variante d'erreur.
   (b) Le dossier, quand Kesh le crée, l'est en `0o700` (`tokio::fs::DirBuilder`, `recursive(true)`,
   `mode(0o700)` sous `#[cfg(unix)]`) ; un dossier **existant** (le montage de l'hôte) n'est pas modifié.
   Le mode s'applique à **tous les niveaux créés** par l'appel récursif, parents compris (R6-3 = F-P6-6 de
   la validation P6) : un `KESH_ADMIN_BACKUP_DIR` sous un parent absent crée ce parent en `0700` aussi —
   sans conséquence dans le conteneur (root), écrit au doc-comment et aux angles morts.
   (c) Une erreur rend `AppError::AdminPreImportBackupFailed` (AC 15) avec un détail **par étape**, qui
   nomme le chemin (R5-3/F-P5-8 de la validation P5 : le journal est le seul diagnostic, le corps n'expose
   rien) — « création répertoire backup '…' : … » (inchangé), « création fichier partiel '…' : … »
   (`create_new`), « écriture backup '…' : … » (`write_all`), « synchronisation backup '…' : … »
   (`sync_all`), « vérification nom final '…' : … » (`try_exists` en `Err`), « nom final déjà présent
   '…' » (refus d'écraser), « renommage backup '…' → '…' : … » (`rename`) : aucun import sans sauvegarde
   réussie (DC5 inchangé). Les libellés sont du journal, non testés à la lettre.
   (d) Le commentaire `routes/admin.rs:475-477` (« le backup pré-import est pris avant le verrou FOR
   UPDATE ») est **faux** — il est pris sous le verrou (`:226-261`) — et corrigé au passage.

### Volet C — le garde-fou et la documentation (parties sauvegarde)

10. **`configuration_transmise.rs` garde ces décisions.** *(Les sous-points (a), (b) et la partie MariaDB
    de (d) et (e) sont dans la 15-13a.)*
    (c) **Montage de la sauvegarde** : `MONTAGES` gagne l'entrée `("/data/backup", "./backup", "./backup")` ;
    le contrôle de `docker-compose.yml` exige l'**égalité** quand la source attendue ne commence pas par
    `${` (aujourd'hui `starts_with` : `./backups` passerait) — les trois entrées existantes gardent le
    préfixe `${KESH_…_HOST_DIR:-`. Le message de refus pour `docker-compose.prod.yml` reste celui de #558.
    Le message de refus pour `docker-compose.yml` (`configuration_transmise.rs:512-514`, « doit avoir pour
    source `{prefixe_y}…}` ») est faux pour une source **exacte** — il imprimerait « `./backup…}` » : pour
    une entrée dont la source attendue ne commence pas par `${`, le message devient « doit être exactement
    `./backup` (montage fixe), trouvé `…` » (R5-6 de la validation P5) ; celui des trois entrées variables
    est inchangé.
    **La comparaison est extraite en fonction pure** (F-P4-2 de la validation P4, C-15-13-24) :
    `source_conforme(compose: Compose, source: &str, attendue_y: &str, exacte_p: &str) -> bool` — pour `Y`,
    `starts_with(attendue_y)` si `attendue_y` commence par `${`, égalité sinon ; pour `P`, égalité —, que la
    boucle de `controle_transmission` (`configuration_transmise.rs:509-521`) appelle. Sans elle, la règle
    d'égalité vit dans une boucle que l'auto-test (S) n'atteint pas, et rien ne la garde : les vrais
    compose portent `./backup`, que `starts_with("./backup")` accepte aussi.
    **Lien avec le défaut du code** (R4-1, C-15-13-24) : le test `transmission` exige en outre qu'une
    entrée de `MONTAGES` ait pour cible `kesh_api::config::DEFAULT_ADMIN_BACKUP_DIR` (AC 7 a).
    (d) **Auto-test (S)** : `s_sources_de_montage` exerce `source_conforme` — pour `Y` avec
    `attendue_y = "./backup"` : `./backup` **accepté**, `./backups` (préfixe) **refusé**,
    `${X:-./backup}` refusé ; pour `Y` avec `attendue_y = "${KESH_INBOX_HOST_DIR:-"` :
    `${KESH_INBOX_HOST_DIR:-./inbox}` accepté ; pour `P` : `./backup` accepté, `./backups` refusé. Le cas
    `./backups` en `Y` est celui que **M14** (retour à `starts_with` pour toutes les entrées) fait passer :
    il rougit sous M14, ce que rien ne faisait avant la validation P4.
    (e) Les textes qui comptent « trois » montages deviennent faux avec le quatrième : doc de `MONTAGES`
    (« Les trois montages », `:196`), doc de `sources_montages` (« des trois montages », `:378` ; « l'une
    des trois », `:379`) — `grep -n "trois" configuration_transmise.rs` rend exactement ces trois lignes au
    `HEAD` `de285ea8`.

11. **Le manuel d'administration dit vrai, et le PDF aussi** (parties sauvegarde). `docs/manual/fr/admin-manual.tex` :
    (d) **`sec:env-vars`** : ligne `KESH_ADMIN_BACKUP_DIR` (`:757`) — défaut `/data/backup` (monté sur
    `./backup`) ; **hors Docker : poser la variable sur un dossier inscriptible** (F-P3-5, comme les lignes
    voisines `KESH_STATIC_DIR` et `KESH_LOCALES_DIR`, `:703-704`). Le paragraphe des montages d'hôte
    (`:807-822`) dit que `./backup` est monté par les deux compose, **sans** variable d'hôte — dont la
    phrase `:812` (« montages sont fixes — `./inbox`, `./documents`, `./log` »), qui énumère les montages
    de `docker-compose.prod.yml` et gagne `./backup` (F-P4-3) ; le tableau `:819-821` (variables d'hôte)
    n'a pas de ligne à gagner, `./backup` n'ayant pas de variable. **Les dix
    tableaux de `sec:env-vars` restent sans `Overfull \hbox`** (acquis de la 15-11a). Le paragraphe du mode
    `cargo run` natif (`:194`, « Mode `cargo run` natif (hors Docker) sur Linux non-root », qui pose
    `KESH_PORT`) gagne une incise : hors Docker, `KESH_ADMIN_BACKUP_DIR` sur un dossier inscriptible, sans
    quoi tout import est refusé (AC 15). **Aucune de ces incises (`:757`, `:194`) ne donne d'exemple sous
    `/tmp`** (R5-4 de la validation P5) : à moins de 90 caractères de `BACKUP_DIR`, il ferait rougir le
    contrôle négatif `BACKUP_DIR.{0,90}/tmp` de l'AC 11 l, qui vise l'ancien défaut et non un exemple — et
    l'exemple naturel à recopier, `/tmp/kesh-e2e/backup` des recettes de `docs/testing.md` (AC 16 b), est
    précisément celui-là. Un exemple, s'il en faut un, se prend hors de `/tmp` (`$HOME/kesh-backup`).
    (e) **`sec:backup-ui-keshbackup`** (`:1664`, `:1667-1672`, `:1699`) : la sauvegarde de sécurité est
    dans le dossier `backup/` du répertoire du compose (`/volume1/docker/kesh/backup` sur Synology),
    nommée `kesh-pre-import-<horodatage>-….keshbackup`, **créée en mode `0600`, propriétaire `root`** (le
    conteneur tourne en root) — sur Synology, les ACL du dossier partagé peuvent s'y ajouter : limite non
    mesurée, écrite comme telle ; c'est un **secret** ; purge à la charge de l'exploitant. **Restaurer
    depuis elle** (C-15-13-8) : l'import se fait par le navigateur, et ni File Station, ni un partage SMB,
    ni un `scp` sans `sudo` ne lisent un fichier `root` en `0600` ; le manuel donne donc le geste de
    rapatriement, en SSH :
    `sudo cp backup/kesh-pre-import-<...>.keshbackup /volume1/<partage>/`, puis
    `sudo chown <utilisateur> /volume1/<partage>/kesh-pre-import-<...>.keshbackup`, l'import par l'écran,
    puis **la suppression de la copie** (secret) — geste **rejoué au T0** sur un conteneur jetable. La
    liste « L'import refuse le fichier et préserve l'installation si » (titre `:1667`, liste `:1668-1672` —
    **pas** `:1676-1690`, qui est la liste « Trois propriétés » des reprises de données) gagne l'entrée de
    l'AC 15 : la sauvegarde de sécurité ne peut pas être prise ni écrite — **deux pistes**, comme le
    message (AC 15 c, F-P5-7 de la validation P5) : le dossier de sauvegarde (non inscriptible, disque
    plein) ou la base de données (momentanément inaccessible) — le message le dit, aucune sauvegarde n'est
    créée, rien n'est supprimé. Le défaut `/tmp` disparaît
    de `:1664` et de `:1699`. **Fichier `.partial`** (AC 9 a, C-15-13-21) : le manuel dit qu'un
    `kesh-pre-import-….keshbackup.partial` est une écriture interrompue, **pas** une sauvegarde, et qu'il
    peut être supprimé. **Forme du rapatriement** (F-P4-8) : le bloc `sudo cp` / `sudo chown` est
    introduit par « depuis le répertoire du compose » (le chemin `backup/…` est relatif) et posé dans un
    `lstlisting` ordinaire (76 caractères), **pas** dans un `keshwarning` (70) : la ligne
    `sudo chown <utilisateur> /volume1/<partage>/kesh-pre-import-<...>.keshbackup` en mesure **76**
    (`printf '%s' … | wc -m`, remesuré en remédiation P5 avec `<...>`), soit la limite exacte ; si la mise
    en page l'exige, elle est coupée par `\`. **Le bloc est en ASCII** (F-P5-6 de la validation P5) : `<...>`
    et non `<…>` (U+2026) — **par prudence** : le contrôle aplati ne dépend ainsi d'aucun rendu de glyphe
    par `listings` (R6-1 de la validation P6 : dix `lstlisting` du manuel portent déjà du non-ASCII ; la
    règle vaut pour ce bloc, non pour le manuel) ; la prose, elle, garde `…`. Contrôle aplati positif ajouté à l'AC 11 l
    pour la ligne entière — **si la ligne est coupée** à la mise en page (`breaklines`, ou `\`), le contrôle
    suit la forme coupée relevée au T5, écrite au Dev Agent Record (F-P6-5 de la validation P6).
    **Vocabulaire** (F-P6-8) : aux sites que la story réécrit (`:757`, `:1699` à la spécification), le
    manuel dit « dossier de sauvegarde », le mot même du message de l'AC 15. **Ouverture de la section** (`:1643`, « sans accès
    SSH ni ligne de commande ») : vraie pour l'export et l'import par l'écran, mais la section reçoit
    désormais un geste en SSH ; une phrase la précise — la sauvegarde de sécurité **pré-import** fait
    exception, son rapatriement demande un accès SSH (voir plus bas) — (F-P4-7). Le tableau `:1712`
    (« restauration self-service sans SSH ») et `website/roadmap.html:231` (« SSH-free migration and
    restore ») parlent de l'export/import par l'écran et restent vrais : assumés, à l'inventaire.
    (f) **`sec:maj-0-13`** (`:1757-1841`) — le paragraphe **sauvegarde** ; le paragraphe MariaDB est dans
    la 15-13a. Re-télécharger le compose suffit pour le montage. **Pour qui garde son compose — le
    paragraphe `:1793`** (F-P3-1, C-15-13-18) : le geste « ajouter le montage `./backup` » (sous `volumes:` de
    `kesh-api`, **pas** sous `environment:` : la phrase d'ouverture « Sous `environment:` du service
    `kesh-api` : » devient fausse et est réécrite) y est écrit, pour les deux compose, avec sa ligne de
    contrôle sur la **cible du montage**, pas sur le chemin : `docker compose config | grep -c 'target: /data/backup'`
    → 1 (graphie de la forme longue relevée au T0 ; si elle diffère, le motif suit la mesure, écrite au Dev
    Agent Record). Le motif `/data/backup` seul ne discrimine pas : `docker compose config` imprime aussi
    l'environnement, et un `.env` qui a décommenté `#KESH_ADMIN_BACKUP_DIR=/data/backup` (`.env.example:182`
    — le public même de ce geste) fait sortir `KESH_ADMIN_BACKUP_DIR: /data/backup` **sans** montage
    (R4-2/F-P4-1 de la validation P4). Le T0 vérifie le contrôle **rouge** (0) sur un compose sans le
    montage avec cette variable posée. Le titre « Pour qui garde son fichier compose : **deux
    gestes**. » perd son décompte global ; le paragraphe donne le décompte **par fichier**, recompté à
    l'écriture sur l'état de la branche : avec la 15-13b seule, **trois** gestes pour chacun des deux
    compose (les deux de la 15-11a, plus le montage) ; si la 15-13a est déjà mergée, **cinq** pour
    `docker-compose.yml` et **trois** pour `docker-compose.prod.yml`. **Effets sans refus** (`:1773`) —
    `KESH_ADMIN_BACKUP_DIR (un chemin non monté fait écrire dans le conteneur)` reste vrai ; y ajouter,
    **conditionné au compose** (F-P3-1) : **avec le compose de la 0.13.0**, sans la ligne, la sauvegarde
    va désormais dans `./backup` ; **avec un compose gardé** sans le montage, elle va dans `/data/backup`
    **du conteneur** — éphémère, **comme avant** (le manuel n'écrit pas `/tmp` à cet endroit : la cellule
    `:1773` commence par `KESH_ADMIN_BACKUP_DIR`, et un `/tmp` à moins de 90 caractères rougirait le contrôle
    négatif `BACKUP_DIR.{0,90}/tmp` de l'AC 11 l — R4-7 ; l'ancien `/tmp` n'est nommé qu'au CHANGELOG,
    AC 13 c). Le manuel fait partie du corpus du contrôle (F) de
    `configuration_transmise.rs` (`:899-920`) : ne nommer **aucun** jeton `KESH_…` inexistant, pas même pour
    dire qu'il n'existe pas (« aucune variable `KESH_BACKUP_…` » rougirait en fantôme).
    (i) **Stratégie recommandée** (`:1417-1425`) : le dossier `./backup` est dans le périmètre du dossier
    du compose ; il contient des secrets.
    (k) Toute ligne de commande neuve dans un encadré `keshwarning` tient en **70 caractères** au plus, en
    `keshnote`/`lstlisting` ordinaire en **76** (mesure de la 15-11a).
    (l) **PDF régénéré** (`make fr` dans `docs/manual/`), contrôlé **aplati**
    (`pdftotext admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`, ligatures normalisées comme à la 15-11a).
    Contrôles **discriminants** — chacun rouge sur le PDF d'avant (relevé au T0) : **positifs** —
    `/data/backup` ; `sudo cp backup/` et `kesh-pre-import` (rapatriement — `sudo chown` figure déjà deux
    fois au PDF d'avant et ne discrimine pas) ; tous vérifiés **absents** du PDF aplati à `468ce610`
    (`target/gate-logs/15-13-p1-R-admin-plat.txt`) en remédiation P1 ; `keshbackup.partial` et
    `target: /data/backup` (vérifiés absents, en remédiation P4, de
    `target/gate-logs/15-13b-p4-F-admin-manual-plat.txt` : `grep -c partial` → 0) ; la ligne `sudo chown`
    **entière**, en ASCII (`grep -cF 'sudo chown <utilisateur> /volume1/<partage>/kesh-pre-import-<...>.keshbackup'`
    → 1 : elle vérifie le rendu du bloc par `listings`, F-P5-6) et l'entrée de la liste des refus qui nomme
    la base (`grep -cE 'base de données.{0,40}inaccessible'` ≥ 1, F-P5-7) — les deux vérifiés absents, en
    remédiation P5, de `target/gate-logs/15-13b-p5-F-admin-plat.txt` (`grep -cF 'kesh-pre-import-<...>'`
    → 0 ; le motif de la base → 0) ; **négatifs** —
    `grep -E 'BACKUP_DIR.{0,90}/tmp'` vide (la cellule du tableau, `:757`, que `défaut /tmp` ne voit pas) ;
    `défaut /tmp` absent ; `deux gestes` absent (présent **une** fois au PDF d'avant, relevé en validation
    P3 — discriminant). Au `.log`, aucun `Overfull \hbox` **nouveau** (comparé au `.log` d'avant, relevé au
    T0).
    (n) **Brochure** (`docs/manual/fr/marketing-brochure.tex:398`, « Export / import complet
    d'installation (`.keshbackup`) pour migration ou restauration **sans accès SSH** ») **relue** au T5
    (F-P3-11) : la promesse reste vraie pour un `.keshbackup` exporté par le navigateur ; la sauvegarde de
    sécurité **pré-import**, elle, ne se restaure qu'après un rapatriement en SSH (C-15-13-8). Laisser la
    ligne telle quelle ou la nuancer — le choix et sa raison au Dev Agent Record ; si elle change, le PDF
    de la brochure est régénéré par le même `make fr`.

12. **`DOCKER_START.md`** (partie sauvegarde).
    (a') Une ligne sur le dossier `./backup` : créé par Docker au premier `docker compose up`, il reçoit la
    sauvegarde de sécurité prise avant chaque import ; **secret**, ignoré de git (AC 8 e), à purger.
    **Emplacement** (F-P5-4 de la validation P5) : le fichier n'a pas de section « dossiers » et ne nomme
    aucun montage (`grep -n "inbox\|documents\|log/\|backup" DOCKER_START.md` → vide) ; la ligne est une puce
    de plus à la fin de § *Notes* (`DOCKER_START.md:140`, après celle du volume `mariadb_data`, que la
    15-13a peut retoucher : conflit d'une ligne au rebase, sans autre effet). *(Le reste de l'AC 12 — prérequis, étape `.env`, accès à la base,
    volume, « Base de données ne s'initialise pas », retrait d'`init-demo.sh` — est dans la 15-13a.)*

13. **CHANGELOG `[0.13.0]`** (parties sauvegarde).
    (b) **Corrigé** — entrée #552 : la sauvegarde prise avant un import va dans `./backup`, survit au
    conteneur, créée en mode `0600` ; la restaurer demande de la rapatrier (manuel) ; hors Docker,
    `KESH_ADMIN_BACKUP_DIR` à poser. Entrée #576 : un import dont la sauvegarde n'a pas pu être écrite
    le dit, au lieu d'annoncer une sauvegarde qui n'existe pas. Rubrique **Sécurité** : les dossiers montés
    par défaut (`backup`, `inbox`, `documents`, `log`) sont ignorés de git et du contexte de build (AC 8 e).
    *(La rubrique **Retiré** `init-demo.sh` : 15-13a.)*
    (c) **Propagation** : `CHANGELOG.md:46` porte **cinq** phrases que la 15-13 rend fausses (F-P3-1).
    Ventilation (R5-5 de la validation P5 : la phrase des gestes était revendiquée par les deux fiches,
    4 + 2 = 6 ≠ 5) : **une** relève de la 15-13b seule ; **trois** de la 15-13a seule (refus de Compose,
    défaut sans avertissement des mots de passe, `.env` copié du gabarit) ; **une est partagée**, « le
    manuel d'administration décrit les **deux gestes** » — soit 1 + 3 + 1 partagée = 5. Les « quatre »
    phrases de l'AC 13 c de la 15-13a comptent la partagée ; la fiche sœur n'est pas retouchée (close, en
    développement), le décompte se lit ici.
    — La phrase **propre** à la 15-13b, réécrite : « — la sauvegarde pré-import reste dans `/tmp` » (avec
    le compose de la 0.13.0, elle va dans `./backup` ; avec un compose gardé sans le montage, dans
    `/data/backup` du conteneur, éphémère).
    — La phrase **partagée**, « deux gestes » : réécrite par la **première** des deux fiches mergée,
    son décompte **refait** par la seconde au rebase (§ *Dépendances*) — le décompte par fichier de
    l'AC 11 f (trois et trois avec la 15-13b seule, cinq et trois avec les deux) ou un renvoi sans nombre,
    qui n'a alors rien à recompter.
    **Et `CHANGELOG.md:44`** (entrée #550 de `[0.13.0]`, F-P4-3 de la validation P4) : « **Inchangé** : avec
    `docker-compose.prod.yml`, les montages restent `./documents`, `./inbox`, `./log` » devient faux dans la
    même version — la phrase gagne « ; la 0.13.0 y ajoute `./backup` (sauvegarde pré-import, #552) ».

14. **Inventaire des sites non résolus tenu** (§ *Inventaire* des Dev Notes) : à la fin du développement,
    chaque site de l'inventaire est soit résolu par un AC, soit écrit comme angle mort, et les greps de
    contrôle (T9) ne trouvent **aucun** site hors de l'inventaire.

### Volet D — le message d'un import refusé avant la sauvegarde (#576) et les recettes hors Docker

15. **Un import qui échoue avant que la sauvegarde soit sur disque le dit, sans promettre de copie.**
    (C-15-13-9.)
    (a) Variante neuve `AppError::AdminPreImportBackupFailed(String)` (`errors.rs`) : HTTP 500, code
    `ADMIN_PRE_IMPORT_BACKUP_FAILED`, clé `error-admin-pre-import-backup-failed`, détail journalisé et
    jamais exposé (comme `AdminFullImportFailed`). Elle couvre **tout** échec antérieur à l'écriture
    réussie du fichier : `routes/admin.rs:236`, `:240`, `:246` (transaction, verrou, ligne `_kesh_version`
    absente), `:473`, `:487` (dossier, fichier — et `write_backup_file` de l'AC 9) ; avec les deux appels
    de l'AC 15 b (`:172`, `:259`), **sept** sites antérieurs à l'écriture (R5-2 de la validation P5). Le doc-comment de `AdminFullImportFailed`
    (`errors.rs:448-452`, « backup pré-import impossible ») est corrigé : elle ne couvre plus que les
    échecs **postérieurs** à la sauvegarde (`admin.rs:283` à `:462`, douze sites, inchangés).
    (b) Les deux sites qui naissent **hors** du module — `check_schema_compat` (`admin_backup/import.rs:204`,
    appelé `admin.rs:172`) et `build_keshbackup` (`map_db` → `AdminFullExportFailed`,
    `admin_backup/export.rs:103-105`, appelé `admin.rs:259`) — sont convertis **à l'appel**, dans
    `routes/admin.rs`, par une fonction pure `avant_sauvegarde(AppError) -> AppError` qui change
    `AdminFullImportFailed` et `AdminFullExportFailed` en `AdminPreImportBackupFailed` et rend toute autre
    variante telle quelle (`ImportSchemaMismatch` reste un 400). `admin_backup` n'est pas modifié.
    **Forme prescrite de l'appel** : `….await.map_err(avant_sauvegarde)?` (rustfmt la répartit sur
    plusieurs lignes : sans effet, le test 20 lit l'instruction jusqu'au `;` — R6-2 de la validation P6), dans l'instruction même de
    l'appel — pas de fermeture `|e| avant_sauvegarde(e)`, pas de conversion différée. **Le branchement est
    gardé** (F-P5-1 de la validation P5, C-15-13-28) : les deux sources ne sont pas injectables à bon
    compte (une table supprimée fait d'abord rougir `check_schema_compat` en 400), si bien que le test 14
    n'établit que la fonction pure ; le test 20, **lexical**, vérifie qu'elle est appliquée aux deux appels
    de l'import, et **pas** à celui de l'export (`full_export`, `:39`, où un échec reste un échec
    d'export).
    (c) Texte, **quatre locales** (`fr-CH`, `de-CH`, `en-CH`, `it-CH`) et **repli Rust** de `errors.rs`
    (rendu quand aucun catalogue n'est chargé) : l'import a échoué **avant** la sauvegarde de sécurité ou
    pendant son écriture ; **aucune sauvegarde n'a été créée** ; **rien n'a été supprimé** ; vérifier les
    journaux du serveur, puis réessayer — **sans orienter vers une seule cause** (F-P4-5 de la validation P4,
    C-15-13-25) : sur les **sept** sites antérieurs à l'écriture (AC 15 a-b, R5-2 de la validation P5),
    **cinq** relèvent de la **base** (`:172` lecture du schéma, `:236` transaction, `:240` verrou, `:246`
    ligne `_kesh_version` absente, `:259` export de la base — lecture des données ou assemblage du ZIP
    de `build_keshbackup`, aucune erreur `serde_json` n'y naît, R6-4 de la validation P6), **deux** du **dossier** (`:473`, `:487` devenu `write_backup_file`, avec ses étapes
    `.partial`). **Le texte nomme toujours les deux pistes, à égalité** (R5-1 de la validation P5,
    C-15-13-27, qui rectifie C-15-13-25 : l'option « n'en nommer aucune » est **retirée**) — le dossier de
    sauvegarde (inscriptible ?) **et** la base de données (joignable ?) —, dans les quatre locales **et** le
    repli Rust. C'est ce qui fonde l'écartement du contrôle du dossier au démarrage (angle mort,
    C-15-13-9) : le message nomme les deux causes possibles au moment où elles comptent ; un texte qui n'en
    nommerait aucune laisserait le cas nominal de #576 (instance hors Docker sans la variable) sans
    piste, et l'écartement sans fondement. **Jetons des pistes, à la lettre** (vérifiés par les tests 15
    et 16) : `fr-CH` « dossier de sauvegarde » et « base de données » ; `de-CH` « Sicherungsordner » et
    « Datenbank » ; `en-CH` « backup folder » et « database » ; `it-CH` « cartella di backup » et
    « database » ; repli Rust : ceux de `fr-CH`. La
    **négation** est la
    forme attendue (« aucune sauvegarde n'a été créée », « Es wurde keine Sicherung erstellt », « No backup
    was created », « Non è stato creato alcun backup ») : le test 16 n'interdit pas les mots « créé » /
    « erstellt » / « created » / « creato », il interdit la **promesse** de l'ancien texte — la phrase
    intérieure de sa parenthèse, **sans** les parenthèses, par locale (liste au test 16) : un texte neuf qui
    reprendrait la promesse sans ses parenthèses rougit aussi (R3-9). Le texte de
    `error-admin-full-import-failed` (après la sauvegarde) **ne change pas** : il dit vrai dans tous les
    cas qu'il couvre désormais. Le repli Rust d'`AdminFullImportFailed` gagne « une sauvegarde de
    sécurité a été écrite » pour que les deux replis se distinguent. **La négation est exigée, pas
    seulement l'absence de la promesse** (F-P4-4) : le test 16 vérifie que le texte de chaque locale
    **contient** sa négation (les quatre formulations ci-dessus, à la lettre) — un texte neuf « Échec de
    l'import. » ne dirait plus qu'aucune copie n'existe, ce qui est l'objet de #576 ; le test 15 vérifie de
    même que le repli Rust contient « aucune sauvegarde n'a été créée ». **Les deux pistes sont exigées de
    même** (R5-1) : le test 16 vérifie que chaque texte **contient** ses deux jetons, le test 15 que le
    repli Rust contient « dossier de sauvegarde » et « base de données » (mutations **M49**, **M50**).
    (d) Frontend **non touché** : un 500 affiche le `message` du corps (`frontend/src/lib/features/admin-restore/AdminRestorePanel.svelte:81-84`),
    aucun code client ne lit `ADMIN_FULL_IMPORT_FAILED`. Le test de parité des catalogues
    (`kesh-i18n/src/loader.rs:729-800`) reste vert sans entrée neuve dans `dette-parite-connue.txt`.
    (e) Manuel : la liste des refus de `sec:backup-ui-keshbackup` (AC 11 e).

16. **Les recettes du dépôt qui lancent un backend hors Docker posent `KESH_ADMIN_BACKUP_DIR`.**
    Avec le défaut `/data/backup`, une instance lancée par `cargo run` sous un utilisateur ordinaire
    refuse tout import (AC 15). Ce site est **invisible** du grep du T9 : c'est l'**absence** de la
    variable qui le crée (§ *Inventorier les sites NON RÉSOLUS* du CLAUDE.md). **Portée exacte** : aucune
    spec Playwright n'importe d'installation (`grep -rliE "keshbackup|full-import|admin/restore"
    frontend/tests` → vide, rejoué en remédiation P2) ; la suite E2E ne
    rencontre donc jamais le défaut. Ces recettes servent à qui lance, sur le backend qu'elles démarrent,
    un import **à la main** par le navigateur (recette manuelle de Guy, développement) — c'est pour lui que
    la variable est posée, pas pour la suite. Recettes recensées par **deux** greps (F-P3-5 : le premier,
    par la formulation, ne voyait pas les modes `cargo run` documentés pour l'exploitant) :
    `grep -rn "cargo run -p kesh-api" --exclude-dir={target,node_modules,.git,_bmad-output,_bmad,.claude,.svelte-kit} .`
    puis `grep -rn "cargo run" …` (mêmes exclusions), et tri :
    (a) **`CLAUDE.md`** § *E2E (Playwright)*, montage local (`:174-182`) : **deux** modifications, et
    aucune autre (fichier d'instructions) — l'ajout de `KESH_ADMIN_BACKUP_DIR=target/kesh-backup` à la
    ligne de commande de la recette, et, à `:174` (« Le montage qui marche, vérifié le 2026-08-04 »),
    la mention que la commande a changé depuis : « vérifié le 2026-08-04 ; `KESH_ADMIN_BACKUP_DIR` ajoutée
    le <date> (15-13b), recette rejouée au T10 » — une date de vérification ne reste pas accolée à une
    commande qui n'est plus celle qui a tourné. Le Dev Agent Record dit que c'est la recette **modifiée**
    qui a tourné au T10. Cette recette a une copie **hors dépôt**, dans la mémoire de travail de
    l'orchestrateur (F-P4-8 a) : la story ne la touche pas ; l'orchestrateur réalignera sa mémoire ;
    (b) **`docs/testing.md`** : recette minimale (`:161-173`, qui crée déjà `/tmp/kesh-e2e/{inbox,documents}`
    → ajouter `backup` au `mkdir -p` et `KESH_ADMIN_BACKUP_DIR=/tmp/kesh-e2e/backup`), recettes de
    récupération de mot de passe (`:278-286`) et d'envoi d'e-mails (`:296-304`) — **toutes trois avec la même
    valeur absolue** `KESH_ADMIN_BACKUP_DIR=/tmp/kesh-e2e/backup` (R4-8 de la validation P4) : la recette de
    `:296-304` se lance depuis un autre répertoire (`KESH_STATIC_DIR=../frontend/build`), où un chemin
    relatif pointerait ailleurs ; Kesh crée le dossier s'il manque (AC 9 b) ;
    (b') **modes `cargo run` documentés pour l'exploitant** (F-P3-5) : `admin-manual.tex:194` (AC 11 d),
    `.env.example:77-82` (AC 8 c), `README.md:90` (« en mode `cargo run` natif sur Linux non-root, lancer
    le backend avec `KESH_PORT` … ») — chacun gagne la mention de `KESH_ADMIN_BACKUP_DIR` ;
    (c) **assumés**, avec leur raison : `frontend/playwright.config.ts:18` (commentaire qui renvoie à
    `docs/testing.md`) ; `frontend/DEBUGGING-KF007.md:17` (note de débogage historique, KF-007 close) ;
    `docs/known-failures.md:119` (archivé, ne se modifie plus) ; `crates/kesh-db/README.md:57` et
    `docs/testing.md:32` (citent `cargo run` sans le lancer) ;
    `frontend/tests/e2e/password-recovery.spec.ts:16` (commentaire de recette **partielle** — la
    `DATABASE_URL` y est élidée en `"mysql://..."` — dont la forme complète est `docs/testing.md:278-286`,
    résolue en (b) ; le modifier toucherait le frontend, que l'AC 15 d laisse intact) ;
    `frontend/tests/e2e/global-setup.ts:34` (message d'erreur « ex: cargo run -p kesh-api » : un
    exemple, pas une recette) ; et ce que rend le second grep sans être une recette de backend :
    `DOCKER_START.md:131`, `.env.example:32`, `:186`, `:249`, `docker-compose.dev.yml:2`,
    `frontend/vite.config.ts:20`, `admin-manual.tex:668`, `:1315`, `docs/testing.md:61`,
    `CHANGELOG.md:655` (mentions du mode, sans recette de lancement) ; `scripts/prepare-release.sh:109`,
    `crates/kesh-db/src/post_restore.rs:1168`, `crates/kesh-db/tests/test_schema_guard.rs:548`,
    `CLAUDE.md:798`, `docs/kesh-specifications.txt:1110` (autres binaires — exemple `kesh-db`, CLI
    `kesh-seed` — ou prose). **`scripts/`** : aucun script ne lance de backend (`grep -rln "cargo run\|kesh-api"
    scripts/` → `prepare-release.sh`, qui lance un exemple `kesh-db`, et `audit-tenant-scoping.py`, qui
    n'en lance aucun). **CI** : `release.yml:91-101` lance l'image publiée (conteneur root, `/data/backup`
    inscriptible) sans import ; `ci.yml` ne lance aucun backend.

## Tasks / Subtasks

- [x] **T0 — Mesures avant d'écrire** (AC 11 e, 11 f, 11 l)
  - [x] Rapatriement de la sauvegarde (AC 11 e) rejoué sur un **conteneur jetable** : un conteneur root
        écrit un fichier `0600` dans un dossier monté de l'hôte ; vérifier qu'il est illisible de
        l'utilisateur, que `sudo cp` + `sudo chown` le rendent lisible, puis supprimer copie et dossier.
  - [x] Graphie de `docker compose config` pour un montage (forme longue `target: /data/backup`), pour la
        ligne de contrôle de l'AC 11 f ; puis le contrôle **rouge** : sur une copie du compose **sans** le
        montage, avec `KESH_ADMIN_BACKUP_DIR=/data/backup` posé dans l'environnement,
        `docker compose config | grep -c 'target: /data/backup'` → **0** (alors que `grep -c '/data/backup'`
        rend 1 : la ligne d'environnement) — R4-2/F-P4-1 de la validation P4.
  - [x] `make fr` sur l'état d'avant : garder le `.log` (référence des `Overfull`) et le PDF aplati
        (référence des contrôles de l'AC 11 l, chacun **rouge** dessus).
- [x] **T1 — Compose, gabarit, fichiers d'exclusion** (AC 8)
  - [x] `docker-compose.yml` et `docker-compose.prod.yml` : montage `./backup` + commentaire (rien d'autre
        dans `docker-compose.prod.yml`).
  - [x] `.gitignore` (`/inbox/`, `/documents/`, `/backup/` — **ancrés**) et `.dockerignore` (`log/`,
        `inbox/`, `documents/`, `backup/`) — AC 8 e ; `git ls-files` sous ces dossiers à la racine → vide ;
        `git check-ignore -v --no-index "frontend/src/routes/(app)/admin/backup/+page.svelte"` → **non**
        ignoré (sans `--no-index`, git ne rapporte jamais un fichier suivi : le contrôle ne pourrait pas
        rougir — R4-3 de la validation P4 ; avec `backup/` non ancré, la même commande le rend ignoré).
  - [x] `.env.example` : commentaire `KESH_ADMIN_BACKUP_DIR` (`:179-182`), paragraphe (d) du mode
        `cargo run` natif (`:77-82`), et les deux énumérations des montages fixes (`:208-210`, `:241-242`,
        AC 8 c).
- [x] **T2 — Code** (AC 7, 9, 15)
  - [x] `config.rs` : `pub const DEFAULT_ADMIN_BACKUP_DIR`, doc-comments.
  - [x] `routes/admin.rs` : `write_backup_file` (`.partial` en `create_new` + `0o600`, `sync_all`, refus si
        le nom final existe, `rename` ; `.partial` supprimé au mieux en cas d'échec) ; dossier créé en `0o700` ; commentaire `:475-477` corrigé ; sites
        antérieurs à la sauvegarde sur `AdminPreImportBackupFailed` ; `avant_sauvegarde` aux appels de
        `check_schema_compat` et `build_keshbackup` (forme `.map_err(avant_sauvegarde)`, AC 15 b, test 20).
  - [x] `errors.rs` : variante `AdminPreImportBackupFailed`, son bras de réponse et son repli ; doc et
        repli d'`AdminFullImportFailed` (AC 15 a, c).
  - [x] `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl` : clé
        `error-admin-pre-import-backup-failed`, à côté de `error-admin-full-import-failed`.
- [x] **T3 — Tests Rust** (AC 7, 8 e, 9, 10 c-e, 15) — voir § *Tests et mutations*. Dans
      `configuration_transmise.rs` : quatrième entrée de `MONTAGES`, fonction pure `source_conforme` (égalité
      exacte pour une source sans `${`), lien `DEFAULT_ADMIN_BACKUP_DIR` ↔ cible de `MONTAGES`, textes
      « trois » mis à jour, message de refus `Y` d'une source exacte (AC 10 c, e) ; test 19 (AC 8 e).
- [x] **T5 — Manuel et brochure** (AC 11) : sites (d) (ligne `KESH_ADMIN_BACKUP_DIR`, montages, `:194`),
      (e), (f) (paragraphe sauvegarde, `:1773`, `:1793`), (i), puis relecture de la brochure (n) ;
      `make fr` (trois PDF), contrôles (k)–(l).
- [x] **T6 — `DOCKER_START.md`, recettes, CHANGELOG** (AC 12 a', 13, 16) — `CLAUDE.md` : la ligne de
      commande de la recette et la mention de date de `:174` (AC 16 a), rien d'autre ; CHANGELOG `:44`
      (montages de `docker-compose.prod.yml`) et `:46` (AC 13 c).
- [x] **T8 — Mutations** : chacune appliquée, test rouge relevé, fichier restauré **et touché**
      (`touch`, mémoire « mutation restaurée, binaire périmé ») ; tableau au Dev Agent Record.
- [x] **T9 — Propagation et inventaire** (AC 14) — exclusions communes
      `--exclude-dir={target,node_modules,.git,_bmad-output,_bmad,.claude,.svelte-kit}` :
      (1) `grep -rnIE "BACKUP_DIR|pré-import|pre-import|admin-full-import-failed|AdminFullImportFailed|/tmp|(compose|décrit les)[^.]{0,20}deux gestes|backup/" …`
      — chaque résultat est dans l'inventaire (résolu ou angle mort), sinon il y entre (le jeton `/tmp`
      est large : trier les seuls sites qui parlent de la sauvegarde, les autres en bloc par fichier). Le
      jeton des gestes est **restreint** comme dans la 15-13a (R4-4 de la validation P4 : `deux gestes`
      seul rendait six homonymes sans rapport — `CLAUDE.md:760`, `routes/users.rs:334`,
      `tests/users_e2e.rs:560`, `invoice_settlements.rs:395`, `user-manual.tex:1309`,
      `invoice-unvalidate.spec.ts:67`) ; la forme restreinte ne rend que `CHANGELOG.md:46` et
      `admin-manual.tex:1793` ;
      (2) les deux greps de l'AC 16 (`cargo run -p kesh-api`, puis `cargo run`) ;
      (3) `git ls-files | grep -E '(^|/)(log|inbox|documents|backup)/'` → seuls des fichiers hors de la
      racine (aujourd'hui : `frontend/src/routes/(app)/admin/backup/`), qu'aucune ligne ancrée ne masque ;
      (4) **par la valeur des montages** (F-P4-3 de la validation P4 : les greps (1) à (3) cherchent le
      sujet, et ne voient pas une phrase qui **énumère** les montages sans nommer la sauvegarde) :
      `grep -rnIE '\./(inbox|documents|log)\b' …` — tout site qui énumère les montages de
      `docker-compose.prod.yml` (ou de `kesh-api`) en entier gagne `./backup`, les autres sont triés
      (inventaire, ligne « grep (4) »).
- [x] **T10 — Gates** : `scripts/test-fast.sh` complet (base du worktree remise à zéro avant —
      `DROP`/`CREATE` de **ses** bases, jamais un redémarrage du conteneur), frontend non touché (gate
      frontend tout de même, CLAUDE.md), **E2E complet au dernier commit de code** (D7), lancé avec la
      recette corrigée de l'AC 16 ; la story ne touche aucune migration (exception `kesh-db` sans objet).

*(T4 — CI — et T7 — recette MariaDB sur conteneur jetable — sont de la 15-13a.)*

## Tests et mutations

Périmètre : tests **neufs ou modifiés** par la 15-13b, de `de285ea8` au commit de développement.
Numérotation de la fiche unique conservée.

| # | Test (fichier) | Établit | Mutation qui le rend rouge |
|---|---|---|---|
| 3 | `transmission` (existant, `configuration_transmise.rs:1733`, `MONTAGES` étendu, contrôle par `source_conforme` ; **et** : une entrée de `MONTAGES` a pour cible `kesh_api::config::DEFAULT_ADMIN_BACKUP_DIR`, R4-1) | AC 7 a, 8 a, 10 c | **M9** retirer `./backup:/data/backup` de `docker-compose.prod.yml` · **M10** `./backups:/data/backup` dans `docker-compose.yml` · **M11** `${KESH_BACKUP_HOST_DIR:-./backup}:/data/backup` dans `docker-compose.yml` (rouge aussi au (F) : fantôme) · **M15** (aussi au test 6 b) |
| 5 | `s_sources_de_montage` (existant, complété) — exerce la fonction pure `source_conforme` (AC 10 d) : en `Y` avec `"./backup"`, `./backup` accepté et `./backups` **refusé** ; en `Y` avec un préfixe `${…:-`, la source variable acceptée ; en `P`, égalité | AC 10 c-d | **M14** `source_conforme` revient à `starts_with` pour toutes les entrées (le cas `./backups` en `Y` passe) |
| 6 | `from_env_empty_or_blank_vars_take_code_default_silently` (existant, `config.rs:1648`, assertion par la constante — 6 a) + `from_env_absent_backup_dir_takes_data_backup` (nouveau — 6 b, assertion sur le **littéral** `"/data/backup"`, seul littéral voulu côté test : il fixe la valeur dont #552 dépend) | AC 7 | **M15** **valeur de la constante** `DEFAULT_ADMIN_BACKUP_DIR` changée en `"/tmp"` (geste naturel : 6 a, qui compare la constante à elle-même, reste vert ; 6 b et le test 3 rougissent) |
| 11 | `write_pre_import_backup_en_0600` (nouveau, `#[cfg(unix)]`, `mod tests` neuf de `routes/admin.rs`, `tempfile`) — `write_pre_import_backup` sur un sous-dossier absent d'un `TempDir` : dossier créé en `0700`, fichier en `0600`, contenu identique ; **assertions de montage** : un fichier témoin écrit par `std::fs::write` dans le même dossier n'est **pas** en `0600`, **et** un dossier témoin créé par `std::fs::create_dir` à côté n'est **pas** en `0700` (R4-9 de la validation P4 : sans ce second témoin, M25 passe sous umask `077`) — sinon l'umask rend le test non discriminant : échec explicite, dont le message **nomme la cause** (« umask de l'environnement trop restrictif (077 ?) : le test ne peut pas distinguer le mode posé par le code »), et non un défaut du code ; sous l'umask `022` du poste de dev et de la CI, sans effet. Après l'écriture, aucun `.partial` ne reste dans le dossier | AC 9 a-b | **M24** `.mode(0o600)` retiré · **M25** `DirBuilder` sans `mode` (dossier en `0755`) |
| 12 | `write_backup_file_n_ecrase_pas` (nouveau, même module) — `write_backup_file` sur un chemin final **déjà occupé** échoue en `AppError::AdminPreImportBackupFailed`, le fichier existant garde son contenu, et aucun `.partial` ne reste ; **+** `write_backup_file_passe_par_un_partiel` (nouveau, même module, C-15-13-21) — un `<chemin>.partial` **préexistant** (écriture interrompue) fait échouer l'appel en `AdminPreImportBackupFailed`, le nom final **n'existe pas** après l'échec, et ce `.partial` préexistant **garde son contenu** (on ne supprime que ce qu'on a créé, F-P5-3 de la validation P5) : le nom de la sauvegarde n'apparaît qu'au terme d'une écriture complète passée par le `.partial` | AC 9 a, 9 c | **M26** vérification d'existence du nom final retirée (le `rename` écrase) · **M36** `write_backup_file` rend `AdminFullImportFailed` · **M46** écrire directement le nom final, sans `.partial` (le second test rougit : l'appel réussit, le nom final existe) · **M51** nettoyage du `.partial` étendu à l'échec de `create_new` (le second test rougit : le `.partial` préexistant a disparu) |
| 14 | `avant_sauvegarde_convertit_les_echecs_anterieurs` (nouveau, unitaire, `routes/admin.rs`) — `AdminFullImportFailed` et `AdminFullExportFailed` → `AdminPreImportBackupFailed`, détail conservé ; `ImportSchemaMismatch` et `InvalidBackupStructure` rendus tels quels | AC 15 b | **M30** `avant_sauvegarde` rend l'erreur telle quelle · **M31** `avant_sauvegarde` convertit toute variante (un 400 devient un 500) |
| 15 | `pre_import_backup_failed_maps_to_500_without_promise` (nouveau, `errors.rs`, repli Rust — aucun catalogue n'est chargé dans les tests de la bibliothèque, `init_error_i18n` n'y est jamais appelé) — 500, `ADMIN_PRE_IMPORT_BACKUP_FAILED`, message sans le détail et sans promesse de sauvegarde, et qui **contient** « aucune sauvegarde n'a été créée » (F-P4-4) ainsi que les deux pistes, « dossier de sauvegarde » **et** « base de données » (R5-1 de la validation P5) ; **témoin** : `AdminFullImportFailed` rend `ADMIN_FULL_IMPORT_FAILED` et un repli **différent** | AC 15 a, c | **M32** le bras de la variante neuve rend le code et la clé d'`AdminFullImportFailed` · **M50** repli Rust de la variante neuve sans « base de données » (une seule piste) |
| 16 | `catalogues_distinguent_l_echec_de_sauvegarde` (nouveau, `errors.rs`, `I18nBundle::load` sur `concat!(env!("CARGO_MANIFEST_DIR"), "/../kesh-i18n/locales")` — chemin indépendant du répertoire courant (F-P3-10) —, **sans** toucher au catalogue global) — pour chacune des quatre locales : `error-admin-pre-import-backup-failed` ≠ `error-admin-full-import-failed` de la même locale ; hors `fr-CH`, ≠ le texte `fr-CH` (sinon `format` est retombé sur le français, `loader.rs:116-121`) ; ne contient pas la **promesse** de l'ancien texte, phrase intérieure de sa parenthèse **sans les parenthèses** (R3-9) : `un backup automatique a été créé avant l'opération`, `vor dem Vorgang wurde automatisch ein Backup erstellt`, `an automatic backup was created before the operation`, `prima dell'operazione è stato creato un backup automatico` — **pas** des sous-chaînes que la négation prescrite contient (`a été créé` est préfixe de « n'a été créée ») ; **assertion de montage** : chaque interdit figure dans `error-admin-full-import-failed` de **sa** locale (texte inchangé), faute de quoi l'interdit est une coquille et le test passe à vide. Vérifié en remédiation P3 : `grep -cF` de chaque interdit (sans parenthèses) → 1 dans le `messages.ftl` de sa locale (`fr-CH:1478`, `de-CH`/`en-CH`/`it-CH:1407`), et dans aucun autre ; `printf '%s' "<négation naturelle>" \| grep -cF -- "<interdit>"` → 0 pour les vingt couples (quatre formulations de l'AC 15 c et « Non è stato creato un backup automatico », contre les quatre interdits) ; **et** (F-P4-4 de la validation P4) chaque texte neuf **contient** la négation de sa locale, à la lettre (`aucune sauvegarde n'a été créée`, `Es wurde keine Sicherung erstellt`, `No backup was created`, `Non è stato creato alcun backup`) ; **et** (R5-1 de la validation P5) chaque texte neuf **contient** ses deux jetons de piste, à la lettre (`dossier de sauvegarde`/`base de données`, `Sicherungsordner`/`Datenbank`, `backup folder`/`database`, `cartella di backup`/`database`) | AC 15 c | **M33** texte `de-CH` remplacé par celui d'`error-admin-full-import-failed` · **M34** clé retirée d'`it-CH` (rouge aussi à `parity_between_locales`) · **M38** texte `fr-CH` neuf qui recopie la parenthèse de l'ancien · **M45** texte `fr-CH` neuf réduit à « Échec de l'import. » (sans négation ni promesse) · **M49** texte `en-CH` neuf sans « database » (une seule piste, le dossier) |
| 17 | `full_import_refuses_when_backup_cannot_be_written` (nouveau, `admin_full_import_e2e.rs`) — `admin_backup_dir` = chemin d'un **fichier** existant (la création du dossier échoue même sous root : en CI comme en local) ; `spawn_app` factorisé en `spawn_app_with(pool, config)` (DRY) ; 500, `ADMIN_PRE_IMPORT_BACKUP_FAILED`, message = repli (ce binaire n'appelle pas `init_error_i18n`) ; installation intacte (société et administrateur d'origine, comme le test 18) | AC 15 a, 9 c | **M35** la création du dossier dans `write_pre_import_backup` (`admin.rs:473` à la spécification, réécrite en `DirBuilder` : F-P6-7) rend `AdminFullImportFailed` |
| 18 | `full_import_rolls_back_on_insert_failure` (existant, `admin_full_import_e2e.rs:559-605`, complété) — échec **après** la sauvegarde : code `ADMIN_FULL_IMPORT_FAILED` | AC 15 a | **M37** le site `restore :` de `run_backup_and_restore` (`admin.rs:291`) rend `AdminPreImportBackupFailed` |
| 19 | `montages_hors_du_depot` (nouveau, `configuration_transmise.rs`) — pour **chaque** entrée de `MONTAGES`, le dossier de sa source `docker-compose.prod.yml` (`./x` → `x`) a sa ligne dans `.gitignore` sous la forme **ancrée** `/x/` — **seule tolérance** : `log`, pour qui `log/` (ligne existante, `.gitignore:34`) est admise, la liste des tolérances étant une constante qui ne compte que `log` (assertion) — et dans `.dockerignore` (`x/`) ; la liste est **dérivée** de `MONTAGES`, pas recopiée (un cinquième montage ajouté demain est contrôlé sans retouche) ; **assertion de montage** : `MONTAGES` compte **au moins** quatre entrées, dont celle de cible `/data/backup` (R4-5 de la validation P4, C-15-13-26 : un compte exact obligerait à retoucher le test au cinquième montage, contre la phrase précédente), et chaque ligne est cherchée **entière** (après `trim`), pas en sous-chaîne (`log/` ne doit pas satisfaire `/backup/`, ni `backup/` satisfaire `/backup/`) | AC 8 e | **M39** retirer `/backup/` de `.gitignore` · **M40** retirer `backup/` de `.dockerignore` · **M41** `/backup/` remplacé par `backup/` (non ancré) dans `.gitignore` (R3-1) |
| 20 | `avant_sauvegarde_branchee_aux_appels_de_l_import` (nouveau, `mod tests` de `routes/admin.rs`, F-P5-1 de la validation P5, C-15-13-28) — **garde lexicale** : lit `include_str!("admin.rs")` tronqué à la première ligne `#[cfg(test)]` (le module de test ne se lit pas lui-même), lignes dont le `trim_start` commence par `//` écartées (les doc-comments nomment `build_keshbackup` : `:129`, `:256`) ; repère chaque **appel** `check_schema_compat(` et `build_keshbackup(` (identifiant suivi de `(`, précédé ni de `fn ` ni d'un `use`), le rattache à la fonction de premier niveau qui l'entoure (dernière ligne `pub async fn <nom>` / `async fn <nom>` en colonne 0 qui le précède) et lit son **instruction** (texte jusqu'au `;` suivant, retours à la ligne compris : `rustfmt` peut la couper) — dans `full_import` et `run_backup_and_restore`, l'instruction **contient** `.map_err(avant_sauvegarde)` ; dans `full_export`, elle **ne le contient pas** ; **assertion de montage** : exactement **un** appel de `check_schema_compat` et **deux** de `build_keshbackup` (un par route), sinon échec qui nomme le décompte trouvé — un garde qui ne trouve rien passerait à vide, et un appel ajouté demain doit être trié | AC 15 b | **M47** `.map_err(avant_sauvegarde)` retiré à l'appel de `check_schema_compat` (`:172`) · **M48** retiré à l'appel de `build_keshbackup` (`:259`) · **M52** `.map_err(avant_sauvegarde)` **ajouté** à l'appel de `build_keshbackup` de `full_export` (`:39`) — branche négative (F-P6-2 de la validation P6) |

**Décompte** (recompté sur le tableau) : 12 lignes, toutes côté Rust, qui portent **14 fonctions de test**
(10 neuves : lignes 6 b, 11, 12 ×2, 14, 15, 16, 17, 19, 20 ; 4 existantes complétées : lignes 3, 5, 6 a, 18), dans
cinq fichiers existants (`configuration_transmise.rs`, `config.rs`, `routes/admin.rs`, `errors.rs`,
`admin_full_import_e2e.rs`) ; **28 mutations** (M9–M11, M14, M15, M24–M26, M30–M41, M45–M52 — M52 ajoutée au T0, F-P6-2), toutes
attendues rouges. M11 rougit **deux** familles ((T) et (F)), M15 et M34 deux tests chacune : attendu, le
relever. **M26** change de sens avec l'écriture par `.partial` : `create_new` porte désormais sur le
`.partial`, et c'est la vérification d'existence du nom final qui empêche l'écrasement. Non couverts par
un test, écrits comme angles morts : les sites `admin.rs:236`, `:240`, `:246` (panne de base avant la
sauvegarde, non injectable sans abstraction) ; les pannes **réelles** derrière `:172` et `:259` (lecture du
schéma, lecture des données — non injectables, F-P5-1) : leur **branchement** sur `avant_sauvegarde` est
gardé par le test 20 (M47, M48 rouges), mais aucun test n'en fait passer une panne effective jusqu'à la
réponse HTTP ; et la suppression du `.partial` de l'AC 9 a après un échec de
`write_all`, `sync_all`, de la vérification du nom final ou de `rename` (non injectable) — y compris l'échec
de cette suppression (`.partial` laissé, nommé par le `warn!`). En revanche, qu'un échec de `create_new`
**ne supprime pas** le `.partial` préexistant est vérifié par le test 12 (F-P5-3 de la validation P5).

Le test **(L)** (`garde_lecture_du_code`) et **(E)** (`env_example`) doivent rester verts **sans
modification** de leurs listes : la story n'ajoute aucune lecture d'environnement ni aucune variable.

## Dev Notes

### Ce que la story change, et ce qu'elle préserve

- **Code** : trois modules de `kesh-api` — `config` (défaut, tests), `routes/admin` (écriture du fichier,
  `avant_sauvegarde`), `errors` (variante de #576) — et les catalogues de `kesh-i18n` (une clé, quatre
  locales). Aucune migration ; `kesh-db` non touché ; `admin_backup` non touché (AC 15 b) ; frontend non
  touché (AC 15 d) ; les textes de l'écran de restauration disent « côté serveur », sans chemin —
  `messages.ftl:1453`, `:1458` — et restent vrais.
- **Préservé** : DC5 (« jamais d'import sans backup réussi ») ; le texte d'`error-admin-full-import-failed`
  (vrai pour tout ce qu'il couvre désormais) ; l'ordre verrou → sauvegarde → restore
  (`routes/admin.rs:226-261`) ; le nom du fichier ; la réponse JSON (`backupCreated`) ; la transmission de
  `KESH_ADMIN_BACKUP_DIR` en `${…:-}` (C75) ; les trois montages existants (C83, #558) ;
  `docker-compose.dev.yml`.

### La mise à jour d'une installation existante — les cas propres à la sauvegarde

*(Les cas MariaDB sont dans la 15-13a.)*

| État de l'installation avant mise à jour | Après la mise à jour | Ce que dit le manuel |
|---|---|---|
| compose **gardé** (non re-téléchargé), image 0.13.0 | sauvegarde dans `/data/backup` **non monté** (éphémère, comme avant) | le geste « ajouter le montage `./backup` » du paragraphe « Pour qui garde son fichier compose » (AC 11 f) ; l'« Effet sans refus » conditionné au compose le dit |
| compose neuf, **image ancienne** (oubli du `pull`) | sauvegarde toujours dans `/tmp` (défaut 0.12.x) | le manuel impose `docker compose pull` avant `up -d` (déjà écrit, 15-11a) |
| **restaurer après un import raté** (le cas même de #552) | la sauvegarde est dans `./backup`, `root` `0600` : illisible de File Station, de SMB et d'un `scp` sans `sudo` | rapatriement en SSH (`sudo cp` puis `sudo chown` vers un dossier partagé), import par l'écran, suppression de la copie (AC 11 e, C-15-13-8) |

`docker-compose.prod.yml` (Synology, MariaDB externe) : seul change le montage `./backup`, créé par Docker
au premier `up -d` dans `/volume1/docker/kesh/` (propriétaire `root`, dans le périmètre Hyper Backup du
dossier du compose — `admin-manual.tex:1555`).

### Pourquoi `/data/backup`, singulier, et un montage fixe (C-15-13-3)

- Le gabarit propose **déjà** `#KESH_ADMIN_BACKUP_DIR=/data/backup` (`.env.example:182`) : un exploitant
  qui l'a décommenté écrit déjà dans `/data/backup`. Monter **ce** chemin le rend persistant sans qu'il
  ait rien à changer ; `/data/backups` (pluriel) aurait laissé ce cas dans le conteneur.
- Défaut **dans le code** (comme `/data/documents`, `/data/inbox`, `config.rs:951-957`) plutôt qu'un défaut
  de déploiement dans le compose : C75 (le défaut vit dans le code, le compose transmet `${…:-}`) ; et un
  compose **tiers** qui monte `/data` profite du changement.
- Montage **fixe** dans les deux compose : une variable d'hôte neuve relèverait de #558 (procédure de
  déplacement, périmètre de sauvegarde) ; le test (F) rougirait d'ailleurs sur tout `KESH_*` inconnu.
- Hors Docker, `/data/backup` n'est pas inscriptible par un utilisateur ordinaire : l'import est alors
  refusé **avant** toute suppression (`create_dir_all` échoue, `AdminPreImportBackupFailed` — dont le
  message le dit désormais, AC 15), sans perte ; les recettes du dépôt et les modes `cargo run` documentés
  posent la variable (AC 16). Le défaut `/tmp` d'avant ne protégeait pas mieux un serveur dont `/tmp` est
  un tmpfs.

### Pourquoi `0600` (C-15-13-4), et ce que cela coûte (C-15-13-8)

Dans `/tmp` du conteneur, le fichier mourait avec lui ; sur l'hôte, il vit — dans un dossier que Docker
crée `root:root` en `0755`, et, sur `docker-compose.yml`, à côté de dossiers que le manuel invite à
partager en SMB. Le `.keshbackup` contient les condensés des mots de passe et les jetons de session
(`admin-manual.tex:1654`). `0600` (et `0700` pour un dossier que Kesh crée) est le minimum qu'impose
le changement de durée de vie. Refus d'écraser : le nom est unique (horodatage, pid, compteur) ; un
écrasement serait un défaut, autant qu'il échoue. **Persistant, un fichier interrompu en cours d'écriture
le devient aussi** (F-P4-6) : écrit sous son nom final, il serait indiscernable d'une sauvegarde valide
jusqu'à l'import, qui le refuserait en « corrompu » (SHA-256) au moment même où l'on en a besoin. D'où
l'écriture sous `.partial` puis le renommage (C-15-13-21) : seul un fichier complet et synchronisé porte le
nom d'une sauvegarde.

Le prix est le rapatriement : un fichier `root` en `0600` ne se télécharge ni par File Station ni par SMB.
Il est **assumé** (C-15-13-8, décision de l'orchestrateur) plutôt qu'un mode plus large : la sauvegarde
contient toute la comptabilité et ses secrets, et le geste `sudo cp` + `sudo chown` tient en deux lignes
du manuel, rejouées au T0. Sur Synology, les ACL du dossier partagé `/volume1/docker` peuvent s'ajouter aux
droits POSIX : l'effet réel n'est pas mesuré depuis le poste, le manuel le dit (angle mort).

### Inventaire fermé des sites — parties sauvegarde

Greps de constitution : ceux du T9. Constitué à partir de l'inventaire de la fiche unique (57 lignes,
réparties entre les deux sous-fiches : § *Découpage* de la fiche index), puis complété des sites trouvés en
validation P3. **Résolu** = un AC le traite ; **assumé** = angle mort écrit.

| Site | Sujet | Sort |
|---|---|---|
| `docker-compose.yml` `volumes:` de `kesh-api` ; `docker-compose.prod.yml` idem | montage sauvegarde | résolu (AC 8) |
| `docker-compose.yml:121`, `docker-compose.prod.yml:151` (`KESH_ADMIN_BACKUP_DIR: ${…:-}`) | transmission | inchangé, voulu (AC 8 b) |
| `.env.example:179-182` | gabarit, commentaire du défaut | résolu (AC 8 c) |
| `.env.example:77-82` | mode `cargo run` natif (hors Docker) | résolu (AC 8 c, F-P3-5) |
| `.env.example:208-210`, `:241-242` (« ses montages sont fixes — `./inbox`, `./documents`, `./log` ») | énumération des montages de `docker-compose.prod.yml` | résolu (AC 8 c, F-P4-3) |
| `CHANGELOG.md:44` (« **Inchangé** : … les montages restent `./documents`, `./inbox`, `./log` ») | énumération des montages, entrée #550 de `[0.13.0]` | résolu (AC 13 c, F-P4-3) |
| grep (4) du T9, reste : `docker-compose.yml:127-143`, `docker-compose.prod.yml:112-168` (commentaires et montages un par un), `docker-compose.dev.yml:34`, `.env.example:216`, `:237`, `:244`, `:263`, `:266`, `CHANGELOG.md:46` (« revient à `./documents` »), `:674`, `crates/kesh-api/tests/configuration_transmise.rs:200-205`, `:367`, `docs/kesh-specifications.txt:1739`, `admin-manual.tex:819-821`, `:837`, `:853`, `:884`, `:898`, `:1421`, `:1516`, `:1555`, `:1778` | un montage nommé seul, ou une variable d'hôte — aucune énumération de **tous** les montages | **assumé** — restent vrais avec le quatrième (`:1421`, `:1555` : périmètre de sauvegarde, AC 11 i ; `:200-205` : `MONTAGES`, résolu par l'AC 10 c ; `:819-821` : tableau des variables d'hôte, sans ligne pour `./backup`, AC 11 d) |
| `.github/workflows/release.yml:91-101` | lance l'image publiée (root, `/data/backup` inscriptible), sans import | **assumé** (AC 16) |
| `.gitignore` (seul `log/`, `:34`), `.dockerignore` (aucun) | dossiers montés par défaut (`./backup`, `./inbox`, `./documents`, `./log`) non ignorés dans un clone | résolu (AC 8 e, test 19) ; `log/` non ancré : tolérance nommée |
| `frontend/src/routes/(app)/admin/backup/+page.svelte`, `+page.ts` (versionnés) ; rendus par le jeton `backup/` du grep (1) (F-P5-2 de la validation P5) : `frontend/src/routes/(app)/admin/restore/+page.ts:1`, `frontend/src/lib/features/admin-backup/AdminBackupPanel.svelte:4`, `crates/kesh-db/migrations/20260915000001_audit_log_company_id.sql:22` | homonyme du dossier `backup` (route, commentaires ; la migration ne se modifie pas — P8) | **assumé** — protégé par l'ancrage `/backup/` (AC 8 e, M41) ; les trois derniers ne parlent pas du dossier de sauvegarde |
| `crates/kesh-api/src/config.rs:250-256`, `:939-947`, `:1643-1661` | défaut `/tmp` | résolu (AC 7) |
| `crates/kesh-api/src/config.rs:508`, `:1508` | `/tmp` des constructeurs de test | **assumé**, voulu (AC 7 c) |
| `crates/kesh-api/src/config.rs:1586`, `:1601`, `:1748`, `:1769` | `KESH_ADMIN_BACKUP_DIR` comme variable témoin des tests (`reset_env`, `VIDE_EGALE_DEFAUT`, `env_nonempty`) | **assumé** — restent vrais, aucune valeur `/tmp` |
| `crates/kesh-api/src/routes/admin.rs:259-261`, `:471-495` | écriture | résolu (AC 9) |
| `crates/kesh-api/src/routes/admin.rs:475-477` | commentaire faux (« avant le verrou ») | résolu (AC 9 d) |
| `crates/kesh-api/src/routes/admin.rs:42`, `:129-130`, `:174`, `:219`, `:227`, `:481`, `:492` | doc et journal « backup pré-import » | **assumé** — restent vrais (`:481` est le nom du fichier, inchangé) |
| `crates/kesh-api/src/routes/admin.rs:467-470` | doc de `write_pre_import_backup` (« Échec d'écriture → 500 ») | résolu (AC 9 a : mode `0600`, refus d'écraser, variante d'erreur) |
| `crates/kesh-api/src/routes/admin.rs:236`, `:240`, `:246`, `:473`, `:487` ; `:172`, `:259` (appels) | échecs antérieurs à la sauvegarde (sept sites) | résolu (AC 15 a-b) ; branchement de `:172` et `:259` gardé par le test 20 ; `admin.rs:39` (`build_keshbackup` de l'**export**) **sans** `avant_sauvegarde`, voulu (test 20) |
| `crates/kesh-api/src/routes/admin.rs:283` à `:462` (douze sites `AdminFullImportFailed`) | échecs postérieurs | **inchangé, voulu** (AC 15 a) |
| `crates/kesh-api/src/errors.rs:448-452`, `:1819-1828` | variante et message de #576 | résolu (AC 15 a, c) |
| `crates/kesh-api/src/admin_backup/import.rs:204`, `admin_backup/export.rs:103-105` | variantes nées hors du module | résolu **à l'appel** (AC 15 b) ; fichiers non modifiés |
| `crates/kesh-api/src/admin_backup/export.rs:5` | doc « backup pré-import » | **assumé** — reste vrai |
| `crates/kesh-i18n/locales/*/messages.ftl` — `error-admin-full-import-failed` (`fr-CH:1478`, autres `:1407`) | message de #576 | résolu (AC 15 c) |
| `crates/kesh-api/src/routes/admin.rs:72-110` (`stream_via_tempfile`, `std::env::temp_dir()`) | fichier temporaire de l'**export** | **assumé** : temporaire par nature, supprimé après envoi |
| `crates/kesh-db/src/post_restore.rs:591`, `crates/kesh-db/tests/post_restore_transactionality.rs:8`, `crates/kesh-api/tests/admin_full_import_e2e.rs:704` | `AdminFullImportFailed` du rejeu des backfills, **postérieur** à la sauvegarde (R3-6, F-P3-7) | **inchangé, voulu** — échecs postérieurs, le message reste vrai ; `kesh-db` non touché |
| `crates/kesh-api/tests/configuration_transmise.rs:153-154` (`AJOUTS`), `:196`, `:199-207` (`MONTAGES`), `:378-379` | transmission ; montages ; textes « trois » | `:153-154` inchangé, voulu (AC 8 b) ; `:196`, `:199-207`, `:378-379` résolus (AC 10 c, e) |
| `CLAUDE.md:178-182`, `docs/testing.md:161-173`, `:278-286`, `:296-304` | recettes `cargo run -p kesh-api` sans `KESH_ADMIN_BACKUP_DIR` | résolu (AC 16 a-b) |
| `admin-manual.tex:194`, `README.md:90` | modes `cargo run` natif documentés pour l'exploitant | résolu (AC 16 b', 11 d) |
| `frontend/playwright.config.ts:18`, `frontend/DEBUGGING-KF007.md:17`, `docs/known-failures.md:119`, `crates/kesh-db/README.md:57`, `docs/testing.md:32` | autres mentions de `cargo run -p kesh-api` | **assumé** (AC 16 c) |
| `frontend/tests/e2e/password-recovery.spec.ts:16`, `frontend/tests/e2e/global-setup.ts:34` | `cargo run -p kesh-api` | **assumé** (AC 16 c) |
| `DOCKER_START.md:131`, `.env.example:32`, `:186`, `:249`, `docker-compose.dev.yml:2`, `frontend/vite.config.ts:20`, `admin-manual.tex:668`, `:1315`, `docs/testing.md:61`, `CHANGELOG.md:655` | mentions du mode `cargo run`, sans recette de lancement (second grep de l'AC 16) | **assumé** (AC 16 c) |
| `scripts/prepare-release.sh:109`, `crates/kesh-db/src/post_restore.rs:1168`, `crates/kesh-db/tests/test_schema_guard.rs:548`, `CLAUDE.md:798`, `docs/kesh-specifications.txt:1110` | `cargo run` d'autres binaires, ou prose | **assumé** (AC 16 c) |
| `DOCKER_START.md` § *Notes* (`:140`, puce neuve — le fichier n'a pas de section « dossiers ») | `./backup` | résolu (AC 12 a', F-P5-4) |
| `admin-manual.tex:757`, `:807-822` (dont `:812`, énumération des montages fixes) | `sec:env-vars` (ligne `KESH_ADMIN_BACKUP_DIR`, montages) | résolu (AC 11 d, F-P4-3) |
| `admin-manual.tex:1643` (« sans accès SSH ni ligne de commande », ouverture de `sec:backup-ui-keshbackup`) | contradiction avec le rapatriement en SSH de la même section | résolu (AC 11 e, F-P4-7 : phrase d'exception) |
| `admin-manual.tex:1712` (« restauration self-service sans SSH »), `website/roadmap.html:231` (« SSH-free migration and restore »), `README.md:48` (« sans accès SSH ✓ », F-P6-4 de la validation P6) | promesse de l'export/import par l'écran | **assumé** — vraie pour un `.keshbackup` exporté ; la sauvegarde pré-import en fait exception, dite à `:1643` (F-P4-7) |
| `admin-manual.tex:1417-1425`, `:1664`, `:1667-1672`, `:1699` | sauvegarde | résolu (AC 11 e, i) |
| `admin-manual.tex:1452-1477`, `:2312-2317` (`BACKUP_DIR` du script de sauvegarde) | homonyme : variable de shell, pas de Kesh | **assumé** — hors sujet (le `cd` et `exec db` du même script : 15-13a, AC 11 h) |
| `admin-manual.tex:1757-1841` — paragraphe sauvegarde, `:1773` (« Effets sans refus »), `:1793` (« deux gestes », « Sous `environment:` ») | mise à jour 0.13.0 — parties sauvegarde | résolu (AC 11 f) |
| `docs/manual/fr/marketing-brochure.tex:398` | « restauration sans accès SSH » | relu (AC 11 n) — vrai pour un `.keshbackup` exporté ; décision au Dev Agent Record |
| `CHANGELOG.md:46` — « — la sauvegarde pré-import reste dans `/tmp` » et « deux gestes » | rendues fausses | résolu (AC 13 c) |

**Décompte** : 41 lignes (recompté sur le tableau — `grep -c '^| '` rend 42, en-tête compris ; le
séparateur commence par `|---` et n'est pas compté) : 36 à la validation P4, plus cinq (F-P4-3 : `.env.example`,
`CHANGELOG.md:44`, reste du grep (4) ; F-P4-7 : `:1643`, `:1712`/`roadmap.html`) ; `:812` enrichit une ligne
existante.

### Angles morts assumés (écrits, non traités)

- **ACL Synology** sur `/volume1/docker/kesh/backup` : peuvent élargir ou restreindre l'accès au fichier
  `0600` ; non mesurées depuis le poste (AC 11 e).
- **Échecs antérieurs à la sauvegarde sans test** : `routes/admin.rs:236`, `:240`, `:246` (panne de base
  avant le verrou) et la suppression du `.partial` de l'AC 9 a — non injectables sans abstraction ;
  relus en revue, écrits ici. *(Revue de code P1, E5 : leur conversion est désormais gardée
  **lexicalement** par `echecs_avant_sauvegarde_tous_convertis` — aucune `AdminFullImportFailed` ni
  `AdminFullExportFailed` avant l'appel de `write_pre_import_backup`, autant de `?` que de conversions ;
  la panne effective, elle, reste non injectée.)* **Échecs de `write_all`, `sync_all`, `try_exists` et
  `rename`** (revue de code P1, B4) : aucun n'est exercé individuellement ; ils convergent vers la même
  suppression du `.partial`, dont le retrait est attrapé par le test 12 a, mais une mutation du détail ou de
  la variante d'une seule de ces branches ne ferait rougir aucun test. **`:172` et `:259`** (F-P5-1 de la validation P5) : la panne réelle (lecture
  du schéma, lecture des données, `zip`) n'est pas injectable non plus ; seul leur **branchement** sur
  `avant_sauvegarde` est gardé, **lexicalement**, par le test 20 (M47, M48) — un garde qui lit le source
  et non un comportement : une écriture que le test ne reconnaît pas (fermeture, conversion différée)
  le fait rougir à tort, ce qui est voulu (forme prescrite à l'AC 15 b) ; aucun test ne fait traverser
  une panne effective jusqu'à la réponse HTTP. L'extraction d'une fonction `sauvegarde_pre_import`
  testable sur une base dégradée a été écartée (C-15-13-28).
- **`.partial` non supprimé** : si l'écriture échoue après la création **et** que `remove_file` échoue à
  son tour — ou si le processus est arrêté pendant l'écriture, ou encore si la **requête est annulée**
  (client qui coupe la connexion : le futur du handler est abandonné, ni `remove_file` ni le `warn!` ne
  s'exécutent ; revue de code P1, B2 = E1) —, un `….keshbackup.partial` reste dans le
  dossier de sauvegarde ; il ne porte pas le nom d'une sauvegarde, le message « aucune sauvegarde n'a été
  créée » reste vrai, le `warn!` nomme le chemin, et le manuel dit qu'un `.partial` peut être supprimé
  (AC 9 a, 11 e, C-15-13-21). Suppression non injectable, non testée.
- **Fenêtre entre la vérification du nom final et le renommage** : non fermée (`rename` remplace une cible
  existante sur Unix ; `renameat2(RENAME_NOREPLACE)` n'est pas dans `std`, et un `hard_link` sans écrasement
  dépend du système de fichiers du montage) ; sans portée pratique — le nom est unique par construction, et
  un second écrivain du même nom serait déjà un défaut (C-15-13-21).
- **Durabilité du renommage** : le dossier n'est pas synchronisé après `rename` ; une coupure de courant
  juste après pourrait perdre l'entrée du dossier — le fichier était complet et synchronisé, seul le nom
  serait en cause. Non traité ; écrit aussi au doc-comment de `write_backup_file` (revue de code P1,
  B3 = E4).
- **`try_exists` suit les liens symboliques** (revue de code P1, E6) : un lien pendant au nom final est vu
  « absent », et `rename` remplace le lien (pas sa cible). Sans portée : nom unique par construction,
  dossier `0700` créé par Kesh. Écrit au doc-comment, non traité.
- **SELinux** : le montage `./backup` n'a pas de suffixe `:z` (les trois montages existants non plus) ;
  sur un hôte SELinux en mode `enforcing`, l'écriture pourrait être refusée — l'import le dirait alors
  (AC 15). Non mesuré.
- **Purge des sauvegardes** : à la charge de l'exploitant (inchangé) ; elles sont désormais persistantes,
  donc elles s'accumulent — une par import, de la taille de la base. Le manuel le dit ; pas de rotation.
- **Emplacement non communiqué à l'écran** : la réponse de l'import ne porte pas le nom du fichier
  (`backupCreated` seulement) ; écarté (C-15-13-6).
- **Contrôle au démarrage du dossier de sauvegarde** (un `warn!` si `/data/backup` n'est pas
  inscriptible) : écarté (C-15-13-9, rectifié par C-15-13-27) — le message de l'AC 15 **nomme les deux
  causes possibles**, le dossier de sauvegarde et la base de données, au moment où elles comptent (il ne
  peut pas dire laquelle : le journal le dit), les tests 15 et 16 l'exigent (M49, M50), et les recettes
  hors Docker posent la variable (AC 16). Ce fondement tient **tant que** le message nomme le dossier :
  s'il cessait de le faire, l'écartement serait à rouvrir.
- **Parents créés en `0700`** (R6-3 = F-P6-6) : `DirBuilder` récursif pose `0700` sur chaque niveau qu'il
  crée ; un `KESH_ADMIN_BACKUP_DIR` sous un parent absent (hors Docker, sous root : `/data`) le crée fermé.
  Sans conséquence dans le conteneur (root) ; écrit au doc-comment, non traité. *(Revue de code P1,
  E3 : le comportement est désormais **testé** sur deux niveaux par
  `write_pre_import_backup_dossiers_crees_et_existants`, qui vérifie aussi qu'un dossier existant n'est
  pas modifié — A-4.)*
- **Montage d'hôte `./backup` créé `root:root` `0755`** : l'exploitant qui le veut fermé fait
  `chmod 700 backup` (manuel) ; les fichiers sont `0600` de toute façon.

### Règle de découpage

**Deux crates, quatre modules de code** — `kesh-api::config` (le défaut seulement), `kesh-api::routes::admin`,
`kesh-api::errors` et les catalogues de `kesh-i18n` (`locales/*/messages.ftl`, un seul module métier
quelle que soit la locale) —, plus des tests (`configuration_transmise.rs` étendu,
`admin_full_import_e2e.rs`) et des fichiers de configuration et de documentation (`docker-compose.yml`,
`docker-compose.prod.yml`, `.env.example`, `.gitignore`, `.dockerignore`, `CLAUDE.md` — la recette et sa
date —, `docs/testing.md`, `README.md`, manuel (+ PDF, brochure relue), `DOCKER_START.md`, `CHANGELOG.md`).
Convention de décompte (celle de la fiche unique) : le seuil compte les **modules de code** au sens du
`CLAUDE.md`. Seuil (« plus de 5 ») non atteint. `admin_backup` **n'est pas** touché : ses deux variantes sont
converties à l'appel (AC 15 b), précisément pour ne pas ouvrir un module de plus.

### Références

- Fiche index `15-13-mariadb-et-sauvegarde.md` (découpage, recompte, Change Log des validations P1 à P3) ;
  fiche unique complète au commit **8a9bcd27**.
- Issues #552, #576 (absorbée en validation P1), #558 (cahier des charges hérité, trois commentaires).
- Rapports de validation (non versionnés) : `target/gate-logs/15-13-p{1,2,3}-{R,F}.md` ; prompts
  `15-13-validate-prompt-p{1,2,3}.md`.
- `_bmad-output/implementation-artifacts/15-11a-compose-transmet-la-configuration.md` — § *Angles morts*
  (`#552`), AC 12 (conventions du manuel : 76/70 caractères, `make fr`, contrôle aplati, `Overfull` des
  dix tableaux).
- `_bmad-output/implementation-artifacts/15-11b-lecture-unique-des-variables.md` — `env_nonempty`, test
  (L).
- `crates/kesh-api/tests/configuration_transmise.rs` — `MONTAGES` (doc `:196-198`, `const` `:199-207`), `controle_transmission`
  (`:443-533`), test `transmission` (`:1733`), `s_sources_de_montage` (`:1969`).
- CLAUDE.md : § *Test Locally First* (E2E au dernier commit de code, D7), § *Propagation post-patch*, §
  *Recompter ses propres comptes rendus*, § *Le prompt d'une passe doit NOMMER le manuel*, § *Inventorier
  les sites NON RÉSOLUS*, § *Issue Tracking Rule* (mots-clés sur la PR).

## Dev Agent Record

### Agent Model Used

Opus 5.5 (agent de développement, en autonomie), worktree `kesh-15-13b`, cible
`CARGO_TARGET_DIR=kesh-15-13b/target`, bases `kesh_1513b` et `kesh_e2e_1513b`, port E2E 3018.

### Debug Log References

Journaux sous `target/gate-logs/` du worktree (non versionnés) : `15-13b-t0-*` (mesures du T0,
rapatriement, PDF d'avant aplati, `.log` d'avant), `15-13b-mutations.txt` (banc), `15-13b-gate-backend.txt`,
`15-13b-gate-frontend.txt`, `15-13b-e2e.txt`, `15-13b-e2e-backend.log`, `15-13b-admin-apres-plat.txt`,
`15-13b-admin-apres.log`.

### Completion Notes List

- **Gates réellement exécutés, au dernier commit de code `50cada06`** (les commits suivants ne portent que
  documentation, PDF et comptes rendus) :
  - remise à zéro de **mes** bases (`DROP`/`CREATE` de `kesh_1513b` et `kesh_e2e_1513b`, 75 migrations,
    seed `scripts/seed-dev-db.sql`) après `wait-kesh.sh` ;
  - `scripts/test-fast.sh` (fmt + clippy `-D warnings` + nextest, `DATABASE_URL` sur `kesh_1513b`) :
    **2983 exécutés, 2983 passés, 4 ignorés**, rc 0 ;
  - frontend : `npm run check` 0 erreur (27 avertissements préexistants), `lint-i18n-ownership` PASS,
    `test:unit` **1095/1095** (112 fichiers), `build` vert ;
  - **E2E complet** (`npm run test:e2e`, 10,0 min, lancé à 06:09 UTC) avec la recette **modifiée** du
    `CLAUDE.md` (`KESH_ADMIN_BACKUP_DIR=target/kesh-backup`), secrets générés (`openssl rand`),
    `KESH_TEST_MODE=true` des deux côtés, `KESH_COOKIE_SECURE=false`, les quatre `KESH_SMTP_*`
    (`/health` → `smtpConfigured:true`), `KESH_INBOX_DIR`/`KESH_DOCUMENTS_DIR` du worktree : **245 passés,
    9 échecs, 19 ignorés**. Les 9, jugés fichier par fichier contre `docs/testing.md` § *Les échecs
    attendus* : 7 KF-029 (#97 — `mode-expert:26`, `:41`, `onboarding-path-b:65`, `:92`, `onboarding:57`,
    `:77`, `:150`) et 2 KF-045 (#421 — `invoices.spec.ts:415`, `:439`, run avant 12:00 UTC). **Aucun hors
    liste.** Backend arrêté par son PID.
  - **Import réel** sur ce backend après la suite (export puis import par l'API, admin du seed) : 200,
    `backupCreated:true` ; `target/kesh-backup` créé en **700**, sauvegarde `kesh-pre-import-…keshbackup`
    en **600**, aucun `.partial`. Dossier supprimé ensuite.
- **Total de départ des tests** (revue de code P1, A-5) : **2973** à `200f5e79`, **déduit** et non mesuré —
  2983 exécutés au commit `50cada06`, moins 10 attributs de test ajoutés et 0 retiré entre les deux bornes
  (`git diff 200f5e79..50cada06 -- crates | grep -cE '^\+.*(#\[test\]|#\[tokio::test\]|#\[sqlx::test)'`
  → 10 ; même motif en `^-` → 0). Aucun nextest n'a tourné à `200f5e79` dans ce worktree.
- **Remédiation de la revue de code P1** (Opus 5.5) : **une seule ligne de production touchée**, le texte
  du `warn!` de `write_backup_file` (continuation `\` rétablie, plus de suite d'espaces dans le message) ;
  **3 tests neufs** dans `mod tests` de `routes/admin.rs` (périmètre : de `7ff477ab` au commit de
  remédiation ; `#[test]` ×2, `#[tokio::test]` ×1, aucun retiré) — `aucun_blanc_parasite_dans_le_code`
  (garde lexicale, indépendante du texte : aucune ligne de code de production ne porte deux espaces
  consécutives après son indentation ; **rouge avant la correction**, vert après),
  `echecs_avant_sauvegarde_tous_convertis` (E5 : aucune `AdminFullImportFailed`/`AdminFullExportFailed`
  avant `write_pre_import_backup` dans `run_backup_and_restore`, autant de `?` que de conversions, montage
  (3, 1)), `write_pre_import_backup_dossiers_crees_et_existants` (E3 : parent intermédiaire **et** feuille
  en `0700` ; A-4 : un dossier existant `0755` reste `0755`) ; le test 20 et le neuf partagent
  `code_de_production()` (DRY, sans changement de comportement). **Mutations jouées** (fichier restauré par
  copie **et touché**, `cmp` vérifié) : begin → `AdminFullImportFailed` et verrou → `map_db_error` rougissent
  `echecs_avant_sauvegarde_tous_convertis` ; `builder.mode(0o755)` rougit le test 11 et le neuf ; un
  `create_dir_all` du parent avant le `DirBuilder` (parent en `0755`) rougit **le seul** neuf ; un
  `set_permissions(dir, 0700)` avant la création rougit le seul neuf. Doc-comments complétés :
  `write_backup_file` (angles morts annulation, durabilité du renommage, `try_exists` et liens
  symboliques), `AdminFullImportFailed`, `check_schema_compat`, `build_keshbackup` (conversion à l'appel
  par `avant_sauvegarde`, A-3). **Gate ciblé réellement exécuté**, après remise à zéro de **ma** base
  `kesh_1513b` (`DROP`/`CREATE`, 75 migrations, seed) et `wait-kesh.sh` : `cargo fmt --all -- --check`
  vert ; `cargo clippy --workspace --all-targets -- -D warnings` vert (relancé après les doc-comments) ;
  `cargo nextest run -p kesh-api -E 'test(/routes::admin::tests/) | binary(configuration_transmise) |
  binary(/^admin_/)'` : **90 exécutés, 90 passés** (`routes::admin::tests` 8, `configuration_transmise` 23,
  `admin_full_import_e2e` 33, `admin_pat_denied_e2e` 16, `admin_full_export_e2e` 7, `admin_backup_e2e` 3 ;
  journal `target/gate-logs/15-13b-review-p1-remediation-cible.txt`). **Gate complet et E2E : non rejoués**
  — à faire au dernier commit de code (D7).
- **Mutations (T8)** — banc `target/mut/mutations.py`, chaque mutation appliquée, test ciblé, fichier
  restauré par `git checkout` **et touché** : **28/28 rouges**, aucune par erreur de compilation. M11 rougit
  `transmission` **et** `fantomes` ; M15 `from_env_absent_backup_dir_takes_data_backup` **et** `transmission`
  (le test 6 a, qui compare à la constante, reste vert : attendu) ; M34 `catalogues_distinguent_…` **et**
  `kesh-i18n parity_between_locales`. Formes retenues là où la fiche décrit l'intention : **M30** retrait
  du bras de conversion (seul `autre => autre` reste) ; **M31** `autre =>
  AdminPreImportBackupFailed(format!("{autre}"))` ; **M36** variante rendue au refus d'écraser ; **M46**
  `partial = path` **et** vérification du nom final neutralisée (sans la seconde, l'appel échoue quand même
  sur le nom final — la mutation ne représenterait pas « écrire directement le nom final ») ; **M51**
  `remove_file(&partial)` ajouté dans l'erreur de `create_new`.
- **Manuel** : `make fr` (trois PDF), contrôles de l'AC 11 l sur le PDF aplati final — positifs :
  `/data/backup` ×8, `sudo cp backup/` ×1, `kesh-pre-import` ×3, `keshbackup.partial` ×1,
  `target: /data/backup` ×1, ligne `sudo chown` **entière** ×1 (non coupée), `base de données.{0,40}inaccessible`
  ×1 ; négatifs : `BACKUP_DIR.{0,90}/tmp` 0, `défaut /tmp` 0, `deux gestes` 0 — chacun rouge sur le PDF
  d'avant (T0). `Overfull \hbox` : **55**, liste identique à l'avant (trois débordements apparus à la
  première compilation corrigés, C-15-13b-4). `user-manual.pdf` et `marketing-brochure.pdf` régénérés
  **identiques à l'octet** (`.tex` inchangés) : non modifiés. **Brochure** (AC 11 n) : relue, laissée telle
  quelle (C-15-13b-4).
- **Contrôle de l'AC 11 f** : motif `target: /data/backup` confirmé au T0 (forme longue de
  `docker compose config`), rouge (0) sur un compose sans montage avec la variable posée.
- **Rapatriement (AC 11 e)** : non mesurable au T0 (démon Docker bloqué, C-15-13b-1), **mesuré à 07:48**
  après son retour, conteneurs `debian:bookworm-slim` jetables : Docker crée `./backup` en `root:root
  0755` ; le fichier `0600 root` est illisible de l'utilisateur (`cat`, `cp` → *Permission denied*) ;
  `cp` puis `chown <uid>` **en root** — joués par un conteneur root, `sudo` n'étant pas utilisable sans mot
  de passe depuis l'agent — rendent la copie lisible (`0600`, propriétaire l'utilisateur) ; copie et
  dossier supprimés. Le geste `sudo` lui-même, sur un NAS, reste à la recette de Guy.
- **Écarts à la fiche** : (1) `CLAUDE.md` — seule la ligne de commande change, pas la mention de date
  de l'AC 16 a (consigne de l'orchestrateur, C-15-13b-2) ; (2) CHANGELOG — la phrase « deux gestes »
  devient un renvoi sans nombre (option de l'AC 13 c, C-15-13b-3) ; (3) le nom du fichier est décrit
  au manuel, non écrit en entier (C-15-13b-4) ; (4) le test 17 exige en plus la négation dans le message.
- **Test 3** : le lien `DEFAULT_ADMIN_BACKUP_DIR` ↔ `MONTAGES` est porté par `controle_transmission`
  (donc par `transmission`).
- **Inventaire (AC 14)** : greps du T9 rejoués sur l'arbre final — (1) `BACKUP_DIR.{0,90}/tmp` ne rend
  que les trois recettes `/tmp/kesh-e2e/backup` de `docs/testing.md` (voulues, AC 16 b) ; le jeton des
  « deux gestes » restreint ne rend plus rien ; `pré-import|pre-import` ne rend que des sites de
  l'inventaire et la clé neuve ; (2) `cargo run -p kesh-api` : les sites de l'AC 16 ; (3) `git ls-files`
  sous les quatre dossiers à la racine : vide ; (4) par la valeur `\./(inbox|documents|log)\b` : les deux
  énumérations complètes (`.env.example:214`, `admin-manual.tex:812`) se poursuivent par `./backup` à la
  ligne suivante ; les autres sont des défauts de montages variables. Aucun site hors inventaire.
- **Issues** : la PR devra porter `closes #552` et `closes #576` (titre ou corps) ; `refs #558`.
- **Conflits certains** avec la 15-13a au merge : `CHANGELOG.md` (paragraphe « Action requise ») et
  `admin-manual.tex` (« Pour qui garde son fichier compose ») — § *Dépendances* ; le manuel compte ici
  « Trois gestes pour chacun des deux compose » (15-13b seule) : à recompter si la 15-13a passe avant.
- **Clôture (rebase sur la 15-13a, `bcded0c8`)** : conflits résolus comme annoncés, le décompte du manuel
  **recompté** — « Cinq gestes pour `docker-compose.yml`, trois pour `docker-compose.prod.yml` » (deux lignes
  d'`environment:` dans les deux fichiers, le port et les mots de passe MariaDB pour le premier seul, le
  montage `./backup` dans les deux) ; le CHANGELOG garde « les gestes, fichier par fichier » de la 15-13a,
  sans nombre. Gates rejoués sur l'état rebasé : voir le Change Log.

### File List

- `.dockerignore`, `.gitignore`, `.env.example`, `docker-compose.yml`, `docker-compose.prod.yml`
- `crates/kesh-api/src/config.rs`, `crates/kesh-api/src/routes/admin.rs`, `crates/kesh-api/src/errors.rs`
- `crates/kesh-api/tests/configuration_transmise.rs`, `crates/kesh-api/tests/admin_full_import_e2e.rs`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`
- `docs/manual/fr/admin-manual.tex`, `docs/manual/fr/admin-manual.pdf`
- `CHANGELOG.md`, `CLAUDE.md` (ligne de commande de la recette E2E), `DOCKER_START.md`, `README.md`,
  `docs/testing.md`
- `_bmad-output/implementation-artifacts/15-13b-sauvegarde-avant-import-persistante.md`,
  `_bmad-output/implementation-artifacts/sprint-status.yaml`,
  `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`

## Change Log

- 2026-10-09 — **Création par découpage de la 15-13, et remédiation de la validation P3** (agent remédiateur,
  Opus 5.5, en autonomie). Historique de la fiche unique (spécification, validations P1 et P2) : Change Log
  de la fiche index. **Trend** de la fiche unique : P1 (Opus 5.5 ×2) 8 MEDIUM distincts → P2 (Sonnet ×2) 6
  → P3 (Opus 5.5 ×2 : R 2 MEDIUM / 8 LOW, F 3 MEDIUM / 8 LOW, aucun doublon) **5 MEDIUM distincts** ; 0
  CRITICAL/HIGH aux trois passes. **Signal D5 de recyclage** levé deux passes de suite (P2 : deux MEDIUM nés
  de la P1 ; P3 : R3-1, R3-2 et F-P3-2 nés de la P2) → **découpage** décidé par l'orchestrateur
  (C-15-13-16) selon la couture #551 / #552-#576.
  - **Reçu de la 15-13** : AC 7, 8, 9, 10 c/d (montage)/e (« trois »), 11 d (ligne `KESH_ADMIN_BACKUP_DIR`,
    montages)/e/f (paragraphe sauvegarde)/i/k/l (contrôles sauvegarde), 12 a (ligne `./backup`, devenue
    12 a'), 13 b (#552, #576, Sécurité)/c (phrase `/tmp`), 14, 15, 16 ; T0 (rapatriement, `make fr`), T1
    (montages, exclusions, gabarit), T2 (`config`, `routes/admin`, `errors`, catalogues), T3, T5, T6
    (recettes, CHANGELOG), T8, T9, T10 ; tests 3, 5, 6, 11, 12, 14 à 19 ; mutations M9–M11, M14, M15,
    M24–M26, M30–M40 ; trois lignes du tableau des cas (deux scindées) ; huit angles morts.
  - **Remédiation P3 appliquée ici** : **R3-1** (AC 8 e : motifs **ancrés** `/inbox/`, `/documents/`,
    `/backup/`, tolérance nommée pour `log/` et sa justification ; test 19 qui l'exige ; **M41** ; homonyme
    `frontend/src/routes/(app)/admin/backup/` à l'inventaire et à C-15-13-14) ; **F-P3-1** (AC 11 f et
    13 c : « deux gestes » → décompte par fichier ; « sans la ligne → `./backup` » conditionné au compose
    0.13.0 ; contrôle négatif au PDF) ; LOW **R3-6/F-P3-7** (`post_restore.rs:591`,
    `post_restore_transactionality.rs:8`, `admin_full_import_e2e.rs:704` à l'inventaire), **R3-9** (test
    16 : interdits sans les parenthèses, revérifiés par `grep -cF` sur les quatre catalogues et les vingt
    couples de négations), **F-P3-5** (modes hors Docker : AC 11 d, `admin-manual.tex:194`,
    `.env.example:77-82`, `README.md:90` ; AC 16 étendu au grep `cargo run` sans `-p`, sites triés),
    **F-P3-10** (`CARGO_MANIFEST_DIR`), **F-P3-11** (brochure `:398`, AC 11 n).
  - **Recompte** (depuis cette fiche) : **10 AC** (numéros 7–16 de la fiche unique), **9 tâches** (T0–T3,
    T5, T6, T8–T10), **11 lignes de test** portant **12 fonctions Rust** (8 neuves, 4 existantes
    complétées) ; **20 mutations** (M9–M11, M14, M15, M24–M26, M30–M41) ; **3 lignes** au tableau des cas ; **36
    lignes** d'inventaire ; **8 angles morts** ; **4 modules de code**.
- 2026-10-09 — **Remédiation de la validation P4** (agent remédiateur, Opus 5.5, en autonomie). Passe P4 :
  deux lentilles **Sonnet** en contexte frais (prompt `15-13b-validate-prompt-p4.md`) — R (regression
  hunter) **0 CRITICAL / 0 HIGH / 2 MEDIUM / 8 LOW**, F (adversaire plein périmètre) **0 / 0 / 3 MEDIUM /
  5 LOW** ; R4-2 = F-P4-1 — soit **4 MEDIUM distincts**. **Trend** (fiche unique puis 15-13b) : P1 (Opus 5.5
  ×2) 8 MEDIUM distincts → P2 (Sonnet ×2) 6 → P3 (Opus 5.5 ×2) 5 → découpage → P4 (Sonnet ×2) **4** ;
  0 CRITICAL/HIGH aux quatre passes. **Signal D5** (sévérité égale, MEDIUM → MEDIUM) **déclaré au Project
  Lead via l'orchestrateur** : les quatre MEDIUM sont distincts de ceux de la P3 ; **trois** sont d'origine
  — R4-1 (constante liée à rien, `DEFAULT_ADMIN_BACKUP_DIR` depuis la spécification `d023c019`), F-P4-2 (M14
  sans test, depuis `d023c019`), F-P4-3 (trou d'inventaire) — mais **un naît de la remédiation P3** :
  la ligne de contrôle `docker compose config | grep -c '/data/backup'` de R4-2/F-P4-1 apparaît au commit
  `a847f369` (vérifié par `git log -S`), écrite pour F-P3-1. C'est la forme qui découpe selon
  l'amendement D5 ; elle est ici **isolée** (un défaut sur quatre, dans un texte du manuel, non dans le
  code) et la fiche est déjà issue d'un découpage, à quatre modules : pas de nouveau découpage proposé par
  le remédiateur — l'arbitrage revient à l'orchestrateur.
  - **MEDIUM** : **R4-1** (constante `pub`, liée au littéral par le test 6 b et à la cible de `MONTAGES`
    par le test 3 ; M15 réécrite « valeur de la constante », rouge à deux tests ; C-15-13-24) ;
    **R4-2/F-P4-1** (contrôle de l'AC 11 f sur `target: /data/backup` ; T0 : cas « variable posée,
    montage absent → 0 ») ; **F-P4-2** (fonction pure `source_conforme`, exercée par (S) avec `./backups`
    refusé ; M14 désormais rouge ; C-15-13-24) ; **F-P4-3** (`.env.example:208-210`, `:241-242`,
    `CHANGELOG.md:44`, `admin-manual.tex:812` à l'inventaire et aux AC 8 c, 11 d, 13 c, tâches T1, T6 ;
    grep (4) du T9 **par la valeur** `\./(inbox|documents|log)\b` rejoué à `c702b7d5`, chaque site trié).
  - **Décision de l'orchestrateur** (F-P4-6, C-15-13-21) : écriture sous `.partial` (`0600`, même dossier),
    `sync_all`, puis `rename` ; refus d'écraser par vérification du nom final ; test
    `write_backup_file_passe_par_un_partiel` et **M46** ; manuel : un `.partial` peut être supprimé ;
    angles morts « `.partial` non supprimé », « fenêtre vérification → renommage », « durabilité du
    renommage ».
  - **LOW** : **F-P4-4** (négation exigée aux tests 15 et 16, **M45**) ; **F-P4-5** (texte qui ne
    privilégie pas le dossier : deux pistes à égalité ou aucune, C-15-13-25) ; **F-P4-7** (`:1643` reçoit
    une phrase d'exception ; `:1712` et `website/roadmap.html:231` assumés) ; **F-P4-8** (« depuis le
    répertoire du compose », `lstlisting` à 76 caractères pour la ligne `sudo chown` de 74 ; copie de la
    recette hors dépôt : l'orchestrateur réalignera sa mémoire) ; **R4-3** (`--no-index` au T1) ; **R4-4**
    (jeton `deux gestes` restreint) ; **R4-5** (test 19 : « au moins quatre ») ; **R4-6** (= F-P4-2) ;
    **R4-7** (« comme avant » au manuel, `/tmp` au seul CHANGELOG) ; **R4-8** (valeur absolue
    `/tmp/kesh-e2e/backup` pour les trois recettes de `docs/testing.md`) ; **R4-9** (dossier témoin d'umask
    au test 11) ; **R4-10** (références `:196-198`/`:199-207`, `ls-files` rejoué à `c702b7d5`,
    supersession de « cinq modules » de C-15-13-9 dans C-15-13-26).
  - **Règles changées** : (1) **l'écriture de la sauvegarde** passe par `.partial` + renommage (AC 9 a) —
    le refus d'écraser change de mécanisme (vérification du nom final, M26 réécrite) ; (2) **le texte de
    #576** n'oriente plus vers le seul dossier (AC 15 c) ; (3) **le défaut `/data/backup`** est désormais
    lié par test au littéral et au montage (AC 7 a). Les deux premières touchent la production : une passe
    P5 **complète** est recommandée, non une passe ciblée.
  - **Recompte** (depuis cette fiche, `grep`) : **10 AC**, **9 tâches**, **11 lignes de test** portant
    **13 fonctions Rust** (9 neuves, 4 existantes complétées) ; **22 mutations** (M9–M11, M14, M15,
    M24–M26, M30–M41, M45, M46 ; `sort -u` → 22) ; **3 lignes** au tableau des cas ; **41 lignes**
    d'inventaire ; **10 angles morts** ; **4 modules de code** (inchangé : `config`, `routes/admin`,
    `errors`, catalogues `kesh-i18n`). Union avec la 15-13a : 46 mutations, M1 à M46 sans trou ni doublon.
- 2026-10-09 — **Remédiation de la validation P5** (agent remédiateur, Opus 5.5, en autonomie). Passe P5 :
  deux lentilles **Opus 5.5** en contexte frais (prompt `15-13b-validate-prompt-p5.md`) — R (regression
  hunter) **0 CRITICAL / 0 HIGH / 1 MEDIUM / 6 LOW**, F (adversaire plein périmètre) **0 / 0 / 1 MEDIUM /
  7 LOW** ; aucun doublon MEDIUM — soit **2 MEDIUM distincts** ; LOW en partie communs (R5-3 = F-P5-8).
  **Trend** (fiche unique puis 15-13b) : P1 (Opus 5.5 ×2) 8 MEDIUM distincts → P2 (Sonnet ×2) 6 → P3
  (Opus 5.5 ×2) 5 → découpage → P4 (Sonnet ×2) 4 → P5 (Opus 5.5 ×2) **2** ; 0 CRITICAL/HIGH aux cinq passes.
  **Signal D5** (sévérité égale, MEDIUM → MEDIUM), déclaré au Project Lead via l'orchestrateur :
  **R5-1 naît de la remédiation P4** (« ou n'en nomme aucune », ajouté par `3ebedca9` pour F-P4-5) — forme
  de recyclage ; **F-P5-1** est d'origine (introduction d'`avant_sauvegarde`, remédiation P1 `07e168e1`).
  Trois LOW naissent aussi de la P4 (R5-2, R5-3/F-P5-8, R5-4) et un de C-15-13-21 (F-P5-3). Décroissance
  monotone, un défaut recyclé sur deux : pas de découpage proposé par le remédiateur — l'arbitrage revient
  à l'orchestrateur.
  - **MEDIUM** : **R5-1** (décision de l'orchestrateur, C-15-13-27) — le message d'échec antérieur à la
    sauvegarde nomme **toujours** les deux pistes, dossier de sauvegarde et base de données, dans les quatre
    locales et le repli Rust, par des jetons fixés à la lettre ; tests 15 et 16 complétés, **M49**, **M50** ;
    angle mort « contrôle au démarrage » réécrit (le message nomme les deux causes) ; rectifie C-15-13-25
    (« ou aucune » retiré) et le motif de C-15-13-9. **F-P5-1** (décision de l'orchestrateur, tranchée par
    le remédiateur, C-15-13-28) — un garde lexical **est** retenu : test 20
    `avant_sauvegarde_branchee_aux_appels_de_l_import` (lecture de `admin.rs`, appels de l'import convertis,
    appel de l'export `:39` non converti, assertion de montage sur le nombre d'appels) ; forme d'appel
    prescrite à l'AC 15 b ; **M47**, **M48** couvertes ; la panne réelle derrière `:172`/`:259` reste à
    l'angle mort et à la phrase de clôture du tableau des tests.
  - **LOW** : **R5-2** (sept sites antérieurs, cinq de base et deux de dossier, AC 15 a et c) ;
    **R5-3/F-P5-8** (AC 9 a-c : un détail par étape ; `try_exists` en `Err` = échec ; C-15-13-29) ;
    **R5-4** (AC 11 d : aucune incise ne donne d'exemple sous `/tmp`) ; **R5-5** (AC 13 c : « deux gestes »
    **partagée**, ventilation 1 + 3 + 1 partagée = 5 ; la fiche 15-13a, close, n'est pas retouchée) ;
    **R5-6** (AC 10 c : message « doit être exactement `./backup` » pour une source exacte) ; **R5-7** (AC 7 b :
    doc-comment du test 6 a ; boucle `configuration_transmise.rs:509-521`, message `Y` `:512-514`, revérifiés) ;
    **F-P5-2** (trois homonymes du jeton `backup/` à l'inventaire, ligne de l'homonyme) ; **F-P5-3** (AC 9 a
    et test 12 : l'échec de `create_new` ne supprime rien, le `.partial` préexistant garde son contenu,
    **M51** ; C-15-13-29) ; **F-P5-4** (AC 12 a' : puce à la fin de § *Notes*, `DOCKER_START.md:140` ;
    inventaire) ; **F-P5-5** (AC 8 e : `./backup` créé par Docker au premier `docker compose up`, rempli au
    premier import) ; **F-P5-6** (AC 11 e : bloc `lstlisting` en ASCII, `<...>` — ligne `sudo chown`
    remesurée à **76**, la limite ; contrôle aplati positif de la ligne entière à l'AC 11 l) ; **F-P5-7**
    (AC 11 e : l'entrée de la liste des refus nomme les deux pistes ; contrôle aplati positif à l'AC 11 l).
    Les deux contrôles aplatis neufs vérifiés absents de `target/gate-logs/15-13b-p5-F-admin-plat.txt`
    (`grep -cF` → 0 chacun).
  - **Grep du symptôme par la valeur** (fiche) : `ou aucune`, `n'en nomme aucune`, `cinq (sites|échecs)`,
    `<…>`, `\b74\b`, `509-519`, `premier import`, `dit la cause`, `section « dossiers »` — hors Change Log, ne
    restent que des textes neufs et justes : « rempli au premier import » (AC 8 e), « et non `<…>` »
    (AC 11 e), « n'a pas de section « dossiers » » (AC 12 a', inventaire).
  - **Règles changées** : (1) **le texte de #576** nomme obligatoirement les deux pistes (test 15/16, M49,
    M50) — l'option « aucune » disparaît ; (2) **l'appel d'`avant_sauvegarde`** a une forme prescrite, gardée
    lexicalement (test 20) ; (3) **le nettoyage du `.partial`** ne porte que sur celui créé par l'appel, et
    `try_exists` en erreur vaut échec. Les trois touchent la production : une passe P6 **complète** reste
    justifiée, ou, si l'orchestrateur juge la convergence acquise, une passe ciblée sur ces trois points.
  - **Recompte** (depuis cette fiche, `grep`) : **10 AC**, **9 tâches**, **12 lignes de test** portant
    **14 fonctions Rust** (10 neuves, 4 existantes complétées) ; **27 mutations** (M9–M11, M14, M15,
    M24–M26, M30–M41, M45–M51 ; `sort -u` → 27) ; **3 lignes** au tableau des cas ; **41 lignes**
    d'inventaire (inchangé : lignes enrichies) ; **10 angles morts** (inchangé) ; **4 modules de code**
    (inchangé). Union avec la 15-13a (24 mutations) : **51**, M1 à M51 sans trou ni doublon.

- **2026-10-09 — validation P6 (Sonnet ×2, prompt `15-13b-validate-prompt-p6.md`)** : R 0 au-dessus de LOW
  (4 LOW), F 0 au-dessus de LOW (8 LOW) ; axes exercés et non exercés déclarés par les deux lentilles.
  **Validation close.** Trend (15-13 puis 15-13b) : P1 8 MEDIUM → P2 6 → P3 5 (découpage, C-15-13-16) → P4 4
  → P5 2 → P6 0. Modèles : Opus ×2, Sonnet ×2, Opus ×2, Sonnet ×2, Opus ×2, Sonnet ×2. Les 12 LOW de la P6
  (`target/gate-logs/15-13b-p6-{R,F}.md`) sont **à appliquer au T0 du développement** : R6-1 (justification
  « aucun `lstlisting` hors ASCII » fausse : dix listings en portent), R6-2 (forme de l'AC 15 b sur plusieurs
  lignes après rustfmt), R6-3 = F-P6-6 (`DirBuilder` récursif en `0700` applique le mode aux parents créés :
  l'écrire), R6-4 (pas de `serde_json` dans `export.rs`), F-P6-1 (conflit certain avec la 15-13a sur
  `CHANGELOG.md:46` et `admin-manual.tex:1793` : à écrire aux Dépendances), F-P6-2 (M52 : branche négative du
  test 20), F-P6-3, F-P6-4 (`README.md:48`), F-P6-5, F-P6-7, F-P6-8 (« dossier » contre « répertoire »).

- **2026-10-09 — T0 du développement** (agent de développement, Opus 5.5, en autonomie ; worktree `kesh-15-13b`,
  base `200f5e79` + `dd9c3b1f`).
  - **Les 12 LOW de la P6 appliqués** : R6-1 (AC 11 e, motif ASCII « par prudence »), R6-2 (AC 15 b, rustfmt),
    R6-3 = F-P6-6 (AC 9 b et angle mort « parents créés en `0700` »), R6-4 (AC 15 c, ZIP et non `serde_json`),
    F-P6-1 (§ *Dépendances* : conflits certains sur le paragraphe « Action requise » du CHANGELOG et sur
    « Pour qui garde son fichier compose »), F-P6-2 (**M52**, branche négative du test 20 ; **28 mutations**,
    union avec la 15-13a : 52, M52 absente de la 15-13a — vérifié), F-P6-3 (AC 9 a), F-P6-4 (`README.md:48` à
    l'inventaire, ligne assumée — **42 lignes d'inventaire inchangées**, ligne enrichie), F-P6-5 (AC 11 e :
    forme coupée), F-P6-7 (M35, M37 désignées par le texte ; chemin et lignes `:81-84` d'`AdminRestorePanel`),
    F-P6-8 (AC 11 e : « dossier de sauvegarde » au manuel). Angles morts : **11**.
  - **Relecture contre `200f5e79`** (la 15-7a2, la 15-12a et la 15-6a sont mergées depuis la spécification) :
    **aucun écart de fond**, seulement des numéros de ligne décalés. `routes/admin.rs`, `config.rs`,
    `configuration_transmise.rs`, `admin_full_import_e2e.rs`, les compose, `.env.example`, `docs/testing.md`,
    `README.md`, `DOCKER_START.md`, `.gitignore`, `.dockerignore`, `CLAUDE.md` : **inchangés** (`git diff --stat
    de285ea8 200f5e79` vide sur eux) — tous les numéros de la fiche y valent. Décalés : `errors.rs` +13
    (`AdminFullExportFailed` `:459`, `AdminFullImportFailed` `:467`, doc `:461-466`, bras `:1874-1884`) ;
    catalogues +5 (`error-admin-full-import-failed` `fr-CH:1483`, `de/en/it-CH:1412`) ; `admin-manual.tex` +3
    (`:1646` ouverture de la section, `:1796` « Pour qui garde… ») ; `CHANGELOG.md` +10 (`:54` entrée #550
    « Inchangé », `:56` paragraphe « Action requise »). Les AC citent la ligne de la spécification ; le
    développement se repère au texte.
  - **Mesures** (`target/gate-logs/15-13b-t0-*`) : (1) **graphie** de `docker compose config` (client seul,
    sans démon) pour `- ./backup:/data/backup` : forme longue `type: bind` / `source: …/backup` /
    `target: /data/backup` / `bind: create_host_path: true`, **identique** pour les deux compose ;
    `grep -c 'target: /data/backup'` → **1** avec le montage ; sur les compose **sans** montage avec
    `KESH_ADMIN_BACKUP_DIR=/data/backup` dans `.env` → **0** (alors que `grep -c '/data/backup'` → 1, la ligne
    d'environnement) : le contrôle de l'AC 11 f est discriminant, motif inchangé. (2) **`make fr` d'avant** :
    `.log` de référence (55 lignes `Overfull`), PDF aplatis ; contrôles de l'AC 11 l sur le PDF d'avant :
    positifs tous à **0** (`/data/backup`, `sudo cp backup/`, `kesh-pre-import`, `keshbackup.partial`,
    `target: /data/backup`, ligne `sudo chown` entière, `base de données.{0,40}inaccessible`) ; négatifs
    présents (`défaut /tmp` ×2, `deux gestes` ×1, `BACKUP_DIR.{0,90}/tmp` ×3) — tous discriminants. PDF
    versionnés restaurés (`git checkout`). (3) **Rapatriement sur conteneur jetable** : **non mesuré au T0** —
    le démon Docker de la station est bloqué au noyau (tâches `dockerd` en état D sur un rw-semaphore depuis
    07:06, `docker run` expire à 60 s), et le relancer toucherait `kesh-mariadb-dev` (interdit). Reporté au
    T10, choix C-15-13b-1.
  - **Registre** : `15-13` (`split`) et `15-13b` (`in-progress`) ajoutées à `sprint-status.yaml`.
- **2026-10-09 — Développement** (Opus 5.5, en autonomie) : T1 à T10 faits ; commits `fcd711e4` (T1),
  `50cada06` (T2-T3, dernier commit de code), `969c1ee2` (T5-T6). Gates au dernier commit de code :
  backend 2983/2983, frontend 1095/1095, E2E 245 passés / 9 échecs attendus (7 KF-029, 2 KF-045) ;
  28/28 mutations rouges ; PDF contrôlé aplati, 55 `Overfull` inchangés. Choix C-15-13b-1 à 4. Statut
  `review`.
- **2026-10-09 — Revue de code P1** (Sonnet ×3 : lentilles B, E, A ; rapports
  `target/gate-logs/15-13b-review-p1-{B,E,A}.md`) : **0 CRITICAL, 0 HIGH, 0 MEDIUM, 15 LOW** (B 4, E 6,
  A 5 ; B1 = E2 = A-1, B2 = E1, B3 = E4 se recouvrent). Remédiation (Opus 5.5) : **une seule ligne de
  production touchée, le texte du `warn!`** de `write_backup_file` (B1 = E2 = A-1) ; 3 tests neufs (garde du
  blanc parasite, E5, E3 + A-4) ; doc-comments (B2 = E1, B3 = E4, E6, A-3) ; angles morts de la fiche
  (B2 = E1, B3 = E4, B4, E6 ; **12** désormais, +1 : `try_exists` et liens symboliques) ; total de départ
  des tests au record (A-5). **A-2** laissé en l'état : écart assumé et consigné (C-15-13b-2). Gate ciblé
  vert (fmt, clippy, 90/90 sur les binaires d'admin et `configuration_transmise`) ; gate complet et E2E au
  dernier commit de code.
- **2026-10-09 — Revue de code P2 ciblée** (Haiku, une lentille, prompt versionné
  `15-13b-review-prompt-p2-ciblee.md`, commit du prompt `a534db0a` avant rebase, `21741f70` après) : selon le
  journal de l'orchestrateur, **0 finding au-dessus de LOW**. ⚠️ **Le rapport lui-même est perdu** : il était
  écrit sous `target/gate-logs/15-13b-review-p2-ciblee.md`, effacé par un `cargo clean` après un crash ; ni
  le détail des LOW éventuels ni la liste des axes exercés et non exercés ne peuvent être relus. Ce bilan
  n'est donc adossé qu'au journal de l'orchestrateur, non au rapport. Aucun patch n'a suivi.
  **Revue de code close.** Trend : P1 (Sonnet ×3, lentilles B, E, A) 0 CRITICAL / 0 HIGH / 0 MEDIUM / 15 LOW
  → remédiation `7b1db902` (une ligne de production, le texte d'un `warn!` ; trois tests) → P2 ciblée (Haiku)
  0 au-dessus de LOW. Aucun reclassement.
- **2026-10-09 — Clôture et intégration** (Opus 5.5, en autonomie). Branche de sauvegarde
  `backup/15-13b-avant-rebase-bcded0c8` (tête d'avant : `a534db0a`), puis **rebase sur `origin/main` =
  `bcded0c8`** (15-6b, 15-12b, 15-13a mergées depuis la base `200f5e79`).
  - **Conflits** : fiches 15-13 et 15-13b (ajout/ajout — `main` portait la version d'avant la validation P5,
    reçue de la branche de la 15-13a ; version de la branche retenue, aucune ligne propre à `main` perdue,
    vérifié au diff) ; registre des choix et `sprint-status.yaml` **par union** (C-15-13-27 à -29 et
    C-15-13b-1 à -4 à côté des C-15-13a-1 à -4 ; entrées 15-12b, 15-13, 15-13a de `main`, entrée 15-13b de la
    branche) ; `CHANGELOG.md` (paragraphe « Action requise » de `main` — refus de Compose, mots de passe
    MariaDB — avec la phrase de la sauvegarde pré-import de la 15-13b à la place de « reste dans `/tmp` » ;
    rubrique Sécurité : l'entrée « dossiers montés » de la 15-13b suivie des entrées #557 et #551 de `main`) ;
    `DOCKER_START.md` (volume `kesh-mariadb-data` de `main` et ligne `./backup`) ; `admin-manual.tex` (les
    deux paragraphes « La sauvegarde de sécurité pré-import » et « Exercices clôturés dans le désordre » ;
    **décompte recompté** : cinq gestes pour `docker-compose.yml`, trois pour `docker-compose.prod.yml` ; le
    montage `./backup` en dernier geste, après le port et les mots de passe MariaDB ; la phrase « Puis
    `docker compose config -q` » réunit le contrôle `ports` de la 15-13a et le compte de `target:
    /data/backup` de la 15-13b, son `lstlisting` placé avant le « Contrôle de fin de mise à jour »).
    Fusionnés sans conflit et relus : les deux compose (port MariaDB fermé, mots de passe `:?`, **et**
    `./backup:/data/backup`), `.env.example`, `CLAUDE.md`, `README.md`, `docs/testing.md`. Choix C-15-13b-5.
  - **PDF** : `admin-manual.pdf` régénéré (`make -B admin`) sur le `.tex` fusionné ; 55 `Overfull`, aucun
    dans les deux zones fusionnées ; contrôles aplatis : `/data/backup` ×8, `sudo cp backup/` ×1,
    `kesh-pre-import` ×3, `keshbackup.partial` ×1, `target: /data/backup` ×1, ligne `sudo chown` entière ×1,
    `Cinq gestes` ×1, `Exercices clôturés dans le désordre` ×2, `Changer un mot de passe MariaDB` ×2 ;
    négatifs `défaut /tmp`, `deux gestes`, `Trois gestes`, `Quatre gestes` : 0. Manuel utilisateur et
    brochure non touchés par la story : PDF de `main` conservés.
  - **Gates sur l'état rebasé** (cibles cargo nettoyées, compilation **à froid**,
    `CARGO_TARGET_DIR=kesh-15-13b/target` ; bases `kesh_1513b` et `kesh_e2e_1513b` recréées, 75 migrations,
    seed sur `kesh_1513b`) : `scripts/test-fast.sh` — fmt vert, clippy `-D warnings` vert, nextest **3046
    exécutés, 3046 passés, 4 ignorés** ; frontend — `check` 0 erreur (27 avertissements préexistants),
    `lint-i18n-ownership` PASS, `test:unit` **1139/1139** (114 fichiers), `build` vert ; **E2E complet**
    (port 3013, recette de `docs/testing.md` avec `KESH_ADMIN_BACKUP_DIR`, `smtpConfigured:true`, lancé vers
    09:24 UTC) : **245 passés, 9 échecs, 19 ignorés** — les 9 jugés fichier par fichier contre
    `docs/testing.md` § *Les échecs attendus* : 7 KF-029 (#97 : `mode-expert:26`, `:41`,
    `onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`, `:150`) et 2 KF-045 (#421 : `invoices.spec.ts:415`,
    `:439`, run avant 12:00 UTC) ; aucun hors liste. Import réel par l'API sur ce backend après la suite :
    200, `backupCreated:true`, sauvegarde `kesh-pre-import-…keshbackup` en `0600`. Journaux sous
    `/home/gcorbaz/devel/kesh-gate-logs/15-13b-cloture-*`. Statut **done**.
- **2026-10-09 — Second rebase, sur `245b91ee` (15-7b1)** (Opus 5.5, en autonomie). Branche de sauvegarde
  `backup/15-13b-avant-rebase-245b91ee` (tête d'avant : `b2cf4ee7`, celle de la PR #584), puis rebase des
  11 commits sur `origin/main` = `245b91ee`.
  - **Conflits** : registre des choix (C-15-7b1-1 à -3 de `main`, puis C-15-13b-1 de la branche — union, aucun
    doublon de numéro) ; `admin-manual.pdf` **régénéré** (`make -B admin`, 83 pages) sur le `.tex` fusionné,
    jamais fusionné ; `sprint-status.yaml` par union, ligne `last_updated` de la 15-13b renumérotée (42) après
    la (41) de la 15-7b1. `CHANGELOG.md` et `admin-manual.tex` fusionnés sans conflit et relus. Contrôle de
    perte : les 973 lignes ajoutées par `245b91ee` confrontées aux lignes retirées par la branche — deux lignes
    génériques communes (`.await`, `})?;`), sans perte, `git range-diff` donnant les commits de code
    identiques. PDF aplati : `105 des 112` ×1 (texte de `main`), `104 des 112` ×0, `/data/backup` ×8,
    `kesh-pre-import` ×3, `keshbackup.partial` ×1, `sudo cp backup/` ×1. Manuel utilisateur touché par `main`
    seul : son PDF conservé.
  - **Gates sur l'état rebasé** (`CARGO_TARGET_DIR=kesh-15-13b/target` ; `kesh_1513b` et `kesh_e2e_1513b`
    recréées, 75 migrations, seed sur `kesh_1513b`) : `scripts/test-fast.sh` — fmt vert, clippy `-D warnings`
    vert, nextest **3057 exécutés, 3057 passés, 4 ignorés** ; frontend — `check` 0 erreur (27 avertissements
    préexistants), `lint-i18n-ownership` PASS, `test:unit` **1139/1139** (114 fichiers), `build` vert.
    **E2E complet** — un premier lancement a échoué au `globalSetup` (500, `1114 The table 'credit_note_lines'
    is full` : tmpfs de MariaDB plein, `ibdata1` à 3,5 Go, sans rapport avec la branche) ; après redémarrage de
    MariaDB par l'orchestrateur et reconstruction des deux bases, relancé (port 3013, recette de
    `docs/testing.md` avec `KESH_ADMIN_BACKUP_DIR` inscriptible, `smtpConfigured:true`, 11:11–11:21 UTC) :
    **245 passés, 9 échecs, 19 ignorés** — jugés fichier par fichier contre `docs/testing.md` § *Les échecs
    attendus* : 7 KF-029 (#97 : `mode-expert:26`, `:41`, `onboarding-path-b:65`, `:92`, `onboarding:57`,
    `:77`, `:150`) et 2 KF-045 (#421 : `invoices.spec.ts:415`, `:439`, run avant 12:00 UTC) ; aucun hors liste.
    Journaux : `/home/gcorbaz/devel/kesh-gate-logs/15-13b-rebase-245b91ee-*`. Statut **done** maintenu.
