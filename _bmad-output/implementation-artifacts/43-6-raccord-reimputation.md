# Story 43-6 : « Changer le compte analytique » refusé sur une facture réglée en groupe — raccord avec la 42-1 (#823, #1167)

Status: review

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

- [x] **T1 — Tests d'abord** (`crates/kesh-api/tests/project_reassignment_e2e.rs`, helpers `snapshot`, `line_projects`,
  `entry_versions`, `audits` ; groupe monté par `kesh_db::repositories::grouped_settlements::settle_invoices_in_tx`, comme
  `admin_full_import_e2e.rs:6487`, ou par la route comme `reconciliation_group_e2e.rs`) :
  - [x] AC1 — groupe de deux factures, réimputation de l'une → refus, `details`, empreinte et audit inchangés ;
  - [x] AC3 — version périmée sur facture groupée → `OPTIMISTIC_LOCK_CONFLICT` ;
  - [x] AC4 — compte visé archivé, compte visé inconnu, exercice clos + Comptable, rien à changer → code de groupe ;
  - [x] AC5 — groupe dé-rapproché (`POST /api/v1/reconciliation/transactions/{tx}/cancel`) puis réimputé → `200` ;
  - [x] AC2 — les tests existants de la 42-1 passent sans modification.
- [x] **T2 — Erreur** : `crates/kesh-db/src/errors.rs`, variante `DbError` neuve (près de `ProjectReassignmentClosedPeriod`,
  :2031) portant `invoice_id` et `journal_entry_id`, et son bras de `code()` (:2511) ; `crates/kesh-api/src/errors.rs`, bras
  du mappage `DbError → réponse` (près de :5643) : `409`, `details { invoiceId, journalEntryId }`, `t("error-project-
  reassignment-grouped-settlement", <repli = valeur fr-CH>)`.
- [x] **T3 — Garde** : `project_reassignment.rs`, étape (3) de `reassign_invoice` (la lecture devient
  `journal_entry_id, journal_entry_line_id`, le refus tombe avant la lecture des avoirs) ; doc du module (« Ordre des verrous
  et des refus », :17-44) et doc de `reassign_invoice` (:387-408) : l'étape neuve et sa précédence sur « rien à faire ».
- [x] **T4 — i18n** : clé `error-project-reassignment-grouped-settlement` dans `crates/kesh-i18n/locales/{fr-CH,de-CH,
  it-CH,en-CH}/messages.ftl`, bloc de la 36-16a. Gardes qui jouent : `textes_coherents.rs::les_replis_rust_suivent_le_catalogue`
  (:724) et `le_vocabulaire_analytique_est_compte_analytique` (G25, :2920 : ni « projet », « Projekt », « progett »,
  « project » dans les valeurs, ni « projet » au manuel et dans `api-external.md`) ; `SELECTEURS_RESOLUS_COTE_SERVEUR`
  (`kesh-i18n/src/loader.rs:383`) seulement si le message prend un sélecteur.
- [x] **T5 — Écran** : aucun code neuf dans `ProjectReassignDialog.svelte` ; un test dans `ProjectReassignDialog.test.ts`
  (patron du test « 403 : le message de l'API, sans Confirmer ») prouve l'affichage du message pour ce code ; `sitesTotal`
  (`frontend/src/lib/shared/i18n-keys.test.ts:1153`) recompté, inchangé.
- [x] **T6 — Documentation** :
  - `docs/api-external.md`, tableau de la route (:628-635) : rangée `PROJECT_REASSIGNMENT_GROUPED_SETTLEMENT | 409,
    details` **entre** `OPTIMISTIC_LOCK_CONFLICT` et « *(rien à faire)* » ; la phrase du `200` (:620) dit qu'une facture
    réglée en groupe est refusée même quand rien ne changerait ;
  - `docs/api-external.md`, tableau général des codes (:1248-1249) : rangée neuve, « *(Depuis la v0.19.0.)* » ;
  - `docs/manual/fr/user-manual.tex` § `sec:reimputation-analytique` (:4229), puce « Ce que le geste touche » (:4238-4242) :
    l'exception de la facture réglée avec d'autres par un même virement, et le dé-rapprochement qui la lève ;
  - `docs/manual/fr/user-manual.tex:3575-3577` (règlement groupé, *Défaire* et *Limites*) : la phrase « Le changement de
    compte analytique d'une facture réglée en groupe est refusé en version~0.19.0 » devient vraie avec la story ; y ajouter
    le renvoi à `\S\ref{sec:reimputation-analytique}` et la levée par l'annulation du rapprochement. **Les deux sites se
    relisent ensemble** (validation P2, V2-1). Seul le manuel FR traite de la réimputation ; PDF non commités.
- [x] **T7 — Recette visuelle** (AC7).
- [x] **T8 — Gates** du prompt de développement ; mutations déclarées (ci-dessous).

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

**Verrous** : aucun verrou neuf ; la lecture des règlements est celle de la 42-1, avec une colonne de plus. Les cycles
connus avec `cancel_in_tx` (dé-rapprochement du groupe, qui verrouille les règlements avant la facture) et avec le groupe
client sont bornés par le rejeu des routes `Rejouee` (doc du module, :40-44) ; la 43-6 ne les aggrave pas (validation P2,
V2-2). Un geste qui refuse n'écrit rien, donc ne fait échouer aucun groupe concurrent ; seul un geste qui réussit (facture
sans règlement groupé) incrémente la version de la facture.

