# Story 25.4-b1 : Le résiduel aux agrégats — balance âgée et échéancier

Status: in-progress

**Issue : [#416]** — cette story en livre la partie **agrégats** ; la partie **rappels** est la
25-4-b2. ⛔ **La PR de b1 porte `refs #416`, pas `closes`** : l'issue ne se ferme qu'avec b2.

**Mère : `25-4-propager-le-residuel.md`** (`split`). **Sœur livrée : 25-4-a** (PR #472, #455 et #456) —
elle a rendu la formule juste (avoir compté TTC) et posé le test de parité des formes jointes ;
cette story en est le **premier appelant**. ⚠️ Branche **empilée** sur celle de 25-4-a : à rebaser
sur `main` après le merge de #472.

## Story

En tant que comptable,
je veux que la balance âgée et l'échéancier montrent ce que chaque client **doit encore**, et non
ce que la facture valait à l'émission,
afin de relancer les bons clients pour les bons montants, et que la balance âgée concorde avec le
compte débiteurs du grand livre.

## Le défaut, vérifié dans le code

Depuis la 24-2, une facture se règle en plusieurs fois ; le reste dû se **calcule** (24-2, D3). Les
écrans **unitaires** le savent (fiche facture : badge « partiellement payée », « Reste dû »). Les
écrans **agrégés**, non — ils totalisent le **TTC** :

| Site | Ce qu'il somme |
|---|---|
| Balance âgée — `crates/kesh-report/src/aged_receivables.rs:111-131` (`generate`) | `COALESCE(lt.ttc, 0)` dans les cinq tranches et le total |
| Totaux de l'échéancier — `crates/kesh-db/src/repositories/invoices.rs:882-900` (`due_dates_summary`) | `unpaid_total`, `overdue_total` en `lt.ttc` ; le commentaire `:890-892` les dit « montants dus » — faux depuis la 24-2 |
| Lignes de l'échéancier — `list_by_company_paginated` (`:815-821`) | `total_ttc` seul ; **aucun** reste dû par ligne |
| Export CSV de l'échéancier — `list_for_export` (`:2456-2461`), `routes/invoices.rs:1463-1465` | colonne « Total » = TTC, commentée « montant dû » ; statut sans « partiel » (`:1443-1449`) |
| Page échéancier — `frontend/src/routes/(app)/invoices/due-dates/+page.svelte` | colonne Total = `totalTtc` (`:453`), `statusOf` sans `partial` (`:306-314`), dialogue `amountDue={null}` (`:493-507`) |

Une facture de 1 000.— réglée à 900.— pèse **1 000.—** partout, au lieu de 100.—. ⚠️ La balance âgée
est un **instrument de décision** (qui relancer, pour combien) et doit **concorder avec le compte
débiteurs** du grand livre, juste depuis la 24-2 : les deux divergent aujourd'hui exactement du
montant des règlements partiels, et **aucun test ne compare** les deux (vérifié : aucun test de
`kesh-report` ne lit un solde de compte).

⚠️ **Les formes jointes du résiduel n'ont aucun appelant** (`INVOICE_SETTLED_DERIVED_JOIN_SQL`,
`INVOICE_CREDITED_DERIVED_JOIN_SQL`) ; 25-4-a en a prouvé la parité avec les formes scalaires. La
24-2 (D3) a prescrit la **jointure dérivée** pour les listes et agrégats : la forme corrélée y
serait réévaluée par ligne — un N+1 déguisé.

## Acceptance Criteria

### Volet 1 — la grandeur, une seule fois

**AC 1** — Une constante (ou une paire) canonique porte le **reste dû sous forme jointe** : les trois
tables dérivées (`lt` TTC, `cnt` avoir, `st` réglé) et l'expression
`COALESCE(lt.ttc, 0) − COALESCE(cnt.credited, 0) − COALESCE(st.settled, 0)`. Elle vit à côté
d'`amount_due` (`invoice_settlements.rs`), avec un doc-comment qui la dit **miroir** de la forme
scalaire. ⛔ Aucun site de cette story ne réécrit l'expression à la main.

**AC 2** — Le **test de parité** de 25-4-a (`tests/invoice_amount_due_parity.rs`) est étendu : pour
chaque facture du jeu, l'expression jointe de l'AC 1 = `amount_due` scalaire.

### Volet 2 — la balance âgée

**AC 3** — Les cinq tranches et le total de `aged_receivables::generate` somment le **reste dû**
(AC 1), non le TTC. Les tranches restent assises sur `i.due_date` : ⛔ **un règlement partiel ne
rajeunit pas la créance restante**.

**AC 4** — Le périmètre des factures ne change pas : `status = 'validated' AND paid_at IS NULL`
(une facture soldée a `paid_at`, 24-2 D4). Le `HAVING total <> 0` reste, et porte désormais sur le
reste dû. ⚠️ Un reste dû **négatif** (trop-perçu hérité) n'est **pas écrêté** : il apparaît, en
négatif — le doc-comment d'`amount_due` l'exige.

**AC 5** — ⛔ **Invariant de concordance** (test) : sur une société dont le compte débiteurs n'est mû
que par des factures, des règlements et des avoirs — factures sans règlement, partiellement réglée,
soldée, créditée —, le **total de la balance âgée = solde du compte 1100 au grand livre**
(`general_ledger` ou somme `debit − credit`). C'est l'assertion que #416 réclame et qui n'existe
pour aucun écran.

⚠️ **Portée de l'invariant — énoncée par une RÈGLE, pas par une liste.** *(Réécrite en validation
P3 : trois passes ont chacune trouvé des chemins que la liste précédente omettait — quatre en P3.
Une énumération de formes est ouverte par nature ; `CLAUDE.md` § « Inventorier les sites NON
RÉSOLUS ».)*

