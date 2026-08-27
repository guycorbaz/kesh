# Epic 24 — Vague 1 : les livres justes

**Kickoff : 2026-08-26.** Première vague de correction du plan d'action issu de l'audit des
trois experts (`audit-experts-2026-08-26.md`). La vague 0 — *la vérité* — est close : la
documentation dit désormais ce que le code fait.

⚠️ **Cet epic est la contrepartie BMAD du jalon GitHub « Vague 1 »**, qui porte les
19 issues. Les deux suivis doivent rester alignés : les issues tiennent le détail et le
décompte, cet epic tient l'ordre et le raisonnement.

## Le défaut fondateur

**L'encaissement d'une facture client ne produit aucune écriture comptable** — ni le marquage
manuel (`mark_as_paid`), ni la réconciliation bancaire (`accept_one_invoice`), qui rapproche la
transaction de l'**écriture de vente** elle-même.

⚠️ **Le mode d'échec est silencieux, et c'est ce qui le rend grave.** Le bilan reste
**équilibré** — la partie double est respectée — mais deux postes sont faux du même montant en
sens inverse : **débiteurs surévalués**, **banque sous-évaluée**. Aucun contrôle interne ne
rougit.

Conséquences établies : le **rapprochement bancaire est impossible** (l'écart au 31.12 vaut le
total des encaissements clients depuis l'origine — c'est le premier contrôle d'un réviseur) ;
le **lettrage n'a rien à lettrer**, ce qui a fait geler l'epic 15 ; et le **seul chiffre du
tableau de bord est faux**.

⚠️ Et les deux chemins existants **s'excluent** : `post_manual` permet d'obtenir la bonne
écriture, mais `paid_at` n'est alors pas posé, donc la facture **reste relançable**. Ou bien la
comptabilité est juste et le client reçoit un rappel pour une facture payée, ou bien le suivi
est juste et les livres sont faux.

**Arbitrage du Project Lead** : *« Une facture est payée parce que la comptabilité le
montre. »* `paid_at` devient la **projection** d'une écriture de règlement.

## L'ordre, et pourquoi le grand livre passe devant

**Deux experts sur trois placent le grand livre en tête, indépendamment l'un de l'autre.**
L'argument de l'expert-comptable :

> *« C'est l'instrument qui rend les autres défauts visibles. »*

Sans lui, on ne peut ni détecter une écriture aberrante, ni mesurer l'écart entre le compte
débiteurs et la balance âgée, ni contrôler la TVA, ni préparer un bouclement — **ni vérifier
que l'écriture d'encaissement fonctionne** une fois livrée. C'est aussi du code purement
additif, sans risque de régression, là où l'encaissement touche des chemins vivants.

| # | story | objet |
|---|---|---|
| **24-1** | Grand livre | l'extrait de compte, et le filtre par compte (#373, #374) |
| 24-2 | Encaissement client | l'écriture `D banque / C débiteurs` (#371) |
| 24-3 | Règlement hors banque | espèces, compensation — le `settlement_type` des fournisseurs (#372) |
| — | *les 15 autres issues du jalon* | audit inaltérable et consultable, comptes de classe 9, immuabilité des écritures, export de souveraineté, balance à quatre colonnes… |

*Le découpage au-delà de 24-3 se fera au fil de l'avancement : les issues sont déjà écrites et
autonomes, il n'y a pas lieu de figer un ordre maintenant.*

## Ce qui est hors de cet epic

La **TVA** (vague 2) — son risque est réel mais il concerne l'assujetti, donc le profil
indépendant, deuxième priorité de cible. La **comptabilité personnelle** (vague 3) — budgets
pluriannuels et déductions, qui relèvent de la **construction** et non de la correction. Et le
**lettrage** (epic 15, gelé), qui reprendra une fois 24-2 livrée.

## Priorité de cible

Arbitrée le 2026-08-26 : **personnelle → indépendant → PME**.

⚠️ Cet ordre **inverse le degré de maturité constaté** par les experts. Il est légitime — Kesh
est d'abord l'outil de son auteur — mais il a une conséquence pour cet epic : les corrections
de la vague 1 servent **les trois profils**, ce qui est précisément ce qui les met en tête. Des
livres faux le sont pour tout le monde.

## Ce qui clôt cet epic

Le jalon GitHub « Vague 1 » à zéro issue ouverte. Et un critère de fond, qu'aucune issue ne
porte à elle seule : **le solde du compte bancaire de Kesh doit pouvoir être rapproché du
relevé de la banque.** Tant que ce rapprochement est impossible, la vague n'a pas atteint son
but, quel que soit le décompte des issues.
