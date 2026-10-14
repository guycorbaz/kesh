# Prompt — validation P2 de la spec, Story 29-4 (une saisie concurrente ne passe plus sur un compte qu'on clôt)

*Versionné le 2026-10-14. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) : la P1 était
**Sonnet** ; la P2 est **Opus** pour les deux lentilles — le fichier d'epic exige pour cette story une validation
complète Opus **et** Sonnet. Haiku reste réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh-29-4`, branche `story/29-4` (base `16eeed21`, kickoff de l'Epic 29). Fiche :
`_bmad-output/implementation-artifacts/29-4-cloture-compte-concurrente.md` (une fiche, **deux** stories : 29-4a, AC1-AC5,
AC11 ; 29-4b, AC6-AC8 ; AC9 et AC10 partagés). Registre de la story :
`_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-4.md` (C-29-4-1 à 18). Epic :
`_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` § 29-4. Issue #651 (`gh api repos/guycorbaz/kesh/issues/651`
et `…/651/comments`).

**Historique de la boucle.** P1 (Sonnet) : lentille R 0/0/6/7, lentille F 0/0/2/7 — rapports
`/home/gcorbaz/devel/kesh-gate-logs/29-4-validate-p1-{R,F}.md`, à lire. Remédiation : commit de la fiche postérieur à
`cd61980b` (`git log --oneline cd61980b..HEAD`, puis `git diff cd61980b HEAD -- _bmad-output/`), résumée au Change Log
(entrée « Validation P1 »). Antécédent : fiche `28-4-cloture-compte-bancaire.md` et rapports
`/home/gcorbaz/devel/kesh-gate-logs/28-4-validate-p{1,2,3,4,5}-{R,F}.md` — ils décrivent les cycles qui ont fait écarter
la garde verrouillante. Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes
déclarés, les sites NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Règle de splitting
préventif ; § Pattern batch ; § Issue Tracking Rule) ; `docs/agents-orchestration.md` ; rétrospective
`epic-28-retro-2026-10-14.md` § 4.2 (une simplification se relit sur les verrous **et** la naissance de la vue) et § 4.3.

## Axe obligatoire, commun aux deux lentilles — relire ce que la remédiation P1 a écrit

*La sévérité ne stagne pas, elle se déplace vers ce qu'on vient d'écrire* (CLAUDE.md § *La passe ciblée*). La
remédiation P1 a **ajouté** de la conception, non seulement corrigé du texte. Chacun des ajouts suivants se relit comme
s'il était neuf, contre le code au sol :

1. **AC11 et C-29-4-13 — l'épinglage `SET SESSION innodb_snapshot_isolation = OFF`** dans `create_pool`
   (`crates/kesh-db/src/pool.rs`), avec tolérance de l'erreur 1193. Est-ce juste (portée de session de la variable ;
   ordre par rapport au `SET SESSION sql_mode …` existant ; effet sur les sauvegardes et la restauration, qui ouvrent
   leurs propres instantanés — Story 28-3, `kesh-db/src/snapshot.rs`, `backup.rs`) ? Couvre-t-il **toutes** les
   connexions de production (cherche tout `MySqlPool` ou `MySqlConnection` créé hors de `create_pool` dans
   `kesh-api/src`, `kesh-db/src`, `kesh-seed`) ? Ignorer 1193 est-il sûr, et que se passerait-il si la variable
   existait mais en lecture seule ? Le test T29a-8 est-il réalisable sur `mariadb:10.11` (la fiche affirme la variable
   présente sur 10.11.16 — constat de la remédiation, à recouper au code et à la documentation, sans exécuter), et son
   volet (a) dit-il vrai sur ce qu'il peut constater ? Les écartées (traduire 1020 ; refuser de démarrer ; exigence au
   manuel) le sont-elles pour de bonnes raisons ?
2. **AC4 — la séquence réécrite de T29a-4** : est-elle réalisable pas à pas (qui tient quoi, qui attend qui, à chaque
   étape ; que fait la clôture **rejouée** pendant que `T1` et `T2` tiennent encore leurs verrous ; un second cycle
   peut-il naître entre l'étape (5) et l'étape (8)) ? Le sondage de `capture_rejeu::operations()` est-il possible
   (`crates/kesh-api/tests/common/capture_rejeu.rs`) ? Et **le retrait du rejeu de la réouverture** (C-29-4-5 amendé,
   C-29-4-14) : cherche un cycle réaliste où la réouverture serait victime — en particulier par une revérification de
   la 29-4b sur un compte **clos** (elle tient son partagé jusqu'au `rollback`).
3. **AC3 — T29a-6 et T29a-7** (« l'écrivain a inséré, la clôture attend, puis l'écrivain revérifie ») : rougissent-ils
   si la prémisse P-d (2) est fausse, et pour la bonne raison (l'assertion sur `close_in_tx` **non rejouée** rendant
   `1213`) ? Le délai de 500 ms suffit-il à établir que la demande exclusive est **dans la file** — et sinon, le test
   peut-il passer à vide ? Comment l'écrivain fait-il son `INSERT` « sans la revérification » une fois celle-ci posée
   dans `create_with_transactions` / `create_batch` (le test reste-t-il possible après le dev) ? M29-19 est-elle tuée ?
4. **AC5 — l'ensemble `G` extrait du code et l'égalité avec `FENETRE`** : refais l'extraction depuis le symptôme
   (appelants d'`ensure_open_in_tx` / `ensure_open_unarchived_in_tx` ; littéraux lisant `closed_on` `FROM
   bank_accounts` sans verrou, hors `bank_accounts.rs`) — rend-elle bien les neuf fonctions annoncées, ni plus ni moins ?
   L'exclusion de `bank_accounts.rs` cache-t-elle un site ? Les pré-vols de route écartés le sont-ils à bon droit ?
   L'**ordre textuel par positions de littéraux** (AC5 iii, AC8) est-il implémentable avec `lexique::decouper`
   (`crates/kesh-db/tests/lexique/mod.rs`) tel que décrit, et tue-t-il M29-6, M29-7, M29-14 à M29-18 ?
5. **Dérogation de splitting (29-4b, C-29-4-18) et décompte des modules** : recompte les deux parties depuis la fiche,
   avec la convention qu'elle écrit (code / doc-comments seuls / documents / tests). La 29-4a, avec `pool.rs`, reste-t-elle
   à cinq ? La dérogation est-elle motivée selon la règle (rollout mécanique, risque accepté) ?
6. **Les textes réécrits** : angle mort « PATCH » et résidu C-29-4-7 « théorique » (le raisonnement sur le poids des
   transactions tient-il ?) ; borne de l'audit au volet 1 ; état transitoire entre les deux PR (AC9) ; course
   archivage / lot (L5, C-29-4-16) — la fiche la dit hors #651 : est-ce vrai, et le défaut est-il bien nommé ?
7. **Propagation** : chaque valeur que la remédiation a changée (`T-a1`…`T-a5`, `T-b1`…`T-b4`, `M1`…`M15` → `T29a-n`,
   `T29b-n`, `M29-n` ; « sept → neuf » devenu « sept → huit » ; « 31 + 3 » → « 27 + 3 » ; la réouverture rejouée) est-elle
   propagée partout dans la fiche, le registre et `sprint-status.yaml` — **greper la valeur** —, sans réécrire les
   relevés datés (Change Log de création, choix d'origine annotés, prompt P1) ?

## Axes prioritaires (rappel de la P1, à reprendre sur l'état actuel)

1. **Prémisses de #651 et de la fiche** (table *Prémisses*, P-a à P-f) : vraie au code (citer) ? vraie d'InnoDB /
   MariaDB 10.11 (dire si **vérifiée en pratique** dans le dépôt — nommer le test — ou **raisonnée**) ? que deviendrait la
   conception si elle était fausse ? Une prémisse fausse qui porte une fermeture de fenêtre est au moins **HIGH**.
2. **Périmètre minimal** contre #651, phrase par phrase ; table *Dehors* (dont les trois lignes ajoutées en P1).
3. **Inventaire clos des sites non résolus** — refais-le depuis le symptôme ; un site à fenêtre réelle omis, ou une
   démonstration « sans fenêtre » fausse (W30, W20, W1-W6, `accept_one_*`), est au moins **HIGH**.
4. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` § *Clore un compte bancaire*
   (`\label{sec:clore-compte-bancaire}`, puce l. 1748-1752) et toute autre mention de la fenêtre ; `docs/manual/fr/admin-manual.tex`
   (version de MariaDB exigée, l. 108 « MariaDB 10.11+ », et l. 2708 — l'épinglage de l'AC11 rend-il ces lignes vraies
   pour 11.6+, ou faut-il les écrire ?) ; `docs/api-external.md` l. 241 ; `CHANGELOG.md` (section `[0.13.3]`, l. 37 — qui
   ne doit **pas** être réécrite). Ce que l'AC9 prescrit d'écrire sera-t-il vrai au code prescrit ? Contrôle aussi les
   **PDF**, aplatis : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (et `admin-manual.pdf`).