**La règle** : l'égalité vaut **si et seulement si** le compte débiteurs n'est mouvementé **que** par
les écritures de **vente**, de **règlement client** et d'**avoir** que Kesh passe pour des factures
**validées**, toutes **datées au plus tard à la date d'arrêté** (`as_of`), et **n'a jamais changé**
dans les réglages de facturation. Les **contre-passations** de ces mouvements (annulation d'un
règlement, d'un rapprochement de facture) restent **dans** la règle : elles inversent exactement
les mêmes comptes et montants, pendant que le reste dû remonte d'autant. Tout autre mouvement l'en
écarte. *Exemples*, non exhaustifs, tous
vérifiés atteignables : solde d'ouverture ; écriture saisie au journal ; rapprochement bancaire hors
facture — manuel, ventilé, par règle (`post_manual`, `post_split`, `accept_one_split`,
`accept_one_rule`) ; règlement d'une facture **fournisseur** par compte interne imputé au compte
débiteurs (compensation) ; changement du compte débiteurs par défaut (et l'avoir qui, depuis, crédite
le nouveau compte — **défaut produit, #473**) ; règlement client imputé au compte débiteurs lui-même
(**#474**) ; **données antérieures à la 0.12.1** — factures marquées payées sans écriture par
l'ancien `mark_as_paid` (avant 0.12.0), facture annulée par un avoir après règlement partiel
(**0.12.0 publiée**, `git show v0.12.0:…/credit_notes.rs:302`) — qu'elles viennent d'une
**restauration** ou d'une **mise à jour sur place**.

Le test de l'AC 5 se place **dans** la règle : les quatre états **vivants**, et un `as_of`
postérieur ou égal à toutes les dates de pièces. ⛔ Il n'y ajoute aucun cas hors règle. Le
doc-comment de `generate` énonce la règle, pas la liste des exemples.

**AC 6** — Doc-comment de `generate` (`:95-99` : « Montants = TTC dérivé ») et libellés : la balance
âgée montre le **reste dû** ; l'écran (`AgedReceivablesView.svelte`) et son CSV gardent leur
structure — seules les valeurs changent. Si un libellé visible dit « TTC » ou « total facturé »,
il devient « reste dû » (4 locales).

### Volet 3 — l'échéancier

**AC 7** — `due_dates_summary` : `unpaid_total` et `overdue_total` somment le **reste dû** ; le
commentaire `:890-892` dit vrai.

**AC 8** — `InvoiceListItem` porte `amount_settled` et `amount_due` (`Decimal`), projetés par **les
deux SELECT** qui le désérialisent — `list_by_company_paginated` **et** `list_for_export` (⚠️ le
doc-comment `:263-269` le rappelle : un oubli échoue au **runtime**). Calculés par la **forme
jointe** (AC 1) ; `total_ttc` reste. ⛔ Pas de N+1 : le nombre de requêtes d'une page ne dépend pas
du nombre de lignes (invariant 7 de la 24-2).

**AC 9** — `InvoiceListItemResponse` (`routes/invoices.rs:333-364`) expose `amountSettled` et
`amountDue` (camelCase, chaîne décimale). ⚠️ Même discipline que la fiche
(`routes/invoices.rs:242-248`) : **toujours calculés** dans une liste, jamais `None` par défaut. Le
type frontend (`invoices.types.ts`) suit.

**AC 10** — Page échéancier :
- une colonne **« Reste dû »** (`amountDue`) à côté du Total TTC, qui reste ;
- le statut de ligne rend **`partial`** quand `paidAt` est nul et `amountSettled > 0` — même règle
  que la fiche (`invoices/[id]/+page.svelte:424-438`), idéalement **la même fonction** partagée ;
- le résumé montre les totaux de l'AC 7 (les libellés disent « reste dû » s'ils disaient autre
  chose) ;
- le dialogue de règlement reçoit **`amountDue={inv.amountDue}`** : pré-rempli, et sa garde client
  de trop-perçu (`SettleInvoiceDialog.svelte:108-112`) redevient active depuis l'échéancier. Le
  commentaire `:493-499` qui explique pourquoi il ne l'était pas est retiré.

**AC 11** — Export CSV de l'échéancier : une colonne **« Reste dû »** après « Total » (clé d'en-tête
×4, `CSV_HEADER_KEYS` / `CSV_HEADER_FALLBACKS`) ; le statut rend « partiellement payée »
(`payment-status-partial`, clé existante) ; le commentaire `:1463-1464` « TTC (montant dû) » est
corrigé.

### Volet 4 — tests, textes

**AC 12** — Tests, au minimum :

| Test | Prouve |
|---|---|
| parité étendue (AC 2) | forme jointe du reste dû = `amount_due` |
| `aged_uses_amount_due_not_ttc` | facture 8,1 % de 1 081.— réglée 900.— : pèse 181.— dans sa tranche |
| `aged_partial_payment_keeps_original_bucket` | la tranche suit `due_date`, pas la date du règlement |
| `aged_total_matches_receivable_ledger` | AC 5 |
| `due_dates_summary_totals_are_amount_due` | AC 7 |
| `list_items_carry_amount_due` (repo **et** e2e HTTP) | AC 8-9, les deux SELECT ; la valeur traverse la frontière |
| export CSV : colonne reste dû, statut partiel | AC 11 |
| Vitest échéancier | colonne, statut `partial`, dialogue pré-rempli |
| **E2E** `invoices_echeancier.spec.ts` | le cas de la 24-3 (`:147-158`) cesse de **saisir** le montant : il vérifie qu'il est **pré-rempli** au reste dû, sur une facture partiellement réglée |

⛔ **Chaque facture de test porte une TVA non nulle** (leçon de 25-4-a). ⛔ Mutations à exécuter et
consigner : TTC remis dans une tranche de la balance âgée — **et TTC remis dans le seul total** (doit
faire rougir `aged_total_matches_receivable_ledger`) ; `amount_due` retiré de `list_for_export`
seul ; `partial` retiré de `statusOf` ; `amountDue={null}` remis.

**AC 13** — Textes (`docs/manual/fr/user-manual.tex`, PDF **contrôlé aplati**), § Balance âgée
(`\label{sec:balance-agee}`) — **quatre** passages, tous au TTC aujourd'hui :
- `:1576` « encours débiteur (le total dû, TVA comprise) » → le **reste dû**, après règlements
  partiels et avoirs ;
- `:1578`, légende de la capture : « encours débiteur **TTC** » → « reste dû » ;
- `:1580-1581` : « La colonne « Non échu » **garantit** que le total général réconcilie avec le
  solde du compte clients » — vrai **seulement** dans la règle de l'AC 5. La phrase devient, par la
  règle et non par une liste : *« Le total général réconcilie avec le solde du compte clients tant
  que ce compte n'est mouvementé que par les factures, leurs règlements et leurs avoirs (y compris
  leurs annulations). Toute autre écriture qui le touche l'en écarte : solde de départ, écriture
  saisie au journal, virement bancaire affecté à ce compte sans passer par une facture, paiement
  d'une facture fournisseur compensé sur ce compte, ou règlement dont la contrepartie est ce compte
  lui-même. De même un changement du compte clients dans les réglages de facturation, ou des données
  antérieures à la version 0.12.1, restaurées ou mises à jour. »* ;
