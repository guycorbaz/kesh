# Story 36-16a : changer le compte analytique d'une facture client, même payée — le patron du geste (#1167)

Status: ready-for-dev

*Fiche rédigée le 2026-11-15 par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-16`, branche
`story/36-16-reimputation-analytique` partie de `epic/36-0.18.0`, par le workflow `bmad-create-story`. **Première des trois
stories nées du découpage de la 36-16** (`36-16-reimputation-analytique.md`, `Status: split` ; C-36-16-10) : elle **pose le
patron** — dépôt, route, dialogue partagé, audit, avertissement et accusé — sur la facture client ; 36-16b (facture fournisseur)
et 36-16c (écriture manuelle) partent d'elle fusionnée. Phase 2, release **0.18.0**. **Aucune migration** (D0). **`kesh-db`
touché : gate complet même en cours de boucle.** Voie **1** (`kesh-mariadb-dev:3306`, bases `kesh_e3616a`, `kesh_e2e_e3616a`,
`kesh_ci_e3616a`). Code lu à **`fed34a9c7`**. Registres : `e36-choix-autonomes-e360.md` (C-e360-3, -11, -15, -16) ;
`e36-choix-autonomes-36-16-reimputation-analytique.md` (**C-36-16-1 à -10, communs aux trois stories**). Validation non lancée.*

**Issues** : la PR porte `refs #1167` ; aucun mot-clé de fermeture dans un commit (#1167 se ferme avec la dernière des trois).
**Vocabulaire** : la story part de `epic/36-0.18.0` **après la fusion de la 36-15** (#1161), dont elle reprend les termes
(« compte analytique » en fr-CH ; de-CH, it-CH, en-CH **en attente de l'arbitrage de Guy**, D2 de la fiche 36-15).

## Story

En tant que titulaire qui suit ses revenus par compte analytique, je veux **changer le compte analytique d'une facture client
quel que soit son statut** — payée, annulée, d'un exercice clos compris —, d'un seul geste, afin que la facture **et toutes ses
écritures** (vente, règlements, solde du reste, avoirs) passent au nouveau compte sans défaire de règlement ni de rapprochement.
Le geste ne touche ni montant, ni compte du plan, ni date, ni numéro, ni lettrage, ni rapprochement ; il est tracé au journal
d'audit, acteur compris (utilisateur ou clé d'API).

## Ce qui se lit au code (`fed34a9c7`)

*L'inventaire complet, par le symptôme, est dans la fiche d'origine (`36-16-reimputation-analytique.md`, § *Ce qui se lit au
code*) et au journal `kesh-gate-logs/e36-16/inventaire-fiche.log` (sonde positive `invoices.rs:2446`). Ce qui sert ici :*

- **Colonnes** (aucune migration) : `invoices.project_id` (`migrations/20260703000001_invoices_project.sql`),
  `journal_entry_lines.project_id` (`20260702000001_projects_analytics.sql`), `BIGINT NULL`, `ON DELETE RESTRICT`. Aucun déclencheur.
  Les rapports lisent la **ligne**, en net (`kesh-report/src/project_report.rs:263-271`).
- **Aucun `UPDATE journal_entry_lines … project_id`** n'existe (`git grep -n "UPDATE journal_entry_lines"` : `letterings.rs:1204`,
  `:1409`, lettrage seul) : la story écrit le premier.
- **Les écritures d'une facture client et où elles prennent leur projet** :
  - vente — `invoices.rs:2443-2446` (`project_id: invoice_before.project_id,`), liée par `invoices.journal_entry_id` ;
  - règlement — `invoice_settlements_write.rs:348`, projet lu sous verrou `:143-145` (« … FROM invoices WHERE id = ? AND company_id
    = ? FOR UPDATE ») ; liée par `invoice_settlements.journal_entry_id` ;
  - solde du reste — `invoice_settlements_write.rs:712`, lu sous verrou `:553-555`, lié par `invoice_settlements::create_in_tx`
    (`:728-730`) ;
  - avoir — `credit_notes.rs:718` (`project_id: invoice.project_id,`), facture verrouillée d'abord (`:309`, « (1) invoices FOR
    UPDATE ») ; lié par `credit_notes.journal_entry_id` ; facture `cancelled` par avoir total (`credit_notes.rs:790`) ;
  - encaissement par rapprochement — `routes/reconciliation.rs:2814` (`project_id: invoice.project_id,`), puis
    `invoice_settlements::create_in_tx` (`:2843-2845`) — **lecture sous verrou non établie** (étape 0, D9).
