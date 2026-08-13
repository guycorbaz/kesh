# Story 22.4a : La couche qui interdit un PAT sur les routes d'administration

## Status

review

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

- [x] **T1 — La couche** (AC1, AC3). Un `route_layer` sur `admin_routes`, à côté de `require_admin_role`. L'ordre entre les deux **couches** est à arbitrer : il décide du code rendu quand toutes deux refusent. Cible en **D6**, tranchée par le test d'AC1.
  - [x] **Un commentaire d'avertissement aux deux `.route_layer(...)`** : tout ajout de route à `admin_routes` se fait **au-dessus** d'eux — et jamais par réaffectation plus loin dans le fichier —, sinon il échappe aux deux couches (cf. **D1**).
- [x] **T2 — Le code d'erreur** (AC4). Variante `AppError`, bras dans le `match` d'`errors.rs` (calque `ApiKeyManagementForbidden`, `:1104-1106`), clé i18n sur les **quatre** locales, **message distinct**. ⚠️ La variante **s'ajoute** : `API_KEY_MANAGEMENT_FORBIDDEN` garde son consommateur (cf. D5).
- [x] **T3 — Le test de complétude** (AC1). `include_str!` sur `lib.rs`, bornes par marqueurs, et les **cinq exigences** d'AC1 :
  - [x] compter les constructeurs de méthode, assertion **exacte `== 25`** — jamais un plancher ;
  - [x] tronquer chaque ligne à son premier `//` avant comptage ;
  - [x] asserter les deux marqueurs, une fois chacun, et **exactement deux** `.route_layer(` dans le bloc ;
  - [x] asserter qu'aucun ajout de route ne suit le **premier** `.route_layer(` — liste complète en AC1 exigence 4 ;
  - [x] asserter, **sur le fichier entier**, qu'aucune réaffectation `admin_routes =` n'existe hors déclaration. ⚠️ **Le nombre attendu est 2, pas 3** : l'exigence 4 d'AC1 parlait de « trois sites connus » en comptant `lib.rs:356`, qui est un **commentaire**. L'assertion portant sur la source **décommentée**, il n'en reste que deux — la déclaration et le `.merge(`. Plus robuste : un commentaire ne peut plus faire ni faux positif ni faux négatif ;
  - [x] asserter le **complément** de constructeurs : ni `any(`, `on(`, `trace(`, `connect(`, ni `*_service(` dans le bloc ;
  - [x] couvrir les 25 constructeurs + les 5 `HEAD`, asserter **le code**, attente **par couple** selon le tableau d'AC1 ;
  - [x] un couple avec un PAT de créateur **Comptable**, code selon **D6**.
  - Vérification à la main du décompte, ventilation comprise :
    ```sh
    sed -n '182,284p' crates/kesh-api/src/lib.rs | grep -v '^\s*//' \
      | grep -oE '\b(get|post|put|delete|patch|head|options)\(' | sort | uniq -c
    # 4 delete( · 5 get( · 6 post( · 10 put(  → 25
    ```
- [x] **T4 — Le test du chemin d'attaque** (AC5). `a_leaked_admin_pat_can_no_longer_create_an_administrator` — et il vérifie **aussi** qu'aucun utilisateur n'a été créé en base : l'échec est réel, pas seulement rapporté.
- [x] **T5 — Non-régression** (AC2, AC3). `api_keys_e2e.rs` et les E2E d'administration restent verts. **Trois assertions changent, et trois seulement.** ⚠️ Ne pas toucher à `api_keys_e2e.rs:425`.
  - [x] **La fixture frontend `frontend/src/lib/features/admin-backup/admin-backup.api.test.ts:112`** porte `API_KEY_MANAGEMENT_FORBIDDEN` et devient un mensonge sur le contrat — **sans rougir**, puisqu'elle n'asserte qu'un rejet.
  - [x] **L'assertion de source de D3** : les trois appels à `ensure_not_pat` d'`admin.rs` (`:37`, `:143`) et de `fiscal_years.rs` (`:317`) vérifiés présents par `include_str!`.

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

