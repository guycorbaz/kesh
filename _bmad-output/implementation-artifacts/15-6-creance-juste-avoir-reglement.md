# Story 15.6 : La créance juste — l'avoir crédite le compte de la vente ; un règlement ne vise pas le compte qu'il solde

## Status

split

⛔ **Fiche d'index — elle ne porte ni critères ni tâches.** La story a été découpée **avant** sa
première rédaction complète, parce qu'elle franchissait d'emblée le critère de périmètre de la
§ *Règle de splitting préventif* du `CLAUDE.md` (choix **C-15-6-1** de
`epic-15-choix-autonomes.md`) ; la validation P1 en a détaché une quatrième (choix **C-15-6-9**). Les
quatre fiches filles sont `ready-for-dev`.

## Les quatre sous-stories

| | fiche | ce qu'elle porte | issues |
|---|---|---|---|
| **15-6a** | `15-6a-avoir-credite-la-creance-de-la-vente.md` | L'avoir contre-passe la créance **et l'arrondi** sur les comptes que l'écriture de vente a mouvementés, non ceux des réglages du moment. Lecteur partagé du compte de créance (trois copies de la même requête remplacées) et lecteur recoupé de la ligne d'arrondi ; **tous** les comptes que l'avoir écrit (créance, arrondi, TVA due, produits) verrouillés **en partage, avant l'exercice**, pour la course de lecture (un seul helper de verrou de liste, partagé avec la 15-5d), un compte de créance, d'arrondi ou de TVA due archivé refusé en le nommant (`ACCOUNT_ARCHIVED`, code de la contre-passation, libellé « Impossible d'émettre l'avoir ») ; la route est rejouée sur interblocage **par la 15-5e** (C54). Inventaire des sites qui relisent un compte après la pièce ; cycles connus nommés, sans prétention d'absence (#536) ; TVA due de l'avoir en angle mort tracé (#525). Manuel (§ *Avoirs*), CHANGELOG `[0.13.0]`. | `closes #473, closes #523` |
| **15-6b** | `15-6b-contrepartie-distincte-de-la-creance.md` | Un règlement — client ou fournisseur, par virement ou par compte interne, manuel, par rapprochement ou **par lot de paiement** — refuse une contrepartie égale au compte qu'il solde — comptes d'arrondi et de solde du reste compris, comparés par identifiant : code `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT`, `details.role` distinguant le compte choisi du compte désigné dans les réglages (deux remèdes, un message par rôle — six clés —, 4 locales ; le compte d'arrondi porte `rounding` quel que soit le geste) ; validation et avoir en angles morts écrits (#537, #525) ; helper commun dans `invoice_settlements` dont la 15-6d emprunte la comparaison ; trois clés plates pour les comptes désignés ; les lots refusent la facture **avant** de produire le pain.001, et un lot bloqué à la confirmation dit ses deux issues. Les deux écrans de règlement ne le proposent plus. Manuel, `docs/api-external.md`. | `refs #474` (angles morts : #537, #525) |
| **15-6c** | `15-6c-configuration-sans-ecriture-nulle.md` | La configuration ne prépare plus l'écriture nulle : le compte comptable d'un compte bancaire ne peut être le compte débiteurs ou créanciers des réglages, et réciproquement ; exemption « inchangé » ; contrôles dans la transaction, les deux gestes sérialisés (`LOCK IN SHARE MODE`). Écrans des comptes bancaires et des réglages (débiteurs, et créanciers depuis la 15-5d). Manuels, `docs/api-external.md`. | `closes #474` |
| **15-6d** | `15-6d-contrepartie-distincte-de-la-banque.md` | Le rapprochement manuel et l'acceptation par règle refusent une contrepartie égale au compte de la banque, avec le refus **et à la place** du flux ventilé existant ; une règle sur ce compte n'est plus proposée ; l'écran du rapprochement manuel ne la propose plus (filtre local) ; le chemin par règle contrôle aussi le compte de banque actif (même réponse sur les quatre chemins). Manuel, `docs/api-external.md`. | `closes #524` |

⚠️ **Ordre** : 15-6a → 15-6b (mêmes fichiers : `invoice_settlements_write.rs`, `reconciliation.rs`).
La **15-6c ne commence qu'après le merge de la 15-5b**, qui modifie les mêmes fonctions de
`crates/kesh-api/src/routes/bank_accounts.rs` (`validate_journal_account_id`, création, remplacement,
lien), le dépôt `crates/kesh-db/src/repositories/bank_accounts.rs` et le PUT des réglages
(`company_invoice_settings.rs`, AC10, AC11 et AC19 de la 15-5b), et après la 15-6b (fonctions d'écran
partagées, `ClaimSide`) et après la **15-5d** (compte créanciers exposé à l'écran des réglages). La
**15-6d** se rebase après la 15-5b (mêmes fonctions ; plus de dépendance d'ordre des refus depuis
C-15-6-23), **passe après la 15-6b**, dont elle emprunte la comparaison du helper commun de
`invoice_settlements` (C-15-6-28), et gagne à passer après la **15-5c**, qui réécrit le manuel du
rapprochement. Les 15-6a et
15-6b se rebasent aussi après les **15-5a à 15-5e** (mêmes fonctions : `accept_one_invoice`,
`settle_invoice`, `pay_in_tx`, `validate_invoice` ; la **15-5e**, réécrite par C54, rejoue sur
interblocage toutes les routes d'écriture — avoir compris — et réécrit le doc-comment canonique de
l'ordre des verrous, le « 5 bis » et le Pattern 5 : **une seule règle d'ordre des verrous pour
l'epic**, la sienne, que les 15-6 suivent sans la réécrire) — chacune a sa tâche T0. La 15-6b gagne à passer après la **15-5c**
aussi : le libellé de son code dans `failed[]` passe par le module des refus par lot que la 15-5c pose
(#492, choix C-15-6-12).

⚠️ **CHANGELOG** : la première story de la version mergée crée `## [0.13.0] — Non publié` en tête si
elle est absente ; les suivantes y ajoutent leur entrée (règle écrite dans chaque fiche).

## Pourquoi le découpage

Réunies, #473 et #474 touchent `kesh-db` (avoir, règlement client, règlement fournisseur, lots de
paiement, comptes bancaires, erreurs), `kesh-api` (rapprochement, comptes bancaires, réglages de
facturation, erreurs), `kesh-i18n`, plusieurs écrans (`SettleInvoiceDialog`, fiche de facture
fournisseur, lots, comptes bancaires, réglages), le manuel et le CHANGELOG — bien plus de cinq
modules. La seule dérogation codifiée (cycle de dépendances Cargo) ne s'applique pas. Chaque fille
reste à cinq modules métier au plus, la plomberie d'erreur et d'i18n suivant le geste qu'elle sert
(patron de la 15-5a). La validation P1 a ajouté à la 15-6b les lots de paiement et #524 ; avec les
deux, elle repassait au-dessus de cinq modules — d'où la 15-6d (finding F4).

## Décisions prises pour la story et où elles vivent

| choix | objet | fiche |
|---|---|---|
| C-15-6-1 | découpage en trois, ordre (ordre révisé par C-15-6-12) | toutes |
| C-15-6-2 | l'avoir lit la créance sur la vente (arrondi révisé par C-15-6-7, TVA par C-15-6-8) | 15-6a |
| C-15-6-3 | refus dédié, deux modes, après les contrôles existants ; arrondi et natures résolus par le type (révisé par C-15-6-18, C-15-6-26) | 15-6b |
| C-15-6-4 | filtre d'écran par rôle singleton, sans champ d'API nouveau (précisé par C-15-6-14) | 15-6b |
| C-15-6-5 | configuration : refus du couple dans les deux sens, exemption « inchangé » (course : révisé par C-15-6-13) | 15-6c |
| C-15-6-6 | ~~voisin hors périmètre~~ — **retiré par C-15-6-9** | — |
| C-15-6-7 | #523 absorbée : arrondi lu sur la vente, compte archivé refusé par `create_in_tx` (voie b) | 15-6a |
| C-15-6-8 | TVA due de l'avoir : angle mort tracé par #525, limite dite au manuel | 15-6a |
| C-15-6-9 | #524 → 15-6d, même refus que le flux ventilé | 15-6d |
| C-15-6-10 | lots de paiement : refus à la création, garde à la confirmation | 15-6b |
| C-15-6-11 | gardes sur les lecteurs ; lecteur sœur fournisseur | 15-6b |
| C-15-6-12 | #492 relève de la 15-5c | 15-6b, index |
| C-15-6-13 | contrôles dans la transaction, deux gestes sérialisés, ordre des verrous | 15-6c |
| C-15-6-14 | fonctions d'écran par ensemble d'ids, partagées 15-6b/15-6c (la 15-6d filtre localement, C-15-6-23) | 15-6b, 15-6c |
| C-15-6-15 | spec E2E de lien bancaire : 1100 → 1000 (références de fixture corrigées par C-15-6-22) | 15-6c |
| C-15-6-16 | comptes de la vente verrouillés, refus nommé `ACCOUNT_ARCHIVED` (révise la voie b de C-15-6-7) | 15-6a |
| C-15-6-17 | écrans sans archivés ; clés conformes au lint | 15-6b |
| C-15-6-18 | comptes d'écart comparés à la créance par identifiant (révise C-15-6-3) | 15-6b |
| C-15-6-19 | lots : refus contextualisé à la confirmation, achat malformé en `failed[]`, écran de création non filtré | 15-6b |
| C-15-6-20 | décompte des modules, signal D5 déclaré, pas de découpage | 15-6b |
| C-15-6-21 | `LOCK IN SHARE MODE` ; réglages : 409 d'abord ; code des réglages hors guide (révise C-15-6-13) | 15-6c |
| C-15-6-22 | appels directs au dépôt, tests étendus de la 15-5b, code mort supprimé, signal D5 | 15-6c |
| C-15-6-23 | égalité avec le compte de banque testée d'abord ; règle plus proposée ; filtre local (révise C-15-6-9) | 15-6d |
| C-15-6-24 | comptes de la vente verrouillés en partage, avant l'exercice (rectifie C-15-6-16) | 15-6a |
| C-15-6-25 | un seul vocabulaire de refus pour l'avoir : variante propre, code `ACCOUNT_ARCHIVED` (révise C-15-6-16) | 15-6a |
| C-15-6-26 | un code, un rôle (`details.role`), deux remèdes ; tables de refus du guide ; comparaison séparée de la construction du refus (révise C-15-6-18) | 15-6b, 15-6d |
| C-15-6-27 | `FOR UPDATE` de `before` testé, attente prouvée, pas de verrou pour une dé-liaison, manuel, guide | 15-6c |
| C-15-6-28 | compte de banque actif sur le chemin par règle, gardes ventilées figées, comparaison empruntée à la 15-6b (révise C-15-6-23 sur deux points, cf. C-15-6-31) | 15-6d |
| C-15-6-29 | verrou partagé étendu à tous les comptes écrits par l'avoir, rejeu de la route, ligne absente → `Invariant` (rectifie C-15-6-24) | 15-6a |
| C-15-6-30 | rôle `rounding` du compte d'arrondi quel que soit le geste ; validation et avoir en angles morts (#537, #525) ; liste vide conditionnée | 15-6b |
| C-15-6-31 | clôture de la validation ; C-15-6-28 révise C-15-6-23 ; texte du refus selon le champ | 15-6d |
| C-15-6-32 | alignement sur la 15-5e de C54 : rejeu de la route par la 15-5e, verrou pour la course de lecture seule, aucun réordonnancement, cycles nommés ; helper de verrou partagé avec la 15-5d ; TVA à source unique (révise C-15-6-29) | 15-6a |
| C-15-6-33 | alignement sur la 15-5e de C54 ; chaque comparaison suit sa lecture ; trois clés plates ; critère de T2 par nom sur liste fermée ; sujet de refus typé | 15-6b |
| C-15-6-34 | 15-6b close à la P6 ; produit de repli et validation d'achat tracés (#525, #537) ; comparaison après son compte et la créance ; manuel d'administration ; dispersion déclarée sans découpage | 15-6b, 15-6c |
| C-15-6-35 | 15-6a : un seul contrat pour le helper de verrou de liste ; critère DRY à exception écrite ; partenaires réels du cycle (iv) ; dépendance ferme à la 15-5d | 15-6a |

## Change Log

- 2026-10-08 — Spécification initiale (bmad-create-story, en autonomie), découpée d'emblée en
  15-6a / 15-6b / 15-6c. Choix C-15-6-1 à C-15-6-6 consignés. Validation : à lancer, fiche par fiche.
- 2026-10-08 — **Validation P1** des trois fiches (Sonnet 4.6, lentilles F et R). Bilan : 0 CRITICAL ;
  HIGH — 15-6a ×1 (#523), 15-6b ×4 (#524, `confirm_batch`, tests 5 et 7 irréalisables), 15-6c ×3
  (transaction, course, AC6 périmé). Tous appliqués ; 15-6d créée pour #524 ; choix C-15-6-7 à
  C-15-6-15 consignés. Détail dans le Change Log de chaque fiche.
- 2026-10-08 — **Validation P2** des 15-6a, 15-6b, 15-6c et **P1** de la 15-6d (Opus 5.5, lentilles
  R et F). Bilan : 0 CRITICAL ; HIGH — 15-6c ×1 (`FOR SHARE`, né de la remédiation P1, relevé par les
  deux lentilles) ; MEDIUM — 15-6a 3 + 2, 15-6b 3 + 5, 15-6c 3 + 3, 15-6d 3 + 3 (R + F, avant
  convergence). Tous appliqués ; choix C-15-6-16 à C-15-6-23 consignés. **Signaux de découpage
  déclarés au Project Lead** : 15-6b (huit modules au barème des fichiers) et 15-6c (HIGH né de la
  remédiation) — ni l'une ni l'autre découpée, arbitrage de Guy attendu. Détail dans le Change Log de
  chaque fiche.
- 2026-10-08 — **Validation P3** des 15-6a, 15-6b, 15-6c (Sonnet, lentilles R et F) et **P2** de la
  15-6d (Opus, lentilles R et F) ; remédiation Opus 5.5. Bilan (R + F, avant dédoublonnage) :
  0 CRITICAL ; 15-6a 1 HIGH, 6 MEDIUM, 10 LOW ; 15-6b 5 MEDIUM, 9 LOW ; 15-6c 3 MEDIUM, 11 LOW ;
  15-6d 3 MEDIUM, 12 LOW. Tous appliqués ; choix C-15-6-24 à C-15-6-28 consignés. Points saillants :
  15-6a — verrou des comptes de la vente en `LOCK IN SHARE MODE` **avant** l'exercice (le HIGH, né de
  la remédiation P2, inversait l'ordre canonique de `validate_invoice`), refus « Impossible d'émettre
  l'avoir » sous le code `ACCOUNT_ARCHIVED` ; 15-6b — `details.role` et clé distincte pour un compte
  désigné dans les réglages, helper commun ; 15-6c — `FOR UPDATE` de `before` épinglé par un test,
  attentes prouvées ; 15-6d — compte de banque actif contrôlé sur le chemin par règle, **la 15-6d passe
  désormais après la 15-6b**. **Signaux D5 déclarés au Project Lead** : 15-6a (sévérité remontée,
  HIGH né de la remédiation P2) et 15-6b (MEDIUM → MEDIUM, deux défauts nés de la remédiation P2) —
  aucune découpée ; 15-6c et 15-6d : non levés à cette passe (le signal P2 de la 15-6c reste
  déclaré). Toutes les fiches gardent des MEDIUM : **P4 à lancer** (complète pour la 15-6b ; ciblée
  possible pour la 15-6a). Décomptes : 15-6a 9 AC, 7 tâches, 14 tests ; 15-6b 13 AC, 10 tâches,
  21 tests ; 15-6c 11 AC, 9 tâches, 16 tests ; 15-6d 10 AC, 6 tâches, 12 tests.
- 2026-10-08 — **Validation P4** des 15-6a et 15-6b (Opus 5.5, lentilles R et F), **P4 ciblée** de la
  15-6c (Haiku 4.5) et **P3** de la 15-6d (Sonnet, lentilles R et F) ; remédiation Opus 5.5. Bilan (R +
  F, avant dédoublonnage) : 0 CRITICAL, 0 HIGH ; 15-6a 4 MEDIUM, 12 LOW ; 15-6b 5 MEDIUM, 10 LOW ;
  15-6c 0 ; 15-6d 10 LOW. Tous appliqués ; choix C-15-6-29 à C-15-6-31 consignés. Points saillants :
  15-6a — le verrou partagé couvre **tous** les comptes que l'avoir écrit (la TVA due et les produits
  demandaient leur S après l'exercice), la route `POST /api/v1/credit-notes` rejoue un 1213, cycle
  avoir ↔ rapprochement de la même facture tracé par **#536** ; 15-6b — rôle `rounding` pour le compte
  d'arrondi du solde du reste, « résolu par le type » remplacé par des angles morts (**#537**, #525),
  message de liste vide conditionné. **Signal D5 déclaré au Project Lead** pour la 15-6b (F1 recycle le
  discriminant de la P3 ; contenu, pas de découpage — un troisième recyclage en P5 sortirait l'AC3 bis
  en story propre) ; la 15-6a voit sa sévérité baisser (P3 1 HIGH → P4 0 HIGH). **Validations closes :
  15-6c et 15-6d** (0 au-dessus de LOW ; « 0 » de la passe ciblée de la 15-6c vérifié par
  l'orchestrateur). **P5 à lancer** pour les 15-6a et 15-6b (MEDIUM à cette passe). Décomptes : 15-6a
  9 AC, 7 tâches, 16 tests ; 15-6b 13 AC, 10 tâches, 22 tests ; 15-6c 11 AC, 9 tâches, 16 tests ;
  15-6d 10 AC, 6 tâches, 12 tests.
- 2026-10-08 — **Validation P5** des 15-6a et 15-6b (Sonnet, lentilles R et F ; remédiation Opus 5.5).
  Bilan (R + F, avant dédoublonnage) : 0 CRITICAL, 0 HIGH ; 15-6a 3 MEDIUM, 13 LOW ; 15-6b 6 MEDIUM,
  10 LOW. Tous traités ; choix C-15-6-32 et C-15-6-33 consignés. Fil conducteur : **la 15-5e**, que
  les deux fiches ignoraient, et **sa réécriture par C54** survenue pendant la remédiation (la défense
  contre l'interblocage est le rejeu) : dépendances « après 15-5a à 15-5e » ; la 15-6a s'appuie sur le
  rejeu de sa route par la 15-5e, garde son verrou pour la seule course de lecture, ne réordonne rien
  et ne prétend plus à l'absence de cycle ; la 15-6b garde l'ordre des lectures du code (chaque
  comparaison suit sa lecture), passe à trois clés plates, relève `sitesTotal`, et remplace le critère
  de T2 par un relevé par nom sur liste fermée. **Signal D5** : levé par la sévérité pour la 15-6b
  (MEDIUM → MEDIUM), non déclenché — le MEDIUM qui touche l'AC3 bis vient d'un changement extérieur
  (la 15-5e), pas d'un patch ; idem pour R5-1/R5-2 de la 15-6a. **P6 complète (Opus) à lancer** pour
  les deux. Décomptes : 15-6a 9 AC, 7 tâches, 18 tests ; 15-6b 13 AC, 10 tâches, 23 tests.
- 2026-10-08 — **Validation P6** des 15-6a et 15-6b (Opus 5.5, lentilles R et F ; prompts
  `15-6a-validate-prompt-p6.md`, `15-6b-validate-prompt-p6.md`). **15-6b** : 0 CRITICAL, 0 HIGH,
  0 MEDIUM, 18 LOW (R 11, F 7) — **validation close** (C-15-6-34) ; produit de repli et validation
  d'achat désormais tracés (#525, #537) ; manuel d'administration ajouté à l'AC11 (frontière de la
  15-6c alignée) ; dispersion (neuf modules au barème de la règle) déclarée au Project Lead, non
  découpée. **15-6a** : 0 CRITICAL, 0 HIGH, 5 MEDIUM (4 distincts), 12 LOW — remédiée (C-15-6-35) :
  un seul contrat pour le helper de verrou, critère DRY à exception écrite, doctrine résiduelle
  retirée, partenaires réels du cycle (iv), dépendance ferme à la 15-5d ; **signal D5 levé**
  (recyclage de la P5, contenu au texte de l'AC6) et dispersion déclarés, pas de découpage ; **P7
  complète (Sonnet) due**. Décomptes inchangés : 15-6a 9 AC, 7 tâches, 18 tests ; 15-6b 13 AC,
  10 tâches, 23 tests. À faire par l'orchestrateur : commentaire sur #536 (F6-6 de la 15-6a).
