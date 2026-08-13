# Story 22.4a : La couche qui interdit un PAT sur les routes d'administration

## Status

backlog

⚠️ **Née du découpage de la story 22-4**, le 2026-08-13, après quatre passes de `bmad-create-story validate`. Elle en reçoit **le mécanisme** ; la story **22-4b** reçoit la frontière documentaire. Le motif du découpage et la conduite de merge sont au § *Découpage*.

## Story

**As a** administrateur d'une instance Kesh exposant une API à jeton,
**I want** qu'un jeton fuité ne puisse en aucun cas créer un utilisateur, réinitialiser un mot de passe ou modifier la configuration de la société,
**so that** **révoquer le jeton suffise à arrêter l'incident** — ce qui n'est pas le cas aujourd'hui.

Ferme **#167** (KF-036) *dans le code*. Story de l'**Epic 22 « Technical Debt Closure »**, et la seule défaillance connue ouverte qui touche à la **sécurité**.

## Découpage — pourquoi deux stories, et pourquoi une seule PR

**Le découpage est un découpage de SPÉCIFICATION, pas de livraison.**

La story 22-4 a subi quatre passes de `validate`. Les deux dernières ont **stagné à HIGH** avec un volume de MEDIUM en hausse — le critère de non-convergence de la § *Règle de splitting préventif* du `CLAUDE.md`. Le diagnostic est net : **c'est la frontière documentaire qui ne converge pas.** Chaque lentille de chaque passe y trouvait un site de plus — le guide d'intégration, puis son tableau des codes d'erreur, puis le manuel, puis sept énoncés d'autres specs. Le mécanisme, lui, a été confirmé exact par toutes les lentilles à chaque passe.

Deux mental-models distincts, donc deux stories : ici le mécanisme, revu en passes adversariales ; en **22-4b** le rollout documentaire, revu **au file-by-file** comme la règle le prescrit pour une sous-story de rollout.

⚠️ **Mais les deux se mergent dans UNE SEULE PR**, et ce n'est pas négociable. La § *Synchroniser TOUTES les docs* du `CLAUDE.md` impose qu'un changement de doc vive dans la PR du code qui le motive. Livrer 22-4a seule publierait un logiciel dont la documentation **enseigne à exploiter** un trou désormais fermé — `api-external.md:235` dit à l'impératif d'utiliser une route qui rendra `403`. **La PR porte `closes #167` et ne part qu'avec les deux moitiés.**

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

### Trois nombres, et il faut les tenir séparés

Le « dix-neuf » ci-dessus compte des **enregistrements `.route(…)`**. Un routeur axum route par **méthode**, et une méthode enregistrée n'est pas toujours une méthode servie. Les trois nombres, recomptés sur `lib.rs:182-284` :

| Nombre | Valeur | Ce que c'est |
|---|---|---|
| Enregistrements | **19** | les appels `.route(…)` |
| **Constructeurs de méthode** | **25** | `4 delete` · `5 get` · `6 post` · `10 put` — **c'est le compteur de T3** |
| Couples servis | **30** | les 25, plus les **5 `HEAD`** qu'axum sert via les handlers `get` |

**Cinq** des dix-neuf enregistrements portent deux ou trois méthodes : `/users`, `/users/{id}`, `/vat-rates/{id}`, `/dunning-levels/{id}` et `/admin/email-templates/{template_type}/{language}` (trois à lui seul). Ajouter une méthode à l'un d'eux ne change **pas** le nombre d'enregistrements : un compteur calibré sur les `.route(` n'y verrait rien — c'est le mode d'échec que cette story existe pour fermer, reproduit dans son propre garde-fou.

⚠️ **Les cinq `HEAD` sont servis, protégés, et n'étaient comptés nulle part.** Quand aucun `head` n'est enregistré, axum sert `HEAD` par le handler `get` (`method_routing.rs`, `call!(req, HEAD, get)`) et annonce `Allow: GET,HEAD`. `HEAD /api/v1/admin/full-export` avec un PAT `read-only` passe donc le gate de portée (`is_safe_method` accepte `HEAD`) et **atteint la couche**. Ce sont cinq couples de démonstration gratuits pour la jambe `read-only` d'AC1. *(Relevé en passe 4 : la story disait « 25 couples » sous une définition « couple (méthode, chemin) » qui en donne 30.)*