- **Ce qui délie** : l'annulation d'un règlement supprime sa ligne (`invoice_settlements_write.rs:1043`, « DELETE FROM
  invoice_settlements WHERE id = ? AND company_id = ? ») ; la dévalidation supprime la vente (`invoices.rs:1678`, « journal_entry_id
  = NULL »). Contre-passation : `journal_entries.reverses_entry_id` (`20260828000002_journal_entries_reversal.sql:44`), lignes
  recopiées dans l'ordre (`journal_entries.rs:2921-2951`).
- **Exercices et verrou** : `fy_status == "Closed"` (`journal_entries.rs:1280-1282`), exercice postérieur clos (`:1283-1288`,
  `fiscal_years::find_later_closed`), `companies.books_locked_through` (`:1293-1300`, seuil inclusif `:1602-1610`). Tous applicatifs.
- **Réemploi** : `projects::validate_taggable_in_tx` (`projects.rs:89` ; `404` / `409 ILLEGAL_STATE_TRANSITION`,
  `routes/journal_entries.rs:888-903`) ; ordre « companies → projects → invoices » (`invoices.rs:1128-1133`) ; `Acteur`
  (`entities/audit_log.rs:173`), `From<&CurrentUser>` (`kesh-api/src/audit.rs:18`), `NewAuditLogEntry::par` (`audit_log.rs:263`) ;
  `audit_labels.rs` (`:150`, `message_key` `:229-231`, garde `tests/audit_label_registry.rs`) ; `tests/audit_route_registry.rs`
  (`:516`) ; `comptable_routes` (`lib.rs:536`) ; `retry_on_deadlock` (`routes/invoices.rs:898`).
- **Écran** : `canManage` (`routes/(app)/invoices/[id]/+page.svelte:92-94`), libellé `:1279-1285` (`data-testid="invoice-project"`) ;
  sélecteur du brouillon `InvoiceForm.svelte:811-830` (option « — Aucun », repli d'un projet hors liste).
- **Manuel** : `user-manual.tex:3766` (section analytique), `:3779-3790` (affectation) ; rien sur un changement après coup.
  **API** : `docs/api-external.md:1007` (le `PUT` du brouillon), § 10 `:1034`.

## Décisions

*Les décisions D0 à D11 sont **communes aux trois stories** (C-36-16-1 à -9) ; 36-16b et 36-16c y renvoient.*

- **D0 — Sans migration.** Si le développement en découvre le besoin, il s'arrête et le dit (C-e360-3 : la story passe à la 0.19.0).
- **D1 — Une route par objet, `POST …/{id}/reassign-project`** (C-36-16-1), dans `comptable_routes`, clés `read-write`, pas MCP.
  Ici : `POST /api/v1/invoices/{id}/reassign-project`, corps `{ "projectId": n | null, "version": n, "acknowledgeClosedPeriods"?:
  bool }`, réponse `{ invoice, journalEntryIds }`. Identifiants `project*` gardés (C-e360-15). **Contrat de l'API : ajout seulement.**
  *Écartées* : le `PUT` étendu aux validées ; une route sur les lignes ; un chemin `…/analytic-account`.
- **D2 — Les écritures d'une pièce** (C-36-16-2) : celles qui lui sont reliées aujourd'hui, et la contre-passation de chacune
  (`reverses_entry_id IN (…)`). Les paires **déliées** gardent l'ancien compte (net nul), réimputables par le geste de 36-16c.
  *Écartées* : les retrouver par l'audit ; refuser le geste tant qu'il en existe.
- **D3 — Toutes les lignes de ces écritures** prennent le nouveau compte, `NULL` compris (invariant des créations,
  `NewJournalEntry::project_id` stampé sur toutes les lignes, `entities/journal_entry.rs:193-197`). *Écartée* : ne changer que les
  lignes à l'ancien compte.
- **D4** — *(écriture manuelle : 36-16c.)*
- **D5 — Période close : permise, avec accusé** (C-36-16-4). Exercice `Closed`, suivi d'un exercice clos, ou date ≤
  `books_locked_through`, sans `acknowledgeClosedPeriods: true` → **`409 PROJECT_REASSIGNMENT_CLOSED_PERIOD`**, rien d'écrit,
  `details { entryCount, fiscalYears: [noms], lockedThrough }`. *Écartées* : refuser ; avertir à l'écran seulement.
- **D6 — Une trace par geste** (C-36-16-6) : `invoice.project_reassigned`, par `NewAuditLogEntry::par(Acteur::from(&current_user),
  …)`, `details { fromProjectId, toProjectId, journalEntryIds, closedPeriodsAcknowledged }`, libellé ×4. *Écartée* : une par écriture.
- **D7 — Versions** (C-36-16-7) : version de la facture exigée, lue sous `FOR UPDATE` (patron 34-5,
  `invoice_settlements_write.rs:132-135`), incrémentée ; version de chaque écriture touchée incrémentée.
- **D8 — Rien à faire, rien d'écrit** (C-36-16-8) : même compte → `200`, sans version, audit ni accusé ; compte visé validé par
  `validate_taggable_in_tx` ; compte actuel archivé sans effet ; `null` retire le compte.
- **D9 — Verrous** : sentinelle `companies` → `projects` (si compte visé non nul) → facture `FOR UPDATE` → lignes des écritures
  `FOR UPDATE`, par `id` croissant ; handler rejoué (`Rejouee`). Règlement, solde et avoir lisent le projet sous le verrou de la
  facture (établi ci-dessus). **Étape 0** : établir le même fait pour l'encaissement par rapprochement (`reconciliation.rs:2814`) ;
  s'il ne l'est pas, faire passer la lecture sous verrou dans la story (limite voisine) ou demander une issue.
- **D10 — Tout statut à l'API, bouton hors brouillon** (C-36-16-5), pour `canManage`.
- **D11 — Vocabulaire de la 36-15.**
- **D12 — Le dialogue est partagé** : `frontend/src/lib/features/projects/ProjectReassignDialog.svelte` reçoit **une liste de
  rangées** (libellé, compte actuel, compte choisi) et rend un sélecteur par rangée ; la facture client n'en passe qu'une. 36-16b
  l'emploie tel quel, 36-16c avec une rangée par ligne. Il gère l'envoi, le `409 PROJECT_REASSIGNMENT_CLOSED_PERIOD`
  (avertissement, *Confirmer*, renvoi avec accusé) et les erreurs. *Écartée* : un dialogue par écran (trois copies, contre la règle DRY).

## Critères d'acceptation

1. **AC1** — Facture client **payée** (vente, deux règlements dont un rapproché, un solde du reste, un avoir partiel) au compte P :
   le geste vers Q rend `200` ; la facture et **chaque ligne** de la vente, des règlements, du solde et de l'avoir portent Q ;
   `journalEntryIds` les liste.
2. **AC2** — Rien d'autre ne change : instantané complet des écritures (montants, comptes, dates, numéros, descriptions,
   `lettering_key`, `lettering_origin`), des `invoice_settlements`, des `bank_transactions.matched_entry_id` et du statut, identique
   hors `project_id`, `version`, `updated_at`.
