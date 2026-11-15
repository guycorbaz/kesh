# Epic 36 — registre des choix autonomes de la story 36-16 (clé `36-16-reimputation-analytique`)

Registre propre à la story 36-16 (#1167), **versé par union**. Chaque choix : contexte, option retenue, options écartées,
réversibilité. Code lu à `fed34a9c7`. Fiche : `36-16-reimputation-analytique.md`. Les choix du kickoff
(`e36-choix-autonomes-e360.md`, C-e360-15 et C-e360-16) font foi ; ceux-ci les complètent.

---

### C-36-16-1 — Trois routes `POST …/reassign-project` (fiche, D1)

- **Contexte** : C-e360-16 demande une route dédiée à l'API externe sans la nommer ; C-e360-15 garde les identifiants `project*`.
- **Retenu** : `POST /api/v1/{invoices|supplier-invoices|journal-entries}/{id}/reassign-project`, dans `comptable_routes`, corps
  `projectId` (ou `lines[]` pour l'écriture), `version`, `acknowledgeClosedPeriods` ; verbe à la manière de `write-off` et
  `unvalidate`. Ajout pur au contrat.
- **Écartées** : étendre le `PUT` d'une facture aux validées (contrat de remplacement complet de brouillon) ; une route sur les
  lignes d'écriture (réimputerait une vente sans sa facture) ; un chemin `…/analytic-account` (identifiant public au nouveau
  vocabulaire, contraire à C-e360-15).
- **Réversible** : oui avant publication ; après la 0.18.0, un renommage serait un changement de contrat.

### C-36-16-2 — Les écritures d'une facture : liées et leurs contre-passations ; paires déliées hors du geste (D2)

- **Contexte** : l'annulation d'un règlement client supprime sa ligne `invoice_settlements` (`invoice_settlements_write.rs:1043`),
  celle d'un paiement fournisseur vide le lien (`supplier_invoices.rs:2929`) ; ces paires ne sont plus reliées à la facture.
- **Retenu** : liées aujourd'hui + `reverses_entry_id` de celles-ci ; paires déliées laissées (net nul par compte analytique),
  réimputables par le geste de l'écriture, qui ne les refuse pas.
- **Écartées** : les retrouver par l'audit ou le libellé (lecture fragile, cf. #541) ; refuser le geste tant qu'il en existe.
- **Réversible** : oui ; une story suivante peut étendre l'ensemble sans changer le contrat.

### C-36-16-3 — L'écriture manuelle se réimpute par ligne, la paire contre-passée par `line_order` (D4)

- **Contexte** : #1167 dit « changer le compte analytique de ses lignes » ; l'écriture manuelle porte un projet par ligne
  (`user-manual.tex:3782-3783`) ; la contre-passation recopie les lignes dans l'ordre (`journal_entries.rs:2921-2951`).
- **Retenu** : corps `lines[{lineId, projectId}]` ; la ligne de même rang de l'autre écriture de la paire suit ; refus
  `PROJECT_REASSIGNMENT_USE_DOCUMENT` pour une écriture possédée par une pièce autre qu'une transaction bancaire.
- **Écartées** : un compte unique pour toute l'écriture (écrase la ventilation) ; accepter les écritures de pièce (une vente
  réimputée sans sa facture).
- **Réversible** : oui.

### C-36-16-4 — Période close : accusé de réception exigé, `409 PROJECT_REASSIGNMENT_CLOSED_PERIOD` (D5)

*Annoté le 2026-11-15 : **révisé par C-36-16-11** (l'accusé est réservé à l'Administrateur en session).*

- **Contexte** : #1167, point 5 : « possible sur un exercice clos ou une période verrouillée, avec un avertissement à l'écran ».
  Une clé d'API n'a pas d'écran.
- **Retenu** : sans `acknowledgeClosedPeriods: true`, refus sans écriture, `details { entryCount, fiscalYears, lockedThrough }` ;
  l'écran avertit et renvoie avec l'accusé. Exercice postérieur clos compté comme clos. Patron de `expectUnlinkOnly` (34-11b).
- **Écartées** : refuser ; avertir à l'écran seulement ; une route de lecture préalable (trois routes de plus).
- **Réversible** : oui ; le choix prudent (l'API ne change jamais un exercice clos sans l'avoir dit).

### C-36-16-5 — Tout statut à l'API, bouton hors brouillon (D10)

- **Contexte** : #1167 : « quel que soit leur statut (validée, payée, annulée) » ; le brouillon a déjà son formulaire.
- **Retenu** : la route accepte aussi le brouillon (aucune écriture : seule la facture change) ; l'écran ne montre le bouton que
  hors brouillon, pour Admin et Comptable (`canManage`).
- **Écartée** : refuser le brouillon en `409` — un refus de plus, sans gain.
- **Réversible** : oui.

### C-36-16-6 — Une trace d'audit par geste, sur l'objet visé (D6)

- **Retenu** : `invoice.project_reassigned`, `supplier_invoice.project_reassigned`, `journal_entry.project_reassigned`, par
  `NewAuditLogEntry::par` ; `details` avec ancien et nouveau compte, écritures touchées, accusé.