Petite : prête **mercredi soir** sur une branche de travail réunie.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 — deux agents : le premier (Phase A, Phase B jusqu'à `cba56143e`), interrompu à 10 h 46 par
`systemd-oomd` (station tuée : construction Rust, construction frontend et E2E d'un autre projet ensemble) ; le second,
la reprise (`kesh-gate-logs/e436/reprise-prompt.md`), de `8a75833ec` au passage en revue.

### Completion Notes List

- **Code** (commit `600b9d75f`) : variante `DbError::ProjectReassignmentGroupedSettlement { invoice_id, journal_entry_id }`
  et son code ; bras `409` du mappage avec `details { invoiceId, journalEntryId }` et `t("error-project-reassignment-
  grouped-settlement", repli fr-CH)` ; dans `reassign_invoice`, la lecture verrouillante de l'étape (3) rend
  `journal_entry_line_id` et refuse avant la lecture des avoirs (`project_reassignment.rs`, « (3 bis) ») ; doc du module et
  de la fonction. Clé Fluent dans les quatre locales ; `docs/api-external.md` (phrase du `200`, rangée de l'ordre des refus,
  tableau général) ; manuel FR, deux sites (§ règlement groupé, *Limites* ; § `sec:reimputation-analytique`, puce neuve).
  Aucun code neuf à l'écran ; `sitesTotal` inchangé (`3495` à `i18n-keys.test.ts:1153`, fichier non touché par la branche).
- **Tests** : `project_reassignment_e2e.rs` passe de 27 à 32 `#[sqlx::test]` (`ff79da71c` → `8a75833ec`), sans ligne
  retirée (AC2) : `a_grouped_settlement_is_refused` (AC1), `a_stale_version_comes_before_the_grouped_refusal` (AC3),
  `the_grouped_refusal_comes_before_target_and_closed_period` et `a_grouped_invoice_is_refused_even_when_nothing_would_change`
  (AC4), `an_uncancelled_group_lets_the_invoice_move_again` (AC5). G9 (`les_replis_rust_suivent_le_catalogue`) reçoit la clé
  (`65c15ff62`). Vitest : `ProjectReassignDialog.test.ts` de 6 à 7 cas. E2E : `project-reassignment.spec.ts` de 4 à 5 tests,
  le refus réel du serveur à l'écran (`cba56143e`) ; `createManualCreditStatementViaApi` extrait de
  `reconciliation-group.spec.ts` vers `helpers/api-fixtures.ts` (DRY).
- **Rouges** (`kesh-gate-logs/e436/rouges.log`, avant la garde) : 5 tests, 3 rouges (AC1, les deux d'AC4), 2 verts attendus
  (AC3 et AC5 sont des non-régressions sur le code d'origine).
- **Mutations** (`kesh-gate-logs/e436/mut/`, script `muter-e436.py`, lanceur `run-rust-e436.sh`, restauration
  `git restore --worktree` + `touch`) — toutes tuées :
  - M1, garde retirée : AC1 et AC4 rougissent (3 tests) ;
  - M2a, garde après (7) : AC4 rougit (`404` au lieu du code de groupe ; `200` sur « rien à faire ») ;
  - M2b, garde après « rien à faire » : AC4 « rien à faire » rougit (`200`) ;
  - M2c, garde entre (5) et (6) (reprise) : AC4 rougit (`404`, `200`) ;
  - M2d, garde après (7) et compte visé neutralisé (reprise, isole la période close) : AC4 rougit (`403` au lieu de `409`) ;
  - M3, garde avant la version : AC3 rougit (code de groupe au lieu de `OPTIMISTIC_LOCK_CONFLICT`) ;
  - M4 (écran), le code traité comme la période close : le test Vitest rougit ;
  - M5 (repli Rust divergent du catalogue) : **survivante** avant `65c15ff62` (`M5.log`, G9 ne listait pas la clé), tuée
    après (`M5-bis.log`).
  AC5 n'a pas de mutation propre (fiche, « Tests et mutations ») : non-régression déclarée.
- **Recette visuelle** (AC7) : `/home/gcorbaz/devel/kesh-gate-logs/e436/captures/43-6-refus-groupe.png` — dialogue
  *Changer le compte analytique* sur une facture réglée en groupe, refus affiché dans `project-reassign-error`, ni
  avertissement ni *Confirmer* ; thème clair, base jetable de l'E2E (`kesh_e2e_e436`), prise par l'E2E complet. Artefact de
  la capture pleine page : le voile du dialogue ne couvre que la hauteur de la fenêtre (le bas de la page reste net) —
  propre à `fullPage: true`, non à l'écran.
- **Reprise** : bases `kesh_e436`, `kesh_e2e_e436` (voie 2) recréées, migrées (95 migrations du worktree), `kesh_e436`
  semée ; `kesh_ci_e436` recréée par `gate-ci.sh`. Aucune erreur d'artefact cargo : rien n'a été nettoyé dans `target/`.
  Attente de mem-guard (mybibli) de 10 h 59 à 11 h 20. Toutes les constructions en `-j 2`, une commande lourde à la fois.
- **Gates au passage en revue**, tous au commit `8a75833ec` :
  - `scripts/test-fast.sh --ci`, voie 2 (`kesh-gate-logs/e436/test-fast-reprise.log`) : **6132 tests, 6132 passés**
    (1 lent), 14 ignorés ;
  - `scripts/gate-ci.sh kesh_ci_e436`, voie 1 (`gate-ci-reprise.log`) : **6149 passés, 0 échoué, 18 ignorés** ;
  - frontend (`front-*-reprise.log`) : `check` 0 erreur (29 avertissements, 5 fichiers) ; `lint-i18n-ownership` PASS ;
    `test:unit` **278 fichiers, 4612 tests passés** ; `build` vert ;
  - E2E complet, voie 2, port 3436, base recréée vide (`e2e-reprise.log`, script `e2e-complet-e436.sh`) : **481 passés,
    7 échoués, 17 sautés** ; les sept échecs sont exactement ceux de la KF-029 (#97) de `docs/testing.md` § *Les échecs
    attendus* (`mode-expert.spec.ts:26`, `:41` ; `onboarding-path-b.spec.ts:65`, `:92` ; `onboarding.spec.ts:57`, `:77`,
    `:150`) ; le test neuf passe (`project-reassignment.spec.ts:297`).
  - **Pas encore tourné** : la revue de code ; le `gate-ci` et l'E2E de clôture se rejoueront au dernier commit de code de
    la boucle de revue.

### File List

- `crates/kesh-db/src/errors.rs`
- `crates/kesh-db/src/repositories/project_reassignment.rs`
- `crates/kesh-api/src/errors.rs`
- `crates/kesh-api/tests/project_reassignment_e2e.rs`
- `crates/kesh-api/tests/textes_coherents.rs`
- `crates/kesh-i18n/locales/fr-CH/messages.ftl`
- `crates/kesh-i18n/locales/de-CH/messages.ftl`
- `crates/kesh-i18n/locales/it-CH/messages.ftl`
- `crates/kesh-i18n/locales/en-CH/messages.ftl`
- `docs/api-external.md`
- `docs/manual/fr/user-manual.tex`
- `frontend/src/lib/features/projects/ProjectReassignDialog.test.ts`
- `frontend/tests/e2e/helpers/api-fixtures.ts`
- `frontend/tests/e2e/project-reassignment.spec.ts`
- `frontend/tests/e2e/reconciliation-group.spec.ts`
- `_bmad-output/implementation-artifacts/43-6-raccord-reimputation.md`
- `_bmad-output/implementation-artifacts/43-6-raccord-reimputation-review-prompt-p1.md`
- `_bmad-output/implementation-artifacts/e0190-choix-autonomes-e436.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/planning-artifacts/epic-43-transaction-plusieurs-factures.md` (Phase A)

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
- 2026-11-18 — Validation P2 (Sonnet, contexte frais, `kesh-gate-logs/e436/validation-p2.md`) : 0 CRITICAL, 0 HIGH,
  0 MEDIUM, 2 LOW (V2-1 second site du manuel `user-manual.tex:3575-3577` ajouté à T6 ; V2-2 Dev Notes *Verrous*
  reformulées). **Validation terminée** sur cette passe. Tendance : P1 (Opus) 0/0/2/6 → P2 (Sonnet) 0/0/0/2 ;
  aucun reclassement.
- 2026-11-18 — Développement (`600b9d75f`, `65c15ff62`, `cba56143e`), interrompu à 10 h 46 par le plantage de la station ;
  reprise : espace de `projects-reassign-open` rétabli dans les quatre catalogues et capture renommée `43-6-refus-groupe`
  (`8a75833ec`, C-e436-5) ; mutations M2c et M2d ajoutées (C-e436-6) ; gates du passage en revue verts (Dev Agent Record) ;
  statut `review`. Gate-ci et E2E de clôture à rejouer au dernier commit de code.
