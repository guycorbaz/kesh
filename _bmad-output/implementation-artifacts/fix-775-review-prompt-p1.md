# Revue de code P1 — issue #775 (solde progressif du grand livre au-delà de la première page)

Prompt versionné, commun aux trois lentilles de la passe 1. **Passe complète.** Modèle : **Sonnet**, contexte frais,
lecture seule. Une lentille par agent : **B**, **E** ou **A**.

## Objet

Worktree : `/home/gcorbaz/devel/kesh-775`, branche `fix/775-grand-livre-solde-pagine`, partie de `origin/main` au
commit `ed981c7d`.
Diff à relire : `git -C /home/gcorbaz/devel/kesh-775 diff ed981c7d...HEAD` (aplati : un seul diff, pas la suite des
commits).

**L'issue #775 (P1, erreur comptable).** `crates/kesh-report/src/general_ledger.rs`, fonction `generate` : lu avec
`offset` (`GET /api/v1/reports/general-ledger?offset=…`, `routes/reports.rs`, ouvert aux clés d'API ; bientôt l'outil
MCP `kesh_grand_livre`), chaque page repartait du solde d'ouverture de la **période**, sans les lignes sautées. Sur un
compte de résultat, une fin d'exercice tombant exactement à la frontière de deux pages n'était ni posée en
`fiscal_year_breaks` ni suivie d'une remise à zéro.

**Correction attendue** : le solde de départ d'une page = ouverture de la période + somme signée des lignes sautées,
**dans le même ordre de tri** que `fetch_lines` ; pour un compte de résultat, remises à zéro aux exercices franchis ;
exercice courant initialisé sur la dernière ligne sautée. **Choix retenu par le développeur** (à contester s'il est
faux) : parcours des lignes sautées, lues par `fetch_lines(…, 0, Some(offset.min(line_count)))`, passées dans le même
`Cumul` que les lignes rendues ; les ruptures franchies dans les lignes sautées ne sont pas rendues (elles
appartiennent aux pages précédentes).

Fichiers : `crates/kesh-report/src/general_ledger.rs`, `crates/kesh-report/tests/general_ledger.rs`, `CHANGELOG.md`.

Lectures utiles : `crates/kesh-report/src/opening.rs` (`signed`, `is_bilan`, `opening_from`),
`crates/kesh-api/src/services/rapports.rs` (fonction `grand_livre`, régimes affichage/export),
`crates/kesh-api/src/routes/reports.rs`, `crates/kesh-report/src/pdf.rs` et `csv.rs` (consommateurs du solde
progressif), `frontend/src/routes/(app)/reports/+page.svelte` (l'écran n'envoie pas `offset`),
`docs/api-external.md`, `docs/manual/fr/*.tex` (manuels : disent-ils quelque chose de la pagination du grand livre ?
**contrôler aussi le PDF**, aplati : `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`).

## ⛔ Interdits

- **N'exécute JAMAIS `scripts/prepare-release.sh`**, ni directement ni par aucun détour (il écrit : bump des dix
  crates, CHANGELOG). N'exécute pas `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt ; ni
  `cargo` sous aucune forme, ni `npm`, ni `npx`, ni `sqlx`, ni `docker`, ni `mariadb`/`mysql`, ni `make`/`latexmk`.
- N'écris, ne modifie, ne crée aucun fichier — **sauf ton rapport**. Aucune commande git qui écrit (`commit`,
  `checkout`, `switch`, `stash`, `reset`, `rebase`, `tag`, `push`), aucune commande `gh` qui écrit (pr, merge, issue,
  comment). Aucun `pkill`/`kill`.
- N'ouvre jamais `~/kesh-test/`.
- Commandes autorisées : `git diff`, `git show`, `git log`, `git ls-tree`, `grep`, `sed -n`, `cat`, `pdftotext`,
  lecture de fichiers.

## Lentille (une seule par agent)