- `:1587-1588`, note : « le **total dû TTC** de chaque facture, jamais le montant hors taxe » → « le
  **reste dû** de chaque facture, jamais le montant hors taxe ni le total facturé ».

⚠️ Greper le symptôme (`TTC`, `total dû`) dans **toute** la section et dans le PDF aplati, pas
seulement ces lignes. Puis : § Échéancier (`:927`) décrit la colonne et le pré-remplissage.
CHANGELOG `[0.12.1]` *Fixed*. README : rien, sauf si la ligne v0.12.1 cite la balance âgée.

## Tasks / Subtasks

- [x] **T0 — rectifier la portée de l'état hérité posée par 25-4-a** : **cinq** sites disent
      l'état « avoir après règlement partiel » atteignable **seulement par l'import d'une
      sauvegarde** — quatre commentaires, `errors.rs:221`, `invoice_settlements_write.rs:333`,
      `tests/invoice_amount_due_parity.rs:139`, `tests/invoice_settlement.rs:980`, et **leur source**,
      la fiche `25-4-a-residuel-juste.md` (AC 13, `:180`), rectifiée par une note datée plutôt que
      réécrite (story livrée). Faux : la **0.12.0
      publiée** accepte cet avoir, une installation **mise à jour sur place** le porte. Écrire « des
      données antérieures à la 0.12.1, restaurées ou mises à jour ». *(Le manuel de 25-4-a dit
      déjà « avant la version 0.12.1, ou restaurée » : juste.)*