3. **AC3** — Les rapports « Dépenses » / « Rendement » de Q contiennent ce que ceux de P contenaient pour la facture ; ceux de P ne
   le contiennent plus (hors paires déliées).
4. **AC4** — Exercice clos ou période verrouillée : sans accusé, `409 PROJECT_REASSIGNMENT_CLOSED_PERIOD` avec ses `details`, rien
   d'écrit ; avec accusé, `200`, audit `closedPeriodsAcknowledged: true`.
5. **AC5** — Compte visé archivé → `409 ILLEGAL_STATE_TRANSITION` ; inconnu ou d'une autre société → `404` ; facture d'une autre
   société → `404` ; version périmée → `409 OPTIMISTIC_LOCK_CONFLICT` ; rôle Lecture ou clé `read` → `403` ; rien d'écrit. Compte
   actuel archivé : réussite.
6. **AC6** — Audit `invoice.project_reassigned` avec ses `details` ; par clé, `actor_type = 'api_key'` et la clé ; libellé ×4.
7. **AC7** — Après l'annulation d'un règlement, ce règlement et sa contre-passation gardent l'ancien compte ; une facture annulée
   par avoir total se réimpute, vente et avoir compris.
8. **AC8** — Même compte : `200`, version inchangée, aucun audit ; `projectId: null` retire le compte partout.
9. **AC9** — Écran (facture payée, Comptable) : *Changer le compte analytique* ouvre le dialogue partagé (compte actuel, sélecteur,
   « — Aucun », archivés absents sauf l'actuel) ; *Enregistrer* réimpute, le libellé change sans rechargement, un message confirme ;
   en période close, l'avertissement nomme l'exercice ou la date du verrou et seul *Confirmer* envoie l'accusé. Lecture, brouillon :
   pas de bouton.
10. **AC10** — La route est au registre `audit_route_registry.rs` (`Traced`, `Rejouee`) ; `audit_acteur_garde.rs` reste vert **sans
    entrée neuve**.
11. **AC11** — Manuel utilisateur (`.tex` ; PDF régénéré à l'intégration, contrôlé aplati) : sous-section *Changer le compte
    analytique après coup* (facture client ; ce que le geste touche et ne touche pas ; avertissement ; paires déliées) ;
    `docs/api-external.md` : la route, l'accusé, l'ordre des refus, les codes neufs au § 10.
12. **AC12** — Textes visibles aux quatre locales, au vocabulaire de la 36-15 ; `i18n-keys.test.ts` (`sitesTotal`) recompté.

## Tests à écrire, et la preuve du rouge

Premier rouge : la route rend `404` avant le correctif (`kesh-gate-logs/e36-16a/rouges.log`). Puis, correctif **commité**, une
mutation par propriété (E5 : script dans `kesh-gate-logs/e36-16a/mut/`, restauration `git restore --worktree -- <f>` puis
`touch <f>`, journal `mut/<n>.log`).

| Test | Lieu | Mutation qui le fait rougir |
|---|---|---|
| T1 facture payée réimputée entière | `crates/kesh-db/tests/project_reassignment.rs` (squash `test-schema`) | retirer `invoice_settlements` ou `credit_notes` de l'ensemble |
| T2 instantané hors `project_id` identique | idem | ajouter `lettering_key = NULL` à l'`UPDATE` |
| T3 paire déliée intacte, annulée par avoir réimputée | idem | étendre l'ensemble aux paires déliées ; retirer la jointure `reverses_entry_id` |
| T4 invariant D3 sur les créations automatiques | idem | `project_id: None` dans `credit_notes.rs:718` (sonde sur un site existant) |
| T5 période close sans / avec accusé | idem + `crates/kesh-api/tests/project_reassignment_e2e.rs` | court-circuiter le contrôle ; `<` au lieu de `<=` sur `books_locked_through` |
| T6 acteur clé d'API | `project_reassignment_e2e.rs` | `Acteur::utilisateur(current_user.user_id)` au lieu de `Acteur::from` |
| T7 refus (archivé, inconnu, société, version, rôle, portée) | idem | retirer `validate_taggable_in_tx` ; la comparaison de version |
| T8 rien à faire | kesh-db | retirer le court-circuit D8 |
| T9 dialogue partagé (une et plusieurs rangées, 409 → accusé) | Vitest `features/projects/ProjectReassignDialog.test.ts` | ne pas renvoyer `acknowledgeClosedPeriods` |
| T10 page de la facture | `routes/(app)/invoices/[id]/invoice-display-page.test.ts` | ne pas rafraîchir le libellé ; montrer le bouton au rôle Lecture |
| T11 libellé d'audit ×4 | `tests/audit_label_registry.rs` (existant) | rougit d'office si le code manque |

**E2E exigé** — `frontend/tests/e2e/project-reassignment.spec.ts` : (1) facture client **payée** préparée par l'API (deux comptes
analytiques, validation, règlement), geste au navigateur, libellé changé, puis `GET /journal-entries/{id}` de la vente et du
règlement : toutes les lignes au nouveau compte ; (2) même geste après `PUT /companies/current/books-lock` : avertissement,
*Confirmer*, succès — **verrou levé en `finally`** (`…/books-lock/release`). Rouge d'abord (correctif retiré par `git stash`,
journal `e2e-rouge.log`).