- **B — Blind Hunter** : lis le diff comme un relecteur qui ne connaît pas le dépôt. Justesse comptable du solde
  progressif (signe par type de compte, remise à zéro, `closing_balance` d'une rupture) ; identité de l'ordre de tri
  entre lignes sautées et lignes rendues ; cohérence des doc-comments et du CHANGELOG avec le code ; les tests
  prouvent-ils ce qu'ils annoncent — peuvent-ils passer à vide (test muet), leurs valeurs attendues sont-elles
  recalculables à la main ? DRY : le `Cumul` partagé, les helpers de test (`post_fy`, `deux_exercices`,
  `section_paginee`, `lecture_par_pages`).
- **E — Edge Case Hunter** : parcours chaque branche et chaque bord. Au minimum : `offset` = 0, 1, `line_count − 1`,
  `line_count`, `> line_count`, négatif ; `limit` absent (export), 1, plafond ; compte sans ligne mais à ouverture non
  nulle ; compte de bilan / de résultat × période dans un seul exercice / à cheval / commençant en plein exercice ;
  rupture sur la **première** ligne de la page, sur la dernière ligne sautée, plusieurs ruptures dans les lignes
  sautées ; plusieurs écritures le même jour de part et d'autre d'une fin d'exercice ; lignes d'un même compte dans
  une même écriture ; plusieurs comptes demandés (l'offset vaut **par compte**) ; période sans exercice couvrant
  `from`. Coût : O(offset) par compte et par page — acceptable ? Concurrence : lignes sautées, page et totaux lus par
  trois requêtes hors transaction — le défaut est-il neuf ou préexistant ?
- **A — Acceptance Auditor** : confronte le livrable aux exigences ci-dessous, une à une, avec preuve :
  1. correction **à la cause** dans `generate` (pas un rafistolage à l'appelant) ;
  2. même ordre de tri que `fetch_lines` ; remises à zéro pour un compte de résultat ; exercice courant initialisé sur
     la dernière ligne sautée ;
  3. le choix (agrégat ou parcours) et sa raison écrits dans le message de commit ;
  4. quatre tests : continuité sur un compte de bilan ; compte de résultat franchissant un exercice **exactement** à la
     frontière de deux pages ; page en plein milieu d'un exercice ; équivalence N pages / lecture entière, ligne pour
     ligne ;
  5. deux mutations (retour à l'ouverture seule ; omission de la rupture à la frontière) tuées par le test visé — le
     journal est `/home/gcorbaz/devel/kesh-gate-logs/775-mutations.log` : la panique relevée vient-elle bien du test
     visé ? les mutations sont-elles les bonnes ?
  6. l'écran et l'export PDF/CSV : utilisent-ils `generate` avec `offset` ? (vérifier au code) ;
  7. d'autres rapports paginés (balance, journal, autres) ont-ils le même défaut ? (vérifier au code) ;
  8. manuels et `docs/api-external.md` : décrivent-ils la pagination du grand livre, et disent-ils vrai ?
  9. CHANGELOG : entrée *Corrigé* de la 0.16.0 citant #775, fidèle au code ;
  10. commits `refs #775`, **aucun mot-clé de fermeture** dans un message de commit :
      `git -C /home/gcorbaz/devel/kesh-775 log ed981c7d..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
      doit ne rien rendre.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/775-review-p1-<B|E|A>.md` (selon ta lentille) :

1. **Axes exercés** et **axes NON exercés** — liste explicite. Un « 0 finding » sans cette liste ne compte pas.
2. Findings numérotés `P1-<B|E|A>-n`, chacun avec sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier:ligne,
   description, preuve, correctif proposé. **Toute affirmation de présence ou d'absence d'un code s'appuie sur une
   sortie `grep -nF` copiée dans le rapport.** Un défaut préexistant, hors du diff, se signale comme tel (« hors
   périmètre, à tracer »), avec sa reproduction raisonnée.
3. Verdict : nombre de findings par sévérité.
