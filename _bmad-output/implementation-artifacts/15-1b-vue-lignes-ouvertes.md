# Story 15.1b : La vue « ce qui reste ouvert » — et ce qu'elle prétend égaler

## Status

draft

## Story

**As a** indépendant ou fiduciaire qui tient ses comptes dans Kesh,
**I want** voir d'un coup d'œil ce qui reste ouvert sur un compte client ou fournisseur,
**so that** je puisse relancer les bons débiteurs, justifier le solde d'un compte, et clore
un exercice en sachant ce qu'il porte.

Deuxième des trois sous-stories issues du **split de la 15-1**. ⚠️ **Suppose la 15-1a
livrée** — elle lit la marque que celle-ci pose.

## Ce que cette story doit résoudre, et qui n'est pas ce qu'on croit

La 15-1 d'origine tenait « ouvert » pour une définition à écrire. La passe 3 a montré que
c'est une **question de fond non tranchée**, et qu'une implémentation fidèle à la spec
produisait un résultat faux **par trois chemins différents**.

### ⛔ Décision 1 — Lettrer ne calme aujourd'hui NI la balance âgée NI les relances

Le *so that* de cette story dit « relancer les bons débiteurs ». Or les deux dispositifs qui
relancent ne connaissent que `invoices.paid_at` :

```
crates/kesh-report/src/aged_receivables.rs:127      … AND i.paid_at IS NULL
crates/kesh-db/src/repositories/dunning_eligibility.rs:87   AND i.paid_at IS NULL
```

et `grep -rn "lettering" crates/ frontend/` rend **zéro occurrence** : rien ne les fera
changer d'avis.

⚠️ **Le scénario n'est pas théorique.** Facture réglée en espèces, lettrée. L'écran de
lettrage affiche « rien d'ouvert ». La **balance âgée** continue de la porter en 61-90 jours,
et le **moteur de relance envoie un rappel** — puis un deuxième. **Ce défaut-là n'est pas
muet : il est adressé au client.**

C'est le miroir exact de D4 : la 15-1 imposait à la vue de lettrage de lire `paid_at`, et
**rien n'imposait aux autres vues de lire la marque**. Un seul sens de la relation avait été
vu.

**Deux conduites, à arbitrer — c'est une décision de fond, pas d'implémentation :**

**(A)** Lettrer la ligne de créance d'une facture **pose `invoices.paid_at`** — source
unique, les trois vues s'accordent sans rien changer chez elles. ⚠️ **Mais `mark_as_paid`
n'a AUCUNE garde d'exercice** (cf. Décision 3), et ce chemin en hériterait.

**(B)** Le hors-périmètre est **assumé et énoncé**, au même titre que la borne du règlement
groupé : un critère dédié, un test, et une phrase à l'écran. Aujourd'hui, un développeur
fidèle à la spec n'implémentera **ni l'un ni l'autre**.

### ⛔ Décision 2 — Côté client, `paid_at` n'a aucune contrepartie comptable

`accept_one_invoice` **ne crée aucune écriture** — il met à jour deux tables. Le repository
l'écrit noir sur blanc :

```
crates/kesh-db/src/repositories/invoices.rs:1923
    /// **Ne crée AUCUNE écriture comptable** en v0.1. […] Ici `paid_at`
    /// est un simple marqueur opérationnel.
```

Or la validation de facture **débite la créance** (`invoices.rs:1368`).

⚠️ **Conséquence, et elle touche le *so that* de la story** : dix factures réglées par
virement importé laissent le compte 1100 avec **dix débits jamais crédités** — son solde est
de dix factures. Une vue qui applique « ouvert = ni lettré ni marqué payé » affiche **zéro
ligne ouverte**. « Justifier le solde d'un compte » n'est alors pas atteignable, et c'est le
chemin que la 15-1 qualifiait elle-même de **plus fréquent**.

**Deux conduites** : afficher côte à côte le solde du compte et le total des lignes ouvertes,
en **assumant l'écart et en le nommant** ; ou restreindre la vue aux comptes qui **ont** une
contrepartie comptable — les fournisseurs, cf. Décision 4 — et déclarer le cas client comme
angle mort. ⛔ **Ce qui n'est pas tenable, c'est de laisser croire au développeur que la vue
justifie le solde.**

### ⛔ Décision 3 — Le canal `paid_at` n'a aucune garde d'exercice

`mark_as_paid` (`invoices.rs:1926`) vérifie `status = 'validated'` et la version optimiste.
**Rien d'autre** — pas de `fiscal_years`, pas de `FiscalYearClosed`. Et il sert aussi au
**dé-marquage**, exposé par `POST /api/v1/invoices/:id/mark-paid` (`routes/invoices.rs:1033`).

⚠️ Exercice 2025 clos : le refus de délettrage de la 15-1a interdit de rouvrir une ligne.
Mais un **dé-marquage** rend cette même ligne « ouverte » au sens de la définition — le
résultat que le refus existe pour empêcher, obtenu par l'autre canal.

**Soit la garde d'exercice est étendue au marquage/dé-marquage** — changement hors périmètre,
à ouvrir en CR —, **soit la story écrit que sa garantie d'immuabilité est partielle**. La
laisser implicite fait croire à une protection qui n'existe pas.

### ⛔ Décision 4 — Le fournisseur n'est pas le symétrique du client, il est en avance

La 15-1 affirmait que « les comptes fournisseurs n'ont aucun équivalent de ce mécanisme ».
**C'est faux**, vérifié en passe 3 :

