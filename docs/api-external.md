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
| Taux de TVA | `GET /vat-rates` | — ¹ |

*(Préfixe `…/api/v1` omis dans le tableau. Les corps de requête d'écriture peuvent différer des champs renvoyés en lecture : référez-vous aux formulaires correspondants de l'interface web pour les champs attendus.)*

⁴ Depuis la v0.13.0 — voir « Modifier une écriture » et « Supprimer une écriture » ci-dessous.

### Modifier une écriture — `PUT /api/v1/journal-entries/{id}`

**Ouverte aux clés API** en écriture (`read-write`), comme la saisie (`POST /journal-entries`). *(Depuis la v0.13.0 ; de la v0.12.0 à la v0.12.1, ce `PUT` rendait toujours `409 ENTRY_IS_POSTED`.)*

Une écriture se modifie **tant que son exercice est ouvert**, qu'**aucun exercice postérieur n'est clôturé** (le bilan est cumulatif : il reprend l'écriture), qu'**aucune pièce ne la possède** et que sa date — l'ancienne comme la nouvelle — est **postérieure** à la période verrouillée. En pratique : les écritures saisies à la main, l'écriture d'ouverture et les compléments de soldes de départ. Le numéro, l'exercice et la date de création **ne changent jamais** ; la nouvelle date doit rester dans l'exercice **de l'écriture**.

Corps : celui du `POST`, plus la version lue — `{ "entryDate", "journal", "description", "lines": [{ "accountId", "debit", "credit", "projectId"? }], "version" }`. Réponse : `200` et l'écriture, même forme que le `POST`. Un corps identique à l'état présent rend `200` sans rien écrire.

Chaque modification effective est **tracée** au journal d'audit (`journal_entry.updated`), avec l'état **avant et après**, lignes comprises — et **la clé** qui l'a faite, quand c'est une clé.

`GET /journal-entries/{id}` porte de quoi décider avant d'essayer : `modifiable`, et, quand il vaut `false`, `modificationBlockedBy` (le motif) et `modificationBlockedLabel` (numéro de pièce, nom de l'exercice postérieur clos ou borne du verrou). ⚠️ Ce motif ne voit que l'état **présent** : le corps, la nouvelle date et la version ne sont contrôlés qu'au `PUT`.

Refus, **dans l'ordre où ils parlent** — 15 lignes :

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

Les refus `409` d'une pièce portent `details.documentId` (l'identifiant de la pièce — pour `DETACHED_SUPPLIER_SETTLEMENT`, celui de la facture fournisseur annulée) et `details.documentNumber` (son numéro, quand elle en a un). `LATER_FISCAL_YEAR_CLOSED` porte `details.fiscalYearId` et `details.fiscalYearName` — le plus proche exercice postérieur clos.

⚠️ **`LATER_FISCAL_YEAR_CLOSED` ne garde aujourd'hui que ce `PUT` et la suppression (`DELETE`, ci-dessous).** La saisie (`POST /journal-entries`, clés comprises), la contre-passation, les règlements, la dévalidation et le complément des soldes de départ acceptent encore une écriture dans un exercice ouvert alors qu'un exercice postérieur est clôturé — dont le bilan reporté change alors. Limite connue, suivie par l'issue [#543](https://github.com/guycorbaz/kesh/issues/543).

⚠️ **Une écriture refusée se corrige par contre-passation** (`POST /journal-entries/{id}/reverse`), sauf l'écriture d'une pièce, qui se corrige depuis sa pièce. ⚠️ **Une écriture sur un compte archivé ou devenu non imputable reste modifiable** — à condition de remplacer ce compte : l'enregistrement refuse tant qu'une ligne le vise. ⚠️ Un interblocage avec une écriture concurrente est rejoué par le serveur ; s'il persiste, la réponse est un `500` — réessayez.

### Supprimer une écriture — `DELETE /api/v1/journal-entries/{id}`

