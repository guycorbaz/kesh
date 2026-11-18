# Story 43-6 : « Changer le compte analytique » refusé sur une facture réglée en groupe — raccord avec la 42-1 (#823, #1167)

Status: ready-for-dev

*Fiche rédigée le 2026-11-17 au soir par l'agent de rédaction de l'Epic 43 (Opus 5.5), workflow `bmad-create-story`
(ex-T-R de la 43-1 ; validation P1, M5) ; **récrite dans la nuit** sur les décisions de l'orchestrateur : la 42-2 et la 42-3
sortent de la 0.19.0, un seul code de refus. Code lu à `origin/epic/42-reimputation-analytique` pour
`project_reassignment.rs`. Release **0.19.0**. **En dernier**, sur la branche réunie que désigne l'orchestrateur (42-1 et
43-1a fusionnées). Décision C-43-11. **Ne sort jamais** si la 42-1 et la 43-1a sont publiées ensemble (C-43-18).*

**Issues** : `refs #823`, `refs #1167`.

## Le problème

En 0.19.0, seul le geste **client** de la 42-1 existe. `reassign_invoice` (`project_reassignment.rs:410`) lit les écritures
de règlement de la facture (`SELECT journal_entry_id FROM invoice_settlements …`, :443-453), puis
`lock_entries_and_lines_in_tx` (:143), `entries_that_change` (:204), `period_exposure_in_tx` (:232) et
`reassign_lines_in_tx` (:352-363, `UPDATE journal_entry_lines SET project_id = ? WHERE entry_id IN (…)`) raisonnent **par
écriture**. Sur une écriture groupée (43-1a), il réimputerait la ligne de banque, les lignes de différence et les lignes des
autres factures, sans le dire.

## Story

**En tant que** Comptable ou Administrateur qui change le compte analytique d'une facture client après coup (geste de la
42-1), **je veux** que Kesh refuse ce geste sur une facture réglée avec d'autres par un même virement, avec un message qui
le dit, **afin que** la ligne de banque, les différences et les créances des autres factures du groupe ne changent pas de
compte analytique à mon insu.

## Décision

Le geste **refuse** une facture dont un règlement porte un identifiant de ligne :

- code **`PROJECT_REASSIGNMENT_GROUPED_SETTLEMENT`**, `409`, `details: { "invoiceId", "journalEntryId" }` — l'écriture
  groupée ; si la facture en a plusieurs, la plus petite `journal_entry_id` parmi ses règlements dont la ligne est non
  nulle ;