## Tâches

- [ ] **Étape 0** — LOW de la dernière validation ; D9 pour `reconciliation.rs:2814` (ligne lue citée au Change Log) ; 36-15
  fusionnée, termes repris.
- [ ] **T1 kesh-db** — module `repositories/project_reassignment.rs` : `invoice_entry_ids_in_tx`, `period_exposure_in_tx`
  (générique, réemployé par b et c), `reassign_lines_in_tx` (`SELECT … FOR UPDATE` puis `UPDATE journal_entry_lines SET project_id
  = ? WHERE entry_id IN (…)`, versions +1), `reassign_invoice` ; variante `DbError::ProjectReassignmentClosedPeriod` et son code aux
  deux tables de `errors.rs` (`:232`, `:1967`).
- [ ] **T2 kesh-api** — `invoices::reassign_project_handler`, route, `retry_on_deadlock`, mappage `409` et `details` ; registre des
  routes ; `audit_labels.rs`.
- [ ] **T3 kesh-i18n** — libellé d'audit et textes du dialogue ×4, près des blocs `invoice-detail-*`.
- [ ] **T4 frontend** — `ProjectReassignDialog.svelte` (D12) ; appel d'API dans `features/invoices` (fichier `invoices.api.ts`,
  module déjà compté avec la page) ; bouton et dialogue sur `invoices/[id]/+page.svelte`.