> ⚠️ **Un précédent invoqué par la passe 2 était inventé, et il est réfuté.** Elle affirmait « six enregistrements multi-méthodes » — il y en a cinq — et que `/admin/email-templates/{…}` serait « passé d'une à trois méthodes entre les stories 20-1 et 20-2 ». La route est **née** avec ses trois méthodes :
>
> ```sh
> git log --all --oneline -S'email-templates/{template_type}/{language}' -- crates/kesh-api/src/lib.rs
> #   8bca0a1e feat(story-20-1): implémente le socle templates d'e-mail (backend)
> git show 8bca0a1e^:crates/kesh-api/src/lib.rs | grep -c email-templates   # 0 — pas encore là
> ```
>
> L'argument de conception n'en dépend pas : cinq enregistrements multi-méthodes suffisent. Mais il était donné comme un fait advenu. *(Réfuté en passe 3.)*

## Décisions

**D1 — Le blocage se pose en COUCHE sur le routeur, pas dans les handlers.**

L'issue proposait les deux options. **L'état du code tranche** : l'approche par handler a été appliquée trois fois et **oubliée seize**. Ce n'est pas un défaut de diligence, c'est la propriété d'un dispositif qui doit être rappelé à chaque ajout de route — et rien ne le rappelle. Un `route_layer` sur `admin_routes`, à côté de `require_admin_role` qui vit déjà là, se pose **une fois** et couvre toute route ajoutée ensuite **dans le bloc**.

> ⚠️ **« Par construction » est trop fort, et la nuance est un vrai trou : `route_layer` n'enveloppe que les routes DÉJÀ présentes à l'appel.** Lu dans le source (`axum-0.8.9/src/routing/path_router.rs:311-341`) :
>
> ```rust
> let routes = self.routes.into_iter()
>     .map(|(id, endpoint)| (id, endpoint.layer(layer.clone())))
>     .collect();
> ```
>
> Une route ajoutée **après** les couches — `admin_routes = admin_routes.route(...)` écrit plus loin, ou un remaniement qui remonte les couches « pour la lisibilité » — **compile, ne panique pas** (le `panic!` ne vise que le routeur vide) et échappe **aux deux** couches : ni RBAC, ni anti-PAT. Elle ne garde que `require_auth`, c'est-à-dire l'authentification sans l'autorisation.
>
> ⚠️ **Et le danger n'est pas dans le bloc, il est APRÈS lui.** Entre la fin d'`admin_routes` et son `.merge(` de `lib.rs:871` s'étendent près de six cents lignes, et `build_router` **utilise déjà trois fois** l'idiome de réaffectation sur `main_router` (`:897`, `:908`, `:919`), dont une sous un `if` de configuration. La forme la plus probable d'un futur ajout admin optionnel est donc exactement celle qui échappe à tout :
>
> ```rust
> if state.config.some_feature {
>     admin_routes = admin_routes.route("/api/v1/admin/purge", post(handler));
> }
> ```
>
> **Ce trou préexiste à cette story** — il vaut déjà pour `require_admin_role` seul — mais la promesse de complétude le masquerait. Il est fermé par le commentaire d'avertissement de T1 et les assertions de T3. *(Relevé en passe 3, puis élargi hors du bloc en passe 4.)*

**D2 — Un code d'erreur distinct, ET trois tests existants à mettre à jour.** `AppError::ApiKeyManagementForbidden` rend aujourd'hui `403 API_KEY_MANAGEMENT_FORBIDDEN`. Le réutiliser pour « vous ne pouvez pas rouvrir un exercice avec un jeton » **mentirait à l'appelant** : son message parle de gestion de clés. Une variante dédiée — `API_KEY_ADMIN_FORBIDDEN` — avec son message propre sur les quatre locales.

