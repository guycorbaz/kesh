# Prompt — revue de code P1, Story 15-1b (les postes ouverts d'un compte, et les rapprochements proposés)

*Versionné le 2026-10-10. Trois lentilles (Sonnet), contexte frais chacune, lecture seule.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-15-1b`, branche `story/15-1b-vue-lignes-ouvertes`.
**Le diff à revoir** : `git diff 7ba3781c 8dd84606` (le commit de développement ; `7ba3781c` est `origin/main`, la 15-1b-0
fusionnée ; `HEAD` ne porte en plus que ce prompt). Fiche : `_bmad-output/implementation-artifacts/15-1b-vue-lignes-ouvertes.md`
— Définitions, Reçu, AC1 à AC12, Tasks et « Tests de T6 » (19 tests numérotés, le 8 déplacé), Dev Notes, et le **Dev Agent
Record**, qui déclare trois choix (C-15-1b-14 à 16, registre `epic-15-choix-autonomes.md`), douze mutations, des mesures
d'`EXPLAIN` et des décomptes. Règles : `CLAUDE.md` du dépôt — § « Exception `kesh-db` », § « Recompter ses propres comptes
rendus », § « Le prompt d'une passe doit NOMMER le manuel ». Issue : #518 (la story ne la ferme pas).

**Contexte** : story backend. `kesh-core::lettering::proposals` (moteur pur) ; `kesh-db/src/repositories/letterings/open_items.rs`
(module enfant de `letterings`, ré-exporté : `open_items`, `lettering_proposals`, constantes `OPEN_ITEMS_*_SQL`) ;
`letterings::is_letterable` / `letterable_account_ids` ; `invoice_settlements::amount_due_scalar_sql` (extraite d'`amount_due`) ;
`DbError::LetteringProposalsTooManyLines` → 422 ; deux routes `GET` dans `kesh-api/src/routes/letterings.rs` ;
`AccountResponse::new` et `letterable` sur les cinq handlers des comptes ; `LedgerLine.lettering_code` ; une clé i18n ×4 ;
`docs/api-external.md` et `CHANGELOG.md`. Frontend non touché (types laissés à la 15-1c).

## Lentilles

- **B — Blind Hunter : bugs, SQL, transactions**, sans présupposer que la fiche a raison. Les trois constantes de la requête
  A **ligne à ligne** : ordre des marqueurs et des liaisons dans `open_items` (six puis huit), la dérivée (`GROUP BY
  lettering_key`, `MAX(je2.entry_date)`, bornée au compte et à la société, sans filtre de date), le `LEFT JOIN` et la
  condition `g.lettered_on IS NULL OR g.lettered_on > ?` — l'invariant « `openTotal == balance` » est-il **vrai sur tout état
  atteignable** (groupe à cheval sur deux comptes ? impossible ? lettrage d'une ligne datée au-delà de `X` seule ? lignes
  à montant nul ? lignes d'un groupe dont une ligne appartient à une autre société ?) ; `FORCE INDEX` (nom exact de
  l'index, effet sur les tests) ; tri. La requête B : `invoice_states_sql` (forme scalaire, `EXISTS` des règlements), le
  choix de la facture (`client_invoice_of` : `invoice`, `settlement` → `invoice_id`) — la facture d'un règlement peut-elle
  être d'une autre société, ou annulée ; `DocumentState::of` et la précédence d'AC4. La transaction ouverte par `open_items`
  et `lettering_proposals` : `begin` sur une connexion prêtée, `rollback` — que devient-elle quand une `?` sort entre les
  deux (la connexion revient-elle au pool avec une transaction ouverte ?). Le moteur pur : égalité des montants
  (`Decimal` à échelles différentes), lignes à double montant, classement total, glouton, coût mémoire au plafond.
  `lettering_proposals` : le plafond après R5, `limit` (`usize`), `document` d'une ligne de proposition.
- **E — Edge Case Hunter : bords et états.** `asOf` vide, futur, antérieur à toute écriture ; `limit`/`offset` aux bornes
  (0, négatifs, 500, 501, `i64::MAX`) ; compte archivé lettrable ; compte devenu non lettrable ; compte sans aucune ligne ;
  page vide (offset au-delà du total) — la requête B s'exécute-t-elle sans écriture (`document_owners` et `open_period_rule`
  sur des listes vides) ; exercice inconnu de la règle (`Invariant`) ; origine de lettrage inconnue ; facture d'un règlement
  sans numéro ; facture héritée `paid_at` **et** créditée ; reste dû négatif (trop-perçu) ; 2 000 / 2 001 candidates ; paire
  contre-passation dont l'une des lignes est en période close. **Les tests prouvent-ils** ce qu'ils nomment (le test
  aurait-il échoué sans le code ? assertion de montage avant chaque assertion négative ?) : rejoue mentalement les douze
  mutations du Dev Agent Record et cherche une **mutation plausible qui resterait verte** (ex. : `reason` calculé autrement ;
  `inOpenPeriod` toujours vrai ; `lettered_on` d'un autre groupe ; `letterable` des réponses unitaires ; repli Rust du 422
  différent du FTL ; tri sans `fiscal_year_id` ; `document` d'une ligne de proposition).
- **A — Acceptance Auditor : conformité, décomptes, documentation.** Chaque AC (AC1 à AC12) satisfait ; chaque test 1 à 20
  (sauf 8) existe et prouve ce qu'il nomme — la table de correspondance est dans l'en-tête de
  `crates/kesh-db/tests/open_items.rs`, `crates/kesh-report/tests/open_items_invariant.rs`,
  `crates/kesh-api/tests/open_items_e2e.rs` et les `mod tests` de `proposals.rs` et `open_items.rs`. AC2 : la garde lexicale
  en liste blanche couvre-t-elle **toutes** les constantes de la requête A et refuserait-elle une jointure de pièce
  (`JOIN invoices`, `LEFT JOIN credit_notes`, sous-requête `EXISTS (SELECT … FROM invoices`) ? AC3 : `manuallyLetterable` et
  le filtre R5 passent-ils par `DocumentKind::blocks_manual_lettering` (aucune liste recopiée) ? AC8 : le Dev Agent Record
  décrit-il fidèlement les plans (relis le test ignoré `explain_plans`) ; l'écart à AC4 (forme scalaire au lieu des jointures
  dérivées) est-il couvert par AC8 et C-15-1b-16 ? AC10 : `docs/api-external.md` (sections « Les postes ouverts d'un compte »,
  « Les rapprochements proposés », § 7, § 10) dit-il **exactement** ce que le code fait (défauts, bornes, ordre des refus, rejet
  du `limit` non numérique, `amountDue` au centime, `document.type`) ; `CHANGELOG.md` ; gardes `textes_coherents.rs`
  (`la_forme_libre_nnnn_nom_est_juste_partout`) sur les textes neufs. AC11 : les cinq handlers. **Décomptes du Dev Agent
  Record recomptés** (28 tests neufs : 4 / 2 / 14 / 1 / 7 ; 3323 au gate = 3296 + 27 ; 9 modules métier ; `cas.len()` 11).
  **Manuel** : la story déclare le manuel inchangé ; vérifie-le par la valeur — `grep -n -i "lettr\|postes ouverts\|grand livre" docs/manual/fr/*.tex`
  et le PDF aplati (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers `/home/gcorbaz/devel/kesh-gate-logs/`) :
  décrit-il quelque chose (Grand livre, plan comptable, lettrage par l'API) que le code livré rend faux ?

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/15-1b-review-p1-<B|E|A>.md` (ta lettre). Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction proposée. Pour tout
finding affirmant qu'un code est absent ou présent : la sortie d'un `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` (et le texte aplati des PDF au même
endroit) ; aucune commande qui écrit dans le dépôt ou dans une base : `scripts/*` (dont `scripts/prepare-release.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/`rebase`,
`sqlx`, `cargo` (aucune sous-commande), `npm`, `npx`, `docker`, `gh` en écriture, aucune requête SQL. Autorisés : lecture,
`grep`, `sed -n`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers `/home/gcorbaz/devel/kesh-gate-logs/`.
