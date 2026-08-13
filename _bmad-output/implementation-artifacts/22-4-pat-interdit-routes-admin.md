# Story 22.4 : Un jeton PAT ne doit jamais atteindre une route d'administration

## Status

backlog

## Story

**As a** administrateur d'une instance Kesh exposant une API à jeton,
**I want** qu'un jeton fuité ne puisse en aucun cas créer un utilisateur, réinitialiser un mot de passe ou modifier la configuration de la société,
**so that** **révoquer le jeton suffise à arrêter l'incident** — ce qui n'est pas le cas aujourd'hui.

Ferme **#167** (KF-036). Quatrième story de l'**Epic 22 « Technical Debt Closure »**, et la seule défaillance connue ouverte qui touche à la **sécurité**.

## Contexte

Le middleware `require_auth` accepte un jeton `Bearer kesh_pat_…` sur **toutes** les routes `/api/v1/*`. Le `CurrentUser` qu'il construit porte le **rôle courant du créateur** du jeton, relu en base. Un jeton `read-write` créé par un Admin franchit donc le contrôle de portée, puis le contrôle RBAC — et se retrouve devant les routes d'administration comme s'il était cet Admin devant son navigateur.

### Pourquoi cela dépasse un simple défaut de permission

La décision **DC6** de la story 17-2 interdit déjà à un PAT de **gérer les clés** (`ensure_not_pat` sur `/api/v1/settings/api-keys`), et son motif est explicite : *« une clé fuitée ne peut pas se cloner ni s'escalader »*. Mais le blocage s'arrête là, si bien qu'il existe un contournement :

```
PAT read-write fuité (créateur Admin)
  → POST /api/v1/users            (nouvel admin, mot de passe choisi par l'attaquant)
  → login par l'interface         (en tant que ce nouvel admin)
  → POST /api/v1/settings/api-keys (nouvelles clés, à volonté)
⇒ révoquer la clé d'origine n'arrête plus rien
```

**C'est la propriété de containment qui tombe**, pas seulement une permission. Un incident de fuite de clé se traite normalement par une révocation ; ici la révocation devient inopérante dès qu'un compte a été créé.

### Portée réelle — et elle est étroite

⚠️ **Seuls les jetons créés par un Admin sont concernés.** Un PAT créé par un Comptable est arrêté par `require_admin_role` : aucune escalade possible, et c'est le cas d'usage nominal (DC4, « gérer ses intégrations »). Les clés Admin sont rares, et la révocation reste efficace **tant qu'aucun compte n'a été créé**. Ce n'est pas une régression : la story 17-2a est la première introduction du PAT.

## L'état réel du code, recompté — et il diffère de l'issue

L'issue #167 date de plusieurs mois. Deux de ses éléments ont bougé, et un troisième était sous-estimé.

**Les références de ligne sont périmées** : elle cite `lib.rs:102-126`, le routeur admin est aujourd'hui aux lignes **182 à 284**.

**Une remédiation partielle a eu lieu depuis**, sans que l'issue soit mise à jour : `ensure_not_pat` a été posé sur `admin/full-export`, `admin/full-import` (Story 17-3a/17-3c) et `fiscal-years/{id}/reopen`.

**Et le trou est bien plus large que « les routes user-management »** — recompté sur le routeur `admin_routes` :

| Route sous `require_admin_role` | Gardée contre PAT ? |
|---|---|
| `/api/v1/users` · `/users/{id}` · `/disable` · `/reset-password` | ❌ |
| `/api/v1/company/invoice-settings` | ❌ |
| `/api/v1/vat-rates` · `/vat-rates/{id}` | ❌ |
| `/api/v1/company/dunning-settings` | ❌ |
| `/api/v1/dunning-levels` · `/dunning-levels/{id}` | ❌ |
| `/api/v1/invoices/{id}` (suppression) | ❌ |
| `/api/v1/invoices/{id}/reminders/{reminderId}/cancel` | ❌ |
| `/api/v1/admin/email-templates` (×2) | ❌ |
| `/api/v1/companies/current/email` · `/contact-details` | ❌ |
| `/api/v1/admin/full-export` · `/full-import` | ✅ |
| `/api/v1/fiscal-years/{id}/reopen` | ✅ |

**Trois routes gardées sur dix-neuf.** Ce n'est pas seulement la gestion des utilisateurs : ce sont aussi les taux de TVA, les paramètres de relance, les modèles d'e-mail et les coordonnées de la société.

### Dix-neuf enregistrements, VINGT-CINQ routes — et c'est le second nombre qui compte

⚠️ **Le « dix-neuf » ci-dessus compte des enregistrements `.route(…)`, pas des routes.** Un routeur axum route par **méthode** : `/api/v1/users` porte un `GET` **et** un `POST`. Recompté ligne à ligne sur `lib.rs:182-284` :

| Méthode | Couples |
|---|---|
| `get` | 5 |
| `post` | 6 |
| `put` | 10 |
| `delete` | 4 |
| **Total** | **25** |

**Cinq** des dix-neuf enregistrements portent deux ou trois méthodes : `/users`, `/users/{id}`, `/vat-rates/{id}`, `/dunning-levels/{id}` et `/admin/email-templates/{template_type}/{language}` (trois à lui seul). Ajouter une méthode à l'un d'eux ne change **pas** le nombre d'enregistrements : un compteur calibré sur les `.route(` n'y verrait rien — c'est le mode d'échec que cette story existe pour fermer, reproduit dans son propre garde-fou.

> ⚠️ **Deux chiffres de ce paragraphe étaient faux, et le second était un précédent inventé.** La passe 2 affirmait « six enregistrements » — il y en a **cinq** — et surtout que `/admin/email-templates/{…}` serait « passé d'une à trois méthodes entre les stories 20-1 et 20-2 ». **C'est faux** : la route est née avec ses trois méthodes, en un seul commit.
>
> ```sh
> git log --all --oneline -S'email-templates/{template_type}/{language}' -- crates/kesh-api/src/lib.rs
> #   8bca0a1e feat(story-20-1): implémente le socle templates d'e-mail (backend)
> git show 8bca0a1e^:crates/kesh-api/src/lib.rs | grep -c email-templates   # 0 — la route n'existe pas encore
> git show 8bca0a1e:crates/kesh-api/src/lib.rs  | sed -n '197,200p'         # get + put + delete, d'emblée
> ```
>
> **L'argument de conception ne dépend pas de ce précédent** — cinq enregistrements multi-méthodes suffisent à établir la classe d'évolution contre laquelle le compteur doit protéger. Mais il était présenté comme un fait advenu, et il ne l'est pas. *(Réfuté en passe 3, `git log -S` à l'appui. Le relevé de passe 2 portait la mention « recompté à la main » et « confirmé par les trois lentilles indépendamment » : ni le recompte ni la confirmation n'avaient eu lieu sur ce point.)*

