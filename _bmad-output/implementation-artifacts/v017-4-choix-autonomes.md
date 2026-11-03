# Release 0.17.0 — registre des choix autonomes de la Story v017-4 (v017-4a et v017-4b)

Registre propre à la story (`docs/agents-orchestration.md` § 1), à verser **par union** au merge. Fiche parente :
`v017-4-facture-fournisseur-brouillon.md` ; sous-fiches `v017-4a-facture-fournisseur-brouillon-backend.md` et
`v017-4b-facture-fournisseur-brouillon-ecran.md`. Numérotation `C-v017-4-k`, commune aux deux sous-stories. Issues en
`refs` seulement dans les commits. Les numéros de ligne cités sont ceux de `8e126cfd` (tête de `release/0.17.0`,
candidate `v0.17.0-rc1`). Chaque choix : contexte, option retenue, alternatives écartées, réversibilité. Les choix
marqués **(Q)** figurent aussi parmi les questions au Project Lead (fiche parente, § *Questions au Project Lead*).

Rédigé le 2026-10-31 en autonomie (Opus 5.5), sous les contraintes du Project Lead : les données de l'installation du
NAS se gardent (`kesh-gate-logs/prompts/v017-donnees-a-garder.md`) — elle tourne désormais en `v0.17.0-rc1` et porte
des livres réels ; aucune écriture passée ne se réécrit ; un appariement propose, il ne crée jamais.

---

### C-v017-4-1 — Découper : v017-4a (backend, migration) et v017-4b (écran, manuels)

- **Contexte** : la story touche `kesh-db` (migration, dépôt des factures fournisseur, contrôles de clôture,
  journal, lettrage, lots), `kesh-api` (routes, erreurs, audit, MCP, exports, registres), `kesh-i18n`, le `frontend`
  (liste, fiche, formulaire, page d'édition, écran de clôture), les manuels FR et l'API externe : bien plus de cinq
  modules (`CLAUDE.md` § *Règle de splitting préventif*, premier critère). Aucun cycle Cargo, et le backend se teste
  seul en HTTP.
- **Retenu** : une fiche parente qui porte le contexte, les décisions, le contrat et **tous** les AC ; deux sous-fiches
  qui les répartissent par numéro (table de correspondance, comme la v016-2) : **4a** = migration, dépôt, routes, API,
  contrôle de clôture, tests Rust, textes de la migration ; **4b** = écran, formulaire (et ses quatre défauts, C-v017-4-20),
  i18n de l'écran, manuel utilisateur, E2E, captures. Même worktree, même branche ; la 4a d'abord.
- **Écartées** : (b) story unique — dérogation non justifiable ici ; (c) une fiche par sous-story rédigée en entier —
  doublerait le texte des AC, que deux passes devraient tenir égaux.
- **Réversible** : oui (organisation).

### C-v017-4-2 — Le statut stocké `open` garde son sens « validée, à payer » ; seul `draft` s'ajoute

- **Contexte** : le mandat dit « les factures existantes deviennent `validated` ». Trois façons de l'entendre. Le code
  filtre partout sur `status = 'open'` (propositions de réconciliation `reconciliation.rs:825`, `:885`, `:935` ; lots
  `payment_batches.rs:400` ; règlement `supplier_invoices.rs:826`, `:1187` ; écran des lots
  `payment-batches/+page.svelte:53`, `:91`) ; une sauvegarde `.keshbackup` de la 0.16.x ou de la rc1 porte des lignes
  `open`.
- **Retenu** : `status` gagne la seule valeur `draft` ; `open` reste la valeur stockée d'une facture **validée et non
  payée** ; aucune ligne existante n'est réécrite. Les filtres `= 'open'` excluent les brouillons **d'eux-mêmes**, et une
  sauvegarde 0.16.x ou rc1 se restaure sans conversion ni rejeu. À l'écran, le libellé reste « Ouverte » (glossaire,
  partie A, `docs/i18n-glossaire.md:93`) et l'action s'appelle « Valider » : le manuel dit « validée, elle est ouverte ».
