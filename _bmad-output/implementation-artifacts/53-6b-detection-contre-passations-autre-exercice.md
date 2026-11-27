# Story 53-6b : Retrouver les contre-passations d'annulation de rapprochement tombées dans un autre exercice (#1674)

Status: backlog

*Fiche **née du découpage de la 53-6 le 2026-11-27** (décision du Project Lead, signal de recyclage de la validation P2). **Dépend de la 53-6a** (`53-6a-annulation-rapprochement-date-origine.md`, le correctif : sans lui, la détection listerait un stock qui continue de grossir). Epic 53 « Écritures justes » — **sortie proposée vers un epic suivant** (borne de six stories, `epic-53-ecritures-justes.md` ; l'orchestrateur la place). Priorité de la 53-6 : P1 (#1674) ; la détection est la partie « livres déjà touchés ». **Aucune migration.** **Sa validation n'a pas eu lieu** : la fiche est reprise telle que la fiche d'origine la portait (D7, D8, AC-14, AC-16 (3)) et **doit être validée (P1 à P-n) avant tout développement**, avec les constats de la P2 listés ci-dessous. Décisions du Project Lead du 2026-11-27 : Q4 — la détection se fait ici ; Q6 — on ne rouvre pas un exercice clos.*

## Story

En tant que comptable dont Kesh a daté, avec les versions précédentes, des contre-passations d'annulation de rapprochement dans un autre exercice que leur écriture, je veux **retrouver** ces cas dans Kesh et savoir comment les corriger moi-même par une écriture, afin que mes livres soient justes sans que Kesh corrige rien d'office.

## Prémisses reprises de la fiche d'origine (à relire au code à T0)

**P9 — une contre-passation ne se contre-passe pas, une écriture contre-passée non plus (confirmée) : la correction des livres touchés se
saisit à la main.** `ReversalBlocker::IsAReversal` et `AlreadyReversed` (`errors.rs:272`, variantes 1 et 2) ; `reversed_by` dérivé de
l'`UNIQUE` de `reverses_entry_id` (`journal_entries.rs:2784`). Ni l'origine E ni son inverse R ne peuvent être contre-passées de nouveau
par Kesh : la correction (D8) passe par deux écritures OD saisies.

**P10 — la piste d'audit distingue l'annulation de rapprochement des autres contre-passations (confirmée).** `reconciliation_cancel.rs:
974-1006` : `reconciliation.cancelled`, entité `bank_transaction`, détails `kind`, `matchedEntryId`, `reversalJournalEntryId` (nul pour un
retrait seul), `unlinkOnly`, `invoiceId`, `settlementId`, `supplierInvoiceId` ; pour un paiement Wise groupé, une entrée par transaction
remise en attente, **même** `reversalJournalEntryId`, `groupedWith` (`:1008-1028`). Une contre-passation de `/journal-entries/{id}/reverse`
ou d'une annulation de règlement depuis la fiche n'a pas cette entrée. C'est le critère de la requête de lecture que le Project Lead a lancée
sur sa base (dossier hors dépôt ; **aucun résultat n'en est recopié ici**) et celui de l'outil de détection (D7).

**Décision Q6 (2026-11-27).** On ne rouvre pas un exercice clos : pour un cas dont l'exercice d'origine a été **clôturé et ses comptes approuvés**, le manuel dit de **ne pas rouvrir** — l'inverse tombée dans l'exercice suivant y vaut alors correction d'un exercice antérieur, à laisser, éventuellement commentée par une écriture de texte ; il ne propose la correction que pour un exercice d'origine **ouvert** (jamais clôturé, ou rouvert) ou clôturé sans comptes déposés. Le choix reste à l'utilisateur et à sa fiduciaire ; Kesh ne corrige rien d'office.

## Constats de la validation P2 de la fiche d'origine à reporter ici (M3 de la P2, M7 et M8 de la P1)

- **M3 (P2) — un panneau permanent.** Sous (a2) (aucun acquittement), le cas **ne quitte jamais** le panneau : la correction D8 ajoute deux écritures OD et ne déplace pas l'inverse R, qui reste dans l'exercice M (P9 : ni E ni R ne se contre-passent). La formule « tant que l'inverse reste dans un autre exercice que l'écriture » laissait croire à une sortie qui n'arrive pas : elle est **retirée**. Le texte du panneau dit au contraire : « Kesh ne peut pas savoir si ce cas a été corrigé : il reste listé après correction. » Coût à peser en validation : un panneau **permanent** sur la page des imports pour toute installation touchée (celle de Guy l'est), que l'on finit par ne plus lire. Recommandation de la fiche d'origine : (a2), en le disant ainsi, et l'acquittement (a1) (action d'audit neuve, route `POST` mutante à inscrire à `audit_route_registry.rs`, aucune migration) en issue ; si le panneau permanent est jugé inacceptable, (a1) entre dans la story, ou la détection se réduit à la procédure et à la requête au manuel d'administration (Q4 (b), écartée par la décision du 2026-11-27 mais à rouvrir si (a1) alourdit trop).
- **M7 (P1) — portée par société lue sur l'écriture**, non sur l'audit (D7).
- **M8 (P1) — sortie d'un cas du panneau** : voir M3.

