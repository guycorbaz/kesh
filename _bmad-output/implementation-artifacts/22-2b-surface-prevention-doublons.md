# Story 22.2b : La surface — deux sondes, deux signaux, et rien qui bloque

## Status

ready-for-dev

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

## Décisions

**D-b1 — Signaler, jamais bloquer.** Deux clients peuvent légitimement porter des noms très proches : deux sociétés d'un même groupe, deux homonymes, un père et son fils. Le dispositif **informe**. Un blocage produirait des contournements — un espace ajouté au nom — qui salissent le carnet plus sûrement qu'un doublon assumé.

**D-b2 — L'IDE est un signal fort, le nom un signal faible.**

| Ce qui correspond | Avertissement |
|---|---|
| **Numéro IDE identique** | Franc : c'est un identifiant d'État, deux entités ne le partagent pas. |
| **Nom proche** | Nuancé : *« un contact au nom proche existe »*, avec de quoi le reconnaître. |

**D-b3 — Aucune route nouvelle ; une seule extension du chemin existant.** *(Q1 de la 22-2, tranchée par Guy.)*
`GET /api/v1/contacts?search=…&limit=…` fait déjà l'essentiel : scopé société (`routes/contacts.rs:591`), archivés exclus par défaut (`push_where_clauses:158`), `name` en FULLTEXT. Il manque une seule chose : **`ide_number` n'est pas cherché**. Il rejoint donc l'email et le numéro de client dans les **deux** branches `LIKE`.
⚠️ **Les deux branches, ou aucune.** N'en traiter qu'une compile, passe les tests dont le terme survit à `escape_boolean_ft`, et cesse **silencieusement** de chercher l'IDE quand le terme n'est fait que d'opérateurs.
⚠️ **Le bénéfice de bord est plus faible qu'il n'y paraît** : `'CHE109322551' LIKE '%CHE-109.322.551%'` rend **0** — l'IDE est stocké normalisé sur 12 caractères, le `LIKE` porte sur le terme brut. Seule la saisie **sans séparateurs** remontera le contact, et c'est justement pas la forme qu'on lit sur une facture papier. Ne pas réécrire la promesse inverse.

**D-b4 — « Réutiliser le chemin » n'est pas « c'est indexé ».** Mesuré :

| Requête | Plan | Lignes examinées |
|---|---|---|
| `MATCH` seul | `fulltext`, clé `ft_contacts_name` | 1 |
| **`MATCH OR LIKE` (la requête réelle)** | **`range`, clé `idx_contacts_company_name`** | **1506** |

Sur 3016 contacts : **1,26 ms + 2,36 ms** pour la paire `COUNT` + `SELECT` d'une sonde, contre 0,09 ms en `MATCH` seul. Une pause de frappe émet **quatre** requêtes — deux sondes × (`COUNT(*)` + `SELECT`), le `COUNT` étant émis même avec un `limit` de 20. **#301 demandait d'instruire ce coût** : le voici, ≈ 4 ms par sonde à 3000 contacts, acceptable. Ce qu'il ne faut pas écrire, c'est « c'est indexé » — un dev à qui l'on dit cela ne mesurera jamais.

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
Le dialogue se ferme depuis **trois** sites (`+page.svelte:352`, `:358`, `:846`) et n'est **jamais démonté** — le nettoyage d'`onMount:113` ne joue qu'à la sortie de la page. Poser le nettoyage sur la fermeture obligerait à le poser trois fois, et un quatrième site futur le contournerait en silence. `openCreate()` et `openEdit()` (`:216-264`) réinitialisent déjà champ par champ : minuteries, compteurs et propositions s'y ajoutent.

**D-b9 — Une proposition informe ; elle ne navigue pas.**
`openEdit(c)` réassigne **les dix-neuf champs** du formulaire (`+page.svelte:246-269`) : un clic mal spécifié **effacerait la saisie en cours sans confirmation**. La proposition est **inerte**. C'est la position cohérente avec D-b1, et la moins coûteuse : elle dispense de spécifier la protection de la saisie **et** le sort d'un contact archivé entre l'affichage et le clic.

