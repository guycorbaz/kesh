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
écritures** passent au nouveau compte — la vente, les règlements et le solde du reste d'une facture payée ; la vente et l'avoir
d'une facture annulée — sans défaire de règlement ni de rapprochement. Le geste ne touche ni montant, ni compte du plan, ni date,
ni numéro, ni lettrage, ni rapprochement ; il est tracé au journal d'audit, acteur compris (utilisateur ou clé d'API). Sur un
exercice clos ou une période verrouillée, seul un Administrateur peut le faire, après un avertissement.

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
  - règlement — `invoice_settlements_write.rs:348`, projet lu sous verrou `:145-146` (« … FROM invoices WHERE id = ? AND company_id
    = ? FOR UPDATE ») ; liée par `invoice_settlements.journal_entry_id` ;
  - solde du reste — `invoice_settlements_write.rs:712`, lu sous verrou `:553-555`, lié par `invoice_settlements::create_in_tx`
    (`:728-730`) ;
  - avoir — `credit_notes.rs:718` (`project_id: invoice.project_id,`), facture verrouillée d'abord (`:309`, « (1) invoices FOR
    UPDATE ») ; lié par `credit_notes.journal_entry_id`. **Un avoir est toujours total et exclut tout règlement** : refusé dès
    qu'un règlement lié existe ou que `paid_at` est posé (`credit_notes.rs:391`, « if settlement.is_some() ||
    invoice.paid_at.is_some() ») ; un seul par facture (`uq_credit_notes_invoice`) ; la facture passe `cancelled`
    (`credit_notes.rs:790`) et n'est plus réglable. **Deux états réels, donc** : facture payée (vente, règlements, solde) et facture
    annulée par avoir (vente, avoir) — jamais les deux à la fois ;
  - encaissement par rapprochement — `routes/reconciliation.rs:2814` (`project_id: invoice.project_id,`), puis
    `invoice_settlements::create_in_tx` (`:2843-2845`). La facture y est lue **sans** verrou, **délibérément** (`:2938-2940`,
    « ⛔ **Pas de `FOR UPDATE` sur la facture en amont** : il lirait `version` à jour — ce contrôle passerait toujours »). La
    protection est le verrou optimiste `:2952-2955` (« UPDATE invoices SET paid_at = ?, version = version + 1 … WHERE id = ? AND
    company_id = ? AND version = ? AND status = 'validated' »), refus `race_during_update` si aucune ligne n'est touchée
    (`:2970-2975`), sous l'invariant `:2926-2936` (« tout écrit qui change le reste dû incrémente `version` »). **Établi à la
    lecture (D9)** : le geste incrémentant la version de la facture (D7), une acceptation fondée sur un instantané antérieur au
    geste est refusée, et l'écriture à l'ancien compte n'est pas conservée. **On ne pose pas de `FOR UPDATE` à `:2814`.**
- **Ce qui délie** : l'annulation d'un règlement supprime sa ligne (`invoice_settlements_write.rs:1043`, « DELETE FROM
  invoice_settlements WHERE id = ? AND company_id = ? ») ; la dévalidation supprime la vente (`invoices.rs:1678`, « journal_entry_id
  = NULL »). Contre-passation : `journal_entries.reverses_entry_id` (`20260828000002_journal_entries_reversal.sql:44`), lignes
  recopiées dans l'ordre (`journal_entries.rs:2942-2951`). **Côté client, une écriture encore liée n'a jamais de
  contre-passation** : la vente se supprime, le règlement se contre-passe **et** perd sa ligne dans la même transaction (`:1043`),
  l'avoir ne se contre-passe pas (`ReversalBlocker::OwnedByCreditNote`) — d'où D2.
- **Exercices et verrou** : `fy_status == "Closed"` (`journal_entries.rs:1280-1282`), exercice postérieur clos (`:1283-1288`,
  `fiscal_years::find_later_closed`), `companies.books_locked_through` (`:1293-1300`, seuil inclusif `:1602-1610`). Tous applicatifs.
