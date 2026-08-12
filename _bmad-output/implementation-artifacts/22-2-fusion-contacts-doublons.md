# Story 22.2 : Fusionner deux contacts en doublon

## Status

backlog

## Story

**As a** indépendant ou fiduciaire dont le carnet d'adresses porte le même client deux fois,
**I want** réunir les deux fiches en une seule, sans que mes factures déjà émises changent de destinataire,
**so that** mon carnet dise la vérité, sans que ma comptabilité soit retouchée.

Ferme **#300**. Deuxième story de l'**Epic 22 « Technical Debt Closure »**, à la suite de la **22-1** dont elle est le prolongement.

## Contexte

Un même client finit par exister deux fois — saisi deux fois, importé deux fois, ou créé par deux personnes. Rien aujourd'hui ne permet de les réunir : on archive l'un à la main, et tout ce qui pendait à lui reste où il était.

Le cas devient plus fréquent avec le **numéro de client** livré en 16-3b. Deux fiches peuvent porter des numéros **identiques à l'œil** et distincts pour la base — composition Unicode, caractère invisible encastré, collation (#294, #295). La **Story 22-1** ferme cela pour l'avenir, mais le parc existant peut déjà en contenir : il faut alors pouvoir les réunir.

## Décisions

**D1 — On ne fusionne que le vivant.** *(arbitrage de Guy, 2026-08-12)*

Repointer `contact_id` sur une facture validée modifierait une pièce que Kesh tient pour **immuable** — c'est le sens du verrou de clôture d'exercice et de la conservation dix ans (**CO art. 957-964**). Le document dirait après coup qu'il a été adressé à quelqu'un d'autre.

| Passe au **survivant** | Reste **où il est** |
|---|---|
| Les personnes de contact (`contact_persons`) | Les factures déjà émises (`invoices`) |
| Le numéro de client, si le survivant n'en a pas | Les avoirs (`credit_notes`) |
| Toute la facturation **future** | Les factures fournisseurs (`supplier_invoices`) |
| | Les instantanés d'audit, historiques par nature |

**D2 — Le perdant est archivé, jamais supprimé.** Ses pièces passées restent lisibles et rattachées à lui. Son archivage **libère son numéro de client** — comportement déjà en place depuis la 16-3b, et qui rend justement possible de donner ce numéro au survivant.

**D3 — La fusion est irréversible, et l'utilisateur le sait avant.** Défaire une fusion supposerait de mémoriser l'état antérieur de chaque champ fusionné ; le coût est sans rapport avec le service rendu. L'écran le dit **avant** l'action, pas après.

## Acceptance Criteria

**AC1 — Fusionner réunit ce qui est vivant.**
Après fusion de A dans B : les personnes de contact de A appartiennent à B ; A est archivé ; A n'apparaît plus dans la liste par défaut ni dans les sélecteurs de facturation.
*Preuve* : test repository sur les quatre tables du rayon d'impact.

**AC2 — L'historique comptable n'est pas touché.**
Les factures, avoirs et factures fournisseurs de A **restent rattachés à A**, avant comme après.
*Preuve* : le test compte les pièces de A avant et après — le nombre est le même, et leur `contact_id` est inchangé. ⚠️ **C'est l'assertion centrale de la story** : sa mutation — repointer les pièces — doit la faire tomber.

**AC3 — Le numéro de client suit, mais ne écrase jamais.**
Si B n'a pas de numéro, il reçoit celui de A. Si B en a déjà un, **celui de A est perdu avec l'archivage** et l'écran le dit avant.
*Preuve* : les deux cas testés, et le cas où les deux en ont un vérifie que celui de B est **intact**.

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

- [ ] **T1 — Repository** (AC1, AC2, AC4). L'opération de fusion en une transaction : transférer `contact_persons`, transférer le numéro si le survivant n'en a pas, archiver le perdant, incrémenter les `version`. **Ne toucher ni `invoices`, ni `credit_notes`, ni `supplier_invoices`.**
- [ ] **T2 — Tests repository** (AC1, AC2, AC3, AC4). Dont le test d'AC2 et **sa mutation jouée**.
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

**Une seule est transférée** : `contact_persons`. Les trois autres sont précisément ce que D1 protège.

L'échéancier et les relances (`dunning_eligibility`) joignent le contact **à travers `invoices`** : ils suivent le rattachement d'origine sans qu'on ait rien à faire — ce qui est le comportement voulu, puisque la facture n'a pas changé de destinataire.

### Ce qui ne doit pas être « réparé » au passage

⚠️ **Les instantanés d'audit sont historiques.** `contact_snapshot_json` fige l'état d'un contact au moment d'une opération. Une fusion n'est pas une raison de les réécrire — ils décrivent ce qui était vrai alors.

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