- [x] **T1 — la grandeur jointe** (AC 1-2).
- [x] **T2 — balance âgée** (AC 3-6), dont l'invariant de concordance.
- [x] **T3 — échéancier, backend** (AC 7-9, AC 11).
- [x] **T4 — échéancier, frontend** (AC 10), i18n.
- [x] **T5 — tests et mutations** (AC 12).
- [x] **T6 — textes** (AC 13).
- [ ] **T7 — gates** : backend complet (base remise à zéro), frontend complet, **E2E complet**.

## Dev Notes

### Ce qu'il ne faut pas faire

- ⛔ **Ne pas stocker le reste dû** (24-2, D3).
- ⛔ **Ne pas utiliser `amount_due` scalaire dans une boucle** sur les lignes d'une liste : N+1.
- ⛔ **Ne pas retirer `total_ttc`** de la liste : l'échéancier affiche les deux, et d'autres écrans le
  lisent.
- ⚠️ **Double calcul du TTC dans les listes** : `total_ttc` y vient de la forme **corrélée**
  (`invoices.rs:819`, `:2459`), et la forme jointe de l'AC 1 recalcule `lt.ttc`. Il est **permis** —
  et préférable — de dériver `total_ttc` de `lt.ttc` dans ces deux SELECT : `invoice_ttc_parity.rs`
  prouve déjà l'égalité des deux formes.
