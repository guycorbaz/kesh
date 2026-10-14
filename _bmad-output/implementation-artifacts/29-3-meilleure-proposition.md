# Story 29.3 : La proposition affichée suit une priorité écrite, et l'écran la dit

## Status

**review** — développement fait le 2026-10-14 (Opus 5.5 ; Dev Agent Record ; revue de code P1 à lancer, `29-3-review-prompt-p1.md`, Sonnet, B/E/A). Fiche créée le 2026-10-14 en autonomie (`bmad-create-story`, Opus 5.5), sur `16eeed21`
(kickoff de l'Epic 29), branche `story/29-3`, worktree `/home/gcorbaz/devel/kesh-29-3`. **Validation P1 remédiée**
le 2026-10-14 (rapports R 0/1/5/7, F 0/0/5/7 ; règle des rangs refondue par décision de l'orchestrateur,
C-29-3-11) ; **validation P2 remédiée** le 2026-10-14 (Opus, R 0/0/3/6, F 0/0/1/8) ; **passe ciblée P3 remédiée** le
2026-10-14 (Haiku, une lentille : 0/0/3/6) — **validation close, dev à lancer**. Choix consignés au
registre propre à la story, `epic-29-choix-autonomes-29-3.md`, **C-29-3-1 à C-29-3-18** (à verser par union au
registre de l'epic). 9 AC. PR : **`closes #527`** — dans le titre ou le corps de la PR **seulement**, jamais dans un
message de commit (B4 de la rétrospective 28).

⚠️ **Sérialisée derrière la 29-2** (#529), qui touche les mêmes fichiers (§ *Zones partagées avec la 29-2*). Le
développement **commence par un rebase** sur `origin/main` une fois la 29-2 mergée, et **relit** tous les numéros de
ligne de cette fiche (relevés sur `16eeed21`). Deux stories voisines sortent dans la **même release v0.13.4** et
touchent ce que la 29-3 écrit : la **29-5** ([#668](https://github.com/guycorbaz/kesh/issues/668), départage à score
égal) et la **29-6** ([#665](https://github.com/guycorbaz/kesh/issues/665), trop-perçu de quelques centimes) ; si
l'une est mergée avant le développement, le rebase relit les angles morts qui la citent (T0).

## Story

**As a** utilisateur qui rapproche ses relevés (Comptable ou Admin),
**I want** que la proposition affichée pour une transaction soit choisie selon une priorité écrite, que l'écran me
dit, et qui ne fasse jamais passer une règle d'affectation devant une facture proposée pour ce paiement,
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

**Conséquence, établie au code** : les trois sous-scores sont positifs ou nuls, donc `total > 0` ⇔ au moins un
sous-score > 0 — **toute** facture candidate porte un signal, que la règle figure ou non dans la liste. Quand elle y
figure, aucune facture n'atteint 0,5, donc aucune n'a un montant exact, et **chaque** facture porte un sous-score de
référence ou de contact non nul. Il n'existe aujourd'hui **aucune** facture candidate « reconnue par le seul montant
à quelques centimes ». Corollaire : pour toute sortie de `propose_matches`, le critère du rang 0 (AC1) redit le filtre
`total > 0` de `matching.rs:81` *(validation P2, R9 et R6)*.

### Prémisse du manuel : corrigée

#527 dit que le manuel (« vers :1511 ») annonce « la meilleure » candidate. **Ce n'est plus vrai** : le texte a été
réécrit par #481/#519 (CHANGELOG, section v0.13.0, l. 248) ; `grep -n meilleure docs/manual/fr/user-manual.tex` ne rend
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
| **2** | facture **sans aucun signal** — reconnue par le seul montant à quelques centimes | aucun indice : la règle passe devant. **Inatteignable aujourd'hui** (une telle facture a un total nul et n'est pas candidate, `matching.rs:81`) ; le rang existe pour qu'une source future de factures — un choix manuel ([#666](https://github.com/guycorbaz/kesh/issues/666)), ou un score de montant à gradient (la décision L17 de `matching.rs:112-114`, écartée en v0.1) — ne puisse pas placer en silence une facture sans indice devant une règle. *(La 29-2 n'ajoute ni source ni variant, et un poids changé, resté positif, ne peuple pas ce rang : validation P2, R6.)* |

À rang égal, l'ordre de `propose_matches` est **conservé** (tri **stable** : score décroissant, puis son départage —
`invoice_id` croissant au sol, que la Story 29-5, #668, remplace ; la 29-3 n'en dépend pas).

**Pourquoi le signal d'identité, et non le score** (décision de l'orchestrateur, C-29-3-11) : la règle du
`CLAUDE.md` *Un appariement automatique propose, il ne crée jamais* — le faux rattachement **muet** est le pire cas.
Accepter une règle pour le paiement d'un client porte ce paiement en produit : **produit compté deux fois, facture
restée due, rappel envoyé à un client qui a payé** (`user-manual.tex:2911` met déjà en garde contre ce mode
d'échec). Accepter la mauvaise facture **du même payeur**, à l'inverse, se voit au lettrage (son autre facture
reste due, il la réclame) et se défait ; la facture d'un autre client, non — c'est le risque résiduel écrit plus bas. L'argument qui fondait le rang
0 de la première rédaction (numéro dans la référence) vaut pour **toute** facture identifiée — y compris par le
seul nom du client (0,1) ou par un préfixe de référence (0,2, 0,3).

**Pourquoi pas le tri brut par score** (l'option 1 de #527) : le 1,0 de la règle est une **convention**, pas une
mesure — le comparer aux scores des factures compare deux grandeurs sans commune mesure ; et il ferait passer la
règle devant **toutes** les factures faibles, c'est-à-dire devant des paiements de clients identifiés.

**Effet net, pour l'utilisateur** : l'ordre affiché est **le même qu'avant** la story sur tout ce qui est atteignable
(C-29-3-11, conséquence démontrée ci-dessus) ; ce qui change, c'est qu'il est **écrit** (une fonction pure, testée),
**dit** à l'écran (AC5) et **documenté** pour les clients d'API (AC8). La règle n'est toujours pas calculée quand une
facture atteint 50 % (seuil inchangé, C-29-3-5).

⚠️ **Risque résiduel, écrit** : le faux rattachement **symétrique** — une facture proposée, donc placée devant la
règle, alors que le paiement n'est pas le sien. Le code en ouvre **trois voies**, toutes pour un montant à 5 centimes
près du reste dû d'une facture ouverte : (1) une contrepartie à la fois cliente et visée par une règle (un
remboursement d'un fournisseur qui est aussi client) ; (2) une référence de paiement **courte ou générique** contenue
dans le numéro, ou un **préfixe commun** de quatre caractères (`matching.rs:132`, `:142`, sans longueur minimale) ;
(3) un nom de contrepartie **contenu dans** celui d'un client sans être ce client, ou l'inverse (`:158`,
bidirectionnel — un remboursement « Coop » contre un client « Coop Immobilier »). C'est l'ordre d'aujourd'hui. Il se
voit au lettrage **seulement si** la facture choisie est celle du payeur réel (son autre facture reste due, il la
réclame) ; un paiement d'un tiers porté sur la facture d'un client marque celle-ci payée et supprime ses rappels,
**sans signal**. Non traité (C-29-3-11 ; *élargi en validation P2, R7*).

⚠️ **Angle mort écrit (C-29-3-7) — une facture candidate peut être proposée puis refusée.** Une transaction qui
**dépasse** le reste dû de 1 à 5 centimes est chargée par la tolérance du dépôt, mais l'acceptation la refuse en
`RECONCILIATION_OVERPAYMENT` (`classify_payment`, `crates/kesh-db/src/repositories/invoice_settlements.rs:175-184` :
`paid > raw_due` → trop-perçu, hors arrondi au centime). Une telle facture porte un signal (sinon elle n'est pas
candidate), reste au rang 0 et passe devant une règle valable — **ni aggravé ni réparé** par la story (elle passait
déjà devant). C'est la famille de #548 sur le **montant** : **[#665](https://github.com/guycorbaz/kesh/issues/665)**
(ouverte, P2), traitée par la **Story 29-6** dans la même release v0.13.4. ⚠️ **Aucun contournement n'est à
prescrire** : le règlement saisi depuis la fiche de la facture refuse lui aussi le montant reçu
(`SettleInvoiceDialog.svelte:179-184`, et au serveur la même `classify_payment`,
`invoice_settlements_write.rs:217-228` ; le manuel le dit à `user-manual.tex:1308`) — le manuel s'en tient au
constat et au renvoi (AC7, C-29-3-16). D'autres refus d'acceptation d'une facture proposée existent, de
**configuration** (compte bancaire lié au compte débiteurs, compte d'arrondi manquant ou égal au compte débiteurs,
documentés à `user-manual.tex:2056`), d'**état comptable** (exercice fermé ou invalide, période verrouillée —
`FISCAL_YEAR_INVALID`, `PERIOD_LOCKED`), de **course** (facture réglée ou redatée, transaction rapprochée entre
l'affichage et l'acceptation — `RECONCILIATION_INVOICE_NOT_ELIGIBLE`, `RECONCILIATION_ALREADY_RECONCILED`) ou
d'**intégrité** (`INVOICE_SALE_ENTRY_MALFORMED`) — **le manuel ne documente pas ces trois derniers** (seuls les refus
de configuration le sont, `:2056`) : ni aggravés ni réparés *(rectifié en validation P2, F9/R9 : « tous de
configuration » était trop fort ; en passe ciblée P3, P3-3 : le manuel n'énumère pas les refus d'état, de course et
d'intégrité à `:2060`)*.

⚠️ **Angle mort écrit (C-29-3-12) — le départage à score égal.** À rang et score égaux, `propose_matches` place la
facture de plus petit `invoice_id` en tête (`matching.rs:97-105`) — la plus ancienne, sans égard à l'exigibilité. La
29-2 (C-29-2-7 et C-29-2-11, fiche voisine non mergée) fait entrer parmi les candidates des factures **échues,
relancées, ou pas encore échues** — dont des factures anciennes à **échéance lointaine**, candidates sans limite de
temps : une facture ancienne non échue passera devant une facture échue du même client et du même reste dû. Ce
départage est porté par **[#668](https://github.com/guycorbaz/kesh/issues/668)** (P2), traité par la **Story 29-5**
dans la même release v0.13.4 ; son attendu, tel que la fiche 29-5 le tranche (art. 87 CO), est : à score égal, la **date
d'exigibilité** `max(date de facture, échéance)` croissante, puis la date de facture, puis l'identifiant. **La 29-3 ne le traite pas** : il touche `kesh-reconciliation` (ou la
requête du dépôt), hors périmètre, et `order_candidates` ne réordonne pas à l'intérieur d'un rang (stabilité voulue)
— elle **conserve** donc tel quel le départage que la 29-5 posera. La 29-3 ne touche pas non plus l'`ORDER BY i.date
ASC, i.id ASC` ni le `LIMIT 50` de `find_unpaid_invoices_for_window` (`kesh-db` `reconciliation.rs:258-259`) : au-delà
de 50 candidates de même reste dû, ils écartent les plus récentes — remarque transmise à la 29-5 (C-29-3-17).
*(Réécrit en validation P2, R3/F4 : #668 existait et n'était pas cité, et la portée était recopiée d'une rédaction
périmée de C-29-2-7.)*

## Critères d'acceptation

**AC1 — Une fonction pure ordonne les candidates (C-29-3-2, C-29-3-3, C-29-3-8, C-29-3-11).**

Dans `crates/kesh-api/src/routes/reconciliation.rs`, à côté des types de réponse (`CandidateType` `:119`,
`ReconciliationCandidate` `:126-151`) :

- `fn candidate_rank(c: &ReconciliationCandidate) -> u8` — rend 0, 1 ou 2 selon le tableau ci-dessus. Un
  **`match` exhaustif** sur `c.candidate_type` (**aucun bras `_`**) : un variant ajouté plus tard (le choix manuel
  de #666 pourrait en introduire un ; la 29-2 n'en ajoute pas) ne compile pas sans être classé — ce sera le
  seul `match` sur `CandidateType` (`grep -rn CandidateType crates/` : définition, champ, deux constructions —
  aucun `match` au sol). Le rang 0 d'une facture se lit sur les **sous-scores**
  `s.amount_score == 1.0 || s.reference_score > 0.0 || s.contact_score > 0.0` — valeurs littérales `1.0`, `0.5`,
  `0.0` posées par `matching.rs`, donc comparables exactement ; **jamais** sur un seuil de `score.total` (C-29-3-3).
- `fn order_candidates(candidates: &mut [ReconciliationCandidate])` — `candidates.sort_by_key(candidate_rank)`,
  **stable** (`sort_by_key` l'est ; ne pas employer `sort_unstable_by_key`). Elle ne suppose **rien** de l'ordre
  d'entrée (AC2 cas 2, 3 et 6).
- Doc-comments : la règle et le pourquoi de chaque rang (le faux rattachement muet, le refus du tri brut par score),
  le fait que le rang 2 est **inatteignable aujourd'hui** et pourquoi il existe, la mention que l'écran affiche et
  accepte `candidates[0]`, et les angles morts de C-29-3-7 (**#665**, Story 29-6) et C-29-3-12 (**#668**, Story
  29-5) — chacun relu au rebase si sa story est mergée (T0).

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
   l'intérieur d'un rang ; c'est `propose_matches` qui a fixé leur ordre — `invoice_id ASC` au sol, le départage de la
   29-5 ensuite) ;
9. `[R]` → `[R]` ; `[]` → `[]` ;
10. `candidate_rank` en direct : `(1,0,0)` → 0 ; `(0,1,0)` → 0 ; `(0,0.5,0)` → 0 ; `(0,0,1)` → 0 ; `(0,0,0)` → 2 ;
    règle → 1. *(La branche « montant exact » n'est observable que là : une facture à montant exact atteint 0,5 et
    supprime la règle — F11 de la P1.)*
11. **stabilité sur une taille réaliste** (M6) : entrée de 51 éléments — `R` en tête, puis 49 factures de rang 0
    `f49(0,0,1)` … `f1(0,0,1)` (identifiants **décroissants**), puis `f50(0,0,0)` — → les 49 factures dans l'ordre
    d'entrée (`49, 48, …, 1`), puis `R`, puis `f50`. La sortie attendue se construit par une expression
    indépendante de la production (`(1..=49).rev()`), écrite dans le test. 51 = le plafond de 50 factures du dépôt
    (`LIMIT 50`, `kesh-db` `reconciliation.rs:259`) plus la règle.

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
(C-29-3-15) : la seconde phrase se **borne à la facture proposée**, sans dire ce qui suffit à la faire proposer — une
facture n'est candidate que si son reste dû égale le montant à 5 centimes près, dans la fenêtre de dates, et une
référence qui « rapproche » un acompte de sa facture ne la rend pas candidate (`kesh-db` `reconciliation.rs:250-259`,
`HAVING`) *(validation P2, R1 : la rédaction P1 « dès que sa référence… la rapproche du paiement » affirmait une
condition suffisante que le code ne donne pas)* :

- **fr-CH** : `Une seule proposition est affichée par transaction : la facture la mieux notée s’il y en a une, sinon la règle d’affectation qui correspond. Une facture proposée passe toujours devant une règle.`
- **de-CH** : `Pro Transaktion wird ein einziger Vorschlag angezeigt: die am besten bewertete Rechnung, falls es eine gibt, sonst die passende Zuordnungsregel. Eine vorgeschlagene Rechnung geht einer Regel immer vor.`
- **it-CH** : `Per ogni transazione è mostrata una sola proposta: la fattura con il punteggio migliore, se ce n’è una, altrimenti la regola di assegnazione corrispondente. Una fattura proposta precede sempre una regola.`
- **en-CH** : `One proposal is shown per transaction: the best-scored invoice if there is one, otherwise the matching reconciliation rule. A proposed invoice always comes before a rule.`

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
dans la fenêtre ; montant **972,89** — sous la tolérance, montant non exact ; **toute référence du gabarit**
— `<Ref>`, `<Ustrd>`, `<EndToEndId>`, `<AcctSvcrRef>` — **remplacée** par une valeur unique à l'exécution de
préfixe `PRIO`, qu'aucun numéro de facture ne contient ni ne commence par les mêmes quatre caractères — format de
numérotation regrepé au développement : une autre facture ouverte à 972,90 marquerait sinon 0,2 ou 0,4 et passerait
devant celle du spec, 0,1, faisant rougir l'assertion `candidate-top-1[data-invoice-id=…]`) ; la contrepartie ne
contient **que** le nom du contact et la valeur de la règle, tous deux uniques à l'exécution ; règle par `POST
/api/v1/reconciliation/rules` avec une valeur de correspondance **unique à l'exécution** (patron de
`reconciliation-deleted-import.spec.ts:1-13`), contenue dans la contrepartie du relevé, laquelle contient aussi le
**nom du contact** (unique à l'exécution) : la facture a un signal de contact (0,1), la règle correspond. Nettoyage de
la règle en `afterAll`. **Ce qui tient la facture hors de portée des autres specs**, c'est le reste de **0,01** que
laisse l'acceptation — pas son contact, la candidature se faisant par le reste dû et la date (`HAVING`), le contact
n'ajoutant que 0,1. Si le spec échoue avant « Accepter », la facture reste due à 972,90 et deviendrait candidate **à
montant exact** pour toute transaction de 972,90 d'un autre spec dans sa fenêtre (toutes les factures de
`createAndValidateInvoiceViaApi` ont ce TTC) : l'`afterAll` lit donc son reste dû et, s'il vaut encore 972,90, la règle
par l'API de règlement (`POST /api/v1/invoices/{id}/settlements`, 972,89), ce qui laisse le même reste de 0,01
(C-29-3-18 ; *validation P2, R8*). Parcours : la ligne montre la **facture** (`candidate-top-1[data-invoice-id=…]`), **pas** le badge *Règle*,
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
| `:2049` | « une règle n'est affichée […] que s'il n'y a aucune facture candidate --- même une facture de score faible passe devant elle » | **vrai** ; ajouter la **raison** (une facture candidate est rapprochée du paiement par sa référence, le nom de son client ou son montant exact ; une règle porterait ce paiement en produit, renvoi à la FAQ `:2911`), le fait que **la page le rappelle au-dessus de la liste**, et l'angle mort de C-29-3-7, **sans contournement** (C-29-3-16) : « une facture que le paiement dépasse de quelques centimes est proposée, mais son acceptation est refusée en trop-perçu (issue \#665) » (forme des renvois d'issue du manuel : `:1308`, « issue \#401 ») — le règlement depuis la fiche refuse lui aussi le montant reçu (`:1308`), et la Story 29-6 traite #665 dans la même release : si elle est mergée au rebase, cet ajout s'adapte à ce qu'elle livre ou disparaît |
| `:2116` | « seulement quand aucune facture n'est candidate » | **vrai**, inchangé |
| `:2905` | « Une règle n'est d'ailleurs affichée que s'il n'y a aucune facture candidate. » | **vrai**, inchangé |

Régénérer le PDF (`make fr` dans `docs/manual/`, sous le verrou de gate) et le **contrôler aplati**, ligatures
**normalisées** (`pdftotext` rend « aﬀichée » avec U+FB00 : un `grep` sur « affich- », « différ- », « fi », « fl »
donnerait un faux négatif) :
`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | sed 's/ﬀ/ff/g;s/ﬁ/fi/g;s/ﬂ/fl/g;s/ﬃ/ffi/g'`
puis `grep -c` d'une portion de **chacun des quatre ajouts** — un à `:2041` (score conventionnel), trois à `:2049`
(la raison, le rappel à l'écran, l'angle mort #665) — → 1 chacun ; « même une facture de score faible passe devant
elle » → toujours 1 ; et `grep -c "depuis la fiche de la facture (issue"` → 0 (aucun contournement prescrit pour
#665, C-29-3-16) ; **contrôle positif en plus** *(passe ciblée P3, P3-7)* : `grep -c "refusée en trop-perçu"` → 1
après régénération (au sol, avant T5 : 0), et `grep -o ".\{40\}665.\{40\}"` dont les occurrences se relisent une à
une — le contrôle d'absence ne vise que la graphie exacte. **Portions courtes**, de trois ou quatre mots : l'aplatissement garde les césures (« réinitia-
lisation », cinq au sol : `pdftotext … | grep -cE "[[:alpha:]]-$"`), si bien qu'une portion à cheval sur une césure
rend 0 ; une portion qui rend 0 se cherche dans le PDF avant de conclure à son absence *(validation P2, F6)*. Manuels DE/IT/EN : vides (README seuls), rien à faire. Le manuel d'administration ne
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
  écrite *(depuis la v0.13.4 ; l'ordre lui-même est celui des versions précédentes)* : d'abord les candidates
  `candidateType = "invoice"` qui portent un indice (montant égal au centime, référence ou contrepartie liée), par
  score décroissant puis selon le départage de `propose_matches` (identifiant croissant au sol ; à relire si la 29-5,
  #668, est mergée au rebase) ; puis la règle d'affectation (une au plus, qui se reconnaît à `candidateType =
  "rule"` ; ses sous-scores valent 0 et son `score.total = 1.0` est conventionnel, **non comparable** au score d'une
  facture), présente seulement si aucune facture n'atteint `0.5` ; l'écran affiche et accepte `candidates[0]`
  *(validation P2, F8 : un intégrateur appliquant le critère « porte un indice » aux sous-scores aurait classé la
  règle parmi les candidates sans indice)*. La 29-2 ajoute une phrase dans **le même paragraphe** (C-29-2-5) : zone
  de conflit certaine.

**AC9 — Mutations (à exécuter, résultats au Change Log).**

| # | mutation | doit faire rougir |
|---|---|---|
| M1 | retirer l'appel `order_candidates` de `get_proposals` | **aucun test — mutant équivalent sur les entrées atteignables**, déclaré tel : la règle est poussée en dernier et chaque facture qui la côtoie est de rang 0 (fiche § *Quelles factures*). Non comptée tuée. À exécuter quand même : si un test rougit, la prémisse de C-29-3-11 est fausse, et c'est un finding. **Variante, équivalente pour la même raison** et déclarée telle : l'appel placé **avant** le `push` de la règle (`:806`) *(validation P2, F5)* |
| M2 | `candidate_rank` : règle → 3 (derrière tout) | AC2 cas 5, 6, 10, 11 |
| M3 | `candidate_rank` : règle → 0 (devant tout) | AC2 cas 2, 3, 6, 10, 11 |
| M4 | rang 0 sans `contact_score > 0.0` | AC2 cas 1, 2, 6, 10, 11 ; **AC3 cas 1** |
| M5 | rang 0 sans `reference_score > 0.0` | AC2 cas 3, 4, 10 ; **AC3 cas 2** |
| M6 | `sort_by_key` → `sort_unstable_by_key` | AC2 cas 11 ; si elle survit, la déclarer **non observable** avec la sortie du run, ne pas la compter tuée |
| M7 | tri par `score.total` décroissant à la place du rang | AC2 cas 1, 2, 3, 4, 6, 11 ; **AC3 cas 1 et 2** (la règle à 1,0 passerait en tête) *(liste complétée en validation P2, F2/R4)* |
| M8 | rang 0 sans `amount_score == 1.0` | **AC2 cas 10 seul** — branche observable en direct seulement (montant exact ⇒ 0,5 ⇒ pas de règle) ; le cas 7 reste vert (`f2` garde le rang 0 par sa référence, `f1` passe seul au rang 2, derrière lui : ordre inchangé) |
| M9 | Svelte : phrase de l'AC5 hors de la branche non vide, ou sans la condition « une ligne a une candidate » | Vitest « absente liste vide » / « absente quand aucune ligne n'a de candidate » |
| M10 | facture du test muet réparé (AC3) retirée | ce test |
| M11 | `candidate_rank` : facture sans signal → 0 (le `else 2` devient `else 0` : toute facture au rang 0) | AC2 cas 5, 6, 10, 11 — la mutation qui éprouve le rang 2, argument retenu pour garder la fonction (C-29-3-11) *(ajoutée en validation P2, F5)* |

*(Recalculs : sous M7, le cas 2 devient `[R, f1]` (attendu `[f1, R]` : rouge), le cas 4 devient `[R, f1, f2]`, le cas 11 commence par `R` ; le
cas 5 reste vert (`[R, f1]` déjà attendu). Sous M11, le cas 5 sort `[f1, R]`, le cas 6 `[f3, f4, R]`, le cas 11 finit
par `f50, R`. Sous M4, le cas 8 ne bouge pas — les deux factures passent ensemble au rang 2 — et n'est pas compté ;
sous M5, le cas 4 rougit parce que `f2(0,0.5,0)` passe seul au rang 2, derrière `R`, `f1` restant au rang 0 par son
contact.)*

Restauration : `git checkout -- <fichier>` **puis `touch <fichier>`** (`docs/agents-orchestration.md` § 7).

## Zones partagées avec la 29-2 (#529) — à relire au rebase

La 29-2 (ancrage de la fenêtre de dates sur l'échéance ou la relance) a une fiche voisine, non mergée à la date de
celle-ci (`kesh-29-2/_bmad-output/implementation-artifacts/29-2-paiement-tardif.md`, lue le 2026-10-14 : § *Modules
touchés*, § *Zones partagées avec la 29-3*, § *Exclusions écrites*, C-29-2-5, C-29-2-6, C-29-2-7, C-29-2-11). Le
tableau ci-dessous en est tiré, non déduit *(refait en validation P2, R3 : la version P1 se déduisait de #529 et
annonçait des zones — frontend, i18n, nouveau `CandidateType` — que la fiche 29-2 exclut)*. Les numéros sont ceux de
`16eeed21`.

| fichier | zone 29-2 (d'après sa fiche) | zone 29-3 | risque |
|---|---|---|---|
| `routes/reconciliation.rs` | commentaire de la passe 2 de `get_proposals` (`:653-662`) ; étape 6 d'`accept_one_invoice` (fenêtre, `:1716-1765`) ; ni source de factures ni `CandidateType` neufs | fin de la passe 4 (`:722-826`), fonction et module de test neufs | **faible** : zones disjointes du même fichier. `order_candidates` reste **le dernier** geste avant `ReconciliationProposal` |
| `kesh-db/repositories/reconciliation.rs` | `WHERE` et jointures de `find_unpaid_invoices_for_window` ; `ORDER BY i.date ASC, i.id ASC` et `LIMIT 50` **inchangés** (C-29-2-7) | **aucune** — ni l'`ORDER BY` ni le `LIMIT 50` (C-29-3-17) | nul — la 29-3 ne touche pas `kesh-db` |
| `kesh-reconciliation/src/matching.rs` | aucune (`propose_matches` non touché) | **aucune** (C-29-3-12) | nul avec la 29-2 ; la **29-5** (#668) y touchera peut-être le départage : `order_candidates` le conserve (tri stable) |
| `tests/reconciliation_e2e.rs` | tests de parité et du paiement tardif (AC6 de la 29-2) | trois tests neufs (dont le test muet déplacé), `create_rule_ok` | faible (ajouts ; vérifier qu'un `create_rule_ok` n'a pas été ajouté entre-temps) |
| `tests/reconciliation_rules_e2e.rs` | aucune | retrait du test muet | nul |
| frontend (`ReconciliationProposals.svelte`, `.test.ts`, `i18n-keys.test.ts`), catalogues `messages.ftl` | **aucune** (C-29-2-6 : ni frontend, ni i18n, ni capture) | phrase, en-tête `.svelte:8`, fabriques et tests, une clé, `sitesTotal` +1 | nul avec la 29-2 ; `sitesTotal` se **recompte** quand même au rebase (une autre story a pu en ajouter) |
| `user-manual.tex` | `:1305`, `:2029`, `:2031`, `:2902` | `:2041`, `:2049` | **moyen** : même sous-section ; les lignes bougent. Regreper les **valeurs** (« aucune facture », « score faible », « mieux notée »), pas les numéros |
| `docs/api-external.md` | une phrase dans § *Accepter des propositions de rapprochement* (`:547-567`, C-29-2-5) | une phrase `:549`, même paragraphe | **certain** : conflit de texte à résoudre à la main, les deux phrases gardées |
| `CHANGELOG.md` | section `[0.13.4] — Non publié` (créée par la première story mergée) | même section | **trivial mais à vérifier** : une seule section admise par le script de release |

La 29-3 **ne change pas** la forme JSON de `GET /proposals`, ni l'ordre sur les entrées atteignables : une 29-2 qui en
dépend n'a rien à adapter. ⚠️ **Ce que la fiche 29-2 attend encore de la 29-3, et que la 29-3 ne fait pas** : sa
version lue le 2026-10-14 écrit, à son § *Zones partagées*, que « la 29-3 ou la 29-5 toucheront probablement
l'`ORDER BY` / `LIMIT 50` adjacents » (au conditionnel, à deux voix), et, à son § *Exclusions*, que l'ordre des
candidates (règle contre facture) revient à la 29-3 (#527) tandis que le **départage à score égal et la troncature
`LIMIT 50`** reviennent à la **29-5** (#668). La 29-3 ne touche ni l'`ORDER BY` ni le `LIMIT 50` (C-29-3-12,
C-29-3-17) : la troncature et le départage sont à la 29-5, qui les a pris (son AC3). Au rebase, relire ces deux §
de la 29-2 dans sa version mergée.

**Zones avec la 29-5 (#668) et la 29-6 (#665)**, même release, fiches lues le 2026-10-14 (passe ciblée P3) :
la 29-5 touche le départage (`matching.rs:97-105`) **et** l'`ORDER BY` / `LIMIT 50` de
`find_unpaid_invoices_for_window` (`kesh-db`, son AC3) — la 29-3 ne touche pas `kesh-db`, l'affirmation tient ; ce que
disent l'angle mort C-29-3-12, le doc-comment de l'AC1, la phrase de `api-external.md` (AC8, que la 29-5 cite dans sa
rédaction d'avant la passe P2 de la 29-3 : « par score décroissant puis identifiant croissant ») et le cas 8 de l'AC2
sont à relire à son merge. La 29-6 touche la même requête et le manuel `:2049` (trop-perçu de quelques centimes) — ce
que disent l'angle mort C-29-3-7 et l'ajout #665 du manuel (AC7). T0 les relit selon l'ordre de merge.

## Tasks / Subtasks

- [x] T0 — Rebase sur `origin/main` (29-2 mergée) ; relire les lignes citées ; regreper « aucune facture », « score
  faible », « mieux notée » dans le manuel ; vérifier si la 29-2 a ajouté un `CandidateType`, une source de
  factures, une section `[0.13.4]` du CHANGELOG, des sites `i18nMsg(` (AC1, AC5, AC8) ; si la **29-5** (#668) est
  mergée, relire l'angle mort C-29-3-12, le doc-comment de l'AC1 et la phrase de `api-external.md` (départage) ; si
  la **29-6** (#665) est mergée, relire l'angle mort C-29-3-7 et l'ajout #665 du manuel (AC7).
- [x] T1 — Tests d'abord : AC2 (unitaires — rouges par compilation tant que les fonctions n'existent pas) ; AC3 :
  aide `create_rule_ok` dans `reconciliation_e2e.rs`, deux tests HTTP (verts au sol : ils figent l'ordre) ; Vitest
  AC4 — **fabrique de proposition à deux candidates et constructeur de candidate règle**, puis les deux tests
  `[règle, facture]` et `[facture, règle]` ; Vitest AC5 (rouges au sol).
- [x] T2 — `candidate_rank`, `order_candidates`, appel dans `get_proposals`, réécriture du commentaire `:722-724`,
  doc-comments (AC1).
- [x] T3 — Test muet : **déplacer** `get_proposals_invoice_candidate_overrides_rule` de `reconciliation_rules_e2e.rs`
  vers `reconciliation_e2e.rs`, le réécrire avec une facture de reste égal au montant (score ≥ 0,5) et une règle qui
  correspond, asserter l'absence de la règle ; retirer l'original et son en-tête, laisser un renvoi d'une ligne
  (C-29-3-13). Ce n'est pas une retouche : c'est un test neuf, et la fiche le compte comme tel.
- [x] T4 — Phrase de l'écran conditionnée à « une ligne a une candidate » + quatre catalogues + `sitesTotal`
  recompté + en-tête `.svelte:8` (AC5).
- [x] T5 — Manuel (deux sites, quatre ajouts) + PDF contrôlé aplati, ligatures normalisées (AC7) ; CHANGELOG (section
  créée ou complétée) et `docs/api-external.md` (AC8).
- [x] T6 — Mutations M1-M11 (AC9), M1 (et sa variante) et M6 exécutées et déclarées selon leur résultat.
- [x] T7 — E2E neuf + captures clair et sombre (AC6) ; gate complet backend et frontend, E2E complet, au dernier
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

Opus 5.5 (`claude-opus-5-5`), `bmad-dev-story` en autonomie, le 2026-10-14, worktree `/home/gcorbaz/devel/kesh-29-3`.

### Debug Log References

Journaux (hors dépôt) : `/home/gcorbaz/devel/kesh-gate-logs/293-test-fast.log`, `293-front.log`, `293-e2e-full.log`,
`293-mutations.log`, `293-make-fr.log`. Bases `kesh_293` (remise à zéro avant le gate complet) et `kesh_e2e_293`
(reconstruite avant l'E2E complet), port 3044, `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-29-3/target`.

### Completion Notes List

**T0 — rebase et zones partagées.** Développement commencé sur la tête de `story/29-2` (`d5df4994`) ; la 29-2 mergée en
squash sur `main` (`c36f39bc`, #673) pendant le développement, la branche a été **rebasée** par
`git rebase --onto origin/main d5df4994 story/29-3` (demande de l'orchestrateur), à un point d'arrêt sûr (copie
propre, campagne de mutations terminée) : dix commits rejoués **sans conflit**, `git diff d5df4994 origin/main` vide
(contenu de la 29-2 identique), `sprint-status.yaml` relu par `yaml.safe_load`. Zones relues : la 29-2 ne touche ni
`CandidateType`, ni la passe 4 de `get_proposals`, ni le frontend, ni l'i18n (`sitesTotal` à 2107 au rebase) ; elle a
créé la section `[0.13.4] — Non publié` du CHANGELOG (« Corrigé ») — complétée ; elle a ajouté un paragraphe **Dates**
après celui de `api-external.md:549` — la phrase de la 29-3 va à la fin de `:549`, sans conflit ; le manuel : les cinq
sites de l'inventaire de l'AC7 sont aux mêmes lignes (`:2041`, `:2048`, `:2049`, `:2116`, `:2905`), la 29-2 ayant
réécrit `:2029`/`:2031`/`:2902` sans les toucher. 29-5 et 29-6 non mergées : angles morts C-29-3-7 et C-29-3-12
inchangés. Aucun LOW de validation n'était laissé ouvert à T0 (passe ciblée P3 : « aucun refusé »).

**Tests rouges d'abord** (commit `fdc859d7`, avant toute ligne de production) :

- AC2 — rouges par compilation :
  ```
  6 error[E0425]: cannot find function `candidate_rank` in this scope
  1 error[E0425]: cannot find function `order_candidates` in this scope
  error: could not compile `kesh-api` (lib test) due to 7 previous errors
  ```
- AC3 — **verts au sol**, comme prévu (l'ordre atteignable ne change pas, C-29-3-11) :
  `3 tests run: 3 passed` (`proposals_keep_a_referenced_invoice_before_the_rule`,
  `get_proposals_invoice_candidate_overrides_rule`, `proposals_put_an_invoice_named_by_the_counterparty_before_the_rule`).
- Vitest AC4 — verts au sol (l'écran obéissait déjà à l'ordre reçu ; ils comblent l'absence de toute candidate règle
  dans les tests) ; AC5 — rouge : `× une ligne a une candidate ⇒ la phrase est rendue…` —
  `TestingLibraryElementError: Unable to find an element by: [data-testid="reconciliation-priority-hint"]`
  (`Tests 1 failed | 29 passed (30)`). Les tests d'absence (liste vide, erreur, aucune candidate) sont verts au sol
  par nature.

**Livré** (AC1 à AC9) : `candidate_rank` (`match` exhaustif, sans `_`, rang 0 sur les sous-scores) et
`order_candidates` (`sort_by_key`, stable), appelée en dernier geste sur `candidates` ; commentaire « AC #114 » réécrit ;
11 cas unitaires (`candidate_order_tests`) ; `create_rule_ok` et deux aides locales (`single_proposal_candidates_293`,
`seed_invoice_and_tx_293`) dans `reconciliation_e2e.rs`, deux tests HTTP neufs et le test muet **déplacé** et réécrit
(facture à 0,5, aucune règle), retiré de `reconciliation_rules_e2e.rs` avec son en-tête « Test 15 », renvoi d'une
ligne ; Vitest : fabrique `makeProposalWithCandidates`, constructeurs `ruleCandidate`/`invoiceCandidate`, deux tests
AC4, cinq tests AC5 (dont celui de C-29-3-22) ; phrase de l'écran aux quatre catalogues, mot pour mot de la fiche ;
`sitesTotal` 2107 → 2108 (collecte du test : `expected 2108 to be 2107` avant la mise à jour ; `grep -o "i18nMsg("`
sur le composant : 19 aux deux bornes `d5df4994`/`fdc859d7`, 20 après) ; en-tête du composant réécrit ; manuel
(quatre ajouts, `\label{sec:faq-aucun-match}`), PDF régénéré ; CHANGELOG (« Modifié ») et `api-external.md` ; E2E
`reconciliation-priority.spec.ts` (deux tests, clair et sombre) et deux captures.

**Contrôle du PDF** (aplati, ligatures normalisées, commande de l'AC7) : « un score conventionnel » 1 ; « porterait ce
paiement en produit » 1 ; « La page le rappelle au-dessus » 1 ; « refusée en trop-perçu » 1 (0 au sol, `.tex`) ;
« même une facture de score faible passe devant elle » 1 ; « depuis la fiche de la facture (issue » 0 ;
`grep -o ".\{40\}665.\{40\}"` : une occurrence, « …ation est refusée en trop-perçu (issue #665) ; • sans
candidate… » ; renvoi rendu « (§21.4) » ; « ?? » 0 ; césures `[[:alpha:]]-$` : 4.

**Mutations (AC9)** — script de campagne hors dépôt, motif vérifié unique avant écriture, restauration par
`git checkout -- <fichier>` puis `touch`, sous verrou ; cible : `kesh-api` lib `candidate_order_tests` +
`reconciliation_e2e` + `reconciliation_rules_e2e` (157 tests), ou le Vitest du composant :

| # | résultat | tests rouges (relevés au journal) |
|---|---|---|
| M1 | **équivalente**, comme déclaré (157/157 verts) | — |
| M1b (appel avant le `push`) | **équivalente**, comme déclaré (157/157) | — |
| M2 | tuée (4) | cas 5, 6, 10, 11 |
| M3 | tuée (5) | cas 2, 3, 6, 10, 11 |
| M4 | tuée (6) | cas 1, 2, 6, 10, 11 ; HTTP `proposals_put_an_invoice_named_by_the_counterparty_before_the_rule` |
| M5 | tuée (4) | cas 3, 4, 10 ; HTTP `proposals_keep_a_referenced_invoice_before_the_rule` |
| M6 | tuée (1) | cas 11 |
| M7 | tuée (8) | cas 1, 2, 3, 4, 6, 11 ; les deux HTTP |
| M8 | tuée (1) | cas 10 |
| M9a (sans la condition) | tuée | Vitest « aucune ligne n'a de candidate » |
| M9b (hors de la branche) | **survivante au premier run**, puis tuée après ajout d'un test (C-29-3-22) | Vitest « rechargement en erreur après une liste à candidates » |
| M10 | tuée (1) | `get_proposals_invoice_candidate_overrides_rule` |
| M11 | tuée (4) | cas 5, 6, 10, 11 |

Les listes de rouges sont **exactement** celles que prévoit le tableau de l'AC9, pour chaque mutation. 13 jeux
(11 mutations, M1 et M9 en deux variantes) : 11 tuées, 2 équivalentes (M1, M1b).

**Gates au dernier commit de code** (`71caa721`, branche rebasée sur `c36f39bc`) :

- `scripts/test-fast.sh` (fmt + clippy + nextest, sous verrou, base remise à zéro) : **3624 tests run: 3624 passed,
  6 skipped** — 3611 de la 29-2 + 11 unitaires + 3 HTTP − 1 test retiré de `reconciliation_rules_e2e.rs`.
- Frontend (sous verrou) : `npm run check` 0 erreur, 29 avertissements (tous dans des fichiers non touchés :
  `BankProfileForm`, `RuleFormModal`, `ManualReminderDialog`, `reports`, `design-system`) ; `lint-i18n-ownership`
  PASS ; `npm run test:unit` **146 fichiers, 1617 tests** (1610 + 7) ; `npm run build` vert.
- **E2E complet** (sous verrou, `kesh_e2e_293` reconstruite, port 3044) : **282 passés, 7 échoués, 19 ignorés** — les
  sept échecs sont les sept KF-029 de `docs/testing.md` § *Les échecs attendus* (`mode-expert:26`, `:41`,
  `onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`, `:150`), aucun autre ; les deux tests neufs passés.
- Le spec neuf seul, avant le complet : 2 échecs puis 2 (montage — C-29-3-21, C-29-3-20), puis **2 passés**.

**Captures** (`29-3-captures/`) : la phrase est au-dessus de la table, la ligne montre la facture (10 %), sans badge
*Règle*. L'application n'ayant pas de thème sombre global, le sombre émulé rend comme le clair (C-29-3-25).

**Écart de méthode, déclaré** : les trois tests HTTP neufs ont été **ajoutés en fin de**
`reconciliation_e2e.rs` par une redirection `cat >> … <<'EOF'` (ajout pur, aucune substitution), et non par l'outil
d'édition ; toutes les autres modifications de code l'ont été par l'outil d'édition, plus `cargo fmt`.

**Choix ajoutés** : C-29-3-19 à C-29-3-26 (26 au total, `grep -cE '^#{3,4} C-29-3-'`).

**Remarques pour l'orchestrateur** : (1) `reconciliation-amount-due.spec.ts` croit départager par le contact une
contrepartie qui ne porte pas le nom enregistré (« Pia <nom> ») — le départage annoncé n'a pas lieu (C-29-3-21) ;
P4 au plus, pas d'issue ouverte par l'agent. (2) À signaler sur #527 : la prémisse « le manuel annonce la meilleure »
était déjà périmée (fiche, § *Prémisse du manuel*).

### File List

- `crates/kesh-api/src/routes/reconciliation.rs` — `candidate_rank`, `order_candidates`, appel, commentaire, module
  `candidate_order_tests`
- `crates/kesh-api/tests/reconciliation_e2e.rs` — `create_rule_ok`, deux aides, trois tests
- `crates/kesh-api/tests/reconciliation_rules_e2e.rs` — test muet retiré, renvoi
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` — `reconciliation-labels-priority-hint`
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte` — phrase, en-tête
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.test.ts` — fabriques, 7 tests
- `frontend/src/lib/shared/i18n-keys.test.ts` — `sitesTotal` 2108
- `frontend/tests/e2e/reconciliation-priority.spec.ts` — neuf
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/user-manual.pdf`
- `docs/api-external.md`, `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/29-3-captures/reconciliation-priorite-{clair,sombre}.png`
- `_bmad-output/implementation-artifacts/29-3-meilleure-proposition.md`, `epic-29-choix-autonomes-29-3.md`,
  `29-3-review-prompt-p1.md`, `sprint-status.yaml`

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
    Rangs, Story, AC1-AC3, AC5-AC7, AC9 réécrits en conséquence ; C-29-3-1, 3, 4, 6 et 7 annotés au registre (annotations datées, texte d'origine conservé).
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
- 2026-10-14 — **Validation P2** (Opus, deux lentilles en contexte frais, lecture seule ; rapports
  `kesh-gate-logs/29-3-validate-p2-{R,F}.md`). **Trend : R 0 CRITICAL / 0 HIGH / 3 MEDIUM / 6 LOW (9) ; F 0 / 0 / 1 /
  8 (9)** — 18 findings, 13 sujets distincts (convergences : R2/F1, R3/F4, R4/F2, R5/F3, R9/F9 en partie) ; MEDIUM
  distincts : 3 (R1, R2/F1, R3), **tous nés de la remédiation P1** sauf la partie « zones partagées » de R3 (fiche
  d'origine). Les deux lentilles confirment la conséquence de C-29-3-11 (rang 2 vide sur toute entrée atteignable, M1
  équivalente) et l'argument principal de la fonction. Remédiation (Opus 5.5) :
  - **R1 MEDIUM** : la phrase de l'écran affirmait une condition suffisante (« dès que sa référence… la rapproche ») ;
    bornée à la facture **proposée** aux quatre langues (« Une facture proposée passe toujours devant une règle. »),
    propagée à la Story, à l'AC5 et à C-29-3-15 (annotation).
  - **R2/F1 MEDIUM** : le contournement prescrit au manuel pour #665 échouait (la fiche refuse aussi le trop-perçu) ;
    retiré, le manuel s'en tient au constat et au renvoi, le contrôle du PDF vérifie son absence (**C-29-3-16**) ;
    #665 est la **Story 29-6**, même release.
  - **R3 MEDIUM, F4 LOW** : **#668** cité (Story **29-5**, attendu rectifié selon l'art. 87 CO) à l'angle mort, à
    l'AC1, au registre (C-29-3-12 annoté) ; portée de l'angle mort alignée sur C-29-2-11 (factures non échues
    comprises) ; `ORDER BY`/`LIMIT 50` hors de la 29-3, remarque transmise à la 29-5 (**C-29-3-17**) ; tableau des
    zones partagées **refait d'après la fiche 29-2** (frontend, i18n, `CandidateType` neuf retirés ;
    `docs/api-external.md:549` zone **certaine**) ; zones avec la 29-5 et la 29-6 écrites ; T0 complétée.
  - **Arbitrage de C-29-3-11 rendu par l'orchestrateur : la fonction est gardée** (annotation au registre).
  - LOW traités : **F2/R4** (M7 : cas 1, 2, 3, 4, 6, 11) ; **F3/R5** (quatre ajouts au manuel, contrôle de chacun) ;
    **F5** (**M11** ajoutée — facture sans signal au rang 0, tuée par les cas 5, 6, 10, 11 — et variante « appel
    avant le `push` » déclarée équivalente avec M1) ; **F6** (portions courtes, césures du PDF) ; **F7** (section
    v0.13.0 du CHANGELOG) ; **F8** (règle reconnue à `candidateType = "rule"`, sous-scores nuls, à
    `api-external.md`) ; **F9/R9** (refus d'état comptable, de course et d'intégrité, `:2060`) ; **R6** (raison du
    rang 2 : #666 ou un score à gradient, non la 29-2 ni un poids changé ; C-29-3-8 annoté) ; **R7** (risque
    résiduel élargi aux trois voies, « visible au lettrage » borné) ; **R8** (vraie protection de l'E2E : le reste de
    0,01 ; références du relevé remplacées, `afterAll` qui règle la facture en cas d'échec, **C-29-3-18**) ; **R9**
    (`LIMIT 50` à `:259`, en-tête du registre, conséquence vraie sans condition, commentaire de `sprint-status.yaml`).
    **Aucun finding refusé.**
  - Choix ajoutés : **C-29-3-16 à C-29-3-18** ; annotés : C-29-3-7, 8, 11, 12, 15. Décomptes recomptés depuis la
    fiche : 9 AC ; T0-T7 (8 tâches) ; M1-M11 (11 mutations, dont M1 équivalente) ; 11 cas à l'AC2 ; 3 tests HTTP à
    l'AC3 ; 5 sites du manuel relus, 2 modifiés, 4 ajouts ; 18 choix.
  - Passe ciblée P3 à lancer : Haiku, une lentille, braquée sur le seul diff de cette remédiation
    (`29-3-validate-prompt-p3-ciblee.md`).

- **2026-10-14 — Validation P3 (passe ciblée) remédiée** (Opus 5.5 ; une lentille, Haiku, braquée sur `b41e5b14` ;
  rapport `kesh-gate-logs/29-3-validate-p3-ciblee.md` : CRITICAL 0, HIGH 0, MEDIUM 3, LOW 6).
  - **Trend de la validation** : P1 R 0/1/5/7, F 0/0/5/7 → P2 R 0/0/3/6, F 0/0/1/8 → **P3 ciblée 0/0/3/6**.
  - Tous les constats vérifiés au texte source (`grep -nF`) avant correction ; **aucun refusé**.
  - **P3-1 MEDIUM** : l'attendu de #668 se cite d'après la fiche 29-5 (date d'exigibilité `max(date, échéance)`
    croissante, puis date, puis identifiant — art. 87 CO), non d'après le premier commentaire de l'issue.
  - **P3-2 MEDIUM** : le paragraphe sur la fiche 29-2 réécrit d'après sa version réelle (conditionnel « la 29-3 ou la
    29-5 », § *Exclusions* : départage et `LIMIT 50` à la 29-5) ; deux citations inexistantes retirées.
  - **P3-3 MEDIUM** : « énumérés à `:2060` » retiré ; le manuel ne documente pas les refus d'état, de course et
    d'intégrité.
  - LOW : **P3-4** (C-29-3-16, raison « caduc » fausse, annotée) ; **P3-5** (« ce sera le seul `match` », aucun au
    sol) ; **P3-6** (titre de C-29-3-12) ; **P3-7** (contrôle positif du PDF ajouté à l'AC7) ; **P3-8** (note M7 :
    le cas 2 « devient `[R, f1]`, rouge ») ; **P3-9** (zones 29-5/29-6 d'après leurs fiches).
  - **Validation close : la remédiation de la passe ciblée ne touche aucune ligne de production** (fiche et registre
    seulement ; `git diff --stat` sur `crates frontend docs` : vide).
  - Décomptes inchangés : 9 AC ; T0-T7 ; M1-M11 ; 18 choix (annotés en plus : C-29-3-7, 12, 16).

- **2026-10-14 — Développement** (`bmad-dev-story`, Opus 5.5, autonomie). Rebase sur `main` après le merge de la 29-2
  (`c36f39bc`), sans conflit. Commits `fdc859d7` (tests rouges d'abord), `1bba5f28` (code), `2f9e00f3` (manuel, PDF,
  API, CHANGELOG), `ec544621` (test qui tue M9b), `71caa721` (E2E et captures). Gates au dernier commit de code :
  test-fast **3624/3624**, Vitest **1617** (146 fichiers), check/lint/build verts, E2E **282 passés + 7 KF-029**.
  Mutations : 13 jeux, 11 tuées, M1 et M1b équivalentes comme déclaré ; M9b survivante au premier run, tuée par un
  test ajouté (C-29-3-22). Choix C-29-3-19 à 26. Statut `review` ; revue P1 : `29-3-review-prompt-p1.md` (Sonnet,
  B/E/A).
