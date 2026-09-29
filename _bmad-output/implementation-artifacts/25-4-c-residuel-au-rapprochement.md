# Story 25.4-c : Le résiduel au rapprochement bancaire

Status: ready-for-dev

**Issue : [#420]** — ⛔ la PR porte `closes #420`, titre ET corps (§ *Issue Tracking Rule*). Voisine :
**[#476]** (arrondi au centime) et **[#480]** (verrou à l'acceptation) — **hors périmètre**, sorties
au découpage du 2026-09-29 vers les sœurs **25-4-c3** et **25-4-c2**. La PR ne ferme que #420.

**Mère : `25-4-propager-le-residuel.md`** (`split`) — source des faits. **Sœurs** : 25-4-a (mergée, #472 :
le reste dû juste), 25-4-b1 (mergée, #475 : formes jointes aux agrégats), 25-4-b2 (mergée, #479 : les
rappels) ; **25-4-c2** (#480, verrou et instantané à l'acceptation) et **25-4-c3** (#476, l'arrondi
au centime), nées du découpage de cette story. Branche `story/25-4-c-residuel-au-rapprochement`, rebasée sur `main` (`9098b2be`) ;
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
5. **L'arrondi — hors périmètre (25-4-c3, #476).** `line_total` est à 4 décimales
   (`invoices.rs:425-427`) : le reste dû peut en porter 4 (10.0050), la transaction bancaire 2
   (`bank_imports.sql:62`). Ce que cette story en laisse, **sans l'aggraver** :
   - le **filtre** reste juste sur le reste **brut** : la tolérance ± 0.05 couvre l'écart
     |brut − arrondi| ≤ 0.005 ; aucun `ROUND` n'y est ajouté (ce serait une seconde définition de
     l'arrondi, que la c3 doit poser une seule fois) ;
   - le **score** d'une facture à reste de 4 décimales ne vaut pas 1 (10.01 ≠ 10.005 après `normalize`,
     `matching.rs:113`) — **comme aujourd'hui avec le TTC** ;
   - les gardes de trop-perçu, les tests de solde, le dialogue de règlement comparent toujours le brut.
   Tout cela est la c3 — y compris l'écart d'un demi-centime qu'un paiement arrondi laisserait au grand
   livre (lignes en `DECIMAL(19,4)`), question comptable ouverte à Guy.
6. **Le verrou à l'acceptation — hors périmètre (25-4-c2, #480).** `accept_one_invoice` ne verrouille
   pas la facture. ⛔ **Cette story n'y ajoute aucun `FOR UPDATE`** : sous REPEATABLE READ, un verrou
   posé après la première lecture de la transaction lirait `version` à jour et `amount_due` périmé, et
   désarmerait le contrôle optimiste (`reconciliation.rs:1498-1506`) qui refuse aujourd'hui une seconde
   acceptation concurrente. Ce qui reste exposé après cette story : une acceptation contre un règlement
   manuel **partiel** simultané (`settle_invoice` n'incrémente `version` qu'au solde,
   `invoice_settlements_write.rs:233-236`). Préexistant — atteignable aujourd'hui par le score de
   référence — et rendu plus fréquent par cette story ; la c2 le ferme, et **doit suivre**.
7. **`LIMIT 50` sans `ORDER BY`** (`reconciliation.rs:116`, dépôt) : au-delà de 50 candidats, lesquels
   sont gardés n'est pas déterminé. Préexistant, hors périmètre — à ne pas aggraver.

## Acceptance Criteria

### Volet 1 — le reste dû dans le rapprochement

**AC 1** — `find_unpaid_invoices_for_window` filtre sur le **reste dû** par la forme **jointe** : les
tables dérivées d'`amount_due_derived_joins()` et `INVOICE_AMOUNT_DUE_DERIVED_SQL`, jamais réécrites à
la main, ni la forme corrélée — sur le reste **brut**, tolérance inchangée (inventaire § 5). Le
candidat porte **le reste dû (`amount_due`) et le TTC (`total_ttc`)** : le premier pour le filtre, le
score et l'affichage, le second pour la mention de l'AC 3 ; tous les lecteurs de l'actuel `total_ttc`
passent au reste dû, sauf cette mention. Les doc-comments le disent.

**AC 2** — `propose_matches` reçoit le **reste dû** aux deux appels : propositions (`:575`) et re-score
à l'acceptation (`:1196-1208`, forme scalaire `invoice_settlements::amount_due` **dans la transaction**,
à la place d'`invoices::total_ttc`). `matching.rs` : noms et doc-comments disent « montant à régler »,
plus « TTC ».

**AC 3** — Le montant affiché dans la proposition (`invoice_amount`, `:594`) est le **reste dû**, au
même format qu'aujourd'hui (`normalize().to_string()`) ; le commentaire `:591-593` suit. Présentation
*(Q3)* : sur une facture déjà réglée en partie (reste ≠ TTC), suivi de « reste dû sur <TTC> » —
champ TTC ajouté à la réponse, libellé dans les 4 locales. Sans règlement, rien n'est ajouté.

**AC 4** — Une facture **sans règlement** se comporte exactement comme avant : reste dû = TTC ;
candidats, scores et montant affiché inchangés. Les tests existants le tiennent
(`reconciliation_repository.rs:484-531`, `:807` ; `reconciliation_e2e.rs:819-822`), noms et messages
mis à jour là où ils disent « TTC ».

### Volet 2 — tests, textes

**AC 5** — Tests qui auraient échoué avant le patch, chacun sur une facture **partiellement réglée** à
TVA **non nulle** :
- dépôt : la facture 1 000 réglée 400 est **candidate** pour 600, et ne l'est plus pour 1 000 ;
- propositions (e2e API) : `amountScore == 1.0` et `invoiceAmount` = le reste ;
- acceptation (e2e API) : le virement du solde s'accepte et solde la facture — sans passer par le
  score de référence (numéro de facture absent de la transaction) **ni par le score de contact**
  (contrepartie absente ou différente du contact : sinon le total vaut 0.10 et l'acceptation passe
  déjà sur le code actuel, `:1224`) ; la réponse asserte `amountScore == 1.0` ;
- Playwright : une facture réglée en partie apparaît dans les propositions avec son reste.

**AC 6** — Manuel : `user-manual.tex` § rapprochement dit que la proposition porte sur **ce qui reste
à payer** et qu'un solde de facture partiellement réglée est reconnu. Deux passages voisins suivent :
`:1011` promet déjà que la QR du rappel « permet au rapprochement bancaire de reconnaître le
paiement » — vrai **par** cette story, à garder cohérent ; `:1014` dit le versement « reste dû + frais »
« refusé comme trop-perçu au rapprochement » — en réalité il n'est **pas proposé** (hors tolérance), et
n'est refusé en trop-perçu que s'il porte la référence de la facture : à préciser. ⚠️ **Le paragraphe du score
(`:1384-1390`) est faux sur le code, indépendamment de cette story** : il annonce un score gradué
(« écart < 1 CHF = score moyen »), un critère « Date », une « référence QR Bill », un seuil de 80 % et
un auto-accept à 95 % — rien de tel n'existe. Il est **réécrit dans cette story** sur le code réel *(Q2)* ; les deux autres
passages faux (lot dit « atomique », rapprochement manuel / éclatement dits « par facture »,
`:1411`, sous-section à partir de `:1421`) font l'objet d'une **issue séparée**, ouverte à l'implémentation. PDF régénéré, contrôlé aplati.
CHANGELOG `[0.12.1]` *Fixed*.

## Tasks / Subtasks

- [ ] **T1 — candidats** (AC 1, 4) : forme jointe, `amount_due` + `total_ttc` au candidat, lecteurs.
- [ ] **T2 — score et re-score** (AC 2, 3) : les deux appelants, affichage et mention, `matching.rs`.
- [ ] **T3 — tests et mutations** (AC 5).
- [ ] **T4 — textes** (AC 6) ; le champ TTC ajouté à la réponse : vérifier `docs/api-external.md`
  (aujourd'hui muet sur ses champs) ; ouvrir l'issue des deux passages faux du manuel (Q2).
- [ ] **T5 — gates** : backend complet (base remise à zéro), frontend complet, **E2E complet**.

## Dev Notes

### Ce qu'il ne faut pas faire

- ⛔ **La forme corrélée dans le filtre des candidats** : c'est une liste (`LIMIT 50`).
- ⛔ **Réécrire la formule du reste dû** : `INVOICE_AMOUNT_DUE_DERIVED_SQL` et `amount_due` existent.
- ⛔ **Rendre le score gradué** : le binaire est une décision v0.1 assumée (`matching.rs:108-117`) ; cette
  story change la grandeur comparée, pas la forme du score.
- ⛔ **Toucher l'écriture** : elle porte le montant de la transaction, c'est juste.
- ⛔ **Ajouter un `FOR UPDATE` dans `accept_one_invoice`** (inventaire § 6) : c'est la c2, et fait à
  moitié il désarme le contrôle optimiste.
- ⛔ **Arrondir le reste dû** où que ce soit, filtre compris (inventaire § 5) : c'est la c3.
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
| `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte:285`, `reconciliation.types.ts:30` | affichage |
| `docs/manual/fr/user-manual.tex:1005-1016, 1375-1440` | rappel (QR), rapprochement |

### Montage du Playwright (AC 5)

Aucune spec ne montre aujourd'hui une facture candidate (`reconciliation.spec.ts:9-13` le déclare hors
périmètre). Recette : `createAndValidateInvoiceViaApi(page, contactId, '2026-05-10')`
(`frontend/tests/e2e/helpers/api-fixtures.ts:100`), règlement partiel par
`POST /api/v1/invoices/{id}/settlements`, puis import d'un CAMT réécrit comme dans
`reconciliation-cancel.spec.ts:87-107` (date de comptabilisation 2026-05-15, montant unique égal au
reste, sans référence de facture).

### Gardes-fous du dépôt

- Aucune migration. Repositories `kesh-db` touchés ⇒ **gate complet même en cours de boucle de revue**.
- Modules : `kesh-db`, `kesh-reconciliation`, `kesh-api`, `frontend`, `kesh-i18n` (le libellé « reste
  dû sur » de Q3) — **cinq**, sous le seuil de découpage (« plus de 5 »).
- Née d'un **découpage** (règle de splitting, non-convergence P2 → P3) : la validation reprend sur ce
  périmètre réduit.

## Arbitrages

*Retenus le 2026-09-29 : Guy a demandé de continuer sans trancher (le découpage, lui, a son accord
explicite) ; ce sont les recommandations de
la création, appliquées par défaut et **révisables par lui**. La validation n'a pas à les contester,
seulement leur mise en œuvre.*

**Q1 — inclure #476 (l'arrondi) ?** Retenu d'abord par défaut, puis **retiré au découpage du
2026-09-29** (accord de Guy) : la validation P3 a montré qu'arrondir la comparaison laisse un
demi-centime au grand livre — une question comptable, pas une comparaison. → **25-4-c3**.

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
- **2026-09-29** — Validation P3 (Opus, prompt `25-4-c-validate-prompt-p3.md`) : **1 CRITICAL, 3 HIGH,
  3 MEDIUM, 5 LOW** — **non appliqués, en attente d'arbitrage**. Le CRITICAL et deux HIGH portent sur
  l'AC 5-bis écrit par la remédiation P2 : sous REPEATABLE READ, un `FOR UPDATE` posé après la première
  lecture laisse `amount_due` lire l'instantané périmé tout en rafraîchissant `version` — il **désarme**
  le verrou optimiste qui refuse aujourd'hui la seconde acceptation (F1, vérifié : `settle_invoice` ne
  bumpe `version` qu'au solde, `invoice_settlements_write.rs:233-236` ; l'acceptation toujours,
  `reconciliation.rs:1498-1501`) ; le test de concurrence ne discriminait rien (F2) ; l'interblocage
  décrit était le mauvais et son issue est un 500 sans rejeu (F3). F4 HIGH : payer 10.01 une facture de
  10.0050 laisse la créance **créditrice** de 0.0050 au grand livre (lignes en `DECIMAL(19,4)`) et
  affiche « Reste dû −0.01 ». **Règle de découpage déclenchée** : sévérité P3 > P2 (F7).
- **2026-09-29** — **Découpée** (accord de Guy) : l'arrondi (ex-volet 2, #476) part en **25-4-c3**, le
  verrou (ex-AC 5-bis, #480 ouverte) en **25-4-c2** ; F1–F4, F7, F8, F10, F12 de P3 les suivent. Restent
  appliqués ici : F5 (filtre sur le brut, candidat portant reste dû **et** TTC), F6 (test d'acceptation
  hors score de contact), F9 (manuel `:1011`, `:1014`), F11 (recette Playwright). AC renumérotées :
  ex-AC 7 → AC 5, ex-AC 8 → AC 6.

[#416]: https://github.com/guycorbaz/kesh/issues/416
[#420]: https://github.com/guycorbaz/kesh/issues/420
[#476]: https://github.com/guycorbaz/kesh/issues/476
[#480]: https://github.com/guycorbaz/kesh/issues/480
