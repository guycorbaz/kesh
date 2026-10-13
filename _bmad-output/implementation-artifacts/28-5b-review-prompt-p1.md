# Prompt — revue de code P1, Story 28-5b (annuler le rejet d'une transaction, côté écran)

*Versionné le 2026-10-13. Trois lentilles (**Sonnet**), contexte frais chacune, lecture seule.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-28-5`, branche `story/28-5b`, partie de `origin/main` `eab77cf1` (28-5a
fusionnée, PR #653).
**Le diff à revoir** : `git diff eab77cf1 31df9d74 -- . ':(exclude)_bmad-output'` (`31df9d74` =
**dernier commit de code** ; les commits suivants ne portent que la fiche, les registres et ce prompt). ⚠️ Pathspec
`':(exclude)…'` et **non** `':!…'`, que le shell peut interpréter. Fiche :
`_bmad-output/implementation-artifacts/28-5-transaction-rejetee-atteignable.md` — **partie b seulement** : AC5, AC6,
AC7 (b), AC8, AC9 (b), AC11, AC12 (b) ; tâches T0b-T14 ; § *Tests* (V1-V6, M9-M12, M16) ; Dev Notes (inventaire des
sites non résolus, § *Textes* partie b, puce *Rejeter* du manuel) ; et le **Dev Agent Record** de la 28-5b (rouge
d'abord, mutations, gate final, E2E, captures, décomptes). Choix C-28-5-1 à C-28-5-40 au registre
`epic-28-choix-autonomes.md` (C-28-5-36 à 40 : ceux de ce développement). Règles : `CLAUDE.md` du dépôt — § « Recompter
ses propres comptes rendus », § « Propagation post-patch », § « Le prompt d'une passe doit NOMMER le manuel »,
§ « Une passe qui ne déclare pas ses axes exercés ne compte pas », § *E2E* (« vert » se juge contre
`docs/testing.md` § *Les échecs attendus*). Issue : #526 (c'est la PR de la 28-5b qui la ferme).

⛔ **DONNÉES PERSONNELLES — DÉPÔT PUBLIC.** N'ouvre aucun fichier hors du dépôt et de
`/home/gcorbaz/devel/kesh-gate-logs/` ; rien sous `~/kesh-test/`.

**Contexte** : story **écran**. `frontend/src/lib/features/reconciliation/` : `UnrejectTransactionAction.svelte`
(neuf), `reconciliation-unreject.ts` (neuf, `isRejected`), `reconciliation.api.ts` (`unrejectTransaction`),
`ReconciliationProposals.svelte` (indication en fin de section), `failed-proposal-label.ts` (repli
`RECONCILIATION_ALREADY_REJECTED` réécrit) ; `frontend/src/lib/features/bank-import/` : `bank-import.types.ts`
(`autoMatchRejectedAt`), `bank-import-deletion.ts` (doc-comment) ; `frontend/src/routes/(app)/bank-import/[id]/+page.svelte`
(montage du composant). Tests Vitest : `reconciliation-unreject.test.ts`, `UnrejectTransactionAction.test.ts` (neufs),
`bank-import-detail-page.test.ts`, `ReconciliationProposals.test.ts`, `reconciliation.api.test.ts`,
`failed-proposal-label.test.ts`, fixtures de `bank-import-deletion.test.ts` et `DeleteBankImportAction.test.ts`,
`shared/i18n-keys.test.ts` (`sitesTotal` 2079 → 2083). E2E : `frontend/tests/e2e/reconciliation-unreject.spec.ts`
(neuf). Catalogues `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` (quatre clés neuves, une réécrite).
`docs/manual/fr/user-manual.tex` et `.pdf` ; `CHANGELOG.md`.

⚠️ **Région partagée avec la 28-6** (en parallèle sur le serveur de réconciliation, C-28-5-29) :
`failed-proposal-label.ts`, `failed-proposal-label.test.ts`, `sitesTotal` d'`i18n-keys.test.ts`, section
`reconciliation-*` des catalogues. Signale tout ce qui gênerait une **union** au rebase.

⛔ **Axes communs aux trois lentilles** (chacune les déclare exercés ou non) :

1. **Une seule définition de « rapprochée », et « rejetée » qui s'y adosse.** `isRejected` appelle-t-elle
   `isReconciled` sans seconde définition ? `!= null` (un champ absent n'est pas un rejet) ? Le composant ne rend-il
   rien hors d'une rejetée (C-28-5-37) ? Une transaction rapprochée **et** marquée (état forgé) reste-t-elle sans
   mention ni bouton ?
2. **Rôles.** La mention pour tous les rôles, le bouton pour Admin et Comptable seulement (`canManage` calculé par la
   page, passé au composant) ; le serveur refuse de toute façon (`403`). L'indication de la page *Réconciliation*
   est-elle vraie pour tout rôle (C-28-5-22) ?
3. **Le geste.** Bouton désactivé pendant l'appel, un double-clic n'envoie qu'une requête ; succès →
   `notifySuccess` puis relecture ; échec → `notifyError(errorMessageOf(e))` **et** relecture. Toute erreur affichée
   passe-t-elle par `errorMessageOf` (`frontend/src/lib/shared/utils/error-message.ts` ; garde
   `erreurs-en-texte.test.ts`) ? Le `204` est-il bien accepté par `apiClient.post<void>` ? Que se passe-t-il si la
   page est démontée pendant l'appel, ou si `load()` échoue après le succès ?
4. **L'indication (AC6).** En fin de `<section>`, après le bloc des échecs partiels, **hors de toute branche** :
   visible liste vide, chargée, en erreur, en chargement ?
5. **Textes et i18n.** Quatre clés neuves × quatre catalogues, `reconciliation-failed-already-rejected` réécrite dans
   les quatre ; chaque repli **égal mot pour mot** à la valeur fr-CH (apostrophe `’` comprise :
   `grep -nF "s’annule" frontend/src/lib/features/reconciliation/failed-proposal-label.ts crates/kesh-i18n/locales/fr-CH/messages.ftl`) ;
   libellés de menu cités entre parenthèses égaux à `nav-bank-import` dans chaque locale ; terminologie du glossaire
   (`docs/i18n-glossaire.md`) ; la date de la mention formatée par `Intl.DateTimeFormat(i18nLocale(), { dateStyle:
   'medium' })` — et le fuseau : une date UTC proche de minuit s'affiche-t-elle au bon jour ? `sitesTotal`
   **recompté** (`grep -o "i18nMsg(" <fichier> | wc -l` aux deux bornes `eab77cf1` et `31df9d74`) ;
   `lint-i18n-ownership` (les clés `reconciliation-*` ne vivent que sous `lib/features/reconciliation/`).
6. **Inventaire des sites non résolus** (Dev Notes, D4-ter) : la ligne « `frontend/.../bank-import/[id]/+page.svelte` —
   **résolu** (AC5) » est-elle vraie ? `grep -rn "autoMatchRejectedAt" frontend/src` : chaque site est-il à
   l'inventaire ou résolu par lui ? Les angles morts assumés (colonne *Statut* brute, page *Imports précédents*,
   limite de 100 suivie par #649) sont-ils toujours écrits ?
7. **Tests et décomptes.** V1-V6 existent-ils, prouvent-ils ce qu'ils nomment, rougissaient-ils avant la correction
   (`kesh-gate-logs/28-5b/red.log`) ? Les cinq mutations M9-M12, M16 (`kesh-gate-logs/28-5b/mutations-run.txt` et
   `mutations/`) : chacune tuée par le test que nomme la fiche ? Cherche une **mutation plausible qui resterait
   verte**. **Recompte depuis la source** tout décompte du Dev Agent Record et vérifie son **périmètre**.
8. ⛔ **LE MANUEL, `.tex` ET PDF, ET LE CHANGELOG.** La puce *Rejeter* de `docs/manual/fr/user-manual.tex`
   (§ *Accepter ou rejeter les propositions*) dit-elle **vrai au code** : la mention, le bouton, les rôles, le retour
   dans la liste sous la limite des 100 (#649), l'absence d'écriture, la trace d'audit *Rejet de réconciliation
   annulé* (`audit-log-action-reconciliation-unrejected`), l'indication sous la liste ? Les phrases « aucun geste de
   l'interface ne la remet ensuite dans la liste », « sans aucune action » et « Ne rejetez donc » ont-elles disparu ?
   § *Supprimer un import* (« une transaction rejetée ne compte pas », « rejetées comprises ») et *Un refus n'empêche
   pas les autres* restent-ils vrais ? Cherche **ailleurs** dans le manuel utilisateur, le manuel d'administration, le
   README, `website/` et `docs/api-external.md` une phrase que la story rend fausse. Contrôle le **PDF aplati et
   normalisé** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | sed "s/’/'/g; s/ﬀ/ff/g; s/ﬁ/fi/g; s/ﬂ/fl/g; s/ﬃ/ffi/g; s/ﬄ/ffl/g"`
   vers `/home/gcorbaz/devel/kesh-gate-logs/28-5b-review-p1-<lettre>-user-manual-flat.txt` — le PDF suit-il le
   `.tex` ? `CHANGELOG.md` : entrées *Ajouté* et *Corrigé* dans la **seule** section `## [0.13.3] — Non publié`,
   phrases prescrites par AC9 (b) mot pour mot (C-28-5-40), aucune promesse que le code ne tient pas.

