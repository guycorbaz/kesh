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

---

## 4. Portées (*scopes*)

| Portée | Méthodes HTTP autorisées | Usage |
|--------|--------------------------|-------|
| `read` | `GET`, `HEAD`, `OPTIONS` | Lecture seule. Toute tentative d'écriture → `403 API_KEY_READ_ONLY`. |
| `read-write` | Toutes les méthodes (`GET`, `HEAD`, `OPTIONS`, `POST`, `PUT`, `PATCH`, `DELETE`) | Lecture **et** modification, **sous réserve du rôle du créateur**. |

**La permission effective est l'intersection du rôle du créateur et de la portée de la clé.** Une clé `read-write` créée par un Comptable ne pourra pas faire ce que seul un Administrateur peut faire : la portée `read-write` ne *promeut* jamais le créateur. Appliquez le **principe du moindre privilège** (voir §8).

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

Les principales ressources accessibles via l'API (liste non exhaustive — toute route `/api/v1/*` de l'UI est consommable) :

| Ressource | Lecture (`read`) | Écriture (`read-write`) |
|-----------|------------------|--------------------------|
| Identité de la clé | `GET /auth/me` | — |
| Plan comptable | `GET /accounts` | `POST /accounts`, … |
| Contacts | `GET /contacts`, `GET /contacts/{id}` | `POST /contacts`, … |
| Produits | `GET /products`, `GET /products/{id}` | `POST /products`, … |
| Factures | `GET /invoices`, `GET /invoices/{id}` | `POST /invoices`, … |
| Écritures comptables | `GET /journal-entries`, `GET /journal-entries/{id}` | `POST /journal-entries`, … |
| Taux de TVA | `GET /vat-rates` | — |

*(Préfixe `…/api/v1` omis dans le tableau. Les corps de requête d'écriture peuvent différer des champs renvoyés en lecture : référez-vous aux formulaires correspondants de l'interface web pour les champs attendus.)*

---

## 8. Sécurité & bonnes pratiques

- **Ne committez jamais une clé** dans un dépôt de code, un fichier de config versionné ou un ticket. Stockez-la comme un secret (variable d'environnement, gestionnaire de secrets).
- **Préférez une expiration.** Une clé sans expiration reste valide jusqu'à révocation — c'est plus exposé qu'une session web (qui expire en quelques minutes).
- **Principe du moindre privilège** :
  - utilisez une clé **`read`** dès que la lecture suffit (analyse, reporting, IA d'observation) ;
  - faites créer la clé par un utilisateur au **rôle le plus restreint** possible.
- ⚠️ **Une clé créée par un Administrateur hérite des pouvoirs d'Administrateur** (création d'utilisateurs, réinitialisation de mots de passe, paramètres de facturation de la company). C'est une limitation connue de la v0.2 (voir §9, [KF-036 / #167](https://github.com/guycorbaz/kesh/issues/167)). **N'utilisez une clé d'origine Administrateur que si l'intégration en a réellement besoin.**
- **Révoquez immédiatement** toute clé suspectée compromise : l'effet est instantané. Désactiver le compte créateur invalide également toutes ses clés.

---

## 9. Limitations connues (v0.2)

| Limitation | Détail | Suivi |
|------------|--------|-------|
| **Portée binaire globale** | Pas de permissions fines par ressource (ex. `invoices:read` seul). Une clé est `read` ou `read-write` sur **toute** l'API de sa company. | [#100](https://github.com/guycorbaz/kesh/issues/100) |
| **Pas de limitation de débit (*rate-limiting*) par clé** | Aucun plafond de requêtes par clé en v0.2. Mitigez via l'expiration et la révocation. | [#100](https://github.com/guycorbaz/kesh/issues/100) |
| **Gestion des clés réservée à l'UI web** | Lister/créer/révoquer une clé via l'API est interdit (`403 API_KEY_MANAGEMENT_FORBIDDEN`), même en `read-write` — protection anti-auto-propagation. | DC6 |
| **Auto-propagation des clés Administrateur** | Une clé `read-write` créée par un Admin atteint les routes réservées aux Admins. | [KF-036 / #167](https://github.com/guycorbaz/kesh/issues/167) (v0.3) |
| **Pas de spécification OpenAPI** | Aucun schéma OpenAPI/Swagger n'est publié en v0.2 (la base de code n'embarque pas `utoipa`). Documentez vos appels à partir de ce guide. | v0.3 |

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
| `403` | `API_KEY_MANAGEMENT_FORBIDDEN` | Tentative de gérer des clés (`/api/v1/settings/api-keys`) via l'API. |
| `400` | `VALIDATION_ERROR` | Corps de requête invalide (champ manquant, valeur hors limites, …). |
| `404` | `NOT_FOUND` | Ressource absente ou appartenant à une autre company (anti-énumération). Certaines ressources renvoient un code spécifique (ex. `ACCOUNT_NOT_FOUND`). |

---

## Voir aussi

- Manuel d'administration : section « Sécurité → Clés API (PAT) ».
- Issue d'origine : [#100](https://github.com/guycorbaz/kesh/issues/100).
