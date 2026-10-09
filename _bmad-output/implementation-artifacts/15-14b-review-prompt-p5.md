# Prompt — revue de code P5, Story 15-14b

*Versionné le 2026-10-09. Trois lentilles (Sonnet), contexte frais chacune, en lecture seule (bmad-code-review :
Blind Hunter, Edge Case Hunter, Acceptance Auditor). Rotation D6 : passes complètes Sonnet ↔ Opus ; Haiku en passe ciblée.*

Worktree `/home/gcorbaz/devel/kesh-15-14b`, branche `story/15-14b-exploitation-et-multi-societe`. **Le diff à revoir** :
`git -C /home/gcorbaz/devel/kesh-15-14b diff e892dcfa..1805c40a -- . ':(exclude)_bmad-output'` (un seul diff aplati :
manuels `.tex` et PDF, brochure, README, site, `docker-compose.prod.yml` (une ligne de commentaire), compose de dev,
gardes G14–G18 et `crates/kesh-api/tests/common/manuel.rs`, CHANGELOG — les fiches `_bmad-output/` sont le contexte).
**Fiche** : `_bmad-output/implementation-artifacts/15-14b-exploitation-et-multi-societe.md` (AC, tâches, inventaires
par commande avec bloc `E`, recette, contrôle de cohérence garde par garde, Dev Agent Record). Issues (par
`gh api repos/guycorbaz/kesh/issues/N`) : #575, #554, #127. Registre : `epic-15-choix-autonomes.md` (C-15-14-1 à 61).
Règles : `CLAUDE.md`. Recette rejouée : `/home/gcorbaz/devel/kesh-gate-logs/15-14b-recette-sauvegarde.log` ; mutations :
`/home/gcorbaz/devel/kesh-gate-logs/15-14b-mutations.log`.

**Cinquième passe.** Trend : P1 1 HIGH / 6 MEDIUM → P2 1 HIGH / 3 MEDIUM → P3 0 / 4 → P4 0 / 4 (signal D5 : recyclage
dans `kesh-restore.sh`). Décision de l'orchestrateur après la P4 : **simplification à la racine** (C-15-14-77 à 80),
remédiée par `5eccbae3` (scripts, recette, manuel, PDF), `bc60639d` (G16, dernier commit de code), `1805c40a`. Ce qui
change : `kesh-restore.sh` NE TOUCHE PLUS À KESH (refus si un conteneur du service `kesh-api` tourne, détecté par
label sans nom de projet ; ni stop ni up ; `SAUVEGARDE_PROJET` supprimé) ; nom de base lu dans le dump (un seul
`CREATE DATABASE`), refus s'il diffère de `SAUVEGARDE_BASE` ; verrou `dump/.verrou` tenu pendant toute la restauration ;
dump de sécurité obligatoire si la base existe, pris par `kesh-dump.sh` lui-même (`DUMP_CIBLE`, `DUMP_COMPTE`) ; base
illisible = geste manuel décrit au manuel ; deux gestes de reprise (« terminer » / « revenir ») ; signaux convertis en
sortie. L'oracle de la recette est désormais une empreinte du CONTENU (dump trié par clé primaire, haché), le
`CHECKSUM TABLE` de la P3 s'étant révélé non fiable. Recette : `/home/gcorbaz/devel/kesh-gate-logs/15-14b-recette-p4.log`
(61 contrôles). **Relis d'abord `kesh-restore.sh` et `kesh-dump.sh` dans leur version finale, ligne à ligne** : chaque
sortie (refus, échec, interruption) laisse-t-elle la base intacte quand rien n'est rechargé, le verrou libéré, aucun
fichier partiel ? la détection « Kesh en marche » peut-elle rater un conteneur (statut `restarting`, autre label) ? la
lecture du nom de base dans le dump est-elle robuste (dump sans `CREATE DATABASE`, plusieurs, nom avec caractères
spéciaux) ? l'appel de `kesh-dump.sh` par `kesh-restore.sh` partage-t-il correctement le verrou (pas d'auto-blocage) ?
la recette prouve-t-elle chaque sortie, et son oracle de contenu est-il sûr ? G16 mord-elle ? Le manuel (`.tex` ET PDF
aplati) décrit-il exactement l'ordre des gestes : arrêter `kesh-api`, lancer, redémarrer ?

