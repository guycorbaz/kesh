# Story 22.2a : Le socle d'appariement — un module pur, et rien d'autre

## Status

ready-for-dev

Story-zéro du découpage de la **22-2** (arbitrage de Guy, 2026-08-17, question **Q5**). Elle ne livre **aucune surface utilisateur** : rien de ce qu'elle contient n'est visible, branché, ni appelé. C'est délibéré, et c'est ce qui la rend relisable.

## Story

**As a** développeur qui implémentera la prévention des doublons,
**I want** que la logique d'appariement existe d'abord comme un module pur, testé sans DOM, sans base et sans réseau,
**so that** les défauts du mécanisme se paient en secondes de vitest, et non en passes de revue adversariale.

Sous-story de **#301**. La seconde moitié est la **22-2b**, qui la branche.

## Contexte — pourquoi ce découpage, et pourquoi ici

La 22-2 unifiée a subi **quatre passes** de `bmad-create-story validate`. Trend `9 → 11 → 20 → 6`, sévérité maximale `1C → 0C → 2C → 3C`. Le critère de non-convergence de la § *Règle de splitting préventif* s'est déclenché **deux passes de suite**.

**Le diagnostic n'est pas « la story est trop large »** — elle touche quatre modules, sous le seuil de cinq. C'est que **le mécanisme et sa surface ne se relisent pas avec la même lentille** :

| Passe | Ce qu'elle a trouvé | D'où ça venait |
|---|---|---|
| P2 | dérives de décompte, décisions sans preuve | les patches de P1 |
| P3 | **le mécanisme ne fait pas le travail** *(exécution)* | la spec d'origine |
| P4 | **3 `CRITICAL`** | le patch de remédiation de P3 |

Les trois `CRITICAL` de la passe 4 tenaient tous en trois lignes de logique pure — un ordre d'instructions inversé, un compteur mal soustrait, une preuve écrite dans le mauvais langage. **Aucun n'avait besoin d'un navigateur, d'une base ou d'un serveur pour être attrapé.** Ils ont pourtant coûté une passe adversariale complète, parce qu'ils étaient noyés dans une spec qui parlait aussi d'`aria-live`, de FTL et de Playwright.

Cette story met ce mécanisme sous un gate qui coûte **quelques secondes**.

## Périmètre — ce qui est DEDANS, et ce qui n'y est PAS

**Dedans** : un module TypeScript pur, ses fonctions, ses tests unitaires.

**Dehors, explicitement** — tout cela est la **22-2b** :

- l'appel réseau, la temporisation, les compteurs de génération, la remise à zéro à l'ouverture ;
- le balisage, `aria-live`, l'emplacement des avertissements, les deux niveaux de signal ;
- les clés i18n et les quatre locales ;
- l'extension de `push_where_clauses` à `ide_number` (backend) ;
- l'E2E et le manuel.

⚠️ **Aucun fichier `.svelte` n'est touché par cette story.** Si un patch en touche un, c'est qu'il déborde.

## Décisions

Les décisions ci-dessous sont **héritées de la 22-2** et reproduites ici en entier — cette story doit se lire seule.