**Ouverte aux clés API** en écriture (`read-write`). *(Depuis la v0.13.0 ; de la v0.12.0 à la v0.12.1, ce `DELETE` rendait toujours `409 ENTRY_IS_POSTED`, code **retiré** depuis.)*

Une écriture se supprime **dans le même cadre qu'elle se modifie** : exercice ouvert, aucun exercice postérieur clôturé, aucune pièce, pas une contre-passation ni une écriture contre-passée, pas le paiement d'une facture fournisseur annulée, date **postérieure** à la période verrouillée. Le motif de `GET /journal-entries/{id}` (`modifiable`, `modificationBlockedBy`) vaut aussi pour la suppression. Réponse : `204`, sans corps.

La suppression est **tracée** au journal d'audit (`journal_entry.deleted`), avec le **contenu complet** de l'écriture, lignes comprises — et **la clé** qui l'a faite, quand c'est une clé. ⚠️ **Le numéro n'est jamais réattribué** : la suppression laisse un trou dans la numérotation de l'exercice, que la trace d'audit explique.

Refus, **dans l'ordre où ils parlent** — 9 lignes :

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

Les `details` sont ceux du `PUT` (`documentId`, `documentNumber` ; `fiscalYearId`, `fiscalYearName`). ⚠️ Une écriture d'une pièce sous la période verrouillée répond par **sa pièce** (`409`), pas par `PERIOD_LOCKED` : c'est la pièce qui dit où corriger. ⚠️ Un interblocage est rejoué par le serveur ; s'il persiste, la réponse est un `500` — réessayez.

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

⛔ Les **cinq autres** codes du tableau des refus — 10 lignes en tout, 5 avec `details` et 5 sans — `FISCAL_YEAR_CLOSED`, `ENTRY_IS_REVERSED`, `PERIOD_LOCKED`, `OPTIMISTIC_LOCK_CONFLICT`, `ILLEGAL_STATE_TRANSITION` — n'émettent **aucun** `details`. Un message du type « bloquée par le document {documentNumber} » opposerait donc la facture à elle-même dans trois cas sur cinq.

⚠️ **« Envoyée au client » est un refus sec** : il ne se lève par aucune confirmation. Une facture que le client détient se corrige par un **avoir**. La garde ne connaît que ce que Kesh a envoyé lui-même — un PDF téléchargé puis transmis à la main ne laisse aucune trace.

⚠️ **Un brouillon qui porte déjà un numéro ne change pas d'exercice** : `PUT /invoices/{id}` refuse une date hors de l'exercice qui a émis le numéro (`INVOICE_NUMBER_FISCAL_YEAR_MISMATCH`, `409`).

### Lister et annuler les règlements d'une facture