- ⚠️ **Pré-remplissage à reste dû ≤ 0** : `SettleInvoiceDialog` affiche alors une erreur client dès
  l'ouverture (`:104-112`). Comportement **déjà présent** sur la fiche
  (`invoices/[id]/+page.svelte:1241`) ; un reste dû ≤ 0 sur une facture `validated` sans `paid_at`
  n'existe que par un état hérité. Ne pas le corriger ici ; ne pas l'aggraver.
- **Montage E2E** du cas « partiellement réglée » : régler une partie par l'API
  (`authedApiContext(page)`, patron `invoices-settlement-cancel.spec.ts:46`), puis ouvrir le dialogue
  depuis l'échéancier.
- ⚠️ **Le tri** « TotalAmount » de l'échéancier trie sur `i.total_amount` (HT) alors que la colonne
  affiche le TTC — défaut **préexistant**, hors périmètre ; ne pas l'aggraver, le signaler.

### Où regarder

| Fichier | Rôle |
|---|---|
| `crates/kesh-db/src/repositories/invoice_settlements.rs` | constantes jointes, `amount_due` |
| `crates/kesh-report/src/aged_receivables.rs:95-175` | requête, doc |
| `crates/kesh-report/tests/aged_receivables.rs` | tests existants (leur facture « payée » est montée par `UPDATE paid_at`) |
| `crates/kesh-db/src/repositories/invoices.rs:246-275, 800-830, 882-930, 2436-2470` | `InvoiceListItem`, liste, résumé, export |
| `crates/kesh-api/src/routes/invoices.rs:333-364, 1364-1470` | DTO de liste, export CSV |
| `frontend/src/routes/(app)/invoices/due-dates/+page.svelte` | page |
| `frontend/src/routes/(app)/invoices/[id]/+page.svelte:424-438` | `paymentStatus()` de la fiche, à partager |
| `frontend/src/lib/features/invoices/SettleInvoiceDialog.svelte:35-112` | pré-remplissage, garde client |
| `frontend/tests/e2e/invoices_echeancier.spec.ts:100-160, 200-201` | cas de la 24-3 ; « la colonne Total affiche le TTC (montant dû) » |
| `crates/kesh-report/src/general_ledger.rs` | solde d'un compte, pour l'AC 5 |

### Gardes-fous du dépôt

- Aucune migration.
- Repositories touchés : **gate complet même en cours de boucle** (§ Exception `kesh-db`).
- i18n : les nouvelles clés et tout `i18nMsg(` ajouté bougent `sitesTotal` — recompter.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5.

### Debug Log References

- La mutation « reste dû retiré du seul export » a d'abord **échoué à compiler** (argument nommé
  `due` devenu inutilisé dans le `format!`) : mutation invalide, refaite en gardant l'argument
  (`({due}) * 0 + COALESCE(lt.ttc, 0)`) — elle est alors tuée.
- Le reste dû arrive de l'API à l'échelle du calcul SQL (« 68.1000 ») ; `SettleInvoiceDialog`
  le recopiait tel quel dans le champ montant — **aussi sur la fiche**, depuis la 24-2. Le
  pré-remplissage passe désormais par `Big(amountDue).toFixed(2)`.
