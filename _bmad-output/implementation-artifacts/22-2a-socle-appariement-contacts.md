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

**D-a3 — Le terme est NORMALISÉ, et la normalisation REMPLACE.**
Les dix opérateurs `BOOLEAN MODE` (`+ - > < ( ) ~ * " \`, cf. `crates/kesh-db/src/util/search.rs:41`) sont remplacés par une **espace**, puis les espaces multiples sont repliées et les bords rognés.
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

⚠️ **Le seuil de trois est une POLITIQUE D'ERGONOMIE, pas une contrainte du moteur.** La première rédaction de la 22-2 affirmait le contraire et se trompait : `innodb_ft_min_token_size = 3` gouverne les tokens **exacts**, pas les préfixes, et `push_where_clauses:205` appose toujours `*` — `AGAINST('Du*')` rend **1**, `AGAINST('Du')` rend **0**. Le seuil limite le bruit et le nombre de requêtes ; c'est un bon choix, mais un choix.

**D-a5 — Le classement, écrit en toutes lettres.**
Quatre critères, **dans cet ordre**, sur des chaînes repliées en casse et sans accents (cohérent avec `utf8mb4_general_ci`, la collation réelle de la table, vérifiée) :

1. le nom **commence par** le terme complet ;
2. puis le **nombre de tokens du terme présents** dans le nom, décroissant ;
3. puis la **longueur du plus long préfixe commun**, décroissante ;
4. puis l'**ordre alphabétique**.

⚠️ **Le critère 4 n'est pas un critère de pertinence : c'est ce qui rend le classement DÉTERMINISTE**, donc testable. Sans lui, deux implémentations conformes à la lettre rendent des sous-ensembles différents et aucun relecteur ne peut trancher « conforme / non conforme ».

**Pourquoi ce classement existe** — et c'est le défaut le plus grave qu'ait trouvé la revue de la 22-2, établi par exécution :

```sql
-- carnet : Jean Bernard, Jean Dupont, Jean Favre, Jean Martin, Jean Rochat, Jean Zwahlen
SELECT id,name FROM contacts WHERE company_id=1 AND active=TRUE
 AND MATCH(name) AGAINST('Jean Zwahlen*' IN BOOLEAN MODE) ORDER BY name ASC LIMIT 5;
→ Jean Bernard · Jean Dupont · Jean Favre · Jean Martin · Jean Rochat
→ total réel = 6            ⚠️ « Jean Zwahlen » EST ABSENT
```

`escape_boolean_ft` ne préfixe **aucun** token de `+` — la sémantique est **OU inclusif** (`search.rs:88-92` le dit) — le tri est **alphabétique** et il n'existe **aucun** tri par pertinence à demander (`ContactSortBy` n'offre que `Name | CreatedAt | UpdatedAt`). Le doublon exact était donc **évincé de la fenêtre** : le dispositif était muet sur le seul cas pour lequel il existe, et bavard sur cinq contacts sans rapport.

**D-a6 — « et N autres » soustrait le contact édité.**
⚠️ `total` est le `COUNT(*)` **serveur** (`repositories/contacts.rs:388-419`), qui ne connaît pas la notion de « soi » : l'exclusion du contact en cours d'édition n'a lieu qu'au **client**. Sans la soustraction, corriger un caractère du nom d'une fiche existante affiche **« et 1 autre » au-dessus d'une liste vide**, en désignant la fiche qu'on modifie.

**D-a7 — L'IDE se vérifie sur le CHAMP, contre la valeur ENVOYÉE.**
Un contact peut remonter parce que la chaîne figure dans son nom ou son email, **sans porter cet IDE**. Le porteur n'est retenu que si `c.ideNumber === normalized && c.id !== editingId`.
⚠️ `normalized` est la valeur **passée en argument**, jamais une relecture d'un champ de formulaire : `normalizeIdeForApi('')` rend **`null`**, et `c.ideNumber === null` désignerait **tout contact sans IDE**. Le module étant pur, cette garde est structurelle — il n'a accès à aucun champ.

## Acceptance Criteria

**AC-a1 — `normalizeTerm` remplace les opérateurs par une espace.**
`Coop-Vaud` → `Coop Vaud` · `C++` → `C` · `Müller-Weber` → `Müller Weber` · `  a--b  ` → `a b` · `***` → `` (chaîne vide).
*Preuve*, **1** test paramétré couvrant les cinq cas. Mutation : « supprimer au lieu de remplacer », qui rend `CoopVaud`.

**AC-a2 — `buildTerm` compose selon le type.**
`Entreprise` → la raison sociale rognée. `Personne` → `« prénom nom »` rognée, **y compris quand l'un des deux est vide** (`("Jean","")` → `"Jean"`, `("","Dupont")` → `"Dupont"`, `("","")` → `""`).
*Preuve*, **2** tests : un par type, celui sur `Personne` couvrant les trois cas de vacuité. Mutation : « ne lire que la raison sociale », qui rend le dispositif entièrement mort pour les personnes physiques.

**AC-a3 — `isArmed` mesure le plus long token du terme NORMALISÉ.**
Armé : `Jean`, `Dubarde Sàrl`. Muet : `Du`, `An Li`, **`Yo-An`**, **`C++`**, `""`.
*Preuve*, **1** test paramétré couvrant les sept cas. ⚠️ **Mutation nommée : « mesurer le seuil avant de normaliser »** — un simple ordre d'instructions, qui laisse `Du`, `An Li` et `""` corrects et ne fait tomber que `Yo-An` et `C++`. C'est pour eux que ce test existe.

**AC-a4 — `rank` classe selon les quatre critères, et il est DÉTERMINISTE.**
*Preuve*, **3** tests :
1. **la fixture des six `Jean X`** — terme `Jean Zwahlen`, `Jean Zwahlen` sort **en tête** et figure dans les cinq premiers. ⚠️ **C'est LA preuve du mécanisme.** Mutation : **« `rank` = identité »**, qui rend l'ordre alphabétique du SQL et évince Zwahlen — le défaut exact que cette story existe pour fermer ;
2. **le critère 1 prime le critère 2** : un nom qui commence par le terme passe devant un nom qui partage davantage de tokens sans commencer par lui ;
3. **le déterminisme** : deux appels sur la même entrée dans deux ordres d'entrée différents rendent **la même** sortie. Mutation : « retirer le critère 4 », que les deux autres tests laissent verts.

**AC-a5 — `excludeSelf` et `countOthers` s'accordent.**
`excludeSelf(items, editingId)` retire le contact édité. `countOthers(total, items, retenus, editingId)` rend `max(0, total − soiPrésent − retenus.length)`.
*Preuve*, **2** tests :
1. le cas nominal — `total = 12`, 5 retenus, pas d'édition ⇒ **7** ;
2. ⚠️ **le cas de l'édition solitaire** — `total = 1`, l'unique élément est le contact édité, donc 0 retenu ⇒ **0**, et non 1. Mutation : « `total − retenus.length` », qui rend **1** et fait afficher « et 1 autre » au-dessus d'une liste vide, en désignant la fiche qu'on modifie. Le premier test reste **vert** sous cette mutation.

**AC-a6 — `findIdeHolder` vérifie sur le champ et exclut soi.**
Rend le contact dont `ideNumber` **égale** la valeur passée et dont l'`id` diffère de `editingId` ; `undefined` sinon.
*Preuve*, **3** tests :
1. un porteur réel est trouvé ;
2. un contact qui remonte **sans porter cet IDE** n'est pas retenu ;
3. ⚠️ **`null` ne désigne personne** — appelé avec `null`, la fonction rend `undefined` même si le lot contient des contacts dont `ideNumber` est `null`. Mutation : comparer sans garder de garde sur la vacuité, qui fait de `null === null` un appariement.

**AC-a7 — Le module est PUR.**
Aucun `import` de `$app/*`, de `$lib/shared/utils/api-client`, de `$lib/features/contacts/contacts.api`, ni de quoi que ce soit qui touche au DOM, au réseau ou à l'horloge.
*Preuve*, **1** test : `grep`-équivalent en assertion — le fichier ne contient aucun de ces imports. ⚠️ Cette AC est ce qui garantit que le gate de la story reste à quelques secondes ; sans elle, la première dépendance introduite la ramène silencieusement au coût d'un test de composant.

## Tasks / Subtasks

- [ ] **T-a1 — Créer `frontend/src/lib/features/contacts/duplicate-probe.ts`** (toutes les AC). Fonctions exportées : `normalizeTerm`, `buildTerm`, `isArmed`, `rank`, `excludeSelf`, `countOthers`, `findIdeHolder`.
  - [ ] **L'ordre `normaliser → mesurer → décider`** (D-a3 puis D-a4) est la partie fragile — l'écrire dans le doc-comment du module, pas seulement dans le code.
  - [ ] Aucune clé i18n dans ce fichier (D-a1). Aucun import réseau ni DOM (AC-a7).
- [ ] **T-a2 — Créer `duplicate-probe.test.ts`** (toutes les AC). Vitest direct, **sans `render`** — c'est tout le bénéfice de la story.
  - [ ] **Chaque test nomme sa mutation en commentaire**, comme le fait `products-page.test.ts`. Une preuve dont la mutation n'est pas écrite se relit mal et se supprime facilement.
  - [ ] La fixture des six `Jean X` d'AC-a4 est la plus importante du lot : la reprendre **telle quelle**, elle est l'énoncé du défaut d'origine.
- [ ] **T-a3 — Gate.** `npm run check` · `npm run lint-i18n-ownership` · `npm run test:unit` · `npm run build`, depuis `frontend/`.
  - [ ] ⚠️ **Aucun gate backend n'est requis** : cette story ne touche **aucun** fichier Rust, aucune migration, aucun `.svelte`. Le dire dans le Dev Agent Record plutôt que de déclarer un gate qui n'a pas tourné.

## Décompte des preuves — la seule table qui fait foi

| AC | Preuves | Nature |
|---|---:|---|
| AC-a1 — normalisation | 1 | vitest pur |
| AC-a2 — composition du terme | 2 | vitest pur |
| AC-a3 — seuil | 1 | vitest pur |
| AC-a4 — classement | 3 | vitest pur |
| AC-a5 — exclusion et compteur | 2 | vitest pur |
| AC-a6 — porteur d'IDE | 3 | vitest pur |
| AC-a7 — pureté du module | 1 | vitest pur |

**Total, sommé depuis la colonne : 13 preuves, toutes unitaires, aucune n'exigeant DOM, base ni réseau.**

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
| introduire un import réseau | le gate quitte le régime de la seconde | AC-a7 |

### Conventions

`@testing-library/svelte` v5 + Svelte 5 sont disponibles, mais **cette story n'en a pas besoin** : tous ses tests sont des appels de fonction. Patron de rédaction des doc-comments de test : `frontend/src/routes/(app)/products/products-page.test.ts`, dont l'en-tête explique pourquoi une assertion sur l'argument d'un mock attrape des mutations qu'un test fonctionnel laisse passer.

### References

- Story **22-2** (umbrella, statut `split`) — les quatre passes de revue et leur Change Log complet. **C'est la source de toutes les décisions reproduites ici.**
- Story **22-2b** — la surface, qui consomme ce module.
- Issue **#301** — le besoin.
- ⚠️ **Issues #314 et #315** — les défauts **à la source** que ce module contourne. `normalizeTerm` (D-a3) atténue #314 ; `rank` (D-a5) atténue #315. **Aucune des deux fonctions ne corrige quoi que ce soit dans la recherche elle-même** — elles compensent au client ce que la base ne sait pas faire, et le doc-comment du module doit le dire, faute de quoi un lecteur futur croira le problème réglé.
- `CLAUDE.md` — § *Règle de splitting préventif* (le critère qui a déclenché ce découpage), § *Test Locally First*.

## Dev Agent Record

### Agent Model Used

*(à remplir)*

### Debug Log References

### Completion Notes List

### File List