**D-b10 — La localité s'affiche, elle ne se cherche pas.** Une correspondance « nom **et** localité » serait un signal plus fort, mais l'obtenir demanderait une dimension de requête supplémentaire — alors que le même service est rendu **à coût nul** en affichant la localité : c'est l'utilisateur qui discrimine, et il le fait mieux qu'un seuil.
⚠️ La localité vaut `""` pour tout contact sans adresse (`routes/contacts.rs:213-219`, `city: c.address_city.unwrap_or_default()`) — c'est le cas de la fiche minimale que cette story vise. Quand ni localité ni numéro de client n'existent, **replier sur l'email** : deux propositions ne doivent jamais être rigoureusement identiques à l'écran.

## Acceptance Criteria

**AC-b1 — La frappe du nom fait apparaître les contacts proches, le plus proche en premier.**
Dès que le module de la 22-2a arme la sonde, les contacts **actifs** de la société sont interrogés avec `limit: 20`, classés et coupés à cinq, avec la mention « et N autres » quand il en reste. Chaque proposition porte le **nom complet**, puis la **localité** et le **numéro de client** quand ils sont renseignés, à défaut l'**email** (D-b10). Les propositions sont **inertes** (D-b9).
*Preuve*, **5** tests de composant plus **1** d'intégration base :
1. `Entreprise` — les trois éléments d'affichage sont rendus. Mutation : « ne rendre que `c.name` », qui supprime le mécanisme de discrimination de la story sans faire rougir rien d'autre ;
2. `Personne` — la sonde part sur prénom + nom. Mutation : « ne lire que la raison sociale » ;
3. un contact **sans adresse et sans numéro de client** : l'email est rendu, aucune ligne vide ni tiret orphelin ;
4. **l'argument de l'appel** : `expect(listContactsMock).toHaveBeenCalledWith(expect.objectContaining({ includeArchived: false, limit: 20 }))`.
   ⚠️ **Une preuve fonctionnelle ne convient PAS.** `includeArchived` est un paramètre de query-string traité **par le serveur** ; sous `vi.mock`, le double rend ce qu'on lui a dit **sans regarder son argument**. « Un archivé n'apparaît pas » reste donc vert sous la mutation `false → true`. C'est ce qu'énonce le doc-comment de `products-page.test.ts` : *seule l'assertion sur l'argument l'attrape* ;
5. **cliquer sur une proposition ne modifie NI le formulaire NI son état** (D-b9). Mutation : `onclick={() => openEdit(c)}` — un ajout de bonne foi, qui compile et efface dix-neuf champs ;
6. **`#[sqlx::test]` — la fenêtre SQL est assez large.** Fixture des six `Jean X` ; `limit: 20` doit rendre **les six** lignes et `total = 6`. Mutation : « revenir à `limit: 5` ». ⚠️ Cette preuve porte sur la **requête** — sémantique OU, tri alphabétique, taille de fenêtre — et sur elle seule ; les preuves de composant passent par un `vi.mock` et en sont aveugles. *(Le classement lui-même est prouvé par la 22-2a, en TypeScript : un test Rust ne peut pas appeler `rank()`.)*

**AC-b2 — Un IDE déjà pris est signalé franchement, y compris quand le porteur est archivé.**
La saisie d'un IDE **complet** (`validateIdeFormat`) déjà porté par un contact de la société — **actif ou archivé** (D-b5) — déclenche un avertissement explicite, **distinct** du signal « nom proche », **avant** l'enregistrement. Quand le porteur est archivé, le message **le dit**.
⚠️ `uq_contacts_company_ide` refuse déjà le doublon à l'enregistrement, en `409 IDE_ALREADY_EXISTS` (`map_contact_error:536-541`). Cette AC ne remplace pas la garde : elle évite d'y arriver **après** avoir saisi toute la fiche.
*Preuve*, **5** tests de composant plus **1** assertion E2E :
1. l'avertissement s'affiche sur un porteur **actif** ;
2. **l'argument** `includeArchived: true`, **et** le message rendu pour un porteur archivé est **distinct** de celui d'un porteur actif — deux clés i18n séparées. ⚠️ Les deux moitiés sont indispensables : l'argument parce qu'une preuve fonctionnelle est verte sous la mutation, le libellé parce que **c'est lui le recours** — sans la mention « archivé », l'utilisateur reçoit un avertissement sur un contact introuvable dans son carnet, qu'il ne pourra ni modifier ni désarchiver ;
3. **ouvrir en édition** la fiche d'un contact qui porte un IDE ne déclenche **aucun** avertissement franc — ni à l'ouverture, ni après avoir retapé le même IDE ;
4. **vider le champ IDE pendant que la sonde est en vol**, puis résoudre avec un lot contenant un contact **sans IDE** ⇒ aucun avertissement. *(La garde est structurelle dans la 22-2a ; ce test vérifie que le câblage lui passe bien la valeur **envoyée**, et non une relecture du champ.)* ;
5. **le retrait** : afficher l'avertissement sur un IDE pris, corriger un chiffre vers un IDE **libre** ⇒ plus aucun avertissement. Mutation : « n'écrire l'avertissement que dans la branche `if (holder)` », qui laisse à l'écran un message désignant un numéro que l'utilisateur ne porte plus ;
6. **E2E** : le `409` est bien levé si l'avertissement est ignoré (cf. T-b6).