## Lentilles

- **B — Blind Hunter : bugs et états**, sans présupposer que la fiche a raison. Réactivité Svelte 5 du composant
  (`$props`, `$derived` de la date quand `tx` change après la relecture, `$state` de `submitting` quand la ligne est
  recréée par le `{#each … (tx.id)}`) ; `new Date()` sur une chaîne invalide (« Invalid Date ») ; un rejet annulé par
  un autre onglet pendant l'affichage ; deux lignes rejetées annulées coup sur coup ; l'effet du champ neuf sur les
  autres lecteurs de `BankTransactionDto` (`DeleteBankImportAction`, `reconciledCount`) ; la spec E2E (nettoyage en
  `finally`, contexte d'API administrateur conservé après la connexion Consultation, utilisateur **désactivé**,
  C-28-5-36).
- **E — Edge Case Hunter : bords, et les tests prouvent-ils ce qu'ils nomment.** Les mocks d'`i18n.svelte` des tests
  (repli rendu, `i18nLocale` fixé à `fr-CH`) peuvent-ils rendre un test vert à tort ? V2 : l'assertion « bouton
  désactivé pendant » et « un appel » survivent-elles à une mutation qui retire `disabled` mais garde la garde
  `if (submitting) return`, ou l'inverse ? V3 : la ligne rapprochée **marquée** suffit-elle à tuer M9 par la page ?
  V4 : `compareDocumentPosition` prouve-t-il « après le bloc » ? V6 : la lecture du catalogue (`valeurFr`) passe-t-elle
  à vide si la clé est absente ? La spec E2E : passe-t-elle à vide si la ligne n'est jamais trouvée (`toHaveCount(0)`
  sur un localisateur qui ne correspond à rien) — l'assertion décisive « la mention disparaît » est-elle précédée
  d'une assertion qu'elle était là sur **la même** ligne ? Les captures : six, aux noms prescrits, hors dépôt ?
