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

**D2 — Un code d'erreur distinct.** `AppError::ApiKeyManagementForbidden` rend aujourd'hui `403 API_KEY_MANAGEMENT_FORBIDDEN`. Le réutiliser pour « vous ne pouvez pas rouvrir un exercice avec un jeton » **mentirait à l'appelant** : son message parle de gestion de clés. Une variante dédiée — `API_KEY_ADMIN_FORBIDDEN` — avec son message propre sur les quatre locales.

**D3 — Les trois `ensure_not_pat` existants restent.** La couche les rend redondants sur ces routes, mais ils gardent une valeur : `full-export`, `full-import` et la réouverture d'exercice sont des opérations dont l'interdiction aux jetons est **intrinsèque**, indépendamment du routeur où elles vivent. Un déplacement de route ne doit pas les découvrir. *(Une redondance assumée et écrite vaut mieux qu'une garde retirée « parce que la couche s'en occupe ».)*

**D4 — La frontière DC6 est réécrite dans la spec parente.** Elle disait « un PAT ne gère pas les clés » ; elle dira « **un PAT n'atteint aucune route `require_admin_role`** ». C'est cette formulation qui rend le garde-fou vérifiable.

## Acceptance Criteria

**AC1 — Aucune route admin n'est atteignable par un PAT.**
*Preuve* : un test qui **énumère les routes du routeur `admin_routes`** et vérifie que chacune rend `403` avec un PAT `read-write` dont le créateur est Admin.
⚠️ **L'énumération est le cœur de l'AC.** Un test qui vérifie trois routes choisies à la main reproduirait exactement le défaut qu'on corrige : il passerait, et la quatrième route ajoutée demain ne serait pas couverte. Si l'énumération dynamique n'est pas praticable avec axum, alors **une liste explicite doublée d'une assertion de comptage** — `assert_eq!(routes_testées, routes_du_routeur)` — qui **rougit** dès qu'une route est ajoutée sans son test.

**AC2 — Le cas nominal n'est pas cassé.**
Un PAT `read-write` créé par un **Comptable** conserve l'accès à tout ce qu'il pouvait atteindre : écritures, factures, contacts, produits.
*Preuve* : les tests existants de `17-2a` restent verts **sans modification**. ⚠️ S'il faut en modifier un, c'est le signe que la couche est posée trop haut.

**AC3 — Un Admin devant son navigateur n'est pas affecté.**
La couche ne regarde que `api_key_id`, jamais le rôle.
*Preuve* : les suites E2E d'administration — utilisateurs, TVA, relances, modèles d'e-mail — restent vertes.

**AC4 — L'erreur dit ce qui se passe.**
`403` avec le code `API_KEY_ADMIN_FORBIDDEN` et un message traduit sur les quatre locales, distinct de celui de la gestion de clés.
*Preuve* : l'assertion de la **chaîne** de code vit dans `errors.rs`, seul endroit où le corps de la réponse est lisible ; et le test d'appariement positionnel de `kesh-i18n`.

**AC5 — Le chemin d'attaque est fermé, et le test le raconte.**
*Preuve* : un test qui rejoue la chaîne complète — PAT Admin → `POST /api/v1/users` → **403**. C'est le test qui dit *pourquoi* la story existe ; son nom doit le porter.

**AC6 — La documentation dit la nouvelle frontière.**
Manuel administrateur : ce qu'un jeton peut et ne peut pas faire, et que **l'administration en est exclue quel que soit le rôle du créateur**. La spec 17-2 (DC6) est amendée. CHANGELOG dans les mots de l'utilisateur.

## Tasks / Subtasks

- [ ] **T1 — La couche** (AC1, AC3). Un `route_layer` sur `admin_routes`, à côté de `require_admin_role`. ⚠️ **Vérifier l'ordre d'application** : dans axum, `route_layer` s'empile — s'assurer que le refus PAT ne dépend pas du passage préalable du RBAC, et documenter l'ordre obtenu plutôt que le supposer.
- [ ] **T2 — Le code d'erreur** (AC4). Variante `AppError`, ligne dans le `match` d'`errors.rs`, clé i18n sur les **quatre** locales.
- [ ] **T3 — Le test d'énumération** (AC1). Le point difficile de la story. Chercher d'abord si axum expose les routes d'un `Router` ; sinon, liste explicite **plus** assertion de comptage qui rougit à tout ajout.
- [ ] **T4 — Le test du chemin d'attaque** (AC5).
- [ ] **T5 — Non-régression** (AC2, AC3). Les suites `api_pat_*` et les E2E d'administration, **sans modification**.
- [ ] **T6 — Documentation** (AC6). Manuel admin, amendement de DC6 dans `17-2*`, CHANGELOG.

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

1. **L'énumération des routes** — axum expose-t-il les chemins d'un `Router` construit ? Si non, l'assertion de comptage est le repli, et il faut décider **où** elle vit pour qu'un ajout de route la fasse rougir.
2. **Les autres routeurs** — cette story ferme `admin_routes`. Le routeur **comptable+** doit-il l'être aussi ? Non par défaut : le cas d'usage nominal du PAT est précisément d'écrire des écritures et des factures. Mais **la question mérite d'être écrite et tranchée**, faute de quoi une passe de revue la rouvrira.
3. **Les jetons existants** — faut-il alerter l'administrateur qu'un jeton créé par un Admin perd des accès qu'il avait ? Aucune migration n'est nécessaire, mais un mot au CHANGELOG évite un ticket de support.
