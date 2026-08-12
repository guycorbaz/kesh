# Story 22.3 : Fusionner deux contacts en doublon

## Status

en-veille

⚠️ **Story NON ENGAGÉE, et le motif tient en une phrase : il n'y a rien à réparer.**

Kesh est déployé mais **ne tient pas encore les comptes réels** — le jalon « Première clôture d'exercice tenue dans Kesh » est ouvert. Il n'existe donc **aucun parc de doublons**. Une fonction de fusion est une **réparation** ; la spécifier maintenant reviendrait à traiter un cas dont on ne connaît ni la fréquence ni la forme.

**L'effort va à la prévention** — Story **22-2** (#301) : signaler un contact proche **avant** l'enregistrement. Elle coûte le même prix aujourd'hui qu'elle coûtera plus tard, mais elle **évite** la dette au lieu de la rembourser.

Cette story est conservée parce que son travail d'analyse est fait et vérifié — le rayon d'impact mesuré, le renversement de D1 argumenté sur le code, le verrou d'exercice vérifié. Le jour où des doublons réels apparaîtront, il n'y aura qu'à reprendre. *(Arbitrage de Guy, 2026-08-12.)*

## Story

**As a** indépendant ou fiduciaire dont le carnet d'adresses porte le même client deux fois,
**I want** réunir les deux fiches en une seule, historique compris,
**so that** mon carnet dise la vérité et que tout ce qui concerne ce client se lise au même endroit.

Ferme **#300**. Troisième story de l'**Epic 22 « Technical Debt Closure »**, à la suite de la **22-1** dont elle est le prolongement — et **en veille** derrière la **22-2**, qui prévient ce qu'elle répare.

## Contexte

Un même client finit par exister deux fois — saisi deux fois, importé deux fois, ou créé par deux personnes. Rien aujourd'hui ne permet de les réunir : on archive l'un à la main, et tout ce qui pendait à lui reste où il était.

Le cas devient plus fréquent avec le **numéro de client** livré en 16-3b. Deux fiches peuvent porter des numéros **identiques à l'œil** et distincts pour la base — composition Unicode, caractère invisible encastré, collation (#294, #295). La **Story 22-1** ferme cela pour l'avenir, mais le parc existant peut déjà en contenir : il faut alors pouvoir les réunir.

## Décisions

**D1 — La fusion repointe TOUT, historique compris.** *(arbitrage de Guy, 2026-08-12 — décision **renversée** le même jour, cf. encadré)*

Les factures, avoirs, factures fournisseurs et personnes de contact du perdant passent au survivant. Le perdant, vidé, est **archivé**.

> ⚠️ **Cette décision a d'abord été prise à l'envers, et le motif de son renversement est le cœur de la story.**
>
> La première rédaction posait « on ne fusionne que le vivant », au nom de l'immuabilité comptable : repointer `contact_id` sur une facture validée aurait modifié une pièce que Kesh tient pour figée (CO 957-964).
>
> **Cette prémisse est fausse, et le code le dit.** `invoices` ne porte **aucune** colonne de snapshot du débiteur — ni `debtor_name`, ni `debtor_address`. Le PDF résout **tout à la volée** depuis le contact au moment de la génération (`invoice_pdf_service.rs` : `contact.name`, `contact.address`, `contact.client_number`). C'est même une décision explicite de la Story 16-3b : *« pas de copie dénormalisée sur `invoices`, un changement doit se refléter sur les PDF régénérés »*.
>
> **Kesh traite donc déjà `contact_id` comme un pointeur vers la fiche courante**, non comme le procès-verbal de qui a été facturé : renommer un contact change **déjà** ce que dit une facture ancienne réimprimée. Repointer un doublon vers sa fiche maîtresse est la même opération, et elle est plus juste — les deux lignes désignent la même entité réelle.
>
> Et ce qu'on croyait protéger ne l'était pas : **repointer `contact_id` ne touche aucune écriture comptable.** Le journal enregistre le compte et le montant ; le lien au contact passe par la facture. La partie double reste intacte.

**D1-bis — Le verrou d'exercice clos ne s'y oppose pas, vérifié.** `DbError::FiscalYearClosed` n'est levé que dans `journal_entries.rs` — création et modification d'écritures. **Rien dans `invoices.rs` ne le lève.** Le verrou garde la partie double, pas la métadonnée de rattachement. Une facture d'un exercice clôturé peut donc être repointée sans forcer aucune garde.

**D2 — Le perdant est archivé, jamais supprimé.** Une fois vidé, il ne porte plus rien — mais son archivage garde la trace de son existence et de la fusion, et **libère son numéro de client** (comportement en place depuis la 16-3b), ce qui permet précisément de le donner au survivant s'il n'en a pas.