## Dev Agent Record

### Ce qui a été implémenté

**La couche** est `middleware::rbac::require_not_pat`, posée en `route_layer` sur `admin_routes` **après** `require_admin_role` — donc extérieure à lui, donc elle répond la première. C'est ce qui donne la précédence voulue par **D6**, et le test `comptable_created_pat_gets_the_same_code_as_an_admin_one` l'a **tranché par exécution** : la spec refusait de s'en remettre à une lecture du source d'axum, et elle avait raison de laisser le test arbitrer.

**Le bloc `admin_routes` est borné** par `// KESH-ADMIN-ROUTES-BEGIN` / `// KESH-ADMIN-ROUTES-END`, et porte un avertissement de six lignes aux `.route_layer(` : *toute route s'ajoute au-dessus*.

**Le test de complétude** vit dans `crates/kesh-api/tests/admin_pat_denied_e2e.rs`, en deux familles qui ne prouvent pas la même chose — sept tests de **source** (sans base) pour la complétude, cinq tests **HTTP** pour le comportement.

### Deux écarts à la spec, tous deux dans le sens du durcissement

**1. L'assertion sur `admin_routes` attend 2 occurrences, pas 3.** L'exigence 4 d'AC1 comptait « trois sites connus » en incluant `lib.rs:356`, qui est un **commentaire**. Comme l'assertion porte sur la source décommentée, il n'en reste que deux. C'est plus robuste : un commentaire ne peut plus faire ni faux positif ni faux négatif — et de fait, mes propres commentaires d'avertissement contiennent le mot `admin_routes` et l'exemple `admin_routes = admin_routes.route(...)`, qui auraient fait rougir une assertion sur la source brute.

**2. Une exigence non prévue s'est imposée : `lib_rs_has_no_double_slash_inside_literals`.** Le décommentage tronque chaque ligne à son premier `//` — ce qui couperait aussi une URL dans une chaîne. `lib.rs` n'en contient aucune aujourd'hui ; ce test le vérifie, de sorte que le jour où l'une y entre, le dispositif rougit au lieu de dériver en silence.

### Les mutations, jouées et non raisonnées

La § *Conventions de test* l'exige, et c'est le cœur d'une story dont le sujet est le test muet. **Huit mutations appliquées, huit rougissements** — chacune exécutée, aucune déduite :

| Mutation | Test qui rougit |
|---|---|
| M1 — une route chaînée **après** les couches | `nothing_is_added_after_the_first_route_layer` |
| M2 — une méthode ajoutée à un enregistrement existant | `block_declares_exactly_the_listed_couples` |
| M3 — `any(` à la place de `get(` (**avec son import**, pour que ça compile) | `block_uses_no_unlisted_route_constructor` |
| M4 — une réaffectation `admin_routes` hors du bloc | `admin_routes_is_never_reassigned` |
| M5 — le marqueur de fin retiré | `admin_routes_block_is_bounded_exactly_once` |
| M6 — un `ensure_not_pat` de D3 retiré | `the_three_intrinsic_guards_are_still_wired` |
| M7 — **la couche retirée** | 6 tests, dont les 5 HTTP |
| M8 — le message i18n copié depuis la gestion de clés | `admin_forbidden_message_is_translated_and_distinct_from_key_management` |

⚠️ **M3 a d'abord été jouée sans son import et n'a rien prouvé** : la mutation cassait la compilation, si bien qu'aucun test ne tournait. Rejouée avec `any` importé — le scénario réel, puisqu'un développeur qui écrit `any(` ajoute l'import —, elle compile, et le test rougit. *Une mutation qui ne compile pas n'est pas une mutation jouée.*

⚠️ **M7 se comporte exactement comme AC1 l'annonçait** : elle fait rougir les 25 couples de la jambe `read-write` et les 5 `get` de la jambe `read-only`, et **reste sans effet sur les 20 couples mutants** — que le gate de portée arrête en amont. C'est la démonstration que la jambe `read-only` de la première rédaction était bien un test muet sur ces vingt-là.