- ⚠️ **Relevé, non corrigé** : dans `invoice_echeancier_e2e.rs`, trois tests préexistants
  déstructurent `seed_base` en `(company_id, admin_id)` alors qu'il rend `(admin_id, company_id)`
  — ils passent parce que les deux identifiants valent 1 dans une base neuve. Les tests de 25-4-a,
  qui avaient copié ce patron, sont rectifiés ici.

### Completion Notes List

- **T0** : les quatre commentaires de 25-4-a qui bornaient l'état hérité à l'import disent
  désormais « données antérieures à la 0.12.1, restaurées ou mises à jour sur place » ; la fiche
  de 25-4-a porte sa note datée (validation P4).
- **La grandeur jointe** (AC 1-2) : `amount_due_derived_joins()` (les trois tables dérivées ; une
  fonction, `concat!` ne sachant pas assembler des constantes), `INVOICE_AMOUNT_DUE_DERIVED_SQL`,
  `INVOICE_AMOUNT_SETTLED_DERIVED_SQL` ; la parité de 25-4-a étendue — reste dû joint = scalaire,
  facture par facture, reste dû négatif compris.
- **Balance âgée** (AC 3-6) : les cinq tranches et le total somment le reste dû, tranches sur
  `due_date` ; doc-comments (module et `generate`) énoncent la **règle** de concordance et
  l'absence de filtre des règlements par `as_of`.
- **Échéancier, serveur** (AC 7-9, 11) : résumé au reste dû ; `InvoiceListItem` porte
  `amount_settled` / `amount_due`, projetés par **les deux** SELECT (liste et export) ; `total_ttc`
  y vient désormais de `lt.ttc` (permis par les Dev Notes) ; DTO camelCase ; export CSV : colonne
  « Reste dû » (clé ×4) et statut « Partiellement payée ».
- **Échéancier, écran** (AC 10) : `paymentStatusOf` partagé par la fiche et l'échéancier ;
  colonne « Reste dû » (clé ×4, `sitesTotal` 1750 → 1751 recompté) ; dialogue pré-rempli, garde
  client de trop-perçu rendue active ; commentaires qui disaient « TTC (montant dû) » corrigés.
- **Textes** (AC 13) : les **quatre** passages de la § Balance âgée (dont la réserve, par la
  règle) et la § Échéancier ; PDF régénéré et **contrôlé aplati** (0 `??`, six phrases de contrôle
  présentes, trois formules « TTC » absentes) ; CHANGELOG *Fixed*. Grep du symptôme sur le dépôt :
  seule la ligne v0.7.0 publiée du README, légitime.

**Tests ajoutés** (recomptés, branche de 25-4-a → arbre de travail) : `invoice_amount_due_parity`
4 → 6, `kesh-report/tests/aged_receivables` 4 → 7, `invoice_echeancier_e2e` 13 → 15, Vitest
`invoice-helpers.test.ts` 12 → 16, `due-dates-page.test.ts` 0 → 4 (nouveau). E2E
`invoices_echeancier.spec.ts` : le parcours de la 24-3 **vérifie** le montant pré-rempli au lieu
de le saisir, et un cas « réglée en partie » est ajouté.

**Mutations** (observées, fichiers restaurés) :

| Mutation | Tests rouges |
|---|---|
| TTC dans une tranche de la balance âgée | `aged_uses_amount_due_not_ttc` |
| TTC dans le seul total | `aged_total_matches_receivable_ledger`, `aged_uses_amount_due_not_ttc` |
| reste dû retiré du seul export | `list_items_carry_amount_due` |
| TTC dans le résumé de l'échéancier | `due_dates_summary_totals_are_amount_due` |
| `partial` retiré de `paymentStatusOf` | 2 Vitest (helper, page) |
| `amountDue={null}` remis | Vitest « dialogue pré-rempli » |

### Gates

- Backend : base remise à zéro, `scripts/test-fast.sh` **2491 / 2491** (2484 + 7).
- Frontend : `check` 0 erreur (27 avertissements), `lint-i18n-ownership` PASS, `test:unit`
  **834 / 834** (826 + 8), build OK.
