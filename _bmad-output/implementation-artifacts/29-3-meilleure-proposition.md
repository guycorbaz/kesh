# Story 29.3 : La proposition affichée suit une priorité écrite, et l'écran la dit

## Status

**ready-for-dev** — fiche créée le 2026-10-14 en autonomie (`bmad-create-story`, Opus 5.5), sur `16eeed21`
(kickoff de l'Epic 29), branche `story/29-3`, worktree `/home/gcorbaz/devel/kesh-29-3`. **Validation P1 remédiée**
le 2026-10-14 (rapports R 0/1/5/7, F 0/0/5/7 ; règle des rangs refondue par décision de l'orchestrateur,
C-29-3-11) ; validation P2 à lancer (`29-3-validate-prompt-p2.md`, Opus, lentilles R et F). Choix consignés au
registre propre à la story, `epic-29-choix-autonomes-29-3.md`, **C-29-3-1 à C-29-3-15** (à verser par union au
registre de l'epic). 9 AC. PR : **`closes #527`** — dans le titre ou le corps de la PR **seulement**, jamais dans un
message de commit (B4 de la rétrospective 28).

⚠️ **Sérialisée derrière la 29-2** (#529), qui touche les mêmes fichiers (§ *Zones partagées avec la 29-2*). Le
développement **commence par un rebase** sur `origin/main` une fois la 29-2 mergée, et **relit** tous les numéros de
ligne de cette fiche (relevés sur `16eeed21`).

## Story

**As a** utilisateur qui rapproche ses relevés (Comptable ou Admin),
**I want** que la proposition affichée pour une transaction soit choisie selon une priorité écrite, que l'écran me
dit, et qui ne fasse jamais passer une règle d'affectation devant une facture que sa référence, le nom de son client
ou son montant exact rapproche du paiement,
**so that** « Accepter » ne porte pas en produit le paiement d'une facture — produit compté deux fois, facture
restée due, sans que rien le signale —, et que je sache pourquoi une règle valable n'est pas affichée.

Issue : [#527](https://github.com/guycorbaz/kesh/issues/527) (P2, bug ; relevée par la lentille R de la validation P2
de la 15-5c). Epic : `_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` § *29-3*. Cible : **v0.13.4**.
#527 laisse deux issues : « triées par score, toutes origines confondues — **ou** une règle de priorité écrite et dite
à l'écran ». La story retient la seconde (C-29-3-1, refondu par C-29-3-11).

### Le défaut, au sol (`16eeed21`)

`get_proposals` (`crates/kesh-api/src/routes/reconciliation.rs:586`) construit, par transaction :

1. les candidates **factures**, triées par `propose_matches` (`crates/kesh-reconciliation/src/matching.rs:62-107`) en
   `score.total DESC, invoice_id ASC` (`:752-790` de la route : `match_proposals.into_iter().filter_map(…)`) ;
2. **si aucune** facture n'atteint `INVOICE_OVERRIDE_THRESHOLD = 0.5` (`:725`, `:748-750`), la **première règle**
   qui correspond (`first_matching_rule`, `:796`), **ajoutée en fin de liste** (`candidates.push`, `:806`), avec un
   score **conventionnel** de 1,0 (`:818-823`) — aucun retri ensuite (`:827`).

Le frontend n'affiche et n'accepte que `candidates[0]`
(`frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte:208` pour l'acceptation, `:338-368` pour
l'affichage). **Prémisse de #527 confirmée** : dès qu'une facture a un score strictement positif mais < 0,5, elle
passe devant la règle, et « Accepter » la règle.

**Ce qui est faux, après examen (C-29-3-11).** Non pas que la facture passe devant — c'est le bon ordre, voir
*La règle retenue* —, mais que **rien ne l'écrit ni ne le dit** : l'ordre est un effet de bord de la place du
`push` ; le commentaire de la passe 4 (`:722-724`) affirme que « rule + invoice coexistent (les 2 s'affichent) »,
ce qui est faux à l'écran, qui n'en montre qu'une ; et l'utilisateur qui a posé une règle ne voit ni la règle ni la
raison pour laquelle elle est absente.

**Quelles factures sont « faibles », exactement.** Le dépôt ne charge que les factures dont le reste dû égale le
montant de la transaction **à 5 centimes près** (`AMOUNT_TOLERANCE_HUNDREDTHS = 5`, `:72` ;
`find_unpaid_invoices_for_window`, `crates/kesh-db/src/repositories/reconciliation.rs:234-274`, `HAVING`). Le score
de montant est binaire (`matching.rs:109-119`) : 1 si égal au centime. Le score d'une facture est
`0,50·montant + 0,40·référence + 0,10·contact`, la référence valant 1 (la référence de la transaction et le numéro de
la facture se contiennent, **dans un sens ou dans l'autre**, sans longueur minimale, `matching.rs:132-134`), 0,5
(préfixe commun ≥ 4 caractères) ou 0, le contact 1 (le nom de la contrepartie et celui du client se contiennent,
dans un sens ou dans l'autre, `:148-161`) ou 0. **Une facture de total nul n'est pas candidate** (`total > 0.0`,
`matching.rs:81`). Les seuls totaux possibles **sous 0,5** sont donc, montant **différent de 1 à 5 centimes** :

| référence | contact | total | la règle est-elle calculée ? |
|---|---|---|---|
| 1 | 1 | **0,5** (exact en `f64` : `0,0 + 0,4 + 0,1`) | non — seuil atteint |
| 1 | 0 | **0,4** | oui |
| 0,5 (préfixe commun) | 1 | 0,3 | oui |
| 0,5 | 0 | 0,2 | oui |
| 0 | 1 | 0,1 | oui |
| 0 | 0 | 0 | — la facture n'est pas candidate |

**Conséquence, établie au code** : quand la règle figure dans la liste, **chaque** facture candidate porte un
sous-score de référence ou de contact non nul (montant différent ⇒ `amount_score = 0` ⇒ `total > 0` exige l'un des
deux). Il n'existe aujourd'hui **aucune** facture candidate « reconnue par le seul montant à quelques centimes ».

### Prémisse du manuel : corrigée

#527 dit que le manuel (« vers :1511 ») annonce « la meilleure » candidate. **Ce n'est plus vrai** : le texte a été
réécrit par #481/#519 (CHANGELOG, v0.12.x, l. 248) ; `grep -n meilleure docs/manual/fr/user-manual.tex` ne rend
rien. Le manuel décrit aujourd'hui l'ordre **fidèlement** (`user-manual.tex:2049` : « une règle n'est affichée […]
que s'il n'y a aucune facture candidate --- même une facture de score faible passe devant elle ») — et cet ordre est
**conservé** (C-29-3-11) : le manuel reste vrai ; il lui manque la **raison**, et le rappel que l'écran dit la règle.
*(À signaler sur l'issue par l'orchestrateur.)*

### La règle retenue (C-29-3-1, refondu par C-29-3-11)

**Une priorité écrite, en trois rangs, dont le critère est le signal d'identité, non le score total.**

| rang | candidate | pourquoi |
|---|---|---|
| **0** | facture qui porte un **signal** : montant égal au centime (`amount_score == 1`), **ou** référence liée au numéro (`reference_score > 0` : 1 ou 0,5), **ou** contrepartie liée au nom du client (`contact_score > 0`) — quel que soit son score total | un indice qui rattache le paiement à **cette** facture ; le porter en produit par une règle serait le faux rattachement muet |
| **1** | la règle d'affectation (une au plus) | un critère posé par l'utilisateur, qui ne sait rien des factures ouvertes |
| **2** | facture **sans aucun signal** — reconnue par le seul montant à quelques centimes | aucun indice : la règle passe devant. **Inatteignable aujourd'hui** (une telle facture a un total nul et n'est pas candidate, `matching.rs:81`) ; le rang existe pour qu'une source future de factures (la 29-2, un choix manuel, un poids changé) ne puisse pas placer en silence une facture sans indice devant une règle |

À rang égal, l'ordre de `propose_matches` est **conservé** (tri **stable** : score décroissant, puis `invoice_id`
croissant).

**Pourquoi le signal d'identité, et non le score** (décision de l'orchestrateur, C-29-3-11) : la règle du
`CLAUDE.md` *Un appariement automatique propose, il ne crée jamais* — le faux rattachement **muet** est le pire cas.
Accepter une règle pour le paiement d'un client porte ce paiement en produit : **produit compté deux fois, facture
restée due, rappel envoyé à un client qui a payé** (`user-manual.tex:2911` met déjà en garde contre ce mode
d'échec). Accepter la mauvaise facture, à l'inverse, se voit au lettrage et se défait. L'argument qui fondait le rang
0 de la première rédaction (numéro dans la référence) vaut pour **toute** facture identifiée — y compris par le
seul nom du client (0,1) ou par un préfixe de référence (0,2, 0,3).

**Pourquoi pas le tri brut par score** (l'option 1 de #527) : le 1,0 de la règle est une **convention**, pas une
mesure — le comparer aux scores des factures compare deux grandeurs sans commune mesure ; et il ferait passer la
règle devant **toutes** les factures faibles, c'est-à-dire devant des paiements de clients identifiés.

**Effet net, pour l'utilisateur** : l'ordre affiché est **le même qu'avant** la story sur tout ce qui est atteignable
(C-29-3-11, conséquence démontrée ci-dessus) ; ce qui change, c'est qu'il est **écrit** (une fonction pure, testée),
**dit** à l'écran (AC5) et **documenté** pour les clients d'API (AC8). La règle n'est toujours pas calculée quand une
facture atteint 50 % (seuil inchangé, C-29-3-5).

⚠️ **Risque résiduel, écrit** : le faux rattachement **symétrique** — une contrepartie qui est à la fois cliente et
visée par une règle (un remboursement d'un fournisseur qui est aussi client), dont le montant tombe à 5 centimes près
du reste dû d'une de ses factures — fait proposer la facture plutôt que la règle. C'est l'ordre d'aujourd'hui ; il se
voit au lettrage. Non traité (C-29-3-11).

⚠️ **Angle mort écrit (C-29-3-7) — une facture candidate peut être proposée puis refusée.** Une transaction qui
**dépasse** le reste dû de 1 à 5 centimes est chargée par la tolérance du dépôt, mais l'acceptation la refuse en
`RECONCILIATION_OVERPAYMENT` (`classify_payment`, `crates/kesh-db/src/repositories/invoice_settlements.rs:175-184` :
`paid > raw_due` → trop-perçu, hors arrondi au centime). Une telle facture porte un signal (sinon elle n'est pas
candidate), reste au rang 0 et passe devant une règle valable — **ni aggravé ni réparé** par la story (elle passait
déjà devant). C'est la famille de #548 sur le **montant** : **[#665](https://github.com/guycorbaz/kesh/issues/665)**
(ouverte, P2). D'autres refus d'acceptation d'une facture proposée existent, tous de **configuration** (compte
bancaire lié au compte débiteurs, compte d'arrondi manquant ou égal au compte débiteurs, exercice clos) : déjà
documentés au manuel (`user-manual.tex:2056`), ni aggravés ni réparés.

⚠️ **Angle mort écrit (C-29-3-12) — le départage à score égal.** À rang et score égaux, `propose_matches` place la
facture de plus petit `invoice_id` en tête (`matching.rs:97-105`) — la plus ancienne. La 29-2 (C-29-2-7, fiche
voisine non mergée) fait entrer des factures échues et relancées parmi les candidates : une facture ancienne relancée
passera devant une récente du même client et du même montant. Un départage par **proximité de la date d'ancrage** à
la date du paiement est proposé ; il dépend de l'ancre que définit la 29-2 et touche `kesh-reconciliation`, hors
périmètre : **non mécanique, non intégré, issue demandée** (rapport à l'orchestrateur).

## Critères d'acceptation

**AC1 — Une fonction pure ordonne les candidates (C-29-3-2, C-29-3-3, C-29-3-8, C-29-3-11).**

Dans `crates/kesh-api/src/routes/reconciliation.rs`, à côté des types de réponse (`CandidateType` `:119`,
`ReconciliationCandidate` `:126-151`) :

- `fn candidate_rank(c: &ReconciliationCandidate) -> u8` — rend 0, 1 ou 2 selon le tableau ci-dessus. Un
  **`match` exhaustif** sur `c.candidate_type` (**aucun bras `_`**) : un variant ajouté plus tard (la 29-2 peut en
  introduire un) ne compile pas sans être classé. Le rang 0 d'une facture se lit sur les **sous-scores**
  `s.amount_score == 1.0 || s.reference_score > 0.0 || s.contact_score > 0.0` — valeurs littérales `1.0`, `0.5`,
  `0.0` posées par `matching.rs`, donc comparables exactement ; **jamais** sur un seuil de `score.total` (C-29-3-3).
- `fn order_candidates(candidates: &mut [ReconciliationCandidate])` — `candidates.sort_by_key(candidate_rank)`,
  **stable** (`sort_by_key` l'est ; ne pas employer `sort_unstable_by_key`). Elle ne suppose **rien** de l'ordre
  d'entrée (AC2 cas 2, 3 et 6).
- Doc-comments : la règle et le pourquoi de chaque rang (le faux rattachement muet, le refus du tri brut par score),
  le fait que le rang 2 est **inatteignable aujourd'hui** et pourquoi il existe, la mention que l'écran affiche et
  accepte `candidates[0]`, et les angles morts de C-29-3-7 (**#665**) et C-29-3-12 (avec le numéro de l'issue que
  l'orchestrateur ouvrira ; à défaut au développement, « départage à score égal, cf. fiche 29-3 »).

`get_proposals` appelle `order_candidates(&mut candidates)` **après** l'ajout éventuel de la règle (`:806-826`) et
avant la construction de `ReconciliationProposal` (`:827`) — **dernier** geste sur `candidates`. Le commentaire
« AC #114 (Pass 2 Q8) » (`:722-724`) est réécrit : il dit la priorité, et que l'écran n'affiche qu'une candidate (il
affirme aujourd'hui « les 2 s'affichent »). Rien d'autre ne change dans la route : ni le seuil, ni la forme JSON, ni
les filtres.

⚠️ **Observabilité, écrite d'avance** : sur les entrées atteignables, `order_candidates` ne change rien (la règle est
poussée en dernier et chaque facture qui la côtoie est de rang 0). Son effet n'est observable que par les tests
unitaires de l'AC2, qui lui présentent des entrées que `get_proposals` ne produit pas (règle en tête, facture sans
signal), et par les mutations du **critère** de rang (M4, M5, M7), que les tests HTTP tuent. C'est voulu
(C-29-3-11) et c'est ce que disent les mutations de l'AC9.

**AC2 — Tests unitaires de l'ordre (module `#[cfg(test)] mod candidate_order_tests`, même fichier).**

Constructeurs locaux d'une candidate facture (`invoice_id`, sous-scores donnés, total = somme pondérée) et d'une
candidate règle (score 1,0). Notation : `f<id>(m, r, c)` = facture d'identifiant `id`, sous-scores montant, référence,
contact ; `R` = règle. **Entrée** et **sortie** écrites en dur pour chaque cas, jamais relues de la production :

1. entrée `[f1(0,0,1), R]` (l'ordre de production, le cas de #527) → `[f1, R]` ;
2. entrée `[R, f1(0,0,1)]` (règle **en tête**) → `[f1, R]` ;
3. entrée `[R, f1(0,1,0), f2(0,0.5,0)]` → `[f1, f2, R]` (ordre d'entrée conservé dans le rang 0) ;
4. entrée `[f1(0,0.5,1), f2(0,0.5,0), R]` → inchangé ;
5. entrée `[f1(0,0,0), R]` (facture **sans signal**, fabriquée : inatteignable par la route) → `[R, f1]` ;
6. entrée `[R, f3(0,0,1), f4(0,0,0)]` → `[f3, R, f4]` (les trois rangs) ;
7. factures seules, montant exact : entrée `[f2(1,1,0), f1(1,0,0)]` → inchangé (stabilité) ;
8. deux factures de rang 0 à score égal, entrée `[f5(0,0,1), f3(0,0,1)]` → inchangé (le tri ne réordonne pas à
   l'intérieur d'un rang ; c'est `propose_matches` qui a fixé `invoice_id ASC`) ;
9. `[R]` → `[R]` ; `[]` → `[]` ;
10. `candidate_rank` en direct : `(1,0,0)` → 0 ; `(0,1,0)` → 0 ; `(0,0.5,0)` → 0 ; `(0,0,1)` → 0 ; `(0,0,0)` → 2 ;
    règle → 1. *(La branche « montant exact » n'est observable que là : une facture à montant exact atteint 0,5 et
    supprime la règle — F11 de la P1.)*
11. **stabilité sur une taille réaliste** (M6) : entrée de 51 éléments — `R` en tête, puis 49 factures de rang 0
    `f49(0,0,1)` … `f1(0,0,1)` (identifiants **décroissants**), puis `f50(0,0,0)` — → les 49 factures dans l'ordre
    d'entrée (`49, 48, …, 1`), puis `R`, puis `f50`. La sortie attendue se construit par une expression
    indépendante de la production (`(1..=49).rev()`), écrite dans le test. 51 = le plafond de 50 factures du dépôt
    (`LIMIT 50`, `kesh-db` `reconciliation.rs:258`) plus la règle.

**AC3 — Tests d'intégration HTTP (`crates/kesh-api/tests/reconciliation_e2e.rs`, `#[sqlx::test(migrations =
"../kesh-db/test-schema")]`).**

Le fichier a les aides de factures (`seed_validated_invoice`, `:384`), de transactions (`seed_bank_transactions`,
`:512`, `make_new_tx` `:482` : `end_to_end_id` et `transaction_id` nuls, référence passée en `&str` — `""` pour
« sans référence », écartée par `coalesce`), de comptes (`setup_company` `:570` : contact « `<label>` Client », comptes
1100, 1020, 3000 ; `account_ids` `:441` rend le 3000 Ventes, imputable, actif, distinct du 1020 de la banque). Il n'a
**aucune** aide de règle : la tâche T1 y ajoute `create_rule_ok(app, jwt, body)` (POST `/api/v1/reconciliation/rules`,
201 attendu — patron `reconciliation_rules_e2e.rs:356-370`, une dizaine de lignes ; les aides de ces fichiers sont
privées à chacun, comme `spawn_app`). Transaction créditrice CHF, facture et transaction **du même jour** en plein
exercice 2026 (C-29-3-10). Les deux cas sont **verts au sol et après** : ils figent l'ordre (aucun test HTTP ne peut
être rouge avant, l'ordre atteignable étant inchangé, C-29-3-11) et tuent M4, M5 et M7.

- `proposals_put_an_invoice_named_by_the_counterparty_before_the_rule` : `setup_company(…, "Dubois", …)` (contact
  « Dubois Client ») ; facture `F-2026-0291` de 100.00 ; transaction **99.99**, contrepartie « Dubois Client »,
  **sans** référence ; règle `{"label": "Dubois", "matchType": "counterparty_contains", "matchValue": "Dubois",
  "counterpartyAccountId": <3000>}`. Attendu : deux candidates, `candidates[0]` la facture (`candidateType ==
  "invoice"`, score total 0,1, sous-score de contact 1), `candidates[1]` la règle. Puis `POST /accept` de
  `candidates[0]` → `accepted` d'une facture ; reste dû **0.01** (règlement partiel, `classify_payment` →
  `Ordinary`, comme `accept_settles_an_invoice_in_two_payments`, `:3417`).
- `proposals_keep_a_referenced_invoice_before_the_rule` : `setup_company(…, "Martin", …)` (contact « Martin
  Client ») ; facture `F-2026-0292` de 100.00 ; transaction **99.99**, contrepartie « Garage Bel-Air », référence
  `F-2026-0292` ; règle `counterparty_contains` « Garage » vers le 3000. Contact : « garage bel-air » et « martin
  client » ne se contiennent pas → 0 ; total **0,4**. Attendu : `candidates[0]` la facture (sous-score de référence
  1, de contact 0), `candidates[1]` la règle.
- **Le test muet** `get_proposals_invoice_candidate_overrides_rule` (`reconciliation_rules_e2e.rs:961-1004`) **ne
  crée aucune facture** (« Pas d'invoice créée », `:990`) et assert la **présence** de la règle : il n'éprouve pas ce
  que son nom annonce (C-29-3-6). **Il est déplacé** dans `reconciliation_e2e.rs` (C-29-3-13 : là sont les aides de
  facture et les deux tests ci-dessus) et réécrit : facture de reste **égal** au montant de la transaction (score
  ≥ 0,5), règle qui correspond à la contrepartie ; attendu : **aucune** candidate `rule`, `candidates[0]` la
  facture. Dans `reconciliation_rules_e2e.rs`, le test et son en-tête « Test 15 » disparaissent ; une ligne de
  commentaire y renvoie au nouvel emplacement.

**AC4 — L'acceptation accepte la candidate affichée (vérification, et tests).**

Établi au code : le client envoie l'identifiant de **la** candidate qu'il affiche (`ReconciliationProposals.svelte:208-
224`, même tableau `proposals` que le rendu `:338-368`) ; le serveur accepte cet identifiant (`accept_one_invoice`
`:1621`, `accept_one_rule` `:2736`) **sans** relire l'ordre — ni `accept_once`, ni `accept_batch` (`:1162`, `:1303`)
ne choisissent une candidate. Aucun changement d'API. Le **Pattern batch** est intact : aucun nouveau refus, aucune
`AppError` globale ajoutée.

**Aucun test Vitest du dépôt ne construit aujourd'hui une candidate règle** (`makeProposalWithCandidate`,
`ReconciliationProposals.test.ts:65-110`, ne fabrique qu'une facture, `ruleId: null` ; le test `:199` accepte une
facture seule) : la branche règle d'`onAccept` (`.svelte:209-216`) et le badge `candidate-rule-badge` ne sont
éprouvés nulle part. La story comble ce manque (tâche T1) : une fabrique de proposition à **deux** candidates
(`makeProposalWithCandidates(txId, candidates)`, et un constructeur de candidate règle : `ruleId`, `ruleLabel`,
`ruleMatchType`, `counterpartyAccountId`, `counterpartyAccountName`, score 1,0), puis deux tests :

- `candidates = [règle, facture]` → la ligne affiche le badge *Règle* (`candidate-rule-badge`) et « Accepter » appelle
  `acceptProposals` avec `{ type: 'rule', bankTransactionId, ruleId, counterpartyAccountId }` — jamais l'`invoiceId`
  de la seconde ;
- `candidates = [facture, règle]` → pas de badge *Règle*, et « Accepter » envoie `{ type: 'invoice', invoiceId }`
  de la première.

Le serveur ne rend jamais `[règle, facture]` sur une entrée atteignable ; le premier test éprouve que l'écran
**obéit à l'ordre reçu**, sans le refaire (C-29-3-2).

**AC5 — L'écran dit la priorité (C-29-3-4, C-29-3-14, C-29-3-15).**

Une phrase au-dessus de la table, **dans la branche « liste non vide »** (`{:else}` de
`ReconciliationProposals.svelte:279`), **avant** la table (`:301`), et **seulement si au moins une ligne a une
candidate** (`proposals.some((p) => p.candidates.length > 0)` — sans quoi « une seule proposition est affichée » ne
dirait rien de vrai, C-29-3-14) : `<p class="mb-2 text-sm text-text-muted"
data-testid="reconciliation-priority-hint">`, clé `reconciliation-labels-priority-hint`, repli = valeur fr-CH mot pour
mot (contrôle du même type que `ReconciliationProposals.test.ts:661-678`). Valeurs — rien de plus fort que le code
(C-29-3-15 : « rapproche » couvre la référence contenue dans un sens ou dans l'autre et le préfixe commun, le nom
contenu dans un sens ou dans l'autre ; ni « figure dans », ni « désigne ») :

- **fr-CH** : `Une seule proposition est affichée par transaction : la facture la mieux notée s’il y en a une, sinon la règle d’affectation qui correspond. Une facture passe devant une règle dès que sa référence, le nom de son client ou son montant exact la rapproche du paiement.`
- **de-CH** : `Pro Transaktion wird ein einziger Vorschlag angezeigt: die am besten bewertete Rechnung, falls es eine gibt, sonst die passende Zuordnungsregel. Eine Rechnung geht einer Regel vor, sobald ihre Referenz, der Name ihres Kunden oder ihr genauer Betrag sie mit der Zahlung verbindet.`
- **it-CH** : `Per ogni transazione è mostrata una sola proposta: la fattura con il punteggio migliore, se ce n’è una, altrimenti la regola di assegnazione corrispondente. Una fattura precede una regola non appena il suo riferimento, il nome del suo cliente o il suo importo esatto la collegano al pagamento.`
- **en-CH** : `One proposal is shown per transaction: the best-scored invoice if there is one, otherwise the matching reconciliation rule. An invoice comes before a rule as soon as its reference, its customer’s name or its exact amount links it to the payment.`

Termes alignés sur l'écran : « Règles d'affectation » (`nav-reconciliation-rules`, fr `:116`), « Zuordnungsregeln »
(de `:116`), « Regole di assegnazione » (it `:116`), « Reconciliation rules » (en `:116`). Apostrophe typographique `’`
en fr-CH et it-CH, comme les clés voisines (fr `:1225`). Gardes i18n à passer, **nommées** : `npm run
lint-i18n-ownership` (la clé est dans le préfixe `reconciliation-` de son dossier), `i18n-libelle-en-dur.test.ts`,
parité des quatre catalogues, et **`frontend/src/lib/shared/i18n-keys.test.ts`** : son compteur `sitesTotal`
(`:634`, 2107 au sol) prend **+1** pour le nouveau site `i18nMsg(` — à **recompter à la source au rebase** (la 29-2
peut en ajouter : `grep -o "i18nMsg(" <fichier> | wc -l` aux deux bornes, collecte du test), avec le commentaire daté
qui accompagne chaque variation (`:620-633`) ; `sitesNonResolus`, `relais`, `sitesGabarit` inchangés, mesurés.
L'en-tête du composant (`.svelte:8`, « on affiche la candidate top-1 (la mieux scorée) ») est réécrit pour dire la
priorité et renvoyer à `order_candidates`. Tests Vitest : phrase rendue quand une ligne a une candidate ; **absente**
liste vide, en erreur, et quand **toutes** les lignes sont « Aucune correspondance » ; repli égal au catalogue fr-CH.

**AC6 — E2E Playwright (`frontend/tests/e2e/reconciliation-priority.spec.ts`, neuf).**

Montage par API, fixtures **synthétiques** (`docs/agents-orchestration.md` § 8), base E2E **partagée** : rien ne doit
attraper les transactions d'un autre spec. Aides existantes : `createContactWithAddressViaApi` et
`createAndValidateInvoiceViaApi(page, contactId, '2026-05-10')` (`frontend/tests/e2e/helpers/api-fixtures.ts:30`,
`:100` ; TTC fixe 972,90), relevé CAMT minimal comme `reconciliation-amount-due.spec.ts:20-110` (date du 2026-05-15,
dans la fenêtre ; montant **972,89** — sous la tolérance, montant non exact) ; règle par `POST
/api/v1/reconciliation/rules` avec une valeur de correspondance **unique à l'exécution** (patron de
`reconciliation-deleted-import.spec.ts:1-13`), contenue dans la contrepartie du relevé, laquelle contient aussi le
**nom du contact** (unique à l'exécution) : la facture a un signal de contact (0,1), la règle correspond. Nettoyage de
la règle en `afterAll` (la facture validée ne se supprime pas ; son contact unique la tient hors de portée des autres
specs). Parcours : la ligne montre la **facture** (`candidate-top-1[data-invoice-id=…]`), **pas** le badge *Règle*,
et la phrase de l'AC5 ; « Accepter » réussit, la ligne quitte la liste, la facture a un reste dû de 0,01 (lu par
l'API). **Captures clair et sombre** de la page avec la phrase, versées à côté de la fiche au développement (recette
visuelle de l'epic). E2E complet au **dernier commit de code**.

**AC7 — Manuel utilisateur (`docs/manual/fr/user-manual.tex`) et PDF.**

Inventaire **clos** des phrases qui décrivent la place de la règle (`grep -n "aucune facture\|score faible\|aucune}
facture\|mieux notée" docs/manual/fr/user-manual.tex` au sol : `:2041`, `:2048`, `:2049`, `:2116`, `:2905` — à
regreper au rebase, la 29-2 réécrit le paragraphe voisin `:2029`). L'ordre étant **conservé** (C-29-3-11), chacune a
été relue contre la règle retenue :

| site | aujourd'hui | verdict |
|---|---|---|
| `:2041` | « Quand aucune facture n'est candidate, la réconciliation propose à la place la règle […] score de 100 % » | **vrai** à l'écran ; ajouter « --- un score conventionnel, qui n'est pas comparé à celui des factures » |
| `:2048` | « la facture la mieux notée s'il y en a une » | **vrai**, inchangé |
| `:2049` | « une règle n'est affichée […] que s'il n'y a aucune facture candidate --- même une facture de score faible passe devant elle » | **vrai** ; ajouter la **raison** (une facture candidate est rapprochée du paiement par sa référence, le nom de son client ou son montant exact ; une règle porterait ce paiement en produit, renvoi à la FAQ `:2911`), le fait que **la page le rappelle au-dessus de la liste**, et l'angle mort de C-29-3-7 : « une facture que le paiement dépasse de quelques centimes est proposée, mais son acceptation est refusée en trop-perçu --- enregistrez alors le règlement depuis la fiche de la facture (issue \#665) » (forme des renvois d'issue du manuel : `:1308`, « issue \#401 ») |
| `:2116` | « seulement quand aucune facture n'est candidate » | **vrai**, inchangé |
| `:2905` | « Une règle n'est d'ailleurs affichée que s'il n'y a aucune facture candidate. » | **vrai**, inchangé |

Régénérer le PDF (`make fr` dans `docs/manual/`, sous le verrou de gate) et le **contrôler aplati**, ligatures
**normalisées** (`pdftotext` rend « aﬀichée » avec U+FB00 : un `grep` sur « affich- », « différ- », « fi », « fl »
donnerait un faux négatif) :
`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | sed 's/ﬀ/ff/g;s/ﬁ/fi/g;s/ﬂ/fl/g;s/ﬃ/ffi/g'`
puis `grep -c` d'une portion de **chacun** des trois ajouts → 1 chacun ; « même une facture de score faible passe
devant elle » → toujours 1. Manuels DE/IT/EN : vides (README seuls), rien à faire. Le manuel d'administration ne
parle pas des candidates (`grep -c candidate docs/manual/fr/admin-manual.tex` → 0 au sol).

**AC8 — CHANGELOG et documentation d'API.**

- **`CHANGELOG.md`** : la section v0.13.4 **n'existe pas** au sol (le fichier commence par `## [0.13.3] —
  2026-10-14`, `:11`). La **créer** en tête, sous la forme exacte que `scripts/prepare-release.sh` finalisera
  (`## [0.13.4] — Non publié`, `:237` du script) — **si la 29-2 l'a déjà créée au rebase, y ajouter** (une seule
  section admise, `:241-246` du script). Entrée `Changed` : la page de réconciliation dit, au-dessus de la liste,
  quelle proposition elle affiche — une facture candidate passe toujours devant une règle d'affectation ([#527]) ;
  pour les **intégrations par clé d'API**, l'ordre des `candidates` de `GET /api/v1/reconciliation/proposals` est
  désormais écrit et documenté (`docs/api-external.md`) — **il ne change pas**. Et le lien `[#527]` à la forme des
  entrées voisines.
- **`docs/api-external.md`** § *Accepter des propositions de rapprochement* (`:549`, après « Les propositions
  viennent de `GET /api/v1/reconciliation/proposals` ») : une phrase — l'ordre de `candidates` est une priorité
  écrite *(depuis la v0.13.4 ; l'ordre lui-même est celui des versions précédentes)* : d'abord les factures qui
  portent un indice (montant égal au centime, référence ou contrepartie liée), par score décroissant puis identifiant
  croissant ; puis la règle d'affectation (une au plus, `score.total = 1.0` conventionnel, **non comparable** au
  score d'une facture), présente seulement si aucune facture n'atteint `0.5` ; l'écran affiche et accepte
  `candidates[0]`.

**AC9 — Mutations (à exécuter, résultats au Change Log).**

| # | mutation | doit faire rougir |
|---|---|---|
| M1 | retirer l'appel `order_candidates` de `get_proposals` | **aucun test — mutant équivalent sur les entrées atteignables**, déclaré tel : la règle est poussée en dernier et chaque facture qui la côtoie est de rang 0 (fiche § *Quelles factures*). Non comptée tuée. À exécuter quand même : si un test rougit, la prémisse de C-29-3-11 est fausse, et c'est un finding |
| M2 | `candidate_rank` : règle → 3 (derrière tout) | AC2 cas 5, 6, 10, 11 |
| M3 | `candidate_rank` : règle → 0 (devant tout) | AC2 cas 2, 3, 6, 10, 11 |
| M4 | rang 0 sans `contact_score > 0.0` | AC2 cas 1, 2, 6, 10, 11 ; **AC3 cas 1** |
| M5 | rang 0 sans `reference_score > 0.0` | AC2 cas 3, 4, 10 ; **AC3 cas 2** |
| M6 | `sort_by_key` → `sort_unstable_by_key` | AC2 cas 11 ; si elle survit, la déclarer **non observable** avec la sortie du run, ne pas la compter tuée |
| M7 | tri par `score.total` décroissant à la place du rang | AC2 cas 1, 3, 6 ; **AC3 cas 1 et 2** (la règle à 1,0 passerait en tête) |
| M8 | rang 0 sans `amount_score == 1.0` | **AC2 cas 10 seul** — branche observable en direct seulement (montant exact ⇒ 0,5 ⇒ pas de règle) ; le cas 7 reste vert (`f2` garde le rang 0 par sa référence, `f1` passe seul au rang 2, derrière lui : ordre inchangé) |
| M9 | Svelte : phrase de l'AC5 hors de la branche non vide, ou sans la condition « une ligne a une candidate » | Vitest « absente liste vide » / « absente quand aucune ligne n'a de candidate » |
| M10 | facture du test muet réparé (AC3) retirée | ce test |

*(Recalculs : sous M4, le cas 8 ne bouge pas — les deux factures passent ensemble au rang 2 — et n'est pas compté ;
sous M5, le cas 4 rougit parce que `f2(0,0.5,0)` passe seul au rang 2, derrière `R`, `f1` restant au rang 0 par son
contact.)*

Restauration : `git checkout -- <fichier>` **puis `touch <fichier>`** (`docs/agents-orchestration.md` § 7).

## Zones partagées avec la 29-2 (#529) — à relire au rebase

La 29-2 (ancrage de la fenêtre de dates sur l'échéance ou la relance) a une fiche voisine, non mergée à la date de
celle-ci (worktree `kesh-29-2`, C-29-2-7 lu) ; ce qui suit se déduit de #529, de ce choix et du code. Zones
**probables** de conflit :

| fichier | zone 29-2 (probable) | zone 29-3 | risque |
|---|---|---|---|
| `routes/reconciliation.rs` | passe 2 de `get_proposals` (`:633-674`, `InvoiceDateWindow::for_transaction`), peut-être une **seconde source** de factures ou un **nouveau `CandidateType`** / un champ | fin de la passe 4 (`:722-826`), fonction et module de test neufs | **Moyen.** Un variant neuf fait refuser la compilation de `candidate_rank` : le classer (à justifier ; une facture sans aucun sous-score irait au rang 2) — c'est voulu (C-29-3-8). Des candidates ajoutées **après** la passe 4 : `order_candidates` reste **le dernier** geste avant `ReconciliationProposal`. |
| `kesh-db/repositories/reconciliation.rs` | `InvoiceDateWindow`, `find_unpaid_invoices_for_window` | **aucune** | nul — la 29-3 ne touche pas `kesh-db` |
| `kesh-reconciliation/src/matching.rs` | aucune (C-29-2-7 : départage inchangé) | **aucune** (C-29-3-12) | nul |
| `tests/reconciliation_e2e.rs` | tests neufs de la fenêtre | trois tests neufs (dont le test muet déplacé), `create_rule_ok` | faible (ajouts ; vérifier qu'un `create_rule_ok` n'a pas été ajouté par la 29-2) |
| `tests/reconciliation_rules_e2e.rs` | aucune connue | retrait du test muet | faible |
| `ReconciliationProposals.svelte` | boutons de ligne (`:370-398`), peut-être un bouton « choisir une facture » et un modal | phrase avant la table (`:279-301`), en-tête `:8` | faible ; si la 29-2 rend plusieurs candidates visibles, la phrase de l'AC5 est à revoir (« une seule proposition est affichée ») |
| `ReconciliationProposals.test.ts` | tests de boutons | fabriques et tests neufs | faible (ajouts) |
| `i18n-keys.test.ts` | `sitesTotal` (+n si la 29-2 ajoute des sites) | `sitesTotal` +1 | **à recompter** au rebase, jamais à additionner de confiance |
| `user-manual.tex` | `:2029` (fenêtre de 30 jours), `:2060`, `:2068-2076` (affectation manuelle), `:2902`, `:2911`, `:1305` (rappels) | `:2041`, `:2049` | **Moyen** : même sous-section ; les lignes bougent. Regreper les **valeurs** (« aucune facture », « score faible », « mieux notée »), pas les numéros. |
| catalogues `messages.ftl` | clés `reconciliation-*` neuves | une clé `reconciliation-labels-*` | faible (ordre alphabétique : vérifier la parité) |
| `CHANGELOG.md` | section `[0.13.4] — Non publié` | même section | **trivial mais à vérifier** : une seule section admise par le script de release |
| `docs/api-external.md` | § réconciliation, si la 29-2 change la fenêtre décrite | une phrase `:549` | faible |

La 29-3 **ne change pas** la forme JSON de `GET /proposals`, ni l'ordre sur les entrées atteignables : une 29-2 qui en
dépend n'a rien à adapter.

## Tasks / Subtasks

- [ ] T0 — Rebase sur `origin/main` (29-2 mergée) ; relire les lignes citées ; regreper « aucune facture », « score
  faible », « mieux notée » dans le manuel ; vérifier si la 29-2 a ajouté un `CandidateType`, une source de
  factures, une section `[0.13.4]` du CHANGELOG, des sites `i18nMsg(` (AC1, AC5, AC8).
- [ ] T1 — Tests d'abord : AC2 (unitaires — rouges par compilation tant que les fonctions n'existent pas) ; AC3 :
  aide `create_rule_ok` dans `reconciliation_e2e.rs`, deux tests HTTP (verts au sol : ils figent l'ordre) ; Vitest
  AC4 — **fabrique de proposition à deux candidates et constructeur de candidate règle**, puis les deux tests
  `[règle, facture]` et `[facture, règle]` ; Vitest AC5 (rouges au sol).
- [ ] T2 — `candidate_rank`, `order_candidates`, appel dans `get_proposals`, réécriture du commentaire `:722-724`,
  doc-comments (AC1).
- [ ] T3 — Test muet : **déplacer** `get_proposals_invoice_candidate_overrides_rule` de `reconciliation_rules_e2e.rs`
  vers `reconciliation_e2e.rs`, le réécrire avec une facture de reste égal au montant (score ≥ 0,5) et une règle qui
  correspond, asserter l'absence de la règle ; retirer l'original et son en-tête, laisser un renvoi d'une ligne
  (C-29-3-13). Ce n'est pas une retouche : c'est un test neuf, et la fiche le compte comme tel.
- [ ] T4 — Phrase de l'écran conditionnée à « une ligne a une candidate » + quatre catalogues + `sitesTotal`
  recompté + en-tête `.svelte:8` (AC5).
- [ ] T5 — Manuel (deux sites, trois ajouts) + PDF contrôlé aplati, ligatures normalisées (AC7) ; CHANGELOG (section
  créée ou complétée) et `docs/api-external.md` (AC8).
- [ ] T6 — Mutations M1-M10 (AC9), M1 et M6 exécutées et déclarées selon leur résultat.
- [ ] T7 — E2E neuf + captures clair et sombre (AC6) ; gate complet backend et frontend, E2E complet, au dernier
  commit de code, sous le verrou ; ne déclarer que ce qui a tourné.

## Dev Notes

### Ce qu'il faut préserver

- Le **seuil** `INVOICE_OVERRIDE_THRESHOLD = 0.5` et son effet : la règle n'est pas même calculée quand une facture
  l'atteint (C-29-3-5 ; AC #114 de la 8-5b).
- Le tri de `propose_matches` (`matching.rs:97-105`) : la story ne le touche pas ; `kesh-reconciliation` reste
  hors périmètre (C-29-3-12).
- La forme JSON (`CandidateType` `:119`, `ReconciliationCandidate` `:126-151`), y compris le 1,0 conventionnel de la
  règle et son affichage « 100 % » (`ScoreBadge.svelte`) : changer l'affichage du score de la règle n'est pas demandé.
- L'**ordre** sur les entrées atteignables (C-29-3-11) : M1 doit rester un mutant équivalent ; si elle tue un test,
  c'est la prémisse qui est fausse.
- L'acceptation (`accept_*`) : aucune ligne modifiée.
- La règle « une seule candidate visible » (KF-026, #76, ouverte) : la story ne la lève pas (C-29-3-9).

### Sites qui lisent l'ordre — inventaire clos (validation P1, R et F)

`grep -rnE "candidates\[0\]|candidates\.first|candidates\.push|\.candidates\b|top-1|candidateType"` sur
`crates/kesh-api/{src,tests}`, `frontend/{src,tests}` : un seul lecteur de production (`ReconciliationProposals.svelte:
205-208, 338-368`) ; un seul écrivain (`reconciliation.rs:806`) ; les tests Rust existants (`reconciliation_e2e.rs:709,
818, 839, 2818, 3629, 5420` ; `reconciliation_rules_e2e.rs:954, 1004, 1076, 1138, 1194, 1635, 2268, 2841` ;
`bank_accounts_closed_e2e.rs:637`) cherchent par `any`/`find`/`filter` ou ne lisent que la forme : **aucun ne dépend
de l'ordre** ; E2E Playwright : `reconciliation-amount-due.spec.ts:164` (facture seule), `reconciliation-unreject`,
`reconciliation-deleted-import` (règle seule) : indemnes.

### Gates

`kesh-db` non touché : gate ciblé permis en boucle (`binary(reconciliation_e2e)`, `binary(reconciliation_rules_e2e)`,
tests unitaires de `kesh-api`), fmt et clippy workspace ; gate complet + E2E complet au dernier commit de code. Base
`kesh_293`, `kesh_e2e_293`, port **3044**, `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-29-3/target`.

### Pièges

- **Flottants** : `0,4 + 0,1` vaut exactement `0,5` en `f64` dans l'ordre d'évaluation de `matching.rs:80` — le cas
  « référence + contact » atteint le seuil et n'a pas de règle. Le test HTTP du cas 2 doit donc **éviter** que la
  contrepartie et le nom du client se contiennent (montage écrit à l'AC3).
- **Contact** : `contact_score` est un « contient » bidirectionnel insensible à la casse, sans longueur minimale
  (`matching.rs:148-161`) ; « Dubois Client » et « Dubois » donnent 1. **Référence** : idem, `matching.rs:132-134` —
  une référence courte contenue dans le numéro donne le score plein.
- **Règle sur un crédit** : les règles s'appliquent aux crédits (pas de filtre de signe, P-H7) ; la règle du test doit
  viser un compte **imputable**, actif, autre que la banque (`active_account_ids`, `:711-715`), sans quoi elle n'est
  pas proposée.
- **Acceptation d'une facture de score faible** : `accept_one_invoice` ne refuse que `score.total <= 0.0`
  (`RECONCILIATION_SCORE_TOO_LOW`, `:1802-1811`) ; une facture à 0,1 s'accepte.

### References

- Route : `crates/kesh-api/src/routes/reconciliation.rs:586-845` (`get_proposals`), `:1621` (`accept_one_invoice`),
  `:2736` (`accept_one_rule`).
- Score : `crates/kesh-reconciliation/src/matching.rs:1-25` (doc du module), `:62-161`.
- Règles : `crates/kesh-reconciliation/src/rules.rs:85-95` (`first_matching_rule`).
- Trop-perçu : `crates/kesh-db/src/repositories/invoice_settlements.rs:160-184` ; #665.
- Frontend : `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte`, `ScoreBadge.svelte`,
  `ReconciliationProposals.test.ts`, `frontend/src/lib/shared/i18n-keys.test.ts`.
- Manuel : `docs/manual/fr/user-manual.tex:2025-2060`, `:2114-2117`, `:2896-2912`.
- API : `docs/api-external.md:549`. Release : `scripts/prepare-release.sh:234-262`.
- `CLAUDE.md` § *Pattern batch*, § *Un appariement automatique propose, il ne crée jamais* (le faux rattachement
  muet — argument de C-29-3-11).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-10-14 — Fiche créée (`bmad-create-story`, Opus 5.5) en autonomie. Prémisses de #527 établies au code :
  ordre « factures puis règle » sans retri et affichage de `candidates[0]` **confirmés** ; « la meilleure » du manuel
  **réfutée** (texte réécrit par #481/#519, il décrit aujourd'hui le défaut). Défaut voisin relevé (C-29-3-7, trop-perçu
  de quelques centimes proposé puis refusé) : issue demandée. Choix C-29-3-1 à C-29-3-10.
- 2026-10-14 — **Validation P1** (Sonnet, deux lentilles en contexte frais, lecture seule ; rapports
  `kesh-gate-logs/29-3-validate-p1-{R,F}.md`). **Trend : R 0 CRITICAL / 1 HIGH / 5 MEDIUM / 7 LOW (13) ; F 0 / 0 /
  5 / 7 (12)** — 25 findings, dont 11 MEDIUM et plus (R 6, F 5) ; convergences entre lentilles : R2/F3 (test muet),
  R3/F4 (fabrique Vitest), R5/F5 (référence bidirectionnelle), R6/F9b (contrat d'API), R7/F6 (#665), R9/F8a (terme
  EN), R10a/F9a (en-tête du composant), R10b/F8b (phrase sans candidate), R11/F2 (M6), R12/F12 (montage E2E).
  Remédiation (Opus 5.5) :
  - **R1 HIGH** (le rang 2 de la première rédaction plaçait la règle devant des factures identifiées par le client ou
    un préfixe de référence) : **décision de l'orchestrateur**, consignée en **C-29-3-11** — une facture qui porte un
    signal d'identité (sous-score de référence ou de contact non nul, ou montant exact) passe devant la règle, quel que
    soit son total. Examinée au code avant application : **aucun défaut du produit démontré** ; une **conséquence
    démontrée** — sur les entrées atteignables, l'ordre est celui d'aujourd'hui (une facture sans signal a un total nul
    et n'est pas candidate, `matching.rs:81`) ; la story écrit, teste, dit et documente cet ordre, sans le changer.
    Rangs, Story, AC1-AC3, AC5-AC7, AC9 réécrits en conséquence ; C-29-3-1 et C-29-3-3 annotés au registre.
  - **R2/F3 MEDIUM** : test muet **déplacé** dans `reconciliation_e2e.rs` et réécrit, décrit comme une tâche (T3,
    C-29-3-13 ; l'extraction des aides de facture vers `tests/common/` écartée).
  - **R3/F4 MEDIUM** : affirmation fausse de l'AC4 retirée ; fabrique à deux candidates et constructeur de règle en
    T1 ; tests `[règle, facture]` et `[facture, règle]`.
  - **R4 MEDIUM** : garde `sitesTotal` de `i18n-keys.test.ts` nommée (+1, à recompter au rebase).
  - **R5/F5 MEDIUM** : phrase de l'écran réécrite sans rien affirmer de plus fort que le code (« rapproche »), aux
    quatre langues (C-29-3-15) ; bidirectionnalité écrite au tableau et aux Pièges.
  - **R6 MEDIUM, F9 LOW** (a, b, c) : `docs/api-external.md:549` et section `[0.13.4] — Non publié` du CHANGELOG (à créer ou
    compléter) à l'AC8 ; en-tête `.svelte:8` ; zones partagées complétées.
  - **F1 MEDIUM** : M3 tuable (cas 2, 3, 6 : règle non terminale). **F2 MEDIUM, R11 LOW** : M6 tuable par le cas 11
    (51 éléments), ou déclarée non observable avec la sortie du run. M1 déclarée **équivalente** d'avance, avec preuve
    (conséquence de C-29-3-11).
  - LOW traités : R7/F6 (#665 cité partout), R8 (montages HTTP écrits valeur par valeur), R9/F8a (terme EN
    « reconciliation rule »), R10/F8b (en-tête `.svelte:8` ; phrase conditionnée à « une ligne a une candidate »,
    C-29-3-14), R12/F12 (montage E2E et aides nommés), R13 (numéros de ligne des types), F7 (ligatures du PDF), F10
    (entrées écrites pour chaque cas), F11 (branche « montant exact » observable en direct seulement, M8). **Aucun
    LOW refusé.**
  - Information de la 29-2 (C-29-2-7, départage par proximité de date) : non mécanique (dépend de l'ancre de la 29-2,
    touche `kesh-reconciliation`), écrite en **angle mort** (C-29-3-12), **issue demandée**.
  - Angle mort C-29-3-7 complété : les autres refus d'une facture proposée sont de configuration, déjà au manuel.
  - Choix ajoutés : **C-29-3-11 à C-29-3-15**. Décomptes recomptés depuis la fiche : 9 AC ; T0-T7 (8 tâches) ;
    M1-M10 (10 mutations) ; 11 cas à l'AC2 ; 3 tests HTTP à l'AC3 (dont le test muet déplacé) ; 5 sites du manuel
    relus, 2 modifiés ; 15 choix.
  - Validation P2 à lancer : Opus, lentilles R et F (`29-3-validate-prompt-p2.md`), axe obligatoire : ce que la
    remédiation P1 a écrit.
