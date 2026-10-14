# Prompt — validation P3 de la spec, Story 29-4 (une saisie concurrente ne passe plus sur un compte qu'on clôt)

*Versionné le 2026-10-14. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) : la P1
était **Sonnet**, la P2 **Opus** ; la P3 est **Sonnet** pour les deux lentilles. Haiku reste réservé à une passe ciblée de
fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh-29-4`, branche `story/29-4` (base `16eeed21`, kickoff de l'Epic 29). Fiche :
`_bmad-output/implementation-artifacts/29-4-cloture-compte-concurrente.md` (une fiche, **deux** stories : 29-4a, AC1-AC5 ;
29-4b, AC6-AC8 ; AC9 et AC10 partagés ; **AC11 retiré en P2**). Registre de la story :
`_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-4.md` (C-29-4-1 à 22). Epic :
`_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` § 29-4. Issues #651 et **#669** (`gh api
repos/guycorbaz/kesh/issues/651`, `…/651/comments`, `…/issues/669`).

**Historique de la boucle.** P1 (Sonnet) : R 0/0/6/7, F 0/0/2/7. P2 (Opus) : R 0/1/3/6, F 0/1/3/7 — rapports
`/home/gcorbaz/devel/kesh-gate-logs/29-4-validate-p{1,2}-{R,F}.md`, à lire, **la P2 surtout**. La P2 a levé le **signal
D5** (un HIGH convergent, R1 = F1, né de la remédiation P1 : l'épinglage de session posé dans `create_pool`, fonction sans
appelant). Remédiation P2 : `git log --oneline 2fdcfb5b..HEAD`, puis `git diff 2fdcfb5b HEAD -- _bmad-output/` ; résumée
au Change Log (entrée « Validation P2 »). Décisions de l'orchestrateur appliquées en P2, **à évaluer pour leurs
conséquences, non à rediscuter dans leur principe** : (1) retrait de l'épinglage (AC11, `configurer_session`, T29a-8,
M29-5), question portée par #669 ; (2) la réouverture **est** rejouée ; (3) le `PATCH` du compte **est** rejoué ; (4) deux
variantes de la revérification — `closed_on` seul pour les annulations, `closed_on` **et** `archived` pour l'import (W22)
et le lot (W17).

Antécédent : fiche `28-4-cloture-compte-bancaire.md` et rapports `/home/gcorbaz/devel/kesh-gate-logs/28-4-validate-p{1,2,3,4,5}-{R,F}.md`.
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites NON
RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Règle de splitting préventif, amendement D5 ;
§ Pattern batch ; § Issue Tracking Rule) ; `docs/agents-orchestration.md` ; rétrospective `epic-28-retro-2026-10-14.md`
§ 4.2 (une simplification se relit sur les verrous **et** la naissance de la vue) et § 4.3.

## Axe obligatoire, commun aux deux lentilles — relire la remédiation P2

*La sévérité ne stagne pas, elle se déplace vers ce qu'on vient d'écrire* : en P2, quatre des six défauts distincts de
sévérité MEDIUM ou plus venaient de la remédiation P1. Chacun des gestes suivants se relit comme neuf, contre le code au
sol :

1. **Le retrait est-il complet ?** Greper la **valeur**, sur la fiche, le registre et `sprint-status.yaml` :
   `AC11`, `T29a-8`, `M29-5`, `T1-bis`, `configurer_session`, `create_pool`, `pool.rs`, `épingl`, `1193`,
   `after_connect`, `snapshot_isolation`. Chaque occurrence restante doit être soit un **relevé daté** (Change Log de
   création et de la P1, choix d'origine annotés, prompts P1 et P2), soit la note de retrait, soit l'angle mort renvoyé à
   #669. Une prescription vivante qui suppose encore l'épinglage — un test, une mutation, un doc-comment à écrire, une
   ligne de tâche, un texte de manuel « vrai grâce à l'épinglage » — est un défaut. L'angle mort écrit (sous MariaDB
   11.6+, `1020` → `500`, « comme les deux sites préexistants ») est-il exact au code (`routes/reconciliation.rs:2166-2170`,
   `bank_imports.rs:439-446`, `kesh-db/src/errors.rs`, `kesh-api/src/errors.rs`) ? Le texte que l'AC9 prescrit au manuel
   reste-t-il vrai pour le serveur que le dépôt suppose ? #669 porte-t-il bien ce que la fiche lui renvoie ?
2. **Les deux variantes (AC1, AC2, AC5, AC7, AC8)** : la symétrie avec `ENSURE_OPEN_SQL` / `ENSURE_OPEN_UNARCHIVED_SQL`
   (`bank_accounts.rs:750-754`, `:839`, `:871`) est-elle exacte, ordre « archivé avant clos » compris ? La conversion
   `NotFound` → `IllegalStateTransition` de W17 est-elle sûre (un `NotFound` peut-il naître d'autre chose qu'un archivé
   après l'`INSERT` à clé étrangère) ? W22 rend-il bien `404 BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` par `import_create_error`
   sans retouche ? Une annulation peut-elle, par un chemin quelconque, appeler la variante ? L'assertion (i bis) du
   registre est-elle décidable telle que décrite ? T29a-11, T29a-12, T29b-5 rougiraient-ils au sol pour la bonne raison,
   et sont-ils réalisables (archivage par `archive_for_company` sous la sentinelle, compte sans transaction ; règlement par
   virement puis archivage) ? M29-22 à M29-25 sont-elles tuées par les tests nommés ?
3. **Les deux enveloppes neuves (AC4)** : `reopen_once` et `patch_once` — qu'est-ce qui reste devant l'enveloppe, qu'est-ce
   qui passe dedans ; chaque 1213 atteint-il le prédicat (`retry.rs:29-31`, conversions `?` et `Err(e) =>
   AppError::Database(e)`) ; `rollback` explicite sur refus ; idempotence d'une nouvelle tentative (le `PATCH` relit
   `claims_for_target` à chaque tentative ; l'audit n'est écrit qu'une fois) ; T29a-9 (séquence de T29a-4 sur un compte
   clos) et T29a-10 (patron `victime()`, partagés de `company_invoice_settings` tenus par le `PATCH`, fermeture en `FOR
   UPDATE` sur cette ligne) sont-ils réalisables pas à pas ? Le point (vi) d'`audit_route_registry.rs` passe à **dix** :
   recompte les sept au sol, et vérifie que les trois routes sont bien `SansEcritureAuJournal` (l. 333, 339, 340).
4. **T29a-6 / T29a-7 réécrits (AC3)** : la sonde `INNODB_TRX` (`trx_mysql_thread_id`, `trx_state = 'LOCK WAIT'`) existe-t-elle
   sous MariaDB 10.11 (raisonné, ou documenté au dépôt) ? Le `CONNECTION_ID()` relevé par la tâche est-il celui de la
   connexion qui exécute `close_in_tx` ? L'étape (4 bis) de T29a-6 (une `pending` insérée par SQL après la
   revérification) peut-elle attendre, et sur quoi ? Le test rougit-il encore pour la bonne raison si P-d (2) est fausse ?
5. **Les textes réécrits** : § *Cycles par la file d'attente* (généralisé à tout demandeur d'exclusif), l'argument de
   poids (F11 de la P2), le résidu du cycle à quatre (« route de gestion à égalité de poids »), l'angle mort « PATCH »
   déclaré fermé, l'interblocage préexistant import ou lot / route de gestion (F5) ; table *Dehors* réécrite ; CHANGELOG
   élargi à l'archivage ; `api-external.md:555` ajouté à l'AC9 ; doc-comments `bank_accounts.rs:866-867` et
   `routes/bank_accounts.rs:904-907`.
6. **Décompte des modules et dérogation** : recompte les deux parties depuis la fiche (convention : code / doc-comments
   seuls / documents / tests). La 29-4a est-elle bien à **quatre** modules de code (les deux enveloppes neuves et la
   variante vont-elles dans des modules déjà comptés) ? La dérogation de la 29-4b tient-elle encore ? Le signal D5 est-il
   écrit fidèlement au Change Log (§ *Règle de splitting préventif*, amendement du 2026-10-06) ?
7. **Recomptes** : tests neufs de la partie a (onze annoncés : T29a-1 à T29a-12, T29a-8 retiré), de la partie b (T29b-1 à
   T29b-5), mutations (24 identifiants : M29-1 à M29-25, M29-5 retiré), choix (22), routes rejouées au point (vi) (dix),
   et chaque numéro de ligne cité par la remédiation P2 (`sed -n`).

## Axes prioritaires (à reprendre sur l'état actuel)

1. **Prémisses** (table *Prémisses*, P-a à P-f) : vraie au code (citer) ? vraie d'InnoDB / MariaDB 10.11 — **vérifiée en
   pratique** dans le dépôt (nommer le test) ou **raisonnée** ? Que deviendrait la conception si elle était fausse ? Une
   prémisse fausse qui porte une fermeture de fenêtre est au moins **HIGH**.
2. **Périmètre minimal** contre #651, phrase par phrase ; table *Dehors*. La fermeture de L5 (archivage) et les deux
   enveloppes neuves sont-elles bien mécaniques, ou ouvrent-elles un chemin que la story ne teste pas ?
3. **Inventaire clos des sites non résolus** — refais-le depuis le symptôme ; un site à fenêtre réelle omis, ou une
   démonstration « sans fenêtre » fausse (W30, W20, W1-W6, `accept_one_*`), est au moins **HIGH**.
4. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` § *Clore un compte bancaire*
   (`\label{sec:clore-compte-bancaire}`, puce l. 1748-1752) et toute autre mention de la fenêtre ou de l'archivage
   concurrent ; `docs/manual/fr/admin-manual.tex` (l. 108 « MariaDB 10.11+ », l. 2708 — que disent-elles maintenant que
   l'épinglage est retiré ?) ; `docs/api-external.md` l. 241 et 555 ; `CHANGELOG.md` (section `[0.13.3]`, qui ne doit
   **pas** être réécrite). Ce que l'AC9 prescrit d'écrire sera-t-il vrai au code prescrit ? Contrôle aussi les **PDF**,
   aplatis : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (et `admin-manual.pdf`).
