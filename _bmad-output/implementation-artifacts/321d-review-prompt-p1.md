# Prompt — revue de code P1 de la Story 32-1d (le serveur de la sauvegarde automatique), trois lentilles, Sonnet

Tu relis le développement de la clé **32-1d** de la Story 32-1 de Kesh (comptabilité suisse) : *le serveur de la
sauvegarde automatique* — deux constats LOW de **#711** : **L-B5**, moitié serveur (le drapeau `lastSuccessInFuture`
servi par `GET /api/v1/admin/backup-schedule`, calculé par `auto_backup::is_success_in_future`), et **L-B7** (le refus
400 d'un réglage hors domaine, `PUT /api/v1/admin/backup-schedule`, rendu dans la langue de l'instance par la clé
`admin-backup-schedule-error-out-of-domain`). Worktree `/home/gcorbaz/devel/kesh-32-1d`, branche `story/32-1d`, partie
de `main` à `5ea108ae`. Le diff à relire, **aplati** (un seul diff, pas la suite des commits) :
`git -C /home/gcorbaz/devel/kesh-32-1d diff 5ea108ae...HEAD`. Langue du rapport : français. Modèle : **Sonnet**.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/32-1-defauts-courts.md` — **partie D** (AC-d1 à AC-d6, tâches T-d0 à T-d3),
§ *Dev Notes* (*Pourquoi quatre clés*, *32-1d — remarques de conception*, *Inventaire des sites non résolus*, *Fichiers
touchés*, CHANGELOG), et la sous-section **32-1d** du Dev Agent Record — et le registre
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-321.md` (C-32-1-1 à C-32-1-39, et C-32-1-44 à C-32-1-48
du développement ; C-32-1-40 à 43 appartiennent à la branche non fusionnée de la 32-1c, absente ici : ce n'est pas un
trou). Le Dev Agent Record déclare des gates, des rouges et des mutations : **ne les prends pas pour argent comptant**,
recoupe-les avec les journaux `/home/gcorbaz/devel/kesh-gate-logs/321d-*.log` et la capture
`/home/gcorbaz/devel/kesh-gate-logs/321-captures/321d-*.png` (lecture seule ; une image se regarde avec l'outil de
lecture de fichiers). La 32-1d ne touche **ni le manuel ni le frontend** : l'axe « manuel et PDF » se réduit à vérifier
qu'aucune phrase du manuel ne décrit le refus (`grep -nF 'hors domaine' docs/manual/fr/*.tex`).

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans le scratchpad, ni ailleurs ; **aucune redirection** (`>`, `>>`, `tee`), pas même vers
  un fichier « jetable » : compare par des commandes enchaînées (pipe). Un fichier créé par erreur se **déclare** au
  rapport ; il ne se supprime pas.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`).
- **N'exécute rien qui écrit, compile ou exécute du code** : ni **`scripts/prepare-release.sh`**, ni
  `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`, ni aucun script de
  `kesh-gate-logs` (dont `321d-capture.mjs`) ou du scratchpad, ni `make`, ni `latexmk`, ni `cargo` (aucune
  sous-commande, `cargo fmt` compris), ni `npm`/`npx`/`node`, ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni `curl`,
  ni `python3`, ni `sed -i`, ni `perl`. Aucune commande `gh` qui écrit ; `gh issue view 711|710|725|727` est permis.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection),
  `cat`, `ls`, `find`, `wc`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation d'absence** (« le site X n'est pas corrigé », « le test Y manque ») s'appuie sur une sortie
  **`grep -nF`** copiée ; toute affirmation de présence, sur une sortie `grep -nF` ou `sed -n` copiée.

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention.

1. `crates/kesh-api/src/auto_backup.rs` : `is_success_in_future` — juste contre `passe()` et `is_overdue` (égalité à
   `now`, réussite absente, désactivée) ? L'implication « vrai ⇒ `is_overdue` vrai » tient-elle pour **toute**
   entrée, intervalle compris ? Les cinq tests du `mod tests` prouvent-ils ce que dit leur nom ?
2. `crates/kesh-api/src/routes/backup_schedule.rs` : le champ `last_success_in_future` (nom servi en camelCase,
   toujours présent, calculé avec le **même** `now` qu'`overdue` et `nextDueAt`) ; le refus par `t(…)` — le repli
   égale-t-il **à l'octet** la valeur `fr-CH` du catalogue (continuations `\` comprises) ? Un autre site du fichier
   rend-il encore un texte français en dur ?
3. `crates/kesh-i18n/locales/*/messages.ftl` : la clé neuve dans les quatre locales — traductions justes, graphie
   suisse (`ss`), bornes (6, 12, 24, 168 ; 1 à 60) égales à `ALLOWED_INTERVAL_HOURS`, `RETENTION_MIN`,
   `RETENTION_MAX` (`crates/kesh-db/src/repositories/backup_schedule.rs`) ; place dans le bloc.
4. `crates/kesh-api/tests/i18n_e2e.rs` : l'aide `catalogues()` (employée par `spawn_app_with_locale` et par le test
   neuf) ; le test `backup_schedule_out_of_domain_refusal_in_german` — prend-il le verrou `get_i18n_lock` **avant**
   `spawn_app_with_locale` ? passerait-il à vide (clé absente → clé brute ; locale non appliquée → français) ?
5. `crates/kesh-api/tests/admin_auto_backup_e2e.rs` : les assertions neuves de `prochaine_et_retard` ; l'assertion
   `overdue` d'origine conservée.
6. `crates/kesh-api/tests/textes_coherents.rs` : l'entrée neuve de la table de G9 (`TABLE: [(&str, Cles); 5]`).
7. `CHANGELOG.md` : le bloc `## [0.16.0] — Non publié` (forme que `scripts/prepare-release.sh` attend, l. 233-260 ; la
   lire, ne pas l'exécuter) et sa ligne sous `### Corrigé` — vraie contre le code ?

## Lentille E — Edge cases

1. `is_success_in_future` avec `last_success_at` à `now + 1 ms`, à `now`, à `now − 1 ms` ; sauvegarde désactivée avec
   réussite future ; réussite future **et** tentative future. `lastSuccessInFuture` vrai avec `overdue` faux est-il
   atteignable ?
2. La réponse sert-elle le drapeau aussi après `PUT` et `POST …/run` (même constructeur `BackupScheduleResponse::new`) ?
   Un client actuel (frontend, type `BackupSchedule` de `frontend/src/lib/features/admin-backup/`) casse-t-il sur un
   champ inconnu (lecture du code client, sans rien exécuter) ?
3. Le refus traduit : l'ordre des gardes (corps JSON invalide → 400 d'Axum avant la validation ; `version` périmée →
   409 après) ; `t()` avant `init_error_i18n` (repli) — le repli reste-t-il juste ? Le test i18n peut-il être pollué
   par un autre test qui réinitialise le bundle global (`init_error_i18n`) sans prendre le verrou, dans le **même
   binaire** ?
4. **Mutations** M-d1 (`is_success_in_future` toujours faux) et M-d2 (chaîne française rétablie) : chacune est-elle
   tuée par le test que la fiche nomme (journaux `321d-mut-M-d1.log`, `321d-mut-M-d2.log`) ? Une mutation plausible
   non jouée survivrait-elle — par exemple `passe(…).is_none()` remplacé par `last_success_at > now` sans garde
   `enabled`, ou une clé présente en `de-CH` mais égale au français ? Raisonne sur le code, n'exécute rien.
5. Inventaire de la classe « refus de validation en français seul » : la fiche attend que le décompte multiligne de
   `AppError::Validation(` à chaîne littérale baisse de **1**, le décompte formaté ne bougeant pas. Le Dev Agent Record
   le dit-il, chiffré aux deux bornes ?

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC-d1 à AC-d6, rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve :

- **AC-d1** : signature exacte `pub fn is_success_in_future(schedule: &BackupSchedule, now: NaiveDateTime) -> bool` ;
  réemploi de `passe()` ; cinq tests unitaires (futur, passé, absent, égal, désactivée) ; champ `lastSuccessInFuture`
  servi par `new` ; `prochaine_et_retard` faux/faux/vrai ; **rouge constaté** avant (journal
  `321d-rouge-AC-d1-vert-AC-d2.log` : `Null` contre `false`).
- **AC-d2** : `t("admin-backup-schedule-error-out-of-domain", …)` ; test `i18n_e2e.rs` (montage `DeCh`, connexion
  administrateur, `GET` de la version, `PUT` à 7 h, 400, `VALIDATION_ERROR`, égal au `de-CH`, différent du `fr-CH`) ;
  **rouge constaté** (journal `321d-rouge-AC-d2.log`) ; G9 à 5.
- **AC-d3** : `grep -c '^admin-backup-' crates/kesh-i18n/locales/*/messages.ftl` à la base (`git show
  5ea108ae:<f> | grep -c …`) et à `HEAD` : **+1** par locale ; parité (`321d-cible-i18n.log`) ; gardes frontend
  (journaux `321d-frontend-*.log`).
- **AC-d4** : capture du refus en allemand, ou la raison écrite au Dev Agent Record ; la capture montre-t-elle le
  message **du serveur** (le script `321d-capture.mjs` réécrit le corps du `PUT` et transmet au vrai backend — lis-le,
  ne l'exécute pas) ?
- **AC-d5** : chiffres du Dev Agent Record contre les journaux (`321d-gate-complet.log`, `321d-frontend-*.log`,
  `321d-e2e.log`) ; échecs E2E jugés fichier par fichier contre `docs/testing.md` § *Les échecs attendus* ; gate et
  E2E **au dernier commit de code** (compare la révision des journaux à `git log --oneline 5ea108ae..HEAD` : les
  commits postérieurs ne touchent que de la documentation).
- **AC-d6** : deux mutations, journaux, diff résiduel 0.
- **Décomptes** : « tests ajoutés » recomptés aux deux bornes (`git show 5ea108ae:<f> | grep -cE
  '#\[(sqlx::|tokio::)?test'` contre `HEAD`), périmètre déclaré.
- **Messages de commit** : `git log 5ea108ae..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?)
  #[0-9]'` doit ne rien rendre ; chaque commit porte `refs #711`.
- **Registre et statut** : la clé `32-1d-defauts-sauvegarde-serveur` de `sprint-status.yaml` est à `review` avec un
  commentaire daté ; le Dev Agent Record n'écrit que la sous-section 32-1d ; registre numéroté à partir de C-32-1-44.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/321d-review-p1-<B|E|A>.md` (selon ta lentille) :

- bilan en tête : CRITICAL / HIGH / MEDIUM / LOW ;
- findings numérotés `B1…`/`E1…`/`A1…`, chacun avec sévérité, fichier:ligne, **preuve copiée** (sortie `grep -nF`,
  `sed -n` ou `git diff`), correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** — un « 0 finding » sans cette liste ne compte
  pas comme passe ;
- rien d'autre n'est écrit nulle part.
