# Story 25.4 : Propager le résiduel — découpée en 25-4-a / 25-4-b / 25-4-c

Status: split

**Issues : [#416], [#420]** (fiche de l'epic) — **et [#455], [#456]**, rattachées ici le 2026-09-26 :
les quatre portent sur **la même grandeur**, le montant restant dû d'une facture client, et les
quatre sont au jalon « Vague 1 », que l'Epic 25 doit mettre à zéro.

## Pourquoi découper — § *Règle de splitting préventif*

Inventaire des sites fait le 2026-09-26 (trois lectures du code, chaque site revérifié par
`grep -nF`, cf. § ci-dessous). Les quatre issues réunies touchent **six modules** : `kesh-db`
(résiduel, avoirs, rapprochement, échéancier), `kesh-report` (balance âgée),
`kesh-reconciliation` (score), `kesh-api` (routes factures, rapprochement, relances, erreurs),
`kesh-i18n`, `frontend` (fiche facture, échéancier, dialogue de règlement). Le seuil est **5** : la
règle impose le découpage **avant** `bmad-create-story`.

Le découpage suit l'ordre des dépendances, non celui des issues :

| Story | Issues | Ce qu'elle fait | Pourquoi à ce rang |
|---|---|---|---|
| **25-4-a** — le résiduel juste | [#455], [#456] | la formule canonique soustrait l'avoir **TTC** ; un avoir est refusé sur une facture **réglée, même en partie** ; le test de parité promis et jamais écrit | ⛔ **les deux suivantes réemploient la formule** : la propager avant de la corriger propagerait le défaut |
| **25-4-b** — le résiduel aux agrégats | [#416] | balance âgée, totaux de l'échéancier, **colonne « reste dû »** de l'échéancier et de son CSV, dialogue de règlement pré-rempli, **montant réclamé par les relances** | le cœur de #416 ; première utilisatrice des formes jointes |
| **25-4-c** — le résiduel au rapprochement | [#420] | filtre des candidats, score, re-score à l'acceptation, montant affiché dans la proposition | indépendante de 25-4-b, mais du même socle |

## Ce que l'inventaire a trouvé, et que les issues ne disaient pas

1. ⛔ **#420 a DEUX sites, et le premier est avant le score.** `find_unpaid_invoices_for_window`
   (`crates/kesh-db/src/repositories/reconciliation.rs:115`) filtre les candidats par
   `HAVING total_ttc BETWEEN ? - ? AND ? + ?` : le virement du solde d'une facture partiellement
   réglée n'est **même pas candidat**. Corriger `amount_score` seul (`matching.rs:68`) ne changerait
   rien. Troisième site : le re-score à l'acceptation, `reconciliation.rs:1196`
   (`invoices::total_ttc`).
2. ⛔ **Les relances réclament le TTC complet.** `render_reminder`
   (`crates/kesh-api/src/routes/invoice_email.rs:334`) : `total_due = ttc + other_fees + level_fee`.
   Une facture de 1 000.— réglée à 900.— est relancée pour 1 000.— plus les frais. Aucune issue ne le
   nomme ; c'est **le même symptôme** que #416 — la grandeur agrégée n'est pas le résiduel —, sur un
   document **envoyé au client**. Rattaché à 25-4-b.
3. ⚠️ **La relance joint le PDF de la facture**, dont la QR-facture porte le TTC
   (`invoice_email.rs:1140-1166`, `invoice_pdf_service::render`). Un client qui paie la relance en
   scannant la QR paie **à nouveau le montant entier**. Question posée à Guy (Q1 ci-dessous).
4. ⚠️ **#455 est latent sur les données neuves.** Un avoir est **total** et fait passer la facture à
   `cancelled` (`credit_notes.rs:560-563`) ; or les agrégats, le rapprochement et le règlement ne
   lisent que des factures `validated`. Le terme « avoir » du résiduel n'est donc **non nul que sur
   une facture annulée**, et le reste dû ne s'y affiche que si `amountSettled > 0` — c'est-à-dire
   **seulement dans l'état que #456 permet de produire**. Les deux issues se corrigent ensemble.
5. ⚠️ **Les formes jointes existent et ne servent nulle part.** `INVOICE_SETTLED_DERIVED_JOIN_SQL`
   et `INVOICE_CREDITED_DERIVED_JOIN_SQL` (`invoice_settlements.rs:40`, `:56`) n'ont aucun appelant,
   et le « test de parité » annoncé par le commentaire des lignes 27-29 **n'existe pas**. 25-4-b sera
   leur premier appelant : la parité doit être prouvée **avant**.
6. **Pourquoi aucun test n'a vu #455** : toutes les factures des tests de règlement sont à **0 % de
   TVA** (`crates/kesh-db/tests/invoice_settlement.rs:32`), où HT = TTC.
7. ⚠️ **Un test encode #456 comme attendu** : la fixture `monter`
   (`crates/kesh-db/tests/invoice_settlement.rs:937-990`) règle 40 sur 100 puis crée un avoir avec
   `.expect("avoir après règlement partiel — le vrai chemin l'accepte")`. Il **décrit le défaut** ;
   il se corrige, il ne se contourne pas.

## Hors périmètre des trois stories

- **L'imputation de l'écart** d'un règlement partiel (escompte, frais, perte) — [#384], 25-6.
- **Le côté fournisseur** : une facture fournisseur se règle en une fois, il n'y a pas de résiduel
  partiel (#416, § Portée).
- **L'avoir partiel et le remboursement** — v0.2, inchangé.

## Questions pour Guy

**Q1 — la QR-facture jointe à une relance.** Elle porte le TTC complet. Sur une facture
partiellement réglée, un client qui paie par la QR paie deux fois la part déjà réglée. Trois
issues : (a) la QR de la relance porte le **reste dû** — le PDF joint n'est alors plus tout à fait
la facture envoyée à l'origine, ce qui touche #387 (l'archivage du PDF) ; (b) la relance d'une
facture partiellement réglée joint un PDF **sans montant dans la QR** (la norme le permet : le
client saisit le montant) ; (c) laisser tel quel et le dire dans le texte de la relance. Ma
recommandation : (b), qui ne ment pas et ne modifie pas la facture. **À trancher avant 25-4-b.**

## Change Log

- **2026-09-26** — Découpée avant spécification (§ *Règle de splitting préventif*, six modules),
  sur inventaire vérifié. #455 et #456 rattachées (même grandeur, même jalon). Deux sites trouvés
  hors des issues : le filtre SQL des candidats du rapprochement (#420), et le montant des relances
  (rattaché à 25-4-b). 25-4-a spécifiée dans la foulée.

[#384]: https://github.com/guycorbaz/kesh/issues/384
[#387]: https://github.com/guycorbaz/kesh/issues/387
[#416]: https://github.com/guycorbaz/kesh/issues/416
[#420]: https://github.com/guycorbaz/kesh/issues/420
[#455]: https://github.com/guycorbaz/kesh/issues/455
[#456]: https://github.com/guycorbaz/kesh/issues/456
