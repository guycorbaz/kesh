# Prompt — validation P1 de la spec, Story 29-4 (une saisie concurrente ne passe plus sur un compte qu'on clôt)

*Versionné le 2026-10-14. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) : passes
complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : **Sonnet** pour les deux lentilles
(la fiche a été écrite par Opus). Le fichier d'epic exige pour cette story une validation **complète Opus et Sonnet** :
la P2 sera Opus.*

Dépôt `/home/gcorbaz/devel/kesh-29-4`, branche `story/29-4` (base `16eeed21`, kickoff de l'Epic 29). Fiche :
`_bmad-output/implementation-artifacts/29-4-cloture-compte-concurrente.md` (une fiche, **deux** stories : 29-4a,
AC1-AC5 ; 29-4b, AC6-AC10 ; AC9 partagé). Registre de la story :
`_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-4.md` (C-29-4-1 à 12). Epic :
`_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` § 29-4. Issue #651 (`gh api repos/guycorbaz/kesh/issues/651`
et `…/651/comments`). Antécédent : fiche `28-4-cloture-compte-bancaire.md` (AC6, AC7, § *Concurrence*, L1), registre
`epic-28-choix-autonomes.md` (C-28-4-14, -16, -17, -21, -24) et les rapports de validation de la 28-4,
`/home/gcorbaz/devel/kesh-gate-logs/28-4-validate-p{1,2,3,4,5}-{R,F}.md` — **ils décrivent les cycles qui ont fait
écarter la garde verrouillante : c'est contre eux que la conception de la 29-4 se juge**. Règles : `CLAUDE.md`
(§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites NON RÉSOLUS, la propagation
post-patch, le recompte des comptes rendus ; § Règle de splitting préventif ; § Pattern batch ; § Issue Tracking Rule) ;
`docs/agents-orchestration.md` ; rétrospective `epic-28-retro-2026-10-14.md` § 4.2 (une simplification se relit sur les
verrous **et** la naissance de la vue) et § 4.3.

## Axes prioritaires (communs aux deux lentilles)

1. **Prémisses de #651, et celles de la fiche.** La fiche corrige deux prémisses de l'issue (W30 sans fenêtre par I1 ;
   W24 seulement pour un import vide) et en pose plusieurs d'InnoDB (l'`INSERT` d'un enfant pose un verrou partagé
   `LOCK_REC_NOT_GAP` sur la ligne parente ; une revérification `LOCK IN SHARE MODE` par égalité sur la clé primaire
   redemande **le même** verrou et n'en crée aucun quand il est tenu ; une demande partagée attend derrière une demande
   exclusive **en attente** ; la victime d'un interblocage est la transaction la plus légère). Pour chacune : est-elle
   vraie au code (citer), vraie d'InnoDB / MariaDB 10.11 (dire sur quoi repose la conviction, et si elle est
   **vérifiée en pratique** dans le dépôt — T-a31, `rejeu_interblocage_e2e.rs` — ou seulement raisonnée), et que
   deviendrait la conception si elle était fausse ? Une prémisse fausse qui porte une fermeture de fenêtre est au moins
   **HIGH**.
2. **Périmètre minimal.** Confronte la section *Périmètre minimal* à #651 phrase par phrase : chaque chemin exposé
   est-il fermé ou démontré sans fenêtre ? chaque AC, tâche, test se rattache-t-il à #651 ? Le découpage 29-4a / 29-4b
   (C-29-4-6) tient-il contre la règle de splitting (refais le décompte des modules) ; la 29-4a est-elle cohérente
   seule (son registre dit `Ouverte("29-4b")`) ?
3. **Inventaire clos des sites non résolus.** La fiche affirme que les seuls sites à fenêtre réelle sont W9, W17, W22,
   W24 (import vide), W25, W28, W29. Refais l'inventaire **depuis le symptôme** — toute garde « compte clos » qui
   n'est pas lue sous un verrou de la ligne du compte —, et non depuis la liste de la 28-4 : registre
   `crates/kesh-api/tests/bank_account_closed_inventory.rs` (31 sites), appelants des gardes
   (`grep -rn "ensure_open_in_tx\|ensure_open_unarchived_in_tx\|closed_on" crates/*/src`), pré-vols de route R1-R8.
   Un site à fenêtre réelle omis, ou une démonstration « sans fenêtre » fausse (W30, W20, W1-W6, `accept_one_*`), est
   au moins **HIGH**.
4. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` § *Clore un compte bancaire*
   (`\label{sec:clore-compte-bancaire}`, puce l. 1747-1752) et toute autre mention de la fenêtre ; `docs/api-external.md`
   l. 241 ; `CHANGELOG.md` (section `[0.13.3]`, l. 37 — qui ne doit **pas** être réécrite). Ce que l'AC9 prescrit
   d'écrire sera-t-il vrai au code prescrit (en particulier : « ne passe pas sur le compte clos », « la clôture peut
   attendre », « un import peut être refusé après l'analyse ») ? Contrôle aussi le **PDF**, aplati :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`.