- **Rôles** : la réouverture d'un exercice et la levée du verrou de période sont **réservées à l'Administrateur, motif
  obligatoire** (`lib.rs:474-489`, « Admin uniquement + motif obligatoire … DÉVERROUILLER DÉFAIT UNE GARANTIE ») ; la pose du
  verrou est un `POST /api/v1/companies/current/books-lock` `{ through }` en `comptable_routes` (`lib.rs:564`,
  `frontend/src/lib/features/settings/settings.api.ts:50-51`), la levée un `POST …/books-lock/release` `{ through, motif }`
  (`settings.api.ts:58-66`), qu'aucune clé n'atteint (`crates/kesh-api/tests/admin_pat_denied_e2e.rs`). `CurrentUser` porte `role`
  et `api_key_id` (`kesh-api/src/middleware/auth.rs:40`, `:44`).
- **Verrous partagés** : la sentinelle de société est `SELECT id FROM companies WHERE id = ? FOR UPDATE`
  (`bank_accounts.rs`, `acquire_company_sentinel_lock`) ; la pose du verrou des livres lit `books_locked_through … FOR UPDATE` sur
  la même ligne (`companies.rs:1022`) ; la clôture verrouille la ligne d'exercice (`fiscal_years.rs:163`, « FROM fiscal_years WHERE
  id = ? AND company_id = ? FOR UPDATE »).
- **Réemploi** : `projects::validate_taggable_in_tx` (`projects.rs:89` ; `NotFound` et `IllegalStateTransition` rendus `404` et
  `409 ILLEGAL_STATE_TRANSITION` par le `From<DbError>` générique de `AppError`, ce que vérifie le test
  `project_validation_errors_map_to_4xx`, `routes/journal_entries.rs:888-903`) ; ordre « companies → projects → invoices »
  (`invoices.rs:1128-1133`) ; `Acteur`
  (`entities/audit_log.rs:173`), `From<&CurrentUser>` (`kesh-api/src/audit.rs:18`), `NewAuditLogEntry::par` (`audit_log.rs:263`) ;
  `audit_labels.rs` (`:150`, `message_key` `:229-231`, garde `tests/audit_label_registry.rs`) ; `tests/audit_route_registry.rs`
  (`:516`) ; `comptable_routes` (`lib.rs:536`) ; `retry_on_deadlock` (`routes/invoices.rs:898`).
- **Écran** : `canManage` (`frontend/src/routes/(app)/invoices/[id]/+page.svelte:92-94`), projets chargés archivés compris
  (`:269`, `listProjects(true)`), libellé `:1279-1285` (`data-testid="invoice-project"`) ; sélecteur du brouillon
  `frontend/src/lib/components/invoices/InvoiceForm.svelte:811-830` (option « — Aucun », repli d'un projet hors liste) ; appels des
  projets : `frontend/src/lib/features/projects/projects.api.ts`.
- **Manuel** : `user-manual.tex:3766` (section analytique), `:3779-3790` (affectation) ; rien sur un changement après coup.
  **API** : `docs/api-external.md:1007` (le `PUT` du brouillon), § 10 `:1034`, et le paragraphe **« Interblocages : les écritures
  au journal sont rejouées »** (`:1103`), qui énumère les routes rejouées.

## Décisions

*Les décisions D0 à D13 sont **communes aux trois stories** (C-36-16-1 à -9, -11 à -13) ; 36-16b et 36-16c y renvoient.*

- **D0 — Sans migration.** Si le développement en découvre le besoin, il s'arrête et le dit (C-e360-3 : la story passe à la 0.19.0).
- **D1 — Une route par objet, `POST …/{id}/reassign-project`** (C-36-16-1), dans `comptable_routes`, clés `read-write`, pas MCP.
  Ici : `POST /api/v1/invoices/{id}/reassign-project`, corps `{ "projectId": n | null, "version": n, "acknowledgeClosedPeriods"?:
  bool }`, réponse `{ invoice, journalEntryIds }`. Identifiants `project*` gardés (C-e360-15). **Contrat de l'API : ajout seulement.**
  *Écartées* : le `PUT` étendu aux validées ; une route sur les lignes ; un chemin `…/analytic-account`.
- **D2 — Les écritures d'une pièce** (C-36-16-2) : celles qui lui sont reliées aujourd'hui. La fonction générique de collecte
  prend un paramètre `include_reversals` (ajouter `reverses_entry_id IN (…)`) : **faux pour la facture client**, où une écriture
  encore liée n'a jamais de contre-passation (§ *Ce qui délie*), **vrai pour la facture fournisseur** (36-16b : contre-passation de
  l'achat d'une annulée). Aucun test de 36-16a ne prétend éprouver la jointure (elle ne rougirait que sur un état fabriqué en SQL) ;
  36-16b l'éprouve sur un état réel. Les paires **déliées** gardent l'ancien compte (net nul), réimputables par le geste de 36-16c.
  *Écartées* : les retrouver par l'audit ; refuser le geste tant qu'il en existe.