## Décisions

**D1 — Le blocage se pose en COUCHE sur le routeur, pas dans les handlers.**

L'issue proposait les deux options. **L'état du code tranche** : l'approche par handler a été appliquée trois fois et **oubliée seize**. Ce n'est pas un défaut de diligence, c'est la propriété d'un dispositif qui doit être rappelé à chaque ajout de route — et rien ne le rappelle. Un `route_layer` sur `admin_routes`, à côté de `require_admin_role` qui vit déjà là, se pose **une fois** et couvre toute route ajoutée ensuite **dans le bloc**.

> ⚠️ **« Par construction » est trop fort, et la nuance est un vrai trou : `route_layer` n'enveloppe que les routes DÉJÀ présentes à l'appel.** Lu dans le source (`axum-0.8.9/src/routing/path_router.rs:311-341`), il itère sur `self.routes` au moment où il est appelé :
>
> ```rust
> let routes = self.routes.into_iter()
>     .map(|(id, endpoint)| (id, endpoint.layer(layer.clone())))
>     .collect();
> ```
>
> Une route chaînée **après** les `.route_layer(...)` — un `admin_routes = admin_routes.route(...)` ajouté plus loin, ou un remaniement qui remonte les couches « pour la lisibilité » — **compile, ne panique pas** (le `panic!` ne se déclenche que sur un routeur vide) et échappe **aux deux** couches : ni RBAC, ni anti-PAT. Elle ne garderait que `require_auth`, c'est-à-dire l'authentification sans l'autorisation.
>
> **Ce trou préexiste à cette story** — il vaut déjà pour `require_admin_role` seul — mais la story ne le referme pas, et sa promesse de complétude le masquerait. Deux gestes, en T1 : un **commentaire d'avertissement** aux deux `.route_layer(...)` disant que tout ajout se fait au-dessus, et **une assertion de source** dans le test de T3 — aucun `.route(`, `.nest(` ou `.route_service(` après le dernier `.route_layer(` du bloc. *(Relevé en passe 3, source d'axum à l'appui.)*

**D2 — Un code d'erreur distinct, ET trois tests existants à mettre à jour.** `AppError::ApiKeyManagementForbidden` rend aujourd'hui `403 API_KEY_MANAGEMENT_FORBIDDEN`. Le réutiliser pour « vous ne pouvez pas rouvrir un exercice avec un jeton » **mentirait à l'appelant** : son message parle de gestion de clés. Une variante dédiée — `API_KEY_ADMIN_FORBIDDEN` — avec son message propre sur les quatre locales.

> ⚠️ **Cette décision a une conséquence que la première rédaction avait manquée, et qui contredisait D3.**
>
> `route_layer` enveloppe le service : la couche répond **avant** que le handler ne soit appelé — quel que soit son ordre par rapport à `require_admin_role`. Les trois routes déjà gardées répondront donc le **nouveau** code, et non plus celui que leurs tests assertent :
>
> | Test | Ligne | Assertion actuelle |
> |---|---|---|
> | `full_export_via_pat_returns_403` | `admin_full_export_e2e.rs:409` | `API_KEY_MANAGEMENT_FORBIDDEN` |
> | `full_import_via_pat_returns_403` | `admin_full_import_e2e.rs:414` | `API_KEY_MANAGEMENT_FORBIDDEN` |
> | `reopen_via_pat_returns_403` | `fiscal_years_e2e.rs:1638` | `API_KEY_MANAGEMENT_FORBIDDEN` |
>
> **Ces trois assertions DOIVENT changer, et c'est la seule modification de test autorisée par cette story.** La première rédaction disait l'inverse — « sans modification » — et avertissait même qu'une modification signalerait « une couche posée trop haut ». **Cet avertissement égarait** : la cause n'est pas le placement, qui est correct, mais l'interaction entre D2 et D3. *(Réfuté en passe 1 de `validate`, preuve d'exécution à l'appui.)*
>
> ⚠️ Ne PAS toucher à `api_keys_e2e.rs:425` : cette route-là est bien de la gestion de clés, son code reste le bon.

**D3 — Les trois `ensure_not_pat` d'`admin_routes` restent, et leur câblage devient prouvé par la SOURCE.** La couche les rend redondants sur ces routes, mais ils gardent une valeur : `full-export`, `full-import` et la réouverture d'exercice sont des opérations dont l'interdiction aux jetons est **intrinsèque**, indépendamment du routeur où elles vivent. Un déplacement de route ne doit pas les découvrir. *(Une redondance assumée et écrite vaut mieux qu'une garde retirée « parce que la couche s'en occupe » — mais elle a un prix, celui décrit en D2.)*

> ⚠️ **Et ce prix est plus élevé que D2 ne le disait : après T5, plus aucun test ne prouve que ces trois gardes sont câblées.** Leurs trois tests assertaient le code du handler ; basculés sur le nouveau code par T5, ils prouvent désormais **la couche**. Les retirer toutes les trois ne ferait alors **rougir aucun test** — la valeur que D3 revendique deviendrait invérifiable, et une « simplification » ultérieure les emporterait sans le moindre signal. *(Relevé en passe 2.)*
>
> **Un test unitaire sur `ensure_not_pat` n'y répond pas** : la fonction reste couverte par ses **trois autres** sites d'appel, ceux des routes de gestion de clés (`api_keys.rs:108`, `:121`, `:211`), qui vivent dans `comptable_routes` et que cette story ne touche pas. Ce qui perd sa couverture, c'est le **câblage des trois sites d'`admin_routes`** — `admin.rs:37`, `admin.rs:143`, `fiscal_years.rs:317` —, pas le comportement de la fonction.
>
> **Mécanisme retenu** : une assertion **de source**, dans le fichier de test de T3 et par le même `include_str!` — les trois fichiers doivent contenir leurs appels à `ensure_not_pat`. C'est la technique que cette story établit déjà pour la complétude, appliquée à une garde dont le seul mode d'échec est la disparition. *Repli si elle s'avère impraticable* : écrire noir sur blanc, dans une ligne de dette du story file, que ces trois gardes sont une défense en profondeur **sans couverture de test** — jamais laisser la revendication de D3 sans l'un des deux.

**D4 — La frontière DC6 est réécrite à ses QUATRE ÉNONCÉS, répartis sur TROIS documents.** Elle disait « un PAT ne gère pas les clés » ; elle dira « **un PAT n'atteint aucune route `require_admin_role`** ».

⚠️ **Compter les documents et compter les énoncés donne deux nombres différents, et la confusion des deux était un défaut de cette décision.** Relevé au `grep -rn "DC6"`, l'énoncé de la frontière vit à :