### Une preuve plus faible que les autres, et il faut le dire

Les cinq couples `HEAD` n'assertent que le **statut**, pas le code : une réponse `HEAD` n'a pas de corps, donc `error.code` y est illisible **par construction**. C'est assumé — `HEAD` emprunte exactement la pile de couches de son `GET`, déjà couvert au code — mais c'est une preuve plus faible, et elle est marquée comme telle dans le test.

### File List

| Fichier | Nature |
|---|---|
| `crates/kesh-api/src/middleware/rbac.rs` | la couche `require_not_pat` |
| `crates/kesh-api/src/lib.rs` | marqueurs, couche montée, avertissement |
| `crates/kesh-api/src/errors.rs` | variante `ApiKeyAdminForbidden` + bras du `match` |
| `crates/kesh-i18n/locales/{fr,de,en,it}-CH/messages.ftl` | clé `error-api-key-admin-forbidden` |
| `crates/kesh-i18n/src/loader.rs` | test de traduction + distinction |
| `crates/kesh-api/tests/admin_pat_denied_e2e.rs` | **neuf** — 12 tests |
| `crates/kesh-api/tests/admin_full_export_e2e.rs` · `admin_full_import_e2e.rs` · `fiscal_years_e2e.rs` | les trois assertions de D2 |
| `frontend/src/lib/features/admin-backup/admin-backup.api.test.ts` | la fixture qui ne rougit pas |

### Gates exécutés