- **Écartée** : une trace par écriture touchée.
- **Réversible** : oui avant publication (codes d'audit conservés ensuite, `audit_labels.rs`).

### C-36-16-7 — Les versions des écritures touchées s'incrémentent (D7)

- **Retenu** : version de la pièce exigée et incrémentée ; version de chaque écriture touchée incrémentée, pour qu'un `PUT`
  périmé d'une écriture manuelle soit refusé.
- **Écartée** : ne pas toucher les écritures — un `PUT` lu avant le geste remettrait l'ancien compte en silence.
- **Réversible** : oui.

### C-36-16-8 — Rien à faire, rien d'écrit ; `null` retire le compte (D8)

- **Retenu** : même valeur → `200` sans version, sans audit, sans accusé ; compte visé validé par `validate_taggable_in_tx`
  (mappages existants `404` / `409 ILLEGAL_STATE_TRANSITION`) ; compte actuel archivé sans effet.
- **Écartée** : un code neuf `PROJECT_ARCHIVED` (il existe en `failed[]` de la réconciliation, `reconciliation.rs:765`) — le geste
  suit le mappage du brouillon et de l'écriture manuelle, pour un seul comportement par route de geste.
- **Réversible** : oui.

### C-36-16-9 — Story écrite d'un tenant, en trois lots, découpage proposé (fiche, *Modules touchés*)

- **Contexte** : dix modules ; règle du `CLAUDE.md` et consigne de la phase 2 : découpage proposé au rapport, sans l'écrire.
- **Retenu** : une fiche, trois lots A/B/C, chacun lisible seul ; 36-16a (client, pose le patron), 36-16b (fournisseur), 36-16c
  (écriture) proposés à l'orchestrateur.
- **Écartée** : écrire trois fiches (hors du mandat de ce prompt).
- **Réversible** : oui ; la décision revient à l'orchestrateur.

### C-36-16-10 — Découpage écrit en 36-16a, 36-16b, 36-16c (décision de l'orchestrateur du 2026-11-15)

- **Contexte** : C-36-16-9 proposait le découpage ; l'orchestrateur l'a approuvé (règle de splitting préventif : dix modules ;
  règle de Guy du 2026-11-15, stories courtes).
- **Retenu** : la fiche d'origine passe à `split` ; **36-16a** (`ready-for-dev`) pose le patron — dépôt `project_reassignment.rs`,
  route, dialogue partagé à plusieurs rangées (D12), audit, accusé de période close — sur la facture client, cinq modules de code,
  et porte les décisions communes C-36-16-1 à -9 ; **36-16b** (fournisseur) et **36-16c** (écriture manuelle) en partent fusionnée
  (`backlog` jusque-là) et y renvoient. Le point d'étape 0 « projet lu sous le verrou de la pièce » est rangé par pièce :
  `reconciliation.rs:2814` en 36-16a, `supplier_invoices.rs:2163` en 36-16b. Les trois partent de la 36-15 fusionnée pour le
  vocabulaire.
- **Écartées** : garder une story (dix modules) ; un découpage par couche (dépôt, API, écran) — aucune tranche ne livrerait un
  geste utilisable ni ne se testerait de bout en bout.
- **Réversible** : oui, tant qu'aucune n'est développée.

### C-36-16-11 — L'accusé de période close est réservé à l'Administrateur en session (révise C-36-16-4 ; arbitrage de l'orchestrateur, validation P1 de 36-16a, V1-5)

- **Contexte** : la réouverture d'un exercice et la levée du verrou de période sont « Admin uniquement + motif obligatoire »
  (`lib.rs:474-489`, « DÉVERROUILLER DÉFAIT UNE GARANTIE ») ; C-36-16-4 ouvrait l'accusé à tout Comptable et à toute clé
  `read-write`, sans écran pour une clé.
- **Retenu** (choix prudent) : une écriture exposée qui changerait → `403 PROJECT_REASSIGNMENT_CLOSED_PERIOD_ADMIN_ONLY` pour un
  Comptable et pour toute clé d'API, accusé ou non ; un Administrateur en session reçoit `409 PROJECT_REASSIGNMENT_CLOSED_PERIOD`
  sans accusé et réussit avec. En période ouverte, Comptable et clé `read-write` réimputent. Vaut pour 36-16a, b et c.
- **Écartées** : l'accusé ouvert à tous (C-36-16-4 d'origine) ; un motif obligatoire comme la réouverture (non demandé ; ajout
  possible sans changer le contrat).
- **Réversible** : oui ; élargir plus tard ne casse aucun client.

### C-36-16-12 — « Rien à faire » compare la pièce et les lignes ; `entryCount` compte ce qui changerait (V1-14, V1-16)

- **Retenu** : `200` sans écriture seulement si la pièce **et** toutes les lignes de l'ensemble portent déjà le compte visé ; sinon
  le geste réaligne les lignes divergentes. `entryCount` du refus de période = écritures exposées qui changeraient.
- **Écartée** : comparer à la seule pièce — une dérive historique ne se réparerait jamais.
- **Réversible** : oui.

### C-36-16-13 — Lectures verrouillantes et ordre des refus (V1-9, V1-13)

- **Retenu** : sentinelle `companies` toujours (elle porte `books_locked_through` et sérialise la pose du verrou) → pièce `FOR
  UPDATE` → version → ensemble relu sous verrou → en-têtes et lignes `FOR UPDATE` par `id` → rien à faire → compte visé (validé
  seulement s'il diffère de l'actuel) → l'exercice de **chaque écriture qui changerait** `FOR UPDATE` par `start_date` croissant,
  statut lu sous ce verrou, puis `find_later_closed_in_tx` → `403` / `409` de période → écritures, audit, `COMMIT`. *(Corrigé à la
  validation P2 de 36-16a, V2-1 : la version du P1 disait « exercices exposés `FOR UPDATE` », ce qui obligeait à lire le statut
  hors verrou.)* Cycles connus, bornés par le rejeu des deux côtés : facture ↔ `companies` (validation, règlement, solde,
  annulation, avoir) et exercice ↔ `companies` (acceptation par rapprochement) — fiche 36-16a, D9.
- **Écartée** : lire exercice et verrou sans verrou, comme `modification_blocker` — une clôture concurrente commiterait entre le
  contrôle et l'écriture.
- **Réversible** : oui.
