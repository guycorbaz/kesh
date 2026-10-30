# Story v017-3c : la page de rapprochement signale une transaction qui semble déjà comptabilisée

Status: backlog

*Fiche courte, rédigée le 2026-10-30 (Opus 5.5, autonomie) à la validation P2 de la v017-3a, sur décision de
l'orchestrateur déclarée à Guy (C-v017-3-25, découpage). **Hors de la release 0.17.0.** Elle reprend ce que la v017-3a
portait jusqu'à sa validation P2 — ses anciens AC-10, AC-11, AC-12, AC-14, D6, D13, D15, mutations M13, M15, M21 et sa
question Q2 — avec les findings de la P2 qui s'y rattachent (R1, F5, F6, F14, F16 en partie). Elle sera complétée
(`bmad-create-story`) et validée quand elle sera planifiée. Registre : `v017-3-choix-autonomes.md` (C-v017-3-7,
C-v017-3-22, C-v017-3-23, C-v017-3-25). Issue : **#828** (ouverte le 2026-10-30 ; § *Issue*). Dépend de la **v017-3a** fusionnée.*

## Story

En tant que Comptable qui rapproche ses relevés, je veux que la page de rapprochement me **signale** une transaction en
attente qui ressemble à un mouvement déjà comptabilisé, et me **demande confirmation** avant que je l'accepte, l'affecte
ou l'éclate, afin de ne pas passer une seconde écriture pour un mouvement déjà aux livres.

## Origine

- **Précision du Project Lead sur #671 (2026-10-23)** : « Kesh doit afficher un avertissement et demander à
  l'utilisateur quoi faire : la garder en attente, la rejeter, la rattacher à l'écriture existante, etc. — jamais
  l'affecter d'office […]. Le danger visé : affecter manuellement une transaction déjà comptabilisée passe une seconde
  écriture. »
- La v017-3a (release 0.17.0) livre le **geste** — rattacher à une écriture existante — sans le **signal**. Le signal a
  été conçu dans la 3a puis retiré à sa validation P2 : ses deux formes (`alreadyBooked`, `matchedElsewhereCount`) y ont
  produit des défauts à chaque passe (C-v017-3-25).

## Périmètre (à détailler à la création de la fiche)

1. **`alreadyBooked`** dans `GET /reconciliation/proposals`, par transaction, champ additif : `settlementCount`
   (`find_link_candidates`, 29-8) et `entryCount` (`find_entry_candidates`, v017-3a), comptés par **les mêmes fonctions**
   que les listes du dialogue — aucune seconde règle (C-v017-3-7).
2. **`matchedElsewhereCount`** — le second côté d'un virement entre deux comptes propres (C-v017-3-22) : écritures dont le
   mouvement sur le compte du grand livre de `t` égale `t.amount`, non contre-passées, **rapprochées d'une transaction
   d'un compte bancaire dont le compte du grand livre diffère de celui de `t`** *(finding R1 = F6 de la P2 de la 3a : avec
   le prédicat « autre compte bancaire » seul, deux comptes bancaires qui partagent un compte du grand livre —
   `20260507200001_bank_account_journal_link.sql:21`, aucune contrainte `UNIQUE` ; `bank_accounts.rs:1786`, `sharing` —
   feraient conseiller de rejeter une transaction non comptabilisée)*. Un second mode du constructeur de la 3a, un test
   qui asserte que les deux textes ne diffèrent que par les prédicats voulus, un test « comptes partagés → non compté »,
   une mutation qui retire le prédicat.
3. **L'avertissement** sur la ligne (`data-testid="already-booked-warning"`), qui ne cite aucun numéro d'issue, renvoie
   au bouton existant de la ligne (pas de second bouton) :
   - si `settlementCount + entryCount > 0` : « Cette transaction semble déjà comptabilisée : {count} écriture(s) non
     rapprochée(s) du compte de la banque lui correspond(ent) — au même montant, ou d'au plus 5 centimes de moins pour
     un règlement de facture client *(F14 de la P2 : la 29-8 n'admet qu'un règlement inférieur, `user-manual.tex:2390`)*.
     Rattachez-la plutôt que de l'affecter : l'affecter passerait une seconde écriture. » ;
   - si `matchedElsewhereCount > 0` : « Une écriture du même montant est déjà rapprochée d'une transaction d'un autre de
     vos comptes bancaires : c'est probablement l'autre côté d'un virement entre vos comptes. Ne l'affectez pas :
     rejetez-la. » ;
   - *(F5 de la P2)* la première phrase **ne s'affiche pas** quand son compte est nul (jamais « 0 écriture(s) ») ; les
     deux comptes positifs : les deux phrases, dans cet ordre, et la consigne « rejetez-la » **ne s'applique qu'à la
     seconde** — texte du cas combiné à fixer à la création ; Vitest des quatre cas, avec leur texte attendu.