> ⚠️ **Cette décision a une conséquence qui contredisait D3.**
>
> `route_layer` enveloppe le service : la couche répond **avant** le handler, quel que soit son ordre par rapport à `require_admin_role`. Les trois routes déjà gardées répondront donc le **nouveau** code :
>
> | Test | Ligne | Assertion actuelle |
> |---|---|---|
> | `full_export_via_pat_returns_403` | `admin_full_export_e2e.rs:409` | `API_KEY_MANAGEMENT_FORBIDDEN` |
> | `full_import_via_pat_returns_403` | `admin_full_import_e2e.rs:414` | `API_KEY_MANAGEMENT_FORBIDDEN` |
> | `reopen_via_pat_returns_403` | `fiscal_years_e2e.rs:1638` | `API_KEY_MANAGEMENT_FORBIDDEN` |
>
> **Ces trois assertions DOIVENT changer, et c'est la seule modification de test autorisée par cette story.** La première rédaction disait l'inverse et avertissait qu'une modification signalerait « une couche posée trop haut » — **cet avertissement égarait** : la cause n'est pas le placement, correct, mais l'interaction entre D2 et D3. *(Réfuté en passe 1.)*
>
> ⚠️ Ne PAS toucher à `api_keys_e2e.rs:425` : cette route-là est bien de la gestion de clés, son code reste le bon.
>
> ⚠️ **Sept énoncés de spécification décrivent aussi l'ancien code pour ces trois routes** (`17-3:49` et `:158`, `17-3a:27` et `:54`, `17-3c:28` et `:103`, `14-2:402`). Ils sont traités par la **story 22-4b** — pas ici, mais pas oubliés.

**D3 — Les trois `ensure_not_pat` d'`admin_routes` restent, et leur câblage devient prouvé par la SOURCE.** La couche les rend redondants, mais `full-export`, `full-import` et la réouverture d'exercice sont des opérations dont l'interdiction aux jetons est **intrinsèque**, indépendamment du routeur où elles vivent. Un déplacement de route ne doit pas les découvrir.

> ⚠️ **Après T5, plus aucun test ne prouve que ces trois gardes sont câblées.** Leurs tests assertaient le code du handler ; basculés sur le nouveau code, ils prouvent désormais **la couche**. Les retirer toutes les trois ne ferait rougir **aucun** test.
>
> **Un test unitaire sur `ensure_not_pat` n'y répond pas** : la fonction reste couverte par ses trois **autres** sites d'appel, ceux des routes de gestion de clés (`api_keys.rs:108`, `:121`, `:211`), qui vivent dans `comptable_routes`. Ce qui perd sa couverture, c'est le **câblage** des trois sites d'`admin_routes` — `admin.rs:37`, `admin.rs:143`, `fiscal_years.rs:317`.
>
> **Mécanisme retenu** : une assertion **de source**, dans le fichier de test de T3 et par le même `include_str!`. *Repli si impraticable* : écrire noir sur blanc que ces trois gardes sont une défense en profondeur **sans couverture de test** — jamais laisser la revendication de D3 sans l'un des deux. *(Relevé en passe 2.)*

**D5 — Le routeur `comptable_routes` reste HORS périmètre, et c'est une décision.** Vérifié : il ne porte aucune route capable de créer un utilisateur, de réinitialiser un mot de passe ou de changer la configuration de la société. Et le fermer casserait le cas d'usage nominal du PAT (DC4), qui est d'écrire des écritures et des factures.

⚠️ **Conséquence pour T2** : les routes de gestion des clés (`lib.rs:575`, `:579`) vivent dans `comptable_routes`. Le code `API_KEY_MANAGEMENT_FORBIDDEN` garde donc un consommateur vivant et testé (`api_keys_e2e.rs:425`) : la nouvelle variante **s'ajoute**, elle ne remplace ni ne renomme rien.

