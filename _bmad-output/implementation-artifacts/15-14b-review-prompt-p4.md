# Prompt — revue de code P4, Story 15-14b

*Versionné le 2026-10-09. Trois lentilles (Opus), contexte frais chacune, en lecture seule (bmad-code-review :
Blind Hunter, Edge Case Hunter, Acceptance Auditor). Rotation D6 : passes complètes Sonnet ↔ Opus ; Haiku en passe ciblée.*

Worktree `/home/gcorbaz/devel/kesh-15-14b`, branche `story/15-14b-exploitation-et-multi-societe`. **Le diff à revoir** :
`git -C /home/gcorbaz/devel/kesh-15-14b diff e892dcfa..d2d8afc2 -- . ':(exclude)_bmad-output'` (un seul diff aplati :
manuels `.tex` et PDF, brochure, README, site, `docker-compose.prod.yml` (une ligne de commentaire), compose de dev,
gardes G14–G18 et `crates/kesh-api/tests/common/manuel.rs`, CHANGELOG — les fiches `_bmad-output/` sont le contexte).
**Fiche** : `_bmad-output/implementation-artifacts/15-14b-exploitation-et-multi-societe.md` (AC, tâches, inventaires
par commande avec bloc `E`, recette, contrôle de cohérence garde par garde, Dev Agent Record). Issues (par
`gh api repos/guycorbaz/kesh/issues/N`) : #575, #554, #127. Registre : `epic-15-choix-autonomes.md` (C-15-14-1 à 61).
Règles : `CLAUDE.md`. Recette rejouée : `/home/gcorbaz/devel/kesh-gate-logs/15-14b-recette-sauvegarde.log` ; mutations :
`/home/gcorbaz/devel/kesh-gate-logs/15-14b-mutations.log`.

**Quatrième passe.** Trend : P1 1 HIGH / 6 MEDIUM → P2 1 HIGH / 3 MEDIUM (refonte en vrais scripts, C-15-14-68) → P3
0 HIGH / 4 MEDIUM (rapports `/home/gcorbaz/devel/kesh-gate-logs/15-14b-review-p{1,2,3}-{B,E,A}.md`), remédiée par
`34de2e91` (scripts, recette, manuel, PDF), `2012b0c8` (G16, dernier commit de code), `d87e4737`, `0a3d30e3`, `d2d8afc2` ;
branche rebasée sur `e892dcfa`. Recette : `/home/gcorbaz/devel/kesh-gate-logs/15-14b-recette-p3.log` (49 contrôles, dont
5-bis « dump de sécurité impossible, base présente », empreinte de toutes les tables, ligne non ASCII) ; mutations des
scripts contre la recette : 7/7. Choix C-15-14-73 à 76 (pas d'option `--sans-securite`, réglage `SAUVEGARDE_PROJET`, URL
non épinglée). **Relis d'abord les deux scripts ligne à ligne, dans leur version finale** : sonde de la base
(`information_schema.SCHEMATA`) — que se passe-t-il si la sonde elle-même échoue (réseau, droits) ? verrou contre deux
dumps simultanés (nature, nettoyage après interruption) ; `trap` ; ordre arrêt / dump de sécurité / rechargement /
redémarrage, et chaque sortie anticipée : Kesh est-il TOUJOURS redémarré, la base TOUJOURS intacte quand rien n'est
rechargé ? `-p` du projet compose ; `--default-character-set=utf8mb4`. La recette prouve-t-elle chacune de ces sorties ?
G16 (g) (réglages manuel ↔ scripts, défauts, horaires, absence de routines/vues dans les migrations) mord-elle ? Le manuel
(`.tex` et PDF aplati, commandes longues en listing) est-il juste et rejouable ?

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

Rapport complet dans `/home/gcorbaz/devel/kesh-gate-logs/15-14b-review-p4-<B|E|A>.md` : findings numérotés, sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` pour toute affirmation de présence ou
d'absence ; code cité relu), correction proposée ; ⛔ **axes exercés ET non exercés**. Dernier message : le chemin, le
bilan par sévérité, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans
une base, ni aucune commande qui compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`),
`make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`/`apply`, `sqlx`, `cargo`, `npm`, `npx`,
`gh issue create`/`comment`/`edit`, SQL d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`,
`gh api` en lecture, `pdftotext` vers la sortie standard.