| Gate | Résultat |
|---|---|
| `cargo fmt --all -- --check` | vert *(après un `cargo fmt` — l'ordre des `use` du test neuf dérivait)* |
| `cargo clippy --workspace --all-targets -- -D warnings` | vert, 0 avertissement |
| `cargo nextest` sur le rayon d'impact — `admin_pat_denied_e2e`, `admin_full_{export,import}_e2e`, `fiscal_years_e2e`, `api_keys_e2e`, `package(kesh-i18n)` | **108/108**, 269 s |
| `npm run lint-i18n-ownership` | PASS |
| `npm run check` | 0 erreur (27 avertissements préexistants) |
| `npm run test:unit` | 512/512, 63 fichiers |
| `npm run build` | vert |
| **Gate backend complet** — `scripts/test-fast.sh --ci` | **vert : 2177/2177**, 0 échec, 0 retry, 0 flake, 4 ignorés (préexistants), 3893 s. Code de retour du **gate** relevé à `0` dans un fichier dédié — voir le piège ci-dessous |

**Périmètre du décompte** : `2177` est le total du workspace au commit de dev de cette story ; **13 tests sont neufs** — 12 dans `admin_pat_denied_e2e`, 1 dans `kesh-i18n` —, recomptés depuis la source (`grep -c` aux deux bornes). `2177 − 13 = 2164`, la ligne de base annoncée pour la v0.9.0 : les deux comptes se recoupent.

⚠️ Les E2E Playwright n'ont **pas** été exécutés : cette story ne touche au frontend qu'une fixture de test unitaire. Ils restent dus avant le `push` de la PR, conformément à la § *Test Locally First*.

### Le premier gate « complet » n'en était pas un — deux pièges, coup sur coup

**Piège 1 — le code de retour rapporté n'était pas celui du gate.** La commande lancée était `scripts/test-fast.sh … > log 2>&1` suivie d'un `tail` : le code de retour du bloc est celui du **`tail`**, soit `0`. Le gate, lui, avait échoué. **Un montage qui fait paraître vert un gate rouge est plus dangereux que le rouge** — c'est la variante « shell » du test muet. Le second run capture le code dans un fichier dédié.

**Piège 2 — le fail-fast a laissé 1350 tests sur 2177 sans tourner.** Le profil `default` de nextest coupe au premier échec : `827/2177 tests run`. Un tel run **ne peut pas** être déclaré gate complet, même si son unique échec est bénin.

**L'échec, lui, est la KF-038 et rien d'autre.** `reconciliation_e2e::post_accept_skips_non_chf_transaction` a rendu `409` au lieu de `200` — un conflit de verrou optimiste, pas un refus d'autorisation. Vérifié plutôt que supposé :

- le test s'authentifie en **JWT** sur `/api/v1/reconciliation/accept`, route de `comptable_routes` ; la couche neuve ne se déclenche que sur `api_key_id.is_some()` **et** sur `admin_routes`. Elle ne peut pas être en cause ;
- **rejoué isolé : vert en 7,9 s** ;
- l'issue **#228** est ouverte et s'intitule mot pour mot « [KF-038] Flake test intégration — `reconciliation_e2e::post_accept_skips_non_chf_transaction` **sous contention parallèle** ».

⚠️ **La contention était réelle et extérieure au dépôt** : la pile e2e d'un autre projet (`e2e-db-1`, `mybibli-rust-test-db`) tournait sur la même machine. C'est le cas de figure de la § *Plafonds mémoire* — la victime n'est pas la fautive.

Le gate a donc été rejoué en **profil `ci`** (`fail-fast = false`, `retries = 1`), seul montage qui donne à la fois la couverture complète et l'absorption du flake connu. **Résultat : 2177/2177, sans même un retry** — la KF-038 ne s'est pas reproduite, ce qui confirme la contention comme cause et non le code.

## Change Log

**2026-08-13 — créée par découpage de la story 22-4**, après quatre passes de `validate` (Sonnet ×3 → Haiku + Opus ×2 → Sonnet ×3 → Haiku + Opus ×2). Trend de la story mère : `1/3/6` → `0/3/7` → `0/4/6` → `0/4/9`. Deux stagnations consécutives à HIGH, MEDIUM en hausse : critère de non-convergence atteint. Arbitrage de Guy — découper.

**Ce fichier reçoit le mécanisme, déjà remédié des quatre passes.** Les findings de passe 4 qui le concernent sont intégrés : le compteur aveugle à cinq familles de constructeurs (`any`, `on`, `trace`, `connect`, `*_service`), l'assertion bornée au bloc alors que le trou vit après lui, les cinq couples `HEAD` jamais comptés, le 405 non uniforme sur les chemins partagés, `authenticated_routes` jamais examiné par D5, le test i18n cité pour une preuve qu'il ne fait pas, et le décommentage par ligne entière.

---

**2026-08-13 — `bmad-dev-story` : implémentation.** La couche, le code d'erreur, 13 tests neufs. **Gate backend complet vert, 2177/2177** ; frontend vert (512/512, `check` sans erreur, `build`) ; `clippy` workspace sans avertissement.

**Ce que l'implémentation a appris, et qui ne se lit pas dans la spec :**

- **Les huit mutations ont toutes rougi** — dont M7, retirer la couche, qui se comporte exactement comme AC1 l'annonçait : elle atteint les 25 couples `read-write` et les 5 `get` de la jambe `read-only`, et **reste sans effet sur les 20 mutants**. La démonstration, par exécution, que cette jambe était bien un test muet avant la passe 2.
- **Une mutation qui ne compile pas n'est pas une mutation jouée** : M3 (`any(`) n'a d'abord rien prouvé, la compilation cassant faute d'import.
- **L'exigence 4 d'AC1 comptait un commentaire.** L'assertion attend 2 occurrences de `admin_routes`, pas 3 : `lib.rs:356` est un commentaire, et l'assertion porte sur la source décommentée. Plus robuste — mes propres commentaires d'avertissement auraient fait rougir une assertion sur la source brute.
- **Un piège de gate a failli faire passer un rouge pour un vert**, et il n'a rien à voir avec les tests : le code de retour lu était celui du `tail` de la commande, pas celui du gate. Consigné en Dev Agent Record, § *Le premier gate « complet » n'en était pas un*.