| Document | Ligne | Statut du document |
|---|---|---|
| `17-2a-api-pat-backend.md` | `35` (décision) **et** `58` (tâche) | `done` — le plus lu, avec son Dev Agent Record |
| `17-2-api-pat-integrations.md` | `57` | `ready-for-dev` — vestige de découpage, **mais** `17-2a:206` le désigne comme « spec parente convergée, **source des DC1-DC6** » |
| `17-2c-api-pat-doc.md` | `34` | `done` — et c'est **la source des passages de documentation qui deviennent faux** (cf. AC6) |

**Trois documents, quatre énoncés** — `17-2a` en porte deux. La formulation « quatre documents » a subsisté en D4 et en AC6 après que T6 avait déjà été corrigée en « quatre énoncés » : un correctif appliqué à un site et non propagé aux autres, le mode d'échec même que la § *Propagation post-patch* du `CLAUDE.md` demande de greper. *(Relevé en passe 3 par deux lentilles indépendamment.)*

⚠️ **Un QUATRIÈME document mentionne DC6 et reste hors périmètre — vérifié, pas supposé.** `17-2b-api-pat-frontend.md`, `done`, l'invoque à `:172` et `:220` : « la page est session-JWT-cookie uniquement ; un PAT ne peut pas l'atteindre (DC6 backend = filet) ». Ces deux phrases restent **vraies** après cette story, parce qu'elles portent sur la page de gestion des clés, dont la route vit dans `comptable_routes` et n'est donc pas touchée par la couche (cf. D5). **Rien à amender — mais il fallait l'écrire** : la passe 3 a relevé que l'affirmation « il n'y a rien à y faire » n'avait jamais été formulée ni vérifiée, alors que le document existe et porte le mot.

**Le « PAS dans `17-2` » de la première rédaction était donc à l'envers** : `17-2` n'est pas à éviter, il est à amender **avec** les autres. Un lecteur qui suit la piste `17-2a:206` vers la spec parente y trouverait sinon l'ancienne frontière, avec l'autorité de la « source ». *(Relevé en passe 2.)*

⚠️ **Les entrées de LIMITATION, elles, ne se réécrivent pas — elles se closent.** `17-2a:114`, `17-2a:255` et `17-2c:35` portent la dette L3 / KF-036 dans un Dev Agent Record et des sections de rétrospective : ce sont des **traces historiques**, et les réécrire effacerait la décision de l'époque. Elles reçoivent une mention « **fermé par la story 22-4 (#167)** », pas une réécriture.

**D5 — Le routeur `comptable_routes` reste HORS périmètre, et c'est une décision, non une question.** Vérifié : il ne porte aucune route capable de créer un utilisateur, de réinitialiser un mot de passe ou de changer la configuration de la société — les seuls vecteurs de la propriété de containment que cette story défend. Et le fermer casserait le cas d'usage nominal du PAT (DC4), qui est précisément d'écrire des écritures et des factures. *(Cette décision était rangée en question ouverte ; deux lentilles ont relevé qu'elle était en réalité tranchée, et qu'un « oui » ultérieur casserait AC2 sans garde-fou. Promue ici.)*

⚠️ **Conséquence à ne pas manquer en T2** : les routes de gestion des clés (`/api/v1/settings/api-keys`, `lib.rs:575` et `:579`) vivent dans `comptable_routes`, **pas** dans `admin_routes`. Le code `API_KEY_MANAGEMENT_FORBIDDEN` garde donc un consommateur vivant et testé (`api_keys_e2e.rs:425`) : la nouvelle variante **s'ajoute**, elle ne remplace ni ne renomme rien.

**D6 — Quand les DEUX couches refusent, c'est la couche anti-PAT qui répond.**

Le cas existe et n'était pas spécifié : un PAT créé par un **Comptable**, présenté sur une route d'`admin_routes`. `require_admin_role` le refuse (rôle insuffisant) **et** la nouvelle couche le refuse (c'est un PAT). L'ordre des deux `route_layer` détermine alors le code observable : `API_KEY_ADMIN_FORBIDDEN` ou le `Forbidden` du RBAC.

**Décision : `API_KEY_ADMIN_FORBIDDEN`, sans exception.** Un code unique pour « un PAT a touché une route admin », quel que soit le rôle de son créateur — c'est la seule forme qui rende AC1 assertable **par couple** sans se ramifier sur le rôle, et elle ne divulgue rien : le porteur du jeton sait déjà qu'il porte un jeton.

⚠️ **T1 « l'ordre n'a pas à être arbitré » était vrai pour le handler et faux pour le code de réponse.** La vérification de passe 1 établissait que `route_layer` enveloppe le service — donc que la couche précède le **handler**. Elle ne disait rien de l'ordre entre **deux couches**, qui est une autre question. Un arbitrage réel a été classé « sans objet » sur la foi d'une preuve qui portait ailleurs. *(Relevé en passe 2.)*

**Le test est l'arbitre, pas la lecture du source d'axum.** AC1 asserte le code attendu sur le cas « créateur Comptable » ; si le montage retenu rend l'autre code, le test rougit et T1 déplace la couche. Aucune affirmation sur la sémantique d'empilement d'axum n'est portée ici — elle n'a pas été vérifiée dans le source, et la story n'en a pas besoin.

## Acceptance Criteria

**AC1 — Aucune route admin n'est atteignable par un PAT, quel que soit son scope.**

**« Route » signifie ici un couple (méthode, chemin)**, et non un chemin : `/api/v1/users` porte un `GET` et un `POST`, qui sont deux routes. Le « 19 » du tableau ci-dessus compte des **enregistrements `.route(…)`** ; le nombre qui fait foi est **25**, recompté par méthode au § *Dix-neuf enregistrements, vingt-cinq routes*. *(Ambiguïté relevée en passe 1, chiffrée en passe 2 : un test calibré sur 19 serait structurellement incomplet.)*

*Preuve* : un test qui couvre **les 25 couples (méthode, chemin)** d'`admin_routes` avec un PAT dont le créateur est Admin, **aux deux scopes**, et qui asserte **le code d'erreur** et non le seul statut.

⚠️ **Asserter `403` ne prouve rien ici : TROIS gardes distinctes le produisent** sur ces routes — le RBAC, le gate de portée `read-only`, et la couche neuve. Un test conforme à la lettre « rend 403 » passerait **sans que la couche soit jamais atteinte**. La leçon est déjà écrite dans le dépôt, à `fiscal_years_e2e.rs:1634-1637` : « asserter le CODE prouve que le 403 vient bien de `ensure_not_pat` et non du middleware RBAC ». *(Relevé en passe 2.)*

**L'attente s'écrit PAR COUPLE, parce que le scope `read-only` en change la moitié.** Le gate de portée (`middleware/auth.rs:141-142`) vit dans `require_auth`, appliqué en `route_layer` sur le routeur **extérieur** (`lib.rs:869-877`) : il répond donc **avant** la couche neuve. Avec un PAT `read-only`, une méthode mutante n'atteint jamais `admin_routes`.

| Scope du PAT | Couples | Code attendu | Ce que le couple prouve |
|---|---|---|---|
| `read-write` | les **25** | `API_KEY_ADMIN_FORBIDDEN` | la couche |
| `read-only` | les **5** `get` | `API_KEY_ADMIN_FORBIDDEN` | la couche |
| `read-only` | les **20** autres | `API_KEY_READ_ONLY` | le gate de portée, **antérieur à cette story** |

⚠️ **La jambe `read-only` ajoutée en passe 1 était un test muet sur 20 couples sur 25** : elle prescrivait `403` partout, et la mutation « retirer la couche doit faire rougir » y est **insatisfaisable** — ces vingt-là rougissent ou verdissent sans que la couche existe. Écrite par couple, elle redevient une preuve : les cinq `get` sont le seul endroit où le scope `read-only` atteint la couche, et c'est justement là que « une route admin en lecture reste un vecteur de fuite » se démontre. *(Relevé en passe 2.)*

**Le cas « créateur Comptable » est asserté aussi** — un couple suffit —, avec le code arrêté en **D6**.

⚠️ **La complétude est le cœur de l'AC, et son mécanisme est arrêté ici.** `axum 0.8` **n'expose aucune énumération** — vérifié dans son source : `Router` n'offre que `has_routes() -> bool`. L'énumération dynamique est donc **impossible**, et le repli d'abord envisagé — comparer à « le nombre de routes du routeur » — était **circulaire** : ce nombre ne pouvait venir que d'une seconde constante entretenue à la main, c'est-à-dire du « quelqu'un doit se souvenir » que cette story existe pour éliminer.

**Le mécanisme retenu dérive le compte de la SOURCE, qui est la seule chose qu'un ajout de route modifie forcément** : le test lit `lib.rs` (`include_str!`) et compte, dans le bloc `admin_routes`, **les constructeurs de méthode** — `get(`, `post(`, `put(`, `delete(`, `patch(`, `head(`, `options(` —, et non les `.route(`. Le compte attendu est **25**. Une route ajoutée sans son test fait donc **rougir** le test, sans que personne n'ait à y penser.

⚠️ **Compter les `.route(` aurait reproduit le défaut dans le garde-fou lui-même.** **Cinq** enregistrements portent déjà plusieurs méthodes : ajouter une méthode à l'un d'eux ne change pas le nombre d'enregistrements, et le compteur resterait vert sur une route neuve non gardée. *(Le CRITICAL de passe 1 portait sur **d'où vient** le nombre ; sa remédiation n'avait pas traité **ce qu'il compte**. Relevé en passe 2 ; le « six » et le précédent historique de ce relevé sont réfutés en passe 3, cf. le § plus haut.)*