- **D3 — Toutes les lignes de ces écritures** prennent le nouveau compte, `NULL` compris (invariant des créations,
  `NewJournalEntry::project_id` stampé sur toutes les lignes, `entities/journal_entry.rs:193-197`). *Écartée* : ne changer que les
  lignes à l'ancien compte.
- **D4** — *(écriture manuelle : 36-16c.)*
- **D5 — Période close : réservée à l'Administrateur, avec accusé** (C-36-16-4, **révisé par C-36-16-11**). Une écriture
  **exposée** est une écriture qui changerait (D8) et qui est dans un exercice `Closed`, dans un exercice suivi d'un exercice clos,
  ou datée au plus tard de `books_locked_through`. S'il en existe une :
  - geste d'un **Comptable**, ou par **une clé d'API** quelle qu'elle soit → **`403 PROJECT_REASSIGNMENT_CLOSED_PERIOD_ADMIN_ONLY`**,
    rien d'écrit, mêmes `details`, l'accusé n'y change rien — comme la réouverture d'exercice et la levée du verrou
    (`lib.rs:474-489`), parce que le geste défait la même garantie ;
  - geste d'un **Administrateur en session**, sans `acknowledgeClosedPeriods: true` → **`409 PROJECT_REASSIGNMENT_CLOSED_PERIOD`**,
    rien d'écrit ; avec l'accusé → réussite, audit `closedPeriodsAcknowledged: true`.
  `details { entryCount, fiscalYears: [noms], lockedThrough }`, `entryCount` = nombre d'écritures **exposées qui changeraient**
  (V1-16). En période ouverte, Comptable et clé `read-write` réimputent sans accusé. *Écartées* : accusé ouvert à tout Comptable
  et à toute clé (première version : une clé, sans écran, changerait un exercice clos) ; refuser à tous ; un motif obligatoire
  comme la réouverture — non demandé, l'audit garde l'accusé et l'Administrateur ; s'ajoute sans changer le contrat si Guy le veut.
- **D6 — Une trace par geste, dans la transaction** (C-36-16-6) : `invoice.project_reassigned`, écrite **par le dépôt, avant le
  `COMMIT`**, par `NewAuditLogEntry::par(acteur, …)` — le dépôt reçoit l'`Acteur` (signature en tâche T1), le handler passe
  `Acteur::from(&current_user)` ; jamais un `user_id` nu (sans quoi `Acteur::utilisateur(` ferait rougir `audit_acteur_garde.rs`).
  `details { fromProjectId, toProjectId, journalEntryIds, closedPeriodsAcknowledged }`, libellé ×4. *Écartées* : une trace par
  écriture ; l'audit au handler après commit (geste réussi, trace perdue).
- **D7 — Versions** (C-36-16-7) : version de la facture exigée, lue sous `FOR UPDATE` (patron 34-5,
  `invoice_settlements_write.rs:132-135`), **incrémentée d'une unité** ; version de **chaque** écriture touchée incrémentée d'une
  unité ; écritures hors de l'ensemble inchangées. C'est l'incrément de la facture qui protège l'encaissement par rapprochement (D9).
- **D8 — Rien à faire, rien d'écrit** (C-36-16-8, **précisé par C-36-16-12**) : le geste n'a rien à faire si la facture **et
  toutes les lignes** de l'ensemble portent déjà le compte visé → `200`, sans version, audit ni accusé. Une facture dont une
  écriture diverge (donnée historique) est donc réalignée par le geste. Compte visé validé par `validate_taggable_in_tx` **seulement
  s'il diffère du compte actuel de la facture** (grandfathering, `invoices.rs:1143-1144`) ; `null` retire le compte.