```
crates/kesh-db/src/repositories/supplier_invoices.rs:495   pub async fn pay(
crates/kesh-db/migrations/20260628000001_supplier_invoices.sql:44,47,67-68
    settlement_type VARCHAR(20) NULL,
    settlement_journal_entry_id BIGINT NULL,
    CHECK (settlement_type IS NULL OR settlement_type IN ('bank_transfer', 'internal_account'))
```

Le fournisseur a **davantage** que le client : un règlement **hors import bancaire compris**
(`internal_account`), qui pose `paid_at`, `status='paid'`, **et une vraie écriture de
règlement** référencée par une colonne dédiée.

Deux conséquences :

1. Sur le compte fournisseur, **les deux lignes à lettrer existent déjà** — `C 2000` à
   l'achat, `D 2000` au règlement, même compte, sens opposés, même TTC. Il n'y a rien à
   construire, seulement à afficher.
2. ⛔ **Une facture fournisseur payée est rattachée à DEUX écritures.** Une vue qui joint sur
   `purchase_journal_entry_id` seul masque la ligne d'achat et **laisse la ligne de règlement
   ouverte à jamais** : le compte 2000 afficherait un débit fantôme **par facture payée**,
   alors que son solde est zéro.

## Critères d'acceptation

⚠️ **Les critères ci-dessous sont écrits pour rester vrais quel que soit l'arbitrage des
quatre décisions**, sauf là où c'est indiqué. Ils ne peuvent pas tous être figés avant.

**AC1** — Depuis l'écran, l'utilisateur choisit un compte et voit ses lignes **ouvertes**,
avec le total.

**AC2** — ⚠️ Une facture réglée par la **réconciliation bancaire** n'apparaît **pas** comme
ouverte, bien qu'elle ne porte aucune marque de lettrage. « Ouvert » = **ni lettré, ni marqué
payé**. C'est le chemin le plus fréquent, et une vue qui ne regarderait que la marque
mentirait là où elle doit informer.

**AC3** — ⚠️ **La règle vaut pour les DEUX tables de factures**, `invoices` et
`supplier_invoices`, chacune avec son propre `paid_at`. **Un test par table.**

**AC3-bis** — ⚠️ **Et pour les DEUX écritures d'une facture fournisseur payée** *(relevé en
passe 3)* : l'achat (`purchase_journal_entry_id`) **et** le règlement
(`settlement_journal_entry_id`). Une jointure qui n'en voit qu'une laisse l'autre ouverte à
jamais. **Un test qui paie une facture fournisseur et vérifie que le compte 2000 n'affiche
plus AUCUNE ligne ouverte.**

**AC4** — L'écran indique **pourquoi** une facture manifestement réglée peut rester ouverte :
paiement partiel ou règlement groupé, hors périmètre. Sans cela, la vue paraît fausse là où
elle est seulement bornée.

**AC5** — *(dépend de la Décision 1)* La relation entre la marque de lettrage, la **balance
âgée** et le **moteur de relance** est **explicite** : soit lettrer pose `paid_at` et les
trois vues s'accordent, soit l'écart est énoncé à l'écran et couvert par un test qui
**documente** le comportement retenu. ⛔ **Le silence n'est pas une option** : il produit un
rappel envoyé à un débiteur soldé.

**AC6** — *(dépend de la Décision 2)* Ce que la vue prétend égaler est **écrit**. Si elle ne
justifie pas le solde du compte, elle ne le laisse pas croire.

**AC7** — *(dépend de la Décision 3)* La portée de la garantie d'immuabilité sur exercice
clos est **énoncée** : ce que le refus de délettrage protège, et ce que le canal `paid_at`
laisse passer.

## Tasks

- [ ] **T1** — Repository : lister les lignes ouvertes d'un compte. ⚠️ La requête implémente
      la définition d'« ouvert » — les jointures sur les **deux** tables de factures et les
      **deux** écritures fournisseur en font partie.
- [ ] **T2** — Route `GET` lignes ouvertes d'un compte.
- [ ] **T3** — Écran de consultation.
- [ ] **T4** — Tests : **AC2, AC3 et AC3-bis en priorité** — le piège de la définition, sur
      les deux tables **et** les deux écritures. Puis le test qui matérialise l'arbitrage
      d'AC5.
- [ ] **T5** — i18n : quatre locales, allowlist vide.
- [ ] **T6** — Manuel utilisateur : ce que la vue montre, et **ce qu'elle ne montre pas
      encore**.

## Dev Notes

⚠️ **Le sélecteur E2E ne se fige jamais sur un libellé traduit** — `data-testid` sans
exception (garde #326, son allowlist ne doit pas s'allonger).

⚠️ **La base de gate se remet à zéro AVANT le gate**, inconditionnellement (KF-039, #310).

## Change Log

### Création par split de la 15-1 — 2026-08-25

Issue du **split de la Story 15-1**. Recueille les deux HIGH de définition (**P3-3** relance
et balance âgée, **P3-4** les deux écritures fournisseur) et deux MEDIUM (**P3-6** `paid_at`
sans contrepartie comptable, **P3-8** le canal sans garde d'exercice).

⛔ **Quatre décisions de fond restent ouvertes et bloquent le développement.** Elles sont
posées en tête, chacune avec ses conduites possibles et son coût. Ce ne sont pas des
précisions manquantes : chacune, laissée au développeur, produit un résultat faux — et trois
d'entre elles le produisent **en silence**.
