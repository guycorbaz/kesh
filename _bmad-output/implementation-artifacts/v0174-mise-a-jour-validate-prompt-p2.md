# Validation P2 — fiches v0174-mise-a-jour (Opus, lentille complète, lecture seule)

Tu es la lentille de validation P2 des fiches BMAD **v0174-mise-a-jour** de Kesh (release 0.17.4, issue #847 : Kesh écrit
lui-même une sauvegarde complète avant d'appliquer des migrations). Modèle : **Opus**. Contexte frais. Lentille
**complète** (prémisses, faisabilité, fermeture). Travaille en français soigné.

La validation P1 (Sonnet, deux lentilles) a rendu R 0/2/8/7 et F 0/0/6/6 ; sa remédiation (commit `8d177fc3`) a
**découpé** la fiche et **ajouté** des choix (C-v0174m-15 à 24). La sévérité se déplace vers ce qu'on vient d'écrire :
relis en priorité ce que la remédiation a introduit (`git -C /home/gcorbaz/devel/kesh-0174m diff 3aba70dc 8d177fc3`),
sans négliger le reste.

## À lire, en entier

1. Les trois fiches, worktree `/home/gcorbaz/devel/kesh-0174m`, `_bmad-output/implementation-artifacts/` :
   `v0174-mise-a-jour.md` (parente, `split` : analyse du code, conception commune, découpage),
   `v0174-mise-a-jour-1.md` (la sauvegarde avant migration), `v0174-mise-a-jour-2.md` (l'écran).
2. Le registre `v0174-choix-autonomes-v0174m.md` (C-v0174m-1 à 24, amendements compris).
3. Les rapports de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/v0174m-validate-p1-R.md` et `-F.md` — pour juger si
   chaque finding est réellement fermé (le Change Log de `v0174-mise-a-jour-1.md` dit le traitement de chacun).
4. Le mandat : `/home/gcorbaz/devel/kesh-gate-logs/prompts/v0174-mise-a-jour-fiche.md`,
   `/home/gcorbaz/devel/kesh-gate-logs/prompts/v0174-commun.md` ; l'issue : `gh issue view 847 --json title,body,comments`
   (lecture seule).
5. `/home/gcorbaz/devel/kesh-0174m/CLAUDE.md` : *Review Iteration Rule*, *Règle de splitting préventif*, *Migration
   breaking policy* (P1, P6, P7, P8), *Issue Tracking Rule*, *Inventorier les sites non résolus*.
6. Le code, au commit `7bd4d5c0` (par `Read`, `git grep`, `git show 7bd4d5c0:…`) : `crates/kesh-api/src/main.rs` ;
   `crates/kesh-db/src/{version,lib,backup,snapshot}.rs` ; `crates/kesh-api/src/admin_backup/{export,import,manifest}.rs` ;
   `crates/kesh-api/src/auto_backup.rs` ; `crates/kesh-api/src/routes/{admin,backup_schedule}.rs` ;
   `crates/kesh-api/src/{config,document_storage,lib}.rs` ; `crates/kesh-api/tests/{configuration_transmise.rs,
   admin_auto_backup_e2e.rs,common/binaire.rs}` ; `crates/kesh-db/tests/{common/mod.rs,test_schema_guard.rs}` ;
   `frontend/src/lib/features/admin-backup/` ; `crates/kesh-i18n/locales/*/messages.ftl` ; `Dockerfile` ;
   `docker-compose.yml` ; `docker-compose.prod.yml` ; `.env.example` ; `scripts/synology/kesh-restore.sh` ;
   `crates/kesh-db/migrations/` ; les tags `v0.15.0`, `v0.17.2`, `v0.17.3` (`git show v0.15.0:…`, `git ls-tree`).
   `sqlx` : `~/.cargo/registry/src/index.crates.io-*/sqlx-core-0.8.6/src/migrate/migrator.rs` et
   `sqlx-mysql-0.8.6/src/migrate.rs`.
7. **Le manuel** : `docs/manual/fr/admin-manual.tex` (§ *Sauvegarde et restauration* entier, § *Mise à jour de Kesh*
   entier, § *Variables d'environnement complètes*, lignes 812, 2005-2010, 2282, 2329-2333, 2377, 2442) **et son PDF**
   `docs/manual/fr/admin-manual.pdf`, aplati avant tout grep :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -o "…"`. `CHANGELOG.md` (tête),
   `README.md:50`, `docs/testing.md` (§ *Base de dev jetable*).

⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : utilise `/usr/bin/grep` ou `git grep`.

## Ce que tu vérifies (axes)

- **A1 — Prémisses au code.** Chaque affirmation du § *Ce que le code dit* de la parente et des rappels des sous-fiches
  (lignes, comportements de `sqlx`, `'0.1.0'` initial, locales relatives, topologie de production, précédent
  `kesh-restore.sh`) est-elle vraie à `7bd4d5c0` ? Copie la ligne lue pour toute affirmation contestée ou critique.
- **A2 — Inventaire et importabilité (C-v0174m-1, -17, -24).** Le prédicat aligné sur la garde est-il juste ?
  L'import à inventaire paramétré (`parse_and_verify_avec_inventaire`, `restore_tables_in_tx_avec_inventaire`) est-il
  faisable sans changer le comportement de production (ordre enfants → parents, `onboarding_state`, `audit_log` fusionnée,
  `check_schema_compat`, `verifier_coherence`, rejeu `post_restore`) ? T1-ter est-il réellement discriminant et son
  oracle indépendant ? La garde de format T18 est-elle décidable (clés du manifeste, types de `decode_cell`) ?
- **A3 — Plan et tentatives répétées (C-v0174m-3, -15, -16).** Le scénario de la boucle de redémarrage est-il fermé :
  la plus ancienne du couple survit-elle à toutes les séquences (échec avant sauvegarde, après sauvegarde, migration
  « sale », retour à la version de départ entre deux tentatives, horloge qui recule) ? Le message R-b dit-il vrai dans
  chaque cas ? Le plan illisible est-il refusé sans faux refus (1146 seul vaut *neuve*) ?
- **A4 — Ordre des étapes et échappatoire (C-v0174m-4, -8, -21).** Échappatoire avant la version de départ : cohérent avec
  le manuel ? La voie `docker compose run --rm -e … kesh-api` fonctionne-t-elle réellement avec les deux compose (ports,
  volumes, réseau `frontend` externe du compose de production, `restart`, conteneur `up -d` déjà présent) ? La variable
  passe-t-elle les familles L, T, V, E, F de `configuration_transmise` ?
- **A5 — Échecs.** Chaque `Refus` a-t-il son test et sa mutation ? T16 et T19 (pool fermé) sont-ils montables et
  discriminants (une erreur de pool fermé n'est-elle pas confondue avec 1146) ? Un échec reste-t-il sans refus ?
- **A6 — Rétention, nom, liste (C-v0174m-5, -6, -15 ; sous-fiche 2).** `NOM_PRE_UPGRADE` (bornes 40, réduction,
  `inconnue`), la notion de « couple » par suffixe, la généralisation d'`appliquer_la_retention` sans changer celle de
  `auto/` (`NOM_AUTO`, `proteges`, dernière complète) ; la liste et le téléchargement du type neuf (lien, `O_NOFOLLOW`,
  404, clés d'API).
- **A7 — Retour en arrière (C-v0174m-10, -19, -22).** La procédure du § *Manuel*, point 3, aboutit-elle dans les deux
  topologies, par le compte Kesh, sans `docker compose exec` dans les sections Synology (G14 à G16) ? Le choix « plus
  récente ou plus ancienne du couple » est-il exploitable par un administrateur ? Les deux jambes de recette sont-elles
  jouables sur la station (images publiées `gcorbaz/kesh:0.15.0`, `0.17.2` ; une 0.15.0 accepte-t-elle de monter jusqu'à la
  branche ; son assistant a-t-il « Restaurer une sauvegarde ») et la topologie de production reproductible
  (`--memory 1g`, MariaDB autonome, compte Kesh à `ALL PRIVILEGES`) ?
- **A8 — Tests.** Montage commun (`UPDATE _kesh_version`, assertion de montage du nombre de migrations) ; inclusion
  `#[path]` des helpers de `kesh-db` depuis `kesh-api` (dépendances, `test_schema_guard`, forme de l'attribut) ; T8 et
  T8-bis (environnement complet de `Config::from_env`, `KESH_LOCALES_DIR`, port occupé) ; T6 (fichier épars de 600 Mio,
  seuil exact) ; T9/T9-bis (horloge injectable ?) ; T17 (fonctions pures de messages : la conception les rend-elle
  réellement pures ?).
- **A9 — Mutations.** M1 à M23 (et M11, M11-bis, M12 de la 2) : chacune tuée par le test nommé ? Une mutation importante
  manque-t-elle ?
- **A10 — Manuel, CHANGELOG, doc.** Le § *Manuel* (points 1 à 10) corrige-t-il toutes les phrases du `.tex` **et du PDF**
  que la story rend fausses, et seulement celles-là ? Les jetons du point 9 rendent-ils réellement quelque chose dans le
  PDF à `7bd4d5c0` (exécute le `pdftotext` aplati) ? Le texte proposé est-il exact au regard du code prescrit ? Le
  CHANGELOG est-il vrai (la 0.17.4 sans migration : vérifie `kesh-0174s/_bmad-output/implementation-artifacts/v0174-session.md`,
  C-v0174s-2, en lecture) ? `README.md`, `docs/testing.md`, `website/` ?
- **A11 — Découpage et périmètre.** Les deux sous-fiches satisfont-elles la *Règle de splitting* (modules comptés) ? La 1
  est-elle livrable seule (et son manuel juste sans la 2) ? « Aucune migration » ; #848 hors du geste ; l'issue demandée
  (mémoire) justifiée ?
- **A12 — Registre et cohérence.** C-v0174m-1 à 24 : contexte, retenu, écartées, réversibilité ; amendements cohérents
  entre eux et avec les trois fiches (noms, variable, dossier, nombre gardé, messages, numéros de test et de mutation) ;
  aucune décision sans choix ; décomptes (findings P1 : 29) recomptés.

## Sévérités

CRITICAL (la sauvegarde ne serait pas restaurable par la version de départ, une migration pourrait passer sans
sauvegarde ni refus, ou des données seraient perdues), HIGH (AC faux ou intestable, refus manquant, procédure de retour
qui n'aboutit pas), MEDIUM (prémisse fausse, inventaire incomplet, test qui ne tuerait pas sa mutation, manuel faux ou
incomplet, incohérence entre fiches), LOW (rédaction).

## Interdits — lecture seule stricte

Aucune commande qui écrit, compile ou exécute du code du dépôt : pas de `cargo`, `npm`, `npx`, `make`, `latexmk`,
`docker`, `curl`, `sqlx`, `mariadb` ; **pas de `scripts/prepare-release.sh`** ni d'aucun script de `scripts/` ; pas de
`git checkout` ni `git switch` (sous aucune forme), `git reset`, `git stash`, `git commit`, `git push`, `git tag`,
`git rebase`, `git worktree` ; aucune commande `gh` qui écrit (`gh issue view` en lecture seulement) ; pas de `claude`,
`pkill`, `killall`, `rm` ; pas de `sed -i`, `perl -pi`, `tee`, ni de redirection `>` vers le dépôt ou vers `/tmp`.
`pdftotext … -` vers la sortie standard est permis. **Tu n'écris qu'un seul fichier** : ton rapport. N'ouvre ni
`~/kesh/`, ni `~/kesh-test/`.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/v0174m-validate-p2.md` (chemin absolu), en français :

1. En-tête : modèle, date, commit lu (`git -C /home/gcorbaz/devel/kesh-0174m rev-parse HEAD`).
2. Décompte `CRITICAL/HIGH/MEDIUM/LOW`, recompté depuis ta liste ; dis combien viennent de la remédiation P1.
3. Findings numérotés `V2-n` : sévérité, axe, fichier et citation, preuve au code (ligne lue copiée : sortie
   `git grep -nF` ou extrait `Read`), correctif proposé.
4. **Findings P1 non fermés** : ceux dont le traitement déclaré ne ferme pas le défaut (numéro P1, pourquoi).
5. **Axes exercés et axes non exercés** (A1 à A12, un par un, avec ce que tu as réellement lu ou greppé, manuel et PDF
   compris). Un « 0 finding » sans cette liste ne compte pas comme passe.
