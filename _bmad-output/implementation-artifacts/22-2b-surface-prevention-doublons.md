# Story 22.2b : La surface — deux sondes, deux signaux, et rien qui bloque

## Status

ready-for-dev

**Dégelée le 2026-08-18.** Elle avait été mise en attente parce que plusieurs de ses findings portaient sur la **frontière** avec le socle, et les corriger contre un contrat supposé aurait été à refaire. **Le socle est désormais codé et ses contrats sont figés par du code qui tourne** — cf. § *Ce que le socle fournit, et sa signature exacte*.

Seconde moitié du découpage de la **22-2** (arbitrage de Guy, 2026-08-17, question **Q5**). Elle **branche** le module pur livré par la **22-2a** et lui donne une surface : requêtes, avertissements, traductions, et la preuve de bout en bout.

⚠️ **Ordre SÉRIE : 22-2a d'abord, 22-2b ensuite.** Cette story importe le module de la 22-2a ; commencée avant, elle réécrirait la logique qu'elle est censée consommer, et le découpage n'aurait servi à rien.

## Story

**As a** utilisateur qui saisit un contact,
**I want** que Kesh me signale, pendant que je tape, qu'un contact proche existe déjà,
**so that** je reprenne la fiche existante au lieu d'en créer une seconde — au moment où cela ne coûte encore rien.

**Ferme #301** (les deux moitiés dans **une seule PR**, cf. § *Livraison*).

## Contexte

Le même client finit par exister deux fois : saisi une première fois, ressaisi plus tard sans qu'on s'en souvienne. Rien ne le signale au moment où c'est encore gratuit — **avant** l'enregistrement.