⚠️ **`authenticated_routes` porte les mutations d'onboarding** — `/onboarding/coordinates`, `/org-type`, `/bank-account`, `/seed-demo`, `/reset`, `/finalize` —, qui **changent bien la configuration de la société** et n'exercent aucun contrôle de rôle. Elles sont fermées par leur **garde d'étape** (`set_coordinates` exige `step_completed == 5 && !is_demo`) : sur une instance finalisée, la surface est nulle. Hors périmètre, mais désormais **vérifié et écrit** — la première rédaction de D5 concluait sur `comptable_routes` alors que les routes de configuration société vivent dans le troisième routeur, jamais examiné. *(Relevé en passe 4.)*

**D6 — Quand les DEUX couches refusent, c'est la couche anti-PAT qui répond.**

Un PAT créé par un **Comptable**, présenté sur une route d'`admin_routes` : `require_admin_role` le refuse (rôle insuffisant) **et** la nouvelle couche le refuse (c'est un PAT). L'ordre des deux `route_layer` détermine le code observable.

**Décision : `API_KEY_ADMIN_FORBIDDEN`, sans exception.** Un code unique pour « un PAT a touché une route admin », quel que soit le rôle du créateur — la seule forme qui rende AC1 assertable **par couple** sans se ramifier sur le rôle, et elle ne divulgue rien : le porteur du jeton sait déjà qu'il porte un jeton.

⚠️ **T1 « l'ordre n'a pas à être arbitré » était vrai pour le handler et faux pour le code de réponse.** La vérification de passe 1 établissait que `route_layer` enveloppe le service — donc que la couche précède le **handler**. Elle ne disait rien de l'ordre entre **deux couches**. *(Relevé en passe 2.)*