- **D9 — Verrous, lectures, et ordre des refus** (**C-36-16-13**). Dans une transaction, rejouée sur interblocage
  (`retry_on_deadlock`, registre `Rejouee`) :
  1. sentinelle `companies … FOR UPDATE`, **toujours** (même pour `null`), qui lit aussi `books_locked_through` : la pose du verrou
     des livres (`companies.rs:1022`) est ainsi sérialisée avec le geste ;
  2. facture `FOR UPDATE` (`404` si absente ou d'une autre société) → version (`409 OPTIMISTIC_LOCK_CONFLICT`) ;
  3. **ensemble relu sous verrou, après la facture** : `invoice_settlements WHERE invoice_id = ? … FOR UPDATE`, `credit_notes …
     FOR UPDATE`, la vente ; puis les en-têtes d'écriture `FOR UPDATE` par `id` croissant (exercice, date, version) — une lecture
     ordinaire manquerait un règlement commité depuis l'instantané (leçon 25-3-b) ;
  4. lignes de ces écritures `FOR UPDATE` ; court-circuit D8 (`200`) ;
  5. compte visé (`validate_taggable_in_tx` : `404`, `409 ILLEGAL_STATE_TRANSITION`) — la sentinelle étant déjà tenue, l'ordre
     projets-après-facture ne crée pas d'inversion avec la création ni l'archivage, qui la prennent d'abord ;
  6. exercices des écritures exposées `FOR UPDATE` par `id` croissant, `find_later_closed_in_tx` — sérialisé avec la clôture
     (`fiscal_years.rs:163`) ; l'acceptation par rapprochement tient l'exercice **avant** la facture : interblocage possible,
     rattrapé par le rejeu, couvert par T5 (cas « clôture concurrente » joué en deux transactions) ;
  7. exposition → `403` (non-Administrateur) ou `409` (sans accusé) ;
  8. `UPDATE` des lignes, des versions, de la facture ; audit ; `COMMIT`.
  Aucun `UPDATE` ne précède l'étape 8 : un refus n'écrit rien. **L'encaissement par rapprochement n'est pas modifié** (§ *Ce qui se
  lit au code* : protégé par `version = ?`, `:2952-2955`) ; règlement, solde et avoir lisent le projet sous le verrou de la facture,
  donc après le geste.
- **D10 — Tout statut à l'API, bouton hors brouillon** (C-36-16-5), pour `canManage`.
- **D11 — Vocabulaire** : pour le nom « compte analytique », les clés neuves reprennent **le mot que porte la 36-15 fusionnée** dans
  chaque locale, relu à l'étape 0 (de-CH, it-CH, en-CH attendent l'arbitrage de Guy : la story ne tranche rien ; si la 36-15 change
  ensuite ses termes, elle change aussi ceux-ci).
- **D12 — Le dialogue est partagé, contrat fermé** : `frontend/src/lib/features/projects/ProjectReassignDialog.svelte`.
  - Props : `rows: { id: number; label: string; currentProjectId: number | null }[]` (36-16a et 36-16b : une rangée, `id` = la
    facture ; 36-16c : une rangée par ligne, `id` = `lineId`) ; `projects: ProjectResponse[]` (chargés par la page, archivés
    compris — la page de la facture le fait déjà, `:269`) ; `submit: (choices: { id: number; projectId: number | null }[],
    acknowledge: boolean) => Promise<void>` (rejette avec l'erreur de l'API) ; `onDone: () => void`.
  - Sélecteur : « — Aucun », projets non archivés, plus l'actuel de la rangée s'il est archivé.
  - *Enregistrer* inactif tant qu'aucune rangée n'a changé.
  - `409 PROJECT_REASSIGNMENT_CLOSED_PERIOD` : avertissement nommant `details.fiscalYears` et/ou `details.lockedThrough` (date
    suisse) et `entryCount`, bouton *Confirmer* qui rappelle `submit(choices, true)`. `403 …_ADMIN_ONLY` : message (seul un
    Administrateur peut changer le compte analytique d'un exercice clos ou d'une période verrouillée), sans *Confirmer*. Autres
    erreurs : message de l'API sous le formulaire.
  - Les fonctions d'appel vivent dans `frontend/src/lib/features/projects/projects.api.ts` (`reassignInvoiceProject` ici ; 36-16b
    et 36-16c y ajoutent les leurs) : un seul module d'API pour le geste.
  *Écartée* : un dialogue par écran (trois copies, contre la règle DRY).