- [ ] **T5** — tests T1 à T11, rouges puis mutations ; E2E.
- [ ] **T6** — captures (dialogue, avertissement, succès sur facture payée ; rôle Lecture sans bouton), thème clair,
  `kesh-gate-logs/e36-16a/captures/`.
- [ ] **T7** — manuel utilisateur et `docs/api-external.md`.
- [ ] **Gates** (`e35-dev-commun.md` § *Quels gates, quand*, voie 1, `flock /tmp/station-heavy.lock flock
  /home/gcorbaz/devel/kesh-gate-logs/.gate.lock scripts/mem-guard.sh …`, une commande par prise, `free -g` avant) : `kesh-db`
  touché → `scripts/test-fast.sh --ci` même entre les passes ; en `review`, `test-fast --ci`, `scripts/gate-ci.sh kesh_ci_e3616a`,
  frontend complet ; remédiations, `test-fast --ci` (+ frontend) ; clôture, `gate-ci` **et** E2E complet sur `kesh_e2e_e3616a`
  recréée, au dernier commit de code. Chaque chiffre cite journal et commit.

## Note de version

*Ajouté* — **Changer le compte analytique d'une facture client, même payée.** Sur la fiche d'une facture validée, payée ou
annulée, *Changer le compte analytique* réimpute d'un geste la facture et toutes ses écritures — vente, règlements, solde du reste,
avoirs —, sans toucher montant, compte, date, numéro, lettrage ni rapprochement. Sur un exercice clos ou une période verrouillée,
Kesh avertit que les rapports analytiques de cet exercice changeront et demande de confirmer. Le changement est inscrit au journal
d'audit, avec l'ancien et le nouveau compte. Pour les intégrations : `POST /api/v1/invoices/{id}/reassign-project`, ouverte aux clés
en écriture, et le code `PROJECT_REASSIGNMENT_CLOSED_PERIOD` (refs #1167).

## Dev Notes

### Modules touchés

`kesh-db` (dépôt neuf, `errors.rs`) ; `kesh-api/routes/invoices` (avec `lib.rs`, `errors.rs`, `audit_labels.rs`, tests de garde) ;
`kesh-i18n` (×4) ; `frontend/src/lib/features/projects` (dialogue) ; `routes/(app)/invoices/[id]` (avec l'appel de
`features/invoices`). **Cinq modules de code**, plus la documentation (manuel, `api-external.md`).

### Hors périmètre

- Facture fournisseur (**36-16b**), écriture manuelle et paires déliées (**36-16c**).
- La répartition d'une facture entre plusieurs comptes analytiques ; le vocabulaire (36-15) ; le MCP.
- L'attribution du règlement saisi (36-11b, #431).

### Fichiers communs

| Fichier | Stories | Règle |
|---|---|---|
| `messages.ftl` ×4, `user-manual.tex`, `invoices/[id]/+page.svelte` | 36-15 | **part de 36-15 fusionnée** |
| `project_reassignment.rs`, `ProjectReassignDialog.svelte`, `project-reassignment.spec.ts`, `api-external.md`, manuel | 36-16b, 36-16c | **elles partent de 36-16a fusionnée** |
| `invoice_settlements_write.rs`, `reconciliation.rs` | 36-9 | rien d'écrit ici sauf si l'étape 0 le demande ; union à l'intégration |
| `audit_route_registry.rs`, `lib.rs`, `errors.rs` | 36-9, 36-10, 36-18 | lignes distinctes, union |
| `i18n-keys.test.ts` (`sitesTotal`) | toutes | recompté à la source après chaque fusion (C-e350-18) |
| `CHANGELOG.md` | toutes | jamais écrit par la branche (C-e360-11) |

### Ce qui doit rester vrai

Aucun montant, compte, date, numéro, lettrage, rapprochement, règlement ni statut ne change ; une paire contre-passée est sur un
seul compte ; le `PUT` et `modification_blocker` restent tels quels ; aucun acteur sans clé ajouté ; aucune donnée réelle.

### Références

Fiche d'origine `36-16-reimputation-analytique.md` ; fichier d'epic (rang 15) ; C-e360-15, -16 ; fiche 36-15 (D2) ; #1167, #1161,
#1158, #541 ; stories 19-3, 19-4, 34-5, 34-11b, 35-8a1.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-15 — Création (Opus 5.5), par découpage de `36-16-reimputation-analytique.md` (C-36-16-10, décision de l'orchestrateur :
  dix modules ; règle des stories courtes de Guy du 2026-11-15). Reprend son lot A, ses décisions communes et le point d'étape 0
  sur `reconciliation.rs:2814`. Code lu à `fed34a9c7`. Aucune migration.