**Le test est l'arbitre.** AC1 asserte le code sur le cas « créateur Comptable » ; si le montage rend l'autre code, le test rougit et T1 déplace la couche. *(La passe 4 a lu le source et conclut que poser l'anti-PAT **après** `require_admin_role` donne naturellement la précédence voulue — chaque `.layer()` enveloppant le précédent. C'est une indication, pas la preuve : le test reste l'arbitre.)*

## Acceptance Criteria

**AC1 — Aucune route admin n'est atteignable par un PAT, quel que soit son scope.**

*Preuve* : un test qui couvre **les 25 constructeurs de méthode** d'`admin_routes` — plus les **5 `HEAD`** servis par les handlers `get` —, avec un PAT dont le créateur est Admin, **aux deux scopes**, et qui asserte **le code d'erreur** et non le seul statut.

⚠️ **Asserter `403` ne prouve rien ici : TROIS gardes distinctes le produisent** — le RBAC, le gate de portée `read-only`, et la couche neuve. Un test conforme à la lettre « rend 403 » passerait **sans que la couche soit jamais atteinte**. La leçon est déjà écrite dans le dépôt, à `fiscal_years_e2e.rs:1634-1637` : « asserter le CODE prouve que le 403 vient bien de `ensure_not_pat` et non du middleware RBAC ». *(Relevé en passe 2.)*

**L'attente s'écrit PAR COUPLE, parce que le scope `read-only` en change la moitié.** Le gate de portée (`middleware/auth.rs:141-142`) vit dans `require_auth`, appliqué en `route_layer` sur le routeur **extérieur** (`lib.rs:869-877`) : il répond **avant** la couche neuve.

| Scope du PAT | Couples | Code attendu | Ce que le couple prouve |
|---|---|---|---|
| `read-write` | les **25** | `API_KEY_ADMIN_FORBIDDEN` | la couche |
| `read-only` | les **5** `get` + les **5** `HEAD` | `API_KEY_ADMIN_FORBIDDEN` | la couche |
| `read-only` | les **20** mutants | `API_KEY_READ_ONLY` | le gate de portée, **antérieur à cette story** |

⚠️ **La jambe `read-only` de la première rédaction était un test muet sur 20 couples sur 25** : elle prescrivait `403` partout, et la mutation « retirer la couche doit faire rougir » y est **insatisfaisable**. Écrite par couple, elle redevient une preuve. *(Relevé en passe 2.)*

**Le cas « créateur Comptable » est asserté aussi** — un couple suffit —, avec le code arrêté en **D6**.

### Le mécanisme de complétude, et ses trois aveuglements fermés

⚠️ `axum 0.8.9` **n'expose aucune énumération** — `Router` n'offre que `has_routes() -> bool`. L'énumération dynamique est **impossible**, et le repli d'abord envisagé — comparer à « le nombre de routes du routeur » — était **circulaire** : ce nombre ne pouvait venir que d'une seconde constante entretenue à la main, c'est-à-dire du « quelqu'un doit se souvenir » que cette story existe pour éliminer.

**Le mécanisme dérive le compte de la SOURCE** : le test lit `lib.rs` (`include_str!`), borne le bloc `admin_routes` par des marqueurs de commentaire, retire les commentaires, compte les constructeurs de méthode, et asserte **exactement 25**.

**Cinq exigences, chacune fermant un mode d'échec réel** :

1. **Bornes par marqueurs de commentaire**, jamais par numéros de ligne — ils se périmeraient au premier remaniement.
2. **Les deux marqueurs sont assertés présents, une fois chacun**, et le bloc borné contient **exactement deux** `.route_layer(`. L'identifiant `admin_routes` apparaît **hors** du bloc à `lib.rs:356` et `:871` : un marqueur bâti sur ce mot seul attraperait la mauvaise borne — le même comptage de l'ouverture du bloc à la fin du fichier rend **176**. Et sans l'assertion des deux couches, un marqueur de fin posé juste après le dernier `.route(` — placement naturel — rendrait l'exigence 4 **vide**.
3. **Les commentaires sont retirés avant comptage**, en tronquant chaque ligne à son premier `//` et non en écartant les lignes entières. `lib.rs:939-940` porte des `.route(` commentés, atteignables si un marqueur disparaît ; et un commentaire de **fin de ligne** (`… // post( viendra en 20-4`) ferait rougir à tort un filtre par ligne entière.
4. **Rien ne s'ajoute après les couches, ni DANS le bloc ni APRÈS lui.** Dans le bloc : aucun `.route(`, `.route_service(`, `.nest(`, `.nest_service(`, `.merge(`, `.fallback(` ni `.method_not_allowed_fallback(` après le **premier** `.route_layer(`. Hors du bloc : sur le fichier entier, `admin_routes` n'apparaît qu'à ses **trois** sites connus, et **aucune** ligne ne matche `admin_routes\s*=` hors de sa déclaration.
5. **Aucun constructeur hors liste.** Le compteur reconnaît sept constructeurs ; `axum` en exporte **vingt-deux**. Asserter le **complément** — aucun jeton `\b(any|on|trace|connect)\(` ni `\b\w+_service\(` dans le bloc — ferme la classe entière au lieu d'énumérer.

⚠️ **Le cinquième aveuglement était le plus grave, et il annulait la promesse de la story.** Une route écrite `.route("/api/v1/admin/purge", any(handler))` enregistre **neuf** méthodes, est correctement **protégée**, et **laisse le compteur à 25** : la protection tient, le **rappel** ne tient pas — or c'est le rappel qui est l'objet de la story. Idem avec `on(MethodFilter::POST, h)` ou `post_service(...)`. *(Relevé en passe 4.)*

⚠️ **Ce que le mécanisme ne verra PAS, même ainsi** : une sous-route montée par `.nest(` ou `.route_service(` **au-dessus** des couches est correctement protégée mais n'ajoute aucun constructeur au texte borné — ses `get(`/`post(` vivent ailleurs. L'exigence 4 l'interdit dans le bloc ; si un tel montage devenait nécessaire, il vient **avec** l'ajustement du compteur.

**AC2 — Le cas nominal n'est pas cassé.**
Un PAT `read-write` créé par un **Comptable** conserve l'accès à tout ce qu'il pouvait atteindre : écritures, factures, contacts, produits.
*Preuve* : `api_keys_e2e.rs` reste vert **sans modification** — ses onze contextes sont créés en rôle Comptable et aucun de ses tests n'exerce une route d'`admin_routes`.

**AC3 — Un Admin devant son navigateur n'est pas affecté.**
La couche ne regarde que `api_key_id`, jamais le rôle.
*Preuve* : les suites E2E d'administration — utilisateurs, TVA, relances, modèles d'e-mail — restent vertes.
⚠️ **À une exception près, prévue et bornée** : les **trois** assertions de code d'erreur listées en D2. Toute autre modification de test est un signal d'alerte, pas celle-là.

**AC4 — L'erreur dit ce qui se passe.**
`403` avec le code `API_KEY_ADMIN_FORBIDDEN` et un message traduit sur les quatre locales, **distinct** de celui de la gestion de clés.
*Preuve* :
- le bras de `match` d'`errors.rs` rend la chaîne `API_KEY_ADMIN_FORBIDDEN` (calque `ApiKeyManagementForbidden`, `errors.rs:1104-1106`) ;
- les tests d'AC1 lisent `body["error"]["code"]` et assertent la chaîne **de bout en bout** ;
- un test dans **`crates/kesh-i18n/src/loader.rs`**, calqué sur `client_number_labels_are_translated_in_all_four_locales` (`:261`) : la clé existe dans les quatre locales **et** sa traduction diffère du français dans les trois autres ;
- **le message diffère de celui de la gestion de clés** — `format(locale, "error-api-key-admin-forbidden") != format(locale, "error-api-key-management-forbidden")`, au moins en `fr-CH`.

⚠️ **Sans cette dernière clause, le « distinct » n'est prouvé nulle part** — alors que c'est le motif entier de D2. T2 prescrit de **calquer** le bras existant ; un calque qui copie le code *et* le message satisferait les trois autres clauses à la lettre. *(Relevé en passe 3.)*

⚠️ **Aucun test d'appariement de locales n'existe dans `kesh-i18n`** : pas de répertoire `tests/`, et aucun de ses 22 tests unitaires ne contrôle la parité. Les fichiers sont déjà désappariés — **1266** clés en `fr-CH` contre **1209** dans les trois autres, les 57 de la **KF #283**, à ne pas traiter ici. **L'assertion `!= fr` du calque n'est donc pas décorative** : une clé absente retombe sur le français, et un test qui vérifierait seulement « la clé rend autre chose que son nom » serait vert sur trois locales vides.

⚠️ Ce repli est établi par `I18nBundle::format`, branche *Fallback vers FR-CH* — **et non** par le test `format_missing_key_in_de_falls_back_to_fr` (`loader.rs:227`), qui malgré son nom asserte l'autre branche : une clé absente **partout** rend son propre nom. *(La story 22-4 avait déjà été prise à invoquer un test imaginaire ; ici le test existe mais ne prouve pas ce qu'on lui faisait dire. Relevé en passe 4.)*