- **D13 — Rôle à l'écran** : le bouton paraît pour `canManage` hors brouillon ; le dialogue ne cache pas la période close au
  Comptable : il lui montre le refus `403` reçu.

## Critères d'acceptation

*Tous les états se construisent **par les routes** (ou les fonctions de dépôt qu'elles appellent), jamais par un `INSERT` direct.*

1. **AC1** — Facture client **payée** au compte P — vente, deux règlements dont un rapproché (par `POST /reconciliation/accept`), un
   solde du reste (`POST /invoices/{id}/write-off`) : le geste vers Q rend `200` ; la facture et **chaque ligne** de la vente, des
   règlements et du solde portent Q ; `journalEntryIds` les liste, et rien d'autre.
2. **AC2** — Facture client **annulée par avoir** (validée puis `POST /credit-notes`, aucun règlement) au compte P : le geste vers Q
   réimpute la facture, la vente et l'avoir.
3. **AC3** — Rien d'autre ne change : instantané complet des écritures (montants, comptes, dates, numéros, descriptions,
   `lettering_key`, `lettering_origin`), des `invoice_settlements`, des `bank_transactions.matched_entry_id` et du statut, identique
   hors `project_id`, `version`, `updated_at`.
4. **AC4** — **Versions** : celle de la facture augmente d'une unité, celle de chaque écriture touchée d'une unité, celle des
   écritures hors de l'ensemble (autre facture, paire déliée) ne bouge pas.
5. **AC5** — **Encaissement concurrent** : une acceptation par rapprochement dont l'instantané précède le geste (transaction ouverte
   et facture lue avant le `COMMIT` du geste) est refusée en `RECONCILIATION_INVOICE_NOT_ELIGIBLE`, `reason: race_during_update`,
   sans écriture ; rejouée après, elle passe avec le nouveau compte.
6. **AC6** — Les rapports « Dépenses » / « Rendement » de Q contiennent ce que ceux de P contenaient pour la facture ; ceux de P ne
   le contiennent plus (hors paires déliées).
7. **AC7** — **Période close** (exercice clos ; puis, séparément, verrou des livres au-delà de la vente) : un Comptable reçoit `403
   PROJECT_REASSIGNMENT_CLOSED_PERIOD_ADMIN_ONLY` avec ou sans accusé ; une clé `read-write` aussi ; un Administrateur en session
   reçoit `409 PROJECT_REASSIGNMENT_CLOSED_PERIOD` sans accusé et `200` avec, audit `closedPeriodsAcknowledged: true` ; chaque
   refus n'écrit rien ; `entryCount` ne compte que les écritures exposées qui changeraient.
