# Story 22.2 : Prévenir les doublons à la saisie d'un contact

## Status

backlog

## Story

**As a** utilisateur qui saisit un contact,
**I want** que Kesh me signale, pendant que je tape, qu'un contact proche existe déjà,
**so that** je reprenne la fiche existante au lieu d'en créer une seconde — au moment où cela ne coûte encore rien.

Ferme **#301**. Deuxième story de l'**Epic 22 « Technical Debt Closure »**.

## Contexte

Le même client finit par exister deux fois : saisi une première fois, ressaisi plus tard sans qu'on s'en souvienne. Rien ne le signale au moment où c'est encore gratuit — **avant** l'enregistrement.

**Le moment est le bon, et il ne se représentera pas.** Kesh est déployé mais **ne tient pas encore les comptes réels** : le jalon « Première clôture d'exercice tenue dans Kesh » est ouvert. Il n'existe donc **aucun parc de doublons à réparer**.

C'est ce qui rend l'arbitrage net : la prévention coûte le même prix aujourd'hui qu'elle coûtera dans deux ans, mais elle **évite** la dette au lieu de la rembourser. La fonction de fusion (Story 22-3, #300) est une réparation — elle reste en veille, et le mieux serait qu'elle n'ait jamais à servir.

## Décisions

**D1 — Signaler, jamais bloquer.** Deux clients peuvent légitimement porter des noms très proches : deux sociétés d'un même groupe, deux homonymes, un père et son fils. Le dispositif **informe** et laisse l'utilisateur décider. Un blocage produirait des contournements — un espace ajouté au nom — qui salissent le carnet plus sûrement qu'un doublon assumé.

**D2 — Le numéro IDE est un signal fort, le nom un signal faible.** Ils n'appellent pas le même avertissement :

| Ce qui correspond | Avertissement |
|---|---|
| **Numéro IDE identique** | Franc : c'est un identifiant d'État, deux entités ne le partagent pas. Presque certainement le même contact. |
| **Nom proche** | Nuancé : *« un contact au nom proche existe »*, avec de quoi le reconnaître — nom complet, localité, numéro de client. |

**D3 — Réutiliser la recherche existante, ne pas en inventer une seconde.** Le carnet se cherche déjà par index FULLTEXT sur `name`, complété d'un `LIKE` sur l'email et le numéro de client (Story 16-3b). C'est ce chemin qu'il faut réemployer. ⚠️ **Il porte DEUX branches** dans `push_where_clauses` — celle du terme échappé vide et celle du cas courant : **les deux, ou aucune**.

**D4 — Hors périmètre, explicitement.** Deux cas voisins relèvent d'un autre traitement, et les confondre ferait dériver cette story :

- l'**appariement automatique** à l'import de documents — gouverné par la règle « un appariement propose, il ne crée jamais » du `CLAUDE.md` ;
- le **rachat d'entreprise** (#302) — ce n'est pas un doublon mais une **succession datée** entre deux entités distinctes, avec deux numéros IDE.

## Acceptance Criteria

**AC1 — La frappe du nom fait apparaître les contacts proches.**
À la saisie du nom dans le formulaire de création, les contacts existants dont le nom est proche sont proposés, avec de quoi les reconnaître : nom complet, localité, et numéro de client s'il existe.
*Preuve* : test de composant — saisir un fragment du nom d'un contact existant le fait apparaître.

**AC2 — Un numéro IDE déjà pris est signalé franchement.**
La saisie d'un IDE déjà porté par un contact actif de la société déclenche un avertissement explicite, distinct du signal « nom proche », **avant** l'enregistrement.
⚠️ La contrainte `uq_contacts_company_ide` refuse déjà le doublon **à l'enregistrement**, en `409`. Cette AC ne remplace pas la garde : elle évite d'y arriver, et surtout d'y arriver **après** avoir saisi toute la fiche.
*Preuve* : test de composant, et un test qui vérifie que le `409` reste levé si l'avertissement est ignoré.

**AC3 — L'avertissement ne bloque jamais.**
L'utilisateur peut enregistrer malgré le signal.
*Preuve* : test E2E — ignorer l'avertissement crée bien le contact. ⚠️ **C'est l'assertion qui protège D1** : sa mutation — désactiver le bouton d'enregistrement — doit la faire tomber.

**AC4 — Rien ne part avant que la frappe se calme.**
La recherche est temporisée et annulable : taper vingt caractères ne déclenche pas vingt requêtes, et une réponse tardive n'écrase pas un affichage plus récent.
*Preuve* : test unitaire sur le nombre d'appels pour une frappe continue, et sur l'ordre d'arrivée des réponses.

**AC5 — Le dispositif est muet quand il n'a rien à dire.**
Aucun contact proche ⇒ aucun encombrement de l'écran, aucun message.
*Preuve* : test de composant sur un carnet vide.

**AC6 — L'avertissement est traduit sur les quatre locales.**
*Preuve* : le test d'appariement positionnel de `kesh-i18n`, et `lint-i18n-ownership` vert. ⚠️ Le domaine `contact-*` est aujourd'hui à **62 clés dans chacune** des quatre locales, sans dérive — ne pas l'ouvrir.

## Tasks / Subtasks

- [ ] **T1 — Point d'entrée de recherche** (AC1, AC2). Décider s'il faut une route dédiée ou si la recherche existante suffit. ⚠️ Elle est **paginée et scopée par société** — vérifier que le scoping tient sur ce chemin aussi ; c'est le genre de garde qu'une route nouvelle oublie.
- [ ] **T2 — Composant d'avertissement** (AC1, AC2, AC3, AC5). Deux niveaux de signal (D2), aucun blocage.
- [ ] **T3 — Temporisation et annulation** (AC4). Une réponse tardive ne doit jamais écraser un affichage plus récent — le défaut classique, invisible en test manuel et systématique en réseau lent.
- [ ] **T4 — i18n** (AC6). Les clés sur les quatre locales, dans le domaine `contact-*`.
- [ ] **T5 — E2E** (AC3). ⚠️ Le fichier **DOIT** être nommé `*.spec.ts` : `playwright.config.ts` filtre sur `testMatch: /(.+\.)?spec\.[jt]s/`, et un `*.test.ts` posé dans `tests/e2e/` est **silencieusement ignoré** — il ne rougit jamais, il se tait.
- [ ] **T6 — Documentation** (AC1). Manuel utilisateur : ce que l'avertissement dit, et qu'il n'empêche rien.

## Dev Notes

### Ce qui existe déjà, et qu'il ne faut pas réinventer

La recherche du carnet (`repositories/contacts.rs`, `push_where_clauses`) combine un index **FULLTEXT** sur `name` et un `LIKE` sur l'email et le numéro de client. Le commentaire du fichier explique pourquoi le numéro de client n'est **pas** dans l'index FULLTEXT — les séparateurs cassent les tokens.

⚠️ **Deux branches `LIKE`** : l'une sert quand le terme échappé est vide, l'autre le cas courant. N'en traiter qu'une compile, passe les tests dont le terme survit à `escape_boolean_ft`, et **cesse silencieusement de chercher** quand le terme n'est fait que d'opérateurs FULLTEXT.

La contrainte `uq_contacts_company_ide` existe depuis l'origine et refuse déjà le doublon d'IDE en `409`, via `map_contact_error`.

### Le piège de cette story

Un avertissement trop bavard **ne protège plus rien** : on apprend à le fermer sans le lire, et il devient un clic de plus. C'est le vrai risque, plus que le faux négatif. Le seuil doit donc être **serré** — mieux vaut manquer un doublon que crier trois fois par jour à tort.

### Conventions de test

Mutations **jouées, pas raisonnées**. Pour AC3, la mutation est explicite : désactiver le bouton d'enregistrement doit faire tomber le test.
Les affirmations d'absence se vérifient au `grep -nF` avant d'être écrites.

### References

- Issue **#301**.
- Story **22-3** (#300) — la fusion, en veille : ce que cette story doit rendre inutile.
- Issue **#302** — la succession d'entreprise, à ne pas confondre.
- `CLAUDE.md` — § *Un appariement automatique propose, il ne crée jamais*.

## Questions ouvertes

1. **Le seuil et les champs** — nom seul, ou nom + localité ? Une correspondance sur le nom **et** la localité est un signal bien plus fort qu'un nom seul, et beaucoup moins bruyant.
2. **Le formulaire d'édition** — l'avertissement vaut-il aussi quand on **modifie** un contact existant, en renommant vers un nom déjà pris ? Probablement oui, et à moindres frais puisque le dispositif sera écrit.
