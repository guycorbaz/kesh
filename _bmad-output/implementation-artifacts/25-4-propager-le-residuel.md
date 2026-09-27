# Story 25.4 : Propager le résiduel — découpée en 25-4-a / 25-4-b / 25-4-c / 25-4-d

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

Le découpage (quatre stories depuis le 2026-09-27) suit l'ordre des dépendances, non celui des issues :

| Story | Issues | Ce qu'elle fait | Pourquoi à ce rang |
|---|---|---|---|
| **25-4-a** — le résiduel juste | [#455], [#456] | la formule canonique soustrait l'avoir **TTC** ; un avoir est refusé sur une facture **réglée, même en partie** ; le test de parité promis et jamais écrit | ⛔ **les deux suivantes réemploient la formule** : la propager avant de la corriger propagerait le défaut |
| **25-4-b** — le résiduel aux agrégats | [#416] | balance âgée, totaux de l'échéancier, **colonne « reste dû »** de l'échéancier et de son CSV, dialogue de règlement pré-rempli, **montant réclamé par les relances** | le cœur de #416 ; première utilisatrice des formes jointes |
| **25-4-c** — le résiduel au rapprochement | [#420] | filtre des candidats, score, re-score à l'acceptation, montant affiché dans la proposition | indépendante de 25-4-b, mais du même socle |
| **25-4-d** — solder le reste | [#384] | imputer l'écart d'un règlement partiel — **perte sur débiteur**, escompte, frais bancaires — pour clore une facture partiellement réglée ; la part de **TVA** réduit la TVA due, seule la part HT va au compte de perte | ramenée de la 25-6 le 2026-09-27 (Guy) : elle repose entièrement sur le résiduel juste de 25-4-a |

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
4. ⚠️ **#455 est masqué à l'écran — mais pas à l'API.** *(Corrigé en validation P1 de 25-4-a : la
   première rédaction disait « latent ».)* L'avoir est **total** et fait passer la facture à
   `cancelled` (`credit_notes.rs:560-575`) ; le terme « avoir » n'est donc non nul que sur une
   facture annulée. L'**écran** n'y affiche le reste dû que si `amountSettled > 0` — l'état que #456
   permet de produire. Mais `GET /invoices/{id}`, ouverte aux clés API, rend `amountDue` **sans
   condition de statut** : sur toute facture créditée à TVA non nulle, il vaut la TVA. Les deux
   issues se corrigent ensemble parce qu'elles portent sur la même grandeur et le même état.
5. ⚠️ **Les formes jointes existent et ne servent nulle part.** `INVOICE_SETTLED_DERIVED_JOIN_SQL`
   et `INVOICE_CREDITED_DERIVED_JOIN_SQL` (`invoice_settlements.rs:40`, `:56`) n'ont aucun appelant,
   et le « test de parité » annoncé par le commentaire des lignes 27-29 **n'existe pas**. 25-4-b sera
   leur premier appelant : la parité doit être prouvée **avant**.
6. **Pourquoi aucun test n'a vu #455** : toutes les factures des tests de règlement sont à **0 % de
   TVA** (`crates/kesh-db/tests/invoice_settlement.rs:32`), où HT = TTC.
7. ⚠️ **Un test encode #456 comme attendu** : la fixture `monter`
   (`crates/kesh-db/tests/invoice_settlement.rs:937-1019`) règle 40 sur 100 puis crée un avoir avec
   `.expect("avoir après règlement partiel — le vrai chemin l'accepte")`. Il **décrit le défaut** ;
   il se corrige, il ne se contourne pas.

## ✅ Arbitrages du 2026-09-27 — l'avoir sur facture encaissée, et la perte

- **L'avoir reste refusé sur une facture réglée, en tout ou en partie** (25-4-a). Un avoir sur
  facture encaissée est un geste normal, mais il laisse au client un **crédit** que Kesh ne sait ni
  montrer, ni rembourser, ni imputer : l'autoriser seul reproduirait la dette muette de #456. Sa
  levée est tracée par **[#471]** (crédit visible, remboursement ou imputation).
- **#384 est ramenée dans la 25-4, en 25-4-d**, pour clore une facture partiellement réglée dont le
  solde ne sera pas payé (perte sur débiteur), ou réglée sous déduction d'un escompte ou de frais.
  ⚠️ La part de TVA du montant passé en perte **réduit la TVA due** (correction de l'impôt sur
  créance irrécouvrable) ; seule la part HT va au compte de perte.

## Hors périmètre des quatre stories

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

### ✅ Arbitrage du 2026-09-26 (Q1) — (b), et le reste à payer dit au client

*« b, mais indiquer au client ce qu'il reste à payer. »* (Guy)

Pour 25-4-b, donc :

- la relance d'une facture **partiellement réglée** joint un PDF dont la QR-facture **ne porte pas
  de montant** — le client le saisit ; une facture sans règlement garde sa QR à montant ;
- la relance **dit ce qu'il reste à payer** : `{totalDue}` devient **reste dû + frais**, et non plus
  TTC + frais (`invoice_email.rs:334`) — c'est le montant que le client doit saisir.

- ✅ **Le reste à payer figure AUSSI sur le PDF joint**, à côté de la QR sans montant (Guy,
  2026-09-26) : *« sinon le client ne peut pas savoir ce qu'il reste à payer sans recherche et
  calculs »*. Le PDF d'une relance de facture partiellement réglée n'est donc plus exactement la
  facture d'origine — à articuler en spécification avec #387 (l'archivage du PDF envoyé).

- ✅ **Recadrage (Guy, 2026-09-27)** : *« en fait c'est un rappel pour un montant partiel. »* Le
  document joint à une relance de facture partiellement réglée n'est pas la facture réémise : c'est
  **un rappel pour le montant restant**. L'objection « le PDF n'est plus la facture d'origine »
  tombe donc — ce n'est pas la facture —, et #387 ne s'en trouve pas touché. ⚠️ **Point ouvert
  posé à Guy** : dès lors, la QR du rappel doit-elle porter le **reste à payer** plutôt que rester
  sans montant ?
- ✅ **Tranché (Guy, 2026-09-27)** : *« si c'est un rappel pour facture impayée, on met le montant
  total de la facture, et si c'est un montant partiel, on met le montant à payer : on traite les
  deux types de rappel de la même manière. »* Une seule règle : **la QR du rappel porte le reste
  dû** — qui vaut le TTC sur une facture sans règlement. L'option (b) « sans montant » est
  **abandonnée**. Guy suggère aussi de rappeler **le numéro de la facture d'origine** : la QR le
  porte déjà (référence QRR dérivée de la facture, ou message non structuré = numéro,
  `invoice_pdf_service.rs:218-250`) et **doit le garder à l'identique** — c'est ce que lit le
  rapprochement ; le PDF du rappel le nomme en toutes lettres, avec montant initial, déjà réglé,
  reste à payer.
- ✅ **Les frais restent configurables, et à zéro rien ne s'affiche** (Guy, 2026-09-27) : *« ils
  doivent être configurables : si 0, ne rien afficher. »* Les frais sont déjà réglés par niveau de
  rappel (`dunning_levels.fee_amount`) ; le PDF et le texte du rappel n'ont **aucune ligne de
  frais** quand le montant est nul — ni « frais : 0.00 », ni total qui les mentionne.
- ⚠️ **Point ouvert pour la spécification de 25-4-b — les frais de rappel.** Le texte réclame
  aujourd'hui `TTC + frais` (`invoice_email.rs:334`), mais les frais ne semblent **pas
  comptabilisés** (aucune écriture dans `invoice_reminders.rs`, à vérifier). Si la QR portait
  « reste dû + frais », le virement du client **dépasserait le reste dû** et serait refusé comme
  trop-perçu, au rapprochement comme au règlement manuel (`RECONCILIATION_OVERPAYMENT`). À établir
  avant de fixer le montant de la QR.

⚠️ **Ne pas contester l'arbitrage en revue** : en contester la mise en œuvre.

## Change Log

- **2026-09-27** — Arbitrages : avoir sur facture réglée refusé en 25-4-a, levée tracée par #471 ; #384 ramenée en 25-4-d.
- **2026-09-27** — Frais de rappel : configurables, aucune ligne affichée à zéro.
- **2026-09-27** — Q1 retranchée : la QR du rappel porte le reste dû (le TTC s'il n'y a aucun règlement) ; numéro de facture conservé ; point ouvert sur les frais.
- **2026-09-27** — Recadrage de Q1 : le document est un rappel pour un montant partiel, pas la facture réémise.
- **2026-09-26** — Complément de Q1 : le reste à payer figure aussi sur le PDF joint.
- **2026-09-26** — Q1 tranchée par Guy : QR sans montant pour une facture partiellement réglée, et le reste à payer dit au client (rattaché à 25-4-b).
- **2026-09-26** — Découpée avant spécification (§ *Règle de splitting préventif*, six modules),
  sur inventaire vérifié. #455 et #456 rattachées (même grandeur, même jalon). Deux sites trouvés
  hors des issues : le filtre SQL des candidats du rapprochement (#420), et le montant des relances
  (rattaché à 25-4-b). 25-4-a spécifiée dans la foulée.

[#384]: https://github.com/guycorbaz/kesh/issues/384
[#471]: https://github.com/guycorbaz/kesh/issues/471
[#387]: https://github.com/guycorbaz/kesh/issues/387
[#416]: https://github.com/guycorbaz/kesh/issues/416
[#420]: https://github.com/guycorbaz/kesh/issues/420
[#455]: https://github.com/guycorbaz/kesh/issues/455
[#456]: https://github.com/guycorbaz/kesh/issues/456
