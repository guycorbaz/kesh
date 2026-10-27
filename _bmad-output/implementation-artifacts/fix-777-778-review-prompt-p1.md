# Revue de code P1 — issues #777 et #778 (grand livre : ouverture et clôture d'un compte de résultat à cheval sur plusieurs exercices)

Prompt versionné, commun aux trois lentilles de la passe 1. **Passe complète.** Modèle : **Sonnet**, contexte frais,
lecture seule. Une lentille par agent : **B**, **E** ou **A**.

## Objet

Worktree : `/home/gcorbaz/devel/kesh-777`, branche `fix/777-778-grand-livre-exercices`, partie de `origin/main` au
commit `782bbc2f`.
Diff à relire : `git -C /home/gcorbaz/devel/kesh-777 diff 782bbc2f...HEAD` (aplati : un seul diff, pas la suite des
commits). Le PDF `docs/manual/fr/user-manual.pdf` est binaire : lis-le aplati (ci-dessous).

**#777 (P1, erreur comptable).** `crates/kesh-report/src/general_ledger.rs`, `generate` : sur un compte de
**résultat**, l'exercice courant de `Cumul` valait `None` avant la première ligne. Quand la période commence dans
l'exercice N (l'ouverture porte le cumul de N avant `from`) et que la première ligne de la période tombe en N+1,
aucune rupture n'était posée : l'ouverture se reportait dans N+1. Exemple de l'issue (synthétique) : période
2025-06-01 → 2026-12-31, vente de 400 le 2025-03-01, de 100 le 2026-02-01 ; rendu `opening = 400`, soldes `[500]`,
aucune rupture ; attendu : une rupture qui clôt 2025 à 400, puis un solde de 100.

**#778 (P2).** `closing` d'un compte de résultat valait `opening + mouvements de la période`, sans les remises à
zéro du solde progressif (720 là où la dernière ligne affiche 120, test `rupture_d_exercice_a_la_frontiere_de_deux_pages`).

**Définition décidée (Guy, 2026-10-27, « corrige 777 et 778 avant la release 0.16.0 » ; formulée par
l'orchestrateur)** : sur un compte de résultat, le solde de clôture d'une période à cheval sur plusieurs exercices est
**le solde de l'exercice en cours à la fin de la période**, c'est-à-dire le dernier solde progressif, remis à zéro à
chaque fin d'exercice franchie ; le cumul de la période reste dans `totalDebit`/`totalCredit`. Compte de bilan :
inchangé.

**Choix du développeur** (à contester s'ils sont faux) :

1. `Cumul::new` reçoit l'exercice qui contient `from` (`fiscal_year_containing`, lu une fois par appel, plus par
   compte) — le même dont le premier jour borne l'ouverture (`opening_balance` le reçoit désormais au lieu de le
   relire).
2. « Exercice en cours à la fin de la période » = le **dernier exercice commencé au plus tard le `to`**
   (`fiscal_year_current_at`), et non « l'exercice qui contient `to` ».
3. `closing` (résultat, période qui franchit un exercice) = somme signée des lignes de la période dont
   `fiscal_year_id` = cet exercice (`fetch_totals`, deux `SUM(CASE …)` de plus) — même clé que les remises à zéro de
   `Cumul`. Sinon `opening + signed(totaux)`.
4. Quand l'exercice final n'est pas celui de la dernière ligne lue, `Cumul::termine` pose une **rupture finale**,
   datée du **premier jour** de l'exercice final, rendue sur la seule dernière page (celle qui porte la dernière
   ligne ; la première si la période n'a aucune ligne). C'est ce qui fait coïncider « dernier solde progressif » et
   « solde de l'exercice en cours », y compris quand cet exercice n'a aucune ligne (clôture 0).
5. Rendu : l'écran et le PDF ne posaient une rupture qu'**entre** deux lignes ; une rupture en tête de section
   (#777) ou finale (#778) n'aurait pas été vue. `LedgerSection::break_before(idx)` / `trailing_breaks()` (Rust,
   partagés par le PDF et le CSV) et leurs jumeaux TypeScript dans `GeneralLedgerView.svelte`. En tête de section, la
   rupture se reconnaît à sa **date** (= date de la première ligne du nouvel exercice) ; entre deux lignes, au
   changement d'exercice comme avant. Les ruptures finales sont « celles qu'aucune ligne ne place ».
