# Story 29.3 : La proposition affichée suit une priorité écrite, et l'écran la dit

## Status

**ready-for-dev** — fiche créée le 2026-10-14 en autonomie (`bmad-create-story`, Opus 5.5), sur `16eeed21`
(kickoff de l'Epic 29), branche `story/29-3`, worktree `/home/gcorbaz/devel/kesh-29-3`. Validation P1 à lancer
(`29-3-validate-prompt-p1.md`, Sonnet, lentilles R et F). Choix consignés au registre propre à la story,
`epic-29-choix-autonomes-29-3.md`, **C-29-3-1 à C-29-3-10** (à verser par union au registre de l'epic).
9 AC. PR : **`closes #527`** — dans le titre ou le corps de la PR **seulement**, jamais dans un message de commit
(B4 de la rétrospective 28).

⚠️ **Sérialisée derrière la 29-2** (#529), qui touche les mêmes fichiers (§ *Zones partagées avec la 29-2*). Le
développement **commence par un rebase** sur `origin/main` une fois la 29-2 mergée, et **relit** tous les numéros de
ligne de cette fiche (relevés sur `16eeed21`).

## Story

**As a** utilisateur qui rapproche ses relevés (Comptable ou Admin),
**I want** que la proposition affichée pour une transaction soit choisie selon une priorité écrite, que l'écran me
dit, et qui ne cache pas une règle d'affectation derrière une facture qui ne correspond qu'à quelques centimes près,
**so that** « Accepter » ne règle pas une facture que rien ne désigne, alors qu'une règle valable attendait derrière.

Issue : [#527](https://github.com/guycorbaz/kesh/issues/527) (P2, bug ; relevée par la lentille R de la validation P2
de la 15-5c). Epic : `_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` § *29-3*. Cible : **v0.13.4**.

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
masque la règle, et « Accepter » la règle.

**Quelles factures sont « faibles », exactement.** Le dépôt ne charge que les factures dont le reste dû égale le
montant de la transaction **à 5 centimes près** (`AMOUNT_TOLERANCE_HUNDREDTHS = 5`, `:72` ;
`find_unpaid_invoices_for_window`, `crates/kesh-db/src/repositories/reconciliation.rs:234-274`, `HAVING`). Le score
de montant est binaire (`matching.rs:109-119`) : 1 si égal au centime. Le score d'une facture est
`0,50·montant + 0,40·référence + 0,10·contact`, la référence valant 1, 0,5 (préfixe commun ≥ 4 caractères) ou 0. Les
seuls totaux possibles **sous 0,5** sont donc, montant **différent de 1 à 5 centimes** :

| référence | contact | total | la règle est-elle proposée ? |
|---|---|---|---|
| 1 (le numéro figure dans la référence) | 1 | **0,5** (exact en `f64` : `0,0 + 0,4 + 0,1`) | non — seuil atteint |
| 1 | 0 | **0,4** | oui, aujourd'hui **derrière** la facture |
| 0,5 (préfixe commun) | 1 | 0,3 | oui, derrière |
| 0,5 | 0 | 0,2 | oui, derrière |
| 0 | 1 (nom du client) | 0,1 | oui, derrière |

### Prémisse du manuel : corrigée

#527 dit que le manuel (« vers :1511 ») annonce « la meilleure » candidate. **Ce n'est plus vrai** : le texte a été
réécrit par #481/#519 (CHANGELOG, v0.12.x, l. 248) ; `grep -n meilleure docs/manual/fr/user-manual.tex` ne rend
rien. Le manuel décrit aujourd'hui **fidèlement** le défaut (`user-manual.tex:2049` : « une règle n'est affichée […]
que s'il n'y a aucune facture candidate --- même une facture de score faible passe devant elle »). Ce qui reste faux
après la story, et qu'il faut réécrire, est inventorié à l'AC7. *(À signaler sur l'issue par l'orchestrateur.)*

### La règle retenue (C-29-3-1)

**Ni le tri brut par score, ni le statu quo : une priorité écrite, en trois rangs.**

| rang | candidate | pourquoi |
|---|---|---|
| **0** | facture **reconnue** : montant égal au centime (`amount_score == 1`) **ou** numéro de facture contenu dans la référence (`reference_score == 1`) | une preuve qui désigne **cette** facture |
| **1** | la règle d'affectation (une au plus) | un critère que l'utilisateur a posé lui-même |
| **2** | toute autre facture (montant à quelques centimes, préfixe de référence, nom du client) | un indice, pas une preuve |

À rang égal, l'ordre de `propose_matches` est **conservé** (tri **stable** : score décroissant, puis `invoice_id`
croissant).

**Pourquoi pas le tri brut par score** (l'option 1 de #527) : le 1,0 de la règle est une **convention**, pas une
mesure — le comparer aux scores des factures compare deux grandeurs sans commune mesure. Et il ferait passer la règle
devant une facture **dont la référence porte le numéro** (0,4) : le paiement d'un client qui a versé quelques centimes
de moins, avec le numéro de sa facture en communication, partirait en produit par la règle — **produit compté deux
fois, facture restée due, rappel envoyé à un client qui a payé** (`user-manual.tex:2911` met déjà en garde contre ce
mode d'échec). Avant la story, ce cas était bien servi (la facture passait devant) : le tri brut l'aggraverait.

**Pourquoi pas « la règle toujours devant »** : même raison. **Pourquoi pas le statu quo + une phrase à l'écran**
(l'option 2 de #527, prise au pied de la lettre) : une facture désignée par le seul nom du client (10 %) continuerait
de cacher une règle valable — c'est le défaut de l'issue.

Effet net, pour l'utilisateur : la règle passe désormais devant une facture **non reconnue** ; une facture reconnue
garde la première place ; la règle n'est **toujours pas** proposée quand une facture atteint 50 % (seuil inchangé,
C-29-3-5).

⚠️ **Angle mort écrit (C-29-3-7) — une facture faible peut être proposée puis refusée.** Une transaction qui
**dépasse** le reste dû de 1 à 5 centimes est chargée par la tolérance du dépôt, mais l'acceptation la refuse en
`RECONCILIATION_OVERPAYMENT` (`classify_payment`, `crates/kesh-db/src/repositories/invoice_settlements.rs:175-184` :
`paid > raw_due` → trop-perçu, hors arrondi au centime). Une telle facture, si sa référence porte le numéro, reste au
rang 0 et passe devant une règle valable — **ni aggravé ni réparé** par la story (elle passait déjà devant). C'est la
famille de #548 (proposer ce que l'acceptation refuse) sur le **montant**, non sur la date ; la réparer touche le SQL
du dépôt et la tolérance, que la 29-2 retouche : **issue à ouvrir** (rapport), pas d'intégration ici.

## Critères d'acceptation

**AC1 — Une fonction pure ordonne les candidates (C-29-3-1, C-29-3-3, C-29-3-8).**

Dans `crates/kesh-api/src/routes/reconciliation.rs`, à côté des types de réponse :

- `fn candidate_rank(c: &ReconciliationCandidate) -> u8` — rend 0, 1 ou 2 selon le tableau ci-dessus. Un
  **`match` exhaustif** sur `c.candidate_type` (**aucun bras `_`**) : un variant ajouté plus tard (la 29-2 peut en
  introduire un) ne compile pas sans être classé. Le rang 0 se lit sur les **sous-scores**
  `c.score.amount_score == 1.0 || c.score.reference_score == 1.0` — valeurs littérales `1.0` posées par `matching.rs`,
  donc comparables exactement ; **jamais** sur un seuil de `score.total` (C-29-3-3).
- `fn order_candidates(candidates: &mut [ReconciliationCandidate])` — `candidates.sort_by_key(candidate_rank)`,
  **stable** (`sort_by_key` l'est ; ne pas employer `sort_unstable_by_key`).
- Doc-comments : la règle, le pourquoi de chaque rang (dont le refus du tri brut par score), la mention que
  l'écran affiche et accepte `candidates[0]`, et l'angle mort de C-29-3-7 avec le numéro de l'issue ouverte.

`get_proposals` appelle `order_candidates(&mut candidates)` **après** l'ajout éventuel de la règle (`:806-826`) et
avant la construction de `ReconciliationProposal` (`:827`). Le commentaire « AC #114 (Pass 2 Q8) » (`:722-724`) est
réécrit pour dire l'ordre. Rien d'autre ne change dans la route : ni le seuil, ni la forme JSON, ni les filtres.

**AC2 — Tests unitaires de l'ordre (module `#[cfg(test)] mod candidate_order_tests`, même fichier).**

Constructeurs locaux d'une candidate facture (sous-scores donnés) et d'une candidate règle. Cas, chacun nommé :

1. règle + facture contact seul (0,1) → `[règle, facture]` (le cas de #527) ;
2. règle + facture préfixe de référence (0,2) et facture préfixe + contact (0,3) → `[règle, 0,3, 0,2]` ;
3. règle + facture référence pleine (0,4) → `[facture, règle]` ;
4. règle + facture 0,4 + facture 0,1 → `[0,4, règle, 0,1]` ;
5. factures seules, montant exact (0,5 et 0,9) → ordre d'entrée **inchangé** (stabilité) — entrée `[0,9 id 2,
   0,5 id 1]` rendue telle quelle ;
6. deux factures de rang 2 à score égal, `invoice_id` 5 puis 3 en entrée → ordre d'entrée conservé (le tri ne
   réordonne pas à l'intérieur d'un rang ; c'est `propose_matches` qui a fixé `invoice_id ASC`) ;
7. règle seule → `[règle]` ; liste vide → vide ;
8. `candidate_rank` : valeurs 0/1/2 sur une facture `amount_score = 1` (rang 0), `reference_score = 1` (rang 0),
   `reference_score = 0,5` + `contact_score = 1` (rang 2), règle (rang 1).

Les rangs et les ordres attendus sont **écrits en dur**, jamais relus de la production.

**AC3 — Tests d'intégration HTTP (`crates/kesh-api/tests/reconciliation_e2e.rs`, `#[sqlx::test(migrations =
"../kesh-db/test-schema")]`).**

Le fichier a les aides de factures (`seed_validated_invoice`, `:384`) et de transactions (`seed_bank_transactions`,
`:512`) ; la règle se crée par `POST /api/v1/reconciliation/rules` (forme : `reconciliation_rules_e2e.rs:962-978`),
contrepartie un compte de produit **imputable** et **distinct** du compte de la banque. Transaction créditrice CHF,
date en plein exercice 2026 et dans la fenêtre de la facture (ne pas dépendre du détail de la fenêtre, que la 29-2
change : facture datée **du jour même** de la transaction).

- `proposals_put_a_matching_rule_before_a_weak_invoice` : facture de reste 100.00, client « Dubois SA » ;
  transaction 99.99 contrepartie « Dubois SA », **sans** référence ; règle « contrepartie contient Dubois ». Attendu :
  `candidates[0].candidateType == "rule"`, `candidates[1]` la facture (score 0,1). Puis `POST /accept` de
  `candidates[0]` → acceptée en **règle** ; la facture reste **non payée** (reste dû 100.00, `paid_at` nul).
- `proposals_keep_a_referenced_invoice_before_the_rule` : même montage, transaction 99.99 **avec** le numéro de la
  facture en référence et une contrepartie qui ne contient **pas** le nom du client (sinon 0,5 : seuil atteint, pas
  de règle). Attendu : `candidates[0]` la facture (score 0,4), `candidates[1]` la règle. `POST /accept` de
  `candidates[0]` → accepté en facture ; reste dû 0.01 (règlement partiel, `classify_payment` → `Ordinary`).
- Le test muet `get_proposals_invoice_candidate_overrides_rule` (`reconciliation_rules_e2e.rs:961-1004`) **ne crée
  aucune facture** et n'éprouve donc pas ce que son nom annonce (C-29-3-6) : il crée une facture de reste égal au
  montant de la transaction (score ≥ 0,5) et assert que **aucune** candidate n'est `rule` et que `candidates[0]` est
  la facture. Son commentaire « Pas d'invoice créée » disparaît.

**AC4 — L'acceptation accepte la candidate affichée (vérification, et test).**

Établi au code : le client envoie l'identifiant de **la** candidate qu'il affiche (`ReconciliationProposals.svelte:208-
224`, même tableau `proposals` que le rendu `:338-368`) ; le serveur accepte cet identifiant (`accept_one_invoice`
`:1621`, `accept_one_rule` `:2736`) **sans** relire l'ordre — ni `accept_once`, ni `accept_batch` (`:1162`, `:1303`)
ne choisissent une candidate. Aucun changement d'API. Le **Pattern batch** est intact : aucun nouveau refus, aucune
`AppError` globale ajoutée. Test Vitest neuf dans `ReconciliationProposals.test.ts` : une proposition
`candidates = [règle, facture]` → la ligne affiche le badge *Règle* (`candidate-rule-badge`) et « Accepter » appelle
`acceptProposals` avec `{ type: 'rule', ruleId, counterpartyAccountId }` — jamais l'`invoiceId` de la seconde
(le cas symétrique `[facture, règle]` existe déjà, `:199`, à revérifier au rebase).

**AC5 — L'écran dit la priorité (C-29-3-4).**

Une phrase sous l'en-tête de la liste, **dans la branche « liste non vide »** (`{:else}` de
`ReconciliationProposals.svelte:279`), **avant** la table (`:301`) : `<p class="mb-2 text-sm text-text-muted"
data-testid="reconciliation-priority-hint">`, clé `reconciliation-labels-priority-hint`, repli = valeur fr-CH mot pour
mot (contrôle du même type que `ReconciliationProposals.test.ts:661-678`). Valeurs :

- **fr-CH** : `Une seule proposition est affichée par transaction : d’abord une facture dont le montant correspond exactement ou dont le numéro figure dans la référence du paiement, sinon la règle d’affectation qui correspond, sinon la facture la mieux notée.`
- **de-CH** : `Pro Transaktion wird ein einziger Vorschlag angezeigt: zuerst eine Rechnung, deren Betrag genau übereinstimmt oder deren Nummer in der Zahlungsreferenz steht, sonst die passende Zuordnungsregel, sonst die am besten bewertete Rechnung.`
- **it-CH** : `Per ogni transazione è mostrata una sola proposta: prima una fattura il cui importo corrisponde esattamente o il cui numero figura nel riferimento del pagamento, altrimenti la regola di assegnazione corrispondente, altrimenti la fattura con il punteggio migliore.`
- **en-CH** : `One proposal is shown per transaction: first an invoice whose amount matches exactly or whose number appears in the payment reference, otherwise the matching allocation rule, otherwise the best-scored invoice.`

Apostrophe typographique `’` en fr-CH, comme les clés voisines (`:1225`). Les gardes i18n du dépôt passent
(`npm run lint-i18n-ownership`, `i18n-libelle-en-dur.test.ts`, parité des quatre catalogues). Tests Vitest : phrase
rendue liste non vide ; **absente** liste vide et en erreur (elle parle de propositions affichées) ; repli égal au
catalogue fr-CH.

**AC6 — E2E Playwright (`frontend/tests/e2e/reconciliation-priority.spec.ts`, neuf).**

Montage par API comme `reconciliation-deleted-import.spec.ts` (règle par `POST /api/v1/reconciliation/rules`, relevé
importé, nettoyage en `afterAll`), fixtures **synthétiques** (`docs/agents-orchestration.md` § 8) : le cas 1 de l'AC3
de bout en bout — la ligne montre le badge *Règle* et la phrase de l'AC5, « Accepter » réussit, la ligne quitte la
liste, la facture reste ouverte (lue par l'API). **Captures clair et sombre** de la page avec la phrase, versées à
côté de la fiche au développement (recette visuelle de l'epic). E2E complet au **dernier commit de code**.

**AC7 — Manuel utilisateur (`docs/manual/fr/user-manual.tex`) et PDF.**

Inventaire **clos** des phrases rendues fausses par la story (`grep -n "aucune facture\|score faible\|aucune} facture"
docs/manual/fr/user-manual.tex` au sol, quatre sites — à regreper au rebase, la 29-2 réécrit le paragraphe voisin
`:2029`) :

| site | aujourd'hui | à écrire |
|---|---|---|
| `:2041` | « Quand aucune facture n'est candidate, la réconciliation propose à la place la règle » | la règle est proposée quand aucune facture n'atteint **50 %** ; sa place suit la priorité de `:2048-2049` ; « score de 100 % » conservé, avec « conventionnel : il n'est pas comparé à celui des factures » |
| `:2048-2049` | « la facture la mieux notée s'il y en a une » ; « une règle n'est affichée […] que s'il n'y a aucune facture candidate --- même une facture de score faible passe devant elle » | les trois rangs, dans les mots de la phrase de l'écran (AC5), et la mention que l'écran le rappelle au-dessus de la liste ; l'exemple du paiement à quelques centimes près avec le numéro en référence (rang 0) |
| `:2116` | « seulement quand aucune facture n'est candidate » | « seulement quand aucune facture n'atteint 50 % » + renvoi à la priorité (`\S` de l'acceptation) |
| `:2905` | « Une règle n'est d'ailleurs affichée que s'il n'y a aucune facture candidate. » | « Une règle n'est affichée en premier que si aucune facture n'est reconnue (montant exact ou numéro dans la référence). » |

Et une phrase pour l'angle mort de C-29-3-7, à `:2048-2049` : une facture dépassée de quelques centimes par le
paiement peut être affichée puis refusée en trop-perçu (renvoi à l'issue). Régénérer le PDF (`make fr` dans
`docs/manual/`, sous le verrou de gate) et le **contrôler aplati** :
`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -c "même une facture de score faible"`
→ **0**, et la nouvelle formulation présente. Manuels DE/IT/EN : vides (README seuls), rien à faire. Le manuel
d'administration ne parle pas des candidates (`grep -c candidate docs/manual/fr/admin-manual.tex` → 0 au sol).

**AC8 — CHANGELOG.**

Entrée `Fixed` de la v0.13.4 (section non datée en tête, ou à créer « Non publié ») : la proposition affichée suit une
priorité écrite, que l'écran rappelle ; une règle n'est plus cachée par une facture qui ne correspond qu'à quelques
centimes ou par le nom ([#527]).

**AC9 — Mutations (à exécuter, résultats au Change Log).**

| # | mutation | doit faire rougir |
|---|---|---|
| M1 | retirer l'appel `order_candidates` de `get_proposals` | AC3 cas 1 |
| M2 | `candidate_rank` : règle → 3 (derrière tout) | AC2 cas 1, 2, 4, 8 ; AC3 cas 1 |
| M3 | `candidate_rank` : règle → 0 (devant tout) | AC2 cas 3, 4 ; AC3 cas 2 |
| M4 | rang 0 sans `reference_score == 1.0` (montant seul) | AC2 cas 3, 4, 8 ; AC3 cas 2 |
| M5 | rang 0 sans `amount_score == 1.0` | AC2 cas 8 |
| M6 | `sort_by_key` → `sort_unstable_by_key` | AC2 cas 5 ou 6 (si elle survit, la déclarer **non observable** sur ces tailles avec la raison, ne pas la compter tuée) |
| M7 | tri par `score.total` décroissant à la place du rang | AC2 cas 3, 4 ; AC3 cas 2 |
| M8 | Svelte : phrase de l'AC5 déplacée hors de la branche non vide | Vitest « absente liste vide » |
| M9 | facture du test muet réparé (AC3) retirée | ce test |

Restauration : `git checkout -- <fichier>` **puis `touch <fichier>`** (`docs/agents-orchestration.md` § 7).

## Zones partagées avec la 29-2 (#529) — à relire au rebase

La 29-2 (fenêtre de dates tenant compte de l'échéance et/ou choix manuel d'une facture) n'a pas encore de fiche à la
date de celle-ci ; ce qui suit se déduit de #529 et du code. Zones **probables** de conflit :

| fichier | zone 29-2 (probable) | zone 29-3 | risque |
|---|---|---|---|
| `routes/reconciliation.rs` | passe 2 de `get_proposals` (`:633-674`, `InvoiceDateWindow::for_transaction`), peut-être une **seconde source** de factures (échéance, rappels) ou un **nouveau `CandidateType`** / un champ ; un éventuel type d'acceptation | fin de la passe 4 (`:722-826`), fonction et module de test neufs | **Moyen.** Si la 29-2 ajoute un variant, le `match` exhaustif de `candidate_rank` refuse de compiler : le classer (sans doute rang 2 pour une facture trouvée par l'échéance seule, à justifier) — c'est voulu (C-29-3-8). Si elle ajoute des candidates **après** la passe 4, `order_candidates` doit rester **le dernier** geste avant `ReconciliationProposal`. |
| `kesh-db/repositories/reconciliation.rs` | `InvoiceDateWindow`, `find_unpaid_invoices_for_window` | **aucune** | nul — la 29-3 ne touche pas `kesh-db` |
| `ReconciliationProposals.svelte` | boutons de ligne (`:370-398`), peut-être un bouton « choisir une facture » et un modal | phrase avant la table (`:279-301`) | faible ; si la 29-2 rend plusieurs candidates visibles, la phrase de l'AC5 est à revoir (« une seule proposition est affichée ») |
| `ReconciliationProposals.test.ts` | tests de boutons | tests neufs | faible (ajouts) |
| `user-manual.tex` | `:2029` (fenêtre de 30 jours), `:2060`, `:2068-2076` (affectation manuelle), `:2902`, `:2911`, `:1305` (rappels) | `:2041`, `:2048-2049`, `:2116`, `:2905` | **Moyen** : même sous-section ; les lignes bougent. Regreper les **valeurs** (« aucune facture », « score faible »), pas les numéros. |
| catalogues `messages.ftl` | clés `reconciliation-*` neuves | une clé `reconciliation-labels-*` | faible (ordre alphabétique : vérifier la parité) |
| `CHANGELOG.md` | entrée v0.13.4 | entrée v0.13.4 | trivial |

La 29-3 **ne change pas** la forme JSON de `GET /proposals` : une 29-2 qui en dépend n'a rien à adapter.

## Tasks / Subtasks

- [ ] T0 — Rebase sur `origin/main` (29-2 mergée) ; relire les lignes citées ; regreper « aucune facture » et
  « score faible » dans le manuel ; vérifier si la 29-2 a ajouté un `CandidateType` (AC1).
- [ ] T1 — Tests rouges d'abord : AC2 (unitaires), AC3 (HTTP, cas 1 rouge avant le correctif ; cas 2 vert avant et
  après — il fige ce qui ne doit pas régresser), Vitest AC4/AC5.
- [ ] T2 — `candidate_rank`, `order_candidates`, appel dans `get_proposals`, doc-comments (AC1).
- [ ] T3 — Réparer le test muet (AC3, C-29-3-6).
- [ ] T4 — Phrase de l'écran + quatre catalogues (AC5).
- [ ] T5 — Manuel + PDF contrôlé aplati (AC7) ; CHANGELOG (AC8).
- [ ] T6 — Mutations (AC9).
- [ ] T7 — E2E neuf + captures clair et sombre (AC6) ; gate complet backend et frontend, E2E complet, au dernier
  commit de code, sous le verrou ; ne déclarer que ce qui a tourné.

## Dev Notes

### Ce qu'il faut préserver

- Le **seuil** `INVOICE_OVERRIDE_THRESHOLD = 0.5` et son effet : la règle n'est pas même calculée quand une facture
  l'atteint (C-29-3-5 ; AC #114 de la 8-5b).
- Le tri de `propose_matches` (`matching.rs:97-105`) : la story ne le touche pas ; `kesh-reconciliation` reste
  hors périmètre.
- La forme JSON (`ReconciliationCandidate`, `:121-147`, struct `:126`), y compris le 1,0 conventionnel de la règle et son affichage
  « 100 % » (`ScoreBadge.svelte`) : changer l'affichage du score de la règle n'est pas demandé.
- L'acceptation (`accept_*`) : aucune ligne modifiée.
- La règle « une seule candidate visible » (KF-026, #76, ouverte) : la story ne la lève pas (C-29-3-9).

### Gates

`kesh-db` non touché : gate ciblé permis en boucle (`binary(reconciliation_e2e)`, `binary(reconciliation_rules_e2e)`,
tests unitaires de `kesh-api`), fmt et clippy workspace ; gate complet + E2E complet au dernier commit de code. Base
`kesh_293`, `kesh_e2e_293`, port **3044**, `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-29-3/target`.

### Pièges

- **Flottants** : `0,4 + 0,1` vaut exactement `0,5` en `f64` dans l'ordre d'évaluation de `matching.rs:80` — le cas
  « référence + contact » atteint le seuil et n'a pas de règle. Le test HTTP du cas 2 doit donc **éviter** que la
  contrepartie contienne le nom du client.
- **Contact** : `contact_score` est un « contient » bidirectionnel insensible à la casse (`matching.rs:148-161`) ; une
  contrepartie « Dubois » et un client « Dubois SA » donnent 1. Choisir des noms qui ne se contiennent pas pour le
  cas 2.
- **Règle sur un crédit** : les règles s'appliquent aux crédits (pas de filtre de signe, P-H7) ; la règle du test doit
  viser un compte **imputable**, actif, autre que la banque (`active_account_ids`, `:711-715`), sans quoi elle n'est
  pas proposée.

### References

- Route : `crates/kesh-api/src/routes/reconciliation.rs:586-845` (`get_proposals`), `:1621` (`accept_one_invoice`),
  `:2736` (`accept_one_rule`).
- Score : `crates/kesh-reconciliation/src/matching.rs:1-25` (doc du module), `:62-161`.
- Règles : `crates/kesh-reconciliation/src/rules.rs:85-95` (`first_matching_rule`).
- Trop-perçu : `crates/kesh-db/src/repositories/invoice_settlements.rs:160-184`.
- Frontend : `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte`, `ScoreBadge.svelte`.
- Manuel : `docs/manual/fr/user-manual.tex:2025-2060`, `:2114-2117`, `:2896-2912`.
- `CLAUDE.md` § *Pattern batch*, § *Un appariement automatique propose, il ne crée jamais* (le faux rattachement
  muet — argument de C-29-3-1).

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