- E2E : EN_ATTENTE

### File List

| Fichier | Nature |
|---|---|
| `crates/kesh-db/src/repositories/invoice_settlements.rs` | grandeur jointe |
| `crates/kesh-db/src/repositories/invoices.rs` | résumé, `InvoiceListItem`, deux SELECT |
| `crates/kesh-report/src/aged_receivables.rs` | requête, doc |
| `crates/kesh-api/src/routes/invoices.rs` | DTO, export CSV |
| `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` | `echeancier-csv-header-amount-due`, `due-dates-column-amount-due` |
| `crates/kesh-db/src/errors.rs`, `invoice_settlements_write.rs` | T0 |
| `crates/kesh-db/tests/invoice_amount_due_parity.rs` | parité étendue, résumé, listes ; T0 |
| `crates/kesh-db/tests/invoice_settlement.rs` | T0 |
| `crates/kesh-report/tests/aged_receivables.rs` | 3 tests |
| `crates/kesh-api/tests/invoice_echeancier_e2e.rs` | 2 e2e ; ordre de `seed_base` rectifié (tests 25-4-a) |
| `frontend/src/lib/features/invoices/invoice-helpers.ts` / `.test.ts` | `paymentStatusOf` |
| `frontend/src/lib/features/invoices/invoices.types.ts` | `amountSettled`, `amountDue` |
| `frontend/src/lib/features/invoices/SettleInvoiceDialog.svelte` | pré-remplissage au centime |
| `frontend/src/routes/(app)/invoices/[id]/+page.svelte` | statut partagé |
| `frontend/src/routes/(app)/invoices/due-dates/+page.svelte` / `due-dates-page.test.ts` | colonne, statut, dialogue |
| `frontend/src/lib/shared/i18n-keys.test.ts` | `sitesTotal` 1751 |
| `frontend/tests/e2e/invoices_echeancier.spec.ts` | pré-remplissage, cas partiel |
| `docs/manual/fr/user-manual.tex` / `.pdf` | balance âgée, échéancier |
| `CHANGELOG.md` | *Fixed* |

## Change Log

- **2026-09-27** — Signal MED→MED arbitré par Guy : **pas de découpage** (« continue ainsi »). Dev lancé.
- **2026-09-27** — **validation P5 ciblée** (Haiku, diff `5480b4f8..5824c048`, prompt
  `25-4-b1-validate-prompt-p5.md`) — **0 finding**, preuves recopiées (grep des cinq sites de T0,
  garde de la 0.12.0) et **concordantes avec les greps de l'orchestrateur**. ⛔ **Boucle close en 5
  passes** : `2M/3L → 0 (+1M orchestrateur) → 3M/3L → 1M/3L → 0`, rotation Sonnet → Haiku → Opus →
  Sonnet → Haiku, trois passes ciblées ; toutes les corrections sur la spec, aucune sur du code.
  ⚠️ **Signal de découpage à remonter à Guy** : quatre passes consécutives au niveau MEDIUM
  (§ *Règle de splitting préventif*, critère de sévérité). Toutes portaient sur **une seule
  phrase** — la portée de l'invariant de l'AC 5 —, close par le passage d'une liste à une règle.
- **2026-09-27** — **validation P4 ciblée** (Sonnet, prompt `25-4-b1-validate-prompt-p4.md`) — **1
  MEDIUM, 3 LOW**. MEDIUM : un **cinquième** site de la formulation fausse — la fiche de 25-4-a
  elle-même (AC 13, `:180`), source des quatre commentaires ; ajouté à T0, rectifiée par une note
  datée. LOW : la règle ne disait pas que les contre-passations (annulation de règlement ou de
  rapprochement) restent dans sa portée — ajouté ; mutation dédiée au test de l'AC 5 ; réserve du
  manuel reformulée pour ne plus pouvoir se lire comme visant les règlements ordinaires. Éprouvée
  contre la dévalidation, l'annulation de règlement et de rapprochement, la facture soldée et le
  reste dû négatif : **la règle tient**.
