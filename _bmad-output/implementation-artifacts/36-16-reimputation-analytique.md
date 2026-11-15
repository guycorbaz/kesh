# Story 36-16 : changer le compte analytique d'une facture, même payée, et d'une écriture passée (#1167)

Status: split

> **Découpée le 2026-11-15** (C-36-16-10) en **36-16a** (`36-16a-reimputation-facture-client.md`, le patron et la facture
> client), **36-16b** (`36-16b-reimputation-facture-fournisseur.md`) et **36-16c** (`36-16c-reimputation-ecriture-manuelle.md`).
> Ces trois fiches font foi ; celle-ci reste comme relevé de l'inventaire et des décisions d'origine.

*Fiche rédigée le 2026-11-15 par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-16`, branche
`story/36-16-reimputation-analytique` partie de `epic/36-0.18.0`, par le workflow `bmad-create-story`. Phase 2, release
**0.18.0**, rang 15 du fichier d'epic. **Aucune migration** (colonnes `project_id` existantes, D0). **`kesh-db` touché : gate
complet même en cours de boucle.** Voie **1** (`kesh-mariadb-dev:3306`, bases `kesh_e3616`, `kesh_e2e_e3616`, `kesh_ci_e3616`).
Code lu à la tête du worktree, **`fed34a9c7`**. Registres : `e36-choix-autonomes-e360.md` (C-e360-3, -11, -15, -16) ;
`e36-choix-autonomes-36-16-reimputation-analytique.md` (C-36-16-1 à -9). Validation non lancée.*

**Issues** : la PR porte `refs #1167` (le mot-clé de fermeture est l'affaire de la PR de release). **Aucun mot-clé de
fermeture** dans un commit. **Dépend de la 36-15** (#1161) : la story part de `epic/36-0.18.0` **après** la fusion de la 36-15,
dont elle reprend le vocabulaire (« compte analytique », traductions arrêtées par la D2 de la fiche 36-15).

⚠️ **Plus de cinq modules** (§ *Dev Notes*, *Modules touchés* : dix). La fiche est écrite d'un tenant, en **trois lots**
(A facture client, B facture fournisseur, C écriture manuelle) qui sont le **découpage proposé** à l'orchestrateur : chaque lot
se lit seul, A pose le patron que B et C réemploient.

## Story

En tant que titulaire qui tient ses livres dans Kesh et suit ses dépenses et ses revenus par compte analytique, je veux
**changer le compte analytique** d'une facture client ou fournisseur **quel que soit son statut** — payée comprise, d'un
exercice clos compris —, et celui des lignes d'une écriture déjà passée, **d'un seul geste**, afin que les rapports « Dépenses »
et « Rendement » disent juste sans défaire de règlement, de rapprochement ni d'écriture légale. Le compte analytique relève de
la gestion interne (art. 957a CO) : le geste ne touche aucun montant, aucun compte du plan, aucune date, aucun numéro, aucun
lettrage ni rapprochement, et il est **tracé** au journal d'audit, à l'acteur près (utilisateur ou clé d'API).

## Ce qui se lit au code (`fed34a9c7`)

### Où vit la dimension analytique

- Trois colonnes, toutes `BIGINT NULL` sur `projects(id)` en `ON DELETE RESTRICT` :
  `journal_entry_lines.project_id` (`crates/kesh-db/migrations/20260702000001_projects_analytics.sql`, « ADD COLUMN project_id
  BIGINT NULL, ADD CONSTRAINT fk_jel_project »), `supplier_invoices.project_id` (`20260702000002_supplier_invoices_project.sql`),
  `invoices.project_id` (`20260703000001_invoices_project.sql`). `journal_entries` n'en a pas : le projet « document-level »
  n'est qu'un champ de l'entité de création, `NewJournalEntry::project_id` (`crates/kesh-db/src/entities/journal_entry.rs:197`),
  « stampé sur **toutes** les lignes de l'écriture à l'insertion », la ligne primant (`:208-210`, « `line.project_id.or(new.project_id)` »).
- Aucun déclencheur sur ces tables (`git grep -nil TRIGGER -- crates/kesh-db/migrations/` : `20260507200001_bank_account_journal_link.sql`,
  `20260513000001_reconciliation_rules.sql`, sans rapport). Aucune empreinte d'écriture à recalculer (`git grep -niE
  "hash|checksum|empreinte" -- crates/kesh-db/src/repositories/journal_entries.rs` ne rend que le rappel P8 de `:22`).
- Les rapports lisent la **ligne**, jamais la pièce : `crates/kesh-report/src/project_report.rs:5` (« Lit la dimension
  `journal_entry_lines.project_id` »), agrégat **net** `:263-264` (« COALESCE(SUM(jel.debit), 0) - COALESCE(SUM(jel.credit), 0) AS
  amount »), filtré `AND a.account_type = 'Expense'` (`:269`) et `AND jel.project_id IN ({ids_ph})` (`:271`) ; le détail par
  écriture `:290-303` (`HAVING amount != 0`, par `je.id`).
- Le PDF de la facture ne montre pas le compte analytique (`git grep -n -i project -- crates/kesh-qrbill/src` vide) : un PDF
  figé n'est pas concerné.

### Aucun chemin ne réécrit aujourd'hui le `project_id` d'une ligne passée

Inventaire par le symptôme — « une écriture SQL qui pose `project_id` » —, journal `kesh-gate-logs/e36-16/inventaire-fiche.log`
(commandes et sorties copiées, HEAD `fed34a9c7`) :

| Commande | Sites de production |
|---|---|
| `git grep -n "INSERT INTO journal_entry_lines" -- 'crates/*/src/*'` | `journal_entries.rs:512` (création), `:1672` (modification d'une écriture manuelle, lignes supprimées puis réinsérées) ; `pre_upgrade_backup/tests.rs:244`, `journal_entries.rs:5373` sont des tests |
| `git grep -n "UPDATE journal_entry_lines" -- 'crates/*/src/*'` | `letterings.rs:1204`, `:1409` (lettrage seul) ; `sql_lexical.rs:400` est un test du lexeur — **aucun `UPDATE … project_id`** |
| `git grep -n "project_id = ?" -- 'crates/*/src/*'` | `invoices.rs:1306` (brouillon), `supplier_invoices.rs:758` (brouillon, `update_draft_in_tx`, « AND status = 'draft' »), `reconciliation_rules.rs:333` (projet par défaut d'une règle) |
| `git grep -nE "INSERT INTO (invoices\|supplier_invoices)\b"` | `invoices.rs:740`, `supplier_invoices.rs:637` (création) ; `pre_upgrade_backup/tests.rs:271`, `journal_entries.rs:4045` sont des tests |

Le projet d'une écriture automatique se pose donc **une fois**, à sa création, depuis celui de la pièce (`NewJournalEntry {
project_id, … }`) ; second détecteur, `git grep -nE "^\s*project_id(: [a-z_.]+)?,?$"` sur les dépôts, routes, services et
`kesh-reconciliation` (sonde positive : il rend bien `invoices.rs:2446`, journal) — les sites qui **écrivent** :

- **Vente** — `invoices.rs:2443-2446`, à la validation : « le tag document-level de la facture est propagé sur toutes les lignes
  de l'écriture de vente » / `project_id: invoice_before.project_id,`.
- **Règlement client** — `invoice_settlements_write.rs:348` (`project_id,`), lu sous verrou `:143-145` (« SELECT status,
  journal_entry_id, project_id, invoice_number, date, version FROM invoices WHERE id = ? AND company_id = ? FOR UPDATE ») ; la ligne
  `invoice_settlements` est créée dans la même transaction.
- **Solde du reste** — `invoice_settlements_write.rs:712` (`project_id,`, écriture « Solde facture … » au journal OD), lu sous
  verrou `:553-555` ; liée par `invoice_settlements::create_in_tx` (`:728-730`, « La ligne de solde, par le même `create_in_tx` que les
  règlements »).
- **Avoir** — `credit_notes.rs:718` (`project_id: invoice.project_id,`, « l'avoir HÉRITE le projet de sa facture d'origine »,
  `:713-717`), facture verrouillée d'abord (`:309`, « (1) invoices FOR UPDATE »).
- **Encaissement par rapprochement** — `routes/reconciliation.rs:2814` (`project_id: invoice.project_id,`, « Le règlement hérite
  du projet de la facture »), puis `invoice_settlements::create_in_tx` (`:2843-2845`, « (f) La liaison facture ↔ écriture »).
- **Achat** — `supplier_invoices.rs:1777` (`project_id,`, « Propagation document-level »), lu `:1585` ; projet contrôlé
  `:1611-1612` (`validate_taggable_in_tx`).
- **Paiement fournisseur** — `supplier_invoices.rs:2163` (`project_id: inv.project_id,`, « Le règlement hérite du projet de la
  facture »), chemin commun du paiement direct, du lot pain.001 et du rapprochement (`pay_in_tx_as`, `reconciliation.rs:3426`).
- **Contre-passation** — `journal_entries.rs:2921-2923` lit `… project_id, lettering_key FROM journal_entry_lines WHERE entry_id
  = ? ORDER BY line_order FOR UPDATE`, et `:2942-2951` recopie `project_id: *project_id` ligne à ligne, **dans le même ordre**
  (angle mort du second détecteur, à cause de `*` : lu à la main).
- **Écriture manuelle** — par ligne, à la création (`journal_entries.rs:521`) et à la modification (`:1681`) ; contrôle des
  projets neufs `:1517-1530`.
- **Rapprochement sans pièce** — contrepartie (`kesh-reconciliation/src/manual.rs:110`, `routes/reconciliation.rs:786`, `:797`,
  `:5161`), éclatement par ligne (`split.rs:142`, `:244`, `routes/reconciliation.rs:3811`, `:5556`), projet par défaut d'une règle
  (`:4280`).
- Hors écriture : `services/factures.rs:404`, `:430`, `routes/invoices.rs:740`, `:799`, `routes/supplier_invoices.rs:583` (corps
  de création ou de brouillon) ; `routes/invoices.rs:377`, `routes/supplier_invoices.rs:238`, `routes/journal_entries.rs:223`,
  `services/rapports.rs:*`, `routes/reports.rs:576` (lecture) ; `journal_entries.rs:6093` (test).

**Conséquence** : la story ajoute le **seul** `UPDATE journal_entry_lines SET project_id` du dépôt ; aucun site existant n'est
à corriger, mais tout site qui **lit** le projet de la pièce pour créer une écriture doit le lire **sous le verrou de la pièce**
que le geste prend (D9).

### Les écritures d'une pièce — ce qui les relie, et ce qui les délie

- **Facture client** : `invoices.journal_entry_id` (vente) ; `invoice_settlements.journal_entry_id` (règlements **et** soldes,
  `20260827000001_invoice_settlements.sql`, « journal_entry_id BIGINT NOT NULL », `UNIQUE KEY uq_invoice_settlements_entry`) ;
  `credit_notes.journal_entry_id` (`20260627000001_credit_notes.sql:36`). Une facture `cancelled` l'est par un avoir total
  (`credit_notes.rs:790`, « UPDATE invoices SET status = 'cancelled' »).
- **Facture fournisseur** : `purchase_journal_entry_id`, `settlement_journal_entry_id` (`supplier_invoices.rs:42-45`, colonnes de
  `FIND_SCOPED_SQL`) ; l'annulation contre-passe l'achat et **garde** le lien d'achat (`supplier_invoices.rs:2636-2640` ne vide que
  `settlement_*`) — la contre-passation se retrouve par `journal_entries.reverses_entry_id` (`20260828000002_journal_entries_reversal.sql:44`).
- **Ce qui délie** : l'annulation d'un règlement client **supprime** sa ligne (`invoice_settlements_write.rs:1043`, « DELETE FROM
  invoice_settlements WHERE id = ? AND company_id = ? ») — l'écriture et sa contre-passation ne sont plus reliées à rien ;
  l'annulation d'un paiement fournisseur vide `settlement_journal_entry_id` (`supplier_invoices.rs:2929`) ; une fournisseur annulée
  **alors que payée** garde un paiement « détaché », retrouvé par la seule trace d'audit (`journal_entries.rs:1074-1093`,
  `DETACHED_SETTLEMENT_TRACES`). La dévalidation, client (`invoices.rs:1678`, « journal_entry_id = NULL ») ou fournisseur
  (`supplier_invoices.rs:1399`), **supprime** l'écriture : rien ne reste.
- Ces paires déliées portent l'ancien projet sur l'original **et** sur l'inverse : leur net par projet est nul (agrégat net
  ci-dessus), le détail les montre en `+x`/`−x` (D2).
- **La propriété d'une écriture** se lit par `journal_entries::document_owners` (`journal_entries.rs:2272`) et `DocumentKind`
  (`:2113-2128` : `Invoice`, `CreditNote`, `SupplierInvoice`, `Settlement`, `BankTransaction`).

### Les verrous existants

- `modification_blocker` (`journal_entries.rs:1260`), l'ordre des refus d'un `PUT` d'écriture : exercice clos (`fy_status ==
  "Closed"`, `:1280-1282` — `if fy_status == "Closed"`), exercice postérieur clos (`:1283-1288`), garde de propriété (`modification_guard`, `:1112` —
  `reversal_blockers` hors `AccountArchived`, plus le paiement détaché), verrou de période `books_locked_through` (`:1293-1300`),
  puis la marque de lettrage (`lettering_guard`). Le `PUT` les applique sous verrou (`update`, `:1455` ; verrou de période
  `:1602-1610`, seuil inclusif). **Une écriture rapprochée est donc gelée** (`ReversalBlocker::MatchedBankTransaction`,
  `errors.rs` `ReversalBlocker`, `:59`) : c'est le cas courant des écritures de contrepartie de Guy, que ni le `PUT` ni aucun
  autre geste ne laisse aujourd'hui réimputer.
- Le verrou des livres : `companies.books_locked_through` (`companies.rs:1003`, `pub async fn lock_books`), route
  `PUT /api/v1/companies/current/books-lock` (`lib.rs:564`), levée `…/books-lock/release` (`lib.rs:487`).
- **Aucun de ces verrous n'est en base** (pas de déclencheur) : ils sont tous applicatifs, et le geste ne les traverse pas (D5).

### L'existant à réemployer

- `projects::validate_taggable_in_tx` (`projects.rs:89`) : sentinelle `companies` puis `projects … FOR UPDATE`, `NotFound` (→ `404`)
  ou `IllegalStateTransition("le projet analytique est archivé")` (→ `409 ILLEGAL_STATE_TRANSITION`,
  `routes/journal_entries.rs:888-903`). Ordre des verrous de la modification d'un brouillon : « companies → projects → invoices »
  (`invoices.rs:1128-1133`), et validation **seulement si la valeur change** (`:1143-1144`, « grandfathering »).
- L'acteur : `Acteur` (`entities/audit_log.rs:173`), `impl From<&CurrentUser> for Acteur` (`kesh-api/src/audit.rs:18`),
  `NewAuditLogEntry::par` (`audit_log.rs:263`, « la forme qui ne peut pas oublier la clé »). La garde
  `crates/kesh-api/tests/audit_acteur_garde.rs` compte tout `Acteur::utilisateur(`, `NewAuditLogEntry::user(`, `api_key_id: None`.
- Les libellés d'audit : `crates/kesh-api/src/audit_labels.rs` (liste triée, `:150` « invoice.settlement_cancelled »), clé
  `audit-log-action-…` dérivée par `message_key` (`:229-231`), garde `tests/audit_label_registry.rs` (quatre locales).
- Le registre des routes mutantes : `crates/kesh-api/tests/audit_route_registry.rs` (`:516`, `("post",
  "invoices::unvalidate_invoice_handler", Traced, Rejouee)`), qui rougit pour toute route neuve non examinée.
- Les routes de geste d'une facture sont dans `comptable_routes` (`lib.rs:536`), rejouées sur interblocage
  (`routes/invoices.rs:898`, « retry_on_deadlock("invoices::unvalidate" »).
- L'écran : `canManage` (Admin ou Comptable, `routes/(app)/invoices/[id]/+page.svelte:92-94`), libellé du projet `:1279-1285`
  (`data-testid="invoice-project"`) ; fournisseur `routes/(app)/supplier-invoices/[id]/+page.svelte:812-814`
  (`supplier-invoice-project`) ; écriture `routes/(app)/journal-entries/[id]/+page.svelte:626` (projet par ligne). Le sélecteur
  du brouillon : `InvoiceForm.svelte:811-830` (`<select>`, option « — Aucun », option de repli pour un projet hors liste).

### Le manuel et la documentation de l'API

- `docs/manual/fr/user-manual.tex:3766` (`\section{Comptabilité analytique par projet}`), `:3779-3790` (« Affecter un projet aux
  écritures » : par ligne pour l'écriture manuelle, au document pour les factures, « propagé à toutes les lignes de l'écriture
  générée ») ; rien sur un changement après coup (`git grep -n -iE "changer le (projet|compte analytique)" -- docs/manual/fr`
  vide). La 36-15 en réécrit le vocabulaire : la story ajoute une sous-section **après** sa fusion.
- `docs/api-external.md:1007` : le `PUT /invoices/:id` d'un brouillon « efface le tag analytique » si `projectId` manque ; la
  table des erreurs est au § 10 (`:1034`).

### Migration : aucune (D0)

Colonnes, clés et index existent ; les codes d'action d'audit sont des chaînes (`audit_log.action VARCHAR(64)`,
`20260413000001_audit_log.sql`, contrainte `chk_audit_log_action_nonempty` seule) ; `supplier_invoice.project_reassigned` fait
35 caractères. Aucune table, colonne ni contrainte neuve.

## Décisions

- **D0 — Sans migration.** Tout tient dans les colonnes existantes (ci-dessus). *Écartée* : une colonne `project_reassigned_at`
  sur les pièces — l'audit la remplace. Si le développement découvre qu'il en faut une, **il s'arrête et le dit** (C-e360-3, la
  story passe alors à la 0.19.0).
- **D1 — Trois routes neuves, un geste par objet** (C-36-16-1), dans `comptable_routes`, ouvertes aux clés `read-write`, **pas au
  serveur MCP** (C-e360-16) :
  - `POST /api/v1/invoices/{id}/reassign-project` — corps `{ "projectId": n | null, "version": n, "acknowledgeClosedPeriods"?: bool }` ;
  - `POST /api/v1/supplier-invoices/{id}/reassign-project` — même corps ;
  - `POST /api/v1/journal-entries/{id}/reassign-project` — corps `{ "lines": [{ "lineId": n, "projectId": n | null }], "version": n,
    "acknowledgeClosedPeriods"?: bool }`.
  Les identifiants restent `project*` (C-e360-15 : identifiants internes et champs de l'API gardés) ; seul le texte visible dit
  « compte analytique ». **Changement de contrat de l'API : additif** — trois routes et des codes d'erreur neufs, aucun champ
  retiré ni changé, aucune réponse existante modifiée ; il s'écrit au CHANGELOG comme un ajout, non sous ⚠️. *Écartées* : (a) un
  champ `projectId` accepté par le `PUT` d'une facture validée — le `PUT` est un remplacement complet de brouillon
  (`api-external.md:1007`), l'étendre brouillerait les deux contrats ; (b) une route unique `PATCH /journal-entry-lines` — elle
  laisserait réimputer une ligne de vente sans sa facture.
- **D2 — Les écritures d'une facture** (C-36-16-2) : celles qui lui sont **reliées aujourd'hui** (vente ou achat, règlements et
  soldes en `invoice_settlements`, avoirs, paiement fournisseur courant) **et** la contre-passation de chacune
  (`reverses_entry_id IN (…)`, ce qui couvre l'achat d'une fournisseur annulée). Les paires **déliées** (règlement annulé, paiement
  fournisseur annulé ou détaché) gardent l'ancien compte : leur net par compte analytique est nul, et elles restent réimputables
  par le geste de l'écriture (D4), qui ne les refuse pas (aucune pièce ne les possède). La réponse et l'audit nomment les écritures
  réimputées. *Écartées* : retrouver les paires déliées par l'audit ou le libellé — lecture fragile (le palliatif de #541 en est un
  exemple, `journal_entries.rs:1074-1093`), pour un net nul ; refuser le geste tant qu'il en existe — il bloquerait la facture
  payée après un règlement annulé, cas courant.
- **D3 — Toutes les lignes des écritures d'une pièce prennent le nouveau compte**, `NULL` compris. C'est l'invariant que posent
  les créations (`NewJournalEntry::project_id` stampé sur toutes les lignes) ; le développement vérifie qu'aucun chemin n'écrit une
  ligne de pièce avec un autre projet que celui de la pièce (test T-A4). *Écartée* : ne réimputer que les lignes qui portaient
  l'ancien compte — elle laisserait une ligne `NULL` d'une écriture héritée d'avant l'Epic 19 sans compte, alors que l'utilisateur
  a demandé que la pièce entière change.
- **D4 — L'écriture manuelle, par ligne** (C-36-16-3) : le corps nomme des lignes de **l'écriture visée** (chacune une fois,
  `projectId` éventuellement `null`) ; les lignes non nommées ne changent pas. **Une paire contre-passée se réimpute ensemble** :
  si l'écriture est contre-passée, ou est une contre-passation, la ligne de même `line_order` de l'autre écriture prend le même
  compte (la contre-passation recopie les lignes dans l'ordre, `journal_entries.rs:2921-2951`). **Refus** si l'écriture, ou l'autre
  de la paire, est possédée par une pièce autre qu'une transaction bancaire (`document_owners`, `DocumentKind` ≠
  `BankTransaction`) : `409 PROJECT_REASSIGNMENT_USE_DOCUMENT`, `details { documentKind, documentId, documentLabel }` — le geste est
  celui de la pièce. Une écriture **rapprochée**, **lettrée**, en exercice clos ou en période verrouillée est acceptée (D5). *Écartée* :
  un compte unique pour toute l'écriture — il écraserait la ventilation par ligne que l'écriture manuelle permet
  (`user-manual.tex:3782-3783`).
- **D5 — Exercice clos, exercice postérieur clos, période verrouillée : permis, avec un accusé de réception** (C-36-16-4). Si une
  écriture réimputée est dans un exercice `Closed`, dans un exercice suivi d'un exercice clos, ou datée au plus tard de
  `books_locked_through`, et que le corps ne porte pas `acknowledgeClosedPeriods: true`, le serveur rend **`409
  PROJECT_REASSIGNMENT_CLOSED_PERIOD`** sans rien écrire, `details { entryCount, fiscalYears: [noms], lockedThrough }` ; l'écran
  montre alors l'avertissement (« Les rapports analytiques de l'exercice … changeront ») et renvoie le geste avec l'accusé. Un
  lettrage, un rapprochement ne déclenchent pas d'accusé (ils ne sont pas touchés). *Écartées* : (a) refuser en période close — la
  demande l'exclut (« reste possible », #1167, point 5) ; (b) avertir à l'écran seulement — une clé d'API changerait un exercice clos
  sans l'avoir vu, ce que le patron `expectUnlinkOnly` de la 34-11b refuse déjà ailleurs (`api-external.md:739`).
- **D6 — Une trace d'audit par geste, sur l'objet visé**, par `NewAuditLogEntry::par(Acteur::from(&current_user), …)` :
  `invoice.project_reassigned` / `supplier_invoice.project_reassigned` (entité la facture), `journal_entry.project_reassigned`
  (entité l'écriture). `details` : `fromProjectId`, `toProjectId` (pièce) ou `lines: [{ entryId, lineId, from, to }]` (écriture),
  `journalEntryIds`, `closedPeriodsAcknowledged` (booléen). Libellés aux quatre locales (`audit_labels.rs`). *Écartée* : une trace
  par écriture touchée — dix lignes pour un geste, et la piste perd le geste.
- **D7 — Versions** : la `version` de la facture (ou de l'écriture visée) est **exigée** et lue sous `FOR UPDATE` (patron 34-5,
  `invoice_settlements_write.rs:132-135`) ; écart → `409 OPTIMISTIC_LOCK_CONFLICT`. Le geste incrémente la version de la pièce **et**
  celle de chaque écriture touchée (un `PUT` concurrent sur une écriture manuelle périmée est refusé). *Écartée* : ne pas toucher la
  version des écritures — un `PUT` lu avant le geste réécrirait les lignes avec l'ancien compte, en silence.
- **D8 — Rien à faire, rien d'écrit** : même compte que l'actuel (pièce), ou toutes les lignes nommées déjà à leur valeur (écriture) →
  `200` et l'objet tel qu'il est, sans version incrémentée, sans audit, sans accusé exigé. Le compte **actuel** archivé ne bloque
  pas ; le compte **visé** est contrôlé par `validate_taggable_in_tx` (inconnu ou d'une autre société → `404 NOT_FOUND`, archivé →
  `409 ILLEGAL_STATE_TRANSITION`, mappage existant). `null` retire le compte.
- **D9 — Concurrence et ordre des verrous** : `companies` (sentinelle) → `projects` (par `validate_taggable_in_tx`, seulement si un
  compte visé est non nul) → la pièce `FOR UPDATE` → les lignes des écritures `FOR UPDATE`, dans l'ordre des `id` d'écriture ; le
  handler est rejoué sur interblocage (`retry_on_deadlock`, registre `Rejouee`). Les chemins qui créent une écriture de pièce lisent
  le projet **sous** le verrou de la pièce : établi pour le règlement et le solde client (`:143-145`, `:553-555`) et l'avoir
  (`credit_notes.rs:309`) ; **à établir à l'étape 0 du développement** pour l'encaissement par rapprochement
  (`reconciliation.rs:2814`) et le paiement fournisseur (`supplier_invoices.rs:2163`) — s'il ne l'est pas, la lecture passe sous
  verrou dans la story (limite voisine) ou une issue s'ouvre.
- **D10 — Tout statut à l'API, le bouton hors brouillon** (C-36-16-5) : la route accepte brouillon, validée, payée, annulée ; à
  l'écran, *Changer le compte analytique* apparaît sur une facture non brouillon pour `canManage` (le brouillon garde son formulaire).
  L'écriture manuelle : le bouton apparaît pour un Comptable ou un Admin sur toute écriture que D4 ne refuse pas ; refusée, il est
  absent et la fiche ne change pas.
- **D11 — Vocabulaire** : celui de la 36-15 (« compte analytique » en fr-CH ; de-CH, it-CH, en-CH selon la D2 de sa fiche) ; la
  story n'introduit aucun terme qu'elle ne fixe pas.

## Critères d'acceptation

**Lot A — facture client**

1. **AC-A1** — Une facture client **payée** (vente, deux règlements dont un rapproché, un solde du reste, un avoir partiel) avec le
   compte analytique P : `POST /invoices/{id}/reassign-project { projectId: Q, version }` rend `200`, la facture porte Q, **chaque
   ligne** de la vente, des règlements, du solde et de l'avoir porte Q, et la réponse liste ces écritures (`journalEntryIds`).
2. **AC-A2** — **Rien d'autre ne change** : pour chaque écriture touchée, montants, comptes, dates, numéros, description, lettrage
   (`lettering_key`, `lettering_origin`), rapprochement (`bank_transactions.matched_entry_id`), lignes `invoice_settlements` et
   statut de la facture sont identiques avant et après (comparaison de l'instantané complet hors `project_id`, `version`,
   `updated_at`).
3. **AC-A3** — Après le geste, le rapport « Dépenses » / « Rendement » de Q contient ce que celui de P contenait pour la facture, et
   celui de P ne le contient plus (hors paires déliées, AC-A7).
4. **AC-A4** — Une écriture en **exercice clos** ou en **période verrouillée** : sans accusé, `409 PROJECT_REASSIGNMENT_CLOSED_PERIOD`,
   `details { entryCount, fiscalYears, lockedThrough }`, **rien d'écrit** ; avec `acknowledgeClosedPeriods: true`, `200`, et l'audit
   porte `closedPeriodsAcknowledged: true`.
5. **AC-A5** — Compte visé **archivé** → `409 ILLEGAL_STATE_TRANSITION` ; inconnu ou d'une autre société → `404` ; facture d'une
   autre société → `404` ; version périmée → `409 OPTIMISTIC_LOCK_CONFLICT` ; utilisateur Lecture → `403` ; clé `read` → `403` ; dans
   chaque cas rien n'est écrit. Compte **actuel** archivé : le geste réussit.
6. **AC-A6** — **Audit** : une entrée `invoice.project_reassigned`, `details { fromProjectId, toProjectId, journalEntryIds,
   closedPeriodsAcknowledged }` ; par une clé d'API, `actor_type = 'api_key'` et la clé ; par une session, l'utilisateur. Le journal
   d'audit l'affiche par son libellé dans les quatre locales.
7. **AC-A7** — **Paires déliées** : après l'annulation d'un règlement, le geste ne touche ni ce règlement ni sa contre-passation
   (ancien compte, net nul) ; une facture **annulée** par avoir total se réimpute, vente et avoir compris.
8. **AC-A8** — **Rien à faire** : même compte → `200`, version inchangée, aucune entrée d'audit ; `projectId: null` retire le compte
   de la facture et de toutes ses lignes.
9. **AC-A9** — **À l'écran** (facture payée, Comptable) : *Changer le compte analytique* ouvre un dialogue (compte actuel, sélecteur,
   « — Aucun », comptes archivés absents de la liste sauf l'actuel) ; *Enregistrer* réimpute, le libellé de la fiche change sans
   rechargement et un message confirme ; en période close, le dialogue montre l'avertissement nommant l'exercice ou la date du
   verrou, et seul *Confirmer* envoie l'accusé. Lecture : pas de bouton. Brouillon : pas de bouton.

**Lot B — facture fournisseur**

10. **AC-B1** — Facture fournisseur **payée** (achat, paiement) → toutes les lignes de l'achat et du paiement portent Q ; une facture
    **annulée** → l'achat et sa contre-passation portent Q ; AC-A2, A4, A5, A6 (`supplier_invoice.project_reassigned`), A8 valent ;
    le paiement **détaché** d'une annulée payée n'est pas touché (D2).
11. **AC-B2** — À l'écran de la facture fournisseur : même dialogue, même avertissement, même message ; Lecture et brouillon sans
    bouton.

**Lot C — écriture manuelle**

12. **AC-C1** — Une écriture manuelle **rapprochée**, **lettrée**, ou d'un exercice clos (avec accusé), trois lignes sur deux comptes
    analytiques : `POST /journal-entries/{id}/reassign-project` change les lignes nommées seules ; ligne, lettrage, rapprochement,
    montants intacts (AC-A2) ; `journal_entry.project_reassigned` tracé, acteur compris.
13. **AC-C2** — **Paire** : une écriture contre-passée (ou sa contre-passation) → la ligne de même rang de l'autre écriture prend le
    même compte, et `journalEntryIds` nomme les deux.
14. **AC-C3** — **Refus** : écriture de vente, de règlement, d'avoir, d'achat ou de paiement fournisseur →
    `409 PROJECT_REASSIGNMENT_USE_DOCUMENT` nommant la pièce ; ligne d'une autre écriture, ligne nommée deux fois → `400
    VALIDATION_ERROR` ; rien n'est écrit.
15. **AC-C4** — À l'écran de l'écriture : *Changer le compte analytique* (Comptable, écriture non refusée) ouvre un dialogue d'un
    sélecteur par ligne ; écriture d'une pièce : pas de bouton, le motif existant de la fiche reste.

**Commun**

16. **AC-D1** — Les trois routes sont au registre `audit_route_registry.rs` (`Traced`, `Rejouee`) ; la garde `audit_acteur_garde.rs`
    reste verte **sans entrée neuve** (aucun acteur sans clé ajouté).
17. **AC-D2** — Le **manuel utilisateur** (`.tex` et PDF) dit le geste, ce qu'il touche et ne touche pas, l'avertissement d'exercice
    clos, le cas de l'écriture manuelle et la limite des paires déliées ; `docs/api-external.md` décrit les trois routes, l'accusé,
    l'ordre des refus et les codes neufs au § 10.
18. **AC-D3** — Textes visibles dans les **quatre locales**, au vocabulaire de la 36-15 ; aucun texte en dur hors repli `i18nMsg`.

## Tests à écrire, et la preuve du rouge

Chaque test neuf rougit sans le correctif : la route neuve rend `404` avant elle (premier rouge, journal
`kesh-gate-logs/e36-16/rouges.log`) ; puis, correctif écrit et **commité**, une mutation par propriété (E5 : script versionné dans
`kesh-gate-logs/e36-16/mut/`, restauration `git restore --worktree -- <f>` puis `touch <f>`, journal `mut/<n>.log`).

| Test | Lieu | Mutation qui le fait rougir |
|---|---|---|
| T-A1 réimputation complète d'une facture payée (vente, règlements, solde, avoir) | `crates/kesh-db/tests/project_reassignment.rs` (squash `test-schema`) | retirer `invoice_settlements` (ou `credit_notes`) de l'ensemble des écritures |
| T-A2 instantané hors `project_id` identique (lettrage, rapprochement, montants) | idem | ajouter `lettering_key = NULL` à l'`UPDATE` |
| T-A3 paire déliée intacte ; contre-passation d'une liée réimputée | idem | retirer la jointure `reverses_entry_id` ; l'étendre aux paires déliées |
| T-A4 invariant D3 : après chaque création automatique, toutes les lignes d'une écriture de pièce portent le projet de la pièce | idem | `project_id: None` à la place de `invoice.project_id` dans `credit_notes.rs:718` (sonde : le test doit rougir sur un site existant) |
| T-A5 période close / verrouillée sans accusé → refus sans écriture ; avec accusé → `200` | idem + `kesh-api/tests/project_reassignment_e2e.rs` | court-circuiter le contrôle de période ; inverser le test de `books_locked_through` (`<` au lieu de `<=`) |
| T-A6 acteur : clé d'API → `actor_type = 'api_key'` | `kesh-api/tests/project_reassignment_e2e.rs` | `Acteur::utilisateur(current_user.user_id)` à la place de `Acteur::from` |
| T-A7 refus : archivé, inconnu, autre société, version, rôle, portée | idem | supprimer l'appel à `validate_taggable_in_tx` ; la comparaison de version |
| T-A8 rien à faire → version et audit inchangés | kesh-db | supprimer le court-circuit D8 |
| T-A9 dialogue : rendu, envoi, 409 → avertissement → accusé, succès | Vitest `frontend/src/lib/features/projects/ProjectReassignDialog.test.ts` et `invoice-display-page.test.ts` | ne pas renvoyer `acknowledgeClosedPeriods` ; ne pas rafraîchir le libellé |
| T-B1 fournisseur payée et annulée | kesh-db, kesh-api | retirer `settlement_journal_entry_id` de l'ensemble |
| T-B2 écran fournisseur | `supplier-invoice-display-page.test.ts` | masquer le bouton |
| T-C1 lignes nommées seules, paire par `line_order` | kesh-db | apparier par `id` de ligne au lieu de `line_order` |
| T-C2 refus d'une écriture de pièce, ligne étrangère, doublon | kesh-db, kesh-api | supprimer le contrôle `document_owners` |
| T-C3 écran de l'écriture | `journal-entry-page.test.ts` | afficher le bouton sur une écriture de vente |
| T-D1 libellés d'audit ×4 | `tests/audit_label_registry.rs` (existant) | — (rougit d'office si un code manque) |
| **E2E** `frontend/tests/e2e/project-reassignment.spec.ts` | Playwright | — |

**E2E exigé** (le geste traverse la frontière HTTP et se joue dans le navigateur) : (1) facture client **payée** préparée par
l'API (deux comptes analytiques, validation, règlement), *Changer le compte analytique* au navigateur, libellé de la fiche changé,
puis `GET /journal-entries/{id}` de la vente et du règlement : toutes les lignes au nouveau compte ; (2) même geste après
`PUT /companies/current/books-lock` à une date postérieure : avertissement, *Confirmer*, succès — **levée du verrou en
`finally`** (`…/books-lock/release`), sans quoi les specs suivantes rougissent ; (3) facture fournisseur payée ; (4) écriture
manuelle rapprochée, une ligne changée. Rouge d'abord : la spec rougit sur le correctif retiré (`git stash` de la branche de code,
journal `e2e-rouge.log`).

## Tâches

**Étape 0** — relire les LOW du dernier rapport de validation ; établir D9 pour `reconciliation.rs:2814` et `supplier_invoices.rs:2163`
(ligne lue citée au Change Log) ; vérifier que la 36-15 est fusionnée dans `epic/36-0.18.0` et reprendre ses termes.

**Lot A** (pose le patron)
- [ ] A1 — `kesh-db` : module `repositories/project_reassignment.rs` — `invoice_entry_ids_in_tx` (liées + contre-passations),
  `period_exposure_in_tx` (exercices clos, postérieur clos, verrou), `reassign_lines_in_tx` (`UPDATE journal_entry_lines SET
  project_id = ? WHERE entry_id IN (…)` après `SELECT … FOR UPDATE`, version des écritures +1), `reassign_invoice` ; variantes
  `DbError` (`ProjectReassignmentClosedPeriod`, `ProjectReassignmentUseDocument`) et leurs codes aux **deux** tables de
  `errors.rs` (`:232`, `:1967`).
- [ ] A2 — `kesh-api` : handler `invoices::reassign_project_handler`, route dans `comptable_routes`, `retry_on_deadlock`, acteur
  `Acteur::from(&current_user)`, mappage `AppError` (409 et `details`) ; registre `audit_route_registry.rs` ; `audit_labels.rs`.
- [ ] A3 — `kesh-i18n` : libellés d'audit et textes du dialogue ×4, à côté des blocs `invoice-detail-*` (jamais en fin de fichier) ;
  `i18n-keys.test.ts` (`sitesTotal`) recompté à la source.
- [ ] A4 — frontend : `features/projects/ProjectReassignDialog.svelte` (partagé, un ou plusieurs sélecteurs), appel d'API dans
  `features/invoices`, bouton et dialogue sur `invoices/[id]/+page.svelte`.
- [ ] A5 — Tests T-A1 à T-A9, rouges puis mutations.

**Lot B** — [ ] `supplier_invoice_entry_ids_in_tx`, `reassign_supplier_invoice`, route et handler, écran fournisseur, T-B1, T-B2.

**Lot C** — [ ] `reassign_entry_lines` (paire par `line_order`, refus `document_owners`), route et handler, écran de l'écriture
(un sélecteur par ligne), T-C1 à T-C3.

**Commun** — [ ] E2E (quatre cas) ; [ ] captures (dialogue, avertissement, succès, sur facture payée ; fournisseur ; écriture ;
écriture de pièce sans bouton) dans `kesh-gate-logs/e36-16/captures/`, thème clair ; [ ] manuel utilisateur (sous-section *Changer
le compte analytique après coup*, après la section que la 36-15 renomme), PDF régénéré à l'intégration, contrôlé aplati ; [ ]
`docs/api-external.md` (trois routes, § 10) ; [ ] gates (ci-dessous).

**Gates** (`e35-dev-commun.md` § *Quels gates, quand*, voie 1, sous `flock /tmp/station-heavy.lock flock
/home/gcorbaz/devel/kesh-gate-logs/.gate.lock scripts/mem-guard.sh …`, une commande par prise, `free -g` avant) : `kesh-db` est
touché, donc **gate complet** (`scripts/test-fast.sh --ci`, bases `kesh_e3616` remises à neuf) même entre les passes ; au passage
en `review`, `test-fast --ci`, `scripts/gate-ci.sh kesh_ci_e3616` et le frontend complet ; aux remédiations, `test-fast --ci` (et
frontend si touché) ; à la clôture, `gate-ci` **et** E2E complet sur `kesh_e2e_e3616` recréée, au dernier commit de code. Chaque
chiffre cite son journal (`kesh-gate-logs/e36-16/…`) et son commit.

## Note de version

*Ajouté* — **Changer le compte analytique d'une facture, même payée.** Sur la fiche d'une facture client ou fournisseur
validée, payée ou annulée, *Changer le compte analytique* réimpute d'un geste la facture et toutes ses écritures — vente ou
achat, règlements, solde du reste, avoirs —, sans toucher montant, compte, date, numéro, lettrage ni rapprochement. Le même geste
existe sur une écriture passée, ligne par ligne, même rapprochée ou lettrée. Sur un exercice clos ou une période verrouillée, Kesh
avertit que les rapports analytiques de cet exercice changeront et demande de confirmer. Chaque changement est inscrit au journal
d'audit, avec l'ancien et le nouveau compte. Pour les intégrations : trois routes `POST …/reassign-project` (factures, factures
fournisseur, écritures), ouvertes aux clés en écriture, et les codes `PROJECT_REASSIGNMENT_CLOSED_PERIOD`,
`PROJECT_REASSIGNMENT_USE_DOCUMENT` (refs #1167).

## Dev Notes

### Hors périmètre

- La répartition d'une facture entre plusieurs comptes analytiques, ligne par ligne (#1167, « non demandé »).
- Le renommage du vocabulaire (#1161, 36-15).
- Le serveur MCP (lecture seule, C-e360-16).
- Les paires déliées et le paiement fournisseur détaché (D2) : réimputables par le geste de l'écriture ; aucune issue à ouvrir
  tant que la revue ne le demande pas — si elle juge la limite trop large, issue `enhancement` à demander à l'orchestrateur.
- Le règlement saisi d'une facture client garde l'attribution qu'il a (36-11b, #431) : la story n'y touche pas.

### Modules touchés (règle de découpage du `CLAUDE.md`)

`kesh-db` (dépôt neuf, `errors.rs`) ; `kesh-api/routes/invoices` ; `kesh-api/routes/supplier_invoices` ;
`kesh-api/routes/journal_entries` (avec `lib.rs`, `errors.rs`, `audit_labels.rs`, tests de garde) ; `kesh-i18n` (×4) ;
`frontend/src/lib/features/projects` (dialogue) ; `routes/(app)/invoices/[id]` ; `routes/(app)/supplier-invoices/[id]` ;
`routes/(app)/journal-entries/[id]` ; `docs` (manuel, API). **Dix modules, au-delà de cinq** : découpage proposé au rapport —
**36-16a** facture client (lot A : dépôt, route, dialogue partagé, audit, avertissement, i18n, manuel et API pour la facture ;
cinq modules), **36-16b** facture fournisseur (lot B, trois modules, part de 36-16a fusionnée), **36-16c** écriture manuelle (lot
C, trois modules, part de 36-16a fusionnée ; B et C sans fichier commun hors i18n, manuel et `api-external.md`).

### Fichiers communs avec les autres stories de la 0.18.0

| Fichier | Stories | Règle |
|---|---|---|
| `messages.ftl` ×4, `user-manual.tex`, les trois pages de détail, `InvoiceForm.svelte` | 36-15 (vocabulaire) | **36-16 part de 36-15 fusionnée** (C-e360-16) |
| `invoice_settlements_write.rs` | 36-9 (#1044, solde du reste) | la story n'y écrit rien (D9 établi à la lecture) ; si l'étape 0 le demande, union à l'intégration |
| `crates/kesh-api/tests/audit_acteur_garde.rs` | 36-11a à 36-11d (0.19.0) | aucune entrée neuve ici (AC-D1) |
| `crates/kesh-api/tests/audit_route_registry.rs`, `lib.rs`, `errors.rs` (`kesh-api`, `kesh-db`) | 36-9, 36-10, 36-18 possibles | lignes distinctes, union |
| `docs/api-external.md` | 36-9, 36-10 | sections distinctes |
| `frontend/src/lib/shared/i18n-keys.test.ts` (`sitesTotal`) | toute story qui ajoute un `i18nMsg(` | recompté à la source après chaque fusion (C-e350-18) |
| `CHANGELOG.md` | toutes | **jamais écrit par la branche** (C-e360-11) : la *Note de version* ci-dessus |

### Ce qui doit rester vrai

- Aucun montant, compte, date, numéro, lettrage, rapprochement, ligne `invoice_settlements` ni statut ne change (AC-A2).
- L'équilibre des rapports par compte analytique : une paire contre-passée est toujours sur un seul compte (D2, D4).
- Le `PUT` d'une écriture et sa garde (`modification_blocker`) restent tels quels : le geste est un chemin à côté, non un
  assouplissement du `PUT`.
- Aucun acteur sans clé ajouté (garde `audit_acteur_garde.rs`).
- Aucune donnée réelle dans les tests ni les captures ; rien sous `~/kesh/`.

### Références

`_bmad-output/planning-artifacts/epic-36-premiere-facture-et-suite.md` (rang 15, *Phase 2 — décisions de Guy du 2026-11-15*) ;
`e36-choix-autonomes-e360.md` (C-e360-15, C-e360-16) ; fiche `36-15-compte-analytique.md` (branche
`story/36-15-compte-analytique`, D2) ; #1167, #1161, #1158 (même principe, remarque modifiable en période close), #541 (paiement
détaché) ; stories 19-3, 19-4 (propagation du projet), 34-5 (version lue sous verrou), 34-11b (accusé `expectUnlinkOnly`),
35-8a1 (`NewAuditLogEntry::par`).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-15 — Création de la fiche (Opus 5.5, `bmad-create-story`), code lu à `fed34a9c7`. Inventaire par le symptôme journalisé
  (`kesh-gate-logs/e36-16/inventaire-fiche.log`, sonde positive `invoices.rs:2446`). Aucune migration. Dix modules : découpage en
  trois lots proposé à l'orchestrateur, non écrit. Choix C-36-16-1 à C-36-16-9 au registre de la story.
- 2026-11-15 — **Découpage approuvé par l'orchestrateur** (dix modules ; règle des stories courtes de Guy du 2026-11-15) et écrit :
  `36-16a-reimputation-facture-client.md` (lot A, décisions communes, étape 0 sur `reconciliation.rs:2814`),
  `36-16b-reimputation-facture-fournisseur.md` (lot B, étape 0 sur `supplier_invoices.rs:2163`),
  `36-16c-reimputation-ecriture-manuelle.md` (lot C). `Status: split` (C-36-16-10).