## Décisions

- **D7 — Détection en lecture des livres déjà touchés (Q4 a).** Dépôt : `reconciliation_cancel::cross_fiscal_year_cancellations(pool,
  company_id)` → `Vec<CrossFiscalYearCancellation>`, une ligne par **contre-passation** (dédoublonnée sur `reversalJournalEntryId` : un
  paiement Wise groupé écrit une entrée d'audit par transaction, P10), pour les entrées `reconciliation.cancelled` dont
  `reversalJournalEntryId` n'est pas nul et dont l'inverse et l'origine (`reverses_entry_id`) sont dans deux exercices différents ; champs :
  transaction bancaire, instant de l'annulation, genre, origine (id, numéro, date, exercice id/nom/statut), inverse (idem), total des
  débits de l'origine. **Portée par société lue sur l'écriture, non sur l'audit** (validation P1, M7) : `audit_log.company_id` est nullable (« NULL = société
  indéterminable, état permanent », `crates/kesh-db/test-schema/0001_schema_squash.sql:192`) — filtrer dessus **omettrait en silence** des cas ;
  la requête joint `journal_entries` (origine, `company_id = ?`) et ne lit l'audit que pour la nature du geste (`details_json`, colonne de
  `audit_log`, non `details`). Index de `audit_log` : `(entity_type, entity_id)`, `(user_id, created_at)`, `(company_id, created_at)` — aucun
  sur `action` : le plan filtre `entity_type = 'bank_transaction'` puis `action` ; à mesurer par `EXPLAIN` comme dit plus bas.
  **Lecture seule** : une transaction de lecture, aucun verrou, aucune écriture (pas même d'audit). Route
  `GET /api/v1/reconciliation/cancellations/cross-fiscal-year` (Comptable+ comme la vue ; clés `read` admises), tri par instant. Écran : un
  panneau dans `frontend/src/lib/features/reconciliation/` (`CrossFiscalYearCancellations.svelte`), monté sur la page des imports
  (`routes/(app)/bank-import/+page.svelte`), **absent** quand la liste est vide, qui dit pour chaque cas « L'écriture n° { $n } du { $date }
  (exercice { $fy }) a été contre-passée le { $date2 } dans l'exercice { $fy2 } » avec deux liens *Voir l'écriture* et un renvoi au
  manuel (§ D8). ⚠️ Avant d'écrire la requête : `EXPLAIN` sur `audit_log` filtré par `action` et `entity_type` (index existants ? la table
  grossit sans fin) — si le plan parcourt toute la table, le dire au Dev Agent Record et ouvrir une issue de performance plutôt que d'ajouter
  un index (ce serait une migration). ⚠️ `JSON_VALUE` existe sous MariaDB 10.11 (base de CI) ; vérifier que les entrées d'audit antérieures
  portent toutes `reversalJournalEntryId` (clé posée depuis la 25-3-b, à confirmer au code de cette story).
