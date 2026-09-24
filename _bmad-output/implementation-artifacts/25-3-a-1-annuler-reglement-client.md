# Story 25.3-a-1 : Annuler un règlement client — par contre-passation

Status: ready-for-dev

**Issue : [#414]** — `refs #414`, **sans la fermer** : l'issue couvre aussi le fournisseur, que
porte la sœur **25-3-a-2**, et c'est **elle** qui la fermera. ⚠️ Ne mettre `closes` nulle part ici.

**Mère : `25-3-a-annuler-reglement.md`** (statut `split`), découpée le 2026-09-24 par la règle de
découpage (sévérité P2 → P3 non décroissante). ⛔ **La mère reste la SOURCE DES FAITS** : ses trois
passes de validation, son Change Log et ses arbitrages valent ici ; cette fiche les **applique** au
seul côté client et intègre les corrections de la passe 3. Grand-mère :
`25-3-annuler-reglement-et-rapprochement.md` (`split`). **Socle** : `25-3-zero-reverse-in-tx.md`,
mergé (PR #453). **Sœurs** : 25-3-a-2 (fournisseur, s'appuie sur le socle **étendu ici**), 25-3-b
(#418, dé-rapprochement, **appellera** `cancel_settlement_in_tx`), 25-3-c (annulation d'une facture
fournisseur, absorbe #454).

## Story

En tant que comptable,
je veux annuler un règlement client enregistré par erreur,
afin de corriger une imputation sans écriture manuelle au journal, qui laisserait le résiduel et
`paid_at` en désaccord avec les livres.

## Pourquoi

⛔ **Aujourd'hui, une écriture de règlement est définitivement incorrigible** : la route de
contre-passation la refuse (`OWNED_BY_SETTLEMENT`), le gel de la 24-4b interdit de la modifier ou de
la supprimer, et la seule sortie — une écriture manuelle — laisse `invoice_settlements` et `paid_at`
dire le contraire du grand livre. Et le manuel **promet déjà** le geste (`user-manual.tex:1048`,
« annulez d'abord le règlement »).

## ⛔ Le fait qui structure la story — le socle refuse ces écritures

`reverse_in_tx` (`journal_entries.rs:1401`) commence par `reversal_blocker` et rend
`EntryNotReversable` pour tout motif autre que `AccountArchived` (`:1427-1436`). Une écriture de
règlement client y ressort en `OWNED_BY_SETTLEMENT`. Et la levée naïve est **fausse** :
`reversal_blocker` ne rend que le **premier** motif (`:1299-1346`), `OwnedBySettlement` (rang 6)
précède `MatchedBankTransaction` (rang 7), et un règlement créé par **rapprochement**
(`reconciliation.rs:1416`, `:1465-1470`) porte **les deux**. Ignorer le premier contre-passerait en
silence un paiement que la banque dit rapproché. ⇒ **exemption ÉTROITE, précédence poursuivie**.

## Arbitrages de Guy qui s'appliquent ici (2026-09-24)

- **Ligne `invoice_settlements` RETIRÉE**, non marquée annulée (grand-mère).
- **Une facture client envoyée ne se modifie plus** : le chemin reste l'avoir.
- **Un paiement détaché de sa facture redevient à lettrer** — il attend d'être rattaché à une
  facture, au besoin créée pour lui (modèle bexio ; lettrage = Epic 15).
- ⛔ **Q5 — un règlement dont l'écriture est dans un exercice CLOS ne s'annule pas ; rouvrir
  l'exercice doit rester possible**, et c'est le chemin. *C'est un arbitrage* : il prime sur la
  phrase de l'issue #414 (« une date dans un exercice ouvert — jamais à la date d'origine si son
  exercice est clos »), qui décrivait la datation et non ce cas, et **il rend faux le manuel**
  (`:1771-1774`, « elle reste contre-passable […] aucune raison de rouvrir l'exercice ») pour les
  écritures de règlement — à corriger (AC 12).

⚠️ **Ne pas contester ces arbitrages en revue** : en contester la mise en œuvre.

## Acceptance Criteria

### Le socle

1. **Une contre-passation « au titre de son propriétaire ».** `journal_entries` expose une variante
   de `reverse_in_tx` qui reçoit l'**autorité** de l'appelant — le motif qu'il a qualité pour lever
   **et** l'identifiant de la pièce. Ici : `(OwnedBySettlement, invoice_settlements.id)`.
   ⚠️ Le type de l'autorité est un **enum** dont cette story écrit **une** variante ; la 25-3-a-2 y
   ajoutera la sienne (règlement fournisseur) — le nommer et le documenter en conséquence, sans
   écrire de variante que rien n'exerce.

   Exigences, toutes testées :
   - **une seule logique** : `reverse_in_tx` (sans autorité) et la variante délèguent à une même
     fonction interne ; ⛔ aucune quatrième contre-passation ;
   - l'exemption ne vaut que si le motif **ET** l'identifiant de pièce correspondent à l'autorité ;
   - ⛔ **la précédence se poursuit après le motif levé** : le calcul des motifs rend leur **liste
     ordonnée** (ou accepte le motif exempté) ; `reversal_blocker` continue de rendre le **premier**,
     sans changement pour ses appelants (GET de l'écriture, route de contre-passation) ;
   - `AccountArchived` garde son traitement : ignoré au recensement, refusé à l'étape 3 par un
     **400 qui nomme les comptes** (`ReversalAccountsArchived`).

2. **La date est celle du jour, dans un exercice ouvert — tranché par le socle.** `entry_date =
   today`, exercice **ouvert** couvrant le jour, sinon `FiscalYearInvalid` (400). ⚠️ Pas de date
   fournie par l'appelant. Le verrou de période (24-4c) s'applique par `create_in_tx_inner` :
   `PERIOD_LOCKED` (400), sans code neuf. **La borne d'exercice clos (AC 3) appartient au GESTE, pas
   au socle**, dont le comportement ne change pas.

### Ce qui empêche l'annulation

3. **Un type fermé, une précédence figée, une seule fonction.** Un enum `SettlementCancelBlocker`
   (`kesh-db/src/errors.rs`, à côté de `UnvalidationBlocker`, même discipline : un code par motif,
   jamais le générique), et **une** fonction de dépôt
   `settlement_cancel_blocker(executor, company_id, settlement_id)` qui sert **à la fois** la lecture
   (AC 8) et l'écriture (AC 4). Précédence, **testée paire par paire** :

   | rang | variante | code | levable ? | à l'écriture |
   |---|---|---|---|---|
   | 1 | `InvoiceCredited` | `INVOICE_CREDITED` (réemploi de `UnvalidationBlocker` : même état du monde) | **non** — un avoir ne se supprime pas | 409, `DbError::SettlementNotCancellable` |
   | 2 | `FiscalYearClosed` | `FISCAL_YEAR_CLOSED` | oui — un Administrateur rouvre l'exercice (`fiscal_years::reopen`) | 409, `DbError::SettlementNotCancellable` |
   | 3 | `MatchedBankTransaction` | `MATCHED_BANK_TRANSACTION` | oui — dé-rapprocher (25-3-b) | **laissé au socle** : 409 `EntryNotReversable` |
   | 4 | `AccountArchived` | `ACCOUNT_ARCHIVED` | oui — réactiver le compte | **laissé au socle** : 400 qui **nomme** les comptes |
   | 5 | `NoOpenFiscalYearToday` | `FISCAL_YEAR_INVALID` | oui — créer l'exercice du jour | **laissé au socle** : 400 `FiscalYearInvalid` |

   ⛔ **Ordre : le définitif d'abord, puis ce qui se lève, du plus lourd au plus léger** — sans quoi
   l'écran ferait rouvrir un exercice pour découvrir ensuite une facture créditée.
   ⛔ **Les rangs 4 et 5 suivent l'ordre RÉEL du socle**, qui contrôle les comptes archivés
   (étape 3 de `reverse_in_tx`, `journal_entries.rs:1460-1462`) **avant** l'exercice du jour
   (étape 4, `:1479-1481`). Les écrire dans l'ordre inverse ferait annoncer un motif à la lecture et
   en refuser un autre au clic sur un règlement qui cumule les deux (passe 4, lentille A). *La lecture
   se règle sur l'écriture, jamais l'inverse* — et le test de la paire 4-5 le vérifie des deux
   côtés.

   ⛔ **Qui refuse, à l'écriture.** Le geste ne refuse **lui-même** que les rangs 1 et 2, qui sont
   les siens (le socle ne les connaît pas). Pour les rangs 3 à 5, il **laisse passer** et c'est le
   socle qui refuse, avec son erreur canonique. C'est ce qui garde le 400 qui nomme les comptes
   (rang 4) et fait du socle la **seule** garde écrite du rapprochement (rang 3) — pas deux gardes
   qui se recouvrent, dont l'une masquerait la mutation de l'autre.

   **Lecture de « exercice clos »** : `fiscal_years.status = 'Closed'`, joint par
   `journal_entries.fiscal_year_id` de l'écriture de **règlement**. Patron : `delete_in_tx`
   (`journal_entries.rs:1014-1035`). **Lecture de « exercice du jour »** :
   `fiscal_years::find_open_covering_date(today)`, la fonction même du socle.

   ⚠️ **Limite assumée** : le verrou de période **du jour** (24-4c interdit une borne future, une
   borne égale au jour reste possible) n'est **pas** dans le calcul — il se contrôle au clic
   (`PERIOD_LOCKED`, 400). À écrire au doc-comment.

   ⚠️ **Coût de la lecture** : `GET …/settlements` appelle `settlement_cancel_blocker` **par
   règlement** — N petites requêtes bornées par le nombre de règlements **d'une** facture (quelques
   unités). Acceptable et voulu (une seule fonction pour lire et écrire) ; ne pas l'« optimiser » en
   une seconde requête qui dupliquerait la précédence.

   `DbError::SettlementNotCancellable { blocker }` est une **variante neuve** (gabarit :
   `EntryNotReversable`), mappée en 409 avec `blocker.code()` dans `crates/kesh-api/src/errors.rs`.

### Le geste

4. **`cancel_settlement_in_tx(tx, company_id, invoice_id, settlement_id, user_id)`** — sans
   `BEGIN` ni `COMMIT` —, plus son enveloppement transactionnel. Dans cet ordre :
   1. verrou `FOR UPDATE` sur la facture (même ordre que `settle_invoice`) — `NotFound` hors société ;
   2. verrou sur la ligne `invoice_settlements` scopée par `company_id` **et** `invoice_id` —
      `NotFound` sinon ;
   3. `settlement_cancel_blocker` : rangs 1-2 → refus ; rangs 3-5 → on continue (AC 3) ;
   4. contre-passation au titre de `(OwnedBySettlement, settlement_id)` (AC 1) ;
   5. **retrait** de la ligne `invoice_settlements` ;
   6. résiduel par `invoice_settlements::amount_due`, **projection** de `paid_at` (AC 5), `version + 1`
      et `updated_at` **toujours** (`settle_invoice` ne les bumpe qu'au solde ; l'annulation change
      toujours l'état) ;
   7. audit (AC 7), dans la même transaction.

   ⛔ **La 25-3-b l'appellera** après avoir défait le lien bancaire dans la même transaction — c'est
   ce qui lève le rang 3 sans exemption. D'où la forme `_in_tx` publique, et l'interdiction d'y
   écrire quoi que ce soit qui présuppose l'appelant HTTP. ⚠️ **Elle en hérite aussi le rang 2** :
   un rapprochement d'exercice clos ne se défera pas sans réouverture — à écrire dans la fiche de la
   25-3-b quand elle sera spécifiée.

5. **`paid_at` retombe à `NULL` si, et seulement si, le résiduel redevient positif.** Une facture
   réglée en deux fois dont on annule **un** règlement reste partiellement réglée ; celle dont on
   annule l'**unique** règlement redevient « à régler ». Un résiduel resté ≤ 0 **laisse `paid_at`
   intact** — branche **défensive** : l'application ne produit pas cet état (trop-perçu refusé par
   l'encaissement **et** par le rapprochement ; facture créditée arrêtée au rang 1). Son test forge
   l'état par SQL et **le dit**.

6. **Le règlement d'une facture créditée est un paiement à lettrer — refusé ici, et nommé.** Un avoir
   **peut** viser une facture partiellement réglée (`credit_notes.rs:291-304` ne lit que `paid_at`),
   la bascule en `cancelled` (`:561-564`) et laisse le règlement en place. Ce règlement **n'est pas une
   anomalie** (arbitrage : il est à lettrer, Epic 15) ; cette story **n'y touche pas** — rang 1,
   `INVOICE_CREDITED`, et le texte (AC 9) dit le chemin : le lettrer à une facture. **[#456]** porte
   le reste (faut-il encore refuser l'avoir ; comment signaler ce paiement d'ici l'Epic 15) ;
   **[#455]** porte le reste dû faux d'une telle facture (avoir HT retranché d'un TTC). Les factures
   réglées **avant** `20260827000001` portent `paid_at` sans ligne : rien à annuler, et la story ne
   leur en invente pas.

7. **L'audit** : `invoice.settlement_cancelled`, **littéral** au site d'insertion (hors
   `SITES_INDIRECTS`), inscrit à `audit_labels.rs::ACTIONS` (liste triée), libellé dans les **quatre**
   locales. Charge : montant, date du règlement, `settlementJournalEntryId`,
   `reversalJournalEntryId`, `amountDueAfter`. ⚠️ Le socle écrit **en plus** `journal_entry.reversed`
   sur l'écriture : **deux lignes, c'est voulu**.

### L'API

8. **Les routes.**
   - `POST /api/v1/invoices/{id}/settlements/{settlement_id}/cancel` — **Comptable+**, dans
     `comptable_routes` (`lib.rs`), à côté de `POST /invoices/{id}/settlements`. Clés API
     d'écriture admises, comme la jumelle (gate générique par méthode, `middleware/auth.rs:141`) — le
     dire dans `api-external.md`. Réponse : la facture relue (`InvoiceResponse` + `with_settlement`,
     `routes/invoices.rs:640`) et `reversalJournalEntryId`. Au registre `audit_route_registry.rs` :
     `Traced`, totaux **recomptés** depuis la source, message de ventilation compris (départ :
     106 / 88 / 109).
   - `GET /api/v1/invoices/{id}/settlements` — **n'existe pas** (`list_for_invoice` n'a que des
     appelants de test) ; sans elle, l'écran ne peut pas désigner le règlement. **Tout rôle**
     (`authenticated_routes`, patron `GET /invoices/{id}/reminders`, `lib.rs:736`). Les routes GET
     ne sont pas au registre de routes (il ne porte que les mutantes). Chaque élément : `id`,
     `journalEntryId`, `amount`, `settledOn`, `settlementType`, **`cancellable`**,
     **`cancelBlockedBy`** (code de l'AC 3), **`cancelBlockedLabel`** (le **numéro du compte** au
     rang 4 — sans lui l'écran dirait « réactivez-le » sans dire lequel, cf. `reversal_blocker`
     `:1325-1334`), **`cancelBlockedDocumentId`** (l'identifiant de la **transaction bancaire** au
     rang 3). Tous calculés par `settlement_cancel_blocker`.
   - ⚠️ `cancellable` ne tient pas compte du **rôle** : un utilisateur Consultation lira `true` et
     recevra 403 au clic. C'est le patron des écrans existants ; l'écran masque le bouton par rôle
     (AC 10), le serveur refuse de toute façon.

### Les textes

9. **Les motifs de l'annulation ont leurs PROPRES textes, écrits UNE fois.**
   - Famille **`invoices-settlement-cancel-blocked-*`** — préfixe **pluriel**, pour passer
     `lint-i18n-ownership` depuis `frontend/src/lib/features/invoices/` (le singulier `invoice-`
     l'y fait échouer, dette #30, cf. les `KNOWN_VIOLATIONS` du script) —, une clé par code de
     l'AC 3, **quatre** locales, **plus** un texte d'erreur pour `SettlementNotCancellable` au repli
     serveur (`crates/kesh-api/src/errors.rs`).
   - **Un seul** module dans `features/invoices/` qui mappe `cancelBlockedBy` → texte, en `switch`
     exhaustif avec garde `never`. ⛔ Ne pas recopier le mapping de `journal-entries/[id]/+page.svelte`
     (autre écran, « cette écriture »), ne pas le détourner.
   - Textes : rang 1, le chemin est le **lettrage** (paiement à lettrer) ; rang 2, **rouvrir
     l'exercice** (Administrateur) ; rang 3, **annuler d'abord le rapprochement** — tant que la
     25-3-b n'est pas livrée, sans promettre de bouton ; rang 4, **réactiver le compte n° X** ;
     rang 5, **créer l'exercice** couvrant le jour.
   - Le repli en dur dit **mot pour mot** le FTL fr-CH.

   **Les motifs de la contre-passation directe** : seul `OWNED_BY_SETTLEMENT` change ici (« son
   annulation viendra avec la contre-passation des règlements » → nommer le chemin : annuler le
   règlement depuis la fiche de la facture, **qui indique si c'est possible**). Sites, tous
   ensemble (grep de la **clé** et du **code**) : FTL `journal-entries-reverse-blocked-settlement`
   ×4 (fr-CH `:343`, autres `:349`), repli serveur `errors.rs:2466-2478`, repli Svelte
   `journal-entries/[id]/+page.svelte:151-165`. ⚠️ `OWNED_BY_SUPPLIER_INVOICE` est l'affaire de la
   25-3-a-2 ; `MATCHED_BANK_TRANSACTION` celle de la 25-3-b. **Les refus restent** : contre-passer
   directement une écriture de règlement reste faux.
   ⚠️ **Défaut antérieur dans le même `switch`, corrigé au passage** : le repli de `OWNED_BY_INVOICE`
   (`:143`, « corrigez-la par un avoir ») ne dit pas ce que dit le FTL fr-CH (`messages.ftl:340`,
   « dévalidez la facture, ou corrigez-la par un avoir »).

### L'écran

10. **Fiche facture client** (`invoices/[id]/+page.svelte`) : la liste des règlements (date, montant,
    mode, lien vers l'écriture) ; un bouton « Annuler le règlement » par ligne **`cancellable`**,
    masqué pour un rôle sans droit d'écriture ; le **motif** (AC 9) à la place du bouton sinon.
    Confirmation avant l'envoi, qui dit ce qui va se passer (« une écriture inverse datée
    d'aujourd'hui sera passée »). Après succès : liste, totaux, statut et bouton « Enregistrer un
    règlement » **relus**. Chaque refus au clic affiche son motif — le 409 porte `code` et `details`,
    le 400 `ACCOUNT_ARCHIVED` porte `details.rejected[]` —, jamais « Transition interdite ».

    **Commentaires devenus faux** — inventaire par
    `grep -rn "#414" frontend/src frontend/tests crates/` :
    `invoices/[id]/+page.svelte:341-342`, `:697-699` (« rien ne le remplace »), `:1178-1184` ;
    `invoices.api.ts:124` ; `i18n-keys.test.ts:139` ; `kesh-api/src/lib.rs:516-518` ;
    `kesh-db/src/errors.rs:70-72` (doc d'`OwnedBySettlement`, « Chemin : #414 ») ;
    `repositories/invoices.rs:2188` ; `tests/e2e/invoices.spec.ts:322-324` ;
    `tests/e2e/invoices_echeancier.spec.ts:173-182` — ce dernier **affirme l'absence** du geste :
    le réécrire pour qu'il vérifie sa **présence**. ⚠️ Les mentions de #414 qui visent le
    **fournisseur** (`errors.rs:67-68`) restent pour la 25-3-a-2 : trier chaque site, ne pas les
    réécrire en bloc.

### Tests

11. ⛔ Chaque garde **prouvée par mutation**, vue **rouge sur assertion**, mutation décrite au Dev
    Agent Record.
    - **Socle** : exemption étroite (un autre `settlement_id` ne lève rien) ; précédence poursuivie —
      ⛔ **prouvée par un appel DIRECT à la variante** sur une écriture de règlement rapprochée, qui
      doit rendre `MATCHED_BANK_TRANSACTION` ; mutation « l'exemption court-circuite toute la
      précédence » ⇒ rouge. `reverse_in_tx` sans autorité **inchangé** : les **27** tests de
      `journal_entry_reversal_e2e` passent **sans retouche**.
    - **Précédence du geste** (AC 3) : chaque rang seul, et **chaque paire** de rangs cumulés,
      produits par les **vrais** chemins — avoir après règlement partiel (`settle_invoice` puis
      `create_credit_note`), clôture (`fiscal_years::close`), rapprochement
      (`accept_one_invoice`), exercice du jour absent, archivage d'un compte. Mutation : inverser
      deux rangs ⇒ rouge.
    - **Paire 4-5** (compte archivé **et** exercice du jour absent) : la lecture rend
      `ACCOUNT_ARCHIVED` **et** l'écriture aussi — c'est la preuve que la lecture suit le socle.
    - **Exercice du jour absent** (rang 5) : ⚠️ **montage explicite** — le règlement dans un exercice
      **ouvert** qui **ne couvre pas** le jour, et **aucun** exercice couvrant le jour (le cas de
      janvier). Le montage « régler aujourd'hui puis clore » produit le rang **2**, pas le 5.
    - **Client** : règlement unique annulé (résiduel = TTC, `paid_at` NULL, ligne retirée, écriture
      inverse avec `reverses_entry_id`, l'origine en `ALREADY_REVERSED` et l'inverse en
      `IS_A_REVERSAL`) ; règlement **partiel** parmi deux (AC 5) ; branche défensive sur état
      **forgé et déclaré tel** ; double annulation → la seconde rend `NotFound`.
    - ⛔ **Composition et rollback** : dans une transaction fournie, appeler `cancel_settlement_in_tx`,
      **lire dans la transaction** que la contre-passation existe et que la ligne a disparu
      (lecture **positive**), puis **abandonner** la transaction (drop sans commit) — et vérifier
      qu'après, rien n'a changé. C'est le cas réel de la 25-3-b. ⚠️ Pas de montage par « conflit de
      version » : la version est lue sous verrou dans la même transaction, l'état est impossible.
    - **Étanchéité multi-tenant** : société B ne peut ni lister ni annuler un règlement de A
      (`NotFound`), et **rien n'est écrit** — vérifié table par table : `journal_entries`,
      `journal_entry_lines`, `invoice_settlements`, `invoices`, `audit_log`.
    - **API** : `POST …/cancel` → 200, **403 pour Consultation**, 404, 409 avec `code` et `details` ;
      `GET …/settlements` → **200 pour tout rôle**, Consultation compris, 404 hors société.
    - **Vitest** : liste, bouton masqué avec motif (chaque code), masqué par rôle, relecture après
      succès. **Gardes i18n recomptées** (`i18n-keys.test.ts` `ATTENDU.*`,
      `i18n-un-repli-par-cle.test.ts` `CLES_RELEVEES`, `i18n-libelle-en-dur.test.ts`
      `CANDIDATES_ATTENDUES`) — **recomptées depuis la source**, jamais incrémentées de confiance.
    - **Playwright** : régler puis annuler, de bout en bout.

### Documentation et gates

12. **Manuel utilisateur FR** (`docs/manual/fr/user-manual.tex`) :
    - ⚠️ **aucune section n'existe sur l'enregistrement d'un règlement client** — seulement des
      mentions incidentes (`:338`, `:348`) et une formule périmée (`:782`, « Payée : marquée comme
      payée après réconciliation »). Créer, près de « Échéancier des factures » (`:925`), une
      sous-section **« Enregistrer et annuler un règlement »** (le premier geste est un manque
      antérieur, comblé parce que le second en a besoin), et corriger `:782` ;
    - ce que l'annulation écrit, ce qu'elle refuse et pourquoi (les cinq motifs, dont le règlement
      d'un exercice **clos** → rouvrir l'exercice) ;
    - ⛔ **`:1771-1774` devient faux** pour les règlements (Q5) : le corriger ;
    - `:1048` (dévalidation : « annulez d'abord le règlement ») devient vrai — le relire ;
    - `:1769` (contre-passation : « passez par le chemin de la pièce ») — reste vrai : **le relire**,
      sans le modifier ;
    - ⚠️ **défaut antérieur dans la section que cette story touche** : l'encadré `:514-515` affirme
      qu'« aucun verrou de période plus fin […] n'existe encore », faux depuis la 24-4c (`:454`) —
      corriger.

    Régénérer le PDF et le **contrôler aplati** (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`).
    `api-external.md` : les deux routes, clés API d'écriture admises. **`README.md`** : le bloc
    « Fonctionnalités » ne porte aucune entrée pour le règlement client (seule la ligne 40 couvre le
    fournisseur) — en ajouter une (enregistrer et annuler un règlement), règle de synchronisation du
    README. `website/` : rien (le site ignore toute l'Epic 24, dette antérieure hors périmètre).
    ⚠️ **`:1771-1774` ne se corrige ici que pour le règlement CLIENT** : la 25-3-a-2 l'étend au
    fournisseur quand son geste existera — l'écrire générique maintenant décrirait un geste absent. `CHANGELOG.md`, section
    **`[0.12.1] — Non publié`** — aucune ligne d'une section publiée ne se réécrit, et le décompte
    « 94 actions » **se recompte**.

13. **Gate complet** — `kesh-db` touché : ciblage interdit, même en boucle de revue. Base remise à
    zéro avant chaque gate complet.

## Tasks / Subtasks

- [ ] **T1 — Le socle** (AC 1, 2) : motifs en liste ordonnée, autorité, `reverse_in_tx` inchangé.
- [ ] **T2 — Les motifs du geste** (AC 3) : `SettlementCancelBlocker`, `settlement_cancel_blocker`,
      `DbError::SettlementNotCancellable` et son mapping.
- [ ] **T3 — Le geste** (AC 4, 5, 6) et l'audit (AC 7).
- [ ] **T4 — Les routes** (AC 8), registre de routes recompté.
- [ ] **T5 — Les textes** (AC 9) : famille `invoices-settlement-cancel-blocked-*` ×4, module de
      mapping, `OWNED_BY_SETTLEMENT` ×4 + replis, correction du repli `OWNED_BY_INVOICE`.
- [ ] **T6 — L'écran** (AC 10) et les commentaires devenus faux.
- [ ] **T7 — Tests et mutations** (AC 11).
- [ ] **T8 — Documentation** (AC 12), gates (AC 13), PR en `refs #414`.

## Dev Notes

### Ce que cette story ne fait pas

- **Le fournisseur** : 25-3-a-2 (même socle, sa propre variante d'autorité et ses motifs).
- **Le dé-rapprochement** : 25-3-b (#418). Un règlement rapproché est refusé par le socle.
- **L'avoir**, l'annulation d'une facture (25-3-c pour le fournisseur), `cancel` (#454), #455, #456,
  #416 (résiduel dans les rapports), #384 (écart de règlement).

### Ce qu'il faut savoir du code existant

- **Encaissement** : `invoice_settlements_write.rs::settle_invoice` — verrou facture, compte de
  créance lu **sur l'écriture de vente**, trop-perçu refusé, écriture `D contrepartie / C créance`,
  ligne `invoice_settlements`, `paid_at` si résiduel ≤ 0, audit par ternaire (→ `SITES_INDIRECTS`).
  Par rapprochement : `reconciliation.rs::accept_one_invoice` (`:1056`), même ligne (`:1416`) +
  `matched_entry_id` (`:1465-1470`). Des cinq sites qui posent `matched_entry_id`, c'est le seul
  qui crée une ligne de règlement.
- **Contre-passation** : `reverse_in_tx` (`:1401-1530`) — verrou, motifs, lignes par `line_order`,
  comptes archivés, exercice ouvert **du jour**, `create_in_tx_inner(..., Some(id))`, audit. Le
  projet analytique est porté **par ligne** (`journal_entries.rs:426`) : la contre-passation le
  **préserve**. Ne pas modifier l'ordre du socle (contrat de la 25-3-zero).
- **Conséquences voulues** : une facture dont tous les règlements sont annulés redevient
  **relançable** et, faute d'autre motif, **dévalidable** (`UnvalidationBlocker::Settled` lit la
  ligne **ou** `paid_at`).
- **Erreurs** (`crates/kesh-api/src/errors.rs`) : `EntryNotReversable` → 409 + `details{documentId,
  documentNumber}` ; `ReversalAccountsArchived` → 400 + `details.rejected[]` ; `FiscalYearInvalid`
  → 400 ; `PeriodLocked` → 400 ; `NotFound` → 404 ; `DbError::FiscalYearClosed` → 400 (`:2682`,
  **non réemployé** ici : le rang 2 passe par `SettlementNotCancellable`, pour un code unique par
  geste).
- **Frontend** : pas de fichiers de langue propres (`i18nMsg(key, fallback)` lit les FTL du
  serveur) ; affichage d'erreur par `isApiError` / `notifyError`.

### Pièges nommés d'avance

1. L'exemption large — **muette** : seul le règlement rapproché la trahit.
2. Deux gardes du même motif — l'une masque la mutation de l'autre (d'où AC 3 : le socle seul garde
   les rangs 3-5).
3. Le test sur un état que l'application ne produit pas — d'où les vrais chemins de l'AC 11.
4. Le test de rollback aux assertions négatives.
5. Un motif corrigé à un site sur quatre ; des totaux incrémentés au lieu d'être recomptés.

### Règle de splitting

Modules : `kesh-db`, `kesh-api`, `kesh-i18n`, `frontend` — quatre. Issue elle-même d'un découpage.

### References

- [Source: crates/kesh-db/src/repositories/journal_entries.rs:1234-1555] — `reversal_blocker`, `reverse_in_tx`, `reverse`.
- [Source: crates/kesh-db/src/repositories/journal_entries.rs:1014-1035] — `delete_in_tx`, lecture du statut d'exercice.
- [Source: crates/kesh-db/src/errors.rs:57-170] — `ReversalBlocker`, `UnvalidationBlocker` (gabarit).
- [Source: crates/kesh-db/src/repositories/invoice_settlements_write.rs] — `settle_invoice`.
- [Source: crates/kesh-db/src/repositories/invoice_settlements.rs] — `amount_due`, `list_for_invoice`.
- [Source: crates/kesh-db/src/repositories/credit_notes.rs:291-304,561-564] — avoir sur facture partiellement réglée.
- [Source: crates/kesh-api/src/routes/reconciliation.rs:1416,1465-1470] — règlement rapproché.
- [Source: frontend/scripts/lint-i18n-ownership.js] — règle de propriété des clés.
- [Source: docs/manual/fr/user-manual.tex:514-515,782,1048,1769,1771-1774] — ce que le manuel dit.
- [Source: _bmad-output/implementation-artifacts/25-3-a-annuler-reglement.md] — mère, source des faits.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Étape | Note |
|---|---|---|
| 2026-09-24 | validate P4 | **Deux lentilles Sonnet** en contexte frais, prompt `25-3-a-1-validate-prompt-p4.md`, axes déclarés (non exercés : recompte détaillé des gardes i18n, exécution). **3 MEDIUM, 2 LOW** — **aucun HIGH** : la sévérité décroît (P3 : HIGH + MEDIUM). ✅ **MEDIUM (A)** : la table mettait l'exercice du jour (rang 4) avant le compte archivé (rang 5), l'**inverse** de l'ordre réel du socle (`reverse_in_tx` : étape 3 archivés, étape 4 exercice du jour) — lecture et écriture auraient rendu deux motifs différents ; rangs 4 et 5 **permutés**, test de la paire des deux côtés, jumelle 25-3-a-2 corrigée du même défaut. ✅ **MEDIUM (B)** : `:1771-1774` du manuel tombait entre les deux fiches — ici client seulement, la 25-3-a-2 l'étend ; **`README.md`** absent de l'AC 12 — ajouté. LOW : `:1769` sans verbe (« le relire ») ; coût de la lecture par règlement écrit comme voulu. Symptôme grepé (« rang 4 », « rang 5 », rangs de la table) sur les deux fiches : six sites repris dans la 25-3-a-1 (table, garde du 400, libellé, textes, montage, « pas le 5 »), deux dans la 25-3-a-2 (table, libellé) ; lignes du socle citées **recomptées** (`:1460-1462`, `:1479-1481` — la lentille donnait des lignes approchées). |
| 2026-09-24 | split | Née du découpage de la 25-3-a (règle de découpage : sévérité P2 → P3 non décroissante ; découpage proposé indépendamment par les deux lentilles de la passe 3). Intègre les corrections de la passe 3 pour le côté client : **Q5 tranchée par Guy** (règlement d'exercice clos → refus, **rouvrir l'exercice** reste le chemin ; le manuel `:1771-1774` devient faux) ; **type fermé** `SettlementCancelBlocker` et précédence **définitif d'abord** (`INVOICE_CREDITED`, puis exercice clos, rapprochement, exercice du jour absent, compte archivé) ; **une seule garde par motif** à l'écriture (le geste refuse ses rangs, le socle les autres — ce qui garde le 400 qui nomme les comptes et rend la mutation de l'AC 7 observable) ; champs `cancelBlockedLabel` / `cancelBlockedDocumentId` ; exercice du jour absent **dans** `cancellable`, verrou de période du jour en **limite assumée** ; clés au préfixe **pluriel** `invoices-` (lint d'appartenance) ; test de rollback par **composition** au lieu d'un conflit de version impossible ; montage explicite du rang 4 ; inventaire des commentaires #414 élargi à `frontend/src` et `crates/` ; deux défauts antérieurs corrigés au passage (repli `OWNED_BY_INVOICE`, encadré `:514-515` du manuel). `refs #414` seulement : la 25-3-a-2 fermera l'issue. |