**Quatre exigences sur le mécanisme, chacune fermant un mode d'échec réel de `lib.rs`** :

1. **Bornes par marqueurs de commentaire**, jamais par numéros de ligne — ils se périmeraient au premier remaniement.
2. **Les deux marqueurs sont assertés présents, et une fois chacun.** L'identifiant `admin_routes` apparaît **hors** du bloc, à `lib.rs:356` (dans un commentaire de `comptable_routes`) et à `:871` (au `merge`) : un marqueur bâti sur ce mot seul attraperait la mauvaise borne. **Mesuré** : de l'ouverture du bloc à la fin du fichier, le même comptage rend **176** au lieu de 25 — sept fois trop, et un test qui n'énumère alors plus rien de précis. Un marqueur disparu, lui, donnerait un bloc vide — et un test muet.
3. **Les lignes de commentaire sont retirées avant comptage.** `lib.rs` porte des `.route(` en commentaire à `:939-940`, dans le bloc `NOTE` de fin de fichier ; et le bloc `admin_routes` lui-même contient des commentaires mentionnant `GET`. Un compteur naïf y prendrait du texte pour du code.

4. **Aucun `.route(`, `.nest(` ni `.route_service(` après le dernier `.route_layer(` du bloc** — cf. le trou d'`route_layer` décrit en **D1**. Sans cette quatrième assertion, une route peut être à la fois **non gardée** et **non comptée**.

⚠️ Une assertion **exacte** (`== 25`) et non « > 0 » : un plancher laisserait passer un bloc tronqué. *(La passe 2 a relevé que ce garde-fou restait un « sans doute » en question ouverte alors que `lib.rs` porte déjà les deux pièges ; il est arrêté ici. La quatrième exigence est venue en passe 3.)*

⚠️ **Ce que le mécanisme ne verra PAS, et il faut le savoir avant de s'y fier** : une sous-route montée par `.nest(` ou `.route_service(` **au-dessus** des couches est correctement **protégée** à l'exécution, mais n'ajoute aucun constructeur de méthode au texte borné — ses `get(`/`post(` vivent dans un autre fichier. Le compteur resterait à 25 et la liste de couples d'AC1 aussi, alors que la surface réelle a grandi. Ce n'est pas un trou de protection, c'est un **trou de preuve**. Conduite à tenir : pour `admin_routes`, les routes s'ajoutent par `.route()` direct — et si un montage par composition devient nécessaire, il vient **avec** l'ajustement du compteur. *(Relevé en passe 3.)*

**AC2 — Le cas nominal n'est pas cassé.**
Un PAT `read-write` créé par un **Comptable** conserve l'accès à tout ce qu'il pouvait atteindre : écritures, factures, contacts, produits.
*Preuve* : `api_keys_e2e.rs` reste vert **sans modification** — vérifié en passe 1 : aucun de ses tests n'exerce une route d'`admin_routes`, et tous ses contextes sont créés en rôle Comptable.

**AC3 — Un Admin devant son navigateur n'est pas affecté.**
La couche ne regarde que `api_key_id`, jamais le rôle.
*Preuve* : les suites E2E d'administration — utilisateurs, TVA, relances, modèles d'e-mail — restent vertes.
⚠️ **À une exception près, prévue et bornée** : les **trois** assertions de code d'erreur listées en D2 changent d'ancien vers nouveau code. Toute autre modification de test est un signal d'alerte, pas celle-là.

**AC4 — L'erreur dit ce qui se passe.**
`403` avec le code `API_KEY_ADMIN_FORBIDDEN` et un message traduit sur les quatre locales, distinct de celui de la gestion de clés.
*Preuve* :
- le bras de `match` d'`errors.rs` rend la chaîne `API_KEY_ADMIN_FORBIDDEN` (calqué sur `ApiKeyManagementForbidden`, `errors.rs:1104-1106`) ;
- les tests d'AC1 lisent `body["error"]["code"]` et assertent la chaîne **de bout en bout** — c'est là que le contrat se vérifie, pas dans `errors.rs` ;
- **un test dans `kesh-i18n` calqué sur `client_number_labels_are_translated_in_all_four_locales`** (`crates/kesh-i18n/src/loader.rs:261`) : la clé existe dans les quatre locales, **et** sa traduction diffère du français dans les trois autres ;
- **le message diffère de celui de la gestion de clés** — `format(locale, "error-api-key-admin-forbidden") != format(locale, "error-api-key-management-forbidden")`, au moins en `fr-CH`.