**D-a1 — Où vit le module, et pourquoi le lint i18n ne s'y oppose pas.**
`frontend/src/lib/features/contacts/duplicate-probe.ts`.
⚠️ Le piège du lint est réel mais ne mord pas ici : `lint-i18n-ownership.js:126-137` n'extrait que les appels `i18nMsg(...)`, et ce module n'en contient **aucun** — c'est précisément sa raison d'être. Un fichier sans clé ne peut pas violer une règle de propriété de clés. *(C'est ce qui permet de le loger dans `features/contacts/` malgré la discordance singulier/pluriel qui met `ContactCard.svelte` et `ContactPersonsManager.svelte` en dur dans `KNOWN_VIOLATIONS`.)*

**D-a2 — Le terme se compose selon le type de contact.**

| Type | Champs du formulaire | Terme |
|---|---|---|
| `Entreprise` | `#form-name` (raison sociale) | la valeur, `trim()` |
| `Personne` | `#form-firstname` **+** `#form-lastname` | `` `${prénom} ${nom}`.trim() `` |

⚠️ Le serveur recompose `name` de la même façon (`routes/contacts.rs:362-380`, `format!("{f} {l}")`). Une fonction branchée sur la seule raison sociale serait **entièrement inopérante pour les personnes physiques**, sans rien casser ni faire échouer la compilation.
⚠️ Pendant la frappe, l'un des deux champs d'une `Personne` est **presque toujours vide** — le formulaire n'exige les deux qu'à la soumission (`validate_common:362-370`). C'est l'état **normal** de la fonction, pas un cas dégradé.

**D-a3 — Le terme est NORMALISÉ : Unicode d'abord, opérateurs ensuite.**

**Étape 1 — forme Unicode.** `s.normalize('NFC')`. ⚠️ **Sans elle, le seuil de D-a4 classe le même mot armé ou muet selon le clavier** : `.length` compte des unités UTF-16, donc `aé` vaut **2** en NFC et **3** en NFD (forme que produisent couramment macOS et certains IME). Vérifié par exécution :

```
NFC "aé" : length 2 → isArmed false
NFD "aé" : length 3 → isArmed true
```

Le dépôt a déjà tiré cette leçon côté Rust — `kesh_core::text::canonical_key` applique NFKC en étape 2, pour un motif documenté dans son propre doc-comment. Le module TypeScript ne doit pas la réapprendre.

**Étape 2 — les opérateurs.** Les dix opérateurs `BOOLEAN MODE` (`+ - > < ( ) ~ * " \`, cf. `crates/kesh-db/src/util/search.rs:41`) sont remplacés par une **espace**, puis les espaces multiples sont repliées et les bords rognés.
⚠️ **Remplacer, PAS supprimer.** Supprimer est exactement ce que fait `escape_boolean_ft` (`search.rs:124-127`, `.filter(…).collect()`), et c'est la cause du défaut : retaper `Coop-Vaud` à l'identique produit `CoopVaud*`, qui ne matche **ni** `Coop` **ni** `Vaud`, les deux tokens réellement indexés. Vérifié par exécution : `AGAINST('CoopVaud*')` → **0**, `AGAINST('Coop*')` → **1**.

**D-a4 — Le seuil se mesure APRÈS la normalisation, et sur le PLUS LONG TOKEN.**

```ts
const normalized = normalizeTerm(raw);                                        // D-a3 d'abord
const armed = Math.max(0, ...normalized.split(/\s+/).map((t) => t.length)) >= 3;  // ensuite
```

⚠️ **L'ordre est la partie fragile, et l'inverser casse dans les DEUX sens** — vérifié par exécution en passe 4 de la 22-2 :

| Terme tapé | Brut | Normalisé | Seuil mesuré sur le BRUT | Conséquence |
|---|---|---|---|---|
| `Yo-An` (contact `Yo-An SA` en base) | 1 token de 5 | `Yo An` → deux tokens de 2 | **passe** | `AGAINST('Yo An*')` → **0 résultat**. Silence garanti sur un doublon exact |
| `C++` | 1 token de 3 | `C` → un token de 1 | **passe** | `AGAINST('C*')` → **tout contact commençant par C**. Bruit maximal |

`split(/\s+/)` ne connaît que l'espace comme frontière de token — or la normalisation en **crée dix autres**. Mesurer avant, c'est mesurer des tokens qui n'existeront plus.

⚠️ **Le seuil de trois est une POLITIQUE D'ERGONOMIE, pas une contrainte du moteur.** La première rédaction de la 22-2 affirmait le contraire et se trompait : `innodb_ft_min_token_size = 3` gouverne les tokens **exacts**, pas les préfixes, et `push_where_clauses:202` appose toujours `*` — `AGAINST('Du*')` rend **1**, `AGAINST('Du')` rend **0**. Le seuil limite le bruit et le nombre de requêtes ; c'est un bon choix, mais un choix.

**D-a5 — Le classement, écrit en toutes lettres.**
Quatre critères, **dans cet ordre**, sur des chaînes repliées en casse et sans accents (cohérent avec `utf8mb4_general_ci`, la collation réelle de la table, vérifiée) :

1. le nom **commence par** le terme complet ;
2. puis le **nombre de tokens DISTINCTS du terme** présents dans le nom, décroissant — un terme qui répète un token (`jean jean`) ne compte ce token **qu'une fois**, sans quoi le classement récompenserait une faute de frappe ;
3. puis la **longueur du plus long préfixe commun**, décroissante ;
4. puis l'**ordre alphabétique** ;
5. puis l'**`id`**, croissant.

**Le repli, écrit — « sans accents » n'est pas un algorithme.** `s.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLowerCase()`.
⚠️ **Ses limites sont connues et assumées, et elles ne mordent PAS sur le carnet suisse ordinaire** : `ü`, `ä`, `ö`, `é`, `è` se replient tous correctement (`Müller` → `muller`, `Zürich` → `zurich`). Ce qui ne se replie pas, ce sont les **ligatures** — `œ` (français) et `ß`, vérifié par exécution (`fold("Straße")` → `straße`). ⚠️ *La rédaction précédente citait `ß` comme un cas « suisse ordinaire » : c'est faux, l'allemand de Suisse ne l'emploie pas et écrit `ss`.* L'algorithme est écrit plutôt que décrit parce que deux implémentations « conformes à la lettre » de « sans accents » divergeraient sur ces ligatures.
⚠️ **Ce repli fait l'INVERSE de `canonical_key`**, et ce n'est pas une incohérence : `canonical_key` sert l'**unicité** d'un identifiant, où un accent doit distinguer (`text.rs:117-124` le teste). `rank` sert la **ressemblance**, où un accent doit rapprocher. Deux besoins opposés, deux fonctions.

⚠️ **Les critères 4 et 5 ne sont pas des critères de pertinence : ce sont ceux qui rendent le classement DÉTERMINISTE**, donc testable.
**Le critère 5 a été ajouté après vérification par exécution** : le critère 4 ne suffit pas. Deux contacts **strictement homonymes** — le cas « un père et son fils », que la 22-2b nomme explicitement comme légitime — sont ex æquo sur les quatre premiers critères, et un tri stable rend alors l'ordre d'**entrée** :

```
entrée [id 101, id 202] → [101, 202]
entrée [id 202, id 101] → [202, 101]      ⚠️ sorties DIFFÉRENTES
```

Sans le critère 5, la preuve **4** d'AC-a4 est **littéralement insatisfiable**.

**Pourquoi ce classement existe** — et c'est le défaut le plus grave qu'ait trouvé la revue de la 22-2, établi par exécution :

```sql
-- carnet : Jean Bernard, Jean Dupont, Jean Favre, Jean Martin, Jean Rochat, Jean Zwahlen
SELECT id,name FROM contacts WHERE company_id=1 AND active=TRUE
 AND MATCH(name) AGAINST('Jean Zwahlen*' IN BOOLEAN MODE) ORDER BY name ASC LIMIT 5;
→ Jean Bernard · Jean Dupont · Jean Favre · Jean Martin · Jean Rochat
→ total réel = 6            ⚠️ « Jean Zwahlen » EST ABSENT
```

`escape_boolean_ft` ne préfixe **aucun** token de `+` — la sémantique est **OU inclusif** (`search.rs:85` le dit en toutes lettres ; le bloc `88-92` porte un AUTRE avertissement, sur le joker collé au seul dernier token) — le tri est **alphabétique** et il n'existe **aucun** tri par pertinence à demander (`ContactSortBy` n'offre que `Name | CreatedAt | UpdatedAt`). Le doublon exact était donc **évincé de la fenêtre** : le dispositif était muet sur le seul cas pour lequel il existe, et bavard sur cinq contacts sans rapport.

**D-a6 — « et N autres » soustrait le contact édité, et sa PRÉCONDITION est écrite.**

⚠️ **`countOthers` déduit la présence de soi en cherchant `editingId` dans `items` — donc dans la FENÊTRE, pas dans `total`.** Si le contact édité correspond au terme mais tombe **hors** de la fenêtre (`total = 25`, `limit: 20`, son nom triant en 23ᵉ position alphabétique), il n'est pas dans `items` : `soi` vaut 0, et le compteur sur-compte d'une unité — il compte la fiche qu'on modifie comme « un autre ». Trouvé par **convergence de deux lentilles** en passe 1.

**Précondition, à respecter par l'appelant** : `countOthers` suppose que si le contact édité figure dans `total`, il figure aussi dans `items`. La 22-2b la garantit tant que `total ≤ limit` ; **au-delà, l'écart d'une unité est assumé et documenté** — il n'affecte qu'un compteur indicatif, jamais une proposition affichée ni une décision. La corriger exigerait d'exclure `editingId` côté **serveur**, ce qui ouvrirait un paramètre de requête pour un gain cosmétique.

⚠️ `total` est le `COUNT(*)` **serveur** (`repositories/contacts.rs:388-419`), qui ne connaît pas la notion de « soi » : l'exclusion du contact en cours d'édition n'a lieu qu'au **client**. Sans la soustraction, corriger un caractère du nom d'une fiche existante affiche **« et 1 autre » au-dessus d'une liste vide**, en désignant la fiche qu'on modifie.

**D-a7 — L'IDE se vérifie sur le CHAMP, contre la valeur ENVOYÉE.**
Un contact peut remonter parce que la chaîne figure dans son nom ou son email, **sans porter cet IDE**. Le porteur n'est retenu que si `c.ideNumber === normalized && c.id !== editingId`.
⚠️ `normalized` est la valeur **passée en argument**, jamais une relecture d'un champ de formulaire : `normalizeIdeForApi('')` rend **`null`**, et `c.ideNumber === null` désignerait **tout contact sans IDE**. Le module étant pur, cette garde est structurelle — il n'a accès à aucun champ.

## Acceptance Criteria

**AC-a1 — `normalizeTerm` remplace les opérateurs par une espace.**
`Coop-Vaud` → `Coop Vaud` · `C++` → `C` · `Müller-Weber` → `Müller Weber` · `  a--b  ` → `a b` · `***` → `` (chaîne vide).
*Preuve*, **2** tests :
1. un test paramétré couvrant les cinq cas ci-dessus. Mutation : « supprimer au lieu de remplacer », qui rend `CoopVaud` ;
2. **la forme Unicode** : le même mot saisi en **NFD** (`e` + accent combinant) et en **NFC** rend la **même** chaîne. Mutation : « omettre `normalize('NFC')` », que le test 1 laisse verte — aucun de ses cinq cas ne porte d'accent décomposé.

**AC-a2 — `buildTerm` compose selon le type (D-a2).**
⚠️ Les deux paramètres de nom sont typés **`string`**, jamais `string | undefined` : `` `${undefined} ${l}` `` injecterait le mot `"undefined"` dans le terme de recherche. Le typage suffit à éliminer le cas **à la compilation**, ce qui vaut mieux qu'un test.
`Entreprise` → la raison sociale rognée. `Personne` → `« prénom nom »` rognée, **y compris quand l'un des deux est vide** (`("Jean","")` → `"Jean"`, `("","Dupont")` → `"Dupont"`, `("","")` → `""`).
*Preuve*, **2** tests : un par type, celui sur `Personne` couvrant les trois cas de vacuité. Mutation : « ne lire que la raison sociale », qui rend le dispositif entièrement mort pour les personnes physiques.

**AC-a3 — `isArmed` mesure le plus long token d'une chaîne DÉJÀ NORMALISÉE.**

⚠️ **Le contrat, sans ambiguïté possible : `isArmed` NE normalise PAS.** Elle reçoit la sortie de `normalizeTerm` et se contente de mesurer. C'est ainsi que la 22-2b l'appelle (`isArmed(normalized)`), et c'est ce que son nom de paramètre doit dire.

| Terme tapé | `normalizeTerm` rend | `isArmed` |
|---|---|---|
| `Jean` | `Jean` | **armé** |
| `Dubarde Sàrl` | `Dubarde Sàrl` | **armé** |
| `Du` | `Du` | muet |
| `An Li` | `An Li` | muet |
| `Yo-An` | `Yo An` | muet |
| `C++` | `C` | muet |
| `""` | `""` | muet |

*Preuve*, **1** test paramétré sur les sept cas, **écrits `isArmed(normalizeTerm(x))`** — jamais `isArmed(x)` sur la valeur brute.
⚠️ **Cette précision n'est pas rédactionnelle.** Écrits en brut, `isArmed("Yo-An")` et `isArmed("C++")` rendent **`true`** (5 et 3 caractères d'un seul tenant) — l'inverse de ce que la table attend. Un développeur qui suivrait la lettre en conclurait qu'`isArmed` doit normaliser en interne, ce qui contredirait le site d'appel de la 22-2b. Relevé en passe 1.
⚠️ **Mutation nommée : « mesurer le seuil avant de normaliser »** — un simple ordre d'instructions dans l'appelant, qui laisse `Du`, `An Li` et `""` corrects et ne fait tomber que `Yo-An` et `C++`. C'est pour eux que ce test existe.

**AC-a4 — `rank` classe selon les cinq critères, et il est DÉTERMINISTE (D-a5).**
*Preuve*, **4** tests :
1. **la fixture des six `Jean X`** — terme `Jean Zwahlen`, `Jean Zwahlen` sort **en tête** et figure dans les cinq premiers. ⚠️ **C'est LA preuve du mécanisme.** Mutation : **« `rank` = identité »**, qui rend l'ordre alphabétique du SQL et évince Zwahlen — le défaut exact que cette story existe pour fermer ;
2. **le critère 1 prime le critère 2**, avec **cette fixture-ci** : terme `Jean` ; `A = "Jeanne Dupont"` (commence par `Jean` mais ne porte **aucun** token `jean` — `jeanne` ≠ `jean`) ; `B = "Marie Jean"` (ne commence pas par le terme mais porte le token exact, donc **strictement plus** de tokens que A). `A` doit sortir devant `B`. Mutation : « intervertir les critères 1 et 2 ».
   ⚠️ **La fixture naïve ne discrimine RIEN, et c'est pourquoi elle est écrite ici.** Prendre `"Acme Corp"` contre `"Corp Acme Trading"` sur le terme `Acme` laisse la mutation **verte** : un nom qui commence par le terme le contient trivialement, donc sature aussi le critère 2. Il faut la frontière de mot — la même subtilité que D-a4 exploite pour `Yo-An`. Relevé en passe 1 ;
3. **le déterminisme sur des noms distincts** : deux appels sur la même entrée dans deux ordres différents rendent **la même** sortie. Mutation : « retirer le critère 4 » ;
4. **le déterminisme sur deux HOMONYMES STRICTS** — deux contacts au nom identique, `id` 101 et 202. Mutation : « retirer le critère 5 », que le test 3 laisse **verte** puisque ses noms diffèrent. ⚠️ C'est le test qui a manqué : sans critère 5, la preuve 3 est littéralement insatisfiable sur ce cas.

**AC-a5 — `excludeSelf` et `countOthers` s'accordent (D-a6).**
`excludeSelf(items, editingId)` retire le contact édité. `countOthers(total, items, retenus, editingId)` rend `max(0, total − soiPrésent − retenus.length)`.
*Preuve*, **3** tests :
1. le cas nominal — `total = 12`, 5 retenus, pas d'édition ⇒ **7** ;
2. ⚠️ **le cas de l'édition solitaire** — `total = 1`, l'unique élément est le contact édité, donc 0 retenu ⇒ **0**, et non 1. Mutation : « `total − retenus.length` », qui rend **1** et fait afficher « et 1 autre » au-dessus d'une liste vide, en désignant la fiche qu'on modifie. Le premier test reste **vert** sous cette mutation ;
3. **le contact édité HORS FENÊTRE** — `total = 25`, `items` en contient 20 **sans** le contact édité, 5 retenus ⇒ le résultat vaut **20**, et l'écart d'une unité avec le compte réel (19) est **assumé**, cf. la précondition de D-a6. ⚠️ Ce test ne prouve pas une correction : il **fige un comportement connu** pour qu'un développeur futur ne le prenne pas pour un bug à réparer au prix d'un paramètre de requête.

**AC-a6 — `findIdeHolder` vérifie sur le champ et exclut soi (D-a7).**
Rend le contact dont `ideNumber` **égale** la valeur passée et dont l'`id` diffère de `editingId` ; `undefined` sinon.
*Preuve*, **3** tests :
1. un porteur réel est trouvé ;
2. un contact qui remonte **sans porter cet IDE** n'est pas retenu ;
3. ⚠️ **`null` ne désigne personne** — appelé avec `null`, la fonction rend `undefined` même si le lot contient des contacts dont `ideNumber` est `null`. Mutation : comparer sans garder de garde sur la vacuité, qui fait de `null === null` un appariement.

**AC-a7 — Le module est PUR — et « pur » veut dire exactement ceci.**

⚠️ **La formulation catch-all « ni quoi que ce soit qui touche au DOM, au réseau ou à l'horloge » a été RETIRÉE** : elle rendait l'AC indécidable, aucun outillage annoncé ne pouvant trancher pour un import non énuméré. Deux clauses **vérifiables** la remplacent :

- **(a)** le fichier ne contient **aucun `import`** de `$app/*`, `$lib/shared/utils/api-client`, `$lib/features/contacts/contacts.api`, ni d'aucun module de `$lib/components/` ;
- **(b)** le fichier ne contient **aucun appel** à `fetch(`, `setTimeout(`, `setInterval(`, `Date.now(`, `document.`, `window.`.

⚠️ **(b) est indispensable et (a) ne la couvre pas** : en JavaScript, ces globales s'appellent **sans aucune ligne `import`**. Une preuve qui ne regarderait que les imports serait verte sur un `Date.now()` nu.

*Preuve*, **2** tests : un par clause, lisant le source du module et assertant l'absence des motifs. Cette AC est ce qui garantit que le gate de la story reste à quelques secondes ; sans elle, la première dépendance introduite la ramène silencieusement au coût d'un test de composant.

## Tasks / Subtasks

- [ ] **T-a1 — Créer `frontend/src/lib/features/contacts/duplicate-probe.ts`** (toutes les AC). Fonctions exportées : `normalizeTerm`, `buildTerm`, `isArmed`, `rank`, `excludeSelf`, `countOthers`, `findIdeHolder`.
  - [ ] **L'ordre `normaliser → mesurer → décider`** (D-a3 puis D-a4) est la partie fragile — l'écrire dans le doc-comment du module, pas seulement dans le code.
  - [ ] Aucune clé i18n dans ce fichier (D-a1). Aucun import réseau ni DOM (AC-a7).
  - [ ] **Documenter la DOUBLE normalisation dans le doc-comment du module**, faute de quoi elle passera pour du code mort : `normalizeTerm` applique **NFC** (pour stabiliser le `.length` d'`isArmed`, D-a3), puis `rank` applique **NFD + strip** pour le repli d'accents (D-a5). Les deux sont nécessaires et ne font pas doublon — l'une stabilise une **mesure**, l'autre produit une **comparaison**. Un terme NFC re-normalisé en NFD puis strippé donne le même résultat qu'un strip direct : la composition est inoffensive.
  - [ ] **Noter aussi le cas turc**, pour qu'il ne soit pas « corrigé » plus tard : `İ` (U+0130) se minuscule en `i` + point combinant, que le strip d'accents de `rank` retire ensuite — le résultat final est `i`, ce qui est correct. **Aucune action requise**, mais un lecteur qui découvre ce chemin sans explication le prendra pour un défaut.
- [ ] **T-a2 — Créer `duplicate-probe.test.ts`** (toutes les AC). Vitest direct, **sans `render`** — c'est tout le bénéfice de la story.
  - [ ] **Chaque test nomme sa mutation en commentaire**, comme le fait `products-page.test.ts`. Une preuve dont la mutation n'est pas écrite se relit mal et se supprime facilement.
  - [ ] La fixture des six `Jean X` d'AC-a4 est la plus importante du lot : la reprendre **telle quelle**, elle est l'énoncé du défaut d'origine.
- [ ] **T-a3 — Gate.** `npm run check` · `npm run lint-i18n-ownership` · `npm run test:unit` · `npm run build`, depuis `frontend/`.
  - [ ] ⚠️ **Aucun gate backend n'est requis** : cette story ne touche **aucun** fichier Rust, aucune migration, aucun `.svelte`. Le dire dans le Dev Agent Record plutôt que de déclarer un gate qui n'a pas tourné.

## Décompte des preuves — la seule table qui fait foi

| AC | Preuves | Nature |
|---|---:|---|
| AC-a1 — normalisation | 2 | vitest pur |
| AC-a2 — composition du terme | 2 | vitest pur |
| AC-a3 — seuil | 1 | vitest pur |
| AC-a4 — classement | 4 | vitest pur |
| AC-a5 — exclusion et compteur | 3 | vitest pur |
| AC-a6 — porteur d'IDE | 3 | vitest pur |
| AC-a7 — pureté du module | 2 | vitest pur |

**Total, sommé depuis la colonne : 17 preuves, toutes unitaires, aucune n'exigeant DOM, base ni réseau.**

Aucun autre passage de cette story n'énonce de total.

## Dev Notes

### Ce que cette story ne doit PAS faire

Elle ne branche rien. Le module qu'elle livre n'est **appelé par personne** à la fin de la story, et c'est normal — `22-2b` s'en charge. Un dev qui le câble « pour vérifier que ça marche » a quitté le périmètre : sa vérification, c'est le vitest.

### Les mutations, rassemblées

Elles sont la vraie spécification de cette story. Chacune est un défaut **réellement survenu** pendant les quatre passes de revue de la 22-2 unifiée :

| Mutation | Ce qu'elle casse | Preuve qui tombe |
|---|---|---|
| supprimer au lieu de remplacer | `Coop-Vaud` ne se retrouve plus lui-même | AC-a1 |
| ne lire que la raison sociale | mort pour toutes les personnes physiques | AC-a2 |
| **mesurer le seuil avant de normaliser** | silence sur `Yo-An`, bruit sur `C++` | AC-a3 |
| **`rank` = identité** | le doublon exact est évincé de la fenêtre | AC-a4 |
| `total − retenus.length` | « et 1 autre » au-dessus d'une liste vide | AC-a5 |
| `null === null` sur l'IDE | le signal franc crie sur un champ vide | AC-a6 |
| omettre `normalize('NFC')` | le même mot armé ou muet selon le clavier | AC-a1 test 2 |
| intervertir les critères 1 et 2 | *(et la fixture naïve ne l'attrape pas)* | AC-a4 test 2 |
| retirer le critère 4 | deux noms distincts se classent selon l'ordre d'entrée | AC-a4 test 3 |
| retirer le critère 5 | deux homonymes se classent selon l'ordre d'entrée | AC-a4 test 4 |
| introduire un import réseau | le gate quitte le régime de la seconde | AC-a7 clause (a) |
| appeler `Date.now()` sans import | échappe à toute preuve qui ne lit que les imports | AC-a7 clause (b) |

### Conventions

`@testing-library/svelte` v5 + Svelte 5 sont disponibles, mais **cette story n'en a pas besoin** : tous ses tests sont des appels de fonction. Patron de rédaction des doc-comments de test : `frontend/src/routes/(app)/products/products-page.test.ts`, dont l'en-tête explique pourquoi une assertion sur l'argument d'un mock attrape des mutations qu'un test fonctionnel laisse passer.

### References

- Story **22-2** (umbrella, statut `split`) — les quatre passes de revue et leur Change Log complet. **C'est la source de toutes les décisions reproduites ici.**
- Story **22-2b** — la surface, qui consomme ce module.
- Issue **#301** — le besoin.
- ⚠️ **Issues #314 et #315** — les défauts **à la source** que ce module contourne. `normalizeTerm` (D-a3) atténue #314 ; `rank` (D-a5) atténue #315. **Aucune des deux fonctions ne corrige quoi que ce soit dans la recherche elle-même** — elles compensent au client ce que la base ne sait pas faire, et le doc-comment du module doit le dire, faute de quoi un lecteur futur croira le problème réglé.
- `CLAUDE.md` — § *Règle de splitting préventif* (le critère qui a déclenché ce découpage), § *Test Locally First*.

## Change Log

### Passe 1 de `bmad-create-story validate` — 2026-08-17, Sonnet ×3, contexte frais

Bruts : 3 + 6 + 4 = 13. Deux convergences. **Retenus : 0 CRITICAL / 3 HIGH / 4 MEDIUM / 4 LOW — 11 findings, 11 correctifs appliqués.**

⚠️ **Zéro `CRITICAL` — c'est la première fois depuis le début de ce dossier**, et c'est le premier signe que le découpage travaille : les défauts du mécanisme se trouvent désormais dans un module que trois lentilles ont pu **exécuter en Node** plutôt que relire.

**Reclassement assumé.** L'EdgeCaseHunter a rendu **4 `CRITICAL`** ; j'en retiens **2 `HIGH` et 2 `MEDIUM`**, et je le dis plutôt que de le taire — je suis l'auteur de la spec, donc la tentation de minimiser existe. Le barème donné aux lentilles définit `CRITICAL` comme « la spec, suivie à la lettre, produit un logiciel faux ». Un seuil qui s'arme différemment selon la forme Unicode du clavier, ou un repli d'accents non spécifié, dégradent le classement sans produire d'avertissement faux ni de donnée fausse : c'est `MEDIUM`. En revanche, un classement **non déterministe** rend sa propre preuve *littéralement insatisfiable*, et un contrat de fonction contradictoire est **infaisable tel quel** : ce sont bien des `HIGH`.

**Les trois `HIGH`, tous vérifiés par exécution :**

1. **`rank` n'était pas déterministe sur deux homonymes stricts.** Le critère 4 (alphabétique) ne départage pas deux noms identiques — le cas « un père et son fils », que la 22-2b nomme comme légitime. Un tri stable rend alors l'ordre d'**entrée** : `[101,202]` puis `[202,101]` sortent différemment. La preuve de déterminisme exigeait exactement le contraire. *(Elle portait alors le n° 3 ; le correctif a inséré une preuve 4 dédiée aux homonymes, et c'est elle qui porte désormais cette exigence — les renvois par NUMÉRO se périment à chaque insertion, ce que la passe 2 a dû rattraper deux fois.)* **Un cinquième critère (`id`) est ajouté**, avec sa preuve dédiée.
2. **Le contrat d'`isArmed` était contradictoire.** AC-a3 listait `isArmed("Yo-An")` → muet, alors que la 22-2b l'appelle sur une chaîne **déjà normalisée** ; en brut, `"Yo-An"` rend `true`. Un développeur suivant la lettre en aurait conclu qu'`isArmed` doit normaliser en interne — contredisant le site d'appel. **Le contrat est écrit, et les sept cas s'écrivent désormais `isArmed(normalizeTerm(x))`.**
3. **La preuve 2 d'AC-a4 n'avait pas de fixture, et la fixture naturelle ne discrimine rien.** « Le critère 1 prime le critère 2 » : avec `"Acme Corp"` contre `"Corp Acme Trading"`, la mutation « intervertir les critères » reste **verte** — un nom qui commence par le terme le contient trivialement, donc sature aussi le critère 2. **La fixture qui discrimine est désormais écrite** (`Jean` / `"Jeanne Dupont"` / `"Marie Jean"`), et elle exploite la même frontière de mot que D-a4.

**Les quatre `MEDIUM`** : aucune normalisation Unicode, donc un seuil qui varie selon NFC/NFD (`aé` vaut 2 ou 3) — **`normalize('NFC')` devient l'étape 1 de D-a3** ; le repli « sans accents » de D-a5 n'était **pas un algorithme** — il est écrit, avec ses limites (`ß`, `œ`) et la raison pour laquelle il fait l'**inverse** de `canonical_key` ; `countOthers` sur-compte d'une unité quand le contact édité tombe hors de la fenêtre — **convergence de deux lentilles**, traitée par une **précondition écrite** et un test qui fige le comportement plutôt que par un paramètre de requête ; et `AC-a7` promettait une pureté sémantique que sa preuve ne pouvait pas tenir — la clause catch-all est **retirée** au profit de deux clauses vérifiables, dont une sur les **globales appelées sans import** (`Date.now()`, `fetch(`), qu'aucune preuve sur les imports ne voit.

**Décompte : 13 → 17 preuves**, recompté depuis les AC. Table des mutations : 7 → 12.

⚠️ **Note de méthode, valable pour la suite.** **Deux des six lentilles de cette passe ont rendu un décompte faux** — l'une annonçait un `CRITICAL` que son propre corps ne décrivait pas, l'autre s'est corrigée en note. Interrogée, la première a répondu franchement : *« mon décompte final était faux, je me suis trompé en additionnant »*. C'est le défaut du § *Recompter ses propres comptes rendus*, et il ne touche pas que l'auteur : **il touche aussi les relecteurs**. Un décompte de rapport de revue se recompte comme le reste.

⚠️ **Passe 2 due** par la § *Review Iteration Rule* (3 `HIGH`). Rotation : **Haiku**, contexte frais.

### Passe 2 de `bmad-create-story validate` — 2026-08-17, Haiku ×3, contexte frais

Bruts : 2 + 3 + 1 = 6, plus **1 trouvé par l'orchestrateur** en auditant les renvois. Un non-finding écarté. **Retenus : 0 CRITICAL / 0 HIGH / 3 MEDIUM / 3 LOW — 6 findings, 6 correctifs.**

**Trend : `0C/3H/4M/4L` → `0C/0H/3M/3L`.** La sévérité maximale **décroît** (`HIGH → MEDIUM`) et le nombre baisse. Le `BlindHunter` a rendu un rapport **vide de tout défaut substantiel**, avec un recompte indépendant des deux totaux (17 et 31) — confirmés justes.

**Le thème de cette passe est la DÉRIVE DE RENUMÉROTATION**, et elle est entièrement de mon fait. La passe 1 a **inséré** des preuves au milieu de listes numérotées ; tous les renvois « preuve N d'AC-x » situés **après** l'insertion se sont périmés en silence. Ici, D-a5 affirmait que « sans le critère 5, la preuve **3** est insatisfiable » — or l'insertion avait déplacé le cas des homonymes en **4ᵉ** position. Le renvoi pointait la preuve voisine, celle qui n'a pas besoin du critère 5.

⚠️ **Aucune lentille ne l'a vu ; c'est un audit systématique de l'orchestrateur qui l'a trouvé** — et il en a trouvé deux autres dans la 22-2b. Un renvoi par **numéro** dans une liste **insérable** est un piège muet de plus, et il ne se voit qu'en confrontant chaque référence au contenu réel de sa cible.

Les deux autres `MEDIUM` : le critère 2 du classement ne disait pas s'il compte les tokens **distincts** ou **avec multiplicité** — c'est **distinct**, sans quoi le classement récompenserait une faute de frappe (`jean jean`) ; et la **table des mutations listait 11 lignes** quand le Change Log de la passe 1 en annonçait 12 — la mutation « retirer le critère 4 » n'y avait jamais été portée.

Les trois `LOW` sont de la précision : l'exemple `ß` était **mal choisi pour un carnet suisse** — l'allemand de Suisse ne l'emploie pas et écrit `ss`, et les caractères réellement en jeu (`ü`, `ä`, `ö`) se replient correctement ; la **double normalisation** NFC puis NFD demande un doc-comment, faute de quoi elle passera pour du code mort ; et le cas turc `İ` est **correct par construction** mais mérite d'être noté pour ne pas être « corrigé » plus tard.

⚠️ **Passe 3 due** (3 `MEDIUM`). Rotation : **Opus**, contexte frais.

## Dev Agent Record

### Agent Model Used

*(à remplir)*

### Debug Log References

### Completion Notes List

### File List