**AC-b3 — Les sondes ne changent RIEN à l'état du bouton.** *(Propriété différentielle, pas absolue.)*
L'état du bouton d'enregistrement est **exactement celui qu'il aurait sans les sondes** : ni `disabled` ni `formValidation` ne sont touchés.
⚠️ **Formuler l'AC en absolu — « le bouton reste actif » — serait FAUX** dans des états que la story couvre par ailleurs : le formulaire désactive **déjà** le bouton, aujourd'hui et sans rapport avec cette story, quand une `Personne` n'a qu'un des deux champs de nom (`:276-279`), quand l'adresse est partielle (`:281-282`), ou quand l'IDE est partiellement tapé (`:286-288`). Un dev qui écrirait la preuve sur l'un de ces états verrait un bouton désactivé et pourrait « corriger » `formValidation` — c'est-à-dire **supprimer une garde préexistante pour satisfaire une AC mal bornée**.
*Preuve*, **1** assertion E2E, jouée sur un formulaire **par ailleurs valide** (cf. T-b6).

**AC-b4 — Rien ne part avant que la frappe se calme, et une réponse tardive ne dit plus rien.**
Temporisation de 300 ms (le pas du dépôt), compteur de génération, **une paire par sonde** (D-b7), état remis à zéro à l'ouverture (D-b8).
*Preuve*, **5** tests unitaires :
1. **le nombre d'appels** sur une frappe continue de vingt caractères, minuteries factices — **attendu : un seul** ;
2. deux requêtes résolues **dans l'ordre inverse** — la première arrivée en dernier est ignorée ;
3. **le test croisé** : armer la sonde nom, puis la sonde IDE avant résolution ⇒ **les deux** avertissements aboutissent. Mutation : « factoriser une seule paire », que les deux autres laissent verte ;
4. **la sonde IDE sur une suite valide → invalide → valide** ⇒ **deux** appels, un par passage à la validité ;
5. **la remise à zéro à l'ouverture** : armer une sonde, fermer le formulaire, le rouvrir ⇒ aucune proposition, et la réponse armée avant la fermeture n'en fait pas apparaître. Mutation : « ne rien réinitialiser dans `openCreate()` ».
⚠️ Ne PAS prouver l'annulation par un `AbortSignal` passé à `apiClient` : il est écrasé en silence (D-b6), un tel test mesurerait du vide.

**AC-b5 — Le dispositif est muet quand il n'a rien à dire.**
- **(a)** la sonde non armée par le module de la 22-2a ⇒ **aucune requête** ;
- **(b)** armée, aucun contact proche ⇒ aucun encombrement, **aucun texte** ;
- **(c)** une sonde qui **échoue** est muette elle aussi — réseau coupé, `500`, réponse illisible : aucun message d'erreur, aucune exception remontée, et **l'autre sonde continue**. Une sonde n'est pas une action de l'utilisateur : il n'a rien demandé, on ne lui doit aucun rapport d'échec.
*Preuve*, **3** tests de composant : pour (a), la saisie de deux caractères ne déclenche **aucun** appel ; pour (b), un carnet vide ne rend **aucun texte** — ⚠️ asserter l'absence de **texte**, jamais l'absence de **nœud**, les zones restant en permanence dans le DOM pour qu'`aria-live` fonctionne (T-b4) ; pour (c), la sonde IDE répond `500` pendant que la sonde nom répond `200` ⇒ aucun message d'erreur, et les propositions s'affichent normalement. Mutation : « retirer le `try/catch` d'une sonde ».

