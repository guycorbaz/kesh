# Story 25.3-a-2 : Annuler un règlement fournisseur — par contre-passation

Status: ready-for-dev

**Issue : [#414]**, qu'elle **ferme** : `closes #414` dans le **titre ET le corps** de la PR (squash).
⛔ **Derrière la 25-3-a-1**, qui porte le côté client et **pose tout ce que celle-ci réutilise** :
l'enum d'autorité du socle, l'enum `SettlementCancelBlocker`, `DbError::SettlementNotCancellable`,
la **queue commune** des motifs `settlement_entry_cancel_blocker(entry_id)` et le module partagé de
ses textes. Ne pas commencer avant son merge.

**Mère : `25-3-a-annuler-reglement.md`** (`split`) — **source des faits**, avec ses trois passes.
Sœur : **25-3-a-1** (validée). Voisine : **25-3-c** (annuler une facture fournisseur **payée**,
sauf exercice clos ; absorbe #454).

## Story

En tant que comptable,
je veux annuler le règlement d'une facture fournisseur enregistré par erreur, pour la ramener à
« ouverte »,
afin de la régler à nouveau correctement — ou de l'annuler ensuite par le bouton « Annuler la
facture », qui existe déjà pour une facture ouverte (`cancel`).

## Ce qui distingue le fournisseur du client

Le règlement fournisseur n'a **pas** de ligne `invoice_settlements` : il est porté par la **ligne
de facture** (`status`, `settlement_type`, `settlement_bank_account_id`, `settlement_account_id`,
`settlement_journal_entry_id`, `paid_at`), un seul règlement, pour le TTC
(`supplier_invoices.rs::pay_in_tx`, `:542`). Il n'est jamais rapproché d'une transaction bancaire :
les cinq sites de `reconciliation.rs` qui posent `matched_entry_id` (`:1467`, `:1892`, `:2256`,
`:2948`, `:3409`) lient tous une écriture **qu'ils viennent de créer** (vérifié en passes 1, 3 et 4).

## Arbitrages de Guy qui s'appliquent ici (2026-09-24)

- **Annuler le règlement ramène la facture à `open`.** L'annulation d'une facture **payée** — son
  règlement redevenant **à lettrer** — est la **25-3-c**, pas celle-ci.
- **Lot pain.001 confirmé** : l'annulation du règlement est autorisée, le lot est **laissé tel
  quel** (historique de l'ordre transmis ; cas d'usage : le rejet bancaire).
- **Règlement d'un exercice CLOS : refusé ; rouvrir l'exercice reste le chemin** (Q5).

⚠️ **Ne pas contester ces arbitrages en revue** : en contester la mise en œuvre.

## Acceptance Criteria

1. **L'autorité fournisseur ne couvre QUE l'écriture de règlement — et c'est le SOCLE qui le
   vérifie.** Nouvelle variante de l'enum d'autorité de la 25-3-a-1 : règlement fournisseur,
   portant `supplier_invoices.id`. ⛔ `reversal_blocker` attribue `OwnedBySupplierInvoice` à
   l'écriture d'achat **comme** à celle de règlement (`journal_entries.rs:1271-1276`,
   `purchase_journal_entry_id = je.id OR settlement_journal_entry_id = je.id`) : la règle
   « motif et identifiant de pièce correspondent » de la 25-3-a-1 **exempterait donc aussi
   l'écriture d'achat**.
   - Le contrôle vit dans la **fonction interne du socle**, au moment où cette variante est
     appariée : `SELECT 1 FROM supplier_invoices WHERE id = ? AND company_id = ? AND
     settlement_journal_entry_id = <écriture>` — faute de ligne, **aucune exemption**, et le motif
     `OWNED_BY_SUPPLIER_INVOICE` est opposé comme aujourd'hui. ⚠️ **Pas dans le geste** : le geste ne
     passe jamais que `settlement_journal_entry_id`, le contrôle y serait vrai par construction et
     **aucun test ne pourrait le mettre en défaut**.
   - Test : appel **direct** de la variante du socle sur l'**écriture d'achat** de la facture →
     refus ; mutation « supprimer le contrôle » ⇒ rouge.
   - ⚠️ **Ordre imposé dans le geste** : la contre-passation (qui lit la colonne) **précède**
     l'`UPDATE` qui la vide (AC 3) — inverser les deux ferait échouer l'autorité.

2. **Les motifs : une tête fournisseur, la queue COMMUNE.**

   | rang | motif | code | à l'écriture |
   |---|---|---|---|
   | 1 | facture non `paid` | **`SUPPLIER_INVOICE_NOT_PAID`** (variante `SupplierInvoiceNotPaid` de `SettlementCancelBlocker`, ajoutée ici) | 409 `DbError::SettlementNotCancellable` |
   | 2, 4, 5 | **queue commune** de la 25-3-a-1, **numérotée comme elle** : rang 2, exercice de l'écriture de règlement clos (`FISCAL_YEAR_CLOSED`, par le geste, 409) ; rang 4, compte archivé (`ACCOUNT_ARCHIVED`, par le socle, 400 qui **nomme**) ; rang 5, aucun exercice ouvert le jour (`FISCAL_YEAR_INVALID`, par le socle, 400). **Rang 3 volontairement absent** (`MATCHED_BANK_TRANSACTION`, inatteignable ici) | — | — |

   ⛔ **Aucun jumeau** : la tête fournisseur `supplier_settlement_cancel_blocker(executor,
   company_id, supplier_invoice_id)` évalue le rang 1 puis **appelle**
   `settlement_entry_cancel_blocker(settlement_journal_entry_id)`. La queue rend aussi
   `MATCHED_BANK_TRANSACTION` — inatteignable côté fournisseur (§ ci-dessus), mais c'est le socle qui
   en garde, et le calcul ne doit pas le **supposer**.
   ⚠️ Le rang 1 **coupe court par construction** : une facture non `paid` n'a pas d'écriture de
   règlement, la queue ne s'évalue pas. Aucune paire « 1 + autre » n'existe.
   Le code `SUPPLIER_INVOICE_NOT_PAID` ne collisionne avec rien (`SUPPLIER_INVOICE_NOT_OPEN`,
   `payment_batches.rs:265`, a un autre sens).

3. **Le geste.** Wrapper + cœur `_in_tx`, sur le modèle de `pay` / `pay_in_tx` :
   1. verrou facture `FOR UPDATE`, **puis** l'écriture de règlement **et son exercice**
      (`… JOIN fiscal_years … FOR UPDATE`) — ⛔ **obligatoire**, trouvé en passe 1 de revue de
      code de la 25-3-a-1 : la queue commune lit l'exercice **sans verrou**, et le socle ne
      verrouille l'exercice de l'origine qu'après ; sans ce verrou, une clôture concurrente
      passerait entre les deux. Patron : `cancel_settlement_in_tx`, étape 2-bis, et son test
      d'entrelacement `une_cloture_concurrente_attend_l_annulation` ;
   2. `supplier_settlement_cancel_blocker` : rangs 1 et « exercice clos » → refus ; les autres → on
      continue, le socle refuse (même partage que la 25-3-a-1) ;
   3. contre-passation de `settlement_journal_entry_id` au titre de l'autorité (AC 1) ;
   4. `UPDATE` → `status = 'open'`, `settlement_type`, `settlement_bank_account_id`,
      `settlement_account_id`, `settlement_journal_entry_id` et `paid_at` **à `NULL`**, `version + 1`,
      garde `AND version = ? AND status = 'paid'` (défense en profondeur ; la version est lue sous
      verrou, 0 ligne n'arrive pas en usage normal) ;
   5. audit (AC 4), qui **conserve** l'identifiant de l'écriture de règlement annulée et celui de sa
      contre-passation — seule trace du lien, la colonne étant remise à `NULL`.

   ⚠️ **Pourquoi vider les colonnes** : il n'y en a qu'un jeu, et un nouveau règlement l'écraserait.
   `chk_supplier_invoices_paid_has_settlement` n'impose rien à une facture `open`. Le lien survit
   dans l'audit **et** au grand livre (l'écriture d'origine ressort ensuite en `ALREADY_REVERSED`).

   ⚠️ **Pourquoi le geste n'appelle PAS `guard_not_in_generated_batch`** : une facture `paid` ne peut
   pas être dans un lot `generated` — `create_batch` exige `open` (`payment_batches.rs:265`), `pay`
   et `cancel` refusent une facture en lot `generated` (`supplier_invoices.rs:474-503`), et
   `confirm_batch` règle la facture et passe le lot en `confirmed` dans la même transaction
   (`:360-376`). Ne pas recopier la garde, ne pas la chercher.

   ⛔ **Lot confirmé, puis nouveau lot : le risque de DOUBLE PAIEMENT.** Une facture revenue à `open`
   repart dans la liste des factures proposées pour un lot (`create_batch` n'écarte que les
   non-`open` et les lots `generated`, `payment_batches.rs:265-283` ;
   `payment-batches/+page.svelte:52,90`). Si l'ordre a **réellement** été exécuté par la banque et
   qu'on annule le règlement pour une autre raison (mauvaise date, mauvais compte), un nouveau lot
   **paierait deux fois**. L'arbitrage autorise l'annulation ; la story **le dit** :
   - la réponse de lecture porte **`lastConfirmedBatch`** — `{ id, confirmedAt }` du lot confirmé
     **le plus récent** (`ORDER BY confirmed_at DESC, id DESC`) qui contient la facture, ou `null` ;
   - ⛔ **Ce champ est HISTORIQUE, et il est nommé comme tel.** `payment_batch_items` ne relie
     aucune ligne à un règlement précis (aucune colonne vers l'écriture, migration
     `20260628000002`), et ses lignes survivent à toute annulation. Après un cycle « lot confirmé →
     annulation → nouveau règlement **direct** », le lot le plus récent **n'a pas produit** le
     règlement courant. Le champ ne prétend donc **jamais** dire d'où vient le règlement courant
     (passe 5, lentille B : c'était le défaut de la première rédaction, qui aurait affiché un fait
     **faux et nommé**) ;
   - la **confirmation** d'annulation, s'il est renseigné, dit un fait **toujours vrai** : « cette
     facture figure dans le lot de paiement n° X, confirmé le … ; si la banque a exécuté cet ordre,
     elle a déjà été payée — corrigez alors par un nouveau règlement direct, jamais par un nouveau
     lot » ;
   - ⚠️ *Alternative écartée* : une colonne liant le règlement au lot qui l'a produit (migration,
     écriture dans `confirm_batch`) rendrait l'avertissement exact au lieu d'historique ; elle
     déborde de l'objet de la story, et l'avertissement historique suffit à prévenir le double
     paiement ;
   - le manuel le dit (AC 10).

4. **L'audit** : `supplier_invoice.settlement_cancelled`, littéral, inscrit à `ACTIONS` (triée),
   libellé ×4. Le socle écrit en plus `journal_entry.reversed` — voulu.

5. **La route** : `POST /api/v1/supplier-invoices/{id}/settlement/cancel` — **Comptable+**, dans
   `comptable_routes`, à côté de `pay` et `cancel` ; clés API d'écriture admises, comme ses jumelles
   (gate générique par méthode). Registre de routes : `Traced`, totaux **recomptés depuis la
   source** (ils auront bougé avec la 25-3-a-1).

6. **Les champs de lecture, et où ils vivent.** `SupplierInvoiceResponse` gagne
   `settlementCancellable`, `settlementCancelBlockedBy`, `settlementCancelBlockedLabel` (numéro du
   compte archivé) et `lastConfirmedBatch` (AC 3). ⚠️ `from_parts`
   (`routes/supplier_invoices.rs:90`) est **synchrone** et a **cinq** appelants (`:251`, `:287`,
   `:312`, `:327`, `imported_supplier_invoices.rs:282`). Patron : `InvoiceResponse::with_settlement`
   (`routes/invoices.rs:640`) et sa discipline (`:240-250`) — **`Option`, sérialisé `null` quand
   non calculé** (« l'absence se lit, le zéro ment »), jamais une valeur par défaut qui mentirait.
   Constructeur additionnel appelé par le **GET**, la réponse de **`pay`** (qui remplace l'état de
   l'écran, `+page.svelte:87`) et celle de l'**annulation**. `SupplierInvoiceListItemResponse`
   n'est pas touché.

7. **L'écran** (`supplier-invoices/[id]/+page.svelte`) — mêmes exigences que la 25-3-a-1 (AC 10) :
   - bouton « Annuler le règlement » si **`status === 'paid' && settlementCancellable === true`**,
     masqué pour un rôle sans droit d'écriture (la page ne teste aucun rôle aujourd'hui, `:215-279`) ;
   - le **motif** à la place du bouton **seulement** si `status === 'paid'` et
     `settlementCancellable !== true` — jamais sur une facture `open` ou `cancelled` (le rang 1
     afficherait « non payée » à côté du formulaire « Payer ») ;
   - **confirmation** avant l'envoi, qui dit ce qui va s'écrire, avec l'avertissement de lot de
     l'AC 3 le cas échéant ;
   - ⛔ **refus au clic affiché LOCALEMENT** (409 `code`, 400 `details.rejected[]`) — **jamais** dans
     `errorMsg`, qui remplace **toute la fiche** (`:107`, `:146-147`) ;
   - lien vers l'écriture de règlement, absent aujourd'hui (`:280-284`).

8. **Les textes.**
   - La **queue** : le module **partagé** de la 25-3-a-1 (famille `settlement-cancel-blocked-*`),
     **réutilisé**, sans jumeau.
   - La **tête** : clé **`supplier-invoices-settlement-cancel-blocked-not-paid`** dans
     `features/supplier-invoices/` — `lint-i18n-ownership` compare un dossier **à tiret** par
     préfixe `${feature}-` (`keyBelongsToFeature`, script `:176`) : ce préfixe passe. Le type
     TypeScript du fournisseur est l'union **de sa tête et de la queue** : aucun cas mort.
   - **Quatre** locales ; repli serveur de `SettlementNotCancellable` pour le nouveau code ; replis
     en dur **mot pour mot** le FTL fr-CH. Le texte de tête : il n'y a pas de règlement à annuler
     sur une facture non payée.
   - **Le motif de contre-passation directe `OWNED_BY_SUPPLIER_INVOICE`** : son texte (« annulez la
     facture ») est **faux pour l'écriture de règlement**, et il couvre **aussi** l'écriture d'achat.
     Nouveau texte **qui couvre les deux rôles** sans changer `reversal_blocker` : « Cette écriture
     appartient à une facture fournisseur : annulez la facture ou son règlement depuis sa fiche. »
     Sites, ensemble : FTL `journal-entries-reverse-blocked-supplier-invoice` ×4 (fr-CH `:342`,
     autres `:348`), repli serveur `errors.rs:2467-2470` (le bras
     `OwnedBySupplierInvoice`), repli Svelte `journal-entries/[id]/+page.svelte:151-165`, doc-comment `kesh-db/src/errors.rs:67-68`, manuel
     (`:502-507`, `:1769`). ⚠️ **Limite assumée** : pour l'écriture d'achat d'une facture **déjà
     annulée**, le texte reste approximatif — `cancel` ne pose pas `reverses_entry_id`
     (`supplier_invoices.rs:820-836`), si bien que `reversal_blocker` rend encore
     `OWNED_BY_SUPPLIER_INVOICE` au lieu d'`ALREADY_REVERSED`. L'ancien texte avait le même défaut ;
     la **25-3-c** (#454) le ferme en faisant passer `cancel` par le socle.

9. **Tests** (mutations vues rouges sur assertion) :
   - autorité refusée sur l'**écriture d'achat** par appel direct du socle (AC 1) ;
   - tête : facture `open` → `SUPPLIER_INVOICE_NOT_PAID` ; **double annulation** → la seconde rend
     ce code (le cas réel du rang 1) ;
   - queue : **renvoi** aux tests de la 25-3-a-1 (au niveau de `settlement_entry_cancel_blocker`),
     **plus** un cas par motif atteignable **par le chemin fournisseur** (exercice clos, compte
     archivé, exercice du jour absent — montage explicite : règlement dans un exercice ouvert qui
     ne couvre pas le jour, aucun exercice couvrant le jour), et la **paire compte archivé + exercice
     du jour absent** des deux côtés (lecture **et** clic rendent `ACCOUNT_ARCHIVED`) ;
   - `paid → open`, colonnes vidées, **deux cycles complets** (payer, annuler, payer, annuler) — la
     première écriture de règlement ressort en `ALREADY_REVERSED` et ne gêne pas le second cycle ;
   - facture réglée **par lot confirmé** : annulation, lot inchangé, `lastConfirmedBatch` renseigné,
     l'avertissement à l'écran ;
   - ⛔ **deux cycles sur `lastConfirmedBatch`** : lot confirmé → annulation → règlement **direct**
     → le champ désigne toujours l'ancien lot et le texte reste **vrai** (il ne dit pas « payée par
     ce lot ») ; puis deux lots confirmés successifs → le champ désigne le **plus récent** ;
   - **composition et rollback** (appel `_in_tx`, lecture **positive** dans la transaction,
     abandon, rien n'a changé) ;
   - étanchéité multi-tenant table par table (`journal_entries`, `journal_entry_lines`,
     `supplier_invoices`, `audit_log`) ;
   - API (200, 403 Consultation, 404, 409 avec `code`) ;
   - **vitest** — aucun test n'existe aujourd'hui pour cette fiche : fichier à créer (patron
     `contacts-page.test.ts`) : bouton selon `settlementCancellable` **et** le statut, motif
     seulement pour `paid`, état après `pay`, refus au clic affiché sans effacer la fiche,
     avertissement de lot ;
   - Playwright : payer puis annuler le règlement. Gardes i18n **recomptées**.

10. **Documentation.**
    - Manuel, « Régler une facture fournisseur » (`:1098-1108`) : le geste, ses refus ;
      **« Paiements par fichier pain.001 »** (`:1110-1124`) : le lot confirmé laissé tel quel, le
      rejet bancaire comme cas d'usage, et ⛔ **l'avertissement de double paiement** (AC 3) ;
    - ⛔ **`:1771-1774`** (écriture d'exercice clos « reste contre-passable ») : la 25-3-a-1 l'a
      corrigé pour le règlement **client** — **l'étendre au règlement fournisseur** (Q5 vaut des
      deux côtés) ;
    - PDF contrôlé **aplati** ;
    - **`README.md:40`** (« Factures fournisseurs & règlement ») : ajouter l'annulation du
      règlement ; relire `:41` (pain.001) ;
    - `api-external.md` : la route, clés API d'écriture admises ;
    - `CHANGELOG` `[0.12.1] — Non publié`, et le décompte « 94 actions » (déjà bougé par la
      25-3-a-1) **recompté** ;
    - ⛔ **C'est cette story qui ferme #414** : relancer
      `grep -rn "#414" frontend/src frontend/tests crates/` **après** le merge de la 25-3-a-1, et
      réécrire ce qui parle encore du côté fournisseur (`lib.rs:518`,
      `invoices/[id]/+page.svelte:1183` s'ils subsistent, `kesh-db/src/errors.rs:67-68`).
    - **Gate complet** (`kesh-db`).

## Tasks / Subtasks

- [ ] **T1 — Autorité fournisseur dans le socle** (AC 1).
- [ ] **T2 — Tête des motifs** (AC 2), `SupplierInvoiceNotPaid`.
- [ ] **T3 — Le geste** (AC 3), l'audit (AC 4), la route (AC 5).
- [ ] **T4 — Champs de lecture** (AC 6) et écran (AC 7).
- [ ] **T5 — Textes** (AC 8), dont `OWNED_BY_SUPPLIER_INVOICE`.
- [ ] **T6 — Tests** (AC 9), documentation et gates (AC 10), PR `closes #414`.

## Dev Notes

- **Anti-modèle** : `supplier_invoices::cancel` réécrit sa contre-passation à la main — ne pas
  l'imiter ; sa correction est la 25-3-c (#454).
- Le projet analytique est posé au niveau document puis **recopié par ligne** à l'insertion
  (`journal_entries.rs:426`) : la contre-passation du socle, qui reprend `project_id` par ligne, le
  **préserve**.
- L'export de souveraineté exportera les colonnes `settlement_*` à `NULL` ; le lien reste
  retrouvable par `audit_log` (exporté) et `reverses_entry_id`. `payment_batch_items` d'un lot
  confirmé continuera de désigner une facture `open` : conséquence voulue de l'arbitrage (AC 3).
- ⚠️ Cette fiche **réutilise** : si une revue y trouve un trou, vérifier d'abord que la 25-3-a-1 ne
  le couvre pas.

### References

- [Source: crates/kesh-db/src/repositories/supplier_invoices.rs:470-898] — garde de lot, `pay`, `pay_in_tx`, `cancel`.
- [Source: crates/kesh-db/src/repositories/payment_batches.rs:265-283,360-376] — création et confirmation de lot.
- [Source: crates/kesh-db/migrations/20260628000001_supplier_invoices.sql:65-73] — contraintes de statut.
- [Source: crates/kesh-api/src/routes/supplier_invoices.rs:62-90,295-331] — réponse, `pay`, `cancel`.
- [Source: crates/kesh-api/src/routes/invoices.rs:240-250,640] — discipline `Option` de `with_settlement`.
- [Source: frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte:75-109,146-147,215-284] — l'écran.
- [Source: frontend/scripts/lint-i18n-ownership.js:176] — préfixe d'un dossier à tiret.
- [Source: _bmad-output/implementation-artifacts/25-3-a-1-annuler-reglement-client.md] — socle étendu, queue commune.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Étape | Note |
|---|---|---|
| 2026-09-24 | validate P6 | **Une** lentille Haiku 4.5, passe complète, contexte frais (prompt `25-3-a-validate-prompt-p6-couple.md`, lentille B), axes déclarés (non exercés : exécution, PDF, clés FTL de la queue, recomptes — tous différés au dev). `lastConfirmedBatch` **vérifié** calculable et son texte vrai dans les cycles ; rangs 2, 4, 5 cohérents avec la sœur ; citations exactes. ❌ **1 MEDIUM réfuté** : « le texte d'`OWNED_BY_SUPPLIER_INVOICE` ne dit pas encore “ou son règlement” aux trois sites » reproche au **code** de ne pas encore porter ce que l'AC 8 prescrit — pas un défaut de la spec (le prompt le rappelait). ⇒ **0 au-dessus de LOW : boucle close.** Trend (mère puis fille) : P1 1 HIGH / 3 MED → P2 2 MED → P3 1 HIGH / ~9 MED (**découpage**) → P4 9 MED → P5 1 HIGH (**né de la remédiation P4**) → P6 **0**. Modèles : Sonnet, Haiku, Opus, Opus, Sonnet, Haiku. ⚠️ La remontée P4 → P5 (MEDIUM → HIGH) déclenchait formellement la règle de découpage ; non appliquée — le HIGH venait de la remédiation, non de la conception, et la fiche ne porte qu'un geste — **signalé à Guy**. |
| 2026-09-24 | validate P5 | **Une** lentille Sonnet, passe complète, contexte frais (prompt `25-3-a-validate-prompt-p5-couple.md`, lentille B), axes déclarés (non exercés : exécution, PDF — rien n'est encore régénéré —, clés FTL de la queue non encore créées, recomptes différés au dev). ⛔ **1 HIGH, né de la remédiation P4** : `settledByConfirmedBatchId` n'était pas lié au règlement **courant** — `payment_batch_items` ne porte aucun lien vers une écriture, et ses lignes survivent aux annulations ; après « lot confirmé → annulation → règlement direct », l'avertissement aurait affirmé « payée par le lot n° X » à tort. ⇒ champ **historique** `lastConfirmedBatch` (le plus récent, critère d'ordre écrit), texte **toujours vrai**, test à deux cycles ; colonne de corrélation écartée et **dite** écartée. LOW : rangs de la queue **numérotés comme la 25-3-a-1** (2, 4, 5 ; rang 3 absent, dit) ; citation du repli serveur ramenée au bras exact (`:2467-2470`) ; `payment_batches.rs:264` → `:265`. Symptôme grepé (`settledByConfirmedBatchId`) : quatre sites, tous repris. |
| 2026-09-24 | validate P4 | **Deux lentilles Opus** en contexte frais, prompt `25-3-a-2-validate-prompt-p4.md`, axes déclarés (non exercés : exécution, recompte des gardes i18n et du registre de routes, sens des traductions, écrans de détail d'un lot). **9 MEDIUM, 10 LOW, aucun HIGH**, recoupés. Fiche **réécrite**. ⛔ **Le constat de fond** : « étendus ou jumeaux, au choix du développeur » ne se choisissait pas — étendre le calcul client (qui prenait un `settlement_id`) mettait des cas morts dans les deux `switch`, le jumeler recréait une seconde précédence. ⇒ **tête propre + queue commune** sur l'écriture, **posée par la 25-3-a-1** (qui en est rouverte) ; code de tête **figé** (`SUPPLIER_INVOICE_NOT_PAID`) ; textes : queue partagée, tête en `supplier-invoices-…`. Autres MEDIUM corrigés : le contrôle de l'autorité vit **dans le socle** (dans le geste il était vrai par construction, donc intestable), contre-passation **avant** le vidage de la colonne ; paire « 1-2 » **impossible** remplacée par la double annulation, et la paire compte archivé / exercice du jour exigée ; ⛔ **risque de double paiement** après un lot confirmé, désormais **dit** (champ `settledByConfirmedBatchId`, avertissement à la confirmation, manuel pain.001) sans toucher à l'arbitrage ; écran : condition d'affichage du motif, masquage par rôle, confirmation, refus au clic **hors** `errorMsg` (qui efface la fiche) ; `README.md:40`. LOW : pourquoi pas de garde de lot ; « annuler ensuite (25-3-c) » inexact ; deux cycles complets ; `null` et non `skip_serializing_if` (patron `with_settlement`) ; majuscule du texte ; limite assumée du texte d'`OWNED_BY_SUPPLIER_INVOICE` pour une facture annulée (fermée par la 25-3-c) ; recompte du CHANGELOG ; contenu d'`api-external.md` ; grep `#414` après la sœur, puisque c'est **cette** story qui ferme l'issue ; renvoi au manuel pain.001 ; fichier vitest à créer. |
| 2026-09-24 | split | Née du découpage de la 25-3-a. Intègre les corrections de la passe 3 côté fournisseur : bouton conditionné à `settlementCancellable` et non au statut ; champs portés aussi par la réponse de `pay` (qui remplace l'état de l'écran) ; **cinq** appelants de `from_parts`, non quatre ; texte d'`OWNED_BY_SUPPLIER_INVOICE` qui couvre l'écriture d'achat **et** celle de règlement ; motif « non `paid` » **codé** ; test de rollback par composition. Ferme #414, **après** la 25-3-a-1. |