- **Écartées** : (b) renommer `open` en `validated` par un `UPDATE` — réécrit toutes les lignes de Guy, impose un
  backfill trié P7 **et** un `CHECK` qui accepte encore `open` (sans quoi l'import d'une sauvegarde 0.16.x échoue à
  l'insertion, avant tout rejeu), et touche tous les filtres et le contrat de l'API ; (c) libellé « Validée » à l'écran —
  propagation à toutes les occurrences de « ouverte » des manuels (réconciliation, lots, sous-centime), sans gain
  comptable ; reste possible plus tard (question Q4).
- **Réversible** : oui — un renommage reste possible par une migration future ; le libellé est une clé i18n.

### C-v017-4-3 — Une migration breaking au sens de P1, sans nouveau bump de version

- **Contexte** : un brouillon n'a pas d'écriture : `purchase_journal_entry_id BIGINT NOT NULL`
  (`20260628000001_supplier_invoices.sql:42`) doit devenir nullable, et `chk_supplier_invoices_status` (`:65-66`)
  admettre `draft`. Un binaire antérieur décode la colonne en `i64` (`entities/supplier_invoice.rs:39`) : un brouillon le
  ferait échouer — **breaking** (P1). `MODIFY COLUMN` est un déclencheur de P3. `kesh_version_min_required` vaut déjà
  `0.17.0` dans cette release (`20261030000001_bank_imports_manual_source.sql`, dernière ligne), et les dix crates sont
  en `0.17.0` (`crates/kesh-api/Cargo.toml:3`).
- **Retenu** : migration `20261031000001_supplier_invoices_draft.sql` — `MODIFY COLUMN … BIGINT NULL`, `CHECK` du
  statut refait avec `draft`, deux `CHECK` neufs (un brouillon n'a ni écriture d'achat ni règlement ; une facture non
  brouillon a son écriture d'achat), puis, **en dernière instruction**, `UPDATE _kesh_version SET
  kesh_version_min_required = '0.17.0' WHERE id = 1;` — même valeur, exigé par P2/P3 et sans effet. Aucun bump Cargo
  (P2-bis tenu : déjà 0.17.0). P7 : `EXEMPT_MIGRATIONS`, `Durable`. Aucune ligne existante touchée.
- **Écartées** : (b) écriture « fantôme » à zéro pour les brouillons — fausse le journal et la numérotation ; (c) table
  séparée des brouillons — double toute la lecture (liste, fiche, MCP, export) ; (d) relever `min_required` à une
  version au-delà de 0.17.0 — exigerait un bump des crates hors de la release (`0.17.1`).
- **Conséquence écrite (limite L1)** : la rc1 et la rc2 portent la même version Cargo `0.17.0` ; `min_required` ne
  peut pas les distinguer. Une sauvegarde de la rc2 qui porte un brouillon **ne s'importe pas** dans la rc1 (le `CHECK`
  de la rc1 refuse `draft`) ; la rc1 refuse de démarrer sur une base passée par la rc2 (migration inconnue de `sqlx`). Le
  retour de la rc2 à la rc1 ne passe que par la sauvegarde d'avant la mise à jour — c'est la règle de toute mise à jour,
  dite dans les notes de la candidate et le manuel d'administration.
- **Réversible** : non une fois publiée (P8) ; avant publication, oui.

### C-v017-4-4 — Créer, c'est enregistrer un brouillon ; les contrôles liés à l'écriture passent à la validation

- **Contexte** : aujourd'hui `create_in_tx` (`supplier_invoices.rs:388-651`) contrôle et écrit d'un seul geste : forme
  des lignes, fournisseur, projet, comptes de charge, réglages, comptes désignés, exercice ouvert, écriture d'achat.
  Guy veut corriger avant que rien ne soit passé.
- **Retenu** : `POST /supplier-invoices` et la complétion d'un document créent un **brouillon** sans écriture. À
  l'enregistrement (création et `PUT`), Kesh juge ce qui ne dépend que de la saisie : lignes non vides, échelle,
  forme (`purchase_lines_form`), coordonnées de paiement, échéance (C-v017-4-12), fournisseur de la société marqué
  fournisseur, projet utilisable, et les comptes de charge (existants, de la société, actifs, de type charge,
  imputables — même passe que la validation, une seule fonction). À la **validation**, Kesh rejuge le tout sous verrous
  puis juge ce qui dépend de l'écriture : réglages (compte créanciers), comptes désignés, exercice ouvert couvrant la
  date, garde à l'usage — l'ordre et les verrous de l'actuel `create_in_tx`, précédés du verrou de la facture. Un
  brouillon peut donc être daté dans un exercice qui n'existe pas encore.
- **Écartées** : (b) ne rien contrôler à l'enregistrement — le brouillon accepterait un compte d'une autre société ou
  un actif, et le refus n'arriverait qu'à la validation ; (c) tout contrôler à l'enregistrement, exercice compris — un
  brouillon ne pourrait pas attendre la création de l'exercice.
- **Réversible** : oui.

### C-v017-4-5 — `PUT /supplier-invoices/{id}` : remplacement complet d'un brouillon, avec la règle du document

- **Contexte** : le client a `PUT /invoices/{id}` (brouillon, `version`). Un brouillon issu d'un document **avec QR** a
  ses coordonnées de paiement prises du QR (`routes/imported_supplier_invoices.rs:209-246`) et son TTC égal au montant du QR
  (`:262-288`, `AMOUNT_MISMATCH`). Un document se joint aussi après coup (`attach_in_tx`, sans contrôle de montant).
- **Retenu** : `PUT` remplace en entier l'en-tête et les lignes d'un **brouillon** (corps = celui de la création +
  `version`), sous verrou optimiste. Les règles « document » de la complétion (devise CHF, routage IBAN/QR-IBAN et
  cohérence QRR, coordonnées prises du document, `QR_FIELDS_FROM_DOCUMENT`, `AMOUNT_MISMATCH`) sont **extraites** en une
  fonction unique, appelée par la complétion **et** par le `PUT` quand le brouillon a un document joint : la règle vaut
  pour le document, quel que soit le chemin qui l'a joint. Un brouillon sans document, ou à document sans QR, se modifie
  comme une saisie.
- **Écartées** : (b) `PATCH` champ par champ — deux sémantiques pour un seul formulaire ; (c) laisser les coordonnées
  modifiables malgré le QR — défait la garde de la 30-2b.
- **Réversible** : oui.

### C-v017-4-6 — Supprimer un brouillon : Administrateur seul, document rendu « à traiter », historique de lot opposé

- **Contexte** : client — `DELETE /invoices/{id}` réservé à l'Administrateur, brouillon seul (`lib.rs:299-308`), refus
  `INVOICE_MUST_BE_UNVALIDATED_FIRST` pour une facture validée. Côté fournisseur, `payment_batch_items.supplier_invoice_id`
  est `ON DELETE RESTRICT` (`20260628000002_payment_batches.sql:54`) et `imported_supplier_invoices.supplier_invoice_id`
  `ON DELETE SET NULL` (`20260629000001_imported_supplier_invoices.sql:65`) : supprimer la facture laisserait le
  document `completed` sans facture, invisible.
- **Retenu** : `DELETE /supplier-invoices/{id}`, **brouillon seul**, monté avec les routes Administrateur (fermé aux
  clés, comme le client). Dans la même transaction : le document joint repasse « à traiter » par `detach_in_tx`
  (`imported_supplier_invoices.rs:321`) et son audit `imported_supplier_invoice.detached` ; puis la facture est supprimée
  (lignes en cascade), audit `supplier_invoice.deleted` avec l'instantané. Refus : facture ouverte →
  `409 SUPPLIER_INVOICE_MUST_BE_UNVALIDATED_FIRST` ; payée ou annulée → `409 ILLEGAL_STATE_TRANSITION` ; brouillon qui
  figure dans un lot (tout statut — un brouillon dévalidé peut en garder l'historique) →
  `409 SUPPLIER_INVOICE_HAS_PAYMENT_BATCH_HISTORY`, au lieu d'une erreur de clé étrangère opaque.
- **Écartées** : (b) Comptable+ — asymétrie avec le client sans motif ; (c) supprimer aussi le document — perte d'une
  pièce reçue.
- **Réversible** : oui.

### C-v017-4-7 — Valider : `POST /supplier-invoices/{id}/validate`

- **Contexte** : client — `POST /invoices/{id}/validate` sans corps (`routes/invoices.rs:815`), Comptable+, clés admises.
- **Retenu** : même route côté fournisseur, Comptable+, clés `read-write`, rejouée sur interblocage, **corps
  `{ "version": n }`** (le brouillon se modifie : la validation doit porter sur ce que l'utilisateur a relu — écart voulu
  avec la validation client, qui ne prend pas de corps). Passe l'écriture d'achat par le générateur actuel
  (`generate_purchase_journal_lines`, `supplier_invoices.rs:120`), sans changement d'écriture ; `draft → open` ; audit
  `supplier_invoice.validated` (`journalEntryId`). Les refus de la création actuelle s'y retrouvent (exercice,
  comptes, réglages, projet), plus `ILLEGAL_STATE_TRANSITION` (non brouillon) et `OPTIMISTIC_LOCK_CONFLICT`.
- **Écartées** : (b) sans `version`, comme le client — une validation pourrait passer une version modifiée entre-temps
  par un autre onglet.
- **Réversible** : oui.

### C-v017-4-8 — Dévalider : suppression de l'écriture d'achat, la règle de la facture client

- **Contexte** : la règle **déjà retenue** pour la facture client est la **suppression** de l'écriture, non sa
  contre-passation : `invoices::unvalidate` (`repositories/invoices.rs:1501`), doc-comment « son écriture comptable est
  supprimée (Story 25-2-b-1, #440) », par `journal_entries::delete_in_tx(…, enforce_ownership = false)`
  (`journal_entries.rs:1819`), qui garde l'exercice clos, l'exercice postérieur clos, l'écriture contre-passée, le
  verrou de période et le lettrage ; la numérotation garde un trou, assumé et décrit au manuel
  (`user-manual.tex:740-755`, « Deux chemins en creusent »).
- **Retenu** : `POST /supplier-invoices/{id}/unvalidate`, corps `{ "version": n }`, Comptable+, clés `read-write`,
  rejouée. `open → draft`, `purchase_journal_entry_id → NULL` dans **un seul** `UPDATE`, puis `delete_in_tx` sur
  l'écriture d'achat (deuxième appelant de production du drapeau `false`, à nommer dans son doc-comment). Empêchements,
  dans l'ordre : statut (payée → `409 SUPPLIER_INVOICE_PAID`, qui nomme le geste « annuler le règlement d'abord » ;
  brouillon ou annulée → `409 ILLEGAL_STATE_TRANSITION`), version, lot `generated` (`409 SUPPLIER_INVOICE_IN_PAYMENT_BATCH`,
  code existant), écriture d'achat rapprochée d'une transaction (`409 MATCHED_BANK_TRANSACTION`, code existant —
  indispensable parce que `delete_in_tx` sans propriété ne lit plus `modification_guard` et que
  `bank_transactions.matched_entry_id` est `ON DELETE SET NULL`), puis ceux de `delete_in_tx`. Audit
  `supplier_invoice.unvalidated` avec l'instantané et l'identifiant de l'écriture supprimée, plus le
  `journal_entry.deleted` de `delete_in_tx`.
- **Écartées** : (b) contre-passation — laisserait deux écritures par correction, précisément ce que Guy ne veut plus,
  et contredirait la règle client ; (c) refuser la dévalidation d'une facture qui a l'historique d'un lot confirmé —
  le règlement a déjà été annulé explicitement ; la fiche montre ce lot.
- **Réversible** : oui (comportement).

### C-v017-4-9 — Payée, la facture ne se modifie plus ; un brouillon ne s'annule pas

- **Retenu** : rien ne change pour une facture payée : pour la corriger, annuler le règlement (elle redevient
  ouverte), puis la dévalider. Annuler un brouillon est refusé : `supplier_invoice_cancel_blocker`
  (`supplier_invoices.rs:1269`) gagne une tête « brouillon » (variante `SupplierInvoiceDraft`, code
  `SUPPLIER_INVOICE_DRAFT`) ; la fiche n'offre pas « Annuler » sur un brouillon, qui se supprime.
- **Écartées** : (b) annuler un brouillon = le supprimer — deux verbes pour un geste.
- **Réversible** : oui.

### C-v017-4-10 — Complétion d'un document : un brouillon, document attaché, QR en lecture seule

- **Retenu** : `POST /imported-supplier-invoices/{id}/complete` crée un **brouillon** et passe le document `completed`
  lié à lui, comme aujourd'hui (`mark_completed`). Ses coordonnées de QR restent en lecture seule au brouillon
  (C-v017-4-5). La réponse est inchangée en forme (la facture relue), son `status` vaut `draft`.
- **Écartées** : (b) complétion = facture validée — deux comportements pour « créer », et la pièce importée est
  précisément celle où Guy s'est trompé de date (#853).
- **Réversible** : oui.

### C-v017-4-11 — Réconciliation, lots, propositions : les brouillons s'excluent par les filtres existants ; inventaire clos

- **Contexte** : C-v017-4-2 fait que tout lecteur `status = 'open'` exclut les brouillons. Les autres lecteurs du
  statut doivent être examinés un à un.
- **Retenu** : inventaire **clos** des lecteurs du statut et de l'écriture d'achat (fiche parente, § *Inventaire*),
  chacun avec son comportement attendu devant un brouillon et, pour les chemins de paiement, un test. Choix nommé :
  la proposition de fournisseurs par IBAN (`contacts.rs:401`, `status <> 'cancelled'`) **compte** les brouillons — un
  IBAN saisi par l'utilisateur sur un brouillon est une donnée de la société, la règle « propose, ne crée jamais »
  tient.
- **Réversible** : oui.

### C-v017-4-12 — #853 : échéance et date par défaut — intégré (points 1 et 2), point 3 laissé

- **Contexte** : #853, constat de Guy sur la rc1. Le Project Lead l'a versé à cette story le 2026-10-31 (C-v017-4-20).
- **Retenu** : (1) une échéance **antérieure** à la date de facture est refusée au serveur — conversion commune
  `to_new_supplier_invoice` (`routes/supplier_invoices.rs`), donc création, complétion et `PUT` —,
  `400 SUPPLIER_INVOICE_DUE_BEFORE_INVOICE_DATE`, `details { invoiceDate, dueDate }`, et signalée au formulaire sous
  le champ avant l'envoi ; une échéance **égale** est admise. Les factures existantes ne sont pas touchées (le contrôle
  ne joue qu'à l'enregistrement). (2) Avec un document (`document !== null`), le formulaire **ne pré-remplit pas** la
  date du jour (`SupplierInvoiceForm.svelte:89-92`) : le champ est vide et obligatoire, et dit pourquoi ; aucun QR
  suisse ne porte de date de facture, la date de dépôt n'est pas celle de la facture. (3) L'avis « date de facture
  postérieure au dépôt de plus de quelques jours » **n'est pas** fait : heuristique sans règle, demandée en issue.
- **Réversible** : oui.

### C-v017-4-13 — Un contrôle de clôture neuf : K8 `DRAFT_SUPPLIER_INVOICES`

- **Contexte** : jusqu'ici une facture fournisseur était toujours au journal. Un brouillon daté dans l'exercice est une
  **charge non comptabilisée** : la clôture doit le dire, comme K4 le dit des factures client en brouillon
  (`closing_checks.rs:12`, `:474-481`). Le contrat dit « `checks` porte toujours sept contrôles »
  (`docs/api-external.md:255-256`).
- **Retenu** : K8 `DRAFT_SUPPLIER_INVOICES`, sévérité `info`, mêmes forme et bornes que K4 (compte de tous, liste de 20
  au plus `{id, label, date}`, par date puis `id` ; `label` = nom du fournisseur, et son numéro s'il en a un), placé
  **après** K4 dans l'ordre rendu ; « sept » devient « huit » (contrat changé, annoncé). Rien ne bloque.
- **Écartées** : (b) étendre K4 — deux types de pièces sous un code qui dit « factures client » ; (c) issue séparée —
  la story crée le risque, elle le signale.
- **Réversible** : oui.

### C-v017-4-14 — Les factures annulées : masquées par défaut dans la liste ; la suppression réelle est posée à Guy (Q)

- **Contexte** : Guy veut « pouvoir supprimer des factures annulées sur la version précédente ». Une facture annulée
  porte son écriture d'achat **et** sa contre-passation (`cancel_in_tx`, `supplier_invoices.rs:1335`), lettrées
  ensemble ; payée, un règlement **détaché** reste au grand livre, gelé par la trace d'audit de l'annulation
  (`modification_guard`, `journal_entries.rs:1066-1100`). `delete_in_tx` refuse de supprimer une écriture
  contre-passée — « la supprimer effacerait la correction, ce que l'art. 958f CO interdit précisément »
  (`journal_entries.rs`, étape 3-bis) —, et la garde de modification refuse de supprimer une contre-passation.
- **Retenu** : **aucune suppression d'écriture n'est prescrite** (consigne du mandat). Cette story livre l'option non
  destructive : la liste des factures fournisseur **masque par défaut** les factures annulées (case « Afficher les
  factures annulées »), par un filtre `status` optionnel de `GET /supplier-invoices` (défaut inchangé : tout). Les
  options et la recommandation sont posées à Guy (Q1).
- **Écartées, sous réserve de Q1** : (A) suppression réelle de la facture et de ses deux écritures — contourne trois
  gardes, creuse deux trous de numérotation, rouvre le gel d'un règlement détaché ; (C) suppression de la seule
  facture, écritures gardées — perd le lien pièce ↔ écritures et dégèle le règlement détaché.
- **Réversible** : oui.

### C-v017-4-15 — L'écran : brouillon visible partout, une page d'édition, pas de « Enregistrer et valider »

- **Retenu** : badge « Brouillon » dans la liste et sur la fiche ; sur la fiche d'un brouillon : *Modifier* (page
  `/supplier-invoices/{id}/edit`, même composant `SupplierInvoiceForm`, pré-rempli), *Valider* (avec confirmation qui
  dit que l'écriture d'achat sera passée), *Supprimer le brouillon* (Administrateur, confirmation) ; ni « Payer » ni
  « Annuler la facture ». Sur une facture ouverte : *Dévalider* (Administrateur et Comptable, confirmation qui dit que
  l'écriture d'achat sera supprimée et que la numérotation gardera un trou), en plus de « Payer » et « Annuler ». Le
  formulaire de saisie dit « Enregistrer le brouillon ». Patron : la fiche de facture client
  (`invoices/[id]/+page.svelte:341-370`, `:1089-1093`, `:1453`).
- **Écartées** : (b) un second bouton « Enregistrer et valider » — commode, mais s'écarte du patron client et coupe un
  échec de validation de la saisie (question Q3).
- **Réversible** : oui.

### C-v017-4-16 — Pas de motif de dévalidation calculé d'avance ; le refus s'affiche au clic

- **Contexte** : la fiche fournisseur calcule d'avance les motifs d'annulation (`cancellable`, `settlementCancellable`) ;
  la fiche client n'en calcule pas pour la dévalidation (`invoices/[id]/+page.svelte:341-370`).
- **Retenu** : parité avec le client : le bouton *Dévalider* s'affiche pour une facture ouverte et un rôle d'écriture ;
  un refus s'affiche sous lui, avec un message par code (fonction pure testée, patron `supplierInvoiceCancelMessage`).
  Aucun champ neuf dans la réponse.
- **Écartées** : (b) champ `unvalidatable`/`unvalidateBlockedBy` — exigerait un lecteur des motifs de `delete_in_tx`
  sans la garde de propriété, jumeau de `modification_blocker` (`journal_entries.rs:1215`) : du code de plus pour un
  confort, et un risque de divergence (« aucun jumeau »).
- **Réversible** : oui.

### C-v017-4-17 — Le dépôt : quatre fonctions neuves, la création « validée » composée pour les seeds et les tests

- **Contexte** : 45 appels de `supplier_invoices::create` dans `crates/kesh-db/tests/supplier_invoices_repository.rs`,
  et d'autres dans les tests, le seed (`kesh-seed/src/pieges.rs:585`) et la complétion — tous attendent une facture au
  journal.
- **Retenu** : `create_draft_in_tx` (+ enveloppe), `update_draft`, `delete_draft`, `validate_in_tx` (+ enveloppe),
  `unvalidate` ; l'actuel `create`/`create_in_tx` est **renommé** `create_validated`/`create_validated_in_tx` et devient
  la **composition** `create_draft_in_tx` puis `validate_in_tx` (un seul corps pour chaque moitié, aucun jumeau), gardée
  pour les seeds et les tests ; aucune route ne l'appelle. Le renommage rend chaque appelant visible au compilateur.
- **Écartées** : (b) garder le nom `create` avec une sémantique « validée » — un futur appelant le prendrait pour la
  création de l'écran ; (c) réécrire les 45 tests en deux appels — bruit sans gain.
- **Réversible** : oui.

### C-v017-4-18 — La candidate suivante : `v0.17.0-rc2` (Q)

- **Retenu** : la story entre dans la release 0.17.0 par une candidate `v0.17.0-rc2` ; la version Cargo reste `0.17.0`.
  À confirmer par Guy (Q2), avec la limite L1 (C-v017-4-3).
- **Réversible** : oui.

### C-v017-4-19 — MCP : les brouillons sont lus comme les autres factures

- **Retenu** : `kesh_factures_fournisseur` et `kesh_facture_fournisseur` rendent aussi les brouillons ; `statut` gagne
  `draft` (`mcp/outils/commun.rs:147-151`), `ecritureAchatId` vaut `null` pour un brouillon
  (`mcp/outils/factures_fournisseur.rs:233`). Aucun outil n'écrit. Le manuel d'administration le dit là où il décrit
  ces sorties.
- **Écartées** : (b) masquer les brouillons à l'agent — l'agent qui aide à la saisie doit les voir.
- **Réversible** : oui.

### C-v017-4-20 — Ajout du Project Lead du 2026-10-31 : quatre défauts du même formulaire (#853, #830, #835, #831)

- **Contexte** : message de l'orchestrateur, décision du Project Lead du 2026-10-31 : « Quatre défauts du **même
  formulaire** de facture fournisseur entrent dans cette story, pour éviter une seconde story parallèle sur les mêmes
  fichiers. » Prémisses établies au code (`8e126cfd`) :
  - **#853** — le formulaire pré-remplit la date du jour (`SupplierInvoiceForm.svelte:89`, `:92`) ; aucun contrôle de
    l'échéance contre la date, ni au formulaire ni au serveur (`to_new_supplier_invoice`, `routes/supplier_invoices.rs`) :
    **confirmée** ;
  - **#830** — `isValidAmount` admet la virgule (`journal-entries/balance.ts`, `amountRegExp`, `[.,]`) ; l'envoi passe la
    chaîne brute (`SupplierInvoiceForm.svelte:233-234`, `quantity: l.quantity || '1'`, `unitPrice: l.unitPrice`) ;
    `sumTtc` fait `new Big(l.quantity || '0')` sans normaliser (`supplier-invoice-form.ts:35`), lève sur « 12,50 » et
    ignore la ligne (`:36-37`) : **confirmée**. *Correction de prémisse* : la saisie manuelle des relevés (v016-2b) ne
    normalise pas une chaîne à l'envoi, elle **parse** par `parseAmount` (`balance.ts:68-71`, `replace(',', '.')`) puis
    envoie la valeur lue (`bank-import/manual-statement.ts:115-119`) ; c'est ce patron — parser une fois, envoyer le
    nombre — qui est retenu. Constat voisin, même fonction : `sumTtc` lit une quantité vide `0`, l'envoi `1` (`:35` contre
    `:233`) — corrigé avec, mécanique ;
  - **#835** — `formatSwissAmount(sumTtc(fLines))` (`SupplierInvoiceForm.svelte:591`) arrondit à deux décimales :
    **confirmée** ;
  - **#831** — la liste affiche `formatSupplierInvoiceTotal(inv.totalAmount)` (`supplier-invoices/+page.svelte:93`),
    deux décimales (`supplier-invoice-helpers.ts:10-17`) ; aucun banc de la page de liste : **confirmée**.
- **Retenu** : les quatre dans la sous-fiche **écran** (4b), sauf le refus serveur de #853, qui est dans la conversion
  commune et va à la 4a. CHANGELOG *Corrigé*, `refs` ; fermeture portée par la PR de release.
- **Réversible** : oui.

---

*Les choix C-v017-4-21 à 25 consignent les décisions de l'orchestrateur sur les questions Q1 à Q5, prises le 2026-10-31
dans le mandat autonome du Project Lead, avant la validation P1. Les entrées précédentes restent telles qu'écrites ;
celles que ces décisions modifient sont nommées ci-dessous.*

### C-v017-4-21 — Q1 : les factures annulées sont masquées, non supprimées

- **Contexte** : C-v017-4-14 et la question Q1 (options A, B, C).
- **Retenu** : option **(B)** — la liste masque par défaut les factures annulées, sans toucher au journal ; livrée par
  cette story (AC-15, AC-21). L'option **(A)**, la suppression réelle de la facture et de ses deux écritures, **n'est pas
  dans la 0.17.0** : elle sera soumise au Project Lead à part. Limite L6, écrite au manuel utilisateur.
- **Écartées** : (A) dans cette release ; (C).
- **Réversible** : oui.

### C-v017-4-22 — Q2 : candidate `v0.17.0-rc2`

- **Retenu** : oui ; la limite L1 (rc2 → rc1 seulement par la sauvegarde d'avant la mise à jour) s'écrit dans les notes
  de la candidate. Confirme C-v017-4-18.
- **Réversible** : oui.

### C-v017-4-23 — Q3 : pas de bouton « Enregistrer et valider »

- **Retenu** : confirme C-v017-4-15 : on enregistre le brouillon, puis on valide depuis sa fiche.
- **Réversible** : oui.

### C-v017-4-24 — Q4 : l'état validé s'affiche « Validée » (révise le libellé de C-v017-4-2)

- **Contexte** : C-v017-4-2 gardait le libellé « Ouverte ». Le vocabulaire du Project Lead est « création → validation
  → paiement ».
- **Retenu** : la valeur stockée reste `open` (le reste de C-v017-4-2 tient : aucune ligne réécrite, filtres et contrat
  inchangés) ; seuls le libellé, les textes et le filtre de liste changent. `supplier-invoices-status-open` prend la
  valeur de `invoice-status-validated` de chaque catalogue (« Validée », « Validiert », « Convalidata », « Validated ») ;
  aucun texte de l'écran, des catalogues, des manuels, de `api-external.md` ni du texte neuf du CHANGELOG ne nomme plus
  cet état « ouverte » — inventaire clos aux AC-21 et AC-29 ; « ouvert » au sens du poste ouvert d'un compte reste.
  Mutation M31.
- **Écartées** : garder « Ouverte » (première rédaction).
- **Réversible** : oui (clés i18n et textes).

### C-v017-4-25 — Q5 : #853 se ferme avec la release

- **Retenu** : `closes #853` à la PR de release ; son point 3 (avis sur une date de facture bien postérieure au dépôt)
  part dans une issue neuve, ouverte par l'orchestrateur. Limite L5.
- **Réversible** : oui.

---

*Les choix C-v017-4-26 à 33 consignent la remédiation de la validation P1 (2026-10-31 ; rapports
`kesh-gate-logs/v0174-validate-p1-{R,F}.md`) et les décisions de l'orchestrateur qui l'accompagnent. Les entrées
précédentes restent telles qu'écrites ; celles que ces choix modifient sont nommées.*

### C-v017-4-26 — K8 rendu en dernier (R1 ; précise C-v017-4-13)

- **Contexte** : C-v017-4-13 plaçait K8 après K4 ; deux tests indexent `checks` par position
  (`closing_checks_e2e.rs:258`, `kesh-db/tests/closing_checks.rs:322`), et le contrat dit « dans cet ordre ».
- **Retenu** (décision de l'orchestrateur) : K8 s'ajoute **en dernier** ; K1 à K7 inchangés ; ses requêtes à `ALL_SQL`
  (`closing_checks.rs:522`), garde T-a2 verte ; total 7 → 8.
- **Écartée** : après K4 — décale K5 à K7 pour un intégrateur qui lit par position.
- **Réversible** : oui avant publication.

### C-v017-4-27 — Une fonction pure unique pour le total d'une facture fournisseur (R2)

- **Contexte** : `total_ttc` n'existe que comme sortie de `generate_purchase_journal_lines`
  (`supplier_invoices.rs:120-178`), qui exige le compte de TVA récupérable dès que la TVA est positive ; un brouillon ne
  juge pas les réglages.
- **Retenu** (décision) : `kesh_core::accounting::purchase_invoice_totals`, pure, appelée par l'enregistrement du
  brouillon **et** par la génération de l'écriture ; test d'un brouillon à TVA sans compte de TVA récupérable : il
  s'enregistre, sa validation exige les réglages.
- **Écartées** : un second calcul au dépôt (divergerait un jour de la ligne `C 2000`) ; exiger les réglages au brouillon.
- **Réversible** : oui.

### C-v017-4-28 — Régime de verrous et de rejeu par route (R3)

- **Retenu** (décision) : tableau du § *Verrous et rejeu* de la fiche parente. Enregistrement (création, `PUT`) : lecture
  sans verrou des références, jugement définitif sous verrous à la validation ; création reclassée `Traced,
  SansEcritureAuJournal`, son enveloppe de rejeu retirée ; `rejeu_interblocage_e2e.rs:865` porté sur la validation.
  `DELETE` : **document puis facture**, l'ordre de `attach_in_tx`, sans enveloppe — l'ordre unique supprime le cycle que
  le doc-comment de `detach_in_tx` interdit. Validation et dévalidation rejouées.
- **Écartées** : `DELETE` facture puis document sous enveloppe de rejeu — un cycle connu qu'on accepte alors qu'un ordre
  l'évite ; garder les `FOR UPDATE` des comptes à l'enregistrement — verrous exclusifs sans écriture qui en dépende.
- **Réversible** : oui.

### C-v017-4-29 — Les règles de la complétion ne valent au `PUT` que pour un document né de la complétion (R5 ; révise C-v017-4-5)

- **Contexte** : C-v017-4-5 appliquait ces règles « quel que soit le chemin qui a joint » le document ; or
  `attach_in_tx` joint un document sans aucune d'elles, et le `PUT` d'une facture de Guy à document joint après coup
  aurait remplacé ses coordonnées de paiement en silence.
- **Retenu** (décision) : une colonne `imported_supplier_invoices.link_origin` (`completion`, `attachment`, `NULL` =
  antérieur), posée par la complétion et le rattachement, remise à `NULL` par le détachement, ajoutée par la migration de
  la story (nullable, sans backfill) ; le `PUT` applique les règles de la complétion **seulement** si `link_origin =
  'completion'`. Test du cas de Guy : facture validée, document joint, dévalidée, éditée, revalidée. Limite L7 : un lien
  antérieur n'impose rien.
- **Écartées** : déduire l'origine du journal d'audit (fusionné à la restauration, fragile) ; comparer les coordonnées de
  la facture à celles du QR (heuristique) ; appliquer les règles à tout document (changement silencieux de destination
  de paiement).
- **Réversible** : la colonne non une fois publiée ; son usage, oui.

### C-v017-4-30 — `sourceDocument` porte ce que l'édition d'un brouillon né d'un QR doit afficher (F5)

- **Retenu** (décision : un champ plutôt qu'une route) : `sourceDocument` gagne `importedSupplierInvoiceId`,
  `linkOrigin` et `qr` (les champs du mode QR du formulaire, `null` sans QR) ; la 4b les consomme. Un champ : la fiche
  et la page d'édition lisent déjà la facture dans un seul instantané ; une route de plus serait une seconde lecture, et
  la liste des documents n'est pas bornée.
- **Écartées** : une route `GET …/source-document/details` ; `listImported('completed').find(…)` à l'écran.
- **Réversible** : oui (ajout de champs).

### C-v017-4-31 — Dérogation : la 4a ne se publie qu'avec la 4b (R6)

- **Retenu** (décision) : même branche, publication commune ; la 4a joue gate backend, Vitest et E2E ciblés de ses
  routes ; l'E2E complet se joue au dernier commit de code de la 4b et couvre les deux.
- **Écartées** : déplacer l'adaptation des specs dans la 4a — elle n'empêcherait pas l'écran de créer des brouillons
  qu'il ne sait pas valider.
- **Réversible** : oui (organisation).

### C-v017-4-32 — « Validée » : inventaire par la valeur sur tout le dépôt ; « saisissez, puis validez » (F1, F2, F6, R9)

- **Retenu** (décision) : le tri des « ouverte » (et `offen`, `aperta`, `open` en prose) se fait **par la valeur** sur
  `crates/`, `frontend/` (sources, bancs, E2E), `docs/` (manuels, `api-external.md`, glossaire), les quatre catalogues,
  `README.md`, `website/` ; le parcours de Guy s'écrit « saisissez, puis validez la facture » ; la règle vaut pour les
  textes de la 4a (`api-external.md`, manuel d'administration, CHANGELOG) ; le contrôle du PDF compte toutes les formes
  relevées par F6, apostrophe typographique comprise (F7).
- **Réversible** : oui.

### C-v017-4-33 — La limite L1 va aux notes de la candidate seulement (F17)

- **Retenu** : L1 (rc2 → rc1) s'écrit aux notes de la candidate `v0.17.0-rc2` ; ni le CHANGELOG `[0.17.0]` ni le manuel
  d'administration, qui décrivent la version publiée, ne la portent. Précise C-v017-4-22.
- **Réversible** : oui.

---

*Les choix C-v017-4-34 à 36 consignent la remédiation de la validation P2 (2026-10-31 ; rapports
`kesh-gate-logs/v0174-validate-p2-{R,F}.md`) et les arbitrages de l'orchestrateur. Les entrées précédentes restent telles
qu'écrites.*

*Annotation datée du 2026-10-31 (validation P2, R1 = F1) à **C-v017-4-28** : l'ordre retenu pour `DELETE` — « document
puis facture, l'ordre de `attach_in_tx` » — était **faux** : le document que verrouille `attach_in_tx` en premier est le
document **neuf**, et la ressource partagée avec un `DELETE` est l'entrée de `uq_imported_supplier_invoice`, qu'il ne
touche qu'**après** la facture ; c'est le cycle que le doc-comment de `detach_in_tx` interdit
(`imported_supplier_invoices.rs:311-314`). Remplacé par C-v017-4-34.*

### C-v017-4-34 — `DELETE` d'un brouillon : la facture d'abord, puis `detach_in_tx` (révise C-v017-4-28)

- **Contexte** : R1 (HIGH) et F1 convergent ; doc-comment de `detach_in_tx`, `imported_supplier_invoices.rs:311-314` :
  « un détachement qui demanderait la facture **après** le document formerait un cycle avec [`attach_in_tx`] (document
  neuf, puis facture, puis entrée d'unicité) ».
- **Retenu** (arbitrage de l'orchestrateur) : facture `FOR UPDATE`, statut, articles de lot, **puis** `detach_in_tx`,
  puis `DELETE` — l'ordre « facture → entrée d'unicité » que le code sait sûr ; sans enveloppe. M20-quater inversée
  (mutant : le document avant la facture).
- **Écartée** : l'ordre inverse sous enveloppe de rejeu — accepter un cycle qu'un ordre évite.
- **Réversible** : oui.

### C-v017-4-35 — La contrainte de statut sous un nom neuf, posée avant le retrait de l'ancienne

- **Contexte** : R5 ; précédent du dépôt, `20261001000004_invoice_settlements_write_off.sql:17-19` : « un CHECK ne se
  modifie pas en place → DROP CONSTRAINT + ADD CONSTRAINT, chacun dans son instruction » — dans cet ordre, un échec entre
  les deux laisse la colonne sans garde.
- **Retenu** (arbitrage) : `chk_supplier_invoices_status_v2` (et les deux `CHECK` neufs) par une première instruction,
  le retrait de `chk_supplier_invoices_status` par une seconde. Un échec à mi-chemin sur la base de Guy laisse `status`
  gardé par les deux contraintes, jamais par aucune.
- **Écartées** : `DROP` puis `ADD` du même nom (le précédent tel quel) ; les deux dans une seule instruction (conflit de
  nom non éprouvé sur MariaDB 10.11).
- **Réversible** : non une fois publiée ; le nom `_v2` reste.

### C-v017-4-36 — Les preuves de la P2 : sauvegarde sans la colonne, prix vide refusé au formulaire

- **Retenu** (arbitrages) : (1) la sauvegarde 0.16.x forgée de l'AC-3 (a) **ne porte pas** `link_origin` — c'est la
  preuve que demande la contrainte « données à garder » (R2) ; (2) le prix vide est refusé par le formulaire
  (`isValidAmount(l.unitPrice, { allowEmpty: false })` au site de `linesInvalid`), mécanique, avec un banc qui tue M33
  — et non seulement par le serveur (F2), motif : l'erreur se dit à sa place, sans aller-retour, et la lecture unique de
  #830 ne peut plus lire `0` ; (3) la complétion est reclassée `SansEcritureAuJournal` et reste rejouée (R6) ; (4) M12b,
  mutant équivalent, est retirée (R4).
- **Réversible** : oui.

---

*Les choix C-v017-4-37 à 40 consignent la remédiation de la validation P3 (2026-10-31 ; rapports
`kesh-gate-logs/v0174-validate-p3-{R,F}.md`) et les arbitrages de l'orchestrateur. Les entrées précédentes restent telles
qu'écrites.*

*Annotation datée du 2026-10-31 (validation P3, R2) au **Retenu** de **C-v017-4-28** : « création reclassée `Traced,
SansEcritureAuJournal`, **son enveloppe de rejeu retirée** » était faux sur le second point — les verrous partagés de
l'insertion (clés étrangères des lignes sur les comptes, `company_id` contre la sentinelle exclusive) forment un cycle
avec une validation aux comptes croisés. L'enveloppe est **gardée** (C-v017-4-37) ; le classement
`SansEcritureAuJournal` reste juste.*

### C-v017-4-37 — Le rejeu se garde sur la création, le `PUT` et le `DELETE` (R2 ; révise C-v017-4-28 et C-v017-4-34)

- **Retenu** (arbitrage de l'orchestrateur : « ne crée pas de limite résiduelle sur une base en production ») :
  `retry_on_deadlock` sur la création (gardé), le `PUT` et le `DELETE` (ajoutés) ; la transaction annulée n'écrit rien,
  le rejeu est sûr. Le `DELETE` garde son ordre (facture puis document), qui évite le cycle avec le rattachement ;
  l'enveloppe couvre les autres. Les trois restent `SansEcritureAuJournal` au registre, rejouées « quand même » (point
  (vi) recompté).
- **Écartée** : la limite écrite d'un `500` transitoire sur un geste de saisie.
- **Réversible** : oui.

### C-v017-4-38 — Tests pilotés des courses ; décodage tolérant des lots (R1 = F3, R3 = F4)

- **Retenu** : (1) les tests de M20-quater (`DELETE`) et de M20 (validation) sont **pilotés** : attache rejouée à la main
  jusqu'à la sentinelle, ou validation non commitée, puis attente du blocage de l'autre geste par
  `attendre_une_requete_en_cours` (`PROCESSLIST`), puis reprise ; le mutant rougit à coup sûr. Écarté : le protocole de F
  (intervalle tenu, sonde `INNODB_TRX`, `FOR UPDATE NOWAIT`) — `INNODB_TRX` exige le privilège `PROCESS`, interdit par
  `docs/agents-orchestration.md` § 6.1, et il ne prouve que l'ordre, non l'absence d'interblocage. (2)
  `payment_batches.rs:378-385` : dernier champ du tuple en `Option<i64>`, l'écriture extraite après le contrôle de
  statut ; un brouillon sort en `failed[]` `SUPPLIER_INVOICE_NOT_OPEN` (§ *Pattern batch*), testé.
- **Réversible** : oui.

### C-v017-4-39 — Le repli du `MODIFY COLUMN` sous un nom neuf de clé étrangère (R5)

- **Retenu** : l'essai joue trois formes — `MODIFY` seul, repli au même nom, repli au nom neuf
  `fk_supplier_invoices_purchase_je_v2` (une instruction : `DROP FOREIGN KEY`, `MODIFY`, `ADD CONSTRAINT` au nom neuf) —
  et la migration retient la première qui passe ; chaque résultat au Dev Agent Record. Même logique que C-v017-4-35 pour
  le `CHECK` : jamais un état sans garde.
- **Écartée** : `DROP`, `MODIFY`, `ADD` en trois instructions — un échec entre elles laisserait la colonne sans clé.
- **Réversible** : non une fois publiée.

### C-v017-4-40 — Quatre conduites écrites (F2, R7, R9)

- **Retenu** : (1) `SUPPLIER_INVOICE_DRAFT` entre à l'union `SupplierInvoiceCancelCode` **avec** son `case` et son
  texte — le serveur la rend sur le `GET` d'un brouillon, et la fonction est exhaustive (`never`,
  `invoice-cancel.ts:74-77`) ; (2) `link_origin` n'est pas exposé à l'outil MCP des pièces (`PiecePublique`) — l'agent n'en
  a pas l'usage ; (3) un brouillon à historique de lot n'est pas supprimable (limite L8 ; son chemin : valider puis
  corriger, ou valider puis annuler) ; (4) la précédence de la dévalidation met le lot `generated` avant l'exercice clos,
  à l'inverse de l'annulation fournisseur : elle suit sa fonction sœur `invoices::unvalidate`, dont les empêchements
  propres à la pièce précèdent ceux de `delete_in_tx`.
- **Réversible** : oui.
- *Annotation du 2026-10-31 (étape 0 du développement de la 4a, validation P5, P5)* : au point (3), « valider puis
  annuler » désigne l'annulation de la **facture** une fois validée (motif : elle n'est pas due), non celle du lot ; et
  l'historique de lot qui rend un brouillon non supprimable naît **quel que soit le sort du lot** — annulé (ses articles
  restent, `payment_batches.rs:697`) ou confirmé puis règlement annulé —, comme la fiche le dit à la limite L8 depuis la
  validation P4 (F7).

---

*Les choix C-v017-4-41 et 42 consignent la remédiation de la validation P4 (2026-10-31 ; rapports
`kesh-gate-logs/v0174-validate-p4-{R,F}.md`). Les entrées précédentes restent telles qu'écrites.*

### C-v017-4-41 — Le test piloté du `DELETE` porte sur le dépôt, sans l'enveloppe de rejeu (F2)

- **Contexte** : C-v017-4-38 (test piloté) et C-v017-4-37 (rejeu du `DELETE`) sont justes chacun ; ensemble, au niveau
  de la route, l'enveloppe rejouerait le `DELETE` sacrifié par InnoDB et le mutant M20-quater survivrait.
- **Retenu** (arbitrage) : le test qui tue M20-quater appelle `supplier_invoices::delete_draft` (dépôt), sans
  `retry_on_deadlock`, et asserte l'absence de `DbError` d'interblocage des deux côtés ; un second test, de route,
  constate que l'enveloppe rend `204`/`200` et jamais `500` — il ne tue pas le mutant.
- **Réversible** : oui.

### C-v017-4-42 — Un seul texte pour le motif « brouillon », posé par la 4a ; l'édition passe au formulaire ce qui convient (F1, F3)

- **Retenu** : (1) la clé `supplier-invoices-cancel-blocked-draft` et son texte (« Cette facture est un brouillon : elle
  ne s'annule pas, car rien n'a été passé au journal. ») sont posés à l'AC-13 (4a), parce que le serveur et l'écran
  partagent un texte par motif (`errors.rs:4565-4566`) ; le texte ne prescrit aucune suppression, un Comptable ou une clé
  `read-write` ne pouvant pas supprimer ; l'AC-25 réutilise la clé. (2) À l'édition, le formulaire reçoit l'objet `qr`
  seulement si `linkOrigin === 'completion'` et `qr !== null`, sinon `null` ; le scan navigateur est caché dès que la
  facture a un document (prop `hasDocument`) ; la prop `document` est un `Pick` de douze champs.
- **Écartées** : la clé posée par la 4b (deux propriétaires) ; un texte qui invite à supprimer.
- **Réversible** : oui.

---

*Les choix C-v017-4-43 et suivants consignent le développement de la 4a (2026-10-31, Opus 5.5, autonomie ;
worktree `kesh-0175`). Les entrées précédentes restent telles qu'écrites.*

### C-v017-4-43 — La migration retient le `MODIFY COLUMN` seul (essais sur MariaDB 10.11)

- **Contexte** : AC-1 (1), C-v017-4-39 — trois formes à essayer dans l'ordre sur une base jetable à l'état 87.
- **Mesuré** (bases `kesh_v0174a_mig1` à `3`, MariaDB 10.11 du conteneur partagé) : (1) `MODIFY COLUMN … BIGINT NULL`
  seul → **accepté**, la clé `fk_supplier_invoices_purchase_je` gardée sous son nom ; (2) repli au même nom (`DROP
  FOREIGN KEY`, `MODIFY`, `ADD CONSTRAINT` du même nom, une instruction) → **refusé**, `ERROR 1005 (HY000) … errno:
  121 "Duplicate key on write or update"`, la table intacte (`NOT NULL`, clé présente) ; (3) repli au nom neuf →
  accepté, et l'index implicite **prend** le nom `fk_supplier_invoices_purchase_je_v2` (contrairement à ce que la fiche
  supposait : « l'index garde son nom »).
- **Retenu** : la forme (1), la première qui passe. Les `CHECK` suivent l'ordre de C-v017-4-35 (le neuf posé avant le
  retrait de l'ancien), essayés de même sur `kesh_v0174a_mig1` : acceptés.
- **Réversible** : non une fois publiée.

### C-v017-4-44 — Le `PUT` : la route tient la transaction ; le dépôt expose `lock_draft_in_tx` et `update_draft_in_tx`

- **Contexte** : P4 de la validation P5 ; les règles « document » rendent des `AppError`.
- **Retenu** : `lock_draft_in_tx` (facture `FOR UPDATE`, statut puis version) est appelé par la route **avant** la
  lecture ordinaire du document et les règles ; `update_draft_in_tx` le rappelle (second `FOR UPDATE`, aucun verrou
  neuf) puis remplace. `update_draft` (pool) les compose pour les tests. Enveloppe `retry_app_on_deadlock("supplier_invoices::update")`.
- **Écartée** : un rappel passé au dépôt (fermeture `AppError` dans `kesh-db`, qui ne connaît pas `AppError`).
- **Réversible** : oui.

### C-v017-4-45 — Une variante `DbError::SupplierInvoiceNotUnvalidatable`, `details { documentId, documentNumber }` pour ses trois motifs

- **Contexte** : AC-11 ; la fiche dit « forme existante » pour `SUPPLIER_INVOICE_IN_PAYMENT_BATCH`.
- **Retenu** : un enum `SupplierUnvalidationBlocker` (`Paid`, `InPaymentBatch`, `MatchedBankTransaction`) et une
  réponse de la forme de `InvoiceNotUnvalidatable` (code du motif, message suffixé du numéro, `details` avec le lot ou
  la transaction) pour les trois — le code `SUPPLIER_INVOICE_IN_PAYMENT_BATCH` est le même que celui de l'annulation ;
  sa réponse porte en plus `details`, ajout sans retrait.
- **Écartée** : réemployer `UnvalidationBlocker` (facture client) — ses `match` exhaustifs auraient reçu des motifs
  qui ne la concernent pas.
- **Réversible** : oui.

### C-v017-4-46 — K8 a la forme de K4 ; son libellé est converti en `utf8mb4`

- **Retenu** : `CheckDetails::DraftSupplierInvoices(DraftDetails)` — `{ "invoices": [{ id, label, date }] }`, la forme
  de K4 (l'écran réutilise son rendu). Le libellé `CASE … CONCAT(numéro, ' — ', fournisseur)` est enveloppé d'un
  `CAST(… AS CHAR CHARACTER SET utf8mb4)` : sans lui, MariaDB le rendait en `VARBINARY` (décodage refusé, mesuré au
  premier gate).
- **Réversible** : oui.

### C-v017-4-47 — Les audits de la suppression d'un brouillon s'écrivent au dépôt

- **Retenu** : `delete_draft_in_tx` écrit `imported_supplier_invoice.detached` (si un document est rendu) puis
  `supplier_invoice.deleted`, par `NewAuditLogEntry::user` — la route est fermée aux clés, l'acteur est toujours un
  utilisateur ; le test piloté appelle le dépôt nu et voit les mêmes traces.
- **Réversible** : oui.

### C-v017-4-48 — La validation refait sous verrou les contrôles de saisie, et réécrit `total_amount`

- **Retenu** : `validate_in_tx` relit les lignes du brouillon et reprend l'ancien corps de la création — fournisseur,
  forme des lignes, comptes de charge `FOR UPDATE` (actif, type, imputable), projet par `validate_taggable_in_tx` —, puis
  `UPDATE … total_amount = <TTC de la génération>` : égal au total du brouillon par construction (même fonction pure),
  réécrit pour que la facture validée et son écriture ne puissent diverger. Un brouillon migré d'une facture
  antérieure à la v017-3b, dont une ligne n'est pas au centime, est refusé à la revalidation (voulu : #687).
- **Réversible** : oui.

### C-v017-4-49 — La fixture du lettrage pose la facture selon le schéma

- **Contexte** : `lettering_documents.rs` (support) sert des tests de rattrapage montés sur un schéma **antérieur** à
  `20261031000001`, où un brouillon ne s'insère pas (`purchase_journal_entry_id NOT NULL`).
- **Retenu** : la fixture lit `IS_NULLABLE` ; sur l'ancien schéma, elle pose l'écriture d'achat et la ligne `open`
  comme la création d'avant la 0.17.0 ; sur le courant, `create_validated`.
- **Réversible** : oui.

### C-v017-4-50 — L'AC-30 de la 4a : un helper de relevé saisi neuf, non `decor()`

- **Retenu** : `createManualDebitStatementViaApi` (`helpers/api-fixtures.ts`) — une ligne de débit, solde de départ
  `0.00`, arrivée `-montant`, équilibré au centime, confirmations posées (relevé identique, lignes, écart de solde).
  `decor()` de `reconciliation-supplier.spec.ts` lit une fixture CAMT d'un montant figé.
- **Réversible** : oui.

### C-v017-4-51 — Le filtre `status` de la liste se lit au service

- **Retenu** : `services::factures_fournisseur::statuts_du_filtre` (absent → tous ; valeurs séparées par des virgules,
  blancs ignorés, doublons fondus ; inconnue ou vide → `400 VALIDATION_ERROR`) ; le dépôt `list` reçoit `&[&str]`
  (vide = tous). L'outil MCP passe une liste vide.
- **Réversible** : oui.

## Développement de la v017-4b (écran) — 2026-10-31

### C-v017-4-52 — Les confirmations du cycle sont des dialogues, et le refus s'affiche sous le bouton

- **Contexte** : AC-22 (« confirmation ») ; la fiche fournisseur confirmait jusqu'ici par `window.confirm`, la fiche
  client par des dialogues ; l'AC-31 demande les captures des trois confirmations.
- **Retenu** : trois `Dialog` (patron de `invoices/[id]/+page.svelte`) — *Valider*, *Dévalider*, *Supprimer le
  brouillon* —, chacun disant ce qui va s'écrire. Un refus **ferme** le dialogue et s'affiche **sous son bouton**
  (`…-validate-error`, `…-unvalidate-error`, `…-delete-error`), jamais dans `errorMsg` ; un refus d'écran périmé
  (`ILLEGAL_STATE_TRANSITION`, `OPTIMISTIC_LOCK_CONFLICT`) relit la facture.
- **Écartée** : `window.confirm` (une confirmation native ne se capture pas, et n'est pas traduite par l'écran).
- **Réversible** : oui.

### C-v017-4-53 — Les messages de la dévalidation : les textes du serveur pour la pièce, des textes propres pour l'écriture

- **Contexte** : AC-25 (`supplierInvoiceUnvalidateMessage(code, label)`).
- **Retenu** : les trois motifs propres à la pièce (`SUPPLIER_INVOICE_PAID`, `…_IN_PAYMENT_BATCH`,
  `MATCHED_BANK_TRANSACTION`) reprennent **les clés du serveur** (`error-supplier-invoice-unvalidate-blocked-*`, espace
  global `error-`) et leur suffixe ` (<numéro>)`, `label` étant `details.documentNumber` ; l'état, la version et les
  refus de `delete_in_tx` (`FISCAL_YEAR_CLOSED`, `LATER_FISCAL_YEAR_CLOSED`, `ENTRY_IS_REVERSED`, `PERIOD_LOCKED`,
  `ENTRY_LETTERED`) ont sept clés `supplier-invoices-unvalidate-blocked-*` qui disent ce qui empêche **la dévalidation**
  et qui le lève — les textes du serveur parlent d'« écriture » —, la réouverture d'exercice avec son ordre et sa borne
  (gardes G8 et bornes de `kesh-i18n`) ; un code hors de la précédence rend le repli `supplier-invoices-unvalidate-error`.
  La fiche passe **tout** refus de la dévalidation par cette fonction.
- **Écartées** : afficher `err.message` (l'AC-25 prescrit la fonction) ; reprendre les clés génériques
  (`error-fiscal-year-closed-generic`…), qui ne disent pas quoi faire pour dévalider.
- **Limite** : les dates d'un `PERIOD_LOCKED` et le nom de l'exercice d'un `LATER_FISCAL_YEAR_CLOSED`, que porte le
  message du serveur, ne sont pas repris.
- **Réversible** : oui.

### C-v017-4-54 — Un seul texte pour l'échéance antérieure, au format suisse à l'écran

- **Retenu** : le refus du formulaire et celui du serveur (`SUPPLIER_INVOICE_DUE_BEFORE_INVOICE_DATE`) s'affichent sous
  l'échéance par la **clé du serveur** (`error-supplier-invoice-due-before-invoice-date`), dates au format suisse
  (`formatSwissDate`) ; le refus du formulaire est vivant (signalé dès la saisie) et bloque l'envoi.
- **Réversible** : oui.

### C-v017-4-55 — La date vide a son message sous le champ ; `aria-required`, non `required`

- **Retenu** : avec un document (`document !== null` ou `hasDocument`), l'aide « Date de la facture, telle qu'imprimée
  sur le document » s'affiche ; une date vide rend `supplier-invoices-err-invoice-date` **sous le champ**
  (`supplier-invoice-date-error`), avant le contrôle de l'échéance et des lignes. L'attribut natif `required` est écarté :
  la validation du navigateur passerait avant le message de Kesh (et bloquerait la soumission implicite sans rien dire).
- **Réversible** : oui.

### C-v017-4-56 — `decimalForServer` garde les décimales saisies ; un montant attendu illisible part tel quel

- **Retenu** : quantité, prix et montant attendu lus une fois par `parseAmount`, envoyés avec un point et le nombre de
  décimales saisi (« 12,50 » → `"12.50"`) ; quantité vide → `"1"` ; un montant attendu hors format part tel quel et le
  serveur le refuse (le formulaire ne le contrôlait pas avant, keshnote du manuel inchangée). `lineNotInCentimes` et
  `sumTtc` lisent par la même fonction (DRY).
- **Réversible** : oui.

### C-v017-4-57 — La page d'édition rend un brouillon lisible même si ses références ont bougé

- **Retenu** : le fournisseur du brouillon est ajouté à la liste par `getContact` s'il n'y est pas ; les comptes de
  charge des lignes restent proposés même devenus inactifs ou non imputables (la validation les jugera) ; le taux de TVA
  lu (`8.10`, `0.00`) est aligné sur la valeur d'une option (`formLineFromResponse`). Le formulaire lit `initial` une
  fois, au montage.
- **Réversible** : oui.

### C-v017-4-58 — Le lien de K8 mène à la liste des factures fournisseurs

- **Contexte** : K4 mène à `/invoices?status=draft&dateFrom…` ; la liste fournisseur n'a pas de filtre par l'URL.
- **Retenu** : chaque brouillon de K8 mène à sa fiche ; le lien du contrôle, à `/supplier-invoices` (les brouillons y
  portent leur badge).
- **Écartée** : un filtre d'URL neuf sur la liste (hors périmètre).
- **Réversible** : oui.

### C-v017-4-59 — « Brouillon enregistré. » à la création comme à la complétion

- **Contexte** : AC-24 (e) nomme la complétion ; la page `new` enregistre les deux par une seule fonction et une seule
  clé (`imported-supplier-invoices-completed`).
- **Retenu** : la même notification, vraie dans les deux cas (la création aussi rend un brouillon). La valeur de la clé
  change aux quatre catalogues ; la ligne « créé(e) » du glossaire, qui la citait comme attestation, est annotée.
- **Réversible** : oui.

### C-v017-4-60 — La date de la facture se pose dans le helper E2E

- **Retenu** : `fillFirstInvoiceLine` (`helpers/inbox.ts`) pose la date du jour si le champ est vide (validation P2,
  F3 : « un seul endroit ») ; les specs qui posent une date gardent la leur.
- **Réversible** : oui.

### C-v017-4-61 — Ce que l'inventaire « ouverte » laisse

- **Retenu** : la règle Q4 vise les textes de l'écran, des manuels et de l'API. Restent : les commentaires et
  doc-comments de **tests et de code Rust** de la 4a (`reconciliation_e2e.rs`, `services_rollout.rs`…, hors périmètre
  écran), la ligne **historique** de la v0.14.0 du README (`:232`), les « factures ouvertes » de l'accueil (factures
  **client**), le poste ouvert (« la dette redevient ouverte »), l'exercice ouvert.
- **Réversible** : oui.

### C-v017-4-62 — La capture « refus d'annulation d'un brouillon » n'existe pas à l'écran

- **Contexte** : le mandat de développement demande la capture du « refus d'annulation d'un brouillon » ; l'AC-22
  prescrit l'**absence** du bloc *Annuler la facture* sur un brouillon (M21-bis), et l'AC-25 dit que le motif
  `SUPPLIER_INVOICE_DRAFT` n'est pas affiché par la fiche.
- **Retenu** : la fiche d'un brouillon (sans bloc d'annulation) et le **refus de dévalidation** (AC-31) sont capturés ;
  écart déclaré au rapport.
- **Réversible** : oui.

## Remédiation de la revue de code P1 de la v017-4b — 2026-10-31 (WIP) et 2026-11-03

*Les arbitrages du Project Lead (mandat `v0174b-remediation-p1.md`) s'imposent : lot affiché, libellé vrai du refus des
lots, refus visibles hors du bloc brouillon, décor E2E avec une annulée, captures K8 refaites, tous les LOW traités ou
tracés. Les choix ci-dessous sont ce qui restait à trancher.*

### C-v017-4-63 — Le lot confirmé se dit dans le bloc de la facture validée, à tous les rôles, et dans la confirmation

- **Contexte** : E-2 = A-1 ; arbitrage 1. `lastConfirmedBatch` est un fait historique (le lot confirmé le plus récent
  qui contient la facture, `payment_batches::last_confirmed_batch_for_invoice`). Sur une facture `open`, sa présence
  implique un règlement par ce lot puis annulé (un lot confirmé paie toutes ses factures).
- **Retenu** : un avertissement au-dessus de *Dévalider*, visible de **tous** les rôles (le bloc de la facture validée
  n'est plus réservé à Administrateur et Comptable ; seul le bouton l'est), repris dans le dialogue de confirmation ;
  date au format suisse ; texte « … puis son règlement a été annulé. Si la banque a exécuté cet ordre, vérifiez le
  relevé avant de changer son montant. »
- **Écartée** : refuser la dévalidation (option (c) de C-v017-4-8, déjà écartée) ; l'afficher à la page d'édition seule
  (trop tard : la dévalidation est faite).
- **Réversible** : oui.
- ⚠️ **Annotation du 2026-11-03, revue de code P2 (B2-1 = A2-1, B2-3, B2-5)** : le **contexte était faux**. La
  présence de `lastConfirmedBatch` sur une facture `open` n'établit ni que ce lot a produit le dernier règlement annulé
  (lot confirmé → annulation → règlement direct → annulation), ni que le montant n'a pas déjà changé (dévalidation,
  correction, revalidation) — le dépôt le dit (`payment_batches.rs`, « un fait HISTORIQUE, jamais l'origine du
  règlement courant »). Le texte retenu ci-dessus est remplacé par le fait historique, calqué sur
  `supplier-invoices-settlement-cancel-batch-warning` : « Cette facture figure dans le lot de paiement n° X, confirmé le
  D. » (clé `-undated` sans date si `confirmedAt` est nul), visible de **tous** les rôles ; la consigne « Si la banque a
  exécuté cet ordre, la facture a peut-être déjà été payée : vérifiez le relevé avant de changer son montant. » (clé
  `-action`) n'est dite qu'aux rôles qui dévalident, et dans la confirmation. Le bloc n'existe plus, pour la
  Consultation, sans lot à nommer (A2-4 = E2-3). de-CH « Zahlungsauftrag », comme la clé voisine (A2-5 = B2-7).

### C-v017-4-64 — Les refus du cycle se rendent hors des blocs de statut, et s'effacent au début de chaque geste

- **Contexte** : E-1 = A-6, B-2, B-3 = E-3.
- **Retenu** : `validateError`, `deleteError`, `unvalidateError` rendus après les blocs de statut ; `clearCycleErrors()`
  au début de chaque geste ; `isStaleState` relit aussi sur `SUPPLIER_INVOICE_PAID` et
  `SUPPLIER_INVOICE_MUST_BE_UNVALIDATED_FIRST`.
- **Écartée** : relire après tout `409` (trop large : `SUPPLIER_INVOICE_IN_PAYMENT_BATCH` ne dit pas l'écran périmé).
- **Réversible** : oui.
- ⚠️ **Annotation du 2026-11-03, revue de code P2 (B2-2 = A2-2 = E2-1)** : « au début de chaque geste » ne valait que
  pour valider, dévalider et supprimer. `clearCycleErrors()` est désormais appelée aussi par **payer** (au début du
  geste), **annuler le règlement** et **annuler la facture** (après la confirmation) — tout geste qui change le statut.
  Joindre et détacher un document ne changent pas le statut et ne l'appellent pas.

### C-v017-4-65 — Le refus des lots : « Facture qui n'est pas à payer (brouillon, payée ou annulée) »

- **Contexte** : B-1 ; arbitrage 2. Le serveur rend `SUPPLIER_INVOICE_NOT_OPEN` pour tout statut autre que `open`.
- **Retenu** : un libellé vrai pour les trois états, aux quatre catalogues et au repli ; un banc relie le repli au
  catalogue fr-CH (la mutation du seul catalogue survivait).
- **Écartée** : distinguer le brouillon au serveur (backend de la 4a, hors périmètre).
- **Réversible** : oui.

### C-v017-4-66 — K8 dit « … et N autre(s) » au-delà des vingt brouillons nommés

- **Contexte** : E-10. C-v017-4-58 a écarté un filtre d'URL sur la liste.
- **Retenu** : la ligne « … et N autre(s), dans la liste des factures fournisseurs. » quand `count` dépasse les
  brouillons nommés. La liste, limitée à 100 sans pagination, peut ne pas les montrer tous : limite tracée (issue
  demandée).
- **Réversible** : oui.

### C-v017-4-67 — L'exercice postérieur clos se nomme ; le verrou de période reste sans bornes

- **Contexte** : E-4. `LATER_FISCAL_YEAR_CLOSED` porte `details.fiscalYearName` ; `PERIOD_LOCKED` ne porte aucun
  `details`.
- **Retenu** : une clé `-unvalidate-blocked-later-fiscal-year-closed-named` quand le nom est rendu, la clé générique
  sinon. Pour `PERIOD_LOCKED`, aucune borne à l'écran tant que le serveur n'en rend pas : issue demandée (backend).
- **Réversible** : oui.

### C-v017-4-68 — Édition : le taux expiré d'une ligne est proposé, les projets archivés ne le sont pas

- **Contexte** : E-7, B-7.
- **Retenu** : `extraRates` — les taux des lignes du brouillon absents des taux actifs, proposés comme options ;
  `projectsRes.filter((p) => !p.archived || p.id === inv.projectId)`.
- **Écartée** : `listVatRates(true)` (tout l'historique dans le sélecteur, pour un cas rare).
- **Réversible** : oui.

### C-v017-4-69 — Les fonds d'avertissement neufs prennent le jeton défini `warning`

- **Contexte** : `bg-warning-soft` n'est défini nulle part (#645, `jetons-non-definis.test.ts`) : la classe n'engendre
  aucune règle.
- **Retenu** : `bg-warning/10` pour l'avertissement du lot (fiche et dialogue) et le badge « Brouillon » de la liste,
  sites de la 4b. Les sites antérieurs (import bancaire, comptes bancaires, `SupplierInvoiceForm`, `SupplierInvoiceTotal`,
  `SupplierCreatePanel`) restent à #645.
- **Réversible** : oui.

### C-v017-4-70 — E2E : la facture annulée et le lot confirmé sont posés par l'API ; captures dans `captures-r1/`

- **Contexte** : B-5 = E-9, A-2, arbitrages 4 et 5.
- **Retenu** : le test « liste, refus de dévalidation, K8… » crée une facture validée puis annulée (absente case
  décochée, présente avec son badge case cochée, absente de nouveau) et une facture payée par un lot confirmé puis
  désengagée (lot nommé sur la fiche et dans la confirmation, dialogue fermé par Échap — la garde KF-043 refuse un
  sélecteur sur le libellé « Annuler ») ; K8 amené à la vue avant sa capture, et capturé seul. Les captures neuves vont
  dans `kesh-gate-logs/v0174b/captures-r1/` : celles de la P1 ne sont pas écrasées (rien ne les rattraperait).
- **Réversible** : oui.