**D3 — La fusion est irréversible, et l'utilisateur le sait avant.** Défaire une fusion supposerait de mémoriser l'état antérieur de chaque champ fusionné ; le coût est sans rapport avec le service rendu. L'écran le dit **avant** l'action, pas après.

## Acceptance Criteria

**AC1 — Après la fusion, plus RIEN ne pointe le perdant.**
Les factures, avoirs, factures fournisseurs et personnes de contact de A appartiennent à B. A est archivé et n'apparaît plus dans la liste par défaut ni dans les sélecteurs de facturation.
*Preuve* : un test qui, après fusion, compte les lignes de **chacune des quatre tables** portant `contact_id = A` — le compte doit être **zéro**, et le total de B doit avoir augmenté d'autant.
⚠️ **C'est l'assertion centrale de la story.** Sa mutation — oublier **une** des quatre tables — doit la faire tomber, et sur la bonne table. C'est le mode d'échec le plus probable : une fusion qui laisse trois pièces orphelines derrière elle est indiscernable d'une fusion réussie tant qu'on ne compte pas.

**AC2 — Aucune écriture comptable n'est touchée.**
La partie double est intacte : ni `journal_entries`, ni les montants, ni les comptes ne changent. Le total du grand livre et la balance sont identiques avant et après.
*Preuve* : somme des débits et des crédits comparée avant/après, et `journal_entries` inchangée ligne pour ligne.
⚠️ **C'est la contrepartie d'AC1** : la fusion déplace un rattachement, elle ne retouche pas la comptabilité. Confondre les deux est exactement l'erreur qui a fait écrire cette story à l'envers la première fois.

**AC2-bis — Une pièce d'un exercice CLÔTURÉ se repointe aussi.**
*Preuve* : le test monte une facture dans un exercice clos, fusionne, et vérifie que le repointage a eu lieu **sans lever `FiscalYearClosed`**.
⚠️ Ce test fixe la décision D1-bis. S'il devenait rouge un jour, c'est qu'une garde aurait été étendue aux colonnes de `invoices` — et il faudrait alors rouvrir la décision, pas contourner la garde.

**AC3 — Le numéro de client suit, mais n'écrase jamais.**
Si B n'a pas de numéro, il reçoit celui de A. Si B en a déjà un, celui de A **disparaît avec l'archivage** — et l'écran le dit avant.
*Preuve* : les deux cas testés ; celui où les deux en ont un vérifie que le numéro de B est **intact**.

**AC4 — On ne fusionne pas n'importe quoi.**
Refus si les deux contacts n'appartiennent pas à la même société, si l'un est déjà archivé, ou si A et B sont le même contact.
*Preuve* : trois cas, chacun avec son code d'erreur propre — pas un `400` générique.

**AC5 — La fusion est tracée intégralement.**
Une entrée d'audit dédiée : qui, quand, quel perdant, quel survivant, et ce qui a été transféré.
*Preuve* : lecture de l'audit après fusion.

**AC6 — L'écran dit ce qui va se passer, et ce qui est irréversible.**
Avant confirmation, l'utilisateur voit : ce qui passe au survivant, ce qui reste au perdant, et que **l'opération ne se défait pas**.
*Preuve* : test E2E — le libellé d'avertissement est présent, et la fusion n'a lieu qu'après confirmation explicite.

**AC7 — Les doublons de numéro de client sont proposés.**
La détection s'appuie sur la **forme canonique** livrée par la 22-1 : deux contacts actifs de la même société dont le numéro a la même canonique sont proposés comme candidats.
⚠️ **Rien de plus dans cette story.** La détection par nom ou adresse est un autre sujet, explicitement **hors périmètre** — elle demande une mesure de similarité, un seuil, et une tolérance aux faux positifs qui n'ont rien à voir avec l'égalité stricte d'une clé canonique.

## Tasks / Subtasks