5. **Propagation** : `grep -rn "651\|\bL1\b\|\bL5\b" crates docs CHANGELOG.md frontend/src website README.md` et
   `grep -rn "snapshot_isolation\|ER_CHECKREAD" crates docs` — chaque site prescrit par l'AC9 est-il listé, et chaque
   site non listé a-t-il une raison d'être laissé ?

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque AC testable et couvert par une tâche ; chaque AC par un test **qui rougirait au sol pour la bonne raison** ;
  chaque mutation tuée par le test nommé, ou déclarée équivalente avec sa raison ; chaque fait, chemin, numéro de ligne
  **revérifié** ; les textes prescrits (doc-comments, registre, manuels, `api-external.md`, CHANGELOG) confrontés au code
  prescrit ; les tests existants qui pourraient changer de sens (T-a31, T-a45, T-a48, `a_concurrent_close_waits_for_the_unreconciliation`,
  `annuler_un_rejet_pendant_une_cloture_refuse_par_w30`, `bank_accounts_closed_e2e.rs`, `rejeu_interblocage_e2e.rs`,
  `audit_route_registry.rs`, les tests existants du `PATCH` et de la réouverture — cherche-les —, `reconciliation_e2e.rs:7556`
  et `:8401`) ; l'axe obligatoire 1 (retrait complet) est **ton** axe premier.
