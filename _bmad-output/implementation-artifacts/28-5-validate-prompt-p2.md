# Prompt — validation P2 de la spec, Story 28-5 (28-5a serveur / 28-5b écran)

*Versionné le 2026-10-13. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 était Sonnet : **P2 est
Opus** pour les deux lentilles (la remédiation P1 a été écrite par Opus — sois d'autant plus adversarial sur elle).*

Dépôt `/home/gcorbaz/devel/kesh-28-5`, branche `story/28-5` (base `5aad8b35`).
Fiche : `_bmad-output/implementation-artifacts/28-5-transaction-rejetee-atteignable.md` — **remédiation P1 au commit
`34ee9a72`** (`git show 34ee9a72` donne le diff exact ; la fiche d'avant est à `c8acbfe1`).
Rapports P1 : `/home/gcorbaz/devel/kesh-gate-logs/28-5-validate-p1-R.md` et `…-p1-F.md` (6 MEDIUM, 14 LOW).
Epic : `_bmad-output/planning-artifacts/epic-28-recette-v0-13-2.md` (§ 28-5, § *Ordre*). Issues #526 (corps et deux
commentaires) et **#649** (limite de 100, ouverte à la P1) — `gh api` en lecture. Registre :
`_bmad-output/implementation-artifacts/epic-28-choix-autonomes.md` (C-28-5-1 à 24 ; C-28-5-11 et 12 marqués
remplacés). Précédents de découpage : fiche 26-2 (`26-2-description-compte-bancaire.md`) et fiche 28-4
(`/home/gcorbaz/devel/kesh-28-4/_bmad-output/implementation-artifacts/28-4-cloture-compte-bancaire.md`, qui porte
déjà W30/R8/M33 pour la route `unreject`). Consignes : `docs/agents-orchestration.md`. Règles : `CLAUDE.md`
(§ Review Iteration Rule et ses sous-sections, § Règle de splitting préventif, § Pattern batch, § Issue Tracking Rule).

## Axe PREMIER (commun aux deux lentilles) — les régressions de la remédiation P1

*La sévérité se déplace vers ce qu'on vient d'écrire* (`CLAUDE.md`, § *La passe ciblée*). Lis d'abord
`git show 34ee9a72` et éprouve **chaque** changement :

1. **`conclude_locked_attempt` rendu générique** (C-28-5-18, Dev Notes § *Séquence* point 6) : l'auxiliaire est
   aujourd'hui celui de `post_manual` et `post_split`, deux routes **rejouées**. Une signature générique en `T`
   compile-t-elle sans toucher au corps (le `commit` rend-il un type compatible, les appelants infèrent-ils `i64`) ?
   Son `match` convient-il à la route neuve (bras `FiscalYearClosed`, `SplitImbalance`, `TransactionAborted`,
   `Rule*` : inatteignables ici — un code rendu serait-il faux ?) ? Le `commit` d'une fermeture qui a rendu un refus
   (`NotFound` / `NotRejected`) est-il réellement sans effet (rien écrit, verrou `FOR UPDATE` relâché) ? Le doc-comment
   « ⛔ un 1213 sort intact » reste-t-il vrai ?
2. **La lecture du compte partagée** avec `cancel_reconciliation_once` (l. 4514-4524) : « mot pour mot » et « ne peut
   être victime d'un interblocage » — vrai pour une `SELECT` ordinaire en autocommit sous InnoDB ? Le rejeu de
   `…/cancel` (registre `Rejouee`, prédicat sur `DbError::Sqlx`) reste-t-il intact ?
3. **Garde `rows_affected() != 1` → `OptimisticLockConflict` → `409 OPTIMISTIC_LOCK`** (C-28-5-19) : code et message
   cohérents avec `docs/api-external.md` ? M13 et M14 réellement équivalentes (preuve écrite exacte) ? M15 « survivante
   assumée » : l'argument tient-il, et l'AC12 (a) l'autorise-t-elle telle qu'écrite ?
4. **AC13 / B1 / B9** (C-28-5-21) : le cycle rejeter → annuler → rejeter → annuler, et l'acceptation `split` après
   annulation via `setup_accept_split_ctx` (`reconciliation_e2e.rs:5382`) — le montage existe-t-il avec la signature
   supposée, une proposition `split` atteint-elle `classify_batch_transaction`, le refus « avant » est-il bien
   `RECONCILIATION_ALREADY_REJECTED` au pré-vol ?
5. **Textes nuancés pour la limite de 100** (C-28-5-20) : le toast, la puce du manuel, l'AC2 et `api-external.md`
   disent-ils vrai sans sur-promettre ni sous-promettre ? Le nombre 100 est-il celui du code **côté écran**
   (`reconciliation.api.ts:18`) autant que côté serveur ? Les quatre traductions disent-elles la même chose ? La
   phrase du manuel « n'y reparaît qu'une fois les plus récentes traitées » est-elle exacte (tri `booking_date DESC,
   id DESC`) ?
6. **Libellé `already-rejected` réécrit** et **indication de l'AC6** (C-28-5-21, C-28-5-22) : vrais pour tous les
   rôles et tous les écrans où ils s'affichent ? Repli de `failed-proposal-label.ts` égal au catalogue fr-CH, mot pour
   mot (apostrophes) ? Position « en fin de `<section>` » cohérente avec « sous la liste » du manuel ?
7. **Inventaire clos** (R3) : chaque ligne du `grep` déclaré est-elle désormais au tableau ? Refais le grep.

## Axe SECOND — la cohérence du découpage 28-5a / 28-5b

- **Rien entre deux chaises** : chaque AC, tâche, test (B1-B9, V1-V6), mutation (M1-M16), clé i18n, fichier du
  tableau *Régions touchées* appartient à **une** partie, et une seule ; la colonne « AC » du tableau de statut est
  exacte.
- **Chaque partie livrable et testable seule** : la 28-5a sans aucun écran (ses gardes — `audit_route_registry`,
  `audit_label_registry`, G9 de `textes_coherents.rs`, `i18n_e2e` — passent-elles avec les **seules** deux clés
  serveur ? une garde frontend, p. ex. `i18n-keys.test.ts` ou une garde de clés orphelines, rougirait-elle sur une
  clé de catalogue que la 28-5a ajoute sans site `i18nMsg` ?) ; la 28-5b sur `origin/main` après la fusion.
- **Gates** : la 28-5a (kesh-db touché, doc seulement) a-t-elle bien gate complet en boucle et **E2E complet** au
  dernier commit de code ? la 28-5b, qui modifie les catalogues `kesh-i18n`, a-t-elle son gate backend ?
- **Mots-clés de PR** (`refs #526` / `closes #526`), clé mère `split`, deux clés `ready-for-dev` dans
  `sprint-status.yaml` (lis-le par `grep`, pas par un interpréteur), phrase « aucune release entre les deux » au DAR
  de la 28-5a, entrée d'epic.
- **Entre les deux PR** : l'état intermédiaire (route sans écran, `autoMatchRejectedAt` exposé mais non typé côté
  frontend, libellé `already-rejected` encore impasse, manuel encore « aucun geste ») est-il sûr et dit ?
