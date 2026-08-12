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

## Décisions

**D1 — Le blocage se pose en COUCHE sur le routeur, pas dans les handlers.**

L'issue proposait les deux options. **L'état du code tranche** : l'approche par handler a été appliquée trois fois et **oubliée seize**. Ce n'est pas un défaut de diligence, c'est la propriété d'un dispositif qui doit être rappelé à chaque ajout de route — et rien ne le rappelle. Un `route_layer` sur `admin_routes`, à côté de `require_admin_role` qui vit déjà là, se pose **une fois** et couvre par construction toute route ajoutée ensuite.

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

**D3 — Les trois `ensure_not_pat` existants restent.** La couche les rend redondants sur ces routes, mais ils gardent une valeur : `full-export`, `full-import` et la réouverture d'exercice sont des opérations dont l'interdiction aux jetons est **intrinsèque**, indépendamment du routeur où elles vivent. Un déplacement de route ne doit pas les découvrir. *(Une redondance assumée et écrite vaut mieux qu'une garde retirée « parce que la couche s'en occupe » — mais elle a un prix, celui décrit en D2.)*

**D4 — La frontière DC6 est réécrite dans `17-2a-api-pat-backend.md`, PAS dans `17-2`.** Elle disait « un PAT ne gère pas les clés » ; elle dira « **un PAT n'atteint aucune route `require_admin_role`** ».

⚠️ **La cible importe, et la première rédaction se trompait.** Le texte de DC6 réellement autoritaire vit **verbatim dans `17-2a-api-pat-backend.md`** — la story `done`, avec son Dev Agent Record —, tandis que `17-2-api-pat-integrations.md` est resté en `ready-for-dev`, vestige du découpage. Amender le second seul laisserait **le document que tout le monde lit** avec l'ancienne frontière. *(Relevé en passe 1 de `validate`.)*

**D5 — Le routeur `comptable_routes` reste HORS périmètre, et c'est une décision, non une question.** Vérifié : il ne porte aucune route capable de créer un utilisateur, de réinitialiser un mot de passe ou de changer la configuration de la société — les seuls vecteurs de la propriété de containment que cette story défend. Et le fermer casserait le cas d'usage nominal du PAT (DC4), qui est précisément d'écrire des écritures et des factures. *(Cette décision était rangée en question ouverte ; deux lentilles ont relevé qu'elle était en réalité tranchée, et qu'un « oui » ultérieur casserait AC2 sans garde-fou. Promue ici.)*

## Acceptance Criteria

**AC1 — Aucune route admin n'est atteignable par un PAT, quel que soit son scope.**

**« Route » signifie ici un couple (méthode, chemin)**, et non un chemin : `/api/v1/users` porte un `GET` et un `POST`, qui sont deux routes. Le « 19 » du tableau ci-dessus compte des **enregistrements `.route(…)`**, dont plusieurs portent deux ou trois méthodes — le nombre de couples à couvrir est donc **supérieur à 19**, et c'est celui-là qui fait foi. *(Ambiguïté relevée en passe 1 : un test calibré sur 19 serait structurellement incomplet.)*

*Preuve* : un test qui couvre **chaque couple (méthode, chemin)** d'`admin_routes` et vérifie qu'il rend `403`, avec un PAT dont le créateur est Admin — **testé aux deux scopes, `read-write` ET `read-only`**. Le discriminant retenu ne regarde que `api_key_id`, jamais le scope ; ne prouver que `read-write` laisserait le second cas sans critère, alors qu'une route admin en lecture reste un vecteur de fuite.

⚠️ **La complétude est le cœur de l'AC, et son mécanisme est arrêté ici.** `axum 0.8` **n'expose aucune énumération** — vérifié dans son source : `Router` n'offre que `has_routes() -> bool`. L'énumération dynamique est donc **impossible**, et le repli d'abord envisagé — comparer à « le nombre de routes du routeur » — était **circulaire** : ce nombre ne pouvait venir que d'une seconde constante entretenue à la main, c'est-à-dire du « quelqu'un doit se souvenir » que cette story existe pour éliminer.