**AC5 — Le chemin d'attaque est fermé, et le test le raconte.**
*Preuve* : un test qui rejoue la chaîne complète — PAT Admin → `POST /api/v1/users` → **`403 API_KEY_ADMIN_FORBIDDEN`**. C'est le test qui dit *pourquoi* la story existe ; son nom doit le porter. ⚠️ Le **code**, ici aussi : un `403` seul serait rendu par le RBAC sur un PAT de créateur Comptable.

## Tasks / Subtasks

- [ ] **T1 — La couche** (AC1, AC3). Un `route_layer` sur `admin_routes`, à côté de `require_admin_role`. L'ordre entre les deux **couches** est à arbitrer : il décide du code rendu quand toutes deux refusent. Cible en **D6**, tranchée par le test d'AC1.
  - [ ] **Un commentaire d'avertissement aux deux `.route_layer(...)`** : tout ajout de route à `admin_routes` se fait **au-dessus** d'eux — et jamais par réaffectation plus loin dans le fichier —, sinon il échappe aux deux couches (cf. **D1**).
- [ ] **T2 — Le code d'erreur** (AC4). Variante `AppError`, bras dans le `match` d'`errors.rs` (calque `ApiKeyManagementForbidden`, `:1104-1106`), clé i18n sur les **quatre** locales, **message distinct**. ⚠️ La variante **s'ajoute** : `API_KEY_MANAGEMENT_FORBIDDEN` garde son consommateur (cf. D5).
- [ ] **T3 — Le test de complétude** (AC1). `include_str!` sur `lib.rs`, bornes par marqueurs, et les **cinq exigences** d'AC1 :
  - [ ] compter les constructeurs de méthode, assertion **exacte `== 25`** — jamais un plancher ;
  - [ ] tronquer chaque ligne à son premier `//` avant comptage ;
  - [ ] asserter les deux marqueurs, une fois chacun, et **exactement deux** `.route_layer(` dans le bloc ;
  - [ ] asserter qu'aucun ajout de route ne suit le **premier** `.route_layer(` — liste complète en AC1 exigence 4 ;
  - [ ] asserter, **sur le fichier entier**, que `admin_routes` n'apparaît qu'à ses trois sites et qu'aucune réaffectation `admin_routes =` n'existe hors déclaration ;
  - [ ] asserter le **complément** de constructeurs : ni `any(`, `on(`, `trace(`, `connect(`, ni `*_service(` dans le bloc ;
  - [ ] couvrir les 25 constructeurs + les 5 `HEAD`, asserter **le code**, attente **par couple** selon le tableau d'AC1 ;
  - [ ] un couple avec un PAT de créateur **Comptable**, code selon **D6**.
  - Vérification à la main du décompte, ventilation comprise :
    ```sh
    sed -n '182,284p' crates/kesh-api/src/lib.rs | grep -v '^\s*//' \
      | grep -oE '\b(get|post|put|delete|patch|head|options)\(' | sort | uniq -c
    # 4 delete( · 5 get( · 6 post( · 10 put(  → 25
    ```
