# Prompt — revue de code P1, Story 15-1a2-0 (le lettrage se fige avec la période)

*Versionné le 2026-10-09. Trois lentilles (Sonnet), contexte frais chacune, lecture seule.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-15-1a2-0`, branche `story/15-1a2-0-lettrage-fige-avec-la-periode`.
**Le diff à revoir** : `git diff f9b6b199 9b51bab1` (le commit de développement ; la fiche validée est à
`f9b6b199`). Fiche : `_bmad-output/implementation-artifacts/15-1a2-0-lettrage-fige-avec-la-periode.md` — D1 à D5,
AC1 à AC9 (AC8 déplacé), Dev Notes, et le **Dev Agent Record**, qui déclare deux choix (C-15-1a2-0-1, -2) et
des décomptes. Règles : `CLAUDE.md` du dépôt. Issue : #518 (la story ne la ferme pas). Registre des choix :
`_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`.

**Contexte** : le rang 2 bis est **dormant** — aucun chemin de production ne pose encore de groupe de lettrage
d'origine `document` (15-1a2-i le fera) ; les tests les posent en SQL brut par
`crates/kesh-db/tests/support/document_group.rs`. Toute la documentation publique du refus (manuels,
`api-external.md`, CHANGELOG) est **délibérément** portée par la 15-1a2-i (AC18, C-15-1a2-24).

## Lentilles

- **B — Blind Hunter : bugs, concurrence, SQL (`kesh-db`, `kesh-api`)**, sans présupposer que la fiche a
  raison. `letterings.rs` : `OpenPeriodRule`, `open_period_rule` (identifiants dupliqués, liste vide, exercice
  d'une autre société, `find_later_closed` appelé sur la bonne date), `line_in_open_period`,
  `lines_in_open_period`, `document_group_frozen_by_periods` (requêtes, filtre de société, ordre des clés,
  groupe vide, plusieurs groupes sur une écriture) ; le prédicat partagé `line_open_in_period` et le mode
  `Manual` (comportement inchangé ?) ; `books_locked_through` passé de `Transaction` à `MySqlConnection`.
  `settlement_cancellation.rs` : place du rang 2 bis (après le 2, avant le 3), lecture **sans verrou** — la
  tolérance écrite (D4) est-elle la seule fenêtre ? une lecture ordinaire posée ici change-t-elle l'instantané
  `REPEATABLE READ` d'un geste avant un verrou qu'il prend ensuite (relire l'ordre des verrous de
  `cancel_settlement_in_tx`, `reconciliation_cancel::cancel_in_tx` — « aucune lecture non verrouillante avant
  l'étape 3 » —, `supplier_invoices::cancel_in_tx` et `cancel_settlement_in_tx`) ? Les quatre filtres (D3) :
  motifs liés, l'erreur rendue, le dé-rapprochement refuse-t-il **avant** de défaire le lien ? Le mapping
  (`kesh-api/src/errors.rs`) : statut 409, code, clé par famille ; un chemin peut-il finir en 500 ?
- **E — Edge Case Hunter : bords, écran, i18n.** Les bords de la règle : borne égale à la date (stricte),
  borne `NULL`, exercice clos, exercice suivi d'un clos, groupe à cheval sur deux exercices, ligne la plus
  récente qui n'est pas le dernier règlement. Les gestes : facture soldée par règlement + solde (rang 1 bis
  avant 2 bis), facture fournisseur payée, dé-rapprochement d'une écriture `Entry` héritée (D3, exception
  nommée). Les douze textes (`crates/kesh-i18n/locales/*/messages.ftl`) : identiques à la fiche D2/D5 ? chaque
  locale porte le marqueur d'ordre (G8) et la borne « jusqu'à celui-ci » (G8-bis) ; les replis Rust et Svelte
  disent **mot pour mot** le fr-CH ; « sa facture » / « son paiement » vrais dans chaque famille. Le frontend :
  `settlement-cancel-blocked.ts`, `reconciliation-cancel.ts` (+ `reconciliation.types.ts`, `MOTIFS`),
  `invoice-cancel.ts` — union exhaustive (`never`), un code non reconnu affiche-t-il le message serveur ? Les
  gardes recomptées : `CLES_RELEVEES` 214, `sitesTotal` 1923, `REPLIS_A_SITE_UNIQUE` (+3) — **recompte-les
  depuis la source** (`grep -o "i18nMsg("` aux deux bornes, `git show f9b6b199:<fichier>`).
- **A — Acceptance Auditor : conformité à la fiche et au manuel.** Chaque AC (AC1–AC7, AC9) est-il satisfait,
  chaque test listé à « Tests prévus » existe-t-il, porte-t-il le nom prévu, et **prouve-t-il** ce qu'il nomme
  (aurait-il échoué avant le patch ? une assertion de montage protège-t-elle les assertions négatives ?) ? Les
  « tests existants touchés » de la fiche sont-ils traités (le `match` d'`ecriture_attendue`, les `_ => false`
  des bancs fournisseurs) ? La liste des **sites nommés** de D2 : chaque site réécrit ou trié au Dev Agent
  Record ? Les paires exclues (InvoiceCredited × 2 bis, tête `SupplierInvoiceNotPaid` × 2 bis, rang 6 × 2 bis)
  sont-elles **nommées** dans les tests avec leur raison ? Les deux choix consignés sont-ils justifiés ? Les
  décomptes du Dev Agent Record (12 tests Rust neufs, Vitest, `sitesTotal`, sites AC8) se recomptent-ils depuis
  la source ? **Le manuel** : `docs/manual/fr/user-manual.tex` **et le PDF aplati** (`pdftotext
  docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers `/home/gcorbaz/devel/kesh-gate-logs/`) —
  la story n'y touche pas (C-15-1a2-24) : le manuel, tel qu'il est, contredit-il le code livré **dans un état
  atteignable par l'utilisateur** ? (le rang étant dormant, une liste de motifs incomplète n'est pas une
  contradiction atteignable — dis-le explicitement si tu le conclus, ou montre l'état atteignable). Le message
  neutre de `LETTERING_IS_DOCUMENT` est-il cité ailleurs (docs, manuels) sous son ancienne forme ?

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/15-1a2-0-review-p1-<B|E|A>.md` (ta lettre). Findings avec
sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction
proposée. Pour tout finding affirmant qu'un code est absent ou présent : la sortie d'un `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` (et le texte aplati du
PDF au même endroit) ; aucune commande qui écrit dans le dépôt ou dans une base : `scripts/*` (dont
`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`, `git commit`/
`add`/`checkout`/`switch`/`stash`/`reset`, `sqlx`, `cargo` (aucune sous-commande), `npm`, `npx`, `docker`, `gh`
en écriture, aucune requête SQL. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh issue view`,
`pdftotext` vers `/home/gcorbaz/devel/kesh-gate-logs/`.