- **D8 — La correction se fait par l'utilisateur, par deux écritures, jamais d'office.** Procédure au manuel (nouvel alinéa du § *Annuler un
  rapprochement*, « Une contre-passation tombée dans un autre exercice ») : pour une origine E (exercice N, date dE) dont l'inverse R est dans
  l'exercice M : si N est **ouvert** (jamais clôturé, ou rouvert — Q6), saisir dans N, à la date dE, une écriture OD aux lignes de E
  inversées (« Correction : contre-passation de l'écriture n° … dans son exercice ») ; puis dans M, à la date de R, une écriture OD aux
  lignes de E (« Correction : annule la contre-passation n° … passée dans le mauvais exercice »). N porte alors E et son inverse, M porte R et
  son inverse ; aucun total ne change ailleurs. Kesh ne peut pas les générer : E est déjà contre-passée, R est une contre-passation (P9). ⚠️
  **À vérifier et à dire au manuel** (T6) : les deux écritures de correction touchent le compte du grand livre de la banque — apparaissent-
  elles parmi les écritures candidates d'un rattachement (`GET …/entry-candidates`) ou comme « déjà comptabilisée » ? Si oui, le manuel dit
  de ne pas les rattacher (elles ne portent aucun mouvement bancaire) ; si le risque de faux rattachement est réel, ouvrir une issue (CLAUDE.md
  § *Qualité du code* : « le faux rattachement est muet »). Lettrage : sur un compte lettrable (débiteurs, créanciers), les lignes des deux
  écritures de correction se soldent entre elles ; le manuel dit de les lettrer ensemble (`POST /letterings`, ou l'écran du lettrage). Si N
  est clos et ses comptes déposés : ne rien corriger (Q6).

## Critères d'acceptation

Tests d'intégration en `kesh-db` et de bout en bout en `kesh-api` (motif de la 53-6a) ; les cas s'obtiennent en écrivant l'**état ancien par SQL direct** (aucun chemin de production ne le produit plus après la 53-6a).

- **AC-14 — Détection (D7)** : la route rend un cas par contre-passation tombée dans un autre exercice (jeu : un cas `entry`, un cas
  `invoice_settlement`, un paiement Wise groupé de deux transactions → **un** cas, une annulation dans le même exercice → aucun, un retrait
  seul → aucun, une contre-passation par `/reverse` dans un autre exercice → aucun) ; elle n'écrit rien (compte de `journal_entries`,
  `audit_log` avant et après) ; elle est scopée à la société ; le panneau est absent sans cas. Les cas s'obtiennent en écrivant l'état ancien
  par SQL direct dans le test (aucun chemin de production ne le produit plus).
- **AC-2 — Manuel** (`docs/manual/fr/user-manual.tex`, § *Annuler un rapprochement*, nouvel alinéa « Une contre-passation tombée dans un autre exercice » : la procédure D8 (Q6 comprise), la précision sur les écritures candidates d'un rattachement (T3), le lettrage) ; guide de l'API (`docs/api-external.md`, route de détection) ; quatre manuels (DE/IT/EN) selon la règle du dépôt ; les PDF ne se commitent pas dans `story/…`. Recherche du symptôme dans les `.tex` **et** les PDF aplatis.
- **AC-3 — E2E Playwright** : le panneau de détection (état ancien posé par l'endpoint de test ou par SQL) liste le cas et mène aux deux écritures ; recette visuelle : capture du panneau sur une base de démonstration, dans `kesh-gate-logs/`.
- **AC-4 — Gates et mutations** : gate complet, frontend, E2E complet au dernier commit de code ; mutation M1 (dédoublonnage retiré : un paiement Wise groupé donne deux cas) tuée par l'AC-1.
- **AC-5 — Aucune migration** : `git diff --stat <base>..HEAD -- crates/kesh-db/migrations/` vide ; schéma de test inchangé.

## Tâches

- [ ] **T0 — Validation de la fiche** (P1 Opus puis suite, règle de boucle) et reprise de `main` ; relire P9, P10, D7, D8 au code ; confirmer la présence de `reversalJournalEntryId` dans toutes les entrées d'audit.
- [ ] **T1 — Détection** : `EXPLAIN` d'abord, dépôt, route (`crates/kesh-api/src/lib.rs`, à côté de `:1095-1129`), tests AC-1 ; commit.
- [ ] **T2 — Écran** : `CrossFiscalYearCancellations.svelte`, montage sur `bank-import/+page.svelte`, i18n quatre locales, `sitesTotal` recompté, tests unitaires ; commit.
- [ ] **T3 — Vérification D8** (écritures candidates d'un rattachement) puis manuel et API (AC-2) ; commit.
- [ ] **T4 — E2E** (AC-3), recette visuelle, gates, mutation (AC-4) ; Dev Agent Record ; commit et push de la seule branche `story/…`.

## Dev Notes

- Fichiers : `crates/kesh-db/src/repositories/reconciliation_cancel.rs` (requête), `crates/kesh-api/src/routes/reconciliation.rs` et `lib.rs` (route), `crates/kesh-i18n/locales/*/messages.ftl`, `frontend/src/lib/features/reconciliation/` (panneau), `frontend/src/routes/(app)/bank-import/+page.svelte`, `docs/manual/fr/user-manual.tex`, `docs/api-external.md`. Modules : `kesh-db`, `kesh-api`, `kesh-i18n`, `frontend/…/reconciliation` (quatre).
- Ne pas écrire d'IBAN, de solde ni de ligne du relevé réel de Guy dans les tests ni la fiche (mémoire « données réelles hors dépôt public »).
- **Notes de version** : « La page des imports bancaires liste les annulations de rapprochement tombées, avec les versions précédentes, dans un autre exercice que leur écriture ; le manuel dit comment les corriger. Route nouvelle `GET /api/v1/reconciliation/cancellations/cross-fiscal-year`. » Aucune migration.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-27 — **Fiche née du découpage de la 53-6** (décision du Project Lead) ; statut `backlog`, validation à venir ; dépend de la 53-6a ; sortie vers un epic suivant proposée au fichier d'epic.
