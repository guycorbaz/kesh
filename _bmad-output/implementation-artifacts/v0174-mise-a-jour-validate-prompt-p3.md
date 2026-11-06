# Validation P3 — fiches v0174-mise-a-jour (Sonnet, lentille complète, braquée d'abord sur la remédiation P2, lecture seule)

Tu es la lentille de validation P3 des fiches BMAD **v0174-mise-a-jour** de Kesh (release 0.17.4, issue #847 : Kesh écrit
lui-même une sauvegarde complète avant d'appliquer des migrations). Modèle : **Sonnet**. Contexte frais. Travaille en
français soigné.

**Contexte de la boucle.** P1 (Sonnet) : R 0/2/8/7, F 0/0/6/6. P2 (Opus) : 0/1/10/9, dont 16 findings sur 20 venus de la
remédiation P1. La boucle recyclait (la rétention a produit un HIGH à chaque passe ; l'échappatoire par
`docker compose run` un MEDIUM à chaque passe) ; l'orchestrateur a décidé de **simplifier** : plus aucune suppression
automatique des sauvegardes d'avant mise à jour (C-v0174m-25), une seule voie pour l'échappatoire (C-v0174m-28). La
remédiation est le commit `1be2d297`.

**Ordre de lecture imposé** : d'abord ce que la remédiation P2 a écrit ou retiré
(`git -C /home/gcorbaz/devel/kesh-0174m diff 878c03ef 1be2d297`), en cherchant ce que le retrait a laissé derrière lui
(références orphelines, tests ou mutations qui n'ont plus d'objet, messages qui supposent encore une rétention, manuel qui
le dit) ; ensuite le reste des fiches.

## À lire, en entier

1. Les fiches, worktree `/home/gcorbaz/devel/kesh-0174m`, `_bmad-output/implementation-artifacts/` :
   `v0174-mise-a-jour.md` (parente, `split`), `v0174-mise-a-jour-1.md` (sauvegarde avant migration),
   `v0174-mise-a-jour-2.md` (écran).
2. Le registre `v0174-choix-autonomes-v0174m.md` (C-v0174m-1 à 29 ; C-6, C-15, C-21 retirés).
3. Les rapports précédents : `/home/gcorbaz/devel/kesh-gate-logs/v0174m-validate-p1-R.md`, `-p1-F.md`,
   `/home/gcorbaz/devel/kesh-gate-logs/v0174m-validate-p2.md` — pour juger si chaque finding est réellement fermé (Change
   Log de `v0174-mise-a-jour-1.md`).
4. Le mandat : `/home/gcorbaz/devel/kesh-gate-logs/prompts/v0174-mise-a-jour-fiche.md`,
   `/home/gcorbaz/devel/kesh-gate-logs/prompts/v0174-commun.md` ; `gh issue view 847 --json title,body,comments`,
   `gh issue view 911 --json title,body` (lecture seule).
5. `/home/gcorbaz/devel/kesh-0174m/CLAUDE.md` : *Review Iteration Rule* (dont *La passe ciblée*), *Règle de splitting
   préventif*, *Migration breaking policy*, *Issue Tracking Rule*, *Inventorier les sites non résolus*.
6. Le code, au commit `7bd4d5c0` (par `Read`, `git grep`, `git show 7bd4d5c0:…`) : `crates/kesh-api/src/{main,lib,config,
   auto_backup}.rs`, `crates/kesh-api/src/admin_backup/{export,import,manifest}.rs`,
   `crates/kesh-api/src/routes/{admin,backup_schedule}.rs`, `crates/kesh-db/src/{version,backup,snapshot}.rs`,
   `crates/kesh-api/tests/{configuration_transmise.rs,admin_auto_backup_e2e.rs,common/binaire.rs,common/manuel.rs}`,
   `crates/kesh-db/tests/{common/mod.rs,test_schema_guard.rs}`, `docker-compose.yml`, `docker-compose.prod.yml`,
   `.env.example`, `scripts/synology/kesh-restore.sh`, `frontend/src/lib/features/admin-backup/`.
7. **Le manuel** : `docs/manual/fr/admin-manual.tex` (§ *Sauvegarde et restauration*, § *Mise à jour de Kesh*, § *Variables
   d'environnement complètes*, `sec:backup-dsm`, lignes 812, 870-871, 1525-1526, 2282, 2377, 2442, 2930, 2976) **et son PDF**
   `docs/manual/fr/admin-manual.pdf`, aplati avant tout grep :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -oE "…" | wc -l` — rejoue les
   comptes « avant » du § *Manuel*, point 9, de la sous-fiche 1 (sonde positive : un jeton qui rend 0 avant ne contrôle
   rien). `CHANGELOG.md` (tête), `README.md:50`, `docs/testing.md`.

⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : utilise `/usr/bin/grep` ou `git grep`.

## Ce que tu vérifies (axes)

- **A1 — Ce que le retrait a laissé.** Toute trace de rétention, de « couple » protégé, de compteur de tentatives, de
  `docker compose run`, de `VIDE_SIGNIFIANT`, de M8/M8-bis/M18/M22/T9-bis dans les trois fiches, le registre (hors
  historique daté) et les textes prescrits (messages, manuel, CHANGELOG). Les tests et mutations restants ont-ils encore
  un objet ? M27 est-elle tuée par T9 ?
- **A2 — La règle RG et les messages.** La règle « plus ancienne sauvegarde de `{dir}` écrite après le dernier démarrage
  réussi de la version de départ » est-elle **vraie** dans toutes les séquences de V2-1 (couple qui change, version de
  départ qui a servi entre deux tentatives, tentative sous échappatoire, migration « sale ») ? Un administrateur peut-il
  l'appliquer (où lit-il la date du dernier démarrage réussi : journal, `last_boot_at`, nom des fichiers en UTC) ? R-a,
  R-b, R-c, E1, W6 disent-ils vrai dans chaque cas (`Refus::Plan` sans `{de}` ni échappatoire, version illisible) ? E1
  est-il implémentable tel que prescrit dans `main.rs` (l'issue de la sauvegarde disponible à l'échec de `MIGRATOR.run`) ?
- **A3 — Accumulation.** Sans suppression, la boucle de redémarrage (`restart: unless-stopped`) peut-elle remplir le disque
  avant qu'un humain intervienne ? Le seuil d'espace (qui compte la plus grande sauvegarde de `auto/` et `pre-upgrade/`)
  arrête-t-il la boucle avant le disque plein, et que dit alors le refus ? Le manuel le dit-il ?
- **A4 — Échappatoire, une seule voie.** `SANS_DEFAUT` : la variable satisfait-elle réellement les familles L, T, V, E, F de
  `configuration_transmise.rs` (forme `${NOM:-}`, lignes commentées de `.env.example`, contrôle de la raison au
  doc-comment) ? Le message et le manuel pour un compose antérieur sont-ils exacts et suffisants (`docker compose up -d`
  après ajout de la ligne) ?
- **A5 — Visibilités et montages.** `pub mod admin_backup` : effets (lint `unreachable_pub`, doc, gardes lexicales qui
  supposent `pub(crate)`) ? T1-ter, T3, T16 (second cas), T19, T20 (avertissements portés par l'issue) et T6-quater sont-ils
  montables ? T18 (gabarit d'octets, `TYPES_DECODES`, clés à tous les niveaux) est-il décidable ?
- **A6 — Retour en arrière.** La commande du compose standard (`sh -c` dans le conteneur, quoting des backticks et des
  guillemets) s'exécute-t-elle telle qu'écrite ? Le renvoi Synology dans `sec:backup-dsm` (C-v0174m-29) tombe-t-il sous
  G14 à G16 sans les faire rougir (verbes interdits, `MARIADB_ROOT_PASSWORD`, dump de sécurité avant rechargement) ?
- **A7 — Manuel et CHANGELOG.** Les points 1 à 10 du § *Manuel* de la 1 couvrent-ils toutes les phrases que la story rend
  fausses, `.tex` et PDF ; aucune phrase n'annonce-t-elle une rétention ? Les comptes « avant » du point 9 se retrouvent-ils
  (rejoue-les) ? Le CHANGELOG est-il vrai ?
- **A8 — Les findings P1 et P2 sont-ils fermés ?** Un par un, contre le Change Log de la 1.
- **A9 — Registre et cohérence.** C-25 à C-29 complets (contexte, retenu, écartées, réversibilité) ; les choix retirés
  marqués comme tels ; numéros de choix, de tests et de mutations cohérents entre les quatre fichiers ; décomptes
  (P2 : 20 findings, 16 issus de la remédiation P1) recomptés.
- **A10 — Découpage.** Les modules de chaque sous-fiche (tableau de la parente) ; la 1 livrable seule, son manuel juste
  sans la 2.

## Sévérités

CRITICAL (la sauvegarde ne serait pas restaurable par la version de départ, une migration pourrait passer sans
sauvegarde ni refus, ou des données seraient perdues), HIGH (AC faux ou intestable, refus manquant, procédure de retour
qui n'aboutit pas, règle de choix du fichier fausse), MEDIUM (prémisse fausse, inventaire incomplet, test qui ne tuerait
pas sa mutation, manuel faux ou incomplet, incohérence entre fiches), LOW (rédaction).

## Interdits — lecture seule stricte

Aucune commande qui écrit, compile ou exécute du code du dépôt : pas de `cargo`, `npm`, `npx`, `make`, `latexmk`,
`docker`, `curl`, `sqlx`, `mariadb` ; **pas de `scripts/prepare-release.sh`** ni d'aucun script de `scripts/` ; pas de
`git checkout` ni `git switch` (sous aucune forme), `git reset`, `git stash`, `git commit`, `git push`, `git tag`,
`git rebase`, `git worktree` ; aucune commande `gh` qui écrit (`gh issue view` en lecture seulement) ; pas de `claude`,
`pkill`, `killall`, `rm` ; pas de `sed -i`, `perl -pi`, `tee`, ni de redirection `>` vers le dépôt ou vers `/tmp`.
`pdftotext … -` vers la sortie standard est permis. **Tu n'écris qu'un seul fichier** : ton rapport. N'ouvre ni
`~/kesh/`, ni `~/kesh-test/`.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/v0174m-validate-p3.md` (chemin absolu), en français :

1. En-tête : modèle, date, commit lu (`git -C /home/gcorbaz/devel/kesh-0174m rev-parse HEAD`).
2. Décompte `CRITICAL/HIGH/MEDIUM/LOW`, recompté depuis ta liste ; dis combien viennent de la remédiation P2.
3. Findings numérotés `V3-n` : sévérité, axe, fichier et citation, preuve au code (ligne lue copiée : sortie
   `git grep -nF` ou extrait `Read`), correctif proposé.
4. **Findings P1 et P2 non fermés** (numéro, pourquoi).
5. **Axes exercés et axes non exercés** (A1 à A10, un par un, avec ce que tu as réellement lu ou greppé, manuel et PDF
   compris). Un « 0 finding » sans cette liste ne compte pas comme passe.
