# Story 25.4-c : Le résiduel au rapprochement bancaire

Status: ready-for-dev

**Issue : [#420]** — ⛔ la PR porte `closes #420`, titre ET corps (§ *Issue Tracking Rule*). Voisine :
**[#476]** (arrondi de la QR contre garde de trop-perçu), **incluse** (Q1) — la PR porte aussi `closes #476`.

**Mère : `25-4-propager-le-residuel.md`** (`split`) — source des faits. **Sœurs** : 25-4-a (mergée, #472 :
le reste dû juste), 25-4-b1 (mergée, #475 : formes jointes aux agrégats), 25-4-b2 (mergée, #479 : les
rappels). Branche `story/25-4-c-residuel-au-rapprochement`, rebasée sur `main` (`9098b2be`) ;
la b2 ne touche aucun fichier du rapprochement.

## Story

En tant que comptable,
je veux que le rapprochement bancaire reconnaisse le virement du **solde** d'une facture déjà
partiellement réglée,
afin de ne pas avoir à rapprocher à la main un paiement qui correspond exactement à ce que le client
devait encore.

## Le défaut, vérifié dans le code — pire que ce que dit l'issue

#420 annonce un **score nul**. C'est en amont que ça casse : la facture **n'est même pas candidate**.

| Site | Grandeur | Effet sur 1 000.— réglée de 400.—, virement de 600.— |
|---|---|---|
| Filtre des candidats — `find_unpaid_invoices_for_window`, `crates/kesh-db/src/repositories/reconciliation.rs:93-130` | `HAVING total_ttc BETWEEN ? - ? AND ? + ?` (`:115`), TTC par **sous-requête corrélée** (`:107`, `INVOICE_TTC_SUBQUERY_SQL`) | 1 000 ∉ 600 ± 0.05 : **exclue**, jamais scorée |
| Score — `propose_matches`, `crates/kesh-reconciliation/src/matching.rs:68` | `amount_score(tx.amount, *total_ttc)`, binaire (`:113`) | 0 si elle passait le filtre |
| Re-score à l'acceptation — `accept_one_invoice`, `crates/kesh-api/src/routes/reconciliation.rs:1196` | `invoices::total_ttc(&mut **tx, invoice.id)` | score 0 → `RECONCILIATION_SCORE_TOO_LOW` (`:1226-1235`), **avant** la garde de trop-perçu |
| Montant affiché dans la proposition — `reconciliation.rs:594` | `cand.total_ttc` (commentaire `:591-593` « = TTC ») | l'écran montre 1 000, pas 600 (`ReconciliationProposals.svelte:285`) |

Déjà justes : la garde de trop-perçu (`:1329-1344`, `amount_due`), le solde atteint (`:1444-1451`),
l'écriture (montant de la transaction, `:1387-1422`).

⚠️ Le défaut est né de la 24-2 (règlements multiples) — *la correction déplace le défaut vers ce
qu'elle vient d'écrire*. Aucun test ne pose un candidat sur une facture partiellement réglée ;
`accept_settles_an_invoice_in_two_payments` (`crates/kesh-api/tests/reconciliation_e2e.rs:2792`) passe
par l'API directe, grâce au score de **référence**, et n'asserte ni candidat ni score.

### Ce que l'inventaire a établi, et qui fixe les choix

1. **Le moteur de score est pur** : le montant arrive dans le triplet `(Invoice, Option<Contact>,
   Decimal)` (`matching.rs:63`). Passer au reste dû, c'est changer **ce que l'appelant y met** —
   `matching.rs` ne demande que nommage et doc (`:8-11`, `:58-60`). `propose_matches` a exactement deux
   appelants : `reconciliation.rs:575` (propositions) et `:1208` (re-score).
2. **Le filtre doit passer à la forme JOINTE** (`amount_due_derived_joins()` +
   `INVOICE_AMOUNT_DUE_DERIVED_SQL`, `invoice_settlements.rs:93-106`), celle que la 25-4-b1 a mise en
   service pour les listes : la forme corrélée y est interdite (`invoice_settlements.rs:32-34`, « N+1
   déguisé »). Le **re-score** porte sur UNE facture : forme scalaire (`amount_due`), dans la
   transaction.
3. **Les factures fournisseur ne sont pas concernées** : absentes du rapprochement ; elles se règlent
   en une fois (`supplier_invoices.rs:554-593`). Les règles, éclatements et rapprochements manuels ne
   portent aucune facture (`invoice_id: 0`, `reconciliation.rs:1965`, `:2374` ; `post_manual`
   `:2709-2710`).
4. **L'avoir** : une facture créditée est `cancelled` (`credit_notes.rs:586`, dépôt) et sort du filtre
   `status = 'validated'` ; le terme « avoir » du reste dû y vaut 0 en pratique — la forme jointe le
   porte de toute façon.
5. **L'arrondi (#476)** : `line_total` est à 4 décimales (`invoices.rs:425-427`), la TVA seule
   arrondie à 2 (`:165-176`) — le **reste dû peut avoir 4 décimales** (10.0050). La transaction bancaire
   est à 2 (`bank_imports.sql:62`). Conséquences :
   - le **score** ne peut jamais valoir 1 : 10.01 ≠ 10.005 après `normalize` (`matching.rs:113`) ;
   - les **gardes de trop-perçu** comparent la valeur brute : `reconciliation.rs:1336`,
     `invoice_settlements_write.rs:167` — le paiement exact de la QR (10.01) est refusé ;
   - les **tests de solde** aussi : `reconciliation.rs:1451` et `invoice_settlements_write.rs:233`
     (`due_after <= 0`) — un paiement de 10.00 laisse 0.0050, la facture reste « partiellement réglée »
     pour un demi-centime, sans `paid_at`.
   - l'**annulation d'un règlement** aussi : `invoice_settlements_write.rs:464-465`
     (`due_after > Decimal::ZERO` rouvre la facture). ⚠️ Branche **défensive** : l'application ne peut
     pas y produire un résidu non nul inférieur au centime — il faudrait qu'avant l'annulation la
     facture ait été **trop** payée, ce que la garde de trop-perçu refuse (le commentaire `:458-461`
     le dit). Elle suit le helper par cohérence, et se teste par une ligne de règlement **insérée
     directement en base** ;
   - le **dialogue de règlement** du frontend : `SettleInvoiceDialog.svelte:111` compare la saisie à
     `Number(amountDue)` **brut**, alors que le champ est pré-rempli arrondi (`:75`, `toFixed(2)`) —
     le 10.01 proposé par le dialogue lui-même y est refusé comme dépassant le reste (10.005).
   Le filtre, lui, tolère l'écart grâce aux ± 0.05.
   - **L'arrondi existe déjà, deux fois** : `Money::round_to_centimes()`
     (`crates/kesh-core/src/types/money.rs:61-69`, `MidpointAwayFromZero`) et `reminder_amount_due`
     (`crates/kesh-api/src/routes/invoice_pdf_service.rs:112-118`, la 25-4-b2 : arrondi du reste dû
     pour la QR du rappel, refus si ≤ 0 ; son test `:894` porte déjà le cas 10.0050 → 10.01).
6. **Aucun verrou sur la facture à l'acceptation** : `accept_one_invoice` lit la facture par
   `find_invoice_by_id_for_company` (`crates/kesh-db/src/repositories/reconciliation.rs:217-230`,
   `SELECT` simple) puis le reste dû (`:1329`) sans `FOR UPDATE`, alors que le règlement manuel
   (`invoice_settlements_write.rs:63-68`), son annulation (`:380`) et l'annulation d'un rapprochement
   (`reconciliation_cancel.rs:296`) verrouillent `invoices`. Deux acceptations concurrentes, ou une
   acceptation contre un règlement manuel, peuvent lire le même reste et passer toutes deux la garde
   de trop-perçu. Préexistant — mais cette story fait d'une facture partiellement réglée un candidat
   **ordinaire**, là où elle était exclue : la course devient atteignable. D'où l'AC 5-bis.
7. **`LIMIT 50` sans `ORDER BY`** (`reconciliation.rs:116`, dépôt) : au-delà de 50 candidats, lesquels
   sont gardés n'est pas déterminé. Préexistant, hors périmètre — à ne pas aggraver.

## Acceptance Criteria

### Volet 1 — le reste dû dans le rapprochement

**AC 1** — `find_unpaid_invoices_for_window` filtre sur le **reste dû** par la forme **jointe** : les
tables dérivées d'`amount_due_derived_joins()` et `INVOICE_AMOUNT_DUE_DERIVED_SQL`, jamais réécrites à
la main, ni la forme corrélée. Le candidat porte le reste dû (le champ est renommé — `amount_due` — et
tous ses lecteurs suivent) ; son doc-comment le dit.

**AC 2** — `propose_matches` reçoit le **reste dû** aux deux appels : propositions (`:575`) et re-score
à l'acceptation (`:1196-1208`, forme scalaire `invoice_settlements::amount_due` **dans la transaction**,
à la place d'`invoices::total_ttc`). `matching.rs` : noms et doc-comments disent « montant à régler »,
plus « TTC ».

**AC 3** — Le montant affiché dans la proposition (`invoice_amount`, `:594`) est le **reste dû** ; le
commentaire `:591-593` suit. Présentation *(Q3)* : le reste dû ; sur une facture déjà réglée en
partie, suivi de « reste dû sur <TTC> » (libellé dans les 4 locales), pour que le comptable
reconnaisse la facture. Sans règlement, rien n'est ajouté.

**AC 4** — Une facture **sans règlement** se comporte exactement comme avant : reste dû = TTC ;
candidats, scores et montant affiché inchangés. Les tests existants le tiennent
(`reconciliation_repository.rs:484-531`, `:807` ; `reconciliation_e2e.rs:819-822`), noms et messages
mis à jour là où ils disent « TTC ».

### Volet 2 — l'arrondi au centime *(Q1 : inclus)*

**AC 5** — Une seule grandeur, **le reste dû arrondi au centime** (`MidpointAwayFromZero`, la stratégie
de la QR — `generator.rs:38-39`), sert de montant à régler : filtre des candidats, score, garde de
trop-perçu du rapprochement (`:1336`) **et** du règlement manuel (`invoice_settlements_write.rs:167`),
tests de solde (`:1451` et `:233`), réouverture à l'annulation d'un règlement
(`invoice_settlements_write.rs:464-465`), et contrôle de saisie du dialogue de règlement
(`SettleInvoiceDialog.svelte:111`, comparé au reste arrondi et non à `Number(amountDue)`). Côté Rust,
un helper unique, à côté d'`amount_due`, la porte ; il **s'appuie sur `Money::round_to_centimes()`**
(`kesh-core`) sans recopier la stratégie, et **`reminder_amount_due` (b2) l'appelle** en gardant son
refus du reste nul — il ne reste qu'une définition du « reste dû au centime ». Aucun site ne
réarrondit à sa façon. ⛔ Le reste dû **stocké/calculé** n'est pas modifié : seule la comparaison
arrondit.

**AC 6** — Sur une facture dont le reste dû brut est 10.0050 : le virement de **10.01** est candidat,
score 1 sur le montant, s'accepte, et **solde** la facture (`paid_at` posé, audit `invoice.paid`) ; un
virement de **10.02** reste un trop-perçu refusé ; un règlement manuel de 10.01 solde aussi, **y
compris depuis le dialogue** (le 10.01 pré-rempli est accepté). L'annulation d'un règlement qui laisse
un reste brut de 0.0040 ne rouvre pas la facture — test de **dépôt**, règlements insérés directement
en base (état inatteignable par l'application, cf. inventaire § 5), et le test le dit.

**AC 5-bis** — `accept_one_invoice` **verrouille la facture** (`SELECT … FROM invoices WHERE id = ? AND
company_id = ? FOR UPDATE`, le patron de `invoice_settlements_write.rs:63-68`) avant de lire le reste dû
pour le re-score et la garde de trop-perçu : au **chargement** de la facture (étape 5,
`reconciliation.rs:1104-1106`), donc avant la garde de statut, le re-score, la garde de trop-perçu et
le verrou d'exercice (`fiscal_years::find_open_covering_date`, `:1350`, `FOR UPDATE`).
**Ordre des verrous, relevé dans le code** :

| Chemin | Ordre |
|---|---|
| `accept_one_invoice` (après patch) | `GET_LOCK` du compte bancaire (`with_account_lock`, tout le lot) → transaction bancaire lue **sans** verrou → **facture** → exercice → écritures, puis `UPDATE bank_transactions` |
| `settle_invoice` (`invoice_settlements_write.rs`) | **facture** (`:68`) → compte bancaire (`:116`) → compte (`:151`) → exercice (`:174`) |
| `cancel_reconciliation` (`reconciliation_cancel.rs`) | transaction bancaire (`:284`) → ligne de règlement (`:293`) → **facture** (`:296`) → écriture/exercice (`:309`) |

Facture avant exercice partout : conforme. L'annulation, elle, verrouille la transaction bancaire et
la ligne de règlement **avant** la facture ; une acceptation et une annulation concurrentes **sur la même
facture** (deux transactions bancaires distinctes) peuvent donc s'interbloquer. InnoDB le détecte et
annule **toute** la transaction perdante (erreur 1213) — pas seulement le point de sauvegarde de la
proposition : le lot entier échoue en `DATABASE_ERROR` et se rejoue. Risque accepté, préexistant en
nature (le verrou d'exercice l'ouvre déjà) ; le Dev Agent Record le consigne, sans l'aggraver. Test : deux acceptations concurrentes du même solde sur une facture partiellement réglée → une
acceptée, l'autre en `RECONCILIATION_OVERPAYMENT` (ou refusée par la garde de statut), jamais deux
règlements.

### Volet 3 — tests, textes

**AC 7** — Tests qui auraient échoué avant le patch, chacun sur une facture **partiellement réglée** à
TVA **non nulle** :
- dépôt : la facture 1 000 réglée 400 est **candidate** pour 600, et ne l'est plus pour 1 000 ;
- propositions (e2e API) : `amountScore == 1.0` et `invoiceAmount` = le reste ;
- acceptation (e2e API) : le virement du solde s'accepte et solde la facture — sans passer par le
  score de référence (numéro de facture absent de la transaction) ;
- le cas 10.0050 de l'AC 6, au rapprochement, au règlement manuel (API et Vitest du dialogue) et à
  l'annulation ;
- la concurrence de l'AC 5-bis ;
- Playwright : une facture réglée en partie apparaît dans les propositions avec son reste.

**AC 8** — Manuel : `user-manual.tex` § rapprochement dit que la proposition porte sur **ce qui reste
à payer** et qu'un solde de facture partiellement réglée est reconnu. ⚠️ **Le paragraphe du score
(`:1384-1390`) est faux sur le code, indépendamment de cette story** : il annonce un score gradué
(« écart < 1 CHF = score moyen »), un critère « Date », une « référence QR Bill », un seuil de 80 % et
un auto-accept à 95 % — rien de tel n'existe. Il est **réécrit dans cette story** sur le code réel *(Q2)* ; les deux autres
passages faux (lot dit « atomique », rapprochement manuel / éclatement dits « par facture »,
`:1411`, sous-section à partir de `:1421`) font l'objet d'une **issue séparée**, ouverte à l'implémentation. PDF régénéré, contrôlé aplati.
CHANGELOG `[0.12.1]` *Fixed*.

## Tasks / Subtasks

- [ ] **T1 — candidats** (AC 1, 4) : forme jointe, champ renommé, lecteurs.
- [ ] **T2 — score et re-score** (AC 2, 3) : les deux appelants, affichage, `matching.rs`.
- [ ] **T3 — arrondi** (AC 5, 6) : helper sur `Money::round_to_centimes()`, `reminder_amount_due`
  rebranché, sept sites (dont le dialogue frontend).
- [ ] **T3-bis — verrou** (AC 5-bis) : `FOR UPDATE` dans `accept_one_invoice`, ordre des verrous.
- [ ] **T4 — tests et mutations** (AC 7).
- [ ] **T5 — textes** (AC 8) ; si un champ TTC s'ajoute à la réponse des propositions (Q3), vérifier
  `docs/api-external.md` (aujourd'hui muet sur ses champs).
- [ ] **T6 — gates** : backend complet (base remise à zéro), frontend complet, **E2E complet**.

## Dev Notes

### Ce qu'il ne faut pas faire

- ⛔ **La forme corrélée dans le filtre des candidats** : c'est une liste (`LIMIT 50`).
- ⛔ **Réécrire la formule du reste dû** : `INVOICE_AMOUNT_DUE_DERIVED_SQL` et `amount_due` existent.
- ⛔ **Rendre le score gradué** : le binaire est une décision v0.1 assumée (`matching.rs:108-117`) ; cette
  story change la grandeur comparée, pas la forme du score.
- ⛔ **Toucher l'écriture** : elle porte le montant de la transaction, c'est juste.
- ⚠️ **L'ordre des gardes à l'acceptation** : le score (`:1226`) précède le trop-perçu (`:1336`). Un
  virement supérieur au reste doit continuer de sortir en `RECONCILIATION_OVERPAYMENT` s'il passe le
  score, en `SCORE_TOO_LOW` sinon — ne pas l'inverser sans le dire.
- ⚠️ `get_proposals_invoice_candidate_overrides_rule` (`reconciliation_rules_e2e.rs:962`) dépend du seuil
  0,5 (`INVOICE_OVERRIDE_THRESHOLD`, `:556`) : un score de montant qui change change la préséance.

### Où regarder

| Fichier | Pourquoi |
|---|---|
| `crates/kesh-db/src/repositories/reconciliation.rs:55-130` | candidats, fenêtre, tolérance |
| `crates/kesh-reconciliation/src/matching.rs:1-145` | score, triplet |
| `crates/kesh-api/src/routes/reconciliation.rs:480-640, 1056-1600` | propositions, acceptation, gardes |
| `crates/kesh-db/src/repositories/invoice_settlements.rs:27-220` | formes jointe et scalaire |
| `crates/kesh-db/src/repositories/invoice_settlements_write.rs:60-70, 160-260, 460-470` | verrou, règlement manuel, annulation |
| `crates/kesh-core/src/types/money.rs:61-69`, `crates/kesh-api/src/routes/invoice_pdf_service.rs:112-118` | arrondis existants |
| `frontend/src/lib/features/invoices/SettleInvoiceDialog.svelte:70-115` | contrôle de saisie |
| `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte:285`, `reconciliation.types.ts:30` | affichage |
| `docs/manual/fr/user-manual.tex:1375-1440` | rapprochement |

### Gardes-fous du dépôt

- Aucune migration. Repositories `kesh-db` touchés ⇒ **gate complet même en cours de boucle de revue**.
- Modules : `kesh-db`, `kesh-reconciliation`, `kesh-api`, `frontend`, `kesh-i18n` (le libellé « reste
  dû sur » de Q3) — **cinq**, sous le seuil de découpage (« plus de 5 »).

## Arbitrages

*Retenus le 2026-09-29 : Guy a demandé de continuer sans trancher ; ce sont les recommandations de
la création, appliquées par défaut et **révisables par lui**. La validation n'a pas à les contester,
seulement leur mise en œuvre.*

**Q1 — inclure #476 (l'arrondi) ?** Le reste dû peut porter des demi-centimes ; comparé brut, il
empêche le score de valoir 1, fait refuser le paiement exact de la QR comme trop-perçu, et laisse une
facture « partiellement réglée » pour 0.005. C'est la même comparaison que cette story touche, aux
mêmes sites, plus le règlement manuel. **Retenu : inclus** (volet 2) — sans lui, la 25-4-c
reconnaît le solde d'une facture à 2 décimales mais pas celui d'une facture dont le TTC en a 4.

**Q2 — le manuel du rapprochement.** Le paragraphe du score est faux sur le code (score gradué, date,
référence QR, seuils 80/95 % — inexistants), et deux autres passages aussi : l'acceptation par lot
dite « atomique » alors qu'elle est en succès partiel, et le rapprochement manuel / l'éclatement dits
« par facture » alors qu'ils n'en portent aucune (`:1411`, `:1421` et suivantes). **Retenu** : corriger
dans cette story le paragraphe du score (il décrit la grandeur que la story change) ; ouvrir une issue
pour les deux autres.

**Q3 — le montant affiché.** **Retenu** : le **reste dû** (« 600.00 ») ; sur une facture réglée en
partie, suivi de « reste dû sur 1 000.00 » pour que le comptable reconnaisse la facture.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-09-28** — Créée. Inventaire par exploration, citations clés vérifiées ; le défaut est
  l'**exclusion** par le filtre SQL, pas seulement un score nul ; un cinquième site d'arrondi trouvé
  (`invoice_settlements_write.rs:233`) ; trois questions à Guy.
- **2026-09-29** — Rebasée sur `main` après le merge de la b2 (#479). Q1–Q3 retenues selon les
  recommandations (Guy : « continue ») ; #476 incluse.
- **2026-09-29** — Validation P1 (Sonnet, prompt `25-4-c-validate-prompt-p1.md`) : **3 HIGH, 2 MEDIUM,
  1 LOW**, tous vérifiés sur le code avant patch. F1 HIGH : l'arrondi existait deux fois
  (`Money::round_to_centimes`, `reminder_amount_due`) → le helper s'appuie sur le premier, le second
  l'appelle. F2 HIGH : `SettleInvoiceDialog.svelte:111` refusait le 10.01 qu'il pré-remplit → ajouté à
  l'AC 5. F3 MEDIUM : réouverture à l'annulation (`invoice_settlements_write.rs:464`) → ajoutée.
  F4 MEDIUM : lignes du manuel décalées de 20 à 30 → corrigées. F5 HIGH : aucun verrou facture dans
  `accept_one_invoice`, course rendue atteignable par la story → AC 5-bis. F6 LOW : `api-external.md`
  → T5. Axes non exercés par la lentille : faisabilité du Playwright et tuabilité des mutations.
- **2026-09-29** — Validation P2 (Haiku, prompt `25-4-c-validate-prompt-p2.md`) : **0 finding, rendu
  sans aucune preuve** ; déclaré « constructible » le cas d'annulation à 0.0040 et « cohérent » l'ordre
  des verrous sans l'avoir établi. Repris par l'orchestrateur : **2 MEDIUM**. (a) Le cas d'annulation
  de l'AC 6 est **inatteignable** par l'application (il suppose un trop-perçu préalable) → test de
  dépôt par insertion directe, dit comme tel. (b) L'AC 5-bis laissait l'ordre des verrous au dev → relevé
  dans le code (`GET_LOCK` / facture / exercice ; annulation : transaction bancaire / règlement /
  facture), placement du verrou fixé à l'étape 5, interblocage acceptation-annulation écrit comme
  risque accepté.

[#416]: https://github.com/guycorbaz/kesh/issues/416
[#420]: https://github.com/guycorbaz/kesh/issues/420
[#476]: https://github.com/guycorbaz/kesh/issues/476