Ce qui change : la section Synology du manuel d'administration (Hyper Backup ne voit pas la base ; pré-script de dump
à deux comptes — `kesh_backup` en `SELECT, LOCK TABLES` pour la sauvegarde, compte Kesh pour la restauration —,
fichiers `[client]`, `.tmp` puis `mv`, post-script qui garde le dump, recovery par Snapshot qui recharge le dump,
3-2-1) ; une installation = une société (manuels, brochure, README, site) ; connexion par identifiant ; compose de
développement ; G14–G18.

Axes : **la recette est-elle juste et sûre telle qu'écrite** (privilèges réels de MariaDB 10.11 pour `mariadb-dump`
avec les options du listing, `--add-drop-database` et `CREATE DATABASE` par le compte Kesh, mot de passe à caractères
spéciaux dans le fichier d'options, droits 600, arrêt de `kesh-api` pendant le rechargement, dump de la veille gardé si
le pré-script échoue, secrets dans `backup/`) ? Un administrateur qui la suit perd-il des données dans un cas (dump
vide, rotation, restauration d'un mauvais fichier) ? Chaque garde rougirait-elle sur la régression visée et resterait-
elle verte sur un texte juste ? L'angle mort déclaré (`unlink`, `find -delete`) en cache-t-il d'autres ? Les textes
multi-société sont-ils vrais au code (une installation, une société ; `companies` comme table) ?

## Lentilles

- **B — Blind Hunter** : le diff seul, sans la fiche. Une commande du manuel fausse ou dangereuse, un texte neuf qui
  affirme une chose fausse au code, une contradiction entre deux textes du diff, une garde qui passerait à vide ou
  rougirait à tort, du LaTeX cassé, code mort, duplication (DRY), doc-comments devenus faux.
- **E — Edge Case Hunter** : les bornes des gardes (normalisation des apostrophes, ligatures, macros LaTeX, césures
  des PDF, continuations `\` des chaînes Rust, clé ajoutée demain dans une seule locale) ; les quatre locales clé
  par clé ; et surtout les **sites NON modifiés qui devraient l'être** — rejoue les commandes d'inventaire et cherche
  hors de leur périmètre (`website/`, `docs/`, README, frontend, replis Rust) la même affirmation fausse.
- **A — Acceptance Auditor** : chaque AC de la fiche contre le code ET les tests (un AC sans test qui le prouve est un
  finding) ; le Dev Agent Record ne déclare-t-il que ce qui a tourné (chiffres recomptés :
  `grep -c '#\[sqlx::test\]\|#\[test\]\|#\[tokio::test\]'` aux deux bornes ; 27 mutations déclarées) ; le **manuel** (`docs/manual/fr/*.tex` **et PDF aplatis** : `pdftotext -nopgbrk f.pdf - | tr '\n' ' ' | tr -s ' '`,
  ligatures ﬀ/ﬁ/ﬂ normalisées), `docs/api-external.md`, CHANGELOG, i18n 4 locales.

## Ce que tu rends

Rapport complet dans `/home/gcorbaz/devel/kesh-gate-logs/15-14b-review-p5-<B|E|A>.md` : findings numérotés, sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` pour toute affirmation de présence ou
d'absence ; code cité relu), correction proposée ; ⛔ **axes exercés ET non exercés**. Dernier message : le chemin, le
bilan par sévérité, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans
une base, ni aucune commande qui compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`),
`make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`/`apply`, `sqlx`, `cargo`, `npm`, `npx`,
`gh issue create`/`comment`/`edit`, SQL d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`,
`gh api` en lecture, `pdftotext` vers la sortie standard.