8. **AC8** — **Ordre des refus** (D9) : facture absente ou d'une autre société `404` → version périmée `409
   OPTIMISTIC_LOCK_CONFLICT` (même si le compte ne change pas) → rien à faire `200` → compte visé inconnu `404` ou archivé `409
   ILLEGAL_STATE_TRANSITION` (le compte **actuel** archivé, visé de nouveau, rend `200`) → `403`/`409` de période close ; rôle Lecture
   ou clé `read` : `403` du contrôle de rôle, avant tout. Un test par paire voisine.
9. **AC9** — **Audit** : `invoice.project_reassigned`, `details { fromProjectId, toProjectId, journalEntryIds,
   closedPeriodsAcknowledged }`, écrit dans la transaction du geste ; par clé, `actor_type = 'api_key'` et la clé ; libellé ×4.
10. **AC10** — **Paires déliées** : après l'annulation d'un règlement, ce règlement et sa contre-passation gardent P et leur version.
11. **AC11** — **Rien à faire** : facture et lignes déjà à Q → `200`, versions inchangées, aucun audit ; facture à Q dont une ligne
    diverge → la ligne est réalignée ; `projectId: null` retire le compte partout.
12. **AC12** — **Écran** (facture payée) : *Changer le compte analytique* (Comptable, Admin) ouvre le dialogue partagé ; *Enregistrer*
    inactif tant que rien ne change ; envoi, libellé rafraîchi sans rechargement, message ; Administrateur en période close :
    avertissement nommant l'exercice ou la date du verrou, *Confirmer* ; Comptable en période close : le message du `403`, sans
    *Confirmer*. Lecture, brouillon : pas de bouton.
13. **AC13** — La route est au registre `audit_route_registry.rs` (`Traced`, `Rejouee`) ; `audit_acteur_garde.rs` vert **sans entrée
    neuve**.
14. **AC14** — Manuel utilisateur (`.tex` ; PDF à l'intégration, contrôlé aplati) : sous-section *Changer le compte analytique après
    coup* (facture client, ce que le geste touche et ne touche pas, réserve de l'Administrateur en période close, paires déliées) ;
    `docs/api-external.md` : la route, l'accusé et le rôle, l'ordre des refus, les codes neufs au § 10, la route ajoutée au
    paragraphe « Interblocages » (`:1103`) avec « depuis la v0.18.0 ».
15. **AC15** — Textes aux quatre locales selon D11 ; `frontend/src/lib/shared/i18n-keys.test.ts` (`sitesTotal`) recompté.

## Tests à écrire, et la preuve du rouge

Premier rouge : la route rend `404` avant le correctif (`kesh-gate-logs/e36-16a/rouges.log`). Puis, correctif **commité**, une
mutation par propriété (E5 : script dans `kesh-gate-logs/e36-16a/mut/`, restauration `git restore --worktree -- <f>` puis
`touch <f>`, journal `mut/<n>.log`).

| Test | Lieu | Mutation qui le fait rougir |
|---|---|---|
| T1a facture payée (vente, règlements dont un rapproché, solde), par les routes | `crates/kesh-api/tests/project_reassignment_e2e.rs` | retirer `invoice_settlements` de l'ensemble |
| T1b facture annulée par avoir (vente, avoir), par les routes | idem | retirer `credit_notes` de l'ensemble |
| T2 instantané hors `project_id`, `version`, `updated_at` identique | idem | ajouter `lettering_key = NULL` à l'`UPDATE` |
| T3 versions : facture +1, chaque écriture touchée +1, hors ensemble inchangée | idem | retirer le `version = version + 1` des écritures |
| T4 encaissement concurrent (AC5) : transaction d'acceptation ouverte et facture lue, geste commité, `accept_one_invoice` → `race_during_update`, rien d'écrit | `crates/kesh-api` (test d'intégration de la route de rapprochement, deux connexions) | retirer le `version = version + 1` de la **facture** dans le geste |
| T5 paire déliée intacte (AC10) | idem T1 | étendre l'ensemble à toute écriture de libellé « Règlement facture » (sonde) |
| T6 invariant D3 sur les créations automatiques | `crates/kesh-db/tests/project_reassignment.rs` (squash `test-schema`) | `project_id: None` dans `credit_notes.rs:718` (sonde sur un site existant) |
| T7 période close : Comptable `403`, clé `403`, Admin `409` puis `200` ; verrou au jour exact (seuil inclusif) ; clôture concurrente en deux transactions | `project_reassignment_e2e.rs` | réserver le `403` aux clés seules ; `<` au lieu de `<=` sur `books_locked_through` ; lecture de l'exercice sans `FOR UPDATE` (le cas concurrent passe sans accusé) |
| T8 audit dans la transaction, acteur clé | idem | `Acteur::utilisateur(current_user.user_id)` au lieu de `Acteur::from` ; trace écrite après le `COMMIT` (un échec injecté après commit la perd : le test lit la trace dans la même transaction de test) |
| T9 ordre des refus (AC8), une paire voisine par test | idem | permuter version et rien-à-faire ; valider le compte visé même s'il est l'actuel |
| T10 rien à faire ; ligne divergente réalignée | idem | comparer à la facture seule |
| T11 dialogue partagé : deux rangées et leurs `id` dans `submit` ; *Enregistrer* inactif sans changement ; `409` → avertissement → *Confirmer* → `submit(…, true)` ; `403` → message sans *Confirmer* | Vitest `frontend/src/lib/features/projects/ProjectReassignDialog.test.ts` | ne pas renvoyer `acknowledge` ; montrer *Confirmer* sur `403` |
| T12 page de la facture | `frontend/src/routes/(app)/invoices/[id]/invoice-display-page.test.ts` | ne pas rafraîchir le libellé ; bouton au rôle Lecture |
| T13 libellé d'audit ×4 | `tests/audit_label_registry.rs` (existant) | rougit d'office si le code manque |

**E2E exigé** — `frontend/tests/e2e/project-reassignment.spec.ts`, en Administrateur (utilisateur de l'E2E) : (1) facture client
**payée** préparée par l'API (deux comptes analytiques, validation, règlement), geste au navigateur, libellé changé, puis `GET
/journal-entries/{id}` de la vente et du règlement : toutes les lignes au nouveau compte ; (2) même geste après `POST
/api/v1/companies/current/books-lock` `{ through }` (date postérieure à la vente) : avertissement, *Confirmer*, succès. **Levée** en
`finally` par `POST …/books-lock/release` `{ through: null, motif }`, en session Administrateur (une clé est refusée) ; et, contre
un `finally` qui n'aurait pas tourné (run interrompu), un `beforeAll` de la spec qui lève tout verrou résiduel avant de commencer,
pour que la suite complète ne reste pas verrouillée. Rouge d'abord (correctif retiré par `git stash`, journal `e2e-rouge.log`).

## Tâches

- [ ] **Étape 0** — LOW de la dernière validation ; relire `reconciliation.rs:2926-2975` sur la base de départ (le verrou optimiste
  y est toujours ; **n'y rien poser**) ; 36-15 fusionnée, ses termes relus (D11).
- [ ] **T1 kesh-db** — module `repositories/project_reassignment.rs` :
  - `pub async fn reassign_invoice(pool, acteur: Acteur, may_acknowledge: bool, company_id: i64, invoice_id: i64, project_id:
    Option<i64>, version: i32, acknowledge: bool) -> Result<Reassigned, DbError>` — les étapes de D9, la trace
    `invoice.project_reassigned` par `NewAuditLogEntry::par(acteur, …)` **avant** le `COMMIT` ;
  - génériques, réemployés par b et c : `entry_ids_in_tx(…, include_reversals)`, `lock_entries_and_lines_in_tx`,
    `period_exposure_in_tx` (exercices `FOR UPDATE`, postérieur clos, `books_locked_through` lu sur la sentinelle),
    `reassign_lines_in_tx` (`UPDATE journal_entry_lines SET project_id = ? WHERE entry_id IN (…)`, versions +1) ;
  - variantes `DbError::ProjectReassignmentClosedPeriod { … }` et `…ClosedPeriodAdminOnly { … }`, codes aux deux tables de
    `errors.rs` (`:232`, `:1967`).
- [ ] **T2 kesh-api** — `invoices::reassign_project_handler` : `may_acknowledge = current_user.role == Role::Admin &&
  current_user.api_key_id.is_none()`, `Acteur::from(&current_user)`, `retry_on_deadlock`, mappage `409`/`403` et `details` ; route ;
  registre des routes ; `audit_labels.rs`. `reconciliation.rs` n'est **pas** modifié.
- [ ] **T3 kesh-i18n** — libellé d'audit et textes du dialogue ×4, près des blocs `invoice-detail-*`.
- [ ] **T4 frontend** — `ProjectReassignDialog.svelte` (contrat D12) ; `reassignInvoiceProject` dans
  `frontend/src/lib/features/projects/projects.api.ts` ; bouton et dialogue sur `invoices/[id]/+page.svelte`.
- [ ] **T5** — tests T1a à T13, rouges puis mutations ; E2E.
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
annulée, *Changer le compte analytique* réimpute d'un geste la facture et toutes ses écritures — vente, règlements et solde du
reste, ou vente et avoir —, sans toucher montant, compte, date, numéro, lettrage ni rapprochement. Sur un exercice clos ou une
période verrouillée, le geste est réservé à l'Administrateur : Kesh l'avertit que les rapports analytiques de cet exercice
changeront et lui demande de confirmer. Le changement est inscrit au journal d'audit, avec l'ancien et le nouveau compte. Pour les
intégrations : `POST /api/v1/invoices/{id}/reassign-project`, ouverte aux clés en écriture en période ouverte, et les codes
`PROJECT_REASSIGNMENT_CLOSED_PERIOD`, `PROJECT_REASSIGNMENT_CLOSED_PERIOD_ADMIN_ONLY` (refs #1167).

## Dev Notes

### Modules touchés

`kesh-db` (dépôt neuf, `errors.rs`) ; `kesh-api/routes/invoices` (avec `lib.rs`, `errors.rs`, `audit_labels.rs`, tests de garde) ;
`kesh-i18n` (×4) ; `frontend/src/lib/features/projects` (dialogue et appel d'API, `projects.api.ts`) ;
`frontend/src/routes/(app)/invoices/[id]`. **Cinq modules de code** — `features/invoices` n'est pas touché —, plus la
documentation (manuel, `api-external.md`). `kesh-api/routes/reconciliation` est lu, non modifié.

### Limites assumées

- Le geste ne retrouve pas les paires déliées (D2) ; 36-16c les atteint.
- La note de l'invariant de version (`reconciliation.rs:2926-2936`) ne nomme pas le geste : la modifier ferait un sixième module ;
  le doc-comment de `reassign_invoice` dit qu'il incrémente la version **pour** ce verrou, et T4 le garde.

### Hors périmètre

- Facture fournisseur (**36-16b**), écriture manuelle et paires déliées (**36-16c**).
- La répartition d'une facture entre plusieurs comptes analytiques ; le vocabulaire (36-15) ; le MCP.
- L'attribution du règlement saisi (36-11b, #431).

### Fichiers communs

| Fichier | Stories | Règle |
|---|---|---|
| `messages.ftl` ×4, `user-manual.tex`, `invoices/[id]/+page.svelte` | 36-15 | **part de 36-15 fusionnée** |
| `project_reassignment.rs`, `ProjectReassignDialog.svelte`, `project-reassignment.spec.ts`, `api-external.md`, manuel | 36-16b, 36-16c | **elles partent de 36-16a fusionnée** |
| `invoice_settlements_write.rs`, `reconciliation.rs` | 36-9 | lus, jamais écrits par cette story |
| `projects.api.ts` | 36-16b, 36-16c | une fonction par geste, union |
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
- 2026-11-15 — Remédiation de la validation P1 (Sonnet 5.5, `kesh-gate-logs/e36-16/validation-36-16a-p1.md`, 0/2/7/7), prémisses
  relues au code. V1-1 : AC1 scindé en deux états réels construits par les routes (payée ; annulée par avoir), « avoir partiel »
  retiré (`credit_notes.rs:391`). V1-2 : conclusion corrigée — l'encaissement par rapprochement est protégé par `version = ?`
  (`reconciliation.rs:2952-2955`), aucun verrou n'y est posé ; AC5, T4 et sa mutation. V1-3 : AC4, T3. V1-4 : `include_reversals`,
  faux ici. V1-5 : accusé réservé à l'Administrateur en session (C-36-16-11), AC7, T7. V1-6 : `POST …/books-lock` `{ through }`,
  levée Administrateur avec `motif`, `beforeAll`. V1-7 : contrat du dialogue (D12). V1-8 : signature avec `Acteur`, audit avant
  le `COMMIT` (D6, T1, T8). V1-9 : lectures et ordre (D9, C-36-16-13). V1-10 : citations (`:145-146`, test de mappage, chemins).
  V1-11 : paragraphe « Interblocages ». V1-12 : appel dans `projects.api.ts`, cinq modules. V1-13 : ordre des refus (AC8, T9).
  V1-14 : rien-à-faire compare aussi les lignes (C-36-16-12). V1-15 : D11. V1-16 : `entryCount`. Aucun finding réfuté.
