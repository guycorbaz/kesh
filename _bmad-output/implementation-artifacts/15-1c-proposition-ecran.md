# Story 15.1c : Proposer un rapprochement, et l'écran qui le porte

## Status

draft

## Story

**As a** indépendant ou fiduciaire qui tient ses comptes dans Kesh,
**I want** que Kesh me propose les rapprochements évidents et me laisse lettrer à la main les
autres, depuis un écran dédié,
**so that** je solde mes comptes sans ressaisir ce que le logiciel voit déjà.

Troisième des trois sous-stories issues du **split de la 15-1**. ⚠️ **Suppose 15-1a livrée**
(elle pose la marque) **et 15-1b spécifiée** (la vue et sa définition d'« ouvert »).

## Décisions héritées, et la correction majeure de la passe 1

### D5 — Kesh propose, l'utilisateur valide

Conforme à la règle du dépôt *« un appariement automatique propose, il ne crée jamais »*.
Aucun lettrage ne s'écrit sans validation humaine.

**Les critères se reprennent de la réconciliation plutôt que de s'inventer** — ils y sont
éprouvés depuis l'Epic 8. ⚠️ **Mais ils ne se transposent PAS tous, et les reprendre en bloc
casserait trois des quatre cas que le lettrage existe pour couvrir** *(relevé en passe 1,
vérifié au sol)*.

**Ce qui se reprend — les critères STRUCTURELS, seuls universels :**

| critère | valeur |
|---|---|
| même compte | identité stricte |
| sens opposés | débit ↔ crédit |
| fenêtre de dates | ⚠️ voir la réserve ci-dessous |
| montant | **égalité STRICTE** — voir la réserve ci-dessous |

**Ce qui NE se reprend PAS — les filtres propres à la facture client :**

⚠️ `status = 'validated'` et `paid_at IS NULL` interrogent la table `invoices`. Or **trois
des quatre cas visés n'ont aucune facture `validated` derrière la ligne à apparier** : les
comptes **fournisseurs** (`supplier_invoices`), deux **écritures manuelles** qui se soldent,
et une facture **annulée par un avoir** — qui passe à `status = 'cancelled'`
(`credit_notes.rs:563`, `supplier_invoices.rs:811`), alors que la paire
facture/contre-passation est le cas de lettrage **le plus propre qui soit**.

Et le lien manque : **`journal_entry_lines` ne porte ni `invoice_id` ni
`supplier_invoice_id`** — le seul lien va de `invoices.journal_entry_id` vers l'**entête**
d'écriture. Un moteur ligne-à-ligne ne peut pas s'appuyer sur le statut de facture sans une
jointure que le schéma ne permet pas.

**Ces filtres restent légitimes pour TRIER les suggestions du cas facture client impayée ;
ils ne doivent pas conditionner l'ÉLIGIBILITÉ d'une paire de lignes.**

### D6 — Écran dédié, et sa frontière avec la réconciliation doit être lisible

L'écran ressemblera à celui de la réconciliation. Leur frontière doit être lisible **pour
l'utilisateur**, pas seulement dans le code : deux écrans voisins qui font des choses
différentes se confondent.

## Deux réserves ouvertes, à arbitrer avant le développement

### ⛔ Réserve 1 — La tolérance de montant, et les frais bancaires

La réconciliation tolère **5 centimes** (`AMOUNT_TOLERANCE_HUNDREDTHS = 5`,
`kesh-api/src/routes/reconciliation.rs:60`), précisément pour absorber les frais bancaires.
Mais la borne « un lettrage = une facture = un règlement » impose l'**égalité stricte** :
lettrer 1000 avec 300 prétendrait qu'une créance est soldée.

Reprendre la tolérance produirait un système qui **propose ce qu'il refuse ensuite**. La spec
tranche donc pour l'**égalité stricte des deux côtés**.

⚠️ **Sa conséquence doit être vue avant le développement** : un règlement amputé de frais
bancaires ne sera **jamais lettrable**, et la facture restera affichée ouverte. Conduite
alternative, si l'arbitrage change : garder la tolérance au **classement** des suggestions,
**jamais** au filtre d'éligibilité.

### ⛔ Réserve 2 — La fenêtre de 30 jours tue le cas que AC2 exige nommément

`WINDOW_DAYS = 30` (`reconciliation.rs:55`) a un sens d'origine **borné** : le code note
lui-même que les *« paiements tardifs > 30 j sont reportés Story 8-5 manual »* (l. 1125).

⚠️ Transposée au lettrage, elle **écarte la contre-passation** — le cas que D5 appelle « le
plus propre qui soit ». L'écriture d'annulation fournisseur est **datée du jour**
(`supplier_invoices.rs:786-795`), pas de la facture. Une facture de mars annulée en novembre
forme une paire parfaite qui ne serait **jamais proposée**.

⚠️ **Et le test d'AC2 passerait quand même** — vert, si son auteur date les deux pièces à
moins de 30 jours. Un test qui ne dit rien du cas réel. Même effet sur un règlement client à
45 jours, c'est-à-dire sur le débiteur qu'on veut relancer.

**Deux conduites** : une fenêtre **distincte et justifiée** pour le lettrage — l'exercice
comptable est le candidat naturel —, ou la fenêtre au **classement** et non au **filtre**,
exactement la conduite alternative de la Réserve 1.

## Critères d'acceptation

**AC1** — L'utilisateur peut lettrer deux lignes **manuellement**, sans proposition.

**AC2** (porte **D5**) — Kesh **propose** des rapprochements selon les critères
**structurels** de D5. Aucun lettrage n'est écrit sans validation explicite.

⚠️ **La proposition ne filtre PAS sur le statut de la facture.** **Un test nommé par cas** :
une paire **facture/avoir** (`status = 'cancelled'`), une paire sur un **compte
fournisseur**, et une paire d'**écritures manuelles** sans facture doivent chacune être
proposées. ⚠️ **Le test de la contre-passation date les deux pièces à plus de 30 jours
d'écart** — sinon il passe sans rien prouver (cf. Réserve 2).

**AC3** — Le lettrage est **refusé si les deux montants ne sont pas égaux**, et **le message
de refus nomme la cause** — *« les montants diffèrent ; le lettrage partiel n'est pas encore
géré »* —, sinon l'utilisateur conclut à un défaut.

**AC4** — La marque est **visible** sur la ligne, dans le détail d'écriture et dans l'écran
dédié ; les lignes partageant une marque sont identifiables entre elles.

**AC5** (porte **D6**) — ⚠️ **L'écran énonce sa frontière avec la réconciliation bancaire,
pour l'UTILISATEUR.** Deux exigences **distinctes** : **(1)** un texte **visible** — bandeau
ou aide contextuelle — dit ce que cet écran fait et ce qu'il ne fait pas ; **(2)** le test
E2E l'atteint par un `data-testid` stable et **jamais** par son libellé traduit. Un
`data-testid` ne satisfait pas (1) : il est invisible.

## Tasks

- [ ] **T1** — Moteur de proposition (D5). ⚠️ **Ne pas dupliquer** les critères structurels
      de la réconciliation : extraire si nécessaire. Et **ne pas reprendre** ses filtres de
      facture.
- [ ] **T2** — Routes de proposition.
- [ ] **T3** — Écran dédié, avec la frontière énoncée (AC5).
- [ ] **T4** — Tests : **AC2 en priorité**, ses trois cas nommés, avec l'écart de dates
      réaliste sur la contre-passation. Puis AC3 et AC5.
- [ ] **T5** — i18n : quatre locales dès l'écriture, allowlist vide.
- [ ] **T6** — Manuel utilisateur : ce que le lettrage fait, et **ce qu'il ne fait pas
      encore** — le partiel et le groupé.

## Dev Notes

⚠️ **Le sélecteur E2E ne se fige jamais sur un libellé traduit** — `data-testid` sans
exception (garde #326).

⚠️ **Un E2E n'est pas un test comme un autre** : c'est le seul qui vérifie qu'une valeur
traverse réellement la frontière HTTP.

## Change Log

### Création par split de la 15-1 — 2026-08-25

Issue du **split de la Story 15-1**. Recueille la correction majeure de la passe 1 sur D5
(les filtres de facture qui excluaient trois cas sur quatre) et le MEDIUM **P3-7** de la
passe 3 (la fenêtre de 30 jours qui tue la contre-passation).

⛔ **Deux réserves restent ouvertes** : la tolérance de montant face aux frais bancaires, et
la fenêtre de dates. Toutes deux sont tranchées **par défaut** dans la spec, avec leur
conduite alternative nommée — elles se changent d'un mot tant que le développement n'a pas
commencé.