**Le mécanisme retenu dérive le compte de la SOURCE, qui est la seule chose qu'un ajout de route modifie forcément** : le test lit `lib.rs` (`include_str!`) et compte les `.route(` du bloc `admin_routes`. Une route ajoutée sans son test fait donc **rougir** le test, sans que personne n'ait à y penser.

⚠️ **La borne du bloc doit être robuste** — un marqueur de commentaire délimitant `admin_routes`, et non un numéro de ligne, qui se périmerait au premier remaniement.

**AC2 — Le cas nominal n'est pas cassé.**
Un PAT `read-write` créé par un **Comptable** conserve l'accès à tout ce qu'il pouvait atteindre : écritures, factures, contacts, produits.
*Preuve* : `api_keys_e2e.rs` reste vert **sans modification** — vérifié en passe 1 : aucun de ses tests n'exerce une route d'`admin_routes`, et tous ses contextes sont créés en rôle Comptable.

**AC3 — Un Admin devant son navigateur n'est pas affecté.**
La couche ne regarde que `api_key_id`, jamais le rôle.
*Preuve* : les suites E2E d'administration — utilisateurs, TVA, relances, modèles d'e-mail — restent vertes.
⚠️ **À une exception près, prévue et bornée** : les **trois** assertions de code d'erreur listées en D2 changent d'ancien vers nouveau code. Toute autre modification de test est un signal d'alerte, pas celle-là.

**AC4 — L'erreur dit ce qui se passe.**
`403` avec le code `API_KEY_ADMIN_FORBIDDEN` et un message traduit sur les quatre locales, distinct de celui de la gestion de clés.
*Preuve* : l'assertion de la **chaîne** de code vit dans `errors.rs`, seul endroit où le corps de la réponse est lisible ; et le test d'appariement positionnel de `kesh-i18n`.

**AC5 — Le chemin d'attaque est fermé, et le test le raconte.**
*Preuve* : un test qui rejoue la chaîne complète — PAT Admin → `POST /api/v1/users` → **403**. C'est le test qui dit *pourquoi* la story existe ; son nom doit le porter.

**AC6 — La documentation dit la nouvelle frontière.**
Manuel administrateur : ce qu'un jeton peut et ne peut pas faire, et que **l'administration en est exclue quel que soit le rôle du créateur**.
Le CHANGELOG dit, dans les mots de l'utilisateur, **qu'un jeton créé par un Admin perd des accès qu'il avait** — sans quoi la première intégration qui tombe en `403` produit un ticket de support.
*Preuve*, et c'est la seule AC qui en manquait :
- `grep -nF "require_admin_role" _bmad-output/implementation-artifacts/17-2a-api-pat-backend.md` rend la **nouvelle** formulation de DC6 ;
- `grep -c "jeton" docs/manual/fr/admin-manual.tex` a augmenté, et la section citée décrit l'exclusion ;
- l'entrée CHANGELOG existe et mentionne la perte d'accès des jetons Admin existants.

## Tasks / Subtasks

- [ ] **T1 — La couche** (AC1, AC3). Un `route_layer` sur `admin_routes`, à côté de `require_admin_role`. ✅ **L'ordre n'a pas à être arbitré** — vérifié en passe 1 dans le source d'axum : `route_layer` **enveloppe le service**, donc la couche s'exécute avant le handler quel que soit son ordre relatif au RBAC. La conséquence est en D2.
- [ ] **T2 — Le code d'erreur** (AC4). Variante `AppError`, ligne dans le `match` d'`errors.rs`, clé i18n sur les **quatre** locales.
- [ ] **T3 — Le test de complétude** (AC1). Le point difficile, désormais tranché : `axum 0.8` n'expose **aucune** énumération (`has_routes()` seul). Le test dérive donc le compte de la **source** — `include_str!` sur `lib.rs`, comptage des `.route(` entre deux marqueurs de commentaire bornant `admin_routes`. ⚠️ Borner par **marqueurs**, jamais par numéros de ligne.
  - [ ] Couvrir chaque couple **(méthode, chemin)**, pas chaque chemin.
  - [ ] Les deux scopes : `read-write` **et** `read-only`.