⚠️ **Sans cette dernière clause, le « distinct » d'AC4 n'est prouvé nulle part** — alors que c'est le motif entier de D2. T2 prescrit de **calquer** le bras existant ; un calque qui copie le code *et* le message satisferait les trois autres clauses à la lettre, et rendrait à l'appelant un message parlant de gestion de clés là où il est question d'administration. C'est précisément le mensonge que D2 refuse. *(Relevé en passe 3.)*

⚠️ **Aucun test d'appariement de locales n'existe dans `kesh-i18n` — la première rédaction invoquait un test imaginaire.** Vérifié : le crate n'a **pas de répertoire `tests/`**, et ses **22 tests unitaires** (`src/`) ne comportent aucun contrôle de parité globale. Les fichiers sont d'ailleurs déjà désappariés — **1266 clés en `fr-CH` contre 1209** dans les trois autres, soit les **57 clés de la KF #283**. **Ne pas chercher à résoudre cet écart ici** : il a son propre suivi.

⚠️ **Et l'assertion `!= fr` du précédent n'est pas décorative : sans elle, le test passe sur une clé absente.** Une clé manquante **retombe silencieusement sur le français** — c'est le comportement établi, et testé, par `format_missing_key_in_de_falls_back_to_fr` (`loader.rs:227`). Un test qui se contenterait de vérifier « la clé rend autre chose que son nom » serait donc vert sur trois locales vides. C'est pour cela que le calque du précédent est prescrit plutôt qu'un `grep -c` sur les fichiers : il ferme le mode d'échec, le `grep` ne le voit pas. *(Relevé en passe 2 : les Dev Notes imposent de vérifier au `grep -nF` toute affirmation d'absence ; celle-ci était une affirmation de **présence**, et elle était fausse. Le précédent, lui, a été trouvé au grep de propagation qui a suivi le patch.)*

**AC5 — Le chemin d'attaque est fermé, et le test le raconte.**
*Preuve* : un test qui rejoue la chaîne complète — PAT Admin → `POST /api/v1/users` → **`403 API_KEY_ADMIN_FORBIDDEN`**. C'est le test qui dit *pourquoi* la story existe ; son nom doit le porter. ⚠️ Le **code**, ici aussi : un `403` seul serait rendu par le RBAC sur un PAT de créateur Comptable, et ne prouverait pas que la chaîne d'escalade est coupée.

**AC6 — La documentation dit la nouvelle frontière, sur les quatre SUPPORTS qui la portent.**

Quatre supports, quatre chantiers de T6 — manuel, guide d'intégration, spécifications, CHANGELOG :

Manuel administrateur : ce qu'un jeton peut et ne peut pas faire, et que **l'administration en est exclue quel que soit le rôle du créateur**.
**Guide d'intégration `docs/api-external.md`** : c'est le document que lit celui qui écrit le client API, et le manuel y renvoie explicitement (`admin-manual.tex:1768`).
Les **quatre énoncés de DC6, sur trois spécifications**, selon la répartition arrêtée en **D4** — qui dit aussi pourquoi `17-2b` n'a rien à corriger.
Le CHANGELOG dit, dans les mots de l'utilisateur, **qu'un jeton créé par un Admin perd des accès qu'il avait** — sans quoi la première intégration qui tombe en `403` produit un ticket de support.

⚠️ **`docs/api-external.md` manquait entièrement des AC et des tâches de cette story, et c'est le plus grave des oublis documentaires** — le fichier existe et est à jour de l'ancien état des choses : la story fermerait la faille dans le code en laissant écrit ailleurs, à l'impératif, comment l'exploiter. Six sites, relevés au `grep` :

| Ligne | Ce qui devient faux |
|---|---|
| `:235` | **une instruction d'usage** : « pour changer un mot de passe via l'API, utilisez `PUT /api/v1/users/:id/reset-password` » — rendra `403` |
| `:222` | l'avertissement « une clé créée par un Administrateur hérite des pouvoirs d'Administrateur », avec renvoi à KF-036 |
| `:246` | la ligne « Auto-propagation des clés Administrateur » du tableau des limitations |
| `:229` et `:231` | toute la recommandation de `PUT /api/v1/users/:id` en remplacement complet — passage devenu inatteignable par PAT |
| `:205` et `:209` | le tableau des ressources donne les mutations `/vat-rates` comme disponibles, la note ¹ ne les réservant qu'au **rôle** Administrateur |
| `:73` | « la permission effective est l'intersection du rôle du créateur et de la portée » — désormais **incomplet** : les routes admin sont fermées quelle que soit cette intersection |

*Preuve*, et cette AC est celle qui en manquait le plus :
- **une chaîne discriminante, introduite par T6, présente exactement quatre fois** — `grep -rcF "n'atteint aucune route require_admin_role" _bmad-output/implementation-artifacts/17-2{,a,c}-*.md` rend `1`, `2`, `1`, aux trois fichiers de D4 ; et l'ancienne formulation « ne peut PAS lister/créer/révoquer » ne subsiste à **aucun** des quatre énoncés ;

> ⚠️ **La clause précédente était satisfaisable SANS faire le travail — c'était le défaut d'AC1 reproduit dans AC6.** Elle prescrivait `grep -rn "require_admin_role"` sur les trois specs. Or cette commande rend **déjà 7 résultats aujourd'hui**, avant tout amendement :
>
> ```sh
> grep -rn "require_admin_role" _bmad-output/implementation-artifacts/17-2{,a,c}-*.md | wc -l   # 7
> ```
>
> **Et aucun des sept n'est aux quatre lignes visées** : trois sont les entrées de limitation L3 que D4 interdit précisément de réécrire (`17-2a:114`, `17-2a:255`, `17-2c:35`), les quatre autres sont des Dev Notes hors sujet. La commande ne sait donc pas distinguer « les quatre énoncés ont été amendés » de « rien n'a été fait ». C'est mot pour mot le reproche qu'AC1 adresse à l'assertion `403` : *une preuve que plusieurs causes produisent ne prouve aucune d'elles*. *(Relevé en passe 3.)*
- **`grep -nF "KF-036" docs/manual/fr/admin-manual.tex docs/api-external.md` ne rend plus aucun avertissement présenté comme une limitation ouverte** — seules des mentions au passé, ou rien ;
- `admin-manual.tex:1765` (le bloc `keshwarning` « Moindre privilège ») ne décrit plus l'héritage des pouvoirs Admin comme un fait courant, et `:1756` élargit « la gestion des clés est impossible via l'API » à toute l'administration ;
- l'entrée CHANGELOG existe et mentionne la perte d'accès des jetons Admin existants.

⚠️ **La clause de preuve de la passe 1 mesurait un mot que le manuel n'emploie pas.** `grep -c "jeton" docs/manual/fr/admin-manual.tex` rend **3**, et **aucune** de ces trois occurrences ne concerne un PAT : deux portent sur le jeton de réinitialisation de mot de passe (`:1026`, `:1063`), la troisième sur les jetons de session dans un `.keshbackup` (`:1582`). Le manuel dit « **clé API (PAT)** ». Un critère « le compte a augmenté » était donc satisfaisable **sans toucher** au passage qui devient faux. *(Relevé en passe 2 — c'est la § « greper la valeur, pas la formulation » du `CLAUDE.md`, prise à l'envers : ici la formulation grepée n'existait même pas.)*