5. **Recomptes** : chaque nombre de la fiche se recompte depuis la source — 31 sites, 27 et 3 entrées du registre,
   sept routes `SansEcritureAuJournal` rejouées au point (vi) d'`audit_route_registry.rs` (→ huit), neuf fonctions de
   `G`, tests `T29a-1` à `T29a-8` et `T29b-1` à `T29b-4`, mutations `M29-1` à `M29-19`, choix (18), modules par partie,
   et chaque numéro de ligne cité (`sed -n`).

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque AC testable et couvert par une tâche ; chaque AC par un test **qui rougirait au sol pour la bonne raison** ;
  chaque mutation tuée par le test nommé, ou déclarée équivalente avec sa raison ; chaque fait, chemin, numéro de ligne
  **revérifié** ; les textes prescrits (doc-comments, registre, manuels, `api-external.md`, CHANGELOG) confrontés au code
  prescrit ; les tests existants qui pourraient changer de sens (T-a31, T-a45, T-a48, `a_concurrent_close_waits_for_the_unreconciliation`,
  `annuler_un_rejet_pendant_une_cloture_refuse_par_w30`, `bank_accounts_closed_e2e.rs`, `rejeu_interblocage_e2e.rs`,
  `audit_route_registry.rs`, `reconciliation_e2e.rs:7556` et `:8401`) ; propagation de « L1 » / « #651 » complète
  (`grep -rn "651\|\bL1\b" crates docs CHANGELOG.md frontend/src website README.md`) et de `snapshot_isolation`
  (`grep -rn "snapshot_isolation\|ER_CHECKREAD" crates docs`).