- [ ] **T4 — Le test du chemin d'attaque** (AC5).
- [ ] **T5 — Non-régression** (AC2, AC3). `api_keys_e2e.rs` et les E2E d'administration restent verts. **Trois assertions changent, et trois seulement** — celles listées en D2. ⚠️ Ne pas toucher à `api_keys_e2e.rs:425`.
- [ ] **T6 — Documentation** (AC6). Manuel admin ; amendement de DC6 dans **`17-2a-api-pat-backend.md`** — le document autoritaire, cf. D4 — ; CHANGELOG mentionnant la perte d'accès des jetons Admin existants.

## Dev Notes

### Ce qui est déjà en place

`ensure_not_pat` — `crates/kesh-api/src/routes/api_keys.rs:95` — teste `current_user.api_key_id.is_some()` et rend `AppError::ApiKeyManagementForbidden`, mappé en `403 API_KEY_MANAGEMENT_FORBIDDEN` (`errors.rs:1104`).

`CurrentUser.api_key_id: Option<i64>` — `middleware/auth.rs:44`, renseigné à `Some(...)` par le chemin PAT (`auth/api_key.rs:181`) et à `None` par le chemin JWT (`auth.rs:161`). **C'est le seul discriminant nécessaire**, et il est déjà fiable.

`require_admin_role` — `middleware/rbac.rs`, appliqué en `route_layer` sur `admin_routes` (`lib.rs:284`). C'est le voisin auprès duquel la nouvelle couche se pose.

### Le piège de cette story, et il est du même genre que le défaut

⚠️ **Un test écrit à la main sur quelques routes reproduirait le défaut.** Seize routes sur dix-neuf ont été oubliées parce que rien ne rappelait de les traiter ; un test qui n'énumère pas oubliera de la même façon la dix-neuvième. C'est pour cela qu'AC1 exige soit l'énumération, soit une assertion de comptage qui **fail-loud** — le même raisonnement que le garde-fou **P6** du `CLAUDE.md` sur les migrations positionnelles.

### Ce qu'il ne faut pas « simplifier »

Ne pas retirer les trois `ensure_not_pat` existants au motif que la couche les couvre — cf. **D3**.

### Conventions de test

Mutations **jouées, pas raisonnées**. Pour AC1, la mutation est explicite : retirer la couche doit faire tomber le test, **sur toutes les routes** et non sur une seule.
Les affirmations d'absence se vérifient au `grep -nF` avant d'être écrites.

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

**Il n'en reste qu'une, et elle est mineure :**

1. **Le marqueur de bornage d'`admin_routes`** dans `lib.rs` — quelle forme exacte, et faut-il un test qui vérifie que les **deux** marqueurs existent encore ? Sans eux, le comptage de T3 porterait sur un bloc vide et le test passerait **à vide** : c'est le mode d'échec du test muet, que ce dépôt a déjà payé deux fois. Une assertion « le compte est > 0 » suffit à le fermer, et devrait sans doute figurer dans T3.

## Change Log

**2026-08-12 — `bmad-create-story validate`, PASSE 1 (Sonnet, trois lentilles en parallèle).**

| Lentille | CRIT | HIGH | MED | LOW |
|---|---|---|---|---|
| Blind Hunter (spec seule) | **1** | 2 | 3 | 2 |
| Edge Case Hunter (spec + code) | 0 | **1** | 1 | 0 |
| Acceptance Auditor (spec + issue + story parente) | 0 | 0 | 2 | 1 |