## Tasks / Subtasks

- [ ] **T1 — La couche** (AC1, AC3). Un `route_layer` sur `admin_routes`, à côté de `require_admin_role`. La couche précède le **handler** quel que soit son ordre relatif au RBAC — vérifié en passe 1 dans le source d'axum, `route_layer` enveloppe le service ; conséquence en D2. ⚠️ **Mais l'ordre entre les deux COUCHES, lui, est à arbitrer** : il décide du code rendu quand toutes deux refusent. La cible est fixée en **D6**, et c'est le test d'AC1 qui tranche — pas une lecture du source.
  - [ ] **Un commentaire d'avertissement aux deux `.route_layer(...)`** : tout ajout de route à `admin_routes` se fait **au-dessus** d'eux, sinon il échappe aux deux couches (cf. **D1** et le source d'axum). C'est un garde-fou de lecture ; l'assertion qui le tient est en T3.
- [ ] **T2 — Le code d'erreur** (AC4). Variante `AppError`, bras dans le `match` d'`errors.rs` (calque `ApiKeyManagementForbidden`, `:1104-1106`), clé i18n sur les **quatre** locales. ⚠️ La variante **s'ajoute** : `API_KEY_MANAGEMENT_FORBIDDEN` garde son consommateur (`comptable_routes`, cf. D5).
- [ ] **T3 — Le test de complétude** (AC1). Le point difficile, désormais tranché : `axum 0.8.9` n'expose **aucune** énumération (`has_routes()` seul). Le test dérive donc le compte de la **source** — `include_str!` sur `lib.rs`, entre deux marqueurs de commentaire bornant `admin_routes`.
  - [ ] Compter les **constructeurs de méthode** (`get(`, `post(`, `put(`, `delete(`, …), **pas** les `.route(` : 19 enregistrements portent **25** couples. Le mécanisme est vérifié sur la source réelle, ventilation comprise :
    ```sh
    sed -n '182,284p' crates/kesh-api/src/lib.rs | grep -v '^\s*//' \
      | grep -oE '\b(get|post|put|delete|patch|head|options)\(' | sort | uniq -c
    # 4 delete( · 5 get( · 6 post( · 10 put(  → 25
    ```
    ⚠️ Le `sed` de bornes n'est là que pour la vérification à la main — **le test borne par marqueurs**, cf. les deux points suivants.
  - [ ] Assertion **exacte** `== 25`, jamais un plancher.
  - [ ] Retirer les lignes de commentaire avant comptage (`lib.rs:939-940` porte des `.route(` commentés).
  - [ ] Asserter que les **deux** marqueurs existent, **une fois chacun** (`admin_routes` apparaît hors du bloc à `:356` et `:871`).
  - [ ] Asserter qu'**aucun** `.route(`, `.nest(` ou `.route_service(` ne suit le dernier `.route_layer(` du bloc — cf. **D1**.
  - [ ] Couvrir les 25 couples, et asserter **le code d'erreur**, pas le seul `403`.
  - [ ] Les deux scopes, avec l'attente **par couple** — cf. le tableau d'AC1 : `read-only` n'atteint la couche que sur les 5 `get`.
  - [ ] Un couple avec un PAT de créateur **Comptable**, code attendu selon **D6**.
- [ ] **T4 — Le test du chemin d'attaque** (AC5).
- [ ] **T5 — Non-régression** (AC2, AC3). `api_keys_e2e.rs` et les E2E d'administration restent verts. **Trois assertions changent, et trois seulement** — celles listées en D2. ⚠️ Ne pas toucher à `api_keys_e2e.rs:425`.
  - [ ] **La fixture frontend `admin-backup.api.test.ts:112`** porte `API_KEY_MANAGEMENT_FORBIDDEN` et devient un mensonge sur le contrat — **sans rougir**, puisqu'elle n'asserte qu'un rejet. La mettre à jour, et savoir qu'elle ne signalera rien si on l'oublie.
  - [ ] **L'assertion de source de D3** : les trois appels à `ensure_not_pat` d'`admin.rs` (`:37`, `:143`) et de `fiscal_years.rs` (`:317`) sont vérifiés présents par `include_str!`, dans le même fichier de test que T3.
- [ ] **T6 — Documentation** (AC6). Quatre chantiers, aucun optionnel :
  - [ ] **`docs/api-external.md`** — les six sites du tableau d'AC6, dont l'instruction d'usage de `:235`.
  - [ ] **Manuel admin FR** — `:1765` (le `keshwarning`) et `:1756`. ⚠️ **Régénérer le PDF** (`latexmk -xelatex` dans `docs/manual/fr/`) et le commiter : la convention du dépôt versionne les PDF.
  - [ ] **Les quatre énoncés de DC6** selon **D4** — `17-2:57`, `17-2a:35` et `:58`, `17-2c:34` — et la mention « fermé par 22-4 » sur les trois entrées de limitation (`17-2a:114`, `17-2a:255`, `17-2c:35`), **sans les réécrire**.
  - [ ] **CHANGELOG** — la perte d'accès des jetons Admin existants, dans les mots de l'utilisateur.

## Dev Notes

### Ce qui est déjà en place

`ensure_not_pat` — `crates/kesh-api/src/routes/api_keys.rs:95` — teste `current_user.api_key_id.is_some()` et rend `AppError::ApiKeyManagementForbidden`, mappé en `403 API_KEY_MANAGEMENT_FORBIDDEN` (bras à `errors.rs:1104`, chaîne à `:1106`).

**Ses six sites d'appel, et la moitié seulement est concernée** : `api_keys.rs:108`, `:121`, `:211` (les routes de gestion de clés, montées dans `comptable_routes` — **hors** périmètre, cf. D5) ; `admin.rs:37`, `admin.rs:143`, `fiscal_years.rs:317` (les trois d'`admin_routes` — c'est de celles-là que parle **D3**).

`CurrentUser.api_key_id: Option<i64>` — `middleware/auth.rs:44`, renseigné à `Some(...)` par le chemin PAT (`auth/api_key.rs:181`) et à `None` par le chemin JWT (`auth.rs:161`). **C'est le seul discriminant nécessaire**, et il est déjà fiable.

`require_admin_role` — `middleware/rbac.rs`, appliqué en `route_layer` sur `admin_routes` (`lib.rs:284`). C'est le voisin auprès duquel la nouvelle couche se pose.

**Le gate de portée est ANTÉRIEUR, et il change ce que le test peut prouver** — `middleware/auth.rs:141-142`, dans `require_auth`, appliqué en `route_layer` sur le routeur extérieur (`lib.rs:869-877`) : un PAT `read` sur une méthode non sûre (`is_safe_method` = `GET`/`HEAD`/`OPTIONS`) reçoit `403 API_KEY_READ_ONLY` **sans jamais atteindre `admin_routes`**. Cf. le tableau par couple d'AC1.

### Le piège de cette story, et il est du même genre que le défaut

⚠️ **Un test écrit à la main sur quelques routes reproduirait le défaut.** Seize routes sur dix-neuf ont été oubliées parce que rien ne rappelait de les traiter ; un test qui n'énumère pas oubliera de la même façon la dix-neuvième. C'est pour cela qu'AC1 exige soit l'énumération, soit une assertion de comptage qui **fail-loud** — le même raisonnement que le garde-fou **P6** du `CLAUDE.md` sur les migrations positionnelles.

### Ce qu'il ne faut pas « simplifier »

Ne pas retirer les trois `ensure_not_pat` existants au motif que la couche les couvre — cf. **D3**.

### Conventions de test

Mutations **jouées, pas raisonnées**. Pour AC1, la mutation est explicite : retirer la couche doit faire tomber le test — **sur les 25 couples de la jambe `read-write`, et sur les 5 `get` de la jambe `read-only`**. Sur les 20 couples mutants en `read-only`, elle est **insatisfaisable par construction** : le gate de portée répond avant, et c'est pourquoi AC1 y attend un autre code plutôt que le même `403`.

Les affirmations d'absence se vérifient au `grep -nF` avant d'être écrites — **et les affirmations de présence tout autant**. La première rédaction invoquait un test de parité i18n qui n'a jamais existé (cf. AC4) : une affirmation de présence fausse est plus coûteuse, car elle fait passer une clause de preuve pour couverte.

### i18n — la clé manquante ne rougit pas

⚠️ `kesh-i18n` n'a **aucun** test de parité de locales — pas de répertoire `tests/`, et aucun de ses **22 tests unitaires** ne contrôle la parité globale. Une clé absente d'une locale **retombe silencieusement sur le français** (`loader.rs:227`). Les quatre fichiers sont déjà désappariés : **1266 clés en `fr-CH`, 1209 dans les trois autres** — les 57 de la **KF #283**, hors périmètre de cette story.

**Le contrôle à écrire est un calque, pas une invention** : `client_number_labels_are_translated_in_all_four_locales` (`loader.rs:261`) fait exactement ce qu'AC4 demande, y compris l'assertion `!= fr` sans laquelle le repli silencieux rend le test vert à tort.

⚠️ **Ne pas se fier aux tableaux de gate qui annoncent « `cargo test -p kesh-i18n` 21/21 (parité FTL 4 locales) »** — `21-7:334` et `21-6c:305`. Le chiffre était juste à l'époque (22 aujourd'hui), mais la glose « parité » décrit une couverture qui n'existe pas. Ces lignes sont des **traces de gates exécutés** : on ne les réécrit pas, on ne s'y appuie pas.

