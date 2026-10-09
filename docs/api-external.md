# API externe de Kesh — clés d'accès PAT

> Public : développeurs et intégrateurs (scripts, agents IA, ETL, dashboards BI, ERP) qui consomment l'API de Kesh au nom d'une entreprise (*company*).
> Version : v0.2 (Epic 17 « Infra & Souveraineté », issue [#100](https://github.com/guycorbaz/kesh/issues/100)).

Kesh expose son API HTTP REST aux intégrations externes via des **clés d'accès personnelles** (PAT — *Personal Access Token*). Une clé permet à une IA externe (Claude API, ChatGPT, agent custom) ou à un logiciel tiers de lire (et, si autorisé, modifier) les données comptables d'une company **sans partager d'identifiants utilisateur**.

---

## 1. Vue d'ensemble

- **Mêmes routes que l'UI web.** L'API externe n'a pas d'URL dédiée : elle réutilise les routes `/api/v1/*` existantes. Seul le mode d'authentification diffère (en-tête `Authorization: Bearer …` au lieu du cookie de session du navigateur).
- **Une clé = une company.** Chaque clé est liée à une seule entreprise. Toutes les requêtes faites avec cette clé sont automatiquement restreintes aux données de cette company — il n'y a pas de paramètre `company` à passer.
- **Une clé agit au nom de son créateur.** La clé hérite du **rôle courant** de l'utilisateur qui l'a créée (relu à chaque requête). Si ce créateur est désactivé ou voit son rôle changer, l'effet est **immédiat** sur la clé.
- **Deux niveaux d'accès** (*scope*) : lecture seule (`read`) ou lecture-écriture (`read-write`).

---

## 2. Authentification

Présentez la clé dans l'en-tête HTTP `Authorization`, schéma `Bearer` :

```
Authorization: Bearer kesh_pat_XXXXXXXXXXXXXXXXXXXXXXXXXXX
```

### Format de la clé

Une clé Kesh a toujours la forme **`kesh_pat_`** suivie de **27 caractères** alphanumériques (`0-9`, `A-Z`, `a-z`), soit **36 caractères au total**. Exemple (factice) :

```
kesh_pat_3kQ9mZ1aB7cD4eF6gH8iJ0kL2mN
```

Le secret encode 160 bits d'entropie cryptographique. Côté serveur, **seule une empreinte SHA-256 est stockée** — le secret en clair n'existe qu'au moment de la création et **ne peut plus jamais être récupéré** ensuite. En cas de perte, créez une nouvelle clé et révoquez l'ancienne.

### Vérifier qu'une clé fonctionne

L'endpoint `GET /api/v1/auth/me` renvoie l'identité effective de la clé (le créateur) — utile pour valider une intégration :

```sh
curl -H "Authorization: Bearer kesh_pat_VOTRE_CLE" \
     https://kesh.example.ch/api/v1/auth/me
```

---

## 3. Créer et gérer ses clés (interface web uniquement)

> ⚠️ **La gestion des clés se fait exclusivement depuis l'interface web** — pas via l'API. Une requête API qui tenterait de lister, créer ou révoquer une clé est rejetée (`403 API_KEY_MANAGEMENT_FORBIDDEN`), **même avec une clé `read-write`**. C'est une protection délibérée : une clé compromise ne peut pas se cloner ni en créer de nouvelles.

Dans Kesh, connecté en tant qu'utilisateur (rôle Comptable ou Administrateur) :

1. Allez dans **Paramètres → Clés API** (`/settings/api-keys`).
2. Cliquez sur **Nouvelle clé**, renseignez :
   - un **nom** (ex. « Agent IA comptable », « Export BI nocturne ») ;
   - une **portée** : *Lecture seule* (`read`) ou *Lecture-écriture* (`read-write`) ;
   - une **expiration** optionnelle (laissez vide pour une clé permanente).
3. **Copiez immédiatement la clé affichée** : elle n'est montrée qu'une seule fois.
4. Pour invalider une clé, utilisez **Révoquer** — l'effet est immédiat (la requête suivante échoue en `401`).

La liste affiche aussi la date de dernière utilisation et le statut (active / expirée / révoquée).

Après la mise à jour en 0.13.0, une clé qui désignait une société effacée (défaut [#528](https://github.com/guycorbaz/kesh/issues/528)) est **révoquée par Kesh** et répond `401 UNAUTHENTICATED` ; elle paraît « révoquée » dans la liste. Créez-en une neuve.

---

## 4. Portées (*scopes*)

| Portée | Méthodes HTTP autorisées | Usage |
|--------|--------------------------|-------|
| `read` | `GET`, `HEAD`, `OPTIONS` | Lecture seule. Toute tentative d'écriture → `403 API_KEY_READ_ONLY`. |
| `read-write` | Toutes les méthodes (`GET`, `HEAD`, `OPTIONS`, `POST`, `PUT`, `PATCH`, `DELETE`) | Lecture **et** modification, **sous réserve du rôle du créateur**. |

**La permission effective est l'intersection du rôle du créateur et de la portée de la clé.** Une clé `read-write` créée par un Comptable ne pourra pas faire ce que seul un Administrateur peut faire : la portée `read-write` ne *promeut* jamais le créateur. Appliquez le **principe du moindre privilège** (voir §8).

> ⚠️ **Et cette intersection ne suffit pas : l'administration est fermée aux clés API, sans exception.** Aucune route réservée au rôle **Administrateur** n'est atteignable avec une clé, **quel que soit le rôle de son créateur** — y compris une clé `read-write` créée par un Administrateur. La réponse est `403 API_KEY_ADMIN_FORBIDDEN`.
>
> Concrètement, ces opérations se font **dans l'interface web** : gestion des utilisateurs et réinitialisation de mots de passe, paramètres de facturation et de relance de l'entreprise, création et modification des taux de TVA et des niveaux de rappel, modèles d'e-mail, coordonnées de l'entreprise, suppression d'une facture, export et import d'installation, réouverture d'un exercice clôturé.
>
> Le motif est le même que pour la gestion des clés : **une clé compromise ne doit pas pouvoir créer un compte administrateur**, sans quoi la révoquer n'arrêterait plus l'incident.

> ⚠️ **La consultation du journal d'audit est fermée elle aussi**, et elle ne relève pourtant pas de l'administration. Les trois routes `/api/v1/audit-log`, `/audit-log/vocabulary` et `/audit-log/export.csv` refusent toute clé, y compris une clé `read` créée par un Administrateur. **Le code diffère** : `403 API_KEY_MANAGEMENT_FORBIDDEN`, et non `API_KEY_ADMIN_FORBIDDEN` (cf. §10). Son libellé parle de « gestion de clés » pour des raisons historiques ; ici, il signifie simplement qu'une clé n'a pas accès au journal d'audit. La lecture se fait dans l'interface web, par un Comptable ou un Administrateur.

---

## 5. URL de base et périmètre des données

L'URL de base dépend de votre déploiement :

```
http(s)://<hôte>:<port>/api/v1
```

Par exemple `https://kesh.example.ch/api/v1` (l'hôte et le port sont ceux configurés par votre administrateur ; voir le manuel d'administration, section « Référence des ports »).

Toutes les requêtes sont **automatiquement restreintes à la company de la clé**. Vous n'avez aucun identifiant d'entreprise à transmettre.

---

## 6. Exemples

Les exemples ci-dessous utilisent le carnet d'adresses (`/api/v1/contacts`). Le même schéma s'applique à toutes les ressources `/api/v1/*` (voir §7).

### 6.1 `curl`

**Lecture** (clé `read` suffisante) :

```sh
curl -H "Authorization: Bearer kesh_pat_VOTRE_CLE" \
     https://kesh.example.ch/api/v1/contacts
```

**Écriture** (clé `read-write` requise) — créer un contact :

```sh
curl -X POST \
     -H "Authorization: Bearer kesh_pat_VOTRE_CLE" \
     -H "Content-Type: application/json" \
     -d '{
           "contactType": "Entreprise",
           "name": "Restaurant du Pont SA",
           "isClient": true,
           "email": "compta@restaurant-du-pont.ch"
         }' \
     https://kesh.example.ch/api/v1/contacts
```

### 6.2 Python (`requests`)

```python
import requests

BASE = "https://kesh.example.ch/api/v1"
HEADERS = {"Authorization": "Bearer kesh_pat_VOTRE_CLE"}

# Lecture
resp = requests.get(f"{BASE}/contacts", headers=HEADERS, timeout=30)
resp.raise_for_status()
contacts = resp.json()

# Écriture (clé read-write)
nouveau = {
    "contactType": "Entreprise",
    "name": "Restaurant du Pont SA",
    "isClient": True,
    "email": "compta@restaurant-du-pont.ch",
}
resp = requests.post(f"{BASE}/contacts", json=nouveau, headers=HEADERS, timeout=30)
resp.raise_for_status()
print(resp.json())
```

### 6.3 JavaScript (`fetch`)

```javascript
const BASE = "https://kesh.example.ch/api/v1";
const HEADERS = { Authorization: "Bearer kesh_pat_VOTRE_CLE" };

// Lecture
const contacts = await fetch(`${BASE}/contacts`, { headers: HEADERS })
  .then((r) => r.json());

// Écriture (clé read-write)
const created = await fetch(`${BASE}/contacts`, {
  method: "POST",
  headers: { ...HEADERS, "Content-Type": "application/json" },
  body: JSON.stringify({
    contactType: "Entreprise",
    name: "Restaurant du Pont SA",
    isClient: true,
    email: "compta@restaurant-du-pont.ch",
  }),
}).then((r) => r.json());
```

### 6.4 Agent IA / serveur MCP

L'API de Kesh est consommable par **tout client HTTP**, donc par tout agent IA ou serveur MCP (*Model Context Protocol*) capable d'appeler des API REST avec un en-tête d'authentification.

> ℹ️ **Il n'existe pas de serveur MCP « Kesh-natif » en v0.2.** Pour exposer Kesh à un agent, utilisez un serveur MCP HTTP générique (ou un client d'API custom) et injectez-y l'en-tête `Authorization`.

Exemple de configuration d'un serveur MCP HTTP générique pointant vers Kesh (le format exact dépend du serveur MCP utilisé) :

```json
{
  "mcpServers": {
    "kesh": {
      "type": "http",
      "baseUrl": "https://kesh.example.ch/api/v1",
      "headers": {
        "Authorization": "Bearer kesh_pat_VOTRE_CLE"
      }
    }
  }
}
```

Pour une IA en lecture seule (analyse de comptes, génération de rapports), créez une clé **`read`** : elle ne pourra jamais modifier vos données.

---

## 7. Ressources disponibles

Les principales ressources accessibles via l'API (liste non exhaustive — toute route `/api/v1/*` de l'UI est consommable, **à deux exceptions près, toutes deux fermées aux clés API** : les routes d'administration, et **la consultation du journal d'audit** (`/api/v1/audit-log*`) — cf. §4 pour l'administration, §10 pour le journal, dont le **code d'erreur diffère**) :

| Ressource | Lecture (`read`) | Écriture (`read-write`) |
|-----------|------------------|--------------------------|
| Identité de la clé | `GET /auth/me` | — |
| Plan comptable | `GET /accounts` | `POST /accounts`, `PUT /accounts/{id}` ³, … |
| Contacts | `GET /contacts`, `GET /contacts/{id}` | `POST /contacts`, … |
| Produits | `GET /products`, `GET /products/{id}` | `POST /products`, … |
| Factures | `GET /invoices`, `GET /invoices/{id}` | `POST /invoices`, `PUT /invoices/{id}`, … ² |
| Écritures comptables | `GET /journal-entries`, `GET /journal-entries/{id}` | `POST /journal-entries`, `PUT /journal-entries/{id}` ⁴, `DELETE /journal-entries/{id}` ⁴, … |
| Lettrages | `GET /letterings/{key}` ⁵ | `POST /letterings` ⁵, `DELETE /letterings/{key}` ⁵ |
| Taux de TVA | `GET /vat-rates` | — ¹ |
| Comptes bancaires | `GET /bank-accounts` | `POST /bank-accounts`, `PUT /bank-accounts/{id}`, `PATCH /bank-accounts/{id}` (lien au grand livre), `DELETE /bank-accounts/{id}` (archivage) |

*(Préfixe `…/api/v1` omis dans le tableau. Les corps de requête d'écriture peuvent différer des champs renvoyés en lecture : référez-vous aux formulaires correspondants de l'interface web pour les champs attendus.)*

⁴ Depuis la v0.13.0 — voir « Modifier une écriture » et « Supprimer une écriture » ci-dessous.

⁵ Depuis la v0.13.0 — voir « Lettrer des lignes » ci-dessous.

**Les lignes d'une écriture portent leur lettrage** *(depuis la v0.13.0)* : dans toute réponse qui expose les lignes (`GET /journal-entries`, `GET /journal-entries/{id}`, et les réponses de `POST`, `PUT` et de la contre-passation), chaque ligne porte trois champs, **toujours présents** : `letteringKey` (la clé du groupe de lettrage), `letteringCode` (son code affiché, `AA` pour la clé 27) et `letteringOrigin` (`manual`, posé par les routes du lettrage, ou `reversal`, posé par la contre-passation ; `document` est réservé aux lettrages des pièces que Kesh posera de lui-même, voir « Lettrer des lignes ») — tous trois `null` quand la ligne est **ouverte**.

### Modifier une écriture — `PUT /api/v1/journal-entries/{id}`

**Ouverte aux clés API** en écriture (`read-write`), comme la saisie (`POST /journal-entries`). *(Depuis la v0.13.0 ; de la v0.12.0 à la v0.12.1, ce `PUT` rendait toujours `409 ENTRY_IS_POSTED`.)*

Une écriture se modifie **tant que son exercice est ouvert**, qu'**aucun exercice postérieur n'est clôturé** (le bilan est cumulatif : il reprend l'écriture), qu'**aucune pièce ne la possède**, que sa date — l'ancienne comme la nouvelle — est **postérieure** à la période verrouillée, et qu'**aucune de ses lignes n'est lettrée** (une écriture lettrée se délettre d'abord, `DELETE /api/v1/letterings/{key}`). En pratique : les écritures saisies à la main, l'écriture d'ouverture et les compléments de soldes de départ. Le numéro, l'exercice et la date de création **ne changent jamais** ; la nouvelle date doit rester dans l'exercice **de l'écriture**.

Corps : celui du `POST`, plus la version lue — `{ "entryDate", "journal", "description", "lines": [{ "accountId", "debit", "credit", "projectId"? }], "version" }`. Réponse : `200` et l'écriture, même forme que le `POST`. Un corps identique à l'état présent rend `200` sans rien écrire.

Chaque modification effective est **tracée** au journal d'audit (`journal_entry.updated`), avec l'état **avant et après**, lignes comprises — et **la clé** qui l'a faite, quand c'est une clé.

`GET /journal-entries/{id}` porte de quoi décider avant d'essayer : `modifiable`, et, quand il vaut `false`, `modificationBlockedBy` (le motif) et `modificationBlockedLabel` (numéro de pièce, nom de l'exercice postérieur clos, borne du verrou ou code de lettrage). ⚠️ Ce motif ne voit que l'état **présent** : le corps, la nouvelle date et la version ne sont contrôlés qu'au `PUT`.

Refus, **dans l'ordre où ils parlent** — 16 lignes :

| Refus | Code | Statut |
|---|---|---|
| Corps illisible (JSON invalide, champ manquant) | — (rejet de l'extracteur) | `400` / `422` |
| Écriture déséquilibrée, montant ou libellé invalide | `ENTRY_UNBALANCED`, `VALIDATION_ERROR` | `400` |
| Écriture inconnue ou d'une autre company | `NOT_FOUND` | `404` |
| Exercice de l'écriture clôturé | `FISCAL_YEAR_CLOSED` | `400` |
| Exercice **postérieur** clôturé | `LATER_FISCAL_YEAR_CLOSED` | `400` |
| Écriture contre-passée | `ENTRY_IS_REVERSED` | `409` |
| Écriture qui est elle-même une contre-passation | `IS_A_REVERSAL` | `409` |
| Écriture d'une pièce — facture, avoir, facture fournisseur, règlement ou solde, transaction bancaire rapprochée | `OWNED_BY_INVOICE`, `OWNED_BY_CREDIT_NOTE`, `OWNED_BY_SUPPLIER_INVOICE`, `OWNED_BY_SETTLEMENT`, `MATCHED_BANK_TRANSACTION` | `409` |
| Paiement d'une facture fournisseur **annulée** (règlement détaché) | `DETACHED_SUPPLIER_SETTLEMENT` | `409` |
| Version périmée | `OPTIMISTIC_LOCK_CONFLICT` | `409` |
| Nouvelle date hors de l'exercice de l'écriture | `DATE_OUTSIDE_FISCAL_YEAR` | `400` |
| Projet inconnu / archivé (un projet **déjà présent** sur l'écriture reste toléré) | `NOT_FOUND` / `ILLEGAL_STATE_TRANSITION` | `404` / `409` |
| Compte inconnu, archivé ou d'une autre company — **y compris sur une ligne inchangée** | `INACTIVE_OR_INVALID_ACCOUNTS` | `400` |
| Compte non imputable — **y compris sur une ligne inchangée** | `ACCOUNT_NOT_POSTABLE` | `400` |
| Ancienne **ou** nouvelle date dans la période verrouillée (seuil inclusif) | `PERIOD_LOCKED` | `400` |
| Une ligne de l'écriture est **lettrée** — délettrer d'abord | `ENTRY_LETTERED` | `409` |

Les refus `409` d'une pièce portent `details.documentId` (l'identifiant de la pièce — pour `DETACHED_SUPPLIER_SETTLEMENT`, celui de la facture fournisseur annulée) et `details.documentNumber` (son numéro, quand elle en a un) ; `ENTRY_LETTERED` porte sa forme propre, `details.letteringCode` — le **code** du premier groupe de lettrage de l'écriture (ordre `id`) —, **sans** `documentId` ni `documentNumber` (aucune pièce n'est en cause). `LATER_FISCAL_YEAR_CLOSED` porte `details.fiscalYearId` et `details.fiscalYearName` — le plus proche exercice postérieur clos.

`LATER_FISCAL_YEAR_CLOSED` garde **toutes** les écritures du journal *(depuis la v0.13.0, #543)* : ce `PUT`, la suppression (`DELETE`, ci-dessous), la dévalidation d'une facture, et **toute création** d'écriture — saisie (`POST /journal-entries`, clés comprises), contre-passation, validation et règlement d'une facture, solde du reste, avoir, factures fournisseur et leur paiement, lot de paiement, import de facture fournisseur, soldes de départ et leur complément, rapprochements et leurs annulations. Il refuse aussi la création d'un exercice (`POST /fiscal-years`). L'état « exercice ouvert suivi d'un exercice clôturé » ne se forme plus depuis la v0.13.0 (les exercices se clôturent dans l'ordre) ; il ne se rencontre que dans des données antérieures (installation mise à jour, sauvegarde restaurée). Le refus naît de l'écriture elle-même : il se place, dans chaque route, **après** `FISCAL_YEAR_CLOSED` / `FISCAL_YEAR_INVALID` et **avant** `PERIOD_LOCKED`.

⚠️ **Une écriture refusée se corrige par contre-passation** (`POST /journal-entries/{id}/reverse`), sauf l'écriture d'une pièce, qui se corrige depuis sa pièce. L'écriture **lettrée** se contre-passe aussi ; pour la **modifier** plutôt, délettrez-la d'abord (`DELETE /api/v1/letterings/{key}`). La contre-passation **lettre** ce qui est libre : chaque ligne de l'origine sur un compte lettrable, non encore lettrée, forme avec son miroir un groupe `reversal` ; la réponse `201` porte ces lignes lettrées (`letteringCode` non nul), et l'origine ne change que par cette marque — montants, comptes, dates et libellés intacts. Une ligne déjà lettrée garde son groupe, et son miroir reste ouvert. ⚠️ **Une écriture sur un compte archivé ou devenu non imputable reste modifiable** — à condition de remplacer ce compte : l'enregistrement refuse tant qu'une ligne le vise. ⚠️ Un interblocage avec une écriture concurrente est rejoué par le serveur ; s'il persiste, la réponse est un `500` — réessayez.

### Supprimer une écriture — `DELETE /api/v1/journal-entries/{id}`

**Ouverte aux clés API** en écriture (`read-write`). *(Depuis la v0.13.0 ; de la v0.12.0 à la v0.12.1, ce `DELETE` rendait toujours `409 ENTRY_IS_POSTED`, code **retiré** depuis.)*

Une écriture se supprime **dans le même cadre qu'elle se modifie** : exercice ouvert, aucun exercice postérieur clôturé, aucune pièce, pas une contre-passation ni une écriture contre-passée, pas le paiement d'une facture fournisseur annulée, date **postérieure** à la période verrouillée, aucune ligne lettrée. Le motif de `GET /journal-entries/{id}` (`modifiable`, `modificationBlockedBy`) vaut aussi pour la suppression. Réponse : `204`, sans corps.

La suppression est **tracée** au journal d'audit (`journal_entry.deleted`), avec le **contenu complet** de l'écriture, lignes comprises — et **la clé** qui l'a faite, quand c'est une clé. ⚠️ **Le numéro n'est jamais réattribué** : la suppression laisse un trou dans la numérotation de l'exercice, que la trace d'audit explique.

Refus, **dans l'ordre où ils parlent** — 10 lignes :

| Refus | Code | Statut |
|---|---|---|
| Écriture inconnue ou d'une autre company | `NOT_FOUND` | `404` |
| Exercice de l'écriture clôturé | `FISCAL_YEAR_CLOSED` | `400` |
| Exercice **postérieur** clôturé | `LATER_FISCAL_YEAR_CLOSED` | `400` |
| Écriture contre-passée | `ENTRY_IS_REVERSED` | `409` |
| Écriture qui est elle-même une contre-passation | `IS_A_REVERSAL` | `409` |
| Écriture d'une pièce — facture, avoir, facture fournisseur, règlement ou solde | `OWNED_BY_INVOICE`, `OWNED_BY_CREDIT_NOTE`, `OWNED_BY_SUPPLIER_INVOICE`, `OWNED_BY_SETTLEMENT` | `409` |
| Écriture rapprochée d'une transaction bancaire | `MATCHED_BANK_TRANSACTION` | `409` |
| Paiement d'une facture fournisseur **annulée** (règlement détaché) | `DETACHED_SUPPLIER_SETTLEMENT` | `409` |
| Date dans la période verrouillée (seuil inclusif) | `PERIOD_LOCKED` | `400` |
| Une ligne de l'écriture est **lettrée** — délettrer d'abord | `ENTRY_LETTERED` | `409` |

Les `details` sont ceux du `PUT` (`documentId`, `documentNumber` ; `fiscalYearId`, `fiscalYearName` ; `letteringCode`). ⚠️ Une écriture d'une pièce sous la période verrouillée répond par **sa pièce** (`409`), pas par `PERIOD_LOCKED` : c'est la pièce qui dit où corriger. ⚠️ `ENTRY_LETTERED` parle **après** tous les autres refus, au `PUT` comme au `DELETE` : quand il parle, le délettrage (`DELETE /letterings/{key}`) aboutit, sauf geste concurrent sur les mêmes lignes. ⚠️ Un interblocage est rejoué par le serveur ; s'il persiste, la réponse est un `500` — réessayez.

### Lettrer des lignes — `/api/v1/letterings`

*(Depuis la v0.13.0 ; l'écran viendra.)* Un **lettrage** marque comme se soldant entre elles des lignes d'un **même compte** d'actif ou de passif — une écriture et sa contre-passation, un acompte et sa reprise. Un groupe réunit **de 2 à 200 lignes** dont la somme `débit − crédit` est **exactement nulle** ; pas de lettrage partiel. Sa **clé** est le plus petit identifiant de ses lignes, son **code** cette clé écrite en lettres (`1 → A`, `27 → AA`). L'origine d'un groupe est `manual` quand il est posé par ces routes, `reversal` quand la **contre-passation** l'a posé (une ligne et sa correction — « Modifier une écriture », ci-dessus). L'origine `document` (une pièce soldée) est réservée aux lettrages des pièces que Kesh posera de lui-même dans une version ultérieure ; aucune route ne la rend encore. ⚠️ Une écriture dont une ligne est lettrée ne se modifie ni ne se supprime (`409 ENTRY_LETTERED`) : délettrez d'abord.

**`POST /api/v1/letterings`** — écriture (`read-write`). Corps : `{ "lineIds": [ … ] }`. Réponse `201` : `{ key, code, origin: "manual", accountId, lines: [ { id, entryId, entryNumber, fiscalYearId, fiscalYearName, date, debit, credit } ] }`. ⚠️ Le numéro d'écriture repart à 1 à chaque exercice : il se lit avec `fiscalYearId` / `fiscalYearName`.

**`GET /api/v1/letterings/{key}`** — lecture (`read` suffit). `{key}` est la clé numérique **ou** le code (`27` ou `AA`, minuscules acceptées) ; toute autre valeur rend `404`. Réponse `200`, même forme, `origin` réel.

**`DELETE /api/v1/letterings/{key}`** — écriture (`read-write`). Délettre le groupe : ses lignes redeviennent ouvertes. Réponse `204`.

Les deux gestes sont **tracés** au journal d'audit (`lettering.created`, `lettering.removed`), lignes et exercices compris — et **la clé** qui les a faits, quand c'est une clé.

**La règle des périodes** : lettrer **comme délettrer** exige qu'**au moins une** ligne du groupe soit en période ouverte — exercice ouvert, aucun exercice postérieur clôturé, date **postérieure** à la période verrouillée (seuil inclusif). Un groupe à cheval sur un exercice clôturé et un exercice ouvert se lettre ; un groupe dont **toutes** les lignes sont en période close ne se lettre ni ne se délettre — le lettrage se fige avec la période.

**Ce qui ne se lettre pas à la main** : les lignes d'une **pièce** (facture, avoir, facture fournisseur, règlement ou solde) — leur lettrage est celui de leur pièce ; et les comptes de charge et de produit, ainsi que les comptes liés à un compte bancaire (archivé compris), qui relèvent du rapprochement bancaire. Une écriture **rapprochée** d'une transaction bancaire, elle, n'est pas une pièce : elle se lettre.

Refus du `POST`, **dans l'ordre où ils parlent** — une requête qui cumule deux causes rend la première :

| Refus | Code | Statut |
|---|---|---|
| Plus de 200 lignes | `LETTERING_TOO_MANY_LINES` | `400` |
| Moins de deux lignes, ou une ligne répétée | `LETTERING_TOO_FEW_LINES` | `400` |
| Une ligne inconnue ou d'une autre company (indiscernables) | `NOT_FOUND` | `404` |
| Lignes de comptes différents | `LETTERING_ACCOUNTS_DIFFER` | `409` |
| Compte non lettrable (charge, produit, compte bancaire) | `LETTERING_ACCOUNT_NOT_LETTERABLE` | `409` |
| Toutes les lignes en période close (exercice clôturé, exercice suivi d'un exercice clôturé, période verrouillée) | `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` | `409` |
| Une ligne appartient à une pièce | `LETTERING_LINE_OWNED_BY_DOCUMENT` | `409`, `details.documentId` / `details.documentNumber` |
| Une ligne est déjà lettrée | `LETTERING_LINE_ALREADY_LETTERED` | `409`, `details.code` = le code de son groupe |
| La somme n'est pas nulle | `LETTERING_UNBALANCED` | `409`, `details.difference` (décimal en chaîne) |

Refus du `DELETE`, dans l'ordre :

| Refus | Code | Statut |
|---|---|---|
| Clé ou code invalide, groupe inconnu ou d'une autre company | `NOT_FOUND` | `404` |
| Groupe d'origine `document` — annuler le règlement, pas délettrer *(aucun groupe `document` n'existe encore)* | `LETTERING_IS_DOCUMENT` | `409` |
| Groupe `reversal` dont une ligne appartient à une pièce (par exemple l'écriture d'achat d'une facture fournisseur annulée) | `LETTERING_LINE_OWNED_BY_DOCUMENT` | `409` |
| Toutes les lignes en période close | `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` | `409` |

Le délettrage n'exige pas que le compte soit encore lettrable : un groupe dont le compte a été retypé ou rattaché depuis à un compte bancaire se délettre. ⚠️ `LETTERING_CONCURRENT_CHANGE` (`409`) signale qu'un groupe a changé entre la lecture et l'écriture : réessayez. ⚠️ Un interblocage est rejoué par le serveur ; s'il persiste, la réponse est un `500` — réessayez.

### Dévalider une facture — `POST /api/v1/invoices/{id}/unvalidate`

**Ouverte aux clés API** en écriture, comme la validation (`POST /invoices/{id}/validate`).
Elle repasse une facture validée en **brouillon**, **conserve son numéro** et supprime son écriture comptable. Deux sorties ensuite : supprimer le brouillon, ou le corriger et le revalider — la revalidation **reprend le même numéro**, sans consommer de nouveau.

Corps : `{ "version": n }` — le verrou optimiste. Réponse : la facture, même forme que la validation.

| Refus | Code | Statut |
|---|---|---|
| Règlement, même partiel | `INVOICE_HAS_SETTLEMENTS` | `409` |
| Créditée par un avoir | `INVOICE_CREDITED` | `409` |
| Historique de rappels | `INVOICE_HAS_REMINDERS` | `409` |
| Envoyée au client | `INVOICE_EMAILED` | `409` |
| Écriture rapprochée d'une transaction bancaire | `MATCHED_BANK_TRANSACTION` | `409` |
| Exercice clos | `FISCAL_YEAR_CLOSED` | `400` |
| Exercice **postérieur** clôturé *(depuis la v0.13.0)* | `LATER_FISCAL_YEAR_CLOSED` | `400` |
| Écriture contre-passée | `ENTRY_IS_REVERSED` | `409` |
| Écriture en période verrouillée | `PERIOD_LOCKED` | `400` |
| Version périmée | `OPTIMISTIC_LOCK_CONFLICT` | `409` |
| Facture qui n'est pas au statut « validée » | `ILLEGAL_STATE_TRANSITION` | `409` |

⚠️ **Le champ `details` est générique : `documentNumber` n'est PAS toujours un numéro de document, et pas toujours celui d'un AUTRE document.** Branchez votre logique sur le **code d'erreur**, qui est stable ; lisez `details` comme un complément d'affichage, motif par motif :

| Code | `documentId` | `documentNumber` |
|---|---|---|
| `INVOICE_HAS_SETTLEMENTS` | l'identifiant du règlement — **`null`** si le refus vient du seul `paid_at`, c'est-à-dire pour une facture réglée **avant la v0.12**, qui a introduit les lignes de règlement | le numéro de **la facture elle-même** |
| `INVOICE_CREDITED` | l'identifiant de l'avoir | le numéro de l'avoir — **`null`** tant que l'avoir est un brouillon |
| `INVOICE_HAS_REMINDERS` | l'identifiant du rappel | le numéro de **la facture elle-même** |
| `INVOICE_EMAILED` | `null` | l'**adresse du destinataire** |
| `MATCHED_BANK_TRANSACTION` | l'identifiant de la transaction bancaire | le numéro de **la facture elle-même** |

`LATER_FISCAL_YEAR_CLOSED` porte sa forme propre — `details.fiscalYearId` et `details.fiscalYearName`, l'exercice postérieur clôturé le plus proche —, **pas** `documentId` / `documentNumber`.

⛔ Les **cinq autres** codes du tableau des refus — 11 lignes en tout, 6 avec `details` et 5 sans — `FISCAL_YEAR_CLOSED`, `ENTRY_IS_REVERSED`, `PERIOD_LOCKED`, `OPTIMISTIC_LOCK_CONFLICT`, `ILLEGAL_STATE_TRANSITION` — n'émettent **aucun** `details`. Un message du type « bloquée par le document {documentNumber} » opposerait donc la facture à elle-même dans trois cas sur cinq.

⚠️ **« Envoyée au client » est un refus sec** : il ne se lève par aucune confirmation. Une facture que le client détient se corrige par un **avoir**. La garde ne connaît que ce que Kesh a envoyé lui-même — un PDF téléchargé puis transmis à la main ne laisse aucune trace.

⚠️ **Un brouillon qui porte déjà un numéro ne change pas d'exercice** : `PUT /invoices/{id}` refuse une date hors de l'exercice qui a émis le numéro (`INVOICE_NUMBER_FISCAL_YEAR_MISMATCH`, `409`).

### Lister et annuler les règlements d'une facture

**`GET /api/v1/invoices/{id}/settlements`** — lecture (`read` suffit). Les règlements de la facture, du plus ancien au plus récent : `id`, `journalEntryId`, `amount`, `settledOn`, `settlementType` (`bank_transfer` / `internal_account` / `write_off` — un solde, ci-dessous), `writeOffNature` (la nature d'un solde, `null` pour un règlement), et **`cancellable`** — calculé par la fonction même qui refuserait l'annulation. Quand il vaut `false`, `cancelBlockedBy` porte le code du motif, `cancelBlockedLabel` le numéro du compte archivé et `cancelBlockedDocumentId` l'identifiant de la transaction bancaire rapprochée. ⚠️ `cancellable` ne dit rien des **droits** de la clé : une clé `read` lit `true` et reçoit `403` à l'annulation. ⚠️ **Angle mort assumé** *(v0.13.0)* : `cancellable` ne voit pas qu'un exercice **postérieur** à l'exercice du jour est clôturé (données antérieures à la v0.13.0, exercice futur clôturé d'avance) — il vaut alors `true` et l'annulation rend `400 LATER_FISCAL_YEAR_CLOSED` (issue [#568](https://github.com/guycorbaz/kesh/issues/568)).

**`POST /api/v1/invoices/{id}/settlements/{settlementId}/cancel`** — écriture (`read-write`), ouverte aux clés comme `POST /invoices/{id}/settlements`. Sans corps. Annule le règlement par **contre-passation** : une écriture inverse **datée du jour**, dans l'exercice ouvert qui le couvre ; la ligne de règlement est retirée ; `paidAt` retombe à `null` si le reste dû redevient positif. Réponse : `{ invoice, reversalJournalEntryId }`, la facture relue.

| Refus | Code | Statut |
|---|---|---|
| Facture créditée par un avoir — le règlement reste ouvert au compte débiteurs | `INVOICE_CREDITED` | `409` |
| Un **solde** existe sur la facture — annuler d'abord le solde (le solde lui-même reste annulable) | `INVOICE_WRITTEN_OFF` | `409` |
| Règlement d'un exercice **clos** — un administrateur doit rouvrir les exercices clôturés jusqu'à celui-ci, en commençant par le plus récent | `FISCAL_YEAR_CLOSED` | `409` |
| Règlement rapproché d'une transaction bancaire | `MATCHED_BANK_TRANSACTION` | `409`, `details.documentId` = la transaction |
| Compte du règlement archivé | `ACCOUNT_ARCHIVED` | `400`, `details.rejected[]` nomme les comptes |
| Aucun exercice ouvert ne couvre la date du jour | `FISCAL_YEAR_INVALID` | `400` |
| L'exercice du jour est suivi d'un exercice clôturé — la contre-passation y changerait son bilan *(depuis la v0.13.0)* | `LATER_FISCAL_YEAR_CLOSED` | `400`, `details.fiscalYearId` / `fiscalYearName` |
| Date du jour dans une période verrouillée | `PERIOD_LOCKED` | `400` |
| Facture ou règlement introuvable (ou d'une autre société) | `NOT_FOUND` | `404` |

**Refus de `POST /api/v1/invoices/{id}/settlements` (enregistrer un règlement)** — distinct de l'annulation ci-dessus : un règlement dont un compte est **le compte débiteurs de la facture** (celui que son écriture de vente a débité) est refusé en `400 SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT`, sans rien écrire — l'écriture serait `D 1100 / C 1100`, et le reste dû baisserait sans que rien ne bouge au grand livre. `details` : `{ accountId, accountNumber, claim: "receivable", role }`, où `role` vaut `counterparty` (le compte interne choisi, ou le compte comptable lié au compte bancaire du virement — refusé après les refus propres de ce compte, avant le trop-perçu) ou `rounding` (le compte de différences d'arrondi des réglages, quand le paiement solde au centime — à remplacer dans *Paramètres → Facturation*). *(Depuis la v0.13.0.)*

### Solder le reste d'une facture

**`POST /api/v1/invoices/{id}/write-off`** — écriture (`read-write`), ouverte aux clés comme `POST /invoices/{id}/settlements`. Corps : `{ nature, settledOn, version }`, `nature` parmi `discount` (escompte accordé), `bank_fees` (frais bancaires retenus par la banque du client), `bad_debt` (perte sur débiteur), `rounding` (reste d'arrondi, moins de 5 centimes). **Sans montant** : le serveur solde **tout** le reste dû, au reste exact, si bien que la facture est payée. La `version` de la facture est exigée — un écran périmé, ou un reste qui a changé depuis la lecture, est refusé en `409 OPTIMISTIC_LOCK_CONFLICT`. L'écriture, au journal OD, débite le compte de la nature (réglé dans *Paramètres → Facturation*), crédite la créance ; pour `discount` et `bad_debt`, elle débite aussi la **TVA due au prorata des taux** de la facture. Le solde apparaît dans la liste des règlements (`settlementType = write_off`) et s'annule comme eux. Réponse : `{ invoice, journalEntryId, amount }`.

| Refus | Code | Statut |
|---|---|---|
| Nature inconnue | `VALIDATION_ERROR` | `400` |
| Facture non validée ou annulée par avoir | `ILLEGAL_STATE_TRANSITION` | `409` |
| `version` périmée | `OPTIMISTIC_LOCK_CONFLICT` | `409` |
| Facture déjà payée, rien à solder, reste d'arrondi de 5 centimes ou plus, date antérieure à la facture | `INVALID_INPUT` | `400` |
| Aucun compte utilisable désigné pour la nature | `WRITE_OFF_ACCOUNT_NOT_CONFIGURED` | `400` |
| Compte de TVA due absent ou inutilisable — archivé, non imputable (escompte, perte) | `CONFIGURATION_REQUIRED` | `400` |
| Un compte désigné pour la nature, le reste d'arrondi ou la TVA due est le compte débiteurs de la facture (compte débiteurs changé de type depuis, puis désigné) — l'écriture serait `D 1100 / C 1100` *(depuis la v0.13.0)* | `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT` | `400`, `details.role` (`write_off_nature`, `rounding`, `vat_payable`) ; refusés dans cet ordre, le premier nommé |
| Aucun exercice ouvert ne couvre la date | `FISCAL_YEAR_INVALID` | `400` |
| L'exercice de la date est suivi d'un exercice clôturé *(depuis la v0.13.0)* | `LATER_FISCAL_YEAR_CLOSED` | `400`, `details.fiscalYearId` / `fiscalYearName` |
| Date dans une période verrouillée | `PERIOD_LOCKED` | `400` |

### Émettre un avoir — refus de compte

**`POST /api/v1/credit-notes`** — l'avoir contre-passe l'écriture de la facture sur **les comptes que la vente a mouvementés** : il crédite le compte débiteurs que la facture a **débité** et contre-passe l'arrondi sur le compte d'arrondi que la facture a mouvementé, non sur ceux des réglages du moment ; un réglage débiteurs vide ne l'empêche donc plus *(depuis la v0.13.0)*. Le compte de TVA due, lui, reste celui des réglages.

| Refus | Code | Statut |
|---|---|---|
| Compte débiteurs ou compte d'arrondi de la vente archivé, ou compte de TVA due des réglages archivé (si l'avoir porte de la TVA) — contrôlé **avant** l'exercice | `ACCOUNT_ARCHIVED` | `400`, `details.rejected[{accountId, accountNumber}]` |
| Compte de produit d'une ligne archivé | `CREDIT_NOTE_REVENUE_ACCOUNT_ARCHIVED` | `400`, `details.rejected[]` nomme la ligne |
| Compte de produit par défaut non désigné dans les réglages (exigé dans tous les cas), ou compte de TVA due non désigné alors que l'avoir porte de la TVA | `CONFIGURATION_REQUIRED` | `400` |

⚠️ **Changement** *(v0.13.0)* : un compte débiteurs, d'arrondi ou de TVA due archivé rendait `400 INACTIVE_OR_INVALID_ACCOUNTS` (ou `ROUNDING_ACCOUNT_NOT_CONFIGURED` pour l'arrondi) ; un réglage débiteurs vide rendait `CONFIGURATION_REQUIRED`.

### Annuler le règlement d'une facture fournisseur

**`POST /api/v1/supplier-invoices/{id}/settlement/cancel`** — écriture (`read-write`), ouverte aux clés comme `POST /supplier-invoices/{id}/pay`. Sans corps. Contre-passe l'écriture de règlement (datée du jour) et ramène la facture à `open` : `settlementType`, `settlementJournalEntryId`, `paidAt`… reviennent à `null`. Le lot de paiement confirmé qui l'a éventuellement réglée **n'est pas modifié**. Réponse : `{ invoice, reversalJournalEntryId }`. Distinct de `POST /supplier-invoices/{id}/cancel`, qui annule la **facture** elle-même (ci-dessous).

`GET /supplier-invoices/{id}`, la réponse de `pay` et celle de l'annulation portent `settlementCancellable`, `settlementCancelBlockedBy` (code du motif), `settlementCancelBlockedLabel` (numéro du compte archivé) et `lastConfirmedBatch` (`{ id, confirmedAt }` du lot confirmé **le plus récent** qui contient la facture — un fait **historique**, qui ne dit pas d'où vient le règlement courant). Ailleurs, ces champs valent `null` : *non calculés*. Après `pay` ou l'annulation, la facture est **relue** avec ses champs, dans une même lecture : la réponse décrit l'état courant — celui d'une écriture concurrente éventuelle —, jamais un état qui se contredit.

**Refus de `POST /api/v1/supplier-invoices/{id}/pay` (régler une facture fournisseur)** — distinct de l'annulation : une contrepartie qui est **le compte créanciers de la facture** (celui que son écriture d'achat a crédité) — compte interne, ou compte comptable lié au compte bancaire du virement — est refusée en `400 SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT`, sans rien écrire (`D 2000 / C 2000` passerait la facture « payée » sans paiement) ; `details` : `{ accountId, accountNumber, claim: "payable", role: "counterparty" }`. Les lots de paiement font le même contrôle : `POST /api/v1/payment-batches` place la facture dans `failed[]` (`200`) avec ce code et `details.bankAccountId` en plus — ou avec `SUPPLIER_INVOICE_PURCHASE_ENTRY_MALFORMED` (`details.reason = "no_credit_line_on_purchase_entry"`, `details.purchaseEntryId`) si son écriture d'achat n'a pas de ligne de crédit ; `POST /api/v1/payment-batches/{id}/confirm` refuse la confirmation entière en `400` du même code, `details.paymentBatchId` et `details.supplierInvoiceId` en plus, si le compte bancaire a été relié au compte créanciers depuis la création du lot (le lot reste `generated` : relier le compte bancaire à son propre compte de banque puis confirmer, ou annuler le lot). *(Depuis la v0.13.0.)*

Refus de l'annulation : `SUPPLIER_INVOICE_NOT_PAID` (`409`, la facture n'est pas payée), `FISCAL_YEAR_CLOSED` (`409`), `ACCOUNT_ARCHIVED` (`400`, `details.rejected[]`), `FISCAL_YEAR_INVALID` (`400`), `LATER_FISCAL_YEAR_CLOSED` (`400`, l'exercice du jour est suivi d'un exercice clôturé — *depuis la v0.13.0*), `PERIOD_LOCKED` (`400`), `NOT_FOUND` (`404`). ⚠️ **Angle mort assumé** : `settlementCancelBlockedBy` ne prédit pas `LATER_FISCAL_YEAR_CLOSED` ([#568](https://github.com/guycorbaz/kesh/issues/568)) — `settlementCancellable` peut valoir `true` et le clic être refusé.

### Annuler une facture fournisseur

**`POST /api/v1/supplier-invoices/{id}/cancel`** — écriture (`read-write`), ouverte aux clés comme `pay`. Sans corps. Annule la facture, **ouverte ou payée** : contre-passe l'écriture d'**achat** (datée du jour, liée par `reversesEntryId`) et passe la facture à `cancelled`. Payée, son **règlement reste au grand livre, détaché** : `settlementType`, `settlementJournalEntryId`, `paidAt`… reviennent à `null`, et l'écriture de règlement, qui n'appartient plus à aucune pièce, devient un paiement sans facture — contre-passable depuis sa fiche d'écriture. L'audit `supplier_invoice.cancelled` garde le lien (`settlementJournalEntryId`). Réponse : la facture relue, avec ses champs de lecture (Story 25-3-c).

`GET /supplier-invoices/{id}`, les réponses de `pay`, de l'annulation du règlement et de l'annulation de la facture portent aussi `cancellable`, `cancelBlockedBy` (code du motif) et `cancelBlockedLabel` (numéro du compte archivé) — même discipline : `null` = *non calculé*.

Refus, dans l'ordre de précédence : `SUPPLIER_INVOICE_CANCELLED` (`409`, déjà annulée), `FISCAL_YEAR_CLOSED` (`409`, exercice de l'**achat** clôturé), `ACCOUNT_ARCHIVED` (`400`, `details.rejected[]`), `FISCAL_YEAR_INVALID` (`400`, aucun exercice ouvert le jour), `SUPPLIER_INVOICE_IN_PAYMENT_BATCH` (`409`, facture dans un lot de paiement en cours — dernier des motifs du geste), `LATER_FISCAL_YEAR_CLOSED` (`400`, l'exercice du jour est suivi d'un exercice clôturé — *depuis la v0.13.0* ; non prédit par `cancelBlockedBy`, angle mort assumé, [#568](https://github.com/guycorbaz/kesh/issues/568)), `PERIOD_LOCKED` (`400`), `NOT_FOUND` (`404`). ⚠️ **Changement** : une facture **payée** était refusée (`409 ILLEGAL_STATE_TRANSITION`) ; une facture ouverte dont l'achat est dans un exercice clôturé était annulée, elle est désormais refusée.

⚠️ **`FISCAL_YEAR_CLOSED` rend ici `409`**, alors que la dévalidation le rend en `400` : c'est un refus du **geste** d'annulation, qui porte sur l'exercice du **règlement** ; la contre-passation, elle, serait datée d'un exercice ouvert.

### Accepter des propositions de rapprochement

**`POST /api/v1/reconciliation/accept`** — écriture (`read-write`), rôle Comptable, ouverte aux clés comme les autres routes de réconciliation ; la clé est nommée au journal d'audit (`reconciliation.accepted`). Corps : `{ bankAccountId, proposals: [...] }`, chaque proposition portant `type` (`invoice`, `split` ou `rule`) et `bankTransactionId` — plus `invoiceId` pour une facture. Les propositions viennent de `GET /api/v1/reconciliation/proposals`, où `invoiceAmount` est le **reste dû** de la facture (le TTC tant que rien n'est réglé) et `invoiceTotalTtc` son total d'origine, présent seulement quand les deux diffèrent.

La réponse est un **succès partiel** en `200` : `{ accepted: [...], failed: [{ bankTransactionId, errorCode, details }] }` — une proposition refusée n'empêche pas les autres. Pour une facture, le score est recalculé par le serveur, et un montant supérieur au reste dû est refusé (`RECONCILIATION_OVERPAYMENT`, jamais écrit). Un compte de l'écriture qui serait **le compte débiteurs de la facture** est refusé, lui aussi par proposition, en `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT` — `details.role` vaut `counterparty` (le compte bancaire est lié au compte débiteurs ; `details.bankAccountId` est posé, et ce refus précède le trop-perçu) ou `rounding` (le compte de différences d'arrondi des réglages est le compte débiteurs ; sans `bankAccountId`) *(depuis la v0.13.0)*. Si la facture change **pendant** l'acceptation — un règlement, un avoir, une dévalidation enregistrés au même moment —, la proposition est refusée en `RECONCILIATION_INVOICE_NOT_ELIGIBLE` (`details.reason` = `race_during_update`) sans rien écrire : relire les propositions et recommencer. Un interblocage transitoire avec une autre opération est **rejoué** par le serveur, sans être montré ; s'il persiste après trois tentatives, la requête finit en `500 INTERNAL_ERROR` et peut être renvoyée telle quelle — rien n'a été écrit. Refus de la requête entière : `400` (corps invalide), `404` (compte bancaire inconnu), `409 RECONCILIATION_ACCOUNT_LOCKED` (une autre acceptation tient le compte).

Une proposition datée dans un exercice **suivi d'un exercice clôturé** (données antérieures à la v0.13.0) est refusée dans `failed[]` en `LATER_FISCAL_YEAR_CLOSED`, avec `details: { fiscalYearId, fiscalYearName }` — l'exercice clôturé le plus proche — et `projectId` quand le projet de la facture est connu (jamais pour `split` ni `rule`) *(depuis la v0.13.0)*. Le rapprochement manuel et ventilé rendent le même refus en `400`.

Pour une proposition `split` ou `rule`, un compte de contrepartie **non imputable** (compte de regroupement, de résultat ou de clôture) est refusé dans `failed[]` en `ACCOUNT_NOT_POSTABLE`, avec `details.rejected[{accountId, accountNumber}]` — la forme du `400` du § 10. Un compte inconnu, d'une autre company ou archivé reste `ACCOUNT_NOT_FOUND` et **prime**. Une règle dont le compte est devenu non imputable n'est plus proposée par `GET /api/v1/reconciliation/proposals` *(depuis la v0.13.0)*.

Une contrepartie qui est **le compte comptable du compte bancaire** lui-même — l'écriture `D banque / C banque` serait nulle — est refusée dans `failed[]` en `VALIDATION_ERROR` avec `details.reason = "counterparty_equals_bank_ledger"`, pour une proposition `split` (une ligne sur ce compte) comme pour une proposition `rule` *(pour `rule`, depuis la v0.13.0)*. Ce refus vient **avant** `ACCOUNT_NOT_FOUND` et `ACCOUNT_NOT_POSTABLE` : un compte de banque devenu non imputable, pris pour contrepartie, rend `VALIDATION_ERROR`. Une règle qui vise ce compte n'est pas proposée par `GET /api/v1/reconciliation/proposals` *(depuis la v0.13.0)*. Si le compte comptable lié au compte bancaire est **archivé**, une proposition `split` ou `rule` est refusée en `BANK_ACCOUNT_NOT_CONFIGURED` avec `details.bankAccountId` — reliez le compte bancaire à un compte actif. ⚠️ **Changement** *(depuis la v0.13.0)* : pour une proposition `rule`, ce refus rendait auparavant `DATABASE_ERROR` (règle ordinaire) ou `ACCOUNT_NOT_FOUND` (règle sur le compte de banque).

Le rapprochement manuel (**`POST /api/v1/reconciliation/manual`**) et ventilé (**`POST /api/v1/reconciliation/split`**) refusent une contrepartie égale au compte de la banque en `400 VALIDATION_ERROR` — message seul, sans `details.reason` *(pour `manual`, depuis la v0.13.0 ; auparavant l'écriture nulle était passée)* —, **avant** le `404 ACCOUNT_NOT_FOUND`, le non-imputable et l'état de la transaction. Ils refusent de même un compte de contrepartie non imputable, en `400 ACCOUNT_NOT_POSTABLE` qui nomme tous les comptes refusés de la requête — après le `404 ACCOUNT_NOT_FOUND`, qui prime *(depuis la v0.13.0)*.

### Annuler un rapprochement bancaire

**`GET /api/v1/reconciliation/transactions/{id}`** — lecture, rôle Comptable (comme les propositions). La transaction bancaire (mêmes champs que dans le détail d'un import, dont `matchedEntryId`), `kind` (`invoice_settlement` : le rapprochement a réglé une facture client ; `entry` : une écriture que seule la transaction possède — éclatement, règle, rapprochement manuel ; `null` : la transaction n'est pas rapprochée), `invoiceId`, `invoiceNumber`, et **`cancellable`** — calculé par la fonction même qui refuserait l'annulation, lu **dans un seul instantané**. Quand il vaut `false`, `cancelBlockedBy` porte le code du motif, `cancelBlockedLabel` le numéro du compte archivé et `cancelBlockedDocumentId` l'**autre** transaction qui pointe la même écriture. ⚠️ Calculé pour **une** transaction : le détail d'un import ne le porte pas. ⚠️ **Angle mort assumé** : `cancellable` ne voit pas qu'un exercice postérieur à l'exercice du jour est clôturé ([#568](https://github.com/guycorbaz/kesh/issues/568)) — le clic rend alors `400 LATER_FISCAL_YEAR_CLOSED`.

**`POST /api/v1/reconciliation/transactions/{id}/cancel`** — écriture (`read-write`), ouverte aux clés comme les autres routes de réconciliation ; la clé est nommée au journal d'audit (`reconciliation.cancelled`). Sans corps. Défait le lien, contre-passe l'écriture du rapprochement (**datée du jour**) et, pour une facture, retire son règlement ; la transaction revient `pending` et réapparaît dans `GET /reconciliation/proposals`. Réponse : `{ bankTransaction, reversalJournalEntryId, invoiceId }`. Un interblocage transitoire avec une autre opération est **rejoué** par le serveur, sans être montré.

| Refus | Code | Statut |
|---|---|---|
| La transaction n'est pas rapprochée | `BANK_TRANSACTION_NOT_RECONCILED` | `409` |
| Facture créditée par un avoir — son règlement reste ouvert au compte débiteurs | `INVOICE_CREDITED` | `409` |
| Écriture du rapprochement dans un exercice **clos** — l'exercice du **paiement**, jamais celui de la facture | `FISCAL_YEAR_CLOSED` | `409` |
| Une autre transaction pointe la même écriture | `MATCHED_BANK_TRANSACTION` | `409` |
| Compte de l'écriture archivé | `ACCOUNT_ARCHIVED` | `400`, `details.rejected[]` nomme les comptes |
| Aucun exercice ouvert ne couvre la date du jour | `FISCAL_YEAR_INVALID` | `400` |
| L'exercice du jour est suivi d'un exercice clôturé — la contre-passation y changerait son bilan *(depuis la v0.13.0)* | `LATER_FISCAL_YEAR_CLOSED` | `400`, `details.fiscalYearId` / `fiscalYearName` |
| Date du jour dans une période verrouillée | `PERIOD_LOCKED` | `400` |
| Compte bancaire en cours de réconciliation par une autre opération | `RECONCILIATION_ACCOUNT_LOCKED` | `409` |
| Transaction introuvable (ou d'une autre société) | `NOT_FOUND` | `404` |

**`GET /api/v1/bank-imports/{id}`** : chaque transaction porte désormais `matchedEntryId`, l'écriture liée par son rapprochement (`null` sinon).

² **Deux opérations sur les factures sont réservées à l'interface web** : `DELETE /invoices/{id}` et `POST /invoices/{id}/reminders/{reminderId}/cancel` (annulation d'un rappel) sont des routes d'administration, donc fermées aux clés (`403 API_KEY_ADMIN_FORBIDDEN`, cf. §4). Tout le reste du cycle de facturation reste ouvert.

⚠️ **`DELETE /invoices/{id}` ne supprime plus que des BROUILLONS.** Sur une facture validée, elle rend `409 INVOICE_MUST_BE_UNVALIDATED_FIRST` : dévalidez-la d'abord (`POST /invoices/{id}/unvalidate`, ci-dessous), ce qui la ramène au brouillon et supprime son écriture comptable. **Cette route-là, elle, est ouverte aux clés en écriture** — l'asymétrie est voulue : dévalider est un geste de facturation réversible, effacer ne l'est pas.

³ **Changer le `accountType` d'un compte qui porte des écritures exige une
confirmation explicite.** Sans elle, `PUT /accounts/{id}` répond **`409
ACCOUNT_HAS_ENTRIES`** et son `details` porte l'ampleur du reclassement :
`entryCount` (écritures concernées), `closedFiscalYears` (exercices **clos**
touchés), `fromType` et `toType`. Pour passer outre, renvoyer la même requête
avec `"confirmAccountRetype": true`. Le champ est facultatif et vaut `false`
s'il est omis — un client existant n'a donc rien à changer tant qu'il ne retype
pas un compte mouvementé. ⚠️ Ce refus est **neuf** : la requête aboutissait
auparavant sans avertir, alors qu'elle reclasse rétroactivement tout
l'historique du compte, exercices clos compris.

¹ **Les mutations de taux de TVA ne sont pas accessibles via l'API** :
`POST /vat-rates`, `PUT /vat-rates/{id}` et `DELETE /vat-rates/{id}` sont
réservées au rôle **Administrateur**, et l'administration est fermée aux clés
API quel que soit le rôle du créateur (`403 API_KEY_ADMIN_FORBIDDEN`, cf. §4).
Ces opérations se font dans l'interface web. La **lecture** reste ouverte à
toute clé.

---

## 8. Sécurité & bonnes pratiques

- **Ne committez jamais une clé** dans un dépôt de code, un fichier de config versionné ou un ticket. Stockez-la comme un secret (variable d'environnement, gestionnaire de secrets).
- **Préférez une expiration.** Une clé sans expiration reste valide jusqu'à révocation — c'est plus exposé qu'une session web (qui expire en quelques minutes).
- **Principe du moindre privilège** :
  - utilisez une clé **`read`** dès que la lecture suffit (analyse, reporting, IA d'observation) ;
  - faites créer la clé par un utilisateur au **rôle le plus restreint** possible.
- ✅ **Une clé créée par un Administrateur n'hérite PAS de ses pouvoirs d'administration.** Aucune route réservée au rôle Administrateur n'est atteignable avec une clé API (`403 API_KEY_ADMIN_FORBIDDEN`, cf. §4) — de sorte que **révoquer une clé compromise suffit à arrêter l'incident**. Appliquez néanmoins le moindre privilège pour ce que la clé peut faire : préférez la portée `read` quand la lecture suffit, et un créateur Comptable quand l'intégration n'a pas besoin d'écrire au-delà.
- **Révoquez immédiatement** toute clé suspectée compromise : l'effet est instantané. Désactiver le compte créateur invalide également toutes ses clés.

---

## 8 bis. Endpoints utilisateurs — sémantique du `PUT`

> ⚠️ **Cette section ne concerne PLUS les clés API.** `PUT /api/v1/users/:id` est une route d'administration, donc fermée aux clés (`403 API_KEY_ADMIN_FORBIDDEN`, cf. §4) : ce qui suit vaut pour l'interface web et pour toute intégration en session, pas pour un client à clé. Conservé ici parce que la sémantique de remplacement, elle, n'a pas changé — et que la même règle s'applique aux factures, qui restent accessibles par clé.

⚠️ **`PUT /api/v1/users/:id` a une sémantique de REMPLACEMENT, pas de fusion partielle.** Tout champ optionnel absent du corps JSON est **réinitialisé** côté serveur. En particulier le champ `email` (utilisé par la récupération de mot de passe self-service) : un `PUT` qui envoie seulement `{ "role": …, "active": …, "version": … }` **efface l'adresse email** de l'utilisateur — qui ne pourra plus réinitialiser son mot de passe par email.

Bonne pratique **dans l'interface web ou pour une intégration en session** : lire l'utilisateur (`GET /api/v1/users`), puis renvoyer **tous les champs** dans le `PUT`, y compris `email` (la valeur courante si inchangée, ou `null` pour effacer délibérément).

La même sémantique de remplacement s'applique à **`PUT /api/v1/invoices/:id`** (facture brouillon) : un corps qui omet `projectId` **efface le tag analytique** de la facture (Epic 19). Relisez la facture (`GET /api/v1/invoices/:id`) et renvoyez `projectId` (valeur courante ou `null` pour détaguer délibérément).

> **Note recovery** : les endpoints publics de récupération de mot de passe (`POST /api/v1/auth/forgot-password`, `POST /api/v1/auth/reset-password`) ne sont **pas** des endpoints PAT — ils sont anonymes (pré-connexion). Et **aucune clé API ne peut réinitialiser un mot de passe** : `PUT /api/v1/users/:id/reset-password` est une route d'administration, fermée aux clés (`403 API_KEY_ADMIN_FORBIDDEN`, cf. §4). Cette opération se fait dans l'interface web.

---

## 9. Limitations connues (v0.2)

| Limitation | Détail | Suivi |
|------------|--------|-------|
| **Portée binaire globale** | Pas de permissions fines par ressource (ex. `invoices:read` seul). Une clé est `read` ou `read-write` sur **toute** l'API de sa company. | [#100](https://github.com/guycorbaz/kesh/issues/100) |
| **Pas de limitation de débit (*rate-limiting*) par clé** | Aucun plafond de requêtes par clé en v0.2. Mitigez via l'expiration et la révocation. | [#100](https://github.com/guycorbaz/kesh/issues/100) |
| **Gestion des clés réservée à l'UI web** | Lister/créer/révoquer une clé via l'API est interdit (`403 API_KEY_MANAGEMENT_FORBIDDEN`), même en `read-write` — protection anti-auto-propagation. | DC6 |
| **Administration réservée à l'interface web** | Les fonctions du rôle Administrateur ne sont pas exposées aux clés API (`403 API_KEY_ADMIN_FORBIDDEN`, cf. §4) — par conception, cf. la note ci-dessous. | — |
| **Pas de spécification OpenAPI** | Aucun schéma OpenAPI/Swagger n'est publié en v0.2 (la base de code n'embarque pas `utoipa`). Documentez vos appels à partir de ce guide. | v0.3 |

> ✅ **Corrigé — auto-propagation des clés Administrateur** ([KF-036 / #167](https://github.com/guycorbaz/kesh/issues/167)). Une clé `read-write` créée par un Administrateur atteignait auparavant les routes réservées aux Administrateurs : elle pouvait donc créer un compte administrateur, ce qui rendait la révocation de la clé inopérante. Ce n'est plus le cas. **Si une intégration existante appelait ces routes, elle reçoit désormais `403 API_KEY_ADMIN_FORBIDDEN`.**
>
> ⚠️ **Dans quelle version ?** Cette fermeture est livrée depuis la **v0.10.0** (entrée [#167](https://github.com/guycorbaz/kesh/issues/167) du [CHANGELOG](../CHANGELOG.md)). Jusqu'à la v0.9.0 incluse, une clé `read-write` créée par un Administrateur atteint les routes d'administration : sur une telle version, traitez une clé d'origine Administrateur comme un secret d'administrateur, et mettez Kesh à jour.

Hors périmètre (non planifié pour v0.2) : OAuth/SSO, webhooks, serveur MCP Kesh-natif (cf. `epic-17.md` — « Hors scope »).

---

## 10. Gestion des erreurs

Les erreurs sont renvoyées en JSON avec ce format :

```json
{ "error": { "code": "API_KEY_READ_ONLY", "message": "…" } }
```

> Le `message` est localisé selon la **langue configurée sur le serveur** (et non l'en-tête `Accept-Language` du client). Fiez-vous au champ `code`, stable, pour le traitement programmatique.

| HTTP | `code` | Cause |
|------|--------|-------|
| `401` | `UNAUTHENTICATED` | Clé absente, invalide, révoquée, expirée — ou créateur désactivé. |
| `403` | `API_KEY_READ_ONLY` | Méthode d'écriture (`POST`/`PUT`/`PATCH`/`DELETE`) avec une clé `read`. |
| `403` | `API_KEY_MANAGEMENT_FORBIDDEN` | Tentative de gérer des clés (`/api/v1/settings/api-keys`) **ou de consulter le journal d'audit** (`/api/v1/audit-log`, `/audit-log/vocabulary`, `/audit-log/export.csv`) via l'API. ⚠️ Le libellé du code parle de « gestion de clés » pour des raisons historiques : sur le journal d'audit, il signifie simplement **qu'une clé API n'y a pas accès**, quel que soit son scope. Ne cherchez pas le défaut dans votre configuration de clé. |
| `403` | `API_KEY_ADMIN_FORBIDDEN` | Route d'**administration** atteinte avec une clé API — quel que soit le rôle du créateur de la clé. Voir §4. |
| `400` | `VALIDATION_ERROR` | Corps de requête invalide (champ manquant, valeur hors limites, …). |
| `400` | `INACTIVE_OR_INVALID_ACCOUNTS` | Un compte référencé est inconnu, archivé, d'une autre company ou d'un mauvais type (par exemple un compte de charge qui n'est pas de charge) ; anti-énumération : le compte n'est pas nommé. |
| `400` | `ACCOUNT_NOT_POSTABLE` | Un compte de la company, **actif**, n'est **pas imputable** — compte de regroupement, de résultat ou de clôture. `details.rejected[{accountId, accountNumber}]` nomme chaque compte refusé, triés par numéro. Rendu par `POST /journal-entries`, `PUT /journal-entries/{id}` (depuis la v0.13.0 — un compte devenu non imputable est refusé même sur une ligne inchangée), `POST /opening-balances`, `POST /invoices/{id}/settlements` (compte interne), `POST /supplier-invoices` (compte de charge), `POST /imported-supplier-invoices/{id}/complete` (compte de charge), `POST /supplier-invoices/{id}/pay` (compte interne), `POST /reconciliation/manual` et `POST /reconciliation/split` (compte de contrepartie), `POST /reconciliation/rules` et `PATCH /reconciliation/rules/{id}` (compte de contrepartie — au `PATCH`, seulement si le compte change ou si la règle est réactivée), `POST /bank-accounts`, `PUT /bank-accounts/{id}` et `PATCH /bank-accounts/{id}` (compte comptable lié — au `PUT` et au `PATCH`, seulement s'il change). Rendu aussi, avec le même détail, par `POST /invoices/{id}/validate` (créance, ou TVA due si la facture en porte), `POST /supplier-invoices` et `POST /imported-supplier-invoices/{id}/complete` (compte créanciers, ou TVA récupérable si la facture en porte) quand un compte **désigné dans les réglages de facturation** est devenu non imputable depuis sa désignation : le message renvoie alors à *Paramètres → Facturation*, où un administrateur doit le remplacer (depuis la v0.13.0 ; ce refus vient après tous les autres, exercice compris). Dans `failed[]` de `POST /reconciliation/accept`, le même code et le même détail pour une proposition `split` ou `rule`. Un compte à la fois inconnu, archivé ou d'un mauvais type **et** non imputable rend `INACTIVE_OR_INVALID_ACCOUNTS`, et un compte archivé ou invalide sur une autre ligne de la même écriture prime lui aussi sur un compte non imputable. *(Depuis la v0.13.0 ; ces refus rendaient auparavant `INACTIVE_OR_INVALID_ACCOUNTS`.)* |
| `400` | `BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT` | `POST /bank-accounts`, `PUT /bank-accounts/{id}` et `PATCH /bank-accounts/{id}` (lien) : le compte comptable lié est le **compte débiteurs** ou le **compte créanciers** désigné dans les réglages de facturation — chaque encaissement ou paiement par ce compte bancaire écrirait une écriture nulle. `details` : `accountId`, `accountNumber`, `claim` (`receivable` \| `payable`). Au `PUT` et au `PATCH`, seulement si le compte lié **change** : un compte bancaire déjà lié ainsi reste modifiable dans ses autres champs. Vient après le `409` de version et après `ACCOUNT_NOT_POSTABLE`. *(Depuis la v0.13.0.)* |
| `400` | `LATER_FISCAL_YEAR_CLOSED` | `PUT` et `DELETE /journal-entries/{id}` : un exercice **postérieur** à celui de l'écriture est clôturé ; son bilan, cumulatif, reprend l'écriture. `POST /fiscal-years` : un exercice postérieur à la date de début demandée est clôturé — un exercice ne se crée pas avant un exercice clôturé (message propre à la création). `details.fiscalYearId` / `details.fiscalYearName` nomment le plus proche. Depuis la Story 15-12b (#543), aussi **toute création** d'écriture dans un tel exercice — `POST /journal-entries`, contre-passation, validation, règlement et dévalidation d'une facture, factures fournisseur, soldes de départ, rapprochements et leurs annulations — et, dans `failed[]` de `POST /reconciliation/accept`, la proposition concernée (`200`). ⚠️ Angles morts assumés : les champs qui annoncent un geste possible ne le prédisent pas — `cancellable` / `cancelBlockedBy` / `settlementCancelBlockedBy` des annulations ([#568](https://github.com/guycorbaz/kesh/issues/568)) et `canComplete` / `completeReason` de `GET /opening-balances/status` ; le refus vient alors au `POST`. *(Depuis la v0.13.0.)* |
| `409` | `EARLIER_FISCAL_YEAR_OPEN` | `POST /fiscal-years/{id}/close` : un exercice **antérieur** est encore ouvert — les exercices se clôturent dans l'ordre, le bilan étant cumulatif. `details.fiscalYearId` / `details.fiscalYearName` nomment le **plus ancien** antérieur ouvert, à clôturer d'abord. ⚠️ **Changement de contrat** : une intégration qui clôturait hors d'ordre reçoit désormais ce refus. Un exercice déjà clos rend `409 ILLEGAL_STATE_TRANSITION`, qui parle d'abord. *(Depuis la v0.13.0.)* |
| `409` | `ENTRY_LETTERED` | `PUT` et `DELETE /journal-entries/{id}` : une ligne de l'écriture est lettrée — délettrez-la d'abord (`DELETE /letterings/{key}`). `details.letteringCode` porte le code du premier groupe (ni `documentId` ni `documentNumber`). Rendu en dernier : quand il parle, le délettrage suffit. *(Depuis la v0.13.0.)* |
| `409` | `DETACHED_SUPPLIER_SETTLEMENT` | `PUT` et `DELETE /journal-entries/{id}` : l'écriture est le paiement d'une facture fournisseur annulée — une sortie de banque réelle, qui se corrige par contre-passation. `details.documentId` est l'identifiant de la facture. *(Depuis la v0.13.0.)* |
| `400` | `LETTERING_TOO_FEW_LINES`, `LETTERING_TOO_MANY_LINES` | `POST /letterings` : moins de deux lignes distinctes, ou plus de 200. *(Depuis la v0.13.0.)* |
| `409` | `LETTERING_ACCOUNTS_DIFFER`, `LETTERING_ACCOUNT_NOT_LETTERABLE`, `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`, `LETTERING_LINE_OWNED_BY_DOCUMENT`, `LETTERING_LINE_ALREADY_LETTERED`, `LETTERING_UNBALANCED`, `LETTERING_IS_DOCUMENT`, `LETTERING_CONCURRENT_CHANGE` | `POST` et `DELETE /letterings` : refus du lettrage — voir « Lettrer des lignes ». *(Depuis la v0.13.0.)* |
| `404` | `NOT_FOUND` | Ressource absente ou appartenant à une autre company (anti-énumération). Certaines ressources renvoient un code spécifique (ex. `ACCOUNT_NOT_FOUND`). |

**Interblocages : les écritures au journal sont rejouées.** Toute route qui écrit au journal comptable rejoue d'elle-même un interblocage transitoire avec une autre opération, sans le montrer. Ouvertes aux clés `read-write`, ce sont : `POST /journal-entries`, `PUT` et `DELETE /journal-entries/{id}`, `POST /journal-entries/{id}/reverse`, `POST /opening-balances`, `POST /opening-balances/complete`, `POST /invoices/{id}/validate`, `POST /invoices/{id}/unvalidate`, `POST /invoices/{id}/settlements`, `POST /invoices/{id}/settlements/{settlementId}/cancel`, `POST /invoices/{id}/write-off`, `POST /credit-notes`, `POST /supplier-invoices`, `POST /supplier-invoices/{id}/pay`, `POST /supplier-invoices/{id}/cancel`, `POST /supplier-invoices/{id}/settlement/cancel`, `POST /imported-supplier-invoices/{id}/complete`, `POST /payment-batches/{id}/confirm`, `POST /reconciliation/accept`, `POST /reconciliation/manual`, `POST /reconciliation/split`, `POST /reconciliation/transactions/{id}/cancel`, `POST /letterings` et `DELETE /letterings/{key}` — ainsi que la création et la clôture d'un exercice, `POST /fiscal-years` et `POST /fiscal-years/{id}/close`, qui n'écrivent pas au journal mais forment des interblocages avec la contre-passation (depuis la v0.13.0) (la restauration d'une sauvegarde, réservée à l'interface d'administration, ne rejoue pas ; l'effacement des données de démonstration, réservé lui aussi à l'interface d'administration, rejoue depuis la v0.13.0). Si l'interblocage persiste après trois tentatives, la requête finit en `500 INTERNAL_ERROR` ; rien n'a été écrit, et elle peut être renvoyée telle quelle.

Une autre route peut, rarement, rendre `500` sur un conflit transitoire d'accès concurrent ; rien n'est alors écrit et la requête peut être renvoyée.

---

## Voir aussi

- Manuel d'administration : section « Sécurité → Clés API (PAT) ».
- Issue d'origine : [#100](https://github.com/guycorbaz/kesh/issues/100).
