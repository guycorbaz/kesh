# Prompt — validation P2 de la spec, Story 29-6 (la réconciliation ne propose plus une facture qu'un trop-perçu de quelques centimes fait refuser)

*Versionné le 2026-10-14. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; P1 était Sonnet, **P2 est Opus** pour les deux lentilles. La remédiation P1 a été
écrite par Opus : c'est un biais d'auteur possible — relis-la comme si elle venait d'un autre.*

Dépôt `/home/gcorbaz/devel/kesh-29-6`, branche `story/29-6` (base `fcd2ae75`, kickoff de l'Epic 29).
Fiche : `_bmad-output/implementation-artifacts/29-6-trop-percu-centimes.md`. Commits de la fiche : `80c5a278`
(création), `16cab2ad` (prompt P1), **`2552b7ab` (remédiation P1 — l'objet premier de cette passe)** :
`git show 2552b7ab` et `git diff 80c5a278 2552b7ab`.
Rapports P1 : `/home/gcorbaz/devel/kesh-gate-logs/29-6-validate-p1-R.md` (0/0/1/8) et `…-p1-F.md` (0/0/0/1).
Epic : `_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` § *29-6*, § *Ordre*, § *Releases*. Issue #665 :
`gh api repos/guycorbaz/kesh/issues/665`. Fiches voisines **non mergées**, en lecture seule :
`/home/gcorbaz/devel/kesh-29-2/_bmad-output/implementation-artifacts/29-2-paiement-tardif.md` (C-29-2-8 : le même
contournement « puis rejeter ») et
`/home/gcorbaz/devel/kesh-29-3/_bmad-output/implementation-artifacts/29-3-meilleure-proposition.md` (**même composant**
`ReconciliationProposals.svelte`, une indication et une clé i18n à elle, `sitesTotal` +1).
Registre : `_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-6.md` (C-29-6-1 à 15).
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Règle de splitting préventif ; § Pattern
batch ; § E2E) ; `docs/agents-orchestration.md` ; rétrospective `epic-28-retro-2026-10-14.md` § 4 et § 7.

## Axe obligatoire — relire la remédiation P1, qui étend le périmètre à l'écran

La remédiation P1 a ajouté l'**AC10** (une ligne « Aucune correspondance » se coche, pour *Rejeter* seulement ;
*Accepter* grisé et gardé ; une indication et une clé i18n ; tests Vitest ; spec E2E neuf ; captures clair et sombre),
sur **décision de l'orchestrateur**. La décision n'est pas en cause ; **sa traduction en fiche l'est**. Le motif mesuré
du dépôt : *la sévérité se déplace vers ce qu'on vient d'écrire*. Donc, au code de `fcd2ae75` :

1. **Les faits de l'AC10** : la route `POST /reconciliation/reject` ne lit-elle vraiment aucune candidate
   (`classify_batch_transaction`, `reject_batch`) ? Le test `post_reject_marks_transactions_as_manually_reviewed`
   (`reconciliation_e2e.rs:1560`) monte-t-il vraiment des transactions sans facture ? *Annuler le rejet* (28-5b) est-il
   vraiment indépendant des candidates ? Une transaction **sans proposition** peut-elle être rejetée puis réapparaître
   ailleurs qu'attendu (fenêtre des 100 plus récentes, compte clos 28-4, transaction d'un import supprimé 27-x) ?
2. **Le comportement prescrit** : la sélection, le `disabled` d'*Accepter*, la garde d'`onAccept`, l'indication et sa
   condition — tous les états sont-ils écrits (sélection vide, une ligne sans proposition seule, mélange, décocher,
   ligne à **règle** seule, rechargement après le lot, `busy`) ? *Accepter* peut-il, par un chemin quelconque, envoyer
   une ligne sans proposition ? Une transaction qui **perd** sa proposition entre deux chargements, cochée, se
   comporte-t-elle comme écrit ? Le compteur « Accepter (n) » reste-t-il vrai ? Les bilans *Échecs partiels* et le
   compteur de succès (15-5c) restent-ils justes pour un lot de rejet qui mêle les deux sortes de lignes ?
3. **Les textes prescrits** : les quatre valeurs de `reconciliation-labels-reject-only-hint` disent-elles vrai, avec
   les termes de l'écran dans chaque locale (`reconciliation-labels-no-candidate`, `-actions-accept`,
   `-actions-reject`) ? Le texte du manuel (`:2029`, `:2050`) et du CHANGELOG prescrits par l'AC7 sont-ils vrais **au
   code prescrit** ? Une autre phrase du manuel, de l'API externe, du site (`website/`) ou d'un commentaire devient-elle
   fausse par l'AC10 sans être inventoriée (le second grep du § *Propagation* est-il complet) ?
4. **Les tests et mutations neufs** : chaque test Vitest prescrit rougit-il au sol ? M9, M10, M11 sont-elles tuées par
   les tests nommés, et la garde d'`onAccept` déclarée non éprouvée l'est-elle à juste titre ? Le spec E2E neuf est-il
   **faisable sur la base partagée** (une transaction qu'aucune facture ni règle d'un autre spec ne rapproche ; la
   fenêtre des 100 ; nettoyage en `finally`) ? Le second test de dépôt (R4, M8) garde-t-il réellement « filtre avant
   `LIMIT 50` » — `insert_test_invoice` en boucle (numéro unique), ordre `(date, id)`, et la mutation M8 décrite
   est-elle la bonne ?
5. **Le recompte des modules** (C-29-6-12) : cinq — `kesh-db`, `kesh-api`, `frontend/…/reconciliation`, `kesh-i18n`,
   documentation — est-il juste (`i18n-keys.test.ts` dans `frontend/src/lib/shared`, spec E2E dans `frontend/tests`) ?
   L'argument contre le découpage 29-6a / 29-6b tient-il ? Un sixième module caché fait dépasser la règle : au moins
   **MEDIUM**.