**Le défaut central était une contradiction entre deux de mes propres décisions.** D2 crée un code d'erreur distinct ; D3 conserve les trois `ensure_not_pat` existants. Or `route_layer` **enveloppe le service** — vérifié dans le source d'axum — donc la couche répond **avant** le handler, quel que soit son ordre relatif au RBAC. Les trois tests qui assertent `API_KEY_MANAGEMENT_FORBIDDEN` (`admin_full_export_e2e.rs:409`, `admin_full_import_e2e.rs:414`, `fiscal_years_e2e.rs:1638`) recevront donc le **nouveau** code et rougiront.

⚠️ **Et mon avertissement d'AC2 aurait égaré le développeur** : il disait qu'une modification de test signalerait « une couche posée trop haut ». La cause n'était pas le placement — correct — mais ma contradiction. Il aurait conduit à remettre en cause la bonne décision. AC2 dit désormais que **trois** assertions changent, et trois seulement.

**Le CRITICAL portait sur une circularité, et il avait raison.** AC1 exigeait d'énumérer les routes du routeur, avec pour repli une assertion contre « le nombre de routes du routeur ». Mais `axum 0.8` **n'expose aucune énumération** — `Router` n'offre que `has_routes() -> bool`, vérifié dans le source. Ce nombre n'aurait donc pu venir que d'une seconde constante à la main : **exactement le « quelqu'un doit se souvenir » que cette story existe pour éliminer**. Le mécanisme retenu dérive désormais le compte de la **source** (`include_str!` sur `lib.rs`), seule chose qu'un ajout de route modifie forcément.

**Trois autres corrections de fond :**

- **« Route » n'était pas défini.** Le tableau compte des **chemins** (19 enregistrements `.route(…)`), un routeur axum route par **méthode** : `/api/v1/users` porte un `GET` et un `POST`. Un test calibré sur 19 aurait été structurellement incomplet. AC1 parle désormais de couples (méthode, chemin).
- **AC1 ne prouvait que le scope `read-write`**, alors que le discriminant retenu ne regarde que `api_key_id` et ignore le scope. Le cas `read-only` n'avait aucun critère. Ajouté.
- **J'amendais la mauvaise cible.** D4 disait d'amender « la spec parente 17-2 » — or le texte de DC6 autoritaire vit **verbatim dans `17-2a`**, la story `done`, tandis que `17-2` est un vestige de découpage resté `ready-for-dev`. Suivie à la lettre, la consigne aurait laissé le document que tout le monde lit avec l'ancienne frontière.

**Deux points de forme corrigés** : AC6 était la seule AC **sans clause de preuve**, donc la seule impossible à cocher objectivement — et c'était précisément celle qui portait l'amendement à la cible fausse. Et la frontière du routeur `comptable_routes`, tranchée en substance mais rangée en question ouverte, est promue en **D5** : sans cela, un « oui » ultérieur aurait cassé AC2 sans qu'aucun garde-fou ne le signale.

**Ce que les lentilles ont confirmé plutôt que réfuté** — et c'est utile de le savoir : le décompte 16/19 est exact, recompté ligne à ligne ; les trois corrections que la spec apportait à l'issue #167 sont vraies, dont la remédiation partielle établie par `git log -S` et non par déduction ; `require_admin_role` n'est appliqué **nulle part ailleurs**, donc la couche couvre bien toute la surface admin présente et future ; et aucune branche résiduelle du chemin d'attaque n'a été trouvée — le changement de mot de passe en self-service, seul candidat, exige la vérification Argon2 du mot de passe courant que l'attaquant n'a pas.

**Trois questions ouvertes sur trois sont closes.** Il n'en reste qu'une, mineure et née de la correction elle-même : la forme du marqueur bornant `admin_routes` dans `lib.rs`, et le garde-fou qui empêche le comptage de porter sur un bloc vide — le mode d'échec du test muet.