- **A — Acceptance Auditor : conformité, décomptes, documentation.** Chaque AC de la partie b contre le code :
  satisfait, partiel, absent. Les régions touchées sont-elles celles du tableau *Régions touchées* (partie b) ? Aucun
  fichier de `crates/` hors des quatre catalogues, aucune migration, aucun `Cargo.toml` (`git diff --stat`). Les
  chiffres de gates déclarés au Dev Agent Record correspondent-ils aux journaux (`kesh-gate-logs/28-5b/final-*.log`,
  `final-gate.out`) ? Les échecs E2E sont-ils **nommés** et jugés un par un contre `docs/testing.md` ? L'écart à la
  lettre de l'issue (pas de filtre « Rejetées », C-28-5-1) est-il écrit pour la PR ? Les choix C-28-5-36 à 40 :
  justifiés, ou défaut ?

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/28-5b-review-p1-<B|E|A>.md` (ta lettre). Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction proposée. Pour
tout finding affirmant qu'un code ou un texte est absent ou présent : la sortie d'un `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** (les huit axes communs et ceux de ta lentille) — un « 0 finding » sans
elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` (et le texte aplati du PDF au
même endroit) ; aucune commande qui écrit dans le dépôt ou dans une base, ni qui compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`,
`xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/`rebase`/`restore`/`push`/`tag`, `sqlx`, `cargo`
(aucune sous-commande), `rustc`, `npm`, `npx`, `node`, `vitest`, `playwright`, `deno`, `bun`, `python`/`python3`,
`perl`, `ruby`, `php`, `sed -i`, `awk` en écriture, tout autre interpréteur, `mariadb`, `mysql`, `docker`, `curl` et
`wget` (aucune requête HTTP), `gh` en écriture, `pkill`/`kill`, aucune requête SQL. Aucun fichier hors du dépôt.
Autorisés : lecture, `grep`, `sed -n`, `wc`, `sort`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers
`/home/gcorbaz/devel/kesh-gate-logs/`.