4. **Confirmation** avant *Accepter*, *Affecter manuellement* et *Éclater* quand la sélection contient une ligne signalée
   (`data-testid="already-booked-confirm"`, nomme les transactions, *Continuer quand même* / *Annuler*) ; *Rejeter* sans
   confirmation ; aucun refus serveur (un appel d'API direct reste libre — C-v017-3-7).
5. **Coût** (C-v017-3-23) : la somme par écriture parcourt tout l'historique du compte à chaque appel, jusqu'à trois fois
   par transaction de la page (500 au plus). Mesure sur un historique synthétique de 1 000, 5 000 et 20 000 écritures,
   500 transactions en attente, trois mesures par palier ; « mauvaise » pente : plus de 3 s à 20 000, ou croissance plus
   que linéaire ; dans ce cas, section *Performance debt* et issue, remède nommé « une lecture par page » (`SUM` par
   écriture filtré `HAVING movement IN (…)` sur les montants de la page, appariement en Rust). La mesure **précède**
   l'implémentation du remède.
6. **Textes** : manuel (§ *Réconciliation*, la consigne « rejetez-la » que la 3a laisse devient l'avertissement),
   `docs/api-external.md` (le champ additif), quatre catalogues i18n, captures (ligne signalée, second côté, cas combiné,
   confirmation), E2E (avertissement puis confirmation sur *Affecter manuellement*).

## Mutations reprises de la 3a

| # | Mutation | Doit rougir |
|---|---|---|
| ex-M13 | proposition : `entry_count` toujours 0 | test HTTP de `alreadyBooked` |
| ex-M15 | écran : confirmation sautée pour *Affecter manuellement* | Vitest de la confirmation |
| ex-M21 | `matchedElsewhereCount` : prédicat « compte du grand livre différent » retiré | test « comptes partagés » |

## Question au Project Lead (ex-Q2 de la 3a)

L'avertissement est un avis : *Accepter*, *Affecter manuellement* et *Éclater* restent possibles après une confirmation ;
rien n'est refusé. Préférez-vous qu'une ligne signalée ne propose plus rien, comme celle d'un lot de paiement en cours ?

## Issue

*Ouverte : **#828** (`enhancement`, P2). Texte d'origine ci-dessous.*

**Titre** : « Rapprochement : signaler une transaction qui semble déjà comptabilisée, et confirmer avant de l'affecter »

**Corps** :

> ## Constat
>
> Depuis la v0.17.0 (story v017-3a, #671), une transaction bancaire en attente se **rattache** à une écriture déjà
> passée — règlement fournisseur saisi depuis la fiche, écriture saisie au journal, règlement par compte interne sur le
> compte de la banque — par le dialogue « Rattacher à une écriture existante ». Mais rien ne **signale** une telle
> transaction sur la page de rapprochement : l'utilisateur doit penser à ouvrir le dialogue. S'il accepte une
> proposition, affecte ou éclate la transaction, il passe une **seconde** écriture pour un mouvement déjà aux livres
> (double comptage du compte de banque), sans que rien l'en avertisse. C'est le danger que nommait la précision du
> Project Lead sur #671 (2026-10-23).
>
> Le second côté d'un virement entre deux comptes propres est dans le même cas : après le rattachement d'un côté, l'autre
> ne porte aucun signal (le traitement des deux côtés est l'objet de #823).
>
> ## Attendu
>
> - Sur chaque ligne de la page de rapprochement, un avertissement quand la transaction a des écritures candidates au
>   rattachement, ou quand une écriture de même montant est déjà rapprochée sur un autre compte bancaire **à compte du
>   grand livre distinct** (le second côté d'un virement).
> - Une confirmation avant *Accepter*, *Affecter manuellement* et *Éclater* pour une ligne signalée ; aucun refus serveur.
> - Une mesure du coût de la page sur un historique volumineux, et une dette de performance si la pente est mauvaise.
>
> Conception reprise de la v017-3a (retirée de la release 0.17.0 à sa validation P2) : fiche
> `_bmad-output/implementation-artifacts/v017-3c-avertissements-propositions.md`.
>
> Refs #671, #823.

## Change Log

- 2026-10-30 — Fiche courte créée (Opus 5.5, autonomie) au découpage de la v017-3a (C-v017-3-25) ; statut `backlog`,
  hors de la release 0.17.0.