**`GET /api/v1/invoices/{id}/settlements`** — lecture (`read` suffit). Les règlements de la facture, du plus ancien au plus récent : `id`, `journalEntryId`, `amount`, `settledOn`, `settlementType` (`bank_transfer` / `internal_account` / `write_off` — un solde, ci-dessous), `writeOffNature` (la nature d'un solde, `null` pour un règlement), et **`cancellable`** — calculé par la fonction même qui refuserait l'annulation. Quand il vaut `false`, `cancelBlockedBy` porte le code du motif, `cancelBlockedLabel` le numéro du compte archivé et `cancelBlockedDocumentId` l'identifiant de la transaction bancaire rapprochée. ⚠️ `cancellable` ne dit rien des **droits** de la clé : une clé `read` lit `true` et reçoit `403` à l'annulation.

**`POST /api/v1/invoices/{id}/settlements/{settlementId}/cancel`** — écriture (`read-write`), ouverte aux clés comme `POST /invoices/{id}/settlements`. Sans corps. Annule le règlement par **contre-passation** : une écriture inverse **datée du jour**, dans l'exercice ouvert qui le couvre ; la ligne de règlement est retirée ; `paidAt` retombe à `null` si le reste dû redevient positif. Réponse : `{ invoice, reversalJournalEntryId }`, la facture relue.

| Refus | Code | Statut |
|---|---|---|
| Facture créditée par un avoir — le règlement est un paiement **à lettrer** | `INVOICE_CREDITED` | `409` |
| Un **solde** existe sur la facture — annuler d'abord le solde (le solde lui-même reste annulable) | `INVOICE_WRITTEN_OFF` | `409` |
| Règlement d'un exercice **clos** — un administrateur doit le rouvrir | `FISCAL_YEAR_CLOSED` | `409` |
| Règlement rapproché d'une transaction bancaire | `MATCHED_BANK_TRANSACTION` | `409`, `details.documentId` = la transaction |
| Compte du règlement archivé | `ACCOUNT_ARCHIVED` | `400`, `details.rejected[]` nomme les comptes |
| Aucun exercice ouvert ne couvre la date du jour | `FISCAL_YEAR_INVALID` | `400` |
| Date du jour dans une période verrouillée | `PERIOD_LOCKED` | `400` |
| Facture ou règlement introuvable (ou d'une autre société) | `NOT_FOUND` | `404` |

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
| Aucun exercice ouvert ne couvre la date | `FISCAL_YEAR_INVALID` | `400` |
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

Refus : `SUPPLIER_INVOICE_NOT_PAID` (`409`, la facture n'est pas payée), `FISCAL_YEAR_CLOSED` (`409`), `ACCOUNT_ARCHIVED` (`400`, `details.rejected[]`), `FISCAL_YEAR_INVALID` (`400`), `PERIOD_LOCKED` (`400`), `NOT_FOUND` (`404`).

### Annuler une facture fournisseur

**`POST /api/v1/supplier-invoices/{id}/cancel`** — écriture (`read-write`), ouverte aux clés comme `pay`. Sans corps. Annule la facture, **ouverte ou payée** : contre-passe l'écriture d'**achat** (datée du jour, liée par `reversesEntryId`) et passe la facture à `cancelled`. Payée, son **règlement reste au grand livre, détaché** : `settlementType`, `settlementJournalEntryId`, `paidAt`… reviennent à `null`, et l'écriture de règlement, qui n'appartient plus à aucune pièce, devient un paiement sans facture — contre-passable depuis sa fiche d'écriture. L'audit `supplier_invoice.cancelled` garde le lien (`settlementJournalEntryId`). Réponse : la facture relue, avec ses champs de lecture (Story 25-3-c).

`GET /supplier-invoices/{id}`, les réponses de `pay`, de l'annulation du règlement et de l'annulation de la facture portent aussi `cancellable`, `cancelBlockedBy` (code du motif) et `cancelBlockedLabel` (numéro du compte archivé) — même discipline : `null` = *non calculé*.

Refus, dans l'ordre de précédence : `SUPPLIER_INVOICE_CANCELLED` (`409`, déjà annulée), `FISCAL_YEAR_CLOSED` (`409`, exercice de l'**achat** clôturé), `ACCOUNT_ARCHIVED` (`400`, `details.rejected[]`), `FISCAL_YEAR_INVALID` (`400`, aucun exercice ouvert le jour), `SUPPLIER_INVOICE_IN_PAYMENT_BATCH` (`409`, facture dans un lot de paiement en cours — en dernier), `PERIOD_LOCKED` (`400`), `NOT_FOUND` (`404`). ⚠️ **Changement** : une facture **payée** était refusée (`409 ILLEGAL_STATE_TRANSITION`) ; une facture ouverte dont l'achat est dans un exercice clôturé était annulée, elle est désormais refusée.

⚠️ **`FISCAL_YEAR_CLOSED` rend ici `409`**, alors que la dévalidation le rend en `400` : c'est un refus du **geste** d'annulation, qui porte sur l'exercice du **règlement** ; la contre-passation, elle, serait datée d'un exercice ouvert.

### Accepter des propositions de rapprochement

**`POST /api/v1/reconciliation/accept`** — écriture (`read-write`), rôle Comptable, ouverte aux clés comme les autres routes de réconciliation ; la clé est nommée au journal d'audit (`reconciliation.accepted`). Corps : `{ bankAccountId, proposals: [...] }`, chaque proposition portant `type` (`invoice`, `split` ou `rule`) et `bankTransactionId` — plus `invoiceId` pour une facture. Les propositions viennent de `GET /api/v1/reconciliation/proposals`, où `invoiceAmount` est le **reste dû** de la facture (le TTC tant que rien n'est réglé) et `invoiceTotalTtc` son total d'origine, présent seulement quand les deux diffèrent.

La réponse est un **succès partiel** en `200` : `{ accepted: [...], failed: [{ bankTransactionId, errorCode, details }] }` — une proposition refusée n'empêche pas les autres. Pour une facture, le score est recalculé par le serveur, et un montant supérieur au reste dû est refusé (`RECONCILIATION_OVERPAYMENT`, jamais écrit). Si la facture change **pendant** l'acceptation — un règlement, un avoir, une dévalidation enregistrés au même moment —, la proposition est refusée en `RECONCILIATION_INVOICE_NOT_ELIGIBLE` (`details.reason` = `race_during_update`) sans rien écrire : relire les propositions et recommencer. Un interblocage transitoire avec une autre opération est **rejoué** par le serveur, sans être montré ; s'il persiste après trois tentatives, la requête finit en `500 INTERNAL_ERROR` et peut être renvoyée telle quelle — rien n'a été écrit. Refus de la requête entière : `400` (corps invalide), `404` (compte bancaire inconnu), `409 RECONCILIATION_ACCOUNT_LOCKED` (une autre acceptation tient le compte).

Pour une proposition `split` ou `rule`, un compte de contrepartie **non imputable** (compte de regroupement, de résultat ou de clôture) est refusé dans `failed[]` en `ACCOUNT_NOT_POSTABLE`, avec `details.rejected[{accountId, accountNumber}]` — la forme du `400` du § 10. Un compte inconnu, d'une autre company ou archivé reste `ACCOUNT_NOT_FOUND` et **prime**. Une règle dont le compte est devenu non imputable n'est plus proposée par `GET /api/v1/reconciliation/proposals` *(depuis la v0.13.0)*.

Le rapprochement manuel (**`POST /api/v1/reconciliation/manual`**) et ventilé (**`POST /api/v1/reconciliation/split`**) refusent de même un compte de contrepartie non imputable, en `400 ACCOUNT_NOT_POSTABLE` qui nomme tous les comptes refusés de la requête — après le `404 ACCOUNT_NOT_FOUND`, qui prime *(depuis la v0.13.0)*.

### Annuler un rapprochement bancaire

**`GET /api/v1/reconciliation/transactions/{id}`** — lecture, rôle Comptable (comme les propositions). La transaction bancaire (mêmes champs que dans le détail d'un import, dont `matchedEntryId`), `kind` (`invoice_settlement` : le rapprochement a réglé une facture client ; `entry` : une écriture que seule la transaction possède — éclatement, règle, rapprochement manuel ; `null` : la transaction n'est pas rapprochée), `invoiceId`, `invoiceNumber`, et **`cancellable`** — calculé par la fonction même qui refuserait l'annulation, lu **dans un seul instantané**. Quand il vaut `false`, `cancelBlockedBy` porte le code du motif, `cancelBlockedLabel` le numéro du compte archivé et `cancelBlockedDocumentId` l'**autre** transaction qui pointe la même écriture. ⚠️ Calculé pour **une** transaction : le détail d'un import ne le porte pas.

**`POST /api/v1/reconciliation/transactions/{id}/cancel`** — écriture (`read-write`), ouverte aux clés comme les autres routes de réconciliation ; la clé est nommée au journal d'audit (`reconciliation.cancelled`). Sans corps. Défait le lien, contre-passe l'écriture du rapprochement (**datée du jour**) et, pour une facture, retire son règlement ; la transaction revient `pending` et réapparaît dans `GET /reconciliation/proposals`. Réponse : `{ bankTransaction, reversalJournalEntryId, invoiceId }`. Un interblocage transitoire avec une autre opération est **rejoué** par le serveur, sans être montré.

| Refus | Code | Statut |
|---|---|---|
| La transaction n'est pas rapprochée | `BANK_TRANSACTION_NOT_RECONCILED` | `409` |
| Facture créditée par un avoir — son règlement est un paiement **à lettrer** | `INVOICE_CREDITED` | `409` |
| Écriture du rapprochement dans un exercice **clos** — l'exercice du **paiement**, jamais celui de la facture | `FISCAL_YEAR_CLOSED` | `409` |
| Une autre transaction pointe la même écriture | `MATCHED_BANK_TRANSACTION` | `409` |
| Compte de l'écriture archivé | `ACCOUNT_ARCHIVED` | `400`, `details.rejected[]` nomme les comptes |
| Aucun exercice ouvert ne couvre la date du jour | `FISCAL_YEAR_INVALID` | `400` |
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
> ⚠️ **Dans quelle version ?** Ce correctif n'est **pas** dans la v0.9.0 : il figure sous **`[Unreleased]`** du [CHANGELOG](../CHANGELOG.md) et sera livré à la prochaine version publiée. Si vous exploitez la v0.9.0, **la faille y est encore ouverte** — traitez une clé d'origine Administrateur comme un secret d'administrateur.

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
| `400` | `LATER_FISCAL_YEAR_CLOSED` | `PUT` et `DELETE /journal-entries/{id}` : un exercice **postérieur** à celui de l'écriture est clôturé ; son bilan, cumulatif, reprend l'écriture. `details.fiscalYearId` / `details.fiscalYearName` nomment le plus proche. Ne garde pas encore les autres chemins d'écriture (#543). *(Depuis la v0.13.0.)* |
| `409` | `DETACHED_SUPPLIER_SETTLEMENT` | `PUT` et `DELETE /journal-entries/{id}` : l'écriture est le paiement d'une facture fournisseur annulée — une sortie de banque réelle, qui se corrige par contre-passation. `details.documentId` est l'identifiant de la facture. *(Depuis la v0.13.0.)* |
| `404` | `NOT_FOUND` | Ressource absente ou appartenant à une autre company (anti-énumération). Certaines ressources renvoient un code spécifique (ex. `ACCOUNT_NOT_FOUND`). |

**Interblocages : les écritures au journal sont rejouées.** Toute route qui écrit au journal comptable rejoue d'elle-même un interblocage transitoire avec une autre opération, sans le montrer. Ouvertes aux clés `read-write`, ce sont : `POST /journal-entries`, `PUT` et `DELETE /journal-entries/{id}`, `POST /journal-entries/{id}/reverse`, `POST /opening-balances`, `POST /opening-balances/complete`, `POST /invoices/{id}/validate`, `POST /invoices/{id}/unvalidate`, `POST /invoices/{id}/settlements`, `POST /invoices/{id}/settlements/{settlementId}/cancel`, `POST /invoices/{id}/write-off`, `POST /credit-notes`, `POST /supplier-invoices`, `POST /supplier-invoices/{id}/pay`, `POST /supplier-invoices/{id}/cancel`, `POST /supplier-invoices/{id}/settlement/cancel`, `POST /imported-supplier-invoices/{id}/complete`, `POST /payment-batches/{id}/confirm`, `POST /reconciliation/accept`, `POST /reconciliation/manual`, `POST /reconciliation/split` et `POST /reconciliation/transactions/{id}/cancel` (la restauration d'une sauvegarde et l'effacement des données de démonstration, réservés à l'interface d'administration, ne rejouent pas). Si l'interblocage persiste après trois tentatives, la requête finit en `500 INTERNAL_ERROR` ; rien n'a été écrit, et elle peut être renvoyée telle quelle.

Une autre route peut, rarement, rendre `500` sur un conflit transitoire d'accès concurrent ; rien n'est alors écrit et la requête peut être renvoyée.

---

## Voir aussi

- Manuel d'administration : section « Sécurité → Clés API (PAT) ».
- Issue d'origine : [#100](https://github.com/guycorbaz/kesh/issues/100).