### References

- Issue **#167** (KF-036) — dont les références de ligne sont périmées et la portée sous-estimée, cf. § *L'état réel du code*.
- Story **17-2a** — `17-2a-api-pat-backend.md`, limitation **L3** et décisions DC2, DC4, DC6.
- Issue **#100** — la demande d'origine de l'API à jeton.
- `CLAUDE.md` — § *Migration breaking policy* garde-fou **P6** pour l'analogie du test fail-loud.

## Questions ouvertes

**Les trois questions de la première rédaction sont closes** — deux par vérification dans le code, une par arbitrage :

- ~~L'énumération des routes~~ → **tranchée** : axum ne l'expose pas, le compte se dérive de la source (AC1, T3).
- ~~Le routeur comptable+~~ → **tranchée** et promue en **D5**.
- ~~Alerter sur les jetons existants~~ → **oui**, et c'est désormais une clause de preuve d'**AC6**.

**La quatrième, née de la correction elle-même, est close par la passe 2 :**

- ~~Le marqueur de bornage d'`admin_routes`~~ → **tranchée, et le « > 0 » qu'elle envisageait était insuffisant.** `lib.rs` porte **déjà les deux pièges** : des `.route(` en commentaire (`:939-940`) et l'identifiant `admin_routes` hors du bloc (`:356`, `:871`). Un marqueur mal posé donne 0 (test muet) ou 176 (test faux) — un plancher ne voit ni l'un ni l'autre. **Les quatre exigences** sont écrites en AC1 et déclinées en T3 : bornes par marqueurs et jamais par numéros de ligne, présence des deux marqueurs assertée, décommentage avant comptage, et rien d'ajouté après le dernier `.route_layer(`. L'assertion **exacte à 25** s'y ajoute comme garde-fou de comptage.

**Aucune question ouverte ne subsiste.**

## Change Log

**Ce journal porte la TRACE des passes, pas une seconde description de la story.** Ce que chaque correctif prescrit vit dans le corps — décisions, AC, tâches, Dev Notes —, et le corps est la seule source. Ici : les modèles, le trend, ce que chaque passe a appris **qui ne se déduit pas du corps**, et les réfutations. Rien qui se recompte ailleurs.

⚠️ **Cette règle est née d'un défaut mesuré, pas d'un souci d'élégance.** Aux passes 2 et 3, la majorité des findings les plus lourds portaient sur ce journal et non sur la story : un total contredisant sa ventilation (deux fois), un titre annonçant deux HIGH pour trois, un précédent historique inventé, un numéro de ligne dérivé. Le journal avait grossi jusqu'à 38 % du document et redisait le corps en divergeant de lui. **Un compte rendu qui duplique devient un compte rendu qui ment** — élagué le 2026-08-13 sur arbitrage de Guy, en réponse au trend de la passe 3.

### Trend

| Passe | Modèles | CRIT | HIGH | MED | LOW |
|---|---|---|---|---|---|
| 1 | Sonnet ×3 | **1** | 3 | 6 | 3 |
| 2 | Haiku (aveugle) + Opus ×2 | 0 | 3 | 7 | 3 |
| 3 | Sonnet ×3 | 0 | **4** | 6 | 2 |

Décomptes **dédupliqués** inter-lentilles. La sévérité maximale décroît une fois (`CRITICAL → HIGH`) puis **stagne** ; le compte de HIGH monte de 3 à 4 en passe 3.

