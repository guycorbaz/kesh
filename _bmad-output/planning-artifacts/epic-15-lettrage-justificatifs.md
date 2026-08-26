# Epic 15 — Lettrage, justificatifs & compléments

**Statut** : 🚧 kickoff le 2026-08-24
**Cible release** : v0.12 (à confirmer)
**Origine** : `epics.md` § *Epic 14 : Justificatifs, Lettrage & Compléments v0.2* (renuméroté 15 à la rétrospective de l'Epic 5)
**FR couverts** : FR85, FR86 (lettrage) · FR63, FR64 (justificatifs) · FR87, FR81, FR74 (versioning, modèles, manuels)

**Arbitrage de Guy au kickoff** : *« le lettrage est indispensable à la mise en production »*.
Ce motif commande l'ordre des stories et tranchera les questions de périmètre.

## ⚠️ Ordre INVERSÉ par rapport au découpage d'origine

`epics.md` liste les justificatifs en 15.1 et le lettrage en 15.2. **Le lettrage passe
en premier**, sur l'arbitrage ci-dessus.

**Vérifié au sol, et c'est ce qui rend l'inversion possible** : les deux stories sont
**indépendantes**. Le lettrage touche `journal_entry_lines` ; les justificatifs touchent
un volume de fichiers et une table de liaison. Aucune ne bloque l'autre.

*(Précédent de méthode : l'Epic 16 avait lui aussi inversé son ordre au kickoff, mais pour
la raison opposée — une dépendance de schéma rendait la séquence d'origine impossible. Ici
il n'y a aucune contrainte technique : l'inversion sert la priorité de mise en production,
et rien d'autre.)*

## Ce qui existe déjà — relevé au sol le 2026-08-24, pas supposé

⚠️ **Trois rapprochements sont DÉJÀ en place, et aucun ne fait le travail du lettrage.**
Les confondre conduirait soit à réécrire ce qui existe, soit à croire le besoin couvert.

| dispositif | ce qu'il lie | table / colonne |
|---|---|---|
| Facture ↔ écriture de vente | la pièce et son écriture | `invoices.journal_entry_id` |
| Transaction bancaire ↔ écriture (Epic 8) | l'encaissement et son écriture | `bank_transactions.matched_entry_id`, `.status` |
| Marqueur de paiement | une facture réputée payée | `invoices.paid_at` |

**Ce qui manque, et que le lettrage seul apporte** : le rapprochement **ligne à ligne**,
c'est-à-dire l'appariement de la ligne « créance client » d'une écriture de vente avec la
ligne « créance client » de l'écriture d'encaissement qui la solde.

C'est cela qui permet de répondre à la seule question qui compte pour un compte client :
**qu'est-ce qui reste ouvert ?** `paid_at` dit qu'une facture est payée ; il ne dit pas
qu'un compte est soldé, ne gère pas les paiements partiels, ni un règlement couvrant
plusieurs factures.

**État du terrain** :

```
lettering_code : ABSENT — aucune migration, aucun code (grep sur crates/, 0 occurrence)
journal_entry_lines : id, entry_id, account_id, line_order, debit, credit, project_id
```

Tout est à construire, et rien n'est à défaire.

## Découpage

| Story | Objet | FR |
|---|---|---|
| **15-1** | **Lettrage** — code de lettrage sur les lignes, lettrer / délettrer, visibilité | FR85, FR86 |
| **15-2** | **Pièces justificatives** — attacher un fichier à une écriture, volume dédié, export | FR63, FR64 |
| **15-3** | Versioning des parseurs, modèles de documents, manuels embarqués | FR87, FR81, FR74 |
| **15-4** | Journaux personnalisables | — |

⚠️ **La 15-4 est spécifiée depuis le 2026-08-12** et ne l'était pas auparavant : elle
n'existait qu'en ligne de suivi, sans aucune spécification. Le trou avait été relevé à la
revue de projet — c'est un précédent à garder en tête pour les trois autres.

## Décisions du kickoff — arbitrages de Guy, 2026-08-24

Les quatre questions ouvertes sont tranchées. Elles sont **normatives** pour la 15-1.

### D1 — Le lettrage COMPLÈTE la réconciliation, il ne la remplace pas

⚠️ **Établi au sol avant l'arbitrage** : `accept_one_invoice`
(`kesh-api/src/routes/reconciliation.rs:1003-1313`) apparie déjà transaction bancaire ↔ facture, **propose et
fait valider**, puis pose `matched_entry_id`, `status = 'reconciled'` et `paid_at`. Le
chemin « facture réglée par virement importé » est donc **entièrement couvert**, et par le
geste même que Guy décrit pour le lettrage.

Le lettrage vise ce que ce chemin ne voit pas :

- les règlements **hors import bancaire** — espèces, compensation, virement non importé ;
- les comptes **fournisseurs** ;
- les **écritures manuelles** qui se soldent (acomptes, avances) ;
- et surtout la vue **« qu'est-ce qui reste ouvert sur ce compte ? »**, qu'aucun écran ne
  donne aujourd'hui.

⚠️ **Conséquence à ne pas découvrir en développement : la vue doit lire les DEUX
mécanismes.** Une facture réglée par la réconciliation porte `paid_at` **sans** être
lettrée. Si l'écran ne regardait que `lettering_code`, il l'afficherait comme **ouverte** —
c'est-à-dire qu'il mentirait précisément là où il doit informer. La définition de
« ouvert » est donc *ni lettré ni marqué payé*, et c'est un critère d'acceptation, pas un
détail d'implémentation.

### D2 — Un lettrage, une facture, un règlement

Pas de paiement **partiel** (facture réglée en plusieurs fois), pas de règlement **groupé**
(un virement pour plusieurs factures) — les deux hors périmètre au début.

⚠️ **Ce que cette borne coûte, écrit d'avance** : les factures relevant de ces deux cas
resteront affichées comme **ouvertes**. C'est acceptable parce que rien ne se perd en
silence — l'utilisateur les voit et sait pourquoi —, mais le règlement groupé est **courant
chez un client qui reçoit plusieurs factures par mois**. À rouvrir dès que l'usage réel le
demande, et le dire dans le manuel plutôt que de le laisser surprendre.

### D3 — Lettrer sur un exercice clôturé : OUI. Délettrer : NON

Le lettrage **ne modifie aucun montant** : il note qu'une créance est soldée. Il n'entre
donc pas en conflit avec l'immuabilité post-clôture, et c'est **ce qui rend le lettrage à
cheval réellement possible** — une facture de décembre payée en février se lettre après la
clôture de l'exercice.

Le délettrage reste refusé sur exercice clôturé, conformément à `epics.md`.

⚠️ **Le motif de Guy — « la clôture ne devrait se faire que lorsqu'il n'y a plus
d'écritures à passer » — ne suffit PAS à lui seul**, et il faut le dire : une créance
client ouverte au 31.12 est **normale**, elle figure au bilan. Attendre que tout soit réglé
pour clôturer reviendrait à ne jamais clôturer. C'est donc bien D3, et non l'ordre des
opérations, qui rend le cas à cheval possible.

### D4 — Écran dédié

Un écran de lettrage propre, sur le modèle de Bexio. ⚠️ **Risque à traiter à la
spécification** : il ressemblera à l'écran de réconciliation, qui propose déjà des
appariements. Deux écrans voisins qui font des choses différentes se confondent — leur
frontière doit être lisible **pour l'utilisateur**, pas seulement pour le développeur.

## Questions résiduelles pour la spécification

Elles ne se devinent pas depuis `epics.md`, dont les critères d'acceptation tiennent en
six lignes. **Chacune change le schéma ou l'interface**, donc aucune ne peut attendre le
développement.

1. **Sur quels critères Kesh propose-t-il ?** Montant exact et compte, référence de
   facture repérée dans le libellé, fenêtre de dates ? La réconciliation a déjà un jeu de
   critères éprouvé (`WINDOW_DAYS`, bornes sur la date de facture) — **s'en inspirer plutôt
   que d'en inventer un second**, et dire lequel dans la spec.
2. **Le lettrage porte-t-il sur la ligne ou sur l'écriture ?** `epics.md` prescrit
   `lettering_code` sur `journal_entry_lines`. À confirmer contre D2 : avec un
   appariement 1↔1, un code sur l'écriture suffirait peut-être — mais la ligne est le
   niveau juste dès qu'on rouvrira le partiel ou le groupé.
3. **Que voit-on dans l'écran dédié**, et comment se distingue-t-il de la réconciliation
   pour un utilisateur qui n'a pas lu le code ? (cf. D4)

## Risques

| # | Risque | Parade |
|---|---|---|
| R1 | **Le lettrage recoupe la réconciliation bancaire** (Epic 8) et l'on construit deux fois le même appariement. | Le § *Ce qui existe déjà* borne : la réconciliation lie une **transaction** à une **écriture** ; le lettrage lie deux **lignes**. À revérifier à la spécification, pas à supposer. |
| R2 | **Une migration sur `journal_entry_lines`** touche la table la plus centrale du produit. | Politique de migration (P1-P8) : `ADD COLUMN` nullable est non-breaking, donc pas de bump `min_required` ; ligne d'audit d'idempotence obligatoire (P5), triage `POST_RESTORE_BACKFILLS` (P7). |
| R3 | **La v0.11.1 tourne sur le NAS** et ce sera bientôt une comptabilité réelle. | Le jalon *« Première clôture d'exercice tenue dans Kesh »* est encore ouvert : **il n'y a pas de parc à protéger aujourd'hui**, et c'est ce qui rend cette migration moins risquée qu'elle ne le sera jamais. ⚠️ Ce raisonnement cessera de valoir dès la bascule. |
| R4 | Le périmètre des justificatifs (15-2) déborde sur la **stratégie de sauvegarde** — un volume de fichiers hors dump MariaDB. | À écrire dans la spec de la 15-2, pas à découvrir en développement : `epics.md` le signale déjà. |

## Ce que le kickoff N'a pas fait

⚠️ **La revue de projet n'a pas eu lieu.** Le `CLAUDE.md` impose l'ordre
`bmad-retrospective` → revue de projet, et la rétrospective de l'Epic 23 s'est close le
2026-08-23. La dernière revue date du **2026-08-19** — c'est elle qui avait déclenché la
publication de v0.10.0.

Elle regarde ce que ni `git log` ni les issues ne montrent : échéances, dépendances avec
d'autres dossiers, engagements pris ailleurs. **Ce plan est donc écrit sans elle**, et son
choix de priorité repose sur le seul arbitrage de Guy — ce qui est suffisant pour démarrer,
mais ne remplace pas la revue.