- [ ] **T4 — Le test du chemin d'attaque** (AC5).
- [ ] **T5 — Non-régression** (AC2, AC3). `api_keys_e2e.rs` et les E2E d'administration restent verts. **Trois assertions changent, et trois seulement.** ⚠️ Ne pas toucher à `api_keys_e2e.rs:425`.
  - [ ] **La fixture frontend `frontend/src/lib/features/admin-backup/admin-backup.api.test.ts:112`** porte `API_KEY_MANAGEMENT_FORBIDDEN` et devient un mensonge sur le contrat — **sans rougir**, puisqu'elle n'asserte qu'un rejet.
  - [ ] **L'assertion de source de D3** : les trois appels à `ensure_not_pat` d'`admin.rs` (`:37`, `:143`) et de `fiscal_years.rs` (`:317`) vérifiés présents par `include_str!`.

## Dev Notes

### Ce qui est déjà en place

`ensure_not_pat` — `crates/kesh-api/src/routes/api_keys.rs:95` — teste `current_user.api_key_id.is_some()` et rend `AppError::ApiKeyManagementForbidden` (bras à `errors.rs:1104`, chaîne à `:1106`).

**Ses six sites d'appel, dont la moitié seulement est concernée** : `api_keys.rs:108`, `:121`, `:211` (routes de gestion de clés, dans `comptable_routes` — **hors** périmètre) ; `admin.rs:37`, `:143`, `fiscal_years.rs:317` (les trois d'`admin_routes` — ceux de **D3**).

`CurrentUser.api_key_id: Option<i64>` — `middleware/auth.rs:44`, `Some(...)` par le chemin PAT (`auth/api_key.rs:181`), `None` par le chemin JWT (`auth.rs:161`). **Seul discriminant nécessaire**, et déjà fiable.

`require_admin_role` — `middleware/rbac.rs`, appliqué en `route_layer` sur `admin_routes` (`lib.rs:284`), **et nulle part ailleurs**.

**Le gate de portée est ANTÉRIEUR** — `middleware/auth.rs:141-142`, dans `require_auth`, appliqué sur le routeur extérieur (`lib.rs:869-877`) : un PAT `read` sur une méthode non sûre (`is_safe_method` = `GET`/`HEAD`/`OPTIONS`) reçoit `403 API_KEY_READ_ONLY` **sans jamais atteindre `admin_routes`**.

### Le 405 n'est pas uniforme, et une passe l'a cru