- **F — Faisabilité, verrous et naissance de la vue (adversaire de la conception)** — ⛔ **la table des verrous et des
  cycles est obligatoire au rapport**, refaite **depuis le code**, chemin par chemin, pour W9, W17, W22, W24, W25, W28,
  W29, la clôture et la réouverture (désormais non rejouée) : la suite des verrous au sol, **verrous implicites de clé
  étrangère compris** (`companies`, `bank_accounts`, `accounts`, `users`, intervalles d'index uniques et non uniques), la
  position exacte prescrite pour la revérification, et les trois volets du critère — (1) verrou **vraiment** déjà tenu
  (W17, W22) ou **vraiment** dernière acquisition (dans la fonction **et** dans la route appelante, W25 imbriqué dans W9
  compris, borne de l'audit comprise) ; (2) **cherche le cycle** contre **tous** les détenteurs d'un exclusif sur la
  ligne du compte — extraction refaite, lectures verrouillantes **et** écritures implicites (`UPDATE`/`DELETE`) —, contre
  les cycles nommés par les P1-P3 de la 28-4, contre les **cycles à trois** par la file d'attente de la clôture, et contre
  les cycles que l'épinglage de session pourrait changer (un chemin qui rendait 1020 et rend désormais une attente) ;
  pour chaque cycle : **tous** ses participants sont-ils rejoués (lis les enveloppes) ? (3) naissance de la vue inchangée
  sur chaque chemin ? Et : le rejeu de la clôture (AC4) est-il sûr (idempotence, contrôles hors tentative, conversion des
  erreurs, `rollback` explicite sur refus) ; les tests à trois connexions (T29a-4, T29b-3) et ceux de l'autre ordre
  (T29a-6, T29a-7) sont-ils réalisables tels que décrits ; la victime est-elle celle annoncée ? Une piste écartée à tort
  qui serait moins risquée est au moins **MEDIUM**.

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve**
(commande exécutée et sortie copiée), **origine** (conception d'origine de la fiche, **remédiation P1**, prémisse de
l'issue, ou héritage de la 28-4) — l'origine « remédiation P1 » se dit explicitement, c'est la mesure de la boucle —,
correction proposée. Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée ; toute affirmation sur
InnoDB dit si elle est raisonnée ou vérifiée, et où. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding »
sans elle ne compte pas comme passe ; l'axe obligatoire (1 à 7 ci-dessus) y figure point par point. Pour la lentille F,
la **table des verrous et des cycles**, chemin par chemin, figure au rapport. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/29-4-validate-p2-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/29-4-validate-p2-F.md` (lentille F) ; dernier message : le chemin du rapport, le
bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ Lecture seule stricte. N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/` (ton rapport, et au besoin un
aplatissement de PDF nommé `29-4-validate-p2-<R|F>-*.txt`). Aucune commande qui écrit, compile ou exécute : `scripts/*`
(dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `scripts/regen-test-schema.sh`),
`make`, `latexmk`, `xelatex`, `cargo`, `npm`, `npx`, `node`, **`python3`, `python`, `perl`, `ruby` et tout autre
interpréteur**, `docker`, **`mariadb`, `mysql` et toute requête SQL** (même en lecture, même sur la base de dev),
`bash` sur un script du dépôt, `gh` en écriture (`issue`, `pr`, `comment`…). ⛔ **Aucun `git checkout`, `git switch`,
`git checkout --detach`, `git worktree`, `git stash`, `git reset`, `git rebase`, `git commit`, `git add`, `git fetch`,
`git pull`, `git tag`, `git push`, sous aucune forme** — pour lire une autre révision, `git show <rév>:<chemin>`. Rien de
`~/kesh-test/` (relevé réel de Guy) ne se lit ni ne se cite ; aucune donnée réelle dans le rapport. Autorisés : lecture
de fichiers, `grep`, `sed -n`, `awk` en lecture, `git log` / `show` / `diff` / `blame`, `gh api` en lecture, `pdftotext`
vers la sortie standard ou vers `kesh-gate-logs/`.