- **Ordre avec la 28-4 et la 28-6** : régions partagées (`failed-proposal-label.ts`, catalogues, `api-external.md`)
  exactes ; la note « à l'attention de l'orchestrateur » est-elle juste ?

## Axes repris de la P1 (toujours obligatoires)

- **Périmètre minimal** : la fiche ferme-t-elle #526 **et rien de plus** ? Un AC ou un test neuf de la remédiation
  est-il superflu ? Le tableau *Exclu* tient-il (chaque défaut a son issue) ?
- **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` l. 1724-1765, 1933-1952, 2014+ ; le texte
  prescrit sera-t-il vrai au code prescrit ? **PDF** aplati :
  `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` ; les motifs de l'AC8 rougissent-ils
  réellement aujourd'hui ? Même exercice pour `docs/api-external.md` (l. 534-575) et le CHANGELOG prescrit.
- **Recomptes** depuis la source : compteurs du registre des routes (115/109/91/118 → 116/110/92/119), `sitesTotal`
  2075, 13 AC (dont trois en deux volets), tâches T0a-T7a et T0b-T14, B1-B9, V1-V6, M1-M16, C-28-5-1 à 24, six clés
  neuves + une réécrite × quatre locales, six captures, et **chaque numéro de ligne cité** (`sed -n`) — en
  particulier ceux que la remédiation a ajoutés (`:3800-3820`, `:3822-3826`, `:3840-3851`, `:4312-4395`, `:5382`,
  l. 991, `repositories/reconciliation.rs:385-400`, `failed-proposal-label.ts:210-214`, catalogues l. 1219 / 1156).
- **Concurrence** (lentille F surtout) : séquence complète sous REPEATABLE READ, absence de cycle contre chaque
  écrivain de `bank_transactions`, classement `SansEcritureAuJournal`, déterminisme de B5, B9 et de l'E2E.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de #526 et du § 28-5 de l'epic couverte par un AC testable **de la bonne partie**, ou exclue par
  écrit ; chaque AC par une tâche et par un test qui rougirait ; chaque mutation par le test annoncé ; chaque fait,
  chemin et numéro de ligne revérifié ; textes prescrits (quatre locales, manuel, `api-external.md`, CHANGELOG)
  confrontés au code ; gardes existantes qui changent de sens, nommées dans la bonne partie.
- **F — Faisabilité, concurrence et contrat (adversaire de la conception)** : axes premier 1-4 en priorité (le code
  prescrit compile-t-il et se comporte-t-il comme écrit ?), puis concurrence, contrat d'API (codes, anti-énumération,
  clés d'API contre § 4 et § 7), champ additif, testabilité isolée de chaque partie.

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée ; dis pour chacun s'il vient **de la remédiation
P1** (`34ee9a72`) ou de la fiche d'origine. Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas comme passe. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/28-5-validate-p2-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/28-5-validate-p2-F.md` (lentille F) ; dernier message : le chemin du rapport,
le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et les scripts
Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/`reset`/
`rebase`/`tag`/`push`, `cargo`, `npm`, `npx`, `node`, **`python3`, `python`, `perl`, `ruby` ou tout autre
interpréteur** (même « en lecture seule », même pour lire un YAML), `docker`, **`mariadb`, `mysql` et toute requête
SQL**, `bash` sur un script du dépôt, `gh` en écriture. Rien de `~/kesh-test/` (relevé réel de Guy) ne se lit ni ne se
cite. Autorisés : lecture, `grep`, `sed -n`, `awk` en lecture, `wc`, `git log`/`show`/`diff`/`blame`, `gh api` en
lecture, `pdftotext` vers la sortie standard ou vers un fichier de `/home/gcorbaz/devel/kesh-gate-logs/`.