6. **Les zones partagées** : la 29-3 écrit dans le même composant (indication de priorité, en-tête, `sitesTotal`) ; la
   29-2 garde C-29-2-8 et compte sur l'AC10. Le § *Zones partagées* le dit-il assez pour un rebase sans perte ? Les
   deux indications (priorité 29-3, rejet seul 29-6) coexistent-elles sans se contredire, y compris quand **toutes**
   les lignes sont « Aucune correspondance » ?
7. **Les LOW de P1 remédiés** (R2 à R9, F1) : chaque correction est-elle juste, et n'a-t-elle rien cassé ailleurs dans
   la fiche (R6 a aussi corrigé un renvoi AC1 → AC2 ; R7 touche le fichier d'epic et `sprint-status.yaml`) ? Une
   entrée datée a-t-elle été réécrite au lieu d'être annotée ?

Une prescription fausse au code, un état non écrit qui ferait accepter une ligne sans proposition, ou un texte qui
promettrait ce que l'écran ne fait pas : au moins **MEDIUM**.

## Axes de fond (repris de P1, à exercer de nouveau)

8. **Prémisses** et **démonstration** (`classify_payment(P, R) = Overpayment ⇔ R < P − 0,005`), bornes, `R` négatif,
   précision des paramètres liés ; une erreur de borne est **HIGH**.
9. **Inventaire clos des sites** (Dev Notes (a), (b), (c)) : refais les `grep` copiés, élargis-les ; pour l'AC10, tout
   code ou test qui suppose qu'une ligne sans proposition **ne se coche pas** (`tx-checkbox`, `candidates.length`,
   `candidates[0]`) — Vitest, Playwright, composants voisins (`ManualMatchModal`, `TransactionSplitModal`).
10. **Manuel contre code (obligatoire)** : `docs/manual/fr/user-manual.tex` l. 971, 1308, 2025, 2029, 2049, 2050, 2056,
    2057, 2062, 2330, 2901 ; ce que le manuel dit aujourd'hui est-il vrai, ce que l'AC7 prescrit le sera-t-il ?
    **Contrôle le PDF**, aplati et normalisé : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '
    | sed 's/ﬀ/ff/g;s/ﬁ/fi/g;s/ﬂ/fl/g;s/ﬃ/ffi/g'` (vers la sortie standard seulement). Les contrôles PDF prescrits
    (positifs puis négatifs) sont-ils exécutables tels qu'écrits ? Même exigence pour `docs/api-external.md` (`:505`,
    `:551`) et le CHANGELOG prescrit.
11. **Recomptes** : chaque nombre et numéro de ligne de la fiche se recompte depuis la source — AC (10), tâches (8 :
    T0 à T6 et T3bis), mutations (11), choix (15), modules (5), cas d'AC6 (a-h), lignes d'exemple (10), `sitesTotal`
    (2107 au sol), numéros de ligne de l'AC10, et les décomptes de l'entrée « Validation P1 » du Change Log.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de #665, du § *29-6* de l'epic et de la décision de l'orchestrateur couverte par un AC testable, chaque
  AC par une tâche, chaque AC par un test **qui rougirait** au sol ; chaque fait, chemin et numéro de ligne revérifié
  (`sed -n`, `grep -nF`) ; les textes prescrits (manuel, i18n, CHANGELOG, API) confrontés au code prescrit ; les gardes
  existantes qui pourraient changer de sens (`ReconciliationProposals.test.ts`, `i18n-keys.test.ts`,
  `failed-proposal-label.test.ts`, `textes_coherents.rs`, les specs Playwright `reconciliation-*.spec.ts`) ; Pattern
  batch intact.
- **F — Faisabilité et exactitude (adversaire de la conception)** : axes 2, 4, 5 et 8 en priorité — les états de la
  sélection écrits jusqu'au bout (Svelte 5 : `$derived`, `$state`, `Set` réaffecté), la faisabilité des tests Vitest
  (`render`, `fireEvent`, mocks de `reconciliation.api`) et du spec E2E sur la base partagée, la faisabilité du second
  test de dépôt et la justesse de M8 à M11, la démonstration et les binds (`HALF_CENTIME`, `HAVING`), B5 (verrous et
  naissance de la vue : rien de déplacé dans une transaction).

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence =
sortie de `grep -nF` copiée. Distingue ce qui vient de la **remédiation P1** (`2552b7ab`) de ce qui vient de la
conception d'origine. ⛔ **La liste des axes exercés ET non exercés** (1 à 11, et la lentille) — un « 0 finding » sans
elle ne compte pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-6-validate-p2-R.md` (lentille R)
ou `/home/gcorbaz/devel/kesh-gate-logs/29-6-validate-p2-F.md` (lentille F) ; dernier message : le chemin du rapport,
le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/` — **aucune redirection de sortie vers un
fichier** ailleurs, `/tmp` compris (écart de la lentille R en P1). Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et les scripts
Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout` **sous aucune forme** (`--detach` compris)/
`switch`/`restore`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`/`worktree`, `cargo`, `npm`, `npx`, `node`,
`vitest`, `playwright`, **tout interpréteur** (`python3`, `python`, `perl`, `ruby`, `bash`/`sh` sur un script, `awk`
qui écrit), `docker`, **`mariadb`/`mysql`** et toute requête SQL, `gh` en écriture. Rien de `~/kesh-test/` (relevé
réel de Guy) ne se lit ni ne se cite. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`/`blame`,
`gh api` en lecture, `pdftotext` vers la sortie standard.