- **F — Faisabilité, verrous et naissance de la vue (adversaire de la conception)** — ⛔ **la table des verrous et des
  cycles est obligatoire au rapport**, refaite **depuis le code**, chemin par chemin, pour W9, W17, W22, W24, W25, W28,
  W29, la clôture, **la réouverture (désormais rejouée)** et **le `PATCH` du compte (désormais rejoué)** : la suite des
  verrous au sol, **verrous implicites de clé étrangère compris** (`companies`, `bank_accounts`, `accounts`, `users`,
  `company_invoice_settings`, intervalles d'index uniques et non uniques), la position exacte prescrite pour la
  revérification, la **variante** appelée, et les trois volets du critère — (1) verrou **vraiment** déjà tenu (W17, W22)
  ou **vraiment** dernière acquisition (dans la fonction **et** dans la route appelante, W25 imbriqué dans W9 compris,
  borne de l'audit comprise) ; (2) **cherche le cycle** contre **tous** les détenteurs et demandeurs d'un exclusif sur la
  ligne du compte — extraction refaite, lectures verrouillantes **et** écritures implicites (`UPDATE`/`DELETE`, la
  restauration comprise) —, contre les cycles nommés par les P1-P3 de la 28-4, contre les cycles **par la file
  d'attente** (à trois et à quatre), et contre les cycles que la **lecture d'`archived`** pourrait changer (l'archivage
  comme demandeur d'exclusif, la sentinelle `companies`) ; pour chaque cycle : **tous** ses participants sont-ils rejoués
  (lis les enveloppes) ? Les deux **enveloppes neuves** changent-elles un cycle (un `PATCH` rejoué qui reprend ses
  partagés de réglages ; une réouverture rejouée qui se réinscrit dans la file) ? Le **résidu du cycle à quatre** (route
  de gestion à égalité de poids) et l'**argument de poids** (F11 de la P2) tiennent-ils ? (3) naissance de la vue
  inchangée sur chaque chemin, `PATCH` compris ? Les tests à plusieurs connexions (T29a-4, T29a-9, T29a-10, T29b-3) et
  ceux de l'autre ordre (T29a-6, T29a-7, avec la sonde `INNODB_TRX`) sont-ils réalisables tels que décrits ; la victime
  est-elle celle annoncée ? Une piste écartée à tort qui serait moins risquée est au moins **MEDIUM**.

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve**
(commande exécutée et sortie copiée), **origine** (conception d'origine de la fiche, remédiation P1, **remédiation P2**,
prémisse de l'issue, ou héritage de la 28-4) — l'origine « remédiation P2 » se dit explicitement, c'est la mesure de la
boucle —, correction proposée. Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée ; toute
affirmation sur InnoDB dit si elle est raisonnée ou vérifiée, et où. ⛔ **La liste des axes exercés ET non exercés** —
un « 0 finding » sans elle ne compte pas comme passe ; l'axe obligatoire (1 à 7 ci-dessus) y figure point par point. Pour
la lentille F, la **table des verrous et des cycles**, chemin par chemin, figure au rapport. Le **bilan par sévérité** se
recompte depuis tes propres findings avant d'être écrit (le rapport F de la P2 annonçait 7 LOW et en numérotait 8).
Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-4-validate-p3-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/29-4-validate-p3-F.md` (lentille F) ; dernier message : le chemin du rapport, le
bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ Lecture seule stricte. N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/` (ton rapport, et au besoin un
aplatissement de PDF nommé `29-4-validate-p3-<R|F>-*.txt`). Aucune commande qui écrit, compile ou exécute : `scripts/*`
(dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `scripts/regen-test-schema.sh`),
`make`, `latexmk`, `xelatex`, `cargo`, `npm`, `npx`, `node`, **`python3`, `python`, `perl`, `ruby` et tout autre
interpréteur**, `docker`, **`mariadb`, `mysql` et toute requête SQL** (même en lecture, même sur la base de dev),
`bash` sur un script du dépôt, `sed -i`, `gh` en écriture (`issue`, `pr`, `comment`…). ⛔ **Aucun `git checkout`,
`git switch`, `git checkout --detach`, `git worktree`, `git stash`, `git reset`, `git rebase`, `git commit`, `git add`,
`git fetch`, `git pull`, `git tag`, `git push`, sous aucune forme** — pour lire une autre révision, `git show
<rév>:<chemin>`. Rien de `~/kesh-test/` (relevé réel de Guy) ne se lit ni ne se cite ; aucune donnée réelle dans le
rapport. Autorisés : lecture de fichiers, `grep`, `sed -n`, `awk` en lecture, `git log` / `show` / `diff` / `blame`,
`gh api` en lecture, `pdftotext` vers la sortie standard ou vers `kesh-gate-logs/`.