- [ ] **T1 — Repository** (AC1, AC2, AC4). L'opération de fusion **en une seule transaction** : repointer `contact_id` sur `invoices`, `credit_notes`, `supplier_invoices` et `contact_persons`, transférer le numéro si le survivant n'en a pas, archiver le perdant, incrémenter les `version`. ⚠️ **Les quatre tables, ou aucune** — une transaction partielle laisserait des pièces orphelines qu'aucun écran ne montrerait.
- [ ] **T2 — Tests repository** (AC1, AC2, AC2-bis, AC3, AC4). Dont **la mutation jouée** d'AC1 : retirer une table du repointage doit faire tomber le test, sur cette table-là.
- [ ] **T3 — Audit** (AC5). Entrée dédiée. ⚠️ Le dépôt maintient `contact_snapshot_json` **à la main** — vérifier qu'elle suffit à décrire les deux côtés de la fusion.
- [ ] **T4 — Route** (AC4). Codes d'erreur propres par cas de refus, sur le patron de `map_contact_error`. L'assertion des **chaînes** de code vit dans `errors.rs`, seul endroit où le corps de la réponse est lisible.
- [ ] **T5 — Détection des candidats** (AC7). S'appuie sur la canonique de la 22-1 — **cette story dépend donc de la 22-1**.
- [ ] **T6 — Écran** (AC6). Sélection du survivant, récapitulatif de ce qui bouge, avertissement d'irréversibilité, confirmation.
- [ ] **T7 — E2E** (AC6). ⚠️ Le fichier **DOIT** être nommé `*.spec.ts` : `playwright.config.ts` filtre sur `testMatch: /(.+\.)?spec\.[jt]s/`, et un `*.test.ts` posé dans `tests/e2e/` est **silencieusement ignoré** — il ne rougit jamais, il se tait.
- [ ] **T8 — Documentation** (AC6). Manuel utilisateur : ce que la fusion fait, ce qu'elle ne fait pas, et qu'elle ne se défait pas. CHANGELOG.

## Dev Notes

### Le rayon d'impact, mesuré et non supposé

**Quatre** clés étrangères pointent `contacts`, toutes nommées `contact_id` :

| Migration | Table |
|---|---|
| `20260416000001_invoices.sql` | `invoices` |
| `20260627000001_credit_notes.sql` | `credit_notes` |
| `20260628000001_supplier_invoices.sql` | `supplier_invoices` |
| `20260706000001_contact_persons.sql` | `contact_persons` |

**Les quatre sont transférées.** C'est peu de plomberie ; toute la difficulté était dans la décision, et elle est tranchée en D1.

L'échéancier et les relances (`dunning_eligibility`) joignent le contact **à travers `invoices`** : ils suivent donc automatiquement le repointage, sans code supplémentaire. Vérifié au passage : ni la balance âgée, ni l'éligibilité aux relances, ni la liste des factures ne filtrent sur `contact.active` — un impayé ne disparaît donc jamais de la vue, quelle que soit la branche retenue.

### Ce qui ne doit pas être « réparé » au passage

⚠️ **Les instantanés d'audit sont historiques.** `contact_snapshot_json` fige l'état d'un contact au moment d'une opération. Une fusion n'est pas une raison de les réécrire — ils décrivent ce qui était vrai alors. C'est la **seule** trace qui doit continuer de désigner le perdant.

### Pourquoi le repointage ne falsifie rien

`invoices` ne porte **aucune** colonne de snapshot du débiteur. Le PDF résout `contact.name`, `contact.address` et `contact.client_number` **à la génération** (`invoice_pdf_service.rs`). Renommer un contact change donc déjà ce que dit une facture ancienne réimprimée — c'est la décision D5 de la 16-3b, assumée. Repointer un doublon vers sa fiche maîtresse relève de la même mécanique, et désigne la même entité réelle.

Le verrou d'exercice, lui, garde la **partie double** : `DbError::FiscalYearClosed` n'est levé que dans `journal_entries.rs`. Aucune garde de `invoices.rs` ne le lève.

### Dépendance

**Cette story suit la 22-1** : AC7 s'appuie sur la forme canonique qu'elle livre. Les AC1 à AC6 n'en dépendent pas et pourraient être livrés avant — mais la détection sans la canonique ne proposerait que les doublons **strictement** identiques, c'est-à-dire ceux que l'index unique interdit déjà. Elle serait donc vide.

### Conventions de test

Mutations **jouées, pas raisonnées**. Pour AC2, la mutation est explicite : repointer les factures doit faire tomber le test.
Les affirmations d'absence se vérifient au `grep -nF` avant d'être écrites.

### References

- Issue **#300**.
- Story **22-1** — la forme canonique dont dépend AC7.
- Story **16-3b** — l'archivage libère le numéro de client (décision D2-bis).
- `CLAUDE.md` — § *Review Iteration Rule*, § *Propagation post-patch*.

## Questions ouvertes

1. **Conflits de champs** — quand A et B portent des adresses ou des e-mails différents, le survivant garde-t-il les siens sans discussion, ou l'écran propose-t-il de choisir champ par champ ? La première branche est simple et suffit probablement ; à confirmer.
2. **Volume** — combien de doublons de numéro de client existent réellement sur la base de production ? La réponse dit si AC7 mérite un écran ou une simple liste. C'est la **même requête** que celle demandée par T1 de la story 22-1.