**AC-b6 — Les avertissements sont traduits dans les quatre locales.**
*Preuve*, **1** test Rust dédié dans `crates/kesh-i18n/src/loader.rs`, sur le patron de `client_number_labels_are_translated_in_all_four_locales` (`loader.rs:260-277`) — pour chaque clé neuve, `!= key` **et** `!= fr` sur `de-CH`, `it-CH`, `en-CH`. Plus `npm run lint-i18n-ownership` vert.
⚠️ **Il n'existe AUCUN test de parité globale dans `kesh-i18n`.** Le loader **replie silencieusement sur le français** (`format_missing_key_in_de_falls_back_to_fr`, `loader.rs:227`), et les fichiers sont déjà désappariés de 57 clés (KF #283). Une clé oubliée dans trois locales **ne rougit nulle part** : c'est l'assertion `!= fr` qui l'attrape, et rien d'autre.
⚠️ Le domaine `contact-*` compte **62 clés dans chacune** des quatre locales — recompté le 2026-08-17. Le laisser apparié.

**AC-b7 — L'avertissement vaut aussi à l'édition, sans se signaler lui-même.**
Renommer un contact vers un nom déjà porté déclenche le même signal nuancé, le contact édité étant exclu de ses propres résultats.
*Preuve*, **2** tests de composant :
1. ouvrir la fiche d'un contact `Entreprise`, modifier sa raison sociale d'un caractère ⇒ il **ne figure pas** dans les propositions ;
2. **et AUCUNE mention « et N autres » n'apparaît** quand il était le seul à correspondre. ⚠️ Le compteur est calculé par la 22-2a, mais **c'est ici qu'on vérifie qu'on lui passe bien `editing?.id`** : sans cela, corriger un caractère affiche « et 1 autre » **au-dessus d'une liste vide**, en désignant la fiche qu'on modifie. La preuve 1 reste **verte** sous ce défaut, puisqu'elle ne regarde que la liste.

## Tasks / Subtasks