⚠️ **Cinq chemins d'`admin_routes` existent aussi dans un autre routeur** — `/api/v1/vat-rates`, `/company/invoice-settings`, `/company/dunning-settings`, `/dunning-levels`, `/invoices/{id}`. Au `merge`, c'est le **fallback du dernier routeur fusionné** qui subsiste (`axum/src/routing/mod.rs:690-696`), et `authenticated_routes` est mergé après `admin_routes` (`lib.rs:870-873`).

Conséquence observable : `PATCH /api/v1/vat-rates` avec un PAT rend **`405`**, tandis que `PATCH /api/v1/users` — chemin admin exclusif — rend **`403 API_KEY_ADMIN_FORBIDDEN`**. **Impact sécurité nul** : aucun handler admin n'est atteint dans les deux cas. Le coût est un implémenteur qui écrit un test « méthode non enregistrée → 403 » et le voit rougir sur la moitié des chemins.

*(La passe 3 avait inscrit « le 405 est engendré après les deux couches » parmi ses vérifications sans défaut. C'est vrai des chemins exclusifs, faux des chemins partagés. Réfuté en passe 4.)*

### Le piège de cette story, et il est du même genre que le défaut

⚠️ **Un test écrit à la main sur quelques routes reproduirait le défaut.** Seize routes sur dix-neuf ont été oubliées parce que rien ne rappelait de les traiter ; un test qui n'énumère pas oubliera de la même façon la vingtième. C'est le raisonnement du garde-fou **P6** du `CLAUDE.md` sur les migrations positionnelles.

### Ce qu'il ne faut pas « simplifier »

Ne pas retirer les trois `ensure_not_pat` existants au motif que la couche les couvre — cf. **D3**.

### Conventions de test

Mutations **jouées, pas raisonnées**. Pour AC1 : retirer la couche doit faire tomber le test — **sur les 25 couples de la jambe `read-write`, et sur les 5 `get` + 5 `HEAD` de la jambe `read-only`**. Sur les 20 couples mutants en `read-only`, elle est **insatisfaisable par construction**, et c'est pourquoi AC1 y attend un autre code.

Les affirmations d'absence se vérifient au `grep -nF` avant d'être écrites — **et les affirmations de présence tout autant**. Les quatre passes de 22-4 ont produit un test i18n imaginaire, un précédent historique inventé et un test cité pour une preuve qu'il ne fait pas : **les trois étaient des affirmations de présence.**

### References

- Issue **#167** (KF-036) — références de ligne périmées et portée sous-estimée, cf. § *L'état réel du code*.
- Story **17-2a** — `17-2a-api-pat-backend.md`, limitation **L3**, décisions DC2, DC4, DC6.
- Story **22-4b** — la frontière documentaire, **à merger dans la même PR**.
- `CLAUDE.md` — § *Migration breaking policy* garde-fou **P6** pour l'analogie du test fail-loud ; § *Synchroniser TOUTES les docs* pour la conduite de merge.

## Change Log

**2026-08-13 — créée par découpage de la story 22-4**, après quatre passes de `validate` (Sonnet ×3 → Haiku + Opus ×2 → Sonnet ×3 → Haiku + Opus ×2). Trend de la story mère : `1/3/6` → `0/3/7` → `0/4/6` → `0/4/9`. Deux stagnations consécutives à HIGH, MEDIUM en hausse : critère de non-convergence atteint. Arbitrage de Guy — découper.

**Ce fichier reçoit le mécanisme, déjà remédié des quatre passes.** Les findings de passe 4 qui le concernent sont intégrés : le compteur aveugle à cinq familles de constructeurs (`any`, `on`, `trace`, `connect`, `*_service`), l'assertion bornée au bloc alors que le trou vit après lui, les cinq couples `HEAD` jamais comptés, le 405 non uniforme sur les chemins partagés, `authenticated_routes` jamais examiné par D5, le test i18n cité pour une preuve qu'il ne fait pas, et le décommentage par ligne entière.

**Aucun gate exécuté** : la story n'a pas de code. Les tableaux de gate restent vides jusqu'à `dev-story`.