**Arbitrage du 2026-08-13 (Guy)** : le critère de découpage préventif du `CLAUDE.md` est atteint dans sa lettre, mais trois des quatre HIGH de la passe 3 portent sur les comptes rendus et aucun sur le mécanisme de sécurité — dont la passe 3 a confirmé la description exacte sur vingt-et-une vérifications. **Décision : ne pas découper ; élaguer ce journal et poursuivre en passe 4.**

### Passe 1 — 2026-08-12

**Le défaut central était une contradiction entre deux décisions de la story.** D2 crée un code d'erreur distinct, D3 conserve les trois `ensure_not_pat` : or `route_layer` enveloppe le service, donc la couche répond avant le handler et les trois tests existants reçoivent le **nouveau** code. La première rédaction affirmait l'inverse et avertissait qu'une modification de test signalerait « une couche posée trop haut » — **un avertissement qui aurait fait remettre en cause la bonne décision.**

**Le CRITICAL portait sur une circularité, et il avait raison** : AC1 exigeait d'énumérer les routes du routeur, avec pour repli une assertion contre « le nombre de routes du routeur ». `axum` n'expose aucune énumération ; ce nombre n'aurait pu venir que d'une seconde constante à la main — exactement le « quelqu'un doit se souvenir » que la story existe pour éliminer.

Trois autres corrections de fond : « route » n'était pas défini (chemins contre couples méthode-chemin) ; seul le scope `read-write` était prouvé ; et D4 amendait la mauvaise cible.

**Ce que les lentilles ont confirmé plutôt que réfuté** : le décompte des routes gardées, les trois corrections apportées à l'issue #167, le fait que `require_admin_role` n'est appliqué nulle part ailleurs, et l'absence de branche résiduelle du chemin d'attaque — le changement de mot de passe self-service, seul candidat, exige la vérification Argon2.

### Passe 2 — 2026-08-13

Collecte interrompue avant remédiation ; les correctifs ont été appliqués à la reprise, le même jour.

**Verdict sur la passe 1 : elle n'a pas menti, mais deux de ses corrections ont figé un périmètre incomplet en lui donnant l'apparence de l'exhaustivité.** C'est le mode d'échec propre à ce genre de document : **une clause qui énumère se lit comme close.**

Les trois HIGH, dans le corps désormais : le compteur mesurait les enregistrements quand le critère exige les couples (§ *Dix-neuf enregistrements*, AC1, T3) ; la jambe `read-only` était un test muet sur vingt couples sur vingt-cinq, le gate de portée répondant avant la couche (tableau par couple d'AC1) ; `docs/api-external.md` manquait des AC et des tâches (AC6, T6). Les sept MEDIUM ont produit D6, l'amendement de D3 et de D4, et les exigences du mécanisme de comptage.

**Deux enseignements que le corps ne porte pas :**

- **Un remède proposé peut être moins bon que ce que le dépôt porte déjà.** Le relevé prescrivait un `grep -c` pour prouver l'i18n ; `kesh-i18n` a `client_number_labels_are_translated_in_all_four_locales`, qui asserte que la traduction diffère du français et ferme donc le repli silencieux. Un `grep -c` aurait été un test muet de plus, dans la story dont le sujet est le test muet. **C'est le grep de propagation post-patch qui l'a trouvé, pas la collecte.**
- **`21-7` et `21-6c` annoncent « parité FTL 4 locales » pour une suite qui ne la contrôle pas.** Laissé tel quel — ce sont des traces de gates réellement exécutés — et signalé en Dev Notes pour que personne ne s'y appuie.

**Réfuté** : le LOW affirmant que le bras de `match` d'`errors.rs` était en `:1105` — il est en `:1104`, comme la story l'écrivait.

### Passe 3 — 2026-08-13

**Trois des quatre HIGH portent sur les comptes rendus des passes précédentes, et le quatrième sur une clause de preuve.** Aucun sur le mécanisme.

- **Un précédent historique était inventé.** La passe 2 affirmait que `/admin/email-templates/{…}` serait « passée d'une à trois méthodes entre les stories 20-1 et 20-2 » — le fait qui justifiait le mécanisme de comptage. `git log -S` établit qu'elle est **née** avec ses trois méthodes (`8bca0a1e`). Et « six enregistrements multi-méthodes » en compte **cinq**. Les deux avaient été recopiés dans le corps **sous un commit déclarant que chaque fait du relevé avait été revérifié depuis la source**. *L'argument de conception tient sans eux ; il était donné comme un fait advenu.*
- **La clause de preuve d'AC6 était satisfaisable sans faire le travail** : le `grep -rn "require_admin_role"` qu'elle prescrivait rend **sept** résultats avant tout amendement, aucun aux lignes visées, et trois sont les entrées de limitation que D4 interdit de réécrire. C'est le reproche qu'AC1 adresse à l'assertion `403`, reproduit dans AC6.
- **« Quatre documents » pour trois documents et quatre énoncés** : T6 avait été corrigée, D4 et AC6 non — un correctif non propagé. Et `17-2b` mentionne DC6 sans avoir jamais été examiné ; vérifié depuis, ses deux phrases restent vraies.
- **« Les dix findings » pour une ventilation qui en compte treize.**

**Le MEDIUM le plus utile vient du source d'axum** : `route_layer` n'enveloppe que les routes **déjà présentes à l'appel**. Une route chaînée après les couches échappe **aux deux**, compile et ne panique pas. Le trou préexiste à la story, mais sa promesse de complétude l'aurait masqué → avertissement en T1, quatrième assertion en T3. Les cinq autres : `.nest(`/`.route_service(` échappent au compteur sans échapper à la protection ; AC4 ne prouvait pas que le message diffère de celui de la gestion de clés ; le passage de cinq à six sites documentaires n'était pas réconcilié ; deux listes « trois exigences » divergentes ; un titre annonçant deux HIGH pour trois.

**Vérifié sans défaut, et à ne pas rejouer** : l'ensemble des références de ligne du corps, une par une, sauf le renvoi vers le guide d'intégration — `:1768`, et non `:1770` comme le disait la passe 2 · tous les décomptes du corps · le `405` d'une méthode non enregistrée est engendré **après** les deux couches, comme `HEAD` implicite sur une route `get` · `CurrentUser` est disponible quand la couche s'exécute · le rôle Consultation ne peut pas créer de PAT · `comptable_routes` ne porte aucun vecteur de containment · `/onboarding/coordinates` reste fermé par sa garde d'étape · **l'ordre d'empilement voulu par D6 s'obtient naturellement** en posant la couche après le RBAC — et la spec a raison de laisser le test l'arbitrer plutôt que de s'y fier.

### Ce qui n'a PAS été fait, aux trois passes

**Aucun gate n'a été exécuté** : la story n'a pas de code, seul son fichier de spécification a changé. Les tableaux de gate restent vides jusqu'à `dev-story`.