6. Le CSV n'écrivait **aucune** rupture : il les écrit désormais comme lignes (sans débit ni crédit, solde de
   l'exercice clos dans `SoldeProgressif`), pour rester additionnable (clôture = ouverture + mouvements − Σ ruptures).
7. Le PDF remet le « À reporter » à zéro après une rupture (il reportait le solde de l'exercice clos).

Fichiers : `crates/kesh-report/src/general_ledger.rs`, `csv.rs`, `pdf.rs`, `crates/kesh-report/tests/general_ledger.rs`,
`frontend/src/lib/features/reports/GeneralLedgerView.svelte`, `GeneralLedgerView.test.ts`, `reports.types.ts`,
`CHANGELOG.md`, `docs/api-external.md`, `docs/manual/fr/user-manual.tex` (+ `.pdf`).

Lectures utiles : `crates/kesh-report/src/opening.rs` (`signed`, `is_bilan`, `opening_from`),
`crates/kesh-report/src/trial_balance.rs` (sa clôture, bornée à un exercice), `crates/kesh-api/src/services/rapports.rs`
(`grand_livre`), `crates/kesh-api/src/routes/reports.rs`, `crates/kesh-api/src/mcp/outils/grand_livre.rs` (l'outil
MCP rend `cloture` et `ruptures` tels quels — sa description dit-elle quelque chose de faux ?),
`crates/kesh-db/src/repositories/letterings/open_items.rs` (le lien « Postes ouverts » promet que leur total égale la
clôture — comptes lettrables seulement), le journal de mutations `/home/gcorbaz/devel/kesh-gate-logs/777-mutations.log`.
Manuel : `pdftotext /home/gcorbaz/devel/kesh-777/docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, puis
chercher « Période libre » et « Le CSV écrit » (le PDF typographie l'apostrophe en `’`).

## ⛔ Interdits

- **N'exécute JAMAIS `scripts/prepare-release.sh`**, ni directement ni par aucun détour (il écrit : bump des dix
  crates, CHANGELOG). N'exécute aucun script du dépôt (`scripts/test-fast.sh`, `scripts/mem-guard.sh`…), ni `cargo`
  sous aucune forme, ni `npm`, `npx`, `sqlx`, `docker`, `mariadb`/`mysql`, `make`/`latexmk`.
- **Aucune redirection** (`>`, `>>`, `<`, `tee`, here-doc) : compare par des commandes enchaînées (pipe) et lis vers
  la sortie standard seulement.
- **Ni `rm` sous aucune forme, ni `bc`.**
- N'écris, ne modifie, ne crée aucun fichier — **sauf ton rapport**, écrit avec l'outil d'écriture de fichiers. Aucune
  commande git qui écrit (`commit`, `checkout`, `switch`, `stash`, `reset`, `rebase`, `tag`, `push`), aucune commande
  `gh` qui écrit (pr, merge, issue, comment). Aucun `pkill`/`kill`.
- N'ouvre jamais `~/kesh-test/`.
- Commandes autorisées : `git diff`, `git show`, `git log`, `git ls-tree`, `grep`, `sed -n`, `cat`, `pdftotext`
  (vers `-`), `tr`, lecture de fichiers.

## Lentille (une seule par agent)

- **B — Blind Hunter** : lis le diff comme un relecteur qui ne connaît pas le dépôt. Justesse comptable : signe par
  type de compte, remise à zéro, `closing_balance` de chaque rupture, clôture d'un compte de résultat vs dernier solde
  progressif, compte de bilan inchangé. Cohérence des doc-comments (module, `Cumul`, `FiscalYearBreak.date`,
  `fetch_totals`), du CHANGELOG, de `docs/api-external.md` et du manuel avec le code. Les tests prouvent-ils ce qu'ils
  annoncent — peuvent-ils passer à vide (test muet), leurs valeurs attendues se recalculent-elles à la main ? DRY :
  la règle de placement des ruptures existe en Rust **et** en TypeScript — divergent-elles ?
- **E — Edge Case Hunter** : parcours chaque branche et chaque bord. Au minimum : `from` hors de tout exercice (avant
  le premier, ou dans un trou entre deux) ; `to` après le dernier exercice ou dans un trou ; `from` = premier jour
  d'un exercice ; période d'un seul exercice ; deux, trois exercices, avec et sans ligne dans celui du milieu, avec et
  sans ligne dans le dernier ; compte sans aucune ligne mais à ouverture non nulle (rupture finale ; la ligne « Aucun
  mouvement… Le solde d'ouverture reste dû » de l'écran devient-elle fausse ?) ; `offset` = 0, `line_count − 1`,
  `line_count`, `> line_count` (où va la rupture finale ?) ; `limit` absent (export), 1, plafond ; écran tronqué à 500 ;
  plusieurs comptes demandés ; filtre « compte sans mouvement ni solde » (`line_count == 0 && opening.is_zero()`) ;
  `unnatural_balance` calculé sur la nouvelle clôture ; plusieurs écritures le même jour de part et d'autre d'une fin
  d'exercice ; placement par **date** en tête de section — peut-il mal placer, doubler ou perdre une rupture (pages,
  troncature, données incohérentes `entry_date`/`fiscal_year_id`) ? Exercices qui se chevauchent : préexistant ou neuf ?
  Concurrence : totaux, lignes et ruptures lus hors transaction — neuf ou préexistant ?
- **A — Acceptance Auditor** : confronte le livrable aux exigences ci-dessous, une à une, avec preuve :
  1. #777 corrigé **à la cause** dans `generate`/`Cumul` : exercice courant initial = celui qui contient `from` ; une
     rupture est posée et **rendue** avant la première ligne de l'exercice suivant (écran, PDF, CSV, API, MCP) ;
  2. #778 : `closing` suit la définition ci-dessus pour un compte de résultat ; inchangé pour un compte de bilan ;
     indépendant de la page ;
  3. la définition est écrite au CHANGELOG (entrée *Corrigé* de la 0.16.0 citant #777 et #778), au manuel § *Grand
     livre* (PDF régénéré, contrôlé aplati) et dans `docs/api-external.md` ; aucun écran ni export n'affiche « Solde de
     clôture » d'une façon que la définition rend fausse (écran, PDF, CSV, MCP) ;
  4. tests (données synthétiques) : le cas de l'issue #777 ; un compte de résultat sur trois exercices ; un compte de
     bilan inchangé ; la clôture d'un compte de résultat à cheval sur deux exercices ; l'équivalence pagination /
     lecture entière (#775), restée verte ;
  5. deux mutations — retour à `current_fy = None` au départ ; `closing` recalculé comme cumul — tuées par le test
     visé, assertion relevée, diff nul après restauration : vérifier le journal `777-mutations.log` contre les tests
     (numéros de ligne, messages) ; les mutations sont-elles les bonnes ?
  6. consommateurs de `closing`, `opening` et `fiscal_year_breaks` : écran (`GeneralLedgerView.svelte`), PDF, CSV,
     outil MCP `kesh_grand_livre`, balance (concordance écrite au manuel § *Balance des comptes* : « ouverture et
     clôture sont celles du grand livre sur la même période » — encore vrai ?), postes ouverts (lien depuis l'en-tête) ;
  7. commits `refs #777` / `refs #778`, **aucun mot-clé de fermeture** dans un message de commit :
     `git -C /home/gcorbaz/devel/kesh-777 log 782bbc2f..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
     doit ne rien rendre.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/777-review-p1-<B|E|A>.md` (selon ta lentille) :

1. **Axes exercés** et **axes NON exercés** — liste explicite. Un « 0 finding » sans cette liste ne compte pas.
2. Findings numérotés `P1-<B|E|A>-n`, chacun avec sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier:ligne,
   description, preuve, correctif proposé. **Toute affirmation de présence ou d'absence d'un code s'appuie sur une
   sortie `grep -nF` copiée dans le rapport** ; toute affirmation sur le manuel, sur le PDF aplati. Un défaut
   préexistant, hors du diff, se signale comme tel (« hors périmètre, à tracer »), avec sa reproduction raisonnée.
3. Verdict : nombre de findings par sévérité.