- [ ] **T-b1 — Rendre l'IDE cherchable, et prouver la fenêtre** (AC-b2, **et preuve 6 d'AC-b1**). Ajouter `ide_number` aux **DEUX** branches `LIKE` de `push_where_clauses` (`crates/kesh-db/src/repositories/contacts.rs:195-210`).
  - [ ] Deux tests `#[sqlx::test]`, **un par branche** : le cas courant (`search=CHE109322551`) et le cas où `escaped.is_empty()`, c'est-à-dire un terme **intégralement** composé des dix opérateurs — par exemple `search=***`.
  - [ ] ⚠️ **Ne PAS se caler sur `test_search_handles_special_chars:1283` pour la seconde branche : il ne l'exerce pas.** Il cherche `"100%"`, et `%` **n'est pas** un opérateur `BOOLEAN MODE` (`util/search.rs:41`) — le terme survit intact et le test emprunte la branche `else`. **Aucun test du dépôt n'exerce aujourd'hui la branche `escaped.is_empty()`.**
  - [ ] **Le `#[sqlx::test]` de la fenêtre** (preuve 6 d'AC-b1) : fixture des six `Jean X`, `limit: 20` ⇒ six lignes et `total = 6`.
  - [ ] ⚠️ **`push_where_clauses` est un chemin PARTAGÉ** — cf. § *Rayon d'impact de T-b1*. Gate **complet** au dernier commit : la § *Test Locally First* interdit le ciblage dès qu'un patch touche `crates/kesh-db/`.
- [ ] **T-b2 — Brancher les deux sondes** (AC-b1, AC-b2, AC-b5). Importer le module de la **22-2a** — `normalizeTerm`, `buildTerm`, `isArmed`, `rank`, `excludeSelf`, `countOthers`, `findIdeHolder` — et **ne réimplémenter aucune de ces fonctions**.
  - [ ] Forme exacte des deux appels :

```ts
// AC-b1 — sonde « nom proche ». Actifs SEULEMENT (D-b5), fenêtre large, soi exclu.
const normalized = normalizeTerm(buildTerm(formContactType, formName, formFirstName, formLastName));
if (!isArmed(normalized)) return;                       // 22-2a décide, pas nous
const rn = await listContacts({ search: normalized, limit: 20, includeArchived: false });
const retenus = rank(excludeSelf(rn.items, editing?.id), normalized);
const proches = retenus.slice(0, 5);
const autres  = countOthers(rn.total, rn.items, proches, editing?.id);

// AC-b2 — sonde IDE. Archivés COMPRIS (D-b5), vérification sur le champ.
const ide = normalizeIdeForApi(formIde);
if (!ide || !validateIdeFormat(formIde)) return;
const ri = await listContacts({ search: ide, limit: 5, includeArchived: true });
const holder = findIdeHolder(ri.items, ide, editing?.id);   // `ide` = la valeur ENVOYÉE
```

  - [ ] ⚠️ **Les deux appels ne diffèrent que par deux paramètres et se lisent côte à côte.** Une inversion d'`includeArchived` ne casse rien, ne fait rougir aucun test **fonctionnel**, et rend des résultats plausibles dans les deux sens — le filtrage étant fait par le serveur et un `vi.mock` ne regardant pas ses arguments. D'où deux preuves **sur l'argument**, une par sens.
- [ ] **T-b3 — Temporisation et garde d'ordre** (AC-b4). Débounce 300 ms + compteur de génération, patron `ContactPicker.svelte:36-78` (D-b6). **Une paire par sonde** (D-b7).
  - [ ] Réutiliser `debounce` de `$lib/features/journal-entries/debounce.ts` **ou** le `setTimeout` inline de `+page.svelte:185-192` — ne pas écrire un troisième mécanisme. Le helper est mal rangé ; le **déplacer** est hors périmètre, s'en servir ne l'est pas.
  - [ ] Remise à zéro des deux paires dans `openCreate()` **et** `openEdit()` (D-b8) — **pas** sur la fermeture, qui a trois sites.
  - [ ] Nettoyer les timers au démontage — `+page.svelte:113` le fait déjà pour la recherche de liste.
  - [ ] Brancher sur **`oninput`** (`ContactPicker.svelte:127`, `+page.svelte:463`), **jamais sur `onkeydown`** : un collage ne produit aucune frappe clavier, et une sonde branchée sur le clavier resterait muette sur le geste le plus courant de tous.
- [ ] **T-b4 — Balisage et signaux** (AC-b1, AC-b2, AC-b3, AC-b5, AC-b7). Deux niveaux visuellement distincts (D-b2) ; ne toucher **ni** au `disabled` du bouton **ni** à `formValidation` (`:275`).
  - [ ] **Chaque avertissement est rendu IMMÉDIATEMENT SOUS le champ qui le déclenche** — propositions sous le nom, avertissement franc sous `#form-ide` — et **jamais** dans le bloc `formError` du bas : le `Dialog.Content` est en `max-h-[90vh] overflow-y-auto` (`:637`) et **défile en interne**. Un avertissement hors écran est un avertissement muet.
  - [ ] **Les deux zones sont rendues EN PERMANENCE dans le DOM**, `aria-live="polite"`, contenu vide quand il n'y a rien à dire. Patron : `ResetPasswordForm.svelte:162-169`, dont le commentaire dit « *toujours dans le DOM pour que aria-live fonctionne* ». ⚠️ **Ne PAS copier le `role="combobox"` de `ContactPicker`** : c'est un patron de **sélection**, inadapté à un avertissement passif (D-b9), et il ne porte aucun `aria-live`.
  - [ ] Encapsuler **chaque** sonde dans un `try/catch` qui se tait, comme `ContactPicker.svelte:64-67`.
  - [ ] **L'avertissement franc s'efface quand `formError` prend le relais** après un `409` (`:361` pose déjà `contact-error-ide-duplicate`), pour que la même phrase ne s'affiche pas deux fois.
  - [ ] Il n'existe pas de composant `alert` dans `lib/components/ui/` — se caler sur le style de `formError`, sans créer de primitive.
  - [ ] **Créer `frontend/src/routes/(app)/contacts/+page.test.ts`** — il n'existe **aucun** test de cette page (seul `contact-helpers.test.ts` existe pour ce domaine). Patron : `products-page.test.ts`, mocks hoistés avant l'import.
- [ ] **T-b5 — i18n** (AC-b6). Clés neuves dans les **quatre** FTL, domaine `contact-*`, plus le test Rust dédié.
  - [ ] **Clés attendues** : l'en-tête des propositions, la mention « et N autres », l'avertissement franc sur porteur **actif**, et celui sur porteur **archivé** — ce dernier **distinct**, c'est la preuve 2 d'AC-b2.
  - [ ] ⚠️ **Ne PAS réutiliser `contact-error-ide-duplicate`** (`fr-CH:374`) : elle sert au message d'échec du `409`, et ne sait ni nommer le porteur ni dire qu'il est archivé.
  - [ ] **Mettre à jour `contact-filter-search-placeholder`** dans les 4 locales **et** son fallback en dur (`+page.svelte:455`) : il **énumère les colonnes cherchées**, et T-b1 en ajoute une.
- [ ] **T-b6 — E2E** (AC-b3, **et preuve 6 d'AC-b2**). ⚠️ Le fichier **DOIT** être nommé `*.spec.ts` : `playwright.config.ts:35` filtre sur `testMatch: /(.+\.)?spec\.[jt]s/`, et un `*.test.ts` posé dans `tests/e2e/` est **silencieusement ignoré**.
  - [ ] Patron : `frontend/tests/e2e/contact-client-number.spec.ts`, `seedTestState('with-company')` puis login.
  - [ ] ⚠️ **ARCHIVER NE LIBÈRE PAS L'IDE** — contrainte **plate** (D-b5), contrairement au numéro de client dont le patron vient. Chaque assertion emploie un **IDE distinct à checksum valide** (`CHE109322551`, `CHE116281838`, tous deux vérifiés) ; l'isolation entre exécutions repose sur le `truncate_all` de `seedTestState`. Et l'astuce `CLI-${Date.now()}` de la 22-1 est **intransposable** : le dernier chiffre d'un IDE est un checksum modulo 11 (`che_number.rs:102-131`).
  - [ ] **Test 1 (AC-b3), trois assertions DANS CET ORDRE**, sur une `Entreprise` par ailleurs valide : (i) **le signal nuancé est VISIBLE** ; (ii) le bouton est **activé** ; (iii) soumettre **crée bien le contact**, vérifié après rechargement.
  - [ ] **Test 2 (preuve 6 d'AC-b2), trois assertions DANS CET ORDRE** : (i) **le signal franc est VISIBLE** ; (ii) le bouton est **activé** ; (iii) soumettre **échoue en `409`** et l'interface le dit.
  - [ ] ⚠️ **La première assertion de chaque test n'est PAS décorative : c'est la seule qui prouve quelque chose de cette story.** Les assertions (ii) et (iii) sont **VERTES SUR `main` AUJOURD'HUI** — créer un contact fonctionne déjà, un IDE dupliqué donne déjà `409`. Sans (i), la mutation « **n'implémenter aucune sonde** » laisse les deux tests au vert.
- [ ] **T-b7 — Documentation** (AC-b1, AC-b2). `docs/manual/fr/user-manual.tex`, § *Contacts* → *Création d'un contact* (ligne 506) : ce que l'avertissement dit, et **qu'il n'empêche rien**. Régénérer le PDF (`make fr`) et le commiter.
  - [ ] **Corriger `user-manual.tex:532`**, qui **énumère** les colonnes cherchées (« sur le **nom**, complété par une recherche sur l'**email** et le **numéro de client** ») — T-b1 en ajoute une.
  - [ ] ⚠️ Ne PAS toucher à la ligne 538 (« *Contacts → Import CSV* »), qui annonce une fonction inexistante : c'est **#291**, une issue à part. **À ne pas confondre avec la 532.**

## Décompte des preuves — la seule table qui fait foi

| AC | Preuves | Nature |
|---|---:|---|
| AC-b1 — nom proche | 5 + 1 | composant + `#[sqlx::test]` |
| AC-b2 — IDE déjà pris | 5 + 1 | composant + E2E |
| AC-b3 — l'état du bouton est inchangé | 1 | E2E |
| AC-b4 — temporisation et ordre | 5 | unitaire front |
| AC-b5 — muet quand rien à dire | 3 | composant |
| AC-b6 — quatre locales | 1 | unitaire Rust |
| AC-b7 — édition sans se signaler | 2 | composant |
| *(T-b1 — l'IDE cherchable)* | 2 | `#[sqlx::test]` |

**Totaux, sommés depuis la colonne** : **15 tests de composant** (5 + 5 + 3 + 2) · **5 tests unitaires front** · **2 assertions E2E** (1 + 1) · **1 test unitaire Rust** · **3 tests d'intégration base** (1 + 2). **Soit 26 preuves.**

Aucun autre passage de cette story n'énonce de total. *(S'y ajoutent les **13 preuves** de la 22-2a, comptées chez elle.)*

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

### Les huit pièges muets, nommés

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

## Dev Agent Record

### Agent Model Used

*(à remplir)*

### Debug Log References

### Completion Notes List

### File List