**Le moment est le bon, et il ne se représentera pas.** Kesh est déployé mais **ne tient pas encore les comptes réels** : le jalon « Première clôture d'exercice tenue dans Kesh » est ouvert. Il n'existe donc **aucun parc de doublons à réparer**. La prévention coûte le même prix aujourd'hui qu'elle coûtera dans deux ans, mais elle **évite** la dette au lieu de la rembourser. La fusion (Story 22-3, #300) est une réparation ; le mieux serait qu'elle n'ait jamais à servir.

## Périmètre

**Dedans** : le backend rendant l'IDE cherchable, les deux sondes et leur temporisation, le balisage et les deux niveaux de signal, l'i18n sur quatre locales, l'E2E, le manuel.

**Dehors** : toute la logique d'appariement — normalisation, seuil, classement, exclusion de soi, vérification de l'IDE — qui est **livrée et prouvée par la 22-2a**. ⚠️ Si un patch de cette story réimplémente l'une de ces fonctions, il déborde.

**Dehors, explicitement** (hérité de la 22-2) : l'**appariement automatique** à l'import, gouverné par la règle « un appariement propose, il ne crée jamais » du `CLAUDE.md` ; le **rachat d'entreprise** (#302), qui est une succession datée entre deux entités distinctes ; la **recherche mid-word**, régression assumée depuis 7-4 / KF-005 et gardée par `test_search_no_longer_matches_mid_word`.

## Ce que le socle fournit, et sa signature exacte

⚠️ **Ceci n'est plus une intention : c'est l'API d'un module qui existe, dont les 44 preuves passent et dont les 24 mutations sont jouées.** Toute divergence entre cette story et ces signatures est un défaut de CETTE story.

```ts
import {
  probeTerm, normalizeTerm, fold, buildTerm, isArmed,
  rank, excludeSelf, countOthers, findIdeHolder
} from '$lib/features/contacts/duplicate-probe';

probeTerm(type: ContactType, name: string, firstName: string, lastName: string)
  : { normalized: string; armed: boolean }        // compose buildTerm → normalizeTerm → isArmed
rank(items: ContactResponse[], normalizedTerm: string): ContactResponse[]   // CLASSE, ne tronque ni ne filtre
excludeSelf(items: ContactResponse[], editingId: number | null): ContactResponse[]
countOthers(total: number, items: ContactResponse[], affiches: ContactResponse[], editingId: number | null): number
findIdeHolder(items: ContactResponse[], normalized: string | null, editingId: number | null): ContactResponse | undefined
```

Quatre points de frontière, chacun trouvé par une passe de revue et **désormais tranché par le code** :

- **`editingId` est `number | null`, jamais `undefined`.** Le site d'appel écrit donc `editing?.id ?? null` — `editing` étant `ContactResponse | null`, `editing?.id` nu rendrait `number | undefined` et `npm run check` le rejetterait.
- **`buildTerm` prend QUATRE paramètres**, le type puis les trois champs de nom.
- **`rank` reçoit un terme DÉJÀ normalisé, et ne tronque pas.** La fenêtre de cinq est découpée par l'appelant.
- **`probeTerm` porte l'ORDRE** `normaliser → mesurer → décider`. ⚠️ **L'appeler plutôt que composer soi-même les trois étapes n'est pas un confort** : c'est ce qui rend la mutation « mesurer le seuil avant de normaliser » jouable *dans le socle*. Recomposer à la main ici rouvrirait le trou que le découpage a fermé.

**Et le socle fait plus que ce que cette story supposait**, depuis la revue du 2026-08-18 : `normalizeTerm` retire les invisibles de largeur nulle (un ZWSP collé depuis un courriel), `fold` replie les formes de compatibilité (`ＤＵＢＡＲＤＥ` = `DUBARDE`), et `buildTerm` ne fabrique plus le mot `"null"` à partir d'un champ absent. Cette story n'a **rien** à réimplémenter de tout cela.

## Décisions

**D-b1 — Signaler, jamais bloquer.** Deux clients peuvent légitimement porter des noms très proches : deux sociétés d'un même groupe, deux homonymes, un père et son fils. Le dispositif **informe**. Un blocage produirait des contournements — un espace ajouté au nom — qui salissent le carnet plus sûrement qu'un doublon assumé.

**D-b2 — L'IDE est un signal fort, le nom un signal faible.**

| Ce qui correspond | Avertissement |
|---|---|
| **Numéro IDE identique** | Franc : c'est un identifiant d'État, deux entités ne le partagent pas. |
| **Nom proche** | Nuancé : *« des contacts pourraient correspondre »*, avec de quoi les reconnaître. |

⚠️ **La distinction visuelle des deux niveaux n'a PAS de preuve dédiée, et c'est assumé** : elle est **acquittée par construction** — deux clés i18n distinctes (T-b5) et deux points d'ancrage distincts, chacun sous son champ (T-b4). Une preuve séparée porterait sur une propriété de style, que ni vitest ni Playwright ne jugent utilement.
⚠️ Et le libellé du signal nuancé ne dit plus « au nom proche » : la sonde propose aussi ce que le serveur a trouvé par l'email ou le numéro de client (**D-b13**).

**D-b3 — Aucune route nouvelle ; une seule extension du chemin existant.** *(Q1 de la 22-2, tranchée par Guy.)*
`GET /api/v1/contacts?search=…&limit=…` fait déjà l'essentiel : scopé société (`routes/contacts.rs:591`), archivés exclus par défaut (`push_where_clauses:158`), `name` en FULLTEXT. Il manque une seule chose : **`ide_number` n'est pas cherché**. Il rejoint donc l'email et le numéro de client dans les **deux** branches `LIKE`.
⚠️ **Les deux branches, ou aucune.** N'en traiter qu'une compile, passe les tests dont le terme survit à `escape_boolean_ft`, et cesse **silencieusement** de chercher l'IDE quand le terme n'est fait que d'opérateurs.
⚠️ **Le bénéfice de bord est plus faible qu'il n'y paraît** : `'CHE109322551' LIKE '%CHE-109.322.551%'` rend **0** — l'IDE est stocké normalisé sur 12 caractères, le `LIKE` porte sur le terme brut. Seule la saisie **sans séparateurs** remontera le contact, et c'est justement pas la forme qu'on lit sur une facture papier. Ne pas réécrire la promesse inverse.

**D-b4 — « Réutiliser le chemin » n'est pas « c'est indexé ».** Mesuré :

| Requête | Plan | Lignes examinées |
|---|---|---|
| `MATCH` seul | `fulltext`, clé `ft_contacts_name` | 1 |
| **`MATCH OR LIKE` (la requête réelle)** | **`range`, clé `idx_contacts_company_name`** | **1506** |

Sur 3016 contacts : **1,26 ms + 2,36 ms** pour la paire `COUNT` + `SELECT` d'une sonde, contre 0,09 ms en `MATCH` seul. ⚠️ **Le compte de requêtes était faux, et il l'était par défaut** : chaque `GET /api/v1/contacts` exécute **trois** requêtes SQL, pas deux — le handler lit d'abord `companies` (`get_company_for`, `routes/contacts.rs:566`) pour le contrôle défensif et la langue d'instance. Une pause de frappe émet donc **six** requêtes SQL pour **deux** requêtes HTTP.
⚠️ **Et la mesure précède T-b1**, qui ajoute une quatrième branche `LIKE` sur `ide_number` : une comparaison de plus par ligne examinée, sans changement de plan d'accès. L'écart attendu est faible, mais un chiffre doit déclarer son périmètre. **#301 demandait d'instruire ce coût** : le voici, ≈ 4 ms par sonde à 3000 contacts, acceptable. Ce qu'il ne faut pas écrire, c'est « c'est indexé » — un dev à qui l'on dit cela ne mesurera jamais.

**D-b5 — Les deux sondes n'ont PAS la même politique d'archivage.**
⚠️ Les deux contraintes d'unicité de `contacts` sont **asymétriques**, et la migration `20260810000001:39-49` l'explique en toutes lettres :

| Contrainte | Forme | Un contact archivé… |
|---|---|---|
| `uq_contacts_company_ide` | **plate** (`20260414000001:23`) | **garde son IDE à vie** — un IDE d'État ne se réattribue pas |
| `uq_contacts_company_client_number` | **partielle** | **libère son numéro** |

Un contact archivé occupe donc toujours son IDE : ressaisir cet IDE donne un `409`. Une sonde interrogeant avec le défaut `includeArchived: false` **ne verrait rien** et se tairait — l'utilisateur remplirait toute la fiche pour se heurter à un refus **dont le coupable est invisible dans son carnet**, et sans recours : un contact archivé n'est ni modifiable (`IllegalStateTransition`) ni désarchivable, **aucune route ne l'ouvre**.

- **la sonde IDE interroge `includeArchived: true`**, et l'avertissement **dit que le porteur est archivé** quand il l'est ;
- **la sonde nom reste sur `includeArchived: false`** : un contact archivé n'est pas une fiche qu'on veut reprendre.

**D-b6 — La garde anti-écrasement est un compteur de génération, PAS un `AbortSignal`.**
⚠️ `api-client.ts:259-268` : `fetch(url, { ...init, credentials: 'include', signal: controller.signal })`. Le spread place `signal` **après** `...init` : tout `signal` passé par l'appelant est **écrasé en silence**. Un code qui crée un `AbortController` et le passe à `apiClient.get` **n'annule rien**, et rien ne le lui dit. Le commentaire de `api-client.ts:250-258` documente ce cas et désigne d'avance le futur appelant concerné — **c'est nous**.
Le patron éprouvé est le compteur de `ContactPicker.svelte:37,57,61,65,69` : il ne coupe pas la requête, il rend sa réponse inoffensive — ce qu'on demande.
Écarté : composer par `AbortSignal.any([...])` dans `api-client.ts`, qui modifierait le chemin HTTP de **toute** l'application pour une page.

**D-b7 — Deux sondes, deux minuteries, deux compteurs.** Elles ne partagent **rien**. Une paire factorisée ferait qu'une réponse tardive de l'une **invalide la fraîcheur de l'autre** — un des deux avertissements ne s'afficherait jamais, sans erreur ni test rouge. `ContactPicker` ne peut pas servir de modèle : il n'a qu'un flux.

**D-b8 — L'état des sondes repart de zéro à l'OUVERTURE du formulaire, pas à sa fermeture.**
Le dialogue se ferme depuis **trois** sites (`+page.svelte:352`, `:358`, `:846`) et n'est **jamais démonté** — le nettoyage d'`onMount:113` ne joue qu'à la sortie de la page. Poser le nettoyage sur la fermeture obligerait à le poser trois fois, et un quatrième site futur le contournerait en silence. `openCreate()` et `openEdit()` (`:216-266`) réinitialisent déjà champ par champ : minuteries, compteurs et propositions s'y ajoutent.

**D-b8-bis — Le contrat de `editingId` est `number | null`, et le site d'appel s'y conforme.**
⚠️ `editing` est typé `ContactResponse | null` (`+page.svelte:60`), donc `editing?.id` s'évalue en **`number | undefined`** — jamais `null`. Or la 22-2a documente et teste `null` comme unique convention d'absence (sa preuve 3 d'AC-a6). Les trois appels doivent donc s'écrire **`editing?.id ?? null`**, faute de quoi `npm run check` — gate obligatoire de la § *Test Locally First* — rejette un code que cette story prescrit **littéralement**. Trouvé en passe 1, à la frontière exacte du découpage.

**D-b9 — Une proposition informe ; elle ne navigue pas.**
`openEdit(c)` réassigne **les dix-neuf champs** du formulaire (`+page.svelte:241-266`) : un clic mal spécifié **effacerait la saisie en cours sans confirmation**. La proposition est **inerte**. C'est la position cohérente avec D-b1, et la moins coûteuse : elle dispense de spécifier la protection de la saisie **et** le sort d'un contact archivé entre l'affichage et le clic.

**D-b11 — La zone d'avertissement « nom proche » vit HORS des deux branches de type, et la bascule de type réarme la sonde.**

⚠️ **Deux contraintes de T-b4 se contredisent si l'on n'y prend pas garde.** Le champ de nom n'est pas un champ : c'est `#form-name` pour une `Entreprise` et `#form-firstname` + `#form-lastname` pour une `Personne`, dans les deux branches d'un `{#if formContactType === 'Personne'}`. Or T-b4 exige que la zone soit rendue **en permanence dans le DOM** pour qu'`aria-live` fonctionne. **Une zone logée dans l'une des deux branches est démontée à chaque bascule** — donc ni permanente, ni annoncée.

La zone unique est donc placée **après** le bloc type-dépendant, hors des deux branches.

⚠️ **Et la bascule de type réarme la sonde.** `openCreate`/`openEdit` ne remettent à zéro qu'à l'**ouverture** (D-b8) ; changer `#form-type` en cours de saisie ne les appelle pas, et **les champs de nom conservent leur valeur**. Sans réarmement, un avertissement calculé sur l'ancien type resterait affiché — orphelin, sous un champ qui n'est plus dans le DOM — jusqu'à la frappe suivante. Le `<select id="form-type">` déclenche donc le même recalcul que les champs de nom.

**D-b10 — La localité s'affiche, elle ne se cherche pas.** Une correspondance « nom **et** localité » serait un signal plus fort, mais l'obtenir demanderait une dimension de requête supplémentaire — alors que le même service est rendu **à coût nul** en affichant la localité : c'est l'utilisateur qui discrimine, et il le fait mieux qu'un seuil.
⚠️ La localité vaut `""` pour tout contact sans adresse (`routes/contacts.rs:213-219`, `city: c.address_city.unwrap_or_default()`) — c'est le cas de la fiche minimale que cette story vise. **L'invariant est : deux propositions ne sont JAMAIS identiques à l'écran.** Les niveaux localité → numéro de client → email y pourvoient d'ordinaire ; quand ils n'y suffisent pas, `#<id>` est ajouté.
⚠️ **Énoncer une cascade au lieu d'un invariant laissait un trou**, et il est banal : le père et le fils de D-b1, **même ménage donc même localité**, sans numéro ni email. La localité *existe*, donc la cascade ne se déclenche pas — et les deux lignes s'affichent `Jean Dupont — Lausanne` à l'identique. La condition n'est donc pas « les niveaux manquent » mais « **les libellés construits sont égaux** », ce qui se code en une comparaison et ne coûte aucune requête.
⚠️ **Le dernier niveau n'est pas décoratif : sans lui, l'invariant que cette décision énonce est FAUX.** `email` est `string | null` et `clientNumber` aussi — rien n'interdit une fiche sans aucun des trois, et c'est justement la « fiche minimale et vite faite » que cette story vise. Deux homonymes minimaux — le père et le fils de D-b1 — s'afficheraient alors **rigoureusement identiques**, et l'utilisateur ne pourrait plus dire laquelle est laquelle. L'`id` est déjà dans `ContactResponse`, ne coûte aucune requête, et est **garanti distinct**.

**D-b12 — `rank` réordonne la fenêtre, il ne l'étend pas — et au-delà, le dispositif redevient muet.**

⚠️ **Limite résiduelle, mesurée, non corrigée.** Le tri serveur reste **alphabétique** : au-delà de `limit` contacts partageant un token du terme, le doublon exact **n'entre pas dans la fenêtre**, et aucun classement client ne peut le rattraper. Reproduit sur 25 sociétés en `Sàrl` — terme `Zurcher Sàrl`, `total = 25`, fenêtre arrêtée à `Nicolet Sàrl`, **`Zurcher Sàrl` absente**. L'utilisateur voit alors cinq sociétés sans rapport et « et 20 autres », et **rien sur celle qu'il recrée**.

C'est le **résidu de #315**, que D-b3 assume déjà comme une atténuation. Il faut néanmoins l'écrire ici, parce que le titre de la preuve 9 d'AC-b1 — « la fenêtre SQL est assez large » — ne vaut que pour sa fixture de six lignes et **généralise une propriété que l'exécution réfute**.

**Ce qui est retenu** : porter la limite dans la documentation de la fonction et dans le manuel, **sans** la corriger ici. La corriger demanderait le tri par pertinence de #315, qui touche un chemin partagé par quatre repositories.
⚠️ Une atténuation existe et est **écartée** : conditionner l'affichage à `total ≤ limit`, c'est-à-dire ne rien proposer plutôt que proposer faux. Elle est cohérente avec « mieux vaut manquer un doublon que crier à tort », mais elle rendrait le dispositif **muet sur les carnets les plus fournis**, c'est-à-dire ceux où le risque de doublon est le plus élevé. Le bruit est ici préférable au silence, à condition d'être documenté.

**D-b13 — La sonde nom propose ce que le SERVEUR a trouvé, y compris par l'email ou le numéro de client.**

⚠️ Le prédicat SQL est `MATCH(name) OR email LIKE OR client_number LIKE` — et `rank` **classe sans filtrer**. Un contact remonté par son seul email entre donc dans les propositions, sous un en-tête qui annonce « un contact au nom proche existe ». Vérifié : terme `Martin`, le carnet rend `Boulangerie Kalt SA` (email `martin@…`) et `Garage du Lac` (numéro `MARTIN-2024`), dont **aucun nom ne porte le terme**.

**Ce qui est retenu** : ne pas filtrer, et **changer le libellé**. `contact-duplicate-heading` dit « **Contacts déjà enregistrés qui pourraient correspondre** », jamais « au nom proche ». Un contact dont l'email porte le terme tapé **est** un candidat au doublon pertinent — le filtrer serait perdre de l'information ; c'est l'en-tête qui mentait, pas la recherche.

## Acceptance Criteria

**AC-b1 — La frappe du nom fait apparaître les contacts proches, le plus proche en premier.**
Dès que le module de la 22-2a arme la sonde, les contacts **actifs** de la société sont interrogés avec `limit: 20`, classés et coupés à cinq, avec la mention « et N autres » quand il en reste. Chaque proposition porte le **nom complet**, puis la **localité** et le **numéro de client** quand ils sont renseignés, à défaut l'**email** (D-b10). Les propositions sont **inertes** (D-b9).
*Preuve*, **8** tests de composant plus **1** d'intégration base :
1. `Entreprise` — les trois éléments d'affichage sont rendus. Mutation : « ne rendre que `c.name` », qui supprime le mécanisme de discrimination de la story sans faire rougir rien d'autre ;
2. `Personne` — la sonde part sur prénom + nom. Mutation : « ne lire que la raison sociale » ;
3. un contact **sans adresse et sans numéro de client** : l'email est rendu, aucune ligne vide ni tiret orphelin ;
4. **l'argument de l'appel de LA SONDE**, isolé des autres appels de la page :
   ```ts
   const sondes = listContactsMock.mock.calls.filter(([q]) => q.search === 'Coop Vaud');
   expect(sondes).toHaveLength(1);
   expect(sondes[0][0]).toEqual({ search: 'Coop Vaud', limit: 20, includeArchived: false });
   ```
   ⚠️ **Trois exigences, et chacune ferme un trou distinct.**
   **(a) Filtrer sur `search`** — `toHaveBeenCalledWith(objectContaining({ includeArchived: false, limit: 20 }))` est **satisfait par l'appel que la page fait DÉJÀ au montage** : `loadContacts()` passe exactement `limit: 20` et `includeArchived: false` par défaut (`+page.svelte:48,51`), et le patron impose `vi.clearAllMocks()` en `beforeEach`, donc **avant** le `render`. Vérifié en lançant vitest : la mutation reste **verte**. La preuve annoncée comme seule garde du piège ne le gardait pas.
   **(b) `toEqual`, pas `objectContaining`** — la sonde ne passe ni `offset`, ni `sortBy`, ni `sortDirection` ; l'égalité stricte est ce qui la distingue de la liste.
   **(c) Asserter `search` lui-même** — c'est le paramètre dont **tout** dépend, et il n'était épinglé nulle part. La valeur attendue `'Coop Vaud'` prouve au passage que le terme est **normalisé** : la mutation « envoyer `buildTerm(...)` brut » rendrait `'Coop-Vaud'` et rouvrirait #314.
   ⚠️ **Une preuve fonctionnelle ne convient PAS.** `includeArchived` est un paramètre de query-string traité **par le serveur** ; sous `vi.mock`, le double rend ce qu'on lui a dit **sans regarder son argument**. « Un archivé n'apparaît pas » reste donc vert sous la mutation `false → true`. C'est ce qu'énonce le doc-comment de `products-page.test.ts` : *seule l'assertion sur l'argument l'attrape* ;
5. **cliquer sur une proposition ne modifie NI le formulaire NI son état** (D-b9). Mutation : `onclick={() => openEdit(c)}` — un ajout de bonne foi, qui compile et efface dix-neuf champs ;
6. **la chaîne de repli épuisée** — deux contacts **homonymes** sans adresse, sans numéro de client **et sans email** : les deux propositions doivent **différer** à l'écran (repli sur `#<id>`, D-b10). Mutation : « s'arrêter à l'email », qui rend deux lignes rigoureusement identiques ;
7. **la bascule de type en cours de saisie** (D-b11) — taper une raison sociale qui fait apparaître des propositions, puis basculer sur `Personne` sans effacer : la sonde **se réarme** sur le nouveau terme, et aucun avertissement calculé sur l'ancien type ne subsiste. Mutation : « ne recalculer que sur les champs de nom », qui laisse un avertissement orphelin sous un champ retiré du DOM ;
8. **classé, coupé à cinq, et compté** — une seule fixture pour trois clauses qu'aucune preuve ne couvrait. Huit contacts correspondants, le doublon exact triant **dernier** alphabétiquement ; `total = 8` ⇒ (a) **exactement cinq** propositions rendues, ni quatre ni vingt ; (b) le doublon exact est **le premier** ; (c) « et 3 autres » est **visible**.
   ⚠️ Trois mutations d'un caractère chacune laissaient les 31 preuves vertes : **`retenus = excludeSelf(...)` sans l'enveloppe `rank(...)`** (retour au tri alphabétique du SQL, doublon évincé — la raison d'être du découpage), **`proches = retenus`** sans le `.slice(0, 5)`, et **ne jamais rendre le bloc du compteur** — dont la seule mention dans les preuves assertait son *absence* ;
9. **`#[sqlx::test]` — la fenêtre SQL est assez large.** Fixture des six `Jean X` ; `limit: 20` doit rendre **les six** lignes et `total = 6`. Mutation : **« préfixer chaque token de `+` »** (sémantique ET) ou « trier par `CreatedAt` » — l'une comme l'autre font tomber le `total = 6` ou la présence des six lignes.
   ⚠️ **La mutation annoncée jusqu'ici, « revenir à `limit: 5` », était INJOUABLE ici** : `limit` est un **paramètre d'entrée** du repository (`routes/contacts.rs:571`, `clamp(1, MAX_LIST_LIMIT)`), jamais une constante du backend — la seule occurrence de `limit: 20` dans le produit est une ligne **TypeScript**, qu'aucun test Rust ne peut observer. C'est la preuve 4 d'AC-b1, durcie ci-dessus, qui garde cette valeur. ⚠️ Cette preuve porte sur la **requête** — sémantique OU, tri alphabétique, taille de fenêtre — et sur elle seule ; les preuves de composant passent par un `vi.mock` et en sont aveugles. *(Le classement lui-même est prouvé par la 22-2a, en TypeScript : un test Rust ne peut pas appeler `rank()`.)*

**AC-b2 — Un IDE déjà pris est signalé franchement, y compris quand le porteur est archivé.**
La saisie d'un IDE **complet** (`validateIdeFormat`) déjà porté par un contact de la société — **actif ou archivé** (D-b5) — déclenche un avertissement explicite, **distinct** du signal « nom proche », **avant** l'enregistrement. Quand le porteur est archivé, le message **le dit**.
⚠️ `uq_contacts_company_ide` refuse déjà le doublon à l'enregistrement, en `409 IDE_ALREADY_EXISTS` (`map_contact_error:536-541`). Cette AC ne remplace pas la garde : elle évite d'y arriver **après** avoir saisi toute la fiche.
*Preuve*, **5** tests de composant plus **1** assertion E2E :
1. l'avertissement s'affiche sur un porteur **actif** ;
2. **l'argument de la sonde IDE**, saisi en **forme SÉPARÉE** — celle que l'interface affiche et qu'`openEdit` écrit :
   ```ts
   // l'utilisateur tape CHE-123.456.788
   const sondes = listContactsMock.mock.calls.filter(([q]) => q.includeArchived === true);
   expect(sondes[0][0]).toEqual({ search: 'CHE123456788', limit: 5, includeArchived: true });
   ```
   ⚠️ **Sans l'assertion sur `search`, la mutation `search: formIde` survit aux 31 preuves ET à l'E2E** — et rend la sonde **muette pour tout utilisateur réel** : l'UI affiche `CHE-123.456.789` en placeholder (`+page.svelte:769`) et `openEdit` écrit **inconditionnellement** la forme séparée (`:256`), or D-b3 établit que `'CHE123456788' LIKE '%CHE-123.456.788%'` rend **0**. Le mock, lui, rend le porteur sans regarder son argument, et l'E2E ne l'attrape pas si le test tape la forme déjà normalisée.
   **Et** le message rendu pour un porteur archivé est **distinct** de celui d'un porteur actif — deux clés i18n séparées. ⚠️ Les deux moitiés sont indispensables : l'argument parce qu'une preuve fonctionnelle est verte sous la mutation, le libellé parce que **c'est lui le recours** — sans la mention « archivé », l'utilisateur reçoit un avertissement sur un contact introuvable dans son carnet, qu'il ne pourra ni modifier ni désarchiver ;
3. **ouvrir en édition** la fiche d'un contact qui porte un IDE ne déclenche **aucun** avertissement franc — ni à l'ouverture, ni après avoir retapé le même IDE ;
4. **vider le champ IDE pendant que la sonde est en vol**, puis résoudre avec un lot contenant un contact **sans IDE** ⇒ aucun avertissement. *(La garde est structurelle dans la 22-2a ; ce test vérifie que le câblage lui passe bien la valeur **envoyée**, et non une relecture du champ.)* ;
5. **le retrait** : afficher l'avertissement sur un IDE pris, corriger un chiffre vers un IDE **libre** ⇒ plus aucun avertissement. Mutation : « n'écrire l'avertissement que dans la branche `if (holder)` », qui laisse à l'écran un message désignant un numéro que l'utilisateur ne porte plus ;
6. **E2E** : le `409` est bien levé si l'avertissement est ignoré (cf. T-b6).

**AC-b3 — Les sondes ne changent RIEN à l'état du bouton (D-b1).** *(Propriété différentielle, pas absolue.)*
L'état du bouton d'enregistrement est **exactement celui qu'il aurait sans les sondes** : ni `disabled` ni `formValidation` ne sont touchés.
⚠️ **Formuler l'AC en absolu — « le bouton reste actif » — serait FAUX** dans des états que la story couvre par ailleurs : le formulaire désactive **déjà** le bouton, aujourd'hui et sans rapport avec cette story, quand une `Personne` n'a qu'un des deux champs de nom (`:276-279`), quand l'adresse est partielle (`:284-285`), ou quand l'IDE est partiellement tapé (`:286-288`). Un dev qui écrirait la preuve sur l'un de ces états verrait un bouton désactivé et pourrait « corriger » `formValidation` — c'est-à-dire **supprimer une garde préexistante pour satisfaire une AC mal bornée**.
*Preuve*, **2** tests :
1. une assertion E2E, jouée sur un formulaire **par ailleurs valide** (cf. T-b6) ;
2. **un test de composant sur un état COMPOSÉ** : une garde préexistante désactive le bouton (adresse partielle, par exemple) **pendant** qu'un avertissement de doublon est actif ⇒ le bouton est désactivé **exactement comme il le serait sans la sonde**, et `formValidation` rend le message d'adresse, pas autre chose.
⚠️ **Sans ce second test, une propriété universelle n'est prouvée qu'en UN point de l'espace d'états** — celui, précisément, où aucune garde préexistante ne joue. Or c'est dans les états composés qu'un développeur est tenté de « corriger » `formValidation` pour faire passer sa preuve. Relevé en passe 1.

**AC-b4 — Rien ne part avant que la frappe se calme, et une réponse tardive ne dit plus rien.**
Temporisation de 300 ms (le pas du dépôt), compteur de génération, **une paire par sonde** (D-b7), état remis à zéro à l'ouverture (D-b8).
*Preuve*, **6** tests unitaires :
1. **le nombre d'appels** sur une frappe continue de vingt caractères, minuteries factices — **attendu : un seul** ;
2. deux requêtes résolues **dans l'ordre inverse** — la première arrivée en dernier est ignorée ;
3. **le test croisé** : armer la sonde nom, puis la sonde IDE avant résolution ⇒ **les deux** avertissements aboutissent. Mutation : « factoriser une seule paire », que les deux autres laissent verte ;
4. **la sonde IDE sur une suite valide → invalide → valide** ⇒ **deux** appels, un par passage à la validité ;
5. **la remise à zéro à l'ouverture en CRÉATION** : armer une sonde, fermer le formulaire, le rouvrir ⇒ aucune proposition, et la réponse armée avant la fermeture n'en fait pas apparaître. Mutation : « ne rien réinitialiser dans `openCreate()` » ;
6. **la remise à zéro à l'ouverture en ÉDITION** : armer une sonde depuis « Créer », fermer sans enregistrer, puis ouvrir **en édition** un contact `B` ⇒ aucune proposition résiduelle.
   ⚠️ **Mutation : « ne réinitialiser que dans `openCreate()` », que le test 5 laisse VERTE.** D-b8 exige les deux sites ; une seule preuve n'en couvrait qu'un. Le symptôme est concret : les propositions d'une session « créer » restent affichées sur la fiche de `B`, désignant des contacts sans rapport, jusqu'à la frappe suivante. Relevé en passe 1.
⚠️ Ne PAS prouver l'annulation par un `AbortSignal` passé à `apiClient` : il est écrasé en silence (D-b6), un tel test mesurerait du vide.

**AC-b5 — Le dispositif est muet quand il n'a rien à dire.**
- **(a)** la sonde non armée par le module de la 22-2a ⇒ **aucune requête** ;
- **(b)** armée, aucun contact proche ⇒ aucun encombrement, **aucun texte** ;
- **(b-bis)** **DÉSARMER, C'EST EFFACER** — ramener le terme sous le seuil, vider le champ IDE, ou basculer de type sans rien retaper : les propositions et l'avertissement **disparaissent**, et une réponse partie avant le désarmement ne les repeuple pas ;
- **(c)** une sonde qui **échoue** est muette elle aussi — réseau coupé, `500`, réponse illisible : aucun message d'erreur **propre à la sonde**, aucune exception remontée, et **l'autre sonde continue**. Une sonde n'est pas une action de l'utilisateur : il n'a rien demandé, on ne lui doit aucun rapport d'échec.
  ⚠️ **La promesse est bornée à ce que cette story maîtrise, et il faut le dire.** Sur réseau coupé, timeout ou `503`, `api-client.ts:316-359` appelle `apiHealth.setDegraded()` **à l'intérieur** de l'`apiClient` : le bandeau global apparaît, et le `GET` est **rejoué jusqu'à cinq fois sur 14,3 s** (`DEGRADED_RETRY_DELAYS_MS`). Aucun `try/catch` de l'appelant ne l'empêche, et une sonde toutes les 300 ms multiplie les occasions de le lever. Risque **accepté**, au même titre que celui du `401` — cf. § *Ce que la story ne répare pas*. La preuve, écrite contre un `vi.mock`, ne traverse jamais l'`apiClient` : elle ne mesure que la moitié qui nous revient.
*Preuve*, **5** tests de composant : pour (a), la saisie de deux caractères ne déclenche **aucun** appel ; pour (b), un carnet vide ne rend **aucun texte** — ⚠️ asserter l'absence de **texte**, jamais l'absence de **nœud**, les zones restant en permanence dans le DOM pour qu'`aria-live` fonctionne (T-b4) ; pour **(b-bis)**, **deux** tests — ramener le terme de trois à deux caractères **vide** les propositions, et vider le champ IDE **retire** l'avertissement franc, mutation « sortir par un `return` nu sans rien effacer » ; pour (c), la sonde IDE répond `500` pendant que la sonde nom répond `200` ⇒ aucun message d'erreur, et les propositions s'affichent normalement, mutation « retirer le `try/catch` d'une sonde ».

**AC-b6 — Les avertissements sont traduits dans les quatre locales.**
*Preuve*, **2** tests Rust dans `crates/kesh-i18n/src/loader.rs` — le premier sur les clés neuves, le second sur `contact-filter-search-placeholder`, dont les quatre valeurs doivent mentionner l'IDE **avec le sigle de LEUR locale** une fois T-b1 livrée :
```rust
for (locale, jeton) in [(Locale::FrCh,"IDE"),(Locale::DeCh,"UID"),(Locale::ItCh,"IDI"),(Locale::EnCh,"UID")] {
    let v = bundle.format(&locale, "contact-filter-search-placeholder", None);
    assert!(v.contains(jeton), "{locale:?} n'énumère pas l'IDE ({jeton})");
}
```
⚠️ **Le second est indispensable et le premier ne le couvre pas** : AC-b6 ne teste que les clés **neuves**, or ce libellé **existe déjà** et n'est que *modifié*. Un développeur qui met à jour `fr-CH` en oubliant les trois autres ne fait rougir **rien** — la clé reste présente, différente de son nom, et différente du français (ce sont des traductions distinctes). C'est le seul item de la story dont l'oubli partiel échappe à tous les gates. Relevé en passe 1.
Le premier test suit le patron de `client_number_labels_are_translated_in_all_four_locales`, sur le patron de `client_number_labels_are_translated_in_all_four_locales` (`loader.rs:260-277`) — pour chaque clé neuve, `!= key` **et** `!= fr` sur `de-CH`, `it-CH`, `en-CH`. Plus `npm run lint-i18n-ownership` vert.
⚠️ **Il n'existe AUCUN test de parité globale dans `kesh-i18n`.** Le loader **replie silencieusement sur le français** (`format_missing_key_in_de_falls_back_to_fr`, `loader.rs:227`), et les fichiers sont déjà désappariés de 57 clés (KF #283). Une clé oubliée dans trois locales **ne rougit nulle part** : c'est l'assertion `!= fr` qui l'attrape, et rien d'autre.
⚠️ Le domaine `contact-*` compte **62 clés dans chacune** des quatre locales — recompté le 2026-08-17. Le laisser apparié.

**AC-b7 — L'avertissement vaut aussi à l'édition, sans se signaler lui-même.**
Renommer un contact vers un nom déjà porté déclenche le même signal nuancé, le contact édité étant exclu de ses propres résultats.
*Preuve*, **2** tests de composant :
1. ouvrir la fiche d'un contact `Entreprise`, modifier sa raison sociale d'un caractère ⇒ il **ne figure pas** dans les propositions ;
2. **et AUCUNE mention « et N autres » n'apparaît** quand il était le seul à correspondre. ⚠️ Le compteur est calculé par la 22-2a, mais **c'est ici qu'on vérifie qu'on lui passe bien `editing?.id`** : sans cela, corriger un caractère affiche « et 1 autre » **au-dessus d'une liste vide**, en désignant la fiche qu'on modifie. La preuve 1 reste **verte** sous ce défaut, puisqu'elle ne regarde que la liste.

## Tasks / Subtasks

- [x] **T-b1 — Rendre l'IDE cherchable, et prouver la fenêtre** ✅ *(2026-08-18)* (AC-b2, **et preuve 9 d'AC-b1** ; met en œuvre **D-b3**). Ajouter `ide_number` aux **DEUX** branches `LIKE` de `push_where_clauses` (`crates/kesh-db/src/repositories/contacts.rs:195-210`).
  - [ ] Deux tests `#[sqlx::test]`, **un par branche** : le cas courant (`search=CHE109322551`) et le cas où `escaped.is_empty()`, c'est-à-dire un terme **intégralement** composé des dix opérateurs — par exemple `search=***`.
  - [ ] ⚠️ **Ne PAS se caler sur `test_search_handles_special_chars:1283` pour la seconde branche : il ne l'exerce pas.** Il cherche `"100%"`, et `%` **n'est pas** un opérateur `BOOLEAN MODE` (`util/search.rs:41`) — le terme survit intact et le test emprunte la branche `else`. **Aucun test du dépôt n'exerce aujourd'hui la branche `escaped.is_empty()`.**
  - [ ] **Le `#[sqlx::test]` de la fenêtre** (preuve 9 d'AC-b1) : fixture des six `Jean X`, `limit: 20` ⇒ six lignes et `total = 6`.
  - [ ] ⚠️ **`push_where_clauses` est un chemin PARTAGÉ** — cf. § *Rayon d'impact de T-b1*. Gate **complet** au dernier commit : la § *Test Locally First* interdit le ciblage dès qu'un patch touche `crates/kesh-db/`.
- [x] **T-b2 — Brancher les deux sondes** ✅ *(2026-08-18, code — preuves à venir)* (AC-b1, AC-b2, AC-b5, **AC-b7**). Importer le socle de la **22-2a** et **ne réimplémenter aucune de ses fonctions**.
  - [ ] Forme exacte des deux appels :

```ts
import { probeTerm, rank, excludeSelf, countOthers, findIdeHolder }
  from '$lib/features/contacts/duplicate-probe';

const soi = editing?.id ?? null;                 // `number | null`, jamais `undefined`

// ── AC-b1 — sonde « nom proche ». Actifs SEULEMENT (D-b5), fenêtre large.
const { normalized, armed } =
  probeTerm(formContactType, formName, formFirstName, formLastName);

if (!armed) { proches = []; autres = 0; nameSeq++; return; }   // ⚠️ DÉSARMER, c'est EFFACER
const seq = ++nameSeq;
const rn = await listContacts({ search: normalized, limit: 20, includeArchived: false });
if (seq !== nameSeq) return;                                   // garde d'ordre (D-b6)
const retenus = rank(excludeSelf(rn.items, soi), normalized);
proches = retenus.slice(0, 5);                                 // rank ne tronque PAS
autres  = countOthers(rn.total, rn.items, proches, soi);

// ── AC-b2 — sonde IDE. Archivés COMPRIS (D-b5), vérification sur le champ.
const ide = normalizeIdeForApi(formIde);
if (!ide || !validateIdeFormat(formIde)) { holder = undefined; ideSeq++; return; }
const s2 = ++ideSeq;
const ri = await listContacts({ search: ide, limit: 5, includeArchived: true });
if (s2 !== ideSeq) return;
holder = findIdeHolder(ri.items, ide, soi);                    // `ide` = la valeur ENVOYÉE
```

  - [ ] ⚠️ **`probeTerm`, jamais la recomposition à la main.** Composer soi-même `buildTerm → normalizeTerm → isArmed` remet l'**ordre des opérations** chez l'appelant — c'est-à-dire hors de tout module testable. La mutation « mesurer le seuil avant de normaliser » redeviendrait injouable, et c'est le trou que le découpage a fermé.
  - [ ] ⚠️ **DÉSARMER, C'EST EFFACER.** Les deux gardes écrivent l'état **avant** de sortir, et **incrémentent leur compteur**. Sortir par un `return` nu laisse à l'écran des propositions calculées sur un terme que l'utilisateur a déjà effacé, et laisse une réponse en vol les repeupler. ⚠️ **Sans cela, la preuve 7 d'AC-b1 est littéralement insatisfiable** : basculer `Entreprise → Personne` sans effacer rend un terme vide, donc non armé, donc le `return` — et l'avertissement de l'ancien type **subsiste**.
  - [ ] ⚠️ **`editing?.id ?? null`, jamais `editing?.id` nu.** `editing` est `ContactResponse | null`, donc `editing?.id` rend `number | undefined`, que la signature `number | null` du socle refuse : `npm run check` rejetterait le code.
  - [ ] ⚠️ **C'est ICI qu'AC-b7 se joue**, pas dans le balisage de T-b4 : ce bloc est l'unique site où `soi` atteint `excludeSelf`, `countOthers` et `findIdeHolder`.
  - [ ] ⚠️ **Les deux appels ne diffèrent que par deux paramètres et se lisent côte à côte.** Une inversion d'`includeArchived` ne casse rien et rend des résultats plausibles dans les deux sens — le filtrage étant serveur, et un `vi.mock` ne regardant pas ses arguments. D'où les preuves **sur l'argument**, une par sens.
- [x] **T-b3 — Temporisation et garde d'ordre** ✅ *(2026-08-18, code — preuves à venir)* (AC-b4). Débounce 300 ms + compteur de génération, patron `ContactPicker.svelte:36-78` (D-b6). **Une paire par sonde** (D-b7).
  - [ ] Réutiliser `debounce` de `$lib/features/journal-entries/debounce.ts` **ou** le `setTimeout` inline de `+page.svelte:185-192` — ne pas écrire un troisième mécanisme. Le helper est mal rangé ; le **déplacer** est hors périmètre, s'en servir ne l'est pas.
  - [ ] Remise à zéro des deux paires dans `openCreate()` **et** `openEdit()` (D-b8) — **pas** sur la fermeture, qui a trois sites.
  - [ ] Nettoyer les timers au démontage — `+page.svelte:113` le fait déjà pour la recherche de liste.
  - [ ] Brancher sur **`oninput`** (`ContactPicker.svelte:127`, `+page.svelte:463`), **jamais sur `onkeydown`** : un collage ne produit aucune frappe clavier, et une sonde branchée sur le clavier resterait muette sur le geste le plus courant de tous.
- [x] **T-b4 — Balisage et signaux** ✅ *(2026-08-18, code — preuves à venir)* (AC-b1, AC-b2, AC-b3, AC-b5, AC-b7). Deux niveaux visuellement distincts (D-b2) ; ne toucher **ni** au `disabled` du bouton **ni** à `formValidation` (`:275`).
  - [ ] **Chaque avertissement est rendu IMMÉDIATEMENT SOUS le champ qui le déclenche** — propositions sous le nom, avertissement franc sous `#form-ide` — et **jamais** dans le bloc `formError` du bas : le `Dialog.Content` est en `max-h-[90vh] overflow-y-auto` (`:637`) et **défile en interne**. Un avertissement hors écran est un avertissement muet.
  - [ ] ⚠️ **La zone « nom proche » est placée HORS des deux branches du `{#if formContactType === 'Personne'}`** (D-b11), après le bloc type-dépendant. Logée dans une branche, elle serait **démontée à chaque bascule de type** — donc ni permanente, ni annoncée, ce qui contredit la sous-tâche suivante.
  - [ ] **Le `<select id="form-type">` réarme la sonde** au même titre que les champs de nom (D-b11) : les valeurs saisies survivent à la bascule, l'avertissement doit suivre.
  - [ ] **Les deux zones sont rendues EN PERMANENCE dans le DOM**, `aria-live="polite"`, contenu vide quand il n'y a rien à dire. Patron : `ResetPasswordForm.svelte:162-169`, dont le commentaire dit « *toujours dans le DOM pour que aria-live fonctionne* ». ⚠️ **Ne PAS copier le `role="combobox"` de `ContactPicker`** : c'est un patron de **sélection**, inadapté à un avertissement passif (D-b9), et il ne porte aucun `aria-live`.
  - [ ] Encapsuler **chaque** sonde dans un `try/catch` qui se tait, comme `ContactPicker.svelte:64-67`.
  - [ ] **Rendre « et N autres » SOUS la liste des cinq propositions**, conditionné à `autres > 0` — clé `contact-duplicate-others-count`, argument `count`. ⚠️ Conditionner sur `autres > 0` et non sur `total > 5` : c'est la soustraction de D-a6 qui fait foi, et elle vaut zéro dans le cas de l'édition solitaire (AC-b7 preuve 2).
  - [ ] **L'avertissement franc s'efface quand `formError` prend le relais** après un `409` (`:361` pose déjà `contact-error-ide-duplicate`), pour que la même phrase ne s'affiche pas deux fois.
  - [ ] Il n'existe pas de composant `alert` dans `lib/components/ui/` — se caler sur le style de `formError`, sans créer de primitive.
  - [ ] **Créer `frontend/src/routes/(app)/contacts/contacts-page.test.ts`** ⚠️ **PAS `+page.test.ts`** : SvelteKit réserve tout nom préfixé `+` dans `src/routes/`, et le `build` échoue en `Files prefixed with + are reserved`. C'est pourquoi le patron s'appelle `products-page.test.ts`. Ni `npm run check` ni `vitest` ne le voient — **seul le `build` l'attrape**. — il n'existe **aucun** test de cette page (seul `contact-helpers.test.ts` existe pour ce domaine). Patron : `products-page.test.ts`, mocks hoistés avant l'import.
- [x] **T-b5 — i18n** ✅ *(2026-08-18 — 4 clés × 4 locales, libellé de recherche, et les 2 tests Rust ; 4 mutations jouées)* (AC-b6). Clés neuves dans les **quatre** FTL, domaine `contact-*`, plus le test Rust dédié.
  - [ ] **Clés neuves, NOMMÉES** — les décrire sans les nommer rendait le test d'AC-b6 inécrivable :

| Clé | Rôle | Argument |
|---|---|---|
| `contact-duplicate-heading` | en-tête du bloc de propositions | — |
| `contact-duplicate-others-count` | « et {$count} autres » | `count` |
| `contact-duplicate-ide-active` | avertissement franc, porteur **actif** | `name` |
| `contact-duplicate-ide-archived` | avertissement franc, porteur **archivé** — **distinct**, c'est la preuve 2 d'AC-b2 | `name` |

  - [ ] ⚠️ **Ce correctif avait été DÉCLARÉ en passe 2 sans être appliqué** : un remplacement scripté sans garde de correspondance a été un no-op silencieux, et le Change Log fut rédigé depuis l'intention. **Vérifier au `grep` que le tableau ci-dessus est bien dans le fichier** avant de l'affirmer une seconde fois.
  - [ ] ⚠️ **Le sigle de l'IDE SUIT LA LOCALE** : `IDE` en fr-CH, **`UID`** en de-CH et en-CH, **`IDI`** en it-CH (`contact-col-ide`, vérifié dans les quatre FTL). Un test écrivant naïvement `assert!(v.contains("IDE"))` sur les quatre échouerait sur trois ; « corrigé » en poussant `IDE` partout, il **introduirait** une régression de terminologie que rien d'autre ne rattrape.
  - [ ] ⚠️ **Ne PAS réutiliser `contact-error-ide-duplicate`** (`fr-CH:374`) : elle sert au message d'échec du `409`, et ne sait ni nommer le porteur ni dire qu'il est archivé.
  - [ ] **Mettre à jour `contact-filter-search-placeholder`** dans les 4 locales **et** son fallback en dur (`+page.svelte:455`) : il **énumère les colonnes cherchées**, et T-b1 en ajoute une. ⚠️ **Formulation imposée** : « … ou IDE **sans séparateurs** ». Écrire « ou IDE » tout court promet ce que D-b3 sait faux — `'CHE109322551' LIKE '%CHE-109.322.551%'` rend **0**.
  - [ ] **Écrire le second test Rust d'AC-b6**, qui vérifie les **quatre** valeurs de cette clé. ⚠️ C'est une clé **modifiée**, pas neuve : le test des clés neuves ne la voit pas, et sans ce second test, oublier trois locales ne fait rougir aucun gate.
- [ ] **T-b6 — E2E** (AC-b3, **et preuve 6 d'AC-b2**). ⚠️ Le fichier **DOIT** être nommé `*.spec.ts` : `playwright.config.ts:35` filtre sur `testMatch: /(.+\.)?spec\.[jt]s/`, et un `*.test.ts` posé dans `tests/e2e/` est **silencieusement ignoré**.
  - [ ] Patron : `frontend/tests/e2e/contact-client-number.spec.ts`, `seedTestState('with-company')` puis login.
  - [ ] ⚠️ **ARCHIVER NE LIBÈRE PAS L'IDE** — contrainte **plate** (D-b5), contrairement au numéro de client dont le patron vient. Chaque assertion emploie un **IDE distinct à checksum valide** : `CHE109322551` et `CHE123456788`. ⚠️ **Les deux ont été recalculés** — poids `[5,4,3,2,7,6,5,4]`, modulo 11 (`che_number.rs:9,101-129`) : `109322551` attend 1 et porte 1, `123456788` attend 8 et porte 8. *(Une première rédaction proposait `CHE116281838` en le déclarant « vérifié » : son checksum attend **9** et il porte **8** — il est **invalide**, et le test aurait échoué en `400` au lieu du succès ou du `409` attendu, pour une raison sans rapport avec ce qu'il mesure. Relevé en passe 1.)* L'isolation entre exécutions repose sur le `truncate_all` de `seedTestState`. Et l'astuce `CLI-${Date.now()}` de la 22-1 est **intransposable** : le dernier chiffre d'un IDE est un checksum modulo 11 (`che_number.rs:102-131`).
  - [ ] ⚠️ **Ce que l'IDE distinct protège, c'est le MONTAGE du test 2** — pas la collision qu'il cherche à provoquer : le contact laissé par le test 1 ne doit pas brûler l'IDE dont le test 2 a besoin. **Les deux tests PARTAGENT une base**, comme le patron : un seul `seedTestState('with-company')` en `beforeAll`, aucun `beforeEach` intermédiaire. Le test 1 laisse donc son contact en base pour le test 2. C'est pourquoi **chaque test emploie son propre IDE** — et pourquoi l'archivage ne suffirait pas à les isoler : il libère le numéro de client, **jamais l'IDE** (D-b5). L'isolation entre *exécutions* vient du `truncate_all` de `seedTestState`, pas d'un nettoyage en fin de test.
  - [ ] ⚠️ **LE PRESET NE SEEDE AUCUN CONTACT — chaque test monte le sien.** `with-company` n'appelle que `seed_accounting_company` + `mark_onboarding_complete` (`test_endpoints.rs:183-195`) ; le seul `INSERT INTO contacts` du dépôt est dans `seed_contact_and_product`, réservé au preset `with-data`. **Sans montage, les assertions (i) sont insatisfiables** : un signal nuancé exige un contact au nom proche préexistant, un signal franc exige un porteur de l'IDE tapé, et la table est **vide**. Le test partirait rouge pour une raison sans rapport avec ce qu'il mesure — et le geste le moins coûteux serait alors d'affaiblir (i), c'est-à-dire la seule preuve E2E de la story.
  - [ ] **Montage du test 1** : créer d'abord `Dubarde Vins SA` (IDE `CHE109322551`), puis rouvrir « Nouveau contact ».
  - [ ] **Montage du test 2** : créer d'abord un porteur de `CHE-123.456.788`, puis rouvrir « Nouveau contact ».
  - [ ] **Test 1 (AC-b3), trois assertions DANS CET ORDRE**, sur une `Entreprise` par ailleurs valide : (i) **le signal nuancé est VISIBLE** ; (ii) le bouton est **activé** ; (iii) soumettre **crée bien le contact**, vérifié après rechargement.
  - [ ] **Test 2 (preuve 6 d'AC-b2), trois assertions DANS CET ORDRE** : (i) **le signal franc est VISIBLE** ; (ii) le bouton est **activé** ; (iii) soumettre **échoue en `409`** et l'interface le dit.
  - [ ] ⚠️ **La première assertion de chaque test n'est PAS décorative : c'est la seule qui prouve quelque chose de cette story.** Les assertions (ii) et (iii) sont **VERTES SUR `main` AUJOURD'HUI** — créer un contact fonctionne déjà, un IDE dupliqué donne déjà `409`. Sans (i), la mutation « **n'implémenter aucune sonde** » laisse les deux tests au vert.
- [ ] **T-b7 — Documentation** (AC-b1, AC-b2). `docs/manual/fr/user-manual.tex`, § *Contacts* → *Création d'un contact* (ligne 506) : ce que l'avertissement dit, et **qu'il n'empêche rien**. Régénérer le PDF (`make fr`) et le commiter.
  - [ ] **Corriger `user-manual.tex:532`**, qui **énumère** les colonnes cherchées (« sur le **nom**, complété par une recherche sur l'**email** et le **numéro de client** ») — T-b1 en ajoute une.
  - [ ] ⚠️ **Écrire « le numéro IDE, saisi SANS SÉPARATEURS (`CHE109322551`) », et NE PAS étendre à l'IDE la phrase de la ligne 534.** Celle-ci dit *« C'est ce qui permet de remonter d'une facture papier au contact : saisissez le numéro imprimé sur le document »* — or un IDE imprimé s'écrit `CHE-109.322.551`, et cette saisie-là rend **zéro** (D-b3). Trois lignes séparent les deux phrases : un lecteur fera le lien, essaiera, et n'obtiendra rien. C'est le contraire du service annoncé.
  - [ ] ⚠️ Ne PAS toucher à la ligne 538 (« *Contacts → Import CSV* »), qui annonce une fonction inexistante : c'est **#291**, une issue à part. **À ne pas confondre avec la 532.**

## Décompte des preuves — la seule table qui fait foi

| AC | Preuves | Nature |
|---|---:|---|
| AC-b1 — nom proche | 8 + 1 | composant + `#[sqlx::test]` |
| AC-b2 — IDE déjà pris | 5 + 1 | composant + E2E |
| AC-b3 — l'état du bouton est inchangé | 1 + 1 | E2E + composant |
| AC-b4 — temporisation et ordre | 6 | unitaire front |
| AC-b5 — muet quand rien à dire | 5 | composant |
| AC-b6 — quatre locales | 2 | unitaire Rust |
| AC-b7 — édition sans se signaler | 2 | composant |
| *(T-b1 — l'IDE cherchable)* | 2 | `#[sqlx::test]` |

⚠️ **Les 21 preuves de composant sont écrites et LEURS MUTATIONS JOUÉES** — `frontend/scripts/mutants-22-2b.mjs`, 12 mutations, 0 survivante, 0 hors cible. **TROIS d'entre elles ne prouvaient RIEN avant d'être jouées** : ouvrir une fiche en édition sans retaper l'IDE ne déclenche aucune sonde ; le compteur « et N autres » n'est pas rendu quand la liste est vide ; et une promesse rejetée sans `try/catch` ne fait pas rougir vitest. Aucune relecture ne les aurait vues.

**Totaux, sommés depuis la colonne** : **21 tests de composant** (8 + 5 + 1 + 5 + 2) · **6 tests unitaires front** · **2 assertions E2E** (1 + 1) · **2 tests unitaires Rust** · **3 tests d'intégration base** (1 + 2). **Soit 34 preuves.**

Aucun autre passage de cette story n'énonce de total. *(S'y ajoutent les **44 preuves** de la 22-2a — écrites, exécutées, et gardées par 24 mutations jouées.)*

## Dev Notes

### Rayon d'impact de T-b1 — un chemin partagé, pas un chemin à nous

`push_where_clauses` sert **tout** ce qui liste des contacts :

| Consommateur | Effet de l'ajout d'`ide_number` |
|---|---|
| La recherche du carnet (`+page.svelte:462`) | chercher un IDE **sans séparateurs** y remonte son contact |
| `ContactPicker.svelte` (choix du débiteur sur une facture) | même élargissement, non demandé, non nuisible |
| Tout test comptant des résultats de recherche | un terme qui matche aussi un IDE peut changer un `total` |
| **Le libellé qui DÉCRIT la recherche** | `contact-filter-search-placeholder` **énumère les colonnes** dans 4 FTL + fallback en dur `:455` |
| **Le manuel utilisateur** | `user-manual.tex:532` énumère lui aussi nom / email / numéro de client |

Tests à surveiller nommément : `test_filter_by_search_name:1225`, `test_search_handles_special_chars:1283`, `test_search_no_longer_matches_mid_word:1328`. Aucun ne devrait bouger — **c'est à vérifier par exécution, pas par lecture**.

### Le piège de cette story

Un avertissement trop bavard **ne protège plus rien** : on apprend à le fermer sans le lire. C'est le vrai risque, plus que le faux négatif. Le seuil doit être **serré** — mieux vaut manquer un doublon que crier trois fois par jour à tort. AC-b7 (ne pas se signaler soi-même) et la vérification sur le champ (D-b5/22-2a) ne sont pas des raffinements : ce sont les deux façons les plus rapides de rendre le dispositif inaudible dès la première semaine.

### Les quatorze pièges muets, nommés

Aucun ne casse la compilation, ne fait rougir un test, ni ne produit d'erreur au runtime.

| Piège | Symptôme | Garde |
|---|---|---|
| `signal` passé à `apiClient` (D-b6) | l'annulation ne fait rien, on croit l'avoir | AC-b4, prouvée par ordre de résolution |
| sonde IDE sur les actifs seuls (D-b5) | silence, puis `409` sans coupable visible | AC-b2 preuve 2, **sur l'argument** |
| sonde nom sur les archivés (D-b5, inverse) | propositions polluées par des fiches mortes | AC-b1 preuve 4, **sur l'argument** |
| preuve d'archivage écrite fonctionnellement | le mock ignore ses arguments ⇒ verte sous la mutation | assertions **sur l'argument**, les deux sens |
| assertion E2E sans la visibilité du signal | verte sur `main`, avant qu'une ligne soit écrite | T-b6, la visibilité **en premier** |
| une seule paire `(timer, compteur)` (D-b7) | l'un des deux avertissements ne s'affiche jamais | AC-b4 preuve 3, le test croisé |
| une proposition rendue cliquable (D-b9) | efface les 19 champs de la saisie en cours | AC-b1 preuve 5 |
| remise à zéro dans `openCreate()` seul | les propositions d'une session « créer » survivent sur une fiche éditée | AC-b4 preuve 6 |
| une clé i18n MODIFIÉE dans une seule locale | le test des clés NEUVES ne la voit pas | AC-b6 preuve 2 |
| `search` non épinglé par aucune preuve | `search: formIde` ⇒ sonde **muette pour tout utilisateur réel** | AC-b2 preuve 2, **sur `search`**, en forme séparée |
| assertion sur l'argument non discriminée | **satisfaite par l'appel que la page fait au montage** | AC-b1 preuve 4 : filtrer sur `search`, puis `toEqual` |
| `return` nu au désarmement | avertissement figé sur un terme déjà effacé | AC-b5 (b-bis), mutation « sortir sans effacer » |
| E2E monté sur un preset qui ne seede rien | (i) part rouge sans rapport ⇒ on l'affaiblit | T-b6, montage écrit test par test |
| clé i18n absente de 3 locales (AC-b6) | repli silencieux sur le français | assertion `!= fr`, la seule qui l'attrape |

### Ce que la story ne répare pas, et qu'elle rend plus atteignable

⚠️ **Une session qui expire pendant la composition d'une fiche fait perdre la saisie.** Tout `401` — y compris sur un `GET` de fond — déclenche un refresh, et si le refresh échoue, `api-client.ts:78-79,88-98,157-158` fait `clearSession()` puis `window.location.replace('/login?reason=session_expired')` : une redirection **plein document**. Le formulaire, non enregistré, disparaît.

C'est **préexistant**, et cette story ne le corrige pas — mais elle le rend **plus probable d'être atteint** : il fallait jusqu'ici cliquer « Enregistrer » pour émettre une requête depuis ce formulaire, désormais des sondes partent toutes les 300 ms. Le risque est **accepté explicitement** ; le `try/catch` de T-b4 protège du reste (réseau, `500`) mais pas de celui-là, qui se produit dans l'`apiClient` avant que l'appelant ne reprenne la main. Si le sujet doit être traité, c'est une issue à part.

### Ce qui existe déjà, et qu'il ne faut pas réinventer

**Le typeahead.** `frontend/src/lib/components/invoices/ContactPicker.svelte` fait déjà l'essentiel du câblage : débounce 300 ms, compteur `searchSeq`, nettoyage au démontage. **C'est le précédent à lire avant d'écrire une ligne.**
⚠️ Ce qu'il ne faut **pas** copier : ses libellés français en dur, non traduits. Et ce qu'il faut copier sans discuter : `$props.id()`, **jamais** `crypto.randomUUID()` — cette API n'existe qu'en contexte sécurisé et crashe le rendu sur un déploiement HTTP LAN comme le NAS (#145).

**Les helpers IDE.** `contact-helpers.ts` : `validateIdeFormat` (format seul), `normalizeIdeForApi` (`CHE-123.456.789` → `CHE123456789`), `formatIdeNumber`. Le checksum n'est validé qu'au backend par `CheNumber::new` (`routes/contacts.rs:312-322`) : un IDE au bon format mais au mauvais checksum passe la porte de la sonde et sera refusé en `400` à l'enregistrement. Acceptable — on cherche un doublon, on ne valide pas un IDE.

### Livraison

⚠️ **Les deux moitiés se mergent dans UNE SEULE PR**, qui porte `closes #301`. La 22-2a seule ne livre rien de visible ; la 22-2b seule ne compile pas. Le mot-clé de fermeture se porte sur la **PR**, pas sur les commits intermédiaires — le dépôt merge en squash, donc le message final est le titre et le corps de la PR. Vérifier après merge : `gh issue view 301 --json state --jq .state`.

### Conventions de test

**Backend** : `#[sqlx::test]` monte le squash `crates/kesh-db/test-schema/` — ne pas ajouter d'attribut sur le vrai `MIGRATOR` sans lire `crates/kesh-db/tests/test_schema_guard.rs`.
**Frontend** : `@testing-library/svelte` v5 + Svelte 5, mocks hoistés **avant** l'import du composant. Patron : `products-page.test.ts`.
**Mutations jouées, pas raisonnées.** Les affirmations d'absence se vérifient au `grep -nF` avant d'être écrites — et le grep porte sur la **valeur**, jamais sur la phrase qui l'entoure.

### References

- Story **22-2a** — le module pur que cette story consomme. **À implémenter d'abord.**
- Story **22-2** (umbrella, statut `split`) — les quatre passes de revue et leur Change Log complet, source de toutes les décisions.
- Issue **#301** — le besoin. **#302** — la succession d'entreprise, à ne pas confondre. **#291** — le manuel et son import CSV inexistant, à ne pas traiter ici.
- ⚠️ **Issues #314 et #315 — les deux corrections À LA SOURCE que cette story CONTOURNE délibérément.** #314 : `escape_boolean_ft` supprime les opérateurs au lieu de les remplacer (4 repositories, 5 sites d'appel). #315 : la recherche n'a aucun tri par pertinence, et la sémantique est OU inclusif. La 22-2a contourne les deux **côté client** — normalisation du terme et classement local. **Ce sont des atténuations assumées, pas des corrections** : ne pas les traiter ici, et ne pas laisser croire dans le code qu'elles règlent le problème de fond.
- Story **22-3** (#300) — la fusion, en veille : ce que cette story doit rendre inutile.
- `CLAUDE.md` — § *Un appariement automatique propose, il ne crée jamais*, § *Test Locally First*, § *Un gate laisse la base piégée*.

## Change Log

### Passe 1 de `bmad-create-story validate` — 2026-08-17, Sonnet ×3, contexte frais

Bruts : 1 + 4 + 5 = 10. Une convergence. **Retenus : 0 CRITICAL / 4 HIGH / 4 MEDIUM / 1 LOW — 9 findings, 9 correctifs appliqués.**

⚠️ **Zéro `CRITICAL`, et le BlindHunter a rendu un rapport quasi vide** (1 `LOW`) après avoir contre-vérifié une trentaine d'ancres sur les deux moitiés. C'est le premier rapport de ce dossier qui ne trouve rien de substantiel — le découpage travaille.

**Les quatre `HIGH` :**

1. **`CHE116281838` était FAUX.** La spec l'annonçait « vérifié » ; recalculé au modulo 11 (`che_number.rs:9,101-129`), son checksum attend **9** et il porte **8**. Les deux seules assertions E2E de la story l'auraient employé, et auraient échoué en `400` — pour une raison **sans rapport** avec ce qu'elles mesurent, ce qui est la meilleure façon de faire affaiblir une assertion par un développeur pressé. Remplacé par `CHE123456788`, **recalculé**. ⚠️ *J'avais écrit « tous deux vérifiés » sans l'avoir fait : c'est exactement la faute que ce dossier documente depuis quatre passes.*
2. **La remise à zéro n'était prouvée que sur `openCreate()`.** D-b8 exige les deux sites ; la mutation « ne réinitialiser que dans `openCreate()` » laissait les 26 preuves vertes. Symptôme concret : ouvrir « Créer », taper, fermer, puis **éditer** un contact `B` — les propositions de la session précédente restent affichées sur la fiche de `B`. **Sixième preuve ajoutée à AC-b4.**
3. **La zone d'avertissement contredisait `aria-live`.** Le champ de nom vit dans deux branches d'un `{#if}` selon le type de contact ; une zone logée dans l'une d'elles est **démontée à chaque bascule**, donc ni permanente ni annoncée. **D-b11** la place hors des deux branches — et fait **réarmer la sonde par le `<select>` de type**, que rien ne déclenchait alors que les champs de nom **survivent** à la bascule.
4. **La chaîne de repli d'affichage pouvait s'épuiser.** `email` et `clientNumber` sont tous deux nullables : deux homonymes minimaux — le père et le fils de D-b1 — s'affichaient **rigoureusement identiques**, ce qui contredit l'invariant que D-b10 énonce comme sa propre raison d'être. **Dernier niveau ajouté : `#<id>`**, déjà présent dans le DTO, garanti distinct, sans requête.

**Les quatre `MEDIUM`** : `editing?.id` rend `number | undefined` alors que la 22-2a documente `null` — le code **littéralement prescrit** par cette story aurait été rejeté par `npm run check` (**D-b8-bis**, et c'est le défaut de frontière que le découpage rendait prévisible) ; la mise à jour de `contact-filter-search-placeholder` n'avait **aucune preuve**, la clé étant *modifiée* et non *neuve* — seul item dont l'oubli partiel échappait à tous les gates ; `T-b2` n'était pas taguée `AC-b7` alors qu'elle en porte le mécanisme ; et `AC-b3`, propriété **universelle**, n'était prouvée qu'au seul point de l'espace d'états où aucune garde préexistante ne joue.

**Décompte : 26 → 31 preuves**, recompté depuis les AC. Pièges muets : 8 → 10.

⚠️ **Passe 2 due** par la § *Review Iteration Rule* (4 `HIGH`). Rotation : **Haiku**, contexte frais.

### Passe 2 de `bmad-create-story validate` — 2026-08-17, Haiku ×3, contexte frais

Bruts : 1 + 5 + 0 = 6, plus **1 trouvé par l'orchestrateur**. **Un finding RÉFUTÉ**, un fusionné. **Retenus : 0 CRITICAL / 0 HIGH / 4 MEDIUM / 1 LOW — 5 findings, 5 correctifs.**

**Trend : `0C/4H/4M/1L` → `0C/0H/4M/1L`.** La sévérité maximale **décroît** (`HIGH → MEDIUM`). L'`AcceptanceAuditor` a rendu un **rapport intégralement vide**, après avoir recompté les 31 preuves, les 10 pièges, les 62 clés `contact-*` et **recalculé les deux checksums IDE**.

**Le thème est le même que sur la 22-2a : la dérive de renumérotation.** La passe 1 avait inséré deux preuves dans AC-b1, déplaçant le `#[sqlx::test]` de la fenêtre de la 6ᵉ à la **8ᵉ** place — mais T-b1 le désignait toujours comme « preuve 6 », **à deux endroits**. S'y ajoutait un renvoi **croisé** périmé : cette story annonçait « les **13** preuves de la 22-2a », qui en compte **17** depuis sa propre passe 1.

⚠️ **Une lentille a vu l'un des trois ; l'orchestrateur a trouvé les deux autres** en confrontant systématiquement chaque renvoi numéroté au contenu réel de sa cible. Le geste vaut d'être gardé : après toute insertion dans une liste numérotée, **auditer tous les renvois**, y compris ceux qui pointent l'autre moitié du découpage.

Les deux autres `MEDIUM` : les **quatre clés i18n neuves étaient décrites sans être nommées**, ce qui rendait le test d'AC-b6 inécrivable — elles sont désormais dans un tableau ; et **l'état partagé des deux tests E2E** n'était pas dit : un seul `beforeAll`, aucun `beforeEach`, donc le test 1 laisse son contact en base pour le test 2 — c'est **la raison** pour laquelle chacun emploie son propre IDE, et pourquoi l'archivage ne les isolerait pas (il libère le numéro de client, jamais l'IDE). Le `LOW` : le rendu de « et N autres » n'était prescrit nulle part.

**Un finding RÉFUTÉ au grep, et c'est le mode d'échec documenté du modèle.** Un `HIGH` affirmait que T-b1 « n'énonce pas qu'il faut ajouter `ide_number` à la seconde branche » — or la ligne 166 le dit dans le **titre même de la tâche** (« aux **DEUX** branches `LIKE` »), et le finding **cite cette ligne** avant d'affirmer qu'elle manque. Écarté. C'est exactement ce que prévoit la § *Haiku-specific guardrails* du `CLAUDE.md`, et la raison pour laquelle tout `CRITICAL`/`HIGH` affirmant une absence passe au `grep -nF` avant d'être traité.

⚠️ **Troisième décompte faux de la séance** : cette lentille annonçait « 3 CRITICAL » pour un corps portant 1 `HIGH` et 4 findings mineurs. Sur six lentilles de passe 1 et six de passe 2, **trois** ont mal additionné leurs propres findings.

⚠️ **Passe 3 due** (4 `MEDIUM`). Rotation : **Opus**, contexte frais.

### Remédiation de la passe 3 — 2026-08-18, contre un contrat RÉEL

**Dégel** décidé par Guy une fois le socle codé. Les 18 findings de la passe 3 sont appliqués, et quatre d'entre eux **tombaient d'eux-mêmes** : ils portaient sur des contrats de frontière que la 22-2a **fige désormais par du code qui tourne** — arité de `buildTerm`, type de `editingId`, contrat de `rank`, symétrie du repli. Les corriger plus tôt, contre un contrat supposé, aurait été à refaire.

**Le `CRITICAL` et les deux `HIGH` qui l'accompagnent tiennent en une phrase : aucune preuve n'épinglait `search`, le paramètre dont tout dépend.**

- `search: formIde` au lieu de `search: ide` survivait aux 31 preuves **et** à l'E2E — et rendait la sonde **muette pour tout utilisateur réel**, l'interface affichant et écrivant la forme séparée que le `LIKE` ne matche pas.
- L'assertion « sur l'argument » ajoutée en passe 1 comme *le* correctif était **satisfaite par l'appel que la page fait déjà au montage** : `loadContacts()` passe exactement `limit: 20` et `includeArchived: false`, et le patron place `vi.clearAllMocks()` **avant** le `render`. Vérifié en lançant vitest. La garde annoncée ne gardait rien.
- Trois clauses d'AC-b1 — *classé*, *coupé à cinq*, *et N autres* — n'avaient aucune preuve : trois mutations d'un caractère laissaient les 31 vertes, dont le retrait pur et simple de `rank`.

**Deux `HIGH` de plus, tous deux du même genre : une preuve infaisable.** Le `return` du désarmement n'écrivait aucun état, ce qui rendait la preuve 7 d'AC-b1 **littéralement insatisfiable** ; et le preset `with-company` **ne seede aucun contact**, ce qui rendait insatisfiables les deux assertions E2E — celles dont la story dit qu'elles sont les seules à prouver quelque chose.

**Trois décisions naissent de la remédiation.** **D-b12** écrit la limite résiduelle que `rank` ne peut pas franchir — au-delà de la fenêtre, le doublon exact reste évincé, résidu de #315 — et **écarte** explicitement l'atténuation « ne rien proposer au-delà », qui rendrait le dispositif muet sur les carnets les plus fournis, c'est-à-dire là où le risque est le plus grand. **D-b13** tranche que la sonde propose ce que le serveur a trouvé, email et numéro de client compris, et **corrige l'en-tête** plutôt que de filtrer : c'était le libellé qui mentait, pas la recherche. Et **D-b10** devient un **invariant** — « deux propositions ne sont jamais identiques » — au lieu d'une cascade, qui laissait passer le père et le fils de la même localité.

**Le correctif que la passe 2 avait DÉCLARÉ sans l'appliquer est enfin appliqué** : les quatre clés i18n sont nommées dans un tableau, avec leur argument. Et le sigle de l'IDE **suit la locale** — `IDE` / `UID` / `IDI` / `UID` —, ce qu'un test écrit naïvement aurait « corrigé » en introduisant une régression de terminologie.

⚠️ **Deux dérives m'ont été rattrapées par mes propres gardes pendant cette remédiation, et c'est le fait notable.** Un `assert` de correspondance a annulé un lot entier plutôt que d'écrire à moitié — le contrôle post-patch a vu l'absence, et j'ai réappliqué. Puis l'audit des renvois numérotés a trouvé **deux références périmées** que l'insertion d'une preuve venait de créer. Ce sont exactement les deux fautes de la passe 2, cette fois arrêtées.

**Décompte : 31 → 34 preuves**, recompté depuis les AC. Pièges muets : 10 → **14**.

⚠️ **Aucune passe adversariale n'a relu cette remédiation.** Elle corrige des findings de passe 3 ; elle n'en a pas subi de nouvelle.

## Dev Agent Record

### Agent Model Used

*(à remplir)*

### Debug Log References

### Completion Notes List

### File List