5. **Recomptes** : chaque nombre de la fiche se recompte depuis la source — 31 sites et 3 appelants du registre, « sept »
   routes `SansEcritureAuJournal` rejouées au point (vi) d'`audit_route_registry.rs`, nombre de chemins à fenêtre, de
   tests (T-a1-T-a5, T-b1-T-b4), de mutations (M1-M15), de choix (12), et chaque numéro de ligne cité (`sed -n`).

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque AC testable et couvert par une tâche ; chaque AC par un test **qui rougirait au sol pour la bonne raison**
  (les tests de fenêtre AC3 et AC7 : rougissent-ils à l'étape annoncée ? la sonde `attendre_une_requete_en_cours`
  voit-elle une requête en cours ou bloquée — leçon de la P5 de la 28-4) ; chaque mutation tuée par le test nommé, ou
  déclarée équivalente avec sa raison ; chaque fait, chemin, numéro de ligne **revérifié** ; les textes prescrits (doc-
  comments, registre, manuel, `api-external.md`, CHANGELOG) confrontés au code prescrit ; les gardes et tests existants
  qui pourraient changer de sens (T-a31, T-a48, `a_concurrent_close_waits_for_the_unreconciliation`,
  `annuler_un_rejet_pendant_une_cloture_refuse_par_w30`, `bank_accounts_closed_e2e.rs`, `rejeu_interblocage_e2e.rs`,
  `audit_route_registry.rs`) — la fiche les nomme-t-elle tous ? la propagation de « L1 » / « #651 » est-elle complète
  (greper la **valeur** : `grep -rn "651\|\bL1\b" crates docs CHANGELOG.md frontend/src website README.md`) ?
- **F — Faisabilité, verrous et naissance de la vue (adversaire de la conception)** — **l'axe obligatoire** : refais,
  **depuis le code**, la table *Position par chemin* et la table *Cycles* des Dev Notes. Pour **chaque** chemin touché
  (W9, W17, W22, W24, W25, W28, W29, clôture, réouverture) : la suite des verrous au sol, **verrous implicites de clé
  étrangère compris** (`companies`, `bank_accounts`, `accounts`, `users`, intervalles d'index uniques et non uniques),
  la position exacte prescrite pour la revérification, et les trois volets du critère : (1) le verrou est-il **vraiment**
  déjà tenu (W17, W22 — forme du verrou du contrôle de clé étrangère contre celle de la lecture ; plan d'exécution) ou
  **vraiment** la dernière acquisition (aucun verrou après, dans la fonction **et** dans la route qui l'appelle, y
  compris W25 imbriqué dans W9) ? (2) **cherche le cycle** contre **tous** les détenteurs d'un exclusif sur la ligne du
  compte (refais l'extraction : toute lecture verrouillante et toute écriture de `bank_accounts`, dont `flip_primary_off_for_company`
  qui balaie les comptes de la société, `upsert_primary_in_tx`, `PATCH`, `PUT`, `POST`, `DELETE`, W8, W27, la clôture et
  la réouverture), contre les cycles nommés par les P1-P3 de la 28-4 (routes de gestion, W30 / W24, W22 / W24,
  `accept_one_invoice`, `create_batch` / `confirm_batch`, intervalle de `uq_invoice_settlements_entry`) et contre les
  **cycles à trois** par la file d'attente de la clôture ; pour chaque cycle trouvé : **tous** ses participants
  sont-ils rejoués (lis les enveloppes de `routes/` et de `kesh-db`) ? (3) la naissance de la vue est-elle inchangée
  sur chaque chemin ? Et : le rejeu de la clôture et de la réouverture (AC4) est-il sûr (idempotence, contrôles hors
  tentative, conversion des erreurs) ; les tests à trois connexions (T-a4, T-b3) sont-ils réalisables tels que décrits,
  la transaction de test obtient-elle bien son verrou, la victime est-elle celle annoncée ? Les pistes écartées
  (C-29-4-1 : verrou commun, verrou nommé, lignes enfants, exclusif) le sont-elles pour de bonnes raisons — une piste
  écartée à tort qui serait moins risquée est au moins **MEDIUM**.

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve**
(commande exécutée et sortie copiée), origine (conception de la fiche, prémisse de l'issue, ou héritage de la 28-4),
correction proposée. Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée ; toute affirmation sur
InnoDB dit si elle est raisonnée ou vérifiée, et où. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding »
sans elle ne compte pas comme passe. Pour la lentille F, la **table de l'axe obligatoire**, chemin par chemin, figure au
rapport. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-4-validate-p1-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/29-4-validate-p1-F.md` (lentille F) ; dernier message : le chemin du rapport, le
bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ Lecture seule stricte. N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/` (ton rapport, et au besoin un
aplatissement de PDF nommé `29-4-validate-p1-<R|F>-*.txt`). Aucune commande qui écrit, compile ou exécute : `scripts/*`
(dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `scripts/regen-test-schema.sh`),
`make`, `latexmk`, `xelatex`, `cargo`, `npm`, `npx`, `node`, **`python3`, `python`, `perl`, `ruby` et tout autre
interpréteur**, `docker`, **`mariadb`, `mysql` et toute requête SQL**, `bash` sur un script du dépôt, `gh` en écriture
(`issue`, `pr`, `comment`…). ⛔ **Aucun `git checkout`, `git switch`, `git checkout --detach`, `git worktree`, `git
stash`, `git reset`, `git rebase`, `git commit`, `git add`, `git fetch`, `git pull`, `git tag`, `git push`, sous aucune
forme** — pour lire une autre révision, `git show <rév>:<chemin>`. Rien de `~/kesh-test/` (relevé réel de Guy) ne se lit
ni ne se cite ; aucune donnée réelle dans le rapport. Autorisés : lecture de fichiers, `grep`, `sed -n`, `awk` en
lecture, `git log` / `show` / `diff` / `blame`, `gh api` en lecture, `pdftotext` vers la sortie standard ou vers
`kesh-gate-logs/`.