- **position** : dans `reassign_invoice`, **après le contrôle de version** de la facture (étape 2), **dans la lecture
  verrouillante des règlements de l'étape (3)**. Cette lecture devient
  `SELECT journal_entry_id, journal_entry_line_id FROM invoice_settlements WHERE invoice_id = ? AND company_id = ? ORDER BY
  id FOR UPDATE` (une seule requête, la discipline du module : « l'ensemble relu sous verrou, après la pièce ») ; le refus
  tombe **avant** la lecture des avoirs, `entry_ids_in_tx`, `lock_entries_and_lines_in_tx` (étape 4) et tout ce qui suit.
  Rien n'est écrit ; aucune ligne des autres factures n'est verrouillée ;
- **le refus précède donc tous les refus aval et le « rien à faire »** : une facture groupée reçoit le code de groupe, et
  non `200` (rien à changer), `404 NOT_FOUND` ou `409 ILLEGAL_STATE_TRANSITION` (compte visé inconnu ou archivé),
  `403 PROJECT_REASSIGNMENT_CLOSED_PERIOD_ADMIN_ONLY` ou `409 PROJECT_REASSIGNMENT_CLOSED_PERIOD` (période close). Le geste
  refuse **toute** facture groupée, même quand rien ne changerait ;
- **critère** : au moins un règlement de la facture a `journal_entry_line_id IS NOT NULL` — **la facture porte une ligne
  attribuée**, et non « l'écriture est partagée » : une facture qui ne serait plus qu'une parmi zéro autre sur l'écriture
  (cas fournisseur détaché, en 0.20.0) reste refusée plutôt que de réécrire l'écriture entière. En 0.19.0, le seul site qui
  écrit une ligne non nulle est le groupe client (`grouped_settlements.rs:784`) ; les trois autres appelants de
  `invoice_settlements::create_in_tx` écrivent `None` sur une écriture propre (validation P1, inventaire de l'ensemble clos) ;
- message traduit côté serveur dans les quatre locales (clé Fluent, repli Rust) ; phrase au manuel utilisateur (§ du geste
  de la 42-1) ; `docs/api-external.md` (route du geste et tableau général des codes). **L'écran n'a pas de code neuf** : le
  dialogue de la 42-1 affiche déjà `err.message` pour tout code autre que `PROJECT_REASSIGNMENT_CLOSED_PERIOD`
  (`ProjectReassignDialog.svelte:88-94`) ; aucune clé `i18nMsg` n'est ajoutée, et `sitesTotal` ne bouge pas.

**Après un dé-rapprochement** du groupe (`grouped_settlements::cancel_in_tx`, :1750 : lignes `invoice_settlements` de
l'écriture supprimées, écriture contre-passée), la facture n'a plus de règlement groupé et redevient réimputable ; ni
l'écriture groupée ni sa contre-passation ne lui sont plus reliées (`entry_ids_in_tx(…, false)`), elles ne bougent pas.

*Pour la 0.20.0 (hors de cette story)* : la 42-2 reprendra le même prédicat par ligne sur
`supplier_invoices.settlement_journal_entry_line_id`, y compris pour une facture **détachée** d'un groupe
(`kesh-gate-logs/e42/validation-42-2-p2.md`, V2-1) — refusée ou limitée à sa ligne, jamais l'écriture entière. Une
réimputation limitée à la ligne (plutôt qu'un refus) pourra alors être proposée pour les deux familles.

## Critères d'acceptation

- **AC1** — Une facture réglée dans un groupe : `409 PROJECT_REASSIGNMENT_GROUPED_SETTLEMENT`, `details` exacts
  (`invoiceId`, `journalEntryId` = l'écriture groupée), aucune ligne réécrite (empreinte des lignes de l'écriture groupée
  inchangée, `project_id` de la facture et des autres factures du groupe inchangés, `version` de la facture inchangée),
  aucune trace d'audit du geste.
- **AC2** — Un règlement unitaire est réimputé exactement comme par la 42-1 : ses tests passent sans modification.
- **AC3** — **Ordre amont** : le refus vient après le contrôle de version — une version périmée sur une facture groupée
  rend `409 OPTIMISTIC_LOCK_CONFLICT`, non le code neuf.
- **AC4** — **Ordre aval** : sur une facture groupée, chacun des cas suivants rend le code de groupe :
  - compte visé **archivé** (non `409 ILLEGAL_STATE_TRANSITION`) et compte visé **inconnu** (non `404 NOT_FOUND`) ;
  - écriture groupée dans un **exercice clos**, auteur **Comptable** (non `403
    PROJECT_REASSIGNMENT_CLOSED_PERIOD_ADMIN_ONLY`) ;
  - **rien à changer** — facture et toutes les lignes du groupe, banque comprise, sans compte analytique, cible `null` —
    (non `200`).
- **AC5** — **Groupe dé-rapproché** : après le dé-rapprochement d'un groupe de deux factures, la réimputation de l'une rend
  `200`, `journalEntryIds` = sa seule vente ; les lignes de l'écriture groupée et de sa contre-passation sont inchangées.
- **AC6** — Manuel FR, `docs/api-external.md` (deux tableaux), message serveur dans les quatre locales ; le dialogue affiche
  le message de l'API pour ce code, sans *Confirmer* ni avertissement de période close ; `sitesTotal` recompté et inchangé.
- **AC7** — **Recette visuelle** (`CLAUDE.md` § *Recette visuelle*) : capture du dialogue *Changer le compte analytique*
  ouvert sur une facture réglée en groupe, refus affiché (`data-testid="project-reassign-error"`), thème clair, base de
  démonstration, dans `/home/gcorbaz/devel/kesh-gate-logs/e436/captures/` ; jointe au Dev Agent Record et au prompt de revue.

## Tasks / Subtasks

- [ ] **T1 — Tests d'abord** (`crates/kesh-api/tests/project_reassignment_e2e.rs`, helpers `snapshot`, `line_projects`,
  `entry_versions`, `audits` ; groupe monté par `kesh_db::repositories::grouped_settlements::settle_invoices_in_tx`, comme
  `admin_full_import_e2e.rs:6487`, ou par la route comme `reconciliation_group_e2e.rs`) :
  - [ ] AC1 — groupe de deux factures, réimputation de l'une → refus, `details`, empreinte et audit inchangés ;
  - [ ] AC3 — version périmée sur facture groupée → `OPTIMISTIC_LOCK_CONFLICT` ;
  - [ ] AC4 — compte visé archivé, compte visé inconnu, exercice clos + Comptable, rien à changer → code de groupe ;
  - [ ] AC5 — groupe dé-rapproché (`POST /api/v1/reconciliation/transactions/{tx}/cancel`) puis réimputé → `200` ;
  - [ ] AC2 — les tests existants de la 42-1 passent sans modification.
- [ ] **T2 — Erreur** : `crates/kesh-db/src/errors.rs`, variante `DbError` neuve (près de `ProjectReassignmentClosedPeriod`,
  :2031) portant `invoice_id` et `journal_entry_id`, et son bras de `code()` (:2511) ; `crates/kesh-api/src/errors.rs`, bras
  du mappage `DbError → réponse` (près de :5643) : `409`, `details { invoiceId, journalEntryId }`, `t("error-project-
  reassignment-grouped-settlement", <repli = valeur fr-CH>)`.
- [ ] **T3 — Garde** : `project_reassignment.rs`, étape (3) de `reassign_invoice` (la lecture devient
  `journal_entry_id, journal_entry_line_id`, le refus tombe avant la lecture des avoirs) ; doc du module (« Ordre des verrous
  et des refus », :17-44) et doc de `reassign_invoice` (:387-408) : l'étape neuve et sa précédence sur « rien à faire ».
- [ ] **T4 — i18n** : clé `error-project-reassignment-grouped-settlement` dans `crates/kesh-i18n/locales/{fr-CH,de-CH,
  it-CH,en-CH}/messages.ftl`, bloc de la 36-16a. Gardes qui jouent : `textes_coherents.rs::les_replis_rust_suivent_le_catalogue`
  (:724) et `le_vocabulaire_analytique_est_compte_analytique` (G25, :2920 : ni « projet », « Projekt », « progett »,
  « project » dans les valeurs, ni « projet » au manuel et dans `api-external.md`) ; `SELECTEURS_RESOLUS_COTE_SERVEUR`
  (`kesh-i18n/src/loader.rs:383`) seulement si le message prend un sélecteur.
- [ ] **T5 — Écran** : aucun code neuf dans `ProjectReassignDialog.svelte` ; un test dans `ProjectReassignDialog.test.ts`
  (patron du test « 403 : le message de l'API, sans Confirmer ») prouve l'affichage du message pour ce code ; `sitesTotal`
  (`frontend/src/lib/shared/i18n-keys.test.ts:1153`) recompté, inchangé.
- [ ] **T6 — Documentation** :
  - `docs/api-external.md`, tableau de la route (:628-635) : rangée `PROJECT_REASSIGNMENT_GROUPED_SETTLEMENT | 409,
    details` **entre** `OPTIMISTIC_LOCK_CONFLICT` et « *(rien à faire)* » ; la phrase du `200` (:620) dit qu'une facture
    réglée en groupe est refusée même quand rien ne changerait ;
  - `docs/api-external.md`, tableau général des codes (:1248-1249) : rangée neuve, « *(Depuis la v0.19.0.)* » ;
  - `docs/manual/fr/user-manual.tex` § `sec:reimputation-analytique` (:4229), puce « Ce que le geste touche » (:4238-4242) :
    l'exception de la facture réglée avec d'autres par un même virement, et le dé-rapprochement qui la lève. Seul le
    manuel FR a ce paragraphe ; PDF non commités.
- [ ] **T7 — Recette visuelle** (AC7).
- [ ] **T8 — Gates** du prompt de développement ; mutations déclarées (ci-dessous).

## Tests et mutations

- **Garde retirée** : AC1 rougit (les lignes de banque et de l'autre facture changent de compte).
- **Garde déplacée** après l'étape (4), (5) ou (7) : AC4 rougit (`200`, `ILLEGAL_STATE_TRANSITION`/`NOT_FOUND` ou
  `…CLOSED_PERIOD_ADMIN_ONLY` au lieu du code de groupe).
- **Garde placée avant le contrôle de version** : AC3 rougit.
- **AC5** n'a pas de mutation propre de la garde : il prouve que le refus ne survit pas au groupe (un critère qui
  chercherait l'écriture groupée par un autre chemin que `invoice_settlements`, par exemple par les lignes de créance,
  le ferait rougir) ; il se déclare comme test de non-régression.
- **Écran** : le code traité comme `PROJECT_REASSIGNMENT_CLOSED_PERIOD` (avertissement et *Confirmer*) : le test du dialogue
  rougit.

## Note de version

- **Modifié** — « Changer le compte analytique » d'une facture réglée avec d'autres par un même virement est refusé, avec
  un message qui le dit (refs #823, refs #1167).

## Dev Notes

Sites à modifier (validation P1, V1-5) : `crates/kesh-db/src/errors.rs`, `crates/kesh-api/src/errors.rs`,
`crates/kesh-db/src/repositories/project_reassignment.rs` (code et doc), les quatre `messages.ftl`,
`docs/api-external.md` (deux endroits), `docs/manual/fr/user-manual.tex`, les tests
(`crates/kesh-api/tests/project_reassignment_e2e.rs`, `frontend/src/lib/features/projects/ProjectReassignDialog.test.ts`).
La route (`routes/invoices.rs:1336-1348`) ne traduit rien (`?` seul) et ne change pas ; le registre des routes
(`audit_route_registry.rs`) non plus : ni route ni action d'audit neuves. Aucune route fournisseur ni outil MCP de
réimputation n'existe en 0.19.0 ; une clé d'API passe par la même route et reçoit le même refus.

**Verrous** : le refus, dans l'étape (3), ne crée pas de cycle. Le geste prend `companies FOR UPDATE` d'abord ; le groupe
client prend `companies` en partagé (clé étrangère) avant d'insérer `invoice_settlements` et de mettre à jour `invoices`
(C-43-19). Si le geste passe d'abord, l'`UPDATE invoices … AND version = <lue>` du groupe échoue en `race_during_update`.

Petite : prête **mercredi soir** sur une branche de travail réunie.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-17 — Fiche rédigée (ex-T-R de la 43-1), deux issues (repli, raccord complet).
- 2026-11-17 (nuit) — Récrite : 42-2 et 42-3 hors 0.19.0 ; une seule issue, le refus
  `PROJECT_REASSIGNMENT_GROUPED_SETTLEMENT` après le contrôle de version, critère par ligne ; la 42-2 reprend le prédicat
  en 0.20.0.
- 2026-11-18 — Validation P1 (Opus, contexte frais, `kesh-gate-logs/e436/validation-p1.md`) : 8 findings, 0 CRITICAL,
  0 HIGH, 2 MEDIUM (V1-1 ordre aval non testé, V1-2 recette visuelle absente), 6 LOW (V1-3 à V1-8). Tous appliqués :
  énoncé de story, refus dans la lecture verrouillante de l'étape (3) et précédence sur « rien à faire » et les refus aval
  (AC4), AC5 groupe dé-rapproché, AC6 sans clé frontend, AC7 recette visuelle, tâches, sites réels, mutations. Prémisses
  relues au code de `753baf088`.
