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

## Questions à trancher à la spécification de la 15-1

Elles ne se devinent pas depuis `epics.md`, dont les critères d'acceptation tiennent en
six lignes. **Chacune change le schéma ou l'interface**, donc aucune ne peut attendre le
développement.

1. **Le lettrage est-il partiel ?** Un règlement de 1000 CHF pour deux factures de 600 et
   400 ; un acompte de 300 sur une facture de 1000. ⚠️ Un `lettering_code` posé sur des
   lignes entières ne sait pas exprimer un solde partiel — c'est le choix structurant de
   l'epic, et le PRD ne le tranche pas.
2. **Automatique ou manuel ?** La règle du dépôt *« un appariement automatique propose, il
   ne crée jamais »* s'applique-t-elle ici ? Elle a été écrite pour les contacts et les
   fournisseurs ; un lettrage proposé sur montant et date exact serait dans le même esprit.
3. **Que devient le lettrage à la clôture ?** `epics.md` dit : délettrage refusé sur
   exercice clôturé. Reste à décider ce qui se passe pour un lettrage **à cheval** sur deux
   exercices — cas courant d'une facture de décembre payée en janvier.
4. **Quel écran ?** Le lettrage se fait-il depuis le compte (vue « grand livre d'un
   compte »), depuis la facture, ou depuis un écran dédié ? Le PRD n'en dit rien.

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
