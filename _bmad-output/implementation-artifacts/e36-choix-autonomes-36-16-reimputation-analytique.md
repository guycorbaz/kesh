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