- **2026-09-27** — **validation P3 ciblée** (Opus, prompt `25-4-b1-validate-prompt-p3.md`, inventaire
  de **tous** les chemins qui écrivent une ligne d'écriture) — **3 MEDIUM, 3 LOW** : **quatre chemins
  oubliés** par la portée de l'AC 5 — règlement fournisseur par compte interne imputé au compte
  débiteurs ; changement du compte débiteurs par défaut, que l'avoir suit alors que la vente ne le
  suit pas ; données héritées plus larges (`mark_as_paid` sans écriture) et **pas seulement par
  import** (la 0.12.0 publiée accepte l'avoir après règlement partiel) ; règlement client imputé au
  compte débiteurs lui-même. LOW : deux handlers de rapprochement non cités ; `as_of` non borné.
  ⛔ **Troisième passe consécutive à trouver des cas hors d'une liste** : la portée est **réécrite
  par une règle**, avec des exemples non exhaustifs ; le test se place dans la règle, `as_of`
  postérieur à toutes les pièces ; la réserve du manuel suit la même forme. **Deux défauts produit
  sortis en issues** : #473 (l'avoir crédite le compte débiteurs des réglages, pas celui de la
  vente), #474 (règlement client sur le compte débiteurs lui-même). **T0 ajoutée** : quatre
  commentaires de 25-4-a bornaient à tort l'état hérité à l'import.
- **2026-09-27** — **validation P2** (Haiku, prompt `25-4-b1-validate-prompt-p2.md`) — rapporte **0
  finding**. ⚠️ **Non pris pour argent comptant** : la passe affirmait le CHANGELOG « à créer » (il
  existe) et n'a cité aucun des fichiers qu'on lui demandait de lire pour éprouver la portée de
  l'AC 5. **Repris par l'orchestrateur** : la portée oubliait un chemin — la **ventilation** et la
  **règle** de rapprochement bancaire (`accept_one_split`, `accept_one_rule`), dont la contrepartie
  est un compte libre, donc possiblement le compte débiteurs, sans facture. **1 MEDIUM** ajouté,
  corrigé (portée à quatre cas, réserve du manuel étendue). Vérifié aussi : le filtre « payées »
  de l'échéancier existe (`due-dates/+page.svelte:45`) — ses lignes montreront un reste dû nul,
  cohérent.
- **2026-09-27** — **validation P1** (Sonnet, prompt `25-4-b1-validate-prompt-p1.md`) — **2 MEDIUM,
  3 LOW**, confirmés dans le code et le manuel. MEDIUM : l'invariant balance âgée = grand livre
  (AC 5) était énoncé sans portée — l'état hérité de 25-4-a (facture `cancelled` créditée après
  règlement) et les soldes d'ouverture le rompent ; portée écrite, test borné aux états vivants, et
  le manuel, qui **garantit** la concordance sans réserve, recevra la réserve. MEDIUM : l'AC 13 ne
  citait qu'un des quatre passages « TTC » de la section Balance âgée — les trois autres ajoutés.
  LOW (Dev Notes) : double calcul du TTC dans les listes (dériver `total_ttc` de `lt.ttc` permis) ;
  pré-remplissage à reste dû ≤ 0 (préexistant, hors périmètre) ; montage E2E du cas partiel.
  L'inventaire des sites au TTC, refait depuis le symptôme, n'a trouvé **aucun site orphelin**
  (b1, b2, 25-4-c ou légitimement TTC).
- **2026-09-27** — Créée (Opus 5.5) après découpage de la 25-4-b (arbitrages Q1 ⇒ six modules).
  Sites revérifiés dans le code ; 25-4-a en est le socle.

[#416]: https://github.com/guycorbaz/kesh/issues/416
